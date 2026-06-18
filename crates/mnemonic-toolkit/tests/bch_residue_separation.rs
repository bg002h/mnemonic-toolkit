//! Cross-format BCH domain-separation conformance (constellation-level).
//!
//! The three m-format card codecs (ms1 / md1 / mk1) share the BIP-93 codex32
//! BCH generator, and md1/mk1 additionally share the same `POLYMOD_INIT`. Domain
//! separation between the formats is therefore carried entirely by two things:
//! (a) each format's distinct per-HRP **target residue** constant, and (b) the
//! HRP, which is folded into the checksummed input via `hrp_expand`. This test
//! pins BOTH mechanisms so a future edit that accidentally collapsed a target
//! constant onto another format — or dropped the HRP from the checksum input —
//! fails loudly here.
//!
//! It is the executable answer to "should ms1/md1/mk1 each use a different
//! residue": yes, and this is the cross-crate proof that they do and that it
//! actually separates them — the one invariant none of the three sibling crates
//! can assert in isolation (each sees only its own constant). Per-codec NUMS /
//! standard derivation is pinned where the constant lives:
//!   - md-codec `bch::tests::md_regular_const_reproduces_from_nums_domain`
//!   - mk-codec `consts::tests::nums_constants_reproduce_from_domain`
//!   - ms-codec `tests/bch_all_lengths.rs::ms_regular_const_is_secretshare32_packed`

use md_codec::bch::MD_REGULAR_CONST;
use mk_codec::{MK_LONG_CONST, MK_REGULAR_CONST};
use ms_codec::bch::MS_REGULAR_CONST;

/// (a) The four per-HRP × per-code target residues must be pairwise distinct.
/// A collision would let one format's valid codeword satisfy another format's
/// checksum target.
#[test]
fn target_residues_are_pairwise_distinct() {
    let consts: [(&str, u128); 4] = [
        ("MS_REGULAR_CONST", MS_REGULAR_CONST),
        ("MD_REGULAR_CONST", MD_REGULAR_CONST),
        ("MK_REGULAR_CONST", MK_REGULAR_CONST),
        ("MK_LONG_CONST", MK_LONG_CONST),
    ];
    for i in 0..consts.len() {
        for j in (i + 1)..consts.len() {
            assert_ne!(
                consts[i].1, consts[j].1,
                "domain-separation residues {} and {} collide ({:#x})",
                consts[i].0, consts[j].0, consts[i].1,
            );
        }
    }
}

/// (b) A codeword valid under one codec must be rejected by the other two, and
/// by its own codec under a foreign HRP — concretely proving that the target
/// residue AND the HRP each separate the formats. All three codecs expose the
/// same symmetric `bch_{create_checksum,verify}_regular(hrp, &[u8])` API.
#[test]
fn valid_codeword_of_one_format_is_rejected_by_the_others() {
    // Arbitrary data-part symbols (5-bit values). The regular-code data length
    // is irrelevant to separation, so one representative length suffices; this
    // is a pure-checksum test, not a semantic-payload test.
    let data: Vec<u8> = (0..20u8).map(|i| i % 32).collect();

    // A valid codeword in each codec: data ++ that codec's own regular checksum.
    let ms_word = with_ck(
        &data,
        ms_codec::bch::bch_create_checksum_regular("ms", &data),
    );
    let md_word = with_ck(
        &data,
        md_codec::bch::bch_create_checksum_regular("md", &data),
    );
    let mk_word = with_ck(
        &data,
        mk_codec::string_layer::bch::bch_create_checksum_regular("mk", &data),
    );

    // Positive controls: each codec accepts its own codeword under its own HRP.
    assert!(
        ms_codec::bch::bch_verify_regular("ms", &ms_word),
        "ms must accept its own codeword"
    );
    assert!(
        md_codec::bch::bch_verify_regular("md", &md_word),
        "md must accept its own codeword"
    );
    assert!(
        mk_codec::string_layer::bch::bch_verify_regular("mk", &mk_word),
        "mk must accept its own codeword"
    );

    // Cross-format reject: each sibling codec rejects the foreign codeword
    // (different target residue; for ms vs md/mk also a different POLYMOD_INIT).
    assert!(
        !md_codec::bch::bch_verify_regular("md", &ms_word),
        "md must reject an ms1 codeword"
    );
    assert!(
        !mk_codec::string_layer::bch::bch_verify_regular("mk", &ms_word),
        "mk must reject an ms1 codeword"
    );
    assert!(
        !ms_codec::bch::bch_verify_regular("ms", &md_word),
        "ms must reject an md1 codeword"
    );
    assert!(
        !mk_codec::string_layer::bch::bch_verify_regular("mk", &md_word),
        "mk must reject an md1 codeword"
    );
    assert!(
        !ms_codec::bch::bch_verify_regular("ms", &mk_word),
        "ms must reject an mk1 codeword"
    );
    assert!(
        !md_codec::bch::bch_verify_regular("md", &mk_word),
        "md must reject an mk1 codeword"
    );

    // The HRP is load-bearing too: each codec rejects its OWN codeword under a
    // foreign HRP, because the HRP is folded into the checksummed input.
    assert!(
        !ms_codec::bch::bch_verify_regular("md", &ms_word),
        "ms codeword must fail under the md HRP"
    );
    assert!(
        !md_codec::bch::bch_verify_regular("mk", &md_word),
        "md codeword must fail under the mk HRP"
    );
    assert!(
        !mk_codec::string_layer::bch::bch_verify_regular("ms", &mk_word),
        "mk codeword must fail under the ms HRP"
    );
}

fn with_ck(data: &[u8], ck: [u8; 13]) -> Vec<u8> {
    let mut w = data.to_vec();
    w.extend_from_slice(&ck);
    w
}
