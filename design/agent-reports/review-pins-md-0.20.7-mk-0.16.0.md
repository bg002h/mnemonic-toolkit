# Review: toolkit PR #164 "pins: md-cli-v0.20.7, mk-cli-v0.16.0" (head 25b2b347)

Reviewer: Claude Sonnet 5.5 (claude-sonnet-5-5), medium. Class D. Re-review: no (first review); 0 findings from a prior fix commit.

Verdict: no Critical, no Important. The PR pins both tags everywhere it should and nowhere it should not, and the changed manual text matches the pinned binaries.

## Binaries
- mk: bin/mk sha a8fe84e9... equals the mk-0.16.0-x86_64-linux-musl asset; its tarball sha eb6f2f24... matches `SHA256SUMS.x86_64` fetched fresh from the release. `mk --version` prints `mk 0.16.0`.
- md: bin/md sha 3bc9ee29... equals the content of md-0.20.7-x86_64-linux-musl.tar.gz, whose sha 9833dafc... matches the release `SHA256SUMS.x86_64`. `md compile` is present (cli-compiler). `md --version` prints `md 0.20.7`.
- `first_signed`: both releases' minisigs verify with the install.sh key EF2B8D34D8409754 (md SHA256SUMS.portable, mk SHA256SUMS.x86_64): "Signature and comment signature verified", trusted comments name md-cli-v0.20.7 and mk-cli-v0.16.0. md stays 0.20.4 and mk stays 0.14.0: right (both earlier signed, neither moves).
- Tags exist; they peel to 62280dd9 (md) and 95d70a5a (mk), as the build brief states.

## Do 1: changed text run against the pinned binaries
| file:line | command / claim | exit | matches |
|---|---|---|---|
| 26-recover.md :53-60 (cut) / 24-recover.md :67-75 (cut) | `mk decode` of the qs-26 card: no warning on stderr (stderr is only `note: stdout is watch-only...`) | 0 | yes; the removed text is no longer true, and its absence is right |
| 44-mk-cli.md:446-447 | `mk verify --json` on the qs-26 bundle card: `{"chunk_set_id":{"bundle_stamped":true,"declared":"1c017","derived":"baa99","matches":false},"chunks":2,"ok":true,...}` | 0 | yes: four keys exist; bundle_stamped true on the bundle card |
| same | `mk verify --json` on 12345 (bip48-multisig): `bundle_stamped:false, declared 12345, derived 83bb2, matches:false` | 0 | yes |
| same | `mk verify --json` on 45678 (bip84-no-fingerprint): `bundle_stamped:false, declared 45678, derived c5afb, matches:false` | 0 | yes |
| same | `mk verify --json` on a fresh `mk encode` card (33917): `bundle_stamped:false, declared=derived=33917, matches:true` | 0 | yes |
| 44-mk-cli.md:12 | install line `--tag mk-cli-v0.16.0 --bin mk`; tag exists on the remote | n/a | tag matches install.sh:61 |
| 82-binary-and-launch.md:47-49 | tags md-cli-v0.20.7, mk-cli-v0.16.0 equal install.sh component_info (:55,:61); `Pinned:` labels unchanged (GUI tier) | n/a | yes |
| warnings gone / kept | `mk decode`, `inspect`, `verify`, `repair` on the qs-26 card: 0 "not derived" lines. On 45678: decode, inspect and verify still warn (1 each); repair 0 (its warning is the set-level one, a Settled stay). `md descriptor --from-mk1` prints no such line for either card | 0 | yes; the CHANGELOG claim holds, and the pinned-id cards still warn as the Settled list says |

The brief's "run on the qs-26 bundle card AND a pinned-id card" is done above. Claim "For a chunked card the `--json` envelope also carries `chunk_set_id`": true for chunked cards. I did not find a single-chunk card to show it absent (the one I tried, `--privacy-preserving`, needed other flags), so "for a chunked card" is unverified in its converse. Not graded.

## Do 2: qs-26 rerun
Fresh temp cwd, `MK_BIN=bin/mk bash qs-26-recover-mk1.cmd`, stdout then stderr (the transcript harness merges them): identical to the committed `.out` (diff empty). Stderr is the single `note: stdout is watch-only` line, already in the `.out`. No `.err` file exists for it.

## Do 3: greps
`git grep` for older md/mk tags or versions: every non-design hit outside the Settled list is a historic or GUI-tier entry (manual-gui pins, `docs/manual-gui/CHANGELOG.md`, `94-release-history.md`, `install-verify.test.sh:553` test fixture mk-cli-v0.12.0, `install.sh:184` prose comment which names 0.14.0 as first signed and the pin 0.16.0). No stale md 0.20.6 / mk 0.15.0 in a live pin. `git grep 'not derived from its content|re-mint|chunk-set id'`: remaining hits are only the Settled stays (`82-binary-and-launch.md:122-132`, 45-mr re-mint (about `mr`), manual-gui schema text) and CHANGELOG history. No other bundle-card warning claim.

## Do 4: CHANGELOG
Entry sits under `[Unreleased]`, heading `Installer pins: md 0.20.7 and mk 0.16.0`, same form as toolkit#162's (bold install.sh sentence naming both tags, mirrors list, first_signed, manual changes, GUI stays), and carries the ruled sentence "A card `mnemonic bundle` makes no longer prints a chunk-set-id warning in `mk` or `md`." Both tags named.

## Do 5: CI (read only, at time of review)
Pass: changes, clippy, fmt, install.sh gui pin, lib cross-platform x2, release test mlock, miri, verify-examples, verify-examples-gui, both "sibling pins match install.sh". Pending: test (ubuntu, macos), build mnemonic (macos x2), build, g6, release test, lint, technical-manual, examples, install.sh harnesses, quickstart, musl x2, cross-tool md1 differential, manual. No red. The jobs that gate with the pinned binaries are manual, quickstart, technical-manual, cross-tool differential (cargo install of the new tags), examples and install.sh harnesses.

## Findings
Critical: none. Important: none.

Minor
- M1: PR is draft and CI was still pending; the controller must see the pending jobs green (notably `manual`, `quickstart`, `technical-manual`, `cross-tool-differential`, which `cargo install` the new tags) before merge.

Nit
- N1: `docs/manual-gui/pinned-upstream.toml` comment still describes mk as "the mk output and uniformity changes" with 0.16.0; it does not name the chunk-set-id change. Cosmetic, comment only.
- N2: CHANGELOG entry lines are unwrapped long lines (consistent with the precedent entry; no action).
- N3: the 44-mk-cli.md addition is two hard-wrapped lines at about 100 columns, where neighbours wrap near 70. Ruled text, left as is.

Texts 1, 2, 3, 5, 6, 7 match their rulings; text 4 correctly not done. The PR description names the `.github/` change.

Worktree removed.

Claude Sonnet 5.5 (claude-sonnet-5-5)

0 / 0 / 1 / 3
