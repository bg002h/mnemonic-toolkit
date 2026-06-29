# `ms repair` {#ms-repair}

\index{ms repair}BCH error-correct a single `ms1` string (ms-cli
v0.4.0+). Wraps `ms_codec::decode_with_correction` and renders a
single-chunk repair report. The string is corrected within the
`BCH(93,80,8)` `t=4` capacity (up to four substitution errors).
`ms repair` is the per-codec sibling of the toolkit's
`mnemonic repair`; the two surfaces share the same `RepairJson`
envelope byte-exact.

Single-HRP context: there is no `--hrp` flag and no variadic
positional — an `ms1` is single-chunk by codex32 specification (HRP
`ms` is always a single-string `BCH(93,80,8)` card).

> **GUI form:** see [GUI Forms › ms › repair](#gui-form-ms-repair).

## Outline {#ms-repair-outline}

- [`--ms1`](#ms-repair-ms1) — the `ms1` string to repair (required; `-` reads stdin; secret-bearing)
- [`--json`](#ms-repair-json) — emit a single JSON envelope on stdout instead of the text report

## `--ms1` {#ms-repair-ms1}

The `ms1` string to attempt to repair via BCH error correction.
**Required.** A literal `-` reads the string from stdin (a single
line).

**Secret-bearing** — schema-`secret: true`. The to-be-repaired
`ms1` IS master-secret material: it is BCH-corrupted BIP-39
entropy, and the corrected output reconstructs the seed card. The
GUI renders this as a `SecretLineEdit` widget; a non-empty value
triggers the run-confirm modal at click-Run time. (This is a
deliberate GUI-side `secret: true` override — see
`FOLLOWUPS.md::ms-repair-ms1-not-secret-classified`.)

## `--json` {#ms-repair-json}

Boolean. Emit a single JSON envelope on stdout instead of the
text-form report; the envelope schema byte-matches
`mnemonic repair --json`'s `RepairJson` shape. Default off.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | input already valid; no correction applied; echoed unchanged |
| `5` | `REPAIR_APPLIED` — at least one substitution corrected; stdout = repair report + corrected string |
| `2` | unrepairable (> 4 substitution errors, or a structural `ms1` error before correction could run) |
| `1` | I/O error or other generic failure |

The exit-5 `REPAIR_APPLIED` code is uniform across all four CLIs
(`mnemonic` / `mk` / `ms` / `md`), so wrapper scripts can branch on
`exit == 5`.

## Worked example — one-character repair

:::danger
Examples use the canonical all-`abandon` test vector — a
**public** seed swept since 2017. Never engrave or fund any wallet
derived from it.
:::

1. **ms** tab; pick **Repair (BCH error correction)**.
2. [`--ms1`](#ms-repair-ms1) (masked): paste a valid canonical
   `ms1` with one character corrupted (here position 17 `q` → `z`):

   ```text
   ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f
   ```

3. Click **Run**. The run-confirm modal fires (secret-bearing
   `--ms1`); confirm to proceed.

The output panel renders the repair report with the corrected
`ms1` on the last line; exit code `5`. Because the corrected output
carries the `ms1` (BIP-39 entropy), the D9 secret-on-stdout
advisory fires on stderr — pipe the output to a file or an
encryption tool to avoid scrollback exposure.

## Refusals

| Trigger | Refusal |
|---|---|
| `--ms1` omitted | clap refusal: required argument not provided |
| More than 4 substitution errors | exit 2 (BCH-uncorrectable) |
| Structural `ms1` error before correction | exit 2 |
