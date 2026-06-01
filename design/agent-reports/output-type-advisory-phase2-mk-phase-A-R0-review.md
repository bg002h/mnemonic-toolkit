# Phase A (mk-cli) per-phase R0 review — output-type advisory Phase 2

> Persisted verbatim from the opus architect per-phase review. Reviewed `git diff e5620ce..1748bd8` (A1 `182c5df`, A2 `065eca5`, A3 `1748bd8`) on branch `output-class-advisory-phase2`. GREEN — cleared to tag (tag deferred to release-authorization gate).

## Verdict: GREEN (0C / 0I)

Phase A is correct, in-scope, and faithful to the SPEC and the shipped ms-cli precedent. All six output-producing subcommands emit `WatchOnly` on the success path through both `--json` and text modes; the three inert subcommands emit nothing, exactly matching the SPEC's by-rule classification (incl. footgun F1 for `verify --json` policy_id_stubs and the "by rule, not stdout-empty" classification of `vectors`). Byte parity with ms-cli / toolkit is confirmed at the byte level (U+2014). Full crate suite green (60 tests, 0 failures; cli_output_class = 10), fresh clippy `-D warnings` clean, scope is exactly the 11 expected files, SemVer PATCH is correct. No Critical or Important findings. Cleared to tag.

## Critical
None.

## Important
None.

## Minor
- **M1 — "byte-parity guard" is tautological (does not read sibling source).** `tests/cli_output_class.rs::byte_parity_advisory_lines` asserts each test-file constant equals an inline string literal — a constant-equals-itself self-check. It does NOT `include_str!`/read `ms-cli/src/advisory.rs` or `secret_advisory.rs`, so it cannot catch a future cross-repo divergence. The module doc + test docs slightly overstate this as "cross-repo byte parity is enforced by …". **Why not Important:** (a) the shipped, R0-GREEN ms-cli precedent uses the identical tautological pattern and Phase A was chartered as a byte-for-byte copy of it — re-designing the test here is out of scope; (b) the WATCH_ONLY line (the only class mk emits) IS anchored to real emitted output by the six runtime positive cells via `.contains(WATCH_ONLY_LINE)`. Fix (optional, constellation-wide, defer to a FOLLOWUP): have one repo `include_str!` the others' module or pin the literals in a shared fixture.
- **M2 — positive cells exercise text mode only.** None of the six positive cells pass `--json`. **Why not Important:** confirmed by source read that no mk `run()` body has a `--json` early-return — the emit sits after the unified `if args.json {…} else {…}` in all six handlers, so both modes structurally reach it (SPEC I1's early-return hazard is an md concern, absent in mk). Cheap hardening: add a `--json` variant to one positive cell.

## Coverage table
| Subcommand | stdout | Class | Emit site | Correct? |
|---|---|---|---|---|
| decode | xpub + origin | WatchOnly | decode.rs:38-41 (after if/else, before Ok(0)) | ✓ |
| encode | mk1 string(s) | WatchOnly | encode.rs:97-100 | ✓ |
| inspect | decoded card | WatchOnly | inspect.rs:40-43 | ✓ |
| repair | corrected mk1 | WatchOnly | repair.rs:86-89 (Ok(0) AND Ok(5); error via `?` at :67) | ✓ |
| derive | child xpub | WatchOnly | derive.rs:99-102 | ✓ |
| address | addresses | WatchOnly | address.rs:126-129 (additive to depth warning :84) | ✓ |
| verify | OK / {ok,…,stubs} | inert | — | ✓ (stubs = 4-byte hashes, F1) |
| vectors | corpus JSON | inert | — | ✓ (machine corpus, by rule) |
| gui-schema | CLI-surface JSON | inert | — | ✓ (infrastructure) |
Advisory call-site grep: exactly the 6 handlers, nowhere else.

## Verification
- **Byte-parity ✓** — the 3 literals in `output_advisory.rs:30-32` identical to `ms-cli/advisory.rs:40-42` and `toolkit/secret_advisory.rs:100-102`; em-dash = U+2014 (bytes e2 80 94) whether written `\u{2014}` (mk/ms) or `—` (toolkit). Enum `#[allow(dead_code)]`, no Ord; `worst_class_on_stdout`/`card_kind_class` NOT ported.
- **Tests ✓** — cli_output_class = 10 (6 positive + 3 inert + 1 byte-parity); whole crate 60 passed / 0 failed. repair asserts exit `Some(5)`; inert cells assert absence of all 3 lines.
- **Clippy ✓** — `--all-targets -D warnings` exit 0 on fresh recompile; no orphaned imports.
- **Scope ✓** — 11 files: Cargo.lock, Cargo.toml (0.6.0→0.6.1), main.rs (one `mod` line), 6 handler emit lines, new module, new test. Nothing else.
- **SemVer ✓** — 0.6.0→0.6.1 PATCH, publish-ready.

## Notes
- No `--json` early-return in any of the six `run()` bodies; every early `return` is an `Err(...)` validation or a helper-fn return — none bypasses the advisory. This is why M2 is Minor not Important.
- `address` emits two stderr lines when the xpub is off depth-3 (pre-existing depth advisory :84 + the new WatchOnly line); the fixture is depth-3 so only the advisory appears; `.contains()` tolerates both.
- repair WatchOnly holds for exit 0 and exit 5; the unrepairable case returns via `?` (exit 2) before the emit → inert, as intended.
