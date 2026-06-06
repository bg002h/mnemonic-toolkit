# R0 Architect Review — timestamp-default-zero (v0.47.3) — Round 2

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: a404185f7407f6336`). Confirms the round-1 folds. The one residual
> Minor (cosmetic §5d line numbers) was fixed after this review.

---

**VERDICT: 0 Critical / 0 Important (+ 1 Minor)**

**GREEN — cleared for Phase 1 (RED).**

---

## Folds Verified Correct

**I1 fold:** The SPEC §5d correctly identifies the test `export_wallet_timestamp_carries_default_value_now_as_string`, correctly states what must flip (`== "now"` → `== "0"`) and what stays GREEN (`is_string()`), and correctly describes why (`extract_default_value` for a `text`-kind flag returns `Value::String`). The fold action is complete and correct. One line-number citation in §5d is wrong (see new Minor below), but the fold action itself is unambiguous and correct.

**I2 fold:** SPEC §5e addresses the GUI D33 argv-suppression bug with: no toolkit-side change required (confirmed correct — the toolkit emits `"0"` correctly; the bug is entirely in GUI's hand-maintained schema mismatch), FOLLOWUP filed in Phase 3, two-release arc documented, companion entry in `mnemonic-gui` FOLLOWUPS noted. The `schema_mirror` confirmation in §5e ("it gates flag-NAMES only, not `default_value`; confirmed `schema_check.rs:98-104`") is accurate — the toolkit's `cli_gui_schema.rs` `GuiSchemaFlag` struct serializes `default_value` into the JSON schema output, but the GUI's `schema_mirror` integration test only deserializes `name: String` per the SPEC's sourced citation. There is no toolkit-side runtime artifact the GUI reads at runtime that can close this gap before a GUI pin bump — the FOLLOWUP + two-release arc is the only viable disposition. Fold is sufficient.

**M1, M2, M3 folds:** §5c now explicitly covers the `cli-help` golden regeneration. §8 correctly places `docs/manual-gui/` and `docs/technical-manual/` as out-of-scope with the GUI FOLLOWUP absorbing the `manual-gui` example. §2 now states `i64` (runtime-validated ≥0). All three are correctly folded.

**Restore test gap confirmed non-issue for Phase 1:** `cli_restore.rs:restore_format_bitcoin_core_payload` (line 514) does NOT assert on `timestamp` — it only checks `arr.len()` and `desc` content. `cli_restore_multisig_format.rs` similarly has no timestamp assertions. The Phase 1 RED cells for restore (#2 and #3 in §6) are new cells, correctly phased as writes-before-impl. No existing test goes spuriously RED from the restore flip.

**No new assertions missed by the SPEC's §5d scan:** the complete grep of all `.rs` test files against `"now"` confirms:
- `cli_export_wallet.rs:126` — default-path, must flip (listed in §5d).
- `cli_gui_schema_v5_extensions.rs:123` — `== "now"` assertion, must flip (listed in §5d as the I1 fix).
- `cli_nostr.rs:190,193` — explicit `--timestamp now` override path, stays `"now"` (listed in §5d as stays GREEN).
- `cli_import_wallet_bitcoin_core.rs:460` — comment text only, not an assertion (listed in §5d as no action).
- `cli_auto_repair.rs:358` — `"now"` in a D20 `position` field-set pin, completely unrelated to timestamp (harmless, no action needed).
- Fixture files `core-bip{44,84,86}-mainnet.json` — import inputs, confirmed safe via `roundtrip.rs:1189` (listed in §5d).

---

## Residual Finding

**M1 (new) — SPEC §5d line-number citations for `cli_gui_schema_v5_extensions.rs` are off by eight.**

The SPEC §5d says: "`:115` assertion `== 'now'`" and "`:116` the `is_string()` assertion." The actual file has the function **name** at `:115`, a comment at `:116-119`, `run_gui_schema()` call at `:120`, `find_sub` at `:121`, `find_flag` at `:122`, the `== "now"` assertion at **`:123`**, and `is_string()` at **`:124`**. The described action (flip `:115` → `"0"`, keep `is_string()`) is functionally correct and unambiguous to any implementer reading the actual file. The stale line numbers do not create an implementation trap — the test function name `export_wallet_timestamp_carries_default_value_now_as_string` is unique and self-identifying. No fix required before Phase 1; cosmetic.

> **Operator note:** the cosmetic line numbers were corrected in SPEC §5d (`:123`/`:124`) after this review. R0 is GREEN (0C/0I); cleared for Phase 1 (RED).
