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
        .args(["addresses", "--from", &from_arg, "--address-type", "p2wpkh"])
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
    // Cycle A Group B swap (plan-R0 M-a): `bundle --descriptor` (concrete
    // form) runs through `concrete_keys_to_placeholders` → `lex_placeholders`
    // (unlike `export-wallet --descriptor` above, which parses directly via
    // `MsDescriptor::from_str` and never touches the lexer), so the
    // incidental fixed `/0/*` step now rejects; swap to `<0;1>/*` — this
    // cell's assertion is the watch-only-mode class, orthogonal to the
    // now-separately-covered fixed-step reject.
    let o = mnemonic()
        .args([
            "bundle",
            "--descriptor",
            "wpkh([704c7836/84h/0h/0h]tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*)",
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
            .args([
                "convert",
                "--from",
                &from_arg,
                "--to",
                "xprv",
                "--template",
                "bip84",
            ])
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
            .args([
                "convert",
                "--from",
                &from_arg,
                "--to",
                "xpub",
                "--template",
                "bip84",
            ])
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
            .args([
                "convert",
                "--from",
                &from_arg,
                "--to",
                "path",
                "--template",
                "bip84",
            ])
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
const VALID_MK1_C1: &str =
    "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

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
const IW_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w";

fn bsms_1of1_blob() -> String {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let body = format!("wsh(sortedmulti(1,[{IW_FP}/48'/0'/0'/2']{IW_XPUB_BIP48}/<0;1>/*))");
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
        panic!(
            "slip39 split failed: {}",
            String::from_utf8_lossy(&split_out.stderr)
        );
    }
    let split_stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<&str> = split_stdout.lines().filter(|l| !l.is_empty()).collect();
    // Need at least 2 of the 3-member group (threshold=2).
    assert!(
        shares.len() >= 2,
        "expected >=2 shares, got: {}",
        shares.len()
    );
    let o = mnemonic()
        .args([
            "slip39", "combine", "--share", shares[0], "--share", shares[1],
        ])
        .output()
        .unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}

// ============================================================
// P3: inert commands — no advisory line on normal branch
// ============================================================

/// `decode-address` outputs address metadata (inert) — no advisory line.
#[test]
fn decode_address_is_inert() {
    let o = mnemonic()
        .args([
            "decode-address",
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        ])
        .output()
        .unwrap();
    let s = stderr(&o);
    assert!(
        !s.contains("note: stdout") && !s.contains("warning: stdout carries"),
        "decode-address must be inert: {s}"
    );
}

/// `verify-bundle` on a correct bundle (normal branch) — no advisory line.
#[test]
fn verify_bundle_normal_is_inert() {
    let bundle_json = {
        let o = mnemonic()
            .args([
                "bundle",
                "--slot",
                &format!("@0.phrase={ABANDON}"),
                "--network",
                "mainnet",
                "--template",
                "bip84",
                "--json",
                "--no-engraving-card",
            ])
            .output()
            .unwrap();
        assert!(o.status.success(), "bundle must succeed: {}", stderr(&o));
        String::from_utf8(o.stdout).unwrap()
    };
    let dir = tempfile::tempdir().unwrap();
    let bundle_path = dir.path().join("bundle.json");
    std::fs::write(&bundle_path, &bundle_json).unwrap();
    let o = mnemonic()
        .args([
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.phrase={ABANDON}"),
            "--bundle-json",
            bundle_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let s = stderr(&o);
    assert!(
        !s.contains("note: stdout") && !s.contains("warning: stdout carries"),
        "verify-bundle normal branch must be inert: {s}"
    );
}

/// `compare-cost` outputs policy cost analysis (inert) — no advisory line.
#[test]
fn compare_cost_is_inert() {
    let o = mnemonic()
        .args(["compare-cost", "--miniscript", "pk(A)"])
        .output()
        .unwrap();
    let s = stderr(&o);
    assert!(
        !s.contains("note: stdout") && !s.contains("warning: stdout carries"),
        "compare-cost must be inert: {s}"
    );
}

/// `xpub-search path-of-xpub` outputs a search result (inert) — no advisory line.
#[test]
fn xpub_search_path_of_xpub_is_inert() {
    // A no-match result (exit 4) is still a normal branch with no secret on stdout.
    let o = mnemonic()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            // a random xpub that won't match → exits 4 quickly
            "xpub661MyMwAqRbcGFkPHkfzFnYRJGxq8r6LfnEbEUvQLsxWXfxdF4tLcVEsDAZwTRVABN3czmTUGe1GHb1jCBUX7A4oeXMtKGmVPHqpSMvNks",
            "--json",
        ])
        .write_stdin(format!("{ABANDON}\n"))
        .output()
        .unwrap();
    let s = stderr(&o);
    assert!(
        !s.contains("note: stdout") && !s.contains("warning: stdout carries"),
        "xpub-search path-of-xpub must be inert: {s}"
    );
}

// ============================================================
// P3: auto-repair short-circuit emits the correct output class
// ============================================================

/// Flip a single bech32 character at position `pos` in the data part
/// (after the separator `1`). Same logic as `flip_at` in cli_auto_repair.rs.
fn flip_bech32_p3(s: &str, pos: usize) -> String {
    const ALPHA: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let sep = s.rfind('1').unwrap();
    let (pre, rest) = s.split_at(sep + 1);
    let mut chars: Vec<u8> = rest.bytes().collect();
    let idx = ALPHA.iter().position(|&b| b == chars[pos]).unwrap();
    chars[pos] = ALPHA[(idx + 1) % ALPHA.len()];
    format!("{pre}{}", String::from_utf8(chars).unwrap())
}

const AUTO_REPAIR_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const AUTO_REPAIR_MD1_C0: &str =
    "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const AUTO_REPAIR_MD1_C1: &str =
    "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const AUTO_REPAIR_MD1_C2: &str =
    "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";
const T_LINE: &str = "note: stdout is a keyless descriptor template (no keys)";

/// Cycle F (`ms1-repair-demote-to-candidate`) FLIP — was: auto-repair
/// short-circuit (inspect, 1-char-corrupt ms1) → exit 5, repaired ms1 on
/// stdout → PrivateKeyMaterial (P) advisory line. An ms1 substitution
/// correction is now a demoted candidate — `try_repair_and_short_circuit`
/// falls through, so the ORIGINAL decode error surfaces (exit 1) BEFORE
/// anything reaches stdout; consequently NO output-class advisory fires at
/// all (nothing was written to stdout to classify). ms1's P-classification
/// on the auto-repair path is no longer reachable via inspect/convert; it
/// remains covered by the STANDALONE `repair --ms1` path
/// (`repair_ms1_emits_private_key_material`, which still emits the
/// corrected card at exit 4).
#[test]
fn auto_repair_short_circuit_ms1_no_longer_applies_after_cycle_f_demotion() {
    let bad = flip_bech32_p3(AUTO_REPAIR_MS1, 17);
    let o = mnemonic()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args(["inspect", "--ms1", &bad])
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(1),
        "expected exit 1 (demoted candidate, no short-circuit): {}",
        stderr(&o)
    );
    let s = stderr(&o);
    assert!(
        !s.contains(P_LINE),
        "no output-class advisory should fire — nothing was written to stdout: {s}"
    );
}

/// Auto-repair short-circuit (inspect, 1-char-corrupt md1) → exit 5,
/// repaired md1 on stdout → Template (T) advisory line. [folds C1 widening]
#[test]
fn auto_repair_short_circuit_md1_emits_template() {
    let bad = flip_bech32_p3(AUTO_REPAIR_MD1_C0, 20);
    let o = mnemonic()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args([
            "inspect",
            "--md1",
            &bad,
            "--md1",
            AUTO_REPAIR_MD1_C1,
            "--md1",
            AUTO_REPAIR_MD1_C2,
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(5), "expected exit 5: {}", stderr(&o));
    let s = stderr(&o);
    assert!(
        s.contains(T_LINE),
        "md1 auto-repair must emit T advisory: {s}"
    );
}

/// Cycle F FLIP (mirrors the inspect cell above) — was: auto-repair
/// short-circuit (convert, 1-char-corrupt ms1) → exit 5, repaired ms1 on
/// stdout → PrivateKeyMaterial advisory line. Now: demoted candidate falls
/// through, the original decode error surfaces (exit 1), no stdout, no
/// output-class advisory.
#[test]
fn auto_repair_short_circuit_convert_ms1_no_longer_applies_after_cycle_f_demotion() {
    let bad = flip_bech32_p3(AUTO_REPAIR_MS1, 17);
    let o = mnemonic()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args(["convert", "--from", &format!("ms1={bad}"), "--to", "phrase"])
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(1),
        "expected exit 1 (demoted candidate, no short-circuit): {}",
        stderr(&o)
    );
    let s = stderr(&o);
    assert!(
        !s.contains(P_LINE),
        "no output-class advisory should fire — nothing was written to stdout: {s}"
    );
}

// ============================================================
// P3: file-output suppression — --json-out → no stdout-class line
// ============================================================

/// `seedqr encode --json-out <file>` writes to file, not stdout → no advisory.
#[test]
fn seedqr_jsonout_file_is_inert() {
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
    let s = stderr(&o);
    assert!(
        !s.contains("warning: stdout carries"),
        "file-output → no stdout-class line: {s}"
    );
}
