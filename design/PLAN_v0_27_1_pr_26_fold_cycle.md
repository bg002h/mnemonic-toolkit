# PLAN — mnemonic-toolkit v0.27.1 (PR-#26 post-merge fold cycle)

**Status:** R2 — opus R2 verdict GREEN (1 trivial Minor on Phase 6 CHANGELOG-stub wording folded inline). Plan-doc ready for execution post-PR-#27-merge.
**Scope:** 6 FOLLOWUPs filed against PR #26 post-merge comprehensive review.
**Pre-cycle baseline:** master at the post-merge commit of PR #27 (currently OPEN; cycle blocks on that merge per architect advisory).
**Authorship:** single-instance.
**Target tag:** `mnemonic-toolkit-v0.27.1`. GitHub release with patch-level CHANGELOG.

---

## §1. Context

PR #26 (v0.26.0 release squash; merged 2026-05-18 at `66c8a56`) bundled three features in one squash (compare-cost + xpub-search + import-wallet, +20,728/-60 LOC across 96 files). Post-merge — during PR #27's `pr-review-toolkit:review-pr` close-out — a 5-agent retrospective audit ran against the v0.26.0 changes:

- Phase 1: `code-reviewer` solo
- Phase 2: `silent-failure-hunter` + `comment-analyzer` + `type-design-analyzer` + `pr-test-analyzer` (parallel)

Findings: **2 Critical, 19 Important, 6 recurring type-design anti-patterns, 5 test-quality issues, 8 positive observations.** Full report at `design/agent-reports/pr-26-post-merge-comprehensive-review.md`. No ship-blocker — PR #26 was correctly mergeable.

User-locked disposition (2026-05-19): **fold all 6 FOLLOWUPs in one v0.27.1 patch cycle.** Cycle is toolkit-only (no sibling lockstep; mnemonic-gui pin not bumped). Architect (opus) confirmed branch base: off `master` after PR #27 squash-merges.

---

## §2. The six FOLLOWUPs

| # | FOLLOWUP slug | Sub-findings folded | Tier | Cells (new) | LOC est | Phase |
|---|---|---|---|---|---|---|
| 1 | `pr-26-roundtrip-warning-suppression` | C1 + I7 | bugfix | 3-4 | ~50 | 1 |
| 2 | `pr-26-shape-mismatch-silent-defaults` | I4 + I5 + I6 | bugfix | 6-8 | ~80 | 2 |
| 3 | `pr-26-comment-rot-fold` | C2 + I8-I11 | doc | 0 (lint surface only) | ~30 | 3 |
| 4 | `pr-26-test-coverage-gap-fold` | I12-I19 | coverage | 8 | ~250 | 4 |
| 5 | `pr-26-type-design-anti-pattern-sweep` | I21 + Phase 5a/c partial I20 (5a is API-discipline scaffolding only — type-level fix blocked on wire-shape evolution per Q5b) | refactor | 5 (drift) | ~180 | 5 |
| 6 | `compare-cost-single-leaf-tr-input` | (filed only — defer impl) | feature | 0 | 0 | — (deferred) |

**Total budget (R2):** ~590 LOC (Phase 1 ~50 + Phase 2 ~80 + Phase 3 ~30 + Phase 4 ~250 + Phase 5 ~180) + **22 new test cells** (Phase 1: 3, Phase 2: 6, Phase 4: 8, Phase 5: 5) + ~6-8 captured fixtures + 2 SPEC docs-only commits (Q1a + Q3a-verified-leave-as-is) + 1 new FOLLOWUP slug filed at cycle close per Q5b.

### §2.1 Phase 5b scoping decision

Item #6 (`compare-cost-single-leaf-tr-input`) is **filed as a v0.27.1 FOLLOWUP** because its slug was already cited in user-visible error text + 2 source comments — filing it closes the citation-without-slug loop. Whether to **implement** it in v0.27.1 is a separate question:

- **Argument to include (Phase 5b):** the slug is user-visible; users hitting `cost/mod.rs:75` error and grep'ing for the slug now find a real entry but no code. If we ship v0.27.1 without implementing #6, users see "open, deferred" — clean. The implementation itself is ~80-120 LOC for single-leaf `tr()` only (multi-leaf TapTree is genuinely deferred to a separate cycle).
- **Argument to defer:** taproot cost-comparison semantics need a SPEC anchor before implementing. The existing `cost/strip.rs` and `cost/translate.rs` are SPEC-anchored to `SPEC_compare_cost_v0_26_0.md` which does NOT cover taproot inputs. Adding tr() requires a SPEC amendment — out-of-scope for a patch cycle.

**Lock (subject to R0 challenge):** **Defer #6 implementation to a separate cycle.** v0.27.1 ships only the FOLLOWUP filing (already done in `53a1bf6`). v0.27.1 cycle close drops Phase 5b.

---

## §3. Locked design questions

### Q1 (Phase 1 — roundtrip warning shape).

**`emit_roundtrip_stderr_warning` error-arm template + JSON-mode envelope shape.**

- **Lock:** Stderr arm emits `"warning: import-wallet: roundtrip check skipped: canonicalize_bitcoin_core failed: {e}"` (where `{e}` is the typed `ToolkitError` Display form). Symmetric branch for `canonicalize_bsms` with `bsms` substituted. UTF-8 case: separate notice on stderr `"notice: import-wallet: blob is not UTF-8; roundtrip check uses lossy decode"` followed by lossy-decode'd canonicalize attempt — if THAT fails, fall through to the same warning template.
- **JSON-mode envelope shape (R1 correction):** the existing `roundtrip` object's `status` field is **already SPEC-locked** as ALWAYS PRESENT with the closed enum `"ok" | "blocked_no_emitter" | "canonicalize_failed"` per `design/SPEC_wallet_import_v0_26_0.md:46` + `design/SPEC_mnemonic_toolkit_v0_5.md:716`. R0 caught this — the plan-doc previously implied `status` was being newly-introduced; it is not.
  ```json
  "roundtrip": { "byte_exact": false, "semantic_match": false, "diff": null, "status": "canonicalize_failed", "error": "<message>" }
  ```
  The **only genuinely-additive field** is `error: String`, scoped to the `status == "canonicalize_failed"` branch. v0.26.0 / v0.27.0 consumers parsing `byte_exact` / `semantic_match` / `status` are unaffected; consumers learning to read `error` see a richer payload only when `status == "canonicalize_failed"`. No SemVer concern.
- **See Q1a — Phase 1 must amend SPEC §2.2 ahead of code commit.**

### Q1a (NEW R1 — Phase 1 SPEC amendment-first commit).

**Amend `SPEC_wallet_import_v0_26_0.md` §2.2 + `SPEC_mnemonic_toolkit_v0_5.md` §X (whichever §X declares the `roundtrip` envelope shape) to add the new `error: String` field BEFORE the Phase 1 code commit.**

- **Lock:** Phase 1's first commit is `docs(spec): roundtrip.error field for canonicalize_failed branch (v0.27.1 Phase 1)`. The amendment specifies (a) field name `error`, (b) type `String` (typed `ToolkitError.to_string()` output), (c) scope: present iff `status == "canonicalize_failed"`, omitted otherwise, (d) example payload. Mirrors v0.26.0 cycle's amendment-first discipline (`IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §7.0 amendments preceded source changes).
- **Why amendment-first:** the envelope is the consumer wire-shape contract. Shipping an undocumented field — even an additive one in an existing closed-enum branch — creates the "what does this field mean" ambiguity for downstream consumers (mnemonic-gui at the next pin bump, BIP-322-style upstream parsers, etc.).

### Q2 (Phase 2 — shape-mismatch site rule).

**Distinguish "absent" from "shape-wrong" at three sites: `active`/`internal`, `mk1.origin_fingerprint`, `extract_threshold`.**

- **Lock — `active`/`internal`:** Refactor pattern `.and_then(|v| v.as_bool()).unwrap_or(false)` to:
  ```rust
  match eobj.get("active") {
      None => false,
      Some(serde_json::Value::Bool(b)) => *b,
      Some(other) => return Err(ToolkitError::ImportWalletParse(format!(
          "import-wallet: bitcoin-core: parse error: 'active' must be boolean, got {other}"
      ))),
  }
  ```
  Mirror for `internal`. Pattern matches `parse_range_field` shape-strictness precedent.
- **Lock — `mk1.origin_fingerprint` fallback:** Keep the fallback BUT emit a stderr NOTICE on each substitution: `"notice: import-wallet: mk1[{slot_idx}]: origin_fingerprint absent; substituting xpub-derived fingerprint {hex} (master-fp and current-xpub-fp may differ; downstream wallets may show mismatched origins)"`. Close the self-confessed `let _ = slot_idx; // reserved` gap by wiring `slot_idx` through to the NOTICE.
- **Lock — `extract_threshold` (R1 line-number correction):** Change signature from `fn extract_threshold(&self) -> Option<u8>` to `fn extract_threshold(&self) -> Result<Option<u8>, ToolkitError>`. None case: "no `thresh()` token found in descriptor" — return `Ok(None)`. Some(non-u8) case: "thresh argument `{arg}` exceeds u8 range (>255 cosigners not supported)" — return `Err(BadInput)`. Affected sites (per Phase 0 source verification — R0 caught off-by-N inherited from agent-report I6):
  - Definitions: `wallet_import/bsms.rs:414` + `wallet_import/bitcoin_core.rs:456`.
  - Call sites (signature-change ripple): `wallet_import/bsms.rs:198` + `wallet_import/bitcoin_core.rs:271`.
  Phase 0 re-confirms these against current `master` post-PR-#27-merge.

### Q3 (Phase 3 — comment-rot sweep approach).

**Sweep is a single commit OR per-finding commits?**

- **Lock:** Single commit. The 5 sub-findings are mechanical text edits with no compile risk; bundling them into one commit reduces churn + makes the diff easier to review. Commit subject: `docs: pr-26 comment-rot fold (C2 + I8-I11)`.
- **`compare-cost-single-leaf-tr-input` slug:** I8 cites the slug at `cost/strip.rs:5,51` and `cost/mod.rs:75`. The slug was filed in `53a1bf6` (v0.27.0 cycle); v0.27.1 Phase 3 verifies the slug resolves via `grep -F` and leaves the cite intact.
- **`§7.0.a..d` SPEC citations (R1 — REVERSED to wontfix).** R0 source-verified that I9's premise was wrong: `§7.0.a/b/d` IS a real anchor — it lives in `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md:561` (`#### §7.0 Pre-execution SPEC + BRAINSTORM amendments`) with valid sub-labels a-d. `SPEC_wallet_import_v0_26_0.md:102, 126` themselves cite `§7.0.b` / `§7.0.d` — i.e. the SPEC body relies on these IMPLEMENTATION_PLAN sub-labels. The 2 source comments at `wallet_import/bsms.rs:10` and `bitcoin_core.rs:34` correctly cite IMPLEMENTATION_PLAN §7.0.a; the agent-report I9 finding was off-base.
- **Lock:** I9 = **wontfix** ("agent-report finding was wrong — anchor is correct cross-doc reference"). Phase 3 leaves these 2 source comments untouched. Phase 0 re-verifies the IMPLEMENTATION_PLAN §7.0 anchor existence in current master. See Q3a.

### Q3a (NEW R1 — Phase 0 anchor verification).

**Phase 0 verifies §7.0.a/b/d cross-doc anchors are still load-bearing in current master.**

- **Lock:** Phase 0 runs `grep -n '§7.0\.' design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md design/SPEC_wallet_import_v0_26_0.md crates/mnemonic-toolkit/src/wallet_import/` and confirms ≥4 hits (IMPLEMENTATION_PLAN §7.0 header + 3 source citations). If the IMPLEMENTATION_PLAN file has been deleted or restructured post-v0.26.0, downgrade I9 from wontfix to the original Q3 lock (b) approach.

### Q4 (Phase 4 — test-cell ordering).

**8 new cells across 4 test files. Add cells inline (next to similar existing cells) or as a contiguous block at file end?**

- **Lock:** Contiguous block at file end, header-comment'd with `// =================...` boundary + `// v0.27.1 PR-#26 coverage gap fold (Ix)` comment per cell. Mirrors v0.27.0 Phase 6.5 I4/I5/I6 cell-addition pattern at `tests/cli_export_wallet_from_import_json.rs:329-422`. Easier to bisect; doesn't disrupt existing cell numbering.

### Q5 (Phase 5 — type-design refactor wire-shape preservation).

**Three refactors (`SearchOutcome`, `ImportProvenance`, `BsmsVerification`) — preserve JSON envelope wire shapes byte-for-byte OR allow additive shape evolution?**

- **Lock — wire-shape preservation (byte-exact).** All three refactors use `#[serde(flatten)]` or `#[serde(untagged)]` or manual `Serialize` impls to preserve existing JSON output exactly:
  - **`SearchOutcome<T>` (Phase 5a) — re-scoped per R0 I1.** R0 caught that a flat `enum { Match(T), NoMatch }` does NOT work because the 4 xpub-search result structs carry **always-emitted envelope-scope fields** alongside the match-only payload (e.g. `PathOfXpubResult.{target_xpub_canonical, target_xpub_variant, searched_count}` are present on BOTH `result:"match"` AND `result:"no_match"` per source verification). See Q5a for the resolved shape.
  - **`ImportProvenance` (Phase 5b):** This struct is `pub(crate)`, not Serialize-deriving directly — the envelope-side wire shape lives on `ImportJsonEnvelope` (`bsms_audit` + `source_metadata` as flat sibling fields). Refactor scope: change `ParsedImport`'s internal representation only; keep the envelope-side fields unchanged via per-format match in the envelope emit code.
  - **`BsmsVerification` (Phase 5c):** mirror the v0.27.0 Phase 6.5 I7 `Round1VerificationStatus` enum precedent (`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:843-850`). The wire-shape stays `"signature_verified": false` — the field becomes a derived getter on the enum. v0.26 envelope consumers see no change.

- **Why preservation is non-negotiable:** the v0.26.0 → v0.27.0 BundleJson wire-shape replacement was already a SemVer minor-justified change (per v0.27.0 cycle close). v0.27.1 is a PATCH bump — additive-OK only, no replacement. Type-design refactors that would change wire shape get re-scoped to v0.28+.

### Q5a (NEW R1 — Phase 5a worked shape for `SearchOutcome<T>`).

**Phase 5a — REVISED via R1 cargo-build smoke test. Flat-enum refactor is wire-shape-incompatible.**

- **Source verification (R0):**
  - `cmd/xpub_search/path_of_xpub.rs:144-163` `PathOfXpubResult`: `result: &'static str`, `path: Option<String>`, `template: Option<String>`, `account: Option<u32>`, then ALWAYS-emitted `target_xpub_canonical: String`, `target_xpub_variant: Option<&'static str>`, `searched_count: usize`. **The Option fields have NO `#[serde(skip_serializing_if = "Option::is_none")]`** — `null` is emitted explicitly on no-match (per `path_of_xpub.rs` doc-comment "`null` on no-match").
  - `cmd/xpub_search/passphrase_of_xpub.rs:169-188` — same shape.
  - `cmd/xpub_search/account_of_descriptor.rs:155-170` — match-only `matched_cosigners: Vec<MatchedCosignerJson>` (empty vec on no-match — already harmless) + always-emitted cosigners_total, searched_count_per_cosigner, descriptor_shape, unspendable_internal_keys.
  - `cmd/xpub_search/address_of_xpub.rs:74-91` — already `#[serde(untagged)]` with `Match{...} | NoMatch{...}`.

- **Smoke-test findings (R1 — `/tmp/serde_flatten_smoke/`):**
  - `#[serde(flatten)]` **cannot be used on newtype enum variants** (`Match(#[serde(flatten)] T)`) — serde rejects at derive time.
  - The struct-variant form `Match { #[serde(flatten)] body: T }` compiles AND produces correct match-case wire output BUT omits the `null`-fields on no-match — a wire-shape change for no-match consumers.
  - Conclusion: a tagged-union enum refactor **cannot byte-preserve** the current wire shape that emits `path:null, template:null, account:null` on no-match.

- **Lock (R1 PIVOT) — Private-constructor + smart-builders, NOT an enum refactor.** The type-design goal is "make illegal states unrepresentable at the API boundary." This is achievable WITHOUT changing the serde shape by:
  1. Keep the existing `PathOfXpubResult` struct shape (all fields `pub` — must stay pub since they're across-module-boundary). Wire shape unchanged.
  2. Add a **private wrapper module** with private constructors:
     ```rust
     mod result_builders {
         use super::*;
         pub fn build_match(
             match_payload: PathOfXpubMatch,  // the 3 correlated fields packed
             envelope: PathOfXpubEnvelope,     // the 3 always-emitted fields
         ) -> PathOfXpubResult {
             PathOfXpubResult {
                 result: "match",
                 path: Some(match_payload.path),
                 template: Some(match_payload.template),
                 account: match_payload.account,  // Option per existing semantics
                 target_xpub_canonical: envelope.target_xpub_canonical,
                 target_xpub_variant: envelope.target_xpub_variant,
                 searched_count: envelope.searched_count,
             }
         }
         pub fn build_no_match(envelope: PathOfXpubEnvelope) -> PathOfXpubResult {
             PathOfXpubResult {
                 result: "no_match",
                 path: None,
                 template: None,
                 account: None,
                 target_xpub_canonical: envelope.target_xpub_canonical,
                 target_xpub_variant: envelope.target_xpub_variant,
                 searched_count: envelope.searched_count,
             }
         }
     }
     ```
  3. Replace all direct struct-literal sites with `build_match(...)` / `build_no_match(...)` calls.
  4. Add a `#[deprecated]`-or-`#[doc(hidden)]` note that direct struct-literal construction is the legacy API (it remains usable since fields are `pub`, but the constructor is the recommended path).

- **Why this achieves the type-design goal without wire-shape risk:** internal call sites use the constructors → cannot accidentally emit `result:"match"` with `path:None`. External JSON consumers see the existing wire shape unchanged. The `enum SearchOutcome` pattern from the type-design-analyzer's report is **not realistic under v0.27.1's wire-shape-preservation constraint**; private-constructor discipline is the alternative that delivers the same invariant.

- **Apply pattern to:** `path_of_xpub`, `passphrase_of_xpub`, `account_of_descriptor`. `address_of_xpub` is already enum-typed — no change. Phase 5a adds drift cells for all 4 (3 refactored + 1 reference).

- **Cell discipline (R2 — M3 reconciliation):** **3 drift cells** for Phase 5a (one per refactored struct: path / passphrase / account, each replaying match + no-match fixtures byte-for-byte). `address_of_xpub` is the reference example and is **NOT touched** in Phase 5a — no drift cell needed since no edit means no drift risk; if a future cycle refactors `address_of_xpub`, that cycle adds its own drift cell.

- **Memory note:** record `feedback-serde-flatten-newtype-variant` after the cycle ships, since the underlying serde constraint isn't obvious from doc-comments and R1 only caught it via cargo-build smoke test. Avoids future R0 dispatches re-litigating the same path.

### Q5b (NEW R2 — Phase 5a invariant-enforcement mode lock).

**R1 challenged Q5a's claim that private-constructor scaffolding satisfies the type-design-analyzer's I20 invariant.** It doesn't — direct struct-literal construction remains legal because fields are `pub` for cross-module access, and `#[deprecated]` on the free builder does not propagate to literal construction. The builder is a *recommendation*, not an *enforcement*.

- **Lock — option (a):** Ship Phase 5a builders + an internal-call-site discipline sweep + file a **NEW v0.27.1 cycle-close FOLLOWUP** `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` targeting v0.28+ (where SemVer minor permits the wire-shape change that lets the type-level fix — e.g. tagged enum with `#[serde(skip_serializing_if = "Option::is_none")]` — actually land).
- **Phase 5a deliverable becomes "API-discipline scaffolding for v0.28+,"** not "type-design refactor." The plan-doc's claim is downgraded accordingly throughout (Q5a / §4 Phase 5a / §2 row 5 / CHANGELOG wording at Phase 6).
- **Add Phase 5a R0 pre-flight grep** (call-site audit): the Phase 5a `feature-dev:code-reviewer` R0 dispatch confirms `grep -rn 'PathOfXpubResult {\|PassphraseOfXpubResult {\|AccountOfDescriptorResult {' crates/mnemonic-toolkit/src` returns ≤N direct-literal hits (where N = Phase 5a's expected internal call sites — typically 2 per struct: one match-arm + one no-match-arm in the search subcommand). External-zero hits unlock option (c) in v0.28+. Across `mnemonic-gui` repo at the consumed pin: same grep MUST return zero external hits (these are pub fields but pub(crate) at the toolkit level — gui shouldn't be constructing them anyway).
- **What we still claim in v0.27.1:** Phase 5a improves API discipline (call sites can no longer accidentally drop the `result:"match"` ↔ `path:Some(...)` correlation when refactoring future code paths); does NOT eliminate representable-invalid states at the type level. Lower bar, but honest.

### Q5c (NEW R2 — drift fixture maintenance ownership).

**Once `tests/fixtures/v0_27_0_envelopes/` lands, who maintains it?**

- **Lock:** Fixtures are **pinned to v0.27.0 forever** as regression guards against accidental drift on the v0.27.x patch line. Each future minor bump (v0.28.0+) that legitimately changes wire shape MUST (a) add a companion fixture dir `tests/fixtures/v0_28_0_envelopes/`, (b) update the drift-cell to replay both v0.27.0 AND v0.28.0 fixtures (the v0.27.0 cell may now legitimately FAIL on minor cycles — convert to `#[ignore]` with a doc comment explaining the SemVer-minor rationale), (c) document the wire-shape diff in CHANGELOG `### Changed` per the v0.26.0 → v0.27.0 precedent.
- **Cycle-close requirement:** Phase 6 verifies `tests/fixtures/v0_27_0_envelopes/` exists and is referenced by ≥5 drift cells. Phase 6 holistic review specifically audits this artifact path.

### Q6 (Cycle-wide — drift-regression discipline for Phase 5 refactors).

**How do we prevent the Phase 5 refactors from silently changing wire shape?**

- **Lock — fixture-set + drift regression cells (R1 strengthened per R0 informational observation).** Capture a `tests/fixtures/v0_27_0_envelopes/` directory at Phase 0 with N reference outputs (one per affected emit path, captured via `cargo run --release ...` against v0.27.0 master tip BEFORE any v0.27.1 source change). Phase 5 then replays each fixture through v0.27.1's serializer and asserts byte-equality.
- **Cell count:** **5 drift cells** (R1 — up from R0's 3):
  - 3 cells for Phase 5a (path_of_xpub, passphrase_of_xpub, account_of_descriptor — per Q5a) replaying match + no-match fixtures.
  - 1 cell for Phase 5b (`ParsedImport` ImportProvenance) replaying both BSMS-source and Bitcoin-Core-source envelopes.
  - 1 cell for Phase 5c (`BsmsAuditFields.signature_verified` enum) replaying a 6-line BSMS audit envelope.
- **Precedent:** the Phase 4 (v0.27.0 cycle) `bundle_json_view_round_trips_every_field_of_bundle_json` cell at `src/wallet_import/json_envelope.rs:tests` is the simplest variant; the fixture-set approach extends it across 3 emit paths.

---

## §4. Phase plan

### Phase 0 — Reconnaissance + plan-doc finalization

**Scope:** verify all 19 Important findings against current source (may have shifted since the agent reports were generated against `release/v0.27.0` tip); confirm `master` post-merge tip is the actual base; cross-check `compare-cost-single-leaf-tr-input` defer-to-future-cycle scoping.

**Output artifacts:**
- `design/agent-reports/v0_27_1-phase-0-recon.md` — fact-checked findings list with current-source `file:line` citations.
- `design/PLAN_v0_27_1_pr_26_fold_cycle.md` — this doc, updated with any Phase 0 corrections.

**Cells:** none (recon-only).
**LOC:** doc-only.
**Acceptance:** opus R0 architect review on this plan-doc returns 0 Critical / 0 Important after iteration.

### Phase 1 — `pr-26-roundtrip-warning-suppression` fold (C1 + I7)

**Scope:** Per Q1 + Q1a.

**Phase 1 step order (R1):**
1. **Commit 1 (docs-only):** SPEC amendment per Q1a — add `error: String` field to `roundtrip` `canonicalize_failed` branch in `SPEC_wallet_import_v0_26_0.md` §2.2 + `SPEC_mnemonic_toolkit_v0_5.md` §X.
2. **Commit 2 (impl + cells):** stderr warning arm + JSON-mode `error` field wiring + 3 new cells.

**Files (impl):**
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:471-478` (stderr warning arm)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:334-338, 396-402` (JSON-mode roundtrip emit)

**New cells (`tests/cli_import_wallet_roundtrip.rs` + `tests/cli_import_wallet_bitcoin_core.rs`):**
1. `roundtrip_canonicalize_failure_emits_stderr_warning_lenient` — feed a Bitcoin Core blob the parser accepts + canonicalizer rejects; assert exit 0 + stderr contains "roundtrip check skipped: canonicalize_bitcoin_core failed".
2. `roundtrip_canonicalize_failure_in_json_mode_envelope_carries_error_field` — same blob, `--json` set; assert exit 0 + envelope's `roundtrip.status == "canonicalize_failed"` AND `roundtrip.error` non-empty.
3. `roundtrip_non_utf8_blob_emits_notice_and_lossy_attempt` — bytes-not-UTF-8 blob (e.g., raw 0xFF prefix); assert exit 0 + stderr contains "not UTF-8" notice + roundtrip status reflects lossy-decode outcome.

**LOC:** ~50.
**Per-phase R0 architect review (opus):** dispatched after impl; iterate to 0/0.

### Phase 2 — `pr-26-shape-mismatch-silent-defaults` fold (I4 + I5 + I6)

**Scope:** Per Q2.

**Files:**
- `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:273-280` (active/internal shape-strictness)
- `crates/mnemonic-toolkit/src/wallet_import/json_envelope.rs:258-260` + slot_idx wiring (fingerprint fallback NOTICE)
- `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:354-362` + `bitcoin_core.rs:455-462` (extract_threshold Result return)

**New cells:**
1. `bitcoin_core_active_non_boolean_errors_with_pointer_text` — `"active": "true"` (string) — exit 1 with `ImportWalletParse` citing field name.
2. `bitcoin_core_internal_non_boolean_errors_with_pointer_text` — `"internal": 1` (number) — same shape.
3. `mk1_missing_origin_fingerprint_emits_substitution_notice` — bundle JSON with `mk1[0].origin_fingerprint: null` — exit 0, stderr contains slot index + substituted hex + "downstream wallets may show mismatched origins".
4. `bsms_thresh_overflow_errors_clearly` — descriptor with `thresh(256, ...)` — exit 1 with "exceeds u8 range".
5. `bitcoin_core_thresh_overflow_errors_clearly` — same descriptor via Core path — same shape.
6. `bitcoin_core_active_absent_defaults_false` — `{}` without `active` key — exit 0, parses with `active: false` (regression guard that absent-vs-shape-wrong distinction works correctly).

**LOC:** ~80.
**Per-phase R0 architect review (opus):** dispatched after impl.

### Phase 3 — `pr-26-comment-rot-fold` (C2 + I8-I11)

**Scope:** Per Q3.

**Files:**
- `crates/mnemonic-toolkit/src/env_sentinel.rs:1-13` (C2 — `--slot @N.ms1=` → `--from <node>=` per SPEC §3.1)
- `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:10` + `bitcoin_core.rs:34` (I9 — §7.0.a..d citations rewritten to in-prose locks)
- `crates/mnemonic-toolkit/src/error.rs:181-222` (I10 — "Phase N emits" replaced with function-anchored citations)
- `crates/mnemonic-toolkit/src/cost/mod.rs:75` (I11 — drop "in Phase 2" from user-visible string)

**New cells:** none. Comment-rot sweep verified by `cargo doc --no-deps` clean build + `grep -F` checks for the removed phrases (run as a Phase 3 R0 manual check).

**LOC:** ~30 (comment edits only).
**Per-phase R0 architect review:** lightweight (sonnet acceptable for trivial doc-only phase, per `[[feedback-opus-primary-review-agent]]` — Sonnet acceptable for trivial fold-verify only). Single-round.

### Phase 4 — `pr-26-test-coverage-gap-fold` (I12-I19)

**Scope:** 8 new test cells per Q4 ordering rule.

**Files (test-only):**
- `tests/cli_import_wallet_seed_overlay.rs` — I12 (overlay conflict), I13 (phrase mismatch), I14 (non-entropy ms1)
- `tests/cli_import_wallet_bitcoin_core.rs` — I15 (--select-descriptor matrix: invalid-index, no-active-match, malformed)
- `tests/cli_import_wallet_sniff.rs` — I16 (Ambiguous arm). **(M2 fold — pre-lock fallback:)** If no contrived dual-sniff blob can be constructed in current source, the cell becomes a unit test inside `wallet_import/sniff.rs` that (a) constructs `SniffOutcome::Ambiguous` directly, (b) calls a NEW helper `format_ambiguous_error() -> String` extracted from `cmd/import_wallet.rs:168-170`, (c) asserts the rendered template matches the SPEC §6 ambiguous-error wording. Helper extraction is in-scope for Phase 4 — not a separate refactor.
- `tests/cli_import_wallet_bsms.rs` — I17 (unrecognized line-count: 3/4/5/7+), I18 (sniff false-positive on lowercase / leading whitespace)
- `tests/cli_import_wallet_roundtrip.rs` — I19 (multisig round-trip `byte_exact` / `semantic_match` assertion)

**Cell count: 8.**
**LOC:** ~250.
**Per-phase R0 architect review (opus):** dispatched after all 8 cells; iterate to 0/0.

### Phase 5 — `pr-26-type-design-anti-pattern-sweep` (I20 + I21 + recurring patterns)

**Scope:** Per Q5 + Q5a + Q6.

**Phase 5 follows Phase 4 by design (M1 fold):** Phase 4's 8 new I12-I19 cells act as regression guards over Phase 5's refactor edits. Doing it in the reverse order would lose regression-coverage signal — any wire-shape leak from Phase 5 would only be caught by the Phase 6 holistic review (late + expensive).

**Sub-phase 5a — `SearchOutcome<T>`** (xpub-search 3-result-struct unification per Q5a worked shape; `address_of_xpub` is reference example only).

**Files:**
- `crates/mnemonic-toolkit/src/cmd/xpub_search/path_of_xpub.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/account_of_descriptor.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs` (already untagged; verify pattern conformance only — no edit)
- NEW: `tests/fixtures/v0_27_0_envelopes/{path,passphrase,account}_of_xpub.{match,no_match}.json` (6 fixture files captured at Phase 0 from v0.27.0 master output; pinned forever per Q5c)

**New cells:** **3 drift regression cells** (Q6 / Q5a / R2 M3: one per refactored struct), each pinning match + no-match wire shapes byte-for-byte against the fixtures.

**Pre-flight grep (Phase 5a R0 gate per Q5b):** `grep -rn 'PathOfXpubResult {\|PassphraseOfXpubResult {\|AccountOfDescriptorResult {' crates/mnemonic-toolkit/src` returns ≤N internal direct-literal hits (N = expected match/no-match construction sites — typically 2-3 per struct). External-zero hits in mnemonic-gui repo at consumed pin.

**LOC:** ~100 (Q5a worked-shape refactor is slightly larger than R0's flat-enum estimate).

**Sub-phase 5b — `ImportProvenance`** (`ParsedImport` provenance enum).

**Files:**
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs:60` (ParsedImport struct def)
- `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` (BsmsParser construct site)
- `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs` (BitcoinCoreParser construct site)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (envelope emit consumer match arms)

**New cells:** 1 drift regression — envelope shape byte-equality before/after.

**LOC:** ~50.

**Sub-phase 5c — `BsmsVerification`** (replace `BsmsAuditFields.signature_verified: bool`).

**Files:**
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs:188` (BsmsAuditFields struct def)
- `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:122-124` (construct site)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (envelope emit if it touches signature_verified directly)

**Pattern:** mirror `Round1VerificationStatus` enum from `cmd/import_wallet.rs:843-850` (Phase 6.5 I7 precedent).

**New cells:** 1 drift regression — envelope shape byte-equality.

**LOC:** ~30.

**Phase 5 total: ~180 LOC (5a ~100 + 5b ~50 + 5c ~30) + 5 drift cells (3 for 5a covering path/passphrase/account, 1 for 5b ImportProvenance, 1 for 5c BsmsVerification). `address_of_xpub` already enum-typed — no edit, no extra cell (R1 M3 reconciliation).**
**Per-phase R0 architect review (opus):** dispatched after each sub-phase OR once after all three (preference: once after all three sub-phases land, since the three are mechanically similar and a single review covers the pattern; if 5a uncovers a deeper wire-shape gap, dispatch 5a's R0 standalone before 5b/5c).

### Phase 6 — Cycle close

**Scope:** CHANGELOG + FOLLOWUPS Status flips + version bump + opus end-of-cycle holistic review.

**Tasks:**
1. `crates/mnemonic-toolkit/Cargo.toml` `version = "0.27.1"` (patch bump).
2. `crates/mnemonic-toolkit/CHANGELOG.md` v0.27.1 section: `### Fixed` (Phase 1 + 2 silent-failure folds), `### Changed` (Phase 2 active/internal shape-strictness behavior change — previously silent default; now exit 1 on shape-wrong), `### Changed (internal)` (Phase 5: API-discipline scaffolding for xpub-search result types per Q5b — wire-shape preserved per Q5; type-level invariant blocked on v0.28+ wire-shape evolution; tracked by new FOLLOWUP `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` AND Phase 5b/5c `ImportProvenance` + `BsmsVerification` internal-refactor commits), `### Closed FOLLOWUPS` (5 of 6; #6 `compare-cost-single-leaf-tr-input` remains open — note `[deferred to v0.28+]` in the entry per Risk row 3).
3. `design/FOLLOWUPS.md`: flip **5 entries** to `**Status:** resolved` with closure narratives. Per `[[feedback-per-phase-agents-forget-followup-status-flip]]` memory: Status flips are explicit Phase 6 step. **R2 addition (Q5b):** file 1 NEW v0.27.1-cycle-close FOLLOWUP `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` (deferred to v0.28+ — explains that Phase 5a shipped API-discipline scaffolding only; the type-level invariant requires wire-shape change that PATCH bump disallows).
4. **(M3 fold)** Verify slug `compare-cost-single-leaf-tr-input` remains `**Status:** open` — `grep -A2 '^### \`compare-cost-single-leaf-tr-input\`' design/FOLLOWUPS.md` shows `**Status:** open` literal. No double-state hazard (Status open AND CHANGELOG `### Closed`).
5. Opus end-of-cycle holistic architect review (`feature-dev:code-reviewer` model=opus): full-cycle audit + cell-count verification + version-bump sanity + CHANGELOG accuracy + manual mirror coverage + drift-fixture replay verification.
6. Tag `mnemonic-toolkit-v0.27.1` post-PR-merge; GitHub release with patch CHANGELOG body.

---

## §5. Acceptance gates

| Gate | Check | Phase |
|---|---|---|
| Cycle baseline matches advisory | branch off post-merge master (PR #27 squash-merged) | 0 |
| Build green throughout | `cargo build -p mnemonic-toolkit` succeeds at every commit boundary | per-phase |
| Test suite green | `cargo test -p mnemonic-toolkit` baseline + **22 new cells** (Phase 1: 3 + Phase 2: 6 + Phase 4: 8 + Phase 5: 5, per R2 M2 reconciliation) | per-phase |
| Clippy clean | `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` | per-phase |
| Manual lint OK | `make -C docs/manual lint MNEMONIC_BIN=...` | per-phase |
| Phase 1 SPEC-amendment-first | Q1a docs-only commit precedes Phase 1 impl commit | 1 |
| Q3a §7.0 anchor verification | Phase 0 `grep` ≥4 hits for `§7.0.` in IMPLEMENTATION_PLAN + SPEC + source | 0 |
| Drift fixtures captured pre-edit | `tests/fixtures/v0_27_0_envelopes/` populated from v0.27.0 master output before Phase 5 starts | 0 |
| Wire-shape byte-preservation | 5 drift-regression cells in Phase 5 pass against captured fixtures (Phase 5a × 3 + 5b × 1 + 5c × 1) | 5 |
| Q5b internal-call-site grep returns ≤N hits | Phase 5a R0 gate runs the grep + GUI external-zero verification | 5a R0 |
| Q5c fixture pinning documented | `tests/fixtures/v0_27_0_envelopes/` exists + README explains the pinned-forever discipline | 0 + 6 |
| Per-phase R0 returns 0/0 | opus architect review converges per phase | 1, 2, 4, 5 (sonnet OK for 3) |
| FOLLOWUPS Status flipped in same commit as CHANGELOG | grep verification at cycle close | 6 |
| `compare-cost-single-leaf-tr-input` slug resolves AND stays open | grep design/FOLLOWUPS.md returns 1 hit + `**Status:** open` | 0 (verify) + 6 (re-verify) |
| End-of-cycle holistic returns GREEN (post-fold if needed) | opus review pre-tag | 6 |

---

## §6. Out of scope

- Item #6 (`compare-cost-single-leaf-tr-input`) implementation — defer to a separate cycle with SPEC anchor (§2.1).
- mnemonic-gui pin bump — toolkit-only patch; GUI consumes v0.27.0's existing wire shape (no API surface change per Q5).
- Sibling-codec lockstep — none of the 6 FOLLOWUPs touch md-codec / ms-codec / mk-codec.
- v0.26.x backport — patch ships on v0.27.x line only.
- Cross-format conversion matrix expansion (separate FOLLOWUP `cross-format-conversion-matrix-expansion` filed at v0.27.0 cycle close).
- BSMS Round-1 verify outside the existing v0.27.0 surface.

---

## §7. Risks + mitigations

| Risk | Mitigation |
|---|---|
| Phase 5 type-design refactor leaks wire shape | Q6 drift regression cells (3); end-of-cycle holistic verifies envelope byte-equality against pre-cycle reference output | 
| Phase 4 cell I16 (`SniffOutcome::Ambiguous`) is unreachable with current parsers, may require helper extraction | Pre-flight Phase 4: confirm `cmd/import_wallet.rs:168-170` can be exercised via a test-only helper; if not, downscope I16 to a unit test on the dispatch fn |
| `compare-cost-single-leaf-tr-input` user-visible error cites slug → patch cycle adds slug but no implementation; user may perceive inconsistency | CHANGELOG `### Closed FOLLOWUPS` section explicitly notes #6 is filed-only; cite `[deferred to v0.28+]` in the entry |
| Phase 2 active/internal shape-strictness is a behavior change (previously silent default; now error on shape-wrong) — could break consumers feeding malformed JSON | CHANGELOG `### Changed` entry explicitly notes behavior change; Phase 0 pre-flight `grep` mnemonic-gui at the consumed pin for any non-boolean emission to `active`/`internal` (R1 strengthened — was "verify before tagging") |
| Phase 5a `SearchOutcome<T>` enum refactor is incompatible with wire-shape preservation (smoke test confirmed R1 pivot to private-constructor approach per Q5a) | Phase 5a is now a private-constructor refactor — fields stay `pub` for back-compat; the invariant is enforced via call-site discipline, not the type system. Phase 5a R0 still dispatched STANDALONE before 5b/5c |
| Per-phase R0 returns large finding lists, forcing rework loops | Plan-doc R0 first (this doc) catches design issues before code; per-phase R0 catches implementation-level issues only |
| PR #27 merge delays push v0.27.1 start | Plan-doc + Phase 0 recon completable on scratch branch off `release/v0.27.0` in parallel (architect advisory) |

---

## §8. Plan version history

| Rev | Date | Change | Reviewer status |
|---|---|---|---|
| R0 | 2026-05-19 | Initial draft | opus R0: YELLOW (4 Important + 3 Minor + 3 new Q-items) |
| R1 | 2026-05-19 | R0 folds: I1 → Q5a pivot (private-constructor refactor — flat-enum & flatten-via-struct-variant approaches both proven wire-shape-incompatible via cargo-build smoke test); I2 → Q1 corrected (status is SPEC-locked) + Q1a (SPEC amendment-first); I3 → Q3 reversed to wontfix + Q3a (§7.0 anchor verify); I4 → extract_threshold line ranges corrected; M1 → Phase 5-after-Phase-4 rationale documented; M2 → I16 helper-extract fallback pre-locked; M3 → Phase 6 task 4 verifies #6 slug stays open; §2 budget + §5 acceptance gates + §7 risks updated | opus R1: YELLOW (1 Important + 3 Minor + 2 new Q-items) |
| R2 | 2026-05-19 | R1 folds: I1 (R1) → Q5b lock to option (a) — ship API-discipline-scaffolding builders in v0.27.1 + file new FOLLOWUP `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` targeting v0.28+; M1 → Phase 5 LOC tightened to ~180; M2 → §2 + §5 cell-count reconciled to 22; M3 → Phase 5a cell-count fixed at 3 (no 4th `address_of_xpub` cell since no edit); Q5c → drift-fixture pinning discipline locked; §4 Phase 6 task 3 amended to include the new FOLLOWUP filing | opus R2: GREEN (1 trivial Minor on CHANGELOG-stub wording folded inline) |

