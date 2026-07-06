# Cycle A — test-migration blast-radius sweep (I-1 + M-9(i) discharge)

Sweep of `grep -rn '/0/\*\|/1/\*' crates/mnemonic-toolkit/{src,tests}` @ `origin/master 8c8b9183` (87 raw hits, all read in context). Reject fires on the SUBSTITUTED `@N` form after `concrete_keys_to_placeholders`.

## Load-bearing pipeline fact
`lex_placeholders` is called from ONE place: `parse_descriptor` (`parse_descriptor.rs:852`), reached only via (a) direct `@N`-template input (`bundle --descriptor <TEMPLATE>`, `cmd/bundle.rs:1389`) or (b) `concrete_keys_to_placeholders` (`pipeline.rs:330`, gated on a MANDATORY `[fp/path]xpub` bracket) used by `bundle --descriptor <CONCRETE>`, `import-wallet` (all vendor parsers), `verify-bundle`. **`export-wallet --descriptor` and `compare-cost --descriptor` parse via `miniscript::Descriptor::from_str` directly and NEVER touch `lex_placeholders`** — the basis of most STAYS-PASSING entries.

## Migration strategy — Group A vs Group B (plan-critical)
- **Group A** — tests whose PURPOSE is the collapse behavior (encode `/0/*` as "correct"/"converges"): `cli_cross_start_convergence.rs::a4/a5`. → **MUST assert the reject** (they tested the wrong thing). Swapping to `<0;1>` here WOULD delete the regression (no-weakening rule applies).
- **Group B** — tests of an UNRELATED feature that incidentally used a `/0/*` fixture (select-descriptor, network-override, OOB-index, masked-`older()` advisory, multi-emit). → **swap the fixture to `<0;1>/*` to PRESERVE feature coverage** (NOT weakening: the collapse regression is covered by the NEW dedicated reject tests). Forcing assert-reject here would DELETE coverage of those features.
- **Refined no-weakening principle:** every `/0/*` SHAPE must be covered by a dedicated reject test (new). A feature-test may swap its incidental `/0/*` fixture to `<0;1>` ONLY because that shape's reject is covered elsewhere. Never swap a Group-A collapse-premised test.

## REJECTS-NOW (22 test fns/fixtures)
**Funds-critical (assert `.success()` / `code(Some(0))` on `/0/*` today):**
- `cli_import_wallet_bitcoin_core.rs::core_masked_older_emits_advisory` (:993, `.success()`), `core_clean_older_emits_no_advisory` (:1010) — Group B (advisory) → swap fixture to `<0;1>`.
- `cli_cross_start_convergence.rs::a4_...converge` (:256), `a5_all_four_starts_converge_singlesig` (:276) — **Group A** → assert reject. (`bundle --descriptor "wpkh(@0[{fp}/84'/0'/0']/0/*)"` direct-placeholder + wallet-file side both reject.)
- `cli_older_advisory.rs::fires_and_non_blocking_bundle_concrete_key` (:166), `clean_key_no_advisory_bundle_concrete_key` (:206) — Group B (advisory) → swap to `<0;1>`.
- `cli_import_wallet_network_override.rs::homogeneous_two_mainnet_blob_override_mainnet_ok` (:116) — Group B (network-override + over-rejection control); HIGHEST-SIGNAL → swap fixture `core-two-mainnet.json` to a `<0;1>` blob.
- `cli_import_wallet_envelope_v0_27_0.rs::bitcoin_core_multi_descriptor_yields_one_envelope_per_entry` (:205, `.success()` in shared helper) — Group B (multi-envelope) → swap `core-multi-bip84.json`.
**ASSERT-REJECT / keep fixture (plan-R0 I-C correction — NOT Group B):** `core_fixture_file_mainnet_receive_change_pair_parses` (:897) uses `core-mainnet-receive-change-pair.json` = a raw same-key legacy Core `/0/*`+`/1/*` split → under Part 1 (no merge) BOTH entries reject → flip `bundles=2 .success()` to a reject assertion (exit≠0 + bitcoin-core workaround message). **Keep `core-mainnet-receive-change-pair.json` UNCHANGED** — it is the canonical legacy-split funds regression AND the future INPUT fixture for the `bitcoin-core-receive-change-pair-merge` follow-up. Do NOT swap it to `<0;1>`.

**Other bitcoin-core cells (Group B, swap to `<0;1>`):** `core_multi_descriptor_emit_all` (:183), `core_select_descriptor_by_index` (:220), `core_select_descriptor_active_receive` (:259), `core_select_descriptor_active_change` (:298), `core_multisig_wsh_sortedmulti_2_of_3` (:337), `core_testnet_tpub_network_detected` (:538), `core_fixture_file_multi_bip84_all` (:585); cross-file fixture consumers `cli_import_wallet_roundtrip.rs::fixture_core_multi_bip84_emit_four_bundles` (:265) + `fixture_core_bip49_mainnet_two_entries_emit_two_bundles` (:235). NOTE (plan-R0 M-a): several of these build descriptors INLINE (`build_core_multi`/`build_core_single` `d0=…/0/*`) — swap the in-body LITERALS, not a `.json` file.
**`cli_output_class.rs::bundle_descriptor_emits_watch_only` (:292)** — Group B → swap to `<0;1>`.
**⚠ Special (test-intent-invalidated, needs fixture swap, not just assertion change):** `cli_import_wallet_bitcoin_core.rs::core_select_index_out_of_range_errors` (:713) reuses `core-multi-bip84.json`; entry 0 residue-rejects BEFORE the `--select 99` OOB logic → stderr `"99"`/`"range"` assertion becomes vacuous. Swap to a multipath ≥2-entry fixture.

## STAYS-PASSING (19 groups — anti-over-rejection controls, do NOT touch)
- `core_fixture_file_multipath_receive_change_pair_parses` (bitcoin_core.rs:914-925) — already `<0;1>/*`, distinct FP_A(wpkh)/FP_B(sh(wpkh)) keys. Best boundary control + future merge-negative-control.
- `cli_export_wallet.rs::descriptor_to_bip388_non_multipath_refused` (:984) + `cli_descriptor_concrete.rs::export_wallet_originless_concrete_still_accepted` (:174) — `export-wallet --descriptor` stays on miniscript-direct path (must never regress into the `@N` lexer). Good discriminators.
- `cli_compare_cost.rs::descriptor_wsh_wildcard_...` (:988) — `compare-cost --descriptor` → `Descriptor::from_str` direct.
- `cli_import_wallet_sniff.rs::sniff_ambiguous_with_specter_markers` (:79) — vendor-marker sniff-refusal fires before any parse.
- `src/parse_descriptor.rs::lex_rejects_no_placeholders` (:1598) — no `@` → pre-existing empty-placeholder error path, untouched.
- BSMS line-3 `/0/*,/1/*` free-text path-restrictions (bsms.rs:559,582,610; export bsms.rs:132-161; cli_export_wallet_bsms.rs:96,146,237; 4line fixtures) — audit/output text, NEVER re-parsed (actual descriptors are `<0;1>/*`).
- `roundtrip.rs` canonicalize tests (:952-1004) — `MsDescriptor::from_str` direct.
- Various sniff/classify/strip-comment tests using dummy non-base58 xpubs (specter.rs:625, coldcard.rs:728, descriptor.rs:269,300, pipeline.rs:539,598).
- `cli_export_wallet.rs::cell_1_bitcoin_core_single_sig_wpkh_round_trip` (:120,132) — export EMISSION of `/0/*`+`/1/*`, never re-consumed.

## Counts: REJECTS-NOW 22 · STAYS-PASSING 19 · UNRELATED 14 raw lines. 87/87 accounted. ZERO ambiguous.
