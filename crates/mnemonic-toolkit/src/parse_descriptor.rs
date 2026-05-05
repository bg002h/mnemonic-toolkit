//! User-supplied BIP-388 descriptor parser (v0.3 §4.9).

// Phase A is incremental; some items are reachable only via tests until A.7
// wires the module into the bundle command. Lifted at end of Phase A.
#![allow(dead_code)]

use crate::error::ToolkitError;
use bitcoin::base58;
use bitcoin::bip32::{DerivationPath, Fingerprint};
use bitcoin::hashes::{sha256, Hash};
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use regex::Regex;
use std::str::FromStr;
use std::sync::OnceLock;

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

/// One occurrence of `@N[fp/path]/<multipath>/*` in the raw descriptor.
/// Exit-code mapping for `lex_placeholders` errors is revisited in Phase B
/// (currently routes through `BadInput` → exit 1; SPEC §6.7/§6.9 wants exit 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceholderOccurrence {
    pub i: u8,
    pub fingerprint_anno: Option<Fingerprint>,
    pub origin_path_anno: Option<DerivationPath>,
    pub multipath_alts: Vec<u32>,
    pub wildcard_hardened: bool,
}

/// Lex every `@N[fp/path]/<multipath>/*` occurrence (SPEC §4.9 step 2).
/// Empty result is rejected with the SPEC §6.9 byte-exact message.
pub fn lex_placeholders(descriptor: &str) -> Result<Vec<PlaceholderOccurrence>, ToolkitError> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        // Captures:
        //   1: @N index
        //   2: 8-hex fingerprint inside `[...]`
        //   3: origin path inside `[...]` (may be empty if `[fp]` alone)
        //   4: multipath alts (semicolon-separated digits)
        //   5: wildcard suffix (`/*`, `/*'`, `/*h`)
        Regex::new(
            r"@(\d+)(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?(?:/<([0-9;]+)>)?(/\*(?:'|h)?)?",
        )
        .expect("static regex compiles")
    });
    let mut out = Vec::new();
    for caps in re.captures_iter(descriptor) {
        let i: u8 = caps[1]
            .parse()
            .map_err(|_| ToolkitError::BadInput(format!("@i index out of range: @{}", &caps[1])))?;
        let fingerprint_anno = caps
            .get(2)
            .map(|m| {
                Fingerprint::from_str(m.as_str()).map_err(|e| {
                    ToolkitError::BadInput(format!(
                        "@{i} fingerprint annotation `{}`: {e}",
                        m.as_str()
                    ))
                })
            })
            .transpose()?;
        let origin_path_anno = caps
            .get(3)
            .and_then(|m| {
                let s = m.as_str();
                if s.is_empty() {
                    None
                } else {
                    Some(s.trim_start_matches('/').to_string())
                }
            })
            .map(|s| {
                DerivationPath::from_str(&s).map_err(|e| {
                    ToolkitError::BadInput(format!("@{i} origin path annotation `{s}`: {e}"))
                })
            })
            .transpose()?;
        let multipath_alts = caps
            .get(4)
            .map(|m| {
                m.as_str()
                    .split(';')
                    .map(|n| {
                        n.parse::<u32>().map_err(|_| {
                            ToolkitError::BadInput(format!("@{i} multipath alt `{n}` is not u32"))
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        let wildcard_hardened = caps
            .get(5)
            .map(|m| m.as_str().ends_with('\'') || m.as_str().ends_with('h'))
            .unwrap_or(false);
        out.push(PlaceholderOccurrence {
            i,
            fingerprint_anno,
            origin_path_anno,
            multipath_alts,
            wildcard_hardened,
        });
    }
    if out.is_empty() {
        return Err(ToolkitError::BadInput(
            "descriptor must contain at least one @N placeholder.".into(),
        ));
    }
    Ok(out)
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

    // ---- A.2: lex_placeholders ----

    #[test]
    fn lex_bare_at_zero() {
        let occs = lex_placeholders("wpkh(@0)").unwrap();
        assert_eq!(occs.len(), 1);
        assert_eq!(occs[0].i, 0);
        assert_eq!(occs[0].fingerprint_anno, None);
        assert_eq!(occs[0].origin_path_anno, None);
        assert!(occs[0].multipath_alts.is_empty());
        assert!(!occs[0].wildcard_hardened);
    }

    #[test]
    fn lex_with_multipath_and_wildcard() {
        let occs = lex_placeholders("wpkh(@0/<0;1>/*)").unwrap();
        assert_eq!(occs.len(), 1);
        assert_eq!(occs[0].i, 0);
        assert_eq!(occs[0].multipath_alts, vec![0, 1]);
        assert!(!occs[0].wildcard_hardened);
        assert_eq!(occs[0].fingerprint_anno, None);
    }

    #[test]
    fn lex_with_full_annotation() {
        let occs =
            lex_placeholders("wsh(sortedmulti(2,@0[deadbeef/48'/0'/0'/2']/<0;1>/*,@1[cafef00d/48'/0'/0'/2']/<0;1>/*))")
                .unwrap();
        assert_eq!(occs.len(), 2);
        assert_eq!(occs[0].i, 0);
        assert_eq!(
            occs[0].fingerprint_anno,
            Some(Fingerprint::from_str("deadbeef").unwrap())
        );
        assert_eq!(
            occs[0].origin_path_anno.as_ref().map(|p| p.to_string()),
            Some("48'/0'/0'/2'".to_string())
        );
        assert_eq!(occs[0].multipath_alts, vec![0, 1]);
        assert!(!occs[0].wildcard_hardened);
        assert_eq!(occs[1].i, 1);
        assert_eq!(
            occs[1].fingerprint_anno,
            Some(Fingerprint::from_str("cafef00d").unwrap())
        );
    }

    #[test]
    fn lex_hardened_wildcard() {
        let occs = lex_placeholders("wpkh(@0/<0;1>/*')").unwrap();
        assert_eq!(occs.len(), 1);
        assert!(occs[0].wildcard_hardened);
    }

    #[test]
    fn lex_rejects_no_placeholders() {
        let err = lex_placeholders("wpkh(xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ/0/*)")
            .unwrap_err();
        let msg = err.message();
        assert!(
            msg.contains("at least one @N placeholder"),
            "expected SPEC §6.9 message, got: {msg}"
        );
    }

    #[test]
    fn lex_rejects_index_overflow() {
        // u8 max is 255; @256 must error.
        let err = lex_placeholders("wpkh(@256/<0;1>/*)").unwrap_err();
        let msg = err.message();
        assert!(
            msg.contains("@i index out of range") || msg.contains("@256"),
            "expected index-out-of-range message, got: {msg}"
        );
    }
}
