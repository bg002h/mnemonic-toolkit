# FIX review r1, tk: `design/FIX_convert_path.md` at 4a0c4ffa (fold 0ddd598d)

Reviewer: Opus, effort medium. Scope: architect QT1-QT3 and the STOP-2 doc defect, plus the
brief's (a)-(d) across the doc. Scratch copy: `/scratch/cache/bughunt/mt-fix-review-tk/`
(clone of W at 4a0c4ffa, own target dir; W untouched apart from this file).

## What was run

- **Sonnet-proof (b).** `scripts/apply_doc.py` applies sections 3.1-3.6 and 5.1, using the
  doc's own fenced blocks and anchors only. Every old text matched exactly once. The diff it
  produced has the same +/- lines as `/scratch/cache/convert-path/fix-reference.patch`
  (sha256 `835f1326…cc43`).
- **GREEN (c).** On the doc-applied tree, `cargo nextest run --locked -p mnemonic-toolkit
  --no-fail-fast -E 'binary(cli_convert_path)'` gave 30 run, 30 passed (`green.txt`). The
  controller's RED (30/2/28) and GREEN (30/30) counts match its captures.
- **Plants (c)** (`scripts/plant.py`, which reorders or removes blocks in the fixed `run()`;
  full 30-row run each):

  | plant | rows that fail | reason |
  |---|---|---|
  | `r3first`: R3 before R4 | `r4_before_r3_xprv_to_xpub_path_and_account` | prints the R3 text instead of R4 |
  | `a5first`: 5.a before R4/R3 | `r4_before_xpub_prefix_network` | prints the 5.a text instead of R4 |
  | `nor3`: R3 removed | `r3_before_r5_named_path_with_account` and both `r3_account_*` rows | exits 1 with the R5 text |
  | `a5mid`: R4, then 5.a, then R3 | **none (30/30 pass)** | see M1 |

- **The brief's question about `S` in `r4_before_xpub_prefix_network`.** The fold's row has no
  `--network`, and it does test the R4-before-5.a precedence that the architect ruled: under
  `a5first` it fails with the 5.a text. The architect's literal text (`--from P S …`) cannot
  test it. 5.a needs `--network` to be absent, so with `--network signet` the row prints R4
  under `a5first` too (ran it: rc 2, R4 text). It would just repeat `r4_phrase_to_fingerprint_path_refused`.
  The `S` in the architect's text contradicts its own "(no `--network`)". Leaving it out was
  the right reading, and the doc's last §6 sentence records that choice.
- **Fold check (a).** Every STOP is marked ruled. The final `run()` order is stated and matches
  §3.5. `vpub` has become `zpub`, and the remaining `vpub` (line 380) only explains the
  correction. §3.5 is unchanged. The three optional guard rows are present in §6 and in the
  test file, and §8 and F1 say 30.
- **Correct results (d).** No change to existing encodings beyond what R1-R5 rule. Every new
  refusal fails closed (exit 2 or 1, nothing on stdout). The `reads_path` table matches every
  `args.path` read in `compute_outputs`: lines 1375 (wif), 1430 (bip38), 1471 (seed address)
  and 1532 (xpub address). The xprv arm never reads the path.

## Critical / Important

None.

## Minor

- **M1** (from the fold), §6 last paragraph and §10 STOP-2. The doc says "The three overlaps of
  section 10 are covered by the guard rows". QT2 rules "R4/R3 before 5.a", but only R4-before-5.a
  is guarded. Under the `a5mid` plant (R4, 5.a, R3) all 30 rows pass. The `a5mid` build then
  refuses `--from P --to xpub --path m/84'/1'/0' --account 3 --xpub-prefix zpub` with 5.a, but
  the ruling requires R3. It is Minor because it fails closed (exit 2 either way) and needs
  a builder to depart from §3.5's exact code. Fix: add a §6 row after
  `r4_before_xpub_prefix_network`:
  `| r3_before_xpub_prefix_network (QT2 guard) | --from P --to xpub --path m/84'/1'/0' --account 3 --xpub-prefix zpub (no --network) | 2; R3 text | exit 2, the 5.a text |`.
  Add the matching test, change "30" to "31" in §8 step 5 and in F1, and replace the §6
  sentence with "…and `r4_before_xpub_prefix_network` plus `r3_before_xpub_prefix_network`
  (STOP-2)…". I ran this row as a scratch test (`extra-*.txt`). Unmodified code: fails, the
  stderr has the 5.a text. Fixed code: 31/31 pass. `a5mid` plant: only this row fails.
- **M2** (from the fold), §10 STOP-2, line 380. The doc says: "With `--account 3` and a literal
  path it shows R3". Applied to its own example (`--to fingerprint`), it shows R4. I ran
  `--from P --to fingerprint --account 3 --path m/0 --xpub-prefix zpub` and got rc 2 with the R4
  text. The architect's probe used `--to xpub`. Fix: "With `--to xpub --path "m/84'/1'/0'"
  --account 3` instead it shows R3; without `--account` it shows 5.a." (Probed: R3, then 5.a.)
- **M3** (omitted in the fold), header lines 8, 15 and 16 are stale. They still say "Section 10
  lists the three points the ruling leaves open", "27 run, 2 passed, 25 failed" and "27/27
  passed". §8 and the fold captures say 30 / 2 / 28 and 30/30. A builder who checks their RED
  run against line 15 sees a mismatch. Fix: line 8 → "Section 10 records the three precedence
  points, each ruled by the architect (QT1-QT3)". Lines 15-16 → "…gave 30 run, 2 passed, 28
  failed … (`fold-red.txt`)" and "…30/30 passed (`fold-green.txt`)". Either recompute the
  sha256 values or drop them.
- **M4** (pre-existing), §4 S5. The ruled precedence (R4 before R3 before §11.a's
  `--xpub-prefix` refusal, and R3 before the `--path parse` error) is holder-visible. It reaches
  only the code, not `SPEC_convert_v0_6.md`. S5 says only that R4 is "checked after the wif
  refusal". Fix: append to S5's R3 bullet: "Order: the wif refusal, then the unused-`--path`
  refusal, then this one, then §11.a's `--xpub-prefix` refusal; the `--path parse` error comes
  after all of them."

## Nit

- **N1** (pre-existing): `assert_refused` (cli_convert_path.rs:73) does not check that stdout is
  empty, though §2 says refusals print "nothing on stdout".

## Re-review attribution

This is the first FIX review after the architect fold. Three of the 4M/1N trace to the fold
commit 0ddd598d (M1, M2, M3). Its trailer names Sonnet, and it does not record an
effort. M4 and N1 predate the fold.

Design note (ungraded): in the architect report, the guard-row text for QT2 includes `S`,
which contradicts the "(no `--network`)" beside it. Proof-read future optional row texts
against the precondition they exist to test.

r1: 0C/0I/4M/1N
