# SPEC: one network-contradiction rule for every tool, units 2, 3, 8a, 8b

Normative rule, texts and vectors: SPEC_network_rule.md (Spec A) at 36c1f155; this spec adds units only and never changes §2/§3.
Spec map: A = SPEC_network_rule.md (§1-§10, §13, units 0, 1, 7); B = SPEC_network_rule_tk_rest.md (units 2, 3, 8a, 8b); C = SPEC_network_rule_mr.md (units 4, 5, 8c, 8g); D = SPEC_network_rule_codecs_gui.md (§6, units 6, 8d, 8e, 8f, 8h).
Units here: 2, 3, 8a, 8b. Machine checks: `scripts/spec-network-rule-gate.sh` reads all four specs.

## 11. FIX units with RED rows

The column legend of the RED-row tables is in SPEC_network_rule.md (Spec A) §11.

### Unit 2: restore `--md1` (mnemonic-toolkit + manual)
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R2.1 | fx | - | `mnemonic restore $(mdargs m_signet.txt) --count 1` | 0 | `bc1qvqnan` | - | `coin type` | 8 | TX-S (run form), naming the three test flags and the override |
| R2.2 | fx | - | `mnemonic restore $(mdargs m_signet.txt) --network mainnet --count 1` | 0 | `bc1qvqnan` | - | `coin type` | 8 | TX-S; with the flag 0, `bc1qvqnan...`, TX-S-OK |
| R2.3 | fx | - | `mnemonic restore $(mdargs m_mainnet.txt) --network testnet --count 1` | 0 | `tb1q7cty6` | - | `coin type` | 8 | TX-S |
| R2.4 | fx | - | `mnemonic restore $(mdargs m_signet.txt) $(cos m_signet.txt 8 9 0) $(cos m_signet.txt 12 13 1) --count 1` | 0 | `bc1qvqnan` | - | `network mismatch` | 8 | TX-H1 (`--cosigner @0's and @1's mk1 cards are test-network keys (tpub), but this run is on mainnet (no --network given)`), nothing else; the flag does not pass it |
| R2.5 | fx | - | `mnemonic restore $(mdargs m_signet.txt) --network signet --count 1` | 0 | `tb1qvqnan` | - | `network mismatch` | 0 | unchanged (control) |
| R2.6 | fx | - | `mnemonic restore $(mdargs m_mainnet.txt) --count 1` | 0 | `bc1q7cty6` | - | `network mismatch` | 0 | unchanged (control; coin 0 agrees) |
| R2.7 | fx | - | `mnemonic restore $(mdargs m_mainnet.txt) $(cos m_mainnet.txt 8 9 0) $(cos m_mainnet.txt 12 13 1) --network signet --count 1` | 0 | `tb1q7cty6` | - | `network mismatch` | 8 | TX-H1 (main keys, `--network is signet`) |
| R2.8 | fx | - | `mnemonic restore $(mdargs m_signet.txt) $(cos m_signet.txt 8 9 0) $(cos m_signet.txt 12 13 1) --network signet --count 1` | 0 | `cross-checked` | - | `network mismatch` | 0 | unchanged (control: cosigners on the run's network) |
| R2.9 | fx | - | `mnemonic restore $(mdargs m_signet.txt) --from "phrase=$P" --allow-argv-secret --count 1` | 0 | - | - | `coin type` | 8 | TX-S (run form; a seed does not exempt the md1 coin type); stdout empty |

TRACE rows: `nr2_restore_md1_coin_type_against_run_refused` (R2.1, R2.2,
R2.3, R2.9), `nr2_restore_md1_cosigner_version_refused_flag_ignored` (R2.4, R2.7),
`nr2_restore_md1_controls_unchanged` (R2.5, R2.6, R2.8),
`nr2_restore_md1_no_source_tx_n_tx_t` (fixture owed: a keyed md1 at `m/45'/...`, `fx/m_bip45.txt`).
restore --md1 of a keyless template with --from is a seed-typed-now run (N6): no TX-N/TX-T. With no --cosigner, the template's origins' coin types are still compared with the run (R2.9; Spec A §4.1): a seed typed now does not exempt them. With --cosigner mk1 cards (multisig template completion), the own origin is copied from a cosigner mk1's path family (restore.rs:1210 at 68ea05e0), so each --cosigner mk1 is checked as Spec A §4.1 says: its key version against the run (TX-H1) and its own origin coin type against its own key (TX-S key form, the mk1 own-coin arm). With --origin, the restore --origin arm applies. A template whose origins carry no N1 coin type meets no contradiction of its own.

### Unit 3: import-wallet Coldcard and Jade (mnemonic-toolkit + manual)
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R3.1 | fx | - | `mnemonic import-wallet --blob cc_BTC_T_1.json --json` | 0 | `"network": "mainnet"` | - | `network mismatch` | 8 | TX-H4C (`cc_BTC_T_1.json says chain BTC (mainnet), but its bip84 key is a test-network key (tpub)`) |
| R3.2 | fx | - | `mnemonic import-wallet --blob cc_BTC_T_0.json --json` | 0 | `"network": "mainnet"` | - | `network mismatch` | 8 | TX-H4C only (`cc_BTC_T_0.json says chain BTC (mainnet), but its bip84 key is a test-network key (tpub)`), no TX-S for the coin-0h path; stdout empty |
| R3.3 | fx | - | `mnemonic import-wallet --blob cc_XTN_X_0.json --json` | 0 | `"network": "testnet"` | - | `network mismatch` | 8 | TX-H4C (chain XTN, a mainnet key) |
| R3.4 | fx | - | `mnemonic import-wallet --blob cc_BTC_X_0.json --json` | 0 | `"network": "mainnet"` | - | `network mismatch` | 0 | unchanged (control) |
| R3.5 | fx | - | `mnemonic import-wallet --blob cc_BTC_T_1.json --network testnet --json` | 1 | - | `coin-type-0 network` | - | 8 | TX-H4C, same as R3.1 |
| R3.6 | fx | - | `mnemonic import-wallet --blob jade_T_0.json --json` | 2 | - | `coldcard-multisig` | - | 8 | TX-H1 (`jade keys @0, @1 and @2 are test-network keys (tpub), but this run is on mainnet (no --network given)`); then TX-S without its flag sentence |
| R3.7 | fx | - | `mnemonic import-wallet --blob jade_X_1.json --json` | 2 | - | `coldcard-multisig` | - | 8 | TX-S; with the flag 0 |
| R3.8 | fx | - | `mnemonic import-wallet --blob jade_mixed_0.json --json` | 2 | - | `coldcard-multisig` | - | 8 | TX-H2, TX-H1 without its rerun sentence (N8), then TX-S without its flag sentence (key @1's tpub against the coin-0 path, as V15) |
| R3.9 | fx | - | `mnemonic import-wallet --blob jade_T_1.json --json` | 0 | `"network": "testnet"` | - | `network mismatch` | 8 | TX-H1 (`no --network given`) |
| R3.10 | fx | - | `mnemonic import-wallet --blob ccms_T_0.txt --json` | 2 | - | `coldcard-multisig` | - | 8 | TX-H1, then TX-S without its flag sentence (the same keys as R3.6) |

TRACE rows: `nr3_coldcard_chain_against_key_refused` (R3.1, R3.2, R3.3,
R3.5), `nr3_coldcard_consistent_unchanged` (R3.4),
`nr3_jade_names_jade_and_exits_8` (R3.6, R3.7, R3.8),
`nr3_import_test_blob_no_flag_refused` (R3.9, R3.10).

### Unit 8: the rest (one PR each, in this order)

The order of the sub-units is in SPEC_network_rule.md (Spec A), Unit 8.

8a toolkit remainder (+ manual):
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R8a.1 | fx | - | `mnemonic bundle --descriptor "wpkh([73c5da0a/84h/1h/0h]$T/<0;1>/*)" --network mainnet --json` | 0 | `"network":"mainnet"` | - | `network mismatch` | 8 | TX-H1; no JSON |
| R8a.2 | fx | - | `mnemonic bundle --descriptor "wpkh([73c5da0a/84h/0h/0h]$X/<0;1>/*)" --network testnet --json` | 0 | `"network":"testnet"` | - | `network mismatch` | 8 | TX-H1 |
| R8a.3 | fx | - | `mnemonic convert --from phrase="$P" --to xpub --path m/84h/0h/0h --network testnet --allow-argv-secret` | 0 | `tpubDCxX2s` | - | `coin type` | 8 | TX-S (run form); with the flag 0 |
| R8a.4 | fx | - | `mnemonic convert --from phrase="$P" --to xpub --path m/84h/1h/0h --network mainnet --allow-argv-secret` | 0 | `xpub6Bm9M1` | - | `coin type` | 8 | TX-S |
| R8a.5 | fx | - | `mnemonic convert --from xpub=$T --to xpub --network mainnet` | 0 | `tpubDC8msF` | - | `network mismatch` | 8 | TX-H1 |
| R8a.6 | fx | - | `mnemonic restore --from "phrase=$P" --origin m/84h/0h/0h --network testnet --template bip84 --allow-argv-secret --count 1` | 0 | `tb1q6rz28mc` | - | `coin type` | 8 | TX-S |
| R8a.7 | fx | - | `mnemonic xpub-search path-of-xpub --phrase "$P" --target-xpub $T --allow-argv-secret` | 4 | - | `secret material on argv` | `network mismatch` | 8 | TX-H1 (`--target-xpub is a test-network key (tpub)`), before any search |
| R8a.8 | fx | - | `mnemonic xpub-search address-of-xpub --xpub $X --target-address tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl --address-type p2wpkh` | 4 | - | `no match in searched set` | - | 8 | TX-H3 (signet or testnet address, `no --network given`) without its rerun clause: `... but this run is on mainnet (no --network given). Nothing was written. Give an address of mainnet.`; stdout empty |
| R8a.9 | fx | - | `mnemonic nostr --pubkey 79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798` | 0 | `bc1pmfr3p9j` | - | `names a network` | 0 | stderr has TX-N |
| R8a.10 | fx | - | `mnemonic bundle --template bip84 --slot "@0.wif=cTGhosGriPpuGA586jemcuH9pE9spwUmneMBmYYzrQEbY92DJrbo" --network mainnet --allow-argv-secret` | 0 | `mk1qp` | - | `network mismatch` | 8 | TX-H5 (WIF of the test class, `--network is mainnet`); stdout empty |
| R8a.11 | fx | - | `mnemonic export-wallet --template bip84 --slot "@0.xpub=$T" --slot "@0.fingerprint=73c5da0a" --slot "@0.path=m/84h/0h/0h" --network signet --format bitcoin-core-addresses --count 1` | 0 | `tb1q6rz28mc` | - | `coin type` | 8 | TX-S (key form); with the flag 0, TX-S-OK |
| R8a.12 | fx | - | `mnemonic bundle --template bip84 --slot "@0.phrase=$P" --slot "@0.path=m/84h/0h/0h" --network signet --allow-argv-secret` | 0 | `mk1qp` | - | `coin type` | 8 | TX-S (run form); stdout empty |
| R8a.13 | fx | - | `mnemonic convert --from xprv=tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd --to xpub --network mainnet --allow-argv-secret` | 0 | `tpubD6Nz` | - | `network mismatch` | 8 | TX-H1 (`--from xprv`); stdout empty |
| R8a.14 | fx | - | `mnemonic convert --from xpub=$T --to xpub` | 0 | `tpubDC8msF` | - | `network mismatch` | 8 | TX-H1 (`--from xpub`, no --network given) |
| R8a.15 | fx | - | `mnemonic convert --from "mk1=$(tr '\n' ' ' < mk_T_c1.txt)" --to xpub` | 0 | `tpubDC8msF` | - | `network mismatch` | 8 | TX-H1 (`--from mk1`) |
| R8a.16 | fx | - | `mnemonic convert --from xpub=$T --to fingerprint` | 0 | - | - | `network mismatch` | 8 | TX-H1 (a target that names no network refuses too, D1) |
| R8a.17 | fx | - | `mnemonic convert --from phrase="$P" --to address --script-type p2wpkh --path m/84h/1h/0h/0/0 --network mainnet --allow-argv-secret` | 0 | `bc1q6rz28mc` | - | `coin type` | 8 | TX-S (run form; `the path m/84h/1h/0h/0/0 given with --path`); with the flag 0 |
| R8a.18 | fx | - | `mnemonic derive-child --from xprv=xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu --application xprv --length 0 --index 0 --network testnet --allow-argv-secret` | 0 | `tprv8Zgx` | - | `network mismatch` | 8 | TX-H1 (main key, `--network is testnet`); stdout empty |
| R8a.19 | fx | - | `mnemonic derive-child --from xprv=tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd --application hex --length 16 --index 0 --allow-argv-secret` | 0 | `^[0-9a-f]{32}$` | - | `network mismatch` | 8 | TX-H1 (test key, no --network given); stdout empty |
| R8a.20 | fx | - | `mnemonic xpub-search account-of-descriptor --phrase "$P" --descriptor "wpkh([73c5da0a/84h/1h/0h]$T/<0;1>/*)" --allow-argv-secret` | 4 | - | `no match in searched set` | `network mismatch` | 8 | TX-H1 (`key @0`, no --network given), before any search |
| R8a.21 | fx | - | `mnemonic convert --from phrase="$P" --to xprv --path m --network testnet --allow-argv-secret` | 0 | `tprv8Zgx` | - | `This rehearsal used your real seeds` | 0 | stdout unchanged; the last stderr line is TX-R |
| R8a.22 | fx | - | `mnemonic restore --from "phrase=$P" --template bip84 --expect-xpub $T --allow-argv-secret --count 1` | 4 | - | `MISMATCH` | `network mismatch` | 8 | TX-H1 (`--expect-xpub is a test-network key (tpub)`, no --network given), before the expect check; stdout empty |
| R8a.23 | fx | - | `mnemonic restore --from "phrase=$P" --template bip84 --expect-xpub $T --allow-mismatch --allow-argv-secret --count 1` | 0 | - | - | `network mismatch` | 8 | TX-H1; `--allow-mismatch` does not pass it; stdout empty |
| R8a.24 | fx | - | `mnemonic convert --from "mk1=$(tr '\n' ' ' < mk_X_c1.txt)" --to xpub` | 0 | `xpub6CatWd` | - | `coin type` | 8 | TX-S (key form: `--from mk1's path m/84h/1h/0h is a test-network path (coin type 1h), but --from mk1 is a mainnet key (xpub)`, naming the flag); stdout empty; with `--accept-coin-type-mismatch` 0, stdout `xpub6CatWd...`, stderr TX-S-OK |

TRACE rows: `nr8a_bundle_descriptor_refused` (R8a.1, R8a.2),
`nr8a_convert_path_coin_type_soft` (R8a.3, R8a.4),
`nr8a_convert_xpub_to_xpub_refused` (R8a.5), `nr8a_mk1_own_coin_type_soft` (R8a.24; leg: restore of a keyless multisig template with a cosigner mk1 whose path coin disagrees with its key),
`nr8a_restore_origin_coin_type_soft` (R8a.6),
`nr8a_xpub_search_refuses_before_search` (R8a.7, R8a.8),
`nr8a_nostr_pubkey_prints_tx_n` (R8a.9),
`nr8a_import_wallet_coin_type_flag`, `nr8a_derive_child_tprv_no_flag_refused`,
`nr8a_verify_bundle_cause_line`, `nr8a_verify_bundle_network_refused_exit_8`,
`nr8a_nostr_secret_prints_tx_n`, `nr8a_restore_expect_xpub_network_first`,
`nr8a_tx_r_on_secret_output_test_run`. TX-R in the toolkit is printed by one post-run hook in main.rs after the verb returns (so it follows the verb's own advisories) when the run network is a test network, the verb's stdout class is PrivateKeyMaterial and its input held a seed, ms1, xprv/tprv or WIF; the verbs that can meet this are bundle, convert, derive-child, import-wallet and silent-payment (the PrivateKeyMaterial verbs that have --network). nostr's nsec is no N12 input, so nostr never prints TX-R, and the row "nostr --secret..." drops "; TX-R when N12 holds" and "/ TX-R". verify-bundle requires --network, so its run is always the flag; it prints TX-H1 (mk1 cards) before TX-S (md1 coin types). nostr prints its key advisory, then TX-N or TX-T, and never TX-R. New TRACE rows: nr8a_bundle_slot_wif_refused (R8a.10), nr8a_slot_path_coin_type_soft (R8a.11, R8a.12), nr8a_convert_every_key_source_refused (R8a.13-R8a.16), nr8a_convert_path_coin_type_soft gains R8a.17, nr8a_derive_child_tprv_no_flag_refused (R8a.18, R8a.19), nr8a_xpub_search_account_of_descriptor_key_refused (R8a.20), nr8a_tx_r_on_secret_output_test_run (R8a.21; legs derive-child, bundle, silent-payment), nr8a_restore_expect_xpub_network_first (R8a.22, R8a.23). Fixtures owed by 8a: a verify-bundle bundle with a test mk1 checked under --network mainnet (made with --slot flags or --bundle-json: a bundle --slot @0.phrase= single-sig bundle fails earlier with "slot count 0 out of range"), and an import-wallet descriptor blob with an xpub at coin 1 for nr8a_import_wallet_coin_type_flag. TRACE `nr8a_bundle_slot_wif_refused` (R8a.10) has the legs: the mainnet WIF with `--network signet`; `--network regtest`.

8b exit 8 and one wording on the arms that already refuse, and the
inferring arms (toolkit, md, mk + manual):
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R8b.1 | fx | - | `mnemonic addresses --from xpub=$T --count 1 --address-type p2wpkh` | 0 | `tb1q6rz28mc` | - | `network mismatch` | 8 | TX-H1 (`no --network given`) |
| R8b.2 | fx | - | `mnemonic addresses --from xpub=$T --count 1 --address-type p2wpkh --network mainnet` | 1 | - | `disagrees with the xpub's network kind` | - | 8 | TX-H1 |
| R8b.3 | fx | - | `md address --template "wpkh(@0/<0;1>/*)" --key "@0=$T"` | 1 | - | `expected mainnet xpub version 0488B21E` | - | 8 | TX-H1 (`--key @0 is a test-network key (tpub)`) |
| R8b.4 | fx | - | `mnemonic convert --from xpub=$T --to address --script-type p2wpkh --path m/0/0` | 0 | `tb1q6rz28mc` | - | `network mismatch` | 8 | TX-H1 |
| R8b.5 | fx | - | `mk address $(cat mk_T_c1.txt) --address-type p2wpkh` | 0 | `tb1q6rz28mc` | - | `network mismatch` | 8 | TX-H1 |
| R8b.6 | fx | - | `mk address $(cat mk_T_c1.txt) --address-type p2wpkh --network mainnet` | 64 | - | `disagrees with the xpub's network` | - | 8 | TX-H1 |

TRACE rows: `nr8b_inferring_arms_refuse_without_flag` (R8b.1, R8b.4,
R8b.5), `nr8b_existing_refusals_exit_8_one_wording` (R8b.2, R8b.3, R8b.6), `nr8b_md_from_mk1_version_refused` (fixture owed: a keyless md1 plus `mk_T_c1.txt`).

## 12. Build impact

3. Unit 2: toolkit `cmd/restore.rs`; tests: restore md1 suites that pin
   rc 0 on a mismatched network; manual 41-mnemonic `restore`.
4. Unit 3: toolkit `wallet_import/coldcard.rs`
   ({{mnemonic-toolkit@4553d2da:crates/mnemonic-toolkit/src/wallet_import/coldcard.rs:67}} "`BTC` → mainnet, `XTN` → testnet."),
   `coldcard.rs` lines 156-159, 216-221 and 276 (the other BTC/XTN sites, rest N-2),
   `wallet_import/jade.rs`, `cmd/import_wallet.rs`; tests:
   `tests/cli_import_wallet_snet_network_mismatch.rs`,
   `tests/cli_import_wallet_network_override.rs` (exit codes, the flag);
   manual 41-mnemonic `import-wallet` exit table.
9. Unit 8a: toolkit `cmd/bundle.rs`, `cmd/convert.rs`, `cmd/restore.rs`,
   `cmd/xpub_search*`, `cmd/nostr.rs`, `cmd/derive_child.rs`,
   `cmd/verify_bundle.rs`, the secret-advisory site (TX-R); tests:
   `tests/cli_network_fail_open.rs` rows; manual 41-mnemonic pages of each
   verb; mnemonic-gui schema mirror (`--accept-coin-type-mismatch` on every
   verb that takes it; GUI-managed from the first mirror commit, Spec D §6.1).
10. Unit 8b: toolkit `cmd/addresses.rs`, `cmd/convert.rs`, `synthesize.rs`
   (`CosignerSpec` exit), `network.rs`; md `crates/md-cli/src/parse/keys.rs`,
   `decompose/mod.rs`, md-cli `src/seat/`; mk `crates/mk-cli/src/cmd/address.rs`; tests:
   `e1c_testnet_tpub_to_address_no_network_infers_ok` flips, every
   `assert_mismatch` exit in `tests/cli_network_fail_open.rs`, the --help text of addresses and mk address ("Defaults to the xpub's version bytes") becomes "Defaults to mainnet; an xpub of another class is refused", md and mk
   network refusal pins; manual 41, 42, 44 exit tables.
