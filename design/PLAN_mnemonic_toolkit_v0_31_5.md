# mnemonic-toolkit-v0.31.5 Implementation Plan (Cycle 12 — seedqr-15-18-21-word-counts)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or implement directly given trivial scope.

**Goal:** Ship `mnemonic-toolkit-v0.31.5` (SemVer-PATCH; behavior-expansion). Closes `seedqr-15-18-21-word-counts` FOLLOWUP. Widen `mnemonic seedqr {encode, decode}` word-count support from `{12, 24}` to `{12, 15, 18, 21, 24}` (the complete BIP-39 word-count set).

**Architecture:** Two validation gates need extending — `seedqr::decode`'s digit-length gate (48 or 96 → 48 | 60 | 72 | 84 | 96) and `seedqr::encode`'s word-count gate (12 | 24 → 12 | 15 | 18 | 21 | 24). Error texts on both invalid-shape errors need updating. The downstream `bip39::Mnemonic::parse_in` already accepts all 5 word counts (BIP-39 itself standardizes all 5).

**Tech Stack:** Rust; zero new deps; zero `ToolkitError` variants; zero lib.rs / CLI surface changes; zero GUI lockstep.

**SeedSigner spec rationale:** SeedSigner's published SeedQR spec [^1] documents 12 and 24 words explicitly; 15/18/21 are BIP-39-standard but not in the original SeedQR spec body. The format itself trivially extends — SeedQR's encoding is BIP-39-index × 4-decimal-digits per word, agnostic to word count. SeedSigner's reference impl `src/seedsigner/models/encode_qr.py` accepts arbitrary phrase length (no hard-coded 12/24 gate in `StandardSeedQrEncoder`). Cross-impl validation: re-encoding any of the canonical Trezor 15/18/21-word vectors and round-tripping byte-for-byte. v0.31.5 is the toolkit-side embrace of the BIP-39-complete set.

[^1]: https://seedsigner.com/seedqr-instructions/

**P0 STRICT-GATE recon (verified at master HEAD `92cbdc0`):**
- `seedqr.rs:62` — `if len != 48 && len != 96 { ... }` digit-length gate.
- `seedqr.rs:107` — `if words.len() != 12 && words.len() != 24 { ... }` word-count gate.
- `seedqr.rs:40` + L43 — error message strings to update.
- `tests/cli_seedqr.rs:292` (encode_rejects_15) + L303 (encode_rejects_18) + L314 (encode_rejects_21) — currently refusal cells, must flip to happy-path.
- `tests/cli_seedqr.rs:278` (encode_rejects_13) + L325 (encode_rejects_25) — KEEP as refusal cells (13 and 25 are not BIP-39-valid).

**SemVer rationale (v0.31.4 → v0.31.5 PATCH):** pure behavior-expansion (previously-refused inputs now succeed). No CLI surface change. No new flags / variants / lib re-exports.

**No GUI lockstep:** GUI's `mnemonic seedqr {encode,decode}` invocations transparently broaden — no GUI-side enumeration of word counts.

## File structure

### Source files modified (toolkit library)
- `crates/mnemonic-toolkit/src/seedqr.rs`:
  - L40: `"invalid digit count (expected 48 or 96; got {got})"` → `"invalid digit count (expected 48, 60, 72, 84, or 96; got {got})"`.
  - L43: `"invalid word count: {got} (only 12 or 24 supported)"` → `"invalid word count: {got} (only 12, 15, 18, 21, or 24 supported)"`.
  - L62: `if len != 48 && len != 96 {` → `if !matches!(len, 48 | 60 | 72 | 84 | 96) {`.
  - L107: `if words.len() != 12 && words.len() != 24 {` → `if !matches!(words.len(), 12 | 15 | 18 | 21 | 24) {`.

### Test files modified (toolkit)
- `crates/mnemonic-toolkit/src/seedqr.rs` (in-file `tests` mod):
  - Add canonical 15/18/21-word BIP-39-valid test vectors (entropy-all-zeros + checksum-determined last word).
  - Add happy-path encode + decode + round-trip cells for each new word count.
- `crates/mnemonic-toolkit/tests/cli_seedqr.rs`:
  - `encode_rejects_15_word_count` → rename to `encode_accepts_15_word_count` + flip to happy-path (assert exit 0 + digit-string stdout).
  - `encode_rejects_18_word_count` → rename to `encode_accepts_18_word_count` + flip.
  - `encode_rejects_21_word_count` → rename to `encode_accepts_21_word_count` + flip.
  - Keep `encode_rejects_13_word_count` + `encode_rejects_25_word_count` verbatim. Existing `predicates::str::contains("seedqr: encode: invalid word count")` is a prefix substring that survives the new parenthetical — no assertion update needed (R0 I1 fold).
  - **Existing decode boundary cells suffice** (R0 I3a fold): `seedqr.rs:186-219` (lib) + `cli_seedqr.rs:110-154` (CLI) already cover 47/49/95/97. NO new boundary cells.
  - **Add `encode_json_mode_15_word`** (R0 I3b fold): JSON-envelope happy-path cell asserting `word_count == 15` field is emitted correctly. One cell covers the new-word-count JSON surface.

### Documentation modified (toolkit)
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — `mnemonic seedqr` section: update the "Scope" / "Word counts" subsection to reflect the widening.

### Release tooling
- `crates/mnemonic-toolkit/Cargo.toml:3` — `0.31.4` → `0.31.5`.
- `CHANGELOG.md` — new `## [0.31.5]` section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.31.4` → `mnemonic-toolkit-v0.31.5`.
- `design/FOLLOWUPS.md` — close `seedqr-15-18-21-word-counts`.

## Tasks

### Task 1: Phase 2 — Widen seedqr.rs validation gates

- [ ] **Step 1: Update the 4 lines (errors + gates).**
- [ ] **Step 2: Add in-file unit cells for 15/18/21 round-trips (use canonical Trezor entropy-zero vectors).**
- [ ] **Step 3: Build + run lib tests.**
- [ ] **Step 4: Commit Phase 2.**

### Task 2: Phase 3 — Flip 3 integration refusal cells + add 3 new boundary cells

- [ ] **Step 1: Rename + flip encode_rejects_{15,18,21}_word_count to encode_accepts_*.**
- [ ] **Step 2: Add decode boundary cells (47, 49, 97).**
- [ ] **Step 3: Build + run integration tests.**
- [ ] **Step 4: Commit Phase 3.**

### Task 3: Phase 4 — Manual chapter mirror

- [ ] **Step 1: Update the seedqr "Scope" / "Word counts" subsection.**
- [ ] **Step 2: Run manual lint.**
- [ ] **Step 3: Commit Phase 4.**

### Task 4: Phase 5 — Cycle close

- [ ] **Step 1: Version bump + install.sh self-pin + CHANGELOG entry.**
- [ ] **Step 2: Full pre-tag audit (cargo test --workspace + clippy + manual lint).**
- [ ] **Step 3: Opus end-of-cycle review BEFORE tag.**
- [ ] **Step 4: Commit + tag mnemonic-toolkit-v0.31.5 + push + GH Release.**
- [ ] **Step 5: Wait for install-pin-check CI green.**
- [ ] **Step 6: Close FOLLOWUP + update memory.**

## Cross-phase invariants

- Opus R0 review on plan-doc BEFORE Phase 2 dispatch.
- Opus end-of-cycle review BEFORE tagging.
- No `cargo fmt --all`.
- No GUI lockstep.
- install-pin-check CI gate.

## Risk register

- **Canonical test vectors** — Trezor entropy-all-zeros at 20/24/28 bytes produces 15/18/21-word phrases ending in `agent` / `agree` / `ahead` respectively. Verify via the encode round-trip: phrase → digits → decode → phrase byte-equal.
- **SemVer correctness** — pure behavior-expansion (previously-rejected inputs now succeed); no CLI surface change → PATCH.
- ~~**Error-text regression**~~ — R0 I1 fold: existing `encode_rejects_13_word_count` + `encode_rejects_25_word_count` assertions use `predicates::str::contains("seedqr: encode: invalid word count")` which is a prefix substring that survives the new text. NO assertion update needed.

## Self-review (pre-R0 dispatch)

- ✓ P0 recon confirmed 4 source line numbers at HEAD.
- ✓ Test surface enumerated (3 flips + 3 boundary refusals + 6 round-trip cells).
- ✓ SemVer PATCH justified.
- ✓ No GUI / wire-shape impact.
- ✓ Canonical Trezor vectors identified for 15/18/21.
