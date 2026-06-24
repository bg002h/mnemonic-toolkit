//! Generator for the canonical mk-codec vector corpus.
//!
//! The output corpus carries the family token `crate::consts::GENERATOR_FAMILY`
//! (`"mk-codec X.Y"` per closure Q-10), which rolls on minor-version
//! bumps but not patches. The on-disk filename
//! (`crates/mk-codec/src/test_vectors/v0.1.json`) is intentionally stable
//! across token rolls — see `tests/vectors.rs::VECTOR_FILE` for the
//! filename-vs-family-token convention.
//!
//! Run via:
//!
//! ```text
//! cargo run --bin gen_mk_vectors --features gen-vectors -- \
//!   --output crates/mk-codec/src/test_vectors/v0.1.json
//! ```
//!
//! Cross-implementations validate against the JSON file's pinned
//! SHA-256 (in `crates/mk-codec/tests/vectors.rs`) plus per-vector
//! round-trip equality. The output is byte-deterministic in the
//! fixture set so re-runs produce identical files; see the
//! "Canonicality discipline" block below for the rules.
//!
//! ## Canonicality discipline
//!
//! - Keys sorted alphabetically at every nesting level — `serde_json::Map`
//!   is `BTreeMap`-backed by default, which sorts on insertion.
//! - Hex literals lowercase — emitted via the `lowercase_hex` helper here
//!   (the `bitcoin::hashes` crate's hex encoders are also lowercase by
//!   default but re-implementing here keeps the dependency surface small).
//! - Byte-array fields rendered as continuous hex strings (no `0x` prefix,
//!   no separators).
//! - Indentation: 2 spaces — `serde_json::ser::PrettyFormatter::with_indent(b"  ")`.
//! - Line endings: LF; trailing newline at EOF — appended manually.
//! - Per-vector `chunk_set_id` is fixed so chunked encodings are
//!   byte-deterministic.

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use mk_codec::string_layer::bch::{bytes_to_5bit, encode_5bit_to_string};
use mk_codec::string_layer::chunk::split_into_chunks;
use mk_codec::string_layer::header::{StringLayerHeader, VERSION_V0_1};
use mk_codec::{KeyCard, bytecode::encode_bytecode, encode_with_chunk_set_id};
use serde_json::{Value, json};

/// One fixture spec — abstract enough to drop a new vector by adding
/// one entry to [`fixtures`] without touching the emit code below.
struct FixtureSpec {
    /// Vector identifier (e.g., `V1_bip48_mainnet_1_stub`, `N1_invalid_hrp_bt`).
    name: &'static str,
    /// One-line human-readable description of what the vector exercises.
    description: &'static str,
    /// What kind of vector this is — clean (round-trip succeeds) or
    /// negative (decoder must reject with a specific `Error` variant).
    kind: FixtureKind,
}

/// Per-vector input shape. v0.1.1 schema 2 supports two kinds:
///
/// - [`FixtureKind::Clean`] — full `KeyCard` input; round-trip exercises
///   `encode → strings → decode → KeyCard` against pinned bytecode + strings.
/// - [`FixtureKind::Negative`] — pre-built mk1 strings; decoder must
///   reject with the pinned `expected_error` `Display` rendering. The
///   strings are constructed bottom-up (or by mutating a clean encoding
///   at a specific layer) to bypass encoder-side validation; no
///   conforming mk-codec encoder would emit them.
enum FixtureKind {
    Clean(CleanInput),
    Negative(NegativeInput),
}

struct CleanInput {
    /// Policy ID stub bytes (4 bytes per stub; min 1 stub per closure §4 rule 3).
    policy_id_stubs: Vec<[u8; 4]>,
    /// Master-key fingerprint, or `None` for the privacy-preserving mode
    /// (closure Q-8 — bytecode-header bit 2 cleared).
    origin_fingerprint: Option<[u8; 4]>,
    /// BIP 32 origin path. Must round-trip through `DerivationPath::from_str`
    /// and serialize identically via `Display`.
    origin_path: &'static str,
    /// Mainnet (`NetworkKind::Main`) vs testnet (`NetworkKind::Test`).
    /// Affects the xpub.version field in the compact-73 wire form.
    network: NetworkKind,
    /// Seed byte used to deterministically derive the synthetic
    /// secret key. Distinct seeds produce distinct xpubs across the
    /// vector set, which makes vector inspection tractable when one
    /// fails.
    seed_byte: u8,
    /// Pinned `chunk_set_id` for byte-deterministic chunked encoding.
    /// Per closure Q-5 the 20-bit field is opaque; vectors use
    /// memorable hex digits (0x12345, 0xABCDE, …) rather than zero
    /// to make hand-debugging easier.
    chunk_set_id: u32,
}

struct NegativeInput {
    /// The mk1 strings to feed to `mk_codec::decode`. Empty list is
    /// permitted (used by `N23_empty_input`).
    input_strings: Vec<String>,
    /// The expected `Error::Display` rendering. Pinned byte-exact;
    /// downstream conformance harnesses match against this string.
    expected_error: String,
    /// One-line rationale of what malformation the input contains.
    why: &'static str,
}

/// Helper macro to declare a clean fixture using the same flat-field
/// shape the v0.1.0 corpus generator used. Equivalent to a `FixtureSpec`
/// literal whose `kind` is `FixtureKind::Clean(CleanInput { ... })`;
/// existence keeps the V1..V17 fixture-table call sites compact.
macro_rules! clean_fixture {
    (
        name: $name:literal,
        description: $description:expr,
        policy_id_stubs: $stubs:expr,
        origin_fingerprint: $fp:expr,
        origin_path: $path:literal,
        network: $network:expr,
        seed_byte: $seed:literal,
        chunk_set_id: $csid:literal $(,)?
    ) => {
        FixtureSpec {
            name: $name,
            description: $description,
            kind: FixtureKind::Clean(CleanInput {
                policy_id_stubs: $stubs,
                origin_fingerprint: $fp,
                origin_path: $path,
                network: $network,
                seed_byte: $seed,
                chunk_set_id: $csid,
            }),
        }
    };
}

fn fixtures() -> Vec<FixtureSpec> {
    vec![
        clean_fixture! {
            name: "V1_bip48_mainnet_1_stub_with_fp",
            description: "1-stub mainnet, BIP 48 segwit-v0 multisig (m/48'/0'/0'/2'), \
                 fingerprint present. Typical multisig recovery card.",
            policy_id_stubs: vec![[0x11, 0x22, 0x33, 0x44]],
            origin_fingerprint: Some([0xAA, 0xBB, 0xCC, 0xDD]),
            origin_path: "48'/0'/0'/2'",
            network: NetworkKind::Main,
            seed_byte: 0x01,
            chunk_set_id: 0x12345,
        },
        clean_fixture! {
            name: "V2_bip84_mainnet_1_stub_with_fp",
            description: "1-stub mainnet, BIP 84 native-segwit single-sig (m/84'/0'/0'), \
                 fingerprint present. Std-table indicator 0x03.",
            policy_id_stubs: vec![[0xC0, 0xFF, 0xEE, 0x00]],
            origin_fingerprint: Some([0xDE, 0xAD, 0xBE, 0xEF]),
            origin_path: "84'/0'/0'",
            network: NetworkKind::Main,
            seed_byte: 0x02,
            chunk_set_id: 0x23456,
        },
        clean_fixture! {
            name: "V3_bip48_testnet_1_stub_with_fp",
            description: "1-stub testnet, BIP 48 testnet multisig (m/48'/1'/0'/2'), \
                 fingerprint present. Std-table indicator 0x15.",
            policy_id_stubs: vec![[0x77, 0x88, 0x99, 0xAA]],
            origin_fingerprint: Some([0x10, 0x20, 0x30, 0x40]),
            origin_path: "48'/1'/0'/2'",
            network: NetworkKind::Test,
            seed_byte: 0x03,
            chunk_set_id: 0x34567,
        },
        clean_fixture! {
            name: "V4_bip84_mainnet_1_stub_no_fp",
            description: "1-stub mainnet, BIP 84 (m/84'/0'/0'), fingerprint omitted \
                 (privacy-preserving mode; bytecode-header bit 2 cleared).",
            policy_id_stubs: vec![[0xAB, 0xCD, 0xEF, 0x01]],
            origin_fingerprint: None,
            origin_path: "84'/0'/0'",
            network: NetworkKind::Main,
            seed_byte: 0x04,
            chunk_set_id: 0x45678,
        },
        clean_fixture! {
            name: "V5_explicit_path_4_components_with_fp",
            description: "1-stub mainnet, explicit-path m/9999'/1234'/56'/7' (forces \
                 the 0xFE explicit-path codec), fingerprint present.",
            policy_id_stubs: vec![[0x55, 0x66, 0x77, 0x88]],
            origin_fingerprint: Some([0x01, 0x02, 0x03, 0x04]),
            origin_path: "9999'/1234'/56'/7'",
            network: NetworkKind::Main,
            seed_byte: 0x05,
            chunk_set_id: 0x56789,
        },
        clean_fixture! {
            name: "V6_3_stubs_mainnet_with_fp",
            description: "3-stub mainnet, BIP 48 multisig — exercises multi-stub \
                 listing that grows the bytecode by 2 × 4 bytes vs V1.",
            policy_id_stubs: vec![
                [0xDE, 0xAD, 0x00, 0x01],
                [0xDE, 0xAD, 0x00, 0x02],
                [0xDE, 0xAD, 0x00, 0x03],
            ],
            origin_fingerprint: Some([0xF0, 0x0D, 0xCA, 0xFE]),
            origin_path: "48'/0'/0'/2'",
            network: NetworkKind::Main,
            seed_byte: 0x06,
            chunk_set_id: 0x67890,
        },
        clean_fixture! {
            name: "V7_max_path_components_no_fp",
            description: "1-stub mainnet, explicit-path at the 10-component cap \
                 (m/0'/1'/2'/3'/4'/5'/6'/7'/8'/9'), fingerprint omitted. \
                 Boundary case for path-cap validation (closure Q-3).",
            policy_id_stubs: vec![[0x90, 0x91, 0x92, 0x93]],
            origin_fingerprint: None,
            origin_path: "0'/1'/2'/3'/4'/5'/6'/7'/8'/9'",
            network: NetworkKind::Main,
            seed_byte: 0x07,
            chunk_set_id: 0x78901,
        },
        clean_fixture! {
            name: "V8_bip87_mainnet_1_stub_with_fp",
            description: "1-stub mainnet, BIP 87 multisig (m/87'/0'/0'), \
                 fingerprint present. Std-table indicator 0x07 (the last \
                 mainnet entry of the closure-locked path dictionary).",
            policy_id_stubs: vec![[0x87, 0x65, 0x43, 0x21]],
            origin_fingerprint: Some([0xBA, 0xDD, 0xCA, 0xFE]),
            origin_path: "87'/0'/0'",
            network: NetworkKind::Main,
            seed_byte: 0x08,
            chunk_set_id: 0x89012,
        },
        // V9..V17 — added in v0.1.1 Phase 2 to close
        // `vector-corpus-dictionary-coverage`. Together with V1..V8 they
        // exercised every closure-locked path-dictionary entry except
        // 0x16 (BIP 48 testnet nested-segwit), which v0.1.x reserved
        // pending the parallel md-codec gap closure. v0.2.0 adds V18
        // for 0x16 after both md-codec v0.9.0 and mk-codec v0.2.0
        // closed their respective gaps.
        // Fingerprint state alternates per the milestone plan:
        // V9-V11/V13-V15 with fp; V12/V16/V17 without.
        clean_fixture! {
            name: "V9_bip44_mainnet_1_stub_with_fp",
            description: "1-stub mainnet, BIP 44 single-sig (m/44'/0'/0'), \
                 fingerprint present. Std-table indicator 0x01.",
            policy_id_stubs: vec![[0x44, 0x44, 0x44, 0x44]],
            origin_fingerprint: Some([0xC0, 0x01, 0xCA, 0xFE]),
            origin_path: "44'/0'/0'",
            network: NetworkKind::Main,
            seed_byte: 0x09,
            chunk_set_id: 0x9A012,
        },
        clean_fixture! {
            name: "V10_bip49_mainnet_1_stub_with_fp",
            description: "1-stub mainnet, BIP 49 nested-segwit single-sig \
                 (m/49'/0'/0'), fingerprint present. Std-table indicator 0x02.",
            policy_id_stubs: vec![[0x49, 0x49, 0x49, 0x49]],
            origin_fingerprint: Some([0xFE, 0xED, 0xBE, 0xEF]),
            origin_path: "49'/0'/0'",
            network: NetworkKind::Main,
            seed_byte: 0x0A,
            chunk_set_id: 0xAB123,
        },
        clean_fixture! {
            name: "V11_bip86_mainnet_1_stub_with_fp",
            description: "1-stub mainnet, BIP 86 taproot single-sig \
                 (m/86'/0'/0'), fingerprint present. Std-table indicator 0x04.",
            policy_id_stubs: vec![[0x86, 0x86, 0x86, 0x86]],
            origin_fingerprint: Some([0x86, 0x40, 0x70, 0x05]),
            origin_path: "86'/0'/0'",
            network: NetworkKind::Main,
            seed_byte: 0x0B,
            chunk_set_id: 0xBC234,
        },
        clean_fixture! {
            name: "V12_bip48_nested_segwit_mainnet_1_stub_no_fp",
            description: "1-stub mainnet, BIP 48 nested-segwit multisig \
                 (m/48'/0'/0'/1'), fingerprint omitted (privacy-preserving \
                 mode). Std-table indicator 0x06.",
            policy_id_stubs: vec![[0x48, 0x48, 0x00, 0x01]],
            origin_fingerprint: None,
            origin_path: "48'/0'/0'/1'",
            network: NetworkKind::Main,
            seed_byte: 0x0C,
            chunk_set_id: 0xCD345,
        },
        clean_fixture! {
            name: "V13_bip44_testnet_1_stub_with_fp",
            description: "1-stub testnet, BIP 44 single-sig (m/44'/1'/0'), \
                 fingerprint present. Std-table indicator 0x11 \
                 (testnet-bit-15 variant of 0x01).",
            policy_id_stubs: vec![[0x44, 0x11, 0x00, 0x00]],
            origin_fingerprint: Some([0x44, 0x11, 0xAA, 0xBB]),
            origin_path: "44'/1'/0'",
            network: NetworkKind::Test,
            seed_byte: 0x0D,
            chunk_set_id: 0xDE456,
        },
        clean_fixture! {
            name: "V14_bip49_testnet_1_stub_with_fp",
            description: "1-stub testnet, BIP 49 nested-segwit (m/49'/1'/0'), \
                 fingerprint present. Std-table indicator 0x12.",
            policy_id_stubs: vec![[0x49, 0x12, 0x00, 0x00]],
            origin_fingerprint: Some([0x49, 0x12, 0xCC, 0xDD]),
            origin_path: "49'/1'/0'",
            network: NetworkKind::Test,
            seed_byte: 0x0E,
            chunk_set_id: 0xEF567,
        },
        clean_fixture! {
            name: "V15_bip84_testnet_1_stub_with_fp",
            description: "1-stub testnet, BIP 84 native-segwit (m/84'/1'/0'), \
                 fingerprint present. Std-table indicator 0x13.",
            policy_id_stubs: vec![[0x84, 0x13, 0x00, 0x00]],
            origin_fingerprint: Some([0x84, 0x13, 0xEE, 0xFF]),
            origin_path: "84'/1'/0'",
            network: NetworkKind::Test,
            seed_byte: 0x0F,
            chunk_set_id: 0xF0678,
        },
        clean_fixture! {
            name: "V16_bip86_testnet_1_stub_no_fp",
            description: "1-stub testnet, BIP 86 taproot (m/86'/1'/0'), \
                 fingerprint omitted. Std-table indicator 0x14.",
            policy_id_stubs: vec![[0x86, 0x14, 0x00, 0x00]],
            origin_fingerprint: None,
            origin_path: "86'/1'/0'",
            network: NetworkKind::Test,
            seed_byte: 0x10,
            chunk_set_id: 0x01789,
        },
        clean_fixture! {
            name: "V17_bip87_testnet_1_stub_no_fp",
            description: "1-stub testnet, BIP 87 multisig (m/87'/1'/0'), \
                 fingerprint omitted. Std-table indicator 0x17 \
                 (closes the v0.1 std-table testnet coverage modulo the \
                 0x16 BIP 48 nested-segwit gap; gap closed in v0.2.0 \
                 — see V18).",
            policy_id_stubs: vec![[0x87, 0x17, 0x00, 0x00]],
            origin_fingerprint: None,
            origin_path: "87'/1'/0'",
            network: NetworkKind::Test,
            seed_byte: 0x11,
            chunk_set_id: 0x1289A,
        },
        clean_fixture! {
            name: "V18_bip48_nested_segwit_testnet_1_stub_with_fp",
            description: "1-stub testnet, BIP 48 nested-segwit multisig \
                 (m/48'/1'/0'/1'), fingerprint present. Std-table \
                 indicator 0x16 — added to mk1's path dictionary in \
                 v0.2.0 after md-codec v0.9.0 closed the parallel gap. \
                 Wire-additive: v0.1.x decoders reject this vector with \
                 Error::InvalidPathIndicator(0x16); v0.2+ decoders accept \
                 and resolve to the BIP 48 testnet nested-segwit path.",
            policy_id_stubs: vec![[0x48, 0x16, 0xAA, 0xBB]],
            origin_fingerprint: Some([0x48, 0x16, 0xCC, 0xDD]),
            origin_path: "48'/1'/0'/1'",
            network: NetworkKind::Test,
            seed_byte: 0x12,
            chunk_set_id: 0x239AB,
        },
        // ── Negative vectors N1..N23 (closing decoder-error-variant-parity) ──
        //
        // One per `Error` variant reachable from `mk_codec::decode`'s
        // string-input path. `Error::CardPayloadTooLarge` is encoder-
        // only (fires in `split_into_chunks`, not reachable from
        // `decode`); the exhaustiveness gate in `tests/vectors.rs`
        // documents the variant exemption explicitly.
        n1_invalid_hrp(),
        n2_mixed_case(),
        n3_invalid_string_length(),
        n4_invalid_char(),
        n5_bch_uncorrectable(),
        n6_unsupported_card_type(),
        n7_malformed_payload_padding(),
        n8_chunk_set_id_mismatch(),
        n9_chunk_index_out_of_range(),
        n10_mixed_header_types(),
        n11_cross_chunk_hash_mismatch(),
        n12_unsupported_version(),
        n13_reserved_bits_set(),
        n14_invalid_policy_id_stub_count(),
        n15_invalid_path_indicator(),
        n16_path_too_deep(),
        n17_invalid_path_component(),
        n18_invalid_xpub_version(),
        n19_invalid_xpub_public_key(),
        n20_unexpected_end(),
        n21_trailing_bytes(),
        n23_empty_input(),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative-vector construction helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Wrap arbitrary bytecode bytes in mk1 strings, bypassing encoder-side
/// validation. Negative vectors that target bytecode-layer rejection paths
/// (N12..N21) construct a malformed bytecode bottom-up, then call this
/// helper to package it in the string layer (BCH checksum + chunked-or-
/// single header). The string layer accepts anything bytes-shaped, so
/// the malformation reaches the decoder unmodified.
fn wrap_bytecode_in_mk1(bytecode: &[u8], chunk_set_id: u32) -> Vec<String> {
    use mk_codec::SINGLE_STRING_LONG_BYTES;
    if bytecode.len() <= SINGLE_STRING_LONG_BYTES {
        let header = StringLayerHeader::SingleString {
            version: VERSION_V0_1,
        };
        let mut data_5bit = header.to_5bit_symbols();
        data_5bit.extend(bytes_to_5bit(bytecode));
        vec![encode_5bit_to_string(&data_5bit).expect("encode singlestring")]
    } else {
        let chunks = split_into_chunks(bytecode, chunk_set_id).expect("split_into_chunks");
        chunks
            .into_iter()
            .map(|chunk| {
                let mut data_5bit = chunk.header.to_5bit_symbols();
                data_5bit.extend(bytes_to_5bit(&chunk.fragment));
                encode_5bit_to_string(&data_5bit).expect("encode chunk")
            })
            .collect()
    }
}

/// Build a synthetic `SingleString`-headed mk1 string from arbitrary
/// bytecode bytes (bypasses encoder validation; v0.1 encoders never emit
/// SingleString per SPEC §2.4). Used by N6, N7, N10.
fn synthetic_singlestring(bytecode: &[u8]) -> String {
    let header = StringLayerHeader::SingleString {
        version: VERSION_V0_1,
    };
    let mut data_5bit = header.to_5bit_symbols();
    data_5bit.extend(bytes_to_5bit(bytecode));
    encode_5bit_to_string(&data_5bit).expect("synthetic singlestring encode")
}

/// Build a "standard" valid bytecode (1-stub mainnet, BIP 84, fp omitted)
/// for negative vectors that need a base bytecode to mutate.
fn baseline_valid_bytecode() -> Vec<u8> {
    use bitcoin::bip32::Fingerprint;
    let path = DerivationPath::from_str("84'/0'/0'").expect("path parses");
    let xpub = synthetic_xpub(NetworkKind::Main, 0xFEu8, &path);
    let card = KeyCard::new(
        vec![[0xCAu8, 0xFEu8, 0xBAu8, 0xBEu8]],
        Some(Fingerprint::from([0xDEu8, 0xADu8, 0xBEu8, 0xEFu8])),
        path,
        xpub,
    );
    encode_bytecode(&card).expect("baseline encode")
}

/// Get a clean encoding for a baseline card (a real `encode_with_chunk_set_id`
/// output) — used for negative vectors that mutate a clean encoding at the
/// string layer (N1, N2, N4, N5, N8, N11).
fn baseline_clean_strings(chunk_set_id: u32) -> Vec<String> {
    use bitcoin::bip32::Fingerprint;
    let path = DerivationPath::from_str("84'/0'/0'").expect("path parses");
    let xpub = synthetic_xpub(NetworkKind::Main, 0xFEu8, &path);
    let card = KeyCard::new(
        vec![[0xCAu8, 0xFEu8, 0xBAu8, 0xBEu8]],
        Some(Fingerprint::from([0xDEu8, 0xADu8, 0xBEu8, 0xEFu8])),
        path,
        xpub,
    );
    encode_with_chunk_set_id(&card, chunk_set_id).expect("baseline encode_with_chunk_set_id")
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative-vector specs
// ─────────────────────────────────────────────────────────────────────────────

fn n1_invalid_hrp() -> FixtureSpec {
    // Replace `mk1` HRP+separator with `bt1` on a clean encoding;
    // the rest of the data part stays valid (it's just the HRP that's
    // wrong), so decoder reaches `Error::InvalidHrp("bt")` directly.
    let baseline = baseline_clean_strings(0x12345);
    let perturbed = baseline[0].replacen("mk1", "bt1", 1);
    FixtureSpec {
        name: "N1_invalid_hrp_bt",
        description: "HRP `bt` is not the locked `mk` HRP; decoder rejects \
            before any data-part processing.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![perturbed],
            expected_error: "invalid HRP: bt".to_string(),
            why: "mk1 strings MUST start with HRP `mk` per SPEC §2.1; any other HRP is rejected.",
        }),
    }
}

fn n2_mixed_case() -> FixtureSpec {
    // Capitalise one ASCII char in the data part of a clean encoding.
    let baseline = baseline_clean_strings(0x12345);
    let mut chars: Vec<char> = baseline[0].chars().collect();
    // Char at index 5 is well past `mk1`; the bech32 alphabet is all
    // lowercase, so any lower→upper substitution triggers MixedCase.
    chars[5] = chars[5].to_ascii_uppercase();
    let perturbed: String = chars.into_iter().collect();
    FixtureSpec {
        name: "N2_mixed_case",
        description: "One ASCII char in the data part is uppercase; BIP 173 forbids \
            mixed case to remove a class of transcription ambiguity.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![perturbed],
            expected_error: "mixed case in input string".to_string(),
            why: "BIP 173 §3 prohibits mixed-case strings; mk-codec inherits the rule verbatim.",
        }),
    }
}

fn n3_invalid_string_length() -> FixtureSpec {
    // Construct an `mk1`-prefixed string with exactly 13 data-part chars —
    // below the BCH(93,80,8) regular-code minimum of 14. Decoder rejects
    // before any BCH verification.
    let s = "mk1qpzry9x8gf2tv".to_string(); // "mk1" + 13 valid bech32 chars
    debug_assert_eq!(s.len() - 3, 13);
    FixtureSpec {
        name: "N3_invalid_string_length_too_short",
        description: "Data-part length 13 is below the BCH regular-code minimum of 14.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![s],
            expected_error: "invalid data-part length: 13".to_string(),
            why: "BIP 93 valid lengths: regular [14,93], long [96,108]; 13 is outside both ranges.",
        }),
    }
}

fn n4_invalid_char() -> FixtureSpec {
    // Substitute one `b` (excluded from the bech32 alphabet) into a clean
    // encoding's data part.
    let baseline = baseline_clean_strings(0x12345);
    let mut chars: Vec<char> = baseline[0].chars().collect();
    chars[5] = 'b';
    let perturbed: String = chars.into_iter().collect();
    FixtureSpec {
        name: "N4_invalid_char_b",
        description: "Data part contains `b`, which is not in the bech32 alphabet.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![perturbed],
            expected_error: "invalid character b at position 2".to_string(),
            why: "Bech32 alphabet is `qpzry9x8gf2tvdw0s3jn54khce6mua7l`; 'b' is not in it.",
        }),
    }
}

fn n5_bch_uncorrectable() -> FixtureSpec {
    // Substitute 5 chars in chunk[0]'s data part with values guaranteed to
    // exceed BCH t=4. Position the burst past the chunked header to keep
    // the rejection in BCH territory rather than header-decode.
    let baseline = baseline_clean_strings(0x12345);
    let mut chars: Vec<char> = baseline[0].chars().collect();
    // Indices 11..16: past `mk1` (3 chars) and past 8-symbol chunked
    // header. Substitute each with a different bech32 char to ensure
    // a non-zero 5-bit XOR per position.
    for c in chars.iter_mut().take(16).skip(11) {
        *c = if *c == 'q' { 'p' } else { 'q' };
    }
    let perturbed: String = chars.into_iter().collect();
    FixtureSpec {
        name: "N5_bch_uncorrectable_5_substitutions",
        description: "5-symbol burst exceeds BCH `t=4` correction radius for both \
            BCH(93,80,8) and BCH(108,93,8).",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![perturbed, baseline[1].clone()],
            expected_error: "BCH uncorrectable: long code: more than 4 substitutions \
                or pathological pattern"
                .to_string(),
            why: "BCH `t=4` covers up to 4 substitutions exactly; 5+ exceeds the \
                correction radius and the decoder must surface BchUncorrectable.",
        }),
    }
}

fn n6_unsupported_card_type() -> FixtureSpec {
    // Build a SingleString-shaped string whose `type` byte is 0x02
    // (reserved range 0x02..=0x1F). Header position 1 = type byte.
    // Build the symbol stream manually since `to_5bit_symbols` validates
    // the type field.
    let mut data_5bit = vec![VERSION_V0_1, 0x02u8]; // version=0, type=0x02 (reserved)
    data_5bit.extend(bytes_to_5bit(&[0u8; 8])); // arbitrary payload
    let s = encode_5bit_to_string(&data_5bit).expect("encode");
    FixtureSpec {
        name: "N6_unsupported_card_type_0x02",
        description: "String-layer header `type` byte 0x02 is in the reserved range \
            0x02..=0x1F; decoders MUST reject (SPEC §2.5).",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![s],
            expected_error: "unsupported card type: 0x02".to_string(),
            why: "Only types 0x00 (SingleString) and 0x01 (Chunked) are defined in v0.1; \
                0x02..=0x1F are reserved for future format extensions.",
        }),
    }
}

fn n7_malformed_payload_padding() -> FixtureSpec {
    // SingleString header + payload symbols whose trailing pad bits are
    // non-zero. The existing pipeline.rs unit test pattern (header + 3
    // symbols with the last symbol's low bits set) reliably triggers
    // `MalformedPayloadPadding`.
    let header = StringLayerHeader::SingleString {
        version: VERSION_V0_1,
    };
    let mut data_5bit = header.to_5bit_symbols();
    data_5bit.extend([0u8, 0u8, 0b00011u8]); // last symbol's low 2 bits are non-zero pad
    let s = encode_5bit_to_string(&data_5bit).expect("encode");
    FixtureSpec {
        name: "N7_malformed_payload_padding",
        description: "5-bit payload symbols don't byte-align — trailing pad bits \
            of the final symbol are non-zero.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![s],
            expected_error: "malformed payload padding (5-bit symbols don't byte-align)"
                .to_string(),
            why: "Conforming encoders zero-pad the final 5-bit symbol; a non-zero pad \
                cannot have been produced by `bytes_to_5bit`.",
        }),
    }
}

fn n8_chunk_set_id_mismatch() -> FixtureSpec {
    // Two clean encodings with different chunk_set_id values; splice
    // chunk[0] from one and chunk[1] from the other.
    let a = baseline_clean_strings(0x12345);
    let b = baseline_clean_strings(0x67890);
    FixtureSpec {
        name: "N8_chunk_set_id_mismatch",
        description: "Two-chunk input where chunk[0]'s chunk_set_id differs from \
            chunk[1]'s; decoder rejects at reassembly.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![a[0].clone(), b[1].clone()],
            expected_error: "chunk_set_id mismatch across chunks".to_string(),
            why: "All chunks of one card share `chunk_set_id` (SPEC §2.5); cross-set \
                splicing is detected by the reassembler.",
        }),
    }
}

fn n9_chunk_index_out_of_range() -> FixtureSpec {
    // Build a chunked header with chunk_index = total_chunks (out of range).
    // `to_5bit_symbols` would catch this if we used the public API, so
    // build the symbol stream manually with the off-by-one wire encoding.
    // version=0, type=1, csid=0x12345 packed big-endian, total_chunks=2 → wire=1,
    // chunk_index=2 (>= total_chunks).
    let csid: u32 = 0x12345;
    let total_chunks_wire: u8 = 2 - 1; // 2 - 1 = 1
    let chunk_index: u8 = 2; // out of range
    let mut data_5bit = vec![
        VERSION_V0_1,
        0x01u8, // type = Chunked
        ((csid >> 15) & 0x1F) as u8,
        ((csid >> 10) & 0x1F) as u8,
        ((csid >> 5) & 0x1F) as u8,
        (csid & 0x1F) as u8,
        total_chunks_wire,
        chunk_index,
    ];
    // Append a small fragment so the data-part length is BCH-valid.
    data_5bit.extend(bytes_to_5bit(&[0u8; 53]));
    let s = encode_5bit_to_string(&data_5bit).expect("encode");
    FixtureSpec {
        name: "N9_chunk_index_out_of_range",
        description: "Chunked header declares `chunk_index = total_chunks` (out of \
            range; valid range is `0..total_chunks`).",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![s],
            expected_error: "chunked-header malformed: chunk_index = 2 >= total_chunks = 2"
                .to_string(),
            why: "Per SPEC §4 rule 12, chunk_index MUST satisfy 0 ≤ chunk_index < total_chunks.",
        }),
    }
}

fn n10_mixed_header_types() -> FixtureSpec {
    // Forward direction: SingleString first, Chunked second. The reverse-
    // direction case lands in chunk.rs:reassemble; we cover it via
    // pipeline::tests::decode_rejects_chunked_then_singlestring and
    // skip it here to avoid two N10 vectors.
    let single = synthetic_singlestring(&[0x42u8; 8]);
    let chunked = baseline_clean_strings(0x12345);
    FixtureSpec {
        name: "N10_mixed_header_types_singlestring_then_chunked",
        description: "First string is SingleString-headed, second is Chunked; \
            decoder rejects header-types-disagree.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![single, chunked[0].clone()],
            expected_error: "mixed string-layer header types in input list".to_string(),
            why: "v0.1.1 introduced `Error::MixedHeaderTypes` to disambiguate \
                header-types-disagree from chunked-internal malformations.",
        }),
    }
}

fn n11_cross_chunk_hash_mismatch() -> FixtureSpec {
    // Recompute clean chunks; perturb a fragment byte in the cross-chunk
    // hash region (last 4 bytes of the stream); re-encode that chunk
    // (recomputes BCH checksum on the perturbed payload, so the BCH
    // layer passes cleanly and the rejection lands at the SHA-256
    // verification step).
    let bytecode = baseline_valid_bytecode();
    let chunks = split_into_chunks(&bytecode, 0x12345).expect("split");
    let mut perturbed_chunks = chunks.clone();
    let last = perturbed_chunks.last_mut().expect("≥1 chunks");
    // The cross-chunk hash bytes occupy the last 4 bytes of the stream,
    // which for the typical 84-byte bytecode → 88-byte stream lands at
    // chunk[1].fragment positions 31..35 (35-byte fragment).
    let hash_byte_idx = last.fragment.len() - 1;
    last.fragment[hash_byte_idx] ^= 0xFFu8;
    let strings: Vec<String> = perturbed_chunks
        .into_iter()
        .map(|chunk| {
            let mut data_5bit = chunk.header.to_5bit_symbols();
            data_5bit.extend(bytes_to_5bit(&chunk.fragment));
            encode_5bit_to_string(&data_5bit).expect("encode")
        })
        .collect();
    FixtureSpec {
        name: "N11_cross_chunk_hash_mismatch",
        description: "Last byte of the 4-byte cross-chunk hash is flipped; \
            recomputed SHA-256 over reassembled bytecode disagrees with the \
            recovered tail.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: strings,
            expected_error: "cross-chunk integrity hash mismatch".to_string(),
            why: "SPEC §2.6 — `cross_chunk_hash = SHA-256(bytecode)[0..4]` is recomputed \
                at reassembly and compared byte-for-byte against the stream's tail.",
        }),
    }
}

fn n12_unsupported_version() -> FixtureSpec {
    // Bytecode header with version=1 (high nibble = 0x1, all flag bits 0
    // → header byte = 0x10).
    let mut bytecode = baseline_valid_bytecode();
    bytecode[0] = 0x10u8;
    FixtureSpec {
        name: "N12_unsupported_version_v1",
        description: "Bytecode header has version=1; v0.1 only defines version=0.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&bytecode, 0x12345),
            expected_error: "unsupported version: 1".to_string(),
            why: "SPEC §3.1 — version field bits 7..4 MUST be 0x0 in v0.1.",
        }),
    }
}

fn n13_reserved_bits_set() -> FixtureSpec {
    // Bytecode header with bit 3 set (header byte = 0x08 — version=0, bit 3=1,
    // bit 2=0, bit 1=0, bit 0=0). Triggers ReservedBitsSet.
    let mut bytecode = baseline_valid_bytecode();
    bytecode[0] = 0x08u8;
    // Strip the (now flag-less) origin_fingerprint bytes since bit 2 is
    // unset, otherwise the decoder reads stub_count + stubs + path-from-
    // fp-bytes and gets confused. The baseline has fp-flag set originally;
    // override to a no-fp shape: header(0x08) + stub_count(1) + stub(4 B)
    // + path indicator + xpub.
    let mut rebuilt = vec![0x08u8, 0x01u8];
    rebuilt.extend_from_slice(&[0xCAu8, 0xFEu8, 0xBAu8, 0xBEu8]); // 1 stub
    rebuilt.push(0x03u8); // BIP 84 mainnet std-table indicator
    rebuilt.extend_from_slice(&bytecode[bytecode.len() - 73..]); // xpub_compact tail
    FixtureSpec {
        name: "N13_reserved_bits_set_bit3",
        description: "Bytecode header has bit 3 (reserved) set (header byte 0x08).",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&rebuilt, 0x12345),
            expected_error: "reserved bits set in bytecode header".to_string(),
            why: "SPEC §3.1 — bits 0, 1, 3 are reserved and MUST be 0 in v0.1.",
        }),
    }
}

fn n14_invalid_policy_id_stub_count() -> FixtureSpec {
    // Bytecode with stub_count=0. Build minimally: header + stub_count(0)
    // + path + xpub.
    let mut bytecode = vec![0x00u8, 0x00u8]; // header=0x00 (no fp), stub_count=0
    bytecode.push(0x03u8); // path indicator
    let baseline = baseline_valid_bytecode();
    bytecode.extend_from_slice(&baseline[baseline.len() - 73..]);
    FixtureSpec {
        name: "N14_invalid_policy_id_stub_count_zero",
        description: "Bytecode declares stub_count=0; SPEC §4 rule 3 requires ≥ 1.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&bytecode, 0x12345),
            expected_error: "policy_id_stub_count must be >= 1".to_string(),
            why: "Closure §4 rule 3 — every conforming mk1 KeyCard names ≥ 1 Policy ID stub.",
        }),
    }
}

fn n15_invalid_path_indicator() -> FixtureSpec {
    // Path indicator 0x00 is reserved; valid std-table is 0x01..=0x07
    // (mainnet) and 0x11..=0x17 (testnet); 0xFE is the explicit-path
    // escape; everything else is rejected.
    let mut bytecode = vec![0x00u8, 0x01u8]; // header=0x00, stub_count=1
    bytecode.extend_from_slice(&[0xCAu8, 0xFEu8, 0xBAu8, 0xBEu8]);
    bytecode.push(0x00u8); // INVALID path indicator
    let baseline = baseline_valid_bytecode();
    bytecode.extend_from_slice(&baseline[baseline.len() - 73..]);
    FixtureSpec {
        name: "N15_invalid_path_indicator_0x00",
        description: "Bytecode declares path indicator 0x00 (reserved); \
            valid std-table indicators are 0x01..=0x07 mainnet, 0x11..=0x17 \
            testnet, plus 0xFE explicit-path escape.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&bytecode, 0x12345),
            expected_error: "invalid path indicator byte: 0x00".to_string(),
            why: "SPEC §3.5 reserved indicators include 0x00 and 0xFF.",
        }),
    }
}

fn n16_path_too_deep() -> FixtureSpec {
    // Explicit path with count=11 (cap is 10 per closure Q-3).
    let mut bytecode = vec![0x00u8, 0x01u8];
    bytecode.extend_from_slice(&[0xCAu8, 0xFEu8, 0xBAu8, 0xBEu8]);
    bytecode.push(0xFEu8); // explicit-path indicator
    bytecode.push(0x0Bu8); // count = 11 (one above the cap)
    // 11 LEB128-encoded components (each just one byte for simplicity):
    for i in 0..11u8 {
        bytecode.push(i);
    }
    let baseline = baseline_valid_bytecode();
    bytecode.extend_from_slice(&baseline[baseline.len() - 73..]);
    FixtureSpec {
        name: "N16_path_too_deep_11_components",
        description: "Explicit-path count=11 exceeds the 10-component cap (closure Q-3).",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&bytecode, 0x12345),
            expected_error: "path too deep: 11 components (max 10)".to_string(),
            why: "Closure Q-3 capped explicit-path component count at 10 to bound \
                bytecode size; encoders MUST reject any deeper path.",
        }),
    }
}

fn n17_invalid_path_component() -> FixtureSpec {
    // LEB128 overflow: 6 consecutive bytes with the continuation bit set
    // exceed the u32 range (5 × 7 = 35 bits with each byte contributing 7,
    // so the 6th byte's shift is at 35 — overflow). The decoder's LEB128
    // routine surfaces this as `Error::InvalidPathComponent("LEB128 overflow at shift 35")`
    // (or similar message, matched byte-exact by the harness against the
    // pinned `expected_error`).
    let baseline = baseline_valid_bytecode();
    let xpub_tail = &baseline[baseline.len() - 73..];
    let mut bytecode = vec![0x00u8, 0x01u8];
    bytecode.extend_from_slice(&[0xCAu8, 0xFEu8, 0xBAu8, 0xBEu8]);
    bytecode.push(0xFEu8); // explicit-path indicator
    bytecode.push(0x01u8); // count = 1
    // 6 × 0x80 — every byte sets the continuation bit, so the LEB128
    // decoder consumes all 6 and overflows past u32 capacity at shift=35.
    bytecode.extend_from_slice(&[0x80u8; 6]);
    bytecode.extend_from_slice(xpub_tail);
    FixtureSpec {
        name: "N17_invalid_path_component_leb128_overflow",
        description: "Explicit path's LEB128 component overflows u32 \
            (6 × 0x80 — every byte sets the continuation bit, exceeding \
            the 5-byte BIP 32 child-number representation).",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&bytecode, 0x12345),
            expected_error: "invalid path component: LEB128 overflow at shift 35".to_string(),
            why: "BIP 32 child numbers are 32-bit unsigned; a 6-byte LEB128 \
                stream exceeds u32 capacity and decoders MUST reject with \
                `InvalidPathComponent` per SPEC §4 rule 6.",
        }),
    }
}

fn n18_invalid_xpub_version() -> FixtureSpec {
    // Replace xpub_compact's 4-byte version prefix with 0xDEADBEEF.
    let mut bytecode = baseline_valid_bytecode();
    let xpub_offset = bytecode.len() - 73;
    bytecode[xpub_offset..xpub_offset + 4].copy_from_slice(&[0xDEu8, 0xADu8, 0xBEu8, 0xEFu8]);
    FixtureSpec {
        name: "N18_invalid_xpub_version_0xdeadbeef",
        description: "xpub_compact's version prefix is 0xDEADBEEF, not a known \
            mainnet/testnet xpub version.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&bytecode, 0x12345),
            expected_error: "invalid xpub version: 0xdeadbeef".to_string(),
            why: "Compact-73 xpub form (closure Q-7) carries the BIP 32 version \
                bytes verbatim; decoders validate against {xpub, tpub} prefixes.",
        }),
    }
}

fn n19_invalid_xpub_public_key() -> FixtureSpec {
    // Replace xpub_compact's 33-byte public_key suffix with all-zeros (not
    // a valid compressed secp256k1 point).
    let mut bytecode = baseline_valid_bytecode();
    let bc_len = bytecode.len();
    bytecode[bc_len - 33..].copy_from_slice(&[0u8; 33]);
    FixtureSpec {
        name: "N19_invalid_xpub_public_key_all_zeros",
        description: "xpub_compact's public_key bytes are all zeros — not a valid \
            compressed secp256k1 point.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&bytecode, 0x12345),
            expected_error: "invalid xpub public key: malformed public key".to_string(),
            why: "secp256k1 compressed point validation rejects 33 zero bytes.",
        }),
    }
}

fn n20_unexpected_end() -> FixtureSpec {
    // Truncate the bytecode mid-xpub.
    let mut bytecode = baseline_valid_bytecode();
    bytecode.truncate(bytecode.len() - 5); // drop last 5 xpub bytes
    FixtureSpec {
        name: "N20_unexpected_end_truncated_xpub",
        description: "Bytecode truncated mid-xpub_compact; decoder hits \
            end-of-stream before the 73 xpub bytes are consumed.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&bytecode, 0x12345),
            expected_error: "unexpected end of bytecode".to_string(),
            why: "Decoder reads fields greedily; truncation at any point produces \
                `UnexpectedEnd`.",
        }),
    }
}

fn n21_trailing_bytes() -> FixtureSpec {
    // Append one extra byte after the xpub.
    let mut bytecode = baseline_valid_bytecode();
    bytecode.push(0xFFu8);
    FixtureSpec {
        name: "N21_trailing_bytes_one_extra",
        description: "One extra byte (0xFF) follows the xpub_compact tail; \
            decoder rejects after consuming the expected fields.",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: wrap_bytecode_in_mk1(&bytecode, 0x12345),
            expected_error: "trailing bytes after xpub".to_string(),
            why: "Conforming bytecode terminates exactly at the xpub_compact's \
                73-byte boundary; any tail content is rejected.",
        }),
    }
}

fn n23_empty_input() -> FixtureSpec {
    FixtureSpec {
        name: "N23_empty_input",
        description: "Empty input string list; decoder rejects with \
            ChunkedHeaderMalformed (covers the second call-site of that \
            variant beyond N9's chunk-index-OOB form).",
        kind: FixtureKind::Negative(NegativeInput {
            input_strings: vec![],
            expected_error: "chunked-header malformed: empty input string list".to_string(),
            why: "An empty `&[]` to `decode` has no chunks to process; this is the \
                no-input-at-all case, distinct from header-types-disagree.",
        }),
    }
}

/// Build a deterministic synthetic xpub for a fixture.
///
/// The resulting xpub is a valid BIP 32 extended public key (real
/// secp256k1 point on the curve), but the parent_fingerprint and
/// chain_code are fixed test values rather than derived from a
/// chain-of-trust. Decoders will accept the xpub at the wire level;
/// real-world recovery would re-verify against an external Wallet
/// Instance ID anchor (per SPEC §5).
fn synthetic_xpub(network: NetworkKind, seed_byte: u8, path: &DerivationPath) -> Xpub {
    let secp = Secp256k1::new();
    let secret_bytes = [seed_byte; 32];
    let sk =
        SecretKey::from_slice(&secret_bytes).expect("non-zero seed must be a valid secret key");
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let components: Vec<ChildNumber> = path.into_iter().copied().collect();
    let depth = components.len() as u8;
    let child_number = components
        .last()
        .copied()
        .unwrap_or(ChildNumber::Normal { index: 0 });
    Xpub {
        network,
        depth,
        // Distinct from seed_byte so an attacker can't trivially derive
        // it from public knowledge of the fixture's secret. Vectors
        // exist for testing wire-format conformance, not security.
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, seed_byte]),
        child_number,
        public_key: pk,
        chain_code: ChainCode::from([seed_byte ^ 0xAA; 32]),
    }
}

fn lowercase_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        // `format_args!` would allocate; build the string manually for
        // determinism + lowercase-hex enforcement.
        const HEX: &[u8; 16] = b"0123456789abcdef";
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0F) as usize] as char);
    }
    s
}

fn fixture_to_value(spec: &FixtureSpec) -> Value {
    match &spec.kind {
        FixtureKind::Clean(input) => clean_fixture_to_value(spec.name, spec.description, input),
        FixtureKind::Negative(input) => {
            negative_fixture_to_value(spec.name, spec.description, input)
        }
    }
}

fn clean_fixture_to_value(name: &str, description: &str, input: &CleanInput) -> Value {
    let path: DerivationPath =
        DerivationPath::from_str(input.origin_path).expect("fixture origin_path must parse");
    let xpub = synthetic_xpub(input.network, input.seed_byte, &path);

    let card = KeyCard::new(
        input.policy_id_stubs.clone(),
        input.origin_fingerprint.map(Fingerprint::from),
        path.clone(),
        xpub,
    );

    let bytecode = encode_bytecode(&card).expect("encode_bytecode succeeds for valid fixture");
    let strings = encode_with_chunk_set_id(&card, input.chunk_set_id)
        .expect("encode_with_chunk_set_id succeeds for valid fixture");

    let stubs_json: Vec<Value> = input
        .policy_id_stubs
        .iter()
        .map(|s| Value::String(lowercase_hex(s)))
        .collect();
    let fp_json = match input.origin_fingerprint {
        Some(fp) => Value::String(lowercase_hex(&fp)),
        None => Value::Null,
    };

    json!({
        "name": name,
        "description": description,
        "expected_error": Value::Null,
        "input": {
            "chunk_set_id": input.chunk_set_id,
            "network": match input.network {
                NetworkKind::Main => "mainnet",
                NetworkKind::Test => "testnet",
            },
            "origin_fingerprint": fp_json,
            "origin_path": format!("m/{}", path),
            "policy_id_stubs": stubs_json,
            "xpub": xpub.to_string(),
        },
        "expected": {
            "canonical_bytecode_hex": lowercase_hex(&bytecode),
            "decoder_correction": "clean",
            "strings": strings,
            "total_chunks": strings.len(),
        },
    })
}

fn negative_fixture_to_value(name: &str, description: &str, input: &NegativeInput) -> Value {
    json!({
        "name": name,
        "description": description,
        "expected_error": input.expected_error,
        "input": {
            "strings": input.input_strings,
            "why": input.why,
        },
    })
}

fn main() {
    // Resolve --output (default: crates/mk-codec/src/test_vectors/v0.1.json
    // relative to the workspace root, which is the cwd when run via cargo).
    let mut args = env::args().skip(1);
    let mut output_path: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" | "-o" => {
                output_path = Some(PathBuf::from(
                    args.next().expect("--output requires a path"),
                ));
            }
            other => panic!("unrecognised argument: {other}"),
        }
    }
    let output_path =
        output_path.unwrap_or_else(|| PathBuf::from("crates/mk-codec/src/test_vectors/v0.1.json"));

    let vectors_json: Vec<Value> = fixtures().iter().map(fixture_to_value).collect();
    let document = json!({
        "schema": 2,
        "family_token": mk_codec::GENERATOR_FAMILY,
        "vectors": vectors_json,
    });

    // Pretty-print with 2-space indent + lowercase hex (already enforced
    // upstream in `lowercase_hex`). `serde_json::Map` is BTreeMap-backed
    // by default so keys sort alphabetically at every level. Default
    // `PrettyFormatter` uses a 2-space indent, matching the canonicality
    // discipline pinned in the module-level docs.
    let mut buf: Vec<u8> = Vec::new();
    serde_json::to_writer_pretty(&mut buf, &document)
        .expect("serializing pre-built Value cannot fail");
    // Trailing newline at EOF, LF line endings.
    buf.push(b'\n');

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("create vector output directory");
    }
    let mut out = fs::File::create(&output_path).expect("create output file");
    out.write_all(&buf).expect("write vector JSON");
    out.flush().expect("flush vector JSON");

    eprintln!(
        "wrote {} vectors to {} ({} bytes)",
        fixtures().len(),
        output_path.display(),
        buf.len()
    );
}
