# CI: cache the pandoc + texlive debs (brief toolkit-ci-texlive-cache)

Outcome: the six "Install pandoc + texlive" steps now restore their `.deb` files from `actions/cache` instead of downloading about 900 MB from the runner's apt mirror. On a hit the download is gone (`Need to get 0 B/892 MB`) and the steps take 98 to 126 s. A cold miss is unchanged and still exposed to a slow mirror. Draft PR: bg002h/mnemonic-toolkit#136, branch `ci/texlive-cache`, head b3c97e3b, worktree `/scratch/code/shibboleth/tk-ci-texlive`. Not merged, not tagged. The PR edits `.github/` as authorised by the brief and says so.

## Recon

- PR #134, run 41859043753: `quickstart` "Install pandoc + texlive" hit its 15 min step timeout; the download was 892 MB in 13 min 7 s. Cause: a slow Azure apt mirror, not an install error.
- Over the recent runs sampled (111 install steps across manual, technical-manual, quickstart, examples, manual-gui): median 120 s; 5 steps at 600 s or more, max 823 s. So the failure is a mirror-speed tail, about 1 in 20 steps, not a steady regression.
- Same package lists in all six steps, about 892 MB (975 MB for manual-gui, which adds python3).
- macOS, run 41859043754, `build mnemonic (macos-x86_64|aarch64)`: "job was not acquired by Runner of type hosted" near 15 min. That is runner capacity, a different class from the apt mirror. Report-only; not changed. Those jobs passed on this PR.

## Change

- `ci/apt-cache.sh` (new). `plan <dir> <pkgs>`: `apt-get update` (output to stderr), then `apt-get install --print-uris` against an empty archives dir to list every deb with name, size and hash from the signed index; writes a sorted plan file and prints only `digest=` and `pkgsum=` lines (stdout is appended to `$GITHUB_OUTPUT`). `install <dir> <pkgs>`: the same `apt-get install -y --no-install-recommends` with `Dir::Cache::archives=<dir>` and `Keep-Downloaded-Packages=true`; prunes debs not in the plan; removes root-owned `partial/` and `lock`; chowns the dir so the cache post-step can tar it. A real apt failure still exits non-zero and fails the step.
- `.github/actions/apt-cache/action.yml` (new composite action): plan step, `actions/cache@v4` keyed `apt-<ImageOS>-<pkgsum>-<digest>` with a prefix restore key, install step. `ImageOS` reaches the key through a step output (it is not in the `env` context).
- `manual`, `technical-manual`, `quickstart`, `examples`, `manual-gui`: the six install steps use the action with identical package lists; step `timeout-minutes: 15` and the guard `if` are unchanged. Pins are untouched, so output is byte-identical: same packages, same apt command, and apt re-verifies cached debs against the signed index, so a stale or tampered cache cannot change what is installed.
- Doc-gate guard: each `--also` regex gains `\.github/actions/apt-cache/|ci/apt-cache(\.test)?\.sh$|`, so edits to the new paths run the gate.
- `ci/apt-cache.test.sh` (new, 23 checks, fake `sudo` and `apt-get`), run by a new step in `manual`.

## Test first

- RED seen: the first CI run failed every install step with "Unable to process file command 'output'", because `apt-get update` chatter went into the plan stdout that is tee'd to `$GITHUB_OUTPUT`. The shim test did not catch it at first. Added the check "plan stdout holds only digest= and pkgsum= lines": 22 ok, 1 FAIL on the old script. Fix `apt-get update >&2` (b3c97e3b): 23/23.
- Local gate, run once: `ci/apt-cache.test.sh` 23/23; `ci/doc-gate-guard.test.sh` 158 passed, 0 failed; `ci/code-docs-check.sh` ok; actionlint clean on the five workflows. Earlier mutation plants in the script were caught by the shim test.
- Rust is not touched, so no nextest run.

## Timings (PR runs at head b3c97e3b)

Attempt 1 ran with an empty cache; attempt 2 is a re-run after attempt 1 saved.

| job | attempt 1 (cold) | attempt 2 (warm) | warm log |
|---|---|---|---|
| quickstart | 767 s | 126 s | restored; 0 B/892 MB |
| technical-manual | 728 s | 121 s | restored; 0 B/892 MB |
| manual | 133 s | 107 s | restored; 0 B/892 MB |
| examples | 101 s | 98 s | restored; 0 B/892 MB |
| manual-gui build | 133 s | 126 s | restored; 0 B/975 MB |
| manual-gui lint | timed out, 913 s | 150 s | miss (key not yet saved); 975 MB fetched on a fast mirror, then saved |

Baseline before the change: median 120 s, 5 of 111 at 600 s or more (max 823 s), plus the #134 failure at 787 s. Cold runs were fast or slow by mirror luck (101 to 133 s, or 728 to 767 s); the warm runs are 98 to 126 s because the install and unpack of about 900 MB remains. Warm lint is not yet demonstrated: its attempt 2 was itself a miss.

## Residual risks and follow-ups (none owned by a phase; ownerless)

1. A cold miss still downloads about 900 MB under the unchanged 15 min step timeout; lint's attempt 1 timeout is exactly this. Options not chosen here, a design choice: `Acquire::Retries` and parallel download, or moving the download into its own step with a longer limit. Filed as a suggestion, not done.
2. Cache storage: about 850 to 930 MiB per key, several keys (the manual-gui lint package set differs) against the repo's 10 GB limit with about 1.16 GB used before. Unused caches evict after 7 days, after which the next run is cold. PR-ref and push-event runs have separate scopes, so a PR's first run is cold unless the base branch saved the same key.
3. Real apt behaviour with a restored dir is visible only in CI (the shim test cannot reach it); the warm runs above show it working.
4. macOS runner capacity is unchanged and report-only.
5. The `make`-only and other small apt installs are intentionally left as they were.

0 Critical, 0 Important (no STOP). Counts: 6 install steps converted, 5 workflows, 23 shim checks, 6 of 6 warm re-runs green, 5 of 5 warm cache hits confirmed in logs.
