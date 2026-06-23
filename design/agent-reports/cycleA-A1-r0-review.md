## R0 Adversarial Review — SPEC `A1-gui-pin-drift` (cross-repo GUI-pin drift-check, warn-only CI gate)

**VERDICT: GREEN — 0 Critical / 0 Important / 3 Minor. Cleared to implement.**

Source verified live at `572a15d1` (== spec's claimed Source SHA; `git rev-parse HEAD` = `572a15d18c5...`). Every cited path, line number, factual claim, and the workflow YAML itself were re-grepped/executed against current `origin/master`. The spec is implementation-ready.

---

### Mandated verification checklist (all PASS)

| Mandated check | Result | Evidence |
|---|---|---|
| **Warn-only — cannot hard-fail; confirm exit-0-on-drift** | PASS | All four exit points (no-pin / no-fetch / equal / drift) hit `exit 0`. Ran `set -eu` simulations of every path in a subshell: gh-fail path → exit 0; empty-grep path → exit 0; equal path → exit 0. The `PIN=$(grep ... \| head -n1)` pipeline does NOT abort on a no-match under `set -eu` (no `pipefail`), because the pipeline status is `head`'s 0 — verified empirically. The deliberate omission of `pipefail` is load-bearing and correct. |
| **Version-sorted not lexical** | PASS | `printf '0.9.0\n0.49.0\n' \| sort -V \| head -n1` → `0.9.0` (correct LOWER). Lexical `tail` would wrongly rank 0.9.0 as latest; `sort -V` does not. Lag/ahead/equal branch logic all classify correctly in simulation. The `${PIN#mnemonic-gui-v}` prefix-strip yields pure `0.49.0` so `sort -V` sees `X.Y.Z`. |
| **gh-api/gh-release invocation correct + minimally permissioned (contents: read)** | PASS | `gh release list -R bg002h/mnemonic-gui --limit 1 --json tagName --jq '.[0].tagName'` ran live → `mnemonic-gui-v0.49.0` (flagged `isLatest=true`). `permissions: contents: read` is correct and minimal: the precedent workflows (manual.yml:27, manual-gui.yml:181, quickstart.yml:29) all use `contents: write` ONLY because they UPLOAD release assets; this gate only READS public releases, for which the default `GITHUB_TOKEN` + `contents: read` suffices. No over-permissioning. |
| **schedule cron present** | PASS | `schedule: - cron: '17 6 * * *'` present. Format valid. Precedented: repo's own fuzz-smoke.yml uses `17 7 * * *` and bitcoind-differential.yml uses `17 5 * * *` with the identical 'off-the-hour to avoid thundering herd' rationale; the spec's `17 6` slots between them without collision. Scheduled workflows fire from the default branch (`master`, confirmed via `origin/HEAD -> origin/master`), so the cron will run. |
| **Precedent citations accurate (manual.yml GH_TOKEN lines)** | PASS | manual.yml:26-29 = `permissions:`/`contents: write`/`GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}` ✓; :132-141 = `gh release view/create/upload` ✓. manual-gui.yml:180-183 (`permissions`/`contents: write`/`GH_TOKEN`) + :206-216 (`gh release view/create/upload`) ✓. quickstart.yml:28-31 + :98-107 ✓. All `actions/checkout@v6` ✓. install-pin-check.yml header confirms 'self-pin only … Cross-repo pins (mnemonic-gui, md-cli, ms-cli, mk-cli) … the CI gate can't reach those without cross-repo network calls' — this gate fills exactly that gap. sibling-pin-check.yml:134,139 emit `::warning::` (warn-not-fail precedent) ✓. |
| **actionlint-clean** | PASS | actionlint 1.7.12 on the verbatim YAML → exit 0, zero findings (incl. internal shellcheck pass over the `run:` block). |
| **FOLLOWUP flip correct** | PASS | Header at FOLLOWUPS.md:278, Status bullet at :285 (both as spec §6 claims). Top-level token = `partially-resolved`. The §6 `from:` quote is a BYTE-EXACT substring of the live file (verified via `grep -oF`). The flip closes leg (a) / LANE-PIN-B only and correctly keeps (b) (GUI Cargo.toml rust-version paired-PR) open + keeps the slug `partially-resolved`. Correct — do NOT mark the whole slug resolved. |
| **Edge: install.sh has no GUI pin → degrade to warn** | PASS | Empty `$PIN` → `[ -z "$PIN" ]` → `::warning::… skipping` → exit 0. Verified the no-match grep does not abort under `set -eu`. |
| **Edge: gh rate-limits / network down → degrade to warn** | PASS | `gh release list … 2>/dev/null \|\| true` swallows nonzero; empty `$LATEST` → `::warning::… skipping` → exit 0. Verified in subshell sim. |

### Additional live-source confirmations
- `scripts/install.sh:44` = `mnemonic-gui\|https://github.com/bg002h/mnemonic-gui\|mnemonic-gui-v0.49.0\|no\|` — the anchored extractor `grep -oE 'mnemonic-gui-v[0-9]+\.[0-9]+\.[0-9]+'` returns exactly one match (`mnemonic-gui-v0.49.0`); it correctly does NOT match the bare `mnemonic-gui\|`/url tokens on the same line. `head -n1` future-proofs against duplicate lines.
- **No live drift**: pin (`v0.49.0`) == latest release (`v0.49.0`, `isLatest=true`) → gate is GREEN on first run, exactly as the spec claims. This is a prevent-the-next-regression hygiene gate.
- RELEASE_CHECKLIST.md:43-51 grep block + :54 'If any pin LAGS' confirmed; the §7 optional pointer insertion target is accurate.
- gh 2.95.0, actionlint 1.7.12 present (spec §3 tooling row accurate).
- Scope claims hold: no clap/flag/subcommand/dropdown change ⇒ no schema_mirror; no CLI surface ⇒ no manual-mirror; no sibling API ⇒ no companion FOLLOWUP; no funds/wire path. NO-BUMP / NO-TAG / CI-only is correct — none of the version-site gates (Cargo.toml/lock, READMEs, CHANGELOG/changelog-check) apply.

### The 6 R0-load-bearing 'must-not-alter' design points — all sound
1. `contents: read` — confirmed minimal & correct (read-only public releases). ✓
2. `exit 0` on every path — empirically verified across all four branches. ✓
3. `gh release list --limit 1` + `sort -V` + prefix-strip — version-aware, lexical-trap avoided. ✓ (see Minor #1 for an optional hardening of the *candidate* selection, not the comparator.)
4. Anchored extractor + `head -n1` — verified single match, no token collision. ✓
5. `schedule:` cron mandatory — present, valid, precedented; the only trigger catching 'GUI moves, toolkit idle.' ✓
6. No untrusted event input in `run:` — only `${{ secrets.GITHUB_TOKEN }}` via `env:` and `checkout@v6` with no `ref:`; nothing to escape (workflow-injection-safe). ✓

### Minor findings (none block the gate; all optional)
1. **Candidate-selection robustness** — `--limit 1` selects newest-by-DATE; relies on monotonic release ordering. An out-of-order backport would produce a spurious 'AHEAD' warn (never a fail). Optional: select via `isLatest` flag. (Warn-only ⇒ harmless.)
2. **No fork-cron guard** — repo convention (fuzz-smoke / bitcoind-differential) guards scheduled jobs; spec omits a `github.repository ==` guard. Fork crons would warn harmlessly. Optional.
3. **Decaying ':44' in runtime warning text** — the advisory string hardcodes `scripts/install.sh:44`, which decays on edits above the arm. The gate's extractor is line-agnostic; only the message text decays. Optional: say 'the mnemonic-gui arm in scripts/install.sh'.

### Verification recipe for the implementer (all confirmed runnable here)
```
actionlint .github/workflows/gui-pin-drift-check.yml   # exit 0
grep -oE 'mnemonic-gui-v[0-9]+\.[0-9]+\.[0-9]+' scripts/install.sh | head -n1   # mnemonic-gui-v0.49.0
gh release list -R bg002h/mnemonic-gui --limit 1 --json tagName --jq '.[0].tagName'  # mnemonic-gui-v0.49.0
printf '0.9.0\n0.49.0\n' | sort -V | head -n1   # 0.9.0
```

### Gate decision
**0 Critical / 0 Important. GREEN.** The spec passes the R0 gate and is cleared to implement as-written. The 3 Minors are optional hardenings the implementer may fold or skip at discretion; none are required for GREEN. Per the reviewer-loop convention, if any Minor is folded, re-dispatch a scoped convergence review of the fold before commit; if shipped as-written, proceed to the single-subagent implementation.