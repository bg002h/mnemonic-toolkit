//! FIX-V5 (`design/FIX_v5_wsh_cost.md`): rows for the rulings the triage file
//! `cli_compare_cost_wsh_witness_script.rs` does not cover.
//!
//! - Ruling 4: a `--descriptor sh(wsh(M))` input adds one note to `notes[]`
//!   (JSON) and one `note:` line (text). Only containment is pinned here; the
//!   note's position among the other notes is not.
//! - Triage V5 E6: once the witnessScript is counted, a large k-of-n multisig
//!   is cheaper in tr than in wsh (negative Δ). On the unmodified code every
//!   Δ is positive, so the column misleads in the one case the tool exists to
//!   answer.

use assert_cmd::Command;
use serde_json::Value;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

const KEY_A: &str = "02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9";
const KEY_X_A: &str = "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9";
const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Ruling 4 note text, exactly.
const SH_WSH_NOTE: &str = "input had a p2sh wrapper, sh(wsh(M)); each wsh figure is for bare wsh(M): add 35 vB per input (the scriptSig redeem-script push) and subtract 35 vB from each Δ";

fn run_ok(args: &[&str]) -> String {
    let out = bin().args(args).output().unwrap();
    assert!(
        out.status.success(),
        "mnemonic {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn json_notes(flag: &str, input: &str) -> Vec<String> {
    let s = run_ok(&["compare-cost", flag, input, "--json"]);
    let v: Value = serde_json::from_str(&s).unwrap();
    v["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().to_string())
        .collect()
}

/// delta_vbytes per row of `--miniscript <m> --json`.
fn deltas(m: &str) -> Vec<i64> {
    let s = run_ok(&["compare-cost", "--miniscript", m, "--json"]);
    let v: Value = serde_json::from_str(&s).unwrap();
    v["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["delta_vbytes"].as_i64().unwrap())
        .collect()
}

#[test]
fn v5_sh_wsh_note_in_json_notes() {
    let notes = json_notes("--descriptor", &format!("sh(wsh(pk({KEY_A})))"));
    assert!(
        notes.iter().any(|n| n == SH_WSH_NOTE),
        "notes[] must contain the ruling-4 sh(wsh) note exactly; got {notes:?}"
    );
}

#[test]
fn v5_sh_wsh_note_in_text_note_lines() {
    let s = run_ok(&[
        "compare-cost",
        "--descriptor",
        &format!("sh(wsh(pk({KEY_A})))"),
    ]);
    let want = format!("note: {SH_WSH_NOTE}");
    assert!(
        s.lines().any(|l| l == want),
        "text output must carry the line {want:?}; got:\n{s}"
    );
}

#[test]
fn v5_sh_wsh_note_absent_for_wsh_tr_and_miniscript() {
    for (flag, input) in [
        ("--descriptor", format!("wsh(pk({KEY_A}))")),
        ("--descriptor", format!("tr({NUMS},pk({KEY_X_A}))")),
        ("--miniscript", "pk(A)".to_string()),
    ] {
        let notes = json_notes(flag, &input);
        assert!(
            !notes.iter().any(|n| n.contains("p2sh wrapper")),
            "{flag} {input}: the sh(wsh) note must not appear; got {notes:?}"
        );
    }
}

#[test]
fn v5_delta_negative_for_multi_7_of_10() {
    // After the fix: wsh 256 vB, tr 251 vB, so Δ = -5 for every one of the 120
    // signer sets. Control: 2-of-3 stays tr-dearer (Δ +4).
    let big = deltas("multi(7,A,B,C,D,E,F,G,H,I,J)");
    assert_eq!(big.len(), 120, "7-of-10 has C(10,7) = 120 signer sets");
    assert!(
        big.iter().all(|d| *d < 0),
        "7-of-10: tr must be cheaper than wsh (Δ < 0) once the witnessScript is counted; got {big:?}"
    );
    let small = deltas("multi(2,A,B,C)");
    assert!(
        small.iter().all(|d| *d > 0),
        "2-of-3: wsh must stay cheaper (Δ > 0); got {small:?}"
    );
}
