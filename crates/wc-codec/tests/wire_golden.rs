//! T3-a — frozen Word-Card **wire goldens** (constellation-eval §2 item #3;
//! `design/SPEC_test_hardening_T3_wire_goldens.md` T3-a).
//!
//! # Why this file exists (the round-trip-only blind spot)
//!
//! Every other `wc-codec` test (`tests/pipeline.rs`, `tests/raid.rs`) encodes
//! *and* decodes with the SAME code under test -- a **symmetric** wire change
//! (field order, CRC context, tag placement, RS parity order) passes every
//! round-trip assertion and both fuzzers while silently **bricking every
//! already-engraved steel plate** (the physical word sequence a user carved
//! yesterday no longer decodes with tomorrow's binary). The only defense is a
//! **frozen historical wire oracle**: a literal word list committed as a
//! `const`, independent of all *future* code (though NOT independent of
//! `wc-codec` itself -- it is the codec's own historical output, generated once
//! and frozen; see the eval's own prescription, `design/RECON_T3_wire_goldens.md`
//! Item #3 §3). This is the ONLY oracle type that can catch a symmetric
//! encode+decode drift; T2-b's PGZ reference decoder is the FEC-*correctness*
//! analogue, not a wire-*layout* one -- out of scope here.
//!
//! # Provenance (applies to every `const` below)
//!
//! - `wc-codec` version: `0.1.1` (`Cargo.toml:3` at generation time).
//! - Toolkit git SHA at generation time: `c00ed813` (HEAD at T3 implementation
//!   start, per `design/SPEC_test_hardening_T3_wire_goldens.md` line 3/6).
//! - Generating calls: see each section below. Payload `(bytes, payload_bits)`
//!   inputs were lifted from the toolkit's own canonical-payload adapter
//!   (`mk_codec::KeyCard::canonical_payload_bytes` /
//!   `md_codec::Descriptor::canonical_payload_bytes`) applied to the existing
//!   deterministic `abandon x11 about` fixtures already frozen in
//!   `crates/mnemonic-toolkit/src/word_card_adapter.rs:190-198` (mk1/md1) and
//!   `crates/mnemonic-toolkit/tests/cli_word_card.rs:20-25,28-30` (the RAID
//!   trio) -- NOT re-derived here (`wc-codec` has no mk/md-codec dep). The word
//!   lists were then generated ONCE via the exact `wc_codec::{encode,
//!   raid_encode}` calls shown in each test fn's doc comment, and frozen.
//! - `wc-codec` has NO mk-codec/md-codec dependency (`Cargo.toml`: `bip39` +
//!   `sha2` only) -- the `(payload, payload_bits)` `const`s below are therefore
//!   the input-side freeze this leg's "freeze BOTH sides" requirement needs,
//!   not something this file can re-derive from an `mk1`/`md1` string itself.

use wc_codec::{decode, encode, raid_encode, EncodeOpts, PlateRole, SourceKind};

// ===========================================================================
// mk1-kind golden (i) -- byte-aligned payload (`payload_bits == 8 * len`).
// ===========================================================================

/// The `abandon x11 about` bip84 mk1 card's canonical payload bytes (84
/// bytes), via `mk_codec::decode(&MK1_FIXTURE_CHUNKS).canonical_payload_bytes()`
/// over the mk1 fixture frozen at `word_card_adapter.rs:190-193`.
const MK1_PAYLOAD: [u8; 84] = [
    0x04, 0x01, 0x1c, 0x01, 0x70, 0xfe, 0x73, 0xc5, 0xda, 0x0a, 0x03, 0x04, 0x88, 0xb2, 0x1e, 0x7e,
    0xf3, 0x2b, 0xdb, 0x4a, 0x53, 0xa0, 0xab, 0x21, 0xb9, 0xdc, 0x95, 0x86, 0x9c, 0x4e, 0x92, 0xa1,
    0x61, 0x19, 0x4e, 0x03, 0xc0, 0xef, 0x3f, 0xf5, 0x01, 0x4a, 0xc6, 0x92, 0xf4, 0x33, 0xc4, 0x76,
    0x54, 0x90, 0xfc, 0x02, 0x70, 0x7a, 0x62, 0xfd, 0xac, 0xc2, 0x6e, 0xa9, 0xb6, 0x3b, 0x1c, 0x19,
    0x79, 0x06, 0xf5, 0x6e, 0xe0, 0x18, 0x0d, 0x0b, 0xcf, 0x19, 0x66, 0xe1, 0xa2, 0xda, 0x34, 0xf5,
    0xf3, 0xa0, 0x9a, 0x9b,
];

/// `MK1_PAYLOAD`'s exact bit length. mk1 is byte-aligned: `8 * 84 = 672`.
const MK1_PAYLOAD_BITS: usize = 672;

/// The `EncodeOpts` used to generate every golden in this file: `m=8` parity
/// words, `t=44` (the crate default `DEFAULT_INTEGRITY_BITS`), `U=3` (the
/// crate default `DEFAULT_U_SLOTS`) -- the same parameters
/// `word_card_adapter.rs`'s own `full_wc_round_trip_*` tests use
/// (`parity_words: 8, ..Default::default()`).
fn golden_opts() -> EncodeOpts {
    EncodeOpts {
        parity_words: 8,
        integrity_bits: 44,
        u_slots: 3,
    }
}

/// The FROZEN engraved word sequence for `encode(Mk1Xpub, &MK1_PAYLOAD,
/// MK1_PAYLOAD_BITS, &golden_opts())` -- generated once, 2026-07-10, and pinned
/// literally. 96 words.
const MK1_WORDS: [&str; 96] = [
    "abandon", "actor", "airport", "gas", "forest", "this", "ask", "abandon", "abandon", "abandon",
    "abandon", "advice", "angry", "able", "tiger", "transfer", "title", "habit", "document",
    "phrase", "afraid", "easily", "maple", "worth", "crawl", "unit", "citizen", "inject",
    "pioneer", "private", "assist", "jaguar", "climb", "hawk", "chef", "enhance", "club", "profit",
    "bone", "hybrid", "usual", "taxi", "young", "level", "climb", "spoil", "quiz", "vintage",
    "owner", "budget", "powder", "average", "abuse", "sea", "era", "recall", "world", "gravity",
    "option", "fat", "glove", "mix", "grass", "camp", "rely", "voice", "jacket", "alcohol",
    "borrow", "rude", "million", "oppose", "crowd", "robust", "surface", "police", "language",
    "donkey", "predict", "tail", "play", "wink", "same", "great", "abandon", "patrol", "funny",
    "life", "media", "move", "angry", "easy", "rigid", "pilot", "valid", "arena",
];

#[test]
fn mk1_kind_wire_golden() {
    let words = encode(
        SourceKind::Mk1Xpub,
        &MK1_PAYLOAD,
        MK1_PAYLOAD_BITS,
        &golden_opts(),
    )
    .expect("encode mk1-kind golden");
    assert_eq!(
        words, MK1_WORDS,
        "mk1-kind wire golden mismatch -- the engraved word sequence for this \
         frozen (payload, payload_bits, m, t, u) tuple has changed; an already- \
         engraved plate encoded with the prior wire format would now brick"
    );
    // The golden must still decode (sanity -- a non-decoding golden would be a
    // useless oracle, not a stronger one).
    let decoded = decode(&MK1_WORDS).expect("decode mk1-kind golden");
    assert_eq!(decoded.kind, SourceKind::Mk1Xpub);
    assert_eq!(decoded.payload, MK1_PAYLOAD);
    assert_eq!(decoded.payload_bits, MK1_PAYLOAD_BITS);
}

// ===========================================================================
// md1-kind golden (ii) -- non-byte-aligned payload (`payload_bits % 8 != 0`).
// ===========================================================================

/// The same seed's bip84 md1 descriptor card's canonical payload bytes (81
/// bytes), via `md_codec::reassemble(&MD1_FIXTURE_CHUNKS)
/// .canonical_payload_bytes()` over the md1 fixture frozen at
/// `word_card_adapter.rs:194-198`.
const MD1_PAYLOAD: [u8; 81] = [
    0x20, 0x0e, 0xf5, 0x21, 0x08, 0x00, 0x60, 0x02, 0xd0, 0x39, 0xe2, 0xed, 0x05, 0x0a, 0xa0, 0x84,
    0xa5, 0x3a, 0x0a, 0xb2, 0x1b, 0x9d, 0xc9, 0x58, 0x69, 0xc4, 0xe9, 0x2a, 0x16, 0x11, 0x94, 0xe0,
    0x3c, 0x0e, 0xf3, 0xff, 0x50, 0x14, 0xac, 0x69, 0x2f, 0x43, 0x3c, 0x47, 0x65, 0x49, 0x0f, 0xc0,
    0x27, 0x07, 0xa6, 0x2f, 0xda, 0xcc, 0x26, 0xea, 0x9b, 0x63, 0xb1, 0xc1, 0x97, 0x90, 0x6f, 0x56,
    0xee, 0x01, 0x80, 0xd0, 0xbc, 0xf1, 0x96, 0x6e, 0x1a, 0x2d, 0xa3, 0x4f, 0x5f, 0x3a, 0x09, 0xa9,
    0xb0,
];

/// `MD1_PAYLOAD`'s exact bit length -- **NOT** a multiple of 8 (`644 % 8 ==
/// 4`), the md1 bit-precise case this golden exists to pin
/// (`word_card_adapter.rs`'s own module-doc hazard: "`total_bits` ... MUST be
/// carried verbatim ... never `bytes.len() * 8`").
const MD1_PAYLOAD_BITS: usize = 644;

/// The FROZEN engraved word sequence for `encode(Md1Descriptor, &MD1_PAYLOAD,
/// MD1_PAYLOAD_BITS, &golden_opts())` -- generated once, 2026-07-10, and pinned
/// literally. 92 words.
const MD1_WORDS: [&str; 92] = [
    "advice", "action", "cereal", "gas", "pumpkin", "thing", "series", "abandon", "abandon",
    "abandon", "abandon", "cactus", "jeans", "embark", "avoid", "alcohol", "accident", "domain",
    "detail", "pear", "unfair", "choose", "popular", "announce", "father", "aim", "silent",
    "inhale", "pink", "sing", "select", "illness", "spoil", "pause", "angle", "fatal", "always",
    "powder", "auction", "paper", "staff", "citizen", "shoot", "nut", "artefact", "material",
    "pumpkin", "grab", "embody", "useless", "evoke", "kidney", "blood", "hen", "luggage", "rare",
    "tunnel", "swallow", "unaware", "lobster", "jungle", "dash", "forum", "scan", "resource",
    "liar", "magnet", "detect", "coast", "ticket", "bitter", "minor", "typical", "rice", "deliver",
    "essence", "home", "have", "vessel", "capable", "scale", "salad", "couple", "easily", "random",
    "drift", "finger", "normal", "like", "crater", "vague", "shoe",
];

#[test]
fn md1_kind_wire_golden() {
    let words = encode(
        SourceKind::Md1Descriptor,
        &MD1_PAYLOAD,
        MD1_PAYLOAD_BITS,
        &golden_opts(),
    )
    .expect("encode md1-kind golden");
    assert_eq!(
        words, MD1_WORDS,
        "md1-kind wire golden mismatch -- the engraved word sequence for this \
         frozen non-byte-aligned (payload, payload_bits, m, t, u) tuple has \
         changed; an already-engraved plate would now brick"
    );
    let decoded = decode(&MD1_WORDS).expect("decode md1-kind golden");
    assert_eq!(decoded.kind, SourceKind::Md1Descriptor);
    assert_eq!(decoded.payload, MD1_PAYLOAD);
    assert_eq!(decoded.payload_bits, MD1_PAYLOAD_BITS);
}

// ===========================================================================
// RAID golden (iii) -- n=3, r=2 array, FIXED `array_id_seed`.
// ===========================================================================

/// Three DISTINCT mk1 cards' canonical payloads (the `RAID_MK1`/`RAID_MK2`/
/// `RAID_MK3` fixtures frozen at `cli_word_card.rs:28-30`), each 84 bytes /
/// 672 bits (byte-aligned).
const RAID_PAYLOAD_0: [u8; 84] = [
    0x04, 0x01, 0x1c, 0x01, 0x70, 0xfe, 0x73, 0xc5, 0xda, 0x0a, 0x03, 0x04, 0x88, 0xb2, 0x1e, 0x7e,
    0xf3, 0x2b, 0xdb, 0x4a, 0x53, 0xa0, 0xab, 0x21, 0xb9, 0xdc, 0x95, 0x86, 0x9c, 0x4e, 0x92, 0xa1,
    0x61, 0x19, 0x4e, 0x03, 0xc0, 0xef, 0x3f, 0xf5, 0x01, 0x4a, 0xc6, 0x92, 0xf4, 0x33, 0xc4, 0x76,
    0x54, 0x90, 0xfc, 0x02, 0x70, 0x7a, 0x62, 0xfd, 0xac, 0xc2, 0x6e, 0xa9, 0xb6, 0x3b, 0x1c, 0x19,
    0x79, 0x06, 0xf5, 0x6e, 0xe0, 0x18, 0x0d, 0x0b, 0xcf, 0x19, 0x66, 0xe1, 0xa2, 0xda, 0x34, 0xf5,
    0xf3, 0xa0, 0x9a, 0x9b,
];
const RAID_PAYLOAD_1: [u8; 84] = [
    0x04, 0x01, 0x7f, 0xa8, 0x85, 0x95, 0xb8, 0x68, 0x8d, 0xf1, 0x03, 0x04, 0x88, 0xb2, 0x1e, 0xea,
    0x51, 0x7e, 0xe5, 0xe3, 0xbb, 0x85, 0x12, 0xda, 0xb0, 0xb1, 0x73, 0x2c, 0xdb, 0x9a, 0x94, 0xf4,
    0x2a, 0x4d, 0x05, 0xf2, 0xfe, 0xcd, 0x5a, 0x6f, 0x1f, 0x29, 0x68, 0x39, 0x0b, 0x68, 0xb6, 0x34,
    0x06, 0xde, 0xfd, 0x02, 0x2f, 0xad, 0x1f, 0x6a, 0xb3, 0x60, 0x67, 0x6c, 0x12, 0x39, 0x9e, 0x0d,
    0x9b, 0xd9, 0xdc, 0x74, 0x42, 0xbb, 0xa7, 0xd2, 0x02, 0x00, 0x82, 0x84, 0x20, 0x42, 0x45, 0x28,
    0x91, 0x73, 0x95, 0x14,
];
const RAID_PAYLOAD_2: [u8; 84] = [
    0x04, 0x01, 0x3f, 0xfb, 0x90, 0x95, 0x28, 0x64, 0x50, 0x06, 0x03, 0x04, 0x88, 0xb2, 0x1e, 0xd0,
    0x61, 0xc2, 0x0c, 0x08, 0x3b, 0x17, 0xa7, 0x79, 0xdb, 0x44, 0xbd, 0x19, 0x9c, 0xe5, 0x13, 0x15,
    0x92, 0x6c, 0x35, 0x11, 0x81, 0x02, 0x9b, 0x4d, 0xe3, 0xe6, 0x91, 0x88, 0xf9, 0xf0, 0x10, 0x4d,
    0xa1, 0xf1, 0xb6, 0x02, 0xb3, 0xe3, 0x67, 0x62, 0x16, 0x57, 0x04, 0x2f, 0x41, 0x07, 0x3b, 0xec,
    0x2c, 0x8f, 0x5d, 0x71, 0x57, 0x68, 0x6b, 0x9c, 0xb7, 0x3b, 0xac, 0xe7, 0x2f, 0xd6, 0xcb, 0xcf,
    0x31, 0xb2, 0xcd, 0x4c,
];
const RAID_PAYLOAD_BITS: usize = 672;

/// A FIXED `array_id_seed` literal (`raid_encode` draws it from the caller --
/// `raid.rs:268-270` -- this golden pins a fixed value rather than the
/// entropy-drawing CLI path). An arbitrary but frozen byte string standing in
/// for "the concatenated ordered cosigner fingerprints".
const ARRAY_ID_SEED: &[u8] = b"T3-a-frozen-array-id-seed-v1";

/// The FROZEN engraved word sequences for `raid_encode(&[(RAID_PAYLOAD_0,
/// RAID_PAYLOAD_BITS), (RAID_PAYLOAD_1, RAID_PAYLOAD_BITS), (RAID_PAYLOAD_2,
/// RAID_PAYLOAD_BITS)], ARRAY_ID_SEED, 2, &EncodeOpts::default())` -- generated
/// once, 2026-07-10, and pinned literally. Plate order: 3 data (index 0..2),
/// then ParityA (index 3), then ParityB (index 4); 94 words each.
const RAID_PLATE_0_WORDS: [&str; 94] = [
    "acoustic", "avoid", "abandon", "differ", "immense", "actress", "airport", "gas", "chuckle",
    "thing", "trophy", "abandon", "abandon", "abandon", "abandon", "actor", "abandon", "advice",
    "angry", "able", "tiger", "transfer", "title", "park", "habit", "document", "afraid", "easily",
    "maple", "worth", "crawl", "unit", "pistol", "citizen", "inject", "private", "assist",
    "jaguar", "climb", "hawk", "chef", "prize", "enhance", "club", "bone", "hybrid", "usual",
    "taxi", "young", "level", "quarter", "climb", "spoil", "vintage", "owner", "budget", "powder",
    "average", "abuse", "quote", "sea", "era", "world", "gravity", "option", "fat", "glove", "mix",
    "release", "grass", "camp", "voice", "jacket", "alcohol", "borrow", "rude", "million",
    "reward", "oppose", "crowd", "surface", "police", "language", "donkey", "predict", "length",
    "rough", "treat", "iron", "swing", "alone", "physical", "vague", "used",
];
const RAID_PLATE_1_WORDS: [&str; 94] = [
    "acoustic", "avoid", "length", "differ", "immense", "actress", "airport", "gas", "ride",
    "thing", "trophy", "abandon", "abandon", "abandon", "abandon", "actor", "abandon", "advice",
    "armor", "tuition", "arctic", "fortune", "crowd", "permit", "daughter", "call", "afraid",
    "easily", "marble", "start", "blood", "indicate", "pole", "moment", "idle", "barrel", "stereo",
    "clump", "town", "soap", "snap", "predict", "fatigue", "luxury", "escape", "convince",
    "sausage", "one", "plunge", "moon", "protect", "pizza", "already", "arena", "east", "minimum",
    "asset", "used", "library", "range", "salt", "physical", "height", "only", "border", "subway",
    "castle", "someone", "release", "cushion", "kite", "symbol", "pear", "puzzle", "exile", "cage",
    "able", "rhythm", "anxiety", "lottery", "drastic", "citizen", "cattle", "transfer", "eyebrow",
    "abandon", "satisfy", "elegant", "bind", "entry", "laugh", "pear", "vague", "three",
];
const RAID_PLATE_2_WORDS: [&str; 94] = [
    "acoustic", "awake", "abandon", "differ", "immense", "actress", "airport", "gas", "armor",
    "thing", "trophy", "abandon", "abandon", "abandon", "abandon", "actor", "abandon", "advice",
    "antique", "year", "cancel", "family", "cram", "pepper", "divorce", "scatter", "afraid",
    "easily", "marble", "patch", "bring", "alcohol", "pony", "can", "glass", "polar", "solution",
    "spell", "kick", "oil", "income", "position", "era", "rather", "history", "possible", "gate",
    "ahead", "regular", "vehicle", "puppy", "olympic", "country", "wheat", "letter", "beach",
    "dumb", "bread", "access", "rabbit", "sort", "hollow", "ginger", "razor", "scissors", "future",
    "away", "over", "reject", "radar", "music", "frost", "melt", "sure", "strong", "slice", "oven",
    "risk", "receive", "tourist", "strategy", "fury", "small", "grain", "state", "abandon",
    "route", "focus", "match", "pool", "supply", "payment", "vague", "tortoise",
];
const RAID_PLATE_3_WORDS_PARITY_A: [&str; 94] = [
    "acoustic", "bamboo", "abandon", "differ", "immense", "actress", "airport", "gas", "cabin",
    "thing", "trophy", "abandon", "abandon", "abandon", "abandon", "actor", "abandon", "advice",
    "april", "behind", "slogan", "until", "tonight", "pave", "cable", "parrot", "afraid", "easily",
    "maple", "maze", "defense", "off", "pink", "mansion", "green", "burden", "bird", "medal",
    "resist", "diet", "obtain", "pride", "fence", "brass", "debris", "warfare", "cool", "draft",
    "uniform", "yard", "punch", "belt", "tongue", "allow", "force", "loop", "trash", "window",
    "lift", "rapid", "radio", "local", "vital", "exile", "drop", "still", "expect", "table",
    "reject", "tobacco", "skate", "inner", "garden", "indicate", "little", "fish", "bachelor",
    "ring", "capital", "fiber", "exchange", "spatial", "network", "seminar", "club", "length",
    "sauce", "rain", "only", "coach", "artefact", "peace", "vague", "they",
];
const RAID_PLATE_4_WORDS_PARITY_B: [&str; 94] = [
    "acoustic", "beef", "abandon", "differ", "immense", "actress", "airport", "gas", "ecology",
    "thing", "trophy", "abandon", "abandon", "abandon", "abandon", "aspect", "abandon", "bright",
    "december", "conduct", "industry", "box", "drill", "photo", "alpha", "absorb", "buzz",
    "subway", "nut", "stone", "hospital", "away", "pigeon", "response", "bunker", "bar", "orient",
    "hello", "audit", "nerve", "jelly", "present", "syrup", "unusual", "cricket", "middle",
    "attack", "yellow", "gather", "floor", "puppy", "anger", "blossom", "afraid", "bike", "enjoy",
    "run", "runway", "alley", "rally", "loop", "retreat", "park", "opinion", "punch", "odor",
    "gown", "grid", "renew", "slab", "motion", "deal", "blur", "crater", "subway", "speak", "whip",
    "reveal", "dolphin", "pride", "affair", "metal", "advice", "bonus", "flush", "length", "round",
    "december", "area", "zoo", "jacket", "pencil", "vague", "useful",
];

#[test]
fn raid_n3_r2_wire_golden() {
    let payloads = vec![
        (RAID_PAYLOAD_0.to_vec(), RAID_PAYLOAD_BITS),
        (RAID_PAYLOAD_1.to_vec(), RAID_PAYLOAD_BITS),
        (RAID_PAYLOAD_2.to_vec(), RAID_PAYLOAD_BITS),
    ];
    let plates = raid_encode(&payloads, ARRAY_ID_SEED, 2, &EncodeOpts::default())
        .expect("raid_encode n=3 r=2 golden");
    assert_eq!(plates.len(), 5, "3 data + ParityA + ParityB");

    assert_eq!(plates[0].role, PlateRole::Data);
    assert_eq!(plates[0].index, 0);
    assert_eq!(
        plates[0].words, RAID_PLATE_0_WORDS,
        "RAID data-plate 0 wire golden mismatch"
    );

    assert_eq!(plates[1].role, PlateRole::Data);
    assert_eq!(plates[1].index, 1);
    assert_eq!(
        plates[1].words, RAID_PLATE_1_WORDS,
        "RAID data-plate 1 wire golden mismatch"
    );

    assert_eq!(plates[2].role, PlateRole::Data);
    assert_eq!(plates[2].index, 2);
    assert_eq!(
        plates[2].words, RAID_PLATE_2_WORDS,
        "RAID data-plate 2 wire golden mismatch"
    );

    assert_eq!(plates[3].role, PlateRole::ParityA);
    assert_eq!(plates[3].index, 3);
    assert_eq!(
        plates[3].words, RAID_PLATE_3_WORDS_PARITY_A,
        "RAID ParityA wire golden mismatch"
    );

    assert_eq!(plates[4].role, PlateRole::ParityB);
    assert_eq!(plates[4].index, 4);
    assert_eq!(
        plates[4].words, RAID_PLATE_4_WORDS_PARITY_B,
        "RAID ParityB wire golden mismatch"
    );

    // The golden set must still reconstruct (sanity oracle check).
    let sets: Vec<Vec<&str>> = vec![
        RAID_PLATE_0_WORDS.to_vec(),
        RAID_PLATE_1_WORDS.to_vec(),
        RAID_PLATE_2_WORDS.to_vec(),
        RAID_PLATE_3_WORDS_PARITY_A.to_vec(),
        RAID_PLATE_4_WORDS_PARITY_B.to_vec(),
    ];
    let rec = wc_codec::raid_reconstruct(&sets).expect("reconstruct golden RAID array");
    assert_eq!(
        rec.payloads[0],
        (RAID_PAYLOAD_0.to_vec(), RAID_PAYLOAD_BITS)
    );
    assert_eq!(
        rec.payloads[1],
        (RAID_PAYLOAD_1.to_vec(), RAID_PAYLOAD_BITS)
    );
    assert_eq!(
        rec.payloads[2],
        (RAID_PAYLOAD_2.to_vec(), RAID_PAYLOAD_BITS)
    );
}
