# BIP-129 cross-implementation fixtures (Coinkite Python reference)

This directory holds the regeneration tooling for the **vendored**
cross-implementation fixtures that pin the toolkit's BIP-129 §Encryption
implementation against the independent [Coinkite Python
reference](https://github.com/coinkite/bsms-bitcoin-secure-multisig-setup).

## What the fixtures prove

Each `bsms-coinkite-xref-*.dat` under
`../fixtures/wallet_import/` is a hex `MAC || ciphertext` wire produced by
Coinkite's `bsms.encryption.encrypt()` — an implementation written
independently of this toolkit. The integration tests in
`../cli_import_wallet_bsms_encrypted.rs` decrypt these wires with the
toolkit and assert the recovered plaintext byte-equals the source plaintext
(and that the CLI import produces the same descriptor as the plaintext
import). Agreement between two independent implementations on the same
ciphertext is the cross-impl guarantee.

Combined with `bsms-encrypted-standard-tv3.dat` (a Coinkite-generated TV-3
Round-1 wire, STANDARD 8-byte token; vendored since Cycle 7a), both
directions + both token widths are covered:

| fixture | token width | record kind |
|---|---|---|
| `bsms-encrypted-standard-tv3.dat` | STANDARD (8 B) | Round-1 KEY |
| `bsms-coinkite-xref-round2-2of3.dat` | EXTENDED (16 B) | Round-2 descriptor |

## Why vendored (not a live CI clone)

The fixtures are committed so the default `cargo test` + CI need **no**
clone, **no** `pip`, **no** network. The cross-impl agreement is pinned at
generation time. This is a deliberate scope choice (the originating
FOLLOWUP `bsms-encryption-cross-impl-coinkite-python-smoke` also sketched a
live-CI-gated smoke; that was waived — see the FOLLOWUP closure note —
because the Coinkite repo is frozen (last push 2023-01-24), the toolkit
crypto is already byte-exact against BIP-129 TV-3, and a live external-clone
+ `pip` CI surface adds fragility for marginal drift-detection value).

## Pinned reference

- repo: `https://github.com/coinkite/bsms-bitcoin-secure-multisig-setup`
- SHA: `c30abe3a6d9823b6a3003e89acd66b9f38e11f1c` (frozen 2023-01-24)
- dep: `pyaes` (pure-Python AES)

## Regenerating (developer-only)

```sh
git clone https://github.com/coinkite/bsms-bitcoin-secure-multisig-setup ck
cd ck && git checkout c30abe3a6d9823b6a3003e89acd66b9f38e11f1c
python3 -m venv .venv && .venv/bin/pip install pyaes
.venv/bin/python /path/to/this/repo/crates/mnemonic-toolkit/tests/external/regen_coinkite_vectors.py \
    --coinkite-root .
```

The generation is deterministic (IV = first 16 bytes of the MAC, which is a
pure function of token + plaintext; AES-CTR with a fixed key/IV/plaintext
yields a fixed ciphertext), so a correct run reproduces the committed
`.dat` byte-for-byte. The script self-verifies (re-decrypts its own output
and asserts byte-equality with the plaintext) before writing.

If a source plaintext fixture (e.g. `bsms-2line-multi-2of3.txt`) ever
changes, rerun the script to refresh the corresponding `.dat`.
