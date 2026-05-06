# v0.5.1 Phase Atomic — code-reviewer r1

**Outcome:** 0C/0I/0L/2N — APPROVED.

## Scope reviewed
20 staged paths in Commit 1 (atomic deletion bundle):
- 3 source files modified: `crates/mnemonic-toolkit/src/cmd/bundle.rs`, `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`, `crates/mnemonic-toolkit/src/slot_input.rs`.
- 3 test files deleted: `cli_mode_violations.rs`, `cli_mode_violations_v0_2.rs`, `cli_mode_violations_v0_3.rs`.
- 1 new test file: `cli_mode_violations_v0_5.rs`.
- 13 test files modified for legacy-flag → `--slot` rewrites.

`Cargo.lock` deliberately left unstaged.

## Plan-fidelity verification

All 12 plan items confirmed faithfully implemented:

1. `BundleArgs`: 6 legacy fields removed; `slot` field retained.
2. `VerifyBundleArgs`: 6 legacy fields removed; `slot: Vec<SlotInput>` added with matching clap attribute.
3. `bundle::resolve_slots`: refactored to explicit args-tuple `(template, network, account, language, passphrase)`; promoted to `pub(crate)`; called from both `bundle.rs` and `verify_bundle.rs`.
4. `bundle::bundle_args_to_slots`: deleted entirely (verified via grep).
5. `slot_input::expand_legacy_to_slots`: deleted; 5 unit tests deleted; file-level `#![allow(dead_code)]` comment updated.
6. `bundle::mode_text`: 9 + 2 deleted consts removed (PASSPHRASE_WITH_XPUB, LANGUAGE_WITH_XPUB, XPUB_NEEDS_FINGERPRINT, FINGERPRINT_WITHOUT_XPUB, XPUB_STDIN, XPUB_AND_COSIGNER, COSIGNER_AND_COSIGNERS_FILE, COSIGNER_COUNT_WITHOUT_MULTISIG, PRIVACY_WITH_XPUB, ACCOUNT_INCOMPATIBLE_TEMPLATE, DESCRIPTOR_WITH_COSIGNER_COUNT). 7 retained: THRESHOLD_WITHOUT_MULTISIG, PATH_FAMILY_WITHOUT_MULTISIG, DESCRIPTOR_AND_TEMPLATE, DESCRIPTOR_AND_DESCRIPTOR_FILE, DESCRIPTOR_WITH_THRESHOLD, DESCRIPTOR_WITH_PATH_FAMILY, DESCRIPTOR_WITH_NONZERO_ACCOUNT.
7. `bundle::run`: pre-check ladder reduced; only descriptor-mode + retained template-mode guards remain.
8. `verify_bundle.rs`: `run()` reshaped to dispatch via slot detection; `run_full`/`run_watch_only`/`run_multisig` consume slots through `bundle::resolve_slots`; `descriptor_mode_verify_run` rebuilt; `watch_only_emits_spec_2_2_2_warning_to_stderr` test deleted; `load_bundle_json_into_args` constructor unaffected (struct-update syntax).
9. `cli_mode_violations_v0_5.rs`: 6 tests; expected stderr strings inlined byte-exactly.
10. A.2 test rewrites: all 13 files clean of legacy flags.
11. `cli_unified_slot.rs`: row-6 collision test deleted; `TREZOR_BIP84_XPUB` const deleted; `TREZOR_FP_HEX` retained.
12. `cli_bip388_distinctness.rs::bundle_multisig_full_legacy_cosigner_count_inconsistent_emits_row5` deleted (row-5 trap unreachable post-deletion of `--cosigner-count`).

## Path-defaulting cross-check

Reviewer separately verified the in-flight behavioral change in `bundle::resolve_slots` (Xpub branch path defaulting from empty → `template.derivation_path(network, account)`) is sound:

- Motivated by `cli_verify_bundle_forensics::watch_only_short_circuit_emits_decode_error` PathTooDeep panic in synthesis with depth-3 xpub + empty path.
- Doesn't break `cli_unified_slot::unified_slot_xpub_alone_emits_partial_origin` — that test asserts mode/ms1 only, not `origin_path`.

## Nits folded inline

- `cli_bundle_watch_only.rs:4` — module doc comment updated to reference `--slot` invocation form.
- `cli_unified_slot.rs:298` — inline comment for `unified_slot_xpub_with_fingerprint_no_path` updated to reflect v0.5.1 template-path defaulting.

## Test results

230 lib + 44 integration tests pass. 2 ignored (pre-existing).
