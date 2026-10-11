# ci-docs-skip plan review r1 (independent, adversarial)

Artifact: toolkit branch `design/ci-docs-skip` @ ff2c1393, `design/ci-docs-skip/`
(PLAN_ci_docs_skip_2026-10-11.md, docs-only.sh, docs-only.test.sh, paths/*.txt, docs-skip-gate.py).
Bases read: dm 957a0831, toolkit 4d703bb4, key 340d9da, secret 8fef227, engrave 21136827, mt e0200b6
(pre-hardening; hardening adds only timeouts + concurrency to these jobs).

**Verdict: 1 Critical / 6 Important / 5 Minor / 4 Nit. NOT GREEN.**

Measured here, beyond what the brief lists as already verified:
- Applied D1 + F + D4(release.yml) to scratch clones of dm 957a0831 and toolkit 4d703bb4 (toolkit also D2)
  with a line-level applier, then ran the real gate. Parsed YAML and counts match (dm 7/30/2/2, toolkit
  10/40/3/2), `files` GREEN, but `workflows` is **RED** in both on the correct edit (I1).
- Runner facts: macos-15-arm64 image readme lists "Bash 3.2.57(1)-release" (actions/runner-images).
  Windows checkouts CRLF-convert text files without an `eol=lf` rule: the toolkit's own `rust.yml`
  comment records this measured on 2026-08-27 (three windows-latest jobs RED on a CRLF-converted corpus).
- Note: I ran `git fetch -q origin` once in each of the six repos before realising it touches
  remote-tracking refs; every origin ref resolved to the recon's SHA, so nothing moved.

---

## C1 — Critical — "the default tip passed its checks" is false under `strict: false` PR merges

Location: PLAN "The safety rule" (last paragraph); docs-only.sh header and reference table.

The plan admits one exception (admin bypass). There is a second one, and it is the normal merge path.
Branch protection is `strict: false` in these repos (dm `ci.yml` and engrave `release.yml` header
comments say so; push-via-staging.sh relies on it). With `strict: false`, a PR whose base has moved
is mergeable on its old green contexts. The merge button then creates a new tip M whose tree was never
tested. M's own push run starts only after M is already the default tip.

Failure scenario:
1. PRs P1 and P2 are both green against main B0. P2 merges, so main is B1. P1 merges, and main is
   M = B1 + P1, a tree nobody tested. Say P1 and P2 conflict semantically, so M fails `cargo test`.
2. While M's push run is still running (or after it went red), a docs-only PR P3 opens. Its
   `PR_BASE_SHA` is M, the diff is docs, the guard writes `docs_only=true`, and every required
   context on P3 goes **green**. Its code is M's code, which is untested or failing.
3. P3 merges and becomes M3. M3's push run has `before` = M, so it skips and goes green. Main's
   newest tip now shows green required contexts on broken code. Every later docs-only staging
   push compares with M3 and also goes green. The break surfaces only when some later code change
   runs full, and that change gets the blame.

Before this change, every run on top of M was red until someone fixed it. After it, docs runs paint
over the red. This is a required check passing on code that was never tested, or that failed. The same
hole opens whenever the default tip landed by any route that did not earn its contexts first (bypass, or
a PR merge on a moved base). Pushes to `ci/**` (reference = current default tip) inherit it.

Fix (decide in the plan, one of):
- (a) **Verify the reference instead of assuming.** Give the guard step
  `CHECK_NAME: <this job's rendered context name>` (matrix names work in `env`, e.g.
  `test (${{ matrix.os }})`) and `GH_TOKEN: ${{ github.token }}`. In docs-only.sh, after `base` is
  resolved, query
  `GET $GITHUB_API_URL/repos/$GITHUB_REPOSITORY/commits/$base/check-runs?check_name=<CHECK_NAME>&filter=latest`
  and write `docs_only=false` unless the latest run for that name has `status=completed` and
  `conclusion=success`. Any curl or parse failure means a full run. A reference that was itself
  docs-skipped green is fine: the property is transitive. Engrave's `test` job has
  `permissions: contents: read`, so it also needs `checks: read`. docs-only.test.sh then needs a stub
  for this lookup (e.g. `DOCS_ONLY_CHECKS_CMD`), plus scenarios for "reference red", "reference
  pending" and "API failure".
- (b) Owner ruling to accept the hole. Then state it in the safety rule next to the admin-bypass
  sentence, and say that a docs-only change landing on a red or pending tip will report green.

## I1 — Important — the gate is RED on the correct edit in four repos; its self-test never exercises the removed-lines check

Location: docs-skip-gate.py `allowed_removed()`, `check_workflows()`, `self_test()`; PLAN D1.3.

D1.3 says to *replace* an existing `if:` line. That is necessarily a removed line in `git diff`.
`allowed_removed()` returns `[]` for dm `ci.yml`, key `ci.yml` and secret `rust.yml`, and for
toolkit `rust.yml` it returns only `TK_REMOVED`. So the removed-lines check fails on the plan's own
nine combined-`if` steps. Reproduced:

```
descriptor-mnemonic: files=6 jobs=7 steps=30 combined=2 selftests=2
FAIL: ci.yml: removed lines differ from the plan (unplanned: ['        if: ${{ matrix.arm_native }}', '        if: ${{ matrix.musl_native }}'], not removed: [])
GATE: RED
mnemonic-toolkit: files=19 jobs=10 steps=40 combined=3 selftests=2
FAIL: rust.yml: removed lines differ from the plan (unplanned: ['        if: ${{ matrix.arm_native }}', '        if: ${{ matrix.musl_native }}', "        if: matrix.target == 'aarch64-unknown-linux-gnu'"], not removed: [])
GATE: RED
```

The implementer then reaches a stop point ("a gate RED that is not your own slip") on every one of
these repos. The self-test missed it because it writes `yaml.safe_dump` output and replaces
`allowed_removed` with "whatever was removed". The docstring says the removed-lines check is "tested
on its own", but no such test exists in `self_test()`.

Fix: have `allowed_removed()` also return every base line `l` where `l.strip()` starts with `if: `
and belongs to a step that `guard_job()` combines. Simplest: collect each original `if` text inside
`guard_job()`, then match base lines `        if: <original>`. Add a self-test case that applies the
plan *textually*, e.g. this review's line applier, to the dm base and asserts GREEN, plus one textual
mutation, such as deleting an unrelated comment line, that asserts RED.

## I2 — Important — the guard step fails in both `g6-invariant` jobs (checkout into a subdirectory)

Location: PLAN D1 (toolkit `rust.yml` "every job (9)", secret `rust.yml` "every job (11)"); gate `GUARD`.

In toolkit `g6-invariant`, the first step is `actions/checkout@v6` with `path: mnemonic-toolkit`.
In secret `g6-invariant`, it is `actions/checkout@v4` with `path: mnemonic-secret`, followed by a
second checkout of the sibling. The guard step runs `bash ci/docs-only.sh` in `$GITHUB_WORKSPACE`,
where no `ci/` exists. bash exits 127, the step fails, and the job is red on every event, code or docs.
The g6 cross-repo byte-equality test then never runs again: later steps carry an `if:`, which implies
`success()`. The gate is GREEN here, because parsed YAML matches `expected()`. The failure first shows
up in the PR's CI, which the plan treats as a stop point, with no decided remedy.

Fix (pick one and update GUARDED/EXPECTED_COUNTS):
- (a) Exclude `g6-invariant` from D1 in both repos. It is non-required and cheap. Counts drop by one
  job in each: toolkit 9 jobs / 37 steps, secret 11 jobs / 35 steps (recompute with the gate).
- (b) Give the guard step `working-directory: mnemonic-toolkit` / `mnemonic-secret` in those two jobs,
  and model that variant in the gate.

## I3 — Important — on Windows legs the guard is checked out with CRLF and cannot run

Location: PLAN F (no `.gitattributes` change); docs-only.sh; paths/*.txt.

Windows runners CRLF-convert text files that have no `eol=lf` rule. The toolkit `rust.yml` comment
records exactly this, measured 2026-08-27. None of dm/toolkit/key/secret `.gitattributes` covers
`ci/docs-only*`. Key even has `* text=auto`. So on `windows-latest`, `ci/docs-only.sh` has CRLF line
endings. Git Bash does not strip `\r` (Git for Windows' own commit history notes that its `read` leaves
a trailing CR). Line 33 `full() {\r` is then a syntax error, the step fails, and the job is red, with all
later steps skipped.

Guarded Windows jobs: dm `cargo test (windows-latest)`, key `build (stable|beta|1.85 on windows-latest)`
(three legs), and toolkit `lib cross-platform check` (x86_64-pc-windows-msvc). All are non-required, so
nothing wedges. But Windows test coverage stops and those jobs stay red. The plan's own PR-CI check
("each guarded job's log must show full run") and the probe's "in every guarded job … all 1 changed
files are documentation" both stop here. Even if the script ran, every rule read from the CRLF paths
file would end in `\r`, would never match, and the guard would always write a full run.

Fix: in F, append to each repo's `.gitattributes` (create one in engrave/mt for uniformity):
```
ci/docs-only.sh text eol=lf
ci/docs-only.test.sh text eol=lf
ci/docs-only-paths.txt text eol=lf
```
Also, defensively, add `line=${line%$'\r'}` as the first statement of the rules `while` loop. Have
`files` mode assert the three attributes (`git check-attr eol -- <path>` = `lf`).

## I4 — Important — a pull_request into a non-default base uses an untested reference

Location: docs-only.sh `pull_request)` arm; PLAN safety-rule table.

Every guarded workflow takes `pull_request:` with no `branches:` filter, so PRs into any branch run.
For a PR into `feature-A` (stacked PRs, or a PR into `ci/staging`), `PR_BASE_SHA` is that branch's
tip. Nothing says that tip ever passed. A docs-only PR B on top of an untested or red `feature-A` gets
green required contexts on its head SHA. If B is then retargeted to the default branch by hand, the
`edited` event does not re-run the default `pull_request` types. B shows green and can be merged,
landing `feature-A`'s untested code.

Fix: in the `pull_request)` arm, add
`[ "${GITHUB_BASE_REF:-}" = "$DEFAULT_BRANCH" ] || full "pull_request into ${GITHUB_BASE_REF:-an unset base}, not $DEFAULT_BRANCH"`.
Add a test scenario: PR with `GITHUB_BASE_REF=feature`, expect `false`. Add `GITHUB_BASE_REF=main` to
the existing PR scenarios.

## I5 — Important (undecided point) — D1.3 does not say where `if:` goes on a `- run:` step

Location: PLAN D1.3 ("directly below its `- name:` (or `- uses:` when it has no name)").

Eight guarded steps have neither `name` nor `uses` and start with `- run:`. dm `ci.yml` has six
(lines 57, 59, 75, 87, 103, 134), key `ci.yml` has one (197) and secret `rust.yml` has one (398).
Under the project's Sonnet-proof rule an implementer stops here. Fix: add "or `- run:` when it has
neither". The gate is position-agnostic, so either placement passes.

## I6 — Important — the gate certifies the working tree, not the commit, and no mode bounds the changed-file set

Location: docs-skip-gate.py `check_workflows()` (reads the worktree files and `git diff <base>` of
the worktree), `check_release_gen()`; PLAN "Work order" (gate, then commit).

The gate runs before the commit and reads unstaged files. An implementer who stages explicitly (as
required) but misses one edited workflow gets GREEN gate lines in the commit message for a commit that
does not contain them. Separately, no mode checks files outside `.github/workflows/` and `ci/`: an
accidental edit to, say, `ci/doc-gate-guard.sh` or `Cargo.toml` passes every mode. That makes the gate
a false PASS for "the commit is exactly the plan".

Fix: re-run `files` and `workflows` after the commit, with a precondition that
`git status --porcelain --untracked-files=no` is empty. Add a check that
`git diff --name-only <B> HEAD` equals the planned set: the three `ci/docs-only*` files, the edited
workflow files, `.gitattributes` if I3 is folded, and for engrave also
`scripts/release-workflows/{gen,emit}.py`.

---

## M1 — Minor — release.yml `test` checks out `github.ref`, so HEAD may not be the SHA the check is attached to

The generated `test` job checks out `ref: ${{ github.event.inputs.tag || github.ref }}`. On a push, that
is the branch tip *at job start*, not `github.sha`. If `ci/staging` is deleted and re-created (or
force-pushed) inside the window, run X's job can test, or now docs-skip, a later commit Y. The result
is still attributed to X. In toolkit and secret this job shares the required name
`test (ubuntu-latest)` (H1), and push-via-staging takes the latest check run per context. This is
pre-existing, but the skip makes it cheaper to trigger. Fix: for `push`, add
`[ "$(git rev-parse HEAD)" = "${GITHUB_SHA:-}" ] || full "HEAD is not GITHUB_SHA"`.

## M2 — Minor — required macOS legs run the guard on bash 3.2.57, which the self-test never runs on

The self-test runs only on Linux (clippy/fmt jobs; `runner.os == 'Linux'` in release.yml). toolkit and
secret `test (macos-latest)` are required and run the guard under `/bin/bash` 3.2.57. My review of
the constructs (`<<<`, `PIPESTATUS` in `$( )`, `${x%$'\n'}` with `extquote` on by default, implicit
arrays, `read || [ -n ]`, BSD `grep -E`/`tr '\0'`) found no divergence, and a crash is safe (red). Fix:
also run `bash ci/docs-only.test.sh` in the toolkit and secret `test` job on the macOS leg
(`if: runner.os == 'macOS' && …`).

## M3 — Minor — `-` rules are case-sensitive while macOS checkouts are case-insensitive

A change that adds `Readme.md` next to the protected `README.md` (toolkit), or a case variant of
`design/SPEC_convert_v0_6.md`, counts as docs. On the required macOS leg, the two paths collide on
disk, which can change what `readme_version_current.rs` or the `include_str!` reads. Fix: match `-`
rules with `grep -qiE`.

## M4 — Minor — nothing keeps the protected lists current

The `-` entries are hand-maintained. The next `include_str!`/`read_to_string` of a `design/`, `docs/`,
`bip/` or root `*.md` path, or a symlink from code into those trees, silently makes that file
skip-unsafe. File a FOLLOWUPS item (owning phase: next CI cycle) for a test or CI step that greps
first-party sources for references to allow-listed paths and fails if any is not covered by a `-` rule.

## M5 — Minor — the self-test proves `same()` detects differences, not that `expected()` encodes the plan

All five mutations are caught by the parsed-equality check, so the self-test cannot fail on a wrong
`expected()`. That is inherent, but the brief and the PLAN line "applies five mutations … (all RED)"
should say so. Folding I1's textual-apply case gives one real cross-check.

## N1 — Nit — the self-test mutation "drop the guard step" pops step index 1. In engrave `release.yml` that is the minisign step, not the guard. It is still RED, but mislabelled.
## N2 — Nit — D1.1 "directly below the job's first step" does not say whether the guard goes before or after the blank line and comment block that follow the checkout (e.g. toolkit `test`). It does not affect the gate.
## N3 — Nit — engrave's kept comments and the `docs-only verdict` echo still say "identical to the green parent". The reference is now the default tip for `ci/**` pushes.
## N4 — Nit — `check_workflows()` drops any removed line whose content starts with `--`, because of the `---` header filter. Harmless for these files; filter only the first `---` header line instead.

---

Checked and found sound (no finding):
- Tree-to-tree comparison. It is sound for any reference whose tree passed: a stale
  `pull_request.base.sha` only adds files to the diff. Force-pushes and merge commits on the default
  branch are fine. An unfetchable `before` gives a full run.
- `ci/**` merges and non-default pushes. Fork PRs. `merge_group` / `pull_request_target` (absent; would
  be a full run). Tag/dispatch/schedule give a full run. `default_branch` is present in push and
  pull_request payloads.
- `git fetch --depth=1 origin <sha>` into a depth-1 checkout. Engrave's production detector already
  does this on GitHub. `FETCH_HEAD^{commit}`.
- Required-context coverage. D1+D4 cover every required context in the recon, including the H1
  `release.yml test (ubuntu-latest)` collision in toolkit/secret.
- No guarded job has `always()`/`failure()` aggregators or artifact hand-offs to unguarded jobs.
  key `vectors-roundtrip` `needs: build` is fine. engrave `assemble` runs on tag/dispatch only, which is
  a full run.
- Leaving toolkit's five doc gates unguarded is correct: they consume `docs/`.
- D2: removing the push `paths:` filter only adds runs. Previously-unfiltered omissions
  (`rust-toolchain.toml`, `vendor/**`, `scripts/` beyond install) now run full. push-via-staging judges
  only required contexts. No test or script parses `rust.yml`.
- Protected lists. Re-grepped all six repos (excluding vendor) for reads of `design/`, `docs/`, `bip/`
  and root `*.md`. The only hits are the listed `-` entries; crate-level READMEs/CHANGELOGs; engrave
  `design/hardware/`; engrave scripts no workflow runs; and mt `scripts/check-provenance.sh`, which only
  mt `ci.yml` (unguarded) runs. No non-vendor symlinks point into allow-listed trees, there is no
  `build.rs`, and there is no README `doc = include_str!`.
