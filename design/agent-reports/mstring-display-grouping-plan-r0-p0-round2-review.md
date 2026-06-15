# Plan-R0 (P0 foundation) round 2 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md. Dispatched via Agent (feature-dev:code-architect,
> Opus 4.8). **Verdict: GREEN — 0 Critical / 0 Important.** Plan SHA at review:
> toolkit `d7877bb`. The P0 plan-R0 gate is MET.

---

## Fold verification (round-1 findings)

**C1/C2 — CORRECT + COMPLETE.** Fns now in `src/display_grouping.rs`, declared UNCONDITIONAL `pub mod display_grouping;`. `lib.rs:67-97` is the always-on `pub mod` cluster; the first `#[cfg(fuzzing)]` is line 129, so inserting `pub mod display_grouping;` before it (or among 67-97) is uncgated. `format` stays `#[cfg(fuzzing)]`-gated at lib.rs:143, untouched. Task 3's `use mnemonic_toolkit::display_grouping::{…}` resolves in normal `cargo test` — confirmed by the existing `lib_slip39_rs1024.rs:34` (`use mnemonic_toolkit::slip39::{…}`) pattern for unconditional lib modules. `--lib display_grouping` will select the 10 unit tests (module is in the lib), not zero.

**I1 — CORRECT.** Task 2 Steps 2/4 use `--lib display_grouping`; valid now the module is in the lib.

**m1 — CORRECT.** `sep_char("none")` comment reworded accurately.

**m2 — CORRECT.** Task 3 Step 2 uses `grep -n -B1` and checks for no `#[cfg]` above — a genuine reachability check.

## New-drift sweep (all clean)
1. Stubs (`unimplemented!("stub")`, `_`-prefixed params) compile + fail at runtime as claimed; replaced before commit so the clippy `-D warnings` gate (rust.yml:199) only sees the final impl. No workspace `#![deny(warnings)]`; crate `[lints.rust]` only `unexpected_cfgs=warn`. No defect.
2. All 20 TSV rows hand-evaluated against SPEC §3.1/§3.2 — every `expected` correct (incl. `group_size==len`→no trailing sep; `ms1qpzry9x8gf`,4,comma→`ms1q,pzry,9x8g,f`; CR/LF both `is_whitespace`).
3. `decode()` sentinel handling sound; `<empty>` early-return vs `.replace()` chain mutually exclusive; header assert matches byte-for-byte.
4. `CARGO_MANIFEST_DIR`/../../design path depth correct — confirmed by existing `design_artifacts_presence.rs:13` using the identical pattern.
5. P0 additive-only: no existing test/output change; rustfmt-1.95.0 clean; no cspell scan of `design/*.tsv` (cspell configs scoped to docs/*).
6. `pub` lib items don't trigger `dead_code` (potentially externally consumed) — no denial.
7. TSV format portable for P1–P3 (tab-split + replace decoder, any language).

Minor style note (non-defect): `count >= 20` is exactly tight (20 rows); correct, just no slack.

## Verdict
GREEN — 0 Critical / 0 Important. P0 plan is implementable exactly as written.
