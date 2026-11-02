**⚠ 1 Important: the new `--path` branch can drop `--passphrase` or `--language` and give a wrong key while all 30 rows pass.**

# Whole-diff review s2 (Sonnet): tests-can-fail, `convert --path`

Target `fix/convert-path` at 0a354375 (`git diff 6ccfdebc..0a354375`). Lens: plant each defect back, check the row fails for the stated reason, and check no expectation change hides a regression.
Method: `git archive HEAD` into `/scratch/cache/convert-path/wd-s2/`, own `target/`, harness `/scratch/cache/convert-path/mut-s2.py`. Per-mutant outputs `wd-s2/mut-Mnn.txt`, summaries `wd-s2/mutres.txt` and `mutres2.txt`. Base: 30/30 pass in the scratch copy. The real worktree is untouched except this report.

## No expectation change hides a regression
The diff touches no existing test file. `cli_convert_path.rs` is new. The only pre-existing text changed is the `--template is required…` message, and no other test or doc greps for the old text (workspace green per the gate).

## Plants (convert.rs, each one at a time, run as `binary(cli_convert_path)`)
Caught, with the failing row(s) as the stated reason:
- M01 R1 branch off: 6 rows fail (R1 rows, `convert_template_xpub_path_not_silently_ignored`).
- M02 R4 check off: `r4_xprv_to_xpub_path_refused`. M03 R3 check off: `r3_account_nonzero_with_path_refused_wif`.
- M04 R4 `any` to `all`: one row. M05 drop Xpub from seed row: 3 rows. M07 drop Wif: caught.
- M10 add Fingerprint to seed row, M11 add Phrase: `r4_phrase_to_fingerprint_path_refused`, `r4_phrase_to_entropy_path_refused`.
- M12-M14 xpub row false / true / +Fingerprint, M15 other sources true: caught.
- M17 template wins over path: caught (`r1_path_wins_over_template`).
- M20 network forced mainnet: 4 rows.
- M21 `--path` parse error as exit 2 instead of 1, M22 old no-template text, M23 and M24 refusal texts, M25 named/hex lines back in `--help`, M26 old `--account` help: each caught.
- M27 swap R3 before R4: caught (`r4_before_r3_xprv_to_xpub_path_and_account`).

SURVIVED (all 30 rows pass with the defect planted):
- M06 drop `Xprv` from the seed row of `reads_path`.
- M08 drop `Bip38`. M09 drop `Address`.
- M16 `args.account != 0` changed to `> 1` (account 1 with `--path` accepted).
- M18 pass `""` instead of `pbkdf2_passphrase` to `derive_bip32_from_entropy_at_path`.
- M19 pass English instead of `language.into()`.
M28 was a malformed plant (compile error), ignored; fingerprint-is-master is covered by `r2_path_fingerprint_is_master` (fingerprint 73c5da0a differs from any account fingerprint) and by the master-fingerprint line in `derive_bip32_from_entropy_at_path` (derive_slot.rs:83).

## Findings

### I1. `--passphrase` and `--language` are never exercised through the new `--path` branch
convert.rs (the `if needs_derive && args.path.is_some()` branch, about line 1338): it passes `pbkdf2_passphrase` and `language.into()` to the new call site. No row sets `--passphrase` or `--language` with `--path` and `--to xpub/xprv`. Plants M18 and M19 compile, exit 0, print a key, and pass 30/30. M18 gives the key of the passphrase-less wallet for a holder who typed a passphrase: a wrong key, silently. That is the harm this FIX exists to remove, so the row set cannot catch the class.
Fix: add these two rows (run as printed in the scratch copy; both PASS on the branch, FAIL under M18 and M19 respectively, observed `FAIL ... s2_path_honours_passphrase` and `FAIL ... s2_path_honours_language`). Expected values cross-checked: the passphrase xpub/xprv equal the output of `--passphrase TREZOR --template bip84` for the same phrase; the Spanish xpub equals `--language spanish --template bip84`.
```
#[test]
fn r1_path_honours_passphrase() {
    let o = convert(&["--from", &phrase(), "--network", "signet", "--to", "xpub,xprv", "--passphrase", "TREZOR", "--path", "m/84'/1'/0'"]);
    assert_ok(&o, &[
        "xpub: tpubDC3QUzyNJLFENW1qz75ATkef1wkGURb2bvPiBd7ty35wpR9oTzV12aHjzWFr9eZDnL5H9Wqbq6riaHcfa4FR4iv22uFLufe7Ldor9jr2MJF".to_string(),
        "xprv: tprv8fMNLaw89xZZV2z46TQa4LzYSvELK6Q82cnvu75bYmHYyvu2qbfQr5fspNnjRRF9D4Fte48koK2tWB6ya3xY43KdPvTmFiH76h8W48rLLRQ".to_string(),
    ]);
}

#[test]
fn r1_path_honours_language() {
    let o = convert(&["--from", "phrase=ábaco ábaco ábaco ábaco ábaco ábaco ábaco ábaco ábaco ábaco ábaco abierto", "--language", "spanish", "--network", "signet", "--to", "xpub", "--path", "m/84'/1'/0'"]);
    assert_ok(&o, &["xpub: tpubDCbgL9wqirauBJJQHwtmGoTJx7rqvTAJ1PabwjpnVCu2prdzKRseAyqBsDFpPjstykf4k5LCZpg69GaDD6pXuzddbphbZv6BRuczCnPyKvG".to_string()]);
}
```
(Scratch names are `s2_path_honours_*`; the full scratch test file with all six added rows is `/scratch/cache/convert-path/s2-extra-rows-cli_convert_path.rs`. 36/36 pass on the branch.)

### M1 (Minor). Lone `--to xprv`, `--to bip38`, `--to address` with `--path` from a seed are untested
Plants M06, M08, M09 survive: a regression that makes `reads_path` refuse these valid single-target calls (exit 2 "drop --path") passes the suite. Fails closed. Rows (scratch `s2_path_alone_*`, all pass on the branch, each fails under its plant):
`--from phrase=P12 --network signet --to xprv --path m/84'/1'/0'` gives `xprv: {XPRV_84}`;
`--from phrase=P12 --to bip38 --bip38-passphrase x --path m/84'/0'/0'/0/0` gives `bip38: 6PYP8fdoWkbfb1nGtgRPgSRUsHK113HqGCvrY6YHWMm3BvnKGRnH564c8J`;
`--from phrase=P12 --network signet --to address --script-type p2wpkh --path m/84'/1'/0'/0/0` gives `address: tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl` (the BIP-84 testnet first address for this seed).

### N1 (Nit). R3 boundary at account 1
M16 (`> 1`) survives: the R3 rows use accounts other than 1. Scratch row `s2_account_one_with_path_refused` (`--account 1`, expects R3_TEXT, exit 2) catches it. Equivalent-looking in practice, listed for completeness.

### N2 (Nit). Extra workspace test
Not identified (4175 vs 4174); per the brief, not a finding.

C / I / M / N = 0 / 1 / 1 / 2
