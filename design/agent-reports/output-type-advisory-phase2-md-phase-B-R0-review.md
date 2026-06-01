# Phase B (md-cli) per-phase R0 review — output-type advisory Phase 2

> Opus architect per-phase review of `git diff c599292..6a195a0` (B1 `9a60f1b`, B2 `aba8e35`, B3 `6a195a0`) on branch `output-class-advisory-phase2`. R0 RED (1 Important: CHANGELOG) → fold (`fbac307`) → R1 GREEN (addendum below). Also persisted in `descriptor-mnemonic/design/agent-reports/`.

## R0 Verdict: RED (0C / 1I / 2M)

Code and tests are correct and green on both feature sets. The one blocker was the shipped CHANGELOG entry, which mischaracterized the release as test-only and omitted the actual behavior change.

## Critical
None.

## Important
**I1 — v0.6.2 CHANGELOG entry inaccurate; omits the headline behavior change (`CHANGELOG.md`).** The entry described only B3's slice ("3 inert-subcommand negative cells … Test-only, no binary change"), but the cumulative 0.6.1→0.6.2 release adds `output_advisory.rs` + 13 emit sites across 7 handlers — `md` now prints a stderr note classifying stdout. Fix: rewrite [0.6.2] to lead with the advisory behavior for the whole phase (mirror ms-cli `[0.5.1]`), demote the inert cells to a sub-bullet, drop "no binary change." **→ FOLDED in `fbac307`.**

## Minor
- **M1** — Missing the spec-named `md repair` exit-2 negative cell. Behavior correct empirically (exit 2, zero advisory lines). **→ FOLDED in `fbac307` (`repair_error_path_emits_no_advisory`).**
- **M2** — `byte_parity_advisory_lines` is self-tautological (const vs inline copy in the same file); real drift guard is the positive cells. Cosmetic. **Deferred to the constellation-wide FOLLOWUP (same as Phase A M1).**

## Coverage (all 10 subcommands) — 7 emit / 3 inert
- **Template** (×2 each, json early-return + text — both verified): decode, encode, inspect, bytecode, compile.
- **Template** (single emit at `repair.rs:153` success fall-through; `Ok(2)` fail path at `:121` emits nothing — empirically confirmed): repair.
- **WatchOnly** (×2; cells assert WatchOnly "not template"): address.
- **inert** (assert ABSENCE of all 3 lines): verify, vectors (tempdir, no cwd pollution), gui-schema.
13 emit call sites total; none in the 3 inert handlers.

## Verification
- **Byte-parity**: 3 literals byte-identical across md-cli / toolkit / ms-cli (`\u{2014}` == `—` == UTF-8 `E2 80 94`). Enum derives `Debug,Clone,Copy,PartialEq,Eq` — no Ord, no `worst_class_on_stdout`/`card_kind_class`; `#[allow(dead_code)]` load-bearing.
- **Tests**: default `cli_output_class` 15/0 (→16/0 post-fold), `--features cli-compiler` 17/0 (→18/0). Clippy `-D warnings` both feature sets: 0 warnings (forced fresh via `cargo clean -p md-cli`).
- **Scope**: 13 files; `main.rs` is `+mod output_advisory;` only; zero behavior change to existing logic; no orphaned imports.
- **SemVer**: 0.6.1→0.6.2 PATCH correct.
- **CI note**: `ci.yml` runs default features only → the 2 compile cells aren't CI-gated (matches plan; verified passing under manual feature build).

---

## R1 (fold verification — `fbac307`): GREEN

Folded I1 + M1; M2 deferred to FOLLOWUP. Verified: CHANGELOG [0.6.2] now leads with the advisory behavior, names all 7 handlers correctly, drops "test-only/no binary change", matches the diff. `repair_error_path_emits_no_advisory` feeds an irrecoverable md1 → asserts exit `Some(2)` + `assert_no_advisory`. Fold scope = only `CHANGELOG.md` + the test file (no handler/module/version change; still 0.6.2). Default 16/16, cli-compiler 18/18, clippy clean. **Phase B GREEN — cleared to tag (deferred).**
