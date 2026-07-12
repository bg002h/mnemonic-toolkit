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
    /// Explicit shared origin path applied via the encoder's `--path`
    /// override (`m/...` literal, or a named `bip44|48|49|84|86` form).
    /// `None` = elided origin — the encoder infers the canonical origin via
    /// `canonical_origin`. `Some(..)` is REQUIRED for non-canonical shapes
    /// (`tr()` + TapTree, NUMS-taproot) whose `canonical_origin` returns
    /// `None`: without an explicit origin they mint a card the decoder
    /// rejects with `MissingExplicitOrigin`. Carried into the emitted
    /// `.descriptor.json` via `path_decl` (the `.template` file alone does
    /// not determine it), so the BIP §Test Vectors table pins the
    /// template+path pair for path-carrying rows.
    pub path: Option<&'static str>,
}

/// The canonical 15-entry corpus.
///
/// Part-3 additions (BIP-alignment cycle): `sh_wpkh`, `tr_with_leaf`,
/// `nums_taproot`, `wsh_sortedmulti_2chunk`, and `single_string_boundary`.
///
/// * `sh_wpkh` — un-omitted: since F-A1, `sh(wpkh)` round-trips symmetrically
///   in ELIDED form (`canonical_origin(sh(wpkh))` = `m/49'/0'/0'`), so it is a
///   corpus ADDITION (`path: None`), not an asymmetric omission.
/// * `tr_with_leaf` / `nums_taproot` — non-canonical `tr()` shapes
///   (`canonical_origin` = `None`); expressible now via the `path` field,
///   which supplies the explicit origin the decoder requires.
/// * `wsh_sortedmulti_2chunk` — a genuine 2-member chunk set: a 2-of-8
///   sortedmulti with a master fingerprint on every cosigner makes a 376-bit
///   payload that still FITS a single string (below the 400-bit / 80-symbol
///   regular-code cap), so it is `force_chunked` to route it through `split`,
///   whose 320-bit per-chunk budget then yields two chunks. (Contrast
///   `wsh_multi_chunked`, also `force_chunked` but a chunk-set-of-one.)
/// * `single_string_boundary` (F-V2) — a 2-of-9 sortedmulti with fingerprints
///   on 8 of the 9 cosigners, sized so its single-string regular-code emit
///   lands at 79 of the 80 max data symbols (95 chars total = `md1` plus 79
///   data plus 13 checksum). Proves the regular-only single-string boundary
///   holds right at the codex32 BCH(93,80) cap — NOT chunked, NOT long-code.
#[rustfmt::skip]
pub const MANIFEST: &[Vector] = &[
    Vector { name: "wpkh_basic",         template: "wpkh(@0/<0;1>/*)",                                   keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "pkh_basic",          template: "pkh(@0/<0;1>/*)",                                    keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_multi_2of2",     template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",                keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_multi_2of3",     template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",     keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_sortedmulti",    template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))", keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "tr_keyonly",         template: "tr(@0/<0;1>/*)",                                     keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "sh_wsh_multi",       template: "sh(wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)))",            keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_divergent_paths", template: "wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))",               keys: &[], fingerprints: &[], force_chunked: false, path: None },
    Vector { name: "wsh_with_fingerprints", template: "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
        keys: &[],
        fingerprints: &[(0, [0xDE,0xAD,0xBE,0xEF]), (1, [0xCA,0xFE,0xBA,0xBE])],
        force_chunked: false, path: None },
    Vector { name: "wsh_multi_chunked",  template: "wsh(multi(3,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",     keys: &[], fingerprints: &[], force_chunked: true, path: None },
    // --- Part-3 additions ---
    // F-A1: elided sh(wpkh) now round-trips (canonical origin m/49'/0'/0').
    Vector { name: "sh_wpkh",            template: "sh(wpkh(@0/<0;1>/*))",                               keys: &[], fingerprints: &[], force_chunked: false, path: None },
    // Non-canonical tr()+leaf — explicit origin via the new `path` field.
    Vector { name: "tr_with_leaf",       template: "tr(@0/<0;1>/*,pk(@1/<0;1>/*))",                      keys: &[], fingerprints: &[], force_chunked: false, path: Some("48'/0'/0'/2'") },
    // NUMS-taproot (`is_nums = 1` wire path) — script-path-only tr, explicit origin.
    Vector { name: "nums_taproot",       template: "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
        keys: &[], fingerprints: &[], force_chunked: false, path: Some("48'/0'/0'/2'") },
    // 2-of-8 sortedmulti + fingerprint on every cosigner: a 376-bit payload
    // that FITS a single string (< the 400-bit regular-code cap), so
    // `force_chunked` routes it through `split`, whose 320-bit per-chunk budget
    // yields a genuine 2-member chunk set.
    Vector { name: "wsh_sortedmulti_2chunk",
        template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*))",
        keys: &[],
        fingerprints: &[
            (0, [0x01,0x02,0x03,0x04]), (1, [0x02,0x03,0x04,0x05]),
            (2, [0x03,0x04,0x05,0x06]), (3, [0x04,0x05,0x06,0x07]),
            (4, [0x05,0x06,0x07,0x08]), (5, [0x06,0x07,0x08,0x09]),
            (6, [0x07,0x08,0x09,0x0A]), (7, [0x08,0x09,0x0A,0x0B]),
        ],
        force_chunked: true, path: None },
    // F-V2: single-string regular-code boundary. 2-of-9 sortedmulti with a
    // fingerprint on 8 of the 9 cosigners sizes the payload to 79 of the 80
    // max data symbols → a single 95-char md1 string (NOT chunked, NOT long).
    Vector { name: "single_string_boundary",
        template: "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*,@8/<0;1>/*))",
        keys: &[],
        fingerprints: &[
            (0, [0x01,0x02,0x03,0x04]), (1, [0x02,0x03,0x04,0x05]),
            (2, [0x03,0x04,0x05,0x06]), (3, [0x04,0x05,0x06,0x07]),
            (4, [0x05,0x06,0x07,0x08]), (5, [0x06,0x07,0x08,0x09]),
            (6, [0x07,0x08,0x09,0x0A]), (7, [0x08,0x09,0x0A,0x0B]),
        ],
        force_chunked: false, path: None },
];
