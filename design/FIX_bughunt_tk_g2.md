# FIX: the tk bug-hunt Importants of slices S3, S4 and S5 (grading pass G2) — import-wallet (#16 #17 #18 #19 #29 #31 #49 #57), export-wallet Sparrow labels (#20), convert (#26), compare-cost (#28), silent-payment (#56), word-card (#50), inspect (#51), repair (X1 X3 X4)

Small-fix track (RULES.md "Process"). Branch `bughunt/triage` at `2814a2c0`; it already carries the RED rows (`a9f781b2` and before). Between `a9f781b2` and `2814a2c0` only agent reports changed (`git diff --stat a9f781b2 HEAD -- crates docs .examples-build design/SPEC_* design/FOLLOWUPS.md` is empty). The build applies `FIX_bughunt_tk_g1.md` first, then this document (controller decision).

Sources, all in `design/agent-reports/`:

- `bughunt-2026-10-31-tk.md` (hunt report)
- `bughunt-triage-tk-s3.md`, `bughunt-triage-tk-s4.md`, `bughunt-triage-tk-s5.md` (triage)
- `bughunt-grade-tk-g2.md` (grades and product rulings, `2814a2c0`). This document applies those rulings, with the additions listed in section 11.

Scope (settled by the controller): the 17 Importants of the G2 report, with the fix as ruled there, not the triage patch where the report amended or replaced it. #30 (Minor) is closed by #31's fix; the patch is the same. Every other Minor and Nit is deferred (9.2).

**How this document was checked.**

- The code and test edits were applied in a scratch worktree, `/scratch/code/shibboleth/tk-worktrees/grade-g2-scratch` (detached at `a9f781b2`). They are saved as a reference patch: `/scratch/cache/bughunt/tk/g2/fix-bughunt-tk-g2-reference.patch` (sha256 `f8d51a2d3d66807dc9bc59978ccf45460c3dde175ba6e36b491e3801e89ba5c5`, 22 files, `git apply --check` passes at `2814a2c0`).
- The spec, manual, technical-manual, transcript and examples-corpus edits are a script, `/scratch/cache/bughunt/tk/g2/fix-g2-doc-edits.py` (sha256 `315c6b9c0aaba1868e16be7be9df96d0a1c0e4623975b966314c13142efdcb5e`). It asserts that each old text occurs exactly once, and writes nothing otherwise.
- Every command this document prints is in `/scratch/cache/bughunt/tk/g2/doc-commands.sh` (sha256 `c35dda60c0fc65cd7aa2f012978740c31739bfeeb7684cd7d0ac82d866bde05e`). It was run verbatim on the current binary (`target/debug/mnemonic` built at `a9f781b2`) and on the reference build. The captures are `doc-current.txt` (sha256 `69def3b2…289f`) and `doc-after.txt` (sha256 `feadebd4…46d9`). "exit" lists each pipeline stage's status, left to right.
- Fixtures: `/scratch/cache/bughunt/tk/g2/fixtures/` (sha256 list in `SHA256SUMS` there). The BSMS records were signed with key `0x42`×32 over the first four lines as written (`mk_round1.py`, python `ecdsa`), as RED row #18 does in Rust. `shares-2of3.txt` is `mnemonic ms-shares split --from entropy=000102030405060708090a0b0c0d0e0f --threshold 2 --shares 3`.

All runs used `PATH=$HOME/.cargo/bin:$PATH` (cargo 1.85.0, the `rust-toolchain.toml` pin), `--locked`, and no `--release`.

| check (scratch worktree with all edits) | result |
|---|---|
| `cargo nextest run --locked --workspace --no-fail-fast` | 4134 run, 4106 passed, 28 failed, 36 skipped; the 28 are exactly G1's rows (`bughunt_tk_01`–`14`, `37`–`47`, `52`–`54`), which this document does not fix |
| `cargo clippy --locked --all-targets --workspace -- -D warnings` | exit 0 |
| `cargo fmt --all --check` (toolchain 1.85.0) | exit 0 |
| `cargo test --locked --doc --workspace` | exit 0 |
| `make -C docs/manual verify-examples` (reference binary; `md`/`ms`/`mk` from `~/.cargo/bin`) | 8 FAIL, the same 8 as on the unmodified tree with the unmodified binary (`44-mk-*` and `qs-26-recover-mk1`: the locally installed `mk` is newer than the pin). Without section 5's transcript edits the reference binary adds 2 more: `37-wallet-export-sparrow` and `foreign-formats/roundtrip-sparrow-singlesig` |
| `make -C docs/manual lint` | the same 10 `[lint] FAIL` lines as the unmodified tree (all `44-mk-cli.md` stale markers, same local-`mk` cause); markdownlint 0 errors, cspell 0 issues |
| `make -C docs/manual anchor-check` | OK, 9 danglers, matching the baseline |
| `bash .examples-build/gen.sh` (reference binary first on PATH) vs committed `Examples.md` | 1 line differs (#28, section 5.5); the unmodified binary regenerates it byte-identical |
| `make -C docs/manual-gui verify-examples` | same 6 FAIL on both binaries (pre-existing); no G2 change |

The other direction: a second worktree, `/scratch/code/shibboleth/tk-worktrees/grade-g2-red`, held only the new test files and goldens over unmodified production code. There `-E 'binary(bughunt_s3) | binary(bughunt_s4) | binary(bughunt_s5) | test(/bughunt_tk_36/) | binary(cli_network_fail_open) | binary(cli_export_wallet_sparrow)'` gave 48 run, 25 passed, 23 failed. The 23 are exactly the 19 G2 rows this build turns GREEN, plus the four Sparrow golden cells of section 7. The 25 passes are `bughunt_tk_19b` (a guard row, below), `bughunt_tk_x5` (a FALSE POSITIVE kept as a pin), and the other existing tests of those binaries. On the reference build, `--run-ignored ignored-only` over the same filter gave 16 run, 16 failed: the deferred rows still record their defects.

## 1. Summary

1. **#16 / #17 — a taproot multisig imported from Bitcoin Core or Sparrow lost its threshold.** Text showed `threshold=none` and `--json` showed `bundle.multisig.threshold` 3 for a 2-of-3. Both now read K from `multi_a` / `sortedmulti_a`.
2. **#18 — BSMS Round-1 verification rebuilt line 3 instead of reading it as written.** As a result a record from Coldcard's signer (uppercase fingerprint) or one using `h` hardened markers failed as forged. And a record whose fingerprint case was edited after signing verified. The signed body is now the record's four lines as written.
3. **#19 — Coldcard's legacy BIP-45 P2SH multisig export (`Derivation: m/45'`) was refused.** It now imports when every cosigner is at `m/45'`, taking the network from the xpub version.
4. **#20 — Sparrow export gave every keystore the wallet name as its label.** Sparrow refuses that file for multisig (duplicate labels) and for any name over 16 characters. Every keystore is now `Keystore <n>`.
5. **#29 / #31 (closes #30) — Coldcard generic JSON took a non-integer `account` as 0, and an `xfp` with `+` signs as `0a0b0c0d`.** A multibyte `xfp` panicked. All three are now refused (exit 2).
6. **#49 — Coldcard generic JSON with only `bip48_*` blocks imported the depth-0 master xpub as a BIP-44 account xpub.** It is now refused (exit 2).
7. **#57 — a Bitcoin Core receive/change pair with different scripts was merged into one descriptor, dropping the change script.** It is no longer merged; the existing fixed-step refusal applies (exit 2).
8. **#26 — `convert phrase|seedqr|entropy → address --passphrase` warned that the passphrase was ignored, although it changes the address.** The warning is gone on that edge.
9. **#28 — compare-cost listed one repeated hash digest as several preimages and doubled its count.** One digest is now one preimage, in compare-cost and in the `build-descriptor` cap that must agree with it.
10. **#56 — `silent-payment` turned a non-master (depth ≠ 0) xprv into an sp1 address no wallet derives from the seed.** It is now refused (exit 1).
11. **#50 — `word-card --from` refused a multi-chunk card given as repeated `--from` values or as stdin lines, though its help says chunks are grouped by HRP.** They are now grouped.
12. **#51 — `inspect` decoded only the first of two ms1 cards and ignored the second.** A second ms1 is now refused (exit 1).
13. **X1 — `repair` accepted a mixed-case mk1 card as valid (exit 0) and named the ms1 case error obscurely.** Every kind is now refused with one text, also under `--max-indel`.
14. **X3 — `repair` refused every K-of-N ms1 share, damaged or clean.** Its advice, `ms combine`, fails on a damaged share. A share is now corrected (exit 4, with share-specific VERIFY-ME advice) or echoed (exit 0).
15. **X4 — `repair --max-indel` refused an all-capitals card (the engraved plate form) that it recovers in lowercase.** Capitals now recover exactly as lowercase does.

Nothing else changes for a holder. `--help` texts and the `gui-schema` output are unchanged (no flag, argument or doc comment changes), and so are all other exit codes. Versions are also unchanged in this PR: CHANGELOG and version go in a release commit after merge, as for the ms bug-hunt fixes.

## 2. Holder-visible behaviour, per fix

Fixture variables, as set at the top of `doc-commands.sh` (all public test material; the phrase and ms1 values are test vectors):

~~~sh
FX=/scratch/cache/bughunt/tk/g2/fixtures
XA=xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX
XB=xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6
XC=xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx
XL=xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj
PHRASE="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
ACCT_XPRV=xprv9ybY78BftS5UGANki6oSifuQEjkpyAC8ZmBvBNTshQnCBcxnefjHS7buPMkkqhcRzmoGZ5bokx7GuyDAiktd5HemohAU4wV1ZPMDRmLpBMm   # m/84'/0'/0' of PHRASE (depth 3)
H=5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9
MK0=mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
MK1=mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh
MD0=md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np
MD1=md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d
MD2=md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn
MS1=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
MS1_256=ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc83qygjzv2p29shrqv35xcur50p7q548au6h32276
SHARE=ms12mc8pqf9tqz74mwtgw8g3y86qux3mgf6cq8skgye8ey5fl9   # share 1 of a 2-of-3 split of 00 01 … 0f
~~~

Every `note: stdout is watch-only — public keys only, cannot spend` and `warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')` line is unchanged; they are shown only where they help. Every error line is the binary's `error: ` prefix plus the variant's Display; exit codes are `ToolkitError::exit_code` (`ImportWalletParse` 2, `BadInput` 1, `SilentPayment` 1, `Repair` 2).

### 2.1 #16 Bitcoin Core and #17 Sparrow: taproot multisig threshold

**Behaviour.** `wallet_import/bitcoin_core.rs::extract_threshold` and `wallet_import/sparrow.rs::extract_threshold_local` match `(?:thresh|multi|sortedmulti|multi_a|sortedmulti_a)\((\d+)\s*,`, the same pattern as `wallet_import/specter.rs:427`. For a `tr(NUMS, multi_a(K,…))` / `tr(NUMS, sortedmulti_a(K,…))` import, the text line `bundles[0].threshold=` and the JSON `bundle.multisig.threshold` show K. There is no new text; exit 0 as before. `core16.json` is a one-entry Core blob with `tr(<NUMS>,sortedmulti_a(2,[b8688df1/87'/0'/0']XA/<0;1>/*,[28645006/87'/0'/0']XB/<0;1>/*,[5436d724/87'/0'/0']XC/<0;1>/*))`.

Current:
~~~text
$ mnemonic import-wallet --blob $FX/core16.json --format bitcoin-core | grep "^bundles\[0\]\.threshold="
stdout: bundles[0].threshold=none
$ mnemonic import-wallet --blob $FX/core16.json --format bitcoin-core --json | jq ".[0].bundle.multisig.threshold"
stdout: 3
$ mnemonic import-wallet --blob $FX/sparrow-tr-multi-a-nums-2of3.json --format sparrow | grep "^bundles\[0\]\.threshold="
stdout: bundles[0].threshold=none
$ mnemonic import-wallet --blob $FX/sparrow-tr-multi-a-nums-2of3.json --format sparrow --json | jq ".[0].bundle.multisig.threshold"
stdout: 3
~~~

After the fix (same commands, all exit 0): `bundles[0].threshold=2`, `2`, `bundles[0].threshold=2`, `2`.

**Spec sentence:** none (no spec sentence states the extraction pattern). **Manual:** none.

### 2.2 #18 BSMS Round-1: the signature covers line 3 as written

**Behaviour.** `wallet_import/bsms_round1.rs::signed_body` joins lines 1–4 exactly as the record carries them (after the existing CRLF→LF normalisation), with no trailing newline. `key_field_to_line` and `format_path_no_m_prefix` are deleted; nothing else used them. The parsed `KeyField` is still used for the signer pubkey and the cross-checks. There is no new text. Results:

| record (`$FX/…`) | current | after the fix |
|---|---|---|
| `control.round1` (line 3 `[59865f44/48'/0'/0'/2']<pk>`) | verified, exit 0 | verified, exit 0 |
| `upper.round1` (`[59865F44/…]`, Coldcard's own case, signed as written) | `--bsms-verify-strict`: exit 2 `… pubkey mismatch — recovered 03b83b07… vs declared 0324653e…`; lenient: exit 4, `signature_verified:false` | verified, exit 0 (strict and lenient) |
| `hmark.round1` (`[59865f44/48h/0h/0h/2h]`) | strict: exit 2 `… recovered 022fab45… vs declared 0324653e…` | verified, exit 0 |
| `tampered.round1` (the control record with line 3 changed to `59865F44` after signing) | **verified, exit 0** | strict: exit 2; lenient: exit 4, `signature_verified:false` |

After the fix:
~~~text
$ mnemonic import-wallet --bsms-round1 $FX/upper.round1 --bsms-verify-strict --json
stdout: {"bsms_round1_verifications":[{"description":"Signer 1 key","failure_reason":null,"index":0,"signature_verified":true,"signer_pubkey":"0324653eac434488002cc06bbfb7f10fe18991e35f9fe4302dbea6d2353dc0ab1c","token_hex":"00"}],"source_format":"bsms-round1"}
exit (each pipeline stage, left to right): 0
$ mnemonic import-wallet --bsms-round1 $FX/tampered.round1 --bsms-verify-strict --json
stderr: error: import-wallet: --bsms-round1: BIP-129 signature verification failed for record 0 (signer pubkey 0324653eac434488002cc06bbfb7f10fe18991e35f9fe4302dbea6d2353dc0ab1c): ECDSA recovery succeeded but pubkey mismatch — recovered 035f34051e961c751df15ead4f4e73905d867ceb87a892e1cf26519f1afca62ee5 vs declared 0324653eac434488002cc06bbfb7f10fe18991e35f9fe4302dbea6d2353dc0ab1c
exit (each pipeline stage, left to right): 2
~~~

The BIP-129 published TV-1/TV-2 unit tests in `bsms_round1.rs` pass unchanged (their line 3 is already canonical).

**Spec sentence:** none (no spec states the rebuild). **Manual:** none. `docs/manual/src/30-workflows/3A-bsms-round1-verify.md:169-176` already says the signed body is `"<line 1>\n<line 2>\n<line 3>\n<line 4>"`; the fix makes the code match it.

### 2.3 #19 Coldcard multisig: legacy BIP-45 P2SH export

**Behaviour.** In `wallet_import/coldcard_multisig.rs::parse_text`, when **every** cosigner's derivation is exactly `m/45'` (one hardened component, index 45):

- the network comes from cosigner 0's xpub version: `xpub` → mainnet, `tpub` → testnet;
- the coin-type heterogeneity loop is skipped;
- the existing `assert_slots_network_agrees` check covers every xpub.

Otherwise behaviour is unchanged. In particular, a set that mixes `m/45'` with other paths keeps today's refusal. The descriptor is built as before, so it is `sh(sortedmulti(K,[xfp/45']xpub/<0;1>/*,…))`, the form Coldcard's own `descriptor.py:195-196` writes. `cc19.txt` is a 2-of-3 `Format: P2SH`, `Derivation: m/45'` file with `34A3A4F1: XA`, `FF9DFBCF: XB`, `B7F7DFEA: XC`. `cc19mixed.txt` is a 2-of-2 P2WSH file with one cosigner at `m/45'` and one at `m/48'/0'/0'/2'`.

Current (all three commands below print this stderr line; exit, each pipeline stage left to right: `2 1`, `2 0`, `2` — `mnemonic` exits 2 in all three):
~~~text
stderr: error: import-wallet: coldcard-multisig: parse error: derivation path has only 1 components; need ≥2 for BIP-48 coin-type inference
~~~

After the fix:
~~~text
$ mnemonic import-wallet --blob $FX/cc19.txt --format coldcard-multisig | grep -E "^bundles\[0\]\.(network|threshold)="
stdout: bundles[0].network=mainnet
stdout: bundles[0].threshold=2
exit (each pipeline stage, left to right): 0 0
$ mnemonic import-wallet --blob $FX/cc19.txt --format coldcard-multisig --json | jq -r ".[0].bundle.descriptor"
stdout: sh(sortedmulti(2,[34a3a4f1/45']XA/<0;1>/*,[ff9dfbcf/45']XB/<0;1>/*,[b7f7dfea/45']XC/<0;1>/*))#7a3ms3qj
stderr: notice: import-wallet: cosigner BIP-43 purpose "45'" is non-canonical for multisig; envelope `multisig.path_family` defaults to "bip87"
exit (each pipeline stage, left to right): 0 0
$ mnemonic import-wallet --blob $FX/cc19mixed.txt --format coldcard-multisig
stderr: error: import-wallet: coldcard-multisig: parse error: derivation path has only 1 components; need ≥2 for BIP-48 coin-type inference
exit (each pipeline stage, left to right): 2
~~~

(`XA`/`XB`/`XC` stand for the xpubs in full.) The `path_family` notice and its `"bip87"` value are existing, designed behaviour of `import_wallet.rs::path_family_from_paths` (lines 2134-2140); this fix does not change them (see 10, overlap O7).

**Spec sentence:** `design/SPEC_wallet_import_v0_28_0.md` §11.4 parse contract (section 4, S8). **Manual:** `45-foreign-formats.md` (section 5, B3).

### 2.4 #20 Sparrow export: keystore labels

**Behaviour.** `wallet_export/sparrow.rs::emit_sparrow_wallet_json` labels keystore `i` (0-based slot order) `Keystore <i+1>`, single-sig included. This is Sparrow's own default (`Keystore.DEFAULT_LABEL = "Keystore 1"`). It is always unique, as `Wallet.checkWallet()` requires ("Wallet keystores have duplicate labels"), and within drongo's `MAX_LABEL_LENGTH` of 16 non-space characters (`Keystore.checkKeystore()` "Label too long"). The top-level `name` stays the wallet name. Single-sig changes too because the 16-character cap applies there: a single-sig export named, e.g., `Household savings 2024` (20 non-space characters) was refused by Sparrow before.

Current:
~~~text
$ mnemonic export-wallet --format sparrow --template wsh-sortedmulti --threshold 2 --multisig-path-family bip48 --network mainnet --slot "@0.xpub=$XA" --slot @0.fingerprint=b8688df1 --slot "@0.path=m/48'/0'/0'/2'" --slot "@1.xpub=$XB" --slot @1.fingerprint=28645006 --slot "@1.path=m/48'/0'/0'/2'" --slot "@2.xpub=$XC" --slot @2.fingerprint=5436d724 --slot "@2.path=m/48'/0'/0'/2'" --output - | grep -E '"(name|label)"'
stdout:   "name": "wsh-sortedmulti-0",
stdout:     "name": "Default",
stdout:       "label": "wsh-sortedmulti-0",
stdout:       "label": "wsh-sortedmulti-0",
stdout:       "label": "wsh-sortedmulti-0",
exit (each pipeline stage, left to right): 0 0
~~~

After the fix: the same two `name` lines, then `"label": "Keystore 1",`, `"label": "Keystore 2",`, `"label": "Keystore 3",`; exit 0 0.

`import-wallet` is unaffected: its round-trip does not use the export emitter. The `--json` envelopes of `sparrow-singlesig-p2wpkh.json` and `sparrow-multisig-2of3-p2wsh-sortedmulti.json` are byte-identical before and after (`cmp`). `export-wallet --from-import-json … --format sparrow` now writes `Keystore <n>` where it wrote the lifted name, so the manual's round-trip recipe shows one diff line per keystore (section 5, B2 and 5.4).

**Spec sentences:** `design/SPEC_export_wallet_v0_8.md` §7 (S1, S2). **Manual:** `37-wallet-export.md` (C1), `45-foreign-formats.md` (B2), technical manual (D1, D2), two transcripts (5.4).

### 2.5 #29 / #31 (closes #30) Coldcard generic JSON: `account` and `xfp`

**Behaviour.** In `wallet_import/coldcard.rs`:

- A present `account` that is not a JSON non-negative integer is refused (`ImportWalletParse`, exit 2). That covers a string (even `"5"`), a negative or fractional number, `null` and a boolean, wherever the key appears, including when a bipNN block makes it unused. The existing over-u32 refusal stays.
- `parse_xfp_hex` refuses any non-hex-digit character before decoding (exit 2). This covers `+` (which `u8::from_str_radix` accepts) and multibyte characters (the panic of #30).

Texts (`<v>` is serde_json's rendering of the value):

~~~text
error: import-wallet: coldcard: parse error: `account` must be a non-negative integer, got <v>
error: import-wallet: coldcard: parse error: `xfp` must contain only hex digits
~~~

Current:
~~~text
$ printf '%s' '{"xpub":"'$XL'","xfp":"5436D724","chain":"BTC","account":"5"}' | mnemonic import-wallet --blob - --format coldcard --json | jq -r '.[0].bundle.descriptor'
stdout: pkh([5436d724/44'/0'/0']xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj/<0;1>/*)#7j0lynnu
exit (each pipeline stage, left to right): 0 0 0
~~~
(`-1`, `1.5` and `null` print the same line.)
~~~text
$ printf '%s' '{"xpub":"'$XL'","xfp":"+a+b+c+d","chain":"BTC"}' | mnemonic import-wallet --blob - --format coldcard --json | jq -r '.[0].bundle.descriptor'
stdout: pkh([0a0b0c0d/44'/0'/0']xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj/<0;1>/*)#0yn6s5mf
exit (each pipeline stage, left to right): 0 0 0
$ printf '%s' '{"xpub":"'$XL'","xfp":"aéééb","chain":"BTC"}' | mnemonic import-wallet --blob - --format coldcard --json | jq -r '.[0].bundle.descriptor'
stderr: thread 'main' panicked at crates/mnemonic-toolkit/src/wallet_import/coldcard.rs:406:39:
stderr: byte index 2 is not a char boundary; it is inside 'é' (bytes 1..3) of `aéééb`
exit (each pipeline stage, left to right): 0 101 0
~~~

After the fix (stdout empty in every case):
~~~text
account "5"  → stderr: error: import-wallet: coldcard: parse error: `account` must be a non-negative integer, got "5"   exit 0 2 0
account -1   → … got -1      exit 0 2 0
account 1.5  → … got 1.5     exit 0 2 0
account null → … got null    exit 0 2 0
xfp +a+b+c+d → stderr: error: import-wallet: coldcard: parse error: `xfp` must contain only hex digits   exit 0 2 0
xfp aéééb    → (same text)   exit 0 2 0
~~~

Coldcard's own `ccxp-*.json` multisig xpub export writes `"account": "%d"` as a string (`multisig.py:1648`). That file is refused earlier, with ``missing or non-string top-level `chain` `` on both builds, so this refusal changes no real Coldcard file.

**Spec sentence:** `design/SPEC_wallet_import_v0_28_0.md` §11.3 parse contract (S6). **Manual:** `41-mnemonic.md` import-wallet Refusals (A1), `45-foreign-formats.md` (B5).

### 2.6 #49 Coldcard generic JSON: the master xpub

**Behaviour.** In `select_dominant_bip`'s legacy fallback (no bip86/bip84/bip49/bip44 block), a top-level `xpub` of depth 0 is refused (exit 2). A legacy depth-3 top-level xpub still imports, and when a bipNN block exists the top-level `xpub` is not read. `cc49.json` carries `"xpub": xpub661My…` (the depth-0 master of Coldcard's documented example), `account` 123 and one `bip48_2` block.

Current:
~~~text
$ mnemonic import-wallet --blob $FX/cc49.json --format coldcard --json | jq -r ".[0].bundle.descriptor"
stdout: pkh([0f056943/44'/0'/123']xpub661MyMwAqRbcGC9DmWbtbAmuUjpMYxw4BWE88NSDHB3jSjfUK7KtYJuKa52GbowD3DVLkgsxH9QwPnTx5mjdHykYFEncnmAsNsCTbWzBhA7/<0;1>/*)#jjrp0wr6
exit (each pipeline stage, left to right): 0 0
~~~

After the fix:
~~~text
stderr: error: import-wallet: coldcard: parse error: top-level `xpub` is a depth-0 master xpub, not an account xpub; no bip44/bip49/bip84/bip86 block to import
exit (each pipeline stage, left to right): 2 0
~~~

**Spec sentence:** `design/SPEC_wallet_import_v0_28_0.md` §11.3.1 item 5 (S7). **Manual:** `41-mnemonic.md` (A1), `45-foreign-formats.md` (B4).

### 2.7 #57 Bitcoin Core: a receive/change pair with different scripts

**Behaviour.** In `merge_receive_change_pairs`, a same-key pair merges only if the change body, with its step widened to `<recv;chg>`, is byte-identical to the merged receive body. Otherwise there is no merge, both entries stay, and the first one meets the existing fixed-step refusal. No new text. `core57.json` is `wsh(pk([b8688df1/84'/0'/0']XA/0/*))` (`internal: false`) + `wsh(pkh([b8688df1/84'/0'/0']XA/1/*))` (`internal: true`).

Current:
~~~text
$ mnemonic import-wallet --blob $FX/core57.json --format bitcoin-core --json | jq -r ".[0].bundle.descriptor"
stdout: wsh(pk([b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*))#vpcpnlmy
exit (each pipeline stage, left to right): 0 0
~~~

After the fix:
~~~text
stderr: error: import-wallet: bitcoin-core: parse error: descriptors[0]: @0: derivation steps after the placeholder are not representable in md1; the use-site path must be a multipath `/<a;b>/*` (or bare `/*`) as the final step — a fixed single step like `/0/*` is un-representable (found residue near `/0/*))`)
exit (each pipeline stage, left to right): 2 0
~~~

Ordinary same-script pairs still merge (the merge tests in `cli_import_wallet_bitcoin_core*.rs` pass unchanged).

**Spec sentence:** `design/SPEC_bitcoin_core_receive_change_pair_merge.md` §4.2 cond. 1 (S9). **Manual:** `41-mnemonic.md` (A1, A2), `45-foreign-formats.md` (B1).

### 2.8 #26 convert: the passphrase on the phrase → address edge

**Behaviour.** In `cmd/convert.rs`, `edge_uses_pbkdf2` adds `NodeType::Address` to its target set, so `phrase` / `seedqr` / `entropy` → `address` with `--passphrase` no longer prints `warning: --passphrase ignored on this edge (not a PBKDF2-bearing conversion)`. The warning text itself is unchanged, and still fires on edges that ignore the passphrase (e.g. `--to ms1`).

Current (stdout identical on both builds; only stderr changes):
~~~text
$ printf "%s" "$PHRASE" | PP=TREZOR mnemonic convert --from phrase=- --to address --path "m/84'/0'/0'/0/0" --script-type p2wpkh --passphrase @env:PP
stdout: address: bc1qv5rmq0kt9yz3pm36wvzct7p3x6mtgehjul0feu
stderr: warning: --passphrase ignored on this edge (not a PBKDF2-bearing conversion)
stderr: note: stdout is watch-only — public keys only, cannot spend
exit (each pipeline stage, left to right): 0 0
~~~

After the fix: the same stdout, stderr is the `note:` line alone, exit 0 0; `--json` likewise (stdout `{"schema_version":"1","from_node":"phrase","to":[{"node":"address","value":"bc1qv5rmq0kt9yz3pm36wvzct7p3x6mtgehjul0feu"}]}`). Without `--passphrase` the address is `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`: the passphrase is used.

**Spec sentence:** none. `design/SPEC_convert_v0_6.md:370` already says "`--passphrase` is meaningful (PBKDF2-bearing per §8)" for `phrase → address`; the fix makes the code match it. **Manual:** none.

### 2.9 #28 compare-cost and the build-descriptor cap: one digest, one preimage

**Behaviour.**

- `cost/enumerate.rs::walk_segv0_for_hash_leaves` keeps each (hash kind, digest) once (`HashAsset` derives `PartialEq`).
- `descriptor_builder/gate.rs::hash_and_timelock_counts` counts the same distinct (kind, digest) pairs, so the build-time cap and the compare-cost precheck still agree.
- The doc comments of `check_cap` and `hash_and_timelock_counts` say that digests are deduped on both sides.

Holder-visible effects:

- compare-cost lists one `preimage(h<i>)` per distinct digest, and its `--max-conditions` precheck counts distinct digests (text: unchanged format).
- `build-descriptor`'s over-envelope diagnostic counts distinct digests in its `2^(<k> keys + <h> hashes)` text (format unchanged).

Current:
~~~text
$ mnemonic compare-cost --miniscript "or_i(and_v(v:pk(A),sha256($H)),and_v(v:pk(B),sha256($H)))" --json | jq -c "[.conditions[].label]"
stdout: ["A + preimage(h0)","A + preimage(h1)","B + preimage(h0)","B + preimage(h1)"]
$ mnemonic compare-cost --miniscript "or_i(and_v(v:pk(A),sha256($H)),and_v(v:pk(B),sha256($H)))" --max-conditions 8 >/dev/null
stderr: error: compare-cost: spending conditions exceed --max-conditions cap (16 > 8); raise the cap or simplify the policy
exit (each pipeline stage, left to right): 3
~~~

After the fix:
~~~text
$ mnemonic compare-cost --miniscript "or_i(and_v(v:pk(A),sha256($H)),and_v(v:pk(B),sha256($H)))" | grep -E "^[AB] \+"
stdout: A + preimage(h0) |     69 |   113 |   +44 |       69 |     113 |    +44
stdout: B + preimage(h0) |     68 |   112 |   +44 |       68 |     112 |    +44
$ mnemonic compare-cost --miniscript "or_i(and_v(v:pk(A),sha256($H)),and_v(v:pk(B),sha256($H)))" --json | jq -c "[.conditions[].label]"
stdout: ["A + preimage(h0)","B + preimage(h0)"]
$ mnemonic compare-cost --miniscript "or_i(and_v(v:pk(A),sha256($H)),and_v(v:pk(B),sha256($H)))" --max-conditions 8 >/dev/null
exit (each pipeline stage, left to right): 0
$ mnemonic compare-cost --miniscript "or_i(and_v(v:pk(A),sha256($H)),and_v(v:pk(B),sha256($H)))" --max-conditions 7 >/dev/null
stderr: error: compare-cost: spending conditions exceed --max-conditions cap (8 > 7); raise the cap or simplify the policy
exit (each pipeline stage, left to right): 3
~~~

The examples corpus's degrading-vault `build-descriptor` run is still refused (2^12 × 9 > 4096). Its line changes from `(2^(11 keys + 2 hashes) × 9 timelock-states > cap 4096)` to `(2^(11 keys + 1 hashes) × 9 timelock-states > cap 4096)` (measured by regenerating `Examples.md`, section 5.5).

**Spec sentences:** `design/SPEC_descriptor_builder_engine.md` lines 60 and 62 (S3, S4); `design/SPEC_compare_cost_v0_26_0.md` line 159 (S5). **Manual:** `41-mnemonic.md` (A9, A10); `.examples-build/Examples.md` (5.5). Cross-repo: section 9.4.

### 2.10 #56 silent-payment: a non-master xprv is refused

**Behaviour.** In `cmd/silent_payment.rs::resolve_master_xpriv`, an xprv/tprv whose `depth != 0` is refused as the first check of that branch, before the passphrase warning and the network cross-check:

~~~text
error: silent-payment: the xprv/tprv is at depth <d>, not a master key (depth 0); supply the master xprv/tprv, or the seed phrase, ms1 or entropy it came from
~~~

Exit 1 (`SilentPayment`); stdout empty, text and `--json` alike. Because depth is checked first, a depth-3 tprv with a contradicting `--network` gets this refusal (exit 1) rather than the network mismatch (exit 2). A depth-0 xprv/tprv behaves as today (the network cross-check and the passphrase warning still fire).

Current:
~~~text
$ echo "$ACCT_XPRV" | mnemonic silent-payment --secret-stdin | grep -E "^ *address:"
stdout:   address:      sp1qqgpvz49p4y9gzdv2crscmg70tnmen64v00r5w3v09m0tq8u7meqdxq6742pq2v9hj42ypkxpct2kpmels98ua2nmv3zuw3l7jzak7ypvvvr8durc
exit (each pipeline stage, left to right): 0 0 0
~~~

After the fix:
~~~text
stderr: error: silent-payment: the xprv/tprv is at depth 3, not a master key (depth 0); supply the master xprv/tprv, or the seed phrase, ms1 or entropy it came from
exit (each pipeline stage, left to right): 0 1 1
~~~
(`--json | jq -r .address`: the same stderr line, exit 0 1 0, stdout empty.)

**Spec sentence:** none. `design/SPEC_network_fail_open_assert_agrees.md:110` already describes E3 as "a `tprv` master input"; the test vector change of section 7 brings the test in line with it. **Manual:** `41-mnemonic.md` `--secret` row (A3).

### 2.11 #50 word-card: repeated `--from` chunks are grouped

**Behaviour.** In `cmd/word_card.rs::run_encode_solo`:

- If every `--from` value (after `-` expansion) decodes as a whole card, nothing changes.
- Otherwise the single-chunk values are joined per HRP, in order of first appearance, and values already holding whitespace-joined chunks stay as they are.
- `--raid` is unchanged.

Consequence: several multi-chunk cards of one HRP given chunk by chunk are joined into one list and refused with the codec's own error; pass each such card joined. No new text.

Current:
~~~text
$ cmp <(mnemonic word-card --from "$MK0 $MK1") <(mnemonic word-card --from $MK0 --from $MK1) && echo same
stderr: error: mk1 chunked-header malformed: received 1 chunks, header declares total_chunks = 2
stderr: cmp: EOF on ‘/dev/fd/62’ which is empty
exit (each pipeline stage, left to right): 1
~~~
(The stdin form fails with the same mk1 text. The md1 `--json` form fails with `error: md1 chunk set incomplete: got 1, expected 3`.)

After the fix: all three print `same` (exit 0); repeated `--from`, stdin lines and the joined form give byte-identical output.

**Spec sentence:** none. **Manual:** none; the `--from` help (`word_card.rs:61-62`) and the manual's word-card `--from` row already state the behaviour.

### 2.12 #51 inspect: a second ms1 is refused

**Behaviour.** In `cmd/inspect.rs::decode_card`, an ms1 group with more than one value is refused before decoding:

~~~text
error: inspect: expected 1 ms1 card, got <n>; inspect each ms1 card in its own run
~~~

Exit 1 (`BadInput`); stdout empty, text and `--json`. The error is not codec-class, so auto-repair does not fire. One ms1 is unchanged.

Current:
~~~text
$ printf "%s\n%s\n" $MS1 ms10entrsqqqqqqqqqqqpqqqqqqqqqqqqqqqqcj9sxraq34v7f | mnemonic inspect --no-auto-repair --ms1 -
stdout: kind: ms1
stdout: tag: entr
stdout: payload_kind: Entr
stdout: byte_length: 16
stdout: bit_strength: 128
stdout: entropy_hex: <suppressed; pass --reveal-secret to print>
exit (each pipeline stage, left to right): 0 0
~~~
(The second, corrupt card is never read. With `$MS1_256` second, `--json` prints only the 128-bit card.)

After the fix (both commands):
~~~text
stderr: error: inspect: expected 1 ms1 card, got 2; inspect each ms1 card in its own run
exit (each pipeline stage, left to right): 0 1
~~~

**Spec sentence:** none. **Manual:** `41-mnemonic.md` inspect `--ms1` row (A4).

### 2.13 X1 repair: a mixed-case chunk is refused for every kind

**Behaviour.**

- `repair.rs::parse_chunk` refuses a chunk holding both ASCII lowercase and uppercase letters before any other check (`RepairError::UnparseableInput`).
- `recover_indel_card` (the `--max-indel` path) refuses a mixed-case chunk the same way, before searching.

The text is the same for ms1, mk1 and md1, and the same with or without `--max-indel` (exit 2):

~~~text
error: repair: chunk <N> parse failed before correction could run: string mixes upper and lower case (BIP-173 forbids mixed case)
~~~

All-lowercase and all-capitals chunks are unaffected (X4).

Current:
~~~text
$ printf "%s\n" ms10ENTRsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f | mnemonic repair --ms1 -
stderr: error: repair: chunk 0 parse failed before correction could run: InvalidCase(Lower, 'E')
exit (each pipeline stage, left to right): 0 2
$ mnemonic repair --mk1 mk1QPRSQ${MK0:8} --mk1 $MK1
stdout: mk1QPRSQhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
stdout: mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh
exit (each pipeline stage, left to right): 0
$ mnemonic repair --md1 md1FGD${MD0:6} --md1 $MD1 --md1 $MD2
stderr: error: repair: chunk 0 parse failed before correction could run: chunk 0: string mixes upper and lower case (BIP-173 forbids mixed case)
exit (each pipeline stage, left to right): 2
$ D=${MS1:0:20}${MS1:21}; printf "%s\n" "ms10ENTR${D:8}" | mnemonic repair --max-indel 1 --ms1 -
stderr: error: repair: chunk could not be recovered within --max-indel 1 (HRP 'ms'); the string may have more than 1 inserted/dropped characters, or a different error class
exit (each pipeline stage, left to right): 0 2
~~~
(`--json` on the mk1 form printed `"verdict":"blessed"`.)

After the fix, all five (including `--json` on mk1):
~~~text
stderr: error: repair: chunk 0 parse failed before correction could run: string mixes upper and lower case (BIP-173 forbids mixed case)
exit: 2 (last stage)
~~~
For md1 this drops md-codec's inner `chunk 0: ` prefix; the two `cli_hrp_case_insensitive.rs` md1 cells drive `inspect`, not `repair`, and are unchanged.

**Spec sentence:** none. **Manual:** `41-mnemonic.md` repair (A5, A7) and the stale inspect case line (A8).

### 2.14 X3 repair: K-of-N ms1 shares

**Behaviour.** In `repair.rs::repair_via_ms_codec`, `Err(IsShareNotSingleString)` from `ms_codec::decode_with_correction` routes to `repair_share_natively`. That function re-runs the same polymod and `bch_decode::decode_regular_errors` solve on the string's symbols, then:

- (a) refuses any position outside the data part (`TooManyErrors`, bound 8);
- (b) re-verifies the corrected codeword (`TooManyErrors` if the residue is not zero);
- (c) requires the corrected (or clean) string to decode as `Err(IsShareNotSingleString)` under `ms_codec::decode`. Any other error becomes `PostCorrectionDecodeFailed` with that error's Display; an `Ok` becomes `PostCorrectionDecodeFailed` with detail `corrected string decodes as a single ms1, not a K-of-N share` (unreachable, defensive).

When any corrected chunk is a share (threshold character not `0`), the VERIFY-ME reason is:

~~~text
repair: correction UNVERIFIED — a corrected K-of-N share cannot be self-verified; recombine it with the other shares (mnemonic ms-shares combine --share <SHARE> --share <SHARE> …) and confirm the recovered seed's address/xpub against a known-good copy before use; BIP-93 recommends confirming a corrected codex32 string
~~~

A corrected seed card keeps its existing reason. Exits: corrected share 4 (verdict `candidate`); clean share 0 (verdict `blessed`, echoed). No ms-codec change.

Current (all three, damaged and clean):
~~~text
stderr: error: repair: chunk 0 post-correction decode failed: this is one share of a K-of-N set (threshold '2', index 'q'); use `ms combine` to recombine K shares
exit (each pipeline stage, left to right): 0 2
~~~

After the fix:
~~~text
$ printf "%s\n" ${SHARE:0:21}x${SHARE:22} | mnemonic repair --ms1 -
stdout: # Repair report
stdout: #   ms1 chunk 0: 1 correction at position 18: 'x' -> '8'
stdout: ms12mc8pqf9tqz74mwtgw8g3y86qux3mgf6cq8skgye8ey5fl9
stderr: repair: correction UNVERIFIED — a corrected K-of-N share cannot be self-verified; recombine it with the other shares (mnemonic ms-shares combine --share <SHARE> --share <SHARE> …) and confirm the recovered seed's address/xpub against a known-good copy before use; BIP-93 recommends confirming a corrected codex32 string
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
exit (each pipeline stage, left to right): 0 4
$ printf "%s\n" ${SHARE:0:21}x${SHARE:22} | mnemonic repair --json --ms1 -
stdout: {"schema_version":"1","kind":"ms1","verdict":"candidate","corrected_chunks":["ms12mc8pqf9tqz74mwtgw8g3y86qux3mgf6cq8skgye8ey5fl9"],"repairs":[{"chunk_index":0,"original_chunk":"ms12mc8pqf9tqz74mwtgwxg3y86qux3mgf6cq8skgye8ey5fl9","corrected_chunk":"ms12mc8pqf9tqz74mwtgw8g3y86qux3mgf6cq8skgye8ey5fl9","corrected_positions":[{"position":18,"was":"x","now":"8"}]}]}
stderr: (the same two lines)
exit (each pipeline stage, left to right): 0 4
$ printf "%s\n" $SHARE | mnemonic repair --ms1 -
stdout: ms12mc8pqf9tqz74mwtgw8g3y86qux3mgf6cq8skgye8ey5fl9
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
exit (each pipeline stage, left to right): 0 0
~~~

The advice runs as printed. Repairing a damaged share of the `shares-2of3.txt` set and recombining it with a second share recovers the secret:

~~~text
$ A=$(sed -n 1p $FX/shares-2of3.txt); B=$(sed -n 2p $FX/shares-2of3.txt); C=$(printf "%s\n" ${A:0:21}x${A:22} | mnemonic repair --ms1 - 2>/dev/null | tail -1); printf "%s\n" "$C" | mnemonic ms-shares combine --share - --share "$B" --to entropy
stdout: 000102030405060708090a0b0c0d0e0f
stderr: warning: secret material on argv (--share) — pipe via --share - to avoid /proc/$PID/cmdline exposure
stderr: warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
stderr: note: verify the recovered wallet's expected derived address before trusting
exit (each pipeline stage, left to right): 0 0
~~~
(Current: `error: ms1 not enough shares: have 1, need 2`, exit 1, because the repair produced nothing.)

**Spec sentence:** none. **Manual:** `41-mnemonic.md` repair (A6).

### 2.15 X4 repair --max-indel: all-capitals cards

**Behaviour.** `recover_indel_card`, after X1's mixed-case refusal, lowercases every chunk before searching, as the non-indel path does. A recovered all-capitals card prints and exits exactly as the same card in lowercase. No new text.

Current vs after, for the entr-16 card with character 20 left out (`D=${MS1:0:20}${MS1:21}`):

~~~text
$ D=${MS1:0:20}${MS1:21}; printf "%s\n" "${D^^}" | mnemonic repair --max-indel 1 --ms1 -
current: stderr: error: repair: chunk could not be recovered within --max-indel 1 (HRP 'ms'); the string may have more than 1 inserted/dropped characters, or a different error class
         exit (each pipeline stage, left to right): 0 2
after:   stdout: ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
         exit (each pipeline stage, left to right): 0 5
$ D=${MS1:0:20}${MS1:21}; printf "%s\n" "$D" | mnemonic repair --max-indel 1 --ms1 -
both:    stdout: ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
         exit (each pipeline stage, left to right): 0 5
$ mnemonic repair --max-indel 1 --mk1 ${MK0^^} --mk1 $(echo ${MK1:0:30}${MK1:31} | tr a-z A-Z)
current: exit 2, the same "could not be recovered … (HRP 'mk')" text
after:   stdout: mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh   exit 5
$ mnemonic repair --max-indel 1 --md1 ${MD0^^} --md1 $(echo ${MD1:0:30}${MD1:31} | tr a-z A-Z) --md1 ${MD2^^}
current: exit 2, the same text (HRP 'md')
after:   stdout: md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d   exit 5
~~~

`--json` is identical to the lowercase run: `{"schema_version":"1","status":"unique","confident":true,"candidates":[{"recovered":"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f","indel_count":1,"region":"data-part","direction":"inserted","subst_count":0}]}` for the ms1 card. This closes `toolkit-repair-indel-search-rejects-capitals` (9.5). The ms primer's "retype in lowercase" (mnemonic-secret FIX 5.6.6) stays valid; no ms change is needed.

**Spec sentence:** none. **Manual:** `41-mnemonic.md` repair Case paragraph (A5).

## 3. Code delta (production)

All in `crates/mnemonic-toolkit/src`; the reference patch is the verified text.

1. `wallet_import/bitcoin_core.rs` — #16: the `extract_threshold` regex of 2.1; #57: the change-body identity check of 2.7 in `merge_receive_change_pairs` (fix-57 text).
2. `wallet_import/sparrow.rs` — #17: the `extract_threshold_local` regex of 2.1.
3. `wallet_import/bsms_round1.rs` — #18: `signed_body` from the record's lines (fix-18, which stores the raw line 3 on `BsmsRound1Record`); delete `key_field_to_line` and `format_path_no_m_prefix` with their doc comments.
4. `wallet_import/coldcard_multisig.rs` — #19: `is_bip45_root` and `let bip45 = resolved.iter().all(|c| is_bip45_root(&c.path));` with the network/heterogeneity branch of 2.3, plus its comment.
5. `wallet_export/sparrow.rs` — #20: `SparrowKeystore.label: String` (no lifetime), `label: format!("Keystore {}", i + 1)` with the comment of the reference patch.
6. `wallet_import/coldcard.rs` — #29: the `account` type check before `raw_account` (fix-29); #31/#30: the hex-digit check in `parse_xfp_hex` (fix-30); #49: the depth-0 refusal in `select_dominant_bip`'s legacy fallback (fix-49).
7. `cmd/convert.rs` — #26: `NodeType::Address` in `edge_uses_pbkdf2` (fix-26).
8. `cost/enumerate.rs` — #28: `HashAsset: PartialEq`, dedupe in `walk_segv0_for_hash_leaves` (fix-28).
9. `descriptor_builder/gate.rs` — #28: `hash_and_timelock_counts` counts distinct `(kind, digest)` pairs (a `Vec<(u8, String)>` with a `contains` check, digest as `to_string()`); the doc comments of `check_cap` and `hash_and_timelock_counts`; the unit test of section 7.
10. `cmd/silent_payment.rs` — #56: the depth check and text of 2.10 (fix-56's position; the text without the doubled `silent-payment:` prefix).
11. `cmd/word_card.rs` — #50: `group_chunks_by_hrp` and its use in `run_encode_solo` (fix-50).
12. `cmd/inspect.rs` — #51: the guard and text of 2.12.
13. `repair.rs` — X1: the mixed-case refusal at the top of `parse_chunk` (fix-x1) and in `recover_indel_card`; X3: `repair_share_natively` with (a)–(c), `check_is_share`, `ms1_is_share`, the `IsShareNotSingleString` arm in `repair_via_ms_codec`, and the share reason in the Ms1 arm of `repair_card`; X4: lowercase every chunk in `recover_indel_card` after the X1 check.

Then `cargo fmt`. The raw triage patches are not fmt-clean.

## 4. Spec delta (`design/`)

Applied by `fix-g2-doc-edits.py` (tags S1–S9); each old text occurs once.

- **S1** `SPEC_export_wallet_v0_8.md` §7 example, the keystore `"label": "<wallet-name>",` → `"label": "Keystore 1",`.
- **S2** same file, after the bullet `` - `keystores`: 1 element for `SINGLE`, K elements for `MULTI` (one per cosigner, slot-index order). `` insert:
  `` - `keystores[i].label`: `Keystore <i+1>`, Sparrow's own default label (FIX_bughunt_tk_g2 #20). Always unique, which `Wallet.checkWallet()` requires ("Wallet keystores have duplicate labels"), and within drongo's `Keystore.MAX_LABEL_LENGTH` of 16 non-space characters; `--wallet-name` fills the top-level `name` only. ``
- **S3** `SPEC_descriptor_builder_engine.md:60`: `(distinct-key + hash-LEAF) count` → `(distinct-key + distinct-hash) count`.
- **S4** same file, line 62. Replace the bullet beginning `- **Hash digests are counted as LEAVES, NOT deduped**` with:
  `` - **Hash digests are deduped on both sides** (FIX_bughunt_tk_g2 #28, superseding the Phase-2 r2 I1 leaf count): one digest is one preimage, so `enumerate`'s hash walk keeps each (hash kind, digest) once and the pre-render cap counts the same distinct pairs (`gate.rs::hash_and_timelock_counts`). The `2^(keys+hashes)` exponent uses deduped keys and deduped digests; pinned by `repeated_digest_cap_agrees_with_enumerate` (one digest in two leaves: raw 4, refused at cap 3 on both sides). ``
- **S5** `SPEC_compare_cost_v0_26_0.md:159`: `` - `preimages`: every `sha256(H)`, `hash256(H)`, `ripemd160(H)`, `hash160(H)` — these are preimage requirements. `` →
  `` - `preimages`: every distinct `sha256(H)`, `hash256(H)`, `ripemd160(H)`, `hash160(H)` — these are preimage requirements; a digest repeated in several leaves is one preimage (FIX_bughunt_tk_g2 #28). ``
- **S6** `SPEC_wallet_import_v0_28_0.md` §11.3 parse contract. After `` Extract `xfp` → master fingerprint `` the sentence becomes:
  `` Extract `xfp` → master fingerprint; `xfp` must be exactly 8 hex digits, and any other character is refused (`ImportWalletParse`, exit 2). A present `account` must be a non-negative JSON integer that fits in a u32, wherever it appears; anything else (a string, a negative or fractional number, `null`, a boolean) is refused (`ImportWalletParse`, exit 2), never read as 0 (FIX_bughunt_tk_g2 #29, #31). ``
  The following sentence, `Dominant-BIP selection per heuristic at SPEC §11.3.1 below.`, is kept.
- **S7** same file, §11.3.1 item 5, append:
  `` A depth-0 top-level `xpub` is the master xpub current firmware writes beside its per-BIP blocks, never an account xpub: it is refused (`ImportWalletParse`, exit 2) (FIX_bughunt_tk_g2 #49). ``
- **S8** same file, §11.4 parse contract, append:
  `` When EVERY cosigner's derivation is exactly `m/45'` (BIP-45, Coldcard's legacy P2SH path, which has no coin-type component), the network comes from the xpub version (xpub → mainnet, tpub → testnet) instead of the BIP-48 coin-type; a set that mixes `m/45'` with other paths keeps the coin-type refusal (FIX_bughunt_tk_g2 #19). ``
- **S9** `SPEC_bitcoin_core_receive_change_pair_merge.md` §4.2 cond. 1: `` 1. **Script/threshold identical** — implied by identical grouping key (same template kind + threshold). `` →
  `` 1. **Script/threshold identical** — identical grouping key (same template kind + threshold) AND identical script bodies: with each side's final step replaced by the merged `<stepA;stepB>`, the two bodies must be byte-identical (`wsh(pk(…))` and `wsh(pkh(…))` share a template kind but are different scripts; FIX_bughunt_tk_g2 #57). ``

No other spec changes: at most three sentences in any one spec, and no flag, type or command is added.

## 5. Manual delta (`mnemonic-toolkit` docs, this PR)

Applied by `fix-g2-doc-edits.py`. The old text is quoted exactly; tag numbers match the script.

### 5.1 `docs/manual/src/40-cli-reference/41-mnemonic.md`

- **A1** (#29, #31, #49, #57) import-wallet `### Refusals`. After the row `` | Bitcoin Core receive/change-shaped pair whose keys/origins differ | exit 2 `ImportWalletParse` — not merged (distinct keys are different wallets); see [foreign wallet formats](#foreign-wallet-formats) | `` insert four rows:
  ~~~markdown
  | Bitcoin Core same-key receive/change pair whose script bodies differ (e.g. `wsh(pk(…/0/*))` + `wsh(pkh(…/1/*))`) | exit 2 `ImportWalletParse` — not merged; the first entry gets the fixed-step refusal above |
  | Coldcard generic JSON: `account` present but not a non-negative JSON integer (`"5"`, `-1`, `1.5`, `null`) | exit 2 `ImportWalletParse` — ``import-wallet: coldcard: parse error: `account` must be a non-negative integer, got <value>`` |
  | Coldcard generic JSON: `xfp` holds a character that is not a hex digit | exit 2 `ImportWalletParse` — ``import-wallet: coldcard: parse error: `xfp` must contain only hex digits`` |
  | Coldcard generic JSON: no `bip44`/`bip49`/`bip84`/`bip86` block and a depth-0 (master) top-level `xpub` | exit 2 `ImportWalletParse` — ``import-wallet: coldcard: parse error: top-level `xpub` is a depth-0 master xpub, not an account xpub; no bip44/bip49/bip84/bip86 block to import`` |
  ~~~
- **A2** (#57) Bitcoin Core worked example. Old: `` distinct keys are different wallets. Stderr is silent under `--json` `` New (four lines):
  ~~~markdown
  distinct keys are different wallets. A same-key pair whose script
  bodies differ (e.g. `wsh(pk(…/0/*))` + `wsh(pkh(…/1/*))`) is not
  merged either; its first entry gets the fixed-step refusal (exit 2).
  Stderr is silent under `--json`
  ~~~
- **A3** (#56) silent-payment `--secret` row. After ``A single private key (WIF/minikey) is refused — it cannot derive `m/352'`.`` insert ` A non-master xprv/tprv (depth ≠ 0, e.g. an account-level key) is refused (exit 1).`
- **A4** (#51) inspect `--ms1` row. After `(one HRP per card; per D35)` (the inspect row only) insert `; a second ms1 (positional or stdin line) is refused (exit 1)`.
- **A5** (X1, X4) repair, immediately before `### mk1 set-level re-verify (Cycle E funds fix) {#mnemonic-repair-mk1-set-level-reverify}`, insert:
  ~~~markdown
  **Case.** A chunk typed all in lower case or all in capitals (the
  BIP-93 handwritten and QR form) is accepted, with or without
  `--max-indel`; a corrected or recovered chunk is printed in lower
  case, and a chunk that needed no correction is echoed as typed. A
  chunk that mixes upper and lower case is refused (exit 2) for `ms1`,
  `mk1` and `md1` alike, also under `--max-indel` (BIP-173, BIP-93):
  `repair: chunk <N> parse failed before correction could run: string
  mixes upper and lower case (BIP-173 forbids mixed case)`.
  ~~~
  ("Echoed as typed" was measured: an uncorrected all-capitals mk1 chunk prints in capitals next to a corrected chunk printed in lowercase.)
- **A6** (X3) repair, immediately before `### md1 non-chunked demotion (v0.86.0 funds fix) {#mnemonic-repair-md1-non-chunked-demotion}`, insert:
  ~~~markdown
  **K-of-N shares.** A K-of-N `ms1` share (threshold `2`–`9`) is
  corrected like any `ms1`: a corrected share exits `4` (VERIFY-ME),
  and stderr says `repair: correction UNVERIFIED — a corrected K-of-N
  share cannot be self-verified; recombine it with the other shares
  (mnemonic ms-shares combine --share <SHARE> --share <SHARE> …) and
  confirm the recovered seed's address/xpub against a known-good copy
  before use; BIP-93 recommends confirming a corrected codex32
  string`. A clean share exits `0`, echoed.
  ~~~
- **A7** (X1) repair `### Refusals`. After `` | No chunks supplied | `repair: no chunks supplied` | `` insert:
  `` | `chunk_index N` mixes upper and lower case (any kind; also under `--max-indel`) | `repair: chunk N parse failed before correction could run: string mixes upper and lower case (BIP-173 forbids mixed case)` | ``
- **A8** (X1; stale since md-codec 0.35.3) inspect Advisories: `` the authority on case — `mk1`/`ms1` reject mixed-case input, `md1` is lenient. `` → `` the authority on case — `mk1`, `ms1` and `md1` reject mixed-case input. ``
- **A9** (#28) compare-cost: `` - **Preimages** — `preimage(h<i>)` in AST-order, one per `sha256` / `` / `` `hash256` / `ripemd160` / `hash160` leaf supplied. `` →
  ~~~markdown
  - **Preimages** — `preimage(h<i>)` in AST-order, one per distinct hash
    digest (a digest repeated in several `sha256` / `hash256` /
    `ripemd160` / `hash160` leaves is one preimage).
  ~~~
- **A10** (#28) build-descriptor step 4: `` `2^(keys+hashes) × timelock-states` exceeds the always-previewable `` → `` `2^(keys+hashes) × timelock-states` (each distinct key and each `` / `distinct hash digest counted once) exceeds the always-previewable`.

### 5.2 `docs/manual/src/45-foreign-formats.md`

- **B1** (#57) "Receive/change split exports". Before `- A lone fixed-step entry with no receive/change partner, a hardened` insert:
  ~~~markdown
  - A same-key pair whose scripts differ apart from the final step
    (e.g. `wsh(pk(A/0/*))` + `wsh(pkh(A/1/*))`) is not merged; its
    entries fall through to the use-site residue reject (exit 2).
  ~~~
- **B2** (#20) Sparrow "Round-trip example". Old:
  ~~~markdown
  > **Note on the diff.** As written above, the `diff` is **empty** —
  > the recipe round-trips the wallet name verbatim through the envelope
  > (the v0.37.8 universal source-name lift carries
  > `sparrow_source_metadata.label` back into the re-emitted `name` /
  > `label` fields). To override the lifted name on re-emit, pass
  ~~~
  New:
  ~~~markdown
  > **Note on the diff.** As written above, the `diff` shows only the
  > keystore `label` lines: `export-wallet --format sparrow` labels
  > every keystore with Sparrow's own default, `Keystore <n>` (`n`
  > counting from 1 in cosigner order), because Sparrow refuses
  > duplicate keystore labels and labels longer than 16 characters
  > (spaces not counted). The wallet name round-trips verbatim through
  > the envelope (the v0.37.8 universal source-name lift carries
  > `sparrow_source_metadata.label` back into the re-emitted `name`
  > field). To override the lifted name on re-emit, pass
  ~~~
  The rest of the note is unchanged.
- **B3** (#19) Coldcard multisig "CLI invocation". After `populates `mk1` with all three cosigner xpubs.` insert a blank line and:
  ~~~markdown
  A legacy BIP-45 P2SH export (`Derivation: m/45'`, Coldcard's P2SH
  path) has no coin-type component. When every cosigner is at `m/45'`,
  the network comes from the xpubs' version bytes (`xpub` → mainnet,
  `tpub` → testnet) and the descriptor is
  `sh(sortedmulti(K, [xfp/45']xpub…/<0;1>/*, …))`. A file that mixes
  `m/45'` with other derivation paths is refused (exit 2).
  ~~~
- **B4** (#49) Coldcard single-sig "Dominant-BIP selection". Old: `` inference (`zpub` → BIP-84, `ypub` → BIP-49, `xpub` → BIP-44). The `` New:
  ~~~markdown
  inference (`zpub` → BIP-84, `ypub` → BIP-49, `xpub` → BIP-44). A
  top-level `xpub` at depth 0 is the master xpub that current firmware
  writes beside its per-BIP blocks, not an account xpub: with no
  `bip44`/`bip49`/`bip84`/`bip86` block it is refused (exit 2). The
  ~~~
- **B5** (#29) Coldcard single-sig provenance table: `` | `raw_account` | dominant block | `account` integer (typically 0) | `` → `` | `raw_account` | dominant block | `account` integer (typically 0); a present `account` that is not a non-negative JSON integer (`"5"`, `-1`, `1.5`, `null`) is refused (exit 2) | ``

### 5.3 `docs/manual/src/30-workflows/37-wallet-export.md` and the technical manual

- **C1** (#20) `37-wallet-export.md`, before `Import into Sparrow via *File → Import Wallet → Sparrow → Import`, insert:
  ~~~markdown
  The wallet's name is `--wallet-name`; each keystore is labelled with
  Sparrow's own default, `Keystore 1` … `Keystore N` in slot order,
  because Sparrow refuses duplicate keystore labels and labels longer
  than 16 characters.

  ~~~
- **D1** (#20) `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md`, Sparrow "Vendor-specific flags" row: `` `--wallet-name <STRING>` (label); `` → `` `--wallet-name <STRING>` (wallet `name`; every keystore `label` is `Keystore <n>`, 1-based slot order); ``.
- **D2** (#20) same file, representative output: `      "label": "bip84-0",` → `      "label": "Keystore 1",` (the only keystore label in that example).

### 5.4 Transcripts (`docs/manual/transcripts/`, verified by `verify-examples`)

- `37-wallet-export-sparrow.out`: lines 14, 24 and 34, `"label": "VaultColdStorage",` → `"label": "Keystore 1",`, `"Keystore 2",`, `"Keystore 3",` (the top-level `"name": "VaultColdStorage"` is unchanged).
- `foreign-formats/roundtrip-sparrow-singlesig.out` (was one empty line) becomes exactly:
  ~~~text
  15c15
  <       "label": "bip84-0",
  ---
  >       "label": "Keystore 1",
  ~~~
  (`.err` unchanged; `verify-examples` ignores the `diff`'s exit 1.) The QuickStart's `transcripts/` is a symlink to this directory, so it follows.

### 5.5 Examples corpus (`.examples-build/Examples.md`, CI `examples.yml`)

The line holding `(2^(11 keys + 2 hashes) × 9 timelock-states > cap 4096)` (line 792 at `2814a2c0`) changes to `(2^(11 keys + 1 hashes) × 9 timelock-states > cap 4096)`. That is the whole diff of `bash .examples-build/gen.sh` on the reference build. The script's E1 edit makes it, and the file after E1 is byte-identical to the reference regeneration (`diff` empty, measured). After both FIX documents, the build regenerates the file with `gen.sh` anyway. The resulting diff against `2814a2c0` must be E1 plus whatever G1's fixes change (overlap O3).

### 5.6 Not changed in this PR

`docs/manual-gui/transcripts/tutorial/tut-j3-09-build-descriptor-refusal.stderr.txt` also carries `2 hashes`. It is **not** edited here. `docs/manual-gui/tests/lint.sh` step 11 (`byte_census_against_manifest`, lines 345-355) requires every file in that directory to be byte-identical to the pinned mnemonic-gui tag's `tests/snapshots/tutorial/`. `.github/workflows/manual-gui.yml` runs manual-gui's `verify-examples` against the GUI-pinned release CLIs, not this PR's binary (lines 163-180). So the file follows the next GUI pin (9.4). An edit here would turn manual-gui's lint red (overlap O4).

## 6. RED rows

Files: `tests/bughunt_s3.rs`, `tests/bughunt_s4.rs`, `tests/bughunt_s5.rs`, `src/friendly_bughunt_s4.rs`. "Today" was measured on the red worktree (new test files over unmodified code); "after" on the reference build. Shared helpers added: `bughunt_s5.rs` gains `last_line(&str) -> &str` (a stream's last line; rows that pass a secret on argv also get one argv-advisory line per value) and the constant `PK_ADVISORY` (the stdout-private-key advisory line quoted in section 2).

| row | after the build | exact assertion | today |
|---|---|---|---|
| `bughunt_tk_16_bitcoin_core_import_taproot_multisig_threshold` | GREEN, unchanged | JSON `bundle.multisig.threshold == 2` for the `sortedmulti_a` and the `multi_a` blob | `bundle.multisig.threshold = 3` (both) |
| `bughunt_tk_17_sparrow_import_taproot_multisig_threshold` | GREEN, unchanged | as 16, Sparrow fixture and its `sortedmulti_a` rewrite | `= 3` |
| `bughunt_tk_18_bsms_round1_signature_over_line3_as_written` | GREEN, unchanged | control, uppercase fingerprint, `h` markers and `[fp/]` each give exit 0 and `"signature_verified":true` under `--bsms-verify-strict --json` | uppercase and `h` fail ("pubkey mismatch") |
| `bughunt_tk_18b_bsms_round1_line3_edited_after_signing_refused` | NEW, GREEN | the control record (key `0x42`×32) with line 3's `[59865f44/` replaced by `[59865F44/` after signing: exit 2, stdout `""`, stderr starts `error: import-wallet: --bsms-round1: BIP-129 signature verification failed for record 0 (signer pubkey <pk>): ECDSA recovery succeeded but pubkey mismatch — recovered ` and ends ` vs declared <pk>\n` (`<pk>` = `0324653eac434488002cc06bbfb7f10fe18991e35f9fe4302dbea6d2353dc0ab1c`) | exit 0, verified |
| `bughunt_tk_19_coldcard_multisig_import_bip45_p2sh` | GREEN, tightened | text: exit 0, stdout contains `bundles[0].threshold=2\n` and `bundles[0].network=mainnet\n`; `--json`: exit 0, `[0].bundle.descriptor` starts with `sh(sortedmulti(2,[34a3a4f1/45']<XA>/<0;1>/*,[ff9dfbcf/45']<XB>/<0;1>/*,[b7f7dfea/45']<XC>/<0;1>/*))#` | exit 2 (coin-type text) |
| `bughunt_tk_19b_coldcard_multisig_bip45_mixed_paths_refused` | NEW, passes today and after | `cc19mixed` shape: exit 2, stderr == `error: import-wallet: coldcard-multisig: parse error: derivation path has only 1 components; need ≥2 for BIP-48 coin-type inference\n` | passes; **fails under the triage's fix-19** (cosigner-0-only test: exit 0, `bundles[0].threshold=2`; measured), so it pins the all-cosigners rule |
| `bughunt_tk_20_sparrow_export_keystore_labels_unique` | GREEN, tightened | labels == `["Keystore 1","Keystore 2","Keystore 3"]`; top-level `name == "wsh-sortedmulti-0"` | three `wsh-sortedmulti-0` |
| `bughunt_tk_21_bip388_export_from_descriptor_taproot_nums_multisig` | **rewritten**, `#[ignore = "deferred-minor: bip388-literal-nums-key-refusal-text"]` | control: the `--template tr-sortedmulti-a --taproot-internal-key nums --format descriptor` emit exits 0 and starts `tr(<NUMS>,sortedmulti_a(`; then `export-wallet --format bip388 --descriptor <it> --network mainnet --output -`: exit 1, stdout `""`, stderr == `error: --format bip388 cannot express the literal key 50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0 (a taproot internal key with no origin, e.g. the BIP-341 NUMS key): a BIP-388 wallet policy allows only @N key placeholders; use --format descriptor or --format bitcoin-core for this wallet\n` | run ignored: fails on the text only (exit 1 matches): `error: --format bip388 requires every descriptor key to end in `/<0;1>/*` for the receive/change pair; got key "50929b74…"` |
| `bughunt_tk_22_specter_export_descriptor_mode_device_count` | `#[ignore = "deferred-minor: specter-export-descriptor-mode-device-count"]`, body unchanged | — | fails run ignored |
| `bughunt_tk_23_electrum_import_wallet_name_digits_preserved` | `#[ignore = "deferred-minor: electrum-import-wallet-name-trailing-digits"]` | — | fails run ignored |
| `bughunt_tk_29_coldcard_import_non_integer_account_not_rewritten_to_zero` | GREEN, tightened | for `account` in `"5"`, `-1`, `1.5`, `null`: exit 2, stdout `""`, stderr == ``error: import-wallet: coldcard: parse error: `account` must be a non-negative integer, got <v>\n`` with `<v>` the JSON text as written | exit 0, `…/44'/0'/0']…` |
| `bughunt_tk_30_coldcard_import_multibyte_xfp_is_error_not_panic` | GREEN, tightened (closed by #31's fix) | exit 2, stdout `""`, stderr == ``error: import-wallet: coldcard: parse error: `xfp` must contain only hex digits\n`` | exit 101, panic |
| `bughunt_tk_31_coldcard_import_xfp_plus_signs_refused` | GREEN, tightened | as 30, for `+a+b+c+d` | exit 0, `0a0b0c0d` |
| `bughunt_tk_49_coldcard_import_master_xpub_not_used_as_account_xpub` | GREEN, tightened | exit 2, stdout `""`, stderr == ``error: import-wallet: coldcard: parse error: top-level `xpub` is a depth-0 master xpub, not an account xpub; no bip44/bip49/bip84/bip86 block to import\n`` | exit 0, master xpub in the descriptor |
| `bughunt_tk_57_bitcoin_core_merge_requires_same_script` | GREEN, tightened | exit 2, stdout `""`, stderr == the 2.7 after-fix error line + `\n` | exit 0, merged `<0;1>` |
| `bughunt_tk_58_bitcoin_core_error_names_original_descriptor_index` | `#[ignore = "deferred-minor: core-import-error-index-after-merge"]` | — | fails run ignored |
| `bughunt_tk_15_dice_huge_length_is_refused_cleanly` | `#[ignore = "deferred-minor: derive-child-dice-length-cap"]` | — | fails run ignored |
| `bughunt_tk_24_xpub_search_sh_wsh_default_origin_is_1prime` | `#[ignore = "deferred-minor: xpub-search-sh-wsh-default-origin-notice"]` | — | fails run ignored |
| `bughunt_tk_25_convert_minikey_env_sentinel_resolves` | `#[ignore = "deferred-minor: convert-minikey-env-sentinel"]` | — | fails run ignored |
| `bughunt_tk_26_convert_phrase_to_address_passphrase_not_reported_ignored` | GREEN, tightened | stdout without passphrase == `address: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu\n`; with `--passphrase TREZOR` == `address: bc1qv5rmq0kt9yz3pm36wvzct7p3x6mtgehjul0feu\n` and stderr has no `--passphrase ignored` | warning present |
| `bughunt_tk_27_compare_cost_tr_leaf_hash_arg_not_inflated` | `#[ignore = "deferred-minor: compare-cost-tr-leaf-hash-arg-inflated"]` | — | fails run ignored |
| `bughunt_tk_28_compare_cost_repeated_hash_counted_once` | GREEN, tightened | `--json` labels == `["A + preimage(h0)","B + preimage(h0)"]`; `--max-conditions 8` exit 0; `--max-conditions 7` exit 3 with stderr == `error: compare-cost: spending conditions exceed --max-conditions cap (8 > 7); raise the cap or simplify the policy\n` | 4 labels; cap 8 refused (`16 > 8`) |
| `bughunt_tk_32_path_of_xpub_min_account_2p31_no_panic` | `#[ignore = "deferred-minor: xpub-search-account-window-2p31-panic"]` | — | fails run ignored |
| `bughunt_tk_32_account_of_descriptor_zero_window_2p31_no_panic` | `#[ignore = "deferred-minor: xpub-search-account-window-2p31-panic"]` | — | fails run ignored |
| `bughunt_tk_33_gui_schema_bundle_import_json_does_not_require_template` | `#[ignore = "deferred-minor: gui-schema-bundle-import-json-template-required"]` | — | fails run ignored |
| `bughunt_tk_34_gui_schema_verify_bundle_alternatives_do_not_require_template` | `#[ignore = "deferred-minor: gui-schema-verify-bundle-template-required"]` | — | fails run ignored |
| `bughunt_tk_35_compare_cost_mixed_input_reports_concrete_keys` | `#[ignore = "deferred-minor: compare-cost-concrete-keys-mixed-input"]` | — | fails run ignored |
| `friendly::bughunt_s4::bughunt_tk_36_friendly_length_set_follows_error_allowed` | `#[ignore = "deferred-minor: friendly-ms1-length-set-uses-allowed"]` | — | fails run ignored |
| `bughunt_tk_48_verify_message_legacy_tolerates_trailing_newline` | `#[ignore = "deferred-minor: verify-message-legacy-signature-trim"]` | — | fails run ignored |
| `bughunt_tk_56_silent_payment_refuses_non_master_xprv` | GREEN, tightened | exit 1, stdout `""`, stderr == the 2.10 text with depth 3 + `\n` | exit 0, `sp1…` |
| `bughunt_tk_50_word_card_from_groups_chunks_of_one_card` | GREEN, unchanged | repeated `--from`, two stdin lines, and repeated md1 `--from` each give exit 0 and stdout byte-identical to the joined form | each exits 2 |
| `bughunt_tk_51_inspect_second_ms1_not_ignored` | GREEN, tightened | (a) corrupt second ms1 and (b) valid 256-bit second ms1: exit 1, stdout `""`, `last_line(stderr)` == `error: inspect: expected 1 ms1 card, got 2; inspect each ms1 card in its own run` | exit 0, first card only |
| `bughunt_tk_x1_repair_refuses_mixed_case_chunks` | GREEN, tightened and extended | mixed ms1, md1, mk1, and the mixed ms1 with character 20 left out under `--max-indel 1`: each exit 2, stdout `""`, `last_line(stderr)` == `error: repair: chunk 0 parse failed before correction could run: string mixes upper and lower case (BIP-173 forbids mixed case)` | mk1 exit 0; ms1 `InvalidCase(Lower, 'E')`; md1 inner `chunk 0: ` prefix; `--max-indel` case "could not be recovered" |
| `bughunt_tk_x2_repair_ms1_resolves_env_sentinel` | `#[ignore = "deferred-minor: toolkit-repair-inspect-ms1-env-sentinel"]` | — | fails run ignored |
| `bughunt_tk_x3_repair_corrects_one_substitution_in_kofn_share` | GREEN, tightened | damaged share: exit 4, stdout == `# Repair report\n#   ms1 chunk 0: 1 correction at position 18: 'x' -> '8'\n<SHARE>\n`, stderr == the share UNVERIFIED line + `\n` + `PK_ADVISORY` + `\n`; clean share: exit 0, stdout == `<SHARE>\n`, stderr == `PK_ADVISORY` + `\n` | both exit 2 ("use `ms combine`") |
| `bughunt_tk_x4_repair_max_indel_accepts_all_capitals` | GREEN, tightened | for ms1, mk1 (chunk 1, character 30 left out) and md1 (chunk 1, character 30 left out): the all-capitals run's `(exit, stdout)` == `(5, <the same run in lowercase>.stdout)`; each lowercase baseline exits 5 | capitals exit 2 |
| `bughunt_tk_x5_repair_ms1_stdin_second_line_not_ignored` | unchanged, passes | (FALSE POSITIVE, kept as a pin) | passes |

The header comments of rows 16-20, 29-31, 49, 57, 26, 28, 56, 51, X1, X3 and X4 that say "open product question" or describe accepting alternatives become the ruling; the reference patch has wording for #21, 18b and 19b, which is not binding.

Counts on the reference build: `-E 'binary(bughunt_s3) | binary(bughunt_s4) | binary(bughunt_s5) | test(/bughunt_tk_36/)'` gives every non-ignored row passing (21 G2 rows: the 19 of the table that turn GREEN, plus 19b and X5); `--run-ignored ignored-only` with the same filter: 16 run, 16 failed.

## 7. Tests that change

Outside the bughunt files, exactly these (measured: they are the only non-row tests the reference flips, and each passes in its new form):

1. `src/descriptor_builder/gate.rs::tests::repeated_digest_cap_agrees_with_enumerate` (#28). Same shape: one sha256 digest H in two `and_v(v:sha256(H),…)` leaves over `pk(A)`. New expectations: gate `validate_with_cap(&parsed, 4)` is Ok and `validate_with_cap(&parsed, 3)` is `OverEnvelope`; enumerate `run(4)` is Ok and `run(3)` is `ConditionsTooMany`. Delete the "buggy dedup would have PASSED at cap 4" block (the `over4` assertion). The doc comment says raw = 2^(1+1) = 4 and that digests are deduped on both sides.
2. `tests/cli_network_fail_open.rs` (#56). `TESTNET_TPRV` (lines 62-63, an m/84'/1'/0' account tprv, depth 3, used by `e3a_testnet_tprv_mainnet_network_rejects` and `e3b_testnet_tprv_testnet_network_agrees_ok`) becomes the depth-0 testnet master of the abandon…about vector: `tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd`. Its doc comment becomes `/// Testnet master tprv (depth 0) of the abandon…about vector.` The bodies of e3a/e3b are unchanged. Both pass on the new vector on unmodified code too (measured), so the change is neutral to the old behaviour. The vector's mainnet form equals `cli_silent_payment.rs`'s `ROOT_XPRV` (computed from the BIP-39 seed).
3. `tests/cli_export_wallet_sparrow.rs` cells 1, 2, 3 and 5 (#20), through their goldens. In `tests/export_wallet/sparrow_single_wpkh.json` and `sparrow_single_tr.json` the keystore `"label"` becomes `"Keystore 1"`. In `sparrow_multi_2of3_wsh_sortedmulti.json` and `sparrow_tr_multi_a_nums_2of3.json` they become `"Keystore 1"`, `"Keystore 2"`, `"Keystore 3"`. The top-level `"name"` is unchanged. The import fixture `tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json` is an input and does not change.

The greps that looked for other pins, run on this worktree at `2814a2c0`:

| grep | hits outside the bughunt files | why none changes |
|---|---|---|
| `grep -rln '"walletModel"' crates/mnemonic-toolkit/tests` | the four goldens above; import fixtures and import tests | import never uses the emitter (envelopes byte-identical, 2.4) |
| `grep -rn "repeated_digest_cap_agrees_with_enumerate" crates` | `gate.rs:1271` | item 1 |
| `grep -rn "TESTNET_TPRV" crates/mnemonic-toolkit/tests` | `cli_network_fail_open.rs` 63, 335, 355 | item 2 |
| `grep -rn "passphrase ignored" crates/mnemonic-toolkit/tests` | `cli_convert_happy_paths.rs` 494/517 (asserts absent on a PBKDF2 edge); `cli_silent_payment.rs` 257 (depth-0 `ROOT_XPRV` + passphrase) | neither edge changes |
| `grep -rn '"account": *"' crates/mnemonic-toolkit/tests/fixtures` | none | no fixture carries a string `account` |
| `grep -rn -e "InvalidCase" -e "mixes upper and lower" -e "is lenient" crates/mnemonic-toolkit/tests` | `cli_hrp_case_insensitive.rs` 6, 10, 570-571, 609 | drive `inspect`, not `repair` |
| `grep -rln -e "IsShareNotSingleString" -e "one share of a K-of-N" -e "ms12" crates/mnemonic-toolkit/tests \| xargs grep -ln '"repair"'` | none | no repair test feeds a share |
| `grep -rn "threshold=none" crates/mnemonic-toolkit/tests` | `cli_import_wallet_bsms.rs:652`, `cli_import_wallet_roundtrip.rs` 164/225, `cli_import_wallet_bitcoin_core.rs:275` | single-sig; stays `none` |
| `grep -rln "bsms-round1" crates/mnemonic-toolkit/tests` | `cli_bsms_round1.rs`, `cli_import_wallet_bsms_encrypted.rs` | canonical line 3 vectors; pass unchanged |
| `grep -rln "max-indel" crates/mnemonic-toolkit/tests` | `cli_indel.rs`, `cli_ms1_repair_demote.rs`, `cli_repair.rs` | lowercase inputs; pass unchanged |

## 8. Build impact

Each item is a line of 9.1 with status `owed-to-build`, closed by its GREEN commit.

1. Production code: section 3 items 1-13, then `cargo fmt`.
2. Specs: section 4, S1-S9.
3. Tests: every row of section 6 (tightened, new, rewritten, ignored) and the three test changes of section 7.
4. Manual, technical manual, transcripts: section 5.1-5.4 (`fix-g2-doc-edits.py` applies all of them and stops if an old text does not occur exactly once). After G1's edits, the script's 1-occurrence asserts are the check that no anchor moved.
5. Examples corpus: regenerate `.examples-build/Examples.md` with `bash .examples-build/gen.sh` (source-built binary first on PATH, `EXAMPLES_BIN_DIR`) after both FIX documents. G2's share of the diff is the one line of 5.5.
6. `design/FOLLOWUPS.md`: append 9.1, 9.2, 9.3 and 9.4 in that order, and make the two edits of 9.5.

**Gate (not a change).** Run every CI job the PR touches, locally or as green CI on the head:

- `rust.yml`: fmt, clippy `-D warnings` and `cargo nextest run --locked --workspace`;
- the `manual` job (`make -C docs/manual audit`, including `verify-examples` and the doc-flag-lint mutation self-test);
- `examples.yml` (`gen.sh` regen + `git diff --exit-code`);
- `technical-manual`;
- `fuzz-smoke.yml` (it builds the bin crate under `cfg(fuzzing)`; `src/friendly_bughunt_s4.rs` changes);
- `manual-gui`, which must stay green with no manual-gui file changed (5.6).

Then run `coordinator/scripts/mutation-gate.sh <worktree> <the commit before the build>` before any review. Class: A (toolkit manual, holder-facing text) and B (address/keys and import code, `Cargo.lock` untouched). s1 is Opus executed advice plus design and threat; it runs section 2's commands, `doc-commands.sh`, and the advice of 2.14 and 5.1 A6.

## 9. FOLLOWUPS entries (exact text)

### 9.1 Append to `design/FOLLOWUPS.md` (owed to this build)

~~~markdown
### `bughunt-tk-fix-g2` — tk bug-hunt Importants of slices S3/S4/S5: import-wallet taproot threshold (#16 #17), BSMS Round-1 line 3 as written (#18), Coldcard BIP-45 multisig (#19), Sparrow keystore labels (#20), Coldcard account/xfp/master xpub (#29 #31 #49, closes #30), Core different-script merge (#57), convert passphrase warning (#26), compare-cost repeated digest (#28), silent-payment non-master xprv (#56), word-card chunk grouping (#50), inspect second ms1 (#51), repair mixed case / K-of-N shares / capitals under --max-indel (X1 X3 X4): build obligations (repo: **mnemonic-toolkit**; owning phase: **the build of branch `bughunt/triage`**)

- **Surfaced:** 2026-10-31 bug hunt (`design/agent-reports/bughunt-2026-10-31-tk.md`), triage `bughunt-triage-tk-s3.md` / `-s4.md` / `-s5.md`, grades `bughunt-grade-tk-g2.md` (`2814a2c0`). FIX: `design/FIX_bughunt_tk_g2.md` (small-fix track).
- **Grade:** Important ×17; #30 (Minor) closed by #31's fix.
- **Build impact (1):** production code, FIX §3 items 1-13: `owed-to-build`.
- **Build impact (2):** spec sentences, FIX §4 S1-S9: `owed-to-build`.
- **Build impact (3):** RED rows of FIX §6 and the three test changes of FIX §7: `owed-to-build`.
- **Build impact (4):** manual, technical manual and transcripts, FIX §5.1-5.4: `owed-to-build`.
- **Build impact (5):** `.examples-build/Examples.md` regenerated with `gen.sh` after both FIX documents (FIX §5.5): `owed-to-build`.
- **Build impact (6):** FOLLOWUPS entries and edits of FIX §9: `owed-to-build`.
- **Status:** `open`; owed to the build.
- **Tier:** `toolkit`
~~~

### 9.2 Append to `design/FOLLOWUPS.md` (status `deferred-minor`)

Each entry's fix wording is the ruling of `bughunt-grade-tk-g2.md`.

~~~markdown
### `bip388-literal-nums-key-refusal-text` — `export-wallet --format bip388 --descriptor` refuses a NUMS taproot multisig with a text that blames a "descriptor key" use-site (repo: **mnemonic-toolkit**; owning phase: **the next export-wallet spec**)

- **Surfaced:** 2026-10-31 bug hunt #21; triage CONFIRMED; graded Minor (`design/agent-reports/bughunt-grade-tk-g2.md`): fails closed (exit 1), and the refusal itself is correct by BIP-388 (lines 149-151 and 310: only `@N` key placeholders).
- **Fix (ruled):** keep the refusal (exit 1) with the text ``error: --format bip388 cannot express the literal key <hex> (a taproot internal key with no origin, e.g. the BIP-341 NUMS key): a BIP-388 wallet policy allows only @N key placeholders; use --format descriptor or --format bitcoin-core for this wallet``. Do NOT ship the triage's fix-21 (it emits a BIP-388-invalid policy).
- **Open for the architect:** `--template tr-*-multi-a --taproot-internal-key nums --format bip388` emits the literal NUMS hex in `description_template`, which BIP-388 line 310 lists as invalid. Decide whether that path should refuse too.
- **Row:** `bughunt_tk_21_bip388_export_from_descriptor_taproot_nums_multisig` (rewritten to the ruled text, ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `specter-export-descriptor-mode-device-count` — `export-wallet --format specter --descriptor` lists one `devices` entry for an N-key wallet (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #22; graded Minor: every entry is the placeholder `"unknown"`, and Specter reads the list behind a length guard.
- **Fix (ruled):** in `--descriptor` mode, one `"unknown"` per descriptor key, except origin-less single keys (the NUMS internal key is not a device); at least 1. A taproot internal key that has an origin counts.
- **Row:** `bughunt_tk_22_specter_export_descriptor_mode_device_count` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `electrum-import-wallet-name-trailing-digits` — `import-wallet --format electrum` strips trailing digits from the lifted wallet name (`Vault2` → `Vault`) (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #23; graded Minor: descriptor, keys and threshold are right; only the free-text label changes.
- **Fix (ruled):** strip only a `-<digits>` suffix (triage fix-23).
- **Row:** `bughunt_tk_23_electrum_import_wallet_name_digits_preserved` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `core-import-error-index-after-merge` — after a Bitcoin Core receive/change merge, a later entry's error names its index in the merged list, not in the original `descriptors` (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #58; graded Minor: fails closed with the right reason; the quoted bad checksum still locates the entry.
- **Fix (ruled):** carry the original index into `parse_entry` (triage fix-58).
- **Row:** `bughunt_tk_58_bitcoin_core_error_names_original_descriptor_index` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `derive-child-dice-length-cap` — `derive-child --application dice` with a huge `--length` aborts on allocation (SIGABRT) or spins (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #15; graded Minor (an abort on an absurd argument, no output, no data).
- **Fix (ruled):** cap rolls at 1..=1000000: ``error: --length <N> out of range for --application dice (valid: 1..=1000000 rolls)`` (exit 2); `Vec::with_capacity(rolls.min(1024))`.
- **Row:** `bughunt_tk_15_dice_huge_length_is_refused_cleanly` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `xpub-search-sh-wsh-default-origin-notice` — `xpub-search account-of-descriptor` says it defaults origin-elided `sh(wsh(...))` cosigners to `m/48'/…/2'`; BIP-48 script type for nested segwit is `1'` (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #24; graded Minor: the search results are right; only the notice is wrong.
- **Fix (ruled):** `sh(wsh(...))` → `1'` (triage fix-24); plain `sh(...)` keeps `2'`.
- **Row:** `bughunt_tk_24_xpub_search_sh_wsh_default_origin_is_1prime` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `convert-minikey-env-sentinel` — `convert --from minikey=@env:VAR` hands the literal sentinel to the mini-key parser (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #25; graded Minor: fails closed; secret-channel class (operator ruling 2026-08-27).
- **Fix (ruled):** resolve sentinels for every `is_argv_secret_bearing` node (triage fix-25).
- **Row:** `bughunt_tk_25_convert_minikey_env_sentinel_resolves` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `compare-cost-tr-leaf-hash-arg-inflated` — `compare-cost --descriptor tr(…)` glues the x-only `02` prefix onto a 64-hex hash argument and refuses a valid leaf (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #27; graded Minor: fails closed (exit 3) on an advisory cost tool; `--miniscript` works.
- **Fix (ruled):** do not prefix hash-fragment arguments (triage fix-27).
- **Row:** `bughunt_tk_27_compare_cost_tr_leaf_hash_arg_not_inflated` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `xpub-search-account-window-2p31-panic` — `xpub-search` panics (exit 101) for an account window reaching 2^31 (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #32 (sites `template.rs:78` and `descriptor_intake.rs:434`); graded Minor (a panic on an absurd argument).
- **Fix (ruled):** refuse, never clamp: ``error: --min-account / --max-account / --number-of-accounts: the account window must stay below 2147483648 (2^31): BIP-32 hardened account indexes are 31 bits`` (exit 1), when `--min-account` ≥ 2^31, `--max-account` ≥ 2^31, or `--min-account + --number-of-accounts` > 2^31.
- **Rows:** `bughunt_tk_32_path_of_xpub_min_account_2p31_no_panic`, `bughunt_tk_32_account_of_descriptor_zero_window_2p31_no_panic` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `gui-schema-bundle-import-json-template-required` — `gui-schema` marks `bundle --template` required when only `--import-json` is given (repo: **mnemonic-toolkit** + **mnemonic-gui** lockstep; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #33; graded Minor: the GUI's "required" is a marker only; Run is not blocked.
- **Fix (ruled):** add `--import-json` to the rule's `any_of` (triage fix-33); rationale "--template is required unless --descriptor, --descriptor-file or --import-json is supplied (bundle's primary-mode selection invariant)."; `tests/cli_gui_schema_conditional_rules.rs::bundle_template_required_unless_uses_not_any_of_predicate` `predicates.len()` 2 → 3 and asserts `--import-json`; GUI `src/form/conditional.rs::bundle()` in lockstep.
- **Row:** `bughunt_tk_33_gui_schema_bundle_import_json_does_not_require_template` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `cross-repo`

### `gui-schema-verify-bundle-template-required` — `gui-schema` marks `verify-bundle --template` required when `--md1`, `--bundle-json` or a positional is given (repo: **mnemonic-toolkit** + **mnemonic-gui** lockstep; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #34; graded Minor (GUI marker only; the CLI refuses a keyed bundle without `--template` itself).
- **Fix (ruled):** add `--md1`, `--bundle-json` and `positional_present 0` (triage fix-34); GUI `src/form/conditional.rs::verify_bundle()` in lockstep.
- **Row:** `bughunt_tk_34_gui_schema_verify_bundle_alternatives_do_not_require_template` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `cross-repo`

### `compare-cost-concrete-keys-mixed-input` — `compare-cost` omits the "input had concrete keys" note for mixed abstract-label + hex input (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #35; graded Nit (one advisory note; every number is right).
- **Fix (ruled):** triage fix-35.
- **Row:** `bughunt_tk_35_compare_cost_mixed_input_reports_concrete_keys` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `friendly-ms1-length-set-uses-allowed` — `friendly_ms_codec` prints the entr length set for `UnexpectedStringLength` whatever set the error carries (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #36; graded Nit: latent (no input reaches the self-contradictory form). The reachable cousin is ms-codec's own `ms-length-error-lists-entr-set-only` (mnemonic-secret FOLLOWUPS).
- **Fix (ruled):** print the error's `allowed` field (triage fix-36).
- **Row:** `friendly::bughunt_s4::bughunt_tk_36_friendly_length_set_follows_error_allowed` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `verify-message-legacy-signature-trim` — `verify-message --format legacy|auto` refuses a genuine signature with a trailing newline that the BIP-322 path accepts (repo: **mnemonic-toolkit**; owning phase: **none — ownerless residue**)

- **Surfaced:** 2026-10-31 bug hunt #48; graded Minor: a refusal, not an INVALID verdict.
- **Fix (ruled):** trim leading and trailing whitespace only, as the BIP-322 path does (`verify_message.rs:212`; triage fix-48).
- **Row:** `bughunt_tk_48_verify_message_legacy_tolerates_trailing_newline` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`

### `toolkit-repair-inspect-ms1-env-sentinel` — `mnemonic repair --ms1 @env:VAR` (and `inspect`) never resolves the sentinel (repo: **mnemonic-toolkit**; owning phase: **the erasure-fill spec**)

- **Surfaced:** 2026-10-31 bug hunt X2; graded Minor: fails closed; secret-channel class; no toolkit text advertises it, and the GUI reads the variable itself (`mnemonic-gui src/schema/mnemonic.rs:2545-2547`).
- **Fix (ruled):** resolve `@env:VAR` for `--ms1` on `repair` and `inspect` (triage fix-x2); no argv advisory for an `@env:` value; no GUI change.
- **Row:** `bughunt_tk_x2_repair_ms1_resolves_env_sentinel` (ignored).
- **Status:** `deferred-minor`
- **Tier:** `toolkit`
~~~

### 9.3 Append to `design/FOLLOWUPS.md` (status `needs-triage`)

~~~markdown
### `specter-export-devices-strings-vs-objects` — Specter's importer indexes `devices[i]["label"]` / `["type"]` as objects; `export-wallet --format specter` writes plain strings `"unknown"` (repo: **mnemonic-toolkit**; owning phase: **the next tk triage pass**)

- **Surfaced:** `design/agent-reports/bughunt-triage-tk-s3.md` §4 ("Noticed, not tested"); routed by `bughunt-grade-tk-g2.md`, not graded.
- **What:** Specter's `wallet_importer.py:152-160` (cached copy `/scratch/cache/bughunt/tk/s3/ext/specter_wi.py`) reads `cosigners_types[i]["label"]` and `["type"]` for every descriptor key no existing device matches; `"unknown"["label"]` raises a Python TypeError. If confirmed against Specter's current release, every import of a toolkit Specter file whose keys are new to that Specter fails. Needs a two-direction triage (and an upstream source check) before a grade.
- **Status:** `needs-triage`
- **Tier:** `toolkit`

### `xpub-search-max-account-unbounded-allocation` — `xpub-search path-of-xpub --max-account 2147483648` aborts on a 841813590408-byte allocation (exit 134) (repo: **mnemonic-toolkit**; owning phase: **the next tk triage pass**)

- **Surfaced:** `design/agent-reports/bughunt-triage-tk-s4.md` §4 ("Noticed, not tested"); routed by `bughunt-grade-tk-g2.md`, not graded. Sibling of #15, not #32: the candidate `Vec::with_capacity` in `build_candidate_paths` is unbounded for any very large `--max-account` / `--number-of-accounts`.
- **Status:** `needs-triage`
- **Tier:** `toolkit`
~~~

### 9.4 Append to `design/FOLLOWUPS.md` (cross-repo, owed to the GUI's next toolkit pin)

~~~markdown
### `gui-snapshot-tut-j3-09-distinct-hash-count` — after FIX_bughunt_tk_g2 #28, `build-descriptor`'s over-envelope text for the j3 degrading vault reads `1 hashes`; the GUI tutorial snapshot and the manual-gui copy still read `2 hashes` (repo: **mnemonic-gui**, then **mnemonic-toolkit** `docs/manual-gui`; owning phase: **the mnemonic-gui cycle that pins a toolkit carrying #28**)

- **Where:** mnemonic-gui `tests/snapshots/tutorial/tut-j3-09-build-descriptor-refusal.stderr.txt`, then toolkit `docs/manual-gui/transcripts/tutorial/tut-j3-09-build-descriptor-refusal.stderr.txt` when manual-gui re-pins that GUI tag (manual-gui lint step 11 byte-compares the two; `docs/manual-gui/tests/lint.sh:345-355`).
- **What:** the one line `  [over_envelope] root: policy exceeds the always-previewable envelope (2^(11 keys + 2 hashes) × 9 timelock-states > cap 4096); use the raw `--descriptor` path for arbitrarily complex policies` becomes `… (2^(11 keys + 1 hashes) × 9 timelock-states > cap 4096) …`; the refusal itself and its exit code (2) are unchanged. Do not edit the manual-gui copy in the toolkit PR (FIX_bughunt_tk_g2 §5.6).
- **Status:** `open`
- **Tier:** `cross-repo`
~~~

### 9.5 Edits to existing `design/FOLLOWUPS.md` entries

1. `toolkit-repair-indel-search-rejects-capitals`: replace `` - **Status:** `deferred-minor` `` in that entry with `` - **Status:** `resolved` by FIX_bughunt_tk_g2 X4 (`mnemonic repair --max-indel` lowercases an all-capitals card before searching; a mixed-case card is refused with the X1 text). `` The ms primer text (mnemonic-secret FIX 5.6.6) needs no change.
2. `toolkit-repair-ms1-stdin-second-line-ignored`: its title and status stay; after the `- **What (verbatim from the report):**` block, append:
   ~~~markdown
   - **Re-scoped (FIX_bughunt_tk_g2, X5 residual):** the bug hunt's X5 ("second stdin line ignored") is a FALSE POSITIVE: `repair.rs:360` `expand_dashes` repairs every stdin line as its own string (row `bughunt_tk_x5_repair_ms1_stdin_second_line_not_ignored` pins it). The residual is a card wrapped over two lines, which is read as two strings ("too many errors", exit 2). Ruled fix (owning phase: the erasure-fill spec): when `--ms1 -` reads 2 or more non-empty lines, join them only if no line alone has a valid ms1 length and their concatenation does; otherwise each line stays its own string, as today. The manual's "use `-` to read one chunk from stdin" (the `--ms1` rows of `mnemonic repair` and `mnemonic inspect` in `41-mnemonic.md`) is decided in the same spec.
   ~~~

## 10. Overlaps with G1 (`bughunt-grade-tk-g1.md`, `d78646bd`, and its FIX, `design/FIX_bughunt_tk_g1.md`, uncommitted, read 2026-11-01)

There is no shared production file. G1 touches `restore.rs`, `verify_bundle.rs`, `bundle.rs`, `synthesize.rs`, `parse_descriptor.rs`, `electrum.rs`, `wallet_export/electrum.rs` and wc-codec; G2 touches the files of section 3. There is no shared test file either: G1's rows are in `bughunt_s1.rs` / `bughunt_s2.rs` / wc-codec, G2's in `bughunt_s3`–`s5.rs`.

| # | shared surface | G1 ruling | G2 ruling | verdict |
|---|---|---|---|---|
| O1 | `docs/manual/src/40-cli-reference/41-mnemonic.md` | Template-only md1, restore table rows, Multisig template completion, verify-bundle `--expect-wallet-id` row, restore Scope, Non-representable use-site steps, export-wallet `--format` row (`:1135`), bundle `--import-json` row (`:359`) | import-wallet Refusals and Core worked example, silent-payment `--secret` row, inspect `--ms1` row and case line, repair Case / K-of-N / Refusals, compare-cost preimages, build-descriptor step 4 | disjoint anchors; compatible. Build applies G1 first; G2's script asserts each old text once, so a moved anchor stops the build |
| O2 | `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md` | #54: line 345 `path_family` values | #20: Sparrow row and example (D1, D2) | disjoint; compatible; G1 first |
| O3 | `.examples-build/Examples.md` | not named (G1's #3 / 11b refusals may change `bundle --descriptor-file` examples) | one line (#28, 5.5) | compatible: regenerate once with `gen.sh` after both; each FIX's lines are the whole diff |
| O4 | `docs/manual-gui/transcripts/tutorial/` | grade report #54 said "Regenerate the `docs/manual-gui/transcripts/tutorial/*-bundle-json.stdout.txt` …"; G1's FIX supersedes it: "`docs/manual-gui/` is not edited in this PR (section 9.3)", with FOLLOWUPS `manual-gui-transcripts-path-family-descriptor` for the GUI-pin bump | nothing there changes in this PR; `tut-j3-09` follows the GUI pin (5.6, 9.4) | compatible (G1's FIX and this one agree). One GUI re-pin closes both follow-ups |
| O5 | mnemonic-gui `tests/snapshots/tutorial/` | #54: `tut-j{2,3}-restore-feed-bundle-json.stdout.txt` change when the GUI pins the new toolkit | #28: `tut-j3-09-build-descriptor-refusal.stderr.txt` likewise (9.4) | compatible; one GUI re-pin covers both |
| O6 | G1 #5's refusal advice `Use --format bitcoin-core (descriptor) or --format sparrow for taproot watch-only setup.` | the advice runs (`export-wallet --template bip86 --format sparrow` exit 0) | #20 changes that file's keystore label to `Keystore 1` | compatible: the advice still runs as printed and its output now also imports into Sparrow for any wallet name |
| O7 | the JSON field `multisig.path_family` | #54: `bundle`'s own JSON says `"descriptor"` in descriptor mode, including `--import-json` (SPEC v0.3 §5.3) | #19: the `import-wallet` envelope keeps `path_family_from_paths`'s `"bip87"` + notice for `m/45'` (its own spec row) | compatible: different documents. After both, a BIP-45 import's envelope says `bip87` and `bundle --import-json` of it says `descriptor`; no consumer reads the field (G1 settled) |
| O8 | `bundle --import-json` consuming an import-wallet envelope | #45 / #46: network and `--slot` overlay refusals | #16 / #17: the envelope's `multisig.threshold` is now K | compatible: `bundle` reads K from the descriptor tree, not from the envelope field |
| O9 | `design/FOLLOWUPS.md` | appends its entries | appends 9.1-9.4, edits two entries (9.5) | compatible; G1 first |
| O10 | `design/SPEC_export_wallet_v0_8.md` | §9.1 (Electrum), #5: one bullet after the `keystore.xpub` bullet | §7 (Sparrow), #20: S1 and S2 | disjoint; compatible; G1 first |

## 11. STOPs

None. The one conflict found against G1's grade report (O4, manual-gui transcripts) is already resolved the same way in G1's FIX document.

No small-fix limit is exceeded: no flag, user-facing type or command is added, and no spec gets more than three sentences (section 4).

Decisions taken here beyond `bughunt-grade-tk-g2.md` (for the reviewer, not STOPs):

1. **X1 under `--max-indel`.** The grade ruled a mixed-case refusal for every kind. Measured on the first reference cut, the `--max-indel` path still answered a mixed-case chunk with "could not be recovered … or a different error class". `recover_indel_card` now refuses it with X1's text before searching. X4's "lowercase only chunks without a lowercase letter" then reduces to "lowercase every chunk", since none is mixed.
2. **#18 also refuses a record changed after signing.** A record whose line-3 fingerprint case was changed after signing verified before (the rebuild lowercased it back). That fails now (row 18b): a tightening, in the direction of BIP-129 line 81.
3. **#20 changes the manual's Sparrow round-trip promise.** The recipe's `diff` is no longer empty (B2, 5.4). Keystore labels cannot round-trip without a new envelope field, which would be a type change outside the small-fix track. A real Sparrow file's keystore labels (device names) never round-tripped: the old emitter replaced them with the wallet name.
4. **#56 order.** The depth check runs before the network check, so a depth-3 tprv with a contradicting `--network` gets exit 1 (depth) rather than exit 2 (network).
5. **X3 defensive arm.** If a corrected share decodes as a single ms1, the result is `PostCorrectionDecodeFailed` with detail `corrected string decodes as a single ms1, not a K-of-N share`. This is unreachable from the `IsShareNotSingleString` route, so it is kept only as a guard.
