//! `--json-out` of final-word, electrum-decrypt, slip39 combine and seed-xor
//! combine writes the file owner-only (0600), fresh or existing, through
//! `mnemonic_io_lib::write::write_private` (architect-file-output-convention-1123
//! R2/R4, unit U6). Rows: tk-json-out-umask-secret, tk-bh-i3. Each run sets
//! umask 022 in the child, so a plain `std::fs::write` would leave 0644.
//! The two `split` verbs are not covered: they wait for `write_private_new`.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ABANDON_11: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
const TV_CIPHERTEXT: &str = "ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE=";
const TV_PASSWORD: &str = "test-password";

struct Out {
    code: i32,
    stderr: String,
}

fn run(args: &[&str], stdin: Option<&str>, umask: bool) -> Out {
    use std::io::Write;
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let mut c = Command::new("sh");
    c.arg("-c")
        .arg(if umask {
            "umask 022; exec \"$0\" \"$@\""
        } else {
            "exec \"$0\" \"$@\""
        })
        .arg(bin)
        .arg("--allow-argv-secret")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = c.spawn().unwrap();
    {
        let mut si = child.stdin.take().unwrap();
        if let Some(s) = stdin {
            si.write_all(s.as_bytes()).unwrap();
        }
    }
    let o = child.wait_with_output().unwrap();
    Out {
        code: o.status.code().unwrap_or(-1),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

fn mode_of(p: &Path) -> u32 {
    std::fs::metadata(p).unwrap().permissions().mode() & 0o777
}

/// Run `make(path)` on a fresh path and on a pre-existing 0644 path; both must
/// succeed, print no mode advisory, and leave mode 0600.
fn check(make: impl Fn(&Path) -> Out, expect_in_file: &str) {
    let d = tempfile::tempdir().unwrap();
    for pre in [false, true] {
        let p = d.path().join(if pre { "pre.json" } else { "fresh.json" });
        if pre {
            std::fs::write(&p, b"old").unwrap();
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
        }
        let out = make(&p);
        assert_eq!(out.code, 0, "pre={pre}; stderr: {}", out.stderr);
        assert!(
            !out.stderr.contains("world-readable") && !out.stderr.contains("umask"),
            "pre={pre}: no file-mode advisory; stderr: {}",
            out.stderr
        );
        let body = std::fs::read_to_string(&p).unwrap();
        assert!(body.contains(expect_in_file), "pre={pre}: {body}");
        assert_eq!(mode_of(&p) & 0o077, 0, "pre={pre}: mode {:o}", mode_of(&p));
        assert_eq!(mode_of(&p), 0o600, "pre={pre}: mode {:o}", mode_of(&p));
    }
}

#[test]
fn tk_json_out_umask_secret_final_word_0600() {
    check(
        |p| {
            run(
                &[
                    "final-word",
                    "--from",
                    "phrase=-",
                    "--json-out",
                    p.to_str().unwrap(),
                ],
                Some(ABANDON_11),
                true,
            )
        },
        "candidates",
    );
}

#[test]
fn tk_json_out_umask_secret_electrum_decrypt_0600() {
    check(
        |p| {
            run(
                &[
                    "electrum-decrypt",
                    "--ciphertext",
                    TV_CIPHERTEXT,
                    "--decrypt-password",
                    TV_PASSWORD,
                    "--json-out",
                    p.to_str().unwrap(),
                ],
                None,
                true,
            )
        },
        "plaintext",
    );
}

fn split_shares_slip39() -> Vec<String> {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("split.json");
    let from = format!("phrase={ABANDON_12}");
    let o = run(
        &[
            "slip39",
            "split",
            "--from",
            &from,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
            "--json-out",
            p.to_str().unwrap(),
        ],
        None,
        false,
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
    v["groups"][0]["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn tk_json_out_umask_secret_slip39_combine_0600() {
    let shares = split_shares_slip39();
    check(
        |p| {
            run(
                &[
                    "slip39",
                    "combine",
                    "--share",
                    &shares[0],
                    "--share",
                    &shares[1],
                    "--json-out",
                    p.to_str().unwrap(),
                ],
                None,
                true,
            )
        },
        "\"combine\"",
    );
}

#[test]
fn tk_json_out_umask_secret_seed_xor_combine_0600() {
    let d = tempfile::tempdir().unwrap();
    let sp = d.path().join("split.json");
    let from = format!("phrase={ABANDON_12}");
    let o = run(
        &[
            "seed-xor",
            "split",
            "--from",
            &from,
            "--shares",
            "2",
            "--deterministic-from-master",
            "--json-out",
            sp.to_str().unwrap(),
        ],
        None,
        false,
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(&sp).unwrap()).unwrap();
    let s: Vec<String> = v["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| format!("phrase={}", x.as_str().unwrap()))
        .collect();
    check(
        |p| {
            run(
                &[
                    "seed-xor",
                    "combine",
                    "--share",
                    &s[0],
                    "--share",
                    &s[1],
                    "--shares",
                    "2",
                    "--json-out",
                    p.to_str().unwrap(),
                ],
                None,
                true,
            )
        },
        ABANDON_12,
    );
}

/// tk-bh-i3: a `--json-out` write that fails (`ulimit -f 0`, EFBIG) keeps the
/// existing file byte-identical; a plain truncating write empties it first.
#[test]
fn tk_bh_i3_failed_json_out_keeps_the_old_file() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("keep.json");
    std::fs::write(&p, b"PREVIOUS-BACKUP-BYTES\n").unwrap();
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("umask 022; ulimit -f 0; trap '' XFSZ; exec \"$0\" \"$@\"")
        .arg(bin)
        .args([
            "--allow-argv-secret",
            "final-word",
            "--from",
            "phrase=-",
            "--json-out",
        ])
        .arg(&p)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(ABANDON_11.as_bytes())
            .unwrap();
    }
    let o = child.wait_with_output().unwrap();
    assert_ne!(o.status.code(), Some(0), "the write must fail");
    assert_eq!(
        std::fs::read(&p).unwrap(),
        b"PREVIOUS-BACKUP-BYTES\n",
        "the old file was truncated"
    );
}
