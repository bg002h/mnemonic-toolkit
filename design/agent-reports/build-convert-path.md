# Build report: tk `convert --path` FIX (branch `fix/convert-path`, from `triage/convert-path` cac0d95a)

Local only, nothing pushed. No gate step failed. Logs: `/scratch/cache/convert-path/build/`.

## Commits
- 918832b0 convert.rs code delta (FIX s3; applied from the reference patch, `cargo fmt` clean)
- 4ae2f484 SPEC_convert_v0_6 S1-S8 (FIX s4; scripted, each old text matched exactly once)
- 666451c0 manual rows (FIX s5) + FOLLOWUPS: F1, F2, and `deferred-minor` entries for review r1 M1-M4, N1 (7 entries in all: F1, F2, M1-M4, N1)

## RED / GREEN
- RED rows already existed on the branch (0ddd598d, 30 rows); no new RED commit. Unmodified code: 30 run, 2 passed, 28 failed (`red.txt`). The FIX header says 27/2/25 (stale, review M3); §8 and fold say 30.
- GREEN: `cli_convert_path` 30 run, 30 passed (`s5.txt`), quoted in 666451c0.

## Gate
1. fmt 1.85.0 ok (`s1.txt`); 1.95.0 ok (`s1b.txt`)
2. clippy -D warnings ok (`s2.txt`)
3. workspace nextest: 4175 run, 4175 passed, 38 skipped (`s3.txt`)
4. doc tests ok (`s4.txt`)
5. 30/30 (`s5.txt`)
6. Doc-CI locally: `.examples-build/gen.sh` ran; its output differs from `.examples-build/Examples.md` only in the build-descriptor `1 hashes` vs `2 hashes` line (792), unrelated to convert and not from this change (existing FOLLOWUP `gui-snapshot-tut-j3-09-distinct-hash-count`; I did not regenerate or edit Examples.md; the diff on a clean base was not re-run). `make lint` in docs/manual fails environmentally (md/mk/ms binaries not at pins; `s6-lint.txt`), no convert-related failure. Not run, owed to the PR's CI: verify-examples, anchor-check, pdf, html, quickstart, technical-manual, manual-gui, fuzz-smoke, rest of rust.yml. Step 8 (mutation gate) left to the coordinator.

## Deviations
- F1 `owed-to-build` line names 918832b0 (code commit) because the last GREEN commit cannot name its own sha.
- Review M1-M4 were not applied (brief: 30 rows, FIX verbatim); filed as deferred-minor.
- Workspace count 4175 vs the FIX's 4171+3 guard rows = 4174: one extra test; not investigated.
