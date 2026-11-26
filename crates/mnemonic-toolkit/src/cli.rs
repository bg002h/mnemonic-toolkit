//! `mnemonic` — engraving-bundle CLI for the m-format star.
//!
//! The CLI entry point (`run`) lives in the library, not in `main.rs`, so the
//! integration tests link the library instead of recompiling the binary.
//! `main.rs` is a one-line shim over [`run`].

use crate::error::ToolkitError;
use crate::{argv_guard, cmd, error, out_sink};
use clap::{CommandFactory, Parser, Subcommand};
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
///
/// The root `--help` footer: the passphrase pointer, then the
/// constellation exit-code legend (`SPEC_constellation_exit_codes.md` R11,
/// word for word; the text lives in `exit_code::HELP_LEGEND`). A `concat!` of
/// literals because clap's `after_help` takes a `&'static str` here.
const ROOT_AFTER_HELP: &str = concat!(
    "RECOVERING A FORGOTTEN BIP-39 PASSPHRASE:
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
  offline, on an air-gapped machine.

",
    "Exit status: 0 = done, every check passed. 64 = bad command line, nothing
ran. 101 = internal error. 130 = stopped by the operator. Other codes from
128 up = killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said
no; 2 an input is invalid; 4 verify the result before trusting it; 8 refused
(network mismatch, secret on the command line, unsafe output); 16 a
correction was applied; 32 a file, device or helper failed."
);

/// The arguments whose VALUE is an input the verb works on, not a setting
/// (`SPEC_constellation_exit_codes.md` decision 2): when clap's value parser
/// rejects one of these, the run exits 2 (an input is invalid), not 64. "Who
/// checks the value does not change its class": a fingerprint typo is 2 in
/// every tool, and so is a malformed `--from`/`--slot` under
/// `--allow-argv-secret` (without it the argv guard answers first, 8).
const INPUT_VALUE_ARGS: &[&str] = &[
    "--address",
    "--cosigner",
    "--descriptor",
    "--descriptor-from",
    "--expect-fingerprint",
    "--expect-wallet-id",
    "--expect-xpub",
    "--fingerprint",
    "--from",
    "--master-fingerprint",
    "--md1",
    "--mk1",
    "--ms1",
    "--origin",
    "--path",
    "--search-address",
    "--share",
    "--slot",
    "--target-address",
    "--target-xpub",
    "--template",
    "--xpub",
];

/// The exit code for a clap parse error: 0 for `--help`/`--version`, 2 for a
/// rejected value of an input argument ([`INPUT_VALUE_ARGS`]), 64 otherwise.
fn clap_error_exit_code(e: &clap::Error) -> u8 {
    use clap::error::{ContextKind, ContextValue, ErrorKind};
    if e.exit_code() == 0 {
        return crate::exit_code::OK;
    }
    if matches!(
        e.kind(),
        ErrorKind::InvalidValue | ErrorKind::ValueValidation
    ) {
        if let Some(ContextValue::String(arg)) = e.get(ContextKind::InvalidArg) {
            let name = arg.split([' ', '=']).next().unwrap_or("");
            if INPUT_VALUE_ARGS.contains(&name) {
                return crate::exit_code::INVALID;
            }
        }
    }
    crate::exit_code::USAGE
}

#[derive(Parser, Debug)]
#[command(
    name = "mnemonic",
    about = "engraving-bundle CLI for the m-format star (ms1 + mk1 + md1)",
    version,
    after_help = ROOT_AFTER_HELP
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

    // NOTE the single-paragraph doc comment: clap-derive turns anything after a
    // blank line into `long_help`, and the mere PRESENCE of a long_help flips
    // `--help` from the compact layout to the verbose one for EVERY root option.
    // Measured: it rewrote 122 lines of `.examples-build/Examples.md`, none of
    // them about this flag. The rationale lives in `argv_guard`'s module docs.
    /// SPEC_constellation_cli_uniformity 6d — proceed even though secret material is on argv. Use it only where argv is safe (a single-user air-gapped box, an amnesic Tails session). The DECISION is not made here: `argv_guard::inspect` reads this flag out of raw `std::env::args()` before `Cli::try_parse()` runs; this declaration exists so clap ACCEPTS it and so `--help` and `gui-schema` show it. Greppable, so a reviewer can find every place a script opted in
    #[arg(long, global = true)]
    allow_argv_secret: bool,

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

/// The `mnemonic` binary's `main`, called by the one-line `src/main.rs`.
#[doc(hidden)]
pub fn run() -> ExitCode {
    // v0.34.7 argv-hardening: deny other-UID /proc/$PID/cmdline reads + core dumps.
    mnemonic_toolkit::process_hardening::set_non_dumpable();

    // SPEC_constellation_cli_uniformity 6d, and THE ORDERING IS THE FIX: this
    // must decide before `Cli::try_parse()`. Measured on this binary, clap
    // echoes a stray positional verbatim -- `mnemonic convert --to xpub
    // --template bip84 "<a 12-word phrase>"` prints the whole phrase back at
    // exit 64 -- so a guard placed one line lower has already lost for any
    // shape clap has no declared flag to blame. Nothing below this point runs
    // when the guard refuses: no file is read, no environment variable is
    // resolved, and clap never sees argv.
    // F-687 fold 1 (review N1): `std::env::args()` PANICS (exit 101) on a
    // non-UTF-8 argument. Refuse it as a usage error instead, naming only the
    // position -- the bytes may be a secret, so they are never echoed.
    let argv: Vec<String> = match std::env::args_os()
        .enumerate()
        .map(|(i, a)| a.into_string().map_err(|_| i))
        .collect::<Result<Vec<String>, usize>>()
    {
        Ok(v) => v,
        Err(i) => {
            eprintln!(
                "error: argument {i} on argv (0 is `mnemonic` itself) is not valid UTF-8; \
                 refused (the value is not shown)"
            );
            return ExitCode::from(crate::exit_code::USAGE);
        }
    };
    if let argv_guard::Verdict::Refuse(findings) = argv_guard::inspect(&argv) {
        let _ = writeln!(
            io::stderr(),
            "mnemonic: {}",
            argv_guard::refusal_message(&findings)
        );
        return ExitCode::from(argv_guard::EXIT_REFUSED);
    }

    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            // Exit-code spec decision 2: usage is 64 (clap's own 2 is the
            // INVALID bit here), except a rejected input VALUE, which is 2.
            let base = clap_error_exit_code(&e);
            if e.use_stderr() {
                // A usage error goes to stderr as today; a failed stderr print stays ignored.
                e.print().ok();
                return ExitCode::from(base);
            }
            // `--help` / `--version` on stdout: a failed write follows the T6 contract.
            let mut sink = out_sink::OutSink::stdout();
            sink.check_read_only();
            if sink.error().is_none() {
                if let Err(pe) = e.print().and_then(|()| io::stdout().flush()) {
                    sink.record(pe);
                }
            }
            return finish_exit(&sink, base);
        }
    };

    let stdin = &mut io::stdin();
    let mut sink = out_sink::OutSink::stdout();
    let stdout = &mut sink;
    let stderr = &mut io::stderr();

    let result: Result<u8, ToolkitError> = match &cli.command {
        Command::Bundle(args) => cmd::bundle::run(args, stdin, stdout, stderr),
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

    // Brian's ruling of 2026-11-26: `bundle` (an origin the user did not supply)
    // and `restore` (an origin the user asserted) exit 4 (UNVERIFIED) WITH their
    // output. If that output never reached stdout, R8 adds 32 in `finish_exit`
    // (36), as for every other verb.
    // The verb's own error line first (RepairShortCircuit prints nothing here), then the
    // write-error line from `finish_exit`.
    let code = match result {
        Ok(code) => code,
        // R2 I1: short-circuit fires a clean repair report on stderr inside
        // the helper; do NOT also emit the ToolkitError Display impl (which
        // would tack on "error: " noise). The exit code is carried in the
        // variant itself.
        Err(error::ToolkitError::RepairShortCircuit { exit_code }) => exit_code,
        Err(e) => {
            // Emit error per SPEC §6.5 + §5.5.
            let _ = writeln!(io::stderr(), "{}", e);
            e.exit_code()
        }
    };
    // R6 (decision 5): a verb that used a checksum-corrected value adds 16;
    // the plate is damaged, so say so (mk-codec corrects silently).
    let code = if crate::repair::correction_used() && code < crate::exit_code::USAGE {
        let _ = writeln!(
            io::stderr(),
            "mnemonic: note: a damaged card was corrected by BCH error correction and the \
             corrected value was used (exit bit 16); the plate is damaged -- run \
             `mnemonic repair` on it and re-engrave it"
        );
        crate::repair::with_r6(code)
    } else {
        code
    };
    let _ = sink.flush();
    finish_exit(&sink, code)
}

/// Map the exit code once (T6): a recorded stdout write error prints
/// `mnemonic: cannot write the output: <error>` on stderr and ADDS 32 to the
/// code (exit-code spec R8); 64 and 101 are kept.
fn finish_exit(sink: &out_sink::OutSink<io::Stdout>, code: u8) -> ExitCode {
    if let Some(err) = sink.error() {
        let _ = writeln!(io::stderr(), "mnemonic: cannot write the output: {err}");
    }
    let exit = ExitCode::from(sink.exit_code(code));

    // Cycle B SPEC §2 row 3 + §6 G2.5 — emit a 2-line stderr summary iff
    // any pin_pages_for call soft-failed during this invocation. No-op
    // when failure_count == 0. Runs on both Ok and Err paths.
    mnemonic_toolkit::mlock::report_at_exit();

    exit
}

#[cfg(test)]
mod tests {
    /// R11: the root `--help` ends with the spec's legend, word for word.
    #[test]
    fn root_help_ends_with_the_exit_code_legend() {
        assert!(super::ROOT_AFTER_HELP.ends_with(crate::exit_code::HELP_LEGEND));
        assert!(super::ROOT_AFTER_HELP.starts_with("RECOVERING A FORGOTTEN BIP-39 PASSPHRASE:"));
    }
}
