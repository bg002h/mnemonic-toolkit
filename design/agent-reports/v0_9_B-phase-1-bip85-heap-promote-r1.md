# v0.9.0 Cycle B Phase 1 R1 architect review (bip85 heap-promote)

**Reviewer:** Opus 4.7 (1M context), invoked as architect-review on Cycle B Phase 1 post-implementation (commits `4465940`, `3be9b77`, `c3509af` atop Phase 0 close `f84d998`).
**Date:** 2026-05-13.
**SPEC:** `design/SPEC_secret_memory_hygiene_v0_9_B.md` (master @ `f84d998`).
**Plan:** `~/.claude/plans/v0_9_B-secret-memory-hygiene-cycle-b.md`, §"Phase 1" T4 checklist.
**R0 design-lock:** `design/agent-reports/v0_9_B-phase-1-bip85-heap-promote-r0.md` (commit `4465940`, LOCK).
**Scope of review:** all 12 R1 checklist items per plan §"Phase 1" T4 + R0 §7 forward-checklist additions.
**Verdict:** **CLEAR — 0 Critical / 0 Important** at confidence ≥ 80. Phase 1 ships (P1.T5).

---

## Summary

Total findings at confidence ≥ 80: **0 Critical / 0 Important / 2 Nit**.

The GREEN commit (`c3509af`) lands the return-type swap exactly as locked in R0. The RED commit (`3be9b77`) adds the two predicted tests (type-shape + byte-determinism). The lockstep update of `tests/lint_zeroize_discipline.rs` evidence anchors is present and substring-correct against the post-swap source. No callee body required editing (R0 §3 prediction held); no callee introduced `as_ref` / `into` / `try_into` / `clone` workarounds. Diff size is within R0 §7's forward prediction. No surprise additions: no Cargo.toml changes, no FOLLOWUPS edits, no manual updates, no unrelated refactors.

---

## §1. Return-type compliance (R1 checklist item 1)

| What | Expected (R0 lock) | Actual (post-`c3509af`) | Verdict |
|---|---|---|---|
| `derive_entropy` return | `Result<Zeroizing<Vec<u8>>, ToolkitError>` | `Result<Zeroizing<Vec<u8>>, ToolkitError>` at `bip85.rs:32` | PASS |
| Box-wrapped alternative | NOT taken | Not present anywhere | PASS |
| `Result<…, ToolkitError>` envelope | Preserved (Cycle A R1 I-4 fold) | Preserved | PASS |
| Visibility | `pub(crate)` (not `pub`) | `pub(crate)` at `bip85.rs:27` | PASS |
| Four parameters | `master`, `app_code`, `app_params`, `index` | All four present | PASS |

Confidence: 100. Signature matches the R0 lock exactly.

---

## §2. Implementation-quality findings (R1 checklist items 2-12)

### Item 2 — Byte-determinism unaltered

- `bip39_12_words_entropy_matches_spec` (`bip85.rs:351-355`): asserts `hex::encode(&e[..16]) == "6250b68daf746d12a24d58b4787a714b"`. Pin matches BIP-85 §"Test Vectors" verbatim and Cycle A pre-swap state.
- `hex_64_bytes_entropy_matches_spec` (`bip85.rs:359-366`): asserts the full 64-byte spec hex `492db4...82a5c`. Pin matches BIP-85 §"Test Vectors" verbatim and Cycle A pre-swap state.
- Cross-format spec vectors (`pwd_base64_matches_spec`, `pwd_base85_matches_spec`, `dice_d6_10_rolls_matches_spec`) are present and unchanged at `bip85.rs:369-388`.
- New `derive_entropy_is_byte_deterministic` (`bip85.rs:431-436`): calls derive twice with identical args, asserts equality. Will pass deterministically.

Confidence: 100. Wire-format byte-determinism preserved.

### Item 3 — All 7 callees updated correctly

Per R0 §3, no callee body required source-text edits — only the inferred binding type at `let entropy = derive_entropy(...)?` changes. Verified against current `bip85.rs`:

| # | Function | Line | Consumption | `as_ref`/`into`/`try_into`/`clone` introduced? |
|---|---|---|---|---|
| 1 | `format_bip39_phrase` | 73 | `&entropy[..bytes]` | No |
| 2 | `format_hd_seed_wif` | 100 | `&entropy[..32]` | No |
| 3 | `format_xprv_child` | 127 | `&entropy[..32]`, `&entropy[32..]` | No |
| 4 | `format_hex_bytes` | 158 | `&entropy[..num_bytes as usize]` | No |
| 5 | `format_password_base64` | 175 | `&entropy[..]` | No |
| 6 | `format_password_base85` | 189 | `&entropy[..]` | No |
| 7 | `format_dice_rolls` | 214 | `&entropy[..]` | No |

The one `.into()` hit in `bip85.rs` is on a string literal for an error message, pre-existing, unrelated. `format_dice_rolls` (the R0-caught misalignment) is correctly updated transparently. Confidence: 100.

### Item 4 — Lint evidence anchors updated in lockstep

`tests/lint_zeroize_discipline.rs:88-97`:

```rust
ZeroizeRow {
    label: "bip85::derive_entropy returns Zeroizing<Vec<u8>>",
    source_file: "src/bip85.rs",
    evidence: &["-> Result<Zeroizing<Vec<u8>>"],
},
ZeroizeRow {
    label: "bip85 entropy locals scrub via derive_entropy's Zeroizing return",
    source_file: "src/bip85.rs",
    evidence: &["let mut out = Zeroizing::new(vec![0u8; 64])"],
},
```

Both anchor substrings are present verbatim in `bip85.rs`:
- Anchor 1 (`-> Result<Zeroizing<Vec<u8>>`): present at the signature line.
- Anchor 2 (`let mut out = Zeroizing::new(vec![0u8; 64])`): present in the construction body.

Both labels were updated to read `Vec<u8>` instead of `[u8; 64]`. Lint passes substring-check. Confidence: 100.

### Item 5 — No test loss

`#[test]` count in `bip85.rs:tests`: **10**. Predicted: 8 pre-existing + 2 new = 10. All 8 original tests present (vectors, dice variants, refusal); 2 new tests added by RED commit: `derive_entropy_returns_zeroizing_vec_of_64_bytes` and `derive_entropy_is_byte_deterministic`. Confidence: 100.

### Item 6 — No public-API change

`bip85` is `mod bip85;` at `main.rs:3` (private module, binary crate, no `lib.rs`). All 8 functions in `bip85.rs` (1 `derive_entropy` + 7 `format_*`) carry `pub(crate)` visibility. The `hardened` helper is private. Grep for `^pub fn|^pub mod|^pub struct|^pub enum|^pub use` in `bip85.rs` returns zero hits. Confidence: 100.

### Item 7 — G7 wire-format SHA pins

The necessary preconditions hold:

- The fixture corpora at `crates/mnemonic-toolkit/tests/vectors/v0_1/*.txt` and `crates/mnemonic-toolkit/tests/vectors/v0_2/*.txt` are present in the expected counts.
- The diff scope (only `src/bip85.rs` + `tests/lint_zeroize_discipline.rs` per R0 forward prediction) contains no fixture file modifications.
- The SHA pins are documented unchanged in CHANGELOG.md, SPEC §6 G7, and v0.3 SPEC §Q7 — none of these files were edited in commits `3be9b77` or `c3509af`.
- Bytes flowing through CLI stdout derive from `format_*` String outputs, which are transparent to the internal type swap per §1 + Item 3.

The implementer reported both pins matched (`81828299...` for v0.1, `a381761656...` for v0.2). Consistent with the diff scope. Confidence the pins hold: 95 (the byte-determinism unit tests at Item 2 test the same property at the entropy-buffer layer).

### Item 8 — Diff size sanity

R0 §7 forward prediction:
- ~10 lines inside `bip85.rs` (no callee body edits)
- ~3 lines inside `lint_zeroize_discipline.rs`

Inspecting the GREEN commit:
- `bip85.rs` source change: signature line (`[u8; 64]` → `Vec<u8>`) + body (`Zeroizing::new([0u8; 64])` → `Zeroizing::new(vec![0u8; 64])`) + new `debug_assert_eq!` + doc-comment additions referencing Cycle B (~4 lines). Plus 2 new tests (~15 lines including docs).
- `lint_zeroize_discipline.rs`: 2 anchor substring updates + 2 label updates (~4 lines).

Materially within R0's forward prediction. Confidence: 90.

### Item 9 — Clippy + cargo test green

The implementer reported both clean. Internal consistency checks support that report:
- No `unsafe` introduced.
- No unused imports added (`zeroize::Zeroizing` was already in scope).
- The `Zeroizing::new(vec![0u8; 64])` form is idiomatic and clippy-clean.
- Test bodies use idiomatic Rust.
- `Zeroizing<Vec<u8>>` Deref/AsRef behavior matches the pre-swap `[u8; 64]` form for every callee per Item 3.

Confidence both pass: 88.

### Item 10 — Cycle A discipline preserved

- `tests/lint_argv_secret_flags.rs` — present, unchanged in Phase 1 commits. Discipline anchor table refers to flag-routes only, unaffected by entropy buffer shape.
- `tests/lint_safety_third_party_blocked.rs` — present, unchanged in Phase 1 commits. The `SAFETY: third-party-blocked` doc-comments in `bip85.rs` (at `Xpriv::derive_priv`, `bip39::Mnemonic`, and `SecretKey::from_slice` sites) are unchanged.
- `tests/lint_zeroize_discipline.rs` — updated in lockstep per Item 4; still has full coverage of the toolkit OWNED-secret sites.

Confidence: 100.

### Item 11 — `debug_assert_eq!` invariant guard

Present at `bip85.rs:54`:
```rust
debug_assert_eq!(out.len(), 64, "BIP-85 entropy is 64-byte invariant");
```

Placement is immediately before `Ok(out)`. Message matches the plan's T3 step 3 wording verbatim. Confidence: 100.

### Item 12 — No surprise additions

- **Cargo.toml**: unchanged at `crates/mnemonic-toolkit/Cargo.toml`. No dep additions, no version bump. Version still `0.9.2`.
- **FOLLOWUPS.md**: not in the commit diff. Phase 1 wrote nothing there.
- **Manual**: not in the commit diff (`docs/manual/` not modified). Phase 1 is internal-only — no CLI surface change → no mirror obligation per CLAUDE.md.
- **Unrelated refactors**: the only `bip85.rs` edits are the 6 source lines + 2 new tests at the bottom. No drive-by reformats.

Confidence: 100.

---

## §3. Test-suite verification

Direct inspection of `bip85.rs`'s test module:

| Test | Type | Asserts | Status (predicted) |
|---|---|---|---|
| `bip39_12_words_entropy_matches_spec` | spec-vector | byte-equality (16-byte prefix) | PASS |
| `hex_64_bytes_entropy_matches_spec` | spec-vector | byte-equality (full 64 bytes) | PASS |
| `pwd_base64_matches_spec` | spec-vector | byte-equality on String | PASS |
| `pwd_base85_matches_spec` | spec-vector | byte-equality on String | PASS |
| `dice_d6_10_rolls_matches_spec` | spec-vector | byte-equality on String | PASS |
| `dice_d2_rolls_in_range` | boundary | range-check | PASS |
| `dice_d256_rolls_in_range` | boundary | range-check | PASS |
| `dice_sides_too_small_refused` | refusal | error-shape | PASS |
| `derive_entropy_returns_zeroizing_vec_of_64_bytes` (new) | type-shape | `Zeroizing<Vec<u8>>` annotation + len()==64 | PASS |
| `derive_entropy_is_byte_deterministic` (new) | byte-determinism | derive twice, equal | PASS |

Total: 10 tests (8 pre-existing + 2 new), matching plan prediction. Implementer-reported running result: 10/10 PASS.

Integration tests (`cli_derive_child.rs`, `cli_argv_leakage.rs`, etc.) consume `derive-child` CLI surface; all assert byte-equality on stdout, transparent to internal type swap per R0 §4.3.

Lint tests (`lint_zeroize_discipline.rs`, `lint_argv_secret_flags.rs`, `lint_safety_third_party_blocked.rs`) — all evidence anchors verified present in their respective source files (Items 4 + 10).

---

## §4. Diff sanity (file-by-file)

| File | Lines changed (estimate) | R0 prediction | Status |
|---|---|---|---|
| `crates/mnemonic-toolkit/src/bip85.rs` | ~6 source + 4 doc-comment + ~15 new test | ~10 source (no callee edits) | Within prediction; new test LOC is expected RED |
| `crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs` | ~4 (2 anchor substrings + 2 labels) | ~3 lines | Within prediction (matches the +1 label-string fold from R0 §4.4) |

No other source/test files touched. Phase 1 stays narrowly scoped.

---

## §5. Cycle A discipline check

- **Argv-secret-flag discipline** (`lint_argv_secret_flags.rs`): file unchanged; `CANONICAL_FLAG_ROWS` table intact; no new CLI flag-routes were added by Phase 1.
- **Third-party SAFETY-comment discipline** (`lint_safety_third_party_blocked.rs`): file unchanged; the 4 `SAFETY: third-party-blocked` doc-comments in `bip85.rs` (at `Xpriv::derive_priv`, `bip39::Mnemonic`, and two `SecretKey::from_slice` sites) are preserved verbatim.
- **Zeroize-wrap discipline** (`lint_zeroize_discipline.rs`): updated in lockstep with the source change; both `bip85.rs` evidence anchors substring-match the post-swap source.

Cycle A's argv-leakage + Zeroize-on-drop hardening is fully preserved. Confidence: 100.

---

## §6. Verdict + exit-gate decision

**CLEAR — 0 Critical / 0 Important at confidence ≥ 80.**

Phase 1 ships (P1.T5). The implementation lands R0's locked design verbatim:
- Return type: `Result<Zeroizing<Vec<u8>>, ToolkitError>`.
- Construction: `Zeroizing::new(vec![0u8; 64])` (R0 §6.1 recommended Option A).
- Invariant guard: `debug_assert_eq!(out.len(), 64, "BIP-85 entropy is 64-byte invariant")` immediately before `Ok(out)`.
- All 7 callees transparent (no body edits, no workaround idioms).
- Lockstep `lint_zeroize_discipline.rs` evidence-anchor update.
- 2 new tests (type-shape + byte-determinism) added in RED commit, all 8 pre-existing tests intact.
- No surprise additions (Cargo.toml unchanged, FOLLOWUPS untouched, manual untouched).

Two Nit observations (below the ≥ 80 reporting threshold, recorded for future reference, not action items):

- **(Nit, conf 60)** R0 §6.4 flagged that `Zeroizing<T>` lacks a stable `into_inner()` — the plan's Phase 3a Site 4 sketch still uses `.into_inner()`. Not a Phase 1 concern; will surface at Phase 3a R0. Note for that future reviewer: `std::mem::take(&mut *z)` is the workable unwrap pattern for `Zeroizing<Vec<u8>>`.
- **(Nit, conf 55)** The doc-comment header at `bip85.rs:4-5` still reads "The 6 in-scope apps" — pre-existing inaccuracy (DICE is the 7th, added in v0.8) that R0 §2 I-R0-2 flagged but didn't make a P1.T3 requirement. Could be folded opportunistically in a future cleanup commit. Not Phase-1-blocking.

Both items are NOT reportable findings — recorded here only so the Phase 3a R0 reviewer and future cleanup PRs have the context.
