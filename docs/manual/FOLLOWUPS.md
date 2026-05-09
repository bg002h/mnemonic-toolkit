# Manual FOLLOWUPS

Manual-local deferred-work tracker. Closes lockstep with toolkit
release cadence; every entry resolves into either a manual revision or
a confirmed retirement. Mirrors the FOLLOWUPS pattern used in
`mnemonic-toolkit/design/FOLLOWUPS.md` and the sibling repos.

## Open

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

### Volvelle retirement / codex32-incorporation cycle (manual-v0.1.8)

The following 3 v0.2-candidate entries are closed in lockstep by the volvelle retirement / codex32-incorporation cycle (manual-v0.1.8 release): `bottom-disc-cell-density`, `bottom-disc-registration-tick-radius`, `volvelle-hand-computation-toolkit-gap`. The retirement is documented in `docs/manual/src/60-appendices/65-bch-codex-primer.md` §"Hand-decodability".

**Resolution rationale (audit summary).** Audit of codex32's 2023-03-07 hand-computation document (https://www.secretcodex32.com/docs/2023-03-07--color.pdf) revealed that the v0.1 paper-computer wheels under `docs/volvelles/` were structurally insufficient for hand-decodability: the 32×32 polymod-step grid exposed only the LOW-5-bits of one polymod step and discarded the upper 55–65 bits of state needed to carry forward. Codex32's actual hand-computation works through a 1024-entry Checksum Table + triangular Worksheet + Addition wheel — not a polymod wheel. A v0.2 derivative for mk1 + md1 would have required 6 pages of dense per-format lookup tables plus worksheets — substantial work for cards (xpub + origin; wallet policy) that carry no secret material and where hand-decodability is therefore not load-bearing. ms1 — the only secret-material card — uses BIP-93 codex32 directly and is fully covered by the upstream PDF, now bundled at `docs/codex32/2023-03-07--color.pdf`. The mk1/md1 wheels (volvelle-v0.1.0 through volvelle-v0.1.5 GitHub releases) are deleted.

[Original entries preserved verbatim below for audit trail; their "v0.2 fix" / "v0.2 deliverable" sections are superseded by this closure.]

#### `bottom-disc-cell-density` (was v0.2 candidate)

The v0.1 wheels under `docs/volvelles/` compress the bottom disc's
32×32 cell grid via TikZ `scale=0.31` so the wheel fits letter
paper. This yields ~0.108" effective cell pitch, well below the
original codex32 volvelle spec's 0.35" minimum. Cells are
technically printable but require fine motor skill to scissor-cut
accurately. Acceptable for a v0.1 reference deliverable; not for
production hand-operation.

**v0.2 fix.** Redesign the bottom disc for tabloid-or-larger paper
size, or adopt a different state-encoding convention that doesn't
require a 32×32 cell grid at all (e.g., factor the state into two
smaller wheels). The choice depends on whether wheel-pair-per-card
operation is acceptable; if not, tabloid is the simpler path.

**Where:** `docs/volvelles/{mk-regular,mk-long,md-regular}.tex`.

#### `bottom-disc-registration-tick-radius` (was v0.2 candidate)

The v0.1 wheels' registration alignment ticks render at radius
`3.65–3.85in` in unscaled TikZ coordinates; under the
letter-paper `scale=0.31` they land in the disc interior
(~1.13–1.19" from center) instead of at the outer rim where they
would be most useful for visual alignment verification.

**v0.2 fix.** Move the registration ticks outside the cell grid
so they sit at the disc's outer rim post-scaling. Requires either
re-parameterizing the tick radii against the bottom-disc outer
boundary or reflowing the layout once `bottom-disc-cell-density`
is addressed (the two FOLLOWUPS interact).

**Where:** `docs/volvelles/{mk-regular,mk-long,md-regular}.tex`,
the registration-tick block.

#### `volvelle-hand-computation-toolkit-gap` (was v0.2 candidate)

The v0.1 wheels under `docs/volvelles/` ship the 32×32 polymod-step
cell grid only. Reference: codex32's
`https://www.secretcodex32.com/docs/2023-03-07--color.pdf` ("Codex 32:
A Shamir Secret Sharing Scheme", Curr & Snead, 2022, MIT-licensed). Its
hand-computation toolkit comprises:

- **codex32 Checksum Table** (2 pages) — 1024 entries indexed by 2-char
  bech32 pairs, each output is 13 bech32 chars. The actual BCH polymod
  step lookup; consumed by the Checksum Worksheet.
- **Checksum Worksheet** (generation + verification) — triangular
  grid; user copies share data into the top diagonal, looks up
  2-char pairs in the Checksum Table to fill rows, adds adjacent rows
  pairwise, verifies that the bottom diagonal equals `SECRETSHARE32`.
- **Addition wheel/table** — GF(32) XOR for adjacent-row addition.
- **Bech32 ↔ Binary conversion table** — 32 chars ↔ 5-bit values.
- **Translation, Recovery, Fusion volvelles** — for Shamir share
  arithmetic, not polymod. (Codex32 has *no* polymod wheel; the
  Checksum Table replaces it.)

Our v0.1 wheel answers exactly *"for state with top-5-bits b and input
char c, what's the LOW-5-bits of the polymod output?"* — necessary
but not sufficient: the polymod state is 60 bits (regular code) or
70 bits (long code), and the wheel discards the upper bits the user
needs to carry forward. As a result, **the v0.1 wheel cannot be used
to hand-compute or hand-verify an mk1 / md1 string in isolation.**

**v0.2 deliverable** (per-format, ×3 because mk1-regular, mk1-long,
and md1-regular have different generator polynomials and target
residues):

1. **Per-format Checksum Table.** ~1024 × 13 chars (regular) or
   × 15 chars (long). Two pages each. Enables 2-char-chunk polymod
   lookup matching the codex32 worksheet model.
2. **HRP-prefix encoding card.** A 5-symbol preamble derived from
   BIP-173 HRP expansion (`[3,3,0,13,11]` for `mk`, `[3,3,0,13,4]`
   for `md`) the user feeds before the data part. Codex32 doesn't
   need this on the worksheet because `MS32_CONST` includes the
   `ms` HRP; our `MK_*_CONST` / `MD_REGULAR_CONST` likewise include
   the HRP, so the preamble must be explicit.
3. **Per-format target-residue card.** The 13/15-char bech32-rendered
   target the worksheet's bottom diagonal must equal — analogous to
   codex32's `SECRETSHARE32` text. Today our `\selftestlegend` carries
   the residue as a hex literal but not as the comparable
   per-character string.
4. **Addition wheel or table.** GF(32) XOR. Format-independent — one
   artifact for all three formats.
5. **Checksum Worksheet template.** Triangular grid sized for our
   string lengths (mk1 single-chunk ~52 chars / chunked ~96–108 chars;
   md1 ~22+ chars), with `+` / `=` markers and printed bottom-diagonal.
6. **Bech32 ↔ binary conversion reference.** Codex32 prints this on
   the front-matter; we don't ship it on the wheel pages.

**Interaction with sibling FOLLOWUPS.** This entry partially
supersedes both `bottom-disc-cell-density` and
`bottom-disc-registration-tick-radius`: if v0.2 ships a full
worksheet-and-table toolkit, the polymod wheel becomes optional
companion-art, and the cell-density / tick-radius constraints stop
being load-bearing. An honest v0.2 plan should pick one of:
(a) ship the worksheet+table toolkit and demote the wheels to
optional decorative companions; (b) keep wheels primary and accept
hand-decodability remains aspirational; (c) hybrid — ship a
cell-density-fixed wheel + the worksheet+table toolkit, with the
wheel as a faster lookup for the polymod-step row of the worksheet.
Decision deferred to the v0.2 spec phase.

**Where:** new `docs/volvelles/checksum-tables/{mk-regular,mk-long,md-regular}.tex`
(or `.pdf`); new `docs/volvelles/checksum-worksheet.tex`; new
`docs/volvelles/addition.tex`; updates to the per-format wheel files
to cross-reference the worksheet; new appendix subsection in
`docs/manual/src/60-appendices/65-bch-codex-primer.md` documenting
the hand-computation procedure end-to-end.

### `mk-cli` — Resolved by mk-cli-v0.2.0

**Filed:** 2026-05 (v0.1 cycle, asymmetric-CLI gap)
**Resolved by mk-cli-v0.2.0** — 2026-05-08.

`mk-cli` shipped as `crates/mk-cli/` in `bg002h/mnemonic-key` with the
five planned subcommands (`encode`, `decode`, `inspect`, `verify`,
`vectors`) per spec, plus the `--from-md1` cross-repo policy-id-stub
derivation. The manual's chapter 44 (`44-mk-codec-rust.md`) was
replaced with `44-mk-cli.md` (CLI surface; Rust API reference archived
to `mnemonic-key/docs/MK_CODEC_RUST_API.md`). Frontmatter,
foundations, install chapter, glossary, release-history, quickstart
recovery + watch-only chapters, ultraquickstart, READMEs, and
`CLAUDE.md` all updated to the four-CLI shape. `tests/lint.sh`
flag-coverage gate now treats `mk` as a 4th CLI; `cli-subcommands.list`
carries 5 new lines. The `manual-cli-surface-mirror` invariant extends
from 3 CLIs to 4.

### `pandoc-highlighting-macros-leaked-to-pdf` — closed by manual-v0.1.1

**Filed:** 2026-05-08 (user-reported)
**Resolved:** 2026-05-08 (manual-v0.1.1 patch release)

`pandoc/preamble.tex` redefined the `Highlighting` Verbatim environment without `commandchars=\\\{\}`. Pandoc's syntax-highlighting commands (`\ExtensionTok`, `\NormalTok`, `\AttributeTok`, `\DataTypeTok`, `\OperatorTok`, etc.) require commandchars on the Verbatim environment to expand inside the block; without it, they ship to the PDF as literal text (visible e.g. inside the watch-only multisig command in chapter 32 and the BIP-32 primer in chapter 63 of the published `manual-v0.1.0` PDF).

Fix: added `commandchars=\\\{\},` as the first option in `\DefineVerbatimEnvironment{Highlighting}{Verbatim}{...}`. Verified post-fix: `pdftotext` on the rebuilt PDF returns zero `*Tok` raw-text leaks; rendered code blocks now show clean syntax-highlighted output. Manual PDF page count dropped 129pp → 121pp (the leaked macro names had been bloating the layout).

### `cspell-dictionary-curation` — closed by v0.1 cycle

### `custom-volvelles-per-card-type` — closed by volvelle-v0.1.0

**Filed:** v0.1 cycle (manual Appendix E §"Hand-decodability with the codex32 volvelle")
**Resolved:** 2026-05-08 (volvelle-v0.1.0 release)

`docs/volvelles/{mk-regular,mk-long,md-regular}.pdf` ship as
first-cut HRP-specific wheels for the m-format constellation
NUMS-derived BCH residues — physically the same generator-polynomial
and field-arithmetic geometry as the stock codex32 volvelle, with
the endpoint marks shifted to land on `MK_REGULAR_CONST`,
`MK_LONG_CONST`, and `MD_REGULAR_CONST` respectively. Appendix E
now cross-links these from the §"Hand-decodability" subsection.

**Carry-overs to v0.2 — RETIRED.** The v0.1 deliverable was a reference / first-cut. The original two carry-overs and a third FOLLOWUP filed mid-cycle were all closed in lockstep by the volvelle retirement / codex32-incorporation cycle (manual-v0.1.8); see the unified closure note at the top of this *Closed* section for the audit summary and the verbatim original entries.
