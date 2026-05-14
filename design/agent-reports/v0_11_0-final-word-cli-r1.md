# v0.11.0 Phase 2 — CLI surface R1 reviewer report

**Phase:** P2 — CLI surface (`mnemonic final-word` subcommand)
**Round:** R1 round 1
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commit under review:** `48b4488` (P2 GREEN) on top of `757eb93` (P2 RED)
**Predecessor:** `12e2b5b` — Cycle B mlock infrastructure complete

## Verdict

**0 Critical / 0 Important / 0 Nice-to-have — R1 LOCK round 1.**

Phase 2 ships.

## Scope reviewed

All 12 mandatory reviewer checks (Cycle A discipline, Cycle B discipline,
clap-derive correctness, non-Phrase refusal, refusal-class test coverage,
JSON envelope shape + SHA pins, advisory implementation, test mechanics
including `NamedTempFile` lifetime bug fix, stdin route equivalence,
lint mirror invariants, GUI schema mirror, manual mirror deferral).

Files reviewed:
- `crates/mnemonic-toolkit/src/cmd/final_word.rs` (new, ~200 LOC)
- `crates/mnemonic-toolkit/src/cmd/mod.rs` (1-line addition)
- `crates/mnemonic-toolkit/src/main.rs` (Command::FinalWord variant + dispatch)
- 5× `crates/mnemonic-toolkit/tests/cli_final_word_*.rs` (37 tests)
- `crates/mnemonic-toolkit/tests/cli_gui_schema.rs` (5→6 subcommand bump)
- `crates/mnemonic-toolkit/tests/lint_argv_secret_flags.rs` (+1 row, 20→21)
- `crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs` (+1 row)

## Key validations

1. **Cycle A discipline (correct).** `secret_in_argv_warning` fires only
   for inline `--from phrase=<value>`, never for `--from phrase=-`.
   `Zeroizing<String>` wraps the parsed partial before any downstream
   handling. Wrap site precedes the mlock pin.

2. **Cycle B discipline (correct).** `mnemonic_toolkit::mlock::pin_pages_for(
   partial.as_bytes())` is bound to a `_pin_partial` function-scope guard,
   pins the underlying String heap (not a stack temporary). Mirrors
   `derive_child.rs:128` exactly.

3. **clap-derive (correct).** `value_parser = parse_from_input`,
   `required = true`, `--language` default = "english",
   `--json-out` is `Option<PathBuf>`.

4. **JSON envelope (correct).** Field set + order matches SPEC §2.3
   byte-for-byte. `schema_version: "1"`, no `feature` field. SHA pins
   captured at GREEN:
   - abandon×11: `b74e0b4a6531c926d6f215e5037cf6b322d925fe2bd9ff7f05f626ceca146f02`
   - beef×11: `273ae8ac972ef0f9baefeab4f47668b4a678e80b0f7d241a5e85f1efd295f3fa`

5. **Test mechanics fix (correct).** Both `json_out_world_readable_emits_advisory`
   and `json_out_0o600_does_not_emit_advisory` correctly keep the
   `NamedTempFile` alive through the CLI invocation (the earlier P2 RED
   iteration had a `drop(f)` bug that deleted the path before the CLI
   re-created under umask 022 → mode 644 → false advisory). The
   `std::fs::write` call opens the existing path with O_TRUNC, preserving
   the pre-set mode.

6. **stdin route equivalence (G2 satisfied).** `--from phrase=-` and
   `--from phrase=<value>` produce byte-identical stdout for the same
   partial, exercised by `stdin_route_equals_inline_route_byte_for_byte`.
   `read_stdin_to_string`'s `trim()` correctly handles trailing newlines
   and internal whitespace.

7. **Lint anchors (correct).** Both `phrase=-` and `secret_in_argv_warning`
   appear in `src/cmd/final_word.rs` (line 65) — robust against
   doc-comment churn. `zeroize::Zeroizing::new` appears at the
   partial-wrap site.

8. **GUI schema mirror (correct).** `gui_schema_lists_all_six_subcommands`
   correctly lists `final-word` alphabetically. `cmd::gui_schema::run`
   picks up the new subcommand via `CommandFactory` without code changes.

## Filed observation (non-blocking)

SPEC §2.4/§2.5 narrative cites exit code `64` for refusal classes, but
the implementation routes via `ToolkitError::BadInput::exit_code() == 1`
(toolkit-consistent precedent at `src/error.rs:244`). Tests use
`assert_ne!(exit, 0)` and are tolerant of either value. The implementation
is correct (no behavior change should be made); the SPEC narrative should
be corrected at PE for documentation hygiene. **Not blocking for P2.**

## Manual mirror

Per plan, the manual chapter (`docs/manual/src/40-cli-reference/`) ships
in P3. `docs/manual/tests/lint.sh` flag-coverage check expected to RED
until then. Confirmed as the planned sequencing.

---

## R1 LOCK

Phase 2 R1 LOCK round 1. Ship.
