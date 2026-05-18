//! `mnemonic xpub-search` umbrella subcommand — v0.26.0.
//!
//! Realizes `design/PLAN_v0_26_0_xpub_search.md` C1 scaffolding + phase P1
//! `path-of-xpub`. C2/C3/C4 commits extend `XpubSearchCommand` with new
//! variants and `XpubSearchJson` with new tagged-enum arms; this file's
//! shape is designed for parallel-disjoint commits (small mechanical merges
//! when other modes land).
//!
//! ## SPEC anchors
//!
//! - `§3` P1 path-of-xpub
//! - `§9.5` per-mode JSON envelope (`schema_version: "1"`, `tag = "mode"`)
//! - `§9.4` `XpubSearchNoMatch` exit-4 routing
//!
//! ## JSON envelope
//!
//! ```text
//! {"schema_version": "1", "mode": "path-of-xpub", "result": "match", ...}
//! ```
//!
//! The top-level `schema_version` field is always emitted; the per-mode body
//! is flattened-with-tag (`#[serde(flatten)]` + `tag = "mode"`). `tag = "mode"`
//! deviates from the project's `tag = "kind"` convention (`InspectJson`,
//! `RepairJson`) — see CHANGELOG `### Added` for the rationale.

use crate::error::ToolkitError;
use clap::{Args, Subcommand};
use serde::Serialize;
use std::io::{Read, Write};

pub mod account_of_descriptor;
pub mod account_search;
pub mod address_of_xpub;
pub mod address_search;
pub mod candidate_paths;
pub mod descriptor_intake;
pub mod passphrase_of_xpub;
pub mod path_of_xpub;
pub mod path_search;
pub mod seed_intake;
pub mod target_intake;

// Re-export the per-mode result struct so the unit-cell-in-tests reaches it
// via `mnemonic_toolkit::cmd::xpub_search::PathOfXpubResult`.
pub use account_of_descriptor::AccountOfDescriptorResult;
pub use address_of_xpub::AddressOfXpubResult;
pub use passphrase_of_xpub::PassphraseOfXpubResult;
pub use path_of_xpub::PathOfXpubResult;

/// Umbrella `xpub-search` args. Defers to `XpubSearchCommand` for the
/// per-mode dispatch.
#[derive(Args, Debug)]
pub struct XpubSearchArgs {
    #[command(subcommand)]
    pub command: XpubSearchCommand,
}

#[derive(Subcommand, Debug)]
pub enum XpubSearchCommand {
    /// Given a seed + target xpub, find the BIP-32 path under the seed that
    /// produces the xpub.
    PathOfXpub(path_of_xpub::PathOfXpubArgs),
    /// Given a seed + descriptor, identify which cosigner role(s) and
    /// account(s) the seed plays in the descriptor.
    AccountOfDescriptor(account_of_descriptor::AccountOfDescriptorArgs),
    /// Given a single-sig xpub + one or more target addresses, find the
    /// chain/index under the xpub that produces each address.
    AddressOfXpub(address_of_xpub::AddressOfXpubArgs),
    /// Given a seed + a specific passphrase + target xpub, verify whether
    /// the passphrase produces the xpub at a standard path. P1 with a
    /// mandatory passphrase group.
    PassphraseOfXpub(passphrase_of_xpub::PassphraseOfXpubArgs),
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &XpubSearchArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    match &args.command {
        XpubSearchCommand::PathOfXpub(a) => {
            path_of_xpub::run_path_of_xpub(a, stdin, stdout, stderr, no_auto_repair)
        }
        XpubSearchCommand::AccountOfDescriptor(a) => {
            account_of_descriptor::run_account_of_descriptor(
                a,
                stdin,
                stdout,
                stderr,
                no_auto_repair,
            )
        }
        XpubSearchCommand::AddressOfXpub(a) => {
            address_of_xpub::run_address_of_xpub(a, stdin, stdout, stderr, no_auto_repair)
        }
        XpubSearchCommand::PassphraseOfXpub(a) => {
            passphrase_of_xpub::run_passphrase_of_xpub(a, stdin, stdout, stderr, no_auto_repair)
        }
    }
}

// ---------------------------------------------------------------------------
// JSON envelope (§9.5)
// ---------------------------------------------------------------------------

/// Top-level JSON envelope. Always emits `schema_version: "1"` at the top
/// level; the per-mode body is `#[serde(flatten)]`'d so the resulting JSON
/// has shape `{"schema_version":"1","mode":"<mode>","result":"...",...}`.
#[derive(Debug, Serialize)]
pub struct XpubSearchEnvelope {
    pub schema_version: &'static str,
    #[serde(flatten)]
    pub body: XpubSearchJson,
}

/// Per-mode tagged-enum body. `tag = "mode"` deviates from the project's
/// `tag = "kind"` (per CHANGELOG `### Added` rationale: "mode" is the natural
/// domain term for `xpub-search`'s four sub-modes; "kind" would conflict
/// with `RepairJson`'s `kind: "ms1"|"mk1"|"md1"` per-card semantic).
#[derive(Debug, Serialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum XpubSearchJson {
    PathOfXpub(PathOfXpubResult),
    AccountOfDescriptor(AccountOfDescriptorResult),
    AddressOfXpub(AddressOfXpubResult),
    PassphraseOfXpub(PassphraseOfXpubResult),
}

#[cfg(test)]
mod tests {
    //! Unit cell: `XpubSearchEnvelope` serde round-trip. Pins the
    //! `#[serde(flatten)]` + inner `tag = "mode"` shape (known-tricky serde
    //! pattern) against accidental breakage. Per plan §10 C1 cell list.

    use super::*;

    #[test]
    fn xpub_search_envelope_serde_round_trip_match() {
        let envelope = XpubSearchEnvelope {
            schema_version: "1",
            body: XpubSearchJson::PathOfXpub(PathOfXpubResult {
                result: "match",
                path: Some("m/84'/0'/0'".to_string()),
                template: Some("bip84".to_string()),
                account: Some(0),
                target_xpub_canonical: "xpub6...".to_string(),
                target_xpub_variant: Some("zpub"),
                searched_count: 140,
            }),
        };
        let v = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(v["schema_version"], "1");
        assert_eq!(v["mode"], "path-of-xpub");
        assert_eq!(v["result"], "match");
        assert_eq!(v["path"], "m/84'/0'/0'");
        assert_eq!(v["template"], "bip84");
        assert_eq!(v["account"], 0);
        assert_eq!(v["target_xpub_variant"], "zpub");
        assert_eq!(v["searched_count"], 140);
        assert_eq!(v["target_xpub_canonical"], "xpub6...");
    }

    #[test]
    fn xpub_search_envelope_no_variant_serializes_as_null() {
        let env_no_variant = XpubSearchEnvelope {
            schema_version: "1",
            body: XpubSearchJson::PathOfXpub(PathOfXpubResult {
                result: "no_match",
                path: None,
                template: None,
                account: None,
                target_xpub_canonical: "xpub6...".to_string(),
                target_xpub_variant: None,
                searched_count: 7,
            }),
        };
        let v = serde_json::to_value(&env_no_variant).expect("serialize");
        // Plan §3.4 lock: `target_xpub_variant` is always emitted; None
        // serializes as `null` (no `skip_serializing_if`).
        assert!(
            v["target_xpub_variant"].is_null(),
            "None variant must serialize as null; got {:?}",
            v["target_xpub_variant"]
        );
        assert!(v["path"].is_null());
        assert!(v["account"].is_null());
        assert!(v["template"].is_null());
    }
}
