# `ms combine` {#ms-combine}

\index{ms combine}Recombine **K (or more)** BIP-93 codex32 shares
(produced by [`ms split`](#ms-split)) back into the original secret
(ms-cli v0.7.0+). Supply the distributed share strings as
positional arguments with distinct indices; the recovered secret is
emitted in the form selected by [`--to`](#ms-combine-to).

The recovered output **is private key material** — a
`PrivateKeyMaterial` stderr advisory is emitted and the GUI fires
its run-confirm modal on the secret-bearing share input. The
secret-carrying share at index `s` is NEVER a valid input here (it
is the unshared sentinel and is rejected). The toolkit front-end is
`mnemonic ms-shares combine`.

> **GUI form:** see [GUI Forms › ms › combine](#gui-form-ms-combine).

## Outline {#ms-combine-outline}

- [`--to`](#ms-combine-to) — output form for the recovered secret (`phrase`|`entropy`|`ms1`; default `phrase`)
- [`--json`](#ms-combine-json) — emit a single JSON object on stdout instead of text

## Positional `shares`

The distributed share strings to recombine. **Required**,
**repeating** (K or more, with distinct indices). **Secret-
equivalent** — schema-`secret: true` on the positional — so any
non-empty value triggers the run-confirm modal. The GUI presents
this as a repeating secret-input field. Fewer than K shares, a
repeated index, or the index-`s` secret share are each refused.

## `--to` {#ms-combine-to}

Output form for the recovered secret. Dropdown widget; 3 values,
default `phrase`.

### Outline {#ms-combine-to-outline}

- [`phrase`](#ms-combine-to-phrase)
- [`entropy`](#ms-combine-to-entropy)
- [`ms1`](#ms-combine-to-ms1)

### `phrase` {#ms-combine-to-phrase}

Recover the BIP-39 mnemonic (the default). For a `mnem` share-set
the phrase is rendered in the wordlist language carried on the
wire; for an `entr` share-set it is rendered in English.

### `entropy` {#ms-combine-to-entropy}

Recover the raw entropy as a hex string.

### `ms1` {#ms-combine-to-ms1}

Recover a single unshared `ms1` string (the codex32 single-string
form, threshold digit `0`).

## `--json` {#ms-combine-json}

Boolean. Emit a single JSON object on stdout instead of the text
form. Default off.

## Worked example — recombine 2-of-3

:::danger
Examples use the canonical all-`abandon` 16-byte zero-entropy test
vector — a **public** seed swept since 2017. Never engrave or fund
any wallet derived from it.
:::

1. **ms** tab; pick **Combine (recombine ≥K codex32 shares)**.
2. In the `shares` repeating field, paste any 2 of the 3 shares
   produced by the [`ms split`](#ms-split) 2-of-3 worked example.
3. Leave [`--to`](#ms-combine-to) at default `phrase` (or pick
   `ms1` to recover a single unshared string, or `entropy` for raw
   hex).
4. Click **Run**. The run-confirm modal fires (secret-equivalent
   share input); confirm to proceed.

The output panel emits the recovered secret in the chosen form and
a `PrivateKeyMaterial` stderr advisory.

## Refusals

| Trigger | Refusal |
|---|---|
| Fewer than K shares supplied | `threshold not passed` refusal |
| A repeated share index | `repeated index` refusal |
| The secret share at index `s` supplied | `secret share supplied to combine` refusal |
| A share string fails BIP-93 codex32 parse | exit 1 with `error: <codex32 parse error>` |
