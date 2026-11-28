//! The integration tests of `mnemonic-toolkit`, built as ONE test binary (`all`).
//!
//! Each file under `tests/` used to be its own binary (about 250 of them); compiling
//! and linking those dominated the test build. The files stay where they are and are
//! declared here as modules, so `file!()`, `include_str!` paths, proptest regression
//! files and fixture paths are unchanged. A test's name gains its file as a module
//! prefix: `<file>::<test>`.
//!
//! A NEW test file must be added below; `every_test_file_is_listed` fails otherwise.
//!
//! Kept as separate binaries (Cargo.toml `[[test]]`), because CI runs each by name:
//! `cli_slip39_refusals` (the shipped build without `slip39-test-rng`),
//! `cli_cross_tool_differential`, `bitcoind_differential`, `bitcoind_addr_import`
//! (their own workflows, `--ignored`) and `mlock_g6_invariant` (`--include-ignored`).
//!
//! Process-global state: nextest runs every test in its own process, so tests do
//! not share a process under the gate. No test file sets an environment variable,
//! changes the working directory or calls `std::process::exit`; the one
//! process-wide static (`cli_auto_repair::TMP_SEQ`) is an atomic counter.

mod bch_residue_separation;
mod bip32_vectors;
mod bughunt_1119_unbounded_args;
mod bughunt_s1;
mod bughunt_s2;
mod bughunt_s3;
mod bughunt_s4;
mod bughunt_s5;
mod cli_addresses;
mod cli_addresses_electrum;
mod cli_argv_leakage;
mod cli_argv_secrets_f688;
mod cli_argv_shapeless_echo;
mod cli_argv_unknown_node_echo;
mod cli_auto_repair;
mod cli_bip388_distinctness;
mod cli_bip388_double_star_shorthand;
mod cli_bip388_policy_intake;
mod cli_bip388_validity;
mod cli_bsms_round1;
mod cli_build_descriptor;
mod cli_bundle_full;
mod cli_bundle_h12_taproot_origin;
mod cli_bundle_h7_prefix_origin;
mod cli_bundle_import_json;
mod cli_bundle_json_intake;
mod cli_bundle_json_threshold;
mod cli_bundle_keyless_descriptor;
mod cli_bundle_language_advisory;
mod cli_bundle_md1_template_form;
mod cli_bundle_md1_template_multisig;
mod cli_bundle_multisig;
mod cli_bundle_origin_path_canon;
mod cli_bundle_recovery_threshold_text;
mod cli_bundle_seedqr_slot;
mod cli_bundle_slip0132_info;
mod cli_bundle_watch_only;
mod cli_check_pkk_canonical_golden;
mod cli_compare_cost;
mod cli_compare_cost_v5_fix;
mod cli_compare_cost_wrapper_chain;
mod cli_compare_cost_wsh_witness_script;
mod cli_concrete_nonranged_xpub_reject;
mod cli_convert_account;
mod cli_convert_address;
mod cli_convert_bip38;
mod cli_convert_bip39_vectors;
mod cli_convert_convergence;
mod cli_convert_electrum;
mod cli_convert_happy_paths;
mod cli_convert_help_fixtures;
mod cli_convert_json;
mod cli_convert_language_advisory;
mod cli_convert_minikey;
mod cli_convert_mk1_slip0132_hint;
mod cli_convert_path;
mod cli_convert_refusals;
mod cli_convert_round_trips;
mod cli_convert_slip0132;
mod cli_convert_slip0132_info;
mod cli_cross_start_convergence;
#[allow(non_snake_case)]
mod cli_cycleA_phase2_funds_proof;
mod cli_decode_address;
mod cli_derive_child;
mod cli_descriptor_concrete;
mod cli_descriptor_mode;
mod cli_electrum_decrypt;
mod cli_env_dash_literal;
mod cli_env_var_sentinel;
mod cli_export_wallet;
mod cli_export_wallet_allow;
mod cli_export_wallet_bitcoin_core_addresses;
mod cli_export_wallet_bsms;
mod cli_export_wallet_coldcard;
mod cli_export_wallet_descriptor;
mod cli_export_wallet_electrum;
mod cli_export_wallet_from_import_json;
mod cli_export_wallet_green;
mod cli_export_wallet_jade;
mod cli_export_wallet_older_advisory;
mod cli_export_wallet_seedqr_slot;
mod cli_export_wallet_sparrow;
mod cli_export_wallet_specter;
mod cli_export_wallet_universal_name_lift;
mod cli_export_wallet_unsorted_multi_refusal;
mod cli_f687_passphrase_channels;
mod cli_final_word_advisories;
mod cli_final_word_happy_paths;
mod cli_final_word_json;
mod cli_final_word_refusals;
mod cli_final_word_stdin;
mod cli_funds_safety_advisory;
mod cli_gen_man;
mod cli_gui_schema;
mod cli_gui_schema_classify_descriptor;
mod cli_gui_schema_conditional_rules;
mod cli_gui_schema_v3_extensions;
mod cli_gui_schema_v4_extensions;
mod cli_gui_schema_v5_extensions;
mod cli_help_fixtures;
mod cli_hrp_case_insensitive;
mod cli_import_json_envelope_consistency;
mod cli_import_wallet_account_preserve;
mod cli_import_wallet_bitcoin_core;
mod cli_import_wallet_bsms;
mod cli_import_wallet_bsms_encrypted;
mod cli_import_wallet_coldcard;
mod cli_import_wallet_coldcard_multisig;
mod cli_import_wallet_descriptor;
mod cli_import_wallet_electrum;
mod cli_import_wallet_electrum_bie1;
mod cli_import_wallet_electrum_null_origin;
mod cli_import_wallet_envelope_v0_27_0;
mod cli_import_wallet_format_mismatch_matrix;
mod cli_import_wallet_jade;
mod cli_import_wallet_network_override;
mod cli_import_wallet_p0c_dispatch;
mod cli_import_wallet_roundtrip;
mod cli_import_wallet_seed_overlay;
mod cli_import_wallet_snet_network_mismatch;
mod cli_import_wallet_sniff;
mod cli_import_wallet_sparrow;
mod cli_import_wallet_sparrow_taproot;
mod cli_import_wallet_specter;
mod cli_indel;
mod cli_inspect;
mod cli_inspect_partial;
mod cli_intake_grouped;
mod cli_invalidchecksum_redaction;
mod cli_json_envelopes;
mod cli_liana_template_refusal;
mod cli_md1_length_cap;
mod cli_mk1_repair_reverify;
mod cli_mnem_emit_preserve;
mod cli_mnem_per_card_language;
mod cli_mode_violations_v0_5;
mod cli_ms1_repair_demote;
mod cli_ms1_slot;
mod cli_ms_shares;
mod cli_ms_shares_consume;
mod cli_ms_shares_inconsistent;
mod cli_multipath_duplicate_refusal;
mod cli_network_fail_open;
mod cli_network_rule_export_wallet;
mod cli_network_rule_vectors;
mod cli_non_canonical_descriptor;
mod cli_nostr;
mod cli_older_advisory;
mod cli_output_class;
mod cli_p3_argv_refusal;
mod cli_p3_grouping_surface;
mod cli_positional_hrp_autodetect;
mod cli_printed_text_drift;
mod cli_repair;
mod cli_repair_dead_card_strict;
mod cli_repair_md1_non_chunked;
mod cli_repair_unsupported_version;
mod cli_restore;
mod cli_restore_md1_template;
mod cli_restore_md1_template_cycle13b;
mod cli_restore_md1_template_multisig;
mod cli_restore_multisig;
mod cli_restore_multisig_format;
mod cli_restore_multisig_general;
mod cli_restore_taproot;
mod cli_secret_in_argv_warning;
mod cli_seed_xor_advisories;
mod cli_seed_xor_happy_paths;
mod cli_seed_xor_json;
mod cli_seed_xor_refusals;
mod cli_seed_xor_stdin;
mod cli_seedqr;
mod cli_seedqr_from_unification;
mod cli_seedqr_json_out_mode;
mod cli_seedqr_json_out_replace;
mod cli_self_check;
mod cli_shwpkh_canonical_flip;
mod cli_silent_payment;
#[cfg(feature = "slip39-test-rng")]
mod cli_slip39_advisories;
mod cli_slip39_happy_paths;
#[cfg(feature = "slip39-test-rng")]
mod cli_slip39_json;
mod cli_slip39_language_advisory;
mod cli_slip39_stdin;
mod cli_slot_path_reaches_the_card;
mod cli_snet_build_descriptor_network_warn;
mod cli_snet_coldcard_account_bound;
mod cli_snet_convert_export_network_mismatch;
mod cli_standalone_bijections;
mod cli_tr_bip48_advisory;
mod cli_unified_slot;
mod cli_unrestorable_shape_advisory;
mod cli_verify_bundle_entropy_slot;
mod cli_verify_bundle_exit_tier;
mod cli_verify_bundle_forensics;
mod cli_verify_bundle_full;
mod cli_verify_bundle_hashlock_and_bip388;
mod cli_verify_bundle_md1_template;
mod cli_verify_bundle_md1_template_multisig;
mod cli_verify_bundle_multi_cosigner_mk1;
mod cli_verify_bundle_partial;
mod cli_verify_bundle_seedqr_slot;
mod cli_verify_bundle_watch_only;
mod cli_verify_md1_origin_match;
mod cli_verify_message;
mod cli_wallet_cross_format_convergence;
mod cli_word_card;
mod cli_xpub_search_account_of_descriptor;
mod cli_xpub_search_address_of_xpub;
mod cli_xpub_search_drift_v0_27_0;
mod cli_xpub_search_env_var_sentinel;
mod cli_xpub_search_older_advisory;
mod cli_xpub_search_passphrase_candidates;
mod cli_xpub_search_passphrase_of_xpub;
mod cli_xpub_search_path_of_xpub;
mod design_artifacts_presence;
mod display_grouping_conformance;
mod lib_final_word;
mod lib_seed_xor;
mod lib_slip39_errors;
mod lib_slip39_feistel;
mod lib_slip39_math;
mod lib_slip39_roundtrip;
mod lib_slip39_rs1024;
mod lib_slip39_share;
mod lib_slip39_vectors;
mod lib_slip39_wordlist;
mod lint_argv_secret_flags;
mod lint_safety_first_party_mlock;
mod lint_safety_third_party_blocked;
mod lint_world_readable_helper;
mod lint_zeroize_discipline;
mod mlock_unit;
mod network_rule_support;
mod p3_declined_crate_items;
mod p3_io_lib_pin;
mod prop_backup_restore_roundtrip;
mod prop_subset_search_roundtrip;
mod prop_template_completion_roundtrip;
mod readme_dice_kat_parity;
mod readme_version_current;
mod s2_red;
mod stdout_write_errors;

/// Every `tests/*.rs` file is either declared above, or is one of the separate
/// `[[test]]` binaries named in the header.
#[test]
fn every_test_file_is_listed() {
    const SEPARATE: &[&str] = &[
        "cli_slip39_refusals",
        "cli_cross_tool_differential",
        "bitcoind_differential",
        "bitcoind_addr_import",
        "mlock_g6_invariant",
    ];
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let all_rs = std::fs::read_to_string(dir.join("all.rs")).expect("tests/all.rs is readable");
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("tests/ is readable") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("utf-8 name");
        if stem == "all" || SEPARATE.contains(&stem) {
            continue;
        }
        if !all_rs.lines().any(|l| l == format!("mod {stem};")) {
            missing.push(stem.to_string());
        }
    }
    assert!(
        missing.is_empty(),
        "test files not listed in tests/all.rs: {missing:?}"
    );
}
