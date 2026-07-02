# P0-R0 REVIEW — visual-screenshot track, P0 feasibility spike (per-phase R0)

*(Opus-tier architect review, persisted 2026-07-01. Inputs: spike report `visual-track-p0-spike.md`; spec `SPEC_gui_visual_screenshot_track.md` §3/§4/§5; plan `IMPLEMENTATION_PLAN_gui_visual_screenshot_track.md` P0/P1; recon `visual-track-recon-2026-07-01.md`; live evidence on mnemonic-gui branch `feat/gui-form-snapshots` (`2655dc3` spike code + `0c618d1` corpus; DRAFT PR #27), runs 28555238627 + 28555563213 attempts 1–3, the committed corpus `tests/snapshots/spike/` (62 PNGs), and the run artifacts. Every load-bearing claim below was re-derived from ground truth — logs, check-runs API, git objects, artifact bytes, and independent local re-execution — NOT taken from the report.)*

## VERDICT: **GREEN — 0 Critical / 0 Important** (4 Minor report-accuracy notes, 5 non-blocking advisories). **P1 may begin.**

The spike's GO conclusion is CONFIRMED. Every major evidentiary claim in the report survived adversarial re-derivation; the four inaccuracies found are cosmetic (counts/phrasing in the report text itself, none touching the plan-letter, the sizing evidence, or the fleet results). The two formal ratifications requested by the plan's P0 gate (plan line 15) are issued below.

---

## 1. THE SIZING RATIFICATION (formal output #1)

**RATIFIED: `fit_contents()` at `pixels_per_point = 2.0` is the P1 sizing policy for all 61 form snapshots.**

Grounds, all independently verified:

- **The fixed-frame alternative is disproven against the spec-§4 I4 no-clipping bar, not merely disfavored.** kittest's default 800×600 logical viewport = 1600×1200 physical px @2× (confirmed: `tallest-fixed-800x600.png` IHDR = 1600×1200, 155,429 B at `0c618d1`). The tallest form `mnemonic verify-bundle` measures **850×1253 physical** (IHDR-confirmed) — 53 px over the fixed frame. I opened the fixed-arm PNG: the form is visibly cut below `--no-auto-repair` and **the Run button is absent** — the exact silent-subset failure mode I4 was written to prevent (a PNG depicting less than the structural render lists, with every gate still green). The live log line confirms the in-test derivation: `SPIKE-TALLEST: mnemonic-verify-bundle fit_contents=850x1253px vs fixed-800x600=1600x1200px -> fixed frame would clip: true` (job 84662130957).
- **`fit_contents()` meets the bar by construction and by measurement.** The viewport is sized to content, so viewport clipping is impossible; the spike additionally tree-asserted the Run node (the LAST rendered widget) present for all 61 forms (`spike_form_snapshots.rs`, the `query_all_by_label("Run")` assertion) and I visually spot-opened **verify-bundle (850×1253)**, **restore (850×1169, #2 tallest)**, and **inspect (685×321, secret-bearing)** — all complete: every flag row + positionals + Run visible at the bottom, dark theme, secret fields empty/masked.
- **The "per-form measured height" alternative is REJECTED** — it reaches the same bar only by adding a manual measurement pass that `fit_contents()` performs natively, with a hand-maintained height table as a new drift surface. Strictly more machinery, zero added assurance.
- **The unbounded-height risk is real-but-theoretical and LOUD-FAIL, so no cap is needed.** Quantified: verify-bundle's 29 schema rows → 1253 px ≈ 43 physical px/row; wgpu's default `max_texture_dimension_2d` = 8192 px puts the render ceiling at ≈190 rows — **6.5× today's tallest form**. A future form crossing it fails the render (snapshot error → `failures` → test failure), never silently clips; corpus growth is PR-visible (committed PNG diffs) and arbitrated by the threshold gate. The no-clipping bar + the gate cover the concern; an optional one-line note in the P1 test header suffices.
- **Determinism at this sizing is fleet-proven** (section 3): byte-stable same-env 4/4, threshold-stable across 6 distinct runner instances and a full backend swap.

P1 enforcement of the bar stays as planned: the per-form Run-node assertion + spot-opening the tallest PNGs at the P1 gate (plan line 26).

## 2. THE CI-RECIPE RATIFICATION (formal output #2)

**RATIFIED: Plan A** for the permanent `build.yml` `snapshots` job:

```
sudo apt-get update && sudo apt-get install -y mesa-vulkan-drivers
env: GUI_SNAPSHOTS=1, WGPU_BACKEND=vulkan
GUI_SNAPSHOTS=1 cargo test --test gui_form_snapshots -- --nocapture
post-step census: find tests/snapshots/forms -name '*.new.png' | wc -l  == 61
```

- **Proven live, first try, with margin:** 4/4 spike-A job instances rendered all 62 snapshots with zero render errors across two distinct runner VM images (20260628.225.1 + 20260622.220.1); Plan B (pinned Mesa tarball) never needed; Plan C (llvmpipe-GL) ALSO proven on runners — the documented local-regeneration/fallback path (spec §5 M4), passing the 0.6 threshold against the Vulkan corpus on all 3 sampled instances (a strictly harder pass: full backend swap).
- **The apt Mesa version floats with the runner image BY DESIGN, and that is rung-appropriate:** an image-bump drift is precisely what the spec-§5 I2 remediation ladder exists for (inspect `.diff.png` → per-form threshold raise with sign-off → never blanket-raise → pinned-tarball regeneration as rung 4). Plan B remains the escalation, not the default. No pin is added to the recipe.
- **Adapter-assertion guidance (the report's finding-4 advice is SOUND, with one sharpening):** lavapipe live-verifies as self-reporting `name: "llvmpipe (LLVM 20.1.2, 256 bits)"` with `driver: "llvmpipe"`, `backend: Vulkan` — and the GL arm reports the SAME name with an **empty** `driver` field and `backend: Gl` (both strings re-derived from all 8 job logs; exactly 2 distinct adapter lines fleet-wide). Therefore: **key any assertion on `device_type == Cpu` (assert unconditionally whenever `GUI_SNAPSHOTS=1` — it also enforces the "software rasterizer only" corpus-provenance posture for local regeneration) plus `backend` matched against `WGPU_BACKEND` only when that env var is set** (a hard `backend == Vulkan` assert would break the sanctioned Plan-C local-regen path). Never match on `name` or `driver`. This assertion is advisory for P1 (A1 below), not a gate change.
- **Bonus fleet evidence generated by this review** (strengthens, not required): the gen-run's EPYC **9V74** GL render vs attempt-3's EPYC **7763** GL render — **62/62 byte-identical** across different CPU models AND different runner images on the same backend. The AVX/JIT cross-CPU worry did not materialize even at byte level on the observed fleet; the gate remains threshold-based regardless.

## 3. Independent verification log (what was re-derived, with cites)

| Claim (report line) | Ground truth checked | Result |
|---|---|---|
| (i) headless render, 4 job instances (l.21) | runs 28555238627 (jobs 84661147685/84661535610) + 28555563213 a1/a2/a3 (jobs 84662130957/84670312889/84670713885 + C-side), conclusions via API | **CONFIRMED** — all 8 jobs `success` |
| (ii) byte-identity 4/4 (l.23–24) | `BYTE-IDENTITY: OK — 62/62` present in all 4 spike-A logs; workflow step is a REAL gate (`sha256sum`-manifest `diff -u` + 62-line count, fail-on-nonzero) — not an unconditional echo | **CONFIRMED** |
| (iii) 6 instances @0.6, 558/558 (l.26–40) | runner IDs 1000009096/9100/9101/9102/9103/9104 (attempts API) — match the table 1:1; grep across all 6 post-corpus logs: **558 `status=match`, 0 fail**; the 186 `status=missing` confined to the pre-corpus gen run (124 A + 62 C), as designed | **CONFIRMED** — 3× spike-A 124/124 + 3× spike-C 62/62 |
| compare logic is honest | `spike_form_snapshots.rs` @`2655dc3`: only `SnapshotError::OpenSnapshot` with no pre-existing baseline tolerated; every other error → `failures` → assert; `SnapshotOptions::new()` = default 0.6 | **CONFIRMED** — `status=match` means a real threshold pass |
| adapter strings (l.9–16) | deduped across all 8 logs | **CONFIRMED** verbatim (2 distinct lines: Vulkan + Gl) |
| CPU/image columns (l.29–40) | per-log `/proc/cpuinfo` + image-version grep | **CONFIRMED** — 9V74 only on gen-side GL; 7763 elsewhere; two images |
| corpus provenance "byte-exact from artifact" (l.27) | `git show 0c618d1:tests/snapshots/spike/*` vs artifact `spike-pngs-vulkan-28555238627-1` (id 8025391909) | **CONFIRMED** — 62/62 byte-exact; `hashes.txt` self-sha `3a3e26b2…4192611` matches |
| anchor hashes + dims (l.42–46, 52) | sha256 + IHDR on the git objects | **CONFIRMED** — all 4 hashes; verify-bundle 850×1253, restore 850×1169, ms-vectors 168×110/3,740 B, fixed arm 1600×1200 |
| rankings/timings (l.51–58) | `SPIKE-RANKING-ROW`/`SPIKE-HEIGHT-ROW`/`SPIKE-TIME` log lines | **CONFIRMED** — schema top-10 and height top-6 verbatim; CI timings 16.53–22.13 s |
| corpus size (l.61–62) | `git ls-tree -r -l 0c618d1` | **CONFIRMED** — 61 forms = 2,221,962 B; 62 files = 2,377,391 B |
| MSRV I4 (l.66) | **independently re-executed**: `cargo +1.88.0 check --locked --all-targets` in a detached worktree @`0c618d1` | **CONFIRMED** — exit 0; sole warning = pre-existing `dead_code` in `secret_taxonomy_pin` |
| lock delta (l.69) | `git diff master 2655dc3 -- Cargo.lock` + `cargo tree` | **CONFIRMED** — exactly 12 new stanzas, 0 removed, 1 deletion line (unicode-width disambiguation), no existing version moved; **`cargo tree -e normal,build`: none of the 12 present; `-e all`: present** → dev-graph-only PROVEN. `egui_kittest` confirmed under `[dev-dependencies]` |
| all other checks green on BOTH commits (l.84) | check-runs API on `2655dc3` + `0c618d1` | **CONFIRMED green** — but the count is 12 non-spike contexts, not 13 (m1) |
| secret hygiene in pixels (l.53) | opened `mnemonic-inspect.png`: `--ms1` secret field EMPTY; blank fixture; no injected values in the test source | **CONFIRMED** |

## 4. Assessment of the report's findings (1–8)

1. **Cross-env byte drift under 0.6 — CONFIRMED and quantified:** CI-Vulkan vs CI-GL artifacts: 19/62 files byte-differ, 0 threshold failures. Direct empirical support for the spec-§2 split-gate premise (threshold at the render gate, byte-diff only for the leg-2 file COPY).
2. **"Size ≠ identity" — conclusion CORRECT, supporting detail wrong (m2), and the correction STRENGTHENS it:** per-file sizes are NOT all identical across the CI pair — 4 files differ by ±1 B (`mnemonic-seedqr-encode` +1, `mnemonic-silent-payment` −1, `mnemonic-xpub-search-passphrase-of-xpub` −1, `ms-split` +1) and the deltas CANCEL to the identical 2,221,962 B total. Total byte-size is thus an even weaker signal than the report claims. Keep the operative guidance verbatim (only hashes/threshold-diffs mean anything). **No spec/plan text change needed** — neither document uses byte-size as an identity signal anywhere (spec §3's "corpus size" is a budget; §2/§5 already state the threshold/byte-copy posture); an optional one-liner in the P1 test header is sufficient.
3. **libtest-captured skip marker → `--nocapture` on the permanent job — SOUND, ratified as observability-only.** The census (`.new.png` count == 61) remains the SOLE enforcement per spec §5 (the I1 companion) — `--nocapture` changes no semantics, and it additionally makes the adapter line log-visible in the permanent job (useful for ladder forensics). Adopted into the ratified recipe (§2 above). The `SPIKE-SKIP`-invisible-in-schema-mirror behavior is correct libtest capture, not a defect.
4. **lavapipe self-reports "llvmpipe" — CONFIRMED live; advice sound** with the sharpening in §2 (device_type unconditional; backend env-conditional; never name/driver — the GL arm's `driver` field is EMPTY, so `driver` is doubly unusable).
5. **Plan-C GL passes vs the Vulkan corpus — CONFIRMED** (3 instances × 62/62), plus this review's 62/62 cross-CPU byte-identity. The M4 regeneration UX rests on measured ground.
6. **Fleet homogeneity — CONFIRMED** (EPYC 7763 ×7, 9V74 ×1). Noted residual: all six THRESHOLD comparisons ran on 7763; the 9V74 sample is render-side only — but this review's byte-level 9V74↔7763 comparison closes that gap more strongly than a threshold pass would have. Non-issue.
7. **First-run `.new.png` mechanics — CONFIRMED empirically:** the post-corpus PASSING jobs' `cp tests/snapshots/spike/*.new.png` steps succeeded, proving `.new.png` persists on passing comparisons — the exact property P1's census depends on (plan line 22).
8. **Lock delta 12-not-4 — CONFIRMED; agreed** that P1/R0 texts cite 12 (advisory A4).

## 5. Review findings

**Critical: none. Important: none.**

**Minor (report-text accuracy only; fold into the P1 report's citations, no re-spin of the spike report required):**
- **m1** — report l.84: "All 13 other checks GREEN" → actual **12** non-spike check contexts on both commits (11 `success` + `release` `skipped`, tag-only by design).
- **m2** — report l.73: "identical per-file sizes" is false for the CI pair (4 files ±1 B, canceling); see finding-2 assessment. The headline and guidance stand.
- **m3** — report l.57: "across 7 CI samples" → **12** CI `SPIKE-TIME` samples exist (8 spike-A in-job runs + 4 spike-C); the quoted 16.5–22.1 s range is accurate (measured 16.53–22.13 s).
- **m4** — report l.72: "20/62 differ at the BYTE level" across three envs — the verifiable CI pair measures **19/62**; the 20 requires the session-side local-GL leg (not independently verifiable; plausible; non-load-bearing — the gate is threshold-based).

**Advisories (non-blocking, for P1 execution):**
- **A1** — add the adapter guard to the permanent test per §2 guidance (device_type `Cpu` unconditional under `GUI_SNAPSHOTS=1`; backend checked iff `WGPU_BACKEND` set).
- **A2** — `--nocapture` on the `snapshots` job (ratified §2); census remains the tripwire.
- **A3** — corpus seeding: P1 may legitimately seed `tests/snapshots/forms/` by RENAMING the fleet-proven spike corpus (its provenance is now byte-verified to the run-1 artifact) **iff** the promotion refactor renders byte-identically — which doubles as a free behavior-identical-refactor tripwire; **`tallest-fixed-800x600.png` (the 62nd file) MUST NOT migrate** (census is 61). Locally regenerating per the plan posture is equally valid; CI arbitrates either way.
- **A4** — cite **12** lock packages wherever P1 text repeats the delta (not the plan's predictive 4).
- **A5** — no spec/plan edit for size≠identity (see finding-2 assessment); optional test-header note only.

## 6. P1 readiness — nothing in the spike changes P1's shape

- **Promotion refactor still required as planned** (plan line 19): the spike COPIED `render_one_positional`/`render_positionals`/`render_action_bar` and touched no shipped GREEN test file — `2655dc3` changes exactly 5 files (`spike-snapshots.yml`, `.gitignore`, `Cargo.lock`, `Cargo.toml`, `tests/spike_form_snapshots.rs`); `gui_render_faithfulness.rs` untouched.
- **Absorb+remove fits what was actually built** (plan line 23): P1 deletes `tests/spike_form_snapshots.rs` (296 lines), `.github/workflows/spike-snapshots.yml` (147 lines), and `tests/snapshots/spike/` (62 PNGs); the recipe is absorbed into `build.yml`'s `snapshots` job. Nothing spike-named reaches the merge.
- **`.gitignore` P1 item is ALREADY SATISFIED**: the three `tests/snapshots/**/*.{new,diff,old}.png` lines landed in `2655dc3` — P1 keeps them, adds nothing.
- **Census mechanics validated live** (finding-7 assessment); the required-check ship-step (plan line 25) and the red-corpus mitigation (plan line 24) are unaffected; the branch/DRAFT-PR #27 continue as the P1 vehicle.
- **Budgets under estimate:** corpus 2.12 MiB (vs ~3.4 MB), warm job ≈1.5–2 min — no plan pressure.

## 7. Gate disposition

**P0 R0: GREEN (0C/0I).** Sizing = `fit_contents()` @ ppp 2.0 — RATIFIED. CI recipe = Plan A + `--nocapture` + census, adapter guidance per §2 — RATIFIED. STOP condition never approached (Plan A passed first try; Plan C also green). **P1 may begin on settled ground.**

*Both repos left clean (mnemonic-gui on `master@a522a69`, worktree removed; toolkit untouched except this report). Review evidence: job logs 84661147685/84661535610/84662130957/84662329508/84670312889/84670478651/84670713885/84670904483; artifacts 8025391909 (vulkan-run1), 8025431748 (gl-9V74), 8026547118 (gl-7763-a3); commits `2655dc3`/`0c618d1`; MSRV re-run local (exit 0).*
