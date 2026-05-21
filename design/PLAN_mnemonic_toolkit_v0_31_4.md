# mnemonic-toolkit-v0.31.4 Implementation Plan (Cycle 11 — sparrow-import-detection-regex-defensive-widening)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or implement directly given the trivial scope.

**Goal:** Ship `mnemonic-toolkit-v0.31.4` (SemVer-PATCH; defensive hardening). Closes `sparrow-import-detection-regex-defensive-widening` FOLLOWUP filed at Cycle 9 close (end-of-cycle opus M1 finding).

**Architecture:** Widen `wallet_import/sparrow.rs::parse` Step 6 path-split discriminator from literal substring `script_template.contains("@0/**")` to regex `Regex::new(r"@\d+/\*\*").is_match(script_template)`. The discriminator's purpose is to distinguish DESCRIPTOR-PASSTHROUGH (taproot multisig; concrete `[fp/path]xpub` keys embedded) from TEMPLATE-MODE (any `@N/**` placeholder for substitution). Sparrow's current emit-side at `wallet_export/sparrow.rs` always builds placeholders from `(0..n)` so `@0/**` is always present in template-mode blobs, but a hypothetical future emit-side change (e.g., 2-of-2 with cosigner indexing starting at 1) would silently break the substring discriminator. Defensive only; no current behavior change.

**Tech Stack:** Rust; reuses existing `regex = "1"` dep (already used by `wallet_import/{pipeline,bsms,bitcoin_core}.rs`); zero new deps, variants, lib.rs changes, or CLI surface changes.

**P0 STRICT-GATE recon (verified at master HEAD `bee253f`):**
- `sparrow.rs:338` — literal substring check: `let has_at_placeholder = script_template.contains("@0/**");`.
- `Cargo.toml:32` — `regex = "1"` already a dep.
- `wallet_import/{pipeline,bsms,bitcoin_core}.rs` — `use regex::Regex` pattern is standard across the module.
- No GUI lockstep (no clap surface change).
- No manual mirror (internal discriminator only).

**SemVer rationale (v0.31.3 → v0.31.4 PATCH):**
- Pure defensive hardening; no externally visible behavior change under current Sparrow emit invariants.
- No CLI surface change.

## File structure

### Source files modified (toolkit)
- `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs`:
  - L338: replace `let has_at_placeholder = script_template.contains("@0/**");` with **inline** `Regex::new(...).expect(...).is_match(...)` per the established project pattern at `sparrow.rs:555/566/678`, `bsms.rs:501/520`, `bitcoin_core.rs:530/553/561`, `pipeline.rs:38`, `electrum.rs:920`, `specter.rs:358/467`, `coldcard.rs:507`. R0 I1 fold — NO `LazyLock` (grep confirms zero usages in the crate); inline-per-call mirrors the precedent.
  - Update the inline comment at L326-329 to document the regex widening + cite the closed FOLLOWUP slug + reference the precedent at `sparrow.rs:566`.

### Test files modified (toolkit)
- `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` (in-file `tests` mod):
  - **Regex-unit cell** `at_placeholder_regex_matches_only_template_mode_shapes` — assert positive cases `"@0/**"`, `"@1/**"`, `"@10/**"`, `"wpkh(@1/**)"` match; negative cases `"@/**"`, `"@0/*"`, `"@a/**"`, `""`, `"tr([5436d724/86'/0'/0']xpub.../<0;1>/*)"` (a descriptor-passthrough shape) do NOT match. Deterministic; no hypothetical Sparrow blob construction.
  - **Backward-compat cell** `parse_at_0_placeholder_still_routes_to_template_mode_substitution` — re-asserts an existing fixture (e.g., `sparrow-singlesig-p2wpkh.json` which carries `wpkh(@0/**)`) still routes through template-mode substitution. Locks the no-behavior-change claim.

### Release tooling
- `crates/mnemonic-toolkit/Cargo.toml:3` — `0.31.3` → `0.31.4`.
- `CHANGELOG.md` — new `## [0.31.4]` section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.31.3` → `mnemonic-toolkit-v0.31.4`.
- `design/FOLLOWUPS.md` — close `sparrow-import-detection-regex-defensive-widening`.

## Tasks

### Task 1: Phase 2 — Replace substring check with regex

**Files:** modify `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs`.

- [ ] **Step 1: Add the regex check**

Per R0 I1 fold — use the established project pattern (inline `Regex::new().expect()` per call; `parse()` is not in a hot loop). Mirrors `sparrow.rs:566` precedent.

Replace at L338:
```rust
// Before:
let has_at_placeholder = script_template.contains("@0/**");
// After:
let has_at_placeholder = regex::Regex::new(r"@\d+/\*\*")
    .expect("at-placeholder regex is a fixed string literal")
    .is_match(script_template);
```

`use regex::Regex` is already in scope via other sparrow.rs callsites (sparrow.rs:566); no new import.

- [ ] **Step 2: Update the path-split comment block**

Document the regex-based widening + cite the closed FOLLOWUP.

- [ ] **Step 3: Add 2 test cells (R0 I2 fold)**

**Regex-unit cell** `at_placeholder_regex_matches_only_template_mode_shapes` — direct regex assertion against positive + negative case lists.

**Backward-compat cell** `parse_at_0_placeholder_still_routes_to_template_mode_substitution` — uses the existing `sparrow-singlesig-p2wpkh.json` fixture (carries `wpkh(@0/**)`). Asserts that parse() succeeds + produces a descriptor with the substituted `[fp/84'/0'/0']xpub.../<0;1>/*` shape (matching the existing fixture_singlesig_p2wpkh_parses_clean cell's contract).

- [ ] **Step 4: Build + run lib tests**

```bash
cargo build --package mnemonic-toolkit 2>&1 | tail -3
cargo test --package mnemonic-toolkit --bin mnemonic wallet_import::sparrow 2>&1 | tail -10
```

- [ ] **Step 5: Commit Phase 2**

### Task 2: Phase 3 — Cycle close

- [ ] **Step 1: Bump version + install.sh self-pin + CHANGELOG entry**
- [ ] **Step 2: Full pre-tag audit (cargo test --workspace + clippy)**
- [ ] **Step 3: Opus end-of-cycle review BEFORE tag**
- [ ] **Step 4: Commit + tag mnemonic-toolkit-v0.31.4 + push + GH Release**
- [ ] **Step 5: Wait for install-pin-check CI green**
- [ ] **Step 6: Close FOLLOWUP + update memory**

## Cross-phase invariants

- Opus R0 review on plan-doc BEFORE Phase 2 dispatch (Cycle 7-9 lesson — even for trivial-scope cycles).
- Opus end-of-cycle review BEFORE tagging.
- No `cargo fmt --all`.
- No GUI lockstep (no clap surface change).
- install-pin-check CI gate.

## Risk register

- **Regex correctness** — `r"@\d+/\*\*"` matches `@<digits>/**`. The `/**` suffix needs escaping (the slashes are literal, `**` is two literal asterisks). Verify: `@0/**` matches; `@99/**` matches; `@0/*` does NOT match; `@/**` does NOT match.
- **LazyLock MSRV** — `std::sync::LazyLock` is stable in Rust 1.80+; Cargo workspace `rust-version` is 1.85; safe.
- **Test cell construction** — Sparrow's policyType + keystores structure must be valid to reach the Step 6 discriminator. A blob with only `@1/**` but no `@0/**` is hypothetical; the regression cell may need to be inline assertion only (test the regex compiled OK + matches expected strings) rather than a full parse() invocation.

## Self-review (pre-R0 dispatch)

- ✓ P0 recon confirms L338 still applies + regex dep available.
- ✓ Trivial scope; 1-line code change + 1 regression cell.
- ✓ SemVer PATCH justified (defensive hardening; no behavior change under current Sparrow emit invariants).
- ✓ No GUI / manual / wire-shape impact.
