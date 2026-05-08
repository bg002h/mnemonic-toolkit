# BIP-39 passphrase vs BIP-38 passphrase

Two unrelated standards both call their additional secret a
"passphrase," and they do *very* different things. Confusing them
costs funds. Toolkit v0.8 made the distinction explicit by splitting
them into two separate CLI flags.

## The two standards

| Standard | What the passphrase protects | KDF | Composes with |
|---|---|---|---|
| **BIP-39** | The seed-derivation step | PBKDF2-HMAC-SHA-512, 2048 iterations | a 12-/24-word mnemonic |
| **BIP-38** | A single-key WIF | Scrypt (memory-hard) | one privkey at a time, not a wallet |

A BIP-39 passphrase becomes part of the wallet's identity: with one
phrase + one passphrase you have one wallet; with the same phrase +
a different passphrase you have a different wallet. There's no way
to test a passphrase without trying it.

A BIP-38 passphrase is encryption-at-rest for one private key. The
encrypted form (`6P...`) is decryptable only with the right
passphrase; the wrong passphrase produces a wrong key (silent
failure, then funds going to nowhere on first send).

## The toolkit v0.8 BREAKING change

In toolkit v0.7, `mnemonic convert --passphrase` covered both
purposes. On a `(phrase, ms1)` edge it fed BIP-39 PBKDF2; on a
`(wif, bip38)` edge it fed BIP-38 Scrypt. On a *composite* edge
like `(phrase, bip38)` (encrypt the BIP-39-derived WIF with BIP-38
Scrypt), the same flag was reused for both KDFs — operationally
ambiguous.

v0.8 split the two:

```sh
mnemonic convert \
  --from phrase="…" \
  --to bip38 \
  --passphrase "<BIP-39 passphrase>" \
  --bip38-passphrase "<BIP-38 passphrase>"
```

The two arguments are now independent. On composite edges, both
passphrases must be supplied (or both can be empty). On direct
`(wif, bip38)` and `(bip38, wif)` edges, `--bip38-passphrase` falls
back to `--passphrase` if the dedicated flag is absent.

| Edge | `--passphrase` | `--bip38-passphrase` |
|---|---|---|
| `(phrase, ms1)` | BIP-39 | unused |
| `(wif, bip38)` direct | falls through to BIP-38 if `--bip38-passphrase` unset | BIP-38 |
| `(phrase, bip38)` composite | BIP-39 | BIP-38 (independent; no fallback) |
| `(entropy, bip38)` composite | unused (no BIP-39 step) | BIP-38 (defaults to `""` if unset; BREAKING) |

## NULL-byte passphrase edge case

BIP-38 V3 (the encrypted-private-key spec) explicitly allows
passphrases containing `U+0000` (NULL). POSIX argv cannot carry
NULL bytes (the C runtime's argv parser terminates each argument
at NULL). So passing a NULL-byte passphrase via `--passphrase` or
`--bip38-passphrase` is impossible.

v0.8 added `--passphrase-stdin` to close this gap:

```sh
printf '\x00secret' | mnemonic convert \
  --from wif="…" \
  --to bip38 \
  --passphrase-stdin
```

The flag tells the binary to read `--passphrase`'s value from stdin
(raw, no newline-stripping, no UTF-8 normalisation), preserving
NULL bytes. Mutually exclusive with `--passphrase`.

This was driven by the v0.7.1 BIP-38 test-vector audit cycle, which
exposed a third-party test vector (BIP-38 V3 NULL-byte passphrase)
that the toolkit could not reproduce because of the argv NULL
limitation. v0.8 fixed it.

## Practical recommendations

- **Use a BIP-39 passphrase only if you understand its
  always-mandatory nature.** Forgetting the passphrase loses the
  wallet, and there's no way to check "is this the right one?"
  short of trying to derive an address you've previously used.
- **Use a BIP-38 passphrase for one-off-key paper wallets, never
  for HD wallet backup.** BIP-38 was designed for "import this one
  WIF into your wallet"; HD wallets and the m-format star are for
  multi-key setups.
- **If your software offers "extra word" or "13th word" prompts,
  it is offering BIP-39 passphrase mode.** This is *not* BIP-38.
  Knowing which standard you're invoking is non-negotiable for
  recovery.
