# R0 Architect Review — `SPEC_technical_manual_g2_bare_ci_fix.md` — Round 1

**Reviewer:** opus architect (mandatory R0 gate). Reviewer had Read/Glob/Grep; parent persists.
**Grounded at:** `HEAD == origin/master == cdc0479`.
**Scope:** docs + CI only; no bump, no tag.

## Verdict: GREEN (0 Critical / 0 Important)

The guard logic is correct in both modes, the 12 qualifications are exact and segment-safe, the independent non-auth enumeration confirms the count is exactly 12 and glossary-only, the trigger and AUTHORING edits are accurate, and the disposition (no-bump/no-tag, no lockstep) is right. Three Minors below are wording/scope refinements, none blocking.

---

## Verified-correct (independently confirmed against source)

**Guard trace (the load-bearing logic) — `symbol-ref-check.py` resolve() lines 114-163:**
- `repos_with` is computed at line 143 INSIDE the `if not qualified:` block (line 142), and the collision return is at line 147 also inside that block. The SPEC inserts the guard "immediately after the collision return" — i.e. between line 147 and line 148, still inside `if not qualified:`. **`repos_with` IS in scope at the insertion point.** Correct and necessary placement (placing it outside the block would NameError).
- Traced `bundle.rs::build_unified_card` in `61-glossary.md` (non-auth, `auth=None`): no `crates/` regex match → `qualified=False` → `auth and not qualified` False → enters `if not qualified:` → `repos_with == ["toolkit"]` (only `src/cmd/bundle.rs` matches the `/bundle.rs` suffix; no codec `bundle.rs` exists) → collision predicate `len>=2` False → **new guard `not auth and repos_with == ["toolkit"]` True → returns `unqualified-toolkit` → FAIL.** Holds in BOTH siblings-present and siblings-absent (toolkit present in both).
- Renamed/absent toolkit file cited bare: `suffix_matches` → `[]` → `repos_with == []` → guard False → `shallowest` None → `skip:absent-sibling`. **The guard does NOT fire across a rename — so qualify-now is genuinely REQUIRED; guard alone insufficient.** SPEC's two-part mechanism reasoning is correct.
- Guard does NOT fire for: (a) **auth chapters** — verified ch54 cites `error.rs::`/`format.rs::` bare; `authoritative_repo("54-…")="toolkit"` → `not auth=False` → resolves via the `if auth and not qualified:` branch → `ok` (or `unresolved`→FAIL on rename), never reaches the guard. (b) **bare codec refs** — verified `identity.rs`, `canonicalize.rs`, `canonical_origin.rs`, `origin_path.rs`, `payload.rs`, `phrase.rs`, `key_card.rs`, `xpub_compact.rs`, `to_miniscript.rs` have NO toolkit file → `repos_with` excludes toolkit (local: `["md"]`/etc → False; bare: `[]` → False). (c) **40 qualified refs** — `is_repo_qualified=True` → `crates/` early-branch returns before the block. (d) **true cross-repo collision** — `len(repos_with)>=2` → existing `collision` return at line 147 fires before the new guard.
- After qualification, the `crates/` early-branch (lines 118-127) resolves `crates/mnemonic-toolkit/src/cmd/bundle.rs` via `CRATE_REPO["mnemonic-toolkit"]="toolkit"` → `os.path.isfile` → `(cand,"ok")` → scan loop `g2_checked += 1` → **per-`::`-segment `grep_word` still runs** (segment check NOT dropped). On rename → `isfile` False → `unresolved` → FAIL.

**The 12 tokens (Item 3) — all exact at cited lines in `61-glossary.md`, all segments exist:**

| line | bare token | target | segment(s) verified |
|---|---|---|---|
| 89 ×2 | `bundle.rs::build_unified_card` | `cmd/bundle.rs` | `build_unified_card` @ `cmd/bundle.rs:1033` |
| 113 | `verify_bundle.rs::MappingFailure` | `cmd/verify_bundle.rs` | `MappingFailure` @ `:1527` |
| 141 | `wallet_export/electrum.rs::ELECTRUM_SEED_VERSION_PIN` | `wallet_export/electrum.rs` | `ELECTRUM_SEED_VERSION_PIN` @ `:37` |
| 217 ×2 | `verify_bundle.rs::emit_md1_checks` / `emit_multisig_checks` | `cmd/verify_bundle.rs` | `emit_md1_checks` @ `:1417`; `emit_multisig_checks` @ `:1533` |
| 229 | `wallet_export/mod.rs::build_missing_fields_refusal` | `wallet_export/mod.rs` | `build_missing_fields_refusal` @ `:360` |
| 373 | `wallet_export/mod.rs::TaprootInternalKey` | `wallet_export/mod.rs` | `TaprootInternalKey` @ `:86` |
| 385 | `cmd/export_wallet.rs::ExportWalletArgs::timestamp` | `cmd/export_wallet.rs` | `ExportWalletArgs` @ `:140`; `timestamp` @ `:213` (`\btimestamp\b` whole-word) |
| 433 ×2 | `wallet_export/mod.rs::script_type_from_template` / `script_type_from_descriptor` | `wallet_export/mod.rs` | `:193` / `:213` |
| 453 | `verify_bundle.rs::MappingFailure::XpubNotInPolicy` | `cmd/verify_bundle.rs` | `MappingFailure` @ `:1527`; `XpubNotInPolicy` @ `:1530` |

- The two multi-segment anchors both pass: `ExportWalletArgs::timestamp` — both segments present; `MappingFailure::XpubNotInPolicy` — both present. **No self-RED.**
- Lines 229/385/433 each carry BOTH a `crates/`-qualified token AND a bare one; the SPEC qualifies only the bare one (`build_missing_fields_refusal` vs qualified `MissingField`; `ExportWalletArgs::timestamp` vs qualified `TimestampArg`; `script_type_from_*` vs qualified `WalletScriptType`). Confirmed exact.
- All 12 target file paths exist as written; `electrum.rs` exists at 3 toolkit paths so the `wallet_export/` subpath disambiguates (`suffix_matches("wallet_export/electrum.rs")` → only `src/wallet_export/electrum.rs`). Correct.

**Independent non-auth enumeration (the "do not trust exactly 12" check):**
- Non-auth chapters with any `.rs::` token: `10-foundations/13,14` (both fully `crates/md-codec/...`-qualified), `30-address-derivation/31,32,33`, `60-back-matter/61-glossary.md`. All other back-matter/disclaimer/frontmatter/build-banner: zero `.rs::` tokens.
- **Address-derivation bare refs are codec-only:** `to_miniscript.rs` (md-codec; no toolkit file) and `address_derivation.rs` (md-codec **tests** file, confirmed by the qualified `descriptor-mnemonic/crates/md-codec/tests/address_derivation.rs::` at ch33:57; no toolkit `src/` or `tests/` file by that name). Neither resolves to toolkit → guard will NOT RED them. Behavior unchanged.
- Toolkit-resolving bare/subpath refs in non-auth chapters = **exactly 12, all in `61-glossary.md`** (lines 89×2, 113, 141, 217×2, 229, 373, 385, 433×2, 453). **No toolkit-resolving bare ref exists outside the glossary** → Item 3 is NOT under-scoped; the post-qualify GREEN proof (§6.2) will not surface a surprise.
- Chapter 54 (authoritative toolkit) cites toolkit files bare (`error.rs::`, `format.rs::`) → `auth="toolkit"` → guard does NOT fire; resolves via the auth branch, FAILs on rename once the workflow fires. As SPEC claims.
- Glossary line 393 `mnemonic-toolkit/crates/mnemonic-toolkit/src/error.rs::ToolkitError` uses the double-prefix form but is already-qualified (`is_repo_qualified` True via `REPO_DIR_PREFIX "mnemonic-toolkit/"`; the `crates/` regex matches) → in the "40 qualified" set, correctly left untouched.

**Item 1 / Item 4 / Decisions / disposition:**
- Trigger insertion (after push.paths line 20, after pull_request.paths line 25) is syntactically valid YAML-list-item placement; adds no `path:` checkout override → WS-derivation invariant (4-up from `src/`) preserved.
- AUTHORING stale claim confirmed present at lines 222-223 verbatim ("**The technical manual has no CI workflow** — `make lint` is the gate"); SPEC's 4b correction is accurate. The "Colliding basenames" rule at 196-202 is currently scoped to *colliding* basenames; 4a's generalization to "any toolkit-resolving ref" matches the new guard's behavior.
- **Decision A is TRUE.** Verified `lint.sh`: steps 1-3,5,6 scan `$SRC_DIR` (docs) only; step 4 api-surface-coverage is `|| warn` (never sets `fail=1`); step 7 symbol-ref-check is the only blocking check a code diff can affect. A `crates/**`-triggered run **cannot newly-fail** for any reason other than symbol-ref-check.
- **Decision C honest:** codec-G2 genuinely needs absent siblings; bare CI can't do symbol-level codec-G2; documented in 3 places. Correct characterization.
- **Disposition correct:** no clap-flag/CLI/codec surface touched → no GUI `schema_mirror`, no manual-mirror, no sibling-codec companion. No-bump/no-tag right (binary byte-identical).

---

## Critical
None.

## Important
None.

## Minor

**M1 — Decision A wording slightly imprecise re: api-surface-coverage.** The SPEC (line 38) and Item-4b say the other 6 checks "scan docs text only." That's exact for 5 of them, but api-surface-coverage *reads* `lib.rs`/`format.rs` source (per the workflow comment lines 4-6, which is more precise). The CONCLUSION (a code PR cannot newly-fail) is still correct, but for the *stronger* reason that api-surface-coverage is **warning-only** (`lint.sh:82` `|| warn`), not because it ignores source. Suggest tightening the Decision-A justification to "the other 6 scan docs text only (api-surface-coverage reads lib.rs/format.rs but is warning-only) → cannot newly-fail." Non-blocking; the workflow comment already states it correctly.

**M2 — `TimestampArg` glossary prose is stale, and this cycle edits that very line.** The glossary entry at lines 383-385 describes `TimestampArg` as `Now (renders to "now")` / `Unix(i64)` with `default now`, and the bare token names `ExportWalletArgs::timestamp`. Source is now `pub timestamp: TimestampArgValue` (`cmd/export_wallet.rs:213`) with **default `0`/genesis-rescan** (`:210-212`), per the shipped v0.47.3 timestamp-default-zero cycle (in MEMORY). G2 is unaffected (`timestamp` segment exists; `TimestampArg` still exists in `wallet_export/mod.rs`), so this is NOT a blocker for THIS cycle. But since Item 3 touches line 385, flag it: either (a) note explicitly in the SPEC that the prose-staleness on this line is out-of-scope (qualify-the-path-only), or (b) opportunistically correct the default while editing the line. Recommend (a) to keep the cycle minimal and avoid scope creep; file a one-line FOLLOWUP for the stale `TimestampArg` default/type prose. (This is exactly the "no-sampling, prose blocks not transcript-gated" class burned in prior technical-manual cycles — surfacing it here, not chasing it.)

**M3 — §6.2 GREEN-proof should assert the address-derivation bare codec refs still `skip` (regression guard), not just "no new toolkit ref surfaces."** My enumeration confirms `to_miniscript.rs`/`address_derivation.rs` resolve codec-only, so the guard leaves them as today. But the acceptance run in §6.2 only asserts overall GREEN. Suggest the toolkit-only run additionally eyeball the skip-count delta (should be unchanged for the address-derivation chapters) so a future basename collision (e.g. a new toolkit `to_miniscript.rs`) that would silently flip one of them into the guard is caught. Minor — the planted-rename proof (§6.5) + overall GREEN already cover the substantive risk.

---

**Gate result: GREEN (0C/0I). Implementation may begin.** Per CLAUDE.md the reviewer-loop continues after every fold — re-dispatch this R0 after folding the Minors (or after recording the M2 disposition decision), since folds can introduce drift. The Minors are non-blocking and do not hold the gate.
