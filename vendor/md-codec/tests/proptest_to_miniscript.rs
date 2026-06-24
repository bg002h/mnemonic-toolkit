//! Cycle B (stress program) — P6/P7/P8 + anti-vacuity over the typed (T)
//! and wire (W) strategies. Spec:
//! design/BRAINSTORM_proptest_fragment_domain_expansion.md (R4 GREEN).
//!
//! P6: typed descriptors render through `to_miniscript_descriptor`, wire
//!     round-trip exactly, reparse to an EQUAL `miniscript::Descriptor`
//!     (rust-miniscript's parser is the genuinely independent oracle), and
//!     derive an address end-to-end.
//! P7: wire-valid-but-miniscript-invalid inputs refuse CLEANLY (Err, never
//!     a panic) while the wire round-trip stays exact.
//! P8: encoder-side clean errors on out-of-range k/n/children, plus the
//!     k>n encode-reject regression gate + its frozen decode-side golden
//!     (FOLLOWUP `encode-accepts-k-greater-than-n`, closed in md-codec 0.35.2).
#![cfg(feature = "derive")]

mod common;

use bitcoin::Network;
use common::{
    W_BOUNDARY_TIMELOCKS, canon, collect_tags_and_locks, descriptor_from_tree,
    descriptor_with_pubkeys, hash20, hash32, keyarg, multikeys, node2, node3, thresh_node,
    timelock, tr_node, typed_descriptor_strategy, wire_descriptor_strategy, wrap,
};
use md_codec::chunk::{reassemble, split};
use md_codec::decode::{decode_md1_string, decode_payload};
use md_codec::encode::{encode_md1_string, encode_payload};
use md_codec::to_miniscript::to_miniscript_descriptor;
use md_codec::tree::{Body, Node};
use md_codec::{Descriptor, Error, Tag};
use miniscript::DescriptorPublicKey;
use proptest::prelude::*;
use std::collections::HashSet;
use std::str::FromStr;

// ─── P6 oracle chain (shared by the property and the golden cells) ──────

/// Single-string wire-round-trip assertion that tolerates the cycle-4 H6 cap:
/// a descriptor whose payload exceeds the 80-data-symbol single-string limit
/// (e.g. any wallet-policy descriptor carrying a 65-byte xpub TLV) legitimately
/// rejects `encode_md1_string` with `PayloadTooLongForSingleString` — that is
/// the contractual fail-closed outcome, and the chunked round-trip (asserted
/// separately by the callers) is the authoritative wire check for it. When the
/// descriptor DOES fit a single string, the round-trip must be exact.
fn assert_string_round_trip_or_oversize_reject(c: &Descriptor, ctx: &str) {
    match encode_md1_string(c) {
        Ok(s) => assert_eq!(
            &decode_md1_string(&s).unwrap_or_else(|e| panic!("{ctx}: string decodes: {e:?}")),
            c,
            "{ctx}: string round-trip must be exact"
        ),
        Err(Error::PayloadTooLongForSingleString { .. }) => { /* chunked-only; chunk RT asserted by caller */
        }
        Err(e) => panic!("{ctx}: unexpected string-encode error: {e:?}"),
    }
}

/// Run the full P6 chain on `d` and return the derived mainnet receive
/// address string:
/// 1. `to_miniscript_descriptor(&canon(d), 0)` succeeds (failure is RED,
///    never filtered);
/// 2. wire round-trip: encode→string→chunks→decode == canon(d);
/// 3. reparse fixed-point: `Descriptor::from_str(rendered.to_string())`
///    succeeds AND == the constructed Descriptor (PartialEq);
/// 4. `derive_address(0, 0, Bitcoin)` succeeds and equals the reparsed
///    descriptor's `at_derivation_index(0)` address. (Given step 3 the
///    equality is implied; the marginal value is that the full derivation
///    pipeline errors nowhere. Address-oracle independence is anchored by
///    the golden literals in the self-test cells below.)
fn p6_chain(d: &Descriptor) -> String {
    let c = canon(d);
    // Step 1 — converter must succeed.
    let rendered = to_miniscript_descriptor(&c, 0).unwrap_or_else(|e| {
        panic!("P6 step 1: to_miniscript_descriptor must succeed, got {e:?}\ninput: {c:?}")
    });
    // Step 2 — wire round-trip (payload, string, chunks).
    let (bytes, bits) = encode_payload(&c).expect("P6 step 2: canonical encodes");
    assert_eq!(
        decode_payload(&bytes, bits).expect("P6 step 2: payload decodes"),
        c,
        "P6 step 2: payload round-trip must be exact"
    );
    assert_string_round_trip_or_oversize_reject(&c, "P6 step 2");
    let chunks = split(&c).expect("P6 step 2: splits");
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    assert_eq!(
        reassemble(&refs).expect("P6 step 2: reassembles"),
        c,
        "P6 step 2: chunk round-trip must be exact"
    );
    // Step 3 — reparse fixed-point via rust-miniscript's own parser.
    let rendered_str = rendered.to_string();
    let reparsed = miniscript::Descriptor::<DescriptorPublicKey>::from_str(&rendered_str)
        .unwrap_or_else(|e| panic!("P6 step 3: reparse must succeed, got {e:?}\n{rendered_str}"));
    assert_eq!(
        reparsed, rendered,
        "P6 step 3: reparse must be a fixed point"
    );
    // Step 4 — end-to-end address derivation.
    let got = c
        .derive_address(0, 0, Network::Bitcoin)
        .expect("P6 step 4: derive_address succeeds")
        .assume_checked()
        .to_string();
    let expected = reparsed
        .at_derivation_index(0)
        .expect("P6 step 4: at_derivation_index")
        .address(Network::Bitcoin)
        .expect("P6 step 4: address")
        .to_string();
    assert_eq!(got, expected, "P6 step 4: address differential");
    got
}

/// P7 oracle: `to_miniscript_descriptor` returns a clean `Err` (never a
/// panic, never a wrong descriptor) AND the wire round-trip stays exact.
fn assert_p7_clean_refusal(d: &Descriptor) {
    let c = canon(d);
    let res = to_miniscript_descriptor(&c, 0);
    assert!(
        res.is_err(),
        "P7: expected clean refusal, got Ok({})",
        res.unwrap()
    );
    let (bytes, bits) = encode_payload(&c).expect("P7: wire-valid input encodes");
    assert_eq!(
        decode_payload(&bytes, bits).expect("P7: payload decodes"),
        c,
        "P7: payload round-trip must stay exact"
    );
    assert_string_round_trip_or_oversize_reject(&c, "P7");
    let chunks = split(&c).expect("P7: splits");
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    assert_eq!(
        reassemble(&refs).expect("P7: reassembles"),
        c,
        "P7: chunk round-trip must stay exact"
    );
}

// ─── Permanent oracle self-test cells: known-good through P6 ────────────
// Each pins a GOLDEN ADDRESS LITERAL (derived once, prefix-verified, then
// hard-coded) — this anchors the address oracle independently of the
// converter under test.

#[test]
fn self_test_wsh_and_v_pk_older_144() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            timelock(Tag::Older, 144),
        ),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qjrek53xfxcz9epmg7teke3qh0sgs4za8zgnaf8kzr62rd7gp5nrq6xs44a"
    );
}

#[test]
fn self_test_wsh_andor_pk_older_4096_pk() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node3(
            Tag::AndOr,
            keyarg(Tag::PkK, 0),
            timelock(Tag::Older, 4096),
            keyarg(Tag::PkK, 1),
        ),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qg0snqkymvvd0s4pusv2humsdj2t5yf5as4e5sk9w8zl0quqrj4rqr406t2"
    );
}

#[test]
fn self_test_tr_nums_and_v_sha256_pk() {
    let d = descriptor_with_pubkeys(tr_node(
        true,
        0,
        Some(node2(
            Tag::AndV,
            wrap(Tag::Verify, hash32(Tag::Sha256, [0x11; 32])),
            keyarg(Tag::PkK, 0),
        )),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1p"), "expected P2TR, got {addr}");
    assert_eq!(
        addr,
        "bc1psldl66p3tqj0lxcl7zm4eclrxaet4vz5ppqa6sxt5az8u4a6ef2qp5l03l"
    );
}

// ─── GAP-5: minor coverage goldens (multi 17..20 / after / hash256 / ───────
// ─── ripemd160 / hash160) — oracle-independent address anchors for valid
// shapes that were P6-property-covered but golden-less (and, for multi
// 17..=20, never render/address-tested at all — only wire-tier). Same
// derive-once-then-pin discipline as the cells above.

/// `wsh(multi(17,…20 keys))` — the upper edge of the VALID multi window
/// (miniscript caps at 20; n ≥ 21 is the P7 refusal `:629`). The T-tier
/// property caps at 16 (a key-budget choice), so 17..=20 had ZERO
/// render/reparse/address coverage before this cell.
#[test]
fn self_test_wsh_multi_17_of_20() {
    let d = descriptor_with_pubkeys(wrap(Tag::Wsh, multikeys(Tag::Multi, 17, (0..20).collect())));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qlq87tf75y8xlwqg4nv9g4434xesqrql06kvchzyd9ffeld4zlukqcp0xjl"
    );
}

/// `wsh(multi(17,…17 keys))` — the cap+1 edge (first n above the T-tier 16
/// key-budget), n = k = 17.
#[test]
fn self_test_wsh_multi_17_of_17() {
    let d = descriptor_with_pubkeys(wrap(Tag::Wsh, multikeys(Tag::Multi, 17, (0..17).collect())));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qrd2ly6lk960h360kd8dw8ql4a3zk79awfvvkljnukrj7dk7r7fhsp2wcm3"
    );
}

/// `wsh(and_v(v:pk,after(800000)))` — positive `after` golden (the oracle-
/// independent anchor `older(144)` already has; catches an upstream
/// `after`-Display shift the rust-miniscript differential can't, since both
/// sides would move together).
#[test]
fn self_test_wsh_and_v_pk_after_800000() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            timelock(Tag::After, 800000),
        ),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1q7wasuw8zanhkqlgq8gxa3p4yp92nt4sggx34y2glcrm48fg4lgcqj7yw9g"
    );
}

/// `tr(NUMS,and_v(v:hash256,pk))` — hash256 golden (sha256 already has one;
/// these three complete the four hashlock anchors).
#[test]
fn self_test_tr_nums_and_v_hash256_pk() {
    let d = descriptor_with_pubkeys(tr_node(
        true,
        0,
        Some(node2(
            Tag::AndV,
            wrap(Tag::Verify, hash32(Tag::Hash256, [0x22; 32])),
            keyarg(Tag::PkK, 0),
        )),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1p"), "expected P2TR, got {addr}");
    assert_eq!(
        addr,
        "bc1pc56rq4cvyxyguc9cma3ydyjt2rmclzf7x8rgd4xnu7utk790as6qujyeng"
    );
}

/// `tr(NUMS,and_v(v:ripemd160,pk))` — ripemd160 golden.
#[test]
fn self_test_tr_nums_and_v_ripemd160_pk() {
    let d = descriptor_with_pubkeys(tr_node(
        true,
        0,
        Some(node2(
            Tag::AndV,
            wrap(Tag::Verify, hash20(Tag::Ripemd160, [0x33; 20])),
            keyarg(Tag::PkK, 0),
        )),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1p"), "expected P2TR, got {addr}");
    assert_eq!(
        addr,
        "bc1ps5fy26yhey556tuduck6gfu450kpxhq2h0d7lswe6nfzp7xwytmqksn0d5"
    );
}

/// `tr(NUMS,and_v(v:hash160,pk))` — hash160 golden.
#[test]
fn self_test_tr_nums_and_v_hash160_pk() {
    let d = descriptor_with_pubkeys(tr_node(
        true,
        0,
        Some(node2(
            Tag::AndV,
            wrap(Tag::Verify, hash20(Tag::Hash160, [0x44; 20])),
            keyarg(Tag::PkK, 0),
        )),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1p"), "expected P2TR, got {addr}");
    assert_eq!(
        addr,
        "bc1pp8qa0h4vkhumj7fpqg5lf5y6cau0jdje4h4jprk54x7g3gp8u2yq23k6qa"
    );
}

/// Miniscript-leniency pin: `older(0x10000)` is OUT of the BIP-68 mask
/// (low 16 bits zero — consensus treats it as no-op) yet rust-miniscript
/// 13.0.0 ACCEPTS it. This is the known leniency that motivated the
/// toolkit's own mask gate (toolkit v0.53.9). Pinned Ok LOUDLY: if a
/// future miniscript starts rejecting it, this cell goes red and the
/// P6/P7 class split must be re-derived.
#[test]
fn self_test_older_0x10000_miniscript_leniency() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            timelock(Tag::Older, 0x0001_0000),
        ),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qcj2atyh7su8wnqn3ew4drtxmfh3tl5y3n6uwvw38jep83xfy0alspfzaze"
    );
}

/// Same leniency pin for the time-class out-of-mask value `older(0x00410000)`
/// (bit 22 set, low 16 bits zero). See toolkit v0.53.9.
#[test]
fn self_test_older_0x00410000_miniscript_leniency() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            timelock(Tag::Older, 0x0041_0000),
        ),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qznrwq5w3wmzhlhjkazz4zqlhc092f79x6622uun9fxset0zeld9sqk8djl"
    );
}

/// Empirical sanity proof for the T-tier tap thresh production
/// (`thresh(1, pk_h(@1), s:pk(@2))` as a tap leaf) — round-3 evidence
/// proved the shape via from_str; this cell proves it through the FULL
/// P6 chain including the Tr-only reparse sanity branch.
#[test]
fn self_test_tr_thresh_pkh_swap_pk_leaf() {
    let d = descriptor_with_pubkeys(tr_node(
        false,
        0,
        Some(thresh_node(
            1,
            vec![keyarg(Tag::PkH, 1), wrap(Tag::Swap, keyarg(Tag::PkK, 2))],
        )),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1p"), "expected P2TR, got {addr}");
    assert_eq!(
        addr,
        "bc1pm0mejtph5njw5lespxmn2y3t3fxk9c3maku84llw2s7rhqa6kansahz935"
    );
}

/// Empirical sanity proof for the T-tier tap `a:` W-production
/// (`and_b(pk(@1), a:pk_h(@2))` as a tap leaf): round-3 proved a:pkh
/// type-valid; this cell proves tap-context sanity through the full chain.
#[test]
fn self_test_tr_and_b_pk_alt_pkh_leaf() {
    let d = descriptor_with_pubkeys(tr_node(
        false,
        0,
        Some(node2(
            Tag::AndB,
            keyarg(Tag::PkK, 1),
            wrap(Tag::Alt, keyarg(Tag::PkH, 2)),
        )),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1p"), "expected P2TR, got {addr}");
    assert_eq!(
        addr,
        "bc1pqf9kp8ehq9dn8at3wzp2m76pkc7eq902y73dupl5p02dpzemvk5q88urry"
    );
}

/// Legacy context through the full chain: `sh(or_d(multi(1,@0,@1), pk(@2)))`.
#[test]
fn self_test_sh_or_d_multi_pk() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Sh,
        node2(
            Tag::OrD,
            multikeys(Tag::Multi, 1, vec![0, 1]),
            keyarg(Tag::PkK, 2),
        ),
    ));
    let addr = p6_chain(&d);
    assert!(addr.starts_with('3'), "expected P2SH, got {addr}");
    assert_eq!(addr, "3HVMGTDDMN9FBw8QByVehgNb52m8k4WmwW");
}

/// LOUD characterization of an UPSTREAM rust-miniscript 13.0.0
// ─── GAP-2: the seven previously-unrendered fragment arms ───────────────
// `to_miniscript.rs` had ZERO render-layer test for DupIf/NonZero/
// ZeroNotEqual/OrB/OrC/True/False — the W strategy wire-generates them but
// the render leg P6 runs only over T, which omitted all seven. Each cell
// hosts the fragment in a valid typed wsh descriptor and pins the rendered
// form (sugar INCLUDED: `or_i(X,0)`→`u:`, `and_v(X,1)`→`t:`, fused `dv:`/`tv:`
// — miniscript never Displays a literal 0/1, so pinning the sugar consumer IS
// the True/False contract) + a golden mainnet address. The reparse
// fixed-point inside `p6_chain` is the mis-render oracle. These are the
// render-layer replacement for the byte-layer pins lost when
// `hand_ast_coverage.rs` was removed in the v0.12.0 strip (5350f8a;
// FOLLOWUPs `v06-corpus-{d-wrapper,or-c,j-n-wrapper}-coverage`).

#[test]
fn self_test_wsh_or_b_pk_s_pk() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::OrB,
            keyarg(Tag::PkK, 0),
            wrap(Tag::Swap, keyarg(Tag::PkK, 1)),
        ),
    ));
    let rendered = to_miniscript_descriptor(&canon(&d), 0).unwrap().to_string();
    assert!(rendered.contains("or_b("), "render: {rendered}");
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1q2epc9vj8hy2mzmh9uyaz9adhp4q9yvu0aygw2httyngv6c7ct5wseumd3l"
    );
}

#[test]
fn self_test_wsh_t_or_c_true() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            node2(
                Tag::OrC,
                keyarg(Tag::PkK, 0),
                wrap(Tag::Verify, keyarg(Tag::PkK, 1)),
            ),
            Node {
                tag: Tag::True,
                body: Body::Empty,
            },
        ),
    ));
    let rendered = to_miniscript_descriptor(&canon(&d), 0).unwrap().to_string();
    assert!(rendered.contains("t:or_c("), "render: {rendered}");
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qh3wd6a5nn5ccgqg4hj7aj7mjtgwc39gjakyen2m25my4fx3vdx0q9nhznw"
    );
}

#[test]
fn self_test_wsh_or_i_dupif_v_older() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::OrI,
            keyarg(Tag::PkK, 0),
            wrap(Tag::DupIf, wrap(Tag::Verify, timelock(Tag::Older, 144))),
        ),
    ));
    let rendered = to_miniscript_descriptor(&canon(&d), 0).unwrap().to_string();
    assert!(rendered.contains("dv:older(144)"), "render: {rendered}");
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qre28e06mc7r8fyam0my2uegn096eygzvx52v9jzg07avehev3lws5nf5qc"
    );
}

#[test]
fn self_test_wsh_nonzero_pk() {
    let d = descriptor_with_pubkeys(wrap(Tag::Wsh, wrap(Tag::NonZero, keyarg(Tag::PkK, 0))));
    let rendered = to_miniscript_descriptor(&canon(&d), 0).unwrap().to_string();
    assert!(rendered.contains("j:pk("), "render: {rendered}");
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qewdar8ze6tynzushrg7fmnedlw4xm6q7vj6tmcey2kalwryy7ens08c3x0"
    );
}

#[test]
fn self_test_wsh_or_i_zne_and_v() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::OrI,
            keyarg(Tag::PkK, 0),
            wrap(
                Tag::ZeroNotEqual,
                node2(
                    Tag::AndV,
                    wrap(Tag::Verify, keyarg(Tag::PkK, 1)),
                    timelock(Tag::Older, 144),
                ),
            ),
        ),
    ));
    let rendered = to_miniscript_descriptor(&canon(&d), 0).unwrap().to_string();
    assert!(rendered.contains("n:and_v("), "render: {rendered}");
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qjnwlx28qetpwdp3wfrv3emhhpc3dc2cya75qy3gzqfmzmn2ea36q8fx4y4"
    );
}

#[test]
fn self_test_wsh_or_i_false_u_sugar() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::OrI,
            keyarg(Tag::PkK, 0),
            Node {
                tag: Tag::False,
                body: Body::Empty,
            },
        ),
    ));
    let rendered = to_miniscript_descriptor(&canon(&d), 0).unwrap().to_string();
    assert!(rendered.contains("u:pk("), "render: {rendered}");
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1qly5mzr0gwyquj5jwllans468wnmwc89u27sf9xnqjcldswqwcdxsfms2dk"
    );
}

#[test]
fn self_test_wsh_and_v_true_t_sugar() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            Node {
                tag: Tag::True,
                body: Body::Empty,
            },
        ),
    ));
    let rendered = to_miniscript_descriptor(&canon(&d), 0).unwrap().to_string();
    assert!(rendered.contains("tv:pk("), "render: {rendered}");
    let addr = p6_chain(&d);
    assert!(addr.starts_with("bc1q"), "expected P2WSH, got {addr}");
    assert_eq!(
        addr,
        "bc1q779rp8l2cy6v63ea5elayzeryqguxq89ez7rh3w8ajx6pgf33p7qanlqr5"
    );
}

/// Display/parse asymmetry that P6 found during bring-up (NOT an md-codec
/// bug — reproduced with pure miniscript, no md-codec involvement):
/// a DEPTH-2 taptree built via `TapTree::combine(combine(a,b),c)` Displays
/// as the malformed `{{a,b,c}}` instead of `{{a,b},c}`, and miniscript's
/// OWN `Descriptor::from_str` rejects that output
/// (`IncorrectNumberOfChildren { description: "taptree branch", .. }`).
/// A correctly-written depth-2 string PARSES Ok but re-Displays broken
/// (same checksum — Display is the faulty side).
///
/// md-codec's wire round-trip, converter, and address derivation are all
/// unaffected (none go through the string form) — asserted below. The
/// T-tier generator constrains taptrees to depth ≤ 1 because of this
/// (see common/mod.rs::t_tr_tree). If a future miniscript bump fixes
/// Display, the final assertion flips: restore the depth-2 generator arm
/// and invert this cell.
#[test]
fn upstream_taptree_depth2_display_asymmetry() {
    let d = descriptor_with_pubkeys(tr_node(
        false,
        0,
        Some(common::taptree2(
            common::taptree2(keyarg(Tag::PkK, 1), keyarg(Tag::PkK, 2)),
            keyarg(Tag::PkK, 3),
        )),
    ));
    let c = canon(&d);
    // Converter + derivation + wire all work…
    let rendered = to_miniscript_descriptor(&c, 0).expect("depth-2 taptree converts fine");
    let addr = c
        .derive_address(0, 0, Network::Bitcoin)
        .expect("depth-2 taptree derives fine")
        .assume_checked()
        .to_string();
    assert!(addr.starts_with("bc1p"), "expected P2TR, got {addr}");
    assert_string_round_trip_or_oversize_reject(&c, "depth-2 taptree");
    // …but the rendered STRING is not reparseable under pinned 13.0.0.
    let rendered_str = rendered.to_string();
    assert!(
        miniscript::Descriptor::<DescriptorPublicKey>::from_str(&rendered_str).is_err(),
        "UPSTREAM FIXED? miniscript now reparses its own depth-2 taptree \
         Display ({rendered_str}); restore the t_tr_tree depth-2 arm and \
         invert this cell"
    );
}

// ─── Permanent oracle self-test cells: known-bad through P7 ─────────────

#[test]
fn self_test_bad_sortedmultia_wsh_leaf() {
    // SortedMultiA anywhere — rust-miniscript v13 has no Terminal
    // (FOLLOWUP `md-codec-sortedmulti-a-to-miniscript-rendering-gap`).
    let d = descriptor_with_pubkeys(wrap(Tag::Wsh, multikeys(Tag::SortedMultiA, 1, vec![0, 1])));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_sortedmultia_tap_leaf() {
    let d = descriptor_with_pubkeys(tr_node(
        false,
        0,
        Some(multikeys(Tag::SortedMultiA, 2, vec![1, 2])),
    ));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_rawpkh_leaf() {
    // RawPkH is not constructible via miniscript's public API.
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            Node {
                tag: Tag::RawPkH,
                body: Body::Hash160Body([0x22; 20]),
            },
        ),
    ));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_sortedmulti_under_combinator() {
    // The Cycle-A engrave-but-can't-restore shape: SortedMulti must be the
    // sole child of wsh/sh; under a combinator it wire-round-trips but
    // refuses to render.
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, multikeys(Tag::SortedMulti, 1, vec![0, 1])),
            timelock(Tag::Older, 1),
        ),
    ));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_shape_c_check_over_or_i() {
    // Shape C: Check over a NON-bare-key child double-wraps and errors
    // (`c:` over type B). The 0.35.1 idempotence arm only collapses
    // Check(bare PkK/PkH).
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        wrap(
            Tag::Check,
            node2(Tag::OrI, keyarg(Tag::PkK, 0), keyarg(Tag::PkK, 1)),
        ),
    ));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_after_zero() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            timelock(Tag::After, 0),
        ),
    ));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_after_bit31() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            timelock(Tag::After, 0x8000_0000),
        ),
    ));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_older_zero() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            timelock(Tag::Older, 0),
        ),
    ));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_older_bit31() {
    let d = descriptor_with_pubkeys(wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            timelock(Tag::Older, 0x8000_0000),
        ),
    ));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_wsh_multi_21_keys() {
    let d = descriptor_with_pubkeys(wrap(Tag::Wsh, multikeys(Tag::Multi, 2, (0..21).collect())));
    assert_p7_clean_refusal(&d);
}

#[test]
fn self_test_bad_sh_multi_21_keys() {
    let d = descriptor_with_pubkeys(wrap(Tag::Sh, multikeys(Tag::Multi, 2, (0..21).collect())));
    assert_p7_clean_refusal(&d);
}

// ─── P6 / P7 properties ─────────────────────────────────────────────────

proptest! {
    // P6 — typed strategy renders, wire-round-trips, reparses to a fixed
    // point, and derives end-to-end. NO filtering: any failure is a
    // generator bug or a codec bug, both RED.
    #[test]
    fn p6_typed_to_miniscript_round_trip(d in typed_descriptor_strategy()) {
        p6_chain(&d);
    }

    // P7 (parametrized classes) — consensus-invalid `after` values refuse
    // cleanly and stay wire-exact.
    #[test]
    fn p7_bad_after_refuses_cleanly(v in prop_oneof![Just(0u32), 0x8000_0000u32..=u32::MAX]) {
        let d = descriptor_with_pubkeys(wrap(
            Tag::Wsh,
            node2(
                Tag::AndV,
                wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
                timelock(Tag::After, v),
            ),
        ));
        assert_p7_clean_refusal(&d);
    }

    // P7 — `older(0)` / `older(bit-31-set)` refuse cleanly. (NOT in this
    // set: out-of-BIP-68-mask values like 0x10000 — miniscript accepts
    // them; pinned Ok in the leniency cells above.)
    #[test]
    fn p7_bad_older_refuses_cleanly(v in prop_oneof![Just(0u32), 0x8000_0000u32..=u32::MAX]) {
        let d = descriptor_with_pubkeys(wrap(
            Tag::Wsh,
            node2(
                Tag::AndV,
                wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
                timelock(Tag::Older, v),
            ),
        ));
        assert_p7_clean_refusal(&d);
    }

    // P7 — Segwitv0/Legacy multi with 21..=32 keys exceeds
    // MAX_PUBKEYS_PER_MULTISIG and refuses cleanly in both contexts.
    #[test]
    fn p7_oversize_multi_refuses_cleanly(
        n in 21u8..=32,
        k in 1u8..=20,
        legacy in any::<bool>(),
    ) {
        let root = if legacy { Tag::Sh } else { Tag::Wsh };
        let d = descriptor_with_pubkeys(wrap(root, multikeys(Tag::Multi, k, (0..n).collect())));
        assert_p7_clean_refusal(&d);
    }
}

// ─── P8 — encoder-side clean errors + the k>n gap pin ───────────────────

proptest! {
    // P8 — out-of-range multi-family threshold k (0 or 33..=255) is a clean
    // encoder Err, never a panic.
    #[test]
    fn p8_encode_rejects_out_of_range_multi_k(
        k in prop_oneof![Just(0u8), 33u8..],
        len in 1usize..=8,
        tag in prop::sample::select(vec![
            Tag::Multi, Tag::SortedMulti, Tag::MultiA, Tag::SortedMultiA
        ]),
    ) {
        let d = descriptor_from_tree(
            wrap(Tag::Wsh, multikeys(tag, k, (0..len as u8).collect())),
            true,
        );
        let err = encode_payload(&d).expect_err("k out of 1..=32 must not encode");
        prop_assert!(
            matches!(err, Error::ThresholdOutOfRange { .. }),
            "expected ThresholdOutOfRange, got {err:?}"
        );
    }

    // P8 — out-of-range thresh k is a clean encoder Err.
    #[test]
    fn p8_encode_rejects_out_of_range_thresh_k(k in 33u8..) {
        let d = descriptor_from_tree(
            wrap(
                Tag::Wsh,
                thresh_node(k, vec![keyarg(Tag::PkK, 0), keyarg(Tag::PkK, 1)]),
            ),
            true,
        );
        let err = encode_payload(&d).expect_err("thresh k out of 1..=32 must not encode");
        prop_assert!(
            matches!(err, Error::ThresholdOutOfRange { .. }),
            "expected ThresholdOutOfRange, got {err:?}"
        );
    }
}

#[test]
fn p8_encode_rejects_empty_multi_indices() {
    // 0 keys in a multi-family body: clean ChildCountOutOfRange. The tree
    // still references @0 elsewhere so n ≥ 1 (a keyless tree would fail at
    // PathDecl instead).
    let d = descriptor_from_tree(
        wrap(
            Tag::Wsh,
            node2(
                Tag::AndV,
                wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
                multikeys(Tag::Multi, 1, vec![]),
            ),
        ),
        true,
    );
    let err = encode_payload(&d).expect_err("empty multi must not encode");
    assert!(
        matches!(err, Error::ChildCountOutOfRange { count: 0 }),
        "expected ChildCountOutOfRange, got {err:?}"
    );
}

#[test]
fn p8_encode_rejects_empty_thresh_children() {
    let d = descriptor_from_tree(
        wrap(
            Tag::Wsh,
            node2(
                Tag::AndV,
                wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
                thresh_node(1, vec![]),
            ),
        ),
        true,
    );
    let err = encode_payload(&d).expect_err("empty thresh must not encode");
    assert!(
        matches!(err, Error::ChildCountOutOfRange { count: 0 }),
        "expected ChildCountOutOfRange, got {err:?}"
    );
}

#[test]
fn p8_encode_rejects_more_than_32_multi_indices() {
    // 33 repeated key slots in one multi body (n stays small — duplicates):
    // clean ChildCountOutOfRange at write_node.
    let d = descriptor_from_tree(wrap(Tag::Wsh, multikeys(Tag::Multi, 1, vec![0; 33])), true);
    let err = encode_payload(&d).expect_err("33-slot multi must not encode");
    assert!(
        matches!(err, Error::ChildCountOutOfRange { count: 33 }),
        "expected ChildCountOutOfRange, got {err:?}"
    );
}

#[test]
fn p8_encode_rejects_more_than_32_distinct_keys() {
    // 33 DISTINCT keys → n = 33 > 32: clean KeyCountOutOfRange at the
    // PathDecl (written before the tree).
    let d = descriptor_from_tree(
        wrap(Tag::Wsh, multikeys(Tag::Multi, 1, (0..33).collect())),
        true,
    );
    let err = encode_payload(&d).expect_err("n = 33 must not encode");
    assert!(
        matches!(err, Error::KeyCountOutOfRange { .. }),
        "expected KeyCountOutOfRange, got {err:?}"
    );
}

/// LOUD regression gate for the encoder-side k>n fix (`encode-accepts-k-greater-than-n`,
/// md-codec 0.35.2; companion in mnemonic-toolkit). A multi with k > n (both ≤ 32)
/// now FAILS to encode with `KGreaterThanN` — closing the engrave-but-can't-restore
/// gap (Cycle-A family `bundle-accepts-sortedmulti-in-combinator-restore-cannot`).
/// Pre-fix this cell asserted encode-Ok + decode-Err (the gap); the gate in
/// `write_node`'s `Body::MultiKeys` arm (the mirror of the decode-side reject)
/// inverts it. The decode-side reject's own coverage is preserved by
/// `p8_decode_still_rejects_k_greater_than_n` below.
#[test]
fn p8_encode_rejects_k_greater_than_n() {
    let d = descriptor_from_tree(wrap(Tag::Wsh, multikeys(Tag::Multi, 3, vec![0, 1])), true);
    let err = encode_payload(&d).expect_err("encode must REJECT k=3-of-n=2 (gate landed)");
    assert!(
        matches!(err, Error::KGreaterThanN { k: 3, n: 2 }),
        "expected KGreaterThanN, got {err:?}"
    );
    // The string door rejects too — same trap closed end-to-end.
    let serr = encode_md1_string(&d).expect_err("string encode must reject k>n");
    assert!(
        matches!(serr, Error::KGreaterThanN { k: 3, n: 2 }),
        "expected KGreaterThanN, got {serr:?}"
    );
}

/// Companion to `p8_encode_rejects_k_greater_than_n` for the `Body::Variable`
/// (thresh) arm — `multikeys`/`Tag::Multi` exercises only `Body::MultiKeys`, so
/// the thresh arm of the gate needs its own red-first cell.
#[test]
fn p8_encode_rejects_k_greater_than_n_thresh() {
    let tree = wrap(
        Tag::Wsh,
        thresh_node(3, vec![keyarg(Tag::PkK, 0), keyarg(Tag::PkK, 1)]),
    );
    let err = encode_payload(&descriptor_from_tree(tree, true))
        .expect_err("thresh encode must REJECT k=3-of-n=2");
    assert!(
        matches!(err, Error::KGreaterThanN { k: 3, n: 2 }),
        "expected KGreaterThanN, got {err:?}"
    );
}

/// Boundary: k = n (the valid equal case) still ENCODES on both arms — proves
/// the gate is `>` not `>=` and does not over-reject the legitimate equal case.
#[test]
fn p8_encode_accepts_k_equal_n_boundary() {
    let multi = descriptor_from_tree(wrap(Tag::Wsh, multikeys(Tag::Multi, 2, vec![0, 1])), true);
    assert!(
        encode_payload(&multi).is_ok(),
        "k=2-of-n=2 multi must encode"
    );
    let thresh = descriptor_from_tree(
        wrap(
            Tag::Wsh,
            thresh_node(2, vec![keyarg(Tag::PkK, 0), keyarg(Tag::PkK, 1)]),
        ),
        true,
    );
    assert!(
        encode_payload(&thresh).is_ok(),
        "k=2-of-n=2 thresh must encode"
    );
}

/// Coverage preservation: now that the encoder REFUSES k>n, the decode-side
/// `KGreaterThanN` reject (`tree.rs` Multi/Thresh arms) is no longer reachable
/// via encode-then-decode. This frozen wire payload is the exact pre-gate output
/// of `encode_payload(wsh(multi(3,@0,@1)))` (captured 2026-06-12) — it pins that a
/// corrupt/hostile card whose multi field carries k>n still decodes to a clean
/// `KGreaterThanN` (never a wrong descriptor, never a panic). If the wire format
/// changes, regenerate by encoding the same tree before the gate (or assert the
/// new bytes still carry k=3,n=2).
#[test]
fn p8_decode_still_rejects_k_greater_than_n() {
    let bytes = [
        0xa0, 0x4e, 0x39, 0x52, 0xce, 0xf9, 0x6f, 0x9a, 0xf9, 0xe0, 0x01, 0x82, 0x18, 0x41, 0x40,
    ];
    let err = decode_payload(&bytes, 114).expect_err("decode must reject the k>n wire payload");
    assert!(
        matches!(err, Error::KGreaterThanN { k: 3, n: 2 }),
        "expected KGreaterThanN, got {err:?}"
    );
}

// ─── Anti-vacuity: generator coverage (fixed-seed TestRunner) ───────────

const W_TARGET_TAGS: [Tag; 34] = [
    Tag::Wsh,
    Tag::Sh,
    Tag::Tr,
    Tag::TapTree,
    Tag::PkK,
    Tag::PkH,
    Tag::Multi,
    Tag::SortedMulti,
    Tag::MultiA,
    Tag::SortedMultiA,
    Tag::After,
    Tag::Older,
    Tag::Sha256,
    Tag::Hash256,
    Tag::Ripemd160,
    Tag::Hash160,
    Tag::RawPkH,
    Tag::True,
    Tag::False,
    Tag::Check,
    Tag::Verify,
    Tag::Swap,
    Tag::Alt,
    Tag::DupIf,
    Tag::NonZero,
    Tag::ZeroNotEqual,
    Tag::AndV,
    Tag::AndB,
    Tag::AndOr,
    Tag::OrB,
    Tag::OrC,
    Tag::OrD,
    Tag::OrI,
    Tag::Thresh,
];

/// All to_miniscript-supported tags the typed grammar emits.
const T_TARGET_TAGS: [Tag; 30] = [
    Tag::Wsh,
    Tag::Sh,
    Tag::Tr,
    Tag::TapTree,
    Tag::PkK,
    Tag::PkH,
    Tag::Multi,
    Tag::MultiA,
    Tag::After,
    Tag::Older,
    Tag::Sha256,
    Tag::Hash256,
    Tag::Ripemd160,
    Tag::Hash160,
    Tag::Verify,
    Tag::Swap,
    Tag::Alt,
    Tag::AndV,
    Tag::AndB,
    Tag::AndOr,
    Tag::OrD,
    Tag::OrI,
    Tag::Thresh,
    // GAP-2: the seven previously-omitted fragment arms, now generated by
    // `t_segwit_tree`'s `seven` production (FIXED proven shapes; common/mod.rs).
    Tag::OrB,
    Tag::OrC,
    Tag::DupIf,
    Tag::NonZero,
    Tag::ZeroNotEqual,
    Tag::True,
    Tag::False,
];

const T_BOUNDARY_AFTER: [u32; 7] = [
    1,
    144,
    0xFFFF,
    0x0001_0000,
    499_999_999,
    500_000_000,
    0x7FFF_FFFF,
];
const T_BOUNDARY_OLDER: [u32; 7] = [
    1,
    144,
    0xFFFF,
    0x0001_0000,
    0x0040_0001,
    0x0040_FFFF,
    0x0041_0000,
];

#[test]
fn w_generator_covers_all_fragments() {
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    let mut runner = TestRunner::deterministic();
    let strat = wire_descriptor_strategy();
    let mut tags: HashSet<Tag> = HashSet::new();
    let mut locks: HashSet<u32> = HashSet::new();
    let (mut pubkeys, mut fps, mut origins, mut divergent, mut shared) =
        (false, false, false, false, false);
    for _ in 0..1024 {
        let d = strat.new_tree(&mut runner).expect("generates").current();
        collect_tags_and_locks(&d.tree, &mut tags, &mut locks);
        pubkeys |= d.tlv.pubkeys.is_some();
        fps |= d.tlv.fingerprints.is_some();
        origins |= d.tlv.origin_path_overrides.is_some();
        divergent |= matches!(d.path_decl.paths, md_codec::PathDeclPaths::Divergent(_));
        shared |= matches!(d.path_decl.paths, md_codec::PathDeclPaths::Shared(_));
    }
    for t in W_TARGET_TAGS {
        assert!(tags.contains(&t), "W strategy never generated {t:?}");
    }
    for v in W_BOUNDARY_TIMELOCKS {
        assert!(
            locks.contains(&v),
            "W strategy never generated boundary timelock {v:#x}"
        );
    }
    assert!(pubkeys, "W strategy never attached a Pubkeys TLV");
    assert!(fps, "W strategy never attached a Fingerprints TLV");
    assert!(origins, "W strategy never attached OriginPathOverrides");
    assert!(divergent && shared, "W must mix Shared and Divergent decls");
}

#[test]
fn t_generator_covers_all_fragments() {
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    let mut runner = TestRunner::deterministic();
    let strat = typed_descriptor_strategy();
    let mut tags: HashSet<Tag> = HashSet::new();
    let mut locks: HashSet<u32> = HashSet::new();
    for _ in 0..2048 {
        let d = strat.new_tree(&mut runner).expect("generates").current();
        collect_tags_and_locks(&d.tree, &mut tags, &mut locks);
    }
    for t in T_TARGET_TAGS {
        assert!(tags.contains(&t), "T strategy never generated {t:?}");
    }
    for v in T_BOUNDARY_AFTER {
        assert!(
            locks.contains(&v),
            "T strategy never generated boundary after/older value {v:#x}"
        );
    }
    for v in T_BOUNDARY_OLDER {
        assert!(
            locks.contains(&v),
            "T strategy never generated boundary older value {v:#x}"
        );
    }
}
