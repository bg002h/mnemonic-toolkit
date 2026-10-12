# CI docs-skip: whole-diff adversarial review (pre-ready)

Reviewer: independent, Opus, high effort. Read-only: nothing pushed, commented, merged or edited.

## Heads reviewed

| Repo | PR | Head | Base B (merge-base) |
|---|---|---|---|
| descriptor-mnemonic | #37 | bd9e2d78 | 0da734f2 |
| mnemonic-toolkit | #103 | a41b546c | fb2d1a9a |
| mnemonic-key | #13 | 5ac93405 | 3c7a403f |
| mnemonic-secret | #12 | ecae58cd | 5a57fe8a |
| mnemonic-transaction | #3 | 7915f365 | 9e4539cd |
| mnemonic-engrave | #19 | 1423244e | a2e129d3 |
| mnemonic-refugium | #82 | 8cee9092 | origin/master |

All six code worktrees were clean (`git status --porcelain` printed 0 lines) with one commit on B.

**C=1 I=0 M=4 N=2**

**Verdict: NOT GREEN (1 Critical).** The docs-skip logic itself is sound. I found no way for a change that is not documentation-only to report green without its tests. The Critical is a regression on a code path that is not a docs-skip: backfilling a release onto an existing tag. Refugium #82's PATH line is correct, and the VM evidence shows it works. It has two fragilities, listed as Minors.

---

## Critical

### C1. The release backfill dispatch (`workflow_dispatch` with `tag:` set to an existing tag) now always fails in `test`, so `assemble` is skipped and no backfill is possible

Where: the generated `release.yml` `test` job in all five repos, and its template:
- descriptor-mnemonic `.github/workflows/release.yml:114` (`ref: ${{ github.event.inputs.tag || github.ref }}`), `:118` (`run: bash ci/docs-only.test.sh`), `:128` (`run: bash ci/docs-only.sh`)
- mnemonic-toolkit `release.yml:113/117/127`, mnemonic-key `:109/113/123`, mnemonic-secret `:117/121/131`, mnemonic-transaction `:106/110/120`
- mnemonic-engrave `scripts/release-workflows/gen.py:140` (the checkout `ref:`), `:144` (self-test), and the guard step right below it.

How it fails: the dispatch input `tag` is documented as "Existing tag to build and BACKFILL binaries onto" (dm `release.yml:59`). The `test` checkout uses `ref: inputs.tag`, so it checks out the **tag's tree**. Every tag cut before this merge has no `ci/docs-only.sh` or `ci/docs-only.test.sh`. The self-test step (Linux) and the guard step (every OS) then exit 127, which fails `test`. `assemble` has `needs: [test, build]` (dm `release.yml:271`), so it is skipped and nothing is uploaded. Before this change, `test` ran the suite on the tag tree and passed.

Reproduced on the newest dm release tag:
```
$ git ls-tree -r --name-only descriptor-mnemonic-md-cli-v0.20.3 -- ci/ | grep -c docs-only
0
$ (tag tree extracted) bash --noprofile --norc -eo pipefail -c 'bash ci/docs-only.sh'; echo $?
bash: ci/docs-only.sh: No such file or directory
127
$ ... -c 'bash ci/docs-only.test.sh'; echo $?
bash: ci/docs-only.test.sh: No such file or directory
127
```
The path is in real use. Past dispatch runs checked out tags:
```
descriptor-mnemonic run 35174824685: ref: descriptor-mnemonic-md-cli-v0.15.0 ... git checkout --progress --force refs/tags/descriptor-mnemonic-md-cli-v0.15.0
mnemonic-key run 35175015156 / 35173753668: ref: mk-cli-v0.13.0 ... refs/tags/mk-cli-v0.13.0
```
Severity: under this brief, a release job that breaks is Critical. It fails loudly and gives no wrong result, so the owner may choose to call it Important. It does not affect new tags, the `sign_dry_run` rehearsal on a branch, or engrave (whose `release.yml` has no `tag` input and checks out `github.ref`).

Fix (one line per step, in gen.py and the five generated files). Either choice is safe, because a skipped guard leaves `docs_only` empty and every guarded step then runs:
- self-test: `if: runner.os == 'Linux' && github.event.inputs.tag == ''`. Guard: `if: github.event.inputs.tag == ''`. Or
- `hashFiles('ci/docs-only.sh') != ''` (and `...test.sh`) on the two steps.

## Important

None.

## Minor (to FOLLOWUPS, `deferred-minor`)

- **M1. Check-name collision: `test (ubuntu-latest)` in toolkit and secret.** It is both `rust.yml` `test` and `release.yml` `test`. Each guard's query returns both workflows' runs:
  ```
  $ gh api -X GET repos/bg002h/mnemonic-toolkit/commits/a41b546c.../check-runs -f check_name='test (ubuntu-latest)' -f filter=latest --jq ...
  completed/success suite=103472103586 .../runs/38192043357/...   (rust.yml, PR)
  completed/success suite=103472103568 .../runs/38192043348/...   (release.yml, PR)
  completed/success suite=103472093386 .../runs/38192039847/...   (rust.yml, push)
  completed/success suite=103472093372 .../runs/38192039842/...   (release.yml, push)
  ```
  Normally this is stricter (both must be green). The guard accepts on the other workflow's run only when this job has no run on R at all: the workflow was disabled when R was pushed, a startup failure, or a newly added workflow. Both files have identical push and PR triggers (verified from the parsed `on:`). The other job also runs `cargo nextest` on the same tree. So: Minor. Fix later by making the names unique, or by matching `details_url` to this run's workflow. No other collision exists among the jobs that ran on the six PR heads (computed from each run's job list, grouped by workflow file).
- **M2. mnemonic-key `ci.yml:43-45` comment is now false.** It says "Workflow-level permissions are not granted; each job declares its own … run with the default read-only token", but D5 adds a workflow-level `permissions:` at `:57`.
- **M3. Refugium #82 README step 8, two fragilities.** Both are analysis only: `pwsh` is not installed here (`which pwsh` → not found).
  1. `Add-Content` appends without first adding a newline. The step before it says only "Write `.env`". If an editor leaves no final newline, `PATH=…` is glued onto the `ACTIONS_RUNNER_HOOK_JOB_STARTED=…` line, which breaks the hook path. That fails loudly, not silently.
  2. `.Replace('C:\ci\cargo\bin', …)` is ordinal and case-sensitive. A machine PATH spelled differently (`C:\CI\…`) is a **silent no-op**: instance 2 would quietly keep instance 1's cargo. Suggest `if ($m -notlike '*C:\ci\cargo\bin*') { throw 'C:\ci\cargo\bin not in machine PATH' }` before the `Add-Content`, and a note that the line is a snapshot of the machine PATH (re-run it after a machine-PATH change).

  Re-running appends a second `PATH=` line. That is harmless (the runner applies `.env` lines in order, so the last wins), but the file grows.
- **M4. emit.py `TEST_SETUP` entries need a hand-added `if:`.** Only `mt` has one today, and nothing enforces it for a future entry. The cost is waste only: an unguarded setup step runs on a docs-only change. It is never unsafe. Nothing else in gen.py's `test` template varies per repo except `test_os`, and a Windows entry would still work (the self-test is Linux-gated and the guard runs under Git Bash).

## Nit

- **N1.** engrave `release.yml:93-100`: the comment block still describes the removed detector ("unknown before-SHA … identical to the green parent"), and it now sits above the signing-key step. `:249`: the `docs-only verdict` echo text says "code tree identical to the green parent".
- **N2.** The PATH half is also covered by M3's re-run note; the FOLLOWUPS status line itself is fine.

---

## What I checked and how

1. **Every diff hunk, read by eye** (`git diff B HEAD` per worktree): .gitattributes, every workflow, gen.py and emit.py, the refugium README and FOLLOWUPS. Only planned content was added. Quoting differs only where the job's own `name:` is quoted (toolkit `"install.sh harnesses …"`, `"test (release, …)"`, secret likewise). No YAML 1.1 hazards: `if:` values are plain scalars starting with `steps.` or `(`, and `&&` sits mid-scalar. Expression precedence `(E) && x != 'true'` is correct (comparison binds tighter than `&&`). `actionlint` at every head printed 0 lines.
2. **CHECK_NAME against real check-run names.** I wrote a script that parses each workflow, expands each matrix (include-only too), renders `CHECK_NAME`, and looks the result up in the job names of the PR-event runs on each head (`gh api …/actions/runs?head_sha=…&event=pull_request` → `/jobs`). All 51 guarded legs printed `OK`, and every rendered name equals the job's rendered name, including toolkit `lib cross-platform check (…, …)` for all 3 legs. No guard names another job, apart from the M1 collision.
3. **Permissions.** Default workflow permissions are `read` in dm, tk, mk and ms (`gh api …/actions/permissions/workflow`). The new top-level blocks keep `contents: read` and add `checks: read`. The only token users in those four files are checkout, the toolchain and cache actions, install-action, and mk `release-on-tag`, which keeps its own job-level `contents: write`. No step is conditional on push or tag except that job (grep of `if:` with `github.ref`/`event_name`). All PR and push checks passed under the new blocks. The release.yml `test` jobs lose `contents/id-token/attestations: write`. Their steps (checkout, toolchain, nextest, cargo, mt's apt) need none of them. engrave `test`/`picotool-probe` only gained `checks: read`, and the SH2 step still has `contents: read` for its API reads.
4. **Later steps depending on guarded steps.** In every guarded job, every step after the guard is guarded, except engrave `SH2 demo install links` (it uses only curl and the token, and was already unconditional) and `docs-only verdict`. Cross-job: `assemble` needs `test` but runs only on a tag or dispatch, where the guard always runs in full. That holds except for C1, where `test` fails before it gets there. mk `vectors-roundtrip needs: build` uses no outputs, and mk `release-on-tag` runs only on tags. No job uses `outputs:` from a guarded job.
5. **Windows Git Bash and macOS.** Job logs from the PR heads show the guard getting past the fetch, `mktemp` and `git diff` on Windows and macOS: `docs-only: full run (not documentation: .gitattributes)` in dm `cargo test (windows-latest)`/`(macos-latest)`, mk `build (stable on windows-latest)`, toolkit `lib cross-platform check (x86_64-pc-windows-msvc, windows-latest)` and `test (macos-latest)`. No guarded job sets `defaults:`/`working-directory:`, and every guard step uses `shell: bash`. **Not exercised anywhere yet:** the `gh api` success path on Windows, macOS and `ubuntu-24.04-arm` (gh present, LF output). Every failure there leads to a full run (safe), and the post-merge probe covers it.
6. **Doc classification versus what guarded steps read.** I grepped every non-doc file outside vendor for `design/`, `docs/`, `bip/`, `*.md` and constructed paths (`join("design")`, `include_str!`). Every read reaching a guarded step is protected or runs in an unguarded step:
   - tk `slip0132.rs` SPEC, root README, three-way-merge plan, display-grouping vectors: protected.
   - mk `SPEC_mk_v0_1.md`: protected. `crates/mk-cli/CHANGELOG.md` is not a root file, so it is not documentation.
   - ms `whitespace_only_separator.rs` reads `display-grouping-vectors.tsv*`: protected.
   - engrave `check-minisign-pin.sh` reads README but runs before the guard. `refugium-otp.sh` reads `design/hardware/`, which is protected.
   - mt `check-provenance.sh` reads `design/`, but only `ci.yml` runs it, and `ci.yml` is not guarded.
   - toolkit `docs/technical-manual/examples` is its own `[workspace]`, outside the root members, and only the unguarded technical-manual.yml builds it.
   - engrave `design/measurements/mt-size-probe` is in the workspace `exclude`.
7. **Guard semantics, adversarially.** I checked: tree-to-tree diff with `--no-renames` (a rename out of `design/` shows the code path), PR base.sha stale or ahead of the merge ref, force-push `before`, staging versus the default tip, `[skip ci]` (no runs, so a full run), a pending or cancelled run on R, and runs on R from PR events. Every case that could be reached ends in a full run, or rests on a run that tested R's tree. The guard runs without `-e` because `bash ci/docs-only.sh` starts a fresh shell, which its `rc` logic needs.
8. **Generator (item 6).** Read gen.py's whole `test` template and emit.py's `TEST_SETUP`/`TEST_OS`. The only per-repo inputs are `test_setup` and `test_os` (see M4). C1 is in the template itself.
9. **Refugium #82.** The PowerShell line is well-formed: single-quoted literals, a parenthesised `-Value`, `-Encoding ascii` (ASCII, no BOM), and `Add-Content` appends. The value is one line, because the registry PATH has no newline. The runner reads `.env` line by line, last line wins. On the VM, runner-2 jobs switched to their own cargo after the change. Before, job 114616925610 logged `cargo is located at /c/ci/cargo/bin/cargo`; after, job 114627463267 (01:33Z) logged `cargo is located at /e/ci/cargo-2/bin/cargo` and `cargo-nextest installed at /e/ci/cargo-2/bin/cargo-nextest.exe`. Remaining fragilities: M3.

Not re-derived (the controller had verified it): the gate's byte, parse and count equality, the gate self-test, worktree and remote head state, and the PR check conclusions.
