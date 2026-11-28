# Test census for the toolkit network rule (plan r1, 2026-11-28, at d72a18b1)

Method: a grep census, not a run. For every integration test function in
crates/mnemonic-toolkit/tests/*.rs whose body holds test-network material (tpub/tprv/vpub/upub,
a tb1/bcrt1 address, a coin-1 origin) or a mainnet key with a test --network, the flag column
says what the body passes: NOFLAG (no --network: D1 makes the run mainnet), MAINNET, TESTFLAG,
MAINKEY+TESTFLAG. Expected outcome by class:
- NOFLAG / MAINNET with a test KEY source: flips to exit 8 (TX-H1), unless the material is only
  derived output (a tpub printed from a seed on a test flag) or an address that is only read.
- TESTFLAG with test keys: unchanged, unless a coin-0 path meets a test key (TX-S, exit 8).
- MAINKEY+TESTFLAG: flips to exit 8 (TX-H1) where the main key is an input.
Rule for the implementer: at each unit, every test that turns red must be on this list (or be
explained by a spec row and added here with that row). Change it to the spec's exit and text;
never delete it or loosen its assertion. A red test that is not on the list and is not explained
by a spec row is a regression.

## Integration tests (function, flag class)

- bughunt_1119_unbounded_args.rs::bh_tk_mn18_add_path_count_unchanged NOFLAG
- bughunt_s4.rs::bughunt_tk_56_silent_payment_refuses_non_master_xprv NOFLAG
- cli_addresses.rs::network_infer_override_and_mismatch MAINNET+TESTFLAG
- cli_addresses_electrum.rs::electrum_watch_only_no_xpriv NOFLAG
- cli_bip388_validity.rs::p25_controls_cosigner_internal_key_and_named_alternatives TESTFLAG
- cli_bip388_validity.rs::bip388v_export_descriptor_same_xpub_other_origin_refused NOFLAG
- cli_bip388_validity.rs::bip388v_export_descriptor_same_pubkey_other_chaincode_refused NOFLAG
- cli_build_descriptor.rs::preset_success_json_and_network_compose TESTFLAG
- cli_convert_address.rs::phrase_to_account_upub_bip49_reference_testnet TESTFLAG
- cli_convert_address.rs::xpub_to_address_p2pkh_explicit_script_type_v0_26_0 MAINKEY+TESTFLAG(testnet)
- cli_convert_address.rs::xpub_to_address_testnet_explicit_network_with_vpub TESTFLAG
- cli_convert_convergence.rs::c3_four_encodings_same_master_fp NOFLAG
- cli_convert_convergence.rs::c4_slip0132_variant_octet_round_trip MAINNET+TESTFLAG
- cli_convert_path.rs::r4_guard_xpub_to_address_path_accepted TESTFLAG
- cli_convert_path.rs::s2_path_alone_address_accepted TESTFLAG
- cli_convert_path.rs::s2_path_honours_passphrase TESTFLAG
- cli_convert_path.rs::s2_path_honours_language TESTFLAG
- cli_convert_slip0132.rs::input_normalizer_testnet_vpub_to_xpub_normalizes_to_tpub NOFLAG
- cli_convert_slip0132.rs::input_normalizer_testnet_upub_to_xpub_normalizes_to_tpub NOFLAG
- cli_convert_slip0132.rs::output_xpub_prefix_testnet_zpub_emits_vpub TESTFLAG
- cli_convert_slip0132_info.rs::convert_variant_coverage_upub_testnet NOFLAG
- cli_convert_slip0132_info.rs::convert_variant_coverage_big_u_testnet NOFLAG
- cli_convert_slip0132_info.rs::convert_variant_coverage_vpub_testnet NOFLAG
- cli_convert_slip0132_info.rs::convert_variant_coverage_big_v_testnet NOFLAG
- cli_decode_address.rs::testnet_address_reports_network_set NOFLAG+MAINNET+TESTFLAG
- cli_derive_child.rs::xprv_child_testnet_prefix TESTFLAG
- cli_derive_child.rs::dice_missing_dice_sides_refused NOFLAG
- cli_derive_child.rs::r3_non_master_tprv_refused TESTFLAG
- cli_derive_child.rs::r5_non_master_xprv_depth2_refused NOFLAG
- cli_descriptor_concrete.rs::export_wallet_atn_descriptor_redirects TESTFLAG
- cli_descriptor_concrete.rs::export_wallet_originless_concrete_still_accepted TESTFLAG
- cli_descriptor_mode.rs::descriptor_verify_bundle_detects_tampered_mk1 MAINNET
- cli_export_wallet_bsms.rs::bsms_4line_emit_2of3_wsh_multi_testnet TESTFLAG
- cli_export_wallet_coldcard.rs::cell_1_coldcard_generic_bip84_mainnet_byte_exact MAINNET
- cli_export_wallet_from_import_json.rs::p11a_helper_propagates_import_wallet_failure NOFLAG
- cli_import_json_envelope_consistency.rs::envc_bundle_refuses_edited_origin_path NOFLAG+TESTFLAG
- cli_import_json_envelope_consistency.rs::envelope_mk1_origin_path_only_mismatch_refused NOFLAG+TESTFLAG
- cli_import_json_envelope_consistency.rs::envc_template_md1_envelope_refused TESTFLAG
- cli_import_wallet_bitcoin_core.rs::core_xprv_rejected_exit_2 NOFLAG
- cli_import_wallet_bitcoin_core.rs::phase3_c1_fold_core_tprv_rejected_exit_2 NOFLAG
- cli_import_wallet_bitcoin_core.rs::core_empty_descriptors_array_exit_2 NOFLAG
- cli_import_wallet_bitcoin_core.rs::core_testnet_tpub_network_detected NOFLAG
- cli_import_wallet_roundtrip.rs::fixture_bsms_testnet_2of2_parses_clean NOFLAG
- cli_import_wallet_snet_network_mismatch.rs::descriptor_tpub_on_coin_type_0_rejects NOFLAG
- cli_import_wallet_snet_network_mismatch.rs::descriptor_xpub_on_coin_type_1_rejects NOFLAG
- cli_import_wallet_snet_network_mismatch.rs::descriptor_consistent_testnet_imports NOFLAG
- cli_import_wallet_snet_network_mismatch.rs::specter_xpub_on_coin_type_1_rejects NOFLAG
- cli_import_wallet_snet_network_mismatch.rs::bitcoin_core_tpub_on_coin_type_0_rejects NOFLAG
- cli_import_wallet_snet_network_mismatch.rs::bsms_tpub_on_coin_type_0_rejects NOFLAG
- cli_import_wallet_snet_network_mismatch.rs::bsms_consistent_testnet_imports NOFLAG
- cli_multipath_duplicate_refusal.rs::bundle_template_duplicate_multipath_refused NOFLAG
- cli_multipath_duplicate_refusal.rs::bundle_multisig_second_key_duplicate_multipath_refused NOFLAG
- cli_network_fail_open.rs::e1a_testnet_tpub_to_address_mainnet_rejects MAINNET
- cli_network_fail_open.rs::e1b_testnet_tpub_to_address_testnet_agrees_ok TESTFLAG
- cli_network_fail_open.rs::e1c_testnet_tpub_to_address_no_network_infers_ok NOFLAG
- cli_network_fail_open.rs::e1_minor_signet_override_accepts_testnet_tpub TESTFLAG
- cli_network_fail_open.rs::e2a_testnet_tpub_mainnet_network_rejects MAINNET
- cli_network_fail_open.rs::e2b_testnet_tpub_testnet_network_agrees_ok TESTFLAG
- cli_network_fail_open.rs::e2_minor_mk1_target_testnet_mainnet_network_rejects MAINNET
- cli_network_fail_open.rs::e3a_testnet_tprv_mainnet_network_rejects MAINNET
- cli_network_fail_open.rs::e3b_testnet_tprv_testnet_network_agrees_ok TESTFLAG
- cli_network_fail_open.rs::e4b_template_slot_testnet_xpub_explicit_testnet_network_ok TESTFLAG
- cli_network_fail_open.rs::e5a_descriptor_testnet_tpub_default_mainnet_bsms_4line_rejects NOFLAG
- cli_network_fail_open.rs::e5b_descriptor_testnet_tpub_explicit_testnet_bsms_4line_ok TESTFLAG
- cli_network_fail_open.rs::e5c_descriptor_testnet_tpub_default_mainnet_bsms_2line_ok NOFLAG
- cli_network_fail_open.rs::e5e_multipath_testnet_tpub_default_mainnet_bsms_4line_rejects NOFLAG
- cli_network_fail_open.rs::e5e_multipath_testnet_tpub_explicit_testnet_bsms_4line_ok TESTFLAG
- cli_older_advisory.rs::fires_and_non_blocking_bundle_concrete_key TESTFLAG
- cli_older_advisory.rs::clean_key_no_advisory_bundle_concrete_key TESTFLAG
- cli_output_class.rs::export_wallet_emits_watch_only TESTFLAG
- cli_output_class.rs::bundle_descriptor_emits_watch_only TESTFLAG
- cli_restore.rs::restore_emits_no_private_key_material NOFLAG
- cli_restore.rs::restore_format_path_emits_no_private_key_material NOFLAG
- cli_restore.rs::restore_json_no_seed_material_redacted_non_vacuous NOFLAG
- cli_restore.rs::restore_json_and_output_paths_emit_no_private_key_material NOFLAG
- cli_restore_md1_template_cycle13b.rs::signet_all_own_multisig_template_completes_to_golden TESTFLAG
- cli_restore_multisig.rs::md1_watch_only_no_private_material MAINNET
- cli_restore_multisig.rs::md1_testnet_emits_tpub TESTFLAG
- cli_restore_multisig_format.rs::format_payloads_are_watch_only NOFLAG+MAINNET
- cli_restore_multisig_general.rs::general_multipath_0_1_reconstructs_with_multixpub_arm NOFLAG
- cli_restore_multisig_general.rs::general_testnet_network_corrected NOFLAG+TESTFLAG
- cli_snet_build_descriptor_network_warn.rs::l1_tpub_with_network_mainnet_warns_not_rejects NOFLAG+MAINNET+TESTFLAG
- cli_snet_build_descriptor_network_warn.rs::l1_tpub_network_omitted_infers_testnet_preview NOFLAG
- cli_snet_build_descriptor_network_warn.rs::l1_tpub_with_network_testnet_no_warn TESTFLAG
- cli_snet_convert_export_network_mismatch.rs::m14_tpub_reemit_into_mainnet_prefix_rejects MAINNET
- cli_snet_convert_export_network_mismatch.rs::l11_testnet_wif_to_xpub_testnet_ok TESTFLAG
- cli_wallet_cross_format_convergence.rs::concrete_vs_atn_descriptor_converge_md1_mk1 TESTFLAG
- cli_wallet_cross_format_convergence.rs::concrete_duplicate_cosigner_rejected_bip388 TESTFLAG
- cli_xpub_search_address_of_xpub.rs::network_inference_from_tpub_prefix NOFLAG
- cli_xpub_search_address_of_xpub.rs::network_signet_override_disambiguates_testnet_version_byte TESTFLAG

## Helper functions holding test-network material (not tests)

R1-M6: these were listed as tests. Each is named with the tests that call it; a red caller
counts as on the list.

- cli_bip388_validity.rs::ka: called by the 15 `bip388v_*` tests of the file that build descriptors from it (export-wallet descriptor refusals and guard emits)
- cli_bip388_validity.rs::kb: called by bip388v_export_descriptor_nums_literal_before_multipath, bip388v_export_descriptor_nums_refused, bip388v_guard_distinct_keys_emit, bip388v_export_descriptor_extra_step_refused, bip388v_export_descriptor_policy_json_extra_step_refused, bip388v_guard_extra_step_alternatives_emit
- cli_bip388_validity.rs::ka_other_origin: called by bip388v_export_descriptor_same_xpub_other_origin_refused
- cli_bundle_slip0132_info.rs::info_line: called by bundle_descriptor_watch_only_zpub_emits_info_line_then_card_no_warning, bundle_multisig_full_zpub_cosigner_emits_info_then_card_then_warning, bundle_multisig_two_normalized_slots_emit_info_lines_in_slot_order, bundle_json_watch_only_zpub_emits_info_line_no_warning, bundle_no_engraving_card_zpub_emits_info_line_only
- cli_convert_slip0132_info.rs::info_line: called by the 12 `convert_*` tests of the file (zpub/ypub/upub/vpub info lines)
- cli_export_wallet_from_import_json.rs::extract_xpubs: called by p11b_happy_path_matrix_all_sources_all_descriptor_capable_dests
- cli_multipath_duplicate_refusal.rs::single: called by bundle_concrete_duplicate_multipath_refused, import_wallet_descriptor_duplicate_multipath_refused, verify_bundle_duplicate_multipath_refused, bundle_distinct_multipath_still_accepted
- cli_network_fail_open.rs::assert_mismatch: called by e1a_testnet_tpub_to_address_mainnet_rejects, e2a_testnet_tpub_mainnet_network_rejects, e2_minor_mk1_target_testnet_mainnet_network_rejects, e3a_testnet_tprv_mainnet_network_rejects, e4a_template_slot_testnet_xpub_default_mainnet_network_rejects, e4_minor_master_xpub_slot_testnet_default_mainnet_network_rejects, e5a_descriptor_testnet_tpub_default_mainnet_bsms_4line_rejects, e5e_multipath_testnet_tpub_default_mainnet_bsms_4line_rejects
- cli_network_fail_open.rs::e5_single_branch_desc: called by e4d_descriptor_concrete_passthrough_ok, e5a_descriptor_testnet_tpub_default_mainnet_bsms_4line_rejects, e5b_descriptor_testnet_tpub_explicit_testnet_bsms_4line_ok, e5c_descriptor_testnet_tpub_default_mainnet_bsms_2line_ok
- cli_network_fail_open.rs::e5_multipath_desc: called by e5e_multipath_testnet_tpub_default_mainnet_bsms_4line_rejects, e5e_multipath_testnet_tpub_explicit_testnet_bsms_4line_ok
- prop_backup_restore_roundtrip.rs::is_base58: called by normalize (below)
- prop_backup_restore_roundtrip.rs::normalize: called by backup_restore_roundtrip, tr_taproot_roundtrip, tr_taproot_smoke_both_variants, oracle1_rejects_dropped_timelock, oracle1_rejects_multi_sortedmulti_swap, oracle1_rejects_masked_timelock_value, oracle1_accepts_keyless_equivalent_redepth, smoke_handpicked_policies

## Mainnet key with a test --network

- cli_bundle_h7_prefix_origin.rs::verify_bundle_carries_prefix_origin_through_reparse MAINKEY+TESTFLAG(regtest)
- cli_convert_address.rs::xpub_to_address_p2pkh_explicit_script_type_v0_26_0 MAINKEY+TESTFLAG(testnet)
- cli_non_canonical_descriptor.rs::non_canonical_default_path_consumes_account_arg MAINKEY+TESTFLAG(testnet)
- cli_output_class.rs::export_wallet_emits_watch_only MAINKEY+TESTFLAG(testnet)
- cli_output_class.rs::bundle_descriptor_emits_watch_only MAINKEY+TESTFLAG(testnet)

## Unit tests in src/ that pin today's network refusals (count of matching lines)

- crates/mnemonic-toolkit/src/network.rs 4
- crates/mnemonic-toolkit/src/error.rs 1

## Added at unit 3 (run, not grep; each explained by a spec row)

Spec B R3.9 / Spec A §4.1 "import-wallet, blob class against run" (a test blob
with no flag is a mainnet run; the parse tests now pass `--network testnet`,
and the ones marked * also pin the no-flag refusal, exit 8):
- bughunt_s2.rs::bughunt_tk_45_import_json_testnet_envelope_not_labelled_mainnet NOFLAG *
- cli_import_wallet_bitcoin_core.rs::core_testnet_tpub_network_detected NOFLAG *
- cli_import_wallet_bsms.rs::bsms_2_line_happy_path NOFLAG *
- cli_import_wallet_bsms.rs::bsms_2_line_warning_emitted NOFLAG
- cli_import_wallet_bsms.rs::bsms_2line_decay_4032_fixture_parses NOFLAG
- cli_import_wallet_bsms.rs::bsms_decaying_multisig_n_144 NOFLAG
- cli_import_wallet_bsms.rs::bsms_decaying_multisig_n_4032 NOFLAG
- cli_import_wallet_bsms.rs::bsms_decaying_multisig_n_32768 NOFLAG
- cli_import_wallet_bsms.rs::bsms_sniff_smoke_true_and_false NOFLAG
- cli_import_wallet_coldcard.rs::coldcard_singlesig_bip84_xtn_testnet_parses_clean NOFLAG * (TX-H4 literal)
- cli_import_wallet_coldcard.rs::coldcard_json_envelope_xtn_carries_chain_xtn NOFLAG
- cli_import_wallet_roundtrip.rs::fixture_bsms_2line_decay_144_parses_clean NOFLAG
- cli_import_wallet_roundtrip.rs::fixture_core_testnet_bip84_parses_clean NOFLAG
- cli_import_wallet_sniff.rs::sniff_path_roundtrip NOFLAG
- cli_import_wallet_network_override.rs::testnet_blob_default_network_is_testnet, testnet_blob_override_to_mainnet_refused, mainnet_blob_override_to_signet_refused, mixed_mainnet_testnet_blob_override_mainnet_refused_per_entry: exit 1 -> 8, TX-H1 literals

Spec A §4.1 "import-wallet, every parser: key version against the blob's coin
type" (TX-S key form, exit 8; the coin-type inference refusals are deleted,
plan §3 unit 3 table):
- cli_import_wallet_snet_network_mismatch.rs: the 9 `*_rejects` tests exit 2 -> 8 with the TX-S line; the 2 consistent-testnet controls pass `--network testnet`
- cli_import_wallet_bsms.rs::bsms_mixed_coin_types_rejected: exit 2 -> 8, TX-H2 + TX-H1 (short) literals
- bughunt_s3.rs::bughunt_tk_19b_coldcard_multisig_bip45_mixed_paths_refused and s2_red.rs::s2_red_19c_single_component_non_45_path_refused: the path-length refusal is gone (exit 2 -> 0, imported, no network line)
- src unit tests: bsms `parse_4line_first_address_mismatch_emits_warning` (calls `first_address_audit`), coldcard_multisig `parse_heterogeneous_coin_type_rejected` -> `_parses`, `parse_testnet_path_sets_network_testnet` -> `parse_testnet_path_keeps_path_network_placeholder`, sparrow/specter `parse_testnet_network_inferred_from_coin_type_one` -> `parse_testnet_coin_type_one_no_network_inferred`

## Changed at unit 8a (run; each explained by a spec row)

- On the list above, changed to the spec's exit and text: cli_network_fail_open.rs
  e1a, e2a, e2_minor (exit 2 -> 8, TX-H1 / TX-H3 literals); e1c renamed
  `e1c_testnet_tpub_to_address_no_network_refused` (exit 0 -> 8, TX-H1, D1);
  cli_snet_convert_export_network_mismatch.rs m14 and l11 (exit 2 -> 8, TX-H1 /
  TX-H5 literal); cli_xpub_search_address_of_xpub.rs::network_inference_from_tpub_prefix
  (passes `--network testnet`, and pins the no-flag refusal); cli_convert_convergence.rs
  c4 and the cli_convert_slip0132*.rs NOFLAG tests (pass `--network testnet`);
  cli_output_class.rs::bundle_descriptor_emits_watch_only (origin coin 1h: a tpub at
  coin 0h is TX-S); cli_bundle_h7_prefix_origin.rs (run on mainnet: its descriptor's
  origin is coin 0h); bughunt_s2.rs tk_45 (bundle --import-json, exit 1 -> 8, TX-H4
  literals).
- Not on the list, explained by R8a.18/R8a.19 (a main xprv on a test run, a tprv with
  no flag): cli_derive_child.rs::hd_seed_wif_testnet_prefix and xprv_child_testnet_prefix
  (input is now the tprv of the same master), bughunt_1119_unbounded_args.rs bh_tk_mn7_*
  (pass `--network testnet`; the long-child leg ends with TX-R, N12).
- Not on the list, explained by N12/T6: stdout_write_errors.rs::r4_bundle_dev_full_exits_1
  (TX-R moved before the write-error line; the test is unchanged).
