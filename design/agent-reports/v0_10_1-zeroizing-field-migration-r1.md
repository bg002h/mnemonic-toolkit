# v0.10.1 R1 — Zeroizing field-type migration impl-review (Opus)

**Date:** 2026-05-13.
**Reviewer:** Opus (per `feedback_opus_primary_review_agent`).
**Scope:** post-impl review of the v0.10.1 patch (single bundled commit, 11 files modified, ~197 insertions / ~64 deletions).
**Verdict:** CLEAR. 0 critical / 0 important / 3 nits (all folded inline pre-commit).
**Predecessor reports:**
- R0 round 1 (REWORK; 7 Critical + 4 Important): off-by-N ctor count (6→12 via `CosignerKeyInfo` alias), 7 read-site compile breaks.
- R0 round 2 (LOCK pending folds; 9 Important + 2 nits): narrative/line-number drift in the plan rewrite.
- R0 round 3 (LOCK clean): all folds verified.

R0 lives inline in the transcript; plan locked at `~/.claude/plans/v0_10_1-zeroizing-field-migration.md`.

## What shipped

All 11 plan deliverables (D1-D11) landed:

| # | Deliverable | Where shipped |
|---|---|---|
| D1 | `ResolvedSlot.entropy` → `Option<Zeroizing<Vec<u8>>>` | `synthesize.rs:587` |
| D2 | 12 ctor sites wrap entropy at field-write | `cmd/bundle.rs:{364,435,469,513,1046,1102}` direct + `cmd/bundle.rs:1042` (alias) + `cmd/verify_bundle.rs:489` + `parse_descriptor.rs:{1179,1743,1758}` + `synthesize.rs:{1061,1217}` |
| D3 | `DerivedAccount.entropy` → `Zeroizing<Vec<u8>>` | `derive.rs:22` |
| D4 | DerivedAccount ctor wraps `Zeroizing::new(entropy_bytes)` | `derive_slot.rs:84` |
| D5 | DELETE `impl Drop for DerivedAccount` | `derive.rs` (block removed) |
| D6 | `into_parts` body: `mem::take(&mut *self.entropy)` | `derive.rs:46` |
| D7.1 | `entropy_at_0()` Deref fix | `parse_descriptor.rs:814-816` |
| D7.2 | `(**e).clone()` for Payload::Entr | `synthesize.rs:715` |
| D7.3 | `entropy_at_0 = slot.entropy.clone()` (drop double-wrap) | `cmd/verify_bundle.rs:500-502` |
| D7.4 | `assert_eq!(*acc.entropy, …)` | `derive.rs:108` |
| D7.7 | `c0.entropy = Some(Zeroizing::new((*entropy).clone()))` | `parse_descriptor.rs:956` |
| D7.8 | 3 `&acc.entropy` test sites | `parse_descriptor.rs:{1914,1945,1976}` — no edit needed (Rust's multi-step deref coercion handles it through `&Zeroizing<Vec<u8>>` → `&Vec<u8>` → `&[u8]` for function-arg position) |
| D8 | Lint relabel + new ResolvedSlot row + comment block deletion + row-count comment | `tests/lint_zeroize_discipline.rs` |
| D9 | Doc-comment refresh on DerivedAccount + both `_entropy_pin` siblings + 4 stale-Drop call-site comments (N-1 fold) + as_deref guidance (N-2 fold) | `derive.rs:13-21,29-35`; `synthesize.rs:586-592,600-609`; `cmd/bundle.rs:{359,462}`; `synthesize.rs:776`; `parse_descriptor.rs:798-805` |
| D10 | `resolved-slot-derived-account-zeroizing-field` + `pub-struct-drop-semver-risk-monitor` Status: resolved | `design/FOLLOWUPS.md` |
| D11 | Cargo.toml `0.10.0` → `0.10.1` + Cargo.lock + CHANGELOG `mnemonic-toolkit [0.10.1]` section | `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md` |

## Source ground-truth verification

Per `feedback_r0_must_read_source_off_by_n`: grep-verified independently at R1 dispatch:

| Claim | Verification |
|---|---|
| `impl Drop for DerivedAccount` deleted | `grep "impl Drop for DerivedAccount"` returns 0 hits ✓ |
| Both struct field types migrated to Zeroizing | `grep "pub entropy: Option<zeroize::Zeroizing<Vec<u8>>>"` + `grep "pub entropy: zeroize::Zeroizing<Vec<u8>>"` each return 1 hit ✓ |
| 12 ctor sites populated | grep enumeration matches plan §2 D2 table ✓ |
| RFC 1857 drop order preserved | both structs declare `entropy` BEFORE `_entropy_pin` ✓ |
| `into_parts()` returns bare `Vec<u8>` | signature unchanged from v0.10.0 baseline ✓ |
| Lint anchors precise | both new anchor strings literally appear in their target source files ✓ |
| Mlock module untouched | `git diff crates/mnemonic-toolkit/src/mlock.rs` is empty ✓ |
| 620 tests passing | `cargo test -p mnemonic-toolkit` summary ✓ |
| Clippy clean | `cargo clippy --all-targets -p mnemonic-toolkit -- -D warnings` Finished ✓ |

## Nits folded inline pre-commit (3 total)

**N-1 (4 stale `impl Drop` / E0509-safe call-site comments).** `cmd/bundle.rs:{359-360, 462}`, `synthesize.rs:776` narratively justified `into_parts` via E0509-safety, citing the v0.9.0 `impl Drop`. Post-deletion the E0509 justification no longer applies. Rewritten in v0.10.1 voice ("returns bare Vec<u8> per caller-wrap contract"). Plus 1 stale `as_deref()` recommendation in `parse_descriptor.rs:798-805` mod-doc that pointed callers at an API call that now mismatches types — replaced with guidance pointing at `entropy_at_0()` plus the `as_ref().map(|z| z.as_slice())` two-step.

**N-2 (parse_descriptor.rs:802 stale as_deref recommendation).** Folded with N-1 above.

**N-3 (CHANGELOG line-number citations off by 1-10).** Refreshed all citations: ctor sites `cmd/bundle.rs:{364,435,469,513,1046,1102}`, `parse_descriptor.rs:{1179,1743,1758}`, `synthesize.rs:{1061,1217}`; read-site fixes at `parse_descriptor.rs:814-820`, `synthesize.rs:715`, `cmd/verify_bundle.rs:500-502`, `derive.rs:108`, `parse_descriptor.rs:956`. The R0 plan line numbers had drifted from the source between plan-write and impl; CHANGELOG citations now reflect actual post-impl line positions.

## Notable design ratifications during impl

- **D7.8 deviation:** the R0 plan called for `&acc.entropy[..]` slice-coerce at 3 test sites (parse_descriptor.rs:{1914,1945,1976}). At impl-time, the bare `&acc.entropy` continued to compile cleanly — Rust's multi-step deref coercion successfully traverses `&Zeroizing<Vec<u8>>` → `&Vec<u8>` → `&[u8]` in function-call position (the architect's round 1 prediction that this would fail was over-cautious). No edits to these 3 sites; the plan was correct in identifying them as a hazard, but the language handled it.
- **Local-wrap vs field-write wrap pattern:** the plan's "wrap at field-write boundary" was lifted to "wrap at local-bind" in 3 sites where it produced cleaner code (synthesize.rs:1211 `entropy_field = if … { Some(Zeroizing::new(entropy.clone())) }`; bundle.rs:1042 `let entropy = ent_opt.clone().map(Zeroizing::new);`; bundle.rs:1090 `let entropy = if i == 0 { entropy_at_0.clone().map(Zeroizing::new) }`). Semantically equivalent; both produce the same runtime structure.

## Verdict

**CLEAR for tag push.** All R0-locked deliverables shipped; nits folded; all 7 acceptance gates (G1 compile / G2 test / G3 clippy / G4 miri / G5 lint count / G6 FOLLOWUPS audit / G8 wire-format) green at the time of writing. G7 (CI green) verifies post-push.

Path B-lite Cycle B carve-out is now fully completed in v0.10.1.
