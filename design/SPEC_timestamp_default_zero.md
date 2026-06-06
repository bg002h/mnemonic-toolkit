# SPEC — `--timestamp` default `0` everywhere (v0.47.3)

**FOLLOWUPs:** `export-wallet-timestamp-default-zero` + `timestamp-zero-default-docs-sweep` (paired, ship together).
**Source SHA (origin/master at write time):** `afeb967`
**Cycle type:** PATCH (`v0.47.2` → `v0.47.3`). Toolkit-only.
**Recon:** `cycle-prep-recon-timestamp-default-zero.md` (both slugs ACCURATE in content; `export_wallet.rs` citation drifted `:117`→`:211`; recon surfaced the `restore.rs` third-emitter gap).
**Locksteps:** manual mirror **YES** (the docs-sweep slug). GUI `schema_mirror` **NO** (no flag-NAME/value-enum change). verify-examples **YES** (2 transcripts regenerate). Sibling-codec **NONE**. **GUI cross-repo (NOT gated by `schema_mirror`):** the default-value flip latently breaks the GUI's D33 argv-suppression for an explicit `--timestamp now` selection — file FOLLOWUP `gui-timestamp-default-value-drift-v0.47.3` (two-release arc; see §5e).

---

## 1. Problem

The toolkit emits Bitcoin Core `importdescriptors` recipes with a `timestamp` rescan anchor. The default is inconsistent across emitters:

| Emitter | `--timestamp` flag? | Default today | Emitted JSON |
|---|---|---|---|
| `nostr --import` | yes | `0` | `"timestamp": 0` |
| `export-wallet` | yes | **`now`** | `"timestamp": "now"` |
| `restore --format` (single-sig) | **no** (hardcoded) | **`Now`** | `"timestamp": "now"` |
| `restore --md1 --format` (multisig) | **no** (hardcoded) | **`Now`** | `"timestamp": "now"` |

A `now` anchor tells Bitcoin Core to watch **going forward** (skips the historical rescan); a `0` anchor rescans **from genesis** (discovers an existing key's historical funds). For a recovery/restore or watch-only-import workflow the user almost always wants `0` — they are importing keys that may already have a transaction history. The user requested `0` as the consistent default everywhere (`export-wallet-timestamp-default-zero`).

## 2. Behavior model (verified)

`TimestampArg` (`wallet_export/mod.rs:144-154`):
- `Now` → `to_json()` = `json!("now")` — a JSON **string**.
- `Unix(i64)` (inner type `i64`, runtime-validated ≥0 by `parse_timestamp`, which rejects `n < 0` at `export_wallet.rs:316`) → `json!(n)` — a JSON **number**. `Unix(0)` → `0`.
- `parse_timestamp("now")` → `Now`; `parse_timestamp("0")` → `Unix(0)` (`export_wallet.rs:310-320`).

So flipping a default from `"now"` → `"0"` changes the emitted field from the string `"now"` to the number `0`. **Test assertions must track the type change** (`.as_str() == "now"` → `.as_u64() == 0`). `0` is a valid Bitcoin Core `timestamp` (integer unix seconds; `nostr` already emits it and shipped in v0.34.2).

## 3. Scope decision — FULL SCOPE (R0 to ratify)

**Decision: flip ALL non-`0` emitters to `0`, including `restore`'s two hardcoded sites. Do NOT add a `--timestamp` flag to `restore`.**

- `export_wallet.rs:211` — `default_value = "now"` → `"0"`. (Explicit `--timestamp now` remains available for users who want watch-forward.)
- `restore.rs:608` (`build_import_payload`, single-sig) — `timestamp: TimestampArg::Now` → `TimestampArg::Unix(0)`.
- `restore.rs:661` (`build_multisig_import_payload`, multisig, v0.45.0) — `timestamp: TimestampArg::Now` → `TimestampArg::Unix(0)`.

**Rationale:** (a) the FOLLOWUP's stated goal is "`0` everywhere" — leaving `restore` at `Now` re-creates the inconsistency the cycle removes; (b) genesis-rescan is the semantically *correct* default for a recovery workflow; (c) `restore` has no `--timestamp` flag and adding one is scope creep (would trip GUI `schema_mirror` + manual mirror) and unnecessary — `0` is the right fixed value. **Rejected alternative:** export-wallet-only scope (leaves `restore` emitting `"now"` — inconsistent, fails the FOLLOWUP intent).

## 4. SemVer — PATCH (R0 to ratify)

No clap-surface change: `--timestamp` keeps its name and free-string value-parser; no flag/value-enum added or removed. Pre-1.0 convention (`0.X` = breaking axis): a default-*value* change is **not breaking** (explicit values unchanged; the flag still exists; export-wallet users can opt back to `now`). → **PATCH** `v0.47.2` → `v0.47.3`, consistent with recent non-surface cycles. (MINOR is defensible if a default-output-semantics change is deemed release-significant; R0 ratifies.)

## 5. Changes

### 5a. Source (3 sites)
- `export_wallet.rs:211` — `default_value = "now"` → `"0"`; update the `:210` doc-comment "`now` (default)" → "`0` (default; rescan from genesis)".
- `restore.rs:608` + `:661` — `TimestampArg::Now` → `TimestampArg::Unix(0)`.

### 5b. Docs (manual mirror — the `timestamp-zero-default-docs-sweep` slug)
- `docs/manual/src/40-cli-reference/41-mnemonic.md:707` — export-wallet `--timestamp` row: "`now` (default) or unix seconds" → "`0` (default; rescan from genesis) or `now` or unix seconds".
- `docs/manual/src/30-workflows/37-wallet-export.md:329` — the "`--timestamp now` skips re-scan" prose: add that the default is now `0` (genesis rescan); keep the explanation of what `now` does.
- `docs/manual/src/30-workflows/37-wallet-export.md:36` — worked-example invocation currently shows explicit `--timestamp now`; leave as an explicit example OR drop the flag to show the default — R0/impl decides (cosmetic).
- **Do NOT touch** `41-mnemonic.md:2301` — that is the **nostr** row, already correctly `Default 0`.
- If `restore` doc rows imply a `now` anchor, add a one-line note that `restore --format` emits `timestamp: 0`. (Grep `41-mnemonic.md` restore section at impl time.)

### 5c. Transcript goldens (verify-examples coupling + cli-help)
- `docs/manual/transcripts/cross-format-recipes/recipe-1-bsms-to-bitcoin-core.out` — 2× `"timestamp": "now"` → `"timestamp": 0`.
- `docs/manual/transcripts/cross-format-recipes/recipe-5-specter-to-bitcoin-core.out` — 2× `"timestamp": "now"` → `"timestamp": 0`.
- Regenerate by re-running each `.cmd` against the rebuilt binary; confirm `make -C docs/manual verify-examples` GREEN. (Recipes 2/3/4 do not emit bitcoin-core timestamps — unaffected.)
- **(R0 M1) `docs/manual/transcripts/cli-help/mnemonic-export-wallet.txt:66,68`** — the `--help` golden carries `now (default)` (`:66`) + `[default: now]` (`:68`), which drift to `0` after the doc-comment + `default_value` change. **NOT CI-gated** (`verify-examples.sh:67` excludes `*/cli-help/*`), but regenerate it in Phase 2 for cleanliness (re-capture `mnemonic export-wallet --help`).

### 5d. Test assertions (default-path only — discriminate from explicit `--timestamp now`)
- `cli_export_wallet.rs:126` — default-path assertion `arr[0]["timestamp"].as_str().unwrap() == "now"` → `.as_u64().unwrap() == 0` (+ any sibling change-entry assertion).
- **(R0 I1) `cli_gui_schema_v5_extensions.rs`** — the test `export_wallet_timestamp_carries_default_value_now_as_string` (fn name at `:115`) asserts `ts["default_value"] == "now"` (`:123`) + `is_string()` (`:124`). After the `default_value="0"` flip, `gui-schema`'s `extract_default_value` for a `text`-kind flag returns `Value::String("0")`. **Flip the `:123` assertion `"now"` → `"0"`; the `:124` `is_string()` assertion STAYS GREEN** (still a string, content `"0"`). Rename the test (e.g. `…carries_default_value_zero_as_string`) + update the explanatory comment `:116-119`. **This is the toolkit-CI-RED-at-Phase-2 trap — do not miss it.** (R0-r2 M1: line numbers corrected from the round-1 fold's `:115/:116`.)
- Scan `cli_auto_repair.rs`, `cli_nostr.rs`, `cli_import_wallet_bitcoin_core.rs` for `"now"` assertions: R0 confirmed `cli_auto_repair.rs` + `cli_import_wallet_bitcoin_core.rs` have **no** `"now"` timestamp assertions; `cli_nostr.rs:164` asserts `== 0` (stays GREEN), `:193` is an explicit `--timestamp now` path (stays `"now"`, GREEN); `cli_export_wallet_from_import_json.rs:61` is type-agnostic (`is_string() || is_number()`, stays GREEN). **Keep** any explicit-`--timestamp now` path; **flip** any default-path one.
- **Fixtures `tests/fixtures/wallet_import/core-bip{44,84,86}-mainnet.json` stay AS-IS** — they are import *inputs*; `import-wallet` must continue to accept the historical `"now"` form (R0 confirmed `roundtrip.rs:1189` strips `timestamp` before comparison, so no round-trip golden pins their output).

### 5e. GUI cross-repo dependency (R0 I2 — latent D33 argv-suppression bug)
**No toolkit-side change**, but a paired-FOLLOWUP MUST be filed. The GUI's hand-maintained schema (`mnemonic-gui/src/schema/mnemonic.rs:1044`) declares export-wallet's `--timestamp` as `FlagKind::Timestamp, default_value: Some("now")`, and the D33 default-suppression logic (`mnemonic-gui/src/form/invocation.rs:78`) suppresses the flag from argv when `TimestampValue::Now` matches `default_str == "now"`. After the toolkit default flips to `0`, a GUI user who *explicitly* selects `Now` would have `--timestamp now` **silently suppressed** → toolkit emits `0` instead → the user's explicit choice is discarded. **`schema_mirror` will NOT catch this** (it gates flag-NAMES only, not `default_value`; confirmed `schema_check.rs:98-104`). The bug is **latent until the GUI bumps its toolkit pin to ≥v0.47.3**.

**Action (Phase 3):** file FOLLOWUP **`gui-timestamp-default-value-drift-v0.47.3`** in `design/FOLLOWUPS.md` (with a companion entry in `mnemonic-gui`'s FOLLOWUPS per the cross-repo mirror convention). Its fix, to land at the next GUI pin-bump: update `mnemonic-gui/src/schema/mnemonic.rs:1044` `default_value: Some("now")` → `Some("0")` (and verify the GUI's `is_at_default`/widget-init handles the new default correctly), plus regenerate the manual-gui example (R0 M2: `docs/manual-gui/src/40-mnemonic/45-export-wallet.md:422`). The toolkit cycle cannot fix this directly — the GUI consumes the toolkit by git tag, so v0.47.3 must ship before the GUI can bump; this is an inherently two-release arc.

## 6. Phasing / TDD

- **Phase 1 (RED):** add discriminating cells —
  1. `export-wallet` default-path (no `--timestamp`) → assert `"timestamp": 0` (number). RED today (`"now"`).
  2. `restore --format bitcoin-core` (single-sig) default → assert `timestamp == 0`. RED today.
  3. `restore --md1 --format bitcoin-core` (multisig) default → assert `timestamp == 0`. RED today.
  4. **Guard (stays GREEN):** `export-wallet --timestamp now` explicit → still `"now"` (string) — proves the flip is default-only, not a removal.
- **Phase 2 (GREEN):** apply 5a; regenerate 5c transcripts; update 5d default-path assertions; apply 5b docs. Full suite + clippy + `make -C docs/manual audit` GREEN. Per-phase opus review (persist verbatim before fold).
- **Phase 3 (release):** CHANGELOG `[0.47.3]`; version bump ×5 (Cargo.toml/lock + 2 READMEs + `scripts/install.sh` self-pin); flip both resolved FOLLOWUPs; **file the new FOLLOWUP `gui-timestamp-default-value-drift-v0.47.3`** (§5e) + its `mnemonic-gui` companion.
- **Phase 4 (ship):** clean tree → ff-merge → tag `mnemonic-toolkit-v0.47.3` → push → watch CI (`rust`, `manual` (fires — docs + transcripts changed), `install-pin-check`, `sibling-pin-check`).

## 7. R0 decisions (RATIFIED round 1)
1. **Scope:** **FULL** — flip `export_wallet.rs:211` + `restore.rs:608` + `restore.rs:661`. ✅ R0-ratified.
2. **SemVer:** **PATCH** v0.47.2 → v0.47.3. ✅ R0-ratified.
3. **No `--timestamp` flag added to `restore`.** ✅ R0-ratified (scope creep; `0` is the right fixed value).
4. **Bitcoin Core acceptance:** `0` (number) is a valid `timestamp` (genesis rescan; `nostr` precedent). ✅ R0 confirmed emitter inventory exhaustive (all `to_json()` → `bitcoin_core.rs`; no bundle/synthesize/bip85 timestamp path).

## 8. Out of scope
- Adding a `--timestamp` flag to `restore`.
- Changing the `range` `(0, 999)` or `bitcoin_core_version` defaults.
- The `docs/technical-manual/` API harvest (separate cadence; tracked by `api-harvest-drift-on-synthesize-descriptor-signature`). **(R0 M2)** specifically `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md:402,409` (`"timestamp": "now"` examples) — NOT CI-gated, separate cadence, leave untouched this cycle.
- **(R0 M2) `docs/manual-gui/src/40-mnemonic/45-export-wallet.md:422`** (`"timestamp": "now"` example) — the GUI manual rides the GUI cadence; folded into the `gui-timestamp-default-value-drift-v0.47.3` FOLLOWUP (§5e), not this toolkit cycle.
- The GUI-side `default_value` / D33 fix itself (separate repo, two-release arc; §5e files the FOLLOWUP).
