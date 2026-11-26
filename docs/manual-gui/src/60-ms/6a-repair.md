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
| `0` | input already valid; no correction applied; echoed unchanged. Also a clean distributed share of a K-of-N set, which stderr names (`repair: this is share '<i>' of a <k>-of-N set, not a single card; recover the secret with `ms combine` and <k> shares`) |
| `20` | `16 + 4`: **`VERIFY-ME` Candidate** — at least one substitution corrected; stdout = repair report + corrected string; stderr carries a `repair: correction UNVERIFIED …` advisory. A corrected distributed share is also `20`; it is printed, written to `--out`, and `--json` adds `"share": {"threshold", "index"}` (the GUI keys on that field). `ms repair` has no `--max-indel` flag, so a plain `16` is unreachable from this binary. |
| `22` | `16 + 4 + 2`: the corrected string is neither a valid card nor a distributed share (for example the secret share at index `s`); it is printed, and `--out` is not written. Under `--json` the envelope adds `"not_a_card": {"kind": …, "message": …}` after `verdict` (still `"candidate"`) |
| `2` | unrepairable (> 4 substitution errors, or a structural `ms1` error before correction could run), a clean string that is neither a card nor a share, or empty input (`error: the input is empty (stdin)`) |
| `8` | refused: the `ms1` on argv, or `--out` naming stdout |
| `32` | a file or stream could not be read or written |
| `64` | the command line is wrong; nothing ran |

**No self-verification: always `20`.** `ms1` encodes raw BIP-39 entropy
as a single codex32 string — a bearer secret with no cross-chunk hash,
no fingerprint, and no internal redundancy beyond the BCH checksum
itself. A bounded-distance (≤4-error) substitution correction is
provably the original string, but beyond that bound the correction can
still *succeed* while **aliasing to a DIFFERENT, valid seed** — and
unlike `mk1`/`md1`, there is nothing else to catch it. So every
correction `ms repair` applies exits `20`: bit 16 (a correction was
applied) plus bit 4 (it cannot be verified), with a stderr advisory
recommending the user confirm the derived address/xpub against a
known-good copy before trusting it. BIP-93 recommends confirming a
corrected codex32 string before relying on it; this puts that
recommendation into practice.

**One rule on every CLI.** Bit 16 means a correction was applied and the
corrected value used; bit 4 is added when nothing can verify it. Wrapper
scripts test bits, not one number; see [`mnemonic
repair`](#mnemonic-repair) and [`mk repair`](#mk-repair) for the other
two surfaces' own exit-code tables.

**Version scoping.** These codes ship with the constellation exit-code
cutover. Before it, `ms repair` reported the same Candidate as exit `4`
(from `ms-cli v0.14.0`), and before `ms-cli v0.14.0` as a confident
exit `5` with no UNVERIFIED advisory. The output panel reads a status from a
binary older than the cutover with that release's own meanings (see
[Run and read the output panel](#run-and-read-the-output-panel)).

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
`ms1` on the last line; exit code `20` (`VERIFY-ME` Candidate — see
Exit codes above). A
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
