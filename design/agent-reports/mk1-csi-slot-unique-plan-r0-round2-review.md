# R0 Architecture Review — PLAN_mk1_csi_slot_unique.md — Round 2 (R1)

**Reviewer:** Fable 5 (feature-dev:code-architect)
**Date:** 2026-06-10
**Plan doc:** `design/PLAN_mk1_csi_slot_unique.md` (post-fold)
**Prior:** `design/agent-reports/mk1-csi-slot-unique-plan-r0-round1-review.md`

---

## VERDICT: GREEN — 0 Critical, 0 Important, 0 Minor. Implementation may proceed.

---

## Round-1 finding resolutions

**I1 — Technical manual factually wrong — RESOLVED.** §4 now names `docs/technical-manual/src/40-bundle-formation/42-anti-collision-invariants.md` with specific sub-targets (table :13-14 → `derive_mk1_chunk_set_id(policy_id[0..4]) ^ slot_index` for n≥2; :16 fifth-hex → `(policy_id[2]>>4) ^ slot_index`; :9/:18 binding rule REFINED not deleted; source-pointers += `derive_mk1_chunk_set_id_for_slot`). Frames the invariant as restored-not-falsified, with the explicit guard not to overstate md1↔mk1 agreement.

**I2 — ms1/mk1 card-id mismatch — RESOLVED.** §1 sets `ms1_card_id = derive_mk1_chunk_set_id_for_slot(&stub, i)` (== mk1_card_id per slot); §3 adds the display-match test assertion `ms1_card_id == mk1_card_id for each slot`.

**m1 — cite manual file:line — RESOLVED.** §4 cites `docs/manual/src/40-cli-reference/41-mnemonic.md:392-393` by name (confirmed present at source).

**m2 — range loop for dead-code helpers — RESOLVED.** §1 distinguishes `for i in 0..cosigner_count` (range, dead-code helpers `:443/:615`) from the slice `.enumerate()` (live synthesize_descriptor). Confirmed `synthesize.rs:429` is `for _ in 0..cosigner_count`.

---

## Critical verification — bit-packing claim ("leading 16 bits preserved")

Verified against `synthesize.rs:44-45`: `((stub[0])<<12) | ((stub[1])<<4) | ((stub[2])>>4)`.
- bits 19..12 = stub[0]; bits 11..4 = stub[1]; bits 3..0 = stub[2]>>4 (high nibble).
- ⇒ bits 19..4 = stub[0]||stub[1] (leading 16 / first 4 hex chars); bits 3..0 = the 5th hex char.
- `^ slot` for slot ∈ 0..=15 (4 bits) touches ONLY bits 3..0 ⇒ leading 16 bits unchanged ⇒ all cosigners share the first 4 hex chars, differ only in the 5th by exactly the slot index. **CONFIRMED.**
- Cosigner cap: `> 16 → Err` enforced at `synthesize_multisig_full:355`, `synthesize_multisig_watch_only:500`, `synthesize_unified:753` ⇒ slot ≤ 15 always ⇒ 4-bit containment guaranteed at every emission site. **CONFIRMED.**

The refined-invariant framing (§1 + technical-manual edits) is mathematically sound.

---

## Fold-introduced drift — none

- I2 (`ms1_card_id = ..._for_slot(&stub, i)`) consistent with the display-match test. No contradiction.
- Scope correctly EXCLUDES `anti-collision-16bit-invariant-false` (md1 Md1EncodingId vs mk1/ms1 WalletPolicyId — untouched); technical-manual edits carry the matching no-overstate guard.
- Source-pointer addition of the NEW `derive_mk1_chunk_set_id_for_slot` correctly deferred to impl; existing `derive_mk1_chunk_set_id` entry stays valid.
- Dead-code helpers keep `#[allow(dead_code)]`; only loop var + csi derivation change.

---

## Summary

All four round-1 findings resolved; the new bit-math claim rigorously confirmed against source; cosigner cap verified at all 3 synthesis entry points. **Implementation may proceed.**
