# SPEC — faithful `restore --md1` of taproot single-leaf use-site overrides (#26)

**Date:** 2026-06-19 · **UNCOMMITTED working draft** (SeedHammer freeze; design-only, freeze-safe). **Plan-doc HELD until #25 ships** (its citations need #25's as-built API).
**Brainstorm (R0-GREEN round 1):** `design/BRAINSTORM_restore_taproot_use_site_override_2026-06-19.md` + `design/agent-reports/taproot-override-brainstorm-r0-round1-review.md`.
**Source SHAs (grep-verified at write time):** mnemonic-toolkit `f7e6fca` (origin/master; local HEAD `4e21d94` = 1-ahead untouched parallel-instance docs commit), descriptor-mnemonic `c85cd49` (md-codec 0.36.0). Re-grep at plan time against the POST-#25 source (citation-decay).
**Strict downstream of #25** (`design/SPEC_restore_per_key_use_site_override_2026-06-19.md`, R0-GREEN `5e55603`, NOT yet implemented). **Sequencing:** `design/agent-reports/sequencing-25-26-28-advisory-2026-06-19.md`.
**SemVer:** mnemonic-toolkit **PATCH** (multi_a leg — guard-narrow + classify-reroute + advisory-narrow + tests; no new flag, no wire change, no `ToolkitError` variant; precedent = #25 SPEC:6 treats the analogous non-taproot guard-narrow + faithful reconstruction as PATCH). The **`tr(sortedmulti_a)` renderer-arming** ships under the `taproot-coverage-cycle-on-miniscript-gt-13-1-0` umbrella's own version when crates.io rust-miniscript > 13.1.0 lands (md-codec change). **No GUI/manual lockstep** (no CLI surface change). R0 to confirm the PATCH call.

---

## 1. Scope + class

`restore --md1` today **loud-refuses** every md1 carrying `d.tlv.use_site_path_overrides` via the blanket guard `restore.rs:1247` (`if d.tlv.use_site_path_overrides.is_some()`). #25 replaces that blanket with `has_hardened_use_site(d) OR taproot_override_card(d)` — so AFTER #25, all taproot override cards stay refused, now with a `TaprootUseSiteOverride` advisory that **explicitly cites this follow-up** (#25 SPEC:69/:97). **#26 NARROWS `taproot_override_card` so the restorable subset — non-hardened `tr(NUMS, multi_a(...))` — is admitted, re-routed, and faithfully reconstructed**, while `tr(sortedmulti_a)`, hardened, and out-of-scope taproot stay loud-refused.

**Class: funds-safety / silent-wrong-address.** A wrong per-cosigner derivation suffix → wrong keys → wrong address → lost funds. Exit-0 is NOT the gate; the independent address-equivalence oracle (§5) is. **No live bug today** — the guard keeps it funds-safe (loud refuse); #26 is a deferred-capability unlock (contrast #25, which closes a LIVE md-cli silent-wrong-address bug).

## 2. Dependency on #25 (3 as-built artifacts + 2 surfaces #26 narrows)

#26 cannot be implemented until #25 ships. It consumes, from #25:
- **`md_codec::to_miniscript_descriptor_multipath`** — the per-`@N` multipath descriptor builder. Architect-confirmed **taproot-agnostic by construction**: #25 plan P1.3 builds per-`@N` keys via `expand_per_at_n` then calls the SAME shared `node_to_descriptor(&d.tree, &keys)` walker (`to_miniscript.rs:134-179`) that already handles `Tag::Tr` (`:153-173`) → `Terminal::MultiA` leaf (`:415`). So a re-routed single-leaf `tr(multi_a)` override card reconstructs faithfully with NO new md-codec taproot code.
- **`md_codec::has_hardened_use_site(d)`** — shape-agnostic hardened-anywhere predicate (#25 Point B). Reused verbatim by #26 (D4).
- **`faithful_multisig_descriptor` rewired to the multipath builder** (#25 plan P2.2) + the per-`@N`-aware `ReconstructTranslator`. The toolkit `GeneralFaithful` arm already routes through `faithful_multisig_descriptor` (`restore.rs:1105/1344`) with a parse→print Display-fidelity guard (`:1354-1368`) — taproot general policies already flow here; #26 lands the single-leaf `tr(multi_a)` override card into that same arm.

#26 then narrows the two #25 surfaces:
- **The guard** (`restore.rs:1240-1260`, the `:1247` predicate #25 sets to `… OR taproot_override_card(d)`).
- **The advisory** `TaprootUseSiteOverride` in `unrestorable_advisory.rs` (#25 SPEC:69) — its detector IS `taproot_override_card(d)` today; #26 excludes the restorable subset.

## 3. The single shared predicate (R0 M1 — the parity crux)

Define ONE predicate, reused VERBATIM by the guard-narrow, the classify-reroute, and the advisory-narrow — so guard-admits ⟺ classify-reroutes ⟺ advisory-silent (single source ⇒ exact parity; mirrors #25 SPEC §4.2/M3):

```
fn restorable_taproot_override_card(d: &Descriptor) -> bool {
    taproot_override_card(d)              // #25: Tag::Tr root ∧ use_site_path_overrides.is_some()
        && tr_leaf_is_plain_multi_a(&d.tree)   // leaf tag == MultiA (NOT SortedMultiA)
        && tr_internal_is_nums(&d.tree)        // D7: NUMS internal key (H-point), non-NUMS trunk out of scope
        && !md_codec::has_hardened_use_site(d) // D4: hardened ⇒ unrestorable for watch-only
}
```

(`tr_leaf_is_plain_multi_a` / `tr_internal_is_nums` inspect `d.tree` directly — the predicate runs at guard-time, BEFORE `classify_taproot_restore`, so it cannot rely on classify's verdict.)

**The three wire sites + the parity invariant:**
1. **Guard** (`restore.rs:1247`, post-#25): refuse iff `has_hardened_use_site(d) OR (taproot_override_card(d) && !restorable_taproot_override_card(d))`. (A restorable card is admitted; every other taproot-override card — sortedmulti_a / non-NUMS / hardened — refuses.)
2. **Classify-reroute** (`classify_taproot_restore`, the `Template(TrMultiA,…)` arm `:728-731`): if `restorable_taproot_override_card(d)` → return `GeneralFaithful` (not `Template`) → flows to the faithful arm → #25 multipath builder. Non-override `tr(multi_a)` stays `Template` (fast string-builder path, unchanged).
3. **Advisory-narrow** (`TaprootUseSiteOverride` detector): fire iff `taproot_override_card(d) && !restorable_taproot_override_card(d)`.

**Invariant (test-enforced, §5):** for any `taproot_override_card(d)`, EXACTLY one of {reroute→faithful, loud-refuse + advisory-fires} occurs, partitioned by `restorable_taproot_override_card(d)`. The §5 floor-3 negative test is a backstop, not the sole guarantee.

## 4. Behavior matrix (every taproot override shape terminates definitively)

| Shape (override-carrying, `Tag::Tr` root) | Outcome | Mechanism |
|---|---|---|
| `tr(NUMS, multi_a(...))`, non-hardened | **Faithful reconstruct** | `restorable_…` true → classify `GeneralFaithful` → #25 multipath builder (§2) |
| `tr(NUMS, multi_a(...))`, hardened (`/*h` or hardened alt) | **Loud refuse** | `has_hardened_use_site(d)` (#25 Point B); advisory fires |
| `tr(NUMS, sortedmulti_a(...))`, any | **Loud refuse (interim)** | leaf ≠ MultiA → `restorable_…` false; advisory fires. Renderer-arming designed §8, impl rides umbrella |
| `tr(realkey, multi_a(...))` non-NUMS internal (D7) | **Loud refuse** | `tr_internal_is_nums` false → `restorable_…` false. Out of scope; gated on confirming #25's per-`@N` loop covers `is_nums=false` internal `@N` |
| `tr(...)` with `@`-in-both internal+leaf | **Structural refuse** | `refuse_at_in_both` (`restore.rs:777`) fires at classify-time (n≥3 funds-safety crux) — pre-#26, unchanged |
| Decode-level malformations | **Decode-reject** | md-codec decode (unchanged) |

No exit-0-with-wrong-address path exists for any taproot override shape.

## 5. Test inventory (all RED-first; funds-safety floors non-negotiable)

**D6 oracle — (B) default-CI primary + (A) opportunistic:**
1. **(B, FLOOR 1+2) Divergent-suffix address-equivalence, DEFAULT CI.** `bundle --descriptor 'tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))'`-style card (divergent `@1` suffix — encoded via `parse_descriptor.rs:194-201`/`make_use_site_path:223-236`, no taproot+override emit-refusal) → `restore --md1` → reconstructed per-`@N` addresses == an INDEPENDENT golden from `prop_backup_restore_roundtrip.rs::derive_receive` (`:383`) parsing a HAND-WRITTEN divergent descriptor string through rust-miniscript's own engine (the O3-style `assert_eq` `:437`). **Must use a DIVERGENT `@1` suffix** — a uniform `/<0;1>/*` test is vacuous against the silent-suffix-collapse bug. Near-zero new code (string + assert; harness exists). Independence: the derivation ENGINE is shared (trusted upstream); the reconstruction LOGIC differs and is what fails on a wrong suffix.
2. **(A, opportunistic) bitcoind second engine.** Extend `tests/bitcoind_differential.rs` (already has `tr-nums-multi_a-2of3` `:112-114`; `#[ignore]`/env-gated `:313-320`) with a divergent-suffix `tr(multi_a)` row vs Core `deriveaddresses`. NEVER the gate (a green build must not require a local bitcoind).

**Parity + refusal (FLOOR 3 — no silent-mis-render hole):**
3. `tr(NUMS, multi_a)` non-hardened override → restore SUCCEEDS faithfully (flip the #25 refuse-pin for this shape).
4. `tr(NUMS, sortedmulti_a)` override → restore REFUSES loudly (NEGATIVE test — narrowing must not admit it).
5. `tr(NUMS, multi_a)` HARDENED override (`/*h` and hardened alt) → restore REFUSES loudly (#25 Point B).
6. `tr(realkey, multi_a)` non-NUMS internal → restore REFUSES loudly (D7 out-of-scope stays refused).
7. **Advisory parity:** for each of shapes 3–6, `TaprootUseSiteOverride` advisory fires IFF restore refuses (i.e. silent for 3, fires for 4/5/6) — driven by the SAME `restorable_taproot_override_card` expression (§3).

## 6. SemVer / version sites / locksteps

- **mnemonic-toolkit PATCH** (multi_a leg). Toolkit version sites at ship: `Cargo.toml`, BOTH READMEs, `scripts/install.sh`, `fuzz/Cargo.lock`, `Cargo.lock`, CHANGELOG (per release ritual). fmt: `cargo +<pinned> fmt -p mnemonic-toolkit` then `git checkout -- …/mlock.rs` (g6).
- **No new clap flag, no new dropdown value, no wire-field change, no `ToolkitError` variant** (reuses `ModeViolation` / `HardenedPublicDerivation`) → **NO GUI `schema_mirror` and NO `docs/manual` lockstep** for the multi_a leg. (Confirm at plan time.)
- **`tr(sortedmulti_a)` renderer-arming** = md-codec change (§8), ships under the `taproot-coverage-cycle-on-miniscript-gt-13-1-0` umbrella version when crates.io rust-miniscript > 13.1.0 lands. Cross-cite, do NOT bump for it here.
- **Ordering:** strictly after #25 ships (`0.58.2`). Single coupled toolkit PR (guard-narrow + classify-reroute + advisory-narrow + tests land together, mirroring #25's "guard + advisory + parity tests MUST land together").

## 7. Sequencing / held plan-doc

- Brainstorm + SPEC proceed NOW (freeze-safe; architecture stable, independent of #25's exact signatures). **Plan-doc R0 HELD** until #25 ships and its API is grep-able (the plan must cite as-built `to_miniscript_descriptor_multipath` / `has_hardened_use_site` / the narrowed `taproot_override_card` site). → #26 becomes SPEC-ready (not impl-ready) at #25's ship.
- At plan-time, re-grep: the guard/classify/dispatch line numbers against POST-#25 source (R0 M2); the as-built `taproot_override_card` definition + advisory detector site; confirm #25 shipped the multipath builder + faithful-arm rewiring as planned.

## 8. `tr(sortedmulti_a)` renderer-arming — designed, umbrella-gated (D1/D2)

Designed here so it is plan-ready the moment upstream lands; implemented under the umbrella trigger, NOT this cycle:
- **md-codec:** replace the unconditional hard-`Err` at `to_miniscript.rs:423-428` ("rust-miniscript v13 has no `Terminal::SortedMultiA`") with `Terminal::SortedMultiA` construction (the `(Tag::SortedMultiA, Body::MultiKeys)` leaf arm), available once md-codec can drop the git-fork pin and floor on a crates.io rust-miniscript > 13.1.0 carrying #910. This is the umbrella component `md-codec-sortedmulti-a-to-miniscript-rendering-gap` (FOLLOWUPS.md:4192, option (b)).
- **toolkit (#26 follow-on under the umbrella):** once the renderer exists, extend `restorable_taproot_override_card` to admit `tr(NUMS, sortedmulti_a)` (`tr_leaf_is_plain_multi_a` → `tr_leaf_is_multi_a_or_sortedmulti_a`); the reroute + faithful arm then reconstruct it identically (the multipath builder is leaf-tag-agnostic given a working renderer). The §5 floor-4 negative test flips to a faithful-reconstruct test at that time.
- **Interim (this cycle):** `tr(sortedmulti_a)` override cards loud-refuse + advisory-fire (§4), funds-safe.

## 9. Open verification items (re-grep at plan time) + R0 carry-forwards

**Dependency / location confirmations (the held plan-doc resolves these against POST-#25 source):**
- Confirm #25 shipped `to_miniscript_descriptor_multipath` traversing `Tag::Tr`→`MultiA` and rewired `faithful_multisig_descriptor` to it (the "free re-route" is conditional on this).
- Confirm the as-built `taproot_override_card` predicate + its location (toolkit) + the `TaprootUseSiteOverride` advisory detector site, so #26's narrow edits the right single source.
- Confirm `restorable_taproot_override_card`'s leaf/internal inspection helpers against the md-codec `tree::Node` shape — R0-confirmed `Body::Tr { is_nums, key_index, tree }` (`tree.rs:49-57`) exposes the NUMS internal + the `Tag::MultiA` vs `Tag::SortedMultiA` leaf, exactly as `classify_taproot_restore:698-739` already reads it. Re-confirm field names at plan time.
- Confirm the PATCH SemVer call (R0-precedented: v0.55.1 + v0.55.3 taproot-restore unlocks were PATCH) and that no `ToolkitError` variant is needed (`ModeViolation` toolkit `error.rs:250`, `HardenedPublicDerivation` md-codec `error.rs:357` both exist).
- Re-grep all line numbers (guard `:1247`/dispatch `:1282-1290`; classify `:696`/arms `:729/:736`; `refuse_at_in_both:777`; faithful `:1105/:1344`/Display-guard `:1354-1368`; md-codec `to_miniscript.rs:53/134/153/161/415/423`) against POST-#25 source.

**SPEC-R0 Minors to fold into the plan-doc (review `design/agent-reports/taproot-override-spec-r0-round1-review.md`):**
- **(Min-1)** Cite the umbrella (`FOLLOWUPS.md:4189`) and the component `md-codec-sortedmulti-a-to-miniscript-rendering-gap` (`:352`, option-(b) discussion `:94`) SEPARATELY — §8 currently conflates them at `:4192`. Re-grep all FOLLOWUPS anchors at plan time.
- **(Min-2)** Frame the §5 floor-7 hardened∩taproot parity assertion as "restore **refuses** AND **≥1 of** {`HardenedWildcard`, `TaprootUseSiteOverride`} fires" — both advisories co-fire on the hardened-taproot intersection (the `has_hardened` row of §4 owns the refusal; benign, the advisory `Vec` permits 0..=3). Not "exactly the TaprootUseSiteOverride advisory."
- **(Min-3)** Pin the §5 floor-1 golden as a HAND-WRITTEN divergent descriptor literal (e.g. `@1` suffix `<2;3>` distinct from `@0`'s `<0;1>`), explicitly NOT sourced from restore's own output — mirror #25 plan §4-I1 anti-vacuity (a self-referential golden passes vacuously).
