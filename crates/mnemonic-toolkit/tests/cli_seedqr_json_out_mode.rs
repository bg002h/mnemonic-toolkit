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

// ---- refusal and error rows (whole-diff-tk-seedqr-s2 I1, M1) ----

/// Like `run_umask_022` with a chosen umask (octal digits, e.g. `"000"`).
fn run_with_umask(mask: &str, args: &[&str], stdin: Option<&str>) -> Out {
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let script = format!("umask {mask}; exec \"$0\" \"$@\"");
    let mut c = Command::new("sh");
    c.arg("-c")
        .arg(script)
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

/// Run `seedqr <verb> --json-out <path>` under `mask`.
fn run_verb(verb: &str, mask: &str, path: &std::path::Path) -> Out {
    let p = path.to_str().unwrap();
    let seedqr = format!("seedqr={DIGITS_12}");
    if verb == "decode" {
        run_with_umask(
            mask,
            &["seedqr", "decode", "--from", &seedqr, "--json-out", p],
            None,
        )
    } else {
        run_with_umask(
            mask,
            &["seedqr", "encode", "--from", "phrase=-", "--json-out", p],
            Some(PHRASE_12),
        )
    }
}

fn is_root() -> bool {
    std::fs::read_to_string("/proc/self/status")
        .map(|s| {
            s.lines()
                .any(|l| l.starts_with("Uid:") && l.split_whitespace().nth(2) == Some("0"))
        })
        .unwrap_or(false)
}

fn longer_file_truncated(verb: &str) {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("o.json");
    std::fs::write(&p, vec![b'x'; 4096]).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
    let out = run_verb(verb, "022", &p);
    check(&out, &p, verb);
    let len = std::fs::metadata(&p).unwrap().len();
    assert!(len < 4096, "no tail of the old file: len {len}");
}

fn symlink_target_0600(verb: &str) {
    let d = tempfile::tempdir().unwrap();
    let real = d.path().join("real.json");
    let link = d.path().join("link.json");
    std::fs::write(&real, b"old").unwrap();
    std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let out = run_verb(verb, "022", &link);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert!(std::fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_envelope(&real, verb);
    assert_eq!(mode_of(&real), 0o600, "target mode {:o}", mode_of(&real));
}

fn dev_null_refused(verb: &str) {
    if is_root() {
        eprintln!("skip: running as root, /dev/null fchmod is allowed");
        return;
    }
    let out = run_verb(verb, "022", std::path::Path::new("/dev/null"));
    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    let want = format!(
        "error: seedqr: {verb}: json-out write to \"/dev/null\": Operation not permitted (os error 1)"
    );
    assert_eq!(
        out.stderr.trim_end().lines().last(),
        Some(want.as_str()),
        "{}",
        out.stderr
    );
}

fn missing_parent_refused(verb: &str) {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("no-such-dir").join("o.json");
    let out = run_verb(verb, "022", &p);
    assert_eq!(out.code, 32, "stderr: {}", out.stderr);
    assert!(
        out.stderr
            .contains(&format!("seedqr: {verb}: json-out write to")),
        "{}",
        out.stderr
    );
    assert!(!p.exists());
    assert!(!d.path().join("no-such-dir").exists());
}

fn readonly_parent_refused(verb: &str) {
    if is_root() {
        eprintln!("skip: running as root, a read-only directory does not refuse");
        return;
    }
    let d = tempfile::tempdir().unwrap();
    let sub = d.path().join("ro");
    std::fs::create_dir(&sub).unwrap();
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o555)).unwrap();
    let p = sub.join("o.json");
    let out = run_verb(verb, "022", &p);
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    assert!(
        out.stderr
            .contains(&format!("seedqr: {verb}: json-out write to"))
            && out.stderr.contains("Permission denied (os error 13)"),
        "{}",
        out.stderr
    );
    assert!(!p.exists());
}

fn umask_gives_0600(verb: &str, mask: &str) {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("o.json");
    let out = run_verb(verb, mask, &p);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_envelope(&p, verb);
    assert_eq!(mode_of(&p), 0o600, "mode is {:o}", mode_of(&p));
}

fn readonly_0444_file_untouched(verb: &str) {
    if is_root() {
        eprintln!("skip: running as root, a 0444 file is writable");
        return;
    }
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("o.json");
    std::fs::write(&p, b"old").unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o444)).unwrap();
    let out = run_verb(verb, "022", &p);
    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    assert!(
        out.stderr
            .contains(&format!("seedqr: {verb}: json-out write to"))
            && out.stderr.contains("Permission denied (os error 13)"),
        "{}",
        out.stderr
    );
    assert_eq!(std::fs::read(&p).unwrap(), b"old");
    assert_eq!(mode_of(&p), 0o444);
}

macro_rules! rows {
    ($($name:ident => $body:expr;)*) => { $(#[test] fn $name() { $body } )* };
}

rows! {
    json_out_longer_file_truncated_decode => longer_file_truncated("decode");
    json_out_longer_file_truncated_encode => longer_file_truncated("encode");
    json_out_symlink_target_0600_decode => symlink_target_0600("decode");
    json_out_symlink_target_0600_encode => symlink_target_0600("encode");
    json_out_dev_null_refused_decode => dev_null_refused("decode");
    json_out_dev_null_refused_encode => dev_null_refused("encode");
    json_out_missing_parent_refused_decode => missing_parent_refused("decode");
    json_out_missing_parent_refused_encode => missing_parent_refused("encode");
    json_out_readonly_parent_refused_decode => readonly_parent_refused("decode");
    json_out_readonly_parent_refused_encode => readonly_parent_refused("encode");
    json_out_umask_000_gives_0600_decode => umask_gives_0600("decode", "000");
    json_out_umask_000_gives_0600_encode => umask_gives_0600("encode", "000");
    json_out_umask_077_gives_0600_decode => umask_gives_0600("decode", "077");
    json_out_umask_077_gives_0600_encode => umask_gives_0600("encode", "077");
    json_out_readonly_0444_file_untouched_decode => readonly_0444_file_untouched("decode");
    json_out_readonly_0444_file_untouched_encode => readonly_0444_file_untouched("encode");
}
