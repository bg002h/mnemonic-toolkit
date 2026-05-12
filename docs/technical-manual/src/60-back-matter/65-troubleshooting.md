# Troubleshooting

This appendix maps each `Error` variant surfaced by the four reference crates to a likely cause and a remediation pointer. v1.0 covers every Error variant across all four crates: 43 in md-codec, 22 in mk-codec, 10 in ms-codec, and 26 in mnemonic-toolkit's `ToolkitError` — 101 variants total. Within each section, rows are clustered by emit-site (mirroring the §V.1.4 / §V.2.4 / §V.3.4 / §V.4.4 Error-taxonomy tables) and ordered to match the source-file declaration order within each cluster.

For the authoritative source of each variant — and the complete enum surface — see the cited `error.rs` file in the relevant repo and the corresponding Part V Error-taxonomy table.

## md1 — `md-codec::Error`

Source: `bg002h/descriptor-mnemonic/crates/md-codec/src/error.rs` (43 variants). Display + emission-site catalogue: §V.1.4.

### Bitstream, header, varint

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `BitStreamTruncated { requested, available }` | Decoder read past end-of-stream. Truncated card, hand-edited string, or wire format newer than this decoder. | §II.1 "Bytecode-section parser"; verify the input cards are complete + canonical. |
| `WireVersionMismatch { got }` | The string carries a v0.x or unknown wire-format version field. v0.30 expects `version = 4`. | §II.1 "Auto-dispatch and safe rejection" — re-issue the bundle with a v0.30 encoder, or re-encode the policy from canonical inputs. |
| `MalformedHeader { detail }` | Header bits don't satisfy the v0.30 layout (e.g., reserved bit set, in-band chunked discriminator inconsistent). Declared but currently unconstructed at HEAD (§V.1.7); reserved for future header-layer rejections. | §II.1 "Header layout"; check the encoder for stale bit-allocations. |
| `VarintOverflow { value }` | LP4-ext varint value exceeds the 29-bit single-extension range. Hand-constructed input or future-version length field. | §V.1.4 "Bitstream, header, varint"; the wire layer caps single-extension varints at `2^29 − 1`. |

### Path-decl + use-site

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `PathDepthExceeded { got, max }` | Origin-path depth exceeds `MAX_PATH_COMPONENTS = 15`. | §II.1 "Path encoding"; bound the engraved path to ≤ 15 components. |
| `KeyCountOutOfRange { n }` | `n` outside `1..=32`. | §II.1 "Canonicality rules" rule 1; v0.30 caps `n` at 32 keys. |
| `DivergentPathCountMismatch { n, got }` | Number of divergent paths under `OriginPaths` ≠ `n`. | §II.1 "Per-`@N` divergent paths"; pad or trim to match the key count exactly. |
| `AltCountOutOfRange { got }` | Use-site multipath alt-count ∉ `2..=9`. | §II.1 "Use-site multipath"; multipath `<a;b;...>` carries 2 to 9 alternatives. |

### Tag + tree

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `TagOutOfRange { primary }` | A 6-bit primary tag fell in the reserved range `0x24..=0x3E`, or the extension prefix `0x3F` was followed by an unallocated 4-bit subcode (all subcodes are reserved in v0.30). | §II.1 "Tag table (v0.30)"; check encoder hasn't drifted to a post-v0.30 tag allocation. |
| `ThresholdOutOfRange { k }` | k-of-n threshold `k` outside `1..=32`. | §II.1 "Canonicality rules" rule 1. |
| `ChildCountOutOfRange { count }` | Variable-arity child count outside `1..=32`. | §II.1 "Canonicality rules" rule 1. |
| `KGreaterThanN { k, n }` | `k > n` in `multi(k, …)` / `sortedmulti(k, …)` / `thresh(k, …)`. | §II.1 "Canonicality rules" rule 1. |
| `DecodeRecursionDepthExceeded { depth, max }` | `read_node` recursion exceeded `MAX_DECODE_DEPTH = 128`. Hostile/pathological tree nesting. | §II.1 "Decoder hardening"; legitimate descriptors never approach this cap. |

### TLV section

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `TlvOrderingViolation { prev, current }` | TLV tag not strictly ascending. | §II.1 "TLV section" — TLV entries must be sorted by tag. |
| `PlaceholderIndexOutOfRange { idx, n }` | TLV placeholder index ≥ n. | §II.1 "Canonicality rules" rule 2; references are `@0..=@(n-1)`. |
| `OverrideOrderViolation { prev, current }` | Per-`@N` override entries within a TLV must be in ascending `@N`-index order. | §II.1 "Per-`@N` overrides"; encoders MUST emit ascending. |
| `EmptyTlvEntry { tag }` | TLV entry has zero entries. Encoder MUST omit empty TLVs. | §II.1 "TLV section" — empty TLVs are encoder bugs, not valid wire content. |
| `TlvLengthExceedsRemaining { length, remaining }` | Declared TLV length > bits available. Truncated or hand-edited input. | §II.1 "TLV section"; verify the bundle is complete. |

### Canonicality + placeholder usage

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `PlaceholderNotReferenced { idx, n }` | Placeholder `@i` not referenced anywhere in the tree (BIP-388 well-formedness). | §II.1 "Canonicality rules" rule 2; remove the dead placeholder or wire it into a leaf. |
| `PlaceholderFirstOccurrenceOutOfOrder { expected_first, got_first }` | First-occurrence ordering violated. `@0` must appear before `@1`, `@1` before `@2`, etc., at first sighting. | §II.1 "Canonicality rules" rule 3. |
| `MultipathAltCountMismatch { expected, got }` | All multipaths in a template must share the same alt-count. | §II.1 "Use-site multipath" — mixing `<0;1>` and `<0;1;2>` in one template is forbidden. |
| `ForbiddenTapTreeLeaf { tag }` | A tap-leaf carries a fragment outside the BIP-342 admissible set. | §II.1 "Canonicality rules" rule 4. |
| `OperatorContextViolation { tag, context }` | A tag appeared in an invalid position. `context: TopLevel` rejects roots outside `{Sh, Wsh, Wpkh, Pkh, Tr}`; `context: TapLeaf` rejects a non-BIP-342 fragment inside a TapTree leaf; `context: MultiBody` is structurally unreachable in v0.30 (retained for completeness). | §II.1 "Canonicality rules" rules 2/4. |
| `NUMSSentinelConflict` | `is_nums = 0` but `key_index ≥ n` (the historical sentinel position). | §II.1 "NUMS encoding for `tr()`"; encoder must emit `is_nums = 1` for the NUMS H-point and `is_nums = 0` with `key_index < n` for any `@i` placeholder. |
| `MissingExplicitOrigin { idx }` | Non-canonical wrapper has no explicit origin for some `@N`, and no canonical origin can be inferred. | §II.1 "Non-canonical wrappers"; populate `OriginPathOverrides` or supply an explicit `path_decl`. |
| `InvalidPresenceByte { reserved_bits }` | `WalletPolicyId` canonical-record preimage's `presence_byte` has non-zero reserved bits (bits 2..7). | §II.1 "Identity"; encoders MUST zero reserved bits when building the hash preimage. |
| `InvalidXpubBytes { idx }` | `Pubkeys` TLV's 33-byte compressed pubkey field is not a valid secp256k1 point. | §II.1 "Xpub bytes"; supply a wire-valid xpub or omit the TLV. |

### Chunking

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `ChunkCountOutOfRange { count }` | Chunk count ∉ `1..=64`. | §II.1 "Chunking"; v0.30 caps the chunk set at 64. |
| `ChunkIndexOutOfRange { index, count }` | Chunk index ≥ count. | §II.1 "Chunking"; index range is `0..count`. |
| `ChunkSetIdOutOfRange { id }` | Chunk-set-id exceeds 20-bit range. Encoder bug or wire-format drift. | §II.1 "Chunking"; the id is a 20-bit value. |
| `ChunkHeaderChunkedFlagMissing` | Chunked-flag bit 0 of the first 5-bit symbol of a chunked payload is not 1. | §II.1 "Header layout"; mixed single-string and chunked inputs are rejected. |
| `ChunkCountExceedsMax { needed }` | Encoding requires more chunks than the spec maximum (64). | §II.1 "Chunking"; reduce the policy size or split into separate bundles. |
| `ChunkSetEmpty` | Decoder invoked with no strings. | §II.1 "Chunking"; supply at least one chunk. |
| `ChunkSetInconsistent` | Chunks disagree on version, chunk-set-id, or count. | §II.1 "Chunking"; verify all chunks originate from the same encode invocation. |
| `ChunkSetIncomplete { got, expected }` | Got fewer chunks than declared by the headers. | §II.1 "Chunking"; collect the missing card before decode. |
| `ChunkIndexGap { expected, got }` | Gap in chunk index sequence. | §II.1 "Chunking"; missing card between `expected` and `got`. |
| `ChunkSetIdMismatch { expected, derived }` | After bytecode-layer decode, the recomputed leading 20 bits of `Md1EncodingId` don't match the wire-carried `chunk_set_id`. Mixed chunks from different encodings, or post-encoding payload tampering. | §II.1 "Chunking"; verify all chunks in the input set originate from the same encode invocation. |

### Codex32 envelope

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `Codex32DecodeError(String)` | codex32 decode failure (HRP, alphabet, BCH). The wrapped string describes which check failed. | §I.3 "Error-detection guarantees"; check transcription accuracy and HRP. |
| `Codex32EncodeError(String)` | codex32 encode failure (BCH layer). Declared but currently unconstructed at HEAD (§V.1.7); reserved for future encoder-side BCH faults. | §I.3; reserved variant. |

### Address derivation (feature `derive`)

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `MissingPubkey { idx }` | Address derivation called on a template-only or partial-keys descriptor — no `Pubkeys` TLV for `@idx`. | §III.1 "Wallet-policy mode"; populate every `@N` with an xpub before calling `derive_address`. |
| `ChainIndexOutOfRange { chain, alt_count }` | `derive_address` called with a `chain` index outside the use-site multipath alt-count (or non-zero when no multipath is present). | §III.1 "Address derivation"; `chain` selects the multipath alt — bound it by `alt_count`. |
| `HardenedPublicDerivation` | Use-site path requires hardened derivation from an xpub. BIP-32 forbids this. | §III.1 "Network and addressing"; xpub-only restore cannot produce addresses past a hardened use-site component — recover the xpriv. |
| `AddressDerivationFailed { detail }` | miniscript-layer failure or AST→miniscript converter mismatch. The wrapped detail string describes the underlying error (`miniscript::Error`, `Tr` / `Wsh` constructor failure, arity/context mismatch). | §III.2 "Shape coverage"; check the descriptor falls within the BIP-388 admissible set. |

md-codec has no dedicated pad-bit-rejection variant (unlike mk-codec, see below). Non-zero pad bits at the bytecode-section boundary surface as `MalformedHeader` or `BitStreamTruncated` depending on how the TLV-section parser interprets them. Trailing **zero** pad bits at the TLV-section boundary are tolerated by the rollback-as-padding mechanism — see §II.1 "Canonicality rules" rule 5 and "TLV section".

## mk1 — `mk-codec::Error`

Source: `bg002h/mnemonic-key/crates/mk-codec/src/error.rs` (22 variants). Display + emission-site catalogue: §V.2.4. The enum is `#[non_exhaustive]`.

### String-layer (codex32 plumbing, HRP, chunk-header)

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

### Bytecode-layer

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `UnsupportedVersion(u8)` | The bytecode-header version field is not `0` (v0.1's only valid value), or the string-layer header carries an unknown 5-bit version. Lifted to `ToolkitError::FutureFormat` (exit 3) by the toolkit's `From` impl. | §II.2 "Bytecode header" / "String-layer header". |
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

Source: `bg002h/mnemonic-secret/crates/ms-codec/src/error.rs` (10 variants). Display + emission-site catalogue: §V.3.4. The enum is `#[non_exhaustive]`.

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `Codex32(<inner>)` | Upstream BIP-93 parse / checksum failure (delegated from `rust-codex32`). Covers bad checksum, bad character, mixed case, length out of BIP-93 brackets. | §I.3; §II.3 "Encoding-layer framing" rule 1. |
| `WrongHrp { got }` | HRP ≠ `ms`. | §II.3 "BIP-93 wire fields"; the HRP is structurally inseparable from the BCH check. |
| `ThresholdNotZero { got }` | Threshold byte ≠ `'0'`. v0.1 is single-string only. | §II.3 "BIP-93 wire fields" rule 3. |
| `ShareIndexNotSecret { got }` | Share-index byte ≠ `'s'`. BIP-93 requires `'s'` for threshold=0 (Unshared Secret form). | §II.3 "BIP-93 wire fields" rule 4. |
| `TagInvalidAlphabet { got }` | id-field bytes not in the codex32 alphabet (defensive; unreachable after BIP-93 parse). | §II.3 "Tag type". |
| `UnknownTag { got }` | id is structurally valid but not a member of `RESERVED_TAG_TABLE`. | §II.3 "`RESERVED_TAG_TABLE`". |
| `ReservedTagNotEmittedInV01 { got }` | id is `seed`, `xprv`, `mnem`, or `prvk` — reserved-not-emitted in v0.1. Lifted to `ToolkitError::FutureFormat` (exit 3) by the toolkit's `From` impl. | §II.3 "`RESERVED_TAG_TABLE`"; only `entr` is emittable in v0.1. |
| `ReservedPrefixViolation { got }` | Payload prefix byte ≠ `0x00`. | §II.3 "The `0x00` reserved-prefix byte" — v0.2 promotes the byte to a type discriminator. |
| `UnexpectedStringLength { got, allowed }` | Total string length outside `{50, 56, 62, 69, 75}`. This rule rejects every BIP-93 long-code string in v0.1. | §II.3 "Length envelope (5 valid v0.1 lengths)". |
| `PayloadLengthMismatch { tag, expected, got }` | After stripping the prefix byte, payload length ∉ `{16, 20, 24, 28, 32}` for tag `entr`. | §II.3 "`Payload::Entr` and entropy-length validation". |

## mnemonic-toolkit — `ToolkitError`

Source: `bg002h/mnemonic-toolkit/crates/mnemonic-toolkit/src/error.rs` (26 variants). Display + exit-code + `kind()` catalogue: §V.4.4. The enum is `#[non_exhaustive]`. The "Exit" column reproduces `ToolkitError::exit_code()` per SPEC §6.1.

### Generic input + sibling-codec wrappers

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `BadInput(String)` | Generic exit-1 user-input failure (phrase parse, fingerprint parse, stdin contention). Wrapped detail describes the offending input. | §V.4.4 row 1; check the CLI invocation matches the user-input surface. |
| `Bip39(bip39::Error)` | BIP-39 mnemonic parse or validate failure (bad checksum, wrong word count, out-of-wordlist token). | §IV.1 "Bundle anatomy" — phrase slot input; verify wordlist + length. |
| `Bitcoin(BitcoinErrorKind)` | bitcoin-crate wrapper: BIP-32 derivation error, xpub parse failure, or fingerprint parse failure. | §V.4.4 row 3; see the wrapped `BitcoinErrorKind` for the specific failure. |
| `MsCodec(ms_codec::Error)` | ms1 codec error passes through (exit-code dispatched per `ms_codec_exit_code`). | §V.3 + ms1 troubleshooting section above. |
| `MkCodec(mk_codec::Error)` | mk1 codec error passes through (exit-code dispatched per `mk_codec_exit_code`). | §V.2 + mk1 troubleshooting section above. |
| `MdCodec(md_codec::Error)` | md1 codec error passes through (exit-code dispatched per `md_codec_exit_code`). | §V.1 + md1 troubleshooting section above. |

### Mode dispatch + descriptor pipeline

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `ModeViolation { mode, flag, message }` | A flag is incompatible with the active bundle mode (e.g., `--passphrase` in watch-only mode, `--descriptor` in phrase-only mode). | §V.4.4 row 7; the `mode` / `flag` fields surface in the `details` JSON block. |
| `NetworkMismatch { xpub_network, expected }` | An xpub slot's BIP-32 network prefix doesn't match `--network`. | §III.3 "Network and addressing"; pass `--network testnet` (or remove conflicting xpub). |
| `DescriptorParse(String)` | Descriptor content parse failure: lex, resolve, or walk of `--descriptor`. Distinct from `ModeViolation`. | §III.1 "Descriptor → miniscript"; verify the descriptor parses with rust-miniscript first. |
| `Bip388Distinctness { i, j }` | Two `@i` / `@j` slots resolve to identical `(xpub, derivation path)` at bundle creation. BIP-388 distinct-key rule violation. | §IV.2 "Anti-collision invariants"; assign distinct origin paths or distinct xpubs. |
| `SlotInputViolation { kind, message }` | `--slot @N.<subkey>=<value>` validation violation: `conflict` / `gap` / `invalid-set` / `duplicate-subkey`. | §IV.1 "Bundle formation"; the `kind` discriminant surfaces in `details`. |

### Verify-bundle path

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `BundleMismatch { card, message }` | Verify-bundle: an engraved card (md1/mk1/ms1, optionally `mk1[N]`) doesn't match what the recomputed bundle would emit. If the engraved bundle was produced at a non-zero BIP-32 account, pass `--account <N>`. | §IV.2 "Anti-collision invariants"; the `card` field surfaces in `details`. |
| `DescriptorReparseFailed { detail }` | Verify-bundle: preserved descriptor string fails to round-trip through the rust-miniscript parser. Corrupted JSON, manual edit, or upstream library version mismatch. | §V.4.5 "JSON envelope schema"; re-create the bundle from canonical inputs. |
| `Bip388VerifyDistinctness` | Verify-bundle: bundle violates BIP-388 distinct-key rule (re-emitted from `check_key_vector_distinctness` post-binding). | §IV.2 "Anti-collision invariants"; regenerate with distinct keys. |

### Future-format dispatch

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `FutureFormat { source, detail }` | A sibling codec reported a forward-incompatible version field. Folded from `ms_codec::Error::ReservedTagNotEmittedInV01`, `mk_codec::Error::UnsupportedVersion`, and `md_codec::Error::UnsupportedVersion` by `From` impls. Exit code 3. | §I.1 "Future formats"; upgrade the toolkit, or downgrade the producing encoder. |

### v0.2 multisig configuration (reserved)

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `MultisigConfig { message }` | Threshold/cosigner-count out of range, `k > n`, etc. Exit 1 (user-input). | §IV.1 "Bundle formation" — multisig templates. |
| `CosignerSpec { cosigner_idx, message }` | `--cosigner=<xpub>:<fp>:<path>` parse error at index `cosigner_idx`. | §IV.1 "Bundle formation"; the `cosigner_idx` field surfaces in `details`. |
| `CosignersFile { message }` | `--cosigners-file <path>` JSON parse error. | §IV.1 "Bundle formation"; check JSON validity. |

### `mnemonic convert` subcommand

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `ConvertRefusal(String)` | `mnemonic convert` rejects a `(from, to)` pair as cryptographically unrecoverable, sibling-pivot, or otherwise invalid. | §V.4.4 row 19; see SPEC_convert §3 / §4 for the refusal taxonomy. |

### `mnemonic export-wallet` subcommand

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `ExportWalletSecretInput` | Secret-bearing slot (phrase / entropy / xprv / wif) supplied to `export-wallet`. Watch-only refusal. | §V.4.4 row 20; §V.4.5.9 (output shapes) — pass an xpub-only slot, or use `mnemonic bundle` for a secret-bearing artifact. |
| `ExportWalletFormatStub(&'static str)` | Per-vendor stub format. Variant retained for future per-vendor stub introductions (Sparrow + Specter were promoted to real formats in v0.8.1). | §V.4.4 row 21; §V.4.5.9 enumerates the 8 shipped vendor emitters — none currently route through this variant. |
| `ExportWalletTaprootMultisigUnsupported(&'static str)` | `tr-multi-a` / `tr-sortedmulti-a` template — taproot multisig requires picking an internal-key designation (NUMS vs key-path key); unreachable post-v0.8 NUMS lift. | §III.2 "Shape coverage"; §V.4.5.10 compatibility matrix — use a non-taproot multisig template, or supply `--taproot-internal-key <nums\|@N>` to a format that accepts taproot multisig (`bitcoin-core` / `bip388` / `sparrow` / `specter`). |
| `ExportWalletMissingFields { format, missing }` | A per-format emitter cannot synthesize a required field from the supplied slots/descriptor. Carries a `MissingField` list. Constructed by `SparrowEmitter::collect_missing` (missing `--threshold`) and `SpecterEmitter::collect_missing` (missing `--wallet-name`) at HEAD. | §V.4.5.9 (per-vendor `collect_missing` semantics) and §V.4.5.10 (format × shape matrix with per-format required flags); populate the missing flag, or pick a different export format that accepts the same shape with weaker requirements. |

### `mnemonic derive-child` subcommand

| Variant | Likely cause | Remediation pointer |
|---|---|---|
| `DeriveChildUnsupportedApp` | `--application rsa\|rsa-gpg` deferred pending `rsa`-crate stability (RUSTSEC-2023-0071 unpatched as of v0.8.0). | §V.4.4 row 24; `dice` shipped in v0.8; `rsa` family deferred. |
| `DeriveChildLengthOutOfRange { app, length, valid_text }` | `--length <N>` falls outside the per-application valid range. | §V.4.4 row 25; `valid_text` lists the acceptable range. |
| `DeriveChildLengthNotApplicable` | Non-zero `--length` supplied to an application whose output is fixed-size (`hd-seed`, `xprv`). | §V.4.4 row 26; omit `--length`. |

The `From` impls for `ms_codec::Error`, `mk_codec::Error`, and `md_codec::Error` selectively fold version-future variants (`ReservedTagNotEmittedInV01`, `UnsupportedVersion`) into `FutureFormat` so that callers see exit code 3 on any forward-incompatible card. Every other sibling-codec variant passes through wrapped as `MsCodec(_)` / `MkCodec(_)` / `MdCodec(_)` and inherits the sibling's own exit-code dispatcher.
