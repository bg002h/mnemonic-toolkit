//! RED rows for the tk bughunt whole-diff slice-2 Importants I1-I4
//! (`design/agent-reports/whole-diff-bughunt-tk-s2.md`): each kills a
//! surviving mutant. Fixtures are test vectors, never wallet secrets.

use assert_cmd::Command;

/// 128-bit zero-entropy `ms1` card.
const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// Share 1 of a 2-of-3 split (threshold `2`).
const SHARE_1: &str = "ms12mc8pqf9tqz74mwtgw8g3y86qux3mgf6cq8skgye8ey5fl9";
const XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// Run `mnemonic <args>` with optional stdin; (exit code, stdout, stderr).
fn run(args: &[&str], stdin: Option<&str>) -> (i32, String, String) {
    let mut cmd = mnemonic();
    cmd.args(args);
    if let Some(s) = stdin {
        cmd.write_stdin(s.to_string());
    }
    let out = cmd.output().expect("spawn mnemonic");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// I1: every single-symbol substitution of a K-of-N share (any position past
/// the `ms1` + threshold + id header, any replacement) is corrected to the
/// clean share. Kills `^= m` -> `|= m` in repair.rs.
#[test]
fn s2_red_x3_every_single_substitution_of_a_share_is_corrected() {
    let mut failures: Vec<String> = Vec::new();
    for pos in 4..SHARE_1.len() {
        for rep in ['q', 'p', 'z', 'x', 'l'] {
            if SHARE_1.as_bytes()[pos] as char == rep {
                continue;
            }
            let damaged = format!("{}{}{}", &SHARE_1[..pos], rep, &SHARE_1[pos + 1..]);
            let (code, stdout, stderr) =
                run(&["repair", "--ms1", "-"], Some(&format!("{damaged}\n")));
            if code != 4 || !stdout.ends_with(&format!("{SHARE_1}\n")) {
                failures.push(format!(
                    "pos {pos} -> {rep}: exit {code}; stdout {stdout:?}; stderr {stderr:?}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// I2: a corrected single-string seed card gets the seed-card VERIFY-ME text,
/// not the K-of-N share advice. Kills `ms1_is_share` -> always true.
#[test]
fn s2_red_corrected_seed_card_gets_seed_card_text() {
    let damaged = format!("{}x{}", &VALID_MS1[..21], &VALID_MS1[22..]);
    assert_ne!(damaged, VALID_MS1);
    let (code, stdout, stderr) = run(&["repair", "--ms1", "-"], Some(&format!("{damaged}\n")));
    assert_eq!(code, 4, "stderr: {stderr}");
    assert!(
        stdout.ends_with(&format!("{VALID_MS1}\n")),
        "stdout: {stdout}"
    );
    assert!(
        stderr.contains("a corrected seed card cannot be self-verified"),
        "stderr: {stderr}"
    );
    assert!(!stderr.contains("K-of-N"), "stderr: {stderr}");
}

/// I3: a digest repeated in two leaves is one preimage for every hash kind.
/// Kills deleting the Hash256 / Ripemd160 / Hash160 arms in cost/enumerate.rs.
#[test]
fn s2_red_28b_compare_cost_repeated_digest_all_kinds() {
    let kinds = [
        (
            "hash256",
            "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9",
        ),
        ("ripemd160", "5feceb66ffc86f38d952786c6d696c79c2dbc239"),
        ("hash160", "5feceb66ffc86f38d952786c6d696c79c2dbc239"),
    ];
    for (kind, h) in kinds {
        let ms = format!("or_i(and_v(v:pk(A),{kind}({h})),and_v(v:pk(B),{kind}({h})))");
        let (code, stdout, stderr) = run(&["compare-cost", "--miniscript", &ms, "--json"], None);
        assert_eq!(code, 0, "{kind}: stderr: {stderr}");
        let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let labels: Vec<String> = v["conditions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["label"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(labels, ["A + preimage(h0)", "B + preimage(h0)"], "{kind}");
        let (c8, _, e8) = run(
            &["compare-cost", "--miniscript", &ms, "--max-conditions", "8"],
            None,
        );
        assert_eq!(c8, 0, "{kind} cap 8: {e8}");
        let (c7, _, _) = run(
            &["compare-cost", "--miniscript", &ms, "--max-conditions", "7"],
            None,
        );
        assert_eq!(c7, 3, "{kind} cap 7");
    }
}

/// I4: a one-component derivation path other than `m/45'` is refused, not
/// imported as BIP-45. Kills `&&` -> `||` in coldcard_multisig.rs.
#[test]
fn s2_red_19c_single_component_non_45_path_refused() {
    let blob = format!(
        "Name: SingleComp\nPolicy: 2 of 2\nFormat: P2SH\n\n\
         Derivation: m/44'\n34A3A4F1: {XPUB_A}\n\
         Derivation: m/44'\nFF9DFBCF: {XPUB_B}\n"
    );
    let (code, stdout, stderr) = run(
        &[
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "coldcard-multisig",
        ],
        Some(&blob),
    );
    assert_eq!(code, 2, "stdout: {stdout}");
    assert_eq!(
        stderr,
        "error: import-wallet: coldcard-multisig: parse error: derivation path has only 1 \
         components; need ≥2 for BIP-48 coin-type inference\n"
    );
}
