# `md repair` {#md-repair}

BCH error-correct one or more `md1` strings. The GUI exposes
`md-cli`'s `repair` subcommand (md-cli v0.6.2+), which wraps
`md_codec::decode_with_correction` and renders a per-chunk repair
report. Each chunk is corrected within the regular `BCH(93,80,8)`
code's `t=4` capacity (up to four substitution errors per chunk).

`md repair` is the per-codec sibling of the toolkit's
[`mnemonic repair`](#mnemonic-repair); the two share the same
`RepairJson` envelope schema fields. Since ms-cli v0.14.0 / toolkit
v0.81.0's ms1 demotion, the toolkit's `RepairJson` is a strict
superset (an extra `verdict` field, `"blessed"`/`"candidate"`) that
this NO-BUMP `md repair` envelope does not carry; a shared-field
parser still reads both. A correction sets exit bit 16; a single
string that nothing can verify adds bit 4 (exit 20); see Exit codes
below.

The repair form is the smallest md-tab form: one positional
(`md1-strings`, repeating, required) plus a single optional
boolean flag (`--json`). The conditional-visibility engine has no
arm for this subcommand (`conditional: None`); both controls are
always Visible. `allows_slots: false`.

`md repair` operates on **public** material — `md1` cards encode a
BIP-388 wallet-policy template plus the `policy_id_stub`
cross-binding metadata; they do not carry secret keys. The
run-confirm modal does not fire for any `md repair` invocation.

> **GUI form:** see [GUI Forms › md › repair](#gui-form-md-repair).

## Outline {#md-repair-outline}

- [`--json`](#md-repair-json) — emit a JSON envelope instead of the text report
- [`--in`](#md-repair-in) — read the `md1` strings from a file, one per line

## Positional `<MD1_STRINGS>...`

One or more `md1` strings to attempt to repair (BCH
error-correction). Required, repeating. The GUI renders a
multi-row text widget; add one row per `md1` string. Use `-` to
read one string per line from stdin (no secret material is
involved — `md1` strings are public).

`md repair` accepts BOTH chunked-form `md1` (chunks bearing a
chunk header, as emitted by [`md encode --force-chunked`](#md-encode-force-chunked)
or by automatic chunking when the payload exceeds the
single-string code domain) AND non-chunked single-string `md1`
(the form emitted by plain `md encode` for small payloads).

## `--json` {#md-repair-json}

Boolean. Emit a single JSON envelope on stdout instead of the
text-form report. Default off. The envelope shares the `RepairJson`
fields with [`mnemonic repair --json`](#mnemonic-repair-json); since
toolkit v0.81.0 the toolkit's envelope is a strict superset (adds a
`verdict` field this NO-BUMP `md repair` envelope does not carry; see
the intro above), so a wrapper reading the shared fields parses both.

## `--in` {#md-repair-in}

Path widget. Read the `md1` strings from FILE, one per line, instead of
the positional (P3 §6b, the file channel added in md 0.20). Display separators are stripped per line, so a card copied off an engraving card re-ingests. The
GUI treats `--in` as an alternative to the positional, which is no
longer marked required: fill one or the other.

## Per-chunk atomic semantics

When multiple `md1` strings are supplied (typical for chunked
descriptor backups), if ANY chunk fails to repair (more than four
substitution errors, or a structural wire-format error), the WHOLE
call aborts: it exits `2` with the offending chunk index named on
stderr, and emits NO partial corrected output. Re-run with better
data for the failing chunk.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | every chunk already valid; no correction applied; inputs echoed unchanged. |
| `16` | at least one chunk corrected and verified (a chunked set that reassembles), or a chunk of an incomplete set corrected on its own (the rest of the set verifies it later); stdout = repair report + corrected chunks. |
| `20` | `16 + 4`: a single (non-chunked) string corrected; nothing can verify it, so stderr carries an `UNVERIFIED` note. |
| `22` | `16 + 4 + 2`: a corrected single string that still does not decode; it is printed. |
| `2` | atomic-fail: any chunk exceeding BCH `t=4` capacity (or a structural wire-format error), a complete set that does not reassemble, or a set with a repeated, stray or extra chunk aborts the whole call; the failing chunk index is named on stderr; NO partial output. |
| `32` | a file or stream could not be read or written. |
| `64` | the command line is wrong; nothing ran. |

**The same rule on every CLI.** Bit 16 means a correction was applied
and the corrected value used. Bit 4 is added when nothing can verify it:
a single non-chunked `md1`, and every `ms1` (see [`ms
repair`](#ms-repair)). Wrapper
scripts test bits, not one number: `[ $(( rc & 16 )) -ne 0 ]` means
"corrected".

## Worked example — repair a single-chunk `md1`

1. **md** tab; pick **Repair (BCH error-correction)**.
2. Add one `md1-strings` row: paste a single-chunk `md1` that has
   acquired a one-character substitution error (for instance, one
   of the canonical [§51](#md-per-tab-reference) strings with a
   single mis-engraved character).
3. Leave `--json` off for the human-readable report.
4. Click **Run**. No run-confirm modal (no secret-class input).

The output panel renders the repair report header (describing the
fix) followed by the corrected chunk on the trailing line. Exit
code `20` for a single string (with an `UNVERIFIED` note on stderr),
`16` for a chunked one. For a multi-chunk `md1`, the report names each corrected
chunk's index on its own `#` comment line and emits every chunk on
the trailing lines.

\index{md repair}
