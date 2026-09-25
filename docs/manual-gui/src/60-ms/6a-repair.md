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

- [`--ms1`](#ms-repair-ms1) — the `ms1` string to repair (`-` reads stdin; secret-bearing; or use `--in`)
- [`--json`](#ms-repair-json) — emit a single JSON envelope on stdout instead of the text report
- [`--in`](#ms-repair-in) — read the `ms1` string from a file
- [`--out`](#ms-repair-out) — write the artifact to a file (owner-only `0600`, overwrites)

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

## `--in` {#ms-repair-in}

Path widget. Read the `ms1` string from FILE instead of the
positional (or `--ms1`). It is the private channel that frees stdin: a path on
argv is not secret, so a run that uses it needs no
`--allow-argv-secret` and shows no run-confirm modal. The GUI lets only
one input source through: the first filled source wins and the others
grey out, because the CLI refuses `--in` alongside them.

## `--out` {#ms-repair-out}

Path widget. Write the canonical artifact to FILE, **owner-only
(`0600`)** — the mode is set on the open file, so an existing `0644`
target is tightened too. It **overwrites** and truncates. Not to be
confused with [`ms gen-man --out`](#ms-gen-man), which takes a
directory.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | input already valid; no correction applied; echoed unchanged |
| `4` | **`VERIFY-ME` Candidate** (as of `ms-cli v0.14.0`) — at least one substitution corrected; stdout = repair report + corrected string; stderr carries a `repair: correction UNVERIFIED …` advisory. `ms repair` has no `--max-indel` flag, so exit `5` is unreachable from this binary. |
| `2` | unrepairable (> 4 substitution errors, or a structural `ms1` error before correction could run) |
| `1` | I/O error or other generic failure |

**No self-verification, no exit 5.** `ms1` encodes raw BIP-39 entropy
as a single codex32 string — a bearer secret with no cross-chunk hash,
no fingerprint, and no internal redundancy beyond the BCH checksum
itself. A bounded-distance (≤4-error) substitution correction is
provably the original string, but beyond that bound the correction can
still *succeed* while **aliasing to a DIFFERENT, valid seed** — and
unlike `mk1`/`md1`, there is nothing else to catch it. `ms repair`
therefore **always** demotes a touched correction to an exit-`4`
Candidate, with a stderr advisory recommending the user confirm the
derived address/xpub against a known-good copy before trusting it.
BIP-93 recommends confirming a corrected codex32 string before relying
on it; this demotion puts that recommendation into practice.

**A per-surface model, not a single uniform code.** Each of the four
m-format CLIs applies the same principled rule (refined by Cycle E +
Cycle F): exit-`5` `REPAIR_APPLIED` means a correction is **verified
now** or **verifiable-by-reassembly later** (`mk1`/`md1`'s cross-chunk
structure); exit-`4` `VERIFY-ME` means a substitution correction spent
the checksum's error-detection budget with **no self-oracle** —
`ms1`'s case, always. Wrapper scripts must NOT branch on a single
`exit == 5` constant across the four binaries; see [`mnemonic
repair`](#mnemonic-repair) and [`mk repair`](#mk-repair) for the other
two surfaces' own exit-code tables.

**Version scoping.** This demotion ships in **`ms-cli v0.14.0`**;
before that release `ms repair` reported ANY substitution correction
as a confident exit-`5` `REPAIR_APPLIED`, with no UNVERIFIED advisory.
The pinned `ms` 0.19.1 carries it: the worked example below exits
`4`.

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
`ms1` on the last line; exit code `4` (`VERIFY-ME` Candidate, as of
`ms-cli v0.14.0` — see Exit codes above). A
`repair: correction UNVERIFIED — a corrected seed card cannot be
self-verified; confirm the derived address/xpub against a known-good
copy before use; BIP-93 recommends confirming a corrected codex32
string` advisory fires on stderr. Because the corrected output also
carries the `ms1` (BIP-39 entropy), the D9 secret-on-stdout advisory
fires alongside it — pipe the output to a file or an encryption tool
to avoid scrollback exposure.

## Refusals

| Trigger | Refusal |
|---|---|
| `--ms1` omitted | clap refusal: required argument not provided |
| More than 4 substitution errors | exit 2 (BCH-uncorrectable) |
| Structural `ms1` error before correction | exit 2 |
