//! bughunt-1119 `bh-tk-mn7` / `mn11` / `mn18`: `derive-child --application dice`
//! and `xpub-search` refuse or stream huge argument values (FIX
//! `design/FIX_tk_unbounded_args.md` section 5). Row names are the test names.
//!
//! Every abort or panic row runs under `ulimit -v` (a failed allocation is then
//! deterministic on any host) with a 60 s deadline. Signet only: the master is
//! the public BIP-39 vector `abandon x11 about`.

use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_mnemonic");

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// Signet master tprv of PHRASE.
const T: &str = "tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd";
/// Signet `m/84'/1'/0'` tpub of PHRASE.
const X: &str = "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";
/// Signet `m/44'/1'/0'` tpub of PHRASE.
const X44: &str = "tpubDC5FSnBiZDMmhiuCmWAYsLwgLYrrT9rAqvTySfuCCrgsWz8wxMXUS9Tb9iVMvcRbvFcAHGkMD5Kx8koh4GquNGNTfohfk7pgjhaPCdXpoba";

const TX_R: &str = "warning: This rehearsal used your real seeds. Everything it engraved or printed from them is as secret as your real kit: destroy it, do not discard it.";
const ADVISORY: &str = "warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')\n";
const W1: &str = "error: --min-account / --max-account / --number-of-accounts: the account window must stay below 2147483648 (2^31): BIP-32 hardened account indexes are 31 bits\n";
const DEADLINE: Duration = Duration::from_secs(60);

struct Run {
    code: Option<i32>,
    signal: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    timed_out: bool,
}

impl Run {
    fn out(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
    fn err(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

fn spawn(ulimit_kb: Option<u32>, args: &[&str], stdin: &str) -> std::process::Child {
    let script = match ulimit_kb {
        Some(kb) => format!("ulimit -v {kb} && exec \"$0\" \"$@\""),
        None => "exec \"$0\" \"$@\"".to_string(),
    };
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(script)
        .arg(BIN)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sh");
    let mut si = child.stdin.take().unwrap();
    let data = stdin.to_string();
    std::thread::spawn(move || {
        let _ = si.write_all(data.as_bytes());
    });
    child
}

/// Run to completion (or the deadline), draining both pipes concurrently.
fn run(ulimit_kb: Option<u32>, args: &[&str], stdin: &str) -> Run {
    let mut child = spawn(ulimit_kb, args, stdin);
    let mut so = child.stdout.take().unwrap();
    let mut se = child.stderr.take().unwrap();
    let ho = std::thread::spawn(move || {
        let mut v = Vec::new();
        let _ = so.read_to_end(&mut v);
        v
    });
    let he = std::thread::spawn(move || {
        let mut v = Vec::new();
        let _ = se.read_to_end(&mut v);
        v
    });
    let deadline = Instant::now() + DEADLINE;
    let mut timed_out = false;
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if Instant::now() > deadline {
            timed_out = true;
            child.kill().ok();
            break child.wait().unwrap();
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    Run {
        code: status.code(),
        signal: status.signal(),
        stdout: ho.join().unwrap(),
        stderr: he.join().unwrap(),
        timed_out,
    }
}

fn dice_args(length: &str) -> Vec<&str> {
    vec![
        "derive-child",
        "--from",
        "xprv=-",
        "--application",
        "dice",
        "--dice-sides",
        "6",
        "--length",
        length,
        "--index",
        "0",
        // T is a tprv: the network rule refuses it on mainnet.
        "--network",
        "testnet",
    ]
}

fn stdin_t() -> String {
    format!("{T}\n")
}

#[test]
#[cfg(not(target_os = "macos"))] // `ulimit -v` is unsupported on macOS
fn bh_tk_mn7_dice_long_child_streams_byte_identical() {
    let r = run(Some(200000), &dice_args("10000000"), &stdin_t());
    assert!(!r.timed_out, "deadline; stderr: {}", r.err());
    assert_eq!(r.signal, None, "died by signal; stderr: {}", r.err());
    assert_eq!(r.code, Some(0), "stderr: {}", r.err());
    assert_eq!(r.stdout.len(), 20000000);
    assert_eq!(
        hex::encode(Sha256::digest(&r.stdout)),
        "2d14b9d9f52ab6379a461cd7d41c6bc01a36d69c4aa1c8727d1b9d39d3a62c59"
    );
    // Spec B unit 8a (N12): a test run whose stdout is private key material
    // made from a tprv ends with TX-R.
    assert_eq!(r.err(), format!("{ADVISORY}{TX_R}\n"));
}

#[test]
fn bh_tk_mn7_dice_length_outside_range_is_refused() {
    for n in ["0", "4294967295"] {
        let r = run(None, &dice_args(n), &stdin_t());
        assert!(!r.timed_out);
        assert_eq!(r.code, Some(2), "N={n}; stderr: {}", r.err());
        assert_eq!(r.out(), "", "N={n}");
        assert_eq!(
            r.err(),
            format!("error: --length {n} out of range for --application dice (valid: 1..=2^31-1 rolls)\n"),
            "N={n}"
        );
    }
}

#[test]
#[cfg(not(target_os = "macos"))] // `ulimit -v` is unsupported on macOS
fn bh_tk_mn7_dice_max_length_streams_at_once() {
    let mut child = spawn(Some(200000), &dice_args("2147483647"), &stdin_t());
    let mut so = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = [0u8; 63];
        let r = so.read_exact(&mut buf).map(|_| buf);
        let _ = tx.send(r.map_err(|e| e.to_string()));
    });
    let got = rx.recv_timeout(DEADLINE);
    child.kill().ok();
    child.wait().ok();
    let got = got
        .expect("first 63 bytes within 60 s")
        .expect("read 63 bytes");
    assert_eq!(
        std::str::from_utf8(&got).unwrap(),
        "0,5,0,4,0,2,0,0,2,5,5,1,1,4,0,4,0,0,4,5,0,1,3,3,1,4,3,0,4,4,3,5"
    );
}

fn xs(sub: &str, tail: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = ["xpub-search", sub, "--phrase-stdin", "--network", "signet"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    v.extend(tail.iter().map(|s| s.to_string()));
    v
}

fn run_xs(ulimit_kb: Option<u32>, args: Vec<String>) -> Run {
    let a: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run(ulimit_kb, &a, PHRASE)
}

fn assert_w1(r: &Run) {
    assert!(!r.timed_out, "deadline; stderr: {}", r.err());
    assert_eq!(r.signal, None, "signal; stderr: {}", r.err());
    assert_eq!(r.code, Some(1), "stderr: {}", r.err());
    assert_eq!(r.out(), "");
    assert_eq!(r.err(), W1);
}

#[test]
fn bh_tk_mn11_path_of_xpub_window_crossing_2p31_is_refused() {
    let r = run_xs(
        None,
        xs(
            "path-of-xpub",
            &[
                "--target-xpub",
                X,
                "--min-account",
                "2147483647",
                "--number-of-accounts",
                "2",
            ],
        ),
    );
    assert_w1(&r);
}

#[test]
#[cfg(not(target_os = "macos"))] // `ulimit -v` is unsupported on macOS
fn bh_tk_mn11_path_of_xpub_max_account_2p31_is_refused() {
    let r = run_xs(
        Some(4000000),
        xs(
            "path-of-xpub",
            &["--target-xpub", X, "--max-account", "2147483648"],
        ),
    );
    assert_w1(&r);
}

#[test]
fn bh_tk_mn11_passphrase_of_xpub_min_account_2p31_is_refused() {
    let r = run_xs(
        None,
        xs(
            "passphrase-of-xpub",
            &[
                "--passphrase",
                "",
                "--target-xpub",
                X,
                "--min-account",
                "2147483648",
            ],
        ),
    );
    assert_w1(&r);
}

#[test]
fn bh_tk_mn11_window_ending_at_2p31_is_accepted() {
    let r = run_xs(
        None,
        xs(
            "path-of-xpub",
            &[
                "--target-xpub",
                X,
                "--min-account",
                "2147483646",
                "--number-of-accounts",
                "2",
            ],
        ),
    );
    assert!(!r.timed_out);
    assert_eq!(r.code, Some(4), "stderr: {}", r.err());
    assert!(
        r.err()
            .starts_with("error: no match in searched set: mode=path-of-xpub, paths searched=14; widen the range"),
        "stderr: {}",
        r.err()
    );
}

#[test]
fn bh_tk_mn18_add_path_count_unchanged() {
    let r = run_xs(
        None,
        xs(
            "path-of-xpub",
            &[
                "--target-xpub",
                X,
                "--number-of-accounts",
                "3",
                "--add-path",
                "m/1account'",
                "--add-path",
                "m/9'/account",
            ],
        ),
    );
    assert_eq!(r.code, Some(0), "stderr: {}", r.err());
    let o = r.out();
    assert!(
        o.contains("match: m/84'/1'/0'  (template=bip84, account=0)\n"),
        "{o}"
    );
    assert!(o.contains("searched: 27 candidate paths\n"), "{o}");
}

const TARGET_48: &str = "tpubDEYM1BmQ5rp2PWKvCgvQxNeUrEv8gu5819xRdmu6S23fYpS8x2icwAeoVaBTLyN3fGWJQcWoaiKMduTXWKtG9bXNpVrZPRF7XVxrANtAEcR";
const TARGET_87: &str = "tpubDCChhoz5QdrkoaTfhN9wz9UwMkRvBNezCAcbLJ3yqswVCnGtCvuBtmub29oFk2CEty4eu15WDGa7aJ3QcHKp9JtggPakJ8JaFz4HHtCnA65";

fn order_row(target: &str, adds: &[&str], line: &str, searched: u32) {
    let mut tail = vec!["--target-xpub", target, "--number-of-accounts", "2"];
    for a in adds {
        tail.push("--add-path");
        tail.push(a);
    }
    let r = run_xs(None, xs("path-of-xpub", &tail));
    assert_eq!(r.code, Some(0), "stderr: {}", r.err());
    assert_eq!(
        r.out(),
        format!("match: {line}\ntarget-xpub: {target}\nsearched: {searched} candidate paths\n")
    );
}

#[test]
fn bh_tk_mn18_order_bip48_before_add_path() {
    order_row(
        TARGET_48,
        &["m/48'/1'/account'/2'"],
        "m/48'/1'/1'/2'  (template=bip48-wsh, account=1)",
        16,
    );
}

#[test]
fn bh_tk_mn18_order_add_paths_in_argv_order() {
    order_row(
        TARGET_87,
        &["m/87'/1'/1'", "m/87'/1'/account'"],
        "m/87'/1'/1'  (template=m/87'/1'/1', account=n/a)",
        17,
    );
}

#[test]
fn bh_tk_mn18_order_templates_outer_accounts_inner() {
    order_row(
        TARGET_87,
        &["m/87'/1'/account'", "m/87'/1'/account1'"],
        "m/87'/1'/1'  (template=m/87'/1'/account', account=1)",
        18,
    );
}

const FULL: &str = "15032385536";

#[test]
#[cfg(not(target_os = "macos"))] // `ulimit -v` is unsupported on macOS
fn bh_tk_mn18_path_of_xpub_full_hardened_window_streams() {
    let r = run_xs(
        Some(4000000),
        xs(
            "path-of-xpub",
            &["--target-xpub", X44, "--max-account", "2147483647"],
        ),
    );
    assert!(!r.timed_out, "deadline; stderr: {}", r.err());
    assert_eq!(r.signal, None, "signal; stderr: {}", r.err());
    assert_eq!(r.code, Some(0), "stderr: {}", r.err());
    assert_eq!(
        r.out(),
        format!("match: m/44'/1'/0'  (template=bip44, account=0)\ntarget-xpub: {X44}\nsearched: {FULL} candidate paths\n")
    );
    let j = run_xs(
        Some(4000000),
        xs(
            "path-of-xpub",
            &[
                "--target-xpub",
                X44,
                "--max-account",
                "2147483647",
                "--json",
            ],
        ),
    );
    assert!(!j.timed_out && j.signal.is_none(), "stderr: {}", j.err());
    assert_eq!(j.code, Some(0), "stderr: {}", j.err());
    let v: serde_json::Value = serde_json::from_str(&j.out()).expect("json");
    assert_eq!(v["searched_count"].as_u64(), Some(15032385536), "{v}");
}

#[test]
#[cfg(not(target_os = "macos"))] // `ulimit -v` is unsupported on macOS
fn bh_tk_mn18_account_of_descriptor_full_hardened_window_streams() {
    let desc = format!("wsh(sortedmulti(1,{X44}/<0;1>/*,{X44}/<0;1>/*))");
    let r = run_xs(
        Some(4000000),
        xs(
            "account-of-descriptor",
            &["--descriptor", &desc, "--max-account", "2147483647"],
        ),
    );
    assert!(!r.timed_out, "deadline; stderr: {}", r.err());
    assert_eq!(r.signal, None, "signal; stderr: {}", r.err());
    assert_eq!(r.code, Some(0), "stderr: {}", r.err());
    assert_eq!(
        r.out(),
        format!("match: cosigner @0  m/44'/1'/0'  (template=bip44, account=0)\nmatch: cosigner @1  m/44'/1'/0'  (template=bip44, account=0)\ncosigners total: 2; matched: 2\nsearched: {FULL} candidates × 2 cosigners\n")
    );
}

#[test]
#[cfg(not(target_os = "macos"))] // `ulimit -v` is unsupported on macOS
fn bh_tk_mn18_passphrase_of_xpub_full_hardened_window_streams() {
    let r = run_xs(
        Some(4000000),
        xs(
            "passphrase-of-xpub",
            &[
                "--passphrase",
                "",
                "--target-xpub",
                X44,
                "--max-account",
                "2147483647",
            ],
        ),
    );
    assert!(!r.timed_out, "deadline; stderr: {}", r.err());
    assert_eq!(r.signal, None, "signal; stderr: {}", r.err());
    assert_eq!(r.code, Some(0), "stderr: {}", r.err());
    assert!(
        r.out()
            .ends_with(&format!("searched: {FULL} candidate paths\n")),
        "stdout: {}",
        r.out()
    );
}
