# CI-hardening plan review, round 1

Reviewed: `design/ci-hardening/PLAN_ci_hardening_2026-10-11.md` and
`design/ci-hardening/ci-hardening-gate.py` (toolkit worktree at 2fe6cf38; the
gate is byte-identical to `/scratch/code/shibboleth/_work/ci-hardening/ci-hardening-gate.py`).
Bases checked: dm 957a0831, tk 4d703bb4, gui fc25b2b, mk 340d9da, ms 8fef227,
me 21136827, mt e0200b6. All match the plan.

**Verdict: 1 Critical / 3 Important / 7 Minor / 4 Nit**

What holds up (checked, no finding): following T and C literally (I scripted my
reading of the plan's text edits) gives GREEN with exactly the table counts in
all seven repos, and actionlint stays at 0 findings. `timeout-minutes` is correctly
left off the five `uses:` jobs and is legal on matrix jobs and on the jobs of the
reusable workflow. The concurrency expression does what the plan says for push,
pull_request, schedule and workflow_dispatch. `reproducible-musl-build.yml` has no
`pull_request` trigger, so it correctly gets no C, and none of its callers gets C
either, so there is no caller/callee group deadlock. Workflow `name:`s are unique
within each repo, so no two workflows share a group. No PR-triggered job has a
side effect on PRs: engrave `assemble` and publish are tag/dispatch-only, gui
`github-release` and the doc release steps are tag-only. `sibling-pin-check`'s
regex needs `cargo install … --git … --tag` on one line, so the K comments and
`env:` blocks cannot match it. `ci/doc-gate-guard.test.sh` reads only the guard
`run:` line and examples.yml's `if:` shape, and K/X/T change neither. The gui
`tests/schema_mirror.rs` substring asserts still hold after C/T. The `if:`
scalars in K1/K3 parse cleanly (actionlint clean). Timeouts: the slowest job of
any kind I found in the rare workflows (release, man-pages, musl-binaries,
man-release, repro-drift, reproducible-musl-build, gui build) is 11.8 min. The
QEMU aarch64 repro legs reach 11.1 min.

---

## C1 — Critical — the gate reports GREEN with K edits missing, misplaced, extra or with arbitrary content

**Location:** gate lines 106–120 (`K_STEPS` removed by name anywhere; K-env
popped wherever found); plan lines 228–229 ("It proves … that every planned edit
is present") vs 249–251.

**Failure scenario (reproduced):** I applied the full planned toolkit edit to a
scratch clone (GREEN: `T=42 X=6 C=12 K-steps=8 K-env=9`), then made one mutation
per run. Every mutation came back **`GATE: GREEN`**:

| # | mutation | gate counts | real effect |
|---|---|---|---|
| M1 | one K2 `env: CARGO_INSTALL_ROOT` dropped (quickstart mk-cli) | K-env=8 | that CLI is never cached |
| M2 | `Put cached sibling CLIs on PATH` dropped from manual.yml | K-steps=7 | `MD_BIN=md` → not found, required `manual` check goes red |
| M3 | PATH step's `run:` replaced by `curl … \| sh` | unchanged | arbitrary code in a required check |
| M4 | extra `Restore cached sibling CLIs` step added to rust.yml | K-steps=9 | unplanned step |
| M5 | `CARGO_INSTALL_ROOT` added to cross-tool-differential (deliberately excluded) | K-env=10 | unplanned K |
| M6 | duplicate `run: echo PWNED` key inserted above an existing `run:` | unchanged | PyYAML keeps the last value; **actionlint does catch this one** (`key "run" is duplicated`) |
| M7 | restore step moved below the installs | unchanged | cache never restored before use |
| M8 | docbins restore `path:` changed to `~/.cargo` | unchanged | wrong directory cached |

T, X and C are fully enforced, because every eligible job, step and file must
carry them. K is enforced only through counts that the gate prints but never
compares with anything. The plan's own stop rule ("a count differs from the
table") relies on a human comparing the counts, and M3, M7 and M8 do not change
any count at all. The plan's caveat says K *contents* are unchecked, but the
gate also does not check K *presence*, *position*, *which steps*, or *which
files*. And line 228's claim that every planned edit is present is false.

**Fix (pick one; the first is strictly stronger):**
1. Invert the gate: **build the expected YAML from the base** by applying the
   planned edits at the parsed level (T on every `runs-on` job; X on the six
   texlive steps; C on the eligible files; in `--toolkit` mode, the exact K1 step
   dicts inserted immediately before `Install mk-cli` in jobs
   `manual`/`quickstart`/`technical-manual`, with `<FILE>` substituted; exact
   K2 `env` on exactly those nine steps; exact K3 dicts immediately before their
   two anchor steps in jobs `verify-examples`/`verify-examples-gui`). Then assert
   `new == expected`. That closes M1–M5, M7 and M8 at once, and the
   comment-only blind spot stays as documented.
2. Minimum: hardcode the table (`EXPECTED = {"mnemonic-toolkit": dict(T=42, X=6, C=12, K_steps=8, K_env=9), …}`,
   keyed by a required `--repo <name>` argument) and go RED on any difference.
   Also compare each stripped K step against its exact planned dict, require it
   in the planned job and position, and allow K-env only on the nine named
   steps. Then reword plan line 228.

## I1 — Important — five of the release.yml files are generator output; T is reverted on the next regeneration

**Location:** plan line 55 ("Edit only files under `.github/workflows/`") and T
(plan lines 59–83), applied to `release.yml` in descriptor-mnemonic,
mnemonic-secret, mnemonic-key, mnemonic-transaction and mnemonic-toolkit.

**Failure scenario (reproduced):** `mnemonic-engrave/scripts/release-workflows/emit.py`
writes those five files from the template in `gen.py` (its README: "one generator
so the four files cannot drift"). I ran the base generator into scratch, with the
output path redirected so nothing was written into any checkout. All five
committed `release.yml` files are **byte-identical** to its output today. The plan
hand-adds `timeout-minutes: 60` to their jobs (part of T: dm 16, ms 20, mk 13,
mt 4, tk 42) and leaves `gen.py` alone. The next `python3 emit.py`, for example
for a toolchain or pubkey bump, rewrites all five without the timeouts. Nobody
runs this gate at that point, so the release jobs silently fall back to 360 min.
The plan also creates exactly the drift that the generator exists to prevent.

**Fix:** add `mnemonic-engrave/scripts/release-workflows/gen.py` to the
mnemonic-engrave edit set: insert `    timeout-minutes: 60` directly below each
`    runs-on:` line in its template. Add a gate step: run the edited generator
with its output redirected to a scratch directory, and `cmp` each result against
the edited `release.yml` in the matching worktree. All five must be identical.
State the merge order (the generator PR and the five outputs are mutually
consistent only as a set). Widen plan line 55 to name this file.

## I2 — Important — the toolkit setup steps conflict with what already exists

**Location:** plan lines 49–51, 219–220 and 258–261.

**Failure scenario:**
- Line 50 tells the implementer to run `git worktree add -b ci/hardening-2026-10-11 <path> <base>` "in every repo". For the toolkit, the worktree `/scratch/code/shibboleth/tk-worktrees/ci-hardening` and the branch `ci/hardening-2026-10-11` already exist (`git worktree list` shows them at 2fe6cf38). The command fails, and the plan does not say what to do then.
- Line 258 says commit 1 is "this file, as reviewed". But 2fe6cf38 already exists, titled "design: CI hardening plan and its diff gate (not yet reviewed)". Amend it, add a new commit, or leave it? Two implementers would do different things, and per-fold commit hygiene depends on the answer.
- Line 219 says to *copy* `_work/ci-hardening/ci-hardening-gate.py` into the worktree. After C1 is folded into the worktree copy, a literal implementer copying from `_work` overwrites the fixed gate with the false-PASS one, and the GREEN lines it then pastes into commit messages mean nothing.

**Fix:** state that the toolkit worktree, branch and commit 1 already exist (name
2fe6cf38), and give the policy for plan or gate revisions after review (for
example: one new commit per fold, never amend). Delete the copy instruction and
name the worktree copy as the only authoritative gate. Restrict line 50 to the
six other repos.

## I3 — Important — "a pin, feature **or rustc** change still reinstalls" is false

**Location:** plan lines 33–35 (Why, "cargo tracks source, version, features and
rustc"), and the K1 step comment at plan lines 147–148, which will be committed
into three workflows.

**Failure scenario (reproduced):** I made a tiny git-tagged bin crate and ran
`cargo install --git file://… --tag v1` into one root with explicit `RUSTC`.
With rustc 1.85.0 it compiled and installed. Running it again with cargo/rustc
1.88.0 printed `Ignored package … is already installed`. `.crates2.json` records
`rustc 1.85.0`, but cargo does not compare it. (An earlier attempt without explicit
`RUSTC` was invalid, because the system rustc 1.98.1 was picked up for both runs.)
Consequences:
- K1: if the toolkit bumps `rust-toolchain.toml` (1.85.0 today), the key, which hashes only the workflow file, still hits. The CLIs stay built by the old rustc, and the comment says otherwise.
- K3 guibin: the job sets `RUSTUP_TOOLCHAIN=stable`. Across stable releases the cached `gui-render` stays built by whatever stable created the cache, until `manual-gui.yml` or `pinned-upstream.toml` changes.

The functional risk is low. The defect is a guarantee, stated in committed text
and in the plan's evidence, that does not hold.

**Fix:** remove "or rustc" from the K1 comment and from line 34. Add
`'rust-toolchain.toml'` to the K1 `hashFiles(...)` so a toolchain bump misses.
For guibin, either state in its comment that a new stable does not rebuild, or
key on rustc: add a step `id: rv` running
`echo "v=$(RUSTUP_TOOLCHAIN=stable rustc -V | tr ' ' '-')" >> "$GITHUB_OUTPUT"`
and add `${{ steps.rv.outputs.v }}` to the key. Then update the gate's exact
dicts (C1) to match.

---

## Minor

- **M-1 — the timeout evidence covers only a recent window.** Plan lines 18–20 give "14.0 min" as the max. In the toolkit's `rust.yml`, the job `musl build+test (aarch64-unknown-linux-musl)` (QEMU `cross test`) had a median of 20.7 min and a max of **24.2 min**, all successes, over 2026-06-24…09-26 (run 36210086244: step `cross test` 1326 s). Since 2026-09-26 it takes about 5 min. 60 min still gives 2.5x headroom over the historical max, so the value is fine. But the "over 4x" sentence will be copied into seven PR bodies. Fix: quote the all-time max with its date, or say the figure covers recent runs only.
- **M-2 — "exits at once" / 0.29 s does not carry over to CI.** Even on a hit, cargo prints `Updating git repository` and fetches the sibling repo into a cold `~/.cargo/git`. That is about 17–23 MB for mk/ms/md and 85 MB for the toolkit in verify-examples. The `--locked` installs also do the yanked-check index query. The saving is still large, but say "seconds" rather than "at once". The plan's post-merge measurement will show the real number.
- **M-3 — the cache key has no runner image identity.** Run 38135443590 carries the notice "ubuntu-latest label will migrate to Ubuntu 26 beginning October 19, 2026". The keys use only `runner.os` (`Linux`). During a mixed rollout, a root built on a newer-glibc image can be restored on an older one, which risks `GLIBC_2.x not found` on a required check. I did not reproduce this, and pure-Rust binaries may not need any newer symbol versions. Fix: have a step write `echo "img=$ImageOS" >> "$GITHUB_OUTPUT"` and add it to the three keys (or put it in the PATH step, ordered before the restore).
- **M-4 — K1 placement is ambiguous in technical-manual.yml.** Directly above `- name: Install mk-cli` (base line 123) sits a 5-line section comment (`# ---- Real-bin output-fidelity gate …`) that introduces the installs. "Directly above the step" can mean between that comment and the step, or above the comment. Two implementers would produce different diffs, and the gate cannot see the difference. Fix: say which one, for example "above the `# ---- Real-bin output-fidelity gate` comment line".
- **M-5 — the gate base can move.** Plan lines 224–225 run the gate against `origin/<base>` after a fetch. Toolkit master moves often. A moved base gives a spurious RED (the cause is not the implementer's edit, so the plan says stop), never a false GREEN. Fix: pass `$(git -C <worktree> merge-base HEAD <base>)` as the base ref.
- **M-6 — cancelled manual-gui runs report failure.** The aggregator job `manual-gui` (`if: always()`) still runs when concurrency cancels a run, and it exits 1 on the `cancelled` results. So each superseded manual-gui PR run shows as **failed** (with failure notifications) rather than cancelled. It is harmless for the head SHA. Document it, or note it as a follow-up (`if: ${{ !cancelled() }}`). Not in this plan's scope.
- **M-7 — `actions/cache@v4` adds a warning annotation.** It targets Node 20: run 38135443590 shows "being forced to run on Node.js 24: actions/cache@v4". The plan copies it into four more steps, so each doc job gains this warning. Consistent with bitcoind-differential, and acceptable. Note it so the "no new warnings" reading of the PR is not surprising.

## Nit

- **N-1:** the gate accepts `cancel-in-progress: 1`, because Python's `1 == True`. actionlint covers it.
- **N-2:** `shellcheck` is not installed on this box, so "actionlint: no new findings" does not lint any `run:` script, including the new PATH step.
- **N-3:** the branch prefix `ci/` matches the `push: branches: ['ci/**']` staging triggers. Each PR therefore also gets uncancellable push runs, double CI on a waste-reduction PR. The plan's cache-hit measurement (line 283) does not say whether to re-run the push run or the PR run (each has its own cache scope).
- **N-4:** the commit-message template line "`<one line each for T / X / C / K, with the count>`" does not say how to present K's two counts (K-steps, K-env).

---

Reproduction material (scratch, not committed):
`/tmp/claude-1000/-scratch-code-shibboleth/0ea7914a-eebe-4880-89b7-c21247ab7cc0/scratchpad/`.
`bin/apply.py` is my literal-plan edit simulator, `tk/` is the toolkit scratch
clone, `green/` is the GREEN tree used for the mutations, `rw/` and `rwout/` hold
the redirected engrave generator and its output, and `hello/` is the rustc
reinstall test crate.
