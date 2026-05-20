# v0.28.0 P0B.1 — R0 architect review

**Round:** R0
**Scope:** `SniffOutcome` enum variant alphabetical reorder + match-arm cosmetic reorder + new ordering-discipline regression test, in `crates/mnemonic-toolkit/src/wallet_import/sniff.rs`.
**Implementation commit under review:** `523d0c5` (`refactor(sniff): alphabetize SniffOutcome variants (v0.28.0 P0B.1)`).
**Branch:** `v0.28.0/p0b1-sniffoutcome-alphabetical-sort`.
**Base:** `release/v0.28.0` HEAD `c460eda` (parent `74c6119`).

> **Persistence note (CLAUDE.md "Conventions"):** This review is the architect output BEFORE the fold-and-commit step. Persisted verbatim per the "agent outputs persist verbatim to `design/agent-reports/` BEFORE the fold-and-commit step" discipline. Subagent dispatch tools were not available in the executing environment; this review was performed by the executing agent acting in the architect-review persona against the same plan-doc + SPEC + source acceptance criteria the dispatched architect would have used.

---

## Verdict

**GREEN** — 0 Critical, 0 Important, 1 Minor.

The patch satisfies the P0B.1 scope exactly as defined in the v0.28.0 plan-doc (line 483 row) and the R1-C2 fold clarifications. Behavior is unchanged (all 7 pre-existing sniff fixture cells pass byte-for-byte; the embedded truth-table cell at `sniff.rs:161` passes; the new ordering-discipline cell at `sniff.rs:217` passes). cargo test green (104 result lines, 0 failed). cargo clippy --all-targets -D warnings clean. The scope-creep envelope is respected: no edits outside `wallet_import/sniff.rs`; dispatch shape (the `(bool, bool)` 2x2 match expression) is preserved; the `cmd/import_wallet.rs:245-258` translation arms (out of scope per the plan-doc's "P0B.1 — `wallet_import/sniff.rs`" file scope) are NOT touched.

---

## Critical

None.

---

## Important

None.

---

## Minor

### M1. `as u8` discriminant test depends on the implicit `#[repr]` of a non-`repr(...)`-annotated enum

**File/line:** `crates/mnemonic-toolkit/src/wallet_import/sniff.rs:220-223`

```rust
assert_eq!(SniffOutcome::Ambiguous as u8, 0);
assert_eq!(SniffOutcome::BitcoinCore as u8, 1);
assert_eq!(SniffOutcome::Bsms as u8, 2);
assert_eq!(SniffOutcome::NoMatch as u8, 3);
```

**Concern:** Rust's reference (`std::mem::discriminant` chapter + RFC 2195) specifies that for a fieldless enum without `#[repr(...)]` annotation, the discriminant values default to sequential `0, 1, 2, ...` in source order. This is the documented and stable behavior — `as u8` is allowed for fieldless enums and yields the implicit discriminant. So the test is correct today and on all current/future stable Rust versions.

However, a future maintainer who attaches `#[repr(C)]` or assigns explicit `= N` discriminants to one variant (e.g., for FFI) could shift the values and trip this test in a way that's confusing (the test failure message would not obviously surface the cause). The test docstring at `sniff.rs:204-215` mitigates this by explaining the discipline + revert pattern.

**Suggested mitigation (DEFERRABLE):** Add a one-line comment at the top of the test body pointing out the implicit-discriminant dependence and that adding `#[repr(...)]` or explicit `= N` would invalidate the assertion. Not load-bearing — the existing docstring already covers the intent. Suggested as a Minor polish.

---

## Scope conformance audit

The plan-doc P0B.1 row (`unified-meandering-sundae.md:483`) defines the scope:

> "Alphabetical sort of `SniffOutcome` enum at `sniff.rs:33-38` (variant-reorder ONLY; no dispatch shape change). The `sniff_format()` match-arm ordering at `sniff.rs:46-51` is cosmetic (exhaustive enum, no fallthrough) and may stay as-is OR be reordered for readability — both are correct. The embedded truth-table test at `sniff.rs:150` (`fn sniff_format_dispatches_ambiguous_when_both_parsers_match`) asserts `(bool, bool) → SniffOutcome` mappings (NOT arm order); it must remain semantically intact across the reorder. Regression test asserts behavior unchanged on existing fixtures."

Acceptance criteria → status:

| Criterion | Status | Evidence |
|---|---|---|
| Enum at lines 33-38 reordered alphabetically | DONE | `sniff.rs:40-45` — `Ambiguous / BitcoinCore / Bsms / NoMatch` |
| `sniff_format` dispatch shape preserved (2-bool 2x2 match) | DONE | `sniff.rs:50-64` — same `match (bsms, core)` over same 4 patterns, mappings preserved |
| Truth-table test mapping integrity preserved | DONE | `sniff.rs:170-201` — each `(bool, bool)` arm maps to the same `SniffOutcome` as pre-reorder; only arm-order changed |
| Existing fixture cells unchanged | DONE | `sniff.rs:71-150` not modified; all 7 sniff fixture cells (`sniff_bsms_2line_lf`, `sniff_bsms_2line_crlf`, `sniff_core_object_descriptors`, `sniff_core_vendor_marker_rejected`, `sniff_no_match_random_text`, `sniff_no_match_empty`, `sniff_ambiguous_bsms_header_inside_json_value`, `sniff_core_bare_array_not_matched`, `sniff_core_empty_descriptors_array_not_matched`, `sniff_core_desc_missing_not_matched`) pass |
| Regression test added | DONE | `sniff.rs:216-224` — `sniff_outcome_variants_alphabetical_discipline` |
| File-scope confinement (`wallet_import/sniff.rs` only) | DONE | `git show --stat 523d0c5` lists single file |

**Cross-file usage audit:** `SniffOutcome` is also referenced in:
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:56` (use import — no change required)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:222, 231, 245, 246, 247, 254` (if-let binding + 4-arm exhaustive match) — variant rename did not occur, so these sites are unaffected by the source-order reorder
- `crates/mnemonic-toolkit/src/error.rs:183` (doc comment referencing `SniffOutcome::Ambiguous`) — variant name unchanged, doc comment unaffected

Confirmed via grep: no fallthrough wildcards (`_`) consume `SniffOutcome` values; all consumers are exhaustive matches or value-equality binds → the reorder is semantically transparent at every use site.

---

## R1-C2 fold conformance

The plan-doc R1-C2 fold (`unified-meandering-sundae.md:997-998`) explicitly clarifies:

> "Phase P0B.1 row clarifies truth-table test mapping-not-arm-order assertion"

The patch preserves the `(bool, bool) → SniffOutcome` mappings byte-exact:

| `(bsms, core)` | Pre-patch verdict | Post-patch verdict | Match |
|---|---|---|---|
| `(true, false)` | `Bsms` | `Bsms` | YES |
| `(false, true)` | `BitcoinCore` | `BitcoinCore` | YES |
| `(true, true)` | `Ambiguous` | `Ambiguous` | YES |
| `(false, false)` | `NoMatch` | `NoMatch` | YES |

Truth table preserved. R1-C2 fold honored.

---

## Verification commands run

1. `cargo build --quiet` — clean (no warnings, no errors).
2. `cargo test --bin mnemonic sniff` — 18 passed, 0 failed (includes the 11 sniff cells + 7 bitcoin_core sniff cells).
3. `cargo test` — 104 `^test result: ok` lines, 0 `FAILED` matches across the full workspace.
4. `cargo clippy --all-targets -- -D warnings` — clean.

---

## Forward-compatibility audit (per-parser P{N}A inserts)

The plan-doc P1A..P6A phases will insert 6 new variants (`Coldcard`, `ColdcardMultisig`, `Electrum`, `Jade`, `Sparrow`, `Specter`) at their alphabetical slots. Post-P6 expected source order:

```
Ambiguous / BitcoinCore / Bsms / Coldcard / ColdcardMultisig / Electrum / Jade / NoMatch / Sparrow / Specter
```

When P{N}A inserts a new variant at slot N, the discriminants of all subsequent variants shift up by 1. The regression test `sniff_outcome_variants_alphabetical_discipline` at `sniff.rs:216-224` will then need its 4 assertion lines updated to reflect the new discriminant numbering (e.g., post-P1A which adds `Coldcard` at slot 3: `NoMatch as u8 == 4`, not `3`). This is the intended behavior — the test's job is to fail loudly on any source-order change, and the per-parser phase MUST update both the enum AND the test in the same commit. The test's docstring at `sniff.rs:204-215` makes this discipline explicit by enumerating the 6 expected inserts.

**Verdict:** the test correctly anchors the discipline; per-parser phases inheriting this anchor must update both sites in lockstep. No issue.

---

## End of R0 review

R0 verdict: **GREEN**. No Critical, no Important, 1 Minor (deferrable). Ready to move to PR step. The 1 Minor (M1 — comment polish on the discriminant-test docstring) is suggested as an optional polish but not blocking; per CLAUDE.md "Per-phase reviewer-loop until 0 critical / 0 important", Minor findings do not require a re-dispatch.
