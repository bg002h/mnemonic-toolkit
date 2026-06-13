# R0 round 1 — architect review (verbatim) — SPEC_restore_non_nums_taproot_internal_key.md

> Persisted verbatim per CLAUDE.md (before the fold). Dispatched via Agent tool
> (feature-dev:code-architect, inherited session default model Opus 4.8; the body's self-attribution
> line is the agent's own). Verdict: **YELLOW** (0 Critical, 2 Important, 5 Minor). Source SHA `29613f3`,
> spec commit `7f3ca75`.

---

## R0 Round-1 Review: SPEC_restore_non_nums_taproot_internal_key.md

**Source SHA:** `29613f3`; HEAD `7f3ca75`. **Date:** 2026-06-13.

### Citation integrity — ALL PASS, no drift.
`restore.rs:661-668` (TaprootRestore enum, no internal key yet), `:692` classify, `:700-706` is_nums:false gate, `:719-720` MultiA/SortedMultiA Template arms, `:730` general arm, `:1207-1208` call site hard-codes Nums, `:1079` leaf `Body::MultiKeys{indices}`, `:963-970` ReconstructTranslator Single-guard, `:1287` Display-fidelity guard; `pipeline.rs:113-156` build_tr_multi_a_descriptor (Cosigner arm `:134-155`), `:28-31` from_str→to_string; md-codec `to_miniscript.rs:161-164` is_nums:false→lookup_key, `:411-415` MultiA ok, `:423-425` SortedMultiA hard-error. All confirmed.

### CRITICAL — None.
The index-vs-key concern was investigated and RESOLVED: md-codec allows the same key bytes at two indices, but the Cosigner shortcut `leaf = key_segs \ {idx}` is only dangerous when the trunk INDEX `i` is in the leaf `indices` set (same slot). Same key bytes at different indices (@0,@1 same xpub, trunk=@0, leaf={1,2}) reconstructs `multi_a(k, @1,@2)` which IS the correct leaf (card encoded indices={1,2}). The index check is sufficient; no wrong address. The dangerous shape is `tr(@i, multi_a(k, @i, …))` — same index `i` as trunk AND in leaf — which `key_index ∈ indices` catches exactly.

### IMPORTANT

**I1: `@-in-both` refusal test has no stated construction mechanism.** `bundle --descriptor` REJECTS `tr(K0, multi_a(2,K0,K1))` at encode time (BIP-388 distinct-key gate; cf. `cli_restore_taproot.rs:168-170`). The only way to test the refusal is to construct the `md_codec::Descriptor` directly (`md_codec::tree::*` + `encode_payload` + `chunk::split`) and feed the chunks to `restore`. The suite already uses md_codec directly (`cli_standalone_bijections.rs`, etc.). **Fix:** §7 must specify this construction mechanism. The RED-proof is the crux (per §4 the guard MUST be structural classify-time).

**I2: `--format bip388` (and peers) behavior for a non-NUMS general-tr is unspecified.** The general arm sets `tap_internal_key = Some(Cosigner(idx))`, flows into `EmitInputs.taproot_internal_key`. The existing general-tr bip388 refusal (`restore.rs:814-820`) relies on the NUMS internal key being a bare x-only `Single` with no multipath — but a non-NUMS trunk is a MULTIPATH XPub (`<0;1>/*`), so `is_multipath()` is true and the NUMS-specific refusal would NOT auto-fire. A non-NUMS general-tr might silently emit a bip388 payload. **Fix:** §6/§7 must state the policy — either explicitly refuse (`script_type==P2tr && tap_internal_key != Some(Nums)`) or accept + test-pin. The spec must not be silent.

### MINOR
- **m1:** §5 — `restore.rs:796-798` comment ("taproot_internal_key is Some(Nums)") must update to "Some(Nums) or Some(Cosigner(idx))."
- **m2:** §7 — the existing refusal test `cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums` (`cli_restore_taproot.rs:171-182`, `tr(K2, multi_a(2,K0,K1))`) INVERTS from exit-2 refusal to a golden-asserting SUCCESS (distinct-trunk now reconstructs). Most impactful existing-test mutation; call it out.
- **m3:** §8 — cite the current `41-mnemonic.md` line(s) where "non-NUMS refused" prose lives (grep-verify per CLAUDE.md).
- **m4:** §3 — `TaprootRestore` variant ordering: CLAUDE.md alphabetical rule strictly covers `ToolkitError`, not this enum; note for consistency (Template before GeneralFaithful is non-alphabetical).
- **m5:** `TaprootInternalKey` (`Cosigner`<`Nums` alphabetically) — verify its definition site + ordering; CLAUDE.md alphabetical rule applies if in `error.rs`.

### Deep verdicts (all CONFIRMED CORRECT)
- **§4 @-in-both guard:** index check sufficient (md-codec allows dup key bytes but not the same-slot trap the guard catches); Display-fidelity guard provably CANNOT catch the Cosigner wrong-leaf (Template output IS its own re-print, `pipeline.rs:28-31`) → structural classify-time guard is necessary AND sufficient.
- **§3 route-around non-NUMS end-to-end:** `is_nums:false→lookup_key` → XPub internal key → `Descriptor::new_tr` accepts → `Tr::translate_pk` routes the XPub through ReconstructTranslator's XPub arm (NOT the Single-guard) → MultiXPub `<0;1>/*`, network-corrected → `Tr::Display` renders `tr([fp/path]xpub.../<0;1>/*,<leaf>)` → survives parse→print → `derive_receive_addresses` computes Q=P+t·G for real P. No drop/mis-render.
- **§3 split routing:** MultiA/SortedMultiA single-leaf → Template; general/TapTree → route-around; exhaustive, no fall-through/mis-route. Cosigner `leaf = key_segs \ {idx}` faithful across key counts/orders (expand_per_at_n + key_segs both canonical ascending).
- **§5 NUMS regression:** only the is_nums:false arm changes; Nums variant still emits NUMS_XONLY_HEX; v0.49.1/v0.55.1 goldens byte-identical.
- **SemVer PATCH / no schema_mirror / manual / 2 FOLLOWUP slugs:** confirmed.

### VERDICT: YELLOW — gate does NOT pass. 0 Critical, 2 Important (I1, I2), 5 Minor.
Required before GREEN: I1 (specify the @-in-both md1 direct-construction test mechanism) + I2 (specify `--format bip388` behavior for non-NUMS general-tr). Both spec-amendment only; no architecture redesign.
