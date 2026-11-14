Author: Sonnet 5.5 (model id claude-sonnet-5-5), implementer for the brief build-toolkit-pins-md-0.20.6-mk-0.15.0.

**⚠ Critical/Important: the PR is RED. Four STOPs (S1-S4) leave gates failing; the architect must rule them before review.**

Base: origin/master 13e6643587d1e2a7c78ec7f5ca50acba5879f4cd (did not move). Branch pins/md-0.20.6-mk-0.15.0. PR: bg002h/mnemonic-toolkit#162 (draft). Head SHA: the tip of the branch (the report is its last commit).
Binaries (gates): md 0.20.6 from `md-0.20.6-linux-amd64.tar.gz` (glibc; the musl asset lacks `cli-compiler`: `md compile` says "requires the cli-compiler feature"), mk 0.15.0 and ms 0.20.4 from the `x86_64-linux-musl` assets, sums checked, in /scratch/cache/toolkit-pins-md-mk-logs/rel/bin. Nothing installed over the user's binaries.

## Done
- Pins: install.sh :55 md-cli-v0.20.6, :61 mk-cli-v0.15.0; manual.yml, quickstart.yml, technical-manual.yml (md+mk), cross-tool-differential.yml (md). A `.github/` change (PR says so).
- `44-mk-cli.md:12` prose `cargo install ... --tag mk-cli-v0.13.0` -> v0.15.0 (not in the brief; sibling-pin-check step 1 scans manual prose and requires it).
- `pinned-upstream.toml`: `[installer-ahead]` md -> v0.20.6, new `mk-cli-tag-implied = "mk-cli-v0.15.0"` (no mk row existed; check_cli_pins.py requires it once the installer is ahead). GUI tier unchanged: manual-gui.yml:227/:233, pinned-upstream.toml:112/:114 stay.
- first_signed: md stays 0.20.4 (first signed md release, <= pin). mk CHANGES "" -> "0.14.0": mk 0.14.0 is the first mk release with a .minisig (0.13.0 has none, 0.14.0 has 3), and the pinned 0.15.0 is signed, so install-assets.test.sh requires a non-empty entry <= 0.15.0.
- Exact texts 1, 2, 3 applied word for word (42-md.md C6 paragraph after the `md compile` Flags table; `md address` `--network` row; 44-mk-cli.md pointer sentence).
- Text 3 run: /scratch/cache/toolkit-pins-md-mk-logs/mk_pointer_row.log. `mk address` prints (rc 64) the command, but the script's PASS fails: the fixture cards are testnet, and `md address <MD1>... --from-mk1-file FILE` as printed exits 1 under md 0.20.6 (`md: seating refused: ... its xpub is a testnet key, but --network says mainnet`). With `--network signet` appended: rc=0. See S5.
- Stale comments: install.sh:184 "ms 0.20.2 ... the other four are empty" -> "...; mk 0.14.0 is the first signed mk release, and the pinned mk 0.15.0 is signed too. Only mnemonic-gui's entry is empty (the entries for mnemonic and md were filled by their own pin moves)." pinned-upstream.toml:116 "on two CLIs: ms 0.20.4 ... and md 0.20.4" -> "on four CLIs: mnemonic 0.106.0, md 0.20.6 (...), ms 0.20.4 (...) and mk 0.15.0 (...)".
- Examples golden regenerated; changed lines: 2 table rows (md 0.20.4->0.20.6 tag and asset; mk 0.13.0->0.15.0) and the install dry-run lines for md and mk (download x2, version-check x2). All are md/mk version lines; nothing else.
- Markers: the 10 `(unreleased: mk-cli after 0.13.0: --flag)` markers in 44-mk-cli.md removed (the lint requires it: those flags are in the pinned 0.15.0) and the 4-line explanatory paragraph at :16 deleted (it described markers and named 0.13.0). See S3.
- scripts/install-signature.test.sh: its sed that blanks mk's first_signed matched only `echo ""`; widened to `echo "[0-9.]*"`. Test passes (OK).
- CHANGELOG.md and docs/manual-gui/CHANGELOG.md entries.

## STOPs (not decided by the brief; nothing invented beyond what is marked)
- S1 DRAFTED (item 6): GUI manual §82 `{#installer-ahead}`: intro now names four CLIs; new "Then `mk`, five outputs" block (mk encode ungrouped stdout + stderr card; --separator hyphen/comma refused exit 64 (checked); invalid card exit 1, mk repair 2 (checked rc 1/2); chunk_set_id warnings and correction note (from the 0.14.0 CHANGELOG, not run); mk address refusal now names the md command (checked)); "Last `md`, seven outputs": three new bullets (>8 `or`s refusal (checked), other-network mk1 card refusal (checked), `md encode` decompose remedy lines (from CHANGELOG, not run)). Also lines 24-26 and 40-41 counts. Architect rules before review.
- S2 uncovered manual quotes of changed mk text (not edited): 44-mk-cli.md:47 `--group-size` "insert a separator ... in each emitted `mk1` string" and :48 `--separator <space|hyphen|comma>` "keyword or the literal `-` / `,`": mk 0.14.0 refuses hyphen/comma (exit 64) and moves grouping to the stderr card. I did not audit every other 0.14.0 change (exit-code tables, worked examples) beyond the failing transcripts in S4; the list is likely incomplete.
- S3 DRAFTED: deleting the :16 marker paragraph is my call; with no marker left it is dead text. Alternative: keep it with "0.15.0".
- S4 GATE RED, transcripts: `make audit` (manual) lint OK incl. flag-coverage, then verify-examples FAILS (8 mk transcripts: 44-mk-address-json/-text, 44-mk-decode-json/-text, 44-mk-derive-text, 44-mk-inspect-text (stdout `chunk_set_id: 7b86c (stamped)` line and stderr), qs-26-recover-mk1); the quickstart verify-examples replays the same pool (same 8); technical-manual verify-examples fails 2 (mk1-decode-bip48-multisig, mk1-decode-bip84-no-fingerprint). Cause: mk 0.14.0 warns on stderr when a card's stamped chunk_set_id is not the derived one ("warning: this key card's stamped chunk-set id (7b86c) was not derived from its content, which computes e172a. ..."), and the fixture cards are stamped. Fix choices (regenerate transcripts, or re-mint the fixture cards) are the architect's. Not run on unmodified master with mk 0.15.0 because master pins 0.13.0 (it passes there by construction).
- S5 text 3 as ruled does not run as printed on non-mainnet cards (see above); the manual sentence has no `--network`. Architect to rule whether the sentence or only the mk advisory should mention it.
- S6 GATE RED, `ci/doc-flag-lint.test.sh`: its mutations R2, R4, R6, R7 and its baseline ("flag exemption(s)") anchor on `(unreleased: mk-cli after 0.13.0: --in|--keys)`, which no longer exists (no unreleased mk flag remains). First failure: "mutation anchor not found in 40-cli-reference/44-mk-cli.md: '(unreleased: mk-cli after 0.13.0: --in)'". Needs a redesign of the test's fixtures (architect's).
- S7 install-signature.test.sh edit (above) is mine; flagging it as a judgement.

## Gates (logs in /scratch/cache/toolkit-pins-md-mk-logs/)
- make audit (manual): lint OK (markdownlint 0 errors, lychee 0 errors, flag-coverage OK); verify-examples FAIL (S4). audit.log
- quickstart `make lint`: rc 0. verify-examples: rc 2 (S4).
- technical-manual `make lint`: rc 0. verify-examples: rc 2 (S4).
- ci/doc-flag-lint.test.sh: FAIL (S6). docflag.log
- sibling-pin-check step 1 (workflow script extracted and run): rc 0, "OK all sibling pins match scripts/install.sh" (11 OK lines). Step 2 `check_cli_pins.py`: rc 0 ("installer ahead: md 0.20.6; ms 0.20.4; mk 0.15.0; mnemonic 0.106.0").
- install-msrv-guard, install-man-step, install-verify, install-assets-gate, install-assets (against the real pinned releases): all OK. install-signature: OK after the S7 edit.
- Examples diff: regenerated; diff listed above (version lines only).
- manual-gui `make html`: rc 0. `make lint` with MANUAL_GUI_UPSTREAM_ROOT = a worktree of mnemonic-gui-v0.63.0 (removed after): `[lint] OK` (an earlier run caught MD038 in my draft, fixed).
- manual-gui verify-examples: not run; it uses the GUI-tier binaries (md 0.20.3, mk 0.13.0, mnemonic 0.105.1) and this diff changes no GUI transcript.
- NOT RUN: cross-tool-differential (workflow paths do not match this diff's non-workflow files; only its own yml changed, a one-token tag), cargo nextest (no Rust changed), mr lines of the flag list (MR_BIN unset), PDF builds.

## Final grep `git grep -n 'md-cli-v0\.20\.\|mk-cli-v0\.1[0-9]\.'` (outside design/ and .examples-build): remaining older tags
- manual-gui.yml:174-175,227,233 and pinned-upstream.toml:112,114: GUI tier (md-cli-v0.20.3, mk-cli-v0.13.0) stay until the mnemonic-gui re-pin.
- docs/manual-gui src 11/12/82:27,29/94 and tests/cli-version-history.txt: GUI-tier prose and history lines; pinned-upstream.toml:17-93 history comments; docs/manual-gui/CHANGELOG.md history.
- scripts/install-verify.test.sh:553: mk-cli 0.12.0 test fixture string.
- CHANGELOG.md older entries; cycle-prep-recon-f4-cycle2-ms1-demote.md history.

Counts: Critical 0, Important 4 (S2, S4, S5, S6), Minor/draft 3 (S1, S3, S7).
