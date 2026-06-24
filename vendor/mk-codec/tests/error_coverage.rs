//! Exhaustiveness gate: every `mk_codec::Error` variant must have at least
//! one negative vector in `src/test_vectors/v0.1.json` whose `expected_error`
//! field's rendered `Display` rendering starts with the variant's
//! documented prefix.
//!
//! # How it works
//!
//! [`ErrorVariantName`] is a hand-written mirror enum whose variant names
//! match `mk_codec::Error` case-for-case. `strum::EnumIter` generates
//! `ErrorVariantName::iter()` so adding an entry forces the test to check
//! it. The test then asserts at least one negative vector in the corpus
//! pins an `expected_error` whose `Display` prefix matches the variant.
//!
//! Variants that are reachable only from the encoder (not from
//! `decode`'s string-input path) are explicitly exempt; see [`is_exempt`].
//!
//! # Maintenance rule
//!
//! When a new `Error` variant is added to `crates/mk-codec/src/error.rs`:
//!  1. Add a matching entry to [`ErrorVariantName`] below.
//!  2. Either add a negative vector to `src/test_vectors/v0.1.json` exercising
//!     the variant, or extend [`is_exempt`] with the variant's name plus
//!     a one-line rationale (encoder-only, structurally unreachable, …).
//!
//! This test will fail at CI until the maintainer takes one of those
//! actions. Compared to a runtime substring gate over a hand-curated
//! list inside `tests/vectors.rs`, the mirror-enum + strum pattern means
//! the maintainer only has to update one file (this file) per new
//! variant rather than two — the corpus check follows automatically.
//!
//! Mirrors `descriptor-mnemonic/crates/md-codec/tests/error_coverage.rs`'s
//! pattern, with mk-codec's variant set substituted; see that file for
//! the design rationale around `#[non_exhaustive]` on the source enum.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use strum::{EnumIter, IntoEnumIterator};

/// Mirror enum of every `mk_codec::Error` variant name.
///
/// Variant names MUST match the source enum case-for-case. The
/// [`Self::display_prefix`] method returns the `Error::Display`
/// prefix the test expects to find pinned in some negative vector's
/// `expected_error` field.
///
/// This enum is intentionally **not** `#[non_exhaustive]` — adding an
/// entry is the maintenance gesture; missing entries cause iteration
/// to skip the variant silently. The mirror-vs-source-drift discipline
/// is the maintainer's responsibility (matched md-codec's pattern).
#[derive(Debug, EnumIter)]
#[allow(dead_code)]
enum ErrorVariantName {
    InvalidHrp,
    MixedCase,
    InvalidStringLength,
    InvalidChar,
    BchUncorrectable,
    UnsupportedCardType,
    MalformedPayloadPadding,
    ChunkSetIdMismatch,
    ChunkedHeaderMalformed,
    MixedHeaderTypes,
    CrossChunkHashMismatch,
    UnsupportedVersion,
    ReservedBitsSet,
    InvalidPolicyIdStubCount,
    InvalidPathIndicator,
    PathTooDeep,
    InvalidPathComponent,
    InvalidXpubVersion,
    InvalidXpubPublicKey,
    UnexpectedEnd,
    TrailingBytes,
    CardPayloadTooLarge,
    XpubOriginPathMismatch,
}

impl ErrorVariantName {
    /// The `Error::Display` prefix the corpus's `expected_error` field
    /// should start with for this variant. Matches the format string in
    /// `crates/mk-codec/src/error.rs`'s `#[error("...")]` attributes.
    fn display_prefix(&self) -> &'static str {
        match self {
            Self::InvalidHrp => "invalid HRP",
            Self::MixedCase => "mixed case",
            Self::InvalidStringLength => "invalid data-part length",
            Self::InvalidChar => "invalid character",
            Self::BchUncorrectable => "BCH uncorrectable",
            Self::UnsupportedCardType => "unsupported card type",
            Self::MalformedPayloadPadding => "malformed payload padding",
            Self::ChunkSetIdMismatch => "chunk_set_id mismatch",
            Self::ChunkedHeaderMalformed => "chunked-header malformed",
            Self::MixedHeaderTypes => "mixed string-layer header types",
            Self::CrossChunkHashMismatch => "cross-chunk integrity hash mismatch",
            Self::UnsupportedVersion => "unsupported version",
            Self::ReservedBitsSet => "reserved bits set",
            Self::InvalidPolicyIdStubCount => "policy_id_stub_count must be >= 1",
            Self::InvalidPathIndicator => "invalid path indicator byte",
            Self::PathTooDeep => "path too deep",
            Self::InvalidPathComponent => "invalid path component",
            Self::InvalidXpubVersion => "invalid xpub version",
            Self::InvalidXpubPublicKey => "invalid xpub public key",
            Self::UnexpectedEnd => "unexpected end of bytecode",
            Self::TrailingBytes => "trailing bytes after xpub",
            Self::CardPayloadTooLarge => "card payload too large",
            Self::XpubOriginPathMismatch => "xpub origin-path mismatch",
        }
    }
}

/// Variants the corpus is allowed to skip, with a one-line rationale.
/// Returning `Some(reason)` exempts the variant from the
/// "must-have-negative-vector" check; returning `None` requires coverage.
fn is_exempt(variant: &ErrorVariantName) -> Option<&'static str> {
    match variant {
        ErrorVariantName::CardPayloadTooLarge => Some(
            "encoder-only: emitted from `split_into_chunks` (chunk.rs); not \
             reachable via `decode`'s string-input path because chunked input \
             is bounded by `MAX_CHUNKS=32 × 53-byte fragments = 1696 bytes` \
             stream, exactly the encoder's emit ceiling.",
        ),
        ErrorVariantName::XpubOriginPathMismatch => Some(
            "encoder-side invariant: emitted only from `encode_bytecode` when a \
             KeyCard's xpub.depth/child_number disagree with origin_path; not \
             reachable via `decode`'s string-input path (depth/child_number are \
             not carried on-wire, so there is nothing for a decoder to violate).",
        ),
        _ => None,
    }
}

const VECTOR_FILE: &str = "src/test_vectors/v0.1.json";

fn read_corpus() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(VECTOR_FILE);
    let bytes = fs::read(&path).expect("read src/test_vectors/v0.1.json");
    serde_json::from_slice(&bytes).expect("parse vectors JSON")
}

#[test]
fn every_error_variant_is_exercised_or_explicitly_exempt() {
    let doc = read_corpus();
    let vectors = doc["vectors"].as_array().expect("vectors is array");

    let mut missing: Vec<String> = Vec::new();

    for variant in ErrorVariantName::iter() {
        let prefix = variant.display_prefix();

        if let Some(reason) = is_exempt(&variant) {
            // Exempt variants MUST NOT have a corpus vector — that would be
            // contradictory documentation. (No corpus vector exists today
            // for any exempt variant; this branch encodes the contract.)
            let leaked = vectors.iter().any(|v| {
                v["expected_error"]
                    .as_str()
                    .map(|s| s.starts_with(prefix))
                    .unwrap_or(false)
            });
            assert!(
                !leaked,
                "variant {variant:?} is exempt ({reason}) but a corpus vector \
                 carries `expected_error` starting with {prefix:?} — \
                 either remove the vector or remove the exemption"
            );
            continue;
        }

        let covered = vectors.iter().any(|v| {
            v["expected_error"]
                .as_str()
                .map(|s| s.starts_with(prefix))
                .unwrap_or(false)
        });
        if !covered {
            missing.push(format!("{variant:?} (expected prefix: {prefix:?})"));
        }
    }

    assert!(
        missing.is_empty(),
        "negative-vector parity gap — the following Error variants have no \
         corpus vector pinning a matching `expected_error` prefix:\n  {}\n\n\
         To resolve: either add a negative vector to src/test_vectors/v0.1.json \
         (regenerate via gen_mk_vectors), or add the variant to is_exempt() \
         in this file with a one-line rationale.",
        missing.join("\n  ")
    );
}

/// Cross-check: ensure every negative vector's `expected_error` matches at
/// least one variant in the mirror enum. Catches typo'd `expected_error`
/// strings or stale vectors after a variant rename.
#[test]
fn every_negative_vector_maps_to_a_known_variant() {
    let doc = read_corpus();
    let vectors = doc["vectors"].as_array().expect("vectors is array");

    let prefixes: Vec<&'static str> = ErrorVariantName::iter()
        .map(|v| v.display_prefix())
        .collect();

    let mut orphans: Vec<String> = Vec::new();
    for v in vectors {
        let name = v["name"].as_str().unwrap_or("<unnamed>").to_string();
        let Some(expected) = v["expected_error"].as_str() else {
            continue; // clean vector — `expected_error` is null
        };
        let matches_any = prefixes.iter().any(|p| expected.starts_with(p));
        if !matches_any {
            orphans.push(format!("{name}: {expected:?}"));
        }
    }

    assert!(
        orphans.is_empty(),
        "the following negative vectors carry an `expected_error` that doesn't \
         start with any known Error variant's Display prefix — either fix the \
         vector or update ErrorVariantName / display_prefix() in this file:\n  {}",
        orphans.join("\n  ")
    );
}
