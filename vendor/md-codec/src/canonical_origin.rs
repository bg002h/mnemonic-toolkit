//! Canonical-origin map per spec §4 (v0.13 wallet-policy layer).
//!
//! Given the top-level wrapper of a descriptor template, return the canonical
//! `path-from-master` for elided origin paths — or `None` if the wrapper shape
//! is not in the canonical table (in which case the encoder must emit
//! explicit `OriginPathOverrides` entries for all `@N` placeholders).
//!
//! Wrapper shape → canonical:
//!
//! | Shape                                  | Canonical             |
//! |----------------------------------------|-----------------------|
//! | `pkh(@N)` single-key                   | `m/44'/0'/0'`         |
//! | `wpkh(@N)` single-key                  | `m/84'/0'/0'`         |
//! | `tr(@N)` key-path only (no TapTree)    | `m/86'/0'/0'`         |
//! | `wsh(multi/sortedmulti)`               | `m/48'/0'/0'/2'`      |
//! | `sh(wsh(multi/sortedmulti))`           | `m/48'/0'/0'/1'`      |
//! | `sh(sortedmulti)` legacy P2SH multi    | `None` (forced explicit) |
//! | `tr(@N, TapTree)`                      | `None` (forced explicit) |
//! | anything else                          | `None` (forced explicit) |

use crate::origin_path::{OriginPath, PathComponent};
use crate::tag::Tag;
use crate::tree::{Body, Node};

/// Build an [`OriginPath`] from a slice of `(hardened, value)` tuples.
fn mk_origin(components: &[(bool, u32)]) -> OriginPath {
    OriginPath {
        components: components
            .iter()
            .map(|&(hardened, value)| PathComponent { hardened, value })
            .collect(),
    }
}

/// Returns `true` if `tag` is one of the multisig variants permitted directly
/// inside a canonical `wsh(...)` or `sh(wsh(...))` wrapper (`multi` or
/// `sortedmulti`).
pub(crate) fn is_wsh_inner_multi(tag: Tag) -> bool {
    matches!(tag, Tag::Multi | Tag::SortedMulti)
}

/// Compute the canonical origin path for the top-level wrapper `tree`, per
/// spec §4. Returns `None` for shapes that require explicit
/// `OriginPathOverrides` on the wire.
pub fn canonical_origin(tree: &Node) -> Option<OriginPath> {
    match (&tree.tag, &tree.body) {
        // pkh(@N) single-key → m/44'/0'/0'
        (Tag::Pkh, Body::KeyArg { .. }) => Some(mk_origin(&[(true, 44), (true, 0), (true, 0)])),
        // wpkh(@N) single-key → m/84'/0'/0'
        (Tag::Wpkh, Body::KeyArg { .. }) => Some(mk_origin(&[(true, 84), (true, 0), (true, 0)])),
        // tr(@N) key-path only (no TapTree) → m/86'/0'/0'
        (Tag::Tr, Body::Tr { tree: None, .. }) => {
            Some(mk_origin(&[(true, 86), (true, 0), (true, 0)]))
        }
        // tr(@N, TapTree) → None (forced explicit)
        (Tag::Tr, Body::Tr { tree: Some(_), .. }) => None,
        // wsh(multi/sortedmulti) → m/48'/0'/0'/2'
        (Tag::Wsh, Body::Children(children))
            if children.len() == 1 && is_wsh_inner_multi(children[0].tag) =>
        {
            Some(mk_origin(&[(true, 48), (true, 0), (true, 0), (true, 2)]))
        }
        // sh(wsh(multi/sortedmulti)) → m/48'/0'/0'/1'
        // sh(sortedmulti) legacy → None (handled by the catch-all below)
        (Tag::Sh, Body::Children(children)) if children.len() == 1 => {
            let inner = &children[0];
            if inner.tag == Tag::Wsh {
                if let Body::Children(grand) = &inner.body {
                    if grand.len() == 1 && is_wsh_inner_multi(grand[0].tag) {
                        return Some(mk_origin(&[(true, 48), (true, 0), (true, 0), (true, 1)]));
                    }
                }
            }
            None
        }
        // Everything else: bare wsh(@N), bare sh(@N), miniscript bodies, etc.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Body, Node};

    fn pkh_at(n: u8) -> Node {
        Node {
            tag: Tag::Pkh,
            body: Body::KeyArg { index: n },
        }
    }

    fn wpkh_at(n: u8) -> Node {
        Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: n },
        }
    }

    fn tr_keypath(n: u8) -> Node {
        Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: n,
                tree: None,
            },
        }
    }

    fn tr_with_taptree(n: u8) -> Node {
        // Minimal non-empty TapTree: a single pk_k leaf wrapped in a TapTree
        // node. The exact inner shape is not relevant — only that
        // `tree: Some(_)` so the classifier sees a script-tree variant.
        Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: n,
                tree: Some(Box::new(Node {
                    tag: Tag::PkK,
                    body: Body::KeyArg { index: 1 },
                })),
            },
        }
    }

    fn multi_2of3() -> Node {
        Node {
            tag: Tag::Multi,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        }
    }

    fn sortedmulti_2of3() -> Node {
        Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        }
    }

    fn wsh_of(inner: Node) -> Node {
        Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![inner]),
        }
    }

    fn sh_of(inner: Node) -> Node {
        Node {
            tag: Tag::Sh,
            body: Body::Children(vec![inner]),
        }
    }

    #[test]
    fn pkh_at_n_returns_bip44_origin() {
        let got = canonical_origin(&pkh_at(0)).unwrap();
        assert_eq!(got, mk_origin(&[(true, 44), (true, 0), (true, 0)]));
    }

    #[test]
    fn wpkh_at_n_returns_bip84_origin() {
        let got = canonical_origin(&wpkh_at(0)).unwrap();
        assert_eq!(got, mk_origin(&[(true, 84), (true, 0), (true, 0)]));
    }

    #[test]
    fn tr_keypath_only_returns_bip86_origin() {
        let got = canonical_origin(&tr_keypath(0)).unwrap();
        assert_eq!(got, mk_origin(&[(true, 86), (true, 0), (true, 0)]));
    }

    #[test]
    fn tr_with_taptree_returns_none() {
        assert_eq!(canonical_origin(&tr_with_taptree(0)), None);
    }

    #[test]
    fn wsh_multi_returns_bip48_type_2() {
        let got = canonical_origin(&wsh_of(multi_2of3())).unwrap();
        assert_eq!(
            got,
            mk_origin(&[(true, 48), (true, 0), (true, 0), (true, 2)])
        );
    }

    #[test]
    fn wsh_sortedmulti_returns_bip48_type_2() {
        let got = canonical_origin(&wsh_of(sortedmulti_2of3())).unwrap();
        assert_eq!(
            got,
            mk_origin(&[(true, 48), (true, 0), (true, 0), (true, 2)])
        );
    }

    #[test]
    fn sh_wsh_multi_returns_bip48_type_1() {
        let got = canonical_origin(&sh_of(wsh_of(multi_2of3()))).unwrap();
        assert_eq!(
            got,
            mk_origin(&[(true, 48), (true, 0), (true, 0), (true, 1)])
        );
    }

    #[test]
    fn sh_wsh_sortedmulti_returns_bip48_type_1() {
        let got = canonical_origin(&sh_of(wsh_of(sortedmulti_2of3()))).unwrap();
        assert_eq!(
            got,
            mk_origin(&[(true, 48), (true, 0), (true, 0), (true, 1)])
        );
    }

    #[test]
    fn sh_sortedmulti_legacy_returns_none() {
        // sh(sortedmulti(...)) — legacy P2SH multi, not nested in wsh.
        assert_eq!(canonical_origin(&sh_of(sortedmulti_2of3())), None);
    }

    #[test]
    fn sh_multi_legacy_returns_none() {
        // sh(multi(...)) — legacy P2SH multi, not nested in wsh.
        assert_eq!(canonical_origin(&sh_of(multi_2of3())), None);
    }

    #[test]
    fn bare_wsh_at_n_returns_none() {
        // wsh(@N) — not allowed as a canonical shape; needs explicit override.
        // The inner here is a single pk_k(@0) (single-key wsh, not multisig).
        let inner = Node {
            tag: Tag::PkK,
            body: Body::KeyArg { index: 0 },
        };
        assert_eq!(canonical_origin(&wsh_of(inner)), None);
    }

    #[test]
    fn bare_sh_at_n_returns_none() {
        // sh(@N) — not allowed as a canonical shape.
        let inner = Node {
            tag: Tag::PkK,
            body: Body::KeyArg { index: 0 },
        };
        assert_eq!(canonical_origin(&sh_of(inner)), None);
    }

    #[test]
    fn wsh_with_miniscript_body_returns_none() {
        // wsh(or_d(pk_k(@0), pk_h(@1))) — miniscript body, not a bare
        // multi/sortedmulti. Must be forced explicit.
        let inner = Node {
            tag: Tag::OrD,
            body: Body::Children(vec![
                Node {
                    tag: Tag::PkK,
                    body: Body::KeyArg { index: 0 },
                },
                Node {
                    tag: Tag::PkH,
                    body: Body::KeyArg { index: 1 },
                },
            ]),
        };
        assert_eq!(canonical_origin(&wsh_of(inner)), None);
    }

    #[test]
    fn tr_shape_disambiguation_pair_returns_different_verdicts() {
        // Same outer Tag::Tr, but Body::Tr.tree differs: None → Some(BIP-86),
        // Some(_) → None. Disambiguates on body shape, not just tag.
        let keypath = tr_keypath(0);
        let with_tree = tr_with_taptree(0);
        assert!(canonical_origin(&keypath).is_some());
        assert_eq!(canonical_origin(&with_tree), None);
    }
}
