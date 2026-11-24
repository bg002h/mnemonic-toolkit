//! F-672: every key in a rendered descriptor carries the REQUESTED network's
//! version bytes.
//!
//! md1 stores a key as 65 bytes (`chain_code || pubkey`); the BIP-32 version
//! bytes are not on the wire. Before F-672 the `_with_network` entry points
//! threaded the network only into a wire-kind-1 (Liana) internal key, and
//! every other key was serialised as mainnet `xpub`. Under `--network
//! regtest` that produced `tr(tpub…,{multi_a(2,[…]xpub…,…)})`, which Bitcoin
//! Core refuses (`Multi: key 'xpub6DXu…' is not valid`; engrave
//! `design/agent-reports/e2e-live-site-wallets.md` D-2).
//!
//! GOLDENS COME FROM OUTSIDE md-codec's network code:
//! `tests/fixtures/network/<vector>.<form>.test.txt` is the MAINNET render
//! (unchanged by F-672) with every `xpub` base58check-decoded, its version
//! replaced by `043587CF`, re-encoded, and the BIP-380 checksum recomputed —
//! `tests/fixtures/network/derive.py`, which never calls md.
//!
//! The four vectors cover the four internal-key/wrapper cases: kind 0
//! (NUMS), kind 1 (Liana's derived key), a real internal key, and `wsh`.

#![cfg(feature = "derive")]

use bitcoin::{Network, NetworkKind};
use miniscript::ForEachKey;
use miniscript::descriptor::DescriptorPublicKey;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/vendored.rs"
));

const VECTORS: [&str; 4] = [
    "keyed_compose_tr_two_path_nums", // kind 0: NUMS internal key
    "keyed_tr_liana_kofn_recovery",   // kind 1: Liana's derived internal key
    "keyed_tr_with_leaf",             // a real internal key (slot @0)
    "keyed_wsh_multi_2of3",           // wsh
];

const TEST_NETWORKS: [Network; 3] = [Network::Testnet, Network::Signet, Network::Regtest];

fn golden(name: &str, form: &str) -> String {
    let p = std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/network"
    ))
    .join(format!("{name}.{form}.test.txt"));
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
        .trim()
        .to_string()
}

fn render(d: &Descriptor, form: &str, network: Network) -> String {
    let desc = match form {
        "multipath" => md_codec::to_miniscript_descriptor_multipath_with_network(d, network),
        "c0" => md_codec::to_miniscript_descriptor_with_network(d, 0, network),
        "c1" => md_codec::to_miniscript_descriptor_with_network(d, 1, network),
        _ => unreachable!(),
    };
    desc.unwrap_or_else(|e| panic!("{form} {network}: {e}"))
        .to_string()
}

/// The vector: all three test networks, all three spellings, byte-equal to
/// the independently derived golden. Mutation: drop the network argument in
/// `assemble_origin_and_xkey` (back to `NetworkKind::Main`) -> every leaf
/// renders `xpub` again and all four vectors go red.
#[test]
fn test_networks_render_every_key_as_tpub_matching_the_vector() {
    for name in VECTORS {
        let d = decode_vendored(&load_vendored_phrase(name))
            .unwrap_or_else(|e| panic!("{name}: decode: {e}"));
        for form in ["multipath", "c0", "c1"] {
            let want = golden(name, form);
            for net in TEST_NETWORKS {
                assert_eq!(render(&d, form, net), want, "{name} {form} {net}");
            }
        }
    }
}

/// The property over the WHOLE corpus: every extended key in every
/// renderable keyed vector carries the requested network's kind — test
/// networks `Test`, mainnet `Main` — through both entry points. A vector the
/// renderer refuses must be refused identically on every network (the
/// network may change version bytes, never whether a descriptor renders).
#[test]
fn every_rendered_key_carries_the_requested_network_across_the_corpus() {
    let mut checked = 0;
    let mut mainnet_vs_network_less = 0;
    for name in all_vendored_vector_names() {
        let Ok(d) = decode_vendored(&load_vendored_phrase(&name)) else {
            continue;
        };
        if !d.is_wallet_policy() {
            continue;
        }
        for (label, main) in [
            (
                "multipath",
                md_codec::to_miniscript_descriptor_multipath_with_network(&d, Network::Bitcoin),
            ),
            (
                "c0",
                md_codec::to_miniscript_descriptor_with_network(&d, 0, Network::Bitcoin),
            ),
        ] {
            for net in [
                Network::Bitcoin,
                Network::Testnet,
                Network::Signet,
                Network::Regtest,
            ] {
                let got = match label {
                    "multipath" => {
                        md_codec::to_miniscript_descriptor_multipath_with_network(&d, net)
                    }
                    _ => md_codec::to_miniscript_descriptor_with_network(&d, 0, net),
                };
                let (desc, main) = match (got, &main) {
                    (Ok(g), Ok(m)) => (g, m),
                    (Err(g), Err(m)) => {
                        assert_eq!(g.to_string(), m.to_string(), "{name} {label} {net}");
                        continue;
                    }
                    (g, m) => panic!("{name} {label} {net}: {g:?} vs mainnet {m:?}"),
                };
                let want = NetworkKind::from(net);
                let mut n_keys = 0;
                desc.for_each_key(|k| {
                    let kind = match k {
                        DescriptorPublicKey::XPub(x) => Some(x.xkey.network),
                        DescriptorPublicKey::MultiXPub(x) => Some(x.xkey.network),
                        DescriptorPublicKey::Single(_) => None,
                    };
                    if let Some(kind) = kind {
                        assert_eq!(kind, want, "{name} {label} {net}: {k}");
                        n_keys += 1;
                    }
                    true
                });
                assert!(n_keys > 0, "{name}: no extended key checked");
                // Only the version bytes may differ from the mainnet render.
                let strip = |s: String| {
                    s.split('#')
                        .next()
                        .unwrap()
                        .replace("tpub", "?pub")
                        .replace("xpub", "?pub")
                };
                if net == Network::Bitcoin {
                    // Mainnet through `_with_network` is what the network-less
                    // entry point renders, wherever that one renders at all
                    // (it refuses kind 1, which needs a network).
                    let nl = match label {
                        "multipath" => md_codec::to_miniscript_descriptor_multipath(&d),
                        _ => md_codec::to_miniscript_descriptor(&d, 0),
                    };
                    if let Ok(nl) = nl {
                        assert_eq!(desc.to_string(), nl.to_string(), "{name} {label}");
                        mainnet_vs_network_less += 1;
                    }
                } else {
                    // Same shape and origins, base58 differs past the prefix.
                    assert_eq!(
                        strip(desc.to_string()).len(),
                        strip(main.to_string()).len(),
                        "{name} {label} {net}"
                    );
                }
                checked += 1;
            }
        }
    }
    // 48 keyed vectors x 2 entry points x 4 networks, all rendering (none
    // refused). Exact, so a vector that silently stops rendering is red.
    assert_eq!(checked, 48 * 2 * 4, "renders checked");
    // 96 mainnet renders minus the 4 kind-1 ones the network-less path refuses.
    assert_eq!(mainnet_vs_network_less, 92, "mainnet renders compared");
}
