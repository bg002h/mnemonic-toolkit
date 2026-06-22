# R0 REVIEW (verbatim) — cycle-15 Group A IMPLEMENTATION PLAN

**Dispatched:** 2026-06-21/22. **Base:** `9b7c78a7` (v0.68.0). **Plan:** `design/PLAN_cycle15_groupA_bsms_lint.md`.
**Verdict:** GREEN (0C/0I) — plan ready for single-subagent implementation. (One non-blocking Minor folded post-GREEN: narrow the P1 doc-rewrite span to `:108-113`.)

---

`bsms_crypto.rs` already has 2 secret-pattern matches pre-P1 (the `derive_encryption_key`/`decrypt` Zeroizing), so P1 adding a third doesn't change its secret-bearing status — it stays counted (38) and allowlisted (`NON_ROW_SECRET_FILES`). No cross-phase coupling. Worktree is clean.

I have verified everything. Let me write the verdict.

---

## VERDICT: GREEN (0C / 0I)

I applied the plan's P1 + P2 edits in the worktree, ran the full suite + clippy, proved RED-then-GREEN and the load-bearing union, then reverted (`git status` clean — only the untracked design docs remain). Every code snippet and line citation checks out against base `9b7c78a7`.

### What I ran (all in `wt-cycle15t`, reverted after)
- **P1 GREEN body applied** → `cargo test -p mnemonic-toolkit --lib bsms_crypto`: **21/21 pass** (20 original + the new fence), **zero caller edits**. The body `out.copy_from_slice(&hasher.finalize())` works on `Zeroizing<[u8;32]>` via DerefMut — the plan's snippet compiles exactly as written.
- **P1 RED proof** → reverted signature to bare `[u8;32]` with the fence test present → `error[E0308]: mismatched types ... expected fn pointer, found fn item`. Genuinely RED on base, GREEN under A. Sound.
- **P1 prod caller + integration helper** → `cargo build -p mnemonic-toolkit --tests` clean + `cli_import_wallet_bsms_encrypted`: **28/28 pass**, zero edits. The zero-caller-edit claim is empirically reconfirmed (matches spec R0 round-2).
- **P2 full edit set applied** → `lint_zeroize_discipline`: **6/6 pass** (4 original + `confinement_helpers_flag_production_secret_above_cfg_test` + `test_only_secret_files_confine_secret_patterns_to_cfg_test`).
- **Union is load-bearing** → removed the `.chain(TEST_ONLY_SECRET_FILES.iter())` from the partition scan → `every_secret_bearing_src_file_is_declared_or_allowlisted` FAILED with exactly `src/bundle_unified.rs` undeclared. The I1 fold is real, not vacuous.
- **Synthetic-test index math is correct** — `take(boundary)` yields indices `[0, boundary)`, so the `#[cfg(test)]` boundary line itself is **excluded**; `production_secret_lines(synthetic, 1)` = `vec![0]` (the prod `Zeroizing::new` line), asserts the right thing. The real-file guard matches: `bundle_unified.rs` cfg(test) at line 118, sole `SecretString::new(` at 128 (after boundary) → empty → GREEN.
- **Counts** — live partition = **38** (grep-confirmed, file list enumerated), floor 37 unchanged, 1 slack. P1's added `Zeroizing` doesn't perturb it (bsms_crypto.rs already secret-bearing + allowlisted).
- **nit#2 reword applied** → guard passes, all three decaying numbers (`live 54`, `60 to 66`, `36 + 16 = 52`) dropped, `{n}` retained. Range `18..=66` matches Lane T's current source.
- **Full gate** → `cargo test -p mnemonic-toolkit` (all targets, incl. `readme_version_current`) GREEN; `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Citations spot-checked (all correct)
- P1: signature `:114` ✓; body mirror `:99-102` ✓; `compute_mac` `:136` ✓. Doc-rewrite target `:106-113` — the *defending* text is precisely `:108-113` (`:106-107` are the BIP-129 formula + a blank `///` line); the plan's window is slightly wider but encompasses the right text. **Minor (non-blocking):** implementer should rewrite `:108-113` and leave `:105-107` intact, per the spec's tighter `:108-113`.
- P2: `NON_ROW_SECRET_FILES` bundle_unified entry `:498` ✓; `allowlisted` set `:547-548` ✓; partition scan `:539` ✓; tripwire `:584` ✓; floor `:510`=37 ✓; nit#2 message `:429-434` ✓; `crate_root`/`fs`/`Path` all in scope; no new-test-name collisions.
- P3: all 6 version sites at 0.68.0 — `Cargo.toml:3`, root `README.md:13`, crate `README.md:9`, `install.sh:32`, `Cargo.lock:727`, `fuzz/Cargo.lock:575`; CHANGELOG format `## mnemonic-toolkit [0.X.Y]` (latest `[0.68.0]`). FOLLOWUPS: `bsms-derive-hmac-key-not-zeroizing` (`:4478`, Status `open`) + `bundle-unified-whole-file-allowlist-precision` (`:4490`, Status `open`) both real and open; nit#3 slug correctly absent (to be added). nit#3 bip85 cites exact: `:189` `base64_standard(&entropy[..])`, `:204` `base85_btc(&entropy[..])`, `:252` `let mut out: Vec<String>`.
- SemVer-break-reaches-no-consumer reconfirmed: no fuzz/example/GUI/sibling reference to `derive_hmac_key`/`bsms_crypto` beyond the 3 known sites + the doc/`pub mod`/lint-string lines.

### Phase ordering / Do-NOT list
No cross-phase coupling: P1 (bsms_crypto.rs) and P2 (lint test) are file-disjoint; bsms_crypto.rs stays allowlisted so P1 doesn't move the partition count P2 depends on. The Do-NOT list correctly fences the no-deref-noise, no-compute_mac-wrap, no-bundle_unified-edit, floor-stays-37, and no-fmt regressions.

**Plan is ready for single-subagent implementation.** (The only nit — narrow the P1 doc-rewrite span to `:108-113` — is non-blocking and self-correcting once the implementer reads the block.)
