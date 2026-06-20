# Brainstorm R0 round 1 — restore-md1-taproot-use-site-override-arm (#26) (opus architect, verbatim)

> Reviewer: opus architect (read+bash; toolkit `f7e6fca`/local `4e21d94`, md-codec `c85cd49` 0.36.0). **Verdict: GREEN — 0 Critical, 0 Important.** 2 Minors carried to SPEC. UNCOMMITTED (freeze). Brainstorm R0 converged round 1.

---

**Verdict: GREEN — 0 Critical, 0 Important.**

## Verified correct

**Claim 1 — the "free re-route" premise (D3/§2/§7) HOLDS.** The central architectural bet survives source inspection:
- `node_to_descriptor` (`descriptor-mnemonic/.../to_miniscript.rs:134-179`) is a SHARED tree-walker that takes a pre-built `keys: &[DescriptorPublicKey]` slice and dispatches on the top-level tag. Its `Tag::Tr` arm (`:153-173`) builds the NUMS/cosigner internal key, then walks `tree_to_taptree` (`:258-279`) → `node_to_miniscript::<Tap>` → the `Terminal::MultiA` leaf (`:411-416`). Taproot-structural traversal already exists and is key-agnostic.
- The single-path `to_miniscript_descriptor` (`:53-63`) builds `keys` via `build_descriptor_public_key(e, &d.use_site_path, chain)` — passing the SHARED baseline `&d.use_site_path` (`:60`, `:84`) to EVERY key, not the per-`@N` `&e.use_site_path`. It does NOT read `d.tlv.use_site_path_overrides`. So the gap is precisely per-key-path, NOT taproot-structural — confirmed.
- `to_miniscript_descriptor_multipath` / `has_hardened_use_site` are genuinely ABSENT at `c85cd49` (0 hits, grep-confirmed).
- **The decisive sub-question** — does #25's multipath builder traverse `Tag::Tr`/`MultiA` or only wsh? — resolves in #26's favor: #25's IMPLEMENTATION_PLAN P1.3 (line 34) builds per-`@N` keys via `expand_per_at_n` then calls **`node_to_descriptor(&d.tree, &keys)`** — the SAME shared walker that already handles `Tag::Tr`→`MultiA`. The multipath builder is taproot-agnostic *by construction* (it differs from the single-path builder only in how `keys` are built, not in the tree walk). The "free re-route" claim is therefore TRUE, conditional only on #25 shipping P1.3 as planned (and on its faithful arm `faithful_multisig_descriptor` rewiring to call the multipath builder — #25 plan P2.2). Both are #25 commitments the brainstorm correctly declares as its dependency (§5).

**Claim 2 — the classifier re-route (D3) is coherent.** Verified against `crates/mnemonic-toolkit/src/cmd/restore.rs` at `4e21d94` (== origin/master code):
- `classify_taproot_restore` `:696`; the `Template(CliTemplate::TrMultiA, …)` arm `:728-731` and `TrSortedMultiA` `:733-738` (brainstorm's `:729`/`:736` are exact).
- The `:1247` blanket override guard fires BEFORE classify (`:1247-1260` then classify `:1284`) — dispatch order confirmed (guard precedes classify).
- The `GeneralFaithful` arm flows `template_opt = None` (`:1286`) → `faithful_multisig_descriptor` (`:1344` → `:1105`), and the taproot/general branch derives addresses from the reconstructed STRING through the toolkit's own miniscript with a parse→print Display-fidelity guard (`:1354-1368`). So re-routing override `tr(multi_a)` to `GeneralFaithful` DOES reach the faithful arm → (post-#25) the multipath builder. Confirmed.

**Claim 3 — sortedmulti_a gap + umbrella (D1/D2).** `to_miniscript.rs:423-428` is the unconditional hard-`Err` ("rust-miniscript v13 has no Terminal::SortedMultiA fragment") — exact. The umbrella `taproot-coverage-cycle-on-miniscript-gt-13-1-0` exists (`design/FOLLOWUPS.md:4189`), genuinely parks the SortedMultiA renderer (component `md-codec-sortedmulti-a-to-miniscript-rendering-gap`, `:4192` (b)) on the first crates.io miniscript > 13.1.0 carrying #910+#953 (`:4193-4195`). #910 is in the fork rev `95fdd1c` already; the crates.io trigger is real and unmet.

**Claim 4 — golden oracle (D6) + independence.** `prop_backup_restore_roundtrip.rs::derive_receive` `:383` parses an arbitrary descriptor STRING via `Descriptor::from_str` then derives via rust-miniscript `into_single_descriptors` (`:386-400`). `bitcoind_differential.rs` env-gate confirmed `#[ignore]`-by-default AND skip-when-unset (`:313-320`), with the `tr-nums-multi_a-2of3` shape at `:112-114`. Floor 1 (default-CI, not bitcoind-gated) is therefore well-founded.

**Claim 5 — D7 NUMS scope is structurally correct.** `expand_per_at_n` (`canonicalize.rs:420-471`) iterates `0..d.n` — the COSIGNER placeholder table — resolving each key's `use_site_path` per-`@N` override (`:458-460`). The NUMS internal key is synthesized SEPARATELY in `node_to_descriptor`'s `is_nums:true` branch via `build_nums_internal_key()` (`to_miniscript.rs:161-165`, no origin/path/override). So #25's per-`@N` loop covers exactly the leaf keys and need NOT cover the NUMS internal key. The non-NUMS case stays refused, not silently wrong: `classify_taproot_restore`'s `refuse_at_in_both` (`restore.rs:777-795`) structurally refuses the `@-in-both` shape at classify-time (n≥3 funds-safety crux), and D7 explicitly punts the non-NUMS-trunk-with-override case as out-of-scope gated on a #25 SPEC verification item.

**Supporting facts confirmed:** floor-2 (divergent `@1`) IS achievable in the existing string-driven harness — `bundle --descriptor` of a divergent `tr(multi_a)` encodes `use_site_path_overrides` via `parse_descriptor.rs:194-201` (per-`@N` occurrence vs `@0` baseline, fully round-trippable through `make_use_site_path` `:223-236`), and there is NO taproot+override emit-refusal in `parse_descriptor`. So the override card is constructible and `derive_receive` (a hand-written divergent descriptor) is a non-vacuous golden for the divergent-suffix bug class. The brainstorm's source SHAs and line citations all match origin/master at write time (664/696/729/736/1105/1109/1247/1289 exact).

## CRITICAL
None.

## IMPORTANT
None.

## MINOR

**M1 — name the single shared guard/classify/advisory predicate explicitly at SPEC time (the parity crux).** The guard at `:1247` fires BEFORE `classify_taproot_restore` runs (`:1284`), so the narrowed guard cannot rely on classify's verdict — at guard-time it must itself distinguish admissible non-hardened `tr(multi_a)` from still-refused `tr(sortedmulti_a)`/hardened/out-of-scope by inspecting the leaf tag directly. The brainstorm flags this as an open SPEC item (§7: "narrowed `:1247` guard predicate composes... without gap") and backstops it with the floor-3 negative test, but does not yet name the ONE shared predicate that the guard-narrow, the classify-reroute, AND the `TaprootUseSiteOverride` advisory must all key off — which is exactly the "single source ⇒ exact parity" discipline #25's own SPEC mandated for its analogous case (`taproot_override_card`/`has_hardened_use_site`, #25 SPEC §4.2/M3). *Fix:* the SPEC should require a single predicate (e.g. `restorable_taproot_override_card(d)` = `Tag::Tr` root ∧ leaf is `MultiA` ∧ overrides present ∧ ¬`has_hardened_use_site`) reused verbatim by guard-narrow, classify-reroute, and advisory, so guard-admits ⟺ classify-reroutes-to-GeneralFaithful ⟺ advisory-silent — closing the gap structurally rather than relying on the negative test alone. This is a SPEC-sharpening, not a brainstorm-level false premise.

**M2 — line-number drift housekeeping.** The brainstorm cites `:1247/:1284/:1296` for the guard/classify dispatch; at `4e21d94` the dispatch match is `:1282-1290` (the `:1296` is `k_opt`, not the dispatch). The brainstorm already commits to re-grep at SPEC/plan time (§7, header) — note only, not a gate. Same for the recon's acknowledged DRIFTED-by-1 on the Template arms (now exact at `:729/:736` — the drift self-corrected).

## To turn GREEN
Already GREEN. No Critical or Important findings; the brainstorm may proceed to SPEC. Carry the two MINORs forward into the SPEC:
1. (M1) Require a single shared predicate keying guard-narrow + classify-reroute + advisory for exact parity (mirror #25's M3 single-source discipline); make the floor-3 negative test a backstop, not the sole guarantee.
2. (M2) Re-grep the guard/classify/dispatch line numbers against the post-#25 source when the plan-doc is unheld (the brainstorm already commits to this).
