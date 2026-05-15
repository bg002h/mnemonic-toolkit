# About this manual

`mnemonic-gui`\index{mnemonic-gui} is the cross-platform desktop GUI
overlay for the m-format constellation\index{m-format constellation} —
the four-CLI family (`mnemonic`, `md`, `ms`, `mk`) that engraves
together as a coherent steel-engravable Bitcoin self-custody backup
bundle. The GUI exposes every CLI surface as a form, runs the
underlying binary as a subprocess, and pipes stdout / stderr back into
an output panel. **This manual is the end-user companion to the GUI**;
for the CLI surfaces themselves see the companion `mnemonic-toolkit`
manual.

> **MIT License.** This manual is freely redistributable under MIT
> terms. See the `LICENSE` file in the source repository.

This is **manual-gui v1.0**, tracking `mnemonic-gui v0.3.0` and the
CLI versions that release pinned (toolkit `v0.13.0`, md-cli `v0.5.0`,
ms-cli `v0.2.1`, mk-cli `v0.3.1`). The manual is a *living document*:
each `manual-gui-v*` tag attaches a fresh PDF asset to the
corresponding GitHub release and pushes the rendered HTML to
`https://bg002h.github.io/mnemonic-toolkit/manual-gui/`.

## Who is this manual for?

The manual is **two-track**:

- **Bitcoin power users** — readers familiar with BIP-39 mnemonic
  phrases, BIP-32 derivation paths, descriptors, and multisig — read
  the main flow of every chapter and skip the `:::primer` boxes.
- **Newcomers** — readers who know what a Bitcoin wallet is but have
  not yet used the GUI, do not yet know what BIP-388 is, or have
  never set up multisig — read the `:::primer` boxes for the
  background a chapter assumes, and consult the CLI manual's
  Part VI appendices for full primers.

If you do not yet know what BIP-39, BIP-32, descriptors, or multisig
*are*, start with the foundational concepts in the CLI manual's
Appendix B–D primers, then return here to learn the GUI affordances.

## How to read this manual

The chapters are divided into six parts:

| Part | Chapters | Read order |
|---|---|---|
| I — Foundations | What `mnemonic-gui` is; relation to the four CLIs; the bundle/card/slot mental model | Read first. |
| II — Install | Linux / macOS / Windows install + wgpu/egui graphics-stack notes | Read next; get the binary on `$PATH`. |
| III — Tour | First-launch walkthrough: pick a tab, render a form, hit Run, read output panel | Hands-on; ~10 minutes from cold start to first `mnemonic bundle` run. |
| IV — Per-tab reference | `mnemonic`, `md`, `ms`, `mk` tabs | The bulk. Each tab is one chapter; each subcommand is a section; each Dropdown / NodeValueComposite / repeating flag is anchor-addressable. The GUI's `?` help-icons deep-link directly into these sections. |
| V — Troubleshooting | Common errors; binary-missing diagnostic; OS-snapshot warnings; secret-occlusion advisories | Reference; consult when something goes wrong. |
| VI — Appendices | Glossary; per-flag index; per-Dropdown enumeration reference; release history | Mix of reference and primer material. |

Every example in this manual uses the canonical BIP-39 test vector
`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`.
This phrase is **public** and any wallet derived from it has been
swept by chain watchers. **Never engrave it. Never fund it.** Each
chapter that uses it opens with a `:::danger` admonition restating
this.

## The help-icon contract

Every Dropdown / NodeValueComposite / TaggedOrIndexed / repeating-field
flag in the GUI renders with a `?` button next to its label. Clicking
that button opens this manual in your default browser at the anchor
for that exact flag — `https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from`
for the `--from` dropdown on `mnemonic convert`, for example. Per-
subcommand `?` buttons live next to the subcommand selector dropdown
at the top of each tab. The anchor scheme is documented in
Appendix C (per-flag index) and lint-enforced via bidirectional
schema-coverage checks.

## Versioning and releases

The manual is a *living document*. Each `manual-gui-v*` GitHub tag
attaches a freshly-built PDF asset
(`m-format-gui-manual-${MANUAL_GUI_VERSION}.pdf`) and re-deploys the
HTML render to GitHub Pages. A breaking content change to the manual
itself bumps `manual-gui-MAJOR`; new chapters bump
`manual-gui-MINOR`; copyedits and typo fixes bump
`manual-gui-PATCH`. Independent of `mnemonic-gui` semver — a new
`mnemonic-gui` release may or may not coincide with a manual revision.
