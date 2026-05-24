# End-of-Cycle Review — repair --max-indel (v0.37.1)

**Round:** end-of-cycle (final 0C/0I gate before tag). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Scope:** full branch diff `origin/master...HEAD` (`m-format-incorrect-length-recovery`); Phase-6 commit `e5edc9b`.
**Controller verification:** version 0.37.1 consistent (Cargo.toml/Cargo.lock/both READMEs/install.sh); full default `cargo test -p mnemonic-toolkit` green (128 ok-results, 0 failed); clippy `-D warnings` clean; `make -C docs/manual lint` GREEN (after the Minor fixes below).

## Verdict: GREEN (0 Critical / 0 Important) — ship-ready

### Phase-6 verification (all pass)
1. Version = 0.37.1 across all 5 guarded surfaces. 2. CHANGELOG [0.37.1] accurate; SemVer PATCH correct (additive default-off). 3. Manual: `--max-indel` row + prose subsection + `--json` envelope; `.cspell.json` additions legit domain terms. 4. FOLLOWUPS: parent flipped `open→resolved` + Resolution note; 4 new OPEN entries well-formed; erasure entry carries sibling `Companion:` lines. 5. Advisory test real (asserts exact stderr string + code(5) + recovered on stdout).

### Final integration sweep
- Both recovery paths trace cleanly: data-part (too-short/long → trigger → oracle → Unique → exit 5) and prefix (`resolve_groups` relaxed → HrpMismatch → trigger → collect_prefix → exit 5). Ambiguous → exit 4; Unrecoverable → exit 2 via a non-re-triggering error (IndelUnrecoverable EXCLUDED from is_indel_trigger → no loop).
- No loose ends: no TODO/dead code; sole `#[allow(clippy::needless_range_loop)]` justified. Enum ordering compliant.
- Scope clean: `cmd/inspect.rs` only the 1-line `resolve_groups(...,false)` caller; no sibling-repo/GUI files; no tag in branch.
- Lockstep ledger: toolkit `gui-schema` is clap-derived (auto-emits `--max-indel`); the hand-maintained GUI mirror (`mnemonic-gui/src/schema/mnemonic.rs`) is correctly deferred to the post-tag GUI v0.21.2 paired PR (its `schema_mirror` test runs against the pinned binary → cannot precede the tag).

### Minor (3 — doc-citation only; FOLDED post-review)
1. Manual `--json` example used `"region":"data"`/`"direction":"delete"`; actual emits `"data-part"`/`"deleted"`/`"inserted"` (`cmd/repair.rs::region_str`/`direction_str`). **Fixed** (example + prose). Prevents a JSON-consumer wire-shape misread.
2. Manual repair exit-codes table omitted `4` (ambiguous). **Fixed** (added the row; prose already documented it).
3. FOLLOWUP (d) cited non-existent `recover_deleted`/`recover_inserted`; actual `collect_data_delete`/`collect_data_insert`/`collect_prefix` + the oracles. **Fixed.**
(Manual lint re-run GREEN after fixes.)

### Ship-readiness
Branch ready to tag `mnemonic-toolkit-v0.37.1`. Feature integrates end-to-end across all 6 phases; version stamps consistent; docs/FOLLOWUPS reflect behavior; scope contained to the toolkit.

**Remaining post-tag actions (confirm with human):**
1. Tag `mnemonic-toolkit-v0.37.1` (merge to master → ff → tag → push) — outward-facing, confirm first.
2. Paired GUI PR: `mnemonic-gui v0.21.2` — add `max-indel` to `REPAIR_FLAGS` (`FlagKind::Number{min:0,max:NumberMax::Static(4)}`) + bump toolkit pin to v0.37.1, keeping the lagging `schema_mirror` gate green.
