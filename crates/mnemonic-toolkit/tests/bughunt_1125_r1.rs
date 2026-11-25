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
