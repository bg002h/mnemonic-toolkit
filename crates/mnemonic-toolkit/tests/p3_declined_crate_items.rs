//! P3 row 20 — **THE DECLINE, ASSERTED** (`mnemonic`'s third of it).
//!
//! The plan's boundary table takes **two** of `mnemonic-io-lib`'s eleven items
//! into `mnemonic` — `remedy::history_purge_block` and its structured half
//! `remedy::history_purge_recipes` — and declines the other nine. A decline is
//! invisible in a diff, so a later phase can adopt one while "tidying up" and
//! nothing goes red. These tests are the backstop that makes the nine declines
//! observable.
//!
//! **The reasons are the spec's, not this file's.** §6e **retracted** the
//! generalisation of `me`'s terminal gate — *"the terminal gate stays scoped to
//! `me`'s binary container"*, justified by binary-in-a-scrollback and by
//! nothing else. `ms1`/`mk1`/`md1` strings are short printable ASCII a human
//! must **read** in order to engrave them, so `exit::write_block`'s
//! unconditional `Destination::Terminal` refusal is a behaviour `mnemonic` must
//! not acquire. `observation::PayloadKind` goes for a different reason: its
//! variants are `Bearer` and `CarriesNoSecret`, and a `bundle`'s stdout is
//! neither shaped.

use assert_cmd::Command;

/// **THE STRUCTURAL HALF.** The binary reaches into the shared crate through
/// exactly one module. A `cargo nextest` run cannot see an adoption that is
/// merely *compiled*, so this is asserted against the source itself.
#[test]
fn the_binary_names_only_the_two_adopted_crate_items() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    let mut adopters = 0usize;
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src/ is readable") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("source is utf-8");
            for (i, line) in text.lines().enumerate() {
                let Some(pos) = line.find("mnemonic_io_lib::") else {
                    continue;
                };
                let rest = &line[pos + "mnemonic_io_lib::".len()..];
                if rest.starts_with("remedy::") {
                    adopters += 1;
                } else {
                    let rel = path.strip_prefix(&root).unwrap_or(&path);
                    offenders.push(format!("{}:{}: {}", rel.display(), i + 1, line.trim()));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "P3's boundary table adopts `remedy` and NOTHING else from \
         mnemonic-io-lib. These lines reach a declined item; if that is \
         intended it is a boundary change and belongs in a plan, not in a \
         tidy-up:\n{}",
        offenders.join("\n")
    );
    assert!(
        adopters >= 1,
        "the adoption vanished -- `remedy` is what the argv refusal prints, so \
         zero references means the purge recipe is no longer the crate's"
    );
}

/// **THE BEHAVIOURAL HALF.** `mnemonic` writes its cards to a **terminal**
/// without refusing. An adoption of `exit::write_block` that imported `me`'s
/// terminal gate goes RED here.
///
/// `MNEMONIC_FORCE_TTY=1` is the binary's own public-API contract for forcing
/// the TTY-conditional paths, so this exercises the terminal branch without a
/// pty.
#[test]
fn mnemonic_writes_to_a_terminal_without_refusing() {
    const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args([
            "bundle",
            "--slot",
            "@0.phrase=-",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .write_stdin(PHRASE)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("ms1") && stdout.contains("mk1") && stdout.contains("md1"),
        "the cards must reach a terminal; §6e retracted the terminal gate's \
         generalisation. stdout:\n{stdout}"
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("terminal"),
        "a terminal refusal appeared; §6e scopes that gate to `me`'s binary \
         container. stderr:\n{stderr}"
    );
}

/// §6f closure condition 17: **no exit code moves except `mk`'s invalid
/// artifact**, which is another repo's row. `mnemonic`'s
/// 1-or-2-by-input-shape split is unchanged by P3, and the two cells are the
/// ones §6f measured under verbs that EXIST (`mnemonic` has no `decode`).
#[test]
fn mnemonic_invalid_artifact_exit_codes_are_unchanged() {
    for (arg, expected, why) in [
        ("notanartifact", 2, "unknown HRP"),
        ("md1nonsense", 1, "md1 HRP, decode failure"),
    ] {
        let code = Command::cargo_bin("mnemonic")
            .unwrap()
            .args(["inspect", arg])
            .write_stdin("")
            .output()
            .unwrap()
            .status
            .code()
            .unwrap_or(-1);
        assert_eq!(
            code, expected,
            "`mnemonic inspect {arg}` ({why}) must still exit {expected}; P3 \
             renumbers no mnemonic exit code"
        );
    }
}
