//! `mnemonic convert` — path-independence / route-convergence (metamorphic).
//!
//! Design: `design/SPEC_convert_path_independence_tests.md` (R0 RED 0C/2I →
//! folded → R1 GREEN, reviews in `design/agent-reports/convert-convergence-R{0,1}-review.md`).
//!
//! Property: when multiple `convert` routes carry the same key from a source to
//! the same target node, the output bytes are byte-identical. Same class that
//! found F3/F4 in `bundle`. Self-contained — only the `mnemonic` binary; no
//! sibling md/ms/mk binary, no network → default `cargo test`, no `#[ignore]`.
//!
//! Note F-fp (by-design, not a bug): `phrase/entropy→fingerprint` emit the
//! MASTER fingerprint (depth-0, template/account-independent); `xprv/xpub→
//! fingerprint` emit the account NODE's own fingerprint. Different keys, so the
//! matrix never asserts those two equal — C1a covers master-fp invariance.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ENT64: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const DIGITS_24: &str = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102";
const MS1_24: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w";

const MASTER_FP: &str = "5436d724";
const ACCT_NODE_FP: &str = "2bd87e08";
const BIP84_MAINNET_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const BIP84_TESTNET_TPUB: &str = "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";

// BIP-84/86 receive-0 reference addresses for the TREZOR_12 seed (mainnet).
const BIP84_RECEIVE_0: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
const BIP86_RECEIVE_0: &str = "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr";

/// Run `mnemonic convert <args>` (exit 0); return the value of a single-`--to`
/// emission, stripped of the `<node>: ` prefix + trailing newline.
/// Mirrors `cli_convert_round_trips.rs:22-32`.
fn convert_value(args: &[&str]) -> String {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.trim();
    let colon = line.find(": ").expect("convert output must be '<node>: <value>'");
    line[colon + 2..].to_string()
}

/// Run a compound `--to a,b,...`; return per-node values keyed by node name.
fn convert_lines(args: &[&str]) -> std::collections::BTreeMap<String, String> {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    stdout
        .lines()
        .filter_map(|l| l.find(": ").map(|c| (l[..c].to_string(), l[c + 2..].to_string())))
        .collect()
}

// ===========================================================================
// C1a — master fingerprint is invariant across --template / --account.
// (The only fingerprint reachable from a BIP-39 source; F-fp.)
// ===========================================================================
#[test]
fn c1a_master_fp_template_account_invariant() {
    let r1 = convert_value(&["convert", "--from", &format!("phrase={TREZOR_24}"), "--to", "fingerprint", "--template", "bip84"]);
    let r2 = convert_value(&["convert", "--from", &format!("phrase={TREZOR_24}"), "--to", "fingerprint", "--template", "bip44", "--account", "5"]);
    let r3 = convert_value(&["convert", "--from", &format!("entropy={ENT64}"), "--to", "fingerprint", "--template", "bip86"]);
    assert_eq!(r1, MASTER_FP, "C1a: master fp");
    assert_eq!(r1, r2, "C1a: master fp invariant across template/account");
    assert_eq!(r1, r3, "C1a: master fp invariant across source representation");
}

// ===========================================================================
// C2 — phrase→xpub (template) == phrase→xprv→xpub; folds in the account-node
// fingerprint leg (xprv→xpub,fingerprint).
// ===========================================================================
#[test]
fn c2_phrase_xpub_vs_phrase_xprv_xpub() {
    let x1 = convert_value(&["convert", "--from", &format!("phrase={TREZOR_24}"), "--to", "xpub", "--template", "bip84", "--network", "mainnet"]);
    let xprv = convert_value(&["convert", "--from", &format!("phrase={TREZOR_24}"), "--to", "xprv", "--template", "bip84", "--network", "mainnet"]);
    // Compound: xprv→xpub AND xprv→fingerprint (account-node fp) in one call.
    let m = convert_lines(&["convert", "--from", &format!("xprv={xprv}"), "--to", "xpub,fingerprint"]);
    assert_eq!(x1, BIP84_MAINNET_XPUB, "C2: phrase→xpub == pinned bip84 acct xpub");
    assert_eq!(m["xpub"], x1, "C2: phrase→xprv→xpub == phrase→xpub (account key preserved through neuter)");
    assert_eq!(m["fingerprint"], ACCT_NODE_FP, "C2: account-node fp via xprv→fingerprint");
}

// ===========================================================================
// C3 — phrase/entropy/seedqr/ms1→entropy all → same MASTER fingerprint.
// (ms1 has no direct derivation target → must go via entropy.)
// ===========================================================================
#[test]
fn c3_four_encodings_same_master_fp() {
    let fp = convert_value(&["convert", "--from", &format!("phrase={TREZOR_24}"), "--to", "fingerprint", "--template", "bip84"]);
    let fe = convert_value(&["convert", "--from", &format!("entropy={ENT64}"), "--to", "fingerprint", "--template", "bip84"]);
    let fs = convert_value(&["convert", "--from", &format!("seedqr={DIGITS_24}"), "--to", "fingerprint", "--template", "bip84"]);
    let e = convert_value(&["convert", "--from", &format!("ms1={MS1_24}"), "--to", "entropy"]);
    let fm = convert_value(&["convert", "--from", &format!("entropy={e}"), "--to", "fingerprint", "--template", "bip84"]);
    assert_eq!(e, ENT64, "C3: ms1→entropy == ENT64");
    assert_eq!(fp, MASTER_FP, "C3: phrase→fp");
    assert_eq!(fp, fe, "C3: entropy encoding converges");
    assert_eq!(fp, fs, "C3: seedqr encoding converges");
    assert_eq!(fp, fm, "C3: ms1→entropy→fp converges");
}

// ===========================================================================
// C4 — SLIP-0132 variant octet round-trip (7 cells; network-dependent neutral).
// mainnet → xpub; testnet → tpub (apply swaps version bytes; normalize maps
// testnet variants → tpub). zpub-mainnet dropped (cli_convert_slip0132.rs:388).
// ===========================================================================
#[test]
fn c4_slip0132_variant_octet_round_trip() {
    // (seed, --xpub-prefix value, --network, emitted-prefix, round-trip neutral)
    let cells: &[(&str, &str, &str, &str, &str)] = &[
        (BIP84_MAINNET_XPUB, "ypub", "mainnet", "ypub", BIP84_MAINNET_XPUB),
        (BIP84_MAINNET_XPUB, "Ypub", "mainnet", "Ypub", BIP84_MAINNET_XPUB),
        (BIP84_MAINNET_XPUB, "Zpub", "mainnet", "Zpub", BIP84_MAINNET_XPUB),
        (BIP84_TESTNET_TPUB, "ypub", "testnet", "upub", BIP84_TESTNET_TPUB),
        (BIP84_TESTNET_TPUB, "Ypub", "testnet", "Upub", BIP84_TESTNET_TPUB),
        (BIP84_TESTNET_TPUB, "zpub", "testnet", "vpub", BIP84_TESTNET_TPUB),
        (BIP84_TESTNET_TPUB, "Zpub", "testnet", "Vpub", BIP84_TESTNET_TPUB),
    ];
    for (seed, prefix, network, want_prefix, want_neutral) in cells {
        let variant = convert_value(&[
            "convert", "--from", &format!("xpub={seed}"), "--to", "xpub",
            "--xpub-prefix", prefix, "--network", network,
        ]);
        assert!(
            variant.starts_with(want_prefix),
            "C4 ({prefix},{network}): emitted {variant} should start with {want_prefix}"
        );
        let back = convert_value(&["convert", "--from", &format!("xpub={variant}"), "--to", "xpub"]);
        assert_eq!(
            &back, want_neutral,
            "C4 ({prefix},{network}): variant must normalize back to the network-neutral key"
        );
    }
}

// ===========================================================================
// C6 — phrase→bip38 (composite via WIF) == phrase→wif→bip38, byte-identical
// ciphertext. bip38 non-EC encrypt is deterministic (salt = address-hash).
// ===========================================================================
#[test]
fn c6_phrase_bip38_composite_eq_explicit_wif() {
    let path = "m/84'/0'/0'/0/0";
    let b1 = convert_value(&[
        "convert", "--from", &format!("phrase={TREZOR_12}"), "--to", "bip38",
        "--network", "mainnet", "--path", path, "--bip38-passphrase", "correct horse",
    ]);
    let w = convert_value(&[
        "convert", "--from", &format!("phrase={TREZOR_12}"), "--to", "wif",
        "--network", "mainnet", "--path", path,
    ]);
    let b2 = convert_value(&[
        "convert", "--from", &format!("wif={w}"), "--to", "bip38",
        "--bip38-passphrase", "correct horse",
    ]);
    assert!(b1.starts_with("6P"), "C6: bip38 ciphertext is 6P…");
    assert_eq!(b1, b2, "C6: phrase→bip38 == phrase→wif→bip38 (byte-identical ciphertext)");
}

// ===========================================================================
// C7 — phrase→address == phrase→xpub→address (the path-semantics cell).
// phrase→address: --path from MASTER to the leaf. xpub→address: --path relative
// to the account xpub. Constructed to hit the same leaf m/84'(86')/0'/0'/0/0.
// TREZOR_12 (the *_RECEIVE_0 constants are the 12-word seed's vectors).
// ===========================================================================
#[test]
fn c7_phrase_address_eq_phrase_xpub_address_p2wpkh() {
    let a1 = convert_value(&[
        "convert", "--from", &format!("phrase={TREZOR_12}"), "--to", "address",
        "--network", "mainnet", "--path", "m/84'/0'/0'/0/0", "--script-type", "p2wpkh",
    ]);
    let xa = convert_value(&[
        "convert", "--from", &format!("phrase={TREZOR_12}"), "--to", "xpub",
        "--template", "bip84", "--network", "mainnet",
    ]);
    let a2 = convert_value(&[
        "convert", "--from", &format!("xpub={xa}"), "--to", "address",
        "--path", "m/0/0", "--script-type", "p2wpkh",
    ]);
    assert_eq!(a1, BIP84_RECEIVE_0, "C7 p2wpkh: phrase→address");
    assert_eq!(a1, a2, "C7 p2wpkh: phrase→address == phrase→xpub→address");
}

#[test]
fn c7_phrase_address_eq_phrase_xpub_address_p2tr() {
    let a1 = convert_value(&[
        "convert", "--from", &format!("phrase={TREZOR_12}"), "--to", "address",
        "--network", "mainnet", "--path", "m/86'/0'/0'/0/0", "--script-type", "p2tr",
    ]);
    let xa = convert_value(&[
        "convert", "--from", &format!("phrase={TREZOR_12}"), "--to", "xpub",
        "--template", "bip86", "--network", "mainnet",
    ]);
    let a2 = convert_value(&[
        "convert", "--from", &format!("xpub={xa}"), "--to", "address",
        "--path", "m/0/0", "--script-type", "p2tr",
    ]);
    assert_eq!(a1, BIP86_RECEIVE_0, "C7 p2tr: phrase→address");
    assert_eq!(a1, a2, "C7 p2tr: phrase→address == phrase→xpub→address");
}
