//! Integration tests for `mnemonic build-descriptor` (descriptor-builder engine,
//! Phase 3). Pins the 5 archetype descriptor + bip388 goldens, the exit codes,
//! the `--spec-schema` dump, and the bip388 round-trip through the v0.49.0
//! `export-wallet --descriptor` intake (SPEC §7).

use assert_cmd::Command;
use serde_json::Value;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

const FIX: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/descriptor_builder");

struct Archetype {
    name: &'static str,
    spec: &'static str,
    descriptor: &'static str,
    bip388: &'static str,
}

const ARCHETYPES: &[Archetype] = &[
    Archetype {
        name: "simple-timelocked-inheritance",
        spec: include_str!("fixtures/descriptor_builder/simple-timelocked-inheritance.json"),
        descriptor: include_str!("fixtures/descriptor_builder/simple-timelocked-inheritance.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/simple-timelocked-inheritance.bip388"),
    },
    Archetype {
        name: "decaying-multisig",
        spec: include_str!("fixtures/descriptor_builder/decaying-multisig.json"),
        descriptor: include_str!("fixtures/descriptor_builder/decaying-multisig.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/decaying-multisig.bip388"),
    },
    Archetype {
        name: "kofn-recovery",
        spec: include_str!("fixtures/descriptor_builder/kofn-recovery.json"),
        descriptor: include_str!("fixtures/descriptor_builder/kofn-recovery.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/kofn-recovery.bip388"),
    },
    Archetype {
        name: "tiered-recovery",
        spec: include_str!("fixtures/descriptor_builder/tiered-recovery.json"),
        descriptor: include_str!("fixtures/descriptor_builder/tiered-recovery.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/tiered-recovery.bip388"),
    },
    Archetype {
        name: "hashlock-gated",
        spec: include_str!("fixtures/descriptor_builder/hashlock-gated.json"),
        descriptor: include_str!("fixtures/descriptor_builder/hashlock-gated.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/hashlock-gated.bip388"),
    },
];

fn spec_path(name: &str) -> String {
    format!("{FIX}/{name}.json")
}

#[test]
fn archetype_descriptor_goldens() {
    for a in ARCHETYPES {
        let out = bin()
            .args(["build-descriptor", "--spec", &spec_path(a.name), "--network", "mainnet", "--format", "descriptor"])
            .assert()
            .success();
        let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(got, a.descriptor, "descriptor golden mismatch for {}", a.name);
    }
}

#[test]
fn archetype_bip388_goldens() {
    for a in ARCHETYPES {
        let out = bin()
            .args(["build-descriptor", "--spec", &spec_path(a.name), "--network", "mainnet", "--format", "bip388"])
            .assert()
            .success();
        let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(got, a.bip388, "bip388 golden mismatch for {}", a.name);
    }
}

/// SPEC §7 round-trip: the built descriptor → `export-wallet --descriptor
/// --format bip388` reproduces the build-descriptor bip388 (both route through
/// `descriptor_to_bip388_wallet_policy`).
#[test]
fn bip388_round_trips_through_export_wallet_descriptor() {
    for a in ARCHETYPES {
        let descriptor = a.descriptor.trim_end();
        let out = bin()
            .args(["export-wallet", "--descriptor", descriptor, "--format", "bip388"])
            .assert()
            .success();
        let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(got, a.bip388, "round-trip bip388 mismatch for {}", a.name);
    }
}

#[test]
fn json_envelope_has_descriptor_bip388_cost_and_empty_diagnostics() {
    let a = &ARCHETYPES[2]; // kofn-recovery
    let out = bin()
        .args(["build-descriptor", "--spec", &spec_path(a.name), "--network", "mainnet", "--json"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["descriptor"], a.descriptor.trim_end());
    assert!(v["bip388"].is_object());
    assert!(v["cost"].is_object(), "cost preview embedded");
    assert_eq!(v["diagnostics"], serde_json::json!([]));
}

#[test]
fn spec_schema_dumps_versioned_grammar() {
    let out = bin()
        .args(["build-descriptor", "--spec-schema"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["spec_schema_version"], 1);
    assert!(v["node_kinds"].as_array().unwrap().iter().any(|k| k == "andor"));
    assert_eq!(v["multipath_suffix"], "/<0;1>/*");
}

#[test]
fn sigless_branch_fails_json_diagnostics_exit_2() {
    let spec = r#"{"schema_version":1,"wrapper":"wsh","root":{"or_d":[{"pk":"xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13"},{"after":100}]}}"#;
    let out = bin()
        .args(["build-descriptor", "--network", "mainnet", "--json"])
        .write_stdin(spec)
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let diags = v["diagnostics"].as_array().unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0]["kind"], "sigless_branch");
    assert_eq!(diags[0]["node_path"], "root.or_d[1]");
}

#[test]
fn field_error_human_stderr_exit_2() {
    // multi k=5 > 1 key
    let spec = r#"{"schema_version":1,"wrapper":"wsh","root":{"multi":{"k":5,"keys":["xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13"]}}}"#;
    bin()
        .args(["build-descriptor", "--network", "mainnet"])
        .write_stdin(spec)
        .assert()
        .code(2)
        .stderr(predicates::str::contains("schema_field"))
        .stderr(predicates::str::contains("k=5"));
}

#[test]
fn unparseable_spec_exit_2() {
    bin()
        .args(["build-descriptor", "--network", "mainnet"])
        .write_stdin("{ not json")
        .assert()
        .code(2)
        .stderr(predicates::str::contains("spec JSON parse error"));
}

/// SPEC §0 watch-only-out: a secret in a key node is refused (exit 2) and the
/// secret material NEVER appears in any output (stderr human OR `--json`
/// diagnostics). Guards both the step-1 xprv screen and the step-2 `from_str`
/// path (WIF) against a future miniscript-rev error-message change that could
/// start echoing the offending token (Phase-3 review I1).
#[test]
fn secret_keys_refused_without_leaking() {
    const XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";
    const WIF: &str = "L1aW4aubDFB7yfras2S1myaw5ekeuMr3HF1g9Y7e3WL4hNXnHQ7B";
    const XPUB: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
    // A raw 64-hex key (interpreted as x-only). The engine appends `/<0;1>/*`
    // unconditionally → `from_str` rejects it (InvalidPublicKeyLength) → refused,
    // never emitted (the Phase-3-r2 raw-hex-not-emitted proof; arm is no-echo).
    const RAWHEX: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

    let cases = [
        // (label, root-json, secret-substring-to-forbid)
        ("pk(xprv)", format!(r#"{{"pk":"{XPRV}"}}"#), XPRV),
        ("pk(wif)", format!(r#"{{"pk":"{WIF}"}}"#), WIF),
        ("pk(raw-hex)", format!(r#"{{"pk":"{RAWHEX}"}}"#), RAWHEX),
        (
            "multi(xprv)",
            format!(r#"{{"multi":{{"k":2,"keys":["{XPRV}","{XPUB}"]}}}}"#),
            XPRV,
        ),
    ];

    for (label, root, secret) in cases {
        let spec = format!(r#"{{"schema_version":1,"wrapper":"wsh","root":{root}}}"#);

        // human → stderr, exit 2, no secret substring anywhere
        let out = bin()
            .args(["build-descriptor", "--network", "mainnet"])
            .write_stdin(spec.clone())
            .assert()
            .code(2);
        let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert!(!stderr.contains(secret), "{label}: secret leaked to stderr: {stderr}");
        assert!(!stdout.contains(secret), "{label}: secret leaked to stdout");
        assert!(stdout.is_empty(), "{label}: no descriptor emitted for a secret key");

        // --json → diagnostics on stdout, exit 2, no secret substring
        let out = bin()
            .args(["build-descriptor", "--network", "mainnet", "--json"])
            .write_stdin(spec)
            .assert()
            .code(2);
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert!(!stdout.contains(secret), "{label}: secret leaked to --json: {stdout}");
        let v: Value = serde_json::from_str(&stdout).unwrap();
        assert!(!v["diagnostics"].as_array().unwrap().is_empty(), "{label}: diagnostics present");
        assert!(v["descriptor"].is_null(), "{label}: no descriptor in failure envelope");
    }
}

#[test]
fn negative_discrimination_mutated_threshold_breaks_golden() {
    // Mutate kofn-recovery's multi threshold 2→1 and assert the descriptor
    // golden no longer matches (the golden is non-vacuous).
    let mutated = ARCHETYPES[2].spec.replacen("\"k\": 2", "\"k\": 1", 1);
    let out = bin()
        .args(["build-descriptor", "--network", "mainnet", "--format", "descriptor", "--spec", "-"])
        .write_stdin(mutated)
        .assert()
        .success();
    let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_ne!(got, ARCHETYPES[2].descriptor, "mutated threshold must change the descriptor");
}
