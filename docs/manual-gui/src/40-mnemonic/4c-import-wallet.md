# `mnemonic import-wallet` {#mnemonic-import-wallet}

Import a third-party watch-only wallet blob — BSMS Round-2 (BIP-129)
or Bitcoin Core `listdescriptors` JSON — into an m-format bundle.
The companion to [`mnemonic export-wallet`](#mnemonic-export-wallet):
where export TRANSLATES the bundle's public material into a foreign
format, import INGESTS that foreign format and reconstructs the
equivalent bundle. The round-trip discipline gates byte-exact vs
semantic-only equivalence (see the CLI manual chapter on
[foreign wallet formats](#foreign-wallet-formats) for the format
taxonomy).

v0.26.0 ships two source formats — `bsms` and `bitcoin-core`. Both
are watch-only by construction; the resulting bundle's cosigners
carry no secret material unless the user re-attaches a seed via
the `--ms1` / `--slot @N.phrase=` overlay flags. Bitcoin Core blobs
that carry `xprv` extended private keys are refused with a stderr
pointer.

## Outline {#mnemonic-import-wallet-outline}

- [`--blob`](#mnemonic-import-wallet-blob) — path to the third-party wallet blob (required; `-` reads stdin)
- [`--format`](#mnemonic-import-wallet-format) — format override (`bsms` / `bitcoin-core`); auto-detected via sniff if absent
- [`--select-descriptor`](#mnemonic-import-wallet-select-descriptor) — multi-descriptor selector for Bitcoin Core blobs
- [`--ms1`](#mnemonic-import-wallet-ms1) — repeating seed-overlay flag (positional cosigner-index)
- [`--slot`](#mnemonic-import-wallet-slot) — per-slot seed-overlay flag (`@N.phrase=<value>` form only)
- [`--json`](#mnemonic-import-wallet-json) — emit JSON envelope on stdout instead of engraving cards
- [`--no-auto-repair`](#mnemonic-import-wallet-no-auto-repair) — global flag; skip auto-fire repair on decode failures

## `--blob` {#mnemonic-import-wallet-blob}

The path to the third-party wallet blob. Required. Two value
shapes:

- A filesystem path — the GUI renders this as a FilePath widget
  with a file-picker button. The picker filters to extensions
  `.json` / `.txt` / `.bsms` by default; "All Files" toggles the
  filter off.
- The literal `-` — reads the blob from the spawned subprocess's
  stdin. The schema sets `stdio_sentinel: true` for this flag.

The blob's content is NOT secret-class on its own — both source
formats are watch-only. The GUI does not redact `--blob <path>` in
the run-confirm modal.

## `--format` {#mnemonic-import-wallet-format}

The source format. Optional. Without `--format`, the toolkit
auto-detects via sniff (SPEC §6): BSMS blobs begin with the literal
prefix `BSMS 1.0\n`; Bitcoin Core blobs are JSON objects with a
`descriptors: [...]` array.

### Outline {#mnemonic-import-wallet-format-outline}

- [`bsms`](#mnemonic-import-wallet-format-bsms)
- [`bitcoin-core`](#mnemonic-import-wallet-format-bitcoin-core)

### `bsms` {#mnemonic-import-wallet-format-bsms}

BSMS Round-2 per BIP-129. Two accepted wire shapes: a lenient
2-line excerpt (`BSMS 1.0\n<descriptor>#<checksum>`) and the full
6-line shape (additionally carrying token, derivation_path,
first_address, signature). The 2-line shape emits a stderr WARNING
about reduced form. See [BSMS Round-2](#bsms-round-2) in the CLI
manual for full normative grammar.

### `bitcoin-core` {#mnemonic-import-wallet-format-bitcoin-core}

Bitcoin Core's `listdescriptors` RPC output: a JSON envelope of
the shape `{"wallet_name": ..., "descriptors": [...]}`. v0.26.0
accepts both the canonical wrapper shape and the bare-array shape
some older Core clients emit. Refuses `xprv`-bearing descriptors
(`listdescriptors true` output) with a stderr pointer; re-run
`bitcoin-cli listdescriptors` without the `true` argument to get
xpub-only output. See
[Bitcoin Core listdescriptors](#bitcoin-core-listdescriptors) in
the CLI manual.

## `--select-descriptor` {#mnemonic-import-wallet-select-descriptor}

Multi-descriptor selector. Applies to Bitcoin Core blobs only;
BSMS blobs coerce non-default values to `all` with a stderr NOTICE.

### Outline {#mnemonic-import-wallet-select-descriptor-outline}

- [`all`](#mnemonic-import-wallet-select-descriptor-all) (default)
- `<N>` (integer index)
- [`active-receive`](#mnemonic-import-wallet-select-descriptor-active-receive)
- [`active-change`](#mnemonic-import-wallet-select-descriptor-active-change)

### `all` {#mnemonic-import-wallet-select-descriptor-all}

Emit one bundle per descriptor entry in the input. Bundles are
separated by a literal `\n;\n` line in the engraving-card output;
the `--json` envelope is an array.

### `active-receive` {#mnemonic-import-wallet-select-descriptor-active-receive}

Filter to entries with `active: true, internal: false` (Core's
external chain). Multiple matches emit multiple bundles; zero
matches yields exit 1.

### `active-change` {#mnemonic-import-wallet-select-descriptor-active-change}

Filter to entries with `active: true, internal: true` (Core's
internal / change chain). Same multi/zero handling.

The GUI renders this flag as a free-form text input pre-filled
with `all`; the user types one of the named tags (`all`,
`active-receive`, `active-change`) or an integer N. CLI-side
validation rejects invalid values with a clear error in the
output pane.

## `--ms1` {#mnemonic-import-wallet-ms1}

Seed overlay (SPEC §8.3). Repeatable flag; positional cosigner-index
— the i-th `--ms1` applies to cosigner i. The toolkit derives the
xpub from the supplied entropy at the cosigner's blob-declared
origin path and asserts equality against the blob's xpub; mismatch
yields exit 4. Three value shapes are accepted: an inline
`ms1xxx...` string (canonical `ms1` HRP form), the `@env:<VAR>`
sentinel (resolved at clap-parse time via `std::env::var(VAR)`;
whole-value only; missing → exit 1 `EnvVarMissing`), and the
empty-string sentinel `""` (preserves v0.25.1 watch-only semantics;
stderr NOTICE; cosigner stays watch-only).

The GUI renders this as a Text widget with `repeatable: true` and
`secret: true`. The `secret: true` flag triggers the paste-warn
modal at paste time and opens the run-confirm modal before
subprocess spawn. In v0.11.0 the run-confirm modal renders the
assembled argv verbatim per
`[[feedback-run-confirm-modal-renders-argv-verbatim]]`; see the
[Env-var seed channel](#mnemonic-import-wallet-env-var-channel)
section below for the workaround until FOLLOWUP
`gui-import-wallet-env-var-secret-channel` ships in v0.12.0+.

### Env-var seed channel {#mnemonic-import-wallet-env-var-channel}

The v0.11.0 GUI emits user-typed values verbatim on argv; the
toolkit-side resolves `@env:VAR` if the user types the sentinel
explicitly. To avoid argv-leak in the v0.11.0 GUI, type
`@env:MY_VAR` directly into the `--ms1` row with `MY_VAR` exported
in the calling shell before launching the GUI; the toolkit
resolves the sentinel at clap-parse time and the secret never
appears on argv or in the run-confirm modal. Auto-rewriting of
literal seeds to per-cosigner `@env:MNEMONIC_MS1_<i>` sentinels
is FOLLOWUP `gui-import-wallet-env-var-secret-channel` (v0.12.0+).

## `--slot` {#mnemonic-import-wallet-slot}

Per-slot seed overlay via `--slot @<N>.phrase=<BIP-39 phrase>`.
Equivalent to `--ms1`: the phrase is converted to entropy and the
derived xpub at the cosigner's origin path is compared against the
blob's xpub. Mutually exclusive with `--ms1[N]` for the same N.
Accepts the `@env:VAR` sentinel for the phrase value.

In v0.26.0 only the `phrase` subkey is accepted on `import-wallet`;
other subkeys (`entropy`, `xpub`, etc.) are rejected. The GUI's
slot editor renders all subkeys; the **toolkit** (not the GUI)
rejects non-`phrase` subkeys at parse time and the error appears
in the output pane.

## `--json` {#mnemonic-import-wallet-json}

Emit a JSON envelope array on stdout (SPEC §7.4) instead of the
human-readable engraving-card summary. One envelope per emitted
bundle. Each envelope carries `bundle` (parse-side summary),
`source_format`, `roundtrip` (`byte_exact` / `semantic_match` /
`diff` / `status`), `bsms_audit?` (BSMS only), and
`source_metadata?` (Bitcoin Core only).

Under `--json`, the round-trip diff lives in the envelope's
`roundtrip.diff` field only — stderr is silent for the diff.
Default mode renders the diff to stderr.

The GUI renders this as a Boolean toggle. The output panel
auto-formats the JSON when this toggle is set.

## `--no-auto-repair` {#mnemonic-import-wallet-no-auto-repair}

Global flag. Skips auto-fire repair on decode failures and
preserves the pre-v0.22 exit policy. The same flag is honored by
`convert`, `inspect`, and (v0.22.1+) `verify-bundle`. For
`import-wallet`, auto-fire applies to BCH-correctable `mk1` chunks
embedded in the descriptor's key sources.

## Worked example — BSMS Round-2 decay-32768 import {#mnemonic-import-wallet-walkthrough-bsms}

1. **mnemonic** tab; pick **Import Wallet (watch-only)** from the
   subcommand combobox.
2. Click the **`--blob`** file-picker; navigate to the blob (e.g.,
   `/tmp/decay-32768.bsms`); confirm.
3. Leave `--format` unset (sniff will auto-detect BSMS from the
   `BSMS 1.0\n` prefix).
4. Leave `--select-descriptor` at default `all` (BSMS coerces to
   `all` anyway with a stderr NOTICE).
5. Optional: to re-attach the cosigner @0 seed, add a `--ms1` row
   with the cosigner's `ms1xxx...` value, OR add a `--slot
   @0.phrase=<words>` row with the BIP-39 phrase. To keep the seed
   off argv in v0.11.0, type `@env:MY_MS1_0` instead and export
   `MY_MS1_0=<ms1-value>` in the calling shell.
6. Leave `--json` unset for engraving-card stdout (recommended
   for visual inspection); toggle on for machine-readable output.
7. Click **Run**.
   - If a `--ms1` or secret-bearing `--slot` row is filled, the
     run-confirm modal opens and renders the argv verbatim (per
     `[[feedback-run-confirm-modal-renders-argv-verbatim]]`). To
     prevent the seed appearing in the modal AND in `ps` output,
     type `@env:MY_MS1_0` rather than the literal seed (with
     `MY_MS1_0` exported in the shell that launched the GUI).
     Confirm.
   - Output panel renders the synthesized engraving cards (stdout)
     and the BSMS 2-line WARNING (stderr).

Screenshot: TODO post-v0.11.0-GUI tag.

## Worked example — Bitcoin Core listdescriptors active-receive {#mnemonic-import-wallet-walkthrough-core}

Generate the blob (`bitcoin-cli listdescriptors > /tmp/core-export.json`
— *do not* pass `true`; toolkit refuses `xprv` descriptors), pick
it via the file-picker, set `--select-descriptor` to `active-receive`
to filter to the external chain, toggle `--json` on, click **Run**
(no run-confirm modal — no secret-bearing flag). The output panel
renders a JSON envelope array with one entry per matched descriptor;
each carries `bundle`, `source_format: "bitcoin-core"`,
`source_metadata: {wallet_name, active, internal, range}`, and
`roundtrip: {byte_exact, semantic_match, diff, status: "ok"}`.

Screenshot: TODO post-v0.11.0-GUI tag.

## Refusals + advisories

The full refusal + advisory matrix lives in the CLI manual at
[`mnemonic import-wallet` refusals](#mnemonic-import-wallet). Key
GUI-relevant behaviors: inline `--ms1 ms1xxx...` values appear in
argv unless the user types the `@env:VAR` sentinel explicitly (see
[§9.3](#mnemonic-import-wallet-env-var-channel)). The v0.11.0 GUI
emits typed values verbatim; auto-rewriting of literal seeds to
per-cosigner `@env:MNEMONIC_MS1_<i>` sentinels is FOLLOWUP
`gui-import-wallet-env-var-secret-channel` (v0.12.0+). Bitcoin
Core round-trip DROPS the `timestamp` / `next` / `next_index`
wallet-state fields with a stderr NOTICE. BSMS round-trip DROPS
the audit envelope; the `--json` envelope preserves these verbatim
in `bsms_audit` for external re-attachment.

## See also

- [`mnemonic import-wallet` (CLI manual)](#mnemonic-import-wallet) —
  flag-by-flag reference + worked examples on the CLI side.
- [Foreign wallet formats (CLI manual)](#foreign-wallet-formats) —
  normative grammar for BSMS Round-2 and Bitcoin Core
  `listdescriptors`, plus the round-trip discipline + the
  not-yet-supported coverage matrix.
- [`mnemonic export-wallet`](#mnemonic-export-wallet) — the
  watch-only emit side (BSMS emitter is FOLLOWUP for v0.27+;
  Bitcoin Core emit ships v0.13.0+).
