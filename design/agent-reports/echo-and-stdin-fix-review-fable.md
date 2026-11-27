# Final review (fable): commits 7ac8c24a + 582e9e4e (withhold non-HRP prefixes; count path flags that are stdin)

Reviewer: independent adversarial review (Claude Fable 5.2), 2026-11-27.
Worktree `/home/user/wt-address-search`, diff `8339e995..582e9e4e`.
Binary: `cargo build --locked -p mnemonic-toolkit --bin mnemonic` at 582e9e4e (no source file was modified).

Verdict: **0 Critical, 1 Important, 4 Minor, 7 Nit.**

The two reported defects are fixed on every path the reports name, and the r1 findings are
folded correctly. I found no remaining route by which a phrase, hex entropy or an ms1 card
typed into `--ms1`, `--mk1`, `--md1`, `--ms1-stdin`, a positional card argument,
`convert --from ms1=/mk1=`, or a verify-bundle card (flag, positional or `--bundle-json`)
reaches stderr, stdout or `--json` through an error. What is still open sits beside those
paths: one mk1-card field (`restore --cosigner`) that echoes its value in full, three raw
mk-codec `InvalidHrp` renderings the "one rule" does not reach, two verbs whose stdin count
is still incomplete (both fail closed), and tests that are weaker than their doc comments say.

## Method

- Harness 1: 10,825 invocations. 25 secrets (a 12-word phrase, hex entropy with `1`s, a valid
  ms1 card and 19 damaged or wrong-HRP variants of it, an xprv, a WIF) through about 230 command
  shapes covering every verb, by argv (`--allow-argv-secret`) and by stdin where the flag takes
  `-`. A hit is any 6-character window of the secret (for cards: of the payload after
  `ms10entrs`) in stdout or stderr, compared case-insensitively.
- Harness 2: 5,504 invocations with real fixtures (a real single-sig bundle, a real 2-of-2
  multisig bundle, a template bundle), so that the card flags are reached with the other
  cards valid: verify-bundle `--ms1/--mk1/--md1`/positional/`--bundle-json`, `restore
  --md1/--cosigner`, address-of-seed `--cosigner/--descriptor-from`, `--target-xpub`,
  `word-card --from`, multi-chunk "good chunk + secret" sets, `repair --max-indel`.
  Extra secrets: `mk1`+hex, `mk1 `+phrase, `md1`+hex, a SeedQR digit string, a 24-word
  phrase, and hex of the shape `ab1…` (two letters before the only `1`).
- Every hit was read. All rc=0 hits are by-design output (`repair` printing the corrected
  card, `bundle` printing ms1, `seed-xor combine`, verify-bundle `expected`/`actual`).
- Targeted tests run: `cargo test --locked -p mnemonic-toolkit --test all --
  cli_secret_echo_and_stdin cli_hrp_case_insensitive second_stdin_reader_via_descriptor_file
  ms1_slot_holding` (21 passed); `--lib -- hrp wrong_hrp invalid_hrp` (18 passed).

## Important

### I1. `restore --cosigner` echoes its whole value, including a full ms1 card or phrase

`src/cmd/restore.rs:4869`:
`bad(format!("--cosigner expects @N=<mk1|xpub>, got `{spec}`"))`.
`--cosigner` is an mk1-card field. On the keyed-md1 path a bare value (no `@N=`) is printed
back verbatim. It needs no `--allow-argv-secret`, and no argv advisory fires, because
`--cosigner` is not a secret flag:

```
mnemonic restore --from phrase=@env:GOODP --md1 <4 chunks of a 2-of-2 md1> \
    --cosigner ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6
error: --cosigner expects @N=<mk1|xpub>, got `ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6`

… --cosigner 'legal winner thank year wave sausage worth useful legal winner thank yellow'
error: --cosigner expects @N=<mk1|xpub>, got `legal winner thank year wave sausage worth useful legal winner thank yellow`
```

Failure scenario: a user (or a GUI form with a seed-card field next to the cosigner-card
fields) puts the seed card or phrase into a cosigner field. The complete secret lands on
stderr and in any log of it. The value is argv-only (`--cosigner` has no stdin form), which
is why r1 graded the 12-character positional echo Minor; here the echo is the whole secret,
and the fold's own standard for the positional was "no payload character is shown".
This is pre-existing and outside the HRP-diagnostic code the two commits touch, so it is a
gap in the fix's coverage, not a regression. `xpub-search address-of-seed --cosigner`
already does this right ("the value is not repeated").

Fix: drop `{spec}` from the message at 4869 (and at `restore.rs:1860`, which echoes an
`@`-prefixed value with no `=`), and add a CLI test with an ms1 card and a phrase.

## Minor

### M1. Three raw mk-codec `InvalidHrp` renderings bypass `hrp_is_echoable`

The doc comment on `hrp_is_echoable` (`src/repair.rs:131-137`) says "Every HRP diagnostic
uses this one rule". Three sites print `mk_codec::Error` with its own `Display`
(`invalid HRP: {0}`, the whole lower-cased input when it has no `1`, else everything before
the last `1`; `vendor/mk-codec/src/string_layer/bch.rs:668-673`):

- `src/wallet_import/json_envelope.rs:348` — `bundle --import-json` and
  `export-wallet --from-import-json`, both of which take `-` (stdin):
  ```
  cat envelope_with_hex_in_mk1.json | mnemonic bundle --network mainnet --import-json -
  error: --import-json: mk1[0] decode failed: invalid HRP: 3a1c9e5b7d2f40816c9ab3e4f5d607
  ```
  (30 of 32 hex characters; same text from `export-wallet --from-import-json -`.)
- `src/cmd/restore.rs:4896` — `--cosigner @N=mk1…`:
  ```
  mnemonic restore … --cosigner @1=mk13a1c9e5b7d2f40816c9ab3e4f5d60718
  error: --cosigner @1 mk1 decode: invalid HRP: mk13a1c9e5b7d2f40816c9ab3e4f5d607
  ```
- `src/cmd/restore.rs:2605` — same pattern on the unassigned-cosigner path.

This is the r1 I1 defect (mk-codec carries the input in `InvalidHrp`) on routes the fold did
not convert. The triggers are unlikely (a secret inside an envelope's `mk1` array, or a
value that starts with `mk1`), which keeps it Minor. Fix: render through
`friendly::friendly_mk_codec` at all three sites.

Not a leak, for the record: `{e:?}` of `ms_codec::Error` at `src/wallet_import/overlay.rs:161`,
`src/cmd/bundle.rs:2477` and `:3735`. ms-codec's `Debug` delegates to its sanitised `Display`,
and `ms_codec::decode` checks the checksum before the HRP, so `WrongHrp` there only ever names
the HRP of a string that is already a valid codex32 codeword. Measured: a damaged card, a
phrase and hex give "invalid short checksum (input withheld)", `InvalidCase(Lower, ' ')` and
"string length 32 outside v0.1 set".

### M2. Half of the new CLI echo test cannot fail, and the doc comment overstates it

`tests/cli_secret_echo_and_stdin.rs::a_phrase_in_a_card_field_is_not_echoed`:

- Four of its eight cases never reach an HRP diagnostic, because a phrase has spaces and no
  `1` and is refused earlier by unchanged code. Current output:
  - `repair --mk1 -` and `repair --ms1 -`: `chunk 0 parse failed before correction could
    run: missing bech32 separator '1'`
  - `inspect --ms1 -` and `convert --from ms1=`: `ms1 string length 82 not in v0.1 set`
  
  These pass at 8339e995 too. The four real cases are `repair --mk1 <argv>`,
  `inspect --mk1 -`, `convert --from mk1=` and the positional.
- The helper asserts only "exit code is not 0" and "no `aband`". ms-codec caps `WrongHrp.got`
  at 4 characters (`aban`), so the needle `aband` could not detect the pre-fix ms-codec echo
  even on a path that reached it.
- The doc comment lists verify-bundle; there is no verify-bundle case. The six
  `{:?}` → `friendly_*` changes in `src/cmd/verify_bundle.rs` (2470, 2592, 2814, 3049, 3405,
  3562) have no test at all: no test in `tests/` pins a decode row's text, before or after.
- r1 asked for a hex-entropy probe. None was added at CLI level. Hex with a `1` is the input
  that actually reaches `RepairError::HrpMismatch` and `friendly_mk_codec` from stdin:
  ```
  echo 3a1c9e5b7d2f40816c9ab3e4f5d60718 | mnemonic repair --ms1 -
  error: repair: chunk 0 HRP mismatch — expected 'ms', found a prefix that is not a card HRP (input withheld) …
  ```
  That behaviour is correct today and is pinned only by the unit test in `src/repair.rs`.

Fix: add hex cases (`repair --ms1 -`, `repair --mk1 -`, `inspect --mk1 -`,
`convert --from mk1=-`), assert `withheld` as the address-of-seed test does, and add one
verify-bundle `--bundle-json` case with hex in `mk1`.

The other new tests are sound. `candidates_file_on_stdin_is_a_second_reader`,
`import_wallet_path_flags_on_stdin_are_counted`,
`second_stdin_reader_via_descriptor_file_dev_stdin` (asserts exit 2 and the reader name; the
unfixed binary exits 0 or 4) and `ms1_slot_holding_a_phrase_is_not_echoed` (asserts
`withheld`) all fail without the fix.

### M3. Two verbs still have a stdin reader the count misses (both fail closed)

Path-taking flags across `src/cmd`, with their guard:

| Verb | Path flags | Counted against |
|---|---|---|
| address-of-seed | `--descriptor-file` | phrase/ms1 stdin, passphrase, `--descriptor-from md1=-`, each `--from x=-` — complete |
| passphrase-of-xpub | `--passphrase-candidates-file` | phrase/ms1 stdin, passphrase — complete |
| import-wallet | `--blob`, `--bsms-round1`, `--bsms-encryption-token`, `--decrypt-password-file` | each other and the password stdin forms — complete |
| silent-payment, nostr | `--secret-file` | passphrase / `--secret-stdin` — complete (clap group for nostr) |
| electrum-decrypt | `--decrypt-password-file` | `--ciphertext -` — complete |
| verify-message | `--message-file` | clap group with `--message-stdin` — complete |
| **bundle** | `--descriptor-file`, `--import-json <FILE>` | passphrase only; **not `--slot @N.<secret>=-`** (`src/cmd/bundle.rs:228-244`) |
| **verify-bundle** | `--descriptor-file`, `--bundle-json` | passphrase, `--ms1 -`, slot stdin; **not `--from <node>=-`** (`src/cmd/verify_bundle.rs:288-341`) |
| export-wallet | `--from-import-json <FILE>` | nothing (public data only) |
| build-descriptor | `--spec <PATH>` | single reader |

Path detection itself is sound: `path_is_stdin` matches by device and inode, so a symlink to
`/dev/stdin`, `/dev/./stdin`, `/proc/<pid>/fd/0` and a regular file that is also redirected
to fd 0 are all caught (measured on passphrase-of-xpub and silent-payment).

Evidence for the two gaps:

```
cat desc.txt | mnemonic bundle --network mainnet --descriptor-file /dev/stdin --slot @0.phrase=-
error: this descriptor has no keys to engrave as a cosigner card — a keyless script …     (exit 2)

printf '%s\n' "$PHRASE" | mnemonic bundle --network mainnet --import-json /dev/stdin --slot @0.phrase=-
error: --import-json: envelope JSON parse: EOF while parsing a value at line 1 column 0   (exit 1)

cat tpl.json | mnemonic verify-bundle --network mainnet --from phrase=- --passphrase @env:PW \
    --cosigner … --bundle-json /dev/stdin
error: BIP-39 word count 0 invalid (must be 12, 15, 18, 21, or 24)                        (exit 1)
```

None of these silently uses a wrong secret: the second reader gets an empty stream and the
run stops. The user gets a misleading error instead of the "only one input can come from
stdin" refusal. The FOLLOWUP the commit closes says "every verb with `--descriptor-file`";
bundle is not fully covered. Fix: add the slot-stdin reader to bundle's list and the
`--from <node>=-` reader to verify-bundle's.

### M4. Public-key and public-data fields echo a pasted secret in full

Not card fields, and all but one argv-only, so outside the two reports. Listed because the
task asks about any verb, and because one arrives on stdin:

- `xpub-search address-of-seed --target-address <phrase>` →
  `error: --target-address legal winner … yellow: legacy address base58 string`
- `restore --expect-xpub <v>` / `--expect-fingerprint <v>` →
  `restore: xpub mismatch — derived xpub6…, expected <v>`
- `export-wallet --descriptor <v>`, `xpub-search account-of-descriptor --descriptor <v>` →
  `unrecognized name '<v>'`
- `xpub-search account-of-descriptor --descriptor-from <v>` (no `=`) →
  `--descriptor-from expects <node>=<value> form; got `<v>``
  (`src/cmd/xpub_search/account_of_descriptor.rs:299`)
- **stdin:** `printf '%s\n' "$PHRASE" | mnemonic import-wallet --blob - --format bsms` →
  `bsms: parse error: expected header `BSMS 1.0` on line 1, got "legal winner … yellow"`;
  `--format coldcard-multisig` → `line 1: bare xpub `<v>` has no associated derivation path`.

## Nit

- **N1. The two-letter rule is safe; its stated reason is not the reason.**
  The echo is bounded on every gated path to a known HRP or two ASCII letters, lower-cased:
  at most about 9.4 bits, and only for an input whose text before its last `1` is exactly two
  letters. Measured: `echo ab1cdef0234567890abcdef023456789 | mnemonic repair --ms1 -` prints
  `found 'ab'` (8 bits of a 128-bit hex seed; about 0.1% of random hex seeds have that shape).
  A phrase has no `1`, a SeedQR is digits, and a codex32 data part cannot contain `1`, so an
  ms1 payload never appears. The comment's justification ("No BIP-39 word has fewer than
  three letters", `src/repair.rs:135-136`) is true but beside the point; the bound is the reason.
- **N2. `bcrt` can be reported for a longer prefix.**
  ms-codec truncates `WrongHrp.got` to 4 characters before the toolkit sees it, so a prefix
  that merely starts with `bcrt` renders as `got "bcrt"`. No secret has that shape.
- **N3. import-wallet refusal wording.**
  The new block (`src/cmd/import_wallet.rs:390-417`) names readers without the path, repeats
  a name (`--bsms-encryption-token and --bsms-encryption-token all read stdin`), and counts
  `--bsms-round1 -` as stdin although that flag takes only a file. passphrase-of-xpub says
  `--passphrase-candidates-file /dev/stdin` when the path was a regular file redirected to
  fd 0. Legitimate use is unaffected: token from a file with `--blob -`, token on stdin with
  the blob a file, and `--blob /dev/stdin` alone all pass the guard.
- **N4. Stale test name.**
  `cli_hrp_case_insensitive::unknown_hrp_long_positional_echo_truncated` no longer tests a
  truncation.
- **N5. `UnknownHrp` says "positional argument" for `word-card --from`** (pre-existing wording,
  `src/word_card_adapter.rs:127`).
- **N6. Manual.**
  Only the `repair` refusal row was updated
  (`docs/manual/src/40-cli-reference/41-mnemonic.md:4114`), and it is accurate; the example
  at 4203 (`found 'ns'`) still holds. Nothing in `docs/manual/src` or `docs/manual-gui`
  describes a message the fold changed, so nothing is now wrong. The new refusals
  (address-of-seed `--descriptor-file`, `--passphrase-candidates-file`, import-wallet path
  forms) and the withheld texts are undocumented; line 2065 still lists only the `-` spellings.
- **N7. verify-bundle `decode_error` / `detail` text changed shape** (`InvalidHrp("…")` →
  `mk1 wrong HRP: …`, `ChunkSetEmpty` → `md1 chunk set empty`) in `--json` as well as text.
  No test, no manual text, and no source in `mnemonic-gui` or `refugium-wallet` matches on
  the old strings. Exit codes are unchanged everywhere I looked: HRP errors 2, the stdin
  refusals 1 (2 on address-of-seed, as its sibling test expects).

## Answers to the brief

1. **Remaining echo through a card or secret field:** none on the named fields. One on a
   card field the reports did not name (I1), three ungated mk-codec renderings (M1), and
   full-value echoes on public fields (M4). Single-character echoes remain
   (`InvalidChar('b')`, `hex-decode: Invalid character 'K' at position 0`, `seedqr: invalid
   character at position N: 'e'`); each shows one character.
2. **Two-letter rule:** safe (N1).
3. **Uncounted stdin readers:** bundle slot stdin and verify-bundle `--from x=-` (M3); both
   fail closed.
4. **Did the fold break anything:** no. No wrong exit code, no refusal of legitimate use, no
   manual text made false.
5. **Do the new tests test what they claim:** the stdin tests and the address-of-seed echo
   test do; half of `a_phrase_in_a_card_field_is_not_echoed` does not (M2).

Out of scope, not verified against the spec: `bundle --descriptor-file <concrete-key
descriptor> --slot @0.phrase=-` exits 0 with `"mode":"watch-only"` and no word about the
phrase it read.
