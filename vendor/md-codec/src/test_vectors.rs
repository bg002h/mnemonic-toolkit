//! Canonical `md` test-vector corpus.
//!
//! Used by `md-codec`'s own integration tests, by `md-cli`'s `vectors`
//! subcommand, and by `md-cli`'s `tests/json_snapshots.rs` /
//! `tests/template_roundtrip.rs`. Single source of truth: any vector
//! addition / removal / rename happens here.
//!
//! `Vector` is `#[non_exhaustive]` so future fields can be added without a
//! breaking-change bump: external consumers construct nothing — they only
//! read `MANIFEST` entries.

/// One entry of the canonical test-vector corpus.
#[non_exhaustive]
pub struct Vector {
    /// Vector identifier — used in test failure messages and as a stable
    /// handle for cross-suite filtering. Convention: snake_case mirroring
    /// the wallet-policy template's distinguishing structure.
    pub name: &'static str,
    /// BIP-388 wallet-policy template string the vector encodes. Parsed
    /// by `parse::template`; round-tripped through `encode` and `decode`.
    pub template: &'static str,
    /// `(@N, xpub)` pairs binding each `@N` placeholder in `template`. Empty
    /// when the vector exercises template-only paths (no key binding).
    pub keys: &'static [(u8, &'static str)],
    /// `(@N, 4-byte master fingerprint)` pairs aligned with `keys`. Empty
    /// when the vector does not exercise fingerprint round-tripping.
    pub fingerprints: &'static [(u8, [u8; 4])],
    /// When true, force the encoder onto the chunked wire path even if the
    /// payload would fit in a single chunk. Exercises chunk-boundary logic
    /// without padding the template artificially.
    pub force_chunked: bool,
}

/// The canonical 10-entry corpus.
///
/// `tr_with_leaf` and `sh_wpkh` are intentionally omitted: their round-trip
/// via the v0.14+ codec is asymmetric (encode requires explicit origin;
/// decode strips canonical 86'/0'/0' resp. 49'/0'/0'). Coverage for those
/// wrappers is preserved by `parse::template` unit tests
/// (`tr_with_one_leaf`, `sh_wpkh_nested`).
#[rustfmt::skip]
pub const MANIFEST: &[Vector] = &[
    Vector { name: "wpkh_basic",         template: "wpkh(@0/<0;1>/*)",                                   keys: &[], fingerprints: &[], force_chunked: false },
    Vector { name: "pkh_basic",          template: "pkh(@0/<0;1>/*)",                                    keys: &[], fingerprints: &[], force_chunked: false },
    Vector { name: "wsh_multi_2of2",     template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",                keys: &[], fingerprints: &[], force_chunked: false },
    Vector { name: "wsh_multi_2of3",     template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",     keys: &[], fingerprints: &[], force_chunked: false },
    Vector { name: "wsh_sortedmulti",    template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))", keys: &[], fingerprints: &[], force_chunked: false },
    Vector { name: "tr_keyonly",         template: "tr(@0/<0;1>/*)",                                     keys: &[], fingerprints: &[], force_chunked: false },
    Vector { name: "sh_wsh_multi",       template: "sh(wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)))",            keys: &[], fingerprints: &[], force_chunked: false },
    Vector { name: "wsh_divergent_paths", template: "wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))",               keys: &[], fingerprints: &[], force_chunked: false },
    Vector { name: "wsh_with_fingerprints", template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
        keys: &[],
        fingerprints: &[(0, [0xDE,0xAD,0xBE,0xEF]), (1, [0xCA,0xFE,0xBA,0xBE])],
        force_chunked: false },
    Vector { name: "wsh_multi_chunked",  template: "wsh(multi(3,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",     keys: &[], fingerprints: &[], force_chunked: true },
];
