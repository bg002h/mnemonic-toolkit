# import-wallet secret-memory hygiene finish (v0.34.1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the two `import-wallet` secret-memory-hygiene FOLLOWUPs spun out of v0.33.3 — `import-wallet-plaintext-blob-mlock-pin` (pin the wallet blob for ALL formats, not just BIE1) and `bsms-decrypt-record-string-zeroizing` (return `Zeroizing<String>` from `decrypt_bsms_record`).

**Architecture:** Two localized, internal type/lifetime changes in `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — no CLI/wire/output surface change. Both are **non-observable hardening** (mlock page-pinning and zeroize-on-drop produce no asserted output), so verification is **compile + full-regression-shows-no-behavior-change + reviewer confirms the property by construction**, NOT classic failing-test TDD.

**Tech Stack:** Rust; `zeroize::Zeroizing` (already imported at `import_wallet.rs:88`); `mnemonic_toolkit::mlock::pin_pages_for` (`src/mlock.rs:90`). Spec input: `cycle-prep-recon-bsms-decrypt-record-string-zeroizing+import-wallet-plaintext-blob-mlock-pin.md` + `design/FOLLOWUPS.md`. **Source baseline: `6576cbf`** (citations re-verified against current master — v0.34.0 did NOT touch `import_wallet.rs`, so all recon line numbers hold).

**SemVer:** PATCH → **v0.34.1** (the recon's "v0.33.4" is stale — v0.34.0 shipped since). **No GUI/manual/schema-mirror/sibling lockstep** (no CLI surface change — internal type/lifetime only).

**RELEASE LESSON BAKED IN (from the v0.34.0 cycle):** the `scripts/install.sh` self-pin MUST be bumped in the version-bump commit (Task 3), or `install-pin-check` CI fails on the tag.

---

## File structure
- **Modify** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — both hygiene changes (Tasks 1 + 2).
- **Modify** `crates/mnemonic-toolkit/Cargo.toml` (version), `scripts/install.sh` (self-pin), `CHANGELOG.md` (Task 3).

Recommended order (per recon): Task 1 (mlock) then Task 2 (zeroize), so Task 2's rewrite of the Round-2 reassign RHS lands after Task 1 adds its re-pin line nearby.

---

## Task 1: mlock-pin the wallet blob for ALL formats (`import-wallet-plaintext-blob-mlock-pin`)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (`run()` — blob binding `:390`; Round-2 reassign `:1043`)

**Why no failing test:** mlock page-pinning has no observable behavior — it cannot be asserted in a unit/integration test (the existing `mlock_unit.rs` tests cover the `mlock` module itself; `import-wallet`'s pinning is invisible). Verification is: compiles, the full `import-wallet` test suite is unchanged (no regression), and the pin now covers the seed-bearing plaintext path. The `mlock::pin_pages_for` returns a `PinnedPageRange` RAII guard that munlocks on drop.

- [ ] **Step 1: Pin the blob at its binding (covers the plaintext seed-bearing path + all non-reassigning formats).** After `import_wallet.rs:390` (`let mut blob = read_blob(blob_path, stdin)?;`), add:

```rust
    // v0.34.1 — pin the blob for ALL formats. A plaintext `use_encryption:false`
    // Electrum wallet is seed-bearing yet was previously pinned only on the BIE1
    // arm. The blob is `Zeroizing` (v0.33.3) but was swappable until now. NOTE:
    // `blob` is reassigned on the BIE1 (`:434`) and Round-2 (`:1043`) arms; this
    // guard pins the original buffer (covers the no-reassign plaintext path), and
    // each reassign re-pins the new buffer below.
    let _pin_blob = mnemonic_toolkit::mlock::pin_pages_for(&blob);
```

- [ ] **Step 2: Re-pin after the Round-2 reassign.** The BIE1 arm already re-pins at `:435` (`let _pin_pt = …pin_pages_for(&blob);`). The Round-2 reassign at `:1043` (`blob = Zeroizing::new(plaintext.into_bytes());`) replaces the buffer with the decrypted descriptor wire and is NOT re-pinned. Add immediately after that line (note: Task 2 rewrites the reassign's RHS — the re-pin line goes AFTER whatever the reassign becomes):

```rust
        let _pin_round2 = mnemonic_toolkit::mlock::pin_pages_for(&blob);
```

- [ ] **Step 3: Build + no-regression.** Run: `cargo build -p mnemonic-toolkit` → compiles. Run: `cargo test -p mnemonic-toolkit --test cli_import_wallet` (and any other `cli_import_wallet*` test files: `cargo test -p mnemonic-toolkit import_wallet` / the `cli_import_wallet_bsms*` files) → ALL pass unchanged (this is hardening; behavior must not change). Run: `cargo clippy -p mnemonic-toolkit --bin mnemonic -- -D warnings` → clean (no `unused_variables` on `_pin_blob`/`_pin_round2` — the leading underscore suppresses it; mirror the existing `_pin_pt`/`_pin_pw` pattern).

- [ ] **Step 4: Commit** (stage ONLY import_wallet.rs):
```bash
git add crates/mnemonic-toolkit/src/cmd/import_wallet.rs
git commit -m "feat(import-wallet): mlock-pin the wallet blob for all formats" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: zeroize `decrypt_bsms_record`'s plaintext (`bsms-decrypt-record-string-zeroizing`)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (`decrypt_bsms_record` `:2161`/return `:2186`; Round-2 consumer `:1043`; Round-1 consumer `:2299`-`:2313`)

**Why no failing test:** zeroize-on-drop is non-observable. Verification = compiles + full regression unchanged. The win: the intermediate decrypted `String` (a BSMS descriptor — low sensitivity, but still secret-adjacent) is scrubbed on drop instead of lingering.

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

- [ ] **Step 2: Fix the Round-2 consumer (`:1043`).** `Zeroizing<String>` cannot `.into_bytes()` (no move-out through `Deref`). Change:
```rust
        blob = Zeroizing::new(plaintext.into_bytes());
```
to:
```rust
        blob = Zeroizing::new(plaintext.as_bytes().to_vec());
```
(`plaintext.as_bytes()` borrows the inner `str`; `.to_vec()` copies into a fresh `Vec<u8>` that is immediately wrapped in `Zeroizing` and becomes the `blob` buffer — scrubbed on drop. The transient is the blob's own buffer, so no un-scrubbed copy persists. The Task-1 `_pin_round2` re-pin line follows this.)

- [ ] **Step 3: Fix the Round-1 consumer (`:2289`-`:2313`).** The `if`/`else` binds `text`; the `if` arm yields `plaintext` (now `Zeroizing<String>`), so the `else` arm must unify. Change the `else` arm at `:2311`-`:2313`:
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

- [ ] **Step 4: Build + no-regression.** Run: `cargo build -p mnemonic-toolkit` → compiles (if any other consumer of `decrypt_bsms_record` exists, the compiler names it — there are exactly two: `:1033`/`:1043` Round-2 and `:2299`/`:2310` Round-1). Run: `cargo test -p mnemonic-toolkit cli_import_wallet` + the `cli_import_wallet_bsms*` files → ALL pass unchanged (the decrypted BSMS records must still parse + BIP-322-verify identically). Run: `cargo clippy -p mnemonic-toolkit --bin mnemonic -- -D warnings` → clean.

- [ ] **Step 5: Commit** (stage ONLY import_wallet.rs):
```bash
git add crates/mnemonic-toolkit/src/cmd/import_wallet.rs
git commit -m "feat(import-wallet): wrap decrypt_bsms_record plaintext in Zeroizing<String>" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: ship v0.34.1 (version + install.sh self-pin + CHANGELOG + full regression)

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml:3` (version), `scripts/install.sh` (self-pin), `CHANGELOG.md`

- [ ] **Step 1: Bump version.** `crates/mnemonic-toolkit/Cargo.toml:3` → `version = "0.34.1"`.

- [ ] **Step 2: Bump the install.sh self-pin (the v0.34.0 release lesson).** In `scripts/install.sh`, the `component_info` line for the toolkit reads `…|mnemonic-toolkit-v0.34.0|no|`. Change `mnemonic-toolkit-v0.34.0` → `mnemonic-toolkit-v0.34.1`. (Verify with `grep -n 'mnemonic-toolkit-v[0-9]' scripts/install.sh`.) This MUST be in the version-bump commit or `install-pin-check` CI fails on the tag.

- [ ] **Step 3: CHANGELOG entry.** Add above the `[0.34.0]` entry:
```markdown
## mnemonic-toolkit [0.34.1] — <YYYY-MM-DD>

**SemVer-PATCH — import-wallet secret-memory hygiene.** Closes two FOLLOWUPs spun out of v0.33.3: (1) `import-wallet-plaintext-blob-mlock-pin` — the wallet `blob` is now `mlock`-pinned for ALL formats (previously only the BIE1 decrypt arm), so a plaintext seed-bearing Electrum wallet no longer sits swappable; (2) `bsms-decrypt-record-string-zeroizing` — `decrypt_bsms_record` returns `Zeroizing<String>` so the intermediate decrypted BSMS record is scrubbed on drop. Internal type/lifetime only — no CLI/wire/GUI/manual surface change.
```

- [ ] **Step 4: Full regression.** Run: `cargo test -p mnemonic-toolkit` → ALL pass (cell count unchanged — no new behavioral tests; this is hardening). Run: `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

- [ ] **Step 5: Commit** (stage the three files explicitly):
```bash
git add crates/mnemonic-toolkit/Cargo.toml scripts/install.sh CHANGELOG.md
git commit -m "release(toolkit): mnemonic-toolkit v0.34.1 — import-wallet secret-memory hygiene" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 6: FOLLOWUPS.md** — mark both `bsms-decrypt-record-string-zeroizing` + `import-wallet-plaintext-blob-mlock-pin` `Status: resolved — mnemonic-toolkit-v0.34.1` with the resolving approach. Commit `docs(followups):`.

- [ ] **Step 7: Ship (outward-facing — controller/user-authorized).** Merge to master (FF), push, tag `mnemonic-toolkit-v0.34.1`, GH release. NOTE: tag AFTER the install.sh-bumped commit so `install-pin-check` passes. NO GUI lockstep (no CLI surface change).

---

## Self-review (writing-plans checklist)

**1. Spec coverage:** slug `import-wallet-plaintext-blob-mlock-pin` → Task 1 ✓ (pin at `:390` for all formats + re-pin reassigns; BIE1 `:435` already covered). slug `bsms-decrypt-record-string-zeroizing` → Task 2 ✓ (return `Zeroizing<String>` + both consumers `:1043`/`:2311`). Recon's two subtleties (blob reassigned at `:434`/`:1043`; `Zeroizing<String>` can't `.into_bytes()` + Round-1 else-arm unify) → Task 1 Step 2 + Task 2 Steps 2-3 ✓. Version v0.34.1 + install.sh self-pin + no-lockstep → Task 3 ✓.

**2. Placeholder scan:** Only `<YYYY-MM-DD>` (fill at commit time) — not a logic placeholder. No "TBD"/"handle edge cases"/etc.

**3. Type consistency:** `decrypt_bsms_record -> Result<Zeroizing<String>, ToolkitError>` used consistently; both consumers (`:1043` `as_bytes().to_vec()`; `:2311` `Zeroizing::new(raw_text)`) unify to `Zeroizing<String>`; `parse_round1(&str)` unchanged via deref. `_pin_blob`/`_pin_round2`/`_pin_pt` all `PinnedPageRange` guards (underscore-prefixed, no clippy warning).

**Open issue:** none. Both changes are non-observable hardening; the honest verification (compile + full-regression-unchanged + reviewer-confirms-by-construction) is stated per task — flag this explicitly to the R0 reviewer so the absence of new behavioral tests is understood as correct, not a gap.

## Per-cycle reviewer-loop (CLAUDE.md / mandatory standard)
Dispatch opus R0 on THIS plan-doc before any implementation; persist to `design/agent-reports/v0_34_1-plan-r0-review.md`; converge to 0 Critical / 0 Important before coding. End-of-cycle opus review before tagging.
