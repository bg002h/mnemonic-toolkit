# Form fill and Run errors {#form-and-output}

Symptoms you see between filling in the form and reading the
output panel — clap-level argument refusals, conditional-visibility
surprises, output panel parsing.

## "Required" markers and clap refusals

The GUI's form scaffolding marks a flag as `Required` when:

- The `SubcommandSchema.flags` entry sets `required: true`
  (clap-level required), **or**
- A conditional-visibility function elevates it at runtime
  (e.g., `md_encode` marks `--from-policy` Required when the
  positional `[TEMPLATE]` is empty).

Hitting **Run** with a Required field empty surfaces a clap-level
`the following required arguments were not provided: <--flag>`
error in the output panel.

| Symptom | Likely cause | Fix |
|---|---|---|
| `error: the following required arguments were not provided: …` | A flag the form marked Required is empty | Fill the field. The top-of-form red-asterisked label and the in-line red border both highlight which flag. |
| `error: unexpected argument '--…'` | Schema-vs-CLI drift (see [§82 GUI-newer-than-CLI](#binary-and-launch)) | Upgrade the sibling CLI, or remove the offending flag from the form. |
| `error: invalid value '…' for '--…'` | A dropdown variant or `Text` value failed clap's value parser | Re-check the value against the chapter's per-variant section; common traps include language token form (`chinese-simplified` vs `simplifiedchinese` — [§61](#ms-per-tab-reference)). |
| `error: --foo cannot be used with --bar` | Two mutually-exclusive flags both have values, and the runtime guard or clap conflict fired | See the per-flag `Disabled` conditional in the per-tab chapter for which flag the GUI should have disabled (file a FOLLOWUP if the conditional did not fire). |

## Conditional-visibility surprises

The conditional-visibility engine\index{conditional-visibility}
at `mnemonic-gui/src/form/conditional.rs` runs per-subcommand
predicates to elevate or suppress flag widgets based on the
current form state. When a flag widget unexpectedly disappears,
disables, or marks itself Required, the cause is almost always
the conditional engine — not a bug in the schema.

Per-subcommand conditional summaries are documented in the
per-tab chapters:

- `mnemonic bundle` — passphrase XOR rules; see [§42](#mnemonic-bundle).
- `mnemonic export-wallet` — `--taproot-internal-key`
  conditional-required for tr templates; see
  [§45](#mnemonic-export-wallet).
- `md encode` — positional `[TEMPLATE]` XOR `--from-policy`;
  `--context` conditional-required under `--from-policy`;
  `--unspendable-key` value-disabled by `--context segwitv0`;
  see [§53](#md-encode).
- `ms encode` — `--phrase` XOR `--hex` (with `--language`
  hidden on the hex path); see [§63](#ms-encode).
- `mk encode` — `--origin-fingerprint` XOR
  `--privacy-preserving`; see [§73](#mk-encode).

Known v0.3.0 gap: bundle multisig flags
(`--multisig-path-family`, `--threshold`) do **not** disable
under single-sig templates (FOLLOWUP
`gui-bundle-multisig-flags-conditional`). Filling them under
`--template bip84` etc. surfaces a runtime CLI refusal at **Run**
time, not a Disabled state at form-fill time.

## Output panel troubles

| Symptom | Likely cause | Fix |
|---|---|---|
| Output panel is empty after **Run** but `Pinned:` reads cleanly | Subprocess exited 0 with no stdout (e.g., `mk vectors --out <dir>` writes files and emits a stderr line; stdout is empty) | Check the **stderr** tab next to **stdout** in the output panel; many subcommands route diagnostic + log output there. |
| Output panel shows JSON that's hard to read | Subcommand's `--json` mode is on AND the form did not request pretty-printing | Toggle `--pretty` if the subcommand supports it (e.g., `ms vectors --pretty`, `mk vectors --pretty`); else pipe through `jq` from a terminal. |
| Output panel shows a UTF-8 error mid-string | Sibling CLI emitted non-UTF-8 bytes (rare — usually a corrupt input) | Re-run from a terminal; the GUI uses `String::from_utf8_lossy` for display, so a `�` substitution indicates raw bytes the form should not have accepted. File a bug. |
| Output panel truncates a long line | Output buffer wrap; no truncation in the actual subprocess output | Resize the GUI window wider, or check **Copy output** (the clipboard receives the full untruncated text). |

## When **Run** seems to do nothing

If the **Run** button greys briefly but no output appears and
the spinner doesn't render either, the click probably hit the
button while the form was in an invalid state and the GUI
suppressed the spawn. Look for:

- A red-bordered Required field elsewhere on the form (scroll
  the form pane).
- A conditional-visibility constraint marking the active
  selection Required but no value supplied.
- The top-of-form `Pinned: <name> ?` banner indicating the
  binary is missing ([§82](#binary-and-launch)).
