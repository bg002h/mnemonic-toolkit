# FIX: `mnemonic convert --account` is silently ignored where no requested target reads it (I)

Model: Claude Opus 5.5 (claude-opus-5-5), 2026-11-23. Small-fix track (RULES.md "Authors"). Branch
`fix/sec-convert-account-unused-refusal`, base `origin/master` `68ea05e0` (tk 0.107.1). This document's commit
changes no production code and adds no test file; the builder writes the rows in section 6.

Source: security sweep unit `convert-account-unused-refusal`, routing row
`coordinator/reports/security-sweep/grade/routing.md:42`, graded **I** as `convert-account-ignored-nonseed` in
`coordinator/agent-reports/grade-security-sweep-toolkit-gui.md:77`. What changes (routing cell, verbatim):
"`convert --account` is refused when the source is not a seed (xpub/xprv/wif), the same rule `--path` follows".

**Two STOPs are open (section 12).** The document is written for the recommended answers (S1: refusal; S2: the
edge rule). Section 12 lists exactly which rows, texts and tests flip under the other answers.

**How this document was checked.** The code, help and refusal were applied in this worktree as a scratch change
and reverted. They are saved as a reference patch, `/scratch/cache/convert-account-fix-reference.patch` (sha256
`4e8b667d2eca335f8a771140923cf087523eeee0fdfe4635b3f8ec878680a6e9`, 1 file, `convert.rs` +20 -1; `git apply
--check` passes at `68ea05e0`). Every "today" cell in section 6 was run against a debug `mnemonic` built from
`68ea05e0`; every "expected" cell was run against the same tree with the patch. The full `mnemonic-toolkit` nextest
suite was run once with the patch applied to find the tests that change (section 8).

## 1. Summary

`--account` is read on exactly one family of edges (convert.rs:1326-1390 at `68ea05e0`): a seed source
(`phrase`, `seedqr`, `entropy`) with `--to xpub` or `--to xprv`, `--path` absent (R3 already refuses a non-zero
`--account` with `--path`). Every other edge drops it. Today `convert --from xpub=<tpub> --to xpub --network signet
--account 3` exits 0 and prints the input key, so a holder who asked for account 3 gets the key they typed. The
fix mirrors ruling R4 (`--path` is refused when no requested target reads it): a non-zero `--account` is refused
(exit 2) when no requested `--to` target reads it from the `--from` source.

No new flag, type, command or public API. Spec delta: one new §4 bullet and one changed sentence. Small-fix track
is enough.

## 2. Holder-visible behaviour (after the fix)

- A non-zero `--account`, and no requested `--to` target reads it from the `--from` source: exit 2, stdout empty
  (also with `--json`: no JSON is printed), stderr's last line (byte-exact):

  `error: --account is not used by any requested --to target from this --from source (only --to xpub or --to xprv from a phrase, seedqr or entropy reads it); drop --account.`

  Lines printed before the pre-checks (the argv-secret warning) still print first, as for every other convert
  refusal.
- Reading edges: `phrase`/`seedqr`/`entropy` → `xpub`, `xprv`. "Requested target reads it" is "any", as for
  R4: `--to entropy,xpub --account 3` from a phrase is accepted and the xpub is the account-3 key.
- `--account 0` (explicit or default) is never refused: a `u32` with default 0 cannot tell them apart, the same
  rule R3 already states (SPEC §4).
- Order (fixed): the §3 edge refusals (`classify_edge`), then the wif `--path` refusal, then R4 (`--path` unused),
  then R3 (`--account` with `--path`), then this refusal, then the `--xpub-prefix` network refusal and everything
  after. So with `--path` present R4 or R3 always fires first and this refusal only meets runs without `--path`.
  It fires before the dispatch-time refusals, e.g. `--to address requires --path`, which a run with a non-zero
  `--account` and no `--path` therefore no longer reaches.
- `--help`, the `--account` entry (one rendered line; clap drops the final full stop):

  `BIP-32 account index (default 0). Used to compute the template-derived path when `--path` is omitted. Refused (non-zero) together with --path, or where no requested target uses it`

- Unchanged: every run whose `--account` is 0, and every seed → xpub/xprv run with `--template`. No exit code,
  stdout or JSON shape changes on any accepted run.

## 3. Code delta, `crates/mnemonic-toolkit/src/cmd/convert.rs` (the reference patch)

1. The `account` field's doc comment: its last line `/// together with --path.` becomes `/// together with
   --path, or where no requested target uses it.`
2. A new `fn reads_account(from: NodeType, to: NodeType) -> bool`, true exactly when `from` is `Seedqr`, `Phrase`
   or `Entropy` and `to` is `Xpub` or `Xprv`; placed after `refusal_account_with_path`.
3. A new `fn refusal_account_unused() -> ToolkitError` returning `ToolkitError::ConvertRefusal` with the section-2
   text (without the `error: ` prefix, which the error printer adds; exit 2 by `error.rs:653`).
4. Pre-check step `5.0c`, directly after step `5.0b` (R3): when `args.account != 0` and no element of `targets`
   satisfies `reads_account(primary.node, t)`, return `refusal_account_unused()`.

No new `ToolkitError` variant.

## 4. Spec delta, `design/SPEC_convert_v0_6.md`

4.1. §4, a new bullet directly after the bullet "`--path` given with a non-zero `--account`: REFUSED (exit 2). …"
(L227-230 at `68ea05e0`), exact text:

> - A non-zero `--account` given and no requested `--to` target reads it from the `--from` source: REFUSED (exit
>   2). Only these edges read `--account`: `phrase`/`seedqr`/`entropy` → `xpub`, `xprv` (the template-derived
>   account key; with `--path` the bullet above refuses first). An explicit `--account 0` is accepted
>   (indistinguishable from the default). Checked after the `--path` refusals above. Stderr (byte-exact):
>   ```
>   error: --account is not used by any requested --to target from this --from source (only --to xpub or --to xprv from a phrase, seedqr or entropy reads it); drop --account.
>   ```

4.2. L325, the sentence

> Side-inputs that are not meaningful for the chosen edge are IGNORED (not refused), except `--path` (refused when
> no requested target reads it) and a non-zero `--account` given with `--path` (§4).

becomes

> Side-inputs that are not meaningful for the chosen edge are IGNORED (not refused), except `--path` (refused when
> no requested target reads it) and a non-zero `--account` (refused when given with `--path` or when no requested
> target reads it) (§4).

No other spec sentence changes (grep of `--account` over `design/SPEC_convert*.md` at `68ea05e0`: L77, L99, L102,
L227-229, L253, L321, L325, L369; only L325 and the §4 list are affected; L77 and L321 already describe the
reading edges).

## 5. Manual, help and GUI-manual delta (lockstep, RULES "Builds and tests")

5.1. `docs/manual/src/40-cli-reference/41-mnemonic.md:1134`, the `convert` flag row becomes:

> | `--account <ACCOUNT>` | account index (default 0); read only by `--to xpub` / `--to xprv` from a `phrase=`, `seedqr=` or `entropy=` source without `--path`. Refused (exit 2) when non-zero together with `--path`, or when non-zero and no requested target reads it |

5.2. `docs/manual-gui/src/40-mnemonic/44-convert.md:474` (section `--account {#mnemonic-convert-account}`), the
line "BIP-32 account index (default 0). Range 0..2_147_483_647." gains one sentence after it:

> Read only by `--to xpub` / `--to xprv` from a phrase, SeedQR or entropy source without `--path`; a non-zero
> value is refused (exit 2) together with `--path`, or where no requested target reads it.

5.3. Help: section 2's line (comes from the doc comment, section 3 item 1).

5.4. No flag is added, removed or renamed: no `gui-schema` change, no `mnemonic-gui/src/schema/mnemonic.rs`
change, and `docs/manual/tests/lint.sh` flag coverage is unaffected. `gui-schema` output does not carry the help
text (checked: no `Refused (non-zero)` in `mnemonic gui-schema` at `68ea05e0`).

5.5. `CHANGELOG.md`, under `## [Unreleased]`, a new subsection, exact text:

> ### `convert` refuses an unused `--account`
>
> - **`convert`**: a non-zero `--account` is now refused (exit 2) when no requested `--to` target reads it. Only
>   `--to xpub` and `--to xprv` from a phrase, SeedQR or entropy source read it. Before, `--from xpub=… --to xpub
>   --account 3` (and the same from xprv, wif, mk1 or ms1) printed the input key unchanged at exit 0.

## 6. RED rows

New file `crates/mnemonic-toolkit/tests/cli_convert_account.rs` (each `tests/*.rs` is its own binary; nothing to
register). Every row runs `mnemonic --allow-argv-secret convert <args>` through the same `convert`/`Out`/
`assert_refused` helpers as `tests/cli_convert_path.rs` (copied, not shared). Refusal rows assert: the exit code,
**stdout empty**, and a stderr line equal to the text.

Constants (values copied from `tests/cli_convert_path.rs`; signet test seed BIP-39 `abandon x11 about`, master
fingerprint `73c5da0a`):

| name | value |
|---|---|
| `P` | `phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about` |
| `SQ` | `seedqr=000000000000000000000000000000000000000000000003` |
| `EN` | `entropy=00000000000000000000000000000000` |
| `S` | `--network signet` |
| `XPUB_84`, `XPRV_84` | the BIP-84 signet account key at m/84'/1'/0' (`tpubDC8msFGeGuwnKG9U…`, `tprv8fSjiqEQ8YG7Ro7g…`) |
| `WIF_87`, `MK1_84`, `MS1` | as in `cli_convert_path.rs` |
| `XPUB_84_A3` | `tpubDC8msFGeGuwnSsiDodPKT5czjv1psTKmAnqF8tmt8cJ8Besyz2KEbSCkA5M2xdvVZfcTWpBohyLzKNVKhDR1ysXy9aoZMVrS56hjFNJCCNR` (m/84'/1'/3'; equals `convert --from P S --to xpub --path m/84'/1'/3'` at `68ea05e0`) |
| `XPRV_84_A3` | `tprv8fSjiqEQ8YG7ZQgRuyij3fxtAtVti88rbVETrNjaiLVjMAdDMdVeQwasyvm2KrMXdtKKCB9jmp1jY3bgk9SWiEUccJgRYEcJsrubEBjqypZ` |
| ACCT text | section 2's stderr line |

6.1. RED (fail on `68ea05e0`, pass with the patch). "Today" is the real output of the unmodified binary.

| row | args | expected | today (`68ea05e0`) | wrong implementation it catches |
|---|---|---|---|---|
| `acct_xpub_to_xpub_account_refused` | `--from xpub=XPUB_84 S --to xpub --account 3`; leg: the same with `--account 1` | 2; stdout empty; ACCT text | 0; `xpub: XPUB_84` (the input key); stderr only `note: stdout is watch-only — public keys only, cannot spend`. Leg: same | no check at all; a `account > 1` threshold (leg) |
| `acct_xprv_to_xpub_account_refused` | `--from xprv=XPRV_84 S --to xpub --account 3` | 2; stdout empty; ACCT text | 0; `xpub: XPUB_84` | a rule that lists only `xpub` as a non-seed source |
| `acct_wif_to_xpub_account_refused` | `--from wif=WIF_87 S --to xpub --account 3` | 2; stdout empty; ACCT text | 0; `xpub: tpubD6NzVbkrYhZ4WLczPJWReQycCJdd6YVWXubbVUFnJ5KgU5MDQrD998ZJLSmAZzbfovuoxYYPsdiZxe76Ag1d6pWprfgihP5wqjtJSZhrkWq` (depth-0 sentinel) | a check placed after the wif sentinel warning (stdout not empty / warning printed) |
| `acct_mk1_to_xpub_account_refused` | `--from mk1=MK1_84 S --to xpub --account 3` | 2; stdout empty; ACCT text | 0; `xpub: XPUB_84`, the SLIP-0132 zpub note | a source list of only xpub/xprv/wif (the routing cell's parenthetical) |
| `acct_ms1_to_phrase_account_refused` | `--from ms1=MS1 S --to phrase --account 3` | 2; stdout empty; ACCT text | 0; `phrase: abandon … about` | treating ms1 as a seed source that reads `--account` |
| `acct_phrase_to_fingerprint_account_refused` (S2) | `--from P S --template bip84 --to fingerprint --account 3` | 2; stdout empty; ACCT text | 0; `fingerprint: 73c5da0a` | `reads_account` including `Fingerprint`; a source-only rule |
| `acct_phrase_to_entropy_account_refused` (S2) | `--from P S --to entropy --account 3` | 2; stdout empty; ACCT text | 0; `entropy: 00000000000000000000000000000000` | a source-only rule |
| `acct_phrase_to_address_account_refused` (S2) | `--from P S --to address --script-type p2wpkh --account 3` | 2; stdout empty; ACCT text; no line `error: --to address requires --path (xpub does not carry an origin path; supply BIP-32 derivation explicitly).` | 2; that `--to address requires --path …` line (ACCT text missing) | a source-only rule; `reads_account` including `Address` |
| `acct_xpub_to_address_account_before_address_path` | `--from xpub=XPUB_84 S --to address --script-type p2wpkh --account 3` | 2; stdout empty; ACCT text; no `--to address requires --path` line | 2; `error: --to address requires --path (xpub does not carry an origin path; supply BIP-32 derivation explicitly).` | the check placed at dispatch time instead of in the pre-checks |
| `acct_json_refused_no_envelope` | `--from xpub=XPUB_84 S --to xpub --account 3 --json` | 2; stdout empty; ACCT text | 0; stdout a JSON object with `"from_node":"xpub"` and the input key as the xpub value | a check skipped in `--json` mode |
| `acct_help_account_line` | `mnemonic convert --help` (no `--allow-argv-secret`) | exit 0; a trimmed line equal to section 2's help line | the line ends `… Refused (non-zero) together with --path` | help not updated |

6.2. Guards (pass on `68ea05e0` and must keep passing; each pins a boundary of the rule).

| row | args | expected (= today) | wrong implementation it catches |
|---|---|---|---|
| `acct_guard_phrase_template_xpub_account_honoured` | `--from P S --template bip84 --to xpub --account 3` | 0; `xpub: XPUB_84_A3` | refusing on a seed source; `reads_account` missing `Xpub` |
| `acct_guard_seedqr_template_xprv_account_honoured` | `--from SQ S --template bip84 --to xprv --account 3` | 0; `xprv: XPRV_84_A3` | `reads_account` missing `Xprv` or `Seedqr` |
| `acct_guard_entropy_xpub_fingerprint_accepted` | `--from EN S --template bip84 --to xpub,fingerprint --account 3` | 0; `xpub: XPUB_84_A3`, `fingerprint: 73c5da0a` | "every target must read it" instead of "any"; `Entropy` missing |
| `acct_guard_reader_not_first_accepted` | `--from P S --template bip84 --to entropy,xpub --account 3` | 0; `entropy: 00000000000000000000000000000000`, `xpub: XPUB_84_A3` | checking only the first target |
| `acct_guard_xpub_account_zero_accepted` | `--from xpub=XPUB_84 S --to xpub --account 0` | 0; `xpub: XPUB_84` | refusing whenever `--account` is present (explicit 0) |
| `acct_order_r4_before_account_unused` | `--from xpub=XPUB_84 S --to xpub --path m/0 --account 3` | 2; R4 text `error: --path is not used by any requested --to target from this --from source; drop --path.`; no ACCT text | the check placed before R4 |
| `acct_order_r3_before_account_unused` | `--from xpub=XPUB_84 S --to address --script-type p2wpkh --path m/0/0 --account 3` | 2; R3 text `error: --account has no effect when --path is given; put the account in --path (e.g. m/84'/1'/3') or drop --path.`; no ACCT text | the check placed before R3 |
| `acct_order_edge_refusal_before_account_unused` | `--from xprv=XPRV_84 S --to xprv --account 3` | 2; `error: --to xprv is cryptographically unrecoverable from --from xprv (one-way derivation barrier)`; no ACCT text | the check placed before the §3 edge refusals |

Every RED row's "today" and every guard row was run on `68ea05e0` (debug build, `CARGO_TARGET_DIR`
`/scratch/cache/tgt-fix-convert-account`); every "expected" cell was run with the reference patch applied.

## 7. Spec-sentence coverage

| spec sentence (section 4) | row |
|---|---|
| non-zero `--account`, no requested target reads it: REFUSED (exit 2), byte-exact stderr | `acct_xpub_to_xpub_account_refused` and the other 6.1 refusal rows |
| only `phrase`/`seedqr`/`entropy` → `xpub`, `xprv` read it | guards `acct_guard_phrase_template_xpub_account_honoured`, `acct_guard_seedqr_template_xprv_account_honoured`, `acct_guard_entropy_xpub_fingerprint_accepted`; refusals for fingerprint, entropy, address, phrase (ms1), xpub/xprv/wif/mk1 sources |
| explicit `--account 0` accepted | `acct_guard_xpub_account_zero_accepted` |
| checked after the `--path` refusals | `acct_order_r4_before_account_unused`, `acct_order_r3_before_account_unused` |
| L325 sentence | covered by the same rows (it restates §4) |
| help line, manual rows | `acct_help_account_line`; manual rows by the docs CI jobs (section 9) |

## 8. Tests that change

Found by running `cargo nextest run --locked -p mnemonic-toolkit --no-fail-fast` once with the reference patch
applied (log `/scratch/cache/convert-account-patched-nextest.log`); the failures were exactly these two:

1. `tests/cli_convert_path.rs::r3_help_account_refused_with_path`: pins the old help line. Rule: delete it in
   the RED commit; `acct_help_account_line` replaces it (same assertion style, new line).
2. `tests/cli_convert_convergence.rs::c1a_master_fp_template_account_invariant`: its second run
   `phrase=TREZOR_24 --to fingerprint --template bip44 --account 5` is refused under S2's edge rule. Rule: in the
   RED commit drop the two `"--account", "5"` arguments from that run, change its assert message to `"C1a: master
   fp invariant across template"`, and change the section comment `C1a — master fingerprint is invariant across
   --template / --account.` to `C1a — master fingerprint is invariant across --template.` (the account
   invariance it claimed is now a refusal, pinned by `acct_phrase_to_fingerprint_account_refused`). Under S2's
   source rule this test is unchanged.

No golden file changes. `.examples-build/derive_keys.sh` runs `convert --from phrase=- --to xpub --template bip84
--account N` (a reading edge): unchanged.

## 9. Gate (the builder runs all of it; capture each run once to a file and grep it)

`PATH=$HOME/.cargo/bin:$PATH`; `cargo --version` must be 1.85.0; private `CARGO_TARGET_DIR` under `/scratch/cache`;
never `--release`.

1. RED commit: the test file of section 6 and the two test edits of section 8, production code untouched;
   `cargo nextest run --locked -p mnemonic-toolkit -E 'binary(cli_convert_account)' --no-fail-fast`: the 11 rows
   of 6.1 fail, the 8 rows of 6.2 pass (19 run). Paste the counts.
2. GREEN commit: sections 3, 4, 5. `cargo fmt --all --check`; `cargo clippy --locked --all-targets --workspace --
   -D warnings`; `cargo nextest run --locked --workspace --no-fail-fast` 0 failed; `cargo test --locked --doc
   --workspace`; the `cli_convert_account` filter: exactly **19 run, 19 passed** (a short count is a FAIL).
3. Docs CI jobs on the head (locally where possible, else green CI on the PR): `manual.yml`, `manual-gui.yml`,
   `quickstart.yml`, `technical-manual.yml`, `examples.yml`; the rest of `rust.yml` and `fuzz-smoke.yml` as green
   CI.
4. `coordinator/scripts/mutation-gate.sh <worktree> 68ea05e0` after step 2; its missed list goes in the review
   brief. Risk class A (CLI text, manual).

Not covered by this gate: the GUI (mnemonic-gui) convert form, which passes the user's `--account` through; it
shows the refusal like any other convert refusal and needs no change.

## 10. Coordination with the network spec (spec/network-rule `2a534699`, `SPEC_network_rule_tk_rest.md` unit 8a)

Unit 8a decides `convert` network refusals (rows R8a.3-R8a.5, R8a.13-R8a.17, R8a.21). None of them passes
`--account`, and this FIX changes no line that spec decides. This refusal is an argument check that runs with the
R3/R4 pre-checks, before the conversion; the network checks 8a adds need the parsed key, so on a run that has both
a non-zero unused `--account` and a network contradiction this refusal (exit 2) prints and the network refusal
(exit 8) is not reached, exactly as R4's `--path` refusal behaves today. Every row in section 6 passes `--network
signet` with signet keys, so no row depends on that order. If 8a's build places its convert checks before the
R3/R4 pre-checks, `acct_*` rows still pass (no contradiction in them).

## 11. WORDING_REVIEW proposals (mnemonic-toolkit has no `design/WORDING_REVIEW.md`; logged here)

| id | text | prints | status |
|---|---|---|---|
| SEC-CAU-W1 | `error: --account is not used by any requested --to target from this --from source (only --to xpub or --to xprv from a phrase, seedqr or entropy reads it); drop --account.` | `convert` stderr, exit 2 | proposed; mirrors R4's `--path` text plus the reading edges |
| SEC-CAU-W2 | `BIP-32 account index (default 0). Used to compute the template-derived path when `--path` is omitted. Refused (non-zero) together with --path, or where no requested target uses it` | `convert --help` | proposed |
| SEC-CAU-W3 | the section 5.1 and 5.2 manual sentences | manuals | proposed |

## 12. STOPs (product points the rulings do not decide; recommendation given, document written for it)

**S1. Refusal or warning.** Brian's standing ruling: warn-and-proceed over refusals where no funds risk. Running
with an unused `--account` risks no funds directly: the printed key is the input key, which the holder holds. The
routing row says "refused", and the same flag pair is already refused twice (R3, R4). Options:

- (a) Refuse, exit 2 (this document). For: the result at exit 0 is a wrong answer to the question asked ("the
  account-3 key"), not an ignored nicety; a script or a holder copying stdout gets no signal; R3/R4 already make
  `--account`/`--path` misuse refusals, so the two flags follow one rule; dropping the flag costs nothing.
- (b) Warn and proceed, exit 0, stdout unchanged; stderr gains, in the pre-check position (before the watch-only
  note), `warning: --account is not used by any requested --to target from this --from source (only --to xpub or
  --to xprv from a phrase, seedqr or entropy reads it); ignored.` Help line ends `… Refused (non-zero) together
  with --path; ignored with a warning where no requested target uses it`. All 6.1 rows become exit 0 with
  today's stdout plus that line, except the two `address` rows, which still end with the `--to address requires
  --path` refusal after the warning; the §4 bullet says "WARNED (exit 0)" with that line; section 8 item 2 is
  not needed (the run still succeeds).

Recommendation: (a), as routed. The sibling verb already refuses the same misuse: on `68ea05e0`, `mnemonic
addresses --from xpub=XPUB_84 --address-type p2wpkh --network signet --account 3 --count 1` exits 1 with `error:
--account does not apply to --from xpub= (the xpub is already an account key)`. A warning matches the `--passphrase ignored on this edge` precedent, but that
warning concerns an input whose absence cannot change the printed key; here the flag asks for a different key.

**S2. Scope: edge rule or source rule.** The routing cell says "when the source is not a seed (xpub/xprv/wif),
the same rule `--path` follows". R4's rule is per edge: it refuses `phrase → fingerprint --path`
(`r4_phrase_to_fingerprint_path_refused`). Options:

- (a) Edge rule (this document): refused unless some requested target is `xpub`/`xprv` from
  `phrase`/`seedqr`/`entropy`. Also refuses seed sources whose targets do not read it (fingerprint, entropy,
  phrase, ms1, wif, bip38, address). Changes `c1a_master_fp_template_account_invariant` (section 8).
- (b) Source rule: refused only when the source is not `phrase`/`seedqr`/`entropy` (xpub, xprv, wif, mk1, ms1,
  bip38, minikey, electrum-phrase). Rows `acct_phrase_to_fingerprint_account_refused`,
  `acct_phrase_to_entropy_account_refused` and `acct_phrase_to_address_account_refused` become guards with
  today's output; `c1a` is unchanged; the refusal text drops "(only --to xpub or --to xprv …)" for "(only a
  phrase, seedqr or entropy source reads it)"; the §4 bullet lists sources instead of edges.

Recommendation: (a). "The same rule `--path` follows" is the per-edge R4; `phrase → fingerprint --account 3`
prints the master fingerprint at exit 0, the same silent wrong answer the grade names; and the source rule would
still need ms1 decided (it is a seed card that reads no account).

Not a STOP (decided by precedent): explicit `--account 0` is accepted, as R3 already states in SPEC §4.
