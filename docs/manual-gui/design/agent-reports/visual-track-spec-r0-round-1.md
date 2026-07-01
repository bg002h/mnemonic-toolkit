# R0 SPEC review — GUI visual screenshot track — round 1 (2026-07-01)

**Artifact:** `docs/manual-gui/design/SPEC_gui_visual_screenshot_track.md` (draft → R0)
**Recon basis:** `docs/manual-gui/design/agent-reports/visual-track-recon-2026-07-01.md`
**Reviewer:** opus-tier architect, adversarial R0 (gate: 0C/0I before any plan/code)

## Verdict: RED — 0 Critical / 4 Important / 8 Minor-Nit

The architecture is sound: the split-gate design (GUI-side threshold render-gate, manual-side byte copy-gate) is the correct response to the recon's disproof of regenerate-anywhere byte-identity, and the recon itself survives spot-verification against crate sources (no hallucination found — details below). But the spec leaves the single load-bearing link of the provenance chain ("when does the GUI snapshot job fire") open to a lagging-indicator outcome this project has already been burned by, understates the fidelity contract (clipping), and has a first-class-bar secret-hygiene assertion pointed at the wrong value channel. All four Importants are cheap spec-text folds; none invalidates the architecture.

---

## Recon spot-verification (performed, not assumed)

Verified directly against `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`:

- **Threshold semantics** — `egui_kittest-0.31.1/src/snapshot.rs:10-27`: `SnapshotOptions::default()` = `threshold: 0.6`, `output_path: "tests/snapshots"`, doc-comment "enough for most egui tests to pass across different wgpu backends". Comparison call `snapshot.rs:274-275`: `dify::diff::get_results(previous, new.clone(), *threshold, true, None, &None, &None)` — AA-detection ON. dify semantics confirmed: `dify-0.7.4/src/diff.rs:7` (`MAX_YIQ_POSSIBLE_DELTA: 35215.0`), `diff.rs:69-84` (`delta.abs() > threshold` → AA-check → `Different`), `diff.rs:109-124` (any single `Different` pixel → `diffs > 0` → failure), `yiq.rs:55-59` (`0.5053·ΔY² + 0.299·ΔI² + 0.1957·ΔQ²`). Recon accurate.
- **Adapter selection** — `egui_kittest-0.31.1/src/wgpu.rs:11-50`: "Prefer software rasterizers", `DeviceType::Cpu => 0` ("CPU is the best for our purposes!"), backend sort Metal 0 / Vulkan 1 / Dx12 2 / Gl 4. Recon accurate.
- **Env-gating / update flow** — `snapshot.rs:161-169`: `UPDATE_SNAPSHOTS` anything but `false|0|no|off` enables, and updating **succeeds** the test. `snapshot.rs:208-218`: run artifacts `.diff.png` / `.old.png` / `.new.png` with in-source comment "These should be in .gitignore" — the spec's gitignore-3 list is exactly right. Recon accurate.
- **Feature graph** — `egui_kittest-0.31.1/Cargo.toml` `[features]`: `snapshot = ["dep:dify","dep:image","image/png"]`, `wgpu = ["dep:egui-wgpu","dep:pollster","dep:image","dep:wgpu","eframe?/wgpu"]`, no `default` list; `eframe?/wgpu` is weak. Dev-graph-only claim holds; mnemonic-gui's dep is featureless `egui_kittest = "0.31"` (`mnemonic-gui/Cargo.toml`, near line 108).
- **LazyRenderer deferral** — `renderer.rs:36-45, 65-89`: with `wgpu` on, `Default` stores only a builder closure; the wgpu instance is created on first `render()`. The ~58 AccessKit-only kittest tests never call `render()` → unaffected. Recon accurate.
- **Pixel-count budget absence** — `egui_kittest-0.31.1/README.md` (~line 79) cites egui #5683 as TODO. Confirmed: no budget knob exists.
- **Fixture blankness** — `mnemonic-gui/src/form/fixtures.rs:26-29`: `render_fixture` returns `FormState::default()` for all 61 forms. Single-fixture coherence with the structural track is real, not aspirational.
- **Secret defaults** — mechanical scan of `mnemonic-gui/src/schema/{mnemonic,md,ms,mk}.rs`: **63 `secret: true` flags, every one `default_value: None`.** The claim "no auto-seeded default is secret-bearing" is TRUE at current master. Plus second-layer defense: secret fields render masked on load (`src/form/secret_widget.rs:58` `TextEdit::singleline(..).password(true)`; also `slot_editor.rs:52`, `widget.rs:592`, `tree_form.rs:687,711`).

---

## Critical

None.

---

## Important

### I1 — The provenance chain's one load-bearing link is left plan-optional; "schedule" must be struck. (Spec §5, line 28)

Spec §5: "The job also runs on a schedule or on PRs touching `src/` (plan decides) — it is the LEADING gate for visual drift." This sentence is internally contradictory — a scheduled job is by definition a **lagging** gate — and it permits a plan-legal outcome that punches a hole through the whole split-gate design:

The manual's `verify-figures-gui` byte-gate proves `figures/gui/ == pinned-tag files`. The ONLY thing that proves *pinned-tag files == pinned-tag code's rendering* is the GUI-side threshold job. If that job is schedule-only (or path-filtered too narrowly), a form-changing GUI PR merges with stale committed PNGs; a subsequent tag freezes the staleness; the manual pin-bumps, copies the stale PNGs, and its byte-gate goes GREEN on screenshots that do not depict the pinned GUI. That is byte-for-byte the schema_mirror v0.27.x failure class CLAUDE.md documents ("accumulates silently until the next pin bump"; "the drift gate is therefore a lagging indicator") — except here there is no later drift-gate catch at all until the job next happens to run.

**Ruling (decisive, as requested):** the snapshot job MUST run on **every GUI PR + every master push + every `mnemonic-gui-v*` tag push**, as a **required check**. The tag-push run is the provenance anchor Leg 2's byte-gate inherits. `build.yml` already triggers unfiltered on PRs and on `mnemonic-gui-v*` tags (`mnemonic-gui/.github/workflows/build.yml:3-8`) — adding the job there gives all three for free and matches repo convention (no path filters; path-filtering a *required* check also creates skipped-check merge ambiguity, so don't). Cost is one pinned-Mesa job of a few minutes with `Swatinem/rust-cache` (already used by every job in that file) — negligible against the failure class. A schedule adds nothing once per-PR firing exists; delete the option from the spec, don't delegate it to the plan.

**Companion fix (same finding):** the env-gate makes *silent skip* the other half of this hole — if `GUI_SNAPSHOTS=1` is mistyped/dropped in the workflow, `cargo test --test gui_form_snapshots` passes vacuously and the required check stays green forever. The job must prove it exercised 61 comparisons. Cheap tripwire: kittest writes `{name}.new.png` on every gated-ON comparison run (`snapshot.rs:213,240`; only deleted at the start of the *next* run), so a post-test step asserting `find tests/snapshots/forms -name '*.new.png' | wc -l` == 61 (or an in-test executed-count assertion) makes skip loud. Spec §5 should mandate the ran-at-all census, not just the file census.

### I2 — Threshold-churn posture: the spec needs a documented flake/remediation stance (one paragraph), and the spike's cross-runner evidence is a single sample. (Spec §2 line 12, §3 line 18)

**Ruling (decisive, as requested):** keep 0.6 as the gate; chronic churn is *unlikely but not provably zero*, and the spec must say what happens if it appears, because the failure mode of an undocumented flaky required gate is exactly the devaluation the prompt names (reflexive `UPDATE_SNAPSHOTS` churn = the gate stops meaning anything).

Basis for "unlikely": (a) arithmetic — with dify's coefficients, a ±1-LSB shift in ALL of R,G,B on a pixel yields delta ≈ 0.505 < 0.6 (ΔY=1.0; ΔI,ΔQ≈0 since their coefficients sum ≈0), and a single-channel ±1 LSB yields ≤ ~0.16; the first thing that FAILS is a ±2-LSB single-channel shift on one non-AA pixel (≈0.64). Cross-CPU LLVM-JIT drift (AVX2 vs AVX-512 vector width — the recon's honest UNVERIFIED) manifests as float-rounding deltas that quantize to 0/±1 LSB except at quantization boundaries, and boundary-straddling pixels cluster on AA edges, which dify excludes. (b) Empirics — the recon measured a FULL backend swap (Intel Vulkan vs llvmpipe GL) at 0 differing pixels @0.6 (34 @0.1), a far larger perturbation than intra-Mesa cross-CPU drift. So within ONE pinned rasterizer the residual is small. But: the spike's item (iii) is ONE cross-runner sample; GitHub's fleet is CPU-heterogeneous over time.

Required spec additions: (1) spike item (iii) becomes ≥3 distinct runner instances (trivial: re-run the job), all 61 forms once the suite exists, not just the ≥3 spike forms at P-phase close; (2) a remediation ladder, in the spec so it is policy not improvisation: **inspect the `.diff.png` artifact first → if the diff is genuinely invisible, raise the threshold for THAT form only via per-test `SnapshotOptions` with the change + a human visual sign-off recorded in the PR (kittest README's own warning applies) → never blanket-raise the default → if drift is systemic across forms, move regeneration into the pinned environment (wgpu-CI Mesa tarball; regenerate via a `workflow_dispatch` + `UPDATE_SNAPSHOTS` artifact so the corpus equals fleet rendering) — and explicitly: a pixel-COUNT budget is NOT available (egui #5683) and must not be promised.** With that paragraph, the posture is honest and the gate defends its meaning.

### I3 — Secret-hygiene assertion is pointed at the wrong value channel; add the schema-default machine check. (Spec §4, line 23)

§4's INVARIANT covers *injection* ("no snapshot test may inject any value into a `flag_is_secret` field") and the hygiene step "greps the FIXTURE construction". But the fixture is blank (`fixtures.rs:26-29`) — the values that actually reach pixels come from the **schema `default_value` prefill** (`src/schema/mod.rs:107-125`; the auto-seed the spec itself names in §4 line 21). A future PR adding `default_value: Some(<secret-ish>)` to a `secret: true` flag would render into the PNG corpus, pass the fixture grep, ship in the manual, and no grep can ever find it in pixels. Today this is doubly covered — my scan shows all 63 secret flags have `default_value: None`, and secret widgets render `password(true)`-masked regardless (`secret_widget.rs:58`) — but under the project's FIRST-CLASS secret-hygiene bar the *stated assurance* must cover the actual channel. Fold: the hygiene step additionally machine-asserts `flag.secret ⇒ flag.default_value.is_none()` schema-wide (a one-line test over the static schema tables), and the spec names the masked-widget rendering as the second, independent layer. Cheap; closes the future-drift hole honestly.

### I4 — The no-clipping acceptance bar is missing; one of the two permitted sizing answers silently breaks the spec's own coherence claim. (Spec §4 line 22 vs §7 line 39)

§4 claims "The PNG therefore shows exactly the screen the structural render describes" and then defers `fit_contents()` vs "a fixed logical size" to the plan as "bounded, not architecture". It is not bounded as written: the real app scrolls its forms (`mnemonic-gui/src/main.rs:490,499,593` — `ScrollArea::vertical()`), the PR-#24 whole-form render path does NOT (`tests/ui_harness/mod.rs:418-453` renders flags straight into the `Ui`), and the kittest default viewport is 800×600 (`builder.rs:16-28`). At any fixed size, tall forms (`bundle`, `import-wallet`, … with dozens of flag rows at ppp 2.0) get bottom-clipped: the PNG then depicts a strict SUBSET of what the structural render lists, the census-61 still passes, §7's "catches any visual change to any form" becomes false for everything below the fold — and nothing fails. Fold: the spec states the acceptance bar — **every flag row of the settled canonical form must be visible in the PNG (no viewport clipping)** — and the plan's sizing freedom is bounded by that bar (which effectively means `fit_contents()` or per-form measured height). Consequence for §3: the spike's "≥3 representative forms" MUST include the tallest form, and must confirm `fit_contents()` behaves with the harness's non-scrolling whole-form path (expected fine precisely because there's no ScrollArea, but it is the load-bearing sizing fact — measure it, don't assume it).

---

## Explicit rulings requested by the review charter

**(1) Provenance chain / when the job fires:** see I1. HOLE CONFIRMED as spec-permitted (not spec-mandated). Ruling: per-PR + master-push + tag-push, required check, in `build.yml`, no path filter, plus the 61×`.new.png` ran-at-all tripwire. Schedule-only or schedule+release is REJECTED — it recreates the documented schema_mirror lagging-indicator failure with no downstream catch. The tag-push run is the anchor that makes Leg 2's byte-gate meaningful.

**(2) Manual-side fetch:** RULING — reuse, don't re-fetch. The lint job already clones the pinned GUI in full (`.github/workflows/manual-gui.yml:54-62`, `git clone --depth 1 --branch "$PINNED_TAG"` — depth-1 is a *complete tree* at the tag, so `tests/snapshots/forms/` is present; nothing sparse is needed) and threads it as `MANUAL_GUI_UPSTREAM_ROOT`, which `tests/lint.sh` already hard-requires (`lint.sh:56`) and consumes in phases 4-5 (`lint.sh:95-116`). `verify-figures-gui` should be a new lint phase (or a sibling step in the lint job) consuming the same root — one pin-read step, one clone recipe, zero new fetch mechanics. The spec's "shallow/sparse fetch" wording invites a second, divergent fetch path; replace it with "reuse the lint job's pinned clone". (Filed as M5; the spec's mechanism works, it's just worse.)

**(5) Threshold churn:** see I2. 0.6 stands; churn within one pinned Mesa across heterogeneous runner CPUs is unlikely on both arithmetic and empirical grounds but is not provable from one spike sample; the spec adds the remediation ladder (per-form threshold raise + recorded visual sign-off; never blanket; no pixel-count budget exists — say so) and the spike widens cross-runner evidence to ≥3 instances.

---

## Minor / Nit

- **M1 — Spike STOP-condition inconsistency (§3):** llvmpipe-GL is called "the documented fallback" inside item (i), but STOP fires "if both Plan A and Plan B fail" — both A and B are lavapipe-Vulkan, so the fallback's status is undefined at exactly the moment it matters. Clarify: llvmpipe-GL on-runner is Plan C, tested by the spike; if only C works, the gate runs on GL (corpus is generated in the same env, threshold arbitration unchanged); STOP fires only if A, B, AND C all fail on runners. The honest-STOP principle is right; the ladder just needs its real third rung.
- **M2 — Spike deliverables list (§3) omits measurements §4/§5 depend on:** fit_contents-vs-fixed on the tallest form (§4 line 22 promises "P0 measures both" but §3 doesn't list it), full-61 render wall-time + job wall-time (incl. cold vs rust-cached wgpu/dify dev-dep compile), and actual corpus size vs the ~3.4 MB estimate. Add them to §3's persisted-output list.
- **M3 — Leg-2 PDF image-path mechanics (§6 line 36):** the PDF rule runs xelatex from `build/` (`docs/manual-gui/Makefile:171-186` — `cd $(BUILD_DIR) && $(XELATEX) …`), so `\includegraphics{figures/gui/…}` emitted from source-relative paths won't resolve; needs `\graphicspath`, absolute-path rewrite, or copy-into-build. HTML already embeds via `--self-contained` (`Makefile:188-207`) — the spec says `--embed-resources`; align with what exists (or modernize deliberately, since `--self-contained` is deprecated in pandoc 3.x). Fails loudly at `make pdf`, hence Minor — but the spec asserts the builds "build with images embedded" as if free; note the known pitfall.
- **M4 — Regeneration UX honesty (§2 line 12):** "UPDATE_SNAPSHOTS=1 in the pinned environment" is a requirement contributors cannot meet locally. State the real posture: regenerate locally on whatever software rasterizer (recon proved a full backend swap stays under 0.6, so locally-regenerated bytes pass the pinned CI gate), CI's pinned threshold gate arbitrates; optionally provide the `workflow_dispatch` + artifact path from I2's ladder for pinned-env regeneration. `.gitignore` triple (`.new`/`.diff`/`.old`) verified correct against `snapshot.rs:208-218`.
- **M5 — Fetch-mechanics simplification** (see explicit ruling 2): replace "shallow/sparse fetch of the pinned ref" with reuse of the lint job's `MANUAL_GUI_UPSTREAM_ROOT` clone.
- **M6 — Dark theme in a light print manual (§4 line 22, §7 line 40):** the faithfulness call is CORRECT (verified: egui falls back to Dark, kittest supplies no system theme, mnemonic-gui sets no custom visuals — recon Q5 spot-held), and captions/Part-intro documentation is the right mitigation. But 61 dark rectangles in a light PDF is a real, user-visible presentation tradeoff (page ink, visual weight) — surface it to the user for a one-line ack at plan time rather than deciding silently. No spec change beyond that; a light-theme variant stays out of scope (§7).
- **M7 (nit) —** "~58 existing AccessKit-only kittest tests" (§5 line 26): the tests dir has 69 test files; ~58 is the PR-#24 kittest subset. Immaterial; keep the qualifier "kittest".
- **M8 (nit) —** the `GUI_SNAPSHOTS` early-return skip should `eprintln!` a skip marker so a bare `cargo test` transcript shows the suite was gated off (the binding tripwire is I1's `.new.png` census; this is just transcript honesty).

---

## Scope/cost sanity (charter item 7) — CONFIRMED

- Dev-dep-only: verified via kittest's feature graph + weak `eframe?/wgpu` (registry `Cargo.toml`); shipped binary graph untouched; new transitive dev-deps = dify/colored/getopts/rayon (colored + rayon visible in dify source imports, `diff.rs:1-4,128`).
- Existing kittest tests unaffected: verified via `LazyRenderer` deferral (`renderer.rs:36-45,65-89`).
- Plain-committed PNGs, no LFS: SOUND, and load-bearing — the manual-side clone (`manual-gui.yml:54-62`) does no LFS smudge, so LFS pointers would byte-compare garbage or fail decode (kittest's own decode-error text tells users to set up git-lfs, `snapshot.rs:~135`); ~3.4 MB (even ×3 for denser forms) is comfortably plain-committable; egui's LFS choice reflects a far larger corpus. LFS is REJECTED correctly.
- MINOR GUI tag: correct (additive corpus + test + CI job; app behavior unchanged). One footnote: the msrv job runs `cargo check --locked` (lib/bins), so dify's undeclared MSRV is not gated there — §5's "verify dify compiles at 1.88 in P1" is the right and sufficient check.
- Structural track untouched: §8's claim is consistent with everything reviewed; `verify-examples-gui` (`manual-gui.yml:209-259`) is not modified by either leg.
- Fixture coherence (charter item 4): CONFIRMED as the right choice — same `render_fixture` blank-state both tracks (`fixtures.rs:3-14` names both consumers), so the PNG depicts exactly the state the structural render describes and the faithfulness gate proves. Hygiene claim verified at master (63/63 secret flags default-None) — with I3's assertion required to keep it true by machine rather than by audit.

---

## Gate disposition

RED at 0C/4I. All four Importants are spec-text folds (no architecture change): I1 strike-schedule + mandate per-PR/master/tag firing + ran-at-all census; I2 remediation-ladder paragraph + ≥3-instance spike evidence; I3 one-sentence hygiene extension (schema `secret ⇒ default_value: None` assertion); I4 one-sentence no-clipping acceptance bar + tallest-form spike requirement. Fold and re-dispatch for round 2 per the reviewer-loop rule — do NOT proceed to plan or P0 on this draft.
