//! seedqr `--json-out` writes the file owner-only (0600), fresh or existing
//! (FIX_tk_seedqr_json_out_secret_file). Each run sets umask 022 in the child
//! through `sh -c 'umask 022; exec "$0" "$@"'`, so a fresh file would be 0644
//! under the umask.
#![cfg(unix)]

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

const PHRASE_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const DIGITS_12: &str = "000000000000000000000000000000000000000000000003";

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_umask_022(args: &[&str], stdin: Option<&str>) -> Out {
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let mut c = Command::new("sh");
    c.arg("-c")
        .arg("umask 022; exec \"$0\" \"$@\"")
        .arg(bin)
        .arg("--allow-argv-secret")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = c.spawn().expect("spawn sh");
    {
        let mut si = child.stdin.take().unwrap();
        if let Some(s) = stdin {
            si.write_all(s.as_bytes()).unwrap();
        }
    }
    let o = child.wait_with_output().unwrap();
    Out {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

fn mode_of(p: &std::path::Path) -> u32 {
    std::fs::metadata(p).unwrap().permissions().mode() & 0o777
}

fn decode(path: &std::path::Path) -> Out {
    let seedqr = format!("seedqr={DIGITS_12}");
    run_umask_022(
        &[
            "seedqr",
            "decode",
            "--from",
            &seedqr,
            "--json-out",
            path.to_str().unwrap(),
        ],
        None,
    )
}

fn encode(path: &std::path::Path) -> Out {
    run_umask_022(
        &[
            "seedqr",
            "encode",
            "--from",
            "phrase=-",
            "--json-out",
            path.to_str().unwrap(),
        ],
        Some(PHRASE_12),
    )
}

fn assert_envelope(path: &std::path::Path, op: &str) {
    let v: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).expect("file holds the envelope");
    assert_eq!(v["operation"], op);
    assert_eq!(v["phrase"], PHRASE_12);
    assert_eq!(v["digits"], DIGITS_12);
}

fn check(out: &Out, path: &std::path::Path, op: &str) {
    assert_eq!(out.code, 0, "exit; stderr: {}", out.stderr);
    assert_eq!(out.stdout, "", "stdout must stay empty");
    assert_envelope(path, op);
    assert!(
        !out.stderr.contains("world-readable") && !out.stderr.contains("umask"),
        "no file-mode advisory line; stderr: {}",
        out.stderr
    );
    assert_eq!(mode_of(path) & 0o077, 0, "mode is {:o}", mode_of(path));
    assert_eq!(mode_of(path), 0o600, "mode is {:o}", mode_of(path));
}

#[test]
fn seedqr_decode_json_out_fresh_file_0600() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("decode.json");
    let out = decode(&p);
    check(&out, &p, "decode");
}

#[test]
fn seedqr_encode_json_out_fresh_file_0600() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("encode.json");
    let out = encode(&p);
    check(&out, &p, "encode");
}

#[test]
fn seedqr_decode_json_out_existing_0644_tightened() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("decode.json");
    std::fs::write(&p, b"old").unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(mode_of(&p), 0o644, "control: target is 0644 before the run");
    let out = decode(&p);
    check(&out, &p, "decode");
}

#[test]
fn seedqr_encode_json_out_existing_0644_tightened() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("encode.json");
    std::fs::write(&p, b"old").unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(mode_of(&p), 0o644, "control: target is 0644 before the run");
    let out = encode(&p);
    check(&out, &p, "encode");
}
