//! Theme 2 — BCH adversarial. Drive correction via the public decode_with_correction.
mod common;

use common::corrupt_chunk_at;
use md_codec::bitstream::{BitReader, BitWriter};
use md_codec::chunk::{ChunkHeader, decode_with_correction, reassemble, split};
use md_codec::codex32::{unwrap_string, wrap_payload};
use md_codec::encode::Descriptor;
use md_codec::error::Error;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;

fn wpkh_descriptor(depth: u8) -> Descriptor {
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: (0..depth)
                    .map(|i| PathComponent {
                        hardened: true,
                        value: (i as u32) + 1,
                    })
                    .collect(),
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    }
}

fn multi_chunk_descriptor() -> Descriptor {
    // 6 Divergent cosigners × 15 hardened components → ≥4 chunks.
    let paths = (0..6u32)
        .map(|c| OriginPath {
            components: (0..15u32)
                .map(|i| PathComponent {
                    hardened: true,
                    value: c * 100 + i + 1,
                })
                .collect(),
        })
        .collect();
    Descriptor {
        n: 6,
        path_decl: PathDecl {
            n: 6,
            paths: PathDeclPaths::Divergent(paths),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: (0..6).collect(),
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    }
}

// H6 (cycle-4) — encode-side: an oversize descriptor (one that needs chunking)
// must REJECT the default single-string encode with the typed cap error, while
// the chunked path (`split`) succeeds. `multi_chunk_descriptor` splits into ≥3
// chunks → its single-string payload is far over the 80-data-symbol cap.
#[test]
fn encode_md1_string_rejects_oversize_descriptor() {
    let d = multi_chunk_descriptor();
    assert!(
        matches!(
            md_codec::encode_md1_string(&d),
            Err(Error::PayloadTooLongForSingleString { .. })
        ),
        "an oversize descriptor must fail closed on the single-string encode path"
    );
    // The contractual remedy (chunked) still works.
    let chunks = split(&d).expect("chunked encode of an oversize descriptor succeeds");
    assert!(
        chunks.len() >= 2,
        "oversize descriptor must produce >1 chunk"
    );
}

// ── M4 (cycle-4): decode-side `len > 93` rejection (correcting path) ─────────
const M4_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Forge a CLEAN (residue==0, BCH-valid) md1 string with `data_symbols`
/// arbitrary data symbols, bypassing wrap_payload's H6 cap via the raw BCH
/// primitive — used to build over-93-codeword words.
fn forge_clean_md1(data_symbols: usize) -> String {
    let data: Vec<u8> = (0..data_symbols).map(|i| (i as u8) & 0x1F).collect();
    let checksum = md_codec::bch::bch_create_checksum_regular("md", &data);
    let mut s = String::from("md1");
    for &sym in data.iter().chain(checksum.iter()) {
        s.push(M4_ALPHABET[(sym & 0x1F) as usize] as char);
    }
    s
}

// M4 test #1 — a > 93-symbol chunk carrying ≥1 transcription error (residue ≠ 0)
// must be REJECTED with the typed ChunkSymbolCountOutOfRange. Today the
// uncapped symbols.len() enters the unbounded chien_search loop and the decoder
// mis-corrects at an aliased root (β has order 93 → degrees alias for len > 93).
#[test]
fn decode_with_correction_rejects_over_93_symbol_chunk() {
    // 90 data + 13 checksum = 103 codeword symbols (> 93).
    let clean = forge_clean_md1(90);
    assert_eq!(clean.chars().count(), 3 + 103);
    // Introduce one transcription error in the data region (residue != 0).
    let corrupted = corrupt_chunk_at(&clean, 7, 0x1F);
    let refs = [corrupted.as_str()];
    match decode_with_correction(&refs) {
        Err(Error::ChunkSymbolCountOutOfRange {
            chunk_index,
            symbols,
            max,
        }) => {
            assert_eq!(chunk_index, 0);
            assert_eq!(symbols, 103);
            assert_eq!(max, 93);
        }
        other => panic!(
            "over-93-symbol chunk with an error must reject with ChunkSymbolCountOutOfRange, got {other:?}"
        ),
    }
}

// M4 test #4 — positive control: a legitimate chunked md1 set (each chunk
// ≤ 93 symbols) with a single in-capacity error per chunk still repairs.
#[test]
fn valid_chunked_md1_still_repairs() {
    let d = multi_chunk_descriptor();
    let chunks = split(&d).unwrap();
    // Each split chunk is ≤ 64 data + 13 checksum = 77 ≤ 93 → unaffected.
    for c in &chunks {
        assert!(
            c.chars().count() <= 3 + 93,
            "split chunk must be within the 93-symbol cap"
        );
    }
    let mut cs = chunks.clone();
    cs[0] = corrupt_chunk_at(&cs[0], 2, 0x1F);
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    let (got, details) = decode_with_correction(&refs).expect("valid chunked set must repair");
    assert_eq!(
        got, d,
        "chunked repair must recover the original descriptor"
    );
    assert!(!details.is_empty(), "the injected error must be corrected");
}

// T2a — 1..=4-error correction across 3 lengths, through public decode_with_correction.
#[test]
fn t2a_correct_1_to_4_errors_across_lengths() {
    for d in [
        wpkh_descriptor(3),
        wpkh_descriptor(15),
        multi_chunk_descriptor(),
    ] {
        let chunks = split(&d).unwrap();
        for count in 1..=4usize {
            let mut cs = chunks.clone();
            for p in 1..=count {
                cs[0] = corrupt_chunk_at(&cs[0], p, 0x1F);
            }
            let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
            let (got, details) = decode_with_correction(&refs)
                .unwrap_or_else(|e| panic!("t={count} must correct: {e:?}"));
            assert_eq!(got, d, "t={count} recovered a different descriptor");
            assert!(details.len() >= count, "expected >= {count} corrections");
        }
    }
}

// T2b — correction inside the trailing 13-symbol checksum region.
#[test]
fn t2b_correct_checksum_region_errors() {
    let d = wpkh_descriptor(15);
    let chunks = split(&d).unwrap();
    let dp_len = chunks[0].chars().count() - 3; // post-HRP data-part length
    let mut cs = chunks.clone();
    cs[0] = corrupt_chunk_at(&cs[0], dp_len - 1, 0x1F);
    cs[0] = corrupt_chunk_at(&cs[0], dp_len - 7, 0x1F);
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    let (got, _) = decode_with_correction(&refs).expect("checksum-region errors correct");
    assert_eq!(got, d);
}

// A 5-error pattern (data-part positions) VERIFIED-UNCORRECTABLE for wpkh_descriptor(15)'s
// single chunk. A 5-error pattern is NOT guaranteed uncorrectable (Berlekamp-Massey may
// miscorrect). If T2d fires because this pattern starts to (mis)correct after a fixture/fmt
// change, pick another 5-position set (try [2,5,8,11,14] / [1,3,6,9,12] / …) until
// decode_with_correction errs, and update this const + comment.
const UNCORRECTABLE_5ERR: [usize; 5] = [1, 4, 7, 10, 13];

// T2c — randomized 5–8-error sweep. ASSERT != Ok(original) (NOT is_err — md miscorrects to a
// different codeword at ~2^-26). Seeded xorshift, no rand dep.
#[test]
fn t2c_five_to_eight_errors_never_return_original() {
    let d = wpkh_descriptor(15);
    let original = d.clone();
    let chunks = split(&d).unwrap();
    let dp_len = chunks[0].chars().count() - 3;
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for trial in 0..300u32 {
        for n_err in 5..=8usize {
            let mut positions = std::collections::BTreeSet::new();
            while positions.len() < n_err {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                positions.insert((x as usize) % dp_len);
            }
            let mut c0 = chunks[0].clone();
            for &p in &positions {
                c0 = corrupt_chunk_at(&c0, p, ((x as u8) | 1) & 0x1F);
            }
            let mut cs = chunks.clone();
            cs[0] = c0;
            let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
            if let Ok((got, _)) = decode_with_correction(&refs) {
                assert_ne!(
                    got, original,
                    "trial {trial} n_err {n_err}: 5-8 errors silently returned the original"
                );
            }
        }
    }
}

// T2d — the verified-uncorrectable deterministic 5-error pattern → Err.
#[test]
fn t2d_deterministic_five_error_is_err() {
    let d = wpkh_descriptor(15);
    let chunks = split(&d).unwrap();
    let mut c0 = chunks[0].clone();
    for p in UNCORRECTABLE_5ERR {
        c0 = corrupt_chunk_at(&c0, p, 0x1F);
    }
    let mut cs = chunks.clone();
    cs[0] = c0;
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    assert!(
        decode_with_correction(&refs).is_err(),
        "UNCORRECTABLE_5ERR must be uncorrectable — if this fires, the chunk symbols changed; \
         pick another 5-position pattern that errs and update the const (see its doc-comment)"
    );
}

// T2h — multi-chunk: 2 different chunks each ≤ 4 errors → Ok(original).
#[test]
fn t2h_multi_chunk_two_corrupted_within_t() {
    let d = multi_chunk_descriptor();
    let chunks = split(&d).unwrap();
    assert!(chunks.len() >= 2);
    let mut cs = chunks.clone();
    cs[0] = corrupt_chunk_at(&cs[0], 2, 0x1F);
    let li = cs.len() - 1;
    cs[li] = corrupt_chunk_at(&cs[li], 2, 0x1F);
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    let (got, _) = decode_with_correction(&refs).expect("each chunk within t corrects");
    assert_eq!(got, d);
}

// T2i — one chunk over t in a valid multi-chunk set: never silently yields the original
// (atomic-abort intent). Robust != Ok(original) invariant; Err is the expected abort, a rare
// chunk-0 miscorrection surfaces as Ok(different), still ≠ original. (if-let avoids Error: PartialEq.)
#[test]
fn t2i_one_chunk_over_t_never_returns_original() {
    let d = multi_chunk_descriptor();
    let chunks = split(&d).unwrap();
    let mut cs = chunks.clone();
    for p in [1usize, 4, 7, 10, 13] {
        cs[0] = corrupt_chunk_at(&cs[0], p, 0x1F);
    }
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    if let Ok((got, _)) = decode_with_correction(&refs) {
        assert_ne!(
            got, d,
            "a 5-error chunk-0 corruption must never reassemble to the original"
        );
    }
}

// ── Cross-chunk validation (T2e/f/g) ────────────────────────────────────────
// restamp_chunk_header: decode a chunk to (header, payload-bits), mutate the
// 37-bit ChunkHeader, re-encode with a freshly recomputed BCH checksum (so the
// per-chunk hard-verify passes and the mutated header reaches reassemble's
// cross-chunk checks). The identity case `restamp(c, |_| {}) == c` is proven by
// restamp_identity_round_trips below.
fn restamp_chunk_header(chunk: &str, mutate: impl FnOnce(&mut ChunkHeader)) -> String {
    let (bytes, bit_count) = unwrap_string(chunk).expect("valid chunk");
    let mut reader = BitReader::with_bit_limit(&bytes, bit_count);
    let mut header = ChunkHeader::read(&mut reader).expect("valid chunk header");
    mutate(&mut header);
    let mut writer = BitWriter::new();
    header.write(&mut writer).expect("mutated header writes");
    // copy the remaining data bits (bit_count - 37) verbatim, MSB-first
    while reader.remaining_bits() > 0 {
        let take = reader.remaining_bits().min(32);
        let bits = reader.read_bits(take).expect("read remaining payload bits");
        writer.write_bits(bits, take);
    }
    let new_bytes = writer.into_bytes();
    wrap_payload(&new_bytes, bit_count).expect("re-wrap with recomputed BCH")
}

// Step-1a proof: the identity restamp reproduces every chunk byte-for-byte, AND
// the fixture splits into >=3 chunks (T2f needs index-gap room).
#[test]
fn restamp_identity_round_trips() {
    let chunks = split(&multi_chunk_descriptor()).unwrap();
    assert!(
        chunks.len() >= 3,
        "fixture must split into >=3 chunks; got {}",
        chunks.len()
    );
    for c in &chunks {
        assert_eq!(
            &restamp_chunk_header(c, |_| {}),
            c,
            "identity restamp must reproduce the chunk"
        );
    }
}

// T2e — count mismatch across the set → ChunkSetInconsistent (unit variant).
#[test]
fn t2e_reassemble_rejects_count_mismatch() {
    let chunks = split(&multi_chunk_descriptor()).unwrap();
    let mut cs = chunks.clone();
    cs[0] = restamp_chunk_header(&cs[0], |h| h.count = h.count.wrapping_add(1));
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    assert!(matches!(
        reassemble(&refs),
        Err(Error::ChunkSetInconsistent)
    ));
}

// T2f — duplicate index 0 → a gap at the missing index → ChunkIndexGap.
#[test]
fn t2f_reassemble_rejects_index_gap() {
    let chunks = split(&multi_chunk_descriptor()).unwrap();
    assert!(chunks.len() >= 3, "need >=3 chunks; enlarge the fixture");
    let mut cs = chunks.clone();
    cs[1] = restamp_chunk_header(&cs[1], |h| h.index = 0); // duplicate index 0
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    assert!(matches!(
        reassemble(&refs),
        Err(Error::ChunkIndexGap { .. })
    ));
}

// T2g — every header carries a foreign csid (header-consistent) but the reassembled
// payload derives a different csid → ChunkSetIdMismatch.
#[test]
fn t2g_reassemble_rejects_derived_csid_mismatch() {
    let chunks = split(&multi_chunk_descriptor()).unwrap();
    let foreign: u32 = 0x0_AAAA;
    let cs: Vec<String> = chunks
        .iter()
        .map(|c| restamp_chunk_header(c, |h| h.chunk_set_id = foreign))
        .collect();
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    assert!(matches!(
        reassemble(&refs),
        Err(Error::ChunkSetIdMismatch { .. })
    ));
}
