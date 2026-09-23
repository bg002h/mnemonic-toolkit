//! Error variants for the md-codec wire-format codec.

use thiserror::Error;

/// Operator-context kind — where in the descriptor tree an operator appears.
/// Per SPEC v0.30 §11. Carried by [`Error::OperatorContextViolation`] to name
/// which tree-position a forbidden tag was encountered in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextKind {
    /// Top-level descriptor position (e.g., bare `PkK` as the descriptor root).
    TopLevel,
    /// Inside a `tr()` tap-script leaf (BIP-342 tapscript-only operators).
    TapLeaf,
    /// Inside a multi-family body (non-key tag among multi children).
    MultiBody,
}

/// Errors produced by md-codec wire-format components.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// A read of `requested` bits was attempted but only `available` bits remained.
    #[error("attempted to read {requested} bits with only {available} bits remaining")]
    BitStreamTruncated {
        /// Number of bits the caller requested.
        requested: usize,
        /// Number of bits actually available in the stream.
        available: usize,
    },

    /// Wire-format version field is outside the accepted set (`{4, 8}`).
    /// Returned when a payload or chunk-header is read with a version value
    /// outside that set. Per SPEC v0.30 §2.4 + §2.5 + §11.1 and stage 1b's
    /// SPEC §6a row (the accepted set grew from `{4}` to `{4, 8}`).
    #[error("wire-format version mismatch: got {got}; accepted versions: 4, 8")]
    WireVersionMismatch {
        /// Version value parsed from the wire.
        got: u8,
    },

    /// Header malformed in a way other than version mismatch — e.g., chunked-
    /// flag inconsistent with caller context, or chunk-header internal field
    /// out of range. Per SPEC v0.30 §11.1.
    #[error("malformed header: {detail}")]
    MalformedHeader {
        /// Free-form description of the malformedness.
        detail: String,
    },

    /// Path depth exceeds MAX_PATH_COMPONENTS (15).
    #[error("path depth {got} exceeds maximum {max}")]
    PathDepthExceeded {
        /// Actual depth of the path.
        got: usize,
        /// Maximum allowed depth (15).
        max: usize,
    },

    /// Key count `n` out of range. Per SPEC v0.30 §4: `1 ≤ n ≤ 32`.
    #[error("key count {n} out of range; require 1 ≤ n ≤ 32")]
    KeyCountOutOfRange {
        /// Actual key count provided.
        n: u8,
    },

    /// Divergent path count doesn't match key count.
    #[error("divergent path count {got} does not match key count {n}")]
    DivergentPathCountMismatch {
        /// Expected key count.
        n: u8,
        /// Actual path count provided.
        got: usize,
    },

    /// Multipath alt-count out of range. Per SPEC v0.30 §8: `2 ≤ count ≤ 9`.
    #[error("multipath alt-count {got} out of range; require 2 ≤ count ≤ 9")]
    AltCountOutOfRange {
        /// Provided alt-count.
        got: usize,
    },

    /// Tag value outside the allocated v0.30 set: 6-bit primary in reserved
    /// range 0x24..=0x3E, or extension prefix 0x3F followed by an unrecognized
    /// 4-bit subcode 0x00..=0x0F (the entire extension subspace is reserved
    /// in v0.30). `primary` carries the raw 6-bit value read off the wire
    /// (0x3F for extension-subspace failures); the 4-bit subcode is consumed
    /// but not reported. Per SPEC v0.30 §3.2 and §11.1.
    #[error("tag value 0x{primary:02x} out of range")]
    TagOutOfRange {
        /// The raw 6-bit primary value read off the wire.
        primary: u8,
    },

    /// Threshold `k` out of range. Per SPEC v0.30 §4: `1 ≤ k ≤ 32`.
    #[error("threshold k={k} out of range; require 1 ≤ k ≤ 32")]
    ThresholdOutOfRange {
        /// Provided k value.
        k: u8,
    },

    /// Variable-arity child count out of range. Per SPEC v0.30 §4: `1 ≤ count ≤ 32`.
    #[error("child count {count} out of range; require 1 ≤ count ≤ 32")]
    ChildCountOutOfRange {
        /// Provided child count.
        count: usize,
    },

    /// k > n in k-of-n threshold/multisig.
    #[error("threshold k={k} exceeds child count n={n}; require k ≤ n")]
    KGreaterThanN {
        /// Threshold k.
        k: u8,
        /// Child count n.
        n: usize,
    },

    /// TLV ordering violation: a TLV tag was followed by a smaller-or-equal tag.
    #[error(
        "TLV ordering violation: tag 0x{prev:02x} followed by 0x{current:02x}; require ascending"
    )]
    TlvOrderingViolation {
        /// Previous tag value.
        prev: u8,
        /// Current tag value.
        current: u8,
    },

    /// Placeholder index in TLV entry exceeds key count n.
    #[error("placeholder index {idx} out of range; require idx < n={n}")]
    PlaceholderIndexOutOfRange {
        /// Provided index.
        idx: u8,
        /// Key count n.
        n: u8,
    },

    /// Per-`@N` override entries within a TLV must be in ascending `@N`-index order.
    #[error("override ordering violation: @{prev} followed by @{current}; require ascending")]
    OverrideOrderViolation {
        /// Previous index.
        prev: u8,
        /// Current index.
        current: u8,
    },

    /// TLV entry has zero entries; encoder MUST omit empty TLVs per spec §7.5.
    #[error("TLV entry tag 0x{tag:02x} has empty payload; encoder MUST omit empty TLVs")]
    EmptyTlvEntry {
        /// Tag of the empty entry.
        tag: u8,
    },

    /// TLV length exceeds remaining bits in stream.
    #[error("TLV length {length} exceeds remaining bits {remaining}")]
    TlvLengthExceedsRemaining {
        /// Declared length.
        length: usize,
        /// Available bits.
        remaining: usize,
    },

    /// Placeholder @i was not referenced anywhere in the tree (BIP 388 well-formedness).
    #[error("placeholder @{idx} not referenced in tree; n={n}")]
    PlaceholderNotReferenced {
        /// The unreferenced placeholder index.
        idx: u8,
        /// Key count.
        n: u8,
    },

    /// First-occurrence ordering violated (BIP 388 well-formedness).
    #[error(
        "placeholder first-occurrence ordering violated: expected first={expected_first}, got first={got_first}"
    )]
    PlaceholderFirstOccurrenceOutOfOrder {
        /// Expected placeholder index in canonical first-occurrence position.
        expected_first: u8,
        /// Actual placeholder index encountered first.
        got_first: u8,
    },

    /// All multipaths in a template must share the same alt-count.
    #[error("multipath alt-count mismatch: expected {expected}, got {got}")]
    MultipathAltCountMismatch {
        /// Expected alt-count.
        expected: usize,
        /// Mismatched alt-count.
        got: usize,
    },

    /// A `use_site_path_overrides` entry was keyed on `@0`. `@0` is the
    /// canonical baseline (`Descriptor::use_site_path`) and cannot be
    /// overridden — an `@0` entry is a non-canonical / adversarial wire
    /// (our encoders only push overrides for `i ≥ 1`). Per the D5(a) decode
    /// canonical-form check (`restore-md1-per-key-use-site` SPEC §4.1).
    #[error("use-site override keyed on baseline @{idx}; @0 cannot be overridden")]
    BaselineUseSiteOverride {
        /// The offending index (always 0).
        idx: u8,
    },

    /// A `use_site_path_overrides` entry's `UseSitePath` equaled the
    /// resolved baseline (`Descriptor::use_site_path`). A redundant override
    /// is non-canonical (our encoders push an override only when it DIFFERS
    /// from the baseline) and is rejected at decode. Per the D5(a) decode
    /// canonical-form check.
    #[error("redundant use-site override for @{idx}; equals the baseline use-site path")]
    RedundantUseSiteOverride {
        /// The placeholder index whose override duplicates the baseline.
        idx: u8,
    },

    /// Tap-script-tree leaf has a tag that is forbidden per spec §6.3.1.
    #[error("forbidden tap-script-tree leaf tag: 0x{tag:02x}")]
    ForbiddenTapTreeLeaf {
        /// Primary 6-bit tag code (bytecode space) of the forbidden leaf.
        tag: u8,
    },

    /// Operator appears in a forbidden context per SPEC v0.30 §11.
    /// `TopLevel` is enforced decoder-side at `decode_payload`; `TapLeaf` is
    /// covered by the narrower [`Error::ForbiddenTapTreeLeaf`]; `MultiBody` is
    /// structurally unreachable post-v0.30 Phase C (multi-family bodies carry
    /// raw kiw-bit indices, not child tags).
    #[error("operator {tag:?} not allowed in context {context:?}")]
    OperatorContextViolation {
        /// The offending operator tag.
        tag: crate::tag::Tag,
        /// Which tree-position the tag is forbidden in.
        context: ContextKind,
    },

    /// Chunk count out of range. Per SPEC v0.30 §2.5: `1 ≤ count ≤ 64`.
    #[error("chunk count {count} out of range; require 1 ≤ count ≤ 64")]
    ChunkCountOutOfRange {
        /// Provided count.
        count: u8,
    },

    /// Chunk index ≥ count; require index < count.
    #[error("chunk index {index} ≥ count {count}")]
    ChunkIndexOutOfRange {
        /// Provided index.
        index: u8,
        /// Provided count.
        count: u8,
    },

    /// Chunk-set-id exceeds 20-bit range.
    #[error("chunk-set-id 0x{id:x} exceeds 20-bit range")]
    ChunkSetIdOutOfRange {
        /// Provided ID.
        id: u32,
    },

    /// Chunk header missing chunked-flag. Per SPEC v0.30 §2.2: bit 0 of the
    /// first 5-bit symbol of a chunked payload is the chunked-flag (followed
    /// by the 4-bit version field); it MUST be 1 in every chunk header.
    #[error("chunk header chunked-flag missing; per SPEC §2.2 bit 0 of the first symbol must be 1")]
    ChunkHeaderChunkedFlagMissing,

    /// Encoding requires more chunks than the spec maximum (64).
    ///
    /// A SECOND CEILING, independent of the composer's slot and path limits
    /// (F-515). This one is a function of the encoded SIZE — the keys, their
    /// origins and every lock operand — so a policy that passes every documented
    /// composer limit can still land here, and it lands here at `encode`, after
    /// the operator has chosen both the shape and the keys. The message names
    /// the levers because "too big" alone leaves them guessing which of three
    /// things to change.
    #[error(
        "encoding requires {needed} chunks; max is 64 per spec §9.8. \
         This ceiling is on the encoded SIZE, not the slot count, so it is not \
         implied by the composer's limits: reduce the number of keys, shorten \
         their origin paths, or drop a spend path"
    )]
    ChunkCountExceedsMax {
        /// Number of chunks needed.
        needed: usize,
    },

    /// Codex32 decode error (HRP mismatch, alphabet violation, BCH verification failure).
    #[error("codex32 decode error: {0}")]
    Codex32DecodeError(String),

    /// Codex32 encode error (BCH layer failure).
    #[error("codex32 encode error: {0}")]
    Codex32EncodeError(String),

    /// Chunk set is empty (no strings provided to reassemble).
    #[error("chunk set is empty (no strings provided)")]
    ChunkSetEmpty,

    /// Chunks in the set disagree on version, chunk-set-id, or count.
    #[error("chunks in the set disagree on version, chunk-set-id, or count")]
    ChunkSetInconsistent,

    /// Chunk set incomplete: got fewer chunks than `expected`.
    #[error("chunk set incomplete: got {got} chunks, expected {expected}")]
    ChunkSetIncomplete {
        /// Provided chunk count.
        got: usize,
        /// Expected chunk count.
        expected: usize,
    },

    /// Chunk index gap: expected index N, got M.
    #[error("chunk index gap: expected index {expected}, got {got}")]
    ChunkIndexGap {
        /// Expected index in the sequence.
        expected: u8,
        /// Actual index encountered.
        got: u8,
    },

    /// Chunk-set-id mismatch between expected and reassembled-then-derived.
    #[error("chunk-set-id mismatch: expected 0x{expected:x}, derived 0x{derived:x}")]
    ChunkSetIdMismatch {
        /// Expected (from chunks).
        expected: u32,
        /// Derived (from reassembled payload).
        derived: u32,
    },

    /// LP4-ext varint value exceeds single-extension payload range (29 bits).
    #[error("varint value {value} exceeds single-extension range (max 2^29 - 1)")]
    VarintOverflow {
        /// The offending value.
        value: u32,
    },

    /// A non-canonical wrapper has no explicit origin path for some `@N`,
    /// either via `OriginPathOverrides` or a populated `path_decl` entry,
    /// and `canonical_origin(&d.tree)` is `None`. Per spec v0.13 §6.3.
    #[error("non-canonical wrapper requires explicit origin for @{idx}, but none provided")]
    MissingExplicitOrigin {
        /// The placeholder index for which an explicit origin is required.
        idx: u8,
    },

    /// An `OriginPathOverrides[idx]` entry is PRESENT but carries zero path
    /// components. A present-but-empty override is MALFORMED — distinct
    /// from an ABSENT override, which the shared/divergent `path_decl` may
    /// still resolve. `crate::canonicalize::expand_per_at_n` treats a
    /// present override as authoritative over `path_decl` regardless of
    /// its component count, so an empty-but-present override silently
    /// resolves to "no origin" unless rejected explicitly. Rejected
    /// UNCONDITIONALLY (even for a CANONICAL-shape wrapper, e.g.
    /// `wpkh(@0)`) by both the decoder (`crate::validate::
    /// validate_no_empty_origin_overrides`) and `expand_per_at_n`; this is
    /// a DISTINCT error variant from `MissingExplicitOrigin` so it is
    /// never swallowed by partial-allowing decode (P0 pathless/dead-card
    /// partial-decode) — fatal-in-partial. Per spec v0.13 §6.3 (I-1
    /// hardening).
    #[error("origin-path override for @{idx} is present but empty (zero components)")]
    EmptyOriginOverride {
        /// The placeholder index whose override entry is empty.
        idx: u8,
    },

    /// `presence_byte` had non-zero reserved bits (bits 2..7) inside a
    /// `WalletPolicyId` canonical-record preimage. Per spec v0.13 §5.3:
    /// encoders MUST set reserved bits to 0 and decoders MUST reject
    /// inputs with non-zero reserved bits. v0.13's encoder masks reserved
    /// bits explicitly when building the hash preimage; the helper
    /// [`crate::identity::validate_presence_byte`] enforces the
    /// decoder-side contract for canonical-record consumers.
    #[error("WalletPolicyId presence_byte has non-zero reserved bits: 0x{reserved_bits:02x}")]
    InvalidPresenceByte {
        /// The reserved-bit field (bits 2..7) of the offending presence byte.
        reserved_bits: u8,
    },

    /// Two `@N` slots carry the SAME key AND the same use-site, so they
    /// derive an identical child at every address index.
    ///
    /// The policy reads as k-of-n and is satisfiable by fewer parties than it
    /// names: with one key seated twice, its holder can produce two of the
    /// required signatures alone. Legal script, misleading wallet.
    ///
    /// The comparison is the 65-byte `chain code ‖ compressed pubkey` PLUS the
    /// use-site. Not the fingerprint, which identifies a MASTER rather than a
    /// key and would refuse the legitimate multi-account cosigner; not the
    /// base58 xpub, which carries depth/parent metadata that differs between
    /// two sources of the same key. And not the key alone: the same xpub at
    /// two different multipath branches derives DIFFERENT children at every
    /// index, which is a different wallet, not a duplicate. Measured —
    /// `<0;1>` and `<2;3>` over one xpub give different addresses.
    #[error(
        "@{a} and @{b} carry the same key at the same use-site: this policy names \
         {n} cosigners but one of them holds two of the seats"
    )]
    DuplicateKeySlots {
        /// The lower placeholder index of the duplicated pair.
        a: u8,
        /// The higher placeholder index.
        b: u8,
        /// How many `@N` slots the policy declares.
        n: u8,
    },

    /// Two `@N` slots declare the SAME master fingerprint and the SAME
    /// origin path while carrying DIFFERENT xpubs. That is impossible:
    /// BIP-32 is deterministic, so a `(fingerprint, path)` pair identifies
    /// exactly one extended key.
    ///
    /// Detectable from the card alone — no seed, no network, no
    /// derivation — which is why it is refused rather than warned about.
    /// Nothing downstream can catch it: addresses derive from the xpubs a
    /// card CARRIES, not from the origin it declares, so every address
    /// check passes either way and the failure surfaces only when someone
    /// asks a signer to find the key. See F-217.
    #[error(
        "@{a} and @{b} declare the same key origin ([{fingerprint}/{path}]) but different xpubs; \
         one origin identifies exactly one key, so this card describes a wallet that cannot exist"
    )]
    OriginKeyContradiction {
        /// The lower placeholder index of the conflicting pair.
        a: u8,
        /// The higher placeholder index of the conflicting pair.
        b: u8,
        /// The shared master fingerprint, hex.
        fingerprint: String,
        /// The shared origin path, rendered.
        path: String,
    },

    /// A `Pubkeys` TLV entry's 33-byte compressed-pubkey field (bytes
    /// 32..65 of the 65-byte xpub payload) failed to parse as a valid
    /// secp256k1 point. The 32-byte chain code prefix is unvalidated.
    /// Per spec v0.13 §6.4.
    #[error("invalid xpub bytes for @{idx}: pubkey field is not a valid secp256k1 point")]
    InvalidXpubBytes {
        /// The placeholder index whose xpub failed to parse.
        idx: u8,
    },
    /// Address derivation requires a populated `Pubkeys` TLV entry for
    /// every `@N`; this descriptor is missing one (template-only or
    /// partial-keys mode). v0.14+ derivation surface only.
    #[error(
        "missing xpub for @{idx}; address derivation requires wallet-policy mode with all @N populated"
    )]
    MissingPubkey {
        /// The placeholder index whose xpub is absent.
        idx: u8,
    },

    /// `Descriptor::derive_address` was called with a `chain` index
    /// outside the use-site multipath alt-count (or non-zero when no
    /// multipath is present).
    #[error("chain index {chain} out of range; use-site multipath alt-count is {alt_count}")]
    ChainIndexOutOfRange {
        /// The provided chain index.
        chain: u32,
        /// The number of alternatives in the use-site multipath (`0` when
        /// no multipath component is present).
        alt_count: usize,
    },

    /// Address derivation requires non-hardened use-site components,
    /// but this descriptor's use-site path declares a hardened
    /// alternative or hardened wildcard. BIP 32 forbids hardened
    /// derivation from a public key, so an xpub-only restore cannot
    /// produce addresses for this wallet.
    #[error(
        "hardened public-key derivation: use-site path requires hardened component, which BIP 32 forbids on xpub-only restore"
    )]
    HardenedPublicDerivation,

    /// Address derivation failed at the miniscript layer (or in the
    /// AST → miniscript converter). Carries a free-form `detail` string
    /// describing the underlying error — typically a `miniscript::Error`,
    /// a `Tr`/`Wsh` constructor failure (type-check / context error), or
    /// an arity/context mismatch raised by the converter.
    #[error("address derivation failed: {detail}")]
    AddressDerivationFailed {
        /// Free-form description of the underlying failure.
        detail: String,
    },

    /// [`crate::to_miniscript::to_miniscript_descriptor`] /
    /// [`crate::to_miniscript::to_miniscript_descriptor_multipath`] (the
    /// network-less entry points) were called on a descriptor whose
    /// [`crate::encode::Descriptor::wire_version`] is
    /// [`crate::header::Header::WF_UNSPENDABLE_VERSION`] — i.e. it carries a
    /// wire-kind-1 (Liana unspendable) taproot internal key somewhere in its
    /// tree. SPEC §2 step 5: the derived internal key's base58 prefix
    /// (`xpub`/`tpub`) is network-dependent, and the network-less entry
    /// points delegate to mainnet — silently rendering a mainnet xpub for a
    /// testnet wallet is exactly the funds-safety defect this refusal
    /// exists to prevent. Use the `_with_network` entry point instead.
    #[error(
        "a wire-kind-1 (Liana unspendable) internal key needs a network to render its derived xpub prefix; call the _with_network entry point instead of guessing mainnet"
    )]
    NetworkRequiredForUnspendable,

    /// Inside a `tr()` body, `is_nums = false` was paired with a `key_index`
    /// out of range (`key_index >= n`). Per SPEC v0.30 §7 + §11: the
    /// placeholder-index range is `0..n` strictly; the v0.x NUMS sentinel
    /// slot at `key_index = n` is gone (NUMS is now flag-driven via
    /// `Body::Tr.is_nums`). Raised by `validate_placeholder_usage` when the
    /// in-`tr()` overflow condition is hit.
    #[error("NUMS sentinel conflict: is_nums=false with key_index out of range (SPEC §7 §11)")]
    NUMSSentinelConflict,

    /// Decode-side recursion depth exceeded the hardening cap.
    /// `read_node` calls itself recursively for tags with child bodies
    /// (`Tag::Sh`, `Tag::AndV`, `Tag::TapTree`, `Tag::Multi`, `Tag::Tr`,
    /// etc.); a hostile wire payload nesting these tags arbitrarily deep
    /// would blow the Rust stack. The cap is shared across all recursive
    /// tags as a generic anti-DOS hardening bound. v0.19 introduced.
    #[error("decode recursion depth {depth} exceeded maximum {max}")]
    DecodeRecursionDepthExceeded {
        /// Current recursion depth at which the cap fired.
        depth: u8,
        /// Maximum allowed depth.
        max: u8,
    },

    /// BCH correction capacity exceeded: a chunk's syndrome pattern indicated
    /// more errors than the BCH(93, 80, 8) code can correct. F-A9: the
    /// *correction* capacity is `t = 4` substitution errors; the code's
    /// `2t = 8` figure is its *detection* radius (the singleton bound), NOT the
    /// number of correctable errors — the two must not be conflated. A pattern
    /// beyond `t = 4` has no unique correction. v0.34.0 introduced; raised by
    /// [`crate::decode_with_correction`]. Atomic per plan §1 D28: any chunk
    /// failing this check fails the whole multi-chunk call without partial
    /// output.
    #[error(
        "chunk {chunk_index} exceeds the BCH correction capacity of t=4 substitution errors; uncorrectable"
    )]
    TooManyErrors {
        /// 0-indexed position of the offending chunk in the caller's
        /// `&[&str]` slice.
        chunk_index: usize,
        /// The BCH singleton (detection) bound `2t = 8`. Note this is the
        /// detection radius, not the `t = 4` correction capacity stated in the
        /// user-facing message; the field is retained for callers/tests that
        /// pin the code's `2t` parameter.
        bound: u8,
    },

    /// An `older()` value carrying bits that BIP-68 consensus IGNORES, so the
    /// written number is not the delay that is enforced.
    ///
    /// BIP-68 reads only bit 31 (disable), bit 22 (units: blocks vs 512s) and
    /// bits 0-15 (the value). Every other bit is discarded. `older(210000)`
    /// therefore enforces `210000 & 0xFFFF` = 13392 blocks, and
    /// `older(65536)` enforces ZERO — no timelock at all — while both
    /// round-trip through this codec unchanged.
    ///
    /// Refused at encode because the artifact is engraved in metal: a plate
    /// asserting a four-year lock the chain will release in three months is
    /// a funds-safety defect, not a rendering one. A relative lock simply
    /// cannot express a delay above 65535 blocks (or 65535 x 512s ~ 388 days);
    /// use an absolute `after()` height instead.
    #[error(
        "older({written}) is not what consensus enforces: BIP-68 reads only the low 16 bits (and bit 22 for units), so this locks for {enforced} {units}, not {written}. A relative timelock cannot exceed 65535 {units} -- use an absolute after() height for longer delays"
    )]
    RelativeTimelockTruncated {
        /// The value written in the descriptor.
        written: u32,
        /// What BIP-68 actually enforces.
        enforced: u32,
        /// "blocks" or "512-second units".
        units: &'static str,
    },

    /// Encode-side cap (cycle-4 H6): a single codex32 string's data part
    /// exceeded the regular code's `REGULAR_DATA_SYMBOLS_MAX = 80` symbols.
    /// The codex32 regular code is BCH(93, 80, 8); a single string therefore
    /// carries at most 80 data symbols + 13 checksum = 93. `wrap_payload`
    /// rejects an over-length payload (fail-closed) rather than emit an
    /// un-decodable / aliasing-prone single string — callers needing more
    /// capacity must use chunked encoding (`--force-chunked` / `split()`).
    #[error(
        "payload is {data_symbols} data symbols; the codex32 regular code caps single strings at {max} (use chunked encoding / --force-chunked)"
    )]
    PayloadTooLongForSingleString {
        /// The over-length data-symbol count actually computed.
        data_symbols: usize,
        /// The maximum legal data-symbol count (80).
        max: usize,
    },

    /// Decode-side cap, correcting path (cycle-4 M4): a chunk handed to the
    /// BCH-correcting decoder (`decode_with_correction`) had more than 93
    /// symbols. The codex32 regular code's generator `β` has order 93, so a
    /// degree `d` and `d + 93` alias in `chien_search` for an over-93-symbol
    /// word — the correcting decoder would mis-correct at an aliased root.
    /// Reject the out-of-domain chunk before correction (fail-closed).
    #[error(
        "chunk {chunk_index} has {symbols} symbols; the codex32 regular code caps a string at {max}"
    )]
    ChunkSymbolCountOutOfRange {
        /// 0-indexed position of the offending chunk in the caller's slice.
        chunk_index: usize,
        /// The over-length symbol count actually supplied.
        symbols: usize,
        /// The maximum legal codeword length (93).
        max: usize,
    },

    /// Decode-side cap, non-correcting path (cycle-4 I1 / §5.2.3): a single
    /// md1 string handed to the non-correcting primitive (`unwrap_string` /
    /// `decode_md1_string`) had more than 93 symbols. A clean (residue == 0)
    /// over-length word is BCH-verifiable by the length-agnostic
    /// `bch_verify_regular` but is structurally out-of-domain for the regular
    /// code; reject it before BCH verification (fail-closed). No chunk index
    /// (single string, not a chunk).
    #[error("string has {symbols} symbols; the codex32 regular code caps a string at {max}")]
    StringSymbolCountOutOfRange {
        /// The over-length symbol count actually supplied.
        symbols: usize,
        /// The maximum legal codeword length (93).
        max: usize,
    },

    /// F-A8: the ≤7 trailing bits after the last TLV entry (or after the tree
    /// when no TLVs are present) are byte-padding bits that the reference
    /// encoder ALWAYS emits as zero (BitWriter + `wrap_payload` zero-pad to the
    /// next byte boundary). A non-zero trailing pad is a malformed / hand-forged
    /// wire, never produced by our encoders; the TLV rollback rejects it
    /// (fail-closed) instead of silently discarding the non-zero bits. This is
    /// the real error variant the BIP's §Padding rule cites for a non-zero
    /// trailing pad (F-A8 / DG-5).
    #[error("malformed payload padding: {bits} trailing pad bit(s) were not all zero")]
    MalformedPayloadPadding {
        /// Number of trailing pad bits inspected (1..=7).
        bits: usize,
    },

    /// SPEC §6 row 1: a wire-kind-1 (Liana unspendable) `tr()` whose own
    /// tap-script tree contains a `sortedmulti_a` leaf anywhere. Refused as
    /// a belt against a port error: `MultiALeafScript` sorts the
    /// *serialized derived* x-only keys when building the script, while §2's
    /// recipe hashes the leaves' *account-level* 33-byte pubkeys in
    /// unsorted, wire order — a port that feeds the already-sorted script
    /// order into the recipe would silently diverge. `md encode` accepts a
    /// `sortedmulti_a` leaf under any internal key, so this refusal is not
    /// vacuous; cost is near-zero, since Liana itself emits `multi_a`, never
    /// `sortedmulti_a`.
    #[error(
        "wire kind 1 (Liana unspendable internal key) with a sortedmulti_a leaf is refused: a \
         correct implementation hashes the leaves' unsorted, wire-order pubkeys (SPEC §2), and a \
         sortedmulti_a leaf is exactly the shape a sorting port error would silently diverge on"
    )]
    UnspendableWithSortedMultiA,

    /// SPEC §6 row 2: a wire-kind-1 `tr()` whose use-site path is not the
    /// canonical `<0;1>/*` form (`crate::use_site_path::UseSitePath::standard_multipath`).
    /// Liana pairs multipath alternatives positionally (branch 0 = receive,
    /// branch 1 = change); a use-site of e.g. `<2;3>` would have Liana
    /// deriving from `0/i` while a use-site-following device derives from
    /// `2/i` — two different wallets sharing one plate. Refusing is
    /// narrower than reconciling the two conventions.
    ///
    /// F-638 (F-449 stage 2): `idx` names the placeholder when the divergence
    /// is a per-key use-site OVERRIDE (`@idx` has its own path while the
    /// shared use-site is canonical) -- otherwise the message would point
    /// the operator at a shared field they can see is correct, with no
    /// locator for the key that is wrong. `None` when the SHARED use-site
    /// itself diverged. Message-only: the set of refused inputs is
    /// unchanged.
    #[error(
        "wire kind 1 (Liana unspendable internal key) requires the canonical <0;1>/* use-site \
         path{}; Liana pairs multipath alternatives positionally, so any other use-site derives a \
         different wallet than the one this card's addresses would show",
        use_site_locator(*.idx)
    )]
    UnspendableUseSiteNotCanonical {
        /// The placeholder whose per-key override diverged, or `None` when
        /// the shared use-site path did.
        idx: Option<u8>,
    },

    /// SPEC §6 row 4: a wire-kind-1 internal key found on a `tr()` node that
    /// is not the descriptor's own root (e.g. `wsh(tr(...))`). The internal
    /// key of a `tr()` nested inside `sh`/`wsh` is not the descriptor's
    /// internal key, so a "derived unspendable key" has no meaning there —
    /// there is no wallet-level taproot output for it to describe.
    #[error(
        "wire kind 1 (Liana unspendable internal key) is only meaningful on the descriptor's \
         root tr(); found nested under an outer wrapper, where it has no meaning"
    )]
    UnspendableNotRootTr,

    /// SPEC §6 row 6 (the minimum-version rule): the wire version about to
    /// be written is [`crate::header::Header::WF_UNSPENDABLE_VERSION`] (8),
    /// but no node in the tree needs it — i.e.
    /// [`crate::encode::Descriptor::wire_version`] computes a lower minimal
    /// version for the same tree. Enforced at ENCODE only, never at decode
    /// (old payloads must keep decoding even if a future encoder becomes
    /// stricter): a non-minimal version would otherwise admit a second,
    /// distinct on-wire encoding of a kind-0 wallet with a different
    /// `WalletDescriptorTemplateId` than its minimal encoding produces. Not
    /// reachable through any public encoder — `encode_payload_inner` always
    /// derives its version from `Descriptor::wire_version()`, which is
    /// minimal by construction; only a test helper that forces a version can
    /// hit this.
    #[error(
        "wire version {got} was requested but the tree needs only version {minimal}: write the \
         minimal version a tree needs, never a higher one"
    )]
    NonMinimalWireVersion {
        /// The version actually requested for this encode.
        got: u8,
        /// The minimal version `Descriptor::wire_version()` computes for the tree.
        minimal: u8,
    },
}

/// `Error::UnspendableUseSiteNotCanonical`'s locator (F-638): empty for the
/// shared use-site, a named placeholder for a per-key override.
fn use_site_locator(idx: Option<u8>) -> String {
    match idx {
        None => String::new(),
        Some(i) => format!(
            " at every key, and @{i} carries its own different use-site path (the shared \
             use-site is canonical; @{i}'s override is what diverged)"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::Tag;

    /// SPEC v0.30 §11: `OperatorContextViolation` carries the offending tag
    /// + a `ContextKind` discriminator. Pins the type shape and Display
    ///   output against future drift; does NOT claim live wire reachability
    ///   (see FOLLOWUP `v0.30-phase-g-operator-context-violation-unwired`).
    #[test]
    fn operator_context_violation_constructs() {
        let err = Error::OperatorContextViolation {
            tag: Tag::Multi,
            context: ContextKind::MultiBody,
        };
        let s = err.to_string();
        assert!(s.contains("Multi"), "Display must mention tag: {s}");
        assert!(s.contains("MultiBody"), "Display must mention context: {s}");
    }

    /// SPEC v0.30 §7 + §11: `NUMSSentinelConflict` Display pins the SPEC-cite
    /// substring so the doc-comment + format string don't silently drift.
    #[test]
    fn nums_sentinel_conflict_display() {
        let s = Error::NUMSSentinelConflict.to_string();
        assert!(s.contains("§7"), "Display must cite SPEC §7: {s}");
        assert!(s.contains("§11"), "Display must cite SPEC §11: {s}");
    }

    /// F-A9: `TooManyErrors` Display must state the correction capacity
    /// `t = 4`, not conflate it with the `2t = 8` detection radius. The old
    /// "more than 8 errors" text read as if 8 substitutions were correctable.
    #[test]
    fn too_many_errors_message_states_correction_capacity() {
        let s = Error::TooManyErrors {
            chunk_index: 0,
            bound: 8,
        }
        .to_string();
        assert!(
            s.contains("t=4") || s.contains("t = 4"),
            "Display must state correction capacity t=4: {s}"
        );
        assert!(
            !s.contains("more than 8"),
            "Display must not conflate the 2t=8 detection radius with correction: {s}"
        );
    }
}
