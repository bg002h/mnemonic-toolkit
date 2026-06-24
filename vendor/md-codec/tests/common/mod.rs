//! Shared generators + helpers for the md-codec test-hardening suite.
//! Consumed by proptest_roundtrip.rs and bch_adversarial.rs via `mod common;`.
#![allow(dead_code, unused_imports)]

use md_codec::canonicalize::canonicalize_placeholder_indices;
use md_codec::encode::Descriptor;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;
use proptest::prelude::*;

fn divergent_path(n: u8, depth: u8) -> PathDecl {
    let paths = (0..n)
        .map(|c| OriginPath {
            components: (0..depth)
                .map(|i| PathComponent {
                    hardened: true,
                    value: (c as u32) * 100 + (i as u32) + 1,
                })
                .collect(),
        })
        .collect();
    PathDecl {
        n,
        paths: PathDeclPaths::Divergent(paths),
    }
}

pub fn wrap(tag: Tag, inner: Node) -> Node {
    Node {
        tag,
        body: Body::Children(vec![inner]),
    }
}
pub fn keyarg(tag: Tag, index: u8) -> Node {
    Node {
        tag,
        body: Body::KeyArg { index },
    }
}
pub fn multikeys(tag: Tag, k: u8, indices: Vec<u8>) -> Node {
    Node {
        tag,
        body: Body::MultiKeys { k, indices },
    }
}
pub fn node2(tag: Tag, a: Node, b: Node) -> Node {
    Node {
        tag,
        body: Body::Children(vec![a, b]),
    }
}
pub fn node3(tag: Tag, a: Node, b: Node, c: Node) -> Node {
    Node {
        tag,
        body: Body::Children(vec![a, b, c]),
    }
}
pub fn thresh_node(k: u8, children: Vec<Node>) -> Node {
    Node {
        tag: Tag::Thresh,
        body: Body::Variable { k, children },
    }
}
pub fn timelock(tag: Tag, v: u32) -> Node {
    Node {
        tag,
        body: Body::Timelock(v),
    }
}
pub fn hash32(tag: Tag, h: [u8; 32]) -> Node {
    Node {
        tag,
        body: Body::Hash256Body(h),
    }
}
pub fn hash20(tag: Tag, h: [u8; 20]) -> Node {
    Node {
        tag,
        body: Body::Hash160Body(h),
    }
}
pub fn tr_node(is_nums: bool, key_index: u8, tree: Option<Node>) -> Node {
    Node {
        tag: Tag::Tr,
        body: Body::Tr {
            is_nums,
            key_index,
            tree: tree.map(Box::new),
        },
    }
}
pub fn taptree2(l: Node, r: Node) -> Node {
    Node {
        tag: Tag::TapTree,
        body: Body::Children(vec![l, r]),
    }
}

/// n biased to the kiw-width boundaries (exercises kiw 0..5).
fn n_strategy() -> impl Strategy<Value = u8> {
    prop_oneof![
        Just(1u8),
        Just(2),
        Just(3),
        Just(4),
        Just(5),
        Just(8),
        Just(9),
        Just(15),
        Just(16),
        Just(17),
        Just(31),
        Just(32),
        2u8..=32,
    ]
}

/// Bounded-recursion tr() taptree: internal TapTree{Children(2)}; leaves from the
/// permitted allow-list. Leaves reference indices in 1..=max (keypath is @0);
/// descriptor_from_tree renumbers to contiguous 0..n.
fn taptree_strategy(max_key_index: u8) -> impl Strategy<Value = Node> {
    let leaf = prop_oneof![
        (1u8..=max_key_index).prop_map(|i| keyarg(Tag::PkK, i)),
        (1u8..=max_key_index).prop_map(|i| keyarg(Tag::PkH, i)),
        (1u8..=max_key_index).prop_map(|i| multikeys(Tag::MultiA, 1, vec![i])),
        (1u32..=65535).prop_map(|t| Node {
            tag: Tag::Older,
            body: Body::Timelock(t)
        }),
    ];
    leaf.prop_recursive(3, 8, 2, |inner| {
        (inner.clone(), inner).prop_map(|(l, r)| Node {
            tag: Tag::TapTree,
            body: Body::Children(vec![l, r]),
        })
    })
}

/// Distinct placeholder indices referenced by a tree (KeyArg + MultiKeys +
/// non-NUMS Tr.key_index), so n can be derived.
fn referenced_indices(node: &Node, out: &mut std::collections::BTreeSet<u8>) {
    match &node.body {
        Body::KeyArg { index } => {
            out.insert(*index);
        }
        Body::MultiKeys { indices, .. } => {
            out.extend(indices.iter().copied());
        }
        Body::Tr {
            is_nums,
            key_index,
            tree,
        } => {
            if !is_nums {
                out.insert(*key_index);
            }
            if let Some(t) = tree {
                referenced_indices(t, out);
            }
        }
        Body::Children(cs) => {
            for c in cs {
                referenced_indices(c, out);
            }
        }
        Body::Variable { children, .. } => {
            for c in children {
                referenced_indices(c, out);
            }
        }
        _ => {}
    }
}

/// Rewrite every placeholder index through `perm` (old->new). NUMS Tr.key_index
/// is left untouched (no wire repr), matching referenced_indices.
fn renumber_tree(node: &mut Node, perm: &std::collections::BTreeMap<u8, u8>) {
    match &mut node.body {
        Body::KeyArg { index } => {
            *index = perm[&*index];
        }
        Body::MultiKeys { indices, .. } => {
            for i in indices.iter_mut() {
                *i = perm[&*i];
            }
        }
        Body::Tr {
            is_nums,
            key_index,
            tree,
        } => {
            if !*is_nums {
                *key_index = perm[&*key_index];
            }
            if let Some(t) = tree {
                renumber_tree(t, perm);
            }
        }
        Body::Children(cs) => {
            for c in cs.iter_mut() {
                renumber_tree(c, perm);
            }
        }
        Body::Variable { children, .. } => {
            for c in children.iter_mut() {
                renumber_tree(c, perm);
            }
        }
        _ => {}
    }
}

/// Collect referenced indices and RENUMBER the tree to contiguous 0..n.
/// This is descriptor_from_tree's renumber logic, extracted so every root
/// builder (existing strategy, W tier, T tier, P7/P8 cells) goes through
/// the same path — which is what makes `canon()`'s `.expect` safe.
pub fn renumbered(mut tree: Node) -> (Node, u8) {
    let mut set = std::collections::BTreeSet::new();
    referenced_indices(&tree, &mut set);
    let perm: std::collections::BTreeMap<u8, u8> = set
        .iter()
        .enumerate()
        .map(|(rank, &old)| (old, rank as u8))
        .collect();
    renumber_tree(&mut tree, &perm);
    (tree, set.len() as u8)
}

/// Build a Descriptor: collect referenced indices, RENUMBER the tree to contiguous
/// 0..n, then derive n + path-decl. Explicit-origin shapes get a Divergent path.
pub fn descriptor_from_tree(tree: Node, explicit_origin: bool) -> Descriptor {
    let (tree, n) = renumbered(tree);
    let path_decl = if explicit_origin {
        divergent_path(n, 3)
    } else {
        PathDecl {
            n,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![PathComponent {
                    hardened: true,
                    value: 84,
                }],
            }),
        }
    };
    Descriptor {
        n,
        path_decl,
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection::new_empty(),
    }
}

pub fn descriptor_strategy() -> BoxedStrategy<Descriptor> {
    let single_sig = prop_oneof![
        Just(keyarg(Tag::Wpkh, 0)),
        Just(keyarg(Tag::Pkh, 0)),
        Just(Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: None
            }
        }),
    ]
    .prop_map(|t| descriptor_from_tree(t, false));

    let sh_wpkh =
        Just(wrap(Tag::Sh, keyarg(Tag::Wpkh, 0))).prop_map(|t| descriptor_from_tree(t, false));

    let multisig = (
        n_strategy(),
        1u8..=32u8,
        prop::sample::select(vec![Tag::Multi, Tag::SortedMulti]),
    )
        .prop_filter("k<=n", |(n, k, _)| k <= n)
        .prop_map(|(n, k, mtag)| {
            let inner = multikeys(mtag, k, (0..n).collect());
            descriptor_from_tree(wrap(Tag::Wsh, inner), true)
        });

    let sh_wsh = (n_strategy(), 1u8..=32u8)
        .prop_filter("k<=n", |(n, k)| k <= n)
        .prop_map(|(n, k)| {
            let inner = wrap(Tag::Wsh, multikeys(Tag::SortedMulti, k, (0..n).collect()));
            descriptor_from_tree(wrap(Tag::Sh, inner), true)
        });

    let sh_sortedmulti = (n_strategy(), 1u8..=32u8)
        .prop_filter("k<=n", |(n, k)| k <= n)
        .prop_map(|(n, k)| {
            let inner = multikeys(Tag::SortedMulti, k, (0..n).collect());
            descriptor_from_tree(wrap(Tag::Sh, inner), true)
        });

    let tr_multi_a = (2u8..=16u8, 1u8..=16u8)
        .prop_filter("k<=n-1", |(n, k)| *k < *n)
        .prop_map(|(n, k)| {
            let leaf = multikeys(Tag::MultiA, k, (1..n).collect());
            let tree = Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    is_nums: false,
                    key_index: 0,
                    tree: Some(Box::new(leaf)),
                },
            };
            descriptor_from_tree(tree, true)
        });

    let tr_taptree = (2u8..=8u8).prop_flat_map(|max| {
        taptree_strategy(max).prop_map(move |tt| {
            let tree = Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    is_nums: false,
                    key_index: 0,
                    tree: Some(Box::new(tt)),
                },
            };
            descriptor_from_tree(tree, true)
        })
    });

    prop_oneof![
        single_sig,
        sh_wpkh,
        multisig,
        sh_wsh,
        sh_sortedmulti,
        tr_multi_a,
        tr_taptree
    ]
    .boxed()
}

/// canonicalize a descriptor (the fixpoint helper).
pub fn canon(d: &Descriptor) -> Descriptor {
    let mut c = d.clone();
    canonicalize_placeholder_indices(&mut c).expect("strategy descriptors are canonicalizable");
    c
}

// ─────────────────────────────────────────────────────────────────────────
// Cycle B (stress program) — shared key material + W/T strategies.
// Spec: design/BRAINSTORM_proptest_fragment_domain_expansion.md (R4 GREEN).
// ─────────────────────────────────────────────────────────────────────────

/// The BIP-84/86 published-test-vector mnemonic (address_derivation.rs pattern).
const ABANDON_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// 32 account-level xpubs (`m/86'/0'/{i}'`) derived ONCE from the abandon
/// mnemonic via `OnceLock`, each packed as the v0.13 `Pubkeys` TLV payload
/// `(chain_code || compressed_pubkey)`.
pub fn test_xpubs() -> &'static [[u8; 65]; 32] {
    static XPUBS: std::sync::OnceLock<[[u8; 65]; 32]> = std::sync::OnceLock::new();
    XPUBS.get_or_init(|| {
        use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
        use bitcoin::secp256k1::Secp256k1;
        use std::str::FromStr;
        let mn = bip39::Mnemonic::parse(ABANDON_MNEMONIC).expect("known-good mnemonic");
        let seed = mn.to_seed("");
        let secp = Secp256k1::new();
        let master =
            Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).expect("seed gives master");
        let mut out = [[0u8; 65]; 32];
        for (i, slot) in out.iter_mut().enumerate() {
            let path = DerivationPath::from_str(&format!("m/86'/0'/{i}'")).expect("valid path");
            let xpriv = master.derive_priv(&secp, &path).expect("derive priv");
            let xpub = Xpub::from_priv(&secp, &xpriv);
            slot[..32].copy_from_slice(xpub.chain_code.as_ref());
            slot[32..].copy_from_slice(&xpub.public_key.serialize());
        }
        out
    })
}

/// Build a wallet-policy-mode Descriptor: renumber via `renumbered` (the
/// descriptor_from_tree logic), divergent 3-deep origin paths, and attach
/// `tlv.pubkeys` for every `@i` from the OnceLock pool. Usable for any
/// n in 1..=32 (P7 oversize-multi cells go above the T-tier 16 cap).
pub fn descriptor_with_pubkeys(tree: Node) -> Descriptor {
    let (tree, n) = renumbered(tree);
    assert!(
        (1..=32).contains(&n),
        "descriptor must reference 1..=32 keys, got {n}"
    );
    let mut tlv = TlvSection::new_empty();
    tlv.pubkeys = Some((0..n).map(|i| (i, test_xpubs()[i as usize])).collect());
    Descriptor {
        n,
        path_decl: divergent_path(n, 3),
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv,
    }
}

/// Pre-order sequential key-index assignment: every key slot in the tree
/// (KeyArg, MultiKeys element, non-NUMS Tr internal key) gets the next
/// fresh index. Guarantees (a) all-distinct `@i` per descriptor — the
/// T-tier tap rule (b) [each `@i` at most once per tr descriptor incl.
/// internal key] — and (b) first occurrences ascending in pre-order.
pub fn assign_sequential_indices(node: &mut Node, next: &mut u8) {
    match &mut node.body {
        Body::KeyArg { index } => {
            *index = *next;
            *next += 1;
        }
        Body::MultiKeys { indices, .. } => {
            for i in indices.iter_mut() {
                *i = *next;
                *next += 1;
            }
        }
        Body::Tr {
            is_nums,
            key_index,
            tree,
        } => {
            if !*is_nums {
                *key_index = *next;
                *next += 1;
            }
            if let Some(t) = tree {
                assign_sequential_indices(t, next);
            }
        }
        Body::Children(cs) => {
            for c in cs.iter_mut() {
                assign_sequential_indices(c, next);
            }
        }
        Body::Variable { children, .. } => {
            for c in children.iter_mut() {
                assign_sequential_indices(c, next);
            }
        }
        _ => {}
    }
}

// ─── Tier 1 — wire-domain strategy (W): full domains, arbitrary nesting ──

const W_WRAPPERS: [Tag; 7] = [
    Tag::Check,
    Tag::Verify,
    Tag::Swap,
    Tag::Alt,
    Tag::DupIf,
    Tag::NonZero,
    Tag::ZeroNotEqual,
];
const W_ARITY2: [Tag; 6] = [Tag::AndV, Tag::AndB, Tag::OrB, Tag::OrC, Tag::OrD, Tag::OrI];

/// Full-u32 timelock domain, biased to the spec's boundary constants.
pub const W_BOUNDARY_TIMELOCKS: [u32; 11] = [
    0,
    1,
    0xFFFF,
    0x0001_0000,
    0x0040_FFFF,
    0x0041_0000,
    499_999_999,
    500_000_000,
    0x7FFF_FFFF,
    0x8000_0000,
    u32::MAX,
];

fn w_timelock_node() -> BoxedStrategy<Node> {
    let v = prop_oneof![
        3 => prop::sample::select(W_BOUNDARY_TIMELOCKS.to_vec()),
        1 => any::<u32>(),
    ];
    (prop::sample::select(vec![Tag::After, Tag::Older]), v)
        .prop_map(|(t, v)| timelock(t, v))
        .boxed()
}

fn w_keyless_leaf() -> BoxedStrategy<Node> {
    prop_oneof![
        4 => w_timelock_node(),
        1 => any::<[u8; 32]>().prop_map(|h| hash32(Tag::Sha256, h)),
        1 => any::<[u8; 32]>().prop_map(|h| hash32(Tag::Hash256, h)),
        1 => any::<[u8; 20]>().prop_map(|h| hash20(Tag::Ripemd160, h)),
        1 => any::<[u8; 20]>().prop_map(|h| hash20(Tag::Hash160, h)),
        1 => any::<[u8; 20]>().prop_map(|h| hash20(Tag::RawPkH, h)),
        1 => Just(Node { tag: Tag::True, body: Body::Empty }),
        1 => Just(Node { tag: Tag::False, body: Body::Empty }),
    ]
    .boxed()
}

/// Multi-family node over the full wire domain: k ≤ len, duplicate indices
/// permitted (the wire layer doesn't forbid them).
fn w_multikeys(tags: Vec<Tag>, max_idx: u8, max_len: usize) -> BoxedStrategy<Node> {
    (
        prop::sample::select(tags),
        prop::collection::vec(0..=max_idx, 1..=max_len),
    )
        .prop_flat_map(|(tag, idxs)| {
            let len = idxs.len() as u8;
            (1..=len).prop_map(move |k| multikeys(tag, k, idxs.clone()))
        })
        .boxed()
}

fn w_keyed_leaf(max_idx: u8, max_len: usize) -> BoxedStrategy<Node> {
    prop_oneof![
        2 => (0..=max_idx).prop_map(|i| keyarg(Tag::PkK, i)),
        2 => (0..=max_idx).prop_map(|i| keyarg(Tag::PkH, i)),
        3 => w_multikeys(
            vec![Tag::Multi, Tag::SortedMulti, Tag::MultiA, Tag::SortedMultiA],
            max_idx,
            max_len
        ),
    ]
    .boxed()
}

/// One combinator layer over `child`: unary wrappers × arity-2 × AndOr(3)
/// × Thresh(k ≤ children ≤ 4). Worst-case node count = 1 + 4·|child|.
fn w_level(child: BoxedStrategy<Node>) -> BoxedStrategy<Node> {
    prop_oneof![
        2 => (prop::sample::select(W_WRAPPERS.to_vec()), child.clone())
            .prop_map(|(t, c)| wrap(t, c)),
        3 => (prop::sample::select(W_ARITY2.to_vec()), child.clone(), child.clone())
            .prop_map(|(t, a, b)| node2(t, a, b)),
        1 => (child.clone(), child.clone(), child.clone())
            .prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        2 => prop::collection::vec(child, 2..=4).prop_flat_map(|cs| {
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        }),
    ]
    .boxed()
}

/// wsh/sh/sh(wsh) inner subtree with the ≥1-key guarantee carried by
/// construction (a designated key-bearing leaf in every arm). Layered
/// depth ≤ 4, fan-out ≤ 4, ≤ 25 nodes incl. root wrappers (w2 worst case
/// 21; pairing 23; sh(wsh(·)) adds 2). Size enforcement is the encoded
/// ≤ 18,000-bit assert in the final map, not this node arithmetic.
fn w_inner(max_idx: u8, max_len: usize) -> BoxedStrategy<Node> {
    let leaf = prop_oneof![w_keyed_leaf(max_idx, max_len), w_keyless_leaf()].boxed();
    let w1 = w_level(leaf.clone());
    let w01 = prop_oneof![leaf.clone(), w1.clone()].boxed();
    let w2 = w_level(w01);
    let sub = prop_oneof![1 => leaf, 2 => w1.clone(), 2 => w2].boxed();
    let key = w_keyed_leaf(max_idx, max_len);
    prop_oneof![
        1 => key.clone(),
        1 => (prop::sample::select(W_WRAPPERS.to_vec()), key.clone())
            .prop_map(|(t, k)| wrap(t, k)),
        4 => (
            prop::sample::select(W_ARITY2.to_vec()),
            sub,
            key.clone(),
            any::<bool>()
        )
            .prop_map(|(t, s, k, flip)| if flip { node2(t, k, s) } else { node2(t, s, k) }),
        1 => (w1.clone(), key.clone(), w1.clone())
            .prop_map(|(a, k, b)| node3(Tag::AndOr, a, k, b)),
        1 => (key, prop::collection::vec(w1, 1..=3)).prop_flat_map(|(k0, rest)| {
            let mut cs = vec![k0];
            cs.extend(rest);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        }),
    ]
    .boxed()
}

/// tr() root for the W tier. Taptree LEAF top-tags avoid the §6.3.1
/// forbidden set (bare Multi/SortedMulti never appear as a leaf root;
/// combinator leaves may still carry them INSIDE — decode permits that).
/// Key guarantee: non-NUMS internal key, or a designated key-bearing leaf.
fn w_tr(max_idx: u8, max_len: usize) -> BoxedStrategy<Node> {
    let leaf_full = prop_oneof![w_keyed_leaf(max_idx, max_len), w_keyless_leaf()].boxed();
    let tap_keyed = prop_oneof![
        1 => (0..=max_idx).prop_map(|i| keyarg(Tag::PkK, i)),
        1 => (0..=max_idx).prop_map(|i| keyarg(Tag::PkH, i)),
        2 => w_multikeys(vec![Tag::MultiA, Tag::SortedMultiA], max_idx, max_len),
    ]
    .boxed();
    let tap_atom = prop_oneof![2 => tap_keyed.clone(), 1 => w_keyless_leaf()].boxed();
    let w1 = w_level(leaf_full.clone());
    let w2 = w_level(prop_oneof![leaf_full, w1.clone()].boxed());
    let tap_leaf1 = prop_oneof![2 => tap_atom.clone(), 2 => w1].boxed();
    let single_leaf = prop_oneof![2 => tap_atom, 2 => tap_leaf1.clone(), 1 => w2].boxed();
    prop_oneof![
        3 => (0..=max_idx, single_leaf).prop_map(|(i, l)| tr_node(false, i, Some(l))),
        1 => (0..=max_idx).prop_map(|i| tr_node(false, i, None)),
        2 => tap_keyed.clone().prop_map(|l| tr_node(true, 0, Some(l))),
        2 => (tap_keyed, tap_leaf1.clone(), any::<bool>()).prop_map(|(k, o, flip)| {
            let tt = if flip { taptree2(k, o) } else { taptree2(o, k) };
            tr_node(true, 0, Some(tt))
        }),
        1 => (0..=max_idx, tap_leaf1.clone(), tap_leaf1.clone(), tap_leaf1)
            .prop_map(|(i, a, b, c)| tr_node(false, i, Some(taptree2(taptree2(a, b), c)))),
    ]
    .boxed()
}

/// Which TLVs the W tier attaches this case. Pubkeys/fingerprints cap the
/// key universe at n ≤ 8 (payload budget belt-and-braces, spec [I1′]).
#[derive(Clone, Copy, Debug)]
pub struct WTlvMode {
    pubkeys: bool,
    fingerprints: bool,
    origin_overrides: bool,
}

fn w_tlv_mode() -> impl Strategy<Value = WTlvMode> {
    (
        prop::bool::weighted(0.35),
        prop::bool::weighted(0.35),
        prop::bool::weighted(0.35),
    )
        .prop_map(|(pubkeys, fingerprints, origin_overrides)| WTlvMode {
            pubkeys,
            fingerprints,
            origin_overrides,
        })
}

fn w_origin_override_path() -> impl Strategy<Value = OriginPath> {
    prop::collection::vec((any::<bool>(), 0u32..=10_000), 1..=3).prop_map(|cs| OriginPath {
        components: cs
            .into_iter()
            .map(|(hardened, value)| PathComponent { hardened, value })
            .collect(),
    })
}

/// Sparse per-`@N` TLV entry vector (the strategy-output shape).
type SparseTlv<T> = Option<Vec<(u8, T)>>;

/// TLV entries for a renumbered tree with `n` keys. Pubkeys attach the
/// full 0..n map (valid curve points from the T-tier pool); fingerprints
/// and origin overrides attach non-empty ascending subsets.
fn w_tlv_entries(mode: WTlvMode, n: u8) -> BoxedStrategy<TlvSection> {
    let idxs: Vec<u8> = (0..n).collect();
    let pubkeys_s: BoxedStrategy<SparseTlv<[u8; 65]>> = if mode.pubkeys {
        Just(Some(
            (0..n)
                .map(|i| (i, test_xpubs()[i as usize]))
                .collect::<Vec<_>>(),
        ))
        .boxed()
    } else {
        Just(None).boxed()
    };
    let fps_s: BoxedStrategy<SparseTlv<[u8; 4]>> = if mode.fingerprints {
        prop::sample::subsequence(idxs.clone(), 1..=n as usize)
            .prop_flat_map(|sel| {
                prop::collection::vec(any::<[u8; 4]>(), sel.len())
                    .prop_map(move |bytes| Some(sel.iter().copied().zip(bytes).collect::<Vec<_>>()))
            })
            .boxed()
    } else {
        Just(None).boxed()
    };
    let origin_s: BoxedStrategy<SparseTlv<OriginPath>> = if mode.origin_overrides {
        prop::sample::subsequence(idxs, 1..=n as usize)
            .prop_flat_map(|sel| {
                prop::collection::vec(w_origin_override_path(), sel.len())
                    .prop_map(move |paths| Some(sel.iter().copied().zip(paths).collect::<Vec<_>>()))
            })
            .boxed()
    } else {
        Just(None).boxed()
    };
    (pubkeys_s, fps_s, origin_s)
        .prop_map(|(pubkeys, fingerprints, origin_path_overrides)| {
            let mut t = TlvSection::new_empty();
            t.pubkeys = pubkeys;
            t.fingerprints = fingerprints;
            t.origin_path_overrides = origin_path_overrides;
            t
        })
        .boxed()
}

/// Tier 1 (W): decode-valid but not necessarily type-valid descriptors over
/// the FULL wire domains, with randomized Shared/Divergent path-decls and
/// occasional origin/fingerprint/pubkey TLVs. The final map enforces the
/// payload budget on the ACTUAL ENCODING (≤ 18,000 bits, margin under the
/// 20,480-bit chunk cliff) — a loud panic here is the intended drift signal.
pub fn wire_descriptor_strategy() -> BoxedStrategy<Descriptor> {
    w_tlv_mode()
        .prop_flat_map(|mode| {
            let (max_idx, max_len): (u8, usize) = if mode.pubkeys || mode.fingerprints {
                (7, 8)
            } else {
                (31, 16)
            };
            let inner = w_inner(max_idx, max_len);
            let tree = prop_oneof![
                3 => inner.clone().prop_map(|i| wrap(Tag::Wsh, i)),
                2 => inner.clone().prop_map(|i| wrap(Tag::Sh, i)),
                2 => inner.prop_map(|i| wrap(Tag::Sh, wrap(Tag::Wsh, i))),
                3 => w_tr(max_idx, max_len),
            ];
            (tree, any::<bool>()).prop_flat_map(move |(tree, divergent)| {
                let (tree, n) = renumbered(tree);
                w_tlv_entries(mode, n).prop_map(move |tlv| Descriptor {
                    n,
                    path_decl: if divergent {
                        divergent_path(n, 3)
                    } else {
                        PathDecl {
                            n,
                            paths: PathDeclPaths::Shared(OriginPath {
                                components: vec![PathComponent {
                                    hardened: true,
                                    value: 84,
                                }],
                            }),
                        }
                    },
                    use_site_path: UseSitePath::standard_multipath(),
                    tree: tree.clone(),
                    tlv,
                })
            })
        })
        .prop_map(|d| {
            let c = canon(&d);
            let (_bytes, total_bits) =
                md_codec::encode::encode_payload(&c).expect("W-tier descriptor must encode");
            assert!(
                total_bits <= 18_000,
                "W-tier payload budget exceeded: {total_bits} bits > 18,000"
            );
            d
        })
        .boxed()
}

// ─── Tier 2 — typed strategy (T): type-correct-by-construction + xpub TLVs ──

/// Consensus-valid `after` values for the chosen absolute-lock class.
/// Boundary-biased; valid domain is 1..=0x7FFF_FFFF.
fn t_after_value(abs_time: bool) -> BoxedStrategy<u32> {
    if abs_time {
        prop_oneof![
            3 => prop::sample::select(vec![500_000_000u32, 0x7FFF_FFFF]),
            2 => 500_000_000u32..=0x7FFF_FFFF,
        ]
        .boxed()
    } else {
        prop_oneof![
            3 => prop::sample::select(vec![1u32, 144, 0xFFFF, 0x0001_0000, 499_999_999]),
            2 => 1u32..=499_999_999,
        ]
        .boxed()
    }
}

/// Consensus-valid `older` values for the chosen relative-lock class
/// (non-zero, bit 31 clear; class = bit 22). INCLUDES the out-of-BIP-68-mask
/// values 0x10000 (height class) / 0x00410000 (time class) — miniscript's
/// known leniency, pinned Ok in P6 (toolkit v0.53.9 context).
fn t_older_value(rel_time: bool) -> BoxedStrategy<u32> {
    if rel_time {
        prop_oneof![
            3 => prop::sample::select(vec![0x0040_0001u32, 0x0040_FFFF, 0x0041_0000]),
            2 => (1u32..=0xFFFF).prop_map(|v| 0x0040_0000 | v),
        ]
        .boxed()
    } else {
        prop_oneof![
            3 => prop::sample::select(vec![1u32, 144, 0xFFFF, 0x0001_0000]),
            2 => 1u32..=0xFFFF,
        ]
        .boxed()
    }
}

fn t_lock_node(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    prop_oneof![
        t_older_value(rel_time).prop_map(|v| timelock(Tag::Older, v)),
        t_after_value(abs_time).prop_map(|v| timelock(Tag::After, v)),
    ]
    .boxed()
}

fn t_hash_node() -> BoxedStrategy<Node> {
    prop_oneof![
        any::<[u8; 32]>().prop_map(|h| hash32(Tag::Sha256, h)),
        any::<[u8; 32]>().prop_map(|h| hash32(Tag::Hash256, h)),
        any::<[u8; 20]>().prop_map(|h| hash20(Tag::Ripemd160, h)),
        any::<[u8; 20]>().prop_map(|h| hash20(Tag::Hash160, h)),
    ]
    .boxed()
}

/// Bare key leaf (`pk(@i)` / `pk_h(@i)`). Index 0 is a placeholder —
/// `assign_sequential_indices` allocates the real fresh index.
fn t_ka() -> BoxedStrategy<Node> {
    prop_oneof![Just(keyarg(Tag::PkK, 0)), Just(keyarg(Tag::PkH, 0)),].boxed()
}

/// Multi-family leaf with `min_n..=max_n` PLACEHOLDER key slots, k ≤ n.
fn t_multi_node(tag: Tag, min_n: u8, max_n: u8) -> BoxedStrategy<Node> {
    (min_n..=max_n)
        .prop_flat_map(move |n| (1..=n).prop_map(move |k| multikeys(tag, k, vec![0; n as usize])))
        .boxed()
}

/// Segwitv0 (`wsh`) typed B-tree: spec grammar with the segwit Bdu pool
/// (keys, multi, hashlocks, recursive or_d) and W = s:pk | a:<Bdu leaf>.
/// Every arm carries ≥1 key by construction. Worst-case key slots: 15.
fn t_segwit_tree(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    let ka = t_ka();
    let hash = t_hash_node();
    let lock = t_lock_node(rel_time, abs_time);
    let multi3 = t_multi_node(Tag::Multi, 1, 3);
    let leaf_key = prop_oneof![2 => ka.clone(), 1 => multi3.clone()].boxed();
    let leaf_any = prop_oneof![
        2 => leaf_key.clone(),
        1 => hash.clone(),
        1 => lock.clone(),
    ]
    .boxed();
    let bdu0 = prop_oneof![2 => ka.clone(), 1 => multi3.clone(), 1 => hash].boxed();
    let bdu_key = prop_oneof![2 => ka.clone(), 1 => multi3].boxed();
    let bdu1 = prop_oneof![
        2 => bdu0.clone(),
        1 => (bdu0.clone(), bdu0.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
    ]
    .boxed();
    let w0 = prop_oneof![
        1 => Just(wrap(Tag::Swap, keyarg(Tag::PkK, 0))),
        1 => bdu0.clone().prop_map(|b| wrap(Tag::Alt, b)),
    ]
    .boxed();
    let vfirst = prop_oneof![1 => lock, 1 => leaf_any.clone()].boxed();
    let thresh1 =
        (bdu_key.clone(), prop::collection::vec(w0.clone(), 1..=2)).prop_flat_map(|(first, ws)| {
            let mut cs = vec![first];
            cs.extend(ws);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        });
    let b1 = prop_oneof![
        3 => leaf_key.clone(),
        2 => (vfirst.clone(), leaf_key.clone())
            .prop_map(|(x, y)| node2(Tag::AndV, wrap(Tag::Verify, x), y)),
        2 => (leaf_key.clone(), w0.clone()).prop_map(|(b, w)| node2(Tag::AndB, b, w)),
        2 => (leaf_key.clone(), leaf_any.clone()).prop_map(|(a, b)| node2(Tag::OrI, a, b)),
        2 => (bdu1.clone(), leaf_key.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
        2 => (bdu0.clone(), leaf_any.clone(), leaf_key.clone())
            .prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        2 => thresh1,
    ]
    .boxed();
    let thresh2 =
        (bdu_key, prop::collection::vec(w0.clone(), 1..=2)).prop_flat_map(|(first, ws)| {
            let mut cs = vec![first];
            cs.extend(ws);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        });
    let b2 = prop_oneof![
        4 => b1.clone(),
        1 => (b1.clone(), leaf_any.clone())
            .prop_map(|(b, x)| node2(Tag::AndV, wrap(Tag::Verify, b), x)),
        1 => (b1.clone(), w0).prop_map(|(b, w)| node2(Tag::AndB, b, w)),
        1 => (b1.clone(), leaf_any).prop_map(|(a, b)| node2(Tag::OrI, a, b)),
        1 => (bdu1, b1.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
        1 => (bdu0, b1, leaf_key).prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        1 => thresh2,
    ]
    .boxed();
    // GAP-2: the seven fragment arms T previously omitted (DupIf/NonZero/
    // ZeroNotEqual/OrB/OrC/True/False), as FIXED proven shapes. R0-I2
    // CONSTRAINT (load-bearing): children are ONLY keys + locks in these exact
    // positions — do NOT route the leaf_any/bdu*/hash pools into or_b/or_c/j:/n:
    // child slots; those compose type-invalid trees (e.g. or_b(older,s:pk),
    // j:older) that P6 step-1 panics on (no prop_filter). All seven are B-type.
    let seven = prop_oneof![
        // or_b(pk, s:pk)
        Just(node2(
            Tag::OrB,
            keyarg(Tag::PkK, 0),
            wrap(Tag::Swap, keyarg(Tag::PkK, 0)),
        )),
        // t:or_c(pk, v:pk)  ==  and_v(or_c(pk, v:pk), True)
        Just(node2(
            Tag::AndV,
            node2(
                Tag::OrC,
                keyarg(Tag::PkK, 0),
                wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            ),
            Node {
                tag: Tag::True,
                body: Body::Empty,
            },
        )),
        // or_i(pk, d:v:LOCK)
        t_lock_node(rel_time, abs_time).prop_map(|l| node2(
            Tag::OrI,
            keyarg(Tag::PkK, 0),
            wrap(Tag::DupIf, wrap(Tag::Verify, l)),
        )),
        // j:pk
        Just(wrap(Tag::NonZero, keyarg(Tag::PkK, 0))),
        // or_i(pk, n:and_v(v:pk, LOCK))
        t_lock_node(rel_time, abs_time).prop_map(|l| node2(
            Tag::OrI,
            keyarg(Tag::PkK, 0),
            wrap(
                Tag::ZeroNotEqual,
                node2(Tag::AndV, wrap(Tag::Verify, keyarg(Tag::PkK, 0)), l),
            ),
        )),
        // u:pk  ==  or_i(pk, False)
        Just(node2(
            Tag::OrI,
            keyarg(Tag::PkK, 0),
            Node {
                tag: Tag::False,
                body: Body::Empty,
            },
        )),
        // tv:pk ==  and_v(v:pk, True)
        Just(node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            Node {
                tag: Tag::True,
                body: Body::Empty,
            },
        )),
    ]
    .boxed();
    // Standalone wide multi exercises Segwitv0 multi up to the T-tier
    // n ≤ 16 cap (the miniscript limit is 20). The 16 is a deliberate T-tier
    // key-BUDGET choice, NOT an infra limit — test_xpubs() has 32 keys and
    // descriptor_with_pubkeys accepts 1..=32. The valid 17..=20 render/address
    // window is pinned deterministically by the self_test_wsh_multi_17_of_*
    // goldens; n ≥ 21 is P7 oversize-refusal territory.
    let wide_multi = t_multi_node(Tag::Multi, 2, 16);
    prop_oneof![5 => b2, 2 => seven, 1 => wide_multi].boxed()
}

/// Legacy (`sh`) typed tree: depth ≤ 2, ≤ 6 key slots, multi ≤ 6 keys
/// standalone / ≤ 2 in compound positions (pk_cost 520 headroom).
fn t_legacy_tree(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    let ka = t_ka();
    let hash = t_hash_node();
    let lock = t_lock_node(rel_time, abs_time);
    let multi2 = t_multi_node(Tag::Multi, 1, 2);
    let multi6 = t_multi_node(Tag::Multi, 1, 6);
    let bdu0 = prop_oneof![2 => ka.clone(), 1 => multi2.clone(), 1 => hash.clone()].boxed();
    let bdu_key = prop_oneof![2 => ka.clone(), 1 => multi2].boxed();
    let w0 = prop_oneof![
        1 => Just(wrap(Tag::Swap, keyarg(Tag::PkK, 0))),
        1 => bdu0.clone().prop_map(|b| wrap(Tag::Alt, b)),
    ]
    .boxed();
    let vfirst = prop_oneof![1 => lock, 1 => hash, 1 => ka.clone()].boxed();
    let thresh =
        (bdu_key, prop::collection::vec(w0.clone(), 1..=2)).prop_flat_map(|(first, ws)| {
            let mut cs = vec![first];
            cs.extend(ws);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        });
    prop_oneof![
        2 => ka.clone(),
        2 => multi6,
        2 => (vfirst, ka.clone()).prop_map(|(x, y)| node2(Tag::AndV, wrap(Tag::Verify, x), y)),
        2 => (ka.clone(), w0).prop_map(|(b, w)| node2(Tag::AndB, b, w)),
        1 => (ka.clone(), ka.clone()).prop_map(|(a, b)| node2(Tag::OrI, a, b)),
        2 => (bdu0.clone(), ka.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
        1 => (bdu0, ka.clone(), ka).prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        2 => thresh,
    ]
    .boxed()
}

/// Tap leaf grammar — SANE-BY-CONSTRUCTION (spec rule (a)): every leaf is
/// signature-bearing AND non-malleable. The tap Bdu pool is KEYS ONLY
/// (pk/pk_h/multi_a + recursive or_d); hashlocks/timelocks appear ONLY
/// under `v:` inside `and_v(v:<lock|hash>, <sig-bearing B>)`. Every
/// production here has an empirical sanity proof in the round-1/2/3
/// evidence logs or the golden self-test cells.
fn t_tap_leaf(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    let ka = t_ka();
    let hash = t_hash_node();
    let lock = t_lock_node(rel_time, abs_time);
    let multi_a2 = t_multi_node(Tag::MultiA, 2, 2);
    let multi_a3 = t_multi_node(Tag::MultiA, 1, 3);
    let bdu0 = prop_oneof![2 => ka.clone(), 1 => multi_a2].boxed();
    let bdu1 = prop_oneof![
        2 => bdu0.clone(),
        1 => (bdu0.clone(), bdu0.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
    ]
    .boxed();
    let w0 = prop_oneof![
        1 => Just(wrap(Tag::Swap, keyarg(Tag::PkK, 0))),
        1 => bdu0.clone().prop_map(|b| wrap(Tag::Alt, b)),
    ]
    .boxed();
    let vfirst = prop_oneof![1 => lock, 1 => hash, 1 => ka.clone()].boxed();
    let sb_simple = prop_oneof![
        2 => ka.clone(),
        1 => (vfirst.clone(), ka.clone())
            .prop_map(|(x, y)| node2(Tag::AndV, wrap(Tag::Verify, x), y)),
    ]
    .boxed();
    let thresh =
        (bdu0.clone(), prop::collection::vec(w0.clone(), 1..=2)).prop_flat_map(|(first, ws)| {
            let mut cs = vec![first];
            cs.extend(ws);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        });
    prop_oneof![
        2 => ka,
        1 => multi_a3,
        2 => (vfirst, sb_simple.clone())
            .prop_map(|(x, y)| node2(Tag::AndV, wrap(Tag::Verify, x), y)),
        2 => (sb_simple.clone(), w0).prop_map(|(b, w)| node2(Tag::AndB, b, w)),
        2 => (sb_simple.clone(), sb_simple.clone()).prop_map(|(a, b)| node2(Tag::OrI, a, b)),
        2 => (bdu1, sb_simple.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
        2 => (bdu0, sb_simple.clone(), sb_simple).prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        2 => thresh,
    ]
    .boxed()
}

/// tr() roots for the T tier: NUMS-or-key internal, 1–2 sane leaves,
/// multi_a ≤ 16 keys (n-budget). Key slots ≤ 16 total by construction.
///
/// MINIMAL GENERATOR CONSTRAINT (found by P6 during bring-up): taptree
/// DEPTH ≤ 1 — i.e. at most 2 leaves, `{a,b}`. Pinned miniscript 13.0.0
/// has a Display/parse asymmetry on DEPTH-2 taptrees: pure upstream
/// `TapTree::combine(combine(a,b),c)` Displays as the malformed
/// `{{a,b,c}}` (instead of `{{a,b},c}`), which miniscript's OWN
/// `Descriptor::from_str` rejects (IncorrectNumberOfChildren) — and a
/// correctly-written depth-2 string parses Ok but re-Displays broken.
/// NOT an md-codec bug (md-codec wire round-trip and address derivation
/// are unaffected). Pinned loudly by
/// `upstream_taptree_depth2_display_asymmetry` in
/// tests/proptest_to_miniscript.rs; a miniscript bump past the upstream
/// fix flips that cell and this arm can be restored.
fn t_tr_tree(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    let leaf = t_tap_leaf(rel_time, abs_time);
    prop_oneof![
        3 => leaf.clone().prop_map(|l| tr_node(false, 0, Some(l))),
        2 => leaf.clone().prop_map(|l| tr_node(true, 0, Some(l))),
        2 => (any::<bool>(), leaf.clone(), leaf).prop_map(|(nums, a, b)| {
            tr_node(nums, 0, Some(taptree2(a, b)))
        }),
        1 => t_multi_node(Tag::MultiA, 1, 16).prop_map(|m| tr_node(true, 0, Some(m))),
        1 => t_multi_node(Tag::MultiA, 1, 15).prop_map(|m| tr_node(false, 0, Some(m))),
    ]
    .boxed()
}

/// Tier 2 (T): type-correct-by-construction miniscript descriptors with
/// real xpub TLVs attached for every `@i`. NO prop_filter anywhere —
/// every emitted descriptor MUST pass P6's full chain. Timelock classes
/// (rule (c)) are chosen once per descriptor: relative height-XOR-time and
/// absolute height-XOR-time (rel + abs together is fine) — DELIBERATELY
/// stricter than miniscript's per-spend-path rule (see spec [M2′]).
pub fn typed_descriptor_strategy() -> BoxedStrategy<Descriptor> {
    (any::<bool>(), any::<bool>())
        .prop_flat_map(|(rel_time, abs_time)| {
            prop_oneof![
                3 => t_segwit_tree(rel_time, abs_time).prop_map(|t| wrap(Tag::Wsh, t)),
                2 => t_legacy_tree(rel_time, abs_time).prop_map(|t| wrap(Tag::Sh, t)),
                3 => t_tr_tree(rel_time, abs_time),
            ]
        })
        .prop_map(|mut tree| {
            let mut next = 0u8;
            assign_sequential_indices(&mut tree, &mut next);
            assert!(
                (1..=16).contains(&next),
                "T-tier key budget violated: {next} key slots (cap 16)"
            );
            descriptor_with_pubkeys(tree)
        })
        .boxed()
}

// ─── Anti-vacuity walkers (generator_covers_all_fragments) ──────────────

/// Collect every Tag in the tree plus all (After/Older) timelock values.
pub fn collect_tags_and_locks(
    node: &Node,
    tags: &mut std::collections::HashSet<Tag>,
    locks: &mut std::collections::HashSet<u32>,
) {
    tags.insert(node.tag);
    match &node.body {
        Body::Timelock(v) => {
            locks.insert(*v);
        }
        Body::Children(cs) => {
            for c in cs {
                collect_tags_and_locks(c, tags, locks);
            }
        }
        Body::Variable { children, .. } => {
            for c in children {
                collect_tags_and_locks(c, tags, locks);
            }
        }
        Body::Tr { tree: Some(t), .. } => {
            collect_tags_and_locks(t, tags, locks);
        }
        _ => {}
    }
}

/// flip one codex32 symbol at data-part position `pos` (post-"md1") of a chunk.
pub fn corrupt_chunk_at(chunk: &str, pos: usize, xor_mask: u8) -> String {
    const A: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let mut chars: Vec<char> = chunk.chars().collect();
    let idx = 3 + pos;
    assert!(
        idx < chars.len(),
        "corrupt position {pos} past data-part (chunk len {})",
        chars.len()
    );
    let sym = A
        .iter()
        .position(|&b| b == (chars[idx] as u8).to_ascii_lowercase())
        .unwrap() as u8;
    chars[idx] = A[((sym ^ (xor_mask & 0x1F)) & 0x1F) as usize] as char;
    chars.into_iter().collect()
}
