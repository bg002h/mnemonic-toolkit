# Phase E — release prep + final v0.2.0 Review — r1

**Date:** 2026-05-05
**Commit under review:** `f0d9753` (parent: `dc6520c`)
**Local tag:** `mnemonic-toolkit-v0.2.0` (annotated; not pushed)
**Reviewer:** opus phase-review

## Verdict

1 critical / 0 important / 2 low / 0 nits — C-1 fixed inline post-r1.

✅ **v0.2.0 cleared for user-gated push + `gh release create mnemonic-toolkit-v0.2.0`** after the C-1 fix lands.

## Critical (FIXED inline post-r1)

### C-1: CHANGELOG test count factually wrong

**File:** `CHANGELOG.md:38`

`CHANGELOG.md` stated "76 unit + 54 integration = 130 total" but `cargo test --workspace` reports 107 (76 unit + 31 integration). The 54 inflated number conflated parametric-cell iterations with `#[test]` function counts.

**Fix (applied):** Replaced with `76 unit + 31 integration test functions = 107 total (cargo test --workspace). The 31 integration functions cover ~54 parametric cells across 13 test binaries.`

## Important

(none)

## Low / Nit

- **L-1**: Phase C carryovers `dead-assert-tautological` and `dead-inner-guard-bundle-watch-only` remain open at v0.2 tier in FOLLOWUPS.md but were not fixed during v0.2. Re-tier to `v0.2-nice-to-have` or `v0.3` post-release. Defer.
- **L-2**: Watch-only multisig + verify-bundle multisig integration tests not added (Phase E implementer scoped them to v0.3 follow-up). Phase C smoke tests + Phase D 3+6N enumeration unit tests cover the verify-bundle multisig path adequately for v0.2; integration parametric tests that exercise the full CLI round-trip for multisig watch-only are deferred. Acceptable for v0.2.0; record in FOLLOWUPS.

## Verified

- **Tag:** `mnemonic-toolkit-v0.2.0` annotated tag confirmed local-only (not pushed).
- **Cargo.toml:** `version = "0.2.0"` at `crates/mnemonic-toolkit/Cargo.toml`. All required crates.io metadata fields preserved from v0.1.
- **CHANGELOG:** date 2026-05-05; 5 features listed; wire-bit-identical scope correct; v0.1 SHA pin retired; v0.2 SHA pin `a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171` recorded with `shasum` reproduction command; `cargo publish` gating note preserved.
- **README:** multisig (full + watch-only), `--privacy-preserving`, `--self-check`, `--account 5` examples present.
- **SPEC §9.4:** placeholders fully back-filled with concrete r1/r2/r3 architect findings (I1–I3, L2–L3, I-A, I-B, L-A, L-B, N-1, Q1–Q12 closure proof table, §9.4.1 wire-bit-identical claim scope).
- **34 fixtures at `tests/vectors/v0_2/`:** count + content verified (multisig fixtures show 3 mk1 card-sets + md1; privacy fixture distinct from baseline; self-check fixture non-empty).
- **5 new integration test files** present with #[test] functions.
- **Test counts:** 107 passing (`cargo test --workspace`).
- **clippy + fmt:** clean at HEAD.
- **v0.1 wire-bit-identical regression:** 16/16 PASS at HEAD.
- **Phase coverage map:** all Phase A-D review findings resolved inline; Phase C deferred items (L-1, L-2 check enumeration) resolved at Phase D; SPEC r3 carryovers resolved at Phase D.2 (§2.2.2 4-check enumeration) and E.3 (§9.4 backfill).
- **Carryover items acknowledged:** watch-only multisig integration tests deferred to v0.3; K-of-N deferred (gates on ms-codec v0.2); hash-locked descriptors deferred v0.3+; recovery flow + `--output <dir>` deferred; `hex` dep unused retained per `feedback_dont_drop_reserved_deps`.

## v0.2.0 release-time gates

After C-1 CHANGELOG fix:
- ✅ All tests passing (107).
- ✅ clippy + fmt clean.
- ✅ v0.1 single-sig 16-cell wire-bit-identical regression PASS.
- ✅ Cargo.toml `version = "0.2.0"`.
- ✅ CHANGELOG documents v0.1 SHA pin retirement + v0.2 SHA pin.
- ✅ `mnemonic-toolkit-v0.2.0` annotated tag (local; push gated).
- ✅ `cargo publish` remains blocked until ms-codec / mk-codec / md-codec on crates.io.

User-gated next steps:
1. `git push origin master` (push v0.2 design + impl commits).
2. `git push origin mnemonic-toolkit-v0.2.0` (push tag).
3. `gh release create mnemonic-toolkit-v0.2.0` (GitHub release).
4. `cargo publish` deferred (siblings still git-deps).
