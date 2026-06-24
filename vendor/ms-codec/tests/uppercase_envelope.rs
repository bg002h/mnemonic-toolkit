//! BIP-173/93 all-uppercase ms1 acceptance — envelope/wire-layer canonicalization.
//!
//! codex32 accepts consistent-uppercase strings (the BIP-173 QR-alphanumeric
//! form; the checksum engine case-folds; only MIXED case is rejected) — but
//! ms-codec's wire layer used to read RAW string bytes, so a valid
//! all-uppercase MS1 card failed `WrongHrp { got: "MS" }` (and, worse, the
//! combine-side index-`s` guard compared `b's'` against `b'S'` — the U3-guard
//! cell below). Cycle: ms-codec 0.4.2, `design/PLAN_ms1_envelope_uppercase.md`,
//! resolving `design/FOLLOWUPS.md::ms1-envelope-uppercase-bip173`.
//!
//! Per-cell TDD colors (verified at master @ 952bebd, pre-implementation):
//! - U1 RED, U2 RED, U3-mixed RED, U3-guard RED (security), U5 RED,
//!   U6-clean RED;
//! - U3-uniform GREEN (characterization), U4 GREEN (re-pin), U6-corrupted
//!   GREEN (characterization of the pristine-fails/corrupted-repairs
//!   asymmetry's repaired leg).

use ms_codec::codex32::{Codex32String, Fe};
use ms_codec::error::Error;
use ms_codec::{
    combine_shares, decode, decode_with_correction, encode, encode_shares, inspect, Payload, Tag,
    Threshold,
};

/// ms1 HRP (mirrors `ms_codec::consts::HRP`).
const HRP: &str = "ms";

fn entr_single() -> String {
    encode(Tag::ENTR, &Payload::Entr(vec![0xAAu8; 16])).unwrap()
}

// ─── U1: decode accepts the all-uppercase form ───────────────────────────

/// U1 (RED today: `WrongHrp { got: "MS" }`): `decode(&upper)` returns the
/// SAME `(Tag, Payload)` as the lowercase twin.
#[test]
fn u1_decode_uppercase_equals_lowercase_twin() {
    let lower = entr_single();
    let upper = lower.to_uppercase();
    let (lt, lp) = decode(&lower).expect("lowercase twin must decode");
    let (ut, up) = decode(&upper).expect("all-uppercase ms1 must decode (BIP-173 QR form)");
    assert_eq!(ut, lt, "tag must match the lowercase twin");
    assert_eq!(up, lp, "payload must match the lowercase twin");
}

// ─── U2: inspect report parity ────────────────────────────────────────────

/// U2 (RED today): `inspect(&upper)` report equals the lowercase report
/// (tag, threshold, share index, payload fields).
#[test]
fn u2_inspect_uppercase_report_equals_lowercase_report() {
    let lower = encode(
        Tag::ENTR,
        &Payload::Mnem {
            language: 1,
            entropy: vec![0xBBu8; 16],
        },
    )
    .unwrap();
    let upper = lower.to_uppercase();
    let rl = inspect(&lower).expect("lowercase report");
    let ru = inspect(&upper).expect("uppercase ms1 must inspect");
    assert_eq!(ru.hrp, rl.hrp, "hrp");
    assert_eq!(ru.threshold, rl.threshold, "threshold");
    assert_eq!(ru.tag, rl.tag, "tag");
    assert_eq!(ru.share_index, rl.share_index, "share_index");
    assert_eq!(ru.prefix_byte, rl.prefix_byte, "prefix_byte");
    assert_eq!(ru.payload_bytes, rl.payload_bytes, "payload_bytes");
    assert_eq!(ru.checksum_valid, rl.checksum_valid, "checksum_valid");
    assert_eq!(ru.kind, rl.kind, "kind");
    assert_eq!(ru.language, rl.language, "language");
}

// ─── U3: combine — uniform-uppercase, mixed-set, and the index-s guard ────

/// U3-uniform (characterization — likely ALREADY GREEN today: digit
/// threshold, Fe folds, a uniform set passes codex32's raw compares):
/// an ALL-uppercase share set recovers the same secret as lowercase.
#[test]
fn u3_uniform_uppercase_share_set_combines() {
    let p = Payload::Entr(vec![0xCDu8; 16]);
    let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 3, &p).unwrap();
    let (lt, lp) = combine_shares(&shares[..2]).expect("lowercase set combines");
    let upper: Vec<String> = shares[..2].iter().map(|s| s.to_uppercase()).collect();
    let (ut, up) = combine_shares(&upper).expect("uniform-uppercase set must combine");
    assert_eq!(ut, lt);
    assert_eq!(up, lp, "recovered payload must match the lowercase combine");
}

/// U3-mixed (RED today: codex32 `MismatchedHrp("MS", "ms")` out of
/// `interpolate_at`'s raw cross-share compares): ONE uppercase share among
/// lowercase combines under the combine-side canonicalization (C1(a)).
#[test]
fn u3_mixed_set_one_uppercase_share_combines() {
    let p = Payload::Entr(vec![0xCDu8; 16]);
    let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 3, &p).unwrap();
    let mixed = vec![shares[0].to_uppercase(), shares[1].clone()];
    let (tag, recovered) =
        combine_shares(&mixed).expect("one-uppercase-among-lowercase set must combine");
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(recovered, p, "mixed-case set must recover the exact secret");
}

/// U3-guard (SECURITY; RED today = `Ok` carrying the exact secret payload):
/// a uniform-uppercase SAME-ID pair containing the secret-at-S bypassed the
/// `shares.rs` index-`s` guard (`b'S' != b's'`) and codex32's
/// `interpolate_at` short-circuit handed the secret straight back. The
/// realistic whole-card-set-uppercased configuration. Post-fix the
/// canonicalized fields make the guard see `b's'` →
/// `Err(SecretShareSuppliedToCombine)`.
///
/// Fixture per plan (R0-r1 I1 + R0-r2 I1-r2): hand-built
/// `from_seed(HRP, 2, "tst7", Fe::S, wire_bytes)` +
/// `from_seed(HRP, 2, "tst7", Fe::A, filler_of_same_len)`, `.to_uppercase()`
/// BOTH — a lowercase companion dies in `MismatchedHrp` during
/// `interpolate_at`'s validation loop and does NOT reproduce the leak.
#[test]
fn u3_guard_uppercase_secret_at_s_is_rejected_not_leaked() {
    // wire payload = [0x00 reserved-prefix] || 16-byte entropy.
    let mut wire_bytes = vec![0x00u8];
    wire_bytes.extend_from_slice(&[0xABu8; 16]);
    let secret_s = Codex32String::from_seed(HRP, 2, "tst7", Fe::S, &wire_bytes)
        .unwrap()
        .to_string()
        .to_uppercase();
    let filler = vec![0u8; wire_bytes.len()];
    let companion = Codex32String::from_seed(HRP, 2, "tst7", Fe::A, &filler)
        .unwrap()
        .to_string()
        .to_uppercase();
    let res = combine_shares(&[secret_s, companion]);
    assert!(
        matches!(res, Err(Error::SecretShareSuppliedToCombine)),
        "uppercase secret-at-S must be rejected by the index-s guard, \
         not leaked through interpolate_at's short-circuit; got {res:?}"
    );
}

// ─── U4: mixed-case WITHIN one string still rejects (re-pin) ──────────────

/// U4 (re-pin; may be green-first): mixed case WITHIN one string still
/// rejects via codex32 `InvalidCase` — on both the decode and combine legs
/// (combine must NEVER lowercase before the first parse).
#[test]
fn u4_mixed_case_within_one_string_still_rejects() {
    // decode leg: uppercase only the first char of a valid lowercase single.
    let lower = entr_single();
    let mut mixed = lower.clone();
    mixed.replace_range(0..1, "M");
    assert!(
        matches!(
            decode(&mixed),
            Err(Error::Codex32(ms_codec::codex32::Error::InvalidCase(..)))
        ),
        "within-one-string mixed case must stay InvalidCase on decode"
    );

    // combine leg: one within-string-mixed-case share in an otherwise valid set.
    let p = Payload::Entr(vec![0xCDu8; 16]);
    let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 2, &p).unwrap();
    let mut bad = shares[0].clone();
    bad.replace_range(0..1, "M");
    let res = combine_shares(&[bad, shares[1].clone()]);
    assert!(
        matches!(
            res,
            Err(Error::Codex32(ms_codec::codex32::Error::InvalidCase(..)))
        ),
        "within-one-string mixed case must stay InvalidCase on combine; got {res:?}"
    );
}

// ─── U5: true wrong-HRP uppercase reports the canonicalized HRP ───────────

/// U5 (RED today: reports `got: "XS"`): a true wrong-HRP uppercase string
/// (`XS1…`) reports the CANONICALIZED `WrongHrp { got: "xs" }`. Fixture:
/// `from_seed("xs", …)` then `.to_uppercase()` — `from_seed("XS", …)` cannot
/// mint it (its internal set_check_case rejects the mixed intermediate;
/// R0-r2 M4-r2).
#[test]
fn u5_true_wrong_hrp_uppercase_reports_lowercased_hrp() {
    let mut wire_bytes = vec![0x00u8];
    wire_bytes.extend_from_slice(&[0xAAu8; 16]);
    let xs_upper = Codex32String::from_seed("xs", 0, "entr", Fe::S, &wire_bytes)
        .unwrap()
        .to_string()
        .to_uppercase();
    match decode(&xs_upper) {
        Err(Error::WrongHrp { got }) => {
            assert_eq!(got, "xs", "WrongHrp must report the canonicalized HRP");
        }
        other => panic!("expected WrongHrp {{ got: \"xs\" }}, got {other:?}"),
    }
}

// ─── U6: decode_with_correction — clean pass-through + corrupted repair ───

/// U6-clean (RED today — `decode.rs`'s residue==0 path hands the ORIGINAL
/// string to `decode`, so a PRISTINE uppercase card failed while a corrupted
/// one repaired fine): `decode_with_correction(&upper_clean)` equals the
/// lowercase twin's result with EMPTY corrections.
#[test]
fn u6_clean_uppercase_decodes_with_empty_corrections() {
    let lower = entr_single();
    let upper = lower.to_uppercase();
    let (lt, lp, ld) = decode_with_correction(&lower).expect("lowercase twin");
    assert!(ld.is_empty(), "lowercase clean codeword has no corrections");
    let (ut, up, ud) = decode_with_correction(&upper).expect("pristine uppercase card must decode");
    assert_eq!(ut, lt);
    assert_eq!(up, lp);
    assert!(
        ud.is_empty(),
        "pristine uppercase card must report no corrections"
    );
}

/// U6-corrupted (characterization — GREEN today: corrections re-emit
/// lowercase): an uppercase card with 1 symbol error repairs. Together with
/// U6-clean this pins the resolution of the pristine-fails/corrupted-repairs
/// asymmetry.
#[test]
fn u6_corrupted_uppercase_repairs() {
    let lower = entr_single();
    let (lt, lp) = decode(&lower).unwrap();
    let mut upper = lower.to_uppercase().into_bytes();
    // Corrupt one payload-region char (index 10, past "MS1" + header),
    // staying uniform-uppercase and within the codex32 alphabet.
    upper[10] = if upper[10] == b'Q' { b'P' } else { b'Q' };
    let upper = String::from_utf8(upper).unwrap();
    let (ut, up, details) =
        decode_with_correction(&upper).expect("1-error uppercase card must repair");
    assert_eq!(ut, lt);
    assert_eq!(
        up, lp,
        "repaired payload must match the pristine lowercase twin"
    );
    assert_eq!(details.len(), 1, "exactly one correction");
    assert_eq!(
        details[0].position,
        10 - 3,
        "data-part position (post-HRP+separator)"
    );
}
