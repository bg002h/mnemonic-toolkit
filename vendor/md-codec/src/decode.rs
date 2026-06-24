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

/// Decode a Descriptor from the canonical payload bit stream.
/// `bytes` may be zero-padded; `total_bits` is the exact payload bit count.
pub fn decode_payload(bytes: &[u8], total_bits: usize) -> Result<Descriptor, Error> {
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
    crate::validate::validate_explicit_origin_required(&descriptor)?;
    crate::validate::validate_xpub_bytes(&descriptor)?;

    Ok(descriptor)
}

/// Decode a Descriptor from a complete codex32 md1 string.
///
/// Uses the symbol-aligned bit count returned by `unwrap_string` (5 × symbol_count),
/// which is exact at the codex32 layer with ≤4 bits of trailing zero-padding —
/// well within the v11 decoder's TLV-rollback tolerance.
pub fn decode_md1_string(s: &str) -> Result<Descriptor, Error> {
    let (bytes, symbol_aligned_bit_count) = crate::codex32::unwrap_string(s)?;
    decode_payload(&bytes, symbol_aligned_bit_count)
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
