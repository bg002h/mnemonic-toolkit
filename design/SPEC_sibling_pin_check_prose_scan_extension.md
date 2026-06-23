# SPEC — extend `sibling-pin-check.yml` to scan manual/quickstart prose install commands (LANE4-pincheck / b2)

**Cycle:** LANE4-pincheck — close FOLLOWUP `sibling-pin-check-skips-manual-prose-install-commands` (the b2 sub-item of the D-LANE-PIN-B-gates family).
**Tier:** `ci-hardening`.
**SemVer:** **NO-BUMP** (CI-only workflow edit + FOLLOWUPS status flips; no `crates/mnemonic-toolkit/src`, no clap surface, no Cargo version-site, no README marker, no manual flag-table change). Mirrors the parent gate's own CI-only no-bump landing (`design/FOLLOWUPS.md:579` `manual-yml-sibling-pin-vs-install-sh-drift-gate` → `resolved` "CI-only, no version bump"). The project keeps no `[Unreleased]` CHANGELOG block; `changelog-check.yml` fires only on `mnemonic-toolkit-v*` tags, so no CHANGELOG entry is needed.
**Source SHA at write time:** `origin/master` = `cc9f9dc27f30c234ea4bf434fa883dc6be198408` (== local `HEAD`). All line numbers below re-grepped live against this SHA.

---

## 0 — Scope of this lane (decisions, fixed)

- **b2 — SHIP.** Extend `.github/workflows/sibling-pin-check.yml`: (i) add `docs/manual/src/**` + `docs/quickstart/src/**` prose to the scan set (currently `.github/workflows/*.yml` only); (ii) generalize the pkg/tag extraction so it also matches the prose `--tag <tag> --bin <bin>` form. **Approach (REQUIRED): key the canonical lookup on the `--git <repo-url>`** already present in `install.sh` `component_info` (url → tag), sidestepping the bin↔pkg name map.
- **b1 (`manual-yml-sibling-pin-vs-install-sh-drift-gate`) — FLIP-ONLY CONFIRM.** Already `resolved` + GREEN (`FOLLOWUPS.md:579` `Status: resolved`). No code. Re-affirm in the shipping commit message only; do not re-touch its status line (already resolved).
- **b3 (`install-sh-gui-sibling-pin-staleness-ungated`) — DEFERRED, NOT this lane.** No edit. (Cross-repo `gh api` GUI-tag staleness gate; overlaps item E / GUI-MSRV; no live drift today since `install.sh:44` GUI pin `mnemonic-gui-v0.49.0` == latest published GUI tag.)

This spec is for a **single TDD implementer**. Everything below is grep-verified and the exact bash logic has been run end-to-end against the live tree (clean-tree exit 0 + synthetic-drift exit 1 + tagless-skip), and the full proposed workflow is **actionlint-clean** (actionlint 1.7.12, which runs embedded shellcheck on the `run:` step).

---

## 1 — Problem (current behavior, verified)

`.github/workflows/sibling-pin-check.yml` (117 lines @ `cc9f9dc2`) asserts that every `cargo install --git … --tag <tag> <pkg>` line in `.github/workflows/*.yml` matches the canonical pin in `scripts/install.sh`'s `component_info` table. Verified live: the gate's bash logic runs clean against `HEAD` (all sibling-pin lines OK).

Two gaps, both verified empirically:

1. **The scan set is workflow-only.** `sibling-pin-check.yml:82` loops `for wf in .github/workflows/*.yml`. It is **blind to `docs/manual/src/**` and `docs/quickstart/src/**` prose install commands.** The single live prose sibling-pin command is:
   - `docs/manual/src/40-cli-reference/44-mk-cli.md:12` —
     `` `cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.10.2 --bin mk` ``
   (md/ms chapters carry no prose install pin; `21-install.md` in both manual and quickstart carry only **tagless** `cargo install --git` illustrative references — no `--tag`, so no pin to drift. No `docs/quickstart/src/**` prose install command exists today at all.) This line drifted silently across two cycles historically (`mk-cli-v0.6.0` → never bumped at the v0.6.1/v0.7.0 re-pins) per the FOLLOWUP record.

2. **Even pointed at the prose file, the current detector regex does NOT match it.** Verified:
   - Detector @ `sibling-pin-check.yml:109`: `grep -nE 'cargo install --git +https?://[^ ]+ +--tag +[^ ]+ +[a-z][a-z0-9_-]*'` — requires a **bareword pkg** after `--tag <tag>`.
   - Pkg-extraction @ `:92-93`: `grep -oE -- '--tag +[^ ]+ +[a-z][a-z0-9_-]*'` then `awk '{print $3}'`.
   - The prose line uses `--bin mk` after the tag (workflows use bareword pkg `mk-cli`). `--bin` fails the trailing bareword charset of `[a-z][a-z0-9_-]*` because of the leading `--`. **Empirically confirmed: the current regex returns NO MATCH on the prose line.**

So b2 is **not** a one-line `for`-glob addition; the extractor must also learn the `--bin <bin>` form. The chosen fix — **url-keying** — handles both forms uniformly and is more robust than a `--bin mk → mk-cli` alias map.

`docs/manual/tests/lint.sh` does **not** check prose pins either (its steps are markdownlint / cspell / lychee / flag-coverage / glossary / index), so this gate is the only place to close the class.

---

## 2 — Design (exact)

Edit `.github/workflows/sibling-pin-check.yml` in place (single file; the gate stays one job, one bash step). Four logical changes:

### 2.1 Re-key the canonical table on repo-url (was: pkg)

**Current** (`:60-62`):
```sh
CANONICAL=$(grep -oE 'echo "[a-z-]+\|https://[^"]+\|[^"|]+\|[^"|]*\|[^"]*"' "$INSTALL_SH" \
  | sed -e 's/^echo "//' -e 's/"$//' \
  | awk -F'|' 'BEGIN{OFS="|"} $1 != "mnemonic-toolkit" {print $1, $3}')
```
emits `pkg|tag`. **Change the trailing `print $1, $3` → `print $2, $3`** so the table is `url|tag`. Keep the `$1 != "mnemonic-toolkit"` self-pin exclusion (still keyed on pkg-name in field 1 of the source line — that is correct; only the *output* columns change).

> **Consequence to document in-comment (load-bearing):** the GUI arm (`mnemonic-gui|…|mnemonic-gui-v0.49.0`) is and was already in the parsed table (the awk excludes only `mnemonic-toolkit`). Under pkg-keying a GUI install line would have hit the "unknown sibling" warning unless its pkg matched; under url-keying the GUI **url** now resolves. This is **neutral for b2** (no scanned file contains a GUI `cargo install` line — verified) and is a latent enablement for a future b3, not a behavior change today. Note this in the workflow comment so a future reader does not mistake it for scope-creep.

### 2.2 Replace the pkg lookup helper with a url lookup helper

**Current** (`:74-77`):
```sh
canonical_tag_for() {
  local pkg="$1"
  echo "$CANONICAL" | awk -F'|' -v p="$pkg" '$1 == p {print $2; exit}'
}
```
**Replace with** (drop `local` — the gate runs under `sh`-style `set -eu` in `bash`; `local` is fine in bash but the original used it; keep style consistent — either is acceptable, `local` retained below):
```sh
canonical_tag_for_url() {
  echo "$CANONICAL" | awk -F'|' -v u="$1" '$1 == u {print $2; exit}'
}
```

### 2.3 Build a scan set = workflows (minus self) + manual/quickstart prose

**Current** scan is inline `for wf in .github/workflows/*.yml` at `:82`. Replace the loop **header** so it iterates a precomputed `$SCAN_FILES`:

```sh
# Build the scan set: every workflow YAML (except this gate itself)
# + every manual/quickstart prose markdown file. Only the src/ trees
# are scanned (build output is git-ignored). Paths have no spaces in
# this repo, so word-splitting a space-joined list is safe here.
SCAN_FILES=""
for wf in .github/workflows/*.yml; do
  case "$(basename "$wf")" in
    sibling-pin-check.yml) continue ;;
  esac
  SCAN_FILES="$SCAN_FILES $wf"
done
for d in docs/manual/src docs/quickstart/src; do
  [ -d "$d" ] || continue
  mds=$(find "$d" -type f -name '*.md')
  SCAN_FILES="$SCAN_FILES $mds"
done
```

> **IMPLEMENTER GOTCHA (verified — this WILL bite):** do **not** collect the `find` output via a flush-left heredoc (`<<EOF` whose body line `$(find …)` starts at column 0). actionlint's YAML parser parses the `run: |` block scalar and **fails with `could not parse as YAML: could not find expected ':'`** on a column-0 line inside the scalar. Use the `mds=$(find …)` capture above (kept indented). This was caught by running actionlint on the draft; the `mds=$(...)` form is actionlint-clean. The `docs/…/src/**` trees contain no spaces in any path (verified), so the unquoted word-split of `$SCAN_FILES` / `$mds` is safe; shellcheck (via actionlint) does not warn on it here.

### 2.4 Generalize the per-line extractor to url-key + skip the `[ -f ]` guard

Replace the scan body (`:82-110`). The loop now iterates `$SCAN_FILES`, guards each path with `[ -f "$f" ]`, extracts `--git <url>` and `--tag <tag>` (NOT a trailing bareword), and looks up canonical by url. The detector `grep` drops the trailing `[a-z][a-z0-9_-]*` so it matches BOTH the workflow `--tag <tag> <pkg>` form AND the prose `--tag <tag> --bin <bin>` form, while still requiring `--git <url>` **and** `--tag <tag>` (so tagless illustrative prose never matches):

```sh
FAIL=0
for f in $SCAN_FILES; do
  [ -f "$f" ] || continue
  while IFS=: read -r lineno line; do
    url=$(echo "$line" | grep -oE -- '--git +https?://[^ ]+' | awk '{print $2}')
    tag=$(echo "$line" | grep -oE -- '--tag +[^ ]+' | awk '{print $2}')
    if [ -z "$url" ] || [ -z "$tag" ]; then
      echo "::warning::sibling-pin-check: $f:$lineno: could not extract url/tag from cargo-install line — skipped"
      continue
    fi
    canonical=$(canonical_tag_for_url "$url")
    if [ -z "$canonical" ]; then
      echo "::warning::sibling-pin-check: $f:$lineno: unknown repo-url '$url'; add a component_info arm in $INSTALL_SH to gate this pin"
      continue
    fi
    if [ "$tag" != "$canonical" ]; then
      echo "::error::sibling-pin-check: $f:$lineno: pin '$tag' (url $url) does not match $INSTALL_SH canonical '$canonical'"
      FAIL=1
    else
      echo "  OK $f:$lineno: $tag"
    fi
  done < <(grep -nE 'cargo install --git +https?://[^ ]+ +--tag +[^ ]+' "$f" || true)
done
```

### 2.5 Comment/header updates (mandatory, for the audit trail)

- Update the top-of-file comment block (`:3-25`) so the "Scope" paragraph names the **new prose scan set** (`docs/manual/src/**`, `docs/quickstart/src/**`) and states matching is **url-keyed** (handles `--tag <tag> <pkg>` and `--tag <tag> --bin <bin>` forms without a bin↔pkg map). Add a one-line provenance: `# Prose-scan extension (FOLLOWUP sibling-pin-check-skips-manual-prose-install-commands) added 2026-06-23.` Keep the existing Spec/R0 citation lines and add this spec's filename.
- Update the inline `CANONICAL=` comment (`:51-59`) to say the table is now `url|tag` and note the GUI-arm-in-table consequence (§2.1).
- Update the recovery message comment (`:22-25`) to say "the drifted **file**'s `--tag`" (was "workflow file's").

A complete actionlint-clean reference rendering of the full edited file is at
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-toolkit/54f17c66-ca51-48cd-9ac4-7f35cc2ba947/scratchpad/sibling-pin-check.proposed.yml`
(scratch — the implementer should reproduce the edit in-tree, not copy the scratch path; it is provided as the proven target shape).

---

## 3 — Exact files

**Modified (1, the gate):**
- `/scratch/code/shibboleth/mnemonic-toolkit/.github/workflows/sibling-pin-check.yml` — the four changes in §2.1–§2.5.

**Modified (FOLLOWUPS, status flips — same commit):**
- `/scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md`:
  - **b2 — FLIP.** Entry `sibling-pin-check-skips-manual-prose-install-commands` (header @ `:4047`; `Status:` line @ `:4054`). The status line currently reads:
    `- **Status:** \`open\` (gate gap unaddressed) — but the live stale instance is fixed: Wave-3 bumped … prose \`--tag mk-cli-v0.7.0\` → \`mk-cli-v0.10.1\` …. The CLASS fix (extend \`sibling-pin-check.yml\` to scan \`docs/manual/src/**\` + quickstart prose) is LANE-PIN-B, still OPEN.`
    Flip to `resolved` and append a `**Resolution (2026-06-23):**` clause: extended `sibling-pin-check.yml` to scan `docs/manual/src/**` + `docs/quickstart/src/**` prose, re-keyed canonical lookup on `--git` repo-url to handle the `--bin` prose form; trial-runs verified clean-tree exit 0 (now covers `44-mk-cli.md:12`) + synthetic prose-pin drift → `::error::` + exit 1; actionlint clean; CI-only NO-BUMP. (Note: the body text still says the live prose pin is `mk-cli-v0.10.1`; the live file is `mk-cli-v0.10.2` — optionally correct this stale text in the same edit, but it is cosmetic and not load-bearing.)
  - **b1 — NO STATUS CHANGE (flip-only confirm).** Entry `manual-yml-sibling-pin-vs-install-sh-drift-gate` (header @ `:573`; `Status: resolved` @ `:579`) is already resolved + GREEN. Do not edit its status. (Confirm-only; mention in the commit message.)

**NOT modified:**
- `scripts/install.sh` — it IS the canonical source; untouched. (Verified table @ `:29-50`: md-cli `descriptor-mnemonic-md-cli-v0.7.1`, ms-cli `ms-cli-v0.11.0`, mk-cli `mk-cli-v0.10.2`, mnemonic-gui `mnemonic-gui-v0.49.0`, toolkit-self `mnemonic-toolkit-v0.71.0`.)
- `docs/manual/src/40-cli-reference/44-mk-cli.md` — already correct (`mk-cli-v0.10.2` == canonical); the gate scans it, does not edit it. **Do NOT bump any sibling pin in this lane** (see §5 cascade — a pin bump would trip forward-only flag-coverage).
- b3 entry `install-sh-gui-sibling-pin-staleness-ungated` (`:278`) — deferred; untouched.

**Spec / review artifacts (added):**
- `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_sibling_pin_check_prose_scan_extension.md` — this spec.
- `/scratch/code/shibboleth/mnemonic-toolkit/design/agent-reports/sibling-pin-check-prose-scan-R0-review.md` — R0 reviewer transcript (persist verbatim BEFORE the fold-and-commit, per CLAUDE.md).

---

## 4 — Test / verify surface (no unit tests — it's a workflow; the "tests" are the trial-runs)

The gate has no Rust unit test (it's a CI bash step). The TDD discipline here is: **write the trial-run assertions first, then make them pass.** All three have been run live and pass against `HEAD`; the implementer must re-run them on their edited in-tree file before push.

**To execute the edited `run:` body standalone**, extract it (strip the 10-space YAML indent) and run under bash from the repo root:
```sh
cd /scratch/code/shibboleth/mnemonic-toolkit
awk '/^      - name: Verify sibling-CLI pins/{f=1} f&&/^          /{sub(/^          /,""); print}' \
  .github/workflows/sibling-pin-check.yml > /tmp/run_body.sh
```

**Trial 1 — clean tree (MUST exit 0; MUST now include the prose line):**
```sh
bash /tmp/run_body.sh; echo "[exit=$?]"
```
Assert: exit 0, and the output contains `OK docs/manual/src/40-cli-reference/44-mk-cli.md:12: mk-cli-v0.10.2` (the previously-blind prose line). It will also show the pre-existing workflow OKs (`cross-tool-differential.yml:50`, `manual.yml:79/86/90`, `quickstart.yml:73`). Proven live: exit 0, prose line present.

**Trial 2 — synthetic prose drift (MUST exit 1 with `::error::`):**
```sh
PROSE=docs/manual/src/40-cli-reference/44-mk-cli.md
cp "$PROSE" /tmp/prose.bak
sed -i 's/mk-cli-v0.10.2 --bin mk/mk-cli-v0.9.0 --bin mk/' "$PROSE"
bash /tmp/run_body.sh; echo "[exit=$?]"     # expect ::error:: on 44-mk-cli.md:12 + [exit=1]
cp /tmp/prose.bak "$PROSE"                  # RESTORE — verify `git status --short "$PROSE"` is clean
```
Assert: exit 1, and a line `::error::sibling-pin-check: docs/manual/src/40-cli-reference/44-mk-cli.md:12: pin 'mk-cli-v0.9.0' (url https://github.com/bg002h/mnemonic-key) does not match scripts/install.sh canonical 'mk-cli-v0.10.2'`. Then restore and confirm a clean working tree. Proven live: exit 1 with that exact `::error::`, tree restored clean.

**Trial 3 — tagless prose NOT flagged (no false positive):**
Confirm `docs/manual/src/20-quickstart/21-install.md` and `docs/quickstart/src/20-singlesig/21-install.md` (tagless `cargo install --git` mentions) and `44-mk-cli.md:393` (`cargo install`-style, no `--git`/`--tag`) do not appear in the gate output. Proven live: the detector `grep` returns no match on tagless lines.

**Lint gate (MANDATORY before push):**
```sh
actionlint .github/workflows/sibling-pin-check.yml
```
Assert: clean (exit 0, no output). actionlint runs embedded shellcheck on the `run:` step. Proven live on the proposed file: CLEAN (actionlint 1.7.12). **Do not skip — the heredoc gotcha (§2.3) is exactly the class of bug this catches.**

---

## 5 — `ci_gates_to_verify` (each gate this change re-fires, with HOW)

The edit touches **only** `.github/workflows/sibling-pin-check.yml` + `design/FOLLOWUPS.md`. Per-gate cascade (all `paths:` filters re-grepped live):

1. **`sibling-pin-check.yml` (THE edited gate) — RE-FIRES** (on every push/PR; `on: push` has no paths filter, `:27-32`). **HOW to verify:** after push, the Actions run for this workflow must be GREEN and its log must list `OK docs/manual/src/40-cli-reference/44-mk-cli.md:12: mk-cli-v0.10.2` plus the pre-existing workflow OKs. Locally pre-verified by Trial 1 (exit 0). Self-exclusion of `sibling-pin-check.yml` is retained (`:83-85` equivalent).

2. **`manual.yml` (forward-only md-pin flag-coverage cascade) — DOES NOT FIRE.** Its `paths:` filter is `docs/manual/**`, `docs/tools/render-mermaid-cache.py`, `.github/workflows/manual.yml` (verified `manual.yml:10-21`). This lane edits **neither** `docs/manual/**` (the gate scans `44-mk-cli.md` but does not edit it) **nor** `manual.yml`. So `manual.yml` does not re-fire. **HOW to confirm no accidental trip:** ensure the diff touches only `sibling-pin-check.yml` + `FOLLOWUPS.md` (`git diff --name-only` must show exactly those two). **CRITICAL constraint:** do NOT bump any sibling pin in `44-mk-cli.md` — a bump to a newer CLI with added `--help` flags would trip `manual.yml`'s forward-only flag-coverage RED (the known md-pin cascade). This lane is pin-neutral by design.

3. **`quickstart.yml` — DOES NOT FIRE.** `paths:` = `docs/quickstart/**`, `docs/manual/.markdownlint-cli2.jsonc`, `docs/manual/pandoc/filters/**`, `docs/tools/render-mermaid-cache.py`, `.github/workflows/quickstart.yml` (verified `quickstart.yml:10-22`). This lane edits none of those. (The gate *scans* `docs/quickstart/src/**` but there are no prose pins there today; the scan-set add is forward-looking.) `quickstart.yml:73`'s own `mk-cli-v0.10.2 mk-cli` pin is covered by the edited gate (Trial 1 shows it OK).

4. **`install-pin-check.yml` — DOES NOT FIRE.** Tag-only (`mnemonic-toolkit-v*`); no tag is pushed in a CI-only commit. Self-pin already current.

5. **`changelog-check.yml` — DOES NOT FIRE.** Tag-only (`mnemonic-toolkit-v*`, verified `:17-20`). Confirms NO-BUMP: no CHANGELOG entry required.

6. **`cross-tool-differential.yml` — DOES NOT FIRE.** `paths:` = `parse_descriptor.rs` + the differential test + its own workflow (per recon). This lane touches none. Its `:50` md-cli pin (`descriptor-mnemonic-md-cli-v0.7.1`) is itself covered by the edited gate (Trial 1 shows it OK) and is unchanged → funds-oracle Match verdict unaffected.

7. **`rust.yml`, `fuzz-smoke.yml`, `bitcoind-differential.yml`, `technical-manual.yml`, `manual-gui.yml` — DO NOT FIRE.** None have `paths:` matching `sibling-pin-check.yml` or `FOLLOWUPS.md` (src/Cargo, docs/tech**, docs/manual-gui/** scoped respectively). No `crates/**/src` edit ⇒ no Rust build/test/clippy re-fire.

**g6 mlock fmt-anchor:** untouched — no `src` edit, no `cargo fmt`. (Standing rule: NEVER `cargo fmt` mlock.rs. Not engaged here.)

**No GUI / manual flag-coverage lockstep** required: no clap flag/subcommand/dropdown change, no `--json` wire-shape change. (`schema_mirror` is flag-NAME parity only; `manual-cli-surface-mirror` is flag-table parity — both unaffected.)

---

## 6 — Ship mechanism

CI-only commit to `master` (or short-lived branch → FF). No tag. Stage paths **explicitly** (no `git add -A`):
```
git add .github/workflows/sibling-pin-check.yml \
        design/FOLLOWUPS.md \
        design/SPEC_sibling_pin_check_prose_scan_extension.md \
        design/agent-reports/sibling-pin-check-prose-scan-R0-review.md
```
Before push: run all three trial-runs (§4) + `actionlint` (§4 lint gate) and confirm a clean working tree after the synthetic-drift restore. No publish, no sibling-repo PR.

**R0 gate (mandatory, pre-impl):** this is a spec-shaped CI change — dispatch this spec to an opus architect for R0 and converge to **0 Critical / 0 Important** BEFORE editing the workflow (the parent b1 gate went through `SPEC_sibling_pin_drift_gate.md` + an R0 review; b2 as its direct extension follows the same gate). Persist the review verbatim to `design/agent-reports/sibling-pin-check-prose-scan-R0-review.md` before the fold-and-commit. After implementation, a whole-diff adversarial execution review per the post-impl rule.

---

## 7 — SemVer & FOLLOWUP flips (summary)

- **SemVer:** NO-BUMP (CI-only).
- **FOLLOWUP flips in the shipping commit:**
  - `sibling-pin-check-skips-manual-prose-install-commands` (b2): `open` → `resolved` (with the 2026-06-23 resolution clause). `FOLLOWUPS.md:4054`.
  - `manual-yml-sibling-pin-vs-install-sh-drift-gate` (b1): already `resolved` — **no edit**, confirm-only in the commit message.
  - `install-sh-gui-sibling-pin-staleness-ungated` (b3): **DEFERRED** — no edit.

---

## 8 — Reviewer-loop disposition

Dispatched to opus architect for R0 BEFORE implementation per CLAUDE.md mandatory pre-impl R0 gate. R0 must converge to 0C/0I before any workflow YAML is written. Reviewer-loop continues after every fold (re-dispatch after folding findings; folds can introduce drift).
