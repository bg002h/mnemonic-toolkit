//! User-supplied BIP-388 descriptor parser (v0.3 §4.9).

// Phase A is incremental; some items are reachable only via tests until A.7
// wires the module into the bundle command. Lifted at end of Phase A.
#![allow(dead_code)]

use crate::error::ToolkitError;
use bitcoin::base58;
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint};
use bitcoin::hashes::{sha256, Hash};
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
use regex::Regex;
use std::collections::{BTreeMap, HashSet};
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

/// Resolved per-`@N` view (collapse repeated occurrences, validate dense `0..n`,
/// classify origin paths as Shared or Divergent). SPEC §4.9 step 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPlaceholders {
    pub n: u8,
    pub path_decl: PathDecl,
    pub fingerprint_annos: Vec<Option<Fingerprint>>,
    pub use_site_path: UseSitePath,
    pub use_site_path_overrides: Vec<(u8, UseSitePath)>,
}

/// Collapse repeated `@i` shapes, validate dense `0..n`, classify paths
/// Shared vs Divergent, and collect per-`@i` use-site path overrides
/// when `@i≥1` differs from `@0`'s use-site path.
pub fn resolve_placeholders(
    occs: &[PlaceholderOccurrence],
) -> Result<ResolvedPlaceholders, ToolkitError> {
    if occs.is_empty() {
        return Err(ToolkitError::BadInput(
            "descriptor must contain at least one @N placeholder.".into(),
        ));
    }
    let mut by_i: BTreeMap<u8, &PlaceholderOccurrence> = BTreeMap::new();
    for occ in occs {
        if let Some(prev) = by_i.get(&occ.i) {
            if prev.multipath_alts != occ.multipath_alts
                || prev.wildcard_hardened != occ.wildcard_hardened
                || prev.origin_path_anno != occ.origin_path_anno
                || prev.fingerprint_anno != occ.fingerprint_anno
            {
                return Err(ToolkitError::BadInput(format!(
                    "@{} appears with inconsistent path/multipath/hardening/fingerprint",
                    occ.i
                )));
            }
        } else {
            by_i.insert(occ.i, occ);
        }
    }
    let max_i = *by_i.keys().max().expect("non-empty after early return");
    let n = max_i
        .checked_add(1)
        .ok_or_else(|| ToolkitError::BadInput("@N index range exceeds u8".into()))?;
    for i in 0..n {
        if !by_i.contains_key(&i) {
            return Err(ToolkitError::BadInput(format!(
                "@{i} not present; placeholders must be dense 0..n"
            )));
        }
    }
    let at0 = by_i[&0];
    let use_site_path = make_use_site_path(at0);
    let mut use_site_path_overrides = Vec::new();
    for i in 1..n {
        let occ = by_i[&i];
        let usp_i = make_use_site_path(occ);
        if usp_i != use_site_path {
            use_site_path_overrides.push((i, usp_i));
        }
    }
    let all_paths_same = (0..n).all(|i| by_i[&i].origin_path_anno == at0.origin_path_anno);
    let paths = if all_paths_same {
        PathDeclPaths::Shared(to_origin_path(at0.origin_path_anno.as_ref()))
    } else {
        let v: Vec<OriginPath> = (0..n)
            .map(|i| to_origin_path(by_i[&i].origin_path_anno.as_ref()))
            .collect();
        PathDeclPaths::Divergent(v)
    };
    let path_decl = PathDecl { n, paths };
    let fingerprint_annos: Vec<Option<Fingerprint>> =
        (0..n).map(|i| by_i[&i].fingerprint_anno).collect();
    Ok(ResolvedPlaceholders {
        n,
        path_decl,
        fingerprint_annos,
        use_site_path,
        use_site_path_overrides,
    })
}

fn make_use_site_path(occ: &PlaceholderOccurrence) -> UseSitePath {
    let alts: Vec<Alternative> = occ
        .multipath_alts
        .iter()
        .map(|v| Alternative {
            hardened: false,
            value: *v,
        })
        .collect();
    UseSitePath {
        multipath: if alts.is_empty() { None } else { Some(alts) },
        wildcard_hardened: occ.wildcard_hardened,
    }
}

fn to_origin_path(p: Option<&DerivationPath>) -> OriginPath {
    let components = match p {
        None => Vec::new(),
        Some(dp) => dp
            .into_iter()
            .map(|c| match c {
                ChildNumber::Normal { index } => PathComponent {
                    hardened: false,
                    value: *index,
                },
                ChildNumber::Hardened { index } => PathComponent {
                    hardened: true,
                    value: *index,
                },
            })
            .collect(),
    };
    OriginPath { components }
}

/// Substitute every `@i[fp/path]/<multi>/*` token with a bare synthetic xpub.
/// The annotation/multipath/wildcard suffix is dropped — the structural walker
/// only needs the xpub identity. Annotations + use-site paths flow through
/// `ResolvedPlaceholders` separately. Returns `(substituted, key_map)` where
/// `key_map` maps synthetic-xpub-string → placeholder index `i`.
pub fn substitute_synthetic(
    descriptor: &str,
    ctx: ScriptCtx,
) -> Result<(String, BTreeMap<String, u8>), ToolkitError> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"@(\d+)(?:\[[0-9a-fA-F]{8}(?:/\d+(?:'|h)?)*\])?(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?")
            .expect("static regex compiles")
    });
    let mut key_map: BTreeMap<String, u8> = BTreeMap::new();
    let mut keys_seen: HashSet<u8> = HashSet::new();
    let mut bad: Option<ToolkitError> = None;
    let out = re
        .replace_all(descriptor, |caps: &regex::Captures| {
            if bad.is_some() {
                return String::new();
            }
            match caps[1].parse::<u8>() {
                Ok(i) => {
                    let xpub = synthetic_xpub_for(i, ctx);
                    if keys_seen.insert(i) {
                        key_map.insert(xpub.clone(), i);
                    }
                    xpub
                }
                Err(_) => {
                    bad = Some(ToolkitError::BadInput(format!(
                        "@i index out of range: @{}",
                        &caps[1]
                    )));
                    String::new()
                }
            }
        })
        .into_owned();
    if let Some(err) = bad {
        return Err(err);
    }
    Ok((out, key_map))
}

/// Strip optional `[fp/path]` prefix and `/derivation` suffix to recover the
/// bare xpub key as substituted.
fn lookup_key(key_str: &str, km: &BTreeMap<String, u8>) -> Result<u8, ToolkitError> {
    let after_bracket = key_str.find(']').map_or(key_str, |pos| &key_str[pos + 1..]);
    let base = after_bracket.split('/').next().unwrap_or(after_bracket);
    km.get(base).copied().ok_or_else(|| {
        ToolkitError::BadInput(format!(
            "internal: synthetic key {base} not found in key map (rendered: {key_str})"
        ))
    })
}

fn wrap_children(tag: Tag, inner: Node) -> Node {
    Node {
        tag,
        body: Body::Children(vec![inner]),
    }
}

fn build_multi_node(
    tag: Tag,
    k: usize,
    keys: &[&DescriptorPublicKey],
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    let children: Vec<Node> = keys
        .iter()
        .map(|kk| {
            let index = lookup_key(&kk.to_string(), km)?;
            Ok(Node {
                tag: Tag::PkK,
                body: Body::KeyArg { index },
            })
        })
        .collect::<Result<_, ToolkitError>>()?;
    Ok(Node {
        tag,
        body: Body::Variable {
            k: k as u8,
            children,
        },
    })
}

/// Walk the miniscript Descriptor's outermost wrapper into an `md_codec::Node`.
pub fn walk_root(
    desc: &MsDescriptor<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    use miniscript::Descriptor::*;
    match desc {
        Wpkh(w) => Ok(Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg {
                index: lookup_key(&w.as_inner().to_string(), km)?,
            },
        }),
        Pkh(p) => Ok(Node {
            tag: Tag::Pkh,
            body: Body::KeyArg {
                index: lookup_key(&p.as_inner().to_string(), km)?,
            },
        }),
        Wsh(w) => walk_wsh(w, km),
        Sh(s) => walk_sh(s, km),
        Tr(t) => walk_tr(t, km),
        Bare(_) => Err(ToolkitError::BadInput(
            "bare scripts are outside BIP-388 wallet-policy surface".into(),
        )),
    }
}

fn walk_wsh(
    w: &miniscript::descriptor::Wsh<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    let inner = walk_wsh_inner(w, km)?;
    Ok(wrap_children(Tag::Wsh, inner))
}

fn walk_wsh_inner(
    w: &miniscript::descriptor::Wsh<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    use miniscript::descriptor::WshInner;
    match w.as_inner() {
        WshInner::Ms(ms) => walk_miniscript_node(ms, km, /*tap=*/ false),
        WshInner::SortedMulti(sm) => build_multi_node(
            Tag::SortedMulti,
            sm.k(),
            &sm.pks().iter().collect::<Vec<_>>(),
            km,
        ),
    }
}

fn walk_sh(
    s: &miniscript::descriptor::Sh<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    use miniscript::descriptor::ShInner;
    let inner = match s.as_inner() {
        ShInner::Wsh(w) => Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![walk_wsh_inner(w, km)?]),
        },
        ShInner::Wpkh(wp) => Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg {
                index: lookup_key(&wp.as_inner().to_string(), km)?,
            },
        },
        ShInner::Ms(ms) => walk_miniscript_node(ms, km, /*tap=*/ false)?,
        ShInner::SortedMulti(sm) => build_multi_node(
            Tag::SortedMulti,
            sm.k(),
            &sm.pks().iter().collect::<Vec<_>>(),
            km,
        )?,
    };
    Ok(wrap_children(Tag::Sh, inner))
}

fn walk_tr(
    t: &miniscript::descriptor::Tr<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    let key_index = lookup_key(&t.internal_key().to_string(), km)?;
    let tree: Option<Box<Node>> = match t.tap_tree() {
        None => None,
        Some(tt) => Some(Box::new(walk_tap_tree_singleleaf(tt, km)?)),
    };
    Ok(Node {
        tag: Tag::Tr,
        body: Body::Tr { key_index, tree },
    })
}

/// v0.3 supports 0 or 1 leaves. Multi-leaf taproot trees are deferred to v0.4
/// per SPEC §6.8.
fn walk_tap_tree_singleleaf(
    tt: &miniscript::descriptor::TapTree<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    let leaves: Vec<_> = tt.leaves().collect();
    match leaves.len() {
        0 => Err(ToolkitError::BadInput(
            "tap tree present but contains no leaves".into(),
        )),
        1 => walk_miniscript_node(leaves[0].miniscript(), km, /*tap=*/ true),
        n => Err(ToolkitError::BadInput(format!(
            "tap tree with {n} leaves not supported in v0.3 (single-leaf only; multi-leaf deferred to v0.4)"
        ))),
    }
}

/// Walk a miniscript AST node into `md_codec::Node`. v0.3 covers the BIP-388
/// surface modulo deferred items (sortedmulti_a in tap leaves, multi-leaf tap).
/// A.4 ships PkK/PkH/Multi/MultiA/Check (carries from md-cli); A.6 lands the
/// 23 v0.3-NEW Layer 2 arms.
fn walk_miniscript_node<C: miniscript::ScriptContext>(
    ms: &miniscript::Miniscript<DescriptorPublicKey, C>,
    km: &BTreeMap<String, u8>,
    tap_context: bool,
) -> Result<Node, ToolkitError> {
    use miniscript::miniscript::decode::Terminal;
    match &ms.node {
        Terminal::PkK(k) => Ok(Node {
            tag: Tag::PkK,
            body: Body::KeyArg {
                index: lookup_key(&k.to_string(), km)?,
            },
        }),
        Terminal::PkH(k) => Ok(Node {
            tag: Tag::PkH,
            body: Body::KeyArg {
                index: lookup_key(&k.to_string(), km)?,
            },
        }),
        Terminal::Multi(thresh) => build_multi_node(
            Tag::Multi,
            thresh.k(),
            &thresh.data().iter().collect::<Vec<_>>(),
            km,
        ),
        Terminal::MultiA(thresh) => build_multi_node(
            Tag::MultiA,
            thresh.k(),
            &thresh.data().iter().collect::<Vec<_>>(),
            km,
        ),
        Terminal::Check(inner) => {
            if tap_context {
                if let Terminal::PkK(k) = &inner.node {
                    return Ok(Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg {
                            index: lookup_key(&k.to_string(), km)?,
                        },
                    });
                }
                if let Terminal::PkH(k) = &inner.node {
                    return Ok(Node {
                        tag: Tag::PkH,
                        body: Body::KeyArg {
                            index: lookup_key(&k.to_string(), km)?,
                        },
                    });
                }
            }
            Ok(Node {
                tag: Tag::Check,
                body: Body::Children(vec![walk_miniscript_node(inner, km, tap_context)?]),
            })
        }
        _ => Err(ToolkitError::BadInput(format!(
            "unsupported miniscript fragment: {ms}; v0.3 walker covers BIP-388 surface modulo multi-leaf tap trees (deferred to v0.4)"
        ))),
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

    // ---- A.3: resolve_placeholders ----

    #[test]
    fn resolve_dense_two_shared_paths() {
        let occs = lex_placeholders(
            "wsh(multi(2,@0[deadbeef/48'/0'/0'/2']/<0;1>/*,@1[cafef00d/48'/0'/0'/2']/<0;1>/*))",
        )
        .unwrap();
        let r = resolve_placeholders(&occs).unwrap();
        assert_eq!(r.n, 2);
        assert!(matches!(r.path_decl.paths, PathDeclPaths::Shared(_)));
        assert!(r.use_site_path_overrides.is_empty());
        assert_eq!(r.fingerprint_annos.len(), 2);
        assert!(r.fingerprint_annos[0].is_some());
        assert!(r.fingerprint_annos[1].is_some());
    }

    #[test]
    fn resolve_divergent_paths() {
        let occs = lex_placeholders(
            "wsh(multi(2,@0[deadbeef/48'/0'/0'/2']/<0;1>/*,@1[cafef00d/48'/1'/0'/2']/<0;1>/*))",
        )
        .unwrap();
        let r = resolve_placeholders(&occs).unwrap();
        assert_eq!(r.n, 2);
        assert!(matches!(r.path_decl.paths, PathDeclPaths::Divergent(_)));
    }

    #[test]
    fn resolve_rejects_gap() {
        // @0 and @2 with no @1 — must error
        let occs = lex_placeholders("wsh(multi(2,@0/<0;1>/*,@2/<0;1>/*))").unwrap();
        let err = resolve_placeholders(&occs).unwrap_err();
        let msg = err.message();
        assert!(msg.contains("dense") || msg.contains("@1"), "got: {msg}");
    }

    #[test]
    fn resolve_collects_use_site_overrides() {
        let occs = lex_placeholders("wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))").unwrap();
        let r = resolve_placeholders(&occs).unwrap();
        assert_eq!(r.n, 2);
        assert_eq!(r.use_site_path_overrides.len(), 1);
        assert_eq!(r.use_site_path_overrides[0].0, 1);
    }

    #[test]
    fn resolve_collapses_repeated_at_i() {
        // Multipath descriptors expand `@0/<0;1>/*` to two occurrences of @0
        // when the regex doesn't fully fold them. resolve_placeholders must
        // collapse same-i shapes to one slot.
        let occ = PlaceholderOccurrence {
            i: 0,
            fingerprint_anno: None,
            origin_path_anno: None,
            multipath_alts: vec![0, 1],
            wildcard_hardened: false,
        };
        let r = resolve_placeholders(&[occ.clone(), occ]).unwrap();
        assert_eq!(r.n, 1);
    }

    // ---- A.4: walk_root Layer 1 dispatch (10 round-trips) ----

    fn parse_and_walk(template: &str, ctx: ScriptCtx) -> Node {
        let (s, km) = substitute_synthetic(template, ctx).unwrap();
        let d = MsDescriptor::<DescriptorPublicKey>::from_str(&s).unwrap();
        walk_root(&d, &km).unwrap()
    }

    #[test]
    fn walk_wpkh_root() {
        let root = parse_and_walk("wpkh(@0/<0;1>/*)", ScriptCtx::SingleSig);
        assert_eq!(root.tag, Tag::Wpkh);
        assert!(matches!(root.body, Body::KeyArg { index: 0 }));
    }

    #[test]
    fn walk_pkh_root() {
        let root = parse_and_walk("pkh(@0/<0;1>/*)", ScriptCtx::SingleSig);
        assert_eq!(root.tag, Tag::Pkh);
        assert!(matches!(root.body, Body::KeyArg { index: 0 }));
    }

    #[test]
    fn walk_wsh_pk_root() {
        // `pk(K)` desugars to `c:pk_k(K)` in non-tap context → Wsh wrapping Check wrapping PkK.
        let root = parse_and_walk("wsh(pk(@0/<0;1>/*))", ScriptCtx::MultiSig);
        assert_eq!(root.tag, Tag::Wsh);
        let Body::Children(children) = &root.body else {
            panic!("expected Wsh+Children");
        };
        assert_eq!(children[0].tag, Tag::Check);
    }

    #[test]
    fn walk_wsh_sortedmulti_root() {
        let root = parse_and_walk(
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            ScriptCtx::MultiSig,
        );
        assert_eq!(root.tag, Tag::Wsh);
        let Body::Children(children) = &root.body else {
            panic!("expected Wsh+Children");
        };
        assert_eq!(children[0].tag, Tag::SortedMulti);
        let Body::Variable { k, children: subs } = &children[0].body else {
            panic!("expected SortedMulti Variable body");
        };
        assert_eq!(*k, 2);
        assert_eq!(subs.len(), 2);
    }

    #[test]
    fn walk_sh_wpkh_root() {
        let root = parse_and_walk("sh(wpkh(@0/<0;1>/*))", ScriptCtx::SingleSig);
        assert_eq!(root.tag, Tag::Sh);
        let Body::Children(children) = &root.body else {
            panic!("expected Sh+Children");
        };
        assert_eq!(children[0].tag, Tag::Wpkh);
    }

    #[test]
    fn walk_sh_wsh_pk_root() {
        let root = parse_and_walk("sh(wsh(pk(@0/<0;1>/*)))", ScriptCtx::MultiSig);
        assert_eq!(root.tag, Tag::Sh);
        let Body::Children(children) = &root.body else {
            panic!("expected Sh+Children");
        };
        assert_eq!(children[0].tag, Tag::Wsh);
    }

    #[test]
    fn walk_sh_sortedmulti_root() {
        let root = parse_and_walk(
            "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            ScriptCtx::MultiSig,
        );
        assert_eq!(root.tag, Tag::Sh);
        let Body::Children(children) = &root.body else {
            panic!("expected Sh+Children");
        };
        assert_eq!(children[0].tag, Tag::SortedMulti);
    }

    #[test]
    fn walk_sh_ms_pk_root() {
        // sh wrapping bare miniscript (not via wsh) — tests the ShInner::Ms branch.
        let root = parse_and_walk("sh(pk(@0/<0;1>/*))", ScriptCtx::SingleSig);
        assert_eq!(root.tag, Tag::Sh);
        let Body::Children(children) = &root.body else {
            panic!("expected Sh+Children");
        };
        assert_eq!(children[0].tag, Tag::Check);
    }

    #[test]
    fn walk_tr_keypath_root() {
        let root = parse_and_walk("tr(@0/<0;1>/*)", ScriptCtx::SingleSig);
        assert_eq!(root.tag, Tag::Tr);
        let Body::Tr { key_index, tree } = &root.body else {
            panic!("expected Tr body");
        };
        assert_eq!(*key_index, 0);
        assert!(tree.is_none());
    }

    #[test]
    fn walk_tr_singleleaf_multi_a_root() {
        let root = parse_and_walk(
            "tr(@0/<0;1>/*,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
            ScriptCtx::MultiSig,
        );
        assert_eq!(root.tag, Tag::Tr);
        let Body::Tr { key_index: _, tree } = &root.body else {
            panic!("expected Tr body");
        };
        let leaf = tree.as_ref().expect("expected single tap leaf");
        assert_eq!(leaf.tag, Tag::MultiA);
    }

    #[test]
    fn resolve_rejects_inconsistent_shape() {
        let occ_a = PlaceholderOccurrence {
            i: 0,
            fingerprint_anno: None,
            origin_path_anno: None,
            multipath_alts: vec![0, 1],
            wildcard_hardened: false,
        };
        let occ_b = PlaceholderOccurrence {
            i: 0,
            fingerprint_anno: None,
            origin_path_anno: None,
            multipath_alts: vec![2, 3], // differs!
            wildcard_hardened: false,
        };
        let err = resolve_placeholders(&[occ_a, occ_b]).unwrap_err();
        assert!(err.message().contains("inconsistent"));
    }
}
