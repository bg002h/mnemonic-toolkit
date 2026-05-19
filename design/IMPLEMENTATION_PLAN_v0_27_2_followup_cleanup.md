# v0.27.2 patch cycle — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `mnemonic-toolkit-v0.27.2` patch (7 cleanup items closing v0.27-tier FOLLOWUPs) + `mnemonic-gui-v0.11.1` sibling lockstep (workflow YAML + toolkit pin bump + envelope smoke cells), with zero wire-shape change.

**Architecture:** 4-phase sequential structure. Phase 0 = source-truth recon (refreshes line citations against `origin/master`). Phase 1 = toolkit doc + test batch (5 items, additive, zero behavior change). Phase 2 = `ImportProvenance` enum internal refactor (option-(b) accessors). Phase 3 = mnemonic-gui v0.11.1 sibling cycle. Phase 4 = toolkit cycle close with explicit Cargo.lock + install.sh + manual-delta hygiene checklist. Phases 1+2 parallel; 3 sequenced after toolkit tag; 4 closes the toolkit ceremony.

**Tech Stack:** Rust 1.85 (workspace), cargo, clap-derive, serde, jq + ripgrep for verification, gh CLI, GitHub Actions YAML.

**Spec reference:** `design/BRAINSTORM_v0_27_2_followup_cleanup.md` at branch `brainstorm/v0_27_2-cleanup-spec` commit `03c1dae`. Architect reviewer-loop converged at R4 GREEN.

---

## File Structure

### Created
- `crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs` — modify to add 1 new cell `dispatcher_arm_count_matches_pinned_constant` (item 5)
- `design/agent-reports/v0_27_2-phase-1-architect-review.md` — Phase 1 R0 fold log
- `design/agent-reports/v0_27_2-phase-2-architect-review.md` — Phase 2 R0 fold log
- `design/agent-reports/v0_27_2-end-of-cycle-architect-review.md` — Phase 4 holistic review

### Modified (toolkit)
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` — add `ImportProvenance` enum; replace `ParsedImport` field pair with single `provenance` field; add accessors `bsms_audit()` + `source_metadata()` (item 1)
- `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:266` — construction site (item 1)
- `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:291-307` — construction site (item 1)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — mechanical access-site syntax shift at 5 sites (item 1)
- `crates/mnemonic-toolkit/src/error.rs:230-237` — extend `XpubSearchNoMatch` docstring with `n_targets` factor (item 6)
- `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:288-293` — inline comment on `total_scanned` formula (item 6)
- `crates/mnemonic-toolkit/tests/mlock_unit.rs:28` — page-aligned allocation for `g1_1` test (item 4)
- `CLAUDE.md` — 2 new Conventions entries: alphabetical-variant-ordering rule (item 2); architect-reviews-persist-verbatim (item 3)
- `CHANGELOG.md` — `[0.27.2]` entry (Phase 4)
- `design/FOLLOWUPS.md` — 7 Status flips + 1 tier promotion (Phase 4)
- `crates/mnemonic-toolkit/Cargo.toml` — version 0.27.1 → 0.27.2 (Phase 4)
- `Cargo.lock` — regenerated post-bump (Phase 4)
- `scripts/install.sh` — `TAG` self-pin bump (Phase 4)

### Modified (mnemonic-gui — sibling repo `/scratch/code/shibboleth/mnemonic-gui/`)
- `.github/workflows/build.yml` — extend trigger filter (item 7)
- `.github/workflows/schema-mirror.yml` — extend trigger filter (item 7)
- `Cargo.toml` — toolkit dep pin v0.26.0 → v0.27.2 (item 7)
- `pinned-upstream.toml` — toolkit tag pin (item 7)
- `Cargo.lock` — regenerated post-pin-bump (item 7)
- `CHANGELOG.md` — v0.11.1 entry (item 7)
- `tests/cli_envelope_smoke.rs` — NEW; 3-6 cells verifying v0.27.x envelope shape (item 7)

---

## Phase 0 — Recon

**Goal:** Refresh all source-truth citations from the brainstorm spec against the current `origin/master` tip, AFTER PR #29 (Cargo.lock + scratch-gitignore) merges. Lock the verified line numbers + arm counts into Phase 1+2 task code before any agent dispatches.

**Branch setup:** `release/v0.27.2` off `origin/master` post-PR-#29.

### Task 0.1: Confirm PR #29 merged + branch off advanced master

**Files:** none (git operations)

- [ ] **Step 1: Verify PR #29 status (programmatic halt on not-merged)**

```bash
state=$(gh pr view 29 --json state -q '.state')
if [ "$state" != "MERGED" ]; then
  echo "PR #29 not merged (state=$state); halting Phase 0 — wait for merge."
  exit 1
fi
echo "PR #29 merged; proceeding."
```

Expected: `PR #29 merged; proceeding.` on stdout, exit 0. If the script exits 1, the agent must stop — do not proceed to Step 2.

- [ ] **Step 2: Fetch + verify origin/master tip advanced past 2f8b311**

```bash
git fetch origin master
git log origin/master --oneline -5
```

Expected: top entry is the PR #29 squash commit (chore: cargo-lock + scratch-gitignore). 2f8b311 is now ~3 commits back.

- [ ] **Step 3: Create release/v0.27.2 branch off origin/master**

```bash
git checkout -b release/v0.27.2 origin/master
git status -sb
```

Expected: `## release/v0.27.2...origin/master` with no diff.

- [ ] **Step 4: Commit**

No commit at this step; just branch creation.

### Task 0.2: Grep-verify item 1 (ImportProvenance) citations

**Files:** none (verification only); output to `design/agent-reports/v0_27_2-phase-0-recon.md`

- [ ] **Step 1: Re-grep import_wallet.rs access sites**

```bash
grep -nE '\.(bsms_audit|source_metadata)([^_a-zA-Z]|$)' crates/mnemonic-toolkit/src/cmd/import_wallet.rs
```

Expected: **7 lines total** — 5 actionable field-access sites at `{587, 599, 806, 818, 825}` + 2 string-literal false-positives at `{811, 823}` (the `writeln!(... "bundles[{i}].bsms_audit={audit_str}")` format strings — the regex matches `=` as the non-letter trailing context). The 5 actionable sites are what Phase 2 Task 2.6 mechanically edits. If the totals differ, update the recon-dossier values + Phase 2 Task 2.6 step code inline before agent dispatch.

- [ ] **Step 2: Re-grep wallet_import/mod.rs apply_select_descriptor sites**

```bash
grep -nE '\.(bsms_audit|source_metadata)([^_a-zA-Z]|$)' crates/mnemonic-toolkit/src/wallet_import/mod.rs
```

Expected: 2 lines. Spec cites `{150, 167}`.

- [ ] **Step 3: Re-grep ParsedImport construction sites**

```bash
grep -n 'ParsedImport {' crates/mnemonic-toolkit/src/wallet_import/bsms.rs
grep -n 'ParsedImport {' crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs
```

Expected: `bsms.rs:266` and `bitcoin_core.rs:299`. Bitcoin core full construction range is 291-307 (291 = let-binding open; 297 = let-binding close; 299 = Ok(ParsedImport open; 307 = struct close).

- [ ] **Step 4: Lock verified citations in recon dossier**

Create `design/agent-reports/v0_27_2-phase-0-recon.md`:

```markdown
# v0.27.2 Phase 0 recon dossier

**Date:** <YYYY-MM-DD>
**origin/master SHA:** <result of `git rev-parse origin/master`>
**release/v0.27.2 tip:** <result of `git rev-parse HEAD`>

## Item 1 — ImportProvenance refactor

- `cmd/import_wallet.rs` access sites: <verified list>
- `wallet_import/mod.rs` apply_select_descriptor sites: <verified list>
- `wallet_import/bsms.rs` ParsedImport construction: line <verified>
- `wallet_import/bitcoin_core.rs` ParsedImport construction: line <verified>

## Item 5 — gui-schema arm count

- `cmd/gui_schema.rs` `build_subcommand_conditional_rules` dispatcher arms: <verified count>

## Item 6 — drift cells (anticipated zero)

- `tests/cli_xpub_search_drift_v0_27_0.rs`: <grep result for `searched`>
- `tests/cli_import_wallet_envelope_v0_27_0.rs`: <grep result for `searched`>
```

- [ ] **Step 5: Commit recon dossier**

```bash
git add design/agent-reports/v0_27_2-phase-0-recon.md
git commit -m "docs(recon): v0.27.2 Phase 0 recon — citations grep-verified"
```

### Task 0.3: Grep-verify item 5 (gui-schema arm count)

**Files:** `design/agent-reports/v0_27_2-phase-0-recon.md` (append)

- [ ] **Step 1: Run the tightened regex**

```bash
grep -cE '^[[:space:]]+"[a-z-]+" => [a-z_]+_conditional_rules\(\),$' crates/mnemonic-toolkit/src/cmd/gui_schema.rs
```

Expected: `6` (spec ground truth). If different, the dispatcher has grown or shrunk — update Task 1.4's `EXPECTED_ARM_COUNT` constant accordingly.

- [ ] **Step 2: Append result to recon dossier (already done in Task 0.2.4 if the file was written together)**

### Task 0.4: Verify item 6 drift-cell impact (anticipated zero)

**Files:** `design/agent-reports/v0_27_2-phase-0-recon.md` (append)

- [ ] **Step 1: Grep drift cells for the bare `searched` token**

```bash
grep -nE '\bsearched\b[^_]' crates/mnemonic-toolkit/tests/cli_xpub_search_drift_v0_27_0.rs
grep -nE '\bsearched\b[^_]' crates/mnemonic-toolkit/tests/cli_import_wallet_envelope_v0_27_0.rs
```

Expected: ZERO matches for the bare `searched` token used by `ToolkitError::XpubSearchNoMatch`. Note: the test files DO contain `searched_count` and `searched_count_per_cosigner` references — those are unrelated per-target / per-cosigner aggregations and are correctly NOT touched by item 6 (which is about the aggregate `XpubSearchNoMatch.searched` field). The regex `\bsearched\b[^_]` excludes `_count` suffixes via word-boundary + non-underscore lookahead. If any match surfaces, item 6 escalates from doc-only to doc + 1-2 test updates.

### Task 0.5: Verify mnemonic-gui repo state for Phase 3 sizing

**Files:** `design/agent-reports/v0_27_2-phase-0-recon.md` (append)

- [ ] **Step 1: Confirm GUI repo path + current pin**

```bash
ls -la /scratch/code/shibboleth/mnemonic-gui/
grep 'mnemonic-toolkit' /scratch/code/shibboleth/mnemonic-gui/Cargo.toml
grep 'mnemonic-toolkit' /scratch/code/shibboleth/mnemonic-gui/pinned-upstream.toml 2>/dev/null || echo "(no pinned-upstream.toml)"
```

Expected: GUI pin currently at `v0.26.0` (per memory `project-v0-24-0-cycle-shipped`). Phase 3 bumps to `v0.27.2`.

- [ ] **Step 2: Enumerate GUI envelope consumers**

```bash
grep -r --include='*.rs' -l 'schema_version' /scratch/code/shibboleth/mnemonic-gui/src/
grep -r --include='*.rs' -l 'import-wallet\|xpub-search\|bsms_round1\|bsms-round1' /scratch/code/shibboleth/mnemonic-gui/src/
```

Capture the file list — Phase 3 GUI smoke cells exercise these consumers.

- [ ] **Step 3: Check pinned-upstream.toml workflow auto-track mechanism**

```bash
grep -A 3 'tomllib\|pinned-upstream' /scratch/code/shibboleth/mnemonic-gui/.github/workflows/schema-mirror.yml
```

Expected: Python tomllib parse-pre step that reads the toolkit tag. Confirms M3 ground truth that bumping `pinned-upstream.toml` cascades via the workflow.

- [ ] **Step 4: Append GUI recon results to dossier; commit if not committed in Task 0.2.5**

```bash
git add design/agent-reports/v0_27_2-phase-0-recon.md
git commit --amend --no-edit  # if combined with Task 0.2.5; else new commit
```

---

## Phase 1 — Toolkit doc + test batch (items 2, 3, 4, 5, 6)

**Goal:** Land items 2-6 as a single coherent batch. Each item lands as its own commit for FOLLOWUP-status-flip auditability. Zero behavior change; drift-shape regression guarded by existing fixtures.

> 📋 **Recon-dossier discipline:** All line-number citations in Phase 1 + Phase 2 tasks are reference values from spec-write time (`24978e4` → `03c1dae` lineage). Phase 0 produces `design/agent-reports/v0_27_2-phase-0-recon.md` with grep-verified live line numbers. **The dossier is authoritative — if dossier values differ from Phase 1/2 task text, the dossier wins.** Before dispatching a Phase 1 or Phase 2 agent, the operator updates the task text inline to match the dossier. This is the discipline codified in memory `feedback-grep-verify-during-fold-not-just-during-write`.

### Task 1.1: error-rs alphabetical-ordering Convention (item 2)

**Files:**
- Modify: `CLAUDE.md` (Conventions section)

- [ ] **Step 1: Read current Conventions section**

```bash
sed -n '/^## Conventions/,/^## /p' CLAUDE.md | head -20
```

Expected: confirm Conventions section bullet style matches the additions at brainstorm commit `24978e4` (initial spec) → `03c1dae` (R4 GREEN spec) lineage — citation grep-verify rule + reviewer-loop rule. Item 2's new bullet sits alongside these.

- [ ] **Step 2: Add the alphabetical-ordering Convention bullet**

Edit `CLAUDE.md` — under `## Conventions`, append after the existing "Reviewer-loop continues after every fold" bullet:

```markdown
- **`enum ToolkitError` variant declarations + every `match self { ... }` block that exhaustively matches it use alphabetical-by-variant-name ordering.** Drift across concurrent feature PRs (9+ new variants in v0.26.0 cycle) is otherwise a guaranteed merge-conflict generator; alphabetical order makes resolution mechanical. Apply to `error.rs::ToolkitError`, its `Display` impl, `exit_code`, `kind`, and any future exhaustive match blocks.
```

- [ ] **Step 3: Verify markdown lints clean**

```bash
make -C docs/manual lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=true 2>&1 | tail -10
```

Expected: PASS or no markdownlint errors related to CLAUDE.md. The manual lint may not cover CLAUDE.md — check `docs/manual/tests/lint.sh` scope.

- [ ] **Step 4: Commit**

```bash
git add CLAUDE.md
git commit -m "docs(conventions): add ToolkitError alphabetical-variant-ordering rule

Closes design/FOLLOWUPS.md::error-rs-canonical-ordering-doc"
```

### Task 1.2: persist-architect-reviews-verbatim Convention (item 3)

**Files:**
- Modify: `CLAUDE.md` (Conventions section)

- [ ] **Step 1: Add the architect-reviews-persist Convention bullet**

Edit `CLAUDE.md` — under `## Conventions`, append after the Task 1.1 bullet:

```markdown
- **Per-phase architect-review agent outputs persist verbatim to `design/agent-reports/<cycle>-phase-N-<round>-review.md` BEFORE the fold-and-commit step.** Transcript-only review text is unrecoverable from outside the session. Future cycles MUST persist the full review-agent output (Critical / Important / Minor sections + file/line citations) before applying folds, so the audit trail survives session boundaries. Compare-cost cycle (v0.26.0 C2-C5) reviews were lost via this gap.
```

- [ ] **Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs(conventions): persist architect-review outputs verbatim per phase

Closes design/FOLLOWUPS.md::compare-cost-agent-reports-back-fill"
```

### Task 1.3: mlock_unit g1_1 page-aligned fix (item 4)

**Files:**
- Modify: `crates/mnemonic-toolkit/tests/mlock_unit.rs:28` (and surrounding context)

- [ ] **Step 1: Read the current test**

```bash
sed -n '20,35p' crates/mnemonic-toolkit/tests/mlock_unit.rs
```

Expected: test `g1_1_single_page_pin_has_page_count_one` allocating `vec![0xAAu8; 64]` and asserting `pin.page_count == 1` via `mlock::pin_pages_for(&buf)`. The `Vec`'s heap-allocator bump pointer may straddle a page boundary depending on thread-local arena state — this is the flake source.

- [ ] **Step 2: Run the test under parallel execution to reproduce flake (informational)**

```bash
for i in 1 2 3 4 5; do cargo test --test mlock_unit g1_1_single_page_pin_has_page_count_one 2>&1 | tail -2; done
```

If 5/5 PASS, the heap state happens to be aligned today. The fix is invariant regardless.

- [ ] **Step 3: Rewrite the test using `std::alloc::alloc` with explicit page alignment**

Edit `crates/mnemonic-toolkit/tests/mlock_unit.rs` — replace the `g1_1_single_page_pin_has_page_count_one` test body with:

```rust
#[test]
fn g1_1_single_page_pin_has_page_count_one() {
    use std::alloc::{alloc, dealloc, Layout};

    let page_size = mlock::page_size_for_test();
    // SAFETY: Layout is valid (size > 0, align is power of 2). We deallocate
    // before returning. Buffer is page-aligned so pin_pages_for returns exactly 1.
    let layout = Layout::from_size_align(64, page_size).expect("valid layout");
    unsafe {
        let ptr = alloc(layout);
        assert!(!ptr.is_null(), "alloc failed");
        let slice = std::slice::from_raw_parts(ptr, 64);
        let pin = mlock::pin_pages_for(slice);
        assert_eq!(pin.page_count, 1, "page-aligned 64-byte buffer spans exactly 1 page");
        assert!(!pin.start.is_null(), "non-empty buf produces non-null start");
        drop(pin);
        dealloc(ptr, layout);
    }
}
```

Public API used (grep-verified at `src/mlock.rs`): `mlock::page_size_for_test()` (line 221) returns `usize`; `mlock::pin_pages_for(&[u8])` (line 90) returns `PinnedPageRange` with `.page_count: usize` and `.start: *const u8`. The `mlock::*` import is at the top of `mlock_unit.rs` already.

- [ ] **Step 4: Run the test 10× in parallel to verify invariance**

```bash
for i in $(seq 1 10); do cargo test --test mlock_unit g1_1_single_page_pin_has_page_count_one 2>&1 | grep 'test result'; done
```

Expected: 10/10 PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/tests/mlock_unit.rs
git commit -m "test(mlock): force page-aligned allocation in g1_1 to fix parallel-exec flake

Closes design/FOLLOWUPS.md::mlock-g1-1-test-page-alignment-luck"
```

### Task 1.4: gui-schema arm-count regression test (item 5)

**Files:**
- Modify: `crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs`

- [ ] **Step 1: Read current file structure**

```bash
head -30 crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs
tail -10 crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs
```

Note the existing test cells + imports. The new cell uses `std::fs::read_to_string` + regex.

- [ ] **Step 2: Write the failing test (RED)**

Add at the end of `crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs`:

```rust
/// Regression guard: ensures the `build_subcommand_conditional_rules` dispatcher
/// arm count remains pinned. Concurrent feature PRs that add a new subcommand
/// must consciously bump this constant — otherwise three-way merge can silently
/// drop an arm (no `cargo` error since the match is `_ => default`).
///
/// See `design/FOLLOWUPS.md::gui-schema-arm-drop-detector` for rationale.
#[test]
fn dispatcher_arm_count_matches_pinned_constant() {
    const EXPECTED_ARM_COUNT: usize = 6;
    let path = "src/cmd/gui_schema.rs";
    let body = std::fs::read_to_string(path).expect("read gui_schema.rs");
    let re = regex::Regex::new(r#"(?m)^\s+"[a-z-]+" => [a-z_]+_conditional_rules\(\),$"#).unwrap();
    let actual = re.find_iter(&body).count();
    assert_eq!(
        actual, EXPECTED_ARM_COUNT,
        "build_subcommand_conditional_rules arm count drift: \
         expected {EXPECTED_ARM_COUNT}, found {actual} arms in {path}. \
         If you added a new subcommand to the dispatcher, bump EXPECTED_ARM_COUNT \
         and verify no concurrent-PR rebase dropped an arm."
    );
}
```

Note: `regex` may need adding to `dev-dependencies` if not already present.

- [ ] **Step 3: Verify regex is already a workspace dep (no add needed)**

```bash
grep -nE '^regex' crates/mnemonic-toolkit/Cargo.toml
```

Expected: hit at `[dependencies]` section (currently line 28: `regex = "1"`). Integration tests have access to runtime `[dependencies]`, so no `[dev-dependencies]` add is required. If grep returns nothing (e.g., regex was removed in a later cycle), add `regex = "1"` to `[dev-dependencies]`.

- [ ] **Step 4: Run the test (expect PASS — regex matches the 6 arms)**

```bash
cargo test --test cli_gui_schema_conditional_rules dispatcher_arm_count_matches_pinned_constant
```

Expected: PASS. (This is a regression-test add, not TDD-RED-then-GREEN. The "failure" mode is post-commit if someone adds/drops an arm without bumping the constant.)

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/tests/cli_gui_schema_conditional_rules.rs
git add crates/mnemonic-toolkit/Cargo.toml  # only if regex was added
git commit -m "test(gui-schema): pin dispatcher arm count to detect silent merge drops

Closes design/FOLLOWUPS.md::gui-schema-arm-drop-detector"
```

### Task 1.5: xpub-search searched-count doc clarification (item 6)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/error.rs:230-237` — extend docstring
- Modify: `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:288-293` — inline comment

- [ ] **Step 1: Read existing error.rs docstring**

```bash
sed -n '225,245p' crates/mnemonic-toolkit/src/error.rs
```

Confirm the docstring at 230-237 reads: "`searched` is the candidate count exhausted (paths × accounts × cosigners for descriptor mode; addresses × chains × gap-limit for address mode)". The "× n_targets" factor is missing.

- [ ] **Step 2: Edit error.rs docstring**

Edit `crates/mnemonic-toolkit/src/error.rs` — extend the docstring at lines 230-237:

```rust
    /// v0.26.0 `mnemonic xpub-search` — no match found in the searched
    /// candidate set. Exit 4 (sibling to `BundleMismatch` /
    /// `Bip388VerifyDistinctness` — search-target mismatch class).
    /// `mode` distinguishes which xpub-search mode emitted (one of
    /// `"path-of-xpub"`, `"account-of-descriptor"`, `"address-of-xpub"`,
    /// `"passphrase-of-xpub"`); `searched` reports the count of
    /// **candidate-comparisons performed** (work done), not unique
    /// child-addresses derived. Formula:
    ///   - descriptor modes: `paths × accounts × cosigners`
    ///   - address mode: `n_targets × gap_limit × chains` (per-target
    ///     scan over the shared rendered-address Vec; one comparison per
    ///     (target, address) pair)
    /// The per-target JSON envelope fields `scanned_external` /
    /// `scanned_internal` (in `AddressOfXpubResult`) report unique
    /// child-addresses derived per-target (i.e., `gap_limit × chains`).
    XpubSearchNoMatch {
```

- [ ] **Step 3: Edit address_of_xpub.rs inline comment**

Edit `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:288-293` — replace the existing comment with:

```rust
    if all_matched {
        Ok(0)
    } else {
        // Aggregate count of candidate-comparisons performed for the no-match
        // diagnostic. Formula: matches.len() (= n_targets) × gap_limit × chains.
        // Per-target unique candidates are reported in AddressResultJson's
        // scanned_external / scanned_internal fields. See SPEC `searched` semantic
        // notes on ToolkitError::XpubSearchNoMatch.
        let total_scanned = matches
            .iter()
            .map(|_| (args.gap_limit * if scan_internal { 2 } else { 1 }) as usize)
            .sum::<usize>();
        Err(ToolkitError::XpubSearchNoMatch {
            mode: "address-of-xpub",
```

- [ ] **Step 4: Run existing tests (no behavior change expected)**

```bash
cargo test --test cli_xpub_search_drift_v0_27_0
cargo test --test cli_import_wallet_envelope_v0_27_0
cargo test --lib xpub_search
```

Expected: ALL PASS unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/error.rs crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs
git commit -m "docs(xpub-search): clarify searched-count candidate-comparisons semantic

Restores the missing 'n_targets ×' factor in the address-mode formula.
No behavior change — current value matches the canonical
'candidate-comparisons performed' read.

Closes design/FOLLOWUPS.md::xpub-search-address-of-xpub-searched-count-semantic"
```

### Task 1.6: Phase 1 architect review + persist verbatim

**Files:**
- Create: `design/agent-reports/v0_27_2-phase-1-architect-review.md`

- [ ] **Step 1: Dispatch opus architect review on Phase 1 commits**

Use `feature-dev:code-reviewer` (opus) agent. Brief:

```
Review the 5 Phase 1 commits on release/v0.27.2:
- Task 1.1: error-rs alphabetical-ordering Convention added to CLAUDE.md
- Task 1.2: persist-architect-reviews Convention added to CLAUDE.md
- Task 1.3: mlock_unit g1_1 page-aligned fix
- Task 1.4: gui-schema arm-count regression test (pinned at 6)
- Task 1.5: error.rs + address_of_xpub.rs docstring clarification

Verify each commit's diff against the spec at brainstorm commit 03c1dae (R4 GREEN).
Surface Critical / Important / Minor findings.
```

- [ ] **Step 2: Persist agent output verbatim**

Save the agent's full response to `design/agent-reports/v0_27_2-phase-1-architect-review.md` (per the new Task 1.2 Convention).

- [ ] **Step 3: Fold any Critical / Important findings**

If non-clean: fix each finding, then re-dispatch architect (per CLAUDE.md "Reviewer-loop continues after every fold" Convention). Repeat until 0 Critical / 0 Important.

If clean: proceed to Phase 2.

- [ ] **Step 4: Commit review log**

```bash
git add design/agent-reports/v0_27_2-phase-1-architect-review.md
git commit -m "docs(agent-reports): v0.27.2 Phase 1 architect review (GREEN)"
```

---

## Phase 2 — Phase 5b refactor (item 1)

**Goal:** Introduce `ImportProvenance` enum on `ParsedImport`; preserve wire shape via accessors. Option (b) mechanical syntax shift at access sites.

**Sequencing note:** If Phase 5b adds a new `ToolkitError` variant (e.g., `ImportProvenanceMismatch`), Task 1.1's alphabetical rule applies — sequence Phase 2 after Phase 1.1 (always true here since we land Phase 1 first).

### Task 2.1: Add ImportProvenance enum + accessor unit tests (RED)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/mod.rs`

- [ ] **Step 1: Read current ParsedImport struct definition**

```bash
sed -n '55,85p' crates/mnemonic-toolkit/src/wallet_import/mod.rs
```

Confirm the field pair `bsms_audit: Option<BsmsAuditFields>, source_metadata: Option<CoreSourceMetadata>`.

- [ ] **Step 2: Write failing unit tests for the new enum**

Append to `crates/mnemonic-toolkit/src/wallet_import/mod.rs` inside the existing `#[cfg(test)] mod tests` block (or create one if not present at the file root):

```rust
#[cfg(test)]
mod provenance_tests {
    use super::*;

    fn sample_bsms_audit() -> BsmsAuditFields {
        // Field shape grep-verified at src/wallet_import/mod.rs:188-202.
        // No Default impl on BsmsAuditFields or BsmsVerification; construct
        // with minimal valid values directly.
        BsmsAuditFields {
            token: String::new(),
            signature: String::new(),
            first_address: String::new(),
            derivation_path: String::new(),
            verification: BsmsVerification::NotAttempted,
        }
    }

    fn sample_core_metadata() -> CoreSourceMetadata {
        // Field shape grep-verified at src/wallet_import/mod.rs:88-100.
        // No Default impl; minimal valid construction.
        CoreSourceMetadata {
            active: false,
            internal: false,
            range: None,
            dropped_fields: Vec::new(),
            wallet_name: None,
        }
    }

    #[test]
    fn provenance_bsms_variant_yields_some_bsms_audit_and_none_source_metadata() {
        let p = ImportProvenance::Bsms(sample_bsms_audit());
        assert!(p.bsms_audit().is_some(), "Bsms variant exposes bsms_audit");
        assert!(p.source_metadata().is_none(), "Bsms variant does not expose source_metadata");
    }

    #[test]
    fn provenance_bitcoin_core_variant_yields_none_bsms_audit_and_some_source_metadata() {
        let p = ImportProvenance::BitcoinCore(sample_core_metadata());
        assert!(p.bsms_audit().is_none(), "BitcoinCore variant does not expose bsms_audit");
        assert!(p.source_metadata().is_some(), "BitcoinCore variant exposes source_metadata");
    }

    #[test]
    fn provenance_accessors_return_references_not_owned() {
        let p = ImportProvenance::Bsms(sample_bsms_audit());
        let _: Option<&BsmsAuditFields> = p.bsms_audit();
        let _: Option<&CoreSourceMetadata> = p.source_metadata();
    }
}
```

Note: `Default` impl may not exist on `BsmsAuditFields` / `CoreSourceMetadata`. If not, use real constructors — check `wallet_import/bsms.rs` and `bitcoin_core.rs` for sample factory fns.

- [ ] **Step 3: Run tests; expect failure (enum doesn't exist yet)**

```bash
cargo test --lib provenance_tests
```

Expected: compile error — `ImportProvenance` not defined.

### Task 2.2: Implement ImportProvenance enum + accessors (GREEN)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/mod.rs`

- [ ] **Step 1: Add the enum + ParsedImport field update**

Edit `crates/mnemonic-toolkit/src/wallet_import/mod.rs` — add above the `ParsedImport` struct:

```rust
/// Source-provenance of a parsed wallet import. Replaces the v0.26.0 / v0.27.x
/// representable-invalid `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)`
/// pair on `ParsedImport`. Exactly one variant per parse — enforced at the type
/// level.
///
/// See `design/FOLLOWUPS.md::pr-26-import-provenance-enum-internal-refactor`
/// for rationale (v0.27.1 Phase 5b deferral).
///
/// Visibility is `pub(crate)` to match the existing `ParsedImport` /
/// `BsmsAuditFields` / `CoreSourceMetadata` types (all `pub(crate)` per
/// grep-verified `src/wallet_import/mod.rs:60,88,188`). Bumping to `pub`
/// would require E0446-fixing all transitively-referenced types — out of
/// scope for this internal refactor.
#[derive(Debug, Clone)]
pub(crate) enum ImportProvenance {
    /// BSMS Round-2 parse (`wallet_import/bsms.rs`).
    Bsms(BsmsAuditFields),
    /// Bitcoin Core `listdescriptors` parse (`wallet_import/bitcoin_core.rs`).
    BitcoinCore(CoreSourceMetadata),
}

impl ImportProvenance {
    /// Back-compat accessor: returns `Some(&audit)` only for the `Bsms` variant.
    pub(crate) fn bsms_audit(&self) -> Option<&BsmsAuditFields> {
        match self {
            Self::Bsms(audit) => Some(audit),
            Self::BitcoinCore(_) => None,
        }
    }

    /// Back-compat accessor: returns `Some(&metadata)` only for the `BitcoinCore` variant.
    pub(crate) fn source_metadata(&self) -> Option<&CoreSourceMetadata> {
        match self {
            Self::Bsms(_) => None,
            Self::BitcoinCore(meta) => Some(meta),
        }
    }
}
```

- [ ] **Step 2: Update ParsedImport struct field pair → single field**

Edit `crates/mnemonic-toolkit/src/wallet_import/mod.rs` — locate the `ParsedImport` struct (~line 60-80) and replace:

```rust
pub bsms_audit: Option<BsmsAuditFields>,
pub source_metadata: Option<CoreSourceMetadata>,
```

with:

```rust
/// Source-provenance of this parsed import. Use `provenance.bsms_audit()` /
/// `provenance.source_metadata()` accessors to extract the (still-flat) wire
/// shape on the JSON envelope. See `ImportProvenance` for invariant.
pub(crate) provenance: ImportProvenance,
```

- [ ] **Step 3: Add ParsedImport accessor convenience methods (forward access through `provenance`)**

In the same file, add an `impl ParsedImport { ... }` block (or extend existing):

```rust
impl ParsedImport {
    /// Convenience: equivalent to `self.provenance.bsms_audit()`.
    pub(crate) fn bsms_audit(&self) -> Option<&BsmsAuditFields> {
        self.provenance.bsms_audit()
    }

    /// Convenience: equivalent to `self.provenance.source_metadata()`.
    pub(crate) fn source_metadata(&self) -> Option<&CoreSourceMetadata> {
        self.provenance.source_metadata()
    }
}
```

These keep call-site syntax `p.bsms_audit()` / `p.source_metadata()` working without changing every site to `p.provenance.bsms_audit()`.

- [ ] **Step 4: Run unit tests**

```bash
cargo test --lib provenance_tests
```

Expected: PASS (3 cells). If `Default` impls are missing, write minimal sample constructors instead (see Task 2.1 step 2 note).

- [ ] **Step 5: Build fails because callers still use field-access**

```bash
cargo build 2>&1 | grep 'error\[E' | head -10
```

Expected: errors at `cmd/import_wallet.rs:587,599,806,818,825` and `wallet_import/mod.rs:150,167` — field `bsms_audit` / `source_metadata` not found (now methods). Don't commit yet — Tasks 2.5-2.8 fix the call sites.

### Task 2.3: Update bsms.rs construction site (item 1 — Bsms variant)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:266`

- [ ] **Step 1: Read current construction site**

```bash
sed -n '260,275p' crates/mnemonic-toolkit/src/wallet_import/bsms.rs
```

Confirm `Ok(vec![ParsedImport { ... bsms_audit: audit, source_metadata: None, ... }])` at line 266.

- [ ] **Step 2: Replace with provenance-enum construction**

Edit `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` — at line 266, replace the `ParsedImport { ... }` block:

Before:
```rust
Ok(vec![ParsedImport {
    descriptor,
    original_descriptor: descriptor_body.to_string(),
    cosigners,
    network,
    threshold,
    bsms_audit: audit,
    source_metadata: None,
}])
```

After:
```rust
Ok(vec![ParsedImport {
    descriptor,
    original_descriptor: descriptor_body.to_string(),
    cosigners,
    network,
    threshold,
    provenance: ImportProvenance::Bsms(audit),
}])
```

Add `ImportProvenance` to the imports at the file head if not already in scope.

- [ ] **Step 3: cargo build incremental**

```bash
cargo build 2>&1 | grep -E 'error\[E|wallet_import/bsms' | head -5
```

Expected: bsms.rs error gone; bitcoin_core.rs + import_wallet.rs errors remain.

### Task 2.4: Update bitcoin_core.rs construction site (BitcoinCore variant)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:299-307`

- [ ] **Step 1: Read current construction site**

```bash
sed -n '289,310p' crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs
```

Confirm `Ok(ParsedImport { ... bsms_audit: None::<BsmsAuditFields>, source_metadata, ... })` spanning 299-307.

- [ ] **Step 2: Replace with provenance-enum construction**

Edit `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs` — at the `Ok(ParsedImport {` site (line 299):

Before (lines 299-307):
```rust
Ok(ParsedImport {
    descriptor,
    original_descriptor: desc_with_csum.to_string(),
    cosigners,
    network,
    threshold,
    bsms_audit: None::<BsmsAuditFields>,
    source_metadata,
})
```

After:
```rust
Ok(ParsedImport {
    descriptor,
    original_descriptor: desc_with_csum.to_string(),
    cosigners,
    network,
    threshold,
    provenance: ImportProvenance::BitcoinCore(source_metadata.expect("Bitcoin Core construction always builds source_metadata")),
})
```

Note: the let-binding at line 291 is `let source_metadata = Some(CoreSourceMetadata { ... })`. Either unwrap with `.expect()` (panic on bug) or refactor the let-binding to remove the `Some()` wrapper. Cleaner: refactor.

Refactor the let-binding at lines 291-297:

Before:
```rust
let source_metadata = Some(CoreSourceMetadata {
    active,
    internal,
    range,
    dropped_fields,
    wallet_name,
});
```

After:
```rust
let source_metadata = CoreSourceMetadata {
    active,
    internal,
    range,
    dropped_fields,
    wallet_name,
};
```

Then the construction site:

```rust
Ok(ParsedImport {
    descriptor,
    original_descriptor: desc_with_csum.to_string(),
    cosigners,
    network,
    threshold,
    provenance: ImportProvenance::BitcoinCore(source_metadata),
})
```

- [ ] **Step 3: Add `ImportProvenance` to imports if missing**

Check imports at the file head:

```bash
head -30 crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs
```

Add `ImportProvenance` to the `use` line for `wallet_import` (likely `use super::{ParsedImport, ...};` → add `ImportProvenance`).

- [ ] **Step 4: cargo build incremental**

```bash
cargo build 2>&1 | grep -E 'error\[E|wallet_import/bitcoin_core' | head -5
```

Expected: bitcoin_core.rs error gone; only `import_wallet.rs` errors remain.

### Task 2.5: Update apply_select_descriptor access sites (wallet_import/mod.rs:150, 167)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/mod.rs:150, 167`

- [ ] **Step 1: Read both sites + their distinct predicates**

```bash
sed -n '145,180p' crates/mnemonic-toolkit/src/wallet_import/mod.rs
```

Confirm two sites with **DIFFERENT closure bodies** (grep-verified at origin/master):
- `ActiveReceive` filter (around line 150-153): `.map(|m| m.active && !m.internal)` — note the `!` NEGATION on `internal`
- `ActiveChange` filter (around line 167-170): `.map(|m| m.active && m.internal)` — no negation

A single Before/After substitution would either miss one site OR silently corrupt the negation. Apply each site's edit separately, preserving each closure's predicate verbatim.

- [ ] **Step 2a: Replace ActiveReceive site — preserve `!m.internal` negation**

Before (around line 150-153 in `crates/mnemonic-toolkit/src/wallet_import/mod.rs`):
```rust
p.source_metadata
    .as_ref()
    .map(|m| m.active && !m.internal)
    .unwrap_or(false)
```

After:
```rust
p.source_metadata()
    .map(|m| m.active && !m.internal)
    .unwrap_or(false)
```

(Drops `p.source_metadata.as_ref()` → `p.source_metadata()`; accessor returns `Option<&_>` directly. **The closure body `m.active && !m.internal` is UNCHANGED.**)

- [ ] **Step 2b: Replace ActiveChange site — predicate has NO negation**

Before (around line 167-170):
```rust
p.source_metadata
    .as_ref()
    .map(|m| m.active && m.internal)
    .unwrap_or(false)
```

After:
```rust
p.source_metadata()
    .map(|m| m.active && m.internal)
    .unwrap_or(false)
```

(Same mechanical edit; closure body `m.active && m.internal` is UNCHANGED.)

- [ ] **Step 2c: Verify no closure-body drift**

```bash
grep -A 2 'p.source_metadata()' crates/mnemonic-toolkit/src/wallet_import/mod.rs | grep '\.map(|m|'
```

Expected: 2 lines — one with `!m.internal`, one without. If both lines have the same predicate, one site was silently corrupted; revert and re-apply Step 2a/2b separately.

- [ ] **Step 3: cargo build incremental**

```bash
cargo build 2>&1 | grep -E 'error\[E|wallet_import/mod' | head -5
```

Expected: mod.rs errors gone; only cmd/import_wallet.rs errors remain.

### Task 2.6: Update cmd/import_wallet.rs access sites (5 sites)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:587, 599, 806, 818, 825`

- [ ] **Step 1: Pattern A — `&p.bsms_audit` and `&p.source_metadata` (lines 587, 599, 825)**

For each occurrence of:
```rust
if let Some(audit) = &p.bsms_audit {
```
or
```rust
if let Some(meta) = &p.source_metadata {
```
or
```rust
if let Some(m) = &b.source_metadata {
```

Replace `&p.bsms_audit` → `p.bsms_audit()`, `&p.source_metadata` → `p.source_metadata()`, `&b.source_metadata` → `b.source_metadata()`. (Drops the `&` prefix; accessor returns `Option<&_>` directly.)

- [ ] **Step 2: Pattern B — `.is_some()` on owned field (lines 806, 818)**

For each occurrence of:
```rust
if b.bsms_audit.is_some() { ... }
```
or
```rust
if b.source_metadata.is_some() { ... }
```

Replace with:
```rust
if b.bsms_audit().is_some() { ... }
```
or
```rust
if b.source_metadata().is_some() { ... }
```

(Adds `()` for method call; `Option::is_some()` still works on the borrowed Option.)

- [ ] **Step 3: Verify full cargo build clean**

```bash
cargo build 2>&1 | tail -10
```

Expected: PASS with no errors. If any remain, grep the error file:line for additional access sites not enumerated above.

- [ ] **Step 4: Run all wallet_import-touching tests**

```bash
cargo test wallet_import
cargo test --test cli_import_wallet_envelope_v0_27_0
cargo test --test cli_json_envelopes
```

Expected: ALL PASS.

### Task 2.7: Drift-cell verification — envelope shape unchanged

**Files:** none (read-only verification)

- [ ] **Step 1: Run all drift-shape tests**

```bash
cargo test --test cli_import_wallet_envelope_v0_27_0
cargo test --test cli_xpub_search_drift_v0_27_0
```

Expected: ALL PASS. Wire shape preserved — `bsms_audit` and `source_metadata` JSON fields still appear as flat siblings (the serde derive on the struct hasn't changed; only the internal Rust representation has).

- [ ] **Step 2: Spot-check one envelope output manually**

```bash
cargo build --release
./target/release/mnemonic import-wallet --json --format bsms --blob - <<EOF
BSMS 1.0
...
EOF
```

(Use a real BSMS fixture from `tests/fixtures/wallet_import/`.) Expected: JSON envelope shape identical to v0.27.1 — `bsms_audit: {...}` and `source_metadata: null` at top level of the per-bundle entry.

### Task 2.8: Commit Phase 2

- [ ] **Step 1: Stage explicitly**

```bash
git add crates/mnemonic-toolkit/src/wallet_import/mod.rs
git add crates/mnemonic-toolkit/src/wallet_import/bsms.rs
git add crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs
git add crates/mnemonic-toolkit/src/cmd/import_wallet.rs
git status -sb
```

Expected: 4 files staged.

- [ ] **Step 2: Commit**

```bash
git commit -m "refactor(wallet-import): ImportProvenance enum on ParsedImport (Phase 5b)

Replaces the representable-invalid (Option<BsmsAuditFields>,
Option<CoreSourceMetadata>) pair with a single ImportProvenance enum.
Wire shape unchanged — back-compat accessors keep envelope JSON
emit code mechanically near-identical (5 import_wallet.rs sites +
2 apply_select_descriptor sites).

Tier promoted from v0.28+ to v0.27.2 per Shape A cleanup approval.

Closes design/FOLLOWUPS.md::pr-26-import-provenance-enum-internal-refactor"
```

### Task 2.9: Phase 2 architect review + persist verbatim

**Files:**
- Create: `design/agent-reports/v0_27_2-phase-2-architect-review.md`

- [ ] **Step 1: Dispatch opus architect review**

Brief:
```
Review the Phase 2 ImportProvenance refactor commit on release/v0.27.2.
Verify:
- Enum + accessors at wallet_import/mod.rs match spec §2 item 1
- bsms.rs:266 construction site uses ImportProvenance::Bsms correctly
- bitcoin_core.rs:291-307 refactor preserves CoreSourceMetadata shape;
  the Some() unwrap was removed cleanly
- 5 access sites in cmd/import_wallet.rs:{587,599,806,818,825} use accessors
- 2 access sites in mod.rs:{150,167} use accessors
- Drift cells at cli_import_wallet_envelope_v0_27_0.rs still pass
- No new ToolkitError variant introduced (item 2's alphabetical rule N/A)
```

- [ ] **Step 2: Persist verbatim**

Save to `design/agent-reports/v0_27_2-phase-2-architect-review.md`.

- [ ] **Step 3: Fold findings until 0 Critical / 0 Important; re-dispatch each round**

- [ ] **Step 4: Commit review log**

```bash
git add design/agent-reports/v0_27_2-phase-2-architect-review.md
git commit -m "docs(agent-reports): v0.27.2 Phase 2 architect review (GREEN)"
```

---

## Phase 3 — Sibling lockstep (item 7) — mnemonic-gui v0.11.1

> ⚠️ **EXECUTION ORDER NOTE — read before dispatching this phase:** Phase 3 is sequenced **AFTER Phase 4**. Document order here does NOT match execution order. Execute Phase 4 (Toolkit cycle close → tag → GH release → install-pin-check CI) FIRST, THEN return here. The phase numbering reflects the spec's conceptual grouping ("Phase 3 = sibling work"); the actual ship rule is toolkit-first, GUI-second (per `design/PLAN_v0_26_0_three_way_merge.md`). Any agent doing linear-order traversal must skip Phase 3 on first pass and return after Phase 4 completes.

**Goal:** Land mnemonic-gui v0.11.1: workflow YAML extension + toolkit pin bump v0.26.0 → v0.27.2 + envelope smoke cells.

**Sequencing pre-condition:** `mnemonic-toolkit-v0.27.2` tag exists on `origin/master`; GH release published. The GUI cycle pulls the toolkit v0.27.2 tag via `pinned-upstream.toml`.

**Working directory:** `/scratch/code/shibboleth/mnemonic-gui/` (sibling repo).

### Task 3.1: Branch mnemonic-gui v0.11.1 off master

- [ ] **Step 1: Fetch + branch**

```bash
cd /scratch/code/shibboleth/mnemonic-gui
git fetch origin master
git checkout -b release/v0.11.1 origin/master
git status -sb
```

Expected: clean working tree on the new branch.

### Task 3.2: Extend workflow trigger filter (build.yml + schema-mirror.yml)

**Files:**
- Modify: `/scratch/code/shibboleth/mnemonic-gui/.github/workflows/build.yml`
- Modify: `/scratch/code/shibboleth/mnemonic-gui/.github/workflows/schema-mirror.yml`

- [ ] **Step 1: Read current trigger blocks**

```bash
grep -A 3 'pull_request:' .github/workflows/build.yml
grep -A 3 'pull_request:' .github/workflows/schema-mirror.yml
```

Confirm both have `branches: [master]`.

- [ ] **Step 2: Edit build.yml**

Replace:
```yaml
pull_request:
  branches: [master]
```

With:
```yaml
pull_request:
  branches:
    - master
    - "release/**"
```

- [ ] **Step 3: Edit schema-mirror.yml — same replacement**

- [ ] **Step 4: Validate YAML syntax**

```bash
actionlint .github/workflows/build.yml .github/workflows/schema-mirror.yml
```

Expected: no errors. (Per memory `feedback-r2-blocking-vs-cosmetic-gate`: YAML parse errors are Important regardless of test-result-correctness reasoning.)

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/build.yml .github/workflows/schema-mirror.yml
git commit -m "ci(workflows): trigger on PRs targeting release/** branches

Eliminates the silent-skip pattern that v0.11.0 cycle hit (no CI on
PRs targeting release/v0.11.0). Glob pattern \"release/**\" matches
all integration branches.

Closes mnemonic-gui FOLLOWUPS::gui-workflow-trigger-include-release-branches
+ companion toolkit FOLLOWUP."
```

### Task 3.3: Toolkit pin bump v0.26.0 → v0.27.2

**Files:**
- Modify: `/scratch/code/shibboleth/mnemonic-gui/Cargo.toml`
- Modify: `/scratch/code/shibboleth/mnemonic-gui/pinned-upstream.toml`

- [ ] **Step 1: Read current pin sites**

```bash
grep -n 'mnemonic-toolkit' Cargo.toml pinned-upstream.toml
```

Confirm both reference `v0.26.0` (or whatever Phase 0.5 verified).

- [ ] **Step 2: Edit Cargo.toml**

Replace `mnemonic-toolkit-v0.26.0` with `mnemonic-toolkit-v0.27.2` (in `[dependencies]` section's git/tag spec).

- [ ] **Step 3: Edit pinned-upstream.toml**

Replace the toolkit tag entry. If structure is `[mnemonic-toolkit]` table with `tag = "mnemonic-toolkit-v0.26.0"`, replace with `tag = "mnemonic-toolkit-v0.27.2"`.

- [ ] **Step 4: Regenerate Cargo.lock**

```bash
cargo build --workspace 2>&1 | tail -10
```

Expected: builds clean against toolkit v0.27.2. If any compilation errors surface from v0.27.x envelope-shape changes, capture them — they become Task 3.4 GUI smoke cells.

- [ ] **Step 5: Run existing GUI tests against new toolkit pin**

```bash
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/release/mnemonic \
MD_BIN=/scratch/code/shibboleth/descriptor-mnemonic/target/release/md \
MS_BIN=/scratch/code/shibboleth/mnemonic-secret/target/release/ms \
MK_BIN=/scratch/code/shibboleth/mnemonic-key/target/release/mk \
MNEMONIC_GUI_UPSTREAM_ROOT=/scratch/code/shibboleth/mnemonic-toolkit \
cargo test --workspace --no-fail-fast 2>&1 | tail -20
```

Expected: ALL PASS (or only unrelated flakes). If `schema_mirror` or `conditional_visibility` tests fail because of v0.27.0/v0.27.1 wire-shape changes (envelope replacement, xpub-search result types), capture and address in Task 3.4.

- [ ] **Step 6: Commit pin bump + lockfile**

```bash
git add Cargo.toml pinned-upstream.toml Cargo.lock
git commit -m "chore(deps): bump mnemonic-toolkit pin v0.26.0 → v0.27.2

Catches up to v0.27.0 (envelope wire-shape replacement) +
v0.27.1 (PR-#26 fold) + v0.27.2 (this cycle's cleanup).
Closes silent toolkit-drift gap surfaced in v0.27.2 brainstorm M3."
```

### Task 3.4: GUI smoke cells for v0.27.x envelope shape

**Files:**
- Create: `/scratch/code/shibboleth/mnemonic-gui/tests/cli_envelope_smoke.rs`

- [ ] **Step 1: Identify which envelope shapes GUI consumes**

Based on Phase 0.5 Step 2 grep results, list the GUI source files that parse:
- `import-wallet --json` (v0.27.0 BundleJson envelope)
- `xpub-search` result envelopes
- `bsms_round1` verifications (if exposed)

- [ ] **Step 2: Write smoke test file**

Create `tests/cli_envelope_smoke.rs`:

```rust
//! v0.27.x toolkit envelope-shape smoke cells. Verifies the GUI's envelope
//! consumers parse the post-v0.26.0 wire shapes without panic / shape drift.
//! Added in mnemonic-gui v0.11.1 alongside the toolkit pin bump.

use std::process::Command;

const TOOLKIT_BIN: &str = env!("MNEMONIC_BIN");

#[test]
fn import_wallet_json_envelope_parses_v0_27_x_shape() {
    // Grep-verified at envelope_v0_27_0.json: top-level shape is a JSON ARRAY
    // (each element = one bundle). Per-bundle keys: schema_version, source_format,
    // bundle, roundtrip.
    let fixture = include_str!("../../mnemonic-toolkit/crates/mnemonic-toolkit/tests/fixtures/wallet_import/envelope_v0_27_0.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("v0.27.0 envelope parses");
    let entries = parsed.as_array().expect("top-level is a JSON array");
    assert!(!entries.is_empty(), "envelope has at least one bundle entry");
    let entry = &entries[0];
    assert_eq!(entry.get("schema_version").and_then(|v| v.as_str()), Some("1"));
    assert!(entry.get("bundle").is_some(), "entry has bundle field");
    // v0.27.0 replaced compact-summary with full BundleJson; verify the new shape
    let bundle = entry.get("bundle").unwrap();
    assert!(bundle.get("descriptor").is_some(), "bundle has descriptor field (full BundleJson, not compact)");
}

#[test]
fn xpub_search_path_of_xpub_match_envelope_parses() {
    let fixture = include_str!("../../mnemonic-toolkit/crates/mnemonic-toolkit/tests/fixtures/v0_27_0_envelopes/path_of_xpub.match.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("path_of_xpub match envelope parses");
    assert_eq!(parsed.get("result").and_then(|v| v.as_str()), Some("match"));
    assert!(parsed.get("path").is_some());
}

#[test]
fn xpub_search_path_of_xpub_no_match_envelope_parses() {
    let fixture = include_str!("../../mnemonic-toolkit/crates/mnemonic-toolkit/tests/fixtures/v0_27_0_envelopes/path_of_xpub.no_match.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("path_of_xpub no_match envelope parses");
    assert_eq!(parsed.get("result").and_then(|v| v.as_str()), Some("no_match"));
    // Per v0.27.1 Phase 5a: result=no_match implies no path field (or path: null with explicit-null serde)
}

#[test]
fn xpub_search_account_of_descriptor_envelope_parses() {
    // Grep-verified at account_of_descriptor.match.json: per-mode shape has
    // `matched_cosigners[i].{cosigner_index, path, template, account}` — there
    // is NO top-level `account` field.
    let fixture = include_str!("../../mnemonic-toolkit/crates/mnemonic-toolkit/tests/fixtures/v0_27_0_envelopes/account_of_descriptor.match.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("account_of_descriptor match envelope parses");
    let matched = parsed.get("matched_cosigners").expect("matched_cosigners present");
    let first = matched.get(0).expect("at least one matched cosigner");
    assert!(first.get("account").is_some(), "matched_cosigners[0].account present");
}

#[test]
fn xpub_search_passphrase_of_xpub_envelope_parses() {
    let fixture = include_str!("../../mnemonic-toolkit/crates/mnemonic-toolkit/tests/fixtures/v0_27_0_envelopes/passphrase_of_xpub.match.json");
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("passphrase_of_xpub match envelope parses");
    assert_eq!(parsed.get("result").and_then(|v| v.as_str()), Some("match"));
}
```

Note: `include_str!` with `../../` paths needs the toolkit checkout to be a sibling directory. If the GUI's `Cargo.toml` uses `git` dep (not path dep), these fixtures aren't available at build time — refactor to ship the fixtures in the GUI repo OR use `serde_json::from_str` against inline JSON literals derived from the toolkit fixtures.

Inline-JSON fallback if `include_str!` doesn't resolve:

```rust
#[test]
fn import_wallet_json_envelope_parses_v0_27_x_shape() {
    let fixture = r#"{
        "schema_version": "1",
        "bundle": {
            "descriptor": "wsh(sortedmulti(2,xpub.../0/*,xpub.../0/*))",
            "mk1": [...],
            "network": "mainnet"
        }
    }"#;
    let parsed: serde_json::Value = serde_json::from_str(fixture).expect("v0.27.0 envelope parses");
    // ... same assertions
}
```

- [ ] **Step 3: Run smoke tests**

```bash
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/release/mnemonic \
cargo test --test cli_envelope_smoke
```

Expected: 5/5 PASS.

- [ ] **Step 4: Commit**

```bash
git add tests/cli_envelope_smoke.rs
git commit -m "test(envelope-smoke): v0.27.x toolkit envelope shape cells (v0.11.1)

Smoke surface verifying v0.27.0 BundleJson envelope replacement +
v0.27.1 xpub-search result envelopes parse cleanly under the new
toolkit pin. Closes silent-drift-detection gap for future toolkit
bumps."
```

### Task 3.5: mnemonic-gui CHANGELOG entry + version bump

**Files:**
- Modify: `/scratch/code/shibboleth/mnemonic-gui/CHANGELOG.md`
- Modify: `/scratch/code/shibboleth/mnemonic-gui/Cargo.toml` (GUI's own version field)

- [ ] **Step 1: Read current CHANGELOG top**

```bash
head -30 CHANGELOG.md
```

Confirm v0.11.0 section structure.

- [ ] **Step 2: Add v0.11.1 entry**

Insert at top, above v0.11.0:

```markdown
## mnemonic-gui [0.11.1] — <YYYY-MM-DD>

### Changed

- **CI workflow triggers extended to release branches.** `build.yml` and `schema-mirror.yml` now run on PRs targeting `master` AND `release/**` (previously only `master`). Eliminates the silent-skip pattern v0.11.0 cycle worked around via `--admin` merges.
- **mnemonic-toolkit pin bump v0.26.0 → v0.27.2.** Catches up to v0.27.0 cross-format wallet conversion (envelope wire-shape replacement) + v0.27.1 PR-#26 fold + v0.27.2 cleanup. GUI envelope-consumer smoke cells added in `tests/cli_envelope_smoke.rs` for shape stability.

### Closed FOLLOWUPS

- `gui-workflow-trigger-include-release-branches`
```

- [ ] **Step 3: Bump GUI Cargo.toml version**

```bash
grep -n '^version = ' Cargo.toml
```

Replace `version = "0.11.0"` with `version = "0.11.1"`.

- [ ] **Step 4: Regenerate Cargo.lock**

```bash
cargo build --workspace 2>&1 | tail -3
```

- [ ] **Step 5: Commit**

```bash
git add CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "release(gui): mnemonic-gui v0.11.1 — workflow + toolkit pin lockstep"
```

### Task 3.6: GUI PR + tag + release

- [ ] **Step 1: Push branch**

```bash
git push -u origin release/v0.11.1
```

- [ ] **Step 2: Open PR to master**

```bash
gh pr create --base master --head release/v0.11.1 \
  --title "release(gui): mnemonic-gui v0.11.1 — workflow + toolkit pin v0.27.2 lockstep" \
  --body "$(cat <<'EOF'
## Summary

- Extends CI workflow triggers to release branches (`branches: [master, "release/**"]`) — eliminates the v0.11.0 cycle's silent-skip
- Bumps `mnemonic-toolkit` pin v0.26.0 → v0.27.2 (catches up v0.27.0 envelope replacement + v0.27.1 fold + v0.27.2 cleanup)
- Adds `tests/cli_envelope_smoke.rs` smoke cells for v0.27.x envelope shape stability

## Closed FOLLOWUPS

- `gui-workflow-trigger-include-release-branches` (toolkit companion: closed at v0.27.2)

## Test plan

- [x] CI green on this PR (workflow trigger fires correctly via the new pattern — first-firing-after-merge will also be the regression-gate)
- [x] `tests/cli_envelope_smoke.rs` 5/5 PASS
- [x] All existing GUI tests pass against toolkit v0.27.2 pin

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3: Wait for CI; squash-merge**

```bash
# After CI green:
gh pr merge --squash --delete-branch
```

- [ ] **Step 4: Tag + release on the squash commit**

```bash
git checkout master
git pull origin master
git tag mnemonic-gui-v0.11.1
git push origin mnemonic-gui-v0.11.1
gh release create mnemonic-gui-v0.11.1 \
  --title "mnemonic-gui v0.11.1 — workflow + toolkit pin v0.27.2 lockstep" \
  --notes "$(awk '/^## mnemonic-gui \[0.11.1\]/,/^## mnemonic-gui \[0.11.0\]/' CHANGELOG.md | head -n -1)"
```

---

## Phase 4 — Toolkit cycle close

> ✅ **EXECUTION ORDER NOTE:** Phase 4 executes **immediately after Phase 2 closes** (Phase 1 already merged by then). Do not advance to Phase 3 until this phase ships the toolkit tag + GH release. The sibling Phase 3 (mnemonic-gui v0.11.1) consumes the toolkit tag emitted here.

**Goal:** Land CHANGELOG + FOLLOWUPS Status flips + Cargo bumps + install.sh pin + tag + release.

**Working directory:** `/scratch/code/shibboleth/mnemonic-toolkit/` on `release/v0.27.2`.

### Task 4.1: CHANGELOG.md [0.27.2] entry

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Read current CHANGELOG top**

```bash
head -30 CHANGELOG.md
```

- [ ] **Step 2: Insert v0.27.2 entry above v0.27.1**

```markdown
## mnemonic-toolkit [0.27.2] — <YYYY-MM-DD>

Cleanup cycle closing 7 v0.27-tier FOLLOWUPs. Anchored on Phase 5b's deferred `ImportProvenance` enum refactor (tier promoted from `v0.28+` per Shape A approval). Sibling lockstep: mnemonic-gui v0.11.1 (workflow trigger + toolkit pin bump). Zero wire-shape change; patch bump valid.

### Fixed

- **xpub-search address-of-xpub `searched` count semantic clarified** (item 6, doc-only). The aggregate `searched` field on `ToolkitError::XpubSearchNoMatch` reports **candidate-comparisons performed** (`n_targets × gap_limit × chains`), not unique child-addresses derived. The existing docstring at `error.rs:230-237` previously elided the `n_targets` factor for address mode; restored. Per-target `scanned_external` / `scanned_internal` JSON fields (in `AddressOfXpubResult`) already report unique candidates per-target — unchanged. Closes `xpub-search-address-of-xpub-searched-count-semantic`.
- **`mlock_unit::g1_1_single_page_pin_has_page_count_one` no longer flakes under parallel test execution** (item 4). Switched from `Box::new([0u8; 64])` heap-allocator-luck buffer to `std::alloc::alloc` with explicit page-aligned `Layout`. Closes `mlock-g1-1-test-page-alignment-luck`.

### Changed

- **`ParsedImport` internal representation** (item 1, internal refactor). Replaces the representable-invalid `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)` pair with a single `provenance: ImportProvenance` enum. Wire shape unchanged — envelope-side `bsms_audit` / `source_metadata` JSON fields remain flat siblings via back-compat accessors. Closes `pr-26-import-provenance-enum-internal-refactor` (tier promoted from v0.28+).

### Tests

- **+1 cell** `dispatcher_arm_count_matches_pinned_constant` in `tests/cli_gui_schema_conditional_rules.rs` — regression guard for `build_subcommand_conditional_rules` arm count drift (currently pinned at 6). Closes `gui-schema-arm-drop-detector`.
- **+3 unit cells** in `wallet_import/mod.rs::provenance_tests` for the new `ImportProvenance` enum + accessors (item 1).

### Closed FOLLOWUPS

- `pr-26-import-provenance-enum-internal-refactor` (Phase 2; tier promoted from v0.28+)
- `error-rs-canonical-ordering-doc` (Phase 1.1)
- `compare-cost-agent-reports-back-fill` (Phase 1.2)
- `mlock-g1-1-test-page-alignment-luck` (Phase 1.3)
- `gui-schema-arm-drop-detector` (Phase 1.4)
- `xpub-search-address-of-xpub-searched-count-semantic` (Phase 1.5)
- `gui-workflow-trigger-include-release-branches` (sibling close at mnemonic-gui v0.11.1)
```

- [ ] **Step 3: Stage explicitly**

```bash
git add CHANGELOG.md
```

(Do not commit yet — Phase 4 batches the close as one or two commits.)

### Task 4.2: FOLLOWUPS.md Status flips + tier promotion

**Files:**
- Modify: `design/FOLLOWUPS.md`

- [ ] **Step 1: For each of 7 entries, flip `Status: open` → `Status: resolved <SHA>`**

Slugs to update (use the Phase 2/3 commit SHAs from `git log`):

```
pr-26-import-provenance-enum-internal-refactor
error-rs-canonical-ordering-doc
compare-cost-agent-reports-back-fill
mlock-g1-1-test-page-alignment-luck
gui-schema-arm-drop-detector
xpub-search-address-of-xpub-searched-count-semantic
gui-workflow-trigger-include-release-branches
```

For each: locate the entry's `**Status:**` line in `design/FOLLOWUPS.md` and replace:

```markdown
- **Status:** open
```

with:

```markdown
- **Status:** resolved (<short-sha>; v0.27.2 Phase <N>)
```

- [ ] **Step 2: Tier promotion for item 1**

For `pr-26-import-provenance-enum-internal-refactor`, also change:

```markdown
- **Tier:** `v0.28+`
```

to:

```markdown
- **Tier:** `v0.27.2` (resolved; promoted from v0.28+ per Shape A cleanup approval)
```

- [ ] **Step 3: Stage**

```bash
git add design/FOLLOWUPS.md
```

### Task 4.3: Cargo.toml version bump

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml`

- [ ] **Step 1: Read current version**

```bash
grep -n '^version' crates/mnemonic-toolkit/Cargo.toml | head -3
```

Expected: `version = "0.27.1"`.

- [ ] **Step 2: Bump to 0.27.2**

Edit the line to `version = "0.27.2"`.

- [ ] **Step 3: Verify root Cargo.toml has no workspace-version field (sanity)**

```bash
grep -nE '^version|^\[workspace\.package\]' Cargo.toml
```

Expected (grep-verified at plan-write time): root `Cargo.toml` has `[workspace.package]` with `edition`, `license`, `repository`, `homepage`, `rust-version` — but NO `version` field (per-crate versions only). No action needed. If a future cycle adds a workspace-level `version`, also bump here.

- [ ] **Step 4: Stage**

```bash
git add crates/mnemonic-toolkit/Cargo.toml
# also root Cargo.toml if applicable
```

### Task 4.4: Cargo.lock regenerate (cargo build)

**Files:**
- Modify: `Cargo.lock`

- [ ] **Step 1: Run cargo build (NOT cargo check) to regenerate lockfile**

```bash
cargo build --workspace 2>&1 | tail -3
```

Expected: builds clean. `Cargo.lock` now shows `version = "0.27.2"` for the `mnemonic-toolkit` package.

- [ ] **Step 2: Verify**

```bash
grep -A 1 'name = "mnemonic-toolkit"' Cargo.lock | head -3
```

Expected: `version = "0.27.2"`.

- [ ] **Step 3: Stage explicitly**

```bash
git add Cargo.lock
git diff --cached -- Cargo.lock  # verify the diff is the version bump
```

(Per `feedback-phase-6-cargo-lock-stage-with-version-bump` — don't trust "build passed" as evidence the lockfile is staged.)

### Task 4.5: scripts/install.sh self-pin bump

**Files:**
- Modify: `scripts/install.sh`

- [ ] **Step 1: Find the TAG line**

```bash
grep -n 'TAG=' scripts/install.sh | head -5
```

Expected: `TAG="mnemonic-toolkit-v0.27.1"` (or similar).

- [ ] **Step 2: Bump to v0.27.2**

Replace `mnemonic-toolkit-v0.27.1` with `mnemonic-toolkit-v0.27.2`.

- [ ] **Step 3: Verify shell syntax still valid**

```bash
sh -n scripts/install.sh
```

Expected: no syntax errors.

- [ ] **Step 4: Stage**

```bash
git add scripts/install.sh
```

### Task 4.6: Manual-no-flag-delta confirmation

**Files:** none (verification only)

- [ ] **Step 1: Confirm no new CLI flags this cycle**

Items 1-6 are internal-refactor + doc + test. No `clap::Args` or `clap::Subcommand` definitions changed. Verify:

```bash
git diff origin/master...HEAD -- crates/mnemonic-toolkit/src/cmd/ | grep -E '^\+.*(#\[arg|#\[command|long =|short =)' | head -10
```

Expected: zero matches.

- [ ] **Step 2: Run manual lint (smoke) — use installer-managed sibling-CLI paths**

```bash
cargo build --release
make -C docs/manual lint \
  MNEMONIC_BIN=$PWD/target/release/mnemonic \
  MD_BIN=$HOME/.cargo/bin/md \
  MS_BIN=$HOME/.cargo/bin/ms \
  MK_BIN=$HOME/.cargo/bin/mk \
  2>&1 | tail -10
```

Expected: PASS — bidirectional flag-coverage check confirms no manual chapter drift. Uses installer-managed sibling-CLI paths so this doesn't require local sibling-repo checkouts to be built. If `~/.cargo/bin/{md,ms,mk}` are missing (fresh install), fall back to building them locally OR check that they're installed via `scripts/install.sh`.

### Task 4.7: Commit Phase 4 cycle close

- [ ] **Step 1: Stage all Phase 4 artifacts together**

```bash
git status -sb
```

Expected stage list:
- `CHANGELOG.md`
- `design/FOLLOWUPS.md`
- `crates/mnemonic-toolkit/Cargo.toml`
- `Cargo.lock`
- `scripts/install.sh`

- [ ] **Step 2: Commit**

```bash
git commit -m "release(toolkit): mnemonic-toolkit v0.27.2 — 7-FOLLOWUP cleanup cycle

CHANGELOG + FOLLOWUPS Status flips (7 entries; tier promotion for
pr-26-import-provenance-enum-internal-refactor from v0.28+) +
Cargo.toml/Cargo.lock version bump + install.sh self-pin.

Sibling: mnemonic-gui v0.11.1 (workflow + toolkit pin) ships separately.
Wire-shape unchanged; patch-tier valid."
```

### Task 4.8: PR-merge sequencing + tag

- [ ] **Step 1: Push branch**

```bash
git push -u origin release/v0.27.2
```

- [ ] **Step 2: Open PR to master**

```bash
gh pr create --base master --head release/v0.27.2 \
  --title "release(toolkit): mnemonic-toolkit v0.27.2 — 7-FOLLOWUP cleanup" \
  --body "$(awk '/^## mnemonic-toolkit \[0.27.2\]/,/^## mnemonic-toolkit \[0.27.1\]/' CHANGELOG.md | head -n -1)"
```

- [ ] **Step 3: Wait for CI green**

```bash
gh pr checks <PR-number> --watch
```

- [ ] **Step 4: Squash-merge**

```bash
gh pr merge <PR-number> --squash --delete-branch
```

- [ ] **Step 5: Fetch + tag on master squash commit**

```bash
git checkout master
git pull origin master
git tag mnemonic-toolkit-v0.27.2
git push origin mnemonic-toolkit-v0.27.2
```

- [ ] **Step 6: GH release**

```bash
gh release create mnemonic-toolkit-v0.27.2 \
  --title "mnemonic-toolkit v0.27.2 — 7-FOLLOWUP cleanup cycle" \
  --notes "$(awk '/^## mnemonic-toolkit \[0.27.2\]/,/^## mnemonic-toolkit \[0.27.1\]/' CHANGELOG.md | head -n -1)"
```

### Task 4.9: install-pin-check CI verify

- [ ] **Step 1: Wait for tag-fire CI**

```bash
gh run watch
```

Expected: `install-pin-check` job GREEN. If RED — install.sh self-pin drift; hotfix on master with corrected TAG and re-tag.

- [ ] **Step 2: End-of-cycle holistic architect review**

Dispatch opus architect with brief:
```
Review the v0.27.2 cycle close (master squash + tag). Verify:
- CHANGELOG entry matches the 7 closed FOLLOWUPs
- FOLLOWUPS Status flips all applied
- Cargo.toml + Cargo.lock + install.sh all bumped to v0.27.2
- No CLI flag delta (manual chapter drift)
- Tag points at master squash commit
- GH release notes mirror CHANGELOG body
```

Persist verbatim to `design/agent-reports/v0_27_2-end-of-cycle-architect-review.md` per Convention.

If GREEN: cycle complete. THEN trigger Phase 3 (mnemonic-gui v0.11.1) if not already shipped.

If YELLOW/RED: fold findings, post-tag if necessary (matches v0.27.1's `41a6caa` install.sh hotfix precedent).

---

## Self-review notes

- **Spec coverage check:** all 7 items in spec §2 mapped to tasks 1.1-1.5, 2.1-2.8, 3.1-3.6. ✓
- **Phase 4 hygiene checklist:** all 11 spec §3 Phase 4 steps mapped to Tasks 4.1-4.9. ✓
- **No placeholders:** every "Step N" has actual code or actual command. ✓
- **Type consistency:** `ImportProvenance::Bsms`/`BitcoinCore`, `bsms_audit()`/`source_metadata()` accessors used identically in Tasks 2.1-2.6. ✓
- **Sibling lockstep ordering:** Phase 3 sequenced AFTER Phase 4 (toolkit-first, GUI-second per spec §1 + §6). ✓
- **Architect dispatches:** R0 at Phase 1 close (Task 1.6) + Phase 2 close (Task 2.9) + end-of-cycle (Task 4.9). Each persists verbatim per new Convention. ✓

---

## Related memories

- [[project-v0-27-1-cycle-shipped]] — predecessor
- [[feedback-pre-brainstorm-fetch-origin-tip-check]] — Phase 0 sync check discipline
- [[feedback-followups-md-line-numbers-presumed-stale]] — Phase 0 grep-verify discipline
- [[feedback-architect-redispatch-after-every-fold-round]] — Phase 1.6, 2.9, 4.9 reviewer-loop
- [[feedback-grep-verify-during-fold-not-just-during-write]] — when folding architect findings
- [[feedback-phase-6-cargo-lock-stage-with-version-bump]] — Task 4.4
- [[feedback-phase-6-install-sh-pin-bump-required]] — Task 4.5
- [[feedback-per-phase-agents-forget-followup-status-flip]] — Task 4.2
- [[feedback-smaller-cycle-scope-reduces-citation-surface]] — 7-item scope chosen; budget more reviewer rounds
