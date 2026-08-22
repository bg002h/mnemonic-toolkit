//! `mnemonic export-wallet --format bitcoin-core-addresses` — the ONE route
//! that a real Bitcoin Core actually loads for this wallet.
//!
//! Realizes Phase 1b of `mnemonic-engrave/design/PLAN_wallet_file_export.md`.
//!
//! **Why this format exists.** Every descriptor-level route into Core is
//! closed: the wallet's tier-4 spend path needs no signature, and Core refuses
//! `importdescriptors` on that basis on every version through v31.1 with the
//! rule non-waivable on its side. An `addr()` list carries no spend policy, so
//! it imports. You can WATCH this wallet in Core; you cannot DESCRIBE it to
//! Core. Nothing here may claim `--allow` or this format makes the descriptor
//! route work.
//!
//! **Emission is not acceptance.** The `#[test]`s in this file stop at the
//! emitted bytes. The acceptance criterion that matters — a live
//! `importdescriptors` returning per-entry `success: true` — lives in
//! `tests/bitcoind_addr_import.rs`, is `#[ignore]`-by-default, and is run by
//! `scripts/bitcoind-addr-import-gate.sh` (and by the
//! `bitcoin-core-addr-import` CI workflow). Accepting this format on emission
//! alone would repeat the exact defect the plan's round-1 review caught for
//! `--format bitcoin-core`.

use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

fn allow_fixture(name: &str) -> String {
    let p = PathBuf::from("tests/fixtures/export_wallet_allow").join(name);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("fixture {}: {e}", p.display()))
        .trim()
        .to_string()
}

/// The journey's own address capture — see
/// `tests/fixtures/export_wallet_addresses/PROVENANCE.md`.
fn journey(name: &str) -> Vec<String> {
    let p = PathBuf::from("tests/fixtures/export_wallet_addresses").join(name);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("fixture {}: {e}", p.display()))
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn rcw_wsh() -> String {
    allow_fixture("rcw_wsh_descriptor.txt")
}

fn rcw_tr() -> String {
    allow_fixture("rcw_tr_descriptor.txt")
}

/// A plain 2-of-3 `sh(multi(...))` — every spend path needs a signature, so
/// the Phase-1 admission gate runs on it and does NOT fire. The control that
/// proves the flag requirement below is about the sigless branch and not about
/// this format.
const SANE_DESCRIPTOR: &str = "sh(multi(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[5436d724/48'/0'/0'/2']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))#ek6d38cp";

/// A SINGLE-PATH sane descriptor: no `<0;1>`, so there is no change chain to
/// derive and the artifact has to say so rather than inventing one.
const SINGLE_PATH_DESCRIPTOR: &str = "wpkh([5436d724/84'/0'/0']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/0/*)";

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .output()
        .expect("mnemonic failed to spawn");
    Run {
        code: out.status.code().expect("process was signalled"),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Emit and parse, asserting exit 0 first so a refusal never reaches the JSON
/// parser as an opaque error.
fn emit(args: &[&str]) -> Vec<Value> {
    let r = run(args);
    assert_eq!(r.code, 0, "args={args:?}\nstderr={}", r.stderr);
    let v: Value = serde_json::from_str(&r.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON: {e}\n{}", r.stdout));
    v.as_array()
        .unwrap_or_else(|| panic!("importdescriptors payload must be a JSON ARRAY: {v}"))
        .clone()
}

/// `addr(<address>)#<8-char csum>` → `<address>`.
fn address_of(entry: &Value) -> String {
    let desc = entry["desc"].as_str().expect("desc is a string");
    let body = desc
        .strip_prefix("addr(")
        .unwrap_or_else(|| panic!("entry is not an addr() descriptor: {desc}"));
    let (addr, csum) = body
        .rsplit_once(")#")
        .unwrap_or_else(|| panic!("entry has no `)#<csum>` tail: {desc}"));
    assert_eq!(csum.len(), 8, "BIP-380 checksum must be 8 chars: {desc}");
    assert!(
        csum.chars().all(|c| c.is_ascii_alphanumeric()),
        "checksum must be ASCII-alphanumeric: {desc}"
    );
    addr.to_string()
}

fn split_chains(entries: &[Value]) -> (Vec<String>, Vec<String>) {
    let mut recv = Vec::new();
    let mut chg = Vec::new();
    for e in entries {
        if e["internal"].as_bool().expect("internal is a bool") {
            chg.push(address_of(e));
        } else {
            recv.push(address_of(e));
        }
    }
    (recv, chg)
}

// ---------------------------------------------------------------------------
// The address cross-check: four implementations, one address list.
// ---------------------------------------------------------------------------

#[test]
fn wsh_receive_addresses_equal_the_journeys() {
    let d = rcw_wsh();
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "5",
        "--allow",
        "sigless-branch",
    ]);
    let (recv, _) = split_chains(&entries);
    assert_eq!(recv, journey("journey_wsh_receive.txt"));
}

#[test]
fn wsh_change_addresses_equal_the_journeys() {
    let d = rcw_wsh();
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "5",
        "--allow",
        "sigless-branch",
    ]);
    let (_, chg) = split_chains(&entries);
    assert_eq!(chg, journey("journey_wsh_change.txt"));
}

#[test]
fn tr_receive_and_change_addresses_equal_the_journeys() {
    let d = rcw_tr();
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "5",
        "--allow",
        "sigless-branch",
    ]);
    let (recv, chg) = split_chains(&entries);
    assert_eq!(recv, journey("journey_tr_receive.txt"));
    assert_eq!(chg, journey("journey_tr_change.txt"));
    // Anti-vacuity: these are taproot addresses, not the wsh ones.
    assert!(recv.iter().all(|a| a.starts_with("bc1p")), "{recv:?}");
}

/// The change chain is DERIVED, not copied: no address appears on both chains.
/// A change-blind watch wallet silently under-reports the balance, and a
/// change list that is secretly the receive list is the same defect wearing a
/// different label.
#[test]
fn receive_and_change_are_disjoint() {
    let d = rcw_wsh();
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "5",
        "--allow",
        "sigless-branch",
    ]);
    let (recv, chg) = split_chains(&entries);
    assert_eq!(recv.len(), 5);
    assert_eq!(chg.len(), 5);
    for a in &chg {
        assert!(!recv.contains(a), "{a} appears on both chains");
    }
}

// ---------------------------------------------------------------------------
// The wire shape Core actually accepts.
// ---------------------------------------------------------------------------

/// Every entry is a NON-RANGED, INACTIVE watch entry. `range` on a non-ranged
/// descriptor is a Core-side error ("Range should not be specified for an
/// un-ranged descriptor"), and `active: true` would ask Core to derive from a
/// descriptor that cannot derive.
#[test]
fn every_entry_is_non_ranged_and_inactive() {
    let d = rcw_wsh();
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "3",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(entries.len(), 6);
    for e in &entries {
        assert!(
            e.get("range").is_none(),
            "non-ranged entry must omit range: {e}"
        );
        assert_eq!(e["active"], Value::Bool(false), "{e}");
        assert_eq!(e["timestamp"], 0, "{e}");
    }
}

/// Receive first, then change; receive entries carry the caveat label and
/// change entries carry NONE. Core refuses `label` together with
/// `internal: true` ("Internal addresses should not have a label"), so a label
/// on a change entry is not a cosmetic slip — it fails the import.
#[test]
fn change_entries_carry_no_label_because_core_refuses_one() {
    let d = rcw_wsh();
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "3",
        "--allow",
        "sigless-branch",
    ]);
    let (recv, chg): (Vec<&Value>, Vec<&Value>) = entries
        .iter()
        .partition(|e| !e["internal"].as_bool().unwrap());
    assert_eq!(recv.len(), 3);
    assert_eq!(chg.len(), 3);
    for e in &recv {
        assert!(
            e.get("label").is_some(),
            "receive entry needs the caveat: {e}"
        );
    }
    for e in &chg {
        assert!(
            e.get("label").is_none(),
            "Core refuses label+internal — this entry would fail the import: {e}"
        );
    }
    // Order: the whole receive chain, then the whole change chain.
    let internals: Vec<bool> = entries
        .iter()
        .map(|e| e["internal"].as_bool().unwrap())
        .collect();
    assert_eq!(internals, vec![false, false, false, true, true, true]);
}

/// Core's OWN checksum for the journey's first receive address, captured from
/// `getdescriptorinfo "addr(bc1qr6h…)"` on Bitcoin Core v27.0 (2026-08-22).
/// This is a Core-computed value pinned into a test that runs EVERYWHERE, so
/// the checksum claim does not rest solely on the `#[ignore]`d live gate.
const CORE_CHECKSUM_JOURNEY_RECEIVE_0: &str = "nf7wvmq9";

#[test]
fn the_checksum_is_the_one_bitcoin_core_computes() {
    let d = rcw_wsh();
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "1",
        "--allow",
        "sigless-branch",
    ]);
    let desc = entries[0]["desc"].as_str().unwrap();
    let expected = format!(
        "addr({})#{CORE_CHECKSUM_JOURNEY_RECEIVE_0}",
        journey("journey_wsh_receive.txt")[0]
    );
    assert_eq!(desc, expected);
}

// ---------------------------------------------------------------------------
// The artifact describes its own limits, in-band.
// ---------------------------------------------------------------------------

#[test]
fn the_artifact_states_its_own_count_and_the_no_derivation_caveat_in_band() {
    let d = rcw_wsh();
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "7",
        "--allow",
        "sigless-branch",
    ]);
    let label = entries[0]["label"].as_str().expect("caveat label");
    // The count, both chains, and the last usable index.
    assert!(label.contains("7 receive"), "{label}");
    assert!(label.contains("7 change"), "{label}");
    assert!(label.contains("0-6"), "last index must be stated: {label}");
    // The caveat itself.
    assert!(label.contains("FIXED LIST"), "{label}");
    assert!(label.contains("NO DERIVATION"), "{label}");
    assert!(label.contains("--count"), "the way to extend it: {label}");
    // Every receive entry carries it, so a consumer reading any part of the
    // receive half sees it.
    for e in entries.iter().filter(|e| !e["internal"].as_bool().unwrap()) {
        assert_eq!(e["label"].as_str().unwrap(), label);
    }
}

#[test]
fn a_single_path_descriptor_emits_receive_only_and_says_so() {
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        SINGLE_PATH_DESCRIPTOR,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "4",
    ]);
    assert_eq!(entries.len(), 4, "no change chain exists to derive");
    for e in &entries {
        assert_eq!(e["internal"], Value::Bool(false));
    }
    let label = entries[0]["label"].as_str().unwrap();
    assert!(label.contains("0 change"), "{label}");
    assert!(
        label.contains("single-path"),
        "the artifact must say WHY there is no change chain: {label}"
    );
}

// ---------------------------------------------------------------------------
// `--count`
// ---------------------------------------------------------------------------

/// The default is STATED, not implied: 20 per chain, the BIP-44 gap limit.
#[test]
fn default_count_is_twenty_per_chain() {
    let d = rcw_wsh();
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(entries.len(), 40);
    let (recv, chg) = split_chains(&entries);
    assert_eq!(recv.len(), 20);
    assert_eq!(chg.len(), 20);
    // The first five still agree with the journey — a longer window does not
    // shift the list.
    assert_eq!(recv[..5], journey("journey_wsh_receive.txt")[..]);
    assert_eq!(chg[..5], journey("journey_wsh_change.txt")[..]);
    assert!(entries[0]["label"].as_str().unwrap().contains("20 receive"));
}

#[test]
fn help_states_the_default_count() {
    let r = run(&["export-wallet", "--help"]);
    assert_eq!(r.code, 0);
    assert!(
        r.stdout.contains("--count"),
        "--count must appear in help: {}",
        r.stdout
    );
    assert!(
        r.stdout.contains("default 20"),
        "the default must be STATED in help, not implied: {}",
        r.stdout
    );
}

#[test]
fn count_zero_is_refused_rather_than_emitting_an_empty_watch_list() {
    let d = rcw_wsh();
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "0",
        "--allow",
        "sigless-branch",
    ]);
    assert_ne!(r.code, 0, "stdout={}", r.stdout);
    assert!(r.stderr.contains("--count"), "{}", r.stderr);
}

/// `--count` belongs to this format. Supplying it elsewhere must not change a
/// single byte of any other format's output (the per-format ignored-input
/// contract this surface has had since v0.8).
#[test]
fn count_is_silently_ignored_by_every_other_format() {
    let d = rcw_wsh();
    for f in ["bitcoin-core", "bip388", "descriptor"] {
        let base = run(&[
            "export-wallet",
            "--descriptor",
            &d,
            "--format",
            f,
            "--allow",
            "sigless-branch",
        ]);
        let with = run(&[
            "export-wallet",
            "--descriptor",
            &d,
            "--format",
            f,
            "--count",
            "3",
            "--allow",
            "sigless-branch",
        ]);
        assert_eq!(base.code, 0, "{}", base.stderr);
        assert_eq!(base.stdout, with.stdout, "--count changed --format {f}");
    }
}

// ---------------------------------------------------------------------------
// The Phase-1 admission gate, met by this format like every other.
// ---------------------------------------------------------------------------

#[test]
fn tr_refuses_without_the_flag_and_names_it() {
    let d = rcw_tr();
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
    ]);
    assert_eq!(r.code, 2, "stdout={}", r.stdout);
    assert!(
        r.stderr
            .contains("this wallet has a spend path that requires no signature"),
        "{}",
        r.stderr
    );
    assert!(
        r.stderr.contains("--allow sigless-branch"),
        "the refusal must NAME the flag, not fail with a generic sanity message: {}",
        r.stderr
    );
    // Not miniscript's own wording.
    assert!(
        !r.stderr
            .contains("All spend paths must require a signature"),
        "{}",
        r.stderr
    );
}

#[test]
fn wsh_refuses_without_the_flag_too_because_the_gate_is_uniform() {
    let d = rcw_wsh();
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
    ]);
    assert_eq!(r.code, 2, "stdout={}", r.stdout);
    assert!(r.stderr.contains("--allow sigless-branch"), "{}", r.stderr);
}

/// The THIRD intake arm (`--from-import-json`) reaches this format through the
/// same gate — Phase 1's topology (B) has no ungated arm, and a sigless
/// envelope is gated exactly like a sigless `--descriptor`.
#[test]
fn the_from_import_json_arm_is_gated_and_emits_like_the_others() {
    let env = "tests/fixtures/export_wallet_allow/envelope_sigless_wsh.json";
    let flagless = run(&[
        "export-wallet",
        "--from-import-json",
        env,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "2",
    ]);
    assert_eq!(flagless.code, 2, "stdout={}", flagless.stdout);
    assert!(
        flagless.stderr.contains("--allow sigless-branch"),
        "{}",
        flagless.stderr
    );

    let entries = emit(&[
        "export-wallet",
        "--from-import-json",
        env,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "2",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(entries.len(), 4);
    let (recv, chg) = split_chains(&entries);
    // Same wallet, same addresses, whichever arm it arrived through.
    assert_eq!(recv, journey("journey_wsh_receive.txt")[..2]);
    assert_eq!(chg, journey("journey_wsh_change.txt")[..2]);
}

/// The refusal tail is FORMAT-AWARE, and this is the whole point of the
/// format. For a descriptor-route format the flag buys emission of a file no
/// wallet application will accept; for THIS format the emitted addresses
/// genuinely import. Saying the descriptor-route sentence here would be false.
#[test]
fn the_refusal_tail_does_not_repeat_the_descriptor_routes_false_sentence() {
    let d = rcw_tr();
    let addresses = run(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
    ]);
    let descriptor_route = run(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core",
    ]);
    assert_eq!(addresses.code, 2);
    assert_eq!(descriptor_route.code, 2);
    assert!(
        descriptor_route
            .stderr
            .contains("it does not make any wallet application accept it"),
        "the descriptor route's wording is UNCHANGED by Phase 1b: {}",
        descriptor_route.stderr
    );
    assert!(
        !addresses
            .stderr
            .contains("it does not make any wallet application accept it"),
        "false for this format — these addresses DO import: {}",
        addresses.stderr
    );
    // …and it still refuses to claim the descriptor route works.
    assert!(
        addresses.stderr.contains("descriptor"),
        "it must still say the DESCRIPTOR never imports: {}",
        addresses.stderr
    );
}

#[test]
fn a_sane_wallet_needs_no_flag() {
    let entries = emit(&[
        "export-wallet",
        "--descriptor",
        SANE_DESCRIPTOR,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "2",
    ]);
    assert_eq!(entries.len(), 4);
    let (recv, chg) = split_chains(&entries);
    assert_eq!(recv.len(), 2);
    assert_eq!(chg.len(), 2);
}

/// The `--template` / `--slot` arm reaches the same emitter through the same
/// gate — a builder-produced descriptor cannot carry a sigless branch, so the
/// rule runs and does not fire.
#[test]
fn the_template_arm_reaches_this_format_too() {
    let entries = emit(&[
        "export-wallet",
        "--template",
        "wsh-sortedmulti",
        "--slot",
        "@0.xpub=xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX",
        "--slot",
        "@0.fingerprint=b8688df1",
        "--slot",
        "@1.xpub=xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6",
        "--slot",
        "@1.fingerprint=28645006",
        "--threshold",
        "2",
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "2",
    ]);
    assert_eq!(entries.len(), 4);
}

// ---------------------------------------------------------------------------
// The constraint that survives everything.
// ---------------------------------------------------------------------------

/// Neither the help text nor any message this format emits may claim the
/// DESCRIPTOR route into Core works. It does not, on any version through
/// v31.1, and the rule is non-waivable there.
#[test]
fn nothing_claims_the_descriptor_route_into_core_works() {
    let r = run(&["export-wallet", "--help"]);
    let hay = format!("{}{}", r.stdout, r.stderr).to_lowercase();
    for claim in [
        "enables export to core",
        "makes bitcoin core accept",
        "core will accept the descriptor",
    ] {
        assert!(!hay.contains(claim), "help text claims {claim:?}");
    }
    // The format's own help line must say what it IS: addresses, not a
    // descriptor.
    assert!(r.stdout.contains("bitcoin-core-addresses"), "{}", r.stdout);
}

/// A watch-only artifact must not carry secrets, and `export-wallet` is
/// watch-only by definition. Pinned so a future emitter change cannot leak an
/// xpub, an origin path or a fingerprint into an address list.
#[test]
fn the_artifact_carries_addresses_and_nothing_else() {
    let d = rcw_wsh();
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &d,
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "2",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stdout.contains("xpub"), "{}", r.stdout);
    assert!(!r.stdout.contains("sha256("), "{}", r.stdout);
    assert!(!r.stdout.contains("multi("), "{}", r.stdout);
    assert!(!r.stdout.contains("270028"), "{}", r.stdout);
}

/// `restore --md1` gains the format for free (it shares `emit_payload`) — and
/// the point of asserting it here is the other half: `restore` is NOT gated,
/// so this works with no `--allow`, exactly as `--format bitcoin-core` does.
#[test]
fn restore_reaches_the_format_without_a_flag() {
    let md1 = std::fs::read_to_string("tests/fixtures/export_wallet_allow/rcw_wsh_md1_keyed.txt")
        .expect("md1 fixture");
    let chunks: Vec<&str> = md1.split_whitespace().collect();
    let mut args: Vec<&str> = vec!["restore"];
    for c in &chunks {
        args.push("--md1");
        args.push(c);
    }
    args.extend_from_slice(&["--format", "bitcoin-core-addresses"]);
    let r = run(&args);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert!(
        !r.stderr.contains("--allow sigless-branch"),
        "restore must stay ungated: {}",
        r.stderr
    );
}
