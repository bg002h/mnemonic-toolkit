# SPEC — A1-gui-pin-drift: cross-repo GUI-pin drift-check (warn-only CI gate)

**Repo:** `/scratch/code/shibboleth/mnemonic-toolkit`
**Source SHA (live, == recon SHA):** `572a15d1`
**FOLLOWUP slug:** `install-sh-gui-sibling-pin-staleness-ungated` — open remainder **(a) / "LANE-PIN-B"**
**SemVer:** NO-BUMP. **CI-only YAML. NO-TAG.** No funds / wire / clap / manual-mirror / schema_mirror surface.
**Grounding recon:** `cycle-prep-recon-install-sh-gui-sibling-pin-staleness-ungated.md` (re-verified live below).

---

## 1. Problem (one paragraph)

`scripts/install.sh`'s `component_info` table pins the GUI via `cargo install --git --tag mnemonic-gui-vX.Y.Z`. The **toolkit self-pin** is CI-gated (`install-pin-check.yml`, tag-event, hard-fail). The **GUI pin has no equivalent gate** — it can silently lag the latest released `mnemonic-gui-v*` tag (the v0.55.2 case: 8 versions stale, shipping a GUI missing secret-exposure fixes). There is **no live drift today** (pin == latest, see §5), so this is a *prevent-the-next-regression* hygiene gate, GREEN on first run.

## 2. Decision (fixed)

Add ONE net-new **warn-only** workflow `.github/workflows/gui-pin-drift-check.yml` that:
- extracts the `mnemonic-gui-v*` pin from `scripts/install.sh` (the arm at **`scripts/install.sh:44`**),
- fetches the **latest released** `mnemonic-gui` tag via `gh release list -R bg002h/mnemonic-gui --limit 1` (read-only cross-repo, public repo, default `GITHUB_TOKEN`),
- emits `::warning::` on drift and **exits 0 always** (a cross-repo network call must never hard-fail a PR),
- is **version-aware** — uses `gh release list --limit 1` (latest-by-date, correct) + `sort -V` for direction (NOT lexical `tail`, which falsely ranks `v0.9.0` > `v0.49.0`),
- adds a **daily `schedule:` cron** so "GUI moves, toolkit doesn't" is caught even with zero toolkit commits.

Then flip the FOLLOWUP systemic leg (a) → resolved in the shipping commit.

## 3. Live citation re-verification (all confirmed at `572a15d1`)

| Claim | Live check | Result |
|---|---|---|
| GUI pin location/value | `git show origin/master:scripts/install.sh` → `grep -n 'mnemonic-gui-v'` | **`scripts/install.sh:44`** = `mnemonic-gui\|...\|mnemonic-gui-v0.49.0\|no\|` |
| Latest released GUI tag | `gh release list -R bg002h/mnemonic-gui --limit 1 --json tagName --jq '.[0].tagName'` | `mnemonic-gui-v0.49.0` → **pin == latest, NO drift** |
| `gh`-in-CI precedent (`gh release` + `GH_TOKEN`) | `manual.yml:26-29` (`permissions: contents: write` / `env: GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}`), `:132-141` (`gh release view/create/upload`); `manual-gui.yml:180-183,206-216`; `quickstart.yml:28-31,98-107` | confirmed — established pattern |
| `install-pin-check.yml` is self-pin only | header `:10-12` "Cross-repo pins (mnemonic-gui, md-cli, ms-cli, mk-cli) … the CI gate can't reach those without cross-repo network calls" | confirmed — this gate fills exactly that gap |
| `sibling-pin-check.yml` emits `::warning::` (warn-not-fail precedent) | recon §4 / cross-cutting obs 4 | confirmed |
| Audit recipe | `design/RELEASE_CHECKLIST.md:42-51` (`gh release list -R bg002h/mnemonic-gui --limit 1` + the `grep -oE '...-v[0-9.]+' scripts/install.sh` extractor) | confirmed — this gate automates `:43-51` |
| Repo checkout convention | all 4 cited workflows use `actions/checkout@v6` | use `@v6` |
| Tooling | `actionlint 1.7.12`, `gh 2.95.0` present | OK |

## 4. EXACT change — add `.github/workflows/gui-pin-drift-check.yml`

Create the file **verbatim** below. It is **actionlint-clean (exit 0, verified at 1.7.12; actionlint's internal shellcheck pass over the `run:` block is also clean)**, and its two live lookups were smoke-run (§5).

```yaml
name: gui-pin-drift-check

# Warn-only cross-repo drift check for the mnemonic-gui pin in
# scripts/install.sh's component_info table (the `mnemonic-gui` arm,
# currently scripts/install.sh:44). Compares the pinned tag against the
# LATEST RELEASED mnemonic-gui-v* tag in the sibling repo and emits a
# GitHub Actions ::warning:: on drift.
#
# Why warn-only (exit 0 even on drift): this is a CROSS-REPO NETWORK call
# (gh release list -R bg002h/mnemonic-gui). GitHub API can be down /
# rate-limited / the runner offline; that must NOT hard-fail unrelated
# PRs. The install-pin-check.yml gate hard-fails because it is purely
# LOCAL (install.sh self-pin vs the tag being built). This gate cannot
# be local — the truth lives in another repo — so it advises, never
# blocks. (Precedent for warn-not-fail: sibling-pin-check.yml emits
# ::warning:: for unknown-url lines.)
#
# Why a schedule: the failure mode is "GUI moves, toolkit doesn't" — a
# new mnemonic-gui release lands while this repo has no commits. A
# push/PR-only trigger would never fire in that window, so a daily cron
# catches drift even with zero toolkit activity.
#
# Why sort -V (version sort), not lexical tail: mnemonic-gui's tag set
# lexically tails at mnemonic-gui-v0.9.0 (a "9" sorts after a "4"), but
# the true latest is v0.49.0. `gh release list --limit 1` returns the
# most-recent release by publish DATE (not lexical), which is correct;
# the awk numeric comparison below is the version-aware drift test.
#
# Scope: this gate is the systemic cross-repo leg of FOLLOWUP
# `install-sh-gui-sibling-pin-staleness-ungated` (open remainder (a) /
# "LANE-PIN-B"). The toolkit self-pin stays gated by install-pin-check.yml;
# intra-repo install.sh<->docs consistency stays gated by
# sibling-pin-check.yml. This adds the missing cross-repo CURRENCY check.

on:
  push:
    branches: [main, master]
    paths:
      - 'scripts/install.sh'
      - '.github/workflows/gui-pin-drift-check.yml'
  pull_request:
    paths:
      - 'scripts/install.sh'
      - '.github/workflows/gui-pin-drift-check.yml'
  schedule:
    # Daily at 06:17 UTC. Off-the-hour minute avoids the top-of-hour
    # scheduler congestion GitHub warns about for cron workflows.
    - cron: '17 6 * * *'
  workflow_dispatch:

permissions:
  contents: read

jobs:
  gui-pin-drift:
    name: install.sh mnemonic-gui pin vs latest release
    runs-on: ubuntu-latest
    env:
      GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    steps:
      - uses: actions/checkout@v6

      - name: Compare install.sh mnemonic-gui pin to latest released tag
        run: |
          set -eu

          # Pinned tag from install.sh's component_info `mnemonic-gui` arm.
          # The line format is:
          #   echo "mnemonic-gui|<url>|mnemonic-gui-vX.Y.Z|no|"
          # Anchor on `mnemonic-gui-v` so we never match the bare
          # `mnemonic-gui|` package/url tokens.
          PIN=$(grep -oE 'mnemonic-gui-v[0-9]+\.[0-9]+\.[0-9]+' scripts/install.sh | head -n1)
          if [ -z "$PIN" ]; then
            echo "::warning::could not extract a mnemonic-gui-v* pin from scripts/install.sh — skipping drift check"
            exit 0
          fi

          # Latest released mnemonic-gui tag. `gh release list --limit 1`
          # returns the most-recent release by publish DATE (releases are
          # cut in version order, so this is the true latest — NOT a
          # lexical tail). Read-only against the PUBLIC sibling repo; the
          # default GITHUB_TOKEN can read public releases.
          LATEST=$(gh release list -R bg002h/mnemonic-gui --limit 1 \
                     --json tagName --jq '.[0].tagName' 2>/dev/null || true)
          if [ -z "$LATEST" ]; then
            echo "::warning::could not fetch the latest mnemonic-gui release (network/rate-limit/none) — skipping drift check"
            exit 0
          fi

          echo "install.sh mnemonic-gui pin: $PIN"
          echo "latest mnemonic-gui release: $LATEST"

          if [ "$PIN" = "$LATEST" ]; then
            echo "OK install.sh mnemonic-gui pin is current ($PIN)"
            exit 0
          fi

          # Drifted. Determine direction with a VERSION-AWARE compare
          # (sort -V): if PIN sorts strictly below LATEST, install.sh lags.
          # Strip the shared `mnemonic-gui-v` prefix to dotted versions so
          # sort -V sees pure X.Y.Z. (Equality is handled above.)
          PIN_V=${PIN#mnemonic-gui-v}
          LATEST_V=${LATEST#mnemonic-gui-v}
          LOWER=$(printf '%s\n%s\n' "$PIN_V" "$LATEST_V" | sort -V | head -n1)
          if [ "$LOWER" = "$PIN_V" ]; then
            echo "::warning::install.sh mnemonic-gui pin '$PIN' LAGS the latest release '$LATEST'. Bump scripts/install.sh:44 and commit: chore(install): bump mnemonic-gui pin $PIN -> $LATEST"
          else
            echo "::warning::install.sh mnemonic-gui pin '$PIN' is AHEAD of the latest release '$LATEST' (unreleased GUI tag pinned?). Verify scripts/install.sh:44."
          fi
          # Warn-only: never block a PR on a cross-repo currency mismatch.
          exit 0
```

### Design points the implementer must NOT alter (R0-load-bearing)
1. **`permissions: contents: read`** — read-only; the public GUI repo's releases are readable by the default `GITHUB_TOKEN`. Do NOT escalate to `contents: write` (the manual workflows need write only because they UPLOAD assets; this gate only reads).
2. **`exit 0` on every path** — the three exit points (no pin / no fetch / drift) and the equal case all exit 0. Hard-failing on a cross-repo network call is prohibited.
3. **`gh release list --limit 1`** (latest-by-date) + **`sort -V`** for direction. Never `tag | tail` (lexical). The `${PIN#mnemonic-gui-v}` prefix-strip is required so `sort -V` sees pure `X.Y.Z`.
4. **Anchored extractor** `grep -oE 'mnemonic-gui-v[0-9]+\.[0-9]+\.[0-9]+'` — anchoring on `-v` avoids matching the bare `mnemonic-gui|` package/url tokens on the same line. `head -n1` guards against future duplicate lines.
5. **`schedule:` cron is mandatory** — it is the only trigger that catches "GUI moves, toolkit idle." Keep `push`/`pull_request` path-filtered to `scripts/install.sh` + the workflow file (cheap), plus `workflow_dispatch` for manual runs.
6. **No untrusted event input** in `run:` — only `${{ secrets.GITHUB_TOKEN }}` via `env:` and `actions/checkout@v6` with no `ref:`. (Satisfies the workflow-injection guidance; nothing to escape.)

## 5. Verification (run locally before commit)

```sh
# (a) actionlint — THE load-bearing gate. MUST exit 0, zero findings.
actionlint .github/workflows/gui-pin-drift-check.yml
echo "exit: $?"          # expect 0

# (b) the gate's two lookups (what CI runs), from repo root:
grep -oE 'mnemonic-gui-v[0-9]+\.[0-9]+\.[0-9]+' scripts/install.sh | head -n1
#   => mnemonic-gui-v0.49.0
gh release list -R bg002h/mnemonic-gui --limit 1 --json tagName --jq '.[0].tagName'
#   => mnemonic-gui-v0.49.0   (== pin => NO ::warning::; gate GREEN on first run)

# (c) sort -V direction sanity (proves the lexical-vs-version trap is avoided):
printf '%s\n%s\n' "0.9.0" "0.49.0" | sort -V | head -n1
#   => 0.9.0   (correctly the LOWER; lexical tail would wrongly pick it as "latest")
```

**All of (a)/(b)/(c) were executed at spec-authoring time and produced exactly the expected output** (actionlint exit 0; both lookups = `mnemonic-gui-v0.49.0`; `sort -V` ranks `0.9.0` below `0.49.0`).

After merge, confirm the **NEW gate** itself: push the branch / open the PR (it touches `scripts/install.sh`? no — it touches only the new workflow, which IS in the `paths` filter, so it fires) → the `gui-pin-drift` job runs and prints `OK install.sh mnemonic-gui pin is current (mnemonic-gui-v0.49.0)` and passes. You can also trigger it manually via the `workflow_dispatch` button.

## 6. FOLLOWUP flip (in the shipping commit)

Edit `design/FOLLOWUPS.md` entry `install-sh-gui-sibling-pin-staleness-ungated` (header at **`:278`**, Status bullet at **`:285`** in the live file). The leg to flip is **open remainder (a)** only — leg (b) (the `mnemonic-gui` `Cargo.toml rust-version` paired-PR) stays open (not a toolkit lane).

In the **Status** bullet, change the `**OPEN remainder:**` clause from:

> **OPEN remainder:** (a) the systemic cross-repo `gh api` drift-check comparing `install.sh`'s GUI pin to the latest `mnemonic-gui-v*` tag (LANE-PIN-B); (b) the GUI `Cargo.toml` `rust-version "1.85"→"1.88"` correction + GUI README prereq line — a SEPARATE `mnemonic-gui` paired-PR (not this toolkit lane).

to:

> **OPEN remainder:** ~~(a) the systemic cross-repo `gh api` drift-check~~ **RESOLVED (2026-06-23, NO-BUMP):** the systemic cross-repo leg (LANE-PIN-B) shipped as `.github/workflows/gui-pin-drift-check.yml` — a warn-only `gh release list -R bg002h/mnemonic-gui --limit 1` check comparing the `scripts/install.sh:44` GUI pin to the latest released tag (version-aware via `sort -V`; daily `schedule:` cron; `::warning::` + exit 0 on drift; `permissions: contents: read` + `GH_TOKEN: secrets.GITHUB_TOKEN`). No live drift at ship (pin == `mnemonic-gui-v0.49.0`). **Remaining OPEN:** (b) the GUI `Cargo.toml` `rust-version "1.85"→"1.88"` correction + GUI README prereq line — a SEPARATE `mnemonic-gui` paired-PR (not this toolkit lane).

(If both (a) and (b) are ever closed, change the top-level `**Status:**` token from `partially-resolved`. For this cycle (a) closes but (b) remains, so the entry stays `partially-resolved` — do NOT mark the whole slug resolved.)

## 7. OPTIONAL (cosmetic, not gate-affecting) — `design/RELEASE_CHECKLIST.md`

Per the recon, optionally add a one-line pointer in the `## Pre-release ritual` section (after the `:51` `grep` block, before the `If any pin LAGS…` line at `:54`) noting the GUI-pin audit is now CI-assisted:

```md
> The `mnemonic-gui` pin specifically is also watched continuously by
> `.github/workflows/gui-pin-drift-check.yml` (warn-only, daily cron) —
> it ::warning::s when `scripts/install.sh:44` lags the latest released
> `mnemonic-gui-v*` tag. (md/ms/mk pins remain manual-audit-only.)
```

This is purely documentation; it has no gate. Implementer may omit it.

## 8. Commit / staging

- Stage paths **explicitly** (no `git add -A`):
  - `.github/workflows/gui-pin-drift-check.yml`
  - `design/FOLLOWUPS.md`
  - (optional) `design/RELEASE_CHECKLIST.md`
- One commit. Suggested message:
  `ci(install): warn-only cross-repo mnemonic-gui pin drift-check (NO-BUMP)` + flip FOLLOWUP `install-sh-gui-sibling-pin-staleness-ungated` leg (a) → resolved.
- **NO version bump, NO tag.** Do not touch `Cargo.toml` / `Cargo.lock` / READMEs / `CHANGELOG.md` (none of the version-site gates apply — this is CI-only).

## 9. Locksteps — NONE

No clap flag/value/subcommand change ⇒ no GUI `schema_mirror`. No CLI surface change ⇒ no manual-mirror (`docs/manual/src/40-cli-reference/`). No sibling-codec API change ⇒ no companion FOLLOWUP. No funds/wire path touched.