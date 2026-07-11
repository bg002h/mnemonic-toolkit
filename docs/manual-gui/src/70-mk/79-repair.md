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
`code` field naming the BCH variant (`regular`/`long`). Default off.

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
| `5` | at least one string corrected (`REPAIR_APPLIED`); stdout = report + corrected strings. If the supplied strings are an INCOMPLETE `chunk_set_id` group (e.g. a single chunk of a multi-chunk card), stderr additionally carries an `UNVERIFIED` advisory — the exit code does NOT change (see the asymmetry note below) |
| `2` | unrepairable (too many errors, HRP mismatch), **or** a COMPLETE `chunk_set_id` group whose correction fails cross-chunk reassembly (the per-chunk correction aliased to a different, wrong card) |
| `1` | I/O or other generic failure |

**Asymmetry vs. the toolkit — read before relying on `exit == 5`
uniformly.** This standalone `mk repair` binary reports exit `5` for
any NON-rejected correction — including an INCOMPLETE `chunk_set_id`
group, where it adds the `UNVERIFIED` advisory above but does NOT
change the exit code (a DESIGNED Cycle-E difference: `mk1` is
watch-only material, so the standalone binary emits a stderr advisory
yet still exits 5). A COMPLETE group whose correction fails
cross-chunk reassembly is instead REJECTED at exit `2` (row 2 above),
not `5`.
[`mnemonic repair --mk1`](#mnemonic-repair-mk1) (the toolkit's own
copy of this same engine) instead **demotes** that identical
incomplete-group case to exit `4` `VERIFY-ME`. The two surfaces
diverge on this one case; both share every other exit code. The
principled rule across all four CLIs: exit-`5` `REPAIR_APPLIED` means
a correction is **verified now** (a complete, cleanly-reassembling
`chunk_set_id` group) **or verifiable-by-reassembly later** (this
binary's own incomplete-group case, above) — never "an oracle
verified it" standing alone. Exit-`4` `VERIFY-ME` means a substitution
correction that spent the checksum's error-detection budget and has
**no self-oracle** — always true for `ms1` (see [`ms
repair`](#ms-repair)), and true for an incomplete `mk1` group
specifically in `mnemonic repair --mk1`. `md1` never demotes at all
(see [`md repair`](#md-repair)). Wrapper scripts must NOT branch on a
single `exit == 5` constant across the four binaries.

**Version scoping — set-level re-verify ships in `mk-cli v0.12.0`.**
That release (the Cycle-E funds fix) added BOTH the incomplete-group
`UNVERIFIED` advisory AND the complete-group reassembly-reject: a
COMPLETE `chunk_set_id` group whose correction FAILS cross-chunk
reassembly now exits `2` (`SetReassemblyMismatch`; row 2 above),
whereas at `≤ v0.11.2` no set-level re-verify existed, so that same
aliased correction was blessed at exit `5`. Only the INCOMPLETE-group
case kept exit `5` (with the new advisory). The
`v0.12.0`+ advisory reads
`` warning: correction UNVERIFIED — reassemble the full card (`mk
decode`) to confirm; a >4-error correction can alias to a different
card; BIP-93 recommends confirming a corrected codex32 string `` —
worded differently from the `ms`/toolkit surfaces' advisory (compare
[`ms repair`](#ms-repair)) but making the same recommendation.
**This manual is pinned to `mk-cli v0.11.0`** (`pinned-upstream.toml`)
— PRE-fix — so a build at the manual's own pinned tag exits `5` for
BOTH cases: an incomplete-group correction (with no `UNVERIFIED`
line) AND a complete-group reassembly failure (blessed as a
confident fix rather than rejected at exit `2` — the funds gap this
release closes).

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
is on the LAST line. Exit code `5` signals a repair was applied.

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `mk1-strings` provided AND stdin not used | clap-level `required` error |
| Any chunk exceeds the four-error correction capacity | exit 2 with the offending chunk index named |
| HRP is not `mk` | exit 2 — single-HRP context; no cross-HRP suggestion |
