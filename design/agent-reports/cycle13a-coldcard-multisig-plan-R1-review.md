# R0 REVIEW — cycle-13 Lane A PLAN-DOC (coldcard-multisig fidelity pair H11 + H14) — Round 1

**Plan:** `design/IMPLEMENTATION_PLAN_cycle13a_coldcard_multisig_fidelity.md`
**Verified against `origin/master = 9b2a8ae341e0bd7fe2a75ad8d669830d96b93ccb`** (toolkit v0.65.2), vendored `bitcoin 0.32.8`.

## VERDICT: NOT GREEN — 0 Critical / 1 Important / 3 Minor

The plan is strong: phase ordering is correct and RED-first-attributable, the I-2 sorted-slot pairing rule is precise, the Q1 arm change + #13/#13b reasoning is sound and non-vacuous, every load-bearing external protocol fact re-verified against vendored source, and (almost) every line citation re-greps clean. One **Important** finding blocks GREEN: the P1 fixture-reconciliation list is **materially incomplete** — it names 2 of the ≥4 inline fixtures that the depth-gated matrix breaks, and two of the unnamed ones flip from silent-accept to refuse / change refusal-message, which a TDD implementer following the named list would mis-handle.

---

## Citation audit — re-grepped LIVE against `9b2a8ae3` (all accurate unless noted)

Confirmed exact: H11 emit fn `:258`; template-required `:261`; format_str match `:281`; derivations slot-order `:324-328`; collapse `:329-336` (plan says `:330-336` — the `if` is `:329`, body `:330-336`; trivially fine); sortedmulti sort `:345`; single `Derivation:` push `:361`; cosigner loop `:363`; `cs.fingerprint` read `:366`; `<XFP>:` emit `:367`; jade export delegation `:46`. H14: `parse_text :168`; `shared_derivation :197`; `pending_per_cosigner_path :205`; Derivation arm stages `:243`; `<XFP>:` arm `:245-256` with `per_line_path: None :252` + defensive clear `:256`; bare-xpub consumes `:269/:273`; effective-path `:338-341`; `path_components_str :355`; `xpub_parse_result :358`; **`computed_fp :359-360` (nit folded correctly)**; `supplied_fp :361`; truth table `match :363`, Row2 `:368-380`, Row4 `:386`, Row5 `:388-398`; `effective_fp` stamp `:415`; masked consts `FP_A/B/C :945-947`. I-1: canonicalizer `:361`, parse `:368`, sort `:393`, ASSUMES comment `:395-398`, `cosigners[0].path :401`, emit `:413`, Jade delegate `:570`, idempotence baseline `:1397`. Callers: `import_wallet.rs:1447` ✓; `jade.rs:133` (`parse_coldcard_multisig_text`) ✓. **H10 guard `export_wallet.rs:126` (`WshMulti|ShWshMulti`) + `:134` (refusal) ✓.** synthesize `:597/:639/:644/:650/:898` ✓. error.rs `BadInput→1` exit map `:549` ✓, `ImportWalletParse→2` `:582` ✓. SPEC §11.4.1 table `:419-427`, buggy formula `:429` ✓. Report tick `### - [ ] H11 :715`, `### - [ ] H14 :994` ✓. Manual files both EXIST ✓.

**Protocol facts re-verified at vendored source** (`bitcoin-0.32.8/src/bip32.rs`): `Xpub.depth :111` "from the master (which is 0)"; `Xpub::identifier() :833` body hashes `self.public_key.serialize()` (the **stale** `:832` doc-comment says "chaincode" — plan correctly says cite the BODY); `Xpub::fingerprint() :840` = first 4 bytes of that. The H14 thesis (master fp unrecoverable from a depth>0 xpub; `xpub.depth==0` discriminator; the no-`&secp` `fingerprint()` the import calls) is fully sound.

---

## IMPORTANT (blocks GREEN)

### I-1. P1 fixture-reconciliation list is incomplete — ≥2 unnamed inline fixtures break, two flipping silent→refuse (funds-safety axis)

The plan (§P1, `IMPLEMENTATION_PLAN…:133`; RED-test #11) names only **two** fixtures to reconcile — `parse_xfp_header_mismatch_warns_uses_header` (`:990`) and the per-line mismatch (`:1265`/asserted `:1281`) — plus a catch-all "reconcile each `xfp_header_disagreed`/`FP_A`-citing fixture." I decoded the masked consts: **`XPUB_A`/`XPUB_B` are depth 4, `XPUB_C` depth 3** — ALL depth>0. Under the new depth-gated matrix, the real break set is at least:

1. **`parse_xfp_header_mismatch_warns_uses_header` (`:990`)** — `XFP: DEADBEEF` header + bare `XPUB_A` (depth 4) → `(Some,Some)` depth>0 → **H14-c: silent, no `xfp_header_disagreed`**. Its asserts (`xfp_header_disagreed=true`, warning present) FAIL. *Named.*
2. **`parse_per_cosigner_xfp_divergence_warns` (`:1265`)** — `CAFEBABE: XPUB_A` (depth 4) → `(Some,Some)` depth>0 → H14-c silent. Asserts (`xfp_header_disagreed`, warning) FAIL. *Named.*
3. **`parse_no_header_no_per_cosigner_xfp_uses_computed_silent` (`:1042`)** — bare `XPUB_A/B/C` (depth 4) under shared `Derivation:`, **no XFP anywhere** → `(None,Some)` depth>0 → **H14-b: now REFUSES (exit 2)**. The fixture asserts it parses silently and returns `FP_A/B/C`. **FAILS — and it is NOT named.** This is the exact silent→refuse flip the lane is built on; reconciling it requires either re-pointing to depth-0 xpubs (so Row-4-at-depth-0 stays valid) or converting it to a refusal assertion. A TDD implementer following the named list would not know this fixture's *correct* post-change behavior is a refusal.
4. **`parse_heterogeneous_coin_type_rejected` (`:1418`)** — bare `XPUB_A` (depth 4) + bare `tpub_a` (depth 4), no XFP → `(None,Some)` depth>0. The per-cosigner truth-table loop (`:336-431`) runs **before** the coin-type heterogeneity check (`:438-444`), so it now **refuses with the H14-b master-fingerprint message** before reaching the coin-type validation. Its assertion `msg.contains("must share a coin-type")` FAILS. **NOT named.** Fix: supply per-line XFPs (or depth-0 xpubs) so the coin-type path is still exercised.

(For completeness — fixtures that survive: every `{FP_A}: {XPUB_A}` line and the on-disk `coldcard-ms-*` fixtures + `parse_testnet_path…` supply a per-line XFP that **equals** computed, so `(Some,Some)` depth>0 → H14-c silent, same outcome → stay GREEN. The canonicalizer cluster `:1410/:1428/:1445/:1485` feeds those homogeneous blobs and also stays GREEN.)

**Required:** P1 must enumerate fixtures #3 (`:1042`) and #4 (`:1418`) explicitly, state each one's *correct* post-change outcome (#3 → refuse OR re-point to a depth-0 xpub set; #4 → supply XFPs to keep the coin-type assertion reachable), and — because no depth-0 multisig xpub constants currently exist in the test module (`XPUB_A/B/C` are depth 3-4) — **mandate adding depth-0 xpub fixture constants** for the "Row-2/Row-4 stays meaningful at depth 0" rewrites (#1, #2, the H14-d/#10 guard, #9, #13b). Without depth-0 xpubs available, "those must now use depth-0 xpubs so the warning stays meaningful" (`:133`) is not executable. The catch-all phrasing is insufficient: #4 cites no `FP_A`/`xfp_header_disagreed` token at all.

This is *Important*, not *Critical*: the plan's per-phase full-suite gate (`cargo test -p mnemonic-toolkit`, never targeted — correctly mandated per MEMORY `feedback_r0_review_run_full_package_suite`) WOULD eventually surface #3/#4 as RED. But the plan presents the reconciliation as a 2-fixture job and gives the wrong mental model for #3/#4 (silent vs refuse), which is the precise funds-safety axis this R0 is charged to harden.

---

## MINOR (fold; non-blocking)

- **M-1. #15/#16 canonicalizer blobs must carry per-line XFPs (or depth-0 xpubs) to survive P1's re-parse.** `canonicalize_coldcard_multisig` re-parses via `parse_text` (`:368`). After P1, a divergent blob with depth>0 cosigners and **no** supplied XFP refuses at re-parse — so #15/#16 (P4) would RED for the wrong reason (refusal, not collapse). The plan says "feed a divergent-path blob" without specifying it must carry per-line `<XFP_master>:` (the shape P3 actually emits). State this explicitly so #15/#16's RED is attributable to the canonicalizer, not P1's refusal.
- **M-2. Acknowledge the broader canonicalizer test cluster as "verify-still-GREEN."** The plan names only `:1397`; `roundtrip.rs` also has `canonicalize_coldcard_multisig_with_and_without_xfp_header_match :1410`, `_3of5_stable :1428`, `_cosmetic_variants_match :1445`, `_invalid_blob_returns_parse_error :1485`. All stay GREEN (homogeneous, supplied==computed), but P4 should list them as the regression-guard set, not just `:1397`.
- **M-3. #13b per-line XFP choice.** #13b uses depth-0 xpubs with `<XFP_master>:` per-line; if those XFPs ≠ computed they trip H14-d (Row-2 warning at depth 0). #13b asserts only path resolution so it's tolerant, but the plan should note the XFPs should match computed (or the test ignore stderr) to avoid an incidental warning muddying the RED→GREEN.

---

## Confirmations against the prompt's priority axes

- **Citations (1):** all re-greped clean; the two round-2 nits (`computed_fp :359-360`, `cs.fingerprint :366`) are folded correctly; H10 guard `:126`/`:134`, canonicalizer comment `:395-398`, `synthesize_multisig_full :597`, `computed_fp :359-360`, `cs.fingerprint :366` all CONFIRMED.
- **H14 refuse-matrix (2):** the (depth, XFP) matrix is correct (depth-0/no-XFP→silent; depth>0/no-XFP→REFUSE `ImportWalletParse` exit 2; depth>0/with-XFP→silent accept; depth-0/mismatch→warn+supplied), discriminator `xpub.depth==0` sound. SPEC §11.4.1 fix in P1 ✓. The two NAMED `xfp_header_disagreed=true` fixtures (`:1012`/`:1281`) — plan correctly keeps them Row-2-meaningful by re-pointing to depth-0 (not deleting asserts) — **BUT** see I-1: it misses fixtures #3/#4 and lacks depth-0 xpub constants to execute the rewrite.
- **Q1 parser arm (3):** P2 correctly extends `<XFP>:` arm (`:245-256`) to `per_line_path: pending.take()` + delete the `:256` clear, never touching `shared_derivation`; `.take()` precludes stale-leak. #13 GREEN-before-and-after reasoning is correct; **#13b is genuinely RED-until-P2 and non-vacuous** (verified: cosigner 1 resolves to `m/A` today → RED; `m/B` after P2 → GREEN).
- **H11 emit (4):** H11-b mandates reading path+xpub+fp from the SAME sorted slot, never `derivations[i]`; sorted-only reachability via H10 (`:126`/`:134`) confirmed; #1b exercises sort≠slot-order with divergent paths. Sound.
- **I-1 canonicalizer (5):** P4 extends `:361` for heterogeneous paths with #15 (idempotence) + #16 (round-trip-verify), consuming `:1447` + `:570`. Sound modulo M-1.
- **Phase ordering / RED-first (6):** P1(H14)→P2(Q1)→P3(H11 emit)→P4(canonicalizer) genuinely makes each RED attributable; #5 headline lands in P3 after import side is correct; no phase has an un-RED-able test. ~16 tests each mapped to a phase. CONFIRMED.
- **Scope/gates/SemVer (7):** NO version bump (co-ships v0.66.0), NO `cargo fmt`, full `-p` suite + `clippy --workspace --all-targets -D warnings` per phase, NO schema_mirror, manual prose + SPEC §11.4.1 in-scope, mandatory whole-diff review note present (§6), report-tick at `:715`/`:994` with re-grep-at-ship. ALL CONFIRMED.

---

## Mixed-shape blob interaction (verified, no finding)

Consider a mixed-shape blob: shape-1 `<XFP>: <xpub>` cosigner lines interleaved with a stray `Derivation:`. Today, after each `<XFP>:` line, `pending = None` is cleared defensively (`:256`). After P2 deletes that clear and uses `.take()`, the behavior is equivalent for the `<XFP>:` arm itself (it consumes its own pending via `.take()`). The only behavioral delta: previously a `Derivation:` immediately before an `<XFP>:` line was dropped (cleared, ignored); now it's consumed as that cosigner's per-line path. That's exactly H11's round-trip requirement and is correct. No stale-leak hazard because `.take()` empties pending. The P2 change is safe — sound, no finding.

---

## Path to GREEN

Fold **I-1** (enumerate the `:1042` and `:1418` fixtures explicitly with their correct post-change outcomes — refuse vs re-point; mandate adding depth-0 multisig xpub constants since none exist today) + M-1/M-2/M-3; persist this review to `design/agent-reports/cycle13a-coldcard-multisig-plan-R1-review.md`; re-dispatch the plan-R0. The core design, phase ordering, pairing rule, and protocol grounding are sound — only the P1 fixture blast-radius needs to be made complete and executable before TDD begins.

**Not GREEN. One Important finding (I-1) blocks. Do not start coding until folded and re-reviewed to 0C/0I.**
