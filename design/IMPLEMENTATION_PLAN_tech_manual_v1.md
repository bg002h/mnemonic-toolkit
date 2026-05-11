# IMPLEMENTATION PLAN — m-format Constellation Technical Manual v1.0

**Spec:** [`SPEC_tech_manual_v1.md`](./SPEC_tech_manual_v1.md).

| Field | Value |
|---|---|
| Plan version | 1.0 (covers Track A + Track B cuts v0.1 → v1.0) |
| Date drafted | 2026-05-11 |
| Status | Drafting → architect review → user review → autonomous execution |
| Estimated cycles | 6 (Phase 0 + 5 cuts), each independently tagged & released |
| Per-phase review discipline | feature-dev:code-reviewer to 0C/0I before tag; reports persist to `design/agent-reports/` |
| Tag-time discipline | Zero new FOLLOWUPs at release-cycle tags; Lows/Nits fold inline (`zero_followups_from_release_cycles` rule). Mid-cycle commits may file FOLLOWUPs. |

---

## Cross-cutting conventions

### Commit discipline

- Per phase or per logical sub-phase, one commit.
- Subject prefix per phase block: `docs(audit-v0.32 phase N):`, `docs(tech-manual phase X.Y):`, `release(tech-manual-vX.Y.Z):`.
- Stage paths explicitly. No `git add -A` per the repo memory.
- Pre-commit checks: `cargo build --workspace --all-features` in any repo whose code-paths a diff touched; `make lint` for any manual-source diff in `docs/technical-manual/`.

### Reviewer-loop discipline

- Each implementation phase ends with one or more `feature-dev:code-reviewer` rounds.
- Iterate until Critical = 0 and Important = 0.
- Low / Nit findings: fold inline if the phase is a release-tag phase; defer to `design/FOLLOWUPS.md` only if the phase is operational (mid-cycle).
- Reports persist to `design/agent-reports/tech_manual_vX_Y_phase_Z_review_rN.md`.
- Brainstorm / spec / plan / final-cut reviews stay in transcript (don't persist).

### Cycle-exit verification (before each tag)

- `cargo test --workspace --all-features` in every repo touched.
- `make lint` and `make pdf-docker` in `docs/technical-manual/` for any Track B cut.
- `make lint` in `docs/manual/` if Track A touched user-manual chapters.
- Verify HEAD content (`git show HEAD:path`) matches working-tree expectations for spot-checked files; cargo reads the working tree, so don't trust build-success-implies-committed-content.

### Sibling-repo edits

- Track A's md1-repo edits happen in `/scratch/code/shibboleth/descriptor-mnemonic/`.
- All other work happens in `/scratch/code/shibboleth/mnemonic-toolkit/`.
- Cross-repo FOLLOWUPS get `Companion:` cross-citations in lockstep.

---

## Phase 0 — Track A audit & update

**Goal:** every claim about v0.30 wire format + v0.31 error-taxonomy + v0.32 derive_address shape coverage in scope-files matches HEAD. Per-file diffs small and surgical.

**No tag.** Phase 0 closes with two commits — one per repo — using `docs(audit-v0.32):` prefix.

### Phase 0.A — md1-repo audit pass

Working directory: `/scratch/code/shibboleth/descriptor-mnemonic/`.

**0.A.1 — Build the audit checklist.** Walk SPEC §3.1 md1-repo list. For each file produce a short note: "what HEAD says" vs. "what doc says" vs. "delta to apply". Persist as a working note (not committed) at `design/agent-reports/track_a_audit_md1_checklist.md` for the reviewer's reference.

**0.A.2 — Apply file-by-file edits.**

- `README.md` — verify "What MD is for" + "What's covered in v0" + "Status" sections; refresh example invocations against current `md` binary output.
- `crates/md-codec/README.md` — refresh error-taxonomy references (drop `Error::UnsupportedDerivationShape`; verify `OperatorContextViolation`); refresh feature-flag prose against `Cargo.toml` (`[features] default=["derive"]`).
- `crates/md-cli/README.md` — diff against `md --help` output; refresh subcommand surface and any wire-format prose.
- `MIGRATION.md` — verify v0.30 / v0.31 / v0.32 entries; add any missing.
- `CHANGELOG.md` — spot-check v0.30 / v0.31 / v0.32 entries against release commits; correct drift.
- `docs/json-schema-v1.md` — refresh against v0.31 `JsonTag` mirror; correct drift.
- `bip/bip-mnemonic-descriptor.mediawiki` — audit-only sweep for v0.31 (`OperatorContextViolation TopLevel`) and v0.32 (rust-miniscript-based address derivation). No structural changes; only update prose that references removed variants or pre-v0.30 wire layout.
- Inline `//!` and `///` doc-comments in `md-codec` public modules — surgical edits where they reference removed/renamed Error variants or pre-v0.30 wire layout. Use grep to find candidates: `rg -n "UnsupportedDerivationShape|pre-v0.30|v0.11 wire" crates/md-codec/src/`.

**0.A.3 — Verify.** `cargo build --workspace --all-features && cargo build --workspace --no-default-features` green. Re-run any example invocations the doc files quote; confirm output matches.

**0.A.4 — Commit.** One commit, prefix `docs(audit-v0.32 phase 0.A):`. Stage paths explicitly.

**0.A.5 — Code-reviewer round.** Dispatch `feature-dev:code-reviewer`; iterate to 0C/0I. Report persists. **Lows / Nits:** fold inline if local to the file being audited. If a Low requires sibling-repo action (e.g., a stale claim in a sibling crate's README that the md1 doc references), file a cross-repo `FOLLOWUPS.md` entry per SPEC §5.1 with `Companion:` cross-citation. No tag fires; no Zero-FOLLOWUPs constraint applies (the `zero_followups_from_release_cycles` rule activates at tags only).

### Phase 0.B — toolkit-repo audit pass

Working directory: `/scratch/code/shibboleth/mnemonic-toolkit/`.

**0.B.1 — Audit checklist** at `design/agent-reports/track_a_audit_toolkit_checklist.md`.

**0.B.2 — Apply file-by-file edits.**

- `docs/manual/src/40-cli-reference/42-md.md` — diff against `md --help`; refresh flags / synopses / worked examples; preserve voice. Run `tests/lint.sh flag-coverage` locally afterwards — must pass.
- `docs/manual/src/60-appendices/61-glossary.md` — add/correct entries for tag-space rework, NUMS flag, walker normalization, OperatorContextViolation, derive_address shape coverage. Do not add unrelated terms.
- `docs/manual/src/60-appendices/64-descriptors-primer.md` — only replace "not currently supported" framing with v0.32 coverage where applicable; do not expand the chapter.

**0.B.3 — Verify.** `tests/lint.sh flag-coverage` green for `42-md.md`. `make lint` in `docs/manual/` overall green. `cargo build --workspace` green.

**0.B.4 — Commit.** One commit, prefix `docs(audit-v0.32 phase 0.B):`. Stage paths explicitly.

**0.B.5 — Code-reviewer round** to 0C/0I. Same Lows/Nits policy as 0.A.5: fold inline if local; cross-repo items go to `FOLLOWUPS.md` with `Companion:` citations; no Zero-FOLLOWUPs constraint (no tag fires).

### Phase 0 close — handshake

Both commits landed. md1-repo and toolkit-repo each at green. No tag fires; the audit is preparatory for Track B.

---

## Phase 1 — Cut tech-manual-v0.1 (Foundations + Wire formats + skeleton)

**Goal:** First releasable cut of the technical manual. Parts I + II + back-matter skeleton.

Tag at close: `tech-manual-v0.1.0`. GitHub release with PDF asset attached.

### Phase 1.0 — Pipeline scaffold

**Subgoal:** the build pipeline works before any chapter content is written.

**1.0.1 — Skeleton directory.** Create `docs/technical-manual/` with the layout from SPEC §4.1. All directories present. Create stub files at root: `README.md` (one-line pointer to SPEC), `FOLLOWUPS.md` (clone format header from `docs/manual/FOLLOWUPS.md`; empty body), `AUTHORING.md` (placeholder; adapted in 1.0.2). Create stub `src/` chapter files (one per file in SPEC §4.1 tree) containing only a single H1 line matching the planned chapter title — required so pandoc concatenation produces a valid PDF in 1.0.4.

**1.0.2 — Clone-and-adapt build pipeline.** Copy `docs/manual/{Makefile,Dockerfile.build,AUTHORING.md,pandoc/,figures/}` into `docs/technical-manual/`. Adapt:
- Rename `mnemonic-manual-build` → `mnemonic-tech-manual-build` in `DOCKER_IMAGE` and any image-tag references.
- In `Makefile`: change output filename from `m-format-manual.pdf` to `m-format-technical-manual.pdf` in the `pdf` target, any `release-attach` target, and the `help` text.
- In `Makefile`: change the `pdf-docker` recipe's docker `-w` flag from `/work/docs/manual` to `/work/docs/technical-manual`. Confirm any other recipe that references `docs/manual` is rewritten to `docs/technical-manual`.
- Update `pandoc/metadata.yaml` title / subtitle / authors / dedication / output PDF metadata fields.
- Update `AUTHORING.md` to reflect tech-manual scope (audience, conventions).
- Verify all path traversals in the cloned `Makefile` resolve correctly from `docs/technical-manual/` (`TOOLKIT_ROOT`, `WORKSPACE_ROOT`, and the four `*_BIN` variables are self-relative-via-MAKEFILE_LIST and should still work).

**1.0.3 — `tests/lint.sh` adaptation.** Clone from `docs/manual/tests/lint.sh`. Adjust path roots. Adapt the existing checks: markdownlint, cspell, lychee, `\index{}` bidirectional, glossary-coverage, chapter-existence, build-banner enforcement. **Do NOT clone the `flag-coverage` check** — the technical manual does not have a `40-cli-reference/` mirror surface; CLI-flag mirroring stays in the end-user manual exclusively (SPEC §4.4). Create a `tests/api-surface-coverage.sh` *stub* (header + `exit 0`) at Phase 1 — populated at Phase 4.5 with the `tech-manual-api-surface-mirror` hint-grep logic (SPEC §4.4: hint, not gate). The `tech-manual-wire-format-mirror` is process-only and has no lint analog.

**1.0.4 — Smoke build.** `make pdf-docker` with stub chapters present must produce a PDF with TOC + frontmatter + disclaimer + empty Parts. Verify locally. Then run a second `make pdf-docker` and `diff build/m-format-technical-manual.pdf` between the two runs — must be byte-identical (SPEC §7 A11 same-host clause).

**1.0.5 — Phase commit + code-reviewer round.** Commit prefix `docs(tech-manual phase 1.0):`. Reviewer to 0C/0I.

### Phase 1.1 — Frontmatter + disclaimer + Part I (Foundations)

**Subgoal:** write Part I chapters and supporting frontmatter.

**1.1.1 — `00-frontmatter.md` + `00-disclaimer.md`.** Pattern-match the end-user manual's frontmatter and disclaimer. Disclaimer adapted to the technical-manual voice: "reference for implementers and integrators; assume bugs until external review."

**1.1.2 — `10-foundations/11-introduction.md`.** SPEC §4.2.1 §I.1.

**1.1.3 — `10-foundations/12-the-m-format-star.md`.** SPEC §4.2.1 §I.2. Includes a mermaid diagram of the four-format taxonomy + the forked-BCH vs. BIP-93-direct boundary.

**1.1.4 — `10-foundations/13-codex32-and-bch.md`.** SPEC §4.2.1 §I.3. Includes the worked-by-hand decode example. Cite codex32 paper + BIP-93. Mermaid diagram of the HRP-mixing math.

**1.1.5 — `10-foundations/14-conventions-and-notation.md`.** SPEC §4.2.1 §I.4.

**1.1.6 — Phase commit + reviewer round.** Commit prefix `docs(tech-manual phase 1.1):`. Reviewer to 0C/0I.

### Phase 1.2 — Part II §II.1 (md1 wire format)

**Subgoal:** the heaviest chapter; full bit-level md1 spec at engineering depth.

**1.2.1 — Draft `20-wire-formats/21-md1-wire-format.md`** from SPEC §4.2.2 §II.1. **Source-of-truth files** (read before drafting):
- `/scratch/code/shibboleth/descriptor-mnemonic/design/SPEC_v0_30_wire_format.md` (primary wire-format authority).
- `/scratch/code/shibboleth/descriptor-mnemonic/design/SPEC_v0_11_wire_format.md §1.4` (retired-dictionary history note — cite verbatim for the closing history paragraph).
- `/scratch/code/shibboleth/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki` (post-Track-A audited).
- HEAD `crates/md-codec/src/` for any low-level disambiguations.

Include:

- Header layout (4-bit version=4, in-band chunked-flag dispatch).
- Tag space (6-bit primary + 4-bit extension), with the full tag table reproduced.
- Length envelope structure.
- Chunking algorithm.
- Body shapes (`MultiKeys`, `OriginPath`, etc.).
- NUMS flag semantics (`is_nums`, `kiw = ⌈log₂(n)⌉`).
- Canonicality rules.
- Walker normalization (bare `Tag::PkK`/`PkH` at `c:`-sites).
- One full worked encode of a 2-of-3 wsh-sortedmulti.
- One full worked decode of the same.
- History note on retired dictionaries.

**1.2.2 — Mermaid figures.** Header-byte breakdown; tag-space tree; encode-pipeline flow.

**1.2.3 — Index markers.** Inline `\index{}` for: `md1`, `BCH code`, `codex32`, `chunk_set_id`, `OriginPath`, `MultiKeys`, `NUMS`, each Tag-code value, etc. Add matching rows to `60-back-matter/62-index-table.md`.

**1.2.4 — Worked-example transcript.** Capture in `transcripts/md1-encode-decode-2of3-wsh-sortedmulti.txt`; reference from `tests/verify-examples.sh`.

**1.2.5 — Phase commit + reviewer round** to 0C/0I. Commit prefix `docs(tech-manual phase 1.2):`.

### Phase 1.3 — Part II §II.2 (mk1 wire format)

**Subgoal:** mirror Phase 1.2 structure for mk1.

**1.3.1 — Draft `20-wire-formats/22-mk1-wire-format.md`** from SPEC §4.2.2 §II.2. Cite mk1's SPEC + BIP draft. Include forked-BCH discussion, mk1-internal path dictionary (note that the md1↔mk1 mirror invariant retired post md-codec v0.11), share-encoding futures.

**1.3.2 — Mermaid figures** + index markers + worked encode of a single xpub card + transcript capture.

**1.3.3 — Phase commit + reviewer round** to 0C/0I.

### Phase 1.4 — Part II §II.3 (ms1 wire format)

**Subgoal:** mirror Phase 1.2 structure for ms1.

**1.4.1 — Draft `20-wire-formats/23-ms1-wire-format.md`** from SPEC §4.2.2 §II.3. Cite ms1's SPEC + `rust-codex32`. Include BIP-93-direct framing, the watch-only short-circuit four-case table, BIP-39-entropy → master-seed → ms1 binding.

**1.4.2 — Mermaid figures** + index markers + worked encode of a single secret card + transcript capture.

**1.4.3 — Phase commit + reviewer round** to 0C/0I.

### Phase 1.5 — Back-matter skeleton

**Subgoal:** stubs for glossary, index-table, release-history, BIP cross-reference, troubleshooting, bibliography. v0.1 populates only the entries earned by Parts I + II content; subsequent cuts populate the rest.

**1.5.1 — `60-back-matter/61-glossary.md`.** Seed with terms introduced in Parts I + II. Target ~30 entries at v0.1.

**1.5.2 — `60-back-matter/62-index-table.md`.** Populated from all `\index{}` markers in Parts I + II. Target ~100 entries at v0.1.

**1.5.3 — `60-back-matter/63-release-history.md`.** Stub table; seed with the four repos' currently-tagged versions. Acceptance: rows for at least md-codec-v0.30 / v0.31 / v0.32, mk-codec-v0.2.2, ms-codec-v0.1.1, toolkit-v0.8.0.

**1.5.4 — `60-back-matter/64-bip-cross-reference.md`.** Stub table; seed with BIPs cited in Parts I + II.

**1.5.5 — `60-back-matter/65-troubleshooting.md`.** Stub with one section per format; populate decoder-error rows earned by Parts I + II.

**1.5.6 — `60-back-matter/66-bibliography.md`.** Initial bibliography: BIP-32, BIP-39, BIP-93, BIP-340, BIP-341, BIP-379, BIP-388, codex32 paper, miniscript paper, FROST RFC 9591.

**1.5.7 — `99-build-banner.md`.** Cloned-and-adapted from end-user manual.

**1.5.8 — Phase commit + reviewer round** to 0C/0I.

### Phase 1.6 — Cycle exit & tag

**1.6.1 — Cycle-exit verification.** `cargo test --workspace --all-features` (toolkit) green. `make lint && make pdf-docker` in `docs/technical-manual/` green. PDF length 60–110pp per SPEC §6. All worked-example transcripts re-run green via `tests/verify-examples.sh`.

**1.6.2 — Final code-reviewer round.** Whole-cut review against SPEC §7 acceptance criteria scoped to v0.1: A4 (glossary ≥30), A5 (index ≥100), A6 (TOC), A8 (transcripts), A10 (PDF ≥40pp at v0.1 per the revised soft floor; final ≥200pp gate is v1.0). A1 (every BIP-388 shape walk-through) is NOT in scope at v0.1 — it gates on Part III, which ships at v0.2. Phase 1.2 covers one representative policy at bit-level depth for the wire-format chapter; that satisfies Part II §II.1's chapter-internal goal without making any A1-coverage claim. Iterate to 0C/0I.

**1.6.3 — Lows/Nits fold inline.** Per `zero_followups_from_release_cycles`.

**1.6.4 — CHANGELOG entry.** Append a `tech-manual-v0.1.0` row.

**1.6.5 — Tag.** `git tag -a tech-manual-v0.1.0 -m "..."` and `git push --tags`.

**1.6.6 — GitHub release.** `gh release create tech-manual-v0.1.0 docs/technical-manual/build/m-format-technical-manual.pdf` with release notes pointing to the SPEC.

**1.6.7 — User check-in.** Surface release URL + PDF asset to user. v0.2 plan presented before v0.2 starts.

---

## Phase 2 — Cut tech-manual-v0.2 (Address derivation)

**Goal:** Part III added. Address derivation documented end-to-end at engineering depth.

Tag at close: `tech-manual-v0.2.0`.

### Phase 2.1 — Part III §III.1 (Descriptor → miniscript → address)

**2.1.1 — Draft `30-address-derivation/31-descriptor-to-miniscript.md`** from SPEC §4.2.3 §III.1.

**2.1.2 — Mermaid figures** for the three-tier model + origin-path semantics under `Tag::OriginPaths = 0x36`.

**2.1.3 — Worked example:** one descriptor through all three tiers, with the resulting address. Transcript captured.

**2.1.4 — Phase commit + reviewer round** to 0C/0I.

### Phase 2.2 — Part III §III.2 (Shape coverage)

**2.2.1 — Draft `30-address-derivation/32-shape-coverage.md`** from SPEC §4.2.3 §III.2. Exhaustive enumeration of the v0.32 shapes: `tr(K)`, `tr(K, {script_tree})` single-leaf + multi-leaf, `tr(NUMS, {...})`, `sh(multi)`, arbitrary `wsh(<miniscript>)`, tap-leaf miniscript. One worked address-derivation per shape.

**2.2.2 — Mermaid figure:** AST → `miniscript::Descriptor` converter pipeline.

**2.2.3 — Index markers + transcripts.**

**2.2.4 — Phase commit + reviewer round** to 0C/0I.

### Phase 2.3 — Part III §III.3 (Network and addressing)

**2.3.1 — Draft `30-address-derivation/33-network-and-addressing.md`** from SPEC §4.2.3 §III.3. Mainnet / testnet / regtest / signet. SLIP-0132 prefix cross-reference (full coverage stays in the end-user manual). For the deeper interaction surface, cite the end-user manual's `convert` workflow chapter (`mnemonic-toolkit/docs/manual/src/`) by URL / section name only — **do not create a new chapter** in the technical manual for `convert`; that surface is workflow-shaped and belongs in the end-user manual.

**2.3.2 — Phase commit + reviewer round** to 0C/0I.

### Phase 2.4 — Back-matter accretion

**2.4.1 — Glossary additions** for Part III terms (miniscript fragments, tap-leaf, NUMS H-point, etc.). Target +20 entries (running total ≥50).

**2.4.2 — Index additions.** Target +50 entries (running total ≥150).

**2.4.3 — BIP cross-reference rows** for BIPs referenced in Part III.

**2.4.4 — Release-history row** for `tech-manual-v0.1.0`.

### Phase 2.5 — Cycle exit & tag

Same pattern as Phase 1.6: cycle-exit verification, final reviewer round, Lows/Nits inline, CHANGELOG entry, tag `tech-manual-v0.2.0`, GitHub release with PDF asset, user check-in.

---

## Phase 3 — Cut tech-manual-v0.3 (Bundle formation)

**Goal:** Part IV added. Bundle formation + anti-collision invariants + share futures.

Tag at close: `tech-manual-v0.3.0`.

### Phase 3.1 — Part IV §IV.1 (Bundle anatomy)

**3.1.1 — Draft `40-bundle-formation/41-bundle-anatomy.md`** from SPEC §4.2.4 §IV.1. Three-card layout, envelope JSON, engraving-card layout.

**3.1.2 — Mermaid figure:** bundle creation pipeline; bundle verification pipeline.

**3.1.3 — Worked example:** one bundle creation + one verification. Transcripts captured.

**3.1.4 — Phase commit + reviewer round** to 0C/0I.

### Phase 3.2 — Part IV §IV.2 (Anti-collision invariants)

**3.2.1 — Draft `40-bundle-formation/42-anti-collision-invariants.md`** from SPEC §4.2.4 §IV.2. `chunk_set_id` derivation, multiset `md1_xpub_match`, four-case ms1 short-circuit, mk1 cosigner-mapping diagnostic, BIP-388 distinct-key (typed equality with `h` ↔ `'` folding).

**3.2.2 — Worked example:** a colliding bundle and the diagnostic output. Transcript captured.

**3.2.3 — Phase commit + reviewer round** to 0C/0I.

### Phase 3.3 — Part IV §IV.3 (Future shares)

**3.3.1 — Draft `40-bundle-formation/43-future-shares.md`** from SPEC §4.2.4 §IV.3. v0.1 → v0.2-shares migration invariants locked across all three formats. Why ms1 ships first.

**3.3.2 — Phase commit + reviewer round** to 0C/0I.

### Phase 3.4 — Back-matter accretion

Same pattern as Phase 2.4. Glossary +15, index +40, BIP cross-ref rows, release-history row for `tech-manual-v0.2.0`.

### Phase 3.5 — Cycle exit & tag

Tag `tech-manual-v0.3.0`. GitHub release. User check-in.

---

## Phase 4 — Cut tech-manual-v0.4 (Rust API reference)

**Goal:** Part V added — the Rust API reference across all four crates.

Tag at close: `tech-manual-v0.4.0`.

### Phase 4.0 — API surface harvest

**Subgoal:** systematic capture of public API surface across all four crates before any chapter prose is written. Eliminates the "did we miss a function" risk.

**4.0.1 — Per crate, run `cargo doc --no-deps` and walk the public surface.** Generate a working note per crate at `docs/technical-manual/transcripts/api-harvest-<crate>.md` listing every public function, type, trait, module, feature flag, and error variant. (Working notes — not code-reviewer reports — so they live with the manual transcripts, not in `design/agent-reports/`.)

**4.0.2 — Reviewer round** on the harvest notes — has anything been missed? Are doc-comment claims accurate against HEAD?

### Phase 4.1 — Part V §V.1 (`md-codec`)

**4.1.1 — Draft `50-rust-api/51-md-codec-api.md`** from SPEC §4.2.5 §V.1 + Phase 4.0.1 harvest. One row per public symbol; one row per `Error::Variant`. Feature flags. Integration patterns (one section: typical encoder pipeline; one section: typical decoder pipeline).

**4.1.2 — Worked Rust example.** Two artifacts:
1. Example source at `docs/technical-manual/examples/md-codec-api-roundtrip.rs` — ~30-line program that encodes + decodes a policy through the public API. Add `[[example]]` entry to the appropriate `Cargo.toml` (or use `cargo --example` from a small sub-crate inside `docs/technical-manual/examples/`; pick whichever fits the toolkit's existing pattern for runnable docs examples).
2. Transcript invocation at `docs/technical-manual/transcripts/md-codec-api-roundtrip.cmd` containing the shell command (e.g., `cargo run --quiet --example md-codec-api-roundtrip`), and `md-codec-api-roundtrip.out` containing the expected stdout. The existing `tests/verify-examples.sh` `.cmd`/`.out` model handles this without modification.

**4.1.3 — Phase commit + reviewer round** to 0C/0I.

### Phase 4.2 — Part V §V.2 (`mk-codec`)

Pattern as Phase 4.1; library-only.

### Phase 4.3 — Part V §V.3 (`ms-codec`)

Pattern as Phase 4.1.

### Phase 4.4 — Part V §V.4 (`mnemonic-toolkit`)

Pattern as Phase 4.1. Includes JSON envelope schema + engraving-card layout. The Toolkit's CLI surface is **not** Part V's concern (that's the end-user manual); Part V covers the library API and the JSON contracts.

### Phase 4.5 — API-surface coverage helper

**4.5.1 — Populate `tests/api-surface-coverage.sh`** (the stub created at 1.0.3). Implementation: for each of the four crates, run `cargo doc --no-deps --message-format=json` (or `cargo rustdoc -- --output-format json` if available); extract the list of public top-level symbol names; grep each name against the relevant Part V chapter; emit a warning row per symbol absent from the chapter. **Exit 0 on warnings — this is a hint, not a gate** (per SPEC §4.4). The v1.0 gate is the architect sign-off at Phase 5.6.2.

**4.5.2 — Run the helper** and ensure no symbols are missing. Resolve any gaps in Phase 4.1–4.4 chapter content before tagging.

### Phase 4.6 — Back-matter accretion

Glossary +20 (API terms), index +60, BIP cross-ref completion, release-history row for `tech-manual-v0.3.0`.

### Phase 4.7 — Cycle exit & tag

Tag `tech-manual-v0.4.0`. GitHub release. User check-in.

---

## Phase 5 — Cut tech-manual-v1.0 (Back-matter polish + v1.0 declaration)

**Goal:** v1.0 release. Back matter fully populated, all SPEC §7 cumulative acceptance criteria green.

Tag at close: `tech-manual-v1.0.0`.

### Phase 5.1 — Index population

**5.1.1 — Walk every chapter** for terms that warrant `\index{}` markers but lack them. Add markers; add matching `62-index-table.md` rows. Target ≥250 entries total.

**5.1.2 — Bidirectional lint pass.** Every `\index{}` has a row; every row has a marker.

**5.1.3 — Phase commit + reviewer round** to 0C/0I.

### Phase 5.2 — Glossary completion

**5.2.1 — Walk every chapter** for terms first introduced without a glossary entry. Add entries. Target ≥80 entries total.

**5.2.2 — Cross-reference pass.** Every glossary entry points to the section of first definitional use.

**5.2.3 — Phase commit + reviewer round** to 0C/0I.

### Phase 5.3 — BIP cross-reference table completion

**5.3.1 — Walk every chapter** for BIP citations. Aggregate into `64-bip-cross-reference.md`. Target ≥12 BIPs, each with every section citing it.

**5.3.2 — Phase commit + reviewer round** to 0C/0I.

### Phase 5.4 — Release-history table completion

**5.4.1 — Populate release history** through current HEAD across all four repos relevant to the manual.

**5.4.2 — Add rows** for `tech-manual-v0.4.0` and any in-flight sibling releases.

### Phase 5.5 — Bibliography completion + troubleshooting completion

**5.5.1 — Bibliography:** every external reference cited in any chapter has a bibliography row.

**5.5.2 — Troubleshooting:** every error variant from Part V has a row in `65-troubleshooting.md` with cause + remediation pointer.

### Phase 5.6 — Cycle exit & tag

**5.6.1 — Cycle-exit verification** against SPEC §7 acceptance criteria A1–A11 cumulative.

**5.6.2 — Final whole-volume architect review.** Dispatch `feature-dev:code-architect` against the full manual. Architect pattern-matches against (a) the end-user manual's index, (b) each of the four crates' `cargo doc --no-deps` public symbol output, (c) each of the four repos' BIP / SPEC documents. "Every aspect of the software" claim must hold — every public symbol covered in Part V, every wire-format primitive in Part II, every BIP-388 shape in Part III, every bundle invariant in Part IV. Iterate to 0C/0I.

**5.6.3 — Lows/Nits inline** per tag-time discipline.

**5.6.4 — Tag `tech-manual-v1.0.0`.** GitHub release with PDF asset.

**5.6.4.a — Self-row in release history.** Immediately after the tag is pushed, append a `tech-manual-v1.0.0` row to `63-release-history.md` (every prior cut adds the predecessor's row during accretion; the current cut's own row lands as a post-tag amendment). Commit prefix `docs(tech-manual phase 5.6.4.a):`. No re-tag required.

**5.6.5 — Announcement-ready summary** surfaced to user.

---

## Risks during execution

| # | Risk | Mitigation |
|---|---|---|
| ER1 | A sibling repo lands a wire-format change while Track B is in flight | Mirror invariants (SPEC §4.4) fire on the sibling's PR; the corresponding Part II chapter updates in lockstep. If a cut is already tagged, the next cut absorbs the change + bumps the release-history entry. |
| ER2 | `make pdf-docker` reproducibility regresses | Cycle-exit verification re-runs the build cold; if it diverges, debug as a Phase blocker (do not tag until reproducible). |
| ER3 | Worked-example transcript drift | `tests/verify-examples.sh` runs as part of `make lint`; flagged at cycle-exit. |
| ER4 | A code-reviewer round surfaces a scope-relevant Important finding mid-cut | Pause; fold inline; iterate to 0C/0I. Do not advance to next sub-phase with Important findings open. |
| ER5 | A cut exceeds size budget (e.g., Part II in v0.1 overshoots 110pp ceiling) | Architect review at end of the cut decides: scope-cut by deferring sub-sections to a later patch (`tech-manual-v0.1.1`), or accept the overshoot and document the recalibrated sizing in the SPEC. |
| ER6 | Cross-repo PR coordination stalls (a sibling's docs PR not merged when Track B's chapter cites it) | Track B can cite the per-version SPEC instead of the doc PR; once the doc lands, a patch cut backfills the link. |

## Sub-agent dispatch contract

When dispatching reviewer / architect agents during execution, the prompt names:

- The cut version + phase ID (e.g., "tech-manual-v0.1 phase 1.2").
- The specific files in scope for review.
- The SPEC sections the work claims to satisfy.
- The acceptance criteria scoped to this phase.

Reviewers return Critical / Important / Low / Nit categorized findings with file:line citations. Iterate the loop until 0C/0I.
