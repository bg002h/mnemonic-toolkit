//! `mnemonic addresses --from electrum-phrase=` — Electrum native-seed address
//! derivation (FOLLOWUP `electrum-native-seed-address-derivation`).
//!
//! Vectors are Electrum's OWN end-to-end test vectors from
//! `tests/test_wallet_vertical.py` @ commit `e1099925e30d91dd033815b512f00582a8795d25`
//! (`test_electrum_seed_standard` / `_segwit` / `_segwit_passphrase`). Electrum
//! derivation: `PBKDF2-HMAC-SHA512(normalize_text(seed), b"electrum"+normalize_
//! text(passphrase), 2048)` → BIP-32 master; standard → `m/{0,1}/i` p2pkh,
//! segwit → `m/0'/{0,1}/i` p2wpkh.

use assert_cmd::Command;
use std::process::Output;

// ── Electrum standard (p2pkh) ───────────────────────────────────────────────
const STD_SEED: &str = "cycle rocket west magnet parrot shuffle foot correct salt library feed song";
const STD_RECV0: &str = "1NNkttn1YvVGdqBW4PR6zvc3Zx3H5owKRf";
const STD_CHANGE0: &str = "1KSezYMhAJMWqFbVFB2JshYg69UpmEXR4D";

// ── Electrum segwit (p2wpkh) ────────────────────────────────────────────────
const SW_SEED: &str = "bitter grass shiver impose acquire brush forget axis eager alone wine silver";
const SW_RECV0: &str = "bc1q3g5tmkmlvxryhh843v4dz026avatc0zzr6h3af";
const SW_CHANGE0: &str = "bc1qdy94n2q5qcp0kg7v9yzwe6wvfkhnvyzje7nx2p";

// ── Electrum segwit + UNICODE_HORROR passphrase (normalization torture) ──────
const SW_PP_RECV0: &str = "bc1qx94dutas7ysn2my645cyttujrms5d9p57f6aam";
const SW_PP_CHANGE0: &str = "bc1qcywwsy87sdp8vz5rfjh3sxdv6rt95kujdqq38g";
// Byte-exact UNICODE_HORROR (the `UNICODE_HORROR_HEX` literal from
// test_wallet_vertical.py:32) — decoded from hex so combining marks / emoji are
// not corrupted by copy-paste. Exercises the NFKD/lower/CJK normalization path.
const UNICODE_HORROR_HEX: &str = "e282bf20f09f988020f09f98882020202020e3818620e38191e3819fe381be20e3828fe3828b2077cda2cda2cd9d68cda16fcda2cda120ccb8cda26bccb5cd9f6eccb4cd98c7ab77ccb8cc9b73cd9820cc80cc8177cd98cda2e1b8a9ccb561d289cca1cda27420cca7cc9568cc816fccb572cd8fccb5726f7273cca120ccb6cda1cda06cc4afccb665cd9fcd9f20ccb6cd9d696ecda220cd8f74cc9568ccb7cca1cd9f6520cd9fcd9f64cc9b61cd9c72cc95cda16bcca2cca820cda168ccb465cd8f61ccb7cca2cca17274cc81cd8f20ccb4ccb7cda0c3b2ccb5ccb666ccb82075cca7cd986ec3adcc9bcd9c63cda2cd8f6fccb7cd8f64ccb8cda265cca1cd9d3fcd9e";

// 2FA seed (version 101) — must be refused.
const STD_2FA_SEED: &str = "science dawn member doll dutch real can brick knife deny drive list";

fn unicode_horror() -> String {
    let bytes: Vec<u8> = (0..UNICODE_HORROR_HEX.len() / 2)
        .map(|i| u8::from_str_radix(&UNICODE_HORROR_HEX[i * 2..i * 2 + 2], 16).unwrap())
        .collect();
    String::from_utf8(bytes).unwrap()
}

fn mn(args: &[&str]) -> Output {
    Command::cargo_bin("mnemonic").unwrap().args(args).output().unwrap()
}
fn stdout(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}
fn stderr(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}
fn code(o: &Output) -> i32 {
    o.status.code().unwrap()
}

#[test]
fn electrum_standard_p2pkh_vector() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("electrum-phrase={STD_SEED}"),
        "--address-type",
        "p2pkh",
        "--chain",
        "both",
        "--count",
        "1",
    ]);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains(STD_RECV0), "receive[0] {STD_RECV0} in:\n{out}");
    assert!(out.contains(STD_CHANGE0), "change[0] {STD_CHANGE0} in:\n{out}");
}

#[test]
fn electrum_segwit_p2wpkh_vector() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("electrum-phrase={SW_SEED}"),
        "--address-type",
        "p2wpkh",
        "--chain",
        "both",
        "--count",
        "1",
    ]);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains(SW_RECV0), "receive[0] {SW_RECV0} in:\n{out}");
    assert!(out.contains(SW_CHANGE0), "change[0] {SW_CHANGE0} in:\n{out}");
}

#[test]
fn electrum_segwit_passphrase_unicode_horror_vector() {
    // The must-pass normalization discharge (R0 M1): UNICODE_HORROR exercises
    // the NFKD / lower / CJK-strip normalize_text path on the passphrase.
    let o = mn(&[
        "addresses",
        "--from",
        &format!("electrum-phrase={SW_SEED}"),
        "--passphrase",
        &unicode_horror(),
        "--address-type",
        "p2wpkh",
        "--chain",
        "both",
        "--count",
        "1",
    ]);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains(SW_PP_RECV0), "receive[0] {SW_PP_RECV0} in:\n{out}");
    assert!(out.contains(SW_PP_CHANGE0), "change[0] {SW_PP_CHANGE0} in:\n{out}");
}

#[test]
fn electrum_address_type_mismatch_refused_exit_1() {
    // A standard seed with --address-type p2wpkh is refused: the script type is
    // fixed by the Electrum seed version (standard → p2pkh).
    let o = mn(&[
        "addresses",
        "--from",
        &format!("electrum-phrase={STD_SEED}"),
        "--address-type",
        "p2wpkh",
        "--count",
        "1",
    ]);
    assert_eq!(code(&o), 1, "stdout: {}\nstderr: {}", stdout(&o), stderr(&o));
    assert!(
        stderr(&o).contains("fixed by the seed version"),
        "stderr: {}",
        stderr(&o)
    );
}

#[test]
fn electrum_account_refused_exit_1() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("electrum-phrase={STD_SEED}"),
        "--address-type",
        "p2pkh",
        "--account",
        "1",
        "--count",
        "1",
    ]);
    assert_eq!(code(&o), 1, "stdout: {}\nstderr: {}", stdout(&o), stderr(&o));
    assert!(stderr(&o).contains("--account"), "stderr: {}", stderr(&o));
}

#[test]
fn electrum_2fa_refused_exit_1() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("electrum-phrase={STD_2FA_SEED}"),
        "--address-type",
        "p2pkh",
        "--count",
        "1",
    ]);
    assert_eq!(code(&o), 1, "stdout: {}\nstderr: {}", stdout(&o), stderr(&o));
    assert!(stderr(&o).contains("2FA"), "stderr: {}", stderr(&o));
}

#[test]
fn electrum_watch_only_no_xpriv() {
    let o = mn(&[
        "addresses",
        "--from",
        &format!("electrum-phrase={SW_SEED}"),
        "--address-type",
        "p2wpkh",
        "--json",
        "--count",
        "1",
    ]);
    let all = format!("{}{}", stdout(&o), stderr(&o));
    assert!(
        !all.contains("xprv") && !all.contains("zprv") && !all.contains("tprv"),
        "no private key material may leak:\n{all}"
    );
}
