# v0.32.0 plan-doc R0 review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R0
**Plan under review:** `design/PLAN_mnemonic_toolkit_v0_32_0.md`
**Date:** 2026-05-21
**Source SHA:** `7e50902` (master HEAD at review time)

## Verdict

**RED** — 3 Critical / 2 Important / 2 Minor. The plan-doc's central architectural claim about `Ord` ordering is FACTUALLY WRONG; this will break Phase 2 implementation. SemVer rationale is also load-bearing-wrong.

## Critical (C)

**C1. `Ord`-based pattern claim in §"File structure" + §"Risk register" is inverted.** The plan says "slot Seedqr at the end so `Ord` derives don't shift existing legal-set ordering." This is wrong. `is_legal_set` (slot_input.rs:313-332) matches sorted slices against patterns like `[Phrase, Path]`, `[Xpub, MasterXpub]`, etc. — and `validate_slot_set` sorts subkeys (L250) using `Ord` derived in declaration order (L16: `Phrase < Entropy < Xpub < MasterXpub < Fingerprint < Path < Wif < Xprv`). Appending `Seedqr` LAST means sorted slices place it AFTER `Xprv`, so legal patterns become `[Seedqr]`, `[Path, Seedqr]`, `[Fingerprint, Path, Seedqr]` — NOT `[Seedqr, Path]` / `[Seedqr, Fingerprint, Path]` as the plan claims. Fix: place `Seedqr` between `Phrase` and `Entropy` (mirroring Phrase's slot order so the legal-set pattern shape mirrors Phrase exactly), then the patterns `[Seedqr]`, `[Seedqr, Path]`, `[Seedqr, Fingerprint, Path]` are correctly ascending-sorted. The `exempted_v0_19_0` matcher (L274-278) has the same hazard.

**C2. Branch-placement instruction in Task 2 Step 2 is incorrect.** Plan says "Slot the new branch BEFORE the `Phrase` branch." Since `is_legal_set` REFUSES `[Phrase, Seedqr]` co-occurrence, branch order doesn't affect correctness, but placing Seedqr BEFORE Phrase fragments the secret-bearing dispatch group. Recommended: NEW branch should be placed AFTER Phrase, BEFORE Xpub, to keep secret-bearing branches contiguous.

**C3. Error mapping reuses internal `map_seedqr_error` — but the function is private to `cmd/seedqr.rs`.** The plan says "decode errors map through `map_seedqr_error`". But `cmd/seedqr.rs:58` declares it as `fn map_seedqr_error(...)` — no `pub`. The bundle/verify-bundle/export-wallet consumers cannot call it as-is. Plan must specify either (a) promote `map_seedqr_error` to `pub(crate)` and move to a shared module, or (b) re-implement inline at each consumer (with the risk of error-text drift). Currently the mapping is `"seedqr: {action}: {e}"`; if inline-replicated, plan must lock the string format at each site.

## Important (I)

**I1. SemVer-MINOR rationale conflates "new variant" with "user-visible CLI surface change."** `SlotSubkey` is `pub` — adding a non-exhaustive variant IS a public-API change to library consumers. But the toolkit isn't yet on crates.io (per memory, blocked on miniscript `[patch.crates-io]`), and consumers are git-pinned, so the public-API-break argument is weak. The load-bearing trigger is **new clap-flag value-enumeration token**. Per memory `v0.28+ Wave 2 SHIPPED` precedent ("4 FOLLOWUPs closed (...) flag additions" → PATCH), a single new slot-subkey token is arguably PATCH-class. The GUI schema_mirror gate at `mnemonic-gui/src/schema/mnemonic.rs:327-332` carries a `--slot` flag entry with help-text describing the subkey enumeration, but it does NOT carry a parallel `SlotSubkey` enum surface — the schema_mirror gate compares clap flag-NAME parity, not value-enumeration content. Per memory `v0.28+ Wave 3 SHIPPED` R0 I1: "schema_mirror scope misunderstanding — gates clap flag-name parity, NOT JSON wire-shape." Adding a new slot-subkey TOKEN may NOT actually trip the schema_mirror gate. If it doesn't, the GUI pin bump is still desirable (display the new subkey in help/dropdowns) but doesn't auto-fire. This weakens the MINOR argument further. Recommend: rewrite rationale, surface PATCH alternative for user decision.

**I2. Missing `is_stdin_sentinel` refusal-matrix cell.** Plan asserts `apply_slot_stdin` works automatically with the new variant. Verified: `is_stdin_sentinel` delegates to `subkey.is_secret_bearing()`, and the plan adds `Seedqr` to `is_secret_bearing`. So it does work automatically — BUT the plan does NOT explicitly call out that `@N.seedqr=-` will REJECT co-existence with `--passphrase-stdin` AND with another `@M.<secret>=-` slot. Plan should add a refusal-matrix test cell (`bundle_seedqr_slot_double_stdin_refused`) asserting the existing 2-stdin-sentinel guard fires correctly with one seedqr + one phrase stdin slot.

## Minor (M)

**M1. Test surface omits the byte-equal cross-input regression for 12-word case.** Plan's 24-word happy-path cell mentions "assert byte-equal bundle envelope to the phrase-direct variant" — good. But the 12-word equivalent (`bundle_seedqr_slot_happy_path_12word`) doesn't repeat the byte-equal assertion. Lock both happy-path cells to assert envelope-byte-equality to the corresponding `--slot @N.phrase=` invocation.

**M2. Master-xpub help-text drift parking → file FOLLOWUP NOW.** Risk register says "file a separate FOLLOWUP if it surfaces during R0 review." It is now R0; the omission has surfaced. File the FOLLOWUP as a Phase 6 task. The omitted token is `master_xpub` (slot_input.rs:40, present in `from_token` since v0.8.2 but missing from bundle.rs:97-105 help text).

## Source-citation verification

- slot_input.rs L17-32, L34-47, L60-69, L225-301, L313-332, L274-278 — verified at HEAD.
- bundle.rs L91-117 slot help block — verified (omits `master_xpub` as plan notes).
- bundle.rs L433-614 slot consumer — verified; if/else-if chain on `subkeys.contains(&SlotSubkey::X)`.
- seedqr.rs:56 `pub fn decode` — verified.
- cmd/seedqr.rs:58 `map_seedqr_error` — verified as PRIVATE.

## Recommendation

**Block Phase 2 dispatch.** Fold C1+C2+C3 (and verify I1, I2, M1, M2) into a plan-doc revision, then re-dispatch R0 (or R1 confirming the fold). C1 is the load-bearing one — getting the legal-set patterns wrong will cause the unit tests in Step 2 to fail mid-Phase-2.
