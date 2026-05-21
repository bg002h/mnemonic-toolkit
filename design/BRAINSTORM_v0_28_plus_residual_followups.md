# BRAINSTORM — v0.28+ residual FOLLOWUP release plan (15 slugs, 8 cycles, 4 waves)

**Date:** 2026-05-20 (post-A/B/C-cycle ship).
**Source SHA at brainstorm time:** `0ca86b5` (post-Round-2-recon-addendum).
**Sync state:** local master ≡ origin/master.
**Recon dossier:** `cycle-prep-recon-followups-v0_28_plus.md` (this branch root); two-round recon verifies 15 v0.28+ open FOLLOWUPs.
**Predecessor brainstorm:** `design/BRAINSTORM_followups_abc_release_plan.md` (A/B/C cycle, 4 cycles / 5 FOLLOWUPs / shipped 2026-05-20 in this session).

This brainstorm decomposes the 15 reconned v0.28+ open FOLLOWUPs into 8 release cycles across 4 waves (9 if Cycle 7 splits into 7a + 7b per architect I4). It does NOT implement; the next phase invokes the `writing-plans` skill (deferred per `feedback-followups-md-line-numbers-presumed-stale` to per-cycle dispatches) to produce a `PLAN_*.md` per cycle when each is ready to execute.

## Decisions locked (with the user, in this brainstorm session)

1. **Comprehensive multi-cycle plan** covering all 15 v0.28+ open FOLLOWUPs (A/B/C-style decomposition).
2. **Wave sequencing:** Wave 1 (docs+tests parallel) → Wave 2 (hardening) → Wave 3 (refactor + SemVer-minor) → Wave 4-7 (parsers individual, multi-week each).
3. **All 4 parsers in scope** (jade-seedqr / electrum-encrypted / bsms-encryption / sparrow-taproot); sparrow-taproot promotes from v0.29+ tier to in-scope.
4. **Cycle granularity:** Hybrid — Waves 1-3 bundle by character; Wave 4+ parsers individual = **8 cycles total** (or 9 if Cycle 7 splits).
5. **Tag numbering:** Cycles 1-3 = v0.28.5/6/7 (PATCH); Cycle 4 = v0.29.0 (SemVer-minor; xpub-search struct→enum wire-shape break); Cycles 5-8 = v0.29.1/2/3/4 (PATCH unless wire-shape break surfaces mid-cycle).

## FOLLOWUP-to-cycle mapping (15 slugs → 8 cycles)

| Cycle | Tag | Slugs (count) |
|---|---|---|
| 1 | v0.28.5 (docs) | `plan-smoke-step4-ms1-on-bundle-not-supported` + `import-wallet-envelope-schema-version-narrative-drift` (2) |
| 2 | v0.28.6 (test-hygiene) | `cross-format-refusal-matrix-include-coldcard-multisig` + `coldcard-legacy-mk1-mk2-top-level-xpub-inference` (fixture+tests only) (2) |
| 3 | v0.28.7 (hardening) | `bsms-import-taproot-refusal-parity` + `green-emitter-multisig-refusal-template-only` + `wallet-import-format-mismatch-matrix-completion` + `wallet-import-taproot-internal-key` (4) |
| 4 | v0.29.0 (refactor + GUI paired) | `error-rs-retroactive-alphabetical-sort` + `pr-26-import-provenance-three-variant-cleanup` + `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` (3) |
| 5 | v0.29.1 (jade-seedqr) | `wallet-import-jade-seedqr` (1) |
| 6 | v0.29.2 (electrum-encrypted) | `wallet-import-electrum-encrypted` (1) |
| 7 | v0.29.3 (bsms-encryption envelope) | `bsms-bip129-encryption-envelope` (1) |
| 8 | v0.29.4 (sparrow-taproot) | `sparrow-taproot-descriptor-passthrough-import-support` (1; promoted from v0.29+ tier) |

**Closes all 15 v0.28+ open FOLLOWUPs.**

## Cycle inventory

| # | Cycle name | Tag(s) | Effort | Reviewer | GUI lockstep |
|---|---|---|---|---|---|
| 1 | v0.28.5 (docs) | toolkit patch | ~2 h | sonnet | No |
| 2 | v0.28.6 (test-hygiene) | toolkit patch | ~half-day | sonnet | No |
| 3 | v0.28.7 (hardening) | toolkit patch | ~3-5 days | opus | No |
| 4 | v0.29.0 + paired GUI | toolkit MINOR + GUI minor (paired) | ~2-3 days | opus | **Yes** (schema-mirror; xpub-search wire-shape break) |
| 5 | v0.29.1 (jade-seedqr) | toolkit patch + paired GUI | multi-week | opus | **Yes** (new `--format` value) |
| 6 | v0.29.2 (electrum-encrypted) | toolkit patch + paired GUI | multi-week | opus | **Yes** (new `--decrypt-password` flag) |
| 7 | v0.29.3 (bsms-encryption envelope) | toolkit patch + paired GUI | **4-6 weeks; possibly split 7a + 7b** | opus | **Yes** (new `--bsms-encryption-token` flag) |
| 8 | v0.29.4 (sparrow-taproot) | toolkit patch + (paired GUI conditional) | multi-week | opus | Confirm at cycle-start (depends on flag shape) |

**8 cycles (or 9 if Cycle 7 splits into 7a + 7b), ~8-9 toolkit tags + 5-6 GUI tags (paired).**

## Sequencing waves

```
Wave 1 (parallel-safe; ~1 day total):
  ├── Cycle 1 (docs)    ──── ships first
  └── Cycle 2 (tests)   ──── ships second (or parallel)

Wave 2 (~3-5 days):
  └── Cycle 3 (hardening) ── may add 1-2 new ToolkitError variants

Wave 3 (~2-3 days):
  └── Cycle 4 (refactor + v0.29.0 + GUI paired) ── variant-freeze gate

Wave 4-7 (multi-week each; STRICTLY sequential):
  ├── Cycle 5 (jade-seedqr)
  ├── Cycle 6 (electrum-encrypted)
  ├── Cycle 7 (bsms-encryption envelope) ── largest; 4-6 weeks
  └── Cycle 8 (sparrow-taproot)
```

**Dependency model (verified):**

- Wave 1 (Cycles 1+2) ⊥ everything else (doc + test-hygiene; no inter-cycle deps).
- Wave 2 (Cycle 3) → Wave 3 (Cycle 4): **STRICT GATE** — Cycle 4 Phase 0 must re-grep `ToolkitError` variant count POST-Cycle-3 and freeze it before the alphabetical sort begins. Cycle 3 may add `ImportWalletTaprootRefused` + `BsmsTaprootImportRefused` variants (or reuse existing); count must settle.
- Wave 4+ (Cycles 5-8) ⊥ each other (separate parser surfaces); ⊥ Cycles 1-4 (no shared mutation surface).
- Cycle 4 ↔ `mnemonic-gui` paired (schema-mirror lockstep mandatory).
- Cycles 5/6/7 ↔ `mnemonic-gui` paired (new `--format` values + flags trigger schema-mirror invariant per CLAUDE.md).
- Cycle 8 ↔ GUI: confirm at cycle-start (depends on flag shape).

## Per-cycle phase decomposition

### Wave 1

#### Cycle 1 — `mnemonic-toolkit-v0.28.5` (docs)

1. **Phase 0** — recon: confirm citation lines for both doc edits per recon dossier (already verified ACCURATE).
2. **Phase 1** — `plan-smoke-step4-ms1-on-bundle-not-supported`: edit `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §6.3 step 4 (L793); replace nonexistent `--ms1` flag with `--slot @0.phrase=` per `mnemonic bundle --help`.
3. **Phase 2** — `import-wallet-envelope-schema-version-narrative-drift`: doc-comment fix at `cmd/import_wallet.rs:87` (envelope schema_version "1") + `:975` (inner BundleJson schema_version "4"). Rename one to disambiguate OR add cross-reference comments — implementer decides at cycle-start.
4. **Phase 3** — sonnet reviewer + commit (incl. `scripts/install.sh:32` bump v0.28.4 → v0.28.5 + Cargo.toml + CHANGELOG entry) + tag + push + auto-create or manual GH Release.
5. **Phase 4** — FOLLOWUPS Status flips × 2.

#### Cycle 2 — `mnemonic-toolkit-v0.28.6` (test-hygiene)

1. **Phase 0** — recon.
2. **Phase 1** — `cross-format-refusal-matrix-include-coldcard-multisig`: extend `tests/cli_export_wallet_from_import_json.rs:592-593` `TEMPLATE_ONLY_DESTS` to include `"coldcard-multisig"`; broaden `REFUSAL_STDERR_PATTERNS:815` to match `"requires a multisig --template"` substring (intervening "a multisig" not in current pattern); bump cell-count assertion at L871 `32 → 40` (5 dests × 8 sources).
3. **Phase 2** — `coldcard-legacy-mk1-mk2-top-level-xpub-inference`: **PARSER ALREADY IMPLEMENTED** at commit `1304932` (v0.28.0 P3-v2 cycle); this cycle adds fixture + test cells only. File legacy fixture(s) in `tests/fixtures/wallet_import/` (e.g., `coldcard-mk1-legacy-bip44-mainnet.json` for `xpub` prefix, `-bip49-` for `ypub`, `-bip84-` for `zpub`); add ≥4 test cells in `tests/cli_import_wallet_coldcard.rs` (one per SLIP-132 prefix valid case: xpub/ypub/zpub + 1 refusal case for unrecognized prefix per `coldcard.rs:490-493` `Err` arm).
4. **Phase 3** — cargo test + clippy + `make audit` (14/14 transcripts + new test cells).
5. **Phase 4** — sonnet reviewer + commit (incl. install.sh:32 bump v0.28.5 → v0.28.6) + tag + push + GH Release.
6. **Phase 5** — FOLLOWUPS Status flips × 2.

### Wave 2

#### Cycle 3 — `mnemonic-toolkit-v0.28.7` (hardening)

1. **Phase 0 — STRICT P0 GATE: re-validation** (per architect M2 fold). Plan-doc body must NOT be written until BOTH scope-drift slugs are re-grepped:
   - `wallet-import-format-mismatch-matrix-completion`: re-count `ImportWalletFormatMismatch` per arm in `cmd/import_wallet.rs`; lock narrow-arm residuals (currently BSMS/BitcoinCore/ColdcardMultisig per recon Round 1).
   - `wallet-import-taproot-internal-key`: verify whether per-exporter scope is correct (per recon Round 2: per-exporter framing doesn't match code; issue is at `cmd/export_wallet.rs:650` envelope-gate only). If envelope-gate-only, drop per-exporter framing from plan-doc body before proceeding.
2. **Phase 1** — `bsms-import-taproot-refusal-parity`: add early `Tr(_)` short-circuit in `BsmsParser::parse` (around `bsms.rs:70`); refusal symmetric to emit-side `BsmsTaprootRefused` (either NEW ToolkitError variant `BsmsTaprootImportRefused` OR reuse-with-import-side-discriminator). Also broaden `extract_threshold` regex at `bsms.rs:479` to NOT match `sortedmulti_a(` (closes side-channel per recon).
3. **Phase 2** — `green-emitter-multisig-refusal-template-only`: refactor `wallet_export/green.rs:30-44` refusal guard from `inputs.template.is_some() && t.is_multisig()` → `inputs.script_type.is_multisig()` (covers descriptor-mode at `cmd/export_wallet.rs:638` where `template == None`).
4. **Phase 3** — `wallet-import-format-mismatch-matrix-completion`: extend the 3 narrow arms (BSMS / BitcoinCore / ColdcardMultisig) to refuse all other parser sniffs symmetrically. Possibly adds 1-2 `ImportWalletFormatMismatch` arms per format-arm.
5. **Phase 4** — `wallet-import-taproot-internal-key`: post-recon decision; if envelope-gate-only, narrow to single fix at `cmd/export_wallet.rs:650`; if per-exporter, fan out.
6. **Phase 5** — cargo test + clippy + new test cells per slug.
7. **Phase 6** — opus reviewer (4-slug bundle warrants opus per `feedback_opus_primary_review_agent`).
8. **Phase 7** — commit (incl. install.sh:32 bump v0.28.6 → v0.28.7) + tag + push + GH Release.
9. **Phase 8** — FOLLOWUPS Status flips × 4.

### Wave 3

#### Cycle 4 — `mnemonic-toolkit-v0.29.0` + paired `mnemonic-gui-v0.X` (refactor + SemVer-minor)

1. **Phase 0 — STRICT P0 GATE: variant freeze** (per architect I1 fold). Plan-doc body must NOT be written until ToolkitError variant count is locked. Re-grep `ToolkitError` enum count POST-Cycle-3; freeze the set. Sort cannot proceed until this gate passes.
2. **Phase 1 — cross-repo recon:** verify GUI baseline + capture v0.29.0 wire-shape impact on schema-mirror.
3. **Phase 2** — `pr-26-import-provenance-three-variant-cleanup`: refactor `ImportProvenance::Bsms(Option<BsmsAuditFields>)` → 3-variant form (e.g., `Bsms2Line` / `Bsms6LineFromCoordinator(BsmsAuditFields)` / `BsmsRoundtripDescriptorOnly` — exact shape decided at brainstorm-write for the cycle).
4. **Phase 3** — `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution`: convert `PathOfXpubResult` (`:144`), `PassphraseOfXpubResult` (`:169`), `AccountOfDescriptorResult` (`:155`) from structs to tagged enums. **SemVer-minor wire-shape break** (driving v0.29.0 cliff).
5. **Phase 4** — `error-rs-retroactive-alphabetical-sort`: reorder `enum ToolkitError` (43 + any Cycle-3-added variants) alphabetically; cascade reorder across `Display` / `exit_code` / `kind` match blocks. ~250+ arm rewrites. Single "sort-only, no semantic change" commit (or stage in same Cycle-4 commit if sonnet review confirms zero-semantic-drift).
6. **Phase 5** — cargo test + clippy + full test suite.
7. **Phase 6** — `mnemonic gui-schema` regen + verify new JSON output reflects xpub-search wire shape change.
8. **Phase 7** — GUI lockstep at `mnemonic-gui/src/schema/mnemonic.rs`: update xpub-search result struct mirror; bump GUI pin v0.28.4 → v0.29.0 in `Cargo.toml` + `pinned-upstream.toml`; GUI minor bump v0.13.0 → v0.14.0 (wire-shape break upstream).
9. **Phase 8** — opus reviewer (cross-repo; v0.29.0 SemVer-minor + GUI schema-mirror).
10. **Phase 9** — paired commits + tags: toolkit (install.sh bump v0.28.7 → v0.29.0 + CHANGELOG) + GUI tag `mnemonic-gui-v0.14.0`.
11. **Phase 10** — closure-verification: GUI CI schema_mirror gate GREEN against bumped pin (architect-I4 pattern from A/B/C Cycle 3).
12. **Phase 11** — FOLLOWUPS Status flips × 3.

### Wave 4-7 — individual parser cycles (each multi-week)

Plan-docs **DEFERRED** to per-cycle writing-plans dispatch when each is triggered (per `feedback-followups-md-line-numbers-presumed-stale` discipline). High-level shape per cycle:

#### Cycle 5 — `mnemonic-toolkit-v0.29.1` (jade-seedqr)

- **Slug:** `wallet-import-jade-seedqr`.
- **Shape:** add SeedQR parser to `wallet_import/jade.rs`; new fixture; chapter-45 prose; test cells.
- **CLI surface:** add `--format jade-seedqr` value (or fold into auto-detect under existing `--format jade`).
- **GUI lockstep:** mandatory if new `--format` value added.
- **Effort:** multi-week.
- **SemVer:** PATCH (additive enum value).

#### Cycle 6 — `mnemonic-toolkit-v0.29.2` (electrum-encrypted)

- **Slug:** `wallet-import-electrum-encrypted`.
- **Shape:** add encrypted-wallet parse path to `wallet_import/electrum.rs`; PBKDF2-derived AES decrypt; chapter-45 prose; test cells.
- **CLI surface:** add `--decrypt-password <FLAG>` (file/stdin) for argv attack-surface review.
- **GUI lockstep:** mandatory (new flag + password-input UX).
- **Effort:** multi-week.
- **SemVer:** PATCH per CLI-flag additive rule; password-on-argv is a MINOR per architect I3 policy IF passed inline (use file/stdin to keep PATCH).

#### Cycle 7 — `mnemonic-toolkit-v0.29.3` (bsms-encryption envelope) — LARGEST

- **Slug:** `bsms-bip129-encryption-envelope`.
- **Shape:** add STANDARD/EXTENDED encryption envelope handling to `wallet_import/bsms.rs` (PBKDF2-SHA512 c=2048 + AES-256-CTR + HMAC-SHA256). 3 new crypto primitives. Cross-impl smoke vs Coinkite Python ref.
- **CLI surface:** add `--bsms-encryption-token <FILE|->` flag.
- **GUI lockstep:** mandatory.
- **Effort:** **4-6 weeks; possibly split 7a + 7b** (per architect I4 fold):
  - **Cycle 7a:** primitives + unit smokes (pure crypto wiring; PBKDF2 + AES-CTR + HMAC; standalone testable; no CLI yet).
  - **Cycle 7b:** CLI wire-up + cross-impl smoke + MAC-then-decrypt ordering audit.
  - Split-vs-monolith decision deferred to Cycle 7's own brainstorm-write.
- **SemVer:** PATCH (additive); MAC-then-decrypt ordering bug if found = potential SemVer-MAJOR (wire-format-incompatible) — unlikely but watch for it.

#### Cycle 8 — `mnemonic-toolkit-v0.29.4` (sparrow-taproot)

- **Slug:** `sparrow-taproot-descriptor-passthrough-import-support` (promoted from v0.29+ tier).
- **Shape:** add taproot descriptor-passthrough import path to `wallet_import/sparrow.rs`; parallel parse path that detects descriptor-passthrough shape via heuristic.
- **CLI surface:** **confirm at cycle-start** whether new `--format sparrow-taproot` value OR `--format sparrow` auto-detect.
- **GUI lockstep:** conditional on CLI surface decision above.
- **Effort:** multi-week.
- **SemVer:** PATCH.

## Cross-cutting concerns

### SemVer policy (locked per architect I3)

- **PATCH** (`v0.X.Y` → `v0.X.Y+1`): additive enum values; new CLI flag additions; bug fixes; defensive type guards (newtype wrappers); fixture/test additions.
- **MINOR** (`v0.X.Y` → `v0.X+1.0`): wire-shape replacement (struct → tagged enum, e.g., xpub-search); JSON field renames; CLI surface removal/rename; encryption-key material passed inline on argv (attack-surface change).
- **MAJOR** (`v0.X.Y` → `v0.X+10.0` per `0.x` convention): wire-format-incompatible serialization (e.g., bundle format break); semver-broken API change.

Cycles 5/6/7/8 stay PATCH unless they introduce wire-shape replacement mid-cycle. Cycle 4 is the only locked MINOR cliff in this plan.

### `install-pin-check.yml` self-pin discipline

Per architect I5 from A/B/C brainstorm + v0.18.1 precedent: every toolkit-tag commit MUST also bump `scripts/install.sh:32` to the new tag value. Applies to ALL 8 cycles in this plan (Cycles 1-8 all carry toolkit tags). The `install-pin-check.yml` workflow gate validates on tag push.

### GUI schema-mirror lockstep

Per CLAUDE.md "GUI schema-mirror coverage" + architect I4 lagging-indicator gate:

- **Mandatory paired tag:** Cycles 4, 5, 6, 7.
- **Conditional paired tag:** Cycle 8 (decide at cycle-start).
- **No GUI work:** Cycles 1, 2, 3 (no CLI surface or wire-shape changes).

For paired cycles, the discipline is:
1. Toolkit tag lands first (by ~minutes).
2. GUI bumps `pinned-upstream.toml` toolkit pin + `Cargo.toml` toolkit dep version.
3. GUI tag lands second.
4. **Closure-verification:** confirm GUI CI's `schema_mirror`-gate GREEN against the new pin before declaring cycle closed. (Lagging-indicator gate; without this, drift accumulates silently — v0.27.2 historical case study.)

### crates.io publish blocker

Unchanged from A/B/C: still blocked on miniscript `[patch.crates-io]` per memory `project_v0_24_0_cycle_shipped`. All cycles ship via git+tag only; `install.sh` continues to handle install-path.

### Branch convention

Master-direct (per A/B/C precedent).

### Reviewer mode per cycle

- **Sonnet** (small/mechanical): Cycles 1-2.
- **Opus** (judgment-heavy or cross-repo): Cycles 3-8.

### FOLLOWUP closure cadence

Each cycle's final commit message references closed FOLLOWUP slugs + commit-SHA pin. Status flips happen via sed-then-amend or two-commit pattern (A/B/C lesson: SHA-self-reference under amend produces orphaned-but-traceable refs; tag is the durable anchor).

### Wave 4+ plan-docs DEFERRED

Cycles 5-8 plan-docs NOT written in this brainstorm session per `feedback-followups-md-line-numbers-presumed-stale`. Each gets its own writing-plans dispatch when triggered. This brainstorm doc is the SPEC; per-cycle plan-docs come later.

## Architect review folds applied (this brainstorm session)

Opus architect dispatched on Section 1 + Section 2 draft returned **YELLOW** with 4 Important + 2 Minor findings; all folded inline:

| # | Finding | Fold location |
|---|---|---|
| I1 | Cycle 3 → Cycle 4 dependency under-specified | Cycle 4 Phase 0 STRICT GATE: variant-freeze before sort |
| I2 | Cycles 5-8 likely need GUI lockstep | GUI lockstep matrix updated: Cycles 4/5/6/7 mandatory; Cycle 8 conditional |
| I3 | SemVer policy at v0.29.x cliff unspecified | Explicit PATCH/MINOR/MAJOR policy locked in Cross-cutting concerns |
| I4 | Cycle 7 (BSMS encryption) effort larger than "multi-week" | Reframe: 4-6 weeks; possible split into 7a (primitives) + 7b (CLI wire-up) |
| M1 | Cycle 2 framing | Already accurate ("fixture+tests only"); design doc reflects this |
| M2 | Cycle 3 P0 re-validation as HARD gate | Cycle 3 Phase 0 STRICT GATE: re-validate 2 scope-drift slugs before plan-doc body |

Architect also confirmed GREEN aspects (no change): 9-cycle hybrid correct; v0.28.5/6/7 PATCH bumps correct; xpub-search wire-shape break does NOT impact `mnemonic-gui` downstream (architect grep'd `mnemonic-gui/src/` for `PathOfXpubResult` consumers — zero hits; only schema-mirror surface matters, which Cycle 4 already lockstep); Cycle 4 = v0.29.0 SemVer-minor correct.

## Recon dossier reference

`cycle-prep-recon-followups-v0_28_plus.md` at branch root carries per-slug citation verification across both recon rounds. Aggregate at recon time:

- 8 ACCURATE (ready-to-brainstorm).
- 3 line-drift only (mechanical body amendments at brainstorm-write per `feedback-followups-md-line-numbers-presumed-stale`).
- 2 body-amended in-session (`coldcard-legacy-mk1-mk2`, `wallet-import-format-mismatch-matrix-completion`).
- 2 scope-drift unresolved (Cycle 3 P0 re-validation locks).

## Next phase

Per `brainstorming` skill flow, the next step is invoking `writing-plans` skill (one invocation per cycle, deferred to per-cycle triggers per stale-citation discipline). Wave 1 plan-docs (Cycles 1 + 2) are the natural first writing-plans dispatch — both small, both can ship near-term.

**Default lean: write Wave 1 plan-docs (Cycles 1 + 2) now; defer Waves 2-7 plan-docs to their respective cycle-start time.** Mirrors A/B/C Wave-1-first approach.

---

**Brainstorm session ends here.** This document is the SPEC + sequencing lock for the next 8-9 cycles (8 if Cycle 7 monolithic; 9 if split into 7a + 7b) closing the 15 v0.28+ residual FOLLOWUPs. Each cycle's detailed implementation plan is produced by a separate `writing-plans` skill invocation, citing this doc + the recon dossier as authoritative inputs.
