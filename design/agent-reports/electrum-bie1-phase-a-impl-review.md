# Electrum BIE1 Phase-A implementation — opus end-of-phase review (verbatim)

Review of the uncommitted Phase-A working tree (`electrum_crypto.rs` ECIES BIE1 + Cargo.toml deps), feature-dev:code-reviewer (opus). Persisted per CLAUDE.md. **VERDICT: GREEN (0 Critical / 0 Important / 3 Minor).**

## Verified clean (line-by-line vs spesmilo/electrum @ 2e640c83)
- **Order constant** (`electrum_crypto.rs:218-219`): `64 zeros ‖ FFFF…0364141` = exactly the secp256k1 order, 128 hex chars; `\`-newline continuation strips leading whitespace; `from_be_hex` panics on malformed input; the `pw123` scalar KAT transitively proves it.
- **Mod-n reduction** (`:304-307`): `U512::from_be_slice(pbkdf2_64).rem(&n)` → low 32 BE bytes. Correct.
- **PBKDF2** (`:299`): `Hmac<Sha512>`, salt `b""`, 1024, dkLen 64 (distinct from BSMS c=2048 / Format A sha256d).
- **Offsets** (`:330-340`): magic `[0..4]`, ephemeral `[4..37]`, ct `[37..len-32]`, mac `[len-32..]`; `len>=85` first.
- **ECDH/KDF** (`:348-358`): `mul_tweak` → `.serialize()` (33B) → `Sha512` → iv/key_e/key_m `[0:16]/[16:32]/[32:64]`; AES-128 (16-byte key).
- **HMAC-before-decrypt** (`:362-364`): `verify_slice` (constant-time) over `raw[..len-32]` before any AES — no padding oracle.
- **`mul_tweak().expect()`** (`:348-350`): scalar provably in `[1,n-1]` (`InvalidScalar`-on-zero + reduction `<n` + `from_be_bytes` rejects `≥n`); attacker controls only the ephemeral point (`from_slice` rejects non-points; `s·P=O` impossible for prime-order n, `s∈[1,n-1]`). Unreachable.
- **Secret hygiene (production)**: PBKDF2 output, reduction bytes, scalar, sha512 `key`, shared point, plaintext all `Zeroizing`; `iv/key_e/key_m` borrow into the `Zeroizing` key. No un-zeroized production buffer.
- **Error taxonomy**: alphabetical; `HmacMismatch`/`AesDecryptFailure` distinct (Phase A); Display leaks nothing; BIE2 caught before key derivation.
- **Scope**: no CLI/sniff/version/tag/GUI; full storage-file fixture + CLI correctly deferred.

## Minor (no action required for Phase A)
- **M1**: test-only `ecies_encrypt_storage_for_test` `key`/`ecdh`/`buf` not `Zeroizing` (`#[cfg(test)]`, test password) — cosmetic parity only; production path is `Zeroizing`.
- **M2**: Format A `derive_key`/`encrypt_field` `buf` (pre-existing Cycle 6a, out of scope).
- **M3**: `PW123_SCALAR_HEX` provenance comment — self-checking via live KATs.

## Test adequacy
3 KATs are Electrum's OWN committed `test_decrypt_message` vectors (verbatim). Negatives all real+distinct (bad-base64, too-short, bad-magic, BIE2, bad-ephemeral, mac-tamper, wrong-password→HmacMismatch, non-zlib→ZlibDecompressFailure). zlib KAT is a genuine Python-stdlib `789c…` oracle. Test-only encrypt helper's ECDH symmetry (`recip_pk × eph == eph_pk × recip`) is sound. KATs cannot pass unless the whole mod-n + ECDH + KDF + AES + HMAC chain is byte-correct. **Phase A shippable as-is.**
