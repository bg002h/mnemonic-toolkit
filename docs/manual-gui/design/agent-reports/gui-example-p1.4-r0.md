# P1.4 per-phase R0 — gui_example tutorial capture harness + pilots (the DE-RISK gate)

**Reviewer:** opus architect (adversarial; gates re-run, not trusted from the report).
**Scope:** `mnemonic-gui` branch `feat/gui-example-leg1` @ `e9b0e46` (P1.4), diff `52f7689..e9b0e46`
(tests/docs/fixtures only — **zero `src/`**, verified). Authority: plan P1.4 + spec §3.1/§4/§5/§6/§7/§8/§12
+ `agent-reports/gui-example-p0-spike.md` + F1 mini-R0.
**Environment:** `mnemonic 0.75.0` on `$PATH` (matches pinned tier); llvmpipe GL software rasterizer
(`device_type == Cpu`, backend `Gl`) — no local lavapipe, so cross-backend faithfulness to the
CI-authored corpus was measured at the 0.6 dify threshold and passed.

---

## VERDICT: **GREEN (0 Critical / 0 Important).** P1.5 may generate the full 51-shot corpus.

The genuinely risky part — whole-window fidelity, populated-pane **same-frame** determinism,
end-to-end secret masking, byte-reproducibility, and the two fail-closed gates — is **fully
de-risked and empirically proven**. Findings are Minors only (diagnostic-quality + forward-scope
notes for P1.5); none poisons the downstream corpus or re-opens a de-risked contract.

Counts: **0C / 0I / 5 Minor**.

---

## 1. Real-window capture — CONFIRMED (no bypass, no faked shell)

The harness drives the **production shell**, not a reconstructed subset:
`Harness::builder()…build_state(|ctx, app: &mut MnemonicGuiApp| app.ui(ctx), app)` over
`MnemonicGuiApp::new_headless(fixed_appstate_all_found(), None, None)`
(`tests/gui_tutorial_snapshots.rs:576-583`). `ui()` is the verbatim former `eframe::App::update`
whole-window body (`src/app_window.rs:359`); `update()` delegates to it — so the harness renders the
same tab bar + subcommand combo + form + output panel the shipping binary renders.

Opened all four committed pilot PNGs:

- **`tut-ch0-00-orientation-form.png`** — the REAL shell: title `mnemonic-gui`, tab bar
  `mnemonic | md | ms | mk`, **`Pinned: mnemonic 0.75.0`**, subcommand ComboBox = `bundle`, the full
  bundle form, the demo-seed slot row (`@ 0 . xpub =` empty) + `+ Add slot`, the `Run` action bar,
  the `Preview:` line, and the output panel showing **`(no run yet)`**. Exactly the spec-§5.3
  orientation shot.
- **`tut-j1-01-bundle-single-sig-modal.png`** — the REAL `Confirm secret-bearing run` egui Window
  overlaid on the live app; the slot row now reads `@ 0 . phrase = ••••` (masked PasswordInput);
  the modal argv list + the `Preview: … --slot ••••` line. Not a faked modal.
- **`tut-j1-01-bundle-single-sig-run.png`** — the output panel **populated by the real CLI**:
  `argv: mnemonic bundle --network mainnet --template bip84 --language english --slot ••••`
  (masked), `ex:0`, stdout carrying the ms1/mk1/md1 card set, stderr carrying the two production
  warnings. Byte-identical to the committed transcripts.
- **`tut-j1-01-bundle-single-sig-form.png`** — the filled form pre-Run.

`spawn_and_capture` (`src/app_window.rs:1215-1255`) calls `runner::run_with_stdin` **synchronously**
and assigns `app.last_run` in the same call → the populated pane is guaranteed same-frame **by
construction**; the P1.2 pointer comment citing the SPEC §6.5 contract is in place
(`src/app_window.rs:1205-1214`). **Confirmed: whole-window, real `ui()`, real Run clicks, real
populated panes.**

---

## 2. Secret hygiene end-to-end — CLEAN (no leak)

**(a) Right taxonomies, non-vacuous.** The allowlist checker classifies via
`slot_subkey_is_secret` (SECRET_SLOT_SUBKEYS), and `node_secret_taxonomy_nonempty()` exercises
`node_type_is_argv_secret("phrase")` (SECRET_NODE_TYPES_ARGV) + `!SECRET_FLAG_NAMES.is_empty()`
(`tests/tutorial/mod.rs:82-87, 329-331`) — the sibling taxonomies the spec §7 mandates, **not** the
I3 64-flag census. Non-vacuity is asserted (`secret_drive_count() > 0`, driven by J1's S0 phrase).
**BITE proven:** injecting a non-allowlisted secret value into J1's `TypeSlot` made
`secret_values_are_allowlisted` FAIL with the exact §7 message
(`…a secret-classified drive carries a NON-allowlisted value ("attacker injected…")`), then reverted.
The sweep genuinely bites on J1.

**(b) No plaintext in any committed artifact; masking is REAL, not pre-masked.** The committed
transcripts carry only ms1/mk1/md1 encodings + the CLI's own `--slot @0.phrase=` warning (truncated
at `=`, no phrase). The masking is genuine: the harness types the **real S0 phrase** into a
PasswordInput slot, and the run pane's argv echoes `--slot ••••` while stdout carries the **real
derived card set** whose fingerprint `73c5da0a` is the all-`abandon` vector — proving the real
secret was processed by the real CLI and then display-masked (`result.mask` carried through
`spawn_and_capture`, rendered via `render_copy_command_masked`, `src/app_window.rs:453-460`).
The per-step argv-mask assertion confirms the mask bit is set on the secret token
(`tests/gui_tutorial_snapshots.rs:425-433`).

**(c) Modal + pane masking both hold.** Whole-tree no-plaintext (`assert_no_plaintext`,
`tests/gui_tutorial_snapshots.rs:722-736`) fires at all four checkpoints for the secret step —
filled form (full S0 + first word), confirm modal, and populated pane
(`:367-379, :435-441, :493-497`); `has_mask_sentinel` asserts `••••` present before Run. Fixtures are
watch-only by construction: `fixtures_carry_no_secret_material` scans every non-README fixture for
the three phrases + `xprv`/`tprv`/` wif `/`-----BEGIN` and requires ≥4 scanned; the `.desc`/`.json`
carry only public xpubs + the public `opensessame` hashlock digest + fingerprints. **No leak
anywhere in the pipeline.**

---

## 3. Gate teeth — both gates are REAL predicates over REAL state, and BITE

**`pinned-tier-version-gate` — real, not a tautology.** `version_matches(cli, got, expected)` is a
genuine `got == expected` comparison (`tests/tutorial/mod.rs:339-349`). The LIVE gate probes an
**actual** `mnemonic --version` subprocess (`probe_version`, `:914-920`) against
`SCHEMA.pinned_version` (`expected_pinned_version`, `:904-912`) BEFORE any render
(`run_pinned_tier_version_gate`, `:833-840`). The suite-pinned negative bites; I additionally proved
the **LIVE** bite: perturbing `schema/mnemonic.rs` `pinned_version → "mnemonic 0.74.0"` made the
env-gated harness **panic at the gate before any capture** (`…= "mnemonic 0.75.0", expected
"mnemonic 0.74.0" — refusing to render or spawn from a wrong tier`), then reverted. A wrong-tier
local regen cannot produce honest-looking bytes.

**`SAME-FRAME-COMPLETION` — real, observes actual state.** `same_frame_completion(run_landed, what)`
is fed `h.state().last_run.is_some()` after **exactly one** `harness.step()` delivered the queued
click (`step_once_same_frame`, `tests/gui_tutorial_snapshots.rs:703-707`, ruling-9 single-`step()`).
`run_direct`/`run_via_modal` first assert `last_run.is_none()` (proving the same-frame check is
non-vacuous), step once, then assert landed — a real none→some transition over real app state, not a
hardcoded string. The negative unit bites (`…SAME-FRAME-COMPLETION violated…`).

**The direct-click-class proof is legitimate, not a gap dressed up.** J1 exercises the secret
**two-click** modal path; `same_frame_completion_direct_click_class`
(`tests/gui_tutorial_snapshots.rs:859-895`) covers the **one-click** class the pilots omit by driving
a reachable non-secret bundle run (a real public bip84 xpub → watch-only, no confirm modal),
single-click Run through the **same** `step_once_same_frame` helper the corpus uses, asserting real
state (`last_run.is_some()` same-frame, `exit_code == Some(0)`, no modal) and capturing **no**
artifact (nothing enters the census). It ran live and passed
(`TUTORIAL-SAMEFRAME-DIRECT: direct-click class holds (exit 0, no modal)`). It genuinely exercises
the production single-click path.

---

## 4. Byte-determinism — PASS

- **Two forced-regeneration runs (same GL env) are byte-identical** across all 4 PNGs + 3
  transcripts (sha256-compared).
- **The regenerated corpus equals the committed bytes exactly** (`git diff` on
  `tests/snapshots/tutorial/` empty after two UPDATE runs) — the committed pilot corpus reproduces
  bit-for-bit locally.
- **Cross-backend faithful:** compare-mode under llvmpipe GL landed within the 0.6 dify threshold of
  the CI-authored corpus (no `.new.png` divergence written).
- Determinism contract honored: `WINDOW_SIZE = [920.0, 720.0]`, `PPP = 2.0`, `with_max_steps(64)`,
  demo-seed baseline (`new_headless(…, None, None)`), single sequential `#[test]` with one
  `set_current_dir` to the fixture dir (`:293`), injected `PointerMoved` + `MouseWheel{unit: Point}`
  scroll (`wheel_scroll_form`, `:675-696`), run-to-settle. (`.new.png` on disk are gitignored litter,
  correctly ignored; the tracked corpus is 4 PNGs.)

---

## 5. Manifest-as-truth — PASS

Nothing hardcodes 4/25/51. `total_shots()` sums `figure_stems().len()` and `corpus_manifest()`
derives from `MANIFEST` (`tests/tutorial/mod.rs:245-290`); the literals appear only in comments.
`manifest_stems_regen_diff` is an **always-run** `#[test]` (ran under plain `cargo test`) that
regenerates the payload and `assert_eq!`s it against committed `manifest-stems.txt`, plus a
uniqueness + sorted check. `corpus_png_count_matches_manifest` ties the committed PNG count to
`total_shots()`. The census emitter + figure/transcript basename lists all fold from the manifest.

---

## 6. Budget assert — PASS (wired + HARD)

`corpus_budget_under_ceiling` is always-run, sums the committed PNGs, and **panics** above
`BUDGET_HARD_MIB = 20.0` (`tests/gui_tutorial_snapshots.rs:210-227`), reporting above the 15 MiB
target. Pilots measured **1.147 MiB / 4 PNGs**. It re-measures the real corpus at P1.5 (reads
committed bytes each run).

---

## 7. P1.5-READINESS — the harness is a de-risked SPINE; P1.5 EXTENDS it, it does not need SURGERY

The de-risk question ("does this exact machinery scale, or will P1.5 need harness surgery?") splits
cleanly:

### REUSE (proven, generic — zero change)
- The whole-window harness (`app.ui(ctx)` / `new_headless` / 920×720 / ppp 2.0 / max_steps 64).
- Subcommand combo selection for **any** subcommand (`combo_select_subcommand`).
- Slot-editor row drives (`FlipSlotSubkey`, `TypeSlot`) + the 5-rule lookup discipline
  (`on_row_of`, `modal_scoped_run_button`, `close_popup`).
- **Both** Run classes — direct one-click and secret-modal two-click — proven end-to-end.
- The two named gates + secret guards + transcript persistence + census/budget/allowlist gates.
- **Refusal steps need NO addition:** `expect_exit: Option<i32>` + the executor's `exit_code ==
  expect_exit` assertion (`:407-413`) already handle non-zero exits + `expect_stderr`, so J3
  build-descriptor / J4 depth-2 / BSMS refusals are pure manifest data.
- Scroll machinery (`wheel_scroll_form` + `scroll: &[f32]` + `-formN`) is **implemented** but
  unexercised by pilots (recorded no offsets); J4/J5 tall forms exercise it first (ruling 2).

### ADD (P1.5 must author — additive interpreter arms + Step fields, over the SAME spine)
1. **New `Drive` variants for non-slot widgets.** The enum today covers **only** slot rows. J2–J5
   require typing descriptor TEXT (F2 route-around), `--spec` / `--descriptor-file` **Path** fields,
   `--md1` repeating rows, dropdown selection (`--template` NUMS export, `--md1-form`), booleans,
   and a **button-click** drive (`+ Add slot` for J2's 3-slot multisig; `+ add` for md1 row bands).
   Each is a named interpreter arm reusing the proven AccessKit-lookup pattern — the enum + Step are
   the explicit extension points (`#![allow(dead_code)]` already anticipates this; `mod.rs:25,51-55`).
2. **Intra-journey chaining — the ONE un-prototyped mechanism (highest forward uncertainty).** The
   executor builds a FRESH app per step (`execute_step` → `app_harness()`) with **no** cross-step
   state, and `Drive` values are `&'static str`. Chaining (parse md1 chunks/descriptors from step N's
   `RunResult.stdout` → type into step N+1) needs a **runtime-value channel** (a `&'static` field
   cannot hold derived text) + executor threading of the prior RunResult. This is scoped to P1.5 by
   spec §3.1b / plan §108-110 and the pilots correctly don't exercise it — so it is **not a P1.4
   defect** — but it is the mechanism P1.4 leaves unproven. **Recommendation: P1.5 prototype chaining
   FIRST (a single J5-restore or J3-chain step) before the full 25-step build, so a chaining blocker
   surfaces early rather than deep in the corpus.**
3. **`shots: 0` expressiveness (Minor gap in the current schema).** `Step` has no capture-gating
   field; `figure_stems()` unconditionally emits `-form` (+ `-run` if `runs`), so a transcript-only
   step would over-emit PNGs and inflate `total_shots()` → the dual census would then demand PNGs the
   `shots: 0` step must not produce. P1.5 **must add** a capture control and gate `figure_stems()` +
   `total_shots()` + the snapshot calls on it.
4. **Multi-offset driven-field visibility.** The §5.4 check runs at the base offset only (`:348-363`,
   comment acknowledges); P1.5 extends it across recorded scroll offsets.
5. **md/ms/mk version-gate wiring.** `expected_pinned_version` panics (fail-closed) for non-mnemonic
   CLIs; §5.3 is all `mnemonic <sub>`, so likely none spawn md/ms/mk — but P1.5 wires them if any step
   does.

**Manifest schema expressive enough?** For the pilots, yes. For P1.5 it needs three additive
extensions — (a) `shots`/capture control, (b) a runtime/chained value channel, (c) non-slot Drive
variants — all cleanly additive at the documented extension points; **none re-opens the proven
determinism / hygiene / gate spine.** The harness header line "P1.5 grows the corpus by extending the
manifest, **not this harness**" (`tests/gui_tutorial_snapshots.rs:26`) is mildly optimistic — P1.5
*will* extend the harness — but `mod.rs:51-55` states it accurately ("P1.5 adds variants as its steps
need them"). This is a doc reconcile, not a design flaw. **Net: a de-risked spine, not a J1-only
harness.**

---

## 8. Suite / collision — PASS

- **Full package suite green** (`cargo test --jobs 2`): every `test result: ok`, zero failures.
- **`gui_form_snapshots` 61/61 UNCHANGED** (2 tests green under GL); `tests/snapshots/tutorial/` is a
  **disjoint** directory from `tests/snapshots/forms/` — no corpus collision.
- **Always-run tutorial gates** (no rasterizer): 9/9 pass under plain `cargo test`, and the egui-free
  `tutorial` module (census / allowlist / gate negatives) compiles + runs without the `gui` feature.
- **clippy both configs:** `cargo clippy --all-targets -- -D warnings` = exit 0;
  `cargo clippy --no-default-features -- -D warnings` (the exact headless CI command,
  `build.yml:59-60`) = exit 0.
- **`cargo build --no-default-features`** = exit 0 (the load-bearing headless edge; `app_window` is
  `#[cfg(feature = "gui")]`, `src/lib.rs:11-12`, and the gui-gated test binary is not compiled in the
  headless job).
- Zero `src/` changes in the P1.4 diff (verified). Working tree pristine after all perturbations;
  HEAD `e9b0e46`.

---

## 9. Findings by severity

**Critical (0):** none.
**Important (0):** none.

**Minor:**
- **m1 — demo-seed baseline is pinned implicitly, not by an explicit named assertion.** Plan §93 /
  spec §6.3 ask for the demo seed "asserted, not assumed." Today it is enforced only by the ch0
  orientation snapshot byte-compare + the row-0 drive dependency (J1 `FlipSlotSubkey`/`TypeSlot` on
  `occ=0`, and the direct-click test's `rows[0]` assert). A future demo-seed change (the
  `new_headless` comment anticipates a template-aware re-seed) would surface as a snapshot pixel-diff
  or a `slots.rows[occ]` index panic rather than a named breach. **Recommend** a one-line explicit
  baseline assertion at harness start in P1.5 (e.g. `form_state` contains `"mnemonic:bundle"` with a
  single empty Xpub row). Cite: `tests/gui_tutorial_snapshots.rs:576-583`; `src/app_window.rs:270-286`.
- **m2 — harness header overstates P1.5 as manifest-only** (`tests/gui_tutorial_snapshots.rs:26`
  "not this harness"); `mod.rs:51-55` is accurate. Doc reconcile at P1.5.
- **m3 — `Step` cannot express `shots: 0`** (no capture-gating field; `figure_stems()`/`total_shots()`
  unconditional). Required P1.5 addition; harmless for the shot-bearing pilots. Cite:
  `tests/tutorial/mod.rs:105-181, 275-277`.
- **m4 — driven-field visibility check is base-offset only** (`tests/gui_tutorial_snapshots.rs:348-363`);
  extend across recorded scroll offsets in P1.5.
- **m5 (advisory, not a P1.4 defect) — intra-journey chaining is un-prototyped.** See §7 item 2;
  recommend P1.5 prototype it before the full corpus build.

None blocks P1.5. No fix is required before P1.5 begins; m1–m4 are natural P1.5 folds and m5 is a
sequencing recommendation for P1.5.

---

## 10. Bottom line

P1.4 delivers exactly what a de-risk gate must: the capture machinery that will generate the full
51-shot corpus is **proven** to drive the real window, populate panes same-frame from the real
pinned CLI, mask secrets end-to-end (typed real S0 → `••••` everywhere, real card set out), gate
wrong-tier + async-runner regressions with genuinely-biting predicates, and reproduce byte-for-byte.
The remaining journeys need additive interpreter arms over this same spine, with intra-journey
chaining as the one mechanism worth an early P1.5 prototype. **GREEN — P1.5 may generate the full
corpus.**
