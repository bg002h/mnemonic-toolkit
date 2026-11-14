**⚠ 1 Important (I1: mk exit-table row 1 says a `--out` write error exits 1; `mk encode --out` exits 64).**

# Review: toolkit PR #162 pins md-cli-v0.20.6 / mk-cli-v0.15.0 (head 3837fc30)

Reviewer: Claude Sonnet 5.5 (claude-sonnet-5-5), medium. Round 1 (no prior fix round to attribute against; counts below are for the whole PR).

Verified up front: local md (glibc `linux-amd64` asset), mk, ms tarball sums match the release SHA256SUMS files (md x86_64 musl `b3ab17b1...`, mk `640da46c...`, ms `6813d839...`, md portable `85d9a07a...`; local glibc `md` binary sha 13cbad72 is the one inside the portable asset). `first_signed`: md-cli-v0.20.4 has .minisig (v0.20.3 none); mk-cli-v0.14.0 has .minisig (v0.13.0 none) -> install.sh `first_signed` md 0.20.4 / mk 0.14.0 correct. verify-examples.sh against the pinned binaries (mnemonic 0.106.0 debug build from the builder worktree): manual 62/62, quickstart 62/62 (symlink to manual transcripts), technical-manual 18/18 pass.

## Findings

### I1 (Important) 44-mk-cli.md exit table row 1: "or writing `--out`" is false for `mk encode --out`
Row `| 1 | mk1 format violation ... Also an I/O error reading stdin or writing `--out` (an unreadable `--in FILE` is `64`). |`.
Run (pinned mk 0.15.0): `mk encode --xpub <xpub> --origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --out /nonexistentdir/x` -> `error: --out /nonexistentdir/x: No such file or directory`, rc 64; `--out <directory>` rc 64; `--out /dev/full` rc 64. Source (mk-cli-v0.15.0 `cmd/encode.rs:440-441`): `write_private(...).map_err(|e| CliError::UsageError(format!("--out {path}: {e}")))` = 64 for every `mk encode --out` failure. Only `mk vectors --out` / `mk gen-man --out` give rc 1 (`error: io error: Permission denied`, rc 1 for gen-man). A reader acting on the table would expect 1 from a failed `mk encode --out`.
Fix wording (row 1, replace the parenthetical sentence): `Also an I/O error reading stdin, or writing `mk vectors --out` / `mk gen-man --out` (an unreadable `--in FILE` and an unwritable `mk encode --out FILE` are `64`).`
(Text is as ruled in fold brief B3; the ruling is what is wrong.) Both clauses verified against the binary: stdin-is-a-directory gives rc 1 (`io error: Is a directory`).

## What was run (all against the pinned binaries md 0.20.6, mk 0.15.0, ms 0.20.4)

| Where | Command / claim | Result |
|---|---|---|
| 44-mk Output, JSON | `mk encode ... --json` keys; `--privacy-preserving` | `origin_fingerprint` `73c5da0a` / null, `origin_path` `84'/0'/0'`: matches |
| 44-mk, 43-ms, 11/12-style text | `--separator hyphen|comma|-|,` | rc 64 with "Use `--separator space`" (mk) / "no longer offered" (ms encode, split, hashlock); `" "` and `space` rc 0: matches. Grouped-with-`-`/`,` cards still decode (mk): matches |
| 44-mk Output | stdout unbroken, card on stderr with `group size:` `separator:` lines | matches |
| 44-mk exit table | invalid card decode/verify/inspect/derive/address rc 1, `mk repair` rc 2 (uncorrectable), rc 5 (corrected), `--json` envelope `exit_code` 1 | matches (the envelope example shows 1; `mk repair --json` gives 2, see M1) |
| 44-mk exit table row 1 | "I/O error reading stdin or writing `--out`" rc 1 | stdin: rc 1 matches; `mk encode --out` failures rc 64: **I1** |
| 44-mk exit table | unreadable `--in` rc 64 | matches |
| 44-mk, 42-md "When stdout fails" | `> /dev/full` for verb and `--help` and `--version`: `mk: cannot write the output: ...` / `md: ...` rc 1; `| head` rc 0; failing verb keeps its own code (`mk decode` partial card rc 1) | matches |
| 44-mk Output, 82 | `mk inspect` ends `chunk_set_id: <id> (stamped)` | matches |
| 82, 24-recover, qs-26 | warning on decode/inspect/verify/derive/address of a bundle card; `mk verify` repeats on stdout after `OK:`; ids `1c017` in `22-first-bundle.out` | matches |
| 82 | BCH-repaired card: stderr `note: BCH error-correction repaired ... chunk 0: 1 correction(s)` on decode and verify, exit 0, names `mk repair` | matches |
| 44-mk verify rows | stubs in other order: rc 0 + note; wrong stub rc 4; subset rc 4 | matches |
| 44-mk address | multisig cosigner card: rc 64, message names `md address <MD1>... --from-mk1-file FILE` | matches |
| 42-md compile, 82 | 9 nested `or`s: segwitv0 rc 1; tap only when inside one leaf (`and(pk(Q),<9 ors>)`) rc 1; 8 ors rc 0; `thresh(1,...)` 12 keys rc 0; `md encode --from-policy` same bound; message names `thresh(1,...)` | matches |
| 42-md `--network` row, 82 | `md address ... --from-mk1-file` with a tpub card: default/mainnet rc 1 "its xpub is a testnet key, but --network says mainnet. Re-run with `--network testnet`, `--network signet` or `--network regtest`"; mainnet card with testnet/signet/regtest rc 1 "Re-run with `--network mainnet`"; `md descriptor` same | matches |
| 82 md encode remedies | tpub descriptor with origin: `--network <testnet|signet|regtest>` lines, run as printed with testnet/signet/regtest rc 0; zpub-less/xpub: no `--network`; no-origin tpub: only `--emit template` line, and that line runs rc 0 | matches |
| 43-ms | `ms encode`/`split`/`hashlock` `--separator` rows | match |

## Do 2: doc-flag-lint mutation cases
`ci/doc-flag-lint.test.sh` with the pinned binaries: 16 passed, 0 failed (baseline, M0, R1-R7, C1-C2, F1-F2, G0, G1, G3). Replanted R2, R4, R7, the stale-marker (R3 shape, on mk) and M0 by hand in a copy and read the lint's own output: R2 prints `does not name the pinned release; expected '(unreleased: mk-cli after 0.15.0: ...)'`; R4 prints `flag --future-flag is documented for \`mk encode\` ... pinned binary does not define it`; R7 prints `names no flag`; stale marker prints `stale marker ... --keys is defined by the pinned mk-cli 0.15.0`; M0 prints `exempt: --future-flag ... unreleased after mk-cli 0.15.0` and no FAIL. Every case fails on its own assertion, none on an unrelated error. No Critical.

## Do 3: transcripts
`verify-examples.sh` (the CI runner) on manual, quickstart (symlink to manual transcripts) and technical-manual transcripts with the pinned binaries and the builder's mnemonic 0.106.0: all pass (62/62/18). The only transcripts showing the `stamped chunk-set id` warning are `qs-26-recover-mk1.out` (bundle card, with the note in 24-recover.md and 26-recover.md) and the two technical-manual vectors (`12345`, `45678`; sentence in 22-mk1-wire-format.md names `83bb2`/`c5afb`, which equal the warning's computed values). No other transcript teaches a warning card; `24-recover-mk1` and `qs-24-verify` use `mnemonic`, which prints none.

## Do 4: older tags
`git grep` at the head outside the settled GUI tier, historical `design/`, `CHANGELOG.md` and agent reports: no older md-cli/mk-cli tag or version remains where it should have moved. Remaining hits are version-attributed prose (`md-cli 0.20.0+` etc.), the `manual-gui.yml` GUI pins, `scripts/install-verify.test.sh` synthetic fixtures, and `lint.sh:105` comment example (`after 0.13.0`, M2). `first_signed`: md-cli-v0.20.4 and mk-cli-v0.14.0 carry `.minisig` (3 minisig assets each); md-cli-v0.20.3 and mk-cli-v0.13.0 carry none; mk-cli-v0.15.0 and md-cli-v0.20.6 carry them: `md 0.20.4`, `mk 0.14.0` correct.

## Do 5: CI (head 3837fc30, draft)
All jobs pass except `manual` and `quickstart`, still pending when read (no red check). They `cargo install` the pinned `mk-cli-v0.15.0` and `descriptor-mnemonic-md-cli-v0.20.6 --features cli-compiler` (workflows `manual.yml:158,168`, `quickstart.yml:134,143`) and run lint, verify-examples and the doc-flag-lint mutation test against them; `technical-manual` (same pins) passed. `sibling pins match install.sh`, `install.sh harnesses`, `verify-examples`, `verify-examples-gui`, `lint`, `manual-gui` pass.

## Minors and Nits
- M1 (Minor): 44-mk-cli.md:592 JSON error envelope example shows `"exit_code": 1` only; `mk repair --json` on an uncorrectable card gives `"exit_code": 2` (run). Table row 2 now says so; add "(2 for `mk repair`)" to the example's lead-in. Needs two unusual conditions (JSON mode, repair), fails closed: not Important.
- M2 (Minor): docs/manual/tests/lint.sh:105 comment still cites the example `(unreleased: mk-cli after 0.13.0: --in)`; it is only an example of the form, now names a released flag. Update to `0.15.0`/a planted name.
- N1 (Nit): `ci/doc-flag-lint.test.sh` R2 and R4 both expect the same "does not define it" text, which R2's mutation also prints; R4 still fails if a missing marker were accepted, so it is not a defect.
- N2 (Nit): scripts/install.sh:183-188 comment has one over-long line ("...Bare versions only, never tags: the pin gates grep this file for the") after the rewrap; cosmetic.

0 Critical, 1 Important (I1), 2 Minor, 2 Nit. Zero findings come from a prior fix round (this is the first review of the PR; the fold commit de390d5a was authored by an architect/folder, the model/effort are not stated in the brief).

0 C / 1 I / 2 M / 2 N
