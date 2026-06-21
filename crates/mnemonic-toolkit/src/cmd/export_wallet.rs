//! `mnemonic export-wallet` subcommand.
//!
//! Realizes `design/SPEC_export_wallet_v0_8.md` (subcommand grammar, refusal
//! contract, per-vendor format emitters).

use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::MultisigPathFamily;
use crate::template::CliTemplate;
use crate::wallet_export::{
    build_descriptor_string, script_type_from_descriptor, script_type_from_template,
    validate_watch_only, validate_watch_only_resolved, Bip388Emitter, BitcoinCoreEmitter,
    BsmsEmitter, BsmsForm, ColdcardEmitter, DescriptorEmitter, ElectrumEmitter, EmitInputs,
    GreenEmitter, JadeEmitter, SparrowEmitter, SpecterEmitter, TaprootInternalKey, TimestampArg,
    WalletFormatEmitter, WalletScriptType,
};
use clap::{Args, ValueEnum};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliExportFormat {
    #[value(name = "bitcoin-core")]
    BitcoinCore,
    #[value(name = "bip388")]
    Bip388,
    #[value(name = "coldcard")]
    Coldcard,
    #[value(name = "coldcard-multisig")]
    ColdcardMultisig,
    #[value(name = "jade")]
    Jade,
    #[value(name = "sparrow")]
    Sparrow,
    #[value(name = "specter")]
    Specter,
    #[value(name = "electrum")]
    Electrum,
    #[value(name = "green")]
    Green,
    #[value(name = "bsms")]
    Bsms,
    #[value(name = "descriptor")]
    Descriptor,
}

/// SPEC v0.37 §2.3 — formats whose file-import surface refuses a bare
/// descriptor and requires a `--template`. On the `--from-import-json` path
/// these receive a template derived from the envelope descriptor; all other
/// formats (descriptor-passthrough / template-agnostic) keep `template: None`
/// (regression guard: bip388/sparrow branch on `template.is_some()`).
/// Exhaustive (no `_` arm) so a new `CliExportFormat` variant forces a decision.
fn format_requires_template(f: CliExportFormat) -> bool {
    use CliExportFormat::*;
    match f {
        Sparrow | Coldcard | ColdcardMultisig | Jade | Electrum => true,
        BitcoinCore | Bip388 | Bsms | Green | Specter | Descriptor => false,
    }
}

/// Shared `WalletFormatEmitter` dispatch: `collect_missing`-first → refuse →
/// `emit`. Consolidates the formerly-4 byte-identical copies (FOLLOWUP
/// `restore-emit-dispatch-3way-dedup`; recon corrected "3-way" → "4-way":
/// `run`, `run_from_import_json`, and restore's single-sig + multisig
/// `build_*import_payload`). Each caller builds its own `EmitInputs` (the
/// genuinely per-site part); this owns only the format dispatch.
///
/// Routing single-sig restore through here unifies the `coldcard-multisig`
/// refusal: a single-sig `--template` now hits the 6-variant template `_ =>`
/// arm ("requires a multisig --template …") rather than the old
/// restore-specific "requires a multisig wallet" string — exit 1 (BadInput)
/// either way.
pub(crate) fn emit_payload(
    inputs: &EmitInputs,
    format: CliExportFormat,
) -> Result<String, ToolkitError> {
    // SPEC §4 missing-info channel — every emitter exposes a per-format
    // `collect_missing` predicate; non-empty result short-circuits to the
    // deterministic refusal via `ToolkitError::ExportWalletMissingFields`
    // (which routes through `build_missing_fields_refusal`).
    let (missing, format_name): (Vec<crate::wallet_export::MissingField>, &'static str) =
        match format {
            CliExportFormat::BitcoinCore => {
                (BitcoinCoreEmitter::collect_missing(inputs), "bitcoin-core")
            }
            CliExportFormat::Bip388 => (Bip388Emitter::collect_missing(inputs), "bip388"),
            CliExportFormat::Coldcard => (ColdcardEmitter::collect_missing(inputs), "coldcard"),
            CliExportFormat::ColdcardMultisig => (
                ColdcardEmitter::collect_missing(inputs),
                "coldcard-multisig",
            ),
            CliExportFormat::Jade => (JadeEmitter::collect_missing(inputs), "jade"),
            CliExportFormat::Sparrow => (SparrowEmitter::collect_missing(inputs), "sparrow"),
            CliExportFormat::Specter => (SpecterEmitter::collect_missing(inputs), "specter"),
            CliExportFormat::Electrum => (ElectrumEmitter::collect_missing(inputs), "electrum"),
            CliExportFormat::Green => (GreenEmitter::collect_missing(inputs), "green"),
            CliExportFormat::Bsms => (BsmsEmitter::collect_missing(inputs), "bsms"),
            CliExportFormat::Descriptor => {
                (DescriptorEmitter::collect_missing(inputs), "descriptor")
            }
        };
    if !missing.is_empty() {
        return Err(ToolkitError::ExportWalletMissingFields {
            format: format_name,
            missing,
        });
    }

    // SPEC cycle-2 H10 — refuse an UNSORTED multisig (`wsh-multi` /
    // `sh-wsh-multi`) to the field-less electrum / coldcard(-multisig) / jade
    // vendors. Those file formats are BIP-67 sortedmulti-only (no field to
    // express literal `multi(...)` key order), so emitting an unsorted multisig
    // would silently coerce to sortedmulti → different witnessScript / address
    // (oracle-proven by `tests/bitcoind_differential.rs`'s
    // `wsh-multi-2of3-divergent` row). STRUCTURED check on the resolved typed
    // `CliTemplate` — immune to the `sortedmulti(`-as-substring false-match a
    // naive `.contains("multi(")` would hit. Disjoint from the per-emitter
    // taproot guard (matches only the two unsorted-`Wsh`/`ShWsh` variants), so
    // `tr-multi-a` / `tr-sortedmulti-a` pass through to their existing taproot
    // refusal unshadowed. Sorted variants + single-sig + the faithful formats
    // (descriptor / sparrow / bitcoin-core) are unaffected. Restore's
    // `build_multisig_import_payload` calls this same chokepoint, so the guard
    // also covers `restore --md1 --format electrum/coldcard/jade` for free.
    if matches!(
        inputs.template,
        Some(CliTemplate::WshMulti | CliTemplate::ShWshMulti)
    ) && matches!(
        format,
        CliExportFormat::Electrum
            | CliExportFormat::Coldcard
            | CliExportFormat::ColdcardMultisig
            | CliExportFormat::Jade
    ) {
        return Err(ToolkitError::ExportWalletUnsortedMultisigUnsupported {
            format: format_name,
        });
    }

    match format {
        CliExportFormat::BitcoinCore => BitcoinCoreEmitter::emit(inputs),
        CliExportFormat::Bip388 => Bip388Emitter::emit(inputs),
        CliExportFormat::Coldcard => ColdcardEmitter::emit(inputs),
        CliExportFormat::ColdcardMultisig => {
            // v0.28.4 (A1): `coldcard-multisig` alias requires a multisig
            // template; singlesig templates route through `--format coldcard`
            // per chapter-45 § Coldcard. Refuse-with-pointer rather than
            // silently delegating, so the import-side acceptance of both
            // `coldcard` and `coldcard-multisig` is mirrored by an
            // export-side semantic distinction.
            match inputs.template {
                Some(
                    CliTemplate::WshMulti
                    | CliTemplate::WshSortedMulti
                    | CliTemplate::ShWshMulti
                    | CliTemplate::ShWshSortedMulti
                    | CliTemplate::TrMultiA
                    | CliTemplate::TrSortedMultiA,
                ) => ColdcardEmitter::emit(inputs),
                _ => Err(ToolkitError::BadInput(
                    "--format coldcard-multisig requires a multisig --template (wsh-sortedmulti, wsh-multi, sh-wsh-sortedmulti, sh-wsh-multi, tr-multi-a, tr-sortedmulti-a). For Coldcard singlesig export use --format coldcard with bip44/bip49/bip84."
                        .into(),
                )),
            }
        }
        CliExportFormat::Jade => JadeEmitter::emit(inputs),
        CliExportFormat::Sparrow => SparrowEmitter::emit(inputs),
        CliExportFormat::Specter => SpecterEmitter::emit(inputs),
        CliExportFormat::Electrum => ElectrumEmitter::emit(inputs),
        CliExportFormat::Green => GreenEmitter::emit(inputs),
        CliExportFormat::Bsms => BsmsEmitter::emit(inputs),
        CliExportFormat::Descriptor => DescriptorEmitter::emit(inputs),
    }
}

#[derive(Args, Debug)]
pub struct ExportWalletArgs {
    /// Pre-built template name. Mutually-required-one-of with --descriptor.
    #[arg(long, conflicts_with = "descriptor")]
    pub template: Option<CliTemplate>,

    /// User-supplied BIP-388 descriptor. Mutually exclusive with --template.
    /// (v0.49.0) Also accepts a BIP-388 wallet-policy JSON `{name,
    /// description_template, keys_info}` (auto-detected by a leading `{`),
    /// expanded to the concrete descriptor — the inverse of `--format bip388`.
    #[arg(long)]
    pub descriptor: Option<String>,

    /// Multisig threshold K (1 ≤ K ≤ N).
    #[arg(long)]
    pub threshold: Option<u8>,

    /// v0.2 multisig path family (default: bip87).
    #[arg(long = "multisig-path-family", value_enum)]
    pub multisig_path_family: Option<MultisigPathFamily>,

    #[arg(long, default_value = "mainnet")]
    pub network: CliNetwork,

    /// Ignored (watch-only); kept for slot parser symmetry.
    #[arg(long)]
    pub language: Option<CliLanguage>,

    /// BIP-32 account index (default 0).
    #[arg(long, default_value = "0")]
    pub account: u32,

    /// Slot input — shape `@N.<subkey>=<value>`. Repeating.
    ///
    /// `<subkey>` is one of:
    ///   phrase       BIP-39 mnemonic (secret)
    ///   seedqr       48 or 96 ASCII digits encoding a BIP-39 phrase
    ///                (secret; decoded inline via seedqr::decode)
    ///   entropy      raw entropy hex (secret)
    ///   xpub         BIP-32 extended public key
    ///   master_xpub  depth-0 master xpub (Coldcard singlesig only;
    ///                see SPEC_export_wallet §5.1)
    ///   fingerprint  4-byte master fingerprint (hex)
    ///   path         BIP-32 derivation path
    ///   wif          Wallet Import Format private key (secret)
    ///   xprv         BIP-32 extended private key (secret)
    ///
    /// `<value>` is the subkey's text form, or `-` to read from
    /// stdin. The subkeys mirror `mnemonic bundle --slot`. For a
    /// watch-only export only `xpub` and `fingerprint` are required;
    /// secret subkeys (incl. seedqr) are REFUSED at the export-wallet
    /// boundary per SPEC §3 (`mnemonic export-wallet is watch-only by
    /// definition`); use `mnemonic bundle` to materialize secret
    /// material.
    #[arg(
        long = "slot",
        action = clap::ArgAction::Append,
        value_parser = crate::slot_input::parse_slot_input,
        verbatim_doc_comment,
    )]
    pub slot: Vec<crate::slot_input::SlotInput>,

    /// Output format. Default bitcoin-core.
    #[arg(long, value_enum, default_value = "bitcoin-core")]
    pub format: CliExportFormat,

    /// Output path. `-` (default) → stdout.
    #[arg(long, default_value = "-")]
    pub output: String,

    /// Bitcoin Core `range` field, comma-separated. Default `0,999`.
    #[arg(long, default_value = "0,999", value_parser = parse_range)]
    pub range: (u32, u32),

    /// Bitcoin Core `timestamp` field. `0` (default; rescan from genesis to
    /// discover an existing key's funds), `now`, or unix seconds.
    #[arg(long, default_value = "0", value_parser = parse_timestamp)]
    pub timestamp: TimestampArgValue,

    /// Bitcoin Core target version. 24 or 25 (default 25).
    #[arg(long = "bitcoin-core-version", default_value = "25")]
    pub bitcoin_core_version: u8,

    /// SPEC v0.8 §2 — wallet name/label for formats that publish one
    /// (Coldcard generic JSON, Sparrow, Specter, Electrum). Optional;
    /// defaults to `<template-human-name>-<account>` (e.g., `bip84-0`,
    /// `wsh-sortedmulti-0`) when omitted. Ignored by formats that have
    /// no name slot (Bitcoin Core / BIP-388 / Jade text / Green).
    #[arg(long = "wallet-name")]
    pub wallet_name: Option<String>,

    /// SPEC v0.8 §7 — Taproot internal-key designation for
    /// `tr-multi-a` / `tr-sortedmulti-a` templates.
    ///
    /// Accepted values:
    ///   nums  BIP-341 reference NUMS x-only point (unspendable
    ///         internal key; common default)
    ///   @N    cosigner N's xpub as the key-path internal key
    ///         (cosigner N is then removed from the multi_a leaf
    ///         set; N is a decimal index 0..=N-1)
    ///
    /// Required under `tr-multi-a` / `tr-sortedmulti-a`; refused
    /// for non-Taproot templates.
    #[arg(
        long = "taproot-internal-key",
        value_parser = parse_taproot_internal_key_arg,
        verbatim_doc_comment,
    )]
    pub taproot_internal_key: Option<TaprootInternalKey>,

    /// SPEC v0.27.0 §3.5 — BSMS Round-2 emit shape. `4-line`
    /// (BIP-129-canonical) is the default; `2-line` is the lenient
    /// excerpt symmetric with the v0.26.0 import-side parser. Ignored
    /// by every other format.
    #[arg(long = "bsms-form", value_enum, default_value = "4-line")]
    pub bsms_form: BsmsForm,

    /// v0.27.0 — emit a per-format wallet config from an
    /// `import-wallet --json` envelope rather than from `--template` /
    /// `--descriptor`. Accepts a file path or `-` to read the envelope
    /// from stdin. The envelope's `bundle.descriptor` becomes the
    /// canonical descriptor for the emitter; cosigner xpubs decode
    /// from `bundle.mk1` per SPEC §3.6.1; network derives from
    /// `bundle.network`. Mutually exclusive with `--template` and
    /// `--descriptor`. `--account` is rejected (the envelope's
    /// `bundle.account` value applies; the account is encoded in the
    /// descriptor's origin paths).
    #[arg(
        long = "from-import-json",
        value_name = "FILE|-",
        conflicts_with_all = ["template", "descriptor"],
    )]
    pub from_import_json: Option<String>,

    /// v0.27.0 — pick a specific entry from a multi-entry envelope
    /// array. Required when the envelope has > 1 entry.
    #[arg(
        long = "from-import-json-index",
        value_name = "N",
        requires = "from_import_json"
    )]
    pub from_import_json_index: Option<usize>,
}

/// SPEC v0.8 §7 parser: `nums` or `@N` (decimal index).
fn parse_taproot_internal_key_arg(s: &str) -> Result<TaprootInternalKey, String> {
    if s == "nums" {
        return Ok(TaprootInternalKey::Nums);
    }
    if let Some(n) = s.strip_prefix('@') {
        let idx: u8 = n
            .parse()
            .map_err(|e| format!("--taproot-internal-key {s:?}: cosigner index must be u8: {e}"))?;
        return Ok(TaprootInternalKey::Cosigner(idx));
    }
    Err(format!(
        "--taproot-internal-key must be 'nums' or '@N' (cosigner index); got {s:?}",
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct TimestampArgValue(pub TimestampArg);

fn parse_range(s: &str) -> Result<(u32, u32), String> {
    let (a, b) = s.split_once(',').ok_or_else(|| {
        format!("--range expects '<start>,<end>' (comma-separated u32 pair); got {s:?}")
    })?;
    let start: u32 = a.parse().map_err(|e| format!("--range start: {e}"))?;
    let end: u32 = b.parse().map_err(|e| format!("--range end: {e}"))?;
    if start > end {
        return Err(format!("--range start {start} must be <= end {end}"));
    }
    Ok((start, end))
}

pub(crate) fn parse_timestamp(s: &str) -> Result<TimestampArgValue, String> {
    if s == "now" {
        Ok(TimestampArgValue(TimestampArg::Now))
    } else {
        let n: i64 = s
            .parse()
            .map_err(|e| format!("--timestamp expects 'now' or unix seconds; got {s:?}: {e}"))?;
        if n < 0 {
            return Err(format!("--timestamp unix seconds must be >= 0; got {n}"));
        }
        Ok(TimestampArgValue(TimestampArg::Unix(n)))
    }
}

/// `stderr` is reachable only from the `--from-import-json` path in v0.27.1+
/// (Phase 2 I5 fold: mk1 origin_fingerprint substitution NOTICE). The
/// secret-on-stdout warning never fires for export-wallet (watch-only by
/// SPEC §3). The parameter remains for callsite symmetry with the other
/// subcommands; the Phase 2 fold turns the prior `_stderr` into a live
/// channel for the `--from-import-json` consumer path only.
pub fn run<W: Write, E: Write>(
    args: &ExportWalletArgs,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // All six formats are now real (Phase 3 promoted Specter); no stubs.

    // v0.27.0 — `--from-import-json` dispatch short-circuits before slot /
    // template / descriptor pre-checks. The envelope carries everything the
    // emitter needs (descriptor + per-cosigner xpubs/fingerprints/paths).
    // Mutex with --template / --descriptor is enforced by clap via
    // `conflicts_with_all` on the `--from-import-json` arg.
    if args.from_import_json.is_some() {
        return run_from_import_json(args, stdout, stderr);
    }

    // SPEC §3 fast-path watch-only validator on the user-supplied raw slot
    // inputs. The SPEC-mandated invariant ("runs on the resolved-slot set") is
    // additionally enforced by `validate_watch_only_resolved` after
    // `resolve_slots` returns (see template branch below).
    validate_watch_only(&args.slot)?;

    // v0.8.2 SPEC §5.1 — `master_xpub` slot subkey is now plumbed through
    // `ResolvedSlot.master_xpub` and surfaced on `EmitInputs.master_xpub_at_0`
    // for the Coldcard generic-JSON emitter. The Phase 1.9 refuse-on-supply
    // guard is retired. Other formats silently ignore the subkey per the
    // per-format ignored-input contract.

    // SPEC v0.8 §7 — `tr-multi-a` / `tr-sortedmulti-a` require
    // `--taproot-internal-key`. The flag designates the BIP-341 internal key
    // (NUMS or cosigner index) for the canonical `tr(<internal>,multi_a(K,...))`
    // construction. v0.7 refused these templates outright; v0.8 supports them
    // with the internal-key designation.
    if let Some(template) = args.template {
        if matches!(
            template,
            CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA
        ) && args.taproot_internal_key.is_none()
        {
            return Err(ToolkitError::BadInput(format!(
                "--template {} requires --taproot-internal-key (use 'nums' for an unspendable BIP-341 NUMS point, or '@N' to designate cosigner N as the key-path internal key)",
                match template {
                    CliTemplate::TrMultiA => "tr-multi-a",
                    CliTemplate::TrSortedMultiA => "tr-sortedmulti-a",
                    _ => unreachable!(),
                },
            )));
        }
    }
    // SPEC v0.8 §7 — `--taproot-internal-key` is taproot-multisig-only.
    if args.taproot_internal_key.is_some()
        && !matches!(
            args.template,
            Some(CliTemplate::TrMultiA) | Some(CliTemplate::TrSortedMultiA)
        )
    {
        return Err(ToolkitError::BadInput(
            "--taproot-internal-key applies only to --template tr-multi-a / tr-sortedmulti-a"
                .into(),
        ));
    }

    // Mutual-exclusion + minimal arg surface checks.
    if args.descriptor.is_some() && args.template.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--template",
            message: "--descriptor and --template are mutually exclusive",
        });
    }
    if args.descriptor.is_none() && args.template.is_none() {
        return Err(ToolkitError::BadInput(
            "export-wallet requires either --template or --descriptor".into(),
        ));
    }

    // `resolved_template` carries the (resolved-slots, template, k) tuple
    // forward to the bip388 branch so we don't double-call `resolve_slots`.
    let mut resolved_template: Option<(Vec<crate::synthesize::ResolvedSlot>, CliTemplate, u8)> =
        None;

    // v0.53.8 (`bip388-policy-name-lossy-roundtrip`): when `--descriptor` is a
    // BIP-388 policy JSON, lift its `name` so a `--format bip388` round-trip
    // (and any other format) preserves it. Declared None UNCONDITIONALLY so the
    // `||` in `wallet_name_is_non_default` below is safe on the --template path.
    let mut bip388_policy_name: Option<String> = None;

    let canonical = if let Some(desc) = &args.descriptor {
        // BIP-388 wallet-policy JSON intake: expand to a concrete descriptor,
        // then fall into the existing concrete passthrough. MUST precede
        // is_at_n_form — a raw policy JSON matches the @N probe (its
        // description_template) AND the key_regex probe (its keys_info), so
        // unguarded it would trip the refusal below.
        let desc_owned;
        let desc = if crate::wallet_import::pipeline::is_bip388_policy_shape(desc) {
            bip388_policy_name = crate::wallet_import::pipeline::bip388_policy_name(desc);
            desc_owned = crate::wallet_import::pipeline::expand_bip388_policy(desc)?;
            &desc_owned
        } else {
            desc
        };
        // @N-probe ONLY (NOT classify_descriptor_form — its rule 4 would reject
        // origin-less concrete that passthrough accepts). SPEC §3.4.
        if crate::wallet_import::pipeline::is_at_n_form(desc) {
            return Err(ToolkitError::BadInput(
                "export-wallet --descriptor accepts only concrete descriptors with inline keys; \
                 for keyless @N templates use --template <T> --slot @N.xpub=… or --from-import-json".into(),
            ));
        }
        // Descriptor passthrough: parse + canonicalize via miniscript.
        use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
        use std::str::FromStr;
        let d = MsDescriptor::<DescriptorPublicKey>::from_str(desc).map_err(|e| {
            ToolkitError::DescriptorParse(format!("export-wallet --descriptor: {e}"))
        })?;
        let adv = crate::timelock_advisory::older_advisories_descriptor(&d);
        crate::timelock_advisory::emit_advisories(&adv, stderr);
        d.to_string()
    } else {
        let template = args.template.expect("checked above");
        // FOLLOWUP `multisig-tr-bip48-script-type-3-policy` (bless + warn):
        // taproot multisig under --multisig-path-family bip48 derives at the
        // non-standard m/48'/.../3'. Honor it, but advise on stderr.
        if let Some(w) = template
            .bip48_nonstandard_script_type_warning(args.multisig_path_family.unwrap_or_default())
        {
            let _ = writeln!(stderr, "{w}");
        }
        // Resolve slots through the shared bundle helper. Watch-only-only at
        // this point — phrase/entropy/xprv/wif rejected by validate_watch_only.
        let (resolved, _slip0132_signals) = crate::cmd::bundle::resolve_slots(
            &args.slot,
            template,
            args.network,
            args.account,
            args.language,
            None,
            args.multisig_path_family.unwrap_or_default(),
        )?;
        // SPEC §3 invariant: validator runs on the resolved-slot set.
        validate_watch_only_resolved(&resolved)?;
        let n = resolved.len() as u8;
        if n == 0 {
            return Err(ToolkitError::BadInput(
                "export-wallet: at least one --slot @N.xpub=... required".into(),
            ));
        }
        // For taproot multisig with a cosigner-internal key, the multi_a leaf
        // set has N-1 cosigners (one becomes the key-path key). Default
        // threshold = N for non-taproot or NUMS-internal; = N-1 when a
        // cosigner is internal. Caller may still override via --threshold.
        let leaf_count = match (template, args.taproot_internal_key) {
            (
                CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA,
                Some(TaprootInternalKey::Cosigner(_)),
            ) => n - 1,
            _ => n,
        };
        // n=1 cosigner-internal degenerate case: removing the only cosigner
        // leaves no multi_a leaves. Refuse cleanly here rather than letting
        // miniscript reject `multi_a(0,)` with an opaque parse error.
        if leaf_count == 0 {
            return Err(ToolkitError::BadInput(
                "--taproot-internal-key @N with a single cosigner leaves no multi_a leaves; supply at least 2 cosigners (or use --taproot-internal-key nums for unspendable key-path)".into(),
            ));
        }
        let k = args.threshold.unwrap_or(leaf_count);
        if k > leaf_count {
            // Distinguish multi_a-leaf-count error (cosigner-internal taproot)
            // from the general cosigner-count error to keep the existing
            // refusal text stable for non-taproot multisig.
            let msg = if leaf_count != n {
                format!("--threshold {k} exceeds multi_a leaf count {leaf_count} (one cosigner is the taproot internal key)")
            } else {
                format!("--threshold {k} exceeds cosigner count {n}")
            };
            return Err(ToolkitError::BadInput(msg));
        }
        if matches!(
            template,
            CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA
        ) && matches!(
            args.taproot_internal_key,
            Some(TaprootInternalKey::Cosigner(_))
        ) {
            // Validate cosigner index range.
            if let Some(TaprootInternalKey::Cosigner(idx)) = args.taproot_internal_key {
                if idx >= n {
                    return Err(ToolkitError::BadInput(format!(
                        "--taproot-internal-key @{idx} out of range; only {n} cosigners supplied",
                    )));
                }
            }
        }
        let canonical = build_descriptor_string(
            template,
            &resolved,
            k,
            args.network,
            args.account,
            args.taproot_internal_key,
        )?;
        resolved_template = Some((resolved, template, k));
        canonical
    };

    // Build EmitInputs once, after slot resolution + descriptor canonicalization
    // + watch-only validation. Each WalletFormatEmitter borrows this struct.
    // SPEC v0.8 §12 — trait dispatch replaces the per-format `Value` match
    // (which was followed by `serde_json::to_string_pretty`). Each emitter now
    // returns its final `String` directly; the v0.7 byte-exact fixtures for
    // `bitcoin-core` / `bip388` remain valid because `to_string_pretty` is
    // deterministic for a given input `Value`.
    let (resolved_slots_ref, template_opt, threshold_opt): (
        &[crate::synthesize::ResolvedSlot],
        Option<CliTemplate>,
        Option<u8>,
    ) = match &resolved_template {
        Some((slots, tmpl, k)) => (slots.as_slice(), Some(*tmpl), Some(*k)),
        None => (&[], None, None),
    };

    let script_type = if let Some(t) = template_opt {
        script_type_from_template(&t)
    } else {
        // Descriptor path: parse once for script-type classification.
        use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
        use std::str::FromStr;
        let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(&canonical).map_err(|e| {
            ToolkitError::DescriptorParse(format!("export-wallet script-type derive: {e}"))
        })?;
        script_type_from_descriptor(&parsed)?
    };

    // SPEC v0.8 §2 — wallet name: user-supplied via `--wallet-name <STRING>`
    // (Phase 1.7 wiring); falls back to `<template-human-name>-<account>` for
    // the template path or `"imported-descriptor"` for the descriptor path.
    // The `wallet_name_is_non_default` flag (v0.37.8 rename) lets Phase 3
    // SpecterEmitter distinguish non-default from default — Specter rejects
    // the silent `"imported-descriptor"` fallback. Both explicit `--wallet-name`
    // and v0.37.8's source-metadata lift on `--from-import-json` count as
    // non-default.
    let wallet_name_resolved: String = match args.wallet_name.as_ref() {
        Some(name) => name.clone(),
        None => match template_opt {
            Some(t) => format!("{}-{}", t.human_name(), args.account),
            // v0.53.8: a BIP-388 policy's `name` (lifted above) overrides the
            // "imported-descriptor" default; bip388 ⇒ --descriptor ⇒ template_opt
            // is None, so it lands in this leaf. Precedence: --wallet-name flag >
            // policy name > default.
            None => bip388_policy_name
                .clone()
                .unwrap_or_else(|| "imported-descriptor".to_string()),
        },
    };

    // v0.8.2 SPEC §5.1 — master_xpub plumbing. Surface @0.master_xpub= into
    // `EmitInputs.master_xpub_at_0` when slot 0 carries one (other slots'
    // master_xpub fields are not consumed by any current emitter, but
    // ResolvedSlot retains them for future per-cosigner needs).
    let master_xpub_at_0 = resolved_slots_ref.first().and_then(|s| s.master_xpub);

    let inputs = EmitInputs {
        canonical_descriptor: crate::wallet_export::CheckedDescriptor::new(&canonical)?,
        resolved_slots: resolved_slots_ref,
        template: template_opt,
        script_type,
        network: args.network,
        account: args.account,
        threshold: threshold_opt,
        threshold_user_supplied: args.threshold.is_some(),
        wallet_name: &wallet_name_resolved,
        // v0.53.8: a lifted BIP-388 policy name counts as non-default (mirrors
        // the import-json `lifted_wallet_name` path) so Specter accepts a named
        // policy instead of refusing the silent "imported-descriptor" default.
        wallet_name_is_non_default: args.wallet_name.is_some() || bip388_policy_name.is_some(),
        taproot_internal_key: args.taproot_internal_key,
        range: args.range,
        timestamp: args.timestamp.0,
        bitcoin_core_version: args.bitcoin_core_version,
        master_xpub_at_0,
        bsms_form: args.bsms_form,
    };

    // Shared 4-way dispatch (collect_missing-first → emit); see `emit_payload`.
    let emitted: String = emit_payload(&inputs, args.format)?;

    if args.output == "-" {
        // v0.27.0 Phase 6.5 PR-review C1 fold: propagate stdout write
        // failure (broken pipe / disk full / closed handle) as a typed
        // I/O error rather than silently exiting 0 with empty stdout.
        writeln!(stdout, "{emitted}").map_err(ToolkitError::Io)?;
        // Emit watch-only class advisory after stdout write (stdout branch only).
        crate::secret_advisory::emit_output_class_advisory(
            crate::secret_advisory::OutputClass::WatchOnly,
            stderr,
        );
    } else {
        std::fs::write(&args.output, format!("{emitted}\n"))
            .map_err(|e| ToolkitError::BadInput(format!("--output {}: {e}", args.output)))?;
    }
    Ok(())
}

/// v0.27.0 Phase 5 entry — `export-wallet --from-import-json <FILE|->`.
/// Consumes an `import-wallet --json` envelope (SPEC §3.2 wire shape;
/// Phase 4 ship) and emits a per-format wallet config. Per plan §3.7 +
/// §3.7.1 (16-field EmitInputs contract).
fn run_from_import_json<W: Write, E: Write>(
    args: &ExportWalletArgs,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    use crate::wallet_export::script_type_from_descriptor;
    use crate::wallet_import::json_envelope::{
        cli_network_from_str, descriptor_body_no_csum, envelope_to_resolved_slots,
        parse_import_json_envelopes,
    };

    // §3.7 Q6 — `--account` on the --from-import-json path is BadInput
    // (the envelope's bundle.account is authoritative; the user supplying
    // a separate account index is ambiguous / contradictory).
    if args.account != 0 {
        return Err(ToolkitError::BadInput(
            "--account is meaningful only with --template / --descriptor; \
             --from-import-json reads the account from the envelope (bundle.account)"
                .to_string(),
        ));
    }

    let value = args
        .from_import_json
        .as_ref()
        .expect("caller checked --from-import-json.is_some()");
    let raw = if value == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(ToolkitError::Io)?;
        buf
    } else {
        std::fs::read_to_string(value)
            .map_err(|e| ToolkitError::BadInput(format!("--from-import-json: read {value}: {e}")))?
    };
    let envelope =
        parse_import_json_envelopes(&raw, args.from_import_json_index, "--from-import-json")?;

    let descriptor_with_csum = envelope.bundle.descriptor.as_deref().ok_or_else(|| {
        ToolkitError::BadInput(
            "--from-import-json: envelope.bundle.descriptor is null; v0.27.0 \
                 wallet-import path always emits the descriptor string verbatim"
                .to_string(),
        )
    })?;
    // Validate the user-supplied BIP-380 checksum up-front; failure is
    // `BadInput` (Phase 5 R0 I1 fold) rather than silently passing through
    // to a downstream miniscript parse error. The body-only string drives
    // the miniscript parse for script-type derivation just below.
    let canonical_descriptor_body =
        descriptor_body_no_csum(descriptor_with_csum, "--from-import-json")?.to_string();

    // Decode mk1 → ResolvedSlots per §3.6.1. v0.27.1 Phase 2 I5 fold:
    // stderr carries the origin_fingerprint substitution NOTICE if any
    // mk1 card omits the master fingerprint.
    let resolved_slots = envelope_to_resolved_slots(&envelope, stderr)?;
    // (v0.37.9 — the F5 `path_raw` boundary band-aid is gone: emitters now
    // render origins via `ResolvedSlot::origin_path_bare()` / `bracketed_origin()`
    // derived from the typed `fingerprint`+`path`, so no `path_raw` normalization
    // is needed. FOLLOWUP `path-raw-bracketed-vs-bare-convention-unification`.)

    // Derive network from envelope.
    let network = cli_network_from_str(&envelope.bundle.network)?;

    // Script-type from the parsed descriptor (canonical form sans checksum).
    use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
    use std::str::FromStr;
    let parsed_ms = MsDescriptor::<DescriptorPublicKey>::from_str(&canonical_descriptor_body)
        .map_err(|e| {
            ToolkitError::DescriptorParse(format!(
                "--from-import-json: descriptor parse for script-type derivation: {e}"
            ))
        })?;
    let script_type = script_type_from_descriptor(&parsed_ms)?;

    // Task 9 (masked older() advisory): fire BEFORE the taproot refuse below,
    // so a consensus-masked older() is surfaced even when the command will
    // subsequently refuse a taproot envelope.
    let adv = crate::timelock_advisory::older_advisories_descriptor(&parsed_ms);
    crate::timelock_advisory::emit_advisories(&adv, stderr);

    // v0.28.7 — Slug 4 Fix-α: refuse taproot envelopes at the single
    // EmitInputs gate. The wallet_import path doesn't surface taproot
    // internal-key designation (NUMS vs raw xonly) in the envelope wire
    // shape; rather than propagate the gap silently to every emitter via
    // `taproot_internal_key: None`, refuse here. Detection uses parse-side
    // script_type (not string-sniff). Fix-β (envelope-field addition for
    // v0.29+) tracked at FOLLOWUP `wallet-import-taproot-internal-key`
    // (resolved v0.28.7 via Fix-α).
    if matches!(
        script_type,
        WalletScriptType::P2tr | WalletScriptType::P2trMulti
    ) {
        return Err(ToolkitError::BadInput(
            "--from-import-json: taproot descriptors are not yet supported on \
             the export-from-envelope path. The wallet_import path doesn't \
             surface taproot internal-key designation (NUMS vs raw xonly). \
             Use --format <emitter> --descriptor <body> directly, or wait \
             for v0.29+ envelope wire-shape evolution. FOLLOWUP: \
             `wallet-import-taproot-internal-key`."
                .into(),
        ));
    }

    // F9 fix (v0.28.2): re-emit via miniscript's canonical Display so
    // `canonical_descriptor` carries the BIP-380 `#<8-char>` checksum
    // suffix required by `EmitInputs.canonical_descriptor`'s invariant
    // (`wallet_export/bsms.rs:86-90`). Pre-fix, the body-only form above
    // flowed verbatim into BSMS L2 + Specter `descriptor` JSON + Green
    // plaintext, where downstream BSMS coordinators (Coldcard Mk4) reject
    // descriptor lines without the checksum. miniscript Display always
    // appends `#<csum>` per BIP-380 §Checksum-on-emit.
    let canonical_descriptor = parsed_ms.to_string();

    // Wallet name: --wallet-name explicit > envelope-lifted source name >
    // default `imported-descriptor`. v0.37.8 universal-name-lift: the
    // envelope carries the original wallet name in one of six per-format
    // source-metadata projections (sparrow.label / specter.label /
    // jade.coldcard_compat.name / electrum.wallet_name / source_metadata.
    // wallet_name / coldcard_multisig_source_metadata.name); lifting it
    // dissolves the Specter `MissingField::WalletName` refusal on
    // round-trip and gives sparrow/jade/coldcard-multisig/electrum a
    // semantically meaningful label instead of the placeholder. The
    // lifted name also flips `wallet_name_is_non_default = true` below
    // (the rename from `wallet_name_was_user_supplied` is the rename
    // that unlocks counting the lifted-name case as "non-default").
    let lifted_wallet_name: Option<String> = envelope.resolved_wallet_name();
    let wallet_name_resolved: String = args
        .wallet_name
        .clone()
        .or_else(|| lifted_wallet_name.clone())
        .unwrap_or_else(|| "imported-descriptor".to_string());

    // Threshold from envelope's bundle.multisig.threshold (None for N=1).
    let threshold = envelope.bundle.multisig.as_ref().map(|m| m.threshold);

    // SPEC v0.37 §2.3 — derive the template from the envelope descriptor for
    // template-requiring formats (sparrow/coldcard/jade/electrum) so they can
    // re-emit; passthrough formats keep None (bip388/sparrow branch on
    // template.is_some()). Taproot is already refused above (§2.4), so the
    // derivation never sees Tr.
    let derived_template: Option<CliTemplate> = if format_requires_template(args.format) {
        // C2: a template-requiring format (k-of-n multisig) cannot represent a
        // GENERAL miniscript policy (timelocks/hashlocks/andor/decay) — refuse
        // loudly rather than silently collapse it to plain multi via
        // `template_from_descriptor`'s `Wsh(_) => WshMulti` arm (the restore-C1
        // collapse class). Singlesig + plain multisig are NOT general → fall
        // through unchanged. Passthrough formats keep `None` (emit faithfully).
        if crate::wallet_export::descriptor_is_general_policy(&parsed_ms) {
            use clap::ValueEnum;
            let format_name = args
                .format
                .to_possible_value()
                .map(|v| v.get_name().to_string())
                .unwrap_or_default();
            return Err(ToolkitError::BadInput(format!(
                "--from-import-json: --format {format_name} cannot represent a general wallet \
                 policy (timelocks/hashlocks/non-multisig miniscript); it is a plain k-of-n \
                 multisig format. Use --format descriptor / bitcoin-core / bip388 for faithful \
                 descriptor passthrough."
            )));
        }
        Some(crate::wallet_export::template_from_descriptor(&parsed_ms)?)
    } else {
        None
    };

    let inputs = EmitInputs {
        canonical_descriptor: crate::wallet_export::CheckedDescriptor::new(&canonical_descriptor)?,
        resolved_slots: &resolved_slots,
        // v0.37: auto-derived for template-requiring formats; None for
        // descriptor-passthrough formats (preserves their passthrough path).
        template: derived_template,
        script_type,
        network,
        account: envelope.bundle.account,
        threshold,
        // envelope's bundle.multisig.threshold is authoritative when present;
        // mirrors the direct path's `threshold_user_supplied: args.threshold.is_some()`.
        threshold_user_supplied: threshold.is_some(),
        wallet_name: &wallet_name_resolved,
        wallet_name_is_non_default: args.wallet_name.is_some() || lifted_wallet_name.is_some(),
        // taproot internal key: v0.27.0 wallet-import path doesn't surface
        // taproot internal-key designation; reject taproot envelopes here
        // (file FOLLOWUP `wallet-import-taproot-internal-key` for v0.28+).
        taproot_internal_key: None,
        range: args.range,
        timestamp: args.timestamp.0,
        bitcoin_core_version: args.bitcoin_core_version,
        master_xpub_at_0: None,
        bsms_form: args.bsms_form,
    };

    // §3.7 — per-format missing-info channel + dispatch via the shared
    // `emit_payload` helper (collect_missing-first → emit); same 4-way dedup.
    let emitted: String = emit_payload(&inputs, args.format)?;

    if args.output == "-" {
        // v0.27.0 Phase 6.5 PR-review C1 fold: propagate stdout write
        // failure (broken pipe / disk full / closed handle) as a typed
        // I/O error rather than silently exiting 0 with empty stdout.
        writeln!(stdout, "{emitted}").map_err(ToolkitError::Io)?;
        // Emit watch-only class advisory after stdout write (stdout branch only).
        crate::secret_advisory::emit_output_class_advisory(
            crate::secret_advisory::OutputClass::WatchOnly,
            stderr,
        );
    } else {
        std::fs::write(&args.output, format!("{emitted}\n"))
            .map_err(|e| ToolkitError::BadInput(format!("--output {}: {e}", args.output)))?;
    }
    Ok(())
}

#[cfg(test)]
mod format_requires_template_tests {
    use super::{format_requires_template, CliExportFormat::*};

    /// v0.37.0 regression guard (SPEC §2.3): the exact template-requiring vs
    /// passthrough partition. If a future change flips a passthrough format
    /// (esp. bip388, which branches output on `template.is_some()`) into the
    /// inject set, this fails at the source — stronger than the behavioral
    /// `p11e_passthrough_formats_unaffected_by_autoderive` guard.
    #[test]
    fn partition_is_exact() {
        for f in [Sparrow, Coldcard, ColdcardMultisig, Jade, Electrum] {
            assert!(format_requires_template(f), "{f:?} must require a template");
        }
        for f in [BitcoinCore, Bip388, Bsms, Green, Specter, Descriptor] {
            assert!(
                !format_requires_template(f),
                "{f:?} must be passthrough (template stays None)"
            );
        }
    }
}

/// SPEC cycle-2 H10 — the structured `emit_payload` guard that refuses an
/// UNSORTED `wsh-multi` / `sh-wsh-multi` template to the field-less
/// electrum / coldcard(-multisig) / jade vendors (which are BIP-67
/// sortedmulti-only and would silently reorder the keys → wrong
/// witnessScript/address). Tests assert on the typed `kind()` /
/// `exit_code()` of the returned `ToolkitError`, the dimension the
/// process-exit-code-only integration tests cannot reach.
#[cfg(test)]
mod h10_unsorted_multi_refusal_tests {
    use super::{emit_payload, CliExportFormat};
    use crate::network::CliNetwork;
    use crate::template::CliTemplate;
    use crate::wallet_export::{BsmsForm, CheckedDescriptor, EmitInputs, TimestampArg, WalletScriptType};

    /// Build a minimal `EmitInputs` carrying a given template. The descriptor
    /// content is a placeholder — the H10 guard fires structurally on the
    /// typed `template` enum BEFORE any per-format emit / descriptor parse, so
    /// only the `#<csum>` suffix shape must be well-formed.
    fn inputs_with_template(template: Option<CliTemplate>) -> EmitInputs<'static> {
        EmitInputs {
            canonical_descriptor: CheckedDescriptor::new(
                "wsh(multi(2,xpubAAAA,xpubBBBB))#abcdefgh",
            )
            .unwrap(),
            resolved_slots: &[],
            template,
            script_type: WalletScriptType::P2wshMulti,
            network: CliNetwork::Mainnet,
            account: 0,
            threshold: Some(2),
            threshold_user_supplied: true,
            master_xpub_at_0: None,
            wallet_name: "h10-test",
            wallet_name_is_non_default: false,
            taproot_internal_key: None,
            range: (0, 999),
            timestamp: TimestampArg::Unix(0),
            bitcoin_core_version: 25,
            bsms_form: BsmsForm::FourLine,
        }
    }

    const FIELDLESS: [CliExportFormat; 4] = [
        CliExportFormat::Electrum,
        CliExportFormat::Coldcard,
        CliExportFormat::ColdcardMultisig,
        CliExportFormat::Jade,
    ];

    /// The two UNSORTED-multi templates → each field-less vendor → the typed
    /// `ExportWalletUnsortedMultisigUnsupported` refusal, exit 2.
    #[test]
    fn unsorted_multi_refused_typed_exit2_for_fieldless_vendors() {
        for tmpl in [CliTemplate::WshMulti, CliTemplate::ShWshMulti] {
            for fmt in FIELDLESS {
                let err = emit_payload(&inputs_with_template(Some(tmpl)), fmt)
                    .expect_err(&format!("{tmpl:?} → {fmt:?} must refuse"));
                assert_eq!(
                    err.kind(),
                    "ExportWalletUnsortedMultisigUnsupported",
                    "{tmpl:?} → {fmt:?} must be the typed H10 refusal, got {}",
                    err.kind()
                );
                assert_eq!(err.exit_code(), 2, "{tmpl:?} → {fmt:?} must exit 2");
            }
        }
    }

    /// The refusal message names a faithful format so the user has a recovery
    /// path (anti-dead-end; pins §2.4 wording at the byte level for the
    /// faithful-format pointer).
    #[test]
    fn unsorted_multi_refusal_message_points_to_faithful_format() {
        let err = emit_payload(
            &inputs_with_template(Some(CliTemplate::WshMulti)),
            CliExportFormat::Electrum,
        )
        .unwrap_err();
        let msg = err.message();
        assert!(
            msg.contains("descriptor"),
            "H10 refusal must point to a faithful format; got: {msg}"
        );
        assert!(
            msg.contains("electrum"),
            "H10 refusal must name the offending format; got: {msg}"
        );
    }

    /// SORTED-multi templates STILL export to the field-less vendors (BIP-67 is
    /// exactly what they implement) — the guard must NOT over-refuse. Any
    /// outcome other than the new typed refusal is acceptable here (the point
    /// is that the H10 guard does NOT fire); a sorted-multi export succeeds.
    #[test]
    fn sorted_multi_not_refused_by_h10_guard() {
        for tmpl in [
            CliTemplate::WshSortedMulti,
            CliTemplate::ShWshSortedMulti,
        ] {
            for fmt in FIELDLESS {
                let res = emit_payload(&inputs_with_template(Some(tmpl)), fmt);
                if let Err(e) = &res {
                    assert_ne!(
                        e.kind(),
                        "ExportWalletUnsortedMultisigUnsupported",
                        "{tmpl:?} → {fmt:?} must NOT hit the H10 unsorted-multi guard"
                    );
                }
            }
        }
    }

    /// Taproot multisig templates pass the H10 guard untouched (disjoint
    /// variant set) and hit their EXISTING per-emitter taproot refusal — proving
    /// §2.3/§2.5 disjointness: the new guard does not shadow the taproot guard.
    #[test]
    fn taproot_multi_hits_existing_taproot_guard_not_h10() {
        for tmpl in [CliTemplate::TrMultiA, CliTemplate::TrSortedMultiA] {
            for fmt in FIELDLESS {
                let err = emit_payload(&inputs_with_template(Some(tmpl)), fmt)
                    .expect_err(&format!("{tmpl:?} → {fmt:?} must refuse (taproot)"));
                assert_ne!(
                    err.kind(),
                    "ExportWalletUnsortedMultisigUnsupported",
                    "{tmpl:?} → {fmt:?} must NOT be the H10 refusal (it is the taproot guard)"
                );
            }
        }
    }

    /// Single-sig templates → field-less vendors: the H10 guard does NOT fire
    /// (it matches only the two unsorted-multi variants).
    #[test]
    fn single_sig_not_refused_by_h10_guard() {
        for tmpl in [CliTemplate::Bip44, CliTemplate::Bip49, CliTemplate::Bip84] {
            for fmt in FIELDLESS {
                let res = emit_payload(&inputs_with_template(Some(tmpl)), fmt);
                if let Err(e) = &res {
                    assert_ne!(
                        e.kind(),
                        "ExportWalletUnsortedMultisigUnsupported",
                        "{tmpl:?} → {fmt:?} must NOT hit the H10 guard"
                    );
                }
            }
        }
    }

    /// FAITHFUL formats (descriptor / sparrow / bitcoin-core) STILL accept an
    /// unsorted `wsh-multi` — they carry the literal `multi(` token and preserve
    /// key order, so the H10 guard must never fire for them.
    #[test]
    fn faithful_formats_not_refused_for_unsorted_multi() {
        for fmt in [
            CliExportFormat::Descriptor,
            CliExportFormat::Sparrow,
            CliExportFormat::BitcoinCore,
        ] {
            let res = emit_payload(
                &inputs_with_template(Some(CliTemplate::WshMulti)),
                fmt,
            );
            if let Err(e) = &res {
                assert_ne!(
                    e.kind(),
                    "ExportWalletUnsortedMultisigUnsupported",
                    "faithful {fmt:?} must NOT hit the H10 unsorted-multi guard"
                );
            }
        }
    }

    /// The direct `--descriptor` path resolves `template == None`; the H10 guard
    /// does NOT fire (and need not — it is already funds-safe). The field-less
    /// emitter's OWN generic `BadInput` ("requires --template") refuses it. This
    /// pins the typed-vs-generic boundary (§2.6 test 3): refused, but NOT by the
    /// new typed kind.
    #[test]
    fn template_none_falls_through_to_generic_badinput_not_h10() {
        for fmt in FIELDLESS {
            let err = emit_payload(&inputs_with_template(None), fmt)
                .expect_err(&format!("{fmt:?} with template=None must refuse"));
            assert_ne!(
                err.kind(),
                "ExportWalletUnsortedMultisigUnsupported",
                "{fmt:?} template=None must be the generic refusal, NOT the typed H10 error"
            );
        }
    }

    /// Import-json resolution mechanism (§2.6 test 2): an unsorted
    /// `wsh(multi)` / `sh(wsh(multi))` descriptor resolves to the unsorted
    /// `WshMulti` / `ShWshMulti` template (the value `run_from_import_json`
    /// feeds into `emit_payload`), so the structured guard fires. Sorted forms
    /// resolve to the sorted variants (allowed). Proves the import-json path
    /// feeds the guard a refused variant without standing up a full envelope.
    #[test]
    fn template_from_descriptor_preserves_unsorted_distinction() {
        use miniscript::{Descriptor, DescriptorPublicKey};
        use std::str::FromStr;
        let cases = [
            (
                "wsh(multi(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))",
                CliTemplate::WshMulti,
            ),
            (
                "wsh(sortedmulti(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))",
                CliTemplate::WshSortedMulti,
            ),
        ];
        for (desc, expected) in cases {
            let parsed = Descriptor::<DescriptorPublicKey>::from_str(desc).unwrap();
            let got = crate::wallet_export::template_from_descriptor(&parsed).unwrap();
            assert_eq!(got, expected, "{desc} must resolve to {expected:?}");
        }
    }
}
