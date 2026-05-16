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

## Version pinning

The GUI pins specific CLI versions at build time. `pinned-upstream.toml`
declares the exact tags this manual matches:

| CLI | Pinned tag | Crates.io version available |
|---|---|---|
| `mnemonic` (this toolkit) | `mnemonic-toolkit-v0.13.0` | n/a (binary-only) |
| `md` | `descriptor-mnemonic-md-cli-v0.5.0` | `md-cli 0.5.0` |
| `ms` | `ms-cli-v0.2.1` | `ms-cli 0.2.1` |
| `mk` | `mk-cli-v0.3.1` | `mk-cli 0.3.1` |

The GUI's schema (Dropdown value-sets, NodeValueComposite shapes,
flag inventories) is generated from each pinned CLI's source. A
mismatch between the installed CLI and the pinned one shows up as
either:

- A flag the GUI offers that the CLI rejects (CLI is older).
- A flag the CLI accepts that the GUI doesn't render (CLI is newer).

Both surface as the CLI's own error message in the output panel.
**Re-install the CLI at the pinned tag** to resolve.

:::primer
The pin is *which CLI release* the GUI knows about, not *which
features your wallet uses*. You can still use the GUI to drive an
older or newer CLI as long as the flag set you actually click on
overlaps. The pin is most load-bearing when the schema changes:
flag renames, new variants in a dropdown, new subcommands.
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
