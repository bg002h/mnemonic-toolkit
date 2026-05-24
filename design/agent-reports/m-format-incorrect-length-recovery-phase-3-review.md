# Phase 3 Review — prefix-region producer + ambiguity contract (ms1 complete)

**Round:** Phase 3 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `31fadd4` (branch `m-format-incorrect-length-recovery`). Files: `src/indel.rs`, `src/repair.rs`.
**Controller verification:** `cargo test -p mnemonic-toolkit --bins` → 806 passed / 2 ignored; 4 Phase-3 tests pass; clippy `-D warnings` clean.

## Verdict: GREEN (0 Critical / 0 Important) — no findings

### Checks out
- **`collect_prefix`** (indel.rs:98-130) matches §2.1/§3: `k="{hrp}1"`; `lo=3.saturating_sub(j)`, `hi=(3+j).min(chars.len())`; `p in lo..=hi`; accept only `levenshtein(head,k)==j` (R0 m2 exact-distance); `cand="{k}{tail}"`; EMPTY allowed; `direction=Inserted if p<3 else Deleted`; `region=Prefix`.
- **Slicing safe:** char-indexed (`chars: Vec<char>`), `p ≤ hi ≤ chars.len()` in-bounds; `lo>hi` ⇒ empty range, no panic.
- **Prefix tests traced:** (a) drop 'm' → "s10entrs…": data producers inert (`data_part_bounds` None, not "ms1"-prefixed) → only prefix fires, p=2→VALID_MS1 Inserted, Unique. (b) "msx10entrs…": p=4 head="msx1" lev=1→VALID_MS1 Deleted, Unique; no other p yields a distinct valid recovered.
- **Load-bearing dedup test** (indel.rs:303-320): identical `recovered`, mismatched region/direction → collapses to 1 under `dedup_by_recovered` (keyed on `recovered` + preceding `sort_by`); would FAIL under derived `PartialEq`. Genuinely guards R0 I2.
- **Ambiguity test** (indel.rs:326-338): `AcceptAll` mock on "ms1qpzr" → ≥2 distinct recovered → the `_ => Ambiguous(hits)` arm fires.
- **Completeness/scope:** all three producers have real bodies (no empty stubs left); `#[allow]` audit clean (collect_prefix/levenshtein allows removed, now called; remaining dead_code allows on Phase-5-reachable items, kept live via rustc allow-root propagation; struct fields read in `#[cfg(test)]` which `--all-targets` builds → no field-never-read). No mk1/cmd/main/flag changes; `recover_indel_card` mk1 arm still the Phase-1 `Unrecoverable` stub.

Phase 3 clears the gate. ms1 recovery COMPLETE. Cleared to advance to Phase 4 (mk1).
