# CI hardening: whole-diff adversarial review (2026-10-11)

**Verdict: GREEN to merge. 0 Critical / 0 Important / 3 Minor / 7 Nit.**

Scope: the eight draft PRs at these heads: toolkit #102 `a97a80c8`, dm #36 `15df6a36`, gui #39 `892f5dc`, mk #12 `56637f8`, ms #11 `7b470ea`, mt #2 `c2f123f`, engrave #18 `232bd0e1`, refugium #80 `b7aba9ec`. The question: does each head do exactly what the plan says, and does anything in a PR body, a commit message or a comment misstate what a merge does?

## What I checked mechanically (beyond the caller's settled list)

- Each PR's `headRefOid` equals the worktree HEAD. Each merge-base equals the current `origin/<base>` tip (the plan's base SHAs). The toolkit's `.github` diff from the merge-base to HEAD is byte-identical to `608eaf7b~1..608eaf7b`, so the master merge `ff889b2f` touched no workflow.
- **Added-lines whitelist** across all seven diffs (this closes the gate's YAML 1.1, quoting and wording blind spot): every added line is one of the plan's literal lines: `timeout-minutes: 60` / `15`, the six-line C block, the K1/K3 step text, the K2 `env:` pair, the K4 comment, or a blank line. Counts per repo match the plan's table. There is no `6_0`, no re-quoting and no stray line.
- `git diff --check`: no whitespace errors and no CR in any repo.
- C placement: in all 26 files the block sits directly above `jobs:`, with one blank line before `jobs:`. No two workflows in any repo share a `name:`, so `${{ github.workflow }}` groups cannot collide across workflows. No `pull_request` trigger uses `types:`. No `pull_request_target` anywhere.
- K placement: in manual and quickstart, K1 sits directly above `- name: Install mk-cli`. In technical-manual it sits above the `# ---- Real-bin output-fidelity gate` section comment. `env:` sits after each step's comments, directly above `run:`. The K4 line is correct.
- K behaviour against its comments: `DOCBIN_ROOT`/`GUIBIN_ROOT` equal the cached paths (`manual-gui.yml:216,233,244,325,338`). All four DOCBIN installs are `--git --tag` installs, so "the four `cargo install`s" is true. Later steps resolve `md`/`ms`/`mk` by bare name or `command -v` (`manual.yml:181-195`, `quickstart.yml:175-178`, `technical-manual.yml:200-203`), so they pick up the prepended `$RUNNER_TEMP/sibling-clis/bin`. Run 38182692639 logs `Cache restored from key: sibling-clis-ubuntu24-manual-b1775b21…`.
- Commit messages (all seven) match the plan's template and the gate counts. The engrave message carries the five `release-gen ... identical` lines.
- #80, read-only on the host: the live and inactive libvirt XML of `refugium-ci-windows` both have `<topology sockets='1' … cores='12' threads='1'/>` with `<vcpu>12</vcpu>`. `sdc` is `/btrfs_raid0/@vm/vmpool/refugium-ci-windows-cache.qcow2` (100.00 GiB, SATA, `discard='unmap'`), `sdb` is the cdrom, and the `vmpool` target path matches the README's `path=`. The runners API lists `refugium-ci-windows-2` online with `self-hosted,X64,Windows`. In run 38179516742 the Windows job ran on `refugium-ci-windows-2`, from 22:12:55 to 22:19:01 (6 min, success).
- #80 timing claim: job 114587069241 (the first Windows run after the change, 22:03:56) has nextest `Summary [ 216.302s]`. The earlier jobs 114574851406, 114573396585, 114571240398, 114569031775 and 114406096612 show 309.7, 361.3, 324.0, 331.5 and 718.3 s, so "310–718 s before" holds. The instance-2 job logs print `pre-job: E: 97.9GB free, E:\ci\cargo-target-2 0.7GB; keeping` and `CARGO_HOME: E:\ci\cargo-2`, and compile only `mr-cli`/`mr-codec`, so the E: cache persists.
- **Not verified:** the live `E:\ci\actions-runner-2\.env` text and the live `pre-job-2.ps1`. A shell on the VM was not available to me (the permission was denied), so I rely on the body's statement that the hook was diffed against the live file.

## mnemonic-toolkit #102

- **Minor: PR body, line 4: "Only `.github/workflows/` files".** The PR changes 27 files. Eight are outside `.github/`: `design/FOLLOWUPS.md` (+24, three new `deferred-minor` entries), the plan, the 398-line gate script, and five agent reports. *Failure scenario:* the owner approves on the body's word that the merge is workflow-only, and master gains the plan, the gate and the FOLLOWUPS entries without his noticing. The content is intended and harmless, but the claim is false. *Fix:* "Workflow changes are in `.github/workflows/` only. The branch also carries the plan, its gate, the review reports and three FOLLOWUPS entries under `design/`."
- **Minor: `.github/workflows/manual.yml:116-118`, `quickstart.yml:99-101`, `technical-manual.yml:129-131` (comment text the plan dictated).** "so a cache built on one image is never restored on another" overstates. `$ImageOS` is `ubuntu24` (it appears in the restored key above), which is the OS family, not the weekly image build (`$ImageVersion`). A cache built on one ubuntu24 image is restored on every later ubuntu24 image. That is harmless for these pure-Rust CLIs, but the comment promises more than the key provides. *Failure scenario:* a future maintainer trusts the comment, puts something image-version-sensitive in the cached root, and gets a stale restore after a runner-image update. *Fix (later, not in this PR, so the gate stays aligned with the plan):* "…for the cache key, so a cache built on one OS image (e.g. ubuntu22) is never restored on another (ubuntu24)". File it as a FOLLOWUPS `deferred-minor`.
- **Nit: PR body, "Why" bullet 4.** The 0.29 s figure is the local hit. The measured CI hit is 8/3/2 s (13 s against 125 s on the miss, run 38182692639 attempts 1 and 2), and it is only in the implementation report. *Fix:* add the CI numbers.
- **Nit: PR body, K bullet.** "`CARGO_INSTALL_ROOT` on the 9 `cargo install` steps" reads as all install steps. The manual-gui installs use `--root` instead. *Fix:* "on the 9 Install mk/md/ms-cli steps in manual, quickstart and technical-manual".
- **Nit: `examples.yml:87`, `manual-gui.yml:111`.** The X line now separates `- name:` from the comment that explains the step's `run:`. This is cosmetic and is what the plan says ("directly below its `- name:`").

## descriptor-mnemonic #36, mnemonic-gui #39, mnemonic-key #12, mnemonic-secret #11, mnemonic-transaction #2

No findings beyond the shared Nits below. Diffs are exactly T + C. Bodies differ from each other only in counts, and the counts are correct.

## mnemonic-engrave #18

No findings beyond the shared Nits. `gen.py` gains exactly three `    timeout-minutes: 60` lines below the three `    runs-on:` lines of `test`, `build` and `assemble`. The body correctly flags the same-day merge coupling.

## Shared (all seven bodies)

- **Nit:** "The slowest successful job found across the last 300 runs of each repo is 24.2 min" conflates two of the plan's measurements. The 24.2 min came from the last 100 successful toolkit `rust.yml` runs (1,287 jobs). Across the last 300 runs of each repo, nothing else exceeded 14.0 min. *Fix:* use the plan's wording.
- **Nit:** the plan link targets `blob/ci/hardening-2026-10-11/…`. `delete_branch_on_merge` is `false` on the toolkit, so the link survives unless the branch is deleted by hand. *Fix:* point it at `master` after the toolkit merge.
- **Nit:** "on 1 pull-request workflows" (mt, engrave): grammar.

## mnemonic-refugium #80

The commands are correct as written and safe to follow literally. Each one either does what it says or fails without harm: `Initialize-Disk` refuses a disk that is already initialized, and `New-Item` refuses existing folders. The README describes the state the body claims: one socket with 12 cores (confirmed in the live and inactive XML), and a second runner whose cache is on E: (confirmed by the runner list and the job logs).

- **Minor: `ci/self-hosted/README.md:152-165` (step 8, "Give it its own Rust install").** Instance 2's `PATH` is never addressed. In instance 2's job log (job 114603431657, run 38184349980), `taiki-e/install-action` prints `info: cargo is located at /c/ci/cargo/bin/cargo`. So instance 2 runs **instance 1's** rustup proxies from `C:\ci\cargo\bin`, with `CARGO_HOME=E:\ci\cargo-2` and `RUSTUP_HOME=E:\ci\rustup-2`. Toolchains and registries are separate, which is what makes it work today. The proxy binaries are shared, though, and `E:\ci\cargo-2\bin` is never put on `PATH`. That contradicts the README's own Linux rule (step 6: "put that `CARGO_HOME/bin` first on `PATH`") and the "its own Rust install" rationale. *Failure scenario:* a `rustup self update` or reinstall on instance 1 rewrites `C:\ci\cargo\bin\*.exe` while an instance-2 job is executing them, which is the cross-instance breakage step 8 says it avoids. *Fix:* either add `PATH=E:\ci\cargo-2\bin;<rest of the machine PATH>` to the `.env`, and note that it must be set at registration, or state plainly that the proxies on `PATH` are shared and only the homes are separate.
- **Nit: `README.md:219-229`.** The cleanup paragraph now covers three hooks. Line 229 still says "Neither touches…". "It needs the execution policy" (221) no longer has a clear subject. The recovery command at 226 names only `C:\ci\hooks\pre-job.ps1`. *Fix:* "None of them touches"; "Both Windows hooks need…"; add the E: path to the `icacls … /reset` hint.
- **Nit: `README.md:146-147`.** `-Number 1` assumes the new disk is Disk 1. That holds on this VM, and the command fails safely otherwise. *Fix:* "(check with `Get-Disk`; the new disk is the one with PartitionStyle RAW)".

## Fold accounting

This is the first whole-diff round, so no findings came from a previous fix commit. None of the Minors blocks the merge. Under the Refugium review rules, the Minors and Nits go to `design/FOLLOWUPS.md` as `deferred-minor`, not into these PRs.
