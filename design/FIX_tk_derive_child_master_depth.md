# FIX-tk-derive-child-master-depth: `derive-child` refuses a non-master xprv/tprv

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: small (one Opus review round). Finding: bh-tk-I4 (bughunt-1119), graded Important (wrong derived secret, exit 0; grade-tk.md:28). Base: origin/master 04c56bfa.

## 1. Defect

`mnemonic derive-child --from xprv=<value>` treats any xprv/tprv as the BIP-85 master. `cmd/derive_child.rs:174-175` parses the value with `Xpriv::from_str` and never checks `depth`. A depth-1 key (an account-level or any child xprv) is used as the BIP-85 root, and the command prints a child secret (exit 0) that no wallet derives from the holder's seed. A holder who later recovers from the seed phrase cannot reproduce the secret.

BIP-85 (bitcoin/bips `bip-0085.mediawiki`, Specification, checked 2026-11-22): "We assume a single BIP32 master root key." and "a unique private key is derived from the BIP32 master root key". In BIP-32 the master key is `m`, depth 0.

The sibling defect in `silent-payment` was fixed by FIX_bughunt_tk_g2 #56 (`design/FIX_bughunt_tk_g2.md:323-347`, code `cmd/silent_payment.rs:126-133`); `derive-child` was missed.

Survey of the other commands that take an xprv (asked by grade-tk.md:52): only `derive_child.rs:174` and `silent_payment.rs:126` parse a holder-supplied xprv as a master. `convert --from xprv` (`convert.rs:1556`) only maps to `xpub` / `fingerprint`, which is correct at any depth. Every other `Xpriv` in the crate is built by `Xpriv::new_master` from a seed. No other call site changes.

## 2. Holder-visible behaviour

In `cmd/derive_child.rs::run`, the `NodeType::Xprv` arm (lines 174-175), after the parse succeeds, an xprv/tprv whose `depth != 0` is refused:

~~~text
error: derive-child: the xprv/tprv is at depth <d>, not a master key (depth 0); supply the master xprv/tprv, or the seed phrase it came from
~~~

PROPOSED (architect confirms): the text and exit code. The text is #56's text with the prefix `derive-child:` and the tail cut to the inputs `derive-child` accepts (`xprv=` or `phrase=`; it takes no ms1 or entropy, so #56's "seed phrase, ms1 or entropy" would name inputs this command refuses).

Decided edges:

- `<d>` is the decimal depth byte of the key (1..=255).
- Exit 1, `ToolkitError::BadInput` (kind `BadInput`). Reasons: SPEC_derive_child_v0_7.md:122 classes an invalid master xprv as exit 1 (BadInput); the existing `--from` node refusal at `derive_child.rs:200-203` is BadInput exit 1 with the same `derive-child:` prefix; #56 exits 1. The exit-2 `DeriveChild*` family (SPEC :124) covers application and length refusals, not the master input. No new `ToolkitError` variant.
- stdout is empty. The private-key stdout warning (`warning: stdout carries private key material ...`) is not printed.
- The refusal is inside the match arm, so it comes before the passphrase warning at `derive_child.rs:206-211`: with `--from xprv=<non-master>` and a passphrase, only the refusal line is printed (no `warning: --passphrase ignored ...`).
- The key is refused whatever its network (xprv or tprv) and whatever `--network` says.
- Earlier refusals keep their order and text: the argv-secret refusal (exit 2, before parsing), the `@env:` resolution errors, the single-stdin conflict (`derive_child.rs:136-140`), and the `Xpriv::from_str` parse error for a malformed key.
- A depth-0 xprv/tprv and every `--from phrase=` input behave exactly as today (BIP-85 vectors unchanged).
- `--help`, the `gui-schema` output and the GUI are unchanged (no flag, argument or doc-comment change; the `--from` doc comment at `derive_child.rs:22-29` is not edited).

## 3. Code delta (production)

1. `crates/mnemonic-toolkit/src/cmd/derive_child.rs` (the only production file): in the `NodeType::Xprv` arm, bind the parsed `Xpriv`, return `ToolkitError::BadInput(format!("derive-child: the xprv/tprv is at depth {}, not a master key (depth 0); supply the master xprv/tprv, or the seed phrase it came from", x.depth))` when `x.depth != 0`, else yield it. The comment cites BIP-85 "BIP32 master root key" and this FIX.

Source files this FIX changes (for build ordering; tk #15 `fix/tk-stdout-write-errors` touched `derive_child.rs` only at its stdout write, line 306, and is already on the base, GREEN f2d1caf5):
- `crates/mnemonic-toolkit/src/cmd/derive_child.rs`
- `crates/mnemonic-toolkit/tests/cli_derive_child.rs`
- `design/SPEC_derive_child_v0_7.md`
- `docs/manual/src/40-cli-reference/41-mnemonic.md`

## 4. RED rows

Run on base 04c56bfa with `target/debug/mnemonic` built `--locked` (cargo 1.85.0). Keys: `X` = BIP-32 test vector 1 chain `m/0'` (depth 1), `xprv9uHRZZhk6KAJC1avXpDAp4MDc3sQKNxDiPvvkX8Br5ngLNv1TxvUxt4cV1rGL5hj6KCesnDYUhd7oWgT11eZG7XnxHrnYeSvkzY7d2bhkJ7`; `T` = the same key with tprv version bytes (`04358394`), `tprv8bxNLu25VazNnppTCP4fyhyCvBHcYtzE3wr3cwYeL4HA7yf6TLGEUdS4QC1vLT63TkjRssqJe4CvGNEC8DzW5AoPUw56D1Ayg6HY4oy8QZ9`. Both are public test-vector keys holding no funds.

| Row (test name in `tests/cli_derive_child.rs`) | Command | Today (seen on base) | After the fix |
|---|---|---|---|
| `r1_non_master_xprv_stdin_refused` | `echo "$X" \| mnemonic derive-child --from xprv=- --application hex --length 16 --index 0` | stdout `ad95c2cd242617af788dc26e9ae95b82`, stderr the private-key warning, exit 0 | stdout empty; stderr exactly `error: derive-child: the xprv/tprv is at depth 1, not a master key (depth 0); supply the master xprv/tprv, or the seed phrase it came from`; exit 1 |
| `r2_non_master_xprv_env_sentinel_refused` | `XP="$X" mnemonic derive-child --from xprv=@env:XP --application bip39 --length 12 --index 0` | stdout `used moral panda trip raccoon soon render bulb remember math hello unable`, exit 0 | same stderr line, stdout empty, exit 1 |
| `r3_non_master_tprv_refused` | `echo "$T" \| mnemonic derive-child --from xprv=- --application xprv --length 0 --index 0 --network signet` | stdout `tprv8ZgxMBicQKsPcuHBxrKgWm53dh7gREHMpgnaNC4HZSp9n11qtFAgxucAYvtiTvWnu7MiP29CbMjcJwwzuLMT5b3GavcAp3twiGEUggFpdBQ`, exit 0 | same stderr line, stdout empty, exit 1 |
| `r4_non_master_xprv_with_passphrase_refused_without_warning` | `echo "$X" \| PP=foo mnemonic derive-child --from xprv=- --application hex --length 16 --index 0 --passphrase @env:PP` | stderr `warning: --passphrase ignored on --from xprv (no BIP-39 mnemonic to extend)`, stdout `ad95c2cd242617af788dc26e9ae95b82`, exit 0 | stderr is only the refusal line (no passphrase warning), stdout empty, exit 1 |

Each test asserts exit code 1, empty stdout, and stderr equal to the refusal line (trimmed). Each RED row fails on base on the exit-code assertion; the builder runs them and records the failure before the GREEN commit.

Regression (not new rows): the existing BIP-85 vector cells of `tests/cli_derive_child.rs` (all use the depth-0 `MASTER_XPRV`) and the `derive_child.rs` unit tests (depth-0 master at :428-431) pass unchanged.

## 5. Tests that change

None. Searched every test that drives `derive-child` (`cli_derive_child.rs`, `bughunt_s4.rs`, `cli_env_var_sentinel.rs`, `cli_f687_passphrase_channels.rs`, `stdout_write_errors.rs`, `cli_output_class.rs`, `cli_argv_leakage.rs`, `cli_p3_argv_refusal.rs`, `cli_secret_in_argv_warning.rs`, `readme_dice_kat_parity.rs`): every xprv they pass is a depth-0 `xprv9s21...` / `tprv8Zgx...` master. The non-master literals in the test tree (`bip32_vectors.rs`, `cli_convert_*.rs`, `cli_import_wallet_bitcoin_core.rs`) do not reach `derive-child`. Four tests are added (section 4).

## 6. Spec delta

`design/SPEC_derive_child_v0_7.md` §7, after line 122 (`**Invalid master xprv refusal:** ...`), insert one paragraph:

`**Non-master xprv refusal (FIX_tk_derive_child_master_depth, bh-tk-I4):** BIP-85 derives from "a single BIP32 master root key", so an xprv/tprv whose depth is not 0 is refused after it parses, before any derivation or warning: ``error: derive-child: the xprv/tprv is at depth <d>, not a master key (depth 0); supply the master xprv/tprv, or the seed phrase it came from``; exit 1 (BadInput class), stdout empty.`

## 7. Manual delta (toolkit manual, lockstep)

`docs/manual/src/40-cli-reference/41-mnemonic.md:1946`, the `derive-child` `--from <FROM>` row. PROPOSED (architect confirms), mirroring #56's A3 sentence. Old cell:

`` `xprv=<value>` or `phrase=<bip39>` (with `--passphrase` + `--language`); `=-` to read from stdin ``

New cell:

`` `xprv=<value>` or `phrase=<bip39>` (with `--passphrase` + `--language`); `=-` to read from stdin. A non-master xprv/tprv (depth ≠ 0, e.g. an account-level key) is refused (exit 1) ``

Not changed: `docs/manual-gui` (pinned to the GUI's release CLI, as for #56, FIX_bughunt_tk_g2 5.6; its `xprv` paragraph already says "Use when the master is already in xprv form"), transcripts and `.examples-build/Examples.md` (no `derive-child` run with an xprv there; the only mention is the command list line 183). CHANGELOG and version: in the release commit after merge, as for G2.

## 8. Gate

`bash /scratch/code/shibboleth/refugium-coordinator/coordinator/scripts/build-gate.sh /scratch/cache/wt-tk-derive-child-depth origin/master --rows 'test(/^r[1-4]_non_master_/)' --rows-expect 4`

It runs the `[mnemonic-toolkit]` conf (fmt, clippy `-D warnings`, `cargo nextest run --locked --workspace`, doc tests) and the mutation gate on the diff. Not covered by it: the `manual` CI job (`make -C docs/manual audit`, with `verify-examples` and the doc-flag-lint mutation self-test) and `manual-gui`; run both, locally or as green CI on the head. Class B (derives secrets).

## 9. FOLLOWUPS

Closes nothing already in `design/FOLLOWUPS.md` (no line for bh-tk-I4). The line the builder files is listed in the author's report, for a direct push after the PR merges (no FOLLOWUPS edit in the PR).
