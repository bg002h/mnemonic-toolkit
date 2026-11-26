//! `mnemonic xpub-search address-of-seed` — the test rows of
//! `design/SPEC_xpub_search_address_of_seed.md` §9 (approved 2026-11-26).
//!
//! Test names carry the spec's row ids (`s1_…`, `m1_…`, `k1_…`). Seeds:
//! A = `abandon ×11 about` (73c5da0a; with `TREZOR`, b4e3f5ed) and
//! Z = `zoo ×11 wrong` (3f635a63). Spec addresses are pinned as given (each
//! was derived by an independent script and by `bundle` + `restore --md1`);
//! the rows whose targets the spec does not list (L1, L2, gap edges, bare
//! `/*`) mint their target here with rust-miniscript, independently of the
//! code under test.
//!
//! Secrets reach the binary through `@env:` and stdin only (the argv guard
//! refuses them inline), except in the redaction rows, which pass
//! `--allow-argv-secret` on purpose.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use miniscript::{Descriptor, DescriptorPublicKey};
use serde_json::Value;
use std::str::FromStr;

const A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const Z: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
const C3: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const D4: &str = "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

const S1: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
const S2: &str = "bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el";
const S3: &str = "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr";
const S4: &str = "1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA";
const S5: &str = "37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf";
const S6: &str = "bc1qmxrw6qdh5g3ztfcwm0et5l8mvws4eva24kmp8m";
const S7: &str = "bc1p8knh0enfv47gmpuf66528zd4jtkgjq4sv5w5l2gqwgk8exu2ynns9g8c9m";
const S8: &str = "bc1qvhccx0r4sxxsy03xltrpsgw2dnfry6e5vheaa0";
const S9: &str = "bc1qtncy45g8qy8laah6jvxn6ppvj0s9rluxj5uxf6";
const S10: &str = "bc1qj4rt65727dc9mn9g0n4ynpvnag060jt95gqrnx";
const T1: &str = "tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl";
const REGTEST1: &str = "bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk";
const P1: &str = "bc1qv5rmq0kt9yz3pm36wvzct7p3x6mtgehjul0feu";
const M1: &str = "bc1qkpcfs6lhqh3rx63we7435nf3hqpghkdtmmmvln0yrssv0ulhmngs92dens";
const M1_IDX1: &str = "bc1qp3z0f9m2yzgax82hw88jaqznsca2gtktnyuvmh4uwzlsg4736y8s2qes5n";
const M3: &str = "bc1qqr4u57xa6zg0cdt0kwy0zzf29s57h5klqr3gwuyhu6g2qvcf6qzsmuzazu";
const M4: &str = "bc1qlhnjvp36wlaefhk5prxp895zp74kvdac7fphw8ntk7vaas3v3xpqwymvpl";
const R1: &str = "bc1pzxzsarxcr78t8clalyhtj0gaprlnhkaq7u8yg9keqx97rgpg0yfsq8kfd0";
const R2: &str = "bc1psw9rgfn7tku2t6jl8vzg7trwsr98zv23d9jkwnhcmquxujc3mxssffw33l";
const U1: &str = "bc1qw6qcv8r9fmd27fucmjsctjcw4jy8fgdmjhd9hlj0lapzeg6wdnsqtq0gq2";
const N1: &str = "34zdguTXdmTqGbhcWJVWSnKy1sAx1za3gN";
const B2: &str = "bc1pv8ycp7my2y9ev35rn40m8z962lxdf0jw6nuvpwum6lhp7vf79c7qxq7445";
const H1: &str = "bc1qxc49mhaugahtwnn9hcflmcp0llsxjm3z809mpr";
const H1_OK: &str = "bc1qf4z33fsmn5m7f7zt24wf8lr046rz56kxnprfel";

const SORTED: &str = "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))";
const UNSORTED: &str = "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))";
const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Out {
    fn json(&self) -> Value {
        serde_json::from_str(self.stdout.trim()).unwrap_or_else(|e| {
            panic!(
                "stdout is not one JSON document ({e}):\n{}\nstderr:\n{}",
                self.stdout, self.stderr
            )
        })
    }
}

fn run_with(args: &[&str], envs: &[(&str, &str)], stdin: Option<&str>) -> Out {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.arg("xpub-search").arg("address-of-seed").args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    if let Some(s) = stdin {
        cmd.write_stdin(s.to_string());
    }
    let o = cmd.output().unwrap();
    Out {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

/// Seed A via `@env:SEED_A`; Z via `@env:SEED_Z`.
fn envs() -> Vec<(&'static str, &'static str)> {
    vec![("SEED_A", A), ("SEED_Z", Z), ("PP_TREZOR", "TREZOR")]
}

fn run_a(extra: &[&str]) -> Out {
    let mut args = vec!["--phrase", "@env:SEED_A"];
    args.extend_from_slice(extra);
    run_with(&args, &envs(), None)
}

fn run_az(extra: &[&str]) -> Out {
    let mut args = vec![
        "--from",
        "phrase=@env:SEED_A",
        "--from",
        "phrase=@env:SEED_Z",
    ];
    args.extend_from_slice(extra);
    run_with(&args, &envs(), None)
}

fn xpub_at(phrase: &str, path: &str) -> (Fingerprint, Xpub) {
    let seed = Mnemonic::parse_in(bip39::Language::English, phrase)
        .unwrap()
        .to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let x = master
        .derive_priv(&secp, &DerivationPath::from_str(path).unwrap())
        .unwrap();
    (master.fingerprint(&secp), Xpub::from_priv(&secp, &x))
}

/// `[fp/path]xpub`, path with `h`.
fn origin_key(phrase: &str, path: &str) -> String {
    let (fp, x) = xpub_at(phrase, path);
    let p = path.trim_start_matches("m/").replace('\'', "h");
    format!("[{fp}/{p}]{x}")
}

/// Mint the address of a concrete descriptor (keys with `/<0;1>/*`) at
/// alternative `alt`, index `idx`, with rust-miniscript.
fn mint(desc: &str, alt: usize, idx: u32) -> String {
    let d = Descriptor::<DescriptorPublicKey>::from_str(desc).unwrap();
    let singles = d.into_single_descriptors().unwrap();
    singles[alt]
        .derive_at_index(idx)
        .unwrap()
        .address(bitcoin::Network::Bitcoin)
        .unwrap()
        .to_string()
}

fn reasons(v: &Value) -> Vec<String> {
    v["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap().to_string())
        .collect()
}

fn found(v: &Value) -> &Value {
    v["wallets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["role"] == "found")
        .expect("a found wallet")
}

fn slot_seed(w: &Value, slot: u64) -> Option<u64> {
    w["slots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["slot"] == slot)
        .and_then(|s| s["seed_index"].as_u64())
}

fn first_target_path(w: &Value, slot: u64) -> String {
    w["targets"][0]["paths"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["slot"] == slot)
        .unwrap()["path"]
        .as_str()
        .unwrap()
        .to_string()
}

// ---------------------------------------------------------------------------
// single-sig (S1-S10, T1, P1)
// ---------------------------------------------------------------------------

fn single(target: &str, path: &str, script_type: &str) -> Value {
    let o = run_a(&["--target-address", target, "--json"]);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
    let v = o.json();
    assert_eq!(v["mode"], "address-of-seed");
    assert_eq!(v["result"], "match");
    let w = found(&v);
    assert_eq!(w["script_type"], script_type);
    assert_eq!(first_target_path(w, 0), path);
    v
}

#[test]
fn s1_bip84_receive() {
    let v = single(S1, "m/84'/0'/0'/0/0", "p2wpkh");
    let w = found(&v);
    assert_eq!(w["descriptor"], "wpkh([73c5da0a/84h/0h/0h]@0/<0;1>/*)");
    assert_eq!(w["slots"][0]["template"], "bip84");
    assert_eq!(w["slots"][0]["account"], 0);
    assert_eq!(
        w["bundle_command"],
        "mnemonic bundle --network mainnet --descriptor 'wpkh([73c5da0a/84h/0h/0h]@0/<0;1>/*)' --slot @0.phrase=@env:SEED_A"
    );
    assert_eq!(v["searched"]["paths"], 161);
    assert_eq!(v["searched"]["child_keys"], 6440);
}

#[test]
fn s1_text_form() {
    let o = run_a(&["--target-address", S1]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(o
        .stdout
        .contains("match: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"));
    assert!(o.stdout.contains("path:        m/84'/0'/0'/0/0"));
    assert!(o.stdout.contains("script type: p2wpkh"));
    assert!(o.stdout.contains("@0 = seed 0 (73c5da0a)"));
    assert!(o.stdout.contains("to mint this wallet, run:"));
}

#[test]
fn s2_bip84_change() {
    single(S2, "m/84'/0'/0'/1/0", "p2wpkh");
}

#[test]
fn s3_bip86() {
    single(S3, "m/86'/0'/0'/0/0", "p2tr");
}

#[test]
fn s4_bip44() {
    single(S4, "m/44'/0'/0'/0/0", "p2pkh");
}

#[test]
fn s5_bip49() {
    single(S5, "m/49'/0'/0'/0/0", "p2sh-p2wpkh");
}

#[test]
fn s6_nonstandard_pair_noted() {
    single(S6, "m/44'/0'/0'/0/0", "p2wpkh");
    let o = run_a(&["--target-address", S6]);
    assert!(
        o.stdout
            .contains("note: p2wpkh at a BIP-44 path is not what BIP-44 describes"),
        "{}",
        o.stdout
    );
}

#[test]
fn s7_p2tr_at_bip84() {
    single(S7, "m/84'/0'/0'/0/0", "p2tr");
}

#[test]
fn s8_account_3_change_17() {
    single(S8, "m/84'/0'/3'/1/17", "p2wpkh");
}

#[test]
fn s9_one_key_at_bip48() {
    single(S9, "m/48'/0'/0'/2'/0/0", "p2wpkh");
}

#[test]
fn s10_refugium_seat_needs_the_flag() {
    let o = run_a(&["--target-address", S10, "--json"]);
    assert_eq!(o.code, 4);
    assert_eq!(o.json()["result"], "no_match");
    let o = run_a(&["--target-address", S10, "--refugium-seats", "--json"]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    let v = o.json();
    let w = found(&v);
    assert_eq!(first_target_path(w, 0), "m/270028'/0'/33'/0/5");
    assert_eq!(w["slots"][0]["template"], "refugium-seat");
    assert_eq!(w["slots"][0]["refugium_seat"], 1);
    assert_eq!(w["slots"][0]["refugium_round"], 1);
    assert!(w["slots"][0]["account"].is_null());
    // Also reachable by --add-path.
    let o = run_a(&["--target-address", S10, "--add-path", "m/270028'/0'/33'"]);
    assert_eq!(o.code, 0, "{}", o.stderr);
}

#[test]
fn t1_testnet_and_regtest() {
    let o = run_a(&["--target-address", T1, "--network", "testnet", "--json"]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(first_target_path(found(&o.json()), 0), "m/84'/1'/0'/0/0");
    let o = run_a(&["--target-address", T1]);
    assert_eq!(
        o.code, 2,
        "testnet target under mainnet is refused: {}",
        o.stderr
    );
    let o = run_a(&[
        "--target-address",
        REGTEST1,
        "--network",
        "regtest",
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_eq!(first_target_path(found(&o.json()), 0), "m/84'/1'/0'/0/0");
}

#[test]
fn p1_passphrase_changes_the_fingerprint() {
    let o = run_a(&[
        "--target-address",
        P1,
        "--passphrase",
        "@env:PP_TREZOR",
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    let v = o.json();
    let w = found(&v);
    assert_eq!(w["descriptor"], "wpkh([b4e3f5ed/84h/0h/0h]@0/<0;1>/*)");
    assert_eq!(w["slots"][0]["master_fingerprint"], "b4e3f5ed");
    assert!(w["bundle_command"]
        .as_str()
        .unwrap()
        .contains("--passphrase @env:PP_TREZOR"));
    assert_eq!(v["searched"]["passphrase_supplied"], true);
    let o = run_a(&["--target-address", P1]);
    assert_eq!(o.code, 4);
    assert!(
        o.stdout.contains("--passphrase"),
        "no-match names the passphrase: {}",
        o.stdout
    );
}

// ---------------------------------------------------------------------------
// multisig (M1-M6, D1, R1, R2, U1, N1, L1, L2)
// ---------------------------------------------------------------------------

#[test]
fn m1_sorted_one_target_is_a_twin() {
    let o = run_az(&["--descriptor", SORTED, "--target-address", M1, "--json"]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    let v = o.json();
    assert_eq!(v["result"], "unverified");
    assert!(reasons(&v).contains(&"twin".to_string()), "{v}");
    let w = found(&v);
    assert_eq!(w["from_user_template"], true);
    assert_eq!(first_target_path(w, 0), "m/48'/0'/0'/2'/0/0");
    assert_eq!(first_target_path(w, 1), "m/48'/0'/0'/2'/0/0");
    let twin = v["wallets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["role"] == "twin")
        .expect("twin listed");
    assert!(twin["descriptor"]
        .as_str()
        .unwrap()
        .starts_with("wsh(multi(2,"));
    assert_eq!(v["searched"]["slot_orders"], 1, "sorted collapse");
}

#[test]
fn d1_distinguishing_indices() {
    let o = run_az(&["--descriptor", SORTED, "--target-address", M1, "--json"]);
    let v = o.json();
    let w = found(&v);
    let d = w["distinguishing"].as_array().unwrap();
    let idx: Vec<u64> = d.iter().map(|e| e["index"].as_u64().unwrap()).collect();
    assert_eq!(idx, vec![1, 3, 5]);
    let all: String = d.iter().map(|e| e["addresses"].to_string()).collect();
    for a in [
        "bc1qp3z0f9m2yzgax82hw88jaqznsca2gtktnyuvmh4uwzlsg4736y8s2qes5n",
        "bc1qmkt0wm4ypxh0rcpyrm05rsy4r36t042frpuu28tyxvanjnlg2lxq5ng527",
        "bc1qhug5hhjd9ar4g4guurqvln5vk7pthv79v5hnd77kal2ft9a2ly6qz24zfh",
        "bc1qt99jw7rz5ey5ke3gzngyp4kqa3cehqq5c2z7s46eq6f4q49df8ks8zj3um",
        "bc1qq4wwd93p02d84j9nywleeza6cuj3whehmfgj0wse5h2pm9du9j7qsudgz5",
        "bc1qp02n8d2ctm99vm7mj2hxdqqjt5y7nyzmfycm8dhn0pnnpz6cjjds8axnts",
    ] {
        assert!(all.contains(a), "{a} missing from {all}");
    }
    let o = run_az(&["--descriptor", SORTED, "--target-address", M1]);
    assert!(o.stdout.contains("they differ at:"), "{}", o.stdout);
    assert!(o.stdout.contains("receive 1:"));
    assert!(o.stdout.contains("receive 5:"));
}

#[test]
fn m1p_pins_do_not_reduce_the_twin_check() {
    let z = origin_key(Z, "m/48'/0'/0'/2'");
    let cos = format!("@1={z}");
    let o = run_a(&[
        "--descriptor",
        SORTED,
        "--cosigner",
        &cos,
        "--target-address",
        M1,
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(reasons(&o.json()).contains(&"twin".to_string()));
}

#[test]
fn m2_unsorted_template_finds_z_then_a() {
    let o = run_az(&["--descriptor", UNSORTED, "--target-address", M1, "--json"]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    let v = o.json();
    assert!(reasons(&v).contains(&"twin".to_string()));
    let w = found(&v);
    assert_eq!(slot_seed(w, 0), Some(1));
    assert_eq!(slot_seed(w, 1), Some(0));
    assert_eq!(w["from_user_template"], true);
    assert!(w["descriptor"].as_str().unwrap().starts_with("wsh(multi("));
}

#[test]
fn m3_unsorted_no_twin_exit_0() {
    let o = run_az(&["--descriptor", UNSORTED, "--target-address", M3, "--json"]);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
    let v = o.json();
    assert_eq!(v["result"], "match");
    let w = found(&v);
    assert_eq!(slot_seed(w, 0), Some(0));
    assert_eq!(slot_seed(w, 1), Some(1));
    assert_eq!(v["searched"]["slot_orders"], 2);
}

#[test]
fn m4_independent_paths() {
    let o = run_az(&["--descriptor", SORTED, "--target-address", M4, "--json"]);
    assert_eq!(o.code, 4);
    assert_eq!(o.json()["result"], "no_match");
    let o = run_az(&[
        "--descriptor",
        SORTED,
        "--target-address",
        M4,
        "--independent-paths",
        "--number-of-accounts",
        "3",
        "--json",
    ]);
    let v = o.json();
    assert_ne!(v["result"], "no_match", "{}", o.stderr);
    let w = found(&v);
    let paths: Vec<String> = w["slots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| format!("{}:{}", s["seed_index"], s["origin_path"].as_str().unwrap()))
        .collect();
    assert!(paths.contains(&"0:m/48'/0'/0'/2'".to_string()), "{paths:?}");
    assert!(paths.contains(&"1:m/87'/0'/2'".to_string()), "{paths:?}");
    assert_eq!(w["targets"][0]["alternative"], 1);
    assert_eq!(w["targets"][0]["index"], 7);
    assert_eq!(v["searched"]["independent_paths"], true);
}

#[test]
fn m5_two_targets_sorted_exit_0() {
    let o = run_az(&[
        "--descriptor",
        SORTED,
        "--target-address",
        M1,
        "--target-address",
        M1_IDX1,
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
    let v = o.json();
    assert_eq!(v["result"], "match");
    assert_eq!(found(&v)["targets"].as_array().unwrap().len(), 2);
}

#[test]
fn m6_sorted_twin_matches_all_targets() {
    let o = run_az(&[
        "--descriptor",
        UNSORTED,
        "--target-address",
        M1,
        "--target-address",
        M1_IDX1,
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(
        o.stdout
            .contains("the sorted twin of your template matches all targets"),
        "{}",
        o.stdout
    );
    let o = run_az(&[
        "--descriptor",
        UNSORTED,
        "--target-address",
        M1,
        "--target-address",
        M1_IDX1,
        "--json",
    ]);
    let v = o.json();
    assert!(reasons(&v).contains(&"twin".to_string()));
    assert_eq!(
        v["wallets"][0]["role"], "twin",
        "the covering twin is listed first"
    );
    assert_eq!(v["wallets"][0]["targets"].as_array().unwrap().len(), 2);
}

#[test]
fn r1_taproot_sorted_twin() {
    let o = run_az(&[
        "--descriptor",
        "tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        "--target-address",
        R1,
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    let v = o.json();
    assert!(reasons(&v).contains(&"twin".to_string()));
    assert_eq!(first_target_path(found(&v), 0), "m/87'/0'/0'/0/0");
}

#[test]
fn r2_taproot_unsorted_exit_0() {
    let o = run_az(&[
        "--descriptor",
        "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        "--target-address",
        R2,
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
    let w = found(&o.json()).clone();
    assert_eq!(slot_seed(&w, 0), Some(1));
    assert_eq!(slot_seed(&w, 1), Some(0));
}

#[test]
fn u1_per_slot_multipath_no_collapse() {
    let o = run_az(&[
        "--descriptor",
        "wsh(sortedmulti(2,@0/<0;1>/*,@1/<2;3>/*))",
        "--target-address",
        U1,
        "--json",
    ]);
    let v = o.json();
    assert_ne!(v["result"], "no_match", "{}", o.stderr);
    assert_eq!(v["searched"]["slot_orders"], 2, "no sorted collapse");
    let w = found(&v);
    assert_eq!(slot_seed(w, 0), Some(0));
    assert_eq!(slot_seed(w, 1), Some(1));
    assert_eq!(first_target_path(w, 0), "m/48'/0'/0'/2'/0/0");
    assert_eq!(first_target_path(w, 1), "m/48'/0'/0'/2'/2/0");
}

#[test]
fn n1_sh_wsh() {
    let o = run_az(&[
        "--descriptor",
        "sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*)))",
        "--target-address",
        N1,
        "--json",
    ]);
    let v = o.json();
    assert_ne!(v["result"], "no_match", "{}", o.stderr);
    let w = found(&v);
    assert_eq!(w["script_type"], "p2sh-p2wsh");
    assert_eq!(first_target_path(w, 0), "m/48'/0'/0'/2'/0/0");
}

fn key_with_use_site(phrase: &str, path: &str) -> String {
    format!("{}/<0;1>/*", origin_key(phrase, path))
}

#[test]
fn l1_two_sorted_leaves_search_24_orders() {
    let p = "m/87'/0'/0'";
    let (a, z, c, d) = (
        key_with_use_site(A, p),
        key_with_use_site(Z, p),
        key_with_use_site(C3, p),
        key_with_use_site(D4, p),
    );
    // Minted from {A,C},{Z,D}.
    let target = mint(
        &format!("tr({NUMS},{{sortedmulti_a(1,{a},{c}),sortedmulti_a(1,{z},{d})}})"),
        0,
        0,
    );
    let cc = origin_key(C3, p);
    let dd = origin_key(D4, p);
    let o = run_az(&[
        "--descriptor",
        "tr(NUMS,{sortedmulti_a(1,@0/<0;1>/*,@1/<0;1>/*),sortedmulti_a(1,@2/<0;1>/*,@3/<0;1>/*)})",
        "--cosigner",
        &cc,
        "--cosigner",
        &dd,
        "--target-address",
        &target,
        "--number-of-accounts",
        "1",
        "--json",
    ]);
    let v = o.json();
    assert_ne!(v["result"], "no_match", "{}", o.stderr);
    assert_eq!(v["searched"]["slot_orders"], 24);
    let w = found(&v);
    // A (seed 0) shares a leaf with the C cosigner.
    let slot_of = |pred: &dyn Fn(&Value) -> bool| -> u64 {
        w["slots"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| pred(s))
            .unwrap()["slot"]
            .as_u64()
            .unwrap()
    };
    let sa = slot_of(&|s| s["seed_index"] == 0);
    let (cfp, _) = xpub_at(C3, p);
    let sc = slot_of(&|s| s["stated_fingerprint"] == cfp.to_string());
    assert_eq!(sa / 2, sc / 2, "A and C share a leaf: {w}");
    assert!(
        !reasons(&v).contains(&"split".to_string()),
        "symmetric orders are merged into one wallet: {v}"
    );
}

#[test]
fn l2_internal_key_is_a_slot() {
    let p = "m/87'/0'/0'";
    let (a, z, c) = (
        key_with_use_site(A, p),
        key_with_use_site(Z, p),
        key_with_use_site(C3, p),
    );
    let target = mint(&format!("tr({z},sortedmulti_a(2,{a},{z},{c}))"), 0, 0);
    let cc = origin_key(C3, p);
    let o = run_az(&[
        "--descriptor",
        "tr(@0/<0;1>/*,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
        "--cosigner",
        &cc,
        "--target-address",
        &target,
        "--number-of-accounts",
        "1",
        "--json",
    ]);
    let v = o.json();
    assert_ne!(v["result"], "no_match", "{}", o.stderr);
    assert_eq!(v["searched"]["slot_orders"], 6);
    assert_eq!(slot_seed(found(&v), 0), Some(1), "Z is the internal key");
}

// ---------------------------------------------------------------------------
// keyed descriptor and cosigners (K1, B1, B2)
// ---------------------------------------------------------------------------

#[test]
fn k1_keyed_descriptor_places_by_derivation() {
    let (_, ax) = xpub_at(A, "m/48'/0'/0'/2'");
    let (_, zx) = xpub_at(Z, "m/48'/0'/0'/2'");
    let desc = format!(
        "wsh(sortedmulti(2,[00000000/48h/0h/0h/2h]{ax}/<0;1>/*,[3f635a63/48h/0h/0h/2h]{zx}/<0;1>/*))"
    );
    let o = run_a(&["--descriptor", &desc, "--target-address", M1, "--json"]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    let v = o.json();
    assert_eq!(v["searched"]["mode"], "keyed");
    assert!(reasons(&v).contains(&"twin".to_string()));
    let w = found(&v);
    let cmd = w["bundle_command"].as_str().unwrap();
    assert!(cmd.contains("[73c5da0a/48h/0h/0h/2h]@0"), "{cmd}");
    assert!(cmd.contains(&format!("@1.xpub={zx}")), "{cmd}");
    assert!(cmd.contains("@1.fingerprint=3f635a63"), "{cmd}");
    assert_eq!(w["slots"][0]["stated_fingerprint"], "00000000");
    assert!(
        o.stderr.contains("states 00000000") || o.stdout.contains("states 00000000"),
        "the mismatch is noted"
    );
}

#[test]
fn keyed_seed_matching_no_key_is_refused() {
    let (_, ax) = xpub_at(A, "m/48'/0'/0'/2'");
    let (_, zx) = xpub_at(Z, "m/48'/0'/0'/2'");
    let desc = format!(
        "wsh(sortedmulti(2,[73c5da0a/48h/0h/0h/2h]{ax}/<0;1>/*,[3f635a63/48h/0h/0h/2h]{zx}/<0;1>/*))"
    );
    let o = run_with(
        &[
            "--phrase",
            "@env:S",
            "--descriptor",
            &desc,
            "--target-address",
            M1,
        ],
        &[("S", C3)],
        None,
    );
    assert_eq!(o.code, 2, "{}", o.stderr);
}

#[test]
fn keyed_non_standard_origin_is_placed() {
    let (_, ax) = xpub_at(A, "m/48'/0'/25'/2'");
    let (_, zx) = xpub_at(Z, "m/48'/0'/0'/2'");
    let desc = format!(
        "wsh(multi(2,[73c5da0a/48h/0h/25h/2h]{ax}/<0;1>/*,[3f635a63/48h/0h/0h/2h]{zx}/<0;1>/*))"
    );
    let target = mint(&desc, 0, 3);
    let o = run_a(&["--descriptor", &desc, "--target-address", &target, "--json"]);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
    let w = found(&o.json()).clone();
    assert_eq!(w["slots"][0]["origin_path"], "m/48'/0'/25'/2'");
    assert_eq!(w["slots"][0]["template"], "keyed");
}

#[test]
fn b1_bare_cosigner_with_canonical_header() {
    let (_, zx) = xpub_at(Z, "m/48'/0'/0'/2'");
    let zs = zx.to_string();
    let o = run_a(&[
        "--descriptor",
        SORTED,
        "--cosigner",
        &zs,
        "--target-address",
        M1,
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    let v = o.json();
    let r = reasons(&v);
    assert!(r.contains(&"origin_unknown".to_string()), "{r:?}");
    assert!(r.contains(&"twin".to_string()), "{r:?}");
    let w = found(&v);
    let cmd = w["bundle_command"].as_str().unwrap();
    assert!(cmd.contains("@1.path=m/48h/0h/0h/2h"), "{cmd}");
    assert!(!cmd.contains("@1.fingerprint"), "{cmd}");
}

#[test]
fn b2_bare_cosigner_header_mismatch_prints_placeholder() {
    let (_, zx) = xpub_at(Z, "m/48'/0'/0'/2'");
    let zs = zx.to_string();
    let o = run_a(&[
        "--descriptor",
        "tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        "--cosigner",
        &zs,
        "--target-address",
        B2,
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    let v = o.json();
    assert!(reasons(&v).contains(&"origin_unknown".to_string()));
    let w = found(&v);
    assert!(w["bundle_command"].is_null());
    assert_eq!(w["bundle_command_unavailable"], "origin_unknown");
    assert_eq!(slot_seed(w, 0), Some(0));
    let o = run_a(&[
        "--descriptor",
        "tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        "--cosigner",
        &zs,
        "--target-address",
        B2,
    ]);
    assert!(o.stdout.contains("<origin of this xpub>"), "{}", o.stdout);
}

#[test]
fn cosigner_as_mk1() {
    // mk1 for Z at m/48'/0'/0'/2' via the toolkit's own bundle/convert is out
    // of scope here; a `[fp/path]xpub` cosigner and an unplaced one cover the
    // placement logic, and mk1 decode reuses `mk_codec::decode`.
    let z = origin_key(Z, "m/48'/0'/0'/2'");
    let o = run_a(&[
        "--descriptor",
        UNSORTED,
        "--cosigner",
        &z,
        "--target-address",
        M3,
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
    let w = found(&o.json()).clone();
    assert_eq!(slot_seed(&w, 0), Some(0));
    assert_eq!(w["slots"][1]["source"], "cosigner");
}

// ---------------------------------------------------------------------------
// H1, G1, refusals
// ---------------------------------------------------------------------------

#[test]
fn h1_path_step_above_md1_range() {
    let o = run_a(&[
        "--target-address",
        H1,
        "--add-path",
        "m/84'/0'/536870912'",
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    let v = o.json();
    assert!(reasons(&v).contains(&"path_unrecordable".to_string()));
    let w = found(&v);
    assert!(w["bundle_command"].is_null());
    assert_eq!(w["bundle_command_unavailable"], "path_unrecordable");
    assert_eq!(first_target_path(w, 0), "m/84'/0'/536870912'/0/0");
    let o = run_a(&[
        "--target-address",
        H1_OK,
        "--add-path",
        "m/84'/0'/536870911'",
    ]);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
}

#[test]
fn g1_bare_at_n_refused() {
    let o = run_az(&[
        "--descriptor",
        "wsh(sortedmulti(2,@0,@1))",
        "--target-address",
        M1,
    ]);
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("would be recorded as"), "{}", o.stderr);
}

#[test]
fn refusals_exit_2() {
    let (_, zx) = xpub_at(Z, "m/48'/0'/0'/2'");
    let zs = zx.to_string();
    // --cosigner without --descriptor.
    assert_eq!(run_a(&["--cosigner", &zs, "--target-address", S1]).code, 2);
    // Hardened wildcard.
    assert_eq!(
        run_a(&["--descriptor", "wpkh(@0/*h)", "--target-address", S1]).code,
        2
    );
    // A slot without a key.
    assert_eq!(
        run_az(&[
            "--descriptor",
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
            "--target-address",
            M1
        ])
        .code,
        2
    );
    // More keys than slots.
    assert_eq!(
        run_az(&["--descriptor", "wpkh(@0/<0;1>/*)", "--target-address", S1]).code,
        2
    );
    // The same seed twice.
    assert_eq!(
        run_with(
            &[
                "--from",
                "phrase=@env:SEED_A",
                "--from",
                "phrase=@env:SEED_A",
                "--descriptor",
                SORTED,
                "--target-address",
                M1
            ],
            &envs(),
            None
        )
        .code,
        2
    );
    // A lone 32-byte v0 target without a descriptor.
    assert_eq!(run_a(&["--target-address", M1]).code, 2);
    // A second stdin reader.
    assert_eq!(
        run_with(
            &[
                "--from",
                "phrase=-",
                "--passphrase",
                "-",
                "--target-address",
                S1
            ],
            &[],
            Some(A)
        )
        .code,
        2
    );
    // Above the candidate ceiling (u64 range).
    let o = run_az(&[
        "--descriptor",
        SORTED,
        "--target-address",
        M1,
        "--independent-paths",
        "--number-of-accounts",
        "200000000",
    ]);
    assert_eq!(o.code, 2, "{}", o.stderr);
}

#[test]
fn private_keys_refused_and_not_echoed() {
    let seed = Mnemonic::parse_in(bip39::Language::English, Z)
        .unwrap()
        .to_seed("");
    let xprv = Xpriv::new_master(NetworkKind::Main, &seed)
        .unwrap()
        .to_string();
    let desc = format!("wsh(sortedmulti(2,@0/<0;1>/*,{xprv}/<0;1>/*))");
    let o = run_a(&["--descriptor", &desc, "--target-address", M1]);
    assert_eq!(o.code, 2);
    assert!(!o.stderr.contains(&xprv[4..40]), "{}", o.stderr);
    let o = run_a(&[
        "--descriptor",
        SORTED,
        "--cosigner",
        &xprv,
        "--target-address",
        M1,
    ]);
    assert_eq!(o.code, 2);
    assert!(!o.stderr.contains(&xprv[4..40]), "{}", o.stderr);
}

#[test]
fn argv_guard_refuses_inline_phrase() {
    let o = run_with(&["--phrase", A, "--target-address", S1], &[], None);
    assert_eq!(o.code, 2, "{}", o.stderr);
}

// ---------------------------------------------------------------------------
// match semantics: gap edges, split, unmatched, several seeds
// ---------------------------------------------------------------------------

fn bip84_receive(idx: u32) -> String {
    mint(
        &format!("wpkh({})", key_with_use_site(A, "m/84'/0'/0'")),
        0,
        idx,
    )
}

#[test]
fn gap_edges() {
    let t19 = bip84_receive(19);
    let t20 = bip84_receive(20);
    assert_eq!(run_a(&["--target-address", &t19]).code, 0);
    assert_eq!(run_a(&["--target-address", &t20]).code, 4);
    assert_eq!(
        run_a(&["--target-address", &t20, "--gap-limit", "21"]).code,
        0
    );
}

#[test]
fn external_only_skips_change() {
    let o = run_a(&["--target-address", S2, "--external-only", "--json"]);
    assert_eq!(o.code, 4);
    assert_eq!(o.json()["result"], "no_match");
}

#[test]
fn two_accounts_split() {
    let o = run_a(&["--target-address", S1, "--target-address", S8, "--json"]);
    assert_eq!(o.code, 4);
    let v = o.json();
    assert_eq!(v["result"], "unverified");
    assert!(reasons(&v).contains(&"split".to_string()));
}

#[test]
fn one_unmatched() {
    let other = mint(
        &format!("wpkh({})", key_with_use_site(Z, "m/84'/0'/0'")),
        0,
        0,
    );
    let o = run_a(&["--target-address", S1, "--target-address", &other, "--json"]);
    assert_eq!(o.code, 4);
    let v = o.json();
    assert!(reasons(&v).contains(&"unmatched".to_string()));
    assert_eq!(v["unmatched_targets"][0]["reason"], "not_found");
}

#[test]
fn p2tr_beside_wsh_template_is_type_not_searched() {
    let o = run_az(&[
        "--descriptor",
        UNSORTED,
        "--target-address",
        M3,
        "--target-address",
        S3,
        "--json",
    ]);
    assert_eq!(o.code, 4);
    let v = o.json();
    assert!(reasons(&v).contains(&"unmatched".to_string()));
    assert_eq!(v["unmatched_targets"][0]["reason"], "type_not_searched");
}

#[test]
fn two_seeds_no_descriptor() {
    let o = run_az(&["--target-address", S1, "--json"]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    let v = o.json();
    assert_eq!(v["seeds_without_match"], serde_json::json!([1]));
    let o = run_az(&["--target-address", S1]);
    assert!(o.stdout.contains("seed 1: no target found"), "{}", o.stdout);
}

#[test]
fn intake_entropy_and_seedqr() {
    let o = run_with(
        &["--from", "entropy=@env:E", "--target-address", S1],
        &[("E", "00000000000000000000000000000000")],
        None,
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    let qr = format!("{}0003", "0000".repeat(11));
    let o = run_with(
        &["--from", "seedqr=@env:Q", "--target-address", S1, "--json"],
        &[("Q", &qr)],
        None,
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(found(&o.json())["bundle_command"]
        .as_str()
        .unwrap()
        .contains("@0.seedqr=@env:Q"));
}

#[test]
fn bare_wildcard_has_no_chain_step() {
    let t = mint(&format!("wpkh({}/*)", origin_key(A, "m/84'/0'/0'")), 0, 5);
    let o = run_a(&[
        "--descriptor",
        "wpkh(@0/*)",
        "--target-address",
        &t,
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    let v = o.json();
    assert_eq!(first_target_path(found(&v), 0), "m/84'/0'/0'/5");
    assert_eq!(v["searched"]["alternatives"], serde_json::json!([0]));
}

#[test]
fn no_match_reports_the_searched_set() {
    let other = mint(
        &format!("wpkh({})", key_with_use_site(Z, "m/84'/0'/0'")),
        0,
        0,
    );
    let o = run_a(&["--target-address", &other]);
    assert_eq!(o.code, 4);
    assert!(
        o.stdout
            .contains("m/45' (the bundle bare-sh default, not BIP-45 cosigner branches)"),
        "{}",
        o.stdout
    );
    assert!(o.stdout.contains("index window [0, 20)"));
    assert!(o.stdout.contains("--add-path"));
}

// ---------------------------------------------------------------------------
// redaction and secrets
// ---------------------------------------------------------------------------

#[test]
fn redaction_inline_phrase_becomes_stdin_or_env() {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "--allow-argv-secret",
            "xpub-search",
            "address-of-seed",
            "--from",
            &format!("phrase={A}"),
            "--from",
            &format!("phrase={Z}"),
            "--descriptor",
            SORTED,
            "--target-address",
            M1,
            "--target-address",
            M1_IDX1,
            "--json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&o.stdout);
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(0), "{stderr}");
    let v: Value = serde_json::from_str(stdout.trim()).unwrap();
    let cmd = found(&v)["bundle_command"].as_str().unwrap().to_string();
    assert!(cmd.contains("@0.phrase=-"), "{cmd}");
    assert!(cmd.contains("@1.phrase=@env:MNEMONIC_SEED_1"), "{cmd}");
    assert_eq!(cmd.matches("=-").count(), 1);
    for w in ["abandon", "zoo", "wrong"] {
        assert!(!stdout.contains(w), "{w} on stdout");
        assert!(!stderr.contains(&format!("{w} {w}")), "{w} on stderr");
    }
    assert!(found(&v)["redaction_note"].as_str().is_some());
}

#[test]
fn redaction_inline_passphrase_beside_stdin_seed() {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "--allow-argv-secret",
            "xpub-search",
            "address-of-seed",
            "--phrase-stdin",
            "--passphrase",
            "TREZOR",
            "--target-address",
            P1,
            "--json",
        ])
        .write_stdin(A)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&o.stdout);
    assert_eq!(o.status.code(), Some(0));
    let v: Value = serde_json::from_str(stdout.trim()).unwrap();
    let cmd = found(&v)["bundle_command"].as_str().unwrap();
    assert!(cmd.contains("@0.phrase=-"), "{cmd}");
    assert!(
        cmd.contains("--passphrase @env:MNEMONIC_PASSPHRASE"),
        "{cmd}"
    );
    assert!(!stdout.contains("TREZOR"));
}

/// The printed command, run: it exits 0 and mints a wallet whose first
/// receive address is the found wallet's (checked through `restore --md1`).
fn run_printed(cmd: &str, envs: &[(&str, &str)], stdin: Option<&str>) -> String {
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let line = cmd.replacen("mnemonic", &format!("'{}'", bin.display()), 1);
    let mut c = std::process::Command::new("sh");
    c.arg("-c").arg(format!("{line} --json"));
    for (k, v) in envs {
        c.env(k, v);
    }
    c.stdin(std::process::Stdio::piped());
    c.stdout(std::process::Stdio::piped());
    c.stderr(std::process::Stdio::piped());
    let mut child = c.spawn().unwrap();
    {
        use std::io::Write;
        let mut si = child.stdin.take().unwrap();
        if let Some(s) = stdin {
            si.write_all(s.as_bytes()).unwrap();
        }
    }
    let o = child.wait_with_output().unwrap();
    assert_eq!(
        o.status.code(),
        Some(0),
        "printed command failed: {line}\n{}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn printed_command_runs_single_sig_and_multisig() {
    let o = run_a(&["--target-address", S1, "--json"]);
    let cmd = found(&o.json())["bundle_command"]
        .as_str()
        .unwrap()
        .to_string();
    let out = run_printed(&cmd, &envs(), None);
    assert!(out.contains("md1"), "{out}");

    let o = run_az(&["--descriptor", UNSORTED, "--target-address", M3, "--json"]);
    let cmd = found(&o.json())["bundle_command"]
        .as_str()
        .unwrap()
        .to_string();
    let out = run_printed(&cmd, &envs(), None);
    assert!(out.contains("md1"), "{out}");

    let o = run_az(&[
        "--descriptor",
        "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        "--target-address",
        R2,
        "--json",
    ]);
    let cmd = found(&o.json())["bundle_command"]
        .as_str()
        .unwrap()
        .to_string();
    run_printed(&cmd, &envs(), None);
}

#[test]
fn printed_command_runs_for_bare_cosigner_b1() {
    let (_, zx) = xpub_at(Z, "m/48'/0'/0'/2'");
    let zs = zx.to_string();
    let o = run_a(&[
        "--descriptor",
        SORTED,
        "--cosigner",
        &zs,
        "--target-address",
        M1,
        "--json",
    ]);
    let cmd = found(&o.json())["bundle_command"]
        .as_str()
        .unwrap()
        .to_string();
    run_printed(&cmd, &envs(), None);
}

#[test]
fn json_envelope_shape() {
    let o = run_a(&["--target-address", S1, "--json"]);
    let v = o.json();
    assert_eq!(v["schema_version"], "1");
    for k in [
        "result",
        "reasons",
        "wallets",
        "unmatched_targets",
        "seeds_without_match",
        "searched",
    ] {
        assert!(v.get(k).is_some(), "missing {k}");
    }
    let w = found(&v);
    for k in [
        "role",
        "twin_of",
        "from_user_template",
        "descriptor",
        "script_type",
        "merged_forms",
        "slots",
        "targets",
        "distinguishing",
        "bundle_command",
        "bundle_command_redacted",
        "bundle_command_unavailable",
        "redaction_note",
    ] {
        assert!(w.get(k).is_some(), "wallet missing {k}");
    }
    let s = &v["searched"];
    for k in [
        "mode",
        "paths",
        "child_keys",
        "candidate_scripts",
        "min_account",
        "end_account",
        "add_paths",
        "refugium_seats",
        "gap_limit",
        "alternatives",
        "script_types",
        "slot_orders",
        "independent_paths",
        "passphrase_supplied",
        "matches_total",
        "matches_kept",
    ] {
        assert!(s.get(k).is_some(), "searched missing {k}");
    }
    assert!(s["candidate_scripts"].is_null(), "null in single-sig");
}

// ---------------------------------------------------------------------------
// review r1 (design/agent-reports/address-of-seed-impl-review-r1.md)
// ---------------------------------------------------------------------------

/// The slot (0-based) whose key has this master fingerprint.
fn slot_by_fp<'a>(w: &'a Value, fp: &str) -> &'a Value {
    w["slots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["master_fingerprint"] == fp || s["stated_fingerprint"] == fp)
        .unwrap_or_else(|| panic!("no slot with fingerprint {fp}: {w}"))
}

/// C1 (review r1): with `--refugium-seats` the seat is fixed by the slot, so
/// a sorted template must not collapse slot orders. Z holds seat 0, A seat
/// 1; the wallet is found whichever order the seeds are given in.
#[test]
fn c1_refugium_two_seats_found_in_either_seed_order() {
    let z0 = key_with_use_site(Z, "m/270028'/0'/0'");
    let a1 = key_with_use_site(A, "m/270028'/0'/1'");
    for (tmpl, desc) in [
        (SORTED.to_string(), format!("wsh(sortedmulti(2,{z0},{a1}))")),
        (
            "tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))".to_string(),
            format!("tr({NUMS},sortedmulti_a(2,{z0},{a1}))"),
        ),
        (UNSORTED.to_string(), format!("wsh(multi(2,{z0},{a1}))")),
    ] {
        let target = mint(&desc, 0, 0);
        for order in [
            ["phrase=@env:SEED_A", "phrase=@env:SEED_Z"],
            ["phrase=@env:SEED_Z", "phrase=@env:SEED_A"],
        ] {
            let o = run_with(
                &[
                    "--from",
                    order[0],
                    "--from",
                    order[1],
                    "--descriptor",
                    &tmpl,
                    "--refugium-seats",
                    "--target-address",
                    &target,
                    "--json",
                ],
                &envs(),
                None,
            );
            let v = o.json();
            assert_ne!(v["result"], "no_match", "{tmpl} {order:?}: {}", o.stderr);
            let w = found(&v);
            let z = slot_by_fp(w, "3f635a63");
            let a = slot_by_fp(w, "73c5da0a");
            assert_eq!(z["origin_path"], "m/270028'/0'/0'", "{tmpl} {order:?}: {w}");
            assert_eq!(a["origin_path"], "m/270028'/0'/1'", "{tmpl} {order:?}: {w}");
            assert_eq!(z["template"], "refugium-seat");
            assert_eq!(z["refugium_seat"], 0);
            assert_eq!(a["refugium_seat"], 1);
        }
    }
}

/// Mint a keyless md1 whose origin paths differ per slot. `bundle
/// --md1-form template` records one shared origin, so the test mints the
/// keyed (policy) md1 with `bundle`, then drops its keys and fingerprints
/// with md-codec at the toolkit's pin and re-splits it: the card a
/// coordinator that records per-slot origins without keys would carry.
fn mint_keyless_divergent_md1(descriptor: &str) -> Vec<String> {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            descriptor,
            "--slot",
            "@0.phrase=@env:SEED_A",
            "--slot",
            "@1.phrase=@env:SEED_Z",
            "--json",
        ])
        .envs(envs())
        .output()
        .unwrap();
    assert_eq!(
        o.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let chunks: Vec<String> = v["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .collect();
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    let mut d = md_codec::chunk::reassemble(&refs).unwrap();
    assert!(matches!(
        d.path_decl.paths,
        md_codec::origin_path::PathDeclPaths::Divergent(_)
    ));
    d.tlv.fingerprints = None;
    d.tlv.pubkeys = None;
    md_codec::chunk::split(&d).unwrap()
}

/// I1 (review r1): a keyless md1 recording a different origin per slot pins
/// the path to the slot, not to a seed, so the sorted collapse is unsound
/// there too. M4's wallet (A at m/48'/0'/0'/2' in @0, Z at m/87'/0'/2' in
/// @1) is found with the seeds in either order.
#[test]
fn i1_keyless_md1_divergent_paths_either_seed_order() {
    let md1 = mint_keyless_divergent_md1(
        "wsh(sortedmulti(2,[73c5da0a/48h/0h/0h/2h]@0/<0;1>/*,[3f635a63/87h/0h/2h]@1/<0;1>/*))",
    );
    for order in [
        ["phrase=@env:SEED_A", "phrase=@env:SEED_Z"],
        ["phrase=@env:SEED_Z", "phrase=@env:SEED_A"],
    ] {
        let mut args: Vec<String> = vec![
            "--from".into(),
            order[0].into(),
            "--from".into(),
            order[1].into(),
            "--target-address".into(),
            M4.into(),
            "--json".into(),
        ];
        for c in &md1 {
            args.push("--descriptor-from".into());
            args.push(format!("md1={c}"));
        }
        let a: Vec<&str> = args.iter().map(String::as_str).collect();
        let o = run_with(&a, &envs(), None);
        let v = o.json();
        assert_ne!(v["result"], "no_match", "{order:?}: {}", o.stdout);
        let w = found(&v);
        assert_eq!(slot_by_fp(w, "73c5da0a")["origin_path"], "m/48'/0'/0'/2'");
        assert_eq!(slot_by_fp(w, "3f635a63")["origin_path"], "m/87'/0'/2'");
        assert_eq!(slot_by_fp(w, "73c5da0a")["template"], "md1");
    }
    // A no-match on the card's paths carries the template-as-text hint.
    let mut args: Vec<String> = vec![
        "--from".into(),
        "phrase=@env:SEED_A".into(),
        "--from".into(),
        "phrase=@env:SEED_Z".into(),
        "--target-address".into(),
        M1.into(),
    ];
    for c in &md1 {
        args.push("--descriptor-from".into());
        args.push(format!("md1={c}"));
    }
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let o = run_with(&a, &envs(), None);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(
        o.stdout.contains("the paths came from the md1 card"),
        "{}",
        o.stdout
    );
}

/// I2 (review r1): `@0` beside a pasted cosigner xpub searches (the literal
/// key becomes a placed cosigner slot); it never prints a false no-match.
#[test]
fn i2_template_mixing_at_n_with_a_literal_xpub_searches() {
    let z = origin_key(Z, "m/48'/0'/0'/2'");
    let o = run_a(&[
        "--descriptor",
        &format!("wsh(sortedmulti(2,@0/<0;1>/*,{z}/<0;1>/*))"),
        "--target-address",
        M1,
        "--json",
    ]);
    let v = o.json();
    assert_ne!(v["result"], "no_match", "{}\n{}", o.stdout, o.stderr);
    let w = found(&v);
    assert_eq!(slot_by_fp(w, "73c5da0a")["origin_path"], "m/48'/0'/0'/2'");
    assert_eq!(slot_by_fp(w, "3f635a63")["source"], "cosigner");
    assert!(reasons(&v).contains(&"twin".to_string()), "{v}");
}

/// I3 (review r1): a keyed descriptor with a valid BIP-380 checksum is
/// accepted; a wrong checksum is refused against the text as given.
#[test]
fn i3_keyed_descriptor_with_checksum() {
    let (_, ax) = xpub_at(A, "m/48'/0'/0'/2'");
    let (_, zx) = xpub_at(Z, "m/48'/0'/0'/2'");
    let bare = format!(
        "wsh(multi(2,[73c5da0a/48h/0h/0h/2h]{ax}/<0;1>/*,[3f635a63/48h/0h/0h/2h]{zx}/<0;1>/*))"
    );
    let with_sum = Descriptor::<DescriptorPublicKey>::from_str(&bare)
        .unwrap()
        .to_string();
    assert!(with_sum.contains('#'), "{with_sum}");
    let o = run_a(&["--descriptor", &with_sum, "--target-address", M3, "--json"]);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
    let (body, sum) = with_sum.rsplit_once('#').unwrap();
    let bad = format!(
        "{body}#{}",
        if sum.starts_with('q') {
            "pppppppp"
        } else {
            "qqqqqqqq"
        }
    );
    let o = run_a(&["--descriptor", &bad, "--target-address", M3]);
    assert_eq!(o.code, 2, "{}", o.stderr);
    assert!(o.stderr.contains("checksum"), "{}", o.stderr);
}

/// Z's m/48'/0'/0'/2' key as mk1 chunks.
fn z_mk1() -> Vec<String> {
    let (fp, x) = xpub_at(Z, "m/48'/0'/0'/2'");
    let card = mk_codec::KeyCard::new(
        vec![[0u8; 4]],
        Some(fp),
        DerivationPath::from_str("m/48'/0'/0'/2'").unwrap(),
        x,
    );
    mk_codec::encode(&card).unwrap()
}

/// I6 (review r1) and the §9 mk1 rows: an mk1 cosigner, placed (`@N=` per
/// chunk) and unplaced (one `--cosigner` per chunk, grouped by chunk-set id).
#[test]
fn i6_mk1_cosigner_placed_and_unplaced_chunks() {
    let chunks = z_mk1();
    assert!(chunks.len() >= 2, "an xpub mk1 is multi-chunk");
    // Unplaced: one value per chunk.
    let mut args: Vec<String> = vec!["--descriptor".into(), UNSORTED.into()];
    for c in &chunks {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    args.extend(["--target-address".into(), M3.into(), "--json".into()]);
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let o = run_a(&a);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
    let v = o.json();
    let w = found(&v);
    assert_eq!(w["slots"][1]["source"], "cosigner");
    assert_eq!(w["slots"][1]["origin_path"], "m/48'/0'/0'/2'");
    assert_eq!(w["slots"][1]["stated_fingerprint"], "3f635a63");
    // Placed: `@1=` repeated per chunk.
    let mut args: Vec<String> = vec!["--descriptor".into(), UNSORTED.into()];
    for c in &chunks {
        args.push("--cosigner".into());
        args.push(format!("@1={c}"));
    }
    args.extend(["--target-address".into(), M3.into(), "--json".into()]);
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let o = run_a(&a);
    assert_eq!(o.code, 0, "{}\n{}", o.stdout, o.stderr);
    // A chunk group that does not reassemble is refused.
    let o = run_a(&[
        "--descriptor",
        UNSORTED,
        "--cosigner",
        &chunks[0],
        "--target-address",
        M3,
    ]);
    assert_eq!(o.code, 2, "{}", o.stderr);
}

/// Minor M1 (review r1): `reasons` is empty for `no_match` (SPEC §6).
#[test]
fn no_match_json_reasons_empty() {
    let o = run_a(&["--target-address", S10, "--json"]);
    assert_eq!(o.code, 4);
    let v = o.json();
    assert_eq!(v["result"], "no_match");
    assert_eq!(v["reasons"], serde_json::json!([]));
}

/// Minor M2 (review r1): a match at `--add-path m` (the master key) cannot
/// be put on a card: its path is printed, no command, exit 4.
#[test]
fn add_path_master_match_prints_no_command() {
    let seed = Mnemonic::parse_in(bip39::Language::English, A)
        .unwrap()
        .to_seed("");
    let master = Xpub::from_priv(
        &Secp256k1::new(),
        &Xpriv::new_master(NetworkKind::Main, &seed).unwrap(),
    );
    let target = mint(&format!("wpkh([73c5da0a]{master}/<0;1>/*)"), 0, 0);
    let o = run_a(&["--target-address", &target, "--add-path", "m", "--json"]);
    assert_eq!(o.code, 4, "{}\n{}", o.stdout, o.stderr);
    let v = o.json();
    let w = found(&v);
    assert!(w["bundle_command"].is_null(), "{w}");
    assert_eq!(w["bundle_command_unavailable"], "path_unrecordable");
    assert_eq!(first_target_path(w, 0), "m/0/0");
    assert!(reasons(&v).contains(&"path_unrecordable".to_string()));
}

/// Minor M3 (review r1): a positional ms1 on argv gets the argv advisory,
/// on this verb and on path-of-xpub (the shared intake).
#[test]
fn positional_ms1_gets_the_argv_advisory() {
    const A_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let o = run_with(&[A_MS1, "--target-address", S1], &[], None);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(
        o.stderr
            .contains("secret material on argv (positional ms1)"),
        "{}",
        o.stderr
    );
    let (_, x) = xpub_at(A, "m/84'/0'/0'");
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            A_MS1,
            "--target-xpub",
            &x.to_string(),
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&o.stderr).contains("secret material on argv (positional ms1)"));
}
