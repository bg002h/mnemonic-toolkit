# IMPLEMENTATION PLAN — cycle-15 Group A: bsms HMAC_KEY hygiene + lint allowlist precision

**Spec (R0-GREEN, 2 rounds):** `design/BRAINSTORM_cycle15_groupA_bsms_lint.md`. Reviews: `design/agent-reports/cycle15-groupA-spec-r0-round{1,2}-review.md`.
**Base:** `9b7c78a7` (v0.68.0). **Branch:** `feature/cycle15-groupA-bsms-lint` (worktree `wt-cycle15t`). **Target:** toolkit MINOR **0.69.0**.
**Single-subagent TDD** (one implementer, this worktree). No clap/wire/schema/manual/GUI surface → no mirror gates. Outputs byte-identical (TV3 unchanged).

Three phases, each RED→GREEN with the FULL `cargo test -p mnemonic-toolkit` suite + `cargo clippy --workspace --all-targets -- -D warnings` at the phase boundary (per the R0-full-suite discipline — CLI/lint phases ripple into version/lint tests outside any one target).

---

## P1 — #3: `derive_hmac_key` → `Zeroizing<[u8;32]>` (secret HMAC_KEY)

**Files:** `src/bsms_crypto.rs` (only). ZERO caller edits (R0-proven: 20/20 bsms tests + the integration helper pass unchanged under Option A — Deref/AsRef covers `hex::encode`, `compute_mac(&hmac_key,…)`, `mac[..]` indexing). **The implementer MUST NOT add any `*`/`&*`/deref to import_wallet.rs, the in-module tests, or cli_import_wallet_bsms_encrypted.rs.** If any caller fails to compile, STOP and report (it would contradict R0) — do not paper over with deref noise.

### P1 RED — type-fence test (add to `bsms_crypto.rs` `#[cfg(test)] mod tests`)
```rust
#[test]
fn derive_hmac_key_returns_zeroizing() {
    // Fn-pointer fence: return types are invariant, so this fails to COMPILE
    // on the bare-[u8;32] signature and compiles only once derive_hmac_key
    // returns Zeroizing<[u8;32]> (cycle-15 Group A first-class-hygiene).
    let _f: fn(&[u8; 32]) -> zeroize::Zeroizing<[u8; 32]> = derive_hmac_key;
}
```
(Confirm `zeroize::Zeroizing` is in scope or path-qualify; the module already imports `Zeroizing` — reuse the existing import.) RED = compile error on base.

### P1 GREEN — change the signature + body + docs
1. `src/bsms_crypto.rs:114` — signature `pub fn derive_hmac_key(encryption_key: &[u8; 32]) -> Zeroizing<[u8; 32]>`; body builds `let mut out = Zeroizing::new([0u8; 32]);` then fills it (mirror `derive_encryption_key`'s `out.as_mut_slice()...`/copy pattern at `:99-102`) and returns `out` by move. No bare `[u8;32]` constructed for the return.
2. **Rewrite ONLY the doc `:108-113`** (leave the BIP-129 formula + blank `///` at `:105-107` intact — plan R0 Minor). The `:108-113` text currently DEFENDS the bare return ("short-lived stack value … attacker who reads the stack already has ENCRYPTION_KEY … caller retaining … should wrap manually"). Replace with: returns `Zeroizing<[u8; 32]>` — the HMAC_KEY is secret-class (derived from the `Zeroizing` ENCRYPTION_KEY) and is scrubbed on drop; the scrub obligation lives in the return type so no caller can leak it by forgetting to wrap (cycle-15 Group A, first-class secret-hygiene).
3. `compute_mac` (`:136`) — STAYS `-> [u8; 32]` (bare). Add ONE doc line: the MAC is a published BIP-129 authentication tag (its first 16 bytes become the on-wire IV; compared against untrusted `mac_recv`) — NOT secret-class, deliberately un-wrapped.

### P1 verify
- `derive_hmac_key_returns_zeroizing` now compiles + passes.
- All 20 bsms tests GREEN with NO edits (esp. `tv3_derive_hmac_key_matches_bip129`, `tv3_compute_mac_matches_bip129`, `tv3_end_to_end_round_trip`) — TV3 MAC/IV/round-trip bytes unchanged = the funds-relevant byte-invariant.
- `tests/cli_import_wallet_bsms_encrypted.rs` GREEN unchanged.
- Full `cargo test -p mnemonic-toolkit` + clippy GREEN. (Lint: bsms_crypto.rs is whole-file-allowlisted CRYPTO-INTERNAL → no row/floor change.)
- Commit P1: `feat(cycle-15 groupA): #3 derive_hmac_key -> Zeroizing<[u8;32]> (BIP-129 HMAC_KEY secret-hygiene)`.

---

## P2 — #4: cfg(test)-confinement lint tier + nit#2 reword

**File:** `tests/lint_zeroize_discipline.rs` (only). `src/bundle_unified.rs` is NOT edited (its secret pattern stays in the cfg(test) `s()` helper).

### P2 RED — pure-helper negative test + the confinement guard
Add pure, file-free helpers + their unit tests FIRST (RED until the helpers exist):
```rust
/// Index of the first line containing `#[cfg(test)]`, if any.
fn first_cfg_test_line(src: &str) -> Option<usize> {
    src.lines().position(|l| l.contains("#[cfg(test)]"))
}
/// Line indices (0-based) of every line containing a SECRET_PATTERN that lies
/// BEFORE `boundary` (i.e. outside the test region). Substring-exact, same
/// SECRET_PATTERNS as the partition scan (NO comment-stripping — intended; a
/// SECRET_PATTERN substring in a doc/comment above #[cfg(test)] deliberately
/// trips, mirroring the partition scan's substring semantics).
fn production_secret_lines(src: &str, boundary: usize) -> Vec<usize> {
    src.lines().enumerate()
        .take(boundary)
        .filter(|(_, l)| SECRET_PATTERNS.iter().any(|p| l.contains(p)))
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn confinement_helpers_flag_production_secret_above_cfg_test() {
    let synthetic = "fn prod() { let k = Zeroizing::new([0u8;32]); }\n#[cfg(test)]\nmod t { fn s() { SecretString::new(x); } }\n";
    let b = first_cfg_test_line(synthetic).expect("has #[cfg(test)]");
    assert_eq!(production_secret_lines(synthetic, b), vec![0]); // the prod Zeroizing::new line
    // and a clean (test-confined-only) source yields no production lines:
    let clean = "fn prod() {}\n#[cfg(test)]\nmod t { fn s() { SecretString::new(x); } }\n";
    let cb = first_cfg_test_line(clean).unwrap();
    assert!(production_secret_lines(clean, cb).is_empty());
}
```

### P2 GREEN — the tier + the real-file guard + union both consumers
1. **New const** (near `NON_ROW_SECRET_FILES`):
```rust
/// Files whose ONLY secret-pattern matches live inside a `#[cfg(test)]` region
/// (test fixtures), verified by `test_only_secret_files_confine_to_cfg_test`.
/// Distinct from NON_ROW_SECRET_FILES (whole-file crypto-internal/primitive
/// exemptions) — these are exempt ONLY because the secret is test-scoped, so a
/// future PRODUCTION secret allocation above the cfg(test) marker is CAUGHT
/// (cycle-15 Group A, closing the bundle-unified whole-file-allowlist masking).
const TEST_ONLY_SECRET_FILES: &[&str] = &[
    "src/bundle_unified.rs", // the sole SecretString::new is the #[cfg(test)] s() SlotInput fixture (cycle-14 L22); SlotInput.value's canonical row is src/slot_input.rs
];
```
   Remove the `"src/bundle_unified.rs"` line from `NON_ROW_SECRET_FILES` (`:498`).
2. **Union in the partition scan** (`every_secret_bearing_src_file_is_declared_or_allowlisted`, `:547-548`):
```rust
let allowlisted: std::collections::HashSet<&str> =
    NON_ROW_SECRET_FILES.iter().chain(TEST_ONLY_SECRET_FILES.iter()).copied().collect();
```
3. **Union in the staleness tripwire** (`non_row_secret_allowlist_is_non_empty_and_each_entry_still_bears_a_secret`, `:584-615`): keep the existing `!NON_ROW_SECRET_FILES.is_empty()` assert; add `assert!(!TEST_ONLY_SECRET_FILES.is_empty(), …)`; iterate `NON_ROW_SECRET_FILES.iter().chain(TEST_ONLY_SECRET_FILES.iter())` for the exists + still-bears-a-secret checks.
4. **New real-file confinement guard:**
```rust
#[test]
fn test_only_secret_files_confine_secret_patterns_to_cfg_test() {
    let root = crate_root();
    for entry in TEST_ONLY_SECRET_FILES {
        let src = fs::read_to_string(root.join(entry))
            .unwrap_or_else(|e| panic!("read {entry}: {e}"));
        let boundary = first_cfg_test_line(&src).unwrap_or_else(||
            panic!("{entry} in TEST_ONLY_SECRET_FILES has no #[cfg(test)] marker"));
        let prod = production_secret_lines(&src, boundary);
        assert!(prod.is_empty(),
            "{entry}: secret pattern(s) at production line(s) {prod:?} (above #[cfg(test)] line {boundary}); \
             a TEST_ONLY exemption requires all secret patterns be test-scoped — move it to a canonical \
             ZEROIZE_ROWS row or NON_ROW_SECRET_FILES");
    }
}
```

### P2 GREEN — nit#2 reword (`:429-434`)
Replace the message (drop the decaying "live 54", "60 to 66", "36 + 16 = 52"; keep `{n}` live):
```rust
"ZEROIZE_ROWS row count = {n}; expected 18..=66. The upper bound carries headroom \
 above the current canonical-row count for near-term secret-site additions — widen \
 it deliberately when a cycle exceeds it. This count is a coarse drift tripwire; the \
 per-row evidence test below is authoritative. Survey §1 toolkit table is canonical."
```

### P2 verify
- New helper test + confinement guard GREEN; `every_secret_bearing_src_file_is_declared_or_allowlisted` + `non_row_secret_allowlist_…` GREEN with the union; partition live count = 38, `SECRET_FILE_FLOOR=37` unchanged.
- Full `cargo test -p mnemonic-toolkit` + clippy GREEN.
- Commit P2: `feat(cycle-15 groupA): #4 cfg(test)-confinement lint tier (close bundle_unified whole-file-allowlist masking) + reword count-guard`.

---

## P3 — release 0.69.0 + FOLLOWUPS

1. **Version sweep 0.68.0 → 0.69.0** (R0-verified site list): `crates/mnemonic-toolkit/Cargo.toml:3`, root `README.md` (`<!-- toolkit-version: -->`), crate `README.md` (`<!-- toolkit-version: -->`), `scripts/install.sh:32` self-pin, root `Cargo.lock` (toolkit package version), `fuzz/Cargo.lock` (toolkit package version), new `CHANGELOG.md` `[0.69.0]` entry. (No ms-codec pin change.)
2. **FOLLOWUPS (`design/FOLLOWUPS.md`):** flip `bsms-derive-hmac-key-not-zeroizing` + `bundle-unified-whole-file-allowlist-precision` `open → resolved` (with the v0.69.0 resolution note). ADD a new slug `bip85-encode-helper-internal-scratch-zeroizing` (nit#3): bip85 `encoded` base64/base85 password intermediate (`bip85.rs:189,204`) + dice `out: Vec<String>` (`:252`) are bare-`String` residues out of Lane T's named scope; same class as `compute_outputs` output Vec; companion to the constellation derived-output program; status `open`, tier `polish/next-cycle`. **Grep-verify the `:189,204,252` cites against current `src/bip85.rs` at write-time** (re-grep convention) — adjust to live lines.
3. **Final gate:** `cargo test -p mnemonic-toolkit` GREEN (incl. `readme_version_current` @ 0.69.0) + `cargo clippy --workspace --all-targets -- -D warnings` clean.
4. Commit P3: `release(cycle-15 groupA): toolkit v0.69.0 — bsms HMAC_KEY Zeroizing + lint cfg(test)-confinement`.

---

## Whole-diff review (orchestrator, mandatory, post-impl)
After P3, the orchestrator dispatches the independent adversarial whole-diff review (per-phase R0 deferred only if no Agent-API; here Agent-API is available so it runs). Then FF-push `HEAD:master` + tag `mnemonic-toolkit-v0.69.0` (guard: origin/master still == ship-base; tag free; coordinate vs the own-account instance — no collision expected).

## Do-NOT list (regression fences)
- Do NOT add deref (`*`/`&*`) to ANY `derive_hmac_key`/`compute_mac` caller (P1).
- Do NOT wrap `compute_mac`'s output (public auth tag).
- Do NOT edit `src/bundle_unified.rs` (the fix is entirely in the lint test).
- Do NOT change `SECRET_FILE_FLOOR` (stays 37).
- Do NOT touch any clap/CLI surface (no schema_mirror/manual mirror).
- Do NOT `cargo fmt` (mlock.rs exemption; toolkit has no fmt gate but avoid churn).
