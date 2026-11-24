//! FIX sec-multipath-duplicate-refusal: CLI rows C1-C6
//! (`design/FIX_sec_multipath_duplicate_refusal.md` section 5). Signet public keys only.

use assert_cmd::Command;

const T84: &str = "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";
const TA: &str = "tpubDFH9dgzveyD8zTbPUFuLrGmCydNvxehyNdUXKJAQN8x4aZ4j6UZqGfnqFrD4NqyaTVGKbvEW54tsvPTK2UoSbCC1PJY8iCNiwTL3RWZEheQ";
const TC: &str = "tpubDEwqCvJxKwKWX9xvRe48uofWJn1Y89Jn8UeH1Efrjb1UEVjUDy3URYTiqWaVCW7WdvHrL8XrSihHEhTwv5H3VDJoakjuCHiAnr6xcF2Xm4s";

fn p(i: usize, body: &str, v: u32) -> String {
    format!(
        "@{i} multipath `<{body}>` repeats the value {v}; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)"
    )
}

fn run(args: &[&str]) -> std::process::Output {
    let mut c = Command::cargo_bin("mnemonic").unwrap();
    c.arg("--allow-argv-secret").args(args).output().unwrap()
}

fn assert_refused(o: &std::process::Output, code: i32, stderr: &str) {
    assert_eq!(o.status.code(), Some(code));
    assert!(
        o.stdout.is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert_eq!(
        String::from_utf8_lossy(&o.stderr),
        format!("error: {stderr}\n")
    );
}

fn single(tail: &str) -> String {
    format!("wpkh([73c5da0a/84h/1h/0h]{T84}{tail})")
}

fn bundle(desc: &str, slots: &[String]) -> std::process::Output {
    let mut a = vec!["bundle", "--network", "signet", "--descriptor", desc];
    for s in slots {
        a.push("--slot");
        a.push(s);
    }
    run(&a)
}

#[test]
fn bundle_concrete_duplicate_multipath_refused() {
    let o = bundle(&single("/<0;0>/*"), &[format!("@0.xpub={T84}")]);
    assert_refused(&o, 2, &p(0, "0;0", 0));
}

#[test]
fn bundle_template_duplicate_multipath_refused() {
    let o = bundle(
        "wpkh([73c5da0a/84h/1h/0h]@0/<1;1>/*)",
        &[format!("@0.xpub={T84}")],
    );
    assert_refused(&o, 2, &p(0, "1;1", 1));
}

#[test]
fn bundle_multisig_second_key_duplicate_multipath_refused() {
    let d = format!(
        "wsh(sortedmulti(2,[73c5da0a/48h/1h/0h/2h]{TA}/<0;1>/*,[28645006/48h/1h/0h/2h]{TC}/<2;2>/*))"
    );
    let o = bundle(&d, &[format!("@0.xpub={TA}"), format!("@1.xpub={TC}")]);
    assert_refused(&o, 2, &p(1, "2;2", 2));
}

#[test]
fn import_wallet_descriptor_duplicate_multipath_refused() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("w.txt");
    std::fs::write(&f, format!("{}\n", single("/<0;0>/*"))).unwrap();
    let o = run(&[
        "import-wallet",
        "--network",
        "signet",
        "--format",
        "descriptor",
        "--blob",
        f.to_str().unwrap(),
    ]);
    assert_refused(
        &o,
        2,
        &format!("import-wallet: descriptor: parse error: {}", p(0, "0;0", 0)),
    );
}

#[test]
fn verify_bundle_duplicate_multipath_refused() {
    let x = format!("@0.xpub={T84}");
    let o = run(&[
        "verify-bundle",
        "--network",
        "signet",
        "--mk1",
        "",
        "--md1",
        "",
        "--descriptor",
        &single("/<0;0>/*"),
        "--slot",
        &x,
    ]);
    assert_refused(&o, 2, &p(0, "0;0", 0));
}

#[test]
fn bundle_distinct_multipath_still_accepted() {
    for t in ["/<0;1>/*", "/<0;1;2>/*"] {
        let o = bundle(&single(t), &[format!("@0.xpub={T84}")]);
        assert_eq!(o.status.code(), Some(0), "{t}");
        assert!(String::from_utf8_lossy(&o.stdout).contains("md1"), "{t}");
    }
}
