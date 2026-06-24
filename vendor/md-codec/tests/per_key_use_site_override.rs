//! Funds-safety tests for faithful per-cosigner use-site path overrides
//! (the non-taproot leg of `restore-md1-per-key-use-site-and-hardened-wildcard`).
//!
//! An md1 multisig card can carry per-`@N` use-site path overrides — a
//! cosigner whose derivation suffix diverges from the shared baseline,
//! e.g. `wsh(multi(2, @0/<0;1>/*, @1/<2;3>/*))`. Two bugs (pre-fix) silently
//! applied the *baseline* suffix to every key:
//!
//! 1. `to_miniscript_descriptor` passed `&d.use_site_path` (baseline) to
//!    every key instead of the per-`@N` `ExpandedKey.use_site_path` →
//!    `derive_address` returned WRONG addresses for the diverging cosigner.
//! 2. The descriptor STRING collapsed every key onto chain-0.
//!
//! These tests are the make-or-break funds-safety gate. The divergent
//! address goldens are computed **OUTSIDE md-codec** — an offline BIP-32
//! derivation via rust-bitcoin `Xpub::derive_pub` at the diverging
//! cosigner's OWN alt (`<2;3>/0` ⇒ child `[2, 0]`), then the multisig
//! `witnessScript` / address is assembled by hand with rust-bitcoin
//! primitives. A test that compared md-codec against md-codec would pass
//! vacuously even with the bug present, so we never do that here.
//!
//! Feature-gated behind `derive` (default-on).

#![cfg(feature = "derive")]

use bitcoin::Network;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};
use md_codec::{Descriptor, OriginPath, PathComponent, PathDecl, PathDeclPaths, Tag, TlvSection};
use std::str::FromStr;

const ABANDON_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// 65-byte `(chain_code || compressed_pubkey)` Pubkeys-TLV payload for the
/// abandon-mnemonic account xpub at `path_str`. Mirrors
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

/// Reconstruct the in-memory placeholder `Xpub` md-codec uses (only
/// chain_code + public_key participate in CKDpub; metadata is placeholder).
fn xpub_from_bytes(bytes: &[u8; 65]) -> Xpub {
    let mut chain_code = [0u8; 32];
    chain_code.copy_from_slice(&bytes[..32]);
    let public_key = bitcoin::secp256k1::PublicKey::from_slice(&bytes[32..]).unwrap();
    Xpub {
        network: bitcoin::NetworkKind::Main,
        depth: 0,
        parent_fingerprint: Default::default(),
        child_number: ChildNumber::Normal { index: 0 },
        public_key,
        chain_code: bitcoin::bip32::ChainCode::from(chain_code),
    }
}

/// Independent BIP-32 leaf-pubkey derivation: `xpub / first / second`,
/// all non-hardened. No md-codec helpers — pure rust-bitcoin.
fn leaf_pubkey(bytes: &[u8; 65], first: u32, second: u32) -> bitcoin::secp256k1::PublicKey {
    let secp = Secp256k1::verification_only();
    let xpub = xpub_from_bytes(bytes);
    xpub.derive_pub(
        &secp,
        &[
            ChildNumber::Normal { index: first },
            ChildNumber::Normal { index: second },
        ],
    )
    .unwrap()
    .public_key
}

fn origin(components: &[(bool, u32)]) -> OriginPath {
    OriginPath {
        components: components
            .iter()
            .map(|&(hardened, value)| PathComponent { hardened, value })
            .collect(),
    }
}

fn alt(value: u32) -> Alternative {
    Alternative {
        hardened: false,
        value,
    }
}

fn alt_h(value: u32) -> Alternative {
    Alternative {
        hardened: true,
        value,
    }
}

/// `<a;b>/*` use-site path (non-hardened wildcard).
fn mp(a: u32, b: u32) -> UseSitePath {
    UseSitePath {
        multipath: Some(vec![alt(a), alt(b)]),
        wildcard_hardened: false,
    }
}

/// Build `wsh(multi(2, @0/<base>/*, @1/<over>/*))` with the abandon-mnemonic
/// xpubs at two distinct account paths and a per-`@N` use-site override on
/// `@1` (when `over != base`). `tree_tag`/wrapper kept simple (wsh+multi).
fn divergent_wsh_multi(base: UseSitePath, over_at1: Option<UseSitePath>) -> Descriptor {
    let xpub0 = account_xpub_bytes("m/48'/0'/0'/2'");
    let xpub1 = account_xpub_bytes("m/48'/0'/1'/2'");
    let mut tlv = TlvSection::new_empty();
    tlv.pubkeys = Some(vec![(0u8, xpub0), (1u8, xpub1)]);
    if let Some(over) = over_at1 {
        tlv.use_site_path_overrides = Some(vec![(1u8, over)]);
    }
    Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Divergent(vec![
                origin(&[(true, 48), (true, 0), (true, 0), (true, 2)]),
                origin(&[(true, 48), (true, 0), (true, 1), (true, 2)]),
            ]),
        },
        use_site_path: base,
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
    }
}

// ─── P1.1 — has_hardened_use_site truth table ────────────────────────────

/// Truth table for the shared hardened-anywhere predicate (Point B):
/// baseline `/*h`, override `/*h`, override-hardened ALT (baseline clean),
/// and all-unhardened → expect `true, true, true, false`.
#[test]
fn has_hardened_use_site_truth_table() {
    // (1) baseline hardened wildcard.
    let baseline_hardened = {
        let mut d = divergent_wsh_multi(UseSitePath::standard_multipath(), None);
        d.use_site_path = UseSitePath {
            multipath: Some(vec![alt(0), alt(1)]),
            wildcard_hardened: true,
        };
        d
    };
    assert!(
        md_codec::to_miniscript::has_hardened_use_site(&baseline_hardened),
        "baseline /*h must be hardened-anywhere"
    );

    // (2) override hardened wildcard (baseline clean).
    let override_hardened_wildcard = divergent_wsh_multi(
        UseSitePath::standard_multipath(),
        Some(UseSitePath {
            multipath: Some(vec![alt(2), alt(3)]),
            wildcard_hardened: true,
        }),
    );
    assert!(
        md_codec::to_miniscript::has_hardened_use_site(&override_hardened_wildcard),
        "override /*h must be hardened-anywhere"
    );

    // (3) override with a hardened ALT inside the multipath (baseline clean).
    let override_hardened_alt = divergent_wsh_multi(
        UseSitePath::standard_multipath(),
        Some(UseSitePath {
            multipath: Some(vec![alt_h(2), alt(3)]),
            wildcard_hardened: false,
        }),
    );
    assert!(
        md_codec::to_miniscript::has_hardened_use_site(&override_hardened_alt),
        "override hardened ALT must be hardened-anywhere"
    );

    // (4) all-unhardened (baseline standard, override divergent but clean).
    let all_clean = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(mp(2, 3)));
    assert!(
        !md_codec::to_miniscript::has_hardened_use_site(&all_clean),
        "fully unhardened card must NOT be hardened-anywhere"
    );
}

/// `derive_address` on an override-hardened-ALT card returns a CLEAN
/// `HardenedPublicDerivation` (pre-fix it slipped to a generic
/// `AddressDerivationFailed` because the baseline-only `derive.rs` checks
/// never inspected the override).
#[test]
fn derive_address_override_hardened_alt_clean_reject() {
    let d = divergent_wsh_multi(
        UseSitePath::standard_multipath(),
        Some(UseSitePath {
            multipath: Some(vec![alt_h(2), alt(3)]),
            wildcard_hardened: false,
        }),
    );
    let err = d.derive_address(0, 0, Network::Bitcoin).unwrap_err();
    assert!(
        matches!(err, md_codec::Error::HardenedPublicDerivation),
        "override hardened alt must yield a clean HardenedPublicDerivation, got {err:?}"
    );
}

/// `derive_address` on an override-hardened-WILDCARD card also yields a
/// clean `HardenedPublicDerivation`.
#[test]
fn derive_address_override_hardened_wildcard_clean_reject() {
    let d = divergent_wsh_multi(
        UseSitePath::standard_multipath(),
        Some(UseSitePath {
            multipath: Some(vec![alt(2), alt(3)]),
            wildcard_hardened: true,
        }),
    );
    let err = d.derive_address(0, 0, Network::Bitcoin).unwrap_err();
    assert!(
        matches!(err, md_codec::Error::HardenedPublicDerivation),
        "override /*h must yield a clean HardenedPublicDerivation, got {err:?}"
    );
}

// ─── P1.2 — D1 per-key derivation VALUE (independent golden) ──────────────

/// THE funds-safety divergent test. `wsh(multi(2, @0/<0;1>/*, @1/<2;3>/*))`
/// at chain 0, idx 0:
/// - `@0` derives at its alt[0]=0 → child `[0, 0]`.
/// - `@1` derives at its OWN alt[0]=2 → child `[2, 0]` (NOT `[0, 0]`).
///
/// The expected address is assembled with rust-bitcoin OUTSIDE md-codec:
/// each leaf pubkey via `Xpub::derive_pub`, the `multi(2,...)` witnessScript
/// built by hand (UNSORTED — `Tag::Multi` preserves key order), then the
/// P2WSH address. Pre-fix md-codec collapses `@1` to `[0, 0]` and produces a
/// DIFFERENT (wrong) address — this test fails RED.
#[test]
fn divergent_suffix_address_independent_golden() {
    let xpub0 = account_xpub_bytes("m/48'/0'/0'/2'");
    let xpub1 = account_xpub_bytes("m/48'/0'/1'/2'");

    let d = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(mp(2, 3)));
    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    // INDEPENDENT golden: @0 at [0,0], @1 at [2,0] (its own diverging alt).
    let pk0 = leaf_pubkey(&xpub0, 0, 0);
    let pk1 = leaf_pubkey(&xpub1, 2, 0);
    // multi(2,...) is UNSORTED: keys appear in template order @0, @1.
    let mut b = bitcoin::blockdata::script::Builder::new().push_int(2);
    for p in [&pk0, &pk1] {
        b = b.push_key(&bitcoin::PublicKey::new(*p));
    }
    let script = b
        .push_int(2)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    let expected = bitcoin::Address::p2wsh(&script, Network::Bitcoin).to_string();

    assert_eq!(
        got, expected,
        "divergent @1 must derive at its own <2;3>/0 = [2,0], not the baseline [0,0]"
    );

    // ANTI-VACUITY: prove the baseline-collapse address (the BUG output) is a
    // DIFFERENT address — so a vacuous oracle can't accidentally pass.
    let pk1_wrong = leaf_pubkey(&xpub1, 0, 0);
    let mut bw = bitcoin::blockdata::script::Builder::new().push_int(2);
    for p in [&pk0, &pk1_wrong] {
        bw = bw.push_key(&bitcoin::PublicKey::new(*p));
    }
    let wrong_script = bw
        .push_int(2)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    let wrong = bitcoin::Address::p2wsh(&wrong_script, Network::Bitcoin).to_string();
    assert_ne!(
        expected, wrong,
        "test fixture sanity: divergent and baseline-collapse addresses must differ"
    );
}

/// Chain 1 (change branch): `@0` at alt[1]=1 → `[1, 0]`; `@1` at its own
/// alt[1]=3 → `[3, 0]`. Confirms per-key divergence holds on BOTH chains.
#[test]
fn divergent_suffix_change_chain_independent_golden() {
    let xpub0 = account_xpub_bytes("m/48'/0'/0'/2'");
    let xpub1 = account_xpub_bytes("m/48'/0'/1'/2'");

    let d = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(mp(2, 3)));
    let got = d
        .derive_address(1, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    let pk0 = leaf_pubkey(&xpub0, 1, 0); // baseline alt[1] = 1
    let pk1 = leaf_pubkey(&xpub1, 3, 0); // override alt[1] = 3
    let mut b = bitcoin::blockdata::script::Builder::new().push_int(2);
    for p in [&pk0, &pk1] {
        b = b.push_key(&bitcoin::PublicKey::new(*p));
    }
    let script = b
        .push_int(2)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    let expected = bitcoin::Address::p2wsh(&script, Network::Bitcoin).to_string();
    assert_eq!(
        got, expected,
        "change-chain divergence must also be faithful"
    );
}

/// `Some`/`None` multipath mix: `wsh(multi(2, @0/<0;1>/*, @1/*))`. `@1`'s
/// override has NO multipath, so at any chain `@1` derives at a BARE
/// wildcard (child `[index]`, no chain component) while `@0` derives at
/// `[chain, index]`. Independent golden built by hand.
#[test]
fn some_none_multipath_mix_independent_golden() {
    let xpub0 = account_xpub_bytes("m/48'/0'/0'/2'");
    let xpub1 = account_xpub_bytes("m/48'/0'/1'/2'");

    let bare = UseSitePath {
        multipath: None,
        wildcard_hardened: false,
    };
    let d = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(bare));

    let got = d
        .derive_address(0, 0, Network::Bitcoin)
        .unwrap()
        .assume_checked()
        .to_string();

    // @0 at [0, 0]; @1 (bare /*) at [0] (no chain component).
    let pk0 = leaf_pubkey(&xpub0, 0, 0);
    let secp = Secp256k1::verification_only();
    let pk1 = xpub_from_bytes(&xpub1)
        .derive_pub(&secp, &[ChildNumber::Normal { index: 0 }])
        .unwrap()
        .public_key;
    let mut b = bitcoin::blockdata::script::Builder::new().push_int(2);
    for p in [&pk0, &pk1] {
        b = b.push_key(&bitcoin::PublicKey::new(*p));
    }
    let script = b
        .push_int(2)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    let expected = bitcoin::Address::p2wsh(&script, Network::Bitcoin).to_string();
    assert_eq!(
        got, expected,
        "None-override key must derive at a bare /* (single child), not [chain, index]"
    );
}

// ─── P1.3 — C2: to_miniscript_descriptor_multipath (descriptor STRING) ────

/// The multipath builder renders per-`@N` GROUPS: `@0` carries `<0;1>` and
/// `@1` carries its divergent `<2;3>` — NOT a chain-0 collapse.
#[test]
fn multipath_builder_renders_per_at_n_groups() {
    let d = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(mp(2, 3)));
    let desc = md_codec::to_miniscript::to_miniscript_descriptor_multipath(&d)
        .expect("multipath builder must succeed for a non-hardened divergent card");
    let s = desc.to_string();
    assert!(
        s.contains("<0;1>/*"),
        "@0 must render its baseline <0;1> group: {s}"
    );
    assert!(
        s.contains("<2;3>/*"),
        "@1 must render its divergent <2;3> group: {s}"
    );
}

/// The multipath descriptor derives the SAME addresses as `derive_address`
/// (rust-miniscript `into_single_descriptors` selects each key's own alt) —
/// cross-checked against the independent golden.
#[test]
fn multipath_builder_address_equivalence() {
    let xpub0 = account_xpub_bytes("m/48'/0'/0'/2'");
    let xpub1 = account_xpub_bytes("m/48'/0'/1'/2'");
    let d = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(mp(2, 3)));

    let desc = md_codec::to_miniscript::to_miniscript_descriptor_multipath(&d).unwrap();
    // into_single_descriptors yields [chain0, chain1]; take chain0 idx0.
    let singles = desc.into_single_descriptors().unwrap();
    let chain0 = &singles[0];
    let got = chain0
        .at_derivation_index(0)
        .unwrap()
        .address(Network::Bitcoin)
        .unwrap()
        .to_string();

    let pk0 = leaf_pubkey(&xpub0, 0, 0);
    let pk1 = leaf_pubkey(&xpub1, 2, 0);
    let mut b = bitcoin::blockdata::script::Builder::new().push_int(2);
    for p in [&pk0, &pk1] {
        b = b.push_key(&bitcoin::PublicKey::new(*p));
    }
    let script = b
        .push_int(2)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    let expected = bitcoin::Address::p2wsh(&script, Network::Bitcoin).to_string();
    assert_eq!(
        got, expected,
        "multipath builder chain0 idx0 = independent golden"
    );
}

/// `Some`/`None` mix in the STRING: `@0` renders `<0;1>` (multipath key);
/// `@1` renders a bare `/*` single-path key (no `<...>` group).
#[test]
fn multipath_builder_some_none_mix_string() {
    let bare = UseSitePath {
        multipath: None,
        wildcard_hardened: false,
    };
    let d = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(bare));
    let desc = md_codec::to_miniscript::to_miniscript_descriptor_multipath(&d).unwrap();
    let s = desc.to_string();
    assert!(s.contains("<0;1>/*"), "@0 stays multipath <0;1>: {s}");
    // @1 must be a single-path key: its rendered key fragment ends in a bare
    // `/*` with NO `<...>` group. Check there is exactly one `<...>` group.
    assert_eq!(
        s.matches('<').count(),
        1,
        "exactly one multipath group (only @0); @1 is single-path: {s}"
    );
}

/// sortedmulti divergent: keys sort per-INDEX at derivation (rust-miniscript
/// owns this via `into_single_descriptors`). The multipath builder must keep
/// `wsh(sortedmulti(...))` and let each chain sort independently. We assert
/// the address equals an independent golden where the two leaf pubkeys are
/// sorted by their serialized bytes.
#[test]
fn multipath_builder_sortedmulti_divergent_independent_golden() {
    let xpub0 = account_xpub_bytes("m/48'/0'/0'/2'");
    let xpub1 = account_xpub_bytes("m/48'/0'/1'/2'");

    // Build a sortedmulti variant of the divergent card.
    let mut d = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(mp(2, 3)));
    d.tree = Node {
        tag: Tag::Wsh,
        body: Body::Children(vec![Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1],
            },
        }]),
    };

    let desc = md_codec::to_miniscript::to_miniscript_descriptor_multipath(&d).unwrap();
    let singles = desc.into_single_descriptors().unwrap();
    let got = singles[0]
        .at_derivation_index(0)
        .unwrap()
        .address(Network::Bitcoin)
        .unwrap()
        .to_string();

    // Independent golden: @0 at [0,0], @1 at [2,0]; SORT the pubkeys.
    let pk0 = leaf_pubkey(&xpub0, 0, 0);
    let pk1 = leaf_pubkey(&xpub1, 2, 0);
    let mut pks = [pk0, pk1];
    pks.sort_by_key(|p| p.serialize());
    let mut b = bitcoin::blockdata::script::Builder::new().push_int(2);
    for p in &pks {
        b = b.push_key(&bitcoin::PublicKey::new(*p));
    }
    let script = b
        .push_int(2)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKMULTISIG)
        .into_script();
    let expected = bitcoin::Address::p2wsh(&script, Network::Bitcoin).to_string();
    assert_eq!(
        got, expected,
        "sortedmulti divergent must sort the per-key-derived pubkeys per index"
    );
}

// ─── P1.4 — D5(a)/(b): decode hardening ───────────────────────────────────

/// D5(a): a card whose `use_site_path_overrides` carries an `@0` entry (the
/// baseline cannot be overridden) is rejected at decode with
/// `BaselineUseSiteOverride { idx: 0 }`. Hand-crafted via encode→decode
/// (our encoders never emit an `@0` override, so this is adversarial wire).
#[test]
fn decode_rejects_baseline_at0_override() {
    let mut d = divergent_wsh_multi(UseSitePath::standard_multipath(), None);
    // Hand-craft an @0 override (distinct from baseline so it isn't ALSO
    // caught as redundant — we want the @0 reject specifically).
    d.tlv.use_site_path_overrides = Some(vec![(0u8, mp(2, 3))]);

    let (bytes, total_bits) = md_codec::encode_payload(&d).expect("encode adversarial @0 override");
    let err = md_codec::decode_payload(&bytes, total_bits)
        .expect_err("decode must reject an @0 use-site override");
    assert!(
        matches!(err, md_codec::Error::BaselineUseSiteOverride { idx: 0 }),
        "expected BaselineUseSiteOverride{{idx:0}}, got {err:?}"
    );
}

/// D5(a): a card whose `@1` override EQUALS the resolved baseline is
/// redundant/non-canonical and is rejected with
/// `RedundantUseSiteOverride { idx: 1 }`.
#[test]
fn decode_rejects_redundant_override_equal_to_baseline() {
    // Baseline standard <0;1>; override @1 == standard <0;1> (redundant).
    let mut d = divergent_wsh_multi(UseSitePath::standard_multipath(), None);
    d.tlv.use_site_path_overrides = Some(vec![(1u8, UseSitePath::standard_multipath())]);

    let (bytes, total_bits) =
        md_codec::encode_payload(&d).expect("encode adversarial redundant override");
    let err = md_codec::decode_payload(&bytes, total_bits)
        .expect_err("decode must reject a redundant use-site override");
    assert!(
        matches!(err, md_codec::Error::RedundantUseSiteOverride { idx: 1 }),
        "expected RedundantUseSiteOverride{{idx:1}}, got {err:?}"
    );
}

/// D5(a) negative control: a GENUINELY divergent `@1` override survives
/// decode (the rejects fire only on @0 / redundant, never on a real
/// divergent card).
#[test]
fn decode_accepts_genuine_divergent_override() {
    let d = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(mp(2, 3)));
    let (bytes, total_bits) = md_codec::encode_payload(&d).expect("encode divergent card");
    let decoded = md_codec::decode_payload(&bytes, total_bits)
        .expect("a genuine divergent override must decode cleanly");
    let overrides = decoded
        .tlv
        .use_site_path_overrides
        .expect("override must survive round-trip");
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides[0].0, 1, "override keyed on @1");
}

/// D5(b): a `Some`-baseline + `None`-override mix is a LEGAL divergent
/// STRUCTURE and must decode without a `MultipathAltCountMismatch` (the
/// None override is skipped by the alt-count check; it is the C2 faithful
/// `XPub` case, not a reject).
#[test]
fn decode_accepts_some_baseline_none_override_mix() {
    let bare = UseSitePath {
        multipath: None,
        wildcard_hardened: false,
    };
    let d = divergent_wsh_multi(UseSitePath::standard_multipath(), Some(bare));
    let (bytes, total_bits) = md_codec::encode_payload(&d).expect("encode Some/None mix");
    md_codec::decode_payload(&bytes, total_bits)
        .expect("Some-baseline + None-override is a legal divergent structure");
}
