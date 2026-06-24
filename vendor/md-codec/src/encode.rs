//! Top-level encoder per spec §13.3.

use crate::bitstream::BitWriter;
use crate::error::Error;
use crate::header::Header;
use crate::origin_path::{PathDecl, PathDeclPaths};
use crate::tlv::TlvSection;
use crate::tree::{Body, Node, write_node};
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

    let mut w = BitWriter::new();
    let header = Header {
        version: Header::WF_REDESIGN_VERSION,
        divergent_paths: matches!(d.path_decl.paths, PathDeclPaths::Divergent(_)),
    };
    header.write(&mut w);
    d.path_decl.write(&mut w)?;
    d.use_site_path.write(&mut w)?;
    let kiw = d.key_index_width();
    write_node(&mut w, &d.tree, kiw)?;
    d.tlv.write(&mut w, kiw)?;
    let total_bits = w.bit_len();
    Ok((w.into_bytes(), total_bits))
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
