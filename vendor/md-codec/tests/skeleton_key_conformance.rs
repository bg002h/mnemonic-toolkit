//! Task 5: the conformance gate -- one key, two routes.
//!
//! `chunks -> key == descriptor -> key` for every vendored keyed vector
//! (`crates/md-codec/tests/vectors/keyed_*.conformance.json`, 46 of them),
//! proving `SkeletonKey` is a property of the POLICY rather than of the door
//! it was read through.
//!
//! ## What "the two routes" actually are
//!
//! * **The chunk route** decodes the REAL md1 wire chunk set vendored beside
//!   each record (`<name>.phrase.txt`: a `chunk-set-id:` header followed by
//!   the chunk strings) via [`md_codec::chunk::reassemble`] -- the same
//!   bytes a SeedHammer II plate carries.
//! * **The descriptor route** parses the BIP-380 descriptor STRING the same
//!   record carries (`chains.0.descriptor` / `chains.1.descriptor` in the
//!   `.conformance.json`, real xpubs/fingerprints/origins inlined, no `@N`
//!   placeholders) and walks its `miniscript::Descriptor<DescriptorPublicKey>`
//!   AST back into an `md_codec::encode::Descriptor`. `md-codec` cannot
//!   dev-depend on `md-cli` (circular; see `tests/render_template_snapshot.rs`),
//!   so there is no borrowed parser for this -- the walker below is this
//!   task's own, built by mirroring `src/to_miniscript.rs`'s forward
//!   converter arm-for-arm in reverse.
//!
//! Both routes feed the ONE [`skeleton`]/[`skeleton_key`] implementation;
//! this file computes nothing about the key itself.
//!
//! ## Why the reconstruction is trustworthy -- and exactly how far that goes
//!
//! `parse_descriptor` below does not depend on getting the use-site path
//! right by construction -- it HARDCODES `UseSitePath::standard_multipath()`
//! (every one of the 46 vectors uses `/<0;1>/*` uniformly; mechanically
//! verified against `md_codec::test_vectors::MANIFEST` before writing this)
//! and then immediately self-checks the whole reconstruction by feeding it
//! BACK through the already-tested forward converter
//! (`md_codec::to_miniscript_descriptor`) and asserting the re-rendered
//! chain-0 and chain-1 descriptor strings are byte-identical to the
//! originals, checksum included. A wrong fingerprint, a dropped origin
//! component, a mis-numbered placeholder, or a wrong use-site guess all show
//! up there, before the conformance assertion ever runs.
//!
//! **What that round trip does NOT certify -- measured, not assumed (review
//! round 1, I-1 / M-1).** Two limits, both real, neither changing today's
//! result (zero disagreements, all 46 vectors, see below), both narrowing
//! what a future maintainer growing this corpus may assume from a clean run:
//!
//! 1. **Only an EXECUTED arm is certified.** Instrumenting the walker over
//!    the 46-vector corpus alone shows 32 of its 48 `Terminal`/root decision
//!    points are hit. Five of the misses are refusals or pre-empted by a
//!    more specific arm (`Terminal::PkK`/`PkH`/`RawPkH` panic on md1's own
//!    invariant; `Terminal::SortedMulti`/`SortedMultiA` in the generic path
//!    are caught by the three legal-position special cases first). **The
//!    remaining eleven are LIVE conversions the 46-vector corpus alone never
//!    reaches:** `Terminal::True`, `Terminal::False`, `Terminal::Alt`,
//!    `Terminal::DupIf`, `Terminal::NonZero`, `Terminal::ZeroNotEqual`,
//!    `Terminal::AndB`, `Terminal::AndOr`, `Terminal::OrC`, a bare root
//!    `Tag::Pkh` (`MsDescriptor::Pkh`), and `sh(wpkh(...))`
//!    (`ShInner::Wpkh`). A wrong `Tag` mapping in any of those eleven would
//!    be invisible to both the round trip and the conformance comparison,
//!    because the arm never runs -- an untested arm cannot be certified by a
//!    gate no matter how strong the gate is on the arms it does exercise.
//!    `eleven_uncovered_arms_round_trip` (below, same file, since the walker
//!    is private to it) closes exactly this gap: one small hand-built
//!    descriptor per arm -- several lifted directly from
//!    `tests/proptest_to_miniscript.rs`'s `self_test_*` cells and
//!    `tests/bitcoind_differential.rs`'s `andor` shape, already proven valid
//!    there rather than invented here -- run through this same
//!    `parse_descriptor` self-check. All eleven are proven by that test, as
//!    of this fold; if the corpus grows to cover any of them directly, that
//!    is a welcome overlap, not a reason to remove the dedicated case.
//! 2. **The round trip pins only the RENDERED PROJECTION of the descriptor,
//!    never the whole `Descriptor` value.** `Body::Tr { key_index }` under
//!    `is_nums: true` is a concrete, verified example: `to_miniscript_descriptor`
//!    ignores `key_index` whenever `is_nums` is set, so a walker that wrote a
//!    wrong value there would still round-trip and still pass this gate.
//!    Harmless TODAY because `skeleton_key` also never reads `key_index`
//!    under `is_nums` (`key_path_kind` reads `is_nums`, not `key_index`) --
//!    but that is a fact about what `skeleton_key` happens to need, not
//!    about what this round trip checks. A future `Skeleton` field reading
//!    an unrendered part of `Descriptor` would need a different guard than
//!    this one.
//!
//! Placeholder NUMBERING is deliberately not matched to the chunk route's:
//! `canonicalize_placeholder_indices` is called on the descriptor-route
//! result before keying it, so both routes converge on the same
//! first-occurrence canonical numbering regardless of the order this
//! walker happened to assign indices in (design note, and the brief's own
//! likely-cause #2).

mod common;

use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use bitcoin::bip32::{ChildNumber, DerivationPath};
use miniscript::descriptor::{
    Descriptor as MsDescriptor, DescriptorPublicKey, ShInner, SinglePubKey, TapTree, Wsh,
};
use miniscript::{Legacy, Miniscript, ScriptContext, Segwitv0, Tap, Terminal, Threshold};

use md_codec::canonicalize_placeholder_indices;
use md_codec::chunk::reassemble;
use md_codec::encode::Descriptor as MdDescriptor;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::skeleton::{skeleton, skeleton_key};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::to_miniscript_descriptor;
use md_codec::tree::{Body, InternalKey, Node};
use md_codec::use_site_path::UseSitePath;

/// BIP-341 NUMS H-point x-only coordinate. Same value as `src/nums.rs`
/// (`pub(crate)`, unreachable from an integration test) -- this file's own
/// copy, exactly as `tests/render_template_snapshot.rs` and
/// `src/test_vectors.rs` already carry it verbatim.
const NUMS_H_POINT_X_ONLY_HEX: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

// ─────────────────────────────────────────────────────────────────────────
// Vector loading. A glob that matches nothing must FAIL the floor
// assertion below, not silently pass -- so `list_keyed_conformance_files`
// returns an EMPTY Vec on a missing directory (never panics), leaving the
// final `checked >= 40` assertion as the one thing that can catch it.
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
// The descriptor route: `miniscript::Descriptor<DescriptorPublicKey>` ->
// `md_codec::encode::Descriptor`. Mirrors `src/to_miniscript.rs`'s
// `node_to_miniscript` / `node_to_descriptor` / `tree_to_taptree` arm for
// arm, in reverse.
// ─────────────────────────────────────────────────────────────────────────

/// Placeholder indices are assigned in walk order; `canonicalize_placeholder_indices`
/// renumbers to first-occurrence order afterward, so this order need not
/// match the chunk route's.
struct KeyRegistry {
    keys: Vec<DescriptorPublicKey>,
}

impl KeyRegistry {
    fn register(&mut self, pk: &DescriptorPublicKey) -> u8 {
        self.keys.push(pk.clone());
        u8::try_from(self.keys.len() - 1).expect("fewer than 256 keys in any vector")
    }
}

fn is_nums_key(pk: &DescriptorPublicKey) -> bool {
    match pk {
        DescriptorPublicKey::Single(single) if single.origin.is_none() => {
            matches!(&single.key, SinglePubKey::XOnly(x) if x.to_string() == NUMS_H_POINT_X_ONLY_HEX)
        }
        _ => false,
    }
}

fn wrap1<Ctx: ScriptContext>(
    tag: Tag,
    inner: &Arc<Miniscript<DescriptorPublicKey, Ctx>>,
    reg: &mut KeyRegistry,
) -> Node {
    Node {
        tag,
        body: Body::Children(vec![terminal_to_node(inner, reg)]),
    }
}

fn wrap2<Ctx: ScriptContext>(
    tag: Tag,
    l: &Arc<Miniscript<DescriptorPublicKey, Ctx>>,
    r: &Arc<Miniscript<DescriptorPublicKey, Ctx>>,
    reg: &mut KeyRegistry,
) -> Node {
    Node {
        tag,
        body: Body::Children(vec![terminal_to_node(l, reg), terminal_to_node(r, reg)]),
    }
}

fn multikeys_node<const MAX: usize>(
    tag: Tag,
    thresh: &Threshold<DescriptorPublicKey, MAX>,
    reg: &mut KeyRegistry,
) -> Node {
    let k = u8::try_from(thresh.k()).expect("k fits u8");
    let indices = thresh.data().iter().map(|pk| reg.register(pk)).collect();
    Node {
        tag,
        body: Body::MultiKeys { k, indices },
    }
}

/// Convert one miniscript leaf/combinator into an md1 `Node`. Mirrors
/// `node_to_miniscript` in `src/to_miniscript.rs`, reversed arm for arm.
fn terminal_to_node<Ctx: ScriptContext>(
    ms: &Miniscript<DescriptorPublicKey, Ctx>,
    reg: &mut KeyRegistry,
) -> Node {
    match &ms.node {
        Terminal::True => Node {
            tag: Tag::True,
            body: Body::Empty,
        },
        Terminal::False => Node {
            tag: Tag::False,
            body: Body::Empty,
        },
        Terminal::PkK(pk) => {
            panic!("bare unwrapped PkK reached the walker (md1 always Check-wraps): {pk}")
        }
        Terminal::PkH(pk) => {
            panic!("bare unwrapped PkH reached the walker (md1 always Check-wraps): {pk}")
        }
        Terminal::RawPkH(_) => panic!("Terminal::RawPkH is not constructible from md1"),
        Terminal::After(lt) => Node {
            tag: Tag::After,
            body: Body::Timelock(lt.to_consensus_u32()),
        },
        Terminal::Older(lt) => Node {
            tag: Tag::Older,
            body: Body::Timelock(lt.to_consensus_u32()),
        },
        Terminal::Sha256(h) => Node {
            tag: Tag::Sha256,
            body: Body::Hash256Body(hash_to_32(h)),
        },
        Terminal::Hash256(h) => Node {
            tag: Tag::Hash256,
            body: Body::Hash256Body(hash_to_32(h)),
        },
        Terminal::Ripemd160(h) => Node {
            tag: Tag::Ripemd160,
            body: Body::Hash160Body(hash_to_20(h)),
        },
        Terminal::Hash160(h) => Node {
            tag: Tag::Hash160,
            body: Body::Hash160Body(hash_to_20(h)),
        },
        Terminal::Alt(inner) => wrap1(Tag::Alt, inner, reg),
        Terminal::Swap(inner) => wrap1(Tag::Swap, inner, reg),
        Terminal::Check(inner) => {
            // Phase E collapse, reversed: `Check(PkK)` / `Check(PkH)` is the
            // bare `pk(...)` / `pkh(...)` fragment on the wire, never a
            // `Tag::Check` wrapping a `Tag::PkK`/`Tag::PkH` -- exactly the
            // special case `node_to_miniscript`'s own `Tag::Check` arm
            // documents re-applying forward.
            match &inner.node {
                Terminal::PkK(pk) => {
                    let index = reg.register(pk);
                    return Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index },
                    };
                }
                Terminal::PkH(pk) => {
                    let index = reg.register(pk);
                    return Node {
                        tag: Tag::PkH,
                        body: Body::KeyArg { index },
                    };
                }
                _ => {}
            }
            wrap1(Tag::Check, inner, reg)
        }
        Terminal::DupIf(inner) => wrap1(Tag::DupIf, inner, reg),
        Terminal::Verify(inner) => wrap1(Tag::Verify, inner, reg),
        Terminal::NonZero(inner) => wrap1(Tag::NonZero, inner, reg),
        Terminal::ZeroNotEqual(inner) => wrap1(Tag::ZeroNotEqual, inner, reg),
        Terminal::AndV(l, r) => wrap2(Tag::AndV, l, r, reg),
        Terminal::AndB(l, r) => wrap2(Tag::AndB, l, r, reg),
        Terminal::AndOr(a, b, c) => Node {
            tag: Tag::AndOr,
            body: Body::Children(vec![
                terminal_to_node(a, reg),
                terminal_to_node(b, reg),
                terminal_to_node(c, reg),
            ]),
        },
        Terminal::OrB(l, r) => wrap2(Tag::OrB, l, r, reg),
        Terminal::OrC(l, r) => wrap2(Tag::OrC, l, r, reg),
        Terminal::OrD(l, r) => wrap2(Tag::OrD, l, r, reg),
        Terminal::OrI(l, r) => wrap2(Tag::OrI, l, r, reg),
        Terminal::Thresh(thresh) => {
            let k = u8::try_from(thresh.k()).expect("k fits u8");
            let children = thresh
                .data()
                .iter()
                .map(|c| terminal_to_node(c, reg))
                .collect();
            Node {
                tag: Tag::Thresh,
                body: Body::Variable { k, children },
            }
        }
        Terminal::Multi(thresh) => multikeys_node(Tag::Multi, thresh, reg),
        Terminal::SortedMulti(thresh) => multikeys_node(Tag::SortedMulti, thresh, reg),
        Terminal::MultiA(thresh) => multikeys_node(Tag::MultiA, thresh, reg),
        Terminal::SortedMultiA(thresh) => multikeys_node(Tag::SortedMultiA, thresh, reg),
    }
}

fn hash_to_32<H: bitcoin::hashes::Hash<Bytes = [u8; 32]>>(h: &H) -> [u8; 32] {
    h.to_byte_array()
}
fn hash_to_20<H: bitcoin::hashes::Hash<Bytes = [u8; 20]>>(h: &H) -> [u8; 20] {
    h.to_byte_array()
}

/// `wsh(...)`'s single child, whether reached at the top level or nested
/// under `sh(...)`. Mirrors `wsh_inner_to_descriptor`'s SortedMulti special
/// case (the ONLY place a bare `Tag::SortedMulti` is legal per
/// `to_miniscript.rs`'s own refusal for the nested position).
fn wsh_to_node(wsh: &Wsh<DescriptorPublicKey>, reg: &mut KeyRegistry) -> Node {
    if let Terminal::SortedMulti(thresh) = &wsh.as_inner().node {
        return multikeys_node(Tag::SortedMulti, thresh, reg);
    }
    terminal_to_node::<Segwitv0>(wsh.as_inner(), reg)
}

/// `sh(...)`'s single child: `sh(wsh(...))`, `sh(wpkh(...))`,
/// `sh(sortedmulti(...))`, or a bare Legacy miniscript. Mirrors
/// `sh_inner_to_descriptor`.
fn sh_inner_to_node(inner: &ShInner<DescriptorPublicKey>, reg: &mut KeyRegistry) -> Node {
    match inner {
        ShInner::Wpkh(w) => {
            let index = reg.register(w.as_inner());
            Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index },
            }
        }
        ShInner::Wsh(wsh) => {
            let child = wsh_to_node(wsh, reg);
            Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![child]),
            }
        }
        ShInner::Ms(ms) => {
            if let Terminal::SortedMulti(thresh) = &ms.node {
                return multikeys_node(Tag::SortedMulti, thresh, reg);
            }
            terminal_to_node::<Legacy>(ms, reg)
        }
    }
}

/// One tap-script leaf's root. `sortedmulti_a` is legal ONLY here (a
/// tap-leaf root), never nested -- mirrors `tree_to_taptree`'s explicit
/// special case, checked before falling through to the generic walker.
fn tap_leaf_to_node(ms: &Miniscript<DescriptorPublicKey, Tap>, reg: &mut KeyRegistry) -> Node {
    if let Terminal::SortedMultiA(thresh) = &ms.node {
        return multikeys_node(Tag::SortedMultiA, thresh, reg);
    }
    terminal_to_node::<Tap>(ms, reg)
}

/// Reconstruct the binary `Tag::TapTree` node from `TapTree`'s flat
/// `(depth, leaf)` list (`TapTree::leaves()`, depth-first left-to-right).
/// Standard "rebuild a full binary tree from an ordered leaf-depth
/// sequence" reduction: repeatedly combine adjacent equal-depth siblings
/// (mirrors upstream's own `Display` reconstruction in
/// `descriptor/tr/taptree.rs::fmt_helper`, which does the same walk to
/// place braces). A single-leaf tree (one `(0, leaf)` entry) falls out of
/// this loop as the bare leaf node itself -- no `TapTree` wrap -- matching
/// the v0.30 "single-leaf wire optimization"
/// (`Body::Tr { tree: Some(<bare leaf>) }`) with no special case needed.
fn taptree_to_node(tt: &TapTree<DescriptorPublicKey>, reg: &mut KeyRegistry) -> Node {
    let mut stack: Vec<(u8, Node)> = Vec::new();
    for item in tt.leaves() {
        let mut node = tap_leaf_to_node(item.miniscript(), reg);
        let mut depth = item.depth();
        while let Some(&(top_depth, _)) = stack.last() {
            if top_depth != depth {
                break;
            }
            let (_, left) = stack.pop().expect("just peeked");
            node = Node {
                tag: Tag::TapTree,
                body: Body::Children(vec![left, node]),
            };
            depth -= 1;
        }
        stack.push((depth, node));
    }
    assert_eq!(
        stack.len(),
        1,
        "taptree reconstruction did not converge to a single root"
    );
    stack.pop().expect("length checked above").1
}

fn ms_descriptor_to_node(desc: &MsDescriptor<DescriptorPublicKey>, reg: &mut KeyRegistry) -> Node {
    match desc {
        MsDescriptor::Wpkh(w) => {
            let index = reg.register(w.as_inner());
            Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index },
            }
        }
        MsDescriptor::Pkh(p) => {
            let index = reg.register(p.as_inner());
            Node {
                tag: Tag::Pkh,
                body: Body::KeyArg { index },
            }
        }
        MsDescriptor::Sh(sh) => {
            let child = sh_inner_to_node(sh.as_inner(), reg);
            Node {
                tag: Tag::Sh,
                body: Body::Children(vec![child]),
            }
        }
        MsDescriptor::Wsh(wsh) => {
            let child = wsh_to_node(wsh, reg);
            Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![child]),
            }
        }
        MsDescriptor::Tr(tr) => {
            let internal_key = if is_nums_key(tr.internal_key()) {
                InternalKey::NumsPoint
            } else {
                InternalKey::Slot(reg.register(tr.internal_key()))
            };
            let tree = tr.tap_tree().map(|tt| Box::new(taptree_to_node(tt, reg)));
            Node {
                tag: Tag::Tr,
                body: Body::Tr { internal_key, tree },
            }
        }
        MsDescriptor::Bare(_) => panic!("bare top-level descriptors are not an md1 shape"),
    }
}

fn derivation_path_to_origin_path(p: &DerivationPath) -> OriginPath {
    let components = p
        .as_ref()
        .iter()
        .map(|c| match c {
            ChildNumber::Normal { index } => PathComponent {
                hardened: false,
                value: *index,
            },
            ChildNumber::Hardened { index } => PathComponent {
                hardened: true,
                value: *index,
            },
        })
        .collect();
    OriginPath { components }
}

/// Parse `desc0_str` (the descriptor route's ONLY real input) into an
/// `md_codec::encode::Descriptor`, self-checking the reconstruction against
/// BOTH `desc0_str` and `desc1_str` before returning (see module doc).
fn parse_descriptor(desc0_str: &str, desc1_str: &str, name: &str) -> MdDescriptor {
    let desc0 = MsDescriptor::<DescriptorPublicKey>::from_str(desc0_str)
        .unwrap_or_else(|e| panic!("{name}: parse chain-0 descriptor: {e}"));

    let mut reg = KeyRegistry { keys: Vec::new() };
    let tree = ms_descriptor_to_node(&desc0, &mut reg);
    let n = u8::try_from(reg.keys.len()).expect("fewer than 256 keys");

    let mut origin_paths = Vec::with_capacity(reg.keys.len());
    let mut fingerprints = Vec::with_capacity(reg.keys.len());
    let mut pubkeys = Vec::with_capacity(reg.keys.len());
    for (i, pk) in reg.keys.iter().enumerate() {
        let idx = u8::try_from(i).expect("fewer than 256 keys");
        let DescriptorPublicKey::XPub(x) = pk else {
            panic!(
                "{name}: key @{idx} is not an XPub ({pk}) -- the descriptor route needs one xpub per placeholder"
            );
        };
        let (fp, origin_derivation) = x.origin.clone().unwrap_or_else(|| {
            panic!(
                "{name}: key @{idx} carries no [origin] -- the descriptor route has no \
                 fingerprint for it and would partition to nothing (likely cause #3)"
            )
        });
        origin_paths.push(derivation_path_to_origin_path(&origin_derivation));
        fingerprints.push((idx, fp.to_bytes()));
        let mut bytes = [0u8; 65];
        bytes[..32].copy_from_slice(x.xkey.chain_code.as_ref());
        bytes[32..].copy_from_slice(&x.xkey.public_key.serialize());
        pubkeys.push((idx, bytes));
    }

    let mut d = MdDescriptor {
        n,
        path_decl: PathDecl {
            n,
            paths: PathDeclPaths::Divergent(origin_paths),
        },
        // Hardcoded, not derived -- see module doc: every one of the 46
        // vectors uses the standard `<0;1>/*` use site (mechanically
        // verified against MANIFEST), and the round-trip check immediately
        // below would fail loudly if that assumption were ever wrong for a
        // future vector.
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(fingerprints),
            pubkeys: Some(pubkeys),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };

    // Machine-check the WHOLE reconstruction (structure, keys, origins,
    // use-site) by feeding it back through the forward converter and
    // demanding a byte-identical re-render of BOTH chains, checksum
    // included. This is what makes the hardcoded use-site above safe: a
    // wrong guess collapses this assertion, not the conformance test below.
    let re0 = to_miniscript_descriptor(&d, 0)
        .unwrap_or_else(|e| panic!("{name}: re-render chain 0: {e}"))
        .to_string();
    assert_eq!(
        re0, desc0_str,
        "{name}: the descriptor-route reconstruction does not round-trip chain 0 \
         (likely cause #1: the use-site guess above is wrong for this vector -- \
         a wrong VALUE trips here first; chain 1 only catches a use-site that is \
         right for chain 0 and wrong for chain 1)"
    );
    let re1 = to_miniscript_descriptor(&d, 1)
        .unwrap_or_else(|e| panic!("{name}: re-render chain 1: {e}"))
        .to_string();
    assert_eq!(
        re1, desc1_str,
        "{name}: the descriptor-route reconstruction does not round-trip chain 1 \
         (likely cause #1: the use-site guess above is wrong for this vector)"
    );

    // Likely cause #2: normalize BOTH routes to the same first-occurrence
    // numbering before keying, since this walker's own numbering need not
    // match the chunk route's (which is canonical by construction).
    canonicalize_placeholder_indices(&mut d)
        .unwrap_or_else(|e| panic!("{name}: canonicalize_placeholder_indices: {e}"));

    d
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

        let d_desc = parse_descriptor(&rec.descriptor0, &rec.descriptor1, &rec.name);
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
        checked >= 40,
        "only {checked} vectors keyed -- the gate is checking almost nothing"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Review round 1, I-1: direct unit coverage for the eleven walker arms the
// 46-vector corpus alone never reaches. See the module doc's "What that
// round trip does NOT certify" section for the full accounting.
// ─────────────────────────────────────────────────────────────────────────

/// A descriptor built from `tests/common/mod.rs`'s `descriptor_with_pubkeys`
/// (real xpubs, no fingerprints) with a fingerprint TLV entry ADDED for every
/// slot. `descriptor_with_pubkeys` alone renders keys with no `[origin]`
/// bracket at all (`to_miniscript.rs::assemble_origin_and_xkey`'s `origin`
/// field is `e.fingerprint.map(...)` -- `None` fingerprint means no bracket,
/// regardless of the divergent origin PATH `descriptor_with_pubkeys` already
/// sets), and `parse_descriptor` refuses a key with no bracket outright (its
/// own "likely cause #3" panic). The value is arbitrary -- these fixtures
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
/// Coverage is then the SAME `parse_descriptor` self-check the main gate
/// uses: a wrong `Tag` mapping anywhere in the fixture makes the reconstructed
/// `Descriptor` re-render differently, and `parse_descriptor`'s own
/// `assert_eq!` catches it before this function's `marker` check ever would.
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
        // this call returns without panicking, `parse_descriptor` walked the
        // arm named by `label` and reproduced it byte-for-byte.
        let _ = parse_descriptor(&rendered0, &rendered1, label);
    }
}
