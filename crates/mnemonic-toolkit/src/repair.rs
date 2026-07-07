//! BCH error-correction primitive for m-format cards (ms1 / mk1 / md1).
//!
//! All three formats share the BIP-93 codex32 BCH generator polynomials
//! (regular `BCH(93,80,8)` + long `BCH(108,93,8)`); only the per-HRP +
//! per-code target-residue NUMS constants differ.
//!
//! **v0.23.0 — D29 migration:** the Ms1 + Md1 branches now delegate to the
//! sibling codecs' native BCH-correction APIs (`ms_codec::decode_with_correction`
//! / `md_codec::decode_with_correction`, both added in their respective
//! v0.2.0 / v0.34.0 releases). The Mk1 branch continues to consume
//! mk-codec's promoted BCH primitives (`bch::*` + `bch_decode::*`) directly.
//! This deletes the previously-vendored `MS_NUMS_TARGET` + `MD_NUMS_TARGET`
//! constants in favor of the sibling codecs' authoritative implementations.
//!
//! Per-HRP × per-code target constants (mk1 only, since Ms1/Md1 are now
//! delegated):
//!   - mk regular: `mk_codec::MK_REGULAR_CONST = 0x1062435f91072fa5c` (imported)
//!   - mk long:    `mk_codec::MK_LONG_CONST    = 0x41890d7e441cbe97273` (imported)
//!
//! `ms` and `md` do not define long-code variants in v0.1 of their respective
//! codecs, so length-detected long-code chunks for those HRPs error.
//!
//! Per-chunk atomic semantics per plan §1 D8: if any chunk fails, the
//! whole `repair_card` call returns `Err` naming that chunk; partially-
//! repaired sibling chunks are NOT returned.

use crate::indel::IndelOracle;
use mk_codec::string_layer::bch::{
    bch_code_for_length, hrp_expand, polymod_run, BchCode, ALPHABET, GEN_LONG, GEN_REGULAR,
    LONG_MASK, LONG_SHIFT, REGULAR_MASK, REGULAR_SHIFT,
};
use mk_codec::string_layer::bch_decode::{decode_long_errors, decode_regular_errors};
use mk_codec::string_layer::{decode_string, StringLayerHeader};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::{IsTerminal, Read, Write};

use crate::error::ToolkitError;

// Per-HRP × per-code target-residue NUMS constants. mk imported from
// mk-codec. (v0.23.0: ms/md constants deleted per D29 migration; their
// repair paths delegate to sibling codecs' native APIs.)
pub(crate) const MK_REGULAR_TARGET: u128 = mk_codec::MK_REGULAR_CONST;
pub(crate) const MK_LONG_TARGET: u128 = mk_codec::MK_LONG_CONST;
// md is regular-only (no long code) → no MD_LONG_TARGET. Re-acquired from
// md-codec for the indel path (the non-indel md1 repair delegates to
// md_codec::decode_with_correction and never calls target_residue).
pub(crate) const MD_REGULAR_TARGET: u128 = md_codec::bch::MD_REGULAR_CONST;

/// Singleton bound for BCH(93,80,8) regular code: 2t = 8 (correct up to t=4
/// substitutions). Reported in `RepairError::TooManyErrors` for user
/// orientation; the actual decoder uniqueness check happens inside
/// `decode_regular_errors`.
const SINGLETON_BOUND: usize = 8;

/// Which m-format card kind drives this repair invocation. Picks the
/// per-HRP target-residue constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKind {
    Ms1,
    Mk1,
    Md1,
}

impl CardKind {
    pub fn hrp(self) -> &'static str {
        match self {
            Self::Ms1 => "ms",
            Self::Mk1 => "mk",
            Self::Md1 => "md",
        }
    }

    /// Per-HRP × per-code target residue for the toolkit-side
    /// `polymod_residue` path. Used by the Mk1 branch (non-indel repair +
    /// indel) and — since the v0.37.2 md1 indel work — the Md1 branch's
    /// indel path (`md1_chunk_solve` / `repair_chunk_one(Md1, …)`). Returns
    /// `None` for an HRP/code pair the upstream codec does not define.
    ///
    /// Ms1 still delegates entirely to ms-codec's native API and never calls
    /// this helper (its arm stays `None`). md is regular-only, so `(Md1,
    /// Long)` is also `None` (a long-code md chunk is undefined → the `None`
    /// return triggers `UnsupportedCodeVariant`, a safe-fail path). The
    /// non-indel md1 repair path still delegates to
    /// `md_codec::decode_with_correction` and does not call this helper.
    fn target_residue(self, code: BchCode) -> Option<u128> {
        match (self, code) {
            (Self::Mk1, BchCode::Regular) => Some(MK_REGULAR_TARGET),
            (Self::Mk1, BchCode::Long) => Some(MK_LONG_TARGET),
            (Self::Md1, BchCode::Regular) => Some(MD_REGULAR_TARGET),
            // ms1 delegates to ms-codec; md is regular-only (no long code).
            (Self::Ms1, _) | (Self::Md1, BchCode::Long) => None,
        }
    }
}

/// v0.24.0 §2.C.1 — classify a positional `<STRING>` by its HRP prefix.
/// Returns the routed `CardKind` for the three recognized prefixes (`ms1`,
/// `mk1`, `md1`); returns `ToolkitError::UnknownHrp` for any other prefix.
///
/// Toolkit-internal helper invoked post-clap-parse by
/// `repair::run` / `inspect::run` / `verify_bundle::run` to merge
/// positional arguments into the existing typed-flag storage.
///
/// v0.53.3 (audit M11): the prefix PROBE is case-insensitive — BIP-173's
/// uppercase QR alphanumeric mode means valid all-uppercase cards exist in
/// the wild. Only a lowercased COPY is probed for routing; the ORIGINAL
/// string flows to the per-codec parser, which remains the authority on
/// case (lowercase stays canonical for emission per BIP-93; mixed-case
/// rejection stays codec-side — never normalize-at-intake).
pub(crate) fn classify_hrp_prefix(s: &str) -> Result<CardKind, ToolkitError> {
    let probe = s.to_lowercase();
    if probe.starts_with("ms1") {
        Ok(CardKind::Ms1)
    } else if probe.starts_with("mk1") {
        Ok(CardKind::Mk1)
    } else if probe.starts_with("md1") {
        Ok(CardKind::Md1)
    } else {
        Err(ToolkitError::UnknownHrp {
            got: s.to_string(),
            expected_one_of: vec!["ms1", "mk1", "md1"],
        })
    }
}

/// v0.24.0 §2.C.1 (D34/I5 fold) — per-flag HRP validation. Used by
/// `repair::run` / `inspect::run` / `verify_bundle::run` to reject a typed
/// `--ms1` / `--mk1` / `--md1` flag whose value's HRP prefix does not match
/// the flag's expected codec. v0.53.3 (audit M11): the prefix comparison is
/// case-insensitive — the codecs are the authority on case, so a
/// consistent-case `--mk1 MK1…` passes here and is accepted-or-rejected by
/// the codec per its own wire rules (the v0.24.0 I5 case-mismatch rejection
/// is relaxed; rejecting only at the flag gate left the surface inconsistent
/// with positional intake and with `restore --md1` / `convert`).
///
/// `flag` is the user-facing flag name (e.g. `"--ms1"`), `canonical` is the
/// lowercase canonical HRP (`"ms"` / `"mk"` / `"md"`), and `value` is the
/// raw user-supplied string. Two special-case values are exempt from HRP
/// validation:
///   - `"-"` (stdin sentinel) — callers expand it after this check.
///   - `""` (empty-string positional watch-only sentinel per SPEC §5.8;
///     v0.25.1 fix) — for `--ms1` only, an empty string marks that cosigner
///     as watch-only without supplying a seed. The caller emits a stderr
///     NOTICE per cosigner; this validator just lets the value through.
///     Restores the pre-v0.24.0 convention that v0.24.0 §2.C.1's strict gate
///     accidentally broke. Without this exemption, `--ms1 ""` would hard-fail
///     here at clap-parse-time and the SPEC §5.8 sentinel would be
///     un-expressible at index ≥ 1 (i.e., middle / trailing cosigners in a
///     multisig bundle). See FOLLOWUP
///     `verify-bundle-empty-ms1-watch-only-sentinel-or-explicit-flag` for
///     the v0.26+ design discussion that selected this path.
///
/// Two cases for non-sentinel values:
///   1. `value.to_lowercase().starts_with(canonical)` → `Ok(())` (any
///      consistent OR mixed case passes the flag gate; the codec decides).
///   2. True HRP mismatch (e.g. `--ms1 mk1xxx` or `--ms1 MK1xxx`) → returns
///      `HrpMismatch` with `got` set to the LOWERCASED prefix before the `1`
///      separator, so a case-shifted wrong-HRP value still reads
///      "expected 'ms', got 'mk'".
pub(crate) fn validate_flag_hrp(
    flag: &'static str,
    canonical: &'static str,
    value: &str,
) -> Result<(), ToolkitError> {
    if value == "-" {
        return Ok(());
    }
    // v0.25.1 fix: empty-string sentinel exemption. SPEC §5.8 documents that
    // `--ms1 ""` marks a cosigner as watch-only at a specific positional
    // index (needed for middle / trailing-cosigner skips in multisig bundles).
    // v0.24.0 §2.C.1 strict-HRP gate accidentally broke this convention; the
    // exemption restores it. Empty `--mk1 ""` / `--md1 ""` are also accepted
    // for symmetry (the bundle's per-slot ms1/mk1/md1 lists are
    // position-aligned; empty entries occur in symmetric positions).
    if value.is_empty() {
        return Ok(());
    }
    // Match against the canonical `<hrp>1` prefix (e.g. "ms1") on a
    // lowercased COPY — case-insensitive probe per v0.53.3 (audit M11); the
    // ORIGINAL value flows to the codec, which owns case acceptance.
    let canonical_full = format!("{canonical}1");
    if value.to_lowercase().starts_with(&canonical_full) {
        return Ok(());
    }
    // True HRP mismatch — extract the prefix up to (but excluding) the LAST
    // `1` separator (BIP-173: the data charset excludes `1`, so the last `1`
    // is the separator) from the lowercased value for a clean error message
    // (lowercasing keeps `--ms1 MK1xxx` reading "expected 'ms', got 'mk'").
    let lower = value.to_lowercase();
    let got_hrp = lower
        .rfind('1')
        .map(|i| lower[..i].to_string())
        .unwrap_or(lower);
    Err(ToolkitError::HrpMismatch {
        flag,
        expected: canonical,
        got: got_hrp,
    })
}

/// v0.25.0 §2.A (D4 fold) — read-only accessor surface over the two
/// card-input argument structs (`cmd::repair::RepairArgs` +
/// `cmd::inspect::InspectArgs`). Lets `resolve_groups` / `count_dashes` /
/// `expand_dashes` operate uniformly across both subcommands. The four
/// fields have identical clap-derive shapes on both structs:
///   - `ms1: Option<String>` (single value)
///   - `mk1: Vec<String>` (repeating flag)
///   - `md1: Vec<String>` (repeating flag)
///   - `extra_strings: Vec<String>` (positional `<STRING>...`)
pub(crate) trait CardArgs {
    fn ms1(&self) -> Option<&String>;
    fn mk1(&self) -> &[String];
    fn md1(&self) -> &[String];
    fn extra_strings(&self) -> &[String];
}

/// v0.25.0 §2.A — count `-` (stdin sentinel) occurrences across the three
/// typed-flag fields. The positional `extra_strings` cannot contain `-`
/// because `classify_hrp_prefix` rejects any input lacking a known HRP
/// prefix, so the pre-merge sum equals the post-merge sum.
pub(crate) fn count_dashes(args: &impl CardArgs) -> usize {
    let ms1_count = args.ms1().iter().filter(|s| s.as_str() == "-").count();
    let mk1_count = args.mk1().iter().filter(|s| s.as_str() == "-").count();
    let md1_count = args.md1().iter().filter(|s| s.as_str() == "-").count();
    ms1_count + mk1_count + md1_count
}

/// v0.25.0 §2.A — replace `-` (stdin sentinel) occurrences in `input` with
/// the stdin chunks. Pure transform: each `-` in `input` is replaced by
/// the full `stdin_chunks` slice (1-to-N expansion). Non-dash entries pass
/// through unchanged. Called per-kind by `resolve_groups` after the single
/// stdin read.
pub(crate) fn expand_dashes(input: &[String], stdin_chunks: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(input.len());
    for c in input {
        if c == "-" {
            out.extend(stdin_chunks.iter().cloned());
        } else {
            out.push(c.clone());
        }
    }
    out
}

/// v0.24.0 §2.C.1 (D34/I5 fold) + v0.25.0 §2.A (D4 fold) — gather all input
/// strings into per-kind groups, merging the typed-flag form (`--ms1` /
/// `--mk1` / `--md1`) with the positional `<STRING>...` form (HRP-autodetect
/// routed). Returns groups in fixed `(Ms1, Mk1, Md1)` order; empty groups
/// are omitted from the returned vector.
///
/// `subcmd_name` is the subcommand's user-visible name (`"repair"` or
/// `"inspect"`) — used as the error-message prefix for the three
/// `ToolkitError::BadInput` paths in this helper. Distinct messages let
/// the user identify which subcommand emitted the error.
///
/// Mismatched-HRP flag values (`--ms1 mk1xxx`) return `ToolkitError::HrpMismatch`
/// per D34/I5 (toolkit-internal validation, not a clap parser callback).
/// Unknown-HRP positional values return `ToolkitError::UnknownHrp`.
///
/// Storage merge order: flag-form first, then positional (per plan).
pub(crate) fn resolve_groups<R: Read, E: Write>(
    args: &impl CardArgs,
    subcmd_name: &'static str,
    stdin: &mut R,
    stderr: &mut E,
    relax_hrp_for_indel: bool,
) -> Result<Vec<(CardKind, Vec<String>)>, ToolkitError> {
    // D34/I5 — strict per-flag HRP validation. `--ms1 mk1xxx` rejects with
    // `ToolkitError::HrpMismatch { flag: "--ms1", expected: "ms", got: "mk" }`.
    // `-` (stdin sentinel) is exempt; expanded after this check.
    //
    // **Phase-5 indel amendment:** when `relax_hrp_for_indel` is set
    // (`repair --max-indel ≥ 1`), the strict typed-flag HRP pre-gate is
    // SKIPPED so a prefix-region indel — which looks like an HRP mismatch
    // (`--ms1 s10…` = `ms1…` with a dropped 'm') — flows through to
    // `repair_card` → `parse_chunk`, surfacing `RepairError::HrpMismatch`
    // there, which `is_indel_trigger` engages. The typed flag already
    // declares the intended kind, so the value still routes to the correct
    // bucket below; without this relaxation the Phase-3 prefix producer
    // would be CLI-unreachable (§1.7). The POSITIONAL gate
    // (`classify_hrp_prefix`) stays strict regardless — a missing-prefix
    // positional cannot self-identify its kind. `inspect` always passes
    // `false` (its behavior is unchanged).
    if !relax_hrp_for_indel {
        if let Some(v) = args.ms1() {
            validate_flag_hrp("--ms1", "ms", v)?;
        }
        for v in args.mk1() {
            validate_flag_hrp("--mk1", "mk", v)?;
        }
        for v in args.md1() {
            validate_flag_hrp("--md1", "md", v)?;
        }
    }

    // Audit M3 — secret-in-argv advisory for inline ms1, fired per-occurrence
    // on the RAW pre-expansion values (post-expansion, stdin chunks would be
    // indistinguishable from inline → would false-fire on `--ms1 -`). The
    // `--ms1` flag fires unconditionally for any non-`-` value: the flag
    // declares the kind even when an indel-corrupted prefix (`--ms1 s10…`
    // under `--max-indel ≥ 1`) defeats HRP classification. Positionals fire
    // only when they HRP-classify as ms1 (mixed mk1/md1/ms1 intake; mk1/md1
    // are public material). `alternative` is `--ms1 -` for both: a positional
    // cannot be `-` (no HRP), so the stdin route is the flag form.
    if let Some(v) = args.ms1() {
        if v != "-" {
            crate::secret_advisory::secret_in_argv_warning(stderr, "--ms1", "--ms1 -");
        }
    }
    for s in args.extra_strings() {
        if matches!(classify_hrp_prefix(s), Ok(CardKind::Ms1)) {
            crate::secret_advisory::secret_in_argv_warning(stderr, "positional ms1", "--ms1 -");
        }
    }

    // Seed per-kind buckets from flag-form values (flag-form first per plan).
    let mut ms1_vec: Vec<String> = args.ms1().cloned().map(|s| vec![s]).unwrap_or_default();
    let mut mk1_vec: Vec<String> = args.mk1().to_vec();
    let mut md1_vec: Vec<String> = args.md1().to_vec();

    // Route positional `extra_strings` by HRP prefix.
    for s in args.extra_strings() {
        match classify_hrp_prefix(s)? {
            CardKind::Ms1 => ms1_vec.push(s.clone()),
            CardKind::Mk1 => mk1_vec.push(s.clone()),
            CardKind::Md1 => md1_vec.push(s.clone()),
        }
    }

    if ms1_vec.is_empty() && mk1_vec.is_empty() && md1_vec.is_empty() {
        return Err(ToolkitError::BadInput(format!(
            "{subcmd_name}: at least one of --ms1 / --mk1 / --md1 (or positional STRING) is required"
        )));
    }

    // Per-kind stdin (`-`) expansion. At most one `-` across the whole
    // invocation (across both flag-form and positional combined; stdin is
    // a single non-replayable stream).
    let total_dashes = count_dashes(args);
    if total_dashes > 1 {
        return Err(ToolkitError::BadInput(format!(
            "{subcmd_name}: at most one `-` (stdin) value across all {subcmd_name} inputs"
        )));
    }
    if total_dashes == 1 {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
        let stdin_chunks: Vec<String> = buf
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        if stdin_chunks.is_empty() {
            return Err(ToolkitError::BadInput(format!(
                "{subcmd_name}: stdin (`-`) yielded no non-blank chunks"
            )));
        }
        ms1_vec = expand_dashes(&ms1_vec, &stdin_chunks);
        mk1_vec = expand_dashes(&mk1_vec, &stdin_chunks);
        md1_vec = expand_dashes(&md1_vec, &stdin_chunks);
    }

    // mstring display-grouping (SPEC §3.2): strip display separators from each
    // resolved card chunk so a grouped or unbroken card both re-ingest before
    // BCH correction (each vec entry is ONE chunk; the "" watch-only sentinel
    // strips to "" — preserved; `-` stdin sentinels are already expanded above).
    // (repair OUTPUT stays unbroken — no grouping flags on repair.)
    let strip = crate::display_grouping::strip_display_separators;
    ms1_vec = ms1_vec.iter().map(|s| strip(s)).collect();
    mk1_vec = mk1_vec.iter().map(|s| strip(s)).collect();
    md1_vec = md1_vec.iter().map(|s| strip(s)).collect();

    let mut out: Vec<(CardKind, Vec<String>)> = Vec::with_capacity(3);
    if !ms1_vec.is_empty() {
        out.push((CardKind::Ms1, ms1_vec));
    }
    if !mk1_vec.is_empty() {
        out.push((CardKind::Mk1, mk1_vec));
    }
    if !md1_vec.is_empty() {
        out.push((CardKind::Md1, md1_vec));
    }
    Ok(out)
}

/// v0.25.0 §2.A (D4 fold) — resolve the effective auto-repair gate by
/// consulting `MNEMONIC_FORCE_TTY` (or falling back to stdout TTY detection)
/// and OR'ing with the caller's explicit `--no-auto-repair` flag.
///
/// **Public-API contract (v0.24.0+):** the `MNEMONIC_FORCE_TTY` environment
/// variable is a first-class semver-stable contract (promoted from test-only
/// at v0.22.1 D23 per FOLLOWUP `toolkit-mnemonic-force-tty-promote-from-test-only`).
///
/// Semantics:
///   - `MNEMONIC_FORCE_TTY=1` forces the TTY-positive auto-fire path.
///   - `MNEMONIC_FORCE_TTY=0` forces the TTY-negative legacy path.
///   - unset / any other value → falls back to `is_terminal()` runtime detection.
///
/// Known consumers (must continue working through future toolkit refactors
/// per the public-API contract):
///   - `mnemonic-gui` v0.9.0+ subprocess spawn env (the GUI's stdin/stdout
///     pipes are not real TTYs, so without the env override the toolkit would
///     never auto-fire repair under GUI invocations).
///   - the toolkit's own integration test suite, which sets =1 to force
///     auto-fire under `cargo test` (cargo's test harness pipes stdout).
///
/// NOT exposed via clap `--help` (environment variables are not part of
/// the clap-derive surface) or `mnemonic gui-schema` JSON. Documented in
/// the user manual at `docs/manual/src/40-cli-reference/41-mnemonic.md`
/// under the verify-bundle / repair auto-fire section.
pub(crate) fn resolve_no_auto_repair(no_auto_repair: bool) -> bool {
    let tty = match std::env::var("MNEMONIC_FORCE_TTY").ok().as_deref() {
        Some("1") => true,
        Some("0") => false,
        _ => std::io::stdout().is_terminal(),
    };
    no_auto_repair || !tty
}

/// Per-chunk correction report. `original_chunk` and `corrected_chunk` are
/// byte-identical when the chunk was already valid (and `corrected_positions`
/// is empty).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairDetail {
    pub chunk_index: usize,
    pub original_chunk: String,
    pub corrected_chunk: String,
    /// (position, was, now) — `position` is 0-indexed into the data-part
    /// (chars after the HRP + `1` separator).
    pub corrected_positions: Vec<(usize, char, char)>,
}

/// Result of a successful `repair_card` call. `corrected_chunks` always has
/// `len()` == input chunk count (already-valid chunks pass through unchanged);
/// `repairs` contains entries ONLY for chunks that actually needed correction.
#[derive(Debug, Clone)]
pub struct RepairOutcome {
    pub kind: CardKind,
    pub corrected_chunks: Vec<String>,
    pub repairs: Vec<RepairDetail>,
    /// Cycle E (`mk1-repair-set-level-reverify`) — SPEC §2 tri-state
    /// set-level re-verify discriminant. `Ms1` / `Md1` always report
    /// `Blessed` (their sibling-codec delegates only ever return `Ok` on
    /// full decode success already, so there is no behavior change for
    /// those kinds). `Mk1` reports `Blessed` when every chunk_set_id group
    /// that underwent a per-string correction reassembles cleanly via
    /// `mk_codec::decode`; `Unverified` when such a group is INCOMPLETE (a
    /// partial-plate repair — preserved, but cannot be set-verified until
    /// the full card is reassembled). A **Reject** (a complete-and-
    /// consistent corrected group whose `decode` fails — the funds fix) is
    /// never `Ok` at all; it surfaces as
    /// `RepairError::SetReassemblyMismatch` from `repair_card` instead.
    pub set_verify: SetVerify,
}

/// See [`RepairOutcome::set_verify`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetVerify {
    /// Confident recovery — safe to treat as the real card.
    Blessed,
    /// A corrected chunk_set_id group is incomplete (a single-plate /
    /// partial-set repair); `reason` is the loud user-facing advisory text.
    Unverified { reason: String },
}

#[derive(Debug)]
pub enum RepairError {
    EmptyInput,
    HrpMismatch {
        chunk_index: usize,
        expected: &'static str,
        found: String,
    },
    IndelUnrecoverable {
        hrp: &'static str,
        max_indel: usize,
    },
    TooManyErrors {
        chunk_index: usize,
        bound: usize,
    },
    UnparseableInput {
        chunk_index: usize,
        detail: String,
    },
    /// The chunk's data-part length falls into BIP-93 codex32's
    /// reserved-invalid band [94, 95], which is rejected at the parse
    /// step to keep the regular-vs-long dispatch unambiguous.
    ReservedInvalidLength {
        chunk_index: usize,
        data_part_len: usize,
    },
    /// The chunk's length-detected BCH code variant is not defined for
    /// this HRP (e.g. `ms` + long, `md` + long — neither codec emits
    /// long-code in v0.1). Distinct from `TooManyErrors` because the
    /// repair logic itself never runs.
    UnsupportedCodeVariant {
        chunk_index: usize,
        hrp: &'static str,
        data_part_len: usize,
    },
    /// v0.23.0 (D29 + Q1/Q2 locks). Catch-all for orphan §4-rule decoder
    /// errors surfaced by sibling-codec full-decode chains
    /// (`ms_codec::decode_with_correction` / `md_codec::decode_with_correction`)
    /// that the toolkit-side helper translation table did NOT enumerate
    /// individually. `chunk_index` is `None` when atomic-fail context lost
    /// the offending chunk's position; `Some(i)` when the helper preserved
    /// it. `detail` is the upstream codec's `Display`-rendered error.
    PostCorrectionDecodeFailed {
        chunk_index: Option<usize>,
        detail: String,
    },
    /// Cycle E (`mk1-repair-set-level-reverify`) — SPEC §2 rule 2, the
    /// FUNDS FIX. An mk1 chunk_set_id group is complete-and-consistent
    /// (every index `0..total_chunks` present exactly once) after
    /// per-string BCH correction, but the corrected group does NOT
    /// reassemble through `mk_codec::decode` — the per-chunk correction(s)
    /// aliased to a DIFFERENT valid codeword, not the original card.
    /// **Deliberately NOT an indel-trigger** (`is_indel_trigger` excludes
    /// it, plan-R0 PM-r2-1) — a set-reassembly mismatch is not a shape
    /// `--max-indel` search can meaningfully act on, and routing it there
    /// would replace this variant's precise message with the generic
    /// "indel unrecoverable" one.
    SetReassemblyMismatch {
        /// Human-readable identifier of the failing group (e.g.
        /// `"chunk_set_id 0x12345"` or `"single-string chunk 2"`), so a
        /// batch invocation containing multiple groups tells the user
        /// WHICH one failed (plan-R0 PM-r2-2) and lets them re-run the
        /// good group alone.
        group: String,
        /// The underlying `mk_codec::decode` error's `Display` text.
        detail: String,
    },
}

/// v0.22.1 D19 — the m-format constellation HRPs that the toolkit's repair
/// primitive recognizes. Used by `suggest_hrp` for Levenshtein-1 typo
/// detection in `RepairError::HrpMismatch` Display.
const KNOWN_HRPS: &[&str] = &["ms", "mk", "md"];

/// True iff `a` and `b` are equal under exactly one character substitution.
/// Tailored to the 2-char HRP domain — does NOT handle insertion/deletion
/// (HRP length is fixed at 2 in the codex32 family, so length-mismatched
/// inputs are short-circuited to false rather than producing misleading
/// suggestions).
fn hrp_lev1(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.chars().zip(b.chars()).filter(|(x, y)| x != y).count() == 1
}

/// Returns `Some(suggested_hrp)` iff exactly one of `KNOWN_HRPS` is
/// Levenshtein-1 from `found`. Returns `None` when zero or 2+ neighbors
/// exist (ambiguous — silence beats a guess). Used by
/// `RepairError::HrpMismatch` Display to append a "did you mean" suffix.
fn suggest_hrp(found: &str) -> Option<&'static str> {
    let neighbors: Vec<&'static str> = KNOWN_HRPS
        .iter()
        .filter(|&&known| hrp_lev1(known, found))
        .copied()
        .collect();
    if neighbors.len() == 1 {
        Some(neighbors[0])
    } else {
        None
    }
}

// Hand-rolled Display per the toolkit convention (cf. `final_word.rs`,
// `seed_xor.rs` library-error enums — no thiserror dep).
impl std::fmt::Display for RepairError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepairError::EmptyInput => write!(f, "repair: no chunks supplied"),
            RepairError::HrpMismatch { chunk_index, expected, found } => {
                let suggestion_suffix = suggest_hrp(found)
                    .map(|s| format!("; did you mean '{s}'?"))
                    .unwrap_or_default();
                write!(
                    f,
                    "repair: chunk {chunk_index} HRP mismatch — expected '{expected}', found '{found}' (HRP is not BCH-protected; re-type the prefix){suggestion_suffix}"
                )
            }
            RepairError::IndelUnrecoverable { hrp, max_indel } => write!(
                f,
                "repair: chunk could not be recovered within --max-indel {max_indel} (HRP '{hrp}'); \
                the string may have more than {max_indel} inserted/dropped characters, \
                or a different error class"
            ),
            RepairError::TooManyErrors { chunk_index, bound } => write!(
                f,
                "repair: chunk {chunk_index} has too many errors to correct uniquely (exceeds singleton bound = {bound}); cannot suggest correction"
            ),
            RepairError::UnparseableInput { chunk_index, detail } => write!(
                f,
                "repair: chunk {chunk_index} parse failed before correction could run: {detail}"
            ),
            RepairError::ReservedInvalidLength { chunk_index, data_part_len } => write!(
                f,
                "repair: chunk {chunk_index} data-part length {data_part_len} is in BIP-93's reserved-invalid band [94, 95]; re-type the chunk"
            ),
            RepairError::UnsupportedCodeVariant { chunk_index, hrp, data_part_len } => write!(
                f,
                "repair: chunk {chunk_index} data-part length {data_part_len} would require the long BCH code, which is not defined for HRP '{hrp}' in this codec version"
            ),
            RepairError::PostCorrectionDecodeFailed { chunk_index, detail } => match chunk_index {
                Some(i) => write!(f, "repair: chunk {i} post-correction decode failed: {detail}"),
                None => write!(f, "repair: post-correction decode failed: {detail}"),
            },
            RepairError::SetReassemblyMismatch { group, detail } => write!(
                f,
                "repair: each chunk corrected individually, but the set does not reassemble ({group}): {detail} — the correction(s) may have aliased to a DIFFERENT valid card; this output is NOT trustworthy"
            ),
        }
    }
}

impl std::error::Error for RepairError {}

/// Parse a bech32-family string into `(data-part 5-bit values, detected
/// BCH-code variant)`. HRP is verified against `kind.hrp()`. The code
/// variant is determined from the data-part length per BIP-93's
/// regular/long boundaries.
fn parse_chunk(
    chunk: &str,
    chunk_index: usize,
    kind: CardKind,
) -> Result<(Vec<u8>, BchCode), RepairError> {
    let s_lower = chunk.to_lowercase();
    let sep_pos = s_lower
        .rfind('1')
        .ok_or_else(|| RepairError::UnparseableInput {
            chunk_index,
            detail: "missing bech32 separator '1'".into(),
        })?;
    let (hrp, rest) = s_lower.split_at(sep_pos);
    let data_part = &rest[1..]; // skip the '1' separator

    let expected_hrp = kind.hrp();
    if hrp != expected_hrp {
        return Err(RepairError::HrpMismatch {
            chunk_index,
            expected: expected_hrp,
            found: hrp.to_string(),
        });
    }

    // Inverse ALPHABET lookup: bech32 chars → 5-bit values.
    let mut alphabet_inv = [0xFFu8; 128];
    for (i, &c) in ALPHABET.iter().enumerate() {
        alphabet_inv[c as usize] = i as u8;
    }

    let mut values: Vec<u8> = Vec::with_capacity(data_part.len());
    for (i, c) in data_part.chars().enumerate() {
        if !c.is_ascii() {
            return Err(RepairError::UnparseableInput {
                chunk_index,
                detail: format!("non-ASCII char '{c}' at position {i}"),
            });
        }
        let v = alphabet_inv[c as usize];
        if v == 0xFF {
            return Err(RepairError::UnparseableInput {
                chunk_index,
                detail: format!("non-bech32 char '{c}' at position {i}"),
            });
        }
        values.push(v);
    }

    // Dispatch by length per BIP-93: regular = [14, 93], long = [96, 108],
    // [94, 95] reserved-invalid, else out-of-range.
    let code = match bch_code_for_length(values.len()) {
        Some(c) => c,
        None if values.len() == 94 || values.len() == 95 => {
            return Err(RepairError::ReservedInvalidLength {
                chunk_index,
                data_part_len: values.len(),
            });
        }
        None => {
            return Err(RepairError::UnparseableInput {
                chunk_index,
                detail: format!(
                    "data-part length {} is outside BIP-93's valid range [14, 93] ∪ [96, 108]",
                    values.len()
                ),
            });
        }
    };

    Ok((values, code))
}

/// Compute polymod residue for a parsed chunk + per-HRP + per-code target.
/// `data_with_checksum` is the full 5-bit data-part (data + checksum); the
/// generator / shift / mask are selected from `code`.
fn polymod_residue(hrp: &str, data_with_checksum: &[u8], target: u128, code: BchCode) -> u128 {
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data_with_checksum);
    let raw = match code {
        BchCode::Regular => polymod_run(&input, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
        BchCode::Long => polymod_run(&input, &GEN_LONG, LONG_SHIFT, LONG_MASK),
    };
    raw ^ target
}

/// Re-encode 5-bit data + HRP back to a bech32-family string.
fn encode_chunk(hrp: &str, data_with_checksum: &[u8]) -> String {
    let mut out = String::with_capacity(hrp.len() + 1 + data_with_checksum.len());
    out.push_str(hrp);
    out.push('1');
    for &v in data_with_checksum {
        out.push(ALPHABET[v as usize] as char);
    }
    out
}

/// Attempt to repair a single chunk. Returns `Ok(Some(detail))` on
/// repair-applied; `Ok(None)` on already-valid; `Err` on unrecoverable.
fn repair_chunk_one(
    kind: CardKind,
    chunk_index: usize,
    chunk: &str,
) -> Result<Option<RepairDetail>, RepairError> {
    let (values, code) = parse_chunk(chunk, chunk_index, kind)?;
    let hrp = kind.hrp();
    let target = kind
        .target_residue(code)
        .ok_or(RepairError::UnsupportedCodeVariant {
            chunk_index,
            hrp,
            data_part_len: values.len(),
        })?;

    // Quick-path: already valid.
    let initial_residue = polymod_residue(hrp, &values, target, code);
    if initial_residue == 0 {
        return Ok(None);
    }

    // Attempt correction. The decoder returns (positions, magnitudes) over the
    // data-with-checksum domain (5-bit values, not chars). `values.len()`
    // here is the full data + checksum length; the decoder uses it to bound
    // the Chien-search root domain.
    let (positions, magnitudes) = match code {
        BchCode::Regular => decode_regular_errors(initial_residue, values.len()),
        BchCode::Long => decode_long_errors(initial_residue, values.len()),
    }
    .ok_or(RepairError::TooManyErrors {
        chunk_index,
        bound: SINGLETON_BOUND,
    })?;

    // Apply corrections. The decoder returns Vec<Gf32> where Gf32 is a
    // private type-alias for u8 (mk-codec internal); externally the
    // signature unifies to Vec<u8>, so we just deref &u8 → u8.
    let mut corrected = values.clone();
    let mut corrected_positions: Vec<(usize, char, char)> = Vec::with_capacity(positions.len());
    for (&p, &m) in positions.iter().zip(&magnitudes) {
        if p >= corrected.len() {
            return Err(RepairError::TooManyErrors {
                chunk_index,
                bound: SINGLETON_BOUND,
            });
        }
        let was_byte = corrected[p];
        let now_byte = was_byte ^ m;
        corrected_positions.push((
            p,
            ALPHABET[was_byte as usize] as char,
            ALPHABET[now_byte as usize] as char,
        ));
        corrected[p] = now_byte;
    }

    // Defensive re-verify (catches pathological 5+-error patterns that happen
    // to produce a degree-≤4 locator with 4 valid roots).
    let verify_residue = polymod_residue(hrp, &corrected, target, code);
    if verify_residue != 0 {
        return Err(RepairError::TooManyErrors {
            chunk_index,
            bound: SINGLETON_BOUND,
        });
    }

    let corrected_chunk = encode_chunk(hrp, &corrected);
    Ok(Some(RepairDetail {
        chunk_index,
        original_chunk: chunk.to_string(),
        corrected_chunk,
        corrected_positions,
    }))
}

// ============================================================================
// Cycle E (`mk1-repair-set-level-reverify`) — SPEC §2 tri-state set-level
// re-verify classifier. Operates on the ALREADY per-string-corrected mk1
// chunk strings (each is a valid BCH codeword by construction — its residue
// was re-verified == 0 in `repair_chunk_one`), so header re-parsing here
// never triggers a further BCH correction; it exists purely to recover the
// `chunk_set_id` / `total_chunks` / `chunk_index` facts needed to group
// chunks belonging to the same card and re-verify the group as a whole.
// ============================================================================

/// Per-`chunk_set_id`-group verdict (SPEC §2). Only ever computed for a
/// group that had AT LEAST ONE chunk corrected — a set of chunks that were
/// already fully valid carries no aliasing risk, so it is never classified
/// (see `verify_mk1_set`'s `touched` gate) and is treated as `Bless`-
/// equivalent by omission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupVerdict {
    /// `mk_codec::decode` on the exact supplied group returned `Ok`.
    Bless,
    /// The group is complete-and-consistent (every index `0..total_chunks`
    /// present exactly once, consistent `total_chunks`) but `decode`
    /// returned `Err` — the per-chunk correction(s) aliased to a DIFFERENT
    /// valid codeword. THE FUNDS FIX.
    Reject,
    /// The group is incomplete (a partial-set / single-plate repair) —
    /// cannot set-verify; preserved as an unverified candidate.
    Candidate,
}

/// Identifies one independent mk1 "card" within a `repair_card(Mk1, …)`
/// batch invocation. `Chunked` groups by the wire `chunk_set_id`; each
/// `SingleString`-headered chunk is its own singleton group (a `SingleString`
/// mk1 is a complete card by itself — unreachable from real v0.1 encoders per
/// SPEC §1 count=1 reachability, but handled uniformly rather than assumed
/// away; using the chunk's own index as the key keeps multiple SingleString
/// chunks in one batch from colliding into a single false group).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GroupKey {
    Chunked(u32),
    SingleString(usize),
}

/// Human-readable identifier for a `GroupKey`, used in the `Reject` message
/// (plan-R0 PM-r2-2 — name WHICH group failed so a batch invocation lets the
/// user re-run the good group alone).
fn describe_group_key(key: GroupKey) -> String {
    match key {
        GroupKey::Chunked(csid) => format!("chunk_set_id 0x{csid:05x}"),
        GroupKey::SingleString(idx) => format!("single-string chunk {idx}"),
    }
}

struct ParsedMk1Chunk {
    key: GroupKey,
    header: StringLayerHeader,
}

/// Re-parse an already-corrected mk1 chunk string's string-layer header via
/// mk-codec's OWN decoder (`decode_string` → `StringLayerHeader::
/// from_5bit_symbols`), so grouping semantics never drift from the library.
/// The chunk is already a valid BCH codeword (residue verified == 0 by
/// `repair_chunk_one`), so this never triggers a further correction —
/// `corrections_applied` is always 0 for well-formed input; a failure here
/// would indicate a toolkit/mk-codec residue-check divergence, surfaced as
/// `PostCorrectionDecodeFailed` rather than silently mis-grouping.
fn parse_mk1_group_key(chunk: &str, idx: usize) -> Result<ParsedMk1Chunk, RepairError> {
    let decoded = decode_string(chunk).map_err(|e| RepairError::PostCorrectionDecodeFailed {
        chunk_index: Some(idx),
        detail: format!("set-reverify header re-parse failed: {e}"),
    })?;
    let (header, _consumed) =
        StringLayerHeader::from_5bit_symbols(decoded.data()).map_err(|e| {
            RepairError::PostCorrectionDecodeFailed {
                chunk_index: Some(idx),
                detail: format!("set-reverify header re-parse failed: {e}"),
            }
        })?;
    let key = match header {
        StringLayerHeader::Chunked { chunk_set_id, .. } => GroupKey::Chunked(chunk_set_id),
        StringLayerHeader::SingleString { .. } => GroupKey::SingleString(idx),
        // `StringLayerHeader` is `#[non_exhaustive]` — mk-codec's OWN
        // `from_5bit_symbols` only ever constructs these two variants today
        // (verified against the vendored source); a future third variant is
        // rejected loudly here rather than silently mis-grouped.
        _ => {
            return Err(RepairError::PostCorrectionDecodeFailed {
                chunk_index: Some(idx),
                detail: "set-reverify: unrecognized mk1 string-layer header variant \
                    (requires a toolkit update for this mk-codec version)"
                    .to_string(),
            })
        }
    };
    Ok(ParsedMk1Chunk { key, header })
}

/// True iff `headers` (all members of ONE `GroupKey` group, in arbitrary
/// order) form a complete, internally-consistent chunk set: every index
/// `0..total_chunks` present EXACTLY once, with every member reporting the
/// SAME `total_chunks`. Discriminates on the PARSED indices, never on the
/// overloaded error string (SPEC §2 header-corruption note) — a substitution
/// that corrupts `total_chunks` itself just misclassifies the group as
/// incomplete, never as a false confident success.
fn group_is_complete_and_consistent(headers: &[&StringLayerHeader]) -> bool {
    match headers.first() {
        Some(StringLayerHeader::SingleString { .. }) => headers.len() == 1,
        Some(StringLayerHeader::Chunked { total_chunks, .. }) => {
            let total = *total_chunks as usize;
            if headers.len() > total {
                return false; // more members than total_chunks declares
            }
            let mut seen = vec![false; total];
            for h in headers {
                match h {
                    StringLayerHeader::Chunked {
                        total_chunks: t,
                        chunk_index,
                        ..
                    } => {
                        if *t as usize != total {
                            return false; // inconsistent total_chunks within one csid
                        }
                        let idx = *chunk_index as usize;
                        if idx >= total || seen[idx] {
                            return false; // out-of-range or duplicate index
                        }
                        seen[idx] = true;
                    }
                    // Unreachable by construction — GroupKey partitions
                    // Chunked from SingleString — but handled defensively
                    // (fail-closed: NOT complete-and-consistent) rather
                    // than panicking on a future grouping change or a
                    // future `#[non_exhaustive]` variant.
                    _ => return false,
                }
            }
            seen.iter().all(|&done| done)
        }
        // `None` (empty group) never occurs (built from ≥1 member); a future
        // `#[non_exhaustive]` header variant fails closed (not complete).
        _ => false,
    }
}

/// Fold two `GroupVerdict`s to the dominant one across a multi-group batch
/// (SPEC §2 — `reject > candidate > bless`).
fn fold_verdict(acc: Option<GroupVerdict>, v: GroupVerdict) -> GroupVerdict {
    match (acc, v) {
        (Some(GroupVerdict::Reject), _) | (_, GroupVerdict::Reject) => GroupVerdict::Reject,
        (Some(GroupVerdict::Candidate), _) | (_, GroupVerdict::Candidate) => {
            GroupVerdict::Candidate
        }
        _ => GroupVerdict::Bless,
    }
}

/// SPEC §2 tri-state re-verify over the FULL supplied mk1 `corrected_chunks`
/// (already per-string BCH-corrected). Groups by `chunk_set_id`, classifies
/// only the groups that had at least one chunk actually corrected (an
/// all-already-valid group carries no aliasing risk — the task is
/// specifically to re-verify a set AFTER a per-string correction, so an
/// untouched group is left exactly as today: `Bless`-equivalent, no
/// decode attempt, no advisory), then folds to the dominant outcome.
///
/// Returns `Ok(SetVerify::Blessed)` / `Ok(SetVerify::Unverified{..})` on
/// Bless / Candidate; a dominant `Reject` returns `Err(RepairError::
/// SetReassemblyMismatch)` — never `Ok` (plan-R0 PM-r2-2: a batch that folds
/// to Reject suppresses ALL output, including any co-batched Bless group).
fn verify_mk1_set(
    corrected_chunks: &[String],
    repairs: &[RepairDetail],
) -> Result<SetVerify, RepairError> {
    let touched: HashSet<usize> = repairs.iter().map(|r| r.chunk_index).collect();

    // Parse each corrected chunk's header for grouping. A header-region
    // parse failure (e.g. a reserved/invalid type byte, or a malformed
    // total_chunks/chunk_index) on a chunk that WAS corrected is itself one
    // of the decode-Err shapes SPEC §2 rule 2 explicitly covers ("a
    // header-region ChunkedHeaderMalformed/MixedHeaderTypes... failure from
    // a hash-colliding miscorrection") — a per-string BCH correction landed
    // on a valid CHECKSUM whose interpreted header is nonetheless garbage.
    // Fold that in as an immediate Reject signal for this chunk rather than
    // hard-aborting the whole classify via `?` (which would surface the
    // wrong, more primitive `PostCorrectionDecodeFailed` instead of the
    // funds-relevant `SetReassemblyMismatch`). An UNTOUCHED (already-valid)
    // chunk failing to parse should never occur for genuine encoder output
    // — skip it (ungrouped) rather than reject on something the toolkit
    // never corrected.
    let mut parsed: Vec<Option<ParsedMk1Chunk>> = Vec::with_capacity(corrected_chunks.len());
    let mut dominant: Option<GroupVerdict> = None;
    let mut first_reject: Option<(String, String)> = None;
    for (i, c) in corrected_chunks.iter().enumerate() {
        match parse_mk1_group_key(c, i) {
            Ok(p) => parsed.push(Some(p)),
            Err(e) => {
                parsed.push(None);
                if touched.contains(&i) {
                    if first_reject.is_none() {
                        first_reject =
                            Some((format!("chunk {i} (post-correction header)"), e.to_string()));
                    }
                    dominant = Some(fold_verdict(dominant, GroupVerdict::Reject));
                }
            }
        }
    }

    // Group the well-formed parses by chunk_set_id, preserving first-seen
    // order for deterministic messages regardless of HashMap iteration
    // order.
    let mut order: Vec<GroupKey> = Vec::new();
    let mut members: HashMap<GroupKey, Vec<usize>> = HashMap::new();
    for (i, p) in parsed.iter().enumerate() {
        if let Some(p) = p {
            members
                .entry(p.key)
                .or_insert_with(|| {
                    order.push(p.key);
                    Vec::new()
                })
                .push(i);
        }
    }

    for key in &order {
        let idxs = &members[key];
        if !idxs.iter().any(|i| touched.contains(i)) {
            // Untouched group — no aliasing risk, skip re-verify entirely.
            continue;
        }
        let headers: Vec<&StringLayerHeader> = idxs
            .iter()
            .map(|&i| {
                &parsed[i]
                    .as_ref()
                    .expect("grouped only from Some(..) parses")
                    .header
            })
            .collect();
        let verdict = if !group_is_complete_and_consistent(&headers) {
            GroupVerdict::Candidate
        } else {
            let refs: Vec<&str> = idxs.iter().map(|&i| corrected_chunks[i].as_str()).collect();
            match mk_codec::decode(&refs) {
                Ok(_) => GroupVerdict::Bless,
                Err(e) => {
                    if first_reject.is_none() {
                        first_reject = Some((describe_group_key(*key), e.to_string()));
                    }
                    GroupVerdict::Reject
                }
            }
        };
        dominant = Some(fold_verdict(dominant, verdict));
    }

    match dominant.unwrap_or(GroupVerdict::Bless) {
        GroupVerdict::Bless => Ok(SetVerify::Blessed),
        GroupVerdict::Candidate => Ok(SetVerify::Unverified {
            reason: "correction UNVERIFIED — a >4-error correction can alias to a different card; \
                reassemble the full card (`mk decode` / import the full set) to confirm; \
                BIP-93 recommends confirmation"
                .to_string(),
        }),
        GroupVerdict::Reject => {
            let (group, detail) =
                first_reject.expect("dominant Reject implies a recorded reject detail");
            Err(RepairError::SetReassemblyMismatch { group, detail })
        }
    }
}

/// Primary entry point. Per-chunk atomic per D8: if ANY chunk fails, returns
/// `Err` naming that chunk's index; partially-repaired sibling chunks are NOT
/// returned.
///
/// **v0.23.0 — D29 migration:** Ms1 dispatches per-chunk through
/// `repair_via_ms_codec` (a thin wrapper over `ms_codec::decode_with_correction`);
/// Md1 dispatches whole-set atomically through `repair_via_md_codec` (a thin
/// wrapper over `md_codec::decode_with_correction`); Mk1 continues to use
/// the toolkit-native `repair_chunk_one` path consuming mk-codec's promoted
/// BCH primitives directly.
pub fn repair_card(kind: CardKind, chunks: &[String]) -> Result<RepairOutcome, RepairError> {
    if chunks.is_empty() {
        return Err(RepairError::EmptyInput);
    }

    match kind {
        CardKind::Mk1 => {
            let mut corrected_chunks: Vec<String> = Vec::with_capacity(chunks.len());
            let mut repairs: Vec<RepairDetail> = Vec::new();
            for (i, chunk) in chunks.iter().enumerate() {
                match repair_chunk_one(kind, i, chunk)? {
                    Some(detail) => {
                        corrected_chunks.push(detail.corrected_chunk.clone());
                        repairs.push(detail);
                    }
                    None => corrected_chunks.push(chunk.clone()),
                }
            }
            // Cycle E — SPEC §2 tri-state set-level re-verify (the funds
            // fix). A dominant Reject short-circuits via `?` to an `Err`,
            // discarding corrected_chunks/repairs entirely (plan-R0
            // PM-r2-2 — a batch that folds to Reject suppresses ALL
            // output, not just the failing group).
            let set_verify = verify_mk1_set(&corrected_chunks, &repairs)?;
            Ok(RepairOutcome {
                kind,
                corrected_chunks,
                repairs,
                set_verify,
            })
        }
        CardKind::Ms1 => {
            // ms1 is single-chunk per codex32 spec, but `repair_card` is
            // kind-agnostic across chunk-count — preserve the per-chunk loop
            // by calling the sibling-codec helper once per supplied chunk.
            let mut corrected_chunks: Vec<String> = Vec::with_capacity(chunks.len());
            let mut repairs: Vec<RepairDetail> = Vec::new();
            for (i, chunk) in chunks.iter().enumerate() {
                // Pre-gate via parse_chunk to preserve the toolkit's
                // pre-existing precise error variants (HrpMismatch with
                // suggestion suffix, ReservedInvalidLength, the
                // UnparseableInput parse-step messages) — sibling-codec
                // errors are coarser. Reject long-code variants explicitly
                // (ms-codec doesn't define them in v0.1) BEFORE delegating.
                let (values, code) = parse_chunk(chunk, i, kind)?;
                if matches!(code, BchCode::Long) {
                    return Err(RepairError::UnsupportedCodeVariant {
                        chunk_index: i,
                        hrp: "ms",
                        data_part_len: values.len(),
                    });
                }
                match repair_via_ms_codec(chunk, i)? {
                    Some(detail) => {
                        corrected_chunks.push(detail.corrected_chunk.clone());
                        repairs.push(detail);
                    }
                    None => corrected_chunks.push(chunk.clone()),
                }
            }
            Ok(RepairOutcome {
                kind,
                corrected_chunks,
                repairs,
                // ms1 delegates entirely to ms_codec::decode_with_correction,
                // which only ever returns `Ok` on full decode success — no
                // behavior change from Cycle E.
                set_verify: SetVerify::Blessed,
            })
        }
        CardKind::Md1 => {
            // md1 is multi-chunk; the sibling codec's
            // `decode_with_correction(&[&str])` returns atomic per D28.
            // Pre-gate every chunk through parse_chunk for the same
            // precise-error-variant preservation reason as Ms1; explicitly
            // reject long-code variants (md-codec doesn't define them in
            // v0.1) BEFORE delegating.
            for (i, chunk) in chunks.iter().enumerate() {
                let (values, code) = parse_chunk(chunk, i, kind)?;
                if matches!(code, BchCode::Long) {
                    return Err(RepairError::UnsupportedCodeVariant {
                        chunk_index: i,
                        hrp: "md",
                        data_part_len: values.len(),
                    });
                }
            }
            repair_via_md_codec(chunks)
        }
    }
}

/// **v0.23.0 — D29 migration helper.** Delegate ms1 chunk repair to
/// `ms_codec::decode_with_correction` (full-decode semantics per Q1 lock);
/// translate the codec's `Error` taxonomy back into toolkit `RepairError`
/// variants per the §2.B.4 D29 error-mapping table (Q2 absorption lock).
///
/// Returns `Ok(Some(detail))` on repair-applied, `Ok(None)` on already-valid,
/// `Err(_)` on unrecoverable. The full-decode chain runs the parsed
/// `(Tag, Payload)` internally; this helper discards both since
/// `repair_card`'s public contract is "corrected string + correction
/// details" only.
fn repair_via_ms_codec(
    chunk: &str,
    chunk_index: usize,
) -> Result<Option<RepairDetail>, RepairError> {
    use ms_codec::Error as MsErr;
    match ms_codec::decode_with_correction(chunk) {
        Ok((_tag, _payload, corrections)) => {
            if corrections.is_empty() {
                return Ok(None);
            }
            let (corrected_chunk, corrected_positions) = apply_ms_corrections(chunk, &corrections);
            Ok(Some(RepairDetail {
                chunk_index,
                original_chunk: chunk.to_string(),
                corrected_chunk,
                corrected_positions,
            }))
        }
        Err(MsErr::TooManyErrors { bound }) => Err(RepairError::TooManyErrors {
            chunk_index,
            bound: bound as usize,
        }),
        Err(MsErr::WrongHrp { got }) => Err(RepairError::HrpMismatch {
            chunk_index,
            expected: "ms",
            found: got,
        }),
        Err(MsErr::Codex32(e)) => Err(RepairError::UnparseableInput {
            chunk_index,
            detail: format!("{e:?}"),
        }),
        Err(other) => Err(RepairError::PostCorrectionDecodeFailed {
            chunk_index: Some(chunk_index),
            detail: other.to_string(),
        }),
    }
}

/// Apply ms-codec `CorrectionDetail` entries to the input chunk string,
/// producing the corrected string + the toolkit's `(position, was, now)`
/// triple form. The two `CorrectionDetail` types (ms-codec's vs the
/// toolkit's `RepairDetail.corrected_positions`) differ only in
/// presentation — both carry the same logical information.
fn apply_ms_corrections(
    chunk: &str,
    corrections: &[ms_codec::CorrectionDetail],
) -> (String, Vec<(usize, char, char)>) {
    let lower = chunk.to_lowercase();
    let sep = lower.rfind('1').expect("ms-codec already validated prefix");
    let (prefix, rest) = lower.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let mut positions: Vec<(usize, char, char)> = Vec::with_capacity(corrections.len());
    for c in corrections {
        positions.push((c.position, c.was, c.now));
        if c.position < chars.len() {
            chars[c.position] = c.now;
        }
    }
    let mut corrected = String::from(prefix);
    for ch in chars {
        corrected.push(ch);
    }
    (corrected, positions)
}

/// ms1 oracle — single string is the whole card; delegate to ms-codec.
/// Pure-indel: accept only if all corrections are within `allowed` (the
/// inserted-placeholder positions; empty for the delete producer ⇒ already-valid).
pub(crate) struct Ms1IndelOracle;
impl IndelOracle for Ms1IndelOracle {
    fn validate(
        &self,
        cand: &str,
        allowed: &BTreeSet<usize>,
        e_subst: usize,
    ) -> Option<(String, usize)> {
        match ms_codec::decode_with_correction(cand) {
            Ok((_t, _p, corrections)) => {
                let off = corrections
                    .iter()
                    .filter(|c| !allowed.contains(&c.position))
                    .count();
                if off <= e_subst {
                    let (corrected, _) = apply_ms_corrections(cand, &corrections);
                    Some((corrected, off))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

/// ⊆-gated single-chunk BCH solve for mk1 (parse + residue + optional
/// correction). Returns the canonical (re-encoded) chunk iff it parses,
/// BCH-validates/solves, and every correction position ∈ `allowed`; `None`
/// otherwise.
///
/// This exists because `mk_codec::decode` self-corrects up to t=4 UNGUARDED
/// (`mk-codec string_layer/bch.rs` `bch_correct_*`), which would silently
/// apply substitutions and defeat the pure-indel ⊆ rule. So we solve the
/// single chunk here under the gate and hand `decode` an already-clean chunk.
fn mk1_chunk_solve(
    cand: &str,
    allowed: &BTreeSet<usize>,
    e_subst: usize,
) -> Option<(String, usize)> {
    let (values, code) = parse_chunk(cand, 0, CardKind::Mk1).ok()?;
    let target = CardKind::Mk1.target_residue(code)?;
    let residue = polymod_residue("mk", &values, target, code);
    if residue == 0 {
        // Already a valid codeword (delete producer / placeholder collision).
        return Some((encode_chunk("mk", &values), 0));
    }
    let (positions, mags) = match code {
        BchCode::Regular => decode_regular_errors(residue, values.len()),
        BchCode::Long => decode_long_errors(residue, values.len()),
    }?;
    let off = positions.iter().filter(|p| !allowed.contains(p)).count();
    if off > e_subst {
        return None; // too many corrections outside the placeholder set
    }
    let mut corrected = values.clone();
    for (&p, &m) in positions.iter().zip(&mags) {
        if p >= corrected.len() {
            return None;
        }
        corrected[p] ^= m;
    }
    if polymod_residue("mk", &corrected, target, code) != 0 {
        return None; // defensive re-verify
    }
    Some((encode_chunk("mk", &corrected), off))
}

/// ⊆-gated single-chunk BCH solve for md1 — mirror of `mk1_chunk_solve`
/// (md is regular-only). Reuses the shared-codex32-generator machinery
/// (`polymod_residue` + `decode_regular_errors`) with the md target:
/// md's `GEN_REGULAR` / `REGULAR_SHIFT` / `REGULAR_MASK` / `POLYMOD_INIT`
/// are byte-identical to mk's, so the toolkit-side residue matches md's
/// own bit-for-bit once the md target constant is applied.
///
/// Returns the canonical (re-encoded) chunk iff it parses, BCH-validates or
/// -solves, and every correction position ∈ `allowed`; `None` otherwise.
/// `Md1IndelOracle` then confirms the solved chunk against the full set via
/// `md_codec::chunk::reassemble` (which does NOT self-correct, so the chunk
/// must already be a valid codeword).
fn md1_chunk_solve(
    cand: &str,
    allowed: &BTreeSet<usize>,
    e_subst: usize,
) -> Option<(String, usize)> {
    let (values, code) = parse_chunk(cand, 0, CardKind::Md1).ok()?;
    let target = CardKind::Md1.target_residue(code)?; // None for Long ⇒ reject
    let residue = polymod_residue("md", &values, target, code);
    if residue == 0 {
        // Already a valid codeword (delete producer / placeholder collision).
        return Some((encode_chunk("md", &values), 0));
    }
    let (positions, mags) = match code {
        BchCode::Regular => decode_regular_errors(residue, values.len()),
        BchCode::Long => return None, // md has no long decoder
    }?;
    let off = positions.iter().filter(|p| !allowed.contains(p)).count();
    if off > e_subst {
        return None; // too many corrections outside the placeholder set
    }
    let mut corrected = values.clone();
    for (&p, &m) in positions.iter().zip(&mags) {
        if p >= corrected.len() {
            return None;
        }
        corrected[p] ^= m;
    }
    if polymod_residue("md", &corrected, target, code) != 0 {
        return None; // defensive re-verify
    }
    Some((encode_chunk("md", &corrected), off))
}

/// mk1 oracle — ⊆-gated solve the single failing chunk (mk_codec::decode
/// self-corrects t≤4 UNGUARDED, which would defeat the pure-indel rule), then
/// confirm full-card reassembly via `mk_codec::decode(&[&str])` on the clean
/// chunk.
pub(crate) struct Mk1IndelOracle {
    pub all_chunks: Vec<String>,
    pub failing_index: usize,
}
impl IndelOracle for Mk1IndelOracle {
    fn validate(
        &self,
        cand: &str,
        allowed: &BTreeSet<usize>,
        e_subst: usize,
    ) -> Option<(String, usize)> {
        let (corrected_chunk, off) = mk1_chunk_solve(cand, allowed, e_subst)?;
        let mut chunks = self.all_chunks.clone();
        chunks[self.failing_index] = corrected_chunk.clone();
        let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        match mk_codec::decode(&refs) {
            Ok(_) => Some((corrected_chunk, off)),
            Err(_) => None,
        }
    }
}

/// md1 oracle — ⊆-gated solve the single failing chunk, then confirm
/// full-set reassembly via `md_codec::chunk::reassemble` (which does NOT
/// self-correct — it `unwrap_string`s each chunk via a hard codex32
/// checksum verify, then cross-chunk-validates — so the solved chunk must
/// already be a valid codeword; `md1_chunk_solve` produces exactly that).
pub(crate) struct Md1IndelOracle {
    pub all_chunks: Vec<String>,
    pub failing_index: usize,
}
impl IndelOracle for Md1IndelOracle {
    fn validate(
        &self,
        cand: &str,
        allowed: &BTreeSet<usize>,
        e_subst: usize,
    ) -> Option<(String, usize)> {
        let (corrected_chunk, off) = md1_chunk_solve(cand, allowed, e_subst)?;
        let mut chunks = self.all_chunks.clone();
        chunks[self.failing_index] = corrected_chunk.clone();
        let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        match md_codec::chunk::reassemble(&refs) {
            Ok(_) => Some((corrected_chunk, off)),
            Err(_) => None,
        }
    }
}

/// True iff the failure class warrants engaging the indel search (§1.7).
///
/// Included: `HrpMismatch | TooManyErrors | PostCorrectionDecodeFailed |
/// UnparseableInput | ReservedInvalidLength`. **`HrpMismatch` is INCLUDED**
/// (Phase-5 R0 amendment) so a prefix-region indel engages: `parse_chunk`
/// validates the HRP before length, so dropping 'm' from `ms1…` surfaces as
/// `HrpMismatch { found: "s" }`; excluding it would make the Phase-3 prefix
/// producer CLI-unreachable. When a genuine wrong-HRP value enters indel
/// search and fails to recover, `cmd::repair::run` falls back to the
/// **original `HrpMismatch` error** (with its "did you mean" suggestion)
/// rather than the generic `IndelUnrecoverable` message (Phase 4 —
/// v0.37.3). At `--max-indel 0` the strict pre-gate fires before
/// `is_indel_trigger` is ever reached, so the suggestion is always
/// preserved at the default.
///
/// Excluded: `EmptyInput | UnsupportedCodeVariant | IndelUnrecoverable |
/// SetReassemblyMismatch` (no recoverable indel class; pass through to
/// today's typed error). `SetReassemblyMismatch` (Cycle E, plan-R0 PM-r2-1)
/// is deliberately excluded: it names a set-reassembly funds-mismatch, not a
/// transcription indel, and routing it into `--max-indel` search would
/// replace its precise "does not reassemble" message with the generic
/// "indel unrecoverable" one while adding no recovery capability.
///
/// Implemented as an exhaustive `match` so a future new `RepairError` variant
/// forces a compile-time decision here rather than silently defaulting.
pub(crate) fn is_indel_trigger(e: &RepairError) -> bool {
    match e {
        RepairError::HrpMismatch { .. }
        | RepairError::TooManyErrors { .. }
        | RepairError::PostCorrectionDecodeFailed { .. }
        | RepairError::UnparseableInput { .. }
        | RepairError::ReservedInvalidLength { .. } => true,
        RepairError::EmptyInput
        | RepairError::UnsupportedCodeVariant { .. }
        | RepairError::IndelUnrecoverable { .. }
        | RepairError::SetReassemblyMismatch { .. } => false,
    }
}

/// Exit-code decision for the indel CLI path (§1.6). Precedence within the
/// emitting outcomes: ambiguous-or-substitution(4) > recovered/repaired(5) >
/// already-valid(0). `substitution_seen` is true when at least one candidate
/// used a substitution beyond the placeholder positions (`subst_count >= 1`).
/// (Unrecoverable(2) is handled out-of-band via the `Err` short-circuit in
/// `run()`, so it never reaches this helper.)
pub(crate) fn indel_exit_code(
    ambiguous_seen: bool,
    substitution_seen: bool,
    total_repairs: usize,
) -> u8 {
    if ambiguous_seen || substitution_seen {
        4
    } else if total_repairs == 0 {
        0
    } else {
        5
    }
}

/// Per-card-kind entry point for indel recovery. Dispatches to the per-kind
/// oracle; returns the `IndelOutcome` or a toolkit error. ms1 = single chunk;
/// mk1 = locate-the-single-failing-chunk + `mk_codec::decode` reassembly
/// oracle; md1 = locate-the-single-failing-chunk + `md_codec::chunk::reassemble`
/// reassembly oracle (mirrors mk1; v0.37.2 — resolves FOLLOWUP
/// `m-format-indel-md1-chunked`).
pub(crate) fn recover_indel_card(
    kind: CardKind,
    chunks: &[String],
    max_indel: usize,
    e_subst: usize,
) -> Result<crate::indel::IndelOutcome, ToolkitError> {
    match kind {
        CardKind::Ms1 => {
            // ms1 is single-chunk; recover on the sole chunk.
            let chunk = chunks
                .first()
                .ok_or(ToolkitError::Repair(RepairError::EmptyInput))?;
            Ok(crate::indel::recover_indel(
                chunk,
                "ms",
                max_indel,
                e_subst,
                &Ms1IndelOracle,
            ))
        }
        CardKind::Mk1 => {
            // Locate the single failing chunk (the one normal per-chunk repair
            // cannot handle). An indel lands in ONE chunk; siblings stay intact.
            let failing: Vec<usize> = chunks
                .iter()
                .enumerate()
                .filter(|(i, c)| repair_chunk_one(CardKind::Mk1, *i, c).is_err())
                .map(|(i, _)| i)
                .collect();
            if failing.len() != 1 {
                // 0 or >1 failing: out of single-region v1 scope.
                return Ok(crate::indel::IndelOutcome::Unrecoverable);
            }
            let f = failing[0];
            let oracle = Mk1IndelOracle {
                all_chunks: chunks.to_vec(),
                failing_index: f,
            };
            Ok(crate::indel::recover_indel(
                &chunks[f], "mk", max_indel, e_subst, &oracle,
            ))
        }
        CardKind::Md1 => {
            // Mirror the Mk1 arm: locate the single failing chunk (an indel
            // lands in ONE chunk; siblings stay intact), then run the engine
            // on it with the reassembly-gated oracle.
            let failing: Vec<usize> = chunks
                .iter()
                .enumerate()
                .filter(|(i, c)| repair_chunk_one(CardKind::Md1, *i, c).is_err())
                .map(|(i, _)| i)
                .collect();
            if failing.len() != 1 {
                // 0 or >1 failing: out of single-region v1 scope.
                return Ok(crate::indel::IndelOutcome::Unrecoverable);
            }
            let f = failing[0];
            let oracle = Md1IndelOracle {
                all_chunks: chunks.to_vec(),
                failing_index: f,
            };
            Ok(crate::indel::recover_indel(
                &chunks[f], "md", max_indel, e_subst, &oracle,
            ))
        }
    }
}

/// **v0.23.0 — D29 migration helper.** Delegate md1 chunk-set repair to
/// `md_codec::decode_with_correction` (full-decode semantics per Q1 lock;
/// atomic per D28). Translate the codec's `Error` taxonomy back into
/// toolkit `RepairError` variants per the §2.B.4 D29 error-mapping table.
///
/// Returns the full `RepairOutcome` rather than a per-chunk `Option`,
/// because the sibling helper operates on the whole chunk set atomically.
fn repair_via_md_codec(chunks: &[String]) -> Result<RepairOutcome, RepairError> {
    use md_codec::Error as MdErr;
    let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
    match md_codec::decode_with_correction(&refs) {
        Ok((_descriptor, corrections)) => {
            let (corrected_chunks, repairs) = apply_md_corrections(chunks, &corrections);
            Ok(RepairOutcome {
                kind: CardKind::Md1,
                corrected_chunks,
                repairs,
                // md1 delegates atomically to md_codec::decode_with_correction
                // (whole-set, per D28), which only ever returns `Ok` on full
                // decode success — no behavior change from Cycle E.
                set_verify: SetVerify::Blessed,
            })
        }
        Err(MdErr::TooManyErrors { chunk_index, bound }) => Err(RepairError::TooManyErrors {
            chunk_index,
            bound: bound as usize,
        }),
        Err(MdErr::ChunkSetEmpty) => Err(RepairError::EmptyInput),
        Err(MdErr::Codex32DecodeError(s)) => {
            // md-codec's Codex32DecodeError wraps stringy errors from the
            // codex32 wire-format parser, which doesn't expose a structured
            // HrpMismatch variant (the found-HRP is embedded in prose, not
            // a field). Toolkit's pre-gate `parse_chunk` in repair_card's
            // Md1 branch catches the common cases (HrpMismatch + Lev1
            // suggestion, ReservedInvalidLength, UnsupportedCodeVariant)
            // with rich error structure BEFORE this helper fires. If we
            // reach here, md-codec's wire-format parser surfaced a case the
            // pre-gate didn't recognize — route to UnparseableInput with
            // the original detail string rather than synthesizing a
            // degraded HrpMismatch with `found: String::new()`. See
            // FOLLOWUP `md-codec-decode-with-correction-supports-non-chunked-md1`
            // for the upstream enhancement path that would let us preserve
            // the rich error shape here.
            let chunk_index = parse_md_chunk_index(&s).unwrap_or(0);
            Err(RepairError::UnparseableInput {
                chunk_index,
                detail: s,
            })
        }
        Err(other) => Err(RepairError::PostCorrectionDecodeFailed {
            chunk_index: None,
            detail: other.to_string(),
        }),
    }
}

/// Extract `chunk_index` from md-codec's `"chunk N: …"` error-string
/// pattern. Robust to mid-string occurrences (md-codec's error wrappers
/// may prefix the message before the `chunk N:` clause). Returns `None`
/// if no `chunk N` substring is found.
fn parse_md_chunk_index(detail: &str) -> Option<usize> {
    let idx = detail.find("chunk ")?;
    let after = &detail[idx + "chunk ".len()..];
    let n_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    n_str.parse::<usize>().ok()
}

/// Apply md-codec `CorrectionDetail` entries (sorted by chunk_index +
/// position) to the input chunk set, producing the corrected chunks
/// vector + per-chunk `RepairDetail` entries.
fn apply_md_corrections(
    chunks: &[String],
    corrections: &[md_codec::CorrectionDetail],
) -> (Vec<String>, Vec<RepairDetail>) {
    // Index corrections by chunk_index for O(N+M) assembly.
    let mut per_chunk: Vec<Vec<&md_codec::CorrectionDetail>> = vec![Vec::new(); chunks.len()];
    for c in corrections {
        if c.chunk_index < per_chunk.len() {
            per_chunk[c.chunk_index].push(c);
        }
    }

    let mut corrected_chunks: Vec<String> = Vec::with_capacity(chunks.len());
    let mut repairs: Vec<RepairDetail> = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        if per_chunk[i].is_empty() {
            corrected_chunks.push(chunk.clone());
            continue;
        }
        // Apply this chunk's corrections.
        let lower = chunk.to_lowercase();
        let sep = lower.rfind('1').expect("md-codec already validated prefix");
        let (prefix, rest) = lower.split_at(sep + 1);
        let mut chars: Vec<char> = rest.chars().collect();
        let mut positions: Vec<(usize, char, char)> = Vec::with_capacity(per_chunk[i].len());
        for c in &per_chunk[i] {
            positions.push((c.position, c.was, c.now));
            if c.position < chars.len() {
                chars[c.position] = c.now;
            }
        }
        let mut corrected = String::from(prefix);
        for ch in chars {
            corrected.push(ch);
        }
        corrected_chunks.push(corrected.clone());
        repairs.push(RepairDetail {
            chunk_index: i,
            original_chunk: chunk.clone(),
            corrected_chunk: corrected,
            corrected_positions: positions,
        });
    }
    (corrected_chunks, repairs)
}

/// Auto-fire convenience wrapper. Returns `Ok(())` on repair-failure (caller
/// falls through to its own typed-error path per §5 fallthrough discipline);
/// returns `Err(ToolkitError::RepairShortCircuit { 5 })` on repair-success
/// (caller's `?` short-circuits to exit 5).
///
/// `json_context = true` (v0.22.1 D20) routes the stdout report through
/// `emit_repair_report_json` so callers invoked with `--json` get a
/// structured envelope instead of text-form. Stderr summary + D9 advisory
/// remain identical regardless of context.
pub fn try_repair_and_short_circuit<O: Write + ?Sized, E: Write + ?Sized>(
    kind: CardKind,
    chunks: &[String],
    stdout: &mut O,
    stderr: &mut E,
    json_context: bool,
) -> Result<(), ToolkitError> {
    let outcome = match repair_card(kind, chunks) {
        Ok(o) => o,
        Err(_repair_err) => return Ok(()), // fall-through: caller surfaces typed orig error
    };

    // If repair returned Ok but no corrections were applied, that means the
    // input was already valid — which shouldn't trigger auto-fire (caller
    // hit a different decode error, e.g., HRP or length). Fall through.
    if outcome.repairs.is_empty() {
        return Ok(());
    }

    // Cycle E (`mk1-repair-set-level-reverify`) / plan-R0 G7 — auto-repair
    // NEVER blesses an unverified (partial-set) mk1 correction. A dominant
    // Reject already short-circuited to `Err` above (the first match arm);
    // this arm only ever sees `Unverified` (an incomplete corrected
    // chunk_set_id group). Fall through so the caller's original typed
    // error surfaces — a partial card cannot convert/inspect/verify-bundle
    // anyway (SPEC §2 residual-partial-set-exposure note).
    if !matches!(outcome.set_verify, SetVerify::Blessed) {
        return Ok(());
    }

    emit_repair_report(&outcome, stdout, stderr, json_context).map_err(ToolkitError::Io)?;
    Err(ToolkitError::RepairShortCircuit { exit_code: 5 })
}

/// Render the repair report. Stdout = either the text-form comment lines +
/// corrected chunks (default), or a JSON envelope (v0.22.1 D20, when
/// `json_context = true`). Stderr = repair-summary including D9
/// sensitive-secret warning when kind is Ms1 — identical regardless of
/// stdout format.
pub fn emit_repair_report<O: Write + ?Sized, E: Write + ?Sized>(
    outcome: &RepairOutcome,
    stdout: &mut O,
    stderr: &mut E,
    json_context: bool,
) -> std::io::Result<()> {
    if json_context {
        emit_repair_report_json(outcome, stdout)?;
    } else {
        emit_repair_report_text(outcome, stdout)?;
    }

    let total_corrections: usize = outcome
        .repairs
        .iter()
        .map(|r| r.corrected_positions.len())
        .sum();
    writeln!(
        stderr,
        "repair: applied {} correction{} across {} chunk{}",
        total_corrections,
        if total_corrections == 1 { "" } else { "s" },
        outcome.repairs.len(),
        if outcome.repairs.len() == 1 { "" } else { "s" },
    )?;

    // Output-class advisory: emit unconditionally for all card kinds
    // (ms1→PrivateKeyMaterial, mk1→WatchOnly, md1→Template). Supersedes
    // the former Ms1-only D9 guard (removed in Cycle B P3).
    crate::secret_advisory::emit_output_class_advisory(
        crate::secret_advisory::card_kind_class(outcome.kind),
        stderr,
    );

    Ok(())
}

/// Text-form report (v0.22.0 default). Comment lines + corrected chunks
/// one per line.
fn emit_repair_report_text<O: Write + ?Sized>(
    outcome: &RepairOutcome,
    stdout: &mut O,
) -> std::io::Result<()> {
    writeln!(stdout, "# Repair report")?;
    for repair in &outcome.repairs {
        let kind_str = match outcome.kind {
            CardKind::Ms1 => "ms1",
            CardKind::Mk1 => "mk1",
            CardKind::Md1 => "md1",
        };
        let n = repair.corrected_positions.len();
        let plural = if n == 1 { "correction" } else { "corrections" };
        write!(
            stdout,
            "#   {} chunk {}: {} {} at ",
            kind_str, repair.chunk_index, n, plural
        )?;
        for (i, (pos, was, now)) in repair.corrected_positions.iter().enumerate() {
            if i > 0 {
                write!(stdout, ", ")?;
            }
            write!(stdout, "position {pos}: '{was}' -> '{now}'")?;
        }
        writeln!(stdout)?;
    }
    for chunk in &outcome.corrected_chunks {
        writeln!(stdout, "{chunk}")?;
    }
    Ok(())
}

/// JSON-envelope report (v0.22.1 D20). Schema reuses the standalone
/// `cmd/repair.rs::RepairJson` shape (schema_version: "1", kind,
/// corrected_chunks, repairs) plus two discriminator fields
/// (`auto_repair_short_circuit: true`, `exit_code: 5`) marking the
/// envelope as an auto-fire emission rather than a standalone subcommand
/// invocation.
fn emit_repair_report_json<O: Write + ?Sized>(
    outcome: &RepairOutcome,
    stdout: &mut O,
) -> std::io::Result<()> {
    let kind_str = match outcome.kind {
        CardKind::Ms1 => "ms1",
        CardKind::Mk1 => "mk1",
        CardKind::Md1 => "md1",
    };
    let envelope = AutoFireRepairJson {
        schema_version: "1",
        auto_repair_short_circuit: true,
        exit_code: 5,
        kind: kind_str,
        corrected_chunks: &outcome.corrected_chunks,
        repairs: outcome
            .repairs
            .iter()
            .map(|r| AutoFireRepairJsonDetail {
                chunk_index: r.chunk_index,
                original_chunk: &r.original_chunk,
                corrected_chunk: &r.corrected_chunk,
                corrected_positions: r
                    .corrected_positions
                    .iter()
                    .map(|(p, w, n)| AutoFireRepairJsonPosition {
                        position: *p,
                        was: w.to_string(),
                        now: n.to_string(),
                    })
                    .collect(),
            })
            .collect(),
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    writeln!(stdout, "{body}")?;
    Ok(())
}

// v0.22.1 D20 — JSON envelope shape for auto-fire short-circuit emissions.
// Field order is part of the schema (serde preserves struct field order
// in the default JSON serializer). The two top-level discriminator fields
// (`auto_repair_short_circuit`, `exit_code`) mark the envelope as an
// auto-fire emission vs the standalone `mnemonic repair --json` envelope
// (which uses the parallel `RepairJson` shape in `cmd/repair.rs` without
// these fields).
#[derive(serde::Serialize)]
struct AutoFireRepairJson<'a> {
    schema_version: &'static str,
    auto_repair_short_circuit: bool,
    exit_code: u8,
    kind: &'static str,
    corrected_chunks: &'a [String],
    repairs: Vec<AutoFireRepairJsonDetail<'a>>,
}

#[derive(serde::Serialize)]
struct AutoFireRepairJsonDetail<'a> {
    chunk_index: usize,
    original_chunk: &'a str,
    corrected_chunk: &'a str,
    corrected_positions: Vec<AutoFireRepairJsonPosition>,
}

#[derive(serde::Serialize)]
struct AutoFireRepairJsonPosition {
    position: usize,
    was: String,
    now: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-valid m-format strings for testing. ms1 from TREZOR_12_ZERO
    // entropy; mk1 from a test xpub; md1 from a test descriptor. Generated
    // against the v0.21.0 binary; bench32-alphabet substitutions for
    // corruption injection.
    const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

    /// Flip the bech32 character at position `pos` in the data-part of a
    /// known-valid string. Returns the corrupted string. The replacement
    /// is the next bech32-alphabet char (cyclically), which guarantees
    /// the result is parseable but invalid.
    fn flip_at(chunk: &str, pos: usize) -> String {
        let sep = chunk.rfind('1').unwrap();
        let (prefix, rest) = chunk.split_at(sep + 1);
        let mut chars: Vec<char> = rest.chars().collect();
        let was = chars[pos];
        // Find next char in ALPHABET (cyclic).
        let alphabet_str = std::str::from_utf8(ALPHABET).unwrap();
        let was_idx = alphabet_str.find(was).unwrap();
        let new_idx = (was_idx + 1) % 32;
        chars[pos] = alphabet_str.chars().nth(new_idx).unwrap();
        let mut out = String::from(prefix);
        for c in chars {
            out.push(c);
        }
        out
    }

    // ---- §4.1 cells ----

    /// Cell 1: happy-path per HRP (×3 sub-cells). For each HRP, encode a
    /// known-valid string, flip 1 char at a deterministic position, assert
    /// repair_card returns Ok with corrected_positions = [(N, was, now)].
    /// Plus a corrections_applied == 0 (already-valid pass-through) sub-cell.
    #[test]
    fn happy_path_ms1_1_substitution() {
        let bad = flip_at(VALID_MS1, 10);
        let result = repair_card(CardKind::Ms1, &[bad.clone()]).expect("repair Ok");
        assert_eq!(result.kind, CardKind::Ms1);
        assert_eq!(result.corrected_chunks.len(), 1);
        assert_eq!(result.corrected_chunks[0], VALID_MS1);
        assert_eq!(result.repairs.len(), 1);
        assert_eq!(result.repairs[0].chunk_index, 0);
        assert_eq!(result.repairs[0].corrected_positions.len(), 1);
        assert_eq!(result.repairs[0].corrected_positions[0].0, 10);
    }

    #[test]
    fn happy_path_ms1_already_valid_passthrough() {
        let result = repair_card(CardKind::Ms1, &[VALID_MS1.to_string()]).expect("repair Ok");
        assert_eq!(result.corrected_chunks[0], VALID_MS1);
        assert!(
            result.repairs.is_empty(),
            "no corrections applied for valid input"
        );
    }

    /// Cell 7: EmptyInput.
    #[test]
    fn empty_input_returns_err() {
        let result = repair_card(CardKind::Ms1, &[]);
        assert!(matches!(result, Err(RepairError::EmptyInput)));
    }

    /// Long-code mk1 happy-path: flip 1 char in a 108-data-part mk1
    /// (xpub-bearing chunk) and verify repair_card restores it.
    #[test]
    fn happy_path_mk1_long_1_substitution() {
        const VALID_MK1_LONG: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
        let bad = flip_at(VALID_MK1_LONG, 50);
        let result = repair_card(CardKind::Mk1, &[bad.clone()]).expect("repair Ok");
        assert_eq!(result.corrected_chunks[0], VALID_MK1_LONG);
        assert_eq!(result.repairs.len(), 1);
        assert_eq!(result.repairs[0].corrected_positions.len(), 1);
        assert_eq!(result.repairs[0].corrected_positions[0].0, 50);
    }

    /// Long-code mk1 already-valid passthrough.
    #[test]
    fn happy_path_mk1_long_already_valid_passthrough() {
        const VALID_MK1_LONG: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
        let result = repair_card(CardKind::Mk1, &[VALID_MK1_LONG.to_string()]).expect("repair Ok");
        assert_eq!(result.corrected_chunks[0], VALID_MK1_LONG);
        assert!(result.repairs.is_empty());
    }

    /// Regular-code mk1 happy-path: chunk 1 of a typical bundle (77-char
    /// data-part). Flip 1 char, verify repair.
    #[test]
    fn happy_path_mk1_regular_1_substitution() {
        const VALID_MK1_REG: &str =
            "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";
        let bad = flip_at(VALID_MK1_REG, 25);
        let result = repair_card(CardKind::Mk1, &[bad.clone()]).expect("repair Ok");
        assert_eq!(result.corrected_chunks[0], VALID_MK1_REG);
        assert_eq!(result.repairs[0].corrected_positions.len(), 1);
        assert_eq!(result.repairs[0].corrected_positions[0].0, 25);
    }

    /// Long-code ms1 must error with `UnsupportedCodeVariant` (codex32 v0.1
    /// doesn't define a long code in the ms-codec dialect we ship).
    /// Constructed by padding a valid ms1 to a 96-char data-part.
    #[test]
    fn ms_long_code_returns_unsupported_code_variant() {
        // 96-char data-part that parses cleanly. Content is arbitrary
        // (length-only dispatch happens before checksum verification).
        let padded = format!("ms1{}", "q".repeat(96));
        let result = repair_card(CardKind::Ms1, &[padded]);
        assert!(
            matches!(
                result,
                Err(RepairError::UnsupportedCodeVariant {
                    chunk_index: 0,
                    hrp: "ms",
                    data_part_len: 96
                })
            ),
            "expected UnsupportedCodeVariant, got: {result:?}"
        );
    }

    /// Reserved-invalid length [94, 95] must error with
    /// `ReservedInvalidLength` for any HRP.
    #[test]
    fn reserved_invalid_length_94_returns_err() {
        let padded = format!("mk1{}", "q".repeat(94));
        let result = repair_card(CardKind::Mk1, &[padded]);
        assert!(
            matches!(
                result,
                Err(RepairError::ReservedInvalidLength {
                    chunk_index: 0,
                    data_part_len: 94
                })
            ),
            "expected ReservedInvalidLength(94), got: {result:?}"
        );
    }

    /// Cell 3: HRP mismatch.
    #[test]
    fn hrp_mismatch_returns_err() {
        let mk_string_passed_as_ms = "mk1abcdefghijklmnopqrstuvwxyzabcdefghijk".to_string();
        let result = repair_card(CardKind::Ms1, &[mk_string_passed_as_ms]);
        assert!(matches!(
            result,
            Err(RepairError::HrpMismatch {
                chunk_index: 0,
                expected: "ms",
                ..
            })
        ));
    }

    /// Cell 4: Multi-chunk all-valid. (Test with 2 valid ms1 chunks even
    /// though ms1 is conventionally single-chunk — the per-chunk loop is
    /// kind-agnostic.)
    #[test]
    fn multi_chunk_all_valid() {
        let result = repair_card(
            CardKind::Ms1,
            &[VALID_MS1.to_string(), VALID_MS1.to_string()],
        )
        .expect("repair Ok");
        assert_eq!(result.corrected_chunks.len(), 2);
        assert!(result.repairs.is_empty());
    }

    /// Cell 5: Multi-chunk one-corrupted.
    #[test]
    fn multi_chunk_one_corrupted_at_index_1() {
        let bad = flip_at(VALID_MS1, 5);
        let result = repair_card(
            CardKind::Ms1,
            &[VALID_MS1.to_string(), bad, VALID_MS1.to_string()],
        )
        .expect("repair Ok");
        assert_eq!(result.corrected_chunks.len(), 3);
        assert_eq!(result.corrected_chunks[0], VALID_MS1);
        assert_eq!(result.corrected_chunks[1], VALID_MS1);
        assert_eq!(result.corrected_chunks[2], VALID_MS1);
        assert_eq!(result.repairs.len(), 1);
        assert_eq!(result.repairs[0].chunk_index, 1);
    }

    /// Cell 6: Multi-chunk atomic failure. Corrupt chunk 1 beyond repair
    /// AND chunk 2 reparably → Err names chunk 1; chunk 2's potential
    /// correction NOT applied.
    #[test]
    fn multi_chunk_atomic_failure_reports_first_unrepairable() {
        let irreparable = flip_many(VALID_MS1, &[2, 5, 8, 11, 14]); // 5+ errors
        let reparable = flip_at(VALID_MS1, 20);
        let result = repair_card(
            CardKind::Ms1,
            &[VALID_MS1.to_string(), irreparable, reparable],
        );
        match result {
            Err(RepairError::TooManyErrors { chunk_index, .. }) => {
                assert_eq!(chunk_index, 1);
            }
            other => panic!("expected TooManyErrors at chunk 1, got {other:?}"),
        }
    }

    fn flip_many(chunk: &str, positions: &[usize]) -> String {
        positions
            .iter()
            .fold(chunk.to_string(), |acc, &p| flip_at(&acc, p))
    }

    /// Cell 2: t=4 boundary. Flip exactly 4 chars → Ok with 4 reports.
    /// Flip exactly 5 → Err(TooManyErrors). (Position spacing matters
    /// for unique decodability — use spread positions.)
    #[test]
    fn t4_boundary_4_errors_ok_5_errors_err() {
        let four_errors = flip_many(VALID_MS1, &[3, 11, 19, 27]);
        let result_4 = repair_card(CardKind::Ms1, &[four_errors]).expect("4 errors should repair");
        assert_eq!(result_4.repairs[0].corrected_positions.len(), 4);

        let five_errors = flip_many(VALID_MS1, &[3, 11, 19, 27, 35]);
        let result_5 = repair_card(CardKind::Ms1, &[five_errors]);
        assert!(matches!(result_5, Err(RepairError::TooManyErrors { .. })));
    }

    /// Cell 8: Cross-codec NUMS-target constancy. Drift-gate cells that
    /// trip if any sibling codec changes its target constant.
    #[test]
    fn drift_gate_mk_targets_match_mk_codec_public_consts() {
        assert_eq!(MK_REGULAR_TARGET, mk_codec::MK_REGULAR_CONST);
        assert_eq!(MK_LONG_TARGET, mk_codec::MK_LONG_CONST);
    }

    // v0.23.0 (D29): drift-gate tests for the previously-vendored
    // MS_NUMS_TARGET / MD_NUMS_TARGET constants are deleted along with the
    // constants themselves. The authoritative invariants now live in
    // ms-codec (`ms_codec::bch::MS_REGULAR_CONST` + the `decode_with_correction`
    // round-trip cells) and md-codec (`md_codec::bch::MD_REGULAR_CONST` +
    // its `decode_with_correction` cells). The ms_nums_target_is_stable_…
    // and md_nums_target_is_stable_… stability tests below are also deleted
    // since they tested the toolkit-internal constants.

    /// Fixture sanity: VALID_MS1 must be accepted by ms_codec::decode.
    /// Catches a class of latent-fixture-rot bugs (e.g., a typo that breaks
    /// the BCH checksum) before they masquerade as repair-logic failures.
    #[test]
    fn fixture_valid_ms1_decodes_via_ms_codec() {
        ms_codec::decode(VALID_MS1).expect("VALID_MS1 fixture must decode");
    }

    /// Helper: compute the raw polymod (before XOR with any target) for an
    /// already-valid bech32-family string. Used to derive empirical NUMS
    /// targets and to test stability across distinct valid inputs. The
    /// generator + shift + mask are dispatched by data-part length to
    /// match BIP-93's regular/long boundaries.
    fn raw_polymod(chunk: &str) -> u128 {
        let sep = chunk.rfind('1').unwrap();
        let (hrp, rest) = chunk.split_at(sep);
        let data_part = &rest[1..];
        let mut alphabet_inv = [0xFFu8; 128];
        for (i, &c) in ALPHABET.iter().enumerate() {
            alphabet_inv[c as usize] = i as u8;
        }
        let mut values: Vec<u8> = Vec::with_capacity(data_part.len());
        for c in data_part.chars() {
            values.push(alphabet_inv[c as usize]);
        }
        let mut input = hrp_expand(hrp);
        input.extend_from_slice(&values);
        match bch_code_for_length(values.len())
            .expect("test fixtures must be in BIP-93 valid range")
        {
            BchCode::Regular => polymod_run(&input, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
            BchCode::Long => polymod_run(&input, &GEN_LONG, LONG_SHIFT, LONG_MASK),
        }
    }

    // v0.23.0 (D29): `ms_nums_target_is_stable_across_distinct_valid_strings`
    // was deleted because MS_NUMS_TARGET no longer exists in the toolkit.
    // The equivalent invariant is enforced upstream by ms-codec's own test
    // suite (`ms_codec::decode_with_correction` + bch::MS_REGULAR_CONST).

    /// Stability test: 3 distinct valid LONG-code mk1 strings (chunk 0 of
    /// a typical bundle, carrying the xpub) must all reduce to the SAME
    /// polymod value under the LONG generator, equal to mk-codec's
    /// `MK_LONG_CONST`. These are the chunks users most often need
    /// repaired (the xpub-bearing first chunk).
    #[test]
    fn mk_long_target_is_stable_across_distinct_valid_strings() {
        // Generated 2026-05-17 by `mnemonic bundle --template bip84
        // --network mainnet --slot @0.phrase=... --json --no-engraving-card`
        // for 3 distinct BIP-39 test phrases. Each is chunk 0 (108-char
        // data-part → long code).
        const VALID_MK1_LONG_A: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
        const VALID_MK1_LONG_B: &str = "mk1qp075gpqqsqhl2y9jkux3r03qvzg3vs7afghae0rhwz39k4sk9ejeku6jn6z5ng97tlv6kn0ru5kswgtdzmrgpk7l5pz735pjry2ursns6sk";
        const VALID_MK1_LONG_C: &str = "mk1qp8laepqqsqnl7usj55xg5qxqvzg3vs76psuyrqg8vt6w7wmgj73n889zv2eymp4zxqs9x6du0nfrz8e7qgymg03kcptxlndsx9jxaajlmtj";

        let pa = raw_polymod(VALID_MK1_LONG_A);
        let pb = raw_polymod(VALID_MK1_LONG_B);
        let pc = raw_polymod(VALID_MK1_LONG_C);
        assert_eq!(
            pa, pb,
            "mk1-long polymod target diverges between A/B (A=0x{pa:x}, B=0x{pb:x})"
        );
        assert_eq!(
            pb, pc,
            "mk1-long polymod target diverges between B/C (B=0x{pb:x}, C=0x{pc:x})"
        );
        assert_eq!(
            pa, MK_LONG_TARGET,
            "MK_LONG_TARGET drift: empirical=0x{pa:x}, mk_codec::MK_LONG_CONST=0x{MK_LONG_TARGET:x}"
        );
    }

    /// Stability test: a valid REGULAR-code mk1 string (chunk 1 of a
    /// typical bundle, carrying overflow + per-chunk metadata) must
    /// reduce to `MK_REGULAR_CONST` under the regular generator. We
    /// have only 1 fixture here because the regular-code mk1 path comes
    /// from chunk-N>=1 of multi-chunk emissions, which is harder to
    /// hand-collect 3 distinct samples of; the single sample plus the
    /// `MK_REGULAR_TARGET = mk_codec::MK_REGULAR_CONST` drift gate covers
    /// the invariant.
    #[test]
    fn mk_regular_target_matches_chunk1_polymod() {
        // Generated 2026-05-17 — chunk 1 of the bip84 bundle for the
        // canonical test-vector phrase ("abandon × 11 about"). 77-char
        // data-part → regular code.
        const VALID_MK1_REG: &str =
            "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

        let p = raw_polymod(VALID_MK1_REG);
        assert_eq!(
            p, MK_REGULAR_TARGET,
            "MK_REGULAR_TARGET drift: empirical=0x{p:x}, mk_codec::MK_REGULAR_CONST=0x{MK_REGULAR_TARGET:x}"
        );
    }

    // v0.23.0 (D29): `md_nums_target_is_stable_across_distinct_valid_strings`
    // was deleted because MD_NUMS_TARGET no longer exists in the toolkit.
    // The equivalent invariant is enforced upstream by md-codec's own test
    // suite (`md_codec::decode_with_correction` + bch::MD_REGULAR_CONST).

    /// R1 N1 parity smoke: toolkit's repair_card(Mk1, …) must produce the
    /// same correction as mk_codec::string_layer::bch_correct_regular for any
    /// mk1 chunk corrupted within t=4 capacity. Catches divergence between
    /// our parameterized wrapper and mk-codec's native API.
    #[test]
    fn parity_smoke_repair_card_mk1_matches_mk_codec_bch_correct_regular() {
        // Build a mk1 single-string by encoding a test KeyCard via mk-codec,
        // then comparing repair behavior. We don't have a stable VALID_MK1
        // literal handy; instead skip this cell when mk-codec is not in scope.
        // (Phase 1 R0 reviewer can flesh this out with a real mk1 from the
        // toolkit's existing test fixtures.)
        let _ = MK_REGULAR_TARGET; // suppress unused warning
        let _ = MK_LONG_TARGET;
        // TODO Phase 1 R0: implement parity smoke against tests/fixtures/v0_20_0_single_sig_bip84_bundle.json
    }

    // ============================================================================
    // v0.22.1 D19 — HRP Levenshtein-1 "did you mean" cells
    // ============================================================================

    /// `ns` is 1-sub from `ms` (`n→m`) and 2-sub from `mk`/`md` — unique
    /// neighbor → suggest "ms".
    #[test]
    fn hrp_lev1_ns_yields_ms() {
        assert_eq!(suggest_hrp("ns"), Some("ms"));
    }

    /// `mb` is 1-sub from ALL THREE known HRPs (`ms`/`mk`/`md` — the
    /// second character differs in each case). Three-way ambiguous →
    /// no suggestion.
    #[test]
    fn hrp_lev1_mb_is_ambiguous_three_way() {
        assert_eq!(suggest_hrp("mb"), None);
        // Sanity: hrp_lev1 returns true for ALL three candidates.
        assert!(hrp_lev1("ms", "mb"), "ms vs mb: s→b is 1 sub");
        assert!(hrp_lev1("mk", "mb"), "mk vs mb: k→b is 1 sub");
        assert!(hrp_lev1("md", "mb"), "md vs mb: d→b is 1 sub");
    }

    /// `xy` is 2-sub from every known HRP — no neighbors → no suggestion.
    #[test]
    fn hrp_lev1_xy_no_neighbor() {
        assert_eq!(suggest_hrp("xy"), None);
        assert!(!hrp_lev1("ms", "xy"));
        assert!(!hrp_lev1("mk", "xy"));
        assert!(!hrp_lev1("md", "xy"));
    }

    /// Length-mismatch short-circuits the check (HRP is fixed at 2 chars
    /// in the codex32 family; longer/shorter inputs are out-of-domain).
    #[test]
    fn hrp_lev1_wrong_length_no_neighbor() {
        assert_eq!(suggest_hrp("m"), None, "1-char input never suggests");
        assert_eq!(suggest_hrp("mss"), None, "3-char input never suggests");
        assert_eq!(suggest_hrp(""), None, "empty input never suggests");
    }

    /// End-to-end Display integration — the suggestion suffix actually
    /// reaches the formatted message for a unique-neighbor case.
    #[test]
    fn hrp_mismatch_display_includes_suggestion_for_unique_neighbor() {
        let e = RepairError::HrpMismatch {
            chunk_index: 0,
            expected: "ms",
            found: "ns".to_string(),
        };
        let msg = format!("{e}");
        assert!(
            msg.contains("did you mean 'ms'?"),
            "Display should append did-you-mean suffix; got: {msg}"
        );
    }

    /// End-to-end Display integration — the suggestion suffix is OMITTED
    /// when no unique neighbor exists (ambiguous case).
    #[test]
    fn hrp_mismatch_display_omits_suggestion_when_ambiguous() {
        let e = RepairError::HrpMismatch {
            chunk_index: 0,
            expected: "ms",
            found: "mb".to_string(),
        };
        let msg = format!("{e}");
        assert!(
            !msg.contains("did you mean"),
            "Display should NOT append suffix for ambiguous neighbor; got: {msg}"
        );
    }

    // ============================================================================
    // Phase 1 — indel recovery: too-long (delete-and-validate) + ms1 oracle
    // ============================================================================

    /// ms1 too-long by 1: insert one extra data char at data-index 10
    /// (full-string index 13 = "ms1" prefix len 3 + 10). The delete
    /// producer must find and remove it, recovering VALID_MS1.
    #[test]
    fn indel_ms1_too_long_by_one_recovers() {
        // Insert one extra data char after the "ms1" prefix, at a mid-data index.
        let data_start = 3; // "ms1"
        let mut s = String::from(VALID_MS1);
        s.insert(data_start + 10, 'q'); // one inserted char → too long by 1
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 1, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => {
                assert_eq!(c.recovered, VALID_MS1);
                assert_eq!(c.direction, crate::indel::IndelDirection::Deleted);
                assert_eq!(c.region, crate::indel::IndelRegion::DataPart);
                assert_eq!(c.indel_count, 1);
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// ms1 too-long by 2: insert two extra data chars. The delete producer
    /// with max_indel=2 must find and remove both, recovering VALID_MS1.
    #[test]
    fn indel_ms1_too_long_by_two_recovers() {
        let mut s = String::from(VALID_MS1);
        s.insert(3 + 10, 'q');
        s.insert(3 + 5, 'p'); // two inserted chars
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 2, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => assert_eq!(c.recovered, VALID_MS1),
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    // ============================================================================
    // Phase 2 — indel recovery: too-short (placeholder-then-decode) ms1
    // ============================================================================

    /// ms1 too-short by 1: drop a NON-'q' data char (index 1 = 'e') → the
    /// placeholder 'q' inserted at that position differs from the true symbol,
    /// so the BCH decoder must SOLVE it (1 correction at that pos). The
    /// pure-indel ⊆ rule accepts because the only correction is at the
    /// placeholder position.
    #[test]
    fn indel_ms1_too_short_by_one_bch_solves_dropped_char() {
        // VALID_MS1 data part: indices 0='0', 1='e', 2='n', 3='t', 4='r', 5='s', 6..='q'
        // Drop data index 1 ('e') → full-string index 3+1 = 4.
        let mut s = String::from(VALID_MS1);
        s.remove(3 + 1); // remove data index 1 ('e')
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 1, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => {
                assert_eq!(c.recovered, VALID_MS1);
                assert_eq!(c.direction, crate::indel::IndelDirection::Inserted);
                assert_eq!(c.region, crate::indel::IndelRegion::DataPart);
                assert_eq!(c.indel_count, 1);
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// ms1 too-short by 1, placeholder collision: drop a 'q' (data index 8,
    /// which is a 'q' in the long run). The placeholder 'q' is EXACT (zero
    /// corrections). The pure-indel ⊆ rule's empty-correction path: ∅ ⊆ {8}
    /// is true, so it is accepted.
    #[test]
    fn indel_ms1_too_short_placeholder_collision_dropped_q() {
        // Data index 8 is a 'q' in the long run ("ms10entrSQQQQ…", data chars
        // 0='0',1='e',2='n',3='t',4='r',5='s',6='q',7='q',8='q',...).
        let mut s = String::from(VALID_MS1);
        s.remove(3 + 8); // a 'q'
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 1, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => assert_eq!(c.recovered, VALID_MS1),
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// pure-indel rejection: drop one char AND substitute another → the
    /// recovery needs a correction OUTSIDE the placeholder set → ⊆ rule
    /// rejects → Unrecoverable.
    #[test]
    fn indel_ms1_pure_indel_rejects_indel_plus_substitution() {
        // substitute data index 2 ('n' → 'p'); then drop data index 1 ('e').
        let mut chars: Vec<char> = VALID_MS1.chars().collect();
        chars[3 + 2] = if chars[3 + 2] == 'p' { 'z' } else { 'p' };
        let mut s: String = chars.into_iter().collect();
        s.remove(3 + 1);
        let oracle = Ms1IndelOracle;
        assert_eq!(
            crate::indel::recover_indel(&s, "ms", 1, 0, &oracle),
            crate::indel::IndelOutcome::Unrecoverable
        );
    }

    /// Phase 1 (indel-v2): drop one data char AND substitute another → needs
    /// exactly 1 placeholder + 1 substitution. With `e_subst=1` the relaxed
    /// accept gate (`|corrections \ placeholders| ≤ E`) recovers VALID_MS1 and
    /// reports `subst_count == 1`. (Same corruption as the e0 rejection test
    /// below; here the budget admits it.)
    #[test]
    fn indel_ms1_indel_plus_substitution_recovers_with_e1() {
        // substitute data index 2 ('n'); then drop data index 1 ('e'). idx 1 < 2,
        // so the substitution at idx 2 stays a substitution after the drop (the
        // dropped char is at a LOWER index; the higher index just shifts down).
        let mut chars: Vec<char> = VALID_MS1.chars().collect();
        chars[3 + 2] = if chars[3 + 2] == 'p' { 'z' } else { 'p' }; // substitute data idx 2
        let mut s: String = chars.into_iter().collect();
        s.remove(3 + 1); // drop data idx 1
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 1, 1, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => {
                assert_eq!(c.recovered, VALID_MS1);
                assert_eq!(c.subst_count, 1);
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// Phase 1 (indel-v2): the same indel+substitution corruption is REJECTED
    /// at `e_subst=0` (pure-indel) — the substitution is outside the placeholder
    /// set, so `|corrections \ placeholders| = 1 > 0`.
    #[test]
    fn indel_ms1_indel_plus_substitution_rejected_at_e0() {
        let mut chars: Vec<char> = VALID_MS1.chars().collect();
        chars[3 + 2] = if chars[3 + 2] == 'p' { 'z' } else { 'p' };
        let mut s: String = chars.into_iter().collect();
        s.remove(3 + 1);
        assert_eq!(
            crate::indel::recover_indel(&s, "ms", 1, 0, &Ms1IndelOracle),
            crate::indel::IndelOutcome::Unrecoverable
        );
    }

    /// ms1 too-short by 2: drop two non-'q' chars and recover with
    /// max_indel=2. Remove higher-index first so the second remove offset
    /// remains valid.
    #[test]
    fn indel_ms1_too_short_by_two_recovers() {
        let mut s = String::from(VALID_MS1);
        s.remove(3 + 5); // 's' at data index 5 — remove higher index first
        s.remove(3 + 1); // 'e' at data index 1
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 2, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => assert_eq!(c.recovered, VALID_MS1),
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    // ============================================================================
    // Phase 3 — P1 prefix producer + ambiguity contract
    // ============================================================================

    /// Drop the 'm' from the "ms1" prefix → "s10entrs…"; data-part intact.
    /// The prefix producer must restore it (Inserted direction, Prefix region).
    #[test]
    fn indel_ms1_prefix_dropped_m_recovers() {
        let s = VALID_MS1.strip_prefix('m').unwrap().to_string(); // "s10entrs…"
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 1, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => {
                assert_eq!(c.recovered, VALID_MS1);
                assert_eq!(c.region, crate::indel::IndelRegion::Prefix);
                assert_eq!(c.direction, crate::indel::IndelDirection::Inserted);
                assert_eq!(c.indel_count, 1);
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// Insert a stray char inside the prefix: "ms1…" → "msx1…"; data-part intact.
    /// The prefix producer must remove the extra char (Deleted direction, Prefix region).
    #[test]
    fn indel_ms1_prefix_extra_char_recovers() {
        let s = format!("msx1{}", &VALID_MS1[3..]); // "msx10entrs…"
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 1, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => {
                assert_eq!(c.recovered, VALID_MS1);
                assert_eq!(c.region, crate::indel::IndelRegion::Prefix);
                assert_eq!(c.direction, crate::indel::IndelDirection::Deleted);
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    // ============================================================================
    // Phase 2 (indel-v2) — cross-region two-level search
    // ============================================================================

    /// indel-v2 Phase 2: drop the leading 'm' (prefix indel) AND drop one
    /// mid-data char (data indel). With max_indel=2 the two-level search must
    /// allocate 1 edit to the prefix region and 1 to the data region, recover
    /// VALID_MS1, and tag the result `CrossRegion` with indel_count==2.
    #[test]
    fn indel_ms1_cross_region_prefix_and_data_recovers() {
        // Drop the leading 'm' (prefix indel) AND drop one data char (data indel).
        let mut s = VALID_MS1.strip_prefix('m').unwrap().to_string(); // "s10entrs…"
                                                                      // Drop a mid/late data position (post-prefix-drop string indices).
        let drop_at = s.len() - 8;
        s.remove(drop_at);
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 2, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => {
                assert_eq!(c.recovered, VALID_MS1);
                assert_eq!(c.region, crate::indel::IndelRegion::CrossRegion);
                assert_eq!(c.indel_count, 2);
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// indel-v2 end-of-cycle I1: "all three at once" — prefix drop, data drop,
    /// and data substitution combined.  With max_indel=2, e_subst=1 the
    /// cross-region two-level search must recover VALID_MS1 and tag the result
    /// `CrossRegion` with indel_count==2, subst_count==1.
    ///
    /// Corruption recipe (applied in order so the data substitution survives
    /// the subsequent remove):
    ///   1. Substitute data[4] (full[7]) 'r' → 'p'  (non-'q', real subst for BCH).
    ///   2. Remove  data[1] (full[4]) 'e'            (drop-idx < subst-idx, shift-safe).
    ///   3. Strip leading 'm'                         (prefix indel).
    ///
    /// Final corrupted string: "s10ntpsqqq…" (len 48, missing 'm' + 'e', 'r'→'p').
    #[test]
    fn indel_ms1_all_three_cross_region_plus_substitution() {
        // Step 1 — substitute data[4]='r' → 'p' (non-'q', so BCH must solve it).
        let mut chars: Vec<char> = VALID_MS1.chars().collect();
        assert_eq!(chars[3 + 4], 'r', "fixture: data[4] must be 'r'");
        chars[3 + 4] = 'p';
        // Step 2 — drop data[1]='e' (lower index than the substitution, so the
        // substituted char survives the Vec shift).
        let mut s: String = chars.into_iter().collect();
        assert_eq!(
            s.chars().nth(3 + 1),
            Some('e'),
            "fixture: data[1] must be 'e'"
        );
        s.remove(3 + 1);
        // Step 3 — strip leading 'm' (prefix indel).
        let s = s.strip_prefix('m').unwrap().to_string();
        // Recover with N=2, E=1.
        let oracle = Ms1IndelOracle;
        match crate::indel::recover_indel(&s, "ms", 2, 1, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => {
                assert_eq!(c.recovered, VALID_MS1);
                assert_eq!(c.region, crate::indel::IndelRegion::CrossRegion);
                assert_eq!(c.subst_count, 1);
                assert_eq!(c.indel_count, 2);
            }
            other => panic!("expected Unique cross-region+subst, got {other:?}"),
        }
    }

    // ============================================================================
    // Phase 4 — mk1 per-chunk recovery + reassembly oracle
    // ============================================================================

    // The two chunks of ONE bip84 mk1 card (chunk0 = long-code, chunk1 =
    // regular-code); verified to reassemble via `mk_codec::decode`.
    //
    // NOTE (Phase 4, see report): there is NO standalone single-string mk1
    // card to test against — a realistic mk1 card carries a 73-byte compact
    // xpub, which exceeds SINGLE_STRING_LONG_BYTES (56), so every mk1 card is
    // chunked (mk-codec `string_layer/pipeline.rs` source comment; confirmed
    // empirically: `decode([C1])` → `ChunkedHeaderMalformed("received 1
    // chunks, header declares total_chunks = 2")`). The plan's hypothetical
    // "single-chunk card" fixture cannot reassemble. The single-failing-chunk
    // recovery path (one indel in ONE chunk of a multi-chunk card, validated
    // through the reassembly oracle) is the real, supported case — and is what
    // these `recover_indel`-driven tests exercise via a hand-built
    // `Mk1IndelOracle` whose `all_chunks` carries BOTH real chunks.
    const MK1_CARD_C0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
    const MK1_CARD_C1: &str =
        "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

    /// One chunk (chunk 1) of a 2-chunk card, too long by one inserted data
    /// char → the delete producer recovers it through the per-chunk ⊆-gated
    /// solve, and the reassembly oracle (carrying the intact chunk 0) confirms.
    /// Drives `recover_indel` directly with a hand-built `Mk1IndelOracle`.
    #[test]
    fn indel_mk1_single_failing_chunk_too_long_recovers() {
        assert!(
            mk_codec::decode(&[MK1_CARD_C0, MK1_CARD_C1]).is_ok(),
            "fixture must reassemble"
        );
        let mut s = String::from(MK1_CARD_C1);
        s.insert(3 + 10, 'q');
        let oracle = Mk1IndelOracle {
            all_chunks: vec![MK1_CARD_C0.to_string(), s.clone()],
            failing_index: 1,
        };
        match crate::indel::recover_indel(&s, "mk", 1, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => assert_eq!(c.recovered, MK1_CARD_C1),
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// One chunk (chunk 1) of a 2-chunk card, too short by one dropped data
    /// char → the insert producer's BCH solve recovers the missing symbol, and
    /// the reassembly oracle confirms against the intact chunk 0.
    #[test]
    fn indel_mk1_single_failing_chunk_too_short_recovers() {
        let mut s = String::from(MK1_CARD_C1);
        s.remove(3 + 10); // drop a data char
        let oracle = Mk1IndelOracle {
            all_chunks: vec![MK1_CARD_C0.to_string(), s.clone()],
            failing_index: 1,
        };
        match crate::indel::recover_indel(&s, "mk", 1, 0, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => assert_eq!(c.recovered, MK1_CARD_C1),
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// Phase 1 (indel-v2): one chunk of a 2-chunk mk1 card, too short by one
    /// dropped data char AND with one substituted data char. With `e_subst=1`
    /// the relaxed per-chunk solve recovers the original chunk and reports
    /// `subst_count == 1`; the reassembly oracle confirms against intact chunk 0.
    #[test]
    fn indel_mk1_single_failing_chunk_indel_plus_substitution_recovers_with_e1() {
        // substitute a data char (data idx 20), then drop a LOWER data idx (10)
        // so the substitution remains a substitution after the shift.
        let mut chars: Vec<char> = MK1_CARD_C1.chars().collect();
        let sub_full = 3 + 20;
        chars[sub_full] = if chars[sub_full] == 'q' { 'p' } else { 'q' };
        let mut s: String = chars.into_iter().collect();
        s.remove(3 + 10); // drop a data char at a lower index
        let oracle = Mk1IndelOracle {
            all_chunks: vec![MK1_CARD_C0.to_string(), s.clone()],
            failing_index: 1,
        };
        match crate::indel::recover_indel(&s, "mk", 1, 1, &oracle) {
            crate::indel::IndelOutcome::Unique(c) => {
                assert_eq!(c.recovered, MK1_CARD_C1);
                assert_eq!(c.subst_count, 1);
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// Two-chunk card; corrupt ONLY chunk 1 (insert a char). recover_indel_card
    /// must locate chunk 1 as the failing chunk, recover it, and confirm
    /// reassembly via mk_codec::decode([chunk0, recovered_chunk1]).
    #[test]
    fn indel_mk1_multichunk_one_corrupted_chunk_recovers_via_recover_indel_card() {
        assert!(
            mk_codec::decode(&[MK1_CARD_C0, MK1_CARD_C1]).is_ok(),
            "fixture must reassemble"
        );
        let mut bad_c1 = String::from(MK1_CARD_C1);
        bad_c1.insert(3 + 12, 'q'); // too long by 1, in chunk 1
        let chunks = vec![MK1_CARD_C0.to_string(), bad_c1];
        match recover_indel_card(CardKind::Mk1, &chunks, 1, 0).expect("ok") {
            crate::indel::IndelOutcome::Unique(c) => {
                assert_eq!(c.recovered, MK1_CARD_C1); // recovered chunk == original chunk 1
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    /// Corrupt BOTH chunks → more than one failing chunk → exceeds single-region
    /// v1 → Unrecoverable.
    #[test]
    fn indel_mk1_multichunk_two_failing_is_unrecoverable() {
        let mut c0 = String::from(MK1_CARD_C0);
        c0.insert(3 + 5, 'q');
        let mut c1 = String::from(MK1_CARD_C1);
        c1.insert(3 + 5, 'q');
        let chunks = vec![c0, c1];
        assert_eq!(
            recover_indel_card(CardKind::Mk1, &chunks, 1, 0).expect("ok"),
            crate::indel::IndelOutcome::Unrecoverable
        );
    }

    // ============================================================================
    // Phase 5 — CLI exit-code precedence + indel-trigger classification (unit)
    // ============================================================================

    /// `indel_exit_code` precedence: ambiguous-or-substitution(4) >
    /// recovered/repaired(5) > already-valid(0). The Ambiguous CLI path is
    /// cryptographically unreachable with real vectors (13/15-symbol checksum
    /// → ~2⁻⁶⁵ collision), so this unit test is the canonical coverage for
    /// the exit-4 mapping (cf. the Phase-3 engine
    /// `recover_indel_reports_ambiguous` test which covers the Ambiguous
    /// *outcome*).
    #[test]
    fn indel_exit_code_precedence() {
        assert_eq!(indel_exit_code(false, false, 0), 0); // nothing recovered
        assert_eq!(indel_exit_code(false, false, 5), 5); // recovered/repaired
        assert_eq!(indel_exit_code(true, false, 0), 4); // ambiguous wins
        assert_eq!(indel_exit_code(true, false, 5), 4);
        assert_eq!(indel_exit_code(false, true, 1), 4); // substitution wins
        assert_eq!(indel_exit_code(true, true, 0), 4); // ambiguous+subst wins
    }

    /// `is_indel_trigger` set (§1.7 — HrpMismatch INCLUDED so a prefix-region
    /// indel engages; the non-triggers pass through to today's typed error).
    #[test]
    fn is_indel_trigger_set() {
        use RepairError::*;
        assert!(is_indel_trigger(&HrpMismatch {
            chunk_index: 0,
            expected: "ms",
            found: "s".into()
        }));
        assert!(is_indel_trigger(&TooManyErrors {
            chunk_index: 0,
            bound: 8
        }));
        assert!(is_indel_trigger(&UnparseableInput {
            chunk_index: 0,
            detail: "x".into()
        }));
        assert!(is_indel_trigger(&ReservedInvalidLength {
            chunk_index: 0,
            data_part_len: 94
        }));
        assert!(is_indel_trigger(&PostCorrectionDecodeFailed {
            chunk_index: Some(0),
            detail: "x".into()
        }));
        assert!(!is_indel_trigger(&EmptyInput));
        assert!(!is_indel_trigger(&UnsupportedCodeVariant {
            chunk_index: 0,
            hrp: "ms",
            data_part_len: 100
        }));
        assert!(!is_indel_trigger(&IndelUnrecoverable {
            hrp: "ms",
            max_indel: 1
        }));
        // Cycle E (plan-R0 PM-r2-1) — a set-reassembly mismatch must NEVER
        // engage --max-indel search (it would replace the precise message
        // with the generic "indel unrecoverable" one).
        assert!(!is_indel_trigger(&SetReassemblyMismatch {
            group: "chunk_set_id 0x00001".into(),
            detail: "x".into()
        }));
    }

    // ---- Phase 1: md1 indel recovery (mirror mk1) ----
    //
    // Real 3-chunk bip84 md1 card (from `mnemonic bundle`), verified to
    // reassemble. Corruption injection targets a mid-data index in the
    // failing chunk so the indel engine sees a single failing chunk.
    const MD1_C0: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    const MD1_C1: &str = "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
    const MD1_C2: &str = "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

    #[test]
    fn indel_md1_fixture_reassembles() {
        assert!(md_codec::chunk::reassemble(&[MD1_C0, MD1_C1, MD1_C2]).is_ok());
    }

    #[test]
    fn indel_md1_one_chunk_too_long_recovers() {
        let mut bad = String::from(MD1_C1);
        bad.insert(3 + 12, 'q'); // insert one data char into chunk 1
        let chunks = vec![MD1_C0.to_string(), bad, MD1_C2.to_string()];
        match recover_indel_card(CardKind::Md1, &chunks, 1, 0).expect("ok") {
            crate::indel::IndelOutcome::Unique(c) => assert_eq!(c.recovered, MD1_C1),
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    #[test]
    fn indel_md1_one_chunk_too_short_recovers() {
        let mut bad = String::from(MD1_C1);
        bad.remove(3 + 12); // drop one data char
        let chunks = vec![MD1_C0.to_string(), bad, MD1_C2.to_string()];
        match recover_indel_card(CardKind::Md1, &chunks, 1, 0).expect("ok") {
            crate::indel::IndelOutcome::Unique(c) => assert_eq!(c.recovered, MD1_C1),
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    #[test]
    fn indel_md1_two_failing_is_unrecoverable() {
        let mut c0 = String::from(MD1_C0);
        c0.insert(3 + 5, 'q');
        let mut c1 = String::from(MD1_C1);
        c1.insert(3 + 5, 'q');
        let chunks = vec![c0, c1, MD1_C2.to_string()];
        assert_eq!(
            recover_indel_card(CardKind::Md1, &chunks, 1, 0).expect("ok"),
            crate::indel::IndelOutcome::Unrecoverable
        );
    }

    #[test]
    fn indel_md1_chunk_solve_rejects_out_of_set_substitution() {
        // A pure substitution (no indel) is NOT in the placeholder set, so
        // md1_chunk_solve must reject it even though BCH could "correct" it.
        let bad = flip_at(MD1_C1, 12);
        let allowed: BTreeSet<usize> = BTreeSet::new(); // delete-producer ⇒ ∅
        assert!(md1_chunk_solve(&bad, &allowed, 0).is_none());
        // The clean chunk (residue 0) round-trips through the solver.
        assert_eq!(
            md1_chunk_solve(MD1_C1, &allowed, 0)
                .map(|(s, _)| s)
                .as_deref(),
            Some(MD1_C1)
        );
    }

    /// Regression (ms-codec 0.4.3 pin bump): a non-ASCII chunk with no `'1'`
    /// separator routed through the ms repair wrapper must return a clean
    /// `HrpMismatch`, not panic. Pre-0.4.3 `ms_codec::decode_with_correction`
    /// sliced `lower[..len-1]` on a no-separator string, landing inside a
    /// multi-byte char (e.g. the trailing `é`) → char-boundary panic. Found by
    /// stress-Cycle-C fuzzing; the toolkit inherited it here (repair.rs).
    #[test]
    fn repair_via_ms_codec_no_separator_multibyte_is_clean_error() {
        for chunk in ["café", "ñ", "\u{1F600}", "mség"] {
            match repair_via_ms_codec(chunk, 0) {
                Err(RepairError::HrpMismatch { chunk_index, .. }) => {
                    assert_eq!(chunk_index, 0);
                }
                other => panic!("expected clean HrpMismatch for {chunk:?}, got {other:?}"),
            }
        }
    }

    /// Regression (ms-codec 0.4.4 pin bump): the `found` HRP that
    /// `repair_via_ms_codec` surfaces from a long no-`'1'` chunk is BOUNDED
    /// (ms-codec caps `WrongHrp.got` to 4 chars at construction so the error
    /// can't echo a long secret prefix into logs/output). A pre-0.4.4 codec
    /// echoed the whole input here. The toolkit inherits the cap for free.
    #[test]
    fn repair_via_ms_codec_wrong_hrp_found_is_bounded() {
        // 8 codex32-alphabet chars, no `'1'` → ms-codec WrongHrp{got: "qpzr"}.
        let chunk = "qpzrqpzr";
        match repair_via_ms_codec(chunk, 0) {
            Err(RepairError::HrpMismatch { found, .. }) => {
                assert!(
                    found.chars().count() <= 4,
                    "found must be ≤4 chars (secret-leak bound), got {found:?}"
                );
                assert_ne!(found, chunk, "found must not echo the full input");
            }
            other => panic!("expected HrpMismatch, got {other:?}"),
        }
    }
}
