# R0 Architect Review — timestamp-default-zero (v0.47.3) — Round 1

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: a8a1b71a9c42af0b5`). The reviewer had Read/Glob/Grep and verified
> claims against source. VERDICT below; fold actions tracked in the SPEC.

---

**VERDICT: 0 Critical / 2 Important (+ 3 Minor)**

---

## Important

**I1 — `cli_gui_schema_v5_extensions.rs:115-124` is missing from the SPEC's blast-radius list (toolkit CI will go RED at Phase 2)**

File: `crates/mnemonic-toolkit/tests/cli_gui_schema_v5_extensions.rs:115-124`

The test `export_wallet_timestamp_carries_default_value_now_as_string` asserts:
```
assert_eq!(ts["default_value"], "now");
assert!(ts["default_value"].is_string());
```
This calls `run_gui_schema()` and inspects the live binary's output. After flipping `export_wallet.rs:211` to `default_value = "0"`, `extract_default_value` returns `Value::String("0")` — so the assertion `== "now"` fails. The SPEC §5d lists this file in its scan list but does not list this test as one that flips. The SPEC's Phase 2 will go RED on CI until this is fixed.

**Fix:** Add to SPEC §5d: flip `cli_gui_schema_v5_extensions.rs:115` assertion from `"now"` to `"0"`. The `is_string()` assertion on line 116 remains GREEN (the value is still a string; only its content changes). Update the test name or add a comment noting the new expected value.

---

**I2 — GUI `default_value: Some("now")` in `mnemonic-gui/src/schema/mnemonic.rs:1044` becomes a D33 argv-suppression bug after the toolkit flip**

File: `mnemonic-gui/src/schema/mnemonic.rs:1044`

The GUI's hand-maintained flag schema for `export-wallet --timestamp` has `default_value: Some("now")`. The D33 default-suppression logic in `mnemonic-gui/src/form/invocation.rs::is_at_default` evaluates `(FlagKind::Timestamp, TimestampValue::Now) => default_str == "now"`. After the toolkit default flips to `0`, this condition still returns `true` — so when a user explicitly selects `TimestampValue::Now` in the GUI widget, the flag is suppressed from argv and the toolkit silently emits `timestamp: 0` instead of `"timestamp": "now"`. The `schema_mirror` integration test will NOT catch this because `GuiSchemaFlag` only deserializes `name: String` (confirmed at `schema_check.rs:98-104`).

This bug does not fire until the GUI bumps its pin to toolkit v0.47.3+. However, unlike purely cosmetic drift, this is a behavioral correctness regression: the user's explicit `--timestamp now` selection is silently discarded.

**Fix:** The SPEC must acknowledge this and file a FOLLOWUP against the GUI pin-bump cycle. Add to SPEC §5 (or a new §5e): "File FOLLOWUP `gui-timestamp-default-value-drift-v0.47.3` against the GUI repo. When the GUI pin-bumps to ≥v0.47.3, update `mnemonic-gui/src/schema/mnemonic.rs:1044` from `default_value: Some("now")` to `default_value: Some("0")`. Failing to do so causes D33 to silently suppress explicit `--timestamp now` from generated argv." The SPEC's lockstep section currently says `GUI schema_mirror NO` — this is accurate for the flag-name gate but the prose must note the D33 correctness dependency and the required paired FOLLOWUP.

---

## Minor

**M1 — `docs/manual/transcripts/cli-help/mnemonic-export-wallet.txt:66,68` will drift but is not CI-gated**

After the flip, this file will have stale `now (default)` (line 66) and `[default: now]` (line 68). The `verify-examples.sh` script excludes `*/cli-help/*` at line 67. The SPEC does not mention regenerating these. Either explicitly call out that these files are regenerated as part of Phase 2 (they should be, as they are transcript goldens for the help text), or note they are excluded from CI and regenerated at leisure. Low risk — not gated, purely cosmetic drift.

**M2 — `docs/manual-gui/src/40-mnemonic/45-export-wallet.md:422` and `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md:402,409` contain `"timestamp": "now"` examples**

These documentation files show the old default in worked examples. Neither is CI-gated (manual-gui and technical-manual are separate docs cadences). The SPEC's docs-sweep §5b covers `docs/manual/` but not these sibling docs trees. Call these out as out-of-scope (they are, per §8's "docs/technical-manual/ API harvest" deferral) or add a note to regenerate them; either way make the SPEC explicit so the implementer does not accidentally touch them.

**M3 — SPEC §2 describes `TimestampArg::Unix(n)` inner type as `u64?` (implicit in "json!(n)")**

The actual type at `wallet_export/mod.rs:142` is `Unix(i64)`. The `parse_timestamp` validator at `export_wallet.rs:316` rejects `n < 0`, so only non-negative values reach `to_json()`, and `json!(0i64)` = `0` in the JSON output. No behavioral consequence, but the SPEC description is imprecise. Fix: note `i64` (runtime-validated ≥0).

---

## Scope decision 1 — FULL SCOPE: RATIFIED

Flip all three emitters: `export_wallet.rs:211`, `restore.rs:608`, `restore.rs:661`. The evidence confirms the recon's structural finding: leaving restore at `TimestampArg::Now` recreates the very inconsistency the cycle removes. Genesis-rescan is the semantically correct default for a restore workflow. No `--timestamp` flag added to `restore` (scope creep; would trip GUI schema_mirror + manual mirror; `0` is the right fixed value and `restore` does not need override capability for this). Ratified as stated.

---

## SemVer decision 2 — PATCH (v0.47.2 → v0.47.3): RATIFIED

No clap-surface change: `--timestamp` flag name unchanged, free-string value-parser unchanged, no flag/variant added or removed. Explicit `--timestamp now` continues to work. The default-value change affects only the no-flag (default) code path. Pre-1.0 convention: default-value change on an existing flag with an available override is not breaking. PATCH is correct; MINOR is defensible but not required. Ratified as stated.

---

## Statically verified clean

The following SPEC claims were verified against source and found accurate:

- `export_wallet.rs:211` — `default_value = "now"` confirmed present; `:210` doc-comment confirmed. Flip site is correct.
- `restore.rs:608` — `timestamp: TimestampArg::Now` in `build_import_payload` confirmed. Flip site is correct.
- `restore.rs:661` — `timestamp: TimestampArg::Now` in `build_multisig_import_payload` confirmed. Flip site is correct.
- `nostr.rs:108` — `default_value = "0"` confirmed. Already correct; not in scope. No false positive.
- `TimestampArg` shape at `wallet_export/mod.rs:142-154` — `Now → json!("now")`, `Unix(i64) → json!(n)` confirmed. Type change `"now"` string → `0` number is real and test assertions must track it.
- Emitter inventory confirmed exhaustive — all `to_json()` calls trace to `bitcoin_core.rs` functions `format_bitcoin_core_importdescriptors` and `import_array_single`. No bundle/synthesize/bip85 timestamp paths exist.
- `cli_export_wallet.rs:126` — `.as_str().unwrap() == "now"` on default-path invocation confirmed. Must flip to `.as_u64().unwrap() == 0`.
- `cli_nostr.rs:164` — `v[0]["timestamp"] == 0` confirmed (stays GREEN). `:193` explicit `--timestamp now` → `"now"` confirmed (stays GREEN, explicit flag path).
- `cli_export_wallet_from_import_json.rs:61` — type-agnostic assertion `is_string() || is_number()` confirmed. Stays GREEN through type change.
- `cli_auto_repair.rs`, `cli_import_wallet_bitcoin_core.rs` — no `"now"` timestamp assertions found. No action needed.
- Recipe goldens: `recipe-1-bsms-to-bitcoin-core.out` and `recipe-5-specter-to-bitcoin-core.out` each contain exactly 2× `"timestamp": "now"`. Must regenerate. Recipes 2/3/4 do not emit bitcoin-core timestamps. Confirmed.
- `verify-examples.sh:67` — `*/cli-help/*` excluded from golden checks. Confirmed.
- `gui_schema.rs::classify_kind` — `--timestamp` (custom `parse_timestamp` value_parser) → falls through to kind `"text"`. Confirmed.
- `gui_schema.rs::extract_default_value` — `text` kind returns `Value::String(first.to_string())`. After flip, emits `"0"` string. Confirmed; `schema_mirror` won't fire (only checks flag names).
- `tests/fixtures/wallet_import/core-bip{44,84,86}-mainnet.json` — import inputs, not output goldens. `roundtrip.rs:1189` strips timestamp before comparison. Safe; do not regenerate.
- `41-mnemonic.md:707` — stale `now (default)` row confirmed. Must update.
- `37-wallet-export.md:36` — explicit `--timestamp now` in worked example. Still valid; no forced change.
- `37-wallet-export.md:329` — `--timestamp now` skips re-scan prose confirmed. Add note that default is now `0`.
- `41-mnemonic.md:2301` — nostr `--timestamp` row already `Default 0`. Do not touch. Confirmed.

---

The SPEC may not proceed to implementation. It must be folded to address I1 and I2 before Phase 1 (RED) begins:

1. Add `cli_gui_schema_v5_extensions.rs:115` (`"now"` → `"0"`) to §5d's explicit flip list.
2. Add a §5e (or append to §5b) stating the GUI `default_value` drift dependency, and add the FOLLOWUP slug `gui-timestamp-default-value-drift-v0.47.3` to the SPEC's lockstep section with an explicit note that the D33 suppression bug latently activates at the GUI's next pin-bump.
