# `md repair` {#md-repair}

BCH error-correct one or more `md1` strings. The GUI exposes
`md-cli`'s `repair` subcommand (md-cli v0.6.2+), which wraps
`md_codec::decode_with_correction` and renders a per-chunk repair
report. Each chunk is corrected within the regular `BCH(93,80,8)`
code's `t=4` capacity (up to four substitution errors per chunk).

`md repair` is the per-codec sibling of the toolkit's
[`mnemonic repair`](#mnemonic-repair); the two share the same
`RepairJson` envelope schema byte-exact, and the exit-5
`REPAIR_APPLIED` signal is uniform across all four CLIs
(`mnemonic`, `mk`, `ms`, `md`).

The repair form is the smallest md-tab form: one positional
(`md1-strings`, repeating, required) plus a single optional
boolean flag (`--json`). The conditional-visibility engine has no
arm for this subcommand (`conditional: None`); both controls are
always Visible. `allows_slots: false`.

`md repair` operates on **public** material — `md1` cards encode a
BIP-388 wallet-policy template plus the `policy_id_stub`
cross-binding metadata; they do not carry secret keys. The
run-confirm modal does not fire for any `md repair` invocation.

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
text-form report. Default off. The envelope's schema byte-matches
[`mnemonic repair --json`](#mnemonic-repair-json)'s `RepairJson`
shape, so a wrapper that already parses one parses the other.

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
| `5` | at least one chunk corrected (`REPAIR_APPLIED`); stdout = repair report + corrected chunks. |
| `2` | atomic-fail: any chunk exceeding BCH `t=4` capacity (or a structural wire-format error) aborts the whole call; the failing chunk index is named on stderr; NO partial output. |
| `1` | I/O error or other generic failure. |

The exit-5 `REPAIR_APPLIED` code is consistent across all four
CLIs, so wrapper scripts can use a uniform `exit == 5` signal.

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
code `5`. For a multi-chunk `md1`, the report names each corrected
chunk's index on its own `#` comment line and emits every chunk on
the trailing lines.

\index{md repair}
