# R0 Architecture Review — PLAN_mk1_csi_slot_unique.md — Round 1

**Reviewer:** Fable 5 (feature-dev:code-architect)
**Date:** 2026-06-10
**Plan doc:** `design/PLAN_mk1_csi_slot_unique.md`

---

## VERDICT: YELLOW — 0 Critical, 2 Important, 2 Minor. GREEN after fold.

---

## Confirmed Correct

**Keystone #1 — csi never recomputed at verify time.** Confirmed. `cmd/verify_bundle.rs` has ZERO calls to `derive_mk1_chunk_set_id`. All three `chunk_set_id_extract` calls (`:1571`, `:2220`, `:2420`) read csi from supplied chunk strings; none derive/validate it. `self_check_bundle` (`cmd/bundle.rs:2060-2138`) decodes each per-cosigner chunk vec individually via `mk_codec::decode(&strs)` at `:2117` — no csi grouping/recompute. "Old cards still verify / no compat break / value-agnostic round-trip" is sound. (Note: 3 grouping sites — :1571 main, :2220/:2420 watch-only cross-checks — all map by xpub after grouping; same-xpub-different-path maps via first-unfilled-matching-slot, position-biased but consistent; the cross-checks are warnings, not blocking.)

**#2 — collision mechanism.** Confirmed. `synthesize.rs:277-278` uses `c.xpub.fingerprint().to_bytes()` for n≥2. Distinctness gate `bundle.rs:449-451` (and `parse_descriptor.rs:1212`) rejects only `xpub.to_string()==.. && path==..`; `parse_descriptor.rs:1878` (`bip388_same_xpub_different_paths_accepted`) confirms same-xpub-different-path is admitted.

**#3 — XOR distinctness + range.** Confirmed. Max cosigner count = 16 (enforced `synthesize.rs:355/:500/:753`) → slot ∈ {0..=15}, 4 bits. `derive_mk1_chunk_set_id` ≤ 0xFFFFF (20-bit). XOR with a 4-bit slot only touches bits 0-3 → result ≤ 20 bits, within `encode_with_chunk_set_id`'s `0..=0xFFFFF` (oversize → ChunkedHeaderMalformed). XOR injective in slot for fixed base → pairwise-distinct.

**#4 — n=1 byte-preservation.** Confirmed. `synthesize_descriptor:260` uses stub-derived for n=1; `synthesize_unified` delegates to it. `single_sig_csi_unchanged_byte_identical_to_pre_fix_fixture` (`tests/cli_verify_bundle_multi_cosigner_mk1.rs:266`) pins the single-sig fixture; `base ^ 0 = base` → byte-identical → stays green.

**#5 — emission sites fully enumerated.** Confirmed by grep. 7 `derive_mk1_chunk_set_id` sites: :166/:198/:260 (n=1 stub; :166/:198 in `#[allow(dead_code)]` synthesize_full/watch_only, :260 live synthesize_descriptor), :278 (live n≥2), :443 (dead synthesize_multisig_full), :615 (dead synthesize_multisig_watch_only), + display bundle.rs:1095.

**#6 — display has stub + i.** Confirmed. `bundle.rs:1083-1085` iterates `resolved.iter().enumerate()`; stub via reassemble→compute_wallet_policy_id at :1089-1097.

**#7 — regression test constructible.** Confirmed via `bip388_same_xpub_different_paths_accepted` + CLI `--slot @N.xpub/.path`. 2-of-2 same-xpub two-paths reaches emission; pre-fix → spurious DecodeFailed.

**#8 — no other multisig mk1 byte-fixture.** Confirmed. Only byte-pinned mk1 fixture is single-sig `v0_20_0_single_sig_bip84_bundle.json`. Multisig cells are round-trip (csi-agnostic). `cli_bundle_full.rs` v0_1 vectors all single-sig.

**#9 — SemVer MINOR.** Confirmed. Multisig mk1 wire bytes change; precedent v0.48.0 trmultia-NUMS. No flag change → no schema_mirror.

---

## Important

**I1 — Technical manual `42-anti-collision-invariants.md` becomes factually wrong for n≥2; not in plan scope.** `docs/technical-manual/src/40-bundle-formation/42-anti-collision-invariants.md` lines 9-20 claim every ms1/mk1/md1 from one bundle derives its id from the same `policy_id`; table :14 `ms1, mk1 | 20 | 5 | derive_mk1_chunk_set_id(policy_id[0..4])`; :18 "leading 16 chunk_set_id bits agree across cards". Post-fix, n≥2 mk1 cosigners have DISTINCT csi (`base ^ i`) → these claims false for multisig. The plan's §4 names only `docs/manual/src/`, missing `docs/technical-manual/src/`. Symbol-pin lint won't catch it (text wrong, `derive_mk1_chunk_set_id` still exists). **Fix:** add the technical-manual chapter to §4; update :9/:14/:16/:18 to the slot-XOR reality for n≥2 (preserve n=1 + md1 description); add `synthesize.rs::derive_mk1_chunk_set_id_for_slot` to the source-pointers table at :149; retitle/qualify the "Shared chunk_set_id prefix" invariant (shared only for n=1).

**I2 — ms1/mk1 card-id display mismatch for multisig slots ≥1; plan defers to R0.** Plan keeps `ms1_card_id = base` (slot-independent) while `mk1_card_id = base ^ i`. For slot ≥1 in full-mode multisig the engraving shows DIFFERENT ms1 vs mk1 ids on the same slot block → a user verifying bundle membership is misled. ms1 has no wire csi (cosmetic label) so no constraint forces base. **Fix:** XOR ms1_card_id with slot too — `ms1_card_id = derive_mk1_chunk_set_id_for_slot(&stub, i)` — so per-slot ms1 and mk1 ids agree and are distinct across slots (clearer for membership). Resolve before impl.

---

## Minor

**m1 — Cite `docs/manual/src/40-cli-reference/41-mnemonic.md:392-393` explicitly in §4.** Live text: "v0.20.0's F1 fix gave each cosigner's chunk-set its own chunk_set_id (xpub-fingerprint-derived)" → stale post-fix. Grep finds it, but name it per the grep-verified-citations convention.

**m2 — `synthesize_multisig_full` loop is `for _ in 0..cosigner_count`, not a slice loop.** The correct change there is `for i in 0..cosigner_count` (range, no `.iter().enumerate()`). The plan's "for c in cosigners → for (i,c) in cosigners.iter().enumerate()" describes the live `synthesize_descriptor` slice loop only; spell out the range-loop variant for the dead-code helper.

---

**GREEN requires:** fold I1 + I2; m1/m2 recommended.
