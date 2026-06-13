# R0 round 2 — architect review (verbatim) — SPEC_restore_non_nums_taproot_internal_key.md

> Persisted verbatim per CLAUDE.md (before the fold). Dispatched via Agent tool
> (feature-dev:code-architect, inherited session default model Opus 4.8). Verdict: **YELLOW**
> (0 Critical, 2 Important, 2 Minor). Source SHA `29613f3`, spec commit `188a7d9` (post-r1-fold).
> The 2 Important findings were NEW defects introduced by the r1 I2 fold (over-broad bip388 refusal).

---

## R0 Round 2 — Architect Review — SPEC_restore_non_nums_taproot_internal_key.md

**Source SHA:** `29613f3` (spec's stated SHA; verified citations match). **Date:** 2026-06-13.

### Citation verification — all prior R0-r1 citations re-confirmed

`restore.rs:661-668` (TaprootRestore enum, no internal key yet — confirmed), `:692` classify entry, `:700` is_nums:false gate, `:719-720` MultiA/SortedMultiA Template arms, `:730` general arm, `:1207-1208` call site hard-codes Nums (CONFIRMED — both arms still `Some(TaprootInternalKey::Nums)`, this is the pre-impl source), `:1079` Multi leaf body (`Tag::Multi | Tag::SortedMulti, Body::MultiKeys { indices, ..}` — note: this line uses the wsh tags, not taproot; the taproot `MultiA`/`MultiKeys` pattern is equivalent and at the same structural location), `:1287` Display-fidelity guard. `pipeline.rs:113-156` build_tr_multi_a_descriptor, `:134-155` Cosigner arm — CONFIRMED and it already handles Cosigner faithfully. `bip388.rs:115-127` Cosigner arm — CONFIRMED. `restore.rs:796-798` comment — CONFIRMED text. Manual `:771`, `:794`, `:1027` — all confirmed, content matches the spec's stated update targets.

### Fold verification

- **I1 fold (§7: @-in-both direct construction via md_codec):** LANDED correctly. §7:146-156 now specifies construction via `md_codec::tree::Body::Tr { is_nums:false, ... }` + `encode_payload` + `chunk::split` + feed to `restore --md1`. The RED-proof is also stated correctly (§7:154-156). Technically `chunk::split` internally calls `encode_payload`, so "encode_payload → chunk::split" is redundant (split wraps encode), but this is a non-breaking implementation detail.
- **I2 fold (§6: explicit non-NUMS bip388 refusal):** LANDED with a contradiction (see IMPORTANT below).
- **m1 fold (§5: comment hygiene):** LANDED — §5 line 110-111 states the update correctly.
- **m2 fold (§7: inverting existing test :172):** LANDED — §7:141-144 calls this out clearly.
- **m3 fold (§8: manual citations):** LANDED — §8:172-173 cites `:771`, `:794`, `:1027`; all confirmed against source.
- **m4/m5 folds (§5: enum ordering note):** LANDED — §5:112-115 correctly scopes the alphabetical rule to `ToolkitError`.

### CRITICAL — None.

The §4 @-in-both structural guard: (1) index check necessary and sufficient — confirmed; dup-key-bytes-at-different-indices does not trigger false positive (Template reads SAME-INDEX semantics). (2) Display-fidelity guard cannot catch Template wrong-leaf — confirmed: `pipeline.rs:28-31` `from_str().to_string()` self-prints. (3) General/route-around arm unaffected — confirmed: it reads the full tree via `to_miniscript`, not the `leaf = all-others-by-index` shortcut. All three held after the fold.

### IMPORTANT

**I1: §6 explicit bip388 guard contradicts the P2trMulti support statement in the same paragraph.**

The I2 fold added at `restore.rs:814-820` context / §6 lines 121-133: "add an explicit guard: a taproot card whose internal key is non-NUMS (`tap_internal_key != Some(Nums)`) refuses `bip388` (and any template-requiring format)." The same paragraph then says: "the `multi_a`/`sortedmulti_a` non-NUMS multisig (`P2trMulti`) follows the same per-format support matrix as the NUMS multisig case."

These directly contradict each other. NUMS multisig supports bip388 (the Template path calls `format_bip388_wallet_policy` which handles both `Nums` and `Cosigner(idx)` — `bip388.rs:109-127`). So P2trMulti non-NUMS should ALSO support bip388 — the `Cosigner(idx)` arm at `bip388.rs:115-127` already emits `tr(@idx/**,multi_a(k,...))` faithfully. But the `tap_internal_key != Some(Nums)` guard would refuse it.

**Fix required:** The explicit bip388 guard applies ONLY inside the `template==None` branch of `build_multisig_import_payload` (the general arm, `restore.rs:832-843` vicinity). It does NOT apply to the Template arm (which takes the `Some(t)` branch and reaches `format_bip388_wallet_policy` directly). The spec must state this placement explicitly: "add the guard inside the `template==None` branch, after `script_type_from_descriptor`, alongside the existing green guard" — not as a global `tap_internal_key` check.

**I2: §7 format-output test spec incorrectly groups non-NUMS multisig with non-NUMS general-tr for bip388 refusal.**

`restore.rs (spec §7, lines 158-162):` "A non-NUMS general-tr (and non-NUMS multisig) with `--format bip388` → **refused** (explicit guard, §6)."

The parenthetical "and non-NUMS multisig" is wrong. The non-NUMS distinct-trunk multisig uses the Template path → `format_bip388_wallet_policy(..., Some(Cosigner(idx)))` → `bip388.rs:115-127` emits `tr(@idx/**,multi_a(k,...))` faithfully. This is a success case, not a refusal. The §7 test must distinguish:

- Non-NUMS general-tr + `--format bip388` → **refused** (explicit guard inside `template==None` branch).
- Non-NUMS distinct-trunk multisig + `--format bip388` → **success** (Template path + existing `Cosigner(idx)` arm; one golden cell pinning the output).

The current merged text would cause an implementer to write a refusal test for non-NUMS P2trMulti bip388 — which would force them to ADD a refusal that the architecture says should not exist, or the test would FAIL because bip388 correctly emits.

**Fix required:** Split §7 format-output test spec into two rows. Remove "and non-NUMS multisig" from the bip388-refused row. Add a success row: "Non-NUMS distinct-trunk multisig + `--format bip388` → succeeds (golden: `tr(@idx/**,multi_a(k,...))` Template path)."

### MINOR

**m1: §7 "existing pattern: `cli_standalone_bijections.rs`" overstates the precedent.**

`cli_standalone_bijections.rs` uses `md_codec::chunk::reassemble` (decodes bundle-produced strings) — it does NOT directly construct `md_codec::Descriptor` structs with `Body::Tr { is_nums:false, ... }`. The public `tree::Node` / `tree::Body` fields make construction feasible, but it is a NEW pattern in the test suite, not an existing one. The spec's "the suite already builds md1 directly" is technically incorrect. Should clarify: "construct a `md_codec::Descriptor` directly using md_codec's public struct fields (`tree::Node`, `tree::Body::Tr { is_nums:false, key_index:i, ... }`, `tree::Body::MultiKeys { indices: vec![i,...] }`) then call `md_codec::chunk::split(&d)` to get the chunks."

**m2: §6 last sentence "R0 to confirm `script_type_from_descriptor` classifies a key-path-bearing tr sanely" is a reviewer action item, not a spec statement.**

Confirmed: non-NUMS general-tr → `P2tr` (no `multi_a(` substring, `wallet_export/mod.rs:237-241`); non-NUMS multisig Template path uses `script_type_from_template` not `script_type_from_descriptor`. No silent payload emission gap exists for the Template path. The sentence should be resolved (remove "R0 to confirm" and state the confirmed outcome) since R0 is now doing round 2.

### VERDICT: YELLOW — 0 Critical, 2 Important, 2 Minor.

Required before GREEN: I1 (clarify bip388 guard placement: `template==None` branch only, not global) + I2 (split §7 test into refused-general-tr and succeeded-multisig rows, remove incorrect "and non-NUMS multisig" refusal assertion). Both are spec-text amendments only; no architecture redesign.

---

### Reviewer notes (pre-verdict observations, captured verbatim)

> Wait — these "R0 v2" annotations exist in the source. This means the SPEC's "R0 v2 I2" annotation at line 798 was added by a PREVIOUS spec/cycle (not this one). The spec says "(R0 v2 I2.)" at §5 — but the current m1 fold says to update the comment text. … The full comment at lines 796-798:
> ```
> `taproot_internal_key` is `Some(Nums)` for a taproot multisig md1 (threaded
> from the §3 classification), `None` for wsh/sh-wsh — so the `--format`
> payload's emitted descriptor carries the correct internal key. (R0 v2 I2.)
> ```
> The "(R0 v2 I2.)" appears to be an annotation from a PRIOR cycle (perhaps v0.55.1) referencing that cycle's R0 round 2, Important finding 2. The spec says this is the comment to update to "Some(Nums) or Some(Cosigner(idx))" — that's correct. The "(R0 v2 I2.)" annotation at the end would need updating too since it now refers to THIS cycle's correction. This is not a spec defect — the spec correctly identifies the file:line and says what to update. The "(R0 v2 I2.)" tail text will need to be handled during implementation. Minor at most.
