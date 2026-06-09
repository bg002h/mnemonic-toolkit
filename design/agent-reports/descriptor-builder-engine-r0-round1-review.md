<!-- VERBATIM opus-architect R0 review, round 1, descriptor-builder engine (Release A, v0.50.0). Persisted BEFORE fold per CLAUDE.md convention. SPEC @ pre-fold; source SHA b596d3f, miniscript 95fdd1c. Verdict: RED 3C/2I/5m. -->

This is the decisive primary-source confirmation for pressure-test #1. The real-world canonical vault shapes in the existing test corpus **all carry explicit `v:` wrappers**: `and_v(v:pkh(@1),older(4032))`, `and_v(v:pk(@0),after(...))`. The bare `and_v(pk,after)` shorthand in SPEC §6 is **mistyped** — `and_v` requires a V-typed left arg, and `pk(K)` is type B (renders `c:pk_k`). So:

1. The wrapper burden on hand-authored fixtures is **real** — the SPEC author's own §6 prose elided the required `v:`. But it's authorable: experts who write these fixtures already get `v:` right everywhere in the existing corpus.
2. The IR is fragment-level, so fixtures must carry `{"w":"v"}` on the `pkh`/`pk` children of `and_v`. The validation gate (from_str typecheck) would reject the un-wrapped form loudly. That's a backstop, not ergonomics.

The decaying-multisig canonical shape per §6 is actually `wsh(andor(multi(...),older(N),andor(multi(...),older(N),and_v(v:pk(...),after(N)))))` — note the `v:` the SPEC's own §6 description omitted.

I have everything needed. Writing the verdict.

---

# R0 REVIEW — descriptor-builder engine (Release A, v0.50.0) — **RED**

**Verdict: RED. 3 Critical, 2 Important.** This does not pass the 0C/0I gate. The architecture is sound and ~85% of the citations are accurate, but three load-bearing reuse claims in the emit/validate path are mechanically wrong against `b596d3f` + miniscript `95fdd1c`, and the SPEC's two named "keystone" GUI-contract pieces (node-addressed diagnostics; `--allow` opt-out) describe an API surface miniscript does not expose. All are fixable by SPEC revision (no redesign), but each must be closed before implementation.

Verification basis: HEAD = `b596d3f` (matches SPEC), miniscript git rev `95fdd1c` confirmed in root `Cargo.toml:17` + `Cargo.lock:675`, compiler feature absent from lock, toolkit `Cargo.toml:3` = `0.49.1`.

---

## CRITICAL

### C1 — §4 emit "via `build_descriptor_string`" is impossible; that function cannot render a fragment tree
`wallet_export/pipeline.rs:18` `build_descriptor_string` is **not** a general descriptor emitter. Its body (`:86-104`) is a hardcoded `match template { Bip44 => "pkh(..)", WshMulti => "wsh(multi(k,..))", ... }` over the fixed `CliTemplate` enum — 10 templated shapes, all flat multisig/single-sig. It has no recursion and cannot emit `wsh(andor(...))`. SPEC §4 bullet 1 and §0's pipeline diagram both route emit "via `build_descriptor_string`"; this is factually wrong.

- **What's actually true:** §1 already specifies the correct mechanism (recursive `Display` on `PolicyNode`). Canonicalization + checksum is reachable exactly as `build_descriptor_string` does it internally at `:28-30`: `MsDescriptor::from_str(&rendered)?.to_string()`. So the engine renders the fragment string itself, then borrows only the *2-line canonicalize-via-roundtrip idiom*, not the function.
- **Fix:** §4/§0 must stop citing `build_descriptor_string` as the emitter. State: "render `PolicyNode → String` (§1); canonicalize + checksum via `from_str(&s)?.to_string()` (the idiom at `pipeline.rs:28-30`)." This also dents the recon's "~60% reuse incl. `wallet_export/pipeline.rs`" sizing — the pipeline-emit reuse is ~2 lines, not the function.
- **Note (don't over-count):** the *other* pipeline function, `descriptor_to_bip388_wallet_policy` (`:166`), **IS general** — it's string-based (`iter_pk` + longest-first `replacen`, `:183-204`) and works on any parsed `wsh(...)`. §4 bullet 2's bip388 reuse is correct. Say so explicitly.

### C2 — §3/§10-risk-3 node-addressed diagnostics are not deliverable against `sanity_check()` as the SPEC describes
This is the named load-bearing GUI-contract piece, and the API doesn't support the described mapping. At `analyzable.rs:225-239`, `sanity_check()` is a **short-circuit if/else chain returning a single, payload-less `AnalysisError`** (`:133-146`): it reports only the **first** failing rule in fixed priority (Sigless → Malleable → ResourceLimits → RepeatedPubkeys → HeightTimelock), and the variant carries **zero** sub-fragment location. There is no per-node identity to map back to the JSON path. SPEC §3 step 3 ("Map each `AnalysisError` variant → a node-addressed diagnostic") and risk #3 ("map each failure to the offending node") cannot be implemented by calling `sanity_check()` and reading the result.

- **What IS available:** the per-rule predicates are public — `requires_sig` (`:187`), `is_non_malleable` (`:190`), `within_resource_limits` (`:195`), `has_mixed_timelocks` (`:198`), `has_repeated_keys` (`:201`). Localizing a failure to a node requires **re-running the relevant predicate per subtree** (the IR knows node→subtree correspondence) — real net-new work the SPEC §3.4 acknowledges as "the fiddly part" but then under-scopes by implying a direct variant→node map exists.
- **Fix:** §3 step 3 + §3.4 must specify the actual mechanism (per-subtree predicate re-check to localize; accept that whole-tree `sanity_check` gives only first-failure + no location). Either commit to per-subtree localization as Release-A scope, or downgrade the GUI contract to "tree-level diagnostic + best-effort node hint" and say so. As written it promises precision the chosen API can't give.

### C3 — §3.3 `--allow <variant>` reviewed-opt-out has no substrate
SPEC §3 step 3 and brainstorm §3.1.2 promise an `--allow <variant>` / `ext_check`-style reviewed opt-out (e.g. deliberate cross-branch timelock mixing). `sanity_check()` takes no params and re-checks unconditionally — you cannot tell it to skip `HeightTimelockCombination`. The only real opt-out paths are (a) `Miniscript::from_str_ext(&s, ExtParams)` at **parse** time (`analyzable.rs:28-121` — `ExtParams` has exactly the per-rule toggles `top_unsafe`/`timelock_mixing`/`malleability`/etc.), or (b) decomposing into the per-rule predicates and selectively skipping. The SPEC implies an API that doesn't exist.

- **Fix:** §3 must specify `from_str_ext(ExtParams)` as the opt-out mechanism (and that it lives at the parse stage, step 2, not the sanity_check stage), mapping each `--allow X` to the matching `ExtParams` field. Or cut `--allow` from Release A.

---

## IMPORTANT

### I1 — §1 no-auto-wrapping: decision not made, and the SPEC's own §6 shorthand is mistyped
§1 explicitly defers to R0: "vet whether v1 should provide a minimal safe-wrap helper… or keep it strictly explicit." A verdict must rule. **Primary-source finding:** the existing test corpus (`cli_non_canonical_descriptor.rs:22,232`, `cli_standalone_bijections.rs:231`, `cli_cross_start_convergence.rs:430`) renders every real vault shape with explicit `v:` wrappers — `and_v(v:pkh(@1),older(4032))`, `and_v(v:pk(@0),after(...))`. But SPEC §6's decaying-multisig description writes `...and_v(pk,after)...` — which is **mistyped**: `and_v(X,Y)` needs X:V; `pk(K)` is type B (`c:pk_k`). `from_str("wsh(and_v(pk(K),after(144)))")` rejects; the `v:`-wrapped form parses. The SPEC author elided a required wrapper in the very fixture that's the IR's acceptance test.

- **Ruling:** Explicit wrappers are **authorable** for Release A's actual deliverable (expert-authored fixtures + power-user JSON, gate as backstop) — **not a tractability blocker**, RED-blocking only as a clarity defect. But the brainstorm/§claim that Release A is "the structured-composition capability the GUI needs" is **over-stated**: the GUI cannot consume the raw fragment IR without itself solving wrapper-inference (`and_v`-needs-V, `or_b`-needs-Wdu, etc.). 
- **Fix (must-address):** (a) correct §6's decaying-multisig shape to carry `v:` (`...and_v(v:pk(A),after(N))...`) so the keystone fixture is type-correct; (b) make an explicit decision: own a minimal safe-wrap helper as Release-A scope (the 2-3 unambiguous cases: `v:` under `and_v` left, `s:`/`a:` repositioning) **OR** explicitly assign wrapper-inference to the GUI cycle and soften the "capability the GUI needs" claim to "the validated emit substrate the GUI's wrapper-inference layer targets."

### I2 — §3.4/§4 cost-preview reuse skips a required multipath-split step
The builder emits multipath `wsh(M)` with `/<0;1>/*` keys. The cost engine's intake `strip::translate_descriptor` (`cost/strip.rs:20-31`) calls `desc.derive_at_index(0)` when `has_wildcard()`. But `derive_at_index` **errors on multipath descriptors** (miniscript `descriptor/mod.rs:705`: "Errors… If the descriptor contains multi-path derivations"). The existing wildcard compare-cost test (`cli_compare_cost.rs:768`) uses single-path `/0/*`, not `/<0;1>/*` — so this path is **untested for multipath** and would fail. §4 ("reuse `cost/enumerate.rs`") and §3.4 present the preview as pure reuse; it needs a net-new step: split via `into_single_descriptors()` (`mod.rs:946`) and enumerate over `[0]` (cost is path-invariant) before `translate_descriptor`/`plan()`.

- **Fix:** §4 must add the multipath→single-path projection step and state it's net-new glue (not covered by the existing `strip` path). Cheap, but real, and it nudges the sizing.

---

## What genuinely PASSES (state for credibility)

- **Pressure-test #3 — build-time cap IS well-defined and IR-computable.** `enumerate.rs:111-121` computes `2^(n_keys+n_hashes) × n_tl_states > hard_cap` from `collect_ast_assets`. Every input is derivable from the IR **before render**: count distinct keys + distinct hash leaves; classify each `older(N)`/`after(N)` against `500_000_000` (abs height vs MTP) and `TIME_LOCK_FLAG` (rel blocks vs 512s) to get `n_abs`/`n_rel ∈ 1..=3` → `n_tl_states = n_abs × n_rel`. Moving the refusal upstream is sound. **One caveat to bake into the SPEC:** the IR key-dedup must match enumerate's `BTreeSet<DescriptorPublicKey>` dedup (`enumerate.rs:374-391`) exactly, or the pre-render cap and the actual enumeration will disagree on `n_keys`.
- **Pressure-test #4 — `plan()` over a built tree works** via the established `strip::translate_descriptor` → `DefiniteDescriptorKey` path (`strip.rs:26-31`), modulo I2's multipath split. No branch-satisfiability false-positive concern beyond that: `plan()` over the asset powerset is the same logic compare-cost already ships; a dead branch surfaces as zero satisfying configs.
- **Pressure-test #6 (bip388 half) — round-trip holds.** `descriptor_to_bip388_wallet_policy` (`:166`) is shape-general; its multipath guard (`:171`) and `/<0;1>/*` suffix requirement (`:218`) match exactly what the builder emits.
- **Pressure-test #7 — disposition correct.** `0.49.1 → v0.50.0` MINOR is right for a new top-level subcommand. The flat `gui_schema.rs` (kind-collapse to `boolean`/`number`/`dropdown`/`path`/`text`, `:34-46`; composites → `"text"`) genuinely cannot express a recursive node grammar → a separate versioned `--spec-schema` gate is justified, not redundant. GUI `schema_mirror` lockstep applies to the subcommand's *flags* (`--spec`/`--network`/`--format`/`--json`/`--spec-schema`); the node-tree schema is correctly its own axis.
- **Pressure-test #8 — citations ~85% accurate.** `enumerate_minimal_conditions:82`, `plan()` at `:258`, `ConditionsTooMany` cap `:115-120`, hash walk `:422`/called `:407`, timelock walks `:451`/`:469`, dual-context `from_str` `:81-83`, `descriptor_to_bip388_wallet_policy:166`, `sanity_check`→`AnalysisError` at `analyzable.rs:225` gated on `std` (`:9`) NOT compiler — all verified accurate at `b596d3f`/`95fdd1c`.

## Pressure-test #2 — sizing/phasing ruling
**Release A is plausibly one MINOR cycle, but only after C1-C3 shrink the "reuse" and re-expose the real net-new surface.** Net-new is larger than the recon's "node-tree (de)serialize + cap + diagnostic mapping" because: emit is render-it-yourself (C1), diagnostics need per-subtree predicate re-checking (C2), and the cost path needs multipath glue (I2). The brainstorm's "ship the full fragment set so the schema freezes once" is correct and should hold — do NOT split the fragment set. But **§9's phasing has an ordering wrinkle (Minor, below)**: the keystone goldens can't be a Phase-1 gate. If the cycle feels heavy after the fixes, the cleanest defer is **node-addressed-localization precision** (ship tree-level diagnostics in A, per-node localization in a fast-follow) — not `--spec-schema`, which must freeze in A per risk #2.

---

## MINOR (fix in the same pass; non-blocking individually)

- **§6/§9 ordering wrinkle:** §9 makes the 5 fixtures a Phase-1 gate, but §6 requires goldens "captured from a verified-correct run" — which needs emit (Phase 3). "Authored" ≠ "golden-pinned." Re-sequence: author fixture *inputs* in Phase 1 (proves IR expressiveness/schema), pin descriptor+bip388 *goldens* in Phase 3 (after emit exists).
- **§3.3 missing 6th variant:** `AnalysisError` has `ContainsRawPkh` (`analyzable.rs:145`) — not listed in §3 step 3. Either map it or note it's unreachable for builder-emitted trees (it can't be — the IR has no raw-pkh node; state that).
- **Upstream typo:** the variant is `BranchExceedResouceLimits` (missing 'r', `analyzable.rs:139`). Any code/diagnostic referencing it must match the typo'd spelling. SPEC §3 spells it `BranchExceedResourceLimits` (correct English, wrong identifier).
- **Cargo.toml citation conflation:** SPEC §front-matter + recon cite `Cargo.toml:35` for the miniscript pin. `:35` of the *toolkit crate* Cargo.toml is `version = "13", default-features=false, features=["std"]`; the actual **rev pin** (`95fdd1c`) is in the **root** `Cargo.toml:17`. Two different files. Disambiguate.
- **§2 `--format`:** recommend SPEC commit to the minimal v1 set (`descriptor` + `bip388`) and defer the "reuse export-wallet emitter set" to avoid pulling more of `wallet_export` (which is template-coupled, per C1) into Release A scope.

---

### Critical Files for Implementation
- /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cost/enumerate.rs
- /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cost/strip.rs
- /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cost/translate.rs
- /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export/pipeline.rs
- /home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/95fdd1c/src/miniscript/analyzable.rs