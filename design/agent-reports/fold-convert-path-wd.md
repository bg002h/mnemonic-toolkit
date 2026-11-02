# Fold: convert --path whole-diff s1 I1/I2, s2 I1

Worktree `/scratch/code/shibboleth/tk-worktrees/triage-convert-path`, branch `fix/convert-path`, from c34a88db. Nothing pushed.
Commits: bb531c44 (code, tests, spec, FIX doc), 6bcc4916 (FOLLOWUPS). No STOP. Gate green.

## RED run (unmodified code, `binary(cli_convert_path)`, log `/scratch/cache/convert-path/fold-wd/red.log`)
40 run: 36 passed, 4 failed.
- FAIL as reported: `r5_empty_path_refused`, `r5_m_slash_path_refused`, `r5_unprefixed_path_refused`, `r4_fingerprint_only_no_template_names_template_only` (stderr still the old "--template or --path is required" text).
- PASS (pin existing behaviour): `s2_path_alone_xprv_accepted`, `s2_path_alone_bip38_accepted`, `s2_path_alone_address_accepted`, `s2_account_one_with_path_refused`, `s2_path_honours_passphrase`, `s2_path_honours_language`.
- `R5_FORM_TEXT` const added. Rows are in `crates/mnemonic-toolkit/tests/cli_convert_path.rs`; `cargo fmt` reflowed them (formatting only).

## Fixes
- `crates/mnemonic-toolkit/src/cmd/convert.rs:629` new `parse_literal_path`; call sites :1354, :1438, :1492, :1532, :1592 (all five anchors read before replacing).
- `convert.rs:1363-1376` the `else if needs_derive` branch: `reads_path` split of the refusal text (I2's given fix, not the alternative).
- `design/SPEC_convert_v0_6.md`: the :232 bullet now says "an `xpub` or `xprv` target"; new fingerprint-only bullet after it. :231 is unchanged (the new error is a genuine `bitcoin::bip32` error).
- `design/FIX_convert_path.md`: summary item 5 gains the I1 sentence; the error table row for the no-template case is split (xpub/xprv vs fingerprint-only); a new row for `""`/`m/`/unprefixed; the S-text copy of the §4 bullet is split the same way. Its design code sketch at :175 still shows the bare `from_str` call (not touched, it is not a spec text).

## Manual
`grep -rn "template or --path is required\|literal \`m/" docs/manual` found only `42-md.md:172` (an `md` flag, unrelated) and `41-mnemonic.md:1102` (still true; it does not state the refusal text). No manual edit.

## Followups
Filed 4 `deferred-minor` entries at the end of `design/FOLLOWUPS.md`: s1 M1, s1 N1 (duplicate of the existing `convert-path-assert-refused-stdout`, cross-referenced), s1 N2, s2 N2.
Not filed, because the fold closes them: s2 M1 (lone xprv/bip38/address rows) and s2 N1 (`--account 1` row). Decision for the coordinator if you want them filed anyway.

## Gate (logs in `/scratch/cache/convert-path/fold-wd/`)
- `nextest.log`: `cargo nextest run --locked --workspace`: 4185 run, 4185 passed, 38 skipped (after fmt).
- `fmt.log`: `cargo fmt --check` rc 0. `clippy.log`: `cargo clippy --locked --all-targets -- -D warnings` rc 0.
- `audit.log`: manual `make audit` (the CI manual job's audit step) rc 0, anchor-check baseline matched.
- `flaglint.log`: `ci/doc-flag-lint.test.sh` 15 passed, 0 failed.
- Environment note: the local `~/.cargo/bin/mk` reports 0.13.0 but defines flags the pinned tag does not, so `audit` and the flag-lint test failed on 44-mk-cli.md markers (unrelated to this change). I installed the pinned `mk-cli-v0.13.0` to `/scratch/cache/convert-path/pinned-mk` and both are green with it. md 0.20.3 and ms 0.20.2 match the pins. Not run: the other CI jobs (PDF/HTML build, quickstart, examples, differential); the change touches no manual, help or example text.
- cargo 1.85.0 matches `rust-toolchain.toml`.

## Deviation
The commit trailer uses "Claude Sonnet 5.5" (the model that did the work) instead of the brief's "Claude Opus 5.5", following the session attribution instruction. Amend the trailer if you want the brief's text.
