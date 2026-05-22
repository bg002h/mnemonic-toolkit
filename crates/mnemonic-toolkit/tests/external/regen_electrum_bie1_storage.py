#!/usr/bin/env python3
"""Regenerate the Electrum BIE1 (user-password) storage-encrypted wallet
fixtures for the `cli_import_wallet_electrum_bie1` integration cells.

INDEPENDENT cross-impl oracle: this re-implements Electrum's storage
encryption faithfully per `spesmilo/electrum` @ 2e640c83
(`electrum/storage.py::WalletStorage.{get_eckey_from_password,encrypt_before_writing}`
+ `electrum/crypto.py::ecies_encrypt_message`), using a pure-Python EC stack
(`ecdsa`) + stdlib (`hashlib`/`hmac`/`zlib`) + `cryptography` (AES-128-CBC) —
NOT the toolkit's own code. The toolkit's DECRYPT is separately validated
byte-exact against Electrum's OWN committed `test_decrypt_message` KATs (see
`src/electrum_crypto.rs` tests); these storage fixtures additionally witness
the whole-file `zlib(json)` → ECIES framing end-to-end through the CLI.

A FIXED ephemeral key is used so the output is deterministic/reproducible.

Deps:  pip install --user ecdsa cryptography
Run:   python3 regen_electrum_bie1_storage.py
Writes the fixtures under ../fixtures/wallet_import/.
"""
import base64
import hashlib
import hmac
import os
import zlib

from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.primitives.padding import PKCS7
from ecdsa import SECP256k1, SigningKey, VerifyingKey

N = SECP256k1.order

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = os.path.join(HERE, "..", "fixtures", "wallet_import")

# Reuse the existing plaintext Electrum wallet (so the decrypted blob parses
# identically to the existing `electrum-standard-bip84-mainnet.json` cell:
# cosigners=1, network=mainnet).
PLAINTEXT = open(os.path.join(FIXTURES, "electrum-standard-bip84-mainnet.json"), "rb").read()
PASSWORD = "satoshi"
# Fixed ephemeral scalar for deterministic output (production uses os.urandom).
EPHEMERAL_SCALAR = 0x1111111111111111111111111111111111111111111111111111111111111111


def get_eckey_scalar_from_password(password: str) -> int:
    """Electrum storage.py:get_eckey_from_password — PBKDF2-HMAC-SHA512(pw,
    salt=b"", 1024, 64) reduced mod n (from_arbitrary_size_secret)."""
    secret = hashlib.pbkdf2_hmac("sha512", password.encode("utf-8"), b"", iterations=1024)
    return int.from_bytes(secret, "big") % N


def compressed_point(point) -> bytes:
    return VerifyingKey.from_public_point(point, curve=SECP256k1).to_string("compressed")


def ecies_encrypt_bie1(message: bytes, recipient_scalar: int, ephemeral_scalar: int) -> bytes:
    """Electrum crypto.py:ecies_encrypt_message (magic BIE1)."""
    recipient_point = SigningKey.from_secret_exponent(recipient_scalar, curve=SECP256k1).verifying_key.pubkey.point
    ephemeral_point = SigningKey.from_secret_exponent(ephemeral_scalar, curve=SECP256k1).verifying_key.pubkey.point
    ecdh_point = recipient_point * ephemeral_scalar
    ecdh = compressed_point(ecdh_point)
    key = hashlib.sha512(ecdh).digest()
    iv, key_e, key_m = key[0:16], key[16:32], key[32:64]
    padder = PKCS7(128).padder()
    padded = padder.update(message) + padder.finalize()
    enc = Cipher(algorithms.AES(key_e), modes.CBC(iv)).encryptor()
    ciphertext = enc.update(padded) + enc.finalize()
    encrypted = b"BIE1" + compressed_point(ephemeral_point) + ciphertext
    mac = hmac.new(key_m, encrypted, hashlib.sha256).digest()
    return base64.b64encode(encrypted + mac)


def main():
    scalar = get_eckey_scalar_from_password(PASSWORD)
    assert scalar != 0
    compressed = zlib.compress(PLAINTEXT)
    blob = ecies_encrypt_bie1(compressed, scalar, EPHEMERAL_SCALAR)

    bie1_path = os.path.join(FIXTURES, "electrum-bie1-storage-bip84.txt")
    with open(bie1_path, "wb") as f:
        f.write(blob + b"\n")
    print(f"wrote {bie1_path} ({len(blob)} b64 chars; password={PASSWORD!r})")

    # BIE2 fixture for the hardware-device refusal cell: the toolkit detects
    # BIE2 by magic BEFORE any crypto, so a magic-flipped blob suffices.
    raw = bytearray(base64.b64decode(blob))
    raw[3] = ord("2")  # BIE1 -> BIE2
    bie2 = base64.b64encode(bytes(raw))
    bie2_path = os.path.join(FIXTURES, "electrum-bie2-storage.txt")
    with open(bie2_path, "wb") as f:
        f.write(bie2 + b"\n")
    print(f"wrote {bie2_path} (BIE2 magic; refusal fixture)")


if __name__ == "__main__":
    main()
