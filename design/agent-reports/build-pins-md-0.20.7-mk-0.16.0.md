Claude Sonnet 5.5 (model id claude-sonnet-5-5), build report: toolkit pins md-cli-v0.20.7, mk-cli-v0.16.0 + FU1 companion.

Base: origin/master 13dcf81e750e999e86c200b6afea2be48b5f8989 (did not move). Pins commit: 595b27f0. Branch pins/md-0.20.7-mk-0.16.0. Worktree /scratch/cache/wt-toolkit-pins-md-mk-2, logs /scratch/cache/toolkit-pins-md-mk-2-logs/.

## Binaries
md 0.20.7 (md-0.20.7-x86_64-linux-musl.tar.gz, sha256 OK, `md compile` present) and mk 0.16.0 (sha256 OK) from the release assets; ms 0.20.4 reused from the previous pin's logs. Not installed over the user's.

## Pins
install.sh:55/:61, the four workflow mirrors (a `.github/` change: cross-tool-differential, manual, quickstart, technical-manual), `pinned-upstream.toml` [installer-ahead] md and mk rows (+ its comment), 82-binary-and-launch.md:47/:49 tag names (they must equal install.sh: check_cli_pins), 44-mk-cli.md:12 (sibling-pin-check). `first_signed` stays md 0.20.4, mk 0.14.0 (both earlier than the pins; 0.20.7 and 0.16.0 are signed: install-assets gate passes). Only the install.sh comment "the pinned mk 0.15.0 is signed too" became 0.16.0. GUI tier (manual-gui.yml:227/:233, [manual-gui] rows) unchanged. Examples golden: only the md/mk version, tag and asset lines changed (12 lines, listed in the diff); nothing else.

## Pre-edit check (qs-26 flow with mk 0.16.0)
rc=0. stdout: xpub, origin_fingerprint, origin_path, policy_id_stubs, chunks lines. stderr: only `note: stdout is watch-only — public keys only, cannot spend`. The `warning:` line is gone. The deleted texts' cut points match the file text (cut exactly from "The `warning:` line is expected" to "...shares with its bundle." and from "On a bundle's mk1 card it also prints" to "...re-engrave it.").

## Exact texts
1. qs-26-recover-mk1.out: warning line removed (only change).
2. quickstart 26-recover.md and 3. manual 24-recover.md: cut as ruled, only that paragraph rewrapped.
5. 44-mk-cli.md `mk verify`: sentence added after "...disagrees with the decoded value." word for word.
6. FOLLOWUPS `mk-chunk-set-id-warning-bundle-stamped` Status: `closed, by the pins commit of this PR (mk-cli-v0.16.0, md-cli-v0.20.7)`.
7. CHANGELOG: one "Installer pins: md 0.20.7 and mk 0.16.0" entry naming both tags, with the required sentence.

## Docs grep (md 0.20.7 / mk 0.16.0 changed text)
Patterns: "not derived from its content", re-mint, re-engrave, stamped chunk-set id, shares with its bundle, bundle id, bundle_stamped, `warning:`. Hits outside the cuts:
- technical-manual transcripts mk1-decode-bip48-multisig.out (12345) and mk1-decode-bip84-no-fingerprint.out (45678): STAY; `make verify-examples` with mk 0.16.0 passes (tm verify rc=0), so they still warn byte-for-byte.
- technical-manual 22-mk1-wire-format.md:352 (says mk 0.14.0 and later warn on the two pinned-id vectors): STAYS, still true (same two vectors).
- manual transcript 44-mk-inspect-text.out:12 `chunk_set_id: e172a (stamped)`: STAYS, passes under mk 0.16.0 (manual audit rc=0; FU3 label nit is a separate open item).
- 44-mk-cli.md:344 `warning:` of `mk repair` (Set-level re-verify): different advisory, STAYS (44-mk-repair-text.err verified by audit).
- manual-gui 82-binary-and-launch.md:122-132 (GUI tier, item 4): UNCHANGED as ruled. It is now stale against the installer's mk 0.16.0 (it says every bundle card warns); it waits for the GUI mk tier.
- 45-mr.md "re-mint" hits (plates, fingerprints, kit notes): about `mr`, unrelated.
- `mk verify --json`/`chunk_set_id` mentions in manual-gui 7x-mk pages describe the GUI-tier mk 0.13.0: STAY.
No other hit; no regenerated transcript other than qs-26 changed. No STOP.

## Gates (logs in the logs dir; no gate failed)
- manual `make audit` rc=0; manual-gui-style `ci/doc-flag-lint.test.sh` rc=0
- quickstart lint rc=0, verify-examples rc=0
- technical-manual lint rc=0, verify-examples rc=0
- manual-gui `make html` rc=0; `make lint` rc=0 with MANUAL_GUI_UPSTREAM_ROOT at a detached mnemonic-gui-v0.63.0 checkout (first run rc=2 only because the default upstream path was missing; the worktree was removed after)
- sibling-pin-check step 1 (extracted run block) rc=0 "OK all sibling pins match"; step 2 (check_cli_pins.py) rc=0
- scripts/install-{msrv-guard,man-step,verify,signature,assets-gate,assets}.test.sh all rc=0
- examples golden regenerated equals committed (rc=0)
- `cargo build --locked --bin mnemonic` for MNEMONIC_BIN: ok.
NOT RUN: the Rust test suite (no Rust changed), cross-tool-differential/other CI-only jobs, PDF builds.

## Remaining older tags (git grep md-cli-v0.20.* / mk-cli-v0.1x.*)
Stay: manual-gui.yml and [manual-gui] rows (GUI tier md 0.20.3, mk 0.13.0), 82/11/12/94 GUI pages and cli-version-history (GUI tier or history), install-verify.test.sh (test fixture), CHANGELOG history, design/ specs and FOLLOWUPS history.

## Counts
Critical 0, Important 0, Minor 0, STOPs 0 (one note: 82 stale, as ruled).
