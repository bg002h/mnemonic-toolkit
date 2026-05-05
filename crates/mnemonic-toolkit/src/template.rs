//! `--template` clap enum + origin paths + md1 wrapper construction.
//!
//! Realizes SPEC §2.1.3 (4 templates), §4.2 (origin paths), §4.6.3
//! (per-template wrapper tag + body).

use crate::network::CliNetwork;
use bitcoin::bip32::DerivationPath;
use clap::ValueEnum;
use md_codec::origin_path::{OriginPath, PathComponent};
use md_codec::tag::Tag;
use md_codec::tree::{Body, Node};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum CliTemplate {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

impl CliTemplate {
    /// BIP-32 origin path for this (template, network, account) cell.
    pub fn origin_path_str(&self, network: CliNetwork, account: u32) -> String {
        let purpose = match self {
            CliTemplate::Bip44 => 44,
            CliTemplate::Bip49 => 49,
            CliTemplate::Bip84 => 84,
            CliTemplate::Bip86 => 86,
        };
        format!("m/{purpose}'/{}'/{}'", network.coin_type(), account)
    }

    /// Parsed BIP-32 derivation path for use with `bitcoin::bip32`.
    pub fn derivation_path(&self, network: CliNetwork, account: u32) -> DerivationPath {
        DerivationPath::from_str(&self.origin_path_str(network, account))
            .expect("template paths are well-formed by construction")
    }

    /// md-codec OriginPath for this (template, network, account) cell.
    /// Used in PathDeclPaths::Shared(...) for Phase 2 synthesize.rs.
    pub fn md_origin_path(&self, network: CliNetwork, account: u32) -> OriginPath {
        let purpose: u32 = match self {
            CliTemplate::Bip44 => 44,
            CliTemplate::Bip49 => 49,
            CliTemplate::Bip84 => 84,
            CliTemplate::Bip86 => 86,
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
    /// All v0.1 templates use placeholder index 0 (single-sig).
    pub fn wrapper_node(&self) -> Node {
        match self {
            CliTemplate::Bip44 => Node {
                tag: Tag::Pkh,
                body: Body::KeyArg { index: 0 },
            },
            CliTemplate::Bip49 => Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::Wpkh,
                    body: Body::KeyArg { index: 0 },
                }]),
            },
            CliTemplate::Bip84 => Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            },
            CliTemplate::Bip86 => Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    key_index: 0,
                    tree: None,
                },
            },
        }
    }

    pub fn human_name(&self) -> &'static str {
        match self {
            CliTemplate::Bip44 => "bip44",
            CliTemplate::Bip49 => "bip49",
            CliTemplate::Bip84 => "bip84",
            CliTemplate::Bip86 => "bip86",
        }
    }
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
        assert!(matches!(CliTemplate::Bip44.wrapper_node().tag, Tag::Pkh));
        assert!(matches!(CliTemplate::Bip49.wrapper_node().tag, Tag::Sh));
        assert!(matches!(CliTemplate::Bip84.wrapper_node().tag, Tag::Wpkh));
        assert!(matches!(CliTemplate::Bip86.wrapper_node().tag, Tag::Tr));
    }

    #[test]
    fn bip49_nests_wpkh_under_sh() {
        let n = CliTemplate::Bip49.wrapper_node();
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
        let n = CliTemplate::Bip86.wrapper_node();
        assert!(matches!(
            n.body,
            Body::Tr {
                key_index: 0,
                tree: None
            }
        ));
    }
}
