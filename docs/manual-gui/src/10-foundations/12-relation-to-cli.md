# How the GUI relates to the four CLIs {#relation-to-cli}

The GUI requires the four CLIs to be **already installed on `$PATH`**.
It does not bundle them. At launch, `mnemonic-gui` runs a `PATH`
sweep for each binary; tabs whose binary is missing render greyed-out
with an install-instructions tooltip.

## Subprocess invocation model

When you click **Run**, the GUI:

1. Composes an argv tuple from the form widgets. Dropdowns become
   `--name value` pairs; NodeValueComposite flags become
   `--name node=value` (fused with `=`); repeating-field flags
   become `--name v1 --name v2 ...`; `--slot @N.subkey=value`
   composes from the slot-editor table.
2. Spawns the binary as a subprocess with that argv. The binary is
   the one detected on `$PATH` at startup — pinned binary paths are
   not yet supported (FOLLOWUP `gui-pin-binary-path`).
3. Captures stdout, stderr, and exit code; renders them into the
   output panel at the bottom of the window.

Secrets enter through the GUI's `SecretLineEdit` widget (a masked
text field). The GUI passes them through the argv exactly as the CLI
expects: e.g. `--passphrase <plaintext>` for `mnemonic convert`, or
`--ms1 <ms1-string>` for `mnemonic bundle`. **The argv is therefore
visible to other processes on the same machine via `/proc/<pid>/cmdline`
(Linux) or equivalent.** This is a documented CLI surface property and
is not GUI-specific; treat it as you would any CLI invocation with a
secret on the command line.

## Version pinning {#version-pinning}

The GUI is built against specific CLI releases. `pinned-upstream.toml`
declares the exact tags this manual matches, and the toolkit's
installer (`scripts/install.sh`) installs exactly these:

| CLI | Pinned tag | Oldest release that works with this GUI |
|---|---|---|
| `mnemonic` (this toolkit) | `mnemonic-toolkit-v0.104.0` | the pinned tag — the GUI's forms mirror this release's flags, and no older release has been tested with this GUI |
| `md` | `descriptor-mnemonic-md-cli-v0.20.3` | the pinned tag — older releases lack flags the form offers (`--in`, `--experimental`, `address --from-mk1`) |
| `ms` | `ms-cli-v0.19.1` | `0.19.1` — `ms verify` with an `ms1` card and `--phrase` fails on `0.19.0` (`cannot read both ms1 and --phrase from stdin`) |
| `mk` | `mk-cli-v0.13.0` | the pinned tag — older releases lack flags the form offers (`encode --chunk-set-id`) |

Do not install these CLIs from crates.io: the copies there are several
releases older than every row above.

The GUI's schema (Dropdown value-sets, NodeValueComposite shapes,
flag inventories) is generated from each pinned CLI's source. The GUI
does **not** read your installed CLIs' versions (the `Pinned:` label
above each form is the version the GUI was built against), so compare
them yourself with `mnemonic --version`, `md --version`, `ms --version`
and `mk --version`. A mismatch shows up as either:

- A flag the GUI offers that the CLI rejects (CLI is older).
- A flag the CLI accepts that the GUI doesn't render (CLI is newer).

Both surface as the CLI's own error message in the output panel.
**Re-run the toolkit installer** (`install.sh`, or `install.sh --only
<cli>` for one CLI) to put the pinned releases in place.

:::primer
The pin is *which CLI release* the GUI knows about, not *which
features your wallet uses*. A newer CLI usually works as long as the
flag set you click on overlaps. An older one is riskier: below the
minimums in the table, a whole class of runs fails, not just a flag.
The pin is most load-bearing when the schema changes: flag renames,
new variants in a dropdown, new subcommands.
:::

## What the GUI does **not** do

- It does not provide a network connection. Address derivation,
  policy verification, and engraving-bundle assembly are all
  offline operations; the GUI never speaks to a peer or a blockchain.
- It does not store form state between sessions by default. Watch-only
  form fields (network, template, account index) optionally persist
  via Phase 8 disk-state (`~/.config/mnemonic-gui/` on Linux, the
  OS-equivalent on macOS / Windows); secret-class fields are
  `#[serde(skip)]` and never serialize, ever. See
  [§14 Secret handling](#secret-handling) for the full type-level
  invariant.
- It does not modify the CLIs. If you find a CLI bug, the GUI is
  the wrong place to fix it; file the bug against the relevant
  sibling repository (`descriptor-mnemonic`, `mnemonic-secret`,
  `mnemonic-key`, or `mnemonic-toolkit`).
