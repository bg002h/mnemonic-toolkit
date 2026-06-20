# IMPLEMENTATION PLAN — faithful `restore --md1` of taproot single-leaf use-site overrides (#26)

**Date:** 2026-06-19 · **SPEC (R0-GREEN):** `design/SPEC_restore_taproot_use_site_override_2026-06-19.md` + `design/agent-reports/taproot-override-spec-r0-round1-review.md`. Brainstorm: `design/BRAINSTORM_restore_taproot_use_site_override_2026-06-19.md`.
**Source SHA (grep-verified at write time):** mnemonic-toolkit `d72856f` (master, post-#25+#28); descriptor-mnemonic md-codec **0.37.0** (published). #25's API is LIVE — the plan-doc HOLD is lifted. Re-grep at execution (citation-decay).
**Versions:** mnemonic-toolkit **PATCH** `0.59.0 → 0.59.1` (capability unlock on an existing flag — no new flag/wire/`ToolkitError` variant; precedent v0.55.1/v0.55.3). **md-codec/mk-codec NO-BUMP** (the `tr(multi_a)` leg rides #25's published 0.37.0). **No GUI/manual lockstep** (no CLI surface change). **`tr(sortedmulti_a)` leg ships under the `taproot-coverage-cycle-on-miniscript-gt-13-1-0` umbrella** when crates.io rust-miniscript > 13.1.0 lands — designed here (§5), NOT implemented this cycle.

## 0. Gate + funds-safety discipline
Per-phase TDD (RED-first) + a per-phase opus R0 to **0C/0I** before ship (CLAUDE.md). Funds-safety / silent-wrong-address: the address-equivalence oracle vs an INDEPENDENT golden (SPEC D6/I1) is the gate, not exit-0. Single coupled toolkit PR — guard-narrow + classify-reroute + advisory-narrow + tests land together (mirrors #25).

## 1. As-built dependency (confirmed on master d72856f / md-codec 0.37.0)
The #26 SPEC's "free re-route" rests on #25's shipped API — all confirmed live:
- **`md_codec::to_miniscript::to_miniscript_descriptor_multipath`** (`md-codec-0.37.0/src/to_miniscript.rs:244`) builds per-`@N` keys then calls the shared **`node_to_descriptor` (`:295`)** — the same walker that handles `Tag::Tr`→`Terminal::MultiA`. Taproot-agnostic by construction → a re-routed single-leaf `tr(multi_a)` override card reconstructs faithfully with NO new md-codec code.
- **`md_codec::to_miniscript::has_hardened_use_site`** — used by the restore guard (`restore.rs:1634`).
- **`taproot_override_card(d)`** `pub(crate)` (`restore.rs:1472`) — #25's blanket taproot-override refusal predicate, used by the guard (`:1641`) AND the `TaprootUseSiteOverride` advisory (`unrestorable_advisory.rs:104`). **#26 NARROWS this.**
- **`faithful_multisig_descriptor`** (`restore.rs:1483`) — already rewired by #25 to the multipath builder; the taproot GeneralFaithful arm flows through it.
- **`classify_taproot_restore`** (`restore.rs:1086`); the `Template(CliTemplate::TrMultiA,…)` arm (`:1119`), `TrSortedMultiA` (`:1126`).

## 2. PHASE 1 — toolkit `0.59.1` (the `tr(multi_a)` leg)

### P2.1 — the single shared predicate (SPEC §3, the parity crux)
- **Impl:** add `pub(crate) fn restorable_taproot_override_card(d: &md_codec::Descriptor) -> bool` (next to `taproot_override_card`, `restore.rs:1472`) = `taproot_override_card(d)` ∧ the `Tag::Tr` leaf is a **plain `MultiA`** (NOT `SortedMultiA`) ∧ the internal key **is NUMS** (D7 — non-NUMS trunk out of scope) ∧ `!md_codec::to_miniscript::has_hardened_use_site(d)`. Inspect `d.tree` directly (a `Body::Tr{is_nums, tree:Some(leaf)}` with `leaf.tag == Tag::MultiA`), mirroring how `classify_taproot_restore:1086` already reads the tree. Re-grep the `tree::Node`/`Body::Tr`/`Tag` shape at execution.
- **TDD (RED):** truth table — `tr(NUMS,multi_a)`+override+non-hardened → true; `tr(NUMS,sortedmulti_a)`+override → false; `tr(realkey,multi_a)`+override (non-NUMS) → false; `tr(NUMS,multi_a)`+override+hardened → false; non-override `tr(NUMS,multi_a)` → false.

### P2.2 — classify-reroute (SPEC §3.2)
- **Impl:** in `classify_taproot_restore` (`:1086`), the `Template(CliTemplate::TrMultiA,…)` arm (`:1119`): when `restorable_taproot_override_card(d)` return **`GeneralFaithful`** (not `Template`) so it flows to `faithful_multisig_descriptor` (`:1483`) → the multipath builder. Non-override `tr(multi_a)` stays `Template` (fast path, unchanged). NOTE: `classify_taproot_restore` currently takes `tree: &Node`; if it lacks access to `d.tlv.use_site_path_overrides`, thread the `Descriptor` (or pass the `restorable_…` verdict computed at the call site) — re-grep the signature + caller at execution.
- **TDD:** an override `tr(NUMS,multi_a)` card classifies to `GeneralFaithful` (assert it reaches the faithful arm, not the `Template` string-builder).

### P2.3 — guard narrowing (SPEC §3.1)
- **Impl:** the restore guard (`restore.rs:1634-1641`) currently refuses iff `has_hardened_use_site(d) OR taproot_override_card(d)`. Narrow the taproot term: refuse iff `has_hardened_use_site(d) OR (taproot_override_card(d) && !restorable_taproot_override_card(d))`. (A `restorable_…` card is admitted → reaches §P2.2 reroute; every other taproot-override shape still refuses.)
- **TDD:** `tr(NUMS,multi_a)`+override+non-hardened → restore SUCCEEDS faithfully; `tr(NUMS,sortedmulti_a)`+override → REFUSES (NEGATIVE — the carve-out must not admit it); `tr(NUMS,multi_a)`+override+hardened → REFUSES; non-NUMS `tr(realkey,multi_a)`+override → REFUSES.

### P2.4 — advisory narrowing (SPEC §3.3, parity)
- **Impl:** the `TaprootUseSiteOverride` detector (`unrestorable_advisory.rs:104`) currently fires on `taproot_override_card(desc)`. Narrow to fire iff `taproot_override_card(d) && !restorable_taproot_override_card(d)` (the SAME expression as the guard's taproot term → exact parity: advisory fires ⟺ restore refuses). Update the message if it implies ALL taproot overrides are unrestorable.
- **TDD:** advisory parity per shape — silent for the now-restorable `tr(NUMS,multi_a)` override; fires for sortedmulti_a / hardened (≥1 advisory — SPEC Min-2: a hardened∩taproot card fires BOTH `HardenedWildcard` and `TaprootUseSiteOverride`, assert "refuses AND ≥1 fires") / non-NUMS.

### P2.5 — funds-safety oracle (SPEC §5 + D6)
- **(D6-B, FLOOR 1+2, default CI):** divergent-suffix `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))` card → `restore --md1` → reconstructed per-`@N` addresses == an INDEPENDENT golden (hand-written divergent descriptor through rust-miniscript `into_single_descriptors`, NOT through md-codec reconstruction — SPEC Min-3: pin it as a literal, assert it is NOT restore's own output). **Divergent `@1` suffix mandatory** (uniform = vacuous). Anchor anti-vacuity: assert `divergent != all-baseline`.
- **(D6-A, opportunistic):** extend `tests/bitcoind_differential.rs` (already has `tr-nums-multi_a-2of3`) with a divergent-suffix `tr(multi_a)` row vs Core `deriveaddresses` — `#[ignore]`/env-gated, NEVER the gate.
- **(FLOOR 3):** the §P2.3/P2.4 negative tests (sortedmulti_a / hardened / non-NUMS still refuse + advise) are the no-silent-mis-render backstop.

### P2.6 — version + ship
- toolkit `0.59.1`: `crates/mnemonic-toolkit/Cargo.toml`, BOTH READMEs (`README.md` + `crates/.../README.md` toolkit-version), `scripts/install.sh:32` self-pin, `fuzz/Cargo.lock`, `Cargo.lock`, `CHANGELOG.md`. **No manual flag change** (no new flag) — but update the manual `### Unrestorable descriptor shapes` prose (`docs/manual/src/40-cli-reference/41-mnemonic.md`): `tr(multi_a)` overrides now restorable; `tr(sortedmulti_a)` + hardened + non-NUMS still listed → `make -C docs/manual lint`. fmt: `cargo +1.95.0 fmt -p mnemonic-toolkit` then `git checkout -- …/mlock.rs` (g6). **Per-phase R0 to 0C/0I.** Ship (commit → ff master → tag `mnemonic-toolkit-v0.59.1` → push). No crates.io (toolkit ships via tag).

## 3. Consolidated test inventory (RED-first)
1. `restorable_taproot_override_card` truth table (P2.1).
2. Reroute: override `tr(NUMS,multi_a)` → GeneralFaithful (P2.2).
3. Guard: succeed/refuse matrix (P2.3).
4. Advisory parity: silent ⟺ restorable; fires ⟺ refuses (P2.4).
5. Address-equivalence: divergent `tr(NUMS,multi_a)` reconstructs == independent golden, divergent `@1`, anti-vacuity (P2.5 / D6-B).
6. Differential corpus row (P2.5 / D6-A, opportunistic).
7. Non-regression: non-override taproot restore unchanged; the wsh/sh override restore (#25) unchanged.

## 4. SemVer / locksteps
- toolkit **PATCH 0.59.1**. md-codec/mk-codec NO-BUMP. **No GUI schema_mirror** (no clap flag/dropdown change), **no manual flag-coverage** change (prose-only manual edit). No `ToolkitError` variant (reuses `ModeViolation`/`HardenedPublicDerivation`).
- Version sites per `project_toolkit_release_ritual_version_sites`. Re-run suite + (opportunistic) differential before tag.

## 5. `tr(sortedmulti_a)` leg — designed, umbrella-gated (NOT this cycle)
Ships under `taproot-coverage-cycle-on-miniscript-gt-13-1-0` (FOLLOWUPS.md umbrella) when crates.io rust-miniscript > 13.1.0 lands the `SortedMultiA` renderer:
- **md-codec** (umbrella component `md-codec-sortedmulti-a-to-miniscript-rendering-gap`): replace the `to_miniscript.rs:425` hard-`Err` with `Terminal::SortedMultiA` construction.
- **toolkit (#26 follow-on)**: extend `restorable_taproot_override_card` to admit `tr(NUMS,sortedmulti_a)` (`tr_leaf_is_plain_multi_a` → `…multi_a_or_sortedmulti_a`); the reroute + faithful arm reconstruct it identically given a working renderer; flip §P2.4 floor-4 from refuse-test to faithful-test.
- **Interim (this cycle):** `tr(sortedmulti_a)` overrides loud-refuse + advise (P2.3/P2.4), funds-safe.

## 6. Open verification items (re-grep at execution — SPEC §9 carry-forwards)
- Confirm `classify_taproot_restore`'s signature/caller for threading the `Descriptor`/override access (P2.2). **(R0-confirmed)** sole caller `restore.rs:1671` has `d` in scope; the `Template(t,ik)` arm `:1672` is where the call-site verdict (`if restorable_taproot_override_card(&d) → force faithful`) slots in, mirroring the non-taproot override path `:1675-1682` — no signature churn needed.
- Confirm the `tree::Node`/`Body::Tr`/`Tag::MultiA`/`Tag::SortedMultiA`/`is_nums` field names for the predicate helpers (P2.1). **(R0 Min-B)** the predicate MUST reuse classify's exact `Body::Tr { tree: Some(inner), .. }` destructure to read the leaf tag (and treat `tree:None` / non-`Body::Tr` as `restorable_…`=false) so its NUMS/leaf read cannot diverge from `classify_taproot_restore` (`:1088-1115`).
- **(R0 Min-A) Re-grep the MD-CODEC lines too** (not just toolkit): md-codec 0.37.0 `to_miniscript.rs` — `node_to_descriptor:295`, `Tag::Tr` arm `:314-334`, `build_nums_internal_key:344`, `MultiA` leaf `:572`, `SortedMultiA` hard-`Err` `:584-586`; `tree.rs:9-14/49-57`. Live deltas from the plan's snapshot: classify `TrMultiA` arm `:1116-1121`; `bitcoind_differential` `tr-nums-multi_a-2of3` row `:131-132` (NOT `:112-114`); FOLLOWUPS umbrella `:4190` + component `:353`.
- **(SPEC Min-1)** Cite the umbrella (`FOLLOWUPS.md`) and the component `md-codec-sortedmulti-a-to-miniscript-rendering-gap` separately.
- **(SPEC Min-2)** Frame the hardened∩taproot parity assertion as "refuse AND ≥1 advisory fires".
- **(SPEC Min-3)** Pin the floor-1 golden as a hand-written divergent literal, NOT restore's own output.
- Re-grep all line numbers (guard `:1634/1641`; classify `:1086`/arm `:1119`; `taproot_override_card:1472`; `faithful_multisig_descriptor:1483`; advisory `:104`) against the execution-time source.
