//! Integration tests for `Descriptor::derive_address` (md1 v0.32+).
//!
//! Each test follows the same shape: derive an account-level xpub from
//! a known mnemonic via rust-bitcoin's bip32 (trusted), pack the
//! `(chain_code, compressed_pubkey)` bytes into the v0.13 `Pubkeys` TLV,
//! then ask md-codec to derive an address and assert it matches a
//! golden vector from the relevant BIP's published test vectors (or, for
//! generic shapes, an independent miniscript-direct derivation done
//! in-test through `miniscript::Descriptor::<DescriptorPublicKey>::from_str`).
//!
//! Feature-gated behind `derive` (default-on; gates the
//! `Descriptor::derive_address` API).

#![cfg(feature = "derive")]

use bitcoin::Network;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;
use md_codec::{Descriptor, OriginPath, PathComponent, PathDecl, PathDeclPaths, Tag, TlvSection};
use std::str::FromStr;

/// The "abandon abandon abandon abandon abandon abandon abandon abandon
/// abandon abandon abandon about" mnemonic — used by BIP 84, BIP 86,
/// BIP 49, and BIP 44 published test vectors.
const ABANDON_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Derive the account-level xpub for the abandon-mnemonic at `path`.
/// Returns a 65-byte `(chain_code || compressed_pubkey)` payload as it
/// would appear in a v0.13 `Pubkeys` TLV entry.
fn account_xpub_bytes(path_str: &str) -> [u8; 65] {
    let mn = bip39::Mnemonic::parse(ABANDON_MNEMONIC).expect("known good mnemonic");
    let seed = mn.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(Network::Bitcoin, &seed).expect("seed → master");
    let path = DerivationPath::from_str(path_str).expect("valid path");
    let account_xpriv = master.derive_priv(&secp, &path).expect("derive priv");
    let account_xpub = Xpub::from_priv(&secp, &account_xpriv);
    let mut out = [0u8; 65];
    out[..32].copy_from_slice(account_xpub.chain_code.as_ref());
    out[32..].copy_from_slice(&account_xpub.public_key.serialize());
    out
}

fn origin(components: &[(bool, u32)]) -> OriginPath {
    OriginPath {
        components: components
            .iter()
            .map(|&(hardened, value)| PathComponent { hardened, value })
            .collect(),
    }
}

fn pkk(index: u8) -> Node {
    Node {
        tag: Tag::PkK,
        body: Body::KeyArg { index },
    }
}

/// BIP 84 test vector — `abandon abandon ... about` at `m/84'/0'/0'/0/0`
/// produces P2WPKH `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`.
/// Source: <https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki>
#[test]
fn bip84_wpkh_receive_address_zero() {
    let xpub_bytes = account_xpub_bytes("m/84'/0'/0'");
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_bytes)]);
            t
        },
    };
    let addr = d.derive_address(0, 0, Network::Bitcoin).unwrap();
    assert_eq!(
        addr.assume_checked().to_string(),
        "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
    );
}

/// BIP 84 — second receive address `m/84'/0'/0'/0/1` is
/// `bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g`.
#[test]
fn bip84_wpkh_receive_address_one() {
    let xpub_bytes = account_xpub_bytes("m/84'/0'/0'");
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_bytes)]);
            t
        },
    };
    let addr = d.derive_address(0, 1, Network::Bitcoin).unwrap();
    assert_eq!(
        addr.assume_checked().to_string(),
        "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g"
    );
}

/// BIP 84 — first change address `m/84'/0'/0'/1/0` is
/// `bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el`. Confirms `chain=1`
/// selects the change branch of the `<0;1>/*` multipath.
#[test]
fn bip84_wpkh_change_address_zero() {
    let xpub_bytes = account_xpub_bytes("m/84'/0'/0'");
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_bytes)]);
            t
        },
    };
    let addr = d.derive_address(1, 0, Network::Bitcoin).unwrap();
    assert_eq!(
        addr.assume_checked().to_string(),
        "bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el"
    );
}

/// BIP 86 test vector — `abandon abandon ... about` at `m/86'/0'/0'/0/0`
/// produces P2TR keypath-only address
/// `bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr`.
/// Confirms BIP 86 NUMS taproot tweak.
/// Source: <https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki>
#[test]
fn bip86_tr_keypath_only_receive_address_zero() {
    let xpub_bytes = account_xpub_bytes("m/86'/0'/0'");
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 86), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: None,
            },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_bytes)]);
            t
        },
    };
    let addr = d.derive_address(0, 0, Network::Bitcoin).unwrap();
    assert_eq!(
        addr.assume_checked().to_string(),
        "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr"
    );
}

/// BIP 44 test vector — `abandon abandon ... about` at `m/44'/0'/0'/0/0`
/// produces P2PKH address `1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA`.
/// Cross-checked against multiple wallet implementations (Electrum,
/// Sparrow, BlueWallet) using the same well-known test mnemonic.
#[test]
fn bip44_pkh_receive_address_zero() {
    let xpub_bytes = account_xpub_bytes("m/44'/0'/0'");
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 44), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Pkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_bytes)]);
            t
        },
    };
    let addr = d.derive_address(0, 0, Network::Bitcoin).unwrap();
    assert_eq!(
        addr.assume_checked().to_string(),
        "1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA"
    );
}

/// Same wpkh wallet as `bip84_wpkh_receive_address_zero` but on
/// `Network::Testnet` produces a `tb1q…` address. Verifies network
/// parameter end-to-end.
#[test]
fn bip84_wpkh_testnet_address() {
    let xpub_bytes = account_xpub_bytes("m/84'/0'/0'");
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_bytes)]);
            t
        },
    };
    let addr = d.derive_address(0, 0, Network::Testnet).unwrap();
    let s = addr.assume_checked().to_string();
    assert!(s.starts_with("tb1q"), "expected testnet bech32, got {s}");
}

/// 2-of-3 wsh-sortedmulti from three independent abandon-mnemonics-like
/// xpubs. Cross-checks the miniscript-converter path
/// (`to_miniscript_descriptor` + `at_derivation_index`) against
/// rust-bitcoin's own primitives applied independently in-test.
#[test]
fn wsh_sortedmulti_2_of_3_address() {
    use bitcoin::bip32::ChildNumber;

    // Three different account paths under the same abandon-mnemonic
    // master: 0', 1', 2'. Gives three independent xpubs without
    // needing three distinct mnemonics.
    let xpub_a = account_xpub_bytes("m/48'/0'/0'/2'");
    let xpub_b = account_xpub_bytes("m/48'/0'/1'/2'");
    let xpub_c = account_xpub_bytes("m/48'/0'/2'/2'");

    let d = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(origin(&[(true, 48), (true, 0), (true, 0), (true, 2)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![0, 1, 2],
                },
            }]),
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_a), (1u8, xpub_b), (2u8, xpub_c)]);
            t
        },
    };
    let addr = d.derive_address(0, 0, Network::Bitcoin).unwrap();
    let got = addr.assume_checked().to_string();

    // Independent verification: do the same math by hand using
    // rust-bitcoin primitives, no md-codec helpers, then assert match.
    let secp = Secp256k1::verification_only();
    let mut pks: Vec<bitcoin::secp256k1::PublicKey> = vec![];
    for bytes in [&xpub_a, &xpub_b, &xpub_c] {
        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&bytes[..32]);
        let pubkey = bitcoin::secp256k1::PublicKey::from_slice(&bytes[32..]).unwrap();
        let xpub = Xpub {
            network: bitcoin::NetworkKind::Main,
            depth: 0,
            parent_fingerprint: Default::default(),
            child_number: ChildNumber::Normal { index: 0 },
            public_key: pubkey,
            chain_code: bitcoin::bip32::ChainCode::from(chain_code),
        };
        let leaf = xpub
            .derive_pub(
                &secp,
                &[
                    ChildNumber::Normal { index: 0 },
                    ChildNumber::Normal { index: 0 },
                ],
            )
            .unwrap();
        pks.push(leaf.public_key);
    }
    pks.sort_by_key(|p| p.serialize());
    let mut b = bitcoin::blockdata::script::Builder::new().push_int(2);
    for p in &pks {
        b = b.push_key(&bitcoin::PublicKey::new(*p));
    }
    let script = b
        .push_int(3)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    let expected = bitcoin::Address::p2wsh(&script, Network::Bitcoin).to_string();

    assert_eq!(got, expected);
    assert!(
        got.starts_with("bc1q"),
        "expected mainnet wsh bech32, got {got}"
    );
}

/// `sh(wsh(sortedmulti(2, ...)))` — BIP 48 type 1 (nested-segwit
/// multi). Independent verification through a parallel rust-bitcoin
/// path; asserts a `3...` mainnet P2SH-form address.
#[test]
fn sh_wsh_sortedmulti_2_of_3_address() {
    use bitcoin::bip32::ChildNumber;

    let xpub_a = account_xpub_bytes("m/48'/0'/0'/1'");
    let xpub_b = account_xpub_bytes("m/48'/0'/1'/1'");
    let xpub_c = account_xpub_bytes("m/48'/0'/2'/1'");

    let d = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(origin(&[(true, 48), (true, 0), (true, 0), (true, 1)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Sh,
            body: Body::Children(vec![Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1, 2],
                    },
                }]),
            }]),
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_a), (1u8, xpub_b), (2u8, xpub_c)]);
            t
        },
    };
    let addr = d.derive_address(0, 0, Network::Bitcoin).unwrap();
    let got = addr.assume_checked().to_string();

    let secp = Secp256k1::verification_only();
    let mut pks: Vec<bitcoin::secp256k1::PublicKey> = vec![];
    for bytes in [&xpub_a, &xpub_b, &xpub_c] {
        let mut chain_code = [0u8; 32];
        chain_code.copy_from_slice(&bytes[..32]);
        let pubkey = bitcoin::secp256k1::PublicKey::from_slice(&bytes[32..]).unwrap();
        let xpub = Xpub {
            network: bitcoin::NetworkKind::Main,
            depth: 0,
            parent_fingerprint: Default::default(),
            child_number: ChildNumber::Normal { index: 0 },
            public_key: pubkey,
            chain_code: bitcoin::bip32::ChainCode::from(chain_code),
        };
        let leaf = xpub
            .derive_pub(
                &secp,
                &[
                    ChildNumber::Normal { index: 0 },
                    ChildNumber::Normal { index: 0 },
                ],
            )
            .unwrap();
        pks.push(leaf.public_key);
    }
    pks.sort_by_key(|p| p.serialize());
    let mut b = bitcoin::blockdata::script::Builder::new().push_int(2);
    for p in &pks {
        b = b.push_key(&bitcoin::PublicKey::new(*p));
    }
    let script = b
        .push_int(3)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    let expected = bitcoin::Address::p2shwsh(&script, Network::Bitcoin).to_string();

    assert_eq!(got, expected);
    assert!(
        got.starts_with('3'),
        "expected mainnet P2SH-form, got {got}"
    );
}

// ─── v0.32 — generic-shape coverage via rust-miniscript shortcut ──
//
// Each new test pairs an md-codec derivation against an independent
// `miniscript::Descriptor<DescriptorPublicKey>::from_str(...)` derivation
// path; byte-identical addresses are the post-condition. Future drift
// between md-codec and upstream rust-miniscript will surface as a paired-
// test mismatch.

/// Pretty-print a 65-byte xpub TLV payload into a base58-check xpub
/// string suitable for embedding in a miniscript descriptor template.
fn xpub_bytes_to_string(bytes: &[u8; 65]) -> String {
    use bitcoin::NetworkKind;
    use bitcoin::bip32::{ChainCode, ChildNumber, Fingerprint, Xpub};
    use bitcoin::secp256k1::PublicKey;
    let mut chain_code = [0u8; 32];
    chain_code.copy_from_slice(&bytes[..32]);
    let pk = PublicKey::from_slice(&bytes[32..]).unwrap();
    let xpub = Xpub {
        network: NetworkKind::Main,
        depth: 0,
        parent_fingerprint: Fingerprint::default(),
        child_number: ChildNumber::Normal { index: 0 },
        public_key: pk,
        chain_code: ChainCode::from(chain_code),
    };
    xpub.to_string()
}

/// Independent miniscript-direct derivation: parse `descriptor_str`,
/// `.at_derivation_index(index).address(network).to_string()`.
fn miniscript_direct_address(
    descriptor_str: &str,
    chain: u32,
    index: u32,
    network: Network,
) -> String {
    let desc = miniscript::Descriptor::<miniscript::DescriptorPublicKey>::from_str(descriptor_str)
        .expect("parse descriptor template");
    // Multipath descriptors need explicit per-chain selection. The plan's
    // converter substitutes the chain alt before calling
    // `at_derivation_index`, so cross-validate by reducing the multipath
    // here too — pick one alt via `single_path_descriptors()`.
    let single = if desc.is_multipath() {
        let alternatives = desc
            .clone()
            .into_single_descriptors()
            .expect("split multipath");
        alternatives
            .into_iter()
            .nth(chain as usize)
            .expect("chain in range")
    } else {
        desc
    };
    let definite = single.at_derivation_index(index).expect("derivation idx");
    definite.address(network).expect("address").to_string()
}

/// Build the standard `<chain;chain'/*` multipath descriptor key suffix
/// for use in miniscript descriptor templates with `<0;1>/*`.
const MULTIPATH_TAIL: &str = "/<0;1>/*";

/// Tier 1 — `sh(sortedmulti(2, @0, @1, @2))` legacy P2SH multisig.
#[test]
fn sh_sortedmulti_2_of_3_address() {
    let xpub_a = account_xpub_bytes("m/45'/0'");
    let xpub_b = account_xpub_bytes("m/45'/1'");
    let xpub_c = account_xpub_bytes("m/45'/2'");

    let d = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Divergent(vec![
                origin(&[(true, 45), (true, 0)]),
                origin(&[(true, 45), (true, 1)]),
                origin(&[(true, 45), (true, 2)]),
            ]),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Sh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![0, 1, 2],
                },
            }]),
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_a), (1u8, xpub_b), (2u8, xpub_c)]);
            t
        },
    };
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let template = format!(
        "sh(sortedmulti(2,{a}{m},{b}{m},{c}{m}))",
        a = xpub_bytes_to_string(&xpub_a),
        b = xpub_bytes_to_string(&xpub_b),
        c = xpub_bytes_to_string(&xpub_c),
        m = MULTIPATH_TAIL,
    );
    let expected = miniscript_direct_address(&template, 0, 0, Network::Bitcoin);
    assert_eq!(got, expected);
    assert!(
        got.starts_with('3'),
        "expected mainnet P2SH-form, got {got}"
    );
}

/// Tier 1 — `tr(NUMS, pk(@0))` script-path-only taproot.
#[test]
fn tr_nums_single_pk_leaf_address() {
    let xpub_a = account_xpub_bytes("m/86'/0'/0'");
    let nums = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 86), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: true,
                key_index: 0,
                tree: Some(Box::new(pkk(0))),
            },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_a)]);
            t
        },
    };
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let template = format!(
        "tr({nums},pk({a}{m}))",
        a = xpub_bytes_to_string(&xpub_a),
        m = MULTIPATH_TAIL,
    );
    let expected = miniscript_direct_address(&template, 0, 0, Network::Bitcoin);
    assert_eq!(got, expected);
    assert!(got.starts_with("bc1p"), "expected mainnet P2TR, got {got}");
}

/// Tier 2 — `tr(@0, pk(@1))` single-leaf taproot with a regular internal
/// key + a script-path single-pk leaf.
#[test]
fn tr_single_pk_leaf_address() {
    let xpub_a = account_xpub_bytes("m/86'/0'/0'");
    let xpub_b = account_xpub_bytes("m/86'/0'/1'");

    let d = Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Divergent(vec![
                origin(&[(true, 86), (true, 0), (true, 0)]),
                origin(&[(true, 86), (true, 0), (true, 1)]),
            ]),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(pkk(1))),
            },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_a), (1u8, xpub_b)]);
            t
        },
    };
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let template = format!(
        "tr({a}{m},pk({b}{m}))",
        a = xpub_bytes_to_string(&xpub_a),
        b = xpub_bytes_to_string(&xpub_b),
        m = MULTIPATH_TAIL,
    );
    let expected = miniscript_direct_address(&template, 0, 0, Network::Bitcoin);
    assert_eq!(got, expected);
}

/// Tier 2 — `tr(@0, multi_a(2, @1, @2, @3))` tap-leaf multisig.
#[test]
fn tr_multi_a_2_of_3_leaf_address() {
    let xpub_a = account_xpub_bytes("m/86'/0'/0'");
    let xpub_b = account_xpub_bytes("m/86'/0'/1'");
    let xpub_c = account_xpub_bytes("m/86'/0'/2'");
    let xpub_d = account_xpub_bytes("m/86'/0'/3'");

    let d = Descriptor {
        n: 4,
        path_decl: PathDecl {
            n: 4,
            paths: PathDeclPaths::Divergent(vec![
                origin(&[(true, 86), (true, 0), (true, 0)]),
                origin(&[(true, 86), (true, 0), (true, 1)]),
                origin(&[(true, 86), (true, 0), (true, 2)]),
                origin(&[(true, 86), (true, 0), (true, 3)]),
            ]),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::MultiA,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![1, 2, 3],
                    },
                })),
            },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![
                (0u8, xpub_a),
                (1u8, xpub_b),
                (2u8, xpub_c),
                (3u8, xpub_d),
            ]);
            t
        },
    };
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let template = format!(
        "tr({a}{m},multi_a(2,{b}{m},{c}{m},{d}{m}))",
        a = xpub_bytes_to_string(&xpub_a),
        b = xpub_bytes_to_string(&xpub_b),
        c = xpub_bytes_to_string(&xpub_c),
        d = xpub_bytes_to_string(&xpub_d),
        m = MULTIPATH_TAIL,
    );
    let expected = miniscript_direct_address(&template, 0, 0, Network::Bitcoin);
    assert_eq!(got, expected);
}

/// Tier 2 — `wsh(pk(@0))` exercises the Phase E `Check(pk_k(...))`
/// re-wrapping path for bare PkK inside Wsh.
#[test]
fn wsh_check_pk_k_address() {
    let xpub_a = account_xpub_bytes("m/84'/0'/0'");

    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![pkk(0)]),
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_a)]);
            t
        },
    };
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let template = format!(
        "wsh(pk({a}{m}))",
        a = xpub_bytes_to_string(&xpub_a),
        m = MULTIPATH_TAIL,
    );
    let expected = miniscript_direct_address(&template, 0, 0, Network::Bitcoin);
    assert_eq!(got, expected);
}

/// Tier 3 — `tr(@0, {pk(@1), pk(@2)})` 2-leaf branching tap-tree.
#[test]
fn tr_branching_two_leaf_address() {
    let xpub_a = account_xpub_bytes("m/86'/0'/0'");
    let xpub_b = account_xpub_bytes("m/86'/0'/1'");
    let xpub_c = account_xpub_bytes("m/86'/0'/2'");

    let d = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Divergent(vec![
                origin(&[(true, 86), (true, 0), (true, 0)]),
                origin(&[(true, 86), (true, 0), (true, 1)]),
                origin(&[(true, 86), (true, 0), (true, 2)]),
            ]),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::TapTree,
                    body: Body::Children(vec![pkk(1), pkk(2)]),
                })),
            },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_a), (1u8, xpub_b), (2u8, xpub_c)]);
            t
        },
    };
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let template = format!(
        "tr({a}{m},{{pk({b}{m}),pk({c}{m})}})",
        a = xpub_bytes_to_string(&xpub_a),
        b = xpub_bytes_to_string(&xpub_b),
        c = xpub_bytes_to_string(&xpub_c),
        m = MULTIPATH_TAIL,
    );
    let expected = miniscript_direct_address(&template, 0, 0, Network::Bitcoin);
    assert_eq!(got, expected);
}

/// Tier 3 — `tr(@0, {pk(@1), multi_a(2, @2, @3)})` mixed-leaf tap-tree.
#[test]
fn tr_branching_with_multi_a_address() {
    let xpub_a = account_xpub_bytes("m/86'/0'/0'");
    let xpub_b = account_xpub_bytes("m/86'/0'/1'");
    let xpub_c = account_xpub_bytes("m/86'/0'/2'");
    let xpub_d = account_xpub_bytes("m/86'/0'/3'");

    let d = Descriptor {
        n: 4,
        path_decl: PathDecl {
            n: 4,
            paths: PathDeclPaths::Divergent(vec![
                origin(&[(true, 86), (true, 0), (true, 0)]),
                origin(&[(true, 86), (true, 0), (true, 1)]),
                origin(&[(true, 86), (true, 0), (true, 2)]),
                origin(&[(true, 86), (true, 0), (true, 3)]),
            ]),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::TapTree,
                    body: Body::Children(vec![
                        pkk(1),
                        Node {
                            tag: Tag::MultiA,
                            body: Body::MultiKeys {
                                k: 2,
                                indices: vec![2, 3],
                            },
                        },
                    ]),
                })),
            },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![
                (0u8, xpub_a),
                (1u8, xpub_b),
                (2u8, xpub_c),
                (3u8, xpub_d),
            ]);
            t
        },
    };
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let template = format!(
        "tr({a}{m},{{pk({b}{m}),multi_a(2,{c}{m},{d}{m})}})",
        a = xpub_bytes_to_string(&xpub_a),
        b = xpub_bytes_to_string(&xpub_b),
        c = xpub_bytes_to_string(&xpub_c),
        d = xpub_bytes_to_string(&xpub_d),
        m = MULTIPATH_TAIL,
    );
    let expected = miniscript_direct_address(&template, 0, 0, Network::Bitcoin);
    assert_eq!(got, expected);
}

/// Tier 3 — `wsh(and_v(v:pk(@0), older(144)))` arbitrary miniscript body
/// with a relative timelock.
#[test]
fn wsh_and_v_address() {
    let xpub_a = account_xpub_bytes("m/84'/0'/0'");

    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::AndV,
                body: Body::Children(vec![
                    Node {
                        tag: Tag::Verify,
                        body: Body::Children(vec![pkk(0)]),
                    },
                    Node {
                        tag: Tag::Older,
                        body: Body::Timelock(144),
                    },
                ]),
            }]),
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_a)]);
            t
        },
    };
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let template = format!(
        "wsh(and_v(v:pk({a}{m}),older(144)))",
        a = xpub_bytes_to_string(&xpub_a),
        m = MULTIPATH_TAIL,
    );
    let expected = miniscript_direct_address(&template, 0, 0, Network::Bitcoin);
    assert_eq!(got, expected);
}

/// Tier 3 — `wsh(thresh(2, pk(@0), s:pk(@1), s:pk(@2)))` wsh threshold
/// over three keys (canonical miniscript thresh shape).
#[test]
fn wsh_thresh_address() {
    let xpub_a = account_xpub_bytes("m/48'/0'/0'/2'");
    let xpub_b = account_xpub_bytes("m/48'/0'/1'/2'");
    let xpub_c = account_xpub_bytes("m/48'/0'/2'/2'");

    let d = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Divergent(vec![
                origin(&[(true, 48), (true, 0), (true, 0), (true, 2)]),
                origin(&[(true, 48), (true, 0), (true, 1), (true, 2)]),
                origin(&[(true, 48), (true, 0), (true, 2), (true, 2)]),
            ]),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::Thresh,
                body: Body::Variable {
                    k: 2,
                    children: vec![
                        pkk(0),
                        Node {
                            tag: Tag::Swap,
                            body: Body::Children(vec![pkk(1)]),
                        },
                        Node {
                            tag: Tag::Swap,
                            body: Body::Children(vec![pkk(2)]),
                        },
                    ],
                },
            }]),
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_a), (1u8, xpub_b), (2u8, xpub_c)]);
            t
        },
    };
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let template = format!(
        "wsh(thresh(2,pk({a}{m}),s:pk({b}{m}),s:pk({c}{m})))",
        a = xpub_bytes_to_string(&xpub_a),
        b = xpub_bytes_to_string(&xpub_b),
        c = xpub_bytes_to_string(&xpub_c),
        m = MULTIPATH_TAIL,
    );
    let expected = miniscript_direct_address(&template, 0, 0, Network::Bitcoin);
    assert_eq!(got, expected);
}

/// Round-trip: encode → wrap → unwrap → decode → derive_address yields
/// the same address as deriving on the source descriptor. Confirms the
/// derivation API plays well with the v0.13 wire round-trip.
#[test]
fn round_trip_then_derive_address() {
    let xpub_bytes = account_xpub_bytes("m/84'/0'/0'");
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub_bytes)]);
            t
        },
    };
    let direct = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    // This wallet-policy descriptor carries a populated 65-byte xpub TLV, whose
    // payload exceeds the codex32 regular code's 80-data-symbol single-string
    // cap (cycle-4 H6) → encode_md1_string now fails closed. Round-trip via the
    // chunked path (`split` → `reassemble`), the contractual remedy.
    assert!(
        matches!(
            md_codec::encode_md1_string(&d),
            Err(md_codec::Error::PayloadTooLongForSingleString { .. })
        ),
        "an oversize wallet-policy descriptor must reject the single-string encode"
    );
    let chunks = md_codec::chunk::split(&d).unwrap();
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    let decoded = md_codec::chunk::reassemble(&refs).unwrap();
    let after = decoded
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    assert_eq!(direct, after);
    assert_eq!(after, "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu");
}

// ── `to-miniscript-check-pkh-double-wrap` (PART 2) ──────────────────────────
// The toolkit walker emits an explicit `Tag::Check(Tag::PkK/PkH)` wire node in
// non-tap context (pre-v0.30 md-cli cards too). The renderer re-applies `Check`
// in the PkK/PkH arms AND wrapped a second `Check` in the `Tag::Check` arm →
// `Check(Check(PkH))` = `c:` over type-B → "cannot wrap a fragment of type B".
// The Check-idempotence collapse renders the bare-key child directly.

/// 1-key Wsh-wrapped policy whose single child is `child`.
fn wsh_one_key_descriptor(child: Node, xpub: [u8; 65]) -> Descriptor {
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![child]),
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub)]);
            t
        },
    }
}

fn render(d: &Descriptor) -> Result<String, md_codec::Error> {
    md_codec::to_miniscript::to_miniscript_descriptor(d, 0).map(|x| x.to_string())
}

/// `Tag::Check(Tag::PkH)` (toolkit dialect) renders IDENTICALLY to bare
/// `Tag::PkH` (md-cli canonical) — `wsh(pkh(...))`. RED pre-fix (the Check(PkH)
/// shape errored). Pins the literal `pkh(` keyword (R0-r1 Minor 1).
#[test]
fn wsh_check_pkh_renders_same_as_bare_pkh() {
    let xpub = account_xpub_bytes("m/84'/0'/0'");
    let bare = render(&wsh_one_key_descriptor(
        Node {
            tag: Tag::PkH,
            body: Body::KeyArg { index: 0 },
        },
        xpub,
    ))
    .expect("bare PkH renders");
    let checked = render(&wsh_one_key_descriptor(
        Node {
            tag: Tag::Check,
            body: Body::Children(vec![Node {
                tag: Tag::PkH,
                body: Body::KeyArg { index: 0 },
            }]),
        },
        xpub,
    ))
    .expect("Check(PkH) must render after the idempotence fix");
    assert_eq!(
        checked, bare,
        "Check(PkH) must render identically to bare PkH"
    );
    assert!(
        checked.contains("pkh("),
        "must be a pkh fragment, not pk/collapsed: {checked}"
    );
    assert!(
        !checked.contains("pk(") || checked.contains("pkh("),
        "no pk() mis-render"
    );
}

/// `Tag::Check(Tag::PkK)` renders identically to bare `Tag::PkK` — `wsh(pk(...))`.
/// RED pre-fix.
#[test]
fn wsh_check_pk_k_explicit_node_renders_same_as_bare() {
    let xpub = account_xpub_bytes("m/84'/0'/0'");
    let bare = render(&wsh_one_key_descriptor(pkk(0), xpub)).expect("bare PkK renders");
    let checked = render(&wsh_one_key_descriptor(
        Node {
            tag: Tag::Check,
            body: Body::Children(vec![pkk(0)]),
        },
        xpub,
    ))
    .expect("Check(PkK) must render after the fix");
    assert_eq!(checked, bare);
    assert!(checked.contains("pk("));
}

/// Boundary pin (R0-r1 Minor 2): the deferred shape C — `Check(or_i(pk_k,pk_k))`
/// — still ERRORS post-fix (the collapse is gated on a BARE-KEY child; an `or_i`
/// child double-wraps and `c:` over type-B is rejected). Never a wrong
/// descriptor. Full support is tracked under the A2 follow-up.
#[test]
fn wsh_check_or_i_shape_c_still_errors() {
    let xpub0 = account_xpub_bytes("m/84'/0'/0'");
    let xpub1 = account_xpub_bytes("m/84'/0'/1'");
    let d = Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84), (true, 0), (true, 0)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::Check,
                body: Body::Children(vec![Node {
                    tag: Tag::OrI,
                    body: Body::Children(vec![pkk(0), pkk(1)]),
                }]),
            }]),
        },
        tlv: {
            let mut t = TlvSection::new_empty();
            t.pubkeys = Some(vec![(0u8, xpub0), (1u8, xpub1)]);
            t
        },
    };
    assert!(
        render(&d).is_err(),
        "shape C (Check over or_i) must still error, not mis-render"
    );
}
