//! `mnemonic` — engraving-bundle CLI for the m-format star.

mod bip85;
mod bundle_unified;
mod cmd;
mod derive;
mod derive_slot;
mod electrum;
mod env_sentinel;
mod error;
mod format;
mod friendly;
mod language;
mod network;
mod parse;
mod parse_descriptor;
mod repair;
mod secret_advisory;
mod slip0132;
mod slot_input;
mod synthesize;
mod template;
mod wallet_export;
mod wallet_import;
mod wordlists;

use clap::{CommandFactory, Parser, Subcommand};
use error::ToolkitError;
use std::io::{self, Write};
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "mnemonic",
    about = "engraving-bundle CLI for the m-format star (ms1 + mk1 + md1)",
    version
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
    /// emit watch-only wallet artifacts (Bitcoin Core importdescriptors, BIP-388 wallet_policy)
    ExportWallet(cmd::export_wallet::ExportWalletArgs),
    /// import a third-party wallet blob into an m-format bundle (v0.26.0 Phase 2: BSMS Round-2 only)
    ImportWallet(cmd::import_wallet::ImportWalletArgs),
    /// derive deterministic child entropy / keys from a master xprv (BIP-85)
    DeriveChild(cmd::derive_child::DeriveChildArgs),
    /// emit the set of BIP-39 last words that yield a valid checksum for an N-1 partial phrase
    FinalWord(cmd::final_word::FinalWordArgs),
    /// split a BIP-39 phrase into N XOR shares OR combine N shares back into a phrase
    SeedXor(cmd::seed_xor::SeedXorArgs),
    /// split a master secret into SLIP-39 K-of-N shares OR combine shares back (Trezor-compatible)
    Slip39(cmd::slip39::Slip39Args),
    /// emit SPEC §7 GUI-overlay flag-surface schema JSON (companion to `mnemonic-gui` v0.2)
    GuiSchema(cmd::gui_schema::GuiSchemaArgs),
    /// BCH error-correct a corrupted m-format card (ms1 / mk1 / md1)
    Repair(cmd::repair::RepairArgs),
    /// describe the contents of an m-format card (ms1 / mk1 / md1)
    Inspect(cmd::inspect::InspectArgs),
    /// search for a target (xpub, descriptor, address, or passphrase) under a seed or xpub
    XpubSearch(cmd::xpub_search::XpubSearchArgs),
}

fn main() -> ExitCode {
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
        Command::VerifyBundle(args) => cmd::verify_bundle::run(args, stdin, stdout, stderr, cli.no_auto_repair),
        Command::Convert(args) => cmd::convert::run(args, stdin, stdout, stderr, cli.no_auto_repair),
        Command::ExportWallet(args) => {
            cmd::export_wallet::run(args, stdout, stderr).map(|_| 0)
        }
        Command::ImportWallet(args) => {
            cmd::import_wallet::run(args, stdin, stdout, stderr, cli.no_auto_repair)
        }
        Command::DeriveChild(args) => {
            cmd::derive_child::run(args, stdin, stdout, stderr).map(|_| 0)
        }
        Command::FinalWord(args) => cmd::final_word::run(args, stdin, stdout, stderr),
        Command::SeedXor(args) => cmd::seed_xor::run(args, stdin, stdout, stderr),
        Command::Slip39(args) => cmd::slip39::run(args, stdin, stdout, stderr),
        Command::GuiSchema(args) => {
            // Re-derive the clap `Command` tree via CommandFactory so the
            // schema reflects the canonical clap-derive surface (single
            // source of truth — no parallel hand-maintained schema).
            let root = Cli::command();
            cmd::gui_schema::run(args, &root, stdout).map(|_| 0)
        }
        Command::Repair(args) => cmd::repair::run(args, stdin, stdout, stderr),
        Command::Inspect(args) => cmd::inspect::run(args, stdin, stdout, stderr, cli.no_auto_repair),
        Command::XpubSearch(args) => {
            cmd::xpub_search::run(args, stdin, stdout, stderr, cli.no_auto_repair)
        }
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
