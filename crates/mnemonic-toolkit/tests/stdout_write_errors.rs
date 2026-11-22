//! FIX-tk-stdout-write-errors (bughunt-1119 bh-tk-I2): a failed stdout write must not exit 0.
//! Rows R1-R18, R22, R23 of `design/FIX-tk-stdout-write-errors.md` section 5. Linux only
//! (`/dev/full`). Signet, public test phrases only.
#![cfg(target_os = "linux")]
#![allow(missing_docs)]

use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_mnemonic");
const P: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const Z: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
const MS: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const K1: &str = "mk1qpg397pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycf";
const K1BAD: &str = "mk1qpg397pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycq";
const K2: &str = "mk1qpg397pp3c8mu0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02ysk3l6fyxg6m66tmvhzh70a";
const D1: &str = "md1fkwsupqpqpm6j8pqqxqqks883w6pg25zpu3ssr0mjvzcsa5rf5sfn48qvmn2cutt";
const D2: &str = "md1fkwsupqtd4z93hpxnz95xjswxaumvyr8z37ccpccsrhz8ql03lv04y7x3v7nh0ze";
const D3: &str = "md1fkwsupq3rrxlke80qvpuacg07005jcne0v3pr4fejqyqn63y95qcngq49s9nrnck";
const W: &str = "mnemonic: cannot write the output: No space left on device (os error 28)";

struct Run {
    rc: i32,
    stderr: String,
}

impl Run {
    fn last(&self) -> &str {
        self.stderr.lines().last().unwrap_or("")
    }
    fn no_io_error_line(&self) {
        assert!(
            !self.stderr.lines().any(|l| l.starts_with("error: I/O error")),
            "unexpected I/O error line: {}",
            self.stderr
        );
    }
}

enum Out {
    Full,
    ReadOnly,
}

fn run(stdin: &str, out: Out, args: &[&str]) -> Run {
    let so = match out {
        Out::Full => Stdio::from(File::options().write(true).open("/dev/full").unwrap()),
        Out::ReadOnly => Stdio::from(File::open("/dev/null").unwrap()),
    };
    let mut child = Command::new(BIN)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(so)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{stdin}\n").as_bytes())
        .ok();
    let o = child.wait_with_output().unwrap();
    Run {
        rc: o.status.code().unwrap(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

fn run_closed_pipe(stdin: &str, args: &[&str]) -> Run {
    let mut child = Command::new(BIN)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{stdin}\n").as_bytes())
        .ok();
    let mut so = child.stdout.take().unwrap();
    let mut one = [0u8; 1];
    so.read_exact(&mut one).unwrap();
    drop(so);
    let o = child.wait_with_output().unwrap();
    Run {
        rc: o.status.code().unwrap(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

const DERIVE: &[&str] = &[
    "derive-child", "--from", "phrase=-", "--application", "hex", "--length", "16", "--index", "0",
];

#[test]
fn r1_derive_child_dev_full_exits_1() {
    let r = run(P, Out::Full, DERIVE);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.last(), W);
}

#[test]
fn r2_convert_text_dev_full_exits_1() {
    let r = run(P, Out::Full, &["convert", "--from", "phrase=-", "--to", "entropy"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.last(), W);
}

#[test]
fn r3_convert_json_dev_full_exits_1() {
    let r = run(P, Out::Full, &["convert", "--from", "phrase=-", "--to", "entropy", "--json"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.last(), W);
}

#[test]
fn r4_bundle_dev_full_exits_1() {
    let r = run(P, Out::Full, &["bundle", "--template", "bip84", "--network", "signet", "--slot", "@0.phrase=-"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.last(), W);
}

#[test]
fn r5_gui_schema_dev_full_exits_1() {
    let r = run("", Out::Full, &["gui-schema"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.stderr.trim_end(), W);
}

#[test]
fn r6_addresses_dev_full_prints_the_contract_line() {
    let r = run(
        P,
        Out::Full,
        &["addresses", "--from", "phrase=-", "--address-type", "p2wpkh", "--network", "signet", "--count", "3"],
    );
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.last(), W);
    r.no_io_error_line();
}

#[test]
fn r7_addresses_closed_pipe_keeps_exit_0() {
    let r = run_closed_pipe(
        P,
        &["addresses", "--from", "phrase=-", "--address-type", "p2wpkh", "--network", "signet", "--count", "2000"],
    );
    assert_eq!(r.rc, 0, "{}", r.stderr);
    assert!(!r.stderr.contains("cannot write"), "{}", r.stderr);
    assert!(!r.stderr.contains("I/O error"), "{}", r.stderr);
}

#[test]
fn r8_gui_schema_closed_pipe_keeps_exit_0() {
    let r = run_closed_pipe("", &["gui-schema"]);
    assert_eq!(r.rc, 0, "{}", r.stderr);
    assert_eq!(r.stderr, "");
}

#[test]
fn r9_help_dev_full_exits_1() {
    let r = run("", Out::Full, &["--help"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.stderr.trim_end(), W);
}

#[test]
fn r10_version_dev_full_exits_1() {
    let r = run("", Out::Full, &["--version"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.stderr.trim_end(), W);
}

const RO: &str = "mnemonic: cannot write the output: Bad file descriptor (os error 9)";

#[test]
fn r11_gui_schema_read_only_stdout_exits_1() {
    let r = run("", Out::ReadOnly, &["gui-schema"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.stderr.trim_end(), RO);
}

#[test]
fn r12_convert_read_only_stdout_exits_1() {
    let r = run(P, Out::ReadOnly, &["convert", "--from", "phrase=-", "--to", "entropy"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.last(), RO);
}

fn vb() -> Vec<&'static str> {
    vec![
        "verify-bundle", "--allow-argv-secret", "--template", "bip84", "--network", "signet", "--ms1", MS,
        "--mk1", K1, K2, "--md1", D1, D2, D3, "--slot", "@0.phrase=-",
    ]
}

#[test]
fn r13_verify_bundle_ok_dev_full_exits_1() {
    let r = run(P, Out::Full, &vb());
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.stderr.trim_end(), W);
}

#[test]
fn r14_verify_bundle_mismatch_dev_full() {
    let r = run(Z, Out::Full, &vb());
    assert_eq!(r.rc, 4, "{}", r.stderr);
    assert_eq!(r.stderr.trim_end(), W);
}

#[test]
fn r15_repair_applied_dev_full_exits_1() {
    let r = run("", Out::Full, &["repair", "--mk1", K1BAD, K2]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.last(), W);
    r.no_io_error_line();
}

#[test]
fn r16_repair_unverified_dev_full() {
    let bad = format!("{}q", MS.strip_suffix('f').unwrap());
    let r = run(&bad, Out::Full, &["repair", "--ms1", "-"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.last(), W);
    r.no_io_error_line();
}

#[test]
fn r17_derive_child_fsize0_exits_1() {
    // A regular file, RLIMIT_FSIZE 0, SIGXFSZ ignored. Stderr is a pipe (a file would hit the same limit).
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out");
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("trap \"\" XFSZ; ulimit -f 0; exec \"$0\" \"$@\"")
        .arg(BIN)
        .args(DERIVE)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(File::create(&out).unwrap()))
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(format!("{P}\n").as_bytes()).ok();
    let o = child.wait_with_output().unwrap();
    let err = String::from_utf8_lossy(&o.stderr).into_owned();
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert_eq!(std::fs::metadata(&out).unwrap().len(), 0);
    assert_eq!(
        err.lines().last().unwrap(),
        "mnemonic: cannot write the output: File too large (os error 27)"
    );
}

#[test]
fn r18_gui_schema_closed_fd_keeps_exit_0() {
    let o = Command::new("sh")
        .arg("-c")
        .arg("exec \"$0\" gui-schema >&-")
        .arg(BIN)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert!(o.stderr.is_empty(), "{}", String::from_utf8_lossy(&o.stderr));
}

#[test]
fn r23_inspect_partial_dev_full_exits_1() {
    let r = run("", Out::Full, &["inspect", "--md1", "md1yppqqxp3cg2x3r70ckk4kjaf"]);
    assert_eq!(r.rc, 1, "{}", r.stderr);
    assert_eq!(r.last(), W);
    r.no_io_error_line();
}

fn scan(dir: &std::path::Path, hits: &mut Vec<String>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            scan(&p, hits);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let text = std::fs::read_to_string(&p).unwrap();
            for (n, l) in text.lines().enumerate() {
                let write = l.contains("writeln!(stdout")
                    || l.contains("write!(stdout")
                    || l.contains("to_writer(&mut *stdout")
                    || l.contains("to_writer_pretty(&mut *stdout");
                if write && l.contains(".ok()") {
                    hits.push(format!("{}:{}", p.display(), n + 1));
                }
            }
        }
    }
}

#[test]
fn r22_no_discarded_stdout_write_in_src() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = Vec::new();
    scan(&src, &mut hits);
    assert!(hits.is_empty(), "{} discarded stdout writes: {hits:#?}", hits.len());
}
