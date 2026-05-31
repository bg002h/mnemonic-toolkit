# Plan-doc R1 review (after R0 fold) — output-type-stderr-advisory Phase 1
**Date:** 2026-05-31 · **Reviewer:** opus architect · **Verdict: RED (0C/1I/5m).**

## R0 folds confirmed compile-correct
C1 (repair.rs:1331-1334 guard removal; outcome.kind:CardKind Copy :409) ✓; C2 (repair :215-216/inspect :155-156 track only any_ms1; loop var &CardKind at repair.rs:140/inspect.rs:110 — collect Vec<OutputClass>+worst; repair has 3 stdout arms Ok/Unique/Ambiguous — cover all) ✓; I1 (seedqr args.json_out exists; stderr at run_decode:137/run_encode:219; emit_*_output is tail) ✓; I2 (run-level :1255-1272; parsed:Vec<ParsedImport>, cosigners:Vec<ResolvedSlot>, entropy:Option<Vec<u8>>; predicate verbatim :1456/:2106) ✓; I3 (outputs:Vec<(NodeType,String)>; filter_map(|(n,_)|) ✓; predicates by-value :117/121) ✓; I4 (derive run has stderr :121; if/else ends :253, Ok(0):254) ✓; SemVer/naming/byte-literals consistent.

## Important
**I-new — P5 negative-assertion discovery grep (`secret material on stdout`) misses 3 of the 5 TTY-gate negative cells → P1 commit RED.** They assert DIFFERENT literals in P1-command suites: `cli_final_word_advisories.rs:72-75` `!contains("candidate list is secret material")`; `cli_seed_xor_advisories.rs:96-98` `!contains("Seed XOR shares on stdout")`; `:133-135` `!contains("combined phrase is secret material")`; (+ the named `cli_slip39_advisories.rs:310-313`). *Fix:* enumerate ALL FIVE cells by name in P1 Step 4 + P5 Step 1; broaden discovery beyond the literal (test-name pattern `piped.*does_not_emit|non.?tty`); invert each to assert the NEW unified P-line is PRESENT on piped stdout; fold in P1 (their commands are P1), not P5.

## Minor
M1 — I5 names only `slip39.rs:54`; also drop `seed_xor.rs:24` + `final_word.rs:20` `use std::io::{IsTerminal,…}` (compare_cost.rs:6 stays — gate not dropped). M2 — P3 auto-repair cell is ms1-only; add mk1→W + md1→T cells (existing `cli_auto_repair.rs:104` md1 cell asserts stdout-only). M3 — ms encode/decode have two stdout branches (`--json` vs text); P emit after BOTH. M4 — ms derive `stderr` is a local `let mut stderr=std::io::stderr()` (`derive.rs:121`), not a threaded param (wording). M5 — electrum-decrypt P1 row labels json-suppression "(I5)"; it's file→inert (cosmetic).

## Controller folds: I-new (5 cells named + grep broadened + P1-folded); M1/M2/M3 useful, M4/M5 cosmetic.
