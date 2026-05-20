# v0.28.0 Phase 7 (G1) — P7C R0 self-review

**Phase:** P7C — Integration cells in `tests/cli_import_wallet_bsms.rs` + new fixtures `bsms-4line-*.txt` ×4 in `tests/fixtures/wallet_import/`.
**Reviewer:** Executor self-review (Task-dispatch unavailable in autonomous session).
**Source SHA reviewed against:** branch `v0.28.0/g1-bsms-4line` rooted at `release/v0.28.0` `71592bc`.
**Verdict:** GREEN.

---

## Critical

NONE.

## Important

### I1. Fixture #4 "first-address-mismatch refusal" semantic interpretation

The user-prompt fixture name says "refusal" but the existing 6-line implementation treats first-address mismatch as a WARNING (informational; exit 0; parse succeeds). P7A does NOT add a hard-refusal path for 4-line (SPEC §10.6 scope limits do not introduce one). I treated the user-prompt wording as shorthand for "the negative-case fixture that triggers the cross-validation diagnostic" — matching the existing 6-line semantics for principle-of-least-surprise. Integration cell `bsms_4line_first_address_mismatch_emits_warning` exercises the WARNING-and-succeed behavior; if the user intends a hard refusal for 4-line, the SPEC §10.2 / §10.6 contracts would need amendment first.

## Minor

### M1. helper `examples/p7_derive_addrs.rs` deleted post-fixture-vendor

The fixture first-addresses are real `/0/0` derivations from the descriptor xpubs (computed via a one-shot `cargo run --example p7_derive_addrs`). The helper was deleted after vendoring the values into the fixtures + happy-path cells; the addresses are now baked-in literals. Cross-check trick: `bsms_4line_sortedmulti_2of3_happy_path` succeeds iff the fixture's line-4 byte-equals the toolkit's `derive_first_address` — if a future descriptor-library bump changes derivation output, the assertion `!stderr.contains("first-address mismatch")` will catch it.

### M2. integration cells run via assert_cmd subprocess

Each integration cell is ~5-30 LOC and forks a `mnemonic` subprocess. Total P7C runtime added is small (~50ms across 6 cells per the cargo test output `finished in 0.08s`). No `#[ignore]` gating needed — none depend on sibling repos.

### M3. `bsms_4line_via_bundle_roundtrip_json` accepts either `ok` or `blocked_no_emitter`

Per SPEC §2.2, the roundtrip envelope's `status` field can be `"ok"`, `"blocked_no_emitter"`, or `"canonicalize_failed"`. The cell asserts EITHER `"ok"` OR `"blocked_no_emitter"` (the load-bearing assertion is that `canonicalize_bsms` did NOT fail on the 4-line shape — proving the P7A R5-C2 mirror fix landed). The exact status value depends on whether the BSMS round-trip emitter is wired into the import-side roundtrip computation; that wire-up is tracked by FOLLOWUP `wallet-export-bsms-emitter` (deferred beyond v0.28.0).

## Integration cell coverage matrix

| Cell | Fixture | SPEC clause exercised |
|---|---|---|
| `bsms_4line_sortedmulti_2of3_happy_path` | `bsms-4line-sortedmulti-2of3.txt` | §10.1 happy-path + §10.2 first-address byte-equal (no WARNING) |
| `bsms_4line_singlesig_wpkh_happy_path` | `bsms-4line-singlesig-wpkh.txt` | §10.1 singlesig variant (threshold = None) |
| `bsms_4line_no_path_restrictions_accepted` | `bsms-4line-no-path-restrictions.txt` | §10.1 line-3 = `"No path restrictions"` literal |
| `bsms_4line_first_address_mismatch_emits_warning` | `bsms-4line-first-address-mismatch.txt` | §10.2 first-address mismatch WARNING |
| `bsms_6line_still_accepted_with_deprecation_notice` | inline-built 6-line blob | §10.4 DEPRECATION NOTICE shape (CLI surface) |
| `bsms_4line_via_bundle_roundtrip_json` | `bsms-4line-sortedmulti-2of3.txt` | §10 + §7.3.1 (canonicalize_bsms accepts 4-line) |

## Fixture inventory

| Fixture | Lines | Descriptor type | First-address derivation source |
|---|---|---|---|
| `bsms-4line-sortedmulti-2of3.txt` | 4 | `wsh(sortedmulti(2,...))` 2-of-3 mainnet | real `/0/0` via `derive_first_address` |
| `bsms-4line-singlesig-wpkh.txt` | 4 | `wpkh(...)` BIP-84 mainnet | real `/0/0` via `derive_first_address` |
| `bsms-4line-no-path-restrictions.txt` | 4 | `wsh(sortedmulti(2,...))` 2-of-2 mainnet | real `/0/0` via `derive_first_address` |
| `bsms-4line-first-address-mismatch.txt` | 4 | same as `sortedmulti-2of3` | deliberately-wrong `bc1qfake...` literal |

## Cargo test result (live)

```
running 28 tests
(...all pass...)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

Inline unit tests (`cargo test --bin mnemonic wallet_import::bsms`):
```
running 21 tests
test wallet_import::bsms::tests::extract_threshold_u8_overflow_is_typed_error ... ok
test wallet_import::bsms::tests::parse_4line_happy_path_populates_audit_with_empty_sentinels ... ok
test wallet_import::bsms::tests::parse_4line_first_address_mismatch_emits_warning ... ok
test wallet_import::bsms::tests::parse_4line_line3_preserved_verbatim_in_audit ... ok
test wallet_import::bsms::tests::parse_6line_emits_deprecation_notice_shape ... ok
(...)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 460 filtered out
```

Full `cargo test` run: no failures; clippy clean (one doc-list warning surfaced and folded inline pre-commit).

## Reviewer-loop reconverge

R0 GREEN; no folds; no R1.

## P7 (A+B+C) end-of-phase gate

- [x] P7A 3 source-site edits applied + grep-discipline post-edit count is 0/7
- [x] P7B 6-line NOTICE replaced with §10.4 DEPRECATION (4 writeln! lines)
- [x] P7C 6 integration cells + 4 fixtures vendored
- [x] All unit + integration tests green (cargo test full run)
- [x] Clippy clean
- [x] Per-phase R0 reviews persisted to `design/agent-reports/v0_28_0-P7{A,B,C}-r0-review.md`

Ready for commit + PR.
