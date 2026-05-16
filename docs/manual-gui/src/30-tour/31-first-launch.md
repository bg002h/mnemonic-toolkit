# First-launch walkthrough

This chapter is a hands-on tour of `mnemonic-gui` from cold start to
your first non-secret-bearing **Run**. It assumes you have installed
the GUI per chapter 21 / 22 / 23 and that the four constellation CLIs
(`mnemonic`, `md`, `ms`, `mk`) are present on `$PATH`. The whole
walkthrough is non-secret — you can do it on any machine without
exposing material from a real wallet. Chapter 32 picks up the
secret-bearing flow and the run-confirm modal; chapter 33 covers the
`?` help-icons.

## Launch and the three-panel layout

Run `mnemonic-gui` from a terminal (or your desktop launcher). On
first launch the window opens with three panels:

```text
+--------------------------------------------------------+
| mnemonic-gui  |  mnemonic ◀ |  md  |  ms  |  mk        |  ← top tab strip
+--------------------------------------------------------+
| Pinned: mnemonic 0.13.0  |  subcommand: bundle ▾  ?           |
| ----                                                          |
|   --network        [ mainnet ▾ ]                              |
|   --template       [ bip84 ▾ ]                                |
|   --account        [ 0    ]                                   |
|   --multisig-path-family [ bip87 ▾ ]                          |  ← central form
|   ...                                                          |
|   Slot rows:  ?                                                |
|     @ [0] . [ xpub ▾ ] = [             ] [✕]                   |
|     [ + Add slot ]                                             |
|   ----                                                         |
|   [ Copy command (POSIX) ] [ Copy command (Windows) ] [ Run ]  |
|   Preview: mnemonic bundle --network mainnet --template ...    |
+----------------------------------------------------------------+
| ☐ show command-line  ☐ show stdout  ☐ show stderr              |
| (no run yet)                                                   |  ← output panel
+----------------------------------------------------------------+
```

The `Pinned: mnemonic 0.13.0` string is the runtime `--version`
banner format that the GUI reads from each CLI binary at launch
and displays for cross-reference. This is intentionally distinct
from the git-tag form `mnemonic-toolkit-v0.13.0` that lives in
`docs/manual-gui/pinned-upstream.toml` and drives CI install. The
two artifacts pin the same release; only their string formats
differ.

The **top tab strip** has the heading `mnemonic-gui`, a separator,
and one button per CLI (`mnemonic`, `md`, `ms`, `mk`). The currently
active tab is marked with `◀` immediately to its right. If a CLI's
binary is missing from `$PATH` the corresponding tab renders greyed
out with a hover tooltip explaining how to install it.

The **central form** has, top-to-bottom: a `Pinned: <version>` label
showing the pinned upstream tag for the active tab, the subcommand
selector (a ComboBox labelled `subcommand`) with a `?` help-icon
next to it, the form widgets for the selected subcommand (one per
flag), an optional `Slot rows:` section if the subcommand accepts
the `--slot` repeating flag, an action bar with **Copy command
(POSIX)**, **Copy command (Windows)**, and **Run** buttons, and an
always-on `Preview:` line showing the assembled argv as you would
type it at the shell.

The **output panel** has three checkboxes (`show command-line`,
`show stdout`, `show stderr`) and a placeholder `(no run yet)` until
you click **Run** for the first time.

## Default-launch state

The first time you open the GUI, the active tab is `mnemonic` and
the subcommand selector is set to `bundle`. The bundle form is
pre-seeded with a small set of demo defaults (`--network mainnet`,
`--template bip84`, `--account 0`, `--multisig-path-family bip87`,
plus one empty slot row with `xpub` selected as its subkey). These
defaults are visible scaffolding so the form is not empty on first
look; they are NOT a wallet you should fund. The bundle subcommand
also requires a secret-class `--ms1` value to actually run, which is
why we do not click **Run** here — chapter 32 picks that up.

## Pick a tab — switch to `mk`

Click the **mk** button in the top tab strip. The tab strip now
shows `mk ◀` and the central form re-renders for the `mk` CLI. The
default `mk` subcommand is `inspect`, which has one flag (`--json`)
and one repeating positional argument (`mk1-strings`).

```text
+----------------------------------------------------------------+
| mnemonic-gui  |  mnemonic |  md |  ms |  mk ◀                  |
+----------------------------------------------------------------+
| Pinned: mk 0.3.1  |  subcommand: inspect ▾  ?                  |
| ----                                                           |
|   ☐ --json                                                     |
|   mk1-strings  ...                                             |
|     [                                                          ]
|   ----                                                         |
|   [ Copy command (POSIX) ] [ Copy command (Windows) ] [ Run ]  |
|   Preview: mk inspect                                          |
+----------------------------------------------------------------+
```

The subcommand selector's *closed* state shows the bare CLI name
(`inspect`). When you click the ▾ to open the dropdown, the
per-option labels switch to the *human-readable* form
(`Inspect (structural commentary)`) — easier to skim when
choosing among 5–10 subcommands. The bare CLI name is what appears
in the `Preview:` line and in the assembled argv. Compare a
similar pair from the open dropdown: the `Encode (xpub → mk1)`
option corresponds to `mk encode`.

## Fill in a positional — paste the canonical mk1

Paste the canonical worked-example mk1 string into the
`mk1-strings` text field:

```text
mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
```

This mk1 is the *master-key card* derived from the canonical
all-`abandon` BIP-39 test vector at the BIP-84 / m/84'/0'/0'
account-zero path. It is **public** material — `mk1` strings
encode an xpub plus a derivation origin and carry no spending
authority on their own. As you type, the `Preview:` line updates
in real time:

```text
Preview: mk inspect mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
```

The argv assembly rules are the same ones the CLI manual documents;
the GUI does not introduce new argument ordering or quoting.
Whitespace inside a positional or flag value is preserved verbatim
(POSIX argv semantics — the GUI passes argv tokens directly to
`std::process::Command::args` with no shell interposition).

## What you have so far

You have launched the GUI, navigated tabs, picked a non-default
subcommand, and filled in one positional argument. The form is
ready to **Run** — which is the next chapter. Nothing on screen so
far is secret-bearing, so the **Run** button will fire the
subprocess immediately when you click it (no run-confirm modal).
The only pre-Run sanity-check at this point is the always-on
`Preview:` line; if its argv looks wrong, fix the form before
clicking **Run**.
