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
    /// `--archetype` preset argv reproducing this fixture byte-exactly
    /// (presets SPEC §7 layer 2 — Release B).
    preset_args: &'static [&'static str],
}

// The fixtures' own key strings (presets SPEC §6 — the immutable canon).
const K1: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
const K2: &str = "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
const K3: &str = "[33333333/48h/0h/0h/2h]xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB";
const K4: &str = "[44444444/48h/0h/0h/2h]xpub661MyMwAqRbcGczjuMoRm6dXaLDEhW1u34gKenbeYqAix21mdUKJyuyu5F1rzYGVxyL6tmgBUAEPrEz92mBXjByMRiJdba9wpnN37RLLAXa";
const K5: &str = "[55555555/48h/0h/0h/2h]xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw";
const HASH_HEX: &str = "926a54995ca48600920a19bf7bc502ca5f2f7d07e6f804c4f00ebf0325084dbc";

const ARCHETYPES: &[Archetype] = &[
    Archetype {
        name: "simple-timelocked-inheritance",
        spec: include_str!("fixtures/descriptor_builder/simple-timelocked-inheritance.json"),
        descriptor: include_str!("fixtures/descriptor_builder/simple-timelocked-inheritance.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/simple-timelocked-inheritance.bip388"),
        preset_args: &[
            "--archetype", "simple-timelocked-inheritance",
            "--key", K1,
            "--recovery-key", K2,
            "--older", "65535",
        ],
    },
    Archetype {
        name: "decaying-multisig",
        spec: include_str!("fixtures/descriptor_builder/decaying-multisig.json"),
        descriptor: include_str!("fixtures/descriptor_builder/decaying-multisig.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/decaying-multisig.bip388"),
        preset_args: &[
            "--archetype", "decaying-multisig",
            "--key", K1, "--key", K2, "--threshold", "2", "--older", "1000",
            "--recovery-key", K3, "--recovery-key", K4,
            "--recovery-threshold", "2", "--recovery-older", "2000",
            "--final-key", K5, "--after", "500000",
        ],
    },
    Archetype {
        name: "kofn-recovery",
        spec: include_str!("fixtures/descriptor_builder/kofn-recovery.json"),
        descriptor: include_str!("fixtures/descriptor_builder/kofn-recovery.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/kofn-recovery.bip388"),
        preset_args: &[
            "--archetype", "kofn-recovery",
            "--key", K1, "--key", K2, "--key", K3, "--threshold", "2",
            "--recovery-key", K4, "--older", "52560",
        ],
    },
    Archetype {
        name: "tiered-recovery",
        spec: include_str!("fixtures/descriptor_builder/tiered-recovery.json"),
        descriptor: include_str!("fixtures/descriptor_builder/tiered-recovery.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/tiered-recovery.bip388"),
        preset_args: &[
            "--archetype", "tiered-recovery",
            "--key", K1, "--key", K2, "--threshold", "2", "--older", "4032",
            "--recovery-key", K3, "--recovery-key", K4, "--recovery-key", K5,
            "--recovery-threshold", "2",
        ],
    },
    Archetype {
        name: "hashlock-gated",
        spec: include_str!("fixtures/descriptor_builder/hashlock-gated.json"),
        descriptor: include_str!("fixtures/descriptor_builder/hashlock-gated.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/hashlock-gated.bip388"),
        preset_args: &[
            "--archetype", "hashlock-gated",
            "--key", K1, "--hash", HASH_HEX,
            "--recovery-key", K2, "--older", "144",
        ],
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

// ======================================================================
// Release B (archetype presets) — presets SPEC §7. Layer 2: the preset
// argv reproduces the SAME Release-A goldens byte-exactly (one fixture
// set, two producers, one canon).
// ======================================================================

#[test]
fn preset_descriptor_goldens() {
    for a in ARCHETYPES {
        let out = bin()
            .args(["build-descriptor"])
            .args(a.preset_args)
            .args(["--format", "descriptor"])
            .assert()
            .success();
        let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(got, a.descriptor, "preset descriptor golden mismatch for {}", a.name);
    }
}

#[test]
fn preset_bip388_goldens() {
    for a in ARCHETYPES {
        let out = bin()
            .args(["build-descriptor"])
            .args(a.preset_args)
            .args(["--format", "bip388"])
            .assert()
            .success();
        let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(got, a.bip388, "preset bip388 golden mismatch for {}", a.name);
    }
}

/// Stdin contract (presets SPEC §1/§7, R0-r1 I1): preset mode never touches
/// stdin — piped content is ignored and the output is byte-equal to the
/// no-stdin run (the golden). The goldens above already cover the
/// no-stdin-attached direction (assert_cmd runs with stdin closed).
#[test]
fn preset_ignores_piped_stdin() {
    let a = &ARCHETYPES[2]; // kofn-recovery
    let out = bin()
        .args(["build-descriptor"])
        .args(a.preset_args)
        .args(["--format", "descriptor"])
        .write_stdin("{ this is not json and must be ignored")
        .assert()
        .success();
    let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(got, a.descriptor, "piped stdin must be ignored in preset mode");
}

/// `--archetype` conflicts with `--spec` at the clap level (presets SPEC §1).
#[test]
fn preset_conflicts_with_spec() {
    bin()
        .args(["build-descriptor", "--archetype", "kofn-recovery", "--spec", "x.json"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be used with"));
}

/// Param flags require `--archetype` at the clap level (presets SPEC §1).
#[test]
fn param_flag_without_archetype_is_clap_error() {
    bin()
        .args(["build-descriptor", "--key", K1])
        .assert()
        .failure()
        .stderr(predicates::str::contains("--archetype"));
}

/// Producer negatives (presets SPEC §3.1/§7): exit 2 + a `param`-kind
/// diagnostic with the `params` sentinel path, naming the flag.
#[test]
fn preset_missing_required_param_exit_2() {
    let out = bin()
        .args([
            "build-descriptor", "--archetype", "kofn-recovery",
            "--key", K1, "--key", K2, "--threshold", "2", "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let diags = v["diagnostics"].as_array().unwrap();
    assert_eq!(diags.len(), 2, "missing --recovery-key AND --older");
    for d in diags {
        assert_eq!(d["kind"], "param");
        assert_eq!(d["node_path"], "params");
    }
    let all = diags.iter().map(|d| d["message"].as_str().unwrap()).collect::<Vec<_>>().join("\n");
    assert!(all.contains("--recovery-key") && all.contains("--older"));
}

#[test]
fn preset_inapplicable_param_exit_2() {
    bin()
        .args([
            "build-descriptor", "--archetype", "kofn-recovery",
            "--key", K1, "--key", K2, "--threshold", "2",
            "--recovery-key", K4, "--older", "52560",
            "--hash", HASH_HEX,
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("param"))
        .stderr(predicates::str::contains("--hash is not a parameter of kofn-recovery"));
}

#[test]
fn preset_decay_ordering_violation_exit_2() {
    bin()
        .args([
            "build-descriptor", "--archetype", "decaying-multisig",
            "--key", K1, "--key", K2, "--threshold", "2", "--older", "2000",
            "--recovery-key", K3, "--recovery-key", K4,
            "--recovery-threshold", "2", "--recovery-older", "2000",
            "--final-key", K5, "--after", "500000",
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("--recovery-older"))
        .stderr(predicates::str::contains("--older"));
}

/// Gate flow-through (presets SPEC §3.2/§7): the producer does NOT duplicate
/// gate rules — k>n and duplicate keys reach the gate and fail with the
/// gate's OWN kinds at node-addressed paths (`flag` provenance lands in
/// Phase 2).
#[test]
fn preset_k_gt_n_flows_to_gate_schema_field() {
    let out = bin()
        .args([
            "build-descriptor", "--archetype", "kofn-recovery",
            "--key", K1, "--key", K2, "--threshold", "5",
            "--recovery-key", K4, "--older", "52560", "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let diags = v["diagnostics"].as_array().unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0]["kind"], "schema_field");
    assert_eq!(diags[0]["node_path"], "root.or_d[0]");
}

#[test]
fn preset_duplicate_key_flows_to_gate_repeated_keys() {
    let out = bin()
        .args([
            "build-descriptor", "--archetype", "kofn-recovery",
            "--key", K1, "--key", K1, "--threshold", "2",
            "--recovery-key", K4, "--older", "52560", "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let diags = v["diagnostics"].as_array().unwrap();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0]["kind"], "repeated_keys");
    assert_eq!(diags[0]["node_path"], "root.or_d[0]");
}

/// Key-order discrimination (presets SPEC §7/§11.3): argv order maps into the
/// quorum untouched — swapping two `--key`s on a `multi` archetype changes
/// the descriptor.
#[test]
fn preset_key_order_is_preserved_and_load_bearing() {
    let a = &ARCHETYPES[2]; // kofn-recovery (multi)
    let swapped: &[&str] = &[
        "--archetype", "kofn-recovery",
        "--key", K2, "--key", K1, "--key", K3, "--threshold", "2",
        "--recovery-key", K4, "--older", "52560",
    ];
    let out = bin()
        .args(["build-descriptor"])
        .args(swapped)
        .args(["--format", "descriptor"])
        .assert()
        .success();
    let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_ne!(got, a.descriptor, "swapped key order must change the multi descriptor");
}

/// Preset golden non-vacuity, PER ARCHETYPE (presets SPEC §7; Phase-1 review
/// I1): mutating one numeric param breaks the byte-exact golden. This is the
/// only layer that catches a lower fn hardcoding a fixture value instead of
/// reading the param (layers 1+2 feed fixture values, so a hardcode is
/// invisible to them).
#[test]
fn preset_negative_discrimination_mutated_param_breaks_golden() {
    let mutations: &[(&str, &str, &str)] = &[
        ("simple-timelocked-inheritance", "--older", "65534"),
        ("decaying-multisig", "--after", "500001"),
        ("kofn-recovery", "--threshold", "3"),
        ("tiered-recovery", "--older", "4033"),
        ("hashlock-gated", "--older", "145"),
    ];
    for (name, flag, mutated_value) in mutations {
        let a = ARCHETYPES.iter().find(|a| a.name == *name).unwrap();
        let mut argv: Vec<&str> = a.preset_args.to_vec();
        let i = argv
            .iter()
            .position(|s| s == flag)
            .unwrap_or_else(|| panic!("{name}: {flag} not in preset_args"));
        argv[i + 1] = mutated_value;
        let out = bin()
            .args(["build-descriptor"])
            .args(&argv)
            .args(["--format", "descriptor"])
            .assert()
            .success();
        let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_ne!(got, a.descriptor, "{name}: mutated {flag} must change the descriptor");
    }
}
