# SPEC R0 round 2 — restore per-key use-site override (opus architect, verbatim)

> Reviewer: opus architect (Plan agent; full read+bash, verified each fold against descriptor-mnemonic `c85cd49` / toolkit `4783f02`).
> SPEC: `design/SPEC_restore_per_key_use_site_override_2026-06-19.md` (post round-1 folds). **Verdict: GREEN (0C/0I).** Lone Minor M6 (dangling §5.4 ref) folded after this review by adding the real §5.6 enumeration table + fixing both references — a documentation-only fold of architect-validated content (no design change), so the R0 loop is converged at GREEN.

---

This confirms C2's rationale. Keys are placed into the tree by `lookup_key(keys, *index)` where `index` is the `@N` placeholder index from the tree node body — so in a general policy, a key's `@N` is determined by the tree's `KeyArg`/`MultiKeys` index, and `translate_pk` visits keys in TREE-TRAVERSAL order, which need not equal `@N` order (e.g., `wsh(or_d(multi(2,@0,@1),and_v(v:pkh(@2),older(144))))` — traversal order is `@0,@1,@2` here but in a reordered policy could diverge). The `ReconstructTranslator::pk` callback receives only the `DescriptorPublicKey`, NOT its `@N`, so it cannot look up the per-`@N` group positionally without a fragile correspondence. C2's requirement to source the per-`@N` keys (with groups) directly from md-codec's `expand_per_at_n` (where `@N` == Vec position, definitionally aligned to the tree's `KeyArg` indices via `lookup_key`) is the SOUND correspondence. The SPEC's C2 fold is correct and its fragility rationale is accurate.

I have completed full verification. Here is my round-2 verdict.

---

## R0 Round 2 — Verdict: GREEN

Every round-1 finding (C1, C2, I1, I2, M1–M5) is closed by its fold, the folds are individually sound against the actual code at descriptor-mnemonic `c85cd49` / toolkit `4783f02`, and the C1+C2+guard-narrowing folds together form a complete, non-contradictory funds-safety story with no remaining silent-mis-render path. No new Critical or Important was introduced. One Minor documentation gap (a dangling cross-reference) is noted below but does not gate.

### Fold-by-fold verification

**C1 — CLOSED.** `plain_template_from_tree` (`restore.rs:1140`) is the sole router for the md1 plain arm; its only `build_descriptor_string` consumer in md1 restore is `restore.rs:1336`, fully gated by `template_opt` from `:1289`. The other two `build_descriptor_string` callers — `restore.rs:387` (single-sig mnemonic-account path, no md1 TLV) and `export_wallet.rs:534` (engrave path) — never see an override card. The renderer at `pipeline.rs:85` hardcodes `/<0;1>/*` per key, structurally incapable of divergent suffixes — confirming the bug the fold avoids. Returning `None` when `use_site_path_overrides.is_some()` routes ALL override cards to the faithful arm, and breaks no non-override card (`is_some()` is false for them; the existing `:350` general-policy `@1/<0;1>/*` card emits no override since it equals baseline). C1 fold is correct and sufficient. No bypass.

**C2 — CLOSED (sound, implementable).** `ExpandedKey.use_site_path` (`canonicalize.rs:344`) carries the FULL per-`@N` `UseSitePath` including the uncollapsed `multipath: Option<Vec<Alternative>>` (`use_site_path.rs:51`); the chain-0 collapse happens only inside `use_site_to_derivation_path` (`to_miniscript.rs:116-131`), not in `expand_per_at_n`. `@N` == Vec position is guaranteed: `expand_per_at_n` loops `for idx in 0..d.n` pushing in order (`canonicalize.rs:435-465`, doc `:339`), and the tree binds keys by `@N` via `lookup_key(keys, index)` (`to_miniscript.rs:140` etc.). The fragile-`iter_pk` concern is real — `ReconstructTranslator::pk` (`restore.rs:1029`) receives only the key, never its `@N`, and `translate_pk` visits in tree-traversal order ≠ `@N` order in general policies — so sourcing the per-`@N` keys+groups from md-codec is the correct correspondence, not over-caution. The `Some`/`None` mix is first-class (`multipath: None` = bare `/*`; the `None` arm at `restore.rs:1086` already emits a single `XPub`), so the C2 reconstruction can emit `MultiXPub` for `@0` and `XPub` for `@1`. Leaving the exact API (key-set vs descriptor-builder) to the impl-plan is acceptable SPEC altitude: the SPEC pins the load-bearing decisions (data source + `@N`=position correspondence + `Some`/`None` coverage); the API form is a mechanical choice with its own per-phase R0.

**I1 — CLOSED.** The md-codec differential feeds bitcoind the `to_miniscript_descriptor` string (`bitcoind_differential.rs:681,715`) and compares to md-codec `derive_address` (`:738`) — both from the same rendering, so a divergent shape passes vacuously under the D1 bug; the only independent anchor is the BIP-84 `wpkh` golden (`:751-756`), which doesn't exercise divergence. §5.1's requirement of an INDEPENDENTLY-computed golden for the diverging cosigner (modeled on the existing `[I3c]` golden at `:748`) is the correct, adequate closure.

**I2 — CLOSED.** `use_site_to_derivation_path` independently `Err`s on a hardened alt at `to_miniscript.rs:125-127`, separate from the `:90` wildcard line. §4.1's I2 scope-correction (faithful-text holds for non-hardened + hardened-WILDCARD only; ALL hardened cases route to loud refusal via Point B before any render/derive; do NOT move the `:126` reject) is internally consistent and does not undermine the single-source-of-truth predicate.

**M1–M5 — all folded correctly:**
- M1: §2 now writes `.unwrap_or_else(|| d.use_site_path.clone())` — matches `canonicalize.rs:460`. ✓
- M2: §4.1 uses `RedundantUseSiteOverride { idx }` / `BaselineUseSiteOverride { idx }`, struct-style matching the existing `OverrideOrderViolation { prev, current }` (`error.rs:137`); the enum is concern-grouped not alphabetical, and the SPEC hedges "if the error enum is sorted" — no contradiction. ✓
- M3: `taproot_override_card(d)` is defined as ONE named predicate reused by guard AND advisory; current guard `:1247` and advisory `:81` are already the same `use_site_path_overrides.is_some()` expression, so the fold preserves exact parity. ✓
- M4: §7 now states the encoder-safety was VERIFIED — confirmed at `template.rs:204-208` (`for i in 1..n` ⇒ never `@0`; push only `if usp_i != use_site_path` ⇒ never redundant), so neither D5(a) reject fires on encoder output. ✓
- M5: D5(b) reframed as legal-and-must-be-supported (not test-only); `validate_multipath_consistency` (`validate.rs:124`) skips `None` entries so the `Some`/`None` mix passes today as a legal structure, and the SPEC correctly ties the faithful STRING to C2. ✓

### Fold-induced drift checks

- **§5.4 enumeration (re-run independently): COMPLETE and NON-CONTRADICTORY.** For every override-card shape — wsh(multi)/wsh(sortedmulti)/sh(wsh(...))/sh(multi) × standard-or-nonstandard `@0` × `Some`/`None`-mix × hardened-or-not × taproot-or-not — the narrowed guard (refuse iff `has_hardened_use_site` OR `taproot_override_card`; else proceed) yields either (a) faithful reconstruction via C1→faithful→C2, or (b) loud refusal. Hardened (baseline or override, wildcard or alt) → Point B refusal (closing the real latent gap where an override-hardened-alt currently slips past the baseline-only `derive.rs:99/110` into a generic `AddressDerivationFailed`). Taproot override → Point C refusal. `sh(multi)` returns `None` from `plain_template_from_tree` (only Wsh/Sh-Wsh matched) → faithful. No silent path remains.
- **C1↔C2 seam: closed for the round-1 proof case.** After C1 routes `wsh(multi(2,@0/<0;1>/*,@1/*))` (pinned test `:415`) to the faithful arm, C2 produces the right STRING because the per-`@N` source gives `@0 → MultiXPub(<0;1>)` and `@1 → XPub` (multipath `None`), instead of the current `self.multipath`-baseline clobber at `:1125` that would render `@1` as `<0;1>`. The corpus vector `wsh_divergent_paths` (`@1/<2;3>`, `Some`/`Some`) is the companion divergent case both oracles must cover, and the SPEC requires both.
- **No new C/I from the folds.** The new md-codec `pub fn has_hardened_use_site` (does not exist yet — confirmed) is additive ⇒ MINOR `0.37.0` is correct; toolkit pins `"0.36"` today, bump is clean. The plain-arm `None`-on-overrides return regresses no existing non-override card (predicate is false for them). The new C2 md-codec API is additive (MINOR-safe regardless of key-set vs builder form).

### Minor (non-blocking, fold if convenient)

- **M6 (new, documentation/traceability):** The SPEC cites a "**§5.4 enumeration**" as the funds-safety completeness evidence in two load-bearing places — §4.2 guard-narrowing (`SPEC:68`) and §7 risk-(c) (`SPEC:98`) — but no such enumeration exists in the SPEC. Section 5 is "Test / oracle strategy" and its item 4 is the guard/parity TESTS, not a shape enumeration. The enumeration's CLAIM is true (I reconstructed it above and it holds), so this is a dangling reference, not a funds-safety hole — but since the SPEC leans on it as the gate evidence, either add the explicit shape-by-shape table as a real §5.4 (or §4.2 sub-table) or change both citations to point at the test matrix in §5 item 4. Minor.

### Bottom line

**GREEN.** 0 Critical, 0 Important. All round-1 findings closed; folds verified sound against current source; the funds-safety story (route-all-overrides-to-faithful + faithful-per-`@N`-reconstruction + narrowed-refuse-on-hardened/taproot) is complete with no silent-mis-render path. The lone Minor (M6 dangling §5.4 reference) is documentation-only and does not gate.

---

## Post-review fold note (this session)
M6 folded: added the real **§5.6 — Funds-safety shape enumeration** table (the architect's own validated reconstruction, verbatim in substance) and updated both `§5.4` references → `§5.6` (`SPEC` §4.2 guard bullet + §7 risk-(c)). No design change. SPEC is R0-GREEN/converged.
