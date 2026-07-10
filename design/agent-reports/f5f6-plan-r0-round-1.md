# PLAN R0 review — F5+F6 GUI recovery wiring — round 1

**Reviewer:** Fable (plan R0, read-only), per user directive. Plan @ GUI `f5cb11f` / toolkit `3d985798`.
**Dispatched:** 2026-07-10 (F5+F6, plan-R0 round 1). Persisted verbatim per CLAUDE.md.

## VERDICT: NOT GREEN — 0 Critical / 2 Important / 3 Minor. Both Importants are single-line plan folds; everything structural verified sound.

## IMPORTANT
**I-A — the I-2 snapshot-regen invocation is a silent green no-op as written** (plan `:16`, G-D `:33`). `UPDATE_SNAPSHOTS=1 cargo test --test gui_form_snapshots` regenerates NOTHING: the harness early-return-skips unless `GUI_SNAPSHOTS=1` (`tests/gui_form_snapshots.rs:109-116`), the skip `eprintln!` is capture-swallowed on a passing test, cargo exits 0 "2 passed" → empty git diff, implementer stalls/misreads. Compounding: **this machine has no lavapipe** (`/usr/share/vulkan/icd.d/` = only `intel_icd.json`/`intel_hasvk_icd.json`), so `GUI_SNAPSHOTS=1` alone fails the A1 adapter guard (`device_type==Cpu`, `:126-133`). The sanctioned local path is the **llvmpipe-GL Plan-C route** (harness header `:34-37`; verified llvmpipe present: `LIBGL_ALWAYS_SOFTWARE=1 glxinfo` → `llvmpipe (LLVM 22.1.6)`):
```
GUI_SNAPSHOTS=1 WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 UPDATE_SNAPSHOTS=1 cargo test --test gui_form_snapshots
```
**Fold:** replace the invocation in P1/I-2 + G-D; keep the "git diff is EXACTLY the one PNG" backstop. Supporting: egui_kittest 0.31.1 `UPDATE_SNAPSHOTS` rewrites ONLY snapshots whose dify diff > 0.6 (`egui_kittest-0.31.1/src/snapshot.rs:245-295`) → sub-threshold cross-rasterizer drift on the other 60 does NOT rewrite (single-PNG-diff expectation correct); `.new/.old/.diff.png` gitignored. Local regen is POSSIBLE (not a blocker) via the full env combo.

**I-B — release ritual omits the README install one-liner** (plan `:25`). `mnemonic-gui/README.md:47` pins `--tag mnemonic-gui-v0.57.0`; git history shows it bumped in v0.55/v0.56/v0.57 releases, and it is UNGATED (no GUI install-pin-check) → missing it ships a stale install instruction. **Fold:** add "`README.md:47` install `--tag` → `mnemonic-gui-v0.58.0`" to the version-site list.

## MINOR
- **M-1** — GUI DOES keep `CHANGELOG.md` (ungated; only a comment ref in `schema-mirror.yml:10`). Make the plan's entry unconditional: "add the v0.58.0 CHANGELOG.md entry (ungated)."
- **M-2** — P2 defaults-drift test must follow the binary-resolution + skip convention (`tests/schema_mirror.rs:606-614`: `MNEMONIC_BIN` → PATH fallback → loud SKIP when absent). (a) The required `schema-mirror gate` job installs pinned v0.75.0 — fine; (b) the PATH fallback on a dev machine picks the installed `mnemonic` (here v0.84.0, 9 minors past the pin) → spurious drift outside CI. Name the cause in P2.
- **M-3** — merge-time gotcha: `gh pr merge` local-branch cleanup fails under a worktree though the server merge succeeds → verify via `gh pr view --json state,mergeCommit`.

## VERIFIED GREEN
- **Phase split** coherent; F5+F6 ONE PR necessary (activation chain); folding I-3 into P1 CORRECT (mutually dependent — the migration's reset `Dropdown("")` is only legal once the `_INFER` consts land; consts-without-migration leaves the at-risk population bugged; separate phases = broken intermediate). P2 gate has concrete STOP + named FOLLOWUP + "do NOT block" → can't block.
- **Release/CI:** version sites `Cargo.toml:3` + Cargo.lock self `:2351` + README `:47` (I-B). NO changelog gate, NO fmt gate (both workflows swept: `clippy`, `headless (no-default-features)`, `msrv (1.88.0)`, `snapshots`, `tutorial-snapshots`, matrix target, `schema-mirror gate`). NO toolkit-pin bump (pin `Cargo.toml:76` v0.75.0; F5 assembler + F6 GUI consts, neither clap-surface). Branch protection live-fetched = exactly `[snapshots, clippy, headless (no-default-features), schema-mirror gate, x86_64-unknown-linux-gnu]`, enforce_admins:false. `tutorial-snapshots` genuinely unaffected (148-file corpus, zero nested/xpub-search fixtures, never calls the assembler).
- **Snapshot regen** threshold-gated (only address-of-xpub crosses 0.6); committing the one PNG right.
- **Guard-rails** sufficient; G-C exact key verified (`persistence.rs:59-60`); the "INDEPENDENT of the default_value gate" note verified (`:389-391`; both flags `None`-default at `schema/mnemonic.rs:3243/:3256`); all 8 I-1 sites exact; `invocation.rs:161/:116/:152` exact.

**Fold I-A + I-B (+ M-1/M-2), re-dispatch round 2 (fast).**
