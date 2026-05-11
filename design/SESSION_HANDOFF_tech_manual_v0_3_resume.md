# Session handoff — `tech-manual-v0.3` execution, begin at Phase 3.1

| Field | Value |
|---|---|
| Created | 2026-05-11 (immediately after `tech-manual-v0.2.0` ship + FOLLOWUP filings) |
| Pause point | Pre-Phase 3.1 (no Phase 3 work started yet) |
| Resume target | Phase 3.1 (Part IV §IV.1: Bundle anatomy) |
| Predecessor cut | `tech-manual-v0.2.0` (tagged 2026-05-11, GitHub release with 119pp PDF) |

## Read these first (in order)

1. **`design/SPEC_tech_manual_v1.md`** — single-volume v1.0 vision; SPEC §4.2.4 defines Part IV's three chapters (§IV.1 Bundle anatomy, §IV.2 Anti-collision invariants, §IV.3 Future shares). SPEC §7 acceptance criteria scoped to v0.3 include glossary ≥65 / index ≥190 (cumulative targets per Phase 3.4's accretion plan).
2. **`design/IMPLEMENTATION_PLAN_tech_manual_v1.md`** — phase decomposition; Phase 3 lives at §"Phase 3 — Cut tech-manual-v0.3". Sub-phases: 3.1 (§IV.1), 3.2 (§IV.2), 3.3 (§IV.3), 3.4 (back-matter accretion), 3.5 (cycle exit & tag).
3. **`design/SESSION_HANDOFF_tech_manual_v0_2_resume.md`** — the v0.2 handoff (now archival). Captures the operational state at v0.2 entry and the lessons inherited from v0.1.
4. **Reviewer reports for v0.2** at `design/agent-reports/tech_manual_v0_2_phase_*_review_r1.md` (5 reports — Phases 2.1, 2.2, 2.3, 2.4, plus the final whole-cut review).

## v0.2 ship state (one-line summary)

- **Tag pushed:** `tech-manual-v0.2.0` at `bg002h/mnemonic-toolkit` (commit `113ed92`).
- **GitHub release:** https://github.com/bg002h/mnemonic-toolkit/releases/tag/tech-manual-v0.2.0
- **PDF asset:** 119pp / 493KB. SHA256 `4f34bd444e979536882eeaf488a130e5aae849b45ee17d87f430223470d95a90`.
- **Reproducibility:** byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds.
- **Lint:** 6/6 green; `verify-examples.sh` **8/8** transcripts pass.
- **Sibling-repo coverage:** unchanged from v0.1 (no sibling code touched this cycle — md-codec v0.32.0, md-cli v0.4.3, mk-codec v0.2.2, ms-codec v0.1.1, ms-cli v0.1.0, mnemonic-toolkit v0.8.0).
- **Workspace tests:** `cargo test --workspace --all-features` = 527 passed / 0 failed / 2 ignored.

## Open FOLLOWUPS (cross-repo, both deferred to md1 work)

Two `cross-repo` entries open in `docs/technical-manual/FOLLOWUPS.md`. Both target the same file (`descriptor-mnemonic/crates/md-codec/tests/address_derivation.rs`); both wait for md1 work to begin before the md1-side mirror entries get filed.

- **`cross-repo md1-wsh-multi-unsorted-integration-test`** (filed Phase 2.2) — add paired-derivation test for unsorted `wsh(multi(...))`. Routes through `node_to_miniscript::<Segwitv0>` `Terminal::Multi` arm at `to_miniscript.rs:365-373`; presently untested.
- **`cross-repo md1-bip49-integration-test`** (filed post-v0.2-tag) — add BIP-49 P2SH-P2WPKH integration test. BIP-49 §"Test vectors" provides `m/49'/1'/0'/0/0` → `2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2`. The module doc-comment at `address_derivation.rs:26` already claims BIP-49 coverage, but no `bip49_*` test exists.

Neither blocks v0.3 work. Both will resolve in lockstep when md1 work next opens.

## User directives carried into v0.3 work

- **Release-history table policy** (from v0.1, persisted): `docs/technical-manual/src/60-back-matter/63-release-history.md` tracks **only** tech-manual cuts (`tech-manual-vX.Y.Z` tags). Phase 3.4 should add the `tech-manual-v0.2.0` row to the table during the back-matter accretion sub-phase; no sibling-repo rows.
- **`zero_followups_from_release_cycles`** activates at Phase 3.5 (the cycle-closing tag commit). All findings fold inline; no new FOLLOWUPs at tag time. Mid-cycle (Phases 3.1–3.4) MAY file FOLLOWUPs.
- **Per-phase reviewer loops** are mandatory: every implementation phase ends with a `feature-dev:code-reviewer` round; iterate to 0C/0I before advancing. Persist reports to `design/agent-reports/tech_manual_v0_3_phase_*_review_r{1,2,...}.md`.

## Operational lessons (carried into Phase 3)

Persistent across v0.1 + v0.2. The v0.2 cycle reinforced these:

- **Source-citation precision.** Cite source files at line-precision (`path:LINE` for symbols, `path:START-END` for ranges). v0.2's reviewer rounds caught fabricated BIP section titles ("BIP-388 §'Wallet policies'" — doesn't exist), stale cross-reference subsection-title anchors ("§II.1 §'History note on retired dictionaries'" — actual heading was "retired wire-layer dictionaries"), an inverted wire-format claim (NUMS `key_index` wire-suppressed but chapter said wire-present), and a mis-cited test (sortedmulti test cited as cross-validation for unsorted multi).
- **Don't blindly inherit SPEC prose.** SPEC §4.2.3 §III.1 referenced `Tag::OriginPaths = 0x36` (a v0.10 reference retired in v0.11). The chapter correctly used the current header-bit-4 mechanism and added a history note instead. Same discipline applies to Part IV: any factual claim copied from a SPEC or doc must be re-verified against HEAD source.
- **In-memory Rust struct ≠ wire encoding.** Phase 2.2's Critical was the NUMS conflation: the `Body::Tr` Rust struct carries `key_index: u8` regardless of `is_nums`, but the wire encoder suppresses the kiw-bit field when `is_nums = 1`. When the chapter is about wire format vs. address derivation vs. API, be precise about which surface is in scope.
- **Audit doc-claims about test coverage against the actual test file.** Post-tag user audit of `address_derivation.rs` revealed that the file's module doc-comment claimed BIP-49 coverage that didn't exist. Same class as Phase 2.2's `wsh(multi(...))` row mis-citing a sortedmulti test. When prose says "tested via T", grep for T and confirm T tests the claimed thing.
- **PDF reproducibility requires CLEAN rebuilds.** `rm -rf build && SOURCE_DATE_EPOCH=... make pdf` twice + `diff`. Stale `build/` is NOT hermetic.
- **CLI input surface ≠ library surface.** v0.2 surfaced that `md address --template` rejects depth-3 xpubs for `tr(...)` shapes (because `ctx_for_template` classifies non-`wpkh`/`pkh`/`sh(wpkh)` as MultiSig and requires depth 4) and rejects SLIP-0132 prefixes. The library admits more shapes than the CLI exercises. Part V (§4) will need to be explicit about this distinction.

## Phase 3 — Cut `tech-manual-v0.3.0` (Bundle formation)

Per IMPLEMENTATION_PLAN §3:

### 3.1 — Part IV §IV.1 (Bundle anatomy)

- SPEC §4.2.4 §IV.1.
- Source-of-truth files:
  - `mnemonic-toolkit/crates/mnemonic-toolkit/src/bundle.rs` (or wherever the bundle envelope is defined) — read before drafting.
  - `mnemonic-toolkit/crates/mnemonic-toolkit/src/verify_bundle.rs` (bundle verification entry point).
  - The end-user manual's `30-workflows/` chapters for the bundle-creation walk-through (cite, don't duplicate).
- Required figures: mermaid bundle creation pipeline + bundle verification pipeline.
- Commit prefix: `docs(tech-manual phase 3.1):`.

### 3.2 — Part IV §IV.2 (Anti-collision invariants)

- SPEC §4.2.4 §IV.2.
- `chunk_set_id` derivation (re-cite §II.1's definition; new content is the cross-card binding role at bundle level).
- Multiset `md1_xpub_match` rule (set-equality with multiplicity; sort-then-compare).
- Four-case ms1 short-circuit table (toolkit SPEC v0.5+).
- mk1 cosigner-mapping diagnostic (`NotSupplied` / `DecodeFailed` / `XpubNotInPolicy`).
- BIP-388 distinct-key enforcement (typed `DerivationPath` equality; `h` ↔ `'` folding).
- Worked example: a colliding bundle and the diagnostic output. Transcript captured.

### 3.3 — Part IV §IV.3 (Future shares)

- SPEC §4.2.4 §IV.3. v0.1 → v0.2-shares migration invariants locked across all three formats. Why ms1 ships first (BIP-93 already specifies the math).

### 3.4 — Back-matter accretion

- Glossary additions for Part IV terms (bundle envelope, engraving card, cosigner-mapping diagnostic, etc.). Target +15 entries (running total ≥72).
- Index additions. Target +40 entries (running total ≥199).
- BIP cross-reference rows for any newly-cited BIPs in Part IV.
- **Release-history row for `tech-manual-v0.2.0`** (per the directive — only tech-manual cuts).

### 3.5 — Cycle exit & tag

Same pattern as Phase 1.6 / Phase 2.5: cycle-exit verification, final reviewer round, Lows/Nits inline (`zero_followups_from_release_cycles`), CHANGELOG entry, tag `tech-manual-v0.3.0`, GitHub release with PDF asset, user check-in.

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

## Commits-so-far table (will accrete during Phase 3)

| Commit | Phase | Description |
|---|---|---|
| _(none yet)_ | 3.1 | Part IV §IV.1 (bundle anatomy) |

## After Phase 3

`tech-manual-v0.4.0` adds Part V (Rust API reference for all four crates). Plan §4.
