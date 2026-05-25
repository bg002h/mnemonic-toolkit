# indel-v2 Phase 3 Review — --max-subst CLI + candidate-list verify advisory + exit-4 + --json

**Round:** Phase 3 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `d54386e` (branch `indel-v2-cross-region-subst-fallback`). Files: `cmd/repair.rs`, `repair.rs`, `tests/cli_indel.rs`.
**Controller verification:** cli_indel 17/17; `indel_exit_code` unit pass; full default suite 128 ok-results (0 failed); clippy clean; no fallback/version yet.

## Verdict: GREEN (0 Critical / 0 Important)

### Checks out
- **Flag** (cmd/repair.rs:68-72): `--max-subst <E>` `range(0..=4)` default 0, "ms1/mk1/md1"; threaded `recover_indel_card(…, args.max_subst as usize)` (`:154`).
- **Exit invariant byte-identical at E=0:** `indel_exit_code(ambiguous_seen, substitution_seen, total_repairs)` (repair.rs:1086-1094) = `if amb||subst {4} else if tr==0 {0} else {5}`. At e_subst=0 the gate requires `off==0` ⇒ every accepted candidate has `subst_count==0` ⇒ `substitution_seen` never set ⇒ `{0,5,4-ambiguous,2}` identical to v0.37.2. `substitution_seen` set ONLY from `c.subst_count>=1` (Unique :159-161) / `v.iter().any(subst_count>=1)` (Ambiguous :173-175). Unit test 3-arg with the 6 cells incl. `(false,true,1)→4`, `(false,false,5)→5`.
- **Verify advisory** (cmd/repair.rs:210-214) on `substitution_seen` (stderr, after ms1 advisory); gated on ACTUAL subst use (subst_count), NOT the flag — pure-indel under `--max-subst 1` → subst_count=0 → no WARNING, exit 5 (Test C).
- **--json** (`:332,356,375`): `subst_count` per candidate + `confident = all(subst_count==0)`.
- **No-op notice** (`:120-122`): `max_subst>=1 && max_indel==0` → stderr notice; no exit/recovery effect.
- **Tests real:** A (drop+flip → exit 4 + WARNING), B (no-op notice), C (E=0 pure-indel exit 5 regression), D (clap reject E=5), E (--json confident=false+subst_count=1). `flip_data` = cyclic bech32 shift at `3+i`, a genuine substitution distinct from the drop.
- **No scope creep:** `Unrecoverable→IndelUnrecoverable` (no fallback); Cargo.toml 0.37.2; no manual/FOLLOWUPS changes; engine/oracles untouched.

### Minor (advisory, fold in Phase 5)
- `--max-indel` toolkit doc-comment (cmd/repair.rs:64) still says "ms1/mk1 only" (stale since v0.37.2 md1). The new `--max-subst` correctly says "ms1/mk1/md1". Fold the `--max-indel` line at Phase 5 for consistency (plan tracks the GUI twin as R0 M2).
- Test B asserts only the notice substring, not exit code — acceptable (notice is the load-bearing behavior).

Phase 3 clears the gate. Cleared to advance to Phase 4 (HrpMismatch fallback).
