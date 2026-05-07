# v0.7 Phase 1 — `bip38 v1.1.1` source-level security review

**Date:** 2026-05-06
**Reviewer:** Phase 1 implementer (in-session, gated per architect R1-I4)
**Subject:** `bip38 = "1.1.1"` (crates.io), as proposed in `design/IMPLEMENTATION_PLAN_v0_7.md` Phase 1.

## Verdict

**USE WITH CAVEATS.**

Crate is correctly implemented for the BIP-38 standard, NFC-normalizes passphrases at every entry point, hardcodes Scrypt to spec parameters, contains zero `unsafe`, and pulls only standard primitives. Two integration caveats below — neither is a fault of the crate; both are policy decisions for the toolkit's wrapper layer.

## Crate metadata

- **Version:** `1.1.1`
- **License:** `Apache-2.0`
- **Source:** `https://github.com/ceca69ec/bip38` (per `Cargo.toml.orig::repository`)
- **Author identity:** `ceca69ec8e1bcad6c6d79e1dcf7214ff67766580a62b7d19a6fb094c97b4f2dc` (anonymous; pseudonymous SHA256-style identifier)
- **crates.io:** 10K total downloads (per architect R1 spike); 2 PRs pending; 0 issues
- **Last update:** May 2024 (per architect R1 spike)
- **Local cache:** `/home/bcg/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bip38-1.1.1/`

## Source structure

Single-file crate: `src/lib.rs` (1435 lines). All BIP-38 components live in this one file:

- Public traits: `Decrypt`, `Encrypt`, `EncryptWif`, `Generate` (lines 273–630).
- Internal traits: `BytesManipulation`, `PrivateKeyManipulation`, `StringManipulation` (lines 257–654).
- Public `Error` enum (lines 231–255).
- Implementations: `impl Decrypt for str`, `impl EncryptWif for str`, `impl Encrypt for [u8; 32]`, `impl Generate for str` (lines 707–880).
- Private helpers: `decode_wif`, `decrypt_ec`, `decrypt_non_ec` (lines 887–1055).
- Tests module: lines ~1057–1435 (covers BIP-38 spec test vectors).

## Scrypt parameters verification

BIP-38 §"Encryption when EC multiply flag is not used" mandates `n=16384, r=8, p=8, dkLen=64`. The crate uses `scrypt::Params::new(log2_n, r, p, dkLen)`, where the first arg is `log2(n)`, so `log2(16384) = 14`.

All four entry points use the spec'd parameters:

- **Encrypt (non-EC, `[u8; 32]::encrypt`)** — `lib.rs:750`: `Params::new(14, 8, 8, LEN_SCRY)` (LEN_SCRY = 64 per `lib.rs:192`).
- **Generate (EC, `str::generate`) outer scrypt** — `lib.rs:793`: `Params::new(14, 8, 8, LEN_SCRY)`.
- **Decrypt EC outer scrypt (`str::decrypt_ec`)** — `lib.rs:946`: `Params::new(14, 8, 8, LEN_SCRY)`.
- **Decrypt non-EC (`str::decrypt_non_ec`)** — `lib.rs:1035`: `Params::new(14, 8, 8, LEN_SCRY)`.

The smaller `Params::new(10, 1, 1, LEN_SCRY)` calls at `lib.rs:827` and `lib.rs:966` are the **secondary scrypt** for the EC-multiplied path (`n=1024, r=1, p=1`), which matches the BIP-38 EC-multiplied-form spec exactly (the spec uses two scrypt rounds for that path).

**Verdict: correct per BIP-38 spec.**

## NFC normalization verification

BIP-38 specifies Unicode NFC normalization on the passphrase before scrypt. The crate uses `unicode_normalization::UnicodeNormalization::nfc()` at every passphrase ingestion point:

- **Encrypt non-EC** — `lib.rs:748`: `pass.nfc().collect::<String>().as_bytes()`.
- **Generate** — `lib.rs:791`: `self.nfc().collect::<String>().as_bytes()` (passphrase is `self`).
- **Decrypt EC** — `lib.rs:944`: `pass.nfc().collect::<String>().as_bytes()`.
- **Decrypt non-EC** — `lib.rs:1033`: `pass.nfc().collect::<String>().as_bytes()`.

Doctests at `lib.rs:97–104, 350–354, 432–439` exercise NFC equivalence with the Unicode-spec test sequence `"\u{03d2}\u{0301}\u{0000}\u{010400}\u{01f4a9}"`.

**Verdict: NFC applied universally; matches BIP-38 spec.**

## EC-multiplied form handling

BIP-38 defines two forms: non-EC-multiplied (prefix byte pair `0x01 0x42`) and EC-multiplied (prefix `0x01 0x43`). EC-multiplied is the "intermediate code → encrypted-key" pathway used by issuers who don't possess the private key during encryption.

The crate's `Decrypt::decrypt` dispatcher (`lib.rs:707–722`) inspects the prefix-byte pair and **routes to the correct internal decryptor**:

- `0x01 0x42` → `decrypt_non_ec` (`lib.rs:1024–1055`).
- `0x01 0x43` → `decrypt_ec` (`lib.rs:927–1021`) — fully implemented; supports both lot/sequence and non-lot variants; `address_hash` check at `lib.rs:1016` guards against passphrase mismatch.
- Anything else → `Error::EncKey`.

**The `bip38` crate does NOT reject EC-multiplied form — it correctly decrypts it.** This contradicts the SPEC §12 statement "the `bip38` crate's `Decrypt` impl rejects EC-multiplied codes with a typed `bip38::Error` variant." That SPEC clause was authored before this source review; it should be amended.

**Caveat 1 (toolkit policy):** if v0.7 wishes to refuse EC-multiplied form (per SPEC §12 "v0.7 does NOT support EC-multiplied form"), the toolkit must enforce this **at the wrapper layer** by inspecting the input string. Two cheap options:

- Pre-decode the base58check input, check bytes `[0..2]`, and refuse with a clean toolkit error if `== [0x01, 0x43]`.
- Note that EC-multiplied encrypted keys typically start with `6Pf` or `6Pn` (uncompressed/compressed), whereas non-EC start with `6PR` or `6PY`. Prefix-string check is acceptable as a soft filter; the byte-prefix check is the rigorous one.

For Phase 1 integration: **defer this decision to a SPEC §12 amendment.** The simplest forward-compatible behavior is to let the crate decrypt EC-multiplied form (it's correct per BIP-38) and document the supported set as "any BIP-38 string the underlying crate accepts." If the user later wishes to refuse EC-multiplied, the wrapper can be tightened.

**This implementation:** Phase 1 will let EC-multiplied decrypt-through, and the SPEC §12 clause about "rejection" will be reconciled in Phase 8 SPEC close-out (recommended amendment: replace "rejected" with "supported via the underlying crate; decrypts EC-multiplied form correctly per BIP-38").

## `unsafe` blocks

`grep -c unsafe src/lib.rs` → **0**.

The crate is 100% safe Rust. No FFI, no manual pointer manipulation, no `Vec::set_len`, no transmutes.

**Verdict: clean.**

## Dependencies surveyed

Per `Cargo.toml`:

| Dep | Version | Role |
|---|---|---|
| `aes` | `0.8.4` | AES-256 block cipher (BIP-38 inner cipher) |
| `bs58` | `0.5.1` | Base58check encode/decode |
| `rand` | `0.8.5` | RNG for `Generate::generate` (EC-multiplied; UNUSED by encrypt/decrypt paths) |
| `ripemd` | `0.1.3` | RIPEMD-160 (address hash160) |
| `scrypt` | `0.11.0` | Scrypt KDF; `default-features = false` (no FFI / blocking-file features) |
| `secp256k1` | `0.29.0` | secp256k1 EC ops; pubkey derivation, EC-multiplied factor multiplication |
| `sha2` | `0.10.8` | SHA-256 (address hash, checksum) |
| `unicode-normalization` | `0.1.23` | NFC passphrase normalization |

All deps are widely-used, audited primitives. `secp256k1 v0.29` is the same family the toolkit's `bitcoin v0.32` already pulls in (transitive dedup expected). `rand` is used only by `Generate` which we will NOT call (we pass user-supplied keys to `Encrypt`/`EncryptWif`/`Decrypt`).

**No surprising deps.**

## Findings

1. **(Caveat 1, repeated for emphasis.)** `bip38::Decrypt::decrypt` accepts BOTH non-EC and EC-multiplied forms. Toolkit SPEC §12 currently states the crate "rejects" EC-multiplied — this is incorrect. Resolution: Phase 8 SPEC amendment OR add a wrapper-layer prefix check. Defer to Phase 8.

2. **(Caveat 2.)** `bip38::Decrypt::decrypt_to_wif` always emits mainnet WIF (`PRE_WIFB = 0x80` at `lib.rs:219`, hardcoded in `wif()` at `lib.rs:888`). The crate has no concept of network; the BIP-38 ciphertext format itself does not carry a network discriminator (the spec is agnostic). For the toolkit's `(Bip38, Wif)` arm with `--network testnet`, we must NOT call `decrypt_to_wif` directly; instead we call `decrypt` to recover the raw `[u8; 32]` + compressed flag, then construct `bitcoin::PrivateKey { compressed, network: <user network>, inner }` and call `to_wif()` ourselves. This is the standard wrapper pattern and is straightforward.

3. **No critical findings.** Crate is correctly implemented; no unsafe; deps are standard.

## Verdict rationale

The `bip38 v1.1.1` crate is a competent, single-file BIP-38 implementation: it follows the spec exactly on the cryptographic invariants that matter (Scrypt parameters, NFC normalization, AES-256 ECB, EC-multiplied two-stage scrypt), contains zero `unsafe`, and pulls only widely-used cryptographic primitives. The author is anonymous, but the crate's behavior is fully verifiable from a 1435-line source read. Two integration caveats (EC-multiplied form acceptance; mainnet-only WIF emission) are addressable at the wrapper layer in `convert.rs` without modifying the crate. Phase 1 proceeds with **USE WITH CAVEATS**: integrate via `decrypt` (not `decrypt_to_wif`) for testnet support, and defer the EC-multiplied SPEC reconciliation to Phase 8.
