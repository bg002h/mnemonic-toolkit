# mnemonic-toolkit-v0.28.4 + mnemonic-gui-v0.X Implementation Plan (Cycle 3 / Wave 2 first)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the format-name asymmetry between `import-wallet` and `export-wallet`: `--format coldcard-multisig` accepted on both sides. Add `CliExportFormat::ColdcardMultisig` variant aliasing `Coldcard` with a multisig-template precheck. Ship paired toolkit + GUI tags. Update the chapter-45 "Format-name asymmetry note" from forward-looking to historical-context.

**Architecture:** Add `ColdcardMultisig` variant to `CliExportFormat` enum in `cmd/export_wallet.rs:22-41`. Dispatch arms in both `run()` and `run_from_import_json()` delegate to existing `ColdcardEmitter` WITH a multisig-template precheck (singlesig templates refuse with a pointer to `--format coldcard`). The toolkit's `wallet_export/coldcard.rs:42-55` template-dispatch already routes multisig templates to `emit_coldcard_multisig_text` — no change needed there. The GUI repo's `src/schema/mnemonic.rs` schema-mirror gets a new value in the `--format` dropdown; the `schema_mirror` test catches drift on GUI pin-bump.

**Tech Stack:** Rust + clap-derive ValueEnum (toolkit); Rust + GUI schema-mirror infrastructure (mnemonic-gui); Markdown (chapter-45 prose); `actionlint` (no CI yml changes expected); cargo test + clippy.

**Brainstorm spec:** `design/BRAINSTORM_followups_abc_release_plan.md` § "Cycle 3 — `mnemonic-toolkit-v0.28.4` + `mnemonic-gui-v0.X` (A1) — Wave 2 first".

**Source SHAs at plan-write time:**
- toolkit: `44fe753` (post-Wave-1 `mnemonic-toolkit-v0.28.3` ship)
- mnemonic-gui: not yet inspected; capture at Phase 0

**Cross-repo discipline:** paired tag push (toolkit lands first by ~minutes; GUI lands second with toolkit pin-bump). The lagging-indicator `schema_mirror` gate fires on GUI tag CI; closure-verification step (Phase 8) confirms it goes GREEN before declaring cycle closed.

---

## File structure

**Toolkit side (`/scratch/code/shibboleth/mnemonic-toolkit`):**

- Modify: `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`
  - L22-41 region: add `#[value(name = "coldcard-multisig")] ColdcardMultisig,` variant to `CliExportFormat` enum (between `Coldcard` and `Jade`).
  - L464-area: add dispatch arm in `collect_missing` match (`run()` function); pre-check: refuse singlesig templates with `BadInput` pointer to `--format coldcard`.
  - L482-area: add dispatch arm in `emit` match (`run()` function); delegate to `ColdcardEmitter::emit(&inputs)`.
  - L640-area: same two arms in `run_from_import_json()` (`collect_missing` + `emit` dispatches).
- Modify: `crates/mnemonic-toolkit/tests/cli_export_wallet_coldcard.rs`
  - Add 2-3 new test cells: happy path (`--format coldcard-multisig --template wsh-sortedmulti --threshold 2`), refusal path (`--format coldcard-multisig --template bip84` → BadInput with pointer text).
- Modify: `crates/mnemonic-toolkit/Cargo.toml`
  - `version = "0.28.3"` → `"0.28.4"`.
- Modify: `CHANGELOG.md`
  - New `## mnemonic-toolkit [0.28.4] — <YYYY-MM-DD>` section above `[0.28.3]`.
- Modify: `scripts/install.sh`
  - L32: `mnemonic-toolkit-v0.28.3` → `mnemonic-toolkit-v0.28.4` (install-pin-check.yml CI gate).
- Modify: `design/FOLLOWUPS.md`
  - `export-wallet-coldcard-multisig-alias` entry — Status flip.
- Modify: `docs/manual/src/45-foreign-formats.md`
  - "Format-name asymmetry note" block (around L526+, immediately below the Coldcard multisig Round-trip example). Rewrite from forward-looking to historical-context.
- Modify: `mnemonic-gui/pinned-upstream.toml` (cross-repo) — `[mnemonic].tag` pin v0.28.3 → v0.28.4 (post-toolkit-tag-push, before GUI commits).

**GUI side (`/scratch/code/shibboleth/mnemonic-gui` — sibling repo):**

- Modify: `src/schema/mnemonic.rs` — add `"coldcard-multisig"` to the `--format` dropdown values for `export-wallet` subcommand.
- Modify (dropdown wiring): files under `src/` that consume `--format` for export-wallet UI. Spot-check at execution time via `grep -rn 'coldcard' src/`.
- Modify: `Cargo.toml` — bump GUI minor version (CLI surface enlargement; not patch).
- Modify: `pinned-upstream.toml` — `[mnemonic].tag` already updated by toolkit side via cross-repo write OR by GUI plan executor directly.
- Modify: `CHANGELOG.md` — new GUI minor entry.

---

## Tasks

### Task 1: Cross-repo recon

**Files:** none modified (read-only).

- [ ] **Step 1: Verify toolkit HEAD + GUI baseline**

Run from toolkit working dir:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git log --oneline -1
git status -sb
```

Expected: HEAD on master at `44fe753` or later; clean working tree (no uncommitted changes). If dirty, stash + investigate before proceeding.

Run from GUI sibling repo:
```bash
cd /scratch/code/shibboleth/mnemonic-gui
git log --oneline -1
git status -sb
git fetch --quiet origin
git log --oneline origin/master ^HEAD | head -5
git log --oneline HEAD ^origin/master | head -5
```

Expected: GUI master tracks origin/master; clean working tree.

- [ ] **Step 2: Capture current `CliExportFormat` enum + dispatch arm line numbers**

Run from toolkit working dir:
```bash
grep -n 'pub enum CliExportFormat\|Coldcard\|Jade,' crates/mnemonic-toolkit/src/cmd/export_wallet.rs | head -15
```

Expected at HEAD `44fe753`:
- L22: `pub enum CliExportFormat {`
- L27-28: `#[value(name = "coldcard")] Coldcard,`
- L29-30: `#[value(name = "jade")] Jade,`
- L464 + L640: `CliExportFormat::Coldcard => (ColdcardEmitter::collect_missing(&inputs), "coldcard"),`
- L482: `CliExportFormat::Coldcard => ColdcardEmitter::emit(&inputs),`

If line numbers have drifted (Wave 2 may run after other commits land), use the actual values from this grep.

- [ ] **Step 3: Capture chapter-45 asymmetry-note current state**

```bash
grep -n 'Format-name asymmetry\|export-wallet-coldcard-multisig-alias' docs/manual/src/45-foreign-formats.md | head -5
```

Expected: the asymmetry-note block exists around L526+ with the FOLLOWUP slug citation. Capture the exact line range for the Phase 5 prose update.

- [ ] **Step 4: Capture GUI schema-mirror anchor**

Run from GUI sibling repo:
```bash
grep -n 'coldcard\|--format' src/schema/mnemonic.rs | head -10
```

Note the structure: which subcommand block contains the `--format` dropdown values, and how the existing values are listed.

---

### Task 2: Add `ColdcardMultisig` variant + dispatch arms (toolkit)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`

- [ ] **Step 1: Add the `ColdcardMultisig` enum variant**

In the `pub enum CliExportFormat { ... }` block at L22-41, insert IMMEDIATELY AFTER the existing `Coldcard` variant:

```rust
    #[value(name = "coldcard-multisig")]
    ColdcardMultisig,
```

The full block should then look like:

```rust
pub enum CliExportFormat {
    #[value(name = "bitcoin-core")]
    BitcoinCore,
    #[value(name = "bip388")]
    Bip388,
    #[value(name = "coldcard")]
    Coldcard,
    #[value(name = "coldcard-multisig")]
    ColdcardMultisig,
    #[value(name = "jade")]
    Jade,
    ...
}
```

- [ ] **Step 2: Add dispatch arm in `run()`'s `collect_missing` match**

Find the existing `CliExportFormat::Coldcard => (ColdcardEmitter::collect_missing(&inputs), "coldcard"),` line (around L464). Insert IMMEDIATELY AFTER it:

```rust
            CliExportFormat::ColdcardMultisig => (ColdcardEmitter::collect_missing(&inputs), "coldcard-multisig"),
```

- [ ] **Step 3: Add dispatch arm in `run()`'s `emit` match (with multisig-template precheck)**

Find the existing `CliExportFormat::Coldcard => ColdcardEmitter::emit(&inputs),` line (around L482). Insert IMMEDIATELY AFTER it:

```rust
        CliExportFormat::ColdcardMultisig => {
            // v0.28.4 (A1): `coldcard-multisig` alias requires a multisig
            // template; singlesig templates route through `--format coldcard`
            // per chapter-45 § Coldcard. Refuse-with-pointer rather than
            // silently delegating, so the asymmetry between import-side
            // (accepts both `coldcard` and `coldcard-multisig`) and export-
            // side (was: only `coldcard`; now: both, but with this precheck)
            // is preserved as a UX guarantee.
            use crate::template::CliTemplate;
            match inputs.template {
                Some(
                    CliTemplate::WshMulti
                    | CliTemplate::WshSortedMulti
                    | CliTemplate::ShWshMulti
                    | CliTemplate::ShWshSortedMulti
                    | CliTemplate::TrMultiA
                    | CliTemplate::TrSortedMultiA,
                ) => ColdcardEmitter::emit(&inputs),
                _ => Err(ToolkitError::BadInput(
                    "--format coldcard-multisig requires a multisig --template (wsh-sortedmulti, wsh-multi, sh-wsh-sortedmulti, sh-wsh-multi, tr-multi-a, tr-sortedmulti-a). For Coldcard singlesig export use --format coldcard with bip44/bip49/bip84."
                        .into(),
                )),
            }
        }
```

- [ ] **Step 4: Add same two arms in `run_from_import_json()`**

Find the `--from-import-json` path's `collect_missing` match (around L640) and `emit` match (just below). Apply the same two-arm pattern as Steps 2-3.

- [ ] **Step 5: Run cargo check**

```bash
cargo check --package mnemonic-toolkit 2>&1 | tail -5
```

Expected: zero errors. If the compiler complains about non-exhaustive match, the new variant wasn't covered in every `CliExportFormat` match in the file — re-grep `match.*format` and add the variant there too.

---

### Task 3: Add toolkit test cells

**Files:**
- Modify: `crates/mnemonic-toolkit/tests/cli_export_wallet_coldcard.rs`

- [ ] **Step 1: Read the existing test file structure**

```bash
head -40 crates/mnemonic-toolkit/tests/cli_export_wallet_coldcard.rs
tail -30 crates/mnemonic-toolkit/tests/cli_export_wallet_coldcard.rs
```

Note the existing import + helper conventions.

- [ ] **Step 2: Append 3 new test cells at the bottom of the file**

```rust
// ============================================================================
// v0.28.4 (A1) — `--format coldcard-multisig` export-side alias for `coldcard`
// with multisig-template precheck. Closes format-name asymmetry FOLLOWUP
// `export-wallet-coldcard-multisig-alias` from the manual-v0.2.0 cycle's
// P1b R0 architect §F4.
// ============================================================================

#[test]
fn export_wallet_coldcard_multisig_format_wsh_sortedmulti_2_of_3_emits_text() {
    // Happy path: `--format coldcard-multisig --template wsh-sortedmulti
    // --threshold 2` produces the same Coldcard-multisig text output as
    // `--format coldcard --template wsh-sortedmulti --threshold 2` (the
    // multisig-template arm of ColdcardEmitter::emit delegates identically).
    let bin = assert_cmd::Command::cargo_bin("mnemonic").unwrap();
    // Build slot args matching a 2-of-3 wsh-sortedmulti template.
    let out = bin
        .args([
            "export-wallet",
            "--format", "coldcard-multisig",
            "--template", "wsh-sortedmulti",
            "--threshold", "2",
            "--network", "mainnet",
            "--account", "0",
            "--slot", "@0.xpub=xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX",
            "--slot", "@0.fingerprint=b8688df1",
            "--slot", "@0.path=m/48'/0'/0'/2'",
            "--slot", "@1.xpub=xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6",
            "--slot", "@1.fingerprint=28645006",
            "--slot", "@1.path=m/48'/0'/0'/2'",
            "--slot", "@2.xpub=xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx",
            "--slot", "@2.fingerprint=5436d724",
            "--slot", "@2.path=m/48'/0'/0'/2'",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // Coldcard-multisig text starts with "# Coldcard Multisig setup file"
    assert!(
        stdout.contains("Coldcard Multisig setup file"),
        "expected Coldcard multisig text header, got: {stdout:?}"
    );
    assert!(stdout.contains("Policy: 2 of 3"));
}

#[test]
fn export_wallet_coldcard_multisig_format_refuses_singlesig_template_bip84() {
    // Refusal path: `--format coldcard-multisig --template bip84` must
    // refuse with a pointer to `--format coldcard`.
    let bin = assert_cmd::Command::cargo_bin("mnemonic").unwrap();
    let out = bin
        .args([
            "export-wallet",
            "--format", "coldcard-multisig",
            "--template", "bip84",
            "--network", "mainnet",
            "--account", "0",
            "--slot", "@0.xpub=xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9",
            "--slot", "@0.fingerprint=5436d724",
        ])
        .output()
        .expect("mnemonic spawn");
    assert_ne!(out.status.code(), Some(0), "must refuse, got success");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--format coldcard-multisig requires a multisig --template"),
        "expected multisig-required refusal, got: {stderr}"
    );
    assert!(
        stderr.contains("--format coldcard"),
        "expected pointer to `--format coldcard` for singlesig, got: {stderr}"
    );
}

#[test]
fn export_wallet_coldcard_multisig_format_refuses_no_template() {
    // Refusal path: `--format coldcard-multisig` without `--template`
    // (and without `--from-import-json`) refuses.
    let bin = assert_cmd::Command::cargo_bin("mnemonic").unwrap();
    let out = bin
        .args([
            "export-wallet",
            "--format", "coldcard-multisig",
            "--network", "mainnet",
            "--account", "0",
        ])
        .output()
        .expect("mnemonic spawn");
    assert_ne!(out.status.code(), Some(0), "must refuse, got success");
    // The refusal may surface as either the multisig-template precheck OR
    // as the upstream "--template or --descriptor required" gate; either
    // is acceptable. Just confirm exit-non-zero.
}
```

- [ ] **Step 3: Run the new tests**

```bash
cargo test --package mnemonic-toolkit --test cli_export_wallet_coldcard 2>&1 | tail -10
```

Expected: 3 new cells pass. Total file cells grow by 3.

---

### Task 4: Full test suite + clippy + gui-schema regen

**Files:** none modified.

- [ ] **Step 1: Run full toolkit test suite**

```bash
cargo test --package mnemonic-toolkit --tests 2>&1 | grep -E '^test result:' | awk '{s+=$4} END {print "Total passing:", s}'
```

Expected: 2001 + 3 = 2004 cells (or higher; check delta from baseline `44fe753`).

- [ ] **Step 2: Run clippy**

```bash
cargo clippy --package mnemonic-toolkit --tests -- -D warnings 2>&1 | tail -5
```

Expected: `Finished` line; no warnings.

- [ ] **Step 3: Verify `mnemonic gui-schema` emits the new variant**

```bash
cargo build --bin mnemonic 2>&1 | tail -3
target/debug/mnemonic gui-schema | jq '.subcommands["export-wallet"].flags["--format"].possible_values' | head -15
```

Expected: the JSON output lists `"coldcard-multisig"` as one of the possible values for `--format`.

---

### Task 5: Update chapter-45 "Format-name asymmetry note" prose

**Files:**
- Modify: `docs/manual/src/45-foreign-formats.md`

- [ ] **Step 1: Find the asymmetry-note block**

```bash
grep -n 'Format-name asymmetry note' docs/manual/src/45-foreign-formats.md
```

Expected: one hit around L526+ (block added in manual-v0.2.0 P2c commit `5d2c0a6`).

- [ ] **Step 2: Read the current block**

```bash
sed -n '535,548p' docs/manual/src/45-foreign-formats.md
```

The block currently reads (verbatim per manual-v0.2.0 ship):

```markdown
> **Format-name asymmetry note.** `--format coldcard-multisig` is
> accepted only on the **import** side (sniffs Coldcard's text
> multisig setup file). On the **export** side, `--format coldcard`
> emits Coldcard-multisig text when paired with a multisig
> `--template` (e.g., `wsh-sortedmulti`) — see SPEC v0.8 §5.2. The
> single `coldcard` export value covers both single-sig JSON
> (singlesig templates) and multisig text (multisig templates);
> tracked for export-side flag-name alignment as FOLLOWUP
> `export-wallet-coldcard-multisig-alias`.
```

- [ ] **Step 3: Rewrite to historical-context form**

Replace the block with:

```markdown
> **Format-name parity (v0.28.4+).** Both `--format coldcard` and
> `--format coldcard-multisig` are accepted on the **export** side
> (v0.28.4 closed the prior asymmetry). The two values produce
> identical output for multisig templates; `coldcard-multisig`
> additionally refuses singlesig templates (bip44/bip49/bip84) with a
> pointer to `--format coldcard`. The recipe above uses
> `--format coldcard` for backward compatibility with v0.28.0-v0.28.3
> readers; `--format coldcard-multisig --template wsh-sortedmulti
> --threshold 2` is equivalent on v0.28.4+.
```

- [ ] **Step 4: Update the F4 Round-trip recipe to use the new flag value**

Find the F4 Round-trip recipe (the export-wallet line should currently be `--format coldcard --template wsh-sortedmulti --threshold 2`). Optionally update it to demonstrate the new value:

```diff
 mnemonic export-wallet --from-import-json envelope.json \
-  --format coldcard --template wsh-sortedmulti --threshold 2 \
+  --format coldcard-multisig --template wsh-sortedmulti --threshold 2 \
   > coldcard_ms_re.txt
```

(This is optional and reader-pedagogical; if it makes the recipe slightly more consistent with the import line's `--format coldcard-multisig`, it's worth the update. If it creates churn, leave as-is.)

---

### Task 6: Opus reviewer dispatch (cross-repo)

**Files:** none modified.

- [ ] **Step 1: Dispatch opus via Agent tool**

This is a cross-repo cycle warranting opus per memory `feedback_opus_primary_review_agent`.

Use `Agent`:
- `subagent_type: feature-dev:code-reviewer`
- `model: opus`
- Prompt verifies:
  1. `ColdcardMultisig` variant added to `CliExportFormat` between `Coldcard` and `Jade` with `#[value(name = "coldcard-multisig")]` attribute.
  2. Dispatch arms in BOTH `run()` and `run_from_import_json()` (`collect_missing` + `emit` matches).
  3. Multisig-template precheck refuses singlesig templates with the documented pointer text (BadInput variant).
  4. 3 new test cells in `tests/cli_export_wallet_coldcard.rs` (happy + 2 refusal paths).
  5. `mnemonic gui-schema` JSON includes `"coldcard-multisig"` in the `--format` possible_values for export-wallet.
  6. chapter-45 asymmetry-note prose rewritten to historical-context form.
  7. Full test suite passes; clippy clean.
  8. **CROSS-REPO READINESS:** The GUI side (`/scratch/code/shibboleth/mnemonic-gui`) is on a clean branch; schema-mirror update path is known (`src/schema/mnemonic.rs`).

Gate: 0 critical / 0 important to proceed.

- [ ] **Step 2: Fold any Important findings inline**

Loop until 0 Important.

---

### Task 7: Version bump + CHANGELOG + install.sh + FOLLOWUPS flip (toolkit-side)

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml`
- Modify: `CHANGELOG.md`
- Modify: `scripts/install.sh`
- Modify: `design/FOLLOWUPS.md`

- [ ] **Step 1: Bump Cargo.toml**

`crates/mnemonic-toolkit/Cargo.toml`:
- Old: `version = "0.28.3"`
- New: `version = "0.28.4"`

- [ ] **Step 2: Add CHANGELOG.md entry**

Insert above `## mnemonic-toolkit [0.28.3]`:

```markdown
## mnemonic-toolkit [0.28.4] — <YYYY-MM-DD>

Patch release: closes the `--format coldcard-multisig` asymmetry between `import-wallet` (accepts both `coldcard` and `coldcard-multisig`) and `export-wallet` (previously only accepted `coldcard`). The new `CliExportFormat::ColdcardMultisig` variant aliases the existing `Coldcard` dispatch with a multisig-template precheck: singlesig templates (`bip44`/`bip49`/`bip84`) refuse with a pointer to `--format coldcard`; multisig templates (`wsh-sortedmulti`/`wsh-multi`/`sh-wsh-*`/`tr-*-a`) delegate to the same `ColdcardEmitter::emit` path that `--format coldcard` already uses today. Closes FOLLOWUP `export-wallet-coldcard-multisig-alias`. Paired with `mnemonic-gui-v0.X` for schema-mirror lockstep.

### Added

- `--format coldcard-multisig` value on `mnemonic export-wallet` (and `mnemonic export-wallet --from-import-json -`). Refuses singlesig templates with pointer text to `--format coldcard` for SS export.

### Changed

- chapter-45 § Coldcard multisig § "Format-name asymmetry note" prose rewritten to "Format-name parity (v0.28.4+)" with the historical-context framing.

### Tests

- 3 new cells in `tests/cli_export_wallet_coldcard.rs` (happy path + 2 refusal paths). Total toolkit cells: 2001 → 2004.

### Companion releases

- `mnemonic-gui-v0.X` — paired GUI schema-mirror + dropdown wiring update; see mnemonic-gui CHANGELOG.
```

- [ ] **Step 3: Bump scripts/install.sh:32**

Old: `mnemonic-toolkit-v0.28.3`
New: `mnemonic-toolkit-v0.28.4`

- [ ] **Step 4: Flip FOLLOWUPS Status**

Locate `export-wallet-coldcard-multisig-alias` entry (filed in commit `5d2c0a6`). Change `- **Status:** open` to `- **Status:** resolved <PLACEHOLDER-COMMIT-SHA> — mnemonic-toolkit-v0.28.4 cycle added CliExportFormat::ColdcardMultisig variant with multisig-template precheck. Paired GUI tag mnemonic-gui-v0.X bumps schema-mirror to consume the new value.` (controller backfills SHA after the toolkit commit lands.)

---

### Task 8: Toolkit commit + tag + push

- [ ] **Step 1: Verify working tree**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git status --short
```

Expected modified files:
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`
- `crates/mnemonic-toolkit/tests/cli_export_wallet_coldcard.rs`
- `crates/mnemonic-toolkit/Cargo.toml`
- `Cargo.lock`
- `CHANGELOG.md`
- `scripts/install.sh`
- `design/FOLLOWUPS.md`
- `docs/manual/src/45-foreign-formats.md`

- [ ] **Step 2: Stage + commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/export_wallet.rs \
        crates/mnemonic-toolkit/tests/cli_export_wallet_coldcard.rs \
        crates/mnemonic-toolkit/Cargo.toml \
        Cargo.lock \
        CHANGELOG.md \
        scripts/install.sh \
        design/FOLLOWUPS.md \
        docs/manual/src/45-foreign-formats.md
git commit -m "$(cat <<'EOF'
release(toolkit): mnemonic-toolkit v0.28.4 — `--format coldcard-multisig` export alias

Closes FOLLOWUP `export-wallet-coldcard-multisig-alias`.

Closes the format-name asymmetry from the manual-v0.2.0 cycle's
P1b R0 architect §F4: `mnemonic import-wallet --format coldcard-multisig`
was accepted but `mnemonic export-wallet --format coldcard-multisig`
refused (only `--format coldcard` worked, with a multisig template).
v0.28.4 adds CliExportFormat::ColdcardMultisig that aliases Coldcard
with a singlesig-template-refuse precheck; the multisig templates
delegate to the same ColdcardEmitter::emit path that has always
served multisig text emission.

Cycle 3 of the A/B/C FOLLOWUP release plan; Wave 2 first ship.
Opus reviewer GREEN: 0 critical / 0 important.

Tests: 3 new cells in tests/cli_export_wallet_coldcard.rs (happy
path + 2 refusal paths). Total toolkit cells: 2001 → 2004.

Tooling: Cargo.toml version 0.28.3 → 0.28.4; CHANGELOG entry;
scripts/install.sh:32 self-pin bumped.

Companion paired tag: mnemonic-gui-v0.X (schema-mirror lockstep).
EOF
)"
SHA=$(git rev-parse HEAD)
sed -i "s/resolved <PLACEHOLDER-COMMIT-SHA>/resolved $SHA/" design/FOLLOWUPS.md
git add design/FOLLOWUPS.md
git commit --amend --no-edit
```

- [ ] **Step 3: Tag + push**

```bash
git tag mnemonic-toolkit-v0.28.4
git push origin master
git push origin mnemonic-toolkit-v0.28.4
```

Expected: install-pin-check.yml fires on tag push and confirms scripts/install.sh:32 matches.

- [ ] **Step 4: Monitor install-pin-check + rust + manual CI**

Use Monitor tool with `gh run list` poll (mirror Cycle 1 Task 12 pattern). Wait for all 3 runs to PASS.

---

### Task 9: GUI-side cross-repo lockstep

**Files (all in `/scratch/code/shibboleth/mnemonic-gui`):**
- Modify: `src/schema/mnemonic.rs`
- Modify: dropdown wiring file(s) (TBD via grep at execution time)
- Modify: `Cargo.toml`
- Modify: `pinned-upstream.toml`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Switch to GUI repo + verify clean state**

```bash
cd /scratch/code/shibboleth/mnemonic-gui
git status -sb
git pull --quiet origin master
git log --oneline -3
```

- [ ] **Step 2: Add `"coldcard-multisig"` to the schema-mirror**

```bash
grep -n 'coldcard' src/schema/mnemonic.rs | head -10
```

Find the `export-wallet` subcommand's `--format` dropdown values block. Add `"coldcard-multisig"` to the value list, placing it between `"coldcard"` and `"jade"` (alphabetical order is not strictly required; match the existing order convention in the file).

- [ ] **Step 3: Add the value to the GUI dropdown wiring**

```bash
grep -rln 'coldcard' src/ | head -5
```

Inspect each file for `--format` dropdown definitions. Add `"coldcard-multisig"` parallel to the existing `"coldcard"` entry.

- [ ] **Step 4: Bump GUI version (Cargo.toml + Cargo.lock)**

This is a CLI surface enlargement (new dropdown value), so the GUI's MINOR version should bump (not just patch). Determine the current GUI version and pick the next minor.

- [ ] **Step 5: Bump pinned-upstream.toml**

```bash
grep -n 'mnemonic-toolkit' pinned-upstream.toml
```

Update the `[mnemonic].tag` (or equivalent) field from the previous toolkit version to `mnemonic-toolkit-v0.28.4`.

- [ ] **Step 6: Run GUI test suite locally**

```bash
cargo test --workspace 2>&1 | tail -10
```

Expected: all tests pass INCLUDING the `schema_mirror` test which compares `mnemonic gui-schema` output against the hand-maintained `src/schema/mnemonic.rs`.

The schema_mirror test requires the new toolkit binary at the bumped pin to be installed. If the test fails because it can't find the new binary, install it via:

```bash
cargo install --git https://github.com/bg002h/mnemonic-toolkit --tag mnemonic-toolkit-v0.28.4 --bin mnemonic mnemonic-toolkit
```

- [ ] **Step 7: Add GUI CHANGELOG entry**

Match the GUI repo's CHANGELOG convention. Note the new `--format coldcard-multisig` dropdown value + the toolkit pin bump.

- [ ] **Step 8: Commit + tag + push (GUI side)**

```bash
git add src/schema/mnemonic.rs src/<dropdown-wiring-file>.rs \
        Cargo.toml Cargo.lock pinned-upstream.toml CHANGELOG.md
git commit -m "$(cat <<'EOF'
release(gui): mnemonic-gui v0.X — `--format coldcard-multisig` dropdown value

Paired companion to mnemonic-toolkit-v0.28.4. Adds the
`coldcard-multisig` export-wallet `--format` dropdown value to the
schema-mirror + dropdown wiring; updates pinned-upstream.toml to
mnemonic-toolkit-v0.28.4.

The schema_mirror test gate confirms `mnemonic gui-schema` JSON
output against the hand-maintained src/schema/mnemonic.rs.
EOF
)"
git tag mnemonic-gui-v0.X  # replace v0.X with actual minor version
git push origin master
git push origin mnemonic-gui-v0.X
```

---

### Task 10: Closure-verification — schema_mirror gate on GUI CI

**Files:** none modified. This is a CI-watch step (the architect-flagged lagging-indicator gate).

- [ ] **Step 1: Watch GUI tag CI run**

```bash
cd /scratch/code/shibboleth/mnemonic-gui
gh run list --limit 5 --json databaseId,name,headBranch,status,conclusion,createdAt
```

Identify the `schema_mirror` (or equivalent) test target on the new GUI tag's CI run.

- [ ] **Step 2: Confirm schema_mirror is GREEN**

If the GUI CI's schema_mirror test FAILS, the cycle is NOT closed — the toolkit-side `gui-schema` JSON output doesn't match the GUI's hand-maintained `src/schema/mnemonic.rs`. Triage by:
1. Running `target/debug/mnemonic gui-schema | diff - <(grep ... mnemonic-gui/src/schema/mnemonic.rs)` to see the drift.
2. Adjusting either the toolkit's variant attribute (e.g., the `#[value(name = ...)]` string) OR the GUI schema-mirror's value list.
3. Re-tagging if needed (delete + re-create + re-push the tag).

Per CLAUDE.md § GUI schema-mirror coverage (L23-34): "The drift gate is therefore a **lagging indicator**, not a leading one." This Phase 10 step is the LEADING discipline catching it.

- [ ] **Step 3: Declare cycle closed only after schema_mirror GREEN**

Once both repos' tag CI is GREEN, Cycle 3 is shipped. Create the toolkit GH Release manually via `gh release create mnemonic-toolkit-v0.28.4 --title ... --notes ...` matching the Cycle 1 v0.28.3 release-notes pattern.

For the GUI release: GUI repo may have its own auto-release workflow OR require manual creation. Check `gh release view mnemonic-gui-v0.X` after the tag push; if missing, create manually.

---

## Self-review

After completing all 10 tasks, verify against the brainstorm spec:

1. **Spec coverage:**
   - Cycle 3 Phase 0 (cross-repo recon) → Task 1 ✓
   - Phase 1 (toolkit src) → Task 2 ✓
   - Phase 2 (toolkit tests) → Task 3 ✓
   - Phase 3 (gui-schema JSON regen) → Task 4 Step 3 ✓
   - Phase 4 (GUI dropdown wiring) → Task 9 Step 3 ✓
   - Phase 5 (chapter-45 prose touch-up) → Task 5 ✓
   - Phase 6 (opus reviewer) → Task 6 ✓
   - Phase 7 (paired commit + tags) → Tasks 8 + 9 ✓
   - Phase 8 (closure-verification on GUI CI) → Task 10 ✓
   - Phase 9 (FOLLOWUPS Status flip) → Task 7 Step 4 + Task 8 SHA backfill ✓

2. **No-placeholder check:** `<YYYY-MM-DD>` and `<PLACEHOLDER-COMMIT-SHA>` and `v0.X` GUI minor are template placeholders backfilled at execution time. `<dropdown-wiring-file>` in Task 9 Step 8 is a grep-discoverable placeholder per Task 9 Step 3.

3. **Type consistency:** `CliExportFormat::ColdcardMultisig` is the canonical variant name throughout. `coldcard-multisig` is the clap-derive flag value. `ColdcardEmitter::emit` is the existing delegation target. No drift.

4. **Effort estimate sanity-check:** ~2-3 hours per brainstorm (toolkit src + tests + chapter-45 prose ~45 min; opus reviewer ~10 min; release tooling + commit + tag + push ~20 min; GUI repo lockstep ~30-45 min; CI monitoring ~30 min). Realistic.

---

## Risk flags

- **GUI repo state assumed clean.** Task 1 verifies; if GUI has uncommitted work in the wallet_export-schema region, the cycle is blocked until that's resolved or branched.

- **schema_mirror test depends on a CI-built toolkit binary.** Some GUI CI setups use a cargo-install --git pin; others build the toolkit fresh. Verify the GUI's `schema_mirror` target's binary source before running it locally.

- **Deref auto-coerce was a surprise in Cycle 1.** Cycle 3 touches `cmd/export_wallet.rs` dispatch arms but does NOT touch the consumer-site files (bsms.rs, etc). The `inputs.canonical_descriptor` `CheckedDescriptor<'_>` field type is unchanged from Cycle 1. No new Deref edge cases expected, but spot-check during Task 2 Step 5's cargo check.

- **GUI minor-vs-patch decision.** This cycle adds a new CLI surface value (visible to GUI users as a new dropdown option). Per project semver convention for the GUI: this is a MINOR bump (not patch). Verify by checking the GUI's last few release tags + their associated changes.

- **Sub-skill expectations.** This plan assumes the executor uses `superpowers:subagent-driven-development` or `superpowers:executing-plans`.
