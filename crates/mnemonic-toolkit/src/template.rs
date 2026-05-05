//! `--template` clap enum + origin paths + md1 wrapper construction.
//!
//! Realizes SPEC §2.1.3 (10 templates: 4 single-sig + 6 multisig), §4.2
//! (origin paths), §4.6.3 (per-template wrapper tag + body).

use crate::network::CliNetwork;
use bitcoin::bip32::DerivationPath;
use clap::ValueEnum;
use md_codec::origin_path::{OriginPath, PathComponent};
use md_codec::tag::Tag;
use md_codec::tree::{Body, Node};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliTemplate {
    #[value(name = "bip44")]
    Bip44,
    #[value(name = "bip49")]
    Bip49,
    #[value(name = "bip84")]
    Bip84,
    #[value(name = "bip86")]
    Bip86,
    /// `wsh(multi(K,@0,...,@N-1))` — segwit unsorted multisig.
    #[value(name = "wsh-multi")]
    WshMulti,
    /// `wsh(sortedmulti(K,@0,...,@N-1))` — segwit sorted multisig.
    #[value(name = "wsh-sortedmulti")]
    WshSortedMulti,
    /// `sh(wsh(multi(K,...)))` — nested-segwit unsorted multisig.
    #[value(name = "sh-wsh-multi")]
    ShWshMulti,
    /// `sh(wsh(sortedmulti(K,...)))` — nested-segwit sorted multisig.
    #[value(name = "sh-wsh-sortedmulti")]
    ShWshSortedMulti,
    /// `tr(@0,multi_a(K,@1,...,@N))` — taproot unsorted multisig (script-path leaf).
    #[value(name = "tr-multi-a")]
    TrMultiA,
    /// `tr(@0,sortedmulti_a(K,@1,...,@N))` — taproot sorted multisig (script-path leaf).
    #[value(name = "tr-sortedmulti-a")]
    TrSortedMultiA,
}

impl CliTemplate {
    /// True if this template is a multisig wrapper (Phase B v0.2).
    pub fn is_multisig(&self) -> bool {
        matches!(
            self,
            CliTemplate::WshMulti
                | CliTemplate::WshSortedMulti
                | CliTemplate::ShWshMulti
                | CliTemplate::ShWshSortedMulti
                | CliTemplate::TrMultiA
                | CliTemplate::TrSortedMultiA,
        )
    }

    /// BIP-32 origin path for this (template, network, account) cell — single-sig only.
    /// Multisig templates don't have a fixed single origin path; callers must use
    /// `MultisigPathFamily::default_origin_path` instead.
    pub fn origin_path_str(&self, network: CliNetwork, account: u32) -> String {
        let purpose = match self {
            CliTemplate::Bip44 => 44,
            CliTemplate::Bip49 => 49,
            CliTemplate::Bip84 => 84,
            CliTemplate::Bip86 => 86,
            // Multisig templates default to BIP-87 path m/87'/coin'/account'
            // (used only by single-sig consumers and engraving-card defaults).
            _ => 87,
        };
        format!("m/{purpose}'/{}'/{}'", network.coin_type(), account)
    }

    /// Parsed BIP-32 derivation path for use with `bitcoin::bip32`.
    pub fn derivation_path(&self, network: CliNetwork, account: u32) -> DerivationPath {
        DerivationPath::from_str(&self.origin_path_str(network, account))
            .expect("template paths are well-formed by construction")
    }

    /// md-codec OriginPath for this (template, network, account) cell.
    /// Used in PathDeclPaths::Shared(...) for synthesize.rs.
    pub fn md_origin_path(&self, network: CliNetwork, account: u32) -> OriginPath {
        let purpose: u32 = match self {
            CliTemplate::Bip44 => 44,
            CliTemplate::Bip49 => 49,
            CliTemplate::Bip84 => 84,
            CliTemplate::Bip86 => 86,
            _ => 87,
        };
        OriginPath {
            components: vec![
                PathComponent {
                    hardened: true,
                    value: purpose,
                },
                PathComponent {
                    hardened: true,
                    value: network.coin_type(),
                },
                PathComponent {
                    hardened: true,
                    value: account,
                },
            ],
        }
    }

    /// md-codec wrapper Node for this template (SPEC §4.6.3).
    /// Single-sig variants ignore `k`/`n` (assert n==1); multisig variants
    /// construct `Body::Variable { k, children: <N PkK leaves> }` per PLAN B.1.
    pub fn wrapper_node(&self, k: u8, n: usize) -> Node {
        match self {
            CliTemplate::Bip44 => {
                debug_assert_eq!(n, 1);
                debug_assert_eq!(k, 1);
                Node {
                    tag: Tag::Pkh,
                    body: Body::KeyArg { index: 0 },
                }
            }
            CliTemplate::Bip49 => {
                debug_assert_eq!(n, 1);
                debug_assert_eq!(k, 1);
                Node {
                    tag: Tag::Sh,
                    body: Body::Children(vec![Node {
                        tag: Tag::Wpkh,
                        body: Body::KeyArg { index: 0 },
                    }]),
                }
            }
            CliTemplate::Bip84 => {
                debug_assert_eq!(n, 1);
                debug_assert_eq!(k, 1);
                Node {
                    tag: Tag::Wpkh,
                    body: Body::KeyArg { index: 0 },
                }
            }
            CliTemplate::Bip86 => {
                debug_assert_eq!(n, 1);
                debug_assert_eq!(k, 1);
                Node {
                    tag: Tag::Tr,
                    body: Body::Tr {
                        key_index: 0,
                        tree: None,
                    },
                }
            }
            CliTemplate::WshMulti | CliTemplate::WshSortedMulti => {
                let inner_tag = if matches!(self, CliTemplate::WshMulti) {
                    Tag::Multi
                } else {
                    Tag::SortedMulti
                };
                Node {
                    tag: Tag::Wsh,
                    body: Body::Children(vec![Node {
                        tag: inner_tag,
                        body: Body::Variable {
                            k,
                            children: pk_k_leaves(0, n),
                        },
                    }]),
                }
            }
            CliTemplate::ShWshMulti | CliTemplate::ShWshSortedMulti => {
                let inner_tag = if matches!(self, CliTemplate::ShWshMulti) {
                    Tag::Multi
                } else {
                    Tag::SortedMulti
                };
                let wsh = Node {
                    tag: Tag::Wsh,
                    body: Body::Children(vec![Node {
                        tag: inner_tag,
                        body: Body::Variable {
                            k,
                            children: pk_k_leaves(0, n),
                        },
                    }]),
                };
                Node {
                    tag: Tag::Sh,
                    body: Body::Children(vec![wsh]),
                }
            }
            CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA => {
                let inner_tag = if matches!(self, CliTemplate::TrMultiA) {
                    Tag::MultiA
                } else {
                    Tag::SortedMultiA
                };
                // For tr-multi-a, all N keys are signing keys in the script-path
                // leaf; key_index=0 designates the BIP-86 NUMS internal key
                // emitted by md-codec when the tap-script-tree is non-empty.
                // (md-codec's compute_wallet_policy_id resolves the NUMS key
                // from the tlv.pubkeys at index 0; we match the convention.)
                Node {
                    tag: Tag::Tr,
                    body: Body::Tr {
                        key_index: 0,
                        tree: Some(Box::new(Node {
                            tag: inner_tag,
                            body: Body::Variable {
                                k,
                                children: pk_k_leaves(0, n),
                            },
                        })),
                    },
                }
            }
        }
    }

    pub fn human_name(&self) -> &'static str {
        match self {
            CliTemplate::Bip44 => "bip44",
            CliTemplate::Bip49 => "bip49",
            CliTemplate::Bip84 => "bip84",
            CliTemplate::Bip86 => "bip86",
            CliTemplate::WshMulti => "wsh-multi",
            CliTemplate::WshSortedMulti => "wsh-sortedmulti",
            CliTemplate::ShWshMulti => "sh-wsh-multi",
            CliTemplate::ShWshSortedMulti => "sh-wsh-sortedmulti",
            CliTemplate::TrMultiA => "tr-multi-a",
            CliTemplate::TrSortedMultiA => "tr-sortedmulti-a",
        }
    }
}

/// Build N `PkK` leaves with key indices `[start, start + n)`.
fn pk_k_leaves(start: u8, n: usize) -> Vec<Node> {
    (0..n)
        .map(|i| Node {
            tag: Tag::PkK,
            body: Body::KeyArg {
                index: start + i as u8,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_path_strings() {
        assert_eq!(
            CliTemplate::Bip44.origin_path_str(CliNetwork::Mainnet, 0),
            "m/44'/0'/0'"
        );
        assert_eq!(
            CliTemplate::Bip49.origin_path_str(CliNetwork::Testnet, 0),
            "m/49'/1'/0'"
        );
        assert_eq!(
            CliTemplate::Bip84.origin_path_str(CliNetwork::Signet, 0),
            "m/84'/1'/0'"
        );
        assert_eq!(
            CliTemplate::Bip86.origin_path_str(CliNetwork::Regtest, 0),
            "m/86'/1'/0'"
        );
    }

    #[test]
    fn md_origin_path_components() {
        let op = CliTemplate::Bip84.md_origin_path(CliNetwork::Mainnet, 0);
        assert_eq!(op.components.len(), 3);
        assert_eq!(op.components[0].value, 84);
        assert!(op.components[0].hardened);
        assert_eq!(op.components[1].value, 0); // mainnet coin
        assert_eq!(op.components[2].value, 0); // account
    }

    #[test]
    fn origin_path_with_nonzero_account() {
        assert_eq!(
            CliTemplate::Bip84.origin_path_str(CliNetwork::Mainnet, 5),
            "m/84'/0'/5'"
        );
        let op = CliTemplate::Bip84.md_origin_path(CliNetwork::Mainnet, 5);
        assert_eq!(op.components[2].value, 5);
        assert!(op.components[2].hardened);
    }

    #[test]
    fn wrapper_nodes_per_template() {
        assert!(matches!(
            CliTemplate::Bip44.wrapper_node(1, 1).tag,
            Tag::Pkh
        ));
        assert!(matches!(CliTemplate::Bip49.wrapper_node(1, 1).tag, Tag::Sh));
        assert!(matches!(
            CliTemplate::Bip84.wrapper_node(1, 1).tag,
            Tag::Wpkh
        ));
        assert!(matches!(CliTemplate::Bip86.wrapper_node(1, 1).tag, Tag::Tr));
    }

    #[test]
    fn bip49_nests_wpkh_under_sh() {
        let n = CliTemplate::Bip49.wrapper_node(1, 1);
        if let Body::Children(children) = &n.body {
            assert_eq!(children.len(), 1);
            assert!(matches!(children[0].tag, Tag::Wpkh));
            assert!(matches!(children[0].body, Body::KeyArg { index: 0 }));
        } else {
            panic!("bip49 should nest wpkh under sh via Body::Children");
        }
    }

    #[test]
    fn bip86_uses_body_tr_keypath_only() {
        let n = CliTemplate::Bip86.wrapper_node(1, 1);
        assert!(matches!(
            n.body,
            Body::Tr {
                key_index: 0,
                tree: None
            }
        ));
    }

    #[test]
    fn multisig_predicate_correct() {
        assert!(!CliTemplate::Bip44.is_multisig());
        assert!(!CliTemplate::Bip86.is_multisig());
        assert!(CliTemplate::WshMulti.is_multisig());
        assert!(CliTemplate::WshSortedMulti.is_multisig());
        assert!(CliTemplate::ShWshMulti.is_multisig());
        assert!(CliTemplate::ShWshSortedMulti.is_multisig());
        assert!(CliTemplate::TrMultiA.is_multisig());
        assert!(CliTemplate::TrSortedMultiA.is_multisig());
    }

    #[test]
    fn wsh_sortedmulti_2_of_3_shape() {
        let n = CliTemplate::WshSortedMulti.wrapper_node(2, 3);
        assert!(matches!(n.tag, Tag::Wsh));
        let Body::Children(ref children) = n.body else {
            panic!("wsh body must be Children");
        };
        assert_eq!(children.len(), 1);
        assert!(matches!(children[0].tag, Tag::SortedMulti));
        let Body::Variable { k, ref children } = children[0].body else {
            panic!("inner sortedmulti body must be Variable");
        };
        assert_eq!(k, 2);
        assert_eq!(children.len(), 3);
        for (i, c) in children.iter().enumerate() {
            assert!(matches!(c.tag, Tag::PkK));
            assert!(matches!(c.body, Body::KeyArg { index } if index as usize == i));
        }
    }

    #[test]
    fn sh_wsh_sortedmulti_2_of_2_shape() {
        let n = CliTemplate::ShWshSortedMulti.wrapper_node(2, 2);
        assert!(matches!(n.tag, Tag::Sh));
        let Body::Children(ref sh_children) = n.body else {
            panic!("sh body must be Children");
        };
        assert_eq!(sh_children.len(), 1);
        assert!(matches!(sh_children[0].tag, Tag::Wsh));
    }

    #[test]
    fn tr_multi_a_2_of_2_shape() {
        let n = CliTemplate::TrMultiA.wrapper_node(2, 2);
        assert!(matches!(n.tag, Tag::Tr));
        let Body::Tr {
            key_index,
            ref tree,
        } = n.body
        else {
            panic!("tr body must be Tr");
        };
        assert_eq!(key_index, 0);
        let leaf = tree.as_deref().expect("tr-multi-a must have tree");
        assert!(matches!(leaf.tag, Tag::MultiA));
    }

    /// Phase B.1 mini-spike (resolves L-2 from PLAN r1 review):
    /// a TrSortedMultiA 2-of-2 wrapper round-trips through md-codec's
    /// chunk::split + chunk::reassemble and yields `is_wallet_policy() == true`.
    #[test]
    fn tr_sortedmulti_a_2_of_2_round_trips_via_md_codec() {
        use md_codec::origin_path::{PathDecl, PathDeclPaths};
        use md_codec::use_site_path::UseSitePath;
        use md_codec::{Descriptor, TlvSection};

        // Canonical 65-byte synthetic xpub filler from md-codec's
        // one_test_xpub_bytes() — chain_code = [0x42; 32], pubkey = SEC1
        // compressed secp256k1 generator G (passes validate_xpub_bytes).
        let mut xpub_bytes = [0u8; 65];
        xpub_bytes[0..32].copy_from_slice(&[0x42; 32]);
        xpub_bytes[32] = 0x02;
        xpub_bytes[33..].copy_from_slice(&[
            0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87,
            0x0B, 0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B,
            0x16, 0xF8, 0x17, 0x98,
        ]);

        // 2-of-2 tr-sortedmulti-a wrapper.
        let tree = CliTemplate::TrSortedMultiA.wrapper_node(2, 2);

        // Build descriptor with 2 xpubs (both copies of the same valid 65-byte
        // filler — chain_code prefix is unvalidated, so distinct entries are
        // allowed even at the same content).
        let path = OriginPath {
            components: vec![
                PathComponent {
                    hardened: true,
                    value: 48,
                },
                PathComponent {
                    hardened: true,
                    value: 0,
                },
                PathComponent {
                    hardened: true,
                    value: 0,
                },
                PathComponent {
                    hardened: true,
                    value: 2,
                },
            ],
        };

        let descriptor = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(path),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree,
            tlv: TlvSection {
                use_site_path_overrides: None,
                fingerprints: Some(vec![
                    (0, [0xAA, 0xBB, 0xCC, 0xDD]),
                    (1, [0x11, 0x22, 0x33, 0x44]),
                ]),
                pubkeys: Some(vec![(0, xpub_bytes), (1, xpub_bytes)]),
                origin_path_overrides: None,
                unknown: Vec::new(),
            },
        };

        let strings = md_codec::chunk::split(&descriptor)
            .expect("chunk::split must accept tr-sortedmulti-a 2-of-2");
        let strs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
        let recovered = md_codec::chunk::reassemble(&strs)
            .expect("chunk::reassemble must accept tr-sortedmulti-a 2-of-2");
        assert!(
            recovered.is_wallet_policy(),
            "tr-sortedmulti-a 2-of-2 round-trip must be wallet-policy"
        );
    }
}
