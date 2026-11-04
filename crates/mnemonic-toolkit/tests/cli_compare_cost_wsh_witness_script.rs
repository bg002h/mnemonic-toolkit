//! Triage V5 (2026-11-03): `mnemonic compare-cost` omits the wsh witness
//! script from the `wsh vB` column.
//!
//! Oracle: Bitcoin Core 31.1 on regtest. Each expected figure below is a REAL
//! signed spend of a P2WSH / P2TR output (`utxoupdatepsbt` -> `walletprocesspsbt`
//! -> `finalizepsbt`), input weight = 4 x (36 outpoint + 1 + scriptSig + 4
//! sequence) + serialized witness bytes, input vB = ceil(weight / 4). Core signs
//! with low-R 71-byte (incl. sighash) ECDSA sigs; the tool's dummy ECDSA sig is
//! 73 bytes, so a multi-sig wsh row may sit 1 vB above Core (the tool's own
//! note allows +-1). The oracle script is not committed (it needs a bitcoind);
//! the measured values are pinned here:
//!
//! | spend                              | Core input WU | Core vB |
//! |------------------------------------|---------------|---------|
//! | wsh(pk(K0))                        | 273           | 69      |
//! | wsh(or_d(pk(K0),pk(K1))), via K0   | 311           | 78      |
//! | wsh(or_d(pk(K0),pk(K1))), via K1   | 312           | 78      |
//! | wsh(multi(2,K0,K1,K2)), K0+K1      | 416           | 104     |
//! | wsh(multi(3,K0..K4)), K0+K1+K2     | 556           | 139     |
//! | sh(wsh(pk(K0)))                    | 413           | 104     |
//! | tr(NUMS,pk(K0))                    | 299           | 75      |
//! | tr(NUMS,or_d(pk(K0),pk(K1))), K0   | 336           | 84      |
//! | tr(NUMS,or_d(pk(K0),pk(K1))), K1   | 337           | 85      |
//! | tr(NUMS,multi_a(2,K0,K1,K2)), K0+K1| 435           | 109     |
//! | tr(NUMS,multi_a(3,K0..K4)), 3 keys | 569           | 143     |
//!
//! The tr column already matches Core (control rows: GREEN before and after
//! the fix); the wsh column is short by varint(len)+len of the witnessScript.

use assert_cmd::Command;
use serde_json::Value;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

/// (label, wsh_vbytes, tr_vbytes, delta_vbytes) per row of `--json` output.
fn rows(flag: &str, input: &str) -> Vec<(String, i64, i64, i64)> {
    let out = bin()
        .args(["compare-cost", flag, input, "--json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "compare-cost {input} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    v["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["label"].as_str().unwrap().to_string(),
                r["wsh_vbytes"].as_i64().unwrap(),
                r["tr_vbytes"].as_i64().unwrap(),
                r["delta_vbytes"].as_i64().unwrap(),
            )
        })
        .collect()
}

fn ms_rows(m: &str) -> Vec<(String, i64, i64, i64)> {
    rows("--miniscript", m)
}

const KEY_A: &str = "02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9";
const KEY_X_A: &str = "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9";
const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

// ---- RED rows: wsh column vs Core -----------------------------------------

#[test]
fn v5_wsh_pk_vbytes_match_core() {
    let r = ms_rows("pk(A)");
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].1, 69, "Core spends wsh(pk) as 273 WU = 69 vB; got {r:?}");
}

#[test]
fn v5_wsh_or_d_vbytes_match_core() {
    let r = ms_rows("or_d(pk(A),pk(B))");
    assert_eq!(r.len(), 2);
    for row in &r {
        assert!(
            (78..=79).contains(&row.1),
            "Core: both or_d branches are 78 vB (+1 for the tool's 73-byte dummy sig); got {r:?}"
        );
    }
}

#[test]
fn v5_wsh_multi_2of3_vbytes_match_core() {
    let r = ms_rows("multi(2,A,B,C)");
    assert_eq!(r.len(), 3);
    for row in &r {
        assert!(
            (104..=105).contains(&row.1),
            "Core 104 vB (+1 for the tool's 73-byte dummy sig); got {r:?}"
        );
    }
}

#[test]
fn v5_wsh_multi_3of5_vbytes_match_core() {
    let r = ms_rows("multi(3,A,B,C,D,E)");
    assert_eq!(r.len(), 10);
    for row in &r {
        assert!(
            (139..=141).contains(&row.1),
            "Core 139 vB (+<=2 for 73-byte dummy sigs); got {r:?}"
        );
    }
}

#[test]
fn v5_descriptor_wsh_pk_vbytes_match_core() {
    let r = rows("--descriptor", &format!("wsh(pk({KEY_A}))"));
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].1, 69, "--descriptor wsh(pk(..)) must equal the Core spend; got {r:?}");
}

#[test]
fn v5_p2wsh_input_never_cheaper_than_p2wpkh_input() {
    // A P2WPKH input is 68 vB; any P2WSH spend carries a witnessScript plus at
    // least one signature, so it cannot be cheaper.
    for m in [
        "pk(A)",
        "or_d(pk(A),pk(B))",
        "and_v(v:pk(A),pk(B))",
        "multi(2,A,B,C)",
    ] {
        for row in ms_rows(m) {
            assert!(
                row.1 >= 68,
                "{m}: wsh vB {} is below the 68 vB P2WPKH input",
                row.1
            );
        }
    }
}

#[test]
fn v5_delta_matches_core() {
    // The tool's note says "Δ values are correct". Core: tr 75 - wsh 69 = +6.
    let r = ms_rows("pk(A)");
    assert_eq!(r[0].3, 6, "Δ vB for pk(A) must be 75 - 69 = 6; got {r:?}");
    // multi(2,A,B,C): Core tr 109 - wsh 104 = +5 (tool's 73-byte sigs: +4).
    let r = ms_rows("multi(2,A,B,C)");
    for row in &r {
        assert!((4..=5).contains(&row.3), "Δ vB for 2-of-3 must be 4..=5; got {r:?}");
    }
}

// ---- GREEN control rows: tr column already matches Core -------------------

#[test]
fn v5_tr_pk_vbytes_match_core() {
    let r = ms_rows("pk(A)");
    assert_eq!(r[0].2, 75);
}

#[test]
fn v5_tr_or_d_vbytes_match_core() {
    let r = ms_rows("or_d(pk(A),pk(B))");
    assert_eq!(r.iter().find(|x| x.0 == "A").unwrap().2, 84);
    assert_eq!(r.iter().find(|x| x.0 == "B").unwrap().2, 85);
}

#[test]
fn v5_tr_multi_a_vbytes_match_core() {
    let r = ms_rows("multi(2,A,B,C)");
    for row in &r {
        assert_eq!(row.2, 109, "tr(NUMS,multi_a(2,..)) is 109 vB in Core; got {r:?}");
    }
    let r = ms_rows("multi(3,A,B,C,D,E)");
    for row in &r {
        assert_eq!(row.2, 143, "tr(NUMS,multi_a(3,..5 keys)) is 143 vB in Core; got {r:?}");
    }
}

#[test]
fn v5_descriptor_tr_nums_pk_vbytes_match_core() {
    let r = rows("--descriptor", &format!("tr({NUMS},pk({KEY_X_A}))"));
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].2, 75);
}

#[test]
fn v5_sh_wsh_descriptor_reports_the_wsh_column() {
    // Documented design (SPEC_compare_cost_v0_26_0.md row `--descriptor
    // sh(wsh(M))`: "comparison ignores p2sh-redeem-script overhead"). Core
    // measures a real sh(wsh(pk)) as 104 vB (35-byte scriptSig push); the tool
    // prints the wsh figure. Control: both wrappers must give the same row.
    let a = rows("--descriptor", &format!("wsh(pk({KEY_A}))"));
    let b = rows("--descriptor", &format!("sh(wsh(pk({KEY_A})))"));
    assert_eq!((a[0].1, a[0].2), (b[0].1, b[0].2));
}
