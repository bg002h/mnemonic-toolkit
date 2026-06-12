//! Integration tests for `mnemonic build-descriptor` (descriptor-builder engine,
//! Phase 3). Pins the 5 archetype descriptor + bip388 goldens, the exit codes,
//! the `--spec-schema` dump, and the bip388 round-trip through the v0.49.0
//! `export-wallet --descriptor` intake (SPEC §7).

use assert_cmd::Command;
use serde_json::Value;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

const FIX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/descriptor_builder"
);

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
        descriptor: include_str!(
            "fixtures/descriptor_builder/simple-timelocked-inheritance.descriptor"
        ),
        bip388: include_str!("fixtures/descriptor_builder/simple-timelocked-inheritance.bip388"),
        preset_args: &[
            "--archetype",
            "simple-timelocked-inheritance",
            "--key",
            K1,
            "--recovery-key",
            K2,
            "--older",
            "65535",
        ],
    },
    Archetype {
        name: "decaying-multisig",
        spec: include_str!("fixtures/descriptor_builder/decaying-multisig.json"),
        descriptor: include_str!("fixtures/descriptor_builder/decaying-multisig.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/decaying-multisig.bip388"),
        preset_args: &[
            "--archetype",
            "decaying-multisig",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "2",
            "--older",
            "1000",
            "--recovery-key",
            K3,
            "--recovery-key",
            K4,
            "--recovery-threshold",
            "2",
            "--recovery-older",
            "2000",
            "--final-key",
            K5,
            "--after",
            "500000",
        ],
    },
    Archetype {
        name: "kofn-recovery",
        spec: include_str!("fixtures/descriptor_builder/kofn-recovery.json"),
        descriptor: include_str!("fixtures/descriptor_builder/kofn-recovery.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/kofn-recovery.bip388"),
        preset_args: &[
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K2,
            "--key",
            K3,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--older",
            "52560",
        ],
    },
    Archetype {
        name: "tiered-recovery",
        spec: include_str!("fixtures/descriptor_builder/tiered-recovery.json"),
        descriptor: include_str!("fixtures/descriptor_builder/tiered-recovery.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/tiered-recovery.bip388"),
        preset_args: &[
            "--archetype",
            "tiered-recovery",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "2",
            "--older",
            "4032",
            "--recovery-key",
            K3,
            "--recovery-key",
            K4,
            "--recovery-key",
            K5,
            "--recovery-threshold",
            "2",
        ],
    },
    Archetype {
        name: "hashlock-gated",
        spec: include_str!("fixtures/descriptor_builder/hashlock-gated.json"),
        descriptor: include_str!("fixtures/descriptor_builder/hashlock-gated.descriptor"),
        bip388: include_str!("fixtures/descriptor_builder/hashlock-gated.bip388"),
        preset_args: &[
            "--archetype",
            "hashlock-gated",
            "--key",
            K1,
            "--hash",
            HASH_HEX,
            "--recovery-key",
            K2,
            "--older",
            "144",
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
            .args([
                "build-descriptor",
                "--spec",
                &spec_path(a.name),
                "--network",
                "mainnet",
                "--format",
                "descriptor",
            ])
            .assert()
            .success();
        let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(
            got, a.descriptor,
            "descriptor golden mismatch for {}",
            a.name
        );
    }
}

#[test]
fn archetype_bip388_goldens() {
    for a in ARCHETYPES {
        let out = bin()
            .args([
                "build-descriptor",
                "--spec",
                &spec_path(a.name),
                "--network",
                "mainnet",
                "--format",
                "bip388",
            ])
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
            .args([
                "export-wallet",
                "--descriptor",
                descriptor,
                "--format",
                "bip388",
            ])
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
        .args([
            "build-descriptor",
            "--spec",
            &spec_path(a.name),
            "--network",
            "mainnet",
            "--json",
        ])
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
    assert!(v["node_kinds"]
        .as_array()
        .unwrap()
        .iter()
        .any(|k| k == "andor"));
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
        assert!(
            !stderr.contains(secret),
            "{label}: secret leaked to stderr: {stderr}"
        );
        assert!(!stdout.contains(secret), "{label}: secret leaked to stdout");
        assert!(
            stdout.is_empty(),
            "{label}: no descriptor emitted for a secret key"
        );

        // --json → diagnostics on stdout, exit 2, no secret substring
        let out = bin()
            .args(["build-descriptor", "--network", "mainnet", "--json"])
            .write_stdin(spec)
            .assert()
            .code(2);
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert!(
            !stdout.contains(secret),
            "{label}: secret leaked to --json: {stdout}"
        );
        let v: Value = serde_json::from_str(&stdout).unwrap();
        assert!(
            !v["diagnostics"].as_array().unwrap().is_empty(),
            "{label}: diagnostics present"
        );
        assert!(
            v["descriptor"].is_null(),
            "{label}: no descriptor in failure envelope"
        );
    }
}

#[test]
fn negative_discrimination_mutated_threshold_breaks_golden() {
    // Mutate kofn-recovery's multi threshold 2→1 and assert the descriptor
    // golden no longer matches (the golden is non-vacuous).
    let mutated = ARCHETYPES[2].spec.replacen("\"k\": 2", "\"k\": 1", 1);
    let out = bin()
        .args([
            "build-descriptor",
            "--network",
            "mainnet",
            "--format",
            "descriptor",
            "--spec",
            "-",
        ])
        .write_stdin(mutated)
        .assert()
        .success();
    let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_ne!(
        got, ARCHETYPES[2].descriptor,
        "mutated threshold must change the descriptor"
    );
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
        assert_eq!(
            got, a.descriptor,
            "preset descriptor golden mismatch for {}",
            a.name
        );
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
        assert_eq!(
            got, a.bip388,
            "preset bip388 golden mismatch for {}",
            a.name
        );
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
    assert_eq!(
        got, a.descriptor,
        "piped stdin must be ignored in preset mode"
    );
}

/// `--archetype` conflicts with `--spec` at the clap level (presets SPEC §1).
#[test]
fn preset_conflicts_with_spec() {
    bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--spec",
            "x.json",
        ])
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
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "2",
            "--json",
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
    let all = diags
        .iter()
        .map(|d| d["message"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(all.contains("--recovery-key") && all.contains("--older"));
}

#[test]
fn preset_inapplicable_param_exit_2() {
    bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--older",
            "52560",
            "--hash",
            HASH_HEX,
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("param"))
        .stderr(predicates::str::contains(
            "--hash is not a parameter of kofn-recovery",
        ));
}

#[test]
fn preset_decay_ordering_violation_exit_2() {
    bin()
        .args([
            "build-descriptor",
            "--archetype",
            "decaying-multisig",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "2",
            "--older",
            "2000",
            "--recovery-key",
            K3,
            "--recovery-key",
            K4,
            "--recovery-threshold",
            "2",
            "--recovery-older",
            "2000",
            "--final-key",
            K5,
            "--after",
            "500000",
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
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "5",
            "--recovery-key",
            K4,
            "--older",
            "52560",
            "--json",
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
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K1,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--older",
            "52560",
            "--json",
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
        "--archetype",
        "kofn-recovery",
        "--key",
        K2,
        "--key",
        K1,
        "--key",
        K3,
        "--threshold",
        "2",
        "--recovery-key",
        K4,
        "--older",
        "52560",
    ];
    let out = bin()
        .args(["build-descriptor"])
        .args(swapped)
        .args(["--format", "descriptor"])
        .assert()
        .success();
    let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_ne!(
        got, a.descriptor,
        "swapped key order must change the multi descriptor"
    );
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
        assert_ne!(
            got, a.descriptor,
            "{name}: mutated {flag} must change the descriptor"
        );
    }
}

// ======================================================================
// Phase 2 (presets SPEC §3.3, §4, §5, §7): --emit-spec, kind-aware
// diagnostic provenance (`flag`), spec-mode byte-stability, --spec-schema
// archetypes section, success-path composition cells.
// ======================================================================

/// Layer 3 (presets SPEC §4/§7): the emitted spec VALUE-equals the fixture
/// JSON (pretty-printing non-contractual), and piping it back through
/// `--spec -` reproduces the descriptor golden byte-exactly.
#[test]
fn emit_spec_value_equals_fixture_and_round_trips() {
    for a in ARCHETYPES {
        let out = bin()
            .args(["build-descriptor"])
            .args(a.preset_args)
            .args(["--emit-spec"])
            .assert()
            .success();
        let emitted = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        let got: Value = serde_json::from_str(&emitted).unwrap();
        let want: Value = serde_json::from_str(a.spec).unwrap();
        assert_eq!(
            got, want,
            "{}: emitted spec != fixture (value equality)",
            a.name
        );

        let out = bin()
            .args(["build-descriptor", "--spec", "-", "--format", "descriptor"])
            .write_stdin(emitted)
            .assert()
            .success();
        let desc = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert_eq!(
            desc, a.descriptor,
            "{}: emit-spec round-trip descriptor",
            a.name
        );
    }
}

/// `--emit-spec` conflicts with `--format` and `--json` at the clap level
/// (presets SPEC §1/§4).
#[test]
fn emit_spec_conflicts_with_format_and_json() {
    for tail in [&["--format", "descriptor"][..], &["--json"][..]] {
        bin()
            .args(["build-descriptor"])
            .args(ARCHETYPES[2].preset_args)
            .args(["--emit-spec"])
            .args(tail)
            .assert()
            .failure()
            .stderr(predicates::str::contains("cannot be used with"));
    }
}

/// `--emit-spec` runs the FULL gate before printing (presets SPEC §4):
/// presets never emit any artifact the gate refuses.
#[test]
fn emit_spec_runs_the_gate_before_printing() {
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "5",
            "--recovery-key",
            K4,
            "--older",
            "52560",
            "--emit-spec",
        ])
        .assert()
        .code(2);
    assert!(
        out.get_output().stdout.is_empty(),
        "no spec emitted on a gate failure"
    );
}

/// Gate flow-through provenance (presets SPEC §3.3/§7): the kind-aware
/// resolver disambiguates two flags landing on the SAME quorum node path,
/// and `keys[i]` paths resolve via prefix semantics (P1-r1 M2).
#[test]
fn gate_diagnostics_carry_flag_provenance_in_preset_mode() {
    // k>n → SchemaField at the quorum node → --threshold (kind-override entry).
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "5",
            "--recovery-key",
            K4,
            "--older",
            "52560",
            "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let d = &v["diagnostics"][0];
    assert_eq!(d["kind"], "schema_field");
    assert_eq!(d["node_path"], "root.or_d[0]");
    assert_eq!(d["flag"], "--threshold");

    // dup key in the SAME quorum → RepeatedKeys at the quorum node → --key
    // (catch-all entry at the same prefix).
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K1,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--older",
            "52560",
            "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let d = &v["diagnostics"][0];
    assert_eq!(d["kind"], "repeated_keys");
    assert_eq!(d["node_path"], "root.or_d[0]");
    assert_eq!(d["flag"], "--key");

    // bad --hash hex → SchemaField at the sha256 node → --hash.
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "hashlock-gated",
            "--key",
            K1,
            "--hash",
            "zz26a54995ca48600920a19bf7bc502ca5f2f7d07e6f804c4f00ebf0325084db",
            "--recovery-key",
            K2,
            "--older",
            "144",
            "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let d = &v["diagnostics"][0];
    assert_eq!(d["kind"], "schema_field");
    assert_eq!(d["node_path"], "root.andor[1]");
    assert_eq!(d["flag"], "--hash");

    // xprv via --key → SecretKey at the keys[i] path → --key via PREFIX
    // semantics (P1-r1 M2: pins prefix, not exact-match, resolution).
    const XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            XPRV,
            "--key",
            K2,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--older",
            "52560",
            "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let d = &v["diagnostics"][0];
    assert_eq!(d["kind"], "secret_key");
    assert_eq!(d["node_path"], "root.or_d[0].multi.keys[0]");
    assert_eq!(d["flag"], "--key");
}

/// Funds-safety (older() BIP-68 mask gate): a recovery branch carrying an
/// older() value that consensus would silently weaken/zero is refused (exit 2)
/// on the REAL binary with a node-localized schema_field diagnostic — not just
/// the unit gate. older(65536) masks to 0 → the recovery branch would be
/// spendable immediately. Mirrors the deep-recon empirical repro.
#[test]
fn masked_older_timelock_refused_exit_2() {
    const A: &str = "xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
    const B: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
    let spec = format!(
        r#"{{"schema_version":1,"wrapper":"wsh","root":{{"or_d":[{{"pk":"{A}"}},{{"and_v":[{{"wrap":{{"w":"v","sub":{{"pk":"{B}"}}}}}},{{"older":65536}}]}}]}}}}"#
    );
    bin()
        .args(["build-descriptor", "--network", "mainnet"])
        .write_stdin(spec)
        .assert()
        .code(2)
        .stderr(predicates::str::contains("schema_field"))
        .stderr(predicates::str::contains("older"))
        .stderr(predicates::str::contains("effective value"));
}

/// Funds-safety on the PRESET path (the deep-recon "2-year vault" headline):
/// `--archetype kofn-recovery --older 105120` — 2 years in blocks overflows the
/// 16-bit BIP-68 value field, so consensus would silently mask it to ~275 days —
/// is refused (exit 2) with a schema_field diagnostic carrying `--older`
/// provenance (attribution attaches via the node-path provenance table).
#[test]
fn preset_masked_older_refused_with_older_provenance() {
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K2,
            "--key",
            K3,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--older",
            "105120",
            "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let d = &v["diagnostics"][0];
    assert_eq!(d["kind"], "schema_field");
    assert_eq!(d["flag"], "--older");
    assert!(
        d["message"].as_str().unwrap().contains("older"),
        "diagnostic must name the older() field: {d}"
    );
}

/// The contractual `flag`-ABSENT cases (presets SPEC §3.3/§7 + P1-r1 M3):
/// a diagnostic whose localized path matches no provenance entry carries NO
/// `flag` key (not `null`).
#[test]
fn cross_branch_duplicates_carry_no_flag() {
    // Cross-branch dup (--key X --recovery-key X) localizes to root.
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "simple-timelocked-inheritance",
            "--key",
            K1,
            "--recovery-key",
            K1,
            "--older",
            "65535",
            "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let d = &v["diagnostics"][0];
    assert_eq!(d["kind"], "repeated_keys");
    assert_eq!(d["node_path"], "root");
    assert!(d.get("flag").is_none(), "flag key must be ABSENT, got {d}");

    // Decaying intra-andor[2] cross-tier dup (--recovery-key X --final-key X)
    // localizes to root.andor[2] — matches no provenance prefix (P1-r1 M3).
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "decaying-multisig",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "2",
            "--older",
            "1000",
            "--recovery-key",
            K3,
            "--recovery-key",
            K4,
            "--recovery-threshold",
            "2",
            "--recovery-older",
            "2000",
            "--final-key",
            K3,
            "--after",
            "500000",
            "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let d = &v["diagnostics"][0];
    assert_eq!(d["kind"], "repeated_keys");
    assert_eq!(d["node_path"], "root.andor[2]");
    assert!(d.get("flag").is_none(), "flag key must be ABSENT, got {d}");
}

/// Producer diagnostics carry the structured flag too (presets SPEC §3.3),
/// and the human rendering appends the provenance suffix.
#[test]
fn producer_diagnostics_carry_flag_and_human_suffix() {
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K2,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--json",
        ])
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let d = &v["diagnostics"][0];
    assert_eq!(d["kind"], "param");
    assert_eq!(d["node_path"], "params");
    assert_eq!(d["flag"], "--older");

    // Human mode: gate diagnostic gets " (from --key)".
    bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K1,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--older",
            "52560",
        ])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("(from --key)"));
}

/// Spec-mode `--json` byte-stability (presets SPEC §7 self-test (c), R0-r1
/// M5): a literal golden pinned from the PRE-Phase-2 binary; the `flag` key
/// must never appear in spec mode. serde_json::Value keys serialize
/// alphabetically (kind, message, node_path).
#[test]
fn spec_mode_json_diagnostics_byte_stable_no_flag_key() {
    let spec = r#"{"schema_version":1,"wrapper":"wsh","root":{"multi":{"k":5,"keys":["xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13"]}}}"#;
    let golden = "{\n  \"diagnostics\": [\n    {\n      \"kind\": \"schema_field\",\n      \"message\": \"multi threshold k=5 must satisfy 1 \u{2264} k \u{2264} 1\",\n      \"node_path\": \"root\"\n    }\n  ]\n}\n";
    let out = bin()
        .args(["build-descriptor", "--json"])
        .write_stdin(spec)
        .assert()
        .code(2);
    let got = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        got, golden,
        "spec-mode --json diagnostics must stay byte-identical"
    );
}

/// Success-path composition under preset mode (P1-r1 M5): `--json` envelope
/// and `--network` human view.
#[test]
fn preset_success_json_and_network_compose() {
    let a = &ARCHETYPES[2]; // kofn-recovery
    let out = bin()
        .args(["build-descriptor"])
        .args(a.preset_args)
        .args(["--json"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["descriptor"], a.descriptor.trim_end());
    assert!(v["bip388"].is_object());
    assert!(v["cost"].is_object());
    assert_eq!(v["diagnostics"], serde_json::json!([]));

    let out = bin()
        .args(["build-descriptor"])
        .args(a.preset_args)
        .args(["--network", "testnet"])
        .assert()
        .success();
    let human = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        human.contains("tb1q"),
        "testnet first receive address expected:\n{human}"
    );
}

/// `--spec-schema` carries the archetypes section (presets SPEC §5):
/// registry-generated, 5 entries, with the pinned wire keys
/// (flag/kind/required/repeatable/min).
#[test]
fn spec_schema_carries_archetypes_section() {
    let out = bin()
        .args(["build-descriptor", "--spec-schema"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let archetypes = v["archetypes"].as_array().expect("archetypes array");
    let ids: Vec<&str> = archetypes
        .iter()
        .map(|a| a["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "decaying-multisig",
            "hashlock-gated",
            "kofn-recovery",
            "simple-timelocked-inheritance",
            "tiered-recovery"
        ]
    );
    for a in archetypes {
        assert!(a["summary"].as_str().is_some_and(|s| !s.is_empty()));
        for p in a["params"].as_array().unwrap() {
            assert!(p["flag"].as_str().unwrap().starts_with("--"));
            assert!(p["kind"].is_string());
            assert!(p["required"].is_boolean());
            assert!(p["repeatable"].is_boolean());
            assert!(p["min"].is_u64());
        }
    }
    // Spot-pin one entry: kofn-recovery's --key is repeatable min 2, kind key.
    let kofn = archetypes
        .iter()
        .find(|a| a["id"] == "kofn-recovery")
        .unwrap();
    let key = kofn["params"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["flag"] == "--key")
        .unwrap();
    assert_eq!(key["kind"], "key");
    assert_eq!(key["repeatable"], true);
    assert_eq!(key["min"], 2);
    assert_eq!(key["required"], true);
    // The Release-A keys are still present (additive extension).
    assert_eq!(v["spec_schema_version"], 1);
    assert!(v["node_kinds"].is_array());
}

// ======================================================================
// v0.52.0 --allow (reviewed sanity opt-out) — allow SPEC §1-§5.
// ======================================================================

const SIGLESS_SPEC: &str = r#"{"schema_version":1,"wrapper":"wsh","root":{"or_d":[{"pk":"xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13"},{"after":100}]}}"#;

fn repeated_keys_spec() -> String {
    format!(
        r#"{{"schema_version":1,"wrapper":"wsh","root":{{"or_d":[{{"multi":{{"k":2,"keys":["{K1}","{K2}"]}}}},{{"and_v":[{{"wrap":{{"w":"v","sub":{{"multi":{{"k":1,"keys":["{K1}","{K2}"]}}}}}}}},{{"older":1000}}]}}]}}}}"#
    )
}

fn mixed_timelock_spec() -> String {
    // older(100) is height-based; older(4194305) = 0x400001 is a VALID
    // time-based (512-second-unit) relative timelock — mixing the two in one
    // branch is what the --allow mixed-timelock path exercises. (Was 0x400000
    // = bit-22 with a zero 16-bit value, a no-op the older() mask gate now
    // correctly rejects at step 1; 0x400001 carries a non-zero value.)
    format!(
        r#"{{"schema_version":1,"wrapper":"wsh","root":{{"and_v":[{{"wrap":{{"w":"v","sub":{{"pk":"{K1}"}}}}}},{{"and_v":[{{"wrap":{{"w":"v","sub":{{"older":100}}}}}},{{"older":4194305}}]}}]}}}}"#
    )
}

/// Allow-success, --json mode (allow SPEC §3/§5 + R0-r1 C1 cost posture):
/// exit 0, `cost` is NULL (deterministic skip — the Tap re-parse would
/// re-run the waived rule), `allowed_rules_fired` present, banner on stderr.
#[test]
fn allow_sigless_json_success_cost_null_banner() {
    let out = bin()
        .args(["build-descriptor", "--allow", "sigless-branch", "--json"])
        .write_stdin(SIGLESS_SPEC)
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert!(v["descriptor"].as_str().unwrap().starts_with("wsh("));
    assert!(
        v["cost"].is_null(),
        "cost must be null on an allowed-insane emit"
    );
    assert_eq!(
        v["allowed_rules_fired"],
        serde_json::json!(["sigless_branch"])
    );
    assert_eq!(v["diagnostics"], serde_json::json!([]));
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("sigless-branch"),
        "banner names the fired rule: {stderr}"
    );
    assert!(
        stderr.to_uppercase().contains("OVERRIDDEN"),
        "banner is unmissable: {stderr}"
    );
}

/// Allow-success, human view: the cost block's position carries the
/// unavailable line on STDOUT (R0-r2 M-r2-2); banner on stderr; exit 0.
#[test]
fn allow_sigless_human_cost_unavailable_line() {
    let out = bin()
        .args(["build-descriptor", "--allow", "sigless-branch"])
        .write_stdin(SIGLESS_SPEC)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cost preview unavailable for a sanity-overridden descriptor"),
        "human cost line: {stdout}"
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("sigless-branch"));
}

/// Allow-success, --format descriptor: bare artifact, banner still on stderr.
#[test]
fn allow_sigless_format_descriptor_bare() {
    let out = bin()
        .args([
            "build-descriptor",
            "--allow",
            "sigless-branch",
            "--format",
            "descriptor",
        ])
        .write_stdin(SIGLESS_SPEC)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.starts_with("wsh(or_d(pk("),
        "bare descriptor: {stdout}"
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("sigless-branch"));
}

/// The flagship resurrection (allow SPEC §0): same-key degrading threshold
/// via --spec + --allow repeated-keys.
#[test]
fn allow_repeated_keys_degrading_threshold() {
    let out = bin()
        .args([
            "build-descriptor",
            "--allow",
            "repeated-keys",
            "--format",
            "descriptor",
        ])
        .write_stdin(repeated_keys_spec())
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.starts_with("wsh(or_d(multi(2,"));
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("repeated-keys"));
}

/// Mixed-timelock (KEYED tree — R0-r1 I1: a keyless one refuses
/// sigless_branch first).
#[test]
fn allow_mixed_timelock_keyed_tree() {
    let out = bin()
        .args([
            "build-descriptor",
            "--allow",
            "mixed-timelock",
            "--format",
            "descriptor",
        ])
        .write_stdin(mixed_timelock_spec())
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("mixed-timelock"));
}

/// Short-circuit semantics (allow SPEC §2): allowing one rule does not
/// waive another — sigless tree + --allow malleable still refuses
/// sigless_branch, now with the rerun hint.
#[test]
fn allow_wrong_rule_still_refuses_with_hint() {
    let out = bin()
        .args(["build-descriptor", "--allow", "malleable", "--json"])
        .write_stdin(SIGLESS_SPEC)
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["diagnostics"][0]["kind"], "sigless_branch");
    assert!(
        v["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("rerun with --allow sigless-branch after review"),
        "refusal hint present"
    );
}

/// Requested-but-unused (allow SPEC §3): sane preset + --allow → exit 0,
/// stderr note, NO allowed_rules_fired key, cost runs normally.
#[test]
fn allow_requested_but_unused_notes_and_normal_envelope() {
    let a = &ARCHETYPES[2]; // kofn-recovery (sane)
    let out = bin()
        .args(["build-descriptor"])
        .args(a.preset_args)
        .args(["--allow", "repeated-keys", "--json"])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert!(
        v.get("allowed_rules_fired").is_none(),
        "no fired key when nothing fired"
    );
    assert!(
        v["cost"].is_object(),
        "cost runs normally when nothing fired"
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("did not fire"),
        "unused-allowance note: {stderr}"
    );
}

/// Preset composition (allow SPEC §0): kofn + duplicate --key +
/// --allow repeated-keys → success.
#[test]
fn allow_composes_with_preset_mode() {
    bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K1,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--older",
            "52560",
            "--allow",
            "repeated-keys",
            "--format",
            "descriptor",
        ])
        .assert()
        .success()
        .stderr(predicates::str::contains("repeated-keys"));
}

/// --emit-spec interaction (allow SPEC §3/§5, R0-r1 I2): the emitted spec
/// records NO allowance — replay without --allow refuses.
#[test]
fn emit_spec_records_no_allowance() {
    let out = bin()
        .args([
            "build-descriptor",
            "--archetype",
            "kofn-recovery",
            "--key",
            K1,
            "--key",
            K1,
            "--threshold",
            "2",
            "--recovery-key",
            K4,
            "--older",
            "52560",
            "--allow",
            "repeated-keys",
            "--emit-spec",
        ])
        .assert()
        .success();
    let emitted = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        !emitted.contains("allow"),
        "spec document must not record allowances"
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("repeated-keys"),
        "banner on the emit-spec run itself"
    );

    let out = bin()
        .args(["build-descriptor", "--spec", "-", "--json"])
        .write_stdin(emitted)
        .assert()
        .code(2);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["diagnostics"][0]["kind"], "repeated_keys");
    assert!(
        v["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("rerun with --allow repeated-keys after review"),
        "refusal hint on the replay (impl-r1 M1)"
    );
}

/// bip388 shape on an allowed repeated-keys emit (R0-r1 M5): duplicate
/// keys_info entries, no dedup — pinned.
#[test]
fn allow_repeated_keys_bip388_duplicate_keys_info() {
    let out = bin()
        .args([
            "build-descriptor",
            "--allow",
            "repeated-keys",
            "--format",
            "bip388",
        ])
        .write_stdin(repeated_keys_spec())
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let keys = v["keys_info"].as_array().unwrap();
    assert_eq!(
        keys.len(),
        4,
        "duplicate keys appear twice (no dedup): {keys:?}"
    );
}

/// Duplicate --allow tokens are idempotent (allow SPEC §1).
#[test]
fn allow_duplicate_tokens_idempotent() {
    let out = bin()
        .args([
            "build-descriptor",
            "--allow",
            "sigless-branch",
            "--allow",
            "sigless-branch",
            "--format",
            "descriptor",
        ])
        .write_stdin(SIGLESS_SPEC)
        .assert()
        .success();
    let v_count = String::from_utf8(out.get_output().stderr.clone())
        .unwrap()
        .matches("sigless-branch")
        .count();
    assert_eq!(v_count, 1, "banner names the rule once");
}
