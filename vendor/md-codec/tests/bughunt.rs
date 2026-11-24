//! Bug-hunt triage rows (md-codec). Each test asserts the CORRECT behaviour,
//! so it FAILS on unmodified code when the reported bug is real.
//! Hunt report: design/agent-reports/bughunt-2026-10-31-md.md.

use bitcoin::Network;
use md_codec::bitstream::{BitReader, BitWriter};
use md_codec::encode::Descriptor;
use md_codec::error::Error;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};

fn shared_path() -> PathDeclPaths {
    PathDeclPaths::Shared(OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 48,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
        ],
    })
}

fn node(tag: Tag, body: Body) -> Node {
    Node { tag, body }
}

fn key(tag: Tag, index: u8) -> Node {
    node(tag, Body::KeyArg { index })
}

fn desc(n: u8, tree: Node, tlv: TlvSection) -> Descriptor {
    Descriptor {
        n,
        path_decl: PathDecl {
            n,
            paths: shared_path(),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv,
    }
}

/// wpkh(@0): the simplest card.
fn wpkh_card() -> Descriptor {
    desc(
        1,
        node(Tag::Wpkh, Body::KeyArg { index: 0 }),
        TlvSection::new_empty(),
    )
}

/// #2. `wsh(thresh(2,pk(@0),s:pk(@1),sln:older(210000)))`: 210000 has bits above
/// BIP-68's 16-bit value + type bit, so consensus enforces 210000 & 0xFFFF =
/// 13392. The timelock check must reach into `thresh` (a `Body::Variable`).
/// Control: the same `older` directly under `wsh(and_v(..))` is refused.
#[test]
fn bughunt_md_02_thresh_older_truncation_checked() {
    let older = node(Tag::Older, Body::Timelock(210_000));
    let direct = node(
        Tag::Wsh,
        Body::Children(vec![node(
            Tag::AndV,
            Body::Children(vec![key(Tag::PkK, 0), older.clone()]),
        )]),
    );
    assert!(
        md_codec::validate::validate_relative_timelocks(&direct).is_err(),
        "control: a bare older(210000) must be refused"
    );
    let nested = node(
        Tag::Wsh,
        Body::Children(vec![node(
            Tag::Thresh,
            Body::Variable {
                k: 2,
                children: vec![key(Tag::PkK, 0), key(Tag::PkK, 1), older],
            },
        )]),
    );
    let r = md_codec::validate::validate_relative_timelocks(&nested);
    assert!(
        matches!(r, Err(md_codec::Error::RelativeTimelockTruncated { .. })),
        "older(210000) inside thresh must be RelativeTimelockTruncated, got {r:?}"
    );
}

/// A valid 65-byte Pubkeys TLV payload: a chain code of 0x01 bytes followed by
/// the secp256k1 generator G in compressed form.
fn pubkey_bytes() -> [u8; 65] {
    let mut out = [1u8; 65];
    let g =
        hex::decode("0279BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798").unwrap();
    out[32..].copy_from_slice(&g);
    out
}

/// wpkh(@0) carrying a pubkey for @0, with the given origin path and
/// use-site multipath.
fn wpkh_with_key(origin: OriginPath, alts: Vec<Alternative>) -> Descriptor {
    let mut tlv = TlvSection::new_empty();
    tlv.pubkeys = Some(vec![(0, pubkey_bytes())]);
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin),
        },
        use_site_path: UseSitePath {
            multipath: Some(alts),
            wildcard_hardened: false,
        },
        tree: node(Tag::Wpkh, Body::KeyArg { index: 0 }),
        tlv,
    }
}

/// Every public entry that builds a miniscript descriptor or an address
/// returns `want` for `d`.
fn assert_all_entries_refuse(d: &Descriptor, want: impl Fn() -> Error) {
    assert_eq!(
        md_codec::to_miniscript::to_miniscript_descriptor(d, 0).map(|x| x.to_string()),
        Err(want()),
        "to_miniscript_descriptor(chain 0)"
    );
    assert_eq!(
        md_codec::to_miniscript::to_miniscript_descriptor_multipath(d).map(|x| x.to_string()),
        Err(want()),
        "to_miniscript_descriptor_multipath"
    );
    assert_eq!(
        d.derive_address(0, 0, Network::Bitcoin)
            .map(|a| a.assume_checked().to_string()),
        Err(want()),
        "derive_address(chain 0, index 0)"
    );
}

/// #4 codec half (ruling R-4). A library-built Descriptor whose use-site
/// multipath carries a non-hardened alternative of 2^31 (not representable
/// as an md1 varint, BIP mediawiki:459) must be refused with VarintOverflow,
/// not silently turned into another child number. Chain 0 selects the 2^31
/// alternative.
#[test]
fn bughunt_md_04b_codec_to_miniscript_alt_2_31_varint_overflow() {
    let d = wpkh_with_key(
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
        },
        vec![
            Alternative {
                hardened: false,
                value: 2_147_483_648,
            },
            Alternative {
                hardened: false,
                value: 1,
            },
        ],
    );
    assert_all_entries_refuse(&d, || Error::VarintOverflow {
        value: 2_147_483_648,
    });
}

/// #4 codec half, origin side (ruling R-4: "any origin ... value"). An
/// origin component of 2^31 (hardened flag set) must be refused with
/// VarintOverflow, not silently turned into another child number.
#[test]
fn bughunt_md_04c_codec_to_miniscript_origin_2_31_varint_overflow() {
    let d = wpkh_with_key(
        OriginPath {
            components: vec![
                PathComponent {
                    hardened: true,
                    value: 84,
                },
                PathComponent {
                    hardened: true,
                    value: 2_147_483_648,
                },
            ],
        },
        vec![
            Alternative {
                hardened: false,
                value: 0,
            },
            Alternative {
                hardened: false,
                value: 1,
            },
        ],
    );
    assert_all_entries_refuse(&d, || Error::VarintOverflow {
        value: 2_147_483_648,
    });
}

/// #4 codec half, unselected alternative (architect ruling Q1/Q2). Chain 0
/// selects alt 0, so the alt of 2^29 that overflows is not the selected one:
/// every entry still refuses it with VarintOverflow. Control: 2^29 - 1 in the
/// same place derives on every entry.
#[test]
fn bughunt_md_04f_codec_unselected_alt_above_2_29_varint_overflow() {
    let origin = || OriginPath {
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
    };
    let alts = |second: u32| {
        vec![
            Alternative {
                hardened: false,
                value: 0,
            },
            Alternative {
                hardened: false,
                value: second,
            },
        ]
    };
    let d = wpkh_with_key(origin(), alts(536_870_912));
    assert_all_entries_refuse(&d, || Error::VarintOverflow { value: 536_870_912 });
    let c = wpkh_with_key(origin(), alts(536_870_911));
    assert!(
        md_codec::to_miniscript::to_miniscript_descriptor(&c, 0).is_ok(),
        "control: to_miniscript_descriptor"
    );
    assert!(
        md_codec::to_miniscript::to_miniscript_descriptor_multipath(&c).is_ok(),
        "control: to_miniscript_descriptor_multipath"
    );
    assert!(
        c.derive_address(0, 0, Network::Bitcoin).is_ok(),
        "control: derive_address"
    );
}

/// #5 (ruling R-5). `wsh(and_v(v:pk(@0),or_d(pk(@1),older(10))))`: an `or_d`
/// nested inside an `and_v` branch is a tag `collect` must refuse, so the
/// whole summary is marked incomplete (no branch is shown). Top-level
/// splitting is unchanged; no cross product is built.
#[test]
fn bughunt_md_05_nested_or_under_and_marks_incomplete() {
    let d = desc(
        2,
        node(
            Tag::Wsh,
            Body::Children(vec![node(
                Tag::AndV,
                Body::Children(vec![
                    node(Tag::Verify, Body::Children(vec![key(Tag::PkK, 0)])),
                    node(
                        Tag::OrD,
                        Body::Children(vec![
                            key(Tag::PkK, 1),
                            node(Tag::Older, Body::Timelock(10)),
                        ]),
                    ),
                ]),
            )]),
        ),
        TlvSection::new_empty(),
    );
    let shape = md_codec::policy_shape::policy_shape(&d);
    assert!(
        !shape.complete,
        "an or_d under and_v must mark the summary incomplete: {shape:?}"
    );
    assert!(
        shape.branches.is_empty(),
        "an incomplete summary shows no branch: {shape:?}"
    );
}

/// #7. n=2, tree multi(@1,@0), fingerprints [(5, fp)]: index 5 is out of range
/// for n=2. Must be a typed error (or Ok), never a panic.
#[test]
fn bughunt_md_07_canonicalize_out_of_range_tlv_index_no_panic() {
    let mut tlv = TlvSection::new_empty();
    tlv.fingerprints = Some(vec![(5u8, [1, 2, 3, 4])]);
    let mut d = desc(
        2,
        node(
            Tag::Wsh,
            Body::Children(vec![node(
                Tag::Multi,
                Body::MultiKeys {
                    k: 2,
                    indices: vec![1, 0],
                },
            )]),
        ),
        tlv,
    );
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        md_codec::canonicalize::canonicalize_placeholder_indices(&mut d)
    }));
    assert!(
        r.is_ok(),
        "canonicalize panicked on an out-of-range TLV index"
    );
    assert_eq!(
        r.unwrap(),
        Err(Error::PlaceholderIndexOutOfRange { idx: 5, n: 2 }),
        "expected the typed out-of-range error for index 5 with n=2"
    );
}

/// #8 (ruling R-8). `TlvSection::write` refuses an unknown entry it cannot
/// represent faithfully before writing anything: a tag outside the reserved
/// range 0x04..=0x1f, or a duplicate unknown tag.
fn tlv_write_err(unknown: Vec<(u8, Vec<u8>, usize)>) -> (Result<(), Error>, usize) {
    let mut t = TlvSection::new_empty();
    t.unknown = unknown;
    let mut w = BitWriter::new();
    let r = t.write(&mut w, 1);
    (r, w.bit_len())
}

#[test]
fn bughunt_md_08_unknown_tlv_tag_above_31_not_silently_truncated() {
    // 0x21 = 33; the tag field is 5 bits, so today it is read back as 1
    // (fingerprints).
    let (r, bits) = tlv_write_err(vec![(0x21, vec![0xAB], 8)]);
    assert_eq!(
        r.map_err(|e| e.to_string()),
        Err("unknown TLV tag 0x21 is outside the reserved range 0x04..=0x1f".to_string())
    );
    assert_eq!(bits, 0, "the refusal comes before anything is written");
}

#[test]
fn bughunt_md_08b_unknown_tlv_tag_duplicating_known_tag_not_written() {
    // Tag 1 (fingerprints) as an "unknown" entry.
    let (r, bits) = tlv_write_err(vec![(1, vec![0xAB], 8)]);
    assert_eq!(
        r.map_err(|e| e.to_string()),
        Err("unknown TLV tag 0x01 is outside the reserved range 0x04..=0x1f".to_string())
    );
    assert_eq!(bits, 0, "the refusal comes before anything is written");
}

#[test]
fn bughunt_md_08c_duplicate_unknown_tlv_tag_refused() {
    // Tag 5 twice: in range, but BIP mediawiki:639/:654 reject duplicates.
    let (r, bits) = tlv_write_err(vec![(5, vec![0xAB], 8), (5, vec![0xCD], 8)]);
    assert_eq!(
        r,
        Err(Error::TlvOrderingViolation {
            prev: 5,
            current: 5
        })
    );
    assert_eq!(bits, 0, "the refusal comes before anything is written");
}

/// #9 (withdrawn, ruling R-9). LP4-ext extension form carrying 5 in 22 bits
/// (canonical is L=3, 3 bits). BIP mediawiki:752 makes decoders lenient, so
/// the non-canonical form is accepted and reads as 5. Passes on unmodified
/// code; it pins the BIP's lenient-decoder rule.
#[test]
fn bughunt_md_09_varint_noncanonical_extension_accepted_per_bip() {
    let mut w = BitWriter::new();
    w.write_bits(15, 4); // L = 15: extension
    w.write_bits(8, 4); // l_high = 8
    w.write_bits(5, 14); // payload_low = 5
    w.write_bits(0, 8); // payload_high = 0 (8 bits)
    let bits = w.bit_len();
    let bytes = w.into_bytes();
    let mut r = BitReader::with_bit_limit(&bytes, bits);
    let v = md_codec::varint::read_varint(&mut r);
    assert_eq!(v, Ok(5));
}

/// #16 (lead L16, ruling R-16). `c as u8` truncates a non-ASCII char to its
/// low byte, so U+01xx (xx = a real codex32 symbol) must NOT decode as symbol
/// xx. It reaches the existing invalid-character error, word for word the
/// text an ASCII non-alphabet char (`'b'`) gets today.
fn swap_first_data_char_with(s: &str, f: impl Fn(char) -> char) -> (String, char) {
    let (head, rest) = s.split_at(3); // "md1"
    let mut it = rest.chars();
    let c = f(it.next().unwrap());
    (format!("{head}{c}{}", it.as_str()), c)
}

fn alias(c: char) -> char {
    char::from_u32(0x100 + c as u32).unwrap()
}

#[test]
fn bughunt_md_16_single_string_non_ascii_alias_refused() {
    let s = md_codec::encode::encode_md1_string(&wpkh_card()).unwrap();
    let (bad, a) = swap_first_data_char_with(&s, alias);
    assert_ne!(bad, s);
    let r = md_codec::decode::decode_md1_string(&bad).map_err(|e| e.to_string());
    assert_eq!(
        r.err(),
        Some(format!(
            "codex32 decode error: character {a:?} not in codex32 alphabet"
        )),
        "non-ASCII alias {bad:?} must be refused as an invalid character"
    );
    // Control: 'b' (ASCII, not in the alphabet) gets the same wording today.
    let (b_bad, _) = swap_first_data_char_with(&s, |_| 'b');
    let rb = md_codec::decode::decode_md1_string(&b_bad).map_err(|e| e.to_string());
    assert_eq!(
        rb.err(),
        Some("codex32 decode error: character 'b' not in codex32 alphabet".to_string())
    );
}

/// Replace chunk 0 of the wpkh card's chunk set with `first`.
fn chunks_with_first(first: String) -> Vec<String> {
    let mut v = md_codec::chunk::split(&wpkh_card()).unwrap();
    v[0] = first;
    v
}

#[test]
fn bughunt_md_16b_chunk_non_ascii_alias_refused() {
    // `reassemble` reads each chunk through the codex32 string decoder, so
    // the text is the single-string one (no chunk prefix).
    let chunk0 = md_codec::chunk::split(&wpkh_card()).unwrap()[0].clone();
    let reassemble_err = |first: String| {
        let v = chunks_with_first(first);
        let refs: Vec<&str> = v.iter().map(String::as_str).collect();
        md_codec::chunk::reassemble(&refs)
            .map(|_| ())
            .map_err(|e| e.to_string())
            .err()
    };
    let (bad0, a) = swap_first_data_char_with(&chunk0, alias);
    assert_ne!(bad0, chunk0);
    assert_eq!(
        reassemble_err(bad0.clone()),
        Some(format!(
            "codex32 decode error: character {a:?} not in codex32 alphabet"
        )),
        "non-ASCII alias in a chunk must be refused as an invalid character: {bad0:?}"
    );
    // Control: 'b' gets the same wording today.
    let (b0, _) = swap_first_data_char_with(&chunk0, |_| 'b');
    assert_eq!(
        reassemble_err(b0),
        Some("codex32 decode error: character 'b' not in codex32 alphabet".to_string())
    );
}

/// #16 on the repair path. `correct_chunks` reads symbols itself
/// (`parse_chunk_symbols`), with a `chunk N:` prefix on its refusal.
#[test]
fn bughunt_md_16c_correct_chunks_non_ascii_alias_refused() {
    let chunk0 = md_codec::chunk::split(&wpkh_card()).unwrap()[0].clone();
    let correct_err = |first: String| {
        let v = chunks_with_first(first);
        let refs: Vec<&str> = v.iter().map(String::as_str).collect();
        md_codec::chunk::correct_chunks(&refs)
            .map(|_| ())
            .map_err(|e| e.to_string())
            .err()
    };
    let (bad0, a) = swap_first_data_char_with(&chunk0, alias);
    assert_ne!(bad0, chunk0);
    assert_eq!(
        correct_err(bad0.clone()),
        Some(format!(
            "codex32 decode error: chunk 0: character {a:?} not in codex32 alphabet"
        )),
        "non-ASCII alias must be refused, not corrected or aliased: {bad0:?}"
    );
    // Control: 'b' gets the same chunk wording today.
    let (b0, _) = swap_first_data_char_with(&chunk0, |_| 'b');
    assert_eq!(
        correct_err(b0),
        Some("codex32 decode error: chunk 0: character 'b' not in codex32 alphabet".to_string())
    );
}

/// #5, every disjunction tag R-5 lists (not only `or_d`).
#[test]
fn bughunt_md_05b_every_nested_disjunction_tag_marks_incomplete() {
    for tag in [Tag::AndOr, Tag::OrB, Tag::OrC, Tag::OrD, Tag::OrI] {
        let kids = if tag == Tag::AndOr {
            vec![key(Tag::PkK, 1), key(Tag::PkK, 2), key(Tag::PkK, 3)]
        } else {
            vec![key(Tag::PkK, 1), key(Tag::PkK, 2)]
        };
        let d = desc(
            4,
            node(
                Tag::Wsh,
                Body::Children(vec![node(
                    Tag::AndV,
                    Body::Children(vec![
                        node(Tag::Verify, Body::Children(vec![key(Tag::PkK, 0)])),
                        node(tag, Body::Children(kids)),
                    ]),
                )]),
            ),
            TlvSection::new_empty(),
        );
        let shape = md_codec::policy_shape::policy_shape(&d);
        assert!(
            !shape.complete && shape.branches.is_empty(),
            "{tag:?} under and_v must mark the summary incomplete: {shape:?}"
        );
    }
}

type TlvSetter = Box<dyn Fn(&mut TlvSection)>;

/// #7, boundary and all four lists: idx == n (the first out-of-range value)
/// in each TLV entry list is the typed error, never a panic.
#[test]
fn bughunt_md_07b_every_tlv_list_idx_equal_n_refused() {
    let usp = UseSitePath {
        multipath: None,
        wildcard_hardened: false,
    };
    let op = OriginPath {
        components: vec![PathComponent {
            hardened: true,
            value: 84,
        }],
    };
    let cases: Vec<(&str, TlvSetter)> = vec![
        (
            "use_site_path_overrides",
            Box::new(move |t| t.use_site_path_overrides = Some(vec![(2, usp.clone())])),
        ),
        (
            "fingerprints",
            Box::new(|t| t.fingerprints = Some(vec![(2, [1, 2, 3, 4])])),
        ),
        (
            "pubkeys",
            Box::new(|t| t.pubkeys = Some(vec![(2, [7u8; 65])])),
        ),
        (
            "origin_path_overrides",
            Box::new(move |t| t.origin_path_overrides = Some(vec![(2, op.clone())])),
        ),
    ];
    for (name, set) in cases {
        let mut tlv = TlvSection::new_empty();
        set(&mut tlv);
        let mut d = desc(
            2,
            node(
                Tag::Wsh,
                Body::Children(vec![node(
                    Tag::Multi,
                    Body::MultiKeys {
                        k: 2,
                        indices: vec![1, 0],
                    },
                )]),
            ),
            tlv,
        );
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            md_codec::canonicalize::canonicalize_placeholder_indices(&mut d)
        }));
        assert_eq!(
            r.ok(),
            Some(Err(Error::PlaceholderIndexOutOfRange { idx: 2, n: 2 })),
            "{name}: idx == n"
        );
    }
}

/// #8 boundaries: tag 0x03 (a known tag) and 0x20 are refused; 0x04 and 0x1f
/// (the ends of the reserved range) are written.
#[test]
fn bughunt_md_08d_unknown_tlv_tag_range_boundaries() {
    for tag in [0x00u8, 0x03, 0x20] {
        let (r, bits) = tlv_write_err(vec![(tag, vec![0xAB], 8)]);
        assert_eq!(
            r,
            Err(Error::TlvUnknownTagOutOfRange { tag }),
            "tag {tag:#x}"
        );
        assert_eq!(bits, 0);
    }
    for tag in [0x04u8, 0x1f] {
        let (r, _) = tlv_write_err(vec![(tag, vec![0xAB], 8)]);
        assert_eq!(r, Ok(()), "tag {tag:#x}");
    }
}
