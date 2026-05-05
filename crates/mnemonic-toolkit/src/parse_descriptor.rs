//! User-supplied BIP-388 descriptor parser (v0.3 §4.9).

// Phase A is incremental; some items are reachable only via tests until A.7
// wires the module into the bundle command. Lifted at end of Phase A.
#![allow(dead_code)]

use bitcoin::base58;
use bitcoin::hashes::{sha256, Hash};
use bitcoin::secp256k1::{Secp256k1, SecretKey};

const SEED_PREFIX: &[u8] = b"toolkit-v0.3";
const MAINNET_XPUB_VERSION: [u8; 4] = [0x04, 0x88, 0xB2, 0x1E];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScriptCtx {
    SingleSig,
    MultiSig,
}

impl ScriptCtx {
    fn depth(self) -> u8 {
        match self {
            ScriptCtx::SingleSig => 3,
            ScriptCtx::MultiSig => 4,
        }
    }
}

/// Synthetic xpub for placeholder `@i` under `ctx`. Deterministic; never wire-emitted.
/// Seed prefix `b"toolkit-v0.3"` is normative — fixture stability depends on it.
pub fn synthetic_xpub_for(i: u8, ctx: ScriptCtx) -> String {
    let depth = ctx.depth();
    let mut seed_buf = Vec::with_capacity(SEED_PREFIX.len() + 2);
    seed_buf.extend_from_slice(SEED_PREFIX);
    seed_buf.push(i);
    seed_buf.push(depth);
    let seed = sha256::Hash::hash(&seed_buf);
    let chain_code = sha256::Hash::hash(&[b'c', b'c', i, depth]).to_byte_array();
    let secret = SecretKey::from_slice(&seed.to_byte_array()).expect("hash is valid scalar");
    let pubkey = secret.public_key(&Secp256k1::new()).serialize();
    let mut bytes = [0u8; 78];
    bytes[0..4].copy_from_slice(&MAINNET_XPUB_VERSION);
    bytes[4] = depth;
    bytes[13..45].copy_from_slice(&chain_code);
    bytes[45..78].copy_from_slice(&pubkey);
    base58::encode_check(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_xpub_0_singlesig_pinned() {
        assert_eq!(
            synthetic_xpub_for(0, ScriptCtx::SingleSig),
            "xpub6BemYiVEULcbqF34sTQgz3c2MzCoNmz8ZJieEwjH6HwnZ54tYQmnFgEwRckq3hLJ9feTr4xUFx7XwJ3nraRrQcPnvEuYfddWQ8A4kwU4QMx",
        );
    }

    #[test]
    fn synthetic_xpub_0_multisig_pinned() {
        assert_eq!(
            synthetic_xpub_for(0, ScriptCtx::MultiSig),
            "xpub6DXuQW1FgeHbfmexToxaz2g1mAAGf1sV2Kd38U6yKMU6oqgww1T3rFuHqLJTyob4TkBpEi7h1Asp9UCh5uPWp1yPMpZjdoh5QXXDBPo19ky",
        );
    }

    #[test]
    fn synthetic_xpub_is_deterministic() {
        assert_eq!(
            synthetic_xpub_for(0, ScriptCtx::MultiSig),
            synthetic_xpub_for(0, ScriptCtx::MultiSig),
        );
    }

    #[test]
    fn synthetic_xpub_varies_by_index_and_ctx() {
        let a = synthetic_xpub_for(0, ScriptCtx::SingleSig);
        let b = synthetic_xpub_for(0, ScriptCtx::MultiSig);
        let c = synthetic_xpub_for(1, ScriptCtx::SingleSig);
        assert_ne!(a, b, "ctx must affect output (depth byte differs)");
        assert_ne!(a, c, "index must affect output");
    }
}
