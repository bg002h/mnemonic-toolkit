# Session handoff — `tech-manual-v0.1` execution, paused at Phase 1.5 close

| Field | Value |
|---|---|
| Created | 2026-05-11 (initial, at Phase 1.2 close) |
| Last refreshed | 2026-05-11 (at Phase 1.5 close) |
| Pause point | Phase 1.5 closed 0C/0I (2I + 3L + 1N folded inline; 2L deferred via FOLLOWUPS); Phase 1.6 not yet started |
| Reason for pause | Phase 1.6 includes risky shared-state actions (`git push --tags`, `gh release create`); user confirmation required before those fire |
| Resume target | Phase 1.6 (cycle exit + `tech-manual-v0.1.0` tag) |

## Read these first (in order)

1. **`design/SPEC_tech_manual_v1.md`** — single-volume v1.0 vision; v0.1 cut scope defined at §6. SPEC §7 acceptance criteria scoped to v0.1 are A4/A5/A6/A8/A10 (NOT A1 — that gates on Part III at v0.2).
2. **`design/IMPLEMENTATION_PLAN_tech_manual_v1.md`** — phase decomposition; cross-cutting conventions (commit prefixes, reviewer-loop discipline, cycle-exit verification, sub-agent dispatch contract). Phases 1.5 + 1.6 detailed at lines ~196–229.
3. **Reviewer reports for Phases 1.0 → 1.4** (six files under `design/agent-reports/tech_manual_v0_1_phase_*_review_r1.md`). Re-read the Phase 1.3 + 1.4 reports' "Operational note" sections in particular — they capture the source-citation precision lessons that carry forward.

## Operational lessons (carried into Phase 1.5 + 1.6)

The wire-format-chapter reviewer-Importants trend was **2 → 5 → 1 → 1** across Phases 1.1 → 1.2 → 1.3 → 1.4. The discipline that drove the reduction:

- **Explicit `3 (HRP+sep) + 1 (threshold) + 4 (id) + 1 (share) + N (payload) + 13 (cksum) = M (total)` decomposition at every quoted character count.** Phase 1.3 introduced this; Phase 1.4's worked walks were error-free.
- **Re-derive bit-math from first principles against SHA-pinned vectors at draft time** (Python script over the canonical hex). Don't trust SPEC tables alone.
- **Cite source files at line-precision**: `path/to/file.rs:LINE` for symbols, `path/to/file.rs:START-END` for ranges, `path/to/file.rs:A,B` for non-contiguous pair-citations. Phase 1.4's L1 finding was an off-by-2 line-start.
- **Don't blindly inherit SPEC prose**: Phase 1.4's Important was an SPEC-inherited parenthetical drift ("length ≥ 96" should have been "99–111 for HRP=ms"). When copying any factual claim from a SPEC into the chapter, re-verify against the authoritative source (BIP-93, source code, etc.).

For Phase 1.5 (back-matter), the bit-math discipline is no longer load-bearing, but the **source-citation precision lesson does carry forward** — every glossary entry, every BIP cross-reference row, every release-history row needs an authoritative cite (BIP-NNN §"Section", per-repo CHANGELOG line, etc.).

For Phase 1.6 (cycle exit + tag), the **`feedback_zero_followups_from_release_cycles` rule activates**: zero new FOLLOWUPs at the tag-time commit. ALL reviewer findings (Critical/Important/Low/Nit) fold inline at Phase 1.6. Mid-cycle commits (Phase 1.5's per-sub-phase commits) MAY file FOLLOWUPs for non-trivial Lows; the tag-time discipline is for the final cycle-closing commit only.

## Commits so far (in chronological order)

### `descriptor-mnemonic` (md1 repo) — `main` branch

| Commit | Phase | Description |
|---|---|---|
| `d47423e` | 0.A | md1-repo doc audit: README, MIGRATION, CHANGELOG, json-schema, md-codec README, md-cli README updated for v0.30/v0.31/v0.32 |
| `7faf2ef` | 0.A close | Fold reviewer Low+Nit inline; review report persisted |
| `16393f0` | 0.B companion | Cross-repo FOLLOWUP: stale `v0.19+` reference in md-cli error string |

### `mnemonic-toolkit` (toolkit repo) — `master` branch

| Commit | Phase | Description |
|---|---|---|
| `a7c7742` | SPEC + PLAN | Cross-repo technical-manual SPEC + PLAN; architect 0C/0I across 2 rounds |
| `713178c` | 0.B | toolkit-repo doc audit: 42-md.md mirror chapter refreshed |
| `e342c25` | 0.B close | Cross-repo FOLLOWUPS filed; review report persisted |
| `445845c` | 1.0 | Pipeline scaffold: 45 files; cloned + adapted from docs/manual/; api-surface-coverage stub; SOURCE_DATE_EPOCH wiring |
| `2e80f17` | 1.0 close | Fold reviewer I+L+N inline; SOURCE_DATE_EPOCH forwarded into pdf-docker; AUTHORING path fixes; .gitkeep files |
| `b617976` | 1.1 | Frontmatter + Part I Foundations: 4 chapters + mermaid figure |
| `6c13a6c` | 1.1 close | Fold reviewer 2I + L + N inline: BCH chapter substantively rewritten (long code retired; target-residue fork explained accurately) |
| `e287bb8` | 1.2 | md1 wire format chapter (§II.1) — heaviest single chapter; 300 lines |
| `34842d6` | 1.2 close | Fold reviewer 5I inline: worked-encode + worked-decode arithmetic corrected; walker normalisation cite fixed |
| `11b994d` | (handoff) | Session handoff at Phase 1.2 close (this doc, prior version) |
| `f609479` | 1.3 | mk1 wire format chapter (§II.2): ~310 lines + 2 mermaid figures + 2 transcripts |
| `3895501` | 1.3 close | Fold reviewer 1I + 1L inline: data-part-vs-total-string mixup ("108-char chunk 0" → "111-char"); reserved-range notation `0x08..0x10` → `0x08..=0x10` |
| `cdc807b` | 1.4 | ms1 wire format chapter (§II.3): ~300 lines + 1 mermaid figure + 2 transcripts |
| `6d53a3f` | 1.4 close | Fold reviewer 1I + 2L inline: long-code bracket parenthetical corrected to "99–111 for HRP=ms"; citation line range corrected; 3 RESERVED_TAG_TABLE rows made consistent |
| `b74218a` | (handoff) | Session handoff at Phase 1.4 close (prior version of this doc) |
| `ae5bb51` | 1.5 | Back-matter skeleton + index/glossary population: 6 chapters seeded; 86 new `\index{}` markers across 7 chapters; 22 new cspell dict entries |
| `3e54791` | 1.5 close | Fold reviewer 2I + 3L + 1N inline; persist report. 2L deferred via FOLLOWUPS (`bibliography-bip-author-canonical-verification`, `troubleshooting-mk-codec-variant-coverage-audit`) |

## Cross-repo FOLLOWUPS

One pair still open from Phase 0.B (Lows surfaced during the toolkit audit but action is in md1):

- Primary: `descriptor-mnemonic/design/FOLLOWUPS.md` → `md-cli-unspendable-key-v0.19-error-string-stale`
- Companion: `mnemonic-toolkit/design/FOLLOWUPS.md` → `md-cli-unspendable-key-v0.19-error-string-stale-companion`

Both tier `cross-repo`. One-line fix to `descriptor-mnemonic/crates/md-cli/src/main.rs:224` ("track v0.19+ for caller-supplied internal-key support" → "deferred to a future version"). Pending an md1-side patch cycle to close both in lockstep — does not block Phase 1.5/1.6.

## Current state of the manual

- **PDF: 100 pp** at end of Phase 1.5 (was 83pp at Phase 1.4 close; +17pp from back-matter skeleton). Within SPEC §6 v0.1 bracket [40, 110].
- **Lint: 6/6 green** (`make -C docs/technical-manual lint`).
- **PDF reproducibility:** `SOURCE_DATE_EPOCH=1746921600 make pdf` byte-identical across **clean** rebuilds (`rm -rf build && SOURCE_DATE_EPOCH=1746921600 make pdf` twice, then `diff`). Stale build state is NOT hermetic — always clean-rebuild for the reproducibility check.
- **Mermaid figures: 4** (1 constellation-star in `12-the-m-format-star.md`; 2 in `22-mk1-wire-format.md`; 1 in `23-ms1-wire-format.md`). Rendered cache committed under `docs/technical-manual/figures/cache/`.
- **Worked-example transcripts: 6** — 2 md1, 2 mk1, 2 ms1. All green via `verify-examples.sh` against HEAD `md`/`mk`/`ms` binaries.
- **Index table: 96 entries** (was 10 at Phase 1.4). Covers Parts I + II markers; lint bidirectional check green. Plan target was "~100 entries at v0.1" — within reasonable tolerance.
- **Glossary: 31 entries** (was 11 at Phase 1.4). Plan target was ~30 at v0.1 — satisfied.
- **Release-history table: 11 rows.** Covers ms-codec v0.1.0/v0.1.1, ms-cli v0.1.0, mk-codec v0.2.2, mnemonic-toolkit v0.7.0/v0.7.1/v0.8.0, md-codec v0.30.0/v0.31.0/v0.32.0, md-cli v0.4.3.
- **BIP cross-reference: 11 BIPs.** BIP-32, 39, 48, 84, 93, 173, 341, 342, 380, 388, 389 mapped to Parts I + II citations.
- **Troubleshooting matrix: 35 error variants** across 3 codecs (curated subset; not exhaustive — `troubleshooting-mk-codec-variant-coverage-audit` FOLLOWUP filed for Phase 4 / v1.0 expansion).
- **Bibliography: 11 BIPs + 3 academic refs + 2 reference impls + 6 per-version SPECs.** Per-BIP author lists deferred to `bibliography-bip-author-canonical-verification` FOLLOWUP (tier `tech-manual-v1.0-nice-to-have`).

## Sibling-repo binaries for `verify-examples`

Pre-built release binaries exist in each sibling worktree:

```bash
MD_BIN=/scratch/code/shibboleth/descriptor-mnemonic/target/release/md
MS_BIN=/scratch/code/shibboleth/mnemonic-secret/target/release/ms
MK_BIN=/scratch/code/shibboleth/mnemonic-key/target/release/mk
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/release/mnemonic
```

`make -C docs/technical-manual verify-examples` with these four env-vars exported produces "OK (6 transcripts pass)" against HEAD.

## Resume plan

When you resume, the next phase is **Phase 1.6 — cycle exit + `tech-manual-v0.1.0` tag**. See the "After Phase 1.5" section below for the sub-phase decomposition.

**Open FOLLOWUPS at resume:**

- `docs/technical-manual/FOLLOWUPS.md::bibliography-bip-author-canonical-verification` (tier `tech-manual-v1.0-nice-to-have`).
- `docs/technical-manual/FOLLOWUPS.md::troubleshooting-mk-codec-variant-coverage-audit` (tier `tech-manual-v0.4`).
- Cross-repo: `descriptor-mnemonic/design/FOLLOWUPS.md::md-cli-unspendable-key-v0.19-error-string-stale` + companion (still open from Phase 0.B; not a Phase 1.6 blocker).

**Important for Phase 1.6:** `feedback_zero_followups_from_release_cycles` activates at tag-time. ALL reviewer findings (Critical / Important / Low / Nit) fold inline at Phase 1.6's closing commit. Existing FOLLOWUPS above are not affected (those were filed during mid-cycle Phase 1.5, and the rule applies to *new* findings at tag-time). Sub-phases 1.6.5 and 1.6.6 (push tag, create GitHub release) are shared-state actions requiring user confirmation per the system instructions.

---

## (Archived) Resume plan for Phase 1.5 (executed in commits `ae5bb51` + `3e54791`)

### 1.5.1 — Glossary stubs (`60-back-matter/61-glossary.md`)

Seed with terms introduced in Parts I + II. Target ~30 entries at v0.1. The existing 11 entries are at Phase 1.1; Phases 1.2–1.4 introduced many new terms. Walk each chapter for terms warranting glossary entries (mk1: chunk_set_id, cross_chunk_hash, compact-73, policy_id_stub, Wallet Instance ID, Tag::ENTR, …; ms1: BIP-93, reserved-prefix byte, RESERVED_TAG_TABLE, codex32, BIP-39 entropy, …; md1: TLV section, OriginPath, NUMS H-point, walker normalization, …). Each entry: one-sentence definition + section pointer to first definitional use.

### 1.5.2 — Index population (`60-back-matter/62-index-table.md`)

Walk each chapter for terms that warrant `\index{}` markers but lack them. Add markers in source; add matching rows here. Target ~100 entries at v0.1. The bidirectional `tests/lint.sh` check gates this — every marker needs a row and vice versa. Currently 10 entries (see "Current state of the manual" above).

### 1.5.3 — Release-history seed (`60-back-matter/63-release-history.md`)

Stub table; seed with the four repos' currently-tagged versions. Acceptance: rows for at least `md-codec-v0.30`, `md-codec-v0.31`, `md-codec-v0.32`, `mk-codec-v0.2.2`, `ms-codec-v0.1.1`, `toolkit-v0.8.0` (per IMPLEMENTATION_PLAN Phase 1.5.3). Each row: date, version, one-line summary, pointer to per-repo CHANGELOG entry.

### 1.5.4 — BIP cross-reference seed (`60-back-matter/64-bip-cross-reference.md`)

Stub table; seed with BIPs cited in Parts I + II. Likely: BIP-32, BIP-39, BIP-44, BIP-48, BIP-49, BIP-84, BIP-86, BIP-87, BIP-93, BIP-173, BIP-340, BIP-341, BIP-380, BIP-388. Each row: BIP number + title + sections of the manual that cite it.

### 1.5.5 — Troubleshooting seed (`60-back-matter/65-troubleshooting.md`)

Stub with one section per format; populate decoder-error rows earned by Parts I + II. Source-of-truth for the error variants: `descriptor-mnemonic/crates/md-codec/src/error.rs`, `mnemonic-key/crates/mk-codec/src/error.rs`, `mnemonic-secret/crates/ms-codec/src/error.rs`.

### 1.5.6 — Bibliography seed (`60-back-matter/66-bibliography.md`)

Initial bibliography: BIP-32, BIP-39, BIP-93, BIP-340, BIP-341, BIP-379, BIP-388, codex32 paper, miniscript paper, FROST RFC 9591.

### 1.5.7 — `99-build-banner.md` review

Already exists; verify content is current.

### 1.5.8 — Phase commit + reviewer round

Commit prefix `docs(tech-manual phase 1.5): ...`. Reviewer dispatch with focus on: glossary entries are accurate definitions, BIP cross-reference rows cite real BIP sections, release-history dates match per-repo CHANGELOGs. Iterate to 0C/0I. Persist report at `design/agent-reports/tech_manual_v0_1_phase_1_5_review_r1.md`. **Mid-cycle policy** applies: L/N may go to FOLLOWUPs.

## After Phase 1.5

**Phase 1.6 — Cycle exit + `tech-manual-v0.1.0` tag.**

- **1.6.1 cycle-exit verification:** `cargo test --workspace --all-features` (toolkit) green; `make -C docs/technical-manual lint && make pdf-docker` green; PDF page count in `[60, 110]` per SPEC §6 (currently 83pp → after Phase 1.5 back-matter, expect 86–90pp); all 6 worked-example transcripts re-run green via `verify-examples.sh`.
- **1.6.2 final reviewer round:** scope to SPEC §7 v0.1-applicable criteria — A4 (glossary ≥30), A5 (index ≥100), A6 (TOC), A8 (transcripts verified), A10 (PDF ≥40pp soft floor). A1 (every BIP-388 shape walk-through) is NOT in scope at v0.1 — it gates on Part III which ships at v0.2. Iterate to 0C/0I.
- **1.6.3 Lows/Nits inline:** `feedback_zero_followups_from_release_cycles` — fold ALL findings inline at this commit; zero new FOLLOWUPs.
- **1.6.4 CHANGELOG entry:** append a `tech-manual-v0.1.0` row to the toolkit's `CHANGELOG.md`.
- **1.6.5 tag:** `git tag -a tech-manual-v0.1.0 -m "…" && git push --tags`.
- **1.6.6 GitHub release:** `gh release create tech-manual-v0.1.0 docs/technical-manual/build/m-format-technical-manual.pdf` — release notes point to the SPEC.
- **1.6.7 user check-in:** surface release URL + PDF asset to user. Pause before any v0.2 work.

## What I won't lose across the session boundary

This handoff document + the seven agent-reports under `design/agent-reports/tech_manual_v0_1_phase_*_review_r1.md` (Phases 0.A, 0.B, 1.0, 1.1, 1.2, 1.3, 1.4) carry the operational state. The SPEC + PLAN are the authoritative scope. The memory entry `tech_manual_v0_1_in_flight` will also flag this on resume (description: "Phase 1.4 closed 0C/0I/0L/0N 2026-05-11; Phase 1.5 (back-matter skeleton) is next").

The four sibling-repo SPEC + BIP + source files are no longer load-bearing — back-matter work derives from the chapter content + per-repo CHANGELOG / BIP cite metadata, not from re-reading mk-codec / ms-codec internals. Phase 1.5 + 1.6 work is qualitatively different from Phases 1.1–1.4 in this respect.
