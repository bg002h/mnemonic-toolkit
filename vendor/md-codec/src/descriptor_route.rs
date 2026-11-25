//! Descriptor TEXT to a decoded md1 [`Descriptor`], and from there to a
//! [`SkeletonKey`] — the library route coordinator-compatibility plan 1b
//! needs, because its evidence is recorded as descriptor text.
//!
//! Promoted from `tests/skeleton_key_conformance.rs`, where plan 1a built it
//! as a test-private walker (`md-codec` cannot dev-depend on `md-cli`). It
//! mirrors `src/to_miniscript.rs`'s forward converter arm for arm, in
//! reverse, and **self-checks every reconstruction** by feeding it back
//! through that forward converter and demanding a byte-identical re-render
//! of BOTH chains, checksum included. A wrong fingerprint, a dropped origin
//! component, a mis-numbered placeholder or a wrong use-site guess is an
//! `Err`, never a wrong key. The one field exempted is each origin-bearing
//! xpub's BIP-32 parent fingerprint, which no card can carry (F-611): it is
//! zeroed on the input side before the comparison, so a wallet's own export
//! round-trips (F-677). Depth and child number are still compared.
//!
//! The use site is always `/<0;1>/*` (`UseSitePath::standard_multipath()`),
//! and the self-check is what makes that assumption safe: a descriptor with
//! any other use site fails the round trip and is refused.
//!
//! What the round trip does NOT certify is unchanged from plan 1a: only an
//! executed walker arm is certified (the conformance test's
//! `eleven_uncovered_arms_round_trip` covers the arms its corpus misses), and
//! the round trip pins only the RENDERED projection of the descriptor.

use std::str::FromStr;
use std::sync::Arc;

use bitcoin::bip32::{ChildNumber, DerivationPath};
use bitcoin::hashes::Hash as _;
use miniscript::descriptor::{
    Descriptor as MsDescriptor, DescriptorPublicKey, ShInner, SinglePubKey, TapTree, Wsh,
};
use miniscript::{
    ForEachKey, Legacy, Miniscript, ScriptContext, Segwitv0, Tap, Terminal, Threshold,
};

use crate::canonicalize::canonicalize_placeholder_indices;
use crate::encode::Descriptor;
use crate::nums::{NUMS_H_POINT_X_ONLY_HEX, liana_unspendable_xpub};
use crate::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use crate::skeleton::{SkeletonKey, skeleton, skeleton_key};
use crate::tag::Tag;
use crate::tlv::TlvSection;
use crate::tree::{Body, InternalKey, Node};
use crate::use_site_path::UseSitePath;

/// Why descriptor text could not be turned into a [`Descriptor`] or a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteError {
    /// The text is not a BIP-380 descriptor miniscript accepts.
    Parse(String),
    /// [`descriptor_from_text`] needs the multipath `/<0;1>/*` form: a
    /// single-chain descriptor cannot say what its other chain is, and the
    /// key keeps the use site.
    NotMultipath,
    /// A multipath descriptor that does not split into exactly two chains.
    NotTwoChains(usize),
    /// A fragment or key form md1 cannot carry.
    Unsupported(String),
    /// Key `@i` carries no `[fingerprint/path]` origin, so it has no
    /// fingerprint to partition by.
    KeyWithoutOrigin(u8),
    /// Key `@i` is not an extended public key.
    NotAnXpub(u8),
    /// More than 255 keys.
    TooManyKeys,
    /// The reconstruction does not re-render to the input on this chain.
    RoundTrip {
        /// Which chain disagreed.
        chain: u32,
        /// The re-render.
        got: String,
        /// The input.
        want: String,
    },
    /// `canonicalize_placeholder_indices` refused the reconstruction.
    Canonicalize(String),
    /// [`skeleton`] refused the decoded descriptor.
    Skeleton(String),
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteError::Parse(e) => write!(f, "not a descriptor: {e}"),
            RouteError::NotMultipath => write!(
                f,
                "a single-chain descriptor: the key keeps the <0;1> use site, so pass the multipath form"
            ),
            RouteError::NotTwoChains(n) => write!(f, "multipath splits into {n} chains, not 2"),
            RouteError::Unsupported(what) => write!(f, "md1 cannot carry {what}"),
            RouteError::KeyWithoutOrigin(i) => {
                write!(f, "key @{i} carries no [fingerprint/path] origin")
            }
            RouteError::NotAnXpub(i) => write!(f, "key @{i} is not an extended public key"),
            RouteError::TooManyKeys => write!(f, "more than 255 keys"),
            RouteError::RoundTrip { chain, got, want } => write!(
                f,
                "the reconstruction does not round-trip chain {chain}: got {got}, want {want}"
            ),
            RouteError::Canonicalize(e) => write!(f, "canonicalize: {e}"),
            RouteError::Skeleton(e) => write!(f, "skeleton: {e}"),
        }
    }
}

impl std::error::Error for RouteError {}

/// Parse a MULTIPATH (`/<0;1>/*`) descriptor into a canonical md1
/// [`Descriptor`]: split it into its two chains and hand both to
/// [`descriptor_from_chains`].
///
/// # Errors
///
/// Any [`RouteError`]; a single-chain descriptor is [`RouteError::NotMultipath`].
pub fn descriptor_from_text(text: &str) -> Result<Descriptor, RouteError> {
    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(text.trim())
        .map_err(|e| RouteError::Parse(e.to_string()))?;
    if !parsed.is_multipath() {
        return Err(RouteError::NotMultipath);
    }
    let singles = parsed
        .into_single_descriptors()
        .map_err(|e| RouteError::Parse(e.to_string()))?;
    if singles.len() != 2 {
        return Err(RouteError::NotTwoChains(singles.len()));
    }
    descriptor_from_chains(&singles[0].to_string(), &singles[1].to_string())
}

/// The [`SkeletonKey`] of a multipath descriptor's text — the one function
/// both the verdict table's generator and `md shape-key --descriptor` call.
///
/// # Errors
///
/// Any [`RouteError`].
pub fn skeleton_key_of_text(text: &str) -> Result<SkeletonKey, RouteError> {
    let d = descriptor_from_text(text)?;
    let s = skeleton(&d).map_err(|e| RouteError::Skeleton(e.to_string()))?;
    Ok(skeleton_key(&s))
}

/// Reconstruct the md1 [`Descriptor`] from a descriptor's chain-0 and chain-1
/// texts, self-checking the reconstruction against BOTH before returning,
/// then canonicalizing its placeholder numbering.
///
/// # Errors
///
/// Any [`RouteError`].
pub fn descriptor_from_chains(chain0: &str, chain1: &str) -> Result<Descriptor, RouteError> {
    let desc0 = MsDescriptor::<DescriptorPublicKey>::from_str(chain0)
        .map_err(|e| RouteError::Parse(e.to_string()))?;

    let mut reg = KeyRegistry { keys: Vec::new() };
    let tree = ms_descriptor_to_node(&desc0, &mut reg)?;
    let n = u8::try_from(reg.keys.len()).map_err(|_| RouteError::TooManyKeys)?;

    let mut origin_paths = Vec::with_capacity(reg.keys.len());
    let mut fingerprints = Vec::with_capacity(reg.keys.len());
    let mut pubkeys = Vec::with_capacity(reg.keys.len());
    let mut network = bitcoin::Network::Bitcoin;
    for (i, pk) in reg.keys.iter().enumerate() {
        let idx = u8::try_from(i).map_err(|_| RouteError::TooManyKeys)?;
        let DescriptorPublicKey::XPub(x) = pk else {
            return Err(RouteError::NotAnXpub(idx));
        };
        let (fp, origin_derivation) = x.origin.clone().ok_or(RouteError::KeyWithoutOrigin(idx))?;
        if x.xkey.network != bitcoin::NetworkKind::Main {
            network = bitcoin::Network::Testnet;
        }
        origin_paths.push(derivation_path_to_origin_path(&origin_derivation));
        fingerprints.push((idx, fp.to_bytes()));
        let mut bytes = [0u8; 65];
        bytes[..32].copy_from_slice(x.xkey.chain_code.as_ref());
        bytes[32..].copy_from_slice(&x.xkey.public_key.serialize());
        pubkeys.push((idx, bytes));
    }

    let mut d = Descriptor {
        n,
        path_decl: PathDecl {
            n,
            paths: PathDeclPaths::Divergent(origin_paths),
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

    for (chain, want) in [(0u32, chain0), (1u32, chain1)] {
        let got = crate::to_miniscript::to_miniscript_descriptor_with_network(&d, chain, network)
            .map_err(|e| RouteError::Unsupported(e.to_string()))?
            .to_string();
        if got != without_parent_fingerprints(want) {
            return Err(RouteError::RoundTrip {
                chain,
                got,
                want: want.to_string(),
            });
        }
    }

    canonicalize_placeholder_indices(&mut d)
        .map_err(|e| RouteError::Canonicalize(e.to_string()))?;
    Ok(d)
}

/// `text` with the BIP-32 parent fingerprint of every ORIGIN-BEARING xpub
/// zeroed, and its checksum (if it had one) recomputed over the result -- the
/// form the self-check in [`descriptor_from_chains`] compares against.
///
/// F-677: a card stores a key as chain code + point, and the parent
/// fingerprint is hash160 of the PARENT point, which is not on the wire
/// (F-611), so every re-render carries `00000000` there. A wallet's own
/// export carries the real one, and demanding it byte-for-byte refused every
/// wallet-exported descriptor. The field takes no part in CKDpub, the script
/// or the key. Nothing else is relaxed: depth and child number are recovered
/// from the origin and still compared, and an origin-less xpub (Liana's
/// unspendable internal key, recognised by full byte equality) is untouched.
///
/// Best effort by construction: text that does not parse, or whose checksum
/// does not verify, comes back unchanged, so it fails the comparison exactly
/// as it did before.
fn without_parent_fingerprints(text: &str) -> String {
    let Ok(parsed) = MsDescriptor::<DescriptorPublicKey>::from_str(text) else {
        return text.to_string();
    };
    let mut out = text.to_string();
    parsed.for_each_key(|pk| {
        if let DescriptorPublicKey::XPub(x) = pk {
            if x.origin.is_some() && x.xkey.parent_fingerprint != Default::default() {
                let zeroed = bitcoin::bip32::Xpub {
                    parent_fingerprint: Default::default(),
                    ..x.xkey
                };
                out = out.replace(&x.xkey.to_string(), &zeroed.to_string());
            }
        }
        true
    });
    if out == text {
        return out;
    }
    match out.rsplit_once('#') {
        // `from_str` above already verified the input's checksum.
        Some((body, _)) => {
            let mut eng = miniscript::descriptor::checksum::Engine::new();
            if eng.input(body).is_err() {
                return text.to_string();
            }
            format!("{body}#{}", eng.checksum())
        }
        None => out,
    }
}

/// Placeholder indices are assigned in walk order;
/// `canonicalize_placeholder_indices` renumbers them afterwards.
struct KeyRegistry {
    keys: Vec<DescriptorPublicKey>,
}

impl KeyRegistry {
    fn register(&mut self, pk: &DescriptorPublicKey) -> Result<u8, RouteError> {
        self.keys.push(pk.clone());
        u8::try_from(self.keys.len() - 1).map_err(|_| RouteError::TooManyKeys)
    }
}

/// True iff `pk` is an origin-less xpub equal to Liana's recipe
/// ([`liana_unspendable_xpub`]) over the tap tree's own leaf keys, in
/// left-to-right leaf and key-occurrence order. FULL byte equality with the
/// recipe, never a structural match (F-449 SPEC §8.10).
fn is_liana_unspendable_key(
    pk: &DescriptorPublicKey,
    tree: Option<&TapTree<DescriptorPublicKey>>,
) -> bool {
    let xkey = match pk {
        DescriptorPublicKey::XPub(x) if x.origin.is_none() => x.xkey,
        DescriptorPublicKey::MultiXPub(x) if x.origin.is_none() => x.xkey,
        _ => return false,
    };
    let Some(t) = tree else { return false };
    let mut leaf_pubkeys: Vec<[u8; 33]> = Vec::new();
    for item in t.leaves() {
        for leaf_pk in item.miniscript().iter_pk() {
            match leaf_pk {
                DescriptorPublicKey::XPub(k) => leaf_pubkeys.push(k.xkey.public_key.serialize()),
                DescriptorPublicKey::MultiXPub(k) => {
                    leaf_pubkeys.push(k.xkey.public_key.serialize())
                }
                DescriptorPublicKey::Single(_) => return false,
            }
        }
    }
    let network = if xkey.network == bitcoin::NetworkKind::Main {
        bitcoin::Network::Bitcoin
    } else {
        bitcoin::Network::Testnet
    };
    liana_unspendable_xpub(&leaf_pubkeys, network) == xkey
}

fn is_nums_key(pk: &DescriptorPublicKey) -> bool {
    match pk {
        DescriptorPublicKey::Single(single) if single.origin.is_none() => {
            matches!(&single.key, SinglePubKey::XOnly(x) if x.to_string() == NUMS_H_POINT_X_ONLY_HEX)
        }
        _ => false,
    }
}

fn wrap1<Ctx: ScriptContext>(
    tag: Tag,
    inner: &Arc<Miniscript<DescriptorPublicKey, Ctx>>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    Ok(Node {
        tag,
        body: Body::Children(vec![terminal_to_node(inner, reg)?]),
    })
}

fn wrap2<Ctx: ScriptContext>(
    tag: Tag,
    l: &Arc<Miniscript<DescriptorPublicKey, Ctx>>,
    r: &Arc<Miniscript<DescriptorPublicKey, Ctx>>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    Ok(Node {
        tag,
        body: Body::Children(vec![terminal_to_node(l, reg)?, terminal_to_node(r, reg)?]),
    })
}

fn multikeys_node<const MAX: usize>(
    tag: Tag,
    thresh: &Threshold<DescriptorPublicKey, MAX>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    let k = u8::try_from(thresh.k()).map_err(|_| RouteError::TooManyKeys)?;
    let indices = thresh
        .data()
        .iter()
        .map(|pk| reg.register(pk))
        .collect::<Result<_, _>>()?;
    Ok(Node {
        tag,
        body: Body::MultiKeys { k, indices },
    })
}

fn leaf(tag: Tag, body: Body) -> Result<Node, RouteError> {
    Ok(Node { tag, body })
}

/// One miniscript node to one md1 `Node`. Mirrors `node_to_miniscript` in
/// `src/to_miniscript.rs`, reversed arm for arm.
fn terminal_to_node<Ctx: ScriptContext>(
    ms: &Miniscript<DescriptorPublicKey, Ctx>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    match &ms.node {
        Terminal::True => leaf(Tag::True, Body::Empty),
        Terminal::False => leaf(Tag::False, Body::Empty),
        // md1 always Check-wraps a key; a bare one is not an md1 shape.
        Terminal::PkK(_) | Terminal::PkH(_) | Terminal::RawPkH(_) => Err(RouteError::Unsupported(
            "a bare pk_k/pk_h/raw_pkh fragment".into(),
        )),
        Terminal::After(lt) => leaf(Tag::After, Body::Timelock(lt.to_consensus_u32())),
        Terminal::Older(lt) => leaf(Tag::Older, Body::Timelock(lt.to_consensus_u32())),
        Terminal::Sha256(h) => leaf(Tag::Sha256, Body::Hash256Body(h.to_byte_array())),
        Terminal::Hash256(h) => leaf(Tag::Hash256, Body::Hash256Body(h.to_byte_array())),
        Terminal::Ripemd160(h) => leaf(Tag::Ripemd160, Body::Hash160Body(h.to_byte_array())),
        Terminal::Hash160(h) => leaf(Tag::Hash160, Body::Hash160Body(h.to_byte_array())),
        Terminal::Alt(inner) => wrap1(Tag::Alt, inner, reg),
        Terminal::Swap(inner) => wrap1(Tag::Swap, inner, reg),
        Terminal::Check(inner) => {
            // `Check(PkK)` / `Check(PkH)` is the bare `pk(...)` / `pkh(...)`
            // fragment on the wire -- the collapse `node_to_miniscript`'s
            // `Tag::Check` arm re-applies forward.
            match &inner.node {
                Terminal::PkK(pk) => {
                    let index = reg.register(pk)?;
                    leaf(Tag::PkK, Body::KeyArg { index })
                }
                Terminal::PkH(pk) => {
                    let index = reg.register(pk)?;
                    leaf(Tag::PkH, Body::KeyArg { index })
                }
                _ => wrap1(Tag::Check, inner, reg),
            }
        }
        Terminal::DupIf(inner) => wrap1(Tag::DupIf, inner, reg),
        Terminal::Verify(inner) => wrap1(Tag::Verify, inner, reg),
        Terminal::NonZero(inner) => wrap1(Tag::NonZero, inner, reg),
        Terminal::ZeroNotEqual(inner) => wrap1(Tag::ZeroNotEqual, inner, reg),
        Terminal::AndV(l, r) => wrap2(Tag::AndV, l, r, reg),
        Terminal::AndB(l, r) => wrap2(Tag::AndB, l, r, reg),
        Terminal::AndOr(a, b, c) => Ok(Node {
            tag: Tag::AndOr,
            body: Body::Children(vec![
                terminal_to_node(a, reg)?,
                terminal_to_node(b, reg)?,
                terminal_to_node(c, reg)?,
            ]),
        }),
        Terminal::OrB(l, r) => wrap2(Tag::OrB, l, r, reg),
        Terminal::OrC(l, r) => wrap2(Tag::OrC, l, r, reg),
        Terminal::OrD(l, r) => wrap2(Tag::OrD, l, r, reg),
        Terminal::OrI(l, r) => wrap2(Tag::OrI, l, r, reg),
        Terminal::Thresh(thresh) => {
            let k = u8::try_from(thresh.k()).map_err(|_| RouteError::TooManyKeys)?;
            let children = thresh
                .data()
                .iter()
                .map(|c| terminal_to_node(c, reg))
                .collect::<Result<_, _>>()?;
            Ok(Node {
                tag: Tag::Thresh,
                body: Body::Variable { k, children },
            })
        }
        Terminal::Multi(thresh) => multikeys_node(Tag::Multi, thresh, reg),
        Terminal::SortedMulti(thresh) => multikeys_node(Tag::SortedMulti, thresh, reg),
        Terminal::MultiA(thresh) => multikeys_node(Tag::MultiA, thresh, reg),
        Terminal::SortedMultiA(thresh) => multikeys_node(Tag::SortedMultiA, thresh, reg),
    }
}

/// `wsh(...)`'s single child. `sortedmulti` is legal here only at the root.
fn wsh_to_node(wsh: &Wsh<DescriptorPublicKey>, reg: &mut KeyRegistry) -> Result<Node, RouteError> {
    if let Terminal::SortedMulti(thresh) = &wsh.as_inner().node {
        return multikeys_node(Tag::SortedMulti, thresh, reg);
    }
    terminal_to_node::<Segwitv0>(wsh.as_inner(), reg)
}

/// `sh(...)`'s single child. Mirrors `sh_inner_to_descriptor`.
fn sh_inner_to_node(
    inner: &ShInner<DescriptorPublicKey>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    match inner {
        ShInner::Wpkh(w) => {
            let index = reg.register(w.as_inner())?;
            leaf(Tag::Wpkh, Body::KeyArg { index })
        }
        ShInner::Wsh(wsh) => Ok(Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![wsh_to_node(wsh, reg)?]),
        }),
        ShInner::Ms(ms) => {
            if let Terminal::SortedMulti(thresh) = &ms.node {
                return multikeys_node(Tag::SortedMulti, thresh, reg);
            }
            terminal_to_node::<Legacy>(ms, reg)
        }
    }
}

/// One tap leaf's root. `sortedmulti_a` is legal only here.
fn tap_leaf_to_node(
    ms: &Miniscript<DescriptorPublicKey, Tap>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    if let Terminal::SortedMultiA(thresh) = &ms.node {
        return multikeys_node(Tag::SortedMultiA, thresh, reg);
    }
    terminal_to_node::<Tap>(ms, reg)
}

/// Rebuild the binary `Tag::TapTree` node from `TapTree::leaves()`'s
/// depth-first `(depth, leaf)` list by combining adjacent equal-depth
/// siblings. A single-leaf tree falls out as the bare leaf node.
fn taptree_to_node(
    tt: &TapTree<DescriptorPublicKey>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    let mut stack: Vec<(u8, Node)> = Vec::new();
    for item in tt.leaves() {
        let mut node = tap_leaf_to_node(item.miniscript(), reg)?;
        let mut depth = item.depth();
        while let Some(&(top_depth, _)) = stack.last() {
            if top_depth != depth {
                break;
            }
            let (_, left) = stack.pop().expect("just peeked");
            node = Node {
                tag: Tag::TapTree,
                body: Body::Children(vec![left, node]),
            };
            depth -= 1;
        }
        stack.push((depth, node));
    }
    match (stack.pop(), stack.is_empty()) {
        (Some((_, root)), true) => Ok(root),
        _ => Err(RouteError::Unsupported(
            "a tap tree that does not converge to one root".into(),
        )),
    }
}

fn ms_descriptor_to_node(
    desc: &MsDescriptor<DescriptorPublicKey>,
    reg: &mut KeyRegistry,
) -> Result<Node, RouteError> {
    match desc {
        MsDescriptor::Wpkh(w) => {
            let index = reg.register(w.as_inner())?;
            leaf(Tag::Wpkh, Body::KeyArg { index })
        }
        MsDescriptor::Pkh(p) => {
            let index = reg.register(p.as_inner())?;
            leaf(Tag::Pkh, Body::KeyArg { index })
        }
        MsDescriptor::Sh(sh) => Ok(Node {
            tag: Tag::Sh,
            body: Body::Children(vec![sh_inner_to_node(sh.as_inner(), reg)?]),
        }),
        MsDescriptor::Wsh(wsh) => Ok(Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![wsh_to_node(wsh, reg)?]),
        }),
        MsDescriptor::Tr(tr) => {
            let internal_key = if is_nums_key(tr.internal_key()) {
                InternalKey::NumsPoint
            } else if is_liana_unspendable_key(tr.internal_key(), tr.tap_tree()) {
                InternalKey::LianaUnspendable
            } else {
                InternalKey::Slot(reg.register(tr.internal_key())?)
            };
            let tree = match tr.tap_tree() {
                Some(tt) => Some(Box::new(taptree_to_node(tt, reg)?)),
                None => None,
            };
            Ok(Node {
                tag: Tag::Tr,
                body: Body::Tr { internal_key, tree },
            })
        }
        MsDescriptor::Bare(_) => Err(RouteError::Unsupported("a bare top-level script".into())),
    }
}

fn derivation_path_to_origin_path(p: &DerivationPath) -> OriginPath {
    let components = p
        .as_ref()
        .iter()
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
        .collect();
    OriginPath { components }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F-677: the self-check now recomputes a wallet export's checksum after
    /// zeroing its parent fingerprints, so it must not launder a checksum
    /// that was wrong to begin with. Chain 1 is never parsed on its own
    /// before the comparison, which makes it the chain where that could
    /// hide. Mutation: parse `text`'s body with the checksum stripped in
    /// `without_parent_fingerprints` -> the tampered chain 1 is accepted and
    /// this reds.
    #[test]
    fn a_wallet_export_with_a_bad_chain1_checksum_is_still_refused() {
        let chain = |c: u32| {
            format!(
                "wsh(or_i(pkh([73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf/{c}/*),and_v(v:pkh([3f635a63/48'/0'/0'/2']xpub6FHZCoNb3tg3o1GAJQxSwgFNF8mLRtTk2GgkF7n5rwzoxBhUEdFWa8cyZRHqytAzKZWsKz8627cQEMCCfR5GDSv6yXegqirpgDUX41Pxybr/{c}/*),older(26280))))"
            )
        };
        let with_sum = |body: String| {
            let mut eng = miniscript::descriptor::checksum::Engine::new();
            eng.input(&body).unwrap();
            format!("{body}#{}", eng.checksum())
        };
        let (c0, c1) = (with_sum(chain(0)), with_sum(chain(1)));
        assert!(descriptor_from_chains(&c0, &c1).is_ok());
        let last = c1.chars().last().unwrap();
        let bad = format!(
            "{}{}",
            &c1[..c1.len() - 1],
            if last == 'q' { 'p' } else { 'q' }
        );
        assert!(matches!(
            descriptor_from_chains(&c0, &bad),
            Err(RouteError::RoundTrip { chain: 1, .. })
        ));
    }

    /// F-655: the recogniser's NEAR MISS. Liana's recipe over the SAME leaves
    /// in a DIFFERENT order is a different chain code, so it must not be
    /// recognised. Mutation: compare only `public_key` (the NUMS point every
    /// candidate shares) instead of the whole xpub -> this test reds.
    #[test]
    fn a_liana_key_derived_over_permuted_leaves_is_not_recognised() {
        let a = "xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zTpf7F2BuXCJ52i";
        let b = "xpub6DXuQW1Q2JpZyteDRGW1pD34uhumfnZJfTsmjDkgd4xcq3L5XX2KUE1n4rmvcDT3RmdchfhbD9DkvSyVhUMBjUYi691iFszgKtf4Bfqe2nL";
        let xa: bitcoin::bip32::Xpub = a.parse().unwrap();
        let xb: bitcoin::bip32::Xpub = b.parse().unwrap();
        let ab = liana_unspendable_xpub(
            &[xa.public_key.serialize(), xb.public_key.serialize()],
            bitcoin::Network::Bitcoin,
        );
        let ba = liana_unspendable_xpub(
            &[xb.public_key.serialize(), xa.public_key.serialize()],
            bitcoin::Network::Bitcoin,
        );
        let leaves =
            format!("{{pk([73c5da0a/48'/0'/0'/3']{a}/0/*),pk([3f635a63/48'/0'/0'/3']{b}/0/*)}}");
        let parse = |ik: &bitcoin::bip32::Xpub| {
            MsDescriptor::<DescriptorPublicKey>::from_str(&format!("tr({ik}/0/*,{leaves})"))
                .unwrap()
        };
        let right = parse(&ab);
        let wrong = parse(&ba);
        let (MsDescriptor::Tr(r), MsDescriptor::Tr(w)) = (&right, &wrong) else {
            unreachable!()
        };
        assert!(is_liana_unspendable_key(r.internal_key(), r.tap_tree()));
        assert!(!is_liana_unspendable_key(w.internal_key(), w.tap_tree()));
    }
}
