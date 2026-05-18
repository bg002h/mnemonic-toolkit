//! BIP-32 path search primitive — derive child xpubs from a master xprv
//! and byte-equal-compare to a target xpub.
//!
//! `match_xpub_against_paths` iterates the candidate set in order, derives
//! the child xpub at each path, and returns the first match. The 65-byte
//! `synthesize::xpub_to_65` projection drives the equality check
//! (chain_code + compressed pubkey) — robust against the SLIP-0132
//! version-byte differences in input xpubs (the target is already
//! canonicalized at this point but the byte-form is the unambiguous identity).

use super::candidate_paths::CandidatePath;
use crate::synthesize::xpub_to_65;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;

/// One matched candidate: template + full path + (optional) account index.
#[derive(Debug, Clone)]
pub struct MatchedPath {
    pub template_name: String,
    pub path: DerivationPath,
    pub account: Option<u32>,
}

/// Walk `candidates` in order; derive the child xpub at each path; return
/// the first byte-equal match to `target_xpub_65` (in the 65-byte form).
pub fn match_xpub_against_paths(
    master_xprv: &Xpriv,
    candidates: &[CandidatePath],
    target_xpub_65: &[u8; 65],
) -> Option<MatchedPath> {
    let secp = Secp256k1::new();
    for c in candidates {
        match master_xprv.derive_priv(&secp, &c.path) {
            Ok(child_xpriv) => {
                let child_xpub = Xpub::from_priv(&secp, &child_xpriv);
                let child_65 = xpub_to_65(&child_xpub);
                if &child_65 == target_xpub_65 {
                    return Some(MatchedPath {
                        template_name: c.template_name.clone(),
                        path: c.path.clone(),
                        account: c.account,
                    });
                }
            }
            Err(_) => continue,
        }
    }
    None
}
