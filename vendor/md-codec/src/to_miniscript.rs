//! v0.32 AST → `miniscript::Descriptor<DescriptorPublicKey>` converter.
//!
//! Replaces the v0.14-era hand-rolled 5-shape allow-list with a generic
//! converter that builds a miniscript `Descriptor` from any
//! BIP-388-parseable md1 wire AST. Address derivation
//! ([`crate::Descriptor::derive_address`]) delegates to this module then
//! to `miniscript::Descriptor::address`.
//!
//! Feature-gated behind `derive` (default-on).

use crate::canonicalize::{ExpandedKey, expand_per_at_n};
use crate::derive::xpub_from_tlv_bytes;
use crate::encode::Descriptor;
use crate::error::Error;
use crate::origin_path::OriginPath;
use crate::tag::Tag;
use crate::tree::{Body, Node};
use crate::use_site_path::UseSitePath;

use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint};
use miniscript::descriptor::{
    DerivPaths, DescriptorMultiXKey, DescriptorPublicKey, DescriptorXKey, SinglePub, SinglePubKey,
    Wildcard,
};
use miniscript::miniscript::limits::{MAX_PUBKEYS_IN_CHECKSIGADD, MAX_PUBKEYS_PER_MULTISIG};
use miniscript::{
    AbsLockTime, Legacy, Miniscript, RelLockTime, ScriptContext, Segwitv0, Tap, Terminal, Threshold,
};
use std::str::FromStr;
use std::sync::Arc;

/// BIP-341 NUMS H-point x-only coordinate. Used as the internal key when
/// `Body::Tr { is_nums: true, .. }`.
const NUMS_H_POINT_X_ONLY_HEX: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Convert an md1 [`Descriptor`] AST to a
/// `miniscript::Descriptor<DescriptorPublicKey>` for `chain` (the
/// multipath alt selector). The trailing wildcard `/*` remains for
/// `miniscript::Descriptor::at_derivation_index` to resolve.
///
/// `chain` is resolved in-place during key construction (multipath alt
/// substituted into each `DescriptorXKey.derivation_path`); the resulting
/// `Descriptor` is single-path.
///
/// # Errors
///
/// - [`Error::MissingPubkey`] / [`Error::InvalidXpubBytes`] /
///   [`Error::MissingExplicitOrigin`] propagated from
///   [`expand_per_at_n`].
/// - [`Error::AddressDerivationFailed`] wrapping any miniscript-layer
///   failure (type check, context error, unsupported fragment) or arity
///   mismatch raised by the converter.
pub fn to_miniscript_descriptor(
    d: &Descriptor,
    chain: u32,
) -> Result<miniscript::Descriptor<DescriptorPublicKey>, Error> {
    let expanded = expand_per_at_n(d)?;
    let mut keys: Vec<DescriptorPublicKey> = Vec::with_capacity(expanded.len());
    for e in &expanded {
        // D1 (faithful per-key reconstruction): each `@N` derives at its
        // OWN already-resolved use-site path (`e.use_site_path`), NOT the
        // shared descriptor baseline. Passing `&d.use_site_path` here was
        // the silent-wrong-address bug for per-cosigner override cards.
        keys.push(build_descriptor_public_key(e, &e.use_site_path, chain)?);
    }
    node_to_descriptor(&d.tree, &keys)
}

/// Returns `true` if ANY use-site path on `d` requires a hardened public
/// derivation step — the descriptor-level baseline (`d.use_site_path`) OR
/// any per-`@N` entry in `d.tlv.use_site_path_overrides`. "Hardened
/// anywhere" means a hardened wildcard (`/*h`) OR any hardened
/// [`Alternative`](crate::use_site_path::Alternative) inside a multipath
/// group.
///
/// BIP 32 forbids hardened derivation from an xpub, so an xpub-only restore
/// cannot produce addresses for such a wallet. This is the single source of
/// truth ("Point B") for the hardened-derivation refusal: the derivation
/// boundary ([`Descriptor::derive_address`](crate::Descriptor::derive_address))
/// uses it to refuse cleanly with [`Error::HardenedPublicDerivation`], and
/// downstream consumers (e.g. the toolkit `restore` guard + advisory) reuse
/// the SAME predicate so refusal and advisory stay in exact parity.
///
/// Note: this scans `use_site_path_overrides` directly (not the
/// `expand_per_at_n` resolution), because the override set is exactly the
/// per-`@N` divergent paths; a key with no override inherits the baseline,
/// which is already covered by the `d.use_site_path` scan.
pub fn has_hardened_use_site(d: &Descriptor) -> bool {
    use_site_is_hardened(&d.use_site_path)
        || d.tlv
            .use_site_path_overrides
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .any(|(_, usp)| use_site_is_hardened(usp))
}

/// Returns `true` if `u` has a hardened wildcard or any hardened multipath
/// alternative.
fn use_site_is_hardened(u: &UseSitePath) -> bool {
    u.wildcard_hardened
        || u.multipath
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .any(|a| a.hardened)
}

/// Origin metadata for one expanded `@N`: `(fingerprint, origin_path)` when
/// a `Fingerprints` TLV entry is present, else `None`. Shared by the
/// single-path and multipath key builders so the two paths can never drift
/// on origin/xkey assembly.
type DescriptorOrigin = Option<(Fingerprint, DerivationPath)>;

/// Assemble the `(origin, xkey)` pair common to both
/// [`build_descriptor_public_key`] (single-path) and
/// [`build_descriptor_multi_public_key`] (multipath) for one expanded `@N`.
fn assemble_origin_and_xkey(
    e: &ExpandedKey,
) -> Result<(DescriptorOrigin, bitcoin::bip32::Xpub), Error> {
    let xpub_bytes = e.xpub.ok_or(Error::MissingPubkey { idx: e.idx })?;
    let xkey = xpub_from_tlv_bytes(e.idx, &xpub_bytes)?;
    let origin = e.fingerprint.map(|fp| {
        (
            Fingerprint::from(fp),
            origin_path_to_derivation(&e.origin_path),
        )
    });
    Ok((origin, xkey))
}

/// `Wildcard::Hardened` for a `/*h` use-site, else `Wildcard::Unhardened`.
/// Hardened wildcards are pre-refused at the derivation boundary
/// ([`has_hardened_use_site`]); this only governs the rendered text.
fn wildcard_for(use_site: &UseSitePath) -> Wildcard {
    if use_site.wildcard_hardened {
        Wildcard::Hardened
    } else {
        Wildcard::Unhardened
    }
}

/// Build a `DescriptorPublicKey::XPub` for one expanded `@N`, with the
/// chain alt substituted into `derivation_path` in-place. Single-path:
/// resolves the `chain`-th multipath alternative now.
fn build_descriptor_public_key(
    e: &ExpandedKey,
    use_site: &UseSitePath,
    chain: u32,
) -> Result<DescriptorPublicKey, Error> {
    let (origin, xkey) = assemble_origin_and_xkey(e)?;

    // Derivation path is the use-site multipath alt (without the trailing
    // wildcard, which is handled via the `wildcard` field below).
    let derivation_path = use_site_to_derivation_path(use_site, chain)?;

    Ok(DescriptorPublicKey::XPub(DescriptorXKey {
        origin,
        xkey,
        derivation_path,
        wildcard: wildcard_for(use_site),
    }))
}

/// Build a `DescriptorPublicKey` for one expanded `@N` carrying its FULL
/// use-site multipath GROUP (not a single resolved chain). Used by
/// [`to_miniscript_descriptor_multipath`] to render the faithful
/// descriptor STRING with per-`@N` `<…;…>` groups.
///
/// - `e.use_site_path.multipath = Some(alts)` → `MultiXPub` with one
///   `DerivationPath` per alternative (e.g. `<2;3>` → `[m/2, m/3]`).
/// - `e.use_site_path.multipath = None` → a single-path `XPub` (bare `/*`).
///
/// rust-miniscript's `into_single_descriptors` selects each key's own alt
/// at derivation time, so per-`@N` groups stay faithful end-to-end (and
/// `sortedmulti` sorts the per-index-derived keys correctly).
fn build_descriptor_multi_public_key(e: &ExpandedKey) -> Result<DescriptorPublicKey, Error> {
    let (origin, xkey) = assemble_origin_and_xkey(e)?;
    let use_site = &e.use_site_path;
    let wildcard = wildcard_for(use_site);

    match &use_site.multipath {
        Some(alts) => {
            let paths: Vec<DerivationPath> = alts
                .iter()
                .map(|a| {
                    let child = if a.hardened {
                        ChildNumber::from_hardened_idx(a.value)
                            .unwrap_or(ChildNumber::Hardened { index: a.value })
                    } else {
                        ChildNumber::Normal { index: a.value }
                    };
                    DerivationPath::from(vec![child])
                })
                .collect();
            let derivation_paths = DerivPaths::new(paths)
                .ok_or_else(|| failed(format!("@{} multipath group is empty", e.idx)))?;
            Ok(DescriptorPublicKey::MultiXPub(DescriptorMultiXKey {
                origin,
                xkey,
                derivation_paths,
                wildcard,
            }))
        }
        None => Ok(DescriptorPublicKey::XPub(DescriptorXKey {
            origin,
            xkey,
            derivation_path: DerivationPath::master(),
            wildcard,
        })),
    }
}

/// Convert an md1 [`Descriptor`] AST to a *multipath*
/// `miniscript::Descriptor<DescriptorPublicKey>` — one key per `@N`
/// carrying its FULL resolved use-site multipath group (`<…;…>`), not the
/// single-chain collapse of [`to_miniscript_descriptor`].
///
/// This is the faithful descriptor-STRING entry for per-cosigner use-site
/// override cards: each `@N`'s group comes from `ExpandedKey.use_site_path`
/// (per-`@N` override composed over the baseline) where `@N` == the
/// `expand_per_at_n` Vec position — the unambiguous correspondence. A
/// `None`-multipath override renders as a single-path `XPub` (bare `/*`)
/// while sibling keys stay `MultiXPub` (the legal `Some`/`None` mix).
///
/// The trailing `/*` wildcards remain for
/// `miniscript::Descriptor::into_single_descriptors` /
/// `at_derivation_index` to resolve. The result is NOT single-path; callers
/// that need addresses call `into_single_descriptors` (rust-miniscript
/// selects each key's own alt per chain).
///
/// Hardened use-site cards are pre-refused by callers via
/// [`has_hardened_use_site`]; a hardened alt reaching this builder renders a
/// hardened child in the group (still a valid descriptor string, never a
/// wrong address — it is never asked to *derive*).
///
/// # Errors
///
/// Same propagation as [`to_miniscript_descriptor`]:
/// [`Error::MissingPubkey`] / [`Error::InvalidXpubBytes`] /
/// [`Error::MissingExplicitOrigin`] from [`expand_per_at_n`], and
/// [`Error::AddressDerivationFailed`] wrapping any miniscript-layer failure.
pub fn to_miniscript_descriptor_multipath(
    d: &Descriptor,
) -> Result<miniscript::Descriptor<DescriptorPublicKey>, Error> {
    let expanded = expand_per_at_n(d)?;
    let mut keys: Vec<DescriptorPublicKey> = Vec::with_capacity(expanded.len());
    for e in &expanded {
        keys.push(build_descriptor_multi_public_key(e)?);
    }
    node_to_descriptor(&d.tree, &keys)
}

/// Translate an `OriginPath` into a `bip32::DerivationPath`.
fn origin_path_to_derivation(p: &OriginPath) -> DerivationPath {
    let children: Vec<ChildNumber> = p
        .components
        .iter()
        .map(|c| {
            if c.hardened {
                ChildNumber::from_hardened_idx(c.value)
                    .unwrap_or(ChildNumber::Hardened { index: c.value })
            } else {
                ChildNumber::from_normal_idx(c.value)
                    .unwrap_or(ChildNumber::Normal { index: c.value })
            }
        })
        .collect();
    DerivationPath::from(children)
}

/// Build the per-key `derivation_path` from the use-site multipath: a
/// single `ChildNumber` for `multipath[chain]`, or empty when no
/// multipath. The trailing `/*` wildcard is encoded via the `wildcard`
/// field, not the path.
fn use_site_to_derivation_path(u: &UseSitePath, chain: u32) -> Result<DerivationPath, Error> {
    let mut comps: Vec<ChildNumber> = Vec::new();
    if let Some(alts) = &u.multipath {
        let alt = alts
            .get(chain as usize)
            .ok_or(Error::ChainIndexOutOfRange {
                chain,
                alt_count: alts.len(),
            })?;
        if alt.hardened {
            return Err(Error::HardenedPublicDerivation);
        }
        comps.push(ChildNumber::Normal { index: alt.value });
    }
    Ok(DerivationPath::from(comps))
}

/// Map an md1 top-level tree node onto a `miniscript::Descriptor`.
fn node_to_descriptor(
    node: &Node,
    keys: &[DescriptorPublicKey],
) -> Result<miniscript::Descriptor<DescriptorPublicKey>, Error> {
    match (&node.tag, &node.body) {
        (Tag::Pkh, Body::KeyArg { index }) => {
            let pk = lookup_key(keys, *index)?;
            miniscript::Descriptor::new_pkh(pk).map_err(|e| failed(e.to_string()))
        }
        (Tag::Wpkh, Body::KeyArg { index }) => {
            let pk = lookup_key(keys, *index)?;
            miniscript::Descriptor::new_wpkh(pk).map_err(|e| failed(e.to_string()))
        }
        (Tag::Sh, Body::Children(children)) if children.len() == 1 => {
            sh_inner_to_descriptor(&children[0], keys)
        }
        (Tag::Wsh, Body::Children(children)) if children.len() == 1 => {
            wsh_inner_to_descriptor(&children[0], keys)
        }
        (
            Tag::Tr,
            Body::Tr {
                is_nums,
                key_index,
                tree,
            },
        ) => {
            let internal_key = if *is_nums {
                build_nums_internal_key()?
            } else {
                lookup_key(keys, *key_index)?
            };
            let script_tree = if let Some(t) = tree {
                Some(tree_to_taptree(t, keys)?)
            } else {
                None
            };
            miniscript::Descriptor::new_tr(internal_key, script_tree)
                .map_err(|e| failed(e.to_string()))
        }
        _ => Err(failed(format!(
            "unsupported top-level tag {:?} with body shape",
            node.tag
        ))),
    }
}

/// Build the NUMS-point `DescriptorPublicKey` (BIP-341 H-point as a
/// single x-only descriptor key, no origin/path).
fn build_nums_internal_key() -> Result<DescriptorPublicKey, Error> {
    let x_only = bitcoin::secp256k1::XOnlyPublicKey::from_str(NUMS_H_POINT_X_ONLY_HEX)
        .map_err(|e| failed(format!("NUMS x-only parse: {e}")))?;
    Ok(DescriptorPublicKey::Single(SinglePub {
        origin: None,
        key: SinglePubKey::XOnly(x_only),
    }))
}

/// Descriptor-level `wsh(...)` inner: choose between
/// `Descriptor::new_wsh_sortedmulti` and `Descriptor::new_wsh(<Miniscript>)`.
fn wsh_inner_to_descriptor(
    inner: &Node,
    keys: &[DescriptorPublicKey],
) -> Result<miniscript::Descriptor<DescriptorPublicKey>, Error> {
    if let (Tag::SortedMulti, Body::MultiKeys { k, indices }) = (&inner.tag, &inner.body) {
        let thresh = build_multi_threshold::<{ MAX_PUBKEYS_PER_MULTISIG }>(
            *k,
            indices,
            keys,
            "wsh-sortedmulti",
        )?;
        return miniscript::Descriptor::new_wsh_sortedmulti(thresh)
            .map_err(|e| failed(e.to_string()));
    }
    let ms = node_to_miniscript::<Segwitv0>(inner, keys)?;
    miniscript::Descriptor::new_wsh(ms).map_err(|e| failed(e.to_string()))
}

/// Descriptor-level `sh(...)` inner: dispatch between `sh(wsh(...))`,
/// `sh(wpkh(...))`, `sh(sortedmulti(...))`, and `sh(<miniscript>)`
/// (Legacy context).
fn sh_inner_to_descriptor(
    inner: &Node,
    keys: &[DescriptorPublicKey],
) -> Result<miniscript::Descriptor<DescriptorPublicKey>, Error> {
    match (&inner.tag, &inner.body) {
        (Tag::Wsh, Body::Children(grand)) if grand.len() == 1 => {
            let grandchild = &grand[0];
            if let (Tag::SortedMulti, Body::MultiKeys { k, indices }) =
                (&grandchild.tag, &grandchild.body)
            {
                let thresh = build_multi_threshold::<{ MAX_PUBKEYS_PER_MULTISIG }>(
                    *k,
                    indices,
                    keys,
                    "sh-wsh-sortedmulti",
                )?;
                return miniscript::Descriptor::new_sh_wsh_sortedmulti(thresh)
                    .map_err(|e| failed(e.to_string()));
            }
            let ms = node_to_miniscript::<Segwitv0>(grandchild, keys)?;
            miniscript::Descriptor::new_sh_wsh(ms).map_err(|e| failed(e.to_string()))
        }
        (Tag::Wpkh, Body::KeyArg { index }) => {
            let pk = lookup_key(keys, *index)?;
            miniscript::Descriptor::new_sh_wpkh(pk).map_err(|e| failed(e.to_string()))
        }
        (Tag::SortedMulti, Body::MultiKeys { k, indices }) => {
            let thresh = build_multi_threshold::<{ MAX_PUBKEYS_PER_MULTISIG }>(
                *k,
                indices,
                keys,
                "sh-sortedmulti",
            )?;
            miniscript::Descriptor::new_sh_sortedmulti(thresh).map_err(|e| failed(e.to_string()))
        }
        _ => {
            let ms = node_to_miniscript::<Legacy>(inner, keys)?;
            miniscript::Descriptor::new_sh(ms).map_err(|e| failed(e.to_string()))
        }
    }
}

/// Recurse into a tap-script-tree node. Returns a `miniscript::TapTree`.
fn tree_to_taptree(
    node: &Node,
    keys: &[DescriptorPublicKey],
) -> Result<miniscript::descriptor::TapTree<DescriptorPublicKey>, Error> {
    if let (Tag::TapTree, Body::Children(children)) = (&node.tag, &node.body) {
        if children.len() != 2 {
            return Err(failed(format!(
                "Tag::TapTree expected 2 children, got {}",
                children.len()
            )));
        }
        let l = tree_to_taptree(&children[0], keys)?;
        let r = tree_to_taptree(&children[1], keys)?;
        return miniscript::descriptor::TapTree::combine(l, r)
            .map_err(|e| failed(format!("TapTree depth: {e}")));
    }
    // Single bare leaf — including the v0.30 single-leaf wire optimization
    // where `Body::Tr { tree: Some(<bare PkK Node>) }` skips the `Tag::TapTree`
    // wrap.
    let ms = node_to_miniscript::<Tap>(node, keys)?;
    Ok(miniscript::descriptor::TapTree::leaf(Arc::new(ms)))
}

/// Convert a miniscript-leaf md1 node into a `Miniscript<Pk, Ctx>`.
fn node_to_miniscript<Ctx>(
    node: &Node,
    keys: &[DescriptorPublicKey],
) -> Result<Miniscript<DescriptorPublicKey, Ctx>, Error>
where
    Ctx: ScriptContext,
{
    let term: Terminal<DescriptorPublicKey, Ctx> = match (&node.tag, &node.body) {
        (Tag::PkK, Body::KeyArg { index }) => {
            // Phase E: bare PkK always emits as Check(pk_k(...)) since
            // miniscript leaves require a `K` (check'd-key) at any
            // satisfied position. md1 wire normalises by stripping the
            // outer `c:` wrapper; re-apply here.
            let pk = lookup_key(keys, *index)?;
            let inner = Miniscript::from_ast(Terminal::PkK(pk)).map_err(into_failed)?;
            Terminal::Check(Arc::new(inner))
        }
        (Tag::PkH, Body::KeyArg { index }) => {
            let pk = lookup_key(keys, *index)?;
            let inner = Miniscript::from_ast(Terminal::PkH(pk)).map_err(into_failed)?;
            Terminal::Check(Arc::new(inner))
        }
        (Tag::Check, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 1)?;
            // Check-idempotence (`to-miniscript-check-pkh-double-wrap`): a
            // `Tag::Check` over a BARE key tag denotes the same fragment as the
            // bare tag — both mean `c:pk_k`/`c:pk_h` (type B), and the PkK/PkH
            // arms above already re-apply `Check`. Wrapping a second `Check`
            // yields `Check(Check(PkH))` = `c:` over type-B → "cannot wrap a
            // fragment of type B". The toolkit walker (non-tap context) and
            // pre-v0.30 md-cli cards both emit this `Tag::Check(Tag::PkK/PkH)`
            // wire shape, and such cards are already engraved — the renderer
            // must accept it. A `Tag::Check` whose child is NOT a bare key
            // (`Check(Check(..))`, shape C `Check(or_i(pk_k,pk_k))`) still
            // double-wraps below and correctly errors — never a wrong descriptor.
            if matches!(
                (&children[0].tag, &children[0].body),
                (Tag::PkK | Tag::PkH, Body::KeyArg { .. })
            ) {
                return node_to_miniscript::<Ctx>(&children[0], keys);
            }
            Terminal::Check(Arc::new(node_to_miniscript::<Ctx>(&children[0], keys)?))
        }
        (Tag::Verify, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 1)?;
            Terminal::Verify(Arc::new(node_to_miniscript::<Ctx>(&children[0], keys)?))
        }
        (Tag::Swap, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 1)?;
            Terminal::Swap(Arc::new(node_to_miniscript::<Ctx>(&children[0], keys)?))
        }
        (Tag::Alt, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 1)?;
            Terminal::Alt(Arc::new(node_to_miniscript::<Ctx>(&children[0], keys)?))
        }
        (Tag::DupIf, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 1)?;
            Terminal::DupIf(Arc::new(node_to_miniscript::<Ctx>(&children[0], keys)?))
        }
        (Tag::NonZero, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 1)?;
            Terminal::NonZero(Arc::new(node_to_miniscript::<Ctx>(&children[0], keys)?))
        }
        (Tag::ZeroNotEqual, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 1)?;
            Terminal::ZeroNotEqual(Arc::new(node_to_miniscript::<Ctx>(&children[0], keys)?))
        }
        (Tag::AndV, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 2)?;
            let l = node_to_miniscript::<Ctx>(&children[0], keys)?;
            let r = node_to_miniscript::<Ctx>(&children[1], keys)?;
            Terminal::AndV(Arc::new(l), Arc::new(r))
        }
        (Tag::AndB, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 2)?;
            let l = node_to_miniscript::<Ctx>(&children[0], keys)?;
            let r = node_to_miniscript::<Ctx>(&children[1], keys)?;
            Terminal::AndB(Arc::new(l), Arc::new(r))
        }
        (Tag::AndOr, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 3)?;
            let a = node_to_miniscript::<Ctx>(&children[0], keys)?;
            let b = node_to_miniscript::<Ctx>(&children[1], keys)?;
            let c = node_to_miniscript::<Ctx>(&children[2], keys)?;
            Terminal::AndOr(Arc::new(a), Arc::new(b), Arc::new(c))
        }
        (Tag::OrB, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 2)?;
            let l = node_to_miniscript::<Ctx>(&children[0], keys)?;
            let r = node_to_miniscript::<Ctx>(&children[1], keys)?;
            Terminal::OrB(Arc::new(l), Arc::new(r))
        }
        (Tag::OrC, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 2)?;
            let l = node_to_miniscript::<Ctx>(&children[0], keys)?;
            let r = node_to_miniscript::<Ctx>(&children[1], keys)?;
            Terminal::OrC(Arc::new(l), Arc::new(r))
        }
        (Tag::OrD, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 2)?;
            let l = node_to_miniscript::<Ctx>(&children[0], keys)?;
            let r = node_to_miniscript::<Ctx>(&children[1], keys)?;
            Terminal::OrD(Arc::new(l), Arc::new(r))
        }
        (Tag::OrI, Body::Children(children)) => {
            arity_eq(node.tag, children.len(), 2)?;
            let l = node_to_miniscript::<Ctx>(&children[0], keys)?;
            let r = node_to_miniscript::<Ctx>(&children[1], keys)?;
            Terminal::OrI(Arc::new(l), Arc::new(r))
        }
        (Tag::Thresh, Body::Variable { k, children }) => {
            let mut subs: Vec<Arc<Miniscript<DescriptorPublicKey, Ctx>>> =
                Vec::with_capacity(children.len());
            for c in children {
                subs.push(Arc::new(node_to_miniscript::<Ctx>(c, keys)?));
            }
            let thresh =
                Threshold::<_, 0>::new(*k as usize, subs).map_err(|e| failed(e.to_string()))?;
            Terminal::Thresh(thresh)
        }
        (Tag::Multi, Body::MultiKeys { k, indices }) => {
            // `Terminal::Multi` is `Ctx`-generic at the variant level;
            // rust-miniscript's `Miniscript::from_ast` enforces context-
            // appropriateness (rejects Multi inside Tap, MultiA inside
            // Segwitv0) via `check_global_consensus_validity`.
            let thresh =
                build_multi_threshold::<{ MAX_PUBKEYS_PER_MULTISIG }>(*k, indices, keys, "multi")?;
            Terminal::Multi(thresh)
        }
        (Tag::MultiA, Body::MultiKeys { k, indices }) => {
            let thresh = build_multi_threshold::<{ MAX_PUBKEYS_IN_CHECKSIGADD }>(
                *k, indices, keys, "multi_a",
            )?;
            Terminal::MultiA(thresh)
        }
        (Tag::SortedMulti, Body::MultiKeys { .. }) => {
            return Err(failed(
                "Tag::SortedMulti must be the sole child of wsh/sh; cannot appear as a miniscript leaf"
                    .to_string(),
            ));
        }
        (Tag::SortedMultiA, Body::MultiKeys { .. }) => {
            return Err(failed(
                "Tag::SortedMultiA must be a tap-leaf root child; rust-miniscript v13 has no Terminal::SortedMultiA fragment"
                    .to_string(),
            ));
        }
        (Tag::After, Body::Timelock(v)) => {
            let lt = AbsLockTime::from_consensus(*v).map_err(|e| failed(e.to_string()))?;
            Terminal::After(lt)
        }
        (Tag::Older, Body::Timelock(v)) => {
            let lt = RelLockTime::from_consensus(*v).map_err(|e| failed(e.to_string()))?;
            Terminal::Older(lt)
        }
        (Tag::Sha256, Body::Hash256Body(h)) => {
            let hash = sha256_from_bytes(h)?;
            Terminal::Sha256(hash)
        }
        (Tag::Hash256, Body::Hash256Body(h)) => {
            let hash = hash256_from_bytes(h)?;
            Terminal::Hash256(hash)
        }
        (Tag::Ripemd160, Body::Hash160Body(h)) => {
            let hash = ripemd160_from_bytes(h)?;
            Terminal::Ripemd160(hash)
        }
        (Tag::Hash160, Body::Hash160Body(h)) => {
            let hash = hash160_from_bytes(h)?;
            Terminal::Hash160(hash)
        }
        (Tag::RawPkH, Body::Hash160Body(_)) => {
            return Err(failed(
                "Tag::RawPkH is not constructible through miniscript's public API".to_string(),
            ));
        }
        (Tag::False, Body::Empty) => Terminal::False,
        (Tag::True, Body::Empty) => Terminal::True,
        (Tag::TapTree, _) => {
            return Err(failed(
                "Tag::TapTree is a tap-tree internal node, not a miniscript leaf".to_string(),
            ));
        }
        (Tag::Tr, _) | (Tag::Wsh, _) | (Tag::Sh, _) | (Tag::Wpkh, _) | (Tag::Pkh, _) => {
            return Err(failed(format!(
                "top-level wrapper {:?} cannot appear inside a miniscript context",
                node.tag
            )));
        }
        _ => {
            return Err(failed(format!(
                "tag {:?} unsupported with body shape",
                node.tag
            )));
        }
    };
    Miniscript::from_ast(term).map_err(into_failed)
}

fn lookup_key(keys: &[DescriptorPublicKey], idx: u8) -> Result<DescriptorPublicKey, Error> {
    keys.get(idx as usize)
        .cloned()
        .ok_or_else(|| failed(format!("@{idx} out of range")))
}

fn build_multi_threshold<const MAX: usize>(
    k: u8,
    indices: &[u8],
    keys: &[DescriptorPublicKey],
    label: &str,
) -> Result<Threshold<DescriptorPublicKey, MAX>, Error> {
    let pks: Vec<DescriptorPublicKey> = indices
        .iter()
        .map(|i| lookup_key(keys, *i))
        .collect::<Result<_, _>>()?;
    Threshold::<DescriptorPublicKey, MAX>::new(k as usize, pks)
        .map_err(|e| failed(format!("{label} threshold: {e}")))
}

fn arity_eq(tag: Tag, got: usize, expected: usize) -> Result<(), Error> {
    if got != expected {
        return Err(failed(format!(
            "{tag:?} expected {expected} children, got {got}"
        )));
    }
    Ok(())
}

fn failed(detail: String) -> Error {
    Error::AddressDerivationFailed { detail }
}

fn into_failed(e: miniscript::Error) -> Error {
    failed(e.to_string())
}

// ─── Pk::Sha256 / Pk::Hash256 / Pk::Ripemd160 / Pk::Hash160 construction ─

fn sha256_from_bytes(h: &[u8; 32]) -> Result<bitcoin::hashes::sha256::Hash, Error> {
    use bitcoin::hashes::Hash;
    Ok(bitcoin::hashes::sha256::Hash::from_byte_array(*h))
}

fn hash256_from_bytes(h: &[u8; 32]) -> Result<miniscript::hash256::Hash, Error> {
    use bitcoin::hashes::Hash;
    Ok(miniscript::hash256::Hash::from_byte_array(*h))
}

fn ripemd160_from_bytes(h: &[u8; 20]) -> Result<bitcoin::hashes::ripemd160::Hash, Error> {
    use bitcoin::hashes::Hash;
    Ok(bitcoin::hashes::ripemd160::Hash::from_byte_array(*h))
}

fn hash160_from_bytes(h: &[u8; 20]) -> Result<bitcoin::hashes::hash160::Hash, Error> {
    use bitcoin::hashes::Hash;
    Ok(bitcoin::hashes::hash160::Hash::from_byte_array(*h))
}
