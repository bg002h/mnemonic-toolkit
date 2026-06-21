# Cycle-5 review — LENS C (test-integrity / fixture-validity / build / differential-oracle gate)

- **Cycle:** cycle-5 "S-NET" network-provenance invariant (`feature/cycle5-snet-network-invariant`)
- **HEAD:** `79028490` (4 commits) · **Base:** `origin/master` `ac4eead0`
- **Worktree:** `/scratch/code/shibboleth/wt-cycle5`
- **Date:** 2026-06-21
- **Lens:** test non-vacuity (mutation testing), fixture validity, build/clippy reality, oracle-gate add-only-rejection reasoning, positive-control coverage
- **Reviewer:** opus software architect (panel lens C)

---

## Method

- Read the full diff (`git diff origin/master..HEAD`), the brainstorm spec + plan-doc, and all 5 new test files + 9 new fixtures.
- Ran the FULL package suite `cargo test -p mnemonic-toolkit` (whole package, per `feedback_r0_review_run_full_package_suite` — NOT `--lib`, which is empty since lib gates `pub mod` behind `cfg(fuzzing)`) and `cargo clippy --all-targets -- -D warnings`.
- **Mutation-tested** the four required RED classes (H9, an H15 parser, L3, L1) plus a bonus axis-2 convert site (M14): reverted each fix in place, ran the targeted test, confirmed RED, then restored. Worktree verified byte-clean against HEAD afterward (`git status --porcelain` shows no tracked modifications).
- Verified the oracle-gate argument by code inspection (no `-chain=main` bitcoind node available; env-gate vars unset).

---

## Critical

None.

## Important

None.

## Minor

### m1 — (advisory) bitcoind differential oracle was NOT executed (env-gated, no running node)
A `bitcoind`/`bitcoin-cli` binary exists at `/usr/local/bin/`, but `BITCOIND_DATADIR`/`BITCOIND_RPCPORT` are unset and no `-chain=main` node is running, so `tests/bitcoind_differential.rs` correctly `#[ignore]`-skips. I did NOT run the harness. This is **not a finding against the cycle** — the spec/plan explicitly permit relying on the unchanged-AGREE argument by code inspection when bitcoind is unavailable, and the inspection argument here is actually *stronger* than a harness run: the oracle exercises only `bundle → restore → derive` (`bitcoind_differential.rs:6-7,162-214`), and **none of `bundle.rs` / `restore` / `synthesize.rs` / `derive_address` / `slip0132.rs` appear in the diff** (`git diff --name-only` confirmed). The harness file itself is unmodified (0-line diff). The AGREE rows cannot regress because the code they traverse is byte-identical. Recorded only so a future reader knows the oracle was reasoned, not executed.

### m2 — (advisory) two reject directions, one tested per axis at some sites
The descriptor parser tests both directions (`tpub-on-0` AND `xpub-on-1`); most other H15 parsers test a single direction plus a consistent control. This is adequate (the helper is direction-symmetric and unit-tested both ways in `network.rs`: `assert_network_agrees_main_vs_test_rejects` + `_test_vs_main_rejects_symmetric`), so per-parser single-direction coverage is not a gap. No action required.

---

## Mutation-test results (non-vacuity proof)

Each fix reverted in place; targeted test confirmed RED; restored. All RED tests proven NON-VACUOUS:

| Class | Fix reverted | Test driven RED | Observed when reverted | Positive control stayed GREEN? |
|---|---|---|---|---|
| **H9** (axis 1, exit 1) | per-entry class-loop → `first()`-only | `mixed_mainnet_testnet_blob_override_mainnet_refused_per_entry` | got `Some(0)`, expected `Some(1)` — Testnet entry silently relabeled | yes (`homogeneous_two_mainnet_blob_override_mainnet_ok`) |
| **H15** (axis 2, exit 2) — descriptor parser | `assert_slots_network_agrees` call commented out | `descriptor_tpub_on_coin_type_0_rejects` + `descriptor_xpub_on_coin_type_1_rejects` | both got `Some(0)` (silently imported "watch-only"), expected `Some(2)` | yes (`descriptor_consistent_mainnet_imports`, `descriptor_consistent_testnet_imports`) |
| **L3** (exit 2) | `u32::try_from` reject → truncating `as u32` | `l3_legacy_top_level_xpub_account_overflow_rejects` | got `Some(0)` (account `u32::MAX+1` truncated to 0, imported) | yes (`l3_legacy_top_level_xpub_in_range_account_bakes_correct_origin`) |
| **L1** (exit 0, WARN) | infer/WARN block → `args.network.unwrap_or(Mainnet)` | `l1_tpub_with_network_mainnet_warns_not_rejects` + `l1_tpub_network_omitted_infers_testnet_preview` | WARN absent (empty stderr); omitted-network preview fell back to mainnet | yes (`l1_tpub_with_network_testnet_no_warn`, `l1_mainnet_xpub_network_omitted_defaults_mainnet_preview`) |
| **M14** (bonus, axis 2, exit 2) | `assert_network_agrees` call removed | `m14_tpub_reemit_into_mainnet_prefix_rejects` | got `Some(0)` (tpub re-emitted into mainnet zpub family) | yes (`m14_consistent_mainnet_xpub_prefix_ok`) |

A key secondary finding from the mutations: when the network check is disabled, **the reject fixture imports cleanly (exit 0)**. This proves the reject fires *at the network check specifically*, not at an earlier parse error — i.e. the fixtures are otherwise-valid blobs for their format, failing ONLY on network provenance. (The reject-test asserters also gate on `stderr.contains("network mismatch")`, so a parse error could not masquerade as a pass.)

---

## Fixture validity — VALID

9 new fixtures inspected. Verdict: **all valid; none passes for the wrong reason.**

- **Dynamically-checksummed inline blobs** (descriptor/specter/bitcoin-core/bsms tests build the blob with `miniscript::descriptor::checksum::Engine` — the SAME engine the parsers validate against via `verify_checksum`, `descriptor.rs:53`). Checksums are correct by construction.
- **Hand-edited coin-type children** (sparrow/coldcard-multisig/electrum JSON/txt + the M13 envelopes): the reject fixtures carry a mainnet `xpub`/`Zpub` on a `…/1'/…` coin-type path (or a `tpub` on `…/0'/…`). The mutation test confirms each parses cleanly with the network check off → they reach the network check, not an earlier error. The electrum `Zpub` fixture is correctly normalized to a neutral mainnet xpub (`electrum.rs:837`) so its slot `.network == Main` vs coin-type-1 (Test) → mismatch.
- **Positive-control fixtures** are network-consistent: `xpub`-on-coin-type-0 / `tpub`-on-coin-type-1 (and the M13 testnet-label-testnet-keys envelope). All exist as files (pre-existing controls reused) and import at exit 0 under the live build (full suite green).
- **L3 fixtures** correctly exercise the legacy top-level-xpub branch: `coldcard-mk1-legacy-bip84-mainnet-account-{5,overflow}.json` are bare `{xpub, xfp, chain, account}` blobs (no per-bipN sub-object), so `deriv_path_str_opt == None` and `raw_account` is interpolated into the origin — the only branch where the truncation manifests. The mutation confirmed the overflow fixture would have silently baked `m/84'/0'/0'` pre-fix (non-vacuous); the in-range control bakes `m/84'/0'/5'`.

## Build / clippy reality

- `cargo test -p mnemonic-toolkit` → **3343 passed / 0 failed / 15 ignored** (aggregated across all binaries; matches the implementer's claim exactly).
- `cargo clippy --all-targets -- -D warnings` → **exit 0, clean** (no warnings).
- No flake observed across the full run and the ~6 targeted mutation re-runs.

## Oracle-gate reasoning — ADD-ONLY-REJECTION CONFIRMED

S-NET changes **no derived address for any previously-accepted valid input.** Evidence by code inspection:
- The 7 import parsers, convert (M14/L11), and export (M13) insert `assert_network_agrees` / `assert_slots_network_agrees` as **pure early-return guards placed AFTER the slots/network are already resolved** — they mutate no derivation field; for consistent input they return `Ok(())` and nothing downstream changes.
- H9: pre-fix and post-fix rebind a homogeneous valid blob to the identical `override_net.to_bitcoin_network()`; the only delta is the *added* reject for heterogeneous blobs. No address change for valid input.
- L1 build-descriptor is the only site that changes a chosen network for output, but ONLY for the **human first-address preview** (canonical/bip388 are network-agnostic, untouched) — and the differential oracle never runs `build-descriptor` (it runs `bundle → restore → derive`).
- The oracle's exercised code (`bundle`/`restore`/`synthesize`/`derive_address`/`slip0132`) is **entirely absent from the diff**; `bitcoind_differential.rs` is itself unmodified. Existing AGREE rows are therefore byte-identical by construction. New rejects are on corrupt/inconsistent input the oracle structurally cannot derive a "correct" address for; those asserts correctly live in the CLI suites as exit-code checks, not in the bitcoind harness.

**No address-change found for any previously-accepted valid input.**

## Positive-control coverage — COMPLETE

Every reject site has at least one passing positive control:

| Reject site | Positive control(s) |
|---|---|
| descriptor (H15) | consistent mainnet + consistent testnet |
| specter (H15) | consistent mainnet |
| sparrow (H15) | `sparrow-singlesig-p2wpkh.json` |
| bitcoin-core (H15) | consistent mainnet |
| bsms (H15=L10) | consistent testnet + consistent mainnet |
| coldcard-multisig (H15) | `coldcard-ms-2of3-p2wsh-no-xfp.txt` |
| electrum-multi (H15=L2) | `electrum-multisig-2of3-wsh.json` |
| H9 (axis 1) | homogeneous `[Bitcoin,Bitcoin]` + the pre-existing override-within-class suite |
| M14 convert | consistent mainnet `--xpub-prefix` |
| L11 wif→xpub | testnet-WIF + `--network testnet`; mainnet-WIF + `--network mainnet` |
| M13 export | `snet-envelope-testnet-consistent.json` |
| L1 build-descriptor | `--network testnet` agree (no warn); mainnet-xpub omitted (bc1, no warn) |
| L3 coldcard account | in-range account=5 bakes `m/84'/0'/5'` |
| network.rs helper | `assert_network_agrees_same_kind_is_ok` (Main/Main, Test/Test) |
| originless no-op (precondition) | full-suite green incl. `cli_descriptor_concrete.rs:174` originless tpub (3343/0) |

No reject site lacks a positive control. A future over-tightening (e.g. turning L1 into a reject, or over-rejecting an originless tpub) would fail an existing test.

---

## Verdict

**LENS-C TESTS/ORACLE: 0C / 0I — GREEN**

All four required RED classes (H9, H15-descriptor, L3, L1) — plus a bonus M14 — proven non-vacuous by in-place fix reversion. Fixtures valid (checksums engine-generated; coin-type children reach the network check, not an earlier parse error). Build 3343/0/15, clippy clean, no flake. Oracle gate satisfied by code inspection: add-only-rejection confirmed, no derived-address change for valid input, oracle-exercised paths untouched by the diff. Positive-control coverage complete. Two advisory Minors (oracle not executed — env-gated; single-direction per-parser coverage — adequate), neither blocking.
