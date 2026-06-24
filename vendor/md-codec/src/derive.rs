//! Address derivation (v0.32).
//!
//! v0.32 replaces the v0.14-era hand-rolled 5-shape allow-list with an
//! AST → [`miniscript::Descriptor`] converter
//! ([`crate::to_miniscript::to_miniscript_descriptor`]) and delegates
//! address rendering to rust-miniscript. Any BIP-388-parseable shape
//! derives — multi-leaf tap-trees, `tr(NUMS, ...)`, `sh(multi)`, arbitrary
//! `wsh(<miniscript>)`, and any tap-leaf miniscript fragment included.
//!
//! Feature-gated: requires `derive` (default-on). Pure-codec consumers can
//! opt out via `default-features = false`.
//!
//! ### What this module does NOT do
//!
//! - Origin path is not consulted. Origin is the path *to* the xpub from
//!   the master seed; address derivation starts at the xpub. The recorded
//!   origin matters for signing flows (PSBT key-source metadata), not for
//!   getting an address.
//! - Master fingerprint (`Fingerprints` TLV) is unused for the same
//!   reason — it identifies the master, not the derivation root.
//! - Hardened use-site components are rejected. Hardened public derivation
//!   is forbidden by BIP 32; an xpub-only restore cannot produce addresses
//!   for a wallet whose use-site path has a hardened alternative or
//!   hardened wildcard.

#[cfg(feature = "derive")]
use crate::encode::Descriptor;
#[cfg(feature = "derive")]
use crate::error::Error;
#[cfg(feature = "derive")]
use bitcoin::NetworkKind;
#[cfg(feature = "derive")]
use bitcoin::address::NetworkUnchecked;
#[cfg(feature = "derive")]
use bitcoin::bip32::{ChainCode, ChildNumber, Fingerprint, Xpub};
#[cfg(feature = "derive")]
use bitcoin::secp256k1::PublicKey;
#[cfg(feature = "derive")]
use bitcoin::{Address, Network};

/// Reconstruct an [`Xpub`] from a 65-byte `Pubkeys` TLV payload.
///
/// Layout: `bytes[0..32]` = chain code; `bytes[32..65]` = compressed
/// public key. The four BIP 32 metadata fields (`network`, `depth`,
/// `parent_fingerprint`, `child_number`) are not used by
/// [`Xpub::derive_pub`] (only `chain_code` and `public_key` participate
/// in `CKDpub`); they are filled with safe placeholders.
#[cfg(feature = "derive")]
pub(crate) fn xpub_from_tlv_bytes(idx: u8, bytes: &[u8; 65]) -> Result<Xpub, Error> {
    let chain_code_bytes: [u8; 32] = bytes[0..32]
        .try_into()
        .expect("32-byte slice is statically sized");
    let chain_code = ChainCode::from(chain_code_bytes);
    let public_key =
        PublicKey::from_slice(&bytes[32..65]).map_err(|_| Error::InvalidXpubBytes { idx })?;
    Ok(Xpub {
        network: NetworkKind::Main,
        depth: 0,
        parent_fingerprint: Fingerprint::default(),
        child_number: ChildNumber::Normal { index: 0 },
        public_key,
        chain_code,
    })
}

#[cfg(feature = "derive")]
impl Descriptor {
    /// Derive the address at `(chain, index)` for this descriptor on
    /// `network`.
    ///
    /// `chain` selects the use-site multipath alternative (e.g. `0` =
    /// receive, `1` = change for the standard `<0;1>/*` form). `index` is
    /// the trailing wildcard child number.
    ///
    /// Returns an [`Address<NetworkUnchecked>`]; callers can
    /// `.assume_checked()` (when they trust the network parameter) or
    /// `.require_network(network)` to lock it down.
    ///
    /// # Errors
    ///
    /// - [`Error::MissingPubkey`] when any `@N` lacks an xpub.
    /// - [`Error::InvalidXpubBytes`] when an xpub's 33-byte pubkey field
    ///   doesn't parse as a valid secp256k1 point.
    /// - [`Error::ChainIndexOutOfRange`] when `chain` is out of range for
    ///   the use-site multipath.
    /// - [`Error::HardenedPublicDerivation`] when the use-site path
    ///   requires a hardened derivation step.
    /// - [`Error::MissingExplicitOrigin`] propagated from
    ///   [`crate::canonicalize::expand_per_at_n`].
    /// - [`Error::AddressDerivationFailed`] for any miniscript-layer
    ///   failure (type check, context error, unsupported fragment).
    pub fn derive_address(
        &self,
        chain: u32,
        index: u32,
        network: Network,
    ) -> Result<Address<NetworkUnchecked>, Error> {
        // Pre-flight: hardened public-key derivation rejection (BIP-32
        // forbids). The shared `has_hardened_use_site` predicate (Point B)
        // covers BOTH the baseline use-site path AND every per-`@N`
        // override — a hardened wildcard OR any hardened multipath
        // alternative, anywhere. The pre-fix baseline-only checks missed a
        // hardened alternative inside an override, which then surfaced only
        // as a generic `AddressDerivationFailed` deep in the converter.
        if crate::to_miniscript::has_hardened_use_site(self) {
            return Err(Error::HardenedPublicDerivation);
        }
        // Pre-flight: chain index in range. Bound by the MAX alt-count across
        // the baseline use-site path AND every per-`@N` override (None →
        // alt-count 1, i.e. only chain 0). The per-key path
        // (use_site_to_derivation_path) is the real authority and STILL
        // fail-closes per key; this coarse gate must not be narrower than the
        // widest key, or a valid override change chain is rejected.
        let baseline_alts = self
            .use_site_path
            .multipath
            .as_ref()
            .map(|a| a.len())
            .unwrap_or(1);
        let max_alts = self
            .tlv
            .use_site_path_overrides
            .iter()
            .flatten()
            .map(|(_, p)| p.multipath.as_ref().map(|a| a.len()).unwrap_or(1))
            .fold(baseline_alts, std::cmp::max);
        if (chain as usize) >= max_alts {
            return Err(Error::ChainIndexOutOfRange {
                chain,
                alt_count: max_alts,
            });
        }

        let desc = crate::to_miniscript::to_miniscript_descriptor(self, chain)?;
        let definite =
            desc.at_derivation_index(index)
                .map_err(|e| Error::AddressDerivationFailed {
                    detail: e.to_string(),
                })?;
        let addr = definite
            .address(network)
            .map_err(|e| Error::AddressDerivationFailed {
                detail: e.to_string(),
            })?;
        Ok(addr.into_unchecked())
    }
}

#[cfg(all(test, feature = "derive"))]
mod tests {
    use super::*;
    use crate::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
    use crate::tag::Tag;
    use crate::tlv::TlvSection;
    use crate::tree::{Body, Node};
    use crate::use_site_path::{Alternative, UseSitePath};

    // ─── xpub_from_tlv_bytes ─────────────────────────────────────────

    #[test]
    fn xpub_from_tlv_bytes_rejects_invalid_pubkey() {
        // 33 zero bytes is not a valid compressed pubkey.
        let bytes = [0u8; 65];
        assert!(matches!(
            xpub_from_tlv_bytes(7, &bytes),
            Err(Error::InvalidXpubBytes { idx: 7 })
        ));
    }

    fn bip84_origin() -> OriginPath {
        OriginPath {
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
        }
    }

    fn one_test_xpub_bytes() -> [u8; 65] {
        let mut bytes = [0u8; 65];
        bytes[0..32].copy_from_slice(&[0x42; 32]);
        bytes[32] = 0x02;
        bytes[33..].copy_from_slice(&[
            0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87,
            0x0B, 0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B,
            0x16, 0xF8, 0x17, 0x98,
        ]);
        bytes
    }

    #[test]
    fn derive_address_missing_pubkey_for_partial_keys() {
        // 2-of-2 wsh-sortedmulti with only @0 populated.
        let d = Descriptor {
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
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: {
                let mut t = TlvSection::new_empty();
                t.pubkeys = Some(vec![(0u8, one_test_xpub_bytes())]);
                t
            },
        };
        let err = d.derive_address(0, 0, Network::Bitcoin).unwrap_err();
        assert!(matches!(err, Error::MissingPubkey { idx: 1 }));
    }

    #[test]
    fn derive_address_chain_out_of_range() {
        let d = Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(bip84_origin()),
            },
            use_site_path: UseSitePath::standard_multipath(), // alt-count=2
            tree: Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            },
            tlv: {
                let mut t = TlvSection::new_empty();
                t.pubkeys = Some(vec![(0u8, one_test_xpub_bytes())]);
                t
            },
        };
        let err = d.derive_address(5, 0, Network::Bitcoin).unwrap_err();
        assert!(matches!(
            err,
            Error::ChainIndexOutOfRange {
                chain: 5,
                alt_count: 2
            }
        ));
    }

    #[test]
    fn derive_address_hardened_wildcard_rejected() {
        let d = Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(bip84_origin()),
            },
            use_site_path: UseSitePath {
                multipath: Some(vec![
                    Alternative {
                        hardened: false,
                        value: 0,
                    },
                    Alternative {
                        hardened: false,
                        value: 1,
                    },
                ]),
                wildcard_hardened: true,
            },
            tree: Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            },
            tlv: {
                let mut t = TlvSection::new_empty();
                t.pubkeys = Some(vec![(0u8, one_test_xpub_bytes())]);
                t
            },
        };
        let err = d.derive_address(0, 0, Network::Bitcoin).unwrap_err();
        assert!(matches!(err, Error::HardenedPublicDerivation));
    }

    /// Build a legal D5(b)-shaped `wpkh(@0)` descriptor with a `None`
    /// baseline use-site (alt-count modeled as 1) plus a per-`@0` use-site
    /// override carrying a `<0;1>` multipath (alt-count 2). `@0` has an
    /// xpub so addresses resolve.
    fn none_baseline_override_change_descriptor() -> Descriptor {
        Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(bip84_origin()),
            },
            use_site_path: UseSitePath {
                multipath: None,
                wildcard_hardened: false,
            },
            tree: Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            },
            tlv: {
                let mut t = TlvSection::new_empty();
                t.use_site_path_overrides = Some(vec![(
                    0u8,
                    UseSitePath {
                        multipath: Some(vec![
                            Alternative {
                                hardened: false,
                                value: 0,
                            },
                            Alternative {
                                hardened: false,
                                value: 1,
                            },
                        ]),
                        wildcard_hardened: false,
                    },
                )]);
                t.pubkeys = Some(vec![(0u8, one_test_xpub_bytes())]);
                t
            },
        }
    }

    #[test]
    fn derive_address_override_change_chain_derivable() {
        // Funds-availability (M3): a `None`-baseline + `Some(<0;1>)`-override
        // wallet must derive its change (chain-1) address. The pre-fix gate
        // bounds `chain` ONLY by the baseline (`None` → only chain 0) and
        // rejects chain=1 with `alt_count: 0`. The per-key path resolves
        // `@0`'s own override multipath and derives correctly post-fix.
        let d = none_baseline_override_change_descriptor();
        // Receive (chain 0) control: derivable in both pre- and post-fix.
        assert!(d.derive_address(0, 0, Network::Bitcoin).is_ok());
        // Change (chain 1): RED today → GREEN after the gate widening.
        let change = d.derive_address(1, 0, Network::Bitcoin);
        assert!(
            change.is_ok(),
            "override change chain must derive, got {change:?}"
        );
    }

    #[test]
    fn derive_address_override_chain_over_max_still_rejects() {
        // Positive control / no over-widening: chain=2 is beyond the
        // override's 2-alt max (max_alts == 2) → still rejected.
        let d = none_baseline_override_change_descriptor();
        let err = d.derive_address(2, 0, Network::Bitcoin).unwrap_err();
        assert!(
            matches!(
                err,
                Error::ChainIndexOutOfRange {
                    chain: 2,
                    alt_count: 2
                }
            ),
            "over-max chain must still reject with alt_count=2, got {err:?}"
        );
    }
}
