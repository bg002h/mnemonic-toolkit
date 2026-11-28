# Fixtures added by the toolkit build plan (2026-11-28)

All from public test data: `$P` = "abandon ... about", `$Q` = "legal winner thank year wave
sausage worth useful legal winner thank yellow", `$X`/`$T` as in scripts/spec-network-rule-gate.sh.
Made with the toolkit built from master d72a18b1.

- `m_bip45.txt` (nr2_restore_md1_no_source_tx_n_tx_t): keyed md1 at m/45' (no N1 coin).
  KA=`mnemonic convert --from phrase=$P --to xpub --path m/45h --network mainnet`,
  KB the same from $Q (fingerprints 73c5da0a, b8688df1 from `--to fingerprint --template bip84`);
  `mnemonic bundle --network mainnet --descriptor "wsh(sortedmulti(2,[73c5da0a/45h]$KA/<0;1>/*,[b8688df1/45h]$KB/<0;1>/*))"`,
  md1 lines kept. Today: restore rc 0, bc1qnstvrzm9... / tb1qnstvrzm9... with --network signet.
- `vb_signet_bip84.txt` (nr8a_verify_bundle_network_refused_exit_8): `mnemonic bundle --network signet
  --template bip84 --slot "@0.phrase=$P"` stdout. Leg: `verify-bundle --network mainnet --template bip84
  --slot "@0.phrase=$P" --mk1 <mk1 lines> --md1 <md1 line>`; today rc 4 with mk1_xpub_match/path/md1 rows
  failing. The seed slot is no source, so the mk1 card is the only HARD source (TX-H1 `the mk1 card`).
  The signet leg without --ms1 (rc 4 today only on ms1_decode) is the TX-V negative leg (see below).
- `d_x1.txt` (nr8a_import_wallet_coin_type_flag): descriptor blob, mainnet $X at 84h/1h/0h;
  `import-wallet --format descriptor --blob d_x1.txt --json`; today rc 2 (NetworkMismatch).
- `d_mixcoin.txt` (nr8a_import_wallet_coin_type_flag, I-5 leg): wsh(sortedmulti(1, xpub at 48h/0h/0h/2h,
  xpub at 48h/1h/0h/2h)); today rc 2 "cosigner 1 has coin-type 1, cosigner 0 has coin-type 0".
  The second key's fingerprint 97139860 is a label, not that key's real fingerprint (import does not check).
- `m_signet_xcos.txt` (as1-i4 leg of nr2_restore_md1_cosigner_version_refused_flag_ignored): the two
  cosigner tpubs of m_signet.txt (mk1 lines 8-9, 12-13) with version bytes swapped to 0488b21e (xpub),
  same key data. `restore $(mdargs m_signet.txt) --cosigner @0=<line1> --cosigner @1=<line2> --count 1`:
  today rc 0 "cross-checked"; after: SOFT only (keys agree with mainnet, md1 coin 1 disagrees with them).
- `vb_mainnet_coin5.txt` (nr8a_verify_bundle_cause_line, TX-V positive leg, R1-I2): X5=`mnemonic convert
  --from phrase=$P --to xpub --path m/84h/5h/0h --network mainnet`; `mnemonic bundle --network mainnet
  --descriptor "wpkh([73c5da0a/84'/5'/0']$X5/<0;1>/*)"` stdout (watch-only, no ms1). Leg: `verify-bundle
  --network mainnet --template bip84 --slot "@0.phrase=$P" --ms1 <ms1 of vb_signet_bip84.txt> --mk1 ..
  --md1 ..`; today rc 4 with exactly mk1_xpub_match, mk1_path_match, md1_xpub_match, md1_origin_match
  failing. Coin 5 is no source (B1) and the key is a mainnet xpub, so no source contradicts the run.
- TX-V negative leg (R1-I2): `vb_signet_bip84.txt` with `--network signet` and no `--ms1`: today rc 4 with
  only ms1_decode failing; after: no TX-V. The mainnet refusal leg of vb_signet_bip84.txt passes `--ms1`
  too, so its today rows are only the four network rows.
