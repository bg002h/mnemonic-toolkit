# PLAN R0 review — F5+F6 GUI recovery wiring — round 2 (convergence)

**Reviewer:** Fable (plan R0 convergence, read-only). Plan @ GUI `f5cb11f`. Round-1: `f5f6-plan-r0-round-1.md`.
**Dispatched:** 2026-07-10 (F5+F6, plan-R0 round 2). Persisted verbatim per CLAUDE.md.

## VERDICT: GREEN — 0 Critical / 0 Important (2 non-blocking Minors). Clear to implement toward mnemonic-gui-v0.58.0.

## Folds verified
**I-A (snapshot no-op) — RESOLVED, env combo exactly correct + complete.** Plan `:18`/`:35` carry `GUI_SNAPSHOTS=1 WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 UPDATE_SNAPSHOTS=1 cargo test --test gui_form_snapshots` — VERBATIM the harness regen header (`tests/gui_form_snapshots.rs:34-37`). A1 guard (`:126-133`) asserts only `device_type==Cpu` (llvmpipe-GL satisfies); backend assert honors `WGPU_BACKEND=gl`; `WGPU_ADAPTER_NAME` affirmatively NOT used (header forbids name matching). `UPDATE_SNAPSHOTS=1` truthy (egui_kittest `snapshot.rs:165-169`); test SUCCEEDS after update (no confusing red). Silent-no-op now impossible: `GUI_SNAPSHOTS=1` runs the suite; wrong env → A1 guard fails LOUDLY (assert, not skip); one-PNG git-diff backstop catches empty regen.
**I-B (README) — RESOLVED.** Plan `:27` adds `README.md:47` `--tag`→v0.58.0 (live line). Repo-wide grep: the ONLY stale `mnemonic-gui-v0.57.0` version-site in tracked non-workflow files (other hits = FOLLOWUPS/CHANGELOG historical + src provenance comments — none render as version/install). No `CARGO_PKG_VERSION`/about-box render in `src/`. `pinned-upstream.toml` stays v0.75.0.
**M-1/M-2/M-3 — captured** (unconditional CHANGELOG; defaults-drift `MNEMONIC_BIN`→PATH→LOUD-SKIP with the v0.84.0-vs-pin cause named, block verified `tests/schema_mirror.rs:608-619`; worktree `gh pr merge` gotcha + `gh pr view` verify).

## No new gaps
Version anchors live: `Cargo.toml:3`=0.57.0, `Cargo.lock:2351`. Required-context list matches live branch protection (5, enforce_admins:false). Threshold claim matches kittest (>0.6 rewrites; CI lavapipe arbitrates the llvmpipe baseline — harness blesses cross-rasterizer regen). Structural items unchanged from round-1 GREEN.

## MINOR (fold at ship, no re-review)
- **M-4** — GUI `FOLLOWUPS.md:15-21` `network-dropdown-default-forces-explicit-mainnet` (OPEN) overlaps F6's `(none)`-sentinel fix for `address-of-xpub`. F6 does NOT close it (`convert`/`export-wallet` `--network` dropdowns remain) → add a one-line partial-ship annotation + cross-cite at release (couples with the planned `gui-dropdown-none-opts0-materialization-audit` FOLLOWUP).
- **M-5** — citation drift: plan `:21` cites `:606-614`; actual block `:608-619`. Trivial.

**CONVERGED — GREEN (0C/0I). Implementation may begin (single Opus implementer, worktree `feature/f5-f6-recovery-wiring`, TDD, one PR, mnemonic-gui-v0.58.0); fold M-4/M-5 into the release-ritual commit.**
