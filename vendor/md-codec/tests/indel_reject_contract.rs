//! Theme 3 — indel reject-contract via reassemble (hard verify, no self-correct).
//! The toolkit's Md1IndelOracle (mnemonic-toolkit .../src/repair.rs) relies on
//! reassemble failing closed on a length-changed string — a fail-OPEN here breaks
//! the `repair --md1 --max-indel` oracle.
use md_codec::chunk::{reassemble, split};
use md_codec::encode::Descriptor;
use md_codec::error::Error;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;

fn fixture() -> Descriptor {
    let paths = (0..4u32)
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

// T3a — insert one symbol mid-data-part → Err (fail-closed; hard verify).
#[test]
fn t3a_insert_rejected() {
    let chunks = split(&fixture()).unwrap();
    let mut chars: Vec<char> = chunks[0].chars().collect();
    chars.insert(3 + 10, 'p'); // mid-data-part insert
    let mut cs = chunks.clone();
    cs[0] = chars.into_iter().collect();
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    assert!(
        reassemble(&refs).is_err(),
        "an inserted symbol must fail closed"
    );
}

// T3b — delete one symbol mid-data-part → Err.
#[test]
fn t3b_delete_rejected() {
    let chunks = split(&fixture()).unwrap();
    let mut chars: Vec<char> = chunks[0].chars().collect();
    chars.remove(3 + 10);
    let mut cs = chunks.clone();
    cs[0] = chars.into_iter().collect();
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    assert!(
        reassemble(&refs).is_err(),
        "a deleted symbol must fail closed"
    );
}

// T3c — truncate below the 13-symbol checksum → Codex32DecodeError (broad pin:
// bch_verify fails before the "too short" message).
#[test]
fn t3c_truncate_below_checksum_is_codex32_error() {
    let chunks = split(&fixture()).unwrap();
    let chars: Vec<char> = chunks[0].chars().collect();
    // keep only "md1" + 5 data symbols (< 13 checksum symbols)
    let trimmed: String = chars[..3 + 5].iter().collect();
    assert!(matches!(
        reassemble(&[trimmed.as_str()]),
        Err(Error::Codex32DecodeError(_))
    ));
}

// T3d — multi-chunk indel never yields a different valid descriptor (the oracle
// guarantee) + the is_err tripwire.
#[test]
fn t3d_multi_chunk_indel_fails_closed() {
    let d = fixture();
    let chunks = split(&d).unwrap();
    let mut chars: Vec<char> = chunks[0].chars().collect();
    chars.insert(3 + 8, 'q');
    let mut cs = chunks.clone();
    cs[0] = chars.into_iter().collect();
    let refs: Vec<&str> = cs.iter().map(String::as_str).collect();
    let r = reassemble(&refs);
    assert!(r.is_err(), "multi-chunk indel must fail closed");
    assert_ne!(
        r.ok(),
        Some(d),
        "indel must never self-correct to the original"
    );
}
