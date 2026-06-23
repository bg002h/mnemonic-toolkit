//! mnemonic-toolkit JSON envelope (schema 4) consumer round-trip.
//! Binary-only crate at v0.8.0 — example consumes the documented BundleJson
//! contract via serde without any toolkit dependency.
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct BundleJson {
    schema_version: String,
    mode: String,
    network: String,
    template: Option<String>,
    origin_path: Option<String>,
    master_fingerprint: Option<String>,
    ms1: Vec<String>,
    mk1: Vec<String>,
    md1: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Minimal realistic BIP-84 single-key (wpkh) BundleJson fixture; schema_version 4.
    let json_fixture = r#"{"schema_version":"4","mode":"full","network":"mainnet","template":"bip84","descriptor":null,"account":0,"origin_path":"m/84'/0'/0'","origin_paths":null,"master_fingerprint":"5436d724","ms1":["ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"],"mk1":["mk1qprsqhpqqsq3c..."],"md1":["md1fgdxlpqpqpm6jzzq..."],"multisig":null,"privacy_preserving":false}"#;
    let bundle: BundleJson = serde_json::from_str(json_fixture)?;
    if bundle.schema_version != "4" {
        return Err(format!("unexpected schema_version: {}", bundle.schema_version).into());
    }
    println!(
        "parsed BundleJson: schema_version={} mode={} network={} template={} origin_path={} fingerprint={} ms1_len={} mk1_len={} md1_len={}",
        bundle.schema_version, bundle.mode, bundle.network,
        bundle.template.as_deref().unwrap_or("<none>"),
        bundle.origin_path.as_deref().unwrap_or("<none>"),
        bundle.master_fingerprint.as_deref().unwrap_or("<none>"),
        bundle.ms1.len(), bundle.mk1.len(), bundle.md1.len(),
    );
    Ok(())
}
