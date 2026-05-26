# v0.37.4 — F3 multisig-path-family derivation fix — architect review (retroactive R0)

**Context:** F3 (`--multisig-path-family bip48` ignored in seed/entropy-mode multisig bundles; derived at the BIP-87 fallback `m/87'/0'/0'` while JSON metadata reported `bip48`) was discovered mid-implementation by cell A7 of the cross-start convergence suite. The cycle's SPEC declared "test-only" and the R0 gate was waived on that basis; F3 then required a product-code change. This review is persisted retroactively per CLAUDE.md (per-phase architect outputs persist verbatim to `design/agent-reports/` before the fold/commit step) to close the governance gap (finding I1). Reviewer: feature-dev:code-reviewer (opus). Base `2dc1276`.

## Verdict

The F3 fix is **correct, complete for the CLI hot paths, and regression-free for the default (bip87) family**. The 7 convergence cells present are sound and non-vacuous. **No Critical or Important code-correctness findings.** Findings below are process/ship-readiness, not code defects.

## Critical
None.

## Important

**I1 — Product behavior change shipped under a "test-only, R0-waived" SPEC.** `design/SPEC_cross_start_convergence_and_bijection_tests.md` declared the cycle test-only and the R0 gate was waived on that basis, but the work changed product code (`resolve_slots` derivation). No persisted R0 existed for F3. Resolution: this document IS the retroactive R0; the SPEC header is corrected to record that A7 surfaced a real defect fixed in-cycle.

**I2 — `cli_standalone_bijections.rs` (B1–B6) and cell A8 absent.** The SPEC's "14 cells, 2 files" matrix is ~half complete (A1, A2, A1-neg, A4, A5, A6, A7 present). Fine as a checkpoint; must not be mislabeled complete. Deferred to FOLLOWUP `cross-start-convergence-remaining-cells`.

## Minor

**M1 — `bsms_2line` carries a redundant `#[allow(dead_code)]`** (`tests/cli_cross_start_convergence.rs`): A7 calls it, so it is live. Drop the attribute. (Applied.)

**M2 — `derive_slot.rs` module docstring "seven spines" count slightly stale** after adding the two `_at_path` helpers. Cosmetic; the new fns delegate through `derive_master_seed`. Not folded.

## Correctness & completeness analysis

- Family honored at every CLI hot-path derivation site: `resolve_slots` computes `multisig_acct_path` once (multisig-only) and applies it at phrase/seedqr, xpub path-absent fallback, and entropy branches. Single-sig stays `None` → unchanged. All five callers thread `args.multisig_path_family.unwrap_or_default()` (`bundle.rs`, `export_wallet.rs`, `verify_bundle.rs` ×3).
- **md1↔mk1 consistency holds**: `synthesize_unified` builds md1 origins from `slots[i].path` (`synthesize.rs:672-673`) and mk1 from the same `ResolvedSlot.path` — both now the family path. The JSON `multisig.path_family` field and the emitted origin path derive from the same flag; the internal inconsistency is resolved (A7 is the executable proof for bip48).
- **No missed sites** (grepped all `derivation_path`/`origin_path_str`/`md_origin_path` callers): remaining direct callers are single-sig test-only helpers, single-sig export emitters, or `path_raw`-empty fallbacks (now dead for multisig since `resolve_slots` populates `path_raw`). Descriptor mode derives at the descriptor's declared path and **refuses** `--multisig-path-family` (`bundle.rs:256`) — correctly out of scope; never affected by F3.
- **bip87 byte-identity** confirmed: `Bip87.default_origin_path` returns `m/87'/coin'/account'` ignoring `script_type` — identical to old `template.derivation_path()`. Pre-fix default-family output unchanged. Consistent with the reported 2430/0 full-suite pass.
- **Edge cases**: tr+bip48 → `m/48'/.../3'` is acceptable (honors an explicit flag; matches the pre-existing `unwrap_or(0)`+script_type convention in `synthesize.rs:344-345` and `xpub_search/candidate_paths.rs:90`). `bip48_script_type().unwrap_or(0)` for single-sig is unreachable (gated by `is_multisig`).

## Test analysis

7 cells well-constructed and non-vacuous: A1-neg pins the fingerprint as load-bearing in mk1; keys derived in-test from the seed (immune to the F2 mislabeled-fixture trap); F1 "honest scoped convergence" framing correct (bitcoin-core `/0/*` split vs BSMS `/<0;1>/*` multipath). A6/A7 are the most order-sensitive (cosigner declaration order through BSMS import) — suspect import-side reordering if they ever flake.

## Ship recommendation

- **5a — Ship the F3 fix + 7 green cells now; finish A8 + B1–B6 as a follow-on.** The fix is correct/complete/proven (A7); the remaining cells are orthogonal coverage, not gating. File a FOLLOWUP.
- **5b — PATCH release v0.37.4, full Phase-6 checklist, after this retroactive R0.** Product behavior change (bip48 multisig output wrong→correct) ⇒ not a no-bump candidate; SemVer-PATCH (bug fix, no new flag, no wire-shape addition — `path_family` field already existed; its value now matches reality). **GUI schema-mirror NOT tripped** (pre-existing flag; only an internal `resolve_slots` param added) ⇒ no paired mnemonic-gui change. Note: GUI already emits `--multisig-path-family bip48` for multisig defaults (FOLLOWUPS.md `gui-bsms-...`), so GUI bip48 output silently corrects on the pin bump. Phase-6: Cargo.toml + stage Cargo.lock + both README version markers + install.sh self-pin + CHANGELOG `[0.37.4]` + FOLLOWUP + corrected SPEC header; clean tree before checkout→ff→tag→push.
