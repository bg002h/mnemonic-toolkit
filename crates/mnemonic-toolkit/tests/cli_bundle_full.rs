//! 16-cell parametric integration test (Task 5.2).
//!
//! Compares stdout byte-exactly against pinned fixtures in
//! `tests/vectors/v0_1/{template}-{network}.txt`. Byte-determinism is
//! guaranteed by `synthesize::derive_mk1_chunk_set_id` deriving the mk1
//! `chunk_set_id` from the policy_id_stub (mirrors md-codec's deterministic
//! CSI derivation).

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn bundle_full_16_cells_byte_exact_against_pinned_vectors() {
    for &t in &["bip44", "bip49", "bip84", "bip86"] {
        for &n in &["mainnet", "testnet", "signet", "regtest"] {
            let expected = std::fs::read_to_string(format!("tests/vectors/v0_1/{}-{}.txt", t, n))
                .expect("fixture exists");
            let out = Command::cargo_bin("mnemonic")
                .unwrap()
                .args([
                    "bundle",
                    "--slot",
                    &format!("@0.phrase={TREZOR_24}"),
                    "--network",
                    n,
                    "--template",
                    t,
                    "--no-engraving-card",
                    // mstring-grouping P4: golden is the unbroken (canonical)
                    // form; default bundle output is now space/5-grouped.
                    "--group-size",
                    "0",
                ])
                .assert()
                .success();
            let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
            assert_eq!(stdout, expected, "byte-exact mismatch for {}-{}", t, n);
        }
    }
}

/// P3 (SPEC_constellation_cli_uniformity 6c / 2a): the DEFAULT bundle text
/// output is now UNBROKEN print-once, and the separator vocabulary is
/// whitespace only.
///
/// **This test pinned the retired behaviour and is rewritten, not deleted.**
/// Before P3 it asserted space/5 at the default and a hyphen at index 5 under
/// `--separator hyphen`; both are exactly what 6c changes, so the assertions
/// are inverted rather than dropped — the print-once property they also carry
/// is unrelated to grouping and must keep being checked.
#[test]
fn bundle_default_text_is_unbroken_print_once() {
    let run = |extra: &[&str]| -> String {
        let mut args: Vec<&str> = vec![
            "bundle",
            "--slot",
            // 12-word all-zeros (single ms1 card) keeps the assertion simple.
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ];
        args.extend_from_slice(extra);
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args(&args)
            .assert()
            .success();
        String::from_utf8(out.get_output().stdout.clone()).unwrap()
    };

    // Default: the ms1 card line is UNBROKEN, print-once (it appears exactly
    // once in stdout, with no second grouped copy).
    let default_out = run(&[]);
    let ms1_line = default_out
        .lines()
        .find(|l| l.starts_with("ms10"))
        .expect("an ms1 line");
    assert!(
        !ms1_line.contains(' '),
        "P3 6c: the default is unbroken; got {ms1_line:?}"
    );
    assert_eq!(
        default_out.lines().filter(|l| *l == ms1_line).count(),
        1,
        "print-once: the ms1 card must appear exactly once; got {default_out:?}"
    );

    // The capability is unchanged -- only the default moved.
    let grouped_out = run(&["--group-size", "5"]);
    let g_line = grouped_out
        .lines()
        .find(|l| l.starts_with("ms10"))
        .expect("an ms1 line");
    assert_eq!(
        g_line.chars().nth(5),
        Some(' '),
        "--group-size 5 must still space-group; got {g_line:?}"
    );

    // 6c: `hyphen` is retired. Asserted on the MESSAGE, not on the exit code
    // alone -- every incomplete `mnemonic` invocation is also non-zero, so an
    // exit-code-only assertion would pass in both worlds. This invocation is
    // otherwise complete and exited 0 before P3.
    let hyphen = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
            "--separator",
            "hyphen",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(hyphen.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("invalid value 'hyphen' for '--separator")
            && stderr.contains("whitespace only"),
        "6c: --separator hyphen must be refused, naming what replaced it; got:\n{stderr}"
    );

    // Invalid separator → clap parse error (non-zero exit).
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
            "--separator",
            "bogus",
        ])
        .assert()
        .failure();
}

/// SPEC v0.6.1 §5.5.a — full bundle (BIP-39 entropy on stdout) MUST emit
/// the secret-on-stdout warning to stderr. Byte-exact text matches convert §7.
/// Asserts on a single template/network to keep the assertion focused; the
/// parametric test above pins the wire format.
#[test]
fn bundle_full_emits_secret_on_stdout_warning() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: stdout carries private key material (can spend)"),
        "full bundle must emit the private-key-material advisory; got stderr: {stderr:?}"
    );
}

/// SPEC v0.6.1 §5.5.a — full bundle in `--json` mode also fires the warning.
/// `any_secret_bearing()` is independent of output mode; the warning sits
/// outside `emit_unified`'s json/text branches.
#[test]
fn bundle_full_json_mode_emits_secret_on_stdout_warning() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: stdout carries private key material (can spend)"),
        "JSON-mode full bundle must emit the private-key-material advisory; got stderr: {stderr:?}"
    );
}

// ============================================================================
// Consensus-masked older() advisory on `bundle --descriptor`
// (SPEC_older_timelock_advisory, Task 5 — Adapter-A hook, Site 1).
// ============================================================================

// Two watch-only cosigner xpubs + fingerprints reused from the import-wallet
// masked-older() corpus (`cli_import_wallet_bitcoin_core.rs`). Content is
// irrelevant to the advisory; only the descriptor's `older()` operand drives it.
const MASKED_FP_A: &str = "b8688df1";
const MASKED_XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const MASKED_FP_B: &str = "28645006";
const MASKED_XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

/// `bundle --descriptor wsh(and_v(v:multi(2,...),older(65536)))` carries a
/// BIP-68 consensus-masked relative timelock (bit 16 is outside the low-16-bit
/// value field → effective value 0). The descriptor-mode hook (Site 1) must
/// emit the non-blocking advisory on stderr while the bundle still succeeds.
#[test]
fn bundle_descriptor_masked_older_emits_advisory() {
    let descriptor = "wsh(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),older(65536)))";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={MASKED_XPUB_A}"),
            "--slot",
            &format!("@0.fingerprint={MASKED_FP_A}"),
            "--slot",
            "@0.path=48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={MASKED_XPUB_B}"),
            "--slot",
            &format!("@1.fingerprint={MASKED_FP_B}"),
            "--slot",
            "@1.path=48'/0'/0'/2'",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("advisory: older(65536) is consensus-masked"),
        "descriptor-mode masked older() must emit the consensus-masked advisory; got stderr: {stderr:?}"
    );
}

/// Clean-input counterpart: `older(2016)` is a valid 16-bit relative timelock
/// (no stray bits, non-zero value) → NO advisory. Guards against the Site-1
/// hook firing on clean operands.
#[test]
fn bundle_descriptor_clean_older_emits_no_advisory() {
    let descriptor = "wsh(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),older(2016)))";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={MASKED_XPUB_A}"),
            "--slot",
            &format!("@0.fingerprint={MASKED_FP_A}"),
            "--slot",
            "@0.path=48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={MASKED_XPUB_B}"),
            "--slot",
            &format!("@1.fingerprint={MASKED_FP_B}"),
            "--slot",
            "@1.path=48'/0'/0'/2'",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("advisory: older"),
        "clean older(2016) must NOT emit an older() advisory; got stderr: {stderr:?}"
    );
}
