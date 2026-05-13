//! `mnemonic` — engraving-bundle CLI for the m-format star.

mod bip85;
mod bundle_unified;
mod cmd;
mod derive;
mod derive_slot;
mod electrum;
mod error;
mod format;
mod friendly;
mod language;
mod network;
mod parse;
mod parse_descriptor;
mod secret_advisory;
mod slip0132;
mod slot_input;
mod synthesize;
mod template;
mod wallet_export;
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
    /// derive deterministic child entropy / keys from a master xprv (BIP-85)
    DeriveChild(cmd::derive_child::DeriveChildArgs),
    /// emit SPEC §7 GUI-overlay flag-surface schema JSON (companion to `mnemonic-gui` v0.2)
    GuiSchema(cmd::gui_schema::GuiSchemaArgs),
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
        Command::VerifyBundle(args) => cmd::verify_bundle::run(args, stdin, stdout, stderr),
        Command::Convert(args) => cmd::convert::run(args, stdin, stdout, stderr),
        Command::ExportWallet(args) => {
            cmd::export_wallet::run(args, stdout, stderr).map(|_| 0)
        }
        Command::DeriveChild(args) => {
            cmd::derive_child::run(args, stdin, stdout, stderr).map(|_| 0)
        }
        Command::GuiSchema(args) => {
            // Re-derive the clap `Command` tree via CommandFactory so the
            // schema reflects the canonical clap-derive surface (single
            // source of truth — no parallel hand-maintained schema).
            let root = Cli::command();
            cmd::gui_schema::run(args, &root, stdout).map(|_| 0)
        }
    };

    match result {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            // Emit error per SPEC §6.5 + §5.5.
            let _ = writeln!(io::stderr(), "{}", e);
            ExitCode::from(e.exit_code())
        }
    }
}
