# Architect blueprint — Electrum BIE1 storage-encrypted wallet import

**Provenance:** Cycle 19 recon (2026-05-21), `feature-dev:code-architect` (opus), grounded read of current `origin/master`. Feature is **deferred** (user: "correct followup + defer"); this is the build/test plan for whenever it is greenlit. Companion to `design/FOLLOWUPS.md` entry `wallet-import-electrum-encrypted-storage-format-b` (verified crypto + library recon). Crypto verified vs `spesmilo/electrum` `crypto.py` + `storage.py`.

---

## Recommendation (lead)

Add the BIE1 ECIES decrypt as a new sibling function set **inside the existing `electrum_crypto.rs`** (it is the "Electrum crypto" home; `decrypt_field` and the ECIES path coexist as two named pipelines), exposing `pub fn ecies_decrypt_storage(blob: &[u8], password: &[u8]) -> Result<Zeroizing<Vec<u8>>, EciesDecryptError>` plus testable per-stage helpers, with a **new** library-local `EciesDecryptError` enum (do not overload `ElectrumDecryptError` — the failure modes and the AES-256/sha256d-vs-AES-128/ECDH semantics are disjoint). For CLI, **pick option (a): add `--decrypt-password*` to `import-wallet`** with orchestrator-side pre-decryption gated on a new `ElectrumParser::sniff_bie1(blob)` predicate — this mirrors the existing BSMS decrypt-then-parse orchestration at `cmd/import_wallet.rs:890` exactly and preserves the `WalletFormatParser` trait surface untouched. Ship as **two sessions (library-first 6a/6b split) + SemVer-MINOR** (net-new flag NAMEs on import-wallet ⇒ mandatory mnemonic-gui schema-mirror lockstep + manual mirror).

## 1. Module placement & public API

Home: `crates/mnemonic-toolkit/src/electrum_crypto.rs` (extend; do not fork a new module). Co-locating keeps `decrypt_field` (Format A) and `ecies_decrypt_storage` (Format B) as two named pipelines, matching how `bsms_crypto.rs` keeps `derive_encryption_key`/`compute_mac`/`decrypt` together (`bsms_crypto.rs:98-183`).

Public API (mirror `bsms_crypto.rs` layered-helper style so each stage is independently KAT-testable):
- `pub fn derive_storage_eckey(password: &[u8]) -> Zeroizing<[u8; 32]>` — PBKDF2-SHA512 → mod-n (see §2).
- `pub fn ecies_kdf(ephemeral_pubkey: &[u8], scalar: &[u8;32]) -> Result<Zeroizing<[u8;64]>, EciesDecryptError>` — ECDH point-mul → compressed → sha512 (see §3).
- `pub fn ecies_decrypt_storage(blob, password) -> Result<Zeroizing<Vec<u8>>, EciesDecryptError>` — full pipeline incl. magic check, HMAC verify, AES-128-CBC, zlib.

New error enum (new type, hand-rolled `Display`, mirroring `electrum_crypto.rs:45-90` and the alphabetical-variant CLAUDE.md convention): variants `AesDecryptFailure`, `Base64DecodeFailure(String)`, `Bie2Unsupported`, `HmacMismatch`, `InvalidMagic([u8;4])`, `InvalidScalar`, `TooShort{got}`, `ZlibDecompressFailure(String)`. Map `HmacMismatch | AesDecryptFailure` to a UNIFIED non-leaky CLI message (mirror `cmd/electrum_decrypt.rs:73-83`).

## 2. Mod-n scalar reduction (the fiddly bit — HIGHEST-RISK / SPIKE FIRST)

In-tree the codebase only ever feeds *already-valid* 32-byte scalars to `SecretKey::from_slice` (`parse_descriptor.rs:851`, `bip85.rs:115/145`, `cost/dummy_keys.rs:25`). PBKDF2-SHA512 emits 64 bytes; you need `int(64 bytes) mod n`. secp256k1 0.29.1 has **no public 512-bit reducer** (`Scalar` is a tweak type, no big-int `rem`); `num-bigint` and `crypto-bigint` are NOT in-tree. Candidate approaches (architect could not fully settle — RESOLVE VIA SPIKE before the plan locks): (i) add `k256` (RustCrypto pure-Rust secp256k1) which has first-class `Scalar: Reduce<U512>` modular reduction; (ii) add `crypto-bigint` (`U512::rem` against the secp256k1 order constant); (iii) hand-roll a `reduce_512_mod_n` (Barrett or byte-wise `acc = (acc<<8 + byte) mod n`) with explicit KATs and no new dep. This parallels FOLLOWUPS' "crypto citations decay — re-grep" warning. Edge case: reduced scalar == 0 → return `InvalidScalar` (practically unreachable; assert it).

## 3. EC point multiplication

`ephemeral_pubkey = blob[4..37]` → `PublicKey::from_slice(&blob[4..37])`. Multiply by the reduced scalar via `PublicKey::mul_tweak(&secp, &Scalar::from_be_bytes(scalar32)?)` (secp256k1 0.29.1 API), then `.serialize()` for the compressed 33-byte form. Matches Electrum's `(ephemeral_pubkey * privkey).get_public_key_bytes(compressed=True)`. Use `Secp256k1::new()` as in `synthesize.rs:13`/`bip85.rs:13`. KAT this stage against a vendored `(ephemeral_pubkey, scalar) → compressed_point` vector.

## 4. CLI integration shape — option (a), justified

Route through **`import-wallet`** with new `--decrypt-password` / `--decrypt-password-file` / `--decrypt-password-stdin` flags, structured as a struct-level `ArgGroup` (copy from `cmd/electrum_decrypt.rs:27-59`, but `required(false)` since most imports are plaintext). The v0.33.0 `electrum-decrypt` subcommand emits *plaintext*, not a parsed wallet — it cannot produce the unified-card import output. The decrypted blob here is *Electrum wallet JSON* that must feed the existing parser — exactly the BSMS decrypt-then-parse pattern at `import_wallet.rs:890`. Reject (b) (new subcommand duplicates orchestration) and (c) (electrum-decrypt has no parser hook). New flag NAMEs trip the GUI `schema_mirror` gate → mandatory paired mnemonic-gui PR.

## 5. Sniff/parse integration

A BIE1 blob is base64 (not `{`), so the existing `ElectrumParser::sniff` (`electrum.rs:217-254`, requires leading `{`) returns false — **no false-positive risk** against the JSON path or other parsers. Slot pre-decrypt in the **orchestrator** (not the parser): when `--decrypt-password*` present (or auto: blob base64-decodes to `len≥85 && [..4]==BIE1`), call `ecies_decrypt_storage`, replace `blob` with recovered JSON, fall through to normal sniff/parse (now sniffs `electrum`). Add `pub(crate) fn looks_like_bie1(blob) -> bool` to `electrum.rs`. Parser trait surface untouched (same lesson as BSMS Round-2, `import_wallet.rs:873-875`).

## 6. BIE2 handling

`blob[..4] == b"BIE2"` → `EciesDecryptError::Bie2Unsupported` → `ToolkitError::BadInput`: *"import-wallet: this Electrum wallet is encrypted with a hardware-device key (BIE2 / XPUB_PASSWORD); it cannot be decrypted from a password. Decrypt it in Electrum with the original device first, then re-import."* Detect BIE2 BEFORE key derivation (no wasted work, no oracle). Permanently unsupported per FOLLOWUPS §2944.

## 7. Secret hygiene

- Password: `Zeroizing<String>` + `mlock::pin_pages_for` (copy `cmd/electrum_decrypt.rs:102-119`).
- Derived scalar `Zeroizing<[u8;32]>`, ECDH `key` `Zeroizing<[u8;64]>`, `key_e`/`key_m` — all `Zeroizing`. Decrypted wallet JSON `Zeroizing<Vec<u8>>` (whole-file plaintext can carry seed/xprv in older versions — treat as secret).
- `flag_is_secret` (`secrets.rs:50`): add `--decrypt-password` and `--decrypt-password-stdin` (NOT `--decrypt-password-file` — it's a path). **v0.33.1 lesson — MUST land in the same PR** + GUI `secrets::flag_is_secret` mirror in lockstep.
- Inline `--decrypt-password` → `secret_in_argv_warning`. import-wallet output is non-secret (descriptor card) → no new stdout advisory.

## 8. Test architecture

Layered KAT cells (`electrum_crypto.rs` `#[cfg(test)]`, mirroring `bsms_crypto.rs:217+`):
1. `derive_storage_eckey` — PBKDF2-SHA512(pw, salt=b"", 1024, 64) → mod-n scalar KAT (hex-pinned).
2. `ecies_kdf` — `(ephemeral_pubkey, scalar) → sha512(compressed_point)` → iv/ke/km KATs.
3. AES-128-CBC decrypt KAT. 4. HMAC-SHA256 verify KAT. 5. zlib decompress KAT (canonical).

End-to-end vendored fixture: `tests/external/regen_electrum_bie1.py` + README (mirror `regen_coinkite_vectors.py`, Cycle 17). Vendor `(blob.b64, password, expected_wallet.json)`; CI cell asserts byte-exact `ecies_decrypt_storage(blob, pw) == expected JSON`. **Oracle authority**: prefer a minimal Python script that imports Electrum's actual `electrum.crypto` / `electrum.storage` to GENERATE the encrypted file (most authoritative — it IS Electrum), pinned to a spesmilo/electrum SHA in the README; bitcore-ecies JS as fallback. **Vendored-only, no live-CI** (Cycle 17 discipline): pin SHA, commit blob, no network/pip in CI. Negatives: wrong-password (`HmacMismatch`), truncated (`TooShort`), corrupted-magic (`InvalidMagic`), BIE2 (`Bie2Unsupported`), HMAC-tampered byte. Plus `tests/cli_import_wallet_electrum_bie1.rs` for the full `import-wallet --decrypt-password*` path + unified card output.

## 9. Phasing & SemVer

**Two-session library-first split (Cycle 6a/6b precedent):**
- *Phase A (library):* `electrum_crypto.rs` ECIES functions + `EciesDecryptError` + all KAT cells + vendored fixture + regen script. **Mandatory opus R0 on the brainstorm BEFORE impl** (must lock the §2 mod-n approach). Pre-tag.
- *Phase B (CLI/ship):* import-wallet flags + ArgGroup + orchestrator pre-decrypt + `looks_like_bie1` + `flag_is_secret` + integration tests + manual `40-cli-reference` update + paired mnemonic-gui schema-mirror PR. Tag.

**SemVer: MINOR.** Net-new flag NAMEs on `import-wallet` + new decrypt capability. Lockstep: (1) mnemonic-gui `schema/mnemonic.rs` import-wallet FlagSchema + `secrets::flag_is_secret` mirror (MANDATORY); (2) `docs/manual/src/40-cli-reference/` import-wallet chapter (MANDATORY). File a FOLLOWUP for the BIE2-permanent-unsupported doc note.

## 10. Top risks/gotchas

1. **Mod-n reduction (§2)** — the only real unknown; secp256k1 0.29.1 has no public 512-bit reducer. SPIKE before the plan locks.
2. **PBKDF2 salt EMPTY (`b""`), iterations=1024** — different from BSMS (`b"No SPOF"`/c=2048) and Format A (sha256d). Three Electrum key schemes coexist in two files; a copy-paste would silently produce wrong keys. KAT each.
3. **AES-128, not AES-256** — codebase only uses `Aes256` today (`electrum_crypto.rs:33`, `bsms_crypto.rs:47`). `use aes::Aes128` + new `cbc::Decryptor<Aes128>` alias; `aes` 0.8.4 supports it, no new dep.
4. **HMAC-before-decrypt ordering** — BIE1 is Encrypt-then-MAC over `blob[:-32]`; verify HMAC BEFORE AES-CBC to avoid a PKCS7 padding oracle. The wrong-password signal is the HMAC, NOT the PKCS7 strip.
5. **Compressed-point SEC form** (`.serialize()`, 33 bytes, not uncompressed) — Electrum hashes the compressed point; a mismatch surfaces only as `HmacMismatch`.
6. **`len ≥ 85` + exact slice offsets** (`[4..37]`, `[37..-32]`, `[-32..]`) — classic ECIES off-by-one; pin with a truncation KAT.
7. **Crypto-citation decay** (FOLLOWUPS §2932 — TWO prior misidentifications in this family) — re-grep spesmilo/electrum at the pinned SHA at plan-write time; record SHA in the SPEC.
8. **`flate2` is the only new dep** — confirmed absent from Cargo.lock; add with default features (miniz_oxide backend, pure-Rust, no system zlib).
