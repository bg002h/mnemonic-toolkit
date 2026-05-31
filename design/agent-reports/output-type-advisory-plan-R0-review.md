# Plan-doc R0 review — output-type-stderr-advisory Phase 1
**Date:** 2026-05-31 · **Reviewer:** opus architect · **SHA:** files match `18cfdce` line numbers · **Verdict: RED (2C/6I/4m).**
Architecture sound (P0 helper compiles); defects in per-command emit-site citations (stdout-only helpers vs run-level) + auto-repair/repair/inspect guard+kind aggregation.

## Critical
**C1 — P3 auto-repair re-route keeps the `if matches!(outcome.kind, CardKind::Ms1)` guard → mk1/md1 stay SILENT.** Live `repair.rs:1331-1334`: `if matches!(outcome.kind, CardKind::Ms1) { secret_on_stdout_warning(outcome.kind, stderr); }`. Plan only swaps the inner call. *Fix:* REMOVE the `if matches!(…Ms1)` guard (`:1332`), emit unconditionally `emit_output_class_advisory(card_kind_class(outcome.kind), stderr)`. `RepairOutcome.kind: CardKind` (`repair.rs:408-409`).
**C2 — P2 repair/inspect: `kind` is UNBOUND at the emit sites; only `any_ms1` tracked; no mk1/md1 widening.** `cmd/repair.rs:215-216` + `inspect.rs:155-156`: `if any_ms1 { secret_on_stdout_warning(CardKind::Ms1, stderr) }`; per-chunk `kind` is loop-local (`repair.rs:144`/`inspect.rs:111`). `card_kind_class(kind)` won't compile. *Fix:* collect the CardKinds reaching stdout into `Vec<OutputClass>` in the chunk loop, then `if let Some(c) = worst_class_on_stdout(&kinds) { emit_output_class_advisory(c, stderr) }`.

## Important
**I1 — seedqr emit-site `:295/:323` is inside stdout-only `emit_decode/encode_output` (no stderr).** Emit at run-level `run_decode/run_encode` (`:133/:215`) after the `emit_*_output` call, gated `args.json_out.is_none()` (file→inert).
**I2 — import-wallet `:2111` is inside `emit_summary` (stdout-only); `entropy=` is a flag not key material.** Real site `~:1257-1270` (stdout+stderr+parsed in scope). Predicate `parsed.iter().flat_map(|p| &p.cosigners).any(|c| c.entropy.is_some())` (`:1456/:2106`). Emit `~:1270`.
**I3 — convert helper sig mismatch.** `outputs.iter()` yields `&(NodeType, String)`. *Fix:* `outputs.iter().filter_map(|(n,_)| convert_target_class(*n))`; `convert_target_class(NodeType)->Option<OutputClass>` {argv_secret→P, side_input_only→None, else W} (`convert.rs:117/121`). stderr in scope `:1099`.
**I4 — ms derive W-line "after language note" lands inside `else{ if defaulted }`** (`derive.rs:246-249`) → `--json`/non-defaulted emit nothing. *Fix:* emit at run-level after the whole `if args.json{}else{}` block (`~:253`), unconditional. ms derive HAS a threaded `stderr` param.
**I5 — TTY-gate drop orphans `use std::io::IsTerminal`** (only consumers at `slip39.rs:544,681` etc.) → `unused_imports` under clippy `-D warnings` (P6) + red P1 commit. *Fix:* remove the orphaned `IsTerminal` import (+ any `stdout()` binding) per file in the same edit.
**I6 — P5 re-pin misses NEGATIVE assertions broken by the TTY-gate drop.** `cli_slip39_advisories.rs:311` asserts `!stderr.contains("reconstructed secret material on stdout")`; after the gate drop slip39-combine emits unconditionally → must INVERT/DELETE, not "re-pin to new wording". *Fix:* P5 calls out negative/absence assertions on the 5 TTY-dropped commands as a distinct class; re-pin the P1-touched suites IN P1 (not deferred) to keep the P1 commit green.

## Minor
M1 — SHA label `18cfdce` vs HEAD; line numbers matched, cosmetic. M2 — `cli_secret_in_argv_warning.rs:13` has the literal in a `//!` doc-comment only (no assertion) → 11 asserting + 1 comment, not "12 asserting". M3 — P3 consolidation grep catches only the 5 literal-bearing sites; slip39-split (`:548` different literal)/addresses/seedqr rely on per-command cells (acceptable, don't imply exhaustive). M4 — derive_child.rs:308 is an inlined writeln literal (not a `_unconditional` call) — literal→helper swap.

## Controller folds: C1 remove Ms1 guard; C2 collect-kinds→worst; I1 seedqr run-level+json gate; I2 import ~:1270; I3 filter_map (n,_); I4 ms derive run-level unconditional; I5 drop IsTerminal imports; I6 negative-assertion class + re-pin in P1.
