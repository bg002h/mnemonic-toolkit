# `mnemonic word-card` {#mnemonic-word-card}

Re-encode a **public** `mk1` (xpub) or `md1` (descriptor) card as an
engravable **BIP-39 Word Card** — a list of dictionary words an engraver
can stamp into steel — or `--decode` a Word Card back into the original
`m*1` card. The encoder layers optional Reed–Solomon parity words for
substitution/erasure repair, and an optional cross-plate RAID array so a
lost plate can be reconstructed from the survivors. This subcommand ships
with toolkit v0.74.0 (the `wc-codec` engine). The GUI exposes it as a flat
**Word Card** form on the `mnemonic` tab.

The Word-Card workflow operates on **public** material only: an `mk1`
holds an xpub, an `md1` holds a wallet-policy template. The secret `ms1`
seed card is **excluded** by design — there is no secret-bearing field on
this form, so the run-confirm modal does not fire and the value editor is
a plain (unmasked) text field.

:::danger
The worked example below derives its `mk1` card from the canonical public
all-`abandon` test seed. That seed is **public** and every wallet derived
from it has been swept. A Word Card encodes a *public* card, so it carries
no spend capability — but **never engrave a demonstration card** onto steel
you intend to reuse, and never treat a test card as a real backup.
:::

> **GUI form:** see [GUI Forms › mnemonic › word-card](#gui-form-mnemonic-word-card).

## Outline {#mnemonic-word-card-outline}

- [`--from`](#mnemonic-word-card-from) — source `mk1` / `md1` card(s) to encode (repeating; PUBLIC material)
- [`--decode`](#mnemonic-word-card-decode) — decode a Word Card back to its `m*1` card
- [`--decode-plate`](#mnemonic-word-card-decode-plate) — one RAID plate's word list for a `--decode` reconstruction (repeating)
- [`--raid`](#mnemonic-word-card-raid) — RAID recovery tier (`0` solo / `1` / `2`)
- [`--parity-words`](#mnemonic-word-card-parity-words) — Reed–Solomon parity words to append
- [`--parity-pct`](#mnemonic-word-card-parity-pct) — Reed–Solomon parity as a percentage of the payload
- [`--integrity-bits`](#mnemonic-word-card-integrity-bits) — integrity-tag bit width (default 44)
- [`--json`](#mnemonic-word-card-json) — emit a JSON envelope instead of the text-form card

## `--from` {#mnemonic-word-card-from}

The source card(s) to encode into a Word Card. Each value is an `mk1`
xpub card or an `md1` descriptor card — **public** material, never a
secret. Text field, **repeating**: supply one `--from` per `mk1` / `md1`.
A multi-chunk card may be passed as all chunks joined OR as one `--from`
per chunk; chunks are auto-grouped by HRP. Use `-` to read one card per
line from stdin.

When [`--raid`](#mnemonic-word-card-raid) is `1` or `2`, supply the `n`
data cards as `n` repeated `--from` occurrences (RAID needs ≥ 2 `mk1`
data cards). The GUI renders this as a repeating row with a **+ Add from**
button — one card per row.

## `--decode` {#mnemonic-word-card-decode}

Boolean (checkbox). Switches the form into **decode** mode: reconstruct
the original `m*1` card from an engraved Word Card. In decode mode the
words come from the positional `<WORD>...` list (one BIP-39 word per
value, or `-` to read whitespace-separated words from stdin) for a single
solo card, or from repeated
[`--decode-plate`](#mnemonic-word-card-decode-plate) occurrences for a
RAID array. The positional single-card form and `--decode-plate` are
mutually exclusive.

## `--decode-plate` {#mnemonic-word-card-decode-plate}

Text field, **repeating**. One RAID plate's whitespace-separated word
list; each occurrence is one plate. Supply the surviving `≥ n` plates of
an `n + r` RAID array to reconstruct a lost data plate. Only meaningful
alongside [`--decode`](#mnemonic-word-card-decode); mutually exclusive
with the positional `<WORD>...` single-card form.

## `--raid` {#mnemonic-word-card-raid}

Number field; RAID recovery tier. `0` = no RAID (a single solo card;
default), `1` = one XOR recovery plate (RAID-5, survives any one lost
plate), `2` = two recovery plates (RAID-6, survives any two). RAID
requires ≥ 2 `mk1` data cards supplied via repeated
[`--from`](#mnemonic-word-card-from). Only tiers `0` / `1` / `2` are
surfaced.

## `--parity-words` {#mnemonic-word-card-parity-words}

Number field. The Reed–Solomon parity-word budget `m` appended to each
card: it corrects `⌊m/2⌋` word substitutions or fills `m` erasures.
Default `0` (error **detection** only, no repair). Mutually exclusive with
[`--parity-pct`](#mnemonic-word-card-parity-pct).

## `--parity-pct` {#mnemonic-word-card-parity-pct}

Number field. Reed–Solomon parity expressed as a **percentage** of the
data-symbol count `K` (`m = ceil(K * pct / 100)`) — an alternative way to
size the same budget as [`--parity-words`](#mnemonic-word-card-parity-words).
For example `25` requests roughly a 25 % redundancy budget. Mutually
exclusive with `--parity-words`.

## `--integrity-bits` {#mnemonic-word-card-integrity-bits}

Number field. The integrity-tag bit width `t` — a non-linear SHA-256
truncation that catches a Reed–Solomon mis-correction with probability
`≤ 2⁻ᵗ`. Default `44` (four words); minimum `33`.

## `--json` {#mnemonic-word-card-json}

Boolean flag. When set, emits a single JSON envelope on stdout instead of
the human-readable text-form card. The envelope carries the encoded word
list (or the decoded `m*1` card), the parity / RAID parameters, and the
integrity tag.

## Worked example — encode an `mk1` card as a Word Card

1. Switch to the **mnemonic** tab; pick **Word Card** in the subcommand
   selector.
2. `--from`: paste an `mk1` xpub card (for example one emitted by
   [`mnemonic bundle`](#mnemonic-bundle) for the canonical test seed).
   The field is a plain text row — `mk1` is public, so it is **not**
   masked.
3. Leave `--decode` unchecked (encode is the default direction).
4. Optionally set `--parity-words` (say `4`) for substitution repair, and
   leave `--raid` at `0` for a single solo card.
5. Click **Run**. Because no field is secret-bearing, **no run-confirm
   modal appears** — the subprocess spawns directly.

The output panel renders the engravable BIP-39 word list (plus the parity
and integrity tail). To go the other way, tick `--decode`, paste the
engraved words into the positional `<WORD>...` list, and **Run** to
recover the original `mk1`.

## Refusals

| Trigger | Refusal |
|---|---|
| `--from` given an `ms1` seed card | the secret `ms1` card is out of scope; Word Card encodes public `mk1` / `md1` only |
| `--raid 1` or `--raid 2` with fewer than 2 `mk1` data cards | RAID needs ≥ 2 data cards (one `--from` each) |
| `--parity-words` together with `--parity-pct` | the two parity-budget flags are mutually exclusive |
| `--decode-plate` together with positional `<WORD>...` | the RAID-plate and single-card decode forms are mutually exclusive |
| `--integrity-bits` below `33` | below the minimum integrity-tag width |

\index{mnemonic word-card}
