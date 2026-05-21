# mnemonic-toolkit-v0.28.7 Implementation Plan (Cycle 3 / Wave 2 hardening)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close 4 hardening FOLLOWUPs surfaced during the v0.28.x cycle: `bsms-import-taproot-refusal-parity` + `green-emitter-multisig-refusal-template-only` + `wallet-import-format-mismatch-matrix-completion` (Option B narrow set per P0 lock) + `wallet-import-taproot-internal-key` (Framing B envelope-gate-only per P0 lock). Tag `mnemonic-toolkit-v0.28.7`.

**Architecture:** Toolkit-only patch. 4 disjoint defect surfaces; each closes in its own phase with isolated source edits + test cells. No GUI lockstep (no CLI surface or wire-shape change). No new ToolkitError variant *unless* Slug 1 takes the `BsmsTaprootImportRefused` option (locked α — yes); Slug 2 adds `WalletScriptType::is_multisig()` method; Slugs 3+4 add zero new variants.

**Tech Stack:** Rust + cargo test + assert_cmd + serde_json. `make audit` to confirm regression-free.

**Brainstorm spec:** `design/BRAINSTORM_v0_28_plus_residual_followups.md` § "Cycle 3 — `mnemonic-toolkit-v0.28.7` (hardening)". P0 recon dossier: `design/cycle-3-p0-recon.md`.

**Source SHA at plan-write time:** `885f522` (v0.28.6 close).

**P0 STRICT-GATE locks (per architect M2 fold):**
- Slug 1 ambiguity resolved: **Option α — new `BsmsTaprootImportRefused` variant** (no `script_type` field; import-side parser has no `WalletScriptType` in scope at parse time).
- Slug 3 scope locked: **Option B — narrow to original FOLLOWUPS residual set** (BSMS / BitcoinCore / ColdcardMultisig arms only; file new FOLLOWUP `wallet-import-format-mismatch-matrix-completion-discovered-gaps` for Coldcard/Sparrow/Specter/Electrum residuals discovered during P0).
- Slug 4 framing resolved: **Framing B — single envelope-gate refusal at `cmd/export_wallet.rs:650`** (`taproot_internal_key: None,` site); drop per-exporter fan-out from plan body.
- Slug 4 fix variant: **Fix-α — refusal-only** (no wire-shape change; Fix-β envelope-field addition stays open for v0.29+).

---

## File structure

### Source files modified

- `crates/mnemonic-toolkit/src/error.rs` — add `BsmsTaprootImportRefused` variant + Display arm + exit_code arm + kind arm (Slug 1).
- `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` — add Tr(_) short-circuit at top of `BsmsParser::parse` (Slug 1); broaden `extract_threshold` regex at L479 to NOT match `sortedmulti_a(` (Slug 1).
- `crates/mnemonic-toolkit/src/wallet_export/green.rs` — refactor refusal-guard L30-44 (Slug 2).
- `crates/mnemonic-toolkit/src/wallet_export/mod.rs` — add `WalletScriptType::is_multisig()` method (Slug 2).
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — extend BSMS / BitcoinCore / ColdcardMultisig dispatch arms to refuse the 17 missing sniff outcomes (Slug 3 Option B).
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` — add taproot-envelope refusal before `EmitInputs` construction (Slug 4 Fix-α).

### Test files modified

- `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs` — rename existing cell `bsms_2line_tr_nums_current_behavior_no_refusal` (L968) → `bsms_2line_tr_nums_refused`; flip exit-0 assertion to exit-2 (Slug 1). Add new cell for `sortedmulti_a` extract_threshold regex side-channel (Slug 1).
- `crates/mnemonic-toolkit/tests/cli_export_wallet_green.rs` — new cell for descriptor-mode multisig refusal via `--from-import-json` (Slug 2).
- `crates/mnemonic-toolkit/tests/cli_import_wallet_format_mismatch_matrix.rs` (NEW) — bundle the 17 cell additions in one matrix file rather than scattering across 3 per-format files (Slug 3 Option B).
- `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs` — new cell for taproot-envelope refusal (Slug 4).

### Release tooling

- `crates/mnemonic-toolkit/Cargo.toml` — version 0.28.6 → 0.28.7.
- `CHANGELOG.md` — new v0.28.7 section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.28.6` → `mnemonic-toolkit-v0.28.7`.
- `design/FOLLOWUPS.md` — 4 Status flips + 1 NEW FOLLOWUP filing (Slug 3 discovered gaps).

---

## Tasks

### Task 1: Phase 1 — Slug 1 (`bsms-import-taproot-refusal-parity`)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/error.rs`
- Modify: `crates/mnemonic-toolkit/src/wallet_import/bsms.rs`
- Modify: `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs`

- [ ] **Step 1: Add `BsmsTaprootImportRefused` variant to `enum ToolkitError`**

Find the variant block in `error.rs` (alphabetical-by-variant-name per CLAUDE.md convention for NEW variants). Locate the `BsmsTaprootRefused` variant at L279 (per recon dossier). Alphabetically `BsmsTaprootImportRefused < BsmsTaprootRefused` (`I` < `R`), so insert it **BEFORE** `BsmsTaprootRefused` (R0-I3 fold):

```rust
/// Import-side parity of `BsmsTaprootRefused`. v0.28.7+: refused at
/// `BsmsParser::parse` entry. No `script_type` field — the import parser
/// has no `WalletScriptType` in scope at parse time (see
/// `design/cycle-3-p0-recon.md` Slug 1 lock α).
BsmsTaprootImportRefused,
```

- [ ] **Step 2: Add Display arm**

Locate the `Display` impl block. Find `BsmsTaprootRefused { script_type } =>` arm. Insert new arm **BEFORE** it (alphabetical, R0-I3 fold):

```rust
ToolkitError::BsmsTaprootImportRefused => write!(
    f,
    "--format bsms does not support taproot import; BIP-129 §1 prerequisites \
     do not yet include BIP-386. Real import support is tracked at FOLLOWUP \
     `bsms-import-taproot-refusal-parity` (resolved v0.28.7). Use \
     --format bitcoin-core (Core-importable) or --format sparrow \
     (Sparrow JSON, taproot-capable) for taproot watch-only setup.",
),
```

- [ ] **Step 3: Add `exit_code` arm**

Locate the `exit_code` `match self {` block. Find `BsmsTaprootRefused {..} =>` arm. Insert new arm **BEFORE** it (alphabetical, R0-I3 fold; exit 2 to match emit-side):

```rust
ToolkitError::BsmsTaprootImportRefused => 2,
```

- [ ] **Step 4: Add `kind` arm**

Locate the `kind` `match self {` block. Find `BsmsTaprootRefused {..} =>` arm. Insert new arm **BEFORE** it (alphabetical, R0-I3 fold):

```rust
ToolkitError::BsmsTaprootImportRefused => "BsmsTaprootImportRefused",
```

- [ ] **Step 5: Add Tr(_) short-circuit at top of `BsmsParser::parse`**

In `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:70`, the existing `parse` body begins after the function signature. Find the EARLIEST point where the descriptor is parsed into `MsDescriptor<DescriptorPublicKey>` — likely after line-format validation. Insert BEFORE that descriptor-parse:

```rust
// v0.28.7 — Slug 1: refuse taproot at parse entry, mirroring emit-side
// BsmsTaprootRefused. BIP-129 §1 prerequisites do not yet include BIP-386.
// Detection mode: cheap textual sniff on `tr(` substring in the descriptor
// block content. Authoritative parse-side detection happens later in this
// fn via `MsDescriptor::Tr(_)`, but we want to refuse before doing the
// (expensive) full descriptor-parse + first-address verify.
if blob_descriptor_text.contains("tr(") {
    return Err(ToolkitError::BsmsTaprootImportRefused);
}
```

The exact variable name (`blob_descriptor_text`) needs to be substituted with the actual local variable holding the descriptor body at that point — the implementer must read the surrounding code to identify it. If the descriptor body is not yet extracted at the desired short-circuit point, perform a minimal extraction (parse line 6 of the BSMS Round-2 block) before the refusal.

- [ ] **Step 6: Defense-in-depth at `extract_threshold` (R0-I1 corrected framing)**

**R0-I1 corrected:** The existing regex `(?:thresh|multi|sortedmulti)\((\d+)\s*,` already correctly does NOT match `sortedmulti_a(` — the regex requires a literal `(` immediately after `sortedmulti`, but `sortedmulti_a(2,...)` has `_` after `sortedmulti`. Confirmed by source-comment at `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs:961-966` and the existing test assertion at L984-988 (`threshold=none` is the current observed reality for `tr(NUMS, sortedmulti_a(2,...))`).

So there is **no regex bug to fix**. The function correctly returns `Ok(None)` for `tr(NUMS, sortedmulti_a(...))` constructs today. The Slug 1 parse-entry refusal (Step 5) now catches these blobs upstream, so the `Ok(None)` path is unreachable for legitimate flows.

The defense-in-depth value: convert the silent `Ok(None)` into an explicit `Err(BsmsTaprootImportRefused)` for `sortedmulti_a(` / `multi_a(` substrings, defending against any future code path that bypasses the parse-entry refusal. Add at the top of `extract_threshold`:

```rust
pub(super) fn extract_threshold(descriptor_body: &str) -> Result<Option<u8>, ToolkitError> {
    // v0.28.7 defense-in-depth: after Slug 1's BsmsTaprootImportRefused at
    // parse-entry, taproot blobs cannot reach this fn legitimately. But if
    // a future code path bypasses the parse-entry refusal, the existing
    // regex would return Ok(None) on `sortedmulti_a(...)` — silently emitting
    // `threshold=none` rather than refusing. Convert that silent miss into
    // an explicit refusal.
    if descriptor_body.contains("sortedmulti_a(") || descriptor_body.contains("multi_a(") {
        return Err(ToolkitError::BsmsTaprootImportRefused);
    }
    // ... rest of fn unchanged.
}
```

This is purely defense-in-depth — no regex change. The function's positive path stays bit-identical.

- [ ] **Step 7: Rename + flip existing test cell**

In `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs:968`, find `fn bsms_2line_tr_nums_current_behavior_no_refusal()`. Rename to `fn bsms_2line_tr_nums_refused()`. Flip the assertion:

Old (paraphrased):
```rust
let out = run_import_with_fixture(...).success();
let envelope: serde_json::Value = ...;
// asserts envelope present
```

New:
```rust
let out = run_import_with_fixture(...).failure();
assert_eq!(out.status.code(), Some(2));
let stderr = String::from_utf8_lossy(&out.stderr);
assert!(
    stderr.contains("--format bsms does not support taproot import"),
    "expected taproot-refusal stderr, got: {stderr}"
);
assert!(
    stderr.contains("bsms-import-taproot-refusal-parity"),
    "expected FOLLOWUP slug reference in stderr, got: {stderr}"
);
```

- [ ] **Step 8: Add new test cell for `sortedmulti_a` regex side-channel**

Append after the renamed cell:

```rust
#[test]
fn bsms_tr_sortedmulti_a_refused_via_extract_threshold_guard() {
    // Defense-in-depth: even if a taproot descriptor bypassed the
    // parse-entry refusal, extract_threshold rejects sortedmulti_a.
    // This test exercises that contract directly via a synthetic fixture.
    let tmpdir = tempfile::tempdir().expect("tempdir");
    let path = tmpdir.path().join("bsms-tr-sortedmulti-a.bsms");
    // Construct a minimal BSMS Round-2 blob with a tr(NUMS, sortedmulti_a(...)) descriptor.
    // The parse-entry refusal SHOULD catch this — this cell asserts it.
    let blob = "BSMS 1.0\nC0\ntestnet\ntr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,@0,@1))\nm/48'/1'/0'/2'\nbc1q...";
    std::fs::write(&path, blob).unwrap();
    let out = assert_cmd::Command::cargo_bin("mnemonic").unwrap()
        .args(["import-wallet", "--format", "bsms", "--blob", path.to_str().unwrap(), "--json"])
        .output().expect("mnemonic spawn");
    assert_ne!(out.status.code(), Some(0), "must refuse, got success");
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("does not support taproot"),
        "expected taproot-refusal stderr, got: {stderr}"
    );
}
```

(Implementer: the fixture blob is illustrative — the real BSMS Round-2 wire format has specific line structure. Adjust to match the existing `bsms_2line_tr_nums_*` cell's fixture-construction pattern.)

- [ ] **Step 9: Run Slug 1 tests**

```bash
cargo test --package mnemonic-toolkit --test cli_import_wallet_bsms bsms_2line_tr_nums_refused bsms_tr_sortedmulti_a 2>&1 | tail -10
```

Expected: 2 passed.

- [ ] **Step 10: Run full BSMS test file**

```bash
cargo test --package mnemonic-toolkit --test cli_import_wallet_bsms 2>&1 | grep -E '^test result:' | tail -1
```

Expected: all cells passing; one cell renamed (count unchanged).

---

### Task 2: Phase 2 — Slug 2 (`green-emitter-multisig-refusal-template-only`)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_export/mod.rs`
- Modify: `crates/mnemonic-toolkit/src/wallet_export/green.rs`
- Modify: `crates/mnemonic-toolkit/tests/cli_export_wallet_green.rs`

- [ ] **Step 1: Add `WalletScriptType::is_multisig()` method in `wallet_export/mod.rs`**

Find the `impl WalletScriptType` block (likely around L160-ish; existing methods include something like `name()` or `is_p2tr()`). Append:

```rust
/// `true` iff this script type is a multisig variant.
///
/// Used by emitters to refuse multisig in descriptor-mode invocations
/// (where `inputs.template == None`, but `inputs.script_type` is still
/// available from `script_type_from_descriptor`). See FOLLOWUP
/// `green-emitter-multisig-refusal-template-only` (resolved v0.28.7).
pub fn is_multisig(&self) -> bool {
    matches!(
        self,
        Self::P2shMulti | Self::P2shP2wshMulti | Self::P2wshMulti | Self::P2trMulti
    )
}
```

(Implementer: confirm the actual `WalletScriptType` variant names by reading the enum — they may differ from `P2shMulti` / `P2shP2wshMulti` / `P2wshMulti` / `P2trMulti`. Adjust the matches arm to use exact variant names.)

- [ ] **Step 2: Refactor green.rs:30-44 refusal guard**

Current code:
```rust
fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
    if let Some(t) = inputs.template {
        if t.is_multisig() {
            return Err(ToolkitError::BadInput(
                "--format green does not support multisig ...".into(),
            ));
        }
    }
    Ok(format!(...))
}
```

New code:
```rust
fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
    // v0.28.7 — Slug 2: refuse multisig in BOTH template-mode and
    // descriptor-mode (--from-import-json). Previously the refusal was
    // gated on `inputs.template.is_some()`, which silently passed multisig
    // descriptor-mode invocations. See FOLLOWUP
    // `green-emitter-multisig-refusal-template-only` (resolved v0.28.7).
    if inputs.script_type.is_multisig() {
        return Err(ToolkitError::BadInput(
            "--format green does not support multisig wallets; Green's \
             import surface is singlesig-only. Use --format bitcoin-core, \
             --format sparrow, or --format coldcard-multisig for multisig."
                .into(),
        ));
    }
    Ok(format!(...))
}
```

(Keep the existing error-message body if it differs from the proposed text above; preserve the singlesig-only intent and the format-suggestions.)

- [ ] **Step 3: Add new descriptor-mode regression cell**

Append to `tests/cli_export_wallet_green.rs`:

```rust
#[test]
fn cell_4_green_descriptor_mode_multisig_refuses() {
    // v0.28.7 — Slug 2: descriptor-mode (--from-import-json) multisig
    // import-then-export must now refuse on the export side, not silently
    // pass through. Previously bug per FOLLOWUP
    // `green-emitter-multisig-refusal-template-only`.
    //
    // Construct the test by importing a multisig BSMS or coldcard-multisig
    // fixture, then export-wallet --format green --from-import-json - .
    let import_out = assert_cmd::Command::cargo_bin("mnemonic").unwrap()
        .args(["import-wallet", "--format", "coldcard-multisig",
               "--blob", "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt",
               "--json"])
        .output().expect("mnemonic import-wallet spawn");
    assert!(import_out.status.success(), "import must succeed");

    let export_out = assert_cmd::Command::cargo_bin("mnemonic").unwrap()
        .args(["export-wallet", "--format", "green", "--from-import-json", "-"])
        .write_stdin(import_out.stdout)
        .output().expect("mnemonic export-wallet spawn");
    assert_ne!(export_out.status.code(), Some(0), "must refuse, got success");
    let stderr = String::from_utf8_lossy(&export_out.stderr);
    assert!(
        stderr.contains("does not support multisig"),
        "expected multisig-refusal stderr, got: {stderr}"
    );
}
```

(Implementer: fixture path `coldcard-ms-2of3-p2wsh-with-xfp.txt` verified to exist at HEAD `885f522` (R1-fold). Also verify that `write_stdin` is the correct API on the assert_cmd Command builder; alternative: `target/debug/mnemonic` direct invocation via `std::process::Command`.)

- [ ] **Step 4: Run Slug 2 tests**

```bash
cargo test --package mnemonic-toolkit --test cli_export_wallet_green 2>&1 | grep -E '^test result:' | tail -1
```

Expected: 4 passed (was 3; +1 new cell).

- [ ] **Step 5: Verify existing cell_2 still passes**

The pre-refactor cell `cell_2_green_multisig_refuses_byte_exact` uses templated multisig input. The refactor changes the refusal mechanism but not the user-facing outcome (still exits non-zero with multisig refusal). Verify:

```bash
cargo test --package mnemonic-toolkit --test cli_export_wallet_green cell_2 2>&1 | tail -5
```

Expected: PASS.

---

### Task 3: Phase 3 — Slug 3 (`wallet-import-format-mismatch-matrix-completion`, Option B narrow set)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_import_wallet_format_mismatch_matrix.rs`

**Scope (Option B locked):** Complete BSMS / BitcoinCore / ColdcardMultisig arms to full coverage. 17 new `ImportWalletFormatMismatch` return sites; ~17-20 new test cells. File a NEW FOLLOWUP for Coldcard/Sparrow/Specter/Electrum residuals discovered during P0.

- [ ] **Step 1: Read the 3 arms' current dispatch logic**

```bash
# Locate the dispatch site
grep -n 'Some("bsms")\|Some("bitcoin-core")\|Some("coldcard-multisig")' crates/mnemonic-toolkit/src/cmd/import_wallet.rs
```

Read each arm + the 50-line surrounding context to understand the dispatch pattern. Note: arms may use `match` on the sniffer output OR explicit `if let` chains.

- [ ] **Step 2: Extend BSMS arm — add 6 refusal cases**

Per P0 recon dossier, the BSMS arm currently refuses only BitcoinCore. Add 6 missing:
- coldcard (single-sig coldcard.json)
- coldcard-multisig
- electrum
- jade
- sparrow
- specter

Each missing case becomes a `_ => return Err(ToolkitError::ImportWalletFormatMismatch { user_format: "bsms", detected_format: "<X>" })` arm (or equivalent per existing pattern).

- [ ] **Step 3: Extend BitcoinCore arm — add 6 refusal cases**

Symmetric to Step 2 for BitcoinCore arm. Missing: coldcard, coldcard-multisig, electrum, jade, sparrow, specter.

- [ ] **Step 4: Extend ColdcardMultisig arm — add 5 refusal cases**

Missing: coldcard, electrum, jade, sparrow, specter. (Bsms + BitcoinCore already refused.)

- [ ] **Step 5: Create matrix test file (R0-C1 fold: fixture paths verified against actual `tests/fixtures/wallet_import/`)**

`crates/mnemonic-toolkit/tests/cli_import_wallet_format_mismatch_matrix.rs`:

```rust
//! Cross-format mismatch matrix — Option B narrow set (v0.28.7 / Slug 3).
//!
//! Closes FOLLOWUP `wallet-import-format-mismatch-matrix-completion` for
//! the 3 narrow arms (BSMS / BitcoinCore / ColdcardMultisig). The other 4
//! arms (Coldcard / Sparrow / Specter / Electrum) have additional residual
//! gaps discovered during P0 recon — tracked at NEW FOLLOWUP
//! `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.

use assert_cmd::Command;

const FIXTURE_BASE: &str = "tests/fixtures/wallet_import";

fn assert_format_mismatch(user_format: &str, fixture: &str, detected_format: &str) {
    let path = std::path::PathBuf::from(FIXTURE_BASE).join(fixture);
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["import-wallet", "--format", user_format,
               "--blob", path.to_str().unwrap(), "--json"])
        .output().expect("mnemonic spawn");
    assert_ne!(out.status.code(), Some(0),
        "expected non-zero exit for {user_format} vs {detected_format}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("format mismatch") || stderr.contains("ImportWalletFormatMismatch"),
        "expected format-mismatch stderr for {user_format} vs {detected_format}, got: {stderr}");
}

// FIXTURES (verified to exist at HEAD `885f522`):
// - coldcard           → coldcard-singlesig-bip84-mainnet.json
// - coldcard-multisig  → coldcard-ms-2of3-p2wsh-with-xfp.txt (NOTE: .txt)
// - electrum           → electrum-standard-bip84-mainnet.json
// - jade               → jade-multisig-2of3-p2wsh.json (only valid Jade fixture; singlesig-refused)
// - sparrow            → sparrow-singlesig-p2wpkh.json
// - specter            → specter-singlesig-p2wpkh.json

// BSMS arm — refuses 7 other formats (BitcoinCore already covered; add 6 new).
#[test] fn bsms_refuses_coldcard()           { assert_format_mismatch("bsms", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn bsms_refuses_coldcard_multisig()  { assert_format_mismatch("bsms", "coldcard-ms-2of3-p2wsh-with-xfp.txt", "coldcard-multisig"); }
#[test] fn bsms_refuses_electrum()           { assert_format_mismatch("bsms", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn bsms_refuses_jade()               { assert_format_mismatch("bsms", "jade-multisig-2of3-p2wsh.json", "jade"); }
#[test] fn bsms_refuses_sparrow()            { assert_format_mismatch("bsms", "sparrow-singlesig-p2wpkh.json", "sparrow"); }
#[test] fn bsms_refuses_specter()            { assert_format_mismatch("bsms", "specter-singlesig-p2wpkh.json", "specter"); }

// BitcoinCore arm — symmetric (6 new refusals).
#[test] fn bitcoin_core_refuses_coldcard()           { assert_format_mismatch("bitcoin-core", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn bitcoin_core_refuses_coldcard_multisig()  { assert_format_mismatch("bitcoin-core", "coldcard-ms-2of3-p2wsh-with-xfp.txt", "coldcard-multisig"); }
#[test] fn bitcoin_core_refuses_electrum()           { assert_format_mismatch("bitcoin-core", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn bitcoin_core_refuses_jade()               { assert_format_mismatch("bitcoin-core", "jade-multisig-2of3-p2wsh.json", "jade"); }
#[test] fn bitcoin_core_refuses_sparrow()            { assert_format_mismatch("bitcoin-core", "sparrow-singlesig-p2wpkh.json", "sparrow"); }
#[test] fn bitcoin_core_refuses_specter()            { assert_format_mismatch("bitcoin-core", "specter-singlesig-p2wpkh.json", "specter"); }

// ColdcardMultisig arm — 5 new refusals.
#[test] fn coldcard_multisig_refuses_coldcard()  { assert_format_mismatch("coldcard-multisig", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn coldcard_multisig_refuses_electrum()  { assert_format_mismatch("coldcard-multisig", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn coldcard_multisig_refuses_jade()      { assert_format_mismatch("coldcard-multisig", "jade-multisig-2of3-p2wsh.json", "jade"); }
#[test] fn coldcard_multisig_refuses_sparrow()   { assert_format_mismatch("coldcard-multisig", "sparrow-singlesig-p2wpkh.json", "sparrow"); }
#[test] fn coldcard_multisig_refuses_specter()   { assert_format_mismatch("coldcard-multisig", "specter-singlesig-p2wpkh.json", "specter"); }
```

17 cells with all fixture paths now verified to exist. If any cell still fails at Task 3 Step 6, the failure is in the source-side arm logic (not a missing fixture).

- [ ] **Step 6: Run the matrix test file**

```bash
cargo test --package mnemonic-toolkit --test cli_import_wallet_format_mismatch_matrix 2>&1 | grep -E '^test result:' | tail -1
```

Expected: 17 passed. If any cell fails:
- Confirm the source-side arm refusal lands on the expected sniff path.
- If a fixture is malformed (sniffer doesn't classify it as expected), substitute or fix.

---

### Task 4: Phase 4 — Slug 4 (`wallet-import-taproot-internal-key`, Framing B Fix-α)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`
- Modify: `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs`

- [ ] **Step 1: Locate the envelope-gate**

`cmd/export_wallet.rs:650` — `taproot_internal_key: None,` field. Read 50 lines surrounding (L600-700) to understand the `run_from_import_json` flow.

- [ ] **Step 2: Add taproot-envelope refusal via parse-side `script_type` check (R0-I2 fold)**

**R0-I2:** Use parse-side `script_type` detection rather than string-sniff. After `script_type_from_descriptor(&parsed_ms)?` at L612, `script_type` is in scope and reliably indicates taproot via `WalletScriptType::P2tr | WalletScriptType::P2trMulti`. This avoids the string-sniff weakness (whitespace prefixes, etc.).

Insert IMMEDIATELY AFTER the `let script_type = script_type_from_descriptor(&parsed_ms)?;` line (around L612, verify exact line):

```rust
// v0.28.7 — Slug 4 Fix-α: refuse taproot envelopes at the single
// EmitInputs gate. The wallet_import path doesn't surface taproot
// internal-key designation (NUMS vs raw xonly) in the envelope wire
// shape; rather than propagate the gap silently to every emitter via
// `taproot_internal_key: None`, refuse here. Detection uses parse-side
// script_type (not string-sniff). Fix-β (envelope-field addition for
// v0.29+) tracked at FOLLOWUP `wallet-import-taproot-internal-key`
// (resolved v0.28.7 via Fix-α).
if matches!(script_type, WalletScriptType::P2tr | WalletScriptType::P2trMulti) {
    return Err(ToolkitError::BadInput(
        "--from-import-json: taproot descriptors are not yet supported on \
         the export-from-envelope path. The wallet_import path doesn't \
         surface taproot internal-key designation (NUMS vs raw xonly). \
         Use --format <emitter> --descriptor <body> directly, or wait \
         for v0.29+ envelope wire-shape evolution. FOLLOWUP: \
         `wallet-import-taproot-internal-key`."
            .into(),
    ));
}
```

(Implementer: confirm `WalletScriptType` is already in scope at the insertion point; if not, add `use crate::wallet_export::WalletScriptType;` at top of file or qualify with full path.)

- [ ] **Step 3: Add taproot-envelope refusal test cell**

Append to `tests/cli_export_wallet_from_import_json.rs`:

```rust
#[test]
fn p_slug4_taproot_envelope_refused_on_from_import_json() {
    // v0.28.7 — Slug 4 Fix-α: taproot envelope (any descriptor starting
    // with tr(...)) is refused at the EmitInputs gate in
    // run_from_import_json. Symmetric refusal mirrors the conceptual gap
    // documented at cmd/export_wallet.rs:650 (taproot_internal_key: None
    // silent propagation).
    //
    // Construct an envelope with a tr(...) descriptor body and assert
    // refusal regardless of --format.
    let envelope_json = serde_json::json!({
        "schema_version": "1",
        "source_format": "bsms",  // any source — the body shape is what matters
        "bundle": {
            "schema_version": "4",
            "descriptor": "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,multi_a(2,[12345678/86'/0'/0']xpub6...,[abcdef00/86'/0'/0']xpub6...))",
            "mode": "full",
            "network": "mainnet",
            // ... other required BundleJson fields
        }
    });
    let envelope_str = envelope_json.to_string();

    for fmt in &["bitcoin-core", "sparrow", "coldcard", "electrum"] {
        let out = assert_cmd::Command::cargo_bin("mnemonic").unwrap()
            .args(["export-wallet", "--format", fmt, "--from-import-json", "-"])
            .write_stdin(envelope_str.clone())
            .output().expect("mnemonic spawn");
        assert_ne!(out.status.code(), Some(0), "format={fmt}: must refuse, got success");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("taproot descriptors are not yet supported"),
            "format={fmt}: expected taproot-refusal stderr, got: {stderr}"
        );
    }
}
```

(Implementer: the envelope JSON skeleton above is illustrative — fill in the actual required BundleJson fields by reading the existing envelope-mode test cells. The cell asserts refusal across 4 representative formats to confirm the refusal is upstream of per-emitter dispatch.)

- [ ] **Step 4: Run Slug 4 tests**

```bash
cargo test --package mnemonic-toolkit --test cli_export_wallet_from_import_json p_slug4 2>&1 | tail -5
```

Expected: PASS.

---

### Task 5: Phase 5 — full test suite + clippy

- [ ] **Step 1: Full toolkit test suite**

```bash
cargo test --package mnemonic-toolkit --tests 2>&1 | grep -E '^test result:' | awk '{s+=$4} END {print "Total passing:", s}'
```

Expected: 2008 + (Slug 1: 1 renamed + 1 new = +1) + (Slug 2: +1) + (Slug 3: +17) + (Slug 4: +1) = **2028**.

- [ ] **Step 2: Clippy**

```bash
cargo clippy --package mnemonic-toolkit --tests -- -D warnings 2>&1 | tail -3
```

Expected: `Finished` with no warnings.

- [ ] **Step 3: `make audit` smoke**

```bash
make -C docs/manual audit \
  MNEMONIC_BIN=$PWD/target/debug/mnemonic \
  MD_BIN=$(which md) MS_BIN=$(which ms) MK_BIN=$(which mk) \
  FIXTURES_DIR=$PWD/crates/mnemonic-toolkit/tests/fixtures/wallet_import \
  2>&1 | tail -5
```

Expected: `[lint] OK` + `[verify-examples] OK`. None of the slugs touch CLI surface or transcripts.

---

### Task 6: Phase 6 — opus architect review

- [ ] **Step 1: Dispatch opus via Agent tool**

Use `Agent`:
- `subagent_type: feature-dev:code-reviewer`
- `model: opus`
- Prompt verifies all 4 slug closures + cross-cutting (alphabetical-by-variant-name; new `WalletScriptType::is_multisig()` method; no GUI lockstep needed; CHANGELOG accuracy):

```
Review Cycle 3 (v0.28.7) full working tree against design/PLAN_mnemonic_toolkit_v0_28_7.md.

4 slug closures to verify:
1. bsms-import-taproot-refusal-parity — new BsmsTaprootImportRefused variant; parse-entry refusal; extract_threshold regex defense-in-depth.
2. green-emitter-multisig-refusal-template-only — WalletScriptType::is_multisig() added; refusal moves from template-gated to script_type-gated; descriptor-mode regression cell.
3. wallet-import-format-mismatch-matrix-completion — Option B narrow set: 17 new arms across BSMS/BitcoinCore/ColdcardMultisig; new matrix test file.
4. wallet-import-taproot-internal-key — Fix-α refusal at envelope gate; cross-format refusal cell.

Cross-cutting:
- alphabetical-by-variant-name ordering for new ToolkitError variants (CLAUDE.md).
- no GUI lockstep needed (no CLI surface change; no wire-shape change).
- CHANGELOG accuracy.
- ALL test cells pass (~2028 total).
- clippy clean.

Gate: 0 critical / 0 important.
```

- [ ] **Step 2: Persist opus review verbatim**

Save the opus output to `design/agent-reports/v0_28_7-phase-6-review.md` BEFORE applying any folds (per CLAUDE.md "Per-phase architect-review agent outputs persist verbatim").

- [ ] **Step 3: Fold any Important findings inline**

If opus returns YELLOW/RED, fix each Important inline + re-dispatch until GREEN.

---

### Task 7: Phase 7 — release tooling + commit + tag + push + GH Release

- [ ] **Step 1: Bump Cargo.toml**

Old: `version = "0.28.6"`
New: `version = "0.28.7"`

- [ ] **Step 2: Add CHANGELOG entry**

Insert above `## mnemonic-toolkit [0.28.6]`:

```markdown
## mnemonic-toolkit [0.28.7] — 2026-05-20

Patch release: 4 hardening FOLLOWUPs surfaced during the v0.28.x cycle.

### Imports / Exports — defect refusal hardening

- **`bsms-import-taproot-refusal-parity`** — Add import-side parity of `BsmsTaprootRefused`. New variant `BsmsTaprootImportRefused` (no `script_type` field — import parser has no `WalletScriptType` in scope). BSMS parser now short-circuits on `tr(` substring at parse-entry, mirroring emit-side refusal. Defense-in-depth: `extract_threshold` now refuses `sortedmulti_a(` taproot constructs. (User-visible: `mnemonic import-wallet --format bsms ... <taproot blob>` now exits 2 with explanatory message + FOLLOWUP slug reference.)

- **`green-emitter-multisig-refusal-template-only`** — Refactor green emitter's multisig refusal from `inputs.template.is_some() && t.is_multisig()` → `inputs.script_type.is_multisig()`. Closes the bug where descriptor-mode (`--from-import-json`) multisig green exports silently passed through despite Green's import surface being singlesig-only. New `WalletScriptType::is_multisig()` method covers `P2shMulti | P2shP2wshMulti | P2wshMulti | P2trMulti`. Anti-pattern survey: isolated to green.rs; no other emitters share the same bug.

- **`wallet-import-format-mismatch-matrix-completion` (Option B narrow set)** — Extend BSMS / BitcoinCore / ColdcardMultisig dispatch arms to refuse all 17 missing sniff outcomes. New matrix test file `tests/cli_import_wallet_format_mismatch_matrix.rs`. NOTE: P0 recon discovered the original FOLLOWUPS scope was structurally narrower than actual residuals — Coldcard / Sparrow / Specter / Electrum arms also have residual gaps. Those discovered gaps are filed as NEW FOLLOWUP `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.

- **`wallet-import-taproot-internal-key` (Fix-α envelope-gate refusal)** — Refuse taproot envelopes at the single `EmitInputs` construction gate in `cmd/export_wallet.rs:run_from_import_json`. The `wallet_import` path doesn't yet surface taproot internal-key designation (NUMS vs raw xonly); refusing at the gate is preferable to silent `taproot_internal_key: None` propagation. Fix-β (envelope wire-shape evolution to carry the field) remains open for v0.29+.

### Tests

- 4 slug closures contribute +20 net cells: +1 Slug 1 (sortedmulti_a regex side-channel; 1 cell renamed), +1 Slug 2 (descriptor-mode multisig refusal), +17 Slug 3 matrix, +1 Slug 4. Total: 2008 → 2028.

### Note

Cycle 3 of v0.28+ residual FOLLOWUP release plan (Wave 2 hardening). See `design/BRAINSTORM_v0_28_plus_residual_followups.md` + `design/PLAN_mnemonic_toolkit_v0_28_7.md` + `design/cycle-3-p0-recon.md`.

No CLI surface change; no wire-shape change; no GUI lockstep.

---
```

- [ ] **Step 3: Bump scripts/install.sh:32**

Old: `mnemonic-toolkit-v0.28.6`
New: `mnemonic-toolkit-v0.28.7`

- [ ] **Step 4: Flip FOLLOWUPS Status × 4 + file 1 NEW FOLLOWUP**

For each of the 4 closed slugs (`bsms-import-taproot-refusal-parity` + `green-emitter-multisig-refusal-template-only` + `wallet-import-format-mismatch-matrix-completion` + `wallet-import-taproot-internal-key`):

```markdown
- **Status:** `resolved <PLACEHOLDER-COMMIT-SHA>` — mnemonic-toolkit-v0.28.7 cycle <slug-specific resolution prose>.
```

ALSO file a NEW FOLLOWUP at the bottom of the `### Imports / Exports` section in `design/FOLLOWUPS.md`:

```markdown
### `wallet-import-format-mismatch-matrix-completion-discovered-gaps` — Coldcard/Sparrow/Specter/Electrum arm residuals (post-Cycle-3 discovery)

- **Surfaced:** 2026-05-20, during Cycle 3 P0 recon (design/cycle-3-p0-recon.md Slug 3). The original `wallet-import-format-mismatch-matrix-completion` FOLLOWUP body listed only BSMS / BitcoinCore / ColdcardMultisig as narrow-arm residuals. P0 recon found 4 additional arms with residual gaps: Coldcard (2 missing: electrum, jade), Sparrow (4 missing: coldcard, electrum, jade, specter), Specter (3 missing: coldcard, electrum, jade), Electrum (1 missing: jade). Total: 10 additional missing arms / ~10 additional test cells.
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — Coldcard, Sparrow, Specter, Electrum dispatch arms.
- **What:** Extend each arm to refuse all wrong-format sniff outcomes symmetrically. Closes the 8×7 = 56-cell full off-diagonal matrix.
- **Why deferred:** Cycle 3 scope was locked at Option B (original 3-arm narrow set) per user decision 2026-05-20.
- **Status:** `open`
- **Tier:** `v0.28+-test-hygiene`
- **Tags:** `wallet`
- **Companion:** Parent `wallet-import-format-mismatch-matrix-completion` (resolved v0.28.7).
```

- [ ] **Step 5: Rebuild + verify binary version**

```bash
cargo build --bin mnemonic 2>&1 | tail -3
target/debug/mnemonic --version
```

Expected: `mnemonic 0.28.7`.

- [ ] **Step 6: Commit + tag + push + GH Release**

```bash
git status --short
git add crates/mnemonic-toolkit/src/error.rs \
        crates/mnemonic-toolkit/src/wallet_import/bsms.rs \
        crates/mnemonic-toolkit/src/wallet_export/green.rs \
        crates/mnemonic-toolkit/src/wallet_export/mod.rs \
        crates/mnemonic-toolkit/src/cmd/import_wallet.rs \
        crates/mnemonic-toolkit/src/cmd/export_wallet.rs \
        crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs \
        crates/mnemonic-toolkit/tests/cli_export_wallet_green.rs \
        crates/mnemonic-toolkit/tests/cli_import_wallet_format_mismatch_matrix.rs \
        crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs \
        crates/mnemonic-toolkit/Cargo.toml \
        Cargo.lock \
        CHANGELOG.md \
        scripts/install.sh \
        design/FOLLOWUPS.md \
        design/PLAN_mnemonic_toolkit_v0_28_7.md \
        design/cycle-3-p0-recon.md
git commit -m "release(toolkit): mnemonic-toolkit v0.28.7 — hardening (4 v0.28+ FOLLOWUPs)..."
SHA=$(git rev-parse HEAD)
sed -i "s/resolved <PLACEHOLDER-COMMIT-SHA>/resolved $SHA/g" design/FOLLOWUPS.md
git add design/FOLLOWUPS.md
git commit --amend --no-edit
git tag mnemonic-toolkit-v0.28.7
git push origin master
git push origin mnemonic-toolkit-v0.28.7
gh release create mnemonic-toolkit-v0.28.7 --title "..." --notes "..."
```

(Full commit message + GH Release notes in Task 7 Step 2 CHANGELOG block above — controller renders them at commit time.)

---

## Self-review

### Spec coverage

- Slug 1 (`bsms-import-taproot-refusal-parity`) → Task 1 ✓
- Slug 2 (`green-emitter-multisig-refusal-template-only`) → Task 2 ✓
- Slug 3 (`wallet-import-format-mismatch-matrix-completion` Option B) → Task 3 ✓
- Slug 4 (`wallet-import-taproot-internal-key` Framing B Fix-α) → Task 4 ✓
- Cargo test + clippy + audit → Task 5 ✓
- Opus architect review → Task 6 ✓
- Release tooling + commit + tag + push + GH Release + NEW FOLLOWUP filing → Task 7 ✓

### Placeholder scan

- `<slug-specific resolution prose>` in Step 4 — implementer fills in per-slug specific resolution at sed-amend time.
- `<PLACEHOLDER-COMMIT-SHA>` — sed-backfill pattern (Cycle 1/2 precedent).
- No TBD/TODO in code blocks; all code blocks contain complete proposed changes.

### Type consistency

- `WalletScriptType::is_multisig()` — added in Task 2 Step 1; used in Task 2 Step 2.
- `BsmsTaprootImportRefused` — added in Task 1 Steps 1-4; used in Task 1 Step 5.
- `ToolkitError::ImportWalletFormatMismatch` — existing variant; extended usage in Task 3.

### Effort estimate sanity-check

Per brainstorm: ~3-5 days. Per-task:
- Task 1 (Slug 1): ~1 day (error.rs variant + parser short-circuit + regex defense + 2 test cells + edge cases on regex)
- Task 2 (Slug 2): ~0.5 day (mod.rs method + green.rs refactor + 1 test cell)
- Task 3 (Slug 3 Option B): ~1-1.5 days (3 arms × ~5-6 cases + 17-cell matrix file + fixture-existence checks)
- Task 4 (Slug 4): ~0.5 day (envelope-gate refusal + 1 cross-format test cell)
- Task 5 (test + clippy + audit): ~0.5 day
- Task 6 (opus review + fold): ~0.5 day
- Task 7 (release tooling): ~0.5 day

Total: ~4.5-5 days. In brainstorm range.

---

## Risk flags

- **Defense-in-depth ordering** (Task 1 Step 6) — implementer must confirm the parse-entry short-circuit (Step 5) fires BEFORE `extract_threshold` is reached on all code paths. If a future code path bypasses the parse-entry refusal, the defense-in-depth `contains("sortedmulti_a(")` check at `extract_threshold` is the second guard; if both fail, taproot blobs would silently return `Ok(None)`. Verify by reading all callers of `extract_threshold`.
- **Slug 3 fixture coverage** (Task 3 Step 5) — fixture paths verified at HEAD `885f522` per R0-C1 fold. If the sniffer returns a different `detected_format` for a given fixture than the test fn name claims, the format-mismatch arm refusal will still fire (since user_format ≠ detected_format), but the test fn name becomes misleading. Implementer should sanity-check sniffer outcomes against fn-name semantics during execution.
- **Slug 2 cell_2 regression** (Task 2 Step 5) — the refactor changes refusal mechanism (template-gated → script_type-gated). Existing `cell_2_green_multisig_refuses_byte_exact` must still PASS post-refactor. If it fails, the refactor is wrong (either `WalletScriptType::is_multisig()` doesn't cover the templated case, or `script_type_from_descriptor` doesn't return a multisig variant for templated input). Investigate before committing.
- **Slug 4 `WalletScriptType` scope** (Task 4 Step 2) — R0-I2 fold uses `matches!(script_type, WalletScriptType::P2tr | WalletScriptType::P2trMulti)`. R1 verified `WalletScriptType` is NOT currently in scope at `cmd/export_wallet.rs` top-of-file (lines L11-17 import group). Implementer must add `WalletScriptType` to the existing `use crate::wallet_export::{...}` import group OR fully-qualify the path. Compile error will be loud + fast if missed.
- **NEW FOLLOWUP filing co-located with closure** (Task 7 Step 4) — be sure the NEW FOLLOWUP `wallet-import-format-mismatch-matrix-completion-discovered-gaps` is filed in the SAME commit as the 4 Status flips; otherwise the cycle's audit trail is fragmented.

---

## Sub-skill expectations

This plan assumes the executor uses `superpowers:subagent-driven-development` (recommended) per the user's Wave-1 choice. Subagents per-task with two-stage review between (sonnet for spec compliance + opus reviewer at Task 6 cycle-close).
