//! `mnemonic` — engraving-bundle CLI for the m-format star.

mod address_render;
mod bip85;
mod bundle_unified;
mod cmd;
mod cost;
mod decode_address;
mod derive;
mod derive_address;
mod derive_slot;
mod descriptor_builder;
mod display_grouping;
mod electrum;
mod env_sentinel;
mod error;
mod format;
mod friendly;
mod indel;
mod language;
mod network;
mod nostr;
mod parse;
mod parse_descriptor;
mod repair;
mod secret_advisory;
mod secret_string;
mod silent_payment;
mod slip0132;
mod slot_input;
mod slot_ms1;
mod synthesize;
mod taproot_override_classify;
mod template;
mod timelock_advisory;
mod unrestorable_advisory;
mod verify_message;
mod wallet_export;
mod wallet_import;
mod word_card_adapter;
mod wordlists;

// T2-a (#6) — never-wrong-payload property harness for the repair engine.
// In-crate `#[cfg(test)]` module (repair/indel are binary-private): direct
// private access to `repair_card` / `recover_indel` incl. the mock-oracle
// ambiguity path. TEST-only, NO-BUMP (compiles only under `cargo test`).
#[cfg(test)]
mod prop_repair_never_wrong;

use clap::{CommandFactory, Parser, Subcommand};
use error::ToolkitError;
use std::io::{self, Write};
use std::process::ExitCode;

/// Top-level `after_help` footer (renders on both `mnemonic -h` and
/// `mnemonic --help`). Points users who hold the entropy but have lost the
/// BIP-39 passphrase at btcrecover: `mnemonic` cannot brute-force a
/// passphrase, because a BIP-39 passphrase has no internal verifier —
/// every candidate yields a valid-looking wallet, so correctness is only
/// definable against a known address/xpub/master-fingerprint, an
/// external-derivation-oracle attack outside this tool's scope. Date-stamped
/// per the 2026-05-25 recon decision; guarded by
/// `cli_help_fixtures::top_level_help_points_to_btcrecover_for_passphrase_recovery`
/// and mirrored in `docs/manual/src/40-cli-reference/41-mnemonic.md`.
const PASSPHRASE_RECOVERY_HELP: &str = "\
RECOVERING A FORGOTTEN BIP-39 PASSPHRASE:
  If you have your seed words (entropy) but not the BIP-39 passphrase
  (the optional \"25th word\"): if you have a LIST of likely passphrases,
  `mnemonic xpub-search passphrase-of-xpub --passphrase-candidates-file
  <file> --target-xpub <a known xpub>` tests each candidate against a
  value you already know. To GENERATE or mutate a keyspace (wordlists,
  masks, typo models), `mnemonic` does not — an external open-source tool
  does: btcrecover searches passphrase candidates and confirms each by
  deriving an address / xpub / master-fingerprint at common default paths
  and matching a value you already know.
    btcrecover (maintained):  https://github.com/3rdIteration/btcrecover
    original:                 https://github.com/gurnec/btcrecover
  Pointer current as of 2026-05-25. Run untrusted recovery tools
  offline, on an air-gapped machine.";

#[derive(Parser, Debug)]
#[command(
    name = "mnemonic",
    about = "engraving-bundle CLI for the m-format star (ms1 + mk1 + md1)",
    version,
    after_help = PASSPHRASE_RECOVERY_HELP
)]
struct Cli {
    /// v0.22.0 — skip auto-fire repair on decode failures; preserve
    /// pre-v0.22 exit policy. Global flag. Honored by `convert`,
    /// `inspect`, and (v0.22.1+) `verify-bundle`. For `verify-bundle`,
    /// auto-fire is additionally gated on `std::io::stdout().is_terminal()`
    /// to preserve the legacy VerifyCheck-row behavior when output is
    /// piped or captured (per v0.22.1 D18 — TTY-conditional default).
    /// Standalone `repair` ignores this flag (the whole point of that
    /// subcommand IS repair). Under `--json` calling contexts the
    /// auto-fire emits a structured JSON envelope on stdout (per v0.22.1
    /// D20) instead of text-form.
    #[arg(long, global = true)]
    no_auto_repair: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// emit a 3-card engraving bundle from a phrase or xpub
    Bundle(cmd::bundle::BundleArgs),
    /// round-trip-check an engraved bundle
    VerifyBundle(cmd::verify_bundle::VerifyBundleArgs),
    /// convert between seed/key formats (BIP-39 / BIP-32 / WIF / ms1 / mk1)
    Convert(cmd::convert::ConvertArgs),
    /// list a wallet's receive/change addresses (batch, read-only)
    Addresses(cmd::addresses::AddressesArgs),
    /// decode a Bitcoin address → network(s) / script type / witness version / scriptPubKey
    DecodeAddress(cmd::decode_address::DecodeAddressArgs),
    /// emit watch-only wallet artifacts (Bitcoin Core importdescriptors, BIP-388 wallet_policy)
    ExportWallet(cmd::export_wallet::ExportWalletArgs),
    /// import a third-party wallet blob into an m-format bundle (v0.26.0 Phase 2: BSMS Round-2 only)
    ImportWallet(cmd::import_wallet::ImportWalletArgs),
    /// derive deterministic child entropy / keys from a master xprv (BIP-85)
    DeriveChild(cmd::derive_child::DeriveChildArgs),
    /// decrypt an Electrum field-encrypted secret (seed phrase / xprv) with a password
    ElectrumDecrypt(cmd::electrum_decrypt::ElectrumDecryptArgs),
    /// emit the set of BIP-39 last words that yield a valid checksum for an N-1 partial phrase
    FinalWord(cmd::final_word::FinalWordArgs),
    /// split a BIP-39 phrase into N XOR shares OR combine N shares back into a phrase
    SeedXor(cmd::seed_xor::SeedXorArgs),
    /// encode/decode SeedQR (BIP-39 mnemonic ↔ numeric digit-string QR payload)
    Seedqr(cmd::seedqr::SeedqrArgs),
    /// Wrap an existing nostr key (npub/nsec) as Bitcoin addresses/descriptors/WIF.
    Nostr(cmd::nostr::NostrArgs),
    /// Derive a BIP-352 silent-payment receiver address (base + labeled) from a seed.
    SilentPayment(cmd::silent_payment::SilentPaymentArgs),
    /// split a master secret into SLIP-39 K-of-N shares OR combine shares back (Trezor-compatible)
    Slip39(cmd::slip39::Slip39Args),
    /// split a secret into BIP-93 codex32 K-of-N (ms1) shares OR combine shares back
    MsShares(cmd::ms_shares::MsSharesArgs),
    /// emit roff man pages for the whole CLI tree into a directory (clap-faithful)
    GenMan(cmd::gen_man::GenManArgs),
    /// emit SPEC §7 GUI-overlay flag-surface schema JSON (companion to `mnemonic-gui` v0.2)
    GuiSchema(cmd::gui_schema::GuiSchemaArgs),
    /// BCH error-correct a corrupted m-format card (ms1 / mk1 / md1)
    Repair(cmd::repair::RepairArgs),
    /// describe the contents of an m-format card (ms1 / mk1 / md1)
    Inspect(cmd::inspect::InspectArgs),
    /// compare wsh-vs-tr per-spending-condition cost for a miniscript or descriptor
    CompareCost(cmd::compare_cost::CompareCostArgs),
    /// search for a target (xpub, descriptor, address, or passphrase) under a seed or xpub
    XpubSearch(cmd::xpub_search::XpubSearchArgs),
    /// verify a Bitcoin message signature (legacy P2PKH signmessage + BIP-322 segwit/taproot)
    VerifyMessage(cmd::verify_message::VerifyMessageArgs),
    /// emit a watch-only restore document (single-sig) from a seed + optional passphrase
    Restore(cmd::restore::RestoreArgs),
    /// build a validated wsh(...) descriptor + BIP-388 policy from a JSON policy-tree spec
    BuildDescriptor(cmd::build_descriptor::BuildDescriptorArgs),
    /// encode an mk1/md1 card as an engravable BIP-39 Word Card (+ optional RAID), or --decode one back
    WordCard(cmd::word_card::WordCardArgs),
}

fn main() -> ExitCode {
    // v0.34.7 argv-hardening: deny other-UID /proc/$PID/cmdline reads + core dumps.
    mnemonic_toolkit::process_hardening::set_non_dumpable();
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            // Override clap's default exit code 2 → 64 to keep format-violations distinct.
            e.print().ok();
            return ExitCode::from(if e.exit_code() == 0 { 0 } else { 64 });
        }
    };

    let stdin = &mut io::stdin();
    let stdout = &mut io::stdout();
    let stderr = &mut io::stderr();

    let result: Result<u8, ToolkitError> = match &cli.command {
        Command::Bundle(args) => cmd::bundle::run(args, stdin, stdout, stderr).map(|_| 0),
        Command::VerifyBundle(args) => {
            cmd::verify_bundle::run(args, stdin, stdout, stderr, cli.no_auto_repair)
        }
        Command::Convert(args) => {
            cmd::convert::run(args, stdin, stdout, stderr, cli.no_auto_repair)
        }
        Command::Addresses(args) => cmd::addresses::run(args, stdin, stdout, stderr),
        Command::DecodeAddress(args) => cmd::decode_address::run(args, stdin, stdout, stderr),
        Command::ExportWallet(args) => cmd::export_wallet::run(args, stdout, stderr).map(|_| 0),
        Command::ImportWallet(args) => {
            cmd::import_wallet::run(args, stdin, stdout, stderr, cli.no_auto_repair)
        }
        Command::DeriveChild(args) => {
            cmd::derive_child::run(args, stdin, stdout, stderr).map(|_| 0)
        }
        Command::ElectrumDecrypt(args) => cmd::electrum_decrypt::run(args, stdin, stdout, stderr),
        Command::FinalWord(args) => cmd::final_word::run(args, stdin, stdout, stderr),
        Command::SeedXor(args) => cmd::seed_xor::run(args, stdin, stdout, stderr),
        Command::Seedqr(args) => cmd::seedqr::run(args, stdin, stdout, stderr),
        Command::Nostr(args) => cmd::nostr::run(args, stdin, stdout, stderr),
        Command::SilentPayment(args) => cmd::silent_payment::run(args, stdin, stdout, stderr),
        Command::Slip39(args) => cmd::slip39::run(args, stdin, stdout, stderr),
        Command::MsShares(args) => cmd::ms_shares::run(args, stdin, stdout, stderr),
        Command::GenMan(args) => {
            // Pass the UNBUILT `Cli::command()` tree — NO pre-`.build()` (C-1).
            // generate_to builds internally after disable_help_subcommand(true),
            // so no `*-help*.1` shadow pages are emitted.
            let root = Cli::command();
            cmd::gen_man::run(args, root, stdout).map(|_| 0)
        }
        Command::GuiSchema(args) => {
            // Re-derive the clap `Command` tree via CommandFactory so the
            // schema reflects the canonical clap-derive surface (single
            // source of truth — no parallel hand-maintained schema).
            let root = Cli::command();
            cmd::gui_schema::run(args, &root, stdout).map(|_| 0)
        }
        Command::Repair(args) => cmd::repair::run(args, stdin, stdout, stderr),
        Command::Inspect(args) => {
            cmd::inspect::run(args, stdin, stdout, stderr, cli.no_auto_repair)
        }
        Command::CompareCost(args) => {
            cmd::compare_cost::run(args, stdin, stdout, stderr).map(|_| 0)
        }
        Command::VerifyMessage(args) => cmd::verify_message::run(args, stdin, stdout, stderr),
        Command::XpubSearch(args) => {
            cmd::xpub_search::run(args, stdin, stdout, stderr, cli.no_auto_repair)
        }
        Command::Restore(args) => {
            cmd::restore::run(args, stdin, stdout, stderr, cli.no_auto_repair)
        }
        Command::BuildDescriptor(args) => cmd::build_descriptor::run(args, stdin, stdout, stderr),
        Command::WordCard(args) => cmd::word_card::run(args, stdin, stdout, stderr),
    };

    let exit = match result {
        Ok(code) => ExitCode::from(code),
        // R2 I1: short-circuit fires a clean repair report on stderr inside
        // the helper; do NOT also emit the ToolkitError Display impl (which
        // would tack on "error: " noise). The exit code is carried in the
        // variant itself.
        Err(error::ToolkitError::RepairShortCircuit { exit_code }) => ExitCode::from(exit_code),
        Err(e) => {
            // Emit error per SPEC §6.5 + §5.5.
            let _ = writeln!(io::stderr(), "{}", e);
            ExitCode::from(e.exit_code())
        }
    };

    // Cycle B SPEC §2 row 3 + §6 G2.5 — emit a 2-line stderr summary iff
    // any pin_pages_for call soft-failed during this invocation. No-op
    // when failure_count == 0. Runs on both Ok and Err paths.
    mnemonic_toolkit::mlock::report_at_exit();

    exit
}
