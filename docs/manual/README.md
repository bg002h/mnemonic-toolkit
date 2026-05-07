# m-format-star user manual

Authored sources, build pipeline, and lint scaffolding for the
m-format-star end-user manual. The manual covers all four sibling
formats: `mnemonic-toolkit` (binary `mnemonic`), `descriptor-mnemonic`
(`md-cli`), `mnemonic-secret` (`ms-cli`), and the no-CLI sibling
`mnemonic-key` (`mk-codec`, used as a Rust library).

Status: **v0.1 in development.** First tag will be `manual-v0.1.0`.

## Building the manual

The canonical build is dockerized for reproducibility:

```sh
make pdf-docker
```

This produces `build/m-format-manual.pdf`. The Dockerfile pins pandoc,
texlive (xelatex + makeindex), mermaid-cli + mermaid-filter, and the
markdown lint toolchain.

For local iteration without Docker, install the pieces yourself:

```sh
# Arch / CachyOS:
sudo pacman -S pandoc-cli texlive-xetex texlive-binextra
npm install -g @mermaid-js/mermaid-cli mermaid-filter markdownlint-cli2 cspell
cargo install lychee
```

Then:

```sh
make md       # build/m-format-manual.md
make pdf      # build/m-format-manual.pdf
make lint     # markdownlint + cspell + lychee + flag-coverage + glossary-coverage + index check
make filter-smoke   # Phase 0 sanity check on the pandoc filter pipeline
```

## Source layout

See `design/IMPLEMENTATION_PLAN_user_manual_v0_1.md` (in the toolkit
repo root's `design/` directory) for the full source-tree layout, build
pipeline contract, and phase-by-phase execution plan.

The two highest-leverage authoring conventions:

1. **Two-track convention.** Power-user prose lives at the top level
   of each chapter; newcomer-friendly background goes inside fenced
   `:::primer` divs (rendered as boxed sidebars in PDF, blockquotes in
   markdown). Deep newcomer primers are in Part VI appendices.
2. **Index markers.** Insert `\index{TERM}` immediately after a term's
   first definitional use. The markdown render path strips the
   markers; the PDF render path picks them up for a page-numbered
   index. The `tests/lint.sh` `index bidirectional` check enforces
   that every `\index{TERM}` has a matching row in
   `src/60-appendices/69-index-table.md` and vice versa.

## Worked-example seed

All worked examples use the canonical BIP-39 test vector
`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`
— which is **public** and **swept**. Every chapter that introduces an
example seed must open with a `:::danger` admonition warning the
reader.

## License

CC0 1.0 Universal (Creative Commons Public Domain Dedication). See
[`../../LICENSE`](../../LICENSE).
