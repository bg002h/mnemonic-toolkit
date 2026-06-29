# `mnemonic seed-xor-split` {#mnemonic-seed-xor-split}

Coldcard-compatible BIP-39 ↔ BIP-39 all-or-nothing XOR splitter
(per Coldcard's `xor_seed.py`). Splits a master BIP-39 phrase into
N XOR shares — every share required to recover the master. Unlike
SLIP-39 (which is K-of-N threshold), seed-xor is N-of-N: missing
one share leaves the master irrecoverable.

Use case: distribute N shares to N independent custodians; any
share lost or destroyed renders the master unrecoverable. Trade-off
versus SLIP-39: simpler protocol (XOR), Coldcard hardware-wallet
interop, but no fault tolerance.

:::danger
Both the master phrase AND the emitted shares are secret-class
material. The §14 Defense 2 cold-node operational warning applies.
The output panel renders all N share phrases on stdout — every
share that appears on screen during the run is exposed to the same
screen-observation threats as the master.
:::

> **GUI form:** see [GUI Forms › mnemonic › seed-xor-split](#gui-form-mnemonic-seed-xor-split).

## Outline {#mnemonic-seed-xor-split-outline}

- [`--from`](#mnemonic-seed-xor-split-from) — master BIP-39 phrase (required; `phrase=<value-or->`)
- [`--shares`](#mnemonic-seed-xor-split-shares) — number of XOR shares to emit (required, ≥ 2)
- [`--language`](#mnemonic-seed-xor-split-language) — BIP-39 wordlist (default `english`)
- [`--deterministic-from-master`](#mnemonic-seed-xor-split-deterministic-from-master) — Coldcard SHA256d-deterministic share generation
- [`--json-out`](#mnemonic-seed-xor-split-json-out) — write JSON envelope to PATH (side-effect)

## `--from` {#mnemonic-seed-xor-split-from}

The master BIP-39 phrase. NodeValueComposite with one valid node:
`phrase` (no other node accepted; the toolkit refuses
`entropy=` / `xprv=` / etc. with `seed-xor only accepts phrase=<value> or phrase=-`).
Required. Schema-`secret: false` but value-dependent (the GUI's
NodeValueComposite widget uses a `SecretLineEdit` for the value
field because `phrase` is in the secret-class set).

Suffix `=-` reads the value from stdin. The phrase must be 12 /
15 / 18 / 21 / 24 words; refused with `seed-xor split: phrase
must be 12/15/18/21/24 words; got <N>`.

### `phrase` {#mnemonic-seed-xor-split-from-phrase}

The only valid node for `--from`. Value is a 12 / 15 / 18 / 21 /
24 word BIP-39 mnemonic. Secret-bearing.

## `--shares` {#mnemonic-seed-xor-split-shares}

Number of XOR shares to emit. Number widget; range 2..255.
Required. Each share is itself a valid BIP-39 phrase of the same
word count as the master. Pasting all N shares back into
`mnemonic seed-xor-combine` reproduces the master exactly.

## `--language` {#mnemonic-seed-xor-split-language}

BIP-39 wordlist used to parse the input phrase AND to encode the
output shares. Default `english`. Same 10 values as
[`mnemonic bundle --language`](#mnemonic-bundle-language).

### Outline {#mnemonic-seed-xor-split-language-outline}

- [`english`](#mnemonic-seed-xor-split-language-english)
- [`simplifiedchinese`](#mnemonic-seed-xor-split-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-seed-xor-split-language-traditionalchinese)
- [`czech`](#mnemonic-seed-xor-split-language-czech)
- [`french`](#mnemonic-seed-xor-split-language-french)
- [`italian`](#mnemonic-seed-xor-split-language-italian)
- [`japanese`](#mnemonic-seed-xor-split-language-japanese)
- [`korean`](#mnemonic-seed-xor-split-language-korean)
- [`portuguese`](#mnemonic-seed-xor-split-language-portuguese)
- [`spanish`](#mnemonic-seed-xor-split-language-spanish)

### `english` {#mnemonic-seed-xor-split-language-english}

See [`mnemonic bundle --language english`](#mnemonic-bundle-language-english).

### `simplifiedchinese` {#mnemonic-seed-xor-split-language-simplifiedchinese}

See [`mnemonic bundle --language simplifiedchinese`](#mnemonic-bundle-language-simplifiedchinese).

### `traditionalchinese` {#mnemonic-seed-xor-split-language-traditionalchinese}

See [`mnemonic bundle --language traditionalchinese`](#mnemonic-bundle-language-traditionalchinese).

### `czech` {#mnemonic-seed-xor-split-language-czech}

See [`mnemonic bundle --language czech`](#mnemonic-bundle-language-czech).

### `french` {#mnemonic-seed-xor-split-language-french}

See [`mnemonic bundle --language french`](#mnemonic-bundle-language-french).

### `italian` {#mnemonic-seed-xor-split-language-italian}

See [`mnemonic bundle --language italian`](#mnemonic-bundle-language-italian).

### `japanese` {#mnemonic-seed-xor-split-language-japanese}

See [`mnemonic bundle --language japanese`](#mnemonic-bundle-language-japanese).

### `korean` {#mnemonic-seed-xor-split-language-korean}

See [`mnemonic bundle --language korean`](#mnemonic-bundle-language-korean).

### `portuguese` {#mnemonic-seed-xor-split-language-portuguese}

See [`mnemonic bundle --language portuguese`](#mnemonic-bundle-language-portuguese).

### `spanish` {#mnemonic-seed-xor-split-language-spanish}

See [`mnemonic bundle --language spanish`](#mnemonic-bundle-language-spanish).

## `--deterministic-from-master` {#mnemonic-seed-xor-split-deterministic-from-master}

Boolean. When set, share generation uses Coldcard's
SHA256d-deterministic algorithm (per Coldcard's `xor_seed.py`)
instead of fresh randomness. Use this when round-trip
interoperability with Coldcard is required (a Coldcard-generated
share set will combine identically with this CLI's
`--deterministic-from-master` output).

When unset (default), shares use fresh randomness from the OS RNG
— each invocation produces a different share set even from the
same master.

## `--json-out` {#mnemonic-seed-xor-split-json-out}

Optional. Writes a versioned JSON envelope to PATH in addition to
plain shares on stdout. Schema includes `schema_version`,
`master_word_count`, `share_count`, `language`,
`deterministic_from_master`, and `shares[]` (the share-phrase
array).

The GUI renders this as a Path widget. World-readable-path advisory
on Unix systems with default umask, same wording as
[`mnemonic final-word --json-out`](#mnemonic-final-word-json-out).

## Worked example — split into 3 XOR shares

1. **mnemonic** tab; pick **Seed XOR Split (Coldcard
   all-or-nothing splitter)**.
2. `--from`: pick `phrase`; paste the canonical master
   `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`.
3. `--shares`: `3`.
4. Leave `--language` at default. Leave
   `--deterministic-from-master` unchecked.
5. Click **Run**. The run-confirm modal appears. Click **Run** in
   the modal.

The output panel renders 3 share phrases on stdout, one per line.
Each share is itself a valid 12-word BIP-39 phrase; the XOR of all
3 share-entropies equals the master entropy. With
`--deterministic-from-master` unchecked the share contents are
non-deterministic (each invocation emits different shares).

## Refusals

| Trigger | Refusal |
|---|---|
| `--from <node>=` other than `phrase=` | `seed-xor only accepts phrase=<value> or phrase=-` (per `cmd/seed_xor.rs`) |
| Master phrase word count not in {12, 15, 18, 21, 24} | `seed-xor split: phrase must be 12/15/18/21/24 words; got <N>` |
| Phrase with words not in the selected `--language` wordlist | BIP-39 parse error |
| `--shares < 2` | clap-level range validation |

## Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--from phrase=<value>` | `warning: secret material on argv (--from phrase=) — pipe via --from phrase=- to avoid /proc/$PID/cmdline exposure` |
| Stdout is a TTY AND share count > 0 | byte-exact per `cmd/seed_xor.rs:193-198`: `warning: Seed XOR shares on stdout — each of the N=<N> lines is independently a complete BIP-39 phrase; ALL N shares are required to reconstruct the master; distribute them to N separate locations; do not paste this output into a single untrusted tool. Substitution of a wrong-but-valid-BIP-39 share is undetectable by Seed XOR — verify the recovered wallet's derived address before trusting it.` |
| `--deterministic-from-master` AND master word count is 15 or 21 | byte-exact per `cmd/seed_xor.rs:183-190`: `warning: --deterministic-from-master with <N>-word input is toolkit-only — Coldcard's xor_seed.py natively supports 12/18/24 only; resulting shares will NOT round-trip a Coldcard device. For Coldcard interop, use 12/18/24-word input.` |
| `--json-out PATH` with world-readable file | world-readable-path advisory (same wording as final-word) |

The TTY-vs-pipe advisory does NOT fire when invoked via the GUI
(the GUI pipes stdout). The share-secrecy concern remains
regardless — the output panel's contents are secret material
until used or discarded.
