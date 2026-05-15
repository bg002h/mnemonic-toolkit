# Run and read the output panel

This chapter picks up where chapter 31 left off, with the `mk
inspect` form filled in. We click **Run**, read the output panel,
and then run a second walkthrough that exercises the run-confirm
modal — the GUI's pre-flight pause for any form containing a
secret-class flag. The second walkthrough uses the canonical
all-`abandon` BIP-39 test phrase and is therefore the first
`:::danger` admonition in this manual.

## Click Run on a non-secret form

With the `mk inspect` form from chapter 31 filled in, click the
**Run** button at the right end of the action bar. Because no
flag in the form is `secret: true` and no NodeValueComposite
node value is in the secret-class set, the GUI fires the
subprocess immediately — no modal interposes. The output panel at
the bottom of the window updates in place:

```text
+--------------------------------------------------------------+
| ☑ show command-line  ☑ show stdout  ☑ show stderr            |
| argv: mk inspect mk1qprsqhpqqsq3cqtsleeutks...854wq4         |
| exit: 0                                                      |
| stdout:                                                      |
|   xpub:                xpub6CatWdiZiodmU...VMrjPC7PW6V       |
|   origin_fingerprint:  73c5da0a                              |
|   origin_path:         m/84'/0'/0'                           |
|   policy_id_stubs:     deadbeef                              |
|   chunks:              2 (regular)                           |
|   xpub_fingerprint:    73c5da0a                              |
|     component[0]:      84h (hardened)                        |
|     component[1]:      0h (hardened)                         |
|     component[2]:      0h (hardened)                         |
|     chunk[0]:          regular (BCH variant)                 |
|     chunk[1]:          regular (BCH variant)                 |
+--------------------------------------------------------------+
```

These are the same fields the CLI surfaces when you run
`mk inspect <MK1-STRING>` from a shell — `mk inspect`'s text-mode
output is documented in detail at the CLI manual's `40-cli-reference/44-mk-cli.md`
chapter. The GUI does not re-format or re-shape the output; it
captures stdout verbatim and displays it in the panel's monospace
scroll region.

The three checkboxes at the top of the panel toggle visibility of
each section: **show command-line** (the assembled argv as it was
spawned), **show stdout** (the binary's standard-output stream),
and **show stderr** (its standard-error stream). All three are
checked by default. The `exit:` line shows the subprocess's exit
code (`0` on success; a non-zero integer on failure; the literal
string `(killed)` if the process was terminated by a signal before
it could exit). When stderr is non-empty (warnings, diagnostic
messages, errors), it renders below stdout in its own scroll
region. If the subprocess could not be spawned at all (binary
missing from `$PATH`, permission denied, etc.), the GUI shows a
red `subprocess error: <message>` line at the top of the panel
instead of an `exit:` line.

## A non-zero-exit example

If you click **Run** on `mk inspect` with no positional arguments
and the `--json` checkbox unchecked, the subprocess exits non-zero
(usage error). The output panel renders the CLI's own error
message in the `stderr:` region:

```text
exit: 2
stderr:
  error: the following required arguments were not provided:
    [MK1_STRINGS]...
  Usage: mk inspect [OPTIONS] [MK1_STRINGS]...
```

This is the CLI's clap-derive usage diagnostic, surfaced through
the GUI without modification. The same diagnostic appears if you
run `mk inspect` from a shell with no arguments.

## A second walkthrough — the run-confirm modal

Now switch back to the **mnemonic** tab and pick **Convert (between
formats)** from the subcommand selector. The `convert` form
appears, with `--from` (a NodeValueComposite flag — pick the
`phrase` node from its dropdown) and `--to` (a Dropdown flag — pick
`ms1`) as the two required flags.

:::danger
The next paragraphs walk through entering a BIP-39 phrase into the
GUI. **Use only the canonical test vector**
`abandon abandon abandon abandon abandon abandon abandon abandon
abandon abandon abandon about`. This phrase is **public**: every
wallet derived from it has been swept by chain watchers continuously
since 2017 and is worth zero satoshis on every chain. **Do not type
your real seed phrase into the GUI on this or any other walkthrough**
— see [§14 Defense 2](#secret-handling) for the unredacted-modal
operational warning that applies to every secret-bearing GUI run.
:::

Fill in the form:

```text
--from   [ phrase ▾ ] = abandon abandon ... about
--to     [ ms1 ▾ ]
```

Click **Run**. Because the `phrase` node is in the secret-class
set, the GUI does NOT fire the subprocess immediately; instead the
**run-confirm modal** appears, centered over the main window:

```text
+-----------------------------------------------------------------+
|              Confirm secret-bearing run                         |
+-----------------------------------------------------------------+
| This invocation passes secret-bearing arguments to              |
| ----                                                            |
| Argv:                                                           |
|   mnemonic                                                      |
|   convert                                                       |
|   --from                                                        |
|   phrase=abandon abandon abandon abandon abandon abandon ...    |
|   --to                                                          |
|   ms1                                                           |
| ----                                                            |
| [ Run ]   [ Cancel ]                                            |
+-----------------------------------------------------------------+
```

Two buttons: **Run** confirms and fires the subprocess; **Cancel**
dismisses the modal and leaves the form unchanged. There is no
Escape-key affordance — see [§14 Defense 2](#secret-handling) for
the threat-model rationale.

**At `mnemonic-gui` v0.3.0 the modal renders the secret-bearing
argv tokens in plaintext** (the `phrase=abandon abandon ... about`
line above is rendered verbatim, character by character, on screen).
This is the v0.3.0 behavior the manual describes today. It is
under-defended against the screen-observation threats listed in
[§14 Defense 2](#secret-handling), and the linked operational
mitigation — running the GUI only on a cold / airgapped node —
applies to **every** secret-bearing GUI invocation, not just this
walkthrough. The redaction gap is tracked at FOLLOWUP
`gui-run-confirm-modal-secret-redaction` in the GUI repo and at
companion `gui-run-confirm-modal-secret-redaction-manual-companion`
in this repo; v1.1 of this manual will revise this section when
the GUI fix lands.

Click **Run** in the modal. The subprocess fires; the output panel
updates with the `ms1` encoding of the test vector:

```text
exit: 0
stdout:
  ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

That same `ms1` string is the canonical worked-example seed-secret
card used throughout the CLI manual's quickstart. You have just
demonstrated that the GUI's `mnemonic convert --from phrase --to
ms1` produces the same output as the CLI invocation it wraps.

## What the output panel does NOT do

- It does not stream output line-by-line as the subprocess writes
  it. The GUI captures stdout and stderr *to completion* and
  renders them after the subprocess exits. Long-running commands
  show no progress indicator beyond the absence of an updated
  `exit:` line.
- It does not preserve previous runs. Each **Run** click overwrites
  the panel with the most recent invocation's output. To keep a
  transcript, copy text out of the panel before re-running, or
  invoke the same argv from a shell where you control the
  scrollback.
- It does not redact secrets in the `argv:` echo line. The
  `argv:` line uses the same un-redacted assembled argv that the
  modal showed. If you ran a secret-bearing form, the secret
  remains visible in the output panel until the next **Run**.

The next chapter (33) covers the `?` help-icons that deep-link
into this manual.
