# Troubleshooting

Operational troubleshooting for the GUI: symptoms you'll see in
the form, the output panel, or the run-confirm modal, mapped to
their underlying cause and a one-line fix. The chapter is
GUI-specific; for CLI-side error diagnostics consult the
companion CLI manual's appendix G (`mnemonic` /
`md` / `ms` / `mk` per-binary troubleshooting matrix).

## When to read this chapter

- The binary launches but a tab refuses to render
  (`Pinned: mnemonic ?` showing instead of a version
  string).
- The form fills in but **Run** does nothing — or fails
  with an error you can't decode.
- The run-confirm modal\index{run-confirm modal} fires when
  you don't expect it to (or doesn't fire when you do).
- The OS surfaces something concerning — a screenshot taker
  caught a secret, the clipboard\index{clipboard} retained a
  pasted phrase, a screenreader announced an unmasked value.

## Chapter map

- [Binary, launch, and version mismatch](#binary-and-launch) —
  the sibling CLIs aren't on `$PATH`, or their version differs
  from the GUI's `Pinned:` banner.
- [Form fill and Run errors](#form-and-output) — empty
  required fields, dropdown rejection, conditional-visibility
  surprises, output-panel parsing.
- [Secret handling and the OS](#secrets-and-os) — run-confirm
  modal\index{run-confirm modal} behavior, the v0.3.0
  redaction gap, screenshot / clipboard / screenreader
  hygiene, cold-node operational mitigations.

## When in doubt

1. **Re-read the chapter for the active tab.** Most
   form-level errors trace back to a misuse of a single
   flag — the per-tab reference is the source of truth.
2. **Open the run-confirm modal**\index{run-confirm modal} **on
   a deliberately bad invocation**. The modal renders the full
   argv before execution; you can spot a typo there without
   committing to **Run**.
3. **Fall back to the CLI**. If you can reproduce the failure
   from a sibling-CLI invocation in a terminal, that confirms
   the issue is upstream of the GUI (e.g., a corrupt input or
   a tooling-version mismatch); the CLI manual will diagnose
   it more directly.
4. **Check the FOLLOWUPS file**. Known GUI-side limitations
   live at `design/FOLLOWUPS.md` in the `mnemonic-gui` repo;
   the v0.3.0 modal-redaction gap and the conditional-visibility
   gaps for multisig templates are both filed there.
