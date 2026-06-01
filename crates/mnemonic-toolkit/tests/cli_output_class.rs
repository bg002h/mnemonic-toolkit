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
// P2: bundle — PrivateKeyMaterial (seed) vs WatchOnly (--descriptor)
// ============================================================

#[test]
fn bundle_seed_emits_private_key_material() {
    let slot = format!("@0.phrase={ABANDON}");
    let o = mnemonic()
        .args([
            "bundle",
            "--slot",
            &slot,
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

#[test]
fn bundle_descriptor_emits_watch_only() {
    let o = mnemonic()
        .args([
            "bundle",
            "--descriptor",
            "wpkh([704c7836/84h/0h/0h]tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/0/*)",
            "--network",
            "testnet",
            "--no-engraving-card",
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(W_LINE), "{}", stderr(&o));
}

// ============================================================
// P2: convert — PrivateKeyMaterial (xprv), WatchOnly (xpub), inert (path)
// ============================================================

#[test]
fn convert_to_xprv_emits_private_key_material() {
    let from_arg = format!("phrase={ABANDON}");
    let s = stderr(
        &mnemonic()
            .args(["convert", "--from", &from_arg, "--to", "xprv", "--template", "bip84"])
            .output()
            .unwrap(),
    );
    assert!(s.contains(P_LINE), "{s}");
}

#[test]
fn convert_to_xpub_emits_watch_only() {
    let from_arg = format!("phrase={ABANDON}");
    let w = stderr(
        &mnemonic()
            .args(["convert", "--from", &from_arg, "--to", "xpub", "--template", "bip84"])
            .output()
            .unwrap(),
    );
    assert!(w.contains(W_LINE), "{w}");
}

#[test]
fn convert_to_path_only_is_inert() {
    let from_arg = format!("phrase={ABANDON}");
    let s = stderr(
        &mnemonic()
            .args(["convert", "--from", &from_arg, "--to", "path", "--template", "bip84"])
            .output()
            .unwrap(),
    );
    assert!(
        !s.contains("note: stdout") && !s.contains("warning: stdout carries"),
        "path-only must be inert: {s}"
    );
}

// ============================================================
// P2: repair — PrivateKeyMaterial (ms1) vs WatchOnly (mk1)
// ============================================================

const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const VALID_MK1_C0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
const VALID_MK1_C1: &str = "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

#[test]
fn repair_ms1_emits_private_key_material() {
    let o = mnemonic()
        .args(["repair", "--ms1", VALID_MS1])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

#[test]
fn repair_mk1_emits_watch_only() {
    let o = mnemonic()
        .args(["repair", "--mk1", VALID_MK1_C0, "--mk1", VALID_MK1_C1])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(W_LINE), "{}", stderr(&o));
}

// ============================================================
// P2: inspect — PrivateKeyMaterial (ms1) vs WatchOnly (mk1)
// ============================================================

#[test]
fn inspect_ms1_emits_private_key_material() {
    let o = mnemonic()
        .args(["inspect", "--ms1", VALID_MS1])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

#[test]
fn inspect_mk1_emits_watch_only() {
    let o = mnemonic()
        .args(["inspect", "--mk1", VALID_MK1_C0, "--mk1", VALID_MK1_C1])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(W_LINE), "{}", stderr(&o));
}

// ============================================================
// P2: nostr — WatchOnly (npub) vs PrivateKeyMaterial (nsec)
// ============================================================

const NPUB: &str = "npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg";
const NSEC: &str = "nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5";

#[test]
fn nostr_npub_emits_watch_only() {
    let o = mnemonic()
        .args(["nostr", "--pubkey", NPUB])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(W_LINE), "{}", stderr(&o));
}

#[test]
fn nostr_nsec_emits_private_key_material() {
    let o = mnemonic()
        .args(["nostr", "--secret-stdin", "--script-type", "p2wpkh"])
        .write_stdin(format!("{NSEC}\n"))
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// P2: import-wallet — PrivateKeyMaterial (with --ms1) vs WatchOnly (watch-only)
// ============================================================

/// Minimal BSMS 2-line blob for the abandon×23+art phrase xpub at m/48'/0'/0'/2'
/// (1-of-1 wsh(sortedmulti)). Constants match cli_import_wallet_seed_overlay.rs.
const IW_FP: &str = "5436d724";
const IW_XPUB_BIP48: &str = "xpub6E79FaRWLSJCAgA2jDHRvyrWKwT6aSmR685zptzyYPvmUd44omcxZ1NAzDtbdFBvEADjcVbV4NzTDwQeU6oiSV9KGiMSWhjANZjbfUHkm3Y";
/// ms1-encoded 32-zero entropy (abandon×23+art).
const IW_MS1: &str =
    "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w";

fn bsms_1of1_blob() -> String {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let body = format!(
        "wsh(sortedmulti(1,[{IW_FP}/48'/0'/0'/2']{IW_XPUB_BIP48}/<0;1>/*))"
    );
    let mut e = ChecksumEngine::new();
    e.input(&body).expect("checksum input must be ASCII");
    let csum = e.checksum();
    format!("BSMS 1.0\n{body}#{csum}\n")
}

#[test]
fn import_wallet_with_ms1_overlay_emits_private_key_material() {
    let blob = bsms_1of1_blob();
    let o = mnemonic()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            IW_MS1,
        ])
        .write_stdin(blob)
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

#[test]
fn import_wallet_watch_only_emits_watch_only() {
    let blob = bsms_1of1_blob();
    let o = mnemonic()
        .args(["import-wallet", "--blob", "-", "--format", "bsms"])
        .write_stdin(blob)
        .output()
        .unwrap();
    assert!(stderr(&o).contains(W_LINE), "{}", stderr(&o));
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
