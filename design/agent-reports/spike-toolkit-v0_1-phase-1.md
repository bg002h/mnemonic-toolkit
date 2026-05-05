# Phase 1 Spike Memo — Toolkit v0.1 Sibling-API Verification

**Date:** 2026-05-04
**Reviewer:** the spike runner (and Phase 1 reviewer at task 1.10)

Locked deps: `bitcoin = "0.32"`, `bip39 = "2"`, `ms-codec @ ms-codec-v0.1.0`, `mk-codec @ mk-codec-v0.2.1`, `md-codec @ md-codec-v0.16.1`. Spike crate at `/tmp/toolkit-spike/` (ephemeral, not committed).

## Verified API surface

### `bitcoin = "0.32"` (SPEC §4.1, §4.3, §4.6.1, §4.8, §6.4.2)

`spike_bitcoin` confirms:

- `Xpriv::new_master(NetworkKind::Main, &seed)` — OK.
- `master.derive_priv(&secp, &path)` — OK.
- `Xpub::from_priv(&secp, &xpriv)` — OK.
- `xpub.chain_code.to_bytes() -> [u8; 32]` — observed `4c78dc09788205d04f0081b4fff22c0b59975430d46b1189fb3610b38261c6a1` for `m/84'/0'/0'` of zero seed.
- `xpub.public_key.serialize() -> [u8; 33]` — observed `0211ba4e0452ca19a308dfdd8d4bf4adeeb2f696868002eff185897dd5a34dec64`.
- `xpub.network` — `Main` (Debug print: `Main`).
- `xpub.depth` — `3`.
- `master.fingerprint(&secp).to_bytes() -> [u8; 4]` — observed `c345e1e9` (lowercase 8-hex Display).
- `Fingerprint::from_str` — accepts upper- and lower-case 8-hex; both round-trip equal; Display always lowercase 8-hex.
- `Xpub::from_str("invalid")` returns `Err(Base58(Decode(InvalidCharacterError { invalid: 108 })))` — `bip32::Error::Base58` variant present.

### `bip39 = "2"` (SPEC §4.1, §6.4.1)

`spike_bip39` confirms:

- `Mnemonic::parse_in(Language::English, "abandon × 23 art")` — OK.
- `to_entropy() -> Vec<u8>` — 32 bytes, all-zero (hex `0000…0000`).
- `to_seed("") -> [u8; 64]` — observed first 16 bytes `408b285c123836004f4b8842c89324c1`.
- `Mnemonic::from_entropy_in(Language::English, &entropy)` — round-trips to `"abandon abandon … abandon art"`.
- `Language::all-languages` feature available (10 wordlists per SPEC §4.1, locked at SPEC §6.4.1).

### `ms_codec` (SPEC §4.4)

Not exercised by a dedicated spike binary (per plan). API confirmed by reading source:

- `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/src/lib.rs:50-55` — pub re-exports: `decode::decode`, `encode::encode`, `Error`, `Result`, `inspect::{inspect, InspectReport}`, `Payload`, `PayloadKind`, `Tag`.
- `Error` is `#[non_exhaustive]` (`error.rs:8`). Named variants: `Codex32(codex32::Error)`, `WrongHrp{got}`, `ThresholdNotZero{got}`, `ShareIndexNotSecret{got}`, `TagInvalidAlphabet{got}`, `UnknownTag{got}`, `ReservedTagNotEmittedInV01{got}`, `ReservedPrefixViolation{got}`, `UnexpectedStringLength{got, allowed}`, `PayloadLengthMismatch{tag, expected, got}` — 10 variants. Matches SPEC §6.4.3 delegation contract (delegates to ms-cli's table, plus `_` fallthrough required).
- `payload.rs` — `Payload::new_seed(...)`, `Payload::new_master_seed(...)`, `Payload::new_xpriv(...)` constructors exist for the three v0.1 secret kinds.

### `mk_codec` (SPEC §4.5)

`spike_mk_codec` confirms:

- `KeyCard::new(policy_id_stubs: Vec<[u8; 4]>, origin_fingerprint: Option<Fingerprint>, derivation_path: DerivationPath, xpub: Xpub) -> KeyCard` — OK.
- `mk_codec::encode(&card) -> Result<Vec<String>>` — OK.
- `mk_codec::decode(&[&str]) -> Result<KeyCard>` — OK; round-trip succeeds.
- Single-card test produced **2 strings** (chunked) — first string len **111**, second len **80**. Both `mk1`-prefixed. (The plan's "single string len 60-80" expectation undershoots: the BIP-84 origin path + 33-byte pubkey + 32-byte chain code at v0.2.1 spills into a chunked pair. Not a SPEC divergence — SPEC §4.5 doesn't pin chunk count, only routing. SPEC §6.4.4 already covers `ChunkSetIdMismatch`/`ChunkedHeaderMalformed` etc., so chunked output is in scope.)
- `Error` is `#[non_exhaustive]` (`error.rs:18`). 22 named variants enumerated; matches SPEC §6.4.4 table 1:1 + `_` fallthrough.

### `md_codec` (SPEC §4.6)

`spike_md_codec` confirms (after fixing spike-side filler bytes — see "Errata" below):

- `Descriptor` typed-struct construction with `n`, `path_decl`, `use_site_path`, `tree`, `tlv` fields — OK.
- `PathDecl { n, paths: PathDeclPaths::Shared(OriginPath{..}) }` — present (`origin_path.rs:93`).
- `UseSitePath::standard_multipath()` — present (`use_site_path.rs:58`).
- `Tag::Wpkh`, `tree::{Node, Body}`, `Body::KeyArg{index}` — present.
- `TlvSection { use_site_path_overrides, fingerprints, pubkeys, origin_path_overrides, unknown }` — present (`tlv.rs:30`).
- `chunk::split(&Descriptor) -> Result<Vec<String>>` — OK; produced **3** `md1`-prefixed strings of len **67** each (template-only descriptor + xpub TLV at depth 3, fp, single key).
- `chunk::reassemble(&[&str]) -> Result<Descriptor>` — OK; round-trip preserves `tlv.pubkeys` and `tlv.fingerprints`.
- `compute_wallet_policy_id(&Descriptor) -> Result<WalletPolicyId>` + `WalletPolicyId::as_bytes() -> &[u8; 16]` — OK; observed policy_id `6650b9803b3c66210140540da8d765a0`, stub bytes `[0..4] = 6650b980`.
- `descriptor.is_wallet_policy() -> bool` — returned `true` for the wpkh + single-pubkey TLV case.
- `Error` NOT `#[non_exhaustive]` (`error.rs:6` shows only `#[derive(Debug, Error, PartialEq, Eq)]` — confirmed). 41 variants enumerated; matches SPEC §6.4.5 routing table exactly (2 Exit-1 + 38 Exit-2 + 1 Exit-3 = 41).

## SPEC patches needed

(none — SPEC r3 claims hold against actual sibling source.)

## Errata / surprises

- **mk1 chunked output:** plan's "single string, len 60-80" was a soft expectation; observed 2 strings (lens 111, 80). Not a SPEC violation; SPEC §4.5 + §6.4.4 anticipate chunked encodings. Synthesizer (Phase 2) and engraving-card output (§5) must treat mk1 as `Vec<String>`, which they already do.
- **md_codec spike pubkey filler:** plan's `[0x42; 65]` filler for the `tlv.pubkeys` slot panicked at `Descriptor::is_wallet_policy()` round-trip with `InvalidXpubBytes { idx: 0 }` — bytes `[32..65]` of a TLV pubkey payload must be a valid 33-byte SEC1 compressed pubkey (prefix 0x02 or 0x03), not arbitrary fill. Spike was patched to use the canonical filler from `crates/md-codec/src/identity.rs::deterministic_xpub` (`[0x11; 32] || 0x02 || [0x22; 32]`); first run with this filler succeeded. SPEC §4.6.1's worked code already builds `[chain_code; 32] || pk.serialize()` from a real xpub, so the SPEC-side claim is correct — the bug was in the spike harness's filler choice. No SPEC patch needed.
