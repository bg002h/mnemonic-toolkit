//! seedqr `--json-out` replaces its file atomically (FIX_bh1119_tk_secret_file_writes,
//! bh-tk-I3). Each run starts the binary through `sh -c`, which sets umask 022 and,
//! for the failing rows, `ulimit -f 0` with SIGXFSZ ignored, so any write of a
//! non-empty file fails with EFBIG (os error 27) while the pipes are unaffected.
#![cfg(unix)]

use std::io::Write;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::process::{Command, Stdio};

const PHRASE_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const DIGITS_12: &str = "000000000000000000000000000000000000000000000003";
const PREVIOUS: &[u8] = b"PREVIOUS-BACKUP-BYTES\n";

const PLAIN: &str = "umask 022; exec \"$0\" \"$@\"";
const NO_SPACE: &str = "umask 022; ulimit -f 0; trap '' XFSZ; exec \"$0\" \"$@\"";

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(script: &str, args: &[&str], stdin: Option<&str>) -> Out {
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(script)
        .arg(bin)
        .arg("--allow-argv-secret")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn sh");
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

fn decode(script: &str, path: &Path) -> Out {
    let seedqr = format!("seedqr={DIGITS_12}");
    let p = path.to_str().unwrap();
    run(
        script,
        &["seedqr", "decode", "--from", &seedqr, "--json-out", p],
        None,
    )
}

fn encode(script: &str, path: &Path) -> Out {
    let p = path.to_str().unwrap();
    run(
        script,
        &["seedqr", "encode", "--from", "phrase=-", "--json-out", p],
        Some(PHRASE_12),
    )
}

/// The names in `dir`, sorted.
fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    v.sort();
    v
}

fn last_line(s: &str) -> &str {
    s.lines().last().unwrap_or("")
}

fn assert_envelope(path: &Path, op: &str) {
    let v: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).expect("file holds the envelope");
    assert_eq!(v["operation"], op);
    assert_eq!(v["phrase"], PHRASE_12);
    assert_eq!(v["digits"], DIGITS_12);
}

fn write_previous(path: &Path) {
    std::fs::write(path, PREVIOUS).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

fn failed_replace_keeps_previous_file(op: &str) {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("seed.json");
    write_previous(&p);
    let out = if op == "encode" {
        encode(NO_SPACE, &p)
    } else {
        decode(NO_SPACE, &p)
    };
    assert_eq!(out.code, 32, "exit; stderr: {}", out.stderr);
    assert_eq!(out.stdout, "", "stdout must stay empty");
    assert_eq!(
        last_line(&out.stderr),
        format!("error: seedqr: {op}: json-out write to {p:?}: File too large (os error 27)"),
        "stderr: {}",
        out.stderr
    );
    assert_eq!(
        std::fs::read(&p).unwrap(),
        PREVIOUS,
        "previous file destroyed by a failed replacement"
    );
    assert_eq!(
        names(d.path()),
        vec!["seed.json"],
        "a temp file was left behind"
    );
}

fn failed_create_leaves_no_file(op: &str) {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("seed.json");
    let out = if op == "encode" {
        encode(NO_SPACE, &p)
    } else {
        decode(NO_SPACE, &p)
    };
    assert_eq!(out.code, 32, "exit; stderr: {}", out.stderr);
    assert_eq!(
        last_line(&out.stderr),
        format!("error: seedqr: {op}: json-out write to {p:?}: File too large (os error 27)"),
        "stderr: {}",
        out.stderr
    );
    assert_eq!(
        names(d.path()),
        Vec::<String>::new(),
        "a failed create left a file"
    );
}

#[test]
fn i3_seedqr_encode_failed_replace_keeps_previous_file() {
    failed_replace_keeps_previous_file("encode");
}

#[test]
fn i3_seedqr_decode_failed_replace_keeps_previous_file() {
    failed_replace_keeps_previous_file("decode");
}

#[test]
fn i3_seedqr_encode_failed_create_leaves_no_file() {
    failed_create_leaves_no_file("encode");
}

#[test]
fn i3_seedqr_decode_failed_create_leaves_no_file() {
    failed_create_leaves_no_file("decode");
}

/// A hard link is replaced at the given name only (T5); the other name keeps the old bytes.
#[test]
fn i3_seedqr_encode_hard_link_other_name_keeps_old_bytes() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("seed.json");
    let other = d.path().join("other-name.json");
    write_previous(&p);
    std::fs::hard_link(&p, &other).unwrap();
    let out = encode(PLAIN, &p);
    assert_eq!(out.code, 0, "exit; stderr: {}", out.stderr);
    assert_envelope(&p, "encode");
    assert_eq!(
        std::fs::read(&other).unwrap(),
        PREVIOUS,
        "the other hard-link name was rewritten"
    );
    assert_ne!(
        std::fs::metadata(&p).unwrap().ino(),
        std::fs::metadata(&other).unwrap().ino(),
        "the given name still shares the old inode"
    );
}

/// A writable file in a folder the holder cannot write: refused before the file is touched (T5).
#[test]
fn i3_seedqr_encode_readonly_folder_refused_file_unchanged() {
    let d = tempfile::tempdir().unwrap();
    let sub = d.path().join("ro");
    std::fs::create_dir(&sub).unwrap();
    let p = sub.join("seed.json");
    write_previous(&p);
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o555)).unwrap();
    // Root ignores the folder mode; the row has no meaning there.
    let probe = sub.join("probe");
    if std::fs::write(&probe, b"").is_ok() {
        std::fs::remove_file(&probe).unwrap();
        std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).unwrap();
        eprintln!("skipped: the folder mode is not enforced for this user");
        return;
    }
    let out = encode(PLAIN, &p);
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(out.code, 1, "exit; stderr: {}", out.stderr);
    assert_eq!(
        last_line(&out.stderr),
        format!("error: seedqr: encode: json-out write to {p:?}: Permission denied (os error 13)"),
        "stderr: {}",
        out.stderr
    );
    assert_eq!(
        std::fs::read(&p).unwrap(),
        PREVIOUS,
        "the file was rewritten in place"
    );
    assert_eq!(names(&sub), vec!["seed.json"]);
}

/// Guard (passes today and after): a symlink stays a link, its target gets the envelope.
#[test]
fn i3_seedqr_encode_symlink_target_replaced_link_kept() {
    let d = tempfile::tempdir().unwrap();
    let target = d.path().join("target.json");
    let link = d.path().join("link.json");
    write_previous(&target);
    std::os::unix::fs::symlink("target.json", &link).unwrap();
    let out = encode(PLAIN, &link);
    assert_eq!(out.code, 0, "exit; stderr: {}", out.stderr);
    assert!(std::fs::symlink_metadata(&link)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_envelope(&target, "encode");
    assert_eq!(names(d.path()), vec!["link.json", "target.json"]);
}

/// Guard (passes today and after): the run after a failed one writes a whole envelope
/// and leaves nothing else in the folder.
#[test]
fn i3_seedqr_encode_retry_after_failure_writes_envelope() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("seed.json");
    write_previous(&p);
    let failed = encode(NO_SPACE, &p);
    assert_eq!(
        failed.code, 32,
        "control: the first run fails; stderr: {}",
        failed.stderr
    );
    let out = encode(PLAIN, &p);
    assert_eq!(out.code, 0, "exit; stderr: {}", out.stderr);
    assert_eq!(out.stdout, "");
    assert_envelope(&p, "encode");
    assert_eq!(names(d.path()), vec!["seed.json"]);
}
