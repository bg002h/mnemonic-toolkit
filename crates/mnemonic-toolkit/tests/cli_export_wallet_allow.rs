//! `mnemonic export-wallet --allow <RULE>` — the reviewed sanity opt-out,
//! ported from `build-descriptor` to the export surface.
//!
//! Realizes Phase 1 of
//! `mnemonic-engrave/design/PLAN_wallet_file_export.md` (R0 GREEN after five
//! rounds). The rulings this file pins, and which round settled each:
//!
//! * **(b), round 2** — `sigless-branch` is the ONLY rule enforced here. The
//!   other four parse (one shared vocabulary) but never run, and say so.
//! * **Topology (B), round 3** — ONE admission gate, on each arm's canonical
//!   descriptor, at `export-wallet`'s two `EmitInputs` construction sites. No
//!   arm routes around it; `cmd/restore.rs`'s two `EmitInputs` builders are out
//!   of scope and unchanged.
//! * **R3-2** — a note may never claim a check that did not run. The
//!   *"passes that rule"* parenthetical belongs only to a rule that ran.
//! * **R4-1** — the note matrix has no arm dimension in row 2, and no
//!   ungated-path cell at all.
//!
//! **What this does NOT do.** `--allow` enables EMISSION. Bitcoin Core refuses
//! this wallet at `importdescriptors` on every version through v31.1 and the
//! rule is non-waivable there; Nunchuk and Sparrow refuse it too. Nothing in
//! this file, the help text or the code may say otherwise.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/export_wallet_allow").join(name)
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture(name))
        .unwrap_or_else(|e| panic!("fixture {name}: {e}"))
        .trim()
        .to_string()
}

/// The reasonably-complex wallet's own concrete `wsh` descriptor. Tier 4 is
/// `and_v(v:after(1383520),sha256(H))` — an absolute timelock AND a hashlock,
/// and no key at all. That single keyless branch is the whole subject here.
fn rcw_wsh() -> String {
    read_fixture("rcw_wsh_descriptor.txt")
}

/// The same wallet in its `tr` form. Identical tier structure, `multi_a`
/// leaves, NUMS internal key.
fn rcw_tr() -> String {
    read_fixture("rcw_tr_descriptor.txt")
}

/// A plain 2-of-3 `sh(multi(...))` — every spend path needs signatures, so the
/// gate runs on it and does NOT fire. The "sane" control.
const SANE_DESCRIPTOR: &str = "sh(multi(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[5436d724/48'/0'/0'/2']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))#ek6d38cp";

/// BIP-48 cosigner xpubs for the `--template` / `--slot` arm.
const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";

/// The four rules ruling (b) leaves UNENFORCED on this surface.
const UNENFORCED: [&str; 4] = [
    "malleable",
    "mixed-timelock",
    "repeated-keys",
    "resource-limit",
];

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

/// Every `--allow` note line on stderr, in order — the note matrix's
/// observable. Filters out the watch-only advisory, the mk1 substitution
/// notice, and the timelock advisories, which are not part of this surface.
fn allow_notes(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .filter(|l| l.starts_with("note: --allow") || l.starts_with("WARNING: sanity rules"))
        .map(|l| l.to_string())
        .collect()
}

// ============================================================================
// A. Baseline cells, per wrapper, `--format bitcoin-core` (round-3 R3-3: it
//    is the default, and the format with measured evidence on BOTH sides of
//    the air gap).
// ============================================================================

/// **BEHAVIOUR CHANGE, round-2 finding F-1.** This exact invocation exits 0
/// today (2694 bytes of shape-perfect JSON) and must now refuse. The wsh hole
/// is a defect to CLOSE, not a behaviour to pin: before Phase 1 the sigless
/// rule reached `tr` only, because that is where `rust-miniscript`'s
/// `Descriptor::from_str` happens to check it — a flag whose reach depends on
/// an upstream parser's shape is not a flag anyone can reason about.
#[test]
fn flagless_sigless_wsh_now_refuses() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &rcw_wsh(),
        "--format",
        "bitcoin-core",
    ]);
    assert_eq!(r.code, 2, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(
        r.stderr.contains("--allow sigless-branch"),
        "the refusal must name the exact flag that waives it: {}",
        r.stderr
    );
    assert!(
        r.stdout.is_empty(),
        "refusal must emit nothing: {}",
        r.stdout
    );
}

/// The `tr` form refused before Phase 1 too — but at the intake parse, with
/// miniscript's own message and no way to proceed. Now it refuses at the
/// admission gate, and the message tells the operator what to do.
#[test]
fn flagless_sigless_tr_refuses_and_names_the_flag() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &rcw_tr(),
        "--format",
        "bitcoin-core",
    ]);
    assert_eq!(r.code, 2, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(
        r.stderr.contains("--allow sigless-branch"),
        "the refusal must name the exact flag that waives it: {}",
        r.stderr
    );
}

#[test]
fn sigless_wsh_exports_with_the_flag() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &rcw_wsh(),
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("importdescriptors JSON");
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 2, "receive + change");
    assert!(arr[0]["desc"].as_str().unwrap().starts_with("wsh(or_i("));
    assert_eq!(arr[1]["internal"], serde_json::json!(true));
}

/// The `:524` intake going lenient is what lets a `tr` form reach the gate at
/// all (round-4 finding R4-2). An implementer who leaves it strict fails
/// exactly here.
#[test]
fn sigless_tr_exports_with_the_flag() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &rcw_tr(),
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("importdescriptors JSON");
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 2, "receive + change");
    assert!(arr[0]["desc"].as_str().unwrap().starts_with("tr("));
}

/// Round-1 finding M2: build's warning says *don't author this*; export's must
/// say *understand what watching this means*. Same vocabulary, different act —
/// so the build-side sentence must NOT appear here.
#[test]
fn fired_warning_speaks_the_export_act_not_the_authoring_act() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &rcw_wsh(),
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert!(
        r.stderr
            .contains("WARNING: sanity rules OVERRIDDEN by --allow and FIRED: sigless-branch"),
        "the fired rule must be named unmissably: {}",
        r.stderr
    );
    assert!(
        !r.stderr
            .contains("failed miniscript's funds-safety analysis"),
        "build-descriptor's authoring wording must not leak onto the export surface: {}",
        r.stderr
    );
    assert!(
        r.stderr
            .contains("anyone who learns the descriptor can move the funds"),
        "export's wording must state what watching this wallet means: {}",
        r.stderr
    );
}

/// The rule RAN on this wallet and did not fire — so "passes that rule" is a
/// true claim here, and the only place it may be printed.
#[test]
fn requested_but_not_fired_note_on_a_sane_wallet() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        SANE_DESCRIPTOR,
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    assert_eq!(
        allow_notes(&r.stderr),
        vec![
            "note: --allow sigless-branch was requested but did not fire (the descriptor passes that rule without it)"
        ],
    );
}

// ============================================================================
// B. `--from-import-json` — gated exactly like `--descriptor`.
//    This is the hole round 3 found (R3-1): under round 2's text a sigless wsh
//    envelope exited 0 with no flag, which would have falsified "the wsh hole
//    closed rather than pinned" in the same document that claimed it.
// ============================================================================

#[test]
fn from_import_json_sigless_wsh_refuses_flagless() {
    let p = fixture("envelope_sigless_wsh.json");
    let r = run(&[
        "export-wallet",
        "--from-import-json",
        p.to_str().unwrap(),
        "--format",
        "bitcoin-core",
    ]);
    assert_eq!(r.code, 2, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(
        r.stderr.contains("--allow sigless-branch"),
        "envelope arm must refuse with the same affordance as --descriptor: {}",
        r.stderr
    );
}

#[test]
fn from_import_json_sigless_wsh_exports_with_the_flag() {
    let p = fixture("envelope_sigless_wsh.json");
    let r = run(&[
        "export-wallet",
        "--from-import-json",
        p.to_str().unwrap(),
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("importdescriptors JSON");
    assert_eq!(v.as_array().unwrap().len(), 2);
    assert!(r.stderr.contains("WARNING: sanity rules OVERRIDDEN"));
}

/// **Do not "fix" this test by relaxing `Fix-α`.** A taproot envelope is
/// refused by the v0.28.7 `Fix-α` gate — the wallet-import wire shape cannot
/// carry a taproot internal-key designation — and that refusal is categorical,
/// with or without `--allow`. Nobody has ruled on lifting it; that is its own
/// decision with its own release note.
#[test]
fn from_import_json_taproot_stays_refused_by_fix_alpha_with_or_without_allow() {
    let p = fixture("envelope_sigless_tr.json");
    for extra in [vec![], vec!["--allow", "sigless-branch"]] {
        let mut args = vec![
            "export-wallet",
            "--from-import-json",
            p.to_str().unwrap(),
            "--format",
            "bitcoin-core",
        ];
        args.extend(extra.iter().copied());
        let r = run(&args);
        assert_eq!(r.code, 1, "stdout={} stderr={}", r.stdout, r.stderr);
        assert!(
            r.stderr.contains(
                "taproot descriptors are not yet supported on the export-from-envelope path"
            ),
            "must refuse for the Fix-\u{3b1} reason, not a sanity-parse accident: {}",
            r.stderr
        );
    }
}

// ============================================================================
// C. Over-admission — round-1 finding I4. Every test in the round-1 list
//    checked that the flag PERMITS; none checked that it permits ONLY what was
//    asked. A granularity lapse is invisible to all of them.
// ============================================================================

#[test]
fn requesting_one_rule_admits_no_other() {
    let envelope = fixture("envelope_sigless_wsh.json");
    for rule in UNENFORCED {
        let r = run(&[
            "export-wallet",
            "--descriptor",
            &rcw_wsh(),
            "--format",
            "bitcoin-core",
            "--allow",
            rule,
        ]);
        assert_eq!(
            r.code, 2,
            "--allow {rule} must not admit a sigless branch: stdout={} stderr={}",
            r.stdout, r.stderr
        );

        let r = run(&[
            "export-wallet",
            "--from-import-json",
            envelope.to_str().unwrap(),
            "--format",
            "bitcoin-core",
            "--allow",
            rule,
        ]);
        assert_eq!(
            r.code, 2,
            "--allow {rule} must not admit a sigless envelope: stdout={} stderr={}",
            r.stdout, r.stderr
        );
    }
}

// ============================================================================
// D. The transforming emitters — round-1 finding I4(b). `bip388` REWRITES the
//    descriptor rather than passing it through, and it is reached only AFTER
//    the relaxation. Before Phase 1 no keyless-leaf vector could exist for it.
// ============================================================================

#[test]
fn keyless_leaf_survives_the_bip388_transforming_emitter() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &rcw_wsh(),
        "--format",
        "bip388",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "stderr={}", r.stderr);
    let v: serde_json::Value = serde_json::from_str(&r.stdout).expect("bip388 wallet policy");
    let tmpl = v["description_template"]
        .as_str()
        .expect("description_template");
    assert!(
        tmpl.contains(
            "and_v(v:after(1383520),sha256(4743d7c47df21d29e3ed3dfec5d0c0a884ccc2708637dddf771c36d214056954))"
        ),
        "the keyless tier-4 leaf must survive the rewrite verbatim: {tmpl}"
    );
    // The transform must not have invented a key for the keyless branch.
    let keys = v["keys_info"].as_array().expect("keys_info");
    assert_eq!(keys.len(), 6, "six cosigner keys, none of them tier 4's");
}

#[test]
fn bip388_keyless_leaf_still_refuses_without_the_flag() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &rcw_wsh(),
        "--format",
        "bip388",
    ]);
    assert_eq!(r.code, 2, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(r.stderr.contains("--allow sigless-branch"), "{}", r.stderr);
}

// ============================================================================
// E. The note matrix — 2 rule-classes x 3 arms = 6 cells, every one asserted.
//    Rounds 3 and 4 both found note-wording defects and both were COMPOSITION
//    failures: a cell that was false, or a cell with no referent. Prose hid
//    them because prose has no cells.
// ============================================================================

/// The `--template` / `--slot` arm. Row 1, column 3: the did-not-fire note,
/// and ONLY that. Not an exemption — a builder-produced descriptor cannot carry
/// a sigless branch, so the rule runs and cannot fire.
fn template_arm_args(allow: &str) -> Vec<&str> {
    vec![
        "export-wallet",
        "--format",
        "bitcoin-core",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--multisig-path-family",
        "bip48",
        "--network",
        "mainnet",
        "--slot",
        "@0.xpub=xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX",
        "--slot",
        "@0.fingerprint=b8688df1",
        "--slot",
        "@0.path=m/48'/0'/0'/2'",
        "--slot",
        "@1.xpub=xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6",
        "--slot",
        "@1.fingerprint=28645006",
        "--slot",
        "@1.path=m/48'/0'/0'/2'",
        "--allow",
        allow,
    ]
}

/// Row 1 of the matrix: `--allow sigless-branch`, per arm.
/// `--descriptor` and `--from-import-json` can each produce EITHER outcome;
/// `--template`/`--slot` can only produce the did-not-fire note. Row 1's
/// columns are deliberately NOT identical (round 5 caught the earlier wording
/// that claimed they were, falsified by the grid printed one line above it).
#[test]
fn note_matrix_row1_allow_sigless_branch_every_arm() {
    const FIRED: &str = "WARNING: sanity rules OVERRIDDEN by --allow and FIRED: sigless-branch";
    const NOT_FIRED: &str = "note: --allow sigless-branch was requested but did not fire (the descriptor passes that rule without it)";

    // Cell (1,1a) --descriptor, rule fires.
    let r = run(&[
        "export-wallet",
        "--descriptor",
        &rcw_wsh(),
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(allow_notes(&r.stderr)[0].starts_with(FIRED), "{}", r.stderr);

    // Cell (1,1b) --descriptor, rule runs and does not fire.
    let r = run(&[
        "export-wallet",
        "--descriptor",
        SANE_DESCRIPTOR,
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(allow_notes(&r.stderr), vec![NOT_FIRED]);

    // Cell (1,2) --from-import-json, rule fires.
    let p = fixture("envelope_sigless_wsh.json");
    let r = run(&[
        "export-wallet",
        "--from-import-json",
        p.to_str().unwrap(),
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(allow_notes(&r.stderr)[0].starts_with(FIRED), "{}", r.stderr);

    // Cell (1,3) --template/--slot: the gate RUNS and CANNOT fire.
    let r = run(&template_arm_args("sigless-branch"));
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(
        allow_notes(&r.stderr),
        vec![NOT_FIRED],
        "the template arm is gated like every other; it just cannot trip the rule"
    );
}

/// Row 2 of the matrix: `--allow <other four>`, per arm. Arm-independent by
/// construction — and R3-2's ruling made visible as a row: these rules never
/// run here, so they get their own wording and may not borrow the
/// "passes that rule" claim.
#[test]
fn note_matrix_row2_unenforced_rule_every_arm() {
    let p = fixture("envelope_sigless_wsh.json");
    for rule in UNENFORCED {
        let expected = format!(
            "note: --allow {rule} has no effect on export-wallet — only sigless-branch is \
             enforced here; the descriptor was NOT checked against {rule}"
        );

        // Column 1 — --descriptor.
        let r = run(&[
            "export-wallet",
            "--descriptor",
            SANE_DESCRIPTOR,
            "--format",
            "bitcoin-core",
            "--allow",
            rule,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert_eq!(
            allow_notes(&r.stderr),
            vec![expected.clone()],
            "arm=--descriptor rule={rule}"
        );

        // Column 2 — --from-import-json (with the flag that admits the wallet,
        // so the note is reachable at all).
        let r = run(&[
            "export-wallet",
            "--from-import-json",
            p.to_str().unwrap(),
            "--format",
            "bitcoin-core",
            "--allow",
            "sigless-branch",
            "--allow",
            rule,
        ]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(
            allow_notes(&r.stderr).contains(&expected),
            "arm=--from-import-json rule={rule}: {}",
            r.stderr
        );

        // Column 3 — --template/--slot.
        let mut args = template_arm_args(rule);
        // template_arm_args already ends with ["--allow", rule].
        args.push("--allow");
        args.push("sigless-branch");
        let r = run(&args);
        assert_eq!(r.code, 0, "{}", r.stderr);
        assert!(
            allow_notes(&r.stderr).contains(&expected),
            "arm=--template rule={rule}: {}",
            r.stderr
        );
    }
}

/// **The uniform gate's observable signature.** Row 2's three columns are
/// identical by construction; if a future edit makes one differ, topology (B)
/// has been broken somewhere. Asserted as byte-equality of the note line, not
/// as a substring, so a divergence cannot hide in wording.
#[test]
fn note_matrix_row2_columns_are_identical() {
    let p = fixture("envelope_sigless_wsh.json");
    for rule in UNENFORCED {
        let from_descriptor = run(&[
            "export-wallet",
            "--descriptor",
            SANE_DESCRIPTOR,
            "--format",
            "bitcoin-core",
            "--allow",
            rule,
        ]);
        let from_envelope = run(&[
            "export-wallet",
            "--from-import-json",
            p.to_str().unwrap(),
            "--format",
            "bitcoin-core",
            "--allow",
            rule,
            "--allow",
            "sigless-branch",
        ]);
        let mut targs = template_arm_args(rule);
        targs.push("--allow");
        targs.push("sigless-branch");
        let from_template = run(&targs);

        let pick = |r: &Run| -> String {
            allow_notes(&r.stderr)
                .into_iter()
                .find(|l| l.contains(&format!("--allow {rule} ")))
                .unwrap_or_else(|| panic!("no note for {rule}: {}", r.stderr))
        };
        let a = pick(&from_descriptor);
        let b = pick(&from_envelope);
        let c = pick(&from_template);
        assert_eq!(a, b, "rule={rule}: --descriptor vs --from-import-json");
        assert_eq!(a, c, "rule={rule}: --descriptor vs --template");
    }
}

/// R3-2, asserted per unenforced rule and on every arm: the
/// *"passes that rule"* parenthetical may only ever be printed by a rule that
/// actually RAN. Under ruling (b) these four never run, so for them the
/// sentence is not a lenient claim — it is a false one.
#[test]
fn passes_that_rule_is_never_printed_for_a_rule_that_did_not_run() {
    let p = fixture("envelope_sigless_wsh.json");
    for rule in UNENFORCED {
        let runs = [
            run(&[
                "export-wallet",
                "--descriptor",
                SANE_DESCRIPTOR,
                "--format",
                "bitcoin-core",
                "--allow",
                rule,
            ]),
            run(&[
                "export-wallet",
                "--from-import-json",
                p.to_str().unwrap(),
                "--format",
                "bitcoin-core",
                "--allow",
                rule,
                "--allow",
                "sigless-branch",
            ]),
            run(&template_arm_args(rule)),
        ];
        for r in runs {
            // Guard against a false PASS: an invocation that never ran the
            // printer trivially satisfies every "must not contain" below.
            assert_eq!(
                r.code, 0,
                "{rule}: the run itself must succeed: {}",
                r.stderr
            );
            let notes = allow_notes(&r.stderr);
            assert!(
                notes
                    .iter()
                    .any(|l| l.contains(&format!("--allow {rule} "))),
                "{rule}: the unenforced-rule note must actually be printed: {}",
                r.stderr
            );
            let notes = notes.join("\n");
            let claim = format!("--allow {rule} was requested but did not fire");
            assert!(
                !notes.contains(&claim),
                "{rule}: must not report a fire-verdict for a rule that never ran: {notes}"
            );
            assert!(
                !notes
                    .lines()
                    .any(|l| l.contains(&format!("--allow {rule} "))
                        && l.contains("passes that rule")),
                "{rule}: must not claim the descriptor passes an unchecked rule: {notes}"
            );
        }
    }
}

// ============================================================================
// F. `cmd/restore.rs` is OUT OF SCOPE — round-4 finding R4-2. The shared
//    pre-`EmitInputs` boundary also serves restore's two production
//    constructors, and one compliant reading of "where all three arms
//    converge" would silently break a shipped, waiver-less surface.
// ============================================================================

/// Measured at toolkit tip `5f88071c` BEFORE this change:
/// `restore --md1 --format bitcoin-core` on this wallet's sigless wsh emits
/// flagless at exit 0, 2694 bytes. It must still do so — and its output must
/// still equal what `export-wallet` emits for the same wallet WITH the flag,
/// which is what proves restore was not gated rather than merely not crashed.
#[test]
fn restore_md1_sigless_wsh_still_emits_flagless() {
    let md1 = read_fixture("rcw_wsh_md1_keyed.txt");
    let mut args: Vec<String> = vec!["restore".into()];
    for chunk in md1.lines().filter(|l| !l.trim().is_empty()) {
        args.push("--md1".into());
        args.push(chunk.trim().into());
    }
    args.push("--format".into());
    args.push("bitcoin-core".into());
    let argv: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let r = run(&argv);
    assert_eq!(
        r.code, 0,
        "restore must NOT acquire an admission gate: stdout={} stderr={}",
        r.stdout, r.stderr
    );
    assert!(
        allow_notes(&r.stderr).is_empty(),
        "restore must emit no --allow notes at all: {}",
        r.stderr
    );

    let via_export = run(&[
        "export-wallet",
        "--descriptor",
        &rcw_wsh(),
        "--format",
        "bitcoin-core",
        "--allow",
        "sigless-branch",
    ]);
    assert_eq!(via_export.code, 0, "{}", via_export.stderr);
    assert_eq!(
        r.stdout, via_export.stdout,
        "restore's FLAGLESS payload must byte-match export's FLAGGED payload — \
         same wallet, and restore never asks for the waiver"
    );
}

/// Phase 1 adds `--allow` to `export-wallet` and to nothing else. `restore`
/// must not grow the flag by accident (which would be the first step toward
/// gating it).
#[test]
fn restore_has_no_allow_flag() {
    let r = run(&["restore", "--allow", "sigless-branch", "--from", "phrase=x"]);
    // 64 = this binary's clap-usage exit code (main.rs remaps clap's 2 to 64 so
    // usage errors stay distinct from format violations).
    assert_eq!(
        r.code, 64,
        "clap should reject an unknown argument: {}",
        r.stderr
    );
    assert!(
        r.stderr.contains("unexpected argument") || r.stderr.contains("--allow"),
        "{}",
        r.stderr
    );
}

// ============================================================================
// G. The constraint that survives all of this.
// ============================================================================

/// **No help text, doc, commit message or release note may say `--allow`
/// "enables export to Core / Nunchuk / Sparrow".** It enables EMISSION.
/// Measured: Core refuses the result at import on every version through v31.1
/// and the rule is non-waivable there; Nunchuk refuses via libnunchuk's
/// `IsSane()`/`NeedsSignature()`; Sparrow has no miniscript engine at all.
#[test]
fn help_text_never_claims_it_enables_import_into_a_wallet_app() {
    let r = run(&["export-wallet", "--help"]);
    assert_eq!(r.code, 0);
    let help = r.stdout.to_lowercase();
    assert!(help.contains("--allow"), "the flag must be documented");
    for forbidden in [
        "enables export to",
        "enable export to",
        "makes bitcoin core accept",
        "allows import into",
    ] {
        assert!(
            !help.contains(forbidden),
            "help text must not claim {forbidden:?}: {}",
            r.stdout
        );
    }
    assert!(
        help.contains("emission") || help.contains("emit"),
        "the help text must say what the flag actually does — permit EMISSION: {}",
        r.stdout
    );
}

/// The vocabulary is shared with `build-descriptor`: all five values parse on
/// both surfaces. (b) is about which ones are ENFORCED, not which ones exist.
#[test]
fn all_five_rule_names_parse_on_the_export_surface() {
    for rule in [
        "malleable",
        "mixed-timelock",
        "repeated-keys",
        "resource-limit",
        "sigless-branch",
    ] {
        let r = run(&[
            "export-wallet",
            "--descriptor",
            SANE_DESCRIPTOR,
            "--format",
            "bitcoin-core",
            "--allow",
            rule,
        ]);
        assert_eq!(r.code, 0, "--allow {rule}: {}", r.stderr);
    }
    let r = run(&[
        "export-wallet",
        "--descriptor",
        SANE_DESCRIPTOR,
        "--format",
        "bitcoin-core",
        "--allow",
        "no-such-rule",
    ]);
    assert_eq!(
        r.code, 64,
        "an unknown rule is a clap usage error: {}",
        r.stderr
    );
}

/// A sanity check on the fixtures themselves: the two constants used as the
/// "sane" control really are sane, and the `--template` cosigners really are
/// the ones the arm builds with. Guards against a control that silently stops
/// being a control (a positive-control failure is what saved the round-1
/// Bitcoin Core measurement).
#[test]
fn the_sane_control_is_actually_sane() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        SANE_DESCRIPTOR,
        "--format",
        "bitcoin-core",
    ]);
    assert_eq!(
        r.code, 0,
        "the control must export with NO flag at all, or every row-2 cell is testing a refusal: {}",
        r.stderr
    );
    assert!(allow_notes(&r.stderr).is_empty());
    assert!(COSIGNER_A_XPUB.starts_with("xpub") && COSIGNER_B_XPUB.starts_with("xpub"));
    assert_eq!(COSIGNER_A_FP.len(), 8);
    assert_eq!(COSIGNER_B_FP.len(), 8);
}

// ============================================================================
// H. "The ONLY admission point, with no arm routing around it" — asserted
//    rather than asserted-about. The observable is that EVERY format on EVERY
//    arm meets the same gate, with the same message, before any
//    format-specific verdict.
// ============================================================================

/// All eleven `--format` values, one sigless wallet, no flag: the gate's
/// message, every time. If any format ever answers with its own refusal here,
/// something reached an emitter without passing the gate.
#[test]
fn every_format_meets_the_same_gate_before_its_own_verdict() {
    const FORMATS: [&str; 11] = [
        "bitcoin-core",
        "bip388",
        "coldcard",
        "coldcard-multisig",
        "jade",
        "sparrow",
        "specter",
        "electrum",
        "green",
        "bsms",
        "descriptor",
    ];
    let wsh = rcw_wsh();
    for f in FORMATS {
        let r = run(&["export-wallet", "--descriptor", &wsh, "--format", f]);
        assert_eq!(
            r.code, 2,
            "--format {f}: stdout={} stderr={}",
            r.stdout, r.stderr
        );
        assert!(
            r.stderr
                .contains("this wallet has a spend path that requires no signature"),
            "--format {f} must meet the ADMISSION gate, not its own verdict: {}",
            r.stderr
        );
    }
}

/// With the flag, admission is over and each format answers on its own terms
/// again. Three faithful formats emit; the rest refuse for the reasons they
/// always refused. Pinned so a future edit cannot quietly turn the gate into a
/// format filter, or a format refusal into an admission decision.
#[test]
fn with_the_flag_each_format_falls_through_to_its_own_verdict() {
    let wsh = rcw_wsh();
    for f in ["bitcoin-core", "bip388", "descriptor"] {
        let r = run(&[
            "export-wallet",
            "--descriptor",
            &wsh,
            "--format",
            f,
            "--allow",
            "sigless-branch",
        ]);
        assert_eq!(r.code, 0, "--format {f}: {}", r.stderr);
    }
    for (f, needle) in [
        ("green", "does not support multisig"),
        ("specter", "requires the following missing fields"),
        (
            "sparrow",
            "requires --template; descriptor passthrough is not supported",
        ),
    ] {
        let r = run(&[
            "export-wallet",
            "--descriptor",
            &wsh,
            "--format",
            f,
            "--allow",
            "sigless-branch",
        ]);
        assert_ne!(r.code, 0, "--format {f} should still refuse: {}", r.stdout);
        assert!(r.stderr.contains(needle), "--format {f}: {}", r.stderr);
        assert!(
            !r.stderr
                .contains("requires no signature (anyone-can-spend)"),
            "--format {f} must be past admission by now: {}",
            r.stderr
        );
    }
}

/// PLAN §2: the constellation CANNOT produce the file Sparrow would silently
/// misimport as a `sortedmulti` 3-of-6 with WRONG ADDRESSES, because the
/// Sparrow emitter refuses descriptor passthrough. Phase 1 must not have
/// disturbed that on a SANE wallet (a sigless one is now caught earlier, by the
/// gate). Phase 3 turns that incidental safety into a deliberate one; this cell
/// only guards it in the meantime.
#[test]
fn sparrow_descriptor_passthrough_refusal_is_untouched_for_a_sane_wallet() {
    let r = run(&[
        "export-wallet",
        "--descriptor",
        SANE_DESCRIPTOR,
        "--format",
        "sparrow",
    ]);
    assert_eq!(r.code, 1, "stdout={} stderr={}", r.stdout, r.stderr);
    assert!(
        r.stderr
            .contains("requires --template; descriptor passthrough is not supported"),
        "{}",
        r.stderr
    );
}

/// Open Q4 (Rust-first): Phase 1 touches the tr/wsh sanity asymmetry the
/// CLI-surface report flags as potentially normative, so the new refusals are
/// pinned HERE, in the primary Rust repo, with vectors — before any Go port
/// tracks them. The vector is (wrapper, flag) -> verdict, on this wallet's own
/// two concrete descriptors.
#[test]
fn rust_first_vectors_for_the_new_refusals() {
    let cases: [(&str, String, bool, i32); 4] = [
        ("wsh", rcw_wsh(), false, 2),
        ("wsh", rcw_wsh(), true, 0),
        ("tr", rcw_tr(), false, 2),
        ("tr", rcw_tr(), true, 0),
    ];
    for (label, desc, flagged, want) in cases {
        let mut args = vec![
            "export-wallet",
            "--descriptor",
            desc.as_str(),
            "--format",
            "bitcoin-core",
        ];
        if flagged {
            args.push("--allow");
            args.push("sigless-branch");
        }
        let r = run(&args);
        assert_eq!(
            r.code, want,
            "vector {label} flagged={flagged}: stdout={} stderr={}",
            r.stdout, r.stderr
        );
    }
}
