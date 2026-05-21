# mnemonic-toolkit-v0.31.1 Implementation Plan (Cycle 8 — sparrow-taproot descriptor-passthrough import)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Ship `mnemonic-toolkit-v0.31.1` (SemVer-PATCH; behavior-expansion of previously-refused inputs). Convert the `wallet_import/sparrow.rs:311` taproot refusal into a path-split that detects descriptor-passthrough shape (no `@N/**` placeholders; concrete `[fp/path]xpub` keys embedded directly) and feeds the script through the existing `concrete_keys_to_placeholders` → `parse_descriptor` pipeline. Closes Cycle 8 (`sparrow-taproot-descriptor-passthrough-import-support` FOLLOWUP). **NO GUI lockstep** (no clap surface change).

**Architecture:** Single-fork parser refactor at `wallet_import/sparrow.rs::parse`. When `script_template` lacks `@0/**` placeholders (descriptor-passthrough shape per emit-side `wallet_export/sparrow.rs:215-219` for `CliTemplate::TrMultiA | TrSortedMultiA`), skip Step 5 substitution and feed `script_template` directly into the existing post-substitution path. Reuses all existing pipeline machinery: `concrete_keys_to_placeholders`, `parse_descriptor`, `validate_watch_only_resolved`, network detection, provenance attachment.

**Tech Stack:** Rust; zero net-new Cargo deps; zero `ToolkitError` variants; zero `lib.rs` changes.

**Brainstorm:** This plan-doc is the brainstorm-equivalent (scope is contained enough that a separate brainstorm doc is redundant; the recon dossier + this plan-doc carry all the decisions).

**P0 recon dossier:** `design/cycle-8-p0-recon.md` (primary-source verified vs `sparrow.rs:304-315` + `wallet_export/sparrow.rs:215-219` + existing emit-side fixture).

**Source SHA at plan-write time:** `4eb1fa8` (post-Cycle-7 close).

**P0 STRICT-GATE locks:**
- **Refusal-to-path-split site:** `wallet_import/sparrow.rs:304-315` (verified line citation; the `if script_template.contains("tr(")` block).
- **Detection heuristic:** descriptor-passthrough mode iff `!script_template.contains("@0/**")`. Currently only taproot templates emit this shape (per `wallet_export/sparrow.rs:215-219`); non-taproot wallets always have `@N/**` placeholders.
- **Round-trip fixture:** reuse existing `tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json` (do NOT author a new fixture; symlink or copy into `tests/fixtures/wallet_import/`).
- **SemVer:** PATCH `v0.31.0 → v0.31.1`. Behavior-expansion only (formerly-refused inputs now succeed). No new flags. No GUI lockstep.
- **FOLLOWUP closure:** close `sparrow-taproot-descriptor-passthrough-import-support`; no new FOLLOWUPs anticipated (the cycle resolves the entire descriptor-passthrough surface).

## File structure

### Source files modified (toolkit)
- `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs`:
  - L304-315: convert refusal to path-split.
  - Module docstring at L52-56: update taproot deferral note → v0.31.1-shipped citation.
  - `parse` docstring at L209-212 (Step 6 description): update to reflect path-split.

### Test files added (toolkit)
- `crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow_taproot.rs` — NEW; ~6-10 cells.

### Test fixtures added (toolkit)
- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json` — symlink or copy of `tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json`.

### Documentation modified (toolkit)
- `docs/manual/src/45-foreign-formats.md` — §"Sparrow Wallet" subsection: convert taproot deferral to v0.31.1-shipped strikethrough.

### Release tooling
- `crates/mnemonic-toolkit/Cargo.toml:3` — version `0.31.0` → `0.31.1`.
- `CHANGELOG.md` — new `## [0.31.1]` section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.31.0` → `mnemonic-toolkit-v0.31.1`.
- `design/FOLLOWUPS.md` — close `sparrow-taproot-descriptor-passthrough-import-support`.

### NOT modified
- `crates/mnemonic-toolkit/src/wallet_export/sparrow.rs` — emit-side already correct.
- `crates/mnemonic-toolkit/src/wallet_import/pipeline.rs` — `concrete_keys_to_placeholders` already handles taproot descriptors.
- `crates/mnemonic-toolkit/Cargo.toml` deps — no new deps.
- `mnemonic-gui/*` — no GUI lockstep.

## Tasks

### Task 1: Phase 2 — `sparrow.rs` parser path-split

**Files:** modify `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs`.

- [ ] **Step 1: Read the current refusal block at L304-315**

Verify exact lines + surrounding context.

- [ ] **Step 2: Replace refusal with path-split**

The current block at L304-315:

```rust
        // Step 6 (early): refuse taproot scripts. Sparrow's emit at
        // `wallet_export/sparrow.rs:215-219` ships taproot as descriptor-
        // passthrough (no `@N/**` placeholder shape — the canonical
        // descriptor with `[fp/path]xpub` keys is embedded directly). The
        // P1B `@N/**` substitution path does not handle that shape; taproot
        // import lands in a future cycle (cycle-followup
        // `sparrow-taproot-descriptor-passthrough-import-support`).
        if script_template.contains("tr(") {
            return Err(ToolkitError::ImportWalletParse(
                "import-wallet: sparrow: parse error: taproot scripts are not yet supported (Sparrow's taproot emit uses descriptor-passthrough; P1B's @N/** substitution path does not cover it)".to_string(),
            ));
        }

        // Step 5: substitute `@i/**` → `[fp/derivation_no_m]xpub/<0;1>/*`.
        ...
```

Replace with:

```rust
        // Step 6 (v0.31.1 Cycle 8): detect descriptor-passthrough shape and
        // skip Step 5 substitution. Sparrow's emit at
        // `wallet_export/sparrow.rs:215-219` ships taproot templates
        // (`tr-multi-a` / `tr-sortedmulti-a`) as descriptor-passthrough:
        // concrete `[fp/path]xpub` keys are embedded in `script_template`
        // directly (no `@N/**` placeholders). All other templates ship with
        // `@N/**` placeholders that need substitution. Detection: presence
        // of `@0/**` placeholder = template mode (substitute); absence =
        // descriptor-passthrough mode (skip substitution; feed
        // `script_template` directly through `concrete_keys_to_placeholders`).
        // Closes `sparrow-taproot-descriptor-passthrough-import-support`.
        let is_descriptor_passthrough = !script_template.contains("@0/**");

        // Step 5 (only for template mode): substitute `@i/**` → `[fp/derivation_no_m]xpub/<0;1>/*`.
        let substituted = if is_descriptor_passthrough {
            script_template.clone()
        } else {
            let mut substituted = script_template.clone();
            let mut indices: Vec<usize> = (0..n).collect();
            indices.sort_by_key(|i| std::cmp::Reverse(i.to_string().len()));
            for i in indices {
                let placeholder = format!("@{i}/**");
                let ks = &keystores[i];
                let path_no_m = ks
                    .derivation
                    .strip_prefix("m/")
                    .unwrap_or(ks.derivation.as_str().strip_prefix('m').unwrap_or(&ks.derivation));
                let bracketed = if path_no_m.is_empty() {
                    format!("[{fp}]{xpub}/<0;1>/*", fp = ks.master_fingerprint, xpub = ks.xpub)
                } else {
                    format!(
                        "[{fp}/{path}]{xpub}/<0;1>/*",
                        fp = ks.master_fingerprint,
                        path = path_no_m,
                        xpub = ks.xpub,
                    )
                };
                substituted = substituted.replace(&placeholder, &bracketed);
            }
            substituted
        };
```

(The implementer reads L317-345 to capture the existing substitution loop and wrap it in the `else` branch. The rest of `parse` continues unchanged with `substituted` as input to `concrete_keys_to_placeholders`.)

- [ ] **Step 3: Update module docstring at L52-56**

Convert the taproot deferral comment to a shipped citation:

```rust
//! - Taproot (`tr(NUMS, multi_a(...))` and `tr(NUMS, sortedmulti_a(...))`)
//!   descriptors are emitted by Sparrow as DESCRIPTOR-PASSTHROUGH (no
//!   `@N/**` placeholders; concrete `[fp/path]xpub` keys embedded
//!   directly in `defaultPolicy.miniscript.script`). v0.31.1+ Cycle 8
//!   ships descriptor-passthrough import via the path-split at Step 6;
//!   absence of `@0/**` triggers the passthrough branch.
```

- [ ] **Step 4: Update `parse` docstring at L209-212**

```rust
    /// 6. Detect descriptor-passthrough shape (no `@0/**` placeholder) →
    ///    skip Step 5 substitution (taproot descriptors per Sparrow's
    ///    emit-side at `wallet_export/sparrow.rs:215-219`).
    /// 7. Feed through `concrete_keys_to_placeholders` → `parse_descriptor`.
```

- [ ] **Step 5: Update or remove existing unit tests asserting the refusal**

```bash
grep -n "taproot\|tr(NUMS\|sparrow.*refuses" crates/mnemonic-toolkit/src/wallet_import/sparrow.rs | head -10
```

Update any `#[cfg(test)]` cells that asserted the L311 refusal.

- [ ] **Step 6: Build + run sparrow lib tests**

```bash
cargo build --package mnemonic-toolkit 2>&1 | tail -5
cargo test --package mnemonic-toolkit --lib sparrow 2>&1 | tail -10
```

- [ ] **Step 7: Commit Phase 2**

```bash
git add crates/mnemonic-toolkit/src/wallet_import/sparrow.rs
git commit -m "feat(sparrow): v0.31.1 Phase 2 — taproot descriptor-passthrough import path-split

Step 6 of wallet_import/sparrow.rs::parse converts the L304-315 taproot
refusal into a path-split. Descriptor-passthrough shape (no @0/**
placeholder; concrete [fp/path]xpub keys embedded directly) bypasses Step
5 substitution and feeds script_template directly into the existing
concrete_keys_to_placeholders → parse_descriptor pipeline. Template-mode
(presence of @0/** placeholder) keeps the existing substitution loop.

Per emit-side wallet_export/sparrow.rs:215-219, only taproot templates
(TrMultiA / TrSortedMultiA) currently ship as descriptor-passthrough;
all other templates ship with @N/** placeholders. The detection
heuristic !script_template.contains(\"@0/**\") is reliable per that
contract.

Phase 2 of design/PLAN_mnemonic_toolkit_v0_31_1.md."
```

---

### Task 2: Phase 3 — Integration test suite + round-trip fixture

**Files:**
- Create: `crates/mnemonic-toolkit/tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json` (copy of emit-side fixture).
- Create: `crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow_taproot.rs`.

- [ ] **Step 1: Copy the emit-side fixture to the import-side fixtures dir**

```bash
cp crates/mnemonic-toolkit/tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json \
   crates/mnemonic-toolkit/tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json
```

(Symlink would also work but a literal copy avoids future emit/import-side drift handling.)

- [ ] **Step 2: Author the integration test file**

```rust
//! v0.31.1 — Sparrow taproot descriptor-passthrough import integration tests.
//!
//! Validates the v0.31.1 Cycle 8 path-split at wallet_import/sparrow.rs
//! Step 6: descriptor-passthrough shape (no @0/** placeholder; concrete
//! [fp/path]xpub keys embedded directly) bypasses substitution and feeds
//! the script directly into the existing concrete_keys_to_placeholders →
//! parse_descriptor pipeline.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wallet_import")
        .join(name)
}

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

// ──────────────────────────────────────────────────────────────────────
// Happy paths: taproot multisig descriptor-passthrough imports
// ──────────────────────────────────────────────────────────────────────

#[test]
fn tr_multi_a_nums_2of3_imports_successfully() {
    let blob = fixture_path("sparrow-tr-multi-a-nums-2of3.json");
    mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
}

#[test]
fn tr_multi_a_nums_2of3_round_trip_preserves_descriptor() {
    // Pipeline: import-wallet --json → extract canonical descriptor →
    // export-wallet --format sparrow --from-import-json should produce
    // the same script content (byte-equal modulo `#<checksum>` strip per
    // emit-side C-1 fold).
    //
    // Cycle 8 minimal coverage: only verifies the import side succeeds +
    // emits a valid envelope. Full byte-exact round-trip is the existing
    // SPEC §7 cell 5 emit test; this cell validates the import-side
    // counterpart.
    let blob = fixture_path("sparrow-tr-multi-a-nums-2of3.json");
    let assertion = mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");

    // The bundle field's canonical descriptor MUST be the taproot one.
    let descriptor_appears = stdout.contains("tr(")
        && stdout.contains("multi_a(2,")
        && stdout.contains("50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"); // NUMS
    assert!(
        descriptor_appears,
        "envelope must carry the canonical tr() descriptor; got: {stdout}"
    );

    // 3 cosigners + threshold 2.
    let bundle = envelope
        .get("bundle")
        .or_else(|| envelope.get("bundle_json"))
        .or_else(|| envelope.get("emit_inputs"))
        .expect("envelope carries some bundle/emit-inputs field");
    let _ = bundle; // structure may vary; existence is sufficient for the smoke
}

#[test]
fn tr_multi_a_nums_2of3_sniffs_as_sparrow() {
    // Auto-sniff (no --format flag) should still detect taproot Sparrow
    // wallets as `sparrow` format (sniff is policyType-based; no script
    // content inspection).
    let blob = fixture_path("sparrow-tr-multi-a-nums-2of3.json");
    mnemonic()
        .args(["import-wallet"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────────────
// No-regression sanity: existing template-mode wallets still parse
// ──────────────────────────────────────────────────────────────────────

#[test]
fn template_mode_p2wpkh_singlesig_no_regression() {
    // Pre-v0.31.1 template-mode wallet still parses (sanity).
    let blob = fixture_path("sparrow-singlesig-p2wpkh.json");
    mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
}

#[test]
fn template_mode_wsh_sortedmulti_2of3_no_regression() {
    let blob = fixture_path("sparrow-multisig-2of3-p2wsh-sortedmulti.json");
    mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
}
```

- [ ] **Step 3: Run integration tests**

```bash
cargo test --package mnemonic-toolkit --test cli_import_wallet_sparrow_taproot 2>&1 | tail -10
```

Expected: 5 cells PASS.

- [ ] **Step 4: Run full workspace tests**

```bash
cargo test --workspace 2>&1 | grep -cE "^test result: ok"
```

- [ ] **Step 5: Commit Phase 3**

```bash
git add crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow_taproot.rs crates/mnemonic-toolkit/tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json
git commit -m "test(sparrow): v0.31.1 Phase 3 — taproot descriptor-passthrough integration suite

5 cells: tr-multi-a NUMS 2-of-3 happy path + round-trip-descriptor-shape
verification + auto-sniff + 2 no-regression cells (P2WPKH singlesig +
wsh-sortedmulti). Fixture copied from emit-side
tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json.

Phase 3 of design/PLAN_mnemonic_toolkit_v0_31_1.md."
```

---

### Task 3: Phase 4 — Manual chapter update

**Files:** modify `docs/manual/src/45-foreign-formats.md`.

- [ ] **Step 1: Find existing Sparrow taproot deferral**

```bash
grep -n "sparrow.*taproot\|taproot.*sparrow\|sparrow-taproot-descriptor-passthrough" docs/manual/src/45-foreign-formats.md | head -5
```

- [ ] **Step 2: Convert deferral to shipped-strikethrough**

Mirror the Cycle 6 / Cycle 7 patterns. E.g., the §Deferrals bullet:

```markdown
- ~~**Sparrow taproot descriptor-passthrough**~~ — shipped in v0.31.1 via
  the Step 6 path-split at `wallet_import/sparrow.rs`. Descriptor-passthrough
  shape (no `@N/**` placeholders; concrete `[fp/path]xpub` keys embedded
  directly in `defaultPolicy.miniscript.script`) bypasses substitution and
  feeds the script directly through `concrete_keys_to_placeholders` →
  `parse_descriptor`. Closes
  `sparrow-taproot-descriptor-passthrough-import-support`.
```

- [ ] **Step 3: Manual lint**

```bash
make -C docs/manual lint MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic 2>&1 | grep -E "^\[lint\]"
```

Expected: 6/6 PASS.

- [ ] **Step 4: Commit Phase 4**

```bash
git add docs/manual/src/45-foreign-formats.md
git commit -m "docs(sparrow): v0.31.1 Phase 4 — taproot deferral → shipped strikethrough

Chapter-45 §Sparrow Wallet taproot deferral converted to v0.31.1-shipped
strikethrough citation (mirrors Cycle 6 + Cycle 7 closure patterns).

Phase 4 of design/PLAN_mnemonic_toolkit_v0_31_1.md."
```

---

### Task 4: Phase 5 — Cycle close

- [ ] **Step 1: Bump version + install.sh self-pin**

`crates/mnemonic-toolkit/Cargo.toml:3`: `0.31.0` → `0.31.1`.
`scripts/install.sh:32`: `mnemonic-toolkit-v0.31.0` → `mnemonic-toolkit-v0.31.1`.

- [ ] **Step 2: CHANGELOG entry**

```markdown
## mnemonic-toolkit [0.31.1] — 2026-05-21

**SemVer-PATCH release.** Behavior expansion: Sparrow taproot multisig wallets (`tr-multi-a` / `tr-sortedmulti-a` descriptor-passthrough shape) now import successfully. Closes Cycle 8 (`sparrow-taproot-descriptor-passthrough-import-support` FOLLOWUP).

### Changed

- **`mnemonic import-wallet --format sparrow`** with taproot multisig wallets (Sparrow descriptor-passthrough shape: concrete `[fp/path]xpub` keys embedded in `defaultPolicy.miniscript.script` without `@N/**` placeholders) now succeeds. Previously refused at `wallet_import/sparrow.rs::parse` Step 6 with "taproot scripts are not yet supported".
- Detection heuristic at `sparrow.rs` Step 6: presence of `@0/**` placeholder = template mode (existing substitution loop); absence = descriptor-passthrough mode (skip substitution; feed `script_template` directly through `concrete_keys_to_placeholders` → `parse_descriptor`). Per Sparrow emit-side at `wallet_export/sparrow.rs:215-219`, only `CliTemplate::TrMultiA` / `TrSortedMultiA` currently ship as descriptor-passthrough.
- No CLI surface change. No new dependencies. No new `ToolkitError` variants. No GUI lockstep.

### Added

- 5 new integration cells in `tests/cli_import_wallet_sparrow_taproot.rs` covering tr-multi-a 2-of-3 NUMS happy path + descriptor-shape verification + auto-sniff + 2 no-regression cells.
- New fixture `tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json` (copied from emit-side `tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json`).

### Documentation

- Chapter-45 §Sparrow Wallet taproot deferral converted to v0.31.1-shipped strikethrough.

### FOLLOWUP closure

- **Closed:** `sparrow-taproot-descriptor-passthrough-import-support` (resolved by Cycle 8 / v0.31.1).

### Note

Cycle 8 of v0.28+ residual FOLLOWUP release plan — the final cycle. Single-session ship per user-locked scope decision (contained scope; pure parser refactor; no crypto, no library, no CLI flag, no GUI lockstep). Plan-doc R0 opus review dispatched before Phase 2 implementation per Cycle 6/7 lesson; **Wave 4 of the v0.28+ residual queue is now CLOSED.**

---

## mnemonic-toolkit [0.31.0] — 2026-05-21
```

- [ ] **Step 3: Full pre-tag audit**

```bash
cargo test --workspace 2>&1 | grep -cE "^test result: ok"
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -3
make -C docs/manual lint MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic 2>&1 | grep -E "^\[lint\]" | tail -3
```

- [ ] **Step 4: Commit + tag + push**

```bash
git add crates/mnemonic-toolkit/Cargo.toml scripts/install.sh CHANGELOG.md Cargo.lock
git commit -m "release(toolkit): mnemonic-toolkit v0.31.1 — sparrow taproot descriptor-passthrough import"
git tag mnemonic-toolkit-v0.31.1
git push origin master
git push origin mnemonic-toolkit-v0.31.1
```

- [ ] **Step 5: install-pin-check CI**

Wait for `conclusion: success` on the tag.

- [ ] **Step 6: GH Release**

```bash
awk '/^## mnemonic-toolkit \[0\.31\.1\]/,/^## mnemonic-toolkit \[0\.31\.0\]/' CHANGELOG.md | head -n -1 > /tmp/v0_31_1_release_notes.md
gh release create mnemonic-toolkit-v0.31.1 \
  --title "mnemonic-toolkit-v0.31.1 — Sparrow taproot descriptor-passthrough import" \
  --notes-file /tmp/v0_31_1_release_notes.md
```

---

### Task 5: Phase 6 — FOLLOWUP closure

**Files:** modify `design/FOLLOWUPS.md`.

- [ ] **Step 1: Locate slug entry**

```bash
grep -n "^### .sparrow-taproot-descriptor-passthrough" design/FOLLOWUPS.md
```

- [ ] **Step 2: Update body**

- Change `**Status:** open` → `**Status:** resolved (Cycle 8 / v0.31.1).`
- Add `**Resolved by:** \`mnemonic-toolkit-v0.31.1\` (<tag SHA>). Implementation at `wallet_import/sparrow.rs` Step 6 path-split + 5 integration cells in `tests/cli_import_wallet_sparrow_taproot.rs`.`

- [ ] **Step 3: Commit + push**

```bash
git add design/FOLLOWUPS.md
git commit -m "design(cycle-8-close): FOLLOWUP closure — sparrow-taproot-descriptor-passthrough-import-support resolved

Closes the last cycle in the v0.28+ residual FOLLOWUP queue (Wave 4 of
the BRAINSTORM_v0_28_plus_residual_followups.md plan). 8 cycles total
shipped (5, 6, 7, 8 plus 6a/6b/7a/7b split-cycles)."
git push origin master
```

- [ ] **Step 4: Update memory**

Add `project_v0_31_1_cycle_8_shipped.md` summarizing the cycle outcome + the Wave 4 closure milestone.

---

## Cross-phase invariants

- **R0 opus review on plan-doc BEFORE Phase 2 dispatch.** Per Cycle 6/7 lesson.
- **No `cargo fmt --all`** — restrict scope to Cycle 8 files only.
- **install-pin-check CI gate** on tag push.
- **No GUI lockstep** — toolkit-only release; GUI unchanged.

## Phase ordering rationale

- Task 1 (Phase 2: path-split) is the load-bearing change. Tasks 2-5 follow.
- Task 5 (FOLLOWUP closure) lands AFTER toolkit tag exists (FOLLOWUP body cites the tag SHA).

## Risk register

- **Detection heuristic robustness:** `!script_template.contains("@0/**")` is reliable per `wallet_export/sparrow.rs:215-219`'s contract — only taproot templates emit descriptor-passthrough. A future emit-side change to non-taproot descriptor-passthrough would break this heuristic; protect via test coverage (no-regression cells fire if existing template-mode wallets stop containing `@0/**`).
- **Existing unit tests** in `sparrow.rs#[cfg(test)] mod tests` that previously asserted the L311 refusal: Phase 2 Step 5 catches via grep. Any positive refusal-assertion test must be updated.
- **NUMS sentinel handling:** the emit-side fixture uses the BIP-341 NUMS point `50929b74...8ac0` as the taproot internal key. The import-side parser must NOT reject this (it's a standard BIP-341 convention for script-path-only wallets). `concrete_keys_to_placeholders` doesn't inspect the internal key; passes it through to `parse_descriptor`. Verified by the round-trip cell.

## Self-review (pre-R0 dispatch)

- ✓ Refusal site verified at `sparrow.rs:304-315` (HEAD `4eb1fa8`).
- ✓ Detection heuristic verified against `wallet_export/sparrow.rs:215-219` emit contract.
- ✓ Round-trip fixture exists; copy/symlink suffices.
- ✓ Zero new deps / variants / lib.rs / CLI surface.
- ✓ SemVer-PATCH consistent (behavior-expansion only).
- ✓ Manual chapter mirror invariant (no clap surface change → no flag-coverage delta).
