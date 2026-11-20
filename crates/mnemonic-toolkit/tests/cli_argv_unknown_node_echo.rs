//! A value with `=` whose node or slot subkey the flag does not accept
//! (`--from phrse=<secret>`) used to skip the pre-parser guard and be echoed by
//! clap's exit-64 error. The guard refuses it at exit 2 with the flag's channel.

use assert_cmd::Command;

const SECRET: &str = "SECRETWORD";
const EXIT_REFUSED: i32 = 2;

fn run(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .write_stdin("")
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

fn assert_refused_without_echo(args: &[&str], channel: &str) {
    let (stdout, stderr, code) = run(args);
    assert_eq!(code, EXIT_REFUSED, "stderr:\n{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(
        stderr.contains("Refused BEFORE the command line was parsed"),
        "{stderr}"
    );
    assert!(stderr.contains(channel), "{stderr}");
    assert!(!stderr.contains(SECRET), "echoed the value:\n{stderr}");
}

#[test]
fn unknown_from_node_is_refused_without_echo() {
    for v in [
        "phrse=SECRETWORD",
        "foo=SECRETWORD=bar",
        "=SECRETWORD",
        " phrase=SECRETWORD",
        "PHRASE=SECRETWORD",
        "@env:A=SECRETWORD",
        "-=SECRETWORD",
        "@0.phrase=SECRETWORD",
    ] {
        assert_refused_without_echo(
            &[
                "convert",
                "--from",
                v,
                "--to",
                "xpub",
                "--template",
                "bip84",
            ],
            "--from phrase=-",
        );
        assert_refused_without_echo(&["slip39", "split", "--from", v], "--from phrase=-");
    }
}

#[test]
fn unknown_slot_subkey_is_refused_without_echo() {
    for v in [
        "@0.phrse=SECRETWORD",
        "foo=SECRETWORD",
        "phrase=SECRETWORD=SECRETWORD",
        "xpub=SECRETWORD",
        "@x.phrase=SECRETWORD",
        "@.phrase=SECRETWORD",
        "@0 .phrase=SECRETWORD",
        "@env:A=SECRETWORD",
        "=SECRETWORD",
    ] {
        assert_refused_without_echo(&["bundle", "--slot", v], "--slot @N.phrase=-");
        assert_refused_without_echo(&["verify-bundle", "--slot", v], "--slot @N.phrase=-");
    }
}

#[test]
fn unknown_seed_xor_share_node_is_refused_without_echo() {
    for v in ["phrse=SECRETWORD", "foo=SECRETWORD", "=SECRETWORD"] {
        assert_refused_without_echo(&["seed-xor", "combine", "--share", v], "--share phrase=-");
    }
}

#[test]
fn public_nodes_and_private_channels_still_pass_the_guard() {
    for args in [
        &["convert", "--from", "xpub=SECRETWORD", "--to", "xpub"][..],
        &["bundle", "--slot", "@0.xpub=SECRETWORD"][..],
        &["bundle", "--slot", "@0.phrase=-"][..],
    ] {
        let (_o, e, _c) = run(args);
        assert!(!e.contains("Refused BEFORE"), "{args:?}: {e}");
    }
}

#[test]
fn seedqr_decode_shapeless_from_advises_the_seedqr_node() {
    for v in ["SECRETWORD", "phrse=SECRETWORD"] {
        assert_refused_without_echo(&["seedqr", "decode", "--from", v], "--from seedqr=-");
    }
}
