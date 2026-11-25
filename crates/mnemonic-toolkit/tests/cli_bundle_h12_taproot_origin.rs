//! H12 (cycle-1), superseded by the 2026-11-25 BIP-defaults ruling:
//! descriptor-mode taproot multisig defaults every origin-elided cosigner to
//! BIP-87 `m/87'/<coin>'/<account>'`, not a BIP-48 `…/3'` leaf.
//!
//! BIP-48 defines script types `1'` (P2SH-P2WSH) and `2'` (P2WSH) only, so a
//! `3'` leaf for P2TR is a convention BIP-48 does not define. BIP-87 is
//! script-agnostic ("m / purpose' / coin_type' / account' / change /
//! address_index"; "Any script that is supported by descriptors (and the
//! specific wallet implementation) is compatible with this BIP.").
//!
//! These tests prove: (a) the emitted per-cosigner `origin_path` is
//! `m/87'/<coin>'/<account>'`; (b) the mk-decoded cosigner xpub equals the
//! INDEPENDENT BIP-87 derivation (an explicit `--slot @N.path=` override), NOT
//! the old `48'/…/3'` derivation; (c) the stderr notice renders the path it
//! used; (d) non-taproot wsh/sh(wsh) are unaffected.

use assert_cmd::Command;
use serde_json::Value;

// Two valid 12-word BIP-39 test seeds (distinct cosigners).
const SEED_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const SEED_B: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";

/// `bundle --network regtest --account 0 --descriptor <d> --slot @N.phrase …`
/// (+ optional per-slot extra args) → parsed `--json` Value + raw stderr.
fn bundle_taproot(descriptor: &str, extra: &[&str]) -> (Value, String) {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "regtest".into(),
        "--account".into(),
        "0".into(),
        "--descriptor".into(),
        descriptor.into(),
        "--slot".into(),
        format!("@0.phrase={SEED_A}"),
        "--slot".into(),
        format!("@1.phrase={SEED_B}"),
        "--no-engraving-card".into(),
        "--json".into(),
    ];
    for e in extra {
        args.push((*e).to_string());
    }
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args(&args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("bundle JSON invalid: {e}\nstdout:\n{stdout}"));
    (v, stderr)
}

fn cosigner_origins(v: &Value) -> Vec<String> {
    v["multisig"]["cosigners"]
        .as_array()
        .expect("multisig.cosigners array")
        .iter()
        .map(|c| {
            c["origin_path"]
                .as_str()
                .expect("origin_path str")
                .to_string()
        })
        .collect()
}

fn cosigner_xpubs(v: &Value) -> Vec<String> {
    v["multisig"]["cosigners"]
        .as_array()
        .expect("multisig.cosigners array")
        .iter()
        .map(|c| c["xpub"].as_str().expect("xpub str").to_string())
        .collect()
}

/// Core assertion: taproot multi_a defaults every origin-elided cosigner to
/// `m/87'/1'/0'` (regtest coin-type 1), and the mk-decoded cosigner xpub
/// equals the INDEPENDENT BIP-87 derivation (NOT the old `48'/…/3'` one).
#[test]
fn taproot_multi_a_defaults_origin_to_bip87() {
    let (v, stderr) = bundle_taproot("tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))", &[]);

    let origins = cosigner_origins(&v);
    assert_eq!(
        origins,
        vec!["m/87'/1'/0'", "m/87'/1'/0'"],
        "taproot multi_a cosigner origins must default to BIP-87"
    );
    assert_eq!(
        v["origin_path"].as_str(),
        Some("m/87'/1'/0'"),
        "top-level origin_path must also be BIP-87"
    );

    // Independent oracle: the SAME bundle with an explicit BIP-87 override.
    let (v87, _) = bundle_taproot(
        "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        &[
            "--slot",
            "@0.path=m/87'/1'/0'",
            "--slot",
            "@1.path=m/87'/1'/0'",
        ],
    );
    assert_eq!(
        cosigner_xpubs(&v),
        cosigner_xpubs(&v87),
        "default taproot cosigner xpubs must equal the explicit BIP-87 derivation"
    );

    // …and DIFFER from the old `48'/…/3'` derivation.
    let (v3, _) = bundle_taproot(
        "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        &[
            "--slot",
            "@0.path=m/48'/1'/0'/3'",
            "--slot",
            "@1.path=m/48'/1'/0'/3'",
        ],
    );
    assert_ne!(
        cosigner_xpubs(&v),
        cosigner_xpubs(&v3),
        "default taproot cosigner xpubs must NOT equal the old 48'/../3' derivation"
    );

    assert!(
        stderr.contains("defaulting to m/87'/1'/0' (BIP-87)"),
        "default-path notice must render the BIP-87 path; got stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("m/48'"),
        "default-path notice must NOT render a BIP-48 path for a taproot descriptor; got:\n{stderr}"
    );
}

/// `tr(NUMS, sortedmulti_a(...))` likewise defaults to BIP-87.
#[test]
fn taproot_sortedmulti_a_defaults_origin_to_bip87() {
    let (v, _) = bundle_taproot("tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))", &[]);
    assert_eq!(
        cosigner_origins(&v),
        vec!["m/87'/1'/0'", "m/87'/1'/0'"],
        "taproot sortedmulti_a cosigner origins must default to BIP-87"
    );
}

/// Clean-negative: a non-taproot wsh(sortedmulti) descriptor is CANONICAL —
/// its origin is elided (empty), and crucially it is NOT given a spurious `3'`
/// (the taproot detection must be specific to `Tag::Tr`).
#[test]
fn wsh_sortedmulti_not_given_3prime() {
    let (v, _) = bundle_taproot("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))", &[]);
    for op in cosigner_origins(&v) {
        assert!(
            !op.ends_with("/3'") && !op.starts_with("m/87'"),
            "wsh cosigner origin must NOT be a taproot default; got {op}"
        );
    }
}

/// Clean-negative: sh(wsh(sortedmulti)) likewise is not given a spurious `3'`.
#[test]
fn sh_wsh_sortedmulti_not_given_3prime() {
    let (v, _) = bundle_taproot("sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*)))", &[]);
    for op in cosigner_origins(&v) {
        assert!(
            !op.ends_with("/3'") && !op.starts_with("m/87'"),
            "sh(wsh) cosigner origin must NOT be a taproot default; got {op}"
        );
    }
}
