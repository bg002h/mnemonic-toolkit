# R0 Review — Round 2 (convergence) — SPEC_bip388_policy_descriptor_expansion.md

> Persisted verbatim from the opus-architect R0 round-2 dispatch. Cycle D.

---

Both gating-gap items confirmed on primary source:

- **I-2(a) citation correction verified:** `descriptor_intake.rs:438-466` are exactly four `detect_shape_*` cells (`:438`, `:446`, `:451`, `:458`) plus `contains_at_n_walker` (`:466`). NONE pin `parse_bip388_json`'s substitution output. The SPEC's reclassification (line 55) is now correct against source. R1's claim was right; the fold's corrected citation is accurate.
- **Named integration test exists:** `account_of_descriptor_bip388_json_match` at `tests/cli_xpub_search_account_of_descriptor.rs:262` — the SPEC's "only pre-existing expansion coverage" claim holds.

All gating items are now on primary source. Final verdict.

---

## VERDICT: **GREEN — 0 Critical / 0 Important / 2 Minor**

The fold is correct and complete. Both gating items (I-1, I-2) landed accurately against current source (`origin/master == HEAD == 053cc1c`), and the four secondary items (M-1..M-5) are folded without introducing new contradiction, citation error, or drift. **Phase-1 implementation can begin.**

---

## CRITICAL (0)
None.

## IMPORTANT (0)
None. Both R1 Important items are correctly resolved:

- **I-1 — GREEN.** SPEC §1.1 (lines 33-36) now uses `#[serde(rename = "name")] _name: String` — byte-identical to the real source (`descriptor_intake.rs:86-87`). Clippy-clean (the `_` prefix silences `dead_code`; CI gate confirmed at `rust.yml:150` `cargo clippy --all-targets -- -D warnings`). The load-bearing rename is present AND the `deny_unknown_fields` parse-break trap is documented inline (lines 30-32). Both halves of the R1 fix landed.
- **I-2 — GREEN, all three parts (a/b/c) verified on primary source.** (a) Citation corrected to `:438-466` and reclassified as `detect_shape`-only — I read those lines: four `detect_shape_*` cells + `contains_at_n_walker`, none pinning substitution output. Correct. (b) Reword landed: "unchanged on the happy path; the malformed-`@N` error path strictly improves" (line 55), with the coverage-gain framing. (c) The Phase-1 pin cell is specified (line 105: xpub-search malformed path exits non-zero with the new `@N beyond keys_info` message).

## MINOR (2)

- **M2-a (stale heading vs. folded body — residual I-2 tension):** §1.1 heading (line 19) still reads "Phase 1 — dedup, **no behavior change**," while the I-2-folded body (line 55) correctly says the malformed-`@N` path "strictly improves." The heading is the exact overclaim I-2 corrected, left un-updated in the heading. Body is honest; heading is stale. Non-blocking — suggest "no happy-path behavior change" in the heading.
- **M2-b (M-2 error string is substring-quoted, not full):** SPEC lines 83 and 112 quote the bundle bare-key refusal as `"no [fp/path]xpub keys found"`. The full surfaced string (after `descriptor_concrete_to_resolved_slots` strips the `"import-wallet: bsms: parse error: "` prefix at `pipeline.rs:239-242`) is `"no [fp/path]xpub keys found in descriptor"`. The SPEC's quote is a faithful substring and the negative test (a `contains`-style refusal assertion) will pass — flagging only so the implementer writes a substring match, not `assert_eq!`. Non-blocking.

---

## Gating checks against source (all confirmed)

- **(a) `BipPolicyJson` clippy-clean + load-bearing rename:** Confirmed — matches `descriptor_intake.rs:85-90` exactly; rename preserved; trap documented.
- **(b) I-2 reword accurate + test cells specified:** Confirmed on primary source (`:438-466` read; named integration test located at `tests/cli_xpub_search_account_of_descriptor.rs:262`).
- **(c) M-1/M-2 correct against source:** M-1 — SPEC line 81 `SingleSigWatchOnly` n=1 / `MultisigWatchOnly` n≥2 matches `bundle.rs:1657` (`(1,false,_)`) and `:1660` (`(_,false,_)`) exactly. M-2 — error attributed to `concrete_keys_to_placeholders`/`descriptor_concrete_to_resolved_slots` (`pipeline.rs:221-225`, `:234-244`); call chain accurate.
- **(d) Ordering invariant pinned (the load-bearing one):** The load-bearing invariant is the §2 auto-detect ordering (`is_bip388_policy_shape` checked FIRST, before `is_at_n_form`/`classify_descriptor_form`). Pinned by the line-110 "Ordering cell (load-bearing)" — and it genuinely discriminates: if the pre-check ran after `is_at_n_form`, the raw policy's `@0/**` trips the `export_wallet.rs:413` refusal and the test fails. The bundle-site ordering is pinned implicitly by line 111 (a raw policy unguarded hits `classify_descriptor_form`'s `(true,true)` mixed-error, `pipeline.rs:136-138`). This is DISTINCT from the §1.1 digit-count "longest-N-first" (M-5), whose test (line 104) pins substitution *output*, not ordering — correct, since `/**` in every token makes that order moot. The two orderings are not conflated in the folded SPEC.
- **(d) RED-before-impl discipline:** §6 header states "tests RED before impl"; the load-bearing cells carry explicit RED-first notes (line 108: "today the policy trips the `:413` refusal"; line 110 ordering cell). Adequate.
- **(e) 2-phase structure + release gate consistent:** §7 release-gate (v0.48.1 markers: `Cargo.toml`, both READMEs, `install.sh` self-pin, re-run-suite-after-bump) is untouched by the fold; R1 already verified the 0.48.0 markers present at `Cargo.toml:3`, `README.md:13`, `crates/mnemonic-toolkit/README.md:9`, `scripts/install.sh:32`. No new inconsistency introduced. Phase split (Phase 1 = shared helper + dedup; Phase 2 = both wirings) is coherent and the §6 test plan maps cleanly onto it.

**Convergence confirmed: GREEN. No re-litigation of option (a). The two Minors are wording-only and do not gate.**
