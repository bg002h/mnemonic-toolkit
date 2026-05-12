# tech-manual v0.3 — Phase 3.4 reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.3.0` (in progress) |
| Phase | 3.4 (back-matter accretion: glossary +16, index +3, release-history row, BIP cross-reference §IV.* updates) |
| Commit under review | `d58da02` |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `docs/technical-manual/src/60-back-matter/61-glossary.md` (+16 entries) · `docs/technical-manual/src/60-back-matter/62-index-table.md` (+3 rows) · `docs/technical-manual/src/40-bundle-formation/43-future-shares.md` (+3 `\index{}` markers) · `docs/technical-manual/src/60-back-matter/63-release-history.md` (+1 row) · `docs/technical-manual/src/60-back-matter/64-bip-cross-reference.md` (6 rows updated) · `docs/technical-manual/.cspell.json` (+1 word) |

## Findings: 0 Critical / 3 Important / 3 Low / 0 Nit

---

## Important

**I-1. BIP cross-reference: BIP-32 row has §IV.3; BIP-32 is not cited in §IV.3 (confidence: 95)**

`64-bip-cross-reference.md`, BIP-32 row: `§I.1, §I.2, §I.3, §I.4, §II.1, §II.2, §II.3, §III.1, §III.3, §IV.3`

Grep of `43-future-shares.md` for "BIP-32" returns zero matches. BIP-32 IS cited in §IV.2 at `42-anti-collision-invariants.md:40` ("...since a BIP-32 derivation step depends on the chain code as well as the parent pubkey"). The §IV.3 citation is a mis-attribution.

Resolution: replace `§IV.3` with `§IV.2` in the BIP-32 row (folded inline at phase close).

---

**I-2. BIP cross-reference: BIP-93 row has §IV.2; BIP-93 is not cited in §IV.2 (confidence: 95)**

`64-bip-cross-reference.md`, BIP-93 row: `§I.2, §I.3, §I.4, §II.2, §II.3, §IV.2, §IV.3`

Grep of `42-anti-collision-invariants.md` for "BIP-93" returns zero matches. BIP-93 IS correctly cited in §IV.3. The §IV.2 entry is spurious.

Resolution: remove `§IV.2` from the BIP-93 row (folded inline at phase close).

---

**I-3. BIP cross-reference: BIP-39 row is missing §IV.2 (confidence: 95)**

`64-bip-cross-reference.md`, BIP-39 row: `§I.1, §I.2, §II.3, §IV.1, §IV.3`

BIP-39 IS cited in §IV.2 at `42-anti-collision-invariants.md:61` ("the bundle was created as full-mode (`expected.ms1[i]` was synthesized as a real BIP-39 entropy ms1 string)").

Resolution: add `§IV.2` to the BIP-39 row (folded inline at phase close).

---

## Low

**L-1. Glossary sort error: `cosigner-mapping diagnostic` placed after `cross_chunk_hash` (confidence: 95)**

Case-insensitive: "cos" ('o') < "cro" ('r'); `cosigner-mapping diagnostic` must precede `cross_chunk_hash`.

Resolution: move `## cosigner-mapping diagnostic` block before `## cross_chunk_hash` (folded inline at phase close).

---

**L-2. Glossary sort error: `multiset` placed before `multipath` (confidence: 95)**

Case-insensitive: "multip" ('p') < "multis" ('s'); `multipath` must precede `multiset`.

Resolution: move `## multiset` block after `## multipath` (folded inline at phase close).

---

**L-3. Glossary sort error: `secret-bearing slot` placed before `script context (rust-miniscript)` (confidence: 95)**

Case-insensitive: "sc" ('c') < "se" ('e'); `script context (rust-miniscript)` must precede `secret-bearing slot`.

Resolution: move `## secret-bearing slot` block after `## script context (rust-miniscript)` (folded inline at phase close).

---

## Resolution (Phase 3.4 close)

All six findings folded inline at the closing commit. None deferred. All are mechanical fixes (BIP table edits; glossary reorderings).

---

## Verified-correct items (no action needed)

**All 16 new glossary source citations verified against HEAD:**

- `bundle`: `synthesize.rs:593`, `verify_bundle.rs:98` — confirmed.
- `bundle envelope`: `format.rs:119-145` — confirmed.
- `BundleMode`: `bundle_unified.rs:34-63` — confirmed.
- `cosigner-mapping diagnostic`: `verify_bundle.rs:831-836`, `:895-947` — confirmed.
- `DescriptorPublicKey`: `to_miniscript.rs:84-89` and `:175` (NUMS path) — confirmed.
- `engraving card`: `format.rs:259-376` — confirmed.
- `H-point (NUMS)`: `to_miniscript.rs:34-35` — confirmed.
- `md1_xpub_match`: `verify_bundle.rs:1194-1232` — confirmed.
- `PathDecl`: `origin_path.rs:82-96` — confirmed.
- `secret-bearing slot`: `slot_input.rs:47-49` — confirmed.
- `verify-bundle`: `verify_bundle.rs:98-201` — confirmed.
- `VerifyCheck`: `format.rs:165-183` — confirmed.
- `watch-only slot`: `slot_input.rs:50-52` — confirmed.
- `XpubNotInPolicy`: `verify_bundle.rs:835` + `:1128-1131` — confirmed.

**Three `\index{}` markers in `43-future-shares.md` confirmed at correct in-text locations.**

**Three new index rows — anchors and sort positions verified.**

**BIP cross-reference non-erroneous entries confirmed:**
- BIP-84 `§IV.1`, BIP-388 `§IV.1, §IV.2`, BIP-389 `§IV.1`, BIP-39 `§IV.1`, BIP-93 `§IV.3` — all correct.

**Release-history row:** date, 119pp PDF size, Part III description, and "back-matter accreted" claim all verified accurate.

**cspell:** "unmappable" present at `.cspell.json:175` and used in `cosigner-mapping diagnostic` body.
