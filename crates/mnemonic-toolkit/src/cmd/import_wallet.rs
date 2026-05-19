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
use crate::language::CliLanguage;
use crate::slot_input::{SlotInput, SlotSubkey};
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

#[derive(Args, Debug, Clone)]
pub struct ImportWalletArgs {
    /// Path to the third-party wallet blob; `-` reads from stdin.
    #[arg(long = "blob", value_name = "FILE|-", required = true)]
    pub blob: PathBuf,

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
    ///
    /// When `--json` is set, the round-trip diff goes ONLY in the envelope;
    /// stderr is silent for the diff.
    #[arg(long = "json")]
    pub json: bool,
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

    // Read blob.
    let blob = read_blob(&args.blob, stdin)?;

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
        emit_json_envelope(stdout, &parsed, &blob, format_str, args.json)?;
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

/// SPEC §7.4 — emit the JSON envelope array on stdout. Each element
/// corresponds to one `ParsedImport`. Round-trip is computed against the
/// ORIGINAL blob (after filtering) via canonicalize_<format>.
///
/// **BSMS round-trip caveat (SPEC §7.3.1 policy):** there is no
/// `mnemonic export-wallet --format bsms` emitter in v0.26.x (FOLLOWUP
/// `wallet-export-bsms-emitter`). Without an emitter we cannot produce
/// a fresh blob to compare against the original; instead we emit
/// `roundtrip: { byte_exact: false, semantic_match: false, diff: null,
/// status: "blocked_no_emitter" }`. The same applies to BSMS at default-
/// mode stderr WARNING emission (see `emit_roundtrip_stderr_warning`).
fn emit_json_envelope<W: Write>(
    stdout: &mut W,
    parsed: &[ParsedImport],
    blob: &[u8],
    format_str: &str,
    _json: bool,
) -> Result<(), ToolkitError> {
    let mut envelopes: Vec<serde_json::Value> = Vec::with_capacity(parsed.len());

    let canon_orig = match format_str {
        "bsms" => canonicalize_bsms(blob).ok(),
        "bitcoin-core" => canonicalize_bitcoin_core(blob).ok(),
        _ => None,
    };

    for p in parsed {
        // Build a compact bundle-view summary. The full BundleJson shape
        // (with synthesized ms1/mk1/md1 cards) is NOT produced by import-
        // wallet in v0.26.0 — synthesis happens in a separate `bundle`
        // pipeline. The summary here is sufficient for downstream
        // consumers to identify the parsed cosigners + carry the entropy
        // attached via seed overlay.
        let cosigners_json: Vec<serde_json::Value> = p
            .cosigners
            .iter()
            .map(|c| {
                json!({
                    "fingerprint": format!("{:08x}", u32::from_be_bytes(c.fingerprint.to_bytes())),
                    "path_raw": c.path_raw,
                    "xpub": c.xpub.to_string(),
                    "has_entropy": c.entropy.is_some(),
                })
            })
            .collect();
        let network_name = network_human_name(p.network);
        let bundle_view = json!({
            "cosigners": cosigners_json,
            "network": network_name,
            "threshold": p.threshold,
        });

        // Round-trip per SPEC §7.4 + §7.3.
        let roundtrip = match format_str {
            "bitcoin-core" => {
                // For Bitcoin Core, we re-canonicalize the same blob (no
                // separate emit step in v0.26.0 — emit-from-bundle is the
                // sibling `mnemonic export-wallet --format bitcoin-core`
                // pipeline). The byte-exact check is original-bytes-vs-
                // canonical; semantic_match is true ONLY when canonicalize
                // succeeded. If canonicalize failed (e.g., exotic descriptor
                // that `BitcoinCoreParser::parse` accepted but the
                // canonicalize path rejected), surface that explicitly via
                // `status: "canonicalize_failed"` rather than silently
                // claiming success.
                match canon_orig.clone() {
                    Some(canon) => {
                        let original_text =
                            std::str::from_utf8(blob).unwrap_or("").to_string();
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
                }
            }
            "bsms" => {
                // SPEC §7.3.1 policy: BSMS export emitter does not exist
                // in v0.26.0 (FOLLOWUP `wallet-export-bsms-emitter`).
                // Emit a non-misleading envelope per SPEC §7.4.
                json!({
                    "byte_exact": false,
                    "semantic_match": false,
                    "diff": serde_json::Value::Null,
                    "status": "blocked_no_emitter",
                })
            }
            _ => json!({}),
        };

        let mut env = serde_json::Map::new();
        env.insert("bundle".to_string(), bundle_view);
        env.insert("source_format".to_string(), json!(format_str));
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

        envelopes.push(serde_json::Value::Object(env));
    }

    let text = serde_json::to_string_pretty(&serde_json::Value::Array(envelopes))
        .map_err(|e| ToolkitError::BadInput(format!("import-wallet --json serialize: {e}")))?;
    writeln!(stdout, "{text}").map_err(ToolkitError::Io)?;
    Ok(())
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

fn network_human_name(n: bitcoin::Network) -> &'static str {
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
