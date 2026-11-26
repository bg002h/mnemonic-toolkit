# `mk repair` {#mk-repair}

BCH error-correct one or more corrupted `mk1`
strings.\index{mk repair} Both BCH code variants are supported:
the regular `BCH(93,80,8)` code for data-parts of 14–93 symbols
(short `mk1` chunks) and the long `BCH(108,93,8)` code for
data-parts of 96–108 symbols (the xpub-bearing first chunk of a
typical chunked emission). Both correct up to four substitution
errors per chunk (singleton bound `t=4`).

`mk repair` is the per-codec sibling of the toolkit's `mnemonic
repair`. The two share the `RepairJson` envelope's fields; since
toolkit v0.81.0's ms1 demotion, the toolkit's envelope is a strict
**superset** — it carries an extra `verdict` field (`"blessed"` /
`"candidate"`, after `kind`) that this NO-BUMP `mk repair` envelope
(`{schema_version, kind, corrected_chunks, repairs}`) does NOT; a
shared-field parser still reads both. Beyond that, `mk repair`
operates exclusively on the `mk` HRP (no `--ms1`/`--mk1`/`--md1`
selector) and emits no "did you mean" HRP suggestion (single-HRP
context).

[`mk decode`](#mk-decode) already performs internal BCH correction
within the same `t=4` capacity during normal decode. `mk repair`
is the explicit-fix-with-report counterpart: it surfaces which
character positions were corrected — useful for salvaging a
corroded engraving, a hand-copied card with a single typo, or
sanity-checking a freshly engraved card before committing to
steel.

This subcommand operates on **public** key-card material only.
The run-confirm modal does not fire.

> **GUI form:** see [GUI Forms › mk › repair](#gui-form-mk-repair).

## `--json` {#mk-repair-json}

Boolean. Emit a single JSON envelope on stdout (`kind` is `"mk1"`)
instead of the text-form report. The envelope shares the `RepairJson`
fields with `mnemonic repair --json`, whose toolkit envelope is a
strict superset since v0.81.0 (adds `verdict`; see above); a
shared-field parser reads both. The per-repair detail may carry a
`code` field naming the BCH variant (`regular`/`long`). Two fields
follow `kind` and are absent unless they apply: `"not_a_card": {"kind",
"message"}` at exit `22`, and `"incomplete": true` for an incomplete
chunk set (exit `0` or `16`). Default off.

## Positional `mk1-strings`

One or more `mk1` strings to attempt to repair. **Repeating**
positional. The literal `-` reads one string per line from stdin
until EOF.

**Per-chunk atomic semantics:** when multiple strings are supplied
(typical for chunked emissions), if ANY chunk exceeds the
four-error capacity the WHOLE call fails with the offending chunk
index named — partial repair of sibling chunks is NOT returned, to
avoid surfacing a half-fixed card.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | all strings already valid (input echoed unchanged) |
| `16` | at least one string corrected (`REPAIR_APPLIED`) and the corrected group decodes; stdout = report + corrected strings. If the supplied strings are an INCOMPLETE `chunk_set_id` group (e.g. a single chunk of a multi-chunk card), stderr additionally carries a `warning:` line: the correction is verifiable when the rest of the set is reassembled. A clean incomplete group is echoed at `0` |
| `18` | `16 + 2`: a corrected group that decodes to a card that then fails validation; it is printed, with an error line (a clean one is printed at `2`). `mnemonic repair --mk1` does not run this check yet |
| `22` | `16 + 4 + 2`: a corrected string that is not a card — a corrected single (not chunked) string that does not decode, or a corrected string whose chunk header does not parse as an mk1 chunk; it is still printed, with an error telling you not to trust it. A corrected single string that decodes (`20`) cannot occur in mk |
| `2` | unrepairable (too many errors, HRP mismatch); a clean string `mk decode` rejects or whose chunk header does not parse; a COMPLETE `chunk_set_id` group whose correction fails cross-chunk reassembly (the per-chunk correction aliased to a different, wrong card); **or** chunks that are not one card's set (an index given twice or outside the set, more chunks than the set declares, or chunks that disagree on its size). Nothing is printed, and the reason is on stderr |
| `32` | a file or stream could not be read or written |
| `64` | the command line is wrong; nothing ran |

**The same codes as the toolkit.** `mk repair` and [`mnemonic repair
--mk1`](#mnemonic-repair-mk1) give the same code, and print the same
strings, on mk-codec's negative test vectors N6 to N21, each clean and
with one substitution.
Bit 16 means a correction was applied: it is **verified now** (a
complete, cleanly-reassembling `chunk_set_id` group) or **verifiable by
reassembly later** (an incomplete group). Bit 4 is added only where
nothing can verify it, as for every `ms1` (see [`ms
repair`](#ms-repair)). Wrapper scripts test bits, not one number.

**Version scoping.** Set-level re-verify shipped in `mk-cli v0.12.0`
(the Cycle-E funds fix): a COMPLETE `chunk_set_id` group whose
correction FAILS cross-chunk reassembly exits `2`
(`SetReassemblyMismatch`), whereas at `≤ v0.11.2` that aliased
correction was blessed. A correction in an incomplete chunk set reads
`` warning: corrected chunk(s) of an incomplete chunk set
(chunk_set_id 0x…): the correction is verifiable when the rest of the
set is reassembled — run `mk decode` with every chunk of the card to
confirm it `` and exits `16`. Before the constellation exit-code
cutover a correction exited `5`, and the advisory said the correction
was UNVERIFIED; the output panel reads such a binary's code with the
old meanings (see [Run and read the output
panel](#run-and-read-the-output-panel)).

## Worked example

:::danger
The `mk1` below is public test material from the canonical
all-`abandon` seed corpus. **Never fund the wallet it describes.**
:::

1. **mk** tab; pick **Repair (BCH error correction)**.
2. Paste the corrupted `mk1` string(s) into the `mk1-strings`
   field.
3. Leave `--json` unchecked.
4. **Run** (no run-confirm modal — `mk repair` operates on public
   material).

The output panel renders the repair report; the corrected string
is on the LAST line. Exit code `16` signals a repair was applied.

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `mk1-strings` provided AND stdin not used | clap-level `required` error |
| Any chunk exceeds the four-error correction capacity | exit 2 with the offending chunk index named |
| HRP is not `mk` | exit 2 — single-HRP context; no cross-HRP suggestion |
