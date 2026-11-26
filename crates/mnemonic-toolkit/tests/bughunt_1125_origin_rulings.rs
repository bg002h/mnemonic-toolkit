//! Brian's rulings of 2026-11-26 15:42Z on two bug-hunt decisions.
//!
//! - `tk-bh1125-decision-descriptor-vs-template-multisig`: whenever `bundle`
//!   records an origin path the user did not supply, it says so, exits 4
//!   ("verify before trusting") and prints the command that states the path
//!   explicitly. The cards are written in full, exactly as before.
//! - `tk-bh1125-r1-m2-no-seed-depth0-render`: `restore --md1` without a seed
//!   for a key whose card origin is empty refuses (exit 2) instead of printing
//!   a depth-0 `[fp]xpub`. The user may assert the path (`--origin @N=m/...`,
//!   or a single-sig scheme for a single-key card); restore then checks it
//!   against any xpub header it has, prints it as asserted, and exits 4.

use assert_cmd::Command;

const A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const B: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const FP_A: &str = "73c5da0a";
const FP_B: &str = "b8688df1";

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_env(args: &[&str], stdin: Option<&str>, env: &[(&str, &str)]) -> Out {
    let mut c = Command::cargo_bin("mnemonic").unwrap();
    c.arg("--allow-argv-secret").args(args);
    for (k, v) in env {
        c.env(k, v);
    }
    if let Some(s) = stdin {
        c.write_stdin(s.as_bytes().to_vec());
    }
    let o = c.output().unwrap();
    Out {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

fn run(args: &[&str]) -> Out {
    run_env(args, None, &[])
}

/// Run a command line printed on stderr. It starts with `mnemonic`, which is
/// replaced by the binary under test; `sh` does the quoting.
fn run_printed(cmd: &str, stdin: Option<&str>, env: &[(&str, &str)]) -> Out {
    let rest = cmd
        .strip_prefix("mnemonic ")
        .unwrap_or_else(|| panic!("printed command starts with `mnemonic `: {cmd}"));
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let mut c = std::process::Command::new("sh");
    c.arg("-c")
        .arg(format!("'{}' {rest}", bin.display()))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for (k, v) in env {
        c.env(k, v);
    }
    let mut child = c.spawn().unwrap();
    {
        use std::io::Write;
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

/// The line after the one containing `marker`, trimmed.
fn line_after(stderr: &str, marker: &str) -> String {
    let lines: Vec<&str> = stderr.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.contains(marker))
        .unwrap_or_else(|| panic!("no line with {marker:?}: {stderr}"));
    lines[at + 1].trim().to_string()
}

fn xpub_at(phrase: &str, path: &str) -> String {
    let o = run(&[
        "convert",
        "--network",
        "mainnet",
        "--from",
        &format!("phrase={phrase}"),
        "--to",
        "xpub",
        "--path",
        path,
    ]);
    assert_eq!(o.code, 0, "convert: {}", o.stderr);
    o.stdout
        .split_whitespace()
        .find(|w| w.starts_with("xpub") && w.len() > 100)
        .expect("xpub")
        .to_string()
}

fn md1_of(json: &str) -> Vec<String> {
    let v: serde_json::Value = serde_json::from_str(json).expect("bundle --json");
    match &v["md1"] {
        serde_json::Value::Array(a) => a.iter().map(|s| s.as_str().unwrap().to_string()).collect(),
        serde_json::Value::String(s) => vec![s.clone()],
        other => panic!("md1 field: {other:?}"),
    }
}

const EXPLICIT: &str = "to state this path explicitly, run:";

/// A chosen-origin bundle: exit 4, the JSON on stdout, a verify warning, and
/// a printed command that, run as printed, mints the same cards with exit 0.
fn assert_chosen_origin(o: &Out, idx_list: &str, path: &str, ctx: &str) -> String {
    assert_eq!(
        o.code, 4,
        "{ctx}: exit 4 (verify before trusting): {}",
        o.stderr
    );
    assert!(
        !md1_of(&o.stdout).is_empty(),
        "{ctx}: cards written in full"
    );
    for want in [
        format!("warning: no origin was given for {idx_list}"),
        format!("chose {path}"),
        "Verify that your wallet keeps these keys there".to_string(),
    ] {
        assert!(
            o.stderr.contains(&want),
            "{ctx}: missing {want:?}: {}",
            o.stderr
        );
    }
    assert!(
        !o.stderr.contains("info: no origin"),
        "{ctx}: the old info notice is gone: {}",
        o.stderr
    );
    line_after(&o.stderr, EXPLICIT)
}

// ---------------------------------------------------------------------------
// Item 2: bundle, descriptor mode, family default.
// ---------------------------------------------------------------------------

#[test]
fn ruling_descriptor_default_path_exits_4_and_prints_a_runnable_command() {
    let xb = xpub_at(B, "m/48'/0'/0'/2'");
    let s1 = format!("@1.xpub={xb}");
    let s1fp = format!("@1.fingerprint={FP_B}");
    let desc = "wsh(and_v(v:pk(@0/<0;1>/*),pk(@1/<0;1>/*)))";
    let args = [
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        desc,
        "--slot",
        "@0.phrase=-",
        "--slot",
        &s1,
        "--slot",
        &s1fp,
        "--json",
    ];
    let o = run_env(&args, Some(A), &[]);
    let cmd = assert_chosen_origin(&o, "@0,@1", "m/48'/0'/0'/2'", "wsh default");
    assert!(
        cmd.ends_with("--slot '@0.path=m/48'\\''/0'\\''/0'\\''/2'\\''' --slot '@1.path=m/48'\\''/0'\\''/0'\\''/2'\\'''"),
        "{cmd}"
    );
    let again = run_printed(&cmd, Some(A), &[]);
    assert_eq!(again.code, 0, "explicit path: no warning: {}", again.stderr);
    assert!(
        !again.stderr.contains("no origin was given"),
        "{}",
        again.stderr
    );
    assert_eq!(md1_of(&again.stdout), md1_of(&o.stdout), "the same cards");
}

#[test]
fn ruling_descriptor_canonical_fill_exits_4_and_states_it_inline_for_seed_slots() {
    // Canonical shape + seed slots: `--slot @N.path=` is refused beside a seed
    // there, so the command writes `[fp/path]@N` into the descriptor.
    let desc = "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))";
    let env = [("SEED_A", A), ("SEED_B", B)];
    let args = [
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        desc,
        "--slot",
        "@0.phrase=@env:SEED_A",
        "--slot",
        "@1.phrase=@env:SEED_B",
        "--json",
    ];
    let o = run_env(&args, None, &env);
    let cmd = assert_chosen_origin(&o, "@0,@1", "m/48'/0'/0'/2'", "canonical wsh");
    assert!(o.stderr.contains("canonical origin"), "{}", o.stderr);
    assert!(
        cmd.contains(&format!("[{FP_A}/48'\\''/0'\\''/0'\\''/2'\\'']@0"))
            && cmd.contains(&format!("[{FP_B}/48'\\''/0'\\''/0'\\''/2'\\'']@1")),
        "{cmd}"
    );
    assert!(!cmd.contains("@0.path="), "refused beside a seed: {cmd}");
    assert!(
        cmd.contains("@0.phrase=@env:SEED_A"),
        "env reference kept: {cmd}"
    );
    let again = run_printed(&cmd, None, &env);
    assert_eq!(again.code, 0, "{}", again.stderr);
    assert_eq!(md1_of(&again.stdout), md1_of(&o.stdout), "the same cards");
}

#[test]
fn ruling_descriptor_canonical_single_key_entropy_slot_is_stated_inline() {
    let desc = "wpkh(@0/<0;1>/*)";
    let ent = "@0.entropy=00000000000000000000000000000000";
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        desc,
        "--slot",
        ent,
        "--json",
    ]);
    let cmd = assert_chosen_origin(&o, "@0", "m/84'/0'/0'", "wpkh entropy");
    assert!(
        cmd.contains(&format!("'wpkh([{FP_A}/84'\\''/0'\\''/0'\\'']@0/<0;1>/*)'")),
        "{cmd}"
    );
    // The inline entropy is a secret: never echoed.
    assert!(!cmd.contains("00000000000000000000000000000000"), "{cmd}");
    assert!(o.stderr.contains("not repeated above"), "{}", o.stderr);
}

#[test]
fn ruling_descriptor_file_names_the_inline_form_to_write() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("d.txt");
    std::fs::write(&f, "wpkh(@0/<0;1>/*)\n").unwrap();
    let o = run_env(
        &[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor-file",
            f.to_str().unwrap(),
            "--slot",
            "@0.phrase=-",
            "--json",
        ],
        Some(A),
        &[],
    );
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(
        o.stderr.contains(&format!("[{FP_A}/84'/0'/0']@0")),
        "names the inline form for the file: {}",
        o.stderr
    );
}

#[test]
fn ruling_bare_sh_keeps_one_warning_an_explicit_command_and_the_m45_alternative() {
    let desc = "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))";
    let o = run_env(
        &[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            "@0.phrase=-",
            "--slot",
            &format!("@1.phrase={B}"),
            "--json",
        ],
        Some(A),
        &[],
    );
    let cmd = assert_chosen_origin(&o, "@0,@1", "m/87'/0'/0'", "bare sh");
    assert!(o.stderr.contains("legacy P2SH"), "{}", o.stderr);
    assert!(
        cmd.contains("@1.phrase=<your @1.phrase>"),
        "redacted: {cmd}"
    );
    assert!(!cmd.contains("legal"), "{cmd}");
    let alt = line_after(&o.stderr, "to mint this bundle at m/45' instead, run:");
    assert!(alt.contains("@0.path=m/45'"), "{alt}");
    assert_eq!(
        o.stderr.matches("no origin was given").count(),
        1,
        "one verify warning: {}",
        o.stderr
    );
    assert_eq!(
        o.stderr.matches("not repeated above").count(),
        1,
        "one redaction note: {}",
        o.stderr
    );
}

// ---------------------------------------------------------------------------
// Item 2: bundle, template mode.
// ---------------------------------------------------------------------------

#[test]
fn ruling_multisig_template_without_family_exits_4_and_names_the_flag() {
    let env = [("SEED_A", A), ("SEED_B", B)];
    let args = [
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--slot",
        "@0.phrase=@env:SEED_A",
        "--slot",
        "@1.phrase=@env:SEED_B",
        "--json",
    ];
    let o = run_env(&args, None, &env);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(!md1_of(&o.stdout).is_empty());
    for want in [
        "warning: --multisig-path-family was not given",
        "BIP-87 (m/87'/0'/0')",
        "Verify that your wallet keeps these keys there",
    ] {
        assert!(o.stderr.contains(want), "missing {want:?}: {}", o.stderr);
    }
    let cmd = line_after(&o.stderr, "to state this choice explicitly, run:");
    assert!(cmd.ends_with("--multisig-path-family bip87"), "{cmd}");
    let again = run_printed(&cmd, None, &env);
    assert_eq!(again.code, 0, "{}", again.stderr);
    assert_eq!(md1_of(&again.stdout), md1_of(&o.stdout), "the same cards");
}

#[test]
fn ruling_named_scheme_or_typed_path_is_not_a_chosen_origin() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    for args in [
        vec!["--template", "bip84", "--slot", &s0],
        vec![
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--slot",
            &s0,
            "--slot",
            &s1,
        ],
        vec![
            "--descriptor",
            "wpkh([73c5da0a/84'/0'/0']@0/<0;1>/*)",
            "--slot",
            &s0,
        ],
    ] {
        let mut full = vec!["bundle", "--network", "mainnet", "--json"];
        full.extend(args.iter().copied());
        let o = run(&full);
        assert_eq!(o.code, 0, "{args:?}: {}", o.stderr);
        assert!(!o.stderr.contains("no origin was given"), "{}", o.stderr);
        assert!(!o.stderr.contains("was not given"), "{}", o.stderr);
    }
}

// ---------------------------------------------------------------------------
// Item 3: restore --md1 without a seed for an empty-origin key.
// ---------------------------------------------------------------------------

/// Re-split `md1` with every origin EMPTIED (a keyed card as the old mint
/// produced it).
fn with_empty_origins(md1: &[String]) -> Vec<String> {
    use md_codec::origin_path::{OriginPath, PathDeclPaths};
    let refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let mut d = md_codec::chunk::reassemble(&refs).expect("md1 decodes");
    d.path_decl.paths = PathDeclPaths::Shared(OriginPath {
        components: Vec::new(),
    });
    md_codec::chunk::split(&d).expect("re-split")
}

fn minted(desc: &str, slots: &[&str]) -> Vec<String> {
    let mut args = vec![
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        desc,
        "--json",
    ];
    for s in slots {
        args.extend(["--slot", s]);
    }
    let o = run(&args);
    assert!(o.code == 0 || o.code == 4, "{}", o.stderr);
    md1_of(&o.stdout)
}

fn empty_origin_multisig() -> Vec<String> {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    with_empty_origins(&minted(
        "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
        &[&s0, &s1],
    ))
}

fn empty_origin_wpkh() -> Vec<String> {
    let s0 = format!("@0.phrase={A}");
    with_empty_origins(&minted("wpkh(@0/<0;1>/*)", &[&s0]))
}

fn restore(md1: &[String], extra: &[&str]) -> Out {
    let mut args: Vec<String> = vec!["restore".into()];
    for s in md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }
    args.extend(extra.iter().map(|s| s.to_string()));
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run(&refs)
}

const ASSERTED: &str = "asserted by you, not on the card";

#[test]
fn ruling_restore_without_seed_refuses_an_empty_origin_key() {
    let md1 = empty_origin_multisig();
    let r = restore(&md1, &[]);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stdout.is_empty(), "nothing rendered: {}", r.stdout);
    for want in [
        "@0, @1",
        "carries NO origin path on the card",
        "without its seed",
        "--origin '@0=m/<path>'",
        "--origin '@1=m/<path>'",
    ] {
        assert!(r.stderr.contains(want), "missing {want:?}: {}", r.stderr);
    }
    // r3 M1: no concrete example path that could be wrong.
    assert!(!r.stderr.contains("for example"), "{}", r.stderr);
    assert!(!r.stderr.contains("48'"), "{}", r.stderr);
    let cmd = line_after(&r.stderr, "state each path you know");
    assert!(cmd.starts_with("mnemonic "), "{cmd}");
}

#[test]
fn ruling_restore_asserted_origin_renders_it_and_exits_4() {
    let md1 = empty_origin_multisig();
    let r = restore(
        &md1,
        &[
            "--origin",
            "@0=m/48'/0'/0'/2'",
            "--origin",
            "@1=m/48'/0'/0'/2'",
        ],
    );
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(
        r.stdout.contains(&format!("[{FP_A}/48'/0'/0'/2']"))
            && r.stdout.contains(&format!("[{FP_B}/48'/0'/0'/2']")),
        "{}",
        r.stdout
    );
    assert!(
        !r.stdout.contains("xpub661My"),
        "no depth-0 key: {}",
        r.stdout
    );
    for want in [ASSERTED, "cannot confirm the purpose or coin"] {
        assert!(r.stderr.contains(want), "missing {want:?}: {}", r.stderr);
    }
    assert!(!r.stderr.contains("Its key is shown"), "{}", r.stderr);
}

#[test]
fn ruling_restore_asserted_origin_is_checked_against_a_supplied_xpub_header() {
    let md1 = empty_origin_multisig();
    let xb = xpub_at(B, "m/48'/0'/0'/2'");
    let cos = format!("@1={xb}");
    // Depth 3 path against a depth-4 key.
    let r = restore(
        &md1,
        &[
            "--origin",
            "@0=m/48'/0'/0'/2'",
            "--origin",
            "@1=m/87'/0'/0'",
            "--cosigner",
            &cos,
        ],
    );
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stdout.is_empty());
    assert!(
        !r.stderr.contains(ASSERTED),
        "no warning before a refusal: {}",
        r.stderr
    );
    assert!(
        r.stderr.contains("m/87'/0'/0'") && r.stderr.contains("depth 4"),
        "names both: {}",
        r.stderr
    );
    // Right depth, wrong last component.
    let r = restore(
        &md1,
        &[
            "--origin",
            "@0=m/48'/0'/0'/2'",
            "--origin",
            "@1=m/48'/0'/0'/1'",
            "--cosigner",
            &cos,
        ],
    );
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stderr.contains("2'"), "{}", r.stderr);
    // The right path: depth and child number agree; still exit 4.
    let r = restore(
        &md1,
        &[
            "--origin",
            "@0=m/48'/0'/0'/2'",
            "--origin",
            "@1=m/48'/0'/0'/2'",
            "--cosigner",
            &cos,
        ],
    );
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(
        r.stderr.contains("depth and child number agree"),
        "{}",
        r.stderr
    );
}

#[test]
fn ruling_restore_with_own_seed_still_refuses_the_foreign_empty_origin_key() {
    let md1 = empty_origin_multisig();
    let from = format!("phrase={A}");
    let r = restore(&md1, &["--from", &from]);
    assert_eq!(r.code, 2, "{}", r.stderr);
    // The seed's own key was found by the (unchanged) search; only @1 is refused.
    assert!(r.stderr.contains("Its key was FOUND"), "{}", r.stderr);
    assert!(
        r.stderr.contains("@1 carries NO origin path"),
        "{}",
        r.stderr
    );
    assert!(!r.stderr.contains("@0, @1"), "{}", r.stderr);
    let cmd = line_after(&r.stderr, "state each path you know");
    assert!(!cmd.contains("abandon"), "seed never echoed: {cmd}");
    assert!(cmd.contains("phrase=<your phrase>"), "{cmd}");
    assert!(cmd.ends_with("--origin '@1=m/<path>'"), "{cmd}");

    let r = restore(&md1, &["--from", &from, "--origin", "@1=m/48'/0'/0'/2'"]);
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(
        r.stdout.contains(&format!("[{FP_B}/48'/0'/0'/2']")),
        "{}",
        r.stdout
    );
}

#[test]
fn ruling_restore_single_key_card_accepts_a_named_scheme_or_a_path() {
    let md1 = empty_origin_wpkh();
    let r = restore(&md1, &[]);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(
        r.stderr.contains("--template"),
        "names the scheme form: {}",
        r.stderr
    );

    for extra in [
        vec!["--template", "bip84"],
        vec!["--origin", "m/84'/0'/0'"],
        vec!["--origin", "@0=m/84'/0'/0'"],
    ] {
        let r = restore(&md1, &extra);
        assert_eq!(r.code, 4, "{extra:?}: {}", r.stderr);
        assert!(
            r.stdout.contains(&format!("[{FP_A}/84'/0'/0']")),
            "{extra:?}: {}",
            r.stdout
        );
        assert!(r.stderr.contains(ASSERTED), "{}", r.stderr);
        // BIP-84 vector: first receive of A at m/84'/0'/0'/0/0.
        assert!(
            r.stdout
                .contains("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"),
            "{}",
            r.stdout
        );
    }
    // A scheme of another script type does not describe this card.
    let r = restore(&md1, &["--template", "bip44"]);
    assert_eq!(r.code, 2, "{}", r.stderr);
}

#[test]
fn ruling_restore_origin_for_a_key_with_a_card_origin_is_refused() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let md1 = minted("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))", &[&s0, &s1]);
    let r = restore(&md1, &["--origin", "@1=m/48'/0'/0'/2'"]);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stderr.contains("@1"), "{}", r.stderr);
    // Unchanged without it.
    let r = restore(&md1, &[]);
    assert_eq!(r.code, 0, "{}", r.stderr);
}
