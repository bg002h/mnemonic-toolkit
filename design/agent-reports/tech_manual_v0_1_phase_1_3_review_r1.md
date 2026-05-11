# tech-manual v0.1 Phase 1.3 review r1 — mk1 wire format (commit f609479)

Reviewed: `docs/technical-manual/src/20-wire-formats/22-mk1-wire-format.md` (~310 lines, ~14pp). Cross-checked against `/scratch/code/shibboleth/mnemonic-key/`: `design/SPEC_mk_v0_1.md`, `bip/bip-mnemonic-key.mediawiki`, and `crates/mk-codec/src/{consts,key_card,error}.rs` + `bytecode/` + `string_layer/`.

## Critical: 0

## Important: 1 (folded inline)

### I1 — V4 decode step 1 uses data-part length (108) for total string length of chunk 0

Chapter line 335 wrote: "Verify `polymod(...) == MK_LONG_CONST` for the **108-char chunk 0**". The V4 chunk 0 has the same 53-byte fragment as V1 chunk 0, producing the same data-part length (108 5-bit symbols) and therefore the same **111-character total string length** (3 HRP+sep + 108 data). The chapter's own V1 encode walk 15 lines above correctly states "Total string length `3 (HRP+sep) + 108 = 111` characters" for the identical case.

Cross-check: the actual V4 chunk 0 string from `transcripts/mk1-decode-bip84-no-fingerprint.cmd` is 111 characters.

This is the same arithmetic-notation error class the Phase 1.2 reviewer caught five times (conflating data-part-symbol count with total-string-character count).

**Fix applied:** rewrote step 1 to "the 111-char chunk 0 (108-char data part + 3-char HRP+sep, long-code); same with `MK_REGULAR_CONST` for the 74-char chunk 1 (71-char data part + 3-char HRP+sep, regular-code)." The explicit decomposition makes the data-part vs total-string distinction visible at every quoted number.

## Low: 1 (folded inline)

### L1 — Range notation `0x08..0x10` ambiguous between inclusive and exclusive

Chapter line 195 used Rust's `..` open-range notation for reserved-indicator ranges, which in Rust is exclusive-end (`0x08..0x10` excludes `0x10`). The actual SPEC §3.5 reservation includes both `0x10` and `0xFD`. Prose may read this as inclusive, but the convention is ambiguous.

**Fix applied:** changed to `0x08..=0x10` and `0x18..=0xFD` (Rust's inclusive-end notation), removing the ambiguity.

## Nit: 0

## Verified correct (sample of clean items)

- All 14 entries of the standard-path table (`0x01..=0x07` mainnet, `0x11..=0x17` testnet) match `crates/mk-codec/src/bytecode/path.rs::STANDARD_PATHS` exactly.
- NUMS-derived constants (`MK_REGULAR_CONST = 0x1062435f91072fa5c`, `MK_LONG_CONST = 0x41890d7e441cbe97273`) match `consts.rs:18,21` verbatim. SHA-256 reproducer formula correct.
- All 19 Error variants cited in the validity-rules table exist in `crates/mk-codec/src/error.rs` with the exact names quoted.
- Bytecode header bit layout (`version` in bits 7..4; bit 2 fingerprint flag; bits 0, 1, 3 reserved) matches `bytecode/header.rs:20-26`.
- Valid v0.1 bytecode-header bytes (`0x00`, `0x04`) consistent with `bytecode/header.rs::round_trip_no_fingerprint` + `round_trip_with_fingerprint` test fixtures.
- 8-symbol chunked-header layout (`version | type | csid×4 | total_chunks-wire | chunk_index`) matches `string_layer/header.rs:67-101`.
- chunk_set_id 20-bit packing (bits 19..15 in symbol 2, ..., 4..0 in symbol 5) matches `string_layer/header.rs:138-141`.
- `total_chunks` off-by-one wire encoding (`count − 1`) matches `string_layer/header.rs:88,146`.
- Compact-73 layout (`version(4) + parent_fingerprint(4) + chain_code(32) + public_key(33) = 73`) matches `bytecode/xpub_compact.rs:43-54`.
- Xpub reconstruction rule (`depth = component_count(path)`, `child_number = last_component(path)`) matches `bytecode/xpub_compact.rs:86-95`.
- Cross-chunk-hash reassembly check (`SHA-256(reassembled_bytecode_without_hash)[0..4]`) matches `string_layer/chunk.rs:193-200`.
- All V1 worked-encode bit-math (53-byte fragment → 85 symbols + 1 zero pad bit; 35-byte fragment → 56 symbols no pad; pre-checksum 93 → long-code 108; pre-checksum 64 → regular-code 77) re-derived from first principles, correct.
- All V4 worked-decode bit-math (53-byte chunk 0 fragment; 31-byte chunk 1 fragment = 27 bytecode bytes + 4-byte hash; 50 symbols → 31 bytes with 2 zero pad bits) re-derived, correct.
- V1 byte structure (header `0x04`, stub `11223344`, fp `aabbccdd`, path indicator `0x05`, xpub.version `0488b21e`, parent_fp `10203001`, chain_code 32×`ab`, public_key starting `031b84c5...`) verified against canonical bytecode hex.
- V1 cross_chunk_hash `83 bb 26 2d` verified via Python SHA-256 on the bytecode.
- mk-codec version "v0.2.2" claim consistent with HEAD `Cargo.toml`.
- Cross-references to §I.2, §I.3, §IV.1, §IV.2, §V.2 match `SPEC_tech_manual_v1.md` §4.2 layout.
- History note (path-dictionary mirror retirement) cites `descriptor-mnemonic/CLAUDE.md` and `mnemonic-key/design/FOLLOWUPS.md::path-dictionary-mirror-stewardship` — both confirmed by inspection.
- Both transcripts (`mk1-decode-bip48-multisig`, `mk1-decode-bip84-no-fingerprint`) round-trip cleanly against HEAD `mk-cli` v0.2.0 binary via `verify-examples.sh`.

## Post-fix verification

- `make lint` 6/6 green.
- `make verify-examples` 4/4 transcripts pass.
- `SOURCE_DATE_EPOCH=1746921600 make pdf` byte-identical across clean rebuilds (72pp).

## Disposition

I1 was a direct precedent of the Phase 1.2 arithmetic-notation error class (one occurrence instead of Phase 1.2's five). L1 was a prose-notation cleanup. Both folded inline. Phase 1.3 close: 0C/0I/0L/0N.

**Operational note:** the bit-math drafting discipline — re-derive each number from first principles using Python against the SHA-pinned vector hex *before* writing the totals row — caught Phase 1.3's structural bit math correctly on first pass. The one residual Important was a *notation* drift (data-part vs total-string), not arithmetic. Subsequent wire-format chapter (ms1 §II.3 at Phase 1.4) should apply the explicit decomposition convention (`3 (HRP+sep) + N (data part) = M (total)`) at every quoted character count to eliminate this class entirely.
