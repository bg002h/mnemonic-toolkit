//! 73-byte compact xpub form per `design/SPEC_mk_v0_1.md` §3.6
//! (closure Q-7).
//!
//! Drops `xpub.depth` and `xpub.child_number` from the wire (both
//! reconstructible from `origin_path`); preserves `xpub.version`,
//! `xpub.parent_fingerprint`, `xpub.chain_code`, `xpub.public_key`.
//!
//! ```text
//! [version          : 4 B]
//! [parent_fingerprint: 4 B]
//! [chain_code       : 32 B]
//! [public_key       : 33 B]
//!                     ────
//!                     73 B
//! ```

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::PublicKey;

use crate::consts::XPUB_COMPACT_BYTES;
use crate::error::{Error, Result};

/// Mainnet xpub version prefix (`xpub`).
const MAINNET_XPUB_VERSION: [u8; 4] = [0x04, 0x88, 0xB2, 0x1E];

/// Testnet xpub version prefix (`tpub`).
const TESTNET_XPUB_VERSION: [u8; 4] = [0x04, 0x35, 0x87, 0xCF];

/// 73-byte compact form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XpubCompact {
    /// 4-byte BIP 32 version prefix.
    pub version: [u8; 4],
    /// 4-byte parent-key fingerprint.
    pub parent_fingerprint: [u8; 4],
    /// 32-byte BIP 32 chain code.
    pub chain_code: [u8; 32],
    /// 33-byte compressed secp256k1 public key.
    pub public_key: [u8; 33],
}

impl XpubCompact {
    /// Build a compact form from a full BIP 32 `Xpub`.
    pub fn from_xpub(xpub: &Xpub) -> Self {
        let version = network_to_version(xpub.network);
        XpubCompact {
            version,
            parent_fingerprint: xpub.parent_fingerprint.to_bytes(),
            chain_code: xpub.chain_code.to_bytes(),
            public_key: xpub.public_key.serialize(),
        }
    }
}

fn network_to_version(network: NetworkKind) -> [u8; 4] {
    match network {
        NetworkKind::Main => MAINNET_XPUB_VERSION,
        NetworkKind::Test => TESTNET_XPUB_VERSION,
    }
}

fn version_to_network(version: [u8; 4]) -> Result<NetworkKind> {
    match version {
        MAINNET_XPUB_VERSION => Ok(NetworkKind::Main),
        TESTNET_XPUB_VERSION => Ok(NetworkKind::Test),
        other => Err(Error::InvalidXpubVersion(u32::from_be_bytes(other))),
    }
}

/// Reconstruct a full BIP 32 `Xpub` from a compact form + the origin
/// path (which provides depth and child_number per Q-7's reconstruction
/// rule).
///
/// Per `design/SPEC_mk_v0_1.md` §3.6:
///
/// ```text
/// depth        := component_count(origin_path)
/// child_number := last_component(origin_path) (with hardened-bit encoding),
///                 or Normal{0} when origin_path is empty (depth-0 / no-path key)
/// ```
///
/// An empty `origin_path` (the no-path / depth-0 case, e.g. a WIF) yields
/// `depth = 0` and `child_number = Normal{0}` (the BIP-32 master
/// convention) — v0.4.0+; earlier versions required a non-empty path.
pub fn reconstruct_xpub(compact: &XpubCompact, origin_path: &DerivationPath) -> Result<Xpub> {
    let network = version_to_network(compact.version)?;
    let components: Vec<ChildNumber> = origin_path.into_iter().copied().collect();
    let depth = components.len() as u8;
    // child_number defaults to the BIP-32 master convention Normal{0} when
    // origin_path is empty (a depth-0 / no-path key, e.g. a WIF — SPEC §3.6).
    // For a non-empty path it is the terminal component; this is the exact
    // inverse of the encode-side guard in encode.rs.
    let child_number = components
        .last()
        .copied()
        .unwrap_or(ChildNumber::Normal { index: 0 });
    let public_key = PublicKey::from_slice(&compact.public_key)
        .map_err(|e| Error::InvalidXpubPublicKey(format!("{e}")))?;
    Ok(Xpub {
        network,
        depth,
        parent_fingerprint: Fingerprint::from(compact.parent_fingerprint),
        child_number,
        public_key,
        chain_code: ChainCode::from(compact.chain_code),
    })
}

/// Encode a compact form to its 73-byte wire layout.
pub fn encode_xpub_compact(compact: &XpubCompact, out: &mut Vec<u8>) {
    out.extend_from_slice(&compact.version);
    out.extend_from_slice(&compact.parent_fingerprint);
    out.extend_from_slice(&compact.chain_code);
    out.extend_from_slice(&compact.public_key);
}

/// Decode 73 bytes into a compact form.
pub fn decode_xpub_compact(cursor: &mut &[u8]) -> Result<XpubCompact> {
    if cursor.len() < XPUB_COMPACT_BYTES {
        return Err(Error::UnexpectedEnd);
    }
    let version: [u8; 4] = cursor[0..4].try_into().unwrap();
    // Validate version eagerly so the error fires here rather than at
    // reconstruction time.
    let _ = version_to_network(version)?;
    let parent_fingerprint: [u8; 4] = cursor[4..8].try_into().unwrap();
    let chain_code: [u8; 32] = cursor[8..40].try_into().unwrap();
    let public_key: [u8; 33] = cursor[40..73].try_into().unwrap();
    *cursor = &cursor[XPUB_COMPACT_BYTES..];
    Ok(XpubCompact {
        version,
        parent_fingerprint,
        chain_code,
        public_key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::test_helpers::synthetic_xpub;
    use std::str::FromStr;

    #[test]
    fn round_trip_full_xpub_depth_4() {
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        let xpub_full = synthetic_xpub(&path);
        let compact = XpubCompact::from_xpub(&xpub_full);
        // Compact must drop depth and child_number — verify by length only.
        let mut wire = Vec::new();
        encode_xpub_compact(&compact, &mut wire);
        assert_eq!(wire.len(), XPUB_COMPACT_BYTES);
        // Round-trip on the wire form.
        let mut cursor: &[u8] = &wire;
        let decoded = decode_xpub_compact(&mut cursor).unwrap();
        assert_eq!(decoded, compact);
        assert!(cursor.is_empty());
        // Reconstruct with the path the xpub was originally derived at.
        let reconstructed = reconstruct_xpub(&decoded, &path).unwrap();
        assert_eq!(reconstructed.depth, 4);
        assert_eq!(reconstructed.network, xpub_full.network);
        assert_eq!(
            reconstructed.parent_fingerprint,
            xpub_full.parent_fingerprint
        );
        assert_eq!(reconstructed.chain_code, xpub_full.chain_code);
        assert_eq!(reconstructed.public_key, xpub_full.public_key);
        // child_number reconstruction
        assert_eq!(reconstructed.child_number, xpub_full.child_number);
    }

    #[test]
    fn reconstruct_depth0_empty_path() {
        let path = DerivationPath::from_str("m").unwrap(); // empty
        let xpub_full = synthetic_xpub(&path); // depth 0, child Normal{0}
        let compact = XpubCompact::from_xpub(&xpub_full);
        let reconstructed = reconstruct_xpub(&compact, &path).unwrap();
        assert_eq!(reconstructed.depth, 0);
        assert_eq!(reconstructed.child_number, ChildNumber::Normal { index: 0 });
        assert_eq!(
            reconstructed.parent_fingerprint,
            xpub_full.parent_fingerprint
        );
        assert_eq!(reconstructed.chain_code, xpub_full.chain_code);
        assert_eq!(reconstructed.public_key, xpub_full.public_key);
        assert_eq!(reconstructed.network, xpub_full.network);
    }

    #[test]
    fn rejects_invalid_version() {
        // 73 bytes with garbage version
        let mut wire = vec![0xDE, 0xAD, 0xBE, 0xEF];
        wire.extend_from_slice(&[0u8; 4 + 32 + 33]);
        let mut cursor: &[u8] = &wire;
        assert!(matches!(
            decode_xpub_compact(&mut cursor),
            Err(Error::InvalidXpubVersion(_)),
        ));
    }

    #[test]
    fn rejects_truncated_input() {
        let wire = vec![0x04, 0x88]; // way under 73
        let mut cursor: &[u8] = &wire;
        assert!(matches!(
            decode_xpub_compact(&mut cursor),
            Err(Error::UnexpectedEnd),
        ));
    }
}
