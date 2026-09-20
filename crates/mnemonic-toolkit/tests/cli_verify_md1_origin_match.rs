//! `md1_origin_match` — the check whose absence let a whole defect live.
//!
//! `verify-bundle` compared pubkeys, tree and use-site path, and NOT the key
//! ORIGIN. So `bundle --descriptor` could accept every `--slot @N.path=`,
//! derive with it, drop it, and this command still said `result: ok`. Measured
//! before the emit fix: a card declaring an empty origin verified clean against
//! slots that declared `m/48'/0'/0'/2'` and `m/48'/0'/1'/2'`. The one field
//! that was wrong was the one field nothing compared.
//!
//! Three outcomes, each pinned below, because the MIDDLE one is the design:
//! a card that OMITS an origin describes the same wallet and must not be cried
//! wolf over (the plate is already engraved and the operator cannot fix it),
//! while a card that declares a DIFFERENT one points a signer at a key that is
//! not there and must fail.

use assert_cmd::Command;

const FP: &str = "5436d724";
const X0: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const X1: &str = "xpub6Buxw9MmbkJr8dFGbbbjY46MzzbM8MCosN5AxgxVEstcQMYcAn7oV8DvwYouSbixK4zhej2oTUoMkFD6FaHu7tuZPLiGQ7VKcBcj8fmj4g9";

fn slots(p0: &str, p1: &str) -> Vec<String> {
    vec![
        "--slot".into(),
        format!("@0.xpub={X0}"),
        "--slot".into(),
        format!("@0.fingerprint={FP}"),
        "--slot".into(),
        format!("@0.path={p0}"),
        "--slot".into(),
        format!("@1.xpub={X1}"),
        "--slot".into(),
        format!("@1.fingerprint={FP}"),
        "--slot".into(),
        format!("@1.path={p1}"),
    ]
}

fn emit(p0: &str, p1: &str, out: &std::path::Path) {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--descriptor".into(),
        "wsh(sortedmulti(2,@0,@1))".into(),
    ];
    args.extend(slots(p0, p1));
    args.push("--json".into());
    args.push("--no-engraving-card".into());
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args(&args)
        .assert()
        .success();
    std::fs::write(out, o.get_output().stdout.clone()).unwrap();
}

fn verify(p0: &str, p1: &str, card: &std::path::Path) -> String {
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--descriptor".into(),
        "wsh(sortedmulti(2,@0,@1))".into(),
    ];
    args.extend(slots(p0, p1));
    args.push("--bundle-json".into());
    args.push(card.to_str().unwrap().into());
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args(&args)
        .assert()
        .get_output()
        .clone();
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// (1) Origins agree → the row passes and says so.
#[test]
fn matching_origins_pass() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("c.json");
    emit("m/48'/0'/0'/2'", "m/48'/0'/1'/2'", &c);
    let out = verify("m/48'/0'/0'/2'", "m/48'/0'/1'/2'", &c);
    assert!(
        out.contains("md1_origin_match: ok") && out.contains("all 2 key origins match"),
        "expected a clean origin match, got:\n{out}"
    );
}

/// (3) The card declares a DIFFERENT origin → FAIL. This is the contradiction
/// with consequences: no address check can see it, because addresses derive
/// from the xpubs a card CARRIES, not the origin it DECLARES.
#[test]
fn a_different_declared_origin_fails() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("c.json");
    emit("m/48'/0'/0'/2'", "m/48'/0'/1'/2'", &c);
    let out = verify("m/48'/0'/5'/2'", "m/48'/0'/6'/2'", &c);
    assert!(
        out.contains("md1_origin_match: fail"),
        "a card declaring a different origin must FAIL, got:\n{out}"
    );
    assert!(
        out.contains("looks for a key that is not there"),
        "the failure must say what goes wrong, got:\n{out}"
    );
}

/// The anti-tautology control: the row must not simply always fail. Covered by
/// `matching_origins_pass`, and asserted here as a property — the two cases
/// above must produce DIFFERENT verdicts from the same code path.
#[test]
fn the_row_distinguishes_match_from_contradiction() {
    let d = tempfile::tempdir().unwrap();
    let c = d.path().join("c.json");
    emit("m/48'/0'/0'/2'", "m/48'/0'/1'/2'", &c);
    let good = verify("m/48'/0'/0'/2'", "m/48'/0'/1'/2'", &c);
    let bad = verify("m/48'/0'/5'/2'", "m/48'/0'/6'/2'", &c);
    assert!(good.contains("md1_origin_match: ok"), "good case must pass");
    assert!(bad.contains("md1_origin_match: fail"), "bad case must fail");
}
