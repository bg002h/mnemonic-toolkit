//! T2-a (#6) — never-wrong-payload property harness for the repair engine.
//!
//! SPEC: `design/SPEC_test_hardening_T2_never_wrong_payload.md` §T2-a. Origin:
//! constellation-eval §2 item #6 — the confirmed miscorrection class (F4: a BCH
//! "repair" once blessed a >4-error wrong-fit as a *different* wallet).
//!
//! **Harness location (Option A′, post-R0 fold).** The toolkit is a BINARY
//! crate: `repair`/`indel`/`error` are `pub` ONLY under `#[cfg(fuzzing)]`
//! (`lib.rs:144-190`) and plain-private in `main.rs`, so `repair_card` /
//! `recover_indel` / `SetVerify` / `RepairError` are UNREACHABLE from a `tests/`
//! integration crate. This module is therefore an IN-CRATE `#[cfg(test)]`
//! module (declared in `main.rs`, the repo's established pattern — cf.
//! `indel.rs:302`, `repair.rs:1910`), giving direct private access to the
//! engine INCLUDING the mock-oracle ambiguity path (a real-vector Ambiguous
//! indel is ≈2⁻⁶⁵, `cmd/repair.rs:455-459`). It compiles only under
//! `cargo test` → TEST-only, NO-BUMP (nothing added to the shipped binary,
//! the lib surface, or the `#[cfg(fuzzing)]` block).
//!
//! **Oracle independence.** Every property is checked against an oracle
//! independent of `repair.rs`'s internals: the ORIGINAL card bytes (for the
//! ≤t / indel / demotion legs) or `mk_codec::decode` (for the F4 mk1
//! set-reassembly leg). Each cell is GREEN unmutated and RED under a NAMED
//! source mutation (see the per-cell doc comments).

use crate::indel::{recover_indel, IndelOracle, IndelOutcome};
use crate::repair::{repair_card, CardKind, Ms1IndelOracle, RepairError, SetVerify};
use mk_codec::string_layer::bch::ALPHABET;
use ms_codec::{Payload, Tag};
use proptest::prelude::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Fixtures — real m-format cards reused byte-identically from the existing
// repair test suite (`repair.rs` unit tests / `cli_*` integration tests).
// ---------------------------------------------------------------------------

/// Single-chunk ms1 (TREZOR_12_ZERO entropy). Also used as a stable base for
/// the >4 smoke.
const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// The canonical "abandon × 11 about" bip84 mk1 card: chunk 0 (long code,
/// 108-char data-part, xpub-bearing) + chunk 1 (regular code, 77-char
/// data-part). Together a VALID 2-chunk card (verified via `mk_codec::decode`).
const VALID_MK1_LONG: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
const VALID_MK1_REG: &str =
    "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

/// Real 3-chunk bip84 md1 card (reassembles via `md_codec::chunk::reassemble`).
const MD1_C0: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const MD1_C1: &str = "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const MD1_C2: &str = "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

/// Canonical v0.30 single-string (non-chunked) md1 — a COMPLETE card in one
/// string, with NO cross-chunk hash. Used for the (c) documented-residual KAT.
const VALID_SINGLE_MD1: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

// ---------------------------------------------------------------------------
// Corruption helpers — all edits are DATA-part only (chars after the last
// separator `1`), stay in the lowercase bech32 charset, and (for
// substitutions) guarantee new != old.
// ---------------------------------------------------------------------------

fn alphabet_str() -> &'static str {
    std::str::from_utf8(ALPHABET).unwrap()
}

/// Number of data-part characters (after the last `1` separator).
fn data_part_len(chunk: &str) -> usize {
    let sep = chunk.rfind('1').unwrap();
    chunk[sep + 1..].chars().count()
}

/// Data-part characters decoded to their 5-bit bech32 values.
fn data_part_5bit(chunk: &str) -> Vec<u8> {
    let sep = chunk.rfind('1').unwrap();
    let alpha = alphabet_str();
    chunk[sep + 1..]
        .chars()
        .map(|c| alpha.find(c).unwrap() as u8)
        .collect()
}

/// Apply substitutions to the DATA-part. `edits` = `(data_part_index, shift)`
/// with `shift ∈ 1..=31`; a non-zero shift (mod 32) guarantees new != old, and
/// `ALPHABET` is entirely lowercase, so no mixed-case / HRP corruption is
/// introduced.
fn substitute(chunk: &str, edits: &[(usize, u8)]) -> String {
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let alpha = alphabet_str();
    for &(pos, shift) in edits {
        let old_idx = alpha.find(chars[pos]).unwrap();
        let new_idx = (old_idx + shift as usize) % 32;
        chars[pos] = alpha.as_bytes()[new_idx] as char;
    }
    let mut out = String::from(prefix);
    out.extend(chars);
    out
}

/// Flip one data-part char to the next bech32 symbol (a single substitution).
fn flip_at(chunk: &str, pos: usize) -> String {
    substitute(chunk, &[(pos, 1)])
}

/// Inject exactly `min(k, data_len)` substitutions at DISTINCT data-part
/// positions, each new != old.
fn inject_k_subs(chunk: &str, k: usize, rng: &mut StdRng) -> String {
    let dlen = data_part_len(chunk);
    let k = k.min(dlen);
    let mut positions: Vec<usize> = (0..dlen).collect();
    positions.shuffle(rng);
    positions.truncate(k);
    let edits: Vec<(usize, u8)> = positions
        .iter()
        .map(|&p| (p, rng.gen_range(1..=31u8)))
        .collect();
    substitute(chunk, &edits)
}

/// Insert one bech32 char into the data-part (input becomes too long by one).
fn insert_data_char(chunk: &str, pos: usize, ch: char) -> String {
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let pos = pos.min(chars.len());
    chars.insert(pos, ch);
    let mut out = String::from(prefix);
    out.extend(chars);
    out
}

/// Delete one data-part char (input becomes too short by one).
fn delete_data_char(chunk: &str, pos: usize) -> String {
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    if chars.is_empty() {
        return chunk.to_string();
    }
    let pos = pos.min(chars.len() - 1);
    chars.remove(pos);
    let mut out = String::from(prefix);
    out.extend(chars);
    out
}

/// Build a doctored 2-chunk mk1 card that is checksum-VALID per chunk yet fails
/// cross-chunk reassembly: copy `VALID_MK1_REG`, alter ONE payload symbol
/// (past the 8-symbol chunked header), and recompute the chunk's BCH checksum
/// via the public `encode_5bit_to_string` (which calls
/// `bch_create_checksum_regular`). Returns the doctored chunk 1; the caller
/// pairs it with `VALID_MK1_LONG` (chunk 0). Searches for the first payload
/// index whose alteration actually breaks `mk_codec::decode([chunk0, doctored])`
/// (the independent oracle), so the fixture is self-validating.
fn doctored_mk1_reg_chunk_breaking_set() -> String {
    let values = data_part_5bit(VALID_MK1_REG); // 77 symbols (64 data + 13 checksum)
    let data_len = values.len() - 13; // regular checksum is 13 symbols
    for alter_idx in 8..data_len {
        // skip the 8-symbol chunked header
        let mut data = values[..data_len].to_vec();
        data[alter_idx] = (data[alter_idx] + 1) % 32;
        let doctored =
            mk_codec::string_layer::encode_5bit_to_string(&data).expect("re-encode doctored chunk");
        if mk_codec::decode(&[VALID_MK1_LONG, &doctored]).is_err() {
            return doctored;
        }
    }
    panic!("no single payload-symbol alteration of VALID_MK1_REG broke the 2-chunk set");
}

// ===========================================================================
// ≤t substitution leg (proptest) — inject k≤4 substitutions (per-chunk for
// multi-chunk cards), assert `repair` emits EXACTLY the original. Oracle =
// the original card bytes.
//
// RED-proof: mutate the correction path to emit a wrong codeword. The three
// cells RED via DIFFERENT paths (verified by execution in post-impl R0):
//   - mk1: at `repair.rs:782` `was_byte ^ m` → `was_byte ^ m ^ 1` corrupts every
//     correction; the chunk's defensive re-verify (`repair.rs:793-799`) then
//     rejects the wrong codeword as `TooManyErrors`, so the mk1 cell REDs at
//     `.expect("repair Ok")` — NOT the equality assert.
//   - md1/ms1: the wrong-codeword emit lives in the local `apply_{md,ms}_corrections`
//     (`c.now` no-op); there is no defensive re-verify on that path, so the wrong
//     codeword is emitted and the cell REDs on the equality oracle (`prop_assert_eq!`).
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// ms1: random 16-byte-entropy card → ≤4 substitutions → repair MUST emit
    /// exactly the original. **Equality ONLY** — a touched ms1 correction is
    /// `Unverified` by Cycle-F design (`repair.rs:1163-1172`), so we do NOT
    /// assert `Blessed` here (that is the (a) cell's job).
    #[test]
    fn prop_ms1_le_t_substitution_recovers_exact_original(
        entropy in prop::collection::vec(any::<u8>(), 16..=16),
        k in 0usize..=4,
        seed in any::<u64>(),
    ) {
        let card = ms_codec::encode(Tag::ENTR, &Payload::Entr(entropy)).expect("encode ms1");
        let mut rng = StdRng::seed_from_u64(seed);
        let corrupted = inject_k_subs(&card, k, &mut rng);
        let outcome = repair_card(CardKind::Ms1, &[corrupted]).expect("repair Ok");
        prop_assert_eq!(&*outcome.corrected_chunks[0], card.as_str());
    }

    /// mk1: the FULL 2-chunk card, ≤4 substitutions PER CHUNK → repair MUST
    /// emit exactly both original chunks, and the set-reassembly re-verify
    /// (Cycle E) blesses the genuine recovery.
    #[test]
    fn prop_mk1_full_set_le_t_substitution_recovers_exact_original(seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let k0 = rng.gen_range(0..=4);
        let k1 = rng.gen_range(0..=4);
        let c0 = inject_k_subs(VALID_MK1_LONG, k0, &mut rng);
        let c1 = inject_k_subs(VALID_MK1_REG, k1, &mut rng);
        let outcome = repair_card(CardKind::Mk1, &[c0, c1]).expect("repair Ok");
        prop_assert_eq!(&*outcome.corrected_chunks[0], VALID_MK1_LONG);
        prop_assert_eq!(&*outcome.corrected_chunks[1], VALID_MK1_REG);
        prop_assert!(
            matches!(outcome.set_verify, SetVerify::Blessed),
            "a genuine ≤4-per-chunk full-set recovery must reassemble → Blessed"
        );
    }

    /// md1: real 3-chunk card, ≤4 substitutions PER CHUNK → repair (atomic via
    /// `md_codec::decode_with_correction`) MUST emit exactly all three original
    /// chunks; a CHUNKED (here, multi-chunk) md1 delegate stays `Blessed` —
    /// its content-id/cross-chunk check runs and passes. A non-chunked
    /// single-string md1 has separate coverage (v0.86.0 demote — see
    /// `f4_c` below) since it has no such oracle.
    #[test]
    fn prop_md1_multichunk_le_t_substitution_recovers_exact_original(seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let orig = [MD1_C0, MD1_C1, MD1_C2];
        let corrupted: Vec<String> = orig
            .iter()
            .map(|c| {
                let k = rng.gen_range(0..=4);
                inject_k_subs(c, k, &mut rng)
            })
            .collect();
        let outcome = repair_card(CardKind::Md1, &corrupted).expect("repair Ok");
        for (i, o) in orig.iter().enumerate() {
            prop_assert_eq!(&*outcome.corrected_chunks[i], *o);
        }
        prop_assert!(matches!(outcome.set_verify, SetVerify::Blessed));
    }

    /// single-indel leg — one random insert OR delete in the data-part of a
    /// random ms1 → `recover_indel` (public engine) with the REAL
    /// `Ms1IndelOracle`. If the outcome is `Unique`, it MUST equal the
    /// original (oracle = original bytes). Ambiguous/Unrecoverable are
    /// acceptable non-`Unique` outcomes and impose no equality obligation.
    ///
    /// RED-proof for the equality property: mutate `Ms1IndelOracle::validate`
    /// (or the shared `apply_ms_corrections`) to return a non-original string
    /// → a `Unique` recovery no longer equals the original. (The dedup/
    /// ambiguity-fold mutation at `indel.rs:116-121` is RED-proven separately
    /// by `indel_ambiguity_fold_pins_multiple_distinct_recovered`, since a
    /// real-vector Ambiguous outcome is cryptographically unreachable here.)
    #[test]
    fn prop_ms1_single_indel_unique_equals_original(
        entropy in prop::collection::vec(any::<u8>(), 16..=16),
        insert in any::<bool>(),
        seed in any::<u64>(),
    ) {
        let card = ms_codec::encode(Tag::ENTR, &Payload::Entr(entropy)).expect("encode ms1");
        let mut rng = StdRng::seed_from_u64(seed);
        let dlen = data_part_len(&card);
        let corrupted = if insert {
            let pos = rng.gen_range(0..=dlen);
            let ch = alphabet_str().as_bytes()[rng.gen_range(0..32)] as char;
            insert_data_char(&card, pos, ch)
        } else {
            let pos = rng.gen_range(0..dlen);
            delete_data_char(&card, pos)
        };
        match recover_indel(&corrupted, "ms", 1, 0, &Ms1IndelOracle) {
            IndelOutcome::Unique(c) => prop_assert_eq!(c.recovered, card),
            IndelOutcome::Ambiguous(_) | IndelOutcome::Unrecoverable => {}
        }
    }

    /// (d) random >4 substitutions — smoke: `repair` never panics and, when it
    /// returns `Ok`, NEVER re-emits the original (a word ≥5 away from the
    /// original codeword can only decode to `Err` or a DIFFERENT codeword under
    /// a t=4 decoder). This is NOT a bless-property gate — it is the
    /// never-return-the-original half of the never-wrong-payload invariant.
    #[test]
    fn prop_random_gt4_never_panics_never_returns_original(
        which in 0usize..2,
        seed in any::<u64>(),
    ) {
        let mut rng = StdRng::seed_from_u64(seed);
        let k = rng.gen_range(5..=8);
        let (kind, card) = if which == 0 {
            (CardKind::Ms1, VALID_MS1)
        } else {
            (CardKind::Mk1, VALID_MK1_REG)
        };
        let corrupted = inject_k_subs(card, k, &mut rng);
        if let Ok(outcome) = repair_card(kind, &[corrupted]) {
            for cc in &outcome.corrected_chunks {
                prop_assert_ne!(&**cc, card);
            }
        }
    }
}

// ===========================================================================
// single-indel dedup/ambiguity fold — mock-oracle RED-proof cell.
// ===========================================================================

/// Pins the `recover_indel` dedup/ambiguity fold (`indel.rs:116-121`). Mirrors
/// `indel.rs:356-374`'s `AcceptAll` recipe: an oracle that accepts EVERY
/// candidate yields ≥2 distinct recovered strings from a typical input, which
/// MUST fold to `Ambiguous`.
///
/// RED-proof: mutate the fold at `indel.rs:116-121` to "return the first hit
/// without dedup/ambiguity" — the outcome then becomes `Unique` and the
/// `matches!(… Ambiguous …)` assertion fails:
///
/// ```text
/// -   dedup_by_recovered(&mut hits);
/// -   match hits.len() {
/// -       0 => IndelOutcome::Unrecoverable,
/// -       1 => IndelOutcome::Unique(hits.into_iter().next().unwrap()),
/// -       _ => IndelOutcome::Ambiguous(hits),
/// -   }
/// +   match hits.into_iter().next() {
/// +       Some(h) => IndelOutcome::Unique(h),
/// +       None => IndelOutcome::Unrecoverable,
/// +   }
/// ```
#[test]
fn indel_ambiguity_fold_pins_multiple_distinct_recovered() {
    struct AcceptAll;
    impl IndelOracle for AcceptAll {
        fn validate(
            &self,
            candidate: &str,
            _allowed: &BTreeSet<usize>,
            _e_subst: usize,
        ) -> Option<(String, usize)> {
            Some((candidate.to_string(), 0))
        }
    }
    let outcome = recover_indel("ms1qpzr", "ms", 1, 0, &AcceptAll);
    assert!(
        matches!(outcome, IndelOutcome::Ambiguous(ref v) if v.len() >= 2),
        "AcceptAll over a 1-indel search must fold to Ambiguous; got {outcome:?}"
    );
}

// ===========================================================================
// F4 leg — DETERMINISTIC constructed / pinned tri-state cells.
// ===========================================================================

/// (a) ms1 demotion pin — a TOUCHED ms1 substitution-correction is `Unverified`
/// (Cycle F): a bounded-distance BCH correction spends the checksum's
/// error-detection budget with no self-oracle, so a >4-error "repair" could
/// alias to a DIFFERENT valid seed undetectably. Also pins the equality of the
/// recovered card.
///
/// RED-proof: remove the Cycle-F demotion at `repair.rs:1163` (a touched ms1
/// then becomes `Blessed` and the `Unverified` arm panics):
///
/// ```text
/// -   let set_verify = if repairs.is_empty() {
/// -       SetVerify::Blessed
/// -   } else {
/// -       SetVerify::Unverified { reason: … }
/// -   };
/// +   let set_verify = SetVerify::Blessed;
/// ```
#[test]
fn f4_a_touched_ms1_correction_is_unverified() {
    let bad = flip_at(VALID_MS1, 10); // single substitution → touched
    let outcome = repair_card(CardKind::Ms1, &[bad]).expect("repair Ok");
    assert!(
        !outcome.repairs.is_empty(),
        "fixture must actually be corrected (touched)"
    );
    assert_eq!(&*outcome.corrected_chunks[0], VALID_MS1);
    match outcome.set_verify {
        SetVerify::Unverified { .. } => {}
        SetVerify::Blessed => {
            panic!("a touched ms1 correction must be Unverified (Cycle-F demotion)")
        }
    }
}

/// (b) mk1 multi-chunk set-reverify — a doctored 2-chunk card whose chunk 1 is
/// checksum-VALID but payload-altered (so it fails cross-chunk reassembly),
/// then perturbed with 3 substitutions so `repair` TOUCHES it and BCH corrects
/// it back to the doctored (wrong) codeword. The Cycle-E set-level re-verify
/// (`verify_mk1_set`) then rejects the complete-but-inconsistent group ⇒
/// `Err(SetReassemblyMismatch)` — the funds fix.
///
/// Independent oracle: `mk_codec::decode([chunk0, doctored])` returns `Err`
/// (asserted inline), proving the doctored set genuinely does not reassemble.
///
/// RED-proof: delete the set-reverify at `repair.rs:1115` — `repair_card` then
/// returns `Ok(Blessed)` re-emitting the doctored (wrong) chunk, and the
/// `Err(SetReassemblyMismatch)` arm panics:
///
/// ```text
/// -   let set_verify = verify_mk1_set(&corrected_chunks, &repairs)?;
/// +   let set_verify = SetVerify::Blessed;
/// ```
#[test]
fn f4_b_mk1_doctored_multichunk_set_reassembly_mismatch_rejects() {
    let doctored = doctored_mk1_reg_chunk_breaking_set();
    // Independent oracle: the doctored (per-chunk-valid) set must NOT reassemble.
    assert!(
        mk_codec::decode(&[VALID_MK1_LONG, doctored.as_str()]).is_err(),
        "independent oracle: doctored 2-chunk set must fail cross-chunk reassembly"
    );
    // Perturb the doctored chunk with 3 substitutions so `repair` corrects them
    // back to the doctored codeword (touching the chunk) — NOT to the original.
    let corrupted = substitute(&doctored, &[(5, 1), (20, 1), (40, 1)]);
    let result = repair_card(CardKind::Mk1, &[VALID_MK1_LONG.to_string(), corrupted]);
    match result {
        Err(RepairError::SetReassemblyMismatch { .. }) => {}
        other => panic!("expected Err(SetReassemblyMismatch) (funds fix); got {other:?}"),
    }
}

/// (c) single-string non-chunked demotion — v0.86.0 FIX
/// (`toolkit-v0860-demote`). A single-string (non-chunked, chunked-flag
/// bit == 0) md1 has NO cross-chunk/content-id hash — the v0.35.0 bypass
/// (`vendor/md-codec/src/chunk.rs:615-631`) routes it straight to
/// `decode_md1_string`, skipping `reassemble`'s content-id check entirely
/// — so a TOUCHED correction is now demoted to `Unverified`, closing the
/// residual a >4-error miscorrection could otherwise alias to a different
/// valid card undetectably. `VALID_SINGLE_MD1`'s data-part starts `'y'`
/// (codex32 value 4 → bit 0 == 0), confirming it is genuinely non-chunked.
/// Formerly pinned the opposite (`Blessed`) as a DOCUMENTED RESIDUAL under
/// FOLLOWUP `repair-single-string-fully-valid-alias-second-oracle`, now
/// resolved by this demote.
///
/// RED-proof: revert the demote at `repair_via_md_codec` (drop the
/// `chunks.len() == 1 && !repairs.is_empty() && is_non_chunked_md1(..)`
/// gate back to unconditional `SetVerify::Blessed`) — the `Unverified`
/// arm then panics.
#[test]
fn f4_c_single_string_non_chunked_md1_correction_is_unverified() {
    let bad = flip_at(VALID_SINGLE_MD1, 3); // single substitution within t≤4
    let outcome = repair_card(CardKind::Md1, &[bad]).expect("repair Ok");
    assert!(
        !outcome.repairs.is_empty(),
        "fixture must actually be corrected (touched)"
    );
    assert_eq!(&*outcome.corrected_chunks[0], VALID_SINGLE_MD1);
    match outcome.set_verify {
        SetVerify::Unverified { .. } => {}
        SetVerify::Blessed => {
            panic!("a touched non-chunked md1 correction must be Unverified (v0.86.0 demote)")
        }
    }
}

/// (e) chunked-of-1 BOUNDARY test — the oracle boundary this cycle's demote
/// hinges on. A chunked-of-1 md1 (chunked-flag bit == 1, count == 1 — the
/// shape `mnemonic bundle` / `--md1-form=template` emits via
/// `md_codec::chunk::split`) DOES retain the content-id oracle (it falls
/// through to `reassemble`, `chunk.rs:632-636`), so a TOUCHED correction
/// MUST stay `Blessed` — demoting on `count == 1` alone (rather than the
/// chunked-flag bit) would have wrongly caught this shape too.
///
/// RED-proof: change the demote predicate from reading the chunked-flag bit
/// to testing `chunks.len() == 1` alone (the wrong, over-broad predicate) —
/// this chunked-of-1 fixture would then wrongly demote to `Unverified`.
#[test]
fn f4_e_chunked_of_1_md1_correction_stays_blessed() {
    let descriptor =
        md_codec::decode_md1_string(VALID_SINGLE_MD1).expect("decode non-chunked fixture");
    let chunked = md_codec::chunk::split(&descriptor).expect("split into chunked-of-1");
    assert_eq!(
        chunked.len(),
        1,
        "fixture descriptor must be small enough to split to exactly 1 chunk"
    );
    let original = &chunked[0];
    let bad = flip_at(original, 3); // single substitution within t≤4
    let outcome = repair_card(CardKind::Md1, &[bad]).expect("repair Ok");
    assert!(
        !outcome.repairs.is_empty(),
        "fixture must actually be corrected (touched)"
    );
    assert_eq!(&*outcome.corrected_chunks[0], original.as_str());
    assert_eq!(
        outcome.set_verify,
        SetVerify::Blessed,
        "a chunked-of-1 md1 retains the content-id oracle — must stay Blessed, not demoted"
    );
}
