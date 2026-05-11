# m-format Constellation Technical Manual — v1.0 SPEC

| Field | Value |
|---|---|
| Document version | v1.0 (single-volume vision; releasable cuts are v0.1 → v1.0) |
| Date drafted | 2026-05-11 |
| Status | Drafting → architect review → user review → execution |
| Source repo | `mnemonic-toolkit` (`docs/technical-manual/`) |
| Companion to | `mnemonic-toolkit/docs/manual/` (end-user manual, v0.1.1) |
| Audience | (1) Implementer / auditor (re-implements or audits a sibling codec or the toolkit). (2) Rust integrator (consumes the codecs / toolkit as library deps). One volume, two halves. |
| First release | `tech-manual-v0.1.0` (Parts I + II + back-matter skeleton) |
| v1.0 target | All five Parts + populated glossary, populated index, full cross-references |
| Sizing target | ~200–300 PDF pages at v1.0 |
| Build artifact | `m-format-technical-manual.pdf` attached to the toolkit GitHub release tagged `tech-manual-vX.Y.Z` |

---

## §1 Goals

The end-user manual (`docs/manual/`) onboards users and walks them through workflows. It is intentionally thin on bit-level wire detail, BCH math, and Rust API surface. **The technical manual is the missing companion** — the single source-of-truth reference for someone who needs to know:

- **G1.** What is on the wire for an `md1` / `mk1` / `ms1` card, bit for bit?
- **G2.** How does the BCH error-correcting math work across the (forked) md1↔mk1 layer and the BIP-93 direct ms1 layer?
- **G3.** How does the toolkit turn a seed phrase into a coherent three-card bundle, and how do the anti-collision invariants compose?
- **G4.** How does address derivation work end-to-end (descriptor → miniscript → address), including the v0.32 shape-coverage extension for taproot-tree, NUMS, sh(multi), arbitrary `wsh(<miniscript>)`, and tap-leaf miniscript?
- **G5.** What is the public Rust API surface of `md-codec`, `mk-codec`, `ms-codec`, and `mnemonic-toolkit`? Feature flags. Error taxonomies. Integration patterns.
- **G6.** How do these answers fit together — i.e. the architectural map of the four-format star, and the rationale for the design boundaries (forked-BCH md1↔mk1 vs. BIP-93-direct ms1).

## §2 Non-goals

- **N1.** Not a BIP. BIPs for individual formats live in their respective repos. The technical manual cites BIPs; it does not replace them.
- **N2.** Not a tutorial. Workflow walkthroughs belong in the end-user manual. The technical manual is reference-shaped.
- **N3.** Not a marketing surface. READMEs and the end-user manual carry promotional framing; the technical manual is post-purchase, reference-grade.
- **N4.** Not a design-rationale archive. Per-version SPECs in each repo's `design/` remain authoritative for "why we did it this way at version X." The technical manual cites them; it does not duplicate them.
- **N5.** No HTML site in v0.1. PDF + concatenated markdown only (matching the end-user manual's v0.1 baseline). HTML deferred to a future cut or to the end-user manual's eventual mdBook upgrade.
- **N6.** No translations in v1.0. English only.

## §3 Track A — Doc-surface audit & update

Track A is **Phase 0** of the implementation plan, not a separate cycle. It precedes Track B because the technical manual reuses audited content from per-repo docs as raw material for prose and examples. Track A's deliverable is a set of small, surgical diffs across both repos.

### §3.1 Scope (files in audit)

**md1 repo (`/scratch/code/shibboleth/descriptor-mnemonic/`):**

- `README.md` — top-level overview. Verify "What's covered in v0" prose matches v0.30+v0.31+v0.32 reality; verify "Status" prose matches HEAD; verify example invocations still produce shown output.
- `crates/md-codec/README.md` — library doc. Verify error-taxonomy references match HEAD (`OperatorContextViolation` shipped in v0.30/v0.31, `Error::UnsupportedDerivationShape` removed in v0.32). Verify feature-flag prose matches `Cargo.toml` (v0.32 added `[features] default=["derive"]`).
- `crates/md-cli/README.md` — CLI doc. Verify subcommand surface against `--help` output. Verify any wire-format prose against v0.30+v0.31 SPEC.
- `MIGRATION.md` — verify v0.30 / v0.31 / v0.32 entries exist and are accurate. Add any missing entries.
- `CHANGELOG.md` — spot-check v0.30 / v0.31 / v0.32 entries against the SPEC and release commits; correct any drift.
- `docs/json-schema-v1.md` — verify JSON envelope shape against v0.31 `JsonTag` mirror; correct any drift.
- `bip/bip-mnemonic-descriptor.mediawiki` — v0.30 phase i did a full rewrite. Audit-only sweep for v0.31 (`OperatorContextViolation TopLevel`) and v0.32 (rust-miniscript-based address derivation references). No structural changes. **Hex / encoded-string examples in the BIP draft are re-verified against HEAD** using the `md` binary (`md decode`, `md inspect`, or `md vectors` as appropriate) — any literal that drifts is updated to match HEAD output. No new examples added.
- Inline `//!` and `///` doc-comments in `md-codec` public modules where they reference removed/renamed Error variants or pre-v0.30 wire layout. Surgical edits only; no structural reorg.

**toolkit repo (`/scratch/code/shibboleth/mnemonic-toolkit/`):**

- `docs/manual/src/40-cli-reference/42-md.md` — `md`-cli mirror chapter. Verify against `md --help`; refresh any flag/option that drifted under v0.30/v0.31/v0.32. (Mirror invariant is gated by `tests/lint.sh flag-coverage`; this audit closes any pre-existing drift.)
- `docs/manual/src/60-appendices/61-glossary.md` — terms that shifted under v0.30 (tag-space rework, NUMS flag, walker normalization). Add or correct entries; do not add unrelated terms.
- `docs/manual/src/60-appendices/64-descriptors-primer.md` — only the prose passages that explicitly reference shapes derive_address now covers. Replace any "not currently supported" framing with the v0.32 coverage; do not expand the chapter.

**Out of scope for Track A:**

- `docs/ultraquickstart.md` — explicitly skipped per the v0.32 scope decision; do not add wire-format detail.
- `docs/quickstart/src/**` — touched only if a wire-format reference already lives there and is now wrong. Track A does not add wire-format content to the quickstart.
- `m-format-manual.pdf` rebuild — not required at Track A close. Re-render flows naturally with the next user-manual cycle.
- Sibling-repo READMEs (`mnemonic-key/`, `mnemonic-secret/`) — neither was on the v0.30/v0.32 axis. Out of scope.
- Per-version SPECs in any `design/` folder — frozen artifacts; out of scope.

### §3.2 Audit method

For each in-scope file:

1. **Read HEAD source of truth** (the relevant `Cargo.toml`, `--help`, public-API doc comments, or the per-version SPEC).
2. **Compare against the doc file** — identify drifted prose, stale examples, removed-variant references.
3. **Emit a small diff** — only the changes required to match HEAD. No new sections. No expansion. Preserve existing voice and structure.
4. **Run any local verification** the doc declares (e.g., re-run example invocations to verify quoted output).

### §3.3 Track A acceptance

- Every claim in scope-files about wire format / error taxonomy / address-derivation coverage matches HEAD.
- Per-file diff is small and surgical (target: <50 lines net per file; flag any file that exceeds for scope review).
- `tests/lint.sh flag-coverage` passes for the `42-md.md` mirror chapter against `md --help` HEAD.
- `cargo build --workspace --all-features` and `cargo build --workspace --no-default-features` pass in both repos (cheap sanity check that doc-comment edits did not break compilation).
- Any hex / encoded-string examples in `bip/bip-mnemonic-descriptor.mediawiki` verified against HEAD `md` binary; drift corrected in-place.
- Track A produces one commit per repo with a `docs(audit-v0.32):` prefix; no version-bump tags fire.

## §4 Track B — Technical manual v1.0

The new artifact: a single-volume PDF + concatenated markdown, structured into five Parts plus back matter.

### §4.1 Source location and layout

```
mnemonic-toolkit/docs/technical-manual/
├── README.md                    ← repo-relative pointer + status
├── AUTHORING.md                 ← conventions (cloned + adapted from docs/manual/)
├── FOLLOWUPS.md                 ← deferred items
├── Makefile                     ← build targets (md, pdf, pdf-docker, lint, verify-examples)
├── Dockerfile.build             ← pinned pandoc + xelatex + mermaid + makeindex
├── pandoc/                      ← templates, filters, preamble, metadata
│   ├── filters/
│   ├── metadata.yaml
│   ├── preamble.tex
│   └── templates/
├── figures/                     ← mermaid sources + pre-rendered SVG cache
├── tests/                       ← lint.sh, verify-examples.sh, fixtures
├── transcripts/                 ← captured CLI / API transcripts
└── src/                         ← one chapter per markdown file
    ├── 00-frontmatter.md
    ├── 00-disclaimer.md
    ├── 10-foundations/
    │   ├── 11-introduction.md
    │   ├── 12-the-m-format-star.md
    │   ├── 13-codex32-and-bch.md
    │   └── 14-conventions-and-notation.md
    ├── 20-wire-formats/
    │   ├── 21-md1-wire-format.md
    │   ├── 22-mk1-wire-format.md
    │   └── 23-ms1-wire-format.md
    ├── 30-address-derivation/
    │   ├── 31-descriptor-to-miniscript.md
    │   ├── 32-shape-coverage.md
    │   └── 33-network-and-addressing.md
    ├── 40-bundle-formation/
    │   ├── 41-bundle-anatomy.md
    │   ├── 42-anti-collision-invariants.md
    │   └── 43-future-shares.md
    ├── 50-rust-api/
    │   ├── 51-md-codec-api.md
    │   ├── 52-mk-codec-api.md
    │   ├── 53-ms-codec-api.md
    │   └── 54-mnemonic-toolkit-api.md
    ├── 60-back-matter/
    │   ├── 61-glossary.md
    │   ├── 62-index-table.md
    │   ├── 63-release-history.md
    │   ├── 64-bip-cross-reference.md
    │   ├── 65-troubleshooting.md
    │   └── 66-bibliography.md
    └── 99-build-banner.md
```

The structure mirrors `docs/manual/`'s `NN-` prefix convention, the pandoc filter / template / preamble pattern, the figures cache pattern, and the `tests/lint.sh` gate pattern. Cloned (not symlinked) so the two manuals evolve independently.

### §4.2 Volume layout

#### §4.2.1 Part I — Foundations (`src/10-foundations/`)

- **§I.1 Introduction.** What this manual is, who should read which Part, how to navigate, the relationship between this manual and per-format BIPs / SPECs.
- **§I.2 The m-format star.** The four-format taxonomy (md1 / mk1 / ms1 / toolkit). What each format encodes. The forked-BCH boundary (md1↔mk1 share polynomials with HRP-mixing + per-format target residues) vs. the BIP-93-direct boundary (ms1 via `rust-codex32`). Why the boundary exists. Cross-card binding overview.
- **§I.3 codex32 and BCH.** The codex32 (BIP-93) primer at engineering depth. BCH code parameters. HRP-mixing math for md1↔mk1. Generator polynomials. Residue targets. Worked decode example (one card, by hand on paper).
- **§I.4 Conventions and notation.** Big-endian bit/byte ordering. The `bN:meaning` annotation form (bit-width + semantic label). Index-marker convention. Cross-reference convention. Reading the wire-format diagrams.

#### §4.2.2 Part II — Wire formats (`src/20-wire-formats/`)

Each format gets one chapter at bit-level depth. Structure mirrored across chapters so a reader who learned md1 can pattern-match mk1 and ms1.

- **§II.1 md1 wire format.** Header layout (4-bit version=4, in-band chunked-flag dispatch). Tag space (6-bit primary + 4-bit extension). Length envelope. Chunking. Body shapes (`MultiKeys`, `OriginPath`, etc.). NUMS flag (`is_nums`, `kiw = ⌈log₂(n)⌉`). Canonicality rules. Walker normalization (bare `Tag::PkK`/`PkH` at `c:`-sites). Worked encode of a 2-of-3 wsh-sortedmulti. Worked decode of the same. History note on retired dictionaries (path / use-site-path / shape — dropped in v0.11 for architectural cleanliness).
- **§II.2 mk1 wire format.** Forked-BCH plumbing. mk1-internal path dictionary (no longer mirrored with md1 since the mirror retired post md-codec v0.11). Single-string vs. K-of-N share futures (v0.2-shares migration locked in prefix byte + grouping + anti-collision invariants). Worked encode of a single xpub card.
- **§II.3 ms1 wire format.** BIP-93 codex32 directly via `rust-codex32`. BIP-39 entropy → master-seed → ms1 binding. Watch-only short-circuit semantics (the four-case table from toolkit SPEC §5.7). Worked encode of one secret card.

#### §4.2.3 Part III — Address derivation (`src/30-address-derivation/`)

- **§III.1 Descriptor → miniscript → address.** The three-tier model (template → derivation → script → address). What each tier knows and what it cannot know. BIP-388 wallet-policy framing. Origin-path semantics (Shared vs. Divergent under `Tag::OriginPaths = 0x36`).
- **§III.2 Shape coverage.** The v0.32 AST → `miniscript::Descriptor` converter. Every BIP-388-parseable shape: `tr(K)`, `tr(K, {script_tree})` with single-leaf and multi-leaf taptrees, `tr(NUMS, {...})`, `sh(multi)`, arbitrary `wsh(<miniscript>)`, tap-leaf miniscript. NUMS H-point handling. Removed `Error::UnsupportedDerivationShape`. Feature-gating (`[features] default=["derive"]`).
- **§III.3 Network and addressing.** Mainnet / testnet / regtest / signet. SLIP-0132 prefix interactions (mentioned for cross-reference; full coverage is in the end-user manual and the toolkit `convert` chapter).

#### §4.2.4 Part IV — Bundle formation (`src/40-bundle-formation/`)

- **§IV.1 Bundle anatomy.** The unified bundle envelope from toolkit v0.5+. Three-card layout (md1 template/policy + mk1 xpub + ms1 secret), where each lives, what the envelope JSON contains, what the engraving card looks like.
- **§IV.2 Anti-collision invariants.** `chunk_set_id` derivation and what it binds. The multiset `md1_xpub_match` rule (set-equality with multiplicity, sort-then-compare). The four-case ms1 short-circuit table. The mk1 cosigner-mapping diagnostic (`NotSupplied` / `DecodeFailed` / `XpubNotInPolicy`). BIP-388 distinct-key enforcement (typed `DerivationPath` equality; `h` ↔ `'` folding).
- **§IV.3 Future shares.** K-of-N share encoding. The v0.1→v0.2-shares migration invariants locked across all three formats. Why ms1 ships first (BIP-93 already specifies the math).

#### §4.2.5 Part V — Rust API reference (`src/50-rust-api/`)

One chapter per crate. Each chapter follows a fixed sub-structure: (a) crate purpose; (b) feature flags; (c) public API by module, with signature + 2–3 line behavior summary + worked invocation; (d) error taxonomy with one row per `Error::Variant`; (e) integration patterns; (f) versioning and MSRV.

- **§V.1 `md-codec`** — encode/decode public API, AST surface, walker, address derivation (feature-gated), error taxonomy (v0.30+v0.31 baseline; `OperatorContextViolation` at TopLevel).
- **§V.2 `mk-codec`** — xpub encode/decode, library-only (mk-cli surface lives in the end-user manual).
- **§V.3 `ms-codec`** — secret encode/decode, BIP-93 codex32 surface delegated to `rust-codex32`.
- **§V.4 `mnemonic-toolkit`** — bundle / verify-bundle / convert / export-wallet / derive-child orchestration. JSON envelope schema. Engraving-card layout.

#### §4.2.6 Back matter (`src/60-back-matter/`)

- **§61 Glossary.** Alphabetical; one row per term; cross-references to the section of first definitional use. Power-user terms only (BIP-39 / BIP-32 / BIP-388 / BIP-93 / miniscript primers stay in the end-user manual's appendices).
- **§62 Index table.** Bidirectional with `\index{}` markers in source. Same lint pattern as the end-user manual.
- **§63 Release history.** One row per tag across all four repos relevant to the manual's coverage. Date, version, one-line summary, pointer to the per-repo CHANGELOG entry.
- **§64 BIP cross-reference.** Table mapping each BIP cited (BIP-32 / BIP-39 / BIP-44 / BIP-48 / BIP-49 / BIP-84 / BIP-86 / BIP-93 / BIP-340 / BIP-341 / BIP-379 / BIP-388 / BIP-DKG-WIP, etc.) to the section(s) that reference it.
- **§65 Troubleshooting.** Decoder-error → likely-cause table for each format. Failure-mode signatures.
- **§66 Bibliography.** Academic and protocol references (Schnorr / FROST RFC 9591 / codex32 paper / miniscript paper / BIP texts).

### §4.3 Build pipeline

Cloned-and-adapted from `docs/manual/`:

- **Makefile targets:** `md`, `pdf`, `pdf-docker`, `figures-cache`, `verify-examples`, `lint`, `clean`, `help`. Same target shape as the end-user manual.
- **Dockerfile.build:** pinned pandoc + xelatex + mermaid-filter + makeindex versions. Reuses the end-user manual's image lineage; may be the same image (renamed `mnemonic-tech-manual-build:latest`) or a divergent image if dependencies need to diverge.
- **Pandoc filters:** primer-box, danger-box, mermaid-filter, strip-latex-from-md, index-marker. Inherited.
- **Mermaid figures:** pre-rendered SVG cache committed to `figures/cache/`; mermaid-filter optional at build time.
- **Tests:** `tests/lint.sh` runs markdownlint + cspell + lychee + flag-coverage (gated against `md`/`mk`/`ms`/`mnemonic` `--help`) + glossary-coverage + index bidirectional. `tests/verify-examples.sh` re-runs worked-example transcripts against locally-built CLIs.

### §4.4 Mirror invariants

The end-user manual's `manual-cli-surface-mirror` invariant (any CLI flag/API change updates `40-cli-reference/`) is unchanged. **The technical manual does not duplicate that invariant** — Part V is a *library API* surface, not a CLI surface; CLI-flag mirroring stays in the end-user manual. The technical manual adds two new invariants:

- **`tech-manual-api-surface-mirror`:** any public Rust API change (function signature, type signature, error variant added/removed, feature flag added/removed) in `md-codec` / `mk-codec` / `ms-codec` / `mnemonic-toolkit` updates the relevant Part V chapter section. **Enforcement mode:** *process*, not lint. Tracked via (a) the per-phase code-reviewer rounds that gate each cut, and (b) the v1.0 architect sign-off at Phase 5.6.2 that pattern-matches `cargo doc --no-deps` against the Part V chapters. A lightweight lint helper at `tests/api-surface-coverage.sh` (added at Phase 4.5) MAY be implemented to grep public symbol names against chapter content for missing rows; it is a hint, not a gate.
- **`tech-manual-wire-format-mirror`:** any wire-format-affecting change in a sibling repo (md-codec, mk-codec, ms-codec) updates the corresponding Part II chapter in lockstep with the implementing PR. **Enforcement mode:** *process*, not lint. Tracked in each sibling repo's `design/FOLLOWUPS.md` with `Companion:` cross-citations; the per-phase code-reviewer round on the next technical-manual cut verifies that cited SPEC versions match HEAD across all four repos.

`tests/lint.sh` continues to enforce only the things it CAN enforce: chapter-existence, `\index{}` bidirectional consistency, markdownlint, cspell, lychee link health, and existence-only checks on cross-referenced sibling files. Process-enforced invariants are explicit about being process-enforced.

## §5 Cross-repo coordination

This work touches four repos:

- **`mnemonic-toolkit`** — primary; hosts the SPEC, the PLAN, the technical manual itself, and the build pipeline. Track A also edits `docs/manual/src/` here.
- **`descriptor-mnemonic` (md1)** — Track A audits + edits documentation files. Track B's Part II §II.1 and Part V §V.1 cite this repo's BIP draft and `md-codec` public API.
- **`mnemonic-key` (mk1)** — Track B's Part II §II.2 and Part V §V.2 cite this repo's SPEC and `mk-codec` public API. No edits in Track A.
- **`mnemonic-secret` (ms1)** — Track B's Part II §II.3 and Part V §V.3 cite this repo's SPEC and `ms-codec` public API. No edits in Track A.

### §5.1 FOLLOWUPS mirroring

Standard cross-repo convention applies: any cross-cutting item discovered while writing the technical manual files entries in both repos' `design/FOLLOWUPS.md` with `Companion:` cross-citation. Examples likely to surface:

- A Part II chapter discovers a wire-format ambiguity in a sibling repo's SPEC → mirror entry to that repo with tier `wire-format-clarification`.
- A Part V chapter wants public API doc-comment improvements in a sibling crate → mirror entry to that repo with tier `api-doc-improvement`.

## §6 Release cuts (decomposition)

PLAN expands these. SPEC describes the contract.

| Cut | Tag | Scope | Acceptance |
|---|---|---|---|
| **A** | (no tag — Track A audit) | All files in §3.1 audited and corrected. | Per-file diff <50 lines net (with scope-review escape hatch). `flag-coverage` lint green for `42-md.md`. Workspace builds green in both repos. |
| **v0.1** | `tech-manual-v0.1.0` | Parts I + II (foundations + all three wire formats) + back-matter skeleton (TOC, glossary stubs, index scaffold, release-history seed). | All chapters lint-green. PDF builds via `pdf-docker`. PDF length 40–110pp (soft floor; flag for scope review if <40pp). All wire-format claims traceable to HEAD SPECs in the cited repos. |
| **v0.2** | `tech-manual-v0.2.0` | Part III (address derivation) added. Glossary + index populated for Part III terms. | v0.32 derive_address shape coverage exhaustively documented. Worked address-derivation examples verified by `verify-examples.sh`. |
| **v0.3** | `tech-manual-v0.3.0` | Part IV (bundle formation) added. Anti-collision invariants documented. | All invariants traceable to toolkit SPECs (v0.4 / v0.5 / v0.6 / v0.7 / v0.8). Worked bundle/verify-bundle examples verified. |
| **v0.4** | `tech-manual-v0.4.0` | Part V (Rust API reference) added — all four crates. | Public API surface matches HEAD `cargo doc` output. Feature-flag prose matches `Cargo.toml`. |
| **v1.0** | `tech-manual-v1.0.0` | Back-matter polish: full index population (every `\index{}` has a row in `62-index-table.md`), glossary completion, BIP cross-reference table complete, release-history table populated through current toolkit tag, bibliography complete. | All lint passes green. PDF length ≥200pp. Architect signs off on "every aspect of the software" coverage claim against the end-user manual's index + each repo's public API surface. |

Each tag is an independent toolkit-repo release with a PDF asset attached, following the `manual-v0.1.0` release pattern.

## §7 Acceptance criteria (v1.0 cumulative)

- **A1.** Every BIP-388-parseable descriptor shape supported by the toolkit has a bit-level encode walk-through in Part II §II.1 and an address-derivation walk-through in Part III.
- **A2.** Every public function in each of the four crates' library APIs is referenced by name + brief description in the relevant Part V chapter. (Doesn't need a full code sample; needs at minimum: signature, one-line behavior summary, error variants returned, feature flag if any.)
- **A3.** Every error variant in each crate's `Error` enum has a row in the relevant Part V chapter's error-taxonomy table.
- **A4.** Glossary has ≥80 entries, each with a cross-reference to the section of first definitional use.
- **A5.** Index has ≥250 entries (alphabetical, deduplicated). Each `\index{}` marker in source has a matching row in `62-index-table.md` and vice versa.
- **A6.** Table of contents auto-generated by pandoc covers every Part / chapter / section. PDF and markdown both render TOC correctly.
- **A7.** BIP cross-reference table covers ≥12 BIPs and lists every section that references each.
- **A8.** Worked examples in every chapter verified by `tests/verify-examples.sh` against locally-built CLIs and library calls.
- **A9.** Both mirror invariants (§4.4) green at v1.0 close.
- **A10.** PDF length ≥200pp (single-volume sizing target).
- **A11.** Build pipeline reproducible: `make pdf-docker` run twice consecutively on the same host produces a byte-identical PDF. Cross-host byte-identity is desired but not a blocking gate (xelatex / PDF metadata reproducibility is best-effort; injection of `SOURCE_DATE_EPOCH` etc. is in scope only if the existing user-manual pipeline already does it).

## §8 Risks and mitigations

| # | Risk | Mitigation |
|---|---|---|
| R1 | Doc drift across four repos as the technical manual is being written | The two new mirror invariants (§4.4) prevent CLI/wire-format drift. Per-phase code-reviewer pass catches semantic drift before each cut tags. |
| R2 | Scope creep — manual becomes its own multi-volume project | Hard sizing target per cut. Architect review gates each cut against scope. Lows/Nits fold inline at tag time per the `zero_followups_from_release_cycles` rule. |
| R3 | Pandoc / xelatex / mermaid-filter brittleness | Reuses the existing `docs/manual/` pipeline, which is already battle-tested through user-manual v0.1.0 / v0.1.1. Dockerfile.build pinning makes builds reproducible. |
| R4 | Worked examples drift as CLIs evolve | `tests/verify-examples.sh` re-runs every example transcript on every PR. Pattern proven in the end-user manual. |
| R5 | Index / glossary maintenance burden | Bidirectional lint gate forces every `\index{}` to have a row in `62-index-table.md`. Glossary stub created at v0.1; populated incrementally per cut. |
| R6 | "Comprehensive" claim drift — readers expect this to cover *everything*, including future features | The disclaimer chapter (cloned from end-user manual) is explicit: tracks toolkit `main`; future features (BIP-85 RSA, K-of-N shares, BIP-DKG / FROST) deferred to subsequent cuts and called out in the release-history appendix. |
| R7 | Cross-repo PR coordination overhead | Track A's per-repo diffs go in independently as `docs(audit-v0.32):` commits. Track B's mirror touches at PR time only affect siblings when a wire-format change actually crosses the boundary. |
| R8 | Bus factor on the pandoc pipeline | `AUTHORING.md` cloned and adapted. Build is `make pdf-docker` from a single command. No specialized knowledge needed to render. |

## §9 What this SPEC does not decide

- **D1.** Exact paragraph-by-paragraph chapter outlines. The chapter outlines emerge from the per-cut plan phases; the SPEC commits to chapter titles and ~paragraph-count budgets, not prose.
- **D2.** Exact figure list. Mermaid diagrams added as chapters are drafted; cumulative figure count budget appears in the per-cut plan phase.
- **D3.** Whether the technical manual eventually adopts mdBook for an HTML render path. Deferred to a post-v1.0 decision.

**Locked decisions** (previously in §9; promoted out for clarity):

- **Release pattern (was D4):** per-cut PDFs get their own `tech-manual-vX.Y.Z` tag + GitHub release, following `manual-v0.1.0`'s pattern.
