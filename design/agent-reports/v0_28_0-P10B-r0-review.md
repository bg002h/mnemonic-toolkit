# Phase P10B — architect R0 review

**Reviewer:** Opus 4.7 (acting as feature-dev:code-architect) — instance-H self-review pre-PR
**Branch:** `v0.28.0/h-core-fixtures`
**Commit under review:** P10B scope (4 more Core fixtures + 4 parse-only + 1 sniff-negative cell)
**Base SHA:** `33ec61d` (release/v0.28.0 tip; rebased from initial branch-point `71592bc`)
**Plan-doc:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` Phase 10 P10B row + §S.9 owner-phase tags

---

## Scope (verbatim from plan-doc Phase 10 table)

> **P10B** | 4 more Core fixtures: `core-explicit-active-false.json`, `core-mainnet-receive-change-pair.json`, `core-multipath-receive-change-pair.json`, `core-empty-descriptors-array.json` (negative). Parse-only + sniff-negative cells. | Same | ~0 src + ~250 tests + 4 fixture files | architect R0 |

§S.9 confirms Core-side ownership: "P10B: `core-explicit-active-false.json`, `core-mainnet-receive-change-pair.json`, `core-multipath-receive-change-pair.json`, `core-empty-descriptors-array.json` (4 fixtures)".

---

## Critical (correctness-blocking; would break downstream consumers)

**None.** P10B delivers exactly the 4 fixtures + 5 test cells (4 parse-only + 1 sniff-negative companion) the plan-doc scopes. No SPEC §6/§10 contract changes; no new parser flags; no `wallet_import/*.rs` source modifications.

## Important (would block P10B merge)

**None.** Fixture-corpus expansion is the smallest plan-doc deliverable surface.

## Minor (fold inline or defer)

**M1 — `core-empty-descriptors-array.json` overlaps semantically with existing stdin-based `core_empty_descriptors_array_exit_2` cell (test fn at `cli_import_wallet_bitcoin_core.rs:495-505`).** The existing cell pipes the literal `"{\"descriptors\": []}"` blob via stdin; P10B adds a fixture-FILE round-trip variant. Both assert exit=2 + parse-error template containing `"empty"`. Overlap is **intentional**:

- The stdin variant proves the parser's empty-array refusal path.
- The fixture-file variant proves the parser handles the same shape when read from disk (no shape-discrepancy between in-memory and on-disk consumption).

This mirrors `core_fixture_file_multi_bip84_all` (line 558-564) which provides a fixture-file companion to the synthetic `core_multi_descriptor_emit_all` cell. **No fold.**

**M2 — `core-empty-descriptors-array.json` sniff-negative companion cell** (`core_fixture_file_empty_descriptors_array_sniff_no_match`) exercises the auto-detect path (no `--format` flag) and pins exit=1 + the "could not detect format" template. This validates that:

- (a) `BitcoinCoreParser::sniff` at `bitcoin_core.rs:91-97` returns `false` on empty `descriptors: []` (verified).
- (b) The post-P0D `sniff_format` consult-all-then-count at `sniff.rs:74-105` aggregates 8 parser bools, with 6 still pre-stubbed to `false` until their per-parser P{N}A sub-phase lands.
- (c) With all 8 bools `false`, the dispatch falls through to `SniffOutcome::NoMatch` → caller emits `ImportWalletAmbiguousFormat` exit=1.

This sniff-negative cell pins the v0.28.0 cutover behavior. Once other parsers' P{N}A sub-phases wire their sniff fns in (per the alphabetical-slot discipline in `sniff.rs:84-93`), the cell remains correct as long as no other parser positively claims an empty Core-shaped blob — a robust assumption given each parser's sniff is positive-marker-based per SPEC §11.x. **No fold; documented inline in cell prose.**

**M3 — `core-multipath-receive-change-pair.json` mixes `wpkh()` (BIP-84) + `sh(wpkh())` (BIP-49) script types across the two entries.** This is uncommon in real-world Core output (usually `listdescriptors` returns either all-BIP-84 or all-BIP-49); a "hybrid" multipath blob with mixed wrappers is more exotic. **Rationale for the mixed shape:** the fixture pins the parser's acceptance of distinct script types within a single multi-entry blob (a contract the parser must hold, even if uncommon in practice). Could be revisited to homogenize to all-wpkh or all-sh(wpkh), but the mixed-wrappers shape is strictly more permissive (passes implies single-wrapper case also passes). **No fold.**

**M4 — `core-mainnet-receive-change-pair.json` omits `timestamp`** while `core-bip49-mainnet.json` (the existing receive/change pair using sh(wpkh) script-type) ALSO omits `timestamp` (per `tests/fixtures/wallet_import/core-bip49-mainnet.json:8`). Consistent. **No fold.**

**M5 — Test cells use the canonical `run_core_file_select(&p, "all")` helper for all positive cases.** This mirrors existing fixture-FILE cells (`core_fixture_file_multi_bip84_all` line 558-564, `core_select_index_out_of_range_errors` line 675-684). The empty-array negative cell forks to a manual `Command::cargo_bin("mnemonic")` builder because the bare `--format bitcoin-core` dispatch (without `--select-descriptor`) is the load-bearing path (parse-time refusal fires before select-filter). **No fold.**

## Source-grep verification table

| Citation | Verified? | Notes |
|---|---|---|
| `tests/cli_import_wallet_bitcoin_core.rs::fixture_path` helper | YES | `tests/cli_import_wallet_bitcoin_core.rs:92-94` |
| `tests/cli_import_wallet_bitcoin_core.rs::run_core_file_select` helper | YES | `tests/cli_import_wallet_bitcoin_core.rs:124-131` |
| `BitcoinCoreParser::sniff` empty-array branch returns false | YES | `wallet_import/bitcoin_core.rs:95-97` |
| Empty-array parse-error template `"top-level \`descriptors\` array is empty; no bundles to emit"` | YES | `wallet_import/bitcoin_core.rs:138-143` |
| Sniff dispatch consult-all-then-count + `NoMatch` outcome | YES | `wallet_import/sniff.rs:100-104` |
| `"could not detect format"` template | YES | `cmd/import_wallet.rs` error mapping for `ImportWalletAmbiguousFormat` (sniff NoMatch path) |
| `active=false` envelope passthrough | YES | Tested live with `explicit-active-false.json` — stdout shows `active=false` cleanly |
| Multi-entry blob produces N bundles under `--select-descriptor all` | YES | Tested live with both `mainnet-receive-change-pair.json` and `multipath-receive-change-pair.json` — both show `bundles=2` |

## Scope-creep audit

| Item | In plan-doc P10B scope? | Acceptable? |
|---|---|---|
| `core-explicit-active-false.json` fixture file | YES | YES |
| `core-mainnet-receive-change-pair.json` fixture file | YES | YES |
| `core-multipath-receive-change-pair.json` fixture file | YES | YES |
| `core-empty-descriptors-array.json` fixture file | YES (negative case) | YES |
| 4 new parse-only test cells | YES | YES |
| 1 sniff-negative companion cell (`empty_descriptors_array_sniff_no_match`) | YES (plan-doc row says "+ sniff-negative cells") | YES |
| Mixed wpkh + sh(wpkh) script-types in `core-multipath-receive-change-pair.json` | Implicit | YES — see Minor M3 |
| New helper functions, new const declarations, new modules | NO | None added |
| SPEC §6 / §10 changes | NO | None added |
| `wallet_import/bitcoin_core.rs` source changes | NO | None added |
| `gui-schema` changes / GUI mirror updates | NO | None — no GUI lockstep required |

**Net scope assessment:** strict adherence to the plan-doc P10B row + §S.9 inventory. The "sniff-negative cells" plural in the P10B row is satisfied by the 1 sniff-negative companion cell on the empty-array fixture; the 3 happy-path fixtures (`explicit-active-false`, `mainnet-receive-change-pair`, `multipath-receive-change-pair`) do not have sniff-negative companions because each is a positive Core-shape blob (Core sniff would correctly claim them). Adding sniff-positive cells on those 3 would be a separate scope item not in the plan-doc; deliberately omitted.

## Test result

- **Baseline (pre-P10B; with P10A already added):** 29 passed in `cli_import_wallet_bitcoin_core.rs` (25 v0.27.x baseline + 4 P10A).
- **Post-P10B:** 34 passed in `cli_import_wallet_bitcoin_core.rs` (29 + 5 new P10B cells). Δ = +5 (4 parse-only + 1 sniff-negative companion). Full workspace test suite green; clippy `--all-targets -D warnings` green.
- **No `#[ignore]`-gated cells added.** All cells run in `cargo test --workspace` default.

## Overall verdict

**GREEN.**

P10B delivers exactly what the plan-doc scopes: 4 vendored Core fixtures (3 happy-path + 1 negative) + 5 test cells (4 parse-only + 1 sniff-negative companion). No SPEC changes, no source changes, no new infrastructure. Net LOC: ~155 test code + 4 fixture files. Net new tests: +5 (all green). Pre-existing tests unbroken (29 → 29 + 5 new = 34 total).

R0 verdict GREEN → ready to merge to `release/v0.28.0` together with P10A.

---

**Sources:**
- Plan-doc Phase 10 row at `/home/bcg/.claude/plans/unified-meandering-sundae.md:557-562`
- Plan-doc §S.9 owner-phase tags at `/home/bcg/.claude/plans/unified-meandering-sundae.md:409-422`
- Existing `core_empty_descriptors_array_exit_2` cell at `tests/cli_import_wallet_bitcoin_core.rs:493-505`
- Existing `core_fixture_file_multi_bip84_all` cell at `tests/cli_import_wallet_bitcoin_core.rs:557-564`
- v0.28.0 sniff_format consult-all-then-count at `wallet_import/sniff.rs:74-105` (P0D)
