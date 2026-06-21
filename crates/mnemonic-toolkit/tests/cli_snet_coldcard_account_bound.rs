//! cycle-5 S-NET L3 ride-along (firewalled from the network helper): the
//! Coldcard single-sig `account` field was read as `u64` then truncated `as
//! u32`, silently rewriting a `>u32::MAX` account index in the baked origin
//! path. The truncation only MANIFESTS in the legacy top-level-xpub fallback
//! (`deriv_path_str_opt == None`), where `raw_account` is interpolated into
//! `m/{purpose}'/{coin}'/{account}'` — a per-bipN fixture uses the sub-object's
//! own `deriv` and never touches `raw_account`, so it would be VACUOUS. Fix:
//! REJECT (ImportWalletParse, exit 2) on out-of-range, never saturate.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

fn run(name: &str) -> std::process::Output {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format",
            "coldcard",
            "--blob",
            fixture(name).to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("spawn")
}

#[test]
fn l3_legacy_top_level_xpub_account_overflow_rejects() {
    // account = u32::MAX + 1 on a legacy top-level-xpub blob (the only branch
    // that interpolates raw_account). RED: pre-fix this truncated to 0 and
    // baked `m/84'/0'/0'`; now it must reject (exit 2), never saturate.
    let out = run("coldcard-mk1-legacy-bip84-mainnet-account-overflow.json");
    assert_eq!(
        out.status.code(),
        Some(2),
        "out-of-range account must reject; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("account") && stderr.contains("exceeds u32 range"),
        "expected account-range diagnostic; got: {stderr}"
    );
}

#[test]
fn l3_legacy_top_level_xpub_in_range_account_bakes_correct_origin() {
    // Positive control: account = 5 (in range) → origin m/84'/0'/5' unchanged.
    let out = run("coldcard-mk1-legacy-bip84-mainnet-account-5.json");
    assert_eq!(out.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(
        v[0]["bundle"]["origin_path"].as_str(),
        Some("m/84'/0'/5'"),
        "in-range account must bake the literal origin; got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
