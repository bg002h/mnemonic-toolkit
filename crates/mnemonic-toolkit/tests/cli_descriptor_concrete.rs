//! A1 — bare-concrete descriptor acceptance on bundle/verify/export.
use assert_cmd::Command;

const CONCRETE_MULTI_APOS: &str = "wsh(sortedmulti(2,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";

fn mnemonic() -> Command { Command::cargo_bin("mnemonic").unwrap() }

/// Expand a bundle JSON value into flat `--md1 <chunk>` and `--mk1 <chunk>`
/// flag pairs suitable for passing to `verify-bundle`. The `mk1` field in
/// bundle JSON is an array-of-arrays (one inner array per cosigner); this fn
/// flattens all inner arrays into a single flat sequence of `--mk1 <chunk>`
/// pairs, which matches `mk1`'s `num_args=1..` clap intake.
fn flags_from_bundle_json(v: &serde_json::Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    // md1 — flat Vec<String>
    if let Some(arr) = v["md1"].as_array() {
        for chunk in arr {
            if let Some(s) = chunk.as_str() {
                out.push("--md1".into());
                out.push(s.to_string());
            }
        }
    }
    // mk1 — array-of-arrays (one inner array per cosigner); flatten all chunks
    if let Some(outer) = v["mk1"].as_array() {
        for inner in outer {
            if let Some(inner_arr) = inner.as_array() {
                for chunk in inner_arr {
                    if let Some(s) = chunk.as_str() {
                        out.push("--mk1".into());
                        out.push(s.to_string());
                    }
                }
            }
        }
    }
    out
}

#[test]
fn verify_bundle_concrete_matches_self_produced_cards() {
    // Produce a bundle from the concrete descriptor, then verify the SAME
    // descriptor against those cards → exit 0.
    let produced = mnemonic()
        .args(["bundle", "--descriptor", CONCRETE_MULTI_APOS, "--network", "testnet", "--json"])
        .output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&produced.stdout).unwrap();
    let mut args: Vec<String> =
        vec!["verify-bundle".into(), "--descriptor".into(), CONCRETE_MULTI_APOS.into(),
             "--network".into(), "testnet".into()];
    args.extend(flags_from_bundle_json(&v));
    let out = mnemonic().args(&args).output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

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
