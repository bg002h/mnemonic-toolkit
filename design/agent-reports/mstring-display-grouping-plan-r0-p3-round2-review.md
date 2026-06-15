# Plan-R0 (P3 mk) round 2 — architect review (verbatim) — GREEN

> Persisted verbatim per CLAUDE.md. Dispatched via Agent (feature-dev:code-architect).
> **Verdict: GREEN — 0 Critical / 0 Important.** Plan SHA at review: toolkit `05dbe1c`;
> mk repo `21786dc`. Verification round after the r1 fold; full `tests/` re-sweep.

## Verdict
GREEN — 0 Critical / 0 Important

## Round-1 fold verification
All three required fixes present + correct: (a) `tests/cli_slip132.rs` in Task 3's file list; (b) `"--group-size", "0"` instruction for `run_encode_decode`'s `mk encode` in Task 3 Step 3; (c) `cli_slip132.rs` lead entry in the suite-sweep. Cited ranges (`:58-84` fn span, `:60-75` encode block) accurate vs live source.

## Critical / Important
None.

## Minor
- m1 (carry-forward): spec §5 says `--separator bogus`→exit 2; mk-cli routes clap errors→64 (`main.rs:68-72`). Plan/test correctly use 64. No action.
- m2 (observation): `cli_repair.rs` chunk-equality asserts compare `mk_codec::encode`-built unbroken strings vs `mk repair` output, which is ALWAYS unbroken (`repair.rs:173-176` `emit_text` does `println!("{chunk}")` on codec-reconstructed chunks; never calls `render_grouped`). Safe without modification.

## Fresh completeness sweep — full result
Every `crates/mk-cli/tests/` file verified:
- `version_help_exit_codes.rs` — no mk1 strings / no `mk encode`. Safe.
- `gui_schema.rs` — subset `find()`/`iter()` JSON assertions; new flags appear as `"text"` kind. Safe.
- `cli_output_class.rs` — `encode_emits_watch_only_advisory` asserts only stderr; `mk1_fixture()` via `mk_codec::encode`. Safe.
- `cli_repair.rs` — fixtures via `mk_codec::encode`; `mk repair` output codec-reconstructed (never grouped). Safe.
- `cli_address.rs` — fixtures via `mk_codec::encode`; no `mk encode` CLI. Safe.
- `cli_derive.rs` — `child_xpub_roundtrips_through_encode` asserts only `contains("mk1")` (grouped still satisfies); rest via `mk_codec::encode`. Safe.
- `round_trip.rs` — `from_md1_derivation` fixed by fold; others via `mk_codec::encode`/`mk vectors`. Safe.
- `cli_slip132.rs` — `run_encode_decode` fixed by fold; `make_card()` safe under INTAKE-first (grouped → `mk verify` CLI strips by Task 4); 5 other `mk encode` calls assert only exit/stderr. Safe.

No additional breaking tests found. Enumeration complete; plan ready for execution.
