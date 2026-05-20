//! `mnemonic import-wallet` format-parser surface.
//!
//! Counterpart to `crate::wallet_export`. Each per-format parser implements
//! `WalletFormatParser` (associated-function shape, no `&self`; matches
//! `WalletFormatEmitter` discipline at `wallet_export/mod.rs:322`).
//!
//! Per `design/SPEC_wallet_import_v0_26_0.md` §8.
//!
//! Submodule tree:
//! - `pipeline` — concrete-keys `[fp/path]xpub` → `@N`-placeholder adapter.
//! - `bsms` — BIP-129 Round-2 parser (2-line + 6-line shapes).
//! - `bitcoin_core` (Phase 3) — Bitcoin Core `listdescriptors` parser.
//!
//! The trait `parse()` accepts an `&mut dyn Write` stderr handle. WARNINGs
//! (e.g., 2-line reduced-form, first-address mismatch) are written directly
//! to the handle so callers can route them to the process stderr or buffer
//! them for tests. This matches the `cmd/*.rs::run(&mut dyn Write, ...)`
//! discipline elsewhere in the codebase.

use crate::error::ToolkitError;
use crate::synthesize::ResolvedSlot;
use std::io::Write;

pub(crate) mod bitcoin_core;
pub(crate) mod bsms;
pub(crate) mod bsms_round1;
pub(crate) mod bsms_verify;
pub(crate) mod coldcard;
pub(crate) mod coldcard_multisig;
pub(crate) mod json_envelope;
pub(crate) mod overlay;
pub(crate) mod pipeline;
pub(crate) mod roundtrip;
pub(crate) mod sniff;
pub(crate) mod sparrow;
pub(crate) mod specter;

/// SPEC §8.1 — every per-format parser implements this trait. Associated-
/// function shape (no `&self`); dispatch is `match format { ... }`-style at
/// the call site (not `dyn WalletFormatParser`). The trait is not
/// object-safe by design.
pub(crate) trait WalletFormatParser {
    /// Heuristic detection: return `true` if `blob` looks like this format.
    /// Used by `wallet_import::sniff::sniff_format` to auto-detect when
    /// `--format` is not supplied.
    fn sniff(blob: &[u8]) -> bool;

    /// Parse the blob into one or more `ParsedImport` bundles. WARNINGs go to
    /// `stderr` directly; the returned `Vec` is the canonical bundle list.
    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError>;
}

/// Source-provenance of a parsed wallet import. Replaces the v0.26.0 / v0.27.x
/// representable-invalid `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)`
/// pair on `ParsedImport`. Exactly one variant per parse — enforced at the type
/// level.
///
/// See `design/FOLLOWUPS.md::pr-26-import-provenance-enum-internal-refactor`
/// for rationale (v0.27.1 Phase 5b deferral).
///
/// Visibility is `pub(crate)` to match the existing `ParsedImport` /
/// `BsmsAuditFields` / `CoreSourceMetadata` types (all `pub(crate)` per
/// grep-verified `src/wallet_import/mod.rs:60,88,188`). Bumping to `pub`
/// would require E0446-fixing all transitively-referenced types — out of
/// scope for this internal refactor.
#[derive(Debug, Clone)]
pub(crate) enum ImportProvenance {
    /// Bitcoin Core `listdescriptors` parse (`wallet_import/bitcoin_core.rs`).
    BitcoinCore(CoreSourceMetadata),
    /// BSMS Round-2 parse (`wallet_import/bsms.rs`). Holds `Option` because
    /// the lenient 2-line excerpt shape carries no audit fields (token /
    /// signature / first_address / derivation_path absent); the 6-line full
    /// BIP-129 Round-2 shape populates `Some(BsmsAuditFields)`.
    Bsms(Option<BsmsAuditFields>),
    /// Coldcard single-sig generic-wallet-export JSON parse
    /// (`wallet_import/coldcard.rs`). SPEC §11.3. Inserted in
    /// alphabetical-by-variant-name slot per CLAUDE.md discipline (between
    /// `Bsms` and `ColdcardMultisig`).
    ///
    /// Constructed by `ColdcardParser::parse` (Phase P3B). The
    /// `cmd/import_wallet.rs` dispatch arm wired at P3C plumbs this variant
    /// to the `--json` envelope `coldcard_source_metadata` field; the
    /// `dead_code` allow covers the P3B → P3C interim.
    #[allow(dead_code)]
    Coldcard(coldcard::ColdcardSourceMetadata),
    /// Coldcard multisig text-file parse (`wallet_import/coldcard_multisig.rs`).
    /// SPEC §11.4. Inserted in alphabetical-by-variant-name slot per CLAUDE.md
    /// discipline; the prior `Coldcard(...)` slot (single-sig, SPEC §11.3) is
    /// added in Phase P3B and lands at the alphabetically-preceding position
    /// without affecting this insertion.
    ///
    /// The variant is constructed by `ColdcardMultisigParser::parse` at P4B
    /// and the `cmd/import_wallet.rs` dispatch arm wired at P4C; the
    /// `dead_code` allow on the variant covers the P4A → P4C interim
    /// (the type exists for downstream-consumer reference + dispatch
    /// stitching but is not yet constructed by any wired call site).
    #[allow(dead_code)]
    ColdcardMultisig(coldcard_multisig::ColdcardMultisigSourceMetadata),
    /// Sparrow Wallet JSON parse (`wallet_import/sparrow.rs`). SPEC §11.1.
    /// Inserted in alphabetical-by-variant-name slot per CLAUDE.md discipline;
    /// the future `Specter(...)` slot (SPEC §11.2, Phase P2) is added in a
    /// later phase and lands at the alphabetically-following position
    /// without affecting this insertion.
    ///
    /// Constructed by `SparrowParser::parse` (Phase P1B). The
    /// `cmd/import_wallet.rs` dispatch arm wired at P1C plumbs this variant
    /// to the `--json` envelope; until then the variant is reachable only
    /// from `wallet_import::sparrow::tests`.
    Sparrow(sparrow::SparrowSourceMetadata),
    /// Specter-DIY wallet JSON parse (`wallet_import/specter.rs`). SPEC §11.2.
    /// Inserted in alphabetical-by-variant-name slot per CLAUDE.md discipline
    /// (after `Sparrow`).
    ///
    /// Constructed by `SpecterParser::parse` (Phase P2B) and consumed by
    /// the `cmd/import_wallet.rs` dispatch arm at P2C which plumbs this
    /// variant to the `--json` envelope `specter_source_metadata` field.
    Specter(specter::SpecterSourceMetadata),
}

impl ImportProvenance {
    /// Back-compat accessor: returns `Some(&audit)` for the `Bsms` variant
    /// when audit fields are present (6-line shape); `None` for the 2-line
    /// excerpt shape or for any non-BSMS variant.
    pub(crate) fn bsms_audit(&self) -> Option<&BsmsAuditFields> {
        match self {
            Self::BitcoinCore(_) => None,
            Self::Bsms(audit) => audit.as_ref(),
            Self::Coldcard(_) => None,
            Self::ColdcardMultisig(_) => None,
            Self::Sparrow(_) => None,
            Self::Specter(_) => None,
        }
    }

    /// Back-compat accessor: returns `Some(&metadata)` only for the `BitcoinCore` variant.
    pub(crate) fn source_metadata(&self) -> Option<&CoreSourceMetadata> {
        match self {
            Self::BitcoinCore(meta) => Some(meta),
            Self::Bsms(_) => None,
            Self::Coldcard(_) => None,
            Self::ColdcardMultisig(_) => None,
            Self::Sparrow(_) => None,
            Self::Specter(_) => None,
        }
    }

    /// Coldcard-specific accessor: returns `Some(&metadata)` only for the
    /// `Coldcard` variant. Consumed by the `--json` envelope emitter in
    /// `cmd::import_wallet::emit_json_envelope` (P3C wiring). Mirrors
    /// `sparrow_source_metadata` / `specter_source_metadata`.
    ///
    /// `dead_code` allow covers the P3B → P3C interim (called from P3C's
    /// envelope-emit site only).
    #[allow(dead_code)]
    pub(crate) fn coldcard_source_metadata(&self) -> Option<&coldcard::ColdcardSourceMetadata> {
        match self {
            Self::BitcoinCore(_) => None,
            Self::Bsms(_) => None,
            Self::Coldcard(meta) => Some(meta),
            Self::ColdcardMultisig(_) => None,
            Self::Sparrow(_) => None,
            Self::Specter(_) => None,
        }
    }

    /// Sparrow-specific accessor: returns `Some(&metadata)` only for the
    /// `Sparrow` variant. Consumed by the `--json` envelope emitter in
    /// `cmd::import_wallet::emit_json_envelope` (P1C wiring).
    pub(crate) fn sparrow_source_metadata(&self) -> Option<&sparrow::SparrowSourceMetadata> {
        match self {
            Self::BitcoinCore(_) => None,
            Self::Bsms(_) => None,
            Self::Coldcard(_) => None,
            Self::ColdcardMultisig(_) => None,
            Self::Sparrow(meta) => Some(meta),
            Self::Specter(_) => None,
        }
    }

    /// Specter-specific accessor: returns `Some(&metadata)` only for the
    /// `Specter` variant. Consumed by the `--json` envelope emitter in
    /// `cmd::import_wallet::emit_json_envelope` (P2C wiring). Mirrors
    /// `sparrow_source_metadata` above.
    pub(crate) fn specter_source_metadata(&self) -> Option<&specter::SpecterSourceMetadata> {
        match self {
            Self::BitcoinCore(_) => None,
            Self::Bsms(_) => None,
            Self::Coldcard(_) => None,
            Self::ColdcardMultisig(_) => None,
            Self::Sparrow(_) => None,
            Self::Specter(meta) => Some(meta),
        }
    }
}

/// SPEC §8.1 — output of one parser invocation. BSMS Round-2 always emits
/// `vec![ParsedImport]` of length 1 (single descriptor). Bitcoin Core
/// `listdescriptors` (Phase 3) emits length-N for N descriptor entries
/// (subject to `--select-descriptor` filtering).
///
/// `cosigners` is the SPEC §8.1 invariant: every entry has `entropy == None`.
/// Phase 5's seed overlay (`apply_seed_overlay`) re-writes selected entries
/// to `Some(entropy)` after Phase 2's `validate_watch_only_resolved` returns
/// Ok. The invariant holds at construction time; the seed overlay is a
/// distinct downstream step.
#[derive(Debug)]
pub(crate) struct ParsedImport {
    /// Typed descriptor shape (post-`@N`-substitution; canonical-form bytes).
    /// Input to `crate::synthesize::synthesize_descriptor` on the v0.27.0
    /// `import-wallet --json` envelope emit path.
    pub(crate) descriptor: md_codec::Descriptor,
    /// Pre-strip raw descriptor verbatim, including the BIP-380
    /// `#<checksum>` suffix. Disjoint use vs `descriptor` above: this
    /// carries the wire-shape string used in `BundleJson.descriptor`
    /// envelope emission (SPEC §3.2.1). For BSMS the raw is line 2 of
    /// Round-2; for Bitcoin Core it is the `desc` JSON field verbatim.
    pub(crate) original_descriptor: String,
    pub(crate) cosigners: Vec<ResolvedSlot>,
    pub(crate) network: bitcoin::Network,
    pub(crate) threshold: Option<u8>,
    /// Source-provenance of this parsed import. Use `provenance.bsms_audit()` /
    /// `provenance.source_metadata()` accessors to extract the (still-flat) wire
    /// shape on the JSON envelope. See `ImportProvenance` for invariant.
    pub(crate) provenance: ImportProvenance,
}

impl ParsedImport {
    /// Convenience: equivalent to `self.provenance.bsms_audit()`.
    pub(crate) fn bsms_audit(&self) -> Option<&BsmsAuditFields> {
        self.provenance.bsms_audit()
    }

    /// Convenience: equivalent to `self.provenance.source_metadata()`.
    pub(crate) fn source_metadata(&self) -> Option<&CoreSourceMetadata> {
        self.provenance.source_metadata()
    }
}

/// SPEC §5 — per-entry Bitcoin Core metadata. Carries `active`, `internal`,
/// `range` for `--select-descriptor` filtering; `dropped_fields` lists any
/// wallet-state field names (`timestamp`, `next`, `next_index`) that were
/// present in the source entry but dropped from the bundle output (drives the
/// stderr NOTICE per SPEC §2.4).
#[derive(Debug, Clone)]
pub(crate) struct CoreSourceMetadata {
    pub(crate) active: bool,
    pub(crate) internal: bool,
    pub(crate) range: Option<(u64, u64)>,
    pub(crate) dropped_fields: Vec<String>,
    /// SPEC §5.1 + Phase 3 R0 I2 fold: top-level `wallet_name` from the
    /// `listdescriptors` envelope, preserved for Phase 4
    /// `canonicalize_bitcoin_core` semantic round-trip + Phase 5 `--json`
    /// envelope emit. Same value on every entry within a single parse
    /// invocation (envelope is one-per-blob).
    pub(crate) wallet_name: Option<String>,
}

/// SPEC §5.3 — `--select-descriptor` filter applied at the CLI dispatch layer
/// AFTER `WalletFormatParser::parse` returns all entries. Phase 5 wires the
/// clap surface (`<N|active-receive|active-change|all>`); Phase 3 exposes the
/// programmatic helper used by tests + future clap glue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectDescriptor {
    All,
    ByIndex(usize),
    ActiveReceive,
    ActiveChange,
}

/// Apply the `--select-descriptor` filter to a `Vec<ParsedImport>`. See SPEC
/// §5.3 for the exhaustive matrix:
///
/// - `All`: pass-through.
/// - `ByIndex(N)`: emit only `parsed[N]`; error exit 1 if out of range.
/// - `ActiveReceive`: emit entries with `source_metadata.active && !internal`;
///   error exit 1 if no matches.
/// - `ActiveChange`: emit entries with `source_metadata.active && internal`;
///   error exit 1 if no matches.
///
/// Filter values that reference `source_metadata` operate only on entries
/// carrying `Some(CoreSourceMetadata)`. BSMS entries (`source_metadata == None`)
/// silently skip the predicate — they cannot satisfy any active-*-filter. The
/// SPEC §5.3 BSMS-side rule ("non-default `--select-descriptor` emits NOTICE
/// and is treated as `all`") is enforced at the CLI dispatch layer (Phase 5)
/// before this helper is reached; this helper just applies the filter.
pub(crate) fn apply_select_descriptor(
    parsed: Vec<ParsedImport>,
    select: SelectDescriptor,
) -> Result<Vec<ParsedImport>, ToolkitError> {
    match select {
        SelectDescriptor::All => Ok(parsed),
        SelectDescriptor::ByIndex(n) => {
            if n >= parsed.len() {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: bitcoin-core: parse error: --select-descriptor {n} out of range (have {} entries)",
                    parsed.len()
                )));
            }
            let mut v = parsed;
            let picked = v.swap_remove(n);
            Ok(vec![picked])
        }
        SelectDescriptor::ActiveReceive => {
            let kept: Vec<ParsedImport> = parsed
                .into_iter()
                .filter(|p| {
                    p.source_metadata()
                        .map(|m| m.active && !m.internal)
                        .unwrap_or(false)
                })
                .collect();
            if kept.is_empty() {
                return Err(ToolkitError::ImportWalletAmbiguousFormat(
                    "import-wallet: bitcoin-core: --select-descriptor active-receive: no active-receive descriptor found".to_string(),
                ));
            }
            Ok(kept)
        }
        SelectDescriptor::ActiveChange => {
            let kept: Vec<ParsedImport> = parsed
                .into_iter()
                .filter(|p| {
                    p.source_metadata()
                        .map(|m| m.active && m.internal)
                        .unwrap_or(false)
                })
                .collect();
            if kept.is_empty() {
                return Err(ToolkitError::ImportWalletAmbiguousFormat(
                    "import-wallet: bitcoin-core: --select-descriptor active-change: no active-change descriptor found".to_string(),
                ));
            }
            Ok(kept)
        }
    }
}

/// SPEC §8.1 — BSMS Round-2 audit metadata. Preserved for `--json` envelope
/// emission; this field captures inline 2/6-line parser audit context only and
/// is unrelated to v0.27.0's `--bsms-round1 <FILE>` BIP-322 verification path
/// (which emits `bsms_round1_verifications[*].signature_verified` instead).
#[derive(Debug, Clone)]
pub(crate) struct BsmsAuditFields {
    pub(crate) token: String,
    pub(crate) signature: String,
    pub(crate) first_address: String,
    pub(crate) derivation_path: String,
    /// v0.27.1 Phase 5c API-discipline scaffolding: replaces the prior
    /// `signature_verified: bool` field. Closed enum makes the `(bool,
    /// Option<reason>)` representable-invalid state unrepresentable internally
    /// even though wire-shape preserves the legacy `"signature_verified": bool`
    /// JSON field via `BsmsVerification::signature_verified()` derived getter.
    /// Pattern mirrors `Round1VerificationStatus` at cmd/import_wallet.rs:844-850
    /// (v0.27.0 Phase 6.5 I7 precedent).
    pub(crate) verification: BsmsVerification,
}

/// SPEC §8.1 — closed-enum verification status for the BSMS Round-2 audit
/// metadata. v0.26.0/v0.27.0 inline 2/6-line parsers do NOT cryptographically
/// verify the 6-line signature blob (the toolkit's actual BIP-322 verification
/// surface is the v0.27.0 `--bsms-round1 <FILE>` path); inline audit therefore
/// always constructs as `NotAttempted`. The `Verified` / `Failed` variants are
/// reserved for a future cycle that wires the 6-line signature blob through
/// `wallet_import::bsms_verify::verify_round1_signature` — until then they are
/// unreachable from user input but unrepresentable at the type level rather
/// than silently representable-as-`true`.
#[derive(Debug, Clone)]
pub(crate) enum BsmsVerification {
    /// Inline 2/6-line parser did not attempt cryptographic verification of
    /// the 6-line signature blob. v0.26.0/v0.27.0 default.
    NotAttempted,
    /// Reserved: signature was cryptographically verified against the
    /// signer's pubkey. Currently unreachable; constructible only when a
    /// future cycle wires inline-signature verification.
    #[allow(dead_code)]
    Verified,
    /// Reserved: signature was cryptographically verified and FAILED.
    /// Currently unreachable; the `reason` carries the verifier's typed error.
    #[allow(dead_code)]
    Failed {
        #[allow(dead_code)]
        reason: String,
    },
}

impl BsmsVerification {
    /// Wire-shape getter: derives the legacy `"signature_verified": bool` JSON
    /// field for `--json` envelope emission. `Verified` → true; everything
    /// else → false (matches the prior `signature_verified: bool` semantics).
    pub(crate) fn signature_verified(&self) -> bool {
        matches!(self, BsmsVerification::Verified)
    }
}

/// SPEC §8.2 — post-construction watch-only invariant. Every parser's
/// `parse()` calls this on its `ParsedImport` cosigners before returning.
/// Returning `Err(ImportWalletWatchOnlyViolation)` indicates an internal
/// bug in the parser (no user-facing path can produce entropy here; seed
/// overlay runs in `cmd::import_wallet::run` after `parse`).
pub(crate) fn validate_watch_only_resolved(cosigners: &[ResolvedSlot]) -> Result<(), ToolkitError> {
    for (i, c) in cosigners.iter().enumerate() {
        if c.entropy.is_some() {
            return Err(ToolkitError::ImportWalletWatchOnlyViolation(i));
        }
    }
    Ok(())
}

#[cfg(test)]
mod provenance_tests {
    use super::*;

    fn sample_bsms_audit() -> BsmsAuditFields {
        // Field shape grep-verified at src/wallet_import/mod.rs:188-202.
        // No Default impl on BsmsAuditFields or BsmsVerification; construct
        // with minimal valid values directly.
        BsmsAuditFields {
            token: String::new(),
            signature: String::new(),
            first_address: String::new(),
            derivation_path: String::new(),
            verification: BsmsVerification::NotAttempted,
        }
    }

    fn sample_core_metadata() -> CoreSourceMetadata {
        // Field shape grep-verified at src/wallet_import/mod.rs:88-100.
        // No Default impl; minimal valid construction.
        CoreSourceMetadata {
            active: false,
            internal: false,
            range: None,
            dropped_fields: Vec::new(),
            wallet_name: None,
        }
    }

    #[test]
    fn provenance_accessors_return_references_not_owned() {
        let p = ImportProvenance::Bsms(Some(sample_bsms_audit()));
        let _: Option<&BsmsAuditFields> = p.bsms_audit();
        let _: Option<&CoreSourceMetadata> = p.source_metadata();
    }

    #[test]
    fn provenance_bitcoin_core_variant_yields_none_bsms_audit_and_some_source_metadata() {
        let p = ImportProvenance::BitcoinCore(sample_core_metadata());
        assert!(p.bsms_audit().is_none(), "BitcoinCore variant does not expose bsms_audit");
        assert!(p.source_metadata().is_some(), "BitcoinCore variant exposes source_metadata");
    }

    #[test]
    fn provenance_bsms_no_audit_variant_yields_none_bsms_audit() {
        let p = ImportProvenance::Bsms(None);
        assert!(p.bsms_audit().is_none(), "Bsms(None) variant yields no bsms_audit (2-line shape)");
        assert!(p.source_metadata().is_none(), "Bsms(None) variant does not expose source_metadata");
    }

    #[test]
    fn provenance_bsms_variant_yields_some_bsms_audit_and_none_source_metadata() {
        let p = ImportProvenance::Bsms(Some(sample_bsms_audit()));
        assert!(p.bsms_audit().is_some(), "Bsms(Some) variant exposes bsms_audit");
        assert!(p.source_metadata().is_none(), "Bsms variant does not expose source_metadata");
    }

    /// P0B.2 regression guard: behavior on the existing 2 variants is
    /// unchanged by the alphabetical reorder of `BitcoinCore` before `Bsms`.
    /// Exhaustively exercises every (variant × accessor) pair and asserts the
    /// same `Some`/`None` outputs as pre-reorder semantics. Variants
    /// constructed in alphabetical source order to mirror the new enum
    /// declaration ordering (R0 invariant: enum + matches stay alphabetical).
    #[test]
    fn provenance_accessor_matrix_invariant_under_alphabetical_reorder() {
        let core = ImportProvenance::BitcoinCore(sample_core_metadata());
        assert!(core.bsms_audit().is_none(), "BitcoinCore → bsms_audit None");
        assert!(
            core.source_metadata().is_some(),
            "BitcoinCore → source_metadata Some"
        );

        let bsms_some = ImportProvenance::Bsms(Some(sample_bsms_audit()));
        assert!(bsms_some.bsms_audit().is_some(), "Bsms(Some) → bsms_audit Some");
        assert!(
            bsms_some.source_metadata().is_none(),
            "Bsms(Some) → source_metadata None"
        );

        let bsms_none = ImportProvenance::Bsms(None);
        assert!(bsms_none.bsms_audit().is_none(), "Bsms(None) → bsms_audit None");
        assert!(
            bsms_none.source_metadata().is_none(),
            "Bsms(None) → source_metadata None"
        );
    }
}
