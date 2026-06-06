# cycle-prep recon — 2026-06-06 — export-wallet-timestamp-default-zero + timestamp-zero-default-docs-sweep

**Origin/master SHA at recon time:** `afeb967`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** working artifacts only (`cycle-prep-recon-*.md`, `CONTINUITY.md`, `feature-coverage-survey-*.md`, `.claude/`) — no tracked-tree modifications.

Slug(s) verified: `export-wallet-timestamp-default-zero`, `timestamp-zero-default-docs-sweep`. Both ACCURATE in content; the source citation has **DRIFTED** (`:117`→`:211`) and the recon surfaces a **third emitter** the FOLLOWUP's "Where" omits — `restore.rs` hardcodes `TimestampArg::Now` at 2 sites (the crux scope decision for R0).

---

## Per-slug verification

### `export-wallet-timestamp-default-zero`
- **WHAT (from FOLLOWUPS.md):** `export-wallet`'s `--timestamp` defaults to `"now"` while `nostr`'s defaults to `"0"`. User wants `0` as the consistent default everywhere. Behavior change to the emitted `importdescriptors` rescan anchor (genesis-rescan vs watch-forward).
- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:117` (`default_value = "now"`) — **DRIFTED-by-94** → now at **`:211`**: `#[arg(long, default_value = "now", value_parser = parse_timestamp)]` (doc-comment at `:210` "`now` (default) or unix seconds"). Content ACCURATE.
  - "`nostr --timestamp` defaults to `0`" — **ACCURATE**: `nostr.rs:108` `default_value = "0"`.
  - Behavior model — **ACCURATE/clarified:** `TimestampArg` (`wallet_export/mod.rs:144`) has variants `Now` (`to_json()` → `json!("now")`, a STRING) and `Unix(n)` (`→ json!(n)`, a NUMBER). `parse_timestamp("now")→Now`, `parse_timestamp("0")→Unix(0)`. So changing the default to `"0"` flips emitted `"timestamp": "now"` → `"timestamp": 0` (string→number type change in the JSON — relevant for test assertions: `.as_str()=="now"` becomes `.as_u64()==0`).
- **Action for brainstorm spec:** Change `export_wallet.rs:211` `default_value = "now"` → `"0"`. **Crux scope decision (resolve at brainstorm + ratify at R0): also flip `restore.rs`'s 2 hardcoded `TimestampArg::Now`?** See cross-cutting #1 — recommend YES (`Unix(0)`), NO new flag. Cite source SHA `afeb967`.

### `timestamp-zero-default-docs-sweep`
- **WHAT (from FOLLOWUPS.md):** Once the default lands, update all docs that state/imply `--timestamp` defaults to `now`.
- **Citations:**
  - "`docs/manual/` (any chapter implying `--timestamp` defaults to `now`)" — **ACCURATE, enumerated:**
    - `docs/manual/src/40-cli-reference/41-mnemonic.md:707` — export-wallet CLI row: `` `now` (default) or unix seconds `` — **STALE, must update** to `0` (default).
    - `docs/manual/src/30-workflows/37-wallet-export.md:36` — example invocation `--timestamp now` (explicit; still valid — keep, or change the worked example to default).
    - `docs/manual/src/30-workflows/37-wallet-export.md:329` — prose "`--timestamp now` skips re-scan (assumes the wallet…)" — explanatory, keep; consider adding "default is now `0` (genesis rescan)".
  - `41-mnemonic.md:2301` "Default `0`" — this is the **nostr** `--timestamp` row (nostr `--import` section, `--pubkey`/`--secret`), already correct — **NOT in scope** (do not touch).
  - "any SPEC mentioning the timestamp default" — grep `design/SPEC*`/`design/` for "timestamp" + "now" at impl time; the live SPEC ref is `wallet_export/mod.rs:142` doc-comment "SPEC §5 timestamp argument".
- **Action for brainstorm spec:** Update `41-mnemonic.md:707` (the one stale default claim) + decide whether to retouch `37-wallet-export.md:36/329`. If restore is flipped (scope decision), add a one-line note that `restore --format` emits `timestamp: 0`. Cite source SHA `afeb967`.

---

## Cross-cutting observations

1. **STRUCTURAL GAP — the FOLLOWUP's "everywhere" omits a third emitter.** `restore.rs` hardcodes `timestamp: TimestampArg::Now` at **two** sites (`~:608` single-sig `--format` path via `emit_payload`; `~:661` `build_multisig_import_payload`, the v0.45.0 multisig path). `restore` has **no `--timestamp` flag** — `Now` is its only behavior. Leaving these at `Now` while flipping export-wallet to `0` would re-create the very inconsistency the cycle removes (restore would emit `"now"`, export-wallet `0`). **Recommendation: full scope** — flip both restore sites to `TimestampArg::Unix(0)`; do NOT add a `--timestamp` flag to restore (scope creep → would trip GUI `schema_mirror` + manual mirror). Semantic bonus: genesis-rescan (`0`) is the *correct* default for a recovery/restore workflow (you want to discover historical funds). This is the one decision R0 must ratify.
2. **verify-examples COUPLING (confirmed live).** `recipe-1-bsms-to-bitcoin-core.cmd` and `recipe-5-specter-to-bitcoin-core.cmd` both pipe `… | export-wallet --from-import-json - --format bitcoin-core` with **no explicit `--timestamp`** → they hit the default. Their `.out` goldens carry `"timestamp": "now"` (2 lines each) → these will flip to `0`; both `.out` files must be regenerated or `make verify-examples` (CI `manual` workflow) goes RED. (Recipes 2/3/4 do not emit bitcoin-core timestamps — unaffected.)
3. **Test-assertion blast radius (default-path only).** Files asserting on `"now"`: `cli_export_wallet.rs:126` (default-path, confirmed — flips to `0`, type `.as_str()`→`.as_u64()`), plus scan `cli_auto_repair.rs`, `cli_gui_schema_v5_extensions.rs`, `cli_nostr.rs`, `cli_import_wallet_bitcoin_core.rs` — **discriminate** explicit `--timestamp now` invocations (stay `"now"`) from default-path ones (flip to `0`) at impl time.
4. **Fixtures are import INPUTS, not goldens.** `tests/fixtures/wallet_import/core-bip{44,84,86}-mainnet.json` contain `"timestamp": "now"` — they are import-wallet *inputs*; import must continue to ACCEPT the historical `"now"` form. **Do NOT regenerate** unless a round-trip re-export comparison pins their output (none found). They also serve as a regression guard that `"now"` still parses.
5. **SemVer ambiguity (FOLLOWUP explicitly flags "warrants its own SemVer call").** No clap-surface change (flag name/value-enum unchanged; `--timestamp` is a free string via `parse_timestamp`). Pre-1.0 convention: the `0.X` axis is breaking-change; a default-*value* change is not breaking (explicit `--timestamp now` still works; no flag removed). → **PATCH** (v0.47.2 → v0.47.3) is the natural call, consistent with recent non-surface cycles. MINOR is defensible if the team treats a default-output-semantics change as release-significant. **Ratify at R0.**

---

## Recommended brainstorm-session scope

- **ONE coherent cycle**, both slugs together (the docs-sweep is the mechanical follow-on to the behavior change — they ship in lockstep). **SemVer: PATCH** v0.47.2 → **v0.47.3** (no surface change; ratify at R0 — MINOR defensible).
- **Size: medium-mechanical.** Source: 1 default change (`export_wallet.rs:211`) + 2 restore hardcodes (`restore.rs` ×2, *if full scope*) = ~3 LOC. Docs: 1 stale row (`41-mnemonic.md:707`) + optional `37-wallet-export.md` retouch. Goldens: 2 transcript `.out` regenerate. Tests: ~1 confirmed default-path assertion flip (`cli_export_wallet.rs:126`) + a discriminating scan of 4 more test files.
- **Locksteps:** **manual mirror — YES** (`41-mnemonic.md:707`, the docs-sweep slug IS the manual). **GUI `schema_mirror` — NO** (no flag-NAME/value-enum change; default value is not gated). **verify-examples — YES** (2 transcripts regenerate; the `manual` CI workflow fires). **Sibling-codec — NONE.**
- **Phasing:** Phase 1 RED = a default-path export-wallet cell asserting `"timestamp": 0` (number) + (if full scope) a `restore --format` cell asserting `timestamp: 0` — both RED against current `now`. Phase 2 GREEN: flip default + restore hardcodes, regenerate the 2 transcripts, update default-path test assertions, manual row. Per-phase opus review. **R0 MUST decide:** (i) full scope (flip restore's 2 `Now`) vs export-wallet-only; (ii) the SemVer call; (iii) whether to retouch the `37-wallet-export.md` worked example or only the stale `:707` default claim.
- **No inter-slug ordering issue** — docs-sweep folds into the same PRs as the behavior change.

