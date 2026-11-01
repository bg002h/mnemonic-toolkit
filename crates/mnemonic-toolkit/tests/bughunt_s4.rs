//! Bug-hunt triage RED rows, tk slice S4 (other commands).
//!
//! Hunt report: `design/agent-reports/bughunt-2026-10-31-tk.md`. Each test
//! asserts the CORRECT behaviour, so it fails on unmodified code if the bug is
//! real. Bug #36 lives in `src/friendly_bughunt_s4.rs` (`friendly` is
//! binary-private). Tests only; no production change.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use serde_json::Value;
use std::os::unix::process::ExitStatusExt;
use std::str::FromStr;
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_mnemonic");

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const OTHER_PHRASE: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

// BIP-85 test-vector master (same constant `cli_derive_child.rs` uses).
const MASTER_XPRV: &str =
    "xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb";

const NUMS_XONLY: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
// BIP-340 test-vector x-only keys (same fixtures `cli_compare_cost.rs` uses).
const KEY_X_ONLY_A: &str = "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9";
const HASH_H: &str = "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9";
// secp256k1 generator, compressed (a valid on-curve 33-byte key).
const COMPRESSED_G: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

fn xpub_at(phrase: &str, path: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(path).unwrap();
    Xpub::from_priv(&secp, &master.derive_priv(&secp, &dp).unwrap())
}

// ---------------------------------------------------------------------------
// #15 bip85 dice: a huge `--length` aborts the process (SIGABRT) instead of a
// clean refusal. Run under `ulimit -v` so the failed allocation is deterministic
// on any box (on a large-memory box the unbounded request can otherwise succeed
// and the command spins for billions of rolls; both are the missing cap).
// Correct (report): a clean BadInput error -> normal exit with non-zero code,
// no signal. OPEN product question: the maximum `--length` for dice.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_tk_15_dice_huge_length_is_refused_cleanly() {
    let mut child = std::process::Command::new("sh")
        .args([
            "-c",
            "ulimit -v 4000000 && exec \"$0\" \"$@\"",
            BIN,
            "--allow-argv-secret",
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "dice",
            "--dice-sides",
            "6",
            "--length",
            "2000000000",
            "--index",
            "0",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn sh");
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut timed_out = false;
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if Instant::now() > deadline {
            timed_out = true;
            child.kill().ok();
            break child.wait().unwrap();
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    let mut err = String::new();
    {
        use std::io::Read;
        child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut err)
            .unwrap();
    }
    assert!(!timed_out, "did not terminate within 60 s; stderr: {err}");
    assert!(
        status.signal().is_none(),
        "process died from signal {:?} (6 = SIGABRT) instead of exiting cleanly; stderr: {err}",
        status.signal()
    );
    assert!(
        status.code().is_some_and(|c| c != 0),
        "expected a clean non-zero refusal; got {status:?}; stderr: {err}"
    );
}

// ---------------------------------------------------------------------------
// #24 xpub-search: the origin-elided default path for sh(wsh(sortedmulti)) must
// be BIP-48 script type 1', as `bundle` does (bip48_script_type_for_root_tag:
// Tr -> 3', Sh -> 1', else 2'). The intake only special-cases Tr.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_tk_24_xpub_search_sh_wsh_default_origin_is_1prime() {
    let a = xpub_at(PHRASE, "m/48'/0'/0'/1'");
    let b = xpub_at(OTHER_PHRASE, "m/48'/0'/0'/1'");
    let desc = format!("sh(wsh(sortedmulti(1,{a}/<0;1>/*,{b}/<0;1>/*)))");
    let out = bin()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &desc,
            "--min-account",
            "0",
            "--number-of-accounts",
            "1",
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("defaulting origin path for @0,@1 to m/48'/0'/0'/1'"),
        "sh(wsh(...)) default origin must be script type 1'; stderr: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// #25 convert: `--from minikey=@env:VAR` must resolve the variable like every
// other secret-bearing node (wif, xprv, ...). The literal `@env:MK` is instead
// handed to the mini-key parser.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_tk_25_convert_minikey_env_sentinel_resolves() {
    // Casascius 30-char vector from `cli_convert_minikey.rs`.
    let out = bin()
        .env("MK", "S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy")
        .args(["convert", "--from", "minikey=@env:MK", "--to", "wif"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success()
            && stdout.contains("wif: 5JPy8Zg7z4P7RSLsiqcqyeAF1935zjNUdMxcDeVrtU1oarrgnB7"),
        "minikey=@env:MK must resolve to the mini-key; status {:?}; stdout: {stdout}; stderr: {stderr}",
        out.status
    );
}

// ---------------------------------------------------------------------------
// #26 convert phrase->address: the passphrase IS used (the address changes with
// it) so the "--passphrase ignored on this edge" warning is false.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_tk_26_convert_phrase_to_address_passphrase_not_reported_ignored() {
    let run = |extra: &[&str]| {
        let mut args: Vec<&str> = vec!["--allow-argv-secret", "convert", "--from"];
        let from = format!("phrase={PHRASE}");
        args.push(&from);
        args.extend_from_slice(&[
            "--to",
            "address",
            "--path",
            "m/84'/0'/0'/0/0",
            "--script-type",
            "p2wpkh",
        ]);
        args.extend_from_slice(extra);
        let o = bin().args(&args).output().unwrap();
        (
            String::from_utf8_lossy(&o.stdout).to_string(),
            String::from_utf8_lossy(&o.stderr).to_string(),
        )
    };
    let (plain_out, _) = run(&[]);
    let (pp_out, pp_err) = run(&["--passphrase", "TREZOR"]);
    let passphrase_used = plain_out != pp_out;
    let warned_ignored = pp_err.contains("--passphrase ignored");
    assert!(
        !(passphrase_used && warned_ignored),
        "address changed with --passphrase ({plain_out:?} vs {pp_out:?}) yet stderr says it was \
         ignored: {pp_err}"
    );
}

// ---------------------------------------------------------------------------
// #27 compare-cost: a valid tr(...) leaf with a 64-hex hash argument must not
// get the x-only key '02' prefix glued onto the hash.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_tk_27_compare_cost_tr_leaf_hash_arg_not_inflated() {
    for hash_fn in ["sha256", "hash256"] {
        let desc = format!("tr({NUMS_XONLY},and_v(v:pk({KEY_X_ONLY_A}),{hash_fn}({HASH_H})))");
        let out = bin()
            .args(["compare-cost", "--descriptor", &desc])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "tr leaf with {hash_fn}(<64-hex>) must be accepted; status {:?}; stderr: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

// ---------------------------------------------------------------------------
// #28 compare-cost: a hash that appears twice is one preimage, not two. Uses
// or_i (the report's or_d example is not valid miniscript: its left child is
// not dissatisfiable). Correct: 2 rows (A+preimage, B+preimage) and a 2^3 = 8
// precheck, so `--max-conditions 8` must pass.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_tk_28_compare_cost_repeated_hash_counted_once() {
    let ms = format!("or_i(and_v(v:pk(A),sha256({HASH_H})),and_v(v:pk(B),sha256({HASH_H})))");
    let out = bin()
        .args(["compare-cost", "--miniscript", &ms, "--json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let labels: Vec<String> = v["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["label"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        labels.len(),
        2,
        "one hash H -> exactly 2 minimal conditions (A+preimage, B+preimage); got {labels:?}"
    );
    let capped = bin()
        .args(["compare-cost", "--miniscript", &ms, "--max-conditions", "8"])
        .output()
        .unwrap();
    assert!(
        capped.status.success(),
        "precheck must count 2^(2 keys + 1 hash) = 8, so --max-conditions 8 must pass; stderr: {}",
        String::from_utf8_lossy(&capped.stderr)
    );
}

// ---------------------------------------------------------------------------
// #32 xpub-search: account values >= 2^31 must be a clean error, never a panic.
// Two real panic sites are reachable:
//   (a) template.rs single-sig derivation_path, via --min-account 2147483648;
//   (b) descriptor_intake::bip48_default_path (origin-elided cosigner) with a
//       zero-width window, which skips the candidate loop.
// ---------------------------------------------------------------------------
fn assert_no_panic(out: &std::process::Output, what: &str) {
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.code() != Some(101) && !stderr.contains("panicked at"),
        "{what} panicked (exit {:?}); stderr: {stderr}",
        out.status.code()
    );
}

#[test]
fn bughunt_tk_32_path_of_xpub_min_account_2p31_no_panic() {
    let target = xpub_at(PHRASE, "m/86'/0'/0'");
    let out = bin()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target.to_string(),
            "--min-account",
            "2147483648",
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    assert_no_panic(&out, "path-of-xpub --min-account 2147483648");
}

#[test]
fn bughunt_tk_32_account_of_descriptor_zero_window_2p31_no_panic() {
    let a = xpub_at(PHRASE, "m/86'/0'/0'");
    let desc = format!("wsh(sortedmulti(1,{a}/<0;1>/*,{a}/<0;1>/*))");
    let out = bin()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase-stdin",
            "--descriptor",
            &desc,
            "--min-account",
            "2147483648",
            "--number-of-accounts",
            "0",
        ])
        .write_stdin(PHRASE)
        .output()
        .unwrap();
    assert_no_panic(
        &out,
        "account-of-descriptor --min-account 2147483648 --number-of-accounts 0",
    );
}

// ---------------------------------------------------------------------------
// #33 / #34 GUI schema: a `required` visibility rule on `--template` must agree
// with clap's `required_unless_present_any`. Evaluate the emitted rules for
// concrete flag sets and compare with what the real parser demands.
// ---------------------------------------------------------------------------
fn eval_pred(p: &Value, flags: &[&str], positionals: &[usize]) -> bool {
    match p["kind"].as_str().unwrap() {
        "flag_present" => flags.contains(&p["flag"].as_str().unwrap()),
        "positional_present" => positionals.contains(&(p["index"].as_u64().unwrap() as usize)),
        "not" => !eval_pred(&p["predicate"], flags, positionals),
        "any_of" => p["predicates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|q| eval_pred(q, flags, positionals)),
        "all_of" => p["predicates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|q| eval_pred(q, flags, positionals)),
        // value / slot predicates do not apply to a bare flag-presence scenario
        "dropdown_value_in" | "composite_node_is" => false,
        k if k.starts_with("slot_count") => false,
        other => panic!("unhandled predicate kind {other}"),
    }
}

fn schema_requires_template(sub: &str, flags: &[&str], positionals: &[usize]) -> bool {
    let out = bin().arg("gui-schema").output().unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let s = v["subcommands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == sub)
        .unwrap_or_else(|| panic!("no subcommand {sub} in schema"));
    s["conditional_rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["effect"]["flag"] == "--template" && r["effect"]["visibility"] == "required")
        .any(|r| eval_pred(&r["when"], flags, positionals))
}

/// Does the real clap parser list `--template` among the missing required args?
fn clap_demands_template(args: &[&str]) -> bool {
    let out = bin().args(args).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    stderr.contains("required arguments were not provided") && stderr.contains("--template <")
}

#[test]
fn bughunt_tk_33_gui_schema_bundle_import_json_does_not_require_template() {
    let clap = clap_demands_template(&[
        "bundle",
        "--network",
        "mainnet",
        "--import-json",
        "/nonexistent-bughunt.json",
    ]);
    let schema = schema_requires_template("bundle", &["--import-json"], &[]);
    assert!(
        !schema || clap,
        "schema marks --template required when only --import-json is given, but clap does not \
         require it (clap_demands_template={clap})"
    );
}

#[test]
fn bughunt_tk_34_gui_schema_verify_bundle_alternatives_do_not_require_template() {
    // (flag-set, positional-set, real-CLI args) triples; clap's alternatives are
    // descriptor, descriptor_file, md1, bundle_json, extra_strings (positional).
    type Case<'a> = (&'a str, Vec<&'a str>, Vec<usize>, Vec<&'a str>);
    let cases: Vec<Case> = vec![
        (
            "--md1",
            vec!["--md1"],
            vec![],
            vec!["verify-bundle", "--network", "mainnet", "--md1", "md1xx"],
        ),
        (
            "--bundle-json",
            vec!["--bundle-json"],
            vec![],
            vec![
                "verify-bundle",
                "--network",
                "mainnet",
                "--bundle-json",
                "/nonexistent-bughunt.json",
            ],
        ),
        (
            "extra_strings positional",
            vec![],
            vec![0],
            vec!["verify-bundle", "--network", "mainnet", "somestring"],
        ),
    ];
    let mut mismatches: Vec<String> = Vec::new();
    for (name, flags, pos, args) in cases {
        let clap = clap_demands_template(&args);
        let schema = schema_requires_template("verify-bundle", &flags, &pos);
        if schema && !clap {
            mismatches.push(format!("{name}: schema requires --template, clap does not"));
        }
    }
    assert!(mismatches.is_empty(), "{mismatches:#?}");
}

// ---------------------------------------------------------------------------
// #35 compare-cost: `Translated.concrete_keys` is documented "true if the input
// had at least one concrete hex pubkey". Mixed abstract+hex input must carry the
// "input had concrete keys" note, as pure-hex input does.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_tk_35_compare_cost_mixed_input_reports_concrete_keys() {
    let ms = format!("or_b(pk(A),s:pk({COMPRESSED_G}))");
    let out = bin()
        .args(["compare-cost", "--miniscript", &ms])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "setup: mixed input must be accepted; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("input had concrete keys"),
        "input has one concrete hex key but the note is absent; stdout: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// #48 verify-message legacy: the BIP-322 path trims the signature (pinned by the
// unit test `signature_surrounding_whitespace_tolerated`); the legacy path must
// too. Build a real P2PKH vector in-test (signing happens only in tests).
// ---------------------------------------------------------------------------
fn legacy_p2pkh_vector(message: &str) -> (String, String) {
    use bitcoin::hashes::Hash;
    use bitcoin::secp256k1::{Message, SecretKey};
    use bitcoin::sign_message::{signed_msg_hash, MessageSignature};
    use bitcoin::{Address, Network, PublicKey};
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(&[0x11u8; 32]).unwrap();
    let pk = PublicKey::new(sk.public_key(&secp));
    let addr = Address::p2pkh(pk, Network::Bitcoin);
    let digest = signed_msg_hash(message);
    let msg = Message::from_digest(digest.to_byte_array());
    let rec = secp.sign_ecdsa_recoverable(&msg, &sk);
    (
        addr.to_string(),
        MessageSignature::new(rec, true).to_base64(),
    )
}

#[test]
fn bughunt_tk_48_verify_message_legacy_tolerates_trailing_newline() {
    let (addr, sig) = legacy_p2pkh_vector("Hello World");
    let run = |signature: &str, format: &str| {
        bin()
            .args([
                "verify-message",
                "--address",
                &addr,
                "--message",
                "Hello World",
                "--signature",
                signature,
                "--format",
                format,
            ])
            .output()
            .unwrap()
    };
    let plain = run(&sig, "legacy");
    assert!(
        plain.status.success(),
        "setup: the untouched signature must verify; stderr: {}",
        String::from_utf8_lossy(&plain.stderr)
    );
    for format in ["legacy", "auto"] {
        let padded = run(&format!("{sig}\n"), format);
        assert!(
            padded.status.success(),
            "--format {format}: genuine signature + trailing newline must verify; status {:?}; \
             stderr: {}",
            padded.status,
            String::from_utf8_lossy(&padded.stderr)
        );
    }
}

// ---------------------------------------------------------------------------
// #56 silent-payment: an account-level xprv (depth 3) is not a master key. The
// help says "the xprv IS the master"; derivation of m/352'/... beneath an
// account node yields an address no BIP-352 wallet derives from the seed.
// Correct (report): refuse a non-master (depth != 0) xprv.
// ---------------------------------------------------------------------------
#[test]
fn bughunt_tk_56_silent_payment_refuses_non_master_xprv() {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let account = master
        .derive_priv(&secp, &DerivationPath::from_str("m/84'/0'/0'").unwrap())
        .unwrap();
    assert_eq!(account.depth, 3, "setup: account-level xprv has depth 3");
    let out = bin()
        .args(["silent-payment", "--secret-stdin"])
        .write_stdin(account.to_string())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        !out.status.success() && !stdout.contains("sp1"),
        "a depth-3 xprv must be refused, not turned into an sp1 address; status {:?}; \
         stdout: {stdout}; stderr: {stderr}",
        out.status
    );
}
