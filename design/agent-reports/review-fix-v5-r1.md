**⚠ r1: 0C/4I/4M/3N. The build itself is green on origin/master as printed (30/30 rows, 4215/4215 suite, manual audit, examples, GUI phase 11); the four Importants are doc steps: F3 never appended, header/§10 still unruled, §7 item 10 fails where §6 runs it, and the pinned sibling CLIs.**

# Review r1: FIX_v5_wsh_cost.md (+E9), fix/v5-wsh-cost 5403688b

Reviewer: independent, Opus. Brief: coordinator/briefs/review-fix-v5-r1.md.
Fold under review: 92c93bf6 (architect fold, Haiku) + 5403688b. Author doc commits before 835cba39.

## Findings (appended as found)

## What I ran (build of the doc's own steps on `origin/master`)

Scratch worktree `/scratch/cache/review-fix-v5-r1/build` (detached from the shared repo, own `target/`), logs in
`/scratch/cache/review-fix-v5-r1/logs/`. The header/§6 cherry-pick line was run as printed, except the branch name
(`review-r1/build-v5-wsh-cost`, so the builder's `git switch -c build/v5-wsh-cost` still works), with
`<reviewed fix/v5-wsh-cost head>` = 5403688b: 3 + 18 commits, no conflict. cargo 1.85.0; miniscript pin unchanged on
master (ff4732e5).

| step | command (as printed) | measured |
|---|---|---|
| §6 step 1 / §3.1 RED | §3.1 filter | exit 100, `30 tests run: 11 passed, 19 failed, 4223 skipped`; the 19 are rows 1-9, 17-22, 26-28, 30 |
| §3.2 before B | §3.2 | exit 100, `12 tests run: 5 passed, 7 failed, 0 skipped` |
| §6 step 2 | `git apply $E/fix-e9-production.patch`, §3.2 | exit 0, `12 tests run: 12 passed, 0 skipped` |
| §6 step 3 | A, C, D, E apply; both `git checkout --` reverts; `cargo build --bin mnemonic`; §3.1 | all apply; exit 0, `30 tests run: 30 passed, 4223 skipped` |
| §7 item 3 | full nextest | `4215 tests run: 4215 passed, 38 skipped` |
| §7 items 1, 4, 5 | fmt (+1.95.0), doc tests, clippy | exit 0 each |
| §7 item 7 | `bash ci/doc-gate-guard.test.sh` | `158 passed, 0 failed` |
| §7 item 8 | `make audit ...` with `~/.cargo/bin` mk | **exit 2**, `[lint] FAILED` (10 "stale marker in 44-mk-cli.md" lines): see I-3 |
| §7 item 8 | same, pinned `mk-cli-v0.13.0` first in PATH | exit 0, `[lint] OK`, `[verify-examples] OK (62 transcripts pass)`, anchor-check OK |
| §7 item 9 | doc-flag-lint mutation test | `12 passed, 3 failed` with `~/.cargo/bin` mk; `15 passed, 0 failed` with the pin |
| §7 item 10 | gen.sh + `git diff --exit-code` | regenerated file == Appendix E's result (`cmp`); **exit 1 when run where §6 step 3 runs it (Appendix E unstaged)**: see I-2 |
| §7 item 12 | `make lint` (manual-gui) with a `mnemonic-gui-v0.63.0` checkout | phase 11 `verify-tutorial-transcripts: OK (95 artifacts byte-identical ...)`, phase 10 OK; phase 4 fails only because `make html` had not run (CI runs html first) |
| figures | 42-md recipe, Appendix G input, J4 step-20 input on the patched binary | 79/85/+6 and 80/86/+6; Appendix G byte-identical; J4 144/+3, 126/+6, 133/+6 (F4's prediction holds) |

The sh(wsh) note's advice, followed literally: 69 + 35 = 104 vB = ceil(413/4) (Core sh(wsh(pk))); 104 + 35 = 139 =
556/4 (Core sh(wsh(multi))). 35 vB = 140 WU exactly, so per-row rounding cannot break it. Δ: 6 - 35 = -29 = 75 - 104.

The single-site claim holds: `plan.witness_size()` has one caller in `crates/*/src` (`cost/enumerate.rs`). F3 and
F4 text in §9 is byte-identical to the architect's fenced texts (python compare).

## Important

### I-1 [fold 92c93bf6] F3 is never appended: the held self-check (rulings 5-6) loses its only tracker
- **Where.** `FIX_v5_wsh_cost.md:364` (§6 step 3 revert bullet: "append F4 (§9)") and `:370` ("Append F2 (§9) to
  `design/FOLLOWUPS.md`"). No §6 step names F3. §9 labels only F1 (`:464`) and F2 (`:474`); the F3 (`:487`) and F4
  (`:494`) entries have no label line and no fence, unlike F1/F2.
- **Failure.** A builder following §6 literally appends F1, F2 and F4. F3 (`compare-cost-wsh-cost-self-check`) is the
  architect's "§9 new F3, appended in the V5 GREEN commit" (STOP-1 text 6): the only record that rulings 5 and 6 (an
  Important-graded deliverable of this FIX) moved to the next miniscript cycle, and the only place that says to drop the
  #1046 double count when the pin moves. Without it, A2 is silently dropped; F2's Build impact (4) then points at a slug
  that does not exist in FOLLOWUPS. The unlabelled, unfenced F3/F4 also leave the builder to choose the exact text to
  copy (where the entry ends: F4 runs straight into `## 10`).
- **Fix text.** (1) §6 step 3, revert bullet: delete "; append F4 (§9)". (2) §6 step 3, replace "Append F2 (§9) to
  `design/FOLLOWUPS.md`." with "Append F2, F3 and F4 (§9) to `design/FOLLOWUPS.md`, in that order, each copied from
  inside its fence." (3) §9: insert before the F3 heading the line
  "F3 (A2, held by the STOP-1 ruling; appended by §6 step 3, status `open`):" and before the F4 heading
  "F4 (the STOP-5 ruling; appended by §6 step 3, status `open`):", and wrap each entry (heading through its `- Tier:`
  line) in a ```` ``` ```` fence as F1/F2 are. `doc-gate.sh` checks only `<!-- evidence -->` `~~~~` fences, so it is
  unaffected (read: `doc-gate.sh:37-70`).

### I-2 [fold 92c93bf6] The header and §10 still say Part A waits and Part B ships alone; §6 says both are ruled
- **Where.** `:5-6` "**Status: AUTHOR DRAFT, not reviewed. Part A (V5) waits on STOP-1. Part B (E9) is complete; per
  grade e9 ruling (a) it ships alone, with the same text, if Part A waits (STOP-2 decides its branch and file name).**";
  §10 heading `:500` "undecided points; the builder does not choose", with STOP-1 `:552` "Options (none chosen)",
  STOP-2 `:568-575` "Undecided: the branch name; the FIX file name; the base", STOP-5 `:605` "Options (none chosen)",
  STOP-7 `:627-629` "Undecided: whether this FIX builds after FIX-V7's GREEN ...". None carries a "Ruled" line
  (the fold report lists STOP-2's as unwritten). Appendix A `:635` "Held by STOP-1 (a) until the ruling."
- **Failure.** Against §6 step 3 ("STOP-1 is ruled option (a) and STOP-5 option (ii)"), a builder has two readings: build
  A+B in one PR (§6), or obey the Status line and §10 (Part A waits; ship E9 alone on a branch STOP-2 leaves open). The
  second contradicts the STOP-2 ruling (one PR) and leaves the builder to choose a branch name and file name, which the
  doc says it must not. STOP-2's ruled fallback (`build/e9-wrapper-chain`, coordinator's call only) is nowhere in the doc.
- **Fix text.** Replace `:5-6` with: "**Status: r1 reviewed. STOP-1 (option (a)), STOP-2, STOP-5 (option (ii)) and STOP-7
  are ruled by `architect-2026-11-04-fix-v5-stops.md`; STOP-3, STOP-4 and STOP-6 are moot (they move with A2 to F3).
  Parts A and B build together, in one PR from `build/v5-wsh-cost` (§6).**" Under each STOP heading insert, as its first
  line:
  - STOP-1: "**Ruled (architect 2026-11-04): option (a).** A2 moves to F3 (§1 A2, §9 F3). The text below is the record."
  - STOP-2: "**Ruled (architect 2026-11-04): E9 ships with V5 in one PR from `build/v5-wsh-cost`; §6 step 2 stays its own
    commit.** Fallback, the coordinator's call only, if the FIX review leaves Part A at C/I after Part B is clean: branch
    `build/e9-wrapper-chain` from `origin/master`, add the file and Appendix B and F1 by the steps below, cite this FIX at
    its `fix/v5-wsh-cost` SHA, and skip §6 step 2 in the V5 PR. The text below is the record."
  - STOP-3, STOP-4, STOP-6: "**Moot under the STOP-1 ruling; F3 rules it before A2 builds.**"
  - STOP-5: "**Ruled (architect 2026-11-04): option (ii).** §6 step 3 reverts both GUI hunks; F4 owns them."
  - STOP-7: "**Ruled (architect 2026-11-04): build on `build/v5-wsh-cost` from `origin/master` (header, §6); V5 and V7
    are independent PRs.** Order: whichever PR is first green with 0C/0I merges first. The second merges `origin/master`
    in, re-runs all of §7, regenerates `.examples-build/Examples.md` with `gen.sh` (§7 item 10) and resolves the
    `design/FOLLOWUPS.md` append." (This also anchors the "Order:" paragraph the fold could not place.)
  - Appendix A `:635`: replace "Held by STOP-1 (a) until the ruling." with "Applied by §6 step 3 (STOP-1 option (a))."

### I-3 [pre-existing, 19bc8836] §6 step 3 runs §7 item 10 before the commit, where its `git diff --exit-code` fails
- **Where.** `:367-369` "Check the transcripts against the patched binary: §7 items 8 and 10. A replayed transcript
  that differs from Appendix D, or a regenerated `Examples.md` that differs from Appendix E's result, is a FAIL",
  then `:371` commit. Item 10 (`:413-415`) is `gen.sh > Examples.md`, then `git diff --exit-code --
  .examples-build/Examples.md`, "Expected: no diff after Appendix E is applied".
- **Failure (run).** At that point Appendix E is applied but unstaged, so `git diff` (worktree vs index) shows Appendix E
  itself: `item10 as printed before commit: exit 1` (measured on the build tree), although the regenerated file equals
  Appendix E's result byte for byte (`cmp`: SAME). The printed check reports FAIL on a correct build; the builder either
  stops or decides on its own that the diff is fine.
- **Fix text.** Replace the `:367` sentence's first part with: "Check the transcripts against the patched binary: §7
  item 8; then `git add .examples-build/Examples.md` and run §7 item 10 (its `git diff` compares the worktree with the
  index, so an unstaged Appendix E is itself a diff)." Measured: after `git add`, gen.sh then `git diff --exit-code`
  exits 0.

### I-4 [pre-existing, 19bc8836] §7 items 8-9 cannot pass on this machine's PATH, and the doc gives no way to get the pins
- **Where.** `:401-402` "with the pinned `mk` (`mk-cli-v0.13.0`), `md` (...) and `ms` (`ms-cli-v0.20.2`) first in PATH",
  items 8 (`:405`) and 9 (`:410`).
- **Failure (run).** `~/.cargo/bin/mk` prints `mk 0.13.0` but is a newer build (it has `--in`, `--from-md1-set`). With it,
  item 8 exits 2 (`[lint] FAILED`, 10 lines `stale marker in 44-mk-cli.md (mk encode): --in is defined by the pinned
  mk-cli 0.13.0`) and item 9 prints `12 passed, 3 failed` (`FAIL baseline: unmutated manual fails flag-coverage`), on a
  correct tree. `--version` cannot tell the builder its mk is not the pin, and the shared `/scratch/cache/ci-tools/bin`
  holds md 0.20.3 and ms 0.20.1 (not the pins). Papercuts records this four times (2026-10-06, -10-11, -10-15, -11-01).
  The fault fails closed, but a printed gate command fails as printed, so it stays I.
- **Fix text.** Replace the `:401-402` preamble clause with: "with the pinned CLIs first in PATH. `--version` does not
  identify them (a newer mk also prints `mk 0.13.0`), so install the tags into a fresh root:" and the block
  ```
  R=/scratch/cache/tk-v5-build-ci-tools
  CARGO_TARGET_DIR="$R/target" cargo install --root "$R" --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.13.0 mk-cli
  CARGO_TARGET_DIR="$R/target" cargo install --root "$R" --git https://github.com/bg002h/descriptor-mnemonic --tag descriptor-mnemonic-md-cli-v0.20.4 md-cli --features cli-compiler
  CARGO_TARGET_DIR="$R/target" cargo install --root "$R" --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.20.2 ms-cli
  export PATH="$R/bin:$HOME/.cargo/bin:$PATH"
  mk encode --help | grep -c from-md1-set   # prints 0 on the pin
  ```
  Run (with `R=/scratch/cache/review-fix-v5-r1/pins`): each install exit 0; `mk 0.13.0`, `md 0.20.4`, `ms 0.20.2`; the
  grep prints 0; then item 8 exits 0 with `[lint] OK` and `[verify-examples] OK (62 transcripts pass)`, and item 9
  prints `15 passed, 0 failed`.

## Minor
- M-1 [fold residue: STOP-7 measured quotes] §3.1 `:224-225` and header `:24-30` quote ac3ff70a-era counts
  (`4235 skipped`; full suite `4227 run: 4216 passed, 11 failed`; fmt drift only in the V7 file). On `build/v5-wsh-cost`
  I measured `30 tests run: 11 passed, 19 failed, 4223 skipped`, then `30 passed, 4223 skipped`, and the full suite
  `4215 tests run: 4215 passed, 38 skipped`, fmt clean. Not C/I: the doc already makes the matched count (30) the
  invariant and §6 step 1 elides the skipped count. Fix (build-owned): in both §3.1 bullets replace `4235 skipped` with
  `<N> skipped` and add "the skipped count moves with the base; quote the measured line (review r1 measured 4223 on
  `origin/master` 12e013bc)"; in the header mark the full-suite and fmt bullets "measured on `ac3ff70a`; on the build
  base §7 items 1 and 3 expect exit 0 and 0 failed".
- M-2 [fold residue] Stale STOP text that no longer matches the ruling: `:72` "written by the builder once STOP-1 is
  ruled" (-> "moved to F3 (§3.3)"); `:242-245` duplicated "No code exists for them" and "written by the builder once
  STOP-1 (and STOP-6 ...) are ruled" (-> "F3 writes them, after it rules STOP-6; the ruling may rename or drop them");
  `:360-361` dangling "Under any other ruling, the ruling's text replaces this step ..." (delete); `:378` "STOP-7 decides
  the base" (-> "The base is `build/v5-wsh-cost` (STOP-7 ruling)"); `:425-427` "with Appendix D's tutorial hunk applied it
  fails ... and STOP-5 decides what passes here" (-> "§6 step 3 reverts that hunk (STOP-5 (ii))"); `:29`, `:37` header
  parentheticals. Not reachable as a wrong build: each sits next to text that rules it, and §3.3's "(and STOP-6)"
  condition is unmet, so a literal builder writes no self-check rows.
- M-3 [fold residue] STOP-7 "Order:" paragraph and the second-PR duties are not in the doc (fold report: no anchor).
  Fails closed: if V7 merges first, `examples.yml` on the PR merge ref and the FOLLOWUPS append conflict surface
  red. Fix text: carried in I-2's STOP-7 "Ruled" line.
- M-4 [pre-existing] §7 item 12 `:421-424` lists `make lint` before `make html` and omits the upstream checkout. Run as
  ordered, lint phase 4 fails ("HTML build not found ... Run `make html` first") and phases 4, 5, 9-12 fail without a
  `mnemonic-gui-v0.63.0` checkout at `MANUAL_GUI_UPSTREAM_ROOT` (default `../mnemonic-gui` beside the worktree). CI runs
  html first (`manual-gui.yml:156`, lint `:160`). Fails closed. Fix: "From `docs/manual-gui`, with
  `MANUAL_GUI_UPSTREAM_ROOT` set to a `git worktree add --detach <dir> mnemonic-gui-v0.63.0` checkout: `make html`, then
  `make lint ...`". With that checkout I measured phases 9, 10 and 11 OK (`verify-tutorial-transcripts: OK (95
  artifacts byte-identical ...)`), which is the check STOP-5 (ii) needs.

## Nit
- N-1 [pre-existing] §5.2 item 2 (`41-mnemonic.md` "tr costs `+6` ... because the tr witness carries an extra 33-byte
  control block"): still true as the dominant term (control block 34 WU, offset by the 64-byte Schnorr signature and
  x-only keys, net +24 WU = +6 vB). Optional: "..., only partly offset by its shorter Schnorr signature and x-only keys".
- N-2 [fold residue] `:89` "Do not apply (STOP-1)", `:993` "STOP-1 holds A2", `:252` "STOP-4 decides ...": meaning still
  right (Appendix F and §3.3 belong to F3); could read "(F3)".
- N-3 [pre-existing] Line cites are at `ac3ff70a`; on master the compare-cost section of `41-mnemonic.md` is 2 lines lower
  (master's #133 edits :194 and :1537); the patches apply with offset.

## Residue grading (fold report "Residue for FIX review")
| residue item | grade |
|---|---|
| Header Status line | I-2 |
| `:72` self-check rows "once STOP-1 is ruled" | M-2 |
| `:89` "Do not apply (STOP-1)" | N-2 |
| §3.3 `:243-244` + duplicated sentence | M-2 |
| `:252` STOP-4 | N-2 |
| `:427` item 12 "STOP-5 decides" | M-2 |
| `:635` Appendix A "Held by STOP-1 (a)" | I-2 (fix text there) |
| `:993` "STOP-1 holds A2" | N-2 |
| §6 step 3 dangling sentence | M-2 |
| STOP-2 "Ruled" line unwritten (and STOP-1/5/7) | I-2 |
| STOP-7 §3.1/§3.2 measured quotes | M-1 (§3.2's `12 tests run: 5 passed, 7 failed, 0 skipped` / `12 passed, 0 skipped` re-measured identical on the build base) |
| STOP-7 "Order:" paragraph | M-3 (text in I-2) |
| NEW_BASE dropped the not-under-/tmp rule | not a finding: 5403688b restored it in §6 `:347` |
| F3 never appended (not in the fold's list) | I-1 |

## Not findings (checked)
- Appendix A's formula equals the #1046 expression and the only `witness_size` caller; Core rows 1-14 pass; F4's
  predicted J4 figures hold on the patched binary.
- Appendix B: `is_wrapper_chain_at` slices `haystack[i + len..]` only after an ASCII-needle byte match, so the index is
  always a char boundary (no panic on non-ASCII input). Its collect and replace sides agree on `sdv :older` (row 26
  passes with all three spellings).
- Mutation gate: not applicable (no production Rust on the FIX branch). Deferred minors: the FIX changes no FOLLOWUPS
  text, so none to re-grade.
- Findings by origin: from the fold 92c93bf6 (Haiku, medium) I-1, I-2, M-1, M-2, M-3, N-2; pre-existing (author
  commits up to 835cba39) I-3, I-4, M-4, N-1, N-3. 5403688b (Opus) introduced none.

Design concern (ungraded): the cherry-pick line carries every `fix/v5-wsh-cost` doc, evidence and review commit into
the product PR; a squash or a doc-only path filter would keep the toolkit history smaller.

C / I / M / N: 0 / 4 / 4 / 3
