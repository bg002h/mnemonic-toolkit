# R0 Review — toolkit Check-PkK non-tap canonical fix (round 1)

Reviewer: Fable 5 architect agent (ac07b0cdb683deaa8), 2026-06-12.
Target: design/BRAINSTORM_check_pkk_non_tap_canonical_fix.md @ toolkit master (da5c162).
Persisted verbatim per CLAUDE.md convention.

## Verdict: YELLOW

One Important: the differential-test vacuity guards break if all 4 entries flip Diverge→Match (the plan doesn't address it). One Important: leg-1's `build-descriptor` emit path is wrong (no md1/policy_id). Both concrete + fixable; the core fix and round-trip safety are empirically sound. Fold + re-dispatch.

## Critical
- None.

## Important
- **I1 — Flipping all 4 Cycle-D entries Diverge→Match leaves ZERO Diverge entries, which TRIPS the test's own anti-vacuity guards.** `cli_cross_tool_differential.rs` has TWO guards both REQUIRING a Diverge: :370-374 `assert!(n_match>=1 && n_diverge>=1)` and :423-426 `assert!(saw_diverge, "harness vacuity: no entry actually produced a Diverge verdict")`. The 4 Diverge entries (wsh-pk:304/wsh-pkh:312/wsh-and_v:320/wsh-or_d:331) are the ONLY Diverge entries. Flipping all 4 → `n_diverge==0` → the differential test PANICS at the vacuity assertion, not passes. No other known toolkit-vs-md-cli divergence exists today (grepped FOLLOWUPS + the file; the only pinned divergence is this one). So the guards must be RESTRUCTURED to a verdict-agnostic non-vacuity check (≥1 Match AND the run actually classified ≥1 entry; drop the hard ≥1-Diverge), with a comment that the canonicity fix landed and the harness's value is now cross-tool MATCH confirmation over the formerly-divergent shapes (it still catches a FUTURE re-divergence). The FOLLOWUP body says "flip to Match" but never flagged this breakage — the plan inherited the blind spot. As written leg-3 does not compile-pass.
- **I2 — leg-1's `build-descriptor --json` emit path is WRONG: it emits no md1/wallet_policy_id.** Ran it on `wsh(pk)`: envelope = `{bip388, cost, descriptor, diagnostics}` — no md1, no policy_id; also wsh-only (`wrapper.values==["wsh"]`, can't cover sh(pk)/sh(pkh)). The correct in-suite emit path is **`bundle --descriptor <D> --network mainnet --json` → `.md1` (chunk array) → wallet_policy_id** (what the differential's `toolkit_ids()` helper uses; bundle.rs:1572 routes through parse_descriptor→walk_root→the fixed walker). For the NORMAL-suite golden, decode the toolkit's own md1 IN-CRATE via `md_codec::compute_wallet_policy_id` (NOT shell `md inspect`, which needs MD_BIN → would gate it cross-tool). Specify the in-suite decode mechanism explicitly.

## Minor
- **M1 — blast-radius omits `sh(wsh(pk))` (8th shape).** Confirmed accepted, policy_id `b21993dd…`, reaches the same Check arm → WILL change. Covered by the general rule but absent from the 7-shape list. Add it or mark the list illustrative.
- **M2 — cite `prop_backup_restore_roundtrip` as supplementary always-on coverage.** It's a NORMAL-suite (non-#[ignore]) test generating pk/pkh leaves through bundle→restore with an O2 md1 FIXED-POINT oracle (:431 `prop_assert_eq!(bundle_md1(&desc2), md1)`); round-trip-safe ⇒ stays green. BUT O2 is a fixed-POINT (self-consistent), not the absolute post-fix bytes — it'd pass even if the fix were reverted. Only leg-1's literal golden pins the absolute new value. Keep leg-1 as the absolute-value gate; cite the property test as the structural-stability gate.
- **M3 — lockstep is 6 files** (Cargo.toml:3, Cargo.lock:727, README.md:13, crate README:9, install.sh:32, CHANGELOG), not a literal "8".
- **M4 — when renaming `walk_check_kept_in_non_tap_context`, also update the 2 differential-file comments citing it by name** (cli_cross_tool_differential.rs:20 and :300 reference `…kept…:2551`).

## Wire-change test coverage assessment (the user mandate)
The 4-leg plan is the RIGHT shape (capture + round-trip + cross-tool + AST) with two implementability defects (I1, I2). Corrected, sufficient + implementable.

**The golden IS obtainable + stable.** md-cli's walker (template.rs:607-625) ALREADY collapses Check(PkK|PkH)→bare unconditionally (no tap gate) = byte-for-byte the post-fix toolkit shape. So `md encode` is the oracle for post-fix output.

**Exact goldens (frozen xpub xpub6DkF…r6KFrf, mfp 73c5da0a, m/48'/0'/0'/2'; @1 = xpub6Dzhy…BXd6Vk):**
| shape | PRE-fix policy_id | POST-fix golden | template_id (post-fix) |
|---|---|---|---|
| wsh(pk(@0)) | 9ad78e4f22021a3de93b97f8ccaffdc8 | **58d1803363f5599914a9f4ba0afa97d7** | 9208f59035e4912d4fca8182a897fafb |
| wsh(pkh(@0)) | 1f9d9e98f94a3cc5e55a42a4f9072466 | **3d6fb9a1656b02b36378645aaea9633e** | 1499fe4902eaa084c9574ed33b7fc109 |
| wsh(and_v(v:pk,pk)) | (changes) | **a513edb6343f69ca59841187a567a5ee** | cb13e9cd9a18a72e538a41482f562da8 |
| wsh(or_d(pk,pk)) | (changes) | **aa4bbe01269571d7e5940f542a3b0a3c** | 247773f7bc8f1e637d2c6f6163f811c5 |

**Golden FORM: pin wallet_policy_id (+template_id), NOT raw md1-string** (toolkit emits 3 chunks, md-cli a single phrase — md1-string not cross-tool-comparable; policy_id/template_id are tool-agnostic + stable + ARE what diverged). Optionally also pin the toolkit's own post-fix md1-chunk-array as a same-tool literal (secondary).

**Emit path: `bundle --descriptor` (I2), decode in-crate. Leg-1 (in-suite literal golden) = always-on primary; leg-3 (differential, #[ignore]-gated) = cross-tool confirmation.** Plan's primacy hypothesis correct.

## Answers to open questions
1. **tap_context: SOLE consumer is the :602 gate** (everything else just threads it; entry points false@432/444/456, true@519/523). Gates nothing else → after dropping the gate the param is DEAD. Recommend REMOVE it (mechanical ~15 call sites) for the honest end-state; keep-it acceptable if churn-averse. One-arm fix either way.
2. Golden form = wallet_policy_id + template_id.
3. Emit path = `bundle --descriptor` (NOT build-descriptor).
4. **SemVer = MINOR, fully precedented.** v0.48.0 CHANGELOG:244 = "**SemVer-MINOR (wire-content change)**" with "md1 AND mk1 both shift … hence MINOR not PATCH" — identical reasoning. 0.54.4→0.55.0.
5. **Blast radius:** single `Terminal::Check` arm (:601). Fix collapses Check-over-bare-key, FALLS THROUGH to emit `Tag::Check` for Check-over-non-key (preserved, verified :621). 7-shape list materially complete; add sh(wsh(pk)) (M1). UNAFFECTED shapes confirmed (multi uses build_multi_node no-Check; tap collapses; Layer-1 wrappers don't walk).
6. **Integration tests:** grepped all wsh(pk)/sh(pk)/and_v/or_d/pkh — ONLY cli_cross_tool_differential.rs pins changing output (the 4 entries). cli_build_descriptor/compare_cost/export_wallet/import_wallet/restore_multisig/fixtures pin descriptors/cost/bip388, 0 walker-wire-byte literals. + prop_backup_restore_roundtrip (M2, always-on). The 4 AST tests: walk_wsh_pk_root:1457, walk_sh_ms_pk_root:1519, walk_check_kept_in_non_tap_context:2550, walk_pk_h_via_wsh_andor:2564 (last uses explicit c:pk_h → post-fix wsh_kids[0].tag becomes Tag::PkH directly, dropping a nesting level — invert accordingly).

## Evidence log
- Citations @ da5c162: gate :601-624 (if tap_context :602); param decl :558 sole-consumer :602; 4 unit tests 1457/1519/2550/2564 all assert Tag::Check; differential Diverge entries 304/312/320/331; Match controls wpkh/pkh/wsh-multi/tr-pk-leaf. Provenance: 6502da5 (v0.3 A.4 port), 3dfca1c (A.5 test). No rationale comment.
- md-cli template.rs:607-625 collapses unconditionally (no tap param).
- Goldens derived (existing binaries, /tmp): bundle --descriptor wsh(pk) → policy 9ad78e4f (PRE); md encode wsh(pk) @bare-xpub+fp+path → 58d18033 (POST golden); ≠. pkh pre 1f9d9e98 / golden 3d6fb9a1; and_v golden a513edb6; or_d aa4bbe01.
- Round-trip: both md1 forms md decode → identical wsh(pk(@0/<0;1>/*)); bundle --self-check exit 0. md-codec to_miniscript bare-PkK→Check(pk_k) + idempotence arm. toolkit pins md-codec "0.35"→0.35.1.
- build-descriptor --json = {bip388,cost,descriptor,diagnostics}, wsh-only (I2).
- Vacuity guards: :370-374 (n_diverge>=1) + :423-426 (saw_diverge) both require Diverge (I1).
- prop_backup_restore_roundtrip #[test] non-ignored, bundle--descriptor, O2 fixed-point :431, pk/pkh.
- Both worktrees clean.
