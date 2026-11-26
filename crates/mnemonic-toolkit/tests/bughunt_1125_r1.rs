//! Bug hunt 2026-11-25, review round 1 fold, plus Brian's BIP-defaults
//! rulings. One test group per item; each names it.
//!
//! - r1 I2: an inline fingerprint-only origin `[fp]@N` on a canonical shape
//!   was silently rewritten to the canonical origin.
//! - r1 I3: Sparrow / Electrum / Coldcard exports substituted the template
//!   default origin for the slot's own path.
//! - B1-B5: the BIP-defaults ruling (xpub-search sh(wsh) 1'; bare sh(...) and
//!   taproot multisig default to BIP-87; bare `@N` refused at mint; m/45' in
//!   the empty-origin seed search).

use assert_cmd::Command;

pub(crate) const A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
pub(crate) const B: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

pub(crate) struct Out {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub(crate) fn run(args: &[&str]) -> Out {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args(args)
        .output()
        .unwrap();
    Out {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

// ---------------------------------------------------------------------------
// r1 I2 — `[fp]@N` (BIP-380: the key IS the master of fp) on a canonical
// shape is refused at mint, like `--slot @N.path=m`.
// ---------------------------------------------------------------------------

#[test]
fn r1_i2_inline_fingerprint_only_origin_on_canonical_shape_is_refused() {
    let seed_a = format!("@0.phrase={A}");
    let seed_b = format!("@1.phrase={B}");
    for (desc, slots) in [
        ("wpkh([73c5da0a]@0/<0;1>/*)", vec![seed_a.as_str()]),
        (
            "wsh(sortedmulti(2,[73c5da0a]@0/<0;1>/*,[b8688df1]@1/<0;1>/*))",
            vec![seed_a.as_str(), seed_b.as_str()],
        ),
    ] {
        let mut args = vec!["bundle", "--network", "mainnet", "--descriptor", desc];
        for s in &slots {
            args.extend(["--slot", s]);
        }
        let o = run(&args);
        assert_eq!(o.code, 2, "{desc}: {}", o.stderr);
        assert!(o.stdout.is_empty(), "{desc}: no card: {}", o.stdout);
        assert!(
            o.stderr.contains("[73c5da0a]@0") && o.stderr.contains("master"),
            "{desc}: {}",
            o.stderr
        );
        assert!(
            o.stderr.contains(
                "(a fingerprint with no path), which says the key is the master key of 73c5da0a. \
                 The master node cannot be recorded"
            ),
            "{desc}: one sentence, no stray spaces: {}",
            o.stderr
        );
        assert!(
            !o.stderr.contains("no origin supplied"),
            "N4: the origin WAS supplied: {}",
            o.stderr
        );
    }
}

/// The canonical-origin notice says what is missing: the PATH.
#[test]
fn r1_n4_canonical_notice_names_the_missing_path() {
    let seed_a = format!("@0.phrase={A}");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wpkh(@0/<0;1>/*)",
        "--slot",
        &seed_a,
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(
        o.stderr
            .contains("warning: no origin was given for @0; the toolkit chose m/84'/0'/0' (this descriptor shape's canonical origin)"),
        "{}",
        o.stderr
    );
}

// ---------------------------------------------------------------------------
// r1 I3 — the Sparrow, Electrum and Coldcard emitters write the slot's own
// origin path, never the template default (owner's ruling, toolkit#2).
// ---------------------------------------------------------------------------

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

#[test]
fn r1_i3_singlesig_exports_keep_the_typed_account_path() {
    let x = xpub_at(A, "m/84'/0'/7'");
    let xs = format!("@0.xpub={x}");
    for format in ["sparrow", "electrum", "coldcard"] {
        let o = run(&[
            "export-wallet",
            "--format",
            format,
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &xs,
            "--slot",
            "@0.fingerprint=73c5da0a",
            "--slot",
            "@0.path=m/84'/0'/7'",
            "--output",
            "-",
        ]);
        assert_eq!(o.code, 0, "{format}: {}", o.stderr);
        assert!(
            o.stdout.contains("m/84'/0'/7'"),
            "{format}: the typed origin must be exported: {}",
            o.stdout
        );
        assert!(
            !o.stdout.contains("m/84'/0'/0'"),
            "{format}: the template default must not replace it: {}",
            o.stdout
        );
    }
}

#[test]
fn r1_i3_multisig_exports_keep_an_explicit_master_path() {
    let master = xpub_at(A, "m");
    let other = xpub_at(B, "m/87'/0'/0'");
    let s0 = format!("@0.xpub={master}");
    let s1 = format!("@1.xpub={other}");
    for format in ["sparrow", "electrum"] {
        let o = run(&[
            "export-wallet",
            "--format",
            format,
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--slot",
            &s0,
            "--slot",
            "@0.fingerprint=73c5da0a",
            "--slot",
            "@0.path=m",
            "--slot",
            &s1,
            "--slot",
            "@1.fingerprint=b8688df1",
            "--output",
            "-",
        ]);
        assert_eq!(o.code, 0, "{format}: {}", o.stderr);
        let v: serde_json::Value = serde_json::from_str(&o.stdout).expect("json");
        let derivs: Vec<String> = match format {
            "sparrow" => v["keystores"]
                .as_array()
                .unwrap()
                .iter()
                .map(|k| {
                    k["keyDerivation"]["derivation"]
                        .as_str()
                        .unwrap()
                        .to_string()
                })
                .collect(),
            _ => ["x1/", "x2/"]
                .iter()
                .map(|k| v[*k]["derivation"].as_str().unwrap().to_string())
                .collect(),
        };
        assert!(
            derivs.contains(&"m".to_string()) && derivs.contains(&"m/87'/0'/0'".to_string()),
            "{format}: {derivs:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// B1-B3 — Brian's BIP-defaults ruling: sh(wsh) → BIP-48 1'; any other
// non-canonical sh(...) and taproot multisig → BIP-87; bundle, verify-bundle
// and xpub-search agree.
// ---------------------------------------------------------------------------

fn md1_of(json: &str) -> Vec<String> {
    let v: serde_json::Value = serde_json::from_str(json).expect("bundle --json");
    v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn origins_on_card(md1: &[String]) -> Vec<String> {
    let refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let d = md_codec::chunk::reassemble(&refs).expect("md1 decodes");
    let expanded = md_codec::canonicalize::expand_per_at_n(&d).expect("expand");
    expanded
        .iter()
        .map(|e| {
            e.origin_path
                .components
                .iter()
                .map(|c| format!("{}{}", c.value, if c.hardened { "'" } else { "" }))
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect()
}

fn run_stdin(args: &[&str], stdin: &str) -> Out {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args(args)
        .write_stdin(stdin.as_bytes().to_vec())
        .output()
        .unwrap();
    Out {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

#[test]
fn b3_taproot_multisig_descriptor_defaults_to_bip87() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let o = run(&[
        "bundle",
        "--network",
        "testnet",
        "--descriptor",
        "tr(@0/<0;1>/*,multi_a(1,@1/<0;1>/*))",
        "--slot",
        &s0,
        "--slot",
        &s1,
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert_eq!(
        origins_on_card(&md1_of(&o.stdout)),
        vec!["87'/1'/0'", "87'/1'/0'"]
    );
    assert!(
        o.stderr.contains(
            "warning: no origin was given for @0,@1; the toolkit chose m/87'/1'/0' (BIP-87)"
        ),
        "{}",
        o.stderr
    );
    assert!(!o.stderr.contains("/3'"), "no BIP-48 3': {}", o.stderr);
    // Review r2 M5: the key-reuse advice rides on every BIP-87 default.
    assert!(
        o.stderr.contains(
            "The same BIP-87 key serves every script type at one account: use a different \
             --account for each wallet from this seed."
        ),
        "{}",
        o.stderr
    );
}

#[test]
fn b1_b2_b3_xpub_search_default_follows_the_ruling() {
    // (shape, default named in the notice, path A's key really sits at).
    // Review r2 I-A: the search must FIND the key at the path the new
    // defaults mint (BIP-87, and m/45' for the bare-P2SH alternative), not
    // only name it.
    for (shape, default, key_path) in [
        (
            "sh(wsh(sortedmulti(1,{a},{b})))",
            "m/48'/0'/0'/1'",
            "m/48'/0'/0'/1'",
        ),
        ("sh(sortedmulti(1,{a},{b}))", "m/87'/0'/0'", "m/87'/0'/0'"),
        ("sh(sortedmulti(1,{a},{b}))", "m/87'/0'/0'", "m/45'"),
        ("tr({b},multi_a(1,{a}))", "m/87'/0'/0'", "m/87'/0'/0'"),
        (
            "wsh(sortedmulti(1,{a},{b}))",
            "m/48'/0'/0'/2'",
            "m/48'/0'/0'/2'",
        ),
    ] {
        let a = format!("{}/<0;1>/*", xpub_at(A, key_path));
        let b = format!("{}/<0;1>/*", xpub_at(B, key_path));
        let desc = shape.replace("{a}", &a).replace("{b}", &b);
        let o = run_stdin(
            &[
                "xpub-search",
                "account-of-descriptor",
                "--phrase-stdin",
                "--descriptor",
                &desc,
                "--min-account",
                "0",
                "--number-of-accounts",
                "1",
            ],
            A,
        );
        assert!(
            o.stderr
                .contains(&format!("defaulting origin path for @0,@1 to {default}")),
            "{shape}: {}",
            o.stderr
        );
        let at = if shape.starts_with("tr(") { "@1" } else { "@0" };
        assert_eq!(o.code, 0, "{shape} at {key_path}: {}{}", o.stdout, o.stderr);
        assert!(
            o.stdout
                .contains(&format!("match: cosigner {at}  {key_path}")),
            "{shape} at {key_path}: {}",
            o.stdout
        );
    }
}

/// The bare-P2SH WARNING, and the m/45' command it prints: run as printed,
/// it mints the same bundle with origin m/45' on @0's card entry, and
/// verify-bundle accepts that bundle.
#[test]
fn b2_bare_sh_multisig_defaults_to_bip87_with_a_runnable_m45_command() {
    let xb = xpub_at(B, "m/87'/0'/0'");
    let s1 = format!("@1.xpub={xb}");
    let desc = "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))";
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
        "@1.fingerprint=b8688df1",
        "--json",
    ];
    let o = run_stdin(&args, A);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert_eq!(
        origins_on_card(&md1_of(&o.stdout)),
        vec!["87'/0'/0'", "87'/0'/0'"]
    );
    for want in [
        "warning: no origin was given for @0,@1 in a legacy P2SH descriptor",
        "m/87'/0'/0' (BIP-87)",
        "m/45', m/45'/0, m/45'/0'/0'",
        "compare the first address",
        "use a different --account",
        "warning: to state this path explicitly, run:",
        "warning: to mint this bundle at m/45' instead, run:",
        "warning: m/45' has no coin or account level (every --account and network gives the \
         same keys); pass the same m/45' origins to verify-bundle.",
    ] {
        assert!(o.stderr.contains(want), "missing {want:?}: {}", o.stderr);
    }
    assert!(
        !o.stderr.contains("info: no origin"),
        "a WARNING, not info: {}",
        o.stderr
    );
    let lines: Vec<&str> = o.stderr.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.contains("to mint this bundle at m/45' instead"))
        .unwrap();
    let cmd = lines[at + 1].trim();
    // @0 (the seed) moves to m/45'; the xpub @1 keeps the path it was minted
    // at, stated explicitly, so the printed command chooses nothing (exit 0).
    assert!(
        cmd.ends_with("--slot '@0.path=m/45'\\''' --slot '@1.path=m/87'\\''/0'\\''/0'\\'''"),
        "{cmd}"
    );
    let cmd = cmd
        .strip_prefix("mnemonic ")
        .map(|rest| {
            format!(
                "'{}' {rest}",
                assert_cmd::cargo::cargo_bin("mnemonic").display()
            )
        })
        .expect("printed as `mnemonic ...`");
    assert!(
        !o.stderr.contains("not repeated above"),
        "no inline secret: {}",
        o.stderr
    );

    // Run the printed command exactly as printed.
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut c| {
            use std::io::Write;
            c.stdin.take().unwrap().write_all(A.as_bytes())?;
            c.wait_with_output()
        })
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        out.status.success(),
        "printed command failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(origins_on_card(&md1_of(&stdout)), vec!["45'", "87'/0'/0'"]);

    // verify-bundle accepts it, given the same slots.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bundle.json");
    std::fs::write(&path, &stdout).unwrap();
    let v = run_stdin(
        &[
            "verify-bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            "@0.phrase=-",
            "--slot",
            "@0.path=m/45'",
            "--slot",
            &s1,
            "--slot",
            "@1.fingerprint=b8688df1",
            "--bundle-json",
            path.to_str().unwrap(),
        ],
        A,
    );
    assert_eq!(v.code, 0, "verify-bundle: {}\n{}", v.stdout, v.stderr);

    // The key on the card is A's m/45' key.
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        v["multisig"]["cosigners"][0]["xpub"].as_str(),
        Some(xpub_at(A, "m/45'").as_str()),
        "@0 is derived at m/45'"
    );
}

/// A non-canonical sh(wsh(...)) is BIP-48 1' (P2SH-P2WSH), not legacy P2SH:
/// the plain default warning, no P2SH warning and no m/45' command.
#[test]
fn b2_sh_wsh_multisig_keeps_bip48_1h_and_no_warning() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "sh(wsh(and_v(v:pk(@0/<0;1>/*),pk(@1/<0;1>/*))))",
        "--slot",
        &s0,
        "--slot",
        &s1,
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert_eq!(
        origins_on_card(&md1_of(&o.stdout)),
        vec!["48'/0'/0'/1'", "48'/0'/0'/1'"]
    );
    assert!(
        o.stderr.contains(
            "warning: no origin was given for @0,@1; the toolkit chose m/48'/0'/0'/1' (BIP-48 cosigner path)"
        ),
        "{}",
        o.stderr
    );
    assert!(!o.stderr.contains("legacy P2SH"), "{}", o.stderr);
    assert!(!o.stderr.contains("m/45'"), "{}", o.stderr);
}

// ---------------------------------------------------------------------------
// B4 — a bare `@N` (no derivation after it) is refused at mint. BIP-388 keys
// are "always followed by" `/**` or `/<M;N>/*`; a bare key minted as `@N/*`
// would describe different keys than the ones typed.
// ---------------------------------------------------------------------------

#[test]
fn b4_bare_placeholder_is_refused_at_mint() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    for (desc, slots) in [
        ("wpkh(@0)", vec![s0.as_str()]),
        (
            "wsh(sortedmulti(2,@0/<0;1>/*,@1))",
            vec![s0.as_str(), s1.as_str()],
        ),
        (
            "wsh(and_v(v:pk(@0),pk(@1/<0;1>/*)))",
            vec![s0.as_str(), s1.as_str()],
        ),
    ] {
        let mut args = vec!["bundle", "--network", "mainnet", "--descriptor", desc];
        for s in &slots {
            args.extend(["--slot", s]);
        }
        let o = run(&args);
        assert_eq!(o.code, 2, "{desc}: {}", o.stderr);
        assert!(o.stdout.is_empty(), "{desc}: no card: {}", o.stdout);
        assert!(
            o.stderr.contains("has no derivation after it") && o.stderr.contains("/<0;1>/*"),
            "{desc}: {}",
            o.stderr
        );
    }
}

/// An explicit `/*` or `/<0;1>/*` still mints, and a `/*` card still restores
/// (decoding is unchanged).
#[test]
fn b4_explicit_derivations_still_mint_and_restore() {
    let s0 = format!("@0.phrase={A}");
    for desc in ["wpkh(@0/*)", "wpkh(@0/<0;1>/*)", "wpkh(@0/**)"] {
        let o = run(&[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            &s0,
            "--json",
        ]);
        assert_eq!(o.code, 4, "{desc}: canonical origin chosen: {}", o.stderr);
        let md1 = md1_of(&o.stdout);
        let mut args = vec!["restore".to_string()];
        for m in &md1 {
            args.extend(["--md1".to_string(), m.clone()]);
        }
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let r = run(&refs);
        assert_eq!(r.code, 0, "{desc}: restore: {}", r.stderr);
    }
}

// ---------------------------------------------------------------------------
// r2 I-B — a foreign `@N.xpub=` on a non-canonical shape takes the default
// origin only when its own header (depth, child number) agrees, as on
// canonical shapes (toolkit#5); otherwise mint and verify-bundle refuse and
// ask for `@N.path=`.
// ---------------------------------------------------------------------------

#[test]
fn r2_ib_foreign_xpub_whose_header_contradicts_the_default_is_refused() {
    let s0 = format!("@0.phrase={A}");
    for (desc, foreign_path) in [
        // A Sparrow / previous-default taproot cosigner key.
        (
            "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
            "m/48'/0'/0'/3'",
        ),
        // A cosigner key at the old bare-sh default.
        ("sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))", "m/48'/0'/0'/1'"),
    ] {
        let s1 = format!("@1.xpub={}", xpub_at(B, foreign_path));
        let mint = |extra: &[&str]| {
            let mut args = vec![
                "bundle",
                "--network",
                "mainnet",
                "--descriptor",
                desc,
                "--slot",
                &s0,
                "--slot",
                &s1,
                "--slot",
                "@1.fingerprint=b8688df1",
                "--json",
            ];
            args.extend(extra);
            run(&args)
        };
        let o = mint(&[]);
        assert_eq!(o.code, 2, "{desc}: {}", o.stderr);
        assert!(o.stdout.is_empty(), "{desc}: no card: {}", o.stdout);
        assert!(
            o.stderr.contains("slot @1: no `@1.path=` was given")
                && o.stderr.contains("does not match the xpub's own header"),
            "{desc}: {}",
            o.stderr
        );

        // With the real origin it mints; verify-bundle without the path
        // refuses the same way, and with it accepts.
        let p1 = format!("@1.path={foreign_path}");
        let ok = mint(&["--slot", &p1]);
        // @0 (the seed) still takes the default: exit 4.
        assert_eq!(ok.code, 4, "{desc}: {}", ok.stderr);
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("b.json");
        std::fs::write(&file, &ok.stdout).unwrap();
        let verify = |extra: &[&str]| {
            let mut args = vec![
                "verify-bundle",
                "--network",
                "mainnet",
                "--descriptor",
                desc,
                "--slot",
                &s0,
                "--slot",
                &s1,
                "--slot",
                "@1.fingerprint=b8688df1",
                "--bundle-json",
                file.to_str().unwrap(),
            ];
            args.extend(extra);
            run(&args)
        };
        let v = verify(&[]);
        assert_eq!(v.code, 2, "{desc}: verify: {}\n{}", v.stdout, v.stderr);
        assert!(
            v.stderr.contains("does not match the xpub's own header"),
            "{desc}: {}",
            v.stderr
        );
        let v = verify(&["--slot", &p1]);
        assert_eq!(v.code, 0, "{desc}: verify: {}\n{}", v.stdout, v.stderr);
    }
}

#[test]
fn r2_ib_foreign_xpub_at_the_bip87_default_still_mints() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.xpub={}", xpub_at(B, "m/87'/0'/0'"));
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        "--slot",
        &s0,
        "--slot",
        &s1,
        "--slot",
        "@1.fingerprint=b8688df1",
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert_eq!(
        origins_on_card(&md1_of(&o.stdout)),
        vec!["87'/0'/0'", "87'/0'/0'"]
    );
}

// ---------------------------------------------------------------------------
// tk-bh1125-tr-tree-bip87-scope (Brian's ruling, 2026-11-26): BIP-87 is for
// multisig ("it is best to restrict this script-agnostic hierarchy to
// multisignature wallets only"), so a non-canonical tr(...) or bare sh(...)
// that is not a multisig is refused when any @N has no origin.
// tk-bh1125-rr1-m3-bip48-thresh-scope (Brian, 2026-11-26 17:25Z and 17:27Z):
// the same holds for the BIP-48 default (wsh, sh(wsh)), and "multisig" means
// the tree references two or more distinct keys, whatever its fragments:
// thresh(2,pk(A),pk(B),after(..)) is multisig, thresh(1,pk(A),after(..),
// older(..)) is not.
// ---------------------------------------------------------------------------

fn assert_bip87_scope_refusal(o: &Out, missing: &str, ctx: &str) {
    assert_scope_refusal(o, missing, "BIP-87 (m/87'/...)", ctx);
}

fn assert_scope_refusal(o: &Out, missing: &str, family: &str, ctx: &str) {
    assert_eq!(o.code, 2, "{ctx}: exit 2 (usage class): {}", o.stderr);
    assert!(o.stdout.is_empty(), "{ctx}: no card minted: {}", o.stdout);
    for want in [
        format!("no origin was given for {missing}, and this descriptor holds only one key"),
        format!("{family} is for wallets with two or more keys"),
        "--slot @N.path=m/...".to_string(),
        "[fp/path]@N".to_string(),
    ] {
        assert!(
            o.stderr.contains(&want),
            "{ctx}: missing {want:?}: {}",
            o.stderr
        );
    }
    // r3 M1: never suggest a concrete example path that may be wrong.
    assert!(
        !o.stderr.contains("for example"),
        "{ctx}: no example path: {}",
        o.stderr
    );
}

#[test]
fn bip87_scope_single_key_tr_script_tree_without_origin_is_refused() {
    let s0 = format!("@0.phrase={A}");
    let desc = "tr(NUMS,and_v(v:pk(@0/<0;1>/*),after(12000000)))";
    let o = run(&[
        "bundle",
        "--network",
        "testnet",
        "--descriptor",
        desc,
        "--slot",
        &s0,
        "--json",
    ]);
    assert_bip87_scope_refusal(&o, "@0", "seed slot");

    // A foreign xpub slot is refused the same way (its origin would be recorded
    // from the default).
    let s0x = format!("@0.xpub={}", xpub_at(A, "m/87'/0'/0'"));
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        desc,
        "--slot",
        &s0x,
        "--slot",
        "@0.fingerprint=73c5da0a",
        "--json",
    ]);
    assert_bip87_scope_refusal(&o, "@0", "xpub slot");
}

#[test]
fn bip87_scope_single_key_bare_sh_without_origin_is_refused() {
    let s0 = format!("@0.phrase={A}");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "sh(and_v(v:pk(@0/<0;1>/*),after(100)))",
        "--slot",
        &s0,
        "--json",
    ]);
    assert_bip87_scope_refusal(&o, "@0", "bare sh");
}

/// Brian, 2026-11-26 17:27Z: two or more distinct keys make a multisig,
/// whatever the fragments, so a two-key taproot tree of `pk` leaves takes the
/// BIP-87 default (with the chosen-origin warning, exit 4). It was refused by
/// d15b2eff, which counted only multi-family fragments.
#[test]
fn bip87_scope_two_key_tree_without_multi_fragment_takes_the_default() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "tr(NUMS,{pk(@0/<0;1>/*),pk(@1/<0;1>/*)})",
        "--slot",
        &s0,
        "--slot",
        &s1,
        "--json",
    ]);
    assert_eq!(o.code, 4, "two-key tree: {}", o.stderr);
    assert_eq!(
        origins_on_card(&md1_of(&o.stdout)),
        vec!["87'/0'/0'", "87'/0'/0'"]
    );
    assert!(
        o.stderr.contains(
            "warning: no origin was given for @0,@1; the toolkit chose m/87'/0'/0' (BIP-87)"
        ),
        "{}",
        o.stderr
    );
    // Only the @N that lacks an origin takes the default.
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "tr(NUMS,{pk([73c5da0a/86'/0'/0']@0/<0;1>/*),pk(@1/<0;1>/*)})",
        "--slot",
        &s0,
        "--slot",
        &s1,
        "--json",
    ]);
    assert_eq!(o.code, 4, "partial origins: {}", o.stderr);
    assert_eq!(
        origins_on_card(&md1_of(&o.stdout)),
        vec!["86'/0'/0'", "87'/0'/0'"]
    );
    assert!(
        o.stderr
            .contains("warning: no origin was given for @1; the toolkit chose m/87'/0'/0'"),
        "{}",
        o.stderr
    );
}

#[test]
fn bip87_scope_non_multisig_tree_with_explicit_origin_mints() {
    let s0 = format!("@0.phrase={A}");
    let desc = "tr(NUMS,and_v(v:pk(@0/<0;1>/*),after(12000000)))";
    // --slot @N.path=
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        desc,
        "--slot",
        &s0,
        "--slot",
        "@0.path=m/86'/0'/5'",
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(origins_on_card(&md1_of(&o.stdout)), vec!["86'/0'/5'"]);
    assert!(!o.stderr.contains("no origin"), "{}", o.stderr);
    assert!(
        !o.stderr.contains("BIP-87 is written for multisig"),
        "{}",
        o.stderr
    );
    // inline [fp/path]@N
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "sh(and_v(v:pk([73c5da0a/44'/0'/0']@0/<0;1>/*),after(100)))",
        "--slot",
        &s0,
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(origins_on_card(&md1_of(&o.stdout)), vec!["44'/0'/0'"]);
}

#[test]
fn bip87_scope_multisig_trees_keep_the_bip87_default() {
    // A taproot tree with a multi_a leaf beside a single-key recovery leaf is a
    // multisig wallet: every @N keeps the BIP-87 default and the warning (exit 4).
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let s2 = "@2.phrase=zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
    let o = run(&[
        "bundle",
        "--network",
        "testnet",
        "--descriptor",
        "tr(NUMS,{multi_a(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:pk(@2/<0;1>/*),older(1000))})",
        "--slot",
        &s0,
        "--slot",
        &s1,
        "--slot",
        s2,
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert_eq!(
        origins_on_card(&md1_of(&o.stdout)),
        vec!["87'/1'/0'", "87'/1'/0'", "87'/1'/0'"]
    );
    assert!(
        o.stderr.contains(
            "warning: no origin was given for @0,@1,@2; the toolkit chose m/87'/1'/0' (BIP-87)"
        ),
        "{}",
        o.stderr
    );
    assert!(
        !o.stderr.contains("BIP-87 is written for multisig"),
        "{}",
        o.stderr
    );
}

/// verify-bundle mints nothing, so it keeps rebuilding at the default: a
/// single-key card minted at m/87' before the ruling still verifies with its
/// original command line.
#[test]
fn bip87_scope_verify_bundle_still_rebuilds_a_pre_ruling_single_key_card() {
    let s0 = format!("@0.phrase={A}");
    let desc = "tr(NUMS,and_v(v:pk(@0/<0;1>/*),after(12000000)))";
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        desc,
        "--slot",
        &s0,
        "--slot",
        "@0.path=m/87'/0'/0'",
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bundle.json");
    std::fs::write(&path, &o.stdout).unwrap();
    let v = run(&[
        "verify-bundle",
        "--network",
        "mainnet",
        "--descriptor",
        desc,
        "--slot",
        &s0,
        "--bundle-json",
        path.to_str().unwrap(),
    ]);
    assert_eq!(v.code, 0, "verify-bundle: {}\n{}", v.stdout, v.stderr);
}

/// An `@N.entropy=` slot cannot carry `@N.path=` (slot grammar), so the
/// refusal names only the inline form for it.
#[test]
fn bip87_scope_refusal_for_an_entropy_slot_names_only_the_inline_form() {
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "tr(NUMS,and_v(v:pk(@0/<0;1>/*),after(12000000)))",
        "--slot",
        "@0.entropy=07070707070707070707070707070707",
        "--json",
    ]);
    assert_bip87_scope_refusal(&o, "@0", "entropy slot");
    assert!(
        o.stderr
            .contains("An @N.entropy= slot (@0) takes only the inline form."),
        "{}",
        o.stderr
    );
}

// ---------------------------------------------------------------------------
// tk-bh1125-rr1-m3-bip48-thresh-scope: the BIP-48 default (m/48'/c'/a'/2' for
// wsh, /1' for sh(wsh)) is for wallets with two or more keys, like BIP-87.
// ---------------------------------------------------------------------------

#[test]
fn bip48_scope_single_key_wsh_shapes_without_origin_are_refused() {
    let s0 = format!("@0.phrase={A}");
    for desc in [
        "wsh(and_v(v:pk(@0/<0;1>/*),older(144)))",
        "wsh(pk(@0/<0;1>/*))",
        // Brian's counter-example: a thresh of one key and timelocks is single-sig.
        "wsh(thresh(2,pk(@0/<0;1>/*),sln:after(1000000),sln:older(65535)))",
        "wsh(thresh(1,pk(@0/<0;1>/*),sln:after(1000000),sln:older(65535)))",
        "sh(wsh(pk(@0/<0;1>/*)))",
        "sh(wsh(and_v(v:pk(@0/<0;1>/*),older(144))))",
    ] {
        let o = run(&[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            &s0,
            "--json",
        ]);
        assert_scope_refusal(&o, "@0", "BIP-48 (m/48'/...)", desc);
    }
    // A foreign xpub slot is refused the same way.
    let s0x = format!("@0.xpub={}", xpub_at(A, "m/48'/0'/0'/2'"));
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(and_v(v:pk(@0/<0;1>/*),older(144)))",
        "--slot",
        &s0x,
        "--slot",
        "@0.fingerprint=73c5da0a",
        "--json",
    ]);
    assert_scope_refusal(&o, "@0", "BIP-48 (m/48'/...)", "xpub slot");
    // An entropy slot names only the inline form.
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(and_v(v:pk(@0/<0;1>/*),older(144)))",
        "--slot",
        "@0.entropy=07070707070707070707070707070707",
        "--json",
    ]);
    assert_scope_refusal(&o, "@0", "BIP-48 (m/48'/...)", "entropy slot");
    assert!(
        o.stderr
            .contains("An @N.entropy= slot (@0) takes only the inline form."),
        "{}",
        o.stderr
    );
}

#[test]
fn bip48_scope_two_key_shapes_keep_the_bip48_default() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    for (desc, want) in [
        (
            "wsh(thresh(2,pk(@0/<0;1>/*),s:pk(@1/<0;1>/*),sln:after(1000000)))",
            "48'/0'/0'/2'",
        ),
        (
            "wsh(or_d(pk(@0/<0;1>/*),and_v(v:pk(@1/<0;1>/*),older(144))))",
            "48'/0'/0'/2'",
        ),
        (
            "sh(wsh(or_d(pk(@0/<0;1>/*),and_v(v:pk(@1/<0;1>/*),older(144)))))",
            "48'/0'/0'/1'",
        ),
    ] {
        let o = run(&[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            &s0,
            "--slot",
            &s1,
            "--json",
        ]);
        assert_eq!(o.code, 4, "{desc}: {}", o.stderr);
        assert_eq!(
            origins_on_card(&md1_of(&o.stdout)),
            vec![want, want],
            "{desc}"
        );
        assert!(
            o.stderr.contains(&format!(
                "warning: no origin was given for @0,@1; the toolkit chose m/{want} (BIP-48 cosigner path)"
            )),
            "{desc}: {}",
            o.stderr
        );
    }
}

#[test]
fn bip48_scope_single_key_wsh_with_explicit_origin_mints() {
    let s0 = format!("@0.phrase={A}");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(and_v(v:pk(@0/<0;1>/*),older(144)))",
        "--slot",
        &s0,
        "--slot",
        "@0.path=m/84'/0'/3'",
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(origins_on_card(&md1_of(&o.stdout)), vec!["84'/0'/3'"]);
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "sh(wsh(pk([73c5da0a/49'/0'/0']@0/<0;1>/*)))",
        "--slot",
        &s0,
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(origins_on_card(&md1_of(&o.stdout)), vec!["49'/0'/0'"]);
}

/// verify-bundle mints nothing and keeps rebuilding at the BIP-48 default, so
/// a single-key wsh card minted there before the ruling still verifies with
/// its original command line (no path).
#[test]
fn bip48_scope_verify_bundle_still_rebuilds_a_pre_ruling_single_key_card() {
    let s0 = format!("@0.phrase={A}");
    for (desc, path) in [
        ("wsh(and_v(v:pk(@0/<0;1>/*),older(144)))", "m/48'/0'/0'/2'"),
        ("sh(wsh(pk(@0/<0;1>/*)))", "m/48'/0'/0'/1'"),
    ] {
        let slot_path = format!("@0.path={path}");
        let o = run(&[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            &s0,
            "--slot",
            &slot_path,
            "--json",
        ]);
        assert_eq!(o.code, 0, "{desc}: {}", o.stderr);
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("bundle.json");
        std::fs::write(&file, &o.stdout).unwrap();
        let v = run(&[
            "verify-bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            &s0,
            "--bundle-json",
            file.to_str().unwrap(),
        ]);
        assert_eq!(
            v.code, 0,
            "{desc}: verify-bundle: {}\n{}",
            v.stdout, v.stderr
        );
    }
}

/// Template mode is unaffected: every multisig template is a multisig, and
/// --multisig-path-family bip48 still mints at m/48'/c'/a'/2'.
#[test]
fn bip48_scope_multisig_template_is_unaffected() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
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
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(
        origins_on_card(&md1_of(&o.stdout)),
        vec!["48'/0'/0'/2'", "48'/0'/0'/2'"]
    );
}

// ---------------------------------------------------------------------------
// rulings2-r1 I-1 — the canonical BIP-48 fill of wsh / sh(wsh) of multi /
// sortedmulti is a multisig hierarchy too: a one-key `multi(1,@0)` with no
// origin is refused at mint like every other one-key tree.
// ---------------------------------------------------------------------------

#[test]
fn bip48_scope_single_key_canonical_multi_without_origin_is_refused() {
    let s0 = format!("@0.phrase={A}");
    for desc in [
        "wsh(multi(1,@0/**))",
        "wsh(sortedmulti(1,@0/<0;1>/*))",
        "sh(wsh(sortedmulti(1,@0/<0;1>/*)))",
        "sh(wsh(multi(1,@0/<0;1>/*)))",
    ] {
        let o = run(&[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            &s0,
            "--json",
        ]);
        assert_scope_refusal(&o, "@0", "BIP-48 (m/48'/...)", desc);
        assert!(
            o.stderr
                .contains("On this descriptor shape a seed slot (@0) takes only the inline form."),
            "{desc}: a phrase slot cannot take @0.path= here: {}",
            o.stderr
        );
        assert!(
            !o.stderr.contains("the toolkit chose"),
            "{desc}: no chosen-origin warning before a refusal: {}",
            o.stderr
        );
    }
    // A foreign xpub slot is refused the same way.
    let s0x = format!("@0.xpub={}", xpub_at(A, "m/48'/0'/0'/2'"));
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(sortedmulti(1,@0/<0;1>/*))",
        "--slot",
        &s0x,
        "--slot",
        "@0.fingerprint=73c5da0a",
        "--json",
    ]);
    assert_scope_refusal(&o, "@0", "BIP-48 (m/48'/...)", "xpub slot");
    // The `[fp]@0` refusal on a one-key canonical multi does not advise
    // dropping the brackets (that would be refused too).
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(sortedmulti(1,[73c5da0a]@0/<0;1>/*))",
        "--slot",
        &s0,
    ]);
    assert_eq!(o.code, 2, "{}", o.stderr);
    assert!(
        !o.stderr.contains("drop the brackets"),
        "no refused advice: {}",
        o.stderr
    );
}

/// The ruling is mint-time admission only: a one-key canonical multi with an
/// explicit origin mints, and a card minted at the BIP-48 path (as the fill
/// used to do) still verifies with no path, as before the ruling.
#[test]
fn bip48_scope_single_key_canonical_multi_explicit_and_pre_ruling_verify() {
    let s0 = format!("@0.phrase={A}");
    for (desc, minted, want) in [
        (
            "wsh(sortedmulti(1,@0/<0;1>/*))",
            "wsh(sortedmulti(1,[73c5da0a/84'/0'/0']@0/<0;1>/*))",
            "84'/0'/0'",
        ),
        (
            "wsh(sortedmulti(1,@0/<0;1>/*))",
            "wsh(sortedmulti(1,[73c5da0a/48'/0'/0'/2']@0/<0;1>/*))",
            "48'/0'/0'/2'",
        ),
        (
            "sh(wsh(sortedmulti(1,@0/<0;1>/*)))",
            "sh(wsh(sortedmulti(1,[73c5da0a/48'/0'/0'/1']@0/<0;1>/*)))",
            "48'/0'/0'/1'",
        ),
    ] {
        let o = run(&[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            minted,
            "--slot",
            &s0,
            "--json",
        ]);
        assert_eq!(o.code, 0, "{minted}: {}", o.stderr);
        if !want.starts_with("48'") {
            assert_eq!(origins_on_card(&md1_of(&o.stdout)), vec![want], "{minted}");
            continue;
        }
        // The card a pre-ruling `bundle --descriptor <desc>` (no origin)
        // minted is this one; verify-bundle still rebuilds it with no path.
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("bundle.json");
        std::fs::write(&file, &o.stdout).unwrap();
        let v = run(&[
            "verify-bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            &s0,
            "--bundle-json",
            file.to_str().unwrap(),
        ]);
        assert_eq!(
            v.code, 0,
            "{desc}: verify-bundle: {}\n{}",
            v.stdout, v.stderr
        );
    }
}

/// Two-key canonical multisig keeps the canonical BIP-48 fill (exit 4 notice).
#[test]
fn bip48_scope_two_key_canonical_multi_keeps_the_fill() {
    let s0 = format!("@0.phrase={A}");
    let s1 = format!("@1.phrase={B}");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(sortedmulti(1,@0/<0;1>/*,@1/<0;1>/*))",
        "--slot",
        &s0,
        "--slot",
        &s1,
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(
        o.stderr.contains("the toolkit chose m/48'/0'/0'/2'"),
        "{}",
        o.stderr
    );
}
