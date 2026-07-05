# P0-R0 review — `gui_example_tutorial` cycle spike (ratify GO + mechanisms; RULE on scope findings)

- **Reviewer:** opus-tier architect (adversarial P0-R0 gate; 0C/0I).
- **Date:** 2026-07-05.
- **Gate scope:** ratify the spike's **GO** on the user-locked contract; ratify its
  S3/S4/S5 mechanism choices; and **RULE** on the F1/F2 scope findings + F3/F6/F7
  dispositions + the PR-#29 disposition, so the implementation plan-doc can be written
  against binding constraints. This review does NOT re-open the spec (R0-GREEN ×2) or the
  user-locked design decisions (§1.1).
- **Artifacts reviewed:** `gui-example-p0-spike.md` (full); `SPEC_gui_example_tutorial.md`
  + `gui-example-spec-r0-round-1.md` / `-round-2.md` + the four recon reports.
- **Live ground-truth spot-checks (mnemonic-gui `spike/gui-example-p0` @ `80f7eb8`,
  3 commits off `master@0d4429d`; toolkit @ current `master`):** all load-bearing
  evidence re-verified against source and CI — see the RATIFICATIONS. Repos left clean.

## VERDICT: **GREEN — 0 Critical / 0 Important — the plan-doc may be written.**

The spike proves the GO end-to-end, and every mechanism claim I spot-verified holds against
ground truth (source + the egui-0.31.1 crate + the live CI on PR #29). The four RATIFICATIONS
below stand. The F1/F2 scope finding is **correctly surfaced and correctly routed to a P1
`src/` fix (not a descope)** — I ratify fix-in-P1 and rule the sound fix SHAPE, adding one
load-bearing constraint the spike's own ratification #7 under-specified (the conditional/dropdown
drift-gate ripple, distinct from the flag-name `schema_mirror`). That constraint is **resolved
by this ruling** (binding on the plan-doc), so no Important remains open. Six Minors below fold
into the plan-doc, which then takes its own R0 per §9/house rule.

The spike-file rename question (§4 names it `gui-example-tutorial-p0-spike.md`; it is written
as `gui-example-p0-spike.md`): **KEEP the current name — do not rename.** The short form is the
established P0-artifact convention across the branch (`spike/gui-example-p0`), PR #29, the
throwaway workflow (`spike-gui-example-p0.yml`), and this review (`gui-example-p0-r0-review.md`);
renaming to the spec's longer name would orphan the spike from its own review. Recorded as a
spec cite-nit (m6) for the spec's next touch, consistent with the cite-nits already in the spec
header.

---

## RATIFICATION 1 — the GO on the locked contract: **RATIFIED**

The user-locked contract is: whole-window shots of the REAL `ui()` loop, output panes genuinely
populated by real pinned-CLI Run clicks (one-click non-secret + the two-click secret modal path,
masked everywhere), byte-deterministic same-env and sub-threshold cross-backend, with the two
named gates BITING. All verified:

- **Whole-window + real extraction.** The S1 commit (`29777ee`) is a genuine, production-shaped
  relocation, spot-verified in `src/app_window.rs`: `new(cc,…)` wrapper (OS effects +
  `AppState::detect_all()` at `:213`, order-preserving) / `new_headless(…)` pure ctor (`:223`) /
  `ui(&mut self, ctx)` (`:359`) / `impl eframe::App::update` delegating `self.ui(ctx)` (`:1122-1123`);
  `main.rs` shrinks to 82 lines (bootstrap only; report says "84" — off by 2, immaterial → m5).
  The two source-scan tripwires (`paste_warn_wiring_v0_40_0`, `r7_no_auto_repair_removal`) extend
  their `include_str!` scan set to `app_window.rs` with `main.rs` retained — verified in the
  `29777ee` diff.
- **Same-env byte determinism.** The spike test asserts in-process RGBA byte-identity across two
  independently-built harnesses at three sites (`gui_example_p0_spike.rs:397` S1, `:525` S2(i)
  double-run, `:857` S3 scroll), plus stdout byte-identity (`:524`). Sound.
- **Cross-backend ≤0.6.** CI on PR #29 is **15/15** (`spike-sample-1` + `spike-sample-2` both
  SUCCESS) — the GL-llvmpipe-local baselines under comparison PASS on Vulkan-lavapipe ×2 runners
  at the default 0.6 dify, i.e. the full backend swap the visual-track spike met. Verified live via
  `gh pr view 29`.
- **Live Run-click populated pane, both paths.** Non-secret one-click export-wallet (real argv
  `argv[0]=="mnemonic"`, exit 0, canonical `wpkh([…]xpub…)` on stdout, public xpub slot token
  asserted UNMASKED at the `SECRET_SLOT_SUBKEYS` boundary — `:487-508`). Secret two-click J1 modal
  (demo-seed Xpub→phrase flip, `PasswordInput` typed, masked `Preview` `••••`, `assert_no_plaintext`
  over EVERY AccessKit node label+value at 4 checkpoints, modal argv masked, slot `mask[i]==true`,
  real card set on stdout — `:591-691`). The two-Run-button collision is proven present
  (`assert_eq!(runs.len(), 2)`, `:651`) and resolved by Window-subtree scoping
  (`by().role(Window).label("Confirm secret-bearing run")`, `:697-711`). The masked-everywhere
  guard is credibly global: egui 0.31 masks password values before AccessKit (I confirmed the
  handled-action set below; the report's `mask_if_password` mechanism is consistent).
- **Both named gates BITE.**
  - *Version gate:* I verified the BITE premise at source — the schema constant `pinned_version:
    "mnemonic 0.74.0"` (`schema/mnemonic.rs:4620`) genuinely differs from the spike tier
    `mnemonic 0.75.0`, so the deliberate wrong-tier probe (`:303-309`) must fire, and it does
    (`SPIKE-GATE-BITE`), while the real gate passes against the on-`$PATH` tier (`:311-314`). The
    gate probes `<cli> --version` BEFORE any render/spawn (`:292`, ahead of the first harness build)
    — a wrong-tier local regen cannot produce honest-looking bytes. Fail-closed (`panic!` on the
    real arm). Sound.
  - *SAME-FRAME tripwire (m1 semantics, folded):* verified the mechanics in
    `step_once_and_assert_same_frame_completion` (`:214-225`) and every call site — the pre-step
    **non-vacuity** assert (`last_run.is_none()` after the queued click, e.g. `:479-482`, `:637`),
    then **exactly ONE `h.step()`**, then `assert last_run.is_some()` BEFORE any further stepping,
    THEN `h.run()` to settle for the shot. This is precisely round-2 m1's prescription (single-frame
    window so a fast-settling async redesign cannot slip past). Applied at all four click sites
    (S2i `:483`, S2ii modal `:665`, S2iii refusal `:556`, S5 chain `:736`). Sound and teeth-bearing.

No STOP condition (§4) was approached. **GO ratified.**

---

## RATIFICATION 2 — S3 scroll: injected `MouseWheel` — **RATIFIED (spot-verified against egui 0.31.1)**

- **(i) AccessKit scroll actions DEAD — independently re-verified.** I grepped the vendored
  `egui-0.31.1` crate: **zero** `ScrollIntoView` / `ScrollUp` / `ScrollDown` / `SetScrollOffset` /
  `Action::Scroll` handlers anywhere. The complete handled-action set is `Focus` / `Click`
  (`response.rs`, `context.rs`) + `Increment` / `Decrement` / `SetValue` (`drag_value.rs`,
  `slider.rs`) — exactly the report's "only Click/Focus/SetValue/Inc/Dec/SetTextSelection". The
  egui version is confirmed `0.31.1` in `Cargo.lock`. The report's claim is TRUE.
- **(ii) Injected wheel WORKS + pixel-stable.** `PointerMoved` to the form-scroll-region center
  then `Event::MouseWheel{unit: Point, delta}` then `run()` (`wheel_scroll_form`, `:944-955`),
  with the smooth-scroll animation settling deterministically. The test asserts both the offset
  moved (`--md1` header y before>after by >100pt, `:839-842`) and byte-identical pixels across
  independent harnesses (`:857`). Delta-in-points is manifest-recordable; clamping makes
  "scroll to bottom" trivially stable. Sound.
- **`with_max_steps(64)` requirement RATIFIED.** kittest's default 4 is too tight for the ~8-frame
  smooth-scroll animation; the harness sets 64 (`:82`), documented at the seam. Correct and
  necessary; carry it into the real harness.
- **(iii) `vertical_scroll_offset` seam stays the unused fallback.** Correct — since (ii) is proven,
  the seam is NOT added to production (avoids a test-only `src/` surface). Note the CHANGELOG
  test-only-surface doc (§11) is therefore NOT triggered — state this in P2.
- Bonus (ratified into discipline): AccessKit drives are id-addressed/pointer-free and work on
  off-viewport widgets, so **only SHOTS need scroll, not drives** — decouples manifest drive order
  from scroll offsets. Good; the §5.4 driven-field-visibility gate is prototyped GREEN (`:811-832`).

**Scroll mechanism (ii) ratified, pixel-stable.**

---

## RATIFICATION 3 — S4 size 920×720 + corpus budget: **SIZE RATIFIED; budget ACCEPTED with a firmed-up gate (not "trims in reserve")**

- **920×720 logical / 1840×1440 physical @ ppp 2.0 — RATIFIED.** I confirmed it is the literal
  production default seed (`main.rs:46`, `.unwrap_or([920.0, 720.0])`; the report's `main.rs:52`
  cite is the `.with_inner_size` line a few rows down — same value, m5). The three rationales hold:
  production fidelity (§5.4's "what the user sees at this moment"; the Chapter-0 shot IS the literal
  first-launch window), page legibility (1840px keeps UI text ~28% larger on paper than 2560px — a
  tutorial is read, not zoomed), and smaller bytes (~330 KiB vs ~420 KiB). Single global size = one
  determinism surface. Correct.
- **Corpus budget — ACCEPT the ~16.4 MiB/51 projection, but the plan MUST convert "trims in
  reserve" into a HARD budget gate.** Repo-size reality: the toolkit's `docs/manual-gui` PNG corpus
  today is **2.12 MiB (61 forms)** — the tutorial adds ~16.4 MiB *per repo* (GUI corpus + toolkit
  byte-copies), i.e. ~+8× on the toolkit side and ~+33 MiB across both repos, committed to git
  history permanently. That is under the spec's own 20 MiB flag, so I do NOT require a trim *now* —
  but "keep §5.3's trims in reserve" is not a gate. **Ruling:** the plan-doc must (a) **re-measure
  the ACTUAL corpus at the full 51-shot manifest** (the 16.4 is a projection from an 11-shot mean;
  populated-pane card shots run heavier than the pilot mean, so the real number may exceed the
  projection), and (b) **pre-commit to the trim levers with a firm ≤20 MiB ceiling, targeting
  ≤15 MiB for margin** — apply the cheap trims (prune `-form2` to the minimum the driven-field
  visibility gate requires; one representative modal shot rather than one per secret step) if the
  real number lands above ~15 MiB. Make this a named budget assertion in the manifest census, not a
  fallback footnote.
- Time projection (~60–90 s test body + ~4–6 min cacheable `cargo install` in CI) is well inside
  job budgets; accept.

---

## RATIFICATION 4 — THE SCOPE RULING (F1/F2): **fix-in-P1 RATIFIED; fix SHAPE ruled, with a binding drift-gate constraint**

This is the review's central decision. I re-verified the entire F1/F2 causal chain at source; it is
**correct**, and the orchestrator's position (fix in the GUI leg at P1, full R0, NOT descope) is
**RATIFIED** — descoping J2/J3/J4 export-wallet steps would be a locked-contract (ALL-journeys)
downgrade reserved to the user (§1.1.1, §4 STOP menu). Details:

### F1 is real and unreachable — verified at source
- `render_with_dispatch` materializes an ABSENT flag and **writes it back** into `state.values`
  (`widget.rs:220-229`): `idx=None` ⇒ `value = default_flag_value_for_flag(flag)` ⇒ pushed to
  `state.values`.
- For export-wallet `--template` (`schema/mnemonic.rs:1384`): `kind: Dropdown(TEMPLATES)`,
  `default_value: None`. With no schema default, `default_flag_value_for_flag`
  (`flag_defaults.rs:76`) falls through to `default_flag_value_for(&Dropdown)`
  (`flag_defaults.rs:29-31`) = `Dropdown(opts[0])` = **`"bip44"`** (`TEMPLATES[0]`, and `TEMPLATES`
  carries **no `""`/unset entry** — verified `schema/mnemonic.rs:69`).
- `has_value("--template")` is then TRUE (Dropdown present iff non-empty — `schema/mod.rs:388`),
  so `conditional::export_wallet` (`conditional.rs:596-601`) disables `--descriptor` (mutex). egui
  ignores typed input to disabled widgets, and there is **no unset option to clear the template** →
  `--descriptor` is unreachable on a virgin export-wallet form. Recon C's "all journeys
  GUI-expressible" holds at the FORM level, fails at the FLOW level. F1 confirmed.
- **This is a genuine `src/` need with no route-around:** J2/J3/J4 `export-wallet --descriptor
  "$(cat vault)"` canonicalise/BSMS/core-export steps take an arbitrary existing descriptor (the
  4-tier vault) that no `--template` can reproduce; export-wallet has no `--descriptor-file`
  either (F5). So the arm must be made reachable.

### The fix SHAPE ruling — **variant (a), GUI-render-scoped, NOT variant (b)**
The spike's ratification #7 says the fix has "clap-surface = UNCHANGED (no schema-mirror impact)."
That is TRUE **only for the narrow flag-name `schema_mirror` test** (which I confirmed gates flag
*names* — `schema_mirror.rs:52-124` — not dropdown values). It **omits the adjacent
conditional-rules drift gate**, and the two candidate variants differ sharply there:

- **Variant (b) — conditional refinement** (don't disable `--descriptor` when `--template` is at
  its materialized default): this **changes what `conditional::export_wallet` emits**, and that
  function is gated by `gui_schema_conditional_drift.rs` against the toolkit's projected rules. I
  verified the toolkit projects the export-wallet mutex **unconditionally** —
  `FlagPresent(--template) → --descriptor Disabled` (`cmd/gui_schema.rs`
  `export_wallet_conditional_rules`) — with no "unless at default" carve-out. A GUI carve-out
  would **drift** from that projection (the drift gate's exemplar sets `--template` and expects
  `--descriptor` Disabled), and making it clean would require a **PAIRED toolkit gui-schema change**
  — breaking the cycle's "zero toolkit `src/` changes / vacuous mirror" invariant (§3.1a, §12.4)
  and the strict GUI-tag-before-toolkit-pin ordering. **REJECTED.**
- **Variant (a) — a GUI-only, export-wallet-scoped unset/"(none)" affordance for `--template`:**
  this does **not** touch the mutex RULE. When `--template` is set (any value), `--descriptor`
  stays disabled = the projected rule; when the user selects the unset option, `--template`→`""`
  ⇒ `has_value` false ⇒ `--descriptor` enabled — exactly the projected mutex semantic. So
  `conditional::export_wallet` keeps emitting the toolkit's projected rules → **drift-gate CLEAN,
  zero toolkit change.** The existing `display_or("(none)", …)` machinery (`widget.rs:528-535`,
  the archetype `--preset`/`--spec` pattern) already renders an empty-string dropdown option as
  "(none)". **RULED SOUND — this is the fix.**

**Binding implementation constraint (the constraint ratification #7 missed):** variant (a) must be
scoped to export-wallet's `--template` rendering/materialization and must **NOT add `""` to the
shared `TEMPLATES` const** — that const feeds bundle/verify-bundle/export-wallet + the
`SINGLE_SIG_TEMPLATES`/`MULTISIG_TEMPLATES` partition-drift gate (`schema_mirror.rs:599-715`) and
the toolkit's `dropdown_value_in("--template", …)` predicates; a shared-const `""` has multi-form
blast radius and partition-drift risk. Prefer a GUI-render sentinel (or a per-flag "allow-unset"
that clears `--template` from `state.values`) that leaves the mirrored/projected template
value-set untouched. The plan-doc must explicitly assess **all** adjacent gates — flag-name
`schema_mirror`, `gui_schema_conditional_drift`, template-partition drift, per-flag `choices`, and
`archetype_schema_mirror` — and confirm each is inert under the chosen implementation.

### F2 — verified; **PREFER the descriptor-text route-around over a GUI-only mutex arm**
Verified at source: `conditional::bundle` disables `--descriptor` for `--descriptor-file`
(`conditional.rs:204-206`) and disables `--template` for `--descriptor` (`:223-224`) but **not for
`--descriptor-file`** — so the demo-seeded `--template=bip84` co-emits with a typed
`--descriptor-file` and the CLI refuses (exit 2), exactly as the spike's refusal pilot demonstrated
(`:544-577`). **Decisive verification:** the toolkit's `bundle_conditional_rules()`
(`cmd/gui_schema.rs:499-513`) projects `--template`-disabled **only for `--descriptor`, not for
`--descriptor-file`** — i.e. the GUI currently *matches* the toolkit (no drift). Therefore a
**GUI-only** `has_descriptor_file → --template Disabled` arm would put the GUI **ahead of** the
toolkit's projection; the clean form is again a **paired** toolkit+GUI conditional-rule addition
(toolkit `src/` change → breaks §12.4). **Ruling:** F2 has a clean, no-`src/` route-around the
spike already proved — ride `--descriptor` **text** (the chaining leg `bundle --descriptor <text>
--json` works because `--descriptor` DOES suppress `--template`, `:726-756`), which is exactly what
the tutorial's intra-journey chaining does (paste descriptor chunks from the prior step's output).
**Prefer the route-around;** do NOT add a GUI-only descriptor-file mutex. If production UX
nonetheless wants the descriptor-file arm hardened, that is a **separate paired cross-repo item**
the plan must call out as scope-expanding (not fold silently). F2 is therefore **NOT a required
`src/` fix** — a manifest routing decision.

### Scope-item disposition
F1 is a **NEW spec-scope item** the plan-doc must carry with its own R0, TDD tests (a reachability
test that drives export-wallet to `--descriptor` and asserts a real run; a conditional-drift-gate
regression; a `has_value`/materialization test for the unset path), corpus re-pins for the
affected form(s) in the **61-form gallery + structural renders** (the report's "≤2 of 61 forms"
is plausible — export-wallet, and bundle only if the fix touches it; verify empirically), and
hint-text/(none)-display behavior verification. F2 is a manifest-routing note, not a scope item.
**Both fixes' clap surface is unchanged** — that part of ratification #7 stands; the correction is
purely that the *conditional/dropdown* mirror surfaces (not the flag-name one) must be assessed.

---

## F3 / F6 / F7 — dispositions (each concrete, none a hand-wave)

- **F3 (materialized-defaults emit flags Examples never passes — `--language english`,
  `--format` opts[0], …):** confirmed at source as inherent to the write-back materialization
  (`widget.rs:220-229`) + `default_value: None` flags always failing `is_at_default`. This is
  **honest GUI argv, not a bug**, and the spec ALREADY disclaims byte-parity with Examples.pdf
  (§2E) and schedules a P1 divergence pass (§2). **Disposition:** (1) Chapter-0/front-matter prose
  states that GUI argv echoes carry the GUI's materialized defaults, so they are the app's real
  argv, not Examples.pdf's minimal CLI lines; (2) the §2 divergence pass explicitly covers
  argv-token divergence (not only refusal wording). No `src/` change; do NOT let F1's fix balloon
  into a general materialization-suppression change (scope creep). The plan must carry this so
  post-impl reviewers don't flag the extra tokens as a defect.
- **F6 (shots render stale `Pinned: mnemonic 0.74.0` pre-bump):** confirmed — the label is the
  schema constant (`schema/mnemonic.rs:4620` = `"mnemonic 0.74.0"`, rendered at `main.rs:517-518`)
  while the CLI is 0.75.0; this is the I1 label-honesty gap, live. **Disposition:** already fully
  covered by the spec — P1's pin bump makes the **two edits** (pin line + the `pinned_version`
  string `:4620` → `"mnemonic 0.75.0"`) BEFORE any corpus capture, and the version gate makes
  premature capture impossible. The plan must **sequence** it explicitly: bump pin + `pinned_version`
  FIRST, THEN capture; the version gate is the machine backstop. No new work; a sequencing
  reaffirmation.
- **F7 (inline-`--descriptor` argv echoes wrap to many lines, ballooning the bottom panel):**
  confirmed as a real layout consequence of the descriptor-**text** path — which, per the F1 ruling
  (export-wallet stays on `--descriptor` text) and the F2 route-around (bundle engrave rides
  `--descriptor` text), is **the path the tutorial uses**, so the ballooning is inherent, not
  avoidable by "routing to file fields" (export-wallet has no file field; the F2 file-field arm is
  the rejected src fix). **Disposition:** accept the ballooning as **viewport-faithful** per §5.4
  (a growing/scrolling panel is correct, not a defect); the manifest carries per-step scroll
  offsets for the ballooned descriptor-text steps, the §5.4 driven-field-visibility assertion
  guarantees no driven field is clipped out of *every* shot, and the full argv is byte-gated in the
  masked transcript regardless. The plan must **verify the ballooned steps render legibly at
  920×720** (narrower window wraps more than 1280) as an explicit S4-interaction check — a named
  manifest/layout item, not deferred.

---

## PR-#29 disposition — **PRESCRIBED: reuse the extraction, discard the spike scaffolding; do NOT merge #29**

I verified the split: the extraction commit `29777ee` is **production-shaped** (correct
`new`/`new_headless`/`ui`/`update` seams, `main.rs`→82 lines, tripwire scan-set extensions, and
CI-green on PR #29 including the pre-existing `snapshots` + `schema-mirror` + `headless
(no-default-features)` jobs — i.e. invisible to existing gates). The spike test
`gui_example_p0_spike.rs`, the throwaway `spike-gui-example-p0.yml`, the `tests/snapshots/spike/*`
PNGs and `tests/spike_fixtures/` are **throwaway** by construction (`GUI_TUTORIAL_SPIKE` env-gate;
the file's own header says the whole file goes away with the pin bump). **Prescription for the
plan-doc:**
1. **Reuse `29777ee`'s content** as the P1 app-shell extraction (cherry-pick or re-author on the
   post-pin-bump base), re-running the full P1 invariant suite (`cargo test --jobs 2` full package,
   `--no-default-features`, clippy `-D warnings`, both source-scan tripwires) on the rebased base —
   NOT a parallel re-implementation (§9 P1; single-implementer rule).
2. **Discard** the spike test + throwaway workflow + spike PNGs + spike fixtures. Author the REAL
   harness `tests/gui_tutorial_snapshots.rs` FRESH per §3.1(b), **inheriting the spike's proven
   mechanics verbatim in intent** — the m1 single-`step()` tripwire, the pre-render version gate,
   the popup-open + Escape-close discipline, the row-anchored/block-bounded lookup, the modal
   Window-subtree scoping, and the injected-wheel + `with_max_steps(64)` scroll. Use the real
   `tutorial-snapshots` CI job (§3.1d), not the 2-sample throwaway.
3. **Do NOT merge PR #29** (it is correctly `DO NOT MERGE`); **close it** with a pointer to the P1
   PR once the extraction lands on `master`.

---

## Findings

### Critical — none.

### Important — none open.
The only substantive gap was the spike's ratification #7 characterising the F1/F2 fix as "no
schema-mirror impact" without distinguishing the flag-name `schema_mirror` (inert) from the
**conditional-rules drift gate** (variant-(b)- and F2-GUI-mutex-entangled, verified against the
toolkit's projection). This is **resolved by RATIFICATION 4** (rule variant (a) GUI-render-scoped;
reject variant (b); prefer the F2 route-around; mandate a full adjacent-gate assessment in the
plan-doc). Because the spike itself explicitly defers the fix design to P1's own R0 and flags
corpus verification, this is a completion of an under-specified forward claim, not a spike-mechanics
defect — hence no open Important.

### Minor (fold into the plan-doc; none blocks the gate)
- **m1.** Plan-doc: encode the F1 fix as variant (a), GUI-render-scoped, with the explicit
  adjacent-gate assessment (flag-name `schema_mirror`, `gui_schema_conditional_drift`,
  template-partition, per-flag `choices`, `archetype_schema_mirror`) + 61-form/structural-render
  re-pin + hint-text/(none) behavior; give it its own R0. (RATIFICATION 4.)
- **m2.** Plan-doc: convert the corpus budget from "trims in reserve" to a HARD gate — re-measure
  at the full 51-shot manifest, firm ≤20 MiB ceiling targeting ≤15 MiB, pre-committed trim levers.
  (RATIFICATION 3.)
- **m3.** Plan-doc: carry F3 (Chapter-0 + divergence-pass prose), F6 (pin+`pinned_version`-before-
  capture sequencing), F7 (ballooned-descriptor legibility check at 920×720) as named items.
- **m4.** Plan-doc: state F2 as a manifest-routing decision (ride `--descriptor` text; the
  GUI-only descriptor-file mutex is REJECTED as toolkit-projection-divergent; any hardening is a
  separate paired cross-repo item).
- **m5.** Report accuracy nits (cosmetic): `main.rs` residual is 82 lines, not "84"; the window
  default literal is `main.rs:46`, not "`:52`" (the `:52` cite is the `with_inner_size` line). No
  action beyond noting.
- **m6.** Spec cite-nit: §4 names the spike report `gui-example-tutorial-p0-spike.md`; it lives
  (correctly, per the P0-artifact convention) as `gui-example-p0-spike.md`. Treat §4's filename as
  satisfied by the current name; fold when the spec is next touched. (No rename performed.)

---

## Gate instruction

**GREEN at 0C/0I — the P0-R0 spike gate is passed. GO is ratified; S3/S4/S5 mechanisms are
ratified; the F1/F2 scope ruling is issued (fix-in-P1, variant (a) GUI-render-scoped for F1,
route-around for F2), and F3/F6/F7 + the PR-#29 disposition are prescribed.** The implementation
plan-doc may now be written; it MUST honor the binding constraints above (F1 as a new R0-gated
scope item with the full adjacent-gate assessment; the corpus budget gate; the pin-before-capture
sequencing; the extraction-reuse/spike-discard PR-#29 prescription) and then take its own R0 per
§9 / the house per-phase gate.

---

## Cites (all live-verified 2026-07-05)

- mnemonic-gui `spike/gui-example-p0` @ `80f7eb8`: `src/app_window.rs:213/223/359/1122-1123`
  (extraction seams); `src/main.rs:46` (window default `[920,720]`), `:517-518` (Pinned label =
  schema constant); `tests/gui_example_p0_spike.rs:82` (max_steps 64), `:214-225` (tripwire),
  `:292-314` (version gate + BITE), `:434-532` (S2i), `:544-577` (S2iii refusal), `:581-711`
  (S2ii modal), `:726-756` (S5 chain), `:811-955` (S3/S4 + wheel).
- mnemonic-gui source: `src/form/widget.rs:220-229` (materialize+write-back), `:528-535`
  (`display_or("(none)")`); `src/form/flag_defaults.rs:29-31/76` (Dropdown default = opts[0]);
  `src/schema/mod.rs:388` (`has_value`); `src/schema/mnemonic.rs:69` (`TEMPLATES`, no `""`),
  `:1384` (export-wallet `--template` default_value None), `:4620` (`pinned_version "mnemonic
  0.74.0"`); `src/form/conditional.rs:204-208/223-224` (bundle), `:596-601` (export-wallet mutex);
  `tests/schema_mirror.rs:52-124` (flag-NAME gate), `:599-715` (template-partition drift);
  `tests/gui_schema_conditional_drift.rs` (conditional-rule drift gate).
- mnemonic-toolkit: `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:412-513`
  (`bundle_conditional_rules` — `--template` disabled for `--descriptor` ONLY, not
  `--descriptor-file`), `export_wallet_conditional_rules` (symmetric template↔descriptor mutex);
  `docs/manual-gui` current PNG corpus = 2.12 MiB / 61 forms.
- egui `0.31.1` (vendored): zero scroll-action handlers; handled set = Focus/Click/Increment/
  Decrement/SetValue.
- CI: `gh pr view 29` — 15/15 (spike-sample-1/2, schema-mirror, snapshots, headless
  no-default-features, msrv, all target builds) SUCCESS; release SKIPPED.
