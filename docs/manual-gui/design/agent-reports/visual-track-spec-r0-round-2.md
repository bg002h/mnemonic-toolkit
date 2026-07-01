# R0 SPEC review — GUI visual screenshot track — round 2 (2026-07-01)

**Artifact:** `docs/manual-gui/design/SPEC_gui_visual_screenshot_track.md` (post-round-1 fold)
**Round 1:** `docs/manual-gui/design/agent-reports/visual-track-spec-r0-round-1.md` (RED, 0C/4I/8m)
**Charter:** confirm fold fidelity for I1–I4 + M1–M8, hunt fold-introduced drift, no re-litigation of settled points (split-gate architecture, recon facts, 0.6 threshold — all ruled sound in round 1).

## Verdict: GREEN — 0 Critical / 0 Important (3 residual Minor/nits, non-blocking)

All four Importants and all eight Minors are folded faithfully; two folds (I1, I3) were independently re-verified against live sources and both hold — I3's fold is in fact *stronger* than round 1 required (detail below). Three wording-level residuals were found (m1–m3 below), all Minor-class, none touching architecture or policy substance. **The spec may proceed to the plan-doc.**

---

## Fold-fidelity verification (Important findings)

### I1 — CI-job firing + ran-at-all tripwire: FOLDED CORRECTLY, unambiguous and complete

§5 now mandates, in a single sentence with no plan-optional escape hatch: **"fires on EVERY PR + every master push + every `mnemonic-gui-v*` tag push, as a REQUIRED check, NO path filter (I1 ruling)"** — exactly the round-1 ruling's three triggers, placed in `build.yml`, with the schedule explicitly REJECTED and the schema_mirror-v0.27.x lagging-indicator rationale carried into the spec text (so future readers get the *why*, not just the rule). The companion tripwire is present and mechanically specified: post-test census `find tests/snapshots/forms -name '*.new.png' | wc -l` == 61 (or an in-test executed-count assertion), plus the M8 `eprintln!` skip marker.

**Independent re-verification:** `mnemonic-gui/.github/workflows/build.yml:3-8` confirms all three triggers already exist (`push: branches: [master], tags: ['mnemonic-gui-v*']`; `pull_request`), so §5's "gives all three for free" rationale is factually grounded — the plan only adds a job, not workflow triggers.

**Coherence check (env-gate × required-check interplay):** coherent. The dedicated job sets `GUI_SNAPSHOTS=1`; the schema-mirror job and dev `cargo test --workspace` runs hit the early-return skip (loud via the M8 marker). The vacuous-pass hole exists only in the job where the gate is required — and that is exactly the job that carries the `.new.png` census, so the hole is plugged where it matters. The `.new.png` artifacts being gitignored (§5 line 28) does not conflict with using them as a CI census (filesystem `find`, not git). No contradiction found.

### I2 — Threshold posture + remediation ladder: FOLDED CORRECTLY, policy-complete and honest

§5's posture paragraph contains every rung of the round-1 ladder, in order: (1) inspect `.diff.png` FIRST; (2) per-form-ONLY threshold raise via per-test `SnapshotOptions`, with the change **plus a human visual sign-off recorded in the PR**; (3) NEVER blanket-raise the default; (4) systemic drift ⇒ regeneration moves into the pinned environment (wgpu-CI Mesa tarball, `workflow_dispatch` + `UPDATE_SNAPSHOTS` artifact). The honesty clause is verbatim-equivalent: "a pixel-COUNT budget is NOT available in kittest (egui #5683) and must not be promised." The arithmetic basis (±1-LSB all-channel ≈ 0.505 < 0.6; first failure = ±2-LSB single-channel non-AA pixel) is a faithful compression of round 1's derivation. §3 spike item (iii) is widened to **≥3 distinct runner instances**, with the full-61 repeat pinned to P-phase close — both halves of the round-1 requirement. Complete.

### I3 — Secret-hygiene machine assertion re-pointed at the schema-default channel: FOLDED CORRECTLY — and verified COMPLETE, not merely present

§4 now states the three layers in the right order and at the right target: (1) machine assertion, schema-wide, `flag.secret == true ⇒ flag.default_value.is_none()` over the **static schema tables** (the actual pixel channel — the auto-seed prefill); (2) masked-widget rendering (`password(true)`, `secret_widget.rs` + call-sites) as the independent second layer; (3) the injection invariant retained as layer 3. The framing "pixels can't be grepped — the assurance is channel-side + widget-side, stated honestly" is exactly the honest posture round 1 demanded.

**Independent re-verification at current master (`mnemonic-gui` f882f830, == origin/master):** precise parse of all `FlagSchema` literals across `src/schema/*.rs` → **63 entries with `secret: true` (54 mnemonic.rs + 9 ms.rs), zero with `default_value: Some(..)`** — the spec's "63/63 TRUE at master today" parenthetical is exact. A raw grep shows 71 `secret: true` occurrences; the 8 extras are (a) 5 `PositionalArgSchema` entries in ms.rs and (b) comment text at mnemonic.rs:2748/2753 — and this is where the fold proves *complete*, not just faithful: **`PositionalArgSchema` (mod.rs:58-77) has NO `default_value` field at all.** The schema-default prefill channel exists ONLY on `FlagSchema`, so the assertion as scoped covers the entire channel by construction; secret positionals additionally render as masked `SecretLineEdit`s under the `secret_widgets["positional:<name>"]` reserved key (layer-2 coverage). No missed channel. The one-line test the spec mandates will keep the invariant true by machine exactly as intended.

### I4 — No-clipping acceptance bar + tallest-form spike: FOLDED CORRECTLY, stated as a hard bar that binds the plan

§4's new paragraph states the bar in normative form — **"every flag row of the settled canonical form must be visible in the PNG (no viewport clipping)"** — and explicitly binds the plan's sizing freedom to it ("the plan's sizing freedom … is bounded by this bar"). The mechanism (why clipping is silent: real app scrolls via `ScrollArea::vertical()`, harness whole-form path does not, census-61 still passes) is carried into the spec so the bar is self-justifying. §3 gains both spike-side requirements: "the set MUST include the TALLEST form (I4)" and the M2 measurement of `fit_contents()` vs fixed-size on that form, with §4 closing the loop ("expected fine precisely because there is no ScrollArea — measure, don't assume"). Complete. (See m3 for a residual phrasing nit that does not weaken the bar.)

---

## Minor-fold checklist (M1–M8)

- **M1** ✓ — Plan C (llvmpipe-GL, `WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1`) is named in §3 with its consequence spelled out (gate runs on GL if only C works; corpus generated in the same env; threshold arbitration unchanged); STOP now fires only if **A, B, AND C** all fail. The round-1 inconsistency (fallback undefined at the STOP moment) is gone; A/B/C + STOP is self-consistent, including `WGPU_BACKEND=vulkan` scoped to A/B only.
- **M2** ✓ — §3's persisted-output list now includes fit_contents-vs-fixed on the tallest form, full-61 render + job wall-time (cold vs rust-cached), and actual corpus size vs the ~3.4 MB estimate.
- **M3** ✓ — §6 carries the PDF pitfall (xelatex runs from `build/`; `\graphicspath`/rewrite/copy-into-build as plan detail, fails loudly at `make pdf`) and aligns HTML with the **existing** `--self-contained` (verified live at `docs/manual-gui/Makefile:195`).
- **M4** ✓ in §5 — honest regeneration posture (local regen on any software rasterizer; CI's pinned threshold gate arbitrates; pinned-env `workflow_dispatch` is the escalation, not the default). See m1 below for a stale §2 phrase the fold missed.
- **M5** ✓ — §6 mandates REUSE of the lint job's pinned clone via `MANUAL_GUI_UPSTREAM_ROOT`, "no second fetch path". Verified live: `.github/workflows/manual-gui.yml:56-61` does `git clone --depth 1 --branch "$PINNED_TAG"` and exports `MANUAL_GUI_UPSTREAM_ROOT`; a depth-1 clone contains `tests/snapshots/forms/`. Consistent with the byte-gate + census-61 + fail-closed wording in both §2 and §6.
- **M6** ✓ — theme recorded as **USER-DECIDED 2026-07-01: dark/faithful**, with the light-print tradeoff explicitly surfaced-and-accepted, captions/Part-intro noting the dark theme, and the light variant kept out of scope (§7). Exactly the M6 prescription (surface for ack — done and recorded).
- **M7** ✓ — "~58 of the 69 test files" with the "kittest" qualifier.
- **M8** ✓ — skip path `eprintln!`s a loud marker, correctly framed as transcript honesty subordinate to the I1 census.

---

## Fold-introduced drift (all Minor/nit, non-blocking)

- **m1 (Minor) — §2 line 12 retains the pre-M4 regen wording.** §2's Leg-1 summary still says "Regeneration UX = `UPDATE_SNAPSHOTS=1` **in the pinned environment**", which is the exact phrase M4 struck; §5's operative paragraph has the corrected honest posture (local regen on any software rasterizer; CI arbitrates; pinned-env is the ladder's escalation). Internal inconsistency, resolved in favor of the detailed section a plan author will implement from — but the summary should not restate the rejected requirement. One-phrase fix: "Regeneration UX = `UPDATE_SNAPSHOTS=1` (locally, any software rasterizer — CI's pinned gate arbitrates; §5)".
- **m2 (nit) — §2 line 13 says the manual gate will "fetch `tests/snapshots/forms/` at the pinned tag"** while §6 (operative) mandates reuse of the lint job's existing clone with "no second fetch path". §6 governs and is unambiguous; align §2's verb ("read from the lint job's pinned clone") when convenient.
- **m3 (nit) — §4 names the bounded sizing options twice with different pairs:** line 22 "fit_contents() per form, or a fixed logical size"; line 24 "fit_contents() vs per-form measured height". Not a contradiction — the no-clipping bar governs whichever mechanism the plan picks (a fixed size tall enough for the tallest form would technically satisfy the bar) — but the two lists should match to avoid a plan author citing the looser one without the bar.

None of these alters policy, architecture, or any gate semantics; all three are single-phrase edits. Per the 0C/0I gate they do not block. If folded, no round 3 is required **provided the edits are confined to the exact phrases cited above**; alternatively carry them into the plan-doc as line items.

---

## New-finding hunt (beyond fold fidelity): NONE at Critical/Important

Checked specifically: the tripwire's reliance on `.new.png` being written on *passing* gated-ON comparisons (settled by round 1's registry verification, `snapshot.rs:213,240` — spec cites it faithfully); required-check semantics across the three triggers (the PR-context check is the merge gate; the tag-push run is the provenance anchor Leg 2 inherits — round-1's own framing, spec mirrors it); Leg-2 ordering (pin bump + copy happen only after the Leg-1 tag, whose own snapshot run is loud if red); §5's MINOR-tag classification (additive corpus/test/CI job, app behavior unchanged — consistent); §1/§3/§5/§6 census consistency (61 everywhere); recon citation date/path consistency. Nothing new rises above nit level.

---

## Gate disposition

**GREEN — 0 Critical / 0 Important.** Round-1's four Importants and eight Minors are all folded to prescription; two folds were re-verified against live sources (build.yml triggers; 63/63 schema census at f882f830) and one (I3) was verified complete beyond the round-1 requirement (the default-value channel provably exists only on `FlagSchema`). The three residuals (m1–m3) are non-blocking one-phrase wording alignments. **The spec has converged and may proceed to the plan-doc.** The P0 spike remains the first plan phase and remains load-bearing (no implementation past it before its persisted result), per §3.
