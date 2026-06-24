//! TLV section per spec §3.7 (extended in v0.13 §3.2 with `Pubkeys` and
//! `OriginPathOverrides`).

use crate::bitstream::{BitReader, BitWriter, re_emit_bits};
use crate::error::Error;
use crate::origin_path::OriginPath;
use crate::use_site_path::UseSitePath;
use crate::varint::{read_varint, write_varint};

/// TLV tag for use-site-path overrides (per-`@N` divergent path declarations).
pub const TLV_USE_SITE_PATH_OVERRIDES: u8 = 0x00;
/// TLV tag for per-`@N` xpub fingerprints (4 bytes each).
pub const TLV_FINGERPRINTS: u8 = 0x01;
/// TLV tag for per-`@N` xpub bytes (chain-code || compressed pubkey, 65 bytes
/// each). Per v0.13 §3.2; supersedes the v0.12 reservation `TLV_XPUBS_RESERVED_V0_12`.
pub const TLV_PUBKEYS: u8 = 0x02;
/// TLV tag for per-`@N` origin-path overrides (BIP-32 path differing from the
/// canonical default for the wrapper). Per v0.13 §3.2.
pub const TLV_ORIGIN_PATH_OVERRIDES: u8 = 0x03;

/// Decoded TLV section. Fields are populated from per-tag readers; unknown
/// tags are preserved verbatim per D6 forward-compat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlvSection {
    /// Per-`@N` use-site path overrides, if present.
    pub use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>,
    /// Per-`@N` xpub fingerprints (4 bytes each), if present.
    pub fingerprints: Option<Vec<(u8, [u8; 4])>>,
    /// Per-`@N` xpub bytes (32-byte chain code || 33-byte compressed pubkey),
    /// if present. Wallet-policy mode predicate is `pubkeys.is_some() &&
    /// !pubkeys.unwrap().is_empty()`.
    pub pubkeys: Option<Vec<(u8, [u8; 65])>>,
    /// Per-`@N` origin-path overrides for wrappers whose canonical path is
    /// either undefined or has been overridden, if present.
    pub origin_path_overrides: Option<Vec<(u8, OriginPath)>>,
    /// Raw payload of unknown TLVs, keyed by tag, for forward-compat round-trip.
    /// Decoders preserve unknown TLVs verbatim through re-encoding.
    pub unknown: Vec<(u8, Vec<u8>, usize)>,
}

impl TlvSection {
    /// Create an empty TLV section (no entries).
    pub fn new_empty() -> Self {
        // Exhaustive struct construction — every field listed by name. If a
        // future field is added, this initializer fails to compile until the
        // author decides on its empty value, preventing accidental drift.
        Self {
            use_site_path_overrides: None,
            fingerprints: None,
            pubkeys: None,
            origin_path_overrides: None,
            unknown: Vec::new(),
        }
    }

    /// Returns true if no TLV entries are present.
    pub fn is_empty(&self) -> bool {
        // Exhaustive destructure — adding a new field forces this method to
        // be updated (compile error on missing pattern).
        let Self {
            use_site_path_overrides,
            fingerprints,
            pubkeys,
            origin_path_overrides,
            unknown,
        } = self;
        use_site_path_overrides.is_none()
            && fingerprints.is_none()
            && pubkeys.is_none()
            && origin_path_overrides.is_none()
            && unknown.is_empty()
    }

    /// Encode this TLV section onto `w`. Entries are emitted in ascending tag order.
    /// `key_index_width` is the bit-width of the per-`@N` placeholder index field.
    ///
    /// # Errors
    ///
    /// - [`Error::EmptyTlvEntry`] if any of `use_site_path_overrides`,
    ///   `fingerprints`, `pubkeys`, or `origin_path_overrides` is `Some(vec![])`.
    ///   Empty TLVs violate the §7.5 omission discipline and are rejected at the
    ///   encoder boundary.
    /// - [`Error::OverrideOrderViolation`] if any entry vector is not strictly
    ///   ascending in `idx`.
    /// - Encoding errors from contained values (`OriginPath::write`, etc.).
    pub fn write(&self, w: &mut BitWriter, key_index_width: u8) -> Result<(), Error> {
        // Exhaustive destructure — same drift-protection guarantee as is_empty.
        let Self {
            use_site_path_overrides,
            fingerprints,
            pubkeys,
            origin_path_overrides,
            unknown,
        } = self;

        // Collect entries, sort by tag.
        let mut entries: Vec<(u8, Vec<u8>, usize)> = Vec::new();

        if let Some(overrides) = use_site_path_overrides {
            if overrides.is_empty() {
                return Err(Error::EmptyTlvEntry {
                    tag: TLV_USE_SITE_PATH_OVERRIDES,
                });
            }
            let mut sub = BitWriter::new();
            let mut last_idx: Option<u8> = None;
            for (idx, path) in overrides {
                if let Some(prev) = last_idx {
                    if *idx <= prev {
                        return Err(Error::OverrideOrderViolation {
                            prev,
                            current: *idx,
                        });
                    }
                }
                last_idx = Some(*idx);
                sub.write_bits(u64::from(*idx), key_index_width as usize);
                path.write(&mut sub)?;
            }
            let bit_len = sub.bit_len();
            entries.push((TLV_USE_SITE_PATH_OVERRIDES, sub.into_bytes(), bit_len));
        }
        if let Some(fps) = fingerprints {
            if fps.is_empty() {
                return Err(Error::EmptyTlvEntry {
                    tag: TLV_FINGERPRINTS,
                });
            }
            let mut sub = BitWriter::new();
            let mut last_idx: Option<u8> = None;
            for (idx, fp) in fps {
                if let Some(prev) = last_idx {
                    if *idx <= prev {
                        return Err(Error::OverrideOrderViolation {
                            prev,
                            current: *idx,
                        });
                    }
                }
                last_idx = Some(*idx);
                sub.write_bits(u64::from(*idx), key_index_width as usize);
                for b in fp {
                    sub.write_bits(u64::from(*b), 8);
                }
            }
            let bit_len = sub.bit_len();
            entries.push((TLV_FINGERPRINTS, sub.into_bytes(), bit_len));
        }
        if let Some(pks) = pubkeys {
            if pks.is_empty() {
                return Err(Error::EmptyTlvEntry { tag: TLV_PUBKEYS });
            }
            let mut sub = BitWriter::new();
            let mut last_idx: Option<u8> = None;
            for (idx, xpub) in pks {
                if let Some(prev) = last_idx {
                    if *idx <= prev {
                        return Err(Error::OverrideOrderViolation {
                            prev,
                            current: *idx,
                        });
                    }
                }
                last_idx = Some(*idx);
                sub.write_bits(u64::from(*idx), key_index_width as usize);
                for b in xpub {
                    sub.write_bits(u64::from(*b), 8);
                }
            }
            let bit_len = sub.bit_len();
            entries.push((TLV_PUBKEYS, sub.into_bytes(), bit_len));
        }
        if let Some(paths) = origin_path_overrides {
            if paths.is_empty() {
                return Err(Error::EmptyTlvEntry {
                    tag: TLV_ORIGIN_PATH_OVERRIDES,
                });
            }
            let mut sub = BitWriter::new();
            let mut last_idx: Option<u8> = None;
            for (idx, path) in paths {
                if let Some(prev) = last_idx {
                    if *idx <= prev {
                        return Err(Error::OverrideOrderViolation {
                            prev,
                            current: *idx,
                        });
                    }
                }
                last_idx = Some(*idx);
                sub.write_bits(u64::from(*idx), key_index_width as usize);
                path.write(&mut sub)?;
            }
            let bit_len = sub.bit_len();
            entries.push((TLV_ORIGIN_PATH_OVERRIDES, sub.into_bytes(), bit_len));
        }
        for (tag, payload, bit_len) in unknown {
            entries.push((*tag, payload.clone(), *bit_len));
        }
        entries.sort_by_key(|(t, _, _)| *t);

        for (tag, payload, bit_len) in entries {
            w.write_bits(u64::from(tag), 5);
            write_varint(w, bit_len as u32)?;
            re_emit_bits(w, &payload, bit_len)?;
        }
        Ok(())
    }

    /// Decode a TLV section from `r`, consuming all remaining bits.
    /// `key_index_width` is the bit-width of placeholder indices; `n` is the key count.
    pub fn read(r: &mut BitReader, key_index_width: u8, n: u8) -> Result<Self, Error> {
        let mut section = Self::new_empty();
        let mut last_tag: Option<u8> = None;
        loop {
            // Save position so we can roll back if this would-be TLV is
            // actually trailing codex32-padding (≤7 bits of zeros).
            let entry_start = r.save_position();
            if r.remaining_bits() < 5 {
                break; // not enough bits for even a tag — clean end-of-stream
            }
            // Try to parse a complete TLV entry. Any failure (truncated read,
            // ordering violation, empty-entry-by-spec, length exceeds remaining)
            // is treated as "trailing padding" if we can rollback cleanly. If
            // rollback would consume <8 bits (consistent with codex32 padding)
            // we accept it; otherwise the error propagates as a real malformed
            // input.
            let parse_result: Result<(), Error> = (|| {
                let tag = r.read_bits(5)? as u8;
                // Ordering check is INSIDE the closure so violations at end-of-
                // stream (where padding bits form a phantom tag=0 after a real
                // tag≥1 entry) become rollback-eligible.
                if let Some(prev) = last_tag {
                    if tag <= prev {
                        return Err(Error::TlvOrderingViolation { prev, current: tag });
                    }
                }
                let bit_len = read_varint(r)? as usize;
                if bit_len > r.remaining_bits() {
                    return Err(Error::TlvLengthExceedsRemaining {
                        length: bit_len,
                        remaining: r.remaining_bits(),
                    });
                }
                // Reject zero-length TLVs uniformly. Encoder MUST omit empty
                // TLVs per spec §7.5; a zero-length entry at the end of stream
                // is treated as padding via the rollback path.
                if bit_len == 0 {
                    return Err(Error::EmptyTlvEntry { tag });
                }
                match tag {
                    TLV_USE_SITE_PATH_OVERRIDES => {
                        let entry = read_use_site_overrides(r, bit_len, key_index_width, n)?;
                        section.use_site_path_overrides = Some(entry);
                    }
                    TLV_FINGERPRINTS => {
                        let entry = read_fingerprints(r, bit_len, key_index_width, n)?;
                        section.fingerprints = Some(entry);
                    }
                    TLV_PUBKEYS => {
                        let entry = read_pubkeys(r, bit_len, key_index_width, n)?;
                        section.pubkeys = Some(entry);
                    }
                    TLV_ORIGIN_PATH_OVERRIDES => {
                        let entry = read_origin_path_overrides(r, bit_len, key_index_width, n)?;
                        section.origin_path_overrides = Some(entry);
                    }
                    _ => {
                        // Unknown — buffer and skip per D6 forward-compat.
                        let mut sub = BitWriter::new();
                        let mut remaining = bit_len;
                        while remaining > 0 {
                            let chunk = remaining.min(8);
                            let bits = r.read_bits(chunk)?;
                            sub.write_bits(bits, chunk);
                            remaining -= chunk;
                        }
                        let payload = sub.into_bytes();
                        section.unknown.push((tag, payload, bit_len));
                    }
                }
                last_tag = Some(tag);
                Ok(())
            })();

            match parse_result {
                Ok(()) => continue,
                Err(e) => {
                    // Decide: rollback-as-padding or propagate error.
                    // Rollback is acceptable iff the bits we'd be discarding
                    // are ≤7 (consistent with codex32 padding boundary).
                    r.restore_position(entry_start);
                    let remaining_at_entry_start = r.remaining_bits();
                    // Padding tolerance: ≤7 bits of trailing zeros after the
                    // last real TLV (or after the tree if no TLVs were emitted).
                    if remaining_at_entry_start <= 7 {
                        break;
                    }
                    // More than 7 bits remained but the parse still failed —
                    // this is genuinely malformed input. Propagate.
                    return Err(e);
                }
            }
        }
        Ok(section)
    }
}

/// Read one sparse `(idx, ...)` index header field: a `key_index_width`-bit
/// `idx`, range-checked against `n`, and (if `last_idx.is_some()`) verified
/// to be strictly greater than the previous idx. Returns the raw idx for
/// the caller to thread into `last_idx` on the next call.
///
/// Used by every sparse-TLV reader (use-site-path overrides, fingerprints,
/// pubkeys, origin-path overrides) so the range/ordering invariants are
/// enforced uniformly in one place.
fn read_sparse_tlv_idx(
    r: &mut BitReader,
    key_index_width: u8,
    n: u8,
    last_idx: Option<u8>,
) -> Result<u8, Error> {
    let idx = r.read_bits(key_index_width as usize)? as u8;
    if idx >= n {
        return Err(Error::PlaceholderIndexOutOfRange { idx, n });
    }
    if let Some(prev) = last_idx {
        if idx <= prev {
            return Err(Error::OverrideOrderViolation { prev, current: idx });
        }
    }
    Ok(idx)
}

/// Generic sparse-TLV body reader.
///
/// **Per spec v0.13 §3.2 + audit follow-up L3 (v0.13.1):** bounds the
/// `BitReader`'s `bit_limit` to `start + bit_len` for the duration of
/// the body loop. This prevents a malformed wire from silently
/// advancing the outer reader's cursor past the declared body boundary
/// — any over-read errors with `BitStreamTruncated` instead of
/// quietly consuming bits from the next TLV. On error, the inner
/// error variant is propagated as-is (no translation) since the same
/// failure mode is meaningful regardless of whether the offending bits
/// were intended as a real record or as trailing slack.
///
/// On success: empty-entries-vec → [`Error::EmptyTlvEntry`].
fn read_sparse_tlv_body<T, F>(
    r: &mut BitReader,
    bit_len: usize,
    tag: u8,
    key_index_width: u8,
    n: u8,
    mut read_value: F,
) -> Result<Vec<(u8, T)>, Error>
where
    F: FnMut(&mut BitReader) -> Result<T, Error>,
{
    let start = r.bit_position();
    let saved_limit = r.save_bit_limit();
    r.set_bit_limit_for_scope(start + bit_len);

    let mut entries: Vec<(u8, T)> = Vec::new();
    let mut last_idx: Option<u8> = None;

    let result = (|| -> Result<(), Error> {
        while r.bit_position() - start < bit_len {
            let idx = read_sparse_tlv_idx(r, key_index_width, n, last_idx)?;
            let value = read_value(r)?;
            last_idx = Some(idx);
            entries.push((idx, value));
        }
        Ok(())
    })();

    r.restore_bit_limit(saved_limit);
    result?;

    if entries.is_empty() {
        return Err(Error::EmptyTlvEntry { tag });
    }
    Ok(entries)
}

fn read_use_site_overrides(
    r: &mut BitReader,
    bit_len: usize,
    key_index_width: u8,
    n: u8,
) -> Result<Vec<(u8, UseSitePath)>, Error> {
    read_sparse_tlv_body(
        r,
        bit_len,
        TLV_USE_SITE_PATH_OVERRIDES,
        key_index_width,
        n,
        UseSitePath::read,
    )
}

fn read_fingerprints(
    r: &mut BitReader,
    bit_len: usize,
    key_index_width: u8,
    n: u8,
) -> Result<Vec<(u8, [u8; 4])>, Error> {
    read_sparse_tlv_body(r, bit_len, TLV_FINGERPRINTS, key_index_width, n, |r| {
        let mut fp = [0u8; 4];
        for byte in &mut fp {
            *byte = r.read_bits(8)? as u8;
        }
        Ok(fp)
    })
}

fn read_pubkeys(
    r: &mut BitReader,
    bit_len: usize,
    key_index_width: u8,
    n: u8,
) -> Result<Vec<(u8, [u8; 65])>, Error> {
    read_sparse_tlv_body(r, bit_len, TLV_PUBKEYS, key_index_width, n, |r| {
        let mut xpub = [0u8; 65];
        for byte in &mut xpub {
            *byte = r.read_bits(8)? as u8;
        }
        Ok(xpub)
    })
}

fn read_origin_path_overrides(
    r: &mut BitReader,
    bit_len: usize,
    key_index_width: u8,
    n: u8,
) -> Result<Vec<(u8, OriginPath)>, Error> {
    // OriginPath::read is self-delimiting (depth field + that-many
    // components) — it terminates without needing an outer length cue.
    read_sparse_tlv_body(
        r,
        bit_len,
        TLV_ORIGIN_PATH_OVERRIDES,
        key_index_width,
        n,
        OriginPath::read,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::origin_path::PathComponent;

    #[test]
    fn empty_tlv_section_round_trip() {
        let s = TlvSection::new_empty();
        assert!(s.is_empty());
        let mut w = BitWriter::new();
        s.write(&mut w, 2).unwrap();
        assert_eq!(w.bit_len(), 0);
    }

    #[test]
    fn use_site_path_override_round_trip() {
        let mut s = TlvSection::new_empty();
        s.use_site_path_overrides = Some(vec![(
            1u8,
            UseSitePath {
                multipath: None,
                wildcard_hardened: true,
            },
        )]);
        let mut w = BitWriter::new();
        s.write(&mut w, 2).unwrap();
        let bit_len = w.bit_len();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        let s2 = TlvSection::read(&mut r, 2, 3).unwrap();
        assert_eq!(s2, s);
        assert_eq!(r.bit_position(), bit_len);
    }

    #[test]
    fn fingerprint_round_trip() {
        let mut s = TlvSection::new_empty();
        s.fingerprints = Some(vec![
            (0u8, [0xaa, 0xbb, 0xcc, 0xdd]),
            (2u8, [0x11, 0x22, 0x33, 0x44]),
        ]);
        let mut w = BitWriter::new();
        s.write(&mut w, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        let s2 = TlvSection::read(&mut r, 2, 3).unwrap();
        assert_eq!(s2, s);
    }

    #[test]
    fn pubkeys_round_trip() {
        // Build two distinguishable 65-byte payloads.
        let mut xpub_a = [0u8; 65];
        for (i, b) in xpub_a.iter_mut().enumerate() {
            *b = i as u8;
        }
        let mut xpub_b = [0u8; 65];
        for (i, b) in xpub_b.iter_mut().enumerate() {
            *b = (0xff - i as u8) ^ 0x5a;
        }
        let mut s = TlvSection::new_empty();
        s.pubkeys = Some(vec![(0u8, xpub_a), (2u8, xpub_b)]);

        let mut w = BitWriter::new();
        s.write(&mut w, 2).unwrap();
        let bit_len = w.bit_len();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        let s2 = TlvSection::read(&mut r, 2, 3).unwrap();
        assert_eq!(s2, s);
        assert_eq!(r.bit_position(), bit_len);
    }

    #[test]
    fn origin_path_overrides_round_trip() {
        // Two distinct origin paths at idx 0 and idx 1.
        let bip84 = OriginPath {
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
                    value: 5,
                },
            ],
        };
        let bip48 = OriginPath {
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
        };
        let mut s = TlvSection::new_empty();
        s.origin_path_overrides = Some(vec![(0u8, bip84), (1u8, bip48)]);

        let mut w = BitWriter::new();
        s.write(&mut w, 2).unwrap();
        let bit_len = w.bit_len();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        let s2 = TlvSection::read(&mut r, 2, 3).unwrap();
        assert_eq!(s2, s);
        assert_eq!(r.bit_position(), bit_len);
    }

    #[test]
    fn ascending_tag_order_enforced_in_encoder() {
        // All four sparse TLVs populated; first-on-the-wire must be tag 0x00.
        let mut s = TlvSection::new_empty();
        s.use_site_path_overrides = Some(vec![(
            0,
            UseSitePath {
                multipath: None,
                wildcard_hardened: false,
            },
        )]);
        s.fingerprints = Some(vec![(0, [0u8; 4])]);
        s.pubkeys = Some(vec![(0, [0u8; 65])]);
        s.origin_path_overrides = Some(vec![(
            0,
            OriginPath {
                components: vec![PathComponent {
                    hardened: true,
                    value: 84,
                }],
            },
        )]);
        let mut w = BitWriter::new();
        s.write(&mut w, 2).unwrap();
        let bytes = w.into_bytes();
        let first_tag = (bytes[0] >> 3) & 0x1F;
        assert_eq!(first_tag, TLV_USE_SITE_PATH_OVERRIDES);
    }

    #[test]
    fn pubkeys_ordering_violation_rejected_at_encoder() {
        // Non-ascending idx pair (1, 0) → encoder must reject.
        let mut s = TlvSection::new_empty();
        s.pubkeys = Some(vec![(1u8, [0u8; 65]), (0u8, [0u8; 65])]);
        let mut w = BitWriter::new();
        let result = s.write(&mut w, 2);
        assert!(matches!(
            result,
            Err(Error::OverrideOrderViolation {
                prev: 1,
                current: 0
            })
        ));
    }

    #[test]
    fn pubkeys_ordering_violation_rejected_at_decoder() {
        // Encode (0, [0;65]) then a deliberately mis-ordered (0, [0;65]) by
        // hand-building the bytes — exercises the read_sparse_tlv_idx
        // helper's ascending check on the read side.
        let mut sub = BitWriter::new();
        // idx=1 (2-bit width) then 65 zero bytes.
        sub.write_bits(1, 2);
        for _ in 0..65 {
            sub.write_bits(0, 8);
        }
        // idx=1 again → ordering violation (1 not > 1).
        sub.write_bits(1, 2);
        for _ in 0..65 {
            sub.write_bits(0, 8);
        }
        let bit_len = sub.bit_len();
        let payload_bytes = sub.into_bytes();

        let mut w = BitWriter::new();
        w.write_bits(u64::from(TLV_PUBKEYS), 5);
        write_varint(&mut w, bit_len as u32).unwrap();
        re_emit_bits(&mut w, &payload_bytes, bit_len).unwrap();
        let total_bit_len = w.bit_len();
        let bytes = w.into_bytes();

        let mut r = BitReader::with_bit_limit(&bytes, total_bit_len);
        let result = TlvSection::read(&mut r, 2, 3);
        assert!(matches!(
            result,
            Err(Error::OverrideOrderViolation {
                prev: 1,
                current: 1
            })
        ));
    }

    #[test]
    fn read_sparse_tlv_idx_out_of_range() {
        // Build 2-bit idx=3 with n=2 → out of range.
        let mut sub = BitWriter::new();
        sub.write_bits(3, 2);
        let bit_len = sub.bit_len();
        let bytes = sub.into_bytes();
        let mut r = BitReader::with_bit_limit(&bytes, bit_len);

        let result = read_sparse_tlv_idx(&mut r, 2, 2, None);
        assert!(matches!(
            result,
            Err(Error::PlaceholderIndexOutOfRange { idx: 3, n: 2 })
        ));
    }

    #[test]
    fn read_sparse_tlv_idx_non_ascending() {
        let mut sub = BitWriter::new();
        sub.write_bits(0, 2);
        let bit_len = sub.bit_len();
        let bytes = sub.into_bytes();
        let mut r = BitReader::with_bit_limit(&bytes, bit_len);

        let result = read_sparse_tlv_idx(&mut r, 2, 3, Some(1));
        assert!(matches!(
            result,
            Err(Error::OverrideOrderViolation {
                prev: 1,
                current: 0
            })
        ));
    }

    #[test]
    fn empty_pubkeys_vec_rejected_at_encoder() {
        let mut s = TlvSection::new_empty();
        s.pubkeys = Some(Vec::new());
        let mut w = BitWriter::new();
        let result = s.write(&mut w, 2);
        assert!(matches!(
            result,
            Err(Error::EmptyTlvEntry { tag }) if tag == TLV_PUBKEYS
        ));
    }

    #[test]
    fn empty_origin_path_overrides_vec_rejected_at_encoder() {
        let mut s = TlvSection::new_empty();
        s.origin_path_overrides = Some(Vec::new());
        let mut w = BitWriter::new();
        let result = s.write(&mut w, 2);
        assert!(matches!(
            result,
            Err(Error::EmptyTlvEntry { tag }) if tag == TLV_ORIGIN_PATH_OVERRIDES
        ));
    }

    // ─── Strict bit_len enforcement (v0.13.1, audit L3) ───────────────

    /// Hand-craft a single-TLV wire with one inflated `bit_len`. Returns
    /// the bytes and the total bit count for `BitReader::with_bit_limit`.
    fn craft_inflated_tlv_wire(
        tag: u8,
        idx: u8,
        idx_width: u8,
        record_payload_bits: &[(u64, usize)],
        slack_bits: usize,
    ) -> (Vec<u8>, usize) {
        let mut w = BitWriter::new();
        // Tag (5 bits).
        w.write_bits(u64::from(tag), 5);
        // bit_len (LP4-ext varint) — declares the actual records' bits + slack.
        let actual_record_bits: usize =
            (idx_width as usize) + record_payload_bits.iter().map(|(_, n)| n).sum::<usize>();
        let declared_bit_len = actual_record_bits + slack_bits;
        write_varint(&mut w, declared_bit_len as u32).unwrap();
        // Records: idx + payload.
        w.write_bits(u64::from(idx), idx_width as usize);
        for (val, bits) in record_payload_bits {
            w.write_bits(*val, *bits);
        }
        // Append slack zero-bits.
        for _ in 0..slack_bits {
            w.write_bits(0, 1);
        }
        let bit_len = w.bit_len();
        (w.into_bytes(), bit_len)
    }

    // The four tests below exercise the L3 audit concern: a wire that
    // declares more `bit_len` than its records actually carry must be
    // rejected, with no silent advancement of the outer reader's
    // cursor past the declared body. The specific error variant depends
    // on the slack-bit pattern (typically `OverrideOrderViolation` when
    // slack starts with zero and the previous idx was 0, or
    // `BitStreamTruncated` when slack is too short for a phantom idx).
    // The contract under test is "rejection happens," not the variant
    // name. The `bit_limit` bound inside `read_sparse_tlv_body` is the
    // load-bearing fix.

    #[test]
    fn fingerprints_with_trailing_slack_rejected() {
        let (bytes, total_bits) =
            craft_inflated_tlv_wire(TLV_FINGERPRINTS, 0, 1, &[(0xDEAD_BEEF, 32)], 4);
        let mut r = BitReader::with_bit_limit(&bytes, total_bits);
        let result = TlvSection::read(&mut r, 1, 1);
        assert!(
            result.is_err(),
            "trailing slack must be rejected, got {:?}",
            result
        );
    }

    #[test]
    fn pubkeys_with_trailing_slack_rejected() {
        let payload: Vec<(u64, usize)> = (0..65).map(|_i| (0x42u64, 8)).collect();
        let (bytes, total_bits) = craft_inflated_tlv_wire(TLV_PUBKEYS, 0, 1, &payload, 3);
        let mut r = BitReader::with_bit_limit(&bytes, total_bits);
        let result = TlvSection::read(&mut r, 1, 1);
        assert!(
            result.is_err(),
            "trailing slack must be rejected, got {:?}",
            result
        );
    }

    #[test]
    fn use_site_path_overrides_with_trailing_slack_rejected() {
        let mut path_w = BitWriter::new();
        UseSitePath::standard_multipath()
            .write(&mut path_w)
            .unwrap();
        let path_bit_len = path_w.bit_len();
        let path_bytes = path_w.into_bytes();
        let mut path_record: Vec<(u64, usize)> = Vec::new();
        let mut br = BitReader::new(&path_bytes);
        let mut consumed = 0;
        while consumed < path_bit_len {
            let chunk = (path_bit_len - consumed).min(8);
            path_record.push((br.read_bits(chunk).unwrap(), chunk));
            consumed += chunk;
        }
        let (bytes, total_bits) =
            craft_inflated_tlv_wire(TLV_USE_SITE_PATH_OVERRIDES, 0, 1, &path_record, 2);
        let mut r = BitReader::with_bit_limit(&bytes, total_bits);
        let result = TlvSection::read(&mut r, 1, 1);
        assert!(
            result.is_err(),
            "trailing slack must be rejected, got {:?}",
            result
        );
    }

    #[test]
    fn origin_path_overrides_with_trailing_slack_rejected() {
        let (bytes, total_bits) =
            craft_inflated_tlv_wire(TLV_ORIGIN_PATH_OVERRIDES, 0, 1, &[(0, 4)], 5);
        let mut r = BitReader::with_bit_limit(&bytes, total_bits);
        let result = TlvSection::read(&mut r, 1, 1);
        assert!(
            result.is_err(),
            "trailing slack must be rejected, got {:?}",
            result
        );
    }
}
