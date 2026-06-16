# PLAN — toolkit Bitcoin Core end-to-end differential oracle (SF-C)

**Cycle:** `toolkit-bitcoind-end-to-end-oracle` (resolves the open FOLLOWUP `design/FOLLOWUPS.md` of the same slug).
**Tier:** test-hardening / **NO-BUMP** (new test file + new CI workflow + FOLLOWUP flip; ZERO `src/` change, ZERO CLI-surface change → no schema_mirror / manual-mirror / version-marker impact; toolkit ships git-tag, this cuts no tag).
**Source SHA:** mnemonic-toolkit `master@ffdf3d3` (re-grounded from the scoping agent's `3c74c9e` pass; re-verify any cite that moved). Lift template: `descriptor-mnemonic/crates/md-codec/tests/bitcoind_differential.rs` + `.github/workflows/bitcoind-differential.yml`.

## 1. Why — the distinct surface

md-codec's bitcoind differential tests the CODEC-internal `Descriptor::derive_address` against Core. The toolkit has NO bitcoind oracle (recon §4: only an `error.rs` help string + a static-fixture comment mention bitcoin-cli). A toolkit oracle tests the **END-TO-END user pipeline** against Core: `bundle --descriptor` (engrave, watch-only) → `restore --md1` (reconstruct) → assert restore's reported descriptor + first addresses against Core's `deriveaddresses`. This validates the toolkit's OWN derivation (`derive_address.rs` / `address_render.rs` — the v0.49.1 route-AROUND md-codec for taproot) + the reconstruction path with an EXTERNAL C++ oracle. STRESS-A's O3 is a same-ecosystem rust-miniscript oracle (the toolkit delegates to the same patched fork); Bitcoin Core is the only oracle outside the rust-miniscript ecosystem — the one that catches the class both share. Headline value: shape `tr(NUMS,sortedmulti_a)` is derivable ONLY by the toolkit's patched fork (`95fdd1c` has `Terminal::SortedMultiA`; md-codec's crates.io 13.0.0 does not), so md-codec's oracle CANNOT cover it — the toolkit oracle is the only place Core checks that surface.

## 2. The pipeline + verified JSON field paths

Reuse STRESS-A's exact invocations (`tests/prop_backup_restore_roundtrip.rs`), swapping the rust-miniscript O3 for a Core `deriveaddresses` oracle.

- **bundle (engrave, watch-only):** `mnemonic bundle --descriptor <DESC> --network mainnet --json --no-engraving-card` → read `v["md1"]` (array of chunk strings). `--descriptor` = `bundle.rs`; `--no-engraving-card` flag present. Watch-only (concrete origin-annotated keys, NO `--slot`).
- **restore (reconstruct):** `mnemonic restore --md1 <c> [--md1 <c>…] --network mainnet --count 5 --json` → read `v["wallets"][0]["descriptor"]` + `v["wallets"][0]["first_addresses"]` (array). `--count` default 1 → MUST pass `--count 5` for indices 0..=4. Both single-sig and multisig restore envelopes share this `wallets[].{descriptor,first_addresses}` shape.
- **Oracle:** split the reconstructed/original descriptor's `/<0;1>/*` multipath to per-chain single descriptor STRINGS (Core rejects multipath) — `Descriptor::<DescriptorPublicKey>::from_str(d).unwrap().into_single_descriptors().unwrap()` returns `Vec<Descriptor>` (NOTE: `into_single_descriptors()` returns `Result`, so `.unwrap()` before indexing); take `[0]`=receive / `[1]`=change and `.to_string()` for the per-chain descriptor Core consumes (the test owns the split, mirroring `derive_address.rs:80-89`'s internal pattern + md-codec's `to_miniscript_descriptor(d,chain)`). Then `bitcoin-cli -chain=main … getdescriptorinfo <chain_desc>` (checksum round-trip) + `deriveaddresses <chain_desc> "[0,4]"`. Assert Core's address[i] == the toolkit's `first_addresses[i]` for chain-0 (receive — the branch restore reports), with a `FUNDS-CRITICAL` failure message. ALSO assert Core's `deriveaddresses` on the chain-0 split of the ORIGINAL descriptor equals `first_addresses` (catches a restore that reconstructs a different-but-Core-valid descriptor). Chain-1 (change) is a secondary check: re-derive the change branch from the SAME reconstructed descriptor via the test's own split and compare Core's chain-1 addresses against the toolkit's chain-1 addresses (obtained by re-running restore's derivation is not exposed; instead compare Core-on-reconstructed-chain1 vs Core-on-original-chain1 for descriptor-equivalence — see Risk 9).

## 3. Corpus (deterministic; NOT proptest-under-CI)

Reuse STRESS-A's frozen origin-annotated mainnet xpub pool (`prop_backup_restore_roundtrip.rs` `KEYS`) so descriptors are bundle-acceptable (origin-annotated concrete keys) AND render `xpub…` (Core on `-chain=main` requires mainnet xpubs). Every shape must sit in `toolkit-derivable ∩ Core-v27-sane ∩ restore-reconstructable` (bounded by `restore.rs::classify_taproot_restore`).

| # | label | descriptor |
|---|---|---|
| 1 | wpkh single-sig | `wpkh([fp/84h/0h/0h]xpub…/<0;1>/*)` — **anti-vacuity golden anchor** |
| 2 | pkh single-sig | `pkh(…/<0;1>/*)` |
| 3 | wsh(sortedmulti 2-of-3) | `wsh(sortedmulti(2,@0,@1,@2))` sole-child |
| 4 | wsh(multi 2-of-3) | `wsh(multi(2,@0,@1,@2))` |
| 5 | sh(wsh(sortedmulti 2-of-3)) | `sh(wsh(sortedmulti(2,@0,@1,@2)))` |
| 6 | wsh timelocked | `wsh(and_v(v:pk(@0),older(144)))` |
| 7 | wsh thresh | `wsh(thresh(2,pk(@0),s:pk(@1),s:pk(@2)))` |
| 8 | tr(NUMS,multi_a 2-of-3) | `tr(<NUMS_HEX>,multi_a(2,@0,@1,@2))` — the route-around path |
| 9 | tr(NUMS,sortedmulti_a 2-of-3) | `tr(<NUMS_HEX>,sortedmulti_a(2,@0,@1,@2))` — **md-codec oracle CANNOT cover this; highest distinct value** |

NUMS_HEX = `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` (BIP-341 H-point; `cli_restore_multisig.rs`). Shapes 8/9 mirror the live `tr_multi_a_reconstructs_nums_descriptor_and_bc1p` / `tr_sortedmulti_a_…` tests.

**Origin labels (M-1):** STRESS-A's `KEYS` literals bake `/48h/0h/0h/2h` into every origin. The toolkit treats the origin as OPAQUE metadata (no purpose/depth validation; derivation uses xpub + `/0/i` only), and Core's `deriveaddresses` ignores it. The corpus uses the `KEYS` xpubs **verbatim with their `/48h/0h/0h/2h` origins for every shape** (NOT cosmetically relabeled per-shape) — simplest, provably round-trips, and the origin is documented in-test as metadata-only. (A reader seeing `wpkh([fp/48h/0h/0h/2h]xpub…)` should not be alarmed: the origin does not affect derivation or the Core compare.)

**Shape-6 default-suite coverage (I-1):** shape 6 (`wsh(and_v(v:pk(@0),older(144)))`) is the ONE positive-corpus shape with NO existing default-suite proof — it's a pure n=1 general wsh policy (STRESS-A always puts a multi at the trunk; `cli_restore_multisig.rs` is multisig-only). md-codec renders the byte-identical shape (its bitcoind shape 9) and the `pk`-in-`and_v` fragment round-trips, so it WILL reconstruct — but the bitcoind oracle is `#[ignore]`-gated (cron-only), giving no default-CI signal. **Phase 2a adds a permanent default-suite characterization cell** (`n1_general_wsh_timelocked_restores_faithfully` in `cli_restore.rs` or `cli_restore_multisig_general.rs`): `bundle --descriptor wsh(and_v(v:pk(KEY0),older(144)))` (watch-only) → `restore --md1 … --json` → assert `wallets[0].descriptor` reconstructs the same policy + `first_addresses[0]` matches an independent miniscript derivation. This gates shape-6 reconstructability in normal CI, not only the env-gated cron job.

**R0 decisions made:** (1) INCLUDE shape 9 (highest distinct value; the toolkit can derive it, the live test proves bundle→restore→bc1p). (2) DEFER the non-NUMS distinct-trunk `tr(key,multi_a)` shape (v0.55.3-reconstructable but needs a NEW frozen depth-3 m/86' trunk-key literal + distinct-cosigner constraint — keep this cycle's key pool = STRESS-A's existing one; a follow-on can add it). (3) Chain-0 (receive) is the primary FUNDS-CRITICAL assertion; chain-1 is a descriptor-equivalence secondary (Risk 9).

**Excluded (restore REFUSES — keep OUT of the positive corpus):** `@`-in-both non-NUMS tr (structural funds-safety refusal), depth-≥2 taproot, nested sortedmulti_a (non-sole-leaf), sortedmulti-in-combinator (bundle accepts, restore refuses — sole-child only). Each cited in `restore.rs::classify_taproot_restore`. (Negative-refusal coverage is OUT of scope — owned by `cli_restore_multisig.rs` + STRESS-A; this oracle is positive-only.)

**Anti-vacuity golden (MANDATORY) — INDEPENDENT derivation (I-2):** shape 1 chain-0 idx-0. The golden is derived **independently in-test via rust-miniscript** by REPLICATING STRESS-A's `derive_receive(desc, count)` helper verbatim (`prop_backup_restore_roundtrip.rs:383-401` — each integration test is its own crate, so the helper is copied, not imported) and calling `derive_receive(WPKH_DESC, 1)[0]`. That helper uses the NON-DEPRECATED API (`derive_at_index`, NOT the deprecated `at_derivation_index` which the pinned fork `95fdd1c` marks `#[deprecated(since="13.0.0")]` and which `clippy -D warnings` would reject) with full `Result` handling:
```rust
fn derive_receive(desc: &str, count: u32) -> Vec<String> {
    use miniscript::{DefiniteDescriptorKey, Descriptor, descriptor::DescriptorPublicKey};
    let d = Descriptor::<DescriptorPublicKey>::from_str(desc).unwrap();
    let receive = if d.is_multipath() { d.clone().into_single_descriptors().unwrap().remove(0) } else { d.clone() };
    (0..count).map(|i| {
        let def: Descriptor<DefiniteDescriptorKey> = if receive.has_wildcard() {
            receive.derive_at_index(i).unwrap()
        } else { Descriptor::<DefiniteDescriptorKey>::try_from(receive.clone()).unwrap() };
        def.address(bitcoin::Network::Bitcoin).unwrap().to_string()
    }).collect()
}
```
This is NOT captured by reading the unit-under-test's own `restore … first_addresses[0]` (which would be circular/vacuous). It is ALSO pinned as `const WPKH_CHAIN0_IDX0_GOLDEN` (frozen anchor); the test asserts `derive_receive(WPKH_DESC, 1)[0] == WPKH_CHAIN0_IDX0_GOLDEN` at startup (catches key-pool/derivation drift), then asserts `restore`'s `first_addresses[0] == GOLDEN` BEFORE any Core compare (so a silently-dead Core connection can never vacuously pass, AND a toolkit-derivation break is caught by an independent oracle, not a self-snapshot). A `golden_asserted` bool flips true exactly once and is asserted at the end. The SAME `derive_receive` helper powers the Phase-2a shape-6 default-suite cell's address assertion.

## 4. New file 1 — `crates/mnemonic-toolkit/tests/bitcoind_differential.rs`

Integration test (shells the built `mnemonic` binary — it tests the CLI pipeline, not a lib fn). Resolve the binary via `MNEMONIC_BIN` → `env!("CARGO_BIN_EXE_mnemonic")` (the `cli_cross_tool_differential.rs::mnemonic_bin()` pattern, so CI can point at any build). Lift VERBATIM from md-codec's test: module doc (connect-only contract, pinned Core v27.0 + sha256, offline `-chain=main`, the three env vars), `struct Wiring`/`read_wiring()` (all three of BITCOINCLI_BIN/BITCOIND_DATADIR/BITCOIND_RPCPORT unset → None=skip; all set → Some; partial → panic), `bitcoin_cli(w,args)` (shells `$BITCOINCLI_BIN -chain=main -datadir=… -rpcport=… <args>`, cookie auth, panic on non-zero/non-JSON). Toolkit-specific: `mnemonic_bin()`, `bundle_md1(desc)`, `restore(md1) -> (desc, addrs)` (mirror STRESS-A), the 9-shape `corpus()` as concrete descriptor STRINGS from `KEYS`, `NUMS_HEX`, `WPKH_CHAIN0_IDX0_GOLDEN`, `N=4`, the multipath-split helper (`into_single_descriptors()`).

`#[test] #[ignore = "requires a pre-running offline -chain=main bitcoind (wiring env vars)"] fn bitcoind_end_to_end_differential()`: read_wiring→skip/panic; fail-loud `getblockchaininfo` + `chain=="main"`; per shape bundle_md1→restore→(recon_desc, reported_addrs); shape-1 golden assert + flip `golden_asserted`; chain-0 `getdescriptorinfo` checksum round-trip + `deriveaddresses "[0,4]"` == reported_addrs (FUNDS-CRITICAL msg); original-descriptor chain-0 `deriveaddresses` == reported_addrs; `assert!(golden_asserted)` + PASS summary.

**fmt:** brand-new file, NOT mlock.rs → subject to the rustfmt-1.95.0 gate. Format with `cargo +1.95.0 fmt` and confirm `cargo +1.95.0 fmt --all -- --check` shows NO `Diff in …bitcoind_differential.rs` (do NOT `cargo fmt --all` blindly — it re-wraps mlock.rs, the g6 hazard; if the check flags only mlock.rs that's the standing exemption). clippy `--all-targets -D warnings` must pass.

## 5. New file 2 — `.github/workflows/bitcoind-differential.yml`

Convention model = the IN-REPO `.github/workflows/cross-tool-differential.yml` (`@v5` actions, `MNEMONIC_BIN`, `cargo build --bin mnemonic`, `-- --ignored`), with the bitcoind lifecycle lifted from md-codec's workflow (M-3). Lift md-codec's 141-line workflow with toolkit edits: `name: bitcoind-differential`; `on: push`+`pull_request` on toolkit-relevant paths (`crates/mnemonic-toolkit/src/{derive_address,address_render,parse_descriptor}.rs`, `crates/mnemonic-toolkit/src/cmd/{restore,bundle}.rs`, the test, the workflow), `schedule: cron "17 5 * * *"`, `workflow_dispatch`; pinned Core v27.0 tarball + sha256 `2a6974c5…44a8` (VERBATIM); steps — `actions/checkout@v5`, `dtolnay/rust-toolchain@1.85.0` + `Swatinem/rust-cache@v2`, **DROP md-codec's "clone patched miniscript fork" step** (toolkit pins the fork via `[patch.crates-io]` in Cargo.toml — cargo resolves it), cache+download+verify+extract bitcoind, start offline `-chain=main` node (`-daemon -datadir -rpcport=18999 -connect=0 -listen=0 -blocksonly=1`, cookie, 60s poll), `cargo build --bin mnemonic` then run with `MNEMONIC_BIN=$GITHUB_WORKSPACE/target/debug/mnemonic` + the three bitcoind env vars: `cargo test -p mnemonic-toolkit --test bitcoind_differential -- --ignored --nocapture`; stop bitcoind `if: always()`; upload debug.log on failure (`@v5`).

## 6. Per-commit-green / CI-impact

The test is `#[ignore]`-gated → `cargo test -p mnemonic-toolkit` (the `rust.yml` matrix) SKIPS it → merging cannot redden the standard suite. The new workflow is path-triggered + daily-cron only. No sibling-pin/install.sh/manual lockstep (no flag/API change). `secret_taxonomy`/schema_mirror untouched.

## 7. Build sequence (R0 FIRST)

- **Phase 0 — R0 gate (no code):** this plan-doc → opus architect → fold → persist verbatim to `design/agent-reports/` → re-dispatch until **0C/0I**. R0 open points: shape-9 include (RECOMMEND yes), distinct-trunk defer (RECOMMEND defer), chain-1 depth (descriptor-equivalence secondary), golden capture procedure.
- **Phase 1 — derive golden (independent) + corpus:** build the 9 concrete descriptor strings from `KEYS`; derive shape-1 chain-0 idx-0 INDEPENDENTLY via rust-miniscript (I-2) and pin it as `WPKH_CHAIN0_IDX0_GOLDEN`; capture shape-6's bundle→restore evidence (reconstructed descriptor + first address) in the persisted review (I-1, the one novel shape). If a local bitcoind is available, run the full `--ignored` test once to confirm Core agrees + that the toolkit's reconstructed-descriptor checksum equals Core's `getdescriptorinfo` checksum for shape 1 (M-4); else rely on the CI first-live-run (as md-codec did).
- **Phase 2a — shape-6 default-suite cell (I-1):** add `n1_general_wsh_timelocked_restores_faithfully` to `cli_restore.rs`/`cli_restore_multisig_general.rs` (bundle→restore→faithful-descriptor + independent-derivation address assert; NO bitcoind). Confirm it's RED if shape-6 restore were broken (non-vacuity) and GREEN now. This gates shape-6 in the default suite.
- **Phase 2 — write the bitcoind test (TDD):** create `tests/bitcoind_differential.rs`; confirm default `cargo test -p mnemonic-toolkit` green (ignored test skipped; Phase-2a cell runs + passes); `cargo +1.95.0 fmt --all -- --check` (no new Diff beyond the standing mlock.rs exemption) + `cargo clippy --all-targets -- -D warnings` clean.
- **Phase 3 — write the workflow.**
- **Phase 4 — per-phase review** (opus, 0C/0I, persist verbatim).
- **Phase 5 — resolve FOLLOWUP** `toolkit-bitcoind-end-to-end-oracle` (open→done + SHA); note the descriptor-mnemonic companion `bitcoind-differential-corpus-breadth` stays open (separate item). (M-2 sequencing: the FOLLOWUP advises running this "after GAP-4a"; GAP-4a — the cross-tool corpus widening — is substantially in place (`cli_cross_tool_differential.rs` wired in CI), so proceeding now is consistent with that guidance.)
- **Phase 6 — stage explicitly** (no `git add -A`): the test, the workflow, FOLLOWUPS.md, the persisted reviews. Push master; confirm the new workflow's first live run goes green (downloads+verifies bitcoind, runs offline node) and the standard `rust.yml` stays green.

## 8. Risks

1. Core rejects `<0;1>` multipath → split to per-chain single descriptors before any Core RPC. 2. Core rejects mainnet xpubs on regtest → offline `-chain=main` only. 3. restore refusals contaminating the positive corpus → corpus constrained to `classify_taproot_restore`'s accepted domain (no @-in-both, depth-≥2, nested sortedmulti_a, combinator-sortedmulti). 4. `deriveaddresses` needs a range for `/*` → always `"[0,4]"`; `bitcoin_cli` panics loud on error. 5. Vacuous pass → golden (pre-compare) + fail-loud `getblockchaininfo`/`chain=="main"` + per-shape checksum round-trip. 6. CI cost → daily cron + path-trigger + dispatch; offline `-chain=main` ~1s start; ~90 address checks + ~18 checksums; tarball cached by sha256. 7. Cron 60-day auto-disable → `workflow_dispatch` lever. 8. restore `--count` default 1 → MUST pass `--count 5`. 9. Chain-1 (change): the toolkit reports only the receive branch in `first_addresses`; chain-1 is validated as DESCRIPTOR-EQUIVALENCE — Core's `deriveaddresses` on the reconstructed-desc chain-1 split must equal Core's on the original-desc chain-1 split (proves restore reconstructed a Core-identical descriptor on BOTH branches, even though the toolkit only surfaces receive). 10. Shape 9 fork-dependency: if the toolkit's patched miniscript ever loses `SortedMultiA`, shape 9 fails at bundle/restore — that's a real regression signal, intended.

## 9. File manifest

**Create:** `crates/mnemonic-toolkit/tests/bitcoind_differential.rs`; `.github/workflows/bitcoind-differential.yml`; `design/agent-reports/toolkit-bitcoind-oracle-plan-r0-round{N}-review.md`. **Modify:** `design/FOLLOWUPS.md` (flip `toolkit-bitcoind-end-to-end-oracle` open→done). **Read-only refs:** md-codec test+workflow; `prop_backup_restore_roundtrip.rs` (pipeline + KEYS); `cli_cross_tool_differential.rs` (MNEMONIC_BIN + workflow shape); `cli_restore_multisig.rs` (NUMS_HEX + tr corpus).
