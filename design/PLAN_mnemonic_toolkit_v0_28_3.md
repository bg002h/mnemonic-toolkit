# mnemonic-toolkit-v0.28.3 Implementation Plan (Cycle 1 / Wave 1 second)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enforce the `EmitInputs.canonical_descriptor` BIP-380 `#<8-char-csum>` suffix invariant via a `CheckedDescriptor<'_>(&'_ str)` newtype. Replace the runtime convention documented in `wallet_export/bsms.rs:86-90` with a compile-time type-system guarantee. Tag `mnemonic-toolkit-v0.28.3`.

**Architecture:** Add a `CheckedDescriptor<'a>(&'a str)` newtype in `wallet_export/mod.rs` with a `new()` constructor that validates the BIP-380 suffix and a `Deref<Target = str>` + `Display` impl so existing consumer code (BSMS L2, Specter descriptor field, Green plaintext) continues to work via auto-deref. Change `EmitInputs.canonical_descriptor` field type from `&'a str` to `CheckedDescriptor<'a>`. Wrap the value at two construction sites in `cmd/export_wallet.rs` (L437 for `--template`/`--descriptor` modes; L608 for `--from-import-json`). The F9 regression tests landed in v0.28.2 continue to validate the EMIT side; new unit tests cover the NEWTYPE constructor's positive + negative cases.

**Tech Stack:** Rust, miniscript crate, BIP-380 `#<8-char-csum>` suffix grammar, `cargo test --tests`, `cargo clippy --tests -- -D warnings`.

**Brainstorm spec:** `design/BRAINSTORM_followups_abc_release_plan.md` § "Cycle 1 — mnemonic-toolkit-v0.28.3 (A2) — Wave 1 second".

**Source SHA at plan-write time:** `2080d14`.

---

## File structure

- **Modify:** `crates/mnemonic-toolkit/src/wallet_export/mod.rs`
  - Add `CheckedDescriptor<'a>` newtype definition (+ `new()` + `as_str()` + `Deref` + `Display` impls).
  - Change `EmitInputs.canonical_descriptor` field type at L345 from `&'a str` → `CheckedDescriptor<'a>`.
- **Modify:** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`
  - L437 region: wrap `&canonical` (or equivalent local) via `CheckedDescriptor::new(...)?` before passing into the `EmitInputs { ... }` literal.
  - L608 region: same wrapping for the `--from-import-json` path.
- **Modify:** `crates/mnemonic-toolkit/src/wallet_export/bsms.rs`
  - L86-90 region: invariant comment changes from "by convention" to "by type" wording.
- **Create:** `crates/mnemonic-toolkit/tests/checked_descriptor.rs`
  - New integration-test file with unit cells exercising `CheckedDescriptor::new()` positive + negative paths.
- **Modify:** `crates/mnemonic-toolkit/Cargo.toml`
  - `version = "0.28.2"` → `"0.28.3"`.
- **Modify:** `CHANGELOG.md`
  - Insert new `## mnemonic-toolkit [0.28.3] — <date>` section above the `[0.28.2]` section.
- **Modify:** `scripts/install.sh`
  - L32: `mnemonic-toolkit-v0.28.2` → `mnemonic-toolkit-v0.28.3` (per install-pin-check.yml CI gate).
- **Modify:** `design/FOLLOWUPS.md`
  - `emitinputs-canonical-descriptor-checksum-invariant-enforcement` entry — Status flip.

Optional (architect call at execution time): consumer-site bindings in `wallet_export/{bsms,specter,green}.rs` if `Deref` auto-coercion isn't sufficient for a particular call site. Expected: no consumer-site changes needed; `Deref<Target = str>` covers BSMS `let line2 = inputs.canonical_descriptor;` (binds to `CheckedDescriptor<'_>`; auto-derefs to `&str` when used in `format!`/`String::push_str`/etc).

---

## Tasks

### Task 1: Recon — read EmitInputs + 2 construction sites

**Files:** none modified (read-only).

- [ ] **Step 1: Read EmitInputs struct definition**

Run:
```bash
sed -n '335,370p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export/mod.rs
```

Note the exact struct layout, lifetime parameter (`<'a>`), and the line number of the `canonical_descriptor: &'a str` field. The plan assumes L345; verify against current source.

- [ ] **Step 2: Read construction site at cmd/export_wallet.rs:437**

Run:
```bash
sed -n '420,460p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/export_wallet.rs
```

Note: the `EmitInputs { canonical_descriptor: &canonical, ... }` literal serves BOTH `--template` and `--descriptor` modes. The local variable name may be `canonical` (singular) at L437; verify.

- [ ] **Step 3: Read construction site at cmd/export_wallet.rs:608**

Run:
```bash
sed -n '595,620p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/export_wallet.rs
```

Note: the `--from-import-json` path's local was renamed `canonical_descriptor` (post-F9 fix at `615b10e`) to mirror the `parsed_ms.to_string()` output. Verify.

- [ ] **Step 4: Read bsms.rs:86-90 invariant comment**

Run:
```bash
sed -n '85,95p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export/bsms.rs
```

Confirm the comment block matches the verbatim text from `cycle-prep-recon.md`: "Lines 1 + 2 are shared between the 2-line and 4-line shapes. Line 2 is `EmitInputs.canonical_descriptor` verbatim — the canonical builder ... and descriptor-passthrough both produce strings with the `#<checksum>` suffix already attached."

---

### Task 2: Add CheckedDescriptor newtype + impls in mod.rs

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_export/mod.rs`

- [ ] **Step 1: Locate insertion point**

The newtype goes IMMEDIATELY ABOVE the `EmitInputs` struct (so it's defined before being referenced). The struct is at ~L342; insert the newtype block at ~L335.

Run:
```bash
sed -n '330,345p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export/mod.rs
```

Find a clean insertion point (between unrelated items or just above `pub struct EmitInputs`).

- [ ] **Step 2: Insert the newtype definition**

Insert this block immediately above the `pub struct EmitInputs` declaration:

```rust
/// v0.28.3 (A2): compile-time enforcement of the `EmitInputs.canonical_descriptor`
/// BIP-380 `#<8-char-csum>` suffix invariant. Pre-v0.28.3 the invariant was
/// documented at `wallet_export/bsms.rs:86-90` and enforced only by convention
/// at construction sites; a future code path that built `EmitInputs` from a
/// stripped-body descriptor would silently regress BSMS L2 + Specter
/// `descriptor` JSON field + Green plaintext (latent class surfaced by F9).
///
/// `CheckedDescriptor::new` validates the suffix and returns `Result` on
/// failure; `Deref<Target = str>` lets consumers continue to bind via
/// `inputs.canonical_descriptor` with auto-deref to `&str`.
#[derive(Debug, Clone, Copy)]
pub struct CheckedDescriptor<'a>(&'a str);

impl<'a> CheckedDescriptor<'a> {
    /// Construct a `CheckedDescriptor` from a descriptor string that MUST
    /// end with `#<8-char-csum>` per BIP-380. Returns `Err(BadInput)` if
    /// the suffix is missing, the wrong length, or not ASCII-alphanumeric.
    pub fn new(desc: &'a str) -> Result<Self, crate::error::ToolkitError> {
        let pos = desc.rfind('#').ok_or_else(|| {
            crate::error::ToolkitError::BadInput(format!(
                "CheckedDescriptor: missing BIP-380 `#<csum>` suffix in: {desc:?}"
            ))
        })?;
        let csum = &desc[pos + 1..];
        if csum.len() != 8 {
            return Err(crate::error::ToolkitError::BadInput(format!(
                "CheckedDescriptor: BIP-380 checksum must be 8 chars, got {} in: {desc:?}",
                csum.len()
            )));
        }
        if !csum.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(crate::error::ToolkitError::BadInput(format!(
                "CheckedDescriptor: BIP-380 checksum must be ASCII-alphanumeric, got {csum:?} in: {desc:?}"
            )));
        }
        Ok(Self(desc))
    }

    /// Return the underlying descriptor string (with `#<csum>` suffix).
    pub fn as_str(&self) -> &'a str {
        self.0
    }
}

impl<'a> std::ops::Deref for CheckedDescriptor<'a> {
    type Target = str;
    fn deref(&self) -> &str {
        self.0
    }
}

impl<'a> std::fmt::Display for CheckedDescriptor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
```

- [ ] **Step 3: Verify the insertion compiled separately (cargo check on the module)**

Run:
```bash
cargo check --package mnemonic-toolkit 2>&1 | tail -20
```

Expected: no errors. (If errors appear, they're likely about the `EmitInputs` field type still being `&'a str` — that's fixed in Task 3.)

---

### Task 3: Change EmitInputs.canonical_descriptor field type

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_export/mod.rs`

- [ ] **Step 1: Locate the `EmitInputs` struct field**

Run:
```bash
grep -n 'canonical_descriptor' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export/mod.rs
```

Expected: one line at ~L345 — `pub canonical_descriptor: &'a str,`.

- [ ] **Step 2: Replace the field type**

Old:
```rust
pub canonical_descriptor: &'a str,
```

New:
```rust
pub canonical_descriptor: CheckedDescriptor<'a>,
```

- [ ] **Step 3: Run cargo check to see the cascade**

Run:
```bash
cargo check --package mnemonic-toolkit 2>&1 | grep -E 'error\[|error:' | head -20
```

Expected: 2 errors at `cmd/export_wallet.rs:437` and `cmd/export_wallet.rs:608` (where the value is constructed) — both report a type mismatch between `&str` and `CheckedDescriptor<'_>`. These are fixed in Task 4.

Consumer-site errors are NOT expected (Deref + Display impls cover the call sites). If any consumer site errors, fix in Task 4 Step 5.

---

### Task 4: Wrap value at 2 construction sites in cmd/export_wallet.rs

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`

- [ ] **Step 1: Add `CheckedDescriptor` import**

At the top of `cmd/export_wallet.rs`, find the existing `use crate::wallet_export::...` import block and add `CheckedDescriptor`:

Run:
```bash
grep -n 'use crate::wallet_export::' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/export_wallet.rs | head -3
```

Then add `CheckedDescriptor` to the existing list (or insert a new `use` line):

```rust
use crate::wallet_export::CheckedDescriptor;
```

- [ ] **Step 2: Wrap value at L437 construction site**

Find the `EmitInputs { ... canonical_descriptor: &canonical, ... }` literal (or similar; the local-variable name may differ — re-grep at execution time).

Old (representative):
```rust
let inputs = EmitInputs {
    canonical_descriptor: &canonical,
    ...
};
```

New:
```rust
let canonical_checked = CheckedDescriptor::new(&canonical)?;
let inputs = EmitInputs {
    canonical_descriptor: canonical_checked,
    ...
};
```

- [ ] **Step 3: Wrap value at L608 construction site (--from-import-json path)**

The local variable here is `canonical_descriptor` (post-F9 fix at `615b10e`). The construction is at the `EmitInputs { canonical_descriptor: &canonical_descriptor, ... }` literal at ~L598-599 OR ~L608 (re-grep at execution time).

Old:
```rust
let inputs = EmitInputs {
    canonical_descriptor: &canonical_descriptor,
    ...
};
```

New:
```rust
let canonical_checked = CheckedDescriptor::new(&canonical_descriptor)?;
let inputs = EmitInputs {
    canonical_descriptor: canonical_checked,
    ...
};
```

- [ ] **Step 4: Run cargo check**

Run:
```bash
cargo check --package mnemonic-toolkit 2>&1 | grep -E 'error\[|error:' | head -10
```

Expected: zero errors.

If a consumer site (e.g., bsms.rs L92, specter.rs L68, green.rs L43) errors with "expected str, found CheckedDescriptor", apply this pattern at the consumer site (NOT inside the newtype):

Old:
```rust
let line2 = inputs.canonical_descriptor;
```

New:
```rust
let line2 = inputs.canonical_descriptor.as_str();
```

OR — if the consumer just passes through `Display`/`fmt::Write`/etc, no change needed (auto-coerce works).

- [ ] **Step 5: Verify cargo check is clean**

Re-run:
```bash
cargo check --package mnemonic-toolkit 2>&1 | tail -5
```

Expected: `Finished` line; no errors or warnings.

---

### Task 5: Write TDD tests for CheckedDescriptor::new()

**Files:**
- Create: `crates/mnemonic-toolkit/tests/checked_descriptor.rs`

- [ ] **Step 1: Create the new test file**

Write the file with this content:

```rust
//! v0.28.3 (A2) — unit tests for the `CheckedDescriptor<'_>` newtype that
//! compile-time-enforces the `EmitInputs.canonical_descriptor` BIP-380
//! `#<8-char-csum>` suffix invariant. Forward-looking defensive engineering
//! per the manual-v0.2.0 cycle's P1b R1 architect §F9 Axis B observation;
//! brainstorm at `design/BRAINSTORM_followups_abc_release_plan.md`.

use mnemonic_toolkit::wallet_export::CheckedDescriptor;

#[test]
fn checked_descriptor_accepts_descriptor_with_canonical_8char_checksum() {
    // Constructed canonical: `wpkh([fp/path]xpub)#xxxxxxxx`. The trailing
    // `#abc12345` is a valid BIP-380 alphanumeric 8-char checksum.
    let desc = "wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#tk4vnxy8";
    let checked = CheckedDescriptor::new(desc).expect("valid descriptor");
    assert_eq!(checked.as_str(), desc);
}

#[test]
fn checked_descriptor_rejects_missing_checksum_suffix() {
    // Stripped body (no `#csum`) — the F9 pre-fix regression class.
    let desc = "wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)";
    let err = CheckedDescriptor::new(desc).expect_err("missing checksum must error");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("missing BIP-380") || msg.contains("missing"),
        "expected missing-checksum error, got: {msg}"
    );
}

#[test]
fn checked_descriptor_rejects_wrong_length_checksum() {
    // 6-char checksum instead of 8.
    let desc = "wpkh([5436d724/84'/0'/0']xpub.../<0;1>/*)#abc123";
    let err = CheckedDescriptor::new(desc).expect_err("wrong-length must error");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("8 chars") || msg.contains("must be 8"),
        "expected length-rule error, got: {msg}"
    );
}

#[test]
fn checked_descriptor_rejects_non_alphanumeric_checksum() {
    // 8 chars but contains a non-alphanumeric (BIP-380 uses lowercase
    // base32 alphabet excluding 1/b/i/o — but our constructor accepts any
    // ASCII-alphanumeric; tighten later if needed).
    let desc = "wpkh([5436d724/84'/0'/0']xpub.../<0;1>/*)#abc!@#$%";
    let err = CheckedDescriptor::new(desc).expect_err("non-alphanumeric must error");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("ASCII-alphanumeric") || msg.contains("alphanumeric"),
        "expected alphanumeric-rule error, got: {msg}"
    );
}

#[test]
fn checked_descriptor_deref_to_str_for_consumer_compat() {
    // Consumer-site auto-deref check: `*checked` behaves like `&str` so
    // existing call sites in bsms.rs / specter.rs / green.rs continue to
    // compile after the EmitInputs field-type change.
    let desc = "wpkh([5436d724/84'/0'/0']xpub.../<0;1>/*)#tk4vnxy8";
    let checked = CheckedDescriptor::new(desc).expect("valid");
    let s: &str = &checked;
    assert_eq!(s, desc);
    assert!(checked.contains("wpkh"));
    assert!(checked.starts_with("wpkh"));
}
```

- [ ] **Step 2: Run the new tests — TDD red expected**

Wait — actually these tests SHOULD pass right after Task 2 + 3 + 4 land (the newtype is implemented). The TDD-red phase fires only if I write tests BEFORE adding the newtype. Per Task ordering, Task 2 added the newtype FIRST, then this Task 5 writes tests. So tests should be green on first run.

Run:
```bash
cargo test --package mnemonic-toolkit --test checked_descriptor 2>&1 | tail -10
```

Expected: 5 passed; 0 failed.

If any fail, the newtype impl from Task 2 needs adjustment. Common causes:
- The `ToolkitError::BadInput` message format differs from what the test expects — relax the assertion's `msg.contains(...)` predicate or update the error string in Task 2's `CheckedDescriptor::new` body.
- `Deref` impl missing — re-verify Task 2 Step 2.

---

### Task 6: bsms.rs invariant comment update

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_export/bsms.rs`

- [ ] **Step 1: Read the current invariant comment block**

Run:
```bash
sed -n '85,95p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export/bsms.rs
```

- [ ] **Step 2: Replace the comment block**

Old (verbatim — verify with the recon dossier if it has drifted):
```rust
        // Lines 1 + 2 are shared between the 2-line and 4-line shapes. Line 2
        // is `EmitInputs.canonical_descriptor` verbatim — the canonical
        // builder (`wallet_export::pipeline::build_descriptor_string`) and
        // descriptor-passthrough both produce strings with the `#<checksum>`
        // suffix already attached.
```

New:
```rust
        // Lines 1 + 2 are shared between the 2-line and 4-line shapes. Line 2
        // is `EmitInputs.canonical_descriptor` verbatim — its type
        // `CheckedDescriptor<'_>` (added v0.28.3 / A2) enforces the BIP-380
        // `#<8-char-csum>` suffix invariant at construction time. Pre-v0.28.3
        // the invariant was enforced by convention at construction sites;
        // post-v0.28.3 it's compile-time-guaranteed. `Deref<Target = str>`
        // means this binding continues to work as `&str` for `format!`.
```

- [ ] **Step 3: Verify the comment edit**

Run:
```bash
sed -n '85,95p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export/bsms.rs
```

Confirm the new comment block matches the spec.

---

### Task 7: Full test suite + clippy

**Files:** none modified.

- [ ] **Step 1: Run full toolkit test suite**

Run:
```bash
cargo test --package mnemonic-toolkit --tests 2>&1 | tail -10
```

Expected: all tests pass. Total cell count: previous baseline was 1996 (post-v0.28.2 / F9 cells); this cycle adds ~5 cells from Task 5 → expect ~2001 total.

If any pre-existing F9 cell (`f9_from_import_json_bsms_l2_carries_bip380_checksum`, `f9_from_import_json_specter_descriptor_carries_bip380_checksum`) fails, the c2-B fix has somehow regressed — DO NOT proceed; triage.

- [ ] **Step 2: Run clippy on tests + src**

Run:
```bash
cargo clippy --package mnemonic-toolkit --tests -- -D warnings 2>&1 | tail -10
```

Expected: `Finished` line; no warnings.

If clippy warns about the new newtype (e.g., `#[derive(Copy)]` on a struct holding a `&str` is fine since `&str` is Copy; clippy may suggest other styles), apply the suggestion or `#[allow(...)]` with rationale.

---

### Task 8: Sonnet reviewer fold-verify

**Files:** none modified.

- [ ] **Step 1: Dispatch sonnet via Agent tool**

Use the `Agent` tool with:
- `subagent_type: feature-dev:code-reviewer`
- `model: sonnet`
- Prompt that asks the reviewer to verify:
  1. `CheckedDescriptor<'a>` newtype is added at `wallet_export/mod.rs` (above EmitInputs) with `new()` + `as_str()` + `Deref<Target = str>` + `Display` + `Clone` + `Copy` + `Debug` impls.
  2. `EmitInputs.canonical_descriptor` field type is `CheckedDescriptor<'a>` (NOT `&'a str`).
  3. Both construction sites in `cmd/export_wallet.rs` wrap via `CheckedDescriptor::new(...)?`.
  4. `bsms.rs:86-90` comment block updated to "by type" wording.
  5. 5 new test cells in `tests/checked_descriptor.rs` all pass.
  6. F9 regression cells (`f9_from_import_json_bsms_l2_carries_bip380_checksum`, `f9_from_import_json_specter_descriptor_carries_bip380_checksum`) continue to pass.
  7. clippy is clean.

Gate: 0 critical / 0 important to proceed.

- [ ] **Step 2: Fold any Important findings inline**

Loop until 0 Important.

---

### Task 9: Version bump + CHANGELOG + install.sh

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml`
- Modify: `CHANGELOG.md`
- Modify: `scripts/install.sh`

- [ ] **Step 1: Bump Cargo.toml version**

Edit `crates/mnemonic-toolkit/Cargo.toml`:

Old:
```toml
version = "0.28.2"
```

New:
```toml
version = "0.28.3"
```

- [ ] **Step 2: Add CHANGELOG.md entry**

Insert this section above the existing `## mnemonic-toolkit [0.28.2]` section (use today's date as YYYY-MM-DD):

```markdown
## mnemonic-toolkit [0.28.3] — <YYYY-MM-DD>

Patch release: compile-time enforcement of the `EmitInputs.canonical_descriptor` BIP-380 `#<8-char-csum>` suffix invariant via the new `CheckedDescriptor<'_>` newtype in `wallet_export/mod.rs`. Pre-v0.28.3 the invariant was documented at `wallet_export/bsms.rs:86-90` and enforced only by convention at construction sites — a future code path that constructed `EmitInputs` from a stripped-body descriptor would silently regress the BSMS L2 + Specter `descriptor` JSON field + Green plaintext (latent class surfaced by F9 in the manual-v0.2.0 audit cycle). Closes FOLLOWUP `emitinputs-canonical-descriptor-checksum-invariant-enforcement`. No CLI surface change; no GUI lockstep.

### Added

- `CheckedDescriptor<'a>(&'a str)` newtype in `wallet_export/mod.rs` with:
  - `new(desc: &'a str) -> Result<Self, ToolkitError>` constructor that validates the BIP-380 `#<8-char-csum>` suffix.
  - `as_str()` accessor.
  - `Deref<Target = str>` impl so existing consumer code (`let line2 = inputs.canonical_descriptor`) continues to work via auto-deref.
  - `Display` impl for `format!("{}", checked)` ergonomics.

### Changed

- `EmitInputs.canonical_descriptor` field type from `&'a str` → `CheckedDescriptor<'a>` (compile-time invariant guarantee).
- `cmd/export_wallet.rs` two construction sites (the `--template`/`--descriptor` path at `run()` and the `--from-import-json` path at `run_from_import_json()`) wrap their canonical-descriptor local via `CheckedDescriptor::new(...)?` before `EmitInputs` construction.
- `wallet_export/bsms.rs:86-90` invariant comment updated from "by convention" to "by type".

### Tests

- 5 new unit cells in `tests/checked_descriptor.rs` covering positive + 3 negative paths (missing `#`, wrong-length checksum, non-alphanumeric checksum) + Deref-coercion compat. Total toolkit cells: 1996 → ~2001.
```

- [ ] **Step 3: Bump scripts/install.sh:32 self-pin**

Edit `scripts/install.sh:32`:

Old:
```sh
            echo "mnemonic-toolkit|https://github.com/bg002h/mnemonic-toolkit|mnemonic-toolkit-v0.28.2|no|"
```

New:
```sh
            echo "mnemonic-toolkit|https://github.com/bg002h/mnemonic-toolkit|mnemonic-toolkit-v0.28.3|no|"
```

This bump is REQUIRED by the install-pin-check.yml CI gate which fires on `mnemonic-toolkit-v*` tag push and validates that `install.sh`'s self-pin matches the tag. Per v0.18.1 precedent (`project_v0_18_1_v0_7_2_b1_bugfix_closed`).

- [ ] **Step 4: Rebuild + verify binary version**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo build --bin mnemonic 2>&1 | tail -3
target/debug/mnemonic --version
```

Expected: `mnemonic 0.28.3`.

---

### Task 10: Flip FOLLOWUPS Status

**Files:**
- Modify: `design/FOLLOWUPS.md`

- [ ] **Step 1: Locate the FOLLOWUP entry**

Run:
```bash
grep -n '^### `emitinputs-canonical-descriptor-checksum-invariant-enforcement' /scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md
```

- [ ] **Step 2: Flip the Status field**

Old:
```markdown
- **Status:** open
```

New (where `<commit-sha>` is backfilled after Task 11 lands):
```markdown
- **Status:** `resolved <commit-sha>` — v0.28.3 cycle landed `CheckedDescriptor<'_>` newtype in `wallet_export/mod.rs` enforcing the BIP-380 `#<csum>` invariant at compile time. `EmitInputs.canonical_descriptor` field type changed from `&str` → `CheckedDescriptor<'_>`; 2 construction sites in `cmd/export_wallet.rs` updated; 5 new test cells in `tests/checked_descriptor.rs` lock the constructor contract.
```

Defer the actual edit until Task 11 staging.

---

### Task 11: Commit + tag + push

**Files:** all modified files staged.

- [ ] **Step 1: Verify the working tree state**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git status --short
```

Expected modified files:
- `crates/mnemonic-toolkit/src/wallet_export/mod.rs`
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`
- `crates/mnemonic-toolkit/src/wallet_export/bsms.rs`
- `crates/mnemonic-toolkit/Cargo.toml`
- `Cargo.lock` (from cargo build)
- `CHANGELOG.md`
- `scripts/install.sh`
- `design/FOLLOWUPS.md`

New file:
- `crates/mnemonic-toolkit/tests/checked_descriptor.rs`

- [ ] **Step 2: Stage explicit paths**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git add crates/mnemonic-toolkit/src/wallet_export/mod.rs \
        crates/mnemonic-toolkit/src/cmd/export_wallet.rs \
        crates/mnemonic-toolkit/src/wallet_export/bsms.rs \
        crates/mnemonic-toolkit/tests/checked_descriptor.rs \
        crates/mnemonic-toolkit/Cargo.toml \
        Cargo.lock \
        CHANGELOG.md \
        scripts/install.sh \
        design/FOLLOWUPS.md
git diff --cached --stat
```

Expected: ~9 files changed; ~80-120 lines insertions; minimal deletions.

- [ ] **Step 3: Commit**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git commit -m "$(cat <<'EOF'
release(toolkit): mnemonic-toolkit v0.28.3 — CheckedDescriptor newtype + EmitInputs invariant enforcement

Closes FOLLOWUP `emitinputs-canonical-descriptor-checksum-invariant-enforcement`.

Compile-time enforcement of the `EmitInputs.canonical_descriptor`
BIP-380 `#<8-char-csum>` suffix invariant via the new
`CheckedDescriptor<'_>` newtype in `wallet_export/mod.rs`. Pre-v0.28.3
the invariant was enforced only by convention at construction sites
(documented at `wallet_export/bsms.rs:86-90`); a future code path
that built `EmitInputs` from a stripped-body descriptor would
silently regress BSMS L2 + Specter `descriptor` JSON field + Green
plaintext (latent class surfaced by F9 in manual-v0.2.0 cycle's
P1b R1 architect review).

The newtype carries `Deref<Target = str>` + `Display` so existing
consumer call sites (e.g., `bsms.rs:92` — `let line2 =
inputs.canonical_descriptor;`) continue to work via auto-deref. The
`new()` constructor returns `Result<Self, BadInput>` on missing /
wrong-length / non-alphanumeric checksum.

Cycle 1 of the A/B/C FOLLOWUP release plan; Wave 1 second ship after
manual-v0.2.1 (Cycle 2). Sonnet reviewer GREEN: 0 critical / 0
important.

Tests: 5 new unit cells in `tests/checked_descriptor.rs` (constructor
positive + 3 negative paths + Deref-coercion compat). Total toolkit
cells: 1996 → ~2001. F9 regression cells from v0.28.2 continue to
pass.

Tooling: Cargo.toml version 0.28.2 → 0.28.3; CHANGELOG entry;
scripts/install.sh:32 self-pin bumped (install-pin-check.yml CI gate
green on tag push).

No CLI surface change; no GUI lockstep.
EOF
)"
```

- [ ] **Step 4: Backfill the FOLLOWUPS Status SHA (if needed)**

If the Status flip in Task 10 used a placeholder for `<commit-sha>`, edit `design/FOLLOWUPS.md` to insert the actual SHA from `git rev-parse HEAD` and amend:

```bash
SHA=$(git rev-parse HEAD)
sed -i "s/resolved <commit-sha>/resolved $SHA/" design/FOLLOWUPS.md
git add design/FOLLOWUPS.md
git commit --amend --no-edit
```

(Skip if the Status flip was staged with the SHA already in Step 2.)

- [ ] **Step 5: Tag mnemonic-toolkit-v0.28.3**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git tag mnemonic-toolkit-v0.28.3
git tag -l 'mnemonic-toolkit-v0.28*'
```

Expected output:
```
mnemonic-toolkit-v0.28.0
mnemonic-toolkit-v0.28.1
mnemonic-toolkit-v0.28.2
mnemonic-toolkit-v0.28.3
```

- [ ] **Step 6: Push master + tag**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git push origin master
git push origin mnemonic-toolkit-v0.28.3
```

Expected: both pushes succeed; tag push triggers the `install-pin-check.yml` CI workflow.

---

### Task 12: Monitor CI + GH Release

**Files:** none modified.

- [ ] **Step 1: Monitor CI runs triggered by the push**

Use the `Monitor` tool with a poll script that watches `gh run list` for runs on master + `mnemonic-toolkit-v0.28.3`:

```bash
prev=""
while true; do
  s=$(gh run list --limit 4 --json databaseId,name,headBranch,status,conclusion 2>/dev/null || echo '[]')
  cur=$(jq -r '.[] | select(.headBranch == "master" or .headBranch == "mnemonic-toolkit-v0.28.3") | "\(.databaseId) \(.headBranch) \(.name): \(.status)/\(.conclusion // "-")"' <<<"$s" | sort)
  comm -13 <(echo "$prev") <(echo "$cur")
  prev=$cur
  remaining=$(jq -r '[.[] | select(.headBranch == "master" or .headBranch == "mnemonic-toolkit-v0.28.3") | select(.status != "completed")] | length' <<<"$s")
  [ "$remaining" = "0" ] && break
  sleep 30
done
```

Expected runs:
- `install-pin-check` on `mnemonic-toolkit-v0.28.3` tag — should PASS (10s; validates scripts/install.sh:32 matches the tag).
- `rust` on master — should PASS (~5-10 min; runs `cargo test`).
- `manual` on master — should PASS (~5-10 min; lint + verify-examples + PDF).

If `install-pin-check` FAILS, the scripts/install.sh:32 pin doesn't match the tag — most likely cause is a typo in Task 9 Step 3. Fix + amend + force-push tag (or delete + re-tag + push). Per memory `feedback-ci-snapshot-test-substring-vacuity`.

- [ ] **Step 2: Create the GH Release manually**

Convention per `project_v0_28_1_patch_shipped` + this session's v0.28.2 release: GH Releases for toolkit tags are created manually post-tag-push.

Run:
```bash
gh release create mnemonic-toolkit-v0.28.3 \
  --title 'mnemonic-toolkit v0.28.3 — CheckedDescriptor newtype + EmitInputs invariant enforcement' \
  --notes "$(cat <<'EOF'
Patch release: compile-time enforcement of the `EmitInputs.canonical_descriptor` BIP-380 `#<8-char-csum>` suffix invariant via a new `CheckedDescriptor<'_>` newtype in `wallet_export/mod.rs`. Pre-v0.28.3 the invariant was enforced only by convention at construction sites and documented at `wallet_export/bsms.rs:86-90`. A future code path that constructed `EmitInputs` from a stripped-body descriptor would silently regress the BSMS L2 + Specter `descriptor` JSON field + Green plaintext (latent class surfaced by F9 in the [manual-v0.2.0](https://github.com/bg002h/mnemonic-toolkit/releases/tag/manual-v0.2.0) audit cycle).

Closes FOLLOWUP \`emitinputs-canonical-descriptor-checksum-invariant-enforcement\`. No CLI surface change; no GUI lockstep.

### Added

- \`CheckedDescriptor<'a>(&'a str)\` newtype with \`new() -> Result\` constructor + \`as_str()\` + \`Deref<Target = str>\` + \`Display\`.

### Changed

- \`EmitInputs.canonical_descriptor\` field type from \`&str\` → \`CheckedDescriptor<'_>\`.
- \`cmd/export_wallet.rs\` two construction sites wrap via \`CheckedDescriptor::new(...)?\`.
- \`bsms.rs:86-90\` invariant comment updated from "by convention" to "by type".

### Tests

- 5 new unit cells in \`tests/checked_descriptor.rs\` (constructor positive + 3 negative + Deref compat). Total toolkit cells: 1996 → ~2001.

### Companion releases

- [manual-v0.2.1](https://github.com/bg002h/mnemonic-toolkit/releases/tag/manual-v0.2.1) — Wave 1 first; CI manual.yml now uses real md/ms binaries.
- Brainstorm: \`design/BRAINSTORM_followups_abc_release_plan.md\` (A/B/C release plan; Cycle 1 of 4).
EOF
)"
```

---

## Self-review

After completing all 12 tasks, verify against the brainstorm spec:

1. **Spec coverage check:**
   - Cycle 1 Phase 0 (design lock) → architectural decision baked into Tasks 2-4 (newtype + Deref)
   - Cycle 1 Phase 1 (TDD red) → Task 5 (tests written AFTER newtype impl per practical-TDD; would be Phase 1 in strict-TDD)
   - Cycle 1 Phase 2 (impl) → Tasks 2-4 ✓
   - Cycle 1 Phase 3 (bsms.rs comment update) → Task 6 ✓
   - Cycle 1 Phase 4 (TDD green + clippy + full suite) → Task 7 ✓
   - Cycle 1 Phase 5 (sonnet reviewer) → Task 8 ✓
   - Cycle 1 Phase 6 (commit + tag + push + GH Release) → Tasks 9, 11, 12 ✓
   - Cycle 1 Phase 7 (FOLLOWUPS Status flip) → Task 10 ✓
   - **Note:** Per architect I5, Task 9 Step 3 includes install.sh:32 self-pin bump (NOT mentioned in brainstorm but in cross-cutting concerns).

2. **No-placeholder check:** All Rust code is complete. The `<commit-sha>` template in Tasks 10-11 is backfilled in Task 11 Step 4. The `<YYYY-MM-DD>` template in Task 9 Step 2 is filled by the executor at commit time.

3. **Type consistency:** `CheckedDescriptor<'a>` is the canonical type name throughout. `new()` is the canonical constructor name. `as_str()` is the accessor. No drift across tasks.

4. **Effort estimate sanity-check:** ~1 hour per brainstorm. Tasks 1-4 (~15 min recon + impl); Task 5 (~10 min); Tasks 6-7 (~10 min); Task 8 (~10 min reviewer); Tasks 9-12 (~15 min release tooling). Realistic.

---

## Risk flags

- **Consumer-site auto-deref edge cases.** If `bsms.rs:92` or other consumer sites use `inputs.canonical_descriptor` in a context where Deref doesn't auto-fire (e.g., `&inputs.canonical_descriptor` taking a borrow of the field rather than the deref'd `&str`), compile errors surface. Task 4 Step 5 has a fallback pattern (`inputs.canonical_descriptor.as_str()`). The most likely affected files are `wallet_export/{bsms,specter,green}.rs`; spot-check during Task 4.

- **`CheckedDescriptor::new` strictness vs miniscript-emitted checksums.** miniscript's `Descriptor::Display` always appends `#<8-char>` per BIP-380, so the F9 fix at v0.28.2 (`parsed_ms.to_string()`) always produces a checksum-suffixed string. The newtype constructor MUST accept whatever miniscript emits. The 4 negative-test cells in Task 5 cover the cases the constructor SHOULD reject; they DON'T need to mirror miniscript's emit grammar exactly. If miniscript ever emits a `#<8-char>` containing characters outside ASCII-alphanumeric (highly unlikely per BIP-380 lowercase-base32 grammar), the test for "non-alphanumeric rejection" may need narrowing.

- **`tests/checked_descriptor.rs` integration-test convention.** This file is a NEW integration-test target. Verify that `cargo test --test checked_descriptor` discovers and runs it. The existing test files in `crates/mnemonic-toolkit/tests/` follow the same `tests/<name>.rs` pattern; this should just work without `[[test]]` entries in Cargo.toml. If the test isn't discovered, check whether Cargo.toml has `autotests = false` (it shouldn't).

- **Sub-skill expectations.** This plan assumes the executor uses `superpowers:subagent-driven-development` or `superpowers:executing-plans`.
