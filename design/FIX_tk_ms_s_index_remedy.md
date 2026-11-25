# FIX: `mnemonic` tells the holder of an index-`s` ms1 string to run `mnemonic ms-shares combine`, which refuses it (bh-tk-ms-s-index)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24. Small track. Grade I under P8 (a backtick command name is a
printed command; it fails as printed). Not a funds path (the refusal exit code and every other text are unchanged).

Base: `origin/master` = `4bfa645f` (Merge pull request #211). Source: coordinator
`agent-reports/review-fix-bh-ms46-r11.md` I-2 (L53-70, run on tk `e30985a5`), the toolkit copy of ms #46's bh-ms-R11
defect. The wording follows the ms side decided in mnemonic-secret `fix/ms-printed-commands`
(`design/FIX_bughunt1119_ms_printed_commands.md` R5, L72-81 and the R5 text table L101-106): "this is the secret share
of a K-of-N set (threshold '<T>', index 's'); it is the secret itself, not a distributed share, and `ms combine` does
not take it", here in the toolkit's own command name.

## 0. Source files this FIX changes

1. `crates/mnemonic-toolkit/src/friendly.rs` (the `IsShareNotSingleString` arm L127-135 and its comment; one new unit
   test, section 4)
2. `crates/mnemonic-toolkit/tests/cli_ms_shares_consume.rs` (new CLI rows, section 4)
3. `docs/manual/src/40-cli-reference/41-mnemonic.md` L391 and L930 (section 5)
4. `CHANGELOG.md` `[Unreleased]` (section 6)

No flag, type, error variant, exit code, `--json` shape or public API changes. ms-codec is not bumped (tk pins 0.7.0;
this arm is tk's own text, not ms-codec's `Display`).

## 1. The defect, measured

`friendly_ms_codec` (`friendly.rs:131-135`) renders every `ms_codec::Error::IsShareNotSingleString` as
``ms1 this is ONE of a K-of-N share set (threshold '<T>', index '<I>'); use `mnemonic ms-shares combine` to recombine
<T> shares``, whatever the index. At index `s` the string is the secret itself (BIP-93: the share with index `s` is
the secret), and `mnemonic ms-shares combine` refuses it (`SecretShareSuppliedToCombine`,
`friendly.rs:136-140`). So the printed command fails as printed.

Every surface that renders this error goes through one formatter: `ToolkitError::MsCodec(e)` Display
(`error.rs:943` -> `friendly_ms_codec`; the only non-test caller, `grep -rn friendly_ms_codec src`). The ms1 decode
sites that reach it with `?`/`ToolkitError::from`: `convert --from ms1=` (`cmd/convert.rs:1738`), `inspect --ms1`
(`cmd/inspect.rs:214`), the `@N.ms1=` slot of `bundle`/`verify-bundle` and `restore --from ms1=`
(`slot_ms1.rs:45`, called from `cmd/bundle.rs:749,1790`, `cmd/verify_bundle.rs:1887`, `cmd/restore.rs:481,923,4538`),
`silent-payment --secret` (`cmd/silent_payment.rs:160`) and `xpub-search`'s ms1 seed intake
(`cmd/xpub_search/seed_intake.rs:210`). The other decode sites print `{e:?}` (Debug; no command name): `bundle.rs:2154,
2891`, `verify_bundle.rs:2442,2564`, `wallet_import/overlay.rs:160`; they are out of scope.

RED, run on `4bfa645f` (`cargo build --locked -p mnemonic-toolkit --bin mnemonic`, cargo 1.85.0; signet). `S2` =
`ms12testsqqqqqqqqqqqqqqqqqqqqqqqqqqqqkwe5qlrp6ykwa` (threshold 2, id `test`, index `s`, checksum-valid; the ms FIX's
`S2`), `A2` = `ms12testaqqqqqqqqqqqqqqqqqqqqqqqqqqqqzfvk7urqpuwd0` (index `a`). `M=target/debug/mnemonic`.

    $ printf '%s\n' $S2 | $M convert --from ms1=- --to phrase      -> rc 2
    error: ms1 this is ONE of a K-of-N share set (threshold '2', index 's'); use `mnemonic ms-shares combine` to recombine 2 shares
    $ printf '%s\n' $S2 | $M inspect --ms1 -                        -> rc 2
    error: ms1 this is ONE of a K-of-N share set (threshold '2', index 's'); use `mnemonic ms-shares combine` to recombine 2 shares
    $ printf '%s\n' MS12TESTSQQQQQQQQQQQQQQQQQQQQQQQQQQQQKWE5QLRP6YKWA | $M convert --from ms1=- --to phrase   -> rc 2
    error: ms1 this is ONE of a K-of-N share set (threshold '2', index 's'); use `mnemonic ms-shares combine` to recombine 2 shares
    $ printf '%s\n' $S2 | $M bundle --template bip84 --network signet --slot @0.ms1=- --no-engraving-card   -> rc 2 (same line)
    $ printf '%s\n' $S2 | $M restore --from ms1=- --template bip84 --network signet                         -> rc 2 (same line)
    $ printf '%s\n' $S2 | $M silent-payment --secret-stdin --network signet                                 -> rc 2 (same line)
    $ printf '%s\n' $S2 | $M ms-shares combine --share -           -> rc 2 (the printed command, as printed)
    error: ms1 the secret share (index 's') must not be combined; supply only the distributed shares (the secret is the recovery target)
    control: $ printf '%s\n' $A2 | $M convert --from ms1=- --to phrase   -> rc 2
    error: ms1 this is ONE of a K-of-N share set (threshold '2', index 'a'); use `mnemonic ms-shares combine` to recombine 2 shares

(`inspect --ms1 -` on `A2` prints the same control line, rc 2.)

## 2. Spec delta (rule)

**R1.** When the share index of an `IsShareNotSingleString` is `s` (compared ASCII-case-insensitively; ms-codec
already lowercases, so an uppercase string prints `'s'`, measured above), the message is the R1 text below: it says
the string is the secret share, that it is the secret itself and not a distributed share, and that
`mnemonic ms-shares combine` does not take it. It names no command to run. For every other index the text is
unchanged, keeping the K-share advice. Unchanged: the error variant, exit 2 (`error.rs:466`), the `error: ` prefix,
stderr only, nothing on stdout; `ms-shares combine`'s own refusal text; every other formatter arm.

### Exact printed text (one stderr line; `<T>` is the threshold digit `2`-`9`, `<I>` the index)

| share index | message today | after the fix |
|---|---|---|
| `s` (or `S`) | ``error: ms1 this is ONE of a K-of-N share set (threshold '<T>', index 's'); use `mnemonic ms-shares combine` to recombine <T> shares`` | ``error: ms1 this is the secret share of a K-of-N set (threshold '<T>', index 's'); it is the secret itself, not a distributed share, and `mnemonic ms-shares combine` does not take it`` **PROPOSED (architect confirms; WORDING_REVIEW)** |
| any other | ``error: ms1 this is ONE of a K-of-N share set (threshold '<T>', index '<I>'); use `mnemonic ms-shares combine` to recombine <T> shares`` | unchanged |

The new text is the ms R5 text with `ms combine` -> `mnemonic ms-shares combine` and tk's `ms1 ` prefix (every arm
of `friendly_ms_codec` starts `ms1 `). The `<T>` digit is printed as the codec gives it (a `char`), as today.

## 3. Code delta (production; prose)

- `crates/mnemonic-toolkit/src/friendly.rs` L131: a new arm before the existing one,
  `IsShareNotSingleString { threshold, index } if index.eq_ignore_ascii_case(&'s')`, formatting the R1 text with
  `threshold`; the existing arm (L131-135) is unchanged and now serves every other index.
- The comment above the arms (L127-130, "must point the user at `mnemonic ms-shares combine`") gains: except at index
  `s`, the secret itself, which `ms-shares combine` refuses (`SecretShareSuppliedToCombine`); there the text says so
  and names no command (bh-tk-ms-s-index).
- No other production file. (Measured: a scratch build of exactly this arm on `4bfa645f`, reverted, printed the R1
  text for every RED row of section 1, uppercase included, rc 2 each, and the unchanged control line for `A2`.)

## 4. Tests

New (each seen RED on `4bfa645f`: fails, since today's text is printed):

| test (file) | input | asserts |
|---|---|---|
| `bh_tk_ms_s_index_friendly_text` (`src/friendly.rs` tests) | `friendly_ms_codec(&IsShareNotSingleString { threshold: '2', index: 's' })` | equals ``ms1 this is the secret share of a K-of-N set (threshold '2', index 's'); it is the secret itself, not a distributed share, and `mnemonic ms-shares combine` does not take it`` exactly; also for `threshold: '9', index: 'S'` the same text with `'9'` (and `index 's'` printed lowercase, the arm writes the literal `s`) |
| `bh_tk_ms_s_index_convert` (`tests/cli_ms_shares_consume.rs`) | stdin `S2\n`, `convert --from ms1=- --to phrase` | exit 2; stdout empty; stderr equals `error: ` + the R1 text for `<T>`=`2` + `\n` |
| `bh_tk_ms_s_index_inspect` (same file) | stdin `S2\n`, `inspect --ms1 -` | exit 2; stdout empty; stderr equals the same line |
| `bh_tk_ms_s_index_uppercase` (same file) | stdin `MS12TESTSQQQQQQQQQQQQQQQQQQQQQQQQQQQQKWE5QLRP6YKWA\n`, `convert --from ms1=- --to phrase` | exit 2; stderr equals the same line (`index 's'`) |

Guard row (passes on both base and fix): `bh_tk_ms_s_index_control_distributed` (same file): stdin `A2\n`,
`convert --from ms1=- --to phrase` -> exit 2, stderr equals ``error: ms1 this is ONE of a K-of-N share set (threshold
'2', index 'a'); use `mnemonic ms-shares combine` to recombine 2 shares`` + `\n`.

`S2` and `A2` are written as constants in the test file (they are synthetic test strings, not funded keys; no entropy
from a real seed). The CLI rows pipe stdin, so they need no `--allow-argv-secret`.

Existing tests that change: none. Checked: `friendly.rs:533-540` `ms_codec_share_points_at_ms_shares_combine`
(index `a`, still contains `ms-shares combine`), `error.rs:1207` (exit code for index `a`, unchanged),
`tests/cli_ms_shares_consume.rs` `inspect_of_a_share_surfaces_friendly_message` /
`convert_from_share_surfaces_friendly_message_and_mapped_exit` (`one_share()` takes the first line of
`ms-shares split`, a distributed index, never `s`: `ms-shares split` emits distributed shares only),
`tests/cli_ms1_slot.rs:641` `ms1_kofn_share_rejected_with_combine_prose` (`encode_shares(...)[0]`, a distributed
share), `slot_ms1.rs:158` `k_of_n_share_is_rejected` (matches any `MsCodec`). The builder confirms the `one_share()`
and `encode_shares` index claim by running the suite (they would fail if they ever drew `s`).

## 5. Toolkit manual lockstep (`docs/manual/src/40-cli-reference/41-mnemonic.md`)

- L391 (`bundle --slot`), the sentence ``A K-of-N codex32 **share** (not a single-string secret) is rejected with a
  pointer to `ms-shares combine` — reassemble the secret first, then slot the combined `ms1`.`` is followed by the new
  sentence: ``The share at index `s` is the secret itself, not a distributed share, and `ms-shares combine` does not
  take it, so it is rejected (exit 2) without that pointer.`` PROPOSED.
- L930 (`verify-bundle --slot`), ``a K-of-N share is rejected with a pointer to `ms-shares combine` `` becomes ``a
  K-of-N share is rejected with a pointer to `ms-shares combine`, except the share at index `s`, the secret itself,
  which `ms-shares combine` does not take`` PROPOSED.

No other manual line names this text (grep of `docs/manual/src` and `docs/manual-gui/src` for `ONE of a K-of-N`,
`pointer to`, `ms-shares combine`). The GUI manual and the GUI schema are unchanged (no flag changes).

## 6. CHANGELOG (`[Unreleased]`, new subsection)

    ### An index-`s` ms1 string is no longer sent to `ms-shares combine`

    - **`convert --from ms1=`, `inspect --ms1`, `bundle`/`verify-bundle --slot @N.ms1=`, `restore --from ms1=`,
      `silent-payment --secret`, `xpub-search`**: an ms1 string at share index `s` (the secret itself, not a
      distributed share) is still refused (exit 2), but the message now says so and no longer tells you to run
      `mnemonic ms-shares combine`, which refuses that string. Before: ``use `mnemonic ms-shares combine` to
      recombine <T> shares``. Other share indices keep that advice. (bh-tk-ms-s-index)

## 7. Out of scope (filed in the author's report, not here)

- `mnemonic repair` on a corrected index-`s` string prints the same wrong pointer from a different text
  (`repair.rs:1252-1260`, `ms1_is_share` is true for every threshold != `0`): measured on `4bfa645f`, stdin
  `ms12testsqqqqqqqqqqqqqqqqqqqqqqqqqqqqkwe5qlrp6ykwq`, `repair --ms1 -` -> rc 4, stderr ``repair: correction UNVERIFIED
  — a corrected K-of-N share cannot be self-verified; recombine it with the other shares (mnemonic ms-shares combine
  --share - --share @env:VAR …, ...)``. Not the same formatter; its text, and the multi-chunk mixed case (`repair`
  takes several ms1 positionals), are undecided. A new row.
- The reviewer's Minor M-5 (`argv_guard.rs`, `--from ms1` told to use `--from phrase=-`) is a different file; it keeps
  its own FOLLOWUPS line (`tk-convert-argv-guard-ms1-remedy`).

## 8. Gate

The builder's gate is the repo's full CI on the branch head, captured once to a file and grepped. Minimum local line
(load gate and `CARGO_BUILD_JOBS=6` per the batch overrides):

`PATH=$HOME/.cargo/bin:$PATH CARGO_BUILD_JOBS=6 cargo nextest run --locked --workspace --no-fail-fast --build-jobs 6 -j 6 > gate.log 2>&1; cargo clippy --locked --all-targets --workspace -- -D warnings; cargo fmt --all --check`

plus the toolkit manual job (the manual lint/build CI runs on `docs/manual`). The four new rows of section 4 are seen
RED on `4bfa645f` before the `friendly.rs` change; the guard row passes on both.

## 9. How this document was checked

- RED: every command of section 1 run on `4bfa645f` (binary built `--locked`, cargo 1.85.0), output pasted.
- GREEN (scratch, reverted, not committed): the section 3 arm built and the section 1 commands rerun; R1 text on all
  index-`s` rows, control unchanged, rc 2 throughout.
- Not checked here: the section 4 test bodies as written, the full gate, clippy, fmt, the manual build.
