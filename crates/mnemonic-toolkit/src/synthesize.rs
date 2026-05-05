//! Bundle synthesis: produce ms1 + mk1 + md1 strings from derived inputs.
//!
//! Realizes SPEC §4.4 (ms1), §4.5 (mk1), §4.6 (md1 typed-struct
//! construction with chain_code||pubkey 65-byte transform), §4.7
//! (cross-binding invariants).

use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::template::CliTemplate;
use bitcoin::bip32::{Fingerprint, Xpub};
use md_codec::origin_path::{PathDecl, PathDeclPaths};
use md_codec::use_site_path::UseSitePath;
use md_codec::{Descriptor, TlvSection};

pub struct Bundle {
    pub ms1: Option<String>,
    pub mk1: Vec<String>,
    pub md1: Vec<String>,
}

/// Convert a `bitcoin::bip32::Xpub` to md-codec's 65-byte form:
///   [0..32]  = chain_code
///   [32..65] = compressed pubkey
/// SPEC §4.6.1.
pub fn xpub_to_65(xpub: &Xpub) -> [u8; 65] {
    let mut out = [0u8; 65];
    out[0..32].copy_from_slice(&xpub.chain_code.to_bytes());
    out[32..65].copy_from_slice(&xpub.public_key.serialize());
    out
}

/// Build the typed Descriptor for a (template, network, xpub, fingerprint).
/// Caller's xpub MUST already be at the template's BIP path; not rederived.
/// SPEC §4.6.
pub fn build_descriptor(
    template: CliTemplate,
    network: CliNetwork,
    xpub: &Xpub,
    fingerprint: Fingerprint,
) -> Descriptor {
    let xpub_65 = xpub_to_65(xpub);
    let fp_bytes: [u8; 4] = fingerprint.to_bytes();
    let origin_path = template.md_origin_path(network);
    let tree = template.wrapper_node();

    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin_path),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(vec![(0, fp_bytes)]),
            pubkeys: Some(vec![(0, xpub_65)]),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    }
}

/// Synthesize a full-mode bundle (entropy known).
/// SPEC §4.4-§4.7.
pub fn synthesize_full(
    entropy: &[u8],
    fingerprint: Fingerprint,
    xpub: Xpub,
    template: CliTemplate,
    network: CliNetwork,
) -> Result<Bundle, ToolkitError> {
    let ms1 = ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Entr(entropy.to_vec()),
    )
    .map_err(ToolkitError::from)?;

    let descriptor = build_descriptor(template, network, &xpub, fingerprint);
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);

    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    let path = template.derivation_path(network);
    let card = mk_codec::KeyCard::new(vec![stub], Some(fingerprint), path, xpub);
    let mk1 = mk_codec::encode(&card).map_err(ToolkitError::from)?;

    debug_assert_eq!(&card.policy_id_stubs[0], &stub);
    debug_assert!(descriptor.is_wallet_policy());

    Ok(Bundle {
        ms1: Some(ms1),
        mk1,
        md1,
    })
}

/// Synthesize a watch-only bundle (no entropy known; ms1 omitted).
/// SPEC §4 watch-only path.
pub fn synthesize_watch_only(
    fingerprint: Fingerprint,
    xpub: Xpub,
    template: CliTemplate,
    network: CliNetwork,
) -> Result<Bundle, ToolkitError> {
    let descriptor = build_descriptor(template, network, &xpub, fingerprint);
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);

    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    let path = template.derivation_path(network);
    let card = mk_codec::KeyCard::new(vec![stub], Some(fingerprint), path, xpub);
    let mk1 = mk_codec::encode(&card).map_err(ToolkitError::from)?;

    debug_assert_eq!(&card.policy_id_stubs[0], &stub);
    debug_assert!(descriptor.is_wallet_policy());

    Ok(Bundle {
        ms1: None,
        mk1,
        md1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derive::derive_full;
    use crate::language::CliLanguage;

    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    fn fixture_full(template: CliTemplate, network: CliNetwork) -> (Vec<u8>, Fingerprint, Xpub) {
        let acc = derive_full(TREZOR_24, "", CliLanguage::English, network, template).unwrap();
        (acc.entropy, acc.master_fingerprint, acc.account_xpub)
    }

    #[test]
    fn xpub_to_65_layout() {
        let (_, _, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bytes = xpub_to_65(&xpub);
        assert_eq!(&bytes[0..32], xpub.chain_code.to_bytes().as_slice());
        assert_eq!(&bytes[32..65], xpub.public_key.serialize().as_slice());
    }

    #[test]
    fn full_bundle_emits_three_cards() {
        let (entropy, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle =
            synthesize_full(&entropy, fp, xpub, CliTemplate::Bip84, CliNetwork::Mainnet).unwrap();
        assert!(bundle.ms1.is_some());
        let ms1 = bundle.ms1.as_ref().unwrap();
        assert!(ms1.starts_with("ms1"));
        assert!(!bundle.mk1.is_empty());
        assert!(bundle.mk1.iter().all(|s| s.starts_with("mk1")));
        assert!(!bundle.md1.is_empty());
        assert!(bundle.md1.iter().all(|s| s.starts_with("md1")));
    }

    #[test]
    fn watch_only_bundle_omits_ms1() {
        let (_, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle =
            synthesize_watch_only(fp, xpub, CliTemplate::Bip84, CliNetwork::Mainnet).unwrap();
        assert!(bundle.ms1.is_none());
        assert!(!bundle.mk1.is_empty());
        assert!(bundle.mk1.iter().all(|s| s.starts_with("mk1")));
        assert!(!bundle.md1.is_empty());
        assert!(bundle.md1.iter().all(|s| s.starts_with("md1")));
    }

    #[test]
    fn cross_binding_holds_round_trip() {
        let (entropy, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle =
            synthesize_full(&entropy, fp, xpub, CliTemplate::Bip84, CliNetwork::Mainnet).unwrap();

        let mk1_strs: Vec<&str> = bundle.mk1.iter().map(|s| s.as_str()).collect();
        let decoded_mk1 = mk_codec::decode(&mk1_strs).unwrap();
        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let decoded_md1 = md_codec::chunk::reassemble(&md1_strs).unwrap();

        let policy_id = md_codec::compute_wallet_policy_id(&decoded_md1).unwrap();
        assert_eq!(&decoded_mk1.policy_id_stubs[0], &policy_id.as_bytes()[..4]);

        assert!(decoded_md1.is_wallet_policy());

        assert_eq!(decoded_mk1.xpub, xpub);
        assert_eq!(decoded_mk1.origin_fingerprint, Some(fp));
    }

    #[test]
    fn cross_binding_holds_all_4_templates_x_4_networks() {
        let templates = [
            CliTemplate::Bip44,
            CliTemplate::Bip49,
            CliTemplate::Bip84,
            CliTemplate::Bip86,
        ];
        let networks = [
            CliNetwork::Mainnet,
            CliNetwork::Testnet,
            CliNetwork::Signet,
            CliNetwork::Regtest,
        ];
        for &t in &templates {
            for &n in &networks {
                let (entropy, fp, xpub) = fixture_full(t, n);
                let bundle = synthesize_full(&entropy, fp, xpub, t, n).unwrap();
                let mk1_strs: Vec<&str> = bundle.mk1.iter().map(|s| s.as_str()).collect();
                let decoded_mk1 = mk_codec::decode(&mk1_strs).unwrap();
                let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
                let decoded_md1 = md_codec::chunk::reassemble(&md1_strs).unwrap();
                let policy_id = md_codec::compute_wallet_policy_id(&decoded_md1).unwrap();
                assert_eq!(
                    &decoded_mk1.policy_id_stubs[0],
                    &policy_id.as_bytes()[..4],
                    "stub linkage failed for {t:?} on {n:?}"
                );
                assert!(decoded_md1.is_wallet_policy(), "{t:?} on {n:?}");
                assert_eq!(decoded_mk1.xpub, xpub, "{t:?} on {n:?}");
                assert_eq!(decoded_mk1.origin_fingerprint, Some(fp), "{t:?} on {n:?}");
            }
        }
    }
}
