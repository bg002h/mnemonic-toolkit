// Liana evidence fixture + kind-1 descriptor builder, spliced into several
// crate roots the same way `vendored.rs` is (see its own header comment for
// why: `examples/` and `tests/` are separate crate roots, and `use` across
// them does not compile). Pulled in with
// `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/liana.rs"))`.
//
// This file assumes `tests/common/vendored.rs` has ALREADY been spliced in
// above it at the call site -- it uses `load_vendored_phrase`,
// `decode_vendored` and `all_vendored_vector_names` from there and does not
// redefine them (stage 1a shipped those; this file is stage 1b's own).
//
// The evidence itself lives in ANOTHER repository (mnemonic-engrave) and is
// vendored into `tests/fixtures/liana/cases.json` by
// `scripts/vendor-liana-evidence.sh`, not read live from this file.

// `Descriptor` is already in scope from `vendored.rs`, spliced in above this
// file at the call site -- importing it again here would collide (E0252,
// vendored.rs's own header comment explains it). `Body`/`InternalKey` are
// NOT re-exported from the `md_codec` crate root (only `Tag` is; see
// `crates/md-codec/tests/address_derivation.rs:19` for the same path), so
// they need their own `use` here and a consumer root must not re-import
// them either.
use md_codec::tree::{Body, InternalKey};

/// A real kind-1 Descriptor, built by decoding a vendored kind-0 vector and
/// swapping only the internal key. A Descriptor cannot be built from
/// `cases.json`'s TLV bytes alone -- it would have no taptree, no older(),
/// no fingerprints and no divergent origins, all of which a byte-identical
/// descriptor gate needs.
///
/// `#[allow(dead_code)]`: this file is spliced into several crate roots
/// (see the header comment), and not every consumer needs a kind-1
/// Descriptor -- e.g. a root that only checks `case(name).leaf_pubkeys()`
/// against the recipe never calls this, so it is dead code in that
/// compilation unit specifically.
#[allow(dead_code)]
fn kind1_from_vector(vector_name: &str) -> Descriptor {
    let mut d = decode_vendored(&load_vendored_phrase(vector_name))
        .unwrap_or_else(|e| panic!("{vector_name}: decode: {e}"));
    match &mut d.tree.body {
        Body::Tr { internal_key, .. } => {
            assert_eq!(
                *internal_key,
                InternalKey::NumsPoint,
                "{vector_name}: fixture must start at kind 0"
            );
            *internal_key = InternalKey::LianaUnspendable;
        }
        _ => panic!("{vector_name} is not a tr descriptor"),
    }
    d
}

/// One vendored Liana evidence case -- see `scripts/vendor-liana-evidence.sh`
/// for exactly how each field was extracted.
///
/// `#[allow(dead_code)]`: this file is spliced into several crate roots
/// (see the header comment), and different roots read different fields --
/// e.g. a descriptor-equality root reads `descriptor_with_checksum` and
/// never touches `liana_receive`/`liana_change`, which an address-equality
/// root reads instead. No single compilation unit is expected to read
/// every field, so some are dead code in any one of them specifically.
#[allow(dead_code)]
#[derive(serde::Deserialize, Clone)]
struct Case {
    name: String,
    accepted: bool,
    leaf_tlv_hex: Vec<String>,     // 65-byte chain code || compressed pubkey
    leaf_pubkeys_hex: Vec<String>, // the 33-byte slices at [32..65]
    expected_xpub: String,
    descriptor_with_checksum: String,
    liana_receive: Vec<String>,
    liana_change: Vec<String>,
}

impl Case {
    /// The 33-byte compressed pubkeys SPEC section 2 hashes, in wire order.
    ///
    /// `#[allow(dead_code)]`: not every spliced-in consumer calls this --
    /// e.g. a root that only asserts `descriptor_with_checksum` or the
    /// `liana_receive`/`liana_change` addresses never decodes
    /// `leaf_pubkeys_hex`, so this method is dead code in that
    /// compilation unit specifically.
    #[allow(dead_code)]
    fn leaf_pubkeys(&self) -> Vec<[u8; 33]> {
        self.leaf_pubkeys_hex
            .iter()
            .map(|h| {
                <[u8; 33]>::try_from(hex::decode(h).expect("hex").as_slice()).expect("33 bytes")
            })
            .collect()
    }
}

/// Every vendored case -- all EIGHT, including the four Liana refused on
/// policy shape: section 2's recipe is correct for those too, and one of
/// them is the only nested taptree in the evidence.
///
/// `#[allow(dead_code)]`: not every spliced-in consumer needs the whole
/// set -- e.g. a root exercising a single named case goes through `case`
/// below (which calls this internally) rather than iterating the corpus
/// itself, so this function has no direct caller in that compilation unit
/// specifically. (And in the other direction, a root that iterates
/// `all_cases()` directly, rather than looking up cases by name, leaves
/// `case` below with no caller -- hence its own `#[allow(dead_code)]`,
/// not laziness.)
#[allow(dead_code)]
fn all_cases() -> Vec<Case> {
    serde_json::from_str(include_str!("../fixtures/liana/cases.json")).expect("cases.json")
}

#[allow(dead_code)]
fn case(name: &str) -> Case {
    all_cases()
        .into_iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no vendored case {name}"))
}

/// The vendored vector names whose decoded tree is a root `tr`, at EITHER
/// internal-key shape -- a NUMS-point root, or a real spendable `Slot` root
/// (the multisig "extracted to the internal key" shape). Measured today: 23
/// of the 65 vectors carry `Body::Tr`.
///
/// Renamed from `all_kind0_tr_vectors` (task 8's dispatch brief, stage 1b):
/// that name was FALSE for the `Slot`-rooted vectors -- a `Slot` internal
/// key has no "kind" at all (the wire kind bit is only ever written for the
/// `is_nums=1` arm, i.e. `NumsPoint`/`LianaUnspendable`; see `tree.rs`'s
/// `write_node`), so calling a keyed-root vector "kind 0" claimed something
/// untrue of it. Measured directly against this function: 10 of the 23 are
/// `NumsPoint`-rooted, 13 are `Slot`-rooted (a throwaway probe test, run
/// then deleted -- see task 8's report for the transcript).
///
/// `wire_version_8.rs`'s `wire_version_is_derived_from_the_tree_not_assumed`
/// is this function's one caller, and it asserts `wire_version() == 4`,
/// which is true for BOTH internal-key shapes -- so it keeps iterating this
/// full population under its new name, unchanged. `nums_rooted_tr_vectors`
/// below is the DIFFERENT, narrower population task 8 needs: only the
/// vectors `kind1_from_vector` can legally swap to kind 1.
///
/// `#[allow(dead_code)]`: not every spliced-in consumer calls this -- e.g.
/// a root that only exercises named cases via `case`/`kind1_from_vector`
/// never needs the full root-tr population, so this function is dead code
/// in that compilation unit specifically.
#[allow(dead_code)]
fn all_root_tr_vectors() -> Vec<String> {
    all_vendored_vector_names()
        .into_iter()
        .filter(|n| {
            matches!(
                decode_vendored(&load_vendored_phrase(n)).map(|d| d.tree.body),
                Ok(Body::Tr { .. })
            )
        })
        .collect()
}

/// The vendored vector names whose decoded tree is a root `tr` with a
/// `NumsPoint` internal key -- the population `kind1_from_vector` can
/// legally swap to `LianaUnspendable` (its `assert_eq!(internal_key,
/// NumsPoint)` panics on anything else, including a `Slot`-rooted vector
/// from `all_root_tr_vectors` above). Task 8's dispatch brief measured "8"
/// against a DIFFERENT, smaller population (the `ik:nums`-tagged subset of
/// `compose_support.rs`'s 13-entry `w:tr`-tagged family table) -- this
/// function filters the FULL vendored corpus instead, which is what
/// `all_root_tr_vectors` (and the old, misnamed `all_kind0_tr_vectors`)
/// actually iterates, and measures 10, not 8. Callers derive their expected
/// count from THIS function's live output, never a hardcoded number, so a
/// filter regression that starts matching nothing fails loudly instead of
/// reading as `ok`.
#[allow(dead_code)]
fn nums_rooted_tr_vectors() -> Vec<String> {
    all_root_tr_vectors()
        .into_iter()
        .filter(|n| {
            matches!(
                decode_vendored(&load_vendored_phrase(n)).map(|d| d.tree.body),
                Ok(Body::Tr {
                    internal_key: InternalKey::NumsPoint,
                    ..
                })
            )
        })
        .collect()
}
