//! Stage 1b task 4: §2's derivation -- Liana's unspendable-xpub recipe
//! (`md_codec::nums::liana_unspendable_xpub`), checked against all eight
//! vendored evidence cases in `tests/fixtures/liana/cases.json`
//! (`scripts/vendor-liana-evidence.sh`).
//!
//! The recipe is Liana's own, `liana/src/descriptors/analysis.rs:398-430`:
//! sha256 over each leaf key's 33-byte compressed pubkey, concatenated in
//! descriptor left-to-right (wire) order -- NOT sorted, NOT deduplicated.
//! Sorting/deduplicating is the abandoned bitcoin/bips PR #1746 recipe, a
//! DIFFERENT wallet -- `sorting_or_deduplicating_produces_a_DIFFERENT_xpub`
//! below pins that they diverge.

mod common;

use bitcoin::Network;
use md_codec::chunk;
use md_codec::decode_payload;
use md_codec::decode_with_correction;
use md_codec::encode_payload;
use md_codec::identity::{compute_wallet_descriptor_template_id, compute_wallet_policy_id};
use md_codec::nums::liana_unspendable_xpub;
use md_codec::to_miniscript::to_miniscript_descriptor_multipath_with_network;
use md_codec::tree::Node;
use md_codec::use_site_path::{Alternative, UseSitePath};
use md_codec::{Error, OriginPath, PathComponent, PathDecl, PathDeclPaths, Tag, TlvSection};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/vendored.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/liana.rs"
));

#[test]
fn the_recipe_reproduces_every_golden_xpub() {
    for c in all_cases() {
        // all EIGHT, not just the four accepted
        let leaves: Vec<[u8; 33]> = c.leaf_pubkeys();
        assert_eq!(
            liana_unspendable_xpub(&leaves, Network::Bitcoin).to_string(),
            c.expected_xpub,
            "{}",
            c.name
        );
    }
}

#[test]
fn sorting_or_deduplicating_produces_a_different_xpub() {
    // Liana's recipe is NOT sorted and NOT deduplicated. The abandoned BIPs
    // PR #1746 recipe is, and it is a different wallet.
    let c = case("preset-kofn-recovery-tr");
    let mut l = c.leaf_pubkeys();
    let straight = liana_unspendable_xpub(&l, Network::Bitcoin).to_string();
    l.sort();
    l.dedup();
    assert_ne!(
        straight,
        liana_unspendable_xpub(&l, Network::Bitcoin).to_string()
    );
}

#[test]
fn a_synthetic_duplicate_leaf_changes_the_xpub_when_removed() {
    // `sorting_or_deduplicating_produces_a_different_xpub` above pins that
    // Liana's recipe is not sorted -- but it does NOT pin the "not
    // deduplicated" half: its `l.sort(); l.dedup();` call removes zero
    // elements, because `preset-kofn-recovery-tr`'s four leaf pubkeys are
    // already distinct. Review round 1 confirmed this by inserting a bare
    // `.dedup()` call into `liana_unspendable_xpub` itself and finding all
    // four original tests stayed green.
    //
    // NO VENDORED case can pin the dedup half, and this is not a fixture
    // gap to fill -- it is structural. md1 refuses a key slot from
    // appearing twice in ANY taptree it can express, for two independent
    // reasons: same-path reuse is forbidden by BIP 388's disjointness
    // rule, and disjoint-path reuse is forbidden by F-417's one-path-per-
    // key-slot rule (see `md encode` refusing both shapes in the task 4
    // brief). So no `cases.json` entry -- vendored today or added later --
    // will ever contain a duplicate leaf pubkey, and a test built only
    // from vendored cases structurally cannot exercise this half.
    //
    // `liana_unspendable_xpub`'s contract is about the SLICE it is
    // handed -- `&[[u8; 33]]` -- not about what md1 can express, so this
    // pins it with a slice built by hand: a real pubkey repeated
    // adjacently. Do not "fix" this by switching it to a vendored case;
    // that would silently un-pin this half again, exactly as happened
    // before this test existed.
    let a = case("preset-kofn-recovery-tr").leaf_pubkeys()[0];
    let with_duplicate = vec![a, a];
    let deduplicated = vec![a];
    assert_ne!(
        liana_unspendable_xpub(&with_duplicate, Network::Bitcoin).to_string(),
        liana_unspendable_xpub(&deduplicated, Network::Bitcoin).to_string(),
        "a duplicated leaf pubkey must change the chain code -- \
         sha256(a||a) must differ from sha256(a)"
    );
}

// Task 8 step 4 (§8.1's nested-taptree vector, I5). `preset-decaying-multisig-tr`
// is the evidence's ONLY nested taptree ({A,{B,C}}), and Liana REFUSED it on
// policy shape -- it is one of the four "accepted: false" cases in
// `cases.json`, not one of the four ACCEPT shapes. So no Liana ACCEPT backs
// the traversal order this test pins: the assertion below is a
// RECIPE-AGREEMENT pin (this xpub matches the two sibling trees' xpub, which
// share the same four keys in the same order), not a measured Liana import.
// Do not mistake it for one.
//
// §8.1 literally asks for "a nested taptree that Liana ACCEPTS". No such
// evidence exists in the vendored corpus -- the only nested case is this
// one, and it was refused, not accepted. This stage delivers the strongest
// available substitute (recipe agreement against the golden xpub) and does
// NOT deliver the thing §8.1 asks for; that is a deliberate, recorded gap,
// not an oversight. Closing it needs a NEW Liana-accepted nested shape
// measured through the harness, which is stage 2's live run (carried into
// stage 2's brief per the C3 ruling's own instruction).
#[test]
fn the_chain_code_depends_on_the_keys_not_the_tree() {
    // kofn-recovery, tiered-recovery and decaying-multisig are three
    // different trees over the same four keys in the same order and share
    // one internal key. decaying-multisig is the only NESTED taptree in
    // the evidence, so it is what proves the traversal is depth-first
    // left-to-right -- by recipe agreement, per the module-level comment
    // above, not by a Liana ACCEPT.
    let names = [
        "preset-kofn-recovery-tr",
        "preset-tiered-recovery-tr",
        "preset-decaying-multisig-tr",
    ];
    let xs: Vec<String> = names
        .iter()
        .map(|n| case(n).expected_xpub.clone())
        .collect();
    assert_eq!(xs[0], xs[1]);
    assert_eq!(xs[1], xs[2]);
}

#[test]
fn a_tpub_wallet_derives_a_tpub_internal_key() {
    // SPEC §2 step 5's testnet branch is TRANSCRIBED, not measured -- all
    // eight evidence descriptors are mainnet. This is the vector that
    // measures it.
    assert!(
        liana_unspendable_xpub(
            &case("preset-kofn-recovery-tr").leaf_pubkeys(),
            Network::Testnet
        )
        .to_string()
        .starts_with("tpub")
    );
}

// Stage 1b task 8: §8's acceptance vectors, proven over md's own vendored
// corpus (the C3 ruling moves the evidence-shape legs -- descriptor- and
// address-equality against Liana's four ACCEPT shapes -- to stage 2, which
// needs `md decompose`/`md descriptor` from md-cli; see task 8's dispatch
// brief).
//
// DISPATCH-BRIEF DEFECT, fixed here rather than worked around: the brief's
// own Step 1 snippet iterated `all_kind0_tr_vectors()` (now
// `all_root_tr_vectors()`, `tests/common/liana.rs`) and fed every name to
// `kind1_from_vector`, whose `assert_eq!(internal_key, NumsPoint)` panics on
// a `Slot`-rooted vector -- 13 of the 23 root-tr vectors in today's corpus.
// Every test below iterates `nums_rooted_tr_vectors()` instead (the new,
// correctly-scoped helper in `tests/common/liana.rs`), including the
// corpus-level item-2 test in step 5b, whose brief snippet carried the
// identical defect.

/// Step 1: round trip and version, over the NUMS-rooted corpus only (see the
/// module comment above for why NOT `all_root_tr_vectors`).
#[test]
fn every_nums_rooted_tr_vector_round_trips_at_kind_1_and_reports_version_8() {
    let vectors = nums_rooted_tr_vectors();
    // Derived from the live corpus, never hardcoded (today: 10). A filter
    // regression that starts matching nothing must fail LOUD here, not read
    // as `ok` -- "a filtered test run that matches nothing says ok" is
    // exactly the trap this assertion exists to close, and it sits directly
    // under task 9's mutation pass, which depends on this loop actually
    // running.
    let expected = vectors.len();
    assert!(
        expected > 0,
        "no NUMS-rooted tr vectors in the corpus -- nums_rooted_tr_vectors() is broken"
    );

    let mut iterated = 0usize;
    let mut round_tripped = 0usize;
    for name in &vectors {
        iterated += 1;
        let d = kind1_from_vector(name);
        assert_eq!(d.wire_version(), 8, "{name}");
        let (bytes, bits) = match encode_payload(&d) {
            Ok(v) => v,
            // SPEC §6 row 1 ONLY: a sortedmulti_a leaf
            // (`keyed_compose_tr_sole_sortedmulti_a` is the sole vector that
            // trips this today). §6's other three rows (non-canonical
            // use-site, non-root tr, non-minimal version) are NOT tolerated
            // here -- the brief's own staleness check found the corpus has
            // no non-canonical-use-site tr vector and every tr vector here
            // is already root, so those rows are unreachable from this loop
            // TODAY, by luck of the corpus rather than by design. Any OTHER
            // refusal must panic loudly rather than being silently skipped,
            // so a future vector that trips one of those rows is caught
            // here instead of quietly narrowing what this test proves.
            Err(Error::UnspendableWithSortedMultiA) => continue,
            Err(e) => panic!(
                "{name}: unexpected refusal {e:?} -- if this is a legitimate \
                 new §6 refusal reachable from the vendored corpus, add it \
                 to the tolerated set above explicitly"
            ),
        };
        assert_eq!(decode_payload(&bytes, bits).unwrap(), d, "{name}");
        round_tripped += 1;
    }
    assert_eq!(
        iterated, expected,
        "the loop must visit every vector nums_rooted_tr_vectors() derived"
    );
    assert!(
        round_tripped > 0,
        "every vector was skipped -- the round-trip property was never exercised"
    );
}

/// The mk1 `policy_id_stub` truncation, reproduced here bit-for-bit from
/// `crates/md-cli/src/seat/disposition.rs:73-74`'s private `top4` (a
/// different crate, and not `pub`, so it cannot be imported): the first 4
/// bytes of a 16-byte [`md_codec::identity::WalletPolicyId`] or
/// [`md_codec::identity::WalletDescriptorTemplateId`].
fn top4(bytes: &[u8; 16]) -> [u8; 4] {
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

/// Step 2: identity distinctness AND stability.
///
/// Review round 1, I1: SPEC §8.5 requires FOUR distinctness properties for
/// kind 0 vs kind 1, and the first draft of this test asserted three
/// (policy id, template id, phrase). The fourth -- a different mk1
/// `policy_id_stub` -- does NOT follow from the other three: distinct
/// 16-byte ids do not imply distinct 4-byte truncations of them (a
/// birthday-adjacent collision at `top4` is a real, independent risk even
/// when the full ids differ), and SPEC §3e names the consequence directly
/// -- an mk1 KEY card false-matching the OTHER kind's plates via
/// `dispositions`'s `policy_id_stubs.contains(&wallet_or_shape)` check.
/// Both stub FLAVOURS get their own assertion: the wallet stub (from
/// `WalletPolicyId`) and the shape stub (from `WalletDescriptorTemplateId`)
/// are independent truncations and neither implies the other.
#[test]
fn kind_0_and_kind_1_get_different_ids_and_a_different_phrase() {
    let k0 = decode_vendored(&load_vendored_phrase("keyed_compose_tr_nums_three_leaves")).unwrap();
    let k1 = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    assert_ne!(
        compute_wallet_policy_id(&k0).unwrap(),
        compute_wallet_policy_id(&k1).unwrap()
    );
    assert_ne!(
        compute_wallet_descriptor_template_id(&k0).unwrap(),
        compute_wallet_descriptor_template_id(&k1).unwrap()
    );
    assert_ne!(
        compute_wallet_policy_id(&k0)
            .unwrap()
            .to_phrase()
            .unwrap()
            .to_string(),
        compute_wallet_policy_id(&k1)
            .unwrap()
            .to_phrase()
            .unwrap()
            .to_string()
    );
    // The fourth property (I1): the mk1 policy_id_stub, both flavours.
    assert_ne!(
        top4(compute_wallet_policy_id(&k0).unwrap().as_bytes()),
        top4(compute_wallet_policy_id(&k1).unwrap().as_bytes()),
        "kind 0 and kind 1 must not share a wallet-confirmed policy_id_stub"
    );
    assert_ne!(
        top4(
            compute_wallet_descriptor_template_id(&k0)
                .unwrap()
                .as_bytes()
        ),
        top4(
            compute_wallet_descriptor_template_id(&k1)
                .unwrap()
                .as_bytes()
        ),
        "kind 0 and kind 1 must not share a shape-confirmed policy_id_stub"
    );
}

/// Golden-file shape for stage 1a's committed `golden/pre_refactor_ids.json`:
/// `{"count": 65, "vectors": [[name, policy_id_hex, template_id_hex,
/// phrase], ...]}`. Defined here (not imported) -- no earlier stage shipped
/// a Rust type for this JSON shape.
#[derive(serde::Deserialize)]
struct IdGolden {
    count: usize,
    vectors: Vec<(String, String, String, String)>,
}

/// Step 2: stage 1a's committed golden, 65 4-tuples, byte-preserved. Do NOT
/// regenerate it -- it is the pre-refactor baseline this stage must not
/// silently move.
#[test]
fn every_existing_v4_identity_is_byte_preserved() {
    let g: IdGolden = serde_json::from_str(include_str!("golden/pre_refactor_ids.json")).unwrap();
    assert_eq!(
        g.count,
        g.vectors.len(),
        "golden file's own count field is stale"
    );
    assert_eq!(g.vectors.len(), 65, "golden file no longer has 65 entries");
    for (name, policy, template, phrase) in &g.vectors {
        let d = decode_vendored(&load_vendored_phrase(name)).unwrap();
        // Neither id type implements `Display` -- `hex::encode(.as_bytes())`
        // is exactly how `examples/dump_ids.rs` produced this golden file in
        // the first place; matching its own encoding is the point.
        assert_eq!(
            hex::encode(compute_wallet_policy_id(&d).unwrap().as_bytes()),
            *policy,
            "{name}"
        );
        assert_eq!(
            hex::encode(
                compute_wallet_descriptor_template_id(&d)
                    .unwrap()
                    .as_bytes()
            ),
            *template,
            "{name}"
        );
        assert_eq!(
            &compute_wallet_policy_id(&d)
                .unwrap()
                .to_phrase()
                .unwrap()
                .to_string(),
            phrase,
            "{name}"
        );
    }
}

/// Step 3: SPEC §8.4's Rust leg -- the dispatch round trip, through
/// `decode_with_correction` (single-string AND chunked), not
/// `decode_payload`, which bypasses the auto-dispatch that made version 5
/// unusable (`wire_version_8.rs`'s own tests pin that regression directly).
///
/// `chunk::split(&d)` ALWAYS wraps every chunk with a chunk header whose
/// chunked-flag bit is hardcoded to 1 (`ChunkHeader::write`,
/// `w.write_bits(1, 1); // chunked = 1`) -- even for a single-chunk set --
/// so its output always exercises the CHUNKED leg of the dispatch,
/// including the "chunked-of-1" case. `encode_md1_string(&d)` is the
/// single-payload encoder (chunked-flag bit 0); it refuses with
/// `Error::PayloadTooLongForSingleString` for any vector whose payload
/// exceeds the single-string symbol cap -- MEASURED: several NUMS-rooted
/// vectors do (e.g. `compose_tr_thirty_two_slots`'s 32 slots need 304
/// symbols against an 80-symbol cap) -- so that leg is tried per-vector and
/// only counted when it actually fits, while the chunked leg (which has no
/// such cap short of `Error::ChunkCountExceedsMax`) runs unconditionally.
/// Both counters are asserted non-zero so neither leg can go silently
/// unexercised across the whole corpus.
#[test]
fn every_nums_rooted_tr_vector_round_trips_through_the_dispatch_at_kind_1() {
    let vectors = nums_rooted_tr_vectors();
    assert!(
        !vectors.is_empty(),
        "no NUMS-rooted tr vectors in the corpus"
    );

    let mut single_string_tested = 0usize;
    let mut chunked_tested = 0usize;
    for name in &vectors {
        let d = kind1_from_vector(name);
        // Review round 1, M2: this used to blanket-skip on ANY encode error
        // while claiming step 1's explicit tolerated set in its comment --
        // the comment was aspirational, not what the code did. Matched now:
        // tolerate ONLY §6 row 1, panic on anything else, same as step 1.
        match encode_payload(&d) {
            Ok(_) => {}
            Err(Error::UnspendableWithSortedMultiA) => continue,
            Err(e) => panic!(
                "{name}: unexpected refusal {e:?} -- if this is a legitimate \
                 new §6 refusal reachable from the vendored corpus, add it \
                 to the tolerated set explicitly"
            ),
        }

        // Single-string leg -- only when the payload actually fits.
        match md_codec::encode_md1_string(&d) {
            Ok(single) => {
                let (decoded_single, details_single) =
                    decode_with_correction(&[single.as_str()]).unwrap();
                assert!(
                    details_single.is_empty(),
                    "{name}: unexpected correction on a freshly minted single-string card"
                );
                assert_eq!(decoded_single, d, "{name}: single-string dispatch");
                single_string_tested += 1;
            }
            Err(Error::PayloadTooLongForSingleString { .. }) => {}
            Err(e) => panic!("{name}: unexpected single-string encode error {e:?}"),
        }

        // Chunked leg -- unconditional.
        let chunks = chunk::split(&d).unwrap();
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        let (decoded_chunked, details_chunked) = decode_with_correction(&refs).unwrap();
        assert!(
            details_chunked.is_empty(),
            "{name}: unexpected correction on a freshly minted chunked card"
        );
        assert_eq!(decoded_chunked, d, "{name}: chunked dispatch");
        chunked_tested += 1;
    }
    assert!(
        single_string_tested > 0,
        "no vector exercised the single-string dispatch leg"
    );
    assert!(
        chunked_tested > 0,
        "no vector exercised the chunked dispatch leg"
    );
}

/// Step 5b, item 2: descriptor equality at the CORPUS level. The C3 ruling
/// moved the evidence-shape leg (kind-1 descriptor vs Liana's own
/// `descriptor_with_checksum`) to stage 2, which needs `md decompose`/`md
/// descriptor` -- but promised a corpus-level replacement here, which a
/// prior fold silently dropped. Restored: the property provable in 1b is
/// that a kind-1 descriptor renders STABLY and re-renders IDENTICALLY --
/// md vs md, not md vs Liana.
///
/// `to_miniscript_descriptor_multipath_with_network` needs a REAL key per
/// `@N` (`Error::MissingPubkey` otherwise) -- a vendored vector, decoded
/// from a keyless md1 template, carries none. Synthetic xpubs from
/// `common::test_xpubs()` (this crate's own `tests/common/mod.rs`, the same
/// pool `descriptor_with_pubkeys` uses) fill every slot; the KEYS are
/// synthetic, but the property under test is renderer stability, not what
/// address the keys resolve to.
#[test]
fn item_2_descriptor_equality_at_the_corpus_level() {
    let vectors = nums_rooted_tr_vectors();
    assert!(
        !vectors.is_empty(),
        "no NUMS-rooted tr vectors in the corpus"
    );

    let mut rendered = 0usize;
    for name in &vectors {
        let mut d = kind1_from_vector(name);
        // Review round 1, M2: same fix as the dispatch test above -- tolerate
        // ONLY §6 row 1, panic on anything else, not a blanket `.is_err()`.
        match encode_payload(&d) {
            Ok(_) => {}
            Err(Error::UnspendableWithSortedMultiA) => continue,
            Err(e) => panic!(
                "{name}: unexpected refusal {e:?} -- if this is a legitimate \
                 new §6 refusal reachable from the vendored corpus, add it \
                 to the tolerated set explicitly"
            ),
        }
        d.tlv.pubkeys = Some(
            (0..d.n)
                .map(|i| (i, common::test_xpubs()[i as usize]))
                .collect(),
        );
        let a = to_miniscript_descriptor_multipath_with_network(&d, Network::Bitcoin)
            .unwrap()
            .to_string();
        let b = to_miniscript_descriptor_multipath_with_network(&d, Network::Bitcoin)
            .unwrap()
            .to_string();
        assert_eq!(a, b, "{name}");
        assert!(
            a.contains('#'),
            "{name}: the BIP-380 checksum is part of the gate"
        );
        rendered += 1;
    }
    assert!(
        rendered > 0,
        "every vector was skipped -- descriptor-render stability was never exercised"
    );
}

// Stage 1b task 7: SPEC §6's ENCODE-side refusals for a wire-kind-1 (Liana
// unspendable) internal key. Two of the four live here, because they need
// only `encode_payload` (public) and `kind1_from_vector`
// (`tests/common/liana.rs`, spliced in above) -- the other two need
// crate-private surface only reachable from a unit test inside `src/`
// (`crates/md-codec/src/encode.rs`'s `unspendable_shape_tests` module).

/// A kind-1 tr whose taptree contains a `sortedmulti_a` leaf -- the shape §6
/// row 1 refuses. VERIFIED SHAPE: `keyed_tr_sortedmulti_a` is NOT this, it is
/// `tr(@0/...,sortedmulti_a(...))` with a SPENDABLE internal key, so
/// `kind1_from_vector`'s kind-0 assert would panic on it. The NUMS-rooted
/// vector is `keyed_compose_tr_sole_sortedmulti_a`:
/// `tr({NUMS},sortedmulti_a(2,@0,@1,@2))`, use-site `<0;1>/*` (its own
/// `.descriptor.json` confirms both), so `kind1_from_vector`'s swap is legal
/// and this fixture trips ONLY row 1, never row 2.
fn tr_liana_with_sortedmulti_a_leaf() -> Descriptor {
    kind1_from_vector("keyed_compose_tr_sole_sortedmulti_a")
}

/// A kind-1 tr whose slots sit at a non-canonical use site. md1 carries ONE
/// path per slot (F-417), so this is a whole-descriptor change, not
/// per-leaf. `keyed_compose_tr_nums_three_leaves`'s own `.descriptor.json`
/// confirms its taptree has no `sortedmulti_a` leaf (its multi-family node is
/// `MultiA`, not `SortedMultiA`) and its own use-site is already the
/// canonical `<0;1>/*`, so overriding it below is what makes this fixture
/// trip ONLY row 2, never row 1.
///
/// `UseSitePath` has NO `parse` and NO `FromStr` -- its surface is
/// `standard_multipath()`, `write()`, `read()` (`use_site_path.rs:70-78`).
/// Build it by struct literal; the fields are public.
fn tr_liana_at_use_site(a: u32, b: u32) -> Descriptor {
    let mut d = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    d.use_site_path = UseSitePath {
        multipath: Some(vec![
            Alternative {
                hardened: false,
                value: a,
            },
            Alternative {
                hardened: false,
                value: b,
            },
        ]),
        wildcard_hardened: false,
    };
    d
}

/// A kind-1 tr whose SHARED use-site stays the canonical `<0;1>/*`, but a
/// single `@N`'s PER-KEY override (`d.tlv.use_site_path_overrides`)
/// diverges -- fix round 1's finding: the whole-descriptor check
/// (`tr_liana_at_use_site` above) cannot see this at all, because
/// `d.use_site_path` is untouched. MEASURED through the operator CLI
/// before the fix: `md encode 'tr(UNSPENDABLE(liana),
/// {pk(@0/<0;1>/*),pk(@1/<2;3>/*)})' --path bip48` minted, and `md decode`
/// showed the `@1` override intact. `parse/template.rs:843-858` builds
/// `use_site_path_overrides` straight from a per-placeholder path, so this
/// is reachable from a plain template, not a hand-crafted wire.
///
/// `keyed_compose_tr_nums_three_leaves` has `use_site_path_overrides: null`
/// (its own `.descriptor.json` confirms it), so starting from
/// `kind1_from_vector` and adding exactly one override entry is what
/// isolates this shape from `tr_liana_at_use_site`'s whole-descriptor one.
fn tr_liana_with_noncanonical_override(idx: u8, a: u32, b: u32) -> Descriptor {
    let mut d = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    d.tlv.use_site_path_overrides = Some(vec![(
        idx,
        UseSitePath {
            multipath: Some(vec![
                Alternative {
                    hardened: false,
                    value: a,
                },
                Alternative {
                    hardened: false,
                    value: b,
                },
            ]),
            wildcard_hardened: false,
        },
    )]);
    d
}

/// A kind-1 `tr()` nested inside `wsh(...)` -- the shape §6 row 4 refuses.
/// `wsh(tr(...))` is not a real BIP-380 shape and `kind1_from_vector` has no
/// vendored vector to swap (every vendored `tr` is a ROOT `tr`), so this is
/// hand-built from `Node`/`Body` directly, same as
/// `crates/md-codec/src/encode.rs`'s in-crate fixtures. Structurally
/// encodable regardless: `write_node`'s `Body::Children` arm recurses
/// generically over any child tag, and the root-tag allow-list
/// (`Sh|Wsh|Wpkh|Pkh|Tr`) is a DECODE-side check (`decode.rs:97-104`), not an
/// encode-side one -- this fixture must be refused before it ever reaches
/// that check.
fn wsh_wrapping_tr_liana() -> Descriptor {
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
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
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    internal_key: InternalKey::LianaUnspendable,
                    tree: Some(Box::new(Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    })),
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    }
}

#[test]
fn kind_1_with_a_sortedmulti_a_leaf_is_refused_at_mint() {
    let d = tr_liana_with_sortedmulti_a_leaf();
    assert!(matches!(
        encode_payload(&d),
        Err(Error::UnspendableWithSortedMultiA)
    ));
}

#[test]
fn kind_1_off_the_canonical_use_site_is_refused() {
    // F-638: the SHARED use-site diverged, so there is no one key to name.
    assert!(matches!(
        encode_payload(&tr_liana_at_use_site(2, 3)),
        Err(Error::UnspendableUseSiteNotCanonical { idx: None })
    ));
}

#[test]
fn kind_1_nested_under_wsh_is_refused() {
    assert!(matches!(
        encode_payload(&wsh_wrapping_tr_liana()),
        Err(Error::UnspendableNotRootTr)
    ));
}

/// Fix round 1: a non-canonical PER-KEY override must be refused even when
/// the shared `use_site_path` is canonical. `tr_liana_at_use_site` above
/// pins the whole-descriptor half of row 2; this pins the override half --
/// the two are independent code paths in `validate_unspendable_shape` and
/// each needs its own test, or one half has no gate (see mutation
/// verification in the task 7 report).
#[test]
fn kind_1_with_a_noncanonical_per_key_override_is_refused() {
    // F-638: the override half carries the placeholder that diverged.
    assert!(matches!(
        encode_payload(&tr_liana_with_noncanonical_override(1, 2, 3)),
        Err(Error::UnspendableUseSiteNotCanonical { idx: Some(1) })
    ));
}

/// F-638 (F-449 stage 2 Task 6): from the override half the message used to
/// describe the SHARED use-site -- a field the operator could see was
/// correct -- and named no key. It now names the placeholder, and says the
/// shared use-site is not the problem. The shared-half wording is unchanged.
#[test]
fn the_use_site_refusal_names_the_placeholder_that_diverged() {
    let shared = Error::UnspendableUseSiteNotCanonical { idx: None }.to_string();
    let over = Error::UnspendableUseSiteNotCanonical { idx: Some(1) }.to_string();
    for m in [&shared, &over] {
        assert!(
            m.contains("requires the canonical <0;1>/* use-site path"),
            "{m}"
        );
    }
    assert!(
        !shared.contains('@'),
        "the shared half has no one key to name: {shared}"
    );
    assert!(over.contains("@1"), "must name the placeholder: {over}");
    assert!(over.contains("shared use-site is canonical"), "{over}");
}
