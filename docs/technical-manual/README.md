# m-format Constellation Technical Manual

Source tree for the **technical manual** — the single-volume, two-half (implementer/auditor + Rust integrator) reference covering wire formats, address derivation, bundle formation, and the Rust API surface across all four sibling repos of the m-format constellation.

Companion to `docs/manual/` (the end-user manual). Same pandoc + xelatex + mermaid pipeline pattern; separate source tree, separate output PDF (`m-format-technical-manual.pdf`).

## Layout

```
.
├── README.md                       ← you are here
├── AUTHORING.md                    ← conventions for chapter authors
├── FOLLOWUPS.md                    ← deferred items tracker
├── Makefile                        ← build targets
├── Dockerfile.build                ← pinned pandoc + xelatex + makeindex
├── pandoc/                         ← filters, templates, metadata
├── figures/                        ← mermaid sources + pre-rendered SVG cache
├── tests/                          ← lint.sh, verify-examples.sh, fixtures
├── transcripts/                    ← captured CLI / API transcripts
├── examples/                       ← runnable Rust examples (Part V)
└── src/                            ← one chapter per markdown file
    ├── 00-frontmatter.md
    ├── 00-disclaimer.md
    ├── 10-foundations/             ← Part I
    ├── 20-wire-formats/            ← Part II
    ├── 30-address-derivation/      ← Part III
    ├── 40-bundle-formation/        ← Part IV
    ├── 50-rust-api/                ← Part V
    └── 60-back-matter/             ← glossary, index, release-history, etc.
```

## Status

Building from `tech-manual-v0.1` onward. See `design/SPEC_tech_manual_v1.md` and `design/IMPLEMENTATION_PLAN_tech_manual_v1.md` for scope, sizing, and cut decomposition.

## Build

```sh
make pdf            # build/m-format-technical-manual.pdf (needs pandoc + xelatex)
make md             # build/m-format-technical-manual.md (concatenated GFM)
make pdf-docker     # same as `make pdf`, inside the pinned Dockerfile.build
make lint           # markdownlint + cspell + lychee + api-surface-coverage hint + glossary + index
make verify-examples # re-run worked-example transcripts against locally-built CLIs
```

## Licence

MIT (matches the parent toolkit).
