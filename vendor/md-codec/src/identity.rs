//! Identity computation per spec §8.

use crate::bitstream::{BitWriter, re_emit_bits};
use crate::canonicalize::{canonicalize_placeholder_indices, expand_per_at_n};
use crate::encode::{Descriptor, encode_payload};
use crate::error::Error;
use crate::phrase::Phrase;
use crate::varint::write_varint;
use bitcoin::hashes::{Hash, sha256};

/// 128-bit canonical identifier for an md1 encoding (spec §8).
///
/// Computed as the first 16 bytes of `SHA-256` over the canonical
/// bit-packed payload bytes produced by [`encode_payload`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Md1EncodingId([u8; 16]);

impl Md1EncodingId {
    /// Construct from a raw 16-byte array.
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Borrow the underlying 16-byte identifier.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Return the 4-byte fingerprint (first 4 bytes of the id).
    pub fn fingerprint(&self) -> [u8; 4] {
        let mut fp = [0u8; 4];
        fp.copy_from_slice(&self.0[0..4]);
        fp
    }
}

/// Compute the [`Md1EncodingId`] for a descriptor by hashing its canonical
/// bit-packed payload encoding (spec §8).
pub fn compute_md1_encoding_id(d: &Descriptor) -> Result<Md1EncodingId, Error> {
    let (bytes, _bit_len) = encode_payload(d)?;
    let hash = sha256::Hash::hash(&bytes);
    let mut id = [0u8; 16];
    id.copy_from_slice(&hash.to_byte_array()[0..16]);
    Ok(Md1EncodingId(id))
}

/// 128-bit BIP 388 wallet-descriptor-template identifier (spec §8.1, γ-flavor).
///
/// Hashes ONLY the BIP 388 template content: use-site-path-decl bits, tree
/// bits, and the `UseSitePathOverrides` TLV entry bits when present. Excludes
/// the header, origin-path-decl, `Fingerprints` TLV, HRP, and BCH checksum,
/// so it is invariant to origin-path changes (e.g. account index) and to
/// fingerprint additions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WalletDescriptorTemplateId([u8; 16]);

impl WalletDescriptorTemplateId {
    /// Construct from a raw 16-byte array.
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Borrow the underlying 16-byte identifier.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// Compute the [`WalletDescriptorTemplateId`] for a descriptor by hashing only
/// the BIP 388 template content per spec §8.1.
pub fn compute_wallet_descriptor_template_id(
    d: &Descriptor,
) -> Result<WalletDescriptorTemplateId, Error> {
    // L15: canonicalize placeholder ordering on a clone first (mirror
    // compute_wallet_policy_id) so the WDT-id is invariant to placeholder
    // index permutation. The identity fast-path leaves already-canonical
    // inputs (the toolkit's @0,@1,… ordering) byte-identical.
    let mut d_canonical = d.clone();
    canonicalize_placeholder_indices(&mut d_canonical)?;
    let d = &d_canonical;
    let mut w = BitWriter::new();
    // Per spec §8.1: use-site-path-decl bits || tree bits || UseSitePathOverrides TLV bits
    let kiw = d.key_index_width();
    d.use_site_path.write(&mut w)?;
    crate::tree::write_node(&mut w, &d.tree, kiw)?;
    if let Some(overrides) = &d.tlv.use_site_path_overrides {
        // Re-encode the UseSitePathOverrides TLV ENTRY (tag + length + payload).
        let mut sub = BitWriter::new();
        for (idx, path) in overrides {
            sub.write_bits(u64::from(*idx), kiw as usize);
            path.write(&mut sub)?;
        }
        let bit_len = sub.bit_len();
        w.write_bits(u64::from(crate::tlv::TLV_USE_SITE_PATH_OVERRIDES), 5);
        crate::varint::write_varint(&mut w, bit_len as u32)?;
        let payload = sub.into_bytes();
        let mut subr = crate::bitstream::BitReader::new(&payload);
        let mut remaining = bit_len;
        while remaining > 0 {
            let chunk = remaining.min(8);
            let bits = subr.read_bits(chunk)?;
            w.write_bits(bits, chunk);
            remaining -= chunk;
        }
    }
    let bytes = w.into_bytes();
    let hash = sha256::Hash::hash(&bytes);
    let mut id = [0u8; 16];
    id.copy_from_slice(&hash.to_byte_array()[0..16]);
    Ok(WalletDescriptorTemplateId(id))
}

/// 128-bit canonical wallet-policy identifier (spec v0.13 §5.3).
///
/// Hashes the canonical-expanded BIP 388 wallet *policy* — template tree
/// plus per-`@N` origin / use-site / fp / xpub records — so that two
/// engravings of the same logical wallet produce identical IDs whether
/// they elide canonical paths or write them out explicitly. Stable
/// across origin- and use-site-elision; presence-significant on
/// fingerprint and xpub axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WalletPolicyId([u8; 16]);

impl WalletPolicyId {
    /// Construct from a raw 16-byte array.
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Borrow the underlying 16-byte identifier.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Render this identifier as a 12-word BIP 39 phrase (spec §8.4).
    pub fn to_phrase(&self) -> Result<Phrase, Error> {
        Phrase::from_id_bytes(self.as_bytes())
    }
}

/// Compute the [`WalletPolicyId`] for a descriptor by hashing its
/// canonical-expanded wallet-policy preimage per spec v0.13 §5.3.
///
/// Construction (byte-exact, no encoder divergence):
///
/// 1. Canonicalize placeholder indices on a clone of `d` (Phase 3a) —
///    callers don't need to remember the precondition.
/// 2. Compute `canonical_template_tree_bytes` by writing the
///    placeholder-form tree via [`crate::tree::write_node`] into a fresh
///    [`BitWriter`] and finalizing (zero-pad to whole-byte boundary).
/// 3. Expand to per-`@N` records via [`expand_per_at_n`] (Phase 3b).
/// 4. For each record (idx-ascending), allocate a fresh `BitWriter`,
///    write `path_bit_len` (LP4-ext varint, in *bits*), then re-emit
///    the path's bits MSB-first via [`re_emit_bits`]; same for the
///    use-site path. Finalize the bitstream — single byte-boundary pad.
/// 5. Build `presence_byte = (fp_present | (xpub_present << 1)) &
///    0b0000_0011` (explicit reserved-bit mask) and concatenate
///    `presence_byte || record_bytes || fp? || xpub?`.
/// 6. Hash input = `canonical_template_tree_bytes || concat(records)`.
/// 7. Return `SHA-256(input)[0..16]`.
///
/// # Errors
///
/// Propagates [`Error::MissingExplicitOrigin`] from [`expand_per_at_n`]
/// for non-canonical wrappers without an explicit origin path; other
/// canonicalization or encoding errors as appropriate.
///
/// # INVARIANT (Option A, spec v0.13 §3 + §5.3)
///
/// `path_decl.paths` is always populated post-decode (v0.11 wire
/// invariant). Canonical-fill into `path_decl` happens at encode time
/// only (per spec §6.3). For a decoded wire this function therefore
/// reads `OriginPathOverrides[idx]` if present, else `path_decl.paths`
/// resolved per the divergent_paths flag, via [`expand_per_at_n`].
///
/// L14 (cycle-10): for an in-memory `Descriptor` built with an ELIDED
/// (empty-components) origin — which `expand_per_at_n` surfaces as an
/// empty `e.origin_path` — this function canonical-fills that single
/// empty case from [`crate::canonical_origin::canonical_origin`] so the
/// policy-id honors its documented "stable across origin-elision"
/// invariant (an elided origin hashes identically to the explicit form).
/// Decoded wires are unaffected (their `path_decl` is always populated,
/// so the empty-origin branch is never taken). Any future change that
/// elides `path_decl` on the wire would extend this canonical_origin
/// lookup to the decode path here and in [`expand_per_at_n`].
pub fn compute_wallet_policy_id(d: &Descriptor) -> Result<WalletPolicyId, Error> {
    // Step 1: canonicalize on a clone so callers don't have to remember
    // the precondition and we never mutate the caller's descriptor.
    let mut d_canonical = d.clone();
    canonicalize_placeholder_indices(&mut d_canonical)?;
    let d = &d_canonical;

    // Step 2: canonical_template_tree_bytes — placeholder-form tree only.
    let mut tree_w = BitWriter::new();
    crate::tree::write_node(&mut tree_w, &d.tree, d.key_index_width())?;
    let canonical_template_tree_bytes = tree_w.into_bytes();

    // Step 3: expand to per-@N records.
    let expanded = expand_per_at_n(d)?;

    // Step 4–5: build each canonical record and concatenate.
    let mut records_concat: Vec<u8> = Vec::new();
    for e in &expanded {
        // Origin path bits (scratch BitWriter; bit_len() captures unpadded
        // length, into_bytes() zero-pads to the next byte boundary).
        //
        // L14: canonical-fill an elided (empty) origin so the policy-id
        // honors its documented "stable across origin-elision" invariant.
        // An empty resolved origin with a canonical wrapper hashes
        // identically to the explicit form. expand_per_at_n already returns
        // explicit paths verbatim, so only the empty case needs the fill;
        // when canonical_origin is None the empty path is structurally
        // precluded HERE (expand_per_at_n's own MissingExplicitOrigin
        // gate), so the unwrap_or_else fallback is unreachable-but-safe.
        //
        // P0 update (pathless/dead-card partial-decode): partial-allowing
        // decode (`decode_payload_with_opts` / `decode_md1_string_with_opts`
        // / `chunk::reassemble_with_opts`, `allow_unresolved_origin: true`)
        // now lets a `canonical_origin(&d.tree) == None` + empty-origin
        // `Descriptor` exist in-process — the render-only callers (md-cli /
        // toolkit) query `Descriptor::unresolved_origin_indices()` on it
        // directly and never route it through `compute_wallet_policy_id`.
        // `expand_per_at_n` itself is NOT partial-aware: it is called
        // unconditionally on `d` a few lines above and still raises
        // `MissingExplicitOrigin` for any canonical_origin==None +
        // empty-origin descriptor regardless of how that `Descriptor` was
        // built or decoded (see its own doc comment). So the
        // `unwrap_or_else` fallback below stays unreachable-but-safe —
        // `compute_wallet_policy_id` remains fail-closed even though
        // partial `Descriptor`s now exist in the same process.
        let origin_for_hash = if e.origin_path.components.is_empty() {
            crate::canonical_origin::canonical_origin(&d.tree)
                .unwrap_or_else(|| e.origin_path.clone())
        } else {
            e.origin_path.clone()
        };
        let mut path_scratch = BitWriter::new();
        origin_for_hash.write(&mut path_scratch)?;
        let path_bit_len = path_scratch.bit_len();
        let path_bytes = path_scratch.into_bytes();

        // Use-site path bits.
        let mut us_scratch = BitWriter::new();
        e.use_site_path.write(&mut us_scratch)?;
        let use_site_bit_len = us_scratch.bit_len();
        let us_bytes = us_scratch.into_bytes();

        // Record bitstream: varint(path_bit_len) || path_bits ||
        // varint(use_site_bit_len) || use_site_bits, with a single
        // byte-boundary pad applied by into_bytes().
        let mut record_bw = BitWriter::new();
        write_varint(&mut record_bw, path_bit_len as u32)?;
        re_emit_bits(&mut record_bw, &path_bytes, path_bit_len)?;
        write_varint(&mut record_bw, use_site_bit_len as u32)?;
        re_emit_bits(&mut record_bw, &us_bytes, use_site_bit_len)?;
        let record_bytes = record_bw.into_bytes();

        // Presence byte: bit 0 = fp, bit 1 = xpub; reserved bits 2..7
        // are explicitly masked to 0 per spec §5.3 (forward-compat:
        // future versions that define a reserved bit must not collide
        // with v0.13's hash on the same wire).
        let fp_present = e.fingerprint.is_some();
        let xpub_present = e.xpub.is_some();
        let presence_byte = ((fp_present as u8) | ((xpub_present as u8) << 1)) & 0b0000_0011;

        records_concat.push(presence_byte);
        records_concat.extend_from_slice(&record_bytes);
        if let Some(fp) = e.fingerprint {
            records_concat.extend_from_slice(&fp);
        }
        if let Some(xpub) = e.xpub {
            records_concat.extend_from_slice(&xpub);
        }
    }

    // Step 6–7: hash and truncate.
    let mut hash_input: Vec<u8> =
        Vec::with_capacity(canonical_template_tree_bytes.len() + records_concat.len());
    hash_input.extend_from_slice(&canonical_template_tree_bytes);
    hash_input.extend_from_slice(&records_concat);
    let hash = sha256::Hash::hash(&hash_input);
    let mut id = [0u8; 16];
    id.copy_from_slice(&hash.to_byte_array()[0..16]);
    Ok(WalletPolicyId(id))
}

/// Validate a `presence_byte` from a `WalletPolicyId` canonical-record
/// preimage (spec v0.13 §5.3). Bit 0 = `fp_present`, bit 1 =
/// `xpub_present`, bits 2..7 reserved (must be 0). Returns
/// [`Error::InvalidPresenceByte`] with the offending reserved-bit
/// field if any of bits 2..7 is set.
///
/// v0.13's encoder masks reserved bits when building the preimage, so
/// this helper is unreachable on v0.13 wire today. It enforces the
/// spec §5.3 "decoders MUST reject" clause for any future
/// canonical-record consumer (e.g., a verification-mode tool that
/// reconstructs the preimage to cross-check a `WalletPolicyId`).
pub fn validate_presence_byte(byte: u8) -> Result<(), Error> {
    let reserved_bits = byte & 0b1111_1100;
    if reserved_bits != 0 {
        return Err(Error::InvalidPresenceByte { reserved_bits });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
    use crate::tag::Tag;
    use crate::tlv::TlvSection;
    use crate::tree::{Body, Node};
    use crate::use_site_path::UseSitePath;

    fn bip84_descriptor() -> Descriptor {
        Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(OriginPath {
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
                }),
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
    fn md1_encoding_id_deterministic() {
        let d = bip84_descriptor();
        let id1 = compute_md1_encoding_id(&d).unwrap();
        let id2 = compute_md1_encoding_id(&d).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn md1_encoding_id_differs_for_different_paths() {
        let d1 = bip84_descriptor();
        let mut d2 = bip84_descriptor();
        if let PathDeclPaths::Shared(p) = &mut d2.path_decl.paths {
            p.components[2] = PathComponent {
                hardened: true,
                value: 1,
            };
        }
        let id1 = compute_md1_encoding_id(&d1).unwrap();
        let id2 = compute_md1_encoding_id(&d2).unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn wdt_id_invariant_to_origin_path_change() {
        let d1 = bip84_descriptor();
        let mut d2 = bip84_descriptor();
        if let PathDeclPaths::Shared(p) = &mut d2.path_decl.paths {
            p.components[2] = PathComponent {
                hardened: true,
                value: 1,
            };
        }
        let id1 = compute_wallet_descriptor_template_id(&d1).unwrap();
        let id2 = compute_wallet_descriptor_template_id(&d2).unwrap();
        // Same template structure (use-site path, tree) → same WDT-Id
        assert_eq!(id1, id2);
    }

    #[test]
    fn wdt_id_differs_for_different_use_site_paths() {
        let d1 = bip84_descriptor();
        let mut d2 = bip84_descriptor();
        d2.use_site_path = UseSitePath {
            multipath: None,
            wildcard_hardened: false,
        };
        let id1 = compute_wallet_descriptor_template_id(&d1).unwrap();
        let id2 = compute_wallet_descriptor_template_id(&d2).unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn wdt_id_invariant_to_fingerprint_addition() {
        let d1 = bip84_descriptor();
        let mut d2 = bip84_descriptor();
        d2.tlv.fingerprints = Some(vec![(0u8, [0xaa, 0xbb, 0xcc, 0xdd])]);
        let id1 = compute_wallet_descriptor_template_id(&d1).unwrap();
        let id2 = compute_wallet_descriptor_template_id(&d2).unwrap();
        // Fingerprints are excluded from WDT-Id hash domain
        assert_eq!(id1, id2);
    }

    /// L15: the WDT-id must be invariant to placeholder-index permutation,
    /// mirroring the policy-id's canonicalization. `wsh(multi(2,@1,@0))`
    /// (non-canonical placeholder ordering) and `wsh(multi(2,@0,@1))`
    /// (canonical) describe the same template and MUST share a WDT-id.
    /// RED today (raw `*idx` is hashed without canonicalization); GREEN
    /// after compute_wallet_descriptor_template_id canonicalizes a clone.
    #[test]
    fn wdt_id_invariant_to_placeholder_ordering() {
        let mk_d = |indices: Vec<u8>| Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(OriginPath {
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
                }),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![Node {
                    tag: Tag::Multi,
                    body: Body::MultiKeys { k: 2, indices },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        // Non-canonical: tree first-occurrence is @1 then @0.
        let d_non_canonical = mk_d(vec![1, 0]);
        // Canonical: tree first-occurrence is @0 then @1.
        let d_canonical = mk_d(vec![0, 1]);
        let id_nc = compute_wallet_descriptor_template_id(&d_non_canonical).unwrap();
        let id_c = compute_wallet_descriptor_template_id(&d_canonical).unwrap();
        assert_eq!(id_nc, id_c);
    }

    // ---- v0.13 WalletPolicyId tests ----

    /// Build a deterministic 65-byte xpub for tests: 32 bytes of `0x11`
    /// (chain code) followed by `0x02 || [0x22; 32]` (compressed pubkey
    /// with even Y prefix). The pubkey bytes are NOT a valid secp256k1
    /// point; tests that exercise §6.4 (`InvalidXpubBytes`) will use a
    /// real point. Phase 4 only hashes raw bytes.
    fn deterministic_xpub() -> [u8; 65] {
        let mut x = [0u8; 65];
        for b in x.iter_mut().take(32) {
            *b = 0x11;
        }
        x[32] = 0x02;
        for b in x.iter_mut().skip(33) {
            *b = 0x22;
        }
        x
    }

    /// Construct the dominant case: 1-of-1 cell-7 wpkh wallet with fp
    /// 0xDEADBEEF and a deterministic xpub at canonical BIP 84 origin.
    fn cell_7_wpkh_descriptor() -> Descriptor {
        Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(OriginPath {
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
                }),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            },
            tlv: {
                let mut t = TlvSection::new_empty();
                t.fingerprints = Some(vec![(0u8, [0xDE, 0xAD, 0xBE, 0xEF])]);
                t.pubkeys = Some(vec![(0u8, deterministic_xpub())]);
                t
            },
        }
    }

    /// **GOLDEN VECTOR** (load-bearing): byte-exact construction of the
    /// 1-of-1 cell-7 wpkh `WalletPolicyId` preimage and SHA-256 truncation.
    ///
    /// Component bit budget (hand-derived; locks LP4-ext varint unit
    /// semantics — lengths are in bits, not bytes):
    ///
    /// ```text
    /// canonical_template_tree:
    ///   Tag::Wpkh primary code 0x00 (5 bits)         = 5 bits
    ///   KeyArg index @0  (kiw=0 since n=1)            = 0 bits
    ///   --------------------------------------------------
    ///   total                                          = 5 bits
    ///   into_bytes() zero-pads to 1 byte              = 0x00
    ///
    /// origin path m/84'/0'/0':
    ///   depth=3       (4 bits)                        =  4
    ///   84'  hardened(1) + varint(84)  = 1 + (4 + 7)  = 12
    ///   0'   hardened(1) + varint(0)   = 1 + (4 + 0)  =  5
    ///   0'   hardened(1) + varint(0)   = 1 + (4 + 0)  =  5
    ///   ------------------------------------------------
    ///   total                                          = 26 bits
    ///
    /// use-site <0;1>/*:
    ///   has-mp=1 (1) + alt_count-2=0 (3)              =  4
    ///   alt0: hardened=0 (1) + varint(0)=4            =  5
    ///   alt1: hardened=0 (1) + varint(1)=5            =  6
    ///   wildcard_hardened=0 (1)                        =  1
    ///   ------------------------------------------------
    ///   total                                          = 16 bits
    ///
    /// record_bw bits:
    ///   varint(26): L=5 (4 bits) + 5-bit payload      =  9
    ///   path bits  (re-emitted)                       = 26
    ///   varint(16): L=5 (4 bits) + 5-bit payload      =  9
    ///   use-site bits (re-emitted)                    = 16
    ///   ------------------------------------------------
    ///   total                                          = 60 bits
    ///   into_bytes() zero-pads to 8 bytes (64 bits)
    ///
    /// presence_byte = (1 | 1<<1) & 0b11 = 0x03
    /// fp = [DE, AD, BE, EF] (4 bytes)
    /// xpub = [11; 32] || 02 || [22; 32]  (65 bytes)
    /// record total =  1 + 8 + 4 + 65 = 78 bytes
    /// hash_input  = canonical_template_tree(1) || record(78) = 79 bytes
    /// ```
    ///
    /// Expected bytes computed independently in `/tmp/golden_vec.py`.
    #[test]
    fn golden_vector_wpkh_cell_7() {
        let d = cell_7_wpkh_descriptor();

        // Independently re-construct the canonical bitstream so the
        // arithmetic assertion (LP4-ext varint unit confusion gate) is
        // checked against locally-computed lengths. We mirror the
        // implementation's component writes here so a unit-confusion
        // bug surfaces in the assertion below before SHA-256 swallows
        // it.
        let path = match &d.path_decl.paths {
            PathDeclPaths::Shared(p) => p.clone(),
            _ => panic!("test fixture is shared"),
        };
        let mut path_scratch = crate::bitstream::BitWriter::new();
        path.write(&mut path_scratch).unwrap();
        let path_bit_len = path_scratch.bit_len();
        let path_bytes = path_scratch.into_bytes();
        assert_eq!(path_bit_len, 26, "BIP-84 origin path is 26 bits");
        assert_eq!(path_bytes, vec![0x3b, 0xd4, 0x84, 0x00]);

        let mut us_scratch = crate::bitstream::BitWriter::new();
        d.use_site_path.write(&mut us_scratch).unwrap();
        let use_site_bit_len = us_scratch.bit_len();
        let us_bytes = us_scratch.into_bytes();
        assert_eq!(use_site_bit_len, 16, "<0;1>/* use-site is 16 bits");
        assert_eq!(us_bytes, vec![0x80, 0x06]);

        // Record bitstream construction must match impl exactly.
        let mut record_bw = crate::bitstream::BitWriter::new();
        crate::varint::write_varint(&mut record_bw, path_bit_len as u32).unwrap();
        crate::bitstream::re_emit_bits(&mut record_bw, &path_bytes, path_bit_len).unwrap();
        crate::varint::write_varint(&mut record_bw, use_site_bit_len as u32).unwrap();
        crate::bitstream::re_emit_bits(&mut record_bw, &us_bytes, use_site_bit_len).unwrap();

        // ARITHMETIC ASSERTION — load-bearing. varint(26)=9 bits and
        // varint(16)=9 bits (both need a 5-bit payload because L=5).
        // Total = 9 + 26 + 9 + 16 = 60. If lengths were in *bytes* (a
        // common bug), the encoded varints would be much smaller (L=2
        // for both → 6 bits each) and this assertion would fail.
        let varint_path_cost = 4 + (32 - (path_bit_len as u32).leading_zeros()) as usize;
        let varint_us_cost = 4 + (32 - (use_site_bit_len as u32).leading_zeros()) as usize;
        let expected_record_bits =
            varint_path_cost + path_bit_len + varint_us_cost + use_site_bit_len;
        assert_eq!(record_bw.bit_len(), expected_record_bits);
        assert_eq!(record_bw.bit_len(), 60, "cell-7 record is 60 bits");

        let record_bytes = record_bw.into_bytes();
        assert_eq!(
            record_bytes,
            vec![0x5d, 0x1d, 0xea, 0x42, 0x0b, 0x08, 0x00, 0x60]
        );

        // Canonical template tree: 5-bit Wpkh primary tag, zero-padded
        // to one byte.
        let mut tree_w = crate::bitstream::BitWriter::new();
        crate::tree::write_node(&mut tree_w, &d.tree, d.key_index_width()).unwrap();
        let tree_bytes = tree_w.into_bytes();
        assert_eq!(tree_bytes, vec![0x00]);

        // Full hash input — byte-by-byte.
        let presence_byte: u8 = 0x03;
        let fp = [0xDE, 0xAD, 0xBE, 0xEF];
        let xpub = deterministic_xpub();
        let mut expected_hash_input: Vec<u8> = Vec::new();
        expected_hash_input.extend_from_slice(&tree_bytes);
        expected_hash_input.push(presence_byte);
        expected_hash_input.extend_from_slice(&record_bytes);
        expected_hash_input.extend_from_slice(&fp);
        expected_hash_input.extend_from_slice(&xpub);
        assert_eq!(expected_hash_input.len(), 79);

        let expected_hex = "00035d1dea420b080060deadbeef\
            1111111111111111111111111111111111111111111111111111111111111111\
            02\
            2222222222222222222222222222222222222222222222222222222222222222";
        assert_eq!(hex(&expected_hash_input), expected_hex);

        // Final identity bytes (computed by /tmp/golden_vec.py).
        let expected_id: [u8; 16] = [
            0x66, 0x50, 0xb9, 0x80, 0x3b, 0x3c, 0x66, 0x21, 0x01, 0x40, 0x54, 0x0d, 0xa8, 0xd7,
            0x65, 0xa0,
        ];

        let id = compute_wallet_policy_id(&d).unwrap();
        assert_eq!(*id.as_bytes(), expected_id);
    }

    /// Trivial hex helper for byte-exact assertions in the golden test.
    fn hex(bs: &[u8]) -> String {
        let mut s = String::with_capacity(bs.len() * 2);
        for b in bs {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }

    /// Two encodings of the same logical wallet — one with the canonical
    /// path explicitly written, one with no explicit path (the encoder
    /// fills `canonical_origin` into `path_decl` per Option A) — produce
    /// identical WalletPolicyId. (In practice, both have the same
    /// `path_decl` payload after canonicalization; this test pins the
    /// invariant for the trivial case.)
    #[test]
    fn walletpolicyid_stable_across_origin_elision() {
        // Explicit: wpkh(@0) with path_decl = Shared(m/84'/0'/0').
        let d_explicit = cell_7_wpkh_descriptor();
        // Elided: same wpkh(@0) wallet, but path_decl is a genuinely EMPTY
        // Shared origin (no explicit path). The canonical wrapper
        // (wpkh → m/84'/0'/0') supplies the path at hash time via the L14
        // canonical-fill. RED today (the empty path hashes a 0000 length
        // prefix + no components, differing from the explicit component
        // bits); GREEN after the L14 fill.
        let mut d_elided = cell_7_wpkh_descriptor();
        d_elided.path_decl = PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
        };
        let id_explicit = compute_wallet_policy_id(&d_explicit).unwrap();
        let id_elided = compute_wallet_policy_id(&d_elided).unwrap();
        // The documented "stable across origin-elision" invariant: the
        // elided form, canonical-filled, must hash identically to the
        // explicit form.
        assert_eq!(id_explicit, id_elided);
    }

    /// Use-site path supplied as the descriptor baseline vs supplied via
    /// `UseSitePathOverrides[0]` — same resolved bits → same ID.
    #[test]
    fn walletpolicyid_stable_across_use_site_elision() {
        let d_baseline = cell_7_wpkh_descriptor();
        let mut d_override = cell_7_wpkh_descriptor();
        d_override.use_site_path = UseSitePath {
            multipath: None,
            wildcard_hardened: false,
        };
        d_override.tlv.use_site_path_overrides =
            Some(vec![(0u8, UseSitePath::standard_multipath())]);
        let id1 = compute_wallet_policy_id(&d_baseline).unwrap();
        let id2 = compute_wallet_policy_id(&d_override).unwrap();
        assert_eq!(id1, id2);
    }

    /// Template-only (no fp, no xpub) WalletPolicyId differs from the
    /// fully-keyed cell-7 version — presence-significance gate.
    #[test]
    fn walletpolicyid_template_only_differs_from_full_cell_7() {
        let full = cell_7_wpkh_descriptor();
        let mut template_only = cell_7_wpkh_descriptor();
        template_only.tlv.fingerprints = None;
        template_only.tlv.pubkeys = None;
        let id_full = compute_wallet_policy_id(&full).unwrap();
        let id_template = compute_wallet_policy_id(&template_only).unwrap();
        assert_ne!(id_full, id_template);
    }

    /// 2-of-2 wsh(multi) with `@0` cell-7 (fp+xpub) and `@1` cell-1
    /// (template-only). presence_bytes are 0b11 and 0b00 respectively;
    /// distinct from a "both fully populated" or "both template-only"
    /// version.
    #[test]
    fn walletpolicyid_partial_keys_distinct() {
        #[allow(dead_code)]
        fn pkk(index: u8) -> Node {
            Node {
                tag: Tag::PkK,
                body: Body::KeyArg { index },
            }
        }
        let bip48_2 = OriginPath {
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
        let mk_d = |fps: Option<Vec<(u8, [u8; 4])>>, pks: Option<Vec<(u8, [u8; 65])>>| Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(bip48_2.clone()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![Node {
                    tag: Tag::Multi,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: {
                let mut t = TlvSection::new_empty();
                t.fingerprints = fps;
                t.pubkeys = pks;
                t
            },
        };
        let xpub = deterministic_xpub();
        // Full: both @0 and @1 have fp+xpub.
        let d_full = mk_d(
            Some(vec![(0, [0x11; 4]), (1, [0x22; 4])]),
            Some(vec![(0, xpub), (1, xpub)]),
        );
        // Mixed: @0 cell-7, @1 cell-1 (no fp, no xpub).
        let d_mixed = mk_d(Some(vec![(0, [0x11; 4])]), Some(vec![(0, xpub)]));
        let id_full = compute_wallet_policy_id(&d_full).unwrap();
        let id_mixed = compute_wallet_policy_id(&d_mixed).unwrap();
        assert_ne!(id_full, id_mixed);
    }

    /// Same per-`@N` records under two different wrapper tags
    /// (`wpkh(@0)` vs `pkh(@0)`) → distinct WalletPolicyId. Wrapper
    /// context is hashed via canonical_template_tree_bytes.
    #[test]
    fn walletpolicyid_wrapper_context_in_template_hash() {
        let d_wpkh = cell_7_wpkh_descriptor();
        let mut d_pkh = cell_7_wpkh_descriptor();
        d_pkh.tree = Node {
            tag: Tag::Pkh,
            body: Body::KeyArg { index: 0 },
        };
        // Force same canonical record by overriding origin to the
        // (BIP-44) canonical for pkh — so the only difference is the
        // wrapper tag in the template tree.
        d_pkh.path_decl = PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent {
                        hardened: true,
                        value: 44,
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
            }),
        };
        // Reset to wpkh's canonical so records share the bytewise
        // origin path — this isolates wrapper-context-only difference.
        d_pkh.path_decl = d_wpkh.path_decl.clone();
        let id_wpkh = compute_wallet_policy_id(&d_wpkh).unwrap();
        let id_pkh = compute_wallet_policy_id(&d_pkh).unwrap();
        assert_ne!(id_wpkh, id_pkh);
    }

    /// Hand-construct two preimages identical except for nonzero
    /// reserved bits in `presence_byte`; they MUST hash to the same
    /// 16-byte WalletPolicyId because the encoder masks reserved bits
    /// to 0 before writing the byte. Property is enforced indirectly:
    /// since `compute_wallet_policy_id` is the only public entry point
    /// and it always masks via `& 0b0000_0011`, two descriptors that
    /// agree on (fp, xpub) presence must produce identical IDs even if
    /// the underlying hash bytes were ever drift-injected. This test
    /// hashes two by-hand preimages to prove SHA-256 is mask-stable.
    #[test]
    fn walletpolicyid_reserved_bits_masking_property() {
        // Construct two preimages: one with presence_byte = 0b11 = 0x03,
        // one with presence_byte = 0b1111_1111 = 0xff. Apply the
        // encoder's mask 0b0000_0011 to both BEFORE hashing — both
        // should reduce to 0x03 and produce the same hash.
        let common = vec![0x00u8, 0x42, 0x42, 0x42];
        // Apply the encoder's mask to two distinct candidate presence
        // bytes (low-bits-only vs. all-ones) — both reduce to 0x03.
        let candidates = [0b0000_0011u8, 0b1111_1111u8];
        let mask = 0b0000_0011u8;
        let masked_a = candidates[0] & mask;
        let masked_b = candidates[1] & mask;
        assert_eq!(masked_a, masked_b);
        let mut input_a = common.clone();
        input_a.push(masked_a);
        let mut input_b = common.clone();
        input_b.push(masked_b);
        let h_a = bitcoin::hashes::sha256::Hash::hash(&input_a);
        let h_b = bitcoin::hashes::sha256::Hash::hash(&input_b);
        assert_eq!(h_a, h_b);

        // Sanity: WITHOUT masking, the hashes differ — proving the
        // mask is the load-bearing step.
        let mut unmasked_a = common.clone();
        unmasked_a.push(candidates[0]);
        let mut unmasked_b = common.clone();
        unmasked_b.push(candidates[1]);
        let h_a_raw = bitcoin::hashes::sha256::Hash::hash(&unmasked_a);
        let h_b_raw = bitcoin::hashes::sha256::Hash::hash(&unmasked_b);
        assert_ne!(h_a_raw, h_b_raw);
    }

    /// `to_phrase()` round-trips through Phrase::from_id_bytes and
    /// returns 12 BIP 39 words for any non-trivial id.
    #[test]
    fn walletpolicyid_to_phrase_returns_12_bip39_words() {
        let d = cell_7_wpkh_descriptor();
        let id = compute_wallet_policy_id(&d).unwrap();
        let phrase = id.to_phrase().unwrap();
        assert_eq!(phrase.0.len(), 12);
        for word in &phrase.0 {
            assert!(!word.is_empty());
        }
    }

    /// `compute_wallet_policy_id` canonicalizes its input internally:
    /// `tr(multi(2, @1, @0))` (non-canonical) and the canonical
    /// equivalent `tr(multi(2, @0, @1))` (with TLVs renumbered
    /// consistently) produce identical IDs.
    #[test]
    fn compute_wallet_policy_id_canonicalizes_first() {
        #[allow(dead_code)]
        fn pkk(index: u8) -> Node {
            Node {
                tag: Tag::PkK,
                body: Body::KeyArg { index },
            }
        }
        let xpub_a = deterministic_xpub();
        let mut xpub_b = deterministic_xpub();
        xpub_b[0] = 0x33;
        let bip48_2 = OriginPath {
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
        // Non-canonical: tree first-occurrence is @1 then @0; pubkeys
        // wired by original index — A↔@0, B↔@1.
        let d_non_canonical = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(bip48_2.clone()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![Node {
                    tag: Tag::Multi,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![1, 0],
                    },
                }]),
            },
            tlv: {
                let mut t = TlvSection::new_empty();
                t.pubkeys = Some(vec![(0, xpub_a), (1, xpub_b)]);
                t
            },
        };
        // Canonical equivalent: tree first-occurrence is @0 then @1;
        // pubkeys renumbered to match (original-@1 → new-@0 → carries B,
        // original-@0 → new-@1 → carries A).
        let d_canonical = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(bip48_2),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![Node {
                    tag: Tag::Multi,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: {
                let mut t = TlvSection::new_empty();
                t.pubkeys = Some(vec![(0, xpub_b), (1, xpub_a)]);
                t
            },
        };
        let id_nc = compute_wallet_policy_id(&d_non_canonical).unwrap();
        let id_c = compute_wallet_policy_id(&d_canonical).unwrap();
        assert_eq!(id_nc, id_c);
    }

    // ─── validate_presence_byte (v0.13.1, spec §5.3) ─────────────────

    #[test]
    fn validate_presence_byte_accepts_all_four_legal_combinations() {
        for byte in [0b00, 0b01, 0b10, 0b11] {
            validate_presence_byte(byte).unwrap();
        }
    }

    #[test]
    fn validate_presence_byte_rejects_lowest_reserved_bit() {
        // bit 2 set
        let err = validate_presence_byte(0b0000_0100).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidPresenceByte {
                reserved_bits: 0b0000_0100
            }
        ));
    }

    #[test]
    fn validate_presence_byte_rejects_high_reserved_bit_with_legal_low_bits() {
        // bit 7 set + fp_present + xpub_present
        let err = validate_presence_byte(0b1000_0011).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidPresenceByte {
                reserved_bits: 0b1000_0000
            }
        ));
    }

    #[test]
    fn validate_presence_byte_rejects_all_bits_set() {
        let err = validate_presence_byte(0xFF).unwrap_err();
        assert!(matches!(
            err,
            Error::InvalidPresenceByte {
                reserved_bits: 0b1111_1100
            }
        ));
    }
}
