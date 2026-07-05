# P0 spike report — gui_example.pdf tutorial cycle (SPEC §4, S1–S5)

> Provenance: P0 spike implementer report, persisted per house convention.
> Cycle: `gui_example_tutorial` (spec `SPEC_gui_example_tutorial.md`, R0-GREEN ×2;
> round-2 m1 folded into the spike mechanics as prescribed).
> Note: SPEC §4 names this file `gui-example-tutorial-p0-spike.md`; the
> orchestrator's dispatch named `gui-example-p0-spike.md` — written to the
> dispatched name, rename at fold time if desired.
>
> Executed 2026-07-05 on mnemonic-gui branch `spike/gui-example-p0`
> (base `master@0d4429d` v0.55.0). Commits:
> - `29777ee` — S1 app-shell extraction (pure relocation → `src/app_window.rs`)
> - `1a1a826` — S2–S5 spike harness + 11 pilot baselines + fixture
> - `80f7eb8` — throwaway 2-sample lavapipe CI workflow
> Draft PR (CI confirmation): https://github.com/bg002h/mnemonic-gui/pull/29
> Local env: llvmpipe-GL (`WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1`,
> Mesa 26.1.2, device_type Cpu), ppp 2.0, dark, debug build,
> `cargo test --jobs 2`. CI env: lavapipe-Vulkan (mesa-vulkan-drivers,
> `WGPU_BACKEND=vulkan`), 2 runner samples.

## VERDICT: **GO** — every user-locked contract element proven end-to-end

Whole-window shots of the REAL `ui()` loop, output panes genuinely populated
by real pinned-CLI executions landed by real Run clicks (one-click non-secret,
two-click modal secret, refusal non-zero exit), two-shots-per-step (+modal),
deterministic to the byte locally and sub-threshold cross-backend. No STOP
condition was approached; S3/S4/S5 ratified their mechanisms/values. Four
journey-level findings need P1 attention (F1 is chapter-blocking for J2/J3/J4
export-wallet steps — see FINDINGS), but none touches the locked contract's
mechanics.

---

## S1 — whole-window render determinism: **GREEN**

- Extraction prototype: `MnemonicGuiApp` lifted verbatim into gui-gated
  `src/app_window.rs` (struct+impls+`render_exit_badge`+`spawn_and_capture`+
  `AUTOSAVE_INTERVAL`). Diff vs original body = **9 hunks, all seam-only**
  (header/imports; pub on 5 harness-read fields; `pub new` doc; `new_headless`
  split at exactly the `detect_all()` line, order-preserving; `ui()` split with
  `update()` delegating). `main.rs` → 84 lines (bootstrap only).
- Invariants held: full suite green (`cargo test --jobs 2`, 74 result lines,
  0 fail), `cargo build --no-default-features` green, clippy
  `--all-targets -D warnings` green. Two source-scan tripwires extended their
  scan set to `app_window.rs` (`paste_warn_wiring_v0_40_0` T-B4,
  `r7_no_auto_repair_removal` source anchor) — the guarded surfaces moved with
  the relocation; main.rs stays scanned. NOT fmt'd (house rule; relocation
  stays verbatim).
- Harness: `Harness::builder().with_size(...).with_pixels_per_point(2.0)
  .with_max_steps(64).build_state(|ctx, app| app.ui(ctx), app)` over
  `new_headless(fixed_appstate_all_found(), None, None)`. Panels + combo +
  pane + action bar all present in the AccessKit tree and the pixels.
- **Same-env byte identity:** two independently-built harnesses render
  IDENTICAL raw RGBA buffers (10,598,400 bytes @ 1840×1440). Cross-process:
  aggregate sha256 over all 11 `.new.png` identical across two full test-process
  runs (`033bc3805e5e…9260`), and every `.new.png` byte-equal (`cmp`) to its
  committed baseline.
- **Cross-backend (Vulkan-lavapipe ↔ GL-llvmpipe) at dify 0.6:** CI samples on
  the draft PR — see CI CONFIRMATION below.

## S2 — live Run-click populated pane: **GREEN** (with one pilot reshape)

**Pilot reshape (spec cite slip + finding F1):** the spec's S2(i) names
"export-wallet with `--descriptor-file`" — export-wallet has NO such flag
(recon-C's `schema/mnemonic.rs:851` cite is VERIFY_BUNDLE_FLAGS; bundle is
`:294`), and the export-wallet `--descriptor` TEXT arm is unreachable in the
production GUI (finding F1). The non-secret pilot therefore used the REACHABLE
export-wallet template+slot mode — mechanics identical (one click, no modal).

- **(i) non-secret** (`export-wallet --template bip84 + @0.xpub slot +
  --format descriptor`): ComboBox POPUP drive explicit (open popup on its own
  layer → option click by human_name → Escape-close, see F4) — popup-open
  state captured as `s2-combo-popup-920x720.png`. Run click delivered via
  **exactly ONE `harness.step()`** (m1): `last_run` None before the step
  (queued click; proves non-vacuity), `Some` immediately after — BEFORE any
  further stepping — then `run()` settles for the shot. Pane: bare
  `argv[0]=="mnemonic"`, `exit: 0`, real stdout
  (`wpkh([00000000/84'/0'/0']xpub6Cat…#…`, 155 B), stderr note. Public
  `@0.xpub=` slot token asserted UNMASKED (`mask[i]==false` — the
  SECRET_SLOT_SUBKEYS taxonomy boundary, exercised from the public side).
  **Double-run identity:** independent harness, same drive → stdout
  byte-identical AND rendered pixels byte-identical.
- **(ii) secret (J1 bundle, S0):** starts from the fresh-app DEMO SEED
  (asserted: `mnemonic:bundle` pre-filled, one EMPTY Xpub slot row — the §6.3
  baseline); the seeded row's subkey flipped Xpub→phrase through the REAL
  subkey-combo popup (S5/M2ii exit); S0 typed into the now-PasswordInput row.
  Masked Preview asserted (`••••` present, no plaintext). Run click #1 (ONE
  step): modal `Confirm secret-bearing run` appears in the SAME frame,
  `last_run` still None (deferral asserted). **Exactly 2 `Run` buttons** with
  the modal open (collision demo) → modal-Run resolved by **Window-subtree
  scoping** (`by().role(Window).label("Confirm secret-bearing run")` then a
  scoped query — egui Windows do carry role+title). Modal argv masked
  (`--slot ••••`), captured as `s2-j1-modal-920x720.png`. Modal-Run click (ONE
  step) → `SAME-FRAME-COMPLETION` holds through the modal path → settle →
  `s2-j1-run-920x720.png`: exit 0, masked argv echo, real J1 card set on
  stdout (`# ms1 (entropy, BCH-checksummed)…`). Slot token IS cleartext in
  the spawned argv with `mask[i]==true` (display-masked), as designed.
- **(iii) refusal probe (bonus, J3/J4-class mechanics):** bundle
  `--descriptor-file policy.desc` + the demo-seeded template → CLI exit 2
  ("mutually exclusive") rendered as a REAL non-zero exit badge + stderr block
  (`s2-bundle-refusal-920x720.png`). Refusal steps' "real non-zero exit"
  contract (SPEC §12.2) is mechanically proven. (Also the F2 demo.)
- **Secret hygiene:** whole-tree no-plaintext assertion (every AccessKit node
  label AND value) at four checkpoints (filled form, modal open, post-run;
  plus the word-level "abandon" probe). egui 0.31 masks password values
  BEFORE AccessKit (`text_edit/builder.rs` `mask_if_password` feeds widget
  info; role PasswordInput) — so the text-channel guard is global with zero
  exclusions. All GREEN.

## SAME-FRAME-COMPLETION tripwire + pinned-tier-version-gate: **both shown to BITE**

- **Version gate** (`gen.sh:22` pattern, §3.1b): probes `<cli> --version`
  before ANY render/spawn. **Bite demo:** probing the installed binary
  (0.75.0) against the CURRENT schema constant (`mnemonic 0.74.0`,
  `schema/mnemonic.rs:4620`) hard-fails with the named diagnostic —
  exactly the wrong-tier-regen scenario (log line `SPIKE-GATE-BITE`). The
  real gate then passes against the spike tier (`mnemonic 0.75.0` — the P1
  pin-bump target, deliberately ahead of `pinned-upstream.toml`'s v0.74.0).
  Local tier inventory (dispatch asked): `mnemonic 0.75.0`, `md 0.11.3`,
  `ms 0.13.2`, `mk 0.11.2` — ALL ahead of the GUI pins
  (v0.74.0/v0.11.0/v0.13.0/v0.11.0); `md` is additionally shadowed by a fish
  alias (`md: aliased to mkdir -p`) in interactive shells only — subprocess
  `$PATH` resolution is unaffected (probed `~/.cargo/bin/md` = 0.11.3). This
  live skew is precisely what the gate must catch, and did.
- **Tripwire** (m1 semantics): click queued → asserted `last_run.is_none()`
  (not yet stepped) → exactly ONE `step()` → assert `last_run.is_some()`
  BEFORE any further stepping → THEN `run()` to settle for the snapshot. The
  pre-step None assertion proves the post-step Some assertion is non-vacuous;
  a fast-settling async redesign cannot slip past a single-frame window.
  Applied per-run at all four click sites (S2i, S2ii modal, S2iii refusal,
  S5 chain).

## S3 — scroll positioning: **mechanism (ii) RATIFIED — injected MouseWheel**

- **(i) AccessKit scroll actions: DEAD.** egui 0.31 handles NO scroll-class
  AccessKit requests (grep over egui-0.31.1 src: zero
  ScrollIntoView/ScrollUp/ScrollDown/SetScrollOffset handlers; only
  Click/Focus/SetValue/Inc/Dec/SetTextSelection).
- **(ii) Injected wheel: WORKS + deterministic.** `PointerMoved` to a point
  inside the form ScrollArea, then `Event::MouseWheel { unit: Point,
  delta: (0, -520) }`, then `run()` (smooth-scroll animation settles;
  requires `with_max_steps(64)` — kittest's default 4 is too tight, ~8
  frames observed). On the journey-tall restore form (10 `--md1` rows) at
  920×720 the content scrolled to its deterministic clamp
  (`--md1` header y 403.75 → 251.25 pt); **pixel-reproducible**: an
  independent harness driven identically renders byte-identical RGBA.
  Delta-in-points is manifest-recordable → per-step scroll offsets are
  expressible as wheel deltas (clamping makes "scroll to bottom" trivially
  stable; intermediate offsets land exactly at the delta when unclamped).
- **(iii) `vertical_scroll_offset` seam: NOT NEEDED** (unused; remains the
  guaranteed fallback if a future egui bump changes wheel semantics).
- Bonus mechanic finding: AccessKit-driven interactions (click/focus/
  type_text) are id-addressed and pointer-free — they work on widgets
  OUTSIDE the current viewport. Drives never need scrolling; only SHOTS do.
  This decouples manifest drive order from scroll offsets.
- §5.4 driven-field-visibility contract prototyped: per driven md1 row,
  assert its rect intersects the viewport in ≥1 captured offset — GREEN.

## S5 — drive-in-window: **GREEN; lookup discipline ratified**

Ratified discipline (all exercised):
1. **Unique-label direct query** for labelled widgets: the subcommand
   selector is `by().role(ComboBox).label("subcommand")` (egui
   `ComboBox::from_label` labels the combo node; flag dropdowns from
   `from_id_salt` have empty labels — no collision). Buttons by exact label
   (`Run`, `+ Add slot`, tab names, popup option rows).
2. **Row-anchored geometric lookup** for UNLABELLED inputs (`on_row_of`):
   the flag-name label node anchors a horizontal band; the target-role
   widget on that band, left-to-right. Used for TextInput (--descriptor,
   --descriptor-file, slot value), ComboBox (--template/--format/slot
   subkey), CheckBox (--json). Exact-label matching prevents
   `--descriptor`/`--descriptor-file` prefix collisions.
3. **Block-bounded row targeting** for repeating-flag rows: md1 row inputs =
   TextInputs between the `--md1` header and the `--cosigner` header
   (`md1_row_inputs`), "type into last empty". The `+ add` collision (one
   per repeating flag) resolved by header-band anchoring
   (`assert_eq!(adds.len(), 1)` inside the band).
4. **Window-subtree scoping** for modal-vs-action-bar `Run` (2 buttons
   proven present; egui Window nodes carry role+title; kittest `Node` is
   `Queryable` over its subtree).
5. **Popup-close discipline (F4):** Escape after every AccessKit popup
   option click (`popup.rs:453`); AccessKit clicks have no pointer position
   so `clicked_elsewhere()` never fires and the popup lingers into the next
   shot (caught visually in the first render — see FINDINGS).
- Demo-seed control: asserted baseline (Xpub row, empty value, bundle
  pre-fill) and flipped to phrase per the J1 path (M2ii) — the Chapter-0
  baseline is controlled, not assumed-empty.
- Chaining (J5 mechanic): GUI-driven `bundle --descriptor <vault> --json`
  run (real click, shots:0-style) → parsed 24 md1 chunks from the captured
  `RunResult.stdout` → typed into restore's repeating rows. Chained values
  parse; rows round-trip (`FlagValue::Text(chunk)` asserted per driven row).

## S4 — window-size ratification: **RECOMMEND 920×720** (+ measurements)

| shot (dark, ppp 2.0) | logical | physical | PNG bytes | render+snapshot ms (debug, llvmpipe) |
|---|---|---|---|---|
| s1-freshapp | 920×720 | 1840×1440 | 223,719 | 844 |
| s1-freshapp | 1280×900 | 2560×1800 | 262,406 | 1,344 |
| s2-combo-popup | 920×720 | 1840×1440 | 325,718 | 911 |
| s2-exportwallet-run | 920×720 | 1840×1440 | 360,021 | 1,173 |
| s4-exportwallet-run | 1280×900 | 2560×1800 | 481,946 | 1,849 |
| s2-j1-modal | 920×720 | 1840×1440 | 309,895 | 1,220 |
| s2-j1-run | 920×720 | 1840×1440 | 432,080 | 1,295 |
| s2-bundle-refusal | 920×720 | 1840×1440 | 288,468 | 1,174 |
| s3-restore-top | 920×720 | 1840×1440 | 425,157 | 1,315 |
| s3-restore-scrolled | 920×720 | 1840×1440 | 433,184 | 1,275 |
| s4-restore-top | 1280×900 | 2560×1800 | 516,022 | 2,246 |

(Bytes above are the FIRST baseline set; the committed popup-free set is
slightly smaller — total 3,714,155 B / 3.54 MiB for 11 shots.)

- **Recommendation: 920×720** (candidate A), for three reasons:
  1. **Production fidelity** — it IS the app's default window seed
     (`main.rs:52`): "what the user sees at this moment" is the tutorial's
     stated contract (§5.4), and Chapter 0 shows the literal first-launch
     window.
  2. **Page legibility** — the book renders shots at page width; 1840 px
     wide keeps UI text ~28% larger on paper than 2560 px. A tutorial is
     read, not zoomed.
  3. **Corpus bytes** — A-shots average ~330 KiB vs B ~420 KiB.
- Scrolling consequence, measured: the journey-tall filled restore form
  (10 md1 rows) needs ONE scroll state at A (deterministic clamp, 152.5 pt)
  and fits entirely at B (last row bottom y=635 < 900). So B would eliminate
  most `-form2` shots, but the spec already embraces viewport-faithful
  scrolling (§5.4: "a visible scrollbar is correct, not a defect"), and the
  gallery remains the full-form canonical reference. Expect a handful of
  two-offset steps at A (restore-class forms only; bundle/export-wallet
  filled states fit at A with the pane present).
- **Corpus budget projection (51 shots):** pilot mix mean 337 KiB → **~16.4
  MiB per repo** (GUI corpus + toolkit byte-copies). Above the spec's 5–15
  MiB estimate, under the ~20 MiB flag threshold — plan should note it and
  keep §5.3's trim options in reserve (drop one modal shot, prune -form2).
- **Time projection:** 11 shots + 8 harness builds + 5 live CLI runs =
  ~12.5–15.5 s locally (debug). 51 shots / 25+ steps extrapolates to
  ~60–90 s test body; CI adds ~4–6 min for `cargo install` of the pinned
  CLI (cacheable). Well inside normal job budgets.

## CI CONFIRMATION (new shot class on the fleet rasterizer)

- Throwaway workflow `.github/workflows/spike-gui-example-p0.yml` on draft
  PR #29: lavapipe recipe (`mesa-vulkan-drivers`, `WGPU_BACKEND=vulkan`),
  pinned `mnemonic-toolkit-v0.75.0` installed, matrix sample [1,2], census
  11 × `.new.png`, `.diff.png` artifacts on failure.
- Baselines under comparison were generated on GL-llvmpipe → a PASS is the
  full Vulkan↔GL cross-backend swap at the default 0.6 dify threshold — the
  same bar the visual-track spike met.
- **Result: BOTH samples GREEN** (run 28733883693: `spike-sample-1` +
  `spike-sample-2` success; census 11/11 `.new.png` on each; zero diff
  artifacts; the gate BITE + GATE-OK lines reproduced on both runners). CI
  adapter: `llvmpipe (LLVM 20.1.2) … device_type: Cpu, backend: Vulkan,
  Mesa 25.2.8` — i.e. the lavapipe ICD (self-reports as llvmpipe, the known
  naming), so the pass covers BOTH a backend swap (GL→Vulkan) AND a Mesa
  delta (26.1.2 local → 25.2.8 CI) on 2 independent runners. Suite time on
  the runners: 35–38 s.
- The pre-existing `build` workflow (incl. the 61-form `snapshots` job,
  run 28733883699) and `schema-mirror` (run 28733883686) also ran GREEN on
  the PR — the extraction is invisible to the existing gates, CI-verified.

## FINDINGS (surprises; none blocks the locked contract's mechanics)

- **F1 (chapter-blocking for J2/J3/J4 export-wallet steps — P1 MUST
  resolve):** the export-wallet `--descriptor` arm is UNREACHABLE in the
  production GUI. `render_with_dispatch` materializes ABSENT dropdowns as
  `Dropdown(opts[0])` and WRITES BACK (`widget.rs:221-229`); export-wallet
  `--template` has no schema default → `is_at_default` false → `--template
  bip44` EMITS on a virgin form, and `conditional::export_wallet`
  (`conditional.rs:599-601`) then Disables `--descriptor` (mutex) — egui
  ignores typed input to disabled widgets, and TEMPLATES has no ""/unset
  option to clear the template. Every J2/J3/J4
  canonicalise/BSMS/refusal/core-export step rides that arm
  (`export-wallet --descriptor "$(cat …)"`). Recon C's "all journeys
  GUI-expressible" holds at the FORM level but fails at the FLOW level
  here. Resolution options for P1 (needs its own mini-gate; it is a `src/`
  change beyond the extraction): (a) an "" unset option in the template
  dropdown (display "(none)" — the `display_or` machinery already exists
  for ARCHETYPES), (b) conditional refinement, (c) descope those steps
  (would gut J2/J3/J4 — surely a USER decision, not an implementer one).
- **F2:** `conditional::bundle` Disables `--template` for `--descriptor`
  but NOT for `--descriptor-file` (`conditional.rs:201-226`) — the CLI
  rejects the pair (exit 2, "mutually exclusive"). With the fresh-app demo
  seed (`--template=bip84`), the J2/J3 engrave shape `bundle
  --descriptor-file X` refuses. The spike turned this into the refusal
  pilot; the real engrave steps must either ride `--descriptor` (text —
  works, proven by the chaining leg) or P1 adds the missing
  `has_descriptor_file` mutex arm GUI-side. Small, gate-worthy fix.
- **F3 (fidelity, not correctness):** dropdown-default materialization
  makes virgin forms EMIT flags the Examples CLI lines never pass —
  observed `--language english` on bundle/export-wallet argv echoes (and
  `--format`'s opts[0] generally). Outputs are unaffected for the journeys,
  but tutorial argv echoes will not match Examples.pdf token-for-token;
  chapter prose (or the P1 divergence pass, §2) should note it.
- **F4 (mechanics, ratified into the discipline):** AccessKit option clicks
  never close egui popups (no pointer → `clicked_elsewhere()` can't fire);
  only the NEXT popup's opening closes the previous one, so the LAST popup
  of a drive lingers into the shot (caught visually in the first modal
  render). Ratified: Escape after every popup selection (egui closes any
  popup on Escape, `popup.rs:453`). Corollary: kittest `run()` needed
  `with_max_steps(64)` for the smooth-scroll animation (~8 frames; default
  max is 4).
- **F5 (cite slips, folded into the pilots):** spec S2(i) says
  "export-wallet with `--descriptor-file`" — no such flag on export-wallet
  (recon-C's `:851` is verify-bundle; bundle is `:294`). Mechanics
  unaffected (pilot reshaped to template+slot mode).
- **F6 (expected, worth stating):** the spike's shots render `Pinned:
  mnemonic 0.74.0` (the schema CONSTANT) while the spawned CLI is 0.75.0 —
  the exact label-honesty gap I1 described, live. Harmless in the throwaway
  pilots (the gate is what makes it visible); the REAL corpus must be
  captured only after P1's pin bump + `pinned_version` string catch-up, and
  the version gate makes premature capture impossible.
- **F7 (pane-layout note for the manifest):** argv echoes that embed a full
  descriptor (the `--descriptor` text path) wrap to many lines and grow the
  bottom panel substantially (the label wraps at window width). If F1/F2
  resolve toward file-path fields with short display names (`policy.desc`),
  the echoes stay one-line — another reason to fix F1/F2 rather than route
  everything through inline descriptors.

## Ratifications for the implementation plan (P1 input)

1. **Window size: 920×720 logical / 1840×1440 physical @ ppp 2.0** (single
   global size).
2. **Scroll mechanism: injected `PointerMoved` + `MouseWheel{unit: Point}`
   + `run()`**, offsets recorded as wheel deltas in the manifest;
   `with_max_steps(64)`; seam (iii) stays unused fallback.
3. **Drive discipline:** the 5-rule lookup discipline above (unique-label /
   row-anchor / block-bound / window-scope / Escape-after-popup).
4. **Click semantics:** every Run (and modal-Run) click = queue → assert
   None → ONE `step()` → SAME-FRAME assert → `run()` → snapshot (m1).
5. **Version gate:** probe every manifest-spawned CLI pre-render against
   the schema `pinned_version` constants / pinned-upstream.toml; hard-fail.
   (Spike prototype is the reference implementation.)
6. **Corpus budget:** ~16.4 MiB/51 shots accepted with §5.3 trims in
   reserve; per-shot ~330 KiB at the ratified size.
7. **P1 scope addition (from F1/F2):** two small GUI-side fixes (template
   unset option or conditional refinement; descriptor-file mutex arm) —
   each is a `src/` change with clap-surface = UNCHANGED (no schema-mirror
   impact) but pixels/argv = changed for 2 of the 61 gallery forms at most
   (verify against the 61-form corpus when fixing). These need the normal
   R0/plan treatment; without F1 the J2/J3/J4 export-wallet chapters cannot
   be captured.

## Evidence index

- Branch `spike/gui-example-p0` @ `80f7eb8` (3 commits); draft PR
  https://github.com/bg002h/mnemonic-gui/pull/29 (DO NOT MERGE as-is; P1
  reworks; workflow file is delete-on-fold).
- Committed baselines (sha256, llvmpipe-GL):
  - `935275af…06ec4` s1-freshapp-1280x900.png (2560×1800)
  - `d10e4f25…53ac` s1-freshapp-920x720.png (1840×1440)
  - `3a06c447…2d6c` s2-bundle-refusal-920x720.png
  - `7c085ee3…5f7a` s2-combo-popup-920x720.png
  - `c842a55c…ecf2` s2-exportwallet-run-920x720.png
  - `6980dcd2…8b3f` s2-j1-modal-920x720.png
  - `2c78cc05…afd82` s2-j1-run-920x720.png
  - `e7068651…1155` s3-restore-scrolled-920x720.png
  - `b9847397…ae5e` s3-restore-top-920x720.png
  - `a3146e5a…c05c` s4-exportwallet-run-1280x900.png
  - `c857e122…072d` s4-restore-top-1280x900.png
- Cross-process aggregate `.new.png` sha256 (2 runs, identical):
  `033bc3805e5e6807663bb1f97be091565fea06c71730d72ec41e66120d199260`.
- Key log lines: `SPIKE-GATE-BITE` (wrong-tier probe fired),
  `SPIKE-GATE-OK: mnemonic 0.75.0`, `SPIKE-S1` (byte-identity),
  `SPIKE-S2i/S2ii/S2iii`, `SPIKE-S5-CHAIN` (24 chunks),
  `SPIKE-S3` (scroll 403.75→251.25 pt, reproducible), `SPIKE-S4`,
  `SPIKE-SHOT-TOTAL` (3.54 MiB / 11 shots; 51-shot projection 16.42 MiB).
