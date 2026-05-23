# v0.34.5 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-22
**Cycle:** mnemonic-toolkit v0.34.5 — MiniKey-leak hardening (`convert` stdout-redaction + `SECRET_NODE_TYPES_ARGV`)
**Branch:** `v0.34.5-minikey-leak-hardening`
**Reviewer:** opus (feature-dev:code-reviewer), R0
**Target:** `design/IMPLEMENTATION_PLAN_v0_34_5_minikey_leak_hardening.md` (verified against live source)

---

## Critical
(none)

The highest-risk claim — **`:1069` is a no-op for MiniKey** — is **verified correct.** `classify_edge` intercepts `(*, MiniKey)` at `convert.rs:663-665` and `(MiniKey, !Wif)` at `:669-671`; the only permitted MiniKey edge is `(MiniKey, Wif)` (`:639`), and the dispatch produces only `Wif` outputs (`unreachable!` at `:1226`,`:1496`). MiniKey is output-unreachable → widening `:1069` cannot add a spurious warning.

The second risk — **`SECRET_NODE_TYPES_ARGV` token set mismatching `is_argv_secret_bearing()`** — also **verified.** `is_secret_bearing()` (`convert.rs:94-106`) = {Phrase, Seedqr, Entropy, Xprv, Wif, Ms1, Bip38, ElectrumPhrase}; `is_argv_secret_bearing()` (`:117-119`) adds MiniKey. Via `as_str()` (`:55-72`) = `{phrase, seedqr, entropy, xprv, wif, ms1, bip38, electrum-phrase, minikey}` — exactly the plan's 9 tokens (order-agnostic `.contains()`). Parity test will pass.

## Important
(none)

## Minor

- **M1 — apply Task 1's `convert.rs:1042`/`:1069` swaps against live lines** (accurate now); reminder only, no action.
- **M2 — stale line cite in the source FOLLOWUP body.** `design/FOLLOWUPS.md:1912` cites `is_argv_secret_bearing` at "line 107"; live is `convert.rs:117`. The plan-doc itself correctly uses `:117` (re-grepped per convention). Optionally correct when Task 3 rewrites that entry's Status. **[FOLD during Task 3.]**
- **M3 — `v["to"][0]["value"]` assertion** assumes single mainnet output; correct for `--from minikey= --to wif` (targets=[Wif], mainnet `5Kb8…`); field `value` matches `ConvertJsonEntry.value` (`convert.rs:451`). Not a defect.

---

## Verification ledger
- Leak+fix: `:1042` from_value redaction inside `if args.json` (`:1041-1046`); `:1069` checks outputs. `is_argv_secret_bearing()` @`:117-119`. ✓
- Non-JSON path (`:1062-1066`) doesn't echo input — `:1042` change is JSON-only. ✓
- No existing minikey test uses `--json`/`from_value` → nothing breaks. ✓
- `from_value`: `Option<&'a str>` + `#[serde(skip_serializing_if = "Option::is_none")]` (`:443-444`) → None omits → `Value` index returns Null → `is_null()` holds. ✓
- `SECRET_NODE_TYPES` (`secret_taxonomy.rs:76-85`) = 8 narrow tokens; ARGV = +minikey; strings match `as_str()`. ✓
- Parity-test shape: `ALL_NODE_TYPE_VARIANTS` macro (`:1701-1730`), existing test (`:1732-1754`), import (`:1680`); new test + import extension compile. Both methods `pub fn`. ✓
- GUI lockstep: an unreferenced new `pub const` can't break a snapshot pinning `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` (unchanged). Reasoning sound. ✓
- Version artifacts: `Cargo.toml:3`=0.34.4, `install.sh:32`=v0.34.4, `CHANGELOG.md:9`=[0.34.4]. ✓
- `serde_json`: regular `[dependencies]` (`Cargo.toml:43`), reachable in integration tests (cli_nostr.rs precedent); plan uses it fully-qualified. ✓

VERDICT: GREEN (0C/0I)

---

## Fold disposition (controller)
GREEN (0C/0I) → gate satisfied, implementation may proceed. M2 (FOLLOWUP `line 107`→`117` cite) folded during Task 3 Step 1. M1/M3 need no action. No plan-doc edit + no R0 re-dispatch (no Critical/Important).
