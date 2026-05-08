# m-format Quick Start guide

> **⚠ DISCLAIMER — UNTESTED ALPHA SOFTWARE.** **This software has not yet been independently tested or audited. Do not use the m-format constellation to back up significant sums of money at this time — doing so is tantamount to asking to be rekt.** Use only with disposable amounts, on testnet, or for evaluation. Assume bugs until external review happens.

Newcomer-aimed onboarding for the m-format constellation. Sibling artifact to the reference manual at `docs/manual/`.

## Build

```sh
cd docs/quickstart
make pdf      # PDF (needs pandoc + xelatex + mermaid-filter on host)
make md       # concatenated GFM markdown
make lint     # markdownlint + cspell + lychee
make verify-examples MNEMONIC_BIN=… MD_BIN=… MS_BIN=…   # transcript drift check
```

## Contributor notes

Several configs are symlinked back to the reference manual (`../manual/`). The QuickStart's `.cspell.json` is a local file using cspell's `import` key for its own word-list extensions without mutating the manual's config.

The symlinks require `git config core.symlinks true` (default on Linux/Mac; off on some Windows installs). If you're on Windows, set this flag before checking out the repo.
