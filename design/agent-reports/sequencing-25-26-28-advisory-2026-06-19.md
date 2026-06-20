# Sequencing advisory — #25 / #26 / #28 (opus architect, verbatim)

> Advisory (NOT an R0 gate). Reviewer: opus architect (read+bash; toolkit `4e21d94`, md-codec `0.36.0`==`c85cd49`, mk-codec `913febc`). Corrected TWO brief premises: (1) #25 adds NO `ToolkitError` variant (uses `bad()`/`ModeViolation`) → the feared error.rs collision does not exist; (2) #25/#28 restore.rs edit regions are physically disjoint. UNCOMMITTED (freeze).

---

# Implementation Sequencing Recommendation — #25 / #26 / #28

## Verification results (vs the brief)
- **#26 ⊃ #25 (strict impl dep): CONFIRMED.** The 3 artifacts #26 needs do not exist yet — `to_miniscript_descriptor_multipath` (0 hits), `has_hardened_use_site` (0 hits); only single-path `to_miniscript_descriptor` exists (`md-codec/src/to_miniscript.rs:53`). #25 produces all three.
- **#28 independent of #25/#26: CONFIRMED.** Consumes only public md-codec 0.36 API (`compute_wallet_descriptor_template_id`); no md-codec/mk-codec bump.
- **Both add a `ToolkitError` variant: FALSE — only #28 does.** #25 plan-doc §4: "No `ToolkitError` variant (toolkit uses `bad(...)`/`ModeViolation`)." Materially lowers the merge surface.
- **#26 sortedmulti_a leg blocked on upstream miniscript: CONFIRMED** — hard `Err` at `md-codec/src/to_miniscript.rs:425`.
- **SeedHammer fork consumes the changed surface: NO for all three.**

### restore.rs region map (load-bearing)
- **#25 edits:** translator `1007–1100`, `faithful_multisig_descriptor` `1105`, override guard `1247–1254`, taproot classify/routing `1268–1289`, `1336/1365`.
- **#28 edits:** dispatch carve-out `177–179`, `run_multisig` keyless `1198–1238`, plus a NEW single-sig completion fn.
- **Physically disjoint within restore.rs.** Only true shared surfaces: (1) `error.rs` — only #28 touches it (no variant-vs-variant collision); (2) the toolkit version line (`Cargo.toml:3` + READMEs + install.sh + CHANGELOG) — the one guaranteed, mechanical conflict (both rebase off 0.58.1).

## Recommended sequence (post-freeze)

### Dependency-FORCED constraints
- **C-1:** #25 Phase 1 (md-codec 0.37.0) must PUBLISH to crates.io before #25 Phase 2 (toolkit) final-pins and before #26 impl starts.
- **C-2:** #26 impl cannot begin until #25 shipped+published (needs the 3 as-built artifacts).
- **C-3:** per-phase R0 0C/0I; #25/#28 funds-safety class → address-equivalence oracle w/ independent golden is the gate.

### Discretionary recommendation
- **Phase A — #25 first, end-to-end (funds-safety urgency overrides everything).** A1: md-codec 0.36.0→0.37.0 + md-cli 0.7.0→0.7.1, R0, PUBLISH. A2: toolkit pins published 0.37.0, ship **0.58.2** PATCH, R0.
- **Phase B — #28, rebased onto A's landed toolkit.** B1: `bundle --md1-form=template` + restore completion, branch off 0.58.2, ship **0.59.0** MINOR. B2 (parallel within B): mk-cli (mnemonic-key) MINOR + mnemonic-gui MINOR paired (schema_mirror flag-name + dropdown values).
- **Phase C — #26 after #25 ships.** C-design (DURING freeze, now): brainstorm→SPEC, HOLD plan-doc. C-impl: `tr(multi_a)`-only re-route, toolkit PATCH **0.59.1**; `tr(sortedmulti_a)` fenced (upstream-miniscript-gated).

### Version assignment (resolves the 0.58.1→ collision)
**#25 → 0.58.2 (PATCH), then #28 → 0.59.0 (MINOR), then #26 → 0.59.1 (PATCH).** Rejected alternative (#28→0.59.0 first, #25→0.59.1): would gate a live-bug funds-safety fix behind a feature MINOR's latency + inflate #25 to PATCH-atop-MINOR for no benefit.

### Parallel-vs-serial verdict for #25/#28: PARALLEL branches, #25 merges first, #28 rebases.
Cut both off origin/master. Do #25 A1→A2 to 0.58.2. #28 rebases onto landed 0.58.2. Expected rebase conflicts: restore.rs — none structural (disjoint regions/functions; only context-line proximity near run_multisig if #25's guard rewrite shifts line numbers — clean 3-way); error.rs — none; version-line files — the one guaranteed mechanical conflict (#28 re-targets 0.58.2→0.59.0). Serializing on one branch would force #28 to wait on #25's irreversible crates.io publish latency despite #28 having zero md-codec dep — wasteful.

### Funds-safety override (decisive)
#25 closes a LIVE md-cli silent-wrong-address bug (`md address <divergent-card>` derives the baseline path for an override cosigner → wrong funds). Strongest constraint; #25 ships first even though #28 is equally impl-ready + conflict-light. #26/#28 have NO live bug (#26 `restore.rs:1247` loud-refuses; #28 new feature).

### Step 5 — #26 design during freeze?
**Brainstorm + SPEC: proceed NOW** (architecture decided + stable: re-route `classify_taproot_restore`→`GeneralFaithful` for override-carrying `tr(multi_a)`; fence `tr(sortedmulti_a)`; none depends on #25's exact signatures). **Plan-doc (R0-GREEN, impl-ready): HOLD until #25's API lands** — citation-decay discipline requires grep-verifying the as-built signatures of `to_miniscript_descriptor_multipath`/`has_hardened_use_site`/reduced `ReconstructTranslator`, which don't exist until #25 ships. Draft brainstorm+SPEC + run SPEC R0 during freeze; pause before plan-doc R0; resume when #25's 0.58.2 lands. → #26 becomes SPEC-ready (not impl-ready) at #25's ship, the correct posture for a strict-downstream item.

### Step 6 — SeedHammer fork tail step
**Not applicable to any of #25/#28/#26.** SH fork mirrors md-codec/mk-codec at the BCH-constant + md1/mk1 chunk-wire level (`POLYMOD_INIT 0x23181b3`, target residue, chunk decode), NOT the toolkit restore/bundle CLI or to_miniscript reconstruction. #25 adds decode *rejects* not new wire fields (`use_site_path_overrides` TLV already in encode.rs:70 + decode.rs:57). #28 keyless-template emission uses an already-existing keyless wire form + a CLI flag. #26 is toolkit-internal routing. Nothing for SH Go to mirror.

## Could-not-verify / caveats
1. md-codec publish mechanics (crates.io ownership, lockstep `=0.37.0` pin) asserted by plan-doc; verified current versions, not a `cargo publish` dry-run.
2. #28's `Md1Form` signature-threading blast radius (~8 test callers + verify_bundle.rs/import_wallet.rs) is the plan's own round-2 Minor; re-grep at execution (line numbers shift after #25 lands).
3. Rebase line-number drift: "disjoint regions" verdict is at 4e21d94; regions stay disjoint (different functions) but #28 must re-grep anchors before rebasing.
4. Local 1-ahead `4e21d94` (parallel SH instance docs FOLLOWUP) stays untouched; re-baseline at execution.

## One-line summary
Order: **#25 (md-codec 0.37.0 publish → toolkit 0.58.2 PATCH) → #28 (toolkit 0.59.0 MINOR, rebased) → #26 (toolkit 0.59.1 PATCH, tr(multi_a)-only, sortedmulti_a fenced).** Drive #25 first on funds-safety, not just dependency. #25/#28 = parallel branches, #25 merges first; only real conflict = version line (mechanical). Start #26 brainstorm+SPEC during freeze NOW; hold its plan-doc R0 until #25's API is grep-able. No SH fork tail step applies.
