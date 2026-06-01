//! Cycle B — output-type stderr advisory: per-command class assertions.
//! P1 cells: fixed-class commands (derive-child, silent-payment,
//! electrum-decrypt, seedqr-encode, seedqr-decode, addresses,
//! export-wallet, final-word, seed-xor-split, seed-xor-combine,
//! slip39-split, slip39-combine) + TTY-gate removal regression cell.
use assert_cmd::Command;

const P_LINE: &str = "warning: stdout carries private key material (can spend)";
const W_LINE: &str = "note: stdout is watch-only — public keys only, cannot spend";
const ABANDON: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// 11-word partial for final-word (valid 11-word partial → 12-word target)
const ABANDON_11: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
/// SeedQR digits encoding the abandon×11+about phrase (48 digits = 12 words)
const ABANDON_QR: &str = "000000000000000000000000000000000000000000000003";
/// Electrum test vector (from cli_electrum_decrypt.rs TV_CIPHERTEXT)
const TV_CIPHERTEXT: &str = "ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE=";
const TV_PASSWORD: &str = "test-password";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}
fn stderr(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stderr).into()
}

// ============================================================
// derive-child → PrivateKeyMaterial
// ============================================================

#[test]
fn derive_child_emits_private_key_material() {
    let from_arg = format!("phrase={ABANDON}");
    let o = mnemonic()
        .args([
            "derive-child",
            "--from",
            &from_arg,
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// silent-payment → PrivateKeyMaterial
// ============================================================

#[test]
fn silent_payment_emits_private_key_material() {
    let o = mnemonic()
        .args(["silent-payment", "--secret", ABANDON])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// electrum-decrypt → PrivateKeyMaterial (stdout branch only)
// ============================================================

#[test]
fn electrum_decrypt_emits_private_key_material() {
    let o = mnemonic()
        .args([
            "electrum-decrypt",
            "--ciphertext",
            TV_CIPHERTEXT,
            "--decrypt-password",
            TV_PASSWORD,
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// seedqr encode → PrivateKeyMaterial
// ============================================================

#[test]
fn seedqr_encode_emits_private_key_material() {
    let from_arg = format!("phrase={ABANDON}");
    let o = mnemonic()
        .args(["seedqr", "encode", "--from", &from_arg])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

#[test]
fn seedqr_encode_json_out_no_private_key_material_line() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("q.json");
    let from_arg = format!("phrase={ABANDON}");
    let o = mnemonic()
        .args([
            "seedqr",
            "encode",
            "--from",
            &from_arg,
            "--json-out",
            p.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        !stderr(&o).contains("warning: stdout carries"),
        "file-output → no stdout-class line: {}",
        stderr(&o)
    );
}

// ============================================================
// seedqr decode → PrivateKeyMaterial
// ============================================================

#[test]
fn seedqr_decode_emits_private_key_material() {
    let from_arg = format!("seedqr={ABANDON_QR}");
    let o = mnemonic()
        .args(["seedqr", "decode", "--from", &from_arg])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// addresses → WatchOnly
// ============================================================

#[test]
fn addresses_emits_watch_only() {
    let from_arg = format!("phrase={ABANDON}");
    let o = mnemonic()
        .args([
            "addresses",
            "--from",
            &from_arg,
            "--address-type",
            "p2wpkh",
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(W_LINE), "{}", stderr(&o));
}

// ============================================================
// export-wallet → WatchOnly
// ============================================================

#[test]
fn export_wallet_emits_watch_only() {
    // export-wallet with a concrete descriptor → watch-only wallet file.
    let o = mnemonic()
        .args([
            "export-wallet",
            "--descriptor",
            "wpkh([704c7836/84h/0h/0h]tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/0/*)",
            "--network",
            "testnet",
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(W_LINE), "{}", stderr(&o));
}

// ============================================================
// final-word → PrivateKeyMaterial
// ============================================================

#[test]
fn final_word_emits_private_key_material() {
    let from_arg = format!("phrase={ABANDON_11}");
    let o = mnemonic()
        .args(["final-word", "--from", &from_arg])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// seed-xor split → PrivateKeyMaterial
// ============================================================

#[test]
fn seed_xor_split_emits_private_key_material() {
    let from_arg = format!("phrase={ABANDON}");
    let o = mnemonic()
        .args([
            "seed-xor",
            "split",
            "--from",
            &from_arg,
            "--shares",
            "2",
            "--deterministic-from-master",
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// seed-xor combine → PrivateKeyMaterial
// ============================================================

#[test]
fn seed_xor_combine_emits_private_key_material() {
    // First get shares deterministically.
    let from_arg = format!("phrase={ABANDON}");
    let split_out = mnemonic()
        .args([
            "seed-xor",
            "split",
            "--from",
            &from_arg,
            "--shares",
            "2",
            "--deterministic-from-master",
        ])
        .output()
        .unwrap();
    let split_stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<&str> = split_stdout.lines().filter(|l| !l.is_empty()).collect();
    let s0 = format!("phrase={}", shares[0]);
    let s1 = format!("phrase={}", shares[1]);
    let o = mnemonic()
        .args([
            "seed-xor", "combine", "--share", &s0, "--share", &s1, "--shares", "2",
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// slip39 split → PrivateKeyMaterial
// TTY-gate-removal regression: piped (non-TTY) stdout still emits the P line.
// ============================================================

#[test]
fn slip39_split_emits_on_pipe_not_just_tty() {
    // TTY-gate-removal regression: piped (non-TTY) stdout still emits the P line.
    let from_arg = format!("phrase={ABANDON}");
    let o = mnemonic()
        .args([
            "slip39",
            "split",
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// slip39 combine → PrivateKeyMaterial
// ============================================================

#[test]
fn slip39_combine_emits_private_key_material() {
    let from_arg = format!("phrase={ABANDON}");
    let split_out = mnemonic()
        .args([
            "slip39",
            "split",
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    if !split_out.status.success() {
        panic!("slip39 split failed: {}", String::from_utf8_lossy(&split_out.stderr));
    }
    let split_stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<&str> = split_stdout
        .lines()
        .filter(|l| !l.is_empty())
        .collect();
    // Need at least 2 of the 3-member group (threshold=2).
    assert!(
        shares.len() >= 2,
        "expected >=2 shares, got: {}",
        shares.len()
    );
    let o = mnemonic()
        .args([
            "slip39",
            "combine",
            "--share",
            shares[0],
            "--share",
            shares[1],
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}
