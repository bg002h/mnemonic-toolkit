# Cycle 6 — P0 STRICT-GATE recon dossier

**Date:** 2026-05-21
**Cycle target:** `mnemonic-toolkit-v0.31.0` (electrum-encrypted) + paired `mnemonic-gui-v0.16.0`.
**Source SHA at recon time:** master HEAD post-Cycle-5 (`0555008` + downstream Cycle 6a artifacts).

## A1 — Critical scope-correction finding

**The parent FOLLOWUP body claims "PBKDF2 + AES-CBC" — this is WRONG.**

Verified against `electrum/crypto.py` (Electrum mainline source, fetched via gh CLI at 2026-05-21):

```python
def _hash_password(password, *, version):
    if version == 1:
        return sha256d(pw)       # ← double SHA-256, NOT PBKDF2

def _pw_encode_raw(data, password, *, version):
    secret = _hash_password(password, version=version)
    ciphertext = EncodeAES_bytes(secret, data)   # iv || aes-cbc(plaintext + PKCS7)
    return ciphertext
```

**Actual scheme:** `key = sha256d(password)` → AES-256-CBC encrypt with random 16-byte IV → PKCS7 pad → output `iv || ciphertext` → base64-encode.

No PBKDF2 iteration. No salt (key is deterministic from password). The "PBKDF2" framing in the FOLLOWUPS.md body at L2576 is incorrect.

**Cycle 6 brainstorm + plan-doc correct this** by spec'ing `sha256d(password) + AES-256-CBC` as the actual scheme. FOLLOWUP body update tracked as a Phase 7 closure update.

## A2 — Two distinct encryption surfaces in Electrum

Recon discovered TWO distinct encryption modes in Electrum:

### Format A — field-level encryption (`pw_encode_bytes` / `pw_decode_bytes`)

Used when individual sensitive fields in the wallet JSON are encrypted while the rest of the file stays plaintext-JSON. The wallet has `"use_encryption": true` AND sensitive fields like `"seed": "<base64-ciphertext>"`.

**Wire format:** `base64(iv (16 bytes) || aes_cbc(plaintext + PKCS7, key, iv))`

NO version-prefix, NO MAC. Decryption MUST validate via PKCS7-padding strip + UTF-8 decode + (optionally) downstream parse semantics.

**Cycle 6 IN-SCOPE.** This is the surface that the current refusal at `wallet_import/electrum.rs:305-313` rejects.

### Format B — whole-file storage encryption (`pw_encode_with_version_and_mac` / `pw_decode_with_version_and_mac`)

Used when the user enables "Encrypt wallet file" in the Electrum GUI. The entire wallet file body becomes a single base64 blob.

**Wire format:** `base64(version_byte || iv (16 bytes) || aes_cbc(plaintext + PKCS7, key, iv) || mac (4 bytes))`

The 4-byte MAC is `sha256(plaintext)[:4]` — Encrypt-and-MAC ordering per `electrum/crypto.py:280`.

**Cycle 6 OUT-OF-SCOPE.** This surface is NOT JSON-parseable at the wallet-file level; the current Electrum sniff/parse pipeline would fail at JSON parse with a different error message. Filing as separate FOLLOWUP `wallet-import-electrum-encrypted-storage-format-b` at Cycle 6 close.

## A3 — Cargo dep status

| Dep | Status |
|---|---|
| `sha2 = "0.10"` | ✓ already direct dep at `Cargo.toml:L?` |
| `hmac = "0.12"` | ✓ already direct dep (unused for sha256d but available) |
| `pbkdf2 = "0.12"` | ✓ already direct dep (NOT needed for Format A) |
| `aes` | ✓ transitive via `bitcoin = "0.32" features=["base64"]`; Cycle 6 adds direct dep at `aes = "0.8"` |
| `cbc` | ✗ NOT a dep; Cycle 6 adds `cbc = "0.1"` |
| `base64` | ✓ transitive via bitcoin; Cycle 6 adds direct dep at `base64 = "0.22"` |

**Two new direct deps:** `cbc = "0.1"` + `base64 = "0.22"`. `aes = "0.8"` already transitively present at compatible version — making it a direct dep is a no-op for the lock-file resolver.

The brainstorm's "minimal new deps" claim holds (`cbc` is genuinely new; `aes` + `base64` are already there).

## A4 — Existing PBKDF2 / HMAC precedent (NOT used for Cycle 6 but verified)

`crates/mnemonic-toolkit/src/slip39/feistel.rs:197` uses `pbkdf2::<Hmac<Sha256>>(...)` — direct precedent for PBKDF2 invocation in this toolkit. NOT applicable to Cycle 6 (Format A uses sha256d, not PBKDF2).

`bip85.rs:49-51` uses `bitcoin::hashes::HmacEngine<sha512::Hash>` — HMAC-SHA512 precedent (different from Electrum's HMAC-SHA256). NOT applicable.

`electrum.rs:13` already imports `bitcoin::hashes::sha512` for separate Electrum-specific purposes; NOT the same as the field-encryption sha256d we'll need.

## A5 — Current refusal site

`crates/mnemonic-toolkit/src/wallet_import/electrum.rs:305-313`:

```rust
let use_encryption = obj
    .get("use_encryption")
    .and_then(|v| v.as_bool())
    .unwrap_or(false);
if use_encryption {
    return Err(ToolkitError::ImportWalletParse(
        "import-wallet: electrum: encrypted wallet files require decrypting via 'electrum --decrypt-wallet' first; encrypted ingest not yet supported (FOLLOWUP wallet-import-electrum-encrypted)"
            .to_string(),
    ));
}
```

Phase 3 (6b) updates this site: if a decrypt password is threaded through, ATTEMPT decryption + continue; else refusal fires with an updated message pointing at `--decrypt-password*`.

## A6 — sha256d known-vector for unit-test cross-check

Trezor's BIP-39 test vector "abandon abandon … about" entropy zero-vector NOT applicable here (different scheme). For Cycle 6 testing, generate vectors from `electrum/crypto.py` directly:

```python
import hashlib
def sha256d(b):
    return hashlib.sha256(hashlib.sha256(b).digest()).digest()

# Known vector for cross-impl smoke:
sha256d(b"test-password").hex()
# Expected output (verified empirically next session):
# <hash to be locked at Phase 1 implementation>
```

The same input/output applies to the toolkit's Rust implementation:
```rust
let pw = b"test-password";
let mut hasher = Sha256::new();
hasher.update(pw);
let first = hasher.finalize_reset();
hasher.update(&first);
let key = hasher.finalize();
assert_eq!(hex::encode(key), "<lock at Phase 1>");
```

Phase 1 implementation locks the exact known-vector hash at unit-test write.

## A7 — Manual chapter prelude

The current chapter-45 §"Encrypted Electrum wallets" subsection points at this FOLLOWUP slug with "deferred" framing. Phase 5 (6b) rewrites the subsection to document the `--decrypt-password*` workflow.

The current `45-foreign-formats.md` already mentions `electrum --decrypt-wallet` as the out-of-band workaround. Phase 5 (6b) RETAINS this guidance — even with native support, an out-of-band path is useful for users without the toolkit installed.

## Recon verdict

**GREEN with one scope-correction finding (A1: PBKDF2 framing is wrong; actual scheme is sha256d). Folded into Cycle 6 brainstorm + plan-doc.** Format B is out of scope, deferred to a new FOLLOWUP at Cycle 6 close (per A2). Cargo deps net-add: 2 (`cbc` + `base64`; `aes` is no-op promotion to direct dep).
