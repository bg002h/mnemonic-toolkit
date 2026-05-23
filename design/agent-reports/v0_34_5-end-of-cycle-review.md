# v0.34.5 — End-of-cycle architect review (opus) — MANDATORY pre-tag gate

**Date:** 2026-05-22
**Cycle:** mnemonic-toolkit v0.34.5 — MiniKey stdout-redaction hardening + `SECRET_NODE_TYPES_ARGV`
**Branch:** `v0.34.5-minikey-leak-hardening`
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle gate (security-handling cycle)
**Scope reviewed:** full cycle diff `/tmp/v0_34_5_cycle.diff` (commits `0724d2b`..`7827803`) + live source

---

## Critical
(none)

## Important
(none)

## Minor

- **Pre-existing stale/now-contradictory doc comment on `is_argv_secret_bearing`** — `convert.rs:108-116`. The comment said the narrow `is_secret_bearing()` "is preserved because it gates separate stdout-redaction / secret-on-stdout machinery (`convert.rs:769, 796`)" and referenced `convert-minikey-stdout-redaction` as an open follow-up "covering widening THAT predicate." This cycle switched that machinery to the wide predicate and resolved the follow-up, so the comment is now contradictory. Out of the strict 2-swap scope, but it's the very doc explaining the changed predicate. **[FOLDED — comment rewritten to reflect v0.34.5.]**

## Verification ledger

1. **Leak closed.** `convert.rs:1045` `from_value` now uses `is_argv_secret_bearing()` → minikey input → `None`; `ConvertJson.from_value` has `#[serde(skip_serializing_if="Option::is_none")]` (`:443`) → field omitted → `is_null()`. `:1076` warning also widened.
2. **No second echo path.** Non-JSON path (`:1066-1067`) emits only outputs, never the input; repair-echo path (`:982/987`) is gated to Ms1/Mk1 (`:971-975`) — minikey can't reach it; `env_sentinel.rs` minikey ref is a doc comment, not an echo. No un-redacted second echo exists.
3. **No over-redaction.** `is_argv_secret_bearing` = narrow + MiniKey only; non-secret conversions still echo `from_value`. `:1076` is a genuine no-op (MiniKey output-unreachable; WIF output already trips it).
4. **Test meaningful + non-vacuous.** `minikey_input_redacted_in_json_from_value` asserts `from_value.is_null()` AND `!stdout.contains(VEC22_KEY)` AND WIF output present; RED pre-fix.
5. **Const + parity.** `SECRET_NODE_TYPES_ARGV` (`secret_taxonomy.rs:95-105`) = 8 narrow + `minikey` (9); new parity test iterates ALL variants; narrow parity + `minikey_intentionally_excluded_from_persistence_taxonomy` intact (MiniKey ∉ narrow, ∈ argv).
6. **Versions aligned.** Cargo.toml=0.34.5, Cargo.lock=0.34.5, install.sh self-pin=v0.34.5, CHANGELOG top=[0.34.5].
7. **FOLLOWUPs resolved accurately;** M2 fold applied (`FOLLOWUPS.md:1912` `line 107`→`117`).
8. **Scope disciplined.** 2 swaps + 1 const + 2 tests + version artifacts + docs. No clap flag change → PATCH + no GUI/manual lockstep correct; narrow `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` untouched → GUI snapshot unaffected.

VERDICT: GREEN (0C/0I)

---

## Fold disposition (controller)
GREEN (0C/0I) → gate satisfied. Folded M1 (rewrote the `is_argv_secret_bearing` doc comment to reflect that the wide predicate now gates the stdout-redaction machinery as of v0.34.5; removed the stale `769,796` cite + the now-resolved follow-up reference). Doc-only, zero behavioral impact, no R0 re-dispatch (no Critical/Important).
