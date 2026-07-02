# P0 SPIKE REPORT — visual-screenshot track feasibility on REAL GitHub runners

*(Implementer report, persisted 2026-07-02. Plan: `IMPLEMENTATION_PLAN_gui_visual_screenshot_track.md` P0; spec §3. Executed on branch `feat/gui-form-snapshots` of mnemonic-gui — DRAFT PR https://github.com/bg002h/mnemonic-gui/pull/27, commits `2655dc3` (spike code) + `0c618d1` (corpus). NOT merged; master untouched. This report is the P0-R0 input that ratifies the sizing choice.)*

## VERDICT: GO — Plan A works first try

**Plan letter: A** — plain `ubuntu-latest` + `sudo apt-get install -y mesa-vulkan-drivers` + `WGPU_BACKEND=vulkan` gives kittest the lavapipe CPU Vulkan adapter, headless, no display, no xvfb. Plan B (pinned Mesa tarball) was **never needed**. Plan C (llvmpipe-GL, `WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1`) was run as always-run extra data and **also works on runners** — it even passes the 0.6 threshold against the Plan-A corpus (full backend swap, 0 diff pixels). The STOP condition never approached.

**Adapter string (spike-A, all instances identical):**
```
AdapterInfo { name: "llvmpipe (LLVM 20.1.2, 256 bits)", vendor: 65541, device: 0,
  device_type: Cpu, driver: "llvmpipe",
  driver_info: "Mesa 25.2.8-0ubuntu0.24.04.2 (LLVM 20.1.2)", backend: Vulkan }
```
NOTE (surprise, minor): lavapipe self-reports its device NAME as "llvmpipe (…)" — the Vulkan frontend is lavapipe but the Gallium core name leaks through. Never grep for "lavapipe" in a future assertion; key on `device_type: Cpu` + `backend: Vulkan`.
spike-C adapter: same name/device_type, `backend: Gl`, driver_info `4.5 (Core Profile) Mesa 25.2.8-0ubuntu0.24.04.2`.

## The three spec-§3 items

### (i) Headless render on a real runner — PROVEN
spike-A rendered all 61 forms + the fixed-size arm (62 snapshots) on every instance, zero render errors, across 4 job instances (run 28555238627; run 28555563213 attempts 1/2/3). This closes the ONE recon-unverified residual (lavapipe-Vulkan live on a runner; recon had it only by wgpu-CI precedent + apt filelist).

### (ii) Same-env byte-identity — PROVEN, 4/4 job instances
Each spike-A job runs the full test TWICE and sha256-compares the 62-PNG sets: `BYTE-IDENTITY: OK — 62/62 PNGs identical across run-1/run-2` in run 1 and in all three attempts of run 2 (log-verified each time).

### (iii) ≥3 distinct runner instances pass 0.6 vs the committed run-1 corpus — PROVEN with margin (6 instances)
Corpus = spike-A run-1's PNGs, committed at `0c618d1` (62 files, byte-exact from artifact `spike-pngs-vulkan-28555238627-1`). Every subsequent comparison used kittest's dify default threshold 0.6.

| sample | run/attempt | job | runner instance | image | CPU | result |
|---|---|---|---|---|---|---|
| corpus gen | 28555238627 a1 | spike-A | GitHub Actions 1000009076 | ubuntu24/20260628.225.1 | EPYC 7763 (avx2, 256-bit) | 62 PNGs generated; in-job byte-identity OK |
| gen-side GL | 28555238627 a1 | spike-C | GitHub Actions 1000009087 | ubuntu24/20260628.225.1 | EPYC 9V74 | 62 rendered (pre-corpus, no compare) |
| **1** | 28555563213 a1 | spike-A | **1000009096** | 20260628.225.1 | EPYC 7763 | **124/124 match @0.6** (2 in-job runs × 62) |
| **2** | 28555563213 a1 | spike-C | **1000009100** | 20260628.225.1 | EPYC 7763 | **62/62 match @0.6 — FULL BACKEND SWAP (GL vs Vulkan corpus)** |
| **3** | 28555563213 a2 | spike-A | **1000009101** | **20260622.220.1** | EPYC 7763 | **124/124 match @0.6** |
| **4** | 28555563213 a2 | spike-C | **1000009102** | 20260622.220.1 | EPYC 7763 | **62/62 match @0.6** (GL) |
| **5** | 28555563213 a3 | spike-A | **1000009103** | 20260628.225.1 | EPYC 7763 | **124/124 match @0.6** |
| **6** | 28555563213 a3 | spike-C | **1000009104** | 20260622.220.1 | EPYC 7763 | **62/62 match @0.6** (GL) |

Zero `status=FAIL`/diff across all 558 threshold comparisons (grep-verified `186 match / 0 non-match` per attempt log). Three of the six passing instances are the pinned Plan-A environment; the other three are the Plan-C backend — a strictly harder pass. Two distinct runner VM images represented.

**Anchor hashes (sha256, corpus @ `0c618d1`; full 62-line manifest = artifact `hashes.txt`, itself sha256 `3a3e26b2…4192611`):**
- `mnemonic-verify-bundle.png` `e0fe0c42be8fbf0d3421247b070cf504a4988dd54f061e4ef533b3f39c7ecf9d` (the tallest)
- `mnemonic-inspect.png` `057513b6084ded9f2af4fa76968a68df39942bf9292e508f43c94b9591b875a4` (secret-bearing form)
- `ms-vectors.png` `77b72cad…482d1210` (smallest)
- `tallest-fixed-800x600.png` `fd3da73c…769efc3f` (the sizing-comparison arm)

## M2 measurements (the sizing-ratification evidence)

### Tallest form + no-clipping (I4)
- **Schema flag-row ranking** (Present flags + positionals + Run; top 10): `verify-bundle 29, restore 27, bundle 21, convert 21, build-descriptor 19, export-wallet 19, xpub-search-account-of-descriptor 17, xpub-search-passphrase-of-xpub 17, xpub-search-path-of-xpub 16, md-encode 15`.
- **Measured PNG heights** (physical px @ ppp 2.0; top 6): `verify-bundle 850×1253, restore 850×1169, convert 1008×957, bundle 857×912, export-wallet 919×828, build-descriptor 778×827`. The two rankings agree on the top-2; convert out-ranks bundle on pixels (wider composite widgets). NOTE: the tallest is **verify-bundle**, not bundle/import-wallet as spec §4's aside guessed (import-wallet is only 14 rows).
- **`fit_contents()` vs fixed 800×600:** the fixed frame renders 1600×1200 physical px; verify-bundle's content is 1253 px tall → **a fixed 800×600 viewport WOULD clip the tallest form** (it is the only form that would). Under `fit_contents()` every form's full content fits by construction; the `Run` node (the LAST rendered widget) was tree-asserted present for **all 61** forms, and the verify-bundle + inspect PNGs were opened and visually confirmed complete (all flag rows + positionals + Run visible, dark theme, secret fields masked/empty).
- **Sizing recommendation for P0-R0 ratification:** `fit_contents()` at ppp 2.0 — measured, no-clipping bar met on the tallest form, and it behaves exactly as spec §4 hoped with the non-scrolling whole-form path (no ScrollArea in the harness render).

### Timing
- **Full-61 render+snapshot loop:** 16.5–22.1 s across 7 CI samples (both jobs, all attempts); 9.6 s locally.
- **Job wall-time:** COLD (no rust-cache): spike-A 3 m 11 s, spike-C 2 m 54 s. CACHED: spike-A 1 m 37–41 s, spike-C 1 m 09–20 s. The permanent P1 `snapshots` job should land ≈1.5–2 min warm.

### Corpus size vs the ~3.4 MB estimate
- **61 form PNGs = 2,221,962 B (~2.12 MiB) — 65% of the estimate.** With the spike-only fixed-size arm: 2,377,391 B committed.
- Largest: `tallest-fixed-800x600` 155,429 B; `verify-bundle` 140,140 B; `restore` 131,695 B; `convert` 106,738 B; `bundle` 105,381 B; `export-wallet` 101,912 B. Smallest: `ms-vectors` 3,740 B (168×110); `md-vectors` 4,043 B; `md-gen-man` 4,365 B.
- Plain-committed PNGs confirmed comfortably sane (no LFS, per spec §5).

## MSRV live check (plan I4)
`cargo +1.88.0 check --locked --all-targets` → **exit 0** (dify/colored/getopts/rayon fingerprints confirmed compiled at 1.88.0). **No pre-existing dev-dep floor violation.** (One pre-existing `dead_code` *warning* in test `secret_taxonomy_pin` — warning-only, unrelated, not a floor issue.)

## Cargo.lock delta
Plan predicted "dify/colored/getopts/rayon only". **Actual: those 4 roots + 8 transitive entries = 12 new packages:** `colored, crossbeam-deque, crossbeam-epoch, dify, either, getopts, gpu-allocator, presser, range-alloc, rayon, rayon-core, unicode-width@0.2.2` (a SECOND unicode-width; 0.1.14 stays). **No existing package version moved** (the single deletion line is the `unicode-width` dep-entry disambiguation). `gpu-allocator`/`presser`/`range-alloc` come from kittest's *explicit* `wgpu` dep features (`metal`,`dx12`) — target-gated to windows/apple at build time, locked cross-platform by design. All dev-graph only; `cargo build --release` never builds them. **Benign; report-not-fix.**

## Findings / surprises (P0-R0 attention items)
1. **Cross-env byte drift is real but tiny, exactly as recon said:** 20/62 PNGs differ at the BYTE level between CI-lavapipe-Vulkan, CI-llvmpipe-GL, and a local llvmpipe-GL (Mesa 26.1.2 / LLVM 22.1.6) — yet **0 pixels exceed 0.6** in every cross-comparison CI ran. Same-env renders were byte-stable every time.
2. **Do NOT use corpus byte-size as an identity signal:** all three environments produced the *identical total* (2,221,962 B) and identical per-file sizes while 20 files differ in content — sub-LSB pixel changes compress to equal sizes. Only hashes/threshold-diffs mean anything.
3. **The GUI_SNAPSHOTS skip marker is libtest-captured:** the schema-mirror job (`cargo test --workspace`, no `--nocapture`) ran the spike test GREEN with the early-return skip, but the `SPIKE-SKIP` eprintln is invisible in its log (captured output of a passing test). The env-gate itself works perfectly; P1's ran-at-all census (`.new.png` count == 61) is the real tripwire, and the P1 snapshots job should pass `--nocapture` if log-visible markers are wanted.
4. **lavapipe reports as "llvmpipe"** (see adapter note above) — matters for any future adapter-string assertion.
5. **Plan-C GL passes the threshold against the Vulkan corpus on every instance** — direct evidence for the spec-§5 M4 regeneration UX (contributors regenerate on any software rasterizer; the pinned CI gate arbitrates).
6. **Runner fleet observed:** AMD EPYC 7763 (7 of 8 job instances) + one EPYC 9V74; all 256-bit vector width per the adapter string; the AVX-512-JIT byte-identity question never materialized on this fleet (and doesn't need to — the gate is threshold-based).
7. **First-run mechanics that P1 inherits:** kittest always writes `{name}.new.png` before comparing, so run-1 corpus harvesting needed no `UPDATE_SNAPSHOTS` flip (the spike test tolerates missing-baseline explicitly; the P1 suite, which hard-fails, uses the plan's I2-folded temp-flip path instead if its PR corpus REDs).
8. **Lock delta wider than the plan's 4-package line** (finding above) — the P1/R0 text should cite 12.

## Deliverable state
- **PR:** https://github.com/bg002h/mnemonic-gui/pull/27 (DRAFT, do-not-merge; body documents the laboratory protocol).
- **Branch:** `feat/gui-form-snapshots` @ `0c618d1` (= `2655dc3` spike code + `0c618d1` corpus), based on master `a522a69`.
- **All 13 other checks GREEN on BOTH commits** (clippy `--all-targets -D warnings`, headless no-default-features, msrv 1.88.0, 7-target build matrix, schema-mirror gate; `release` skipped = tag-only, expected). Nothing deliberately red.
- **Spike files are throwaway per plan:** `tests/spike_form_snapshots.rs` + `.github/workflows/spike-snapshots.yml` + `tests/snapshots/spike/` die in P1 (recipe absorbed into `build.yml`; corpus regenerated under `tests/snapshots/forms/`). `.gitignore` now carries the `.new/.diff/.old` kittest-artifact lines (P1 keeps them).
- **Local pre-flight artifacts** (llvmpipe-GL logs + hash manifests + downloaded CI artifacts) retained session-side; the durable evidence is in the PR's run logs + artifacts (`spike-pngs-vulkan-28555238627-1`, `spike-pngs-gl-28555238627-1`, and the per-attempt uploads of run 28555563213).

## What P0-R0 needs to ratify
1. **Sizing: `fit_contents()` @ ppp 2.0** (evidence above; the fixed-viewport alternative demonstrably clips the tallest form).
2. **Rasterizer recipe for the permanent job: Plan A** (`mesa-vulkan-drivers` + `WGPU_BACKEND=vulkan`), with Plan C proven as the documented local-regeneration/fallback path.
3. The corpus-size/time budget (2.1 MiB / ~1.5–2 min warm job) — both under estimate.
