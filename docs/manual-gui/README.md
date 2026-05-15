# mnemonic-gui user manual

> **⚠ DISCLAIMER — UNTESTED ALPHA SOFTWARE.** **This software has not yet been independently tested or audited. Do not use the m-format constellation to back up significant sums of money at this time — doing so is tantamount to asking to be rekt.** Use only with disposable amounts, on testnet, or for evaluation. Assume bugs until external review happens.

Authored sources, build pipeline, and lint scaffolding for the
**`mnemonic-gui` end-user manual** — a cross-platform egui overlay
over the four CLIs (`mnemonic`, `md`, `ms`, `mk`).

This is the GUI-shaped companion to the CLI manual under
`../manual/`. The two manuals are independent build units: the CLI
manual reflects the clap-derive `--help` output of all four CLIs;
this manual reflects `mnemonic-gui`'s `SubcommandSchema` arrays in
`mnemonic-gui/src/schema/{mnemonic,md,ms,mk}.rs`.

Status: **v1.0 in development.** First tag will be `manual-gui-v1.0.0`.

## How the manual relates to the GUI

Every help-icon (`?`) button in `mnemonic-gui` deep-links to a
section in this manual via stable kebab-case anchors. The manual is
shaped like the GUI: each tab → chapter, each subcommand → section,
each dropdown → outline with one link per variant. The two artifacts
ship in lockstep — a `gui-schema-coverage` lint phase fails CI if
the GUI's SubcommandSchema and the manual's anchors diverge.

See `/home/bcg/.claude/plans/eager-giggling-castle.md` for the
manual-gui v1.0 cycle plan (sections, lint gates, two-PR landing).

## Building the manual

The canonical build is dockerized for reproducibility:

```sh
make pdf-docker
```

This produces `build/m-format-gui-manual.pdf`. The Dockerfile pins
pandoc, texlive (xelatex + makeindex + xurl + seqsplit), and the
markdown lint toolchain.

For local iteration without Docker:

```sh
# Arch / CachyOS:
sudo pacman -S pandoc-cli texlive-xetex texlive-binextra
npm install -g markdownlint-cli2 cspell
cargo install lychee

# Then:
make md       # build/m-format-gui-manual.md
make pdf      # build/m-format-gui-manual.pdf
make html     # build/m-format-gui-manual.html (for gh-pages deploy)
make lint     # markdownlint + cspell + lychee + gui-schema-coverage + outline-coverage + glossary-coverage
make filter-smoke   # sanity check on the pandoc filter pipeline
```

## Source layout

| Range | Purpose |
|---|---|
| `00-*` | Frontmatter + disclaimer |
| `10-foundations/` | What the GUI is, its relation to the 4 CLIs, the bundle/card/slot concept, secret-handling model |
| `20-install/` | Cross-platform install (Linux/macOS/Windows binaries from GitHub releases), wgpu/egui graphics-stack notes |
| `30-tour/` | First-launch walkthrough |
| `40-mnemonic/` | Reference for the `mnemonic` tab (10 subcommands) |
| `50-md/` | Reference for the `md` tab (8 subcommands) |
| `60-ms/` | Reference for the `ms` tab (5 subcommands) |
| `70-mk/` | Reference for the `mk` tab (5 subcommands) |
| `80-troubleshooting/` | Common errors, binary-missing diagnostic, secret-occlusion advisories |
| `90-appendices/` | Glossary, per-flag index, per-dropdown enumeration reference, release history |

## License

MIT License. See [`../../LICENSE`](../../LICENSE).
