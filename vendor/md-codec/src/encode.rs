//! Top-level encoder per spec §13.3.

use crate::bitstream::BitWriter;
use crate::error::Error;
use crate::header::Header;
use crate::origin_path::{PathDecl, PathDeclPaths};
use crate::tlv::TlvSection;
use crate::tree::{Body, InternalKey, Node, write_node};
use crate::use_site_path::UseSitePath;

/// Top-level descriptor parsed/built from a v0.30 wire payload.
///
/// Each field corresponds to a spec section: Header (§3.2), origin
/// `PathDecl` (§3.3), use-site `UseSitePath` (§3.4), descriptor `tree`
/// (§3.5–3.6), and trailing `tlv` section (§3.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Descriptor {
    /// Number of placeholders (1-indexed key universe size).
    pub n: u8,
    /// Origin path declaration (single or per-`@N` divergent).
    pub path_decl: PathDecl,
    /// Use-site (post-key) path applied to every key by default.
    pub use_site_path: UseSitePath,
    /// Descriptor tree root node.
    pub tree: Node,
    /// Trailing TLV section (overrides, fingerprints, etc.).
    pub tlv: TlvSection,
}

impl Descriptor {
    /// Bit width for placeholder-index encoding: ⌈log₂(n)⌉ per SPEC v0.30 §7.
    ///
    /// Index range is `0..n`. The NUMS H-point is signalled by an explicit
    /// `is_nums` bit on `Body::Tr` (SPEC §7), not by a reserved sentinel.
    /// MUST stay in lockstep with `decode::decode_payload`'s independent
    /// computation; a stale formula would silently desync the bitstream.
    pub fn key_index_width(&self) -> u8 {
        // ⌈log₂(n)⌉ for n ≥ 2; clamp to 0 at n ∈ {0, 1}.
        // Identity: ⌈log₂(n)⌉ = bit_length(n-1) for n ≥ 2.
        (32 - (self.n as u32).saturating_sub(1).leading_zeros()) as u8
    }

    /// The minimum wire version that can express this tree (SPEC §3d).
    /// NEVER a constant: identity.rs's two hash sites call write_node
    /// directly, and a constant v4 would make kind 0 and kind 1 hash
    /// identically while their addresses differ.
    ///
    /// CLOSED (stage 1b task 3): this function's return value used to be
    /// able to outrun what `write_node`/`read_node` actually encoded — a
    /// tree carrying `InternalKey::LianaUnspendable` returned 8 here (since
    /// stage 1b task 2) while `Body::Tr`'s match arm (tree.rs) still wrote
    /// the same v4 NUMS bit pattern for it that it wrote for `NumsPoint`,
    /// because the kind bit didn't exist yet. Task 3 added it: `write_node`
    /// now writes, and `read_node` now reads, a kind bit whenever
    /// `wire_version() == Header::WF_UNSPENDABLE_VERSION`, so this
    /// function's answer and the bytes `write_node` actually produces agree
    /// again. A `LianaUnspendable` tree now round-trips through
    /// `encode_md1_string` → `decode_md1_string` as `LianaUnspendable`, and
    /// its chunk set reassembles cleanly through `split` → `reassemble`.
    /// Version-4 bytes remain unchanged — stage 1a's byte-equality gate
    /// still proves it over all 65 vendored vectors, none of which carry
    /// `LianaUnspendable`.
    pub fn wire_version(&self) -> u8 {
        fn needs_v8(n: &Node) -> bool {
            match &n.body {
                Body::Tr { internal_key, tree } => {
                    *internal_key == InternalKey::LianaUnspendable
                        || tree.as_deref().is_some_and(needs_v8)
                }
                Body::Children(cs) | Body::Variable { children: cs, .. } => cs.iter().any(needs_v8),
                _ => false,
            }
        }
        if needs_v8(&self.tree) {
            Header::WF_UNSPENDABLE_VERSION
        } else {
            Header::WF_REDESIGN_VERSION
        }
    }

    /// Returns `true` iff this descriptor is in **wallet-policy mode** per
    /// SPEC §3.3: the `Pubkeys` TLV is present *and* contains at least one
    /// entry. Template-only mode (no `Pubkeys` TLV at all, or `Pubkeys =
    /// Some(vec![])` after sparse-decode) returns `false`.
    ///
    /// The check is a post-TLV-decode predicate; mode dispatch never reads
    /// a header bit.
    pub fn is_wallet_policy(&self) -> bool {
        matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())
    }

    /// Return the deterministic pre-chunking **canonical packed payload** of
    /// this descriptor as a bit-precise `(bytes, total_bits)` pair, for a
    /// downstream consumer that needs the raw, round-trippable byte payload
    /// (without the codex32/`md1` string wrapper or chunk framing).
    ///
    /// This delegates to [`encode_payload`], which canonicalizes BIP 388
    /// placeholder ordering internally (SPEC §6.1) before emitting bits — so
    /// the output is **canonical for any valid input** descriptor (two inputs
    /// that differ only in non-canonical placeholder numbering produce
    /// byte-identical payloads).
    ///
    /// The payload is **bit-aligned**: the returned `bytes` are zero-padded to
    /// a byte boundary, and `total_bits` is the exact unpadded bit count. The
    /// final byte may carry up to 7 trailing zero-pad bits, so `total_bits` is
    /// **load-bearing** — a consumer MUST round-trip with this exact bit count,
    /// not `bytes.len() * 8`. Feed both values back to
    /// [`Descriptor::from_canonical_payload_bytes`] to recover the descriptor.
    pub fn canonical_payload_bytes(&self) -> Result<(Vec<u8>, usize), Error> {
        encode_payload(self)
    }

    /// Reconstruct a [`Descriptor`] from a bit-precise canonical packed
    /// payload previously produced by [`Descriptor::canonical_payload_bytes`].
    ///
    /// `bytes` may be zero-padded to a byte boundary; `total_bits` is the
    /// exact payload bit count (the same value returned alongside the bytes).
    /// Delegates to [`decode_payload`](crate::decode::decode_payload).
    pub fn from_canonical_payload_bytes(
        bytes: &[u8],
        total_bits: usize,
    ) -> Result<Descriptor, Error> {
        crate::decode::decode_payload(bytes, total_bits)
    }
}

/// Encode a [`Descriptor`] into the canonical payload bit stream and return
/// `(bytes, total_bit_count)`. The bytes are zero-padded; `total_bit_count`
/// is the exact unpadded length needed for round-trip decoding (see §3.7's
/// "TLV section ends when codex32 total-length is exhausted" rule).
///
/// Per SPEC §6.1, the encoder canonicalizes BIP 388 placeholder
/// ordering before emitting bits: `@i` first appears in the tree before
/// `@j` for `j > i`. Canonicalization permutes the tree indices,
/// divergent path decl, and per-`@N` TLV maps atomically; if `d` is
/// already canonical it is unchanged.
pub fn encode_payload(d: &Descriptor) -> Result<(Vec<u8>, usize), Error> {
    encode_payload_inner(d, Admission::Enforce, None)
}

/// Which checks `encode_payload_inner` runs. It changes NOTHING about the bytes
/// produced -- every admission check is a pure refusal -- only whether a
/// descriptor is allowed through.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Admission {
    /// Minting a card: apply every rule.
    Enforce,
    /// Re-serialising a card that ALREADY EXISTS, to hash it. See
    /// [`encode_payload_for_identity`].
    SkipPolicy,
}

/// Serialise for IDENTITY only, applying no admission policy.
///
/// THIS EXISTS BECAUSE THE ENCODE-SIDE REFUSALS WERE REACHING DECODE.
/// `validate_origin_key_consistency` says so in its own words a few lines
/// below: "Enforced on encode rather than on decode, deliberately: this stops
/// new impossible cards without making already-written ones unreadable, and a
/// card that cannot be read is a backup that cannot be restored."
///
/// That intent was correct and the implementation leaked. `chunk::reassemble`
/// -- a DECODE path -- verifies a chunk set by recomputing the md1 encoding id,
/// `compute_md1_encoding_id` called `encode_payload`, and `encode_payload`
/// applies admission policy. So every rule added to the mint path retroactively
/// made older cards of that shape undecodable. Measured 2026-09-19: a 2-of-2
/// emitted by the shipped toolkit stopped reading, `verify-bundle` reporting
/// `md1_decode: fail OriginKeyContradiction` on a backup whose keys were
/// perfectly intact.
///
/// Hashing a card is not minting one. The card is already in metal; asking
/// whether it would be admitted TODAY answers a question nobody posed. So the
/// id is computed over the serialisation alone.
///
/// The BYTES ARE UNCHANGED -- this is purely about which checks run. Structural
/// errors (a malformed tree, an over-long path) still surface, because those
/// come from the writers themselves, not from the admission rules.
pub(crate) fn encode_payload_for_identity(d: &Descriptor) -> Result<(Vec<u8>, usize), Error> {
    encode_payload_inner(d, Admission::SkipPolicy, None)
}

/// Serialise a card applying NO admission policy -- for COMPARISON and
/// HASHING of cards that already exist, **never for minting**.
///
/// Same bytes as [`encode_payload`] for every descriptor that function
/// accepts; the only difference is that the mint-time rules (F-217 origin/key
/// consistency, F-218 duplicate key slots, SPEC §6's kind-1 shape rules and
/// the minimum-version rule) are not consulted. Structural errors still
/// surface, because those come from the writers themselves.
///
/// Why it is public (F-639, F-449 stage 2): `md verify` compares a decoded
/// card's serialisation against a template's, and mints nothing. Routing that
/// comparison through [`encode_payload`] let a mint policy decide whether an
/// engraved card could be CHECKED -- the same class as the 2026-09-19
/// regression `encode_payload_for_identity` (crate-private) documents, one layer out. A
/// caller that is about to write a card must use [`encode_payload`] (or
/// [`encode_md1_string`]); a caller that only asks "are these the same
/// bytes?" uses this.
pub fn encode_payload_unadmitted(d: &Descriptor) -> Result<(Vec<u8>, usize), Error> {
    encode_payload_inner(d, Admission::SkipPolicy, None)
}

/// `forced_version`, when `Some`, overrides the wire version this call
/// writes (both the header field and the version threaded into
/// `write_node`) instead of deriving it from `d.wire_version()`. `None` —
/// every real caller — writes the minimal version the tree needs, exactly
/// as before this parameter existed.
///
/// This is what lets [`crate::validate::validate_minimal_wire_version`]
/// (SPEC §6 row 6) be tested at all: no *public* API can construct a
/// non-minimal version (`encode_payload`/`encode_payload_for_identity` both
/// pass `None`), so the refusal is otherwise unreachable and its test would
/// be vacuous. See `encode_payload_at_forced_version` below, `#[cfg(test)]`
/// only.
fn encode_payload_inner(
    d: &Descriptor,
    admission: Admission,
    forced_version: Option<u8>,
) -> Result<(Vec<u8>, usize), Error> {
    let mut d_canonical = d.clone();
    crate::canonicalize::canonicalize_placeholder_indices(&mut d_canonical)?;
    let d = &d_canonical;
    crate::validate::validate_placeholder_usage(&d.tree, d.n)?;
    if let Some(overrides) = &d.tlv.use_site_path_overrides {
        crate::validate::validate_multipath_consistency(&d.use_site_path, overrides)?;
    }
    if matches!(d.tree.tag, crate::tag::Tag::Tr) {
        if let Body::Tr { tree: Some(t), .. } = &d.tree.body {
            crate::validate::validate_tap_script_tree(t)?;
        }
    }
    let version = forced_version.unwrap_or_else(|| d.wire_version());
    // F-217: refuse to MINT a card that declares one key origin for two
    // different keys. Enforced on encode rather than on decode, deliberately:
    // this stops new impossible cards without making already-written ones
    // unreadable, and a card that cannot be read is a backup that cannot be
    // restored. The decode-side refusal waits until the conformance corpus is
    // regenerated, so no gate is ever red while it lands.
    //
    // ALL ARE MINT-TIME POLICY, skipped when we are only re-serialising an
    // existing card to hash it (see `encode_payload_for_identity`). None of
    // them affects the bytes.
    if admission == Admission::Enforce {
        crate::validate::validate_origin_key_consistency(d)?;
        // F-218: refuse to mint a policy that names more cosigners than it has.
        crate::validate::validate_no_duplicate_key_slots(d)?;
        // Stage 1b task 7 / SPEC §6: the wire-kind-1 (Liana unspendable)
        // structural refusals, plus the minimum-version rule. Not in
        // `decode_payload_with_opts` -- a refusal there would make existing
        // plates unreadable, the exact regression this function's own doc
        // comment (above) documents for F-217/F-218.
        crate::validate::validate_unspendable_shape(d)?;
        crate::validate::validate_minimal_wire_version(d, version)?;
    }

    let mut w = BitWriter::new();
    let header = Header {
        version,
        divergent_paths: matches!(d.path_decl.paths, PathDeclPaths::Divergent(_)),
    };
    header.write(&mut w);
    d.path_decl.write(&mut w)?;
    d.use_site_path.write(&mut w)?;
    let kiw = d.key_index_width();
    write_node(&mut w, &d.tree, kiw, version)?;
    d.tlv.write(&mut w, kiw)?;
    let total_bits = w.bit_len();
    Ok((w.into_bytes(), total_bits))
}

/// Test-only hook forcing a NON-MINIMAL wire version past what
/// `Descriptor::wire_version()` would choose — the only way to make SPEC §6
/// row 6's `NonMinimalWireVersion` refusal observable at all, since every
/// real encoder path derives its version from the tree itself (see
/// `encode_payload_inner`'s doc comment). Always runs under
/// `Admission::Enforce`; a caller wanting to skip policy has
/// `encode_payload_for_identity` already.
#[cfg(test)]
pub(crate) fn encode_payload_at_forced_version(
    d: &Descriptor,
    version: u8,
) -> Result<(Vec<u8>, usize), Error> {
    encode_payload_inner(d, Admission::Enforce, Some(version))
}

/// True for any character treated as a display separator on intake: ALL Unicode
/// whitespace plus `-` and `,`. SPEC §3.2 (mstring display-grouping). None of
/// these appear in the codex32 alphabet (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`) or
/// the `ms`/`mk`/`md`/`1` structural chars (SPEC §4), so stripping is unambiguous.
pub fn is_display_separator(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == ','
}

/// Insert `separator` after every `group_size` characters (SPEC §3.1).
/// `group_size == 0` returns the input unchanged. Single line; ASCII-safe.
pub fn render_grouped(s: &str, group_size: usize, separator: char) -> String {
    if group_size == 0 {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + s.len() / group_size);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && i % group_size == 0 {
            out.push(separator);
        }
        out.push(ch);
    }
    out
}

/// Strip every display separator (SPEC §3.2) — used on intake before decode.
/// Idempotent; strips ONLY separators (other chars pass through, so a malformed
/// card is never silently "cleaned" into validity).
pub fn strip_display_separators(s: &str) -> String {
    s.chars().filter(|&c| !is_display_separator(c)).collect()
}

/// Render a codex32 string with optional N-char HYPHEN grouping for
/// transcription aid (spec §10.2). `group_size = 0` returns the input unchanged.
/// Back-compat wrapper over `render_grouped` (hyphen separator). Retained as
/// public API (documented in the technical manual); new callers use
/// `render_grouped` with an explicit separator.
pub fn render_codex32_grouped(s: &str, group_size: usize) -> String {
    render_grouped(s, group_size, '-')
}

/// Encode a Descriptor into a complete codex32 md1 string (HRP + payload + BCH checksum).
/// Returns the canonical single-string form.
pub fn encode_md1_string(d: &Descriptor) -> Result<String, Error> {
    let (bytes, bit_len) = encode_payload(d)?;
    crate::codex32::wrap_payload(&bytes, bit_len)
}

#[cfg(test)]
mod render_tests {
    use super::*;

    #[test]
    fn render_groups_at_4() {
        assert_eq!(render_codex32_grouped("md1qpz9r4cy7", 4), "md1q-pz9r-4cy7");
    }

    #[test]
    fn render_zero_group_size_no_grouping() {
        assert_eq!(render_codex32_grouped("md1qpz9r4cy7", 0), "md1qpz9r4cy7");
    }

    #[test]
    fn render_grouped_separators_and_unbroken() {
        assert_eq!(render_grouped("abcdefghij", 5, ' '), "abcde fghij");
        assert_eq!(render_grouped("abcdefghij", 5, '-'), "abcde-fghij");
        assert_eq!(render_grouped("abcdefghij", 5, ','), "abcde,fghij");
        assert_eq!(render_grouped("abcdefghij", 0, ' '), "abcdefghij");
        assert_eq!(render_grouped("abcde", 5, ' '), "abcde");
        assert_eq!(render_grouped("abcdefg", 3, '-'), "abc-def-g");
        assert_eq!(render_grouped("", 5, ' '), "");
    }

    #[test]
    fn render_codex32_grouped_still_hyphens() {
        // back-compat wrapper: unchanged behavior
        assert_eq!(render_codex32_grouped("abcdefghij", 5), "abcde-fghij");
        assert_eq!(render_codex32_grouped("abcde", 0), "abcde");
    }

    #[test]
    fn strip_display_separators_whitespace_hyphen_comma() {
        assert_eq!(strip_display_separators("abcde fghij"), "abcdefghij");
        assert_eq!(strip_display_separators("ab-cd,ef gh"), "abcdefgh");
        assert_eq!(strip_display_separators("ab\tcd\r\nef"), "abcdef");
        assert_eq!(strip_display_separators("ms1qpzry9x8"), "ms1qpzry9x8");
        let once = strip_display_separators("a b-c,d");
        assert_eq!(strip_display_separators(&once), once);
    }
}

#[cfg(test)]
mod is_wallet_policy_tests {
    use super::*;
    use crate::origin_path::OriginPath;
    use crate::tag::Tag;
    use crate::tlv::TlvSection;

    fn wpkh_template_only() -> Descriptor {
        Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            },
            tlv: TlvSection::new_empty(),
        }
    }

    #[test]
    fn is_wallet_policy_returns_false_for_template_only() {
        // pubkeys = None → not wallet-policy mode.
        let d = wpkh_template_only();
        assert!(!d.is_wallet_policy());
    }

    #[test]
    fn is_wallet_policy_returns_false_for_empty_pubkeys() {
        // pubkeys = Some(vec![]) is impossible to encode (encoder rejects)
        // but the decoder may shape this state in transit. Predicate must
        // still report "not wallet-policy" so dispatch is presence-driven.
        let mut d = wpkh_template_only();
        d.tlv.pubkeys = Some(Vec::new());
        assert!(!d.is_wallet_policy());
    }

    #[test]
    fn is_wallet_policy_returns_true_for_populated_pubkeys() {
        let mut d = wpkh_template_only();
        d.tlv.pubkeys = Some(vec![(0u8, [0u8; 65])]);
        assert!(d.is_wallet_policy());
    }
}

/// Stage 1b task 7: SPEC §6's ENCODE-side refusals for a wire-kind-1 (Liana
/// unspendable) internal key. Two of the four live here as UNIT tests
/// because they need crate-private surface (`encode_payload_inner`'s
/// `Admission::SkipPolicy` bypass, and the `encode_payload_at_forced_version`
/// test-only hook) that `encode_payload` alone cannot reach; the other two
/// (`UnspendableWithSortedMultiA`, `UnspendableUseSiteNotCanonical`) are
/// integration tests in `tests/liana_unspendable.rs`, built from vendored
/// evidence via `kind1_from_vector` -- unreachable from here (`tests/common/`
/// is a different crate root).
#[cfg(test)]
mod unspendable_shape_tests {
    use super::*;
    use crate::decode::decode_payload;
    use crate::origin_path::OriginPath;
    use crate::tag::Tag;
    use crate::tlv::TlvSection;

    /// A kind-1 tr whose taptree contains a `sortedmulti_a` leaf, built
    /// directly from `Node`/`Body` -- `tests/common/` (where
    /// `kind1_from_vector` lives) is a different crate root, unreachable
    /// from a unit test inside `src/`.
    ///
    /// `Tag::SortedMultiA` takes `Body::MultiKeys`, NOT `Body::Variable` --
    /// the latter's doc says "Tag::Thresh ONLY; multi-family tags use
    /// MultiKeys" (tree.rs:38-56). MEASURED: the `Variable` spelling encodes
    /// to 106 bits and `decode_payload` returns
    /// `TlvLengthExceedsRemaining{length:9,remaining:5}`, so a test built on
    /// it would be RED before §6 exists, blaming the decode path instead of
    /// the missing refusal.
    fn in_crate_tr_liana_with_sortedmulti_a_leaf() -> Descriptor {
        let leaf = Node {
            tag: Tag::SortedMultiA,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        };
        let tree = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                internal_key: InternalKey::LianaUnspendable,
                tree: Some(Box::new(leaf)),
            },
        };
        // NOT an empty origin path. `wpkh_template_only` (above) can use one;
        // a `tr(@N, TapTree)` CANNOT -- it sits in `canonical_origin`'s
        // forced-explicit column, and an empty decl gives
        // `Err(MissingExplicitOrigin{idx:0})` at DECODE, which would read as
        // a decode-path bug rather than a fixture bug. MEASURED: empty ->
        // encode Ok 60 bits -> decode Err(MissingExplicitOrigin{idx:0}).
        // Shape copied from `origin_path.rs:216-226`
        // (`path_decl_shared_round_trip`).
        Descriptor {
            n: 3,
            path_decl: PathDecl {
                n: 3,
                paths: PathDeclPaths::Shared(OriginPath {
                    components: vec![
                        crate::origin_path::PathComponent {
                            hardened: true,
                            value: 48,
                        },
                        crate::origin_path::PathComponent {
                            hardened: true,
                            value: 0,
                        },
                        crate::origin_path::PathComponent {
                            hardened: true,
                            value: 0,
                        },
                        crate::origin_path::PathComponent {
                            hardened: true,
                            value: 3,
                        },
                    ],
                }),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree,
            tlv: TlvSection::new_empty(),
        }
    }

    /// MEASURED AGAINST `8f18655c` (this worktree's tip when task 7 began):
    /// Task 3 (the kind bit, `94ab36c7`) has landed, so round-trip EQUALITY
    /// now holds for this fixture -- confirmed empirically below, not
    /// reasoned about, by comparing the decoded `Descriptor` to the input
    /// with `assert_eq!` rather than the weaker `is_ok()`.
    ///
    /// `encode.rs`'s own doc comment on `encode_payload_for_identity`
    /// records the 2026-09-19 regression this test pins as an assertion:
    /// mint-side refusals leaking into decode made a shipped 2-of-2 stop
    /// reading. `Admission::SkipPolicy` is the mint-bypass path
    /// `encode_payload_for_identity`/`chunk::reassemble` use to re-serialise
    /// an ALREADY-MINTED card without re-applying policy that may have
    /// changed since.
    ///
    /// The fixture must do TWO things at once: trip §6's `sortedmulti_a`
    /// refusal under `Admission::Enforce`, AND survive
    /// `Admission::SkipPolicy` encode followed by `decode_payload`. A
    /// fixture that only did the first would make this test vacuous.
    #[test]
    fn a_refused_shape_still_decodes_so_existing_cards_never_stop_reading() {
        let d = in_crate_tr_liana_with_sortedmulti_a_leaf();
        // Confirm it actually trips the mint-time refusal under Enforce --
        // otherwise this fixture would prove nothing about the regression
        // it exists to pin.
        assert!(matches!(
            encode_payload(&d),
            Err(Error::UnspendableWithSortedMultiA)
        ));
        let (bytes, bits) =
            encode_payload_inner(&d, Admission::SkipPolicy, None).expect("mint-bypass encode");
        assert_eq!(
            decode_payload(&bytes, bits).expect("a mint-side refusal reached decode"),
            d,
            "round-trip EQUALITY, not just is_ok() -- Task 3's kind bit means this now holds"
        );
    }

    /// A kind-0 (`NumsPoint`) root `tr()` -- no `LianaUnspendable` anywhere
    /// in the tree -- with one placeholder leaf so `n=1`'s `@0` is
    /// referenced (an unreferenced placeholder is `PlaceholderNotReferenced`,
    /// a different, earlier check).
    fn all_nums_tr() -> Descriptor {
        Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    internal_key: InternalKey::NumsPoint,
                    tree: Some(Box::new(Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    })),
                },
            },
            tlv: TlvSection::new_empty(),
        }
    }

    /// SPEC §6 row 6, the minimum-version rule: forcing version 8 on a tree
    /// with no kind-1 node anywhere must be refused at ENCODE. No public API
    /// lets a caller pick a non-minimal version -- `encode_payload_at_forced_version`
    /// is the `#[cfg(test)]`-only hook that makes this refusal observable at
    /// all (see its own doc comment).
    #[test]
    fn version_8_with_no_kind_1_node_is_refused_at_encode() {
        assert!(matches!(
            encode_payload_at_forced_version(&all_nums_tr(), 8),
            Err(Error::NonMinimalWireVersion {
                got: 8,
                minimal: Header::WF_REDESIGN_VERSION
            })
        ));
    }
}
