# Session handoff — `tech-manual-v0.1` execution, paused at Phase 1.2 close

| Field | Value |
|---|---|
| Created | 2026-05-11 |
| Pause point | Phase 1.2 closed 0C/0I; Phase 1.3 not yet started |
| Reason for pause | Context-budget management; clean break between dense wire-format chapters |
| Resume target | Phase 1.3 (mk1 wire format) |

## Read these first (in order)

1. **`design/SPEC_tech_manual_v1.md`** — single-volume v1.0 vision; v0.1 cut scope defined at §6.
2. **`design/IMPLEMENTATION_PLAN_tech_manual_v1.md`** — phase decomposition; cross-cutting conventions (commit prefixes, reviewer-loop discipline, cycle-exit verification, sub-agent dispatch contract).
3. **Both architect-review reports for SPEC + PLAN** — embedded in this repo's commit `a7c7742` message; no separate file (architect rounds for SPEC + PLAN don't persist per `feedback_iterative_review_every_phase` discipline).
4. **`design/agent-reports/tech_manual_v0_1_phase_1_0_review_r1.md`** — pipeline scaffold review.
5. **`design/agent-reports/tech_manual_v0_1_phase_1_1_review_r1.md`** — Foundations chapters; **2 Important factual errors caught** in the BCH chapter (md1's long code was dropped; the polynomial vs target-residue distinction). Re-read this before drafting any chapter that makes BCH claims.
6. **`design/agent-reports/tech_manual_v0_1_phase_1_2_review_r1.md`** — md1 wire format chapter; **5 Important factual errors caught** in the worked-encode + worked-decode walks (off-by-one bit/symbol arithmetic, missing 5-bit `n-1` path-decl prefix, wrong walker-normalisation source-file pointer). Re-read this before drafting Phase 1.3 (mk1) or Phase 1.4 (ms1) wire-format chapters.

## Operational lesson (twice-confirmed)

**Wire-format chapters require source-cited drafting, not narrative paraphrase.** Phase 1.1 reviewer caught 2 BCH-chapter Importants; Phase 1.2 reviewer caught 5 in the md1 chapter. For Phases 1.3 + 1.4, **before drafting any concrete claim**:

- Run `cargo run --quiet -p mk-cli -- ...` or `ms-cli -- ...` for each worked-encode/decode example, capture output verbatim.
- `git show HEAD:crates/mk-codec/src/<file>.rs` and grep the actual implementation before writing about it.
- Cross-cite each substantive claim to a specific BIP-draft section OR a specific source-file:line.
- Walk through the bit math for each worked example one field at a time, *counting bits*, before writing the totals row.

The two reviewer reports include the source-file paths to grep against.

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

## Cross-repo FOLLOWUPS

One pair filed during Phase 0.B (Lows surfaced during the toolkit audit but action is in md1):

- Primary: `descriptor-mnemonic/design/FOLLOWUPS.md` → `md-cli-unspendable-key-v0.19-error-string-stale`
- Companion: `mnemonic-toolkit/design/FOLLOWUPS.md` → `md-cli-unspendable-key-v0.19-error-string-stale-companion`

Both tier `cross-repo`. One-line fix to `descriptor-mnemonic/crates/md-cli/src/main.rs:224` ("track v0.19+ for caller-supplied internal-key support" → "deferred to a future version"). Pending an md1-side patch cycle to close both in lockstep.

## Current state of the manual

- PDF: **58 pp** at end of Phase 1.2 (start was 30pp stub at Phase 1.0; +28pp content added).
- Lint: 6/6 green (`make -C docs/technical-manual lint`).
- PDF reproducibility: `SOURCE_DATE_EPOCH=1746921600 make pdf` byte-identical across consecutive runs.
- Mermaid figures: 1 (constellation star, in `12-the-m-format-star.md`); rendered cache at `docs/technical-manual/figures/cache/353df7ac…1681dd3.pdf`.
- Worked-example transcripts: 2 (`transcripts/md1-encode-wpkh-basic.{cmd,out}`, `transcripts/md1-decode-wsh-multi-2of3.{cmd,out}`); both verified against HEAD `md` binary.
- Index table: 1 entry (`m-format constellation`).
- Glossary: 11 seed entries.

## Resume plan

When you resume, the next phase is **Phase 1.3 — mk1 wire format chapter**.

1. **Pre-draft source review:**
   - Read `bg002h/mnemonic-key/design/SPEC_*.md` (mk-codec's current SPEC; pick the latest).
   - Read `bg002h/mnemonic-key/bip/...` if a BIP draft exists.
   - Read `bg002h/mnemonic-key/crates/mk-codec/src/*.rs` files for header, encode, decode, BCH plumbing.
   - Note: mk1 dropped the path-dictionary mirror with md1 at md-codec v0.11 (the mirror invariant is RETIRED per `descriptor-mnemonic/CLAUDE.md`). mk1's path dictionary is mk1-internal — the chapter should note this.

2. **Source-cited drafting:**
   - For each chapter section that makes a concrete claim, cite the BIP draft section OR the source-file:line.
   - Walk the worked-encode bit math one field at a time. Don't write the totals row until you've verified the actual encoded card length matches.
   - Run `mk encode '<xpub>'` against the HEAD `mk` binary; capture the output verbatim; use that as the worked-example output. Same for `mk decode`.

3. **Mirror Phase 1.2 structure:**
   - Layer model (encoding + bytecode).
   - Encoding-layer framing.
   - Bytecode layer (mk1's per-card payload structure).
   - Tag table (or equivalent — mk1 has a different field set than md1).
   - Body shapes / TLV section.
   - Canonicality rules.
   - One worked encode + one worked decode with transcripts.
   - History note (mk1-internal path dictionary retirement post md-codec v0.11).
   - Cross-references to §I.3 (shared BCH plumbing), §IV.2 (cross-card invariants — `policy_id_stub` is carried on mk1), §V.2 (`mk-codec` Rust API).

4. **Per-phase reviewer round to 0C/0I** — same `feature-dev:code-reviewer` dispatch pattern as Phase 1.2. Use the Phase 1.2 review prompt as a template; substitute mk1 specifics.

5. **Commit prefix:** `docs(tech-manual phase 1.3): ...`. Persist review report at `design/agent-reports/tech_manual_v0_1_phase_1_3_review_r1.md`.

## After Phase 1.3

- **Phase 1.4** (ms1 wire format) — same pattern; the ms1 chapter is the simplest because ms1 uses BIP-93 codex32 directly (the bytecode-layer surface is much smaller than md1's).
- **Phase 1.5** (back-matter skeleton) — glossary expansion (target ≥30 entries at v0.1 close), index population (target ≥100 entries), release-history seed, BIP cross-ref seed, troubleshooting seed, bibliography seed.
- **Phase 1.6** (cycle exit + `tech-manual-v0.1.0` tag) — final whole-cut reviewer round; cycle-exit verification; CHANGELOG entry; tag + GitHub release with PDF asset.

## What I won't lose across the session boundary

This handoff document + the three agent-reports under `design/agent-reports/tech_manual_v0_1_phase_*_review_r1.md` carry the operational state. The SPEC + PLAN are the authoritative scope. Memory entry for this in-flight work will also flag it on resume.
