<!-- VERBATIM opus-architect Phase-2 per-phase review, round 1, descriptor-builder gate. Persisted per CLAUDE.md. gate.rs @ pre-I1-fold; source b596d3f, miniscript 95fdd1c. Verdict: RED 0C/1I/3m. Step-4 plan() cut BLESSED. -->

# Phase-2 Per-Phase Architect Review — `descriptor_builder/gate.rs` — **RED** (0C / 1I / 3 Minor)

One Important finding blocks GREEN: the step-4 cap's hash count diverges from `cost::enumerate`, breaking the SPEC §3-step-4 "counts MUST agree with enumerate" invariant in the dangerous direction (gate passes a tree the Phase-3 preview refuses). The funds-footgun core (steps 1–3, localization, fail-closed) is sound, and the step-5 `plan()` cut is correct and blessed.

## CRITICAL
None. No path returns `Ok` for a sanity-unsafe tree; no attacker-controlled input panics; the emitted descriptor is always `sanity_check`-clean.

## IMPORTANT

### I1 — Step-4 hash count diverges from `cost::enumerate` → gate passes trees the Phase-3 preview refuses
`gate.rs hash_and_timelock_counts` dedups hashes (`BTreeSet<String>` keyed `"{kind}:{hash}"`) → `n_hashes = distinct digests`. `enumerate` does NOT dedup: `walk_segv0_for_hash_leaves` (`enumerate.rs:406-407, 422-444`) unconditionally `push`es every hash leaf into a `Vec` and `n_hashes = assets.hashes.len()` (`enumerate.rs:90`) = total leaf count. (Keys agree — both `BTreeSet<DescriptorPublicKey>`. Timelocks agree exactly.) Consequence: same digest in ≥2 leaves → gate undercounts. For one key + one digest twice: gate raw = `2^(1+1)×1 = 4`, enumerate raw = `2^(1+2)×1 = 8`. At cap=4 the gate returns Ok while the Phase-3 cost preview trips `ConditionsTooMany` — the review surface vanishes for a policy the gate blessed. Violates the explicit SPEC invariant + the "always-previewable envelope". Important (not Critical): the emitted descriptor is still sanity-clean — no funds at risk — but it's a broken previewability contract over a constructible input. The existing agreement test uses a zero-hash policy so never caught it. Fix: mirror enumerate's asymmetry — count hash leaves (drop the BTreeSet); add a dup-digest agreement regression cell. Do NOT change enumerate.

## MINOR
- **M1** — `Malleable` and `ResourceLimit` sanity kinds have no RED cell. Mappings are source-correct and ride the same `localize()` path as the 3 tested kinds. A malleable cell is the cheapest win; resource-limit-under-sane-cap is impractical (note). Non-blocking.
- **M2** — gate's two-origins-same-xpub key-dedup (R0-r2 M1) asserted but not directly tested in the gate; SPEC §9 assigns a one-xpub-two-origins cap test to Phase 3 — flag so Phase 3 doesn't drop it.
- **M3** — `localize`'s "any `Err` ⇒ defer to ancestor" collapses all errors (vs `localize_parse_failure` which narrowly matches `NonTopLevel`). Sound because step 2 already type-checked the whole tree (only `NonTopLevel` possible). Worth a one-line comment naming the invariant; fallback is fail-closed regardless.

## What passes
- **Step-5 `plan()` cut — BLESSED (correct).** `AnalysisError` doc defines unspendable-path as exactly resource-limits + timelock-mixing — both `sanity_check` rules. Type-correctness ⟹ satisfiability. Whole-tree `plan(&maximal_assets)` after steps 2+3 is tautological — could not construct a sane tree with a dead branch. Keep it cut.
- **Step-3 oracle (F1) — genuine.** Uses explicit `sanity_check()` (gate.rs:124), never `from_str`-OK. `repeated_key_passes_step2_but_step3_rejects` positively asserts `from_str`=Ok then gate rejects RepeatedKeys at root.
- **Sanity predicate mapping — exact** (all 5 match analyzable.rs; ContainsRawPkh fail-closed; exhaustive match = forcing function).
- **§3.4 localization — sound.** Post-order deepest-first; cross-branch rules land on NCA; `child_paths` mirrors `ir::children()`; B-type skip verified against miniscript source.
- **Fail-closed — holds.** Every production path returns a Diagnostic; `.expect()`/`.unwrap()` only in `#[cfg(test)]`.
- **Step 1 field validation — correct + collected** (older `1≤N<2³¹`, after `N≥1`, hashlock len+hex, threshold `1≤k≤n` + empty guard; collects ALL).
- **Step-4 cap (key + timelock axes) — agrees; test genuine** (DEFAULT_PREVIEW_CAP=4096 matches compare_cost default; `cap_agrees_with_enumerate_at_boundary` pins gate raw==enumerate raw==32 via run_compare_cost at boundary; overflow → fail-closed).
- **Diagnostic surface — GUI-ready** (`DiagnosticKind` snake_case Serialize + as_str; `{node_path, kind, message}` matches SPEC §4 --json).

Phase 2 NOT cleared. Fold I1 (count hash leaves + dup-digest regression cell), then re-dispatch. Minors non-blocking; M1 malleable cell cheapest, M2/M3 hygiene. Phase 3 must not begin until 0C/0I.
