# `mnemonic seed-xor-combine` {#mnemonic-seed-xor-combine}

The reconstruction companion to
[`mnemonic seed-xor-split`](#mnemonic-seed-xor-split). Combines
N seed-XOR shares back into the master BIP-39 phrase. All N shares
are required (this is N-of-N XOR — losing any one share renders
the master irrecoverable). Coldcard-compatible: shares produced by
Coldcard's `xor_seed.py` combine identically here.

:::danger
The shares pasted into this form AND the reconstructed master
phrase are all secret-class material. The §14 Defense 2 cold-node
operational warning applies. The reconstructed master appears in
the output panel on stdout — every viewing of the master is
exposed to the same screen-observation threats as direct
phrase entry.
:::

## Outline {#mnemonic-seed-xor-combine-outline}

- [`--share`](#mnemonic-seed-xor-combine-share) — share BIP-39 phrase (required, repeating; at most one stdin)
- [`--shares`](#mnemonic-seed-xor-combine-shares) — asserted share count (required; runtime check)
- [`--language`](#mnemonic-seed-xor-combine-language) — BIP-39 wordlist (default `english`)
- [`--json-out`](#mnemonic-seed-xor-combine-json-out) — write JSON envelope to PATH (side-effect)

## `--share` {#mnemonic-seed-xor-combine-share}

A share BIP-39 phrase. NodeValueComposite with one valid node:
`phrase`. Required. Repeating — pass one occurrence per share.
Schema-`secret: true` (the *flag* is secret-class regardless of
the value). At most ONE share may use `phrase=-` (stdin); a
second stdin-form share is refused (single-stdin-per-invocation).

The GUI renders this as a multi-row NodeValueComposite repeating
widget — one row per share, each with a Dropdown (locked to
`phrase`) and a `SecretLineEdit` value field. Add and remove rows
with the per-row controls.

### `phrase` {#mnemonic-seed-xor-combine-share-phrase}

The only valid node for `--share`. Each share's value is a BIP-39
mnemonic of the same word count as the master that was split.
Secret-bearing.

## `--shares` {#mnemonic-seed-xor-combine-shares}

The asserted share count. Number widget; range 2..255. Required.
The handler-side runtime check requires the actual `--share`
occurrence count to equal `--shares`; mismatch is refused.

This redundancy is intentional: it lets the user explicitly
declare how many shares they're providing, catching off-by-one
input errors at the form-validation boundary rather than at
silent reconstruction time.

## `--language` {#mnemonic-seed-xor-combine-language}

BIP-39 wordlist used to parse share phrases AND to encode the
reconstructed master. Default `english`. Same 10 values as
[`mnemonic bundle --language`](#mnemonic-bundle-language).

### Outline {#mnemonic-seed-xor-combine-language-outline}

- [`english`](#mnemonic-seed-xor-combine-language-english)
- [`simplifiedchinese`](#mnemonic-seed-xor-combine-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-seed-xor-combine-language-traditionalchinese)
- [`czech`](#mnemonic-seed-xor-combine-language-czech)
- [`french`](#mnemonic-seed-xor-combine-language-french)
- [`italian`](#mnemonic-seed-xor-combine-language-italian)
- [`japanese`](#mnemonic-seed-xor-combine-language-japanese)
- [`korean`](#mnemonic-seed-xor-combine-language-korean)
- [`portuguese`](#mnemonic-seed-xor-combine-language-portuguese)
- [`spanish`](#mnemonic-seed-xor-combine-language-spanish)

### `english` {#mnemonic-seed-xor-combine-language-english}

See [`mnemonic bundle --language english`](#mnemonic-bundle-language-english).

### `simplifiedchinese` {#mnemonic-seed-xor-combine-language-simplifiedchinese}

See [`mnemonic bundle --language simplifiedchinese`](#mnemonic-bundle-language-simplifiedchinese).

### `traditionalchinese` {#mnemonic-seed-xor-combine-language-traditionalchinese}

See [`mnemonic bundle --language traditionalchinese`](#mnemonic-bundle-language-traditionalchinese).

### `czech` {#mnemonic-seed-xor-combine-language-czech}

See [`mnemonic bundle --language czech`](#mnemonic-bundle-language-czech).

### `french` {#mnemonic-seed-xor-combine-language-french}

See [`mnemonic bundle --language french`](#mnemonic-bundle-language-french).

### `italian` {#mnemonic-seed-xor-combine-language-italian}

See [`mnemonic bundle --language italian`](#mnemonic-bundle-language-italian).

### `japanese` {#mnemonic-seed-xor-combine-language-japanese}

See [`mnemonic bundle --language japanese`](#mnemonic-bundle-language-japanese).

### `korean` {#mnemonic-seed-xor-combine-language-korean}

See [`mnemonic bundle --language korean`](#mnemonic-bundle-language-korean).

### `portuguese` {#mnemonic-seed-xor-combine-language-portuguese}

See [`mnemonic bundle --language portuguese`](#mnemonic-bundle-language-portuguese).

### `spanish` {#mnemonic-seed-xor-combine-language-spanish}

See [`mnemonic bundle --language spanish`](#mnemonic-bundle-language-spanish).

## `--json-out` {#mnemonic-seed-xor-combine-json-out}

Optional. Writes a versioned JSON envelope with the reconstructed
master to PATH (in addition to plain phrase on stdout). Same
schema-shape conventions as the other `--json-out` flags in this
chapter. World-readable-path advisory on Unix.

## Worked example — combine 3 shares back to master

1. **mnemonic** tab; pick **Seed XOR Combine (reconstruct from
   XOR shares)**.
2. Add 3 `--share` rows; paste the 3 share phrases produced by
   `seed-xor-split` (each is `phrase=<words>` form).
3. `--shares`: `3` (must match the row count).
4. Leave `--language` at default.
5. Click **Run**. The run-confirm modal appears (each share row
   is secret-class). Click **Run** in the modal.

The output panel renders the reconstructed master phrase on
stdout. For shares produced from the canonical master, the output
is exactly:

```text
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

If even one share is wrong (typo, swapped words, missing share),
the reconstruction either fails the BIP-39 checksum check (refused
with a checksum error) or produces a different — and unrelated —
valid 12-word phrase. There is no per-share validity check at this
layer because XOR is malleable: any random 12-word phrase XORs
with another random 12-word phrase to a third random 12-word
phrase.

## Refusals

| Trigger | Refusal |
|---|---|
| `--share` count != `--shares` | handler-side runtime mismatch refusal |
| More than one `--share phrase=-` | single-stdin-per-invocation refusal |
| Reconstructed phrase fails BIP-39 checksum | BIP-39 parse error |
| Any `--share phrase=<value>` with non-BIP-39 word | BIP-39 parse error at the share row |

## Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--share phrase=<value>` | `warning: secret material on argv (--share phrase=) — pipe via --share phrase=- to avoid /proc/$PID/cmdline exposure` (one per inline share) |
| Stdout is a TTY AND reconstruction succeeded | byte-exact per `cmd/seed_xor.rs:317-321`: `warning: combined phrase is secret material — Seed XOR has no authentication tag; verify the recovered wallet's expected derived address before trusting; if a share was substituted with a wrong-but-valid one, the result will validate but derive the wrong wallet` |
