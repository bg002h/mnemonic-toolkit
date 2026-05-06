//! `mnemonic` — engraving-bundle CLI for the m-format star.

mod bundle_unified;
mod cmd;
mod derive;
mod error;
mod format;
mod friendly;
mod language;
mod network;
mod parse;
mod parse_descriptor;
mod slot_input;
mod synthesize;
mod template;

use clap::{Parser, Subcommand};
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
