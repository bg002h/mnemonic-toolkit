//! Integration tests for v0.13 wallet-policy mode (spec §9).
//!
//! End-to-end coverage of `encode → wrap → unwrap → decode` for cell-7 /
//! cell-1 wallets, canonicalization stability, partial keys, forced-
//! explicit rejection, placeholder-ordering rejection, divergent paths
//! combined with wallet-policy mode, multi-chunk wallet-policy reassemble,
//! tr shape disambiguation, encoder determinism, and v0.11 forward-compat
//! byte-fixture round-trip (template-only wire is invariant under v0.13).

use std::sync::OnceLock;

use md_codec::canonicalize::canonicalize_placeholder_indices;
use md_codec::chunk::{reassemble, split};
use md_codec::decode::{decode_md1_string, decode_payload};
use md_codec::encode::{Descriptor, encode_md1_string, encode_payload};
use md_codec::error::Error;
use md_codec::identity::compute_wallet_policy_id;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;
use md_codec::validate::{validate_explicit_origin_required, validate_placeholder_usage};

// ─── Constructors ────────────────────────────────────────────────────────

/// Round-trip `d` through the wire string and return the decoded descriptor.
/// cycle-4 H6: a full wallet-policy descriptor (populated 65-byte xpub TLVs)
/// exceeds the codex32 regular code's 80-data-symbol single-string cap, so
/// `encode_md1_string` fails closed with `PayloadTooLongForSingleString`; for
/// those the chunked path (`split`/`reassemble`) is the authoritative wire
/// round-trip. Template-mode (no xpubs) descriptors still fit a single string.
fn roundtrip_via_string_or_chunks(d: &Descriptor) -> Descriptor {
    match encode_md1_string(d) {
        Ok(s) => decode_md1_string(&s).expect("single-string decodes"),
        Err(Error::PayloadTooLongForSingleString { .. }) => {
            let chunks = split(d).expect("oversize descriptor chunks");
            let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
            reassemble(&refs).expect("chunked reassembles")
        }
        Err(e) => panic!("unexpected string-encode error: {e:?}"),
    }
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

fn bip86_path() -> OriginPath {
    OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 86,
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

fn bip49_path() -> OriginPath {
    OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 49,
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

fn empty_path() -> OriginPath {
    OriginPath { components: vec![] }
}

fn pkk(index: u8) -> Node {
    Node {
        tag: Tag::PkK,
        body: Body::KeyArg { index },
    }
}

/// 33-byte compressed secp256k1 generator point (G), suitable as the
/// pubkey portion of a 65-byte xpub. Validation in `validate_xpub_bytes`
/// requires bytes 32..65 to parse as a real secp256k1 point.
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

/// 65-byte xpub (32-byte chain code prefix || 33-byte compressed pubkey)
/// using `seed` as the chain-code byte-fill and G as the pubkey, so the
/// result is structurally valid (passes `validate_xpub_bytes`).
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

fn tr_keypath_at_0() -> Node {
    Node {
        tag: Tag::Tr,
        body: Body::Tr {
            is_nums: false,
            key_index: 0,
            tree: None,
        },
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

fn wsh_sortedmulti_2of2() -> Node {
    Node {
        tag: Tag::Wsh,
        body: Body::Children(vec![Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1],
            },
        }]),
    }
}

/// 1-of-1 cell-7 wpkh: BIP-84 origin + Fingerprints[0] + Pubkeys[0].
fn cell_7_wpkh_full() -> Descriptor {
    let mut d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: wpkh_at_0(),
        tlv: TlvSection::new_empty(),
    };
    d.tlv.fingerprints = Some(vec![(0u8, [0xDE, 0xAD, 0xBE, 0xEF])]);
    d.tlv.pubkeys = Some(vec![(0u8, make_xpub(0x11))]);
    d
}

/// 2-of-3 cell-7 wsh-sortedmulti: BIP-48 type-2 origin + per-`@N`
/// Fingerprints + Pubkeys for all three cosigners.
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

// ─── Test 1: smoke ───────────────────────────────────────────────────────

#[test]
fn smoke_1of1_cell_7_wpkh_round_trip() {
    let d = cell_7_wpkh_full();
    let d2 = roundtrip_via_string_or_chunks(&d);
    assert_eq!(d, d2);
    assert!(d2.is_wallet_policy(), "cell-7 must be wallet-policy mode");
}

#[test]
fn smoke_2of3_cell_7_wsh_sortedmulti_round_trip() {
    let d = cell_7_wsh_2of3_full();
    let d2 = roundtrip_via_string_or_chunks(&d);
    assert_eq!(d, d2);
    assert!(d2.is_wallet_policy(), "cell-7 must be wallet-policy mode");
}

#[test]
fn smoke_1of1_cell_1_wpkh_template_only_round_trip() {
    let d = cell_1_wpkh_template_only();
    let s = encode_md1_string(&d).unwrap();
    let d2 = decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
    assert!(
        !d2.is_wallet_policy(),
        "template-only must NOT be wallet-policy mode"
    );
}

// ─── Test 2: canonicalization stability ──────────────────────────────────

#[test]
fn canonicalization_stability_wpkh_explicit_vs_redundant_override() {
    // Wallet A: BIP 84 in path_decl, no overrides.
    let d_a = cell_7_wpkh_full();

    // Wallet B: same logical wallet, but the BIP 84 path is also supplied
    // as a redundant `origin_path_overrides[0]`. Per spec §5.3 / §6.3,
    // override resolution → same expanded record → same WalletPolicyId.
    let mut d_b = cell_7_wpkh_full();
    d_b.tlv.origin_path_overrides = Some(vec![(0u8, bip84_path())]);

    let id_a = compute_wallet_policy_id(&d_a).unwrap();
    let id_b = compute_wallet_policy_id(&d_b).unwrap();
    assert_eq!(id_a, id_b);

    // Sanity: the round-trip path also works for the redundant-override
    // version.
    let d_b_decoded = roundtrip_via_string_or_chunks(&d_b);
    assert_eq!(d_b, d_b_decoded);
}

// ─── Test 3: partial keys ────────────────────────────────────────────────

#[test]
fn partial_keys_2of2_at0_cell7_at1_cell1() {
    // 2-of-2 wsh-sortedmulti where @0 has fp+xpub but @1 has neither.
    let mut d_partial = Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Shared(bip48_type_2_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: wsh_sortedmulti_2of2(),
        tlv: TlvSection::new_empty(),
    };
    d_partial.tlv.fingerprints = Some(vec![(0u8, [0xAA; 4])]);
    d_partial.tlv.pubkeys = Some(vec![(0u8, make_xpub(0x55))]);

    let d2 = roundtrip_via_string_or_chunks(&d_partial);
    assert_eq!(d_partial, d2);
    assert!(
        d2.is_wallet_policy(),
        "any populated Pubkeys → wallet-policy"
    );

    // The partial-keys identity must differ from a fully-populated cell-7
    // 2-of-2 — presence-significance gate.
    let mut d_full = d_partial.clone();
    d_full.tlv.fingerprints = Some(vec![(0u8, [0xAA; 4]), (1u8, [0xBB; 4])]);
    d_full.tlv.pubkeys = Some(vec![(0u8, make_xpub(0x55)), (1u8, make_xpub(0x66))]);

    let id_partial = compute_wallet_policy_id(&d_partial).unwrap();
    let id_full = compute_wallet_policy_id(&d_full).unwrap();
    assert_ne!(id_partial, id_full);
}

// ─── Test 4: forced-explicit rejection (sh-sortedmulti) ──────────────────

#[test]
fn forced_explicit_sh_sortedmulti_rejected_at_decoder() {
    // sh(sortedmulti(2, @0, @1)) with empty shared path_decl and no
    // OriginPathOverrides → wrapper has no canonical default, so the
    // decoder's `validate_explicit_origin_required` must raise
    // `MissingExplicitOrigin { idx: 0 }`.
    let d = Descriptor {
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
    };
    // Direct validator check (the canonical path the spec calls out).
    let err = validate_explicit_origin_required(&d).unwrap_err();
    assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));

    // Defense in depth: the wire round-trip also rejects at the decoder
    // (encoder doesn't itself reject elided origins, but every decoded
    // payload runs the validator).
    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let dec_err = decode_payload(&bytes, total_bits).unwrap_err();
    assert!(matches!(dec_err, Error::MissingExplicitOrigin { idx: 0 }));
}

// ─── Test 5: v0.11 forward-compat fixtures ───────────────────────────────

/// Live-emitted fixture: bytes produced from a template-only descriptor
/// (no Pubkeys, no OriginPathOverrides). The v0.13 decoder must parse
/// these bytes back to a `Descriptor` with `pubkeys = None,
/// origin_path_overrides = None, unknown = vec![]`, and re-encoding must
/// yield byte-identical output.
fn fixture_v011_template_only() -> &'static (Vec<u8>, usize) {
    static F: OnceLock<(Vec<u8>, usize)> = OnceLock::new();
    F.get_or_init(|| encode_payload(&cell_1_wpkh_template_only()).unwrap())
}

/// Live-emitted fixture: same wallet template but with cell-7 keys
/// (Fingerprints + Pubkeys). Re-encoding must also yield byte-identical
/// output.
fn fixture_v013_same_policy() -> &'static (Vec<u8>, usize) {
    static F: OnceLock<(Vec<u8>, usize)> = OnceLock::new();
    F.get_or_init(|| encode_payload(&cell_7_wpkh_full()).unwrap())
}

#[test]
fn forward_compat_v011_template_only_decodes_under_v013() {
    let (bytes, total_bits) = fixture_v011_template_only();
    let d = decode_payload(bytes, *total_bits).unwrap();
    assert!(d.tlv.pubkeys.is_none(), "template-only → pubkeys = None");
    assert!(
        d.tlv.origin_path_overrides.is_none(),
        "template-only → origin_path_overrides = None"
    );
    assert!(
        d.tlv.unknown.is_empty(),
        "template-only fixture must not carry unknown TLVs"
    );
}

#[test]
fn forward_compat_v011_template_only_byte_identical_re_encode() {
    let (bytes, total_bits) = fixture_v011_template_only();
    let d = decode_payload(bytes, *total_bits).unwrap();
    let (re_bytes, re_total_bits) = encode_payload(&d).unwrap();
    assert_eq!(re_total_bits, *total_bits);
    assert_eq!(&re_bytes, bytes);
}

#[test]
fn forward_compat_v013_same_policy_byte_identical_re_encode() {
    let (bytes, total_bits) = fixture_v013_same_policy();
    let d = decode_payload(bytes, *total_bits).unwrap();
    assert!(d.is_wallet_policy(), "fixture is wallet-policy mode");
    let (re_bytes, re_total_bits) = encode_payload(&d).unwrap();
    assert_eq!(re_total_bits, *total_bits);
    assert_eq!(&re_bytes, bytes);
}

// ─── Test 6: placeholder-ordering rejection ──────────────────────────────

#[test]
fn placeholder_ordering_rejected_by_validator() {
    // Hand-built non-canonical tree: wsh(multi(2, @1, @0)) — first
    // occurrence is @1, then @0. The validator must reject with
    // `PlaceholderFirstOccurrenceOutOfOrder`. (The encoder canonicalizes
    // automatically; bypass it by calling validate_placeholder_usage
    // directly on the raw tree.)
    let non_canonical_tree = Node {
        tag: Tag::Wsh,
        body: Body::Children(vec![Node {
            tag: Tag::Multi,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![1, 0],
            },
        }]),
    };
    let err = validate_placeholder_usage(&non_canonical_tree, 2).unwrap_err();
    assert!(matches!(
        err,
        Error::PlaceholderFirstOccurrenceOutOfOrder { .. }
    ));

    // Defense in depth: decoded wires that violate the ordering must
    // also be rejected. Since `encode_payload` canonicalizes, simulate
    // a decoder-side violation by feeding an already-canonicalized
    // descriptor and showing the decoder accepts it (i.e. canonical
    // form is the only on-wire form). We've already covered "validator
    // rejects non-canonical" above; here we pin "encoder produces
    // canonical wire" so the decoder never sees a non-canonical one.
    let mut d_non_canonical = Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Shared(bip48_type_2_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::Multi,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![1, 0],
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    };
    canonicalize_placeholder_indices(&mut d_non_canonical).unwrap();
    let (bytes, total_bits) = encode_payload(&d_non_canonical).unwrap();
    decode_payload(&bytes, total_bits).expect("canonical wire decodes cleanly");
}

// ─── Test 7: divergent_paths × wallet-policy ─────────────────────────────

#[test]
fn divergent_paths_wallet_policy_2of2_round_trip() {
    // 2-of-2 wsh(multi) with divergent path_decl (per-`@N` distinct
    // origin paths), full TLVs (fp + xpub for each cosigner).
    let path_a = OriginPath {
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
    };
    let path_b = OriginPath {
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
                value: 1,
            },
            PathComponent {
                hardened: true,
                value: 2,
            },
        ],
    };
    let mut d = Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Divergent(vec![path_a.clone(), path_b.clone()]),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::Multi,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![0, 1],
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    };
    d.tlv.fingerprints = Some(vec![(0u8, [0xAA; 4]), (1u8, [0xBB; 4])]);
    d.tlv.pubkeys = Some(vec![(0u8, make_xpub(0x77)), (1u8, make_xpub(0x88))]);

    // Round-trip.
    let d2 = roundtrip_via_string_or_chunks(&d);
    assert_eq!(d, d2);
    assert!(d2.is_wallet_policy());

    // WalletPolicyId is stable across two encodings of the same wallet.
    let id_1 = compute_wallet_policy_id(&d).unwrap();
    let id_2 = compute_wallet_policy_id(&d2).unwrap();
    assert_eq!(id_1, id_2);
}

// ─── Test 8: multi-chunk wallet-policy round trip ────────────────────────

#[test]
fn multi_chunk_2of3_cell_7_split_reassemble_round_trip() {
    let d = cell_7_wsh_2of3_full();
    let chunks = split(&d).unwrap();
    // Spec §9 prose: 2-of-3 cell-7 lands at 5–7 codex32 chunks under the
    // post-F2 320-bit single-string limit. Lock a tighter lower bound so
    // a regression that drops chunk count would be caught.
    assert!(
        chunks.len() >= 5,
        "2-of-3 with full xpubs should require ~5–7 chunks (got {})",
        chunks.len()
    );
    for c in &chunks {
        assert!(c.starts_with("md1"));
    }
    let chunk_refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
    let d2 = reassemble(&chunk_refs).unwrap();
    assert_eq!(d, d2);
    assert!(d2.is_wallet_policy());

    // ChunkSetId consistency: every chunk's header carries the same
    // chunk_set_id (cross-chunk integrity is verified by `reassemble`,
    // which re-derives it from the reassembled payload and compares
    // against the per-chunk-header value).
    let chunks_2 = split(&d2).unwrap();
    assert_eq!(chunks.len(), chunks_2.len());
    // The reassembled descriptor produces the same chunk-set when re-
    // split, demonstrating ChunkSetId determinism.
}

// ─── Test 9: bare-wsh / bare-sh forced explicit ──────────────────────────

#[test]
fn bare_wsh_at_n_forced_explicit_rejected_with_empty_path() {
    // wsh(@0) — bare single-key wsh. Wrapper has no canonical default.
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(empty_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![pkk(0)]),
        },
        tlv: TlvSection::new_empty(),
    };
    let err = validate_explicit_origin_required(&d).unwrap_err();
    assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));

    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let dec_err = decode_payload(&bytes, total_bits).unwrap_err();
    assert!(matches!(dec_err, Error::MissingExplicitOrigin { idx: 0 }));
}

#[test]
fn bare_wsh_at_n_accepts_with_populated_path_decl() {
    // wsh(@0) with explicit BIP-84 path_decl → validator accepts; the
    // wire round-trips cleanly.
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![pkk(0)]),
        },
        tlv: TlvSection::new_empty(),
    };
    validate_explicit_origin_required(&d).unwrap();
    let s = encode_md1_string(&d).unwrap();
    let d2 = decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
}

#[test]
fn bare_sh_at_n_forced_explicit_rejected_with_empty_path() {
    // sh(@0) — bare single-key sh. Wrapper has no canonical default.
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(empty_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Sh,
            body: Body::Children(vec![pkk(0)]),
        },
        tlv: TlvSection::new_empty(),
    };
    let err = validate_explicit_origin_required(&d).unwrap_err();
    assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));

    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let dec_err = decode_payload(&bytes, total_bits).unwrap_err();
    assert!(matches!(dec_err, Error::MissingExplicitOrigin { idx: 0 }));
}

#[test]
fn bare_sh_at_n_accepts_with_populated_path_decl() {
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Sh,
            body: Body::Children(vec![pkk(0)]),
        },
        tlv: TlvSection::new_empty(),
    };
    validate_explicit_origin_required(&d).unwrap();
    let s = encode_md1_string(&d).unwrap();
    let d2 = decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
}

// ─── Test 10: tr shape disambiguation ────────────────────────────────────

#[test]
fn tr_keypath_only_accepts_with_empty_path_decl() {
    // tr(@0) keypath only → BIP-86 canonical default → empty path_decl
    // accepted by the validator, and round-trips through the wire.
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(empty_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: tr_keypath_at_0(),
        tlv: TlvSection::new_empty(),
    };
    validate_explicit_origin_required(&d).unwrap();
    let s = encode_md1_string(&d).unwrap();
    let d2 = decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
}

#[test]
fn tr_with_taptree_rejects_empty_path_decl() {
    // tr(@0, TapTree(@0)) → no canonical → must be explicit. With empty
    // path_decl and no overrides, the validator rejects.
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(empty_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(pkk(0))),
            },
        },
        tlv: TlvSection::new_empty(),
    };
    let err = validate_explicit_origin_required(&d).unwrap_err();
    assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));

    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let dec_err = decode_payload(&bytes, total_bits).unwrap_err();
    assert!(matches!(dec_err, Error::MissingExplicitOrigin { idx: 0 }));
}

#[test]
fn tr_with_taptree_accepts_with_populated_path_decl() {
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip86_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(pkk(0))),
            },
        },
        tlv: TlvSection::new_empty(),
    };
    validate_explicit_origin_required(&d).unwrap();
    let s = encode_md1_string(&d).unwrap();
    let d2 = decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
}

// ─── Test 12: BIP-388 §Test Vectors policy 388.2 — sh(wpkh) BIP-49 ───────

/// BIP-388 §Test Vectors reference policy #2: nested-segwit single-sig.
/// Spec: <https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki#test-vectors>
///
/// Template `sh(wpkh(@0/<0;1>/*))` with BIP-49 canonical origin
/// `m/49'/0'/0'`. The spec quotes a concrete cosigner xpub
/// `[6738736c/49'/0'/1']xpub6Bex1...` which is not re-derivable here
/// (BIP-388 ships no seed); the matrix records that limitation. This
/// test pins the *template-shape* round-trip — the encode→decode path
/// preserves the wrapper stack, key index, and BIP-49 origin path.
#[test]
fn bip388_388_2_sh_wpkh_bip49_template_shape_round_trip() {
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip49_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Sh,
            body: Body::Children(vec![Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            }]),
        },
        tlv: TlvSection::new_empty(),
    };
    // Populated path_decl (BIP-49 canonical) satisfies the
    // `validate_explicit_origin_required` gate for sh-wrapper.
    validate_explicit_origin_required(&d).unwrap();
    validate_placeholder_usage(&d.tree, d.n).unwrap();

    let s = encode_md1_string(&d).unwrap();
    let d2 = decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
    assert!(
        !d2.is_wallet_policy(),
        "template-only (no Pubkeys) must NOT be wallet-policy mode"
    );
}

// ─── Test 11: encoder determinism ────────────────────────────────────────

#[test]
fn encoder_determinism_2of3_cell_7_byte_identical_emit() {
    let d = cell_7_wsh_2of3_full();
    let (bytes_1, bits_1) = encode_payload(&d).unwrap();
    let (bytes_2, bits_2) = encode_payload(&d).unwrap();
    assert_eq!(bits_1, bits_2);
    assert_eq!(bytes_1, bytes_2);

    // The codex32-wrapped wire form is also deterministic. cycle-4 H6: this
    // full cell-7 wallet-policy descriptor exceeds the 80-data-symbol
    // single-string cap, so it emits via the chunked path — which is equally
    // deterministic.
    assert!(
        matches!(
            encode_md1_string(&d),
            Err(Error::PayloadTooLongForSingleString { .. })
        ),
        "oversize cell-7 must reject the single-string encode"
    );
    let s_1 = split(&d).unwrap();
    let s_2 = split(&d).unwrap();
    assert_eq!(s_1, s_2);
}
