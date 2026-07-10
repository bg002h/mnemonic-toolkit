# Cycle I post-implementation review — test-hardening (gating #1 + wc-codec CI #2)

**Reviewer:** fresh Fable (post-impl, read-only, adversarial). Applied state @ toolkit `e7f8c73a` + ms `9a24999` + live branch-protection GETs.
**Dispatched:** 2026-07-10 (Cycle I, post-impl round 1). Persisted verbatim per CLAUDE.md.

## VERDICT: GREEN — Cycle I (test-hardening #1 + #2) verified complete and correctly applied. No wedge, direct-FF preserved on all 5 repos.

### 1. #2 CI edits match the SPEC and are green — with hard log evidence
- **toolkit `e7f8c73a`** (= current origin/master HEAD, no later revert): `rust.yml` build step → `cargo build --tests --workspace`, test step → `cargo test --workspace` (both, per SPEC §1); job/context `test (ubuntu-latest)` unchanged (G5); mlock-specific `-p` steps untouched. `pull_request:` now bare (C1). `fuzz-smoke.yml`: wc-codec compile-gate build via `--fuzz-dir crates/wc-codec/fuzz` (push/PR), smoke-run steps for `wc_roundtrip`/`wc_decode_never_panics` + `if: failure()` crash upload (cron/dispatch-only smoke job), `crates/wc-codec/{fuzz,src}/**` added to both paths (I2).
- **ms `9a24999`** (= current ms master HEAD): `pull_request` paths deleted, push filter kept (C2). Exact per SPEC §1b.
- **CI green:** `e7f8c73a` → `rust`, `fuzz-smoke`, `sibling-pin-check` all completed/success; `9a24999` → `rust` completed/success.
- **The point of #2 confirmed from the actual run log** (run 29101232595, job `test (ubuntu-latest)`): all 8 wc-codec integration binaries executed — field 10, pad 5, pipeline 24, raid 13, regroup 8, rs 12, sync 23, wordmap 5 = **100 passed / 0 failed**. Root workspace = exactly 2 members (`mnemonic-toolkit 0.83.0`, `wc-codec 0.1.0`); nested `crates/wc-codec/fuzz` workspace invisible to `--workspace` as reasoned.

### 2. Branch protection: live GETs match the matrix EXACTLY
All five: contexts exactly as specified, every `checks[]` bound to `app_id:15368` (GitHub Actions, matching pre-existing `examples`/`snapshots`), `strict:false`, `enforce_admins.enabled:false`, no required PR reviews, no restrictions, applied to each repo's actual default branch. Toolkit kept `examples`; gui kept `snapshots`. Zero drift.

### 3. NO WEDGE — every required context reports on every PR (verified against live default-branch workflow files)
- toolkit `rust.yml` bare `pull_request`; `examples.yml` bare + born-green guard. All 3 fire on a docs-only PR.
- ms `rust.yml` bare `pull_request`; all 4 required contexts are jobs of this one workflow.
- md `ci.yml`: `push: branches:[main]` + bare `pull_request:` — no paths.
- mk `ci.yml`: bare `pull_request:`; `build (stable on ubuntu-latest)`.
- gui `build.yml`/`schema-mirror.yml`: `pull_request: branches:[master, "release/**"]` — base-branch filter (always fires on PRs to master), NO path filters. `x86_64-unknown-linux-gnu` build job has no tag/`if:` gating + handles PR refs.

### 4. Direct-FF preserved on all 5
`enforce_admins:false` everywhere → admin direct-FF pushes bypass required checks (recorded bypass); no push restrictions; no required reviews. Release ritual (direct-FF + tag) keeps working.

### 5. Completeness — nothing dropped, nothing red
No pre-existing setting lost. No required context red at any default-branch HEAD (md `ci.yml` push run at `ef1f3e71` = success; the earlier crowded check-runs page was ~40 daily-cron fuzz runs, not a gap).

### Minor observations (non-blocking)
- **M-1:** wc-codec **smoke-run** steps are cron/dispatch-only, not yet executed (next cron 07:17 UTC). Build half ran green on push. A `workflow_dispatch` pre-verifies. [Orchestrator: triggered run 29103158506 to pre-verify.]
- **M-2:** SPEC M3's "preferred" matrix shape implemented as sequential steps in the existing job — functionally equivalent, avoids the check-run rename M3 flagged.

**Completion statement:** Constellation-eval §2 cross-cutting #1 ("no repo's suite gates merges") and #2 ("wc-codec runs in no CI") are both CLOSED. wc-codec's 100 tests + 2 fuzz targets now run in toolkit CI (tests on every PR/push via `--workspace`; fuzz compile-gate on push/PR; smoke on cron), and all five constellation repos have their reliable ubuntu test/clippy contexts as required, wedge-free branch-protection checks with the admin direct-FF ritual intact.
