//! A1 — bare-concrete descriptor acceptance on bundle/verify/export.
use assert_cmd::Command;

const CONCRETE_MULTI_APOS: &str = "wsh(sortedmulti(2,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";

fn mnemonic() -> Command { Command::cargo_bin("mnemonic").unwrap() }

#[test]
fn bundle_concrete_descriptor_produces_watch_only_cards() {
    let out = mnemonic()
        .args(["bundle", "--descriptor", CONCRETE_MULTI_APOS, "--network", "testnet", "--json"])
        .output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    // Real BundleJson wire-shape: md1 = Vec<String>, ms1 = length-N array with
    // "" sentinels for watch-only, mode = "watch-only".
    assert_eq!(v["mode"], "watch-only", "{v}");
    assert!(v["md1"].as_array().map_or(false, |a| !a.is_empty()), "md1 array: {v}");
    assert!(v["ms1"].as_array().unwrap().iter().all(|s| s == ""), "watch-only ms1 must be all empty: {v}");
}
