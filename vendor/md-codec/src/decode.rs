//! Top-level decoder per spec §13.2.

use crate::bitstream::BitReader;
use crate::encode::Descriptor;
use crate::error::{ContextKind, Error};
use crate::header::Header;
use crate::origin_path::PathDecl;
use crate::tag::Tag;
use crate::tlv::TlvSection;
use crate::tree::read_node;
use crate::use_site_path::UseSitePath;

/// Options controlling decode-time validation strictness (P0 pathless/
/// dead-card partial-decode).
///
/// The default (`allow_unresolved_origin: false`) is BYTE-IDENTICAL to
/// pre-P0 decode behavior: `Error::MissingExplicitOrigin` is raised for
/// any `@N` whose origin cannot be resolved (no canonical default AND no
/// explicit `path_decl`/override). When `true`, that ONE reject is
/// swallowed and the decode succeeds; the caller MUST query
/// [`crate::encode::Descriptor::unresolved_origin_indices`] on the
/// returned descriptor to learn which `@N` are unresolved (nothing extra
/// is recorded on the `Descriptor` itself).
///
/// INVARIANT (funds-load-bearing): this flag relaxes ONLY the
/// `validate_explicit_origin_required` outcome. No other decode check is
/// affected — placeholder-usage, multipath consistency, tap-script-tree
/// leaf validity, xpub-bytes validity, and (via
/// [`crate::chunk::reassemble_with_opts`]) per-chunk BCH, chunk-header
/// consistency, index-gap, and the derived-chunk-set-id / content-id
/// check all stay enforced regardless of this flag. The
/// `Error::EmptyOriginOverride` reject (P0.3) is likewise a DISTINCT,
/// always-fatal error class — never swallowed by this opt-in, even when
/// `allow_unresolved_origin` is `true` (fatal-in-partial).
///
/// `#[non_exhaustive]` (API-freeze, M-2): a future decode-relaxation option
/// can be added as a new field without a SemVer-major bump. Downstream
/// crates therefore cannot construct this with a struct literal — use
/// [`DecodeOpts::default`] (strict) or [`DecodeOpts::partial`]
/// (partial-allowing) instead.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct DecodeOpts {
    /// When `true`, `Error::MissingExplicitOrigin` is not raised at
    /// decode time. Default `false` (strict — today's behavior).
    pub allow_unresolved_origin: bool,
}

impl DecodeOpts {
    /// Partial-allowing decode options (`allow_unresolved_origin: true`),
    /// all other (future) options at their `Default`. The stable
    /// constructor for the render-path opt-in (`md decode`/`md inspect`,
    /// toolkit `verify-bundle` gate) — preferred over a struct literal
    /// since `DecodeOpts` is `#[non_exhaustive]`.
    pub fn partial() -> Self {
        Self {
            allow_unresolved_origin: true,
        }
    }
}

/// Decode a Descriptor from the canonical payload bit stream (strict:
/// byte-identical to pre-P0 behavior). Delegates to
/// [`decode_payload_with_opts`] with the default (strict) options.
/// `bytes` may be zero-padded; `total_bits` is the exact payload bit count.
pub fn decode_payload(bytes: &[u8], total_bits: usize) -> Result<Descriptor, Error> {
    decode_payload_with_opts(bytes, total_bits, DecodeOpts::default())
}

/// Decode a Descriptor from the canonical payload bit stream, honoring
/// `opts` (P0 partial-decode; see [`DecodeOpts`] for the contract).
/// `bytes` may be zero-padded; `total_bits` is the exact payload bit count.
pub fn decode_payload_with_opts(
    bytes: &[u8],
    total_bits: usize,
    opts: DecodeOpts,
) -> Result<Descriptor, Error> {
    let mut r = BitReader::with_bit_limit(bytes, total_bits);

    let header = Header::read(&mut r)?;
    let path_decl = PathDecl::read(&mut r, header.divergent_paths)?;
    let use_site_path = UseSitePath::read(&mut r)?;
    // SPEC v0.30 §7 width formula: ⌈log₂(n)⌉. v0.30 drops the +1 v0.18 used
    // to reserve the NUMS sentinel slot — NUMS is now signalled by an
    // explicit `is_nums` bit on Body::Tr. MUST mirror
    // `Descriptor::key_index_width` exactly; a stale formula silently
    // desyncs the bitstream.
    let key_index_width = (32 - (path_decl.n as u32).saturating_sub(1).leading_zeros()) as u8;
    let tree = read_node(&mut r, key_index_width)?;

    // SPEC §11: root tag MUST be in {Sh, Wsh, Wpkh, Pkh, Tr} (the wrapper-tag
    // allow-list — structural body validation for `Sh`/`Wsh` is separate).
    // Decoder-side hardening (defense in depth) — the parser-side enforces this
    // for CLI/template inputs; this catches malformed wires that bypass the
    // parser via direct bitstream construction. Note: `Sh` covers both
    // `sh(multi)` and `sh(wsh(multi))` which are distinct BIP-388 shapes sharing
    // the same root tag; per-shape validation happens at the policy layer.
    if !matches!(
        tree.tag,
        Tag::Sh | Tag::Wsh | Tag::Wpkh | Tag::Pkh | Tag::Tr
    ) {
        return Err(Error::OperatorContextViolation {
            tag: tree.tag,
            context: ContextKind::TopLevel,
        });
    }

    let tlv = TlvSection::read(&mut r, key_index_width, path_decl.n)?;

    let descriptor = Descriptor {
        n: path_decl.n,
        path_decl,
        use_site_path,
        tree,
        tlv,
    };

    crate::validate::validate_placeholder_usage(&descriptor.tree, descriptor.n)?;
    if let Some(overrides) = &descriptor.tlv.use_site_path_overrides {
        crate::validate::validate_multipath_consistency(&descriptor.use_site_path, overrides)?;
        // D5(a): reject non-canonical override shapes (an `@0` override, or a
        // redundant override equal to the baseline) — never emitted by our
        // encoders; defense-in-depth against hand-crafted wire.
        crate::validate::validate_use_site_overrides_canonical(
            &descriptor.use_site_path,
            overrides,
        )?;
    }
    if matches!(descriptor.tree.tag, crate::tag::Tag::Tr) {
        if let crate::tree::Body::Tr { tree: Some(t), .. } = &descriptor.tree.body {
            crate::validate::validate_tap_script_tree(t)?;
        }
    }
    // Spec v0.13 §6.3 + §6.4: enforce explicit-origin and xpub-validity
    // after the v0.11 ordering / multipath / taptree checks. Order matters:
    // ordering must run first so subsequent checks see canonical indices.
    //
    // P0.3 (I-1): the empty-origin-override reject is UNCONDITIONAL and a
    // DISTINCT error from `MissingExplicitOrigin` — it runs regardless of
    // `opts` and regardless of canonical-shape status (I-1a), so it is
    // never swallowed by partial-allowing decode below (I-1b,
    // fatal-in-partial).
    crate::validate::validate_no_empty_origin_overrides(&descriptor)?;
    match crate::validate::validate_explicit_origin_required(&descriptor) {
        Ok(()) => {}
        Err(Error::MissingExplicitOrigin { .. }) if opts.allow_unresolved_origin => {
            // P0.2: partial-allowing decode swallows ONLY this reject.
            // The caller queries `Descriptor::unresolved_origin_indices()`
            // on the returned descriptor to learn which `@N` are
            // unresolved.
        }
        Err(e) => return Err(e),
    }
    crate::validate::validate_xpub_bytes(&descriptor)?;

    Ok(descriptor)
}

/// Decode a Descriptor from a complete codex32 md1 string.
///
/// Uses the symbol-aligned bit count returned by `unwrap_string` (5 × symbol_count),
/// which is exact at the codex32 layer with ≤4 bits of trailing zero-padding —
/// well within the v11 decoder's TLV-rollback tolerance.
///
/// F-A2: in-band auto-dispatch per SPEC v0.30 §2.3. The chunked-flag lives in
/// bit 0 (LSB) of the first 5-bit symbol (`[v3][v2][v1][v0][chunked]` for a
/// chunk header vs `[divergent][v3][v2][v1][v0]` for a single payload). When
/// set, the string is a chunk-form md1 and MUST route through the
/// chunk-reassembly path (a 1-element set) rather than the single-payload
/// primitive `decode_payload` — mirroring `decode_with_correction`'s
/// single-string auto-dispatch. The usable single-payload version set {4,8,12}
/// is all-even ⇒ every currently-valid single-payload string has first-symbol
/// LSB = 0, so this dispatch never diverts an input that decodes today. No
/// recursion cycle: `reassemble` → `decode_payload`, never back to here.
///
/// Strict (byte-identical to pre-P0 behavior). Delegates to
/// [`decode_md1_string_with_opts`] with the default (strict) options.
pub fn decode_md1_string(s: &str) -> Result<Descriptor, Error> {
    decode_md1_string_with_opts(s, DecodeOpts::default())
}

/// Decode a Descriptor from a complete codex32 md1 string, honoring
/// `opts` (P0 partial-decode; see [`DecodeOpts`]). Routes chunk-form
/// strings through [`crate::chunk::reassemble_with_opts`] and
/// single-payload strings through [`decode_payload_with_opts`], so
/// `opts` reaches whichever layer actually performs the origin check.
pub fn decode_md1_string_with_opts(s: &str, opts: DecodeOpts) -> Result<Descriptor, Error> {
    let (bytes, symbol_aligned_bit_count) = crate::codex32::unwrap_string(s)?;
    // The first symbol occupies the top 5 bits of byte 0 (MSB-first packing),
    // so its LSB (the chunked-flag) is bit 3 of byte 0.
    let chunked_flag = bytes.first().map(|b| (b >> 3) & 0x01).unwrap_or(0);
    if chunked_flag == 1 {
        return crate::chunk::reassemble_with_opts(&[s], opts);
    }
    decode_payload_with_opts(&bytes, symbol_aligned_bit_count, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::encode_payload;
    use crate::origin_path::{OriginPath, PathComponent, PathDeclPaths};
    use crate::tlv::TlvSection;
    use crate::tree::{Body, Node};

    /// SPEC §11 TopLevel check: a wire payload whose root tag is outside the
    /// BIP-388 allow-list `{Sh, Wsh, Wpkh, Pkh, Tr}` must be rejected with
    /// `Error::OperatorContextViolation { context: ContextKind::TopLevel }`.
    /// The encoder has no root-tag gate (only placeholder/multipath/taptree
    /// validators run), so `encode_payload` of an AndV-rooted descriptor
    /// succeeds and round-trips through `decode_payload` exposes the gap.
    #[test]
    fn decode_rejects_non_canonical_root_tag() {
        // The TopLevel check fires in `decode_payload` before any downstream
        // validator runs, so this test reaches the rejection regardless of
        // whether path_decl would satisfy `validate_explicit_origin_required`
        // (it does, but the check is short-circuited above). path_decl is
        // populated here to mirror a realistic descriptor shape.
        let d = Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(OriginPath {
                    components: vec![PathComponent {
                        hardened: true,
                        value: 84,
                    }],
                }),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::AndV,
                body: Body::Children(vec![
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    },
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    },
                ]),
            },
            tlv: TlvSection::new_empty(),
        };
        let (bytes, total_bits) = encode_payload(&d).expect("encode AndV-rooted ok");
        let err = decode_payload(&bytes, total_bits).expect_err("decode must reject");
        assert!(
            matches!(
                err,
                Error::OperatorContextViolation {
                    tag: Tag::AndV,
                    context: ContextKind::TopLevel,
                }
            ),
            "expected OperatorContextViolation{{TopLevel}}, got {err:?}"
        );
    }
}
