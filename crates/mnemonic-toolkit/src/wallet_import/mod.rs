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

pub(crate) mod bsms;
pub(crate) mod pipeline;

/// SPEC §8.1 — every per-format parser implements this trait. Associated-
/// function shape (no `&self`); dispatch is `match format { ... }`-style at
/// the call site (not `dyn WalletFormatParser`). The trait is not
/// object-safe by design.
pub(crate) trait WalletFormatParser {
    /// Heuristic detection: return `true` if `blob` looks like this format.
    /// Used by Phase 5's `sniff` dispatcher to auto-detect when `--format`
    /// is not supplied.
    #[allow(dead_code)] // Phase 5 wires the sniff dispatcher.
    fn sniff(blob: &[u8]) -> bool;

    /// Parse the blob into one or more `ParsedImport` bundles. WARNINGs go to
    /// `stderr` directly; the returned `Vec` is the canonical bundle list.
    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError>;
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
#[allow(dead_code)] // Phase 2 constructs; Phase 5 consumes descriptor + bsms_audit.
pub(crate) struct ParsedImport {
    pub(crate) descriptor: md_codec::Descriptor,
    pub(crate) cosigners: Vec<ResolvedSlot>,
    pub(crate) network: bitcoin::Network,
    pub(crate) threshold: Option<u8>,
    pub(crate) bsms_audit: Option<BsmsAuditFields>,
}

/// SPEC §8.1 — BSMS Round-2 audit metadata. Preserved for `--json` envelope
/// emission; `signature_verified` is always `false` in v0.26.0 (FOLLOWUP
/// `bsms-verify-signatures`).
#[derive(Debug, Clone)]
#[allow(dead_code)] // Phase 5 consumes for --json envelope emission.
pub(crate) struct BsmsAuditFields {
    pub(crate) token: String,
    pub(crate) signature: String,
    pub(crate) first_address: String,
    pub(crate) derivation_path: String,
    pub(crate) signature_verified: bool,
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
