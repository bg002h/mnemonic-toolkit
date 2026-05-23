//! BIP-352 Silent Payments — RECEIVER static address derivation.
//!
//! Derives the receiver's static silent-payment address (`sp1…`/`tsp1…`) from
//! a master `Xpriv`: the scan key `m/352'/coin'/account'/1'/0` and the spend
//! key `m/352'/coin'/account'/0'/0`, encoded as bech32m of the version symbol
//! `q` (v0) + `ser_P(B_scan) || ser_P(B_m)` (two compressed pubkeys). The base
//! (unlabeled) address uses `B_m = B_spend`; a labeled address (m≥1) uses
//! `B_m = B_spend + tagged_hash("BIP0352/Label", ser_256(b_scan) || ser_32(m))·G`.
//!
//! **Out of scope** (no tx inputs / chain access / signing — the toolkit
//! boundary): sender output construction + chain scanning. See
//! `recon-silent-payments.md`. Crypto verified vs BIP-352 (bitcoin/bips,
//! 2026-05-23) + the official `send_and_receive_test_vectors.json`.

use crate::error::ToolkitError;
use bitcoin::bech32::primitives::iter::{ByteIterExt, Fe32IterExt};
use bitcoin::bech32::{Bech32m, Fe32, Hrp};
use bitcoin::bip32::{ChildNumber, Xpriv};
use bitcoin::secp256k1::{PublicKey, Scalar, Secp256k1, SecretKey, Signing, Verification};
use sha2::{Digest, Sha256};

/// HRP per BIP-352: mainnet → `sp`; all testnets (testnet/signet/regtest) → `tsp`.
pub fn sp_hrp(network: bitcoin::Network) -> Hrp {
    match network {
        bitcoin::Network::Bitcoin => Hrp::parse("sp").expect("static hrp"),
        _ => Hrp::parse("tsp").expect("static hrp"),
    }
}

/// `hash_BIP0352/Label(ser_256(b_scan) || ser_32(m))` — BIP-340 tagged hash via
/// SHA-256: `SHA256(SHA256(tag) || SHA256(tag) || msg)`. `ser_256(b_scan)` is
/// the 32-byte big-endian scan scalar; `ser_32(m)` is the 4-byte big-endian u32.
fn bip0352_label_hash(b_scan: &SecretKey, m: u32) -> [u8; 32] {
    let tag = Sha256::digest(b"BIP0352/Label");
    let mut h = Sha256::new();
    h.update(tag);
    h.update(tag);
    h.update(b_scan.secret_bytes()); // ser_256(b_scan), big-endian
    h.update(m.to_be_bytes()); // ser_32(m), big-endian
    h.finalize().into()
}

/// `B_m = B_spend + t·G` for label `m` (m≥1), where `t` is the label tagged hash
/// as a scalar. Uses `PublicKey::add_exp_tweak` (computes `point + t·G`).
pub fn labeled_spend_key<C: Verification>(
    secp: &Secp256k1<C>,
    b_scan: &SecretKey,
    b_spend_pub: PublicKey,
    m: u32,
) -> Result<PublicKey, ToolkitError> {
    let hash = bip0352_label_hash(b_scan, m);
    let tweak = Scalar::from_be_bytes(hash)
        .map_err(|_| ToolkitError::SilentPayment("BIP-352 label tweak scalar out of range".into()))?;
    b_spend_pub
        .add_exp_tweak(secp, &tweak)
        .map_err(|e| ToolkitError::SilentPayment(format!("BIP-352 label tweak point-add: {e}")))
}

/// Encode an `sp`/`tsp` address: version symbol `q` (Fe32::Q, v0) prepended as a
/// raw 5-bit symbol, then `ser_P(B_scan) || ser_P(B_m)` (66 bytes) → bech32m.
/// NOTE: uses the low-level Fe32 path, NOT `segwit::encode` (90-char cap rejects
/// the ~117-char SP address) nor `encode(hrp,&data)` (no version symbol).
pub fn encode_sp_address(hrp: Hrp, b_scan_pub: &PublicKey, b_m_pub: &PublicKey) -> String {
    let mut payload = Vec::with_capacity(66);
    payload.extend_from_slice(&b_scan_pub.serialize());
    payload.extend_from_slice(&b_m_pub.serialize());
    core::iter::once(Fe32::Q)
        .chain(payload.iter().copied().bytes_to_fes())
        .with_checksum::<Bech32m>(&hrp)
        .chars()
        .collect()
}

/// Derive `(b_scan, b_spend)` from a master `Xpriv` at
/// `m/352'/coin_type'/account'/1'/0` (scan) and `m/352'/coin_type'/account'/0'/0`
/// (spend). Returns the raw BIP-32 child private scalars.
pub fn derive_scan_spend<C: Signing>(
    secp: &Secp256k1<C>,
    master: &Xpriv,
    coin_type: u32,
    account: u32,
) -> Result<(SecretKey, SecretKey), ToolkitError> {
    let h = |i: u32| -> Result<ChildNumber, ToolkitError> {
        ChildNumber::from_hardened_idx(i)
            .map_err(|e| ToolkitError::SilentPayment(format!("BIP-352 path index {i}: {e}")))
    };
    let n0 = ChildNumber::from_normal_idx(0)
        .map_err(|e| ToolkitError::SilentPayment(format!("BIP-352 leaf index: {e}")))?;
    // scan: .../1'/0 ; spend: .../0'/0
    let scan_path = [h(352)?, h(coin_type)?, h(account)?, h(1)?, n0];
    let spend_path = [h(352)?, h(coin_type)?, h(account)?, h(0)?, n0];
    let scan = master
        .derive_priv(secp, &scan_path.as_slice())
        .map_err(|e| ToolkitError::SilentPayment(format!("BIP-352 scan derivation: {e}")))?
        .private_key;
    let spend = master
        .derive_priv(secp, &spend_path.as_slice())
        .map_err(|e| ToolkitError::SilentPayment(format!("BIP-352 spend derivation: {e}")))?
        .private_key;
    Ok((scan, spend))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn secp() -> Secp256k1<bitcoin::secp256k1::All> {
        Secp256k1::new()
    }

    /// Byte-exact AUTHORITATIVE oracle: the official BIP-352
    /// `send_and_receive_test_vectors.json` (vendored at
    /// `tests/fixtures/bip352/`, bitcoin/bips@master fetched 2026-05-23).
    /// Receiving cases give raw hex `scan_priv_key`/`spend_priv_key` + `labels`
    /// (m≥1) + `expected.addresses` (index 0 = base, then one per label).
    #[test]
    fn bip352_official_vectors_base_and_labeled_addresses() {
        let raw = include_str!("../tests/fixtures/bip352/send_and_receive_test_vectors.json");
        let groups: Vec<Value> = serde_json::from_str(raw).expect("vectors parse");
        let secp = secp();
        let mut checked = 0usize;
        for tc in &groups {
            let r = &tc["receiving"][0];
            let km = &r["given"]["key_material"];
            let scan_hex = km["scan_priv_key"].as_str().expect("scan_priv_key");
            let spend_hex = km["spend_priv_key"].as_str().expect("spend_priv_key");
            let b_scan = SecretKey::from_slice(&hex::decode(scan_hex).unwrap()).unwrap();
            let b_spend = SecretKey::from_slice(&hex::decode(spend_hex).unwrap()).unwrap();
            let b_scan_pub = b_scan.public_key(&secp);
            let b_spend_pub = b_spend.public_key(&secp);
            // BIP-352 receiver HRP in the vectors is mainnet `sp`.
            let hrp = sp_hrp(bitcoin::Network::Bitcoin);
            let addrs: Vec<String> = r["expected"]["addresses"]
                .as_array()
                .expect("addresses")
                .iter()
                .map(|a| a.as_str().unwrap().to_string())
                .collect();
            let labels: Vec<u32> = r["given"]["labels"]
                .as_array()
                .map(|v| v.iter().map(|m| m.as_u64().unwrap() as u32).collect())
                .unwrap_or_default();
            // index 0 = base (B_m = B_spend)
            assert_eq!(
                encode_sp_address(hrp, &b_scan_pub, &b_spend_pub),
                addrs[0],
                "base address mismatch in case: {}",
                tc["comment"].as_str().unwrap_or("?")
            );
            // index 1.. = labeled, in `labels` order
            for (i, &m) in labels.iter().enumerate() {
                let b_m = labeled_spend_key(&secp, &b_scan, b_spend_pub, m).unwrap();
                assert_eq!(
                    encode_sp_address(hrp, &b_scan_pub, &b_m),
                    addrs[i + 1],
                    "labeled address (m={m}) mismatch in case: {}",
                    tc["comment"].as_str().unwrap_or("?")
                );
            }
            checked += 1;
        }
        assert!(checked >= 20, "expected ≥20 receiving cases; checked {checked}");
    }

    /// seed → `m/352'` derivation pin (the official vectors are key-based, not
    /// seed-based, so this leg is a regression pin over the standard BIP-32
    /// derivation spine; the encode/label crypto is byte-exact-validated above).
    /// Mnemonic = BIP-39 all-zero-entropy 12-word; mainnet, account 0.
    #[test]
    fn seed_to_path_derivation_is_stable_and_well_formed() {
        use bitcoin::bip32::Xpriv;
        let mnemonic = bip39::Mnemonic::from_entropy(&[0u8; 16]).unwrap();
        let seed = mnemonic.to_seed("");
        let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
        let secp = secp();
        let (b_scan, b_spend) = derive_scan_spend(&secp, &master, 0, 0).unwrap();
        // Scan and spend keys must differ (different hardened branch 1' vs 0').
        assert_ne!(b_scan.secret_bytes(), b_spend.secret_bytes());
        let addr = encode_sp_address(
            sp_hrp(bitcoin::Network::Bitcoin),
            &b_scan.public_key(&secp),
            &b_spend.public_key(&secp),
        );
        assert!(addr.starts_with("sp1q"), "mainnet base addr starts sp1q; got {addr}");
        // A v0 mainnet `sp` address is exactly 116 chars (66-byte payload is
        // fixed-length; the BIP's "≥117" assumes the longer `tsp` HRP). The
        // byte-exact validation is the official-vector test above.
        assert_eq!(addr.len(), 116, "mainnet sp v0 address is 116 chars; got {}", addr.len());
        // testnet → tsp1q
        let tmaster = Xpriv::new_master(bitcoin::NetworkKind::Test, &seed).unwrap();
        let (ts, tp) = derive_scan_spend(&secp, &tmaster, 1, 0).unwrap();
        let taddr = encode_sp_address(
            sp_hrp(bitcoin::Network::Testnet),
            &ts.public_key(&secp),
            &tp.public_key(&secp),
        );
        assert!(taddr.starts_with("tsp1q"), "testnet base addr starts tsp1q; got {taddr}");
    }
}
