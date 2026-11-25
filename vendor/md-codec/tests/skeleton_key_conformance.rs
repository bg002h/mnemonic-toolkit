//! The conformance gate -- one key, two routes (coordinator-compat plan 1a
//! Task 5, re-homed by plan 1b).
//!
//! `chunks -> key == descriptor -> key` for every vendored keyed vector
//! (`crates/md-codec/tests/vectors/keyed_*.conformance.json`, 48 of them),
//! proving `SkeletonKey` is a property of the POLICY rather than of the door
//! it was read through.
//!
//! * **The chunk route** decodes the REAL md1 wire chunk set vendored beside
//!   each record (`<name>.phrase.txt`) via [`md_codec::chunk::reassemble`].
//! * **The descriptor route** is now the LIBRARY route,
//!   [`md_codec::descriptor_route::descriptor_from_chains`] -- plan 1b
//!   promoted this file's private walker into md-codec so the verdict
//!   table's generator and `md shape-key` share it. It self-checks every
//!   reconstruction by re-rendering both chains byte-for-byte; see that
//!   module's doc for what the round trip does and does not certify.
//!
//! Both routes feed the ONE [`skeleton`]/[`skeleton_key`] implementation.
//! `tests/coordinator.rs` runs the same gate over every vendored
//! EVIDENCE row, which this corpus does not contain (recon: 0 of 11 Liana
//! live-gate rows, 1 of 56 matrix shapes).

mod common;

use std::path::{Path, PathBuf};

use md_codec::chunk::reassemble;
use md_codec::descriptor_route::descriptor_from_chains;
use md_codec::encode::Descriptor as MdDescriptor;
use md_codec::skeleton::{skeleton, skeleton_key};
use md_codec::tag::Tag;
use md_codec::to_miniscript_descriptor;
use md_codec::tree::{Body, Node};

// ─────────────────────────────────────────────────────────────────────────
// Vector loading. A glob that matches nothing must FAIL the floor
// assertion below, not silently pass -- so `list_keyed_conformance_files`
// returns an EMPTY Vec on a missing directory (never panics), leaving the
// final `checked >= 48` assertion as the one thing that can catch it.
// ─────────────────────────────────────────────────────────────────────────

fn conformance_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/vectors"))
}

fn list_keyed_conformance_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("keyed_") && n.ends_with(".conformance.json"))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    out.sort();
    out
}

/// One vendored keyed conformance record: the fields this test reads.
struct Rec {
    name: String,
    /// The chunk-set's individual md1 wire strings, header line stripped.
    phrase_chunks: Vec<String>,
    /// `chains.0.descriptor` -- real keys, chain-0 use site.
    descriptor0: String,
    /// `chains.1.descriptor` -- the SAME keys at chain-1, used only to
    /// self-check the reconstructed use-site path (see module doc).
    descriptor1: String,
}

fn load(json_path: &Path) -> Rec {
    let text = std::fs::read_to_string(json_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", json_path.display()));
    let v: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("parse {}: {e}", json_path.display()));
    let name = v["name"]
        .as_str()
        .unwrap_or_else(|| panic!("{}: missing .name", json_path.display()))
        .to_string();
    let descriptor0 = v["chains"]["0"]["descriptor"]
        .as_str()
        .unwrap_or_else(|| panic!("{name}: missing chains.0.descriptor"))
        .to_string();
    let descriptor1 = v["chains"]["1"]["descriptor"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("{name}: missing chains.1.descriptor -- every keyed vector uses <0;1>/*")
        })
        .to_string();

    let phrase_path = json_path.with_file_name(format!("{name}.phrase.txt"));
    let phrase_text = std::fs::read_to_string(&phrase_path)
        .unwrap_or_else(|e| panic!("{name}: read {}: {e}", phrase_path.display()));
    let mut lines = phrase_text.lines();
    let header = lines
        .next()
        .unwrap_or_else(|| panic!("{name}: empty phrase file"));
    assert!(
        header.starts_with("chunk-set-id:"),
        "{name}: expected a chunk-set header (every keyed vector is force_chunked), got {header:?}"
    );
    let phrase_chunks: Vec<String> = lines
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    assert!(
        !phrase_chunks.is_empty(),
        "{name}: no chunk lines in {}",
        phrase_path.display()
    );

    Rec {
        name,
        phrase_chunks,
        descriptor0,
        descriptor1,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// The chunk route: the real wire, via the crate's own reassembler.
// ─────────────────────────────────────────────────────────────────────────

fn decode_chunks(chunks: &[String]) -> MdDescriptor {
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    reassemble(&refs).unwrap_or_else(|e| panic!("reassemble: {e}"))
}

// ─────────────────────────────────────────────────────────────────────────
// The gate.
// ─────────────────────────────────────────────────────────────────────────

/// Every vendored keyed vector, keyed twice: once from the md1 chunk set
/// and once from the descriptor the same record carries. The key must not
/// know which door it came through.
#[test]
fn chunks_and_descriptor_yield_the_same_key() {
    let mut checked = 0usize;
    for path in list_keyed_conformance_files(&conformance_dir()) {
        let rec = load(&path);

        let d_chunks = decode_chunks(&rec.phrase_chunks);
        let from_chunks = skeleton_key(
            &skeleton(&d_chunks)
                .unwrap_or_else(|e| panic!("{}: skeleton (chunk route): {e}", rec.name)),
        );

        let d_desc = descriptor_from_chains(&rec.descriptor0, &rec.descriptor1)
            .unwrap_or_else(|e| panic!("{}: descriptor route: {e}", rec.name));
        let from_desc = skeleton_key(
            &skeleton(&d_desc)
                .unwrap_or_else(|e| panic!("{}: skeleton (descriptor route): {e}", rec.name)),
        );

        assert_eq!(
            from_chunks, from_desc,
            "{}: the key knows its route",
            rec.name
        );
        checked += 1;
    }
    assert!(
        checked >= 48,
        "only {checked} vectors keyed -- the gate is checking almost nothing"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Review round 1, I-1: direct unit coverage for the eleven walker arms the
// vector corpus alone never reaches. See `md_codec::descriptor_route`'s
// module doc for what the round trip does NOT certify.
// ─────────────────────────────────────────────────────────────────────────

/// A descriptor built from `tests/common/mod.rs`'s `descriptor_with_pubkeys`
/// (real xpubs, no fingerprints) with a fingerprint TLV entry ADDED for every
/// slot. `descriptor_with_pubkeys` alone renders keys with no `[origin]`
/// bracket at all (`to_miniscript.rs::assemble_origin_and_xkey`'s `origin`
/// field is `e.fingerprint.map(...)` -- `None` fingerprint means no bracket,
/// regardless of the divergent origin PATH `descriptor_with_pubkeys` already
/// sets), and `descriptor_from_chains` refuses a key with no bracket outright
/// (`RouteError::KeyWithoutOrigin`). The value is arbitrary -- these fixtures
/// exist to exercise `Tag` mappings, not to pin a specific fingerprint.
fn with_fingerprints(mut d: MdDescriptor) -> MdDescriptor {
    let fp = [0x73, 0xc5, 0xda, 0x0a];
    d.tlv.fingerprints = Some((0..d.n).map(|i| (i, fp)).collect());
    d
}

/// Ten hand-built descriptors covering the eleven arms I-1 named (`Alt` and
/// `AndB` share one fixture, a tap leaf). Each row: the fixture, and a
/// content marker proving the FORWARD rendering actually reaches the
/// fragment it claims to -- the sugar spellings are miniscript's own Display
/// output and match `tests/proptest_to_miniscript.rs`'s own pins for the
/// same fragments (`tv:` = `and_v(_,1)`, `u:` = `or_i(_,0)`, `dv:` =
/// `dupif(verify(_))`, `j:` = `nonzero`, `n:` = `zeronotequal`, `a:` = `alt`).
/// Coverage is then the SAME `descriptor_from_chains` self-check the main gate
/// uses: a wrong `Tag` mapping anywhere in the fixture makes the reconstructed
/// `Descriptor` re-render differently, and `descriptor_from_chains`'s
/// round trip catches it before this function's `marker` check ever would.
#[test]
fn eleven_uncovered_arms_round_trip() {
    use common::{descriptor_with_pubkeys, keyarg, node2, node3, timelock, tr_node, wrap};

    let true_node = || Node {
        tag: Tag::True,
        body: Body::Empty,
    };
    let false_node = || Node {
        tag: Tag::False,
        body: Body::Empty,
    };

    let cases: Vec<(&str, MdDescriptor, &str)> = vec![
        (
            "Terminal::True -- wsh(and_v(v:pk,1)), tv: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(
                    Tag::AndV,
                    wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
                    true_node(),
                ),
            ))),
            "tv:pk(",
        ),
        (
            "Terminal::False -- wsh(or_i(pk,0)), u: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(Tag::OrI, keyarg(Tag::PkK, 0), false_node()),
            ))),
            "u:pk(",
        ),
        (
            "Terminal::OrC -- wsh(and_v(or_c(pk,v:pk),1)), t:or_c( sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(
                    Tag::AndV,
                    node2(
                        Tag::OrC,
                        keyarg(Tag::PkK, 0),
                        wrap(Tag::Verify, keyarg(Tag::PkK, 1)),
                    ),
                    true_node(),
                ),
            ))),
            "t:or_c(",
        ),
        (
            "Terminal::DupIf -- wsh(or_i(pk,dv:older)), dv: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(
                    Tag::OrI,
                    keyarg(Tag::PkK, 0),
                    wrap(Tag::DupIf, wrap(Tag::Verify, timelock(Tag::Older, 144))),
                ),
            ))),
            "dv:older(",
        ),
        (
            "Terminal::NonZero -- wsh(j:pk), j: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                wrap(Tag::NonZero, keyarg(Tag::PkK, 0)),
            ))),
            "j:pk(",
        ),
        (
            "Terminal::ZeroNotEqual -- wsh(or_i(pk,n:and_v)), n: sugar",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node2(
                    Tag::OrI,
                    keyarg(Tag::PkK, 0),
                    wrap(
                        Tag::ZeroNotEqual,
                        node2(
                            Tag::AndV,
                            wrap(Tag::Verify, keyarg(Tag::PkK, 1)),
                            timelock(Tag::Older, 144),
                        ),
                    ),
                ),
            ))),
            "n:and_v(",
        ),
        (
            "Terminal::Alt + Terminal::AndB -- tr tap leaf and_b(pk,a:pk_h)",
            with_fingerprints(descriptor_with_pubkeys(tr_node(
                false,
                0,
                Some(node2(
                    Tag::AndB,
                    keyarg(Tag::PkK, 1),
                    wrap(Tag::Alt, keyarg(Tag::PkH, 2)),
                )),
            ))),
            "and_b(pk(",
        ),
        (
            "Terminal::AndOr -- wsh(andor(pk,older(144),pk))",
            with_fingerprints(descriptor_with_pubkeys(wrap(
                Tag::Wsh,
                node3(
                    Tag::AndOr,
                    keyarg(Tag::PkK, 0),
                    timelock(Tag::Older, 144),
                    keyarg(Tag::PkK, 1),
                ),
            ))),
            "andor(pk(",
        ),
        (
            "root MsDescriptor::Pkh -- bare pkh(@0)",
            with_fingerprints(descriptor_with_pubkeys(keyarg(Tag::Pkh, 0))),
            "pkh(",
        ),
        (
            "ShInner::Wpkh -- sh(wpkh(@0))",
            with_fingerprints(descriptor_with_pubkeys(wrap(Tag::Sh, keyarg(Tag::Wpkh, 0)))),
            "sh(wpkh(",
        ),
    ];

    assert_eq!(
        cases.len(),
        10,
        "ten fixtures cover the eleven named arms (Alt+AndB share one, a tap leaf)"
    );

    for (label, d, marker) in &cases {
        let rendered0 = to_miniscript_descriptor(d, 0)
            .unwrap_or_else(|e| panic!("{label}: forward render chain 0: {e}"))
            .to_string();
        assert!(
            rendered0.contains(marker),
            "{label}: fixture does not actually reach the claimed fragment -- rendered {rendered0}"
        );
        let rendered1 = to_miniscript_descriptor(d, 1)
            .unwrap_or_else(|e| panic!("{label}: forward render chain 1: {e}"))
            .to_string();
        // The walker's own round-trip self-check IS the coverage proof: if
        // this call returns Ok, `descriptor_from_chains` walked the
        // arm named by `label` and reproduced it byte-for-byte.
        descriptor_from_chains(&rendered0, &rendered1)
            .unwrap_or_else(|e| panic!("{label}: descriptor route: {e}"));
    }
}
