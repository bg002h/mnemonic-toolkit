#!/usr/bin/env python3
"""Regenerate the BIP-129 §Encryption cross-implementation fixtures from the
Coinkite Python reference.

These fixtures pin the toolkit's BIP-129 encryption against an INDEPENDENT
implementation: each `.dat` is a hex `MAC || ciphertext` wire produced by
Coinkite's `bsms.encryption.encrypt()`, which the toolkit must decrypt
byte-for-byte. The fixtures are VENDORED (committed) so the default test
suite + CI need NO clone / pip / network — see this directory's README.md.

This script is developer-run only; it is NOT wired into any CI workflow.

Pinned Coinkite reference:
    repo: https://github.com/coinkite/bsms-bitcoin-secure-multisig-setup
    SHA:  c30abe3a6d9823b6a3003e89acd66b9f38e11f1c  (frozen 2023-01-24)
    dep:  pyaes  (pure-Python AES; `pip install pyaes` in a venv)

Recipe:
    git clone https://github.com/coinkite/bsms-bitcoin-secure-multisig-setup ck
    cd ck && git checkout c30abe3a6d9823b6a3003e89acd66b9f38e11f1c
    python3 -m venv .venv && .venv/bin/pip install pyaes
    .venv/bin/python /path/to/regen_coinkite_vectors.py --coinkite-root .

The generation is deterministic (IV = MAC[:16] = f(token, plaintext);
AES-CTR with a fixed key + IV + plaintext yields a fixed ciphertext), so a
correct run reproduces the committed `.dat` byte-for-byte.
"""

import argparse
import pathlib
import sys

# Cross-impl vectors: (output .dat name, token-hex, plaintext-fixture name).
# The plaintext is read as EXACT bytes (trailing newline preserved); the
# token file is written without a trailing newline.
VECTORS = [
    (
        "bsms-coinkite-xref-round2-2of3.dat",
        "00112233445566778899aabbccddeeff",  # 16-byte EXTENDED token
        "bsms-2line-multi-2of3.txt",
    ),
]

FIXTURE_DIR = (
    pathlib.Path(__file__).resolve().parent.parent / "fixtures" / "wallet_import"
)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--coinkite-root",
        required=True,
        help="path to a checkout of coinkite/bsms-bitcoin-secure-multisig-setup "
        "at the pinned SHA (must import `bsms.encryption`)",
    )
    args = ap.parse_args()

    sys.path.insert(0, args.coinkite_root)
    from bsms.encryption import decrypt, encrypt, key_derivation_function

    for dat_name, token_hex, plaintext_name in VECTORS:
        # EXACT bytes — do NOT strip; the trailing newline is part of the
        # signed/encrypted plaintext.
        plaintext = (FIXTURE_DIR / plaintext_name).read_text()
        key = key_derivation_function(token_hex)
        wire = encrypt(key, token_hex, plaintext)

        # Self-verify: re-decrypt our own output and assert byte-equality
        # with the plaintext BEFORE writing (catches any newline/encoding
        # divergence).
        roundtrip = decrypt(key, wire)
        if roundtrip != plaintext:
            raise SystemExit(
                f"FATAL: re-decrypt of generated wire for {dat_name} does not "
                f"match the plaintext {plaintext_name}; refusing to write a "
                f"divergent fixture"
            )

        (FIXTURE_DIR / dat_name).write_text(wire)
        # Token file: NO trailing newline (the toolkit strips, but keep it
        # canonical).
        (FIXTURE_DIR / dat_name.replace(".dat", "-token.hex")).write_text(token_hex)
        print(f"wrote {dat_name} ({len(wire)} hex chars) + token")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
