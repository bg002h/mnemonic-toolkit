//! User-supplied BIP-388 descriptor parser (v0.3 §4.9).
//!
//! Phase C.6 wired `descriptor_mode_run` to the full pipeline; module-level
//! `#![allow(dead_code)]` was lifted at end-of-phase C per FOLLOWUP
//! `parse-descriptor-allow-dead-code-audit`. Items reachable only from tests
//! carry an inline `#[allow(dead_code)]`.

use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::CosignerSpec;
use crate::synthesize::{xpub_to_65, CosignerKeyInfo};
use bitcoin::base58;
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::hashes::{sha256, Hash};
use bitcoin::secp256k1::{Secp256k1, SecretKey};
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};
use md_codec::{Descriptor as MdDescriptor, TlvSection};
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
/// Lex/resolve/walk errors route through `ToolkitError::DescriptorParse`
/// (exit 2) per SPEC §6.7 (Phase B.0 migrated from `BadInput`).
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
        let i: u8 = caps[1].parse().map_err(|_| {
            ToolkitError::DescriptorParse(format!("@i index out of range: @{}", &caps[1]))
        })?;
        let fingerprint_anno = caps
            .get(2)
            .map(|m| {
                Fingerprint::from_str(m.as_str()).map_err(|e| {
                    ToolkitError::DescriptorParse(format!(
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
                    ToolkitError::DescriptorParse(format!("@{i} origin path annotation `{s}`: {e}"))
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
                            ToolkitError::DescriptorParse(format!(
                                "@{i} multipath alt `{n}` is not u32"
                            ))
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
        return Err(ToolkitError::DescriptorParse(
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
        return Err(ToolkitError::DescriptorParse(
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
                return Err(ToolkitError::DescriptorParse(format!(
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
        .ok_or_else(|| ToolkitError::DescriptorParse("@N index range exceeds u8".into()))?;
    for i in 0..n {
        if !by_i.contains_key(&i) {
            return Err(ToolkitError::DescriptorParse(format!(
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

/// BIP-341 NUMS H-point as x-only hex (32 bytes). Used by:
/// (1) `substitute_nums_sentinel` to replace the user-facing `NUMS` token,
/// (2) `walk_tr` to detect the NUMS internal key and set `Body::Tr.is_nums = true`.
/// Mirrors md-codec's `NUMS_H_POINT_X_ONLY_HEX` (`to_miniscript.rs`).
pub const NUMS_H_POINT_X_ONLY_HEX: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// v0.19.0 SPEC §4.12.e — substitute the literal token `NUMS` appearing as
/// the first argument of `tr(...)` with the BIP-341 unspendable-key hex.
///
/// Word-boundary regex `tr\(NUMS\b` ensures only exact-token matches replace
/// (e.g., `tr(NUMSOMETHING...)` is NOT substituted; `tr(NUMS,...)` and
/// `tr(NUMS)` both are). Called at the top of `parse_descriptor` BEFORE
/// `lex_placeholders` AND `substitute_synthetic` so the substituted form
/// flows through the entire pipeline.
///
/// Returns the input unchanged if no `tr(NUMS` token is present.
pub fn substitute_nums_sentinel(input: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"tr\(NUMS\b").expect("static regex compiles"));
    re.replace_all(input, format!("tr({NUMS_H_POINT_X_ONLY_HEX}").as_str())
        .into_owned()
}

/// v0.19.0 SPEC §6.6 row 16 — detect a bare `tr(<miniscript>)` (no internal
/// key) by matching `tr(` followed by a lowercase-identifier followed by `(`
/// (i.e., a miniscript-fragment function call like `andor`, `and_v`, `pk`,
/// `pkh`, `multi`, `thresh`, etc.).
///
/// Valid internal-key forms are NOT matched: hex pubkey (`tr(0x...)`),
/// xpub (`tr(xpub6...)` — `xpub` is lowercase but followed by digits not
/// `(`), placeholder (`tr(@N)`), annotated key (`tr([fp/path]@N)`), or
/// NUMS-substituted hex (`tr(50929b...)`).
///
/// Run AFTER `substitute_nums_sentinel` (so the NUMS hex form bypasses the
/// detector) but BEFORE rust-miniscript's `MsDescriptor::from_str` so the
/// toolkit emits the SPEC §6.6 row 16 byte-exact text instead of
/// rust-miniscript's lower-level parse error.
pub fn detect_bare_tr(input: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^tr\([a-z][a-z_0-9]*\(").expect("static regex compiles")
    });
    re.is_match(input)
}

/// SPEC §6.6 row 16 byte-exact stderr text. Used by `parse_descriptor` to
/// refuse bare `tr(<miniscript>)` with no internal key. NOT including the
/// `error: ` prefix (the CLI display layer adds it).
pub const BARE_TR_NO_KEY_MSG: &str = "tr() requires an internal key. For script-path-only spending use tr(NUMS, <ms>); for full taproot use tr(@<index>, <ms>) with a slot binding for the internal key.";

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
                    bad = Some(ToolkitError::DescriptorParse(format!(
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

/// Strip `/derivation` suffix to recover the bare xpub key as substituted.
/// The `[fp/path]` bracket-strip is defensive: `substitute_synthetic` already
/// removes annotations before passing to `Descriptor::from_str`, so
/// rust-miniscript never re-annotates the bare xpubs and `key_str` arrives
/// without brackets in normal flow. The strip stays for safety in case a
/// caller bypasses substitute_synthetic.
fn lookup_key(key_str: &str, km: &BTreeMap<String, u8>) -> Result<u8, ToolkitError> {
    let after_bracket = key_str.find(']').map_or(key_str, |pos| &key_str[pos + 1..]);
    let base = after_bracket.split('/').next().unwrap_or(after_bracket);
    km.get(base).copied().ok_or_else(|| {
        ToolkitError::DescriptorParse(format!(
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
    let indices: Vec<u8> = keys
        .iter()
        .map(|kk| lookup_key(&kk.to_string(), km))
        .collect::<Result<_, ToolkitError>>()?;
    Ok(Node {
        tag,
        body: Body::MultiKeys {
            k: k as u8,
            indices,
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
        Bare(_) => Err(ToolkitError::DescriptorParse(
            "bare scripts are outside BIP-388 wallet-policy surface".into(),
        )),
    }
}

fn walk_wsh(
    w: &miniscript::descriptor::Wsh<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    // Post-#915: Wsh::as_inner() returns &Miniscript directly; SortedMulti
    // surfaces as Terminal::SortedMulti inside the inner Ms (handled by
    // walk_miniscript_node). The pre-#915 WshInner enum is gone.
    let inner = walk_miniscript_node(w.as_inner(), km, /*tap=*/ false)?;
    Ok(wrap_children(Tag::Wsh, inner))
}

fn walk_sh(
    s: &miniscript::descriptor::Sh<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    use miniscript::descriptor::ShInner;
    let inner = match s.as_inner() {
        ShInner::Wsh(w) => Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![walk_miniscript_node(
                w.as_inner(),
                km,
                /*tap=*/ false,
            )?]),
        },
        ShInner::Wpkh(wp) => Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg {
                index: lookup_key(&wp.as_inner().to_string(), km)?,
            },
        },
        ShInner::Ms(ms) => walk_miniscript_node(ms, km, /*tap=*/ false)?,
        // Post-#915: ShInner::SortedMulti variant removed; handled via
        // Terminal::SortedMulti inside ShInner::Ms's inner Miniscript.
    };
    Ok(wrap_children(Tag::Sh, inner))
}

fn walk_tr(
    t: &miniscript::descriptor::Tr<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    // v0.19.0 SPEC §4.12.e — when the internal key is the BIP-341 NUMS H-point
    // (substituted into the descriptor by `substitute_nums_sentinel`), set
    // `Body::Tr.is_nums = true` and skip the key_map lookup. `key_index` is
    // ignored by md-codec when `is_nums = true` (per validate.rs:85-96 +
    // to_miniscript.rs:161-165 — `is_nums=true` triggers `build_nums_internal_key()`
    // which constructs the NUMS DescriptorPublicKey from the same hex constant).
    let internal_key_str = t.internal_key().to_string();
    let (is_nums, key_index) = if internal_key_str == NUMS_H_POINT_X_ONLY_HEX {
        (true, 0u8)
    } else {
        (false, lookup_key(&internal_key_str, km)?)
    };
    let tree: Option<Box<Node>> = match t.tap_tree() {
        None => None,
        Some(tt) => Some(Box::new(walk_tap_tree(tt, km)?)),
    };
    Ok(Node {
        tag: Tag::Tr,
        body: Body::Tr {
            is_nums,
            key_index,
            tree,
        },
    })
}

/// Walk a miniscript `TapTree` into a `md_codec::tree::Node`. SPEC §4.9.a:
/// single-leaf descends directly to the leaf miniscript node (no `Tag::TapTree`
/// wrapper); multi-leaf folds miniscript's flat DFS-preorder `(depth, ms)` list
/// into a binary tree of `Tag::TapTree` branches via a depth-stack algorithm.
/// Empty tap_tree is unreachable per BIP-341 + miniscript constructors
/// (TapTree::leaf requires a Miniscript; combine concatenates non-empty trees);
/// SPIKE-1 confirmed via 6 round-trip probes (1/2/3/4/5-leaf shapes incl.
/// asymmetric and right-spine). Algorithm transcribed verbatim from the
/// SPIKE-1 deliverable at design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md.
fn walk_tap_tree(
    tt: &miniscript::descriptor::TapTree<DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<Node, ToolkitError> {
    let leaves: Vec<(u8, _)> = tt.leaves().map(|li| (li.depth(), li.miniscript())).collect();
    if leaves.is_empty() {
        return Err(ToolkitError::DescriptorParse(
            "tap tree present but contains no leaves".into(),
        ));
    }
    if leaves.len() == 1 {
        let (d, ms) = leaves[0];
        if d != 0 {
            return Err(ToolkitError::DescriptorParse(format!(
                "single-leaf tap_tree leaf at depth {d} (expected 0)"
            )));
        }
        return walk_miniscript_node(ms, km, /*tap=*/ true);
    }
    let mut stack: Vec<(u8, Node)> = Vec::with_capacity(leaves.len());
    for (depth, ms) in leaves {
        let leaf_node = walk_miniscript_node(ms, km, /*tap=*/ true)?;
        stack.push((depth, leaf_node));
        while stack.len() >= 2 {
            let (top_d, _) = stack[stack.len() - 1];
            let (next_d, _) = stack[stack.len() - 2];
            if top_d != next_d || top_d == 0 {
                break;
            }
            let (d, right) = stack.pop().unwrap();
            let (_, left) = stack.pop().unwrap();
            stack.push((
                d - 1,
                Node {
                    tag: Tag::TapTree,
                    body: Body::Children(vec![left, right]),
                },
            ));
        }
    }
    if stack.len() != 1 || stack[0].0 != 0 {
        return Err(ToolkitError::DescriptorParse(format!(
            "tap tree did not fold to a single root at depth 0 (stack depths: {:?})",
            stack.iter().map(|(d, _)| *d).collect::<Vec<_>>()
        )));
    }
    Ok(stack.pop().unwrap().1)
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
        // Post-#915: SortedMulti is now Terminal::SortedMulti instead of
        // WshInner::SortedMulti / ShInner::SortedMulti. Same wire output.
        Terminal::SortedMulti(thresh) => build_multi_node(
            Tag::SortedMulti,
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
        // Post-#910: sortedmulti_a in tap-leaves now parses; emit Tag::SortedMultiA.
        Terminal::SortedMultiA(thresh) => build_multi_node(
            Tag::SortedMultiA,
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
        // v0.3-NEW Layer 2 arms (SPEC §4.9.a) ────────────────────────
        Terminal::After(lt) => Ok(Node {
            tag: Tag::After,
            body: Body::Timelock(lt.to_consensus_u32()),
        }),
        Terminal::Older(lt) => Ok(Node {
            tag: Tag::Older,
            body: Body::Timelock(lt.to_consensus_u32()),
        }),
        Terminal::Sha256(h) => Ok(Node {
            tag: Tag::Sha256,
            body: Body::Hash256Body(h.to_byte_array()),
        }),
        Terminal::Hash256(h) => Ok(Node {
            tag: Tag::Hash256,
            body: Body::Hash256Body(h.to_byte_array()),
        }),
        Terminal::Hash160(h) => Ok(Node {
            tag: Tag::Hash160,
            body: Body::Hash160Body(h.to_byte_array()),
        }),
        Terminal::Ripemd160(h) => Ok(Node {
            tag: Tag::Ripemd160,
            body: Body::Hash160Body(h.to_byte_array()),
        }),
        Terminal::RawPkH(h) => Ok(Node {
            tag: Tag::RawPkH,
            body: Body::Hash160Body(h.to_byte_array()),
        }),
        Terminal::True => Ok(Node {
            tag: Tag::True,
            body: Body::Empty,
        }),
        Terminal::False => Ok(Node {
            tag: Tag::False,
            body: Body::Empty,
        }),
        Terminal::Verify(i) => walk_one_child(Tag::Verify, i, km, tap_context),
        Terminal::Swap(i) => walk_one_child(Tag::Swap, i, km, tap_context),
        Terminal::Alt(i) => walk_one_child(Tag::Alt, i, km, tap_context),
        Terminal::DupIf(i) => walk_one_child(Tag::DupIf, i, km, tap_context),
        Terminal::NonZero(i) => walk_one_child(Tag::NonZero, i, km, tap_context),
        Terminal::ZeroNotEqual(i) => walk_one_child(Tag::ZeroNotEqual, i, km, tap_context),
        Terminal::AndV(a, b) => walk_two_children(Tag::AndV, a, b, km, tap_context),
        Terminal::AndB(a, b) => walk_two_children(Tag::AndB, a, b, km, tap_context),
        Terminal::OrB(a, b) => walk_two_children(Tag::OrB, a, b, km, tap_context),
        Terminal::OrC(a, b) => walk_two_children(Tag::OrC, a, b, km, tap_context),
        Terminal::OrD(a, b) => walk_two_children(Tag::OrD, a, b, km, tap_context),
        Terminal::OrI(a, b) => walk_two_children(Tag::OrI, a, b, km, tap_context),
        Terminal::AndOr(a, b, c) => {
            let kids = vec![
                walk_miniscript_node(a, km, tap_context)?,
                walk_miniscript_node(b, km, tap_context)?,
                walk_miniscript_node(c, km, tap_context)?,
            ];
            Ok(Node {
                tag: Tag::AndOr,
                body: Body::Children(kids),
            })
        }
        Terminal::Thresh(thresh) => {
            let children: Vec<Node> = thresh
                .data()
                .iter()
                .map(|sub| walk_miniscript_node(sub, km, tap_context))
                .collect::<Result<_, _>>()?;
            Ok(Node {
                tag: Tag::Thresh,
                body: Body::Variable {
                    k: thresh.k() as u8,
                    children,
                },
            })
        }
    }
}

fn walk_one_child<C: miniscript::ScriptContext>(
    tag: Tag,
    inner: &miniscript::Miniscript<DescriptorPublicKey, C>,
    km: &BTreeMap<String, u8>,
    tap: bool,
) -> Result<Node, ToolkitError> {
    Ok(Node {
        tag,
        body: Body::Children(vec![walk_miniscript_node(inner, km, tap)?]),
    })
}

fn walk_two_children<C: miniscript::ScriptContext>(
    tag: Tag,
    a: &miniscript::Miniscript<DescriptorPublicKey, C>,
    b: &miniscript::Miniscript<DescriptorPublicKey, C>,
    km: &BTreeMap<String, u8>,
    tap: bool,
) -> Result<Node, ToolkitError> {
    Ok(Node {
        tag,
        body: Body::Children(vec![
            walk_miniscript_node(a, km, tap)?,
            walk_miniscript_node(b, km, tap)?,
        ]),
    })
}

/// One xpub bound to a `@i` placeholder. `payload` is the 65-byte BIP-32 xpub
/// raw form (4-byte version + depth + parent_fp + child + chain_code + pubkey).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedKey {
    pub i: u8,
    pub payload: [u8; 65],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFingerprint {
    pub i: u8,
    pub fp: [u8; 4],
}

/// Top-level descriptor pipeline (SPEC §4.9 step 7). Orchestrates lex → resolve →
/// substitute → miniscript parse → walk → md_codec::Descriptor with TLV-populated
/// keys/fingerprints. Mirrors md-cli's `parse_template`.
pub fn parse_descriptor(
    input: &str,
    keys: &[ParsedKey],
    fingerprints: &[ParsedFingerprint],
) -> Result<MdDescriptor, ToolkitError> {
    // v0.19.0 SPEC §4.12.e — NUMS sentinel substitution + §6.6 row 16
    // bare-tr refusal. Both run BEFORE `lex_placeholders` and
    // `substitute_synthetic` so the rest of the pipeline sees the
    // substituted form (NUMS hex) and the bare-tr case is rejected
    // with the friendly row-16 text before rust-miniscript runs.
    let nums_substituted = substitute_nums_sentinel(input);
    let input: &str = nums_substituted.as_str();

    if detect_bare_tr(input) {
        return Err(ToolkitError::DescriptorParse(BARE_TR_NO_KEY_MSG.to_string()));
    }

    let occs = lex_placeholders(input)?;
    let resolved = resolve_placeholders(&occs)?;
    // SPEC §4.10 mode-determination drives ScriptCtx for synthetic xpub depth.
    // n=1 → SingleSig (depth 3) regardless of outer wrapper; n≥2 → MultiSig
    // (depth 4). Replaces the v0.3-r0 string-prefix heuristic per FOLLOWUP
    // `ctx-for-descriptor-heuristic-misroutes` (Phase A end-of-phase I-2).
    let ctx = if resolved.n == 1 {
        ScriptCtx::SingleSig
    } else {
        ScriptCtx::MultiSig
    };

    let (substituted, key_map) = substitute_synthetic(input, ctx)?;
    let ms_desc = MsDescriptor::<DescriptorPublicKey>::from_str(&substituted)
        .map_err(|e| ToolkitError::DescriptorParse(format!("descriptor parse failed: {e}")))?;
    let tree = walk_root(&ms_desc, &key_map)?;

    let pubkeys = if keys.is_empty() {
        None
    } else {
        let mut v: Vec<_> = keys.iter().map(|k| (k.i, k.payload)).collect();
        v.sort_by_key(|(i, _)| *i);
        Some(v)
    };
    let fp_vec = if fingerprints.is_empty() {
        None
    } else {
        let mut v: Vec<_> = fingerprints.iter().map(|f| (f.i, f.fp)).collect();
        v.sort_by_key(|(i, _)| *i);
        Some(v)
    };
    let use_site_path_overrides = if resolved.use_site_path_overrides.is_empty() {
        None
    } else {
        Some(resolved.use_site_path_overrides)
    };

    let mut tlv = TlvSection::new_empty();
    tlv.use_site_path_overrides = use_site_path_overrides;
    tlv.fingerprints = fp_vec;
    tlv.pubkeys = pubkeys;

    Ok(MdDescriptor {
        n: resolved.n,
        path_decl: resolved.path_decl,
        use_site_path: resolved.use_site_path,
        tree,
        tlv,
    })
}

/// SPEC §4.10 mode determination: `n == 1` → SingleSig regardless of outer
/// wrapper; `n ≥ 2` → MultiSig. The mode controls key sourcing, mk1 cardinality,
/// and verify-bundle's check-element count — NOT the descriptor's structural tree.
/// CLI dispatch delegates mode determination to `bind_descriptor_keys`'s
/// internal `n`-based branching; this enum remains for downstream callers
/// (Phase D verify-bundle re-parse + tests) needing an explicit mode value.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum DescriptorMode {
    SingleSig,
    MultiSig,
}

#[allow(dead_code)]
pub fn determine_mode(d: &MdDescriptor) -> DescriptorMode {
    if d.n == 1 {
        DescriptorMode::SingleSig
    } else {
        DescriptorMode::MultiSig
    }
}

// ctx_for_descriptor retired Phase C.6 r2; classification is post-resolve n-based.

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
    // SAFETY: third-party-blocked — `secp256k1::SecretKey` is stack-bound,
    // no Drop+Zeroize; FOLLOWUP `rust-secp256k1-secretkey-zeroize-upstream`.
    let secret = SecretKey::from_slice(&seed.to_byte_array()).expect("hash is valid scalar");
    let pubkey = secret.public_key(&Secp256k1::new()).serialize();
    let mut bytes = [0u8; 78];
    bytes[0..4].copy_from_slice(&MAINNET_XPUB_VERSION);
    bytes[4] = depth;
    bytes[13..45].copy_from_slice(&chain_code);
    bytes[45..78].copy_from_slice(&pubkey);
    base58::encode_check(&bytes)
}

/// Output of `bind_descriptor_keys`: per-`@N` key/fp pairs for `parse_descriptor`,
/// per-`@N` cosigner triples for `synthesize_descriptor`, and entropy if full mode.
#[derive(Debug)]
pub struct DescriptorBinding {
    #[allow(dead_code)]
    pub keys: Vec<ParsedKey>,
    #[allow(dead_code)]
    pub fingerprints: Vec<ParsedFingerprint>,
    pub cosigners: Vec<CosignerKeyInfo>,
    // v0.4.4 Phase S: bundle-level `entropy` field retired. Per-slot entropy
    // lives on `binding.cosigners[i].entropy` (added in v0.4.3 Phase N's
    // CosignerKeyInfo→ResolvedSlot type alias merge). v0.10.1: field type
    // is `Option<Zeroizing<Vec<u8>>>`; callers that need "is binding @0
    // secret-bearing?" use `DescriptorBinding::entropy_at_0()` (defined
    // below), or `binding.cosigners.first().and_then(|c| c.entropy.as_ref()
    // .map(|z| z.as_slice()))` directly — bare `.as_deref()` returns
    // `Option<&Vec<u8>>` (single-step Zeroizing Deref).
}

impl DescriptorBinding {
    #[allow(dead_code)]
    /// v0.4.4 Phase S compatibility shim — returns the @0 slot's entropy
    /// for callers transitioning from the retired `binding.entropy` field.
    /// New code reads `binding.cosigners[0].entropy` directly.
    pub fn entropy_at_0(&self) -> Option<&[u8]> {
        // v0.10.1: ResolvedSlot.entropy migrated to Option<Zeroizing<Vec<u8>>>.
        // `Option::as_deref` is single-step (Zeroizing::Deref::Target = Vec<u8>);
        // chain through `.as_ref().map(|z| z.as_slice())` to reach &[u8].
        self.cosigners
            .first()
            .and_then(|c| c.entropy.as_ref().map(|z| z.as_slice()))
    }
}

#[allow(dead_code)]
/// SPEC §4.11 binding logic. Resolves the four descriptor-mode key sources:
/// full single-sig (--phrase + n=1), watch-only single-sig (--xpub + n=1),
/// full multisig (--phrase + n-1 cosigners), watch-only multisig (no phrase /
/// xpub + n cosigners). Annotation cross-checks per SPEC §4.11. Phase C.2+C.3.
#[allow(clippy::too_many_arguments)]
pub fn bind_descriptor_keys(
    resolved: &ResolvedPlaceholders,
    network: CliNetwork,
    phrase: Option<&str>,
    passphrase: &str,
    language: CliLanguage,
    xpub_arg: Option<&str>,
    master_fp_arg: Option<&str>,
    cosigner_specs: &[CosignerSpec],
) -> Result<DescriptorBinding, ToolkitError> {
    let n = resolved.n;
    if let Some(p) = phrase {
        bind_full_mode(resolved, network, p, passphrase, language, cosigner_specs)
    } else if let Some(x) = xpub_arg {
        if n == 1 {
            bind_watch_only_singlesig(resolved, x, master_fp_arg)
        } else {
            Err(ToolkitError::ModeViolation {
                mode: "watch-only-multisig",
                flag: "--xpub",
                message: "--xpub is single-sig watch-only; for multisig watch-only, use --cosigner / --cosigners-file with no --phrase / --xpub.",
            })
        }
    } else if !cosigner_specs.is_empty() && n >= 2 {
        bind_watch_only_multisig(resolved, cosigner_specs)
    } else {
        Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--phrase / --xpub / --cosigner",
            message: "descriptor uses @0 but no key source provided; supply --phrase OR a cosigner triple bound to @0.",
        })
    }
}

#[allow(dead_code)]
fn bind_full_mode(
    resolved: &ResolvedPlaceholders,
    network: CliNetwork,
    phrase: &str,
    passphrase: &str,
    language: CliLanguage,
    cosigner_specs: &[CosignerSpec],
) -> Result<DescriptorBinding, ToolkitError> {
    let n = resolved.n;
    // SAFETY: third-party-blocked — `bip39::Mnemonic` + `bitcoin::bip32::Xpriv`
    // have no Drop+Zeroize. FOLLOWUPS: `rust-bip39-mnemonic-zeroize-upstream`,
    // `rust-bitcoin-xpriv-zeroize-upstream`. The seed buffer is
    // `Zeroizing<[u8; 64]>` via `derive_master_seed`; the entropy Vec is
    // wrapped in `Zeroizing<Vec<u8>>` below.
    let mnemonic =
        bip39::Mnemonic::parse_in(language.into(), phrase).map_err(ToolkitError::Bip39)?;
    let entropy = zeroize::Zeroizing::new(mnemonic.to_entropy());
    let seed = crate::derive_slot::derive_master_seed(&mnemonic, passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed[..])
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let master_fp = master.fingerprint(&secp);

    // SPEC §4.11: full mode requires @0 [fp/path] annotation.
    let at0_fp = resolved.fingerprint_annos[0].ok_or_else(|| {
        let mode_label = if n == 1 { "single-sig" } else { "multisig" };
        ToolkitError::DescriptorParse(format!(
            "@0 in full {mode_label} descriptor mode requires explicit [fp/path] origin annotation."
        ))
    })?;
    if at0_fp.to_bytes() != master_fp.to_bytes() {
        return Err(ToolkitError::DescriptorParse(
            "@0 origin fingerprint annotation does not match seed master fingerprint".into(),
        ));
    }
    let at0_path = path_from_decl(resolved, 0)?;
    // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy + no
    // Drop; tracked by FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
    let at0_xpriv = master
        .derive_priv(&secp, &at0_path)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let at0_xpub = Xpub::from_priv(&secp, &at0_xpriv);

    let mut keys = Vec::with_capacity(n as usize);
    let mut fps = Vec::with_capacity(n as usize);
    let mut cosigners = Vec::with_capacity(n as usize);

    push_binding(
        &mut keys,
        &mut fps,
        &mut cosigners,
        0,
        &at0_xpub,
        master_fp,
        at0_path,
    );

    if n >= 2 {
        // Full multisig: cosigner_specs.len() must be n - 1 (SPEC §6.9 row 14).
        if cosigner_specs.len() != (n as usize) - 1 {
            return Err(ToolkitError::DescriptorParse(format!(
                "full multisig descriptor mode requires {}-1 = {} cosigner triples (--phrase supplies @0); got {} --cosigner triple(s).",
                n,
                (n as usize) - 1,
                cosigner_specs.len()
            )));
        }
        for (k, spec) in cosigner_specs.iter().enumerate() {
            let i = (k + 1) as u8;
            let path = spec.path.clone().ok_or_else(|| {
                ToolkitError::DescriptorParse(format!(
                    "cosigner @{i}: descriptor mode requires explicit path in --cosigner triple"
                ))
            })?;
            check_anno_match(resolved, i, Some(spec.master_fingerprint), Some(&path))?;
            push_binding(
                &mut keys,
                &mut fps,
                &mut cosigners,
                i,
                &spec.xpub,
                spec.master_fingerprint,
                path,
            );
        }
    }

    // v0.4.4 Phase S: per-slot entropy on the @0 cosigner; bundle-level
    // entropy field retired.
    //
    // v0.10.1: c0.entropy is Option<Zeroizing<Vec<u8>>>. Local `entropy`
    // here is owned Zeroizing<Vec<u8>> (bound earlier in this function via
    // mnemonic.to_entropy() wrapped in Zeroizing::new); deref-clone yields
    // a bare Vec<u8>, re-wrap to match the field type.
    if let Some(c0) = cosigners.first_mut() {
        c0.entropy = Some(zeroize::Zeroizing::new((*entropy).clone()));
    }
    Ok(DescriptorBinding {
        keys,
        fingerprints: fps,
        cosigners,
    })
}

#[allow(dead_code)]
fn bind_watch_only_singlesig(
    resolved: &ResolvedPlaceholders,
    xpub_str: &str,
    master_fp_arg: Option<&str>,
) -> Result<DescriptorBinding, ToolkitError> {
    let xpub = Xpub::from_str(xpub_str)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::XpubParse(format!("{e}"))))?;
    let fp_arg = master_fp_arg.ok_or(ToolkitError::ModeViolation {
        mode: "watch-only",
        flag: "--master-fingerprint",
        message: "--xpub requires --master-fingerprint (xpub mode needs the master fingerprint to populate mk1's origin)",
    })?;
    let fp = Fingerprint::from_str(fp_arg)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::FingerprintParse(format!("{e}"))))?;
    if let Some(anno_fp) = resolved.fingerprint_annos[0] {
        if anno_fp.to_bytes() != fp.to_bytes() {
            return Err(ToolkitError::DescriptorParse(
                "@0 origin fingerprint annotation does not match --master-fingerprint".into(),
            ));
        }
    }
    let path = path_from_decl(resolved, 0).unwrap_or(DerivationPath::master());
    let mut keys = Vec::new();
    let mut fps = Vec::new();
    let mut cosigners = Vec::new();
    push_binding(&mut keys, &mut fps, &mut cosigners, 0, &xpub, fp, path);
    Ok(DescriptorBinding {
        keys,
        fingerprints: fps,
        cosigners,
    })
}

#[allow(dead_code)]
fn bind_watch_only_multisig(
    resolved: &ResolvedPlaceholders,
    cosigner_specs: &[CosignerSpec],
) -> Result<DescriptorBinding, ToolkitError> {
    let n = resolved.n as usize;
    if cosigner_specs.len() != n {
        return Err(ToolkitError::DescriptorParse(format!(
            "watch-only multisig descriptor mode requires {n} cosigner triples (one per @N); got {} --cosigner triple(s).",
            cosigner_specs.len()
        )));
    }
    let mut keys = Vec::with_capacity(n);
    let mut fps = Vec::with_capacity(n);
    let mut cosigners = Vec::with_capacity(n);
    for (k, spec) in cosigner_specs.iter().enumerate() {
        let i = k as u8;
        let path = spec.path.clone().ok_or_else(|| {
            ToolkitError::DescriptorParse(format!(
                "cosigner @{i}: descriptor mode requires explicit path in --cosigner triple"
            ))
        })?;
        check_anno_match(resolved, i, Some(spec.master_fingerprint), Some(&path))?;
        push_binding(
            &mut keys,
            &mut fps,
            &mut cosigners,
            i,
            &spec.xpub,
            spec.master_fingerprint,
            path,
        );
    }
    Ok(DescriptorBinding {
        keys,
        fingerprints: fps,
        cosigners,
    })
}

#[allow(dead_code)]
fn path_from_decl(resolved: &ResolvedPlaceholders, i: u8) -> Result<DerivationPath, ToolkitError> {
    let origin = match &resolved.path_decl.paths {
        PathDeclPaths::Shared(p) => p,
        PathDeclPaths::Divergent(v) => v.get(i as usize).ok_or_else(|| {
            ToolkitError::DescriptorParse(format!(
                "internal: @{i} path index out of bounds (Divergent.len()={})",
                v.len()
            ))
        })?,
    };
    if origin.components.is_empty() {
        return Err(ToolkitError::DescriptorParse(format!(
            "@{i} requires explicit [fp/path] origin annotation in descriptor mode"
        )));
    }
    let path_str = origin
        .components
        .iter()
        .map(|c| {
            if c.hardened {
                format!("{}'", c.value)
            } else {
                format!("{}", c.value)
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    DerivationPath::from_str(&path_str)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))
}

#[allow(dead_code)]
fn check_anno_match(
    resolved: &ResolvedPlaceholders,
    i: u8,
    spec_fp: Option<Fingerprint>,
    spec_path: Option<&DerivationPath>,
) -> Result<(), ToolkitError> {
    if let Some(anno_fp) = resolved.fingerprint_annos[i as usize] {
        if let Some(sf) = spec_fp {
            if anno_fp.to_bytes() != sf.to_bytes() {
                return Err(ToolkitError::DescriptorParse(format!(
                    "@{i} origin annotation does not match cosigner-triple at index {i}"
                )));
            }
        }
    }
    let anno_path = match &resolved.path_decl.paths {
        PathDeclPaths::Shared(p) => Some(p),
        PathDeclPaths::Divergent(v) => v.get(i as usize),
    };
    if let (Some(anno), Some(sp)) = (anno_path, spec_path) {
        if !anno.components.is_empty() {
            // bitcoin's DerivationPath::from_str normalizes both ' and h hardened
            // markers, so cosigner-spec paths using either form compare correctly.
            let anno_str = anno
                .components
                .iter()
                .map(|c| {
                    if c.hardened {
                        format!("{}'", c.value)
                    } else {
                        format!("{}", c.value)
                    }
                })
                .collect::<Vec<_>>()
                .join("/");
            let anno_dp = DerivationPath::from_str(&anno_str)
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
            if anno_dp != *sp {
                return Err(ToolkitError::DescriptorParse(format!(
                    "@{i} origin annotation path does not match cosigner-triple path"
                )));
            }
        }
    }
    Ok(())
}

/// SPEC §4.11.b BIP-388 distinct-key conformance (v0.5: typed-DerivationPath equality).
///
/// Pairwise scan over `binding.cosigners`; returns `Err(Bip388Distinctness { i, j })`
/// for the first colliding pair under typed `(xpub, path)` equality (i < j). The
/// typed `DerivationPath` form folds `h`-notation into `'`-notation, so
/// `48h/0h/0h/2h` and `48'/0'/0'/2'` compare EQUAL and produce a collision.
///
/// Collision detection consults only the typed `path` (v0.4 raw-string
/// equality is REVERSED — see SPEC v0.5 §4.11.b deliberate-reversal paragraph).
/// v0.37.9 deleted the formerly-separate `path_raw: String` cache; the typed
/// `DerivationPath` is the single source of truth.
///
/// Symmetric across bundle creation (exit 2 via §6.6 row 13) and verify-bundle
/// (caller re-wraps to `Bip388VerifyDistinctness` for exit 4 + §4.11.c text).
pub fn check_key_vector_distinctness(binding: &DescriptorBinding) -> Result<(), ToolkitError> {
    let cs = &binding.cosigners;
    for i in 0..cs.len() {
        for j in (i + 1)..cs.len() {
            if cs[i].xpub.to_string() == cs[j].xpub.to_string() && cs[i].path == cs[j].path {
                return Err(ToolkitError::Bip388Distinctness {
                    i: i as u8,
                    j: j as u8,
                });
            }
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn push_binding(
    keys: &mut Vec<ParsedKey>,
    fps: &mut Vec<ParsedFingerprint>,
    cosigners: &mut Vec<CosignerKeyInfo>,
    i: u8,
    xpub: &Xpub,
    fp: Fingerprint,
    path: DerivationPath,
) {
    keys.push(ParsedKey {
        i,
        payload: xpub_to_65(xpub),
    });
    fps.push(ParsedFingerprint {
        i,
        fp: fp.to_bytes(),
    });
    // Per v0.4.3 Phase N: per-slot entropy lives on ResolvedSlot. Legacy
    // descriptor-mode binding sets entropy: None for all slots; @0's entropy
    // (when --phrase is supplied) is set separately in bind_full_mode after
    // this push (see entropy field assignment below) — no, actually this
    // helper is shared across all binding modes and ALWAYS sets entropy: None.
    // The bundle-level binding.entropy field carries the @0 entropy in the
    // legacy v0.3 binding API; v0.4.3 N retires that field and sets
    // cosigners[0].entropy directly in bind_full_mode after this push.
    cosigners.push(CosignerKeyInfo {
        xpub: *xpub,
        fingerprint: fp,
        path,
        entropy: None,
        master_xpub: None,
        language: None,
        _entropy_pin: None,
    });
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
        let Body::MultiKeys { k, indices } = &children[0].body else {
            panic!("expected SortedMulti MultiKeys body");
        };
        assert_eq!(*k, 2);
        assert_eq!(indices.len(), 2);
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
        let Body::Tr {
            is_nums,
            key_index,
            tree,
        } = &root.body
        else {
            panic!("expected Tr body");
        };
        assert!(!is_nums, "BIP-86 single-sig uses a real key, not NUMS");
        assert_eq!(*key_index, 0);
        assert!(tree.is_none());
    }

    // ---- A.8: mode-determination (6 tests) ----

    fn parse_for_mode(template: &str, ctx: ScriptCtx, n: u8) -> MdDescriptor {
        let (keys, fps) = keys_and_fps_for(n, ctx);
        parse_descriptor(template, &keys, &fps).unwrap()
    }

    #[test]
    fn mode_n1_wpkh_singlesig() {
        let d = parse_for_mode("wpkh(@0/<0;1>/*)", ScriptCtx::SingleSig, 1);
        assert_eq!(determine_mode(&d), DescriptorMode::SingleSig);
    }

    #[test]
    fn mode_n1_pkh_singlesig() {
        let d = parse_for_mode("pkh(@0/<0;1>/*)", ScriptCtx::SingleSig, 1);
        assert_eq!(determine_mode(&d), DescriptorMode::SingleSig);
    }

    #[test]
    fn mode_n1_tr_keypath_singlesig() {
        let d = parse_for_mode("tr(@0/<0;1>/*)", ScriptCtx::SingleSig, 1);
        assert_eq!(determine_mode(&d), DescriptorMode::SingleSig);
    }

    #[test]
    fn mode_n1_wsh_pk_singlesig() {
        // wsh(pk(@0)) has n=1; mode is SingleSig regardless of outer wrapper.
        let d = parse_for_mode("wsh(pk(@0/<0;1>/*))", ScriptCtx::MultiSig, 1);
        assert_eq!(determine_mode(&d), DescriptorMode::SingleSig);
    }

    #[test]
    fn mode_n1_degenerate_wsh_multi_singlesig() {
        // wsh(multi(1,@0)) is structurally multi but n=1 → SingleSig (degenerate).
        // Tree-faithfulness invariant: tree shape is preserved (Multi node remains).
        let d = parse_for_mode("wsh(multi(1,@0/<0;1>/*))", ScriptCtx::MultiSig, 1);
        assert_eq!(determine_mode(&d), DescriptorMode::SingleSig);
        // Verify tree faithfulness: tree still contains Multi(k=1, n=1), not collapsed to PkK.
        let inner = match &d.tree.body {
            Body::Children(kids) => &kids[0],
            _ => panic!("expected Wsh+Children"),
        };
        assert_eq!(inner.tag, Tag::Multi);
    }

    #[test]
    fn mode_n2_wsh_sortedmulti_multisig() {
        let d = parse_for_mode(
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            ScriptCtx::MultiSig,
            2,
        );
        assert_eq!(determine_mode(&d), DescriptorMode::MultiSig);
    }

    // ---- C.2+C.3: bind_descriptor_keys (8 tests) ----

    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    fn trezor_master_fp() -> Fingerprint {
        let m = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let seed = m.to_seed("");
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        master.fingerprint(&secp)
    }

    fn trezor_master_fp_hex() -> String {
        let fp = trezor_master_fp();
        hex_fp(&fp.to_bytes())
    }

    fn hex_fp(bytes: &[u8; 4]) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        )
    }

    fn lex_resolve(template: &str) -> ResolvedPlaceholders {
        let occs = lex_placeholders(template).unwrap();
        resolve_placeholders(&occs).unwrap()
    }

    #[test]
    fn bind_full_singlesig_with_correct_annotation() {
        let fp_hex = trezor_master_fp_hex();
        let template = format!("wpkh(@0[{fp_hex}/84'/0'/0']/<0;1>/*)");
        let resolved = lex_resolve(&template);
        let b = bind_descriptor_keys(
            &resolved,
            CliNetwork::Mainnet,
            Some(TREZOR_24),
            "",
            CliLanguage::English,
            None,
            None,
            &[],
        )
        .unwrap();
        assert_eq!(b.cosigners.len(), 1);
        assert_eq!(b.keys.len(), 1);
        assert!(b.entropy_at_0().is_some(), "full mode emits entropy");
    }

    #[test]
    fn bind_full_singlesig_rejects_fp_mismatch() {
        let template = "wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)";
        let resolved = lex_resolve(template);
        let err = bind_descriptor_keys(
            &resolved,
            CliNetwork::Mainnet,
            Some(TREZOR_24),
            "",
            CliLanguage::English,
            None,
            None,
            &[],
        )
        .unwrap_err();
        assert!(matches!(err, ToolkitError::DescriptorParse(_)));
        assert!(err.message().contains("seed master fingerprint"));
    }

    #[test]
    fn bind_full_singlesig_rejects_missing_annotation() {
        let template = "wpkh(@0/<0;1>/*)";
        let resolved = lex_resolve(template);
        let err = bind_descriptor_keys(
            &resolved,
            CliNetwork::Mainnet,
            Some(TREZOR_24),
            "",
            CliLanguage::English,
            None,
            None,
            &[],
        )
        .unwrap_err();
        assert!(err.message().contains("origin annotation"));
    }

    #[test]
    fn bind_watch_only_singlesig_with_xpub() {
        // Use a valid xpub from existing test vectors.
        let xpub = "xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz";
        let template = "wpkh(@0/<0;1>/*)";
        let resolved = lex_resolve(template);
        let b = bind_descriptor_keys(
            &resolved,
            CliNetwork::Mainnet,
            None,
            "",
            CliLanguage::English,
            Some(xpub),
            Some("deadbeef"),
            &[],
        )
        .unwrap();
        assert!(b.entropy_at_0().is_none(), "watch-only mode omits entropy");
        assert_eq!(b.cosigners.len(), 1);
    }

    #[test]
    fn bind_watch_only_singlesig_rejects_anno_fp_mismatch() {
        let xpub = "xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz";
        // Annotation fp `deadbeef` but --master-fingerprint is `cafef00d` → mismatch.
        let template = "wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)";
        let resolved = lex_resolve(template);
        let err = bind_descriptor_keys(
            &resolved,
            CliNetwork::Mainnet,
            None,
            "",
            CliLanguage::English,
            Some(xpub),
            Some("cafef00d"),
            &[],
        )
        .unwrap_err();
        assert!(err.message().contains("--master-fingerprint"));
    }

    #[test]
    fn bind_full_multisig_n_minus_1_cosigners() {
        let fp_hex = trezor_master_fp_hex();
        // n=2 multisig: phrase supplies @0; --cosigner supplies @1.
        let template = format!(
            "wsh(sortedmulti(2,@0[{fp_hex}/48'/0'/0'/2']/<0;1>/*,@1[cafef00d/48'/0'/0'/2']/<0;1>/*))"
        );
        let resolved = lex_resolve(&template);
        let xpub = Xpub::from_str(
            "xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz",
        )
        .unwrap();
        let cosigner = CosignerSpec {
            xpub,
            master_fingerprint: Fingerprint::from_str("cafef00d").unwrap(),
            path: Some(DerivationPath::from_str("48'/0'/0'/2'").unwrap()),
        };
        let b = bind_descriptor_keys(
            &resolved,
            CliNetwork::Mainnet,
            Some(TREZOR_24),
            "",
            CliLanguage::English,
            None,
            None,
            &[cosigner],
        )
        .unwrap();
        assert_eq!(b.cosigners.len(), 2);
        assert!(b.entropy_at_0().is_some());
    }

    #[test]
    fn bind_watch_only_multisig_n_cosigners() {
        let template = "wsh(sortedmulti(2,@0[deadbeef/48'/0'/0'/2']/<0;1>/*,@1[cafef00d/48'/0'/0'/2']/<0;1>/*))";
        let resolved = lex_resolve(template);
        let xpub = Xpub::from_str(
            "xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz",
        )
        .unwrap();
        let path = Some(DerivationPath::from_str("48'/0'/0'/2'").unwrap());
        let c0 = CosignerSpec {
            xpub,
            master_fingerprint: Fingerprint::from_str("deadbeef").unwrap(),
            path: path.clone(),
        };
        let c1 = CosignerSpec {
            xpub,
            master_fingerprint: Fingerprint::from_str("cafef00d").unwrap(),
            path,
        };
        let b = bind_descriptor_keys(
            &resolved,
            CliNetwork::Mainnet,
            None,
            "",
            CliLanguage::English,
            None,
            None,
            &[c0, c1],
        )
        .unwrap();
        assert_eq!(b.cosigners.len(), 2);
        assert!(b.entropy_at_0().is_none());
    }

    // ---- A.2 (v0.4): BIP-388 distinct-key conformance ----

    fn ckd(cosigners: Vec<CosignerKeyInfo>) -> Result<(), ToolkitError> {
        let binding = DescriptorBinding {
            keys: vec![],
            fingerprints: vec![],
            cosigners,
        };
        check_key_vector_distinctness(&binding)
    }

    fn xpub_a() -> Xpub {
        Xpub::from_str("xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz").unwrap()
    }
    fn xpub_b() -> Xpub {
        Xpub::from_str("xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ").unwrap()
    }
    fn xpub_c() -> Xpub {
        Xpub::from_str("xpub6BemYiVEULcbqF34sTQgz3c2MzCoNmz8ZJieEwjH6HwnZ54tYQmnFgEwRckq3hLJ9feTr4xUFx7XwJ3nraRrQcPnvEuYfddWQ8A4kwU4QMx").unwrap()
    }
    fn cinfo(x: Xpub, p: &str) -> CosignerKeyInfo {
        let path = DerivationPath::from_str(p).unwrap();
        CosignerKeyInfo {
            xpub: x,
            fingerprint: Fingerprint::from_str("deadbeef").unwrap(),
            path,
            entropy: None,
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        }
    }

    #[test]
    fn bip388_distinct_n3_passes() {
        let p = "48'/0'/0'/2'";
        ckd(vec![
            cinfo(xpub_a(), p),
            cinfo(xpub_b(), p),
            cinfo(xpub_c(), p),
        ])
        .expect("3 distinct xpubs at same path is BIP-388-distinct");
    }

    #[test]
    fn bip388_n1_degenerate_passes() {
        let p = "48'/0'/0'/2'";
        ckd(vec![cinfo(xpub_a(), p)]).expect("N=1 has no pairs to compare");
    }

    #[test]
    fn bip388_collision_at0_at1_same_xpub_same_path() {
        let p = "48'/0'/0'/2'";
        let err = ckd(vec![cinfo(xpub_a(), p), cinfo(xpub_a(), p), cinfo(xpub_b(), p)])
            .expect_err("@0 == @1 (xpub, path) must collide");
        match err {
            ToolkitError::Bip388Distinctness { i, j } => {
                assert_eq!((i, j), (0, 1));
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn bip388_first_pair_reported_when_two_collisions() {
        // (@0, @2) and (@1, @3) both collide; first-detected (lex(i,j)) is (@0, @2).
        let p = "48'/0'/0'/2'";
        let err = ckd(vec![
            cinfo(xpub_a(), p),
            cinfo(xpub_b(), p),
            cinfo(xpub_a(), p),
            cinfo(xpub_b(), p),
        ])
        .expect_err("two collisions present; first must be reported");
        match err {
            ToolkitError::Bip388Distinctness { i, j } => assert_eq!((i, j), (0, 2)),
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn bip388_same_xpub_different_paths_accepted() {
        // SPEC §4.11.b BIP-388-letter: distinct (xpub, path) tuples → no collision.
        ckd(vec![
            cinfo(xpub_a(), "48'/0'/0'/2'"),
            cinfo(xpub_a(), "48'/0'/1'/2'"),
        ])
        .expect("identical xpubs with different paths are distinct tuples");
    }

    #[test]
    fn bip388_different_xpubs_same_path_accepted() {
        ckd(vec![
            cinfo(xpub_a(), "48'/0'/0'/2'"),
            cinfo(xpub_b(), "48'/0'/0'/2'"),
        ])
        .expect("distinct xpubs at the same path are distinct tuples");
    }

    #[test]
    fn bip388_collision_same_xpub_both_master_path() {
        // SPEC §4.11.b normalization: both paths == DerivationPath::master() = "m"
        // → equal under raw-string comparison → collide.
        let err = ckd(vec![cinfo(xpub_a(), "m"), cinfo(xpub_a(), "m")])
            .expect_err("identical xpubs at master path must collide");
        match err {
            ToolkitError::Bip388Distinctness { i, j } => assert_eq!((i, j), (0, 1)),
            other => panic!("unexpected variant {other:?}"),
        }
    }

    // v0.5 SPEC §4.11.b REVERSAL — typed-DerivationPath equality. Same xpub +
    // paths that differ ONLY in `h` vs `'` notation now COLLIDE.
    #[test]
    fn bip388_h_vs_apostrophe_paths_collide_under_typed_equality_v0_5() {
        let canonical = "48'/0'/0'/2'";
        let h_form = "48h/0h/0h/2h";
        // Under v0.5, the typed DerivationPath folds h → ', so both notations
        // produce the same DerivationPath value and collide for distinctness.
        // (Post-path_raw-deletion: the typed `path` is built directly from each
        // notation; no separate raw string exists.)
        let err = ckd(vec![
            cinfo(xpub_a(), canonical),
            cinfo(xpub_a(), h_form),
        ])
        .expect_err("v0.5 typed-equality treats h-form and apostrophe-form as the same path");
        assert!(matches!(err, ToolkitError::Bip388Distinctness { i: 0, j: 1 }));
    }

    // v0.4.1 H.6 — identical xpub + identical path collide.
    #[test]
    fn bip388_identical_raw_paths_collide() {
        let raw = "48'/0'/0'/2'";
        let err = ckd(vec![
            cinfo(xpub_a(), raw),
            cinfo(xpub_a(), raw),
        ])
        .expect_err("identical xpub + identical path must collide");
        assert!(matches!(err, ToolkitError::Bip388Distinctness { i: 0, j: 1 }));
    }

    // ---- C.5: descriptor-mode wire-bit-identical to template-mode (3 fixtures) ----

    fn parse_synth_descriptor(template: &str, network: CliNetwork) -> crate::synthesize::Bundle {
        let resolved = lex_resolve(template);
        let binding = bind_descriptor_keys(
            &resolved,
            network,
            Some(TREZOR_24),
            "",
            CliLanguage::English,
            None,
            None,
            &[],
        )
        .unwrap();
        let descriptor = parse_descriptor(template, &binding.keys, &binding.fingerprints).unwrap();
        crate::synthesize::synthesize_descriptor(&descriptor, &binding.cosigners, false, bip39::Language::English).unwrap()
    }

    #[test]
    fn descriptor_bip84_matches_template_bip84_md1() {
        use crate::derive::derive_full;
        use crate::template::CliTemplate;
        let acc = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
            0,
        )
        .unwrap();
        let template_bundle = crate::synthesize::synthesize_full(
            &acc.entropy,
            acc.master_fingerprint,
            acc.account_xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        let fp_hex = trezor_master_fp_hex();
        let descriptor_template = format!("wpkh(@0[{fp_hex}/84'/0'/0']/<0;1>/*)");
        let descriptor_bundle = parse_synth_descriptor(&descriptor_template, CliNetwork::Mainnet);
        assert_eq!(
            descriptor_bundle.md1, template_bundle.md1,
            "md1 must be byte-identical for descriptor expression of bip84 template"
        );
    }

    #[test]
    fn descriptor_bip86_matches_template_bip86_md1() {
        use crate::derive::derive_full;
        use crate::template::CliTemplate;
        let acc = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip86,
            0,
        )
        .unwrap();
        let template_bundle = crate::synthesize::synthesize_full(
            &acc.entropy,
            acc.master_fingerprint,
            acc.account_xpub,
            CliTemplate::Bip86,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        let fp_hex = trezor_master_fp_hex();
        let descriptor_template = format!("tr(@0[{fp_hex}/86'/0'/0']/<0;1>/*)");
        let descriptor_bundle = parse_synth_descriptor(&descriptor_template, CliNetwork::Mainnet);
        assert_eq!(
            descriptor_bundle.md1, template_bundle.md1,
            "md1 must be byte-identical for descriptor expression of bip86 template"
        );
    }

    #[test]
    fn descriptor_bip44_matches_template_bip44_md1() {
        use crate::derive::derive_full;
        use crate::template::CliTemplate;
        let acc = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip44,
            0,
        )
        .unwrap();
        let template_bundle = crate::synthesize::synthesize_full(
            &acc.entropy,
            acc.master_fingerprint,
            acc.account_xpub,
            CliTemplate::Bip44,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        let fp_hex = trezor_master_fp_hex();
        let descriptor_template = format!("pkh(@0[{fp_hex}/44'/0'/0']/<0;1>/*)");
        let descriptor_bundle = parse_synth_descriptor(&descriptor_template, CliNetwork::Mainnet);
        assert_eq!(
            descriptor_bundle.md1, template_bundle.md1,
            "md1 must be byte-identical for descriptor expression of bip44 template"
        );
    }

    // ---- v0.3.1: sortedmulti_a in tap-leaves (post-#910 + #915) ----

    #[test]
    fn arm_sorted_multi_via_wsh() {
        // Post-#915: wsh(sortedmulti(2,A,B)) parses to a Wsh whose inner Ms
        // has node = Terminal::SortedMulti(thresh). The walker emits Tag::SortedMulti
        // via the Layer-2 arm rather than the (now-removed) WshInner::SortedMulti
        // Layer-1 arm. Wire output unchanged from v0.3.0.
        let inner = wsh_inner("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))");
        assert_eq!(inner.tag, Tag::SortedMulti);
        let Body::MultiKeys { k, indices } = inner.body else {
            panic!("expected SortedMulti MultiKeys body");
        };
        assert_eq!(k, 2);
        assert_eq!(indices.len(), 2);
    }

    #[test]
    fn arm_sorted_multi_a_via_tap() {
        // v0.3.1 unblock target: tr(@0, sortedmulti_a(2,@0,@1)) now parses
        // (was deferred in v0.3.0 because rust-miniscript v13.0.0 had no
        // sortedmulti_a parser; PR #910 added it on master).
        let root = parse_and_walk(
            "tr(@0/<0;1>/*,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))",
            ScriptCtx::MultiSig,
        );
        assert_eq!(root.tag, Tag::Tr);
        let Body::Tr { tree, .. } = &root.body else {
            panic!("expected Tr body");
        };
        let leaf = tree.as_ref().expect("expected single tap leaf");
        assert_eq!(leaf.tag, Tag::SortedMultiA);
        let Body::MultiKeys { k, indices } = &leaf.body else {
            panic!("expected SortedMultiA MultiKeys body");
        };
        assert_eq!(*k, 2);
        assert_eq!(indices.len(), 2);
    }

    #[test]
    fn descriptor_tr_sortedmulti_a_matches_template_tr_sortedmulti_a_md1() {
        // Wire-bit-identical regression: descriptor-mode tr(@0, sortedmulti_a(...))
        // must produce md1 byte-identical to template-mode --template tr-sortedmulti-a
        // for matching keys/cosigners. This confirms the new Terminal::SortedMultiA
        // walker arm produces the same Tag::SortedMultiA tree the template encoder
        // has been producing since v0.3.0 (template-mode bypasses rust-miniscript).
        use crate::parse::{CosignerSpec, MultisigPathFamily};
        use crate::synthesize::synthesize_multisig_full;
        use crate::template::CliTemplate;

        // Template-mode self-multisig bundle (v0.3.0 path: bypasses miniscript).
        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let template_bundle = synthesize_multisig_full(
            &mnemonic,
            "",
            CliNetwork::Mainnet,
            CliTemplate::TrSortedMultiA,
            2, // threshold
            2, // cosigner_count
            0, // account
            MultisigPathFamily::Bip48,
            false, // privacy_preserving
        )
        .unwrap();

        // Derive the same self-multisig xpub the template produced.
        let script_type = CliTemplate::TrSortedMultiA.bip48_script_type().unwrap_or(0);
        let path_str =
            MultisigPathFamily::Bip48.default_origin_path(CliNetwork::Mainnet, 0, script_type);
        let path = DerivationPath::from_str(&path_str).unwrap();
        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        let master_fp = master.fingerprint(&secp);
        let xpriv = master.derive_priv(&secp, &path).unwrap();
        let xpub = Xpub::from_priv(&secp, &xpriv);
        let cosigner = CosignerSpec {
            xpub,
            master_fingerprint: master_fp,
            path: Some(path.clone()),
        };

        // Build descriptor with @0 + @1 each bound to the self-derived xpub at
        // the BIP-48 self-multisig path (mirrors what the template produces).
        let path_anno = path
            .into_iter()
            .map(|c| match c {
                ChildNumber::Hardened { index } => format!("{}'", index),
                ChildNumber::Normal { index } => format!("{}", index),
            })
            .collect::<Vec<_>>()
            .join("/");
        let fp_hex = hex_fp(&master_fp.to_bytes());
        let descriptor = format!(
            "tr(@0[{fp_hex}/{path_anno}]/<0;1>/*,\
             sortedmulti_a(2,@0[{fp_hex}/{path_anno}]/<0;1>/*,\
             @1[{fp_hex}/{path_anno}]/<0;1>/*))"
        );

        // Drive bind_descriptor_keys directly (descriptor n=2 → full multisig
        // mode requires n-1 = 1 cosigner triple alongside --phrase for @0).
        let resolved = lex_resolve(&descriptor);
        let binding = bind_descriptor_keys(
            &resolved,
            CliNetwork::Mainnet,
            Some(TREZOR_24),
            "",
            CliLanguage::English,
            None,
            None,
            &[cosigner],
        )
        .unwrap();
        let parsed = parse_descriptor(&descriptor, &binding.keys, &binding.fingerprints).unwrap();
        let descriptor_bundle =
            crate::synthesize::synthesize_descriptor(&parsed, &binding.cosigners, false, bip39::Language::English).unwrap();

        assert_eq!(
            descriptor_bundle.md1, template_bundle.md1,
            "md1 must be byte-identical between descriptor-mode and template-mode \
             tr-sortedmulti-a synthesis"
        );
    }

    #[test]
    fn bind_rejects_xpub_with_multisig() {
        let template = "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))";
        let resolved = lex_resolve(template);
        let xpub = "xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz";
        let err = bind_descriptor_keys(
            &resolved,
            CliNetwork::Mainnet,
            None,
            "",
            CliLanguage::English,
            Some(xpub),
            Some("deadbeef"),
            &[],
        )
        .unwrap_err();
        assert!(matches!(err, ToolkitError::ModeViolation { .. }));
    }

    // ---- A.7: parse_descriptor top-level orchestration (4 tests) ----

    /// Build a deterministic ParsedKey for index `i` from the synthetic xpub.
    /// Test-only — Phase B+ supplies real xpubs.
    fn synthetic_parsed_key(i: u8, ctx: ScriptCtx) -> ParsedKey {
        let xpub_str = synthetic_xpub_for(i, ctx);
        let bytes = bitcoin::base58::decode_check(&xpub_str).unwrap();
        // 78-byte BIP-32 xpub → 65-byte md-codec form (drop 4-byte version + 9 unused = 13 prefix bytes).
        let mut payload = [0u8; 65];
        payload.copy_from_slice(&bytes[13..78]);
        ParsedKey { i, payload }
    }

    fn synthetic_parsed_fp(i: u8) -> ParsedFingerprint {
        // Use a stable test-only fingerprint per index.
        let fp = [0xde, 0xad, 0xbe, 0xef ^ i];
        ParsedFingerprint { i, fp }
    }

    fn keys_and_fps_for(n: u8, ctx: ScriptCtx) -> (Vec<ParsedKey>, Vec<ParsedFingerprint>) {
        (
            (0..n).map(|i| synthetic_parsed_key(i, ctx)).collect(),
            (0..n).map(synthetic_parsed_fp).collect(),
        )
    }

    #[test]
    fn parse_descriptor_hash_locked() {
        let template = format!("wsh(and_v(v:pk(@0/<0;1>/*),sha256({H32})))");
        let (keys, fps) = keys_and_fps_for(1, ScriptCtx::MultiSig);
        let d = parse_descriptor(&template, &keys, &fps).unwrap();
        assert_eq!(d.n, 1);
        assert_eq!(d.tree.tag, Tag::Wsh);
        assert!(d.tlv.pubkeys.as_ref().is_some_and(|v| v.len() == 1));
        assert!(d.tlv.fingerprints.as_ref().is_some_and(|v| v.len() == 1));
    }

    #[test]
    fn parse_descriptor_timelock() {
        let template = "wsh(and_v(v:pk(@0/<0;1>/*),older(144)))";
        let (keys, fps) = keys_and_fps_for(1, ScriptCtx::MultiSig);
        let d = parse_descriptor(template, &keys, &fps).unwrap();
        assert_eq!(d.n, 1);
        assert_eq!(d.tree.tag, Tag::Wsh);
        let inner = match &d.tree.body {
            Body::Children(kids) => &kids[0],
            _ => panic!("expected Wsh+Children"),
        };
        assert_eq!(inner.tag, Tag::AndV);
    }

    #[test]
    fn parse_descriptor_hybrid() {
        // Combines hash + timelock via or_d.
        let template = format!("wsh(or_d(pk(@0/<0;1>/*),and_v(v:sha256({H32}),older(144))))");
        let (keys, fps) = keys_and_fps_for(1, ScriptCtx::MultiSig);
        let d = parse_descriptor(&template, &keys, &fps).unwrap();
        assert_eq!(d.n, 1);
        let inner = match &d.tree.body {
            Body::Children(kids) => &kids[0],
            _ => panic!("expected Wsh+Children"),
        };
        assert_eq!(inner.tag, Tag::OrD);
    }

    #[test]
    fn parse_descriptor_multisig_with_annotation() {
        let template = "wsh(sortedmulti(2,@0[deadbeef/48'/0'/0'/2']/<0;1>/*,@1[cafef00d/48'/0'/0'/2']/<0;1>/*))";
        let (keys, fps) = keys_and_fps_for(2, ScriptCtx::MultiSig);
        let d = parse_descriptor(template, &keys, &fps).unwrap();
        assert_eq!(d.n, 2);
        assert_eq!(d.tree.tag, Tag::Wsh);
        let inner = match &d.tree.body {
            Body::Children(kids) => &kids[0],
            _ => panic!("expected Wsh+Children"),
        };
        assert_eq!(inner.tag, Tag::SortedMulti);
        // Origin paths are Shared (both annotations agree).
        assert!(matches!(d.path_decl.paths, PathDeclPaths::Shared(_)));
    }

    // ---- A.6: v0.3-NEW Layer 2 arms (23 tests) ----

    const H32: &str = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
    const H20: &str = "0102030405060708090a0b0c0d0e0f1011121314";

    /// Walk a wsh-rooted descriptor and return the inner (Wsh's child) node.
    fn wsh_inner(template: &str) -> Node {
        let root = parse_and_walk(template, ScriptCtx::MultiSig);
        let Body::Children(kids) = root.body else {
            panic!("expected Wsh+Children, got: {:?}", root);
        };
        kids.into_iter().next().unwrap()
    }

    /// Find a node anywhere in the tree with the given tag (depth-first).
    fn find_tag(node: &Node, tag: Tag) -> Option<&Node> {
        if node.tag == tag {
            return Some(node);
        }
        match &node.body {
            Body::Children(kids) => kids.iter().find_map(|k| find_tag(k, tag)),
            Body::Variable { children, .. } => children.iter().find_map(|k| find_tag(k, tag)),
            Body::Tr { tree, .. } => tree.as_ref().and_then(|t| find_tag(t, tag)),
            _ => None,
        }
    }

    #[test]
    fn arm_after() {
        let inner = wsh_inner("wsh(and_v(v:pk(@0/<0;1>/*),after(144)))");
        let n = find_tag(&inner, Tag::After).expect("After");
        assert!(matches!(n.body, Body::Timelock(144)));
    }

    #[test]
    fn arm_older() {
        let inner = wsh_inner("wsh(and_v(v:pk(@0/<0;1>/*),older(1000)))");
        let n = find_tag(&inner, Tag::Older).expect("Older");
        assert!(matches!(n.body, Body::Timelock(1000)));
    }

    #[test]
    fn arm_sha256() {
        let s = format!("wsh(and_v(v:pk(@0/<0;1>/*),sha256({H32})))");
        let inner = wsh_inner(&s);
        let n = find_tag(&inner, Tag::Sha256).expect("Sha256");
        assert!(matches!(n.body, Body::Hash256Body(_)));
    }

    #[test]
    fn arm_hash256() {
        let s = format!("wsh(and_v(v:pk(@0/<0;1>/*),hash256({H32})))");
        let inner = wsh_inner(&s);
        let n = find_tag(&inner, Tag::Hash256).expect("Hash256");
        assert!(matches!(n.body, Body::Hash256Body(_)));
    }

    #[test]
    fn arm_hash160() {
        let s = format!("wsh(and_v(v:pk(@0/<0;1>/*),hash160({H20})))");
        let inner = wsh_inner(&s);
        let n = find_tag(&inner, Tag::Hash160).expect("Hash160");
        assert!(matches!(n.body, Body::Hash160Body(_)));
    }

    #[test]
    fn arm_ripemd160() {
        let s = format!("wsh(and_v(v:pk(@0/<0;1>/*),ripemd160({H20})))");
        let inner = wsh_inner(&s);
        let n = find_tag(&inner, Tag::Ripemd160).expect("Ripemd160");
        assert!(matches!(n.body, Body::Hash160Body(_)));
    }

    #[test]
    #[ignore = "RawPkH is descriptor-unreachable in rust-miniscript v13 — appears only via raw-script decode (v0.4+ scope)"]
    fn arm_raw_pkh() {
        // Walker arm exists for round-trip intake of pre-encoded bundles;
        // not reachable from --descriptor parse path. Counted as 1 stub.
    }

    #[test]
    fn arm_false() {
        // and_n(X,Y) expands to andor(X,Y,0). The trailing `0` is Terminal::False.
        let inner = wsh_inner("wsh(and_n(pk(@0/<0;1>/*),pk(@1/<0;1>/*)))");
        let n = find_tag(&inner, Tag::False).expect("False");
        assert!(matches!(n.body, Body::Empty));
    }

    #[test]
    fn arm_true() {
        // t:X expands to and_v(v:X, 1). v: requires B; pk_k is K; so chain is
        // tvc:pk_k(K) = t:v:c:pk_k(K) = and_v(verify(check(pk_k(K))), 1).
        let inner = wsh_inner("wsh(tvc:pk_k(@0/<0;1>/*))");
        let n = find_tag(&inner, Tag::True).expect("True");
        assert!(matches!(n.body, Body::Empty));
    }

    #[test]
    fn arm_verify() {
        // and_v(v:X, Y) — the v:X is Terminal::Verify(X).
        let inner = wsh_inner("wsh(and_v(v:pk(@0/<0;1>/*),pk(@1/<0;1>/*)))");
        let n = find_tag(&inner, Tag::Verify).expect("Verify");
        assert!(matches!(n.body, Body::Children(_)));
    }

    #[test]
    fn arm_swap() {
        // and_b(B, s:X) — s:X is Terminal::Swap(X).
        let inner = wsh_inner("wsh(and_b(pk(@0/<0;1>/*),s:pk(@1/<0;1>/*)))");
        let n = find_tag(&inner, Tag::Swap).expect("Swap");
        assert!(matches!(n.body, Body::Children(_)));
    }

    #[test]
    fn arm_alt() {
        // and_b(B, a:X) — a:X is Terminal::Alt(X).
        let inner = wsh_inner("wsh(and_b(pk(@0/<0;1>/*),a:pk(@1/<0;1>/*)))");
        let n = find_tag(&inner, Tag::Alt).expect("Alt");
        assert!(matches!(n.body, Body::Children(_)));
    }

    #[test]
    #[ignore = "DupIf descriptor-unreachable in rust-miniscript v13 — every d: example in ms_tests.rs is invalid_ms"]
    fn arm_dup_if() {
        // Walker arm exists for completeness; counted as 1 stub.
    }

    #[test]
    fn arm_non_zero() {
        // j:X is Terminal::NonZero(X).
        let s = format!("wsh(or_d(j:and_v(vc:pk_k(@0/<0;1>/*),hash160({H20})),pk(@1/<0;1>/*)))");
        let inner = wsh_inner(&s);
        let n = find_tag(&inner, Tag::NonZero).expect("NonZero");
        assert!(matches!(n.body, Body::Children(_)));
    }

    #[test]
    fn arm_zero_not_equal() {
        // n:X is Terminal::ZeroNotEqual(X). Type-restricted: requires B → Z.
        // Try `wsh(c:and_v(vn:pk_k(K), pk_k(K2)))` ... actually let's try via expansion.
        // `vn:older(144)` may work inside and_v(V, B): and_v requires V-typed first arg.
        // vn: is `v:n:` which makes B → V via Z → V (via verify).
        let s = "wsh(and_v(vn:older(144),pk(@0/<0;1>/*)))";
        let inner = wsh_inner(s);
        let n = find_tag(&inner, Tag::ZeroNotEqual).expect("ZeroNotEqual");
        assert!(matches!(n.body, Body::Children(_)));
    }

    #[test]
    fn arm_and_v() {
        let inner = wsh_inner("wsh(and_v(v:pk(@0/<0;1>/*),pk(@1/<0;1>/*)))");
        assert_eq!(inner.tag, Tag::AndV);
        let Body::Children(kids) = inner.body else {
            panic!("expected AndV+Children");
        };
        assert_eq!(kids.len(), 2);
    }

    #[test]
    fn arm_and_b() {
        let inner = wsh_inner("wsh(and_b(pk(@0/<0;1>/*),a:pk(@1/<0;1>/*)))");
        assert_eq!(inner.tag, Tag::AndB);
        let Body::Children(kids) = inner.body else {
            panic!("expected AndB+Children");
        };
        assert_eq!(kids.len(), 2);
    }

    #[test]
    fn arm_and_or() {
        let s = format!("wsh(andor(pk(@0/<0;1>/*),older(144),hash160({H20})))");
        let inner = wsh_inner(&s);
        assert_eq!(inner.tag, Tag::AndOr);
        let Body::Children(kids) = inner.body else {
            panic!("expected AndOr+Children");
        };
        assert_eq!(kids.len(), 3);
    }

    #[test]
    fn arm_or_b() {
        // or_b takes (B, W) — pair B with a:X.
        let inner = wsh_inner("wsh(or_b(pk(@0/<0;1>/*),a:pk(@1/<0;1>/*)))");
        assert_eq!(inner.tag, Tag::OrB);
    }

    #[test]
    fn arm_or_c() {
        // or_c takes (B, V); V via v:X.
        let inner = wsh_inner("wsh(t:or_c(pk(@0/<0;1>/*),v:pk(@1/<0;1>/*)))");
        let n = find_tag(&inner, Tag::OrC).expect("OrC");
        assert!(matches!(n.body, Body::Children(_)));
    }

    #[test]
    fn arm_or_d() {
        let inner = wsh_inner("wsh(or_d(pk(@0/<0;1>/*),pk(@1/<0;1>/*)))");
        assert_eq!(inner.tag, Tag::OrD);
    }

    #[test]
    fn arm_or_i() {
        let inner = wsh_inner("wsh(or_i(pk(@0/<0;1>/*),pk(@1/<0;1>/*)))");
        assert_eq!(inner.tag, Tag::OrI);
    }

    #[test]
    fn arm_thresh() {
        let inner = wsh_inner("wsh(thresh(2,pk(@0/<0;1>/*),s:pk(@1/<0;1>/*),s:pk(@2/<0;1>/*)))");
        assert_eq!(inner.tag, Tag::Thresh);
        let Body::Variable { k, children } = inner.body else {
            panic!("expected Thresh Variable body");
        };
        assert_eq!(k, 2);
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn phase0_i1_bsms_decaying_multisig_walks_end_to_end() {
        // Phase 0 R0 architect-review I1 fold: verify the user's flagship BSMS
        // decaying-multisig AST (pkh + s:pk + sln:older composition) walks through
        // the full `parse_descriptor` pipeline (lex → resolve → substitute_synthetic
        // → MsDescriptor::from_str → walk_root → md_codec::Descriptor). The shape
        // is `wsh(thresh(2, pkh(@0/...), s:pk(@1/...), sln:older(32768)))` — the
        // unique composition is `sln:` = Swap + OrI(False, ZeroNotEqual(Older(N)))
        // plus pkh as a Thresh child (existing arm_thresh uses pk, not pkh).
        let inner = wsh_inner(
            "wsh(thresh(2,pkh(@0/<0;1>/*),s:pk(@1/<0;1>/*),sln:older(32768)))",
        );
        assert_eq!(inner.tag, Tag::Thresh);
        let Body::Variable { k, children } = inner.body else {
            panic!("expected Thresh Variable body");
        };
        assert_eq!(k, 2);
        assert_eq!(children.len(), 3);
    }

    // ---- A.5: dedicated Layer 2 carry tests (5 tests) ----

    #[test]
    fn walk_pk_in_tap_collapses_to_pkk() {
        // pk(K) in tap-leaf desugars to c:pk_k(K); the walker collapses Check(PkK)
        // → PkK in tap context per md-cli precedent. Architect L-4 mid-phase ask.
        let root = parse_and_walk("tr(@0/<0;1>/*,pk(@1/<0;1>/*))", ScriptCtx::MultiSig);
        assert_eq!(root.tag, Tag::Tr);
        let Body::Tr { tree, .. } = &root.body else {
            panic!("expected Tr");
        };
        let leaf = tree.as_ref().unwrap();
        assert_eq!(
            leaf.tag,
            Tag::PkK,
            "tap-context Check should collapse to PkK"
        );
    }

    #[test]
    fn walk_multi_direct_in_wsh() {
        // Hits Terminal::Multi (not SortedMulti) directly; A.4 only covered SortedMulti.
        let root = parse_and_walk("wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))", ScriptCtx::MultiSig);
        assert_eq!(root.tag, Tag::Wsh);
        let Body::Children(children) = &root.body else {
            panic!("expected Wsh+Children");
        };
        assert_eq!(children[0].tag, Tag::Multi);
        let Body::MultiKeys { k, indices } = &children[0].body else {
            panic!("expected Multi MultiKeys body");
        };
        assert_eq!(*k, 2);
        assert_eq!(indices.len(), 2);
    }

    #[test]
    fn walk_check_kept_in_non_tap_context() {
        // Verify wsh(pk(@0)) emits Wsh→Check→PkK (NOT collapsed in non-tap).
        let root = parse_and_walk("wsh(pk(@0/<0;1>/*))", ScriptCtx::MultiSig);
        let Body::Children(wsh_kids) = &root.body else {
            panic!("expected Wsh+Children");
        };
        assert_eq!(wsh_kids[0].tag, Tag::Check);
        let Body::Children(check_kids) = &wsh_kids[0].body else {
            panic!("expected Check+Children");
        };
        assert_eq!(check_kids[0].tag, Tag::PkK);
    }

    #[test]
    fn walk_pk_h_via_wsh_andor() {
        // PkH appears as a Layer 2 fragment via `pkh()` inside miniscript,
        // which desugars to c:pk_h(K). Use `and_v(v:pkh(@0), older(144))`-style
        // composition to hit it; rust-miniscript will route to Terminal::PkH.
        // Note: A.6 lands the and_v/older arms so this exact shape errors here;
        // simpler form: wsh(c:pk_h(@0)) — typecheck-permitting.
        let result = substitute_synthetic("wsh(c:pk_h(@0/<0;1>/*))", ScriptCtx::MultiSig);
        let (s, km) = result.unwrap();
        let d = MsDescriptor::<DescriptorPublicKey>::from_str(&s)
            .expect("c:pk_h should parse as a wsh-inner miniscript");
        let root = walk_root(&d, &km).unwrap();
        let Body::Children(wsh_kids) = &root.body else {
            panic!("expected Wsh+Children");
        };
        assert_eq!(wsh_kids[0].tag, Tag::Check);
        let Body::Children(check_kids) = &wsh_kids[0].body else {
            panic!("expected Check+Children");
        };
        assert_eq!(check_kids[0].tag, Tag::PkH);
    }

    #[test]
    fn walk_multi_a_direct_via_tap() {
        // Layer 2 MultiA via tap-leaf, distinct from A.4's tr_singleleaf_multi_a_root
        // (which asserted Tr-level shape only). Here we assert the leaf's Variable body.
        let root = parse_and_walk(
            "tr(@0/<0;1>/*,multi_a(3,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
            ScriptCtx::MultiSig,
        );
        let Body::Tr { tree, .. } = &root.body else {
            panic!("expected Tr");
        };
        let leaf = tree.as_ref().unwrap();
        assert_eq!(leaf.tag, Tag::MultiA);
        let Body::MultiKeys { k, indices } = &leaf.body else {
            panic!("expected MultiA MultiKeys body");
        };
        assert_eq!(*k, 3);
        assert_eq!(indices.len(), 3);
    }

    // ---- Phase F (v0.4): walk_tap_tree multi-leaf round-trips ----

    /// Recursive descent helper: count leaves + max-depth in a md_codec Node tree
    /// rooted at a TapTree branch (or single leaf).
    fn count_tap_leaves(n: &Node, depth: u8) -> (usize, u8) {
        match (&n.tag, &n.body) {
            (Tag::TapTree, Body::Children(cs)) if cs.len() == 2 => {
                let (l1, d1) = count_tap_leaves(&cs[0], depth + 1);
                let (l2, d2) = count_tap_leaves(&cs[1], depth + 1);
                (l1 + l2, d1.max(d2))
            }
            _ => (1, depth),
        }
    }

    #[test]
    fn walk_tap_tree_2_leaf_balanced() {
        // tr(@0, {pk(@1), pk(@2)}) — depths [1, 1]. Root: TapTree[leaf, leaf].
        let root = parse_and_walk(
            "tr(@0/<0;1>/*,{pk(@1/<0;1>/*),pk(@2/<0;1>/*)})",
            ScriptCtx::MultiSig,
        );
        assert_eq!(root.tag, Tag::Tr);
        let Body::Tr { tree, .. } = &root.body else { panic!("expected Tr") };
        let tap_root = tree.as_ref().expect("multi-leaf tap_tree present");
        assert_eq!(tap_root.tag, Tag::TapTree);
        let (leaves, max_depth) = count_tap_leaves(tap_root, 0);
        assert_eq!(leaves, 2);
        assert_eq!(max_depth, 1);
    }

    #[test]
    fn walk_tap_tree_3_leaf_asymmetric() {
        // tr(@0, {pk(@1), {pk(@2), pk(@3)}}) — depths [1, 2, 2].
        // Root: TapTree[leaf, TapTree[leaf, leaf]].
        let root = parse_and_walk(
            "tr(@0/<0;1>/*,{pk(@1/<0;1>/*),{pk(@2/<0;1>/*),pk(@3/<0;1>/*)}})",
            ScriptCtx::MultiSig,
        );
        let Body::Tr { tree, .. } = &root.body else { panic!("expected Tr") };
        let tap_root = tree.as_ref().unwrap();
        assert_eq!(tap_root.tag, Tag::TapTree);
        let (leaves, max_depth) = count_tap_leaves(tap_root, 0);
        assert_eq!(leaves, 3);
        assert_eq!(max_depth, 2);
        // Confirm asymmetry: left child is leaf (PkK), right is TapTree.
        let Body::Children(children) = &tap_root.body else { panic!("expected TapTree.Children") };
        assert_eq!(children[0].tag, Tag::PkK);
        assert_eq!(children[1].tag, Tag::TapTree);
    }

    #[test]
    fn walk_tap_tree_4_leaf_balanced() {
        // tr(@0, {{pk(@1),pk(@2)},{pk(@3),pk(@4)}}) — depths [2,2,2,2].
        let root = parse_and_walk(
            "tr(@0/<0;1>/*,{{pk(@1/<0;1>/*),pk(@2/<0;1>/*)},{pk(@3/<0;1>/*),pk(@4/<0;1>/*)}})",
            ScriptCtx::MultiSig,
        );
        let Body::Tr { tree, .. } = &root.body else { panic!("expected Tr") };
        let tap_root = tree.as_ref().unwrap();
        assert_eq!(tap_root.tag, Tag::TapTree);
        let (leaves, max_depth) = count_tap_leaves(tap_root, 0);
        assert_eq!(leaves, 4);
        assert_eq!(max_depth, 2);
        // Both children of root are TapTree branches.
        let Body::Children(children) = &tap_root.body else { panic!("expected TapTree.Children") };
        assert_eq!(children[0].tag, Tag::TapTree);
        assert_eq!(children[1].tag, Tag::TapTree);
    }

    #[test]
    fn walk_tap_tree_4_leaf_right_spine() {
        // tr(@0, {pk(@1), {pk(@2), {pk(@3), pk(@4)}}}) — depths [1, 2, 3, 3].
        // SPIKE-1 r1 review L-1 added this shape to confirm the depth-stack
        // algorithm correctly defers folding for right-heavy trees.
        let root = parse_and_walk(
            "tr(@0/<0;1>/*,{pk(@1/<0;1>/*),{pk(@2/<0;1>/*),{pk(@3/<0;1>/*),pk(@4/<0;1>/*)}}})",
            ScriptCtx::MultiSig,
        );
        let Body::Tr { tree, .. } = &root.body else { panic!("expected Tr") };
        let tap_root = tree.as_ref().unwrap();
        let (leaves, max_depth) = count_tap_leaves(tap_root, 0);
        assert_eq!(leaves, 4);
        assert_eq!(max_depth, 3);
    }

    #[test]
    fn walk_tr_singleleaf_multi_a_root() {
        let root = parse_and_walk(
            "tr(@0/<0;1>/*,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
            ScriptCtx::MultiSig,
        );
        assert_eq!(root.tag, Tag::Tr);
        let Body::Tr {
            is_nums: _,
            key_index: _,
            tree,
        } = &root.body
        else {
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

    // ---- v0.19.0 Phase 3: NUMS sentinel + bare-tr detection ----

    #[test]
    fn substitute_nums_sentinel_replaces_tr_nums_with_hex() {
        let input = "tr(NUMS,and_v(v:pk([deadbeef/86h/0h/0h]@0),after(12000000)))";
        let out = substitute_nums_sentinel(input);
        let expected = format!(
            "tr({NUMS_H_POINT_X_ONLY_HEX},and_v(v:pk([deadbeef/86h/0h/0h]@0),after(12000000)))"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn substitute_nums_sentinel_replaces_tr_nums_no_second_arg() {
        // Degenerate case: tr(NUMS) with no tap-tree. Substitutes; rust-miniscript
        // may still accept or reject downstream — the substitution itself is
        // pattern-only.
        let input = "tr(NUMS)";
        let out = substitute_nums_sentinel(input);
        let expected = format!("tr({NUMS_H_POINT_X_ONLY_HEX})");
        assert_eq!(out, expected);
    }

    #[test]
    fn substitute_nums_sentinel_preserves_descriptors_without_tr_nums() {
        let input = "wsh(andor(pkh(@0),after(12000000),pk(@1)))";
        let out = substitute_nums_sentinel(input);
        assert_eq!(out, input);
    }

    #[test]
    fn substitute_nums_sentinel_does_not_match_nums_in_identifier() {
        // Word-boundary regex: `tr(NUMSOMETHING` MUST NOT substitute because
        // NUMS is followed by `O` (a word char), violating `\b`.
        let input = "tr(NUMSOMETHING)";
        let out = substitute_nums_sentinel(input);
        assert_eq!(out, input);
    }

    #[test]
    fn substitute_nums_sentinel_does_not_match_outside_tr() {
        // The regex anchors `tr(` prefix, so a NUMS reference outside tr()
        // (which shouldn't occur in practice; this is defensive) is preserved.
        let input = "wsh(NUMS,pk(@0))";
        let out = substitute_nums_sentinel(input);
        assert_eq!(out, input);
    }

    #[test]
    fn detect_bare_tr_recognizes_miniscript_fragments() {
        // SPEC §6.6 row 16 — `tr(<miniscript-fragment>(...))` is bare-tr.
        assert!(detect_bare_tr("tr(andor(pkh(@0),after(12000000),pk(@1)))"));
        assert!(detect_bare_tr("tr(and_v(v:pk(@0),after(12000000)))"));
        assert!(detect_bare_tr("tr(pk(@0))"));
        assert!(detect_bare_tr("tr(pkh(@0))"));
        assert!(detect_bare_tr("tr(multi(2,@0,@1,@2))"));
        assert!(detect_bare_tr("tr(thresh(2,pk(@0),pk(@1),pk(@2)))"));
    }

    #[test]
    fn detect_bare_tr_rejects_valid_internal_key_forms() {
        // After NUMS substitution: hex internal key.
        assert!(!detect_bare_tr(&format!("tr({NUMS_H_POINT_X_ONLY_HEX},pk(@0))")));
        // Placeholder internal key.
        assert!(!detect_bare_tr("tr(@0)"));
        assert!(!detect_bare_tr("tr(@0,pk(@1))"));
        // Annotated placeholder.
        assert!(!detect_bare_tr("tr([deadbeef/86h/0h/0h]@0,pk(@1))"));
        // xpub-style (lowercase but followed by digits, not `(`).
        assert!(!detect_bare_tr("tr(xpub6CUGRUo,pk(@0))"));
        // Canonical bare-tr keypath.
        assert!(!detect_bare_tr("tr(@0/<0;1>/*)"));
    }

    #[test]
    fn detect_bare_tr_rejects_non_tr_descriptors() {
        // Should only match `tr(...)` root; other wrappers ignored.
        assert!(!detect_bare_tr("wsh(andor(pkh(@0),after(12000000),pk(@1)))"));
        assert!(!detect_bare_tr("pkh(@0)"));
        assert!(!detect_bare_tr("wpkh(@0)"));
        assert!(!detect_bare_tr("sh(wsh(multi(2,@0,@1)))"));
    }

    #[test]
    fn parse_descriptor_refuses_bare_tr_with_row_16_message() {
        // SPEC §6.6 row 16 — byte-exact friendly text.
        let err = parse_descriptor(
            "tr(andor(pkh(@0),after(12000000),pk(@1)))",
            &[],
            &[],
        )
        .unwrap_err();
        match err {
            ToolkitError::DescriptorParse(msg) => {
                assert_eq!(msg, BARE_TR_NO_KEY_MSG);
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn parse_descriptor_accepts_tr_nums_with_taproot_leaf_ms() {
        // V5 Q2 fold — NUMS substitution + rust-miniscript tap-leaf parse.
        // Phase 3 R0 fixture parse-check per `[[feedback-architect-must-run-prose-commands]]`.
        // Minimal tr(NUMS, <tapscript-ms>) shape with a placeholder leaf-key
        // so the toolkit can resolve via Phase 4 phrase derivation.
        //
        // If this assertion fails, rust-miniscript rejected the substituted
        // descriptor — pin the working alternative HERE before Phase 4 builds
        // golden bundles atop this shape.
        let result = parse_descriptor(
            "tr(NUMS,and_v(v:pk(@0/<0;1>/*),after(12000000)))",
            &[],
            &[],
        );
        assert!(
            result.is_ok(),
            "expected tr(NUMS, and_v(v:pk(@0), after(N))) to parse; got {:?}",
            result.err()
        );
    }
}
