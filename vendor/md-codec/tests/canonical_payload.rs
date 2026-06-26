//! KATs for the public, round-trippable canonical packed-payload surface
//! (`Descriptor::canonical_payload_bytes` / `from_canonical_payload_bytes`).
//!
//! These two methods expose the deterministic pre-chunking packed payload of
//! an `md1` descriptor as a bit-precise `(Vec<u8>, total_bits)` pair for a
//! downstream consumer. The payload is bit-aligned (the final byte is
//! zero-padded), so the exact `total_bits` count — not just the byte length —
//! is load-bearing for a faithful round-trip.
//!
//! Fixtures (`cell_1_wpkh_template_only`, `cell_7_wsh_2of3_full`,
//! `multi_chunk_descriptor`) are replicated from `tests/wallet_policy.rs` /
//! `tests/chunking.rs`; the canonicalization-normalizer KAT mirrors the P2
//! property of `tests/proptest_roundtrip.rs` as a concrete vector.

use md_codec::encode::Descriptor;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;

// ─── Replicated fixtures / helpers ───────────────────────────────────────

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

fn bip48_type_2_path() -> OriginPath {
    OriginPath {
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
    }
}

/// 33-byte compressed secp256k1 generator point (G).
fn valid_compressed_pubkey() -> [u8; 33] {
    let mut out = [0u8; 33];
    out[0] = 0x02;
    let x: [u8; 32] = [
        0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87, 0x0B,
        0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B, 0x16, 0xF8,
        0x17, 0x98,
    ];
    out[1..].copy_from_slice(&x);
    out
}

/// Structurally-valid 65-byte xpub (32-byte chain code || 33-byte pubkey).
fn make_xpub(seed: u8) -> [u8; 65] {
    let mut x = [0u8; 65];
    for b in x[0..32].iter_mut() {
        *b = seed;
    }
    x[32..65].copy_from_slice(&valid_compressed_pubkey());
    x
}

fn wpkh_at_0() -> Node {
    Node {
        tag: Tag::Wpkh,
        body: Body::KeyArg { index: 0 },
    }
}

fn wsh_sortedmulti_2of3() -> Node {
    Node {
        tag: Tag::Wsh,
        body: Body::Children(vec![Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        }]),
    }
}

/// 1-of-1 cell-1 (template-only) wpkh: no Fingerprints, no Pubkeys.
fn cell_1_wpkh_template_only() -> Descriptor {
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: wpkh_at_0(),
        tlv: TlvSection::new_empty(),
    }
}

/// 2-of-3 cell-7 wsh-sortedmulti: BIP-48 type-2 origin + per-`@N`
/// Fingerprints + Pubkeys for all three cosigners (wallet-policy mode,
/// multi 0x02-TLV).
fn cell_7_wsh_2of3_full() -> Descriptor {
    let mut d = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(bip48_type_2_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: wsh_sortedmulti_2of3(),
        tlv: TlvSection::new_empty(),
    };
    d.tlv.fingerprints = Some(vec![(0u8, [0x11; 4]), (1u8, [0x22; 4]), (2u8, [0x33; 4])]);
    d.tlv.pubkeys = Some(vec![
        (0u8, make_xpub(0x10)),
        (1u8, make_xpub(0x20)),
        (2u8, make_xpub(0x30)),
    ]);
    d
}

/// Divergent-path 4-cosigner wallet with 15 hardened path components per
/// cosigner — comfortably above the single-string limit, so it spans
/// multiple chunks (replicated from `tests/chunking.rs`).
fn multi_chunk_descriptor() -> Descriptor {
    let mut paths = Vec::new();
    for cosigner in 0..4u32 {
        let mut components = Vec::new();
        for i in 0..15u32 {
            components.push(PathComponent {
                hardened: true,
                value: cosigner * 100 + i + 1,
            });
        }
        paths.push(OriginPath { components });
    }
    Descriptor {
        n: 4,
        path_decl: PathDecl {
            n: 4,
            paths: PathDeclPaths::Divergent(paths),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: (0..4).collect(),
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    }
}

// ─── KAT 1: round-trip (keyless template) ────────────────────────────────

#[test]
fn kat1_roundtrip_template_only() {
    let d = cell_1_wpkh_template_only();
    let (bytes, total_bits) = d.canonical_payload_bytes().expect("encodes");
    let back = Descriptor::from_canonical_payload_bytes(&bytes, total_bits).expect("decodes");
    assert_eq!(back, d);
}

// ─── KAT 2: round-trip (wallet-policy, multi-0x02-TLV) + determinism ──────

#[test]
fn kat2_roundtrip_wallet_policy_and_determinism() {
    let d = cell_7_wsh_2of3_full();
    let (bytes, total_bits) = d.canonical_payload_bytes().expect("encodes");
    let back = Descriptor::from_canonical_payload_bytes(&bytes, total_bits).expect("decodes");
    assert_eq!(back, d);

    // Re-encoding the decoded descriptor yields byte-identical output
    // (determinism through the full round-trip).
    let (bytes2, total_bits2) = back.canonical_payload_bytes().expect("re-encodes");
    assert_eq!(
        (bytes2.as_slice(), total_bits2),
        (bytes.as_slice(), total_bits)
    );

    // Bytes are stable across repeated calls on the same descriptor.
    let (bytes3, total_bits3) = d.canonical_payload_bytes().expect("re-encodes again");
    assert_eq!(
        (bytes3.as_slice(), total_bits3),
        (bytes.as_slice(), total_bits)
    );
}

// ─── KAT 3: canonicalization-as-normalizer ───────────────────────────────

#[test]
fn kat3_canonicalization_is_normalizer() {
    // Non-canonical: tree's first-occurrence sequence is [2, 0, 1].
    let non_canonical = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(bip48_type_2_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![2, 0, 1],
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    };
    // Canonical form: first-occurrence sequence [0, 1, 2].
    let canonical = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(bip48_type_2_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: wsh_sortedmulti_2of3(),
        tlv: TlvSection::new_empty(),
    };

    let (bytes_nc, bits_nc) = non_canonical.canonical_payload_bytes().expect("nc encodes");
    let (bytes_c, bits_c) = canonical.canonical_payload_bytes().expect("c encodes");

    // The encoder canonicalizes internally, so both forms produce
    // byte-identical canonical payloads.
    assert_eq!((bytes_nc.as_slice(), bits_nc), (bytes_c.as_slice(), bits_c));

    // And both decode back to the canonical descriptor.
    let back = Descriptor::from_canonical_payload_bytes(&bytes_nc, bits_nc).expect("decodes");
    assert_eq!(back, canonical);
}

// ─── KAT 4: multi-chunk descriptor round-trip ────────────────────────────

#[test]
fn kat4_roundtrip_multi_chunk_descriptor() {
    let d = multi_chunk_descriptor();
    let (bytes, total_bits) = d.canonical_payload_bytes().expect("encodes");
    let back = Descriptor::from_canonical_payload_bytes(&bytes, total_bits).expect("decodes");
    assert_eq!(back, d);
}

// ─── KAT 5: total_bits is load-bearing ───────────────────────────────────

/// Decode with a `total_bits` that differs from the canonical count and
/// require that the result is NOT a silent reproduction of `d` — it must
/// either error or differ. This is what makes returning only the bytes
/// insufficient: the exact bit count is part of the payload's identity.
fn assert_wrong_bits_not_silently_ok(d: &Descriptor, bytes: &[u8], wrong_bits: usize) {
    match Descriptor::from_canonical_payload_bytes(bytes, wrong_bits) {
        Ok(other) => assert_ne!(
            &other, d,
            "decoding with total_bits={wrong_bits} must not reproduce the descriptor"
        ),
        Err(_) => { /* rejected outright — also acceptable */ }
    }
}

#[test]
fn kat5_total_bits_is_load_bearing() {
    let d = cell_7_wsh_2of3_full();
    let (bytes, total_bits) = d.canonical_payload_bytes().expect("encodes");

    // The fixture is bit-unaligned (trailing zero-pad bits exist in the final
    // byte). NOTE / surprise-vs-recon: the decoder treats up to 7 trailing
    // ZERO bits as codex32 padding (its TLV-rollback tolerance), so an
    // OVER-count up to the byte boundary (`bytes.len()*8`) is *absorbed* and
    // decodes identically. That direction therefore does NOT prove the bit
    // count is load-bearing.
    assert_ne!(
        bytes.len() * 8,
        total_bits,
        "fixture must be bit-unaligned for this KAT to be meaningful"
    );

    // The robust demonstration is the UNDER-count direction: truncating the
    // declared bit count drops real payload bits, which can never be absorbed
    // as trailing padding — so even a single-bit truncation must be rejected
    // or yield a different descriptor.
    assert_wrong_bits_not_silently_ok(&d, &bytes, total_bits - 1);
    assert_wrong_bits_not_silently_ok(&d, &bytes, total_bits - 8);

    // An OVER-count that exceeds the ≤7-bit padding tolerance with a non-zero
    // extra byte is also not silently absorbed.
    let mut padded = bytes.clone();
    padded.push(0xFF);
    assert_wrong_bits_not_silently_ok(&d, &padded, total_bits + 8);
}
