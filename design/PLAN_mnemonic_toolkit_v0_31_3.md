# mnemonic-toolkit-v0.31.3 Implementation Plan (Cycle 10 — seedqr-bundle-slot-integration)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Ship `mnemonic-toolkit-v0.31.3` (SemVer-PATCH; additive flag-value-enumeration extension). Closes `seedqr-bundle-slot-integration` FOLLOWUP. Adds `--slot @N.seedqr=<digit-string>` source on `bundle` / `verify-bundle` / `export-wallet` (all three full-grammar consumers); decodes via `seedqr::decode` at slot-emit time; legal subkey sets `[Seedqr]`, `[Seedqr, Path]`, `[Seedqr, Fingerprint, Path]` mirror v0.19.0 SPEC §6.6.b exception for Phrase.

**Architecture:** New `SlotSubkey::Seedqr` variant (secret-bearing; decodes to BIP-39 phrase at slot-emit). Identical post-decode dispatch as the existing `Phrase` branch in each of the three consumer paths. Convention-matching value semantics: `<digit-string>` literal (48 or 96 ASCII digits with optional whitespace); user pipes file content via shell expansion or `@N.seedqr=-` stdin. `seedqr::decode` is reused verbatim — no library changes.

**Tech Stack:** Rust; zero net-new deps (reuses `seedqr` library shipped v0.30.0); 1 net-new `SlotSubkey` variant; 0 `ToolkitError` variants (decode errors map through the existing `map_seedqr_error`, promoted to `pub(crate)`); 0 lib.rs changes.

**P0 STRICT-GATE locks (user-confirmed 2026-05-21):**
- **Q1 → Option A**: new `SlotSubkey::Seedqr` variant; decode at slot-emit time (not parse-stage).
- **Q2 → Digit-string literal**: `<value>` is the 48/96-digit SeedQR string; convention-matching.
- **Q3 → Parity scope**: legal subkey sets are `[Seedqr]`, `[Seedqr, Path]`, `[Seedqr, Fingerprint, Path]` mirroring v0.19.0 §6.6.b exception for Phrase.
- **Q4 → All three consumers**: `bundle` + `verify-bundle` + `export-wallet`.
- **SemVer → PATCH v0.31.2 → v0.31.3** (R0 I1 confirmed: schema_mirror gate compares clap flag-NAME parity NOT value-content; matches v0.28.x additive-flag PATCH precedent). GUI lockstep NOT triggered; optional GUI pin bump filed as a follow-on FOLLOWUP if desirable.

**R0 review status:** GREEN after fold. Original R0 was RED (3C/2I/2M); all folded inline pre-Phase-2. R0 report at `design/agent-reports/v0_32_0-plan-doc-r0-review.md`. Plan-doc filename retains `v0_32_0` for searchability; release version is v0.31.3.

## P0 STRICT-GATE recon (verified at master HEAD `7e50902`)

- `SlotSubkey` declaration order at `slot_input.rs:17-32`: `Phrase < Entropy < Xpub < MasterXpub < Fingerprint < Path < Wif < Xprv` (custom semantic order; derives `Ord`).
- `validate_slot_set` sorts subkeys via the derived `Ord` (L250); `is_legal_set` (L313-330) matches sorted slices against patterns IN ascending-sort order.
- **Therefore**: to get legal-set patterns `[Seedqr, Path]` (NOT `[Path, Seedqr]`), `Seedqr` MUST be declared AFTER `Phrase` and BEFORE `Entropy` in the enum (i.e., position index 1). The R0 C1 finding that "place Seedqr at the END" would have produced `[Path, Seedqr]` ascending-sorted — which would be a wrong pattern.
- `is_secret_bearing` (L60-62): secret variants are Phrase + Entropy + Xprv + Wif. Adding Seedqr → secret-bearing.
- `apply_slot_stdin` (L178-208): single-stdin-per-invocation invariant enforced by counting `is_stdin_sentinel` slots. Adding Seedqr to `is_secret_bearing` makes `@N.seedqr=-` participate naturally; refusal-matrix cell required.
- `cmd/seedqr.rs::map_seedqr_error` is PRIVATE. C3 fold: promote to `pub(crate)` so bundle/verify-bundle/export-wallet consumers can reuse the canonical mapping.
- `cmd/bundle.rs:91-110` help-text block omits `master_xpub` (pre-existing v0.x drift — file as Phase 6 FOLLOWUP per M2 fold).
- `cmd/bundle.rs:433-614` slot consumer is if/else-if chain on `subkeys.contains(&SlotSubkey::X)`. Branch insertion site: AFTER `Phrase` block (~L433-464), BEFORE `Xpub` block (~L465) — keeps secret-bearing branches contiguous.

## File structure

### Source files modified (toolkit library)
- `crates/mnemonic-toolkit/src/slot_input.rs`:
  - Add `Seedqr` variant to `SlotSubkey` enum AT POSITION 1 (between `Phrase` and `Entropy`; correctness-critical per R0 C1).
  - Add `"seedqr"` token mapping in `from_token` (L34-47) + `as_str` (L48-58).
  - Add `Seedqr` to `is_secret_bearing` (L60-62).
  - Add new entries to `is_legal_set` (L313-330): `[Seedqr]`, `[Seedqr, Path]`, `[Seedqr, Fingerprint, Path]` (ascending-sorted because Seedqr is at position 1, BEFORE Path's position 5).
  - Extend the v0.19.0 `exempted_v0_19_0` matcher (L274-278) to also exempt `[Seedqr, Path]` + `[Seedqr, Fingerprint, Path]`.
  - Update the `from_token` error message at L145-149 to include `seedqr` in the expected-subkeys list.
  - Update the `validate_slot_set` rustdoc (L210-224) to mention the seedqr subkey + its semantic identity with phrase post-decode.

### Source files modified (toolkit consumers)
- `crates/mnemonic-toolkit/src/cmd/seedqr.rs`:
  - Promote `map_seedqr_error` from private fn to `pub(crate)` (R0 C3 fold). Confirm callsite signatures don't need adjustment.
- `crates/mnemonic-toolkit/src/cmd/bundle.rs`:
  - Update `--slot` clap-help block at L91-110: add `seedqr  48 or 96 ASCII digits encoding a BIP-39 phrase (secret; decoded inline)` AFTER the `phrase` line.
  - Slot-consumer branch: NEW branch INSERTED AFTER the `Phrase` block, BEFORE the `Xpub` block. Decode via `seedqr::decode(value)`; on error call `crate::cmd::seedqr::map_seedqr_error(e, "slot decode")` returning `ToolkitError::BadInput` with prefix `"slot @{idx}.seedqr: {action}: {err}"`; on success substitute the decoded phrase into a synthetic Phrase-shape and fall through to the SAME phrase-bind logic via shared helper extraction (factor the Phrase body into a closure or local fn).
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`:
  - Mirror help text update.
  - Mirror slot-consumer branch.
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`:
  - Mirror help text update.
  - Mirror slot-consumer branch.

### Test files modified (toolkit)
- `crates/mnemonic-toolkit/src/slot_input.rs` (in-file `tests` mod):
  - Add `parse_seedqr_singleton` happy-path cell (mirrors L431-445 phrase test pattern).
  - Add `parse_seedqr_stdin_sentinel` cell (asserts `@0.seedqr=-` parses + `is_stdin_sentinel() == true`).
  - Add `validate_set_seedqr_alone_legal` cell.
  - Add `validate_set_seedqr_path_legal` cell.
  - Add `validate_set_seedqr_fingerprint_path_legal` cell.
  - Add `validate_set_seedqr_with_xpub_conflicts` cell (refusal: secret+watch-only).
- `crates/mnemonic-toolkit/tests/cli_bundle_seedqr_slot.rs` (NEW):
  - Cell `bundle_seedqr_slot_happy_path_24word` — 96-digit SeedQR → bundle envelope **byte-equal** to the equivalent `--slot @N.phrase=` invocation.
  - Cell `bundle_seedqr_slot_happy_path_12word` — 48-digit SeedQR → bundle envelope **byte-equal** to phrase-direct (M1 fold: byte-equal asserted on BOTH happy-path cells).
  - Cell `bundle_seedqr_slot_invalid_digit_count_refused` — non-48/96 digit string refused with exit 1 + stderr citing `seedqr: slot decode`.
  - Cell `bundle_seedqr_slot_checksum_failure_refused` — valid digit count but invalid BIP-39 checksum refused.
  - Cell `bundle_seedqr_slot_stdin_sentinel_happy_path` — `@0.seedqr=-` consumes digit-string from stdin.
  - Cell `bundle_seedqr_slot_double_stdin_refused` — `@0.seedqr=- @1.phrase=-` refused per existing single-stdin-per-invocation invariant (I2 fold).
  - Cell `bundle_seedqr_slot_with_path_non_canonical_mode` — `[Seedqr, Path]` legal in non-canonical descriptor mode (mirrors `[Phrase, Path]` v0.19.0 pattern).
- `crates/mnemonic-toolkit/tests/cli_export_wallet_seedqr_slot.rs` (NEW): 2 happy-path cells.
- `crates/mnemonic-toolkit/tests/cli_verify_bundle_seedqr_slot.rs` (NEW): 2 happy-path cells.

### Documentation modified (toolkit)
- `docs/manual/src/40-cli-reference/41-mnemonic.md`:
  - `mnemonic bundle` section: update `--slot` subkey enumeration to include `seedqr`.
  - `mnemonic verify-bundle` section: same.
  - `mnemonic export-wallet` section: same.

### Release tooling
- `crates/mnemonic-toolkit/Cargo.toml:3` — `0.31.2` → `0.31.3`.
- `CHANGELOG.md` — new `## [0.31.3]` section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.31.2` → `mnemonic-toolkit-v0.31.3`.
- `design/FOLLOWUPS.md` — close `seedqr-bundle-slot-integration`. File NEW FOLLOWUPs: (a) `bundle-slot-help-text-master-xpub-drift` (M2 fold — pre-existing v0.x drift); (b) `gui-seedqr-slot-subkey-help-mirror` (optional GUI pin bump for help-text + dropdown surface; lockstep gate doesn't auto-fire).

## Tasks

### Task 1: Phase 2 — `slot_input.rs` library extension

**Files:** modify `crates/mnemonic-toolkit/src/slot_input.rs`.

- [ ] **Step 1: Add `Seedqr` variant to `SlotSubkey` AT POSITION 1**

```rust
pub enum SlotSubkey {
    Phrase,
    Seedqr,    // v0.31.3 — secret-bearing; decodes to BIP-39 phrase at slot-emit
    Entropy,
    Xpub,
    MasterXpub,
    Fingerprint,
    Path,
    Wif,
    Xprv,
}
```

Update `from_token`:
```rust
"seedqr" => Self::Seedqr,
```

Update `as_str`:
```rust
Self::Seedqr => "seedqr",
```

Update `is_secret_bearing`:
```rust
pub fn is_secret_bearing(self) -> bool {
    matches!(self, Self::Phrase | Self::Seedqr | Self::Entropy | Self::Xprv | Self::Wif)
}
```

Update `from_token` error message at L145-149:
```rust
"unknown slot subkey {:?}; expected one of: phrase, seedqr, entropy, xpub, master_xpub, fingerprint, path, wif, xprv"
```

- [ ] **Step 2: Update `is_legal_set` and `exempted_v0_19_0`**

In `is_legal_set` (L313-330), add to the match arms:
```rust
| [Seedqr]
| [Seedqr, Path]
| [Seedqr, Fingerprint, Path]
```

In `validate_slot_set::exempted_v0_19_0` (L274-278), extend:
```rust
let exempted_v0_19_0 = matches!(
    subkeys.as_slice(),
    [SlotSubkey::Phrase, SlotSubkey::Path]
        | [SlotSubkey::Phrase, SlotSubkey::Fingerprint, SlotSubkey::Path]
        | [SlotSubkey::Seedqr, SlotSubkey::Path]
        | [SlotSubkey::Seedqr, SlotSubkey::Fingerprint, SlotSubkey::Path]
);
```

- [ ] **Step 3: Add 6 in-file unit tests**

- [ ] **Step 4: Build + run lib tests**

```bash
cargo test --package mnemonic-toolkit --bin mnemonic slot_input 2>&1 | tail -15
```

- [ ] **Step 5: Commit Phase 2**

```bash
git add crates/mnemonic-toolkit/src/slot_input.rs
git commit -m "feat(slot_input): v0.31.3 Phase 2 — SlotSubkey::Seedqr variant + legal-set extension"
```

### Task 2: Phase 3a — `cmd/seedqr.rs` promotion + `bundle.rs` consumer branch

**Files:** modify `crates/mnemonic-toolkit/src/cmd/seedqr.rs` + `crates/mnemonic-toolkit/src/cmd/bundle.rs`.

- [ ] **Step 1: Promote `map_seedqr_error` to `pub(crate)` (R0 C3 fold)**

In `cmd/seedqr.rs:58`, change `fn map_seedqr_error` → `pub(crate) fn map_seedqr_error`. Verify all existing callsites still compile.

- [ ] **Step 2: Update `--slot` clap help text**

Insert `seedqr  48 or 96 ASCII digits encoding a BIP-39 phrase (secret; decoded inline via seedqr::decode)` AFTER the `phrase` line in the verbatim_doc_comment block.

- [ ] **Step 3: Add slot-consumer branch AFTER Phrase, BEFORE Xpub**

Insert the new branch between the Phrase block (~L433-464) and the Xpub block (~L465+). The branch decodes via `seedqr::decode(value)`, maps error via `crate::cmd::seedqr::map_seedqr_error(e, "slot decode")`, then materializes the resulting phrase into the same downstream phrase-bind logic. Factor shared phrase-bind logic into a local helper closure to avoid duplication.

- [ ] **Step 4: Add integration tests `tests/cli_bundle_seedqr_slot.rs`**

7 cells per the test files plan above.

- [ ] **Step 5: Build + run integration tests**

```bash
cargo test --package mnemonic-toolkit --test cli_bundle_seedqr_slot 2>&1 | tail -10
```

- [ ] **Step 6: Commit Phase 3a**

### Task 3: Phase 3b — consumer branches in `verify_bundle.rs` + `export_wallet.rs`

- [ ] **Step 1: Mirror bundle.rs help + branch in verify_bundle.rs**
- [ ] **Step 2: Mirror in export_wallet.rs**
- [ ] **Step 3: Add `tests/cli_verify_bundle_seedqr_slot.rs` + `tests/cli_export_wallet_seedqr_slot.rs`**
- [ ] **Step 4: Build + run all integration tests**
- [ ] **Step 5: Commit Phase 3b**

### Task 4: Phase 4 — Manual chapter mirror

**Files:** modify `docs/manual/src/40-cli-reference/41-mnemonic.md`.

- [ ] **Step 1: Update `--slot` enumeration in all 3 consumer sections**
- [ ] **Step 2: Run manual lint**

```bash
make -C docs/manual lint MNEMONIC_BIN="$(pwd)/target/debug/mnemonic"
```

- [ ] **Step 3: Commit Phase 4**

### Task 5: Phase 5 — Toolkit cycle close

- [ ] **Step 1: Bump version + install.sh self-pin + CHANGELOG entry**
- [ ] **Step 2: Full pre-tag audit (cargo test --workspace + clippy + manual lint)**
- [ ] **Step 3: Opus end-of-cycle review BEFORE tag**
- [ ] **Step 4: Commit + tag mnemonic-toolkit-v0.31.3 + push + GH Release**
- [ ] **Step 5: Wait for install-pin-check CI green**

### Task 6: Phase 6 — FOLLOWUP closure + new filings

- [ ] **Step 1: Close `seedqr-bundle-slot-integration` (resolved by v0.31.3 release SHA)**
- [ ] **Step 2: File `bundle-slot-help-text-master-xpub-drift` (pre-existing drift; M2 fold)**
- [ ] **Step 3: File `gui-seedqr-slot-subkey-help-mirror` (optional GUI pin bump)**
- [ ] **Step 4: Commit + push FOLLOWUP closure**
- [ ] **Step 5: Update memory**

## Cross-phase invariants

- Opus R0 review on plan-doc BEFORE Phase 2 dispatch (DONE — R0 RED 3C/2I/2M folded → re-dispatch R1).
- Opus end-of-cycle review BEFORE tagging.
- No `cargo fmt --all`; restrict scope to cycle files.
- No GUI lockstep gate forces v0.31.3 → GUI pin bump (per R0 I1); GUI bump filed as optional FOLLOWUP.
- install-pin-check CI gate.

## Risk register

- **`SlotSubkey` enum-order correctness (R0 C1)** — `Seedqr` MUST be declared at position 1 (after Phrase, before Entropy) for `[Seedqr, Path]` / `[Seedqr, Fingerprint, Path]` to be ascending-sorted (per `validate_slot_set::sort` + `is_legal_set::matches`). Placing it at the end would produce `[Path, Seedqr]` ascending-sorted and the legal-set patterns would fail to match.
- **Branch placement (R0 C2)** — bundle.rs / verify_bundle.rs / export_wallet.rs consumer branches go AFTER Phrase, BEFORE Xpub, to keep secret-bearing branches contiguous.
- **`map_seedqr_error` privacy (R0 C3)** — promote to `pub(crate)` to avoid error-text drift across 3 consumer sites.
- **Decode-error UX** — error format `slot @{idx}.seedqr: {action}: {err}` cited consistently across all 3 consumers (verified by `bundle_seedqr_slot_invalid_digit_count_refused` cell).
- **Stdin-sentinel + double-stdin (R0 I2)** — refusal-matrix cell `bundle_seedqr_slot_double_stdin_refused` asserts existing invariant fires correctly.
- **Master-xpub help-text drift (R0 M2)** — pre-existing v0.x drift; file FOLLOWUP at Phase 6.
- **SemVer choice (R0 I1)** — PATCH v0.31.3 (user-confirmed); no GUI lockstep gate.

## Self-review (pre-R1 dispatch)

- ✓ All 3 R0 Criticals folded inline.
- ✓ Both R0 Importants folded.
- ✓ Both R0 Minors folded (byte-equal on both happy-paths + master_xpub FOLLOWUP filed at Phase 6).
- ✓ SemVer changed to PATCH v0.31.3 per user lock.
- ✓ Risk register revised to surface the load-bearing enum-position invariant.
