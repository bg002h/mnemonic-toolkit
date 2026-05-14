# v0.11.0 P0 R0 — final-word SPEC + plan reviewer-loop (Opus)

**Date:** 2026-05-13.
**Reviewer:** Opus (per `feedback_opus_primary_review_agent`).
**Scope:** R0 design-lock on the v0.11.0 BIP-39 final-word completer SPEC + plan.
**Verdict:** LOCK across 3 rounds. R1 LOCK clean.

## Artifacts reviewed

- SPEC: `design/SPEC_final_word_v0_11_0.md`
- Plan + brainstorm: `~/.claude/plans/radiant-seeking-teacup.md`
- FOLLOWUP entry: `design/FOLLOWUPS.md` `bip39-final-word-completer`

## Convergence chain

| Round | Verdict | Findings | Time-to-fold |
|---|---|---|---|
| R0 round 1 | REWORK | 4 Critical + 6 Important + 3 nits | ~30 min |
| R0 round 2 | REWORK | 1 Critical + 3 Important (residuals from incomplete C1 fold + new run-sig finding) | ~10 min |
| R0 round 3 | LOCK | 0 Critical / 0 Important | confirmation pass |

## Round 1 findings (all folded)

**C1 — clap-unsatisfiable `required` + `conflicts_with`.** `--from required = true` + `--phrase-stdin conflicts_with = "from"` cannot both be satisfied. Resolution: drop `--phrase-stdin` entirely; `--from phrase=-` is the sole stdin path (mirrors `convert --from phrase=-` precedent; single secret input justifies single route). Confidence 95.

**C2 — `bip39::Language::word_list()` (with underscore), NOT `.wordlist()`.** API name drift; existing toolkit at `src/wordlists/mod.rs:86` uses the correct spelling. Algorithm block in §2.1 and plan §2.1 both corrected. Confidence 95.

**C3 — `FromInput` is a struct (not enum), at `src/cmd/convert.rs:121` (not `src/cmd/`).** Needs `value_parser = parse_from_input`. Runtime refusal via `if args.from.node != NodeType::Phrase`. Confidence 95.

**C4 — `secret_advisory::secret_in_argv_warning(stderr, flag, alternative)` is the real API**, not `warn_inline_secret`. Confidence 100.

**I1 — `convert.rs:796` is Electrum SeedVersion, NOT a stdout-on-TTY advisory.** Stdout-on-TTY is a NEW advisory class for v0.11.0; no toolkit precedent. Confidence 90.

**I2 — `std::io::IsTerminal` (stable 1.70; MSRV 1.85), not `atty`** (unmaintained per RUSTSEC-2021-0145). Confidence 90.

**I3 — `CliLanguage::human_name() -> &'static str`** (at `src/language.rs:26-39`), NOT a `Display` impl (no Display impl exists). Confidence 85.

**I4 — JSON envelope deviations.** Drop `feature` namespace tag (existing envelopes discriminate by binary + mode/template). Start `schema_version` at `"1"` not `"0"` (existing `bundle --json` is at `"4"`, `convert --json` at `"1"`; starting at `"0"` reverses the project pattern). Confidence 80.

**I5 — `PermissionsExt::mode()` is Unix-only**; needs `#[cfg(unix)]` gate. Confidence 85.

**I7 — refusal class for `--from phrase=-` + `--phrase-stdin` combined.** N/A after C1 collapses to single stdin route.

**N1 — per-N canonical zero-entropy ends differ** (N=12→`about`, N=18→`agent`, N=24→`art`; N=15+N=21 use non-zero-entropy). G1 narrative was incorrectly using `art` for all 5 N.

**N2 — `beef × 12` label inconsistency.** Partial is `beef × 11`; target is `beef × 12`. Renamed to `beef × 12 target` / `partial beef × 11`.

**N3 — chapter number `47-` to verify** at P3 dispatch (not a P0 blocker).

## Round 2 findings (all folded)

**Critical — `run()` signature `stdin: Option<&str>` is incompatible with `read_stdin_to_string<R: Read>(&mut R)`.** Existing pattern (`convert.rs:597`) is `run<R: Read, W: Write, E: Write>(args, stdin: &mut R, stdout: &mut W, stderr: &mut E)`. Both SPEC and plan updated to match.

**Important — residual `--phrase-stdin` references in plan** (4 sites: test-file naming line, lint-evidence array, smoke-test bash invocation, advisory message body). All updated to `--from phrase=-` form.

**Important — §2.6 advisory body text** still cited `pipe via --phrase-stdin`; updated to `pipe via --from phrase=-` (matching what `secret_in_argv_warning` actually emits per the C4-corrected call).

**Minor — plan §5 cross-ref drift** `src/wordlists/mod.rs:17` (doc-comment, not call) → `:86` (actual call site). SPEC had it right; plan now mirrors.

## Round 3 findings

**None.** Verbatim verdict: "All forward-looking sections (§2, §4, test matrix, smoke-tests, advisory bodies) consistently use the corrected forms. Verdict: LOCK."

## Algorithm-correctness math (independently verified at R0 round 1)

For each N ∈ {12, 15, 18, 21, 24}: N × 11 = entropy + CS bits per BIP-39. The Nth word's 11 bits decompose as `(11 - CS) entropy-tail bits || CS checksum bits`. For each of 2^(11-CS) entropy-tail values, SHA-256 fully determines the CS checksum bits → exactly 2^(11-CS) valid completions per partial. Numerically: 128 / 64 / 32 / 16 / 8.

Naïve-enumeration vs bit-math direct-derivation produce IDENTICAL sets (both iterate the same 2^(11-CS) entropy-tail values and filter by SHA-derived checksum equality). The SPEC's rejection of bit-math in §1 is on simplicity/risk grounds, not correctness. ✓

## P0 ship gate

R0 LOCK clean → P0 ship gate satisfied. Commit: `docs(spec): v0.11.0 P0 — final-word SPEC + plan + R0 LOCK`.

Next phase: P1 (library impl + RED → GREEN + R1 reviewer-loop), with its own architect-review convergence per the per-phase discipline the user explicitly reaffirmed.
