# mnemonic-toolkit-v0.31.2 Implementation Plan (Cycle 9 — sparrow-taproot-singlesig template-mode import)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Ship `mnemonic-toolkit-v0.31.2` (SemVer-PATCH; behavior-expansion). Convert the v0.31.1 narrow refusal for taproot SINGLESIG template-mode (`tr(@N/**)` shape per Sparrow's `CliTemplate::Bip86` emit) into a happy path that substitutes `@N/**` placeholders with concrete keys + feeds through the existing `concrete_keys_to_placeholders` → `parse_descriptor` pipeline. Closes Cycle 9 (`sparrow-taproot-singlesig-template-mode-import` FOLLOWUP).

**Architecture:** Single-edit refactor. Cycle 8's path-split at `wallet_import/sparrow.rs::parse` Step 6 introduced THREE branches:
1. `has_tr && !has_at_placeholder` → descriptor-passthrough (Cycle 8 happy path).
2. `has_tr && has_at_placeholder` → narrow refusal for taproot singlesig template-mode (Cycle 8 explicit refusal).
3. `!has_tr` → existing template-mode substitution (pre-v0.31.1 path).

Cycle 9 collapses branches (2) and (3) into a single substitution branch. The `is_descriptor_passthrough` flag stays as the path-split discriminator; taproot singlesig falls naturally into the substitution branch alongside non-taproot template-mode.

**Tech Stack:** Rust; zero net-new deps; zero `ToolkitError` variants; zero `lib.rs` changes; zero CLI surface changes; zero GUI lockstep.

**P0 recon (empirically verified at HEAD `7fa721d`):**
- The descriptor-passthrough path was used to smoke-test a synthetic singlesig blob with `tr([5436d724/86'/0'/0']xpub.../<0;1>/*)` (concrete-keys form). Result: clean import, descriptor preserved verbatim in bundle envelope, ms/mk/md cards generated. Confirms `concrete_keys_to_placeholders` + `parse_descriptor` accept taproot singlesig descriptors with origin brackets.
- The Cycle 8 path-split already has the substitution loop for template-mode in the `else` branch. Cycle 9 removes the narrow refusal so taproot-singlesig falls through to the existing substitution.

**P0 STRICT-GATE locks:**
- **Edit site:** `wallet_import/sparrow.rs` — remove the `if has_tr && has_at_placeholder { ... refusal ... }` block introduced at Cycle 8.
- **Refusal test conversion:** TWO existing tests (`sparrow.rs::tests::parse_p2tr_singlesig_refused` lib unit + `tests/cli_import_wallet_sparrow.rs::sparrow_taproot_singlesig_refused` integration) currently assert refusal. Both convert to happy-path assertions (success + descriptor-shape verification).
- **Cycle 8 boundary test conversion:** `tests/cli_import_wallet_sparrow_taproot.rs::taproot_singlesig_template_still_refused` currently asserts refusal — must convert OR delete + add happy-path counterpart.
- **SemVer:** PATCH `v0.31.1 → v0.31.2`. Behavior-expansion only.
- **No GUI lockstep** (no clap surface change).

## File structure

### Source files modified (toolkit)
- `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs`:
  - Remove the `if has_tr && has_at_placeholder { refuse }` block (Cycle 8 narrow refusal).
  - Update Step 6 comment to reflect v0.31.2 (taproot singlesig template-mode now accepted).
  - Convert `parse_p2tr_singlesig_refused` in-file test to happy-path (`parse_p2tr_singlesig_imports_via_substitution`).

### Test files modified (toolkit)
- `crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow.rs:305` (`sparrow_taproot_singlesig_refused`) → convert to happy-path (`sparrow_taproot_singlesig_imports_via_substitution`).
- `crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow_taproot.rs::taproot_singlesig_template_still_refused` → convert to happy-path (`taproot_singlesig_template_imports_via_substitution`) + retain a coverage assertion that the descriptor preserves the substituted xpub correctly.

### Documentation modified (toolkit)
- `docs/manual/src/45-foreign-formats.md` — §"Taproot import (shipped v0.31.1)" subsection: drop the "narrowing" paragraph about taproot singlesig template-mode; cite v0.31.2 closure.

### Release tooling
- `crates/mnemonic-toolkit/Cargo.toml:3` — `0.31.1` → `0.31.2`.
- `CHANGELOG.md` — new `## [0.31.2]` section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.31.1` → `mnemonic-toolkit-v0.31.2`.
- `design/FOLLOWUPS.md` — close `sparrow-taproot-singlesig-template-mode-import`.

## Tasks

### Task 1: Phase 2 — Remove the narrow refusal branch

**Files:** modify `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs`.

- [ ] **Step 1: Remove the `has_tr && has_at_placeholder` refusal block**

Locate the block introduced in Cycle 8 (currently in the file). Delete the entire `if has_tr && has_at_placeholder { return Err(...) }` block. The remaining flow:
- `let has_tr = script_template.contains("tr(");` — keep for descriptor-passthrough detection.
- `let has_at_placeholder = script_template.contains("@0/**");` — keep.
- `let is_descriptor_passthrough = has_tr && !has_at_placeholder;` — keep.
- Step 5 substitution loop fires for ALL template-mode shapes (including taproot singlesig).

Update the comment block to document the v0.31.2 simplification:

```rust
// Step 6 (v0.31.2 Cycle 9): taproot singlesig template-mode (Bip86:
// `tr(@0/**)`) joins the general template-mode substitution path. Per
// Cycle 9 P0 recon, the resulting `tr([fp/path]xpub/<0;1>/*)` descriptor
// is accepted cleanly by `concrete_keys_to_placeholders` +
// `parse_descriptor`. Closes `sparrow-taproot-singlesig-template-mode-import`.
//
// The path-split remaining: `has_tr && !has_at_placeholder` =
// descriptor-passthrough (Cycle 8 happy path; taproot multisig).
// Otherwise (any template-mode, including taproot singlesig) =
// substitute then parse.
```

- [ ] **Step 2: Convert in-file unit test `parse_p2tr_singlesig_refused` to happy-path**

Rename to `parse_p2tr_singlesig_imports_via_substitution`. Replace the `unwrap_err()` + message-substring check with `unwrap()` + assertions on the parsed descriptor (cosigners count, xpub preserved, etc.).

- [ ] **Step 3: Build + run lib tests**

```bash
cargo build --package mnemonic-toolkit 2>&1 | tail -3
cargo test --package mnemonic-toolkit --bin mnemonic wallet_import::sparrow 2>&1 | tail -10
```

- [ ] **Step 4: Commit Phase 2**

```bash
git add crates/mnemonic-toolkit/src/wallet_import/sparrow.rs
git commit -m "feat(sparrow): v0.31.2 Phase 2 — taproot singlesig template-mode import"
```

### Task 2: Phase 3 — Integration test conversion + new happy-path cells

**Files:** modify both sparrow integration test files.

- [ ] **Step 1: Convert `cli_import_wallet_sparrow.rs::sparrow_taproot_singlesig_refused`**

Rename + flip from refusal to success. Assert the bundle envelope carries the substituted descriptor `tr([5436d724/86'/0'/0']xpub6CAYwo2AfKJy1cdFGBAgLvCrZULhEkZ9C9s4GGXwXzHvNPguMWBcVrGEDjP2ZJdX92gVWLeLrNVVmipTrKqrwMy2eT282xKEyHMbPDrcD9e/<0;1>/*)`.

- [ ] **Step 2: Convert `cli_import_wallet_sparrow_taproot.rs::taproot_singlesig_template_still_refused`**

Rename to `taproot_singlesig_template_imports_via_substitution` + flip from refusal to success. Same assertion shape as Step 1.

- [ ] **Step 3: Add a 2nd taproot singlesig cell**

Add a cell that exercises the BIP-341 NUMS+singlesig-with-tweak shape if Sparrow ever emits one (skip for now; just one cell suffices).

Optional: add a round-trip cell — import the Bip86 fixture, then `--from-import-json` re-emit and assert byte-equal output.

- [ ] **Step 4: Run integration tests**

```bash
cargo test --package mnemonic-toolkit --test cli_import_wallet_sparrow 2>&1 | tail -5
cargo test --package mnemonic-toolkit --test cli_import_wallet_sparrow_taproot 2>&1 | tail -5
```

- [ ] **Step 5: Commit Phase 3**

### Task 3: Phase 4 — Manual chapter update

**Files:** modify `docs/manual/src/45-foreign-formats.md`.

- [ ] **Step 1: Drop the "narrowing" paragraph + cite v0.31.2**

The current §"Taproot import (shipped v0.31.1)" section has a "Narrowing" paragraph saying taproot singlesig template-mode is not yet shipped. Cycle 9 ships it — drop that paragraph + update the section title to "Taproot import (shipped v0.31.1 + v0.31.2)".

- [ ] **Step 2: Run manual lint**

- [ ] **Step 3: Commit Phase 4**

### Task 4: Phase 5 — Cycle close

- [ ] **Step 1: Bump version + install.sh self-pin + CHANGELOG entry**

- [ ] **Step 2: Full pre-tag audit (cargo test --workspace + clippy + manual lint)**

- [ ] **Step 3: Commit + tag mnemonic-toolkit-v0.31.2 + push + GH Release**

- [ ] **Step 4: Wait for install-pin-check CI green**

### Task 5: Phase 6 — FOLLOWUP closure

- [ ] **Step 1: Update `sparrow-taproot-singlesig-template-mode-import` body**

Status → `resolved (Cycle 9 / v0.31.2)`. Add `Resolved by:` line.

- [ ] **Step 2: Commit + push FOLLOWUP closure**

- [ ] **Step 3: Update memory**

Add `project_v0_31_2_cycle_9_shipped.md` entry summarizing outcome.

## Cross-phase invariants

- Opus R0 review on plan-doc BEFORE Phase 2 dispatch.
- No `cargo fmt --all`.
- No GUI lockstep.
- install-pin-check CI gate.

## Risk register

- **`is_descriptor_passthrough` flag still useful?** Yes — descriptor-passthrough (taproot multisig) still bypasses Step 5 substitution. The discriminator stays load-bearing.
- **Taproot-multi-template-mode** (hypothetical `tr(multi_a(@0/**,@1/**,...))` shape) — not currently emitted by Sparrow, but Cycle 9's change would handle it correctly via substitution. No regression.
- **Existing v0.31.1 multisig descriptor-passthrough** — Cycle 9 doesn't touch the `is_descriptor_passthrough` branch; that path is unchanged.

## Self-review (pre-R0 dispatch)

- ✓ P0 recon empirically confirms pipeline accepts taproot singlesig.
- ✓ Edit site exactly identified (remove the narrow refusal block introduced at Cycle 8).
- ✓ All 3 refusal-asserting tests enumerated (1 lib + 2 integration).
- ✓ SemVer PATCH (behavior-expansion only; no clap surface change).
- ✓ No new deps / variants / lib.rs / CLI surface.
- ✓ Manual mirror invariant (no clap change).
