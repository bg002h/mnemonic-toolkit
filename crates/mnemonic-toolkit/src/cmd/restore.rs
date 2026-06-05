//! `mnemonic restore` — watch-only single-sig restore document.
//!
//! Takes secret seed material (`ms1`/`phrase`/`entropy`/`seedqr`) + an optional
//! BIP-39 passphrase and emits a watch-only "restore document" to facilitate
//! restoring a wallet on a PC: the document leads with the master fingerprint
//! (the passphrase-correctness oracle) + first receive address(es), then the
//! concrete single-sig descriptor(s) for bip44/49/84/86 (or a single
//! `--template`).
//!
//! Read-only public derivation: NO private keys reach stdout, NO signing
//! (`feedback_no_signing_read_only_derivation_boundary`). Derivation uses a
//! verification-only secp context and NEVER touches `account_xpriv`.
//!
//! Multisig restore is DEFERRED (SPEC §11 — `restore-multisig-cosigner-scope`).

use std::io::{Read, Write};

use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::Secp256k1;
use clap::Args;

use serde_json::json;

use crate::address_render::render_address_from_xpub;
use crate::cmd::convert::{
    parse_from_input, read_stdin_passphrase, read_stdin_to_string, script_type_from_template,
    NodeType,
};
use crate::cmd::export_wallet::CliExportFormat;
use crate::derive_slot::derive_bip32_from_entropy;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::synthesize::ResolvedSlot;
use crate::template::CliTemplate;
use crate::wallet_export::{
    self, build_descriptor_string, Bip388Emitter, BitcoinCoreEmitter, BsmsEmitter, BsmsForm,
    CheckedDescriptor, ColdcardEmitter, DescriptorEmitter, ElectrumEmitter, EmitInputs,
    GreenEmitter, JadeEmitter, SparrowEmitter, SpecterEmitter, TimestampArg, WalletFormatEmitter,
};

/// The four single-sig templates restore emits when no `--template` is given.
const ALL_SINGLE_SIG: [CliTemplate; 4] = [
    CliTemplate::Bip44,
    CliTemplate::Bip49,
    CliTemplate::Bip84,
    CliTemplate::Bip86,
];

/// `mnemonic restore` arguments.
#[derive(Args, Debug)]
pub struct RestoreArgs {
    /// Seed source: `ms1=<v>` | `phrase=<v>` | `entropy=<hex>` | `seedqr=<digits>`.
    /// Secret values support `@env:VAR` and `-` (stdin). Non-seed nodes
    /// (xpub/xprv/wif/…) are refused (restore needs a master secret).
    #[arg(long)]
    pub from: String,

    /// BIP-39 mnemonic-extension passphrase. `@env:VAR` supported; or
    /// `--passphrase-stdin`. Empty (default) = no passphrase.
    #[arg(long)]
    pub passphrase: Option<String>,

    /// Read the BIP-39 passphrase from stdin (conflicts with `--passphrase`).
    #[arg(long, conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// BIP-39 wordlist language for `phrase=`/`seedqr=` (default english).
    /// A `mnem` ms1 card carries its own wire language; supplying a conflicting
    /// `--language` is refused.
    #[arg(long, value_enum)]
    pub language: Option<CliLanguage>,

    /// Network (default mainnet).
    #[arg(long, value_enum)]
    pub network: Option<CliNetwork>,

    /// BIP-32 account index (default 0).
    #[arg(long, default_value_t = 0)]
    pub account: u32,

    /// Restrict to a single single-sig wallet type. Omit = all four
    /// (bip44/49/84/86). A multisig template is refused (restore is single-sig).
    #[arg(long, value_enum)]
    pub template: Option<CliTemplate>,

    /// Reference master fingerprint (8 lowercase hex). Mismatch → exit 4
    /// (unless `--allow-mismatch`).
    #[arg(long)]
    pub expect_fingerprint: Option<String>,

    /// Reference account xpub (requires `--template`). Mismatch → exit 4
    /// (unless `--allow-mismatch`).
    #[arg(long)]
    pub expect_xpub: Option<String>,

    /// Emit descriptors even when a reference does not match (loud banner, exit 0).
    #[arg(long)]
    pub allow_mismatch: bool,

    /// Number of first-receive addresses to show per wallet type (default 1).
    #[arg(long, default_value_t = 1)]
    pub count: u32,

    /// Emit an importable wallet-software payload (an `export-wallet` emitter:
    /// `descriptor`, `bitcoin-core`, `bip388`, `coldcard`, `sparrow`, …).
    /// REQUIRES a single `--template` (emitters are one-descriptor-in/one-out);
    /// `--format` with no `--template` (the all-4 default) → exit 2. When set,
    /// the importable PAYLOAD goes to stdout and the verification block
    /// (fingerprint / CONFIRM / descriptor / first recv) goes to stderr, so the
    /// payload pipes cleanly into wallet software. (With `--json`, the payload is
    /// embedded as the `import_payload` field instead.)
    #[arg(long, value_enum)]
    pub format: Option<CliExportFormat>,

    /// Emit a single structured JSON object on stdout instead of the text
    /// document. Seed material is NEVER echoed (redacted by construction). The
    /// `import_payload` field is present only when `--format` is also set.
    #[arg(long)]
    pub json: bool,

    /// Write the stdout content to `<FILE>` instead of standard output
    /// (`-`, the default, → stdout). The verification block / banners / advisory
    /// still go to stderr.
    #[arg(long, default_value = "-")]
    pub output: String,
}

fn bad(s: impl Into<String>) -> ToolkitError {
    ToolkitError::BadInput(s.into())
}

/// One derived wallet type: its template, concrete descriptor, and first
/// receive address(es). `slot` is the watch-only `ResolvedSlot` (entropy:
/// None) retained so a `--format` emitter can rebuild `EmitInputs` for the
/// single-template case.
struct WalletRow {
    template: CliTemplate,
    account_xpub: Xpub,
    descriptor: String,
    first_recv: Vec<String>,
    slot: ResolvedSlot,
}

/// Run `mnemonic restore`.
pub fn run<R: Read, W: Write, E: Write>(
    args: &RestoreArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    _no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    let from = parse_from_input(&args.from).map_err(bad)?;
    let from_uses_stdin = from.value == "-";

    // Seed-bearing nodes only — restore needs a master secret to derive from.
    if !matches!(
        from.node,
        NodeType::Ms1 | NodeType::Phrase | NodeType::Entropy | NodeType::Seedqr
    ) {
        return Err(bad(format!(
            "--from {} is not a seed source for restore (use ms1/phrase/entropy/seedqr)",
            from.node.as_str()
        )));
    }

    // Reject a multisig --template (restore is single-sig this cycle).
    if let Some(t) = args.template {
        if t.is_multisig() {
            return Err(bad(
                "restore is single-sig only; --template ∈ {bip44,bip49,bip84,bip86}",
            ));
        }
    }

    // `--expect-xpub` compares the per-template account xpub, which is only
    // unambiguous when a single `--template` is selected.
    if args.expect_xpub.is_some() && args.template.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--expect-xpub",
            message:
                "--expect-xpub requires --template <bip44|bip49|bip84|bip86> (the account xpub is per-type)",
        });
    }

    // `--format` drives a single `export-wallet` emitter — one descriptor in,
    // one payload out — so it cannot straddle the all-4 default. Require a single
    // `--template` (SPEC I-A: ModeViolation exit 2, NOT BadInput exit 1).
    if args.format.is_some() && args.template.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--format",
            message:
                "--format requires --template <bip44|bip49|bip84|bip86> (an importable payload is one descriptor — pick one type)",
        });
    }

    // Single-stdin-per-invocation guard (mirror convert / addresses).
    if args.passphrase_stdin && from_uses_stdin {
        return Err(bad(
            "--passphrase-stdin cannot coexist with --from <node>=- (a single stdin cannot serve both)",
        ));
    }

    // argv-leak advisories for inline secret-bearing values (mirror addresses scope).
    if !from_uses_stdin && !from.value.starts_with("@env:") {
        let node = args.from.split('=').next().unwrap_or("");
        crate::secret_advisory::secret_in_argv_warning(
            stderr,
            &format!("--from {node}="),
            &format!("--from {node}=-"),
        );
    }
    if let Some(pp) = args.passphrase.as_deref() {
        if !pp.starts_with("@env:") {
            crate::secret_advisory::secret_in_argv_warning(
                stderr,
                "--passphrase",
                "--passphrase-stdin",
            );
        }
    }

    // Effective BIP-39 passphrase (stdin / @env: / inline).
    let passphrase: String = if args.passphrase_stdin {
        read_stdin_passphrase(stdin)?
    } else {
        match args.passphrase.as_deref() {
            Some(p) => crate::env_sentinel::resolve_env_var_sentinel(p, "--passphrase")?,
            None => String::new(),
        }
    };
    let passphrase_applied = !passphrase.is_empty();

    // Resolved `--from` value (stdin / @env: / literal).
    let from_value: String = if from_uses_stdin {
        read_stdin_to_string(stdin)?
    } else {
        crate::env_sentinel::resolve_env_var_sentinel(&from.value, "--from")?
    };

    let network = args.network.unwrap_or(CliNetwork::Mainnet);

    // Resolve the seed node → (entropy, derive_language). For ms1, the `mnem`
    // wire language wins (refuse-on-`--language`-conflict, exit 2).
    let (entropy, derive_language): (zeroize::Zeroizing<Vec<u8>>, bip39::Language) = match from.node
    {
        NodeType::Ms1 => {
            let res = crate::slot_ms1::resolve_ms1_slot(&from_value, args.language, 0)?;
            (res.entropy, res.derive_language)
        }
        NodeType::Phrase => {
            let language = args.language.unwrap_or_default();
            let entropy = zeroize::Zeroizing::new(
                Mnemonic::parse_in(language.into(), &from_value)
                    .map_err(ToolkitError::Bip39)?
                    .to_entropy(),
            );
            (entropy, language.into())
        }
        NodeType::Seedqr => {
            let language = args.language.unwrap_or_default();
            let phrase = mnemonic_toolkit::seedqr::decode(&from_value)
                .map_err(|e| crate::cmd::seedqr::map_seedqr_error(e, "restore"))?;
            let entropy = zeroize::Zeroizing::new(
                Mnemonic::parse_in(language.into(), &phrase)
                    .map_err(ToolkitError::Bip39)?
                    .to_entropy(),
            );
            (entropy, language.into())
        }
        NodeType::Entropy => {
            let entropy = zeroize::Zeroizing::new(
                hex::decode(from_value.trim())
                    .map_err(|e| bad(format!("--from entropy= hex-decode: {e}")))?,
            );
            // No wordlist — language is irrelevant to derivation (english).
            (entropy, bip39::Language::English)
        }
        _ => unreachable!("seed-node guard above restricts to ms1/phrase/seedqr/entropy"),
    };

    // Pin the secret buffers for the remainder of the handler scope.
    let _pin_entropy = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
    let _pin_pp = if passphrase.is_empty() {
        None
    } else {
        Some(mnemonic_toolkit::mlock::pin_pages_for(passphrase.as_bytes()))
    };

    let templates: &[CliTemplate] = match &args.template {
        Some(t) => std::slice::from_ref(t),
        None => &ALL_SINGLE_SIG,
    };

    // Derive each selected single-sig type. The master fingerprint is
    // path-independent — identical across all four — so capture it once.
    let secp = Secp256k1::verification_only();
    let mut master_fingerprint: Option<Fingerprint> = None;
    let mut rows: Vec<WalletRow> = Vec::with_capacity(templates.len());

    for &template in templates {
        let acct = derive_bip32_from_entropy(
            &entropy,
            &passphrase,
            derive_language,
            network,
            template,
            args.account,
        )?;
        master_fingerprint = Some(acct.master_fingerprint);

        let script_type = script_type_from_template(template)
            .expect("single-sig template has a ScriptType (multisig rejected above)");

        // First receive address(es): m/0/i children of the account xpub, derived
        // with a verification-only secp (watch-only by construction).
        let mut first_recv = Vec::with_capacity(args.count as usize);
        for i in 0..args.count {
            let chain = ChildNumber::from_normal_idx(0).unwrap();
            let leaf = ChildNumber::from_normal_idx(i).map_err(|_| {
                bad(format!(
                    "address index {i} out of BIP-32 normal range (0..2147483647)"
                ))
            })?;
            let dp: DerivationPath = vec![chain, leaf].into();
            let child = acct
                .account_xpub
                .derive_pub(&secp, &dp)
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
            first_recv.push(render_address_from_xpub(&secp, &child, script_type, network));
        }

        // Concrete descriptor. The watch-only ResolvedSlot mirrors the
        // wallet_import watch-only ctor: all 7 fields spelled, no entropy.
        let slot = ResolvedSlot {
            xpub: acct.account_xpub,
            fingerprint: acct.master_fingerprint,
            path: acct.account_path.clone(),
            entropy: None,
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        };
        let descriptor =
            build_descriptor_string(template, std::slice::from_ref(&slot), 1, network, args.account, None)?;

        rows.push(WalletRow {
            template,
            account_xpub: acct.account_xpub,
            descriptor,
            first_recv,
            slot,
        });
        // NB: `acct` (and its `account_xpriv`) is dropped here — never emitted.
    }

    let master_fingerprint = master_fingerprint.expect("at least one template derived");
    let fp_str = master_fingerprint.to_string().to_lowercase();

    // ---- Verification gate (§3.4) -------------------------------------------
    // Compute the reference comparison (if any). `--expect-xpub` is gated to a
    // single `--template` above, so `rows[0]` is the only row when it is set.
    let mismatch: Option<(&'static str, String, String)> =
        if let Some(expected) = args.expect_fingerprint.as_deref() {
            let expected_norm = expected.trim().to_lowercase();
            if expected_norm != fp_str {
                Some(("fingerprint", fp_str.clone(), expected_norm))
            } else {
                None
            }
        } else if let Some(expected) = args.expect_xpub.as_deref() {
            let derived = rows[0].account_xpub.to_string();
            let expected = expected.trim().to_string();
            if expected != derived {
                Some(("xpub", derived, expected))
            } else {
                None
            }
        } else {
            None
        };

    let has_reference = args.expect_fingerprint.is_some() || args.expect_xpub.is_some();

    if let Some((reference, derived, expected)) = &mismatch {
        if !args.allow_mismatch {
            // Hard fail (exit 4) — no descriptors. The verify summary goes to
            // stderr; the typed error carries the derived-vs-expected detail.
            writeln!(stderr, "✗ MISMATCH").map_err(ToolkitError::Io)?;
            writeln!(
                stderr,
                "master fingerprint: {fp_str}  (passphrase: {})",
                if passphrase_applied { "applied" } else { "none" }
            )
            .map_err(ToolkitError::Io)?;
            return Err(ToolkitError::RestoreMismatch {
                reference,
                derived: derived.clone(),
                expected: expected.clone(),
                slot: None,
            });
        }
    }

    // Verification status label for the `--json` envelope (§3.5).
    let verification_status = if mismatch.is_some() {
        // Reached only with `--allow-mismatch` (the hard-fail path returned above).
        "overridden"
    } else if has_reference {
        "verified"
    } else {
        "unverified"
    };

    // ---- Importable payload (§3.5; Task 2.1) --------------------------------
    // `--format` is gated to a single `--template` above, so `rows[0]` is the
    // only row and the payload is one descriptor in / one payload out.
    let import_payload: Option<String> = if let Some(format) = args.format {
        Some(build_import_payload(format, &rows[0], network, args.account)?)
    } else {
        None
    };

    // ---- Compose the stdout content (§3.5) ----------------------------------
    // The "stdout content" is JSON (when `--json`), or the importable payload
    // alone (when `--format` without `--json`), or the text verification doc.
    // It is routed to `--output <FILE>` when set, else to stdout. The
    // verification block + banners + advisory always go to stderr.
    let stdout_content: String = if args.json {
        let mut verification = json!({ "status": verification_status });
        if let Some((reference, derived, expected)) = &mismatch {
            verification["reference"] = json!(reference);
            verification["derived"] = json!(derived);
            verification["expected"] = json!(expected);
        }
        let wallets: Vec<_> = rows
            .iter()
            .map(|row| {
                json!({
                    "wallet_type": row.template.human_name(),
                    "descriptor": row.descriptor,
                    "first_addresses": row.first_recv,
                })
            })
            .collect();
        // Seed material (the `--from` value, passphrase) is NEVER serialized —
        // the envelope carries only public derivation products. `passphrase_applied`
        // is a bool, not the passphrase itself.
        let mut envelope = json!({
            "master_fingerprint": fp_str,
            "passphrase_applied": passphrase_applied,
            "network": network.human_name(),
            "verification": verification,
            "wallets": wallets,
        });
        if let Some(payload) = &import_payload {
            envelope["import_payload"] = json!(payload);
        }
        let s = serde_json::to_string(&envelope)
            .map_err(|e| bad(format!("json serialization: {e}")))?;
        format!("{s}\n")
    } else if let Some(payload) = &import_payload {
        // `--format` without `--json`: the payload alone is stdout so it pipes
        // cleanly into wallet software; the verification doc goes to stderr.
        format!("{payload}\n")
    } else {
        // Phase-1 text document.
        let mut s = String::new();
        s.push_str(&format!(
            "master fingerprint: {fp_str}  (passphrase: {})\n",
            if passphrase_applied { "applied" } else { "none" }
        ));
        s.push_str(
            "CONFIRM: this fingerprint matches the wallet you are restoring before importing any descriptor.\n",
        );
        for row in &rows {
            s.push('\n');
            s.push_str(&format!("{}:\n", template_label(row.template)));
            s.push_str(&format!("  descriptor: {}\n", row.descriptor));
            for addr in &row.first_recv {
                s.push_str(&format!("  first recv: {addr}\n"));
            }
        }
        s
    };

    // When `--format` is set (and not `--json`), the human verification doc is
    // not the stdout content — surface it on stderr so the operator can still
    // confirm the fingerprint while the payload pipes onward.
    if import_payload.is_some() && !args.json {
        writeln!(
            stderr,
            "master fingerprint: {fp_str}  (passphrase: {})",
            if passphrase_applied { "applied" } else { "none" }
        )
        .map_err(ToolkitError::Io)?;
        writeln!(
            stderr,
            "CONFIRM: this fingerprint matches the wallet you are restoring before importing the payload above."
        )
        .map_err(ToolkitError::Io)?;
        for row in &rows {
            writeln!(stderr, "{}:", template_label(row.template)).map_err(ToolkitError::Io)?;
            writeln!(stderr, "  descriptor: {}", row.descriptor).map_err(ToolkitError::Io)?;
            for addr in &row.first_recv {
                writeln!(stderr, "  first recv: {addr}").map_err(ToolkitError::Io)?;
            }
        }
    }

    // ---- Route the stdout content (stdout | --output FILE) ------------------
    if args.output == "-" {
        write!(stdout, "{stdout_content}").map_err(ToolkitError::Io)?;
    } else {
        std::fs::write(&args.output, &stdout_content)
            .map_err(|e| bad(format!("--output {}: {e}", args.output)))?;
    }

    // ---- Verification banners (stderr) --------------------------------------
    if mismatch.is_some() {
        // Reached only with `--allow-mismatch` (the hard-fail path returned above).
        writeln!(
            stderr,
            "✗ MISMATCH (overridden): derived material does NOT match the supplied reference; \
             descriptors above were produced by the passphrase you provided, NOT the expected wallet"
        )
        .map_err(ToolkitError::Io)?;
    } else if !has_reference {
        writeln!(
            stderr,
            "UNVERIFIED: no --expect-fingerprint/--expect-xpub supplied; verify the master \
             fingerprint above ({fp_str}) against your records before importing"
        )
        .map_err(ToolkitError::Io)?;
    }

    crate::secret_advisory::emit_output_class_advisory(
        crate::secret_advisory::OutputClass::WatchOnly,
        stderr,
    );

    Ok(0)
}

/// Build the importable wallet-software payload for a single template via the
/// `export-wallet` `WalletFormatEmitter` dispatch (§3.5; Task 2.1).
///
/// Mirrors the 16-field `EmitInputs` ctor + dispatch in `cmd::export_wallet::run`
/// (`export_wallet.rs`). NOTE: `EmitInputs.script_type` is
/// `wallet_export::WalletScriptType` — a DIFFERENT enum from the
/// `convert::ScriptType` used for address rendering — so we use
/// `wallet_export::script_type_from_template`, not the convert-side helper.
fn build_import_payload(
    format: CliExportFormat,
    row: &WalletRow,
    network: CliNetwork,
    account: u32,
) -> Result<String, ToolkitError> {
    let script_type = wallet_export::script_type_from_template(&row.template);
    let wallet_name = format!("{}-{}", row.template.human_name(), account);
    let inputs = EmitInputs {
        canonical_descriptor: CheckedDescriptor::new(&row.descriptor)?,
        resolved_slots: std::slice::from_ref(&row.slot),
        template: Some(row.template),
        script_type,
        network,
        account,
        // Single-sig: no multisig threshold.
        threshold: None,
        threshold_user_supplied: false,
        master_xpub_at_0: row.slot.master_xpub,
        wallet_name: &wallet_name,
        wallet_name_is_non_default: false,
        taproot_internal_key: None,
        range: (0, 999),
        timestamp: TimestampArg::Now,
        bitcoin_core_version: 25,
        bsms_form: BsmsForm::default(),
    };

    // P2 R0 I1: mirror the canonical `export-wallet` SPEC §4 missing-info
    // channel (export_wallet.rs:506-525) — run the selected emitter's
    // `collect_missing` FIRST and short-circuit to the same deterministic
    // `ToolkitError::ExportWalletMissingFields` refusal before any `emit()`.
    // restore had previously mirrored only the `emit()` half, so e.g.
    // `--format specter` emitted a placeholder-name wallet (exit 0) where
    // `export-wallet --format specter` (no `--wallet-name`) refuses. Do NOT
    // invent a new error — reuse the export-wallet variant verbatim so the
    // exit code + missing-fields message are byte-identical.
    let (missing, format_name): (Vec<crate::wallet_export::MissingField>, &'static str) =
        match format {
            CliExportFormat::BitcoinCore => (BitcoinCoreEmitter::collect_missing(&inputs), "bitcoin-core"),
            CliExportFormat::Bip388 => (Bip388Emitter::collect_missing(&inputs), "bip388"),
            CliExportFormat::Coldcard => (ColdcardEmitter::collect_missing(&inputs), "coldcard"),
            CliExportFormat::ColdcardMultisig => (ColdcardEmitter::collect_missing(&inputs), "coldcard-multisig"),
            CliExportFormat::Jade => (JadeEmitter::collect_missing(&inputs), "jade"),
            CliExportFormat::Sparrow => (SparrowEmitter::collect_missing(&inputs), "sparrow"),
            CliExportFormat::Specter => (SpecterEmitter::collect_missing(&inputs), "specter"),
            CliExportFormat::Electrum => (ElectrumEmitter::collect_missing(&inputs), "electrum"),
            CliExportFormat::Green => (GreenEmitter::collect_missing(&inputs), "green"),
            CliExportFormat::Bsms => (BsmsEmitter::collect_missing(&inputs), "bsms"),
            CliExportFormat::Descriptor => (DescriptorEmitter::collect_missing(&inputs), "descriptor"),
        };
    if !missing.is_empty() {
        return Err(ToolkitError::ExportWalletMissingFields {
            format: format_name,
            missing,
        });
    }

    match format {
        CliExportFormat::BitcoinCore => BitcoinCoreEmitter::emit(&inputs),
        CliExportFormat::Bip388 => Bip388Emitter::emit(&inputs),
        CliExportFormat::Coldcard => ColdcardEmitter::emit(&inputs),
        CliExportFormat::ColdcardMultisig => Err(bad(
            "--format coldcard-multisig requires a multisig wallet; restore is single-sig — use --format coldcard",
        )),
        CliExportFormat::Jade => JadeEmitter::emit(&inputs),
        CliExportFormat::Sparrow => SparrowEmitter::emit(&inputs),
        CliExportFormat::Specter => SpecterEmitter::emit(&inputs),
        CliExportFormat::Electrum => ElectrumEmitter::emit(&inputs),
        CliExportFormat::Green => GreenEmitter::emit(&inputs),
        CliExportFormat::Bsms => BsmsEmitter::emit(&inputs),
        CliExportFormat::Descriptor => DescriptorEmitter::emit(&inputs),
    }
}

fn template_label(t: CliTemplate) -> &'static str {
    match t {
        CliTemplate::Bip44 => "bip44 (legacy P2PKH)",
        CliTemplate::Bip49 => "bip49 (nested segwit P2SH-P2WPKH)",
        CliTemplate::Bip84 => "bip84 (native segwit P2WPKH)",
        CliTemplate::Bip86 => "bip86 (taproot P2TR)",
        // Multisig templates are rejected before any WalletRow is built.
        _ => "multisig",
    }
}
