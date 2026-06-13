# Plan-doc R0 round 1 — architect review (verbatim) — PLAN_restore_non_nums_taproot_internal_key.md

> Persisted verbatim per CLAUDE.md. Dispatched via Agent tool (feature-dev:code-architect, inherited
> session default model Opus 4.8). **Verdict: GREEN (0 Critical, 0 Important, 4 Minor).** Source SHA
> `29613f3`, plan commit `fae21a1`. The plan-doc R0 gate is MET. The 4 Minors (m1 RED-proof exit-code
> clarity, m2 5-field `Descriptor` construction, m3 N6 prose, m4 FOLLOWUPS placement) were folded into
> the plan before proceeding. The review body includes the architect's pre-verdict working notes where it
> self-downgraded several candidate Importants to Minor after reading the source — preserved verbatim.

---

Now I have enough to render the full review. Let me also verify the exact `validate_placeholder_usage` behavior with an `@-in-both` shape — specifically whether `key_index=0` appearing in `Body::Tr` AND `index=0` appearing in `Body::MultiKeys` causes a canonicalization / duplicate-first-occurrence error.

The `walk_for_placeholders` for `Body::Tr { is_nums: false, key_index: 0 }` marks `seen[0] = true`, pushes `0` to `first_occurrences`. Then walking the `Body::MultiKeys { indices: [0, 1] }` leaf: index 0 is already `seen[0] = true` so it's skipped; index 1 is new → `seen[1] = true`, pushed. So `first_occurrences = [0, 1]` and `n = 2` → validation passes. The `@-in-both` descriptor encodes cleanly. Good — that confirms the plan's test construction path works.

I now have everything I need to render the verdict.

---

## CRITICAL

None.

---

## IMPORTANT

[The architect drafted three candidate Important findings and, on close re-reading of the source, downgraded all of them — preserved verbatim below for the audit trail.]

**[candidate, withdrawn] — Task 2 Step 2 RED-proof may be structurally void.** The RED-proof claim is: "without the guard, restore reconstructs `multi_a(2, @1)` (dropping @0) and exits 0." This depends on the crafted `@-in-both` card reaching `classify_taproot_restore` — which requires `tlv.pubkeys` populated (the plan instructs this, per the R0-r3 m1 fold). Confirmed `encode.rs:50-52` `is_wallet_policy()` = `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())`. `encode_payload`'s `validate_placeholder_usage` passes for the `@-in-both` shape (first_occurrences=[0,1], canonical). After Task 1, `classify_taproot_restore` returns `Ok(TaprootRestore::Template(TrMultiA, Cosigner(0)))`; `build_tr_multi_a_descriptor` computes `leaf = all EXCEPT index 0` = `@1` → `tr(@0_seg, multi_a(2, @1_seg))`, exits 0. The test asserts `.code(2)` → FAIL = correct RED. **The RED-proof IS valid as stated.** The parenthetical "or refuses for an unrelated reason" is imprecise prose. Downgrade to MINOR (m1).

**[candidate, withdrawn] — `refuse_at_in_both` body/type issues.** The helper takes `leaf: &md_codec::tree::Node` and matches `Body::MultiKeys { indices, .. }` (the `..` elides `k`; confirmed `tree.rs:35-40`). `inner` (the node matching `Tag::MultiA`/`Tag::SortedMultiA`) has body `Body::MultiKeys`. `Cosigner(i)` where `i: u8`; `indices.iter().any(|&idx| idx == *i)` — all `u8`, no cast. **No compile error.** Withdrawn.

**[candidate, withdrawn] — Task 3 bip388 guard gating.** The new guard gates on `P2tr | P2trMulti`; the existing green guard gates only `P2tr`. A non-NUMS general-tr `and_v(v:pk(B),older(N))` → `P2tr` (no `multi_a(` substring); a general-tr with a `multi_a(` leaf → `P2trMulti`. Both reach the `None` branch. The plan's `P2tr | P2trMulti` correctly covers both. **Sound.** Withdrawn.

After very careful re-reading I find NO IMPORTANT defects. All code snippets compile cleanly:
- `Body::Tr { is_nums: true, tree: Some(inner), .. }` — `..` elides `key_index`. Correct.
- `Body::Tr { is_nums: false, key_index, tree: Some(inner) }` — binds both. Correct.
- `Body::Tr { tree: None, .. }` — correct.
- `TaprootInternalKey::Cosigner(*key_index)` — `key_index: u8`, `Cosigner(u8)`, no cast. Correct.
- `TaprootRestore::Template(t, ik) => (Some(t), Some(ik))` / `GeneralFaithful(ik) => (None, Some(ik))` — match the new enum.
- `refuse_at_in_both`: `Cosigner(i)` `i: u8`; `idx == *i` no cast. Correct.
- `CliExportFormat::Bip388` (`export_wallet.rs:25-26`), `WalletScriptType::P2tr | P2trMulti` (`mod.rs:168,172`) — all exist. Correct.

Task 1 Step 6 deletes `cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums` ("N2 covers the same shape"). N2's goldens are pinned in Step 4 (before Step 6), N2 passes in Step 5, so deleting the refusal test in Step 6 is correctly sequenced and safe.

---

## MINOR

**m1 — Task 2 Step 2 RED-proof prose is ambiguous about exit code.** "exits 0 (or refuses for an unrelated reason)" creates doubt. The implementer should confirm the exit code is exactly 0 (the wrong-but-successful reconstruction), not a different refusal exit code. Fix: replace the parenthetical with the explicit exit-0 expectation — "without the guard, Task 1's code produces `Template(TrMultiA, Cosigner(0))`, reconstructs `tr(<@0 seg>, multi_a(2, <@1 seg>))` (wrong multisig, one cosigner dropped), and exits 0; the `.code(2)` assertion therefore fails, which IS the RED."

**m2 — `build_at_in_both_descriptor()` scaffold is incomplete for `n` / `path_decl` / `use_site_path`.** The plan correctly says "read the crate first," but the skeleton shows only `tree` and `tlv.pubkeys`. From `encode.rs:17-28`, `Descriptor` has FIVE public fields: `n`, `path_decl`, `use_site_path`, `tree`, `tlv`. A minimal valid `n=2` construction needs all five (standard multipath `path_decl` + `use_site_path`). The "read the crate first" instruction covers it, but an implementer constructing an incomplete literal will hit a compile loop. Fix: enumerate all five required fields in the skeleton with a pointer to the canonical standard-multipath values.

**m3 — Task 3 Step 2's "N6 may currently EMIT" is imprecise.** After Task 1, the non-NUMS distinct-trunk multisig takes `template = Some(TrMultiA)` → the `Some(t)` branch at `:828` → never reaches the `None` branch → the Task-3 bip388 refusal can't affect it. So N6 already succeeds after Task 1 and is NOT a RED for Task 3; N5 is the actual RED. The plan's hedged "may … likely passes already" should state this certainly.

**m4 — §8 FOLLOWUPS: the plan does not specify which FOLLOWUPS.md section the entries belong to.** The file has an open/resolved structure (verify at impl time). Fix: instruct the implementer to read the file's section structure first and place the RESOLVED + new-open entries in the restore/taproot sub-cluster per the existing conventions.

---

VERDICT: GREEN
