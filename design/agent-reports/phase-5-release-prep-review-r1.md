# Phase 5 Release-Prep + Final v0.1.0 Review — r1

**Date:** 2026-05-04
**Commit under review:** `f2bd20a` (release commit)
**Tag:** `mnemonic-toolkit-v0.1.0` (local; user-gated push)
**Reviewer:** opus phase-review

## Verdict

0 critical / 0 important / 3 low / 3 nits

✅ **Phase 5 r1 + final v0.1 review terminator reached — v0.1.0 cleared for user-gated push/release.**

## Critical / Important

(none)

## Phase 2 byte-determinism fix verification

The implementer caught a Phase 2 byte-determinism bug during fixture generation that BOTH Phase 2 r1 and r2 reviews missed. Without the fix, the v0.1 SHA pin would have been meaningless.

- `synthesize::derive_mk1_chunk_set_id` is pure (bit-shift/OR over `stub[0..3]`); no CSPRNG.
- `mk_codec::encode_with_chunk_set_id` is the public deterministic-CSI entry point at `mk-codec/src/key_card.rs:109`, re-exported from lib.rs.
- `ms_codec::encode` and `md_codec::chunk::split` are deterministic (BCH-only, no CSPRNG).
- New unit test `mk1_chunk_set_id_is_deterministic_across_runs` runs encode twice; asserts mk1 + md1 + ms1 byte-equality.

Fix is sound. **Note:** Phase 2 review missed this — recommend FOLLOWUPS entry on review-process improvement (spike before locking byte-exact-fixture invariants).

## Low (deferred to design/FOLLOWUPS.md)

- **L-1:** `ToolkitError::kind()` dead-code in v0.1 (JSON error envelope not yet wired). #[allow(dead_code)] present; reactivated in v0.2.
- **L-2:** `format::chunk_mk1` reserved-stub never called (bundle.rs uses `chunk_5char` directly for mk1). Forward-reserve for mk-codec swap.
- **L-3:** `cli_bundle_watch_only.rs` hardcodes fingerprint `5436d724` rather than reading from decoded mk1. Two-place edit risk for vector changes.

## Nit (deferred)

- **N-1:** CHANGELOG SHA pin doesn't document the exact reproduction command (`shasum -a 256 *.txt | sort | shasum -a 256`). Verifiers may need to guess.
- **N-2:** `ToolkitError::BundleMismatch` is dead-code at runtime (only unit-tested). Forward-reserve per SPEC §6.1.
- **N-3:** `cli_mode_violations.rs` test names say "byte_exact" but use `str::contains`. Naming-precision nit.

## Verified

- **Tag:** `mnemonic-toolkit-v0.1.0` annotated, local only.
- **Cargo.toml:** version 0.1.0, publish=false removed, all crates.io metadata present (description/documentation/readme/keywords/categories), sibling git-deps pinned at expected tags.
- **CHANGELOG.md:** date 2026-05-04, SHA pin `81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6`, test counts + "cargo publish blocked" note present.
- **README.md:** install + quickstart + template/network table + engraving caveats + SPEC §7.4 wordlist-language hazard + sibling pointers + license.
- **Integration tests:** 7 files / 17 tests; all use `assert_cmd::Command::cargo_bin`; no network/timing/random; SPEC §2.2.2 stderr warning asserted; mode-violation exit codes (1/2/64) all asserted.
- **SPEC coverage map:** §2.1 (bundle), §2.2 (verify-bundle), §2.3 (--help), §5.1 (multi-section), §5.3/§5.4 (JSON), §6.6 (mode violations), §6.1 (exit codes) — all covered.
- **Carryovers from Phases 1-4:** none were altered by Phase 5; all remain in deferred FOLLOWUPS state.

## Smoke checks

- `cargo test --workspace`: **71 passed** (54 unit + 17 integration); 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean.
- SHA pin reproduces.

## v0.1.0 release readiness

**Yes — cleared.** 0C/0I. 3 low + 3 nits are v0.2+ material. The Phase 2 byte-determinism fix is sound; the 16-fixture SHA pin is meaningful; integration coverage maps cleanly to SPEC.

User-gated next steps:
1. `git push origin master` (push 14 phase commits).
2. `git push origin mnemonic-toolkit-v0.1.0` (push tag).
3. `gh release create mnemonic-toolkit-v0.1.0 --notes-file <CHANGELOG excerpt>`.
4. `cargo publish` deferred until ms-codec / mk-codec / md-codec land on crates.io.
