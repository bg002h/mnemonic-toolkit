//! Cycle 6 — md-codec rejects MIXED-case md1 per BIP-173 (md-codec was the one
//! constellation codec that leniently accepted it; mk-codec + ms-codec reject).
//! All-upper (the QR form) and all-lower stay valid; only MIXED rejects. The
//! reject is enforced at both decode boundaries: `unwrap_string`
//! (decode_md1_string + reassemble) and `parse_chunk_symbols`
//! (decode_with_correction). Resolves `md-codec-accepts-mixed-case-bip173-leniency`.
#![cfg(feature = "derive")]

mod common;

use common::{descriptor_from_tree, descriptor_with_pubkeys, keyarg, multikeys, wrap};
use md_codec::Tag;
use md_codec::chunk::{decode_with_correction, reassemble, split};
use md_codec::decode::decode_md1_string;
use md_codec::encode::encode_md1_string;

/// Single-chunk descriptor (`wsh(pk)`) for the single-string decode cells.
/// Template-mode (empty TLV, no 65-byte xpub) so its payload stays under the
/// codex32 regular code's 80-data-symbol single-string cap (cycle-4 H6); a
/// pubkeys-populated `wsh(pk)` overflows that cap and must be chunked.
fn one_chunk() -> md_codec::Descriptor {
    descriptor_from_tree(wrap(Tag::Wsh, keyarg(Tag::PkK, 0)), false)
}

/// A descriptor large enough to `split()` into ≥2 chunks (wide multi).
fn many_chunk() -> md_codec::Descriptor {
    descriptor_with_pubkeys(wrap(Tag::Wsh, multikeys(Tag::Multi, 2, (0..20).collect())))
}

const NEEDLE: &str = "mixes upper and lower case";

#[test]
fn decode_md1_string_rejects_mixed_case_data_char() {
    let s = encode_md1_string(&one_chunk()).unwrap();
    // Uppercase the FIRST lowercase data char after the "md1" HRP → mixed.
    let mut chars: Vec<char> = s.chars().collect();
    for c in chars.iter_mut().skip(3) {
        if c.is_ascii_lowercase() {
            *c = c.to_ascii_uppercase();
            break;
        }
    }
    let mixed: String = chars.into_iter().collect();
    let err = decode_md1_string(&mixed).unwrap_err();
    assert!(err.to_string().contains(NEEDLE), "got: {err}");
}

#[test]
fn decode_md1_string_rejects_mixed_hrp() {
    let s = encode_md1_string(&one_chunk()).unwrap(); // "md1" + lower data
    let mixed = format!("Md1{}", &s[3..]); // uppercase one HRP char → mixed
    let err = decode_md1_string(&mixed).unwrap_err();
    assert!(err.to_string().contains(NEEDLE), "got: {err}");
}

#[test]
fn decode_md1_string_accepts_uppercase_round_trip() {
    let d = one_chunk();
    let upper = encode_md1_string(&d).unwrap().to_uppercase();
    assert_eq!(
        decode_md1_string(&upper).unwrap(),
        d,
        "all-upper (QR form) must round-trip"
    );
}

#[test]
fn reassemble_accepts_cross_chunk_case_heterogeneity() {
    // BIP-173 is PER STRING: one chunk wholly UPPER among lowercase siblings is
    // LEGAL (each chunk independently case-uniform) — the QR workflow needs it.
    let d = many_chunk();
    let chunks = split(&d).unwrap();
    assert!(
        chunks.len() >= 2,
        "fixture must span ≥2 chunks, got {}",
        chunks.len()
    );
    let mut set = chunks.clone();
    set[0] = set[0].to_uppercase();
    let refs: Vec<&str> = set.iter().map(String::as_str).collect();
    assert_eq!(reassemble(&refs).unwrap(), d);
}

#[test]
fn reassemble_rejects_internally_mixed_chunk() {
    let d = many_chunk();
    let chunks = split(&d).unwrap();
    let mut set = chunks.clone();
    set[0] = format!("MD1{}", &chunks[0][3..]); // upper HRP + lower data → mixed
    let refs: Vec<&str> = set.iter().map(String::as_str).collect();
    let err = reassemble(&refs).unwrap_err();
    assert!(err.to_string().contains(NEEDLE), "got: {err}");
}

#[test]
fn decode_with_correction_rejects_mixed_chunk() {
    let d = many_chunk();
    let chunks = split(&d).unwrap();
    let mut set = chunks.clone();
    set[0] = format!("MD1{}", &chunks[0][3..]); // internally mixed
    let refs: Vec<&str> = set.iter().map(String::as_str).collect();
    let err = decode_with_correction(&refs).unwrap_err();
    assert!(err.to_string().contains(NEEDLE), "got: {err}");
}

#[test]
fn decode_with_correction_accepts_all_upper() {
    // All-upper chunks (each uniform) through the correction pass-through stay Ok.
    let d = many_chunk();
    let chunks = split(&d).unwrap();
    let set: Vec<String> = chunks.iter().map(|c| c.to_uppercase()).collect();
    let refs: Vec<&str> = set.iter().map(String::as_str).collect();
    let (back, corrections) = decode_with_correction(&refs).unwrap();
    assert_eq!(back, d);
    assert!(
        corrections.is_empty(),
        "no corrections expected for a valid card"
    );
}

/// The KEY pin for the SECOND injection site (`parse_chunk_symbols`): a chunk
/// that is BOTH internally mixed-case AND carries a correctable symbol error.
/// The pass-through (residue==0) cell above is actually caught by the FIRST site
/// (`unwrap_string`, via the forwarded original string), so it does NOT prove
/// `parse_chunk_symbols`'s check. This cell does: with a symbol error the
/// correction BRANCH runs, and only the `parse_chunk_symbols` check rejects it —
/// a single-site (unwrap_string-only) impl would silently CORRECT then accept,
/// reintroducing the R0-I3 inconsistency (0-error mixed rejects, 1-error mixed
/// accepted). RED against a single-site impl; GREEN here.
#[test]
fn decode_with_correction_rejects_mixed_with_symbol_error() {
    const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let d = many_chunk();
    let chunks = split(&d).unwrap();
    // Corrupt one data char of chunk 0 to a DIFFERENT codex32 char (a correctable
    // 1-symbol error → residue != 0 → the correction branch), then uppercase the
    // HRP so the chunk is internally mixed.
    let mut cs: Vec<char> = chunks[0].chars().collect();
    for c in cs.iter_mut().skip(3) {
        if ALPHABET.contains(*c) {
            *c = ALPHABET.chars().find(|&a| a != *c).unwrap();
            break;
        }
    }
    let corrupted: String = cs.into_iter().collect();
    let mut set = chunks.clone();
    set[0] = format!("MD1{}", &corrupted[3..]); // upper HRP + lower (corrupted) data → mixed
    let refs: Vec<&str> = set.iter().map(String::as_str).collect();
    let err = decode_with_correction(&refs).unwrap_err();
    assert!(err.to_string().contains(NEEDLE), "got: {err}");
}
