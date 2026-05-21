# manual-v0.3.0 Implementation Plan (Cycle 4 / Wave 2 second)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Audit + refresh 9 manual chapters carrying pre-v0.15.0 wire-format card strings. Recapture 4 legacy transcripts (`22-first-bundle`, `23-verify`, `24-recover`, `24-recover-md1`) against the v0.28.x binary. Remove SKIP_STEMS from `verify-examples.sh`. Tag `manual-v0.3.0` + ship GH Release with refreshed PDF.

**Architecture:** Multi-chapter audit cycle in the style of manual-v0.2.0 (which audited 3 chapters in the same session). 9 chapters total = 3 quickstart (22/23/24) + 2 workflow (31/35) + 4 CLI-reference (41/42/43/44). The transcripts-as-audit infrastructure already exists from manual-v0.2.0 (`verify-examples.sh` with triple-format + `$FIXTURES_DIR` + `$MK_BIN`) + manual-v0.2.1 (real `md`/`ms` binaries in CI). This cycle does NOT extend CI; it consumes the existing wiring + recaptures + audits prose.

**Tech Stack:** Markdown (chapter prose); bash transcript replay (per-cmd `mktemp -d` cwd); `make audit` for full lint + verify-examples gate; opus dispatch for multi-chapter classification + end-of-cycle holistic review.

**Brainstorm spec:** `design/BRAINSTORM_followups_abc_release_plan.md` § "Cycle 4 — `manual-v0.3.0` (C) — Wave 2 second".

**Effort estimate: 3-5 days** (per architect I3 fold; v0.2.0 took multi-session for 3 chapters; 9 chapters ≈ 3× throughput).

**Source SHA at plan-write time:** `44fe753` (post-Wave-1 ship). At cycle-execution time, will be v0.28.4-or-later if Cycle 3 has shipped.

---

## File structure (anticipated; exact diff scope set by P1b classification)

**Recapture (4 transcripts):**

- `docs/manual/transcripts/22-first-bundle.{cmd,out}` — replay against v0.28.x mnemonic binary
- `docs/manual/transcripts/23-verify.{cmd,out}` — replay against v0.28.x; uses 22's emitted cards as input
- `docs/manual/transcripts/24-recover.{cmd,out}` — replay against v0.28.x
- `docs/manual/transcripts/24-recover-md1.{cmd,out}` — replay against v0.28.x md-cli binary

**Chapter prose audit + fixes (9 chapters):**

- Modify: `docs/manual/src/20-quickstart/22-first-bundle.md` — quickstart bundle worked example; updates per recaptured transcript
- Modify: `docs/manual/src/20-quickstart/23-verify.md` — verify-bundle worked example
- Modify: `docs/manual/src/20-quickstart/24-recover.md` — recover worked example
- Modify: `docs/manual/src/30-workflows/31-singlesig-steel.md` — singlesig steel-engraving workflow (card-string references)
- Modify: `docs/manual/src/30-workflows/35-recovery-paths.md` — recovery paths (card-string references)
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md` — mnemonic CLI reference (chapter-41 inheritance composite already in v0.2.0; check for non-inheritance drift)
- Modify: `docs/manual/src/40-cli-reference/42-md.md` — md CLI reference (binary now real in CI per v0.2.1; flag-coverage gate already passes)
- Modify: `docs/manual/src/40-cli-reference/43-ms.md` — ms CLI reference (same as 42)
- Modify: `docs/manual/src/40-cli-reference/44-mk-cli.md` — mk CLI reference

**Other:**

- Modify: `docs/manual/tests/verify-examples.sh` — remove SKIP_STEMS array (4 entries)
- Modify: `design/FOLLOWUPS.md` — `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh` Status flip
- Optional: `design/AUDIT_FINDINGS_manual_v0_3_0.md` — per-finding triage (parallel to v0.2.0's audit-findings doc)

---

## Tasks

### Task 1: Phase 0 — multi-chapter recon

**Files:** none modified (read-only).

- [ ] **Step 1: Sync state**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git fetch --quiet origin master
git -C $(git rev-parse --show-toplevel) status -sb
git log --oneline origin/master ^HEAD | head -5
target/debug/mnemonic --version
mk --version
ms --version
md --version 2>&1 | grep -v 'mkdir' | head -2
```

Capture: toolkit binary version (should be v0.28.3 or v0.28.4 depending on whether Cycle 3 has shipped); md/ms/mk versions (from Cycle 2 install).

- [ ] **Step 2: Grep stale card strings across all 9 chapters**

```bash
grep -lE 'ms10entrsq|mk1qprsqhp|md1zsxdsp' \
  /scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/20-quickstart/*.md \
  /scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/30-workflows/*.md \
  /scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/40-cli-reference/*.md
```

Expected: 9 files (3 quickstart + 2 workflow + 4 CLI-reference). If the count differs, update the plan scope before proceeding.

- [ ] **Step 3: For each of the 9 chapters, capture line numbers of stale-card-string mentions**

```bash
for f in docs/manual/src/20-quickstart/{22-first-bundle,23-verify,24-recover}.md \
         docs/manual/src/30-workflows/{31-singlesig-steel,35-recovery-paths}.md \
         docs/manual/src/40-cli-reference/{41-mnemonic,42-md,43-ms,44-mk-cli}.md; do
  echo "=== $f ==="
  grep -nE 'ms10entrsq|mk1qprsqhp|md1zsxdsp' "$f"
done > /tmp/manual-v0.3.0-recon.txt
wc -l /tmp/manual-v0.3.0-recon.txt
```

Save the output as input to the audit-findings doc.

- [ ] **Step 4: Amend the FOLLOWUP body's "9 OTHER" wording per architect I2 (folded inline in brainstorm)**

Find `design/FOLLOWUPS.md` entry `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh`. The body says "9 OTHER manual chapters" but parenthetically lists 6. Per the architect's I2 fold in the brainstorm spec, change to "9 total = 3 quickstart (22/23/24) + 6 cross-reference (31/35/41/42/43/44)".

Old (verbatim from manual-v0.2.0 commit `fe32e9e`):

```markdown
- **What:** Refresh both the transcript captures AND the chapter prose. The captures need rerunning against v0.28.x; the prose needs re-audit (claim verification) against the new captured output. Chapter scope: 22-first-bundle.md + 23-verify.md + 24-recover.md (3 chapters). Plus parallel grep + audit of the 9 OTHER manual chapters that mention the stale card strings (per `grep -l 'ms10entrsq\|mk1qprsqhp\|md1zsxdsp' docs/manual/src/**/*.md` — chapters 31/35/41/42/43/44 in addition to the 3 quickstart chapters).
```

New:

```markdown
- **What:** Refresh both the transcript captures AND the chapter prose. The captures need rerunning against v0.28.x; the prose needs re-audit (claim verification) against the new captured output. **Chapter scope (9 total)** = 3 quickstart (22-first-bundle, 23-verify, 24-recover) + 6 cross-reference chapters (31-singlesig-steel, 35-recovery-paths, 41-mnemonic, 42-md, 43-ms, 44-mk-cli) that mention the stale card strings (per `grep -l 'ms10entrsq\|mk1qprsqhp\|md1zsxdsp' docs/manual/src/**/*.md`).
```

(This edit lands in the cycle's first commit alongside Phase 1 recapture work.)

---

### Task 2: Phase 1 — recapture 4 transcripts

**Files:**
- Modify: `docs/manual/transcripts/22-first-bundle.{cmd,out}`
- Modify: `docs/manual/transcripts/23-verify.{cmd,out}`
- Modify: `docs/manual/transcripts/24-recover.{cmd,out}`
- Modify: `docs/manual/transcripts/24-recover-md1.{cmd,out}`

- [ ] **Step 1: Recapture 22-first-bundle.{cmd,out}**

```bash
BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic
OUT=/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/transcripts
TMP=$(mktemp -d); cd "$TMP"
CMD=$(cat "$OUT/22-first-bundle.cmd")
printf '%s\n' "$CMD" | sed "s,\\\$MNEMONIC_BIN,$BIN,g" > run.sh
bash run.sh > "$OUT/22-first-bundle.out" 2>&1
echo "exit: $?"
head -20 "$OUT/22-first-bundle.out"
cd / && rm -rf "$TMP"
```

The captured `.out` is the new content. Compare against the prior `.out` to understand the diff scope.

- [ ] **Step 2: Recapture 23-verify.{cmd,out}**

23-verify.cmd consumes 22-first-bundle.out's emitted card strings. Need to either:
- (a) Update 23-verify.cmd to embed the new (post-recapture) card strings from Step 1's output, OR
- (b) Capture 23-verify against the OLD card strings + accept the `result: mismatch` (which is current state, and would just preserve SKIP_STEMS — not the goal)

Option (a) is the right path: edit 23-verify.cmd to embed the new card strings, then recapture.

```bash
# Inspect 22-first-bundle.out for the new ms1/mk1/md1 strings
grep -E '^(ms1|mk1|md1)[a-z0-9]+' /scratch/code/shibboleth/mnemonic-toolkit/docs/manual/transcripts/22-first-bundle.out | head -10
```

Update 23-verify.cmd to use those new strings. Then recapture:

```bash
BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic
OUT=/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/transcripts
TMP=$(mktemp -d); cd "$TMP"
CMD=$(cat "$OUT/23-verify.cmd")
printf '%s\n' "$CMD" | sed "s,\\\$MNEMONIC_BIN,$BIN,g" > run.sh
bash run.sh > "$OUT/23-verify.out" 2>&1
echo "exit: $?"
head -20 "$OUT/23-verify.out"
cd / && rm -rf "$TMP"
```

Expected exit 0 + `result: ok` (post-recapture; was: `result: mismatch` due to pre-v0.15.0 wire-format).

- [ ] **Step 3: Recapture 24-recover.{cmd,out}**

Similar pattern. 24-recover.cmd uses one of the v0.28.x recovery flows. Just secret-warning lines should differ from the pre-recapture form.

- [ ] **Step 4: Recapture 24-recover-md1.{cmd,out}**

This one uses `$MD_BIN`. Since Cycle 2 / manual-v0.2.1 promoted MD_BIN to real `md`, the recapture should succeed:

```bash
BIN_MD=/home/bcg/.cargo/bin/md
OUT=/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/transcripts
TMP=$(mktemp -d); cd "$TMP"
CMD=$(cat "$OUT/24-recover-md1.cmd")
printf '%s\n' "$CMD" | sed "s,\\\$MD_BIN,$BIN_MD,g" > run.sh
bash run.sh > "$OUT/24-recover-md1.out" 2>&1
echo "exit: $?"
cat "$OUT/24-recover-md1.out"
cd / && rm -rf "$TMP"
```

Expected: a descriptor string output (not the empty stdout pre-recapture).

---

### Task 3: Phase 2 — audit chapter prose (run prose commands end-to-end)

**Files:** none modified yet (this task identifies findings; Phase 3b applies fixes).

- [ ] **Step 1: For each of the 9 chapters, extract documented commands + run them**

For each chapter file, identify every `sh` or `bash` fenced code block + every claim about toolkit binary output. For the 3 quickstart chapters + the 2 workflow chapters, the audit is per-recipe (similar to v0.2.0's chapter-39 audit). For the 4 CLI-reference chapters (41/42/43/44), the audit verifies that `--help` snapshots in the prose match the binary's actual `--help` output AND that any worked examples (recipe-style snippets) replay correctly.

This Step 1 is exploratory — the executor walks each chapter sequentially, captures findings into a per-chapter section of `design/AUDIT_FINDINGS_manual_v0_3_0.md`.

- [ ] **Step 2: Build the AUDIT_FINDINGS_manual_v0_3_0.md table**

Mirror the manual-v0.2.0 finding-table structure (`design/AUDIT_FINDINGS_manual_v0_28_0_content.md`):

```markdown
# AUDIT FINDINGS — manual-v0.3.0 cycle (9-chapter wire-format refresh)

**Phase:** P1a (mechanical transcript capture) + P1b (architect classification).
**Working SHA:** `<HEAD>`
**Binary under test:** `target/debug/mnemonic` (mnemonic <version>); `md` (<version>); `ms` (<version>); `mk` (<version>).

## Per-chapter status

| Chapter | Lines audited | Findings | Status |
|---|---|---|---|
| 22-first-bundle | N | <list> | <classification> |
| 23-verify | N | <list> | <classification> |
| 24-recover | N | <list> | <classification> |
| 31-singlesig-steel | N | <list> | <classification> |
| 35-recovery-paths | N | <list> | <classification> |
| 41-mnemonic | N | <list> | <classification> |
| 42-md | N | <list> | <classification> |
| 43-ms | N | <list> | <classification> |
| 44-mk-cli | N | <list> | <classification> |
```

For each finding, capture: where (file + line range), what (claim vs actual), tentative classification (doc-update / toolkit-fix / gray-area).

---

### Task 4: Phase 3a — P1b architect classification (opus dispatch)

**Files:** none modified yet.

- [ ] **Step 1: Dispatch opus reviewer for finding classification**

Per the manual-v0.2.0 cycle's P1b R0+R1 discipline. Dispatch with `Agent`:
- `subagent_type: feature-dev:code-reviewer`
- `model: opus`
- Prompt: classify each finding from AUDIT_FINDINGS_manual_v0_3_0.md per Q7 rubric (doc-update / toolkit-fix / gray-area locked-per-finding).
- Output: `design/agent-reports/manual-v0_3_0-p1b-r0-classification.md` with per-finding lock + scope estimate.

Per memory `feedback_opus_primary_review_agent`: opus is the right model for multi-chapter judgment-heavy work.

- [ ] **Step 2: Apply opus's classification to AUDIT_FINDINGS_manual_v0_3_0.md**

Update the table's Status column with the locked classification. File any new FOLLOWUPs (toolkit-fix gray-areas → defer; substantive prose expansions → defer) in `design/FOLLOWUPS.md`.

---

### Task 5: Phase 3b — apply prose updates

**Files:** all 9 chapter files + AUDIT_FINDINGS doc.

- [ ] **Step 1: For each doc-update finding, apply the prose change**

This is the bulk of Cycle 4's work. For each chapter, edit the prose to match the captured transcript bytes / the binary's actual output. Common patterns:
- Update old md1/mk1/ms1 prefix strings (`md1zsxdspq...` → `md1fgdxlpq...` or whatever v0.28.x emits).
- Update old descriptor outputs (e.g., `wpkh(@0/<0;1>/*)` may still be correct OR may have changed; verify against runtime).
- Update flag listings (chapters 42/43/44 CLI references) to match `<binary> --help` output exactly.
- Update fingerprint case (BIP-388 canonicalization; per F10 lesson — lowercase consistently).

For each fix, commit per the per-chapter cadence (rather than one massive commit) to keep the diff reviewable.

- [ ] **Step 2: Periodic `make audit` runs**

After every 2-3 chapters' fixes:

```bash
make -C docs/manual audit \
  MNEMONIC_BIN=$(realpath target/debug/mnemonic) \
  MD_BIN=md MS_BIN=ms MK_BIN=mk \
  FIXTURES_DIR=$(realpath crates/mnemonic-toolkit/tests/fixtures/wallet_import)
```

Expected: `[lint] OK` + `[verify-examples] OK (N transcripts pass)` with N growing as SKIP_STEMS shrinks.

---

### Task 6: Phase 4 — remove SKIP_STEMS

**Files:**
- Modify: `docs/manual/tests/verify-examples.sh`

- [ ] **Step 1: Remove the SKIP_STEMS array + the `is_skipped` helper**

The block lives at `docs/manual/tests/verify-examples.sh` around L34-50 (added in manual-v0.2.0 commit `52f33f7`). Locate:

```bash
grep -n 'SKIP_STEMS\|is_skipped' docs/manual/tests/verify-examples.sh
```

Remove the array definition + helper + the call site that skips matching files. The remaining iteration becomes:

```bash
mapfile -t cmd_files < <(find "$TRANSCRIPTS" -type f -name '*.cmd' -not -path '*/cli-help/*' | sort)
# ... (remove the is_skipped check inside the for-loop)
```

- [ ] **Step 2: Verify all 14 transcripts now pass**

```bash
make -C docs/manual audit \
  MNEMONIC_BIN=$(realpath target/debug/mnemonic) \
  MD_BIN=md MS_BIN=ms MK_BIN=mk \
  FIXTURES_DIR=$(realpath crates/mnemonic-toolkit/tests/fixtures/wallet_import)
```

Expected: `[verify-examples] OK (14 transcripts pass)` (10 in-scope + 4 newly-readmitted). If any of the 4 newly-readmitted transcripts fail, the recapture in Task 2 wasn't byte-faithful — re-capture + commit + re-test.

---

### Task 7: Phase 6 — opus end-of-cycle holistic review

**Files:** none modified yet.

- [ ] **Step 1: Dispatch opus**

Mirror manual-v0.2.0 cycle's P5.2 R0+R1 cadence:
- `subagent_type: feature-dev:code-reviewer`
- `model: opus`
- Prompt: holistic review of the entire Cycle 4 commit chain (all chapter prose edits + transcript recaptures + SKIP_STEMS removal). Surface any cross-cutting findings the per-chapter classification might have missed.

Persist report to `design/agent-reports/manual-v0_3_0-p6-end-of-cycle-review.md`.

- [ ] **Step 2: Fold any Important findings inline**

Loop R0 → R1 → ... until 0 Critical / 0 Important.

---

### Task 8: Phase 7 — commit + tag + push + GH Release

**Files:** none modified beyond Phase 3b commits.

- [ ] **Step 1: Final FOLLOWUPS Status flip**

`design/FOLLOWUPS.md` entry `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh`:

Old:
```markdown
- **Status:** `open`
```

New (where `<commit-sha>` is the cycle-close commit, backfilled post-commit):
```markdown
- **Status:** `resolved <commit-sha>` — manual-v0.3.0 cycle audited + refreshed all 9 chapters (22/23/24/31/35/41/42/43/44) carrying pre-v0.15.0 wire-format card strings. 4 transcripts recaptured against v0.28.x; SKIP_STEMS removed from verify-examples.sh; `make audit` now passes 14/14 transcripts.
```

- [ ] **Step 2: Tag manual-v0.3.0**

```bash
git tag manual-v0.3.0
git push origin master  # push any uncommitted cycle-close edits first
git push origin manual-v0.3.0
```

- [ ] **Step 3: Monitor CI**

Use Monitor tool to watch the manual.yml workflow on the tag. Expected: PASS (lint OK + verify-examples 14/14 OK + PDF GH Release with refreshed asset).

The manual.yml workflow auto-creates the GH Release (per `manual.yml:125-132` "Ensure GitHub release exists for this tag" step). No manual `gh release create` needed.

- [ ] **Step 4: Confirm GH Release**

```bash
gh release view manual-v0.3.0 --json tagName,assets,createdAt | jq .
```

Expected: release exists with `m-format-manual.pdf` asset attached.

---

## Self-review

After completing all 8 tasks, verify against the brainstorm spec:

1. **Spec coverage:**
   - Phase 0 (multi-chapter recon + FOLLOWUP body amendment) → Task 1 ✓
   - Phase 1 (recapture 4 transcripts) → Task 2 ✓
   - Phase 2 (audit chapter prose) → Task 3 ✓
   - Phase 3a (P1b architect classification) → Task 4 ✓
   - Phase 3b (apply prose updates) → Task 5 ✓
   - Phase 4 (remove SKIP_STEMS) → Task 6 ✓
   - Phase 5 (local `make audit`) → Task 6 Step 2 ✓
   - Phase 6 (opus end-of-cycle review) → Task 7 ✓
   - Phase 7 (commit + tag + push + GH Release) → Task 8 ✓
   - Phase 8 (FOLLOWUPS Status flip) → Task 8 Step 1 ✓

2. **No-placeholder check:** Task 3 + 5's prose updates are described abstractly because the actual content depends on what the binary emits at execution time (which is what makes this an audit cycle). The structure (per-chapter table, P1b classification, fix loop) is concrete. No TBD blocks.

3. **Type consistency:** N/A (markdown-only cycle).

4. **Effort estimate sanity-check:** 3-5 days per architect I3. Tasks 1-2 (~1 day for recon + recapture); Task 3 (~1-2 days for 9-chapter audit); Tasks 4-5 (~1-2 days for classification + fix loop); Tasks 6-8 (~0.5 day for end-of-cycle close). Realistic.

---

## Risk flags

- **Scope can inflate during Phase 2.** A 9-chapter audit may surface dozens of findings. If the finding count > ~50, consider partitioning C into sub-cycles (per the brainstorm's Option 3 of the C-scope question; deferred at brainstorm time but available if needed). Architect lock at Task 4 can recommend partition.

- **Chapter-42/43 may have ZERO findings.** Cycle 2's surprise observation (local `make audit` with real md/ms binaries surfaced zero flag-coverage warnings) suggests the md + ms CLI references may already be accurate. If so, Cycle 4's effective scope shrinks to 7 chapters; commit message documents this.

- **Toolkit version dependency.** Recapture in Task 2 uses whatever toolkit binary is built at cycle-start time. If Cycle 3 (v0.28.4) hasn't shipped, capture against v0.28.3; if it has, capture against v0.28.4 (recipes using `--format coldcard-multisig` would land in chapter-45 only, which is NOT in Cycle 4's scope — so the choice doesn't affect Cycle 4's deliverable).

- **23-verify.cmd embeds card strings.** Task 2 Step 2 requires editing 23-verify.cmd to embed the NEW card strings (which depend on Task 2 Step 1's output). This is a per-cmd-file fixture-cascade pattern; the executor must run Step 1 first, then update Step 2's .cmd file, then recapture.

- **24-recover-md1.cmd needs real `md` binary.** Per Cycle 2 / manual-v0.2.1 ship, `MD_BIN=md` works in CI; locally, `which md` may return the `mkdir -p` shell alias — use `/home/bcg/.cargo/bin/md` (or wherever cargo installed it) to bypass.

- **Sub-skill expectations.** This plan assumes the executor uses `superpowers:subagent-driven-development` or `superpowers:executing-plans`.

- **`make audit` is the durable gate.** Any cycle-close commit MUST be preceded by a clean `make audit` run (per architect I3 fold). If audit fails on the cycle-close commit, revert + triage; don't ship a broken manual.
