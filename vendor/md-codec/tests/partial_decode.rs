//! P0 — pathless/dead-card partial-decode (md-codec leg).
//!
//! See `design/IMPLEMENTATION_PLAN_pathless_partial_decode.md` +
//! `design/SPEC_pathless_partial_decode.md` (mnemonic-toolkit repo, R0-GREEN).
//!
//! Covers:
//! - P0.2: partial-allowing decode threaded through all three layers
//!   (`decode_payload_with_opts`, `decode_md1_string_with_opts`,
//!   `chunk::reassemble_with_opts`), with the funds-critical RED-proof that
//!   the content-id oracle (derived-chunk-set-id check) stays enforced even
//!   under partial mode.
//! - P0.3 (I-1): the `EmptyOriginOverride` reject at the decode layer —
//!   unconditional (even for canonical shapes) and fatal-in-partial.

use md_codec::bitstream::{BitReader, BitWriter};
use md_codec::chunk::{ChunkHeader, reassemble, reassemble_with_opts, split};
use md_codec::codex32::{unwrap_string, wrap_payload};
use md_codec::decode::{
    DecodeOpts, decode_md1_string, decode_md1_string_with_opts, decode_payload,
    decode_payload_with_opts,
};
use md_codec::encode::{Descriptor, encode_payload};
use md_codec::error::Error;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;

fn empty_path() -> OriginPath {
    OriginPath { components: vec![] }
}

fn bip84_path() -> OriginPath {
    OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 84,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
        ],
    }
}

/// bare `sh(sortedmulti(2,@0,@1))` — dead shape (`canonical_origin ==
/// None`), empty shared path_decl, no overrides. A "dead card": the
/// template is fully renderable but the origin is unresolvable.
fn dead_sh_sortedmulti() -> Descriptor {
    Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Shared(empty_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Sh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![0, 1],
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    }
}

/// Multi-chunk DEAD card: n=6 divergent cosigners, `sh(sortedmulti(...))`
/// DIRECT (dead shape — no `wsh(...)` wrapper, unlike bch_adversarial.rs's
/// canonical `multi_chunk_descriptor()`), idx 0's divergent path EMPTY
/// (unresolved) while idx 1..5 carry full 15-component paths (bulk, to
/// force >=2 chunks — mirrors bch_adversarial.rs's sizing pattern).
fn multi_chunk_dead_descriptor() -> Descriptor {
    let mut paths: Vec<OriginPath> = (0..6u32)
        .map(|c| OriginPath {
            components: (0..15u32)
                .map(|i| PathComponent {
                    hardened: true,
                    value: c * 100 + i + 1,
                })
                .collect(),
        })
        .collect();
    paths[0] = empty_path();
    Descriptor {
        n: 6,
        path_decl: PathDecl {
            n: 6,
            paths: PathDeclPaths::Divergent(paths),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Sh,
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

/// Restamp a chunk's header, re-wrapping with recomputed BCH. Mirrors
/// `bch_adversarial.rs`'s local helper of the same name/shape (T2g
/// pattern) — duplicated here rather than promoted to `common/` since
/// this file is the only other consumer.
fn restamp_chunk_header(chunk: &str, mutate: impl FnOnce(&mut ChunkHeader)) -> String {
    let (bytes, bit_count) = unwrap_string(chunk).expect("valid chunk");
    let mut reader = BitReader::with_bit_limit(&bytes, bit_count);
    let mut header = ChunkHeader::read(&mut reader).expect("valid chunk header");
    mutate(&mut header);
    let mut writer = BitWriter::new();
    header.write(&mut writer).expect("mutated header writes");
    while reader.remaining_bits() > 0 {
        let take = reader.remaining_bits().min(32);
        let bits = reader.read_bits(take).expect("read remaining payload bits");
        writer.write_bits(bits, take);
    }
    let new_bytes = writer.into_bytes();
    wrap_payload(&new_bytes, bit_count).expect("re-wrap with recomputed BCH")
}

// ─── P0.2(a): a MULTI-CHUNK dead card decodes under partial ───────────────

#[test]
fn partial_decode_multi_chunk_dead_card_succeeds() {
    let d = multi_chunk_dead_descriptor();
    let chunks = split(&d).expect("multi-chunk dead card splits");
    assert!(
        chunks.len() >= 2,
        "fixture must force >=2 chunks; got {}",
        chunks.len()
    );
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();

    let opts = DecodeOpts::partial();
    let decoded = reassemble_with_opts(&refs, opts)
        .expect("partial decode of a multi-chunk dead card must succeed");
    assert_eq!(
        decoded.unresolved_origin_indices(),
        vec![0u8],
        "idx 0 carries the empty divergent path"
    );
}

// ─── P0.2(b): funds-critical RED-proof — doctored chunk-set-id REJECTS
//     even under partial mode (content-id oracle intact) ─────────────────

#[test]
fn partial_decode_multi_chunk_dead_card_doctored_csid_still_rejects() {
    let d = multi_chunk_dead_descriptor();
    let chunks = split(&d).expect("multi-chunk dead card splits");
    assert!(chunks.len() >= 2, "need >=2 chunks for this RED-proof");
    let foreign: u32 = 0x0_AAAA;
    let doctored: Vec<String> = chunks
        .iter()
        .map(|c| restamp_chunk_header(c, |h| h.chunk_set_id = foreign))
        .collect();
    let refs: Vec<&str> = doctored.iter().map(String::as_str).collect();

    let opts = DecodeOpts::partial();
    let err = reassemble_with_opts(&refs, opts)
        .expect_err("a doctored-chunk-set-id dead card must still reject under partial decode");
    assert!(
        matches!(err, Error::ChunkSetIdMismatch { .. }),
        "content-id oracle must stay enforced under partial mode, got {err:?}"
    );
}

// ─── P0.2(c): a non-chunked dead card decodes under partial ──────────────

#[test]
fn partial_decode_non_chunked_dead_card_succeeds() {
    let d = dead_sh_sortedmulti();
    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let opts = DecodeOpts::partial();
    let decoded = decode_payload_with_opts(&bytes, total_bits, opts)
        .expect("partial decode of a non-chunked dead card must succeed");
    assert_eq!(decoded.unresolved_origin_indices(), vec![0u8, 1u8]);
}

#[test]
fn partial_decode_via_md1_string_non_chunked_dead_card_succeeds() {
    let d = dead_sh_sortedmulti();
    let s = md_codec::encode_md1_string(&d).expect("dead card fits a single string");
    let opts = DecodeOpts::partial();
    let decoded = decode_md1_string_with_opts(&s, opts)
        .expect("partial decode via decode_md1_string_with_opts must succeed");
    assert_eq!(decoded.unresolved_origin_indices(), vec![0u8, 1u8]);
}

// ─── I-1 fold: decode_md1_string_with_opts CHUNK-FORM route must
//     propagate `opts` into `reassemble_with_opts` (decode.rs chunked
//     branch). This is the EXACT route P1 `md decode`/`inspect` consumes
//     for a chunked-of-1 dead card, and it had zero opts-propagation
//     coverage — forcing that branch strict left all 460 tests green. ──

#[test]
fn partial_decode_via_md1_string_chunked_of_one_dead_card_succeeds() {
    // A small dead card splits into a SINGLE chunk (chunked-flag set on
    // the first symbol). `decode_md1_string_with_opts` must auto-dispatch
    // it through `reassemble_with_opts(&[s], opts)` — carrying the partial
    // opt through — rather than the non-chunked single-payload path.
    // (Mirrors chunking.rs::decode_md1_string_auto_dispatches_single_chunk
    // for the chunked-of-1 dispatch; adds the partial-opts propagation.)
    let d = dead_sh_sortedmulti();
    let chunks = split(&d).expect("dead card splits");
    assert_eq!(
        chunks.len(),
        1,
        "fixture must be a single (chunked-of-1) chunk"
    );
    let s = &chunks[0];

    let opts = DecodeOpts::partial();
    let decoded = decode_md1_string_with_opts(s, opts).expect(
        "partial decode of a chunked-of-1 dead card via decode_md1_string_with_opts must succeed",
    );
    assert_eq!(
        decoded.unresolved_origin_indices(),
        vec![0u8, 1u8],
        "the chunk route must propagate allow_unresolved_origin into reassemble_with_opts"
    );

    // Strict default (allow=false) must STILL reject the same chunk-form
    // string with MissingExplicitOrigin — proves the chunk route honors
    // the strict default too (byte-identical to pre-P0 chunk decode).
    let err = decode_md1_string(s).unwrap_err();
    assert!(
        matches!(err, Error::MissingExplicitOrigin { idx: 0 }),
        "strict default must reject the chunked-of-1 dead card, got {err:?}"
    );
}

// ─── P0.2(d): strict default (false) still rejects all of the above;
//     the 12 committed MissingExplicitOrigin pins stay green (this file
//     re-pins the two chunked/non-chunked funds-critical shapes) ────────

#[test]
fn strict_default_rejects_non_chunked_dead_card() {
    let d = dead_sh_sortedmulti();
    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let err = decode_payload(&bytes, total_bits).unwrap_err();
    assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));
}

#[test]
fn strict_default_rejects_multi_chunk_dead_card() {
    let d = multi_chunk_dead_descriptor();
    let chunks = split(&d).unwrap();
    assert!(chunks.len() >= 2);
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    let err = reassemble(&refs).unwrap_err();
    assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));
}

#[test]
fn decode_payload_with_opts_default_matches_strict_decode_payload() {
    let d = dead_sh_sortedmulti();
    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let err = decode_payload_with_opts(&bytes, total_bits, DecodeOpts::default()).unwrap_err();
    assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));
}

// ─── Round-trip (byte-identical) sanity for a partial-decoded descriptor ──

#[test]
fn partial_decode_round_trip_byte_identical() {
    let d = dead_sh_sortedmulti();
    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let opts = DecodeOpts::partial();
    let decoded = decode_payload_with_opts(&bytes, total_bits, opts).unwrap();
    let (bytes2, total_bits2) = encode_payload(&decoded).unwrap();
    assert_eq!(bytes, bytes2);
    assert_eq!(total_bits, total_bits2);
}

// ─── P0.3 (I-1): EmptyOriginOverride — unconditional + fatal-in-partial ───

#[test]
fn empty_origin_override_rejected_for_non_canonical_shape_via_path_decl_fallback() {
    // sh(sortedmulti(2,@0,@1)): non-canonical (dead) shape. Shared
    // path_decl IS populated (bip84_path) so idx=0's fallback-to-path_decl
    // check in the OLD validate_explicit_origin_required would have
    // silently passed ("OK via path_decl fallback") even though idx=0's
    // ACTUAL resolved origin (override takes precedence at expand time)
    // is EMPTY. The new unconditional empty-override reject must catch
    // this at decode; policy-id must not silently compute from it either.
    let mut d = Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Sh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![0, 1],
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    };
    d.tlv.origin_path_overrides = Some(vec![(0u8, empty_path())]);

    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let err = decode_payload(&bytes, total_bits).unwrap_err();
    assert!(
        matches!(err, Error::EmptyOriginOverride { idx: 0 }),
        "expected EmptyOriginOverride, got {err:?}"
    );

    let policy_err = md_codec::compute_wallet_policy_id(&d).unwrap_err();
    assert!(
        matches!(policy_err, Error::EmptyOriginOverride { idx: 0 }),
        "policy-id must not silently compute from an empty @0 origin, got {policy_err:?}"
    );
}

#[test]
fn empty_origin_override_rejected_for_canonical_wpkh_shape() {
    // wpkh(@0) — CANONICAL shape (BIP-84 default). The OLD
    // validate_explicit_origin_required early-returns Ok whenever
    // canonical_origin is Some, WITHOUT inspecting overrides at all — an
    // empty override on a canonical-shape wire would have sailed through
    // undetected pre-P0.3. The new unconditional check must reject it
    // regardless of canonical-shape status (I-1a).
    let mut d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(empty_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    };
    d.tlv.origin_path_overrides = Some(vec![(0u8, empty_path())]);

    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let err = decode_payload(&bytes, total_bits).unwrap_err();
    assert!(matches!(err, Error::EmptyOriginOverride { idx: 0 }));
}

#[test]
fn empty_origin_override_still_rejects_under_partial_mode() {
    // I-1b: fatal-in-partial. The empty-override reject must NOT be
    // swallowed by allow_unresolved_origin — it is a distinct error class
    // from MissingExplicitOrigin.
    let mut d = dead_sh_sortedmulti();
    d.tlv.origin_path_overrides = Some(vec![(0u8, empty_path())]);
    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let opts = DecodeOpts::partial();
    let err = decode_payload_with_opts(&bytes, total_bits, opts).unwrap_err();
    assert!(
        matches!(err, Error::EmptyOriginOverride { idx: 0 }),
        "empty-override must stay FATAL even in partial-allowing decode mode, got {err:?}"
    );
}
