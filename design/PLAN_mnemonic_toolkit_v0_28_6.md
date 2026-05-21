# mnemonic-toolkit-v0.28.6 Implementation Plan (Cycle 2 / Wave 1 second)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close 2 test-hygiene FOLLOWUPs: `cross-format-refusal-matrix-include-coldcard-multisig` (extend matrix to cover the v0.28.4-added `ColdcardMultisig` export variant) + `coldcard-legacy-mk1-mk2-top-level-xpub-inference` (file fixture + test cells for the legacy mk1/mk2 fallback parser already implemented at commit `1304932`). Tag `mnemonic-toolkit-v0.28.6`.

**Architecture:** Pure test/fixture additions. No toolkit src changes. No GUI lockstep. Two existing test files extended: `tests/cli_export_wallet_from_import_json.rs` (matrix extension) + `tests/cli_import_wallet_coldcard.rs` (legacy fallback cells). 3 new fixtures in `tests/fixtures/wallet_import/`.

**Tech Stack:** Rust + cargo test + assert_cmd + serde_json. `make audit` to confirm regression-free.

**Brainstorm spec:** `design/BRAINSTORM_v0_28_plus_residual_followups.md` § "Cycle 2 — `mnemonic-toolkit-v0.28.6` (test-hygiene)".

**Source SHA at plan-write time:** `f9fbe6a`.

---

## File structure

- **Modify:** `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs`
  - `:592-593` region: extend `TEMPLATE_ONLY_DESTS` to include `"coldcard-multisig"`.
  - `:815` region: broaden `REFUSAL_STDERR_PATTERNS` to match `"requires a multisig --template"` substring (the v0.28.4 multisig-template precheck refusal text contains "a multisig" between "requires" and "--template").
  - `:871` region: bump cell-count assertion `32 → 40` (5 template-only dests × 8 sources).
- **Modify:** `crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard.rs`
  - Append ≥4 new test cells exercising the legacy mk1/mk2 fallback at `coldcard.rs:460-462` + `infer_bip_from_xpub_prefix` at `:471-494`.
- **Create:** `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip44-mainnet.json`
- **Create:** `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip49-mainnet.json`
- **Create:** `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip84-mainnet.json`
- **Modify:** `crates/mnemonic-toolkit/Cargo.toml` — version 0.28.5 → 0.28.6.
- **Modify:** `CHANGELOG.md` — new v0.28.6 section.
- **Modify:** `scripts/install.sh:32` — `mnemonic-toolkit-v0.28.5` → `mnemonic-toolkit-v0.28.6`.
- **Modify:** `design/FOLLOWUPS.md` — 2 Status flips.

---

## Tasks

### Task 1: Recon — read existing test conventions

**Files:** none modified.

- [ ] **Step 1: Read TEMPLATE_ONLY_DESTS + REFUSAL_STDERR_PATTERNS + cell-count assertion**

Run:
```bash
sed -n '585,600p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs
sed -n '810,820p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs
sed -n '865,875p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs
```

Expected at HEAD `f9fbe6a`:
- L592-593: `const TEMPLATE_ONLY_DESTS: &[&str] = &["coldcard", "electrum", "jade", "sparrow"];` (or similar literal).
- L815: `REFUSAL_STDERR_PATTERNS` containing `"requires --template"`.
- L871: `assert_eq!(cell_count, 32, ...)` (8 sources × 4 dests = 32).

Pin actual line numbers post-grep.

- [ ] **Step 2: Read existing coldcard test file structure**

Run:
```bash
wc -l /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard.rs
head -30 /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard.rs
```

Note: imports, helper conventions, fixture-path resolution pattern.

- [ ] **Step 3: Read the legacy fallback code paths in coldcard.rs**

Run:
```bash
sed -n '455,500p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_import/coldcard.rs
```

Confirm:
- L460-462: `if let Some(xpub_str) = obj.get("xpub").and_then(|v| v.as_str()) { ... }` legacy fallback.
- L471-494: `fn infer_bip_from_xpub_prefix` with SLIP-132 prefix mapping.

- [ ] **Step 4: Derive the 3 legacy-fixture xpubs**

For abandon-test-vector phrase (`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`) at the 3 BIP paths:

- BIP-44 mainnet (m/44'/0'/0'): `xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz`
- BIP-49 mainnet (m/49'/0'/0'): `ypub6XAGdCAESS9Lsh1nUhcvTtVycHt3VLnZk3yFqHpEi6tjMjkASCfdmTRGQpdQVCxKtuxiB6cTtKB1ESHHACtRdfV7vRyhVgrM6tWP9YGZsxA`
- BIP-84 mainnet (m/84'/0'/0'): `zpub6qTBkagqERLNDQHfQuvgUYUyW3qNUKNTQqf2N2agYzpb2nVwk2nu2Ko5JeMs2czwCUmkKUUMu33Pp3M44yfTjCXrEzU4Pp7ufuwArvm4G3T`

(These are derived from abandon-test-vector; verify at cycle-start by running `mnemonic bundle` against each template OR by `mnemonic derive` if available. The plan-doc's xpub values are place-holding; implementer MUST verify by deriving from the actual phrase at cycle-start. Discrepancy = file the correct xpub.)

The fingerprint is `5436d724` (matches v0.28.4 transcript captures).

---

### Task 2: Extend cross-format refusal matrix

**Files:**
- Modify: `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs`

- [ ] **Step 1: Bump TEMPLATE_ONLY_DESTS**

In `tests/cli_export_wallet_from_import_json.rs:592-593` (verify line at Task 1):

Old:
```rust
const TEMPLATE_ONLY_DESTS: &[&str] = &["coldcard", "electrum", "jade", "sparrow"];
```

New:
```rust
const TEMPLATE_ONLY_DESTS: &[&str] = &["coldcard", "coldcard-multisig", "electrum", "jade", "sparrow"];
```

(Alphabetical-after-coldcard order.)

- [ ] **Step 2: Broaden REFUSAL_STDERR_PATTERNS to match coldcard-multisig refusal text**

Find `REFUSAL_STDERR_PATTERNS` at L815 (verify post-Task 1). The current pattern array contains `"requires --template"` which DOES NOT match the v0.28.4 added arm's text `"--format coldcard-multisig requires a multisig --template"` (the word "a multisig" intervenes).

Add an additional pattern:

```rust
const REFUSAL_STDERR_PATTERNS: &[&str] = &[
    "requires --template",
    "requires a multisig --template",
    // ... other existing patterns
];
```

Verify the actual surrounding context (the array may have other patterns; add the new line without disturbing them).

- [ ] **Step 3: Bump cell-count assertion**

Find the cell-count assertion at L871:

Old:
```rust
assert_eq!(cell_count, 32, "expected 8 sources × 4 template-only dests = 32 refusal cells; got {cell_count}");
```

New:
```rust
assert_eq!(cell_count, 40, "expected 8 sources × 5 template-only dests = 40 refusal cells; got {cell_count}");
```

(8 × 5 = 40 post-extension.)

- [ ] **Step 4: Run the refusal-matrix test to verify it passes**

Run:
```bash
cargo test --package mnemonic-toolkit --test cli_export_wallet_from_import_json p11c_refusal_matrix 2>&1 | tail -10
```

(The test function name may differ; grep `grep -n 'refusal_matrix\|template_only' tests/cli_export_wallet_from_import_json.rs` to locate it.)

Expected: PASS. If it fails:
- "expected 40, got X" — investigate which (source, dest) pair is missing from the refusal matrix; may need to extend the matrix-construction logic, not just the cell count.
- substring assertion fails — broaden `REFUSAL_STDERR_PATTERNS` further OR tighten the toolkit's refusal text (out-of-scope for this cycle; file FOLLOWUP if needed).

---

### Task 3: Author legacy fixtures (3 files)

**Files:**
- Create: `tests/fixtures/wallet_import/coldcard-mk1-legacy-bip44-mainnet.json`
- Create: `tests/fixtures/wallet_import/coldcard-mk1-legacy-bip49-mainnet.json`
- Create: `tests/fixtures/wallet_import/coldcard-mk1-legacy-bip84-mainnet.json`

- [ ] **Step 1: Derive abandon-test-vector xpubs at the 3 BIP paths**

The legacy mk1/mk2 wallet.json shape (per FOLLOWUP body) is "top-level `xpub` field without per-path blocks... plus `xfp` at root". The minimal shape required by `coldcard.rs:460-462` is just `{"xpub": "<slip132-prefixed-xpub>"}`. Other fields like `xfp` and `chain` may be required by downstream consumers (script-type derivation).

Use the toolkit binary to derive the canonical xpubs from the abandon-test-vector:

```bash
# BIP-44 mainnet (m/44'/0'/0')
/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic bundle --network mainnet --template bip44 \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --json 2>/dev/null | jq -r '.[0].bundle.descriptor'

# BIP-49 mainnet (m/49'/0'/0')
/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic bundle --network mainnet --template bip49 \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --json 2>/dev/null | jq -r '.[0].bundle.descriptor'

# BIP-84 mainnet (m/84'/0'/0')
/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic bundle --network mainnet --template bip84 \
  --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --json 2>/dev/null | jq -r '.[0].bundle.descriptor'
```

Each command outputs the canonical descriptor (e.g., `pkh([5436d724/44'/0'/0']xpub.../<0;1>/*)`). Extract the xpub. Then convert to the SLIP-132 prefix (xpub→BIP-44 stays xpub; for BIP-49 the xpub needs y-prefix conversion; for BIP-84 the xpub needs z-prefix conversion).

**Easier alternative:** install `slip132` crate or use a one-off converter script. Or — pragmatic — use existing test vectors from prior cycles' fixtures (`tests/fixtures/wallet_import/coldcard-singlesig-bip{44,49,84}-mainnet.json` carry the modern-shape xpubs; extract them).

The implementer MUST verify the actual SLIP-132-prefixed values at cycle-start; the plan-doc's xpub strings (in Task 1 Step 4) are illustrative and may not be canonical.

- [ ] **Step 2: Create the BIP-44 legacy fixture**

Create `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip44-mainnet.json` with:

```json
{
  "xpub": "xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz",
  "xfp": "5436D724",
  "chain": "BTC"
}
```

(Verify the xpub against the canonical BIP-44 derivation from abandon-test-vector at cycle-start. The `xfp` and `chain` are conservative — older mk1/mk2 firmware emitted these.)

- [ ] **Step 3: Create the BIP-49 legacy fixture**

Create `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip49-mainnet.json` with:

```json
{
  "xpub": "ypub6XAGdCAESS9Lsh1nUhcvTtVycHt3VLnZk3yFqHpEi6tjMjkASCfdmTRGQpdQVCxKtuxiB6cTtKB1ESHHACtRdfV7vRyhVgrM6tWP9YGZsxA",
  "xfp": "5436D724",
  "chain": "BTC"
}
```

(BIP-49 → `ypub` SLIP-132 prefix. Verify at cycle-start.)

- [ ] **Step 4: Create the BIP-84 legacy fixture**

Create `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip84-mainnet.json` with:

```json
{
  "xpub": "zpub6qTBkagqERLNDQHfQuvgUYUyW3qNUKNTQqf2N2agYzpb2nVwk2nu2Ko5JeMs2czwCUmkKUUMu33Pp3M44yfTjCXrEzU4Pp7ufuwArvm4G3T",
  "xfp": "5436D724",
  "chain": "BTC"
}
```

(BIP-84 → `zpub` SLIP-132 prefix. Verify at cycle-start.)

---

### Task 4: Add legacy-fallback test cells

**Files:**
- Modify: `crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard.rs`

- [ ] **Step 1: Append 4 new test cells at the bottom of the file**

```rust
// ============================================================================
// v0.28.6 — Legacy mk1/mk2 Coldcard wallet.json fallback (parser at
// wallet_import/coldcard.rs:460-462 with SLIP-132 prefix inference at
// :471-494). Parser implementation landed in commit 1304932 (v0.28.0
// P3-v2 cycle); this Cycle adds fixture + test coverage per FOLLOWUP
// `coldcard-legacy-mk1-mk2-top-level-xpub-inference`.
// ============================================================================

use std::path::PathBuf;

fn legacy_fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

#[test]
fn coldcard_legacy_mk1_xpub_prefix_infers_bip44() {
    // Legacy mk1/mk2 wallet.json carries top-level `xpub` + `xfp` with
    // no per-BIP envelope blocks. Parser falls back via
    // `infer_bip_from_xpub_prefix`: xpub/tpub → BIP-44.
    let fixture = legacy_fixture("coldcard-mk1-legacy-bip44-mainnet.json");
    let out = assert_cmd::Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format", "coldcard",
            "--blob", fixture.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("envelope JSON");
    // Verify the parsed descriptor is BIP-44 (`pkh(...)`).
    let descriptor = envelope[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor present");
    assert!(
        descriptor.starts_with("pkh("),
        "expected pkh() descriptor (BIP-44 from xpub prefix), got: {descriptor:?}"
    );
}

#[test]
fn coldcard_legacy_mk1_ypub_prefix_infers_bip49() {
    let fixture = legacy_fixture("coldcard-mk1-legacy-bip49-mainnet.json");
    let out = assert_cmd::Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format", "coldcard",
            "--blob", fixture.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("envelope JSON");
    let descriptor = envelope[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor present");
    assert!(
        descriptor.starts_with("sh(wpkh("),
        "expected sh(wpkh(...)) descriptor (BIP-49 from ypub prefix), got: {descriptor:?}"
    );
}

#[test]
fn coldcard_legacy_mk1_zpub_prefix_infers_bip84() {
    let fixture = legacy_fixture("coldcard-mk1-legacy-bip84-mainnet.json");
    let out = assert_cmd::Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format", "coldcard",
            "--blob", fixture.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("envelope JSON");
    let descriptor = envelope[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor present");
    assert!(
        descriptor.starts_with("wpkh("),
        "expected wpkh(...) descriptor (BIP-84 from zpub prefix), got: {descriptor:?}"
    );
}

#[test]
fn coldcard_legacy_mk1_unrecognized_prefix_refuses() {
    // Unrecognized SLIP-132 prefix → `infer_bip_from_xpub_prefix` returns
    // `Err(ImportWalletParse(... "unrecognized SLIP-132 prefix ..."))` per
    // coldcard.rs:490-493.
    use std::io::Write;
    let tmpdir = tempfile::tempdir().expect("tempdir");
    let path = tmpdir.path().join("coldcard-legacy-bad-prefix.json");
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(br#"{"xpub": "bogusprefix_not_a_slip132_xpub_at_all", "xfp": "5436D724", "chain": "BTC"}"#).unwrap();
    drop(f);

    let out = assert_cmd::Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format", "coldcard",
            "--blob", path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("mnemonic spawn");
    assert_ne!(out.status.code(), Some(0), "must refuse, got success");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unrecognized SLIP-132 prefix") ||
        stderr.contains("legacy top-level"),
        "expected SLIP-132 prefix refusal, got: {stderr}"
    );
}
```

- [ ] **Step 2: Run the new tests**

Run:
```bash
cargo test --package mnemonic-toolkit --test cli_import_wallet_coldcard coldcard_legacy 2>&1 | tail -15
```

Expected: 4 passed; 0 failed. If any fail:
- BIP-44 descriptor doesn't start with `pkh(` → check what the canonical-descriptor-builder emits for BIP-44 (may be `pkh(@0/**)` or something with the envelope wrapper).
- Fixture xpubs are wrong → re-derive from abandon-vector at cycle-start.
- Unrecognized-prefix refusal text differs → adjust `assert!(stderr.contains(...))` substring.

---

### Task 5: Full test suite + clippy

**Files:** none modified.

- [ ] **Step 1: Run full toolkit test suite**

Run:
```bash
cargo test --package mnemonic-toolkit --tests 2>&1 | grep -E '^test result:' | awk '{s+=$4} END {print "Total passing:", s}'
```

Expected: 2004 + 4 (legacy) + 0 (matrix bumps just change assertion values, not count) = ~2008 cells.

- [ ] **Step 2: Run `make audit`**

Run:
```bash
make -C /scratch/code/shibboleth/mnemonic-toolkit/docs/manual audit \
  MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic \
  MD_BIN=/home/bcg/.cargo/bin/md MS_BIN=ms MK_BIN=mk \
  FIXTURES_DIR=/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/fixtures/wallet_import \
  2>&1 | tail -5
```

Expected: `[lint] OK` + `[verify-examples] OK (14 transcripts pass)`. The new legacy fixtures don't affect transcripts (they're test-only).

- [ ] **Step 3: Run clippy**

Run:
```bash
cargo clippy --package mnemonic-toolkit --tests -- -D warnings 2>&1 | tail -3
```

Expected: `Finished` line; no warnings.

---

### Task 6: Sonnet reviewer fold-verify

**Files:** none modified.

- [ ] **Step 1: Dispatch sonnet via Agent tool**

Use `Agent`:
- `subagent_type: feature-dev:code-reviewer`
- `model: sonnet`
- Prompt verifies:
  1. `TEMPLATE_ONLY_DESTS` includes `"coldcard-multisig"` in expected position.
  2. `REFUSAL_STDERR_PATTERNS` broadened to match `"requires a multisig --template"`.
  3. Cell-count assertion bumped 32 → 40.
  4. 3 legacy fixtures present with valid SLIP-132 xpubs derived from abandon-test-vector.
  5. 4 new test cells (`coldcard_legacy_mk1_*`) all pass; cover BIP-44/49/84 + 1 refusal.
  6. Full test suite passes (~2008 cells); clippy clean.

Gate: 0 critical / 0 important.

- [ ] **Step 2: Fold any Important findings inline**

---

### Task 7: Release tooling + FOLLOWUPS Status flips

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml`
- Modify: `CHANGELOG.md`
- Modify: `scripts/install.sh`
- Modify: `design/FOLLOWUPS.md`

- [ ] **Step 1: Bump Cargo.toml version**

Old: `version = "0.28.5"`
New: `version = "0.28.6"`

- [ ] **Step 2: Add CHANGELOG entry**

Insert above `## mnemonic-toolkit [0.28.5]`:

```markdown
## mnemonic-toolkit [0.28.6] — <YYYY-MM-DD>

Patch release: 2 test-hygiene FOLLOWUPs from the post-A/B/C residual backlog.

### Tests

- **`cross-format-refusal-matrix-include-coldcard-multisig`** — Extend the `tests/cli_export_wallet_from_import_json.rs` refusal-matrix coverage to include the v0.28.4-added `--format coldcard-multisig` export variant: `TEMPLATE_ONLY_DESTS` grows to 5 entries; `REFUSAL_STDERR_PATTERNS` broadened to match the `"requires a multisig --template"` refusal substring (the v0.28.4 multisig-template precheck text); cell-count assertion bumped 32 → 40 (8 sources × 5 dests). Closes the FOLLOWUP filed in v0.28.4 cycle commit `826efbc`.

- **`coldcard-legacy-mk1-mk2-top-level-xpub-inference`** — Legacy mk1/mk2 Coldcard `wallet.json` fallback parser (already implemented in commit `1304932` from v0.28.0 P3-v2 cycle) now has fixture + test coverage. 3 new fixtures in `tests/fixtures/wallet_import/coldcard-mk1-legacy-bip{44,49,84}-mainnet.json` carry abandon-test-vector xpubs in SLIP-132 prefix forms (`xpub`/`ypub`/`zpub`); 4 new test cells in `tests/cli_import_wallet_coldcard.rs` exercise the `infer_bip_from_xpub_prefix` SLIP-132 mapping (BIP-44/49/84 happy paths + 1 unrecognized-prefix refusal). Total toolkit cells: 2004 → ~2008.

### Cycle context

Cycle 2 of v0.28+ residual FOLLOWUP release plan (Wave 1 second ship). See `design/BRAINSTORM_v0_28_plus_residual_followups.md`. No CLI surface change; no toolkit src changes; no GUI lockstep.
```

Replace `<YYYY-MM-DD>` with today's date.

- [ ] **Step 3: Bump scripts/install.sh:32**

Old: `mnemonic-toolkit-v0.28.5`
New: `mnemonic-toolkit-v0.28.6`

- [ ] **Step 4: Flip FOLLOWUPS Status × 2**

Locate both FOLLOWUPS entries:
- `cross-format-refusal-matrix-include-coldcard-multisig`
- `coldcard-legacy-mk1-mk2-top-level-xpub-inference`

For each, change Status from `open` (or `resolved <PLACEHOLDER>` if amended in-session) to:

```markdown
- **Status:** `resolved <PLACEHOLDER-COMMIT-SHA>` — mnemonic-toolkit-v0.28.6 cycle closed via <description>.
```

(Use sed-then-amend pattern from Cycle 1; backfill SHA in Task 8.)

For `cross-format-refusal-matrix-include-coldcard-multisig`:

```markdown
- **Status:** `resolved <PLACEHOLDER-COMMIT-SHA>` — mnemonic-toolkit-v0.28.6 cycle extended TEMPLATE_ONLY_DESTS to include "coldcard-multisig", broadened REFUSAL_STDERR_PATTERNS to match the v0.28.4 arm's refusal text "requires a multisig --template", and bumped the cell-count assertion 32 → 40.
```

For `coldcard-legacy-mk1-mk2-top-level-xpub-inference`:

```markdown
- **Status:** `resolved <PLACEHOLDER-COMMIT-SHA>` — mnemonic-toolkit-v0.28.6 cycle added 3 legacy fixtures (coldcard-mk1-legacy-bip{44,49,84}-mainnet.json) + 4 test cells in tests/cli_import_wallet_coldcard.rs covering the SLIP-132 prefix inference (parser implementation landed in commit 1304932 / v0.28.0 P3-v2 cycle).
```

- [ ] **Step 5: Rebuild + verify binary version**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo build --bin mnemonic 2>&1 | tail -3
target/debug/mnemonic --version
```

Expected: `mnemonic 0.28.6`.

---

### Task 8: Commit + tag + push

- [ ] **Step 1: Verify working tree**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git status --short
```

Expected modified files:
- `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs`
- `crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard.rs`
- `crates/mnemonic-toolkit/Cargo.toml`
- `Cargo.lock`
- `CHANGELOG.md`
- `scripts/install.sh`
- `design/FOLLOWUPS.md`

Expected new files:
- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip44-mainnet.json`
- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip49-mainnet.json`
- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip84-mainnet.json`

- [ ] **Step 2: Stage explicit paths**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git add crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs \
        crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard.rs \
        crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip44-mainnet.json \
        crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip49-mainnet.json \
        crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-mk1-legacy-bip84-mainnet.json \
        crates/mnemonic-toolkit/Cargo.toml \
        Cargo.lock \
        CHANGELOG.md \
        scripts/install.sh \
        design/FOLLOWUPS.md
git diff --cached --stat
```

- [ ] **Step 3: Commit**

```bash
git commit -m "$(cat <<'EOF'
release(toolkit): mnemonic-toolkit v0.28.6 — test-hygiene (refusal matrix + coldcard-legacy fixtures)

Closes 2 v0.28+ FOLLOWUPs:
- cross-format-refusal-matrix-include-coldcard-multisig — extend
  TEMPLATE_ONLY_DESTS in cli_export_wallet_from_import_json.rs to
  include "coldcard-multisig" (5 entries; was 4); broaden
  REFUSAL_STDERR_PATTERNS to match "requires a multisig --template"
  substring (v0.28.4 arm's refusal text); bump cell-count assertion
  32 → 40 (8 sources × 5 dests).
- coldcard-legacy-mk1-mk2-top-level-xpub-inference — parser already
  implemented at commit 1304932 (v0.28.0 P3-v2 cycle); this cycle
  adds 3 fixtures + 4 test cells covering BIP-44/49/84 SLIP-132
  prefix inference + 1 unrecognized-prefix refusal.

Cycle 2 of v0.28+ residual FOLLOWUP release plan (Wave 1 second
ship). See design/BRAINSTORM_v0_28_plus_residual_followups.md.
Sonnet reviewer GREEN: 0 critical / 0 important.

Tests: total toolkit cells 2004 → ~2008. F9 cells + checked_descriptor
cells unchanged.

Tooling: Cargo.toml version 0.28.5 → 0.28.6; CHANGELOG entry;
scripts/install.sh:32 self-pin bumped.

No CLI surface change; no toolkit src changes; no GUI lockstep.
EOF
)"
```

- [ ] **Step 4: Backfill SHA + amend**

```bash
SHA=$(git rev-parse HEAD)
sed -i "s/resolved <PLACEHOLDER-COMMIT-SHA>/resolved $SHA/g" design/FOLLOWUPS.md
git add design/FOLLOWUPS.md
git commit --amend --no-edit
```

- [ ] **Step 5: Tag + push**

```bash
git tag mnemonic-toolkit-v0.28.6
git push origin master
git push origin mnemonic-toolkit-v0.28.6
```

---

### Task 9: Monitor CI + GH Release

**Files:** none modified.

- [ ] **Step 1: Monitor**

Use Monitor poll script (A/B/C pattern). Expected:
- `install-pin-check` on tag: PASS.
- `rust` on master: PASS (new test cells exercise the matrix + legacy fallback).
- `manual` on master: SKIPPED (no docs/manual changes) OR PASS if triggered.

- [ ] **Step 2: Create GH Release**

```bash
gh release create mnemonic-toolkit-v0.28.6 \
  --title 'mnemonic-toolkit v0.28.6 — test-hygiene (refusal matrix + coldcard-legacy fixtures)' \
  --notes "$(cat <<'EOF'
Patch release: 2 test-hygiene FOLLOWUPs from the post-A/B/C residual backlog.

### Tests

- **\`cross-format-refusal-matrix-include-coldcard-multisig\`** — Extend the \`tests/cli_export_wallet_from_import_json.rs\` refusal-matrix coverage to include the v0.28.4-added \`--format coldcard-multisig\` export variant. Cell-count grew 32 → 40 (8 sources × 5 dests).

- **\`coldcard-legacy-mk1-mk2-top-level-xpub-inference\`** — Legacy mk1/mk2 Coldcard \`wallet.json\` fallback parser (already implemented in commit \`1304932\` from v0.28.0 P3-v2 cycle) now has fixture + test coverage. 3 new fixtures + 4 test cells exercise the SLIP-132 prefix inference (\`xpub\`/\`ypub\`/\`zpub\` → BIP-44/49/84) + 1 unrecognized-prefix refusal.

Total toolkit cells: 2004 → ~2008.

### Cycle context

Cycle 2 of v0.28+ residual FOLLOWUP release plan (Wave 1 second ship). See [\`design/BRAINSTORM_v0_28_plus_residual_followups.md\`](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/BRAINSTORM_v0_28_plus_residual_followups.md). No CLI surface change; no toolkit src changes; no GUI lockstep.
EOF
)"
```

---

## Self-review

After completing all 9 tasks:

1. **Spec coverage:**
   - Cycle 2 Phase 0 (recon) → Task 1 ✓
   - Phase 1 (cross-format matrix extension) → Task 2 ✓
   - Phase 2 (coldcard legacy fixtures + test cells) → Tasks 3, 4 ✓
   - Phase 3 (cargo test + clippy + `make audit`) → Task 5 ✓
   - Phase 4 (sonnet reviewer) → Task 6 ✓
   - Phase 5 (commit + tag + push + GH Release) → Tasks 7, 8, 9 ✓
   - FOLLOWUPS Status flips × 2 → Task 7 Step 4 ✓

2. **No-placeholder check:** xpub values in Task 1 Step 4 + Tasks 3 Steps 2-4 are illustrative; implementer MUST derive from abandon-test-vector at cycle-start. `<YYYY-MM-DD>` + `<PLACEHOLDER-COMMIT-SHA>` backfilled at commit time. No TBD/TODO.

3. **Type consistency:** N/A (test cells + JSON fixtures only).

4. **Effort estimate sanity-check:** ~half-day per brainstorm. Task 1 (~15 min); Task 2 (~30 min); Task 3 (~1 hour incl. xpub derivation); Task 4 (~1 hour); Task 5 (~15 min); Task 6 (~15 min); Tasks 7-9 (~45 min). Total ~4 hours. Realistic.

---

## Risk flags

- **Xpub derivation correctness** — Task 3 Steps 2-4 carry illustrative xpubs that MUST be verified at cycle-start by re-deriving from abandon-test-vector via the toolkit. If wrong, test cells in Task 4 will fail with "descriptor doesn't start with pkh(/sh(wpkh(/wpkh(" — fix the fixture xpub.

- **Legacy fixture shape may need additional fields** — Task 3 fixtures contain minimal `xpub` + `xfp` + `chain`. If the parser also requires e.g. `account` or `derivation_path`, the test cells will fail with a parse error. Mitigation: read the FULL `coldcard.rs::parse_coldcard_singlesig` function at Task 1 Step 3 to enumerate required fields BEFORE authoring fixtures.

- **REFUSAL_STDERR_PATTERNS substring conflict** — Task 2 Step 2 adds `"requires a multisig --template"` to the pattern array. Verify the existing patterns at L815 don't already cover this (might be redundant). Also verify the matrix test logic at L871 expects EXACTLY ONE pattern per refusal, not multiple — if it expects exactly one, the new pattern overlaps with the existing `"requires --template"` for some arms (singlesig refusals) but not others (the new multisig refusal).

- **Sub-skill expectations.** This plan assumes the executor uses `superpowers:subagent-driven-development` or `superpowers:executing-plans`.
