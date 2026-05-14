# Phase 3a R1 Architect Review — Cycle B

**Reviewer:** Opus 4.7 (1M context), `feature-dev:code-reviewer`
**Date:** 2026-05-13
**Scope:** 7 commits between `7cb2527..4a5335a` (Cycle B Phase 3a Path B-lite implementation)
**Method:** Source-ground-truth verification per `feedback_r0_must_read_source_off_by_n` for every load-bearing claim.

## Verdict

**CLEAR** — 0 Critical / 0 Important — Phase 3a ships.

All v3-fold proposal LOCK requirements are met. The implementation correctly carved out the field-type migration to v0.10.1 and shipped only the struct-sibling pin work with full threat-model coverage at all 5 sites.

---

## Verification matrix

### A. NO field-type migrations leaked — VERIFIED

| Claim | Source | Status |
|---|---|---|
| `ResolvedSlot.entropy` is STILL `Option<Vec<u8>>` | `synthesize.rs:587` | Verified: `pub entropy: Option<Vec<u8>>` |
| `DerivedAccount.entropy` is STILL `Vec<u8>` | `derive.rs:22` | Verified: `pub entropy: Vec<u8>` |
| `impl Drop for DerivedAccount` STILL PRESENT | `derive.rs:58-67` | Verified: scrub via `self.entropy.zeroize()` |
| `into_parts()` body UNCHANGED | `derive.rs:45-55` | Verified: `mem::take(&mut self.entropy)` (not `&mut *self.entropy`) |
| `tests/lint_zeroize_discipline.rs` UNTOUCHED | lines 109-113 deferred-FOLLOWUP comment block intact | Verified |
| `tests/lint_safety_third_party_blocked.rs` UNTOUCHED | file present | Verified |
| `tests/lint_argv_secret_flags.rs` UNTOUCHED | file present | Verified |

### B. `_entropy_pin` fields added correctly — VERIFIED

- `ResolvedSlot._entropy_pin: Option<Arc<PinnedPageRange>>` declared LAST after `master_xpub` (`synthesize.rs:604`). Pub. ✓
- `DerivedAccount._entropy_pin: PinnedPageRange` declared LAST after `account_path` (`derive.rs:34`). Pub. ✓

### C. Arc-wrap correctness — VERIFIED

- `Arc<PinnedPageRange>` on `ResolvedSlot`; Clone-derive preserved (`synthesize.rs:580` `#[derive(Debug, Clone)]`).
- Plain `PinnedPageRange` on `DerivedAccount` (no Clone derive at `derive.rs:20`; consumed via `into_parts`).
- `PinnedPageRange` has `#[derive(Debug)]` (`mlock.rs:57`) so the `Debug` derive on both structs continues to compile. ✓

### D. ALL ctor sites populated — VERIFIED (12 ResolvedSlot/CosignerKeyInfo + 1 DerivedAccount = 13 total)

| Site | File:line | Arm | Pin populated correctly |
|---|---|---|---|
| 1 | `synthesize.rs:1052` | test fixture (`construct_test_descriptor` style) | `_entropy_pin: None` ✓ |
| 2 | `synthesize.rs:1206` | `unified_fixture` test | `Some(Arc::new(pin))` conditional on `entropy_field` ✓ |
| 3 | `parse_descriptor.rs:1169` | `bind_full_mode` helper | `_entropy_pin: None` ✓ (entropy: None — descriptor-mode bridging sets entropy separately) |
| 4 | `parse_descriptor.rs:1733` | `cinfo` test | `_entropy_pin: None` ✓ |
| 5 | `parse_descriptor.rs:1748` | `cinfo_raw` test | `_entropy_pin: None` ✓ |
| 6 | `cmd/verify_bundle.rs:489` | resolved → cosigner conversion | `Some(Arc::new(pin))` conditional on `entropy` ✓ |
| 7 | `cmd/bundle.rs:364` | Phrase arm | `Some(Arc::new(pin))` real entropy; pin captured at `:363` BEFORE `entropy` move on `:369` ✓ |
| 8 | `cmd/bundle.rs:434` | Xpub watch-only | `_entropy_pin: None` ✓ |
| 9 | `cmd/bundle.rs:468` | Entropy arm | `Some(Arc::new(pin))` real entropy; pin captured at `:467` BEFORE `entropy_bytes` move on `:473` ✓ |
| 10 | `cmd/bundle.rs:511` | Wif watch-only | `_entropy_pin: None` ✓ |
| 11 | `cmd/bundle.rs:1042` | descriptor-mode cosigner | `Some(Arc::new(pin))` conditional on `ent_opt`; pin at `:1041` ✓ |
| 12 | `cmd/bundle.rs:1092` | resolved_slots reconstruction | `Some(Arc::new(pin))` conditional on `i==0 && entropy_at_0.is_some()`; pin at `:1091` ✓ |
| 13 | `derive_slot.rs:83` | DerivedAccount ctor | `_entropy_pin: entropy_pin` from `:82` BEFORE `entropy_bytes` move on `:84` ✓ |

For real-entropy ctors, pin is constructed BEFORE the buffer is moved into the struct (Vec is move-by-pointer-copy; heap data pointer is stable across move).

For the cosigner-bridging ctor at `cmd/bundle.rs:1090-1099`: pin correctly conditional on `entropy.is_some()` via `as_ref().map(...)`. ✓

### E. Site 1 per-handler anchors — VERIFIED

| Handler | Anchor location | Verified |
|---|---|---|
| `cmd/bundle.rs::run` | Lines 123-134, AFTER `&synthetic_args` re-binding (`:115-121`) | ✓ Pins `args.passphrase` (Option) and each `args.slot[i].value` |
| `cmd/verify_bundle.rs::run` | Lines 138-150, AFTER both `apply_stdin_substitutions` AND `load_bundle_json_into_args` re-bindings (`:119-136`) | ✓ Pins `args.passphrase` and `args.slot[i].value` |
| `cmd/convert.rs::run` | Lines 673-685, AFTER `effective_passphrase` (`:652`) + `effective_bip38_passphrase` (`:660`) + `primary_value` (`:667`) bound | ✓ Pins all 3 |
| `cmd/derive_child.rs::run` | Lines 124-131, AFTER `from_value` (`:98`) + `stdin_passphrase` (`:108`) bound | ✓ Pins both |

All anchors land in v3-fold §3.1 lock positions.

### F. Site 4 — all 7 bip85 functions — VERIFIED

`let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);` immediately after `derive_entropy(...)?` in:

| Line | Function |
|---|---|
| 84 | `format_bip39_phrase` |
| 110 | `format_hd_seed_wif` |
| 138 | `format_xprv_child` |
| 170 | `format_hex_bytes` |
| 188 | `format_password_base64` |
| 203 | `format_password_base85` |
| 241 | `format_dice_rolls` |

Binding name is `_entropy_pin` (underscore-prefixed but not just `_`) → lives until end-of-function scope, not dropped immediately. ✓

### G. main.rs report_at_exit() — VERIFIED

`mnemonic_toolkit::mlock::report_at_exit()` at `main.rs:101` runs AFTER `let exit = match result { Ok(...) | Err(...) }` block (`:89-96`), then returns `exit`. Both Ok and Err paths flow through the report. The clap-parse-error early-return path at `:60-62` is intentionally skipped per SPEC §4 P3a (no mlock callsite reached before parse). ✓

### H. CI delta — VERIFIED

`.github/workflows/rust.yml` lines 97-116 add the `test-release-mlock-einval` job:
- Linux-only (`runs-on: ubuntu-latest`). ✓
- Uses `cargo test --release -p mnemonic-toolkit --lib mlock::tests::g2_3 -- --include-ignored`. ✓
- Env `MNEMONIC_TEST_MLOCK_FAIL_MODE: "einval"` (defensively quoted). ✓
- All other YAML string values are quoted defensively. ✓
- Tracks the `feedback_r2_blocking_vs_cosmetic_gate` lessons from commits `076e462` + `30cd0e6`. ✓

### I. Cycle A discipline preserved — VERIFIED

- `tests/lint_zeroize_discipline.rs` — UNCHANGED (lines 109-113 still cite `resolved-slot-entropy-zeroizing-field`).
- `tests/lint_safety_third_party_blocked.rs` — UNCHANGED.
- `tests/lint_argv_secret_flags.rs` — UNCHANGED.
- `impl Drop for DerivedAccount` at `derive.rs:58-67` — PRESERVED.
- `derive_master_seed` Zeroizing helper at `derive_slot.rs:32` — PRESERVED.

### J. Test approach deviation justified — CORRECT

The deviation from v3-fold proposal Step 2.5 (no `tests/cli_mlock_g2_subprocess.rs`) is correctly grounded in RFC 1604 (cfg(test) is per-crate-not-per-build). The chosen alternative is sound:

- New `pub fn attempts_for_test()` at `mlock.rs:240` exposes the unconditional `record_attempt()` counter (line 98 — fires before `sys_mlock_attempt`).
- Tests assert `attempts_for_test() > baseline` instead of `failure_count_for_test() > baseline`.
- This works for binary-crate tests because `record_attempt()` runs in BOTH cfg(test) and production paths uniformly — the attempt-counter is platform-uniform observability.
- All 4 in-source `path_b_lite_pin_tests` mods (`bip85.rs:418`, `synthesize.rs:1365`, `derive.rs:238`, `cmd/derive_child.rs:367`) use this pattern.

No subtler subprocess approach would have worked because the FAIL_MODE harness is `cfg(test)` and a spawned `mnemonic` binary's library would compile WITHOUT cfg(test) — the FAIL_MODE branch is unreachable from the binary. The attempts-counter route is the right fix.

### K. Off-by-N pattern recurrence — Observability finding

The v3-fold proposal §3.3 enumerated 6 ResolvedSlot ctors but actual source has 12 (the type alias `pub type CosignerKeyInfo = ResolvedSlot;` at `synthesize.rs:190` introduces 6 additional ctor sites under the alias name).

The R0-v3-fold reviewer report at `design/agent-reports/v0_9_B-phase-3a-rescope-r0-v3-fold.md` lines 33-44 enumerated the same 6 sites without expanding to the alias — it grepped only `ResolvedSlot {` and missed `CosignerKeyInfo {`.

GREEN-time compile errors caught all 6 missing sites correctly (commit `9797985` body acknowledges this discovery). No production bug landed and the implementation is complete and correct. However, this is an exact recurrence of the `feedback_r0_must_read_source_off_by_n` pattern.

**Suggested guard** (not a finding requiring rework — proposal review-time discipline):

> When proposing changes to a struct, grep for ALL bindings via the bare type name AND any `pub type Alias = StructName;` occurrences. The grep query for any future struct-shape proposal should be:
> ```
> rg -n "<StructName> \{|<Alias> \{" crates/
> ```
> after first finding aliases via:
> ```
> rg -n "pub type \w+ = <StructName>" crates/
> ```

This is feedback for the next proposal cycle, not a Phase 3a finding.

---

## Below-threshold observations (informational; confidence < 80)

- **Site 1 pin coverage on cloned slot Vec (R-6 in proposal §5):** `cmd/bundle.rs:218` does `let slots = args.slot.clone();` after the Site 1 pin block. The Site 1 pin is on `args.slot[i].value` (heap of original Vec); the `slots[i].value` (different heap allocation) is what flows downstream. Per the proposal R-6 mitigation choice (b), Sites 2/3 struct-sibling pins cover the derived entropy that's actually consumed. Site 1 pin still covers the substituted-argv String for the brief slot-clone window. Acceptable per LOCK; flagged in proposal §5 R-6.

- **Lint comment block deferred-FOLLOWUP id mismatch:** `tests/lint_zeroize_discipline.rs:110` references `resolved-slot-entropy-zeroizing-field` but FOLLOWUPS.md now marks that as superseded by `resolved-slot-derived-account-zeroizing-field`. The cross-reference still resolves (the old entry is preserved as superseded, not deleted) and the lint comment relabel is explicitly DEFERRED to v0.10.1 per Path B-lite. No action required in Phase 3a.

- **`Send`/`Sync`:** `Arc<PinnedPageRange>` containing `*const u8` makes `ResolvedSlot: !Send + !Sync` and `DerivedAccount: !Send + !Sync`. No threading or async use exists in the codebase (verified via grep for `spawn`, `tokio`, `async`, `thread::`); CLI is single-threaded. No issue.

---

**Verdict: CLEAR — Phase 3a ships.** Proceed to Task 10 (push to origin/master, watch CI, update memory). The Cycle B field-type-migration deferral to v0.10.1 is well-documented and audit-trail-clean; the structural-discipline gap remains at Cycle-A levels through Cycle B, exactly as the v3-fold rescope intended.
