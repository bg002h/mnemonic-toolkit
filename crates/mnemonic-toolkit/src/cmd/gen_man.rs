//! `mnemonic gen-man` subcommand — emit roff man pages from the clap tree.
//!
//! Calls `clap_mangen::generate_to(<root Command>, out_dir)` with **NO
//! pre-`.build()`** (SPEC §2 / C-1). The pages are clap-generated, hence
//! binary-faithful by construction — there is no content-fidelity gate because
//! the man page cannot drift from the binary's actual flag surface.
//!
//! ## Why the bare naive call (no pre-`.build()`)
//!
//! Under clap_mangen 0.3.0 + clap 4.6.1, a pre-`.build()` on the root
//! `Command` POISONS the output with a `help` pseudo-subcommand SHADOW TREE
//! (~18 spurious `mnemonic-help.1` / `mnemonic-*-help-*.1` pages). Root cause:
//! `generate_to` internally does `disable_help_subcommand(true)` THEN
//! `cmd.build()`; an external `root.build()` runs FIRST and materializes the
//! `help` subcommands as real tree entries before the internal disable can
//! suppress them. The naive call (no pre-build) is clean: exactly the
//! per-(sub)command set, zero help pages. The `cli_gen_man.rs` NEGATIVE canary
//! is the regression tripwire for an accidental future pre-build.
//!
//! ## Global-flag rendering (C-2)
//!
//! The toolkit's sole `global = true` flag, `--no-auto-repair`, renders in the
//! `.SH OPTIONS` / `.TP` section of EVERY generated page under clap_mangen
//! 0.3.0 — its `Man` renderer surfaces the root command's global args on each
//! (sub)command page (verified empirically: all 38 pages contain the
//! roff-escaped `\-\-no\-auto\-repair`). The flag is also discoverable via
//! `mnemonic --help`. (The SPEC's original C-2 claim that it renders in zero
//! pages was empirically false; corrected here and pinned by a positive
//! assertion in `cli_gen_man.rs`.)

use crate::error::ToolkitError;
use clap::{Args, Command};
use std::io::Write;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct GenManArgs {
    /// Directory to write the `*.1` man pages into (created if absent).
    /// One page per (nested) subcommand, hyphen-joined parent→child
    /// (e.g. `mnemonic-seed-xor-split.1`).
    #[arg(long, value_name = "DIR")]
    pub out: PathBuf,
}

/// Generate roff man pages for the whole `root` command tree into `args.out`.
///
/// `root` MUST be the UNBUILT `Cli::command()` (no pre-`.build()`, C-1).
pub fn run(args: &GenManArgs, root: Command, stdout: &mut impl Write) -> Result<(), ToolkitError> {
    std::fs::create_dir_all(&args.out).map_err(ToolkitError::Io)?;
    // Bare naive call — NO pre-`.build()` (C-1). `generate_to` builds the tree
    // internally after `disable_help_subcommand(true)`, so no `*-help*.1`
    // shadow pages are emitted.
    clap_mangen::generate_to(root, &args.out).map_err(ToolkitError::Io)?;
    writeln!(stdout, "man pages written to {}", args.out.display()).map_err(ToolkitError::Io)?;
    Ok(())
}
