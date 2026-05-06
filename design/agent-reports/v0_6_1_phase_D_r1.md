# v0.6.1 Phase D code review — r1 (reviewer: feature-dev:code-reviewer)

**Verdict:** APPROVED 0 Critical / 0 Important.

## Source-cross-checks the reviewer ran

- **Stderr ordering ("last write" invariant):** the warning block at `bundle.rs:646-655` is OUTSIDE the `if args.json { ... } else { ... }` branch (which closes at line 645). Engraving-card stderr write is inside the `else` branch only (text mode). Sequence:
  - Text mode: stdout body → engraving card → warning.
  - JSON mode: stdout JSON → warning.
  Both satisfy SPEC §5.5.a "warning is the last stderr write." Confirmed via the two callers of `emit_unified` (`bundle_run_unified` line 250 and `bundle_run_unified_descriptor` line 979) — neither emits any stderr after `emit_unified` returns.
- **Byte-exact text match with convert.rs §7:** identical literal `"warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')"` in both sites. Test in `cli_bundle_full.rs` asserts this exact literal.
- **Watch-only suppression:** `Bundle::any_secret_bearing()` returns `false` when all `ms1` entries are `""` sentinels (`synthesize.rs:34`). Both watch-only negative assertions (`cli_bundle_watch_only.rs` single-sig and `cli_bundle_multisig.rs`) cover the path.
- **Coverage of all emit paths:** both call sites of `emit_unified` (`bundle_run_unified` + `bundle_run_unified_descriptor`) route through the same function; one warning insertion covers all bundle dispatch paths. No other code path in `bundle.rs` emits ms1 to stdout.
- **WIF-only-bundle limitation:** WIF slots set `entropy: None` and synthesize into an empty-string ms1 sentinel; `any_secret_bearing()` returns `false`; warning is correctly suppressed. Matches SPEC §5.5.a documented limitation.

## Low fixed in this commit

- Reviewer noted no JSON-mode positive assertion. Added `bundle_full_json_mode_emits_secret_on_stdout_warning` to `cli_bundle_full.rs` to close the gap (warning fires in `--json` mode too). Trivial; closes Low immediately.

## Cleared for Phase D commit

`cargo test --workspace` reports 239 lib + integration tests pass; +2 new tests + 2 stderr negative assertions on existing watch-only tests.
