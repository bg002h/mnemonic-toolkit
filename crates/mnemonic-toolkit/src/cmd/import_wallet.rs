//! `mnemonic import-wallet` — Phase 5 surface (full v0.26.0).
//!
//! v0.26.0 Phase 5 extends the Phase 2/3 scaffold to the full clap surface
//! per SPEC_wallet_import_v0_26_0.md §2.1:
//!
//!   --blob <FILE|->                                             required
//!   --format <bsms|bitcoin-core>                                optional (sniff default)
//!   --select-descriptor <N|active-receive|active-change|all>    default `all`
//!   --ms1 <STRING>                                              repeatable (positional cosigner-index)
//!   --slot @<N>.phrase=<STRING>                                 (existing slot infra)
//!   --json                                                      bool; emit JSON envelope array
//!   --no-auto-repair                                            global; no-op in v0.26.0 (reserved)
//!
//! Sniff dispatch flow (SPEC §6):
//!   1. Resolve env-var sentinels (`@env:VAR` → `std::env::var(VAR)`).
//!   2. Read blob.
//!   3. If `--format` is absent → invoke `sniff_format`:
//!        - Bsms / BitcoinCore → dispatch to corresponding parser.
//!        - Ambiguous → exit 1 `ImportWalletAmbiguousFormat`.
//!        - NoMatch → exit 1 `ImportWalletAmbiguousFormat` (different stderr template).
//!   4. If `--format <X>` is present → exit 1 `ImportWalletFormatMismatch`
//!      if blob sniffs as a different format. (When the user-supplied
//!      format matches sniff outcome OR sniff is `NoMatch`/`Ambiguous`,
//!      the explicit `--format` is honored.)
//!   5. Parse via selected parser. Apply seed overlay (SPEC §8.3). Apply
//!      `--select-descriptor` filter. Emit stdout (cards-or-JSON).
//!
//! Stderr discipline:
//!   - WARNINGs / NOTICEs from per-format parsers.
//!   - When `--json` is set: round-trip diff goes ONLY in the envelope;
//!     stderr is silent for the diff (SPEC §7.4).

use crate::error::ToolkitError;
use crate::format::{BundleJson, CosignerEntry, MultisigInfo};
use crate::language::CliLanguage;
use crate::slot_input::{SlotInput, SlotSubkey};
use crate::synthesize::synthesize_descriptor;
use crate::wallet_import::{
    apply_select_descriptor,
    bitcoin_core::BitcoinCoreParser,
    bsms::BsmsParser,
    overlay::apply_seed_overlay,
    roundtrip::{canonicalize_bitcoin_core, canonicalize_bsms, unified_diff},
    sniff::{sniff_format, SniffOutcome},
    ParsedImport, SelectDescriptor, WalletFormatParser,
};
use clap::Args;
use serde_json::json;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

/// v0.27.0 SPEC §3.2 — `import-wallet --json` envelope schema version.
/// Pinned at "1" (first version; no migration). Phase 6 documents this in
/// the manual; future bumps update both sites + the SPEC.
pub(crate) const IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION: &str = "1";

#[derive(Args, Debug, Clone)]
pub struct ImportWalletArgs {
    /// Path to the third-party wallet blob; `-` reads from stdin.
    /// v0.27.0: required UNLESS `--bsms-round1` is supplied (Round-1 verify
    /// alone is a meaningful CLI mode; emits per-record verify envelope on
    /// `--json`, exits 0 on verify success).
    #[arg(
        long = "blob",
        value_name = "FILE|-",
        required_unless_present = "bsms_round1"
    )]
    pub blob: Option<PathBuf>,

    /// Format override. If absent, the blob is auto-detected via sniff
    /// (SPEC §6). Supported values: `bsms`, `bitcoin-core`.
    #[arg(
        long = "format",
        value_name = "bsms|bitcoin-core",
        value_parser = clap::builder::PossibleValuesParser::new(["bsms", "bitcoin-core"]),
    )]
    pub format: Option<String>,

    /// Multi-descriptor selector for Bitcoin Core blobs (SPEC §5.3).
    /// Accepts an integer (`0`, `1`, ...), `active-receive`, `active-change`,
    /// or `all` (default). BSMS blobs coerce non-default values to `all`
    /// with a stderr NOTICE.
    #[arg(
        long = "select-descriptor",
        value_name = "N|active-receive|active-change|all",
        default_value = "all"
    )]
    pub select_descriptor: String,

    /// Seed overlay (SPEC §8.3): supply the secret material that matches
    /// the blob's declared xpub at the cosigner's origin path. Repeatable;
    /// positional cosigner-index — the i-th `--ms1` applies to cosigner i.
    /// Accepts the `@env:VAR` sentinel (resolves to `std::env::var(VAR)`).
    /// Empty-string `--ms1 ""` preserves v0.25.1 watch-only sentinel
    /// semantics (cosigner left watch-only + stderr NOTICE).
    #[arg(long = "ms1", value_name = "STRING")]
    pub ms1: Vec<String>,

    /// Per-slot seed overlay via `--slot @<N>.phrase=<BIP-39 phrase>`.
    /// Equivalent to `--ms1`: the phrase is converted to entropy and the
    /// derived xpub at the cosigner's origin path is compared against the
    /// blob's xpub. Mutually exclusive with `--ms1[N]` for the same N.
    /// Accepts the `@env:VAR` sentinel for the phrase value.
    /// In v0.26.0 only the `phrase` subkey is accepted on `import-wallet`;
    /// other subkeys (`entropy`, `xpub`, etc.) are rejected.
    #[arg(long = "slot", value_name = "@N.phrase=<phrase>", value_parser = crate::slot_input::parse_slot_input)]
    pub slot: Vec<SlotInput>,

    /// Emit a JSON envelope array on stdout (SPEC §7.4) instead of the
    /// human-readable summary. Each envelope carries:
    ///   - `bundle`           — parsed bundle summary
    ///   - `source_format`    — "bsms" or "bitcoin-core"
    ///   - `roundtrip`        — { byte_exact, semantic_match, diff? }
    ///   - `bsms_audit?`      — BSMS audit fields (BSMS source only)
    ///   - `source_metadata?` — Bitcoin Core per-entry metadata
    ///   - `bsms_round1_verifications?` — per-record BIP-129 Round-1 SIG
    ///     verify state when `--bsms-round1` supplied (v0.27.0)
    ///
    /// When `--json` is set, the round-trip diff goes ONLY in the envelope;
    /// stderr is silent for the diff.
    #[arg(long = "json")]
    pub json: bool,

    /// v0.27.0 — supply a BIP-129 5-line Round-1 key record (Signer →
    /// Coordinator) for BIP-322 ECDSA signature verification. Repeating
    /// flag — one per record. `<FILE>` reads file contents; `-` reads one
    /// record from stdin (mutually exclusive with `--blob -`).
    ///
    /// Each record is verified independently; verify state propagates to the
    /// `--json` envelope's `bsms_round1_verifications` field. Verify failure
    /// is fatal under `--bsms-verify-strict`; otherwise emits a stderr NOTICE
    /// and sets `signature_verified: false` per-record.
    #[arg(long = "bsms-round1", value_name = "FILE|-")]
    pub bsms_round1: Vec<PathBuf>,

    /// v0.27.0 — make BIP-129 Round-1 SIG verification failures fatal.
    /// Without this flag, verify mismatches emit a stderr NOTICE and proceed
    /// with `signature_verified: false`. With this flag, verify mismatch is
    /// `BsmsSignatureMismatch` exit 2.
    #[arg(long = "bsms-verify-strict")]
    pub bsms_verify_strict: bool,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &ImportWalletArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    // `--no-auto-repair` is resolved here for symmetry with other v0.22.x+
    // subcommands (verify-bundle / convert / inspect). In v0.26.0 the
    // import-wallet path does NOT auto-fire BCH error-correction: BSMS +
    // Bitcoin Core wallet blobs do not carry BCH-coded fields (those live
    // in the toolkit's own ms1/mk1/md1 cards). The flag is reserved for
    // v0.27+ — for now it is documented in `--help` so the schema surface
    // is symmetric and any future BCH-coded import source can adopt the
    // same TTY-conditional auto-fire convention used elsewhere.
    let _ = crate::repair::resolve_no_auto_repair(no_auto_repair);

    // v0.26.0 §3 — resolve `@env:<VAR>` sentinels on secret-bearing flags.
    let env_resolved_owned;
    let args: &ImportWalletArgs = if needs_env_sentinel_resolution(args) {
        env_resolved_owned = resolve_env_sentinels(args)?;
        &env_resolved_owned
    } else {
        args
    };

    // v0.27.0 — `--bsms-verify-strict` without `--bsms-round1` is meaningless;
    // reject explicitly so the user notices the typo.
    if args.bsms_verify_strict && args.bsms_round1.is_empty() {
        return Err(ToolkitError::BadInput(
            "--bsms-verify-strict requires `--bsms-round1` (the flag controls \
             BIP-129 Round-1 SIG verify strictness; there are no records to verify)"
                .to_string(),
        ));
    }

    // v0.27.0 — BIP-129 Round-1 BIP-322 ECDSA verify (independent of --blob).
    let round1_verifications = if !args.bsms_round1.is_empty() {
        verify_bsms_round1_files(&args.bsms_round1, args.bsms_verify_strict, stderr)?
    } else {
        Vec::new()
    };

    // v0.27.0 — Standalone Round-1 verify mode: no --blob supplied. Emit
    // verifications only (no bundle synthesis path).
    let blob_path = match &args.blob {
        Some(p) => p,
        None => {
            if args.json {
                emit_round1_only_envelope(stdout, &round1_verifications)?;
            } else {
                emit_round1_only_summary(stdout, &round1_verifications)?;
            }
            return Ok(0);
        }
    };

    // Read blob.
    let blob = read_blob(blob_path, stdin)?;

    // SPEC §6: sniff dispatch.
    let sniff_outcome = sniff_format(&blob);
    let format_str: &str = match args.format.as_deref() {
        Some("bsms") => {
            if let SniffOutcome::BitcoinCore = sniff_outcome {
                return Err(ToolkitError::ImportWalletFormatMismatch {
                    supplied: "bsms".to_string(),
                    sniffed: "bitcoin-core".to_string(),
                });
            }
            "bsms"
        }
        Some("bitcoin-core") => {
            if let SniffOutcome::Bsms = sniff_outcome {
                return Err(ToolkitError::ImportWalletFormatMismatch {
                    supplied: "bitcoin-core".to_string(),
                    sniffed: "bsms".to_string(),
                });
            }
            "bitcoin-core"
        }
        Some(other) => {
            return Err(ToolkitError::BadInput(format!(
                "--format {other} is not supported in v0.26.0 (bsms + bitcoin-core only)"
            )));
        }
        None => match sniff_outcome {
            SniffOutcome::Bsms => "bsms",
            SniffOutcome::BitcoinCore => "bitcoin-core",
            SniffOutcome::Ambiguous => {
                return Err(ToolkitError::ImportWalletAmbiguousFormat(
                    "import-wallet: blob matches multiple format heuristics; \
                     supply --format <bsms|bitcoin-core>"
                        .to_string(),
                ));
            }
            SniffOutcome::NoMatch => {
                return Err(ToolkitError::ImportWalletAmbiguousFormat(
                    "import-wallet: could not detect format; \
                     supply --format <bsms|bitcoin-core>"
                        .to_string(),
                ));
            }
        },
    };

    // Validate slot subkeys: import-wallet only accepts `phrase` subkey.
    // Other secret-bearing subkeys (entropy / wif / xprv) and watch-only
    // subkeys are rejected at the import-wallet surface — phrase is the
    // only seed-source channel.
    for s in &args.slot {
        if s.subkey != SlotSubkey::Phrase {
            return Err(ToolkitError::BadInput(format!(
                "import-wallet: --slot @{}.{}=: only the `phrase` subkey is supported \
                 by import-wallet in v0.26.0",
                s.index,
                s.subkey.as_str()
            )));
        }
    }

    // Parse via selected format.
    let mut parsed: Vec<ParsedImport> = match format_str {
        "bsms" => BsmsParser::parse(&blob, stderr)?,
        "bitcoin-core" => BitcoinCoreParser::parse(&blob, stderr)?,
        other => {
            return Err(ToolkitError::BadInput(format!(
                "import-wallet --format {other} is not supported in v0.26.0 (bsms + bitcoin-core only)"
            )));
        }
    };

    // Seed overlay (SPEC §8.3). Apply BEFORE select-descriptor filter so
    // the user's overlay-args index the canonical cosigner ordering.
    //
    // Build positional `ms1` vector — ms1[i] is Some(value) for cosigner i.
    let mut ms1_args: Vec<Option<String>> = Vec::with_capacity(args.ms1.len());
    for v in &args.ms1 {
        ms1_args.push(Some(v.clone()));
    }
    let phrase_overlays: Vec<(u8, String)> = args
        .slot
        .iter()
        .filter(|s| s.subkey == SlotSubkey::Phrase)
        .map(|s| (s.index, s.value.clone()))
        .collect();
    if !ms1_args.is_empty() || !phrase_overlays.is_empty() {
        apply_seed_overlay(
            &mut parsed,
            &ms1_args,
            &phrase_overlays,
            CliLanguage::default(),
            stderr,
        )?;
    }

    // SPEC §5.3 — `--select-descriptor` filter. BSMS coerces non-default
    // to `all` per the SPEC NOTICE rule; emit the NOTICE here.
    let select = parse_select(&args.select_descriptor)?;
    let parsed = match format_str {
        "bsms" => match select {
            SelectDescriptor::All => parsed,
            _ => {
                let _ = writeln!(
                    stderr,
                    "notice: import-wallet: bsms: --select-descriptor {} has no effect; \
                     BSMS Round-2 carries a single descriptor",
                    args.select_descriptor
                );
                parsed
            }
        },
        _ => apply_select_descriptor(parsed, select)?,
    };

    // Emit stdout.
    if args.json {
        emit_json_envelope(
            stdout,
            &parsed,
            &blob,
            format_str,
            args.json,
            &round1_verifications,
        )?;
    } else {
        // Default text-mode: emit the Phase 2/3 summary form. Emit
        // round-trip diff on stderr when canonicalize is non-byte-exact.
        emit_summary(stdout, &parsed)?;
        emit_roundtrip_stderr_warning(stderr, &blob, format_str)?;
    }
    Ok(0)
}

/// Parse the `--select-descriptor` flag value into a SelectDescriptor variant.
/// Accepts `all`, `active-receive`, `active-change`, or an integer (mapped to
/// `ByIndex(N)`).
fn parse_select(s: &str) -> Result<SelectDescriptor, ToolkitError> {
    match s {
        "all" => Ok(SelectDescriptor::All),
        "active-receive" => Ok(SelectDescriptor::ActiveReceive),
        "active-change" => Ok(SelectDescriptor::ActiveChange),
        other => {
            if let Ok(n) = other.parse::<usize>() {
                return Ok(SelectDescriptor::ByIndex(n));
            }
            Err(ToolkitError::BadInput(format!(
                "--select-descriptor: invalid value `{other}`; expected `N` (integer), `active-receive`, `active-change`, or `all`"
            )))
        }
    }
}

/// v0.26.0 §3 — cheap pre-check for `@env:` sentinels on `import-wallet`'s
/// secret-bearing flag surfaces (`--ms1`, secret-bearing `--slot`).
fn needs_env_sentinel_resolution(args: &ImportWalletArgs) -> bool {
    let ms1 = args.ms1.iter().any(|v| v.starts_with("@env:"));
    let slot = args
        .slot
        .iter()
        .any(|s| s.subkey.is_secret_bearing() && s.value.starts_with("@env:"));
    ms1 || slot
}

/// v0.26.0 §3 — resolve `@env:<VAR>` sentinels across `import-wallet`'s
/// secret-bearing flag surfaces. Non-secret slot subkeys are NOT resolved
/// per SPEC §3.2 (opt-in per-callsite).
fn resolve_env_sentinels(args: &ImportWalletArgs) -> Result<ImportWalletArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    for v in owned.ms1.iter_mut() {
        *v = resolve_env_var_sentinel(v, "--ms1")?;
    }
    for s in owned.slot.iter_mut() {
        if s.subkey.is_secret_bearing() {
            let flag = format!("--slot @{}.{}=", s.index, s.subkey.as_str());
            s.value = resolve_env_var_sentinel(&s.value, &flag)?;
        }
    }
    Ok(owned)
}

/// v0.27.0 SPEC §3.2 + §3.2.1 — emit the `import-wallet --json` envelope
/// array on stdout. Each element corresponds to one `ParsedImport`.
///
/// The envelope's `bundle` field is the full `BundleJson` shape (SPEC §5.3),
/// synthesized post-parse via `synthesize_descriptor`. v0.26.0's compact
/// `bundle: { cosigners, network, threshold }` summary is replaced wholesale
/// — this is the wire-shape change that v0.27.0's `### Changed` CHANGELOG
/// entry documents. Closes FOLLOWUP `wallet-import-json-envelope-full-bundle`.
///
/// **BSMS round-trip caveat:** v0.27.0 ships a BSMS Round-2 emitter (Phase 3),
/// but wiring import-wallet's round-trip block to consume it is out of scope
/// for Phase 4. Status stays `blocked_no_emitter` until a follow-up cycle
/// rewires the round-trip block to call the new BSMS emitter.
fn emit_json_envelope<W: Write>(
    stdout: &mut W,
    parsed: &[ParsedImport],
    blob: &[u8],
    format_str: &str,
    _json: bool,
    round1_verifications: &[Round1Verification],
) -> Result<(), ToolkitError> {
    let mut envelopes: Vec<serde_json::Value> = Vec::with_capacity(parsed.len());

    let canon_orig = match format_str {
        "bsms" => canonicalize_bsms(blob).ok(),
        "bitcoin-core" => canonicalize_bitcoin_core(blob).ok(),
        _ => None,
    };

    for p in parsed {
        // v0.27.0 SPEC §3.2.1 — synthesize the full BundleJson via
        // descriptor-mode synthesis (`synthesize_descriptor`). Both v0.26.0
        // wallet-import formats (BSMS Round-2 + Bitcoin Core listdescriptors)
        // carry a literal descriptor, so descriptor-mode synthesis applies
        // uniformly.
        let bundle = synthesize_descriptor(&p.descriptor, &p.cosigners, false)?;

        // Per §3.2.1 row `template`: descriptor-mode → `None`.
        // Per §3.2.1 row `descriptor`: source from `original_descriptor`
        // (pre-strip raw including `#<checksum>`). Disjoint use vs the
        // typed `p.descriptor` (input to synthesize above).
        let descriptor_field = Some(p.original_descriptor.clone());

        let n = p.cosigners.len();

        // Per §3.2.1 row `master_fingerprint`: Some only for N=1; None for
        // multisig. Mirrors live cmd/bundle.rs:677-678 emission rule.
        let master_fingerprint = if n == 1 {
            Some(p.cosigners[0].fingerprint.to_string().to_lowercase())
        } else {
            None
        };

        // Per §3.2.1 row `origin_path` / `origin_paths`: mutually exclusive
        // per SPEC §5.3. Extract per-cosigner path string from the bracket-
        // form `path_raw` produced by the wallet-import parsers
        // (`[fp_hex/48'/0'/0'/2']`); strip the fingerprint prefix so the
        // wire-shape matches `cmd/bundle.rs` (`m/48'/0'/0'/2'`).
        let paths: Vec<String> = p
            .cosigners
            .iter()
            .map(|c| origin_path_from_bracket(&c.path_raw))
            .collect();
        let (origin_path, origin_paths) = if n == 1 {
            (paths.first().cloned(), None)
        } else {
            let all_same = paths.windows(2).all(|w| w[0] == w[1]);
            if all_same {
                (paths.first().cloned(), None)
            } else {
                (None, Some(paths.clone()))
            }
        };

        // Per §3.2.1 row `multisig`: Some when N>1, None for N=1.
        let multisig = if n > 1 {
            let cosigners: Vec<CosignerEntry> = p
                .cosigners
                .iter()
                .enumerate()
                .map(|(i, s)| CosignerEntry {
                    index: i,
                    master_fingerprint: Some(s.fingerprint.to_string().to_lowercase()),
                    origin_path: origin_path_from_bracket(&s.path_raw),
                    xpub: s.xpub.to_string(),
                })
                .collect();
            let threshold = p.threshold.unwrap_or(n as u8);
            Some(MultisigInfo {
                template: "descriptor",
                threshold,
                cosigner_count: n,
                path_family: path_family_from_paths(&paths),
                cosigners,
            })
        } else {
            None
        };

        // Per §3.2.1 row `mode`: "watch-only" when all cosigners are
        // watch-only; "full" if any cosigner has entropy attached (seed
        // overlay path). Mirrors `bundle.any_secret_bearing()` rule at
        // `cmd/bundle.rs:611`.
        let mode_str: &'static str = if p.cosigners.iter().any(|c| c.entropy.is_some()) {
            "full"
        } else {
            "watch-only"
        };

        let bundle_json = BundleJson {
            schema_version: "4",
            mode: mode_str,
            network: network_human_name(p.network),
            template: None,
            descriptor: descriptor_field,
            account: 0,
            origin_path,
            origin_paths,
            master_fingerprint,
            ms1: bundle.ms1,
            mk1: bundle.mk1,
            md1: bundle.md1,
            multisig,
            privacy_preserving: false,
        };
        let bundle_value = serde_json::to_value(&bundle_json)
            .map_err(|e| ToolkitError::BadInput(format!("import-wallet --json bundle serialize: {e}")))?;

        // Round-trip per SPEC §7.4 + §7.3 — preserved from v0.26.0 wire shape.
        let roundtrip = match format_str {
            "bitcoin-core" => match canon_orig.clone() {
                Some(canon) => {
                    let original_text = std::str::from_utf8(blob).unwrap_or("").to_string();
                    let byte_exact = original_text == canon;
                    let diff_val = if byte_exact {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(unified_diff(&original_text, &canon))
                    };
                    json!({
                        "byte_exact": byte_exact,
                        "semantic_match": true,
                        "diff": diff_val,
                        "status": "ok",
                    })
                }
                None => json!({
                    "byte_exact": false,
                    "semantic_match": false,
                    "diff": serde_json::Value::Null,
                    "status": "canonicalize_failed",
                }),
            },
            "bsms" => json!({
                "byte_exact": false,
                "semantic_match": false,
                "diff": serde_json::Value::Null,
                "status": "blocked_no_emitter",
            }),
            _ => json!({}),
        };

        let mut env = serde_json::Map::new();
        env.insert(
            "schema_version".to_string(),
            json!(IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION),
        );
        env.insert("source_format".to_string(), json!(format_str));
        env.insert("bundle".to_string(), bundle_value);
        env.insert("roundtrip".to_string(), roundtrip);

        if let Some(audit) = &p.bsms_audit {
            env.insert(
                "bsms_audit".to_string(),
                json!({
                    "token": audit.token,
                    "signature": audit.signature,
                    "first_address": audit.first_address,
                    "derivation_path": audit.derivation_path,
                    "signature_verified": audit.signature_verified,
                }),
            );
        }
        if let Some(meta) = &p.source_metadata {
            env.insert(
                "source_metadata".to_string(),
                json!({
                    "active": meta.active,
                    "internal": meta.internal,
                    "range": meta.range,
                    "dropped_fields": meta.dropped_fields,
                    "wallet_name": meta.wallet_name,
                }),
            );
        }

        // v0.27.0 — propagate Round-1 BIP-322 verify state when --bsms-round1
        // was supplied alongside --blob. Same array on every parsed entry
        // (verifications are blob-independent; surface on every envelope so
        // downstream consumers don't have to index-match).
        if !round1_verifications.is_empty() {
            env.insert(
                "bsms_round1_verifications".to_string(),
                serde_json::Value::Array(
                    round1_verifications
                        .iter()
                        .map(round1_verification_to_json)
                        .collect(),
                ),
            );
        }

        envelopes.push(serde_json::Value::Object(env));
    }

    let text = serde_json::to_string_pretty(&serde_json::Value::Array(envelopes))
        .map_err(|e| ToolkitError::BadInput(format!("import-wallet --json serialize: {e}")))?;
    writeln!(stdout, "{text}").map_err(ToolkitError::Io)?;
    Ok(())
}

/// Extract the m/-prefixed origin path from a wallet-import-parsed
/// `path_raw` (bracket form: `[fp_hex/48'/0'/0'/2']`). The bracket form is
/// produced by `wallet_import::bsms::extract_origin_components` and the
/// Bitcoin Core sibling; it carries the fingerprint inline because the
/// bracketed-origin annotation is BIP-380 syntax. The v0.27.0 envelope's
/// `origin_path` (and `multisig.cosigners[].origin_path`) wire-shape mirrors
/// `cmd/bundle.rs:617-625` (`m/48'/0'/0'/2'`); strip `[fp_hex` and `]` here.
fn origin_path_from_bracket(path_raw: &str) -> String {
    let inner = path_raw.trim_start_matches('[').trim_end_matches(']');
    match inner.find('/') {
        Some(slash) => format!("m{}", &inner[slash..]),
        None => "m".to_string(),
    }
}

/// SPEC §3.2.1 row `multisig.path_family` — heuristic detection from the
/// BIP-43-purpose component of the first cosigner's path. `48'` → bip48
/// (BIP-48 multisig); `87'` → bip87 (BIP-87). Default `bip87` when the
/// purpose-component is unrecognized (matches `MultisigPathFamily::default()`
/// at `parse.rs:67`). Heterogeneity is rare in real-world wallet imports —
/// the BSMS parser's network-detection step already enforces coin-type
/// uniformity, and the BIP-43 purpose tracks that.
fn path_family_from_paths(paths: &[String]) -> &'static str {
    let first = match paths.first() {
        Some(p) => p,
        None => return "bip87",
    };
    let trimmed = first.trim_start_matches("m/");
    let purpose = trimmed.split('/').next().unwrap_or("");
    match purpose {
        "48'" | "48h" => "bip48",
        "87'" | "87h" => "bip87",
        _ => "bip87",
    }
}

/// SPEC §7.4: when `--json` is NOT set, the round-trip diff goes ONLY on
/// stderr (the cards stdout is unaffected). For BSMS we have no emitter
/// in v0.26.0 (FOLLOWUP `wallet-export-bsms-emitter`); we skip the
/// WARNING. For Bitcoin Core we compare original bytes vs canonicalize
/// and emit a WARNING per SPEC §2.4 ("roundtrip not byte-exact; semantic
/// equivalent; diff below").
fn emit_roundtrip_stderr_warning<E: Write>(
    stderr: &mut E,
    blob: &[u8],
    format_str: &str,
) -> Result<(), ToolkitError> {
    if format_str != "bitcoin-core" {
        return Ok(());
    }
    let canon = match canonicalize_bitcoin_core(blob) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };
    let original_text = match std::str::from_utf8(blob) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };
    if original_text == canon {
        return Ok(());
    }
    let diff = unified_diff(original_text, &canon);
    let _ = writeln!(
        stderr,
        "warning: import-wallet: roundtrip not byte-exact; semantic equivalent; diff below"
    );
    let _ = write!(stderr, "{diff}");
    Ok(())
}

pub(crate) fn network_human_name(n: bitcoin::Network) -> &'static str {
    match n {
        bitcoin::Network::Bitcoin => "mainnet",
        bitcoin::Network::Testnet => "testnet",
        bitcoin::Network::Signet => "signet",
        bitcoin::Network::Regtest => "regtest",
        _ => "unknown",
    }
}

fn emit_summary<W: Write>(stdout: &mut W, parsed: &[ParsedImport]) -> Result<(), ToolkitError> {
    writeln!(stdout, "import-wallet: bundles={}", parsed.len()).map_err(ToolkitError::Io)?;
    for (i, b) in parsed.iter().enumerate() {
        writeln!(stdout, "bundles[{i}].cosigners={}", b.cosigners.len())
            .map_err(ToolkitError::Io)?;
        let network_name = network_human_name(b.network);
        writeln!(stdout, "bundles[{i}].network={network_name}").map_err(ToolkitError::Io)?;
        let threshold_str = b
            .threshold
            .map(|t| t.to_string())
            .unwrap_or_else(|| "none".to_string());
        writeln!(stdout, "bundles[{i}].threshold={threshold_str}").map_err(ToolkitError::Io)?;
        let audit_str = if b.bsms_audit.is_some() {
            "some"
        } else {
            "none"
        };
        writeln!(stdout, "bundles[{i}].bsms_audit={audit_str}").map_err(ToolkitError::Io)?;
        let entropy_str = if b.cosigners.iter().any(|c| c.entropy.is_some()) {
            "some"
        } else {
            "none"
        };
        writeln!(stdout, "bundles[{i}].entropy={entropy_str}").map_err(ToolkitError::Io)?;
        let src_meta_str = if b.source_metadata.is_some() {
            "some"
        } else {
            "none"
        };
        writeln!(stdout, "bundles[{i}].source_metadata={src_meta_str}")
            .map_err(ToolkitError::Io)?;
        if let Some(m) = &b.source_metadata {
            writeln!(stdout, "bundles[{i}].active={}", m.active).map_err(ToolkitError::Io)?;
            writeln!(stdout, "bundles[{i}].internal={}", m.internal).map_err(ToolkitError::Io)?;
        }
        for (j, c) in b.cosigners.iter().enumerate() {
            writeln!(
                stdout,
                "bundles[{i}].cosigners[{j}].fingerprint={}",
                hex_lower(&c.fingerprint.to_bytes())
            )
            .map_err(ToolkitError::Io)?;
            writeln!(
                stdout,
                "cosigners[{j}].fingerprint={}",
                hex_lower(&c.fingerprint.to_bytes())
            )
            .map_err(ToolkitError::Io)?;
        }
        writeln!(stdout, "cosigners={}", b.cosigners.len()).map_err(ToolkitError::Io)?;
        writeln!(stdout, "network={network_name}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "threshold={threshold_str}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "bsms_audit={audit_str}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "entropy={entropy_str}").map_err(ToolkitError::Io)?;
    }
    Ok(())
}

fn read_blob<R: Read>(path: &PathBuf, stdin: &mut R) -> Result<Vec<u8>, ToolkitError> {
    if path.as_os_str() == "-" {
        let mut buf = Vec::new();
        stdin.read_to_end(&mut buf).map_err(ToolkitError::Io)?;
        Ok(buf)
    } else {
        fs::read(path).map_err(ToolkitError::Io)
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ---------------------------------------------------------------------------
// v0.27.0 — BIP-129 Round-1 verify (--bsms-round1 + --bsms-verify-strict)
// ---------------------------------------------------------------------------

/// Per-record verify result. Propagates to `--json` envelope when --blob is
/// supplied, OR to the standalone Round-1 verify envelope when --blob is not.
struct Round1Verification {
    index: usize,
    signer_pubkey: String,
    description: String,
    token_hex: String,
    signature_verified: bool,
    /// Stderr-NOTICE detail (only populated on lenient-default verify failure).
    failure_reason: Option<String>,
}

/// Read + parse + verify each `--bsms-round1 <FILE>` entry. Lenient default:
/// verify failure emits stderr NOTICE + sets `signature_verified: false`;
/// strict (`--bsms-verify-strict`) makes verify failure fatal.
fn verify_bsms_round1_files(
    paths: &[PathBuf],
    strict: bool,
    stderr: &mut dyn Write,
) -> Result<Vec<Round1Verification>, ToolkitError> {
    use crate::wallet_import::bsms_round1::{parse_round1, signer_pubkey};
    use crate::wallet_import::bsms_verify::verify_round1_signature;

    let mut out = Vec::with_capacity(paths.len());
    for (i, path) in paths.iter().enumerate() {
        if path.as_os_str() == "-" {
            // v0.27.0 first cut: stdin input for --bsms-round1 deferred. Future:
            // multi-record stdin (one record per blob, separated by sentinel)
            // or single-record-from-stdin (mutually exclusive with --blob -).
            return Err(ToolkitError::BadInput(format!(
                "--bsms-round1 -: stdin input deferred in v0.27.0; supply a file path \
                 (record index {})",
                i
            )));
        }
        let text = std::fs::read_to_string(path).map_err(ToolkitError::Io)?;
        let record = parse_round1(&text)?;
        let pk_hex = hex::encode(signer_pubkey(&record).serialize());

        match verify_round1_signature(&record, i) {
            Ok(()) => {
                out.push(Round1Verification {
                    index: i,
                    signer_pubkey: pk_hex,
                    description: record.description.clone(),
                    token_hex: record.token_hex.clone(),
                    signature_verified: true,
                    failure_reason: None,
                });
            }
            Err(ToolkitError::BsmsSignatureMismatch {
                record_index,
                signer_pubkey: pk_for_err,
                reason,
            }) => {
                if strict {
                    return Err(ToolkitError::BsmsSignatureMismatch {
                        record_index,
                        signer_pubkey: pk_for_err,
                        reason,
                    });
                }
                let _ = writeln!(
                    stderr,
                    "notice: import-wallet: --bsms-round1: signature verification failed \
                     for record {i} (signer pubkey {pk_for_err}): {reason}"
                );
                out.push(Round1Verification {
                    index: i,
                    signer_pubkey: pk_for_err,
                    description: record.description.clone(),
                    token_hex: record.token_hex.clone(),
                    signature_verified: false,
                    failure_reason: Some(reason),
                });
            }
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}

fn emit_round1_only_envelope<W: Write>(
    stdout: &mut W,
    verifications: &[Round1Verification],
) -> Result<(), ToolkitError> {
    let payload = json!({
        "source_format": "bsms-round1",
        "bsms_round1_verifications": verifications
            .iter()
            .map(round1_verification_to_json)
            .collect::<Vec<_>>(),
    });
    let body = serde_json::to_string(&payload)
        .map_err(|e| ToolkitError::BadInput(format!("--bsms-round1 envelope serialize: {e}")))?;
    writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
    Ok(())
}

fn emit_round1_only_summary<W: Write>(
    stdout: &mut W,
    verifications: &[Round1Verification],
) -> Result<(), ToolkitError> {
    writeln!(stdout, "bsms-round1: {} record(s) processed", verifications.len())
        .map_err(ToolkitError::Io)?;
    for v in verifications {
        writeln!(
            stdout,
            "  record[{}]: signer_pubkey={} description={:?} token_hex={} verified={}",
            v.index, v.signer_pubkey, v.description, v.token_hex, v.signature_verified
        )
        .map_err(ToolkitError::Io)?;
    }
    Ok(())
}

fn round1_verification_to_json(v: &Round1Verification) -> serde_json::Value {
    json!({
        "index": v.index,
        "signer_pubkey": v.signer_pubkey,
        "description": v.description,
        "token_hex": v.token_hex,
        "signature_verified": v.signature_verified,
        "failure_reason": v.failure_reason,
    })
}
