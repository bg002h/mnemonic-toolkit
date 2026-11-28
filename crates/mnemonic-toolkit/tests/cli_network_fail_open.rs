//! cycle-H F3 — `assert_network_agrees` fail-open closure on the five
//! previously-unguarded edges (constellation-eval 2026-07-06 F3, IMPORTANT).
//!
//! Per `design/SPEC_network_fail_open_assert_agrees.md` §4: an asserted
//! `--network` (explicit OR clap-default) that disagrees with a key's version
//! bytes must now fail closed (`NetworkMismatch`, exit 2) instead of silently
//! rendering/deriving/re-emitting at the wrong network.
//!
//! All keys are PUBLIC never-fund test vectors, derived from the standard
//! BIP-39 test mnemonic ("abandon"×11 + "about", empty passphrase) at BIP-84
//! paths via `mnemonic convert` (cross-checked against the sibling test files
//! `cli_snet_convert_export_network_mismatch.rs` / `cli_convert_address.rs` /
//! `cli_xpub_search_address_of_xpub.rs`, which independently pin several of
//! the same values).
//!
//! Edge map (SPEC §1):
//!   E1 — `convert` xpub→address        (convert.rs)
//!   E2 — `xpub-search address-of-xpub` (xpub_search/address_of_xpub.rs)
//!   E3 — `silent-payment` xprv/tprv master (silent_payment.rs)
//!   E4 — `export-wallet --template/--slot` (export_wallet.rs)
//!   E5 — `export-wallet --descriptor --format bsms` 4-line mint (wallet_export/bsms.rs)

use assert_cmd::Command;

fn bin() -> Command {
    // P3 6d: this helper is the file's only construction site, so the argv
    // override is applied here once rather than at every caller.
    let mut c = Command::cargo_bin("mnemonic").expect("binary built");
    c.arg("--allow-argv-secret");
    c
}

/// Assert exit 2 + the canonical `NetworkMismatch` stderr message.
fn assert_mismatch(out: &std::process::Output, who: &str) {
    assert_eq!(
        out.status.code(),
        Some(2),
        "{who}: must exit 2 (NetworkMismatch); stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("network mismatch"),
        "{who}: expected the network-mismatch message; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The network rule (Spec A): exit 8, stdout empty, stderr exactly `want`.
#[track_caller]
fn assert_rule(out: &std::process::Output, who: &str, want: &[&str]) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(8),
        "{who}: must exit 8; stderr: {stderr}"
    );
    assert!(out.stdout.is_empty(), "{who}: stdout must be empty");
    assert_eq!(stderr.lines().collect::<Vec<_>>(), want, "{who}");
}

/// TX-H1 for `key @0`, the test tpub, on a run with no `--network` (Spec A §3).
const H1_KEY0_TPUB_NOFLAG: &str = "error: network mismatch: key @0 is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.";

// ---------------------------------------------------------------------------
// Test vectors — all derived from the standard "abandon"×11 + "about" BIP-39
// test mnemonic (empty passphrase), BIP-84 single-sig, account 0. Verified at
// impl time via `mnemonic convert --from phrase=... --to xpub/xprv --template
// bip84 --network <mainnet|testnet>` against the running binary.
// ---------------------------------------------------------------------------

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// BIP-84 mainnet account xpub, m/84'/0'/0'.
const MAINNET_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
/// BIP-84 testnet account tpub, m/84'/1'/0'.
const TESTNET_TPUB: &str = "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";
/// BIP-84 testnet account tprv, m/84'/1'/0'.
const TESTNET_TPRV: &str = "tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd";

// BIP-84 §"Test vectors" reference (bitcoin/bips): account 0 zpub + its first
// receive address at m/0/0. <https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki>
const MAINNET_ZPUB: &str = "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs";
const MAINNET_RECEIVE_0: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
/// `TESTNET_TPUB`'s first receive address, m/0/0 (also independently pinned
/// by the E5 bsms 4-line line-4 derivation — both walk the same `/0/*` path).
const TESTNET_RECEIVE_0: &str = "tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl";

/// mk1 watch-only card wrapping `TESTNET_TPUB` (fingerprint `00000000`,
/// template bip84, network testnet) — 2 whitespace-joined chunks per the mk1
/// wire form `resolve_target_xpub` expects. Deterministic re-derivation via
/// `mnemonic bundle --slot @0.xpub=<TESTNET_TPUB> --slot @0.fingerprint=00000000
/// --template bip84 --network testnet --json` (verified byte-identical across
/// repeated runs — no randomness in the watch-only single-mk1 encode path).
const TESTNET_MK1: &str = "mk1qpgrpdpqqsq5psk6z5qqqqqqzvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpmsqu0z442zjf250ld \
mk1qpgrpdpp3c8mu0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02yskj2hqkegezdnk6eyarut3";

/// Concrete single-sig descriptor over `TESTNET_TPUB`, single receive branch.
fn e5_single_branch_desc() -> String {
    format!("wpkh([00000000/84h/1h/0h]{TESTNET_TPUB}/0/*)")
}

/// Concrete single-sig descriptor over `TESTNET_TPUB`, canonical BIP-389
/// `<0;1>/*` multipath form (the `MultiXPub` variant — R0-round-3-Important-D).
fn e5_multipath_desc() -> String {
    format!("wpkh([00000000/84h/1h/0h]{TESTNET_TPUB}/<0;1>/*)")
}

/// Hex-pubkey-only (`Single`) descriptor — the secp256k1 generator point `G`
/// in compressed form; a well-known public constant already used for the same
/// purpose in `cli_bundle_keyless_descriptor.rs`. No version bytes to
/// contradict — the E5 skip-arm non-regression pin.
const HEX_SINGLE_DESC: &str =
    "wpkh(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798)";

// ===========================================================================
// E1 — `convert` xpub→address (convert.rs)
// ===========================================================================

#[test]
fn e1a_testnet_tpub_to_address_mainnet_rejects() {
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("xpub={TESTNET_TPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
            "--script-type",
            "p2wpkh",
            "--network",
            "mainnet",
        ])
        .output()
        .expect("spawn");
    // Spec B unit 8a: the network rule (TX-H1, exit 8).
    assert_rule(
        &out,
        "E1a convert xpub→address",
        &["error: network mismatch: --from xpub is a test-network key (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."],
    );
}

#[test]
fn e1b_testnet_tpub_to_address_testnet_agrees_ok() {
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("xpub={TESTNET_TPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
            "--script-type",
            "p2wpkh",
            "--network",
            "testnet",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("address: {TESTNET_RECEIVE_0}\n")
    );
}

#[test]
fn e1c_testnet_tpub_to_address_no_network_refused() {
    // Spec B unit 8a (D1): no --network is mainnet; the test key is refused
    // (was: inferred from the key, exit 0).
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("xpub={TESTNET_TPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
            "--script-type",
            "p2wpkh",
        ])
        .output()
        .expect("spawn");
    assert_rule(
        &out,
        "E1c convert xpub→address, no --network",
        &["error: network mismatch: --from xpub is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."],
    );
}

#[test]
fn e1d_mainnet_zpub_to_address_mainnet_agrees_ok() {
    // Non-regression: mainnet key + mainnet --network still works.
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("xpub={MAINNET_ZPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
            "--script-type",
            "p2wpkh",
            "--network",
            "mainnet",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("address: {MAINNET_RECEIVE_0}\n")
    );
}

#[test]
fn e1_minor_signet_override_accepts_testnet_tpub() {
    // Minor-1: --network signet is NetworkKind::Test, same family as tpub —
    // must NOT be over-rejected (2-way NetworkKind granularity pin).
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("xpub={TESTNET_TPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
            "--script-type",
            "p2wpkh",
            "--network",
            "signet",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        format!("address: {TESTNET_RECEIVE_0}\n")
    );
}

// ===========================================================================
// E2 — `xpub-search address-of-xpub` (xpub_search/address_of_xpub.rs)
// ===========================================================================

#[test]
fn e2a_testnet_tpub_mainnet_network_rejects() {
    let out = bin()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            TESTNET_TPUB,
            "--target-address",
            TESTNET_RECEIVE_0,
            "--address-type",
            "p2wpkh",
            "--network",
            "mainnet",
        ])
        .output()
        .expect("spawn");
    // Spec B unit 8a: TX-H1 for the key, TX-H3 for the target, exit 8.
    assert_rule(
        &out,
        "E2a xpub-search address-of-xpub",
        &[
            "error: network mismatch: --xpub is a test-network key (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.",
            &format!("error: network mismatch: --target-address {TESTNET_RECEIVE_0} is a signet or testnet address, but --network is mainnet. Nothing was written. Rerun with --network signet or --network testnet, or give an address of mainnet."),
        ],
    );
}

#[test]
fn e2b_testnet_tpub_testnet_network_agrees_ok() {
    let out = bin()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            TESTNET_TPUB,
            "--target-address",
            TESTNET_RECEIVE_0,
            "--address-type",
            "p2wpkh",
            "--network",
            "testnet",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("match"));
}

#[test]
fn e2_minor_mk1_target_testnet_mainnet_network_rejects() {
    // Minor-1: the guard must cover mk1-decoded xpubs via resolve_target_xpub,
    // not just bare SLIP-0132 xpubs.
    let out = bin()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            TESTNET_MK1,
            "--target-address",
            TESTNET_RECEIVE_0,
            "--address-type",
            "p2wpkh",
            "--network",
            "mainnet",
        ])
        .output()
        .expect("spawn");
    // Spec B unit 8a: TX-H1 for the key, TX-H3 for the target, exit 8.
    assert_rule(
        &out,
        "E2 mk1-target address-of-xpub",
        &[
            "error: network mismatch: --xpub is a test-network key (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.",
            &format!("error: network mismatch: --target-address {TESTNET_RECEIVE_0} is a signet or testnet address, but --network is mainnet. Nothing was written. Rerun with --network signet or --network testnet, or give an address of mainnet."),
        ],
    );
}

// ===========================================================================
// E3 — `silent-payment` xprv/tprv master (silent_payment.rs)
// ===========================================================================

#[test]
fn e3a_testnet_tprv_mainnet_network_rejects() {
    let out = bin()
        .args([
            "silent-payment",
            "--secret",
            TESTNET_TPRV,
            "--network",
            "mainnet",
        ])
        .output()
        .expect("spawn");
    assert_mismatch(&out, "E3a silent-payment xprv/tprv master");
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("sp1"),
        "no silent-payment address must be emitted: {:?}",
        out.stdout
    );
}

#[test]
fn e3b_testnet_tprv_testnet_network_agrees_ok() {
    let out = bin()
        .args([
            "silent-payment",
            "--secret",
            TESTNET_TPRV,
            "--network",
            "testnet",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("tsp1"));
}

#[test]
fn e3c_seed_phrase_input_is_network_agnostic_ok() {
    // Regression pin: the ms1/phrase/entropy branches mint the master AT
    // --network (no embedded bytes to contradict) and stay unguarded.
    let out = bin()
        .args(["silent-payment", "--secret", PHRASE, "--network", "testnet"])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("tsp1"));

    let out_main = bin()
        .args(["silent-payment", "--secret", PHRASE, "--network", "mainnet"])
        .output()
        .expect("spawn");
    assert_eq!(
        out_main.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out_main.stderr)
    );
    assert!(String::from_utf8_lossy(&out_main.stdout).contains("sp1"));
}

// ===========================================================================
// E4 — `export-wallet --template/--slot` (export_wallet.rs)
// ===========================================================================

#[test]
fn e4a_template_slot_testnet_xpub_default_mainnet_network_rejects() {
    // Reproduces the R0-Important-1 live case: fires on the clap DEFAULT
    // --network (mainnet), no explicit flag needed.
    let out = bin()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={TESTNET_TPUB}"),
            "--slot",
            "@0.fingerprint=00000000",
            "--format",
            "electrum",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    // Spec A R1.12: TX-H1, exit 8.
    assert_rule(&out, "E4a export-wallet --template/--slot", &["error: network mismatch: --slot @0.xpub is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."]);
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("zpub"),
        "no mainnet zpub must be minted: {:?}",
        out.stdout
    );
}

#[test]
fn e4b_template_slot_testnet_xpub_explicit_testnet_network_ok() {
    let out = bin()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={TESTNET_TPUB}"),
            "--slot",
            "@0.fingerprint=00000000",
            "--network",
            "testnet",
            "--format",
            "electrum",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"vpub"),
        "expected a testnet vpub (BIP-84 SLIP-0132 coin-type-1) encoding: {stdout}"
    );
}

#[test]
fn e4c_template_slot_mainnet_xpub_default_network_ok() {
    // Non-regression: mainnet slot xpub + default (mainnet) --network.
    let out = bin()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={MAINNET_XPUB}"),
            "--slot",
            "@0.fingerprint=00000000",
            "--format",
            "electrum",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains(MAINNET_ZPUB));
}

#[test]
fn e4d_descriptor_concrete_passthrough_ok() {
    // Spec A unit 1 flips this pin: a test key with no --network (D1: mainnet)
    // is refused on every format, --format descriptor included.
    let out = bin()
        .args([
            "export-wallet",
            "--descriptor",
            &e5_single_branch_desc(),
            "--format",
            "descriptor",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    assert_rule(
        &out,
        "E4d --descriptor --format descriptor",
        &[H1_KEY0_TPUB_NOFLAG],
    );
}

#[test]
fn e4_minor_master_xpub_slot_testnet_default_mainnet_network_rejects() {
    // Minor-C fold: @N.master_xpub= is cross-checked too, not just @N.xpub=.
    let out = bin()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={MAINNET_XPUB}"),
            "--slot",
            "@0.fingerprint=00000000",
            "--slot",
            &format!("@0.master_xpub={TESTNET_TPUB}"),
            "--format",
            "coldcard",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    // Spec A R1.20: TX-H2, then TX-H1 without its rerun sentence.
    assert_rule(&out, "E4 master_xpub slot cross-check", &[
        "error: network mismatch: --slot @0.xpub is a mainnet key (xpub), but --slot @0.master_xpub is a test-network key (tpub). Nothing was written. Every key of one wallet is for one network: export the odd key again.",
        "error: network mismatch: --slot @0.master_xpub is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a mainnet wallet, export the key again with the signer set to mainnet.",
    ]);
}

// ===========================================================================
// E5 — `export-wallet --descriptor --format bsms` 4-line first-address mint
// (wallet_export/bsms.rs)
// ===========================================================================

#[test]
fn e5a_descriptor_testnet_tpub_default_mainnet_bsms_4line_rejects() {
    // Reproduces the R0-round-2-Important-A live case: the E4 slot-guard is
    // inert for --descriptor (empty slots); this is the direct mint guard.
    let out = bin()
        .args([
            "export-wallet",
            "--descriptor",
            &e5_single_branch_desc(),
            "--format",
            "bsms",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    // The network rule refuses before any format renders (Spec A unit 1).
    assert_rule(
        &out,
        "E5a export-wallet --descriptor --format bsms 4-line",
        &[H1_KEY0_TPUB_NOFLAG],
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("bc1q"),
        "no mainnet address line must be emitted: {:?}",
        out.stdout
    );
}

#[test]
fn e5b_descriptor_testnet_tpub_explicit_testnet_bsms_4line_ok() {
    let out = bin()
        .args([
            "export-wallet",
            "--descriptor",
            &e5_single_branch_desc(),
            "--network",
            "testnet",
            "--format",
            "bsms",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.trim_end_matches('\n').split('\n').collect();
    assert_eq!(lines.len(), 4, "expected 4 lines: {stdout}");
    assert_eq!(lines[3], TESTNET_RECEIVE_0);
}

#[test]
fn e5c_descriptor_testnet_tpub_default_mainnet_bsms_2line_ok() {
    // Spec A unit 1: the 2-line excerpt mints no address, but the rule runs
    // before every format, so a test key with no --network is refused here too.
    let out = bin()
        .args([
            "export-wallet",
            "--descriptor",
            &e5_single_branch_desc(),
            "--format",
            "bsms",
            "--bsms-form",
            "2-line",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    assert_rule(&out, "E5c bsms 2-line", &[H1_KEY0_TPUB_NOFLAG]);
}

#[test]
fn e5d_hex_single_key_descriptor_bsms_4line_ok() {
    // Skip-arm non-regression pin: a raw-hex Single key carries no version
    // bytes (`xkey_network()` returns None) — never rejected.
    let out = bin()
        .args([
            "export-wallet",
            "--descriptor",
            HEX_SINGLE_DESC,
            "--format",
            "bsms",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.trim_end_matches('\n').split('\n').collect();
    assert_eq!(lines.len(), 4, "expected 4 lines: {stdout}");
    assert!(lines[3].starts_with("bc1q"), "line 4: {:?}", lines[3]);
}

#[test]
fn e5e_multipath_testnet_tpub_default_mainnet_bsms_4line_rejects() {
    // R0-round-3-Important-D: the MultiXPub (BIP-389 `<0;1>/*`) variant must
    // be covered too — xkey_network() (not a hand-match on XPub only).
    let out = bin()
        .args([
            "export-wallet",
            "--descriptor",
            &e5_multipath_desc(),
            "--format",
            "bsms",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    assert_rule(
        &out,
        "E5e multipath descriptor bsms 4-line",
        &[H1_KEY0_TPUB_NOFLAG],
    );
}

#[test]
fn e5e_multipath_testnet_tpub_explicit_testnet_bsms_4line_ok() {
    let out = bin()
        .args([
            "export-wallet",
            "--descriptor",
            &e5_multipath_desc(),
            "--network",
            "testnet",
            "--format",
            "bsms",
            "--output",
            "-",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.trim_end_matches('\n').split('\n').collect();
    assert_eq!(lines.len(), 4, "expected 4 lines: {stdout}");
    assert_eq!(lines[2], "/0/*,/1/*");
    assert_eq!(lines[3], TESTNET_RECEIVE_0);
}
