# BRAINSTORM — v0.27.2 patch cycle (FOLLOWUP cleanup)

**Author:** brainstorming session 2026-05-19 (Claude Opus 4.7 + user `bg`)
**Status:** spec draft — pending user review, then handoff to `writing-plans` skill
**Target tags:**
- `mnemonic-toolkit-v0.27.2` (toolkit)
- `mnemonic-gui-v0.11.1` (sibling lockstep)
**Source branch (target — not yet branched):** `release/v0.27.2` off `origin/master`. Pre-condition: PR #29 (Cargo.lock + scratch-gitignore) must merge to master first; current local state is on `release/v0.27.1` with PR #29 open.

## Cycle thesis

A focused cleanup-character patch cycle that lands the 7 FOLLOWUP items most aligned with "no wire-shape change / no SPEC change / patch-tier" framing. Anchored on Phase 5b's deferred `ImportProvenance` enum refactor (originally tier `v0.28+`, promoted to `v0.27.2` per user direction). Bundles cleanup items of doc, test, semantic-clarification, and CI-hygiene character with one sibling-repo workflow fix that lands in lockstep as `mnemonic-gui-v0.11.1`.

This is the first half of a two-tier framing the user approved:
- **Phase 1 (this cycle):** v0.27.2 patch — internal-only cleanup + Phase 5b refactor
- **Phase 2 (next cycle, v0.28.0 minor):** big v0.27-tier features — 6 wallet-import format ingests, passphrase-bruteforce, compare-cost tr() support, BIP-129 cutover

This spec covers Phase 1 only. Phase 2 gets its own brainstorm cycle when ready.

## §1 — Cycle target + ship strategy

- **Target tag:** `mnemonic-toolkit-v0.27.2` (patch bump per SemVer). No wire-shape change; no SPEC change; no `### Added` in CHANGELOG. Entries under `### Fixed`, `### Changed`, `### Tests`.
- **Sibling lockstep:** parallel `mnemonic-gui-v0.11.1` patch that (a) extends workflow trigger filter to include release branches, (b) bumps toolkit pin v0.26.0 → v0.27.2 (closes silent-drift gap), (c) adds smoke surface verifying v0.27.0/v0.27.1 envelope shape changes don't break GUI consumers.
- **Ship sequencing rule:** toolkit-first, GUI-second (per `design/PLAN_v0_26_0_three_way_merge.md` precedent). Toolkit v0.27.2 tag + release lands first; mnemonic-gui v0.11.1 then bumps its pin to that toolkit tag and ships.
- **Source branch:** `release/v0.27.2` off `origin/master` after PR #29 (Cargo.lock + scratch-gitignore) merges.
- **Tier promotion:** item 1 (`pr-26-import-provenance-enum-internal-refactor`) is promoted from FOLLOWUP-tier `v0.28+` to `v0.27.2` per the user's Shape A approval. Phase 4 task list applies the `Tier:` line edit in `design/FOLLOWUPS.md`.

## §2 — Per-item plan

Source citations below were verified by opus architect review against `origin/master` SHA `2f8b311`. Phase 0 recon re-verifies them after PR #29 advances master.

### Item 1 — `pr-26-import-provenance-enum-internal-refactor`

- **What:** Introduce `ImportProvenance { Bsms(BsmsAuditFields), BitcoinCore(CoreSourceMetadata) }` enum. Replace `ParsedImport`'s `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)` representable-invalid pair with a single `provenance: ImportProvenance` field. Internal-only refactor; wire shape unchanged (envelope-side `bsms_audit` / `source_metadata` JSON fields stay flat siblings).
- **Approach:** Option (b) from the FOLLOWUP — add back-compat accessors `bsms_audit() -> Option<&BsmsAuditFields>` and `source_metadata() -> Option<&CoreSourceMetadata>`. Call-sites mechanically updated under three patterns:
  - `&p.bsms_audit` → `p.bsms_audit()` (drop `&` prefix; 2 sites at `import_wallet.rs:587,599`)
  - `&b.source_metadata` → `b.source_metadata()` (drop `&` prefix; 1 site at `import_wallet.rs:825`)
  - `b.bsms_audit.is_some()` → `b.bsms_audit().is_some()` (owned-field `.is_some()` pattern; 2 sites at `import_wallet.rs:806, 818`)
  Plus `apply_select_descriptor` raw-field access at `wallet_import/mod.rs:150,167` → `p.source_metadata()` (`.as_ref()` chain replacement).
- **Where (origin/master ground truth, grep-verified against tip `2f8b311`):**
  - `crates/mnemonic-toolkit/src/wallet_import/mod.rs:60-80` — `ParsedImport` struct definition
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:266` — `Bsms` variant construction site (`Ok(vec![ParsedImport { ... bsms_audit: audit, source_metadata: None }])`)
  - `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:291-307` — `BitcoinCore` variant construction site (let-binding at 291-297; struct expr at 299-307)
  - `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:{587, 599, 806, 818, 825}` — envelope-emit access sites (5 sites total; 2 are `.is_some()` calls per recipe above)
  - `crates/mnemonic-toolkit/src/wallet_import/mod.rs:{150, 167}` — `apply_select_descriptor` access sites (2 sites)
- **Sized:** ~80-110 LOC (enum + 2 accessors + ~7 mechanical access-site edits + 4-6 cells)
- **Test coverage:** new unit cells in `wallet_import_unit` asserting (a) construction yields the variant matching the source (BSMS or Bitcoin Core); (b) accessors return `Some(&_)` for the matching variant and `None` for the other; (c) round-trip integration cell preserving envelope wire shape against `tests/cli_import_wallet_envelope_v0_27_0.rs` drift fixtures.

### Item 2 — `error-rs-canonical-ordering-doc`

- **What:** Adopt alphabetical-by-variant-name as the canonical ordering rule for `enum ToolkitError` variant declarations + every `match self { ... }` block that exhaustively matches it (`Display`, `exit_code`, `kind`). Codify in `CLAUDE.md` Conventions section so future cycles converge on the same ordering without per-PR negotiation.
- **Where:** `CLAUDE.md` Conventions section (~5-line addendum)
- **Soft precondition:** if Phase 5b (item 1) introduces a new `ToolkitError` variant, item 2's rule applies — item 2 should land first OR Phase 2 should preemptively follow the alphabetical rule when adding the variant. Phase 0 recon confirms whether item 1 adds a variant.
- **Sized:** ~15 LOC doc
- **Test coverage:** none (convention documentation)

### Item 3 — `compare-cost-agent-reports-back-fill`

- **What:** Codify "per-phase architect reviews persist verbatim to `design/agent-reports/` BEFORE fold-and-commit" as a load-bearing cycle discipline in `CLAUDE.md` Conventions. The compare-cost cycle's reviews were transcript-only; the back-fill meta-record at `design/agent-reports/compare-cost-cycle-meta.md` survives but verbatim review text is unrecoverable. Forward-looking codification only — accept the prior loss and prevent recurrence.
- **Where:** `CLAUDE.md` Conventions section addendum (~3-line)
- **Sized:** ~10 LOC doc
- **Test coverage:** none

### Item 4 — `mlock-g1-1-test-page-alignment-luck`

- **What:** Fix `g1_1_single_page_pin_has_page_count_one` flake under parallel test execution (`cargo test`'s default thread pool). The current test uses `Box<[u8; 64]>` which the heap allocator may straddle across a page boundary depending on bump-pointer state. Force page-aligned allocation: use `std::alloc::alloc` with `Layout::from_size_align(64, *PAGE_SIZE).unwrap()` so the buffer is invariant.
- **Where:** `crates/mnemonic-toolkit/tests/mlock_unit.rs:28` (assertion site); reference `crates/mnemonic-toolkit/src/mlock.rs::pin_pages_for` for page-count derivation.
- **Sized:** ~30 LOC test
- **Test coverage:** modify existing cell to use aligned alloc; optionally add a paired cell `g1_1_single_page_pin_unaligned_alloc_may_span_two_pages` asserting `>= 1 && <= 2` for the unaligned-buffer case (documents the brittleness).

### Item 5 — `gui-schema-arm-drop-detector`

- **What:** Formalize the manual `grep -c '=> .*_conditional_rules()'` rebase-check as a `#[test]` that asserts the live dispatcher arm count against a pinned constant. Three-way merge of `build_subcommand_conditional_rules` across concurrent feature PRs is silently-dropping-risky; bumping the constant becomes the explicit signal whenever a new arm is added.
- **Where:** Add a new `#[test]` cell to `crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs` (related context lives in this file already; new-file split unwarranted at +1 cell). Assert against `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::build_subcommand_conditional_rules` via file-read + regex count.
- **Pinned constant:** `EXPECTED_ARM_COUNT = 6` — verified live: `bundle / verify-bundle / export-wallet / convert / derive-child / compare-cost` (gui_schema.rs:338-343). Phase 0 recon re-verifies the count if the file has changed between origin and `release/v0.27.2`.
- **Sized:** ~30 LOC test
- **Test coverage:** 1 new cell

### Item 6 — `xpub-search-address-of-xpub-searched-count-semantic`

- **What:** **Doc clarification only** per user direction. Keep the current code (`searched = num_targets × gap_limit × chains`); clarify the semantic in two places. The per-target `scanned_external` / `scanned_internal` JSON fields already report unique candidates per-target; `searched` (an aggregate on `ToolkitError::XpubSearchNoMatch`) reports total candidate-comparisons performed.
- **Where (grep-verified):**
  - `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:288-293` — add inline comment at the `total_scanned` computation explaining the `n_targets × gap_limit × chains` formula
  - `crates/mnemonic-toolkit/src/error.rs:230-237` — extend the existing `XpubSearchNoMatch` enum-variant docstring (NOT a serde-derived struct; the variant has no JSON envelope of its own). Current docstring at lines 236-237 says "addresses × chains × gap-limit for address mode" but elides the `× n_targets` factor in the actual formula — restore the missing term + add the "candidate-comparisons performed" framing
- **Sized:** ~10 LOC doc only
- **Test coverage:** zero new cells; zero drift-cell updates (no behavior change, no value change). Existing cells continue to pass.

### Item 7 — `gui-workflow-trigger-include-release-branches` (sibling lockstep)

- **What:** mnemonic-gui v0.11.1 patch that lands three concerns in one PR:
  1. Extend `pull_request: branches:` filter from `[master]` to `[master, "release/**"]` so per-PR CI fires on integration branches (the headline FOLLOWUP fix)
  2. Bump `mnemonic-toolkit` dep pin from v0.26.0 → v0.27.2 in `Cargo.toml` + `pinned-upstream.toml` (closes silent toolkit-drift gap; per user direction on M3)
  3. Add a GUI smoke surface verifying v0.27.0/v0.27.1 envelope shape changes don't break consumers (import-wallet --json envelope wire shape replacement, xpub-search result types, BSMS Round-1 surfaces if GUI exposes them)
- **Where:**
  - `mnemonic-gui/.github/workflows/build.yml` + `mnemonic-gui/.github/workflows/schema-mirror.yml` (4 LOC YAML)
  - `mnemonic-gui/Cargo.toml` + `mnemonic-gui/pinned-upstream.toml` (toolkit pin sites)
  - GUI test source: TBD by Phase 0 (depends on which consumer surfaces exist for the new toolkit features)
- **Sized:** ~15 LOC YAML+TOML + 3-6 GUI smoke cells
- **Test coverage:** 3-6 new GUI smoke cells exercising envelope shape stability across v0.26.0 → v0.27.2 toolkit pin upgrade

## §3 — Phase structure

### Phase 0 — Recon

**Toolkit-side citation refresh:**
- Re-grep `cmd/import_wallet.rs` for `\.(bsms_audit|source_metadata)([^_a-zA-Z]|$)` against current `release/v0.27.2` tip. Lock the 5 access sites + 3 mechanical-edit patterns in §2 item 1 (current Round-2-verified values: lines 587/599/806/818/825 at master tip `2f8b311`; will drift after PR #29 advances master)
- Re-grep `wallet_import/mod.rs` for the same pattern; lock `apply_select_descriptor` sites (currently lines 150/167)
- Re-grep `wallet_import/bsms.rs` for `Ok(vec!\[ParsedImport \{` (currently line 266) and `wallet_import/bitcoin_core.rs` for `Ok(ParsedImport \{` (currently line 299)
- Run `grep -cE '^[[:space:]]+"[a-z-]+" => [a-z_]+_conditional_rules\(\),$' crates/mnemonic-toolkit/src/cmd/gui_schema.rs` to lock item 5's arm-count constant (currently **6**; tightened from the looser `'=> .*_conditional_rules()'` per architect R2 I3 to be reformatter-robust)

**Cross-phase dependency check:**
- Verify whether Phase 5b (item 1) adds any new `ToolkitError` variant; if yes, item 2's ordering rule applies and item 2 should land before Phase 2
- Enumerate drift cells that would be touched by item 6 (anticipated: **zero**, since item 6 is doc-only)

**Sibling-repo (mnemonic-gui) recon for Phase 3 sizing:**
- Read `mnemonic-gui/Cargo.toml` + `mnemonic-gui/pinned-upstream.toml` to confirm current toolkit pin (assumed v0.26.0; verify)
- `grep -r "schema_version" mnemonic-gui/src/` — locate GUI envelope consumers
- `grep -r "import-wallet\|xpub-search\|bsms_round1\|bsms-round1" mnemonic-gui/src/` — locate GUI surfaces exposed for v0.27.x toolkit features
- Verify mnemonic-gui's `pinned-upstream.toml` workflow auto-track mechanism for the v0.26.0 → v0.27.2 toolkit-pin bump (per memory `[[project-v0-5-1-schema-mirror-v2-closed]]`: workflow auto-tracks via Python `tomllib` parse-pre); identify whether `schema_mirror.yml` or any drift gate needs hand-update
- Confirm GUI v0.11.0 tip baseline + check for any in-flight GUI changes that conflict with v0.11.1 scope
- Size Phase 3's smoke cell budget from the surfaces enumerated above (estimated 3-6 cells; finalize at Phase 0 close)

### Phase 1 — Toolkit doc + test batch (items 2, 3, 4, 5, 6)

All items in this phase are doc-only or test-only with zero behavior change. Land as a single PR-ready batch — one commit per item (preferred for FOLLOWUP-status-flip auditability) or one consolidated commit (acceptable if Phase 1 execution surfaces no fold-rewrites):
- Item 2: `CLAUDE.md` alphabetical-variant-ordering rule
- Item 3: `CLAUDE.md` per-phase architect-reviews-persist-verbatim convention
- Item 4: `tests/mlock_unit.rs:28` aligned alloc fix (+ optional paired test)
- Item 5: new test cell pinning gui-schema arm count to 6
- Item 6: inline doc clarification on `searched` field semantic

Stage explicitly: `git add` each file. No `git add -A` per CLAUDE.md convention.

### Phase 2 — Phase 5b refactor (item 1)

- Introduce `ImportProvenance` enum at `wallet_import/mod.rs`
- Add accessors `bsms_audit() -> Option<&BsmsAuditFields>` + `source_metadata() -> Option<&CoreSourceMetadata>`
- Update construction sites: `wallet_import/bsms.rs:266` + `wallet_import/bitcoin_core.rs:291-307` (grep-verified against `origin/master` tip `2f8b311`)
- Mechanical syntax shift at access sites: **5** in `cmd/import_wallet.rs:{587, 599, 806, 818, 825}` + **2** in `apply_select_descriptor` at `wallet_import/mod.rs:{150, 167}` — 7 access sites total. Three mechanical-edit patterns per §2 item 1 (drop `&`, replace `.as_ref()` chain, owned-field `.is_some()`)
- Add 4-6 unit cells + 1-2 integration cells (drift-shape regression against `tests/cli_import_wallet_envelope_v0_27_0.rs`)

**Sequencing relative to Phase 1:** independent IF Phase 5b adds no new `ToolkitError` variant (Phase 0 confirms). If it does, Phase 2 follows Phase 1 (item 2's ordering rule applies to the new variant).

**Architect review of Phase 2 fold:** dispatch opus per project convention before commit-and-push.

### Phase 3 — Sibling lockstep (item 7)

- Branch mnemonic-gui off its master after toolkit v0.27.2 tag lands
- Apply workflow YAML changes
- Bump toolkit pin v0.26.0 → v0.27.2
- Add GUI smoke cells per Phase 0 enumeration
- mnemonic-gui CHANGELOG `### Changed` entry + CI green
- Tag `mnemonic-gui-v0.11.1` + GH release

**Sequencing:** AFTER toolkit v0.27.2 tag (toolkit-first per merge-plan convention). GUI consumes the toolkit tag in `pinned-upstream.toml`.

### Phase 4 — Cycle close (explicit hygiene checklist)

Per the memories `feedback-phase-6-cargo-lock-stage-with-version-bump`, `feedback-phase-6-install-sh-pin-bump-required`, `feedback-per-phase-agents-forget-followup-status-flip`:

1. `CHANGELOG.md` `[0.27.2]` entry under `### Fixed` (item 6 doc) / `### Changed` (item 1 internal refactor) / `### Tests` (items 4, 5) — explicit `git add CHANGELOG.md`
2. `design/FOLLOWUPS.md` Status flips for 7 entries (6 toolkit-side + 1 sibling); apply `Tier:` line edit for item 1 (`v0.28+` → resolved at v0.27.2). Cross-cite the v0.11.1 sibling close on item 7.
3. `crates/mnemonic-toolkit/Cargo.toml` version bump 0.27.1 → 0.27.2; verify workspace-root `Cargo.toml` if a version field is present
4. **`cargo build`** (not `cargo check`) to regenerate `Cargo.lock` with the new version
5. `git add Cargo.lock` explicitly; verify pre-commit `git diff --cached -- Cargo.lock` shows the version bump
6. `scripts/install.sh` self-pin bump `mnemonic-toolkit-v0.27.1` → `mnemonic-toolkit-v0.27.2`
7. Confirm "no CLI flag delta" → no `docs/manual/src/40-cli-reference/` chapter update needed (none of items 1-6 add or remove a CLI flag; sanity-check `make -C docs/manual lint` still green)
8. **PR-merge sequencing:** Open PR `release/v0.27.2 → master`; await CI green; squash-merge to master; **then** `git tag mnemonic-toolkit-v0.27.2` on the squash commit (matches v0.27.1 + v0.26.0 + v0.27.0 cycle precedents — tag points at master squash, not at the unsquashed cycle tip)
9. `git push origin mnemonic-toolkit-v0.27.2` + `gh release create` with CHANGELOG body
10. Verify `install-pin-check` CI gate fires green on tag-merge
11. THEN sequence GUI v0.11.1 (Phase 3 cycle close) — toolkit-first, GUI-second rule

## §4 — Test strategy + budget

- Item 1 (Phase 5b): 4-6 new toolkit cells
- Item 4 (mlock): 1 cell change + optional 1 paired test
- Item 5 (gui-schema arm count): 1 new toolkit cell
- Item 6 (xpub-search searched count): 0 cells (doc only)
- Items 2, 3 (doc): 0 cells
- Item 7 (GUI lockstep): 3-6 GUI smoke cells

**Total estimated:** ~10 toolkit cells + ~3-6 GUI smoke cells.

**Baseline:** toolkit 1576 cells (v0.27.1 tip). Post-cycle: 1586 ± 1.

## §5 — Risk surface

- **Low (items 2, 3, 4, 5, 6):** doc + test + zero-behavior items. Drift-shape regression guarded by both surfaces: integration test files `tests/cli_import_wallet_envelope_v0_27_0.rs` + `tests/cli_xpub_search_drift_v0_27_0.rs`, AND captured envelope fixtures at `tests/fixtures/v0_27_0_envelopes/` (6 JSON files: `path_of_xpub.{match,no_match}.json`, `account_of_descriptor.{match,no_match}.json`, `passphrase_of_xpub.{match,no_match}.json`) and `tests/fixtures/wallet_import/envelope_v0_27_0.json`.
- **Medium (item 1, Phase 5b):** ~7 access-site mechanical edits + 2 construction sites + accessor introduction. Wire-shape unchanged → the v0.27.0 envelope drift cells (`cli_import_wallet_envelope_v0_27_0.rs`) gate option-(b) accessor refactor against shape regressions.
- **Cross-repo low (item 7):** YAML edit + Cargo.toml pin bump + smoke cells. mnemonic-gui CI gates regressions on the integration PR (which targets master, so its workflow trigger fires correctly even before the trigger-filter fix lands).

## §6 — Ship sequence + dependencies

1. **Pre-condition:** PR #29 (Cargo.lock + scratch-gitignore) merges to master. Master tip advances.
2. Pull master locally; create `release/v0.27.2` off the new tip.
3. **Phase 0** recon → fold any line-number / arm-count corrections into the plan-doc inline.
4. **Phase 1** (toolkit doc + test batch) AND **Phase 2** (Phase 5b refactor) — parallel agent dispatches if Phase 0 confirms no Phase 5b new-variant dependency; sequenced (1 → 2) otherwise.
5. **Toolkit cycle close** — Phase 4 steps 1-10 (CHANGELOG → FOLLOWUPS → Cargo.toml → Cargo.lock → install.sh → manual-no-flag-delta check → PR-merge → tag at squash commit → push tag + GH release → install-pin-check CI verify).
6. **Phase 3** begins: branch mnemonic-gui off its master; fold workflow YAML + toolkit pin bump + smoke cells.
7. **GUI cycle close** — `mnemonic-gui-v0.11.1` tag + release.
8. Verify install-pin-check CI gate green on both tags.

## §7 — Out of scope (explicit)

- The 6 wallet-import format ingests (sparrow / specter / electrum / coldcard / coldcard-multisig / jade), `wallet-import-bsms-round-1`, `wallet-import-bsms-encrypted`, `bsms-bip129-full-cutover` — these are the Phase 2 v0.28.0-minor scope, brainstormed in a separate spec when ready.
- `xpub-search-passphrase-bruteforce` — feature work; v0.28.0 minor scope.
- `compare-cost-single-leaf-tr-input` — feature work; requires SPEC anchor first.
- `xpub-search-manual-gui-chapters` — docs work for the 4 xpub-search modes; could fold into a docs-only patch cycle separately.
- Older v0.27-nice-to-have items beyond items 5 + 6 (`xpub-search-descriptor-md1-detection-bech32-validate`) — held for Phase 2 or separate cycle.

## §8 — Related memories

- [[project-v0-27-1-cycle-shipped]] — predecessor cycle (PR-#26 fold)
- [[feedback-phase-6-cargo-lock-stage-with-version-bump]] — Phase 4 task list discipline
- [[feedback-phase-6-install-sh-pin-bump-required]] — Phase 4 task list discipline
- [[feedback-per-phase-agents-forget-followup-status-flip]] — Phase 4 task list discipline
- [[feedback-r0-must-read-source-off-by-n]] — Phase 0 recon discipline (architect review of this spec caught 1 Critical + 4 Important off-by-N findings; Phase 0 re-verifies)
- [[feedback-architect-agent-breaks-citation-drift-cycle]] — architect dispatched at design-time; Phase 2 fold gets a second architect pass
- [[feedback-opus-primary-review-agent]] — opus model used for both architect dispatches
- [[feedback-plan-artifact-mirror-project-convention]] — this spec is itself the SPEC + phased plan; reviewer-iterated before handoff to writing-plans
