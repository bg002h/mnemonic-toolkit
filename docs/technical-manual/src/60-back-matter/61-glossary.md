# Glossary

This is the technical-manual glossary, focused on **wire-format / API / cryptography** terms. End-user-facing concepts (BIP-39 / BIP-32 walkthroughs, multisig UX) live in the end-user manual's glossary at `docs/manual/src/60-appendices/61-glossary.md`.

Entries have accumulated across all five Parts (v0.1 through v1.0). Section pointers cite the section of first definitional use.

## @N

A BIP-388 wallet-policy placeholder for cosigner `N` (0-indexed). `@0` is the first cosigner; the placeholder count `n` determines `kiw`. First defined §I.4.

## address derivation

The end-to-end transformation that turns an md1 template plus its key information into a network-specific bitcoin address. Entry point: `Descriptor::derive_address(chain, index, network)` at `crates/md-codec/src/derive.rs:92-132`. Three tiers — template, derivation, script + address — discussed §III.1.

## base58check

A base-58 encoding with a 4-byte SHA-256(SHA-256(·)) checksum appended; used for legacy P2PKH / P2SH addresses and BIP-32 extended-key serialization. Network-distinguished via leading version bytes (`0x00` mainnet P2PKH; `0x05` mainnet P2SH; `0x6F` / `0xC4` testnet). First cited §III.3.

## BCH

Bose–Chaudhuri–Hocquenghem error-correction code. md1 and mk1 share a Bitcoin-tuned BCH polynomial *forked* from BIP-93 codex32 (HRP-mixed, per-format target residues). ms1 uses BIP-93 codex32 directly via `rust-codex32`. Primer at §I.3.

## bech32

The 32-character alphabet introduced by BIP-173 (SegWit addresses) and reused by BIP-93 codex32. Visually-disambiguated: every pair of plausibly-confusable characters maps to different GF(32) values. First defined §I.3.

## binary-only crate

A Cargo crate exposing only a `[[bin]]` target with no `[lib]` target and no `src/lib.rs`. `mnemonic-toolkit` v0.8.0 is binary-only — external code cannot `use mnemonic_toolkit::*`; integration goes through the `mnemonic` binary's JSON envelopes (§V.4.5) instead. Library extraction is deferred to v0.9+. Defined §V.4.1.

## bip388 (format alias)

The `--format bip388` selector for `mnemonic export-wallet`, emitting a BIP-388 `wallet_policy` JSON object (vendor-neutral; consumed by hardware wallets that implement BIP-388 natively, such as Ledger). Output is a JSON object with `name`, `description_template`, `keys_info`. Accepts every template and every BIP-388-parseable `--descriptor` with multipath `/<0;1>/*`. Emitter: `wallet_export/bip388.rs` (`pub(crate)`). Defined §V.4.5.9.2 (output shape) and §V.4.5.10 (compatibility matrix).

## Bitcoin Core (wallet-export format)

The `--format bitcoin-core` selector for `mnemonic export-wallet`, emitting an `importdescriptors`-compatible JSON array. Multipath `<0;1>/*` splits into two entries (receive `internal: false`, change `internal: true`). Accepts every BIP-388-parseable shape; never refuses for shape reasons. Emitter: `wallet_export/bitcoin_core.rs` (`pub(crate)`). Defined §V.4.5.9.1 (output shape) and §V.4.5.10 (compatibility matrix).

## Blockstream Green (wallet-export format)

The `--format green` selector for `mnemonic export-wallet`, emitting a 3-line text file (two comment lines pointing at the Green help URL plus the canonical descriptor). Singlesig only; multisig is refused (Green's multisig setup is server-mediated, not file-import). Emitter: `wallet_export/green.rs` (`pub(crate)`); shipped in v0.8.1. Defined §V.4.5.9.8 (output shape) and §V.4.5.10 (compatibility matrix).

## BIP-388

Wallet-policy descriptor templates. The canonical JSON shape (`name`, `description_template`, `keys_info`) exchanged between hardware wallets and coordinators. md1 encodes BIP-388 wallet policies. First cited §I.1.

## BIP-388 distinct-key rule

The BIP-388 §"Specification" requirement that a wallet policy's key-information vector contain pairwise-distinct `(xpub, derivation_path)` tuples. Enforced symmetrically by the toolkit at bundle creation (exit 2) and verify-bundle (exit 4). Normalization domain at v0.5+ is typed `DerivationPath` equality (folds `h` ↔ `'`). Defined §IV.2.

## bundle

The toolkit's unit of engraving. Binds three sibling card formats — md1 (wallet policy), mk1 (per-cosigner xpub), ms1 (secret material) — together as one wallet's permanent backup. Synthesized by `synthesize_unified` (`crates/mnemonic-toolkit/src/synthesize.rs:593`); verified by `cmd::verify_bundle::run` (`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:98`). Defined §IV.1.

## bundle envelope

The toolkit-emergent set `{md1, mk1[0..N], ms1[0..N]}` plus the binding rules (`chunk_set_id` cross-prefix agreement, BIP-388 distinctness, multiset `md1_xpub_match`). Not a separate wire format; serialized to JSON via `BundleJson` (`crates/mnemonic-toolkit/src/format.rs:119-145`) with `schema_version = "4"`. Defined §IV.1.

## BundleJson

The serde-derived top-level struct (`crates/mnemonic-toolkit/src/format.rs:120-145`) emitted on stdout by `mnemonic bundle`. Carries `schema_version`, `mode`, `network`, `template`/`descriptor`, `account`, origin path(s), master fingerprint, the three card-set fields (`ms1`, `mk1`, `md1`), `multisig` metadata, and `privacy_preserving`. Defined §V.4.5.1.

## BundleMode

Five-variant enum classifying a bundle by slot composition: `SingleSigFull` / `SingleSigWatchOnly` / `MultisigMultiSource` / `MultisigWatchOnly` / `MultisigHybrid`. Auto-detected from `--slot` inputs by `detect_bundle_mode` (`crates/mnemonic-toolkit/src/bundle_unified.rs:34-63`). Defined §IV.1.

## chain

The multipath alternative selector argument to `Descriptor::derive_address`. For the canonical `<0;1>/*` use-site path, `chain = 0` is the receive branch and `chain = 1` is the change branch. Out-of-range or hardened values are pre-flight-rejected. Discussed §III.1.

## canonical_origin

`md-codec`'s wrapper-shape → canonical BIP-32 origin-path lookup at `canonical_origin.rs:45`. Returns `Some(OriginPath)` for the five canonical wrappers (`pkh@N`, `wpkh@N`, `tr@N` keypath, `wsh(multi|sortedmulti)`, `sh(wsh(multi|sortedmulti))`) and `None` for all other shapes (which then require explicit `@N` overrides via `validate_explicit_origin_required`). Defined §V.1.3.2.

## CKDpub

BIP-32's "public parent key → public child key" function: given an xpub (chain code + compressed pubkey) and a non-hardened child index, deterministically derive the child xpub without secret material. The primitive that turns Tier 2's xpub + chain + index into a definite secp256k1 pubkey. Discussed §III.1.

## chunk_set_id (md1)

20-bit per-encoding identifier carried in every chunked-card header; derived from the leading 20 bits of `Md1EncodingId` = `SHA-256(canonical bytecode)[0..16 bytes]`. Reassembly verifies all chunks share the value; content-identity in 20 bits. Defined §II.1.

## chunk_set_id (mk1)

20-bit per-encoding identifier carried in every mk1 chunked-card header. Opaque (CSPRNG by default; deterministic-from-stub also permitted). Cross-chunk integrity is enforced separately by the 4-byte `cross_chunk_hash`. Defined §II.2.

## chunk_set_id binding (bundle)

The cross-card bundle-level binding role of `chunk_set_id`: md1 prints 4 hex chars (16 bits = `policy_id[0..2]`) at `bundle.rs:707`; ms1/mk1 print 5 hex chars (20 bits = `derive_mk1_chunk_set_id(policy_id[0..4])`) at `bundle.rs:724`. The leading 16 bits agree across all three cards from one bundle. Discussed §IV.2.

## codex32

BIP-93 — a Bitcoin-tuned 32-character alphabet with a BCH-style checksum and human-readable prefix. Adopted directly by ms1; the BCH plumbing is forked (not shared as a crate) by md1 and mk1. Primer at §I.3.

## compact-73

mk1's 73-byte canonical xpub serialization. Strips `xpub.depth` and `xpub.child_number` from the wire (reconstructed at decode time from `origin_path`); preserves `version`, `parent_fingerprint`, `chain_code`, `public_key`. Saves 5 bytes per card vs. BIP-32 serialization. Defined §II.2.

## Coldcard (wallet-export format)

The `--format coldcard` selector for `mnemonic export-wallet`. Singlesig templates (`bip44` / `bip49` / `bip84`) emit Coldcard's generic-wallet-export JSON; multisig templates (`wsh` / `sh-wsh` family) emit a 4+N-line text shape byte-identical to the Coldcard multisig text Jade also ingests. Refuses `bip86` (Coldcard's schema documents only `bip44`/`bip49`/`bip84`) and `tr-multi-a` / `tr-sortedmulti-a` (pending firmware). Master-xpub is plumbed via `--slot @0.master_xpub=<base58>` (singlesig top-level `xpub` field). Emitter: `wallet_export/coldcard.rs` (`pub(crate)`); shipped in v0.8.1, master-xpub wiring landed in v0.8.1. Defined §V.4.5.9.3 (output shape) and §V.4.5.10 (compatibility matrix).

## compute_wallet_policy_id

`md-codec` identity function at `identity.rs:172` that computes a `WalletPolicyId` from a `Descriptor`. Self-canonicalises and self-runs `expand_per_at_n`; renders SHA-256 over canonical template ‖ per-`@N` records (truncated to 16 bytes). Sibling functions `compute_md1_encoding_id` (`identity.rs:39`) and `compute_wallet_descriptor_template_id` (`identity.rs:71`) compute the encoding-id and γ-flavor template-id. Defined §V.1.3.11.

## CosignerEntry

The per-cosigner row struct in `BundleJson.multisig.cosigners` (`crates/mnemonic-toolkit/src/format.rs:94-100`). Carries `index`, `master_fingerprint`, `origin_path`, `xpub`. Defined §V.4.5.5.

## cosigner-mapping diagnostic

The three-mode failure-classification used by `verify-bundle` to attribute an unmappable `--mk1` group: `NotSupplied` (no card for the slot), `DecodeFailed(msg)` (group exists but `mk_codec::decode` rejects it), `XpubNotInPolicy` (decoded successfully but xpub absent from the descriptor's pubkeys-TLV — wrong-key-attack indicator). Precedence: `XpubNotInPolicy > DecodeFailed > NotSupplied` (`verify_bundle.rs:831-836`, two-pass at `:895-947`). Defined §IV.2.

## cross_chunk_hash

4-byte trailer = `SHA-256(canonical_bytecode)[0..4]` appended to mk1's canonical bytecode before chunk-split. Defends content-integrity across the (opaque) `chunk_set_id`. Mismatch → `Error::CrossChunkHashMismatch`. Defined §II.2.

## definite key

A `DescriptorPublicKey` after multipath alt selection and wildcard `/*` resolution: the underlying xpub has been derived along the use-site path with a specific `(chain, index)` and reduced to a single secp256k1 point. The input rust-miniscript wants for `address()` rendering. Discussed §III.1.

## derive (Cargo feature)

`md-codec`'s sole feature flag (`crates/md-codec/Cargo.toml:21-23`, default enabled). Gates `pub mod to_miniscript` and `Descriptor::derive_address`. `pub mod derive` exists unconditionally; the body wraps every item in `#[cfg(feature = "derive")]` — so with the feature off, `md_codec::derive` is an empty-public-API module while `md_codec::to_miniscript` does not exist. Defined §V.1.2.

## DescriptorPublicKey

rust-miniscript's key type used by `miniscript::Descriptor`. The converter at `to_miniscript.rs:84-89` builds `DescriptorPublicKey::XPub { origin, xkey, derivation_path, wildcard: Unhardened }` for each `@N`; the NUMS-internal-key path builds `DescriptorPublicKey::Single { origin: None, key: SinglePubKey::XOnly(H) }`. Discussed §III.1, §III.2.

## divergent_paths

1-bit flag in the md1 single-string header. `1` = per-`@N` divergent paths declared (one path per placeholder); `0` = one shared path applies to all placeholders. Defined §II.1.

## Electrum (wallet-export format)

The `--format electrum` selector for `mnemonic export-wallet`, emitting an Electrum wallet-file JSON. Singlesig wraps a `keystore` object with `xpub` (SLIP-132 lowercase variant per script type) + `derivation` + `root_fingerprint` + `label`; multisig wraps `x1/`, `x2/`, ... cosigner objects with SLIP-132 uppercase variants (`Ypub` / `Zpub`). `seed_version` is pinned to `ELECTRUM_SEED_VERSION_PIN = 17`. Refuses `tr-multi-a` / `tr-sortedmulti-a` (pending libsecp-taproot upstream support). Emitter: `wallet_export/electrum.rs` (`pub(crate)`); shipped in v0.8.1. Defined §V.4.5.9.7 (output shape) and §V.4.5.10 (compatibility matrix).

## ELECTRUM_SEED_VERSION_PIN

The `pub const u32 = 17` at `wallet_export/electrum.rs:37` pinning the `seed_version` field emitted into every Electrum wallet-export file. Empirically validated 2026-05-12 against Electrum 4.5.5 (loader walks the `_convert_version_<N>` migration chain forward to `FINAL_SEED_VERSION` on first save). Defined §V.4.5.9.7.

## EmitInputs

The single struct (`crates/mnemonic-toolkit/src/wallet_export/mod.rs:327-369`, `pub(crate)`) threaded through every `WalletFormatEmitter::emit` call. Carries the canonical descriptor (with `#checksum`), resolved slots, template (or `None` for descriptor-passthrough), `WalletScriptType`, network, account, threshold (and a `threshold_user_supplied` flag), `master_xpub_at_0`, wallet name (and a `wallet_name_was_user_supplied` flag), taproot internal-key designation, range, timestamp, and Bitcoin Core target version. Constructed in `cmd::export_wallet::run` after watch-only validation. Defined §V.4.5.9.

## engraving card

A stderr-only emission from `mnemonic bundle` carrying a fixed-shape per-card identifier index (`# ms1: ...`, `# mk1: ...`, `# md1: ...`) plus template/threshold/cosigners metadata. Produced by `engraving_card_unified` (`crates/mnemonic-toolkit/src/format.rs:259-376`) from a `BundleInputForCard`. Not machine-readable; designed for physical alignment when stamping plates. Defined §IV.1.

## expand_per_at_n

`md-codec`'s per-`@N` resolution helper at `canonicalize.rs:420`. Overlays per-`@N` TLV overrides on descriptor-level baselines to produce a `Vec<ExpandedKey>` (one per placeholder, each carrying `idx`, `origin_path`, `use_site_path`, optional `fingerprint`, optional `xpub`). Precondition: caller has canonicalised indices (or the descriptor came from a decoder, which is canonical by construction). Defined §V.1.3.3.

## forked-BCH boundary

The architectural split between md1+mk1 (which fork BIP-93 BCH plumbing with HRP-mixing + per-format target residues) and ms1 (which uses BIP-93 codex32 directly via `rust-codex32`). Discussed §I.2; mechanics §I.3.

## fingerprint (master)

The 4-byte HASH160-prefix identifier of an xpub's master key (BIP-32 §"Key identifiers"). Carried by md1 in the `Fingerprints` TLV (`0x01`) when an `@N` has an associated master-fingerprint annotation. Used by signing flows (PSBT key-source metadata); not consulted by address derivation. Discussed §III.1.

## GF(32)

The finite field with 32 elements; the codex32 alphabet symbol set. Each card character is one GF(32) symbol; BCH polynomial operations work over this field. First defined §I.3.

## HRP-mixing

The BIP-173 HRP-expansion convention used by all three formats: each HRP character contributes `c >> 5` and `c & 31` to the polymod input (with a zero separator), so the format's HRP is structurally inseparable from the checksum. Defined §I.3.

## H-point (NUMS)

The BIP-341 nothing-up-my-sleeve internal-key x-only coordinate `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` — the SHA-256 of an agreed-upon generator point's compressed encoding, with no known discrete log. Used as the taproot internal key when the wallet has no key-path-spend mode. Pinned at `to_miniscript.rs:34-35`. Discussed §III.2.

## git-tag pin

The dependency-pinning pattern used by `mnemonic-toolkit` at `crates/mnemonic-toolkit/Cargo.toml:20-22`: each sibling codec is pinned to an exact git tag (`ms-codec-v0.1.0`, `mk-codec-v0.2.1`, `md-codec-v0.16.1`) against its source repo, not to a crates.io version. Pre-crates.io-publish state; downstream consumers wanting a stable contract today target the CLI binary + JSON envelopes instead. Defined §V.4.8.

## is_nums

1-bit flag on `Body::Tr` (md1 wire format, v0.30+). When `1`, signals the BIP-341 NUMS H-point as the implicit Taproot internal key (with `key_index` field suppressed entirely on the wire). When `0`, references the placeholder at `key_index` (width `kiw = ⌈log₂(n)⌉`). Defined §II.1.

## Jade (wallet-export format)

The `--format jade` selector for `mnemonic export-wallet`. Multisig templates (`wsh` / `sh-wsh` family) delegate byte-identical to Coldcard's multisig text (§V.4.5.9.3) — Jade's `register_multisig.multisig_file` ingests the same shape. Refuses every singlesig template (Jade reads the seed on-device for singlesig) and `tr-multi-a` / `tr-sortedmulti-a` (pending firmware). Emitter: `wallet_export/jade.rs` (`pub(crate)`); shipped in v0.8.1. Defined §V.4.5.9.4 (output shape) and §V.4.5.10 (compatibility matrix).

## JSON envelope (toolkit)

The serde-serialised top-level stdout output of `mnemonic bundle` (`BundleJson`, §V.4.5.1) or `mnemonic verify-bundle` (`VerifyBundleJson`, §V.4.5.2). External consumers integrate with the toolkit through this envelope rather than through a Rust library; field names, order, and `Option` semantics are determined by the serde-derived structs at HEAD. Pinned via the `schema_version` field. Defined §V.4.5.

## KeyCard

`mk-codec`'s top-level decoded-card struct at `key_card.rs:24` (`#[non_exhaustive] #[derive(Debug, Clone, PartialEq, Eq)]`). Carries `policy_id_stubs: Vec<[u8; 4]>`, `origin_fingerprint: Option<Fingerprint>`, `origin_path: DerivationPath`, `xpub: Xpub`. `KeyCard::new` is intentionally permissive — field-level invariants are enforced at encode time, not construction. Defined §V.2.3.3.

## kiw

Key-index width — `kiw = ⌈log₂(n)⌉` bits, where `n` is the policy's `@N` placeholder count. The bit width of every `key_index` field in md1's multi-family / single-key bodies. For `n = 1`, `kiw = 0` (zero-width). Defined §I.4.

## LP4-ext varint

Length-prefixed-by-4-bits-with-extension variable-width integer encoding used in md1 path-component fields. First 4 bits = length minus 1 (giving 1–16 four-bit groups); remaining bits = the index in MSB-first order. Defined §II.1.

## key_index (md1)

The kiw-bit (per-policy `n`) field identifying which `@N` placeholder a key-reference operator refers to. Carried inline for `wpkh` / `pkh` / `tr` / `pk_k` / `pk_h` bodies. Suppressed entirely on the wire for `Body::Tr` when `is_nums = 1`. Defined §II.1.

## m-format constellation

The four sibling formats — **md1**, **mk1**, **ms1**, and the `mnemonic-toolkit` integration layer. Visually a star with the toolkit at the centre. Defined §I.1.

## md1

The descriptor card. Encodes a BIP-388-style wallet policy. HRP `md`. Library crate `md-codec`; CLI binary `md`. Repo `bg002h/descriptor-mnemonic`. Wire format documented §II.1.

## md1_xpub_match

The `verify-bundle` check that the multiset of supplied-md1 `Tag::Pubkeys = 0x02` TLV values equals the multiset of expected-md1 pubkeys. Sort-then-compare on `Vec<[u8; 65]>` preserves multiplicity, so `wsh(multi(K,@0,@0))` doesn't compare equal to `wsh(multi(K,@0,@1))` (multisig path; single-sig uses `.first()` comparison in `emit_md1_checks` at `verify_bundle.rs:1280-1355` with detail `"65-byte xpub matches expected"`). Multisig implementation at `verify_bundle.rs:1194-1232`. Defined §IV.2.

## Md1EncodingId

The leading 16 bytes of `SHA-256(canonical bit-packed payload bytecode)` for an md1 encoding. Its leading 20 bits become `chunk_set_id (md1)`; the wire carries the prefix, the recomputation is deterministic from the canonical bytecode. Defined §II.1.

## miniscript

A subset of Bitcoin Script with type-checking and analysis properties (BIP-379). Each md1 descriptor body is a miniscript expression beneath the outer wrapper (`wsh()` / `tr()` / etc.). First cited §I.1.

## MissingField

The `pub(crate) enum` at `crates/mnemonic-toolkit/src/wallet_export/mod.rs:224-232` enumerating the seven SPEC §4 missing-info classes per-format emitters can surface via `WalletFormatEmitter::collect_missing`. Per-slot variants (`MasterFingerprint { slot }`, `DerivationPath { slot }`, `Xpub { slot }`) and globals (`ScriptType`, `Threshold`, `WalletName`, `IncompatibleFormatForTemplate`); deterministic sort key sorts globals first by enum discriminant 4 → 7, then per-slot grouped by discriminant 1 / 2 / 3 and ordered by slot index. Bullet construction is sole-sited at `build_missing_fields_refusal` (`wallet_export/mod.rs:279`). Defined §V.4.5.9.

## mk1

The key card. Encodes an xpub plus its BIP-32 origin (master fingerprint + derivation path). HRP `mk`. Library crate `mk-codec`. Repo `bg002h/mnemonic-key`. Wire format documented §II.2.

## ms1

The secret card. Encodes BIP-39 entropy (or a BIP-32 master seed). HRP `ms`. Library crate `ms-codec`; uses `rust-codex32` directly. Repo `bg002h/mnemonic-secret`. Wire format documented §II.3.

## multipath

BIP-389's `<alt_0;alt_1;...;alt_n>` syntax for a use-site path. md1 encodes the alternatives inline in the use-site-path block; the `chain` parameter to `derive_address` selects which alternative resolves the leaf address. Defined §II.1; semantic role discussed §III.1.

## MultisigInfo

The multisig-metadata struct in `BundleJson.multisig` (`crates/mnemonic-toolkit/src/format.rs:104-111`). Carries `template`, `threshold`, `cosigner_count`, `path_family` (`"bip48"` or `"bip87"`), and the per-cosigner `Vec<CosignerEntry>`. `None` for single-sig bundles (serialised as JSON `null`). Defined §V.4.5.4.

## multiset

A set with multiplicity — `{a, a, b}` differs from `{a, b}`. In the technical manual, the relevant case is `md1_xpub_match`, where the comparison must preserve multiplicity so degenerate templates (e.g., `wsh(multi(K,@0,@0))`) don't compare equal to non-degenerate ones. The toolkit implements multiset equality as sort-then-compare on `Vec<[u8; 65]>`. Discussed §IV.2.

## NUMS

Nothing-Up-My-Sleeve. The BIP-341 H-point with no known discrete log, used as the Taproot internal key when a wallet has no cooperative-spend path. In md1 v0.30+ the NUMS encoding is the `is_nums = 1` flag on `Body::Tr`. First defined §II.1.

## non_exhaustive

The `#[non_exhaustive]` Rust attribute applied to most error enums and public structs across the m-format-star (`mk_codec::Error`, `ms_codec::Error`, `ToolkitError`, `KeyCard`, `Payload`, `InspectReport`, `VerifyCheck`'s sibling structs). Forces external `match` arms to include `_ => ...` and prohibits brace-init from outside the crate, preserving forward-compatibility on minor version bumps. **`md_codec::Error` is the exception**: it is NOT `#[non_exhaustive]` (derives `Debug, Error, PartialEq, Eq` only; external `match` over its 43 variants is exhaustive — adding a new variant upstream is a compiler-detected breaking change). Discussed §V.1.3.9, §V.2.7, §V.3.7, §V.4.8.

## origin path

The BIP-32 derivation path from a master seed to an xpub (e.g., `m/84'/0'/0'` for a BIP-84 account-0 xpub). md1 carries the origin path in the inline path-decl block (Shared or Divergent by header bit 4) plus per-`@N` overrides in TLV `0x03`. **Not** consulted by address derivation; metadata for signing flows. Discussed §III.1.

## OriginPath

The md1 wire-format encoding of a BIP-32 derivation prefix shared by all `@N` placeholders (shared mode) or per-placeholder (divergent mode). `Tag::OriginPaths = 0x36` was a TLV tag in md-codec v0.10; v0.11 retired wire-layer dictionaries and v0.30 routes paths through the bit-aligned origin-path declaration directly. Defined §II.1.

## Payload::Entr

ms1's `Payload` enum variant carrying BIP-39 entropy bytes. Accepted lengths `{16, 20, 24, 28, 32}` correspond bijectively to BIP-39 word counts `{12, 15, 18, 21, 24}`. `#[non_exhaustive]` since v0.1.0. Defined §II.3.

## PBKDF2

Password-Based Key Derivation Function 2 (HMAC-SHA512 in the BIP-39 application). Re-derives the 64-byte BIP-32 master seed from `(mnemonic + passphrase)` with 2048 iterations. The application-layer step ms1 chains into after decoding entropy. Defined §II.3.

## PathDecl

The md1 wire-format data structure for the inline origin-path declaration: `{ n: u8 (1..=32), paths: PathDeclPaths }`. The `paths` arm is `Shared(OriginPath)` (single path applies to all `@N`) or `Divergent(Vec<OriginPath>)` (one path per `@N`). Header bit 4 selects the arm. Defined `origin_path.rs:82-96`.

## PathDeclPaths

The two-arm enum nested inside `PathDecl.paths` (`md-codec` `origin_path.rs:90-96`). `Shared(OriginPath)` carries one path that applies to every `@N`; `Divergent(Vec<OriginPath>)` carries one path per `@N`. Header bit 4 (`divergent_paths`) selects which arm is on the wire. Defined §V.1.3.12.

## Payload (ms-codec)

`ms-codec`'s payload enum at `payload.rs:19` (`#[derive(Debug, Clone, PartialEq, Eq)] #[non_exhaustive]`). The single v0.1 variant is `Entr(Vec<u8>)` carrying BIP-39 entropy bytes; future v0.2+ variants (Mnem, Seed, Xprv) are foreseen. Sibling `PayloadKind` discriminates without payload. Methods: `validate`, `kind`, `as_bytes`. Defined §V.3.3.6.

## placeholder

The `@N` token in a BIP-388 wallet-policy template that stands in for a concrete cosigner xpub. md1 carries the template (with placeholders) on-card; the key information (concrete xpubs filling the placeholders) is supplied either inline (`Pubkeys` TLV `0x02`) or out-of-band (mk1 sibling cards, `md address --key`). Discussed §III.1.

## Phrase

`md-codec`'s 12-word BIP-39 phrase wrapper at `phrase.rs:7-10` (`pub struct Phrase(pub [String; 12])`). Produced from a 128-bit id via `Phrase::from_id_bytes` (effectively infallible on 128-bit entropy; the `Result` shape is API-uniform); `Display` joins the 12 words with single spaces. Defined §V.1.3.13.

## policy_id_stub

The top 4 bytes of `SHA-256(canonical md1 bytecode)`. Indexing aid (not a cryptographic primitive): birthday-bound collision probability among 50 stubs at 32 bits is `~2.85×10⁻⁷`. Each mk1 card carries `policy_id_stub` per linked md1 policy. Defined §II.2.

## polymod

The BCH-codeword residue function: given an input symbol stream, advance a fixed-size feedback register through a series of GF(32) multiplications. The verifier compares `polymod(hrp_expand(hrp) || data || checksum)` against the format's target residue. Defined §I.3.

## reserved-prefix byte (ms1)

A single byte at the head of every v0.1 ms1 payload (`0x00` in v0.1; rejected non-zero with `Error::ReservedPrefixViolation`). v0.2 promotes the byte to a type discriminator for share-encoding migration. Defined §II.3.

## Pubkeys TLV

md1's TLV `0x02`: an optional, inline carrier for the cosigner xpub bytes (32-byte chain code + 33-byte compressed pubkey per `@N`, repeated for each populated placeholder). When present, address derivation can resolve `@N` → xpub locally; when absent, xpubs must be supplied externally. Defined §II.1.

## reconstruct_xpub

`mk-codec`'s 73-byte-compact-form → full `bitcoin::bip32::Xpub` reconstructor at `bytecode/xpub_compact.rs:85`. Recovers `depth` from `len(origin_path)` and `child_number` from the path's last component. **Panics** on empty `origin_path` — external callers using `XpubCompact` directly (outside `decode`) must pre-check non-emptiness. Defined §V.2.3.9.

## RESERVED_TAG_TABLE

ms1's 5-entry curated table of payload-type tags (`entr` emit/accept; `seed`, `xprv`, `mnem`, `prvk` reserved-not-emitted in v0.1). Grows by SemVer-minor only. Defined §II.3.

## rust-codex32

Andrew Poelstra's `=0.1.0` (CC0) Rust implementation of BIP-93 codex32. `ms-codec` adopts it directly via a single crate-private `mod envelope` (`ms-codec/src/envelope.rs`) — every contact with `codex32::*` lives in that module; nothing else in `ms-codec` imports it. md1↔mk1 do not use this crate (their HRP-mixed BCH is forked, not upstreamable). Defined §V.3.1.

## target residue

The format-specific GF(32) constant the BCH polymod output is compared against. codex32 uses BIP-93's value; md1's `MD_REGULAR_CONST` derives from `SHA-256("shibbolethnums")`; mk1's `MK_REGULAR_CONST` derives from `SHA-256("shibbolethnumskey")`. Per-format target residues are what *fork* md1↔mk1 from codex32 — the generator polynomial is shared. Defined §I.3.

## schema_version

The `&'static str` field on `BundleJson` / `VerifyBundleJson` (`crates/mnemonic-toolkit/src/format.rs:120, 149`) carrying the JSON-envelope schema generation. Literal `"4"` at every construction site at HEAD v0.8.0. Exists precisely so external consumers can pin against a stable contract independent of the crate version. The `format.rs:114` module-level doc-comment ("v0.2: schema_version 2") is stale and persists at HEAD. Defined §V.4.5.

## script context (rust-miniscript)

rust-miniscript's type-class abstraction over the three valid contexts a miniscript expression can inhabit: `Legacy` (P2SH), `Segwitv0` (P2WSH), `Tap` (taproot script tree). Each context constrains which `Terminal` variants are admissible and the resource limits (key count, opcode count). md1's converter selects the context per shape and routes through `node_to_miniscript::<Ctx>` accordingly. Discussed §III.2.

## secret-bearing slot

A bundle slot whose subkey set contains any of `phrase` / `entropy` / `xprv` / `wif` — the four secret-material subkey types. Discriminator: `SlotSubkey::is_secret_bearing` at `crates/mnemonic-toolkit/src/slot_input.rs:47-49`. Bundle synthesis emits a non-empty `ms1` card for each secret-bearing slot. Defined §IV.1.

## share-set grouping

The v0.2-shares ms1 read-side invariant: ms1 readers reassembling K-of-N shares must dispatch by the reserved-prefix byte before treating BIP-93's `id` field as a share-set group key. Prefix `0x00` → v0.1 single-string secret (never groups); prefix `0x01` → v0.2 entr share (groups by `id`); prefix `≥0x02` → kind-specific path required. Defined §IV.3 (forward-looking).

## SLIP-0132

Alternative BIP-32 extended-key version bytes (`zpub`/`zprv`, `ypub`/`yprv`, `Zpub`/`Yprv`, etc.) that hint at the intended descriptor shape. **Purely cosmetic** — the chain code and pubkey bytes are unchanged; only the leading 4 version bytes differ. md1's `--key @N=...` accepts only the canonical `xpub`/`tpub` family; SLIP-0132 prefixes are normalized via `mnemonic convert`. Discussed §III.3.

## Sparrow (wallet-export format)

The `--format sparrow` selector for `mnemonic export-wallet`, emitting a Sparrow wallet-import JSON (`name`, `network`, `policyType`, `scriptType`, `defaultPolicy`, `keystores`). `policyType` is `"SINGLE"` / `"MULTI"`; `defaultPolicy.miniscript.script` carries the placeholder miniscript (e.g., `wsh(sortedmulti(2,@0/**,@1/**,@2/**))`) — for `tr-multi-a` / `tr-sortedmulti-a`, the canonical descriptor with `#checksum` stripped (Sparrow's policy parser substring-matches on `script`). Requires `--threshold` for multisig templates (refusal: `ExportWalletMissingFields { format: "sparrow", missing: [Threshold] }`). Emitter: `wallet_export/sparrow.rs` (`pub(crate)`); promoted from a stub to a real emitter in v0.8.1. Defined §V.4.5.9.5 (output shape) and §V.4.5.10 (compatibility matrix).

## Specter (wallet-export format)

The `--format specter` selector for `mnemonic export-wallet`, emitting a Specter Desktop wallet-import JSON (`label`, `blockheight`, `descriptor`, `devices`). `descriptor` is the canonical BIP-380 form WITH `#checksum` suffix. Requires `--wallet-name <STRING>` (refusal: `ExportWalletMissingFields { format: "specter", missing: [WalletName] }`). Emitter: `wallet_export/specter.rs` (`pub(crate)`); promoted from a stub to a real emitter in v0.8.1. Defined §V.4.5.9.6 (output shape) and §V.4.5.10 (compatibility matrix).

## Tag::ENTR

ms1's `Tag` constant exposing the `entr` (BIP-39 entropy) type tag (`Tag(*b"entr")`). The only callable `Tag` in v0.1's public API. Defined §II.3.

## Tag (md-codec)

`md-codec`'s 36-variant operator-tag enum (`tag.rs:14-89`). Occupies the 6-bit primary space `0x00..=0x23`; primary `0x24..=0x3E` are reserved; primary `0x3F` is the extension prefix with its 4-bit subspace `0x00..=0x0F` fully reserved. Reading reserved/extension tags yields `Error::TagOutOfRange { primary }`. The authoritative tag-code table is reproduced in §II.1 §"Tag table (v0.30)". Defined §V.1.3.14.

## TapTree

The tap-script-tree structure of a taproot output (BIP-341): a hierarchical merkle tree of leaves, each a miniscript fragment, that supplies script-path-spend alternatives. md1's `Tag::TapTree` is the wire encoding of an internal-node branching point (always 2 children). Bare-leaf `tr(@0, <leaf>)` shapes skip the `Tag::TapTree` wrap via the v0.30 single-leaf wire optimization. Discussed §III.2.

## tap-leaf miniscript

A miniscript fragment embedded as a leaf in a TapTree. Type-checked under the `Tap` script context. Most rust-miniscript miniscript fragments are admissible as tap-leaves; `multi` is rejected (must be `multi_a` under `Tap`). Discussed §III.2.

## TaprootInternalKey

`mnemonic-toolkit`'s `pub enum` at `wallet_export/mod.rs:68` discriminating the taproot internal-key designation supplied to `export-wallet` for `tr-*` templates: `Nums` (BIP-341 NUMS H-point) or a placeholder key. Used by every taproot-capable vendor emitter. Defined §V.4.5.9.

## template (md1)

The BIP-388 wallet policy expression engraved on an md1 card — a typed AST plus the use-site path (multipath alternatives + wildcard). Carries the *shape* of the wallet (descriptor type, key threshold, miniscript fragments); does not carry the *keys* (those are placeholders `@N` filled at derivation time). Discussed §III.1.

## TLV section

The bit-aligned trailing region of md1's bytecode carrying optional metadata blocks (Fingerprints, Pubkeys, OriginPathOverrides, Unknown). TLV tags live in a **separate** 5-bit namespace from the bytecode 6-bit operator-tag space. Defined §II.1.

## TimestampArg

The `pub(crate) enum` at `crates/mnemonic-toolkit/src/wallet_export/mod.rs:121-125` for the Bitcoin Core `timestamp` argument: `Now` (renders to JSON `"now"`) or `Unix(i64)` (renders to a JSON integer). Selected via the `--timestamp <now|unix>` flag (`cmd/export_wallet.rs:87`, default `now`). Consumed by `--format bitcoin-core`; ignored by every other format. Defined §V.4.5.9.

## to_miniscript_descriptor

`md-codec`'s feature-gated AST → rust-miniscript converter at `to_miniscript.rs:54`. Builds a `miniscript::Descriptor<DescriptorPublicKey>` for the given `chain` (multipath alt). Trailing `/*` wildcard is left for the caller's `at_derivation_index(index)` to resolve. v0.32 replaced the v0.14-era 5-shape allow-list with this general AST-driven converter, covering every BIP-388-parseable shape. Defined §V.1.3.16.

## ToolkitError

`mnemonic-toolkit`'s central error enum at `error.rs:10` (`#[non_exhaustive]`; 26 variants at HEAD v0.8.0). Carries `kind()` (stable JSON discriminant), `exit_code()` (SPEC §6.1 mapping), `message()` (friendly), and `details()` (JSON forensic detail). Sibling-codec errors lift in via dedicated `From` impls that selectively fold version-future variants into `FutureFormat` (exit 3). Defined §V.4.4.

## use-site path

The BIP-389 multipath + BIP-32 wildcard segment applied at the descriptor placeholder position (e.g., `<0;1>/*`). Encoded inline in md1's use-site-path block; sparse per-`@N` overrides via TLV `0x00`. Consulted by address derivation: `chain` selects the multipath alt, `index` selects the wildcard child. Discussed §III.1.

## v0.1 → v0.2-shares migration contract

The four ms-codec invariants locked at v0.1 to ensure v0.2 K-of-N share encoding can ship additively without re-engraving v0.1 cards: (1) reserved-prefix byte `0x00` → type discriminator in v0.2, (2) prefix-byte gating of BIP-93 `id`-based share grouping, (3) v0.2 encoder anti-collision against v0.1's `RESERVED_TAG_TABLE`, (4) API back-compat — `encode_shares(tag, Threshold::ZERO, &[p])` wire-bit-identical to v0.1 `encode(tag, &p)`. Authority: `mnemonic-secret/design/SPEC_ms_v0_1.md:212-226`. Discussed §IV.3.

## verify-bundle

The toolkit subcommand that re-derives each card from the user's slot inputs, compares against supplied `--ms1` / `--mk1` / `--md1` (or `--bundle-json`), and emits per-check `VerifyCheck` rows. Entry `cmd::verify_bundle::run` at `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:98-201`. Defined §IV.1.

## VerifyCheck

The per-check row struct in verify-bundle's output (`crates/mnemonic-toolkit/src/format.rs:165-183`). Carries `name`, `passed: bool`, `detail`, and conditional forensic fields (`expected`, `actual`, `diff_byte_offset`, `decode_error`) populated only on failure. Defined §IV.1.

## walker normalisation

md1 encoding convention: emit a bare `Tag::PkK` or `Tag::PkH` at a `c:`-position (instead of wrapping with an explicit `Tag::Check`). The renderer reconstructs the `c:` wrapper at key-leaf positions; saves wire bits on a wrapper that is structurally implied. Defined §II.1.

## wallet-export

The `mnemonic export-wallet` subcommand and its underlying `wallet_export` orchestration module. Emits a vendor-specific watch-only wallet-import artifact (Bitcoin Core / BIP-388 / Coldcard / Electrum / Green / Jade / Sparrow / Specter) from xpub-only slot inputs and a template or descriptor. Refuses secret-bearing inputs (`ExportWalletSecretInput`, exit 2). Defined §V.4.5.9.

## WalletFormatEmitter

The `pub(crate) trait` at `crates/mnemonic-toolkit/src/wallet_export/mod.rs:316-320` every `--format` emitter implements. Three methods: `collect_missing(&EmitInputs) -> Vec<MissingField>` (SPEC §4 deterministic-refusal predicate), `emit(&EmitInputs) -> Result<String, ToolkitError>` (byte-exact output string), `extension() -> &'static str` (file-extension hint for `--output`). Eight impls ship at HEAD: `BitcoinCoreEmitter`, `Bip388Emitter`, `ColdcardEmitter`, `JadeEmitter`, `SparrowEmitter`, `SpecterEmitter`, `ElectrumEmitter`, `GreenEmitter`. `cmd::export_wallet::run` dispatches over `CliExportFormat` to the eight impls. Defined §V.4.5.9.

## WalletPolicyId

`md-codec`'s 128-bit canonical wallet-policy identifier (`identity.rs:114-115`). Computed via `compute_wallet_policy_id`; can be rendered as a 12-word BIP-39 `Phrase` via `WalletPolicyId::to_phrase`. Distinct from `Md1EncodingId` (hashes canonical bytecode) and `WalletDescriptorTemplateId` (hashes template content only, γ-flavor). Defined §V.1.3.11.

## Wallet Instance ID

`SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]`. The cryptographic identity bound at recovery time when a complete assembly's bytecode + xpubs are recomputed and compared against an externally-anchored expected value. Distinct from `policy_id_stub` (which is the 4-byte indexing aid). Defined §II.2.

## WalletScriptType

The `pub(crate) enum` at `crates/mnemonic-toolkit/src/wallet_export/mod.rs:143-152` enumerating the eight script-type classifications wallet-export emitters dispatch on: `P2pkh` / `P2shP2wpkh` / `P2wpkh` / `P2tr` (singlesig) and `P2shMulti` / `P2shP2wshMulti` / `P2wshMulti` / `P2trMulti` (multisig). Derived from `CliTemplate` via `script_type_from_template` (`wallet_export/mod.rs:156`) on the template path, or from the parsed `miniscript::Descriptor` via `script_type_from_descriptor` (`wallet_export/mod.rs:176`) on the descriptor-passthrough path. Defined §V.4.5.9.

## watch-only slot

A bundle slot whose subkey set contains only `xpub` / `fingerprint` / `path` — no secret material. Discriminator: `SlotSubkey::is_watch_only` at `crates/mnemonic-toolkit/src/slot_input.rs:50-52`. Bundle synthesis emits the `""` empty-string sentinel into `ms1[i]` for each watch-only slot per the SPEC §5.8 dense-MsField layout. Defined §IV.1.

## wildcard (BIP-389)

The trailing `/*` (or `/*'` for hardened) in a use-site path that resolves to a child index at derivation time. md1 carries the wildcard hardenedness as a 1-bit field after the multipath block; hardened wildcards are pre-flight-rejected by `derive_address` (BIP-32 forbids hardened public derivation). Discussed §III.1.

## wire format

The bit-level serialisation of a backup card. md1's current wire format is v0.30 (a clean break from v0.x — see `bg002h/descriptor-mnemonic/design/SPEC_v0_30_wire_format.md`). mk1's wire format mirrors md1's BCH plumbing but has its own primary-tag space. ms1's wire format is BIP-93 codex32 directly. Documented §II.

## XpubCompact

`mk-codec`'s 73-byte compact-xpub form at `bytecode/xpub_compact.rs:32`. Fields: `version: [u8; 4]`, `parent_fingerprint: [u8; 4]`, `chain_code: [u8; 32]`, `public_key: [u8; 33]` — depth and child_number stripped because they are reconstructable from the path. Constructed via `XpubCompact::from_xpub`; reversed via `reconstruct_xpub` (caller must guarantee non-empty origin path or `reconstruct_xpub` panics). Defined §V.2.3.9.

## XpubNotInPolicy

The third cosigner-mapping failure mode in verify-bundle: a supplied `--mk1` group decoded cleanly but its xpub is absent from the descriptor's `tlv.pubkeys` set. The wrong-key attack indicator (or evidence that a user supplied an mk1 card from a different wallet). Defined `verify_bundle.rs:835`; emission at `:1128-1131`; precedence rank highest among the three modes. Defined §IV.2.
