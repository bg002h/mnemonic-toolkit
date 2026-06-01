//! `mnemonic seed-xor` subcommand — Coldcard-compatible BIP-39 ↔ BIP-39
//! all-or-nothing XOR-based seed splitter.
//!
//! Realizes `design/SPEC_seed_xor_v0_12_0.md` §2.2. Two sub-subcommands:
//!   - `split`: master phrase → N XOR shares (each a valid BIP-39 phrase).
//!   - `combine`: N shares → master phrase.
//!
//! Cycle A/B discipline rails:
//!   - argv-leakage advisory via `secret_in_argv_warning` for inline secrets
//!   - `Zeroizing<String>` for parsed inputs
//!   - mlock Site 1 pins on parsed entropy bytes
//!   - new K-of-N stdout-on-TTY advisory class (first toolkit use)
//!   - `#[cfg(unix)]` permission-mode advisory on `--json-out`

use crate::cmd::convert::{parse_from_input, read_stdin_to_string, FromInput, NodeType};
use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::secret_advisory::{secret_in_argv_warning, warn_if_world_readable};
use bip39::Mnemonic;
use clap::{Args, Subcommand};
use mnemonic_toolkit::seed_xor::{
    seed_xor_combine, seed_xor_split, seed_xor_split_deterministic, SeedXorError,
};
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct SeedXorArgs {
    #[command(subcommand)]
    pub command: SeedXorCommand,
}

#[derive(Subcommand, Debug)]
pub enum SeedXorCommand {
    /// Split a BIP-39 phrase into N XOR shares (each a valid BIP-39 phrase).
    Split(SeedXorSplitArgs),
    /// Combine N XOR shares back into a BIP-39 phrase.
    Combine(SeedXorCombineArgs),
}

#[derive(Args, Debug, Clone)]
pub struct SeedXorSplitArgs {
    /// Master phrase as `phrase=<value>` (inline) or `phrase=-` (stdin).
    ///
    /// Inline form emits an argv-leakage advisory (`/proc/$PID/cmdline`
    /// exposure); prefer `phrase=-` for sensitive input.
    #[arg(
        long = "from",
        value_name = "phrase=<value-or-->",
        value_parser = parse_from_input,
        required = true,
    )]
    pub from: FromInput,

    /// Number of shares to emit. Must be >= 2.
    #[arg(long = "shares", required = true)]
    pub shares: usize,

    /// BIP-39 language of input + output. Defaults to english.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Use Coldcard's SHA256d-deterministic share generation instead of
    /// the OS CSPRNG. Required for byte-equal Coldcard hardware interop.
    #[arg(long = "deterministic-from-master")]
    pub deterministic_from_master: bool,

    /// Side-effect: write a versioned JSON envelope to this path.
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SeedXorCombineArgs {
    /// Share phrase as `phrase=<value>` (inline) or `phrase=-` (stdin).
    /// Repeating; at most ONE may be `phrase=-` (single stdin per invocation).
    #[arg(
        long = "share",
        action = clap::ArgAction::Append,
        value_parser = parse_from_input,
        required = true,
    )]
    pub share: Vec<FromInput>,

    /// Asserted share count. Must equal the number of `--share` flags.
    #[arg(long = "shares", required = true)]
    pub shares: usize,

    /// BIP-39 language of inputs + output. Defaults to english.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Side-effect: write a versioned JSON envelope to this path.
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &SeedXorArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    match &args.command {
        SeedXorCommand::Split(a) => {
            // v0.26.0 §3 — resolve `@env:<VAR>` sentinels on `--from`
            // before downstream consumption. `--from phrase=` is the only
            // accepted shape (row 8 enforced post-resolution).
            let owned_a;
            let a = if a.from.value.starts_with("@env:") {
                owned_a = resolve_split_env_sentinels(a)?;
                &owned_a
            } else {
                a
            };
            run_split(a, stdin, stdout, stderr)
        }
        SeedXorCommand::Combine(a) => {
            // v0.26.0 §3 — resolve `@env:<VAR>` sentinels on `--share`
            // values (all are `phrase=` secret-bearing per row 8).
            let owned_a;
            let a = if a.share.iter().any(|s| s.value.starts_with("@env:")) {
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
    args: &SeedXorSplitArgs,
) -> Result<SeedXorSplitArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    // `--from phrase=` is the only accepted shape (refusal below at row 8);
    // we resolve the sentinel here so the refusal still fires with the
    // post-resolution value, which preserves user-facing behavior.
    let flag = format!("--from {}=", owned.from.node.as_str());
    owned.from.value = resolve_env_var_sentinel(&owned.from.value, &flag)?;
    Ok(owned)
}

fn resolve_combine_env_sentinels(
    args: &SeedXorCombineArgs,
) -> Result<SeedXorCombineArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    for sh in owned.share.iter_mut() {
        let flag = format!("--share {}=", sh.node.as_str());
        sh.value = resolve_env_var_sentinel(&sh.value, &flag)?;
    }
    Ok(owned)
}

fn run_split<R: Read, W: Write, E: Write>(
    args: &SeedXorSplitArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    if args.from.node != NodeType::Phrase {
        return Err(ToolkitError::BadInput(
            "seed-xor only accepts phrase=<value> or phrase=-".into(),
        ));
    }

    if args.from.value != "-" {
        secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-");
    }

    let master_phrase: zeroize::Zeroizing<String> = if args.from.value == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(args.from.value.clone())
    };
    let _pin_master = mnemonic_toolkit::mlock::pin_pages_for(master_phrase.as_bytes());

    let lang: bip39::Language = args.language.into();
    let mnemonic = Mnemonic::parse_in(lang, master_phrase.as_str())
        .map_err(ToolkitError::Bip39)?;

    let word_count = mnemonic.word_count();
    if !matches!(word_count, 12 | 15 | 18 | 21 | 24) {
        return Err(ToolkitError::BadInput(format!(
            "seed-xor split: phrase must be 12/15/18/21/24 words; got {word_count}",
        )));
    }

    let entropy: zeroize::Zeroizing<Vec<u8>> =
        zeroize::Zeroizing::new(mnemonic.to_entropy());
    let _pin_entropy = mnemonic_toolkit::mlock::pin_pages_for(entropy.as_slice());

    let shares_bytes = if args.deterministic_from_master {
        seed_xor_split_deterministic(entropy.as_slice(), args.shares)
            .map_err(map_seed_xor_error)?
    } else {
        let mut rng = rand_core::OsRng;
        seed_xor_split(entropy.as_slice(), args.shares, &mut rng)
            .map_err(map_seed_xor_error)?
    };

    // Per-share BIP-39 checksum recompute via Mnemonic::from_entropy_in.
    let share_phrases: Vec<zeroize::Zeroizing<String>> = shares_bytes
        .iter()
        .map(|s| {
            Mnemonic::from_entropy_in(lang, s.as_slice())
                .map(|m| zeroize::Zeroizing::new(m.to_string()))
                .map_err(ToolkitError::Bip39)
        })
        .collect::<Result<Vec<_>, _>>()?;

    for phrase in &share_phrases {
        writeln!(stdout, "{}", phrase.as_str())
            .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;
    }

    if let Some(path) = &args.json_out {
        write_split_json(
            path,
            args.language.human_name(),
            word_count,
            args.deterministic_from_master,
            &share_phrases,
            stderr,
        )?;
    }

    // SPEC §2.6 row 5 — deterministic + 15/21 toolkit-only advisory.
    if args.deterministic_from_master && matches!(word_count, 15 | 21) {
        let _ = writeln!(
            stderr,
            "warning: --deterministic-from-master with {word_count}-word input is toolkit-only — \
             Coldcard's xor_seed.py natively supports 12/18/24 only; resulting shares will NOT \
             round-trip a Coldcard device. For Coldcard interop, use 12/18/24-word input.",
        );
    }

    // SPEC §2.6 row 2 — emit class advisory unconditionally (TTY gate dropped, Cycle B P1).
    // Addendum with the bespoke safety clause follows the unified line.
    if !share_phrases.is_empty() {
        crate::secret_advisory::emit_output_class_advisory(
            crate::secret_advisory::OutputClass::PrivateKeyMaterial,
            stderr,
        );
        let _ = writeln!(
            stderr,
            "warning: Seed XOR shares on stdout — each of the N={} lines is independently a complete BIP-39 phrase; ALL N shares are required to reconstruct the master; distribute them to N separate locations; do not paste this output into a single untrusted tool. Substitution of a wrong-but-valid-BIP-39 share is undetectable by Seed XOR — verify the recovered wallet's derived address before trusting it.",
            share_phrases.len(),
        );
    }

    Ok(0)
}

fn run_combine<R: Read, W: Write, E: Write>(
    args: &SeedXorCombineArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // SPEC §2.5 row 8 — non-phrase variants refuse.
    for sh in &args.share {
        if sh.node != NodeType::Phrase {
            return Err(ToolkitError::BadInput(
                "seed-xor only accepts phrase=<value> or phrase=-".into(),
            ));
        }
    }

    // SPEC §2.5 row 9 — multi-stdin refusal.
    let stdin_count = args.share.iter().filter(|s| s.value == "-").count();
    if stdin_count > 1 {
        return Err(ToolkitError::BadInput(
            "seed-xor combine: at most one --share value may be `-` (single stdin per invocation)"
                .into(),
        ));
    }

    // SPEC §2.5 row 3 — cardinality assertion.
    if args.share.len() != args.shares {
        return Err(ToolkitError::BadInput(format!(
            "seed-xor combine: --shares {} requires exactly {} --share arguments; got {} --share values for --shares {}",
            args.shares, args.shares, args.share.len(), args.shares,
        )));
    }

    // Argv-leakage advisory per-occurrence for inline share values.
    for sh in &args.share {
        if sh.value != "-" {
            secret_in_argv_warning(stderr, "--share phrase=", "--share phrase=-");
        }
    }

    // Resolve all shares into Zeroizing<String>.
    let mut share_strings: Vec<zeroize::Zeroizing<String>> = Vec::with_capacity(args.share.len());
    let mut stdin_consumed = false;
    for sh in &args.share {
        let s = if sh.value == "-" {
            if stdin_consumed {
                // Defensive: already rejected by stdin_count check above, but be belt-and-suspenders.
                return Err(ToolkitError::BadInput(
                    "seed-xor combine: at most one --share value may be `-` (single stdin per invocation)".into(),
                ));
            }
            stdin_consumed = true;
            zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
        } else {
            zeroize::Zeroizing::new(sh.value.clone())
        };
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(s.as_bytes());
        share_strings.push(s);
    }

    let lang: bip39::Language = args.language.into();

    // Parse each as BIP-39 in the chosen language; collect entropies.
    let mut share_entropies: Vec<zeroize::Zeroizing<Vec<u8>>> =
        Vec::with_capacity(share_strings.len());
    for (i, s) in share_strings.iter().enumerate() {
        let m = Mnemonic::parse_in(lang, s.as_str()).map_err(|e| match e {
            bip39::Error::UnknownWord(idx) => ToolkitError::BadInput(format!(
                "seed-xor combine: share at position {i}: unknown BIP-39 word at index {idx} (not in selected wordlist; did you pick the right --language?)",
            )),
            other => ToolkitError::BadInput(format!(
                "seed-xor combine: share at position {i} has invalid BIP-39 checksum (not a parseable mnemonic in --language {}): {other}",
                args.language.human_name(),
            )),
        })?;
        share_entropies.push(zeroize::Zeroizing::new(m.to_entropy()));
    }

    // SPEC §2.5 row 4 — mixed-length shares refuse.
    let lengths: Vec<usize> = share_entropies.iter().map(|e| e.len()).collect();
    let first_len = lengths[0];
    if lengths.iter().any(|&l| l != first_len) {
        let word_counts: Vec<usize> = lengths.iter().map(|&l| entropy_bytes_to_word_count(l)).collect();
        return Err(ToolkitError::BadInput(format!(
            "seed-xor combine: all shares must be the same word count; got mix of {word_counts:?}",
        )));
    }

    // Library combine.
    let refs: Vec<&[u8]> = share_entropies.iter().map(|e| e.as_slice()).collect();
    let recovered = seed_xor_combine(&refs).map_err(map_seed_xor_error)?;
    let _pin_recovered = mnemonic_toolkit::mlock::pin_pages_for(recovered.as_slice());

    // Convert recovered entropy back to BIP-39 phrase in the same language.
    let phrase = Mnemonic::from_entropy_in(lang, recovered.as_slice())
        .map(|m| zeroize::Zeroizing::new(m.to_string()))
        .map_err(ToolkitError::Bip39)?;

    writeln!(stdout, "{}", phrase.as_str())
        .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;

    let word_count = entropy_bytes_to_word_count(first_len);
    if let Some(path) = &args.json_out {
        write_combine_json(
            path,
            args.language.human_name(),
            word_count,
            args.share.len(),
            phrase.as_str(),
            stderr,
        )?;
    }

    // SPEC §2.6 row 3 — emit class advisory unconditionally (TTY gate dropped, Cycle B P1).
    // Addendum with the bespoke safety clause follows the unified line.
    crate::secret_advisory::emit_output_class_advisory(
        crate::secret_advisory::OutputClass::PrivateKeyMaterial,
        stderr,
    );
    let _ = writeln!(
        stderr,
        "warning: combined phrase is secret material — Seed XOR has no authentication tag; verify the recovered wallet's expected derived address before trusting; if a share was substituted with a wrong-but-valid one, the result will validate but derive the wrong wallet",
    );

    Ok(0)
}

fn entropy_bytes_to_word_count(len: usize) -> usize {
    match len {
        16 => 12,
        20 => 15,
        24 => 18,
        28 => 21,
        32 => 24,
        _ => 0,
    }
}

fn map_seed_xor_error(e: SeedXorError) -> ToolkitError {
    match e {
        SeedXorError::BadEntropyLength { got, expected_one_of } => ToolkitError::BadInput(format!(
            "seed-xor: entropy length {got} bytes invalid; expected one of {expected_one_of:?}",
        )),
        SeedXorError::TooFewShares { got, min } => ToolkitError::BadInput(format!(
            "seed-xor split: --shares must be >= {min}; got {got}",
        )),
        SeedXorError::MismatchedShareLengths { lengths } => ToolkitError::BadInput(format!(
            "seed-xor combine: all shares must be the same length; got mix of {lengths:?} bytes",
        )),
    }
}

#[derive(serde::Serialize)]
struct SplitJson<'a> {
    schema_version: &'static str,
    operation: &'static str,
    language: &'static str,
    word_count: usize,
    share_count: usize,
    deterministic: bool,
    shares: Vec<&'a str>,
}

#[derive(serde::Serialize)]
struct CombineJson<'a> {
    schema_version: &'static str,
    operation: &'static str,
    language: &'static str,
    word_count: usize,
    share_count: usize,
    phrase: &'a str,
}

fn write_split_json<E: Write>(
    path: &std::path::Path,
    language: &'static str,
    word_count: usize,
    deterministic: bool,
    share_phrases: &[zeroize::Zeroizing<String>],
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let shares: Vec<&str> = share_phrases.iter().map(|s| s.as_str()).collect();
    let envelope = SplitJson {
        schema_version: "1",
        operation: "split",
        language,
        word_count,
        share_count: shares.len(),
        deterministic,
        shares,
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("--json-out serialize: {e}")))?;
    std::fs::write(path, &body)
        .map_err(|e| ToolkitError::BadInput(format!("--json-out write {}: {e}", path.display())))?;

    warn_if_world_readable(path, stderr);
    Ok(())
}

fn write_combine_json<E: Write>(
    path: &std::path::Path,
    language: &'static str,
    word_count: usize,
    share_count: usize,
    phrase: &str,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let envelope = CombineJson {
        schema_version: "1",
        operation: "combine",
        language,
        word_count,
        share_count,
        phrase,
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("--json-out serialize: {e}")))?;
    std::fs::write(path, &body)
        .map_err(|e| ToolkitError::BadInput(format!("--json-out write {}: {e}", path.display())))?;

    warn_if_world_readable(path, stderr);
    Ok(())
}

