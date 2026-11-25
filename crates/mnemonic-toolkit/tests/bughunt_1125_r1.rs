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
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(
        o.stderr
            .contains("info: no origin path supplied for @0; using this descriptor shape's canonical origin m/84'/0'/0'"),
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
