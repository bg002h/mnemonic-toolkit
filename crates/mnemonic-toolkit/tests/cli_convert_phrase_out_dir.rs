//! `mnemonic convert --phrase-out-dir` / `--name-with-passphrase-fp`
//! (`design/SPEC_convert_phrase_out_dir.md` §8). Public test data only: the
//! all-zero entropy (`abandon … about`), its Japanese rendering, and the
//! F-579 fixture `abandon ×23 surface` (31 zero bytes then 0x06).

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;

const ZERO16: &str = "00000000000000000000000000000000";
const PHRASE12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// SeedQR of PHRASE12: eleven `0000` (abandon) then `0003` (about).
const SEEDQR12: &str = "000000000000000000000000000000000000000000000003";
/// ms1 `entr` card of the all-zero 16-byte entropy.
const MS1_ZERO16: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// ms1 `mnem` card of the same entropy, Japanese wordlist on the wire.
const MS1_ZERO16_JA: &str = "ms10entrsqgqsqqqqqqqqqqqqqqqqqqqqqqqqqj9tawneveyd9j";
const FP: &str = "73c5da0a";
/// Master fingerprint of PHRASE12 under passphrase TREZOR (BIP-39 test vectors).
const FP_TREZOR: &str = "b4e3f5ed";
/// Master fingerprint of the Japanese rendering of the all-zero entropy.
const FP_JA: &str = "0ed2c5a4";
/// The Japanese rendering as `convert --to phrase` prints it (rust-bip39's
/// own bytes, so NFKD: `ぞ` is `そ` + U+3099).
fn ja12() -> String {
    let r = run(
        &[
            "convert",
            "--from",
            "entropy=-",
            "--to",
            "phrase",
            "--language",
            "japanese",
        ],
        Some(ZERO16),
    );
    let w = r
        .stdout
        .trim()
        .strip_prefix("phrase: ")
        .unwrap()
        .to_string();
    assert!(w.starts_with("あいこくしん "), "{w}");
    w
}
/// F-579: every word is in the English AND French lists.
const AMBIGUOUS: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon surface";
const AMBIGUOUS_ENTROPY: &str = "0000000000000000000000000000000000000000000000000000000000000006";

fn mn() -> Command {
    let mut c = Command::cargo_bin("mnemonic").unwrap();
    c.arg("--allow-argv-secret");
    c
}

fn today() -> String {
    let out = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn entries(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    v.sort();
    v
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str], stdin: Option<&str>) -> Run {
    let mut c = mn();
    c.args(args);
    if let Some(s) = stdin {
        c.write_stdin(s);
    }
    let o = c.output().unwrap();
    Run {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

fn dir_arg(d: &tempfile::TempDir) -> String {
    d.path().to_str().unwrap().to_string()
}

/// The single saved file; its date part is the local date at the start or
/// end of the run (a midnight crossing is allowed).
fn the_file(dir: &Path, before: &str, tail: &str) -> std::path::PathBuf {
    let after = today();
    let es = entries(dir);
    assert_eq!(es.len(), 1, "{es:?}");
    let ok = es[0] == format!("seed-{before}-{tail}") || es[0] == format!("seed-{after}-{tail}");
    assert!(ok, "got {:?}, want seed-<{before}|{after}>-{tail}", es[0]);
    dir.join(&es[0])
}

fn save_entropy(d: &tempfile::TempDir, extra: &[&str]) -> Run {
    let dir = dir_arg(d);
    let mut a = vec![
        "convert",
        "--from",
        "entropy=-",
        "--to",
        "phrase",
        "--phrase-out-dir",
        &dir,
    ];
    a.extend_from_slice(extra);
    run(&a, Some(ZERO16))
}

#[cfg(unix)]
fn mode_of(p: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(p).unwrap().permissions().mode() & 0o7777
}

// ---------------------------------------------------------------- name

#[test]
fn name_is_date_and_fingerprint_with_no_template() {
    let d = tempfile::tempdir().unwrap();
    let before = today();
    let r = save_entropy(&d, &[]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let f = the_file(d.path(), &before, &format!("{FP}.txt"));
    assert!(
        r.stderr.contains(&format!(
            "saved phrase to {} (fingerprint {FP}, without passphrase)",
            f.display()
        )),
        "{}",
        r.stderr
    );
}

#[test]
fn same_name_from_entropy_phrase_seedqr_and_ms1_inputs() {
    let inputs: [(&str, &str); 4] = [
        ("entropy=-", ZERO16),
        ("phrase=-", PHRASE12),
        ("seedqr=-", SEEDQR12),
        ("ms1=-", MS1_ZERO16),
    ];
    let mut names = Vec::new();
    for (from, val) in inputs {
        let d = tempfile::tempdir().unwrap();
        let dir = dir_arg(&d);
        let r = run(
            &[
                "convert",
                "--from",
                from,
                "--to",
                "phrase",
                "--phrase-out-dir",
                &dir,
            ],
            Some(val),
        );
        assert_eq!(r.code, 0, "{from}: {}", r.stderr);
        let es = entries(d.path());
        assert_eq!(es.len(), 1, "{from}: {es:?}");
        assert!(es[0].ends_with(&format!("-{FP}.txt")), "{from}: {}", es[0]);
        assert_eq!(
            fs::read_to_string(d.path().join(&es[0])).unwrap(),
            format!("{PHRASE12}\n")
        );
        names.push(es[0].clone());
    }
    names.dedup();
    assert!(names.len() <= 2, "{names:?}"); // at most a midnight crossing
}

// ------------------------------------------------------------ language

#[test]
fn english_has_no_suffix_japanese_entropy_has_one_and_its_own_fingerprint() {
    let d = tempfile::tempdir().unwrap();
    let before = today();
    let r = save_entropy(&d, &["--language", "japanese"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let f = the_file(d.path(), &before, &format!("{FP_JA}-japanese.txt"));
    assert_eq!(fs::read_to_string(&f).unwrap(), format!("{}\n", ja12()));
    // The fingerprint is of the Japanese words, as `--to fingerprint` says.
    let fp = run(
        &[
            "convert",
            "--from",
            "entropy=-",
            "--to",
            "fingerprint",
            "--template",
            "bip84",
            "--language",
            "japanese",
        ],
        Some(ZERO16),
    );
    assert_eq!(fp.stdout.trim(), format!("fingerprint: {FP_JA}"));
}

#[test]
fn japanese_ms1_mnem_card_suffix_follows_the_wire_language() {
    for extra in [&[][..], &["--language", "spanish"][..]] {
        let d = tempfile::tempdir().unwrap();
        let dir = dir_arg(&d);
        let before = today();
        let mut a = vec![
            "convert",
            "--from",
            "ms1=-",
            "--to",
            "phrase",
            "--phrase-out-dir",
            &dir,
        ];
        a.extend_from_slice(extra);
        let r = run(&a, Some(MS1_ZERO16_JA));
        assert_eq!(r.code, 0, "{extra:?}: {}", r.stderr);
        let f = the_file(d.path(), &before, &format!("{FP_JA}-japanese.txt"));
        assert_eq!(fs::read_to_string(&f).unwrap(), format!("{}\n", ja12()));
    }
}

// ---------------------------------------------------------- passphrase

#[test]
fn passphrase_without_the_flag_one_fingerprint_and_the_ignored_warning() {
    let d = tempfile::tempdir().unwrap();
    let dir = dir_arg(&d);
    let before = today();
    let mut c = mn();
    c.env("PP", "TREZOR").args([
        "convert",
        "--from",
        "entropy=-",
        "--to",
        "phrase",
        "--phrase-out-dir",
        &dir,
        "--passphrase",
        "@env:PP",
    ]);
    let o = c.write_stdin(ZERO16).output().unwrap();
    let err = String::from_utf8(o.stderr).unwrap();
    assert_eq!(o.status.code(), Some(0), "{err}");
    the_file(d.path(), &before, &format!("{FP}.txt"));
    assert!(
        err.contains("warning: --passphrase ignored on this edge"),
        "{err}"
    );
}

#[test]
fn passphrase_with_the_flag_both_fingerprints_raw_first_no_warning() {
    let d = tempfile::tempdir().unwrap();
    let dir = dir_arg(&d);
    let before = today();
    let mut c = mn();
    c.env("PP", "TREZOR").args([
        "convert",
        "--from",
        "phrase=-",
        "--to",
        "phrase",
        "--phrase-out-dir",
        &dir,
        "--passphrase",
        "@env:PP",
        "--name-with-passphrase-fp",
    ]);
    let o = c.write_stdin(PHRASE12).output().unwrap();
    let err = String::from_utf8(o.stderr).unwrap();
    assert_eq!(o.status.code(), Some(0), "{err}");
    let f = the_file(d.path(), &before, &format!("{FP}-{FP_TREZOR}.txt"));
    assert!(!err.contains("ignored on this edge"), "{err}");
    assert!(
        err.contains(&format!(
            "(fingerprint {FP} without passphrase, {FP_TREZOR} with it)"
        )),
        "{err}"
    );
    // The passphrase is in neither the file nor its name.
    let body = fs::read_to_string(&f).unwrap();
    assert_eq!(body, format!("{PHRASE12}\n"));
    assert!(!f.to_str().unwrap().contains("TREZOR"));
    assert!(!body.contains("TREZOR"));
    // And the second fingerprint is what `--to fingerprint` gives with it.
    let mut c = mn();
    c.env("PP", "TREZOR").args([
        "convert",
        "--from",
        "entropy=-",
        "--to",
        "fingerprint",
        "--template",
        "bip84",
        "--passphrase",
        "@env:PP",
    ]);
    let o = c.write_stdin(ZERO16).output().unwrap();
    assert_eq!(
        String::from_utf8(o.stdout).unwrap().trim(),
        format!("fingerprint: {FP_TREZOR}")
    );
}

#[test]
fn the_flag_with_an_empty_or_absent_passphrase_is_exit_1() {
    for extra in [&[][..], &["--passphrase", ""][..]] {
        let d = tempfile::tempdir().unwrap();
        let mut a = vec!["--name-with-passphrase-fp"];
        a.extend_from_slice(extra);
        let r = save_entropy(&d, &a);
        assert_eq!(r.code, 1, "{extra:?}: {}", r.stderr);
        assert!(
            r.stderr
                .contains("--name-with-passphrase-fp needs a non-empty --passphrase"),
            "{}",
            r.stderr
        );
        assert!(entries(d.path()).is_empty());
    }
}

#[test]
fn the_flag_without_phrase_out_dir_is_exit_64() {
    let r = run(
        &[
            "convert",
            "--from",
            "entropy=-",
            "--to",
            "phrase",
            "--name-with-passphrase-fp",
        ],
        Some(ZERO16),
    );
    assert_eq!(r.code, 64, "{}", r.stderr);
    assert!(r.stdout.is_empty());
}

// ------------------------------------------------------ phrase → phrase

#[test]
fn phrase_to_phrase_under_the_flag_writes_the_canonical_words() {
    let d = tempfile::tempdir().unwrap();
    let dir = dir_arg(&d);
    let before = today();
    let messy = PHRASE12.replace(' ', " \t  ");
    let r = run(
        &[
            "convert",
            "--from",
            "phrase=-",
            "--to",
            "phrase",
            "--phrase-out-dir",
            &dir,
        ],
        Some(&format!("  {messy}\n\n")),
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
    let f = the_file(d.path(), &before, &format!("{FP}.txt"));
    assert_eq!(fs::read(&f).unwrap(), format!("{PHRASE12}\n").as_bytes());
}

#[test]
fn phrase_to_phrase_without_the_flag_keeps_the_refusal_byte_identical() {
    let r = run(
        &["convert", "--from", "phrase=-", "--to", "phrase"],
        Some(PHRASE12),
    );
    assert_eq!(r.code, 2);
    assert_eq!(r.stdout, "");
    assert_eq!(
        r.stderr,
        "error: --to phrase is cryptographically unrecoverable from --from phrase (one-way derivation barrier)\n"
    );
}

// ------------------------------------------------------------ contents

#[test]
fn contents_round_trip_through_bundle() {
    for (lang, entropy_lang) in [
        (None, &[][..]),
        (Some("japanese"), &["--language", "japanese"][..]),
    ] {
        let d = tempfile::tempdir().unwrap();
        let r = save_entropy(&d, entropy_lang);
        assert_eq!(r.code, 0, "{}", r.stderr);
        let es = entries(d.path());
        let body = fs::read_to_string(d.path().join(&es[0])).unwrap();
        assert!(body.ends_with('\n') && !body.ends_with("\n\n"));
        assert!(!body.contains(':'), "no label: {body}");
        let mut a = vec!["bundle", "--network", "mainnet", "--template", "bip84"];
        if let Some(l) = lang {
            a.extend_from_slice(&["--language", l]);
        }
        a.extend_from_slice(&["--slot", "@0.phrase=-"]);
        let b = run(&a, Some(&body));
        assert_eq!(b.code, 0, "{lang:?}: {}", b.stderr);
        let want = if lang.is_some() { FP_JA } else { FP };
        assert!(
            b.stderr.contains(&format!("# fingerprint: {want}"))
                || b.stdout.contains(&format!("# fingerprint: {want}")),
            "{lang:?}: {}{}",
            b.stdout,
            b.stderr
        );
    }
}

#[cfg(unix)]
#[test]
fn file_mode_is_0600_under_umask_000() {
    let d = tempfile::tempdir().unwrap();
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let o = std::process::Command::new("sh")
        .arg("-c")
        .arg("umask 000; printf %s \"$1\" | exec \"$0\" convert --from entropy=- --to phrase --phrase-out-dir \"$2\"")
        .arg(&bin)
        .arg(ZERO16)
        .arg(d.path())
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let es = entries(d.path());
    assert_eq!(es.len(), 1, "{es:?}");
    assert_eq!(mode_of(&d.path().join(&es[0])), 0o600);
}

// ------------------------------------------------- existing entry at name

#[cfg(unix)]
#[test]
fn identical_file_is_already_saved_and_untouched() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(save_entropy(&d, &[]).code, 0);
    let es = entries(d.path());
    let f = d.path().join(&es[0]);
    let md0 = fs::metadata(&f).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let r = save_entropy(&d, &["--json"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stderr
            .contains(&format!("already saved: {}", f.display())),
        "{}",
        r.stderr
    );
    let v: Value = serde_json::from_str(&r.stdout).unwrap();
    assert_eq!(v["to"][0]["already_saved"], true);
    let md1 = fs::metadata(&f).unwrap();
    assert_eq!(md0.modified().unwrap(), md1.modified().unwrap());
    assert_eq!(mode_of(&f), 0o600);
    assert_eq!(entries(d.path()), es, "no seed-tmp-* left");
    assert!(!r.stderr.contains("not restricted"), "{}", r.stderr);
}

#[cfg(unix)]
#[test]
fn a_0644_result_gets_the_permissions_warning() {
    // An already-saved file whose mode the filesystem did not restrict stands
    // in for a vfat/SMB mount (the step-5 check reads the handle's mode).
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    assert_eq!(save_entropy(&d, &[]).code, 0);
    let f = d.path().join(&entries(d.path())[0]);
    fs::set_permissions(&f, fs::Permissions::from_mode(0o644)).unwrap();
    let r = save_entropy(&d, &[]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stderr.contains(&format!(
            "warning: file permissions not restricted on this filesystem; protect {}",
            f.display()
        )),
        "{}",
        r.stderr
    );
    assert_eq!(mode_of(&f), 0o644, "never chmodded");
}

fn expect_exists_refusal(d: &tempfile::TempDir, before: &[String]) {
    let r = save_entropy(d, &["--to", "ms1"]);
    assert_eq!(r.code, 1, "{}", r.stderr);
    assert!(
        r.stderr.contains("already exists and is not these words"),
        "{}",
        r.stderr
    );
    assert_eq!(r.stdout, "", "no stdout bytes on a failed write");
    assert_eq!(
        &entries(d.path()),
        before,
        "nothing written, no seed-tmp-* left"
    );
}

/// Today's name for the all-zero English seed (the run's own date).
fn todays_name() -> String {
    format!("seed-{}-{FP}.txt", today())
}

#[test]
fn differing_file_at_the_name_is_refused() {
    let d = tempfile::tempdir().unwrap();
    let n = d.path().join(todays_name());
    fs::write(&n, b"something else\n").unwrap();
    expect_exists_refusal(&d, &entries(d.path()));
    assert_eq!(fs::read(&n).unwrap(), b"something else\n");
}

#[cfg(unix)]
#[test]
fn symlinks_fifo_and_directory_at_the_name_are_refused() {
    use std::os::unix::fs::symlink;
    // dangling symlink
    let d = tempfile::tempdir().unwrap();
    let target = d.path().join("elsewhere.txt");
    symlink(&target, d.path().join(todays_name())).unwrap();
    expect_exists_refusal(&d, &entries(d.path()));
    assert!(!target.exists(), "the link was not followed");
    // live symlink to a file holding the very same words
    let d = tempfile::tempdir().unwrap();
    let target = d.path().join("elsewhere.txt");
    fs::write(&target, format!("{PHRASE12}\n")).unwrap();
    symlink(&target, d.path().join(todays_name())).unwrap();
    expect_exists_refusal(&d, &entries(d.path()));
    // FIFO (never opened: a blocking open would hang this test)
    let d = tempfile::tempdir().unwrap();
    let c = std::ffi::CString::new(d.path().join(todays_name()).to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
    expect_exists_refusal(&d, &entries(d.path()));
    // directory
    let d = tempfile::tempdir().unwrap();
    fs::create_dir(d.path().join(todays_name())).unwrap();
    expect_exists_refusal(&d, &entries(d.path()));
}

#[test]
fn stale_copies_and_leftover_temps_are_noted() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join(format!("seed-2000-01-01-{FP}.txt")), b"x").unwrap();
    fs::write(d.path().join("seed-tmp-1-0.txt"), b"x").unwrap();
    let r = save_entropy(&d, &[]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stderr
            .contains(&format!("seed-2000-01-01-{FP}.txt already holds this seed")),
        "{}",
        r.stderr
    );
    assert!(
        r.stderr
            .contains("leftover temporary copy of a seed exists"),
        "{}",
        r.stderr
    );
    assert!(r.stderr.contains("seed-tmp-1-0.txt"), "{}", r.stderr);
}

// ------------------------------------------------------------------ DIR

#[test]
fn dir_missing_a_file_or_dash_is_exit_1_and_nothing_created() {
    let d = tempfile::tempdir().unwrap();
    let missing = d.path().join("nope");
    let file = d.path().join("afile");
    fs::write(&file, b"").unwrap();
    for (dir, want) in [
        (missing.to_str().unwrap(), "no such directory"),
        (file.to_str().unwrap(), "not a directory"),
        ("-", "stdout is not accepted"),
    ] {
        let r = run(
            &[
                "convert",
                "--from",
                "entropy=-",
                "--to",
                "phrase",
                "--phrase-out-dir",
                dir,
            ],
            Some(ZERO16),
        );
        assert_eq!(r.code, 1, "{dir}: {}", r.stderr);
        assert!(
            r.stderr.contains("--phrase-out-dir") && r.stderr.contains(want),
            "{}",
            r.stderr
        );
        assert_eq!(r.stdout, "");
    }
    assert!(!missing.exists());
    assert_eq!(entries(d.path()), vec!["afile".to_string()]);
}

#[cfg(unix)]
#[test]
fn world_writable_dir_warns() {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    fs::set_permissions(d.path(), fs::Permissions::from_mode(0o777)).unwrap();
    let r = save_entropy(&d, &[]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stderr.contains("is writable by other users"),
        "{}",
        r.stderr
    );
}

#[test]
fn phrase_out_dir_without_phrase_target_is_exit_1() {
    let d = tempfile::tempdir().unwrap();
    let dir = dir_arg(&d);
    let r = run(
        &[
            "convert",
            "--from",
            "entropy=-",
            "--to",
            "ms1",
            "--phrase-out-dir",
            &dir,
        ],
        Some(ZERO16),
    );
    assert_eq!(r.code, 1, "{}", r.stderr);
    assert!(
        r.stderr.contains("--phrase-out-dir needs --to phrase"),
        "{}",
        r.stderr
    );
    assert!(entries(d.path()).is_empty());
}

// --------------------------------------------------------------- stdout

#[test]
fn phrase_twice_writes_one_file_and_prints_nothing() {
    let d = tempfile::tempdir().unwrap();
    let r = save_entropy(&d, &["--to", "phrase"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(r.stdout, "");
    assert_eq!(entries(d.path()).len(), 1);
    // The only secret output went to the file: no stdout private-material warning.
    assert!(
        !r.stderr.contains("stdout carries private key material"),
        "{}",
        r.stderr
    );
}

#[test]
fn phrase_and_grouped_ms1_file_ungrouped_card_grouped() {
    let d = tempfile::tempdir().unwrap();
    let r = save_entropy(&d, &["--to", "ms1", "--group-size", "5"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let card = r.stdout.trim().strip_prefix("ms1: ").unwrap();
    assert!(card.contains(' '), "grouped: {card}");
    assert_eq!(card.replace(' ', ""), MS1_ZERO16);
    let body = fs::read_to_string(d.path().join(&entries(d.path())[0])).unwrap();
    assert_eq!(body, format!("{PHRASE12}\n"));
    for w in ["abandon", "about"] {
        assert!(
            !r.stdout.contains(w),
            "stdout carries no word: {}",
            r.stdout
        );
    }
}

#[test]
fn json_under_the_flag_is_schema_2_with_path_and_no_value() {
    let d = tempfile::tempdir().unwrap();
    let r = save_entropy(&d, &["--to", "ms1", "--to", "phrase", "--json"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stdout.contains("abandon"));
    let v: Value = serde_json::from_str(&r.stdout).unwrap();
    assert_eq!(v["schema_version"], "2");
    let to = v["to"].as_array().unwrap();
    // `--to phrase --to ms1 --to phrase`: one phrase entry, at the first
    // phrase target's position.
    assert_eq!(to.len(), 2, "{to:?}");
    let f = d.path().join(&entries(d.path())[0]);
    assert_eq!(
        to[0],
        serde_json::json!({"node": "phrase", "path": f.to_str().unwrap(), "already_saved": false})
    );
    assert_eq!(to[1]["node"], "ms1");
    assert_eq!(to[1]["value"], MS1_ZERO16);
    // Target order is kept when the phrase is not first.
    let d = tempfile::tempdir().unwrap();
    let dir = dir_arg(&d);
    let r = run(
        &[
            "convert",
            "--from",
            "entropy=-",
            "--to",
            "ms1,phrase",
            "--phrase-out-dir",
            &dir,
            "--json",
        ],
        Some(ZERO16),
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
    let v: Value = serde_json::from_str(&r.stdout).unwrap();
    assert_eq!(v["to"][0]["node"], "ms1");
    assert_eq!(v["to"][1]["node"], "phrase");
    assert!(v["to"][1].get("value").is_none());
}

#[test]
fn json_without_the_flag_is_byte_identical_schema_1() {
    let r = run(
        &["convert", "--from", "entropy=-", "--to", "phrase", "--json"],
        Some(ZERO16),
    );
    assert_eq!(r.code, 0);
    assert_eq!(
        r.stdout,
        format!("{{\"schema_version\":\"1\",\"from_node\":\"entropy\",\"to\":[{{\"node\":\"phrase\",\"value\":\"{PHRASE12}\"}}]}}\n")
    );
}

// --------------------------------------------------------------- repair

#[test]
fn ms1_decode_failure_on_the_repair_path_writes_no_file() {
    // One substitution in the data part. Under a forced TTY the auto-repair
    // path runs; an ms1 correction is a demoted candidate, so the original
    // decode error surfaces (exit 1, not 5). Either way: no file.
    let mut bad: Vec<char> = MS1_ZERO16.chars().collect();
    bad[20] = if bad[20] == 'q' { 'p' } else { 'q' };
    let bad: String = bad.into_iter().collect();
    let d = tempfile::tempdir().unwrap();
    let dir = dir_arg(&d);
    let mut c = mn();
    c.env("MNEMONIC_FORCE_TTY", "1").args([
        "convert",
        "--from",
        "ms1=-",
        "--to",
        "phrase",
        "--phrase-out-dir",
        &dir,
    ]);
    let o = c.write_stdin(bad).output().unwrap();
    assert_ne!(o.status.code(), Some(0));
    assert!(entries(d.path()).is_empty());
}

// ----------------------------------------------------------- F-579

#[test]
fn f579_english_french_ambiguous_phrase_does_not_panic() {
    for from in ["phrase=-"] {
        let r = run(
            &["convert", "--from", from, "--to", "entropy"],
            Some(AMBIGUOUS),
        );
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert_eq!(r.stdout.trim(), format!("entropy: {AMBIGUOUS_ENTROPY}"));
    }
    // seedqr of the same phrase: 23 × 0000 then surface (index 1745).
    let seedqr = format!("{}1745", "0000".repeat(23));
    let r = run(
        &["convert", "--from", "seedqr=-", "--to", "entropy"],
        Some(&seedqr),
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(r.stdout.trim(), format!("entropy: {AMBIGUOUS_ENTROPY}"));
    // And it saves.
    let d = tempfile::tempdir().unwrap();
    let dir = dir_arg(&d);
    let r = run(
        &[
            "convert",
            "--from",
            "phrase=-",
            "--to",
            "phrase",
            "--phrase-out-dir",
            &dir,
        ],
        Some(AMBIGUOUS),
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
    let body = fs::read_to_string(d.path().join(&entries(d.path())[0])).unwrap();
    assert_eq!(body, format!("{AMBIGUOUS}\n"));
}

// ------------------------------------------------------------- help

#[test]
fn help_names_both_flags_and_the_passphrase_checker_risk() {
    let r = run(&["convert", "--help"], None);
    assert_eq!(r.code, 0);
    assert!(r.stdout.contains("--phrase-out-dir <DIR>"));
    assert!(r.stdout.contains("--name-with-passphrase-fp"));
    assert!(r.stdout.contains("offline passphrase checker"));
    assert!(r.stdout.contains("plaintext seed"));
}

/// ms-cli's reader, emulated (no toolkit test runs `ms`): split on
/// whitespace, join with one space, `parse_in` (ms-cli `parse.rs`).
#[test]
fn ms_cli_reader_emulation_accepts_the_file() {
    for (extra, lang) in [
        (&[][..], bip39::Language::English),
        (&["--language", "japanese"][..], bip39::Language::Japanese),
    ] {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(save_entropy(&d, extra).code, 0);
        let body = fs::read_to_string(d.path().join(&entries(d.path())[0])).unwrap();
        let joined = body.split_whitespace().collect::<Vec<_>>().join(" ");
        let m = bip39::Mnemonic::parse_in(lang, joined.as_str()).unwrap();
        assert_eq!(m.word_count(), 12);
    }
}
