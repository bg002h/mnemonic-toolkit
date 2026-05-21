# mnemonic-toolkit-v0.30.1 Implementation Plan (Cycle 6b — electrum-encrypted watch-only passthrough)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Ship `mnemonic-toolkit-v0.30.1` (SemVer-PATCH; behavior expansion). Replace the encrypted-Electrum-wallet refusal at `wallet_import/electrum.rs:305-313` with a watch-only-passthrough advisory. Closes FOLLOWUP `wallet-import-electrum-encrypted` (resolved as watch-only-passthrough per Cycle 6b R0 fold).

**Architecture:** No new CLI flags. No new modules. Single-point refactor of the existing `parse` function in `wallet_import/electrum.rs` + integration test additions + manual chapter update. The 6a-shipped `electrum_crypto.rs` library stays in-tree as an internal utility for a future seed-extraction subcommand (filed as FOLLOWUP).

**Tech Stack:** Rust + clap-derive (no surface change) + serde_json. No new Cargo deps.

**Brainstorm:** `design/BRAINSTORM_v0_30_1_electrum_encrypted_watch_only.md` (v2; v1 Path-B framing archived at `BRAINSTORM_v0_31_0_electrum_encrypted_v1_path_b.md`).

**R0 review:** `design/agent-reports/v0_31_0-brainstorm-r0-review.md` (verdict RED; Path A folded into v2 brainstorm).

**Source SHA at plan-write time:** `d890de4` (post-Cycle-6a artifacts).

**P0 STRICT-GATE locks (per brainstorm v2 + R0):**

- Single refusal site to refactor: `wallet_import/electrum.rs:305-313` (no other `use_encryption` gates in the parser or sniff).
- Parser-needed plaintext fields: `keystore.xpub` / `keystore.derivation` / `keystore.root_fingerprint` / `keystore.label` (singlesig at L494/504/514/531); `xN/.xpub` / etc. (multisig at L778-816). All plaintext under both encrypted and unencrypted Electrum wallets.
- Encrypted fields ignored at parse: `keystore.seed` / `keystore.xprv` / `keystore.passphrase` / `keystore.keypairs` (singlesig); same paths under `xN/` (multisig).
- Stderr advisory template: `"import-wallet: electrum: wallet is encrypted (use_encryption=true); importing watch-only material only (encrypted seed/xprv/passphrase/keypairs fields ignored). To extract the encrypted seed, use 'electrum --decrypt-wallet' out-of-band then re-import the plaintext wallet."`
- SemVer: PATCH `v0.30.0 → v0.30.1`. NO GUI lockstep (no clap surface change; `gui-schema` JSON unchanged).
- No new `ToolkitError` variants. No `secrets.rs` updates.

**SemVer policy:** PATCH per project precedent for non-breaking behavior expansion (formerly-refused inputs now succeed; no flag additions). No GUI tag bump.

---

## File structure

### Source files modified (toolkit)

- `crates/mnemonic-toolkit/src/wallet_import/electrum.rs:305-313` — refusal → advisory + parse-continuation.
- `crates/mnemonic-toolkit/src/wallet_import/electrum.rs:47` — module docstring: update "Refusals" list to remove `use_encryption: true` (now an advisory, not a refusal).
- `crates/mnemonic-toolkit/src/wallet_import/electrum.rs:258` — `parse` function docstring: update Step 3 description.

### Test files added (toolkit)

- `crates/mnemonic-toolkit/tests/cli_import_wallet_electrum_encrypted.rs` — NEW; ~12-15 cells covering: singlesig + multisig encrypted-wallet happy paths; stderr advisory presence + byte-match; missing-xpub-in-encrypted-wallet still refused; sniff still positive (no change); plaintext wallet still works (no regression).

### Documentation modified (toolkit)

- `docs/manual/src/45-foreign-formats.md` — §"Encrypted Electrum wallets" subsection: drop deferred framing; document watch-only-passthrough semantic + out-of-band `electrum --decrypt-wallet` workflow.
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — `## mnemonic import-wallet` section's stderr-template list: add the new advisory template.

### Release tooling

- `crates/mnemonic-toolkit/Cargo.toml:3` — version `0.30.0` → `0.30.1`.
- `CHANGELOG.md` — new `## [0.30.1]` section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.30.0` → `mnemonic-toolkit-v0.30.1`.
- `design/FOLLOWUPS.md` — close 1 slug + file 2 new slugs (per brainstorm decision item 8).

### NOT modified

- `crates/mnemonic-toolkit/src/electrum_crypto.rs` — kept in-tree as 6a delivered. Library remains unused by CLI (filed as FOLLOWUP `electrum-crypto-seed-extraction-subcommand`).
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — no clap surface change.
- `crates/mnemonic-toolkit/src/secrets.rs` — no flag-secret-classification change.
- `crates/mnemonic-toolkit/Cargo.toml` deps — `aes` / `base64` / `cbc` retained (6a additions; needed for the library module's compilation).
- `mnemonic-gui/*` — no changes; `schema_mirror` test continues to pass against v0.30.1.

---

## Tasks

### Task 1: Phase 2 — Refactor `electrum.rs:305-313` refusal → advisory

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/electrum.rs`

- [ ] **Step 1: Read the current refusal site**

```bash
sed -n '300,320p' crates/mnemonic-toolkit/src/wallet_import/electrum.rs
```

Expected: the `if use_encryption { return Err(...) }` block at L305-313.

- [ ] **Step 2: Replace refusal with advisory**

Replace:
```rust
        // Step 3: use_encryption refusal per §11.6.1.
        let use_encryption = obj
            .get("use_encryption")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if use_encryption {
            return Err(ToolkitError::ImportWalletParse(
                "import-wallet: electrum: encrypted wallet files require decrypting via 'electrum --decrypt-wallet' first; encrypted ingest not yet supported (FOLLOWUP wallet-import-electrum-encrypted)"
                    .to_string(),
            ));
        }
```

With:
```rust
        // Step 3: use_encryption advisory (v0.30.1 / Cycle 6b watch-only-passthrough).
        //
        // Per electrum/keystore.py (verified at Cycle 6 P0 recon §A1 + R0 §C1),
        // Electrum's field-level encryption protects `keystore.{seed,xprv,
        // passphrase,keypairs}`. The fields THIS parser reads
        // (`keystore.{xpub,derivation,root_fingerprint,label}` + multisig
        // analogues) are plaintext under both encrypted AND unencrypted wallets.
        // The encrypted-wallet refusal v0.28.0 shipped was therefore over-
        // restrictive: watch-only import has all the material it needs without
        // touching the encrypted fields. v0.30.1 downgrades the refusal to a
        // stderr advisory + continues with the plaintext xpub/derivation/etc.
        let use_encryption = obj
            .get("use_encryption")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if use_encryption {
            let _ = writeln!(
                stderr,
                "import-wallet: electrum: wallet is encrypted (use_encryption=true); \
                 importing watch-only material only (encrypted seed/xprv/passphrase/keypairs \
                 fields ignored). To extract the encrypted seed, use 'electrum --decrypt-wallet' \
                 out-of-band then re-import the plaintext wallet."
            );
        }
```

- [ ] **Step 3: Update module docstring at L47**

Replace:
```rust
//! Refusals (`2fa` / `imported` / `use_encryption: true`) per SPEC §11.6.1.
```

With:
```rust
//! Refusals (`2fa` / `imported`) per SPEC §11.6.1. Encrypted wallets
//! (`use_encryption: true`) are imported as watch-only at v0.30.1+ per
//! design/BRAINSTORM_v0_30_1_electrum_encrypted_watch_only.md.
```

- [ ] **Step 4: Update `parse` function docstring at L258**

Find the existing docstring line:
```rust
    /// 3. If `use_encryption: true` → REFUSE per §11.6.1.
```

Replace with:
```rust
    /// 3. If `use_encryption: true` → stderr ADVISORY (watch-only-passthrough
    ///    per v0.30.1 / Cycle 6b R0 fold; see brainstorm v2 for rationale).
    ///    The parser continues with the plaintext xpub/derivation/fingerprint/
    ///    label fields the parser actually reads; encrypted seed/xprv/passphrase/
    ///    keypairs are ignored.
```

- [ ] **Step 5: Update any existing unit tests in electrum.rs that asserted the refusal**

Grep for use_encryption tests:
```bash
grep -n "use_encryption.*true\|encrypted wallet files require" crates/mnemonic-toolkit/src/wallet_import/electrum.rs | head -10
```

Update any test that asserts the refusal behavior; replace with an assertion that the import succeeds + an assertion on the stderr advisory (test cells that previously expected `Err(ToolkitError::ImportWalletParse(_))` for encrypted wallets now expect `Ok(_)` + captured stderr contains the advisory substring).

- [ ] **Step 6: Build + run the lib tests for electrum**

```bash
cargo build --package mnemonic-toolkit 2>&1 | tail -5
cargo test --package mnemonic-toolkit --lib electrum 2>&1 | tail -10
```

Expected: clean build + lib tests pass (any test-cell updates from Step 5 hold).

- [ ] **Step 7: Commit Phase 2**

```bash
git add crates/mnemonic-toolkit/src/wallet_import/electrum.rs
git commit -m "feat(electrum): v0.30.1 — encrypted-wallet watch-only passthrough

Replaces the v0.28.0 encrypted-wallet refusal at wallet_import/electrum.rs:305-313
with a stderr advisory. The parser reads only plaintext fields
(keystore.{xpub,derivation,root_fingerprint,label}); encrypted Electrum
fields (keystore.{seed,xprv,passphrase,keypairs}) are never touched. Per
Cycle 6b R0 review the original refusal was over-restrictive in principle.

Phase 2 of design/PLAN_mnemonic_toolkit_v0_30_1.md."
```

---

### Task 2: Phase 3 — Integration tests

**Files:**
- Create: `crates/mnemonic-toolkit/tests/cli_import_wallet_electrum_encrypted.rs`

- [ ] **Step 1: Construct test fixtures**

Generate test fixtures by taking an existing plaintext Electrum wallet JSON fixture and toggling `use_encryption: true` + replacing the sensitive fields with arbitrary-base64-ciphertext placeholders. The plaintext xpub/derivation/fingerprint stays. (Real Electrum-encrypted wallet generation would require Python + a wallet password; the parse path doesn't validate that the ciphertext IS valid base64 since we never decrypt — placeholder strings are fine.)

Sample fixtures:

`crates/mnemonic-toolkit/tests/fixtures/wallet_import/electrum-encrypted-singlesig-watch-only.json`:
```json
{
  "seed_version": 41,
  "wallet_type": "standard",
  "use_encryption": true,
  "keystore": {
    "type": "bip32",
    "xpub": "zpub6qg4U6tF4PdaqyJgxoH9...",
    "derivation": "m/84'/0'/0'",
    "root_fingerprint": "abcd1234",
    "label": "Test Encrypted Wallet",
    "seed": "ENCRYPTED_BASE64_PLACEHOLDER==",
    "xprv": "ENCRYPTED_BASE64_PLACEHOLDER=="
  }
}
```

(Use a real `zpub...` from existing plaintext fixtures to ensure parser-side acceptance.)

`crates/mnemonic-toolkit/tests/fixtures/wallet_import/electrum-encrypted-multisig-2of2-watch-only.json`:
similar shape with `wallet_type: "2of2"` + per-cosigner `x1/`, `x2/` keys.

- [ ] **Step 2: Author `cli_import_wallet_electrum_encrypted.rs`**

```rust
//! v0.30.1 — encrypted Electrum wallet watch-only passthrough.
//! Cycle 6b verifies the L305-313 refusal-to-advisory downgrade.

use assert_cmd::Command;
use std::path::PathBuf;

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wallet_import")
        .join(name)
}

// ─── Happy paths ─────────────────────────────────────────────────────────

#[test]
fn encrypted_singlesig_imports_watch_only() {
    let fixture = fixture_path("electrum-encrypted-singlesig-watch-only.json");
    mnemonic()
        .args(["import-wallet", "--format", "electrum"])
        .arg(&fixture)
        .args(["--json"])
        .assert()
        .success();
}

#[test]
fn encrypted_singlesig_emits_advisory_on_stderr() {
    let fixture = fixture_path("electrum-encrypted-singlesig-watch-only.json");
    mnemonic()
        .args(["import-wallet", "--format", "electrum"])
        .arg(&fixture)
        .args(["--json"])
        .assert()
        .success()
        .stderr(predicates::str::contains("wallet is encrypted"))
        .stderr(predicates::str::contains("watch-only material only"))
        .stderr(predicates::str::contains("electrum --decrypt-wallet"));
}

#[test]
fn encrypted_multisig_imports_watch_only() {
    let fixture = fixture_path("electrum-encrypted-multisig-2of2-watch-only.json");
    mnemonic()
        .args(["import-wallet", "--format", "electrum"])
        .arg(&fixture)
        .args(["--json"])
        .assert()
        .success();
}

#[test]
fn encrypted_multisig_emits_advisory_on_stderr() {
    let fixture = fixture_path("electrum-encrypted-multisig-2of2-watch-only.json");
    mnemonic()
        .args(["import-wallet", "--format", "electrum"])
        .arg(&fixture)
        .args(["--json"])
        .assert()
        .success()
        .stderr(predicates::str::contains("wallet is encrypted"));
}

// ─── Refusals + edge cases ───────────────────────────────────────────────

#[test]
fn encrypted_wallet_with_missing_xpub_still_refuses() {
    // Encrypted wallet missing keystore.xpub (the parser's required plaintext
    // field). The L305-313 advisory fires, then the parser hits the existing
    // "keystore.xpub missing" refusal at electrum.rs:497.
    let fixture = fixture_path("electrum-encrypted-singlesig-no-xpub.json");
    mnemonic()
        .args(["import-wallet", "--format", "electrum"])
        .arg(&fixture)
        .assert()
        .failure()
        .stderr(predicates::str::contains("wallet is encrypted"))
        .stderr(predicates::str::contains("keystore.xpub missing"));
}

#[test]
fn plaintext_wallet_no_regression() {
    // Sanity: plaintext wallet still imports successfully + emits NO advisory.
    let fixture = fixture_path("electrum-standard-plaintext.json");
    let out = mnemonic()
        .args(["import-wallet", "--format", "electrum"])
        .arg(&fixture)
        .args(["--json"])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("wallet is encrypted"),
        "plaintext wallet must NOT emit the encrypted-advisory; got stderr: {stderr}"
    );
}

#[test]
fn encrypted_wallet_sniff_still_positive() {
    // Sniff path (called via --format-auto-detect): use_encryption=true wallet
    // should still sniff-positive as electrum so the parser is dispatched.
    let fixture = fixture_path("electrum-encrypted-singlesig-watch-only.json");
    // No --format flag → auto-sniff.
    mnemonic()
        .args(["import-wallet"])
        .arg(&fixture)
        .args(["--json"])
        .assert()
        .success();
}

// ─── 2fa / imported / non-encryption refusals still fire ─────────────────

#[test]
fn imported_wallet_still_refused() {
    // The `imported` wallet_type refusal at SPEC §11.6.1 still fires
    // independent of use_encryption.
    let fixture = fixture_path("electrum-imported-refused.json"); // existing fixture
    mnemonic()
        .args(["import-wallet", "--format", "electrum"])
        .arg(&fixture)
        .assert()
        .failure();
}

#[test]
fn twofa_wallet_still_refused() {
    let fixture = fixture_path("electrum-2fa-refused.json"); // existing fixture
    mnemonic()
        .args(["import-wallet", "--format", "electrum"])
        .arg(&fixture)
        .assert()
        .failure();
}
```

Cell count: 9 (above the brainstorm's "~12-15" target undershoot is acceptable since the advisory path is simpler than v1's password-flag-family + secret-hygiene matrix).

- [ ] **Step 2.5: Create the test fixtures actually referenced**

Construct the 3 new fixtures (`electrum-encrypted-singlesig-watch-only.json`, `electrum-encrypted-multisig-2of2-watch-only.json`, `electrum-encrypted-singlesig-no-xpub.json`) by copying an existing plaintext fixture + toggling `use_encryption: true` + replacing `seed`/`xprv` with placeholder base64. Use real `zpub`/`xpub` values from the existing plaintext fixtures for the xpub field.

```bash
ls crates/mnemonic-toolkit/tests/fixtures/wallet_import/electrum-* | head
```

Find existing fixtures and base the new ones on them.

- [ ] **Step 3: Run integration tests**

```bash
cargo test --package mnemonic-toolkit --test cli_import_wallet_electrum_encrypted 2>&1 | tail -15
```

Expected: 9 cells PASS.

- [ ] **Step 4: Run full toolkit suite**

```bash
cargo test --package mnemonic-toolkit 2>&1 | tail -5
```

Expected: no regressions.

- [ ] **Step 5: Commit Phase 3**

```bash
git add crates/mnemonic-toolkit/tests/cli_import_wallet_electrum_encrypted.rs crates/mnemonic-toolkit/tests/fixtures/wallet_import/electrum-encrypted-*.json
git commit -m "test(electrum): v0.30.1 — encrypted-wallet watch-only-passthrough integration suite

9 cells covering: singlesig + multisig encrypted wallets import watch-only;
stderr advisory presence + byte-match on key substrings; missing-xpub
still refuses (advisory fires before existing refusal); plaintext-wallet
no regression; sniff still positive; pre-existing 2fa/imported refusals
preserved.

Phase 3 of design/PLAN_mnemonic_toolkit_v0_30_1.md."
```

---

### Task 3: Phase 4 — Manual chapter updates

**Files:**
- Modify: `docs/manual/src/45-foreign-formats.md` (§"Encrypted Electrum wallets" or wherever the deferred-section currently lives)
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md` (`## mnemonic import-wallet` stderr-template list)

- [ ] **Step 1: Locate chapter-45 §"Encrypted Electrum wallets"**

```bash
grep -n "encrypted\|Electrum.*encrypt\|wallet-import-electrum-encrypted" docs/manual/src/45-foreign-formats.md | head -10
```

- [ ] **Step 2: Rewrite the chapter-45 subsection**

Drop "deferred" framing. Replace with watch-only-passthrough documentation:

```markdown
### Encrypted Electrum wallets (watch-only passthrough; v0.30.1+)

Electrum wallets with `use_encryption: true` are imported as **watch-only**
in v0.30.1+. The toolkit reads only the plaintext xpub/derivation/fingerprint
fields (which are NOT encrypted even when the user has set a wallet
password); the encrypted `seed`/`xprv`/`passphrase`/`keypairs` fields are
ignored.

The toolkit emits a stderr advisory on import:

```
import-wallet: electrum: wallet is encrypted (use_encryption=true);
importing watch-only material only (encrypted seed/xprv/passphrase/keypairs
fields ignored). To extract the encrypted seed, use 'electrum --decrypt-wallet'
out-of-band then re-import the plaintext wallet.
```

**To extract the encrypted seed** (e.g., to re-engrave the seed via `mnemonic
bundle`), decrypt the wallet out-of-band via Electrum:

```bash
electrum --decrypt-wallet --wallet-path <path>
```

This produces a plaintext wallet file the toolkit can then import without
the advisory.

**Why watch-only-passthrough?** Per Electrum's `electrum/keystore.py`, the
field-level encryption (`pw_encode_bytes`) protects only the SEED-material
fields (`seed`, `xprv`, `passphrase`, `keypairs`). The watch-only fields
(`xpub`, `derivation`, `root_fingerprint`, `label`) are plaintext under
both encrypted and unencrypted wallets. The pre-v0.30.1 refusal was
over-restrictive in principle.
```

- [ ] **Step 3: Add stderr template to chapter-41**

In `docs/manual/src/40-cli-reference/41-mnemonic.md`, find the `## mnemonic import-wallet` section's stderr-template list (search for existing templates like `"import-wallet: bsms:"`). Add:

```markdown
- `import-wallet: electrum: wallet is encrypted (use_encryption=true); importing watch-only material only ...`
  (advisory only; parse continues with plaintext xpub/derivation/fingerprint/label; v0.30.1+)
```

- [ ] **Step 4: Run manual lint**

```bash
make -C docs/manual lint MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic 2>&1 | tail -10
```

Expected: lint PASS (no clap-surface changes).

- [ ] **Step 5: Architect-must-run-prose-commands check**

The `electrum --decrypt-wallet` recipe in the manual is an out-of-band reference (user-side; not something the toolkit runs). Skip the literal command-run for that one. Verify the stderr advisory template byte-matches the implementation by running the encrypted-fixture test:

```bash
./target/debug/mnemonic import-wallet --format electrum crates/mnemonic-toolkit/tests/fixtures/wallet_import/electrum-encrypted-singlesig-watch-only.json --json 2>&1 1>/dev/null | head -5
```

Expected: the advisory text exactly matches the prose's quoted block.

- [ ] **Step 6: Commit Phase 4**

```bash
git add docs/manual/src/40-cli-reference/41-mnemonic.md docs/manual/src/45-foreign-formats.md
git commit -m "docs(electrum): v0.30.1 — encrypted-wallet watch-only passthrough chapters

Chapter-45 §'Encrypted Electrum wallets' rewritten from deferred-status
framing to v0.30.1 watch-only-passthrough documentation, including the
'electrum --decrypt-wallet' out-of-band recipe for seed extraction.
Chapter-41 §'mnemonic import-wallet' stderr-template list gains the new
advisory template.

Phase 4 of design/PLAN_mnemonic_toolkit_v0_30_1.md."
```

---

### Task 4: Phase 5 — Cycle close (version bump + tag + push)

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml:3`
- Modify: `CHANGELOG.md`
- Modify: `scripts/install.sh:32`

- [ ] **Step 1: Bump version**

`crates/mnemonic-toolkit/Cargo.toml:3`: `0.30.0` → `0.30.1`.

- [ ] **Step 2: Bump install.sh self-pin**

`scripts/install.sh:32`: `mnemonic-toolkit-v0.30.0` → `mnemonic-toolkit-v0.30.1`.

- [ ] **Step 3: CHANGELOG entry**

```markdown
## mnemonic-toolkit [0.30.1] — 2026-MM-DD

**SemVer-PATCH release.** Behavior expansion: encrypted Electrum wallets (`use_encryption: true`) now import as watch-only instead of refusing at parse time. Closes Cycle 6 (`wallet-import-electrum-encrypted` FOLLOWUP, resolved as watch-only-passthrough).

### Changed

- **`mnemonic import-wallet --format electrum`** with `use_encryption: true` wallets now succeeds, emitting a stderr advisory and importing only the plaintext watch-only material (xpub/derivation/fingerprint/label). Previously refused at parse time. The encrypted fields (`seed`/`xprv`/`passphrase`/`keypairs`) are ignored.
- Per Electrum's `electrum/keystore.py`, the field-level encryption protects only seed-material fields; watch-only fields are plaintext under both encrypted and unencrypted wallets. The pre-v0.30.1 refusal was over-restrictive.
- Stderr advisory text: `"import-wallet: electrum: wallet is encrypted (use_encryption=true); importing watch-only material only (encrypted seed/xprv/passphrase/keypairs fields ignored). To extract the encrypted seed, use 'electrum --decrypt-wallet' out-of-band then re-import the plaintext wallet."`

### Added

- 9 new integration-test cells in `tests/cli_import_wallet_electrum_encrypted.rs`.
- Manual chapter-45 §"Encrypted Electrum wallets" rewritten (drops deferred framing).

### Architectural pivot (Cycle 6 R0 fold)

The 6a brainstorm assumed the toolkit needed to decrypt seed/xprv fields. Cycle 6b R0 opus review caught that the parser reads only plaintext xpub/derivation/fingerprint/label — encrypted fields are NEVER consumed. The `--decrypt-password*` flag family (3-form: `--decrypt-password VAL` + `--decrypt-password-file PATH` + `--decrypt-password-stdin`) and supporting machinery were dropped. The 6a-shipped `electrum_crypto.rs` library stays in-tree as an internal utility for a future seed-extraction subcommand (filed forward as FOLLOWUP `electrum-crypto-seed-extraction-subcommand`). No CLI surface change in v0.30.1 → no GUI lockstep.

### FOLLOWUP closure

- **Closed (resolved-watch-only-passthrough):** `wallet-import-electrum-encrypted` (Cycle 6b R0 reinterpreted as watch-only-passthrough; the FOLLOWUP body's pre-v0.30.0 "PBKDF2 + AES-CBC" claim corrected to "sha256d + AES-256-CBC" per Cycle 6 P0 recon §A1).

### Newly filed FOLLOWUPs

- `electrum-crypto-seed-extraction-subcommand` — future use case for the 6a-shipped library (e.g., `mnemonic convert --from electrum-encrypted-wallet --to phrase` or a dedicated subcommand).
- `wallet-import-electrum-encrypted-storage-format-b` — Electrum's Format B whole-file storage encryption (version-byte + AES-CBC + 4-byte MAC).

### Note

Cycle 6 of v0.28+ residual FOLLOWUP release plan, executed as two-session split (6a: library + design; 6b: R0 fold + watch-only-passthrough + ship). Opus brainstorm R0 caught a foundational design error in 6a (RED verdict; Path A pivot). Plan-doc R0 + R1 verification follows.
```

- [ ] **Step 4: Full audit before commit**

```bash
cargo test --workspace 2>&1 | tail -3
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -3
```

Expected: GREEN.

- [ ] **Step 5: Commit + tag + push**

```bash
git add crates/mnemonic-toolkit/Cargo.toml scripts/install.sh CHANGELOG.md Cargo.lock
git commit -m "release(toolkit): mnemonic-toolkit v0.30.1 — encrypted Electrum watch-only passthrough

Behavior-expansion PATCH closing Cycle 6 (wallet-import-electrum-encrypted
FOLLOWUP, resolved as watch-only-passthrough). The v0.28.0 refusal at
wallet_import/electrum.rs:305-313 was over-restrictive in principle: the
parser reads only plaintext xpub/derivation/fingerprint/label fields,
which are NOT encrypted even when use_encryption=true. Encrypted Electrum
wallets now import as watch-only with a stderr advisory.

Architectural pivot from Cycle 6a's --decrypt-password* design per opus
R0 (RED → Path A fold). 6a-shipped electrum_crypto.rs library stays
in-tree as internal helper for future seed-extraction subcommand (filed
as FOLLOWUP).

NO CLI surface change. NO GUI lockstep. NO new ToolkitError variants.

See design/PLAN_mnemonic_toolkit_v0_30_1.md + design/BRAINSTORM_v0_30_1_electrum_encrypted_watch_only.md."

git tag mnemonic-toolkit-v0.30.1
git push origin master
git push origin mnemonic-toolkit-v0.30.1
```

- [ ] **Step 6: Verify install-pin-check CI**

```bash
gh run list --limit 5 --json status,conclusion,name,headBranch | jq '.[] | select(.name|test("install-pin"))'
```

Wait for `conclusion: success`.

- [ ] **Step 7: GH Release**

```bash
awk '/^## mnemonic-toolkit \[0\.30\.1\]/,/^## mnemonic-toolkit \[0\.30\.0\]/' CHANGELOG.md | head -n -1 > /tmp/v0_30_1_release_notes.md
gh release create mnemonic-toolkit-v0.30.1 \
  --title "mnemonic-toolkit-v0.30.1 — encrypted Electrum watch-only passthrough" \
  --notes-file /tmp/v0_30_1_release_notes.md
```

---

### Task 5: Phase 6 — FOLLOWUP closure

**Files:**
- Modify: `design/FOLLOWUPS.md`

- [ ] **Step 1: Close `wallet-import-electrum-encrypted`**

Find the slug entry (cited at P0 §A2 as around L2572-2583; verify line numbers at write time):

```bash
grep -n "^### .wallet-import-electrum-encrypted" design/FOLLOWUPS.md | head
```

Update the entry:
- Status: `open` → `resolved (watch-only-passthrough per Cycle 6b R0; v0.30.1)`
- Add `**Resolved by:** Cycle 6 / mnemonic-toolkit-v0.30.1 (<tag SHA>)` line.
- Correct the "PBKDF2 + AES-CBC" claim in the body to "sha256d + AES-256-CBC" with note: "Pre-v0.30.0 body cited PBKDF2; this was wrong (verified against electrum/crypto.py at Cycle 6 P0 recon §A1; corrected in this entry at Cycle 6b close)."
- Add Architectural-pivot note: "Cycle 6b R0 opus review caught that the parser reads only plaintext xpub/derivation/fingerprint/label fields — encrypted seed/xprv/passphrase/keypairs are never consumed. Original `--decrypt-password*` design dropped; replaced with watch-only-passthrough advisory."

- [ ] **Step 2: File new FOLLOWUP `electrum-crypto-seed-extraction-subcommand`**

Append to FOLLOWUPS.md:

```markdown
### `electrum-crypto-seed-extraction-subcommand` — future use of v0.30.1's electrum_crypto library for seed extraction

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.30.1 Cycle 6b close (Path A R0 fold made the 6a-shipped library unused-by-CLI).
- **Where:** `crates/mnemonic-toolkit/src/electrum_crypto.rs` (library shipped Cycle 6a; 18 unit cells + cross-impl smoke). Currently unreferenced by any CLI module.
- **What:** Surface a new CLI consumer for the `electrum_crypto::decrypt_field` primitive. Two candidate shapes: (a) extend `mnemonic convert` with a new `--from electrum-encrypted-seed=<base64>` source; (b) dedicated `mnemonic electrum-decrypt` subcommand taking an encrypted seed string + password and emitting the plaintext seed. The library's `derive_key` + `decrypt_field` + `encrypt_field` (symmetric) are all production-ready; only the CLI integration is missing.
- **Why deferred:** Cycle 6 was reinterpreted as watch-only-passthrough (no decryption needed for import-wallet). The library is correct but has no user-visible surface yet. A future cycle ships the consumer when the seed-extraction use case is prioritized.
- **Status:** `open`
- **Tier:** `v0.31+`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-electrum-encrypted` (resolved v0.30.1 as watch-only-passthrough).
```

- [ ] **Step 3: File new FOLLOWUP `wallet-import-electrum-encrypted-storage-format-b`**

```markdown
### `wallet-import-electrum-encrypted-storage-format-b` — Electrum Format B whole-file storage encryption

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.30.1 Cycle 6b close (Cycle 6 P0 recon §A2 distinguished Format A field-level encryption (in-scope, now resolved as watch-only-passthrough) from Format B whole-file storage encryption (out-of-scope)).
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/electrum.rs` (sniff + parse pipeline). Format B wallets are NOT JSON-parseable at the file level; the current parser would fail at JSON parse with a different error.
- **What:** Per Electrum's `electrum/crypto.py::pw_encode_with_version_and_mac`, Format B encrypts the ENTIRE wallet file body (not just sensitive fields). Wire format: `base64(version_byte || iv (16 bytes) || aes_cbc(plaintext + PKCS7, key, iv) || mac (4 bytes))` where MAC = `sha256(plaintext)[:4]`. To support Format B, the toolkit would need: (a) sniff to detect a non-JSON-parseable base64 blob as candidate Format B; (b) decrypt-and-parse-as-JSON pipeline; (c) password-input CLI surface (the very `--decrypt-password*` family Cycle 6b dropped, but now justified by an actual need). Format B + Format A field-encryption together would cover Electrum's full encryption surface.
- **Why deferred:** Cycle 6 scope was Format A only. Format B requires the password-flag infrastructure; the v0.30.1 fix only addressed the field-level case.
- **Status:** `open`
- **Tier:** `v0.31+`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-electrum-encrypted` (Format A resolved v0.30.1 as watch-only-passthrough; this is the Format B carve-out).
```

- [ ] **Step 4: Commit + push**

```bash
git add design/FOLLOWUPS.md
git commit -m "design(cycle-6b-close): FOLLOWUP closure + 2 new slugs

Closes wallet-import-electrum-encrypted (resolved as watch-only-passthrough
per Cycle 6b R0 fold; v0.30.1). FOLLOWUP body's pre-v0.30.0 'PBKDF2 + AES-CBC'
claim corrected to 'sha256d + AES-256-CBC' per Cycle 6 P0 recon §A1.

Files 2 new FOLLOWUPs:
- electrum-crypto-seed-extraction-subcommand (future use case for the
  6a-shipped library; currently unused-by-CLI under Path A).
- wallet-import-electrum-encrypted-storage-format-b (Electrum Format B
  whole-file encryption; out-of-scope of Cycle 6's Format A focus)."

git push origin master
```

- [ ] **Step 5: Update memory**

Add `project_v0_30_1_cycle_6b_shipped.md` memory entry summarizing the cycle outcome + lessons (especially the Path B → Path A pivot caught by opus R0). Update the MEMORY.md index.

---

## Cross-phase invariants

- **No new code without R0 verification.** This plan-doc itself should be opus-R0-reviewed before Phase 2 dispatch. Persist R0 verbatim at `design/agent-reports/v0_30_1-plan-doc-r0-review.md`.
- **Single point of source change for Phase 2.** Only `wallet_import/electrum.rs` is touched in the behavior fold. Phase 3 adds tests; Phase 4 adds docs; Phase 5 bumps version + tag; Phase 6 closes FOLLOWUPS.
- **No `cargo fmt --all`** — the master HEAD has pre-existing fmt drift unrelated to Cycle 6 (Cycle 5 + 6a both encountered this). Restrict fmt scope to files touched by this plan.
- **install-pin-check CI gate on tag push.** Same Cycle 5 discipline.
- **Bisect-hygiene:** each phase commit is independently buildable + testable.

## Phase ordering rationale

- Task 1 (Phase 2 refusal-to-advisory) → Task 2 (Phase 3 tests): tests depend on the changed behavior.
- Task 2 → Task 3 (Phase 4 manual): manual prose cites the new advisory text + the new test fixtures.
- Task 3 → Task 4 (Phase 5 cycle close): release bundles all preceding work.
- Task 4 → Task 5 (Phase 6 FOLLOWUP closure): FOLLOWUP entries cite the v0.30.1 tag SHA.

## Risk register

- **Existing tests may assert refusal.** Phase 2 Step 5 catches this; any test asserting `Err(ImportWalletParse(_))` for an encrypted wallet needs updating to `Ok(_) + stderr advisory`.
- **Sniff regression.** The `use_encryption: true` wallets are JSON-parseable, so sniff continues to fire on the existing `seed_version` + `wallet_type` heuristics. Phase 3's `encrypted_wallet_sniff_still_positive` cell catches any sniff regression.
- **Manual lint may surface stderr-template substring drift.** Phase 4 Step 5 verifies byte-match by running the actual command against the fixture.
- **FOLLOWUP body line-number citations decay.** Phase 6 Step 1 uses grep to find the slug header at write-time (not the P0-cited line number which may have drifted since recon).

## Self-review (pre-R0 dispatch)

- ✓ All brainstorm v2 decisions covered by phases.
- ✓ No placeholders (`<...>` is the actual tag-SHA backfill at Phase 5 Step 5; CHANGELOG date `2026-MM-DD` is the ship-day backfill).
- ✓ File paths consistent throughout.
- ✓ No new `ToolkitError` variants.
- ✓ No new Cargo deps.
- ✓ SemVer consistent (PATCH v0.30.1).
- ✓ Cell count ≥ 9 (acceptable per brainstorm "12-15 target undershoot is acceptable" rationale).
