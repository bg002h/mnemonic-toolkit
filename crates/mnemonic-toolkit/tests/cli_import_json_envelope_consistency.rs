//! Envelope consistency (architect ruling 2026-11-04): `bundle --import-json`
//! and `export-wallet --from-import-json` refuse an envelope whose md1, mk1,
//! origin path(s) or ms1 disagree with its `bundle.descriptor`. Signet keys
//! derived from the BIP-39 test phrases `abandon..about`, `zoo..wrong`,
//! `legal winner..yellow`, `letter advice..above` at m/48'/1'/0'/2'.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

const K0: &str = "[73c5da0a/48'/1'/0'/2']tpubDFH9dgzveyD8zTbPUFuLrGmCydNvxehyNdUXKJAQN8x4aZ4j6UZqGfnqFrD4NqyaTVGKbvEW54tsvPTK2UoSbCC1PJY8iCNiwTL3RWZEheQ/<0;1>/*";
const K1: &str = "[3f635a63/48'/1'/0'/2']tpubDFPtPArj4GzBEFHohegg1Xatrc1Fi9oSox5LzuSRX91miwQxuUrEpBxpvDRsmZYJKYFhgdK3UStsjC8JKXfUbMinjFqiEM4uNwzVaCaHpys/<0;1>/*";
const K2: &str = "[b8688df1/48'/1'/0'/2']tpubDEfobrrtptRTbKf4gysDhoabneABDTAcdj3Vbn4XwPsLE2pmqpizSPRG6zHsbAMuiSgWmWPsYCLHTKTPpyrGJ5rAoTpKoQNZcxodiPf2tSJ/<0;1>/*";
const K3: &str = "[28645006/48'/1'/0'/2']tpubDEwqCvJxKwKWX9xvRe48uofWJn1Y89Jn8UeH1Efrjb1UEVjUDy3URYTiqWaVCW7WdvHrL8XrSihHEhTwv5H3VDJoakjuCHiAnr6xcF2Xm4s/<0;1>/*";
const P0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// ms1 entropy card of `zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong`.
const MS1_ZOO: &str = "ms10entrsqrlllllllllllllllllllllllllsnrx3qek9l5q60";
const FORMATS: [&str; 6] = [
    "descriptor",
    "bitcoin-core",
    "bip388",
    "sparrow",
    "coldcard",
    "electrum",
];
const TAIL: &str = "so the envelope is not as import-wallet wrote it; re-create it with import-wallet from the same blob";

fn e1(label: &str) -> String {
    format!("error: {label}: envelope.bundle.md1 does not match envelope.bundle.descriptor, {TAIL}")
}
fn e2_key(label: &str, i: usize) -> String {
    format!(
        "error: {label}: cosigner {i} of envelope.bundle.mk1 (its key, fingerprint or origin path) \
         does not match key @{i} of envelope.bundle.descriptor, {TAIL}"
    )
}
fn e2_count(label: &str, m: usize, n: usize) -> String {
    format!(
        "error: {label}: envelope.bundle.mk1 has {m} cosigners but envelope.bundle.descriptor \
         has {n} keys, {TAIL}"
    )
}
fn e3(i: usize) -> String {
    format!(
        "error: --import-json: envelope.bundle.ms1[{i}] is not the seed of cosigner {i} of \
         envelope.bundle.mk1, {TAIL}"
    )
}

/// `import-wallet --format descriptor --network signet --json` of the
/// descriptor with @0..@3 replaced by K0..K3 (seed for cosigner 0 when `seed0`).
fn import(template: &str, seed0: bool) -> Value {
    let d = template
        .replace("@0", K0)
        .replace("@1", K1)
        .replace("@2", K2)
        .replace("@3", K3);
    let mut args = vec![
        "import-wallet",
        "--format",
        "descriptor",
        "--blob",
        "-",
        "--network",
        "signet",
        "--json",
    ];
    if seed0 {
        args.extend(["--slot", "@0.phrase=@env:ENVC_P0"]);
    }
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("ENVC_P0", P0)
        .args(&args)
        .write_stdin(format!("{d}\n"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "import-wallet {template}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

fn write(dir: &Path, name: &str, v: &Value) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, serde_json::to_string(v).unwrap()).unwrap();
    p
}

fn export(env: &Path, format: &str) -> std::process::Output {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            env.to_str().unwrap(),
            "--format",
            format,
        ])
        .output()
        .unwrap()
}

fn bundle(env: &Path, network: &str) -> std::process::Output {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            network,
            "--import-json",
            env.to_str().unwrap(),
        ])
        .output()
        .unwrap()
}

/// Exit 1, nothing on stdout, and `line` is one whole line of stderr.
fn assert_refused(out: &std::process::Output, line: &str, what: &str) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{what}: stderr={stderr}");
    assert!(
        out.stdout.is_empty(),
        "{what}: stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        stderr.lines().any(|l| l == line),
        "{what}: want line\n{line}\nstderr=\n{stderr}"
    );
}

const PLAIN: &str = "wsh(sortedmulti(2,@0,@1,@2))";

/// The honest 2-of-3 envelope with `edit` applied to `[0].bundle`.
fn edited(dir: &Path, edit: impl FnOnce(&mut Value)) -> PathBuf {
    let mut v = import(PLAIN, false);
    edit(&mut v[0]["bundle"]);
    write(dir, "edited.json", &v)
}

#[test]
fn envc_export_refuses_descriptor_edited_to_1_of_3() {
    let dir = tempfile::tempdir().unwrap();
    let one = import("wsh(sortedmulti(1,@0,@1,@2))", false);
    let p = edited(dir.path(), |b| {
        b["descriptor"] = one[0]["bundle"]["descriptor"].clone()
    });
    for f in FORMATS {
        assert_refused(&export(&p, f), &e1("--from-import-json"), f);
    }
}

#[test]
fn envc_bundle_refuses_descriptor_edited_to_1_of_3() {
    let dir = tempfile::tempdir().unwrap();
    let one = import("wsh(sortedmulti(1,@0,@1,@2))", false);
    let p = edited(dir.path(), |b| {
        b["descriptor"] = one[0]["bundle"]["descriptor"].clone()
    });
    assert_refused(&bundle(&p, "signet"), &e1("--import-json"), "bundle");
}

#[test]
fn envc_export_refuses_md1_of_another_wallet() {
    let dir = tempfile::tempdir().unwrap();
    let other = import("wsh(sortedmulti(2,@0,@1,@3))", false);
    let p = edited(dir.path(), |b| b["md1"] = other[0]["bundle"]["md1"].clone());
    for f in FORMATS {
        assert_refused(&export(&p, f), &e1("--from-import-json"), f);
    }
}

#[test]
fn envc_bundle_refuses_md1_of_another_wallet() {
    let dir = tempfile::tempdir().unwrap();
    let other = import("wsh(sortedmulti(2,@0,@1,@3))", false);
    let p = edited(dir.path(), |b| b["md1"] = other[0]["bundle"]["md1"].clone());
    assert_refused(&bundle(&p, "signet"), &e1("--import-json"), "bundle");
}

#[test]
fn envc_export_refuses_swapped_mk1_card() {
    let dir = tempfile::tempdir().unwrap();
    let other = import("wsh(sortedmulti(2,@0,@1,@3))", false);
    let p = edited(dir.path(), |b| {
        b["mk1"][2] = other[0]["bundle"]["mk1"][2].clone()
    });
    for f in FORMATS {
        assert_refused(&export(&p, f), &e2_key("--from-import-json", 2), f);
    }
}

#[test]
fn envc_bundle_refuses_swapped_mk1_card() {
    let dir = tempfile::tempdir().unwrap();
    let other = import("wsh(sortedmulti(2,@0,@1,@3))", false);
    let p = edited(dir.path(), |b| {
        b["mk1"][2] = other[0]["bundle"]["mk1"][2].clone()
    });
    assert_refused(&bundle(&p, "signet"), &e2_key("--import-json", 2), "bundle");
}

#[test]
fn envc_export_refuses_edited_origin_path() {
    let dir = tempfile::tempdir().unwrap();
    let p = edited(dir.path(), |b| {
        b["origin_path"] = Value::from("m/48'/1'/7'/2'")
    });
    for f in FORMATS {
        assert_refused(&export(&p, f), &e2_key("--from-import-json", 0), f);
    }
}

#[test]
fn envc_bundle_refuses_edited_origin_path() {
    let dir = tempfile::tempdir().unwrap();
    let p = edited(dir.path(), |b| {
        b["origin_path"] = Value::from("m/48'/1'/7'/2'")
    });
    assert_refused(&bundle(&p, "signet"), &e2_key("--import-json", 0), "bundle");
}

/// Same tpub and origin path as cosigner 0, different master fingerprint: the
/// only field that differs between the spliced card and the descriptor key.
fn fingerprint_only_other_card() -> Value {
    let other = import(
        &PLAIN.replace("@0", &K0.replace("73c5da0a", "deadbeef")),
        false,
    );
    other[0]["bundle"]["mk1"][0].clone()
}

#[test]
fn envelope_mk1_fingerprint_only_mismatch_refused() {
    let dir = tempfile::tempdir().unwrap();
    let card = fingerprint_only_other_card();
    let honest = import(PLAIN, false);
    assert_ne!(card, honest[0]["bundle"]["mk1"][0], "fixture: card differs");
    let p = edited(dir.path(), |b| b["mk1"][0] = card.clone());
    for f in FORMATS {
        assert_refused(&export(&p, f), &e2_key("--from-import-json", 0), f);
    }
    assert_refused(&bundle(&p, "signet"), &e2_key("--import-json", 0), "bundle");
}

#[test]
fn envc_export_refuses_missing_mk1_card() {
    let dir = tempfile::tempdir().unwrap();
    let p = edited(dir.path(), |b| {
        b["mk1"].as_array_mut().unwrap().truncate(2);
    });
    for f in FORMATS {
        assert_refused(&export(&p, f), &e2_count("--from-import-json", 2, 3), f);
    }
}

#[test]
fn envc_bundle_refuses_missing_mk1_card() {
    let dir = tempfile::tempdir().unwrap();
    let p = edited(dir.path(), |b| {
        b["mk1"].as_array_mut().unwrap().truncate(2);
        b["ms1"].as_array_mut().unwrap().truncate(2);
    });
    assert_refused(
        &bundle(&p, "signet"),
        &e2_count("--import-json", 2, 3),
        "bundle",
    );
}

#[test]
fn envc_bundle_refuses_foreign_ms1() {
    let dir = tempfile::tempdir().unwrap();
    let mut v = import(PLAIN, true);
    assert!(
        v[0]["bundle"]["ms1"][0]
            .as_str()
            .unwrap()
            .starts_with("ms1"),
        "fixture: seed envelope"
    );
    v[0]["bundle"]["ms1"][0] = Value::from(MS1_ZOO);
    let p = write(dir.path(), "seed-edited.json", &v);
    assert_refused(&bundle(&p, "signet"), &e3(0), "bundle");
}

#[test]
fn envc_template_md1_envelope_refused() {
    // A hand-wrapped `bundle --md1-form template --json` (not an import-wallet
    // envelope): its md1 is the keyless template, not the descriptor's md1.
    let dir = tempfile::tempdir().unwrap();
    let d = "wpkh([73c5da0a/84'/1'/0']tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M/<0;1>/*)";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "signet",
            "--descriptor",
            d,
            "--md1-form",
            "template",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let b: Value = serde_json::from_slice(&out.stdout).unwrap();
    let env = Value::Array(vec![serde_json::json!({
        "schema_version": "1", "source_format": "descriptor", "bundle": b
    })]);
    let p = write(dir.path(), "wrapped.json", &env);
    assert_refused(&bundle(&p, "signet"), &e1("--import-json"), "bundle");
    assert_refused(
        &export(&p, "descriptor"),
        &e1("--from-import-json"),
        "export",
    );
}

#[test]
fn envc_honest_envelopes_pass() {
    let dir = tempfile::tempdir().unwrap();
    let fx = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wallet_import");
    let mut cases: Vec<(PathBuf, &str)> = vec![
        (
            write(dir.path(), "plain.json", &import(PLAIN, false)),
            "signet",
        ),
        (
            write(
                dir.path(),
                "one.json",
                &import("wsh(sortedmulti(1,@0,@1,@2))", false),
            ),
            "signet",
        ),
        (
            write(dir.path(), "seed.json", &import(PLAIN, true)),
            "signet",
        ),
        (
            write(
                dir.path(),
                "general.json",
                &import("wsh(or_d(pk(@2),multi(2,@0,@1)))", false),
            ),
            "signet",
        ),
    ];
    cases.push((fx.join("envelope_v0_27_0.json"), "mainnet"));
    cases.push((fx.join("envelope_v0_27_0_as_released.json"), "mainnet"));
    for (p, net) in &cases {
        let b = bundle(p, net);
        assert_eq!(
            b.status.code(),
            Some(0),
            "bundle {p:?}: {}",
            String::from_utf8_lossy(&b.stderr)
        );
        let e = export(p, "descriptor");
        assert_eq!(
            e.status.code(),
            Some(0),
            "export {p:?}: {}",
            String::from_utf8_lossy(&e.stderr)
        );
    }
}
