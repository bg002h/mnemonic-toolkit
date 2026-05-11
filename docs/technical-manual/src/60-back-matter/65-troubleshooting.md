# Troubleshooting

This appendix maps each `Error` variant surfaced by Parts I + II to a likely cause and the remediation pointer. v0.1 covers a curated subset of wire-format-layer variants for each of the three codecs (the variants reachable through the failure modes documented in Parts I + II). Part V (added at tech-manual-v0.4) populates the full library-API error taxonomy.

For the authoritative source of each variant — and the complete enum surface — see the cited `error.rs` file in the relevant repo.

## md1 — `md-codec::Error`

Source: `bg002h/descriptor-mnemonic/crates/md-codec/src/error.rs`.

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `WireVersionMismatch { got }` | The string carries a v0.x or unknown wire-format version field. v0.30 expects `version = 4`. | §II.1 "Auto-dispatch and safe rejection" — re-issue the bundle with a v0.30 encoder, or re-encode the policy from canonical inputs. |
| `MalformedHeader { detail }` | Header bits don't satisfy the v0.30 layout (e.g., reserved bit set, in-band chunked discriminator inconsistent). | §II.1 "Header layout"; check the encoder for stale bit-allocations. |
| `TagOutOfRange { primary }` | A 6-bit primary tag fell in the reserved range `0x24..=0x3E`, or the extension prefix `0x3F` was followed by an unallocated 4-bit subcode (all subcodes are reserved in v0.30). | §II.1 "Tag table (v0.30)"; check encoder hasn't drifted to a post-v0.30 tag allocation. |
| `OperatorContextViolation { tag, context }` | A tag appeared in an invalid position. `context: TopLevel` rejects roots outside `{Sh, Wsh, Wpkh, Pkh, Tr}`; `context: TapLeaf` rejects a non-BIP-342 fragment inside a TapTree leaf. `context: MultiBody` is structurally unreachable in v0.30 — multi-family bodies carry raw kiw-bit indices, not child tags (`md-codec/src/error.rs:197-207`); the variant is retained for completeness. | §II.1 "Canonicality rules" rules 2/4. |
| `NUMSSentinelConflict` | `is_nums = 0` but `key_index ≥ n` (the historical sentinel position). | §II.1 "NUMS encoding for `tr()`"; encoder must emit `is_nums = 1` for the NUMS H-point and `is_nums = 0` with `key_index < n` for any `@i` placeholder. |
| `ForbiddenTapTreeLeaf` | A tap-leaf carries a fragment outside the BIP-342 admissible set. | §II.1 "Canonicality rules" rule 4. |
| `ChunkSetIdMismatch { expected, derived }` | After bytecode-layer decode, the recomputed leading 20 bits of `Md1EncodingId` don't match the wire-carried `chunk_set_id`. Mixed chunks from different encodings, or post-encoding payload tampering. | §II.1 "Chunking"; verify all chunks in the input set originate from the same encode invocation. |

md-codec has no dedicated pad-bit-rejection variant (unlike mk-codec, see below). Non-zero pad bits at the bytecode-section boundary surface as `MalformedHeader` or `BitStreamTruncated` depending on how the TLV-section parser interprets them. Trailing **zero** pad bits at the TLV-section boundary are tolerated by the rollback-as-padding mechanism — see §II.1 "Canonicality rules" rule 5 and "TLV section".

## mk1 — `mk-codec::Error`

Source: `bg002h/mnemonic-key/crates/mk-codec/src/error.rs`.

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `InvalidHrp(String)` | HRP is not `mk` (or the input is not a valid bech32-shaped string at all). | §II.2 "Card structure"; mk1 strings start with `mk1` (lowercase). |
| `MixedCase` | Input string mixes uppercase and lowercase characters (BIP-93 prohibits). | Re-engrave the card consistently; codex32 is single-case. |
| `InvalidStringLength(usize)` | Data-part length in the reserved gap `94..=95` or outside BIP-93 brackets. | §II.2 "Card structure"; valid lengths are 14..=93 (regular) and 96..=108 (long). |
| `InvalidChar { ch, position }` | Input data-part character is not in the codex32 alphabet (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`). The offending character + 0-indexed position are surfaced for transcription-error feedback. | §I.3 "The codex32 alphabet"; check for visually-confusable substitutions (`0`/`O`, `1`/`l`/`I`). |
| `BchUncorrectable(String)` | BCH detected more errors than it can correct. | §I.3 "Error-detection guarantees" — up to 4 random substitutions correctable; beyond that, re-engrave from the canonical source. |
| `UnsupportedCardType(u8)` | The 5-bit chunk-type byte is in the reserved range `0x02..=0x1F`. | §II.2 "String-layer header"; only `0x00` (SingleString, unreachable in v0.1) and `0x01` (Chunked) are valid. |
| `MalformedPayloadPadding` | Trailing 5-bit symbol pad bits non-zero after BCH. | §II.2 "Canonicality and validity rules" rule 14. |
| `ChunkSetIdMismatch` | Chunks of one card disagree on the 20-bit `chunk_set_id`. | §II.2 "Chunking and cross-chunk integrity"; all chunks of one card must share `chunk_set_id`. |
| `ChunkedHeaderMalformed(String)` | Bad `chunk_index`, gap, duplicate, or `total_chunks` disagreement across the chunk set. | §II.2 "String-layer header"; check for missing/duplicate cards. |
| `MixedHeaderTypes` | Input combines `SingleString` and `Chunked` chunks in one decode invocation. | §II.2 "String-layer header"; v0.1 emits only chunked cards. |
| `CrossChunkHashMismatch` | After reassembly, the trailing 4-byte hash ≠ `SHA-256(reassembled_bytecode)[0..4]`. Content drift across chunks. | §II.2 "Chunking and cross-chunk integrity". |
| `UnsupportedVersion(u8)` | The bytecode-header version field is not `0` (v0.1's only valid value), or the string-layer header carries an unknown 5-bit version. | §II.2 "Bytecode header" / "String-layer header". |
| `ReservedBitsSet` | Bytecode-header bits 0, 1, or 3 are set (all reserved in v0.1). | §II.2 "Bytecode header"; valid v0.1 header bytes are exactly `0x00` and `0x04`. |
| `InvalidPolicyIdStubCount` | `stub_count == 0`. | §II.2 "Policy ID stub"; at least one stub is required. |
| `InvalidPathIndicator(u8)` | A standard-table indicator outside the 14-entry table. | §II.2 "Origin path" — explicit-path escape (`0xFE`) is the alternative. |
| `PathTooDeep(u8)` | Explicit-path `count == 0` or `count > 10`. | §II.2 "Origin path" Case B; the 10-component cap bounds chunk-size attacks. |
| `InvalidPathComponent(String)` | LEB128 overflow or 6th continuation byte. | §II.2 "Origin path" Case B; components are u32 BIP-32 child numbers, max 5 bytes each. |
| `InvalidXpubVersion(u32)` | xpub version bytes ≠ known network prefix (`0x0488B21E` mainnet, `0x043587CF` testnet). | §II.2 "Network detection". |
| `InvalidXpubPublicKey(String)` | xpub public_key is not a valid compressed secp256k1 point. | §II.2 "Xpub compact-73". |
| `UnexpectedEnd` | Decoder hit end-of-stream mid-field. | §II.2 "Payload field order"; check the card is not truncated. |
| `TrailingBytes` | Decoder consumed all expected fields but bytes remain after the 73-byte compact xpub. | §II.2 "Payload field order"; check the card was not double-encoded or appended to. |
| `CardPayloadTooLarge { bytecode_len, max_supported }` | Canonical bytecode exceeds the v0.1 chunking capacity (32 × 53 − 4 = 1692 bytes). Reachable only through pathological hand-constructed inputs; typical mk1 cards land well below this ceiling. | §II.2 "Length envelope"; this is an encoder-side guard. |

## ms1 — `ms-codec::Error`

Source: `bg002h/mnemonic-secret/crates/ms-codec/src/error.rs`.

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `Codex32(<inner>)` | Upstream BIP-93 parse / checksum failure (delegated from `rust-codex32`). Covers bad checksum, bad character, mixed case, length out of BIP-93 brackets. | §I.3; §II.3 "Encoding-layer framing" rule 1. |
| `WrongHrp { got }` | HRP ≠ `ms`. | §II.3 "BIP-93 wire fields"; the HRP is structurally inseparable from the BCH check. |
| `ThresholdNotZero { got }` | Threshold byte ≠ `'0'`. v0.1 is single-string only. | §II.3 "BIP-93 wire fields" rule 3. |
| `ShareIndexNotSecret { got }` | Share-index byte ≠ `'s'`. BIP-93 requires `'s'` for threshold=0 (Unshared Secret form). | §II.3 "BIP-93 wire fields" rule 4. |
| `TagInvalidAlphabet { got }` | id-field bytes not in the codex32 alphabet (defensive; unreachable after BIP-93 parse). | §II.3 "Tag type". |
| `UnknownTag { got }` | id is structurally valid but not a member of `RESERVED_TAG_TABLE`. | §II.3 "`RESERVED_TAG_TABLE`". |
| `ReservedTagNotEmittedInV01 { got }` | id is `seed`, `xprv`, `mnem`, or `prvk` — reserved-not-emitted in v0.1. | §II.3 "`RESERVED_TAG_TABLE`"; only `entr` is emittable in v0.1. |
| `ReservedPrefixViolation { got }` | Payload prefix byte ≠ `0x00`. | §II.3 "The `0x00` reserved-prefix byte" — v0.2 promotes the byte to a type discriminator. |
| `UnexpectedStringLength { got, allowed }` | Total string length outside `{50, 56, 62, 69, 75}`. This rule rejects every BIP-93 long-code string in v0.1. | §II.3 "Length envelope (5 valid v0.1 lengths)". |
| `PayloadLengthMismatch { tag, expected, got }` | After stripping the prefix byte, payload length ∉ `{16, 20, 24, 28, 32}` for tag `entr`. | §II.3 "`Payload::Entr` and entropy-length validation". |
