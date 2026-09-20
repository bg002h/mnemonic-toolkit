//! Helpers shared by the compose integration tests. Included with
//! `#[path = "compose_support.rs"] mod support;` from `compose_crosscheck.rs`
//! and `compose_vectors.rs`; cargo also compiles this file as a test binary of
//! its own (with no tests), hence the allows: the workspace lints `pub` items
//! for docs, and an unused helper in one includer is dead code in that binary.
#![allow(dead_code, missing_docs)]

use std::str::FromStr;

use md_codec::compose::{
    Composed, HashKind, HashLock, PathList, SlotOrigin, compose, compose_with,
};
use md_codec::encode::Descriptor;
use md_codec::origin_path::{OriginPath, PathComponent};
use md_codec::tag::Tag;
use miniscript::bitcoin::bip32::{ChildNumber, Xpub};
use miniscript::bitcoin::secp256k1::Secp256k1;

/// The wallet-policy journey's four cosigners: one master (73c5da0a) at
/// m/48'/0'/{0..3}'/2'. Real public keys; nothing here is a secret.
pub const XPUB: [&str; 4] = [
    "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf",
    "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk",
    "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR",
    "xpub6E6Z3Ss5TXJYNJp4U1q3NZ3pCn82i7KXQAKUtNnzLJ3cCdchQeSdFvXemizaHUF7wNwRQAB8mPdoZhGHLiv49cWPtCnoJY3Az3E8JKxH9Mq",
];
pub const FP: [u8; 4] = [0x73, 0xc5, 0xda, 0x0a];
pub const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
pub const H: HashLock = HashLock::new(HashKind::Sha256, [0xa8; 32]);
pub const HH: &str = "a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8a8";
/// The hashlock corpus's own anchor digest: sha256 of sha256 of "correct horse
/// battery staple" (mnemonic-secret `hashlock-v0.8.json`, derivation[0].sha256_h).
/// Unlike `H`, a preimage for this one EXISTS and is written down, which is what
/// makes `keyed_compose_wsh_timelock_hashlock` a wallet somebody could actually
/// spend from rather than a shape.
pub const HL_BYTES: [u8; 32] = [
    0xb8, 0x67, 0xdb, 0x87, 0x54, 0x79, 0xbc, 0xc0, 0x28, 0x73, 0x52, 0xcd, 0xaa, 0x4a, 0x17, 0x55,
    0x68, 0x9b, 0x83, 0x38, 0x77, 0x7d, 0x09, 0x15, 0xe9, 0xac, 0xd9, 0xf6, 0xed, 0xbc, 0x96, 0xcb,
];
/// The `hash256` digest of the SAME preimage as the ms-codec KAT's
/// `correct horse battery staple` row, so the md1 vectors and the hashlock
/// corpus agree on one preimage across two repos.
pub const HK256: HashLock = HashLock::new(
    HashKind::Hash256,
    [
        0x98, 0xa2, 0x0f, 0xc2, 0x5d, 0xbc, 0xdf, 0x23, 0x6f, 0xb0, 0x30, 0x7e, 0x3f, 0x82, 0xca,
        0xd4, 0x7f, 0xca, 0x2e, 0x80, 0x7f, 0x3e, 0xf8, 0x2c, 0x31, 0x99, 0x35, 0x49, 0x64, 0x1c,
        0xd4, 0x88,
    ],
);
/// The `ripemd160` digest of the SAME preimage as the ms-codec KAT's
/// `correct horse battery staple` row, so the md1 vectors and the hashlock
/// corpus agree on one preimage across two repos.
pub const HRIPE: HashLock = HashLock::new(
    HashKind::Ripemd160,
    [
        0x09, 0xe7, 0xbb, 0x50, 0x51, 0xd8, 0x97, 0x88, 0xfb, 0x4e, 0x4b, 0x37, 0x41, 0x26, 0x72,
        0x1d, 0xbc, 0xc2, 0x94, 0x6b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ],
);
/// The `hash160` digest of the SAME preimage as the ms-codec KAT's
/// `correct horse battery staple` row, so the md1 vectors and the hashlock
/// corpus agree on one preimage across two repos.
pub const H160K: HashLock = HashLock::new(
    HashKind::Hash160,
    [
        0xb5, 0xb7, 0x2c, 0x0e, 0x68, 0x96, 0xff, 0x59, 0xdf, 0xa9, 0x9e, 0x0d, 0x10, 0x52, 0xa9,
        0xc0, 0x21, 0x4c, 0xd0, 0xbd, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ],
);
pub const HL: HashLock = HashLock::new(HashKind::Sha256, HL_BYTES);
pub const HLH: &str = "b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb";

pub fn hardened(values: &[u32]) -> OriginPath {
    OriginPath {
        components: values
            .iter()
            .map(|v| PathComponent {
                hardened: true,
                value: *v,
            })
            .collect(),
    }
}

/// `n` DISTINCT xpubs: the journey's four for the first four slots, then
/// unhardened children of the first one. BIP-32 makes a (fingerprint, path)
/// pair name exactly ONE key, so binding one xpub at two declared origins would
/// describe an impossible wallet (descriptor-mnemonic F-217,
/// `corpus_origin_consistency.rs`); a 32-slot policy therefore needs 32 keys.
pub fn slot_xpubs(n: usize) -> Vec<Xpub> {
    let secp = Secp256k1::verification_only();
    let base = Xpub::from_str(XPUB[0]).expect("fixture xpub parses");
    (0..n)
        .map(|i| {
            if i < XPUB.len() {
                Xpub::from_str(XPUB[i]).expect("fixture xpub parses")
            } else {
                let child = ChildNumber::from_normal_idx(i as u32).expect("small index");
                base.derive_pub(&secp, &[child])
                    .expect("unhardened derivation")
            }
        })
        .collect()
}

/// 65 wire bytes (chain code ‖ compressed point).
pub fn xpub_bytes(x: &Xpub) -> [u8; 65] {
    let mut out = [0u8; 65];
    out[..32].copy_from_slice(&x.chain_code[..]);
    out[32..].copy_from_slice(&x.public_key.serialize());
    out
}

/// Seat every slot at `m/48'/0'/<slot>'/T'` under one master fingerprint and
/// bind distinct xpub bytes: a KEYED descriptor the converter can derive.
pub fn keyed(list: &PathList) -> Descriptor {
    let unseated = compose(list).expect("list is composable");
    let n = unseated.slots.len();
    let declared: Vec<Option<SlotOrigin>> = (0..n)
        .map(|i| {
            Some(SlotOrigin {
                origin: hardened(&[48, 0, i as u32, list.wrapper.script_type()]),
                fingerprint: Some(FP),
            })
        })
        .collect();
    let mut c = compose_with(list, &declared).expect("declared origins compose");
    let xs = slot_xpubs(n);
    c.descriptor.tlv.pubkeys = Some(
        xs.iter()
            .enumerate()
            .map(|(i, x)| (i as u8, xpub_bytes(x)))
            .collect(),
    );
    c.descriptor
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// The rust-miniscript CONCRETE policy with the same spend conditions as
/// `list`, over `keys` (one key string per emitted slot, in slot order). This is
/// the compiler's input for the §5b lift-equality leg; it is built from the path
/// list, not from the lowered tree, so it cannot inherit a lowering defect.
pub fn concrete_policy(list: &PathList, c: &Composed, keys: &[String]) -> String {
    let mut paths: Vec<String> = Vec::new();
    for (pi, p) in list.paths.iter().enumerate() {
        let mut parts: Vec<String> = Vec::new();
        if let Some(ks) = p.keys {
            let pks: Vec<String> = c
                .slots
                .iter()
                .filter(|slot| slot.path == pi)
                .map(|slot| format!("pk({})", keys[usize::from(slot.index)]))
                .collect();
            parts.push(if ks.n == 1 {
                pks[0].clone()
            } else {
                format!("thresh({},{})", ks.k, pks.join(","))
            });
        }
        if let Some(h) = p.hash {
            // The oracle names the KIND, like the lowering it checks. Spelling
            // `sha256` here would make this helper agree with a kind-blind
            // lowering and disagree with a correct one.
            parts.push(format!("{}({})", h.kind().token(), hex(h.digest())));
        }
        if let Some(lock) = p.lock {
            let (tag, v) = lock.operand().expect("validated");
            let name = if matches!(tag, Tag::Older) {
                "older"
            } else {
                "after"
            };
            parts.push(format!("{name}({v})"));
        }
        let mut acc = parts.pop().expect("a path has a part");
        while let Some(x) = parts.pop() {
            acc = format!("and({x},{acc})");
        }
        paths.push(acc);
    }
    let mut acc = paths.pop().expect("a list has a path");
    while let Some(x) = paths.pop() {
        acc = format!("or({x},{acc})");
    }
    acc
}

use md_codec::compose::presets;
use md_codec::compose::{KeySet, Lock, SpendPath, Wrapper};

pub fn k(k: u8, n: u8) -> SpendPath {
    SpendPath {
        keys: Some(KeySet { k, n, sorted: true }),
        hash: None,
        lock: None,
    }
}
pub fn u(k: u8, n: u8) -> SpendPath {
    SpendPath {
        keys: Some(KeySet {
            k,
            n,
            sorted: false,
        }),
        hash: None,
        lock: None,
    }
}
pub fn lk(mut p: SpendPath, l: Lock) -> SpendPath {
    p.lock = Some(l);
    p
}
pub fn hs(mut p: SpendPath, h: HashLock) -> SpendPath {
    p.hash = Some(h);
    p
}
pub fn kl(h: HashLock, l: Option<Lock>) -> SpendPath {
    SpendPath {
        keys: None,
        hash: Some(h),
        lock: l,
    }
}
pub fn pl(w: Wrapper, paths: Vec<SpendPath>) -> PathList {
    PathList { wrapper: w, paths }
}

/// (vector name, path list, rendered text WITHOUT origins, tags). The text is
/// the fixed spelling the Go builder reproduces; origins are added by
/// `template_with_origins` for the MANIFEST form. Tags are the spec rows:
/// `w:<wrapper>`, `paths:<n>`, `head:<bare-multi|single|locked>`,
/// `ik:<extracted-first|extracted-later|nums|none>`,
/// `lock:<none|blocks|units|height|time>`, `hash`, `sorted`, `unsorted`,
/// `keyless-wsh`, `spine:<m>`, `slots:32`; the §4f default-origin tag
/// `origins:default-<wrapper>` (every family vector is unseated, so every one
/// carries the wrapper's default origins); and the MANIFEST binding's
/// fingerprint case, the spec's three: `fp:distinct` (four distinct declared
/// fingerprints), `fp:one-seed-one-path` (one master fingerprint on two or
/// more slots of ONE path), `fp:one-seed-two-paths` (one master fingerprint
/// across two or more paths); `fp:none` marks the unkeyed vectors. `no-corpus`
/// marks an entry pinned by these tests and the §5b cross-check but NOT stored
/// in `MANIFEST`: the exporter and the corpus tests parse under the MINTING
/// disposition, which after Task 8 refuses a signature-free path unless
/// `--experimental`, so the two keyless-wsh vectors cannot be exported. Stage 2
/// mirrors them from this list directly.
pub fn family() -> Vec<(&'static str, PathList, String, Vec<&'static str>)> {
    let tr32: Vec<SpendPath> = (0..8).map(|_| k(4, 4)).collect();
    vec![
        // ---- wsh family
        ("keyed_compose_wsh_sole_sortedmulti", pl(Wrapper::Wsh, vec![k(2, 3)]),
         "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))".to_string(),
         vec!["w:wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-wsh"]),
        ("keyed_compose_wsh_two_path_or_d", pl(Wrapper::Wsh, vec![k(2, 3), lk(k(1, 1), Lock::OlderBlocks(26280))]),
         "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pkh(@3/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:bare-multi", "lock:blocks", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh"]),
        // Same list as the previous entry; the MANIFEST binds four DISTINCT fingerprints.
        ("keyed_compose_wsh_two_path_distinct_fingerprints", pl(Wrapper::Wsh, vec![k(2, 3), lk(k(1, 1), Lock::OlderBlocks(26280))]),
         "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pkh(@3/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:bare-multi", "lock:blocks", "ik:none", "fp:distinct", "origins:default-wsh"]),
        ("keyed_compose_wsh_single_head_or_i", pl(Wrapper::Wsh, vec![k(1, 1), lk(k(1, 1), Lock::OlderUnits(15188))]),
         "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(4209492))))".to_string(),
         vec!["w:wsh", "paths:2", "head:single", "lock:units", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh"]),
        ("keyed_compose_wsh_locked_head_or_i", pl(Wrapper::Wsh, vec![lk(k(2, 2), Lock::AfterHeight(905_000)), k(1, 1)]),
         "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),after(905000)),pkh(@2/<0;1>/*)))".to_string(),
         vec!["w:wsh", "paths:2", "head:locked", "lock:height", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh"]),
        ("keyed_compose_wsh_hash_and_time", pl(Wrapper::Wsh, vec![k(1, 1), lk(hs(k(2, 2), H), Lock::AfterTime(1_893_456_000))]),
         format!("wsh(or_i(pkh(@0/<0;1>/*),and_v(v:multi(2,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256({HH}),after(1893456000)))))"),
         vec!["w:wsh", "paths:2", "head:single", "lock:time", "hash", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh"]),
        // The same three-path shape with a HASHLOCK on the tail: the policy the
        // SeedHammer II's composer builds, and the one the fork's
        // TestDeviceDerivesTheComposerTimelockHashlockPolicy derives an address
        // from. Its digest is HL, whose preimage exists.
        ("keyed_compose_wsh_timelock_hashlock", pl(Wrapper::Wsh, vec![k(1, 1), lk(k(1, 1), Lock::OlderBlocks(144)), lk(hs(k(1, 1), HL), Lock::AfterHeight(800_000))]),
         format!("wsh(or_i(pkh(@0/<0;1>/*),or_i(and_v(v:pkh(@1/<0;1>/*),older(144)),and_v(v:pkh(@2/<0;1>/*),and_v(v:sha256({HLH}),after(800000))))))"),
         vec!["w:wsh", "paths:3", "head:single", "lock:blocks", "lock:height", "hash", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh"]),
        ("keyed_compose_wsh_three_paths", pl(Wrapper::Wsh, vec![k(1, 1), lk(k(1, 1), Lock::OlderBlocks(4032)), lk(k(1, 1), Lock::AfterHeight(1_000_000))]),
         "wsh(or_i(pkh(@0/<0;1>/*),or_i(and_v(v:pkh(@1/<0;1>/*),older(4032)),and_v(v:pkh(@2/<0;1>/*),after(1000000)))))".to_string(),
         vec!["w:wsh", "paths:3", "head:single", "lock:blocks", "lock:height", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh"]),
        ("keyed_compose_wsh_unsorted_sole", pl(Wrapper::Wsh, vec![u(2, 3)]),
         "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))".to_string(),
         vec!["w:wsh", "paths:1", "head:bare-multi", "lock:none", "unsorted", "ik:none", "fp:one-seed-one-path", "origins:default-wsh"]),
        // ---- legacy wrappers
        ("keyed_compose_sh_wsh_sole", pl(Wrapper::ShWsh, vec![k(2, 3)]),
         "sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*)))".to_string(),
         vec!["w:sh-wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh-wsh"]),
        ("keyed_compose_sh_wsh_one_of_two", pl(Wrapper::ShWsh, vec![k(1, 2)]),
         "sh(wsh(sortedmulti(1,@0/<0;1>/*,@1/<0;1>/*)))".to_string(),
         vec!["w:sh-wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh-wsh"]),
        ("keyed_compose_sh_sole", pl(Wrapper::Sh, vec![k(2, 2)]),
         "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))".to_string(),
         vec!["w:sh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh"]),
        ("keyed_compose_sh_two_of_four", pl(Wrapper::Sh, vec![k(2, 4)]),
         "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*))".to_string(),
         vec!["w:sh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-sh"]),
        // ---- taproot family
        ("keyed_compose_tr_two_path_nums", pl(Wrapper::Tr, vec![k(2, 3), lk(k(1, 1), Lock::OlderBlocks(26280))]),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(26280))}})"),
         vec!["w:tr", "paths:2", "ik:nums", "spine:2", "lock:blocks", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr"]),
        // Same list as the previous entry; the MANIFEST binds four DISTINCT fingerprints.
        ("keyed_compose_tr_two_path_distinct_fingerprints", pl(Wrapper::Tr, vec![k(2, 3), lk(k(1, 1), Lock::OlderBlocks(26280))]),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(26280))}})"),
         vec!["w:tr", "paths:2", "ik:nums", "spine:2", "lock:blocks", "fp:distinct", "origins:default-tr"]),
        ("keyed_compose_tr_extracted_first", pl(Wrapper::Tr, vec![k(1, 1), lk(k(1, 1), Lock::OlderBlocks(65535))]),
         "tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(65535)))".to_string(),
         vec!["w:tr", "paths:2", "ik:extracted-first", "spine:1", "lock:blocks", "fp:one-seed-two-paths", "origins:default-tr"]),
        ("keyed_compose_tr_extracted_later_four_paths", pl(Wrapper::Tr, vec![lk(k(1, 1), Lock::OlderBlocks(10)), lk(k(1, 1), Lock::AfterHeight(1_000_000)), k(1, 1), lk(k(1, 1), Lock::OlderUnits(100))]),
         "tr(@0/<0;1>/*,{and_v(v:pk(@1/<0;1>/*),older(10)),{and_v(v:pk(@2/<0;1>/*),after(1000000)),and_v(v:pk(@3/<0;1>/*),older(4194404))}})".to_string(),
         vec!["w:tr", "paths:4", "ik:extracted-later", "spine:3", "lock:blocks", "lock:height", "lock:units", "fp:one-seed-two-paths", "origins:default-tr"]),
        ("keyed_compose_tr_three_paths_extracted_later", pl(Wrapper::Tr, vec![lk(k(1, 1), Lock::OlderBlocks(10)), k(1, 1), lk(k(1, 1), Lock::OlderUnits(5))]),
         "tr(@0/<0;1>/*,{and_v(v:pk(@1/<0;1>/*),older(10)),and_v(v:pk(@2/<0;1>/*),older(4194309))})".to_string(),
         vec!["w:tr", "paths:3", "ik:extracted-later", "spine:2", "lock:blocks", "lock:units", "fp:one-seed-two-paths", "origins:default-tr"]),
        ("keyed_compose_tr_nums_three_leaves", pl(Wrapper::Tr, vec![lk(k(1, 1), Lock::OlderBlocks(1)), lk(k(1, 1), Lock::OlderBlocks(2)), lk(k(2, 2), Lock::AfterHeight(2))]),
         format!("tr({NUMS},{{and_v(v:pk(@0/<0;1>/*),older(1)),{{and_v(v:pk(@1/<0;1>/*),older(2)),and_v(v:multi_a(2,@2/<0;1>/*,@3/<0;1>/*),after(2))}}}})"),
         vec!["w:tr", "paths:3", "ik:nums", "spine:3", "lock:blocks", "lock:height", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr"]),
        ("keyed_compose_tr_sole_sortedmulti_a", pl(Wrapper::Tr, vec![k(2, 3)]),
         format!("tr({NUMS},sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))"),
         vec!["w:tr", "paths:1", "ik:nums", "spine:1", "lock:none", "sorted", "fp:one-seed-one-path", "origins:default-tr"]),
        ("keyed_compose_tr_key_path_only", pl(Wrapper::Tr, vec![k(1, 1)]),
         "tr(@0/<0;1>/*)".to_string(),
         vec!["w:tr", "paths:1", "ik:extracted-first", "spine:0", "lock:none", "origins:default-tr"]),
        ("keyed_compose_tr_unsorted_sole_leaf", pl(Wrapper::Tr, vec![u(2, 2)]),
         format!("tr({NUMS},multi_a(2,@0/<0;1>/*,@1/<0;1>/*))"),
         vec!["w:tr", "paths:1", "ik:nums", "spine:1", "lock:none", "unsorted", "fp:one-seed-one-path", "origins:default-tr"]),
        ("keyed_compose_tr_hash_leaf", pl(Wrapper::Tr, vec![k(2, 2), lk(hs(k(1, 1), H), Lock::AfterTime(1_893_456_000))]),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:pk(@2/<0;1>/*),and_v(v:sha256({HH}),after(1893456000)))}})"),
         vec!["w:tr", "paths:2", "ik:nums", "spine:2", "hash", "lock:time", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr"]),
        // ---- unkeyed: EXPERIMENTAL shapes and the size boundaries (more slots than the four journey keys)
        ("compose_wsh_keyless_hash_path", pl(Wrapper::Wsh, vec![k(2, 3), kl(H, Some(Lock::AfterHeight(1_383_520)))]),
         format!("wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256({HH}),after(1383520))))"),
         vec!["w:wsh", "paths:2", "head:bare-multi", "keyless-wsh", "hash", "lock:height", "ik:none", "fp:none", "origins:default-wsh", "no-corpus"]),
        ("compose_wsh_keyless_hash_only", pl(Wrapper::Wsh, vec![k(1, 1), kl(H, None)]),
         format!("wsh(or_i(pkh(@0/<0;1>/*),sha256({HH})))"),
         vec!["w:wsh", "paths:2", "head:single", "keyless-wsh", "hash", "lock:none", "ik:none", "fp:none", "origins:default-wsh", "no-corpus"]),
        ("compose_wsh_eight_paths", pl(Wrapper::Wsh, (0..8).map(|i| lk(k(1, 1), Lock::OlderBlocks(100 + i))).collect()),
         "wsh(or_i(and_v(v:pkh(@0/<0;1>/*),older(100)),or_i(and_v(v:pkh(@1/<0;1>/*),older(101)),or_i(and_v(v:pkh(@2/<0;1>/*),older(102)),or_i(and_v(v:pkh(@3/<0;1>/*),older(103)),or_i(and_v(v:pkh(@4/<0;1>/*),older(104)),or_i(and_v(v:pkh(@5/<0;1>/*),older(105)),or_i(and_v(v:pkh(@6/<0;1>/*),older(106)),and_v(v:pkh(@7/<0;1>/*),older(107))))))))))".to_string(),
         vec!["w:wsh", "paths:8", "head:locked", "lock:blocks", "ik:none", "fp:none", "origins:default-wsh"]),
        ("compose_tr_seven_leaves", pl(Wrapper::Tr, (0..8).map(|i| if i == 0 { k(1, 1) } else { lk(k(1, 1), Lock::OlderBlocks(100 + i)) }).collect()),
         "tr(@0/<0;1>/*,{and_v(v:pk(@1/<0;1>/*),older(101)),{and_v(v:pk(@2/<0;1>/*),older(102)),{and_v(v:pk(@3/<0;1>/*),older(103)),{and_v(v:pk(@4/<0;1>/*),older(104)),{and_v(v:pk(@5/<0;1>/*),older(105)),{and_v(v:pk(@6/<0;1>/*),older(106)),and_v(v:pk(@7/<0;1>/*),older(107))}}}}}})".to_string(),
         vec!["w:tr", "paths:8", "ik:extracted-first", "spine:7", "lock:blocks", "fp:none", "origins:default-tr"]),
        ("compose_wsh_thirty_two_slots", pl(Wrapper::Wsh, vec![k(9, 9), k(9, 9), k(9, 9), k(5, 5)]),
         "wsh(or_d(multi(9,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*,@8/<0;1>/*),or_d(multi(9,@9/<0;1>/*,@10/<0;1>/*,@11/<0;1>/*,@12/<0;1>/*,@13/<0;1>/*,@14/<0;1>/*,@15/<0;1>/*,@16/<0;1>/*,@17/<0;1>/*),or_d(multi(9,@18/<0;1>/*,@19/<0;1>/*,@20/<0;1>/*,@21/<0;1>/*,@22/<0;1>/*,@23/<0;1>/*,@24/<0;1>/*,@25/<0;1>/*,@26/<0;1>/*),multi(5,@27/<0;1>/*,@28/<0;1>/*,@29/<0;1>/*,@30/<0;1>/*,@31/<0;1>/*)))))".to_string(),
         vec!["w:wsh", "paths:4", "slots:32", "head:bare-multi", "lock:none", "ik:none", "fp:none", "origins:default-wsh"]),
        ("compose_tr_thirty_two_slots", pl(Wrapper::Tr, tr32),
         format!("tr({NUMS},{{multi_a(4,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*),{{multi_a(4,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*),{{multi_a(4,@8/<0;1>/*,@9/<0;1>/*,@10/<0;1>/*,@11/<0;1>/*),{{multi_a(4,@12/<0;1>/*,@13/<0;1>/*,@14/<0;1>/*,@15/<0;1>/*),{{multi_a(4,@16/<0;1>/*,@17/<0;1>/*,@18/<0;1>/*,@19/<0;1>/*),{{multi_a(4,@20/<0;1>/*,@21/<0;1>/*,@22/<0;1>/*,@23/<0;1>/*),{{multi_a(4,@24/<0;1>/*,@25/<0;1>/*,@26/<0;1>/*,@27/<0;1>/*),multi_a(4,@28/<0;1>/*,@29/<0;1>/*,@30/<0;1>/*,@31/<0;1>/*)}}}}}}}}}}}}}})"),
         vec!["w:tr", "paths:8", "slots:32", "ik:nums", "spine:7", "lock:none", "fp:none", "origins:default-tr"]),
        // ---- presets (F-453): ONE vector per archetype, built by CALLING the
        // constructor (so a drifted parameter order or default changes the
        // PathList here too), with a hand-typed expected-text literal (so a
        // drifted LOWERING still fails `every_family_entry_renders_as_listed`).
        // Parameters: 2-of-3 and older=26280 wherever the archetype leaves
        // them free (the journey's own canonical values, §4d fixes no
        // defaults); every vector stays within the four journey xpubs.
        ("keyed_compose_preset_plain_multisig", presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap(),
         "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))".to_string(),
         vec!["w:wsh", "paths:1", "head:bare-multi", "lock:none", "sorted", "ik:none", "fp:one-seed-one-path", "origins:default-wsh", "preset:plain-multisig"]),
        ("keyed_compose_preset_simple_timelocked_inheritance", presets::simple_timelocked_inheritance(Wrapper::Wsh, 26280).unwrap(),
         "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:single", "lock:blocks", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh", "preset:simple-timelocked-inheritance"]),
        ("keyed_compose_preset_kofn_recovery", presets::kofn_recovery(Wrapper::Tr, 2, 3, 26280).unwrap(),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(26280))}})"),
         vec!["w:tr", "paths:2", "ik:nums", "spine:2", "lock:blocks", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-tr", "preset:kofn-recovery"]),
        ("keyed_compose_preset_tiered_recovery", presets::tiered_recovery(Wrapper::Wsh, 2, 2, 1, 2, 26280).unwrap(),
         "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:multi(1,@2/<0;1>/*,@3/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:bare-multi", "lock:blocks", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh", "preset:tiered-recovery"]),
        ("keyed_compose_preset_hashlock_gated", presets::hashlock_gated(Wrapper::Wsh, H, 26280).unwrap(),
         format!("wsh(or_i(and_v(v:pkh(@0/<0;1>/*),sha256({HH})),and_v(v:pkh(@1/<0;1>/*),older(26280))))"),
         // R0 fidelity N-1: the head path is one key PLUS a hash, unlocked --
         // neither head:bare-multi (n = 1), head:single (is_bare_single needs
         // no hash), nor head:locked (no lock). `head:hashed` names this
         // fourth shape; it joins SINGULAR_TAGS below since this is the only
         // family vector with it.
         vec!["w:wsh", "paths:2", "head:hashed", "lock:blocks", "hash", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh", "preset:hashlock-gated"]),
        ("keyed_compose_preset_hashlock_gated_hash256", presets::hashlock_gated(Wrapper::Wsh, HK256, 26280).unwrap(),
         "wsh(or_i(and_v(v:pkh(@0/<0;1>/*),hash256(98a20fc25dbcdf236fb0307e3f82cad47fca2e807f3ef82c31993549641cd488)),and_v(v:pkh(@1/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:hashed", "lock:blocks", "hash", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh", "preset:hashlock-gated"]),
        ("keyed_compose_preset_hashlock_gated_ripemd160", presets::hashlock_gated(Wrapper::Wsh, HRIPE, 26280).unwrap(),
         "wsh(or_i(and_v(v:pkh(@0/<0;1>/*),ripemd160(09e7bb5051d89788fb4e4b374126721dbcc2946b)),and_v(v:pkh(@1/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:hashed", "lock:blocks", "hash", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh", "preset:hashlock-gated"]),
        ("keyed_compose_preset_hashlock_gated_hash160", presets::hashlock_gated(Wrapper::Wsh, H160K, 26280).unwrap(),
         "wsh(or_i(and_v(v:pkh(@0/<0;1>/*),hash160(b5b72c0e6896ff59dfa99e0d1052a9c0214cd0bd)),and_v(v:pkh(@1/<0;1>/*),older(26280))))".to_string(),
         vec!["w:wsh", "paths:2", "head:hashed", "lock:blocks", "hash", "ik:none", "fp:one-seed-two-paths", "origins:default-wsh", "preset:hashlock-gated"]),
        ("keyed_compose_preset_decaying_multisig", presets::decaying_multisig(Wrapper::Wsh, 2, 2, 1, 1, 13140, 26280, 1_000_000).unwrap(),
         "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),older(13140)),or_i(and_v(v:pkh(@2/<0;1>/*),older(26280)),and_v(v:pkh(@3/<0;1>/*),after(1000000)))))".to_string(),
         vec!["w:wsh", "paths:3", "head:locked", "lock:blocks", "lock:height", "ik:none", "fp:one-seed-one-path", "fp:one-seed-two-paths", "origins:default-wsh", "preset:decaying-multisig"]),
    ]
}

/// Tags exempt from the two-vector rule, on two DIFFERENT grounds, each named
/// here so a reader does not take one for the other (S0b whole-diff review
/// M-1):
/// - `spine:0` has exactly ONE legal shape: a taptree with m = 0 leaves is one
///   unlocked single key and nothing else — spec §12 item 1's own exemption.
/// - the five remaining `preset:<name>` tags have MANY legal shapes
///   (`plain-multisig,2of4` under `tr` is as legal a `preset:plain-multisig`
///   vector as the one shipped) but ONE vector by deliverable scope: F-453
///   specifies one MANIFEST vector per archetype. The test pins them at exactly
///   one so that a second vector forces an explicit decision here instead of
///   silently widening the exemption.
///
///   **That is exactly what happened to `head:hashed` and
///   `preset:hashlock-gated` in SPEC_hashlock_kinds phase 1**, and the
///   mechanism is worth recording: the three new vectors were first tagged
///   `head:single`, which is wrong for this shape (`is_bare_single` requires
///   `hash.is_none()`), and the wrong tag KEPT THIS GATE QUIET. Tagging them
///   correctly reds it, which is the gate working. Both tags now have four
///   vectors -- one per hash kind -- so both left the exemption and the
///   ordinary "at least two" rule applies, which is the stronger check.
///
/// §12 item 1's own required-tag list (`compose_vectors.rs`) is NOT extended
/// with `preset:*` — presets are not one of the axes that list names, so
/// nothing there needs touching.
pub const SINGULAR_TAGS: &[&str] = &[
    "spine:0",
    // `head:hashed` LEFT this list in SPEC_hashlock_kinds phase 1: the
    // hashlock-gated preset now has one vector per hash kind, so the shape
    // "one key plus a hash, unlocked" has four and not one.
    "preset:plain-multisig",
    "preset:simple-timelocked-inheritance",
    "preset:kofn-recovery",
    "preset:tiered-recovery",
    // NOT singular since SPEC_hashlock_kinds phase 1: the preset has one vector
    // per hash kind (sha256, hash256, ripemd160, hash160), so the ordinary
    // "at least two" rule applies and is a stronger check than an exemption.
    "preset:decaying-multisig",
];
