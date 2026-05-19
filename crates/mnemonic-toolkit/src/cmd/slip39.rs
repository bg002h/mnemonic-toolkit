//! `mnemonic slip39` subcommand — SLIP-39 K-of-N Shamir backup splitter.
//!
//! Realizes `design/SPEC_slip39_v0_13_0.md` §2.2 (CLI grammar), §2.3
//! (JSON envelope schema), §2.5 (24 refusal classes — row 24 added per
//! P2.2 GREEN Q3 fold), §2.6 (5 advisory classes — row 6 NEW env-var
//! always-on insecurity per Q2 fold), §4 acceptance gates G3, G4, G5,
//! G6, G9. Mirrors `cmd/seed_xor.rs` structurally.
//!
//! Two sub-subcommands:
//!   - `split`: master secret (BIP-39 phrase or hex entropy) → N SLIP-39
//!     shares organized in 1..=16 groups.
//!   - `combine`: ≥K SLIP-39 shares → master secret (hex entropy or
//!     BIP-39 phrase), per the share-set's recorded group/member thresholds.
//!
//! Cycle A/B discipline rails (Q5 fold extracts the world-readable
//! advisory to `crate::secret_advisory::warn_if_world_readable`):
//!   - 5 argv-leakage advisory call sites (split: `--from phrase=`,
//!     `--from entropy=`, `--passphrase`; combine: `--share`,
//!     `--passphrase`)
//!   - `Zeroizing<String>` wraps on parsed `--from`, `--share`,
//!     `--passphrase`
//!   - `mlock::pin_pages_for` Site 1 pins on parsed-input heap buffers
//!     + O(N) per-rendered-share pins inside the stdout-emit loop (per
//!       SPEC §2.1 patch — Q6 fold: `Vec<Zeroizing<String>>` cannot be
//!       pinned in O(1) because the top-level Vec holds non-secret
//!       `String` headers and each share's UTF-8 bytes live in a
//!       separate heap allocation)
//!   - K-of-N stdout-on-TTY parameterized advisory (extends v0.12.0
//!     seed-xor TTY advisory shape)
//!   - shared `secret_advisory::warn_if_world_readable` for `--json-out`
//!     world-readable-path advisory (extracted from
//!     `cmd/seed_xor.rs::emit_world_readable_advisory` per R0 Q5 fold)
//!   - G9 iteration-exponent threshold advisory (E >= 5)
//!   - env-var determinism wedge (Q2 fold): `MNEMONIC_SLIP39_TEST_RNG`
//!     (32-byte hex seed for `ChaCha20Rng`) + `MNEMONIC_SLIP39_TEST_IDENTIFIER`
//!     (decimal u16 in 0..=32767). Always-on insecurity advisory fires
//!     when either env-var is set; production binary path (env-vars
//!     unset) uses `OsRng` + library-generated random identifier.

use crate::cmd::convert::{
    parse_from_input, read_stdin_passphrase, read_stdin_to_string, FromInput, NodeType,
};
use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::secret_advisory::{secret_in_argv_warning, warn_if_world_readable};
use bip39::Mnemonic;
use clap::{Args, Subcommand, ValueEnum};
use mnemonic_toolkit::slip39::{
    parse_slip39_share, render_slip39_share, slip39_combine, slip39_split, GroupSpec, Share,
    Slip39Error,
};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use std::io::{IsTerminal, Read, Write};

const ENV_TEST_RNG: &str = "MNEMONIC_SLIP39_TEST_RNG";
const ENV_TEST_IDENTIFIER: &str = "MNEMONIC_SLIP39_TEST_IDENTIFIER";

/// SPEC §2.4 + plan §3.2 row 12 — the library returns this sentinel
/// value in `Slip39Error::InsufficientShares.group_idx` when the
/// insufficiency is at the GROUP level (not within a single group's
/// members). The CLI handler renders sentinel as the literal `<groups>`
/// token at the stem-formatter boundary.
const GROUP_LEVEL_SENTINEL: u8 = 255;

#[derive(Args, Debug)]
pub struct Slip39Args {
    #[command(subcommand)]
    pub command: Slip39Command,
}

#[derive(Subcommand, Debug)]
pub enum Slip39Command {
    /// Split a master secret into SLIP-39 shares (1..=16 groups × 1..=16 members).
    Split(Slip39SplitArgs),
    /// Combine ≥K SLIP-39 shares back into the master secret.
    Combine(Slip39CombineArgs),
}

#[derive(Args, Debug, Clone)]
pub struct Slip39SplitArgs {
    /// Master secret as `phrase=<value-or->` OR `entropy=<hex-or->`.
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

    /// SLIP-39 passphrase (NOT BIP-39 passphrase).
    ///
    /// Inline value emits an argv-leakage advisory; prefer
    /// `--passphrase-stdin` for sensitive passphrases. The argv-leakage
    /// advisory fires iff this field is `Some(_)` (user supplied the
    /// flag), regardless of value — so empty passphrases
    /// (`--passphrase ""`) still trigger the advisory (R0 C1 fold).
    #[arg(long = "passphrase", conflicts_with = "passphrase_stdin")]
    pub passphrase: Option<String>,

    /// Read passphrase from stdin (single-stdin-per-invocation;
    /// `conflicts_with = "passphrase"` enforced via clap).
    #[arg(long = "passphrase-stdin", default_value_t = false)]
    pub passphrase_stdin: bool,

    /// Groups required to reconstruct (1 <= group-threshold <= group_count).
    #[arg(long = "group-threshold", required = true)]
    pub group_threshold: u8,

    /// Group spec: repeating; `<member_count>,<member_threshold>` per
    /// `--group`. The flag's position in argv is the SLIP-39 `group_idx`
    /// returned in `BadGroupSpec` refusals.
    #[arg(
        long = "group",
        value_name = "N,T",
        required = true,
        action = clap::ArgAction::Append,
        value_parser = parse_group_spec,
    )]
    pub group: Vec<(u8, u8)>,

    /// PBKDF2 cost exponent; 0..=15; iterations = 10000 · 2^E. Trezor's
    /// reference uses E=1 (20000 iterations); E >= 5 emits a performance
    /// advisory.
    #[arg(long = "iteration-exponent", default_value_t = 0)]
    pub iteration_exponent: u8,

    /// BIP-39 language of input phrase; ignored for `entropy=` inputs.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Side-effect: write versioned JSON envelope to PATH (in addition
    /// to plain-stdout shares).
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct Slip39CombineArgs {
    /// SLIP-39 share mnemonic. Repeating; at most ONE may be `-` (stdin).
    ///
    /// Inline values emit a per-occurrence argv-leakage advisory;
    /// prefer `--share -` (stdin) for sensitive shares.
    #[arg(
        long = "share",
        value_name = "<slip39-mnemonic-or->",
        required = true,
        action = clap::ArgAction::Append,
    )]
    pub share: Vec<String>,

    /// SLIP-39 passphrase used at split time. Same shape constraints as
    /// the split flag (Option + conflicts_with).
    #[arg(long = "passphrase", conflicts_with = "passphrase_stdin")]
    pub passphrase: Option<String>,

    /// Read passphrase from stdin (incompatible with any `--share -`
    /// AND with `--passphrase`).
    #[arg(long = "passphrase-stdin", default_value_t = false)]
    pub passphrase_stdin: bool,

    /// Output shape: `entropy` (default; hex on stdout) or `phrase`
    /// (BIP-39 mnemonic).
    #[arg(long = "to", default_value = "entropy")]
    pub to: Slip39ToShape,

    /// BIP-39 language for `--to phrase`; ignored for `--to entropy`.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Side-effect: write versioned JSON envelope to PATH (in addition
    /// to plain-stdout secret).
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

/// `--to` output shape selector. SPEC §2.2 combine flag table.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum Slip39ToShape {
    /// Hex-encoded raw master secret bytes (default).
    Entropy,
    /// BIP-39 mnemonic, language per `--language`.
    Phrase,
}

/// `--group N,T` value parser. `N` is `member_count`, `T` is
/// `member_threshold`. Both 1..=16 per SLIP-0039.
///
/// Range validation (1 <= T <= N <= 16) happens at the library
/// boundary in `slip39_split` and surfaces via the `BadGroupSpec`
/// variant, mapped to SPEC §2.5 rows 4-5 by `map_slip39_error` below.
pub fn parse_group_spec(s: &str) -> Result<(u8, u8), String> {
    let (n, t) = s
        .split_once(',')
        .ok_or_else(|| format!("expected `<member_count>,<member_threshold>`; got `{s}`"))?;
    let n: u8 = n
        .parse()
        .map_err(|e| format!("member_count: {e} (`{n}` is not a valid 1..=16)"))?;
    let t: u8 = t
        .parse()
        .map_err(|e| format!("member_threshold: {e} (`{t}` is not a valid 1..=16)"))?;
    Ok((n, t))
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &Slip39Args,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    match &args.command {
        Slip39Command::Split(a) => {
            // v0.26.0 §3 — resolve `@env:<VAR>` sentinels on `--from` +
            // `--passphrase` before downstream consumption.
            let owned_a;
            let a = if needs_split_env_sentinel_resolution(a) {
                owned_a = resolve_split_env_sentinels(a)?;
                &owned_a
            } else {
                a
            };
            run_split(a, stdin, stdout, stderr)
        }
        Slip39Command::Combine(a) => {
            // v0.26.0 §3 — resolve `@env:<VAR>` sentinels on `--share` +
            // `--passphrase` before downstream consumption.
            let owned_a;
            let a = if needs_combine_env_sentinel_resolution(a) {
                owned_a = resolve_combine_env_sentinels(a)?;
                &owned_a
            } else {
                a
            };
            run_combine(a, stdin, stdout, stderr)
        }
    }
}

fn needs_split_env_sentinel_resolution(args: &Slip39SplitArgs) -> bool {
    let pp = args
        .passphrase
        .as_deref()
        .map(|v| v.starts_with("@env:"))
        .unwrap_or(false);
    // `--from` carries `phrase=` or `entropy=` (both secret-bearing per
    // row 17). Resolve sentinel in the value side.
    let from = args.from.value.starts_with("@env:");
    pp || from
}

fn resolve_split_env_sentinels(args: &Slip39SplitArgs) -> Result<Slip39SplitArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    if let Some(pp) = owned.passphrase.as_ref() {
        owned.passphrase = Some(resolve_env_var_sentinel(pp, "--passphrase")?);
    }
    // Both `phrase=` and `entropy=` are secret-bearing per SPEC §2.5 row 17.
    let flag = format!("--from {}=", owned.from.node.as_str());
    owned.from.value = resolve_env_var_sentinel(&owned.from.value, &flag)?;
    Ok(owned)
}

fn needs_combine_env_sentinel_resolution(args: &Slip39CombineArgs) -> bool {
    let pp = args
        .passphrase
        .as_deref()
        .map(|v| v.starts_with("@env:"))
        .unwrap_or(false);
    let share = args.share.iter().any(|v| v.starts_with("@env:"));
    pp || share
}

fn resolve_combine_env_sentinels(
    args: &Slip39CombineArgs,
) -> Result<Slip39CombineArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    if let Some(pp) = owned.passphrase.as_ref() {
        owned.passphrase = Some(resolve_env_var_sentinel(pp, "--passphrase")?);
    }
    for v in owned.share.iter_mut() {
        *v = resolve_env_var_sentinel(v, "--share")?;
    }
    Ok(owned)
}

fn emit_env_var_advisory<E: Write>(stderr: &mut E) {
    let _ = writeln!(
        stderr,
        "warning: MNEMONIC_SLIP39_TEST_RNG set — output is deterministic and INSECURE; do not use for real shares",
    );
}

fn resolve_passphrase<R: Read>(
    inline: Option<&String>,
    stdin_flag: bool,
    stdin: &mut R,
) -> Result<zeroize::Zeroizing<String>, ToolkitError> {
    if stdin_flag {
        Ok(zeroize::Zeroizing::new(read_stdin_passphrase(stdin)?))
    } else if let Some(p) = inline {
        Ok(zeroize::Zeroizing::new(p.clone()))
    } else {
        Ok(zeroize::Zeroizing::new(String::new()))
    }
}

fn parse_master_to_entropy(
    from: &FromInput,
    language: CliLanguage,
    raw_value: &str,
) -> Result<zeroize::Zeroizing<Vec<u8>>, ToolkitError> {
    match from.node {
        NodeType::Phrase => {
            // SPEC §2.5 row 1 pre-check — word count must be in
            // {12, 15, 18, 21, 24} before bip39 parse.
            let word_count = raw_value.split_whitespace().count();
            if !matches!(word_count, 12 | 15 | 18 | 21 | 24) {
                return Err(ToolkitError::BadInput(format!(
                    "slip39 split: input phrase must be 12/15/18/21/24 words; got {word_count}",
                )));
            }
            let lang: bip39::Language = language.into();
            let m = Mnemonic::parse_in(lang, raw_value).map_err(ToolkitError::Bip39)?;
            Ok(zeroize::Zeroizing::new(m.to_entropy()))
        }
        NodeType::Entropy => {
            // SPEC §2.5 row 2 — both non-hex decode failures AND
            // valid-hex-wrong-byte-length cases share the row 2 stem.
            // For non-hex we report the assumed byte count as
            // chars/2 (rounding down) to follow `error.rs` doc:
            // "mapped to the byte count the CLI would report."
            let bytes = match hex::decode(raw_value) {
                Ok(b) => b,
                Err(_) => {
                    let assumed = raw_value.len() / 2;
                    return Err(ToolkitError::BadInput(format!(
                        "slip39 split: entropy hex must decode to 16/20/24/28/32 bytes; got {assumed} bytes",
                    )));
                }
            };
            if !matches!(bytes.len(), 16 | 20 | 24 | 28 | 32) {
                return Err(ToolkitError::BadInput(format!(
                    "slip39 split: entropy hex must decode to 16/20/24/28/32 bytes; got {} bytes",
                    bytes.len(),
                )));
            }
            Ok(zeroize::Zeroizing::new(bytes))
        }
        _ => unreachable!("row 17 pre-check enforces phrase/entropy node"),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_split<R: Read, W: Write, E: Write>(
    args: &Slip39SplitArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // SPEC §2.5 row 17 — --from variant must be phrase= or entropy=.
    if args.from.node != NodeType::Phrase && args.from.node != NodeType::Entropy {
        return Err(ToolkitError::BadInput(format!(
            "slip39 split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got {}=",
            args.from.node.as_str(),
        )));
    }

    // SPEC §2.5 row 18 — single stdin consumer per invocation.
    // For split: --from - + --passphrase-stdin = max 2 candidates.
    let split_stdin_count = (args.from.value == "-") as usize
        + (args.passphrase_stdin) as usize;
    if split_stdin_count > 1 {
        return Err(ToolkitError::BadInput(
            "slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)".into(),
        ));
    }

    // SPEC §2.5 row 5 — CLI-layer pre-check for `--group 1,1`. The
    // library accepts (member_count=1, member_threshold=1) (no
    // structural violation; both fields ≥ 1 and T ≤ N), but the
    // toolkit policy refuses it as "no recovery benefit." This check
    // is CLI-only per plan §3.5 step 1.
    for (g_idx, (n, t)) in args.group.iter().enumerate() {
        if *n == 1 && *t == 1 {
            return Err(ToolkitError::BadInput(format!(
                "slip39 split: 1-of-1 group offers no recovery benefit; use --group N,T with N >= 2 (toolkit policy); got group {g_idx}=1,1"
            )));
        }
    }

    // SPEC §2.6 rows 1a/1b — argv-leakage advisories for inline --from.
    if args.from.value != "-" {
        match args.from.node {
            NodeType::Phrase => {
                secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-");
            }
            NodeType::Entropy => {
                secret_in_argv_warning(stderr, "--from entropy=", "--from entropy=-");
            }
            _ => unreachable!(),
        }
    }
    // SPEC §2.6 row 1c — argv-leakage advisory for inline --passphrase.
    // Fires on Option::is_some (R0 C1 fold: user-supplied vs. default
    // distinction is structural, regardless of value).
    if args.passphrase.is_some() {
        secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin");
    }

    // Env-var determinism wedge (SPEC §2.6 row 6 — always-on insecurity
    // advisory; SPEC §6 documents the two env-vars).
    let test_rng_hex = std::env::var(ENV_TEST_RNG).ok();
    let test_identifier_raw = std::env::var(ENV_TEST_IDENTIFIER).ok();
    if test_rng_hex.is_some() || test_identifier_raw.is_some() {
        emit_env_var_advisory(stderr);
    }

    // Resolve --from value (Zeroizing + mlock pin).
    let from_value: zeroize::Zeroizing<String> = if args.from.value == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(args.from.value.clone())
    };
    let _pin_from = mnemonic_toolkit::mlock::pin_pages_for(from_value.as_bytes());

    // Resolve --passphrase (Zeroizing + mlock pin).
    let passphrase = resolve_passphrase(args.passphrase.as_ref(), args.passphrase_stdin, stdin)?;
    let _pin_pp = mnemonic_toolkit::mlock::pin_pages_for(passphrase.as_bytes());

    // Parse master secret to entropy bytes (rows 1, 2).
    let master_entropy =
        parse_master_to_entropy(&args.from, args.language, from_value.as_str())?;
    let _pin_master = mnemonic_toolkit::mlock::pin_pages_for(master_entropy.as_slice());

    // Build GroupSpec vec from --group args (R0 I4 fold:
    // order-preserving so group_idx matches argv position).
    let groups: Vec<GroupSpec> = args
        .group
        .iter()
        .map(|(n, t)| GroupSpec {
            member_count: *n,
            member_threshold: *t,
        })
        .collect();

    // Identifier override (env-var wedge).
    let identifier = if let Some(s) = test_identifier_raw {
        let id: u16 = s.parse().map_err(|_| {
            ToolkitError::BadInput(format!(
                "MNEMONIC_SLIP39_TEST_IDENTIFIER: must be a u16 decimal; got {s:?}",
            ))
        })?;
        if id > 32767 {
            return Err(ToolkitError::BadInput(format!(
                "MNEMONIC_SLIP39_TEST_IDENTIFIER: must be 0..=32767 (15-bit); got {id}",
            )));
        }
        Some(id)
    } else {
        None
    };

    // Call slip39_split with either ChaCha20Rng (test wedge) or OsRng
    // (production). Map Slip39Error via map_slip39_error.
    let share_groups: Vec<Vec<Share>> = if let Some(hex_str) = test_rng_hex {
        let seed_bytes = hex::decode(&hex_str).map_err(|_| {
            ToolkitError::BadInput(format!(
                "MNEMONIC_SLIP39_TEST_RNG: must be 32-byte hex (64 chars); got non-hex {hex_str:?}",
            ))
        })?;
        if seed_bytes.len() != 32 {
            return Err(ToolkitError::BadInput(format!(
                "MNEMONIC_SLIP39_TEST_RNG: must decode to exactly 32 bytes; got {} bytes",
                seed_bytes.len(),
            )));
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&seed_bytes);
        let mut rng = ChaCha20Rng::from_seed(seed);
        slip39_split(
            master_entropy.as_slice(),
            passphrase.as_bytes(),
            args.group_threshold,
            &groups,
            args.iteration_exponent,
            false, // extendable hardcoded false per P1c-E.1 R0 Q1
            identifier,
            &mut rng,
        )
    } else {
        let mut rng = rand_core::OsRng;
        slip39_split(
            master_entropy.as_slice(),
            passphrase.as_bytes(),
            args.group_threshold,
            &groups,
            args.iteration_exponent,
            false,
            identifier,
            &mut rng,
        )
    }
    .map_err(map_slip39_error)?;

    // Render shares to strings (one Zeroizing<String> per share).
    let rendered: Vec<Vec<zeroize::Zeroizing<String>>> = share_groups
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|s| zeroize::Zeroizing::new(render_slip39_share(s)))
                .collect::<Vec<_>>()
        })
        .collect();

    // Emit shares to stdout: one per line, blank-line separator
    // between groups, trailing newline. Per-rendered-share mlock pin
    // inside the loop (Q6 fold — O(N) pinning).
    for (g_idx, g) in rendered.iter().enumerate() {
        if g_idx > 0 {
            writeln!(stdout)
                .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;
        }
        for s in g {
            let _pin = mnemonic_toolkit::mlock::pin_pages_for(s.as_bytes());
            writeln!(stdout, "{}", s.as_str())
                .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;
        }
    }

    // SPEC §2.6 row 2 — K-of-N stdout-on-TTY parameterized advisory.
    if std::io::stdout().is_terminal() {
        let total_shares: usize = rendered.iter().map(|g| g.len()).sum();
        let _ = writeln!(
            stderr,
            "warning: SLIP-39 shares on stdout — N={total_shares} shares emitted across {g_count} groups (group-threshold {gt}); each share is independently secret material; distribute per your group/member-threshold policy; do not paste this output into a single untrusted tool",
            g_count = rendered.len(),
            gt = args.group_threshold,
        );
    }

    // SPEC §2.6 row 5 — G9 iteration-exponent threshold advisory.
    if args.iteration_exponent >= 5 {
        let iters = 10000u32 * (1u32 << args.iteration_exponent);
        let _ = writeln!(
            stderr,
            "warning: --iteration-exponent E={e} yields {iters} × PBKDF2-HMAC-SHA-256 iterations; split + combine performance may be observably slow (sub-second to multi-second). Trezor's reference uses E=1 (20000 iters) as default; the SLIP-0039 spec gives no recommended values. E >= 10 may exceed 30s on weak hardware.",
            e = args.iteration_exponent,
        );
    }

    // --json-out side-effect (SPEC §2.3 split schema).
    if let Some(path) = &args.json_out {
        write_split_json(path, args, &share_groups, &rendered, stderr)?;
    }

    Ok(0)
}

#[allow(clippy::too_many_arguments)]
fn run_combine<R: Read, W: Write, E: Write>(
    args: &Slip39CombineArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // SPEC §2.5 row 18 — single stdin consumer per invocation.
    let combine_stdin_count = args.share.iter().filter(|s| *s == "-").count()
        + (args.passphrase_stdin) as usize;
    if combine_stdin_count > 1 {
        return Err(ToolkitError::BadInput(
            "slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)".into(),
        ));
    }

    // SPEC §2.6 rows 1d (per-share inline) + 1e (passphrase inline).
    for sh in &args.share {
        if sh != "-" {
            secret_in_argv_warning(stderr, "--share", "--share -");
        }
    }
    if args.passphrase.is_some() {
        secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin");
    }

    // Resolve --share values (stdin or inline) into Zeroizing<String>.
    let mut share_strings: Vec<zeroize::Zeroizing<String>> = Vec::with_capacity(args.share.len());
    let mut stdin_consumed = false;
    for sh in &args.share {
        let s = if sh == "-" {
            if stdin_consumed {
                return Err(ToolkitError::BadInput(
                    "slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)".into(),
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

    // SPEC §2.5 row 19 — empty share list (stdin read returned blank).
    let non_empty_count = share_strings
        .iter()
        .filter(|s| !s.trim().is_empty())
        .count();
    if non_empty_count == 0 {
        return Err(ToolkitError::BadInput(
            "slip39 combine: at least one share required".into(),
        ));
    }

    // Resolve --passphrase (passphrase-stdin uses read_stdin_passphrase
    // per R0 Note 2 — preserves trailing whitespace + NULL, distinct
    // from read_stdin_to_string which `.trim()`s).
    let passphrase = resolve_passphrase(args.passphrase.as_ref(), args.passphrase_stdin, stdin)?;
    let _pin_pp = mnemonic_toolkit::mlock::pin_pages_for(passphrase.as_bytes());

    // Parse each share via parse_slip39_share. Re-tag the lib's
    // share_idx=0 with the actual input position so the row 9/10/16/23
    // stems byte-faithfully name the position.
    let mut shares: Vec<Share> = Vec::with_capacity(non_empty_count);
    for (idx, s_str) in share_strings.iter().enumerate() {
        if s_str.trim().is_empty() {
            continue;
        }
        let parsed = parse_slip39_share(s_str.as_str()).map_err(|e| {
            map_slip39_error(reindex_share_idx(e, idx))
        })?;
        shares.push(parsed);
    }

    // Library combine → master entropy bytes.
    let master_entropy =
        slip39_combine(&shares, passphrase.as_bytes()).map_err(map_slip39_error)?;
    let _pin_master = mnemonic_toolkit::mlock::pin_pages_for(master_entropy.as_slice());

    // Render output per --to.
    let output: zeroize::Zeroizing<String> = match args.to {
        Slip39ToShape::Entropy => {
            zeroize::Zeroizing::new(hex::encode(master_entropy.as_slice()))
        }
        Slip39ToShape::Phrase => {
            let lang: bip39::Language = args.language.into();
            let m = Mnemonic::from_entropy_in(lang, master_entropy.as_slice())
                .map_err(ToolkitError::Bip39)?;
            zeroize::Zeroizing::new(m.to_string())
        }
    };
    let _pin_output = mnemonic_toolkit::mlock::pin_pages_for(output.as_bytes());

    writeln!(stdout, "{}", output.as_str())
        .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;

    // SPEC §2.6 row 3 — combine reconstructed-secret stdout-on-TTY.
    if std::io::stdout().is_terminal() {
        let _ = writeln!(
            stderr,
            "warning: reconstructed secret material on stdout — verify the recovered wallet's expected derived address before trusting",
        );
    }

    // --json-out side-effect (SPEC §2.3 combine schema).
    if let Some(path) = &args.json_out {
        let identifier = shares[0].identifier;
        let iter_exp = shares[0].iteration_exponent;
        write_combine_json(path, identifier, iter_exp, args.to, output.as_str(), stderr)?;
    }

    Ok(0)
}

/// Re-tag a per-share `Slip39Error` variant's `share_idx=0` (the
/// `parse_slip39_share` default) with the actual input position.
fn reindex_share_idx(e: Slip39Error, idx: usize) -> Slip39Error {
    match e {
        Slip39Error::UnknownWord { word_idx, .. } => Slip39Error::UnknownWord {
            share_idx: idx,
            word_idx,
        },
        Slip39Error::InvalidChecksum { .. } => Slip39Error::InvalidChecksum { share_idx: idx },
        Slip39Error::InvalidPadding { .. } => Slip39Error::InvalidPadding { share_idx: idx },
        Slip39Error::GroupThresholdExceedsCount {
            threshold, count, ..
        } => Slip39Error::GroupThresholdExceedsCount {
            share_idx: idx,
            threshold,
            count,
        },
        Slip39Error::InvalidShareValueLength { got, .. } => Slip39Error::InvalidShareValueLength {
            share_idx: idx,
            got,
        },
        other => other,
    }
}

/// Map a [`Slip39Error`] to the SPEC §2.5 stem byte-faithfully per
/// plan §3.2 `format!` template table. Row 24 `MemberThresholdMismatch`
/// added in this commit (Q3 fold).
fn map_slip39_error(e: Slip39Error) -> ToolkitError {
    use Slip39Error::*;
    let msg = match e {
        BadPhraseWordCount(got) => format!(
            "slip39 split: input phrase must be 12/15/18/21/24 words; got {got}"
        ),
        BadEntropyByteLength(got) => format!(
            "slip39 split: entropy hex must decode to 16/20/24/28/32 bytes; got {got} bytes"
        ),
        BadGroupThreshold { got, group_count } => format!(
            "slip39 split: --group-threshold must be in 1..={group_count} (number of --group flags); got {got}"
        ),
        BadGroupSpec {
            group_idx,
            n: 1,
            t: 1,
        } => format!(
            "slip39 split: 1-of-1 group offers no recovery benefit; use --group N,T with N >= 2 (toolkit policy); got group {group_idx}=1,1"
        ),
        BadGroupSpec { group_idx, n, t } => format!(
            "slip39 split: --group N,T requires 1 <= T <= N <= 16; got group {group_idx}={n},{t}"
        ),
        BadIterationExponent(got) => format!(
            "slip39 split: --iteration-exponent must be 0..=15 (4-bit field); got {got}"
        ),
        IdentifierMismatch => "slip39 combine: shares disagree on identifier; shares must come from the same secret".to_string(),
        IterationExponentMismatch => "slip39 combine: shares disagree on iteration-exponent".to_string(),
        InvalidChecksum { share_idx } => format!(
            "slip39 combine: share at position {share_idx} has invalid SLIP-39 checksum (RS1024)"
        ),
        UnknownWord { share_idx, word_idx } => format!(
            "slip39 combine: share at position {share_idx}: word at index {word_idx} not in SLIP-39 wordlist"
        ),
        DigestVerificationFailed => "slip39 combine: reconstructed master digest mismatch — wrong --passphrase OR a share was substituted".to_string(),
        InsufficientShares {
            group_idx,
            needed,
            got,
        } => {
            if group_idx == GROUP_LEVEL_SENTINEL {
                format!("slip39 combine: insufficient shares for group <groups>: need {needed}, got {got}")
            } else {
                format!("slip39 combine: insufficient shares for group {group_idx}: need {needed}, got {got}")
            }
        }
        GroupThresholdMismatch => "slip39 combine: shares disagree on group_threshold".to_string(),
        GroupCountMismatch => "slip39 combine: shares disagree on group_count".to_string(),
        DuplicateMemberIndex {
            group_idx,
            member_idx,
        } => format!(
            "slip39 combine: duplicate member index {member_idx} in group {group_idx}"
        ),
        InvalidPadding { share_idx } => format!(
            "slip39 combine: share at position {share_idx} has non-zero padding bits (encoding violation)"
        ),
        EmptyShares => "slip39 combine: at least one share required".to_string(),
        InvalidShareValueLength { share_idx, got } => format!(
            "slip39 combine: share at position {share_idx} has value length {got} (must be 16/20/24/28/32 bytes)"
        ),
        ShareValueLengthMismatch => "slip39 combine: shares disagree on value length".to_string(),
        ExtendableMismatch => "slip39 combine: shares disagree on the extendable bit".to_string(),
        GroupThresholdExceedsCount {
            share_idx,
            threshold,
            count,
        } => format!(
            "slip39 combine: share at position {share_idx}: group_threshold {threshold} exceeds group_count {count}"
        ),
        MemberThresholdMismatch => "slip39 combine: shares within a group disagree on member_threshold".to_string(),
    };
    ToolkitError::BadInput(msg)
}

// ============================================================
// JSON envelope structs (SPEC §2.3; field order is part of the schema
// per SPEC §4 G4 SHA-pin + R0 N4 fold).
// ============================================================

#[derive(serde::Serialize)]
struct SplitJson<'a> {
    schema_version: &'static str,
    operation: &'static str,
    identifier: u16,
    iteration_exponent: u8,
    group_threshold: u8,
    groups: Vec<SplitGroupEntry<'a>>,
}

#[derive(serde::Serialize)]
struct SplitGroupEntry<'a> {
    member_count: u8,
    member_threshold: u8,
    // `shares` is intentionally LAST per R0 N4 fold (mirrors
    // `seed_xor.rs::SplitJson::shares` field-order precedent).
    shares: Vec<&'a str>,
}

#[derive(serde::Serialize)]
struct CombineJson<'a> {
    schema_version: &'static str,
    operation: &'static str,
    identifier: u16,
    iteration_exponent: u8,
    output_shape: &'static str,
    phrase: Option<&'a str>,
    entropy_hex: Option<&'a str>,
}

fn write_split_json<E: Write>(
    path: &std::path::Path,
    args: &Slip39SplitArgs,
    share_groups: &[Vec<Share>],
    rendered: &[Vec<zeroize::Zeroizing<String>>],
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // Identifier is consistent across all shares in a single split.
    let identifier = share_groups[0][0].identifier;
    let groups: Vec<SplitGroupEntry> = args
        .group
        .iter()
        .zip(rendered.iter())
        .map(|((n, t), r_g)| SplitGroupEntry {
            member_count: *n,
            member_threshold: *t,
            shares: r_g.iter().map(|s| s.as_str()).collect(),
        })
        .collect();
    let envelope = SplitJson {
        schema_version: "1",
        operation: "split",
        identifier,
        iteration_exponent: args.iteration_exponent,
        group_threshold: args.group_threshold,
        groups,
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("--json-out serialize: {e}")))?;
    std::fs::write(path, &body).map_err(|e| {
        ToolkitError::BadInput(format!("--json-out write {}: {e}", path.display()))
    })?;
    warn_if_world_readable(path, stderr);
    Ok(())
}

fn write_combine_json<E: Write>(
    path: &std::path::Path,
    identifier: u16,
    iteration_exponent: u8,
    to: Slip39ToShape,
    output: &str,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let (output_shape, phrase, entropy_hex) = match to {
        Slip39ToShape::Phrase => ("phrase", Some(output), None),
        Slip39ToShape::Entropy => ("entropy", None, Some(output)),
    };
    let envelope = CombineJson {
        schema_version: "1",
        operation: "combine",
        identifier,
        iteration_exponent,
        output_shape,
        phrase,
        entropy_hex,
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("--json-out serialize: {e}")))?;
    std::fs::write(path, &body).map_err(|e| {
        ToolkitError::BadInput(format!("--json-out write {}: {e}", path.display()))
    })?;
    warn_if_world_readable(path, stderr);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_group_spec_accepts_canonical_shape() {
        assert_eq!(parse_group_spec("3,2").unwrap(), (3, 2));
        assert_eq!(parse_group_spec("16,16").unwrap(), (16, 16));
        assert_eq!(parse_group_spec("1,1").unwrap(), (1, 1));
    }

    #[test]
    fn parse_group_spec_rejects_missing_comma() {
        let err = parse_group_spec("32").unwrap_err();
        assert!(err.contains("member_count"), "got: {err}");
    }

    #[test]
    fn parse_group_spec_rejects_non_numeric() {
        assert!(parse_group_spec("a,2").is_err());
        assert!(parse_group_spec("2,b").is_err());
    }
}
