# m-format Quick Start guide

Newcomer-aimed onboarding for the m-format star. Sibling artifact to the reference manual at `docs/manual/`.

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
