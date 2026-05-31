# Plan-doc R3 confirmation (after R2 fold) — output-type-stderr-advisory Phase 1
**Date:** 2026-05-31 · **Reviewer:** opus architect · **Verdict: RED (0C/1I) → folded.**

## Confirmed
- The five TTY-gate negative cells are EXACTLY five (no sixth): cli_final_word_advisories.rs:72, cli_seed_xor_advisories.rs:96/133, cli_slip39_advisories.rs:265(split)/310(combine). Other `must NOT` hits (argv-leak, json-out 0o600, toolkit-only, G9-iteration) survive the gate-drop by literal mismatch. `grep silent_when_piped` isolates exactly the five. R1/R2 thread CLOSED.
- Net-new commands: no breaking hidden negative cell. bundle watch-only negatives (cli_bundle_watch_only.rs:52, cli_bundle_multisig.rs:68 `!contains("warning: secret material on stdout")`) stay GREEN after P2 (new W-line ≠ old literal) + caught by P5 grep. Legacy-helper removal safe at P3.

## Important (FOLDED — durably)
**I1 — `cli_indel.rs:223,232` positively asserts the OLD D9 literal on the `repair --max-indel` path (re-routed by P2 `cmd/repair.rs:216` or P3 `emit_repair_report:1333`), not enumerated in the breaking phase's re-pin list → goes RED at P2/P3 commit, only found by P5's late grep.** Same defect-class as R1-I-new/R2-I1, on the repair path. *Fix (folded):* (a) DURABLE — added a mandatory **PHASE RE-PIN DISCIPLINE** callout: every wiring phase greps all of tests/ for the OLD strings it changed + re-pins every match + runs full `cargo test` at each phase commit (not just touched suites); (b) SPECIFIC — added cli_indel.rs:223-225,232 re-pin to P3 Step 4 (P2 if cmd/repair.rs:216 fires on the indel path). Architect confirmed no OTHER repair/inspect/convert test asserts the old literal → gap bounded.

## Controller: catch-all + cli_indel folded; re-dispatch R4 (focused confirmation).
