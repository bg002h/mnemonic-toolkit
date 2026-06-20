# PLAN R0 round 1 — restore-md1-taproot-use-site-override-arm (#26) (opus architect, verbatim)

> Reviewer: opus architect (Read+Bash; toolkit master `d72856f`, md-codec **0.37.0** registry copy). **Verdict: GREEN — 0 Critical, 0 Important.** All 7 plan citation-classes verified against live source; the Critical "free re-route" premise holds in BOTH md-codec 0.37.0 AND the as-built toolkit. 3 SPEC Minors carried. A handful of decayed line numbers noted as Minor — all already covered by the plan's own §6 re-grep discipline. 2 Minors folded.

**Verdict: GREEN — 0 Critical, 0 Important.**

## Verified correct

**Source state matches the plan header.** `git rev-parse master` = `d72856f`; md-codec pinned `"0.37"` (`Cargo.toml:36`) → `0.37.0` (`Cargo.lock:677`). #25 IS shipped (FOLLOWUPS.md:110), so "plan-doc HOLD lifted / #25 API LIVE" is accurate.

**§1 — the Critical "free re-route" premise is TRUE on BOTH sides.**
- md-codec 0.37.0: `to_miniscript_descriptor_multipath` at `to_miniscript.rs:244`; `node_to_descriptor` at `:295`. The `(Tag::Tr, Body::Tr{is_nums,key_index,tree})` arm (`:314-334`) builds the NUMS internal via `build_nums_internal_key()` (`:323`→`:344`, BIP-341 H-point), recurses `tree_to_taptree` (`:328`), then `Descriptor::new_tr`. `Tag::MultiA → Terminal::MultiA(thresh)` leaf at `:572-576`. Taproot-agnostic by construction.
- Toolkit as-built: `faithful_multisig_descriptor` (`restore.rs:1483`) ALREADY calls `to_miniscript_descriptor_multipath(d)` at `:1492` (rewired by #25). `GeneralFaithful`/`template_opt=None` → `faithful_multisig_descriptor(&d, network)` at `:1739`. A re-routed override `tr(NUMS,multi_a)` card reaches the multipath builder with NO new md-codec OR toolkit reconstruction code.
- `taproot_override_card` `pub(crate)` `restore.rs:1472`; guard refuses on `has_hardened_use_site` `:1634` + `taproot_override_card` `:1641`; advisory uses it at `unrestorable_advisory.rs:104`. `classify_taproot_restore:1086`; `Template(CliTemplate::TrMultiA,…)` `:1116-1121`; `TrSortedMultiA` `:1123-1128`.

**§P2.1 — predicate realizable from `d` at guard-time.** `tree::Node { tag, body }` (`tree.rs:9-14`); `Body::Tr { is_nums, key_index, tree: Option<Box<Node>> }` (`:49-57`). Leaf = inner `Node`; `inner.tag == Tag::MultiA` vs `Tag::SortedMultiA` distinguishes the leg (as `classify_taproot_restore:1115` already reads). All four conjuncts read off `d` BEFORE classify. `restorable_taproot_override_card` correctly ABSENT today (#26's deliverable).

**§P2.2 — the classify-reroute threading concern is REAL; the plan's fix is implementable.** `classify_taproot_restore(tree: &Node)` (`:1086`) takes ONLY the tree → cannot see overrides. Sole caller `:1671` (`match classify_taproot_restore(&d.tree)?`) has `d` in scope; the `Template(t,ik)` arm (`:1672`) is where `if restorable_taproot_override_card(&d) → force faithful` slots in — MIRRORS the existing non-taproot override path at `:1675-1682`. Verdict-at-call-site = the simplest fix, no signature churn.

**§3 parity invariant — structurally closed, no silent-wrong-address hole.** Use-site guards (`:1634`/`:1641`) fire BEFORE classify (`:1671`); no other taproot-override admit path. Same `restorable_…` expression wires guard (`:1641`), classify-reroute (`:1672`), advisory (`:104`) → partitions every `taproot_override_card(d)` exactly on R. sortedmulti_a (leaf≠MultiA), non-NUMS (`is_nums=false`), hardened all fail R → refuse at `:1641` + advise. An admitted card additionally hits `refuse_at_in_both` (`:1167`) inside classify.

**§P2.5 — oracle achievable + non-vacuous.** `derive_receive` `prop_backup_restore_roundtrip.rs:383` (rust-miniscript `into_single_descriptors:387`); taproot leg `tr_multi_desc:456`/`tr_taproot_roundtrip:472`. `bitcoind_differential.rs` carries `tr-nums-multi_a-2of3` (`:131-132`) + `#[ignore]`/env-gate (`:29`/`:330`); #25's DEFAULT-CI anti-vacuity golden block (`:468`) is the mirror. Independent-golden discipline correctly specified + constructible.

**§4/§5 SemVer + locksteps.** `0.59.0` (`Cargo.toml:3`) → PATCH `0.59.1` correct (capability unlock; no flag/wire/`ToolkitError`). Precedent v0.55.1/v0.55.3 (FOLLOWUPS.md:4163). `ModeViolation` (`error.rs:250`) + `HardenedPublicDerivation` exist. `install.sh:32` self-pin `v0.59.0`. No GUI schema_mirror. Manual edit prose-only (`41-mnemonic.md:70-72` over-lists `tr(multi_a)`); `lint.sh` step 4/6 is flag-coverage → prose edit unaffected.

**§5/§8 sortedmulti_a deferral — funds-safe, umbrella-gated.** md-codec 0.37.0 STILL hard-`Err`s `(Tag::SortedMultiA, Body::MultiKeys)` at `to_miniscript.rs:584-586` → leg cannot reconstruct now; interim loud-refuse funds-safe. Umbrella (FOLLOWUPS.md:4190) + component (`:353`) are SEPARATE entries.

**§6 — all 3 SPEC Minors carried.** Min-1 (umbrella vs component cited separately) plan `:66`; Min-2 (hardened∩taproot = "refuse AND ≥1 advisory") `:34`/`:67` — benign (advisory `Vec` 0..=3, `unrestorable_advisory.rs:87`; co-fires `HardenedWildcard:94` + `TaprootUseSiteOverride:104`); Min-3 (hand-written golden, NOT self-output) `:37`/`:68`.

## CRITICAL
None.

## IMPORTANT
None.

## MINOR
**Min-A — decayed line numbers in §1/§6 (covered by the plan's own re-grep clause).** Live deltas: classify `TrMultiA` arm `:1116-1121`; md-codec MultiA leaf `:572`; md-codec SortedMultiA hard-Err `:584-586` (plan §5 carries 0.36.0's `:425`); `bitcoind_differential` row `:131-132` (plan carries `:112-114`); FOLLOWUPS umbrella `:4190`, component `:353`. Add the md-codec `to_miniscript.rs`/`tree.rs` lines to §6's re-grep list (it enumerates toolkit lines only).

**Min-B — P2.1 predicate should reuse classify's exact `Body::Tr { tree: Some(inner), .. }` destructure** to read the leaf tag (and treat `tree:None`/non-`Body::Tr` as `restorable_…`=false), avoiding a NUMS/leaf-read divergence from classify. Implementation-hygiene; design is right.

## To turn GREEN
Already GREEN. Optionally fold the two Minors (both non-design) before dispatching the implementer — neither gates TDD start.
