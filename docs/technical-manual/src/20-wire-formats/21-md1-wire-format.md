# md1 Wire Format

This chapter documents md1\index{md1}'s current wire format\index{wire format} at bit-level depth. The format is **v0.30** (a clean break from v0.x; decoders reject earlier versions via `Error::WireVersionMismatch`\index{Error::WireVersionMismatch}). For the normative spec, see `bg002h/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki` §"Specification" and `design/SPEC_v0_30_wire_format.md`. The reference implementation is in `crates/md-codec/src/`; this chapter cites specific source files where they pin a wire-format decision.

## Layer model

md1 has two layers (BIP draft §"Specification"):

- **Encoding layer.** Wraps a bit-packed bytecode payload in a codex32-style envelope: HRP `md` + separator `1` + payload + BCH checksum. The encoding layer is responsible for character-level error correction, single-vs-chunked dispatch, and cross-card identity (the `chunk_set_id`).
- **Bytecode layer.** The payload itself: a bit-aligned canonical compact binary encoding of a BIP-388 wallet-policy template plus optional TLV metadata. The bytecode layer is responsible for representing the policy structure; it has no character-level concerns.

The encoding layer wraps the bytecode in either a single string (small policies, ≤93 chars total) or a chunked sequence (larger policies, up to 64 chunks). The two layers compose: the encoder serializes a policy to bytecode, then the encoding layer frames the bytecode into one or more cards.

## Encoding-layer framing

### Card structure

Every md1 card is the concatenation:

```text
<HRP> <separator> <data part> <checksum>
   md         1   header + payload (5-bit symbols)   13 codex32 symbols
```

The HRP is the literal three bytes `md1` (the BCH check is over `hrp_expand("md") || data || checksum`; see §I.3 for the polymod path). The data part — header + payload — is one or more 5-bit codex32 symbols. The checksum is 13 codex32 symbols (the regular code; see §I.3 for the long-code situation).

The total card string length is capped at 93 characters (regular code). Payloads that would exceed this cap are chunked across multiple cards (see "Chunking" below).

### Header layout

The header carries the wire-format version and mode-dispatch flags. Two header shapes exist, distinguished **in-band** by the first symbol's bit 0 (BIP draft §"Auto-dispatch and safe rejection"; reference impl at `crates/md-codec/src/header.rs`):

**Single-string header**\index{single-string header (md1)} (5 bits = one codex32 character):

```text
| 1:divergent_paths | 4:version |
```

- Bit 4 (MSB): `divergent_paths`\index{divergent\_paths} flag. When `1`, the path-declaration carries per-`@N` divergent paths (one path per placeholder); when `0`, a single shared path applies to all placeholders.
- Bits 3–0: `version`. The 4-bit field absorbs the bit that v0.x reserved. v0.30 emits `0b0100` = 4.

**Chunked header**\index{chunked header (md1)} (37 bits = 8 codex32 characters with 3 bits of trailing slack absorbed by the symbol grid):

```text
| 4:version | 1:chunked=1 | 20:chunk_set_id | 6:count-1 | 6:index |
```

- Bits 36–33: `version`. Same 4-bit field as single-string. v0.30 = `0b0100`.
- Bit 32: `chunked` flag, always `1` for chunked headers. This is the bit-0 discriminator of the first 5-bit symbol; auto-dispatch sees it before any other field.
- Bits 31–12: `chunk_set_id`\index{chunk\_set\_id (md1)}, 20 bits. A deterministic per-encoding identifier shared by all chunks of one Template Card; used at reassembly time to detect mixed chunks. Derived from `Md1EncodingId`\index{Md1EncodingId} = leading 16 bytes of `SHA-256`\index{SHA-256}`(canonical bit-packed payload bytecode)`; the wire-level `chunk_set_id` is the leading 20 bits, MSB-first.
- Bits 11–6: `count − 1`, 6 bits. Range `1..64`.
- Bits 5–0: `index`, 6 bits. 0-indexed; range `0..(count − 1)`.

### Auto-dispatch and safe rejection\index{auto-dispatch}

The decoder reads the first 5-bit symbol and dispatches by bit 0 (BIP draft §"Auto-dispatch and safe rejection"):

| First 5 bits | Mode | Action |
|---|---|---|
| `bit0 = 0` | single-payload | treat bits 4..1 as `(divergent_paths, version[3..1])` and bit 0 as `version[0]` |
| `bit0 = 1` | chunked | treat bits 4..1 as the 4-bit version field; consume 32 more bits as chunk-header continuation |

The 4-bit version field has 16 representable values, but the auto-dispatch constrains the usable subset: redesigned wire-format versions MUST have `v0 = 0` (even values), because `v0 = 1` would be mis-classified as chunked. Combined with the collisions against v0.x's 3-bit version, the usable v0.30-family WF-redesign version set is **{4, 8, 12}**. v0.30 uses `version = 4`; future major breaks would use 8 then 12.

v0.x payloads are rejected cleanly without silent mis-decode:

| Input | First 5 bits | Decoder verdict |
|---|---|---|
| v0.x single-payload (version = 0) | `[paths][0][0][0][0]` | bit 0 = 0 → single-payload; version = 0 → `Error::WireVersionMismatch { got: 0 }` |
| v0.x chunked (version = 0) | `[0][0][0][1][0]` | bit 0 = 0 → single-payload; version = 2 → `Error::WireVersionMismatch { got: 2 }` |
| v0.30 single-payload | `[paths][0][1][0][0]` | bit 0 = 0 → single-payload; version = 4 → accepted |
| v0.30 chunked | `[0][1][0][0][1]` | bit 0 = 1 → chunked; version = 4 → accepted |

### Chunking

For payloads exceeding the single-string capacity (385 bits ≈ 48 bytes for the regular code), the encoder splits the bit-packed bytecode into N chunks. Each chunk:

- Carries the 37-bit chunk header (above) instead of the 5-bit payload header.
- Carries a fragment of the assembled bit stream as its payload.
- Has its own BCH checksum (each chunk is independently codex32-verified).

Reassembly: decoder collects N chunks, sorts by `index`, verifies all share the same `chunk_set_id`, concatenates their fragment bits, then runs the bytecode-layer decode on the joined stream prefixed with the 5-bit payload header reconstructed from the chunked-header version + a synthesised single-string-style `divergent_paths` bit recovered from the first chunk's reassembled bytecode.

Cross-chunk integrity check: after bytecode decode, recompute `Md1EncodingId` from the canonical bytecode and confirm the wire-carried `chunk_set_id` matches the leading 20 bits. Mismatch surfaces as `Error::ChunkSetIdMismatch`\index{Error::ChunkSetIdMismatch} `{ expected, derived }`.

## Bytecode layer

The bytecode is a packed bit stream (no byte alignment between sections or operators) carrying — in this order — the **5-bit header**, **origin-path declaration**, **use-site-path declaration**, **tree**, and **TLV section**. The bit stream is laid out MSB-first; the final byte is zero-padded if the total bit count is not a multiple of 8.

### Origin-path declaration\index{OriginPath}

The origin-path declaration encodes the BIP-32 derivation prefix shared by all placeholders (shared mode) or per-placeholder (divergent mode). It uses a depth-prefixed component encoding documented at BIP draft §"Origin path declaration".

In **shared mode** (`divergent_paths = 0`):

```text
| 4:depth | depth × component |
```

In **divergent mode** (`divergent_paths = 1`):

```text
| 4:depth_0 | depth_0 × component | 4:depth_1 | depth_1 × component | … |
```

One path per `@N` placeholder, in placeholder-index order. The total number of paths is determined by `n` (the placeholder count, derived from the tree).

Each component is encoded as:

```text
| 1:hardened | 7-or-more bits varint:index |
```

The variable-width integer encoding is a length-prefixed varint (LP4-ext varint\index{LP4-ext varint} per BIP draft §"LP4-ext varint"): the first 4 bits are the length minus 1 (giving 1–16 four-bit groups), followed by the actual index in MSB-first bit order.

### Use-site-path declaration\index{use-site-path declaration}

Documents the *suffix* path applied at the descriptor's wildcard position (e.g., `/0/*`, `/<0;1>/*`). Always present (even for descriptors without an explicit suffix; the empty suffix encodes as `multipath = null + wildcard_hardened = 0`).

### Tree

The bytecode's tree section is a tag-tree of operators. Each operator's wire shape is:

```text
| 6:tag | <operator-specific body> | <children, recursively> |
```

Tags are drawn from a 6-bit primary code space (`0x00`–`0x3F`); primary `0x3F` is the extension prefix followed by a 4-bit subcode (no extension subcodes are allocated in v0.30 — the entire 16-slot subspace is reserved).

### TLV section\index{TLV section}

A bit-aligned sequence of optional metadata blocks (Fingerprints, Pubkeys, OriginPathOverrides, Unknown). Each TLV is identified by a 5-bit tag from a **separate** namespace from the bytecode 6-bit space (BIP draft §"TLV tag allocations"). The TLV section terminates by end-of-bytecode rather than a length prefix — see BIP draft §"End-of-section detection (rollback-as-padding)" for the trailing-padding tolerance at the section boundary.

The reference implementation pins TLV handling in `crates/md-codec/src/tlv.rs`.

## Tag table (v0.30)

The 36 allocated primary tags. Authoritative source: `crates/md-codec/src/tag.rs`. The table below reproduces the BIP draft's §"Tag table".

| Primary | Operator | Operator data | Children | Notes |
|---|---|---|---|---|
| `0x00` | `wpkh()`\index{Tag::Wpkh} | `key_index`\index{key\_index} (kiw bits) | 0 | Top-level: ACTIVE. As `sh(wpkh)` inner: ACTIVE. |
| `0x01` | `tr()`\index{Tag::Tr} | 1-bit `is_nums`\index{is\_nums} + (`key_index` if `!is_nums`) + 1-bit `has_tree` + optional tree | 0 or 1 | See "NUMS encoding" below. |
| `0x02` | `wsh()`\index{Tag::Wsh} | — | 1 | Top-level: ACTIVE. As `sh(wsh)` inner: ACTIVE. |
| `0x03` | `sh()`\index{Tag::Sh} | — | 1 | Top-level only (per Sh wrapper restriction matrix). |
| `0x04` | `pkh()`\index{Tag::Pkh} | `key_index` (kiw bits) | 0 | Top-level: REJECTED. |
| `0x05` | TapTree inner-node | — | 2 (left / right subtree) | Multi-leaf TapTree branching; recursive. |
| `0x06` | `multi()`\index{Tag::Multi} | 5-bit `k−1` + 5-bit `n−1` + n × `key_index` (kiw bits) | 0 (raw indices) | Multi-family raw-index packing. |
| `0x07` | `sortedmulti()` | as `multi()` | 0 | |
| `0x08` | `multi_a()` | as `multi()` | 0 (taproot context) | |
| `0x09` | `sortedmulti_a()` | as `multi()` | 0 (taproot context) | |
| `0x0A` | `pk_k()`\index{Tag::PkK} (sugar `pk()`) | `key_index` (kiw bits) | 0 | |
| `0x0B` | `pk_h()`\index{Tag::PkH} (sugar `pkh()`) | `key_index` (kiw bits) | 0 | |
| `0x0C` | `c:` (Check)\index{Tag::Check} | — | 1 | Walker normalisation: never wraps a bare `pk_k` / `pk_h` child on the wire. |
| `0x0D` | `v:` (Verify) | — | 1 | |
| `0x0E` | `s:` (Swap) | — | 1 | |
| `0x0F` | `a:` (Alt) | — | 1 | |
| `0x10` | `d:` (DupIf) | — | 1 | |
| `0x11` | `j:` (NonZero) | — | 1 | |
| `0x12` | `n:` (ZeroNotEqual) | — | 1 | |
| `0x13` | `and_v` | — | 2 | |
| `0x14` | `and_b` | — | 2 | |
| `0x15` | `andor` | — | 3 | |
| `0x16` | `or_b` | — | 2 | |
| `0x17` | `or_c` | — | 2 | |
| `0x18` | `or_d` | — | 2 | |
| `0x19` | `or_i` | — | 2 | |
| `0x1A` | `thresh()`\index{Tag::Thresh} | 5-bit `k−1` + 5-bit `n−1` + n × Node | n (full Node children) | Note: thresh keeps `Body::Variable` (per-child tags), unlike multi-family. |
| `0x1B` | `after()` | 32-bit absolute timelock | 0 | |
| `0x1C` | `older()` | 32-bit relative timelock | 0 | |
| `0x1D` | `sha256()` | 32-byte hash | 0 | |
| `0x1E` | `hash160()` | 20-byte hash | 0 | |
| `0x1F` | `hash256()` | 32-byte hash | 0 | Promoted to primary in v0.30. |
| `0x20` | `ripemd160()` | 20-byte hash | 0 | Promoted to primary in v0.30. |
| `0x21` | raw `pk_h` (`expr_raw_pkh`) | 20-byte hash | 0 | Promoted to primary in v0.30. Reachable via `c:` wrapper only. |
| `0x22` | `FALSE` (literal `0`) | — | 0 | Promoted to primary in v0.30. |
| `0x23` | `TRUE` (literal `1`) | — | 0 | Promoted to primary in v0.30. |
| `0x24`–`0x3E` | *reserved* | — | — | Decoder rejects with `Error::TagOutOfRange { primary }`. |
| `0x3F` | *extension prefix* | 4-bit subcode follows | — | No subcodes allocated in v0.30. Decoder consumes the subcode and rejects. |

`kiw` = key-index width = `⌈log₂(n)⌉` bits, where `n` is the policy's placeholder count. For `n = 1`, `kiw = 0` and `key_index` is zero bits wide (no wire representation).

## Body shapes

Three distinct body shapes encode the operands of an operator:

- **Single-key bodies** (`wpkh`, `pkh`, `tr` without NUMS, `pk_k`, `pk_h`): one `key_index` field of width `kiw` bits.
- **Multi-family bodies**\index{multi-family bodies} (`multi`, `sortedmulti`, `multi_a`, `sortedmulti_a`): `5-bit (k−1) | 5-bit (n−1) | n × key_index (kiw bits)` — **raw indices, not tagged Nodes**. The decoder MUST treat the body as exactly n raw kiw-bit fields and reject any tag byte in this region with `Error::OperatorContextViolation`\index{Error::OperatorContextViolation} `{ tag, context: MultiBody }`.
- **Thresh body** (`thresh`): `5-bit (k−1) | 5-bit (n−1) | n × Node` — full Node children, distinct from multi-family raw-index packing.

The reference implementation pins these as `Body::KeyArg`\index{Body::KeyArg}, `Body::MultiKeys`\index{Body::MultiKeys} `{ k, indices }`, and `Body::Variable`\index{Body::Variable} `{ k, children }` respectively, in `crates/md-codec/src/tree.rs`.

## NUMS encoding for `tr()`

The `tr()` operator (`0x01`) has the special body shape:

```text
| 6:Tag::Tr | 1:is_nums | [kiw:key_index iff !is_nums] | 1:has_tree | [tree iff has_tree] |
```

When `is_nums = 1`, the Taproot internal key\index{taproot internal key} is the BIP-341\index{BIP-341} NUMS\index{NUMS} H-point:

```text
50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0
```

and the `key_index` field is **suppressed entirely on the wire** (no kiw-bit field follows). When `is_nums = 0`, a `kiw`-bit `key_index` field follows. Values `≥ n` are rejected with `Error::NUMSSentinelConflict`\index{Error::NUMSSentinelConflict} (the field name is historical; the rejection fires for `is_nums = 0 ∧ key_index ≥ n`).

**Canonicalisation invariant.** Encoders MUST emit `is_nums = 1` iff the descriptor's `tr()` internal key is exactly the BIP-341 NUMS H-point. Encoders MUST emit `is_nums = 0` with `key_index = i` (`0 ≤ i < n`) for any `@i` placeholder internal key.

**History.** The encoding evolved across three versions:

- **v0.17**: separate `Tag::TrUnspendable` at primary `0x1F` + ext sub-code `0x05`.
- **v0.18**: sentinel `key_index = n` in a widened-by-one kiw field.
- **v0.30**: explicit `is_nums` flag (this version). Restores `kiw = ⌈log₂(n)⌉` (saves 1 bit at each `n ∈ {1, 2, 4, 8, 16, 32}`).

## Walker normalisation\index{walker normalisation}

The encoder emits a bare `Tag::PkK` or `Tag::PkH` at the c:-position (instead of wrapping with an explicit `Tag::Check`). The renderer reconstructs the `c:` wrapper at key-leaf positions; this preserves type correctness in the rendered descriptor without spending wire bits on a wrapper that is structurally implied. The invariant is documented in BIP draft §"Round-trip canonical form"; the reference implementation applies the inverse reconstruction in the miniscript-to-AST path (see `crates/md-codec/src/to_miniscript.rs`).

The canonical form invariant: every `Tag::PkK` and `Tag::PkH` appearing as a direct child of a wrapper-tagged operator is rendered with the `c:` desugar at that position; every `Tag::PkK` or `Tag::PkH` appearing as a multi-family child is left bare (multi-family operands are already type-checked by their parent).

## Canonicality rules\index{canonicality rules}

The bytecode has a small set of rules that an encoder MUST follow and a decoder MUST verify. Violations surface as specific `Error` variants:

1. **`is_nums` canonicalisation.** As described above. Non-canonical encodings (e.g., `is_nums = 0` with the literal NUMS H-point xpub for `key_index = i`) MUST be rejected by encoders.
2. **Top-level wrapper.** The root tag MUST be one of `{Sh, Wsh, Wpkh, Pkh, Tr}`. Other top-level tags surface as `Error::OperatorContextViolation { tag, context: ContextKind::TopLevel }` (added in v0.31).
3. **Multi-family raw-index body.** Multi-family bodies are exactly `5 + 5 + n × kiw` bits. Any tag byte appearing in this region is `Error::OperatorContextViolation { tag, context: MultiBody }`.
4. **Tap-leaf admissible operators.** Inside a TapTree leaf, only BIP-342 admissible operators (per the leaf-allow-list) are permitted; others surface as `Error::ForbiddenTapTreeLeaf`\index{Error::ForbiddenTapTreeLeaf}.
5. **Mid-stream padding.** Trailing non-zero pad bits cannot have been produced by a conforming encoder. md-codec's decoder does not carry a dedicated pad-bit-rejection error variant; instead, non-zero pad bits either decode into a malformed TLV header (surfacing as `Error::MalformedHeader { detail }`) or run the bitstream short (`Error::BitStreamTruncated`). Trailing **zero** pad bits at the TLV-section boundary are tolerated by the rollback-as-padding mechanism (BIP draft §"TLV section / End-of-section detection"; reference impl at `crates/md-codec/src/tlv.rs::TlvSection::read`).

## Worked encode: `wpkh(@0/<0;1>/*)` (corpus vector `wpkh_basic`)

Input: `wpkh(@0/<0;1>/*)`, no shared path declared, no TLVs.

Encoder bytecode steps:

1. **Header**: `divergent_paths = 0`, `version = 4` → bits `00100` (5 bits).
2. **Origin-path declaration**: shared-mode. The path-decl always opens with a 5-bit `n − 1` field then a 4-bit `depth` field; for `n = 1` and depth-0 that is `00000 0000` (9 bits). No components follow.
3. **Use-site-path declaration**: `multipath = [0, 1]`, `wildcard_hardened = 0` → bit-encoded per BIP-389 multipath rules (variable width).
4. **Tree**: `wpkh(@0)` → `Tag::Wpkh` (6 bits) + `key_index = 0` (kiw = 0 bits for `n = 1`, suppressed) = 6 bits.
5. **TLV section**: empty.

Total used bytecode: 5 (header) + 9 (path-decl: 5 `n−1` + 4 `depth`) + use-site (variable) + 6 (wpkh) + 0 (TLV) = 36 bits used, padded to the next 5-bit boundary = 40 bits on wire = 8 data symbols. Combined with 13 check symbols + the 3-character `md1` HRP+separator: **24 characters total**.

Resulting card: `md1yqpqqxqq8xtwhw4xwn4qh` (matches corpus vector `crates/md-codec/tests/vectors/wpkh_basic.phrase.txt`).

For the full bit-by-bit trace, see BIP draft §"Bit-layout example" (§II.1's encode walks are summary-shaped; the BIP carries the bit-by-bit layout).

## Worked decode: `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` (corpus vector `wsh_multi_2of3`)

Card: `md1yzpqqxppsgsc8dua4tu0kekyl`. 28 characters total = 3 (HRP + separator `md1`) + 25 (data + check symbols).

Decoder steps:

1. **HRP + BCH.** Verify `polymod(hrp_expand("md") || data_symbols || check_symbols) == MD_REGULAR_CONST`. Pass.
2. **Header.** First 5-bit symbol = the integer value of character `y` = `4` = bits `00100`: `divergent_paths = 0`, `version = 4`. Single-payload mode.
3. **Origin path.** Shared-mode. Read `n − 1` (5 bits) = 2 (so `n = 3`); read `depth` (4 bits) = 0 (no components).
4. **Use-site path.** Multipath `[0, 1]`, wildcard non-hardened.
5. **Tree.** Read 6-bit tag = `Tag::Wsh` (`0x02`); 1 child follows. Read 6-bit tag = `Tag::Multi` (`0x06`); body is `5-bit (k−1=1) | 5-bit (n−1=2) | 3 × kiw-bit key_index` where `kiw = ⌈log₂(3)⌉ = 2`, so 6 bits of indices = `00 01 10` = indices `0, 1, 2`.
6. **TLV.** Empty.
7. **Reconstruct.** Tree: `wsh(multi(2, @0, @1, @2))`. With the use-site path `<0;1>/*` and no per-`@N` overrides, the rendered descriptor is `wsh(multi(2, @0/<0;1>/*, @1/<0;1>/*, @2/<0;1>/*))`. Matches the corpus vector at `crates/md-codec/tests/vectors/wsh_multi_2of3.template`.

The full decode produces 2 + 6 + 4 + use-site + 24 (= 6 + 5 + 5 + 6 indices + chunk-grid alignment) bits, which after BCH-symbol packing exactly fills the corpus card length.

## History note: retired wire-layer dictionaries

Pre-v0.11 wire formats carried byte-aligned framing tags for path-dictionary lookups (`Placeholder`, `SharedPath`, `Fingerprints`, `OriginPaths` at `0x33`–`0x36` in an 8-bit operator-tag namespace). v0.11 retired all four:

- Path-decl framing moved into the bit-aligned §"Origin path declaration" (the v0.10-era `Tag::OriginPaths`\index{Tag::OriginPaths} = `0x36` was retired with v0.11's dictionary cleanup).
- Key references moved into inline `key_index` bit fields.
- Fingerprints, xpubs, and per-`@N` overrides moved into the TLV section.

The full retirement record is in `bg002h/descriptor-mnemonic/design/SPEC_v0_11_wire_format.md §1.4` (cited verbatim: "Wire-layer dictionaries (path, use-site-path, shape). Considered and rejected for architectural cleanliness"). v0.30 retains the post-v0.11 split and additionally widens the bytecode primary tag space from 5-bit to 6-bit, promoting 5 operators (`Hash256`, `Ripemd160`, `RawPkH`, `False`, `True`) from the v0.x extension subspace to direct primary slots `0x1F`–`0x23` for 4-bit-per-occurrence savings.

The companion path-dictionary mirror invariant with mk1 (formalised in v0.9 of md-codec, retired post md-codec v0.11) is recorded in `bg002h/descriptor-mnemonic/CLAUDE.md`.

## Cross-references

- §I.3 covers the BCH plumbing (the encoding-layer outer wrapper).
- §III.1 covers descriptor → miniscript → address (how a decoded bytecode tree becomes a derived Bitcoin address).
- §III.2 covers the v0.32 shape-coverage extension via `to_miniscript_descriptor`.
- §IV.2 covers the cross-card invariants (`policy_id_stub`, `chunk_set_id`, multiset xpub-match).
- §V.1 covers the `md-codec` Rust API surface.

The reference implementation:

- `crates/md-codec/src/header.rs` — header parse + auto-dispatch.
- `crates/md-codec/src/tag.rs` — primary + extension tag space.
- `crates/md-codec/src/tree.rs` — `Body` variants, encode/decode walker.
- `crates/md-codec/src/canonicalize.rs` — placeholder-ordering canonicalisation (permutes `@N` indices so first-encountered is `@0`).
- `crates/md-codec/src/to_miniscript.rs` — bare-PkK/PkH → `c:`-wrapped reconstruction.
- `crates/md-codec/src/origin_path.rs` — path-decl encoding.
- `crates/md-codec/src/tlv.rs` — TLV section.
- `crates/md-codec/src/chunk.rs` — chunked-card framing.
- `crates/md-codec/tests/vectors/` — corpus vectors (template + phrase + bytes-hex + descriptor-json quadruples).
