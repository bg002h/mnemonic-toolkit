# PLAN — multisig mk1 chunk_set_id is slot-unique + display-reproducible (audit I10 + n1-vs-nge2)

**Status:** R0 GREEN at R1 (2026-06-10, `design/agent-reports/mk1-csi-slot-unique-plan-r0-round2-review.md`) — implementation may proceed
**Source grounding:** toolkit `master` (re-grep at impl time). Citations below are snapshots.
**Resolves:** `design/FOLLOWUPS.md::audit-2026-06-10-backlog` item I10 `mk1-chunk-set-id-fingerprint-grouping-assumes-distinct-fps` + the coupled `[obs]` `n1-vs-nge2-csi-derivation-inconsistency`.

## 0. The bug + the history (confirmed at source)

The mk1 **`chunk_set_id` (csi)** is a 20-bit value encoded in each mk1 chunk. `verify_bundle` (`cmd/verify_bundle.rs:1568-1576`) groups *supplied* mk1 chunks by csi to reassemble each cosigner's multi-chunk card, decodes each group, then maps decoded cards to slots **by xpub** (`xpub_to_65`, :1634-1660). **The csi is NEVER recomputed at verify time** — it is purely a per-card reassembly grouping key, read from the chunks. ⇒ the only correctness requirement is **distinctness per cosigner card within a bundle**; old cards keep verifying (no recompute, no compat break).

Derivation today (`synthesize.rs:44`): `derive_mk1_chunk_set_id(stub) = (stub[0]<<12) | (stub[1]<<4) | (stub[2]>>4)` (20-bit; note `stub[3]` and `stub[2]`'s low nibble are unused). Emission:
- **n=1** (single-sig): `derive_mk1_chunk_set_id(&stub)` — policy-stub-derived (synthesize.rs:166/198/260).
- **n≥2** (multisig): `derive_mk1_chunk_set_id(&c.xpub.fingerprint())` **per cosigner** (synthesize.rs:278 live `synthesize_descriptor`; 443/615 in `#[allow(dead_code)]` `synthesize_multisig_full`/`_watch_only` test helpers).

**History:** the fingerprint scheme was itself a prior fix ("F1", `tests/cli_verify_bundle_multi_cosigner_mk1.rs:2-7`) for an EARLIER all-cosigners-share-one-stub-csi collision. F1 moved n≥2 from stub → per-cosigner-fingerprint so distinct cosigners get distinct csi.

**I10 (the residual edge case F1 missed):** two cosigners with the **same xpub at different paths** have the **same fingerprint** → same csi → their chunks merge into one BTreeMap group → `mk_codec::decode(merged)` → `ChunkedHeaderMalformed` → spurious verify failure (slot[0] `DecodeFailed`, slot[1] `NotSupplied`), even though both cards are individually correct/decodable. BIP-388 distinctness (`cmd/bundle.rs:446-460`) rejects only when BOTH `xpub.to_string()` AND `path` are equal, so same-xpub-different-path passes the gate. **Severity:** spurious-failure, not funds-loss / not wrong-bytes; trigger config (xpub reuse across paths in one wallet) is uncommon.

**Coupled bug `n1-vs-nge2-csi-derivation-inconsistency`:** the engraving-card display helper (`cmd/bundle.rs:1087-1109`) computes `mk1_card_id` as the **stub**-derived csi for ALL slots — correct for n=1, but **wrong for multisig** (actual emission is fingerprint-derived). So the displayed card-id a user reads off the plate does NOT match the real mk1 csi for multisig cosigners.

## 1. The fix — one unified slot-aware derivation

Add `derive_mk1_chunk_set_id_for_slot(stub: &[u8;4], slot: u32) -> u32 = derive_mk1_chunk_set_id(stub) ^ slot` (synthesize.rs, next to `:44`). Use it **everywhere** a mk1 csi is produced or displayed, passing the cosigner's slot index:

- **n=1 paths** (synthesize_full :166, synthesize_watch_only :198, synthesize_descriptor :260): `..._for_slot(&stub, 0)`. XOR 0 ⇒ **byte-identical to today** (pins `single_sig_csi_unchanged_byte_identical_to_pre_fix_fixture`).
- **n≥2 paths** (synthesize_descriptor :278; synthesize_multisig_full :443; synthesize_multisig_watch_only :615): `..._for_slot(&stub, i)` where `i` is the cosigner slot index. synthesize_descriptor's loop is `for c in cosigners` (slice) → `for (i, c) in cosigners.iter().enumerate()`. **R0-m2:** the dead-code helpers use a RANGE loop `for _ in 0..cosigner_count` → change to `for i in 0..cosigner_count` (no `.iter().enumerate()`). **Drop the `let fp_bytes = ...fingerprint().to_bytes()` csi lines** (the card's `Some(c.fingerprint)` field is unaffected — that is the CARD fingerprint, a separate thing).
- **Display** (bundle.rs:1095): `..._for_slot(&stub, i)` (the helper iterates `resolved.iter().enumerate()` so `i` is in hand). Update the `:1087` doc-comment ("Both ms1 and mk1 share the policy_id_stub-derived chunk_set_id") to the slot-aware reality.

**Why this scheme (vs the audit's alternatives):**
- **Distinct per slot, deterministically:** XOR is injective in `slot` for a fixed base ⇒ `base ^ i` are pairwise distinct for the distinct slot indices `0..n` (max n=16 ⇒ slot ∈ 0..=15 = 4 bits; csi stays ≤ 0xFFFFF). Immune to fingerprint collision entirely — strictly stronger than F1.
- **Preserves the leading-16-bit bundle-binding invariant (R0-I1 key insight):** the 20-bit csi packs `policy_id[0]`→bits 19..12, `policy_id[1]`→bits 11..4, `policy_id[2]>>4`→bits 3..0. The slot index (≤ 15) XORs into **bits 3..0 ONLY** — the leading **16 bits (19..4) = `policy_id[0..2]` are UNCHANGED**. So every multisig cosigner's mk1 csi shares the same leading 16 bits (and shares them with md1's `policy_id[0..2]`), differing only in the **5th hex char** (the low nibble). The old fingerprint scheme broke that agreement entirely (csi unrelated to `policy_id`); option (a) **restores** the bundle-binding invariant while still disambiguating per cosigner. This makes the technical-manual "Invariant 1" a *refinement*, not a falsification.
- **Unifies n=1 and n≥2** under ONE scheme ⇒ resolves `n1-vs-nge2` at the root (no more stub-vs-fingerprint split).
- **Display reproducible** from `(stub, i)` ⇒ the display fix is the same one-liner; displayed `mk1_card_id` now matches actual emission for multisig.
- **n=1 wire unchanged** (XOR 0) ⇒ no single-sig golden/fixture breakage.
- Dominates `derive(fingerprint_i) ^ i` (which would either change n=1 — breaking the fixture — or perpetuate the n1/n≥2 split) and `fingerprint+path` (probabilistic, more complex, still changes all multisig wire).

**ms1_card_id (display) — R0-I2 RESOLVED: slot-XOR it too.** ms1 is emitted as a single-string `ms_codec::encode` (no real chunk csi); `ms1_card_id` is a cosmetic per-slot label. Set it to `derive_mk1_chunk_set_id_for_slot(&stub, i)` (SAME as `mk1_card_id` for that slot) so the engraving shows **matching ms1 and mk1 ids on the same slot block** (and distinct ids across slots — clearer for bundle-membership). Keeping ms1 at the slot-independent base would make slot ≥1 show ms1 ≠ mk1 on one card block → a user verifying membership is misled. Both labels still share the leading 16 bits (low-nibble-only difference per slot).

## 2. Wire change + SemVer

Multisig mk1 csi values change (fingerprint-derived → `stub ^ slot`) for ALL multisig bundles, not just colliding ones ⇒ **emitted multisig mk1 bytes change**. n=1 single-sig bytes are unchanged. **Backward-compatible for verify** (csi not recomputed; an old multisig card still groups+decodes fine). **SemVer MINOR** (changes emitted card bytes for a card type — precedent: trmultia-NUMS v0.48.0). No fielded-card breakage.

## 3. Tests (TDD)

- **NEW regression (the I10 repro):** a **2-of-2 reusing ONE xpub at two different paths**, emitted then round-tripped through `verify-bundle` (`--bundle-json` and/or flat `--mk1`), MUST map BOTH cosigners (no `DecodeFailed`/`NotSupplied`). Pre-fix this fails (collision); post-fix passes. Place in `tests/cli_verify_bundle_multi_cosigner_mk1.rs` (mirror Cell 1/2 structure; needs a descriptor with `@0` and `@1` resolving to the same xpub at different origin paths — confirm the distinctness gate admits it).
- **Display-match assertion:** for a multisig bundle, the `mk1_card_id` shown in the engraving display equals the actual csi of that slot's emitted mk1 chunks (close `n1-vs-nge2`), and `ms1_card_id` == `mk1_card_id` for each slot (R0-I2). Derive expected via `derive_mk1_chunk_set_id_for_slot(&stub, i)`; assert distinct across slots and equal leading-4-hex (16 bits) across slots.
- **Preserve:** `single_sig_csi_unchanged_byte_identical_to_pre_fix_fixture` (:266) MUST stay green (n=1 unchanged). The existing multisig round-trip cells (1-3, 3-of-3) stay green (they round-trip, csi-value-agnostic).
- Update the module doc-comment of `cli_verify_bundle_multi_cosigner_mk1.rs:2-7` (describes the F1 fingerprint scheme — now superseded by slot-XOR).

## 4. Scope, docs, release

- **In scope:** I10 (collision) + `n1-vs-nge2` (display match). **Out of scope:** `anti-collision-16bit-invariant-false` — that finding is specifically about **md1's** csi using `compute_md1_encoding_id` (Md1EncodingId) while the mk1/ms1 *display* uses `compute_wallet_policy_id` (WalletPolicyId) — an md1-vs-mk1 cross-card concern this fix does NOT touch (we only change the mk1 multisig csi derivation). Do NOT claim to resolve it; do NOT alter md1's csi.
- **R0-I1 — Technical manual** `docs/technical-manual/src/40-bundle-formation/42-anti-collision-invariants.md` ("Invariant 1 — Shared `chunk_set_id` prefix") documents the derivation and MUST be updated for accuracy:
  - Table row :13-14: the mk1 entry's `Source` becomes `derive_mk1_chunk_set_id(policy_id[0..4]) ^ slot_index` for multisig (n≥2); n=1 unchanged. The leading 16 bits are unaffected (XOR hits only the low nibble).
  - :16 (fifth hex char prose): the 5th hex char is `(policy_id[2]>>4) ^ slot_index` for multisig mk1 (was just `policy_id[2]>>4`).
  - :9 / :18 binding rule: the leading-16-bit agreement **still holds** (refine, don't delete) — multisig cosigner mk1 cards share their leading 16 bits and differ only in the 5th hex char by slot. The OLD fingerprint scheme (which this fix replaces) is what actually broke the agreement; this fix restores it. (Be careful: do NOT overstate md1↔mk1 16-bit agreement — that is governed by `anti-collision-16bit-invariant-false`, untouched here.)
  - Source-pointers table (:149-ish): add `synthesize.rs::derive_mk1_chunk_set_id_for_slot`.
  - This chapter is symbol-pin-lint-gated (G2 existence), but the lint won't catch stale PROSE — manual edit + full `make -C docs/technical-manual lint` (or its toolkit subset) before push.
- **R0-m1 — End-user manual:** `docs/manual/src/40-cli-reference/41-mnemonic.md:392-393` reads "v0.20.0's F1 fix gave each cosigner's chunk-set its own `chunk_set_id` (xpub-fingerprint-derived)" → reword to slot-XOR-derived (the F1 fingerprint approach is superseded). No CLI flag/surface change ⇒ no flag-coverage / GUI `schema_mirror` impact; run the manual flag-coverage lint anyway.
- **FOLLOWUPS:** resolve I10 `mk1-csi-multisig-same-xpub-collision` + `n1-vs-nge2-csi-derivation-inconsistency`; note `anti-collision-16bit-invariant-false` remains open.
- **MINOR bump** + CHANGELOG. Update the `derive_mk1_chunk_set_id` doc (synthesize.rs:41-45) to describe the slot-aware companion.
- Full suite green; clippy clean. Re-grep all `derive_mk1_chunk_set_id` / `fingerprint().to_bytes()` csi sites at impl time (7 derive sites + display + the dead-code helpers).
