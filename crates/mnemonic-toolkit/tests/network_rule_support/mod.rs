//! Shared helpers of the network-rule test files (`cli_network_rule_*.rs`).
//! Public test data only: `$X $T $P ...` are the gate's
//! (`scripts/spec-network-rule-gate.sh`), fixtures are `design/network-rule/fx`.
#![allow(dead_code)]

use assert_cmd::Command;
use std::path::PathBuf;

pub const P: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
pub const T1: &str = "[704c7836/84h/1h/0h]tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";
pub const T2: &str = "[97139860/84h/1h/0h]tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X";
pub const X1: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
pub const X2: &str = "[97139860/48h/0h/0h/2h]xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
pub const X1C1: &str = "[11111111/48h/1h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";

/// The TX-N line (Spec A §3).
pub const TX_N: &str = "warning: nothing here names a network and no --network was given: read as mainnet. For a signet, testnet or regtest wallet, rerun with --network signet, --network testnet or --network regtest.";

/// A fixture file's text.
pub fn fx_text(name: &str) -> String {
    std::fs::read_to_string(fx().join(name)).expect("fixture")
}

/// `mdargs`: the md1 lines of a bundle file as `--md1` arguments.
pub fn mdargs(name: &str) -> Vec<String> {
    fx_text(name)
        .lines()
        .filter(|l| l.starts_with("md1"))
        .flat_map(|l| ["--md1".to_string(), l.to_string()])
        .collect()
}

/// `cos FILE L1 L2 N`: lines L1..L2 (1-based) as `--cosigner @N=` arguments.
pub fn cos(name: &str, l1: usize, l2: usize, n: usize) -> Vec<String> {
    fx_text(name)
        .lines()
        .skip(l1 - 1)
        .take(l2 - l1 + 1)
        .flat_map(|l| ["--cosigner".to_string(), format!("@{n}={l}")])
        .collect()
}

/// Run with owned args.
pub fn run_s(args: &[String]) -> std::process::Output {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&a)
}

pub const X: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
pub const T: &str = "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";

pub fn fx() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../design/network-rule/fx")
}

pub fn run(args: &[&str]) -> std::process::Output {
    Command::cargo_bin("mnemonic")
        .expect("binary")
        .current_dir(fx())
        .env_remove("MNEMONIC_NETWORK_JSON")
        .args(args)
        .output()
        .expect("runs")
}

pub fn err(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}
pub fn out(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// Exit 8, stdout empty, and stderr's lines are exactly `want` (in order).
#[track_caller]
pub fn assert_refused(o: &std::process::Output, want: &[&str]) {
    assert_eq!(o.status.code(), Some(8), "stderr: {}", err(o));
    assert!(o.stdout.is_empty(), "stdout must be empty: {}", out(o));
    let e = err(o);
    let lines: Vec<&str> = e.lines().collect();
    assert_eq!(lines, want, "stderr:\n{e}");
}
