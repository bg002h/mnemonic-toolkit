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
        r.stderr
            .contains("depth and child number this path implies")
            && r.stderr.contains("not the steps before it"),
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

// ---------------------------------------------------------------------------
// Review r1 (bughunt-2026-11-25-mnemonic-toolkit-review-rulings-r1) I1, I2.
// ---------------------------------------------------------------------------

/// The empty-origin 2-of-2 card plus the mk1 chunks the same bundle wrote for
/// each slot (their origins are m/48'/0'/0'/2', fingerprints recorded).
fn empty_origin_multisig_with_mk1(privacy: bool) -> (Vec<String>, Vec<Vec<String>>) {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let mut args = vec![
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
        "--json",
        "--slot",
        &s0,
        "--slot",
        &s1,
    ];
    if privacy {
        args.push("--privacy-preserving");
    }
    let o = run(&args);
    assert!(o.code == 0 || o.code == 4, "{}", o.stderr);
    let v: serde_json::Value = serde_json::from_str(&o.stdout).unwrap();
    let mk1 = v["mk1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|slot| {
            slot.as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_string())
                .collect()
        })
        .collect();
    (with_empty_origins(&md1_of(&o.stdout)), mk1)
}

/// `--cosigner @N=<chunk>` once per chunk.
fn cosigner_args(n: u8, chunks: &[String]) -> Vec<String> {
    chunks
        .iter()
        .flat_map(|c| ["--cosigner".to_string(), format!("@{n}={c}")])
        .collect()
}

fn restore_s(md1: &[String], extra: &[String]) -> Out {
    let refs: Vec<&str> = extra.iter().map(|s| s.as_str()).collect();
    restore(md1, &refs)
}

fn json_keys(o: &Out) -> (serde_json::Value, Vec<serde_json::Value>) {
    let v: serde_json::Value = serde_json::from_str(&o.stdout).expect("restore --json");
    let keys = v["wallets"][0]["cosigner_keys"].as_array().unwrap().clone();
    (v, keys)
}

#[test]
fn review_r1_i1_mk1_cosigner_supplies_its_recorded_origin() {
    let (md1, mk1) = empty_origin_multisig_with_mk1(false);
    // Both keys' mk1 cards, no --origin: the paths are on cards; exit 0.
    let mut extra = cosigner_args(0, &mk1[0]);
    extra.extend(cosigner_args(1, &mk1[1]));
    let r = restore_s(&md1, &extra);
    assert_eq!(r.code, 0, "{}", r.stderr);
    for fp in [FP_A, FP_B] {
        assert!(
            r.stdout.contains(&format!("[{fp}/48'/0'/0'/2']")),
            "{}",
            r.stdout
        );
    }
    assert!(!r.stderr.contains(ASSERTED), "{}", r.stderr);
    assert!(
        r.stdout.contains("origin from its mk1 card"),
        "{}",
        r.stdout
    );

    extra.push("--json".into());
    let r = restore_s(&md1, &extra);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (v, keys) = json_keys(&r);
    assert_eq!(v["verification"]["status"], "verified", "{}", r.stdout);
    for k in &keys {
        assert_eq!(k["origin_source"], "mk1", "{k}");
        assert_eq!(k["origin"], "48'/0'/0'/2'", "{k}");
    }

    // One mk1 (@1), @0 asserted: @1 is the mk1's path, @0 asserted; exit 4.
    let mut extra = vec!["--origin".to_string(), "@0=m/48'/0'/0'/2'".to_string()];
    extra.extend(cosigner_args(1, &mk1[1]));
    let r = restore_s(&md1, &extra);
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(
        r.stderr.contains("md1 cosigner @0's origin"),
        "{}",
        r.stderr
    );
    assert!(!r.stderr.contains("(--origin @1)"), "{}", r.stderr);
    assert!(r.stderr.contains("read from the mk1 card"), "{}", r.stderr);
}

#[test]
fn review_r1_i1_origin_contradicting_the_mk1_card_is_refused() {
    let (md1, mk1) = empty_origin_multisig_with_mk1(false);
    // The reviewer's repro: a same-length path ending in the same component.
    let mut extra = vec![
        "--origin".to_string(),
        "@0=m/48'/0'/0'/2'".to_string(),
        "--origin".to_string(),
        "@1=m/86'/0'/0'/2'".to_string(),
    ];
    extra.extend(cosigner_args(1, &mk1[1]));
    extra.push("--json".into());
    let r = restore_s(&md1, &extra);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert!(
        r.stderr.contains("m/86'/0'/0'/2'") && r.stderr.contains("m/48'/0'/0'/2'"),
        "names both: {}",
        r.stderr
    );

    // The same path as the mk1 is confirmed by the card: not an assertion.
    let mut extra = vec!["--origin".to_string(), "@1=m/48'/0'/0'/2'".to_string()];
    extra.extend(cosigner_args(0, &mk1[0]));
    extra.extend(cosigner_args(1, &mk1[1]));
    let r = restore_s(&md1, &extra);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stderr.contains(ASSERTED), "{}", r.stderr);
}

#[test]
fn review_r1_i1_mk1_without_a_path_falls_back_to_the_header_check() {
    use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
    use std::str::FromStr;
    let (md1, _) = empty_origin_multisig_with_mk1(false);
    let real = Xpub::from_str(&xpub_at(B, "m/48'/0'/0'/2'")).unwrap();
    let bare = Xpub {
        depth: 0,
        child_number: ChildNumber::Normal { index: 0 },
        parent_fingerprint: Fingerprint::default(),
        ..real
    };
    let card = mk_codec::KeyCard::new(
        vec![[0, 0, 0, 0]],
        Some(Fingerprint::from_str(FP_B).unwrap()),
        DerivationPath::master(),
        bare,
    );
    let chunks = mk_codec::encode(&card).expect("encode");
    // No path anywhere: refused as before.
    let r = restore_s(&md1, &{
        let mut e = vec!["--origin".to_string(), "@0=m/48'/0'/0'/2'".to_string()];
        e.extend(cosigner_args(1, &chunks));
        e
    });
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(
        r.stderr.contains("@1 carries NO origin path"),
        "{}",
        r.stderr
    );
    // Asserted: exit 4, and the warning does not claim the card checked it.
    let mut e = vec![
        "--origin".to_string(),
        "@0=m/48'/0'/0'/2'".to_string(),
        "--origin".to_string(),
        "@1=m/48'/0'/0'/2'".to_string(),
    ];
    e.extend(cosigner_args(1, &chunks));
    let r = restore_s(&md1, &e);
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(r.stderr.contains("records no path either"), "{}", r.stderr);
    assert!(!r.stderr.contains("agree"), "{}", r.stderr);
}

#[test]
fn review_r1_i1_mk1_fingerprint_contradicting_the_md1_is_refused() {
    use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
    use std::str::FromStr;
    let (md1, _) = empty_origin_multisig_with_mk1(false);
    let real = Xpub::from_str(&xpub_at(B, "m/48'/0'/0'/2'")).unwrap();
    let card = mk_codec::KeyCard::new(
        vec![[0, 0, 0, 0]],
        Some(Fingerprint::from_str("deadbeef").unwrap()),
        DerivationPath::from_str("m/48'/0'/0'/2'").unwrap(),
        real,
    );
    let chunks = mk_codec::encode(&card).expect("encode");
    let mut e = vec!["--origin".to_string(), "@0=m/48'/0'/0'/2'".to_string()];
    e.extend(cosigner_args(1, &chunks));
    let r = restore_s(&md1, &e);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(
        r.stderr.contains("deadbeef") && r.stderr.contains(FP_B),
        "{}",
        r.stderr
    );
}

#[test]
fn review_r1_i1_privacy_mk1_without_fingerprint_still_supplies_the_path() {
    let (md1, mk1) = empty_origin_multisig_with_mk1(true);
    let mut extra = cosigner_args(0, &mk1[0]);
    extra.extend(cosigner_args(1, &mk1[1]));
    let r = restore_s(&md1, &extra);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains("48'/0'/0'/2'"), "{}", r.stdout);
}

#[test]
fn review_r1_i2_stdout_and_json_mark_an_asserted_origin() {
    let md1 = empty_origin_multisig();
    let xa = xpub_at(A, "m/48'/0'/0'/2'");
    let xb = xpub_at(B, "m/48'/0'/0'/2'");
    let c0 = format!("@0={xa}");
    let c1 = format!("@1={xb}");
    let base = [
        "--origin",
        "@0=m/48'/0'/0'/2'",
        "--origin",
        "@1=m/48'/0'/0'/2'",
        "--cosigner",
        &c0,
        "--cosigner",
        &c1,
    ];
    // Text: beside the origin, never "from md1".
    let r = restore(&md1, &base);
    assert_eq!(r.code, 4, "{}", r.stderr);
    for line in r.stdout.lines().filter(|l| l.contains("cosigner @")) {
        assert!(line.contains(ASSERTED), "{line}");
        assert!(!line.contains("from md1"), "{line}");
    }
    // The header check is not described as more than it is.
    assert!(!r.stderr.contains("agree with the xpub"), "{}", r.stderr);
    // JSON: per-key source; status is not "verified" at exit 4.
    let mut j = base.to_vec();
    j.push("--json");
    let r = restore(&md1, &j);
    assert_eq!(r.code, 4, "{}", r.stderr);
    let (v, keys) = json_keys(&r);
    assert_eq!(v["verification"]["status"], "unverified", "{}", r.stdout);
    for k in &keys {
        assert_eq!(k["origin_source"], "asserted", "{k}");
        assert!(k["note"].as_str().unwrap().contains(ASSERTED), "{k}");
    }
    // An exported wallet file: stderr says the file carries an asserted path.
    let mut f = base.to_vec();
    f.extend(["--format", "sparrow"]);
    let r = restore(&md1, &f);
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(
        r.stderr.contains("the exported wallet file carries"),
        "{}",
        r.stderr
    );
}

#[test]
fn review_r1_i2_card_and_seed_search_origins_are_labelled() {
    // A card origin: origin_source "card".
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let md1 = minted("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))", &[&s0, &s1]);
    let r = restore(&md1, &["--json"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (_, keys) = json_keys(&r);
    assert!(
        keys.iter().all(|k| k["origin_source"] == "card"),
        "{keys:?}"
    );
    // A seed-found origin: "seed-search"; the asserted one "asserted".
    let md1 = empty_origin_multisig();
    let from = format!("phrase={A}");
    let r = restore(
        &md1,
        &["--from", &from, "--origin", "@1=m/48'/0'/0'/2'", "--json"],
    );
    assert_eq!(r.code, 4, "{}", r.stderr);
    let (v, keys) = json_keys(&r);
    assert_eq!(keys[0]["origin_source"], "seed-search", "{:?}", keys);
    assert_eq!(keys[1]["origin_source"], "asserted", "{:?}", keys);
    assert_ne!(v["verification"]["status"], "verified");
}

#[test]
fn review_r1_i1_depth_one_xpub_parent_must_be_the_md1_fingerprint() {
    use bitcoin::bip32::{ChildNumber, Fingerprint, Xpub};
    use std::str::FromStr;
    let md1 = empty_origin_multisig();
    let real = Xpub::from_str(&xpub_at(B, "m/48'/0'/0'/2'")).unwrap();
    let forged = |parent: &str| {
        Xpub {
            depth: 1,
            child_number: ChildNumber::Hardened { index: 2 },
            parent_fingerprint: Fingerprint::from_str(parent).unwrap(),
            ..real
        }
        .to_string()
    };
    let run_with = |x: String| {
        let c = format!("@1={x}");
        restore(
            &md1,
            &[
                "--origin",
                "@0=m/48'/0'/0'/2'",
                "--origin",
                "@1=m/2'",
                "--cosigner",
                &c,
            ],
        )
    };
    let r = run_with(forged("deadbeef"));
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(
        r.stderr.contains("parent fingerprint deadbeef") && r.stderr.contains(FP_B),
        "{}",
        r.stderr
    );
    let r = run_with(forged(FP_B));
    assert_eq!(r.code, 4, "{}", r.stderr);
}

// ---------------------------------------------------------------------------
// Final review (bughunt-2026-11-25-mnemonic-toolkit-review-rulings-fable)
// I-1, I-2, M-1, M-2/M-3.
// ---------------------------------------------------------------------------

/// The mk1 chunks a bundle writes for B's key at m/48'/0'/7'/2': same seed
/// (fingerprint b8688df1) as the md1's @1, another key.
fn mk1_of_b_at_account_7() -> Vec<String> {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']@0/<0;1>/*,[b8688df1/48'/0'/7'/2']@1/<0;1>/*))",
        "--json",
        "--slot",
        &s0,
        "--slot",
        &s1,
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    let v: serde_json::Value = serde_json::from_str(&o.stdout).unwrap();
    v["mk1"][1]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn review_fable_i1_mismatching_mk1_supplies_no_origin_under_allow_mismatch() {
    let (md1, mk1) = empty_origin_multisig_with_mk1(false);
    let mkb7 = mk1_of_b_at_account_7();
    // The reviewer's reproduction: A's real mk1, and an mk1 for another key
    // of B's seed, with --allow-mismatch. The mk1 for another key says
    // nothing about @1's origin, so @1 has none: refused, nothing rendered.
    let mut extra = cosigner_args(0, &mk1[0]);
    extra.extend(cosigner_args(1, &mkb7));
    extra.push("--allow-mismatch".into());
    let mut j = extra.clone();
    j.push("--json".into());
    let r = restore_s(&md1, &j);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert!(!r.stderr.contains("48'/0'/7'/2'"), "{}", r.stderr);
    assert!(
        !r.stderr.contains("the mk1's key is the md1's key"),
        "{}",
        r.stderr
    );
    assert!(r.stderr.contains("@1 carries NO origin"), "{}", r.stderr);

    // With @1's path stated, the wallet is shown at the stated path, the
    // mismatching mk1 is not described as checking it, and @1 is not
    // labelled cross-checked or as having its origin from an mk1.
    j.extend(["--origin".into(), "@1=m/48'/0'/0'/2'".into()]);
    let r = restore_s(&md1, &j);
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(!r.stdout.contains("48'/0'/7'/2'"), "{}", r.stdout);
    assert!(
        !r.stderr.contains("NOTE: md1 cosigner @1's origin"),
        "{}",
        r.stderr
    );
    assert!(
        !r.stderr
            .contains("has the depth and child number this path implies"),
        "{}",
        r.stderr
    );
    assert!(
        r.stderr.contains("is not the md1's key"),
        "says the given key does not match: {}",
        r.stderr
    );
    let (v, keys) = json_keys(&r);
    assert_eq!(v["verification"]["status"], "overridden", "{}", r.stdout);
    assert_eq!(keys[0]["origin_source"], "mk1", "{keys:?}");
    assert_eq!(keys[1]["origin_source"], "asserted", "{keys:?}");
    assert_eq!(keys[1]["origin"], "48'/0'/0'/2'", "{keys:?}");
    assert!(
        !keys[1]["note"].as_str().unwrap().contains("cross-checked"),
        "{keys:?}"
    );
}

/// A single-key `wpkh` card whose key is A's MASTER key and whose origin is
/// empty (what the old descriptor-mode mint wrote for a bare `@0`).
fn master_key_wpkh() -> Vec<String> {
    master_key_card(empty_origin_wpkh())
}

fn master_key_card(md1: Vec<String>) -> Vec<String> {
    use bitcoin::bip32::Xpub;
    use std::str::FromStr;
    let refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let mut d = md_codec::chunk::reassemble(&refs).expect("decode");
    let master = Xpub::from_str(&xpub_at(A, "m")).unwrap();
    let mut k65 = [0u8; 65];
    k65[..32].copy_from_slice(master.chain_code.as_bytes());
    k65[32..].copy_from_slice(&master.public_key.serialize());
    for (idx, key) in d.tlv.pubkeys.as_mut().expect("keyed card").iter_mut() {
        if *idx == 0 {
            *key = k65;
        }
    }
    md_codec::chunk::split(&d).expect("re-split")
}

#[test]
fn review_fable_i2_master_key_card_refuses_a_path_below_m() {
    let md1 = master_key_wpkh();
    for extra in [
        vec!["--template", "bip84"],
        vec!["--origin", "m/84'/0'/0'"],
        vec!["--origin", "@0=m/84'/0'/0'"],
    ] {
        let r = restore(&md1, &extra);
        assert_eq!(r.code, 2, "{extra:?}: {}", r.stderr);
        assert!(r.stdout.is_empty(), "{extra:?}: {}", r.stdout);
        assert!(
            r.stderr.contains("is the master key of 73c5da0a"),
            "{extra:?}: {}",
            r.stderr
        );
        assert!(!r.stderr.contains(ASSERTED), "{extra:?}: {}", r.stderr);
    }
    // Multisig: the master-key @0 refuses a stated path too.
    let md1 = master_key_card(empty_origin_multisig());
    let r = restore(
        &md1,
        &[
            "--origin",
            "@0=m/48'/0'/0'/2'",
            "--origin",
            "@1=m/48'/0'/0'/2'",
        ],
    );
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stderr.contains("is the master key of"), "{}", r.stderr);
}

// ---------------------------------------------------------------------------
// Brian's ruling of 2026-11-26 17:33Z (`tk-bh1125-rf-decision-master-key-auto-m`,
// folds `tk-bh1125-rf-nits` N1, N2, N4, N5): an empty-origin key the card
// proves to be the master key resolves to `m` without a seed or an assertion.
// ---------------------------------------------------------------------------

const MASTER_WARNING: &str = "the wallet sits on the master key";

/// The cosigner line of a text restore for `@idx`.
fn cosigner_line(stdout: &str, idx: u8) -> String {
    stdout
        .lines()
        .find(|l| l.trim_start().starts_with(&format!("cosigner @{idx}:")))
        .unwrap_or_else(|| panic!("no cosigner @{idx} line: {stdout}"))
        .to_string()
}

#[test]
fn ruling_master_key_resolves_to_m_without_a_seed() {
    let md1 = master_key_wpkh();
    let master = xpub_at(A, "m");
    // No --origin: resolved from the card, exit 0 (nothing else unverified
    // beyond the usual no-cross-check banner).
    let r = restore(&md1, &[]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(
        r.stdout.contains(&format!("[{FP_A}]{master}")),
        "{}",
        r.stdout
    );
    let line = cosigner_line(&r.stdout, 0);
    assert!(line.contains(&format!("{FP_A} [m]")), "N1: {line}");
    assert!(!line.contains("[]"), "N1: {line}");
    assert!(!line.contains(ASSERTED), "{line}");
    assert!(line.contains("master key"), "{line}");
    assert!(r.stderr.contains(MASTER_WARNING), "{}", r.stderr);
    for not in [
        ASSERTED,
        "not even its depth",
        "carries NO origin path on the card, and",
    ] {
        assert!(!r.stderr.contains(not), "{not:?}: {}", r.stderr);
    }
    assert!(!r.stderr.contains("not a mismatch"), "{}", r.stderr);
    // JSON: origin_source "master"; status as for any card without a
    // cross-check.
    let r = restore(&md1, &["--json"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (v, keys) = json_keys(&r);
    assert_eq!(keys[0]["origin_source"], "master", "{keys:?}");
    assert_eq!(keys[0]["origin"], "", "{keys:?}");
    assert_eq!(v["verification"]["status"], "unverified", "{}", r.stdout);
    // With the master xpub as a cross-check: verified, exit 0.
    let c = format!("@0={master}");
    let r = restore(&md1, &["--cosigner", &c, "--json"]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    let (v, keys) = json_keys(&r);
    assert_eq!(keys[0]["origin_source"], "master", "{keys:?}");
    assert_eq!(v["verification"]["status"], "verified", "{}", r.stdout);
}

#[test]
fn ruling_master_key_explicit_m_is_consistent_and_exits_0() {
    // N2: an explicit `--origin m` for a proven master key is not an
    // assertion that went unchecked.
    let md1 = master_key_wpkh();
    for extra in [vec!["--origin", "m"], vec!["--origin", "@0=m"]] {
        let mut j = extra.clone();
        j.push("--json");
        let r = restore(&md1, &j);
        assert_eq!(r.code, 0, "{extra:?}: {}", r.stderr);
        let (_, keys) = json_keys(&r);
        assert_eq!(keys[0]["origin_source"], "master", "{extra:?}: {keys:?}");
        let r = restore(&md1, &extra);
        assert_eq!(r.code, 0, "{extra:?}: {}", r.stderr);
        for not in [ASSERTED, "not even its depth", "UNVERIFIED: an origin path"] {
            assert!(!r.stderr.contains(not), "{extra:?} {not:?}: {}", r.stderr);
        }
        assert!(r.stderr.contains(MASTER_WARNING), "{extra:?}: {}", r.stderr);
        assert!(
            cosigner_line(&r.stdout, 0).contains(&format!("{FP_A} [m]")),
            "{extra:?}: {}",
            r.stdout
        );
    }
}

#[test]
fn ruling_master_key_in_a_multisig_resolves_and_only_the_other_key_is_refused() {
    let md1 = master_key_card(empty_origin_multisig());
    let r = restore(&md1, &[]);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stderr.contains("@1 carries NO origin"), "{}", r.stderr);
    assert!(!r.stderr.contains("@0, @1"), "{}", r.stderr);
    let cmd = line_after(&r.stderr, "run:");
    assert!(cmd.contains("'@1=m/<path>'"), "{cmd}");
    assert!(!cmd.contains("@0="), "{cmd}");
    // @1 stated: @0 is the master (exit would be 0), @1 asserted (exit 4).
    let r = restore(&md1, &["--origin", "@1=m/48'/0'/0'/2'", "--json"]);
    assert_eq!(r.code, 4, "{}", r.stderr);
    let (_, keys) = json_keys(&r);
    assert_eq!(keys[0]["origin_source"], "master", "{keys:?}");
    assert_eq!(keys[1]["origin_source"], "asserted", "{keys:?}");
    assert!(
        r.stderr.contains(
            "UNVERIFIED: an origin path is asserted by you, not on the card (@1 m/48'/0'/0'/2')"
        ),
        "{}",
        r.stderr
    );
}

#[test]
fn ruling_master_key_with_a_seed_is_labelled_like_the_seedless_path() {
    // With a seed the search is unchanged (it tries m first) and keeps its
    // source; the printed origin and the master-key warning are the same.
    let md1 = master_key_wpkh();
    let from = format!("phrase={A}");
    let r = restore(&md1, &["--from", &from]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stderr.contains(MASTER_WARNING), "{}", r.stderr);
    let seeded = cosigner_line(&r.stdout, 0);
    assert!(seeded.contains(&format!("{FP_A} [m]")), "N1: {seeded}");
    let seedless = restore(&md1, &[]);
    let desc = |o: &Out| line_after(&format!("x\n{}", o.stdout), "x");
    assert_eq!(desc(&r), desc(&seedless), "same descriptor");
    let r = restore(&md1, &["--from", &from, "--json"]);
    let (v, keys) = json_keys(&r);
    assert_eq!(keys[0]["origin_source"], "seed-search", "{keys:?}");
    assert_eq!(keys[0]["origin"], "", "{keys:?}");
    assert_eq!(v["verification"]["status"], "verified", "{}", r.stdout);
}

#[test]
fn ruling_master_check_for_a_non_master_key_prints_m_not_m_slash() {
    // N1: the refusal of `m` for a key that is not the master key.
    let md1 = empty_origin_wpkh();
    let r = restore(&md1, &["--origin", "m"]);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(
        r.stderr.contains("states m for md1 cosigner @0"),
        "{}",
        r.stderr
    );
    assert!(!r.stderr.contains("m/ "), "{}", r.stderr);
}

#[test]
fn ruling_origin_refusals_exit_2_and_an_empty_value_is_refused() {
    let md1 = empty_origin_multisig();
    // N4: an empty value (an unset shell variable) is not `m`.
    for (v, other) in [("@0=", "@1=m/48'/0'/0'/2'"), ("@1= ", "@0=m/48'/0'/0'/2'")] {
        let r = restore(&md1, &["--origin", v, "--origin", other]);
        assert_eq!(r.code, 2, "{v:?}: {}", r.stderr);
        assert!(r.stderr.contains("empty"), "{v:?}: {}", r.stderr);
    }
    let single = empty_origin_wpkh();
    let r = restore(&single, &["--origin", ""]);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stderr.contains("empty"), "{}", r.stderr);
    // N5: every --origin refusal exits 2.
    for extra in [
        vec!["--origin", "@9=m/0"],
        vec!["--origin", "@0=m/x'"],
        vec!["--origin", "@x=m/0"],
        vec!["--origin", "@0"],
        vec!["--origin", "m/48'/0'/0'/2'"],
        vec![
            "--origin",
            "@0=m/48'/0'/0'/2'",
            "--origin",
            "@0=m/48'/0'/0'/2'",
        ],
    ] {
        let r = restore(&md1, &extra);
        assert_eq!(r.code, 2, "{extra:?}: {}", r.stderr);
        assert!(r.stdout.is_empty(), "{extra:?}: {}", r.stdout);
    }
}

#[test]
fn review_fable_m1_header_refusal_prints_no_example_path() {
    let x = xpub_at(A, "m/84'/0'/5'");
    let s = format!("@0.xpub={x}");
    let r = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &s,
        "--slot",
        "@0.fingerprint=73c5da0a",
    ]);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stderr.contains("does not match the xpub"), "{}", r.stderr);
    assert!(!r.stderr.contains("for example"), "{}", r.stderr);
    assert!(!r.stderr.contains("48'"), "{}", r.stderr);
}

#[test]
fn review_fable_m2_m3_asserted_exit_4_is_told_apart_from_a_mismatch() {
    let (md1, mk1) = empty_origin_multisig_with_mk1(false);
    // Asserted origin, one key cross-checked: UNVERIFIED, not PARTIAL.
    let mut extra = vec!["--origin".to_string(), "@0=m/48'/0'/0'/2'".to_string()];
    extra.extend(cosigner_args(1, &mk1[1]));
    let r = restore_s(&md1, &extra);
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(!r.stderr.contains("PARTIAL"), "{}", r.stderr);
    assert!(r.stderr.contains("UNVERIFIED: "), "{}", r.stderr);
    assert!(r.stderr.contains("not a mismatch"), "{}", r.stderr);
    assert!(!r.stderr.contains("MISMATCH"), "{}", r.stderr);
    // A mismatch: refused, nothing on stdout, MISMATCH and the error line.
    let mut extra = cosigner_args(0, &mk1[1]);
    extra.extend(["--origin".into(), "@1=m/48'/0'/0'/2'".into()]);
    let r = restore_s(&md1, &extra);
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert!(r.stderr.contains("✗ MISMATCH"), "{}", r.stderr);
    assert!(r.stderr.contains("error: restore: "), "{}", r.stderr);
    assert!(!r.stderr.contains("UNVERIFIED"), "{}", r.stderr);
}
