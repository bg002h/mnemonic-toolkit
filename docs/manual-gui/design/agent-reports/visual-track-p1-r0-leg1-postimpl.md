# P1 R0 + LEG-1 POST-IMPL WHOLE-DIFF REVIEW — visual-screenshot track, mnemonic-gui leg

*(Opus-tier architect review, persisted 2026-07-01. COMBINED per-phase P1 R0 + Leg-1 post-implementation whole-diff review — combination deliberate and disclosed: Leg 1 = P0 (spike, R0'd GREEN at `visual-track-p0-r0-review.md`, artifacts deleted in P1) + P1 (the only other phase), so one adversarial review over the whole branch diff `a522a69..79a9074` covers both obligations. Inputs: plan `IMPLEMENTATION_PLAN_gui_visual_screenshot_track.md` P1 (lines 17–26); spec `SPEC_gui_visual_screenshot_track.md` §4/§5; the P0-R0 ratifications + advisories A1–A5; live branch `feat/gui-form-snapshots` @ `79a9074` (history `2655dc3` → `0c618d1` → `79a9074`, off master `a522a69` = current origin/master tip); DRAFT PR #27; live CI run 28559684310 / job 84674604424; kittest 0.31.1 registry source; independent local re-execution on llvmpipe-GL. Every load-bearing implementer claim was re-derived from ground truth, not trusted.)*

## VERDICT: **GREEN — 0 Critical / 0 Important** (3 Minor notes, none blocking). **Ready/merge is CLEARED** — subject only to the ship-sequence items in §5 (which are correctly NOT on the branch).

---

## 1. Ratified-design conformance check (`tests/gui_form_snapshots.rs`, 229 lines)

| Ratified element | Verdict | Evidence |
|---|---|---|
| All-61 enumeration over `render_fixture` via the SHARED extended path | **CONFORMS** | `render_emit::all_forms()` (the PR-#24 enumerator, `src/form/render_emit.rs:57`, src untouched — whole-leg `git diff a522a69..79a9074 -- src/` = 0 lines) + `snapshot_harness` composing `ui_harness::render_whole_form` + `render_positionals` + `render_action_bar` — the same three-fn composition `render_extended_form_harness` uses |
| `fit_contents()` @ `pixels_per_point = 2.0` (P0-R0 §1 RATIFIED) | **CONFORMS** | `PIXELS_PER_POINT: f32 = 2.0` const + `harness.fit_contents()` in `snapshot_harness` |
| Default threshold 0.6, NO per-form overrides | **CONFORMS** | `SnapshotOptions::new().output_path(SNAPSHOT_DIR)` — the ONLY builder calls; kittest 0.31.1 `snapshot.rs:24` confirms `threshold: 0.6` default; grep of the file finds no `.threshold(` anywhere |
| A1 adapter guard exact shape | **CONFORMS** | `device_type == Cpu` asserted unconditionally after the env gate; `backend` asserted iff `WGPU_BACKEND` set (via `expected_backend` single-backend spelling map, unknown spellings → loud skip of the backend arm only); `name`/`driver` never matched. **Source-verified parity:** the guard's `create_render_state(default_wgpu_setup())` is byte-for-byte the harness's own default renderer path — kittest `builder.rs:22` `LazyRenderer::default()` → `renderer.rs:39` `WgpuTestRenderer::new` → `wgpu.rs:84` `create_render_state(default_wgpu_setup())` — so the probed adapter IS the selection logic the snapshots use |
| Run-node no-clipping assertion per form | **CONFORMS** | `query_all_by_label("Run")` asserted inside the loop, every form, before the snapshot call |
| Census 61 | **CONFORMS** (three layers) | in-test `all.len() == 61` + `rendered == 61`; CI census 61 × `.new.png` (§4 below) |
| Early-return-skip + eprintln | **CONFORMS** | verified LIVE locally: `SNAPSHOTS-SKIP: GUI_SNAPSHOTS != 1 …` marker printed, `2 passed` (skip = pass, census is the CI tripwire, as designed) |
| I3 hygiene test ALWAYS-RUN | **CONFORMS** | `secret_flags_never_carry_a_default_value` has NO env gate; sweeps `CliTab::ALL` (all 4 tabs) × `schema_for(tab).subcommands`; asserts `subs == 61` (coverage census) + `secret_flags > 0` (non-vacuity) + zero violations. Ran green in the CI snapshots job log AND in the plain local suite. **Scope-completeness verified:** `PositionalArgSchema` (`src/schema/mod.rs:58`) has NO `default_value` field — positionals cannot carry schema defaults at all, so the flag-only scope is complete, not a gap |

**The implementer's disclosed deviation — `try_snapshot_options` + end-of-loop aggregate assert instead of panic-per-form: APPROVED, semantics verified equivalent-or-stronger.**
- **Missing baseline = hard failure:** kittest `snapshot.rs:246–257` — no baseline → `Err(SnapshotError::OpenSnapshot)` (when `UPDATE_SNAPSHOTS` unset), which this suite pushes to `failures` unconditionally. Unlike the P0 spike (which explicitly tolerated `OpenSnapshot` pre-corpus), NOTHING is tolerated here. Size mismatch and threshold diff likewise `Err` → `failures`.
- **No silent-skip path:** the loop over `&all` contains no `continue`; every iteration increments `rendered` and either panics (Run-node assert) or records the snapshot result; `rendered == 61` + `failures.is_empty()` both asserted. Additionally kittest writes `.new.png` BEFORE comparing (`snapshot.rs:239–244`), so even an erroring form leaves its census evidence — the census cannot be dodged by any in-loop path.
- **Net effect vs panic-per-form:** identical red/green outcome, PLUS a partial drift now renders + diffs the WHOLE corpus so the failure-path artifact carries every `.diff.png` (spec-§5 ladder forensics) — strictly better.

## 2. Provenance-chain check (the committed corpus)

- **All 61 blobs at `79a9074:tests/snapshots/forms/` are the EXACT git objects from `0c618d1:tests/snapshots/spike/`** — `git show -M --find-renames=100%` reports 61/61 as pure renames (`{spike => forms}`, Bin, 0 insertions/deletions); a full `git ls-tree` blob-SHA join across both trees matches 61/61 (blob identity ⇒ byte identity, stronger than spot-sha256; spot-verified anyway on `mnemonic-inspect`/`mnemonic-verify-bundle`/`ms-vectors`/`md-encode`: e.g. `06be669d…`, `f77fd5aa…` identical both sides). The spike corpus itself was byte-verified to the run-1 CI artifact by the P0-R0 review — the chain runner-artifact → `0c618d1` → `79a9074` is unbroken.
- **`tallest-fixed-800x600.png` did NOT migrate** — deleted in `79a9074` (`Bin 155429 -> 0`); tip tree has exactly 61 files under `tests/snapshots/`, all in `forms/`, per-tab counts 32/10/10/9 (mnemonic/md/ms/mk) matching the test-header arithmetic.
- **The byte-identity tripwire claim INDEPENDENTLY REPRODUCED (not trusted):** I rendered the OLD path (spike test @ `0c618d1`, private fn copies) and the NEW path (promoted shared fns @ `79a9074`) on the same machine/env (llvmpipe-GL, Mesa 26.1.2, `LIBGL_ALWAYS_SOFTWARE=1`): **61/61 `.new.png` byte-identical** across the refactor — the A3 condition ("rename seeding legitimate IFF the promotion renders byte-identically") holds on my hardware too, not just the implementer's.
- **Refactor verbatim-ness confirmed mechanically:** the four promoted fns extracted from `a522a69:tests/gui_render_faithfulness.rs` vs `79a9074:tests/ui_harness/mod.rs` diff clean modulo exactly (i) `fn` → `pub fn` and (ii) `ui_harness::render_whole_form` → `render_whole_form` (module-local now) — nothing else.

## 3. Promotion refactor left the faithfulness gate untouched-green

- `git diff a522a69..79a9074 -- tests/gui_render_faithfulness.rs`: doc-comment updates, import trims, the four fn deletions, and TWO call-site re-pointings (`ui_harness::render_one_positional`, `ui_harness::render_extended_form_harness`). **Zero assertion lines changed.**
- Passes locally at tip (`1 passed`, 5.48 s — the all-61 sweep). Full suite at tip: **72 binaries, 624 passed / 0 failed / 4 ignored** — exactly master's 622 + the 2 new tests, spike test deleted; every PR-#24 harness consumer green. `#![allow(dead_code)]` at `ui_harness/mod.rs:1` (pre-existing PR-#24 pattern) covers per-binary unused pub fns; CI clippy (`--all-targets -D warnings`) green.

## 4. The `snapshots` job — recipe conformance + the four would-it-catch answers

Job verbatim-conforms to the P0-R0 §2 ratified recipe: NO path filter (and `build.yml` `on:` = PR + master push + `mnemonic-gui-v*` tag push, so the job fires on all three — the tag-run provenance anchor works); job-level `env: GUI_SNAPSHOTS=1, WGPU_BACKEND=vulkan`; `apt-get install mesa-vulkan-drivers`; `cargo test --test gui_form_snapshots -- --nocapture`; census step `test "${count}" -eq 61` — **fail-closed** (find on a missing dir → count 0 → fail; count 60 or 62 → fail; step skipped only when the suite step already failed → job red anyway); `upload-diffs` `if: failure()` + `if-no-files-found: ignore`; `Swatinem/rust-cache` (which caches `~/.cargo` + `target/`, never `tests/snapshots/` — no stale `.new.png` can leak into the census). Live: run **28559684310** / job **84674604424** on `79a9074` = success; log shows `SNAPSHOTS-ADAPTER: … device_type: Cpu, … backend: Vulkan` (lavapipe), `2 passed`, `census: 61 .new.png rendered`. All 12 other check contexts green on the tip (release skipped, tag-only).

**Would the job CATCH:**
- **(a) a form rendering differently** — YES. dify at default 0.6 vs the committed baseline → `Err(SnapshotError::Diff)` → `failures` → suite step red; `.diff.png` written + uploaded. Threshold arms fleet-proven (P0: 6 instances + full backend swap; this review: 61/61 pass with 18/61 byte-drift on a THIRD Mesa version).
- **(b) a missing corpus file** — YES. `Err(OpenSnapshot)` is a hard failure here (nothing tolerated post-corpus; kittest `snapshot.rs:253`).
- **(c) a silently-skipped suite** — YES. The skip path writes zero `.new.png` → census 0 ≠ 61 → `census-61` step fails even though the suite step "passed". (Env-typo class — the exact I1-companion hole — covered.)
- **(d) a non-CPU adapter** — YES. `device_type == Cpu` asserted unconditionally under `GUI_SNAPSHOTS=1`, and the probe provably shares the harness's selection path (§1). Backend arm additionally pins Vulkan in CI (`WGPU_BACKEND` set at job level).

**GL-arm live check (this review):** local `WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1` run passed the guard with `driver: ""` `backend: Gl` — confirming the never-match-name/driver ruling was load-bearing (the GL arm's driver field is empty, exactly as P0 documented) and the sanctioned Plan-C regen path stays green.

## 5. Whole-leg hygiene + what's owed at ship

**Hygiene — clean:**
- Secret pixels: spot-opened `mnemonic-inspect.png` (`--ms1` secret field EMPTY), `ms-derive.png` (`--hex`/`--phrase`/`--passphrase` empty; `--account 0` is a non-secret schema default, permitted by design), plus the 3 tallest — `mnemonic-verify-bundle` (850×1253), `mnemonic-restore` (850×1169), `mnemonic-convert` (1008×957) — all complete to the last flag row + `Run` visible at the bottom (the I4 bar), dark theme. No secret literal in any test source (blank `render_fixture` only — the spec-§4 layer-3 injection invariant holds).
- Corpus: 61 files, **2,221,962 B** (2.12 MiB) — byte-equal to P0's measured figure; real PNG blobs (magic verified), plain storage, no `.gitattributes`/LFS anywhere in the tree.
- `.gitignore`: exactly the three kittest artifact lines (`*.new.png`/`*.diff.png`/`*.old.png` under `tests/snapshots/**`) — none swallow the corpus `.png`s.
- Spike removal: `tests/spike_form_snapshots.rs` (296 l) + `.github/workflows/spike-snapshots.yml` (147 l) + `tests/snapshots/spike/` all deleted; the ONLY spike-named survivor is `tests/spike_widget_drivers.rs` — **master-shipped by PR #24 (`afcd28e`), pre-existing, out of scope, unchanged on this branch.**
- Cargo.lock: whole-leg delta = exactly the 12 dev-graph packages (A4: colored, crossbeam-deque/epoch, dify, either, getopts, gpu-allocator, presser, range-alloc, rayon, rayon-core, unicode-width dup); 0 removals, 0 existing-version moves; P1 commit itself touches no lock/manifest. No fmt churn — every hunk targeted. All 3 commits carry the Co-Authored-By + Claude-Session trailers.
- Branch base `a522a69` = current origin/master tip — no rebase owed.

**Owed at ship (correctly NOT on the branch — verified absent):**
1. **PR #27 title/body refresh** — still the P0 text ("P0 spike in progress" / "DO NOT MERGE") — rewrite for the P1 reality when flipping ready (Minor m1 below).
2. Ready + merge (all 12 checks green at tip; MERGEABLE).
3. **Branch-protection rule** requiring context **`snapshots`** (= the job's declared `name:`; live-verified master is currently UNPROTECTED — API 404) — one-time admin action at merge, snapshots-only scope per the user's 2026-07-01 call (`gui-branch-protection-scope` filed at `mnemonic-gui/FOLLOWUPS.md:998`).
4. **Release commit 0.53.0 → 0.54.0**: `Cargo.toml` + `Cargo.lock` + `CHANGELOG.md` + README self-pin (line 47 `--tag mnemonic-gui-v0.53.0`) — **all four sites verified still at 0.53.0 / no 0.54.0 CHANGELOG entry** — they ride the release commit as planned; m10: once the required-check rule is live, the release commit goes via PR flow or explicit admin bypass, decided at ship.
5. **Tag `mnemonic-gui-v0.54.0`** → **verify the tag-push `snapshots` run is GREEN** (the Leg-2 provenance anchor; P2 step 0 re-checks it mechanically).
6. GUI-side FOLLOWUPS companion entry — **P2's deliverable by plan** (line 38: "none exists yet — the plan creates it"); confirmed absent, NOT owed here. `manual-gui-visual-screenshot-track` remains open at `docs/manual-gui/FOLLOWUPS.md:23`, resolved in P2's shipping commit. The merged PR-#26 help fix rides the tag automatically (already on master at `2914496`).

## 6. Findings

**Critical: none. Important: none.**

**Minor (none blocking; all ship-step or note-only):**
- **m1 — PR #27 title/body are stale P0-spike text ("DO NOT MERGE").** Must be rewritten before flipping ready — otherwise the merge commit inherits a title claiming the PR is an unmergeable spike. Pure ship-step editing; the plan's ready+merge step implies it but doesn't spell it out.
- **m2 — no GUI-side orphan-baseline tripwire.** A stray 62nd committed PNG under `tests/snapshots/forms/` would not fail the GUI job (the census counts rendered `.new.png`, not committed files). Leg-2's `verify-figures-gui` census runs BOTH directions and catches it at the next pin-bump — a lagging but existing net. The implemented census matches the ratified recipe verbatim, so this is a design note for the ladder, not a deviation. No action owed in P1.
- **m3 — the in-test `rendered == 61` assert is redundant** with `all.len() == 61` (the loop cannot skip). Harmless belt-and-braces; keep.

## 7. Independent verification log

| Claim | Ground truth checked | Result |
|---|---|---|
| Suite = ratified design | full read of `79a9074:tests/gui_form_snapshots.rs` vs plan P1 + P0-R0 §1/§2/A1–A5 | CONFORMS (§1) |
| Guard probes the harness's real selection path | kittest 0.31.1 `builder.rs:22` / `renderer.rs:39` / `wgpu.rs:11,84` | CONFIRMED |
| Missing-baseline = failure; `.new.png` always written; 0.6 default | kittest 0.31.1 `snapshot.rs:24,239–257` | CONFIRMED |
| Corpus = spike corpus, 61/61 | rename-detect + blob-SHA join `0c618d1` ↔ `79a9074` | CONFIRMED (100% renames) |
| Refactor byte-identical render | old-path (`0c618d1` spike test) vs new-path (`79a9074` suite) rendered locally, same env | **61/61 byte-identical** |
| "18/61 cross-env byte-drift, threshold-clean" (P1 commit msg) | local llvmpipe-GL (Mesa 26.1.2) vs the corpus | REPRODUCED: 61/61 pass @0.6, exactly 18 byte-differ |
| "624/0/4" full suite | independent `cargo test --jobs 2` at tip | REPRODUCED: 72 binaries, 624/0/4 |
| Faithfulness gate green, assertions untouched | whole-leg diff + local run | CONFIRMED (0 assertion changes; 1 passed) |
| Live CI green w/ adapter line + census | run 28559684310, job 84674604424 logs + PR-27 check rollup (12 contexts) | CONFIRMED (`device_type: Cpu`, `backend: Vulkan`, `census: 61`) |
| Version sites not yet bumped | `79a9074:Cargo.toml`/`Cargo.lock`/`CHANGELOG.md`/`README.md:47` | CONFIRMED — all 0.53.0, no new entry |
| master unprotected (ship-step pending) | `gh api …/branches/master/protection` | CONFIRMED — 404 |
| Trailers on all 3 commits | `git log --format=%(trailers)` | CONFIRMED |

## 8. Gate disposition

**P1 per-phase R0: GREEN. Leg-1 post-impl whole-diff: GREEN. 0 Critical / 0 Important.** The branch implements the ratified design without deviation (the one disclosed choice — aggregate failure collection — is semantics-equivalent and forensically stronger); the corpus provenance chain is object-identity-verified end to end; the permanent gate catches all four drift classes; nothing is owed on the branch that isn't a planned ship-step. **PR #27 may go ready and merge**, then the §5 ship sequence (protection rule → release commit → tag → tag-run verification) proceeds per plan.

*Both repos left clean: mnemonic-gui master checkout untouched, review worktrees (`.worktrees/p1-review`, `.worktrees/spike-old`) removed after evidence collection; toolkit untouched except this report. Evidence: commits `2655dc3`/`0c618d1`/`79a9074`; CI run 28559684310 (job 84674604424); kittest 0.31.1 registry source; local runs on llvmpipe-GL Mesa 26.1.2 (suite 624/0/4; snapshot 61/61 @0.6; old-vs-new 61/61 byte-identical).*
