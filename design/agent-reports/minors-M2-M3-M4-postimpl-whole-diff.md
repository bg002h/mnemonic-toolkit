# Post-impl whole-diff R0 — toolkit v0.84.0→v0.85.0 (M2+M3+M4) — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Uncommitted, 18 files / +153 −50; verified identical before/after; every mutation reverted + sha256-verified. Tree byte-clean.

## Green
`cargo test -p mnemonic-toolkit` = **3705/0** (206 binaries, 18 ignored). clippy `--all-targets -D warnings` clean. `make -C docs/manual lint` all 6 stages pass (markdownlint/cspell/lychee 261-OK/flag-coverage/glossary/index). Never `cargo fmt`; `mlock.rs` diff 0 bytes.

## M2 — byte-identical at slip39.rs:757 ↔ cli_slip39_refusals.rs:336 ↔ 41-mnemonic.md:2194 (python string-compare). Verified in impl: `slip39/mod.rs:312` digest HMAC (`:454`) runs on the reconstructed EMS BEFORE `feistel::decrypt` (`:322`) — can't see the passphrase; "wrong passphrase still exits 0 with a different master" accurate. No new false implication; verify-address note (`:683`) intact.

## M3 — arm `(9, bip39::Language::Portuguese)` (`derive_child.rs:352`) + doc row. Portuguese=9' confirmed vs authoritative bip-0085 (codes 0'-8' also match). Test `bip39_portuguese_diverges_from_english` = divergence + wordlist-membership (no `!is_ascii`). **RED: revert arm→Err → test FAILED, reverted sha-clean.** English output = the official BIP-85 vector.

## M4 (funds-adjacent) — both loci `import_wallet.rs:426-429` (standalone) + `:1380-1383` (combined) = `any(Failed)→Ok(4)`, report/envelope printed; strict errors before either → exit 2 unchanged; plain import round1_verifications empty → exit 0. **RED (a) revert `:429`→Ok(0): cell_5 + encrypted case FAILED, combined still PASSED. RED (b) revert `:1383`: combined FAILED, cell_5 still PASSED → each locus independently guarded.** Direct binary: standalone-flip→4(+NOTICE+envelope), good→0, strict→2, combined-flip→4(+bundle envelope, signature_verified:false), combined-good→0, plain→0. Non-collision (`ImportWalletSeedMismatch=4` blob-overlay mode, same VERIFY-ME class; 2/3/5 taken). Manual exit-code table row 4 (`41-mnemonic.md:1407`, beyond-SPEC addition) factually correct + consistent.

## Release ritual — 0.84.0→0.85.0 at EVERY site (repo-wide `0.84.0` grep = 0): Cargo.toml:3, both README markers, fuzz/Cargo.lock:578-579, root Cargo.lock, install.sh:32 self-pin. **Sibling pins UNCHANGED** (md-cli-v0.11.2 frozen baseline, ms-cli-v0.14.1, mk-cli-v0.12.0, gui-v0.51.0). gen.sh 6 pins (`:3/:44/:109/:126/:711/:724`). CHANGELOG `[0.85.0]` with the loud `$?`-BREAKING callout + GUI-lagging note (matches changelog-check grep). **No dep drift** (both lockfiles = 1+/1− version-string only → no re-vendor).

## Examples — `Examples.md` diff = version-banner only (6 hunks); **regen via `EXAMPLES_BIN_DIR=$PWD/target/debug bash .examples-build/gen.sh` (CI recipe) byte-identical (`cmp` clean)** → `git diff --exit-code` GREEN. (PATH-prepend fails — gen.sh:23 prepends ~/.cargo/bin with a stale 0.75.0; `EXAMPLES_BIN_DIR` required.) `docs/Examples.pdf` valid, 25 pages, 8× `0.85.0`/0× `0.84.0`.

## Findings
Critical: none. Important: none.
**Minor (3, non-blocking → FOLLOWUP):**
1. `src/slip39/error.rs:76-79` (doc) + `:176-179` (library `Display`) still carry the falsified "wrong passphrase" claim — **verified UNREACHABLE from the CLI** (no `From<Slip39Error>`; all CLI paths route through `map_slip39_error`). Worth a library-layer sweep.
2. `docs/manual/src/30-workflows/3A-bsms-round1-verify.md` (~:65-96) doesn't mention the new lenient exit 4 (not wrong, incomplete).
3. `41-mnemonic.md:1421` stderr-templates row labels the encrypted-Round-1 NOTICE "(exit 0)" — can now co-occur with exit-4 (cosmetic, pre-existing gap).

## VERDICT: GREEN (0C/0I) — ready to commit + tag `mnemonic-toolkit-v0.85.0` on CI-green. The 3 Minors are follow-up-file candidates.

---
**SHIP (opus, 2026-07-11):** GREEN. 3 Minors filed as FOLLOWUPs (`slip39-library-layer-passphrase-message-sweep`, `bsms-round1-lenient-exit4-workflow-doc`, `mnemonic-stderr-template-encrypted-notice-label`). Committing + pushing v0.85.0; tag after CI green. Then docs implementer into the clean tree.