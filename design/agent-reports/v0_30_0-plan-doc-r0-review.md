# v0.30.0 plan-doc R0 review

**Reviewer:** opus
**Round:** R0
**Plan under review:** `design/PLAN_mnemonic_toolkit_v0_30_0.md`
**Brainstorm:** `design/BRAINSTORM_v0_30_0_seedqr.md`
**Source SHA:** `4d82a3c` (brainstorm commit; current master HEAD)
**Date:** 2026-05-21

**Tooling note:** the R0 reviewer's available toolset (Read/Grep/Glob/WebFetch/WebSearch) does NOT include Write or Edit. Plus system prompt mandates "Do NOT Write report/summary/findings/analysis .md files." This review is the verbatim review text persisted by the orchestrator.

## Critical (C)

### C1 — `thiserror::Error` derive but `thiserror` is NOT a dependency

**Plan-doc location:** Task 2 Step 3, `seedqr.rs` SeedqrError definition.
**Source ground truth:** `crates/mnemonic-toolkit/Cargo.toml` — no `thiserror` entry. Grep confirms `thiserror` only appears in `repair.rs` docstrings + CHANGELOG mentions, not as an active dep.

**Issue:** Plan-doc prescribes
```rust
#[derive(Debug, thiserror::Error)]
pub enum SeedqrError {
    #[error("invalid digit count (expected 48 or 96; got {got})")]
    ...
}
```

This will fail to compile. The project's lib-local-error precedent (`final_word.rs:69`, `seed_xor.rs:31-67`, `slip39/error.rs:28-29`) uses `#[derive(Debug, Clone, PartialEq, Eq)]` + hand-rolled `impl std::fmt::Display` + `impl std::error::Error`. The plan-doc's `thiserror::Error` derive + `#[error("...")]` attribute syntax is incompatible without adding `thiserror` to Cargo.toml.

**Fix:** Either (a) add `thiserror = "1"` to Cargo.toml `[dependencies]` and the brainstorm/plan-doc declare this as a NEW dep (departs from precedent — also risks rebroadcasting the brainstorm's "zero new Cargo.toml deps required" claim at recon §A3 L58); OR (b) rewrite SeedqrError with the existing hand-rolled `impl Display` precedent, matching `seed_xor.rs:45-67`. Option (b) is the project-precedent-aligned fix.

### C2 — `BadInput` exit code is 1, not 2

**Plan-doc location:** Manual chapter "Exit codes" block (Task 5 Step 1), CHANGELOG (Task 6 Step 3), brainstorm §"Error handling" L143.
**Source ground truth:** `crates/mnemonic-toolkit/src/error.rs:429` — `ToolkitError::BadInput(_) => 1,`. Test at L806: `assert_eq!(ToolkitError::BadInput("x".into()).exit_code(), 1);`.

**Issue:** The plan-doc, brainstorm, and proposed manual chapter all assert "exit code 2" for SeedqrError. This is factually wrong: `ToolkitError::BadInput` exits with code **1**. Same-precedent subcommands `seed-xor` / `slip39` / `final-word` (all using `BadInput` for lib-local errors) exit with 1, not 2.

**Fix:** Replace every "exit 2" / "exit code 2" claim referring to SeedqrError-mapped-via-BadInput with "exit 1". Affected sections: manual chapter "Exit codes" block, CHANGELOG, brainstorm §"Error handling" L143.

### C3 — `SeedqrEncodeArgs.from: FromInput` missing `value_parser` — won't compile

**Plan-doc location:** Task 3 Step 2, `SeedqrEncodeArgs` definition.
**Source ground truth:** `crates/mnemonic-toolkit/src/cmd/seed_xor.rs:46-52` shows the working pattern includes `value_parser = parse_from_input` in the `#[arg]` block. No `FromStr` impl exists for `FromInput`.

**Issue:** Plan-doc prescribes:
```rust
#[arg(
    long = "from",
    value_name = "phrase=VALUE|-",
    required = true,
)]
pub from: FromInput,
```

clap-derive cannot synthesize a parser for `FromInput` without a `FromStr` impl or explicit `value_parser`. Build will fail.

**Fix:** Add `value_parser = parse_from_input` to the `#[arg]` block and import `parse_from_input` alongside `FromInput`.

### C4 — `DIGITS_24` fixture index 99 for "art" is wrong (actually index 102)

**Plan-doc location:** Task 2 Step 3 unit test constants + Task 4 Step 1 CLI test constants + brainstorm §Fixtures L177.
**Source ground truth:** BIP-39 English wordlist (verified against `/home/bcg/.cargo/registry/src/.../splitmonic-0.1.0/src/wordlist/words/english.txt`): line 100 = "arrest" (0-indexed: 99); line 103 = "art" (0-indexed: 102). The canonical Trezor 24-word "all-zeros-entropy" vector last word "art" is correct, but the **index of "art" is 102, NOT 99**.

**Issue:** Plan-doc hard-codes `DIGITS_24 = "...0099"` (92 zeros + 0099). If "art" is at index 102, `decode_24_word_canonical` will fail (decoding 0099 yields "arrest") and `encode_24_word_canonical` will fail (encoding "abandon...art" yields "...0102"). All 24-word fixture-based tests will fail.

**Fix:** Change `DIGITS_24 = "...0099"` → `DIGITS_24 = "...0102"` (92 zeros + `0102`). Update brainstorm §Fixtures narrative to cite index 102 not 99.

## Important (I)

### I1 — `run_encode` does not validate `args.from.node == NodeType::Phrase`

**Source ground truth:** `crates/mnemonic-toolkit/src/cmd/seed_xor.rs:163-167` rejects non-phrase nodes with a clean `BadInput`. `slip39.rs` does the same.

**Issue:** Plan-doc's `run_encode` only checks `args.from.value == "-"` and then immediately feeds `args.from.value` into `seedqr_encode`. A user passing `mnemonic seedqr encode --from xpub=xpub6...` would have the xpub string sent through BIP-39 word tokenization, producing a baffling "BIP-39 checksum failure" error.

**Fix:** Insert at top of `run_encode`:
```rust
if args.from.node != NodeType::Phrase {
    return Err(ToolkitError::BadInput(
        "seedqr encode only accepts phrase=<value> or phrase=-".into(),
    ));
}
```
Add a test cell `encode_rejects_non_phrase_node` to `tests/cli_seedqr.rs`.

### I2 — Secret-memory hygiene missing (no Zeroizing, no mlock pin, no secret_in_argv_warning)

**Source ground truth:** `cmd/seed_xor.rs:163-178` shows the canonical pattern: (a) argv-leakage advisory via `secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-")` for inline secrets; (b) wrap phrase in `Zeroizing<String>`; (c) `mnemonic_toolkit::mlock::pin_pages_for(master_phrase.as_bytes())` page pin.

**Issue:** Both `--from phrase=` (encode) and `--digits` (decode — input encodes a seed) carry secret material. The plan-doc treats them as plain `String`. This is a Cycle-B-era discipline gap (mlock established v0.10.0).

**Fix:** Mirror `seed_xor.rs:163-178`:
- Inline-form `secret_in_argv_warning` advisories for both `--from phrase=...` and `--digits ...`.
- `Zeroizing<String>` wrapping for the resolved phrase + decoded phrase + the digits buffer.
- `mlock::pin_pages_for(..)` page pins on the resolved phrase + computed digits.

### I3 — `secrets.rs::flag_is_secret` not updated for `--digits`

**Source ground truth:** `crates/mnemonic-toolkit/src/secrets.rs:49-59` — the `flag_is_secret` match block enumerates secret-bearing CLI flags. `--digits` is missing.

**Issue:** `--digits` carries a SeedQR-encoded BIP-39 seed — fully secret-bearing. GUI consumers of `gui-schema` JSON expect this classification for paste-warn / run-confirm modal pathways.

**Fix:** Add `"--digits"` to the `flag_is_secret` match arm + add a `--digits classifies as secret` test cell. Note `--from` is intentionally NOT in `flag_is_secret` because secrecy is value-dependent (`secret_taxonomy::SECRET_NODE_TYPES` covers phrase-node-form). `--digits` is unconditionally secret, so flag-level inclusion is correct.

### I4 — GUI schema-mirror placement citation names wrong upper-neighbor

**Source ground truth:** `mnemonic-gui/src/schema/mnemonic.rs`:
- L2359: `seed-xor-split`
- L2367: `seed-xor-combine`
- L2375: `slip39-split`
- L2383: `slip39-combine`

The upper neighbor of `seedqr-*` (alphabetically next) is `slip39-split` at L2375, NOT `slip39-combine` (L2383).

**Issue:** Plan-doc Step 4 prose says "between `seed-xor-combine` and `slip39-combine`" — that range spans the existing `slip39-split` entry. Inserting between them would put `seedqr-*` AFTER `slip39-split`, breaking alphabetical order.

**Fix:** Correct the prose: "between `seed-xor-combine` (L2367) and `slip39-split` (L2375)". Additionally observe: the existing pattern is "split (create-side) before combine (recover-side)" within each parent. For seedqr, the create-side is `encode` and the recover-side is `decode`. So the GUI ordering should be `seedqr-encode` first, then `seedqr-decode` (matching the seed-xor/slip39 verb-ordering convention), even though the plan-doc lists `decode` first in clap-derive's enum order.

### I5 — Dispatch arm uses vestigial `.map(|_| 0)`

**Source ground truth:** `crates/mnemonic-toolkit/src/main.rs:117-119` — `FinalWord`, `SeedXor`, `Slip39` dispatch arms invoke `cmd::*::run(...)` directly with no `.map(|_| 0)`. The plan-doc's prescribed `run` already returns `Result<u8, ToolkitError>` (returns `Ok(0)`).

**Fix:** Drop `.map(|_| 0)`:
```rust
Command::Seedqr(args) => cmd::seedqr::run(args, stdin, stdout, stderr),
```

### I6 — Plan-doc file-structure header cite "tests/cli_seedqr.rs ~400 LOC (30-60 cells)" but actual prescribed test file ships ~18 cells

**Issue:** The brainstorm §Test-plan locks 30-60 cells. The plan-doc cuts back to ~18 in actual coverage. Sources of the gap: no JSON-mode encode rejection cells, no stdin-form-encode tests for both forms, no exit-code-numeric assertions, no negative cells for the deferred 15/18/21/CompactSeedQR variants.

**Fix:** Widen Task 4 Step 1 to ≥30 cells: add (a) `--from non-phrase node` rejection cells (covers I1 fold); (b) `--digits=` JSON-mode encode rejection cells; (c) exit-code-numeric assertions (e.g., `assert_eq!(exit_code, 1)`); (d) 13/15/18/21/25-word counts rejected; (e) stdin-form-encode tests for both forms; (f) round-trip through JSON envelope (encode-via-json-out → parse JSON → decode --digits=<from-json>).

## Minor (M)

### M1 — `s.parse().expect("4 ASCII digits")` panic path in decode

Defensive: convert to `?` return path. Low priority — current code is correct under the established digit-validation invariant.

### M2 — Manual chapter cross-impl recipe has placeholder `<CITE_SYMBOL_PATH_FROM_TASK_1_§A4>`

By-design (Task 1 §A4 deferred); Step 1 explicitly instructs to fill before commit.

### M3 — Strike-through markdown rendering caveat

Acknowledged in risk register.

### M4 — CHANGELOG date placeholder

Standard pattern.

### M5 — `decode` / `encode` ordering vs split/combine precedent

Sibling `seed-xor` / `slip39` list `split` before `combine`. The plan-doc orders alphabetically (`decode` < `encode`). See I4 for GUI ordering implication.

## Verdict

**RED** — 4 Critical findings (one or more would prevent the prescribed code from compiling; one would cause every 24-word fixture-based test to fail; one would emit a wrong exit code documented in manual + CHANGELOG). 6 Important findings would ship a working but degraded surface (memory hygiene, secret classification, non-phrase node validation, citation drift, ordering convention).

## Summary

The plan-doc captures the architectural locks faithfully and the phase decomposition is sound, but four Critical issues block execution as written: (1) `thiserror::Error` derive on `SeedqrError` won't compile because `thiserror` is not in Cargo.toml — the lib-local-error precedent (`final_word`/`seed_xor`/`slip39`) uses hand-rolled `impl Display`; (2) the exit-code claim of "2" for SeedqrError is wrong because `ToolkitError::BadInput` exits 1 per `error.rs:429`; (3) `SeedqrEncodeArgs.from: FromInput` is missing the required `value_parser = parse_from_input` and won't compile; (4) the 24-word `DIGITS_24` test fixture hardcodes index 99 for "art" which IS incorrect (index 99 is "arrest"; "art" is at index 102), causing every 24-word test to fail at the TDD step. Six Important issues concern memory-hygiene precedent (Zeroizing/mlock/argv-warning absent), missing `NodeType::Phrase` validation in `run_encode`, omission of `--digits` from `secrets.rs::flag_is_secret`, a citation drift in GUI schema-mirror placement (`slip39-combine` named where `slip39-split` is the actual upper neighbor), vestigial `.map(|_| 0)` in the dispatch arm, and a cell-count drift (header claims 30-60 cells, actual prescribed test file ships ~18). Recommendation: fold all four Critical + all six Important before re-dispatching R1.
