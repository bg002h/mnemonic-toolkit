# Manual FOLLOWUPS

Manual-local deferred-work tracker. Closes lockstep with toolkit
release cadence; every entry resolves into either a manual revision or
a confirmed retirement. Mirrors the FOLLOWUPS pattern used in
`mnemonic-toolkit/design/FOLLOWUPS.md` and the sibling repos.

## Open

### `mk-cli` — v0.2+ candidate

mk-codec currently ships as a Rust library only (no standalone CLI),
making it the asymmetric sibling among the four formats: `mnemonic`
(integration CLI), `md-cli`, and `ms-cli` are command-line tools, but
the only access to mk-codec is in-code via the Rust crate. The manual
acknowledges this in chapter 44 (`44-mk-codec-rust.md`), which
documents the Rust API surface rather than a CLI.

A future `mk-cli` would close the asymmetry by exposing mk-codec's
encode / decode / inspect / verify / vectors surface as a binary,
parallel to md-cli's shape. End users who want to verify or recover
an mk1 plate from a hardware-air-gapped machine without writing Rust
would have a one-line install path.

**Why it matters.** Without mk-cli, recovery from an mk1 plate
requires either (a) running `mnemonic convert --from mk1=... --to
xpub --to fingerprint --to path` (which works but bundles the entire
toolkit, including all secret-material code paths the user may not
want on a recovery machine), or (b) writing Rust against `mk-codec`
directly. A standalone mk-cli with no secret-material dependencies
would be the cleanest minimal-surface tool for mk1 plate recovery.

**Where it ships.** `crates/mk-cli/` in the `bg002h/mnemonic-key`
repo, following the same crate-extraction pattern md-cli used.

**Cross-repo / cross-doc impact when mk-cli ships.** Concrete
touch-list for whoever picks this up — every item below currently
states or implies "mk-codec is library-only" and will need updating
when that's no longer true:

*Manual side (`docs/manual/`):*

- `src/00-frontmatter.md`: prose that lists the four siblings (currently labels mk-codec as "no CLI") — update.
- `src/10-foundations/11-welcome.md`: same prose pattern in the foundations chapter — update.
- `src/40-cli-reference/44-mk-codec-rust.md`: either replace with `44-mk-cli.md` (CLI surface) or split into two chapters (Rust API + CLI). Decide based on whether the Rust API stays user-facing post-CLI or becomes internal.
- `tests/lint.sh` glossary-coverage / flag-coverage steps: extend the term and flag enumerations to include mk-cli's surface so the lint catches drift.

*QuickStart side (`docs/quickstart/`):*

- `src/20-singlesig/26-recover.md`: the mk1-recovery step currently uses `mnemonic convert --from mk1=… --to xpub --to fingerprint --to path`. With mk-cli available, either swap to the minimal-surface form (`mk-cli decode …`) or document both paths.
- `src/40-watch-only/41-singlesig-watch-only.md` + `42-multisig-watch-only.md`: any prose framing of "mk-codec is library-only" — update.

*UltraQuickStart (`docs/ultraquickstart.md`):*

- The 5-line "Recover" section uses `mnemonic convert --from mk1=…`. Probably the cleanest single-line replacement is `mk-cli decode <mk1-string>` for the air-gapped public-card recovery use case.

*Top-level (`README.md`, `CLAUDE.md`, `crates/mnemonic-toolkit/README.md`):*

- The 3- or 4-row sibling tables: change mk-codec's row from "library-only" to "library + mk-cli".
- CLAUDE.md `## Manual coverage` section lists the CLIs the manual mirrors — extend to include mk-cli.
- The `manual-cli-surface-mirror` invariant currently documented in CLAUDE.md / FOLLOWUPS.md applies to 3 CLIs; will extend to 4. The toolkit-side `tests/lint.sh flag-coverage` gate adds an mk-cli row.

This entry's punch-list assumes the mk-cli surface is roughly: `mk-cli encode` (xpub + origin → mk1), `mk-cli decode` (mk1 → xpub + origin), `mk-cli inspect` (decoded fields with policy_id_stub display), `mk-cli verify` (round-trip check), `mk-cli vectors` (test-vector dump). Adjust the docs touch-list when the actual mk-cli SPEC lands.

### `custom-volvelles-per-card-type` — v0.2+ candidate

The codex32 volvelle (printable paper-computer wheel) at
[BlockstreamResearch/codex32](https://github.com/BlockstreamResearch/codex32)
hand-decodes ms1 strings directly, but mk1 and md1 use HRP-mixed
target residues (`MK_REGULAR_CONST`, `MK_LONG_CONST`,
`MD_REGULAR_CONST`) that the stock wheel doesn't recognise at the
endpoint comparison step. Per Appendix E §"Hand-decodability with
the codex32 volvelle", this can be worked around by printing a
small target-residue card and doing the final XOR manually.

A more elegant solution would be to publish **HRP-specific
volvelles per card type** — physically the same wheel as the
stock codex32 volvelle (same generator polynomial, same field
arithmetic), but with the endpoint marks shifted to land on the
mk1 or md1 target residues for a valid string.

**Why it matters.** The constellation's hand-recovery story is
currently graded — ms1 fully volvelle-decodable, mk1/md1 partially.
A custom volvelle per card would restore the all-on-the-wheel
property for the public cards too, supporting the "no electronics
needed at restoration time" use case end-to-end.

**Where to ship.** A `volvelles/` directory in the toolkit (or
sibling) repo with three printable PDFs (mk1-regular, mk1-long,
md1-regular). Long-code wheels are larger but follow the same
template. The PDFs would carry a CC0 dedication matching the
BlockstreamResearch source material.

**Dependencies.** None — purely a graphic-design / print-layout
deliverable. Could be done by anyone with TikZ / Inkscape capability
and no Rust experience.

### `bch-string-length-empirical-sweep` — v0.2 candidate

The "Typical string length" column of Appendix E's per-card table
(`docs/manual/src/60-appendices/65-bch-codex-primer.md` §"How the
code variant is chosen per card") gives best-read estimates:

- ms1 BIP-39 entropy 16–32 B → ~70 chars
- mk1 single-string mode → ~52–55 chars
- mk1 chunked mode → 96–108 chars
- md1 wallet policy → 75–93 chars

These have not been empirically swept across all payload variants
(network, template, cosigner count, path family, BIP-388 multipath
form, etc.). v0.2 should encode known fixtures spanning all
combinations and report a precise length range per card + payload
kind, replacing the qualifier prose with measured data.

**Where to add the sweep:** `docs/manual/tests/` is the natural
home for a `bch_string_length_sweep.sh` that drives the four CLIs
across a fixture matrix and emits a CSV. The CSV can then update
the table in `65-bch-codex-primer.md` directly.

**Why it matters:** the regular-vs-long code variant choice depends
on string length crossing 93 chars. If a real-world mk1 payload
(e.g., a deeply-nested derivation path) ever lands exactly at 93 /
94 / 95 chars, the encoder behavior at the boundary is currently
unverified by the manual — only by code review. An empirical sweep
closes that gap.

### `release-history-auto-extract` — v0.2 candidate

`src/60-appendices/68-release-history.md` is hand-authored for v0.1.
For v0.2, replace with an auto-extraction script
(`tools/digest-changelogs.sh`) that reads each of the four sibling
repos' `CHANGELOG.md` files and emits a per-repo prose summary keyed
by tag.

**Why:** four repos × per-tag updates = manual diff drift if maintained
by hand. Auto-extraction localises the toil to a single script.

**How to apply:** stage during a v0.2 cycle when there is also a
material content edit elsewhere; do not gate v0.2 on the script alone.

### `figures-cache-implementation` — v0.2 candidate

`make figures-cache` is currently a stub. It should pre-render every
` ```mermaid ` block under `src/` to SVG, key the cache file by
SHA-256 of the source block, and write into `figures/cache/`. The
`MERMAID_FILTER=skip` mode then consumes the cache to support builds
on hosts where `mermaid-filter` (Chromium) is unavailable.

**Why:** Chromium / Puppeteer is a large dependency. Some contributors
on minimal Linux images can't render mermaid live. Pre-rendered cache
lets them still produce a PDF.

**How to apply:** v0.2; not blocking v0.1.

### `npm-package-pinning` — v0.2 candidate

`Dockerfile.build` uses `^` semver ranges on npm packages. For full
reproducibility, pin to exact versions and check in a lockfile
(`package-lock.json` in `docs/manual/`).

**Why:** semver-range installs aren't bit-reproducible across rebuilds.

**How to apply:** introduce `package.json` + `package-lock.json` in v0.2.

### `cspell-dictionary-curation` — closed for v0.1

`cspell` lint will produce many false positives until a project
dictionary is maintained. **Closed:** `docs/manual/.cspell.json`
ships with the v0.1 curated wordlist (m-format vocabulary +
British-spelling forms used across chapters). Future additions
follow the natural pattern of editing this file when chapters add
new technical terms.

### `page-count-overflow-v0.1` — accepted exception

The v0.1 PDF builds at 129 pages, vs. the A2 acceptance criterion
of 60–100 pages. The plan permits "signed off explicitly" overflows.

**Why accepted:** v0.1 covers the entire CLI surface (~120 flags),
8 workflow chapters, 7 compare/contrast chapters, and 8 appendices —
the surface area drove the page count, not narrative bloat. Trimming
to 100pp would require cutting either the BIP-85 chapter, one
appendix primer, or compressing the workflow chapters past the
2-4pp-per-chapter ceiling. None of those reductions improves the
manual's utility.

**How to apply:** future minor manual revisions either accept the
new ceiling or scope-cut explicitly. v0.2 may revisit the layout
constants (font size, margins) for a tighter render.

### `derive-child-dice-out-of-scope-docstring` — toolkit-side

The `mnemonic derive-child --help` output (sourced from
`crates/mnemonic-toolkit/src/cmd/derive_child.rs`) still says
"3 out-of-scope tokens (`rsa`, `rsa-gpg`, `dice`)" — but v0.8
promoted `dice` to in-scope. The chapter 38 prose is correct;
the binary's docstring is stale. **Where:** mnemonic-toolkit
v0.8.x patch.

### `dice-length-cap-undocumented` — toolkit-side

Chapter 38's `--length` table for `dice` says `1..=10000`, but
the source enforces only `≥ 1` (no upper cap). Pick one: either
add a cap to the validator (with the documented limit) or drop
the upper bound from the chapter table. **Where:** mnemonic-toolkit
v0.8.x patch.

### `recovery-paths-bch-error-format-illustrative` — manual-side

`src/30-workflows/35-recovery-paths.md` shows an illustrative
BCH error output ("position 11: invalid character 'Q'"). The
format is illustrative, not transcribed from a binary run. Add
a paired `transcripts/35-recovery-bch-error.{cmd,out}` that
captures the actual error message format from a deliberately-
corrupted ms1 string. **Where:** manual v0.2.

### `format-sparrow-format-specter-stubs` — toolkit-side

`mnemonic export-wallet --format sparrow` and `--format specter`
are accepted at the clap level but return `ExportWalletFormatStub`
errors. v0.1 of the manual recommends `--format bip388` for both
receiving wallets. Either light up the stubs in a v0.8.x patch
(emitting Sparrow-/Specter-native JSON shapes) or change the clap
definition to remove the `sparrow` / `specter` choices entirely.

## Closed

### `pandoc-highlighting-macros-leaked-to-pdf` — closed by manual-v0.1.1

**Filed:** 2026-05-08 (user-reported)
**Resolved:** 2026-05-08 (manual-v0.1.1 patch release)

`pandoc/preamble.tex` redefined the `Highlighting` Verbatim environment without `commandchars=\\\{\}`. Pandoc's syntax-highlighting commands (`\ExtensionTok`, `\NormalTok`, `\AttributeTok`, `\DataTypeTok`, `\OperatorTok`, etc.) require commandchars on the Verbatim environment to expand inside the block; without it, they ship to the PDF as literal text (visible e.g. inside the watch-only multisig command in chapter 32 and the BIP-32 primer in chapter 63 of the published `manual-v0.1.0` PDF).

Fix: added `commandchars=\\\{\},` as the first option in `\DefineVerbatimEnvironment{Highlighting}{Verbatim}{...}`. Verified post-fix: `pdftotext` on the rebuilt PDF returns zero `*Tok` raw-text leaks; rendered code blocks now show clean syntax-highlighted output. Manual PDF page count dropped 129pp → 121pp (the leaked macro names had been bloating the layout).

### `cspell-dictionary-curation` — closed by v0.1 cycle
