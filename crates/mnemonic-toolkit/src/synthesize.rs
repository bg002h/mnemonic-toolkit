//! Bundle synthesis: produce ms1 + mk1 + md1 strings from derived inputs.
//!
//! Realizes SPEC §4.4 (ms1), §4.5 (mk1), §4.6 (md1 typed-struct
//! construction with chain_code||pubkey 65-byte transform), §4.7
//! (cross-binding invariants).

use crate::error::ToolkitError;
use crate::format::MkField;
use crate::network::CliNetwork;
use crate::parse::{CosignerSpec, MultisigPathFamily};
use crate::template::CliTemplate;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::use_site_path::UseSitePath;
use md_codec::{Descriptor, TlvSection};
use std::str::FromStr;

#[derive(Debug)]
pub struct Bundle {
    pub ms1: Option<String>,
    pub mk1: MkField,
    pub md1: Vec<String>,
}

/// Derive a deterministic 20-bit `chunk_set_id` for mk1 from the 4-byte
/// `policy_id_stub`. Top 20 bits, MSB-first. Mirrors md-codec's
/// `derive_chunk_set_id` shape so mk1 byte-output is reproducible across runs
/// (toolkit fixture regeneration relies on this; v0.1 byte-determinism contract).
pub(crate) fn derive_mk1_chunk_set_id(stub: &[u8; 4]) -> u32 {
    ((stub[0] as u32) << 12) | ((stub[1] as u32) << 4) | ((stub[2] as u32) >> 4)
}

/// Convert a `bitcoin::bip32::DerivationPath` to md-codec's `OriginPath`.
/// Used by multisig synthesis where per-cosigner paths come from cosigner
/// specs (watch-only) or path-family-derived strings (full mode).
pub(crate) fn derivation_path_to_origin_path(p: &DerivationPath) -> OriginPath {
    let components = p
        .into_iter()
        .map(|cn| {
            let v: u32 = (*cn).into();
            // bitcoin's child_number `v` already encodes the hardened bit at
            // position 0x8000_0000; mask it off and record `hardened` separately.
            const HARD: u32 = 0x8000_0000;
            let hardened = v & HARD != 0;
            let value = v & !HARD;
            PathComponent { hardened, value }
        })
        .collect();
    OriginPath { components }
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

/// Build the typed Descriptor for a (template, network, xpub, fingerprint, account).
/// Caller's xpub MUST already be at the template's BIP path; not rederived.
/// SPEC §4.6.
pub fn build_descriptor(
    template: CliTemplate,
    network: CliNetwork,
    xpub: &Xpub,
    fingerprint: Fingerprint,
    account: u32,
) -> Descriptor {
    let xpub_65 = xpub_to_65(xpub);
    let fp_bytes: [u8; 4] = fingerprint.to_bytes();
    let origin_path = template.md_origin_path(network, account);
    let tree = template.wrapper_node(1, 1);

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
    account: u32,
) -> Result<Bundle, ToolkitError> {
    let ms1 = ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Entr(entropy.to_vec()),
    )
    .map_err(ToolkitError::from)?;

    let descriptor = build_descriptor(template, network, &xpub, fingerprint, account);
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);

    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    let path = template.derivation_path(network, account);
    let card = mk_codec::KeyCard::new(vec![stub], Some(fingerprint), path, xpub);
    let csi = derive_mk1_chunk_set_id(&stub);
    let mk1 = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;

    debug_assert_eq!(&card.policy_id_stubs[0], &stub);
    debug_assert!(descriptor.is_wallet_policy());

    Ok(Bundle {
        ms1: Some(ms1),
        mk1: MkField::Single(mk1),
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
    account: u32,
) -> Result<Bundle, ToolkitError> {
    let descriptor = build_descriptor(template, network, &xpub, fingerprint, account);
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);

    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    let path = template.derivation_path(network, account);
    let card = mk_codec::KeyCard::new(vec![stub], Some(fingerprint), path, xpub);
    let csi = derive_mk1_chunk_set_id(&stub);
    let mk1 = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;

    debug_assert_eq!(&card.policy_id_stubs[0], &stub);
    debug_assert!(descriptor.is_wallet_policy());

    Ok(Bundle {
        ms1: None,
        mk1: MkField::Single(mk1),
        md1,
    })
}

/// SPEC §4.1 multisig: derive xpub at a path string from the master xpriv.
fn derive_xpub_at_path(
    master: &Xpriv,
    secp: &Secp256k1<bitcoin::secp256k1::All>,
    path_str: &str,
) -> Result<Xpub, ToolkitError> {
    let path = DerivationPath::from_str(path_str)
        .map_err(|e| ToolkitError::BadInput(format!("path parse {}: {}", path_str, e)))?;
    let xpriv = master
        .derive_priv(secp, &path)
        .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
    Ok(Xpub::from_priv(secp, &xpriv))
}

/// Synthesize a full-mode multisig bundle (self-multisig: N cosigners derived
/// from one seed at one path; all N xpubs are byte-identical).
/// SPEC §4.1, §4.5 multisig, §4.6 multisig.
#[allow(clippy::too_many_arguments)]
pub fn synthesize_multisig_full(
    seed_mnemonic: &bip39::Mnemonic,
    passphrase: &str,
    network: CliNetwork,
    template: CliTemplate,
    threshold: u8,
    cosigner_count: usize,
    account: u32,
    path_family: MultisigPathFamily,
    privacy_preserving: bool,
) -> Result<Bundle, ToolkitError> {
    // 1. Validate config (SPEC §2.1.1).
    if cosigner_count == 0 || cosigner_count > 16 {
        return Err(ToolkitError::MultisigConfig {
            message: format!("cosigner_count {} out of range 1..=16", cosigner_count),
        });
    }
    if threshold == 0 || threshold as usize > cosigner_count {
        return Err(ToolkitError::MultisigConfig {
            message: format!(
                "threshold {} out of range 1..={} (cosigner_count)",
                threshold, cosigner_count
            ),
        });
    }
    if !template.is_multisig() {
        return Err(ToolkitError::MultisigConfig {
            message: format!(
                "template {} is single-sig; multisig synthesis requires a multisig template",
                template.human_name()
            ),
        });
    }

    // 2. Master xpriv.
    let seed = seed_mnemonic.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed)
        .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
    let master_fingerprint = master.fingerprint(&secp);

    // 3. Self-multisig: derive all N at the same path-family path.
    let script_type = template.bip48_script_type().unwrap_or(0);
    let path_str = path_family.default_origin_path(network, account, script_type);
    let xpub = derive_xpub_at_path(&master, &secp, &path_str)?;
    let path = DerivationPath::from_str(&path_str)
        .map_err(|e| ToolkitError::BadInput(format!("path parse {}: {}", path_str, e)))?;

    // 4. Build multisig descriptor.
    let xpub_65 = xpub_to_65(&xpub);
    let fp_bytes: [u8; 4] = master_fingerprint.to_bytes();
    let origin_path = derivation_path_to_origin_path(&path);
    let tree = template.wrapper_node(threshold, cosigner_count);

    let fingerprints: Vec<(u8, [u8; 4])> =
        (0..cosigner_count).map(|i| (i as u8, fp_bytes)).collect();
    let pubkeys: Vec<(u8, [u8; 65])> = (0..cosigner_count).map(|i| (i as u8, xpub_65)).collect();

    let descriptor = Descriptor {
        n: cosigner_count as u8,
        path_decl: PathDecl {
            n: cosigner_count as u8,
            paths: PathDeclPaths::Shared(origin_path),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(fingerprints),
            pubkeys: Some(pubkeys),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };

    // 5. Compute policy_id + N-element stubs list.
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);
    let stubs: Vec<[u8; 4]> = vec![stub; cosigner_count];

    // 6+7. Build N KeyCards + emit per-cosigner mk1.
    let mut per_cosigner: Vec<Vec<String>> = Vec::with_capacity(cosigner_count);
    for i in 0..cosigner_count {
        let card = mk_codec::KeyCard::new(
            stubs.clone(),
            if privacy_preserving {
                None
            } else {
                Some(master_fingerprint)
            },
            path.clone(),
            xpub,
        );
        debug_assert_eq!(card.policy_id_stubs, stubs);
        debug_assert!(descriptor.is_wallet_policy());
        let csi = derive_mk1_chunk_set_id(&stubs[i]);
        let chunks = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;
        per_cosigner.push(chunks);
    }

    // 8. md1.
    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    // 9. ms1.
    let entropy = seed_mnemonic.to_entropy();
    let ms1 = ms_codec::encode(ms_codec::Tag::ENTR, &ms_codec::Payload::Entr(entropy))
        .map_err(ToolkitError::from)?;

    Ok(Bundle {
        ms1: Some(ms1),
        mk1: MkField::Multi(per_cosigner),
        md1,
    })
}

/// Synthesize a watch-only multisig bundle from cosigner xpubs.
/// SPEC §4.1, §4.3, §4.5 multisig, §4.6 multisig.
#[allow(clippy::too_many_arguments)]
pub fn synthesize_multisig_watch_only(
    cosigners: &[CosignerSpec],
    network: CliNetwork,
    template: CliTemplate,
    threshold: u8,
    account: u32,
    path_family: MultisigPathFamily,
    privacy_preserving: bool,
) -> Result<Bundle, ToolkitError> {
    let cosigner_count = cosigners.len();

    // 1. Validate config.
    if cosigner_count == 0 || cosigner_count > 16 {
        return Err(ToolkitError::MultisigConfig {
            message: format!("cosigner_count {} out of range 1..=16", cosigner_count),
        });
    }
    if threshold == 0 || threshold as usize > cosigner_count {
        return Err(ToolkitError::MultisigConfig {
            message: format!(
                "threshold {} out of range 1..={} (cosigner_count)",
                threshold, cosigner_count
            ),
        });
    }
    if !template.is_multisig() {
        return Err(ToolkitError::MultisigConfig {
            message: format!(
                "template {} is single-sig; multisig synthesis requires a multisig template",
                template.human_name()
            ),
        });
    }

    // 2. SPEC §4.3 per-cosigner network/xpub cross-check.
    for (i, c) in cosigners.iter().enumerate() {
        if c.xpub.network != network.network_kind() {
            return Err(ToolkitError::CosignerSpec {
                cosigner_idx: i,
                message: format!(
                    "xpub network {:?} does not match --network {}",
                    c.xpub.network,
                    network.human_name()
                ),
            });
        }
    }

    // 3. Per-cosigner path resolution (default to family).
    let script_type = template.bip48_script_type().unwrap_or(0);
    let default_path_str = path_family.default_origin_path(network, account, script_type);
    let default_path = DerivationPath::from_str(&default_path_str).map_err(|e| {
        ToolkitError::BadInput(format!("default path parse {}: {}", default_path_str, e))
    })?;

    let mut paths: Vec<DerivationPath> = Vec::with_capacity(cosigner_count);
    for c in cosigners {
        paths.push(c.path.clone().unwrap_or_else(|| default_path.clone()));
    }

    // 4. SPEC §4.5 path/xpub depth consistency check.
    for (i, c) in cosigners.iter().enumerate() {
        let path_depth = paths[i].len() as u8;
        if path_depth != c.xpub.depth {
            return Err(ToolkitError::CosignerSpec {
                cosigner_idx: i,
                message: format!(
                    "path depth {} does not match xpub depth {}; xpub at depth {} expects path of depth {}",
                    path_depth, c.xpub.depth, c.xpub.depth, c.xpub.depth
                ),
            });
        }
    }

    // 5. Determine PathDeclPaths variant.
    let origin_paths: Vec<OriginPath> = paths.iter().map(derivation_path_to_origin_path).collect();
    let all_same = origin_paths.windows(2).all(|w| w[0] == w[1]);
    let path_decl_paths = if all_same {
        PathDeclPaths::Shared(origin_paths[0].clone())
    } else {
        PathDeclPaths::Divergent(origin_paths)
    };

    // 6. Build descriptor.
    let tree = template.wrapper_node(threshold, cosigner_count);
    let fingerprints: Vec<(u8, [u8; 4])> = cosigners
        .iter()
        .enumerate()
        .map(|(i, c)| (i as u8, c.master_fingerprint.to_bytes()))
        .collect();
    let pubkeys: Vec<(u8, [u8; 65])> = cosigners
        .iter()
        .enumerate()
        .map(|(i, c)| (i as u8, xpub_to_65(&c.xpub)))
        .collect();

    let descriptor = Descriptor {
        n: cosigner_count as u8,
        path_decl: PathDecl {
            n: cosigner_count as u8,
            paths: path_decl_paths,
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(fingerprints),
            pubkeys: Some(pubkeys),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };

    // 7. Policy id + stubs.
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);
    let stubs: Vec<[u8; 4]> = vec![stub; cosigner_count];

    // 8. Per-cosigner KeyCards + mk1.
    let mut per_cosigner: Vec<Vec<String>> = Vec::with_capacity(cosigner_count);
    for (i, c) in cosigners.iter().enumerate() {
        let card = mk_codec::KeyCard::new(
            stubs.clone(),
            if privacy_preserving {
                None
            } else {
                Some(c.master_fingerprint)
            },
            paths[i].clone(),
            c.xpub,
        );
        debug_assert_eq!(card.policy_id_stubs, stubs);
        debug_assert!(descriptor.is_wallet_policy());
        let csi = derive_mk1_chunk_set_id(&stubs[i]);
        let chunks = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;
        per_cosigner.push(chunks);
    }

    // 9. md1.
    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    Ok(Bundle {
        ms1: None,
        mk1: MkField::Multi(per_cosigner),
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
        let acc = derive_full(TREZOR_24, "", CliLanguage::English, network, template, 0).unwrap();
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
        let bundle = synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        assert!(bundle.ms1.is_some());
        let ms1 = bundle.ms1.as_ref().unwrap();
        assert!(ms1.starts_with("ms1"));
        let mk1 = bundle.mk1.as_single().unwrap();
        assert!(!mk1.is_empty());
        assert!(mk1.iter().all(|s| s.starts_with("mk1")));
        assert!(!bundle.md1.is_empty());
        assert!(bundle.md1.iter().all(|s| s.starts_with("md1")));
    }

    #[test]
    fn watch_only_bundle_omits_ms1() {
        let (_, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle =
            synthesize_watch_only(fp, xpub, CliTemplate::Bip84, CliNetwork::Mainnet, 0).unwrap();
        assert!(bundle.ms1.is_none());
        let mk1 = bundle.mk1.as_single().unwrap();
        assert!(!mk1.is_empty());
        assert!(mk1.iter().all(|s| s.starts_with("mk1")));
        assert!(!bundle.md1.is_empty());
        assert!(bundle.md1.iter().all(|s| s.starts_with("md1")));
    }

    #[test]
    fn mk1_chunk_set_id_is_deterministic_across_runs() {
        let (entropy, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let a = synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        let b = synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        assert_eq!(
            a.mk1.as_single().unwrap(),
            b.mk1.as_single().unwrap(),
            "mk1 must be byte-deterministic across runs"
        );
        assert_eq!(a.md1, b.md1, "md1 must be byte-deterministic across runs");
        assert_eq!(a.ms1, b.ms1, "ms1 must be byte-deterministic across runs");
    }

    #[test]
    fn cross_binding_holds_round_trip() {
        let (entropy, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle = synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();

        let mk1_v = bundle.mk1.as_single().unwrap();
        let mk1_strs: Vec<&str> = mk1_v.iter().map(|s| s.as_str()).collect();
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
    fn multisig_full_self_multisig_emits_n_card_sets_all_byte_identical() {
        use bip39::Mnemonic;
        let m = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let bundle = synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            3,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap();
        let multi = bundle.mk1.as_multi().expect("multisig must emit Multi");
        assert_eq!(multi.len(), 3, "3 cosigners → 3 card-sets");
        // Self-multisig: all N cards byte-identical (same xpub, same path, same csi).
        for i in 1..3 {
            assert_eq!(
                multi[0], multi[i],
                "self-multisig cards should be byte-identical"
            );
        }
        // Cross-binding round-trip via decode.
        let card_strs: Vec<&str> = multi[0].iter().map(|s| s.as_str()).collect();
        let decoded = mk_codec::decode(&card_strs).unwrap();
        assert_eq!(decoded.policy_id_stubs.len(), 3);
        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let desc = md_codec::chunk::reassemble(&md1_strs).unwrap();
        assert!(desc.is_wallet_policy());
        let pid = md_codec::compute_wallet_policy_id(&desc).unwrap();
        assert_eq!(&decoded.policy_id_stubs[0], &pid.as_bytes()[..4]);
    }

    #[test]
    fn multisig_watch_only_distinct_xpubs_emits_distinct_card_sets() {
        // Build 2 cosigners from 2 different seeds (different fp/xpub).
        let m1 = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let m2 = bip39::Mnemonic::parse_in(
            bip39::Language::English,
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
        )
        .unwrap();
        let secp = Secp256k1::new();

        let derive = |m: &bip39::Mnemonic, path_str: &str| -> CosignerSpec {
            let seed = m.to_seed("");
            let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
            let fp = master.fingerprint(&secp);
            let path = DerivationPath::from_str(path_str).unwrap();
            let xpriv = master.derive_priv(&secp, &path).unwrap();
            let xpub = Xpub::from_priv(&secp, &xpriv);
            CosignerSpec {
                xpub,
                master_fingerprint: fp,
                path: Some(path),
            }
        };

        let path_str = "m/87'/0'/0'";
        let cosigners = vec![derive(&m1, path_str), derive(&m2, path_str)];

        let bundle = synthesize_multisig_watch_only(
            &cosigners,
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap();
        let multi = bundle.mk1.as_multi().unwrap();
        assert_eq!(multi.len(), 2);
        assert_ne!(multi[0], multi[1], "distinct cosigners → distinct cards");

        // Round-trip both.
        for chunks in multi {
            let strs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
            let decoded = mk_codec::decode(&strs).unwrap();
            assert_eq!(decoded.policy_id_stubs.len(), 2);
        }
    }

    #[test]
    fn multisig_threshold_validation() {
        let m = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        // K = 0 rejected.
        let e = synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            0,
            3,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap_err();
        assert!(matches!(e, ToolkitError::MultisigConfig { .. }));
        // K > N rejected.
        let e = synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            5,
            3,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap_err();
        assert!(matches!(e, ToolkitError::MultisigConfig { .. }));
    }

    #[test]
    fn multisig_privacy_preserving_omits_fingerprints_in_mk1() {
        let m = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let bundle = synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            2,
            0,
            MultisigPathFamily::Bip87,
            true,
        )
        .unwrap();
        let multi = bundle.mk1.as_multi().unwrap();
        for chunks in multi {
            let strs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
            let decoded = mk_codec::decode(&strs).unwrap();
            assert!(
                decoded.origin_fingerprint.is_none(),
                "privacy-preserving mode should omit origin_fingerprint"
            );
        }
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
                let bundle = synthesize_full(&entropy, fp, xpub, t, n, 0).unwrap();
                let mk1_v = bundle.mk1.as_single().unwrap();
                let mk1_strs: Vec<&str> = mk1_v.iter().map(|s| s.as_str()).collect();
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
