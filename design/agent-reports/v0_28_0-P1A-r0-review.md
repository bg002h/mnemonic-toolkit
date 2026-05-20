# v0.28.0 P1A architect review — R0 (inline self-review)

**Phase:** P1A — Sparrow Wallet parser skeleton (sniff impl only; parse deferred to P1B).
**Reviewer:** inline self-review (agent-aa74aea6602d044ab).
**Date:** 2026-05-19.
**Scope of review:** the four files mutated by P1A:

- `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` (new; +274 LOC src + tests inline)
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` (+3 LOC: `pub(crate) mod sparrow;`)
- `crates/mnemonic-toolkit/src/wallet_import/sniff.rs` (+2 LOC: import + bool flip)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (+9 LOC: `SniffOutcome::Sparrow => "sparrow"` arm)

**SPEC anchor:** `design/SPEC_wallet_import_v0_28_0.md` §11.1.
**Plan-doc anchor:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` P1A row at line 491 + §S.1 at line 197-219.

## Verdict

**GREEN.** 0 Critical, 0 Important, 0 Minor. Ready to commit.

## Critical findings

(none)

## Important findings

(none)

## Minor findings

(none)

## Verifications run

- `cargo build -p mnemonic-toolkit` → success.
- `cargo test -p mnemonic-toolkit --bin mnemonic sparrow` → 14/14 pass (10 sniff cells + 1 parse-stub cell + pre-existing `canonicalize_sparrow_skeleton_returns_not_yet_implemented` smoke).
- `cargo test -p mnemonic-toolkit --test cli_import_wallet_sniff --test cli_import_wallet_p0c_dispatch --test cli_import_wallet_bitcoin_core --test cli_import_wallet_bsms` → all green; existing v0.28.0 P0 integration cells unbroken.
- `cargo test -p mnemonic-toolkit` (full suite) → all green; no regression.
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

## Sniff-fixture coverage (plan-doc §S.1 row)

Positive (3 ≥ plan-doc "~3"):
- `sniff_true_on_minimal_single_blob` — SINGLE / P2WPKH.
- `sniff_true_on_minimal_multi_blob` — MULTI / P2WSH 2-of-3 sortedmulti.
- `sniff_true_on_p2tr_blob` — SINGLE / P2TR (taproot singlesig).

Negative (5; plan-doc requested BSMS/Core/Specter):
- `sniff_false_on_bsms_blob` — BSMS 4-line text shape.
- `sniff_false_on_bitcoin_core_blob` — Core `listdescriptors` envelope.
- `sniff_false_on_specter_blob` — Specter blob carrying `label`+`blockheight`+`descriptor`+`devices`.
- `sniff_false_on_empty_keystores` — positive-marker #5 (non-empty array).
- `sniff_false_on_missing_nested_script` — positive-marker #4 (nested `script`).
- `sniff_false_on_unrecognized_policy_type_value` — positive-marker #2 value-set check.
- `sniff_false_on_bare_array` — positive-marker #1 (top-level object).
- `sniff_false_on_random_text` + `sniff_false_on_empty_blob` — generic negative coverage.

## P1B parse-impl deferral (parse arm signature contract)

`SparrowParser::parse` returns `Err(BadInput("sparrow parse: not yet implemented; landing in Phase P1B"))` so the `WalletFormatParser` trait bound is satisfied. The placeholder is regression-pinned via `parse_returns_not_yet_implemented_in_p1a` so a forgotten body-swap in P1B surfaces as a test failure rather than silent regression.

## Critical-constraint compliance — `SniffOutcome::Sparrow` dispatch arm added

The user-message critical constraint flagged the auto-sniff catch-all `unreachable!` at `cmd/import_wallet.rs:325-329`: now that `sniff_format` CAN return `SniffOutcome::Sparrow` (P1A wired the bool), the catch-all would fire on auto-sniffed Sparrow blobs without an explicit arm. The P1A diff adds `SniffOutcome::Sparrow => "sparrow"` BEFORE the catch-all (which routes to the existing P0C-pre-stubbed `"sparrow" => unimplemented!("P1C: parse not yet wired")` arm at line 365). End-to-end behavior matches explicit `--format sparrow`: both paths hit the same P1C placeholder panic, awaiting P1C's parse wiring.

## SPEC §11.1 schema-cite — `SparrowSourceMetadata` field shape match

| SPEC §11.1 field | P1A struct field | match |
|---|---|---|
| `pub label: Option<String>` | `pub(crate) label: Option<String>` | ✓ |
| `pub policy_type: SparrowPolicyType` | `pub(crate) policy_type: SparrowPolicyType` | ✓ |
| `pub script_type: String` | `pub(crate) script_type: String` | ✓ |
| `pub dropped_fields: Vec<String>` | `pub(crate) dropped_fields: Vec<String>` | ✓ |

`pub(crate)` visibility matches the existing `CoreSourceMetadata` / `BsmsAuditFields` convention at `mod.rs:88,141` (R0 M6 note in plan-doc §S.5).

`#[allow(dead_code)]` on the struct + enum is intentional for P1A — the fields are populated in P1B's parse body. Per CLAUDE.md alphabetical-discipline note: `SparrowPolicyType` has 2 variants (`Single`, `Multi`) which is alphabetical by construction.

## ImportProvenance::Sparrow NOT added in P1A

Per the user-message instructions, the `ImportProvenance::Sparrow(SparrowSourceMetadata)` variant is added in **P1C** (the dispatch flip phase), not P1A. P1A only ships the metadata struct + parser skeleton + sniff wiring. This keeps the P1A diff bounded to "make sniff_format return Sparrow when blob is Sparrow"; provenance + accessor wiring is downstream of the parse impl in P1B/C ordering.
