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
    /// SPEC §5 — Bitcoin Core source-specific metadata (`active`, `internal`,
    /// `range`, dropped wallet-state field names). `None` for BSMS parses.
    /// Drives `--select-descriptor active-receive` / `active-change` filtering
    /// at the CLI dispatch layer (`apply_select_descriptor` below).
    pub(crate) source_metadata: Option<CoreSourceMetadata>,
}

/// SPEC §5 — per-entry Bitcoin Core metadata. Carries `active`, `internal`,
/// `range` for `--select-descriptor` filtering; `dropped_fields` lists any
/// wallet-state field names (`timestamp`, `next`, `next_index`) that were
/// present in the source entry but dropped from the bundle output (drives the
/// stderr NOTICE per SPEC §2.4).
#[derive(Debug, Clone)]
#[allow(dead_code)] // Phase 5 consumes `range` for round-trip emit.
pub(crate) struct CoreSourceMetadata {
    pub(crate) active: bool,
    pub(crate) internal: bool,
    pub(crate) range: Option<(u64, u64)>,
    pub(crate) dropped_fields: Vec<String>,
}

/// SPEC §5.3 — `--select-descriptor` filter applied at the CLI dispatch layer
/// AFTER `WalletFormatParser::parse` returns all entries. Phase 5 wires the
/// clap surface (`<N|active-receive|active-change|all>`); Phase 3 exposes the
/// programmatic helper used by tests + future clap glue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Phase 5 wires the full clap parser.
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
                    p.source_metadata
                        .as_ref()
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
                    p.source_metadata
                        .as_ref()
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
