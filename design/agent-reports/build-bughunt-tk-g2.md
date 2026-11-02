build: tk 733bb9fd; gate PASS; 0 STOP

Worktree `/scratch/code/shibboleth/tk-worktrees/bughunt-triage`, branch `bughunt/triage`, from `a937b7bc` (G1 built). Local only, nothing pushed. No version change; no edits to `docs/manual-gui/` or `.github/`. Logs: `/scratch/cache/bughunt/tk/build-g2/`.

## Commits per step
| step | commit | closes (FOLLOWUPS 9.1) |
|---|---|---|
| RED rows (FIX 6, 7): rows failing for the FIX's reasons, tests only | `dde06006` | RED half of (3) |
| production code (FIX 3), fmt | `a7903151` | (1); (3) rows turn GREEN |
| spec edits S1-S9 (FIX 4) | `2f7154ed` | (2) |
| manual, technical manual, transcripts via `fix-g2-doc-edits.py` (FIX 5.1-5.4) | `9c2e9b01` | (4) |
| examples corpus regenerated with `gen.sh` (FIX 5.5) | `54180903` | (5) |
| FOLLOWUPS 9.1-9.4 before `bughunt-tk-fix-review-r1-minors` (still last), plus the two 9.5 edits | `733bb9fd` | (6) |

RED run (`red.log`): 34 G2 rows failed on the G1 head, as the brief said (`bughunt_s3` 14, `bughunt_s4` 13 plus 1 in the bin, `bughunt_s5` 6); each failure message was read and matches the FIX's stated reason.

## Gate
| job / check | result |
|---|---|
| fmt, toolchains 1.85.0 and 1.95.0 (`fmt185.log`, `fmt195.log`) | clean (no output) |
| clippy `-D warnings` | clean |
| `cargo nextest run --locked --workspace` | 4138 run, 4138 passed, 0 failed, 38 skipped (`#[ignore]`) |
| `cargo test --locked --doc --workspace` | 0 FAILED; 2 `test result: ok` blocks |
| `manual`: `make audit` (pinned mk-cli 0.13.0 first on PATH; anchor-check 9 danglers = baseline) | pass (`audit2.log`) |
| `manual`: `ci/doc-flag-lint.test.sh` (as CI passes MNEMONIC_BIN etc.) | 15 passed, 0 failed |
| `examples.yml`: `gen.sh` output vs committed `.examples-build/Examples.md` | byte-identical (`cmp`) |
| `technical-manual`: lint and verify-examples | lint OK; 18 transcripts pass |
| `manual-gui`: `make html`, `make lint` (upstream = mnemonic-gui at `mnemonic-gui-v0.63.0`, cloned to `build-g2/gui-pinned`), `verify-examples` | lint OK; 17 transcripts pass |
| `fuzz-smoke.yml` build jobs: `cargo +nightly-2026-04-27 fuzz build` in `fuzz/` and `--fuzz-dir crates/wc-codec/fuzz` | both exit 0 (the 60 s run jobs not run) |
| FIX 6 rows by exact name, `test(/(^|::)<row>$/)`: G1 | 34 listed; 32 run + 2 `#[ignore]` (`bughunt_tk_14_...`, `bughunt_tk_07_...`) |
| FIX 6 rows by exact name: G2 | 37 listed; 21 run + 16 `#[ignore]` |
| run set: all 53 non-ignored rows (32 + 21) | 53 run, 53 passed, 4123 skipped (`rows-run.log`) |
| ignored set (`--run-ignored ignored-only`, 18 matched) | 18 failed (2 G1 + 16 G2), as expected: these are the `#[ignore]`d RED rows of deferred-minor entries, filed with FOLLOWUPS 9.2/9.3 (`rows-ignored.log`) |
| FIX 2 outputs vs the built `mnemonic` via `doc-commands.sh` | 258 lines, identical to the reference after-fix capture `doc-after.txt`; 97 stdout/stderr lines checked for presence in FIX 2 text: 69 found verbatim, the other 28 are long descriptor lines, the unchanged roundtrip diff block on stderr of the Core taproot `--json` command, and `account` error lines that FIX 2 prints as `got <v>`/per-value rows (checked by hand against FIX lines 191, 214-217) |

The git-tree after the gate is clean (the fuzz build rewrote `fuzz/Cargo.lock`; reverted with `git checkout`).

## 9.1 lines closed
(1) `a7903151`; (2) `2f7154ed`; (3) `dde06006` plus `a7903151`; (4) `9c2e9b01`; (5) `54180903`; (6) `733bb9fd`. The entry's own `Status: open` line is left as the FIX gives it.

## Examples-corpus diff (`.examples-build/Examples.md`, one line)
`-  [over_envelope] root: ... (2^(11 keys + 2 hashes) × 9 timelock-states > cap 4096) ...`
`+  [over_envelope] root: ... (2^(11 keys + 1 hashes) × 9 timelock-states > cap 4096) ...`
That is the 5.5 line; G1's fixes changed no other corpus line.

## Not done, or judgement worth a look
- The 60 s fuzz run jobs of `fuzz-smoke.yml` were not run (build jobs only).
- 9.5 edit 1: I read the trailing sentence "The ms primer text ... needs no change." as outside the replacement (code-span boundary). Not judged a product point.
- `gate.rs`: the reference patch left "SCRATCH" comments and did not add the doc comments on `check_cap` / `hash_and_timelock_counts`; I wrote neutral wording and the doc comments the FIX names.
- Local `~/.cargo/bin/mk` is newer than the pin with the same version string; the manual lint needs the pinned build first on PATH (as G1 did).
- Secret-handling: nothing written; no model ids or email in repo files.
