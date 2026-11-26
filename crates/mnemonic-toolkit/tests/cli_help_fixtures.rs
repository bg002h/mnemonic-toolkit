//! Help-text smoke tests.
//!
//! Uses substring matching rather than byte-exact: clap's exact rendering
//! varies across patch versions; locking to substrings keeps the test stable
//! while still asserting that the documented flags surface.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_lists_subcommands() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("bundle"))
        .stdout(predicate::str::contains("verify-bundle"));
}

#[test]
fn top_level_help_points_to_btcrecover_for_passphrase_recovery() {
    // The recon decision (2026-05-25): `mnemonic` cannot brute-force a
    // forgotten BIP-39 passphrase (no internal verifier — success is only
    // definable against a known address/xpub/fingerprint). The top-level
    // after_help footer points users at btcrecover, which does. Assert the
    // load-bearing substrings only (name + maintained repo + date stamp);
    // exact rendering is clap-version-sensitive.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("btcrecover"))
        .stdout(predicate::str::contains(
            "https://github.com/3rdIteration/btcrecover",
        ))
        .stdout(predicate::str::contains("2026-05-25"));
}

#[test]
fn bundle_help_shows_slot_and_template_flags() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["bundle", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--slot"))
        .stdout(predicate::str::contains("--network"))
        .stdout(predicate::str::contains("--template"))
        .stdout(predicate::str::contains("--no-engraving-card"));
}

#[test]
fn verify_bundle_help_shows_required_flags() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["verify-bundle", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--mk1"))
        .stdout(predicate::str::contains("--md1"))
        .stdout(predicate::str::contains("--ms1"))
        .stdout(predicate::str::contains("--network"))
        .stdout(predicate::str::contains("--template"));
}

/// The constellation exit-code legend (spec R11), word for word. Tools such
/// as mnemonic-gui read the end of a binary's root `--help` to tell the new
/// exit-code scheme from the old one, so the legend must be the LAST text of
/// the root help, in every form that prints it.
const EXIT_CODE_LEGEND: &str = "\
Exit status: 0 = done, every check passed. 64 = bad command line, nothing
ran. 101 = internal error. 130 = stopped by the operator. Other codes from
128 up = killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said
no; 2 an input is invalid; 4 verify the result before trusting it; 8 refused
(network mismatch, secret on the command line, unsafe output); 16 a
correction was applied; 32 a file, device or helper failed.";

#[test]
fn root_help_ends_with_the_exit_code_legend() {
    for args in [&["--help"][..], &["-h"][..], &["help"][..]] {
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "{args:?}");
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(
            stdout.trim_end().ends_with(EXIT_CODE_LEGEND),
            "{args:?}: root help does not end with the legend:\n{stdout}"
        );
    }
}
