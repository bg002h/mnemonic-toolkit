# BRAINSTORM — faithful `restore --md1` of taproot single-leaf use-site overrides (#26)

**Date:** 2026-06-19 · **UNCOMMITTED working draft** (SeedHammer freeze; design-only, freeze-safe).
**Status:** decisions LOCKED (user-approved D1–D7); pending brainstorm R0 (opus architect, 0C/0I gate).
**Source SHAs (grep-verified at write time):** mnemonic-toolkit `f7e6fca` (local HEAD `4e21d94`, 1-ahead parallel-instance docs commit — untouched), descriptor-mnemonic `c85cd49` (md-codec 0.36.0). Re-grep at SPEC/plan time (citation-decay).
**Recon:** `cycle-prep-recon-restore-md1-taproot-use-site-override-arm.md` (repo root, gitignored).
**Sequencing advisory:** `design/agent-reports/sequencing-25-26-28-advisory-2026-06-19.md` — #26 is a STRICT downstream of #25; impl held until #25 ships; brainstorm+SPEC are freeze-safe NOW.

---

## 1. What this is

`restore --md1` today **loud-refuses** every md1 card carrying per-cosigner use-site path overrides (`d.tlv.use_site_path_overrides`) via a blanket guard (`restore.rs:1247` → `ModeViolation`) that fires BEFORE `classify_taproot_restore` (`:1284/1296`). #25 (`restore_per_key_use_site_override`, SPEC+plan R0-GREEN, NOT yet implemented) narrows that blanket guard for the **wsh / sh-wsh** divergent-suffix multisig case by consuming the override TLV per `@N`. **#26 extends the same faithful reconstruction to the TAPROOT single-leaf multisig case** (`tr(NUMS, multi_a(...))` / `tr(NUMS, sortedmulti_a(...))`).

This is **funds-safety / silent-wrong-address class**: a wrong per-cosigner derivation suffix → wrong keys → wrong address → lost funds. Exit-0 is NOT the gate; an independent address-equivalence oracle is (D6).

**No live bug today** — the `:1247` guard keeps it funds-safe (loud refuse). #26 is a deferred-capability unlock, not a bug fix (contrast #25, which closes a LIVE md-cli silent-wrong-address bug).

---

## 2. The two legs (verified cost asymmetry)

| Leg | Reconstruction path | Upstream dependency | This cycle |
|---|---|---|---|
| **`tr(multi_a)` + overrides** | `node_to_descriptor` `Tag::Tr` arm already walks the tree (`to_miniscript.rs:153-173`) → `Terminal::MultiA` leaf (`:411-416`); the ONLY gap is per-key path, which #25's multipath builder fills **taproot-agnostically** | **NONE** — works today on the workspace fork-pin miniscript (`Cargo.toml:28-29`, rev `95fdd1c`; proven by the existing `tr-nums-multi_a-2of3` bitcoind corpus shape `bitcoind_differential.rs:112` + STRESS-A taproot leg `prop_backup_restore_roundtrip.rs:445`) | **SHIP** (atop #25) |
| **`tr(sortedmulti_a)` + overrides** | md-codec's renderer hard-`Err`s (`to_miniscript.rs:425`: "rust-miniscript v13 has no Terminal::SortedMultiA fragment") | **YES** — closing it "the real way" (build `Terminal::SortedMultiA`) requires md-codec to drop the git-fork pin, gated on crates.io rust-miniscript > 13.1.0 shipping #910 | **DESIGN here; impl rides the umbrella** (interim: loud refuse) |

---

## 3. Locked decisions

- **D1 — Scope = BOTH legs designed (chosen over multi_a-only / lowering).** `tr(multi_a)` ships this cycle (toolkit-only, atop #25). `tr(sortedmulti_a)` closed the "real way" (renderer-arming, NOT the per-index-sort "lowering" workaround — user rejected lowering's md-codec blast radius + per-index-sort funds-safety subtlety).
- **D2 — sortedmulti_a home = design both here; impl rides the umbrella.** This brainstorm/SPEC fully designs the `sortedmulti_a` renderer-arming + override reconstruction, but its IMPLEMENTATION cross-cites and rides the existing `taproot-coverage-cycle-on-miniscript-gt-13-1-0` umbrella trigger (FOLLOWUPS.md:4189) — fires when crates.io miniscript > 13.1.0 ships. No duplicate tracking. **"Wait for rust-miniscript as much as possible" (user goal) maps onto THIS leg only** — multi_a has nothing upstream to wait for.
- **D3 — Re-route mechanism = `classify_taproot_restore` returns `GeneralFaithful` (not `Template`) for override-carrying `tr(multi_a)` cards**, so they flow through `faithful_multisig_descriptor` → #25's `to_miniscript_descriptor_multipath` — exactly how #25's C1/C2 handle wsh. **NO bespoke per-`@N` taproot string-builder** (recon rejected `build_descriptor_string`/`build_tr_multi_a_descriptor` threading as fragile: `pipeline.rs:18/85/113` hardcode `format!("{origin}{}/<0;1>/*", s.xpub)` per slot, structurally cannot express a divergent per-`@N` suffix).
- **D4 — Hardened use-site = reuse #25's shape-agnostic `has_hardened_use_site` predicate** → `HardenedPublicDerivation` refusal (cannot derive hardened children from an xpub), identical to the wsh leg. No taproot-specific hardened logic.
- **D5 — Scope boundary = single-leaf `tr(NUMS, multi_a/sortedmulti_a(...))` Template-arm cards carrying overrides ONLY.** General / multi-leaf / depth-≥2 taproot roundtrip stays with its own umbrella FOLLOWUPs (`restore-general-and-multi-leaf-taproot-roundtrip` :4155, `restore-multisig-taproot-reconstruction` :331) — NOT expanded here. (Recon: general taproot policies already inherit #25's C2 for free via the GeneralFaithful arm; #26 is specifically the single-leaf Template arm that routes AROUND it today.)
- **D6 — Golden oracle = (B) independent rust-miniscript recompute in DEFAULT CI + (A) opportunistic bitcoind corpus row.** Primary gate (B): the golden derives expected per-`@N` addresses from a **hand-written divergent-suffix descriptor string** through rust-miniscript's own engine (the existing `prop_backup_restore_roundtrip.rs::derive_receive` :383 harness), compared against `restore`'s reconstructed output (the O3-style `assert_eq` :437). ~1 descriptor string + 1 assertion — near-zero custom code (honors the user's minimize-custom-code goal). The shared derivation ENGINE is acceptable/desirable (it's the trusted upstream; the reconstruction LOGIC is what differs and is tested). Secondary (A): extend `tests/bitcoind_differential.rs` (already has taproot shapes) with a divergent-suffix `tr(multi_a)` row as a fully-independent second-engine cross-check — **opportunistic, never the gate** (a green build must not require a local bitcoind).
- **D7 — NUMS-internal-key scope.** In-scope shape is **NUMS-internal** `tr(NUMS, multi_a(...))`: the internal key is the unspendable H-point, carries NO override → #25's per-`@N` override loop need not cover the internal key here. The **non-NUMS-internal-key** taproot case (real cosigner at the trunk that could itself carry an override) is OUT of #26 scope and gated on confirming #25's per-`@N` loop handles the `is_nums=false` internal `@N` (SPEC verification item; cross-cite the non-NUMS-taproot FOLLOWUPs).

---

## 4. Funds-safety floors (architect, non-negotiable)

1. **The independent (D6-B) golden runs in DEFAULT CI** — not `#[ignore]`/bitcoind-gated (`bitcoind_differential.rs:29` skips when env unset = the default; (A)-alone is refused as the sole oracle).
2. **The golden exercises a DIVERGENT `@1` suffix**, not uniform `/<0;1>/*` — a uniform-path test is vacuous against the exact bug class (silent per-cosigner suffix collapse). The hand-written golden must encode different suffixes per `@N`.
3. **No silent-mis-render hole.** Every override-carrying taproot shape terminates in exactly one of: faithful reconstruction (`tr(multi_a)`, tested per floors 1–2) / loud refusal (`tr(sortedmulti_a)` interim; hardened use-site; out-of-scope shapes) / decode-reject. A NEGATIVE test asserts the `sortedmulti_a`-override card STILL loud-refuses after #26 narrows the `:1247` blanket guard — narrowing must open no gap for the still-unsupported shapes.

---

## 5. Dependency + sequencing (from the advisory)

- **Strict downstream of #25.** Needs 3 #25 artifacts that do not exist yet (verified absent at `c85cd49`): `to_miniscript_descriptor_multipath`, `has_hardened_use_site`, the per-`@N`-aware `faithful_multisig_descriptor` + reduced `ReconstructTranslator`.
- **Design now (freeze-safe): brainstorm + SPEC.** The architecture is decided and stable (D1–D7) and does not depend on #25's exact as-built signatures.
- **HOLD the plan-doc** until #25 ships `0.58.2` and its API is grep-able (citation-decay discipline; the plan-doc must cite as-built signatures). → #26 becomes SPEC-ready (not impl-ready) at #25's ship — correct posture for a strict-downstream item.
- **Version (when it ships):** toolkit PATCH (the multi_a leg is a routing change + test; no flag/wire change → PATCH; e.g. `0.59.1` atop #28's `0.59.0`, or `0.58.3` if #28 hasn't landed). The `sortedmulti_a` leg ships under the umbrella cycle's own version when upstream lands.
- **No SeedHammer-fork tail step** — #26 is toolkit-internal routing; changes no wire field the SH fork mirrors.

---

## 6. Net-new surface (for the SPEC to detail)

1. **toolkit `classify_taproot_restore`** (`restore.rs:696`): override-carrying `tr(multi_a)` (`:729`) → `GeneralFaithful` instead of `Template`; override-carrying `tr(sortedmulti_a)` (`:736`) → explicit loud refusal (interim, cross-cite umbrella) unless/until the renderer is armed.
2. **The `:1247` blanket guard narrowed** to let non-hardened `tr(multi_a)` overrides through while still refusing every other override shape (floor 3 negative test).
3. **md-codec `sortedmulti_a` renderer-arming (umbrella-gated):** replace the `to_miniscript.rs:425` hard-`Err` with `Terminal::SortedMultiA` construction — designed here, implemented under the umbrella trigger.
4. **Tests:** (D6-B) default-CI divergent-suffix `tr(multi_a)` address-equivalence; (D6-A) opportunistic bitcoind row; (floor 3) `sortedmulti_a`-override loud-refuse negative; hardened-use-site refusal (D4).
5. **No new clap flag, no new wire field, no `ToolkitError` variant likely** (reuses `ModeViolation`/`HardenedPublicDerivation`; confirm at SPEC) → no GUI schema_mirror / manual lockstep for the multi_a leg. Confirm at SPEC.

---

## 7. Open SPEC-time verification items (grep at write time)

- **(R0 M1 — the parity crux, MUST land in the SPEC) Single shared predicate.** The `:1247` guard fires BEFORE `classify_taproot_restore`, so the narrowed guard cannot rely on classify's verdict — it must distinguish admissible non-hardened `tr(multi_a)` from still-refused `tr(sortedmulti_a)`/hardened/out-of-scope by inspecting the leaf tag itself. The SPEC MUST define ONE predicate — e.g. `restorable_taproot_override_card(d)` = `Tag::Tr` root ∧ leaf is `MultiA` ∧ overrides present ∧ ¬`has_hardened_use_site(d)` — reused VERBATIM by (i) the guard-narrow, (ii) the classify-reroute-to-`GeneralFaithful`, and (iii) the `TaprootUseSiteOverride` advisory, so guard-admits ⟺ classify-reroutes ⟺ advisory-silent. This closes the gap STRUCTURALLY (single source ⇒ exact parity, mirroring #25 SPEC §4.2/M3); the floor-3 negative test is a backstop, not the sole guarantee.
- Confirm #25's `to_miniscript_descriptor_multipath` traverses `Tag::Tr` → `MultiA` — **R0 confirmed conditional on #25 P1.3 shipping as planned**: P1.3 builds per-`@N` keys via `expand_per_at_n` then calls the SAME shared `node_to_descriptor(&d.tree,&keys)` walker that already handles `Tag::Tr`→`MultiA` (`to_miniscript.rs:134-179`); the multipath builder is taproot-agnostic by construction. Re-confirm against #25's as-built source when the plan-doc is unheld.
- **(R0 M2) Re-grep** `classify_taproot_restore` arm line numbers + the guard/classify dispatch (at `4e21d94` the dispatch match is `:1282-1290`, not `:1296`; Template arms now exact at `:729/:736`) against the POST-#25 source when the plan-doc is unheld.
- Confirm the narrowed `:1247` guard predicate composes with #25's `has_hardened_use_site` / `taproot_override_card` predicate without double-counting / gap (subsumed by the single-predicate discipline above).
- Confirm no `ToolkitError` variant needed (reuses `ModeViolation`/`HardenedPublicDerivation`; vs #28, which adds one).
