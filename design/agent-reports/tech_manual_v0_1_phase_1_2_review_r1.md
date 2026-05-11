# tech-manual v0.1 Phase 1.2 review r1 — md1 wire format (commit e287bb8)

Reviewed: `docs/technical-manual/src/20-wire-formats/21-md1-wire-format.md` (the heaviest single chapter in v0.1 — ~300 lines, ~10pp). Cross-checked against `crates/md-codec/src/{tag,header,tree,canonicalize,origin_path,to_miniscript,error}.rs`, the BIP draft, MIGRATION.md, and corpus vectors.

## Critical: 0

## Important: 5 (all folded inline)

### I1 — Encode example: path-decl decomposition omitted the 5-bit `n-1` field

Path-decl always opens with `5-bit (n−1)` then `4-bit depth`. The chapter said only `4 bits` (just the depth). For wpkh_basic the path-decl is **9 bits** (5+4), not 4.

**Fix applied:** rewrote step 2 to show `00000 0000` (n-1=0 then depth=0) and corrected the totals row to `36 bits used, 40 bits on wire`.

### I2 — Encode totals: 8 data symbols + 24 chars, not 7 + 23

Corpus phrase `md1yqpqqxqq8xtwhw4xwn4qh` is 24 characters (3 HRP + 8 data + 13 check). My arithmetic was off by 1 symbol.

**Fix applied:** corrected to "8 data symbols + 13 check + 3 HRP+sep = 24 characters total."

### I3 — Decode example: 25 data+check chars, not 23

Phrase `md1yzpqqxppsgsc8dua4tu0kekyl` is 28 characters total = 3 HRP+sep + 25 data+check.

**Fix applied:** corrected to "28 characters total = 3 (HRP + separator) + 25 (data + check symbols)."

### I4 — Decode step 2: bit notation had 6 digits, not 5

Wrote `0 0010 0` which is 6 bits collapsed. The character `y` is value 4 = `00100` (5 bits).

**Fix applied:** simplified to "First 5-bit symbol = the integer value of character `y` = `4` = bits `00100`."

### I5 — Walker normalisation source-file cite was wrong

Cited `canonicalize.rs` for the walker normalisation pass. That file does placeholder-ordering canonicalisation (`@N` permutation), not bare-PkK/PkH normalisation. The bare-PkK/PkH invariant is documented in BIP draft §"Round-trip canonical form"; the inverse reconstruction lives in `to_miniscript.rs`.

**Fix applied:** rewrote the walker-normalisation section to cite the BIP draft and `to_miniscript.rs`. Added `to_miniscript.rs` to the reference-implementation pointer list; updated `canonicalize.rs` description to its actual role.

## Low: 0

## Nit: 0

## Verified correct (sample of clean items)

- All 36 tag-table entries (0x00–0x23) exact match against `crates/md-codec/src/tag.rs`.
- Header bit layouts (single-string 5-bit, chunked 37-bit) exact match against `header.rs` + BIP draft §"Header".
- Auto-dispatch rejection table (4 rows: v0.x single, v0.x chunked, v0.30 single, v0.30 chunked) exact match against BIP draft.
- Body shape names (`Body::MultiKeys { k, indices }`, `Body::Variable { k, children }`, `Body::KeyArg { index }`, `Body::Tr { is_nums, key_index, tree }`) exact match against `tree.rs`.
- `tr()` NUMS wire shape verified against `tree.rs` write_node/read_node.
- NUMS history (v0.17 / v0.18 / v0.30) matches MIGRATION.md verbatim.
- All 5 canonicality-rule error variants verified against `error.rs`.
- Both transcripts' `.out` files match corpus vectors exactly.
- Cross-references (§I.3, §III.1, §III.2, §IV.2, §V.1) match SPEC §4.2 layout.
- TLV tag namespace (separate 5-bit space) correctly noted.
- History note on retired wire-layer dictionaries + 5-operator promotion matches BIP draft §500 verbatim.

## Post-fix verification

- `make lint` 6/6 green.
- `make pdf` builds (58pp; unchanged page count — the fixes were small numerical / wording corrections).
- `SOURCE_DATE_EPOCH=1746921600 make pdf` byte-identical across runs.

## Disposition

All 5 Importants were factual errors in the worked-encode and worked-decode walks, plus one wrong source-file pointer. Each would have actively misled a cross-implementer (the encode-totals arithmetic and the decode bit-count were both wrong by 1–2; the path-decl omission was structurally wrong). All folded inline. Phase 1.2 close: 0C/0I/0L/0N.

**Operational note:** the Phase 1.1 reviewer caught two technical errors in the BCH chapter; the Phase 1.2 reviewer caught five in the wire-format chapter. Both rounds confirm: bit-level chapters require source-cited drafting, not narrative paraphrase. Subsequent wire-format chapters (mk1 §II.2, ms1 §II.3) and the worked-encode examples therein should be drafted with explicit `cargo run` / `git show` cross-checks at draft time, not deferred to review.
