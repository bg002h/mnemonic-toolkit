architect vf-s1: 2 answered; 1 prior rulings changed

Question file: coordinator `briefs/architect-vf-s1-q.md`. Spec: `design/SPEC_vendor_freshness_shared.md` at
92e7dd2b. No Critical or Important found. Toolkit is at 8776f5b6 and mr at 84492d99 (origin default branches,
fetched 2026-11-08), not the spec's SHAs. `git diff --stat` of `ci/` and `.github/` from the spec's SHAs to those
heads is empty in both, so the spec's line citations still hold. Local checkouts of key, secret, dm and mr are
behind origin, so every measurement below reads `origin/<default>` through `git show`.

## Checked claims

- `ci/shared-ci.sha256` on origin: toolkit, engrave, transaction, key, secret and dm all hash to `cc22c1754b9f…`
  (sha256sum of `git show`), so the six copies are byte-identical. mr's copy differs (`034f3ccd…`) and holds two
  lines (code-docs-check.sh and its test), as the spec says. Only toolkit has `ci/shared-drift.sh`.
- Who checks `shared-ci.sha256`: engrave and transaction only in `release.yml` (116 and 118). key `ci.yml:150`,
  dm `ci.yml:102`, secret and toolkit `rust.yml` (386 and 382), all in `release.yml` too. mr `ci.yml:109` runs on
  every push.
- Path filters in `vendor-freshness.yml`: toolkit already lists `ci/repro/**` (lines 34 and 42). key, secret and
  dm list only `ci/repro/vendor-freshness.sh` (lines 38 and 46). mr's job (`ci.yml:274`) has no path filter;
  the workflow ignores only agent-reports and FOLLOWUPS.
- The proto hashes equal the `vf:proto-sha256` block (9a2761a6…, 14386f52…).
- No `ci/docs-only-paths.txt` rule matches anything under `ci/repro/`. The only `ci/repro` consumers outside
  vendor-freshness (toolkit `reproducible-musl-build.yml:515-600`) name their scripts one by one, so a new
  non-`.sh` file there is inert.
- Without rsync, `vendor-freshness.test.sh` stops before any row (`set -euo pipefail`, line 28, and the rsync
  pipe, lines 38-39) and exits non-zero. It fails closed, never with a false PASS.

## Q1 (S-1): where the hash lines live, and the drift scope

**Answer: option B, with the pin file moved under `ci/repro/`.**

- The pin file is **`ci/repro/shared-vendor.sha256`**, byte-identical in toolkit, key, secret and dm. It holds
  exactly the output of `sha256sum ci/repro/vendor-freshness.sh ci/repro/vendor-freshness.test.sh`, run from
  the repo root: two LF-terminated lines, `<64 lowercase hex><two spaces><path>`, script first, then test, with
  no header. At the proto hashes:
  ```
  9a2761a6e50a5fda600347bd84af90f8b6c72edfd5eb731b263ff63813d11241  ci/repro/vendor-freshness.sh
  14386f525d7278f0aa210b10f658c6d8d0b4dc0c0cbc0de28c954a7f44f86b27  ci/repro/vendor-freshness.test.sh
  ```
- **The local check** goes in each of the four `vendor-freshness.yml` files, before
  `Resolve Cargo.lock against committed vendor/ (offline, locked)`:
  `- name: Shared vendor gate files are unmodified` with
  `run: sha256sum --check --strict ci/repro/shared-vendor.sha256`.
- **mr** follows the audit as written. Append the same two lines, script then test, to mr's own
  `ci/shared-ci.sha256`, after its existing two lines. `ci.yml:109` already checks that file on every push, so mr
  needs no new step for the pin. mr gets no `shared-vendor.sha256`. It stays out of shared-drift, as it is today
  for the code-docs files.
- **engrave and transaction:** no change, no PR. `ci/shared-ci.sha256` is unchanged everywhere.
- **Drift scope:** toolkit `ci/shared-drift.sh` gains a second group, compared only against
  descriptor-mnemonic, mnemonic-key and mnemonic-secret. Its files are `ci/repro/vendor-freshness.sh`,
  `ci/repro/vendor-freshness.test.sh` and `ci/repro/shared-vendor.sha256`. The script changes are:
  - Header lines 2-7 become:
    ```
    # ci/shared-drift.sh -- check that the shared CI files on each other public
    # Part A repo's default branch are byte-identical to this checkout's: the five
    # docs-gate files in five repos, and the three vendor-gate files in the three
    # repos that commit vendor/.
    #
    # Each repo checks its shared files only against its own ci/shared-ci.sha256
    # or ci/repro/shared-vendor.sha256, so a shared file edited together with its
    # hash line in one repo passes there. This compares the repos with each
    # other. Run from the repo root.
    ```
  - After line 15 (`files=(…)`), add:
    ```
    vendor_repos=(descriptor-mnemonic mnemonic-key mnemonic-secret)
    vendor_files=(ci/repro/vendor-freshness.sh ci/repro/vendor-freshness.test.sh ci/repro/shared-vendor.sha256)
    ```
  - Lines 26-43 (the nested loop) move unchanged into
    `compare() { # compare <repo> <file>: one ok, DRIFT or ERROR line; counts problems in k`, with
    `local repo=$1 f=$2 here_sha there_sha` and `return` in place of `continue`. Two loops then call it:
    `for repo in "${repos[@]}"` over `"${files[@]}"`, then `for repo in "${vendor_repos[@]}"` over
    `"${vendor_files[@]}"`.
  - The PASS line becomes
    `shared-drift: PASS (${#repos[@]} repos x ${#files[@]} files, ${#vendor_repos[@]} repos x ${#vendor_files[@]} vendor files)`,
    which prints `shared-drift: PASS (5 repos x 5 files, 3 repos x 3 vendor files)`.
- **`ci/shared-drift.test.sh`** changes as follows:
  - It gains `vendor_repos` and `vendor_files`.
  - `fresh()` serves each group to its own repos, through a `put <repo> <file>` helper.
  - D1 expects 34 ok lines and the new PASS line.
  - D5 copies both groups.
  - D8 is new: one byte appended to mnemonic-secret's `ci/repro/vendor-freshness.sh` gives
    `DRIFT mnemonic-secret ci/repro/vendor-freshness.sh: ` and `FAIL: 1 problem(s)`.
  - D9 is new: D1's gh arguments name no `ci/repro/` path for engrave or transaction.
- The full diff of both files, against toolkit origin/master 8776f5b6, is
  `/tmp/claude-1000/-scratch-code-shibboleth-refugium-coordinator-coordinator/14229dac-31ea-4af5-8ffb-f1cf9671c2d4/scratchpad/drift.diff`
  (82 lines for the script, 166 in total). Its sha256 is `ba8982fa…`. The resulting script hashes to
  `6dc3f8b7…` and the test to `6e88cb5e…`.
- **Run (build gate):**
  - Unmodified test against the new script: 5 passed, 2 failed (RED).
  - New test: `shared-drift.test.sh: 9 passed, 0 failed`.
  - Planted fault M1 (vendor loop over nothing): D1 and D8 fail, 7 passed, 2 failed.
  - Planted fault M2 (vendor loop over all five repos): D1, D2, D8 and D9 fail, 5 passed, 4 failed.
  - shellcheck is not installed here and no CI job runs it.

**Why.** The audit's item 5 has two clauses that cannot both hold:
- "pinned in each repo's `ci/shared-ci.sha256`";
- "added to … `shared-drift.sh` `files` for the public repos (key, secret, dm)".

`shared-drift.sh:14-15` compares `ci/shared-ci.sha256` across all five siblings, engrave and transaction among
them. So the first clause forces the vendor gate onto engrave and transaction, and the second keeps it off them.
Either option changes one clause, so neither is a free reading. Option B keeps the audit's intent: a three-repo
drift scope and mr pinning in its own file. Option A puts an unrun gate in two repos that have no `vendor/`, and
needs two more PRs, review rounds and catalog rows. That cost buys no correctness, so B wins on Brian's third
priority.

I moved the pin under `ci/repro/` to close a gap. The spec's `ci/shared-vendor.sha256` sits outside every
`vendor-freshness.yml` path filter (`ci/repro/**`), so a PR that edits only the pin would never run the step that
checks it. That is a check that cannot fire on its own file. Under `ci/repro/`, the `ci/repro/**` filter (toolkit
today, and key, secret and dm after Q2) covers the pin with no new filter lines.

Kept unchanged: `shared-drift.yml`'s path filter, the daily schedule (the cron runs the full compare), and the
rollout order (PR-T2 after PR-K, PR-S and PR-D).

**Prior ruling changed (1):** the audit's "pinned in each repo's `ci/shared-ci.sha256`", for the four public
vendor repos. This is not a C-class defect. The audit contradicts itself, and this is the smaller resolution.
mr's clause stands.

**Fold needed: yes.**
- Rewrite §10 as the decision above (status line 3 and line 1002 no longer "open"), and drop the option A text.
- §11 changes:
  - Replace `ci/shared-vendor.sha256` with `ci/repro/shared-vendor.sha256` (toolkit, and key/secret/dm).
  - The pin step's `run:` path changes to match.
  - mr's item stays as written.
- §12 changes:
  - PR-T2 carries the diff above.
  - Drop option A's PR-E and PR-X.
- §13: move the shared-drift edits from "NOT covered" to covered. Add a gate check that applies the diff to
  toolkit at its spec SHA and requires `9 passed, 0 failed`, so the fold's text is gated, not copied.

## Q2: the shared test in every repo's CI

**Answer: confirm, with this exact scope.**
- **key, secret, dm** (`vendor-freshness.yml`):
  - Both path lists change `'ci/repro/vendor-freshness.sh'` to `'ci/repro/**'`. This covers the script, the test,
    the data file and the Q1 pin.
  - After the gate step, add `- name: Mutation tests for the gate` with
    `run: bash ci/repro/vendor-freshness.test.sh`.
  - No other trigger changes.
- **toolkit:** unchanged. It already runs the test at `vendor-freshness.yml:71-72` and already filters on
  `ci/repro/**`.
- **mr:** in `ci.yml`'s `vendor-freshness` job, add `- run: bash ci/repro/vendor-freshness.test.sh` after line
  283. No path filter is added, so it runs on every push, like the gate it sits beside.
- **engrave and transaction:** nothing.

**Why.** The audit's "Callers unchanged" names the gate's invocation (`vendor-freshness.yml` x4, mr `ci.yml:283`)
and does not forbid running its test. The test's rows mutate the repo's own tree, and its control row must stay
green against that repo's `Cargo.lock` and data file. Run only in toolkit, it proves nothing about whether dm's
or secret's gate can fail on dm's or secret's tree. A gate never shown to fail where it runs is the blocking
class. key's 9 skipped G rows print as `skip`, are not counted as passed, and are correct: key has no git source.

Cost is low:
- the four public workflows are path-filtered;
- mr's run is on the free self-hosted runner;
- a missing rsync on mr's runner fails PR-R's own CI loudly (checked above), so the spec's "not covered" item
  becomes a PR-time red, not a silent gap.

This is the spec as authored, plus the `ci/repro/**` filter that Q1 now relies on. **Fold needed: no**, beyond
noting in §10 that the Q1 pin relies on this filter.

## Open (not mine to decide)

None.

## Essential files

- /scratch/cache/spec-vf/design/SPEC_vendor_freshness_shared.md:1-8, 781-835 (§10), 836-946 (§11), 947-981 (§12), 982-1006 (§13)
- mnemonic-toolkit origin/master `ci/shared-drift.sh:1-50`, `ci/shared-drift.test.sh:1-122`, `.github/workflows/shared-drift.yml:1-25`, `.github/workflows/vendor-freshness.yml:25-72`
- mnemonic-key origin/main `.github/workflows/vendor-freshness.yml:1-70` (secret and dm identical at 18, 38, 46, 69)
- mnemonic-refugium origin/master `.github/workflows/ci.yml:108-109, 274-283`, `ci/shared-ci.sha256:1-2`
- /tmp/claude-1000/-scratch-code-shibboleth-refugium-coordinator-coordinator/14229dac-31ea-4af5-8ffb-f1cf9671c2d4/scratchpad/drift.diff:1-166

Counts: 2 answered; 1 prior ruling changed; 0 Critical, 0 Important, 0 Minor, 0 Nit; 0 open.
