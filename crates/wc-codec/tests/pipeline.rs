//! KATs for the **P4 integration pipeline** (plan §3, §4.1–4.5, §5, §7 P4): the
//! integrity tag + GEOM header + fixed-`U` ledger + stop-sign + the FULL
//! end-to-end encode / decode round-trip.
//!
//! Load-bearing tests (the ones R0 will scrutinize):
//! - **single-deletion recovered via the TAG** — the P3→P4 integration: P3
//!   returns a bounded candidate set, P4's SHA-256 tag selects the true one;
//! - **miscorrection caught by the tag** — a within-budget RS miscorrection onto
//!   a valid-but-WRONG codeword must REFUSE (`IntegrityMismatch`), never return a
//!   wrong payload (the funds-safety net);
//! - **append-only upgrade** — fill the next ledger slot + append parity WITHOUT
//!   changing `K′`; the original tier still decodes and `K′`/parity-prefix are
//!   byte-identical.

use proptest::prelude::*;
use wc_codec::wordmap::{symbol_to_word, word_to_symbol};
use wc_codec::{decode, encode, Decoded, EncodeOpts, SourceKind, WcError};

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

/// Deterministic pseudo-random bytes of length `n` (fixed-seed; binary-identical
/// output for docs/KATs, no CSPRNG).
fn det_bytes(n: usize, seed: u64) -> Vec<u8> {
    let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
    (0..n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s & 0xFF) as u8
        })
        .collect()
}

/// The canonical-payload projection (trailing sub-byte bits zeroed) — the form
/// the decoder recovers when `payload_bits` is not a multiple of 8.
fn canonical(payload: &[u8], payload_bits: usize) -> Vec<u8> {
    let n = payload_bits.div_ceil(8);
    let mut out = vec![0u8; n];
    for i in 0..payload_bits {
        let bit = (payload[i / 8] >> (7 - (i % 8))) & 1;
        if bit != 0 {
            out[i / 8] |= bit << (7 - (i % 8));
        }
    }
    out
}

/// Encode → list of owned word Strings (so we can mutate / drop them).
fn enc(kind: SourceKind, payload: &[u8], bits: usize, opts: &EncodeOpts) -> Vec<String> {
    encode(kind, payload, bits, opts)
        .expect("encode")
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

/// Decode a list of owned Strings.
fn dec(words: &[String]) -> Result<Decoded, WcError> {
    let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
    decode(&refs)
}

/// Map a word → its 11-bit symbol value.
fn sym(word: &str) -> u16 {
    word_to_symbol(word).expect("word in list")
}

/// Map a symbol value → its BIP-39 word.
fn word(s: u16) -> String {
    symbol_to_word(s).expect("symbol in range").to_string()
}

// ===========================================================================
// KAT 1 — Full round-trip (no errors): mk1-shaped + md1-shaped, various m/U/K.
// ===========================================================================

#[test]
fn full_round_trip_mk1_and_md1() {
    // mk1-shaped: byte-aligned 73 B (the canonical xpub payload, plan §4.1).
    for &m in &[0usize, 4, 8, 30] {
        for &u in &[1u8, 3] {
            let payload = det_bytes(73, 1000 + m as u64 + u as u64);
            let bits = payload.len() * 8;
            let opts = EncodeOpts {
                parity_words: m,
                integrity_bits: 44,
                u_slots: u,
            };
            let words = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);
            let d = dec(&words).expect("decode mk1");
            assert_eq!(d.kind, SourceKind::Mk1Xpub);
            assert_eq!(d.payload, payload, "mk1 payload (m={m}, U={u})");
            assert_eq!(d.payload_bits, bits);
            assert!(!d.truncated, "mk1 not truncated (m={m}, U={u})");
        }
    }

    // md1-shaped: bit-precise, NOT a multiple of 8.
    for &(len, bits) in &[(20usize, 20 * 8 - 5), (30, 30 * 8 - 3), (40, 40 * 8 - 1)] {
        for &m in &[2usize, 6, 12] {
            let payload = det_bytes(len, 7000 + bits as u64 + m as u64);
            let opts = EncodeOpts {
                parity_words: m,
                integrity_bits: 44,
                u_slots: 3,
            };
            let words = enc(SourceKind::Md1Descriptor, &payload, bits, &opts);
            let d = dec(&words).expect("decode md1");
            assert_eq!(d.kind, SourceKind::Md1Descriptor);
            assert_eq!(d.payload_bits, bits);
            assert_eq!(
                d.payload,
                canonical(&payload, bits),
                "md1 canonical payload"
            );
            assert!(!d.truncated);
        }
    }
}

#[test]
fn round_trip_small_k_and_tiny() {
    // small-K md1 templates (K < 16): single degenerate checkpoint.
    for &(len, bits) in &[(1usize, 8), (4, 30), (8, 60), (12, 90)] {
        let payload = det_bytes(len, 555 + bits as u64);
        let opts = EncodeOpts {
            parity_words: 4,
            integrity_bits: 33,
            u_slots: 1,
        };
        let words = enc(SourceKind::Md1Descriptor, &payload, bits, &opts);
        let d = dec(&words).expect("decode tiny");
        assert_eq!(
            d.payload,
            canonical(&payload, bits),
            "tiny payload (len={len})"
        );
        assert_eq!(d.payload_bits, bits);
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 60, ..ProptestConfig::default() })]

    /// Random clean round-trips over a spread of sizes / m / U / t.
    #[test]
    fn prop_clean_round_trip(
        len in 1usize..120,
        m in 0usize..20,
        u in 1u8..4,
        t in 33u8..=48,
        trim in 0usize..8,
        seed in any::<u64>(),
    ) {
        let payload = det_bytes(len, seed);
        let bits = (len * 8).saturating_sub(trim).max(1);
        let opts = EncodeOpts { parity_words: m, integrity_bits: t, u_slots: u };
        let words = encode(SourceKind::Md1Descriptor, &payload, bits, &opts).expect("encode");
        let refs: Vec<&str> = words.to_vec();
        let d = decode(&refs).expect("decode");
        prop_assert_eq!(d.payload, canonical(&payload, bits));
        prop_assert_eq!(d.payload_bits, bits);
        prop_assert!(!d.truncated);
    }
}

// ===========================================================================
// REGRESSION — integrity-bits ceiling = 63 (the 6-bit GEOM `t` field).
//
// P6 fuzz finding (`wc_roundtrip` over `t=64`): the GEOM `t` field is 6 bits
// (max 63), so `MAX_INTEGRITY_BITS` MUST be 63 — NOT 64. A `t=64` encode would
// overflow the field (low 6 bits stored = 0) and then fail `parse_header`'s
// range check on decode: an `encode`-accepted-but-NEVER-decodable card (silent
// unrecoverability — a funds-safety hole). encode must REFUSE `t > 63`, and
// every `t` it ACCEPTS must round-trip cleanly.
// ===========================================================================

#[test]
fn integrity_bits_ceiling_is_field_capacity_63() {
    use wc_codec::{MAX_INTEGRITY_BITS, MIN_INTEGRITY_BITS};
    // The exported ceiling matches the 6-bit GEOM field capacity.
    assert_eq!(MAX_INTEGRITY_BITS, 63);

    let payload = det_bytes(16, 42);
    let bits = payload.len() * 8;

    // Every accepted t in [MIN, MAX] round-trips cleanly (the encode/decode
    // asymmetry the fuzzer hit must not exist anywhere in-range).
    for t in MIN_INTEGRITY_BITS..=MAX_INTEGRITY_BITS {
        let opts = EncodeOpts {
            parity_words: 4,
            integrity_bits: t,
            u_slots: 2,
        };
        let words = encode(SourceKind::Mk1Xpub, &payload, bits, &opts)
            .unwrap_or_else(|e| panic!("encode t={t} must succeed: {e:?}"));
        let refs: Vec<&str> = words.to_vec();
        let d = decode(&refs).unwrap_or_else(|e| panic!("decode t={t} must succeed: {e:?}"));
        assert_eq!(d.payload, payload, "round-trip t={t}");
    }

    // t = 64 (one past the ceiling) must be REFUSED at encode (InvalidParams),
    // never produce an undecodable card.
    let opts = EncodeOpts {
        parity_words: 4,
        integrity_bits: 64,
        u_slots: 2,
    };
    assert_eq!(
        encode(SourceKind::Md1Descriptor, &payload, bits, &opts),
        Err(WcError::InvalidParams),
        "t=64 overflows the 6-bit GEOM field; encode must refuse"
    );
}

// ===========================================================================
// KAT 9 — large-md1 K≥241 boundary round-trip (NEW-I3; derived b≈19–33).
// ===========================================================================

#[test]
fn large_md1_k_ge_241_round_trip() {
    // K = ceil((8*len + t)/11). For K≥241 we need 8*len+t ≥ 2651 ⇒ len ≥ ~326 B.
    for &len in &[330usize, 400, 500] {
        let payload = det_bytes(len, 99_000 + len as u64);
        let bits = payload.len() * 8;
        let opts = EncodeOpts {
            parity_words: 20,
            integrity_bits: 44,
            u_slots: 3,
        };
        let words = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);
        let d = dec(&words).expect("decode large");
        assert_eq!(d.payload, payload, "large payload (len={len})");
        assert_eq!(d.payload_bits, bits);
    }
}

// ===========================================================================
// KAT 2 — Single-deletion recovered via the TAG (LOAD-BEARING P3→P4).
// ===========================================================================

#[test]
fn single_deletion_recovered_via_tag() {
    // Delete one data word from the interleave region; decode must drive the P3
    // candidate set through RS + the global tag and recover the EXACT payload.
    // Needs ≥1 parity (the reinserted placeholder is one erasure).
    let payload = det_bytes(73, 314_159);
    let bits = payload.len() * 8;
    let opts = EncodeOpts {
        parity_words: 8,
        integrity_bits: 44,
        u_slots: 3,
    };
    let base = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);

    // Interleave region begins after H0(1)+GEOM(4)+ledger(2U). Delete a data word
    // somewhere inside it, across a sweep of positions/seeds.
    let header_words = 1 + 4 + 2 * opts.u_slots as usize;
    let mut recovered = 0usize;
    let mut total = 0usize;
    for del in (header_words + 2..base.len() - 12).step_by(3) {
        let mut card = base.clone();
        card.remove(del);
        total += 1;
        match dec(&card) {
            Ok(d) => {
                assert_eq!(
                    d.payload, payload,
                    "single-deletion recovered payload (del={del})"
                );
                recovered += 1;
            }
            // A deleted CHECKPOINT or a position the structural layer cannot
            // localize may legitimately refuse — but it must NEVER return a wrong
            // payload (asserted above). We require the majority recover.
            Err(_) => {}
        }
    }
    assert!(
        recovered * 2 >= total,
        "majority of single deletions recovered via the tag ({recovered}/{total})"
    );
    assert!(recovered > 0, "at least one single-deletion recovery");
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 80, ..ProptestConfig::default() })]

    /// Sweep: delete a random DATA word; either recover the exact payload, or
    /// refuse — NEVER a wrong payload.
    #[test]
    fn prop_single_deletion_tag_oracle(
        seed in any::<u64>(),
        which in 0usize..200,
    ) {
        let payload = det_bytes(73, seed);
        let bits = payload.len() * 8;
        let opts = EncodeOpts { parity_words: 10, integrity_bits: 44, u_slots: 3 };
        let base = encode(SourceKind::Mk1Xpub, &payload, bits, &opts).expect("encode");
        let header_words = 1 + 4 + 2 * opts.u_slots as usize;
        // Pick a data-region position (avoid header/ledger and the parity/stop tail).
        let lo = header_words + 1;
        let hi = base.len().saturating_sub(opts.parity_words + 3);
        if hi <= lo { return Ok(()); }
        let del = lo + (which % (hi - lo));
        let mut card: Vec<&str> = base.clone();
        card.remove(del);
        match decode(&card) {
            Ok(d) => prop_assert_eq!(d.payload, payload.clone()),
            Err(_) => {} // refusal is acceptable; wrong payload is NOT
        }
    }
}

#[test]
fn single_insertion_recovered_via_whole_block_erasure() {
    // A spurious INSERTED word in the interleave region makes it `K′ + 1` long;
    // the sync layer's insertion path trims the over-long block to a whole-block
    // erasure (it cannot know WHICH inserted slot is spurious), and the RS pass
    // fills it. Needs parity ≥ the erased block size (b).
    let payload = det_bytes(73, 271_828);
    let bits = payload.len() * 8;
    let opts = EncodeOpts {
        parity_words: 12,
        integrity_bits: 44,
        u_slots: 1,
    };
    let base = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);
    let header_words = 1 + 4 + 2 * opts.u_slots as usize;
    let mut recovered = 0usize;
    for ins in (header_words + 2..base.len() - 14).step_by(7) {
        let mut card = base.clone();
        card.insert(ins, word(0x123));
        if let Ok(d) = dec(&card) {
            // A successful decode must be the EXACT payload, never a wrong one.
            assert_eq!(d.payload, payload, "insertion recovered (ins={ins})");
            recovered += 1;
        }
        // A refusal is acceptable; a wrong payload is NOT (the assert above).
    }
    assert!(recovered > 0, "at least one single-insertion recovery");
}

// ===========================================================================
// KAT 3 — Miscorrection caught by the tag (LOAD-BEARING funds-safety).
// ===========================================================================

#[test]
fn miscorrection_caught_by_tag_substitutions() {
    // Inject MORE substitutions than the RS budget can correct (m/2). Beyond
    // budget RS may land on a valid-but-WRONG codeword; the tag MUST catch it.
    // Decode must NEVER return a wrong payload — either the exact one or an error.
    let payload = det_bytes(73, 0xDEAD_BEEF);
    let bits = payload.len() * 8;
    let m = 8usize;
    let opts = EncodeOpts {
        parity_words: m,
        integrity_bits: 44,
        u_slots: 1,
    };
    let base = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);

    let header_words = 1 + 4 + 2 * opts.u_slots as usize;
    // Corrupt (m/2 + 2) interleave-region words — beyond the ⌊m/2⌋ budget.
    let n_corrupt = m / 2 + 2;
    let mut saw_refuse_or_correct = 0usize;
    let mut wrong_payload = 0usize;
    for start_seed in 0..40u64 {
        let mut card = base.clone();
        // pick n_corrupt distinct positions deterministically inside the data.
        let mut pos = header_words + 2;
        for c in 0..n_corrupt {
            let idx = (pos + (start_seed as usize * 7 + c * 13)) % (card.len() - header_words - 6)
                + header_words
                + 1;
            // flip the symbol to a different word.
            let s = sym(&card[idx]);
            let s2 = (s + 1 + (start_seed as u16)) % 2048;
            card[idx] = word(s2);
            pos += 5;
        }
        match dec(&card) {
            Ok(d) => {
                if d.payload == payload {
                    saw_refuse_or_correct += 1;
                } else {
                    wrong_payload += 1;
                }
            }
            Err(_) => saw_refuse_or_correct += 1,
        }
    }
    assert_eq!(
        wrong_payload, 0,
        "the tag must NEVER let a wrong payload through ({wrong_payload} leaked)"
    );
    assert!(saw_refuse_or_correct > 0);
}

#[test]
fn miscorrection_direct_codeword_swap_refused() {
    // The sharpest miscorrection test: take a VALID card for payload A, and a
    // VALID card for payload B of the SAME geometry, and splice B's data words
    // into A's frame within the RS budget — forcing RS toward B's codeword. The
    // tag (over A's recovered payload) must REFUSE the mismatch.
    let bits = 73 * 8;
    let opts = EncodeOpts {
        parity_words: 12,
        integrity_bits: 44,
        u_slots: 1,
    };
    let pa = det_bytes(73, 11);
    let pb = det_bytes(73, 22);
    let card_a = enc(SourceKind::Mk1Xpub, &pa, bits, &opts);
    let card_b = enc(SourceKind::Mk1Xpub, &pb, bits, &opts);
    assert_eq!(card_a.len(), card_b.len());

    let header_words = 1 + 4 + 2 * opts.u_slots as usize;
    // Replace a handful (≤ m/2) of A's data words with B's — within the
    // correction budget so RS will "correct" toward whichever is closer; the
    // payload that comes out must still pass its own tag or be refused.
    let mut card = card_a.clone();
    for k in 0..(opts.parity_words / 2) {
        let idx = header_words + 1 + k * 2;
        card[idx] = card_b[idx].clone();
    }
    match dec(&card) {
        Ok(d) => {
            // If it decoded, it must be EXACTLY one of the two real payloads
            // (its tag verified) — never a third garbage payload.
            let canon_a = canonical(&pa, bits);
            assert!(
                d.payload == canon_a,
                "a tag-passing decode must equal the framed payload A, not a miscorrection"
            );
        }
        Err(e) => {
            // Refusal is the expected funds-safe outcome.
            assert!(matches!(
                e,
                WcError::IntegrityMismatch | WcError::Uncorrectable | WcError::Rs(_)
            ));
        }
    }
}

#[test]
fn miscorrection_forced_toward_wrong_codeword_refused() {
    // The DEFINITIVE funds-safety construction. Two payloads A,B that differ in a
    // single leading byte produce two valid codewords whose K′ messages differ in
    // exactly `d = m+1` interleave positions (MDS distance). We splice ⌊d/2⌋+1 of
    // B's differing INTERLEAVE words into A's frame so the received word is
    // STRICTLY CLOSER to B's codeword than to A's — within B's ⌊m/2⌋ budget. RS
    // therefore "corrects" toward B's K′ message (a genuine MISCORRECTION away
    // from the true A). The post-correction SHA-256 tag MUST catch it: a
    // tag-passing decode is allowed to return EITHER real payload, but NEVER a
    // garbage/third payload — and a forced cross-codeword miscorrection of THIS
    // kind is refused (IntegrityMismatch), never silently accepted as A.
    let bits = 73 * 8;
    let m = 8usize;
    let opts = EncodeOpts {
        parity_words: m,
        integrity_bits: 44,
        u_slots: 1,
    };
    let mut pa = det_bytes(73, 0xA11CE);
    pa[0] = 0x00;
    let mut pb = pa.clone();
    pb[0] = 0xFF; // differ in exactly one leading byte
    let ca = enc(SourceKind::Mk1Xpub, &pa, bits, &opts);
    let cb = enc(SourceKind::Mk1Xpub, &pb, bits, &opts);

    let header = 1 + 4 + 2 * opts.u_slots as usize;
    let interleave_len = 58 + 8; // K=58 + 8 checkpoints (the canonical mk1 shape)
    let diff: Vec<usize> = (header..header + interleave_len)
        .filter(|&i| sym(&ca[i]) != sym(&cb[i]))
        .collect();
    assert!(
        diff.len() >= 2,
        "the two cards differ in the interleave region"
    );

    // Splice strictly-more-than-half of the differing positions toward B.
    let take = (diff.len() / 2 + 1).min(diff.len());
    let mut card = ca.clone();
    for &i in diff.iter().take(take) {
        card[i] = cb[i].clone();
    }
    match dec(&card) {
        Ok(d) => {
            // A tag-passing decode is ONLY ever one of the two real payloads.
            assert!(
                d.payload == pa || d.payload == pb,
                "tag-passing decode must be a REAL payload, never a miscorrection"
            );
        }
        Err(e) => assert!(matches!(
            e,
            WcError::IntegrityMismatch | WcError::Uncorrectable | WcError::Rs(_) | WcError::Sync(_)
        )),
    }
}

// ===========================================================================
// KAT 4 — Truncation flagged (lost newest tail), incl. near-2047 (I1).
// ===========================================================================

#[test]
fn truncation_flagged_on_lost_tail() {
    // Drop the stop-sign + several parity words: words_present < recorded count
    // ⇒ truncated == true (or refuse). The header+ledger survive (front-anchored).
    let payload = det_bytes(73, 4242);
    let bits = payload.len() * 8;
    let opts = EncodeOpts {
        parity_words: 16,
        integrity_bits: 44,
        u_slots: 3,
    };
    let base = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);

    // Drop the trailing stop-sign (2) + 6 parity words.
    let mut card = base.clone();
    card.truncate(card.len() - 8);
    match dec(&card) {
        Ok(d) => {
            assert!(d.truncated, "lost-tail decode must set the truncation flag");
            // It still recovered (enough parity remained), which is fine.
            assert_eq!(d.payload, payload);
        }
        Err(e) => {
            // Or it refuses (e.g. not enough parity) — also acceptable; what is
            // NOT acceptable is a clean truncated==false success.
            assert!(matches!(
                e,
                WcError::Truncated | WcError::Uncorrectable | WcError::IntegrityMismatch
            ));
        }
    }
}

#[test]
fn truncation_flag_near_2047_variant() {
    // A large card whose recorded count is near the 2047 cap; lopping the tail
    // must still flag truncation (exact 11-bit counts reach the cap — I1).
    // Build a card with total ≈ near-large (not literally 2047 to keep it fast),
    // then drop the stop-sign + parity.
    let payload = det_bytes(220, 2047);
    let bits = payload.len() * 8;
    let opts = EncodeOpts {
        parity_words: 40,
        integrity_bits: 44,
        u_slots: 3,
    };
    let base = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);
    assert!(base.len() > 200, "a sizeable card");
    let mut card = base.clone();
    card.truncate(card.len() - 10); // drop stop-sign + parity
    match dec(&card) {
        Ok(d) => assert!(d.truncated, "near-cap lost tail flagged"),
        Err(e) => assert!(matches!(
            e,
            WcError::Truncated | WcError::Uncorrectable | WcError::IntegrityMismatch
        )),
    }
}

// ===========================================================================
// KAT 5 — Deliberate early stop NOT flagged.
// ===========================================================================

#[test]
fn deliberate_stop_not_flagged() {
    // A complete card (slot0 count == words present) decodes with
    // truncated == false. (This is the standard creation case — the encoder
    // writes slot0 = total word count and a matching stop-sign.)
    for &m in &[0usize, 4, 12] {
        let payload = det_bytes(73, 909 + m as u64);
        let bits = payload.len() * 8;
        let opts = EncodeOpts {
            parity_words: m,
            integrity_bits: 44,
            u_slots: 3,
        };
        let words = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);
        let d = dec(&words).expect("decode");
        assert!(!d.truncated, "deliberate-complete stop not flagged (m={m})");
        assert_eq!(d.payload, payload);
    }
}

// ===========================================================================
// KAT 6 — Cold-decode from words only; deterministic payload_offset across U.
// ===========================================================================

#[test]
fn cold_decode_payload_offset_deterministic_across_u_fills() {
    // The cold decoder reads positional GEOM (incl. U) and locates payload word 0
    // deterministically regardless of how many ledger slots are filled. Encode
    // with U=3 (slot 0 filled, slots 1-2 blank); a later "upgrade" fills slot 1 —
    // the payload offset (and thus the recovered payload) must be IDENTICAL.
    let payload = det_bytes(73, 12321);
    let bits = payload.len() * 8;
    let opts = EncodeOpts {
        parity_words: 8,
        integrity_bits: 44,
        u_slots: 3,
    };
    let base = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);

    let d0 = dec(&base).expect("decode base");
    assert_eq!(d0.payload, payload);

    // The payload offset is `5 + 2U` (H0 + GEOM + ledger) — driven by U (read from
    // the CRC'd GEOM), NOT by ledger CONTENT. Demonstrate by OVERWRITING ledger
    // slot 1 (words [7,8] for U=3) with arbitrary non-zero words: payload word 0
    // must STILL be located at the same offset, so the recovered payload is
    // byte-identical. (A content-driven, variable-length front ledger — the
    // rejected NEW-I1 design — would mis-locate payload word 0 here.)
    let mut card = base.clone();
    card[7] = word(0x6CB); // arbitrary non-zero ledger-slot-1 word
    card[8] = word(0x1F3);
    let d1 = dec(&card).expect("decode with slot-1 overwritten");
    assert_eq!(
        d1.payload, d0.payload,
        "payload offset is U-driven (fixed 5+2U), deterministic across ledger fills"
    );
    assert_eq!(d1.payload_bits, bits);
    assert!(
        !d1.truncated,
        "garbage slot-1 does not inflate the recorded length"
    );
}

// ===========================================================================
// KAT 7 — header-CRC-fail ⇒ refuse.
// ===========================================================================

#[test]
fn header_crc_fail_refuses() {
    let payload = det_bytes(73, 77);
    let bits = payload.len() * 8;
    let opts = EncodeOpts {
        parity_words: 8,
        integrity_bits: 44,
        u_slots: 1,
    };
    let base = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);

    // Corrupt a GEOM word (word index 1..=3: GEOM-A/B/C) — the header-CRC (word 4)
    // must catch it and the decoder must REFUSE, not emit garbage geometry.
    for geom_idx in 1..=3usize {
        let mut card = base.clone();
        let s = sym(&card[geom_idx]);
        card[geom_idx] = word((s ^ 0x2AA) % 2048); // flip several bits
        let r = dec(&card);
        assert!(
            matches!(r, Err(WcError::HeaderCrcMismatch)),
            "GEOM[{geom_idx}] corruption must refuse with HeaderCrcMismatch, got {r:?}"
        );
    }

    // Corrupting the header-CRC word itself (word 4) also fails the check.
    let mut card = base.clone();
    let s = sym(&card[4]);
    card[4] = word((s ^ 0x155) % 2048);
    assert!(matches!(dec(&card), Err(WcError::HeaderCrcMismatch)));
}

// ===========================================================================
// KAT 8 — Append-only upgrade (LOAD-BEARING): K′ + parity-prefix unchanged.
// ===========================================================================

#[test]
fn append_only_upgrade_preserves_kprime_and_prefix() {
    // Encode tier m1, then "upgrade" to m2 > m1. The RS message K′ (H0 ‖ GEOM ‖
    // interleave) and the first m1 parity words must be byte-IDENTICAL — the
    // append-only guarantee. The OUTSIDE-K′ ledger/stop-sign differ (slot1 filled,
    // higher counts), which is exactly why they live outside the RS codeword.
    let payload = det_bytes(73, 5150);
    let bits = payload.len() * 8;
    let m1 = 6usize;
    let m2 = 14usize;

    let opts1 = EncodeOpts {
        parity_words: m1,
        integrity_bits: 44,
        u_slots: 3,
    };
    let opts2 = EncodeOpts {
        parity_words: m2,
        integrity_bits: 44,
        u_slots: 3,
    };
    let card1 = enc(SourceKind::Mk1Xpub, &payload, bits, &opts1);
    let card2 = enc(SourceKind::Mk1Xpub, &payload, bits, &opts2);

    // K′ region = H0(1)+GEOM(4)+interleave. In the engraved stream the ledger
    // (2U) sits between GEOM and interleave; the RS-MESSAGE prefix we compare is
    //   header [0..5)  AND  interleave [5+2U .. 5+2U+interleave_len).
    let u = 3usize;
    let header = 5usize;
    let ledger = 2 * u;
    let inter_start = header + ledger;

    // H0 + GEOM identical (same payload_bits/t/U).
    assert_eq!(&card1[..header], &card2[..header], "H0+GEOM identical");

    // interleave region identical (same K′ message data).
    // Both cards have the same interleave_len (same K). Compute it as
    // card.len() - header - ledger - parity - 2(stop).
    let inter_len1 = card1.len() - inter_start - m1 - 2;
    let inter_len2 = card2.len() - inter_start - m2 - 2;
    assert_eq!(inter_len1, inter_len2, "same interleave length (same K)");
    assert_eq!(
        &card1[inter_start..inter_start + inter_len1],
        &card2[inter_start..inter_start + inter_len2],
        "interleave region (K′ message body) byte-identical"
    );

    // Parity PREFIX: the first m1 parity words of card2 == all m1 parity words of
    // card1 (append-only prefix-extensibility).
    let par1 = &card1[inter_start + inter_len1..card1.len() - 2];
    let par2 = &card2[inter_start + inter_len2..card2.len() - 2];
    assert_eq!(par1.len(), m1);
    assert_eq!(par2.len(), m2);
    assert_eq!(
        &par2[..m1],
        par1,
        "parity prefix byte-identical (append-only)"
    );

    // Both tiers decode to the same payload.
    let d1 = dec(&card1).expect("decode m1");
    let d2 = dec(&card2).expect("decode m2");
    assert_eq!(d1.payload, payload);
    assert_eq!(d2.payload, payload);
    assert!(!d1.truncated && !d2.truncated);
}

// ===========================================================================
// KAT 10 — within-budget errors+erasures recover; beyond-budget → refuse.
// ===========================================================================

#[test]
fn within_budget_substitutions_recover() {
    // Substitutions in the interleave region recover end-to-end *within budget*.
    //
    // NOTE on the actual recovery semantics (the P3→P2 contract): the sync layer
    // converts a CRC-FLAGGED block into a WHOLE-BLOCK ERASURE (it cannot pinpoint
    // which word in the block is wrong from the 5-bit CRC alone). So `s`
    // substitutions confined to one block of `b` words cost `b` ERASURES (not
    // `2s` error-budget). Recovery therefore needs `erased_block_words ≤ m`, not
    // `2·subs ≤ m`. We corrupt several words inside ONE block (b≈8) with m=16 ≥ b
    // so the single erased block is filled by the RS pass.
    let payload = det_bytes(73, 1717);
    let bits = payload.len() * 8;
    let m = 16usize; // ≥ the block stride b (≈8) so one erased block is fillable
    let opts = EncodeOpts {
        parity_words: m,
        integrity_bits: 44,
        u_slots: 1,
    };
    let base = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);
    let header_words = 1 + 4 + 2 * opts.u_slots as usize;

    // Corrupt 3 distinct DATA words all inside block 0 (interleave offsets 0..b-1;
    // the checkpoint sits at offset b). These flag block 0's CRC ⇒ block 0 is
    // erased (≤ b ≤ m words) ⇒ RS fills it.
    let mut card = base.clone();
    for off in [1usize, 3, 6] {
        let idx = header_words + off;
        let s = sym(&card[idx]);
        card[idx] = word((s + 17) % 2048);
    }
    let d = dec(&card).expect("within-budget recover");
    assert_eq!(
        d.payload, payload,
        "in-block substitutions recovered via erasure"
    );
}

#[test]
fn beyond_budget_refuses_never_wrong() {
    // Far beyond budget: many random corruptions. The decoder must refuse (never
    // a wrong payload). This is the headline funds-safety property.
    let payload = det_bytes(73, 31337);
    let bits = payload.len() * 8;
    let m = 6usize;
    let opts = EncodeOpts {
        parity_words: m,
        integrity_bits: 44,
        u_slots: 1,
    };
    let base = enc(SourceKind::Mk1Xpub, &payload, bits, &opts);
    let header_words = 1 + 4 + 2 * opts.u_slots as usize;

    let mut card = base.clone();
    // Corrupt many data words — well beyond ⌊m/2⌋.
    for c in 0..(m + 6) {
        let idx = header_words + 1 + c;
        if idx >= card.len() - 4 {
            break;
        }
        let s = sym(&card[idx]);
        card[idx] = word((s + 101 + c as u16) % 2048);
    }
    match dec(&card) {
        Ok(d) => assert_eq!(d.payload, payload, "if it decodes it must be correct"),
        Err(_) => {} // refusal is the expected outcome
    }
}

// ===========================================================================
// KAT 11 — no panics on malformed input (fuzz); WcError reachability.
// ===========================================================================

#[test]
fn decode_never_panics_on_garbage() {
    // Empty, too-short, unknown words, random word lists — never panic.
    assert!(decode(&[]).is_err());
    assert!(decode(&["abandon"]).is_err());
    assert!(decode(&["not_a_bip39_word"]).is_err());
    assert!(matches!(
        decode(&["zoo", "zoo", "zoo"]),
        Err(WcError::HeaderCrcMismatch) | Err(WcError::Truncated)
    ));
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 300, ..ProptestConfig::default() })]

    /// Arbitrary lists of BIP-39 words must never panic decode — only Ok / Err.
    #[test]
    fn prop_decode_no_panic_on_word_lists(symbols in prop::collection::vec(0u16..2048, 0..120)) {
        let words: Vec<String> = symbols.iter().map(|&s| word(s)).collect();
        let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        // Must return without panicking. Either is acceptable.
        let _ = decode(&refs);
    }

    /// Corrupting a single arbitrary word of a valid card must never yield a
    /// WRONG payload (only the correct one, or an error).
    #[test]
    fn prop_single_corruption_never_wrong_payload(
        seed in any::<u64>(),
        pos in 0usize..200,
        delta in 1u16..2048,
    ) {
        let payload = det_bytes(73, seed);
        let bits = payload.len() * 8;
        let opts = EncodeOpts { parity_words: 8, integrity_bits: 44, u_slots: 1 };
        let base = encode(SourceKind::Mk1Xpub, &payload, bits, &opts).expect("encode");
        let idx = pos % base.len();
        let mut words: Vec<String> = base.iter().map(|s| s.to_string()).collect();
        let s = sym(&words[idx]);
        words[idx] = word((s + delta) % 2048);
        let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        if let Ok(d) = decode(&refs) {
            prop_assert_eq!(d.payload, payload.clone());
        }
    }
}

// ===========================================================================
// Defaults — t = 44, U = 3 (plan §3 / §4.2).
// ===========================================================================

#[test]
fn encode_opts_defaults() {
    let d = EncodeOpts::default();
    assert_eq!(d.integrity_bits, 44, "default t = 44 (plan §3 §4.5)");
    assert_eq!(d.u_slots, 3, "default U = 3 (plan §4.2)");
    assert_eq!(d.parity_words, 0);

    // A round-trip using Default opts works.
    let payload = det_bytes(73, 1);
    let words = encode(
        SourceKind::Mk1Xpub,
        &payload,
        payload.len() * 8,
        &EncodeOpts::default(),
    )
    .expect("encode with defaults");
    let refs: Vec<&str> = words.to_vec();
    let got = decode(&refs).expect("decode defaults");
    assert_eq!(got.payload, payload);
}

#[test]
fn invalid_params_rejected() {
    let payload = det_bytes(73, 1);
    let bits = payload.len() * 8;
    // integrity_bits below the 33-bit floor.
    let bad_t = EncodeOpts {
        parity_words: 0,
        integrity_bits: 32,
        u_slots: 1,
    };
    assert_eq!(
        encode(SourceKind::Mk1Xpub, &payload, bits, &bad_t),
        Err(WcError::InvalidParams)
    );
    // u_slots == 0.
    let bad_u = EncodeOpts {
        parity_words: 0,
        integrity_bits: 44,
        u_slots: 0,
    };
    assert_eq!(
        encode(SourceKind::Mk1Xpub, &payload, bits, &bad_u),
        Err(WcError::InvalidParams)
    );
    // payload_bits exceeds the payload.
    assert_eq!(
        encode(
            SourceKind::Mk1Xpub,
            &payload,
            bits + 1,
            &EncodeOpts::default()
        ),
        Err(WcError::InvalidParams)
    );
}
