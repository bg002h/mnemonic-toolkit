# tech-manual v0.3 — final whole-cut reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.3.0` (pre-tag) |
| Phase | 3.5 (final whole-cut, tag-time) |
| Commit under review | `7f9fea3` (Phase 3.4 close — pre-tag HEAD) |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `40-bundle-formation/{41,42,43}*.md` + `60-back-matter/{61,62,63,64}*.md` delta + 3 transcript pairs + `.cspell.json` |

## Findings: 0 Critical / 1 Important / 2 Low / 0 Nit

Per `feedback_zero_followups_from_release_cycles`: all findings fold inline at the closing commit. None deferred.

---

## Important

**I-1. §IV.2 presents `md1_xpub_match` as universally multiset, but single-sig uses a different first-only comparison (confidence: 85)**

`42-anti-collision-invariants.md:24` opens Invariant 2:

> The `md1_xpub_match` check (`verify_bundle.rs:1194-1232`) asserts that the **multiset** of pubkeys in the supplied md1's `Tag::Pubkeys = 0x02` TLV equals the multiset of pubkeys in the expected md1's same TLV.

The cite is in `emit_multisig_checks` — the multisig-only path. Single-sig uses `emit_md1_checks` at `verify_bundle.rs:1280-1355`, which compares only the **first** pubkey via `.first()`:

```rust
let exp_xpub = exp_desc.tlv.pubkeys.as_ref().and_then(|v| v.first()).map(|(_, b)| *b);
let act_xpub = desc.tlv.pubkeys.as_ref().and_then(|v| v.first()).map(|(_, b)| *b);
let xpub_match = exp_xpub == act_xpub;
```

On success the single-sig `md1_xpub_match` emits `detail: "65-byte xpub matches expected"` (confirmed at `verify_bundle.rs:1323` and in `mnemonic-verify-bundle-bip84-abandon.out:9`). The multiset `detail` "all {N} pubkeys match expected (multiset)" (`verify_bundle.rs:1216`) is never emitted for N=1.

The glossary entry for `md1_xpub_match` (`61-glossary.md`) also says "Sort-then-compare on `Vec<[u8; 65]>` preserves multiplicity" — accurate for multisig only.

The chapter sets a disclosure precedent: Invariant 3 explicitly notes "Single-sig (N=1) uses an analogous but simpler path in `emit_verify_checks` (`verify_bundle.rs:620-687`)" — the single-sig counterpart note for Invariant 2 is absent.

Resolution: in §IV.2 add a sentence noting single-sig uses `emit_md1_checks` with `.first()` comparison and detail `"65-byte xpub matches expected"`; in the glossary append the same caveat parenthetically (folded inline at phase close).

---

## Low

**L-1. Release-history table missing `tech-manual-v0.3.0` row (confidence: 95)**

`63-release-history.md` currently ends at the `tech-manual-v0.2.0` row. The `tech-manual-v0.3.0` row must be added before tagging.

Resolution: append v0.3.0 row (folded inline at phase close).

---

**L-2. Index-table section pointers for `abandon test mnemonic` and `BIP-389` are incomplete — both omit §IV.1 (confidence: 80)**

The bidirectional lint passes (it only verifies term presence, not section completeness), but two rows are inaccurate:

- `62-index-table.md` row for `abandon test mnemonic`: points only to `Descriptor to Miniscript to Address`. The term is also `\index{}`-marked at `41-bundle-anatomy.md:189` (BIP-84 worked-example paragraph).
- `62-index-table.md` row for `BIP-389`: points only to `Descriptor to Miniscript to Address`. The term is also `\index{BIP-389}`-marked at `41-bundle-anatomy.md:211` (source-pointers section).

The BIP cross-reference table correctly lists `§IV.1` for BIP-389; only the index table lags.

Resolution: append `Bundle Anatomy` section pointer to both rows (folded inline at phase close).

---

## Resolution (final whole-cut)

All three findings fold inline at the closing commit. None deferred. Per `feedback_zero_followups_from_release_cycles`.

---

## Verified-correct items (no action needed)

**All four per-phase finding resolutions confirmed applied at HEAD:**
- Phase 3.1 (I-1 / I-2 / L-1 / L-2 / N-1 / N-2) — applied at `41-bundle-anatomy.md`.
- Phase 3.2 (I-1 / I-2 / L-1 / N-1) — applied at `42-anti-collision-invariants.md`.
- Phase 3.3 (C-1 / I-1 / L-1) — applied at `43-future-shares.md`.
- Phase 3.4 (I-1 / I-2 / I-3 / L-1 / L-2 / L-3) — applied at `64-bip-cross-reference.md` and `61-glossary.md`.

**Source line numbers spot-checked against HEAD:**
- `bundle.rs:707` md1 4-hex format — confirmed.
- `bundle.rs:724` mk1/ms1 5-hex format — confirmed.
- `verify_bundle.rs:621` single-sig `expected.ms1.first().map(|s| s.is_empty()).unwrap_or(true)` — confirmed.
- `verify_bundle.rs:182` `schema_version: "4"` — confirmed.
- `format.rs:260` `DESCRIPTOR_MAX_INLINE = 80` — confirmed.
- `error.rs:325` and `:328` BIP-388 stderr text — confirmed byte-exact.
- `verify_bundle.rs:1004` `format!("{:?}", e)` for Case-3 decode_error — confirmed.
- `synthesize.rs:42-44` derive_mk1_chunk_set_id packing — confirmed.
- `verify_bundle.rs:1194-1232` multiset sort-then-compare — confirmed exact.
- `verify_bundle.rs:1280-1355` single-sig `.first()` path — confirmed (establishes I-1).
- `verify_bundle.rs:1323` single-sig `"65-byte xpub matches expected"` — confirmed.

**Transcript content verified:**
- `mnemonic-bundle-bip84-abandon.out` — three card sets + engraving card + warning — correct.
- `mnemonic-verify-bundle-bip84-abandon.out` — 9 named checks + `result: ok` = 10 lines — correct.
- `mnemonic-bundle-bip388-collision.out` — single-line `error: BIP-388 distinct-key violation: ...` — byte-exact to `error.rs:325`.

**BIP cross-reference table (post-Phase-3.4 fixes):** BIP-32, BIP-93, BIP-39 all correct.

**Glossary sort order (post-Phase-3.4 fixes):** `cosigner-mapping diagnostic` < `cross_chunk_hash`; `multipath` < `multiset`; `script context` < `secret-bearing slot` — all confirmed.

**cspell dictionary:** "subkeys", "multiset", "miscategorized", "misgrouped", "unmappable" — all present.

**Cross-chapter forward-references fulfilled:**
- §IV.1 → §IV.2 ("Anti-collision invariants ... are §IV.2") — fulfilled.
- §IV.1 → §IV.3 ("the future K-of-N share layer is §IV.3") — fulfilled.
- §IV.3 → §IV.2 (Invariant 1 reference) — fulfilled.

**§IV.3 C-1 resolution:** `interpolate_at` correctly described as reconstruction-only; generation-step note accurate vs. `rust-codex32 v0.1.0` API.

**BundleMode + pre-checks:** five-variant enum, threshold-range / template-N pre-checks — confirmed at `bundle_unified.rs:14-112`.
