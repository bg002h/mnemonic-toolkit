# `mnemonic repair` {#mnemonic-repair}

BCH error-correct a corrupted m-format card (`ms1` / `mk1` / `md1`).
All three formats share the BIP-93 codex32 BCH code family — regular
`BCH(93,80,8)` for short data-parts and long `BCH(108,93,8)` for the
xpub-bearing first chunk of a typical `mk1`. Both codes correct up to
four substitution errors per chunk (singleton bound `t=4`). The GUI
exposes the toolkit's `repair` subcommand as a flat **Repair** form on
the `mnemonic` tab. At least one of `--ms1` / `--mk1` / `--md1` is
required; the three may be combined (one HRP per card).

Use cases: recover a corroded engraving (one or two letters unreadable),
salvage a hand-copied card with a single typo, or sanity-check a freshly
engraved card against its source bundle before committing to steel.

:::danger
The worked example repairs a deliberately-corrupted zero-entropy `ms1`
card. **Never engrave or fund** any wallet recovered from a
demonstration card. A corrected `ms1` is master key material; the
`--ms1` field is secret-bearing. The run-confirm modal redacts the
secret-bearing `--ms1` argv token as a fixed `••••` sentinel (see [§14
Defense 2](#secret-handling)). `mk1` and `md1` carry no secret material.
:::

> **GUI form:** see [GUI Forms › mnemonic › repair](#gui-form-mnemonic-repair).

**At-least-one input (not a conjunction).** The `(required)` markers on `--ms1` / `--mk1` / `--md1` in the GUI form linked above are conditional-sourced: the form marks all three required only until you fill *any one* of them. Supply at least one card to run; you need not provide all three.

## Outline {#mnemonic-repair-outline}

- [`--ms1`](#mnemonic-repair-ms1) — single `ms1` chunk to repair (`-` reads from stdin)
- [`--mk1`](#mnemonic-repair-mk1) — one or more `mk1` chunks (repeating; `-` reads from stdin)
- [`--md1`](#mnemonic-repair-md1) — one or more `md1` chunks (repeating; `-` reads from stdin)
- [`--json`](#mnemonic-repair-json) — emit a single JSON envelope instead of the text repair report
- [`--max-indel`](#mnemonic-repair-max-indel) — insert/delete recovery budget for length-mismatch chunks (0..=4; default 0)
- [`--max-subst`](#mnemonic-repair-max-subst) — substitution budget alongside the indels (0..=4; default 0)

## `--ms1` {#mnemonic-repair-ms1}

Text field. A single `ms1` chunk to repair. Use `-` to read one chunk
from stdin. Combinable with `--mk1` / `--md1` (at least one card across
the three flags is required, per the toolkit's D35 rule). The GUI
renders the value editor as a masked `SecretLineEdit` because an `ms1`
card is secret-bearing — note the argv-leakage advisory fires for
`--ms1` even under `--max-indel` relaxation, where the corrupted value
no longer HRP-classifies.

## `--mk1` {#mnemonic-repair-mk1}

Text field, repeating. One or more `mk1` chunks to repair, rendered as a
repeating row with a **+ Add mk1** button — one chunk per row. Use `-`
on a single occurrence to read chunks from stdin (one per line).
Combinable with `--ms1` / `--md1`. `mk1` is public; the value editors
are not masked.

## `--md1` {#mnemonic-repair-md1}

Text field, repeating. One or more `md1` chunks to repair, rendered as a
repeating row with a **+ Add md1** button. Use `-` on a single
occurrence to read chunks from stdin (one per line). Combinable with
`--ms1` / `--mk1`. `md1` is public; the value editors are not masked.

For multi-chunk inputs the repair is atomic per card: if ANY chunk
fails (e.g. > 4 errors) the whole call fails with the offending
`chunk_index` named, rather than returning a half-fixed card that could
mislead you into committing it.

## `--json` {#mnemonic-repair-json}

Boolean flag. When set, emits a single JSON envelope on stdout
(`schema_version: "1"`) instead of the text-form repair report.

## `--max-indel` {#mnemonic-repair-max-indel}

Number widget (0..=4; default 0). The maximum insert/delete (indel)
distance to search when a chunk failed normal BCH repair because a
character was added (too long) or dropped (too short) during
transcription. `0` disables indel recovery. Applies to `ms1` / `mk1` /
`md1`. A unique indel recovery is reported as `REPAIR_APPLIED` (exit 5).

## `--max-subst` {#mnemonic-repair-max-subst}

Number widget (0..=4; default 0). Also tolerate up to E substitution
(wrong-but-in-place) errors alongside the indels. `0` is pure indel.
A recovery that consumed a substitution is printed as a **VERIFY-ME**
candidate (exit 4) rather than a confident correction — verify each
candidate against an independent source before trusting it.

## Exit codes

The GUI surfaces the toolkit exit code in the output panel:

| Code | Meaning |
|---|---|
| `0` | all chunks already valid (input echoed unchanged) |
| `5` | at least one chunk corrected (incl. a unique `--max-indel` recovery) |
| `4` | ambiguous (multiple candidates) **or** a candidate required ≥1 substitution — verify each |
| `2` | unrepairable (too many errors, HRP mismatch, reserved-length, or `--max-indel` exhausted) |
| `1` | I/O or other generic failure |

## Worked example — repair a single-character `ms1` corruption

1. Switch to the **mnemonic** tab; pick **Repair** in the subcommand
   selector.
2. Paste a corrupted `ms1` chunk (position 17 `q` → `z`) into the
   `--ms1` masked field:

   ```text
   ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f
   ```

3. Leave `--max-indel` and `--max-subst` at `0` (a single substitution
   is within the base BCH correction budget).
4. The `Preview:` line resembles:

   ```text
   mnemonic repair --ms1 ••••
   ```

5. Click **Run**; redact-confirm in the modal.

The output panel renders the repair report on stdout — comment lines
describe the fix and the corrected chunk is on the last line — with exit
code `5`. A `warning: stdout carries private key material (can spend) …`
advisory fires on stderr because the corrected `ms1` is secret-bearing.

\index{mnemonic repair}
