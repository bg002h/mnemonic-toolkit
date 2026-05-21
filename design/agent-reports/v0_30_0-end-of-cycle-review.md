# v0.30.0 end-of-cycle opus review (pre-tag)

**Reviewer:** opus
**Phase:** End-of-cycle (pre-tag-push gate)
**Date:** 2026-05-21
**Toolkit HEAD under review:** 56dd2b6

## Critical (C)
NONE.

## Important (I)
NONE.

## Minor (M)

### M1 — `cli_seedqr.rs` does not import `predicates::prelude::*` (cosmetic; deliberate)

Commit `2e32e45` removed the import after `aea2ac2` added it.

Path: `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_seedqr.rs:1-15`. Sibling `cli_*.rs` tests (e.g., `cli_repair.rs:19`, `cli_inspect.rs:19`, `cli_xpub_search_*.rs`) consistently carry `use predicates::prelude::*;`. The current code uses `predicates::str::contains(...)` as a fully-qualified function path (which works without prelude trait imports), but the divergence from the sibling convention is a minor stylistic inconsistency that future readers may "fix" by adding the import back and then re-encounter the unused-import warning under `cargo clippy -- -D warnings`.

**Recommendation:** add a 1-line `// NOTE: predicates::str::contains is a free function path; prelude import not required.` comment near the top of the file to short-circuit the loop. NOT blocking for tag.

## Verification scorecard

- **(1) brainstorm decisions in code:** PASS (standard-only / 12+24 / English-locked / no-new-deps / no-new-ToolkitError-variant — all confirmed in `src/seedqr.rs:23-53` + `Cargo.toml:19-46`).
- **(2) all R0 folds (C1-C4 + I1-I6):** PASS.
  - C1 hand-rolled `impl Display` at `src/seedqr.rs:32-51`.
  - C2 exit-1 routing via `BadInput` confirmed in `cmd/seedqr.rs:58-60` + tests asserting `.code(1)`.
  - C3 `value_parser = parse_from_input` at `cmd/seedqr.rs:47`.
  - C4 art-index `0102` (=102) at `src/seedqr.rs:143-145` and `tests/cli_seedqr.rs:11`.
  - I1 node-type-Phrase guard at `cmd/seedqr.rs:136-140`.
  - I2 secret-memory hygiene (Zeroizing + mlock pins + secret_in_argv_warning) at `cmd/seedqr.rs:93-103, 108, 142-153, 158`.
  - I3 `"--digits"` in `secrets.rs:56` + test at `:73`.
  - I4 GUI placement (deferred to v0.15.0 lockstep PR, NOT in scope here).
  - I5 no `.map(|_| 0)` at `main.rs:121`.
  - I6 cell count: `tests/cli_seedqr.rs` carries ~32 `#[test]` cells + 18 unit cells in `src/seedqr.rs` ≥ 30 target.
- **(3) code-quality:** CHANGELOG asserts `cargo clippy --all-targets --workspace -- -D warnings` clean + all 113 test groups PASS.
- **(4) `flag_is_secret("--digits")`:** PASS (`secrets.rs:56`).
- **(5) secret-memory hygiene:** PASS (see I2 above).
- **(6) `Command::Seedqr` placement:** PASS (`main.rs:77` between `SeedXor` L75 and `Slip39` L79; `pub mod seedqr;` at `lib.rs:69` and `cmd/mod.rs:14`, both alphabetical).
- **(7) SemVer-MINOR consistency:** PASS. `Cargo.toml:3` → `0.30.0`; `scripts/install.sh:32` → `mnemonic-toolkit-v0.30.0`; `CHANGELOG.md:9` `## mnemonic-toolkit [0.30.0]` with "SemVer-MINOR release" header.
- **(8) manual cross-impl recipe:** PASS (`docs/manual/src/40-cli-reference/41-mnemonic.md:1696-1721`); no `design/` references in chapter body.
- **(9) no new ToolkitError variants:** PASS (grep for `Seedqr|seedqr` in `src/error.rs` = no matches).
- **(10) no new Cargo.toml deps:** PASS (`Cargo.toml` dep list unchanged vs v0.29.0; `bip39` already present at line 31).
- **(11) `gui_schema_lists_all_twenty_subcommands` test:** PASS (`tests/cli_gui_schema.rs:50, 90-92`); asserts 20 alphabetically-sorted subcommands including new `seedqr-decode` + `seedqr-encode`.

## Special-concern verdicts

- **Task 3 `use mnemonic_toolkit::seedqr::` precedent-aligned:** CONFIRMED. `cmd/slip39.rs:48` uses `use mnemonic_toolkit::slip39::{...}` and `cmd/seed_xor.rs:21` uses `use mnemonic_toolkit::seed_xor::{...}`. The spec-reviewer's NON-COMPLIANT verdict was incorrect; user's override stands.
- **`canonical_digits` necessity:** CONFIRMED. `src/seedqr.rs::decode` returns the PHRASE (line 95), NOT the stripped digits. CLI `run_decode` consumes user input that may carry leading/trailing whitespace (test `decode_strips_whitespace` at `src/seedqr.rs:180-183` exercises this); the JSON envelope's `digits` field MUST echo the canonical 48/96-digit form, so re-stripping in `cmd/seedqr.rs:111-116` is load-bearing. The code-quality reviewer's "redundant" finding was incorrect; user's override stands.
- **20-subcommand count:** CONFIRMED at `tests/cli_gui_schema.rs:50` (`gui_schema_lists_all_twenty_subcommands`) + literal list at `:67-89`. Previously 18 (per comment at `:62-64`); now 20 with `seedqr-decode` + `seedqr-encode`.

## Verdict

**GREEN — tag and push.**

The cycle is shippable. All P0-locked decisions, R0-fold deltas, and project-convention invariants are present in the committed code. The only finding is M1 (cosmetic predicates-import-divergence) which is non-blocking and merely a future-reader hint.
