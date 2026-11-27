# Review r1: commit 7ac8c24a (withhold non-HRP prefix; count --descriptor-file /dev/stdin)

Reviewer: independent adversarial review (Claude Opus 5.5), 2026-11-27.
Worktree `/home/user/wt-address-search`, branch `claude/address-of-seed` (toolkit PR #230).
Binary: `cargo build --locked -p mnemonic-toolkit --bin mnemonic` at 7ac8c24a.

Verdict: **0 Critical, 1 Important, 3 Minor, 3 Nit.** The two filed defects are fixed on
the paths the commit names (the ms1 slot, `repair`'s HrpMismatch and address-of-seed's
`--descriptor-file`). The echo is still open in the mk1 slot, and a full phrase or hex
entropy typed there is printed on stderr even when it arrives on stdin.

Gates run: `cargo fmt --check -p mnemonic-toolkit` clean;
`cargo clippy --locked -p mnemonic-toolkit --all-targets -- -D warnings` clean;
`cargo test --locked -p mnemonic-toolkit --lib -- hrp wrong_hrp friendly` 34 passed;
`cargo test --locked -p mnemonic-toolkit --test all -- cli_positional_hrp_autodetect
cli_hrp_case_insensitive cli_indel cli_env_var_sentinel second_stdin ms1_slot_holding repair
inspect` 205 passed, 0 failed, and both new tests ran.

Method for (a): a harness ran about 150 invocations. Each fed one of five secrets (a
12-word phrase, a 24-word phrase, hex entropy containing `1`, an xprv and a WIF) through
stdin, `@env:` and argv with `--allow-argv-secret`. The checker flagged any 6-character
window of the secret, compared case-insensitively, that appeared in stdout or stderr.
`repair`/`inspect` `--ms1/--mk1/--md1` do not resolve `@env:`: the literal sentinel is
validated. For them, stdin and argv are the channels that carry a secret.

## Important

### I1. The mk1 slot still echoes a mistyped secret in full, including from stdin

`friendly_mk_codec` (src/friendly.rs:182) renders `mk_codec::Error::InvalidHrp(s)` as
`mk1 wrong HRP: got {s:?}`. mk-codec 0.4.1 puts the whole lower-cased input into that
variant when the input has no `1`, and everything before the last `1` otherwise
(vendor/mk-codec/src/string_layer/bch.rs:666-673). Unlike ms-codec's `WrongHrp`, it has no
4-character cap. The commit gated only the ms-codec arm.

Two things let a value reach that arm without passing `validate_flag_hrp`. First,
`repair`/`inspect` validate before the `-` stdin expansion (src/repair.rs:303-311 exempts
`-`), and nothing re-validates the expanded lines. Second, `convert --from mk1=` never
calls `validate_flag_hrp` (src/cmd/convert.rs:1796).

These commands reproduce it (stdin is the channel a user picks to keep a secret off argv):

```
printf '%s\n' 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' | mnemonic inspect --mk1 -
error: mk1 wrong HRP: got "abandonabandonabandonabandonabandonabandonabandonabandonabandonabandonabandonabout", expected "mk"

printf '%s\n' 7f1f7e7d7c7b7a797877767574731f10 | mnemonic inspect --json --mk1 -
error: mk1 wrong HRP: got "7f1f7e7d7c7b7a797877767574731f", expected "mk"

printf '%s\n' 7f1f7e7d7c7b7a797877767574731f10 | mnemonic convert --from mk1=- --to xpub
error: mk1 wrong HRP: got "7f1f7e7d7c7b7a797877767574731f", expected "mk"      (also with --json)

mnemonic convert --from mk1=7f1f7e7d7c7b7a797877767574731f10 --to xpub              (argv; same text)
```

For a phrase, `convert --from mk1=-` echoes only the first word ("abandon"). For hex
entropy it echoes 30 of the 32 hex characters, and for a 128-bit seed that is effectively
all of it.

A third route goes through verify-bundle's check rows, which print the codec error with
`{:?}` (src/cmd/verify_bundle.rs:2592, and the same pattern at 2470, 3049, 3405 and 3562).
A `--bundle-json` whose `mk1` entry holds hex entropy prints it in full, in text and in
`--json`:

```
mnemonic verify-bundle --network mainnet --template bip84 --slot @0.phrase=@env:S --bundle-json b2.json
mk1_decode: fail InvalidHrp("7f1f7e7d7c7b7a797877767574731f")
```

The `--json` output carries the same string in `"detail"` and `"decode_error"`.

The xprv and WIF are mixed-case, so mk-codec refuses them with `MixedCase` and nothing is
echoed. Lower-case secrets leak: BIP-39 phrases, hex entropy and SeedQR digits containing
`1`.

Failure scenario: a GUI or user puts the seed into the mk1 field (it sits next to the ms1
field) and pipes it on stdin. The phrase lands in stderr and in any log of it. This is the
same defect the commit fixes for ms1.

Fix:
- Gate the `E::InvalidHrp` arm with `hrp_is_echoable` and add a withheld arm, as was done
  for ms-codec.
- Re-run `validate_flag_hrp` on the stdin-expanded values in repair/inspect, or validate
  after expansion.
- Render verify-bundle's mk1 decode rows through `friendly_mk_codec`, not `{:?}`.
- Add a CLI test for `inspect --mk1 -` and `convert --from mk1=-` with a phrase and with
  hex entropy.

## Minor

### M1. Positional HRP autodetect still echoes the first 12 characters of a secret

`ToolkitError::UnknownHrp` (src/error.rs:999-1013) truncates to 12 characters plus `…`.
It is raised by `classify_hrp_prefix` (src/repair.rs:120) for repair, inspect and
verify-bundle positionals, and by `chunks_to_canonical` (src/word_card_adapter.rs:127) for
`word-card --from`. No `--allow-argv-secret` is needed:

```
mnemonic repair 7f1f7e7d7c7b7a797877767574731f10
error: positional argument '7f1f7e7d7c7b…' does not begin with a recognized HRP prefix (expected one of: ms1, mk1, md1)
mnemonic inspect 'abandon abandon … about'           -> 'abandon aban…'   (2 of 12 words)
mnemonic verify-bundle --network mainnet --template bip84 KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn
                                                      -> 'KwDiBf89QgGb…'   (about 64 bits of the WIF)
mnemonic word-card --from 7f1f7e7d7c7b7a797877767574731f10   -> '7f1f7e7d7c7b…'
```

12 hex characters are 48 bits of a 128-bit entropy. Positionals are argv-only: `-` and
`@env:` are not accepted, so the secret is already exposed through /proc and shell history,
and stderr adds a second copy (for example in a log). That keeps this Minor. It still
contradicts the principle the commit states ("anything else may be the start of a secret
… so it is withheld"). Fix: echo the prefix only when `hrp_is_echoable` on the part before
the last `1`, and otherwise withhold, as `HrpMismatch` now does.

### M2. The rules for naming a typo differ between paths and between the manual and the code

- `validate_flag_hrp` (src/repair.rs:207-211) filters on `ECHOABLE_HRPS` only.
  `RepairError::HrpMismatch` and `friendly_ms_codec` use `hrp_is_echoable`, which also
  allows a one-letter typo. So the same `ns1…` card gives two different diagnostics:
  - `printf ns10entrs…v7f | mnemonic repair --ms1 -` prints
    `found 'ns' … did you mean 'ms'?`
  - `mnemonic --allow-argv-secret repair --ms1 ns10entrs…v7f` (and `inspect --ms1`,
    `verify-bundle --ms1`, `xpub-search --ms1`/`--ms1-stdin`) prints "input withheld" and
    no suggestion.
  
  Before the commit, the argv form named `'ns'`.
- `suggest_hrp` returns `None` when the typo is ambiguous. Every second-letter typo of `ms`
  (`mq`, `mz`, `mb`, …) is one letter from `ms`, `mk` and `md` alike, so all of them are now
  withheld: `printf mq10entrs…v7f | mnemonic repair --ms1 -` prints "found a prefix that is
  not a card HRP (input withheld)". Before the commit it printed `found 'mq'`. The manual
  row docs/manual/src/40-cli-reference/41-mnemonic.md:4114 says `YY` "is named … when it is
  … a one-letter typo of one", which is false for these.
- Public prefixes such as `npub`, `nsec`, `lnbc`, `sp` and an `xpub…` base58 head are now
  withheld too. That costs a little diagnostic value and no secret.

A 2-character prefix before a `1` at index 2 leaks no more than about 10 bits. A simpler
rule would be: echo when the found prefix is 1 to 4 ASCII alphanumerics, or a known HRP.
That keeps every typo diagnostic and still withholds phrases, hex and base58 heads. At the
least, use `hrp_is_echoable` in `validate_flag_hrp` and correct the manual sentence.

No test or manual text asserts an old message shape that the commit broke.
cli_positional_hrp_autodetect.rs (lines 158-173, 265-266 and 421-422) asserts
`got 'mk'`/`got 'ms'`, which still pass. cli_indel.rs:414 asserts only "HRP mismatch".
cli_env_var_sentinel.rs:619 asserts only the exit code. I found no mnemonic-gui source or
test that matches on these strings.

### M3. Other verbs have a path flag that can read fd 0 without being counted

(c) For address-of-seed the count is complete:
- `--phrase-stdin`/`--ms1-stdin`
- `--passphrase -`/`--passphrase-stdin`
- `--descriptor-from md1=-` (an exact match, the same test `md1_template` uses)
- `--descriptor-file` through `path_is_stdin`
- each `--from x=-` (the same exact-value rule `parse_from_input` uses, which does not trim)

`--cosigner` never reads stdin. `--ms1 -` is the literal `-` and is not read from stdin:
it fails with "ms1 string length 1". address-of-seed has no positional. Other verbs miss
some path forms:

- **xpub-search passphrase-of-xpub.** `--passphrase-candidates-file`
  (src/cmd/xpub_search/passphrase_of_xpub.rs:102 and 311) is not in its
  `refuse_second_stdin` list (293-302). Two scenarios:
  - Through a pipe: `printf '<phrase>\n' | mnemonic xpub-search passphrase-of-xpub
    --phrase-stdin --passphrase-candidates-file /dev/stdin --target-xpub <x>` reports "no
    candidates (all lines blank)".
  - With a regular-file redirect: `… < phrase.txt` re-opens the file from offset 0 and
    reports "no candidate … produced the target xpub (1 candidate(s) tried)". The candidate
    it tried was the seed phrase itself.

  This fails closed, but the user could wrongly conclude the passphrase is not in their list.
- **import-wallet.** `--bsms-encryption-token`, `--blob` and `--bsms-round1` are counted
  only when the value is spelled `-` (src/cmd/import_wallet.rs:335-351).
  `--bsms-encryption-token /dev/stdin` together with `--blob -`, or with
  `--decrypt-password-stdin`, is not refused. The token read drains stdin first, and the
  later reader gets an empty buffer, so the blob or password is empty. This should fail
  closed on parse, width or MAC, but it is the same F-687 class.
- bundle, verify-bundle, electrum-decrypt and silent-payment already use `path_is_stdin`.
  nostr and verify-message put their file and stdin forms in one exclusive group. export-wallet
  has a single reader.

## Nit

- **N1. A test doc comment was moved onto the wrong test.**
  The new test was inserted under the existing doc comment of
  `repair_via_ms_codec_wrong_hrp_found_is_bounded` (src/repair.rs:3322-3326). That comment
  ("Regression (ms-codec 0.4.4 pin bump) …") now introduces
  `hrp_mismatch_withholds_a_secret_typed_into_the_ms1_field`, and the old test has none.
  Move the new test above the old comment.
- **N2. One case of the new address-of-seed test passes without the fix.**
  In `ms1_slot_holding_a_phrase_is_not_echoed`
  (tests/cli_xpub_search_address_of_seed.rs:1955), the case
  `--ms1 @env:SEED_A` is vacuous. `xpub-search --ms1` does not resolve `@env:`
  (seed_intake.rs:106-109 validates the literal), so the phrase is never read and the
  assertion passed before the fix too: the old message was `got '@env:seed_a'`. Only the
  `--ms1-stdin` case exercises the fix. Replace it with `--from ms1=@env:SEED_A` or
  `--allow-argv-secret --ms1 <phrase>`, and consider CLI cases for `repair --ms1 -` and
  `verify-bundle --ms1 -`.
- **N3. No test checks that the ms-codec arm still names a real HRP.**
  `friendly::tests::ms_codec_wrong_hrp` (src/friendly.rs:541) uses `got: "mq"`, which now
  takes the withheld arm. Its assertions pass on both arms, so no test pins that
  `friendly_ms_codec` still names an echoable HRP such as `mk`. Add one.
- **N4. The refusal message spells the flag differently from bundle.**
  address-of-seed names the reader `--descriptor-file`, while bundle says
  `--descriptor-file /dev/stdin` (src/cmd/bundle.rs:243). The new test accepts either.
