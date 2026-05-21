# manual-v0.2.1 Implementation Plan (Cycle 2 / Wave 1 first)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote `MD_BIN` and `MS_BIN` from `true` placeholder to real `md` and `ms` binaries in `.github/workflows/manual.yml`, closing both successor FOLLOWUPs of the partial-resolution of `manual-yml-bind-real-mnemonic-bin`. Tag `manual-v0.2.1`.

**Architecture:** Pure CI-yml edit. No toolkit src changes. No test cell changes. Two new `cargo install --git --tag` steps mirror the existing mk-cli install at `manual.yml:72-77`. The Audit manual step's `MD_BIN`/`MS_BIN` argv values flip from `true` to `md`/`ms`. Opportunistically reconcile the pre-existing mk-cli pin staleness (`manual.yml:77` pins `mk-cli-v0.2.0`; `scripts/install.sh:42` pins `mk-cli-v0.4.1`). Plus FOLLOWUPS Status flips in the same commit.

**Tech Stack:** GitHub Actions YAML; `actionlint` for structural validation; `cargo install --git <url> --tag <tag> <pkg>` for sibling-CLI installation.

**Brainstorm spec:** `design/BRAINSTORM_followups_abc_release_plan.md` § "Cycle 2 — manual-v0.2.1 (B) — Wave 1 first".

**Source SHA at plan-write time:** `2080d14`.

---

## File structure

- **Modify:** `.github/workflows/manual.yml`
  - L77 — bump mk-cli pin tag (incidental cross-pin reconcile)
  - L77+ — insert new `cargo install` step for md-cli (mirror mk-cli pattern)
  - L77+ — insert new `cargo install` step for ms-cli
  - L85-96 — flip `MD_BIN=true` → `MD_BIN=md` and `MS_BIN=true` → `MS_BIN=ms` in the Audit manual step
- **Modify:** `design/FOLLOWUPS.md`
  - ~L2682-2700 — flip Status fields for `manual-md-bin-real-binary-promote` and `manual-ms-bin-real-binary-promote`

That's the entire file scope. No new files.

---

## Tasks

### Task 1: Recon — confirm sibling-CLI tags

**Files:** none modified (read-only).

- [ ] **Step 1: Confirm latest sibling-CLI tag pins in `scripts/install.sh`**

Run:
```bash
grep -nE 'descriptor-mnemonic-md-cli-v|ms-cli-v|mk-cli-v' /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh
```

Expected:
- `scripts/install.sh:35` — `descriptor-mnemonic-md-cli-v0.6.0`
- `scripts/install.sh:38` — `ms-cli-v0.4.0`
- `scripts/install.sh:42` — `mk-cli-v0.4.1`

If any pin has advanced since plan-write SHA `2080d14`, use the newer pin in the manual.yml edits below.

- [ ] **Step 2: Capture current `manual.yml` mk-cli install pin for comparison**

Run:
```bash
grep -n 'mk-cli-v' /scratch/code/shibboleth/mnemonic-toolkit/.github/workflows/manual.yml
```

Expected: `manual.yml:77` — `--tag mk-cli-v0.2.0`. Confirms the pre-existing cross-pin staleness.

---

### Task 2: Add md-cli install step + flip MD_BIN

**Files:**
- Modify: `.github/workflows/manual.yml` (insert after the existing mk-cli install step at L72-77; flip MD_BIN in the Audit manual step at L85-96)

- [ ] **Step 1: Read the current mk-cli install step + Audit manual step**

Run:
```bash
sed -n '70,100p' /scratch/code/shibboleth/mnemonic-toolkit/.github/workflows/manual.yml
```

Confirm current structure: "Install mk-cli" step at ~L72-77 followed by "Build mnemonic binary" step then "Audit manual" step.

- [ ] **Step 2: Insert "Install md-cli" step between the existing mk-cli install step and the Build mnemonic binary step**

Insert this block immediately after the existing mk-cli install step (i.e., after `cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.4.1 mk-cli` line, replacing the old v0.2.0):

```yaml
      - name: Install md-cli
        # Mirrors the mk-cli install pattern above. Use package-name selector
        # (`md-cli`) instead of `--bin md`: the descriptor-mnemonic workspace
        # ships multiple binaries.
        run: cargo install --git https://github.com/bg002h/descriptor-mnemonic --tag descriptor-mnemonic-md-cli-v0.6.0 md-cli --features cli-compiler
```

(Note: the `--features cli-compiler` flag mirrors `scripts/install.sh:35`'s feature flag for md-cli. Verify by running `grep 'descriptor-mnemonic-md-cli' /scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh` and matching the `<features>` field of `component_info`'s pipe-separated record.)

- [ ] **Step 3: Bump the existing mk-cli install pin in the same edit**

Replace the mk-cli install line:

Old:
```yaml
        run: cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.2.0 mk-cli
```

New:
```yaml
        run: cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.4.1 mk-cli
```

- [ ] **Step 4: Flip `MD_BIN=true` → `MD_BIN=md` in the Audit manual step**

In the "Audit manual" step body (around L85-96), find the `make audit` invocation and change the `MD_BIN` line:

Old:
```yaml
            MD_BIN=true \
```

New:
```yaml
            MD_BIN=md \
```

- [ ] **Step 5: Verify the edit applied cleanly**

Run:
```bash
sed -n '70,100p' /scratch/code/shibboleth/mnemonic-toolkit/.github/workflows/manual.yml
```

Confirm: "Install mk-cli" with `mk-cli-v0.4.1`, then "Install md-cli" with `descriptor-mnemonic-md-cli-v0.6.0`, then "Build mnemonic binary", then "Audit manual" with `MD_BIN=md`.

---

### Task 3: Add ms-cli install step + flip MS_BIN

**Files:**
- Modify: `.github/workflows/manual.yml` (insert after the new md-cli step; flip MS_BIN in the Audit manual step)

- [ ] **Step 1: Insert "Install ms-cli" step immediately after the new "Install md-cli" step**

```yaml
      - name: Install ms-cli
        # Mirrors the mk-cli + md-cli install patterns above.
        run: cargo install --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.4.0 ms-cli
```

- [ ] **Step 2: Flip `MS_BIN=true` → `MS_BIN=ms` in the Audit manual step**

Old:
```yaml
            MS_BIN=true \
```

New:
```yaml
            MS_BIN=ms \
```

- [ ] **Step 3: Verify the full Audit manual step body post-edits**

Run:
```bash
sed -n '85,100p' /scratch/code/shibboleth/mnemonic-toolkit/.github/workflows/manual.yml
```

Expected (final form):
```yaml
      - name: Audit manual (lint + verify-examples with real mnemonic binary)
        working-directory: docs/manual
        # All flag values below are constants or controlled built-ins
        # ($GITHUB_WORKSPACE); no untrusted github.event.* fields per
        # https://github.blog/security/vulnerability-research/how-to-catch-github-actions-workflow-injections-before-attackers-do/.
        run: |
          make audit \
            MNEMONIC_BIN="$GITHUB_WORKSPACE/target/debug/mnemonic" \
            MD_BIN=md \
            MS_BIN=ms \
            MK_BIN=mk \
            FIXTURES_DIR="$GITHUB_WORKSPACE/crates/mnemonic-toolkit/tests/fixtures/wallet_import"
```

---

### Task 4: Local validation — run `make audit` with real binaries

**Files:** none modified. This is a validation step.

- [ ] **Step 1: Install the sibling CLIs locally to mirror the CI environment**

Run:
```bash
cargo install --git https://github.com/bg002h/descriptor-mnemonic --tag descriptor-mnemonic-md-cli-v0.6.0 md-cli --features cli-compiler
cargo install --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.4.0 ms-cli
```

(mk-cli should already be installed from prior work; verify via `which mk` returns a `$HOME/.cargo/bin/mk` path.)

- [ ] **Step 2: Run `make audit` with the real binaries**

Run:
```bash
make -C /scratch/code/shibboleth/mnemonic-toolkit/docs/manual audit \
  MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic \
  MD_BIN=md \
  MS_BIN=ms \
  MK_BIN=mk \
  FIXTURES_DIR=/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/fixtures/wallet_import
```

Expected: `[lint] OK` + `[verify-examples] OK (10 transcripts pass; 4 skipped per SKIP_STEMS)`.

**IMPORTANT:** the flag-coverage gate in `docs/manual/tests/lint.sh` may now surface NEW warnings for chapters 42 (md) and 43 (ms) that were previously masked by `MD_BIN=true`/`MS_BIN=true` no-op short-circuits. These warnings are EXPECTED and are NOT this cycle's scope to fix — they become findings for Cycle 4 (manual-v0.3.0). If the lint EXITS non-zero (i.e., warnings escalate to errors), that's a hard finding that must be triaged before commit; if it warns but exits 0, proceed.

- [ ] **Step 3: Capture any new warnings for the Cycle 4 backlog**

Run:
```bash
make -C /scratch/code/shibboleth/mnemonic-toolkit/docs/manual audit \
  MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic \
  MD_BIN=md MS_BIN=ms MK_BIN=mk \
  FIXTURES_DIR=/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/fixtures/wallet_import \
  2>&1 | grep -E '(WARN|warning)' > /tmp/manual-v0.2.1-new-warnings.txt
cat /tmp/manual-v0.2.1-new-warnings.txt
```

Save the output for later reference in Cycle 4's P0 recon (paste into `design/PLAN_manual_v0_3_0.md` when it's written).

---

### Task 5: actionlint

**Files:** none modified.

- [ ] **Step 1: Run actionlint on the modified workflow**

Run:
```bash
actionlint /scratch/code/shibboleth/mnemonic-toolkit/.github/workflows/manual.yml
```

Expected: zero output (actionlint silent on success).

If actionlint reports errors, fix and re-run. Common causes: indentation drift on the inserted step blocks; missing colons.

---

### Task 6: Sonnet reviewer fold-verify

**Files:** none modified. This is an architect-review dispatch.

- [ ] **Step 1: Dispatch sonnet via Agent tool**

Use the `Agent` tool with:
- `subagent_type: feature-dev:code-reviewer`
- `model: sonnet`
- Prompt that asks the reviewer to verify:
  1. Both new install steps (md-cli + ms-cli) exist with correct tag pins matching `scripts/install.sh:35` + `:38`.
  2. mk-cli install pin matches `scripts/install.sh:42` (post-bump).
  3. `MD_BIN=md` and `MS_BIN=ms` (NOT `=true`) in the Audit manual step.
  4. actionlint clean (architect runs `actionlint .github/workflows/manual.yml`).
  5. No untrusted github.event.* fields used in any new run: body.

Gate: 0 critical / 0 important to proceed.

- [ ] **Step 2: Fold any Important findings inline**

If the reviewer finds Important issues (indentation, missed mk-cli pin, etc.), fix them and re-dispatch a confirmation round. Loop until 0 Important.

---

### Task 7: Flip FOLLOWUPS Status

**Files:**
- Modify: `design/FOLLOWUPS.md` (entries `manual-md-bin-real-binary-promote` + `manual-ms-bin-real-binary-promote`)

- [ ] **Step 1: Locate the two FOLLOWUP entries**

Run:
```bash
grep -n '^### `manual-md-bin-real-binary-promote\|^### `manual-ms-bin-real-binary-promote' /scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md
```

Expected: two line numbers (approximately L2682 and L2693 per plan-write SHA).

- [ ] **Step 2: Flip the Status field for `manual-md-bin-real-binary-promote`**

In the entry's `- **Status:**` line:

Old:
```markdown
- **Status:** `open`
```

New (where `<commit-sha>` is the commit-SHA created in Task 8 — backfill after Task 8 lands):
```markdown
- **Status:** `resolved <commit-sha>` — manual-v0.2.1 cycle landed real `md` binary install step in `manual.yml` mirroring the mk-cli pattern at L72-77; flag-coverage gate now exercises `md <subcommand> --help` against the cargo-installed binary.
```

- [ ] **Step 3: Flip the Status field for `manual-ms-bin-real-binary-promote`**

Same pattern as Step 2; replace `md` with `ms` in the resolved note.

(Both Status flips happen post-commit; the commit-SHA is known only after Task 8. The convention is: do Task 8 first WITHOUT the Status flips, then amend with a second commit OR fold into a single commit via `git add -p` staging the Status flips alongside the workflow edits. The latter is cleaner — single commit. To do that: skip Task 7 until Task 8's `git add` step, then add the FOLLOWUPS.md edits to the same commit.)

**Recommended:** defer this task's edits until Task 8's staging step; backfill the SHA placeholder via `git commit --amend` if the SHA wasn't known at write time.

---

### Task 8: Commit + tag + push

**Files:** all modified files staged.

- [ ] **Step 1: Verify the working tree state**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git status --short
```

Expected modified files:
- `.github/workflows/manual.yml`
- `design/FOLLOWUPS.md` (from Task 7)

No untracked files (other than `.claude/` which is gitignored per session).

- [ ] **Step 2: Stage explicit paths (no `git add -A`)**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git add .github/workflows/manual.yml design/FOLLOWUPS.md
git diff --cached --stat
```

Expected: 2 files changed; ~20-30 line insertions; minimal deletions.

- [ ] **Step 3: Commit**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git commit -m "$(cat <<'EOF'
release(manual): manual-v0.2.1 — promote MD_BIN + MS_BIN to real binaries

Closes the 2 successor FOLLOWUPs of the manual-v0.2.0 cycle's partial
resolution of manual-yml-bind-real-mnemonic-bin:

- manual-md-bin-real-binary-promote — add `cargo install --git
  descriptor-mnemonic-md-cli-v0.6.0 md-cli --features cli-compiler`
  step to manual.yml; flip MD_BIN=true → MD_BIN=md in the Audit
  manual step.
- manual-ms-bin-real-binary-promote — add `cargo install --git
  ms-cli-v0.4.0 ms-cli` step; flip MS_BIN=true → MS_BIN=ms.

Plus incidental: bump existing mk-cli install pin from mk-cli-v0.2.0
→ mk-cli-v0.4.1 to match scripts/install.sh:42 (cross-pin staleness
surfaced in cycle-prep recon §4).

The flag-coverage gate at docs/manual/tests/lint.sh now exercises
`md <subcommand> --help` and `ms <subcommand> --help` against the
cargo-installed binaries, instead of short-circuiting via `true ...
--help` no-op. Expect new warnings for chapter 42 (md) and chapter 43
(ms) prose drift to surface in CI; these are scope for Cycle 4
(manual-v0.3.0) per design/BRAINSTORM_followups_abc_release_plan.md.

Cycle 2 of the A/B/C FOLLOWUP release plan; Wave 1 first ship.
Sonnet reviewer GREEN: 0 critical / 0 important.
actionlint clean.

Toolkit src: unchanged. No GUI lockstep.
EOF
)"
```

- [ ] **Step 4: Capture the commit SHA + backfill FOLLOWUPS Status if needed**

If the FOLLOWUPS Status flips were staged in Step 2, this step is a no-op. If they weren't, run:
```bash
SHA=$(git rev-parse HEAD)
echo "Commit SHA: $SHA"
# Edit design/FOLLOWUPS.md to insert $SHA into the two `resolved <commit-sha>` placeholders
# Then: git add design/FOLLOWUPS.md && git commit --amend --no-edit
```

- [ ] **Step 5: Tag manual-v0.2.1**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git tag manual-v0.2.1
git tag -l 'manual-v*'
```

Expected output:
```
manual-v0.1.10
manual-v0.2.0
manual-v0.2.1
```

- [ ] **Step 6: Push master + tag**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git push origin master
git push origin manual-v0.2.1
```

Expected output: both pushes show `[new branch]` or `<old-sha>..<new-sha>` for master, and `[new tag]` for the manual-v0.2.1 tag.

---

### Task 9: Monitor CI runs + GH Release

**Files:** none modified.

- [ ] **Step 1: Monitor CI runs triggered by the push**

Use the `Monitor` tool with a poll script that watches `gh run list`:

```bash
prev=""
while true; do
  s=$(gh run list --limit 4 --json databaseId,name,headBranch,status,conclusion 2>/dev/null || echo '[]')
  cur=$(jq -r '.[] | select(.headBranch == "master" or .headBranch == "manual-v0.2.1") | "\(.databaseId) \(.headBranch) \(.name): \(.status)/\(.conclusion // "-")"' <<<"$s" | sort)
  comm -13 <(echo "$prev") <(echo "$cur")
  prev=$cur
  remaining=$(jq -r '[.[] | select(.headBranch == "master" or .headBranch == "manual-v0.2.1") | select(.status != "completed")] | length' <<<"$s")
  [ "$remaining" = "0" ] && break
  sleep 30
done
```

Expected runs:
- `manual` on master — should PASS (builds PDF with real binaries; flag-coverage may emit warnings but exits 0)
- `manual` on `manual-v0.2.1` tag — should PASS (PDF GH Release asset uploaded)
- `rust` on master — should PASS (no toolkit src changes)

If `manual` workflow FAILS, the most likely cause is that the new flag-coverage warnings escalated to errors. Check the workflow log for the `[lint]` exit code. If hard-fail, revert the commit and triage in a follow-up.

- [ ] **Step 2: Create the manual-v0.2.1 GH Release manually**

(The manual.yml workflow auto-uploads the PDF asset; verify it's attached.)

Check:
```bash
gh release view manual-v0.2.1 --json tagName,assets,createdAt
```

If the release exists (auto-created by manual.yml), confirm asset `m-format-manual.pdf` is present.

If the release does NOT auto-create, run:
```bash
gh release create manual-v0.2.1 \
  --title 'manual-v0.2.1 — MD_BIN + MS_BIN binary promote' \
  --notes "$(cat <<'EOF'
Patch release: CI workflow `manual.yml` now installs real `md` and `ms` binaries via `cargo install --git --tag`, and the Audit manual step's `MD_BIN`/`MS_BIN` argv values flip from the `true` no-op placeholder to the real binary commands. Closes the 2 successor FOLLOWUPs of the v0.2.0 cycle's partial-close of `manual-yml-bind-real-mnemonic-bin`.

### Changes

- Add `cargo install --git descriptor-mnemonic-md-cli-v0.6.0 md-cli --features cli-compiler` step to `.github/workflows/manual.yml`.
- Add `cargo install --git ms-cli-v0.4.0 ms-cli` step.
- Bump existing mk-cli install pin from `mk-cli-v0.2.0` → `mk-cli-v0.4.1` to match `scripts/install.sh:42`.
- Flip `MD_BIN=true` → `MD_BIN=md` and `MS_BIN=true` → `MS_BIN=ms` in the Audit manual step.

### Downstream

- The flag-coverage gate at `docs/manual/tests/lint.sh` now exercises `md <subcommand> --help` and `ms <subcommand> --help` against real binaries. Any drift in chapters 42 (md) or 43 (ms) prose-vs-binary will surface as CI warnings. These warnings are scope for the upcoming `manual-v0.3.0` cycle.
- No toolkit src change. No GUI lockstep required.

### Cycle context

Cycle 2 of the A/B/C FOLLOWUP release plan (Wave 1 first ship). See `design/BRAINSTORM_followups_abc_release_plan.md` for the umbrella plan.
EOF
)"
```

---

## Self-review

After completing all 9 tasks, verify against the brainstorm spec:

1. **Spec coverage check:**
   - Cycle 2 Phase 0 (recon) → Task 1 ✓
   - Cycle 2 Phase 1 (manual.yml edits) → Tasks 2-3 ✓
   - Cycle 2 Phase 2 (local validation) → Task 4 ✓
   - Cycle 2 Phase 3 (actionlint + reviewer) → Tasks 5-6 ✓
   - Cycle 2 Phase 4 (commit + tag + push + GH Release) → Tasks 8-9 ✓
   - Cycle 2 Phase 5 (FOLLOWUPS Status flips) → Task 7 ✓ (folded into Task 8 commit)

2. **No-placeholder check:** All edits show actual lines. The Status flip in Task 7 has a `<commit-sha>` template placeholder, but Task 8 Step 4 backfills it before push. No TBD/TODO remains.

3. **Type consistency:** N/A (no Rust types; YAML edits only).

4. **Effort estimate sanity-check:** ~30 minutes total per brainstorm (Phase 0: ~5 min; Phase 1: ~5 min; Phase 2 local validation: ~5-10 min; Phase 3 reviewer: ~5 min; Phase 4 commit+tag+push+GH Release: ~10 min). Reasonable.

---

## Risk flags

- **New flag-coverage warnings might hard-fail CI.** If `make audit` exits non-zero due to escalated warnings on chapter-42/43 prose, the cycle is blocked. Triage at Task 4 Step 3 BEFORE proceeding to commit. Workaround if needed: temporarily ratchet down the gate to allow warnings (defer fix to Cycle 4); this would require a separate `manual-v0.2.2` patch.

- **Sibling-CLI tag pin drift.** If `scripts/install.sh` was updated between plan-write SHA `2080d14` and execution time, the tag pins in Task 1 may be stale. Always pull latest origin/master before starting + use the live `scripts/install.sh` values, not the plan-doc hardcoded ones.

- **Sub-skill expectations.** This plan assumes the executor uses `superpowers:subagent-driven-development` or `superpowers:executing-plans`. Direct manual execution is fine for a small cycle like this but loses the per-task review gate.
