//! C6 (v0.58.1) — `convert --from mk1 --to xpub` path-implied SLIP-0132 hint.
//!
//! Reading an mk1 card whose origin path conventionally implies a SLIP-0132
//! variant (m/49'→ypub, m/84'→zpub, m/48'/…/1'→Ypub, m/48'/…/2'→Zpub) prints a
//! non-blocking stderr NOTE naming the variant + pointing at `--xpub-prefix`.
//! Crucially STDOUT stays the BIP-32-neutral xpub (byte-identity + Bitcoin Core
//! interop preserved — the card cannot distinguish xpub-at-m/84' from
//! zpub-at-m/84', so the variant is a path-convention hint, not recovery).

use assert_cmd::Command;
use serde_json::Value;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

const ZPUB: &str = "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs";
const ZPUB_AS_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

/// Mint an mk1 card via `bundle --slot @0.xpub=…` at the given path/descriptor,
/// returning the space-joined mk1 chunks (the form `convert --from mk1=` reads).
fn mint_mk1(xpub: &str, path: &str, descriptor: &str) -> String {
    let out = bin()
        .args([
            "bundle",
            "--slot",
            &format!("@0.xpub={xpub}"),
            "--slot",
            "@0.fingerprint=5436d724",
            "--slot",
            &format!("@0.path={path}"),
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).expect("bundle --json");
    v["mk1"]
        .as_array()
        .expect("mk1 array")
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Cell 1 — zpub@m/84' card: stdout NEUTRAL xpub, stderr hints zpub.
#[test]
fn mk1_at_bip84_hints_zpub_stdout_neutral() {
    let card = mint_mk1(ZPUB, "m/84'/0'/0'", "wpkh(@0/<0;1>/*)");
    let out = bin()
        .args(["convert", "--from", &format!("mk1={card}"), "--to", "xpub"])
        .assert()
        .success();
    let o = out.get_output();
    let stdout = String::from_utf8_lossy(&o.stdout);
    let stderr = String::from_utf8_lossy(&o.stderr);
    // Stdout is the NEUTRAL xpub (NOT zpub) — byte-identity / interop intact.
    assert!(
        stdout.contains(ZPUB_AS_XPUB) && !stdout.contains(ZPUB),
        "stdout must be neutral xpub, never the zpub: {stdout}"
    );
    // Stderr hints the SLIP-0132 form + the flag.
    assert!(
        stderr.contains("conventionally SLIP-0132 zpub") && stderr.contains("--xpub-prefix zpub"),
        "stderr must hint zpub: {stderr}"
    );
}

/// Cell 2 — taproot m/86' card: neutral path → NO hint.
#[test]
fn mk1_at_bip86_no_hint() {
    let card = mint_mk1(ZPUB_AS_XPUB, "m/86'/0'/0'", "tr(@0/<0;1>/*)");
    let out = bin()
        .args(["convert", "--from", &format!("mk1={card}"), "--to", "xpub"])
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr);
    assert!(
        !stderr.contains("conventionally SLIP-0132"),
        "m/86' (taproot) is neutral → no hint: {stderr}"
    );
}

/// Cell 3 — explicit `--xpub-prefix` suppresses the hint AND emits the variant on
/// stdout (the full round-trip: zpub in → zpub out, byte-identical).
#[test]
fn explicit_xpub_prefix_suppresses_hint_and_emits_variant() {
    let card = mint_mk1(ZPUB, "m/84'/0'/0'", "wpkh(@0/<0;1>/*)");
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("mk1={card}"),
            "--to",
            "xpub",
            "--xpub-prefix",
            "zpub",
            "--network",
            "mainnet",
        ])
        .assert()
        .success();
    let o = out.get_output();
    let stdout = String::from_utf8_lossy(&o.stdout);
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert!(
        stdout.contains(ZPUB),
        "explicit --xpub-prefix zpub emits the zpub: {stdout}"
    );
    assert!(
        !stderr.contains("conventionally SLIP-0132"),
        "the hint suppresses when --xpub-prefix is given: {stderr}"
    );
}

/// Cell 4 (anti-regression) — a NEUTRAL xpub engraved at m/84': stdout is the
/// byte-identical neutral xpub (the hint is PATH-driven, so it still fires —
/// "conventionally zpub" — but stdout never changes). Pins that the note never
/// touches stdout, protecting the `xpub→mk1→xpub` byte-identity contract.
#[test]
fn neutral_xpub_at_bip84_stdout_unchanged_hint_still_fires() {
    let card = mint_mk1(ZPUB_AS_XPUB, "m/84'/0'/0'", "wpkh(@0/<0;1>/*)");
    let out = bin()
        .args(["convert", "--from", &format!("mk1={card}"), "--to", "xpub"])
        .assert()
        .success();
    let o = out.get_output();
    let stdout = String::from_utf8_lossy(&o.stdout);
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert!(
        stdout.contains(ZPUB_AS_XPUB) && !stdout.contains(ZPUB),
        "stdout stays the byte-identical neutral xpub: {stdout}"
    );
    assert!(
        stderr.contains("conventionally SLIP-0132 zpub"),
        "hint is path-driven (m/84' → zpub) regardless of the stored xpub: {stderr}"
    );
}
