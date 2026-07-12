//! Frozen wire-byte / chunk-string goldens (test-hardening T3-b, eval §2 #4).
//!
//! **Gap this closes:** every existing `md-codec` round-trip test
//! (`encode_payload` → `decode_payload`, or `split` → `reassemble`) encodes
//! *and* decodes with the same code under test. A **symmetric** wire-layout
//! regression — e.g. swapping two TLV tag-const VALUES, reordering
//! `Divergent` per-key paths on both write and read, or shifting the
//! chunk-split byte boundary — round-trips cleanly and passes every such
//! test while silently bricking an already-engraved steel plate whose bytes
//! were produced by a *prior* build. The only defense is a **frozen
//! historical oracle**: a literal byte/string value captured from a known-
//! good build and pinned as a `const`, independent of whatever the *future*
//! code under test happens to compute.
//!
//! **Construction:** direct `Descriptor` struct-literals via the public
//! `md-codec` API (no descriptor-string parser exists inside `md-codec`
//! itself — that lives downstream in `md-cli`). This mirrors the existing
//! idiom in `tests/wallet_policy.rs` and `tests/per_key_use_site_override.rs`
//! (both already construct `Descriptor` this way, compiling and passing in
//! CI today). See `mnemonic-toolkit:design/RECON_T3b_api_feasibility.md` for the full
//! feasibility trace.
//!
//! **Oracle honesty:** every frozen `const` below is the codec's own past
//! output — NOT an externally-independent re-derivation. Provenance (crate
//! version + git SHA + exact generating call) is documented per-const so a
//! future reader can tell exactly which build produced it.
//!
//! **Scope:** additive TEST-only. No `src/` change. Does NOT extend
//! `md_codec::test_vectors::MANIFEST` or the `md vectors` generator (that
//! would require production `Vector`-struct + `md-cli` generator edits —
//! deferred as FOLLOWUP `md-corpus-tlv-shapes-in-manifest-export`, out of
//! scope here). Does NOT re-add ≥2-chunk *coverage* (already shipped via
//! `wsh_sortedmulti_2chunk` in `test_vectors.rs`) — the (e) golden below
//! pins that SAME vector's WIRE, not new coverage.

use md_codec::encode::{Descriptor, encode_payload};
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};

// ─── Shared construction helpers (mirrors wallet_policy.rs / per_key_use_site_override.rs) ───

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

/// 33-byte compressed secp256k1 generator point (G) — same fixture as
/// `wallet_policy.rs::valid_compressed_pubkey`.
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

/// 65-byte xpub (32-byte chain-code fill || 33-byte compressed pubkey).
/// `encode_payload` never validates xpub-bytes shape (only `decode_payload`
/// does, via `validate_xpub_bytes`) — this fixture is structurally valid
/// anyway, mirroring `wallet_policy.rs::make_xpub`.
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

fn wsh_multi_2of2() -> Node {
    Node {
        tag: Tag::Wsh,
        body: Body::Children(vec![Node {
            tag: Tag::Multi,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1],
            },
        }]),
    }
}

fn wsh_sortedmulti_2of8() -> Node {
    Node {
        tag: Tag::Wsh,
        body: Body::Children(vec![Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2, 3, 4, 5, 6, 7],
            },
        }]),
    }
}

// ─── (b) wallet-policy / embedded-pubkey ──────────────────────────────────

/// `wpkh(@0/<0;1>/*)` with BIP-84 origin + a populated `Pubkeys` TLV on
/// `@0` — the minimal shape that emits `TLV_PUBKEYS = 0x02`
/// (`tlv.rs:149-172`) and nothing else (no fingerprints TLV, isolating the
/// pubkeys-tag emission).
fn wallet_policy_wpkh_golden() -> Descriptor {
    let mut tlv = TlvSection::new_empty();
    tlv.pubkeys = Some(vec![(0u8, make_xpub(0x11))]);
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: wpkh_at_0(),
        tlv,
    }
}

/// Frozen `encode_payload` output for [`wallet_policy_wpkh_golden`].
///
/// Provenance: `md-codec 0.41.0`, git `b9662e5f953cb7349b02d8b95f6c0925c021650b`
/// (descriptor-mnemonic `main`), captured via
/// `encode_payload(&wallet_policy_wpkh_golden())` → `hex::encode(&bytes)`.
const WALLET_POLICY_WPKH_BYTES_HEX: &str = concat!(
    "200ef5210800600550408888888888888888888888888888888888888888888888888888888",
    "88888888813cdf333f7cee5dd62ad0314ae7438583814dfe6d96e7146cacf940ad8b7c0bcc0"
);
const WALLET_POLICY_WPKH_TOTAL_BITS: usize = 597;

// ─── (c) use-site path override ───────────────────────────────────────────

/// `wsh(multi(2, @0/<0;1>/*, @1/<2;3>/*))` — BIP-48-type-2 shared origin,
/// standard baseline use-site path, a per-`@1` `use_site_path_overrides`
/// entry diverging to `<2;3>/*`. Emits `TLV_USE_SITE_PATH_OVERRIDES = 0x00`
/// (`tlv.rs:99-122`).
fn use_site_override_golden() -> Descriptor {
    let mut tlv = TlvSection::new_empty();
    tlv.use_site_path_overrides = Some(vec![(
        1u8,
        UseSitePath {
            multipath: Some(vec![
                Alternative {
                    hardened: false,
                    value: 2,
                },
                Alternative {
                    hardened: false,
                    value: 3,
                },
            ]),
            wildcard_hardened: false,
        },
    )]);
    Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Shared(bip48_type_2_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: wsh_multi_2of2(),
        tlv,
    }
}

/// Frozen `encode_payload` output for [`use_site_override_golden`].
///
/// Provenance: `md-codec 0.41.0`, git `b9662e5f953cb7349b02d8b95f6c0925c021650b`,
/// captured via `encode_payload(&use_site_override_golden())` → `hex::encode`.
const USE_SITE_OVERRIDE_BYTES_HEX: &str = "2052d84212a00182182140b4c0a160";
const USE_SITE_OVERRIDE_TOTAL_BITS: usize = 116;

// ─── (d) origin override (`Divergent`) ────────────────────────────────────

/// `wsh(multi(2, @0, @1))` with `path_decl.paths = Divergent([bip48_type_2,
/// bip84])` — two DISTINCT per-key origin paths. `encode_payload` auto-sets
/// the header divergent-paths bit (`encode.rs:113-117`); `PathDecl::write`
/// emits each of the `n` `OriginPath`s in order (`origin_path.rs:114-131`).
/// No TLV populated — isolates the `Divergent` path-order wire shape from
/// any TLV-section noise.
fn origin_override_golden() -> Descriptor {
    Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Divergent(vec![bip48_type_2_path(), bip84_path()]),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: wsh_multi_2of2(),
        tlv: TlvSection::new_empty(),
    }
}

/// Frozen `encode_payload` output for [`origin_override_golden`].
///
/// Provenance: `md-codec 0.41.0`, git `b9662e5f953cb7349b02d8b95f6c0925c021650b`,
/// captured via `encode_payload(&origin_override_golden())` → `hex::encode`.
const ORIGIN_OVERRIDE_BYTES_HEX: &str = "a052d842128ef521080060860850";
const ORIGIN_OVERRIDE_TOTAL_BITS: usize = 108;

// ─── (e) 2-chunk framing golden ───────────────────────────────────────────

/// Exact `Descriptor` shape of the committed
/// `tests/vectors/wsh_sortedmulti_2chunk.descriptor.json` corpus fixture:
/// `n=8`, `Shared` path_decl with an ELIDED (empty-components) origin,
/// standard `<0;1>/*` use-site path, `wsh(sortedmulti(2, @0..@7))`, 8
/// fingerprints, `pubkeys: null`. Hand-built here (not read from the JSON
/// file) so this golden is independent of the corpus file changing shape;
/// see `mnemonic-toolkit:design/RECON_T3b_api_feasibility.md` §"Chunk-strings
/// reproducibility" for the field-by-field trace confirming this is the
/// SAME descriptor the MANIFEST entry resolves to.
fn wsh_sortedmulti_2chunk_golden() -> Descriptor {
    let mut tlv = TlvSection::new_empty();
    tlv.fingerprints = Some(vec![
        (0u8, [0x01, 0x02, 0x03, 0x04]),
        (1u8, [0x02, 0x03, 0x04, 0x05]),
        (2u8, [0x03, 0x04, 0x05, 0x06]),
        (3u8, [0x04, 0x05, 0x06, 0x07]),
        (4u8, [0x05, 0x06, 0x07, 0x08]),
        (5u8, [0x06, 0x07, 0x08, 0x09]),
        (6u8, [0x07, 0x08, 0x09, 0x0a]),
        (7u8, [0x08, 0x09, 0x0a, 0x0b]),
    ]);
    Descriptor {
        n: 8,
        path_decl: PathDecl {
            n: 8,
            paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: wsh_sortedmulti_2of8(),
        tlv,
    }
}

/// Frozen `md_codec::chunk::split` output for
/// [`wsh_sortedmulti_2chunk_golden`] — byte-identical to the committed
/// `tests/vectors/wsh_sortedmulti_2chunk.phrase.txt` corpus fixture.
///
/// Provenance: `md-codec 0.41.0`, git `b9662e5f953cb7349b02d8b95f6c0925c021650b`,
/// captured via `md_codec::chunk::split(&wsh_sortedmulti_2chunk_golden())`.
/// `split` is pure `BitWriter` bit-packing (no RNG) — pinned deterministic
/// by `wallet_policy.rs`'s `multi_chunk_2of3_cell_7_split_reassemble_round_trip`
/// and `encoder_determinism_2of3_cell_7_byte_identical_emit` double-call
/// equality checks.
const WSH_SORTEDMULTI_2CHUNK_CHUNK_0: &str =
    "md1fujf0qsppcgqpsgwzwpfewuxvvqqgzqvzzqsrqsz5qcyq5srf0tn67rd3va";
const WSH_SORTEDMULTI_2CHUNK_CHUNK_1: &str =
    "md1fujf0qsgvcyq5rq0q9qcrs3gxquyqns8pqys4cgpy9qkqt2cxjz4rtdyjk";

// ─── Tests ─────────────────────────────────────────────────────────────

#[test]
fn wallet_policy_wpkh_bytes_hex_frozen() {
    let d = wallet_policy_wpkh_golden();
    let (bytes, total_bits) = encode_payload(&d).expect("encode wallet-policy wpkh golden");
    assert_eq!(
        hex::encode(&bytes),
        WALLET_POLICY_WPKH_BYTES_HEX,
        "wallet-policy wpkh wire bytes drifted from the frozen historical golden"
    );
    assert_eq!(total_bits, WALLET_POLICY_WPKH_TOTAL_BITS);
}

#[test]
fn use_site_override_bytes_hex_frozen() {
    let d = use_site_override_golden();
    let (bytes, total_bits) = encode_payload(&d).expect("encode use-site-override golden");
    assert_eq!(
        hex::encode(&bytes),
        USE_SITE_OVERRIDE_BYTES_HEX,
        "use-site-override wire bytes drifted from the frozen historical golden"
    );
    assert_eq!(total_bits, USE_SITE_OVERRIDE_TOTAL_BITS);
}

#[test]
fn origin_override_bytes_hex_frozen() {
    let d = origin_override_golden();
    let (bytes, total_bits) = encode_payload(&d).expect("encode origin-override golden");
    assert_eq!(
        hex::encode(&bytes),
        ORIGIN_OVERRIDE_BYTES_HEX,
        "origin-override (Divergent) wire bytes drifted from the frozen historical golden"
    );
    assert_eq!(total_bits, ORIGIN_OVERRIDE_TOTAL_BITS);
}

#[test]
fn wsh_sortedmulti_2chunk_chunk_strings_frozen() {
    let d = wsh_sortedmulti_2chunk_golden();
    let chunks = md_codec::chunk::split(&d).expect("split wsh_sortedmulti_2chunk golden");
    assert_eq!(
        chunks,
        vec![
            WSH_SORTEDMULTI_2CHUNK_CHUNK_0.to_string(),
            WSH_SORTEDMULTI_2CHUNK_CHUNK_1.to_string(),
        ],
        "2-chunk split STRINGS drifted from the frozen historical golden \
         (this closes the D1 chunk-split-boundary mutation gap — a \
         .bytes.hex-only golden cannot RED a split-boundary regression \
         because split/reassemble round-trips regardless of where the \
         boundary falls)"
    );
}
