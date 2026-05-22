# import-wallet secret-memory hygiene finish (v0.34.1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the two `import-wallet` secret-memory-hygiene FOLLOWUPs spun out of v0.33.3 — `import-wallet-plaintext-blob-mlock-pin` (pin the wallet blob for ALL formats, not just BIE1) and `bsms-decrypt-record-string-zeroizing` (return `Zeroizing<String>` from `decrypt_bsms_record`).

**Architecture:** Two localized, internal type/lifetime changes in `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — no CLI/wire/output surface change. Both are **non-observable hardening** (mlock page-pinning and zeroize-on-drop produce no asserted output), so verification is **compile + full-regression-shows-no-behavior-change + reviewer confirms the property by construction**, NOT classic failing-test TDD (see the per-task notes for the R0-corrected justification).

**Tech Stack:** Rust; `zeroize::Zeroizing` (already imported at `import_wallet.rs:88`); `mnemonic_toolkit::mlock::pin_pages_for(&[u8]) -> PinnedPageRange` (`src/mlock.rs:90`; `Drop` munlocks — and per `man mlock`, locks do NOT stack, so a single `munlock` unlocks regardless of lock count). Spec input: `cycle-prep-recon-bsms-decrypt-record-string-zeroizing+import-wallet-plaintext-blob-mlock-pin.md` + `design/FOLLOWUPS.md` + opus R0 review `design/agent-reports/v0_34_1-plan-r0-review.md`. **Source baseline: `6576cbf`** (citations re-verified — v0.34.0 did NOT touch `import_wallet.rs`).

**SemVer:** PATCH → **v0.34.1**. **No GUI/manual/schema-mirror/sibling lockstep** (internal type/lifetime only; `decrypt_bsms_record` is a file-private `fn`). **Release lesson baked in (v0.34.0):** the `scripts/install.sh` self-pin MUST be bumped in the version-bump commit (Task 3), or `install-pin-check` CI fails on the tag.

---

## File structure
- **Modify** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — both hygiene changes (Tasks 1 + 2).
- **Modify** `crates/mnemonic-toolkit/Cargo.toml` (version), `scripts/install.sh` (self-pin), `CHANGELOG.md`, `design/FOLLOWUPS.md` (Task 3).

Order: Task 1 (mlock) then Task 2 (zeroize), so Task 2's rewrite of the Round-2 reassign RHS lands cleanly next to Task 1's re-pin line.

---

## Task 1: mlock-pin the wallet blob for ALL formats (`import-wallet-plaintext-blob-mlock-pin`)

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (blob binding `:390`; BIE1 reassign `:434` + existing `_pin_pt` `:435`; Round-2 reassign `:1043`)

**Pinning model (R0 C1 — avoid the stale-`munlock` hazard).** `blob` is `let mut` and reassigned on the BIE1 (`:434`) and Round-2 (`:1043`) arms. A run-scoped guard created ONCE at `:391` and left alive would, at end of `run()`, `munlock` the FREED original page range — which the allocator may have re-handed to a still-live secret buffer (the replacement `blob`, pinned by another guard). Because `mlock` locks do not stack (`man mlock`), that stale `munlock` would silently UN-pin a live secret (including the BIE1 seed-bearing recovered JSON) — a regression vs today's `:435` pin. **FIX:** keep a SINGLE `let mut _pin_blob` guard and REASSIGN it at each reassign site. The reassignment first evaluates `pin_pages_for(&blob)` (pinning the new buffer), then drops the prior guard (munlocking the just-freed original — which has not yet been realloc'd, since the new buffer was allocated before the `blob = …` move). Invariant: exactly ONE live blob guard at all times; no end-of-`run()` `munlock` of a realloc'd range. This REPLACES the arm-local `_pin_pt` (`:435`).

**Why no new test (R0 I1 — corrected).** The earlier "cannot be asserted" claim was WRONG: the codebase has `mlock::attempts_for_test()` + assertion tests (`slip39/mod.rs:613`, `bip85.rs:411`, `derive_child.rs:463`). We nonetheless decline a `run()`-driving attempts-counter test here, for two reasons: (a) after this fix the `_pin_blob` pin at `:391` is **unconditional** — it executes for EVERY format before any branch, so the plaintext-seed-bearing-path coverage is guaranteed by construction (a regression would require deleting the single, highly-visible top-of-`run()` line — unlike a branch-specific pin worth a guard); and (b) `run()` has no unit-test harness (the `mod tests` at `:2415` exercises only small helpers like `is_encrypted_bsms_record`), so constructing the ~13-field `ImportWalletArgs` (no `Default`) + a plaintext-wallet fixture for one counter assertion is disproportionate to a PATCH cycle. Verification = compiles + `cargo clippy --all-targets -- -D warnings` clean + full `cli_import_wallet*` regression unchanged + this review.

- [ ] **Step 1: Pin at the binding (single mut guard).** Immediately after `import_wallet.rs:390` (`let mut blob = read_blob(blob_path, stdin)?;`), add:
```rust
    // v0.34.1 — pin the blob for ALL formats (was BIE1-only). A plaintext
    // `use_encryption:false` Electrum wallet is seed-bearing yet was swappable.
    // `blob` is reassigned on the BIE1 (:434) and Round-2 (:1043) arms; this
    // SINGLE guard is REASSIGNED at each so exactly one live guard pins the
    // current buffer (reassigning drops the stale guard → munlocks the freed
    // original immediately, never at end-of-run against a realloc'd page —
    // mlock locks don't stack, so a stale munlock would un-pin a live secret).
    let mut _pin_blob = mnemonic_toolkit::mlock::pin_pages_for(&blob);
```

- [ ] **Step 2: Re-pin at the BIE1 reassign.** REPLACE the existing line at `:435` — `let _pin_pt = mnemonic_toolkit::mlock::pin_pages_for(&blob);` — with a reassignment of the single guard (this drops the original-buffer guard → munlocks the freed original):
```rust
            _pin_blob = mnemonic_toolkit::mlock::pin_pages_for(&blob);
```

- [ ] **Step 3: Re-pin at the Round-2 reassign.** After the `:1043` reassign (whose RHS Task 2 rewrites), add:
```rust
        _pin_blob = mnemonic_toolkit::mlock::pin_pages_for(&blob);
```

- [ ] **Step 4: Build + clippy + no-regression.** Run `cargo build -p mnemonic-toolkit` → compiles. Run `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`. The reassigned RAII guard's value is "used" only via its `Drop` (munlock-on-overwrite); if clippy fires `unused_assignments` on `_pin_blob`, that lint does not model `Drop` side-effects — add `#[allow(unused_assignments)]` on the `let mut _pin_blob` statement WITH a one-line comment ("each reassignment's Drop munlocks the prior buffer — the assignment IS the effect"). Resolve per the actual clippy output (the underscore prefix already suppresses `unused_variables`). Run `cargo test -p mnemonic-toolkit import_wallet` + `cargo test -p mnemonic-toolkit --test cli_import_wallet_electrum_bie1 --test cli_import_wallet_bsms --test cli_import_wallet_bsms_encrypted --test cli_import_wallet_electrum` → ALL pass UNCHANGED (hardening must not change behavior).

- [ ] **Step 5: Commit** (stage ONLY import_wallet.rs):
```bash
git add crates/mnemonic-toolkit/src/cmd/import_wallet.rs
git commit -m "feat(import-wallet): mlock-pin the wallet blob for all formats (single re-pinned guard)" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: zeroize `decrypt_bsms_record`'s plaintext (`bsms-decrypt-record-string-zeroizing`)

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (`decrypt_bsms_record` `:2161`/return `:2186`; Round-2 consumer `:1043`; Round-1 consumer `:2299`-`:2313`)

**Why no failing test:** zeroize-on-drop is non-observable. Verification = compiles + full regression unchanged. The win: the intermediate decrypted BSMS descriptor `String` is scrubbed on drop instead of lingering.

- [ ] **Step 1: Change the return type + wrap.** At `decrypt_bsms_record` (`:2161`): change `-> Result<String, ToolkitError>` to `-> Result<Zeroizing<String>, ToolkitError>`. At the return (`:2186`), wrap:
```rust
    String::from_utf8(plaintext.to_vec())
        .map(Zeroizing::new)
        .map_err(|_| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: {ctx}: decrypted record is not valid UTF-8"
            ))
        })
}
```
(`plaintext` is already `Zeroizing<Vec<u8>>` from `bsms_crypto::decrypt`; its own buffer is independently scrubbed. `Zeroizing` is imported at `:88`.)

- [ ] **Step 2: Fix the Round-2 consumer (`:1043`).** `Zeroizing<String>` cannot `.into_bytes()` (no move-out through `Deref`). Change `blob = Zeroizing::new(plaintext.into_bytes());` to:
```rust
        blob = Zeroizing::new(plaintext.as_bytes().to_vec());
```
(`as_bytes()` borrows the inner `str`; `.to_vec()` copies into a fresh `Vec<u8>` that is immediately wrapped in `Zeroizing` and becomes the `blob` buffer — scrubbed on drop, no un-scrubbed copy persists. Task 1 Step 3's `_pin_blob` re-pin line follows this.)

- [ ] **Step 3: Fix the Round-1 consumer.** The `if`/`else` (`:2289`-`:2313`) binds `text`; the `if` arm yields `plaintext` (now `Zeroizing<String>`), so the `else` arm at `:2311`-`:2312` (`raw_text` is on line `:2312`) must unify. Change:
```rust
        } else {
            raw_text
        };
```
to:
```rust
        } else {
            Zeroizing::new(raw_text)
        };
```
The downstream `let record = parse_round1(&text)?;` (`:2314`) is unchanged — `parse_round1(text: &str)` (`wallet_import/bsms_round1.rs:84`) accepts `&Zeroizing<String>` via `Zeroizing<String> → String → str` deref coercion.

- [ ] **Step 4: Build + no-regression.** `cargo build -p mnemonic-toolkit` → compiles (exactly two `decrypt_bsms_record` consumers — `:1033`/`:1043` Round-2, `:2299`/`:2310` Round-1; the compiler names any other). `cargo test -p mnemonic-toolkit --test cli_import_wallet_bsms --test cli_import_wallet_bsms_encrypted` → ALL pass unchanged (decrypted BSMS records still parse + BIP-322-verify identically). `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

- [ ] **Step 5: Commit** (stage ONLY import_wallet.rs):
```bash
git add crates/mnemonic-toolkit/src/cmd/import_wallet.rs
git commit -m "feat(import-wallet): wrap decrypt_bsms_record plaintext in Zeroizing<String>" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: ship v0.34.1 (version + install.sh self-pin + CHANGELOG + FOLLOWUPS + full regression)

**Files:** Modify `crates/mnemonic-toolkit/Cargo.toml:3`, `scripts/install.sh:32`, `CHANGELOG.md`, `design/FOLLOWUPS.md`

- [ ] **Step 1: Bump version.** `crates/mnemonic-toolkit/Cargo.toml:3` → `version = "0.34.1"`.

- [ ] **Step 2: Bump the install.sh self-pin (the v0.34.0 CI lesson).** `scripts/install.sh:32` reads `…|mnemonic-toolkit-v0.34.0|no|`; change `mnemonic-toolkit-v0.34.0` → `mnemonic-toolkit-v0.34.1` (verify: `grep -n 'mnemonic-toolkit-v[0-9]' scripts/install.sh`). MUST be in the version-bump commit or `install-pin-check` CI fails on the tag.

- [ ] **Step 3: CHANGELOG entry.** Add above the `[0.34.0]` entry:
```markdown
## mnemonic-toolkit [0.34.1] — <YYYY-MM-DD>

**SemVer-PATCH — import-wallet secret-memory hygiene.** Closes two FOLLOWUPs from v0.33.3: (1) `import-wallet-plaintext-blob-mlock-pin` — the wallet `blob` is now `mlock`-pinned for ALL formats via a single re-pinned guard (previously only the BIE1 arm), so a plaintext seed-bearing Electrum wallet no longer sits swappable; (2) `bsms-decrypt-record-string-zeroizing` — `decrypt_bsms_record` returns `Zeroizing<String>` so the intermediate decrypted BSMS record is scrubbed on drop. Internal type/lifetime only — no CLI/wire/GUI/manual surface change.
```

- [ ] **Step 4: FOLLOWUPS.md.** Mark both `bsms-decrypt-record-string-zeroizing` + `import-wallet-plaintext-blob-mlock-pin` `**Status:** resolved — mnemonic-toolkit-v0.34.1` with the resolving approach (single re-pinned guard; `Zeroizing<String>` return).

- [ ] **Step 5: Full regression.** `cargo test -p mnemonic-toolkit` → ALL pass (cell count unchanged — hardening, no new behavioral tests). `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

- [ ] **Step 6: Commit** (stage explicitly):
```bash
git add crates/mnemonic-toolkit/Cargo.toml scripts/install.sh CHANGELOG.md design/FOLLOWUPS.md
git commit -m "release(toolkit): mnemonic-toolkit v0.34.1 — import-wallet secret-memory hygiene" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 7: Ship (outward-facing — controller/user-authorized).** Merge to master (FF), push, tag `mnemonic-toolkit-v0.34.1` AFTER the install.sh-bumped commit, GH release. NO GUI lockstep (no CLI surface change).

---

## Self-review (writing-plans checklist)

**1. Spec coverage:** `import-wallet-plaintext-blob-mlock-pin` → Task 1 (single re-pinned `_pin_blob` guard, R0-C1-correct). `bsms-decrypt-record-string-zeroizing` → Task 2 (`Zeroizing<String>` return + both consumers). Recon subtleties (blob reassigned `:434`/`:1043`; `Zeroizing<String>` can't `.into_bytes()`; Round-1 else-arm unify) → covered. R0 C1 (stale-munlock) → Task 1 pinning model. R0 I1 (testability) → Task 1 "Why no new test" corrected. R0 M1 (line label) → Task 2 Step 3 (`raw_text` at `:2312`). Version/install.sh/no-lockstep → Task 3.

**2. Placeholder scan:** only `<YYYY-MM-DD>` (fill at commit). No logic placeholders.

**3. Type consistency:** `decrypt_bsms_record -> Result<Zeroizing<String>, ToolkitError>`; both consumers unify to `Zeroizing<String>`; `parse_round1(&str)` unchanged via deref. `_pin_blob` is the single `let mut` guard reassigned at `:435`(replaced)/`:1043`-after; clippy resolution noted.

**R0 fold status:** C1 (Critical) folded — single re-pinned guard replaces the parallel-guard hazard. I1 (Important) folded — justification corrected (by-construction unconditional pin + disproportionate harness cost), declining the test with an accurate rationale. M1 folded. Re-dispatch R0 (→ R1) to confirm 0C/0I before implementation.
