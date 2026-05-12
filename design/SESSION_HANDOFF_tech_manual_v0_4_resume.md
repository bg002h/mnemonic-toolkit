# Session handoff — `tech-manual-v0.4` execution, begin at Phase 4.0

| Field | Value |
|---|---|
| Created | 2026-05-11 (immediately after `tech-manual-v0.3.0` ship) |
| Pause point | Pre-Phase 4.0 (no Phase 4 work started yet) |
| Resume target | Phase 4.0 (API surface harvest across the four crates) |
| Predecessor cut | `tech-manual-v0.3.0` (tagged 2026-05-11, 145pp PDF, commit `065ce96`, tag object `274771e`) |

## Read these first (in order)

1. **`design/SPEC_tech_manual_v1.md`** — single-volume v1.0 vision; SPEC §4.2.5 defines Part V (four crate chapters with fixed sub-structure: purpose / feature flags / public API by module / error taxonomy / integration patterns / versioning + MSRV).
2. **`design/IMPLEMENTATION_PLAN_tech_manual_v1.md`** — phase decomposition; Phase 4 lives at §"Phase 4 — Cut tech-manual-v0.4". Sub-phases: 4.0 (API surface harvest), 4.1 (§V.1 md-codec), 4.2 (§V.2 mk-codec), 4.3 (§V.3 ms-codec), 4.4 (§V.4 mnemonic-toolkit), 4.5 (api-surface-coverage helper population), 4.6 (back-matter accretion), 4.7 (cycle exit & tag).
3. **`design/SESSION_HANDOFF_tech_manual_v0_3_resume.md`** — the v0.3 handoff (now archival). Captures the operational state at v0.3 entry and the lessons inherited from v0.1+v0.2.
4. **Reviewer reports for v0.3** at `design/agent-reports/tech_manual_v0_3_phase_3_*_review_r1.md` (4 reports — Phases 3.1, 3.2, 3.3, 3.4) + `tech_manual_v0_3_final_review_r1.md` (whole-cut review).

## v0.3 ship state (one-line summary)

- **Tag pushed:** `tech-manual-v0.3.0` at `bg002h/mnemonic-toolkit` (commit `065ce96`; annotated tag object `274771e`).
- **GitHub release:** https://github.com/bg002h/mnemonic-toolkit/releases/tag/tech-manual-v0.3.0
- **PDF asset:** 145pp / 574,086 bytes. SHA256 `b888fcf55c6d4078f9b5d15d9bd2032e50822fbb33918499f2adcfa21b848a11`.
- **Reproducibility:** byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds.
- **Lint:** 6/6 green; `verify-examples.sh` **11/11** transcripts pass (3 new at v0.3: `mnemonic-bundle-bip84-abandon`, `mnemonic-verify-bundle-bip84-abandon`, `mnemonic-bundle-bip388-collision`).
- **Workspace tests:** `cargo test --workspace --all-features` = 527 passed / 0 failed / 2 ignored.
- **Sibling-repo coverage:** unchanged from v0.2 (no sibling code touched this cycle).

## Open FOLLOWUPS (cross-repo, both deferred to md1 work)

Two `cross-repo` entries open in `docs/technical-manual/FOLLOWUPS.md`. Both target `descriptor-mnemonic/crates/md-codec/tests/address_derivation.rs`; both wait for md1 work to begin.

- **`cross-repo md1-wsh-multi-unsorted-integration-test`** (filed Phase 2.2).
- **`cross-repo md1-bip49-integration-test`** (filed post-v0.2-tag).

Neither blocks v0.4 work. Both will resolve in lockstep when md1 work next opens.

## User directives carried into v0.4 work

- **Release-history table policy** (persisted from v0.1): `docs/technical-manual/src/60-back-matter/63-release-history.md` tracks **only** tech-manual cuts (`tech-manual-vX.Y.Z` tags). Phase 4.6 should add the `tech-manual-v0.3.0` row to the table during back-matter accretion.
- **`zero_followups_from_release_cycles`** activates at Phase 4.7 (the cycle-closing tag commit). All findings fold inline; no new FOLLOWUPs at tag time. Mid-cycle (Phases 4.0–4.6) MAY file FOLLOWUPs.
- **Per-phase reviewer loops** are mandatory: every implementation phase ends with a `feature-dev:code-reviewer` round; iterate to 0C/0I before advancing. Persist reports to `design/agent-reports/tech_manual_v0_4_phase_*_review_r{1,2,...}.md`.

## Operational lessons (carried into Phase 4)

Persistent across v0.1 + v0.2 + v0.3. The v0.3 cycle reinforced these:

- **Source-citation precision.** Cite source files at line-precision (`path:LINE` for symbols, `path:START-END` for ranges). v0.3's per-phase reviews caught: an inverted `synthesize.rs:725-593` range, a misclassified `cs[i].path: Option<DerivationPath>` (actual: `DerivationPath` directly), a stale `verify_bundle.rs:649-715` range (correct watch-only discriminator is at `:621-637`), 3 BIP cross-reference mis-citations at Phase 3.4.
- **Don't blindly inherit SPEC or doc-comment prose.** v0.3 Phase 3.1 caught a stale `schema_version: "2"` claim (HEAD emits `"4"`). v0.3 Phase 3.2 documented but didn't fix a live `bundle.rs:259-260` + `error.rs:69-71` v0.4-era doc-comment lag. v0.3 Phase 3.3 caught a Critical: the chapter claimed `rust-codex32 v0.1.0` exposes `Codex32String::shares` for share generation — it doesn't (only `interpolate_at` for reconstruction). For Part V chapters this lesson is doubled: every cited symbol/function/feature-flag MUST be re-grepped against HEAD `cargo doc --no-deps` output, not paraphrased from memory.
- **In-memory Rust struct ≠ wire encoding ≠ public API surface.** Phase 3.5 caught the `md1_xpub_match` multisig-vs-single-sig path disclosure gap. Part V chapters are *exclusively* about the public API surface — be precise about which surface is in scope (e.g., `BundleJson` is a serialization shape; `Bundle` is an in-memory struct; `cmd::bundle::run` is the CLI dispatch which is NOT Part V's concern).
- **Audit doc-claims about test coverage against the actual test file.** Pattern recurring across cuts. For Part V: each crate's `tests/` directory is the source of truth for "what API is actually exercised end-to-end."
- **PDF reproducibility requires CLEAN rebuilds.** `rm -rf build && SOURCE_DATE_EPOCH=... make pdf` twice + `diff`. Stale `build/` is NOT hermetic.
- **CLI input surface ≠ library surface.** v0.2 surfaced this; v0.3 Phase 3.2 surfaced the related template-mode vs. descriptor-mode bifurcation. Part V is explicitly about the **library surface** — the CLI surface belongs in the end-user manual.

## Phase 4 — Cut `tech-manual-v0.4.0` (Rust API reference)

Per IMPLEMENTATION_PLAN §4:

### 4.0 — API surface harvest

- Per crate (`md-codec`, `mk-codec`, `ms-codec`, `mnemonic-toolkit`), run `cargo doc --no-deps` and walk the public surface.
- Generate working notes at `docs/technical-manual/transcripts/api-harvest-<crate>.md` listing every public function, type, trait, module, feature flag, and error variant.
- Reviewer round on the harvest notes.
- **Commit prefix:** `docs(tech-manual phase 4.0):`.

### 4.1 — §V.1 `md-codec`

- Source: `descriptor-mnemonic/crates/md-codec/`.
- Draft chapter at `50-rust-api/51-md-codec-api.md` from SPEC §4.2.5 §V.1 + Phase 4.0 harvest.
- One row per public symbol; one row per `Error::Variant`. Feature flags. Integration patterns (encoder pipeline + decoder pipeline).
- Worked Rust example: `docs/technical-manual/examples/md-codec-api-roundtrip.rs` + transcript pair at `docs/technical-manual/transcripts/md-codec-api-roundtrip.{cmd,out}`. The existing `tests/verify-examples.sh` `.cmd`/`.out` model handles `cargo run --quiet --example md-codec-api-roundtrip` without modification.
- **Commit prefix:** `docs(tech-manual phase 4.1):`.

### 4.2 — §V.2 `mk-codec`

Pattern as Phase 4.1; library-only. Source: `mnemonic-key/crates/mk-codec/`.

### 4.3 — §V.3 `ms-codec`

Pattern as Phase 4.1. Source: `mnemonic-secret/crates/ms-codec/`.

### 4.4 — §V.4 `mnemonic-toolkit`

Pattern as Phase 4.1. Includes JSON envelope schema + engraving-card layout. CLI surface NOT in scope (belongs in the end-user manual).

### 4.5 — API-surface coverage helper

- Populate `tests/api-surface-coverage.sh` (currently a stub created at Phase 1.0.3).
- Implementation: for each of the four crates, run `cargo doc --no-deps --message-format=json` (or `cargo rustdoc -- --output-format json`); extract public top-level symbol names; grep each name against the relevant Part V chapter; emit a warning row per symbol absent from the chapter. **Exit 0 on warnings — this is a hint, not a gate** (per SPEC §4.4).
- Run the helper and ensure no symbols are missing. Resolve any gaps in 4.1–4.4 chapter content before tagging.

### 4.6 — Back-matter accretion

Same pattern as Phase 3.4. Glossary +20 (API terms), index +60, BIP cross-ref completion, release-history row for `tech-manual-v0.3.0`.

### 4.7 — Cycle exit & tag

Same pattern as Phase 3.5: cycle-exit verification, final reviewer round, Lows/Nits inline (`zero_followups_from_release_cycles`), CHANGELOG entry, tag `tech-manual-v0.4.0`, GitHub release with PDF asset, user check-in.

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

# Per-crate public API surface (for Phase 4.0 harvest)
cd /scratch/code/shibboleth/descriptor-mnemonic && cargo doc --no-deps -p md-codec
cd /scratch/code/shibboleth/mnemonic-key && cargo doc --no-deps -p mk-codec
cd /scratch/code/shibboleth/mnemonic-secret && cargo doc --no-deps -p ms-codec
cd /scratch/code/shibboleth/mnemonic-toolkit && cargo doc --no-deps -p mnemonic-toolkit
```

## Commits-so-far table (will accrete during Phase 4)

| Commit | Phase | Description |
|---|---|---|
| _(none yet)_ | 4.0 | API surface harvest (per-crate working notes) |

## After Phase 4

`tech-manual-v1.0.0` (Phase 5) is the final cut. Back-matter polish: full index population (target ≥250), glossary completion (target ≥80), BIP cross-reference table complete, release-history populated through current toolkit tag, bibliography complete. Final architect sign-off on "every aspect of the software" coverage claim. Plan §5.
