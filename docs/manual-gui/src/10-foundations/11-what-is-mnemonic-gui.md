# What is mnemonic-gui?

`mnemonic-gui` is a cross-platform desktop application that puts a
visual form-based interface over the four m-format constellation CLIs:
`mnemonic`, `md`, `ms`, and `mk`. Each CLI becomes a *tab* in the
GUI; each subcommand becomes a *form*; each flag becomes a *widget*
(text field, dropdown, checkbox, slot editor). When you click **Run**,
the GUI assembles the equivalent argv and invokes the underlying CLI
as a subprocess, streaming stdout and stderr back into an output panel.

The GUI is **not** a re-implementation. It runs the same binaries the
CLI manual documents — `mnemonic-toolkit v0.13.0`,
`descriptor-mnemonic-md-cli v0.5.0`, `ms-cli v0.2.1`, `mk-cli v0.3.1`
(per `pinned-upstream.toml`). Anything the CLI does, the GUI exposes
the same way; anything the GUI shows in a dropdown corresponds 1:1
to a CLI flag value.

:::primer
You can think of the GUI as a *thin overlay*: it has no business
logic of its own. The cryptography, the BIP-39 encoding, the BCH
checksums, the BIP-388 wallet-policy emission — all live in the
sibling CLI crates. The GUI's job is to make those operations
discoverable and approachable for users who don't want to memorise
flag names.
:::

## Three things the GUI does that the CLI cannot

The GUI is not just a form-renderer. Three GUI-specific features have
no CLI equivalent:

1. **Side-by-side tab strip.** The four CLIs each get a tab at the
   top of the window. If `md` isn't installed on `$PATH`, its tab
   greys out with a hover tooltip explaining how to install it.
   This makes the four-CLI constellation discoverable as one unit;
   the shell-side equivalent requires memorising four binary names.
2. **Run-confirm modal for secret-bearing invocations.** When the
   form contains any schema-`secret: true` flag (`--passphrase`,
   `--ms1`, `--bip38-passphrase`, `--share`, etc. — see
   [§14 Secret handling](#secret-handling) for the full list and
   the type-level never-persist invariant), the **Run** button does
   not invoke the subprocess immediately.
   Instead a modal lists the full argv as it will be passed to the
   subprocess, and asks for explicit **Run** / **Cancel**
   confirmation. **At v0.3.0 the modal renders secret-bearing argv
   tokens in plaintext** — see [§14 Secret handling](#secret-handling)
   Defense 2 for the full security implication and operational
   mitigation. Even with that gap the modal still guards against
   muscle-memory clicks on a form pre-populated from disk.
3. **`?` help-icons that deep-link into THIS manual.** Every
   Dropdown / NodeValueComposite / TaggedOrIndexed / repeating-field
   flag in the GUI renders with a `?` button next to its label;
   click → opens the manual at the anchor for that exact flag.
   No equivalent in the CLI's `--help` output (the CLI has tooltips,
   not deep-links).

## What this manual is, by chapter

The chapters following this one cover, in order:

- [Relation to the four CLIs](#how-the-gui-relates-to-the-four-clis)
  — how the GUI invokes them, where pinning matters, the
  subprocess-runner contract.
- [Bundle / card / slot mental model](#the-bundle-card-slot-mental-model)
  — the three engravable cards (`ms1`, `mk1`, `md1`), their
  cross-binding\index{cross-binding}, what a *slot* is on the
  multisig path.
- [Secret handling](#secret-handling) — paste-warn modal, run-confirm
  modal, on-exit zeroize sweep, OS-snapshot occlusion (the four
  GUI-side defenses).

If you already know the m-format constellation from the CLI manual,
[§12](#how-the-gui-relates-to-the-four-clis) is the only chapter
that adds GUI-specific load-bearing facts. If you don't, read all
four in order.
