# v0.5.1 Phase Release — code-architect r1

**Outcome:** 0C/0I/0L/1N — APPROVED.

## Scope reviewed
4 staged files for Phase R (release prep):
- `crates/mnemonic-toolkit/Cargo.toml` — version bump 0.5.0 → 0.5.1.
- `Cargo.lock` — workspace package bumped to 0.5.1.
- `CHANGELOG.md` — new `[0.5.1] — 2026-05-06` entry prepended.
- `design/FOLLOWUPS.md` — `legacy-cli-flag-deletion` + `legacy-flag-deprecation` marked resolved (cite Commit 1 hash `d782a2d`).

## Plan-fidelity verification
1. **Version-bump alignment:** Cargo.toml `0.5.1`, Cargo.lock workspace package `0.5.1`, CHANGELOG header `[0.5.1]`, FOLLOWUPS resolution citations point at `d782a2d`. All four aligned.
2. **CHANGELOG accuracy:** 6 flags / 2 shims / 9 guards / 11 consts (9 + 2 dead extras: `ACCOUNT_INCOMPATIBLE_TEMPLATE`, `DESCRIPTOR_WITH_COSIGNER_COUNT`) / 3 retained / ~584-line deletions / new violations file with 6 tests / 13 consumer files rewritten / path-defaulting refinement — all consistent with the plan inventory and Commits `d782a2d` + `a5391b9`.
3. **CHANGELOG structure** mirrors v0.5.0's entry shape (What's new / Breaking changes / Test corpus / Carry-forward / Architect review reports).
4. **FOLLOWUPS wording** — both entries marked `resolved by v0.5.1 commit d782a2d` with succinct justification.
5. **Wire-bit-identical claim** sound (no synthesis-path changes in v0.5.1 — only flag deletion + dispatch rewire onto the existing unified path which v0.5.0 already used).
6. **Test-count arithmetic** (236 − 6 = 230 lib; 44 integration) confirmed by direct grep against the working tree.
7. **Architect-review report paths** cited in CHANGELOG (`v0_5_1_phase_atomic_r1.md`, `v0_5_1_phase_spec_r1.md`) both exist on disk.

## Nit (non-blocking)

- **CHANGELOG unit-terminology inconsistency across versions.** v0.5.0 entry uses "22 integration suites" (counting test files); v0.5.1 entry uses "44 integration tests" (counting individual `#[test]` functions). Both numbers are internally correct for their chosen unit (14 files × avg ~3 tests = 44; 22 files existed pre-v0.5.1). No reader-confusion risk since each entry is self-consistent. Future entries should pick one unit and stick with it.

**Verdict:** Ready to commit, tag, push, release.
