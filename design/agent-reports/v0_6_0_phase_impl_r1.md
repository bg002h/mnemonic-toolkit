# v0.6.0 Phase Impl — code-reviewer r1

**Outcome:** 0C/2I/2L/1N initial → 0C/0I after foldings. APPROVED.

## Scope reviewed
13 staged paths for the v0.6.0 `mnemonic convert` subcommand:
- `crates/mnemonic-toolkit/src/cmd/convert.rs` (NEW, ~600 LOC).
- `crates/mnemonic-toolkit/src/cmd/mod.rs` (mod registration).
- `crates/mnemonic-toolkit/src/derive.rs` + `derive_slot.rs` (account_xpriv field plumbing for `phrase/entropy → xprv` edge).
- `crates/mnemonic-toolkit/src/error.rs` (new `ConvertRefusal(String)` variant; exit 2).
- `crates/mnemonic-toolkit/src/main.rs` (Command::Convert wired).
- 4 new test files: `cli_convert_happy_paths.rs`, `cli_convert_refusals.rs`, `cli_convert_json.rs`, `cli_convert_help_fixtures.rs`.

## Critical findings folded inline

**C-1: `phrase/entropy → mk1` was using the xpub-specific refusal text** — fixed by adding (Phrase, Mk1) and (Entropy, Mk1) to the catch-all in `classify_edge` (refusal_one_way intercepts before `compute_outputs`); the dead `Mk1 =>` arm in compute_outputs becomes `unreachable!()`.

**C-2: `classify_edge` had systematic gaps for unreachable edges** — refactored to use a positive-list `is_supported_direct_edge` predicate. Any (from, to) NOT in the supported set is automatically classified as one-way barrier (exit 2, byte-exact §3.a stderr). Fixes `(Xprv, Phrase/Entropy/Wif/Ms1/Mk1)`, `(Wif, Ms1/Mk1)`, `(Xpub, Ms1)` etc.

## Important findings folded inline

**I-1: `wif → xpub` sentinel stderr warning missing** — added before output computation: `"warning: wif → xpub emits a depth-0 sentinel with a zeroed chain code; this xpub is not BIP-32 derivable"`.

**I-2: No happy-path test for `mk1 → xpub`** — added `mk1_to_xpub_decode` test using a two-string mk1 fixture from the bip84-mainnet vector; asserts xpub/fingerprint/path projections.

## Low findings folded inline

**L-1: `TREZOR_24_ZERO_MS1` constant was holding 16-byte zero entropy ms1** — renamed to `TREZOR_12_ZERO_MS1` across happy-paths + refusals tests.

**L-2: `mk1 → fingerprint` returned empty string when origin_fingerprint absent** — replaced `.unwrap_or_default()` with `.ok_or_else(|| BadInput(...))?` so the absence is surfaced as a clear error.

## Nit folded inline

**N-1: `ms1_to_phrase_via_implicit_traversal` test name was misleading** — renamed to `ms1_to_phrase_direct_edge`.

## Special edges noted

- `phrase → wif` and `entropy → wif` are deferred-not-refused: they pass classify_edge (added to supported set) and `compute_outputs` returns `BadInput` with a deferral message pointing at the missing leaf-WIF derivation. Exit 1 (deferral, not refusal). Distinction documented in the folding rationale.

## Test results

230 lib + 67 integration tests pass; 2 lib ignored (pre-existing). 23 new convert tests across 4 files (was 22 pre-fold; +1 from mk1→xpub coverage).
