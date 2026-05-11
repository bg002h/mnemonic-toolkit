# Glossary

This is the technical-manual glossary, focused on **wire-format / API / cryptography** terms. End-user-facing concepts (BIP-39 / BIP-32 walkthroughs, multisig UX) live in the end-user manual's glossary at `docs/manual/src/60-appendices/61-glossary.md`.

Entries populate incrementally per cut; the tech-manual-v0.1 seed below tracks what Parts I + II introduce. Section pointers cite the section of first definitional use.

## @N

A BIP-388 wallet-policy placeholder for cosigner `N` (0-indexed). `@0` is the first cosigner; the placeholder count `n` determines `kiw`. First defined §I.4.

## BCH

Bose–Chaudhuri–Hocquenghem error-correction code. md1 and mk1 share a Bitcoin-tuned BCH polynomial *forked* from BIP-93 codex32 (HRP-mixed, per-format target residues). ms1 uses BIP-93 codex32 directly via `rust-codex32`. Primer at §I.3.

## bech32

The 32-character alphabet introduced by BIP-173 (SegWit addresses) and reused by BIP-93 codex32. Visually-disambiguated: every pair of plausibly-confusable characters maps to different GF(32) values. First defined §I.3.

## BIP-388

Wallet-policy descriptor templates. The canonical JSON shape (`name`, `description_template`, `keys_info`) exchanged between hardware wallets and coordinators. md1 encodes BIP-388 wallet policies. First cited §I.1.

## chunk_set_id (md1)

20-bit per-encoding identifier carried in every chunked-card header; derived from the leading 20 bits of `Md1EncodingId` = `SHA-256(canonical bytecode)[0..16 bytes]`. Reassembly verifies all chunks share the value; content-identity in 20 bits. Defined §II.1.

## chunk_set_id (mk1)

20-bit per-encoding identifier carried in every mk1 chunked-card header. Opaque (CSPRNG by default; deterministic-from-stub also permitted). Cross-chunk integrity is enforced separately by the 4-byte `cross_chunk_hash`. Defined §II.2.

## codex32

BIP-93 — a Bitcoin-tuned 32-character alphabet with a BCH-style checksum and human-readable prefix. Adopted directly by ms1; the BCH plumbing is forked (not shared as a crate) by md1 and mk1. Primer at §I.3.

## compact-73

mk1's 73-byte canonical xpub serialization. Strips `xpub.depth` and `xpub.child_number` from the wire (reconstructed at decode time from `origin_path`); preserves `version`, `parent_fingerprint`, `chain_code`, `public_key`. Saves 5 bytes per card vs. BIP-32 serialization. Defined §II.2.

## cross_chunk_hash

4-byte trailer = `SHA-256(canonical_bytecode)[0..4]` appended to mk1's canonical bytecode before chunk-split. Defends content-integrity across the (opaque) `chunk_set_id`. Mismatch → `Error::CrossChunkHashMismatch`. Defined §II.2.

## divergent_paths

1-bit flag in the md1 single-string header. `1` = per-`@N` divergent paths declared (one path per placeholder); `0` = one shared path applies to all placeholders. Defined §II.1.

## forked-BCH boundary

The architectural split between md1+mk1 (which fork BIP-93 BCH plumbing with HRP-mixing + per-format target residues) and ms1 (which uses BIP-93 codex32 directly via `rust-codex32`). Discussed §I.2; mechanics §I.3.

## GF(32)

The finite field with 32 elements; the codex32 alphabet symbol set. Each card character is one GF(32) symbol; BCH polynomial operations work over this field. First defined §I.3.

## HRP-mixing

The BIP-173 HRP-expansion convention used by all three formats: each HRP character contributes `c >> 5` and `c & 31` to the polymod input (with a zero separator), so the format's HRP is structurally inseparable from the checksum. Defined §I.3.

## is_nums

1-bit flag on `Body::Tr` (md1 wire format, v0.30+). When `1`, signals the BIP-341 NUMS H-point as the implicit Taproot internal key (with `key_index` field suppressed entirely on the wire). When `0`, references the placeholder at `key_index` (width `kiw = ⌈log₂(n)⌉`). Defined §II.1.

## kiw

Key-index width — `kiw = ⌈log₂(n)⌉` bits, where `n` is the policy's `@N` placeholder count. The bit width of every `key_index` field in md1's multi-family / single-key bodies. For `n = 1`, `kiw = 0` (zero-width). Defined §I.4.

## LP4-ext varint

Length-prefixed-by-4-bits-with-extension variable-width integer encoding used in md1 path-component fields. First 4 bits = length minus 1 (giving 1–16 four-bit groups); remaining bits = the index in MSB-first order. Defined §II.1.

## m-format constellation

The four sibling formats — **md1**, **mk1**, **ms1**, and the `mnemonic-toolkit` integration layer. Visually a star with the toolkit at the centre. Defined §I.1.

## md1

The descriptor card. Encodes a BIP-388-style wallet policy. HRP `md`. Library crate `md-codec`; CLI binary `md`. Repo `bg002h/descriptor-mnemonic`. Wire format documented §II.1.

## Md1EncodingId

The leading 16 bytes of `SHA-256(canonical bit-packed payload bytecode)` for an md1 encoding. Its leading 20 bits become `chunk_set_id (md1)`; the wire carries the prefix, the recomputation is deterministic from the canonical bytecode. Defined §II.1.

## miniscript

A subset of Bitcoin Script with type-checking and analysis properties (BIP-379). Each md1 descriptor body is a miniscript expression beneath the outer wrapper (`wsh()` / `tr()` / etc.). First cited §I.1.

## mk1

The key card. Encodes an xpub plus its BIP-32 origin (master fingerprint + derivation path). HRP `mk`. Library crate `mk-codec`. Repo `bg002h/mnemonic-key`. Wire format documented §II.2.

## ms1

The secret card. Encodes BIP-39 entropy (or a BIP-32 master seed). HRP `ms`. Library crate `ms-codec`; uses `rust-codex32` directly. Repo `bg002h/mnemonic-secret`. Wire format documented §II.3.

## NUMS

Nothing-Up-My-Sleeve. The BIP-341 H-point with no known discrete log, used as the Taproot internal key when a wallet has no cooperative-spend path. In md1 v0.30+ the NUMS encoding is the `is_nums = 1` flag on `Body::Tr`. First defined §II.1.

## OriginPath

The md1 wire-format encoding of a BIP-32 derivation prefix shared by all `@N` placeholders (shared mode) or per-placeholder (divergent mode). `Tag::OriginPaths = 0x36` was a TLV tag in md-codec v0.10; v0.11 retired wire-layer dictionaries and v0.30 routes paths through the bit-aligned origin-path declaration directly. Defined §II.1.

## Payload::Entr

ms1's `Payload` enum variant carrying BIP-39 entropy bytes. Accepted lengths `{16, 20, 24, 28, 32}` correspond bijectively to BIP-39 word counts `{12, 15, 18, 21, 24}`. `#[non_exhaustive]` since v0.1.0. Defined §II.3.

## PBKDF2

Password-Based Key Derivation Function 2 (HMAC-SHA512 in the BIP-39 application). Re-derives the 64-byte BIP-32 master seed from `(mnemonic + passphrase)` with 2048 iterations. The application-layer step ms1 chains into after decoding entropy. Defined §II.3.

## policy_id_stub

The top 4 bytes of `SHA-256(canonical md1 bytecode)`. Indexing aid (not a cryptographic primitive): birthday-bound collision probability among 50 stubs at 32 bits is `~2.85×10⁻⁷`. Each mk1 card carries `policy_id_stub` per linked md1 policy. Defined §II.2.

## polymod

The BCH-codeword residue function: given an input symbol stream, advance a fixed-size feedback register through a series of GF(32) multiplications. The verifier compares `polymod(hrp_expand(hrp) || data || checksum)` against the format's target residue. Defined §I.3.

## reserved-prefix byte (ms1)

A single byte at the head of every v0.1 ms1 payload (`0x00` in v0.1; rejected non-zero with `Error::ReservedPrefixViolation`). v0.2 promotes the byte to a type discriminator for share-encoding migration. Defined §II.3.

## RESERVED_TAG_TABLE

ms1's 5-entry curated table of payload-type tags (`entr` emit/accept; `seed`, `xprv`, `mnem`, `prvk` reserved-not-emitted in v0.1). Grows by SemVer-minor only. Defined §II.3.

## target residue

The format-specific GF(32) constant the BCH polymod output is compared against. codex32 uses BIP-93's value; md1's `MD_REGULAR_CONST` derives from `SHA-256("shibbolethnums")`; mk1's `MK_REGULAR_CONST` derives from `SHA-256("shibbolethnumskey")`. Per-format target residues are what *fork* md1↔mk1 from codex32 — the generator polynomial is shared. Defined §I.3.

## Tag::ENTR

ms1's `Tag` constant exposing the `entr` (BIP-39 entropy) type tag (`Tag(*b"entr")`). The only callable `Tag` in v0.1's public API. Defined §II.3.

## TLV section

The bit-aligned trailing region of md1's bytecode carrying optional metadata blocks (Fingerprints, Pubkeys, OriginPathOverrides, Unknown). TLV tags live in a **separate** 5-bit namespace from the bytecode 6-bit operator-tag space. Defined §II.1.

## walker normalization

md1 encoding convention: emit a bare `Tag::PkK` or `Tag::PkH` at a `c:`-position (instead of wrapping with an explicit `Tag::Check`). The renderer reconstructs the `c:` wrapper at key-leaf positions; saves wire bits on a wrapper that is structurally implied. Defined §II.1.

## Wallet Instance ID

`SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]`. The cryptographic identity bound at recovery time when a complete assembly's bytecode + xpubs are recomputed and compared against an externally-anchored expected value. Distinct from `policy_id_stub` (which is the 4-byte indexing aid). Defined §II.2.

## wire format

The bit-level serialisation of a backup card. md1's current wire format is v0.30 (a clean break from v0.x — see `bg002h/descriptor-mnemonic/design/SPEC_v0_30_wire_format.md`). mk1's wire format mirrors md1's BCH plumbing but has its own primary-tag space. ms1's wire format is BIP-93 codex32 directly. Documented §II.
