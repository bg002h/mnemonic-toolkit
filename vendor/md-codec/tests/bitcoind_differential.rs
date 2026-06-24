//! Stress Cycle E — Bitcoin Core (`bitcoind`) address differential.
//!
//! Bitcoin Core is an INDEPENDENT C++ implementation of address
//! derivation; cross-checking md-codec's `Descriptor::derive_address`
//! against `bitcoind deriveaddresses` catches the class of funds-critical
//! bug that a same-ecosystem oracle (rust-miniscript, which md-codec's
//! converter delegates to) cannot. This is the highest-assurance
//! differential in the stress program: external ground truth for the
//! funds-critical output (the address a user sends coins to).
//!
//! **Wiring contract — CONNECT-ONLY (the test NEVER spawns bitcoind).**
//! CI (or the local recipe) owns the lifecycle: it starts an offline
//! `-chain=main` node and exports three env vars the test reads —
//! `BITCOINCLI_BIN` (path to the pinned `bitcoin-cli`),
//! `BITCOIND_DATADIR` (so `bitcoin-cli` finds the `.cookie`), and
//! `BITCOIND_RPCPORT`. The test shells
//! `$BITCOINCLI_BIN -chain=main -datadir=$BITCOIND_DATADIR
//! -rpcport=$BITCOIND_RPCPORT <rpc> …` (cookie auth, no credentials).
//!
//! - All three vars UNSET → skip (the standard `#[ignore]` local default).
//! - All three vars SET but `bitcoin-cli getblockchaininfo` fails →
//!   `panic!` (broken provisioning fails RED, never green-by-skip).
//!
//! `#[ignore]`-by-default; run with
//! `cargo test -p md-codec --features derive --test bitcoind_differential
//! -- --ignored --nocapture` after exporting the three vars.
//!
//! Pinned oracle: Bitcoin Core v27.0
//! (sha256 `2a6974c5486f528793c79d42694b5987401e4a43c97f62b1383abf35bcee44a8`).
//! Network: offline `-chain=main` (mainnet) — regtest rejects mainnet
//! xpubs, and md-codec's TLV→xpub path always renders mainnet `xpub…`.

#![cfg(feature = "derive")]

use bitcoin::Network;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};
use md_codec::{Descriptor, OriginPath, PathComponent, PathDecl, PathDeclPaths, Tag, TlvSection};
use std::process::Command;
use std::str::FromStr;

/// How many addresses to derive per (shape, chain): indices 0..=N.
const N: u32 = 4;

/// Anti-vacuity golden (R0 evidence): the published BIP-84 receive
/// address 0 for the abandon-mnemonic. The `wpkh` chain-0 idx-0 shape's
/// md-codec address MUST equal this BEFORE the bitcoind compare — so a
/// silently-wrong bitcoind connection can never make the test vacuously
/// pass.
const WPKH_CHAIN0_IDX0_GOLDEN: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

/// INDEPENDENT anti-vacuity golden for the per-cosigner-override divergent
/// shape `wsh(multi(2, @0/<0;1>/*, @1/<2;3>/*))` at chain 0 / idx 0. `@0`
/// derives at `[0,0]`; the diverging `@1` derives at its OWN `<2;3>/0` =
/// `[2,0]` (NOT the baseline `[0,0]`). Computed OUTSIDE this differential
/// (hand-rolled rust-bitcoin in `per_key_use_site_override.rs`, and
/// independently re-derived by `bitcoind deriveaddresses`) — a vacuous
/// same-render oracle can't make the divergent shape pass without it.
const DIVERGENT_WSH_MULTI_CHAIN0_IDX0_GOLDEN: &str =
    "bc1qja66mak5p34f6fhc3z8lt5at5ndayx5z9h8734z0qc8qr27ly9jskzxxcu";

/// The well-known "abandon abandon … about" mnemonic — used by BIP
/// 44/49/84/86 published test vectors (mfp 73c5da0a). Mirrors
/// `address_derivation.rs`.
const ABANDON_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// ─── Corpus construction (mirrors address_derivation.rs) ────────────────

/// Derive the account-level xpub for the abandon-mnemonic at `path`,
/// returning the 65-byte `(chain_code || compressed_pubkey)` payload as
/// it appears in a v0.13 `Pubkeys` TLV entry. Identical to
/// `address_derivation.rs::account_xpub_bytes`.
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

fn pubkeys(entries: Vec<(u8, [u8; 65])>) -> TlvSection {
    let mut t = TlvSection::new_empty();
    t.pubkeys = Some(entries);
    t
}

/// One corpus entry: a human label + the md-codec `Descriptor`.
struct Shape {
    label: &'static str,
    desc: Descriptor,
}

/// The 10 R0-PROVEN corpus shapes (md-codec-derivable ∩ bitcoind-sane).
/// Each built the SAME way `address_derivation.rs` builds its
/// descriptors (same abandon-mnemonic xpub vectors, same TLV
/// construction).
fn corpus() -> Vec<Shape> {
    vec![
        // 1. single-sig pkh — BIP-44.
        Shape {
            label: "pkh (BIP-44)",
            desc: Descriptor {
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
                tlv: pubkeys(vec![(0u8, account_xpub_bytes("m/44'/0'/0'"))]),
            },
        },
        // 2. single-sig sh(wpkh) — BIP-49 nested segwit.
        Shape {
            label: "sh(wpkh) (BIP-49)",
            desc: Descriptor {
                n: 1,
                path_decl: PathDecl {
                    n: 1,
                    paths: PathDeclPaths::Shared(origin(&[(true, 49), (true, 0), (true, 0)])),
                },
                use_site_path: UseSitePath::standard_multipath(),
                tree: Node {
                    tag: Tag::Sh,
                    body: Body::Children(vec![Node {
                        tag: Tag::Wpkh,
                        body: Body::KeyArg { index: 0 },
                    }]),
                },
                tlv: pubkeys(vec![(0u8, account_xpub_bytes("m/49'/0'/0'"))]),
            },
        },
        // 3. single-sig wpkh — BIP-84.
        Shape {
            label: "wpkh (BIP-84)",
            desc: Descriptor {
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
                tlv: pubkeys(vec![(0u8, account_xpub_bytes("m/84'/0'/0'"))]),
            },
        },
        // 4. single-sig tr keypath — BIP-86.
        Shape {
            label: "tr keypath (BIP-86)",
            desc: Descriptor {
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
                tlv: pubkeys(vec![(0u8, account_xpub_bytes("m/86'/0'/0'"))]),
            },
        },
        // 5. multisig wsh(sortedmulti(2,…)) — BIP-48 type 2.
        {
            let a = account_xpub_bytes("m/48'/0'/0'/2'");
            let b = account_xpub_bytes("m/48'/0'/1'/2'");
            let c = account_xpub_bytes("m/48'/0'/2'/2'");
            Shape {
                label: "wsh(sortedmulti 2-of-3)",
                desc: Descriptor {
                    n: 3,
                    path_decl: PathDecl {
                        n: 3,
                        paths: PathDeclPaths::Shared(origin(&[
                            (true, 48),
                            (true, 0),
                            (true, 0),
                            (true, 2),
                        ])),
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
                    tlv: pubkeys(vec![(0u8, a), (1u8, b), (2u8, c)]),
                },
            }
        },
        // 6. multisig sh(wsh(sortedmulti(2,…))) — BIP-48 type 1.
        {
            let a = account_xpub_bytes("m/48'/0'/0'/1'");
            let b = account_xpub_bytes("m/48'/0'/1'/1'");
            let c = account_xpub_bytes("m/48'/0'/2'/1'");
            Shape {
                label: "sh(wsh(sortedmulti 2-of-3))",
                desc: Descriptor {
                    n: 3,
                    path_decl: PathDecl {
                        n: 3,
                        paths: PathDeclPaths::Shared(origin(&[
                            (true, 48),
                            (true, 0),
                            (true, 0),
                            (true, 1),
                        ])),
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
                    tlv: pubkeys(vec![(0u8, a), (1u8, b), (2u8, c)]),
                },
            }
        },
        // 7. taproot tr(NUMS, multi_a(2,…)) — script-path-only multisig.
        {
            let b = account_xpub_bytes("m/86'/0'/1'");
            let c = account_xpub_bytes("m/86'/0'/2'");
            let d = account_xpub_bytes("m/86'/0'/3'");
            Shape {
                label: "tr(NUMS, multi_a 2-of-3)",
                desc: Descriptor {
                    n: 3,
                    path_decl: PathDecl {
                        n: 3,
                        paths: PathDeclPaths::Divergent(vec![
                            origin(&[(true, 86), (true, 0), (true, 1)]),
                            origin(&[(true, 86), (true, 0), (true, 2)]),
                            origin(&[(true, 86), (true, 0), (true, 3)]),
                        ]),
                    },
                    use_site_path: UseSitePath::standard_multipath(),
                    tree: Node {
                        tag: Tag::Tr,
                        body: Body::Tr {
                            is_nums: true,
                            key_index: 0,
                            tree: Some(Box::new(Node {
                                tag: Tag::MultiA,
                                body: Body::MultiKeys {
                                    k: 2,
                                    indices: vec![0, 1, 2],
                                },
                            })),
                        },
                    },
                    tlv: pubkeys(vec![(0u8, b), (1u8, c), (2u8, d)]),
                },
            }
        },
        // 8. taproot tr(<key>, multi_a(2,…)) — internal key + tap-leaf multisig.
        {
            let a = account_xpub_bytes("m/86'/0'/0'");
            let b = account_xpub_bytes("m/86'/0'/1'");
            let c = account_xpub_bytes("m/86'/0'/2'");
            let d = account_xpub_bytes("m/86'/0'/3'");
            Shape {
                label: "tr(key, multi_a 2-of-3)",
                desc: Descriptor {
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
                    tlv: pubkeys(vec![(0u8, a), (1u8, b), (2u8, c), (3u8, d)]),
                },
            }
        },
        // 9. sane miniscript-in-wsh: wsh(and_v(v:pk, older(144))).
        Shape {
            label: "wsh(and_v(v:pk, older(144)))",
            desc: Descriptor {
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
                tlv: pubkeys(vec![(0u8, account_xpub_bytes("m/84'/0'/0'"))]),
            },
        },
        // 10. sane miniscript-in-wsh: wsh(thresh(2, pk, s:pk, s:pk)).
        {
            let a = account_xpub_bytes("m/48'/0'/0'/2'");
            let b = account_xpub_bytes("m/48'/0'/1'/2'");
            let c = account_xpub_bytes("m/48'/0'/2'/2'");
            Shape {
                label: "wsh(thresh(2, pk, s:pk, s:pk))",
                desc: Descriptor {
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
                    tlv: pubkeys(vec![(0u8, a), (1u8, b), (2u8, c)]),
                },
            }
        },
        // 11. multisig wsh(multi(2,…)) — plain (UNSORTED) legacy-segwit
        //     multisig. Mirrors shape 5 (sortedmulti) with Tag::Multi so the
        //     key ORDER is preserved (not lexicographically sorted): catches
        //     the class of bug where md-codec sorts when it must not.
        {
            let a = account_xpub_bytes("m/48'/0'/0'/2'");
            let b = account_xpub_bytes("m/48'/0'/1'/2'");
            let c = account_xpub_bytes("m/48'/0'/2'/2'");
            Shape {
                label: "wsh(multi 2-of-3, unsorted)",
                desc: Descriptor {
                    n: 3,
                    path_decl: PathDecl {
                        n: 3,
                        paths: PathDeclPaths::Shared(origin(&[
                            (true, 48),
                            (true, 0),
                            (true, 0),
                            (true, 2),
                        ])),
                    },
                    use_site_path: UseSitePath::standard_multipath(),
                    tree: Node {
                        tag: Tag::Wsh,
                        body: Body::Children(vec![Node {
                            tag: Tag::Multi,
                            body: Body::MultiKeys {
                                k: 2,
                                indices: vec![0, 1, 2],
                            },
                        }]),
                    },
                    tlv: pubkeys(vec![(0u8, a), (1u8, b), (2u8, c)]),
                },
            }
        },
        // 12. hashlock wsh(and_v(v:pk, sha256(<h>))) — a SHA256 preimage
        //     lock. Mirrors shape 9 (and_v(v:pk, older)) with the timelock
        //     leg swapped for a Tag::Sha256 hash literal: exercises the
        //     Body::Hash256Body construction + Terminal::Sha256 render.
        Shape {
            label: "wsh(and_v(v:pk, sha256))",
            desc: Descriptor {
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
                                tag: Tag::Sha256,
                                body: Body::Hash256Body([0x42; 32]),
                            },
                        ]),
                    }]),
                },
                tlv: pubkeys(vec![(0u8, account_xpub_bytes("m/84'/0'/0'"))]),
            },
        },
        // 13. absolute-timelock wsh(and_v(v:pk, after(800000))) — a height
        //     CLTV. Mirrors shape 9 with Tag::Older→Tag::After (the OTHER
        //     Body::Timelock tag) at a block height < 500_000_000 so it is
        //     interpreted as a height, not a unix time.
        Shape {
            label: "wsh(and_v(v:pk, after(800000)))",
            desc: Descriptor {
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
                                tag: Tag::After,
                                body: Body::Timelock(800_000),
                            },
                        ]),
                    }]),
                },
                tlv: pubkeys(vec![(0u8, account_xpub_bytes("m/84'/0'/0'"))]),
            },
        },
        // 14. spending-policy wsh(or_d(pk, and_v(v:pk, older(144)))) — the
        //     canonical "primary key OR (backup key after 144 blocks)"
        //     recovery policy. Exercises the Tag::OrD combinator with a
        //     non-trivial right branch over two distinct keys.
        {
            let a = account_xpub_bytes("m/84'/0'/0'");
            let b = account_xpub_bytes("m/84'/0'/1'");
            Shape {
                label: "wsh(or_d(pk, and_v(v:pk, older(144))))",
                desc: Descriptor {
                    n: 2,
                    path_decl: PathDecl {
                        n: 2,
                        paths: PathDeclPaths::Divergent(vec![
                            origin(&[(true, 84), (true, 0), (true, 0)]),
                            origin(&[(true, 84), (true, 0), (true, 1)]),
                        ]),
                    },
                    use_site_path: UseSitePath::standard_multipath(),
                    tree: Node {
                        tag: Tag::Wsh,
                        body: Body::Children(vec![Node {
                            tag: Tag::OrD,
                            body: Body::Children(vec![
                                pkk(0),
                                Node {
                                    tag: Tag::AndV,
                                    body: Body::Children(vec![
                                        Node {
                                            tag: Tag::Verify,
                                            body: Body::Children(vec![pkk(1)]),
                                        },
                                        Node {
                                            tag: Tag::Older,
                                            body: Body::Timelock(144),
                                        },
                                    ]),
                                },
                            ]),
                        }]),
                    },
                    tlv: pubkeys(vec![(0u8, a), (1u8, b)]),
                },
            }
        },
        // 15. branching-policy wsh(andor(pk, older(144), pk)) — "(key0 AND
        //     older(144)) OR key1". Exercises the 3-ary Tag::AndOr combinator
        //     (distinct from the 2-ary or_d above).
        {
            let a = account_xpub_bytes("m/84'/0'/0'");
            let b = account_xpub_bytes("m/84'/0'/1'");
            Shape {
                label: "wsh(andor(pk, older(144), pk))",
                desc: Descriptor {
                    n: 2,
                    path_decl: PathDecl {
                        n: 2,
                        paths: PathDeclPaths::Divergent(vec![
                            origin(&[(true, 84), (true, 0), (true, 0)]),
                            origin(&[(true, 84), (true, 0), (true, 1)]),
                        ]),
                    },
                    use_site_path: UseSitePath::standard_multipath(),
                    tree: Node {
                        tag: Tag::Wsh,
                        body: Body::Children(vec![Node {
                            tag: Tag::AndOr,
                            body: Body::Children(vec![
                                pkk(0),
                                Node {
                                    tag: Tag::Older,
                                    body: Body::Timelock(144),
                                },
                                pkk(1),
                            ]),
                        }]),
                    },
                    tlv: pubkeys(vec![(0u8, a), (1u8, b)]),
                },
            }
        },
        // 16. PER-COSIGNER USE-SITE OVERRIDE (funds-safety): a divergent
        //     `wsh(multi(2, @0/<0;1>/*, @1/<2;3>/*))`. `@1` overrides the
        //     shared `<0;1>` baseline with `<2;3>`, so it MUST derive at its
        //     own alt (chain0 → `/2/*`, chain1 → `/3/*`) — NOT the baseline.
        //     Pre-fix md-codec collapsed every key onto the baseline chain
        //     (silent wrong address). bitcoind (an independent C++ impl)
        //     re-derives from md-codec's own single-chain render AND the
        //     pinned `DIVERGENT_WSH_MULTI_CHAIN0_IDX0_GOLDEN` anchors the
        //     diverging cosigner against an out-of-codec computation.
        {
            let a = account_xpub_bytes("m/48'/0'/0'/2'");
            let b = account_xpub_bytes("m/48'/0'/1'/2'");
            let mut tlv = pubkeys(vec![(0u8, a), (1u8, b)]);
            tlv.use_site_path_overrides = Some(vec![(
                1u8,
                UseSitePath {
                    multipath: Some(vec![
                        Alternative {
                            hardened: false,
                            value: 2,
                        },
                        Alternative {
                            hardened: false,
                            value: 3,
                        },
                    ]),
                    wildcard_hardened: false,
                },
            )]);
            Shape {
                label: "wsh(multi 2-of-2, @1 use-site override <2;3>)",
                desc: Descriptor {
                    n: 2,
                    path_decl: PathDecl {
                        n: 2,
                        paths: PathDeclPaths::Divergent(vec![
                            origin(&[(true, 48), (true, 0), (true, 0), (true, 2)]),
                            origin(&[(true, 48), (true, 0), (true, 1), (true, 2)]),
                        ]),
                    },
                    use_site_path: UseSitePath::standard_multipath(),
                    tree: Node {
                        tag: Tag::Wsh,
                        body: Body::Children(vec![Node {
                            tag: Tag::Multi,
                            body: Body::MultiKeys {
                                k: 2,
                                indices: vec![0, 1],
                            },
                        }]),
                    },
                    tlv,
                },
            }
        },
    ]
}

// ─── bitcoind connection (connect-only cookie client) ───────────────────

/// The three wiring env vars, read once.
struct Wiring {
    cli_bin: String,
    datadir: String,
    rpcport: String,
}

/// Read the three wiring env vars. Returns:
/// - `None` if NONE are set (skip — the `#[ignore]` local default).
/// - `Some(Wiring)` if ALL three are set.
/// - `panic!` if they are partially set (an ambiguous broken provision).
fn read_wiring() -> Option<Wiring> {
    let cli_bin = std::env::var("BITCOINCLI_BIN").ok();
    let datadir = std::env::var("BITCOIND_DATADIR").ok();
    let rpcport = std::env::var("BITCOIND_RPCPORT").ok();
    match (cli_bin, datadir, rpcport) {
        (None, None, None) => None,
        (Some(cli_bin), Some(datadir), Some(rpcport)) => Some(Wiring {
            cli_bin,
            datadir,
            rpcport,
        }),
        (cli_bin, datadir, rpcport) => panic!(
            "bitcoind wiring partially set — all three of BITCOINCLI_BIN/\
             BITCOIND_DATADIR/BITCOIND_RPCPORT must be set together \
             (BITCOINCLI_BIN={cli_bin:?}, BITCOIND_DATADIR={datadir:?}, \
             BITCOIND_RPCPORT={rpcport:?})"
        ),
    }
}

/// Shell `$BITCOINCLI_BIN -chain=main -datadir=… -rpcport=… <args>`
/// (cookie auth) and return parsed JSON. `panic!`s on a process failure
/// or an RPC error (a harness/corpus bug — never a silent skip).
fn bitcoin_cli(w: &Wiring, args: &[&str]) -> serde_json::Value {
    let out = Command::new(&w.cli_bin)
        .arg("-chain=main")
        .arg(format!("-datadir={}", w.datadir))
        .arg(format!("-rpcport={}", w.rpcport))
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn bitcoin-cli ({}): {e}", w.cli_bin));
    if !out.status.success() {
        panic!(
            "bitcoin-cli {:?} failed (status {}): stderr={}",
            args,
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("bitcoin-cli {args:?} output not JSON ({e}): {stdout}"))
}

// ─── The differential ────────────────────────────────────────────────────

/// For the 10 R0-proven corpus shapes × 2 chains × indices 0..=N, derive
/// each address via md-codec AND via a PRE-RUNNING offline `-chain=main`
/// bitcoind v27.0, and assert byte-equality. Plus the per-shape checksum
/// round-trip self-test and a pinned anti-vacuity golden.
///
/// `#[ignore]`-by-default: requires the three wiring env vars + a running
/// node. See the module docs for the run command.
#[test]
#[ignore = "requires a pre-running offline -chain=main bitcoind (wiring env vars)"]
fn bitcoind_address_differential() {
    let Some(w) = read_wiring() else {
        eprintln!(
            "skipping: bitcoind env not set (BITCOINCLI_BIN/BITCOIND_DATADIR/BITCOIND_RPCPORT)"
        );
        return;
    };

    // Fail-LOUD if set-but-silent: the vars are set but the node doesn't
    // answer → broken provisioning fails RED, never green-by-skip.
    let info = bitcoin_cli(&w, &["getblockchaininfo"]);
    assert_eq!(
        info.get("chain").and_then(|c| c.as_str()),
        Some("main"),
        "bitcoind must be on -chain=main (got {info:?})"
    );

    let mut total_checks = 0usize;
    let mut golden_asserted = false;
    let mut divergent_golden_asserted = false;

    for shape in corpus() {
        for chain in 0u32..=1 {
            // md-codec's OWN rendered single-chain descriptor (per-chain
            // /0/* or /1/*, checksummed) — bitcoind's input is derived
            // from exactly the string md-codec derives from. NEVER the
            // <0;1> multipath form (bitcoind rejects it).
            let desc = md_codec::to_miniscript::to_miniscript_descriptor(&shape.desc, chain)
                .unwrap_or_else(|e| {
                    panic!(
                        "md-codec failed to render {} chain {chain}: {e}",
                        shape.label
                    )
                })
                .to_string();

            // [I3a] Checksum round-trip: bitcoind's computed checksum MUST
            // equal the #csum md-codec/miniscript already put in `desc`.
            // Catches canonicalization drift before deriveaddresses.
            let md_csum = desc
                .rsplit_once('#')
                .unwrap_or_else(|| {
                    panic!("{} chain {chain}: desc has no #csum: {desc}", shape.label)
                })
                .1;
            let dinfo = bitcoin_cli(&w, &["getdescriptorinfo", &desc]);
            let bitcoind_csum = dinfo
                .get("checksum")
                .and_then(|c| c.as_str())
                .unwrap_or_else(|| panic!("getdescriptorinfo had no checksum: {dinfo:?}"));
            assert_eq!(
                bitcoind_csum, md_csum,
                "CHECKSUM DRIFT [{}] chain {chain}: bitcoind={bitcoind_csum} md-codec={md_csum} desc={desc}",
                shape.label
            );

            // bitcoind addresses for indices [0, N]. The range arg is
            // MANDATORY for a ranged (/*) descriptor — without it bitcoind
            // errors -8 (bitcoin_cli would panic loud, never a silent
            // "match"). [E-m1]
            let range = format!("[0,{N}]");
            let arr = bitcoin_cli(&w, &["deriveaddresses", &desc, &range]);
            let bitcoind_addrs: Vec<String> = arr
                .as_array()
                .unwrap_or_else(|| panic!("deriveaddresses did not return an array: {arr:?}"))
                .iter()
                .map(|v| {
                    v.as_str()
                        .unwrap_or_else(|| panic!("deriveaddresses element not a string: {v:?}"))
                        .to_string()
                })
                .collect();
            assert_eq!(
                bitcoind_addrs.len(),
                (N as usize) + 1,
                "[{}] chain {chain}: expected {} addresses, got {}",
                shape.label,
                N + 1,
                bitcoind_addrs.len()
            );

            for index in 0..=N {
                let md_addr = shape
                    .desc
                    .derive_address(chain, index, Network::Bitcoin)
                    .unwrap_or_else(|e| {
                        panic!(
                            "md-codec derive_address [{}] chain {chain} idx {index}: {e}",
                            shape.label
                        )
                    })
                    .assume_checked()
                    .to_string();

                // [I3c] Anti-vacuity golden: wpkh chain0 idx0 must equal
                // the published BIP-84 vector — a silently-wrong bitcoind
                // connection can't make the test vacuously pass.
                if shape.label == "wpkh (BIP-84)" && chain == 0 && index == 0 {
                    assert_eq!(
                        md_addr, WPKH_CHAIN0_IDX0_GOLDEN,
                        "anti-vacuity golden: wpkh chain0 idx0 md-codec address drifted"
                    );
                    golden_asserted = true;
                }

                // [I1] INDEPENDENT divergent golden: the per-cosigner-override
                // shape's @1 derives at its OWN <2;3>/0 = [2,0]. The pinned
                // golden is computed out-of-codec, so it catches a
                // baseline-collapse regression that a same-render oracle would
                // pass vacuously.
                if shape.label == "wsh(multi 2-of-2, @1 use-site override <2;3>)"
                    && chain == 0
                    && index == 0
                {
                    assert_eq!(
                        md_addr, DIVERGENT_WSH_MULTI_CHAIN0_IDX0_GOLDEN,
                        "anti-vacuity divergent golden: @1 must derive at <2;3>/0, not the baseline"
                    );
                    divergent_golden_asserted = true;
                }

                let bitcoind_addr = &bitcoind_addrs[index as usize];
                assert_eq!(
                    &md_addr, bitcoind_addr,
                    "ADDRESS DIVERGENCE (FUNDS-CRITICAL) [{}] chain {chain} idx {index}: \
                     md-codec={md_addr} bitcoind={bitcoind_addr} desc={desc}",
                    shape.label
                );
                total_checks += 1;
            }
        }
    }

    assert!(
        golden_asserted,
        "anti-vacuity golden was never asserted — the wpkh shape is missing from the corpus"
    );
    assert!(
        divergent_golden_asserted,
        "divergent anti-vacuity golden was never asserted — the per-cosigner-override shape is missing from the corpus"
    );
    eprintln!(
        "bitcoind differential PASS: {} shapes × 2 chains × {} indices = {} address checks \
         (+ checksum round-trip per shape×chain), all byte-identical vs bitcoind v27.0",
        corpus().len(),
        N + 1,
        total_checks
    );
}
