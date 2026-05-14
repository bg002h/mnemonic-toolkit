# P2 execution plan (v0.13.0 SLIP-39 CLI surface)

**Phase:** v0.13.0 P2 (CLI handler + 5 CLI test files + lint anchors + `gui-schema` recursive bump + manual lint pass + env-var-driven G4 SHA-pin wedge)
**Status:** Plan-mode artifact, R0 architect-reviewed at `design/agent-reports/v0_13_0-p2-cli-plan-r0.md` (1C/5I/4N/5N findings — all folded inline into this revision). Opus follow-up consultation on Q2 settled the G4 determinism wedge in favor of runtime env-var gates (Option D); folded into §3.4 + §5 + §9.
**Date:** 2026-05-14
**Preconditions:** P1c-E.3 R1 LOCK at `81488e3` (Opus 0C/0I/0N clean LOCK). Library cycle (P0/P1a/P1b/P1c-A/B/C/D/E.1/E.2/E.3) COMPLETE; aggregate 899 tests / 0 fail / 8 ignored / 67 jobs at LOCK; clippy clean.

## §1 Goal

Land the `mnemonic slip39 split` + `mnemonic slip39 combine` CLI subcommand pair (`crates/mnemonic-toolkit/src/cmd/slip39.rs` + `main.rs` wiring), 5 CLI integration test files (`tests/cli_slip39_{happy_paths,refusals,advisories,json,stdin}.rs`), and the lint/schema/manual mirror updates that the SPEC §4 acceptance gates require for the user-facing surface. Total estimated diff: ~250–400 LOC of CLI handler + ~900–1100 LOC of CLI tests + ~30 LOC across lint anchor bumps + `gui-schema` test bump + `cli-subcommands.list` add. Closes SPEC §4 gates **G3** (plain stdout shape), **G4** (JSON envelope stability), **G5** (refusal coverage — 23 classes), **G6 Cycle A** (argv-leakage advisory + `Zeroizing<String>` wraps + lint rows), **G7 lint side** (`cli-subcommands.list` add forces P3 to mirror or fail manual lint), and **G9** (iteration-exponent advisory threshold). G6 Cycle B mlock pins on parsed inputs at the CLI layer are also folded in (matches the `cmd/seed_xor.rs:130,145,259,294` precedent). G1 + G2 + G6-Cycle-B-on-library-internals were closed in P1c-E.1/E.2/E.3. G7 manual chapter content + G8 Trezor smoke recipe are deferred to P3.

## §2 File inventory + scope

### §2.1 New files (3 cmd/test files + 5 CLI test files = 8 net new)

| Path | Est. LOC | Mirrors |
|---|---|---|
| `crates/mnemonic-toolkit/src/cmd/slip39.rs` | 350–450 | `cmd/seed_xor.rs` (445 LOC) — same structural template; SLIP-39 has more flags (group repeating, group-threshold, iteration-exponent) + nested-clap-subcommand pattern + 23 refusal classes vs seed-xor's 9 + 5 advisory classes vs seed-xor's 5 |
| `crates/mnemonic-toolkit/tests/cli_slip39_happy_paths.rs` | 200–280 | `cli_seed_xor_happy_paths.rs` (260 LOC) |
| `crates/mnemonic-toolkit/tests/cli_slip39_refusals.rs` | 280–400 | `cli_seed_xor_refusals.rs` (185 LOC); larger because SPEC §2.5 has 23 classes vs seed-xor's 9 |
| `crates/mnemonic-toolkit/tests/cli_slip39_advisories.rs` | 220–280 | `cli_seed_xor_advisories.rs` (240 LOC); 5 advisory classes (G9 threshold + K-of-N TTY + combine-TTY + per-share argv + world-readable) |
| `crates/mnemonic-toolkit/tests/cli_slip39_json.rs` | 200–260 | `cli_seed_xor_json.rs` (210 LOC) |
| `crates/mnemonic-toolkit/tests/cli_slip39_stdin.rs` | 140–200 | `cli_seed_xor_stdin.rs` (145 LOC); SLIP-39 N+1 stdin candidates (N `--share` + 1 `--passphrase-stdin`) drives 3 distinct multi-stdin refusal tests vs seed-xor's 1 |

### §2.2 Modified files (5)

| Path | Change | Lines touched |
|---|---|---|
| `crates/mnemonic-toolkit/src/cmd/mod.rs` | +1: `pub mod slip39;` | 1 |
| `crates/mnemonic-toolkit/src/main.rs` | +1 `Slip39(cmd::slip39::Slip39Args)` enum variant + +1 dispatch arm + about-text doc-comment | 4 |
| `crates/mnemonic-toolkit/tests/cli_gui_schema.rs` | Bump `gui_schema_lists_all_seven_subcommands` → `..._eight_subcommands`; insert `"slip39"` between `"seed-xor"` and `"verify-bundle"` (alphabetical); ADD ~3 new `#[test]` rows pinning `slip39` flag-shape (e.g. `--group` is text + repeating, `--group-threshold` is number, `--iteration-exponent` is number) | ~20 added |
| `crates/mnemonic-toolkit/tests/lint_argv_secret_flags.rs` | Bump strict count `assert_eq!(rows, 23)` → `28` (per **R0 Q1** below); ADD 5 new rows under `// ---- slip39 (5 rows) — v0.13.0 ----` | ~28 added |
| `crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs` | ADD 1 new row under `// ---- cmd/slip39.rs (v0.13.0) ----` mirroring the existing seed-xor row at line 204; loose-bound 18..=35 still satisfied | ~6 added |
| `docs/manual/tests/cli-subcommands.list` | ADD 2 lines: `mnemonic slip39 split` + `mnemonic slip39 combine` | 2 |
| `docs/manual/src/40-cli-reference/41-mnemonic.md` | (Optional at P2; canonical at P3) — minimum viable change to keep `lint.sh` `flag-coverage` green is to have *some* mention of every flag string the `--help` output advertises. **R0 Q4 below.** | TBD |

### §2.3 Out-of-scope at P2 (deferred to P3 / PE)

- Full P3 manual chapter prose (`## mnemonic slip39` H2 with Synopsis + Flags + Worked example + JSON + Refusals + Advisories + Trezor interop H3).
- G8 Trezor smoke-test recipe (manual smoke; not CI-gated).
- PE release rollup (Cargo.toml bump 0.13.0-dev → 0.13.0; CHANGELOG.md entry; `mnemonic-toolkit-v0.13.0` tag).
- `slip39-cli-extendable-flag` FOLLOWUP filed at P2.1 RED (per memory `project_v0_13_0_slip39_in_flight`); closes at the v0.14 cycle that adds the user-facing `--extendable` toggle.

## §3 CLI design (`src/cmd/slip39.rs`)

### §3.1 Args structures

Mirror `seed_xor.rs:26-95` shape — outer `Slip39Args` with `#[command(subcommand)]` field; `Slip39Command` enum with `Split(Slip39SplitArgs)` + `Combine(Slip39CombineArgs)` arms; per-arm `#[derive(Args, Debug)]` struct.

**`Slip39SplitArgs` flags** (per SPEC §2.2):

```rust
#[derive(Args, Debug)]
pub struct Slip39SplitArgs {
    /// Master secret as `phrase=<value-or->` OR `entropy=<hex-or->`.
    #[arg(long = "from", value_name = "phrase=<value-or--> or entropy=<hex-or-->",
          value_parser = parse_from_input, required = true)]
    pub from: FromInput,

    /// SLIP-39 passphrase (NOT BIP-39 passphrase). **R0 C1 fold:** field
    /// type is `Option<String>` (NOT `String + default_value=""`) per the
    /// `cmd/derive_child.rs:61` precedent. The `String + ""` shape would
    /// either spuriously emit the argv-leakage advisory on every invocation
    /// (when the implementer gates on field non-emptiness) OR silently
    /// suppress legitimate empty-passphrase argv occurrences (when the
    /// implementer gates on `args.passphrase.is_some()`). `Option` makes
    /// the user-supplied-vs-default distinction structurally explicit.
    #[arg(long = "passphrase", conflicts_with = "passphrase_stdin")]
    pub passphrase: Option<String>,

    /// Read passphrase from stdin (single-stdin-per-invocation;
    /// `conflicts_with = "passphrase"` enforced via clap).
    #[arg(long = "passphrase-stdin", default_value_t = false)]
    pub passphrase_stdin: bool,

    /// Groups required to reconstruct (1 <= G <= group_count).
    #[arg(long = "group-threshold", required = true)]
    pub group_threshold: u8,

    /// Group spec: repeating; `<member_count>,<member_threshold>`.
    /// **R0 I4 fold:** the `(u8, u8)` tuple's POSITION in this Vec IS the
    /// `group_idx` carried back in `Slip39Error::BadGroupSpec`. Forward via
    /// `args.group.iter().enumerate().map(|(_, (n, t))| GroupSpec {
    /// member_count: *n, member_threshold: *t }).collect()` — the transform
    /// is order-preserving so `group_idx` in the error stem matches the
    /// user-visible group ordinal in the argv.
    #[arg(long = "group", value_name = "N,T", required = true,
          action = clap::ArgAction::Append, value_parser = parse_group_spec)]
    pub group: Vec<(u8, u8)>,

    /// PBKDF2 cost exponent; 0..=15; iterations = 10000 · 2^E.
    #[arg(long = "iteration-exponent", default_value_t = 0)]
    pub iteration_exponent: u8,

    /// BIP-39 language of input phrase; ignored for `entropy=`.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Side-effect: write versioned JSON envelope to PATH.
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}
```

**`Slip39CombineArgs` flags** (per SPEC §2.2):

```rust
#[derive(Args, Debug)]
pub struct Slip39CombineArgs {
    /// Repeating; at most ONE may be `-` (stdin).
    #[arg(long = "share", value_name = "<slip39-mnemonic-or->", required = true,
          action = clap::ArgAction::Append)]
    pub share: Vec<String>,

    /// SLIP-39 passphrase used at split time. **R0 C1 fold:** same
    /// `Option<String>` + `conflicts_with = "passphrase_stdin"` shape as
    /// the split args.
    #[arg(long = "passphrase", conflicts_with = "passphrase_stdin")]
    pub passphrase: Option<String>,

    /// Read passphrase from stdin (incompatible with any `--share -` AND
    /// with `--passphrase`).
    #[arg(long = "passphrase-stdin", default_value_t = false)]
    pub passphrase_stdin: bool,

    /// Output shape: `entropy` (default; hex on stdout) or `phrase` (BIP-39).
    #[arg(long = "to", default_value = "entropy")]
    pub to: Slip39ToShape,  // enum {Entropy, Phrase}

    /// BIP-39 language for `--to phrase`; ignored for `--to entropy`.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Side-effect: write versioned JSON envelope to PATH.
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}
```

**Note on `--extendable`:** v0.13.0 hardcodes `extendable=false` for both directions per P1c-E.1 R0 Q1 + memory `project_v0_13_0_slip39_in_flight`. No CLI flag exposed. The library's `slip39_split` already takes `extendable: bool` so v0.14 can add a `--extendable` toggle without library churn. FOLLOWUP `slip39-cli-extendable-flag` is filed at P2.1 RED.

### §3.2 Refusal mapping (24 classes — Q3 RESOLVED at R0)

Each SPEC §2.5 row maps to either (a) a CLI-layer refusal *before* lib invocation (rows 1, 2, 3, 4, 5, 6, 17, 18, 19) or (b) a lib-layer `Slip39Error` variant rendered into a CLI-layer `ToolkitError::BadInput(format!(...))` per the SPEC §B.2.5 stems table (rows 7–16, 20–24). The `map_slip39_error` helper is the single mapping site — analogous to `map_seed_xor_error` at `cmd/seed_xor.rs:338-350`.

**R0 Q3 fold: SPEC §2.5 grows row 24** (paired patch in P2.2 GREEN) for `MemberThresholdMismatch` — verified at `slip39/error.rs:64-66` it is structurally distinct from row 15 (`DuplicateMemberIndex`) and from row 12 (`InsufficientShares` member-level branch); a single shared lib variant must NOT render under multiple SPEC stems. Total refusal classes 23 → 24.

**R0 I2 fold: every interpolated stem rendered as the actual `format!` template the implementer types** (no more I/J/N/T placeholders — explicit `{share_idx}` / `{group_idx}` / `{member_idx}` / `{needed}` / `{got}` / `{threshold}` / `{count}` field bindings). Library-variant fields verified against `crates/mnemonic-toolkit/src/slip39/error.rs:29-124` + SPEC `design/SPEC_slip39_v0_13_0.md:208-234`:

| Lib variant | SPEC row | `format!` template |
|---|---|---|
| `BadPhraseWordCount(got)` | 1 | `format!("slip39 split: input phrase must be 12/15/18/21/24 words; got {got}")` |
| `BadEntropyByteLength(got)` | 2 | `format!("slip39 split: entropy hex must decode to 16/20/24/28/32 bytes; got {got} bytes")` |
| `BadGroupThreshold { got, group_count }` | 3 | `format!("slip39 split: --group-threshold must be in 1..={group_count} (number of --group flags); got {got}")` |
| `BadGroupSpec { group_idx, n, t }` w/ `(n, t) != (1, 1)` | 4 | `format!("slip39 split: --group N,T requires 1 <= T <= N <= 16; got group {group_idx}={n},{t}")` |
| `BadGroupSpec { group_idx, n: 1, t: 1 }` | 5 | `format!("slip39 split: 1-of-1 group offers no recovery benefit; use --group N,T with N >= 2 (toolkit policy); got group {group_idx}=1,1")` |
| `BadIterationExponent(got)` | 6 | `format!("slip39 split: --iteration-exponent must be 0..=15 (4-bit field); got {got}")` |
| `IdentifierMismatch` | 7 | `"slip39 combine: shares disagree on identifier; shares must come from the same secret"` (no fields to interpolate; no `format!` needed) |
| `IterationExponentMismatch` | 8 | `"slip39 combine: shares disagree on iteration-exponent"` |
| `InvalidChecksum { share_idx }` | 9 | `format!("slip39 combine: share at position {share_idx} has invalid SLIP-39 checksum (RS1024)")` |
| `UnknownWord { share_idx, word_idx }` | 10 | `format!("slip39 combine: share at position {share_idx}: word at index {word_idx} not in SLIP-39 wordlist")` |
| `DigestVerificationFailed` | 11 | `"slip39 combine: reconstructed master digest mismatch — wrong --passphrase OR a share was substituted"` |
| `InsufficientShares { group_idx, needed, got }` | 12 | `format!("slip39 combine: insufficient shares for group {group_idx}: need {needed}, got {got}")` (covers BOTH member-level AND the group-level sentinel `group_idx == GROUP_LEVEL_SENTINEL == 255` case; CLI handler may map sentinel to a special `"<groups>"` token at the format boundary) |
| `GroupThresholdMismatch` | 13 | `"slip39 combine: shares disagree on group_threshold"` |
| `GroupCountMismatch` | 14 | `"slip39 combine: shares disagree on group_count"` |
| `DuplicateMemberIndex { group_idx, member_idx }` | 15 | `format!("slip39 combine: duplicate member index {member_idx} in group {group_idx}")` |
| `InvalidPadding { share_idx }` | 16 | `format!("slip39 combine: share at position {share_idx} has non-zero padding bits (encoding violation)")` |
| `EmptyShares` | 19 | `"slip39 combine: at least one share required"` |
| `InvalidShareValueLength { share_idx, got }` | 20 | `format!("slip39 combine: share at position {share_idx} has value length {got} (must be 16/20/24/28/32 bytes)")` |
| `ShareValueLengthMismatch` | 21 | `"slip39 combine: shares disagree on value length"` |
| `ExtendableMismatch` | 22 | `"slip39 combine: shares disagree on the extendable bit"` |
| `GroupThresholdExceedsCount { share_idx, threshold, count }` | 23 | `format!("slip39 combine: share at position {share_idx}: group_threshold {threshold} exceeds group_count {count}")` |
| `MemberThresholdMismatch` | **24** (NEW per Q3 fold) | `"slip39 combine: shares within a group disagree on member_threshold"` |

CLI-only refusals (no lib variant — handled in CLI handler before lib call):

- **Row 17** (`--from` variant other than `phrase=` / `entropy=`): match on `args.from.node` is `Phrase` or `Entropy`; else refuse with `format!("slip39 split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got {}=", args.from.node.as_str())`.
- **Row 18** (multi-stdin contention): N+1 stdin candidates (N `--share -` + optional `--passphrase-stdin` + `--from -` at split). Single allowed total. The CLI handler counts stdin-consuming inputs at the head of `run_split` / `run_combine` and refuses with `"slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)"` if count > 1. Reuses single-stdin-per-invocation precedent at `cmd/convert.rs:637-651`.

### §3.3 Advisories (8 classes per SPEC §2.6 + G9 threshold)

**R0 I3 fold:** the original "single argv-leakage row covering --from / --share / --passphrase" was incomplete — the lint enumeration in §4.3 has 5 rows because there are 5 distinct (subcommand, flag) pairs. The advisory wiring mirrors that enumeration: 5 distinct call sites (3 in `run_split`, 2 in `run_combine`). Per-occurrence-not-deduped (matches `cmd/seed_xor.rs:237-241`).

**R0 C1 fold:** the argv-leakage advisory for `--passphrase` fires iff `args.passphrase.is_some()` (the user supplied the flag, regardless of value), NOT iff the field is non-empty. Crucial because `Option<String>` distinguishes "user supplied" from "default" structurally.

| # | Trigger | Advisory wiring source |
|---|---|---|
| 1a | `split --from phrase=<inline>` | `secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-")` in `run_split` |
| 1b | `split --from entropy=<inline>` | `secret_in_argv_warning(stderr, "--from entropy=", "--from entropy=-")` in `run_split` |
| 1c | `split --passphrase <inline>` (any value, including empty string) | `if args.passphrase.is_some() { secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin"); }` in `run_split` |
| 1d | `combine --share <inline>` (per-share) | `for sh in &args.share { if sh != "-" { secret_in_argv_warning(stderr, "--share", "--share -"); } }` in `run_combine` (per-occurrence) |
| 1e | `combine --passphrase <inline>` | mirrors 1c, in `run_combine` |
| 2 | `split` AND stdout is TTY | NEW K-of-N parameterized advisory: `warning: SLIP-39 shares on stdout — N=<n> shares emitted across <g> groups (group-threshold <G>); each share is independently secret material; distribute per your group/member-threshold policy; do not paste this output into a single untrusted tool` — extends v0.12.0's seed-xor TTY advisory shape (n=share count, g=group count, G=group_threshold). Single-line; format kept ≤ 200 chars even at the 256-share spec max (verified at draft time against the exact substitution string) |
| 3 | `combine` AND stdout is TTY | `warning: reconstructed secret material on stdout — verify the recovered wallet's expected derived address before trusting` — closely mirrors final-word stdout-on-TTY advisory at `cmd/final_word.rs:99-106` |
| 4 | `--json-out` to a world-readable path | Reuse `#[cfg(unix)]` permission-mode helper. **R0 Q5 fold:** extract `emit_world_readable_advisory` from its `cmd/seed_xor.rs:425-445` private location to `crate::secret_advisory::warn_if_world_readable` at P2.2 GREEN. The `secret_advisory` module currently exports `secret_in_argv_warning`; the new fn gets a doc-comment delineating "argv-leakage advisory class" vs "path-permission advisory class". The seed-xor and final-word call sites get updated in lockstep at P2.2 GREEN (3 call sites total). |
| 5 | `--iteration-exponent E` where E >= 5 | NEW G9 threshold advisory: `warning: --iteration-exponent E=<E> yields <iters> × PBKDF2-HMAC-SHA-256 iterations; split + combine performance may be observably slow (sub-second to multi-second). Trezor's reference uses E=1 (20000 iters) as default; the SLIP-0039 spec gives no recommended values. E >= 10 may exceed 30s on weak hardware.` |
| 6 | `MNEMONIC_SLIP39_TEST_RNG` env-var set on `split` invocation (NEW per Q2 fold; always-on, not suppressible) | `warning: MNEMONIC_SLIP39_TEST_RNG set — output is deterministic and INSECURE; do not use for real shares` (per Opus G4-architect consult — see §3.4 below) |

### §3.4 JSON envelope (per SPEC §2.3) + G4 SHA-pin via env-var wedge (Q2 RESOLVED)

Two `#[derive(serde::Serialize)]` structs (mirroring `cmd/seed_xor.rs:352-371`):

- `SplitJson` fields in declaration order: `schema_version: "1"`, `operation: "split"`, `identifier: u16`, `iteration_exponent: u8`, `group_threshold: u8`, `groups: Vec<SplitGroupEntry>`. `SplitGroupEntry` field order: `member_count`, `member_threshold`, `shares: Vec<&str>` (R0 N4 fold: `shares` LAST, matching `seed_xor.rs:352-361` precedent).
- `CombineJson` fields: `schema_version: "1"`, `operation: "combine"`, `identifier: u16`, `iteration_exponent: u8`, `output_shape: "phrase"|"entropy"`, `phrase: Option<&str>`, `entropy_hex: Option<&str>`.

Field order is part of the schema (SHA-pinned in `tests/cli_slip39_json.rs` per SPEC §4 G4).

**G4 SHA-pin determinism wedge (Q2 RESOLVED — Opus follow-up consultation).** SLIP-39 split is INHERENTLY non-deterministic (the SLIP-0039 spec REQUIRES a 15-bit random identifier + random Shamir shares per encryption pass). SPEC §2.2 deliberately does NOT expose `--identifier` or `--rng-seed` CLI flags. To enable G4 SHA-pin tests without polluting the production user surface, P2 ships a runtime env-var wedge:

- `MNEMONIC_SLIP39_TEST_RNG=<32-byte-hex>`: when set, `cmd::slip39::run_split` decodes the hex, seeds a `ChaCha20Rng::from_seed(...)`, and uses it in place of `OsRng`. Invalid hex / wrong length → refuse with a clear error stem (NOT silent fallback). When unset → production path (`OsRng`).
- `MNEMONIC_SLIP39_TEST_IDENTIFIER=<u16-decimal>`: when set, used as the `identifier` argument to `slip39_split` (overriding the RNG-derived random). Constrained to 0..=32767 (15-bit). When unset → library generates random per spec.
- **Always-on insecurity advisory** (advisory row 6 in §3.3): when EITHER env-var is set, `run_split` emits `warning: MNEMONIC_SLIP39_TEST_RNG set — output is deterministic and INSECURE; do not use for real shares` (per Opus consult: not suppressible; loud-by-design — env-var misuse must be self-disclosing in any captured terminal log).

**Why env-var beats alternatives** (per Opus consult, condensed):
- `#[cfg(test)]` gate is BROKEN: `assert_cmd::Command::cargo_bin("mnemonic")` exercises the SHIPPED production binary, NOT a `#[cfg(test)]`-gated rebuild.
- `#[cfg(feature = "test-determinism")]` works but adds build-graph footprint + risks transitive activation by downstream consumers.
- Hidden CLI flag breaks repo discipline ("if it's in `--help`, it's user-facing"; the only way to avoid this is to lie in `--help` or contradict the rule).
- Env-var is conventionally "operator/test-harness territory" (cf. `RUST_LOG`, `CARGO_*`); zero build-graph footprint; scoped to the process; always-on warning makes misuse loud.

**SPEC patches queued for P2.2 GREEN** (per Opus consult):
- SPEC §B.2 stems table → append row for the `MNEMONIC_SLIP39_TEST_RNG` advisory.
- SPEC §4 G4 → replace the existing parenthetical with: `"SHA-pinned over 2 anchor vectors. Determinism is achieved at test time via the env-vars MNEMONIC_SLIP39_TEST_RNG (32-byte hex seed for ChaCha20Rng) and MNEMONIC_SLIP39_TEST_IDENTIFIER (decimal u16); both are inert in production and trigger an always-on insecurity advisory when set. The production --help surface does not mention these vars; they are documented in SPEC §B.3 only."`
- NEW SPEC §B.3 "Test-only environment variables" subsection naming the two vars, the warning stem, and a non-promise of stability (may change without semver bump; not part of the v0.13.0 user-facing contract).

**CI test wiring:** `cli_slip39_json.rs` SHA-pin tests use `assert_cmd::Command::env("MNEMONIC_SLIP39_TEST_RNG", ...).env("MNEMONIC_SLIP39_TEST_IDENTIFIER", ...)`. Each pinned test ALSO asserts the always-on insecurity stem appears in stderr (so the warning's wiring is itself test-pinned).

### §3.5 Memory hygiene at the CLI layer

- Wrap parsed `--from` value (phrase or entropy hex) in `Zeroizing<String>` before forwarding to the lib (mirrors `cmd/seed_xor.rs:125-129`).
- Wrap parsed `--share` strings in `Zeroizing<String>` before invoking `parse_slip39_share` (mirrors `cmd/seed_xor.rs:244-261`; corrected line range per R0 N1 fold).
- Wrap parsed `--passphrase` value (the inner `String` of `Option<String>`) in `Zeroizing<String>`.
- mlock `pin_pages_for` on each parsed-input heap buffer (per Cycle B Phase 3a Site 1 pattern; precedent `cmd/seed_xor.rs:130,145,259`).
- The `Vec<Vec<Share>>` returned by `slip39_split` is already `Zeroize` + `ZeroizeOnDrop` on `Share.value` (per `slip39/share.rs` + plan §4.1 LOCK); rendered share strings (returned by `render_slip39_share`) wrap in `Zeroizing<String>` at the boundary.

**R0 Note 2 fold — stdin reader choice (silent-correctness foot-gun):**
- For `--from -` and `--share -`: use `read_stdin_to_string` (`cmd/convert.rs:566-572`); calls `.trim()` which strips trailing newline AND leading/trailing whitespace. Safe for SLIP-39 share mnemonics (whitespace-separated word lists; internal single-space separators preserved).
- For `--passphrase-stdin`: use `read_stdin_passphrase` (`cmd/convert.rs:574-590`); strips ONLY a single trailing `\r?\n` to preserve user-supplied whitespace AND NULL bytes inside the passphrase (SLIP-39 passphrase is `&[u8]` at the lib).
- **Mismatching these is silently wrong**: passing `read_stdin_to_string` output as a passphrase silently strips significant whitespace; the bug surfaces only at recovery time when the same passphrase yields a different EMS.

**R0 Q6 fold — per-share output pinning is O(N), NOT O(1) (SPEC §2.1 patch queued for P2.2 GREEN):**

The SPEC §2.1 claim *"Per-share output pin discipline (O(1), not O(N))"* is structurally unachievable with `Vec<Zeroizing<String>>`: the top-level Vec's backing buffer holds `String` headers (24 bytes each on 64-bit; non-secret), but each `String`'s UTF-8 bytes live in a SEPARATE heap allocation (the actual share value). `mlock::pin_pages_for(&v[..])` pins the headers, NOT the share-byte allocations. The cited seed-xor precedent (`cmd/seed_xor.rs:157-164`) uses the same `Vec<Zeroizing<String>>` shape and is also O(N) (the seed-xor implementation in fact does NOT pin the rendered output AT ALL — only the parsed entropy + parsed master phrase get pinned). So:

- **P2 ships O(N) per-share output pinning**: a `for share in &rendered_shares { let _pin = mlock::pin_pages_for(share.as_bytes()); ... writeln!(stdout, "{share}")?; ... }` loop. Each pin's `_pin` guard drops at end of loop iteration; pages stay locked until the next iteration completes (acceptable; sequential write).
- For 256 shares × ~32 bytes/share value = ~8 KB pinned at peak, well within `ulimit -l` (default 64 KB).
- **SPEC §2.1 paragraph patch queued for P2.2 GREEN**: rewrite "Per-share output pin discipline" to say `"per-rendered-share pin (O(N), one mlock::pin_pages_for call per share inside the stdout-emit loop in run_split)"`. Remove the "single pin" claim; remove the `cmd/seed_xor.rs:157-164` precedent cite from SPEC §5 row 12 (it's not a single-pin precedent). The slip39 per-share loop is a NEW pattern, not a precedent reuse.

## §4 Test design

### §4.1 Test file split (5 files, ~1100 LOC)

Mirrors v0.12.0's 5-file split. All use `assert_cmd::Command::cargo_bin("mnemonic")` dispatch + `(stdout, stderr, exit_code)` triple per the seed-xor precedent.

| File | Coverage | Approx test count |
|---|---|---|
| `cli_slip39_happy_paths.rs` | Round-trip 1-of-1, 2-of-3, 1-of-(2-groups), 2-of-3-groups (4-tier hierarchy) at 12/24-word phrase + 32-byte entropy; default-language; trailing-newline; entropy/phrase output shapes; non-default `--passphrase` round-trip | 8–12 |
| `cli_slip39_refusals.rs` | All 23 SPEC §2.5 rows (one `#[test]` each); G5 acceptance gate criterion. Verify exit code 1 + stderr stem byte-faithful | 23 |
| `cli_slip39_advisories.rs` | All 5 advisory classes (positive + negative test each = 10 tests); piped vs TTY discrimination via `assert_cmd` (which always pipes); per-occurrence count for argv-leakage | 10–12 |
| `cli_slip39_json.rs` | `--json-out` schema_version/operation/field shape (split + combine); plain stdout coexists with `--json-out`; G4 SHA-pin (gated on R0 Q2 resolution) | 6–10 |
| `cli_slip39_stdin.rs` | `--share -` round-trip (split → one stdin share + N-1 inline → combine); `--passphrase-stdin` route; multi-stdin refusals (3 pairwise classes per §3.2 row 18) | 6–8 |

### §4.2 `tests/cli_gui_schema.rs` bumps + Q7 nested-subcommand fix (R0 Q7 RESOLVED)

**R0 Q7 fold — pre-RED probe COMPLETE; bug is pre-existing in v0.12.0; fix benefits both seed-xor AND slip39:**

Probed `cargo run --bin mnemonic -- gui-schema | jq '.subcommands[] | select(.name == "seed-xor")'` against the LOCK-clean tree at `81488e3`. Result confirms reviewer prediction:

```json
{ "name": "seed-xor", "flags": [], "positionals": [] }
```

The v0.12.0 release ALREADY ships seed-xor with this empty-flags rendering — `mnemonic-gui` cannot see `seed-xor split` / `seed-xor combine` as discoverable subcommands. **This is a pre-existing v0.12.0 gap, NOT a v0.13.0 regression.** The user's chosen fix (Option Q7 (b): flatten via hyphenated names) repairs BOTH at once.

**P2.1 RED gate work** (added to §5 P2.1 RED commit):
- `cmd/gui_schema.rs::build_schema` recurses into nested subcommand trees: when `cmd.get_subcommands()` for a sub `S` returns non-empty (i.e. `S` is itself a parent of nested sub-subs), emit one `Subcommand { name: format!("{}-{}", S.get_name(), sub_sub.get_name()), ... }` per nested sub-sub IN PLACE OF the parent `S`. So `seed-xor` becomes `seed-xor-split` + `seed-xor-combine`; `slip39` becomes `slip39-split` + `slip39-combine`. Total subcommands: was 7 (with seed-xor as 1), becomes 8 with the v0.12.0 fix (seed-xor-split + seed-xor-combine replace seed-xor); becomes 10 with the v0.13.0 add (slip39-split + slip39-combine added).
- Schema_version stays at 1 — the change is structural (more subcommand names) but does NOT alter the schema document shape. The `mnemonic-gui` v0.2 contract pins `version == 1`; flattened-name addition is additive (existing names disappear; new names appear; the GUI's per-subcommand-name dispatch logic must be updated, but no schema-version bump is required).
- **Companion `mnemonic-gui` work**: file FOLLOWUP entry `slip39-gui-schema-flattening-companion` in `mnemonic-gui` repo (cross-citing this plan's §4.2). The GUI's slip39 surface depends on this rename landing.

**`tests/cli_gui_schema.rs` bumps:**
- Rename `gui_schema_lists_all_seven_subcommands` → `gui_schema_lists_all_ten_subcommands`. Update the alphabetical list to: `["bundle", "convert", "derive-child", "export-wallet", "final-word", "seed-xor-combine", "seed-xor-split", "slip39-combine", "slip39-split", "verify-bundle"]`.
- ADD ~4 new `#[test]` rows pinning slip39-specific flag shapes:
  - `slip39_split_subcommand_has_required_from_and_group_threshold`
  - `slip39_split_subcommand_group_flag_is_text_kind_and_repeating` (`--group N,T` uses custom `parse_group_spec` + `ArgAction::Append` → kind="text" + `repeating=true` per SPEC §7 lossy mapping)
  - `slip39_combine_subcommand_share_flag_is_text_kind_and_repeating`
  - `slip39_split_subcommand_passphrase_flag_is_text_kind_and_optional` (NOT required)
- ADD a test pinning that the seed-xor-* flattening landed correctly: `seed_xor_split_subcommand_has_required_from_and_shares_flags`.

### §4.3 `tests/lint_argv_secret_flags.rs` bumps

Strict-equal count: `assert_eq!(rows, 28)` (was 23; +5 for slip39 — see §4.3.1 below for the +5 vs SPEC's +4 drift).

ADD 5 rows (the `"slip39 ... --foo"` labels mirror the existing `"seed-xor ... --bar"` labels at lines 159-168):

```rust
// ---- slip39 (5 rows) — v0.13.0 ----
FlagRow {
    label: "slip39 split --from phrase=",
    source_file: "src/cmd/slip39.rs",
    evidence: &["--from phrase=-", "secret_in_argv_warning"],
},
FlagRow {
    label: "slip39 split --from entropy=",
    source_file: "src/cmd/slip39.rs",
    evidence: &["--from entropy=-", "secret_in_argv_warning"],
},
FlagRow {
    label: "slip39 split --passphrase",
    source_file: "src/cmd/slip39.rs",
    evidence: &["passphrase_stdin", "passphrase-stdin"],
},
FlagRow {
    label: "slip39 combine --share",
    source_file: "src/cmd/slip39.rs",
    evidence: &["--share -", "secret_in_argv_warning"],
},
FlagRow {
    label: "slip39 combine --passphrase",
    source_file: "src/cmd/slip39.rs",
    evidence: &["passphrase_stdin", "passphrase-stdin"],
},
```

#### §4.3.1 SPEC §4 G6 row-count drift candidate (R0 Q1)

SPEC §4 G6 enumerates 4 new lint rows ("23 → 27"): `slip39 split --from phrase=`, `slip39 split --from entropy=`, `slip39 combine --share`, `slip39 split --passphrase`. **The SPEC omits `slip39 combine --passphrase`.** Per SPEC §2.2 the combine subcommand has both `--passphrase <P>` AND `--passphrase-stdin`, so it IS a secret-bearing flag. The lint convention (per `lint_argv_secret_flags.rs:50-156`) is one row per (subcommand, flag) pair — `convert --passphrase` and `convert --bip38-passphrase` are separate rows; `bundle --passphrase`, `verify-bundle --passphrase`, `derive-child --passphrase` are each separate. So the correct count is 23 → 28 (+5, not +4).

This plan adopts the +5 / 28 reading. **R0 Q1: confirm or rebut; if confirmed, the SPEC §4 G6 row needs a paired SPEC patch at LOCK** (or roll into PE).

### §4.4 `tests/lint_zeroize_discipline.rs` bump

Loose bound 18..=35 still satisfied (current 28 → 29). ADD 1 row matching the seed-xor row at line 204:

```rust
// ---- cmd/slip39.rs (v0.13.0) ----
ZeroizeRow {
    label: "slip39 run() parsed --from + --share + --passphrase wrap in Zeroizing<String>",
    source_file: "src/cmd/slip39.rs",
    evidence: &["zeroize::Zeroizing::new"],
},
```

### §4.5 `docs/manual/tests/cli-subcommands.list` add

ADD 2 lines under the `# mnemonic` block, after `mnemonic seed-xor combine` and before `mnemonic gui-schema`:

```
mnemonic slip39 split
mnemonic slip39 combine
```

This forces `lint.sh` `flag-coverage` to run `mnemonic slip39 split --help` and `mnemonic slip39 combine --help` and grep each emitted flag against `docs/manual/src/40-cli-reference/41-mnemonic.md`. **R0 Q4 below** — minimum-viable manual stub vs P3-canonical chapter.

## §5 Phase split

Three sub-phases, each closing with a reviewer-loop LOCK. **R0 I5 fold:** dropped `cli_slip39_help_fixtures.rs` (verified via Glob: no `tests/cli_*help*` file in the repo; no convention to extend). P2.1 RED relies on the bumped `cli_gui_schema.rs` test as the sole structural-shape gate. **R0 Q7 fold:** P2.1 RED grows the `cmd/gui_schema.rs` flattening fix that retroactively repairs seed-xor's empty-flags rendering.

| Sub-phase | RED commit | GREEN commit | LOCK round |
|---|---|---|---|
| **P2.1** main wiring + minimum-viable subcommand + gui-schema flattening fix | `test(slip39): v0.13.0 P2.1 RED — main.rs Slip39 enum + cli_gui_schema.rs 7→10 subcommand bump (seed-xor-{split,combine} + slip39-{split,combine}) + filed slip39-cli-extendable-flag FOLLOWUP + filed slip39-gui-schema-flattening-companion FOLLOWUP` | `feat(slip39): v0.13.0 P2.1 GREEN — minimal cmd/slip39.rs (Args + Subcommand + run-stub returning a clean ToolkitError::BadInput("P2.1 stub — full impl ships at P2.2") for both sub-arms) + main.rs dispatch + cmd/mod.rs export + cmd/gui_schema.rs nested-subcommand flattening` | post-GREEN R1 LOCK |
| **P2.2** full handler + 5 CLI test files + lint bumps + SPEC patches | `test(slip39): v0.13.0 P2.2 RED — 5 cli_slip39_*.rs files (~1100 LOC) + lint_argv_secret_flags.rs +5 rows (count 23→28) + lint_zeroize_discipline.rs +1 row + env-var SHA-pin tests + extracted secret_advisory::warn_if_world_readable shared helper test` | `feat(slip39): v0.13.0 P2.2 GREEN — cmd/slip39.rs handler impl (split + combine + 24-class refusal mapping + 8-row advisory wiring + JSON envelope + env-var determinism wedge + Zeroizing/mlock discipline) + secret_advisory::warn_if_world_readable extraction + SPEC patches: §2.1 per-share-pin O(N) clarification + §2.5 row 24 add + §4 G4 env-var language + §B.2 advisory row + NEW §B.3 test-only-env-vars subsection + SPEC §4 G6 count 23→28 update` | post-GREEN R1 LOCK |
| **P2.3** manual lint mirror (minimum-viable stub) | `test(manual): v0.13.0 P2.3 RED — cli-subcommands.list adds slip39 split + combine; lint.sh fail demonstrates uncovered flags` | `docs(manual): v0.13.0 P2.3 GREEN — minimum-viable 41-mnemonic.md stub mentioning all slip39 flag strings (P3 fleshes out canonical chapter)` | post-GREEN R1 LOCK |

P3 (manual chapter prose + Trezor smoke recipe) is a separate session after P2.3 LOCKs.
PE (release rollup + tag) is a separate session after P3 LOCKs.

**Review cadence:**
- R0 (this plan) — Opus architect-review on plan; persisted to `design/agent-reports/v0_13_0-p2-cli-plan-r0.md`; folded inline (THIS revision).
- Q2 follow-up consult — separate focused Opus consult on the G4 SHA-pin determinism wedge; recommendation (env-var Option D) folded into §3.4.
- P2.2 pre-GREEN test-design review — Opus reviews the 5 CLI test files for SPEC §2.5 + §2.6 row-by-row coverage *before* the handler impl lands (per memory `feedback_r0_must_read_source_off_by_n` — verify each test's stem match against SPEC §2.5 stems table by grep).
- Each sub-phase post-GREEN R1 review — same Opus architect agent per memory `feedback_opus_primary_review_agent`.

**Note 3 fold — `slip39-cli-extendable-flag` FOLLOWUP draft:** entry filed as companion to `slip39-shamir-secret-sharing` at `design/FOLLOWUPS.md:1039`. Body summary: *"v0.13.0 P2 ships `mnemonic slip39 split` with `extendable=false` hardcoded (per P1c-E.1 R0 Q1 — library accepts `extendable: bool` but CLI does not surface it; v0.13.0 priority is SLIP-39 K-of-N parity with Trezor's reference behavior). v0.14 cycle adds the user-facing `--extendable` toggle, paired with combine-time validation that all parsed shares share the bit; SPEC §2.5 row 22 (`ExtendableMismatch`) already exists for the combine-time refusal."* Tier: stable; no companion repo entry needed (toolkit-only).

## §6 Risk areas (post-fold)

1. **Multi-stdin refusal correctness** (SPEC §2.5 row 18 / N+1 candidates). Three pairwise classes: (a) `--passphrase-stdin` + any `--share -`; (b) `--passphrase-stdin` + `--from -` (only at split); (c) two distinct `--share -` slots (only at combine). The `cmd/convert.rs:637-651` precedent catches the analogous case for convert's smaller surface; we extend the pattern to N+1. Mitigation: §4.1 stdin file pins one test per pairwise class.
2. **G4 SHA-pin env-var wedge — test-suite hygiene** (Q2 RESOLVED). The env-var-driven determinism wedge ships in production binary code paths. Risk: a future refactor of `run_split` could drop the env-var inspection silently, breaking G4 SHA-pin tests AND removing the always-on insecurity advisory. Mitigation: P2.2 RED test asserts the always-on advisory stem (advisory row 6) appears in stderr whenever `MNEMONIC_SLIP39_TEST_RNG` is set; a regression that drops the env-var inspection ALSO drops the advisory, which the test catches.
3. **`gui_schema` flattening — flag-coverage bridge across rename** (Q7 RESOLVED). The `mnemonic-gui` v0.2 contract pins `version == 1`. Renaming `seed-xor` → `seed-xor-split` + `seed-xor-combine` in the schema is additive at the schema-doc level but BREAKING at the GUI's per-subcommand-name dispatch. Companion FOLLOWUP filed in `mnemonic-gui` repo. Mitigation: P2.1 GREEN cites the FOLLOWUP in the commit message; PE rollup verifies the GUI companion FOLLOWUP has been actioned (or downgrades to "ship slip39 + GUI lag accepted").
4. **K-of-N TTY advisory text quality** (SPEC §2.6 row 2). The parameterized text emits N (share count), G (group count), and `group_threshold`. For 16-group × 16-member configs (the SLIP-39 spec max = 256 shares), the message length matters; mitigation = pin a single-line format keeping the 256-share case ≤ 200 chars (verified at draft time per §3.3 row 2 note).
5. **Hidden interaction between `--language english` (default) and `--from entropy=`** at split. The SPEC §2.2 split table says language is "ignored for `entropy=`"; current handler design (§3.1) accepts the flag silently but uses it only on the phrase path. Mitigation: §4.1 happy-path test pins that `--language spanish --from entropy=<hex>` round-trips correctly (language flag silent for entropy input).
6. **`secret_advisory::warn_if_world_readable` extraction breaking 3 call sites** (Q5 RESOLVED). Move-to-shared touches `cmd/seed_xor.rs:425-445` (delete) + `cmd/final_word.rs:178-197` (replace inline block with helper call) + `cmd/slip39.rs` (NEW call site). All 3 sites must update at the SAME P2.2 GREEN commit; partial migration leaves the helper unused (clippy warning) OR with stale duplicates. Mitigation: P2.2 GREEN commit message lists all 3 sites; pre-commit `cargo clippy --all-targets -- -D warnings` catches the partial-migration case.
7. **Stdin reader mismatch (Note 2)** — silent-correctness foot-gun documented in §3.5 above. Mitigation: §3.5 stdin-reader-choice paragraph is the canonical reference; reviewers at P2.2 pre-GREEN test-design review verify each `read_stdin_*` call site uses the right reader for its input class.

## §7 Open questions — RESOLVED at R0 + Q2 follow-up consult

R0 + Opus follow-up settled all seven open questions (see `design/agent-reports/v0_13_0-p2-cli-plan-r0.md` + the dispatching-planner's Q2 consult-response capture). Decisions baked into this revision:

| ID | Resolution | Folded into |
|---|---|---|
| **Q1** | +5 rows (count 23 → 28). SPEC §4 G6 has a paired patch queued for P2.2 GREEN. `slip39 combine --passphrase` row is added alongside the SPEC's 4 enumerated rows; the lint convention's "one row per (subcommand, flag) pair" is preserved. | §4.3 (already +5) + §5 P2.2 SPEC patches |
| **Q2** | Env-var wedge (`MNEMONIC_SLIP39_TEST_RNG` + `MNEMONIC_SLIP39_TEST_IDENTIFIER`) with always-on insecurity advisory. Beats `#[cfg(test)]` (broken — `cargo_bin` builds production), beats `--features test-determinism` (build-graph footprint), beats hidden CLI flag (breaks `--help`-as-user-surface discipline). SPEC §B.2 + §4 G4 + §B.3 patches queued for P2.2 GREEN. | §3.4 + §3.3 row 6 + §5 P2.2 SPEC patches |
| **Q3** | Add SPEC §2.5 row 24 for `MemberThresholdMismatch`. Folding into row 15 / row 12 would mean two distinct lib variants render under one SPEC stem (violates the SPEC §B.2.5 stems-table contract). | §3.2 mapping table + §5 P2.2 SPEC patches |
| **Q4** | Minimum-viable manual stub at P2.3; canonical chapter at P3. `lint.sh:84` flag-coverage step does `grep -oE '--[a-z][a-z0-9-]+'` on `--help` then asserts each appears in the chapter — a 30-line stub listing each flag string is sufficient. | §5 P2.3 row |
| **Q5** | Extract `emit_world_readable_advisory` to `crate::secret_advisory::warn_if_world_readable` at P2.2 GREEN. Three call sites is the threshold (`cmd/seed_xor.rs:425`, `cmd/final_word.rs:178`, `cmd/slip39.rs` NEW). The `secret_advisory` module's doc-comment grows to delineate "argv-leakage class" vs "path-permission class". | §3.3 row 4 + §5 P2.2 GREEN + §6 risk 6 |
| **Q6** | O(N) per-share pinning is the right answer; SPEC §2.1 paragraph "Per-share output pin discipline (O(1), not O(N))" is structurally unachievable with `Vec<Zeroizing<String>>` (Vec backing holds non-secret String headers; per-share UTF-8 bytes live in separate allocations). SPEC §2.1 patch + SPEC §5 row-12 cite removal queued for P2.2 GREEN. | §3.5 + §5 P2.2 SPEC patches |
| **Q7** | Pre-RED probe COMPLETE (already executed against the LOCK-clean tree at `81488e3`): `seed-xor` renders as `{name: "seed-xor", flags: [], positionals: []}` — pre-existing v0.12.0 gap. Fix via flattening (`seed-xor-split`, `seed-xor-combine`, `slip39-split`, `slip39-combine`) at the schema-emit layer; schema_version stays at 1; sibling-repo `mnemonic-gui` FOLLOWUP filed at P2.1 RED. Repairs both v0.12.0 (seed-xor) AND v0.13.0 (slip39) at once. | §4.2 + §5 P2.1 RED + §6 risk 3 |

## §8 No new error variants

P2 introduces no new `Slip39Error` variants — the lib surface was finalized at P1c-E.1 (21 variants). The CLI handler renders existing variants + adds CLI-only refusals (rows 17, 18) at the `ToolkitError::BadInput(stem)` boundary. SPEC §2.5 row 24 (per Q3 fold) is a SPEC-only patch (no new variant — the existing `MemberThresholdMismatch` variant covers it).

## §9 Verification gates

**P2.1 LOCK criteria:**
- `cargo build` succeeds with the new `Slip39` enum variant + `Slip39Args` struct.
- `cargo run -- slip39 --help` enumerates `split` + `combine` sub-subcommands.
- `cargo run -- slip39 split --help` and `... combine --help` enumerate the SPEC §2.2 flag tables (every flag visible).
- `cargo run -- slip39 split <minimal-args>` exits with code 1 + a clean stem `"slip39 split: P2.1 stub — full impl ships at P2.2"` (NOT an `unimplemented!()` panic, which is exit 134; the stub returns `ToolkitError::BadInput(...)` per the SPEC §2.4 exit-1 mapping).
- `tests/cli_gui_schema.rs::gui_schema_lists_all_ten_subcommands` passes — the alphabetical list is `["bundle", "convert", "derive-child", "export-wallet", "final-word", "seed-xor-combine", "seed-xor-split", "slip39-combine", "slip39-split", "verify-bundle"]` (per Q7 flattening fold; seed-xor flattens too as the bug-fix bonus).
- `cargo run -- gui-schema | jq '.subcommands[] | select(.name == "slip39-split")'` returns a non-empty `flags` array (proves the flattening landed and the slip39 stub's flags surface in the schema).
- `slip39-cli-extendable-flag` FOLLOWUP filed at `design/FOLLOWUPS.md` (Note 3 fold body); cross-citing `slip39-shamir-secret-sharing` at `:1039`.
- `slip39-gui-schema-flattening-companion` FOLLOWUP filed in `mnemonic-gui` repo (per Q7 fold).
- Clippy `--all-targets -- -D warnings` clean.

**P2.2 LOCK criteria:**
- All 5 `cli_slip39_*.rs` test files pass — SPEC §4 G3 (plain stdout shape; `cli_slip39_happy_paths.rs`), G4 (JSON envelope SHA-pin via env-var wedge; `cli_slip39_json.rs` — including the always-on insecurity advisory pin), G5 (refusals; `cli_slip39_refusals.rs` 24 rows per Q3 fold), G9 (iteration-exponent threshold; in `cli_slip39_advisories.rs`).
- `lint_argv_secret_flags.rs` strict count `assert_eq!(rows, 28)` passes; per-row evidence anchors all hit (5 new + 23 existing).
- `lint_zeroize_discipline.rs` per-row evidence anchor for the new `cmd/slip39.rs` row hits.
- `secret_advisory::warn_if_world_readable` extracted (Q5 fold); `cmd/seed_xor.rs:425-445` deleted; `cmd/final_word.rs:178-197` updated to call the helper; `cmd/slip39.rs` calls the helper.
- SPEC patches landed in lockstep (single commit at P2.2 GREEN): §2.1 per-share-pin O(N) clarification + §2.5 row 24 add (`MemberThresholdMismatch`) + §4 G4 env-var language + §B.2 advisory row + NEW §B.3 test-only-env-vars subsection + §4 G6 count `23 → 28` update.
- Full project `cargo test --tests` clean (the 899-test baseline + the ~55–70 new tests = ~960 tests; 0 fail / 8 ignored).
- Clippy `--all-targets -- -D warnings` clean.

**P2.3 LOCK criteria:**
- `docs/manual/tests/cli-subcommands.list` contains `mnemonic slip39 split` + `mnemonic slip39 combine`.
- `make -C docs/manual lint MNEMONIC_BIN=$(cargo path) ...` passes — `flag-coverage` step finds every `--help`-emitted flag in `41-mnemonic.md`.
- `41-mnemonic.md` chapter intro updated: "8 subcommands" → "9 subcommands" + cross-link list updated.

**Out of P2 scope (P3 / PE):**
- G7 manual chapter prose canonical content (P3).
- G8 Trezor interop smoke recipe (P3).
- PE release rollup (Cargo.toml bump, CHANGELOG, tag `mnemonic-toolkit-v0.13.0`).
- `slip39-shamir-secret-sharing` FOLLOWUP closure (PE — when the tag ships, this is the formal close).
- `slip39-cli-extendable-flag` FOLLOWUP closure (deferred to v0.14 cycle).
