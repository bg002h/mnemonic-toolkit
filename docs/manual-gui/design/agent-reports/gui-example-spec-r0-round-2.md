# R0 review — SPEC_gui_example_tutorial.md — round 2 (convergence)

- **Reviewer:** opus-tier architect (adversarial R0, spec gate — round-2 scoped convergence).
- **Date:** 2026-07-05.
- **Scope (per convergence discipline):** fold fidelity + fold-introduced drift ONLY. Round-1
  verified the design and all load-bearing recon facts (sync-runner, extraction map, hygiene
  taxonomies, pin-bump delta); none of that is re-litigated here.
- **Artifacts:** `docs/manual-gui/design/SPEC_gui_example_tutorial.md` (post-fold revision,
  status "round 1 folded 2026-07-05"), against the prescriptions in
  `agent-reports/gui-example-spec-r0-round-1.md` (RED 0C/3I+4M).
- **Ground-truth spot-checks (fold-added cites only, at the spec's declared SHAs):**
  mnemonic-gui verified live at exactly `master@0d4429d` — `main.rs:517-518`
  (`Pinned:` + `ui.monospace(sch.pinned_version)` — a schema constant, as the I1 rationale
  states), `main.rs:300-316` (demo seed: `mnemonic:bundle` pre-fill `--network=mainnet` /
  `--template=bip84` / `--account=0` + one empty `SlotSubkey::Xpub` row — byte-matches the
  §6.3/S5 description), `main.rs:~1231-1234` (`path_detect::detect(bin)` real-`$PATH` probe in
  `spawn_and_capture`), `schema/mnemonic.rs:4620` (`pinned_version: "mnemonic 0.74.0"` — the
  P1 catch-up edit site), `runner.rs:197-199` (`MNEMONIC_FORCE_TTY=1`). Toolkit:
  `.examples-build/gen.sh:22` hard assert confirmed; `manual-gui.yml` tag trigger
  `manual-gui-v*` confirmed with latest tag `manual-gui-v1.1.0` (so P4's `manual-gui-v1.2.0`
  is the correct next); path filter confirmed to be `docs/manual-gui/**` + the workflow file
  only (so the M4c note is accurate); `lint.sh` confirmed 9 phases (`1/9`…`9/9`); both
  CHANGELOG sites exist on disk (`mnemonic-gui/CHANGELOG.md`,
  `docs/manual-gui/CHANGELOG.md`); all four recon reports + the round-1 review present in
  `agent-reports/`.

## Verdict: **GREEN — 0 Critical / 0 Important (+3 Minor, non-blocking) — cleared for the P0 spike.**

Every fold matches its round-1 prescription in substance and location; no fold-introduced
drift rises above Minor; no new Critical/Important findings.

---

## Fold-fidelity verification

### I1 → `pinned-tier-version-gate` — **FAITHFUL, genuinely hard, correctly positioned**

- **Hard/fail-closed:** §3.1(b) "hard-fails on any mismatch"; §6 item 4 "hard-fails on tier
  mismatch"; §8 lists it as a named gate. No soft/warn path exists anywhere in the spec. ✓
- **Pre-render positioning:** "Before ANY step renders" (§3.1b) / "before any render"
  (§6.4, §8) — the probe precedes the first snapshot AND the first spawn, so no corpus byte
  can be produced from a wrong tier. ✓
- **Coverage = ALL manifest-spawned CLIs, including `shots: 0` runs:** the gate probes
  "every CLI the manifest will spawn", and the M1 fold makes the `shots: 0` transcript runs
  ORDINARY manifest steps through the same Run path (§3.1b bullet, §5.3) — so they are inside
  the gate's derivation set by construction, not by special-case. §3.1(d)/§6.4 name
  `md`/`ms`/`mk` conditionally, matching the pinned-upstream.toml derivation. Coherent. ✓
- **Local-regen coverage:** §8 "runs in local regen AND CI" — closes the exact
  `UPDATE_SNAPSHOTS` wrong-tier scenario the finding was about (the gate lives inside the
  test, so regen mode cannot bypass it). ✓
- **Label-honesty rationale:** correctly re-states the verified mechanism (`main.rs:518`
  renders a schema CONSTANT; `spawn_and_capture` resolves bare argv[0] against the real
  `$PATH` via `path_detect::detect`, `main.rs:~1232`) — both re-confirmed live at `0d4429d`. ✓
- **All four claimed sites present:** §3.1(b), §6 item 4, §8, §12.2. ✓ Pattern provenance
  (`gen.sh:22`) cited and live-verified.

### I2 → `SAME-FRAME-COMPLETION` — **FAITHFUL: per-step, named, USER-escalation unambiguous**

- **Per-step + named:** §3.1(b) dedicated bullet; assertion runs per step, immediately after
  the Run-click frame (modal-Run frame for secret steps), BEFORE the `-run.png` snapshot;
  covers both `last_run.is_some()` and the `last_run_error` detect-fail arm. ✓
- **Message text:** names the invariant, cites "populated-pane contract, SPEC §6.5", and
  carries the downgrade language verbatim ("any async-runner change is a USER-decision
  downgrade"). ✓
- **§6.5 cite choice reads cleanly:** the pinned-invariant sentence was folded into §6 list
  item 5 ("Deterministic commands only" — the item that owns runner-timing determinism), so
  "SPEC §6.5" resolves exactly; §6.5 cross-cites §3.1b back, and the pointer-comment
  requirement at `spawn_and_capture` is stated in BOTH §6.5 and the P1 checklist (§9 P1). ✓
- **USER-escalation unambiguous at three sites:** §3.1(b) ("weakening the populated-pane
  contract is a USER decision, never an implementation choice", tied to the §4 STOP menu),
  §4 ("None of these may be adopted silently"), §6.5 (enumerates async runner / spinner /
  seeded `last_run` as the reserved downgrades). No wiggle room. ✓
- One timing-semantics sharpening deferred to the plan → **m1** below (not fold drift: the
  round-1 prescription's own wording "before any further `harness.run()`" carries the same
  ambiguity the fold inherited).

### I3 → tag + CHANGELOG ship mechanics — **FAITHFUL, both sites in the right phases**

- **P2:** `mnemonic-gui/CHANGELOG.md` entry for the v0.56.0 leg, incl. the conditional
  scroll-seam test-only-surface doc (S3 fallback iii) — lands BEFORE the `mnemonic-gui-v0.56.0`
  tag, i.e. in the shipping PR. Correct placement. ✓
- **P3:** starts the `docs/manual-gui/CHANGELOG.md` entry. **P4:** finalizes it AND names the
  attach-shipping tag `manual-gui-v1.2.0` (precedent `manual-gui-v1.1.0` — live-verified as
  the latest `manual-gui-v*` tag) — OR an explicitly stated deferral in the shipping PR,
  "never an omission". Exactly the prescription's either/or, with the decision forced into a
  visible artifact. ✓
- **§3.2(c)** explains the dormant-attach mechanics (tag-triggered workflow) and forward-cites
  §9 P4. ✓ Corroborating observation: `manual-gui.yml`'s header comment confirms tag pushes
  bypass the `paths` filter, so the P4 attach fires on the tag regardless of the M4c
  path-filter decision — that decision only affects branch/PR pushes of PDF-only touches,
  which is precisely how the spec frames it ("documentation note, not a gate gap").

### M1 → `shots: 0` production path — **FAITHFUL + coherent with I1 and both censuses**

§3.1(b) bullet + §5.3 + §8 all state the same thing: `shots: 0` steps are ordinary manifest
steps through the GUI-driven Run path (driven form, real click(s), real pinned-CLI execution,
RunResult byte-gated), the transcript census is manifest-derived ("run count ≥ shot-bearing
step count"), and phase 10/11 expected counts come from `manifest-stems.txt` with "no
hardcoded 51". The three-way coherence check (M1 path ↔ I1 gate coverage ↔ run census)
closes: manifest = single derivation source for all three. ✓ (One labeling nit → **m2**.)

### M2/M3 → ComboBox popup + demo seed — **FAITHFUL and mutually consistent**

- Popup drive is an EXPLICIT S2 exit criterion ("popup open + option click explicitly, not
  implicitly") and shared with S5. ✓
- S5's SlotEditor drive starts from the fresh-app demo seed and must flip the seeded Xpub
  row's subkey to `phrase` (the J1 path) — and §6 item 3 pins the identical demo seed in the
  determinism contract with the consequence spelled out ("any future change to the demo seed
  moves tutorial pixels … corpus-regenerating change"). Spike exits and determinism contract
  describe the same baseline, same cite (`main.rs:300-316`), byte-matching the live source
  (mainnet / bip84 / account=0 / one empty Xpub slot row). Chapter-0 orientation shot is tied
  to the same baseline in both places. No divergence. ✓

### M4 → delta record + cite nits + path filter — **FAITHFUL**

- (a) §3.1(e) records the verified ZERO-flag v0.74.0→v0.75.0 delta with the exact two-edit
  catch-up (pin line + `schema/mnemonic.rs:4620` string) and the corpora-not-invalidated
  argument — matching round-1 V5's verified content without overstating it. ✓
- (b) Cite-nit record in the header preserves the recon reports verbatim while pinning the
  correct values (`gen.sh:22`, `main.rs:~1110`). ✓
- (c) §3.2(c) documentation note + §9 P3 decision point ("adds the path to the filter or
  records this note") — live-verified that the filter indeed excludes `docs/gui_example.pdf`
  today. ✓

### Status/provenance bookkeeping — **ACCURATE**

Status line correctly states RED 0C/3I+4M folded 2026-07-05, awaiting round-2 (this review);
the round-1 report is listed in the R0-reviews provenance block with the fold claim; all four
recon reports + round-1 exist in `agent-reports/`; source-SHA block unchanged and correct —
all fold-added citations are GUI-repo lines under the declared `0d4429d` (live-verified at
exactly that SHA), so no SHA-block update was needed. The dispatch-failure note is preserved. ✓

---

## Findings

### Critical — none.

### Important — none.

### Minor (non-blocking; fold at plan time)

**m1. Pin the SAME-FRAME tripwire's click frame to single-`step()` semantics in the
implementation plan.** §3.1(b) places the assertion after the Run-click frame "and its single
`harness.run()` settle". `harness.run()` steps to quiescence — under a hypothetical future
async runner that requests repaints while polling, one `run()` call could step PAST the click
frame until the result lands, and the tripwire would pass without firing (the corpus would
still be byte-correct, but the invariant would be silently stale). This ambiguity is inherited
from round-1's own prescription wording ("before any further `harness.run()`"), so it is NOT
fold drift and does NOT block the gate — but the plan-doc should pin the mechanics: deliver
the Run click (and the modal-Run click) via exactly ONE `harness.step()` each, assert
`last_run.is_some()` before any further stepping, THEN `harness.run()` to settle for the
snapshot. That gives the tripwire full teeth against fast-settling async redesigns; the
`spawn_and_capture` pointer comment and the byte/pixel/census gates remain the backstops
either way.

**m2. Label the 25 as "shot-bearing steps" where the number is used as a total.** The M1 fold
makes explicit that the manifest contains MORE steps than 25 (J2 devices-1/2 converts, the J4
NUMS pair, the J3/J4 `--json` chain runs — all `shots: 0`), per "run count ≥ shot-bearing
step count". §9 P2's "full manifest (25 steps / 51 shots)" and §12.2's "All 25 steps" read as
if 25 were the manifest total. No gate risk (all censuses are manifest-derived; phases 10-12
hardcode nothing), but a one-word change — "25 shot-bearing steps" — prevents a future
census-reader misreading. Author's discretion; can fold silently into the plan-doc instead.

**m3 (nano). Citation-style inconsistency:** I1 sites cite "§6 item 4" while the I2 assertion
message cites "SPEC §6.5"; both resolve to §6's numbered list. Fine as-is; optional
standardization whenever the file is next touched.

---

## Gate instruction

**GREEN at 0C/0I — the R0 spec gate is passed; cleared for the P0 spike (§4).** The three
Minors are non-blocking: m1 and m2 fold naturally into the implementation plan-doc (which gets
its own R0 per §9/house rule); m3 is cosmetic. Update the spec's status line to
R0-GREEN-round-2 and proceed to P0 in a mnemonic-gui worktree; the spike report and the plan
then re-enter the loop as already scheduled.
