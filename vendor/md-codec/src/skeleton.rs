//! `Skeleton` and `SkeletonKey` — Task 4 of the coordinator-compatibility
//! plan (`design/DESIGN_coordinator_compatibility.md` §1A in `mnemonic-engrave`).
//!
//! Everything a later rule may read about a decoded policy, assembled from
//! Tasks 1-3: the branch decomposition ([`crate::policy_shape`]), the
//! abstracting template renderer ([`crate::render::descriptor_to_abstract_template`]),
//! and the two key-identity partitions
//! ([`crate::policy_shape::fp_partition`] / [`crate::policy_shape::key_partition`]).
//!
//! # The hard constraint this module exists to honour
//!
//! A partial-decoded descriptor — `DecodeOpts::partial()`, used by `md
//! decode` (`crates/md-cli/src/cmd/decode.rs`) and `md inspect`
//! (`inspect.rs`) for a card whose `@N` origin never resolved — produces
//! `fp_partition`/`key_partition` output BYTE-IDENTICAL to a genuinely
//! template-only card: both give `[[], []]` / `[]` regardless of whether
//! real key TLV data is present (`crate::policy_shape::fp_partition`'s own
//! doc comment measures this). Two consequences, both load-bearing:
//!
//! 1. [`skeleton`] gates on [`crate::canonicalize::expand_per_at_n`]
//!    *before* computing either partition. A descriptor whose keys will not
//!    expand yields NO `Skeleton` at all (design §1A clause (a3)) — never
//!    one whose partitions happen to render empty.
//! 2. [`Skeleton::keys_present`] is read from raw TLV presence
//!    ([`crate::encode::Descriptor::is_wallet_policy`]), never inferred from
//!    partition emptiness. Inferring it from emptiness would make a
//!    partial-decoded card and a template-only card produce the identical
//!    key with the identical `keys_present`, which is the exact
//!    false-evidence-match — two different policies sharing one key — this
//!    task exists to prevent.

use crate::canonicalize::expand_per_at_n;
use crate::encode::Descriptor;
use crate::error::Error;
use crate::policy_shape::{self, KeyPathKind, PolicyShape, RootKind};
use crate::render::{RenderError, descriptor_to_abstract_template};
use crate::tag::Tag;
use crate::tree::{Body, Node};
use std::fmt::Write as _;

/// Everything a coordinator-compatibility rule may read about one decoded
/// policy. Field membership mirrors design §1A's `Skeleton` exactly.
///
/// `shape.key_path` (not a separate field here) is the `key_path_kind` the
/// design's key-membership clause names — duplicating it as a second field
/// would reopen the exact "one rule, two copies" defect `crate::policy_shape`'s
/// own module doc warns against for its lock-band split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skeleton {
    /// The six-valued top-level wrapper (the fork's `ScriptKind`).
    pub root: RootKind,
    /// Whether an `sh(...)` root wraps a `wsh(...)`. `sh(wsh)` is NOT a
    /// `RootKind` value of its own (design §1A).
    pub inner_wsh: bool,
    /// The canonical `@i` template with lock values and digests abstracted
    /// to `kind#class`, and the use-site path kept (design §1A).
    ///
    /// NOT INJECTIVE against the underlying descriptor (design §1A, "the
    /// key's blind spots"): two policies with the same structure but
    /// DIFFERENT lock values, or DIFFERENT hashlock digests, render the same
    /// `kind#class` label whenever each is the only value of its kind in its
    /// own policy — `older(100)` and `older(65535)` both render
    /// `older(older-blocks#1)`, and two distinct `sha256` preimages both
    /// render `sha256(#1)`. This is an intentional design tradeoff (a
    /// too-coarse key would be *structurally* impossible instead), not a
    /// bug in this renderer — but it means `SkeletonKey` equality is
    /// necessary, not sufficient, evidence that two policies are
    /// byte-identical, and a coordinator refusal keyed on an exact lock
    /// value or a known-preimage list can disagree across two policies that
    /// share a `SkeletonKey`.
    pub template: String,
    /// The semantic decomposition: branches, their slots, locks, hashlocks,
    /// and the taproot internal-key kind.
    pub shape: PolicyShape,
    /// Per spend-path fingerprint partition — `[path][group][slot]`.
    pub fp_partition: Vec<Vec<Vec<u8>>>,
    /// Whole-policy key partition — `[group][slot]`.
    pub key_partition: Vec<Vec<u8>>,
    /// Raw TLV presence (`Descriptor::is_wallet_policy`) — deliberately NOT
    /// derived from partition emptiness (see the module doc), and
    /// deliberately NOT in [`SkeletonKey`]'s membership: it selects a later
    /// verdict, it does not make a template-only card a different policy
    /// from the same card seated.
    pub keys_present: bool,
}

/// Why [`skeleton`] refused to build a [`Skeleton`].
#[derive(Debug, PartialEq, Eq)]
pub enum SkeletonError {
    /// [`PolicyShape::complete`] was `false`: the walk met a node it does
    /// not understand. A partial decomposition keyed as if whole would
    /// match evidence measured on a policy it is not.
    IncompleteShape,
    /// [`expand_per_at_n`] failed — at least one `@N`'s origin cannot be
    /// resolved. No partition is computable, so no key exists (design §1A
    /// clause (a3)); the wrapped [`Error`] is whatever `expand_per_at_n`
    /// itself raised (typically `Error::MissingExplicitOrigin`).
    KeysDoNotExpand(Error),
    /// [`descriptor_to_abstract_template`] hit a structurally malformed
    /// tree node. Unreachable for a decoder-produced [`Descriptor`]
    /// (`RenderError`'s own doc comment); kept as a typed error rather than
    /// a panic for a hand-built one.
    Render(RenderError),
    /// The tree's root tag is not one of `{Wpkh, Pkh, Sh, Wsh, Tr}` — SPEC
    /// §11 already enforces this for every decoded `Descriptor`
    /// (`crates/md-codec/src/decode.rs`); kept as a typed error rather than
    /// a panic for a hand-built one.
    UnrecognizedRoot(Tag),
}

impl std::fmt::Display for SkeletonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkeletonError::IncompleteShape => write!(
                f,
                "the policy walk met a node it does not understand; no key can be computed from a partial decomposition"
            ),
            SkeletonError::KeysDoNotExpand(e) => write!(f, "keys will not expand: {e}"),
            SkeletonError::Render(e) => write!(f, "abstract-template render failed: {e}"),
            SkeletonError::UnrecognizedRoot(t) => write!(f, "unrecognized root tag: {t:?}"),
        }
    }
}

impl std::error::Error for SkeletonError {}

/// The one canonical serialization of a [`Skeleton`] — a `String` newtype
/// so a caller cannot accidentally compare it to an unrelated `String` or
/// reconstruct one by hand. See [`skeleton_key`] for the grammar.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkeletonKey(String);

impl SkeletonKey {
    /// Borrow the underlying serialized string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Derive the top-level [`RootKind`] and whether an `sh(...)` root wraps a
/// `wsh(...)` (`inner_wsh`). A different question from
/// [`policy_shape::policy_shape`]'s own `Tag::Sh` unwrap test
/// (`tree.tag == Tag::Sh && inner.tag == Tag::Wsh`, in `policy_shape.rs`):
/// that check exists to find the script the branch-decomposition walk
/// should descend into; this one exists to name the wrapper for
/// `Skeleton::root`/`Skeleton::inner_wsh`. Two different questions over the
/// same one bit of tree shape, so they stay two small functions rather than
/// one straining to answer both.
///
/// Only called from [`skeleton`] once `PolicyShape::complete` is known
/// `true` for the same tree, so the single-child `Sh`/`Wsh` shape
/// `policy_shape` already required holds here too — this still returns
/// `Err` rather than panicking, for any future caller that does not
/// preserve that order.
fn root_kind(tree: &Node) -> Result<(RootKind, bool), SkeletonError> {
    match tree.tag {
        Tag::Wpkh => Ok((RootKind::Wpkh, false)),
        Tag::Pkh => Ok((RootKind::Pkh, false)),
        Tag::Wsh => Ok((RootKind::Wsh, false)),
        Tag::Tr => Ok((RootKind::Tr, false)),
        Tag::Sh => {
            let Body::Children(children) = &tree.body else {
                return Err(SkeletonError::UnrecognizedRoot(Tag::Sh));
            };
            match children.first().map(|c| c.tag) {
                Some(Tag::Wsh) => Ok((RootKind::Sh, true)),
                Some(Tag::Wpkh) => Ok((RootKind::ShWpkh, false)),
                _ => Ok((RootKind::Sh, false)),
            }
        }
        other => Err(SkeletonError::UnrecognizedRoot(other)),
    }
}

/// Build the [`Skeleton`] for a decoded `d`, or refuse.
///
/// # Errors
///
/// Refuses — never infers, never guesses — in exactly two cases, checked
/// BEFORE any partition is computed (see the module doc's hard constraint):
///
/// - [`SkeletonError::KeysDoNotExpand`]: `expand_per_at_n(d)` failed.
/// - [`SkeletonError::IncompleteShape`]: `policy_shape(d).complete` is
///   `false`.
pub fn skeleton(d: &Descriptor) -> Result<Skeleton, SkeletonError> {
    // Gate 1 (design §1A (a3)), BEFORE either partition is computed: a
    // descriptor whose keys will not expand must not produce a key at all.
    // `expand_per_at_n`'s own success/failure is the ONE check — not a
    // second reimplementation over `unresolved_origin_indices()`, which
    // mirrors different semantics (`validate_explicit_origin_required`'s)
    // and does not cover every failure mode this gate must catch (e.g. a
    // present-but-empty `OriginPathOverrides` entry).
    expand_per_at_n(d).map_err(SkeletonError::KeysDoNotExpand)?;

    // Gate 2: the honesty contract. A partial decomposition keyed as if
    // whole would match evidence measured on a policy it is not.
    let shape = policy_shape::policy_shape(d);
    if !shape.complete {
        return Err(SkeletonError::IncompleteShape);
    }

    let (root, inner_wsh) = root_kind(&d.tree)?;
    let template = descriptor_to_abstract_template(d).map_err(SkeletonError::Render)?;
    let fp_partition = policy_shape::fp_partition(d, &shape);
    let key_partition = policy_shape::key_partition(d);
    // Raw TLV presence, NEVER partition emptiness — see the module doc and
    // `Descriptor::is_wallet_policy`'s own doc comment, the ONE predicate
    // for wallet-policy mode. A second, ad hoc "keys present" test here
    // (e.g. `!fp_partition.iter().all(Vec::is_empty)`) is exactly the "one
    // rule, three implementations" defect this plan has already found —
    // and, per the module doc, it is also simply WRONG: it cannot tell a
    // template-only card from a partial-decoded one.
    let keys_present = d.is_wallet_policy();

    Ok(Skeleton {
        root,
        inner_wsh,
        template,
        shape,
        fp_partition,
        key_partition,
        keys_present,
    })
}

/// Render one group of slots, already ascending
/// ([`policy_shape`]'s `group_ascending` guarantee) — `[s0,s1,...]`.
fn render_slots(slots: &[u8]) -> String {
    let mut out = String::from("[");
    for (i, slot) in slots.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write!(out, "{slot}").unwrap();
    }
    out.push(']');
    out
}

/// Render a list of groups, already ordered by each group's lowest slot
/// (`group_ascending`'s guarantee) — `[[g0][g1]...]`. One balanced bracket
/// expression: self-delimiting, so concatenating two of these (as
/// `render_fp_partition`'s per-path calls do) is unambiguous without a
/// separator between them.
fn render_group_list(groups: &[Vec<u8>]) -> String {
    let mut out = String::from("[");
    for g in groups {
        out.push_str(&render_slots(g));
    }
    out.push(']');
    out
}

/// Render `fp_partition` — one `render_group_list` per path, in template
/// traversal order (`PolicyShape::branches`'s own order), wrapped in one
/// more bracket level so the whole thing is ONE balanced expression. That
/// is what makes concatenating it directly with `key_partition`'s own
/// (differently-nested) rendering in [`skeleton_key`] unambiguous: each is
/// independently self-delimiting, so the boundary between them needs no
/// separator character of its own.
fn render_fp_partition(fp: &[Vec<Vec<u8>>]) -> String {
    let mut out = String::from("[");
    for path in fp {
        out.push_str(&render_group_list(path));
    }
    out.push(']');
    out
}

fn key_path_kind_label(k: KeyPathKind) -> &'static str {
    match k {
        KeyPathKind::NotTaproot => "NotTaproot",
        KeyPathKind::Nums => "Nums",
        KeyPathKind::Xpub => "Xpub",
        KeyPathKind::LianaUnspendable => "LianaUnspendable",
    }
}

/// Compute the one canonical [`SkeletonKey`] serialization for `s`.
///
/// Exactly, and this is the ONE spelling (design §1A): the abstract
/// template, `U+001F`, then `fp_partition` and `key_partition` rendered per
/// `render_fp_partition`/`render_group_list` (slots ascending, groups
/// ordered by their lowest slot, paths in template traversal order — all
/// three already guaranteed by [`policy_shape`]'s `group_ascending` and
/// `PolicyShape::branches`' own order, so this function does no reordering
/// of its own), then `U+001F` and the key-path kind. Slot ids are 0-based;
/// equality classes (inside `template`'s `kind#class` labels) are 1-based.
///
/// `root`/`inner_wsh` are NOT separately serialized here: both are
/// recoverable from `template`'s own literal wrapper spelling (`sh(`,
/// `wsh(`, `sh(wsh(`, …), so appending them as a fourth token would encode
/// the same fact twice. **This is a property of today's `render.rs` prefix
/// table, proven by an argument (fix round 1, I-4's review: the seven root
/// prefixes are pairwise distinguishable at the string's start), NOT a
/// type-level guarantee** — `Skeleton::root`/`Skeleton::inner_wsh` are
/// tested directly (`crates/md-codec/tests/skeleton_key.rs`'s
/// `root_and_inner_wsh_are_read_correctly`) precisely because a future
/// change to either `root_kind` or `render_node`'s prefix choices could
/// silently break the redundancy without this function noticing. Likewise
/// `key_path_kind` IS separately serialized even though it is *currently*
/// redundant with `template` too (`is_nums` always changes the internal-key
/// rendering) — see `crates/md-codec/tests/skeleton_key.rs`'s
/// `key_path_kind_is_part_of_the_key_independent_of_template` for why it is
/// kept in the string anyway. `keys_present` is excluded per design §1A: it
/// selects a verdict, it is not part of the policy's identity.
///
/// `template` is also not injective ON TREES, independent of the
/// abstraction in its own doc comment: `Tag::Pkh` and `Tag::PkH` both
/// render `pkh(@i/...)`, and `Verify(PkK)` and `Verify(Check(PkK))` both
/// render `v:pk(@i/...)` — two spellings of one miniscript fragment each,
/// so sharing a key is correct, not a defect (fix round 1, N-1).
pub fn skeleton_key(s: &Skeleton) -> SkeletonKey {
    let mut out = s.template.clone();
    out.push('\u{001F}');
    out.push_str(&render_fp_partition(&s.fp_partition));
    out.push_str(&render_group_list(&s.key_partition));
    out.push('\u{001F}');
    out.push_str(key_path_kind_label(s.shape.key_path));
    SkeletonKey(out)
}
