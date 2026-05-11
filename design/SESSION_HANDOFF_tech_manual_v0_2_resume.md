# Session handoff — `tech-manual-v0.2` execution, begin at Phase 2.1

| Field | Value |
|---|---|
| Created | 2026-05-11 (immediately after `tech-manual-v0.1.0` ship + FOLLOWUP closures) |
| Pause point | Pre-Phase 2.1 (no Phase 2 work started yet) |
| Resume target | Phase 2.1 (Part III §III.1: Descriptor → miniscript → address) |
| Predecessor cut | `tech-manual-v0.1.0` (tagged 2026-05-11, GitHub release with PDF asset) |

## Read these first (in order)

1. **`design/SPEC_tech_manual_v1.md`** — single-volume v1.0 vision; SPEC §4.2.3 defines Part III's three chapters (§III.1, §III.2, §III.3). SPEC §7 acceptance criteria for v0.2 include A1 partial (one shape walk-through per BIP-388-parseable form — the v0.2 cut introduces this gate).
2. **`design/IMPLEMENTATION_PLAN_tech_manual_v1.md`** — phase decomposition; Phase 2 lives at §"Phase 2 — Cut tech-manual-v0.2" (after the §"Phase 1.6 — Cycle exit & tag" section). Phase 2 sub-phases: 2.1 (§III.1), 2.2 (§III.2), 2.3 (§III.3), 2.4 (back-matter accretion), 2.5 (cycle exit & tag).
3. **`design/SESSION_HANDOFF_tech_manual_v0_1_resume.md`** — the v0.1 handoff doc; preserves the operational state at v0.1 close. Carries forward the source-citation-precision lessons + the per-phase reviewer-loop discipline.
4. **Reviewer reports for v0.1** at `design/agent-reports/tech_manual_v0_1_phase_*_review_r1.md` (8 reports — Phase 0.A, 0.B, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6).

## v0.1 ship state (one-line summary)

- **Tag pushed:** `tech-manual-v0.1.0` at `bg002h/mnemonic-toolkit`.
- **GitHub release:** https://github.com/bg002h/mnemonic-toolkit/releases/tag/tech-manual-v0.1.0
- **PDF asset:** 100pp at v0.1 ship; 97pp after FOLLOWUP closures (release-history table emptied per user directive).
- **Lint:** 6/6 green; `verify-examples.sh` 6/6 green; PDF SOURCE_DATE_EPOCH-byte-identical across clean rebuilds.
- **Sibling-repo coverage:** md-codec v0.32.0, md-cli v0.4.3, mk-codec v0.2.2, ms-codec v0.1.1, ms-cli v0.1.0, mnemonic-toolkit v0.8.0.

## Open FOLLOWUPS

**None at v0.2 entry.** All three v0.1-era follow-ups closed post-tag:

- `md-cli-unspendable-key-v0.19-error-string-stale` (cross-repo) — md1-side commit `df1ed24` retired the stale "v0.19+" reference across 4 sites; toolkit companion closed in lockstep.
- `bibliography-bip-author-canonical-verification` — fetched each cited BIP's canonical mediawiki header; bibliography reconciled (BIP-93, BIP-379, BIP-380, BIP-389 updated; 7 others verified unchanged).
- `troubleshooting-mk-codec-variant-coverage-audit` — added the 5 omitted mk-codec variants; mk1 troubleshooting now covers 22/22 mk-codec Error variants.

See `docs/technical-manual/FOLLOWUPS.md` "Resolved items" section for the full closure record.

## User directives carried into v0.2 work

**Release-history table policy (per user 2026-05-11):** `docs/technical-manual/src/60-back-matter/63-release-history.md` is intentionally empty. The prior sibling-repo release history (md-codec / mk-codec / ms-codec / toolkit versions) is **not** in scope for this manual's coverage. Rows are populated **only** with the technical-manual's own cuts (`tech-manual-vX.Y.Z` tags) as they accumulate. Phase 2.4 should add the `tech-manual-v0.1.0` row to the table during the back-matter accretion sub-phase; no other historical rows belong.

## Operational lessons (carried into Phase 2)

Persistent across Phases 1.1 → 1.6:

- **Source-citation precision.** Cite source files at line-precision (`path:LINE` for symbols, `path:START-END` for ranges). Phase 1.6 caught a fabricated md-codec `MalformedPayloadPadding` variant that had crept into §II.1's canonicality rule 5 and the troubleshooting table; spot-check every error-variant claim against `error.rs`.
- **Don't blindly inherit SPEC prose.** Any factual claim copied from a SPEC or source file must be re-verified against the authoritative source.
- **Re-derive bit-math from first principles.** Phase 1.2 → 1.4 reviewer rounds caught off-by-one and bracket-overflow errors that pure-reasoning would have missed. Phase 2's address-derivation work is more about shape coverage than bit-math, but the discipline carries: every derive_address claim should be cross-validated against an independent `miniscript::Descriptor::from_str(...)` derivation.
- **Iterative review at every phase.** Each implementation phase ends with a `feature-dev:code-reviewer` round. Iterate to 0C/0I before advancing. Persist reports to `design/agent-reports/tech_manual_v0_2_phase_*_review_r{1,2,...}.md`.
- **Tag-time discipline.** `feedback_zero_followups_from_release_cycles` activates at Phase 2.5 (the cycle-closing tag commit). All findings fold inline; no new FOLLOWUPs at tag time. Mid-cycle (Phases 2.1–2.4) can file FOLLOWUPs.

## Phase 2 — Cut `tech-manual-v0.2.0` (Address derivation)

Per IMPLEMENTATION_PLAN §2:

### 2.1 — Part III §III.1 (Descriptor → miniscript → address)

- Source files in scope: SPEC §4.2.3 §III.1.
- Authoritative source-of-truth:
  - `descriptor-mnemonic/crates/md-codec/src/to_miniscript.rs` (the AST → `miniscript::Descriptor` converter shipped at v0.32).
  - `descriptor-mnemonic/crates/md-codec/src/derive.rs` (the `derive_address` entry point).
  - `descriptor-mnemonic/design/SPEC_v0_30_wire_format.md` for the origin-path semantics (Shared vs. Divergent under `Tag::OriginPaths = 0x36`).
  - BIP-388 + BIP-379 + BIP-380 for the framing.
- Required figures: mermaid diagram of the three-tier model (template → derivation → script → address); origin-path semantics diagram.
- Commit prefix: `docs(tech-manual phase 2.1):`.

### 2.2 — Part III §III.2 (Shape coverage)

- SPEC §4.2.3 §III.2. Exhaustive enumeration of v0.32 shapes: `tr(K)`, `tr(K, {script_tree})` single-leaf + multi-leaf, `tr(NUMS, {...})`, `sh(multi)`, arbitrary `wsh(<miniscript>)`, tap-leaf miniscript.
- One worked address-derivation per shape; transcripts captured at `docs/technical-manual/transcripts/` and re-runnable via `verify-examples.sh`.
- Required figure: mermaid diagram of the AST → `miniscript::Descriptor` converter pipeline.
- A1 acceptance gate is partial at v0.2 — one shape walk-through per BIP-388-parseable form (SPEC §7).

### 2.3 — Part III §III.3 (Network and addressing)

- SPEC §4.2.3 §III.3. Mainnet / testnet / regtest / signet. SLIP-0132 prefix cross-reference (full coverage stays in the end-user manual).
- Cross-reference the end-user manual's `convert` workflow chapter (`mnemonic-toolkit/docs/manual/src/`); **do not duplicate** that chapter's content.

### 2.4 — Back-matter accretion

- Glossary additions for Part III terms (miniscript fragments, tap-leaf, NUMS H-point, BIP-388 origin-path Shared vs. Divergent). Target +20 entries (running total ≥50).
- Index additions. Target +50 entries (running total ≥150).
- BIP cross-reference rows for any newly-cited BIPs in Part III.
- Release-history row for `tech-manual-v0.1.0` (per the directive — only tech-manual cuts in this table).

### 2.5 — Cycle exit & tag

Same pattern as Phase 1.6: cycle-exit verification, final reviewer round, Lows/Nits inline (`feedback_zero_followups_from_release_cycles`), CHANGELOG entry, tag `tech-manual-v0.2.0`, GitHub release with PDF asset, user check-in.

## Verification commands (for cycle-exit and per-phase use)

```bash
# Workspace tests
cd /scratch/code/shibboleth/mnemonic-toolkit && cargo test --workspace --all-features

# Manual lint (6 checks)
make -C /scratch/code/shibboleth/mnemonic-toolkit/docs/technical-manual lint

# PDF reproducibility check (clean rebuild + diff)
cd /scratch/code/shibboleth/mnemonic-toolkit/docs/technical-manual && rm -rf build && SOURCE_DATE_EPOCH=1746921600 make pdf

# Worked-example transcripts (the 4 *_BIN paths are pre-built release binaries)
MD_BIN=/scratch/code/shibboleth/descriptor-mnemonic/target/release/md \
  MS_BIN=/scratch/code/shibboleth/mnemonic-secret/target/release/ms \
  MK_BIN=/scratch/code/shibboleth/mnemonic-key/target/release/mk \
  MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/release/mnemonic \
  make -C /scratch/code/shibboleth/mnemonic-toolkit/docs/technical-manual verify-examples
```

## Commits-so-far table (will accrete during Phase 2)

| Commit | Phase | Description |
|---|---|---|
| _(none yet)_ | 2.1 | Part III §III.1 (descriptor → miniscript → address) |

## After Phase 2

`tech-manual-v0.3.0` adds Part IV (bundle formation). Plan §3.
