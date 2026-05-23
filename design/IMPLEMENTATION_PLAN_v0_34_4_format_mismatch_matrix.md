# v0.34.4 Format-Mismatch Matrix Completion — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development / executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Complete the 8×7 off-diagonal format-mismatch matrix in `import-wallet` so every `--format X` against a blob that sniffs as a different format Y refuses with `ImportWalletFormatMismatch` (exit 1).

**Architecture:** Purely additive `match sniff_outcome` arms in 4 incomplete dispatch blocks + 10 one-liner integration cells. No change to sniff logic, parsers, or `Ambiguous`/`NoMatch` tolerance. No new flag → no GUI/manual lockstep. TDD: each new cell is RED until its arm lands.

**Tech Stack:** Rust; `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`; `crates/mnemonic-toolkit/tests/cli_import_wallet_format_mismatch_matrix.rs`.

**Source SHA:** `f4d553e`. **SemVer:** PATCH (`v0.34.3 → v0.34.4`). Toolkit-only, no lockstep.

**Approved design:** presented + user-approved 2026-05-22 ("Approved"). Recon basis: `cycle-prep-recon-batch-4features.md` (matrix audit hand-verified — exactly 10 missing arms).

---

## The 10 missing off-diagonal pairs (hand-verified against `import_wallet.rs` dispatch)

| `--format` block | currently refuses | **add (canonical order)** |
|---|---|---|
| `coldcard` (`Some("coldcard")`, ~L587) | bsms, bitcoin-core, coldcard-multisig, sparrow, specter | **Electrum, Jade** |
| `electrum` (`Some("electrum")`, ~L693) | bsms, bitcoin-core, coldcard, coldcard-multisig, sparrow, specter | **Jade** |
| `sparrow` (`Some("sparrow")`, ~L802) | bsms, bitcoin-core, coldcard-multisig | **Coldcard, Electrum, Jade, Specter** |
| `specter` (`Some("specter")`, ~L837) | bsms, bitcoin-core, coldcard-multisig, sparrow | **Coldcard, Electrum, Jade** |

(Already complete 7/7: `bitcoin-core`, `bsms`, `coldcard-multisig`, `jade` — untouched.)

Canonical arm order within each block (matching the complete arms): `Bsms, BitcoinCore, Coldcard, ColdcardMultisig, Electrum, Jade, Sparrow, Specter` (self skipped).

Each new arm is exactly:
```rust
SniffOutcome::<Variant> => {
    return Err(ToolkitError::ImportWalletFormatMismatch {
        supplied: "<this-format>".to_string(),
        sniffed: "<sniffed-name>".to_string(),
    });
}
```
where `<sniffed-name>` is the kebab CLI name (`coldcard`, `coldcard-multisig`, `electrum`, `jade`, `sparrow`, `specter`).

---

## Task 1: Add the 10 refusal arms (TDD — tests first)

**Files:** Modify `tests/cli_import_wallet_format_mismatch_matrix.rs` (add 10 cells); modify `src/cmd/import_wallet.rs` (add 10 arms).

- [ ] **Step 1: Add the 10 failing test cells.** Append to `tests/cli_import_wallet_format_mismatch_matrix.rs` after the existing ColdcardMultisig section (after L60), mirroring the one-liner pattern. Fixtures are the same ones the file already documents (L31-37) and uses:

```rust

// ── v0.34.4: matrix completion — the 10 residual off-diagonal arms ─────────
// (FOLLOWUP `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.)

// Coldcard arm — 2 new refusals.
#[test] fn coldcard_refuses_electrum()  { assert_format_mismatch("coldcard", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn coldcard_refuses_jade()      { assert_format_mismatch("coldcard", "jade-multisig-2of3-p2wsh.json", "jade"); }

// Electrum arm — 1 new refusal.
#[test] fn electrum_refuses_jade()      { assert_format_mismatch("electrum", "jade-multisig-2of3-p2wsh.json", "jade"); }

// Sparrow arm — 4 new refusals.
#[test] fn sparrow_refuses_coldcard()   { assert_format_mismatch("sparrow", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn sparrow_refuses_electrum()   { assert_format_mismatch("sparrow", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn sparrow_refuses_jade()       { assert_format_mismatch("sparrow", "jade-multisig-2of3-p2wsh.json", "jade"); }
#[test] fn sparrow_refuses_specter()    { assert_format_mismatch("sparrow", "specter-singlesig-p2wpkh.json", "specter"); }

// Specter arm — 3 new refusals.
#[test] fn specter_refuses_coldcard()   { assert_format_mismatch("specter", "coldcard-singlesig-bip84-mainnet.json", "coldcard"); }
#[test] fn specter_refuses_electrum()   { assert_format_mismatch("specter", "electrum-standard-bip84-mainnet.json", "electrum"); }
#[test] fn specter_refuses_jade()       { assert_format_mismatch("specter", "jade-multisig-2of3-p2wsh.json", "jade"); }
```

- [ ] **Step 2: Run them — verify they FAIL** (the missing arms let the blob fall through to parse, which either succeeds-as-wrong-format or errors differently → not the mismatch exit).

Run: `cargo test -p mnemonic-toolkit --test cli_import_wallet_format_mismatch_matrix`
Expected: the 10 new cells FAIL (RED).

- [ ] **Step 3: Add the 10 dispatch arms in `src/cmd/import_wallet.rs`**, each in canonical position within its block (insert before the existing later arm / before `_ => {}`):

  - **`Some("coldcard")` block** — insert after the `SniffOutcome::ColdcardMultisig => {…}` arm, before `SniffOutcome::Sparrow`:
```rust
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
```
  - **`Some("electrum")` block** — insert after `SniffOutcome::ColdcardMultisig => {…}`, before `SniffOutcome::Sparrow`:
```rust
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "electrum".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
```
  - **`Some("sparrow")` block** — currently `Bsms, BitcoinCore, ColdcardMultisig`. Insert `Coldcard` after `BitcoinCore` (before `ColdcardMultisig`); insert `Electrum, Jade, Specter` after `ColdcardMultisig` (before `_ => {}`):
```rust
                // (after BitcoinCore, before ColdcardMultisig)
                SniffOutcome::Coldcard => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "coldcard".to_string(),
                    });
                }
                // (after ColdcardMultisig, before `_ => {}`)
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
                SniffOutcome::Specter => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "specter".to_string(),
                    });
                }
```
  - **`Some("specter")` block** — currently `Bsms, BitcoinCore, ColdcardMultisig, Sparrow`. Insert `Coldcard` after `BitcoinCore` (before `ColdcardMultisig`); insert `Electrum, Jade` after `ColdcardMultisig` (before `Sparrow`):
```rust
                // (after BitcoinCore, before ColdcardMultisig)
                SniffOutcome::Coldcard => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "coldcard".to_string(),
                    });
                }
                // (after ColdcardMultisig, before Sparrow)
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
```

  Also update each modified block's "intentionally narrow"/"matrix is now complete" header comment to note the matrix is now complete for that arm (v0.34.4).

- [ ] **Step 4: Run the tests — verify they PASS (GREEN).**

Run: `cargo test -p mnemonic-toolkit --test cli_import_wallet_format_mismatch_matrix`
Expected: all cells pass (the original + 10 new).

- [ ] **Step 5: Full regression + clippy.**

Run: `cargo test -p mnemonic-toolkit` → all green.
Run: `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

- [ ] **Step 6: Commit.**

```bash
git add crates/mnemonic-toolkit/src/cmd/import_wallet.rs crates/mnemonic-toolkit/tests/cli_import_wallet_format_mismatch_matrix.rs
git commit -m "feat(import-wallet): complete the 8x7 format-mismatch off-diagonal matrix (10 arms)"
```

---

## Task 2: Close the FOLLOWUP + release artifacts

**Files:** `design/FOLLOWUPS.md`, `crates/mnemonic-toolkit/Cargo.toml`, `Cargo.lock`, `scripts/install.sh`, `CHANGELOG.md`, move recon doc into `design/`.

- [ ] **Step 1: Close the slug.** In `design/FOLLOWUPS.md`, set `wallet-import-format-mismatch-matrix-completion-discovered-gaps` Status → resolved:

```
- **Status:** resolved — v0.34.4. All 10 residual off-diagonal arms added (coldcard→electrum,jade; electrum→jade; sparrow→coldcard,electrum,jade,specter; specter→coldcard,electrum,jade); the 8×7 = 56-cell off-diagonal matrix is now complete (bitcoin-core/bsms/coldcard-multisig/jade were already 7/7). 10 new cells in `tests/cli_import_wallet_format_mismatch_matrix.rs`. Closed via cycle-prep recon audit (SHA `f4d553e`).
```

- [ ] **Step 2: Version bump + lock regen** (the `cargo-lock-version-bump-lockstep` discipline). `Cargo.toml` `0.34.3` → `0.34.4`; `cargo build -p mnemonic-toolkit`; confirm `Cargo.lock` mnemonic-toolkit = `0.34.4`.

- [ ] **Step 3: install.sh self-pin** `mnemonic-toolkit-v0.34.3` → `-v0.34.4`.

- [ ] **Step 4: CHANGELOG** `[0.34.4]` entry above `[0.34.3]`:

```
## mnemonic-toolkit [0.34.4] — 2026-05-22

**SemVer-PATCH — `import-wallet` format-mismatch matrix completion.** Completes the 8×7 off-diagonal `--format X` vs sniff-as-Y refusal matrix: adds the 10 residual `ImportWalletFormatMismatch` arms (coldcard→electrum/jade; electrum→jade; sparrow→coldcard/electrum/jade/specter; specter→coldcard/electrum/jade) so an explicit `--format` against a blob of a different detected format always refuses (exit 1) symmetrically. `Ambiguous`/`NoMatch` sniff outcomes remain tolerated (explicit opt-in). 10 new cells in `tests/cli_import_wallet_format_mismatch_matrix.rs`. No CLI surface change → no GUI/manual lockstep. Closes `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.
```

- [ ] **Step 5: Move recon doc** `cycle-prep-recon-batch-4features.md` → `design/`.

- [ ] **Step 6: Manual lint** (no manual change; confirm no regression): `make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk` → 6/6 OK.

- [ ] **Step 7: Commit release artifacts.**

```bash
git add design/FOLLOWUPS.md crates/mnemonic-toolkit/Cargo.toml Cargo.lock scripts/install.sh CHANGELOG.md design/cycle-prep-recon-batch-4features.md design/IMPLEMENTATION_PLAN_v0_34_4_format_mismatch_matrix.md
git commit -m "release(toolkit): mnemonic-toolkit v0.34.4 — format-mismatch matrix completion"
```

- [ ] **Step 8: End-of-cycle opus review → GREEN (0C/0I)**; persist to `design/agent-reports/v0_34_4-end-of-cycle-review.md`.

- [ ] **Step 9: Ship (after user go-ahead)** — merge→master (ff), push, tag `mnemonic-toolkit-v0.34.4`, GH release. No GUI/manual lockstep.

---

## Self-review (writing-plans)

- **Spec coverage:** all 10 missing pairs (audit-verified) → 10 arms + 10 cells. ✓
- **No placeholders:** every arm + cell written verbatim; fixtures are the existing documented ones. ✓
- **Type consistency:** `ToolkitError::ImportWalletFormatMismatch { supplied, sniffed }` matches the existing arms' shape exactly; `SniffOutcome::{Coldcard,Electrum,Jade,Specter}` are live variants. ✓
- **SemVer/lockstep:** PATCH; no flag change → no lockstep. ✓
- **Risk:** near-zero — additive exclusive match arms mirroring proven complete arms; TDD RED→GREEN per cell.
