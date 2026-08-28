//! P3 rows 14 and 15 — THE `mnemonic` ARGV REFUSAL, PRE-PARSER, and its
//! override (`SPEC_constellation_cli_uniformity` §6d, §6h).
//!
//! Before this row, every one of `mnemonic`'s argv-secret channels **warned and
//! proceeded**. §6d rules that a refusal, and rules the *ordering* normative:
//! the decision is reached on raw `std::env::args()` before `Cli::try_parse()`
//! runs, because a guard downstream of the parser has already lost — clap
//! echoes what it cannot place.
//!
//! # What each test here is FOR, so a later reader does not delete one as
//! # redundant
//!
//! - The five channels §7's P3 row names are **spot checks**, not the boundary.
//!   The spec says so in as many words, and F-292 records what the boundary
//!   actually measures to: 48 advisory call sites across 20 source files in
//!   eleven distinct argv-material shapes.
//! - [`argv_parity_covers_every_secret_node_type`] is the boundary assertion,
//!   and it is **generated from `SECRET_NODE_TYPES_ARGV`** — the same
//!   `pub const` the toolkit's own `is_argv_secret_bearing` predicate is held
//!   in lockstep with — so a tenth token cannot be added without this test
//!   seeing it.
//! - The two **controls** exist because refusing `-` or `@env:` would remove
//!   the very remedy the refusal points at.
//! - The §6h test resolves every channel the refusal advises against the
//!   binary's own `gui-schema`. A remedy naming a flag that does not exist is
//!   the defect §6h was written from, and it is not catchable by reading.

use assert_cmd::Command;
use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES_ARGV, SECRET_SLOT_SUBKEYS};

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// `mnemonic`'s refusal family exit code. Not 64 (clap's usage code — this
/// happens *instead of* parsing) and not 3 (already `FutureFormat` here).
const EXIT_REFUSED: i32 = 2;

fn run(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        // `/dev/null` on stdin: a refusal must not depend on, or wait for, a
        // stream. Without this a guard that read stdin first would hang here
        // rather than fail.
        .write_stdin("")
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

fn assert_refused(args: &[&str], material: &str, what: &str) -> String {
    let (stdout, stderr, code) = run(args);
    assert_eq!(
        code, EXIT_REFUSED,
        "{what}: expected the argv refusal at exit {EXIT_REFUSED}; stderr was:\n{stderr}"
    );
    assert!(
        stdout.is_empty(),
        "{what}: a refusal must write NOTHING to stdout; got:\n{stdout}"
    );
    assert!(
        stderr.contains("Refused BEFORE the command line was parsed"),
        "{what}: the refusal must state its ordering; got:\n{stderr}"
    );
    assert!(
        !stderr.contains(material),
        "{what}: THE REFUSAL ECHOED THE MATERIAL -- that is the leak it exists \
         to stop, moved into the message. stderr:\n{stderr}"
    );
    stderr
}

/// The binary's own flag surface, as `{subcommand: [flag, …]}`.
fn schema_flags() -> std::collections::BTreeMap<String, Vec<String>> {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("gui-schema")
        .assert()
        .success();
    let json: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    json["subcommands"]
        .as_array()
        .expect("subcommands")
        .iter()
        .map(|s| {
            (
                s["name"].as_str().unwrap_or_default().to_string(),
                s["flags"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|f| f["name"].as_str().map(str::to_string))
                    .collect(),
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Row 14 — the refusal
// ---------------------------------------------------------------------------

/// §7's five named channels, each measured emitting a warning and **proceeding**
/// before this row. `bundle` carries two of them and therefore reports two
/// findings in one refusal.
#[test]
fn the_five_named_channels_all_refuse() {
    let secret = "correct-horse-battery-staple";

    let stderr = assert_refused(
        &[
            "bundle",
            "--slot",
            &format!("@0.phrase={PHRASE}"),
            "--passphrase",
            secret,
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ],
        secret,
        "bundle --slot @0.phrase= + --passphrase",
    );
    assert!(
        !stderr.contains("abandon"),
        "bundle: the phrase leaked into the refusal:\n{stderr}"
    );
    assert!(
        stderr.contains("--slot @0.phrase=") && stderr.contains("--passphrase"),
        "bundle carries TWO argv channels and must report both:\n{stderr}"
    );

    assert_refused(
        &[
            "convert",
            "--passphrase",
            secret,
            "--from",
            "xpub=x",
            "--to",
            "address",
        ],
        secret,
        "convert --passphrase",
    );
    assert_refused(
        &["derive-child", "--passphrase", secret],
        secret,
        "derive-child --passphrase",
    );
    assert_refused(
        &["restore", "--passphrase", secret],
        secret,
        "restore --passphrase",
    );
    assert_refused(
        &["electrum-decrypt", "--decrypt-password", secret],
        secret,
        "electrum-decrypt --decrypt-password",
    );
}

/// **THE BOUNDARY, GENERATED FROM THE PREDICATE'S OWN TOKEN SET.**
///
/// §7 rules that `mnemonic` keys its refusal off the existing
/// `is_argv_secret_bearing` predicate rather than building a second one that
/// could drift. `SECRET_NODE_TYPES_ARGV` is the token set that predicate is
/// held in lockstep with, so iterating it here is what makes "the predicate is
/// the boundary" checkable rather than asserted.
#[test]
fn argv_parity_covers_every_secret_node_type() {
    assert_eq!(
        SECRET_NODE_TYPES_ARGV.len(),
        9,
        "the token set moved; this test follows it, but the count is pinned so \
         the MOVE is visible in a diff"
    );
    for token in SECRET_NODE_TYPES_ARGV {
        let material = "some-secret-material";
        let stderr = assert_refused(
            &[
                "convert",
                "--from",
                &format!("{token}={material}"),
                "--to",
                "xpub",
                "--template",
                "bip84",
            ],
            material,
            &format!("convert --from {token}="),
        );
        assert!(
            stderr.contains(&format!("--from {token}=")),
            "{token}: the refusal must name the channel it refused:\n{stderr}"
        );
        assert!(
            stderr.contains(&format!("--from {token}=-")),
            "{token}: the refusal must name the private channel that replaces \
             it:\n{stderr}"
        );
    }
}

/// The same, for `--slot @N.<subkey>=`, generated from `SECRET_SLOT_SUBKEYS`.
#[test]
fn argv_parity_covers_every_secret_slot_subkey() {
    for subkey in SECRET_SLOT_SUBKEYS {
        let material = "some-secret-material";
        let stderr = assert_refused(
            &[
                "bundle",
                "--slot",
                &format!("@1.{subkey}={material}"),
                "--network",
                "mainnet",
                "--template",
                "bip84",
            ],
            material,
            &format!("bundle --slot @1.{subkey}="),
        );
        assert!(
            stderr.contains(&format!("--slot @1.{subkey}=-")),
            "{stderr}"
        );
    }
}

/// **THE TWO CONTROLS.** `-` and `@env:` are existing private channels;
/// refusing them removes the remedy the refusal points at.
#[test]
fn the_existing_private_channels_are_not_refused() {
    // `--from phrase=-` with the phrase on stdin: exit 0, real work done.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "phrase=-",
            "--to",
            "xpub",
            "--template",
            "bip84",
        ])
        .write_stdin(PHRASE)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.starts_with("xpub: "), "got {stdout:?}");

    // `@env:` resolves from the environment, so nothing is on argv.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("P3_TEST_PHRASE", PHRASE)
        .args([
            "convert",
            "--from",
            "phrase=@env:P3_TEST_PHRASE",
            "--to",
            "xpub",
            "--template",
            "bip84",
        ])
        .write_stdin("")
        .assert()
        .success();
    let via_env = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(via_env, stdout, "the two private channels must agree");
}

/// **A WATCH-ONLY NODE TYPE IS NOT REFUSED**, per §4: `md1`/`mk1` material is
/// watch-only, so a leak there costs privacy rather than the money, and `md`
/// and `mk` get no argv refusal in this cycle at all. A guard that refused
/// every `--from` would have quietly made that ruling for them.
#[test]
fn watch_only_material_still_travels_on_argv() {
    const XPUB: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={XPUB}"),
            "--to",
            "fingerprint",
        ])
        .write_stdin("")
        .assert()
        .success();
}

/// **THE ORDERING, ASSERTED — and this is the assertion that distinguishes a
/// pre-parser guard from a post-parser one.**
///
/// The invocation below is *also* invalid to clap: `--network` and `--template`
/// are required on `bundle` and are absent. A guard that ran after the parser
/// would produce clap's usage error at exit 64. A guard that runs before it
/// produces the refusal at exit 2, and clap never sees the phrase.
#[test]
fn the_refusal_wins_against_a_clap_error_on_the_same_argv() {
    let (_stdout, stderr, code) = run(&["bundle", "--slot", &format!("@0.phrase={PHRASE}")]);
    assert_eq!(
        code, EXIT_REFUSED,
        "a clap usage error (64) here means the guard is downstream of the \
         parser; stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("required arguments were not provided"),
        "clap ran first:\n{stderr}"
    );
    assert!(!stderr.contains("abandon"), "the phrase leaked:\n{stderr}");
}

/// An unknown subcommand is *also* something clap would reject first.
#[test]
fn the_refusal_wins_against_an_unknown_subcommand() {
    let (_stdout, stderr, code) = run(&["nosuchverb", "--passphrase", "hunter2"]);
    assert_eq!(code, EXIT_REFUSED, "stderr:\n{stderr}");
    assert!(!stderr.contains("hunter2"), "the value leaked:\n{stderr}");
}

/// **§6h — THE REMEDY MUST NOT FORWARD-REFERENCE A CHANNEL THAT DOES NOT
/// EXIST.** Resolved against the binary's own `gui-schema`, not against this
/// file's belief about the binary.
#[test]
fn every_channel_the_refusal_advises_actually_exists() {
    let schema = schema_flags();

    // The plan's named assertion: `--in` exists on NO `mnemonic` verb, so the
    // refusal must never advise it.
    let in_carriers: Vec<&String> = schema
        .iter()
        .filter(|(_, flags)| flags.iter().any(|f| f == "--in"))
        .map(|(name, _)| name)
        .collect();
    assert!(
        in_carriers.is_empty(),
        "`--in` now exists on {in_carriers:?}; the refusal's channel list must \
         be revisited before it may advise it"
    );

    // Every `--X-stdin` channel the guard can advise must be declared on every
    // subcommand that declares `--X`. Measured per companion flag rather than
    // assumed uniform: `--ms1-stdin` exists ONLY on the xpub-search verbs,
    // which is why the guard advises `--ms1 -` elsewhere.
    for companion in [
        "--passphrase",
        "--bip38-passphrase",
        "--decrypt-password",
        "--phrase",
        "--secret",
    ] {
        let stdin_form = format!("{companion}-stdin");
        for (sub, flags) in &schema {
            if flags.iter().any(|f| f == companion) {
                assert!(
                    flags.contains(&stdin_form),
                    "{sub} declares {companion} but not {stdin_form}; the \
                     refusal would advise a channel that does not exist there"
                );
            }
        }
    }

    // And the refusal's own text, on a real invocation, names only channels
    // that resolve.
    let stderr = assert_refused(
        &["nostr", "--secret", "nsec1qqqqq"],
        "nsec1qqqqq",
        "nostr --secret",
    );
    assert!(stderr.contains("--secret-stdin"), "{stderr}");
    assert!(!stderr.contains("--in "), "{stderr}");
}

/// The purge paragraph is the shared crate's — measured against real shells —
/// and NAMES the zsh trap without OFFERING it. Written as "no recipe offers
/// it" rather than "the text does not contain it", because the naive negative
/// fails on the warning itself.
#[test]
fn the_refusal_carries_an_executable_purge_recipe() {
    let stderr = assert_refused(
        &[
            "convert",
            "--passphrase",
            "hunter2",
            "--from",
            "xpub=x",
            "--to",
            "address",
        ],
        "hunter2",
        "purge text",
    );
    assert!(stderr.contains("TO PURGE WHAT ALREADY LEAKED"), "{stderr}");
    for shell in ["zsh:", "bash:", "fish:"] {
        assert!(stderr.contains(shell), "no recipe for {shell}:\n{stderr}");
    }
    assert!(
        stderr.contains("history -d"),
        "the zsh trap must be NAMED as a warning:\n{stderr}"
    );
    for line in stderr.lines() {
        let t = line.trim();
        if t.starts_with("zsh:") || t.starts_with("bash:") || t.starts_with("fish:") {
            assert!(!t.contains("history -d"), "a recipe OFFERS the trap: {t}");
        }
    }
    // The pattern matches on the COMMAND, never on the secret.
    assert!(stderr.contains("\\bmnemonic\\b"), "{stderr}");
}

/// The refusal reports the CLASS and the LENGTH, which is what makes it useful
/// without being a second leak.
#[test]
fn the_refusal_reports_class_and_length_not_the_value() {
    let secret = "0123456789abcdef";
    let stderr = assert_refused(
        &[
            "convert",
            "--from",
            &format!("xprv={secret}"),
            "--to",
            "xpub",
        ],
        secret,
        "class and length",
    );
    assert!(
        stderr.contains("a BIP-32 extended PRIVATE key"),
        "class missing:\n{stderr}"
    );
    assert!(
        stderr.contains(&format!("{} characters", secret.len())),
        "length missing:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// Row 15 — the override
// ---------------------------------------------------------------------------

/// The headline: the override proceeds, and produces **byte-identical** stdout
/// to the private-channel run.
///
/// Byte-equality rather than "exits 0": a `--allow-argv-secret` that admitted
/// the flag but dropped the material would exit 0 too.
#[test]
fn the_override_proceeds_and_matches_the_stdin_run_byte_for_byte() {
    let via_argv = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--allow-argv-secret",
            "--from",
            &format!("phrase={PHRASE}"),
            "--to",
            "xpub",
            "--template",
            "bip84",
        ])
        .write_stdin("")
        .assert()
        .success();
    let via_stdin = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "phrase=-",
            "--to",
            "xpub",
            "--template",
            "bip84",
        ])
        .write_stdin(PHRASE)
        .assert()
        .success();
    assert_eq!(
        via_argv.get_output().stdout,
        via_stdin.get_output().stdout,
        "the admitted material must reach the tool by the same route the stdin \
         channel uses"
    );
}

/// **THE SECOND ASSERTION, AND THE REASON IT IS HERE.**
///
/// §6d warns that leaving the admitted token in the argv handed to clap can
/// reinstate the leak: in `mt`, an unrelated clap error echoed the bearer
/// transaction. **Measured on `mnemonic`, that failure mode does not
/// reproduce** — its material is flag-borne, and clap names the flag, never the
/// value. This test pins that, so an implementation change (a new positional,
/// a clap upgrade) that started echoing would red here rather than ship.
#[test]
fn an_unrelated_clap_error_names_the_flag_and_never_the_value() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--allow-argv-secret",
            "--from",
            &format!("phrase={PHRASE}"),
            "--to",
            "xpub",
            "--template",
            "bip84",
            "--no-such-flag",
        ])
        .write_stdin("")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("--no-such-flag"),
        "clap must name the offending flag:\n{stderr}"
    );
    assert!(
        !stderr.contains("abandon"),
        "clap ECHOED THE ADMITTED MATERIAL -- the leak §6d exists to stop, \
         reinstated by the override:\n{stderr}"
    );
}

/// **THE CONTROL**: with no secret material on argv, `--allow-argv-secret` is a
/// no-op. Without this, a flag that changed behaviour on a clean invocation
/// would go unnoticed.
#[test]
fn the_override_is_a_no_op_when_nothing_is_on_argv() {
    let args = [
        "convert",
        "--from",
        "phrase=-",
        "--to",
        "xpub",
        "--template",
        "bip84",
    ];
    let plain = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .write_stdin(PHRASE)
        .assert()
        .success();
    let mut with_flag_args = vec!["convert", "--allow-argv-secret"];
    with_flag_args.extend_from_slice(&args[1..]);
    let with_flag = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&with_flag_args)
        .write_stdin(PHRASE)
        .assert()
        .success();
    assert_eq!(
        plain.get_output().stdout,
        with_flag.get_output().stdout,
        "stdout must not move"
    );
    assert_eq!(
        plain.get_output().stderr,
        with_flag.get_output().stderr,
        "stderr must not move either -- the override is not an advisory switch"
    );
}

/// The override is declared and therefore greppable and discoverable, which is
/// the property §6d asks for by name ("greppable in a script, so a reviewer can
/// find it").
#[test]
fn the_override_is_declared_on_the_binary() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
    let help = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        help.contains("--allow-argv-secret"),
        "the override must appear in --help; got:\n{help}"
    );
}
