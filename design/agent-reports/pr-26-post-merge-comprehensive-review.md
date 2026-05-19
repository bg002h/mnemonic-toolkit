# PR #26 (v0.26.0) Post-Merge Comprehensive Review

**Date:** 2026-05-19
**PR:** [bg002h/mnemonic-toolkit#26](https://github.com/bg002h/mnemonic-toolkit/pull/26)
**Branch:** `release/v0.26.0` → `master` (squash-merged at `66c8a56`)
**Diff scope:** 96 files, +20,728 / -60 LOC
**Cycle bundled:** compare-cost + xpub-search + import-wallet (three-way integration-branch model per `design/PLAN_v0_26_0_three_way_merge.md`)

**Review framing:** Run-after-merge audit dispatched from the v0.27.0 cycle close as part of PR #27's `pr-review-toolkit:review-pr` skill. Two phases:

- **Phase 1:** `code-reviewer` solo.
- **Phase 2:** 4 specialist agents in parallel (`silent-failure-hunter`, `comment-analyzer`, `type-design-analyzer`, `pr-test-analyzer`).

All 5 agents were `pr-review-toolkit:*` opus dispatches.

---

## Aggregate verdict

**No ship-blocker findings.** PR #26 was correctly mergeable when squashed. The findings below are improvements to file as FOLLOWUPs for the v0.26.x / v0.27.x patch lines.

| Severity | Count |
|---|---|
| Critical | 2 |
| Important | 19 |
| Type-design recurring anti-patterns | 6 patterns |
| Test-quality issues | 5 |
| Strong types / positive observations | 8 |

---

## Critical findings (2)

### C1 — `emit_roundtrip_stderr_warning` silently suppresses canonicalize/UTF-8 errors

- **Source:** silent-failure-hunter
- **File:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:471-478` (two `Err(_) => return Ok(())` arms)
- **Risk class:** silent security-relevant signal loss. The SPEC §7.4 stderr warning is the **only** non-JSON-mode feedback that the user's Bitcoin Core blob isn't being round-tripped byte-exactly. If `canonicalize_bitcoin_core` errors (parser accepts what canonicalizer rejects; non-UTF-8 input; internal serde mismatch), the function returns success with **no stderr diagnostic** — user sees an apparently-clean import that may have silently mutated the descriptor.
- **Fix shape:** emit `"warning: import-wallet: roundtrip check skipped: canonicalize_bitcoin_core failed: {e}"` on the Err arm; for non-UTF-8 input, log a notice that the original was not UTF-8 (via `String::from_utf8_lossy` + explicit notice).

### C2 — `env_sentinel.rs` module doc enumerates a non-existent `--slot @N.ms1=` surface

- **Source:** comment-analyzer
- **File:** `crates/mnemonic-toolkit/src/env_sentinel.rs:1-13`
- **Issue:** Module doc lists 6 secret-flag surfaces. Row 6 is `--slot @N.ms1=`. **`SlotSubkey` (`slot_input.rs:17-32`) has no `Ms1` variant** — variants are `{Phrase, Entropy, Xpub, Wif, Xprv, Cscek}`. SPEC §3.1 explicitly flags this mistake at line 96 and says the actual row 6 is `--from <node>=` (composite for `convert` / `derive-child` / `slip39-*` / `seed-xor-*`).
- **Fix shape:** Replace row 6 with the SPEC §3.1 table content verbatim.

---

## Important findings (19, deduped across all 5 agents)

### Code (5) — code-reviewer Phase 1 + silent-failure Phase 2

| # | Source | Finding | Site |
|---|---|---|---|
| I1 | code-reviewer | xpub-search stdin mutex gap (no `Mutex<Stdin>` guard around the single global handle when multi-mode stdin reads collide). | xpub_search subcommands |
| I2 | code-reviewer | `flag_is_secret` predicate is missing `--phrase` (and any other slot-phrase surface). Argv-leak advisory may not fire on `--slot @N.phrase=…`. | argv-leak machinery |
| I3 | code-reviewer | compare-cost wrong exit code (mapping diverges from SPEC §9). | compare-cost |
| I4 | silent-failure | Bitcoin Core `active`/`internal` silently default `false` on shape-mismatch — unlike `parse_range_field` which correctly errors. Silently flips `--select-descriptor active-receive`/`active-change` to "no match" with a misleading error message. | `wallet_import/bitcoin_core.rs:273-280` |
| I5 | silent-failure | `mk1_card_to_resolved_slot` silently substitutes xpub-derived fingerprint for missing `origin_fingerprint`. Master-fp vs current-xpub-fp are semantically distinct; downstream descriptor reconstruction produces wallets with mismatched origin annotations and the user has no signal. | `wallet_import/json_envelope.rs:258-260` |
| I6 | silent-failure | `extract_threshold` silently maps u8 overflow → `null` threshold. A descriptor with `thresh(256, …)` renders as `"threshold": null`; the user sees a "no-threshold" descriptor when the underlying input is malformed. | `bsms.rs:354-362`, `bitcoin_core.rs:455-462` |
| I7 | silent-failure | JSON-mode round-trip drops the canonicalize error reason (`canonicalize_*.ok()` instead of `.map_err(...)`). Envelope shows only surface `"canonicalize_failed"` with no diagnostic for the consuming tool. | `cmd/import_wallet.rs:334-338, 396-402` |

### Comment (4) — comment-analyzer

| # | Finding | Site |
|---|---|---|
| I8 | Unfiled FOLLOWUP slug `compare-cost-single-leaf-tr-input` cited in user-visible error + 2 comments. `grep design/FOLLOWUPS.md` returns zero hits. | `cost/strip.rs:5,51`; `cost/mod.rs:75` |
| I9 | SPEC citation `§7.0.a..d` is not a real section — `SPEC_wallet_import_v0_26_0.md` has §1..§12 only; `§7.0.a..d` is leaked brainstorm shorthand. | `wallet_import/bsms.rs:10`; `bitcoin_core.rs:34` |
| I10 | `error.rs` doc comments tag variants "Phase 5 emits" / "Phase 2 emits" / "Phase 3 emits" — internal cycle-phase vocabulary that's meaningless post-cycle. | `error.rs:181-222` |
| I11 | User-visible error: `"compare-cost: unsupported wrapper '{w}'; supported in Phase 2: wsh(..), sh(wsh(..)). tr() input is deferred to v0.27 FOLLOWUP."` — internal "Phase 2" vocabulary exposed to end users. | `cost/mod.rs:75` |

### Test (8) — pr-test-analyzer

| # | Finding | Site |
|---|---|---|
| I12 | `--ms1` + `--slot @i.phrase=` conflict path is untested. `overlay.rs:89-94` returns `BadInput("cosigner {i} has both …")` — no cell exercises this. Regression risk: silent precedence change. | `cli_import_wallet_seed_overlay.rs` |
| I13 | Phrase-overlay-mismatch (Source::Phrase + wrong phrase) untested; only `Source::Ms1` mismatch is covered. | same |
| I14 | `apply_seed_overlay` non-entropy ms1 branch untested (`overlay.rs:128-132` rejects successfully-decoded ms1 cards whose payload is not `Payload::Entr`). | same |
| I15 | `--select-descriptor` invalid-index / no-active-match / malformed-selector cells missing. Currently only happy-path selectors (`all`, `2`, `active-receive`, `active-change`) are covered. | `cli_import_wallet_bitcoin_core.rs:107-131` |
| I16 | Sniff `Ambiguous` arm has no live integration test. Inline unit test documents the arm is unreachable with current parsers; the dispatch + stderr template are reachable code with no regression guard. | `wallet_import/sniff.rs:49` |
| I17 | BSMS unrecognized line-count (3/4/5/7+) only weakly covered. `bsms.rs:126-130` rejects all non-{2,6}; no cell pins the rejection template. | `wallet_import/bsms.rs:126-130` |
| I18 | BSMS sniff false-positive (lowercase / leading whitespace) not pinned. Future "tolerance" loosening could silently accept malformed blobs. | `bsms.rs:47-57` |
| I19 | Multisig round-trip suite has no asserted byte-exact / semantic-match comparison. `cli_import_wallet_roundtrip.rs:371-452` checks only fingerprint+count substrings; the `roundtrip` envelope (`byte_exact` / `semantic_match` / `diff`) is asserted only in the single-sig sniff suite. | `cli_import_wallet_roundtrip.rs:371-452` |

### Type-design (2) — type-design-analyzer

| # | Finding | Site |
|---|---|---|
| I20 | `ParsedImport`'s `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)` is a representable-invalid pair. 4 states encode 2 valid + 2 impossible (both-set, both-none for a typed parse). | `wallet_import/mod.rs:60` |
| I21 | `BsmsAuditFields.signature_verified: bool` is set to `false` unconditionally in v0.26.0 — dead state, future-trap. Same shape as the v0.27.0 Phase 6.5 I7 `Round1Verification(bool, Option<reason>)` we just refactored to a `Round1VerificationStatus` enum. | `wallet_import/mod.rs:188` |

---

## Recurring type-design anti-patterns (across multiple sites)

Surfaced by type-design-analyzer with severity rated per type. Cross-cutting patterns worth a sweep:

1. **`result: &'static str` field paired with `Option<...>` payloads** — appears 4 times across xpub-search results (`PathOfXpubResult`, `PassphraseOfXpubResult`, `AccountOfDescriptorResult`, `AddressResultJson` partial). Each is a representable-invalid pair. A single shared `enum SearchOutcome<T> { Match(T) | NoMatch }` would fix all four.
2. **`(bool, Option<reason>)` shape** — `BsmsAuditFields.signature_verified`, `Translated.concrete_keys`, `CosignerExtract.is_nums`. The Phase 6.5 I7 fold pattern (`Round1VerificationStatus` enum) applies.
3. **Parallel `Option`s** — `ParsedImport.{bsms_audit, source_metadata}`; `BundleJsonView.{origin_path, origin_paths}`. Provenance/cardinality enums collapse 4 states to 2.
4. **Parallel index-coupled `Vec`s** — `Translated.{labels, label_pubkeys}` (same index space); `BsmsParser::extract_origin_components` returns `Vec<(Fingerprint, DerivationPath, String, String)>` (4-tuple with last two being wire+typed forms of the same data). Lift to named structs.
5. **Primitive `String` where typed exists** — `BsmsAuditFields.derivation_path` and `.first_address` (typed `DerivationPath` and `bitcoin::Address` exist in the same module's imports); `chain: &'static str` in address-search results (should be enum `{External, Internal}`).
6. **`#[allow(dead_code)]` field clusters** — `CosignerExtract` (2 fields), `BsmsRound1Record.KeyField` (4 fields), entire `BundleJsonView`. Each suppression hides an "is this load-bearing or not" question.

### Top 3 highest-ROI refactors

1. **Unify the 4 xpub-search result structs** with `enum SearchOutcome { Match{…}, NoMatch }`. Eliminates 8 representable-invalid states across 4 types in one edit. ~80 LOC.
2. **`ParsedImport` provenance**: replace `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)` with `provenance: ImportProvenance` enum. ~30 LOC.
3. **`BsmsAuditFields.signature_verified: bool` → `BsmsVerification` enum** (`NotAttempted | Failed(reason) | Verified`). Parallels v0.27.0 Phase 6.5 I7's `Round1VerificationStatus` refactor; closes the future-trap before the BIP-322 inline-verifier lands.

### Strong types (reference examples worth replicating)

- `SniffOutcome` (10/10): closed 4-way enum + truth-table dispatch + Copy/Eq + exhaustive match. Textbook unrepresentable-illegal-states.
- `XpubSearchJson` (`#[serde(tag = "mode")]` + 4 well-named variants).
- `CompareCostError` with `exit_code()` method — centralizes the SPEC §9 mapping. Replicate for `ToolkitError::ImportWallet*` (currently distributed across 7+ variants).
- `DescriptorShape` (clean 3-variant closed enum).
- `AddressResultJson` (`#[serde(untagged)]` with `Match`/`NoMatch` variants).

---

## Test-quality issues (5)

- **Q1.** Substring `contains` assertions on stderr templates are brittle and pervasive across all import-wallet test files. Cell `seed_overlay_via_slot_subkey_phrase` accepts either of two substrings via `||` — template-split regression would pass.
- **Q2.** `bsms_multi_non_sorted_2_of_3` (`cli_import_wallet_bsms.rs:326-366`) accepts BOTH success and failure outcomes; vacuous coverage against SPEC §4.3 declaration-order invariant.
- **Q3.** `seed_overlay_env_var_sentinel` and similar env-var-resolution cells don't pin byte-exact resolved values.
- **Q4.** `soft_cap_advisory_fires_when_rows_exceed_threshold` is `#[ignore]`-gated → no CI surface enforces SPEC §3.3 step 7 invariant. Same `[[feedback-default-cargo-test-runs-sibling-dependent-tests]]` class.
- **Q5.** xpub-search no-match cells don't pin `searched_count` semantics.

---

## Positive observations

- `cli_import_wallet_bitcoin_core.rs:370-429` — three excellent regression cells (tprv refusal, zprv refusal, BIP-380 `xprv` substring false-positive defense) preventing fixed-bug returns.
- `cli_import_wallet_seed_overlay.rs:342-393` — multi-cosigner skip-middle: best behavioral cell in the suite. Three independent BIP-39 seeds, asserts `[true, false, true]` entropy pattern, validates NOTICE template.
- `cli_import_wallet_roundtrip.rs:111-142` — load-bearing declaration-order positional assertion (`idx_pos0 < idx_pos1 < idx_pos2`).
- `cli_compare_cost.rs:317-345` (`or_b_rejects_non_minimal_both_keys_row`) — asserts BOTH positive AND negative invariants for SPEC §3.3 step 5 minimality.
- `cli_xpub_search_address_of_xpub.rs:295-332` (`external_only_skips_internal_chain`) — pins `scanned_internal:0` payload.
- `CompareCostError.exit_code()` — exemplary error-type design.
- `SniffOutcome` — exemplary type design.
- `XpubSearchJson` `#[serde(tag = "mode")]` discipline.

---

## Files cited (absolute paths)

- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`
- `crates/mnemonic-toolkit/src/env_sentinel.rs`
- `crates/mnemonic-toolkit/src/wallet_import/{mod,bsms,bitcoin_core,sniff,overlay,roundtrip,bsms_round1,json_envelope}.rs`
- `crates/mnemonic-toolkit/src/cost/{mod,strip,translate,enumerate}.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/{mod,path_of_xpub,passphrase_of_xpub,account_of_descriptor,address_of_xpub,address_search,descriptor_intake,candidate_paths}.rs`
- `crates/mnemonic-toolkit/src/error.rs`
- `crates/mnemonic-toolkit/tests/cli_import_wallet_{bsms,bitcoin_core,seed_overlay,roundtrip,sniff}.rs`
- `crates/mnemonic-toolkit/tests/cli_compare_cost.rs`
- `crates/mnemonic-toolkit/tests/cli_xpub_search_{path,passphrase,account,address}_of_*.rs`

---

## Disposition

PR #26 already squash-merged at `66c8a56`. Findings tracked via FOLLOWUPs filed in this cycle (`v0.27.0` cycle close):

- `pr-26-roundtrip-warning-suppression` — C1 + I7 (load-bearing silent-failure class)
- `pr-26-shape-mismatch-silent-defaults` — I4 + I5 + I6 (silent-failure class)
- `pr-26-comment-rot-fold` — C2 + I8 + I9 + I10 + I11 (citation accuracy + cycle-phase vocabulary in user-visible text)
- `pr-26-test-coverage-gap-fold` — I12-I19 (overlay/select-descriptor/sniff/BSMS/multisig round-trip)
- `pr-26-type-design-anti-pattern-sweep` — I20 + I21 + recurring anti-patterns 1-6

Strategy + scoping: see "Fix-application strategy" section in this report's commit message and the per-FOLLOWUP `What:` field.
