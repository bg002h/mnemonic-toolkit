//! `mnemonic ms-shares` subcommand — BIP-93 codex32 K-of-N share split/combine.
//!
//! Realizes `design/SPEC_ms_v0_2_kofn.md` §4 (toolkit surface). Mirrors
//! `cmd/slip39.rs` structurally (split-or-combine sub-subcommand pattern,
//! `--from phrase=`/`entropy=` source grammar, `Zeroizing` wraps + mlock pins,
//! argv-leakage advisories, `PrivateKeyMaterial` output-class advisory).
//!
//! Unlike SLIP-39, codex32 K-of-N is a SIMPLE threshold (NOT slip39's
//! group×member model): `--threshold K` (2..=9) + `--shares N` (K..=31).
//!
//! Two sub-subcommands:
//!   - `split`: a secret (BIP-39 phrase or hex entropy) → N ms1 share strings.
//!     A non-English `--from phrase=` + `--language` produces a `mnem` secret
//!     so the wordlist language survives the split (it rides the secret-at-S
//!     wire bytes).
//!   - `combine`: ≥K ms1 shares → recovered secret, rendered per `--to`
//!     (`phrase` default | `entropy` | `ms1`). For `mnem` payloads the phrase
//!     is rendered in the card's wire language.

use crate::cmd::convert::{parse_from_input, read_stdin_to_string, FromInput, NodeType};
use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::secret_advisory::{emit_output_class_advisory, secret_in_argv_warning, OutputClass};
use bip39::Mnemonic;
use clap::{Args, Subcommand, ValueEnum};
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct MsSharesArgs {
    #[command(subcommand)]
    pub command: MsSharesCommand,
}

#[derive(Subcommand, Debug)]
pub enum MsSharesCommand {
    /// Split a secret into N codex32 K-of-N shares (any K recombine).
    Split(MsSharesSplitArgs),
    /// Combine ≥K codex32 shares back into the secret.
    Combine(MsSharesCombineArgs),
}

#[derive(Args, Debug, Clone)]
pub struct MsSharesSplitArgs {
    /// Secret as `phrase=<value-or->` OR `entropy=<hex-or->`.
    ///
    /// Inline forms emit an argv-leakage advisory (`/proc/$PID/cmdline`
    /// exposure); prefer the `=-` (stdin) variant for sensitive input.
    #[arg(
        long = "from",
        value_name = "phrase=<value-or--> or entropy=<hex-or-->",
        value_parser = parse_from_input,
        required = true,
    )]
    pub from: FromInput,

    /// Threshold K — the minimum number of shares needed to recombine.
    /// Must be in 2..=9 (codex32 threshold field is a single ASCII digit;
    /// '0' is the unshared single-string sentinel, '1' is invalid).
    #[arg(long = "threshold", required = true)]
    pub threshold: u8,

    /// Total number of shares N to emit. Must be in K..=31 (there are
    /// exactly 31 valid non-`s` codex32 share indices).
    #[arg(long = "shares", required = true)]
    pub shares: usize,

    /// BIP-39 language of the input phrase; ignored for `entropy=` inputs.
    /// A non-English language produces a `mnem` secret so the wordlist
    /// survives the split (English produces a plain `entr` secret).
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Insert a separator every N chars in each emitted share (0 = unbroken).
    /// SPEC §3. Display only; --json stays unbroken.
    #[arg(long, default_value_t = 5)]
    pub group_size: u16,

    /// Separator: space|hyphen|comma (keyword) or the literal " "|-|, . SPEC §5.
    #[arg(long, default_value = "space", value_parser = crate::display_grouping::parse_separator)]
    pub separator: char,

    /// Emit a JSON object on stdout (`{"shares": [...]}`) instead of the
    /// plain one-share-per-line text form.
    #[arg(long = "json", default_value_t = false)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct MsSharesCombineArgs {
    /// A codex32 K-of-N share string. Repeating; supply at least K. At most
    /// ONE may be `-` (stdin).
    ///
    /// Inline values emit a per-occurrence argv-leakage advisory; prefer
    /// `--share -` (stdin) for sensitive shares.
    #[arg(
        long = "share",
        value_name = "<ms1-share-or->",
        required = true,
        action = clap::ArgAction::Append,
    )]
    pub share: Vec<String>,

    /// Output shape: `phrase` (default; BIP-39 mnemonic in the recovered
    /// card's wire language), `entropy` (hex), or `ms1` (a recovered v0.1
    /// single-string ms1).
    #[arg(long = "to", default_value = "phrase")]
    pub to: MsSharesToShape,

    /// BIP-39 language for `--to phrase` when the recovered secret is a plain
    /// `entr` payload (no wire language). Ignored for `mnem` payloads (their
    /// wire language wins) and for `--to entropy`/`--to ms1`.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Insert a separator every N chars in a recovered `--to ms1` card
    /// (0 = unbroken). SPEC §3. Display only; --json + --to phrase/entropy stay raw.
    #[arg(long, default_value_t = 5)]
    pub group_size: u16,

    /// Separator: space|hyphen|comma (keyword) or the literal " "|-|, . SPEC §5.
    #[arg(long, default_value = "space", value_parser = crate::display_grouping::parse_separator)]
    pub separator: char,

    /// Emit a JSON object on stdout instead of the plain secret line.
    #[arg(long = "json", default_value_t = false)]
    pub json: bool,
}

/// `--to` output shape selector for `combine`.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum MsSharesToShape {
    /// BIP-39 mnemonic (language per the recovered card / `--language`).
    Phrase,
    /// Hex-encoded raw entropy bytes.
    Entropy,
    /// A recovered v0.1 single-string ms1.
    Ms1,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &MsSharesArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    match &args.command {
        MsSharesCommand::Split(a) => {
            // v0.26.0 §3 — resolve `@env:<VAR>` sentinels on `--from` before
            // downstream consumption (mirrors slip39's env-sentinel resolution).
            let owned_a;
            let a = if a.from.value.starts_with("@env:") {
                owned_a = resolve_split_env_sentinels(a)?;
                &owned_a
            } else {
                a
            };
            run_split(a, stdin, stdout, stderr)
        }
        MsSharesCommand::Combine(a) => {
            let owned_a;
            let a = if a.share.iter().any(|v| v.starts_with("@env:")) {
                owned_a = resolve_combine_env_sentinels(a)?;
                &owned_a
            } else {
                a
            };
            run_combine(a, stdin, stdout, stderr)
        }
    }
}

fn resolve_split_env_sentinels(
    args: &MsSharesSplitArgs,
) -> Result<MsSharesSplitArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    // Both `phrase=` and `entropy=` are secret-bearing.
    let flag = format!("--from {}=", owned.from.node.as_str());
    owned.from.value = resolve_env_var_sentinel(&owned.from.value, &flag)?;
    Ok(owned)
}

fn resolve_combine_env_sentinels(
    args: &MsSharesCombineArgs,
) -> Result<MsSharesCombineArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    for v in owned.share.iter_mut() {
        *v = resolve_env_var_sentinel(v, "--share")?;
    }
    Ok(owned)
}

/// Parse a `--from phrase=`/`entropy=` source into raw entropy bytes
/// (Zeroizing). Mirrors `slip39::parse_master_to_entropy` shape.
fn parse_secret_to_entropy(
    from: &FromInput,
    language: CliLanguage,
    raw_value: &str,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ToolkitError> {
    match from.node {
        NodeType::Phrase => {
            let word_count = raw_value.split_whitespace().count();
            if !matches!(word_count, 12 | 15 | 18 | 21 | 24) {
                return Err(ToolkitError::BadInput(format!(
                    "ms-shares split: input phrase must be 12/15/18/21/24 words; got {word_count}",
                )));
            }
            let lang: bip39::Language = language.into();
            let m = Mnemonic::parse_in(lang, raw_value).map_err(ToolkitError::Bip39)?;
            Ok(zeroize::Zeroizing::new(m.to_entropy()))
        }
        NodeType::Entropy => {
            let bytes = match hex::decode(raw_value) {
                Ok(b) => b,
                Err(_) => {
                    let assumed = raw_value.len() / 2;
                    return Err(ToolkitError::BadInput(format!(
                        "ms-shares split: entropy hex must decode to 16/20/24/28/32 bytes; got {assumed} bytes",
                    )));
                }
            };
            if !matches!(bytes.len(), 16 | 20 | 24 | 28 | 32) {
                return Err(ToolkitError::BadInput(format!(
                    "ms-shares split: entropy hex must decode to 16/20/24/28/32 bytes; got {} bytes",
                    bytes.len(),
                )));
            }
            Ok(zeroize::Zeroizing::new(bytes))
        }
        _ => Err(ToolkitError::BadInput(format!(
            "ms-shares split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got {}=",
            from.node.as_str(),
        ))),
    }
}

fn run_split<R: Read, W: Write, E: Write>(
    args: &MsSharesSplitArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // --from variant must be phrase= or entropy= (checked here so the message
    // names the node; parse_secret_to_entropy enforces the same).
    if args.from.node != NodeType::Phrase && args.from.node != NodeType::Entropy {
        return Err(ToolkitError::BadInput(format!(
            "ms-shares split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got {}=",
            args.from.node.as_str(),
        )));
    }

    // argv-leakage advisory for an inline --from value (non-stdin).
    if args.from.value != "-" {
        match args.from.node {
            NodeType::Phrase => secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-"),
            NodeType::Entropy => {
                secret_in_argv_warning(stderr, "--from entropy=", "--from entropy=-")
            }
            _ => unreachable!("pre-check above enforces phrase/entropy node"),
        }
    }

    // Resolve --from value (Zeroizing + mlock pin).
    let from_value: zeroize::Zeroizing<String> = if args.from.value == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(args.from.value.clone())
    };
    let _pin_from = mnemonic_toolkit::mlock::pin_pages_for(from_value.as_bytes());

    // Parse the secret to entropy bytes.
    let entropy = parse_secret_to_entropy(&args.from, args.language, from_value.as_str())?;
    let _pin_entropy = mnemonic_toolkit::mlock::pin_pages_for(entropy.as_slice());

    // Build the ms-codec payload. A non-English phrase produces a `mnem`
    // secret (language survives the split); English / entropy-source produce a
    // plain `entr`. `entropy=` inputs ignore --language (the bytes carry no
    // wordlist), matching the toolkit's encode/convert path.
    let payload = if args.from.node == NodeType::Phrase && args.language != CliLanguage::English {
        ms_codec::Payload::Mnem {
            language: crate::language::cli_language_to_wire_code(args.language),
            entropy: (*entropy).clone(),
        }
    } else {
        ms_codec::Payload::Entr((*entropy).clone())
    };

    // Threshold validation (2..=9) → InvalidThreshold; encode_shares validates
    // K <= N <= 31 → InvalidShareCount. Both surface via friendly_ms_codec.
    let threshold = ms_codec::Threshold::new(args.threshold).map_err(ToolkitError::from)?;
    let shares = ms_codec::encode_shares(ms_codec::Tag::ENTR, threshold, args.shares, &payload)
        .map_err(ToolkitError::from)?;

    // Wrap each rendered share in Zeroizing + mlock-pin inside the emit loop
    // (mirrors slip39's O(N) per-share pinning).
    let rendered: Vec<zeroize::Zeroizing<String>> =
        shares.into_iter().map(zeroize::Zeroizing::new).collect();

    if args.json {
        let envelope = SplitJson {
            schema_version: "1",
            operation: "split",
            threshold: args.threshold,
            shares: rendered.iter().map(|s| s.as_str()).collect(),
        };
        let body = serde_json::to_string(&envelope)
            .map_err(|e| ToolkitError::BadInput(format!("--json serialize: {e}")))?;
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(body.as_bytes());
        writeln!(stdout, "{body}")
            .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;
    } else {
        // mstring display-grouping (SPEC §6): text shares are flag-controlled
        // (default space/5); the --json branch above stays unbroken.
        let gs = args.group_size as usize;
        let sep = args.separator;
        for s in &rendered {
            let _pin = mnemonic_toolkit::mlock::pin_pages_for(s.as_bytes());
            writeln!(stdout, "{}", crate::display_grouping::render_grouped(s.as_str(), gs, sep))
                .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;
        }
    }

    // The whole N-share SET is secret-equivalent → PrivateKeyMaterial.
    emit_output_class_advisory(OutputClass::PrivateKeyMaterial, stderr);
    let _ = writeln!(
        stderr,
        "note: each share is secret material — distribute across separate locations; \
        any {k} of {n} recombine via `mnemonic ms-shares combine`",
        k = args.threshold,
        n = args.shares,
    );

    Ok(0)
}

fn run_combine<R: Read, W: Write, E: Write>(
    args: &MsSharesCombineArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // Single stdin consumer per invocation.
    let stdin_count = args.share.iter().filter(|s| *s == "-").count();
    if stdin_count > 1 {
        return Err(ToolkitError::BadInput(
            "ms-shares combine: at most one stdin consumer per invocation (across --share)".into(),
        ));
    }

    // Per-share inline argv-leakage advisory.
    for sh in &args.share {
        if sh != "-" {
            secret_in_argv_warning(stderr, "--share", "--share -");
        }
    }

    // Resolve --share values (stdin or inline) into Zeroizing<String>.
    let mut share_strings: Vec<zeroize::Zeroizing<String>> = Vec::with_capacity(args.share.len());
    let mut stdin_consumed = false;
    for sh in &args.share {
        let s = if sh == "-" {
            if stdin_consumed {
                return Err(ToolkitError::BadInput(
                    "ms-shares combine: at most one stdin consumer per invocation".into(),
                ));
            }
            stdin_consumed = true;
            zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
        } else {
            zeroize::Zeroizing::new(sh.clone())
        };
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(s.as_bytes());
        share_strings.push(s);
    }

    // Drop empty entries (e.g. a blank stdin read), then require ≥1. The trimmed
    // copies are secret share material — wrap each in `Zeroizing` so the residue
    // is wiped on drop (M1, P3-R0; the pinned `Zeroizing` originals in
    // `share_strings` already exist, this removes the trimmed-clone residue).
    // mstring display-grouping (SPEC §3.2): strip display separators from each
    // share so a grouped or unbroken share both re-ingest (was edge-only trim).
    let shares: Vec<zeroize::Zeroizing<String>> = share_strings
        .iter()
        .map(|s| {
            zeroize::Zeroizing::new(crate::display_grouping::strip_display_separators(s))
        })
        .filter(|s| !s.is_empty())
        .collect();
    if shares.is_empty() {
        return Err(ToolkitError::BadInput(
            "ms-shares combine: at least one --share required".into(),
        ));
    }
    for s in &shares {
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(s.as_bytes());
    }

    // Recombine → (Tag, Payload). Surfaces SecretShareSuppliedToCombine /
    // Codex32(ThresholdNotPassed/Mismatched*/RepeatedIndex) via friendly_ms_codec.
    // `combine_shares` takes `&[String]`; build a transient `Zeroizing` view (it
    // clones each share internally regardless — and this view wipes on drop, so
    // no longer-lived plaintext copy escapes).
    let shares_view: zeroize::Zeroizing<Vec<String>> =
        zeroize::Zeroizing::new(shares.iter().map(|s| (**s).clone()).collect());
    let (tag, payload) = ms_codec::combine_shares(&shares_view).map_err(ToolkitError::from)?;

    // Project the recovered secret per --to. `recovered_lang` is `Some` ONLY for
    // a `mnem` payload — the wordlist language that rides the secret-at-S wire
    // bytes. It is the source of truth for the I1 language-loss advisory (NOT
    // `args.language`, which is ignored on combine of a mnem set, and irrelevant
    // for an entr set that carries no language).
    let (entropy, payload_lang, recovered_lang): (
        zeroize::Zeroizing<Vec<u8>>,
        bip39::Language,
        Option<CliLanguage>,
    ) = match &payload {
        ms_codec::Payload::Entr(bytes) => {
            let l: bip39::Language = args.language.into();
            (zeroize::Zeroizing::new(bytes.clone()), l, None)
        }
        ms_codec::Payload::Mnem {
            entropy,
            language: wire_lang,
        } => {
            let lang = crate::language::wire_code_to_bip39(*wire_lang)?;
            let cli = crate::language::wire_code_to_cli(*wire_lang);
            (zeroize::Zeroizing::new(entropy.clone()), lang, cli)
        }
        _ => {
            return Err(ToolkitError::BadInput(
                "ms-shares combine: recovered an unknown payload kind".into(),
            ))
        }
    };
    let _pin_entropy = mnemonic_toolkit::mlock::pin_pages_for(entropy.as_slice());

    // I1 (P3-R0): `--to entropy` drops the wordlist language carried by a mnem
    // share-set — raw entropy is bytes only. Mirror `slip39.rs::run_combine`:
    // emit the non-English seed advisory keyed off the RECOVERED payload's
    // language. `--to phrase` re-renders in the card language and `--to ms1`
    // re-encodes the mnem payload (payload_kind Mnem + language preserved), so
    // neither loses the language — no advisory on those arms.
    if matches!(args.to, MsSharesToShape::Entropy) {
        if let Some(cli_lang) = recovered_lang {
            if let Some(msg) = crate::language::non_english_seed_advisory(cli_lang, "raw entropy") {
                let _ = writeln!(stderr, "{msg}");
            }
        }
    }

    let output: zeroize::Zeroizing<String> = match args.to {
        MsSharesToShape::Phrase => {
            // SAFETY: third-party-blocked — `bip39::Mnemonic` has no
            // Drop+Zeroize; FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`.
            let m = Mnemonic::from_entropy_in(payload_lang, &entropy[..])
                .map_err(ToolkitError::Bip39)?;
            zeroize::Zeroizing::new(m.to_string())
        }
        MsSharesToShape::Entropy => zeroize::Zeroizing::new(hex::encode(&entropy[..])),
        MsSharesToShape::Ms1 => {
            // Re-encode the recovered secret as a v0.1 single-string ms1
            // (threshold 0). The payload kind (entr/mnem) + wire language are
            // preserved through `encode`.
            zeroize::Zeroizing::new(ms_codec::encode(tag, &payload).map_err(ToolkitError::from)?)
        }
    };
    let _pin_out = mnemonic_toolkit::mlock::pin_pages_for(output.as_bytes());

    if args.json {
        let (output_shape, phrase, entropy_hex, ms1) = match args.to {
            MsSharesToShape::Phrase => ("phrase", Some(output.as_str()), None, None),
            MsSharesToShape::Entropy => ("entropy", None, Some(output.as_str()), None),
            MsSharesToShape::Ms1 => ("ms1", None, None, Some(output.as_str())),
        };
        let envelope = CombineJson {
            schema_version: "1",
            operation: "combine",
            output_shape,
            phrase,
            entropy_hex,
            ms1,
        };
        let body = serde_json::to_string(&envelope)
            .map_err(|e| ToolkitError::BadInput(format!("--json serialize: {e}")))?;
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(body.as_bytes());
        writeln!(stdout, "{body}")
            .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;
    } else {
        // mstring display-grouping (SPEC §6): group ONLY a recovered `--to ms1`
        // card; --to phrase/entropy emit raw. --json stays unbroken (above).
        let rendered = match args.to {
            MsSharesToShape::Ms1 => crate::display_grouping::render_grouped(
                output.as_str(),
                args.group_size as usize,
                args.separator,
            ),
            _ => output.to_string(),
        };
        writeln!(stdout, "{rendered}")
            .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;
    }

    // The recovered secret is PrivateKeyMaterial.
    emit_output_class_advisory(OutputClass::PrivateKeyMaterial, stderr);
    let _ = writeln!(
        stderr,
        "note: verify the recovered wallet's expected derived address before trusting",
    );

    Ok(0)
}

// ============================================================
// JSON envelope structs (field order is part of the wire shape; the toolkit
// `--json` wire-shape is NOT schema-mirror-gated — paired-PR self-update).
// ============================================================

#[derive(serde::Serialize)]
struct SplitJson<'a> {
    schema_version: &'static str,
    operation: &'static str,
    threshold: u8,
    shares: Vec<&'a str>,
}

#[derive(serde::Serialize)]
struct CombineJson<'a> {
    schema_version: &'static str,
    operation: &'static str,
    output_shape: &'static str,
    phrase: Option<&'a str>,
    entropy_hex: Option<&'a str>,
    ms1: Option<&'a str>,
}
