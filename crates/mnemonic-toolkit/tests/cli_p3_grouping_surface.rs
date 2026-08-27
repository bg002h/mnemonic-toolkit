//! P3 row 13 — THE `mnemonic` GROUPING SURFACE (SPEC_constellation_cli_uniformity
//! §6c, §2a).
//!
//! Two changes, one row:
//!
//! 1. **`--group-size` defaults 5 → 0** at every declaring site, so stdout
//!    carries the unbroken artifact and a grouped form is something the operator
//!    asks for rather than something a pipeline has to strip.
//! 2. **`--separator` narrows to whitespace.** `hyphen` and `comma` are retired:
//!    a card is what a human types back into *another* tool, and `mt` refuses a
//!    hyphen-grouped string — a rule that is safe per-tool and unsafe across
//!    tools is exactly the kind an operator carries between tools.
//!
//! **THE CARRIER SET IS GENERATED, NOT HAND-LISTED.** §2a of the spec names
//! `bundle` alone; the binary carries the pair on **four** subcommands. So this
//! file enumerates the carriers from `mnemonic gui-schema` — the binary's own
//! description of its flag surface — and asserts the behavioural table covers
//! exactly that set. A fifth carrier added later reds
//! [`grouping_carrier_set_is_exactly_what_the_schema_declares`] instead of
//! silently escaping the flip.
//!
//! **WHY THE REFUSAL ASSERTIONS NAME THE MESSAGE AND NOT JUST THE EXIT CODE.**
//! `--separator` is a clap `value_parser`, so a bad value is a clap usage error
//! at exit 64 — and *every* incomplete invocation is also 64. A gate written as
//! "exits non-zero" would therefore pass in **both** worlds: before this row
//! because a required argument was missing, after it because the separator was
//! rejected. The invocations below are complete and exit **0 today**, and the
//! assertion is on clap's `invalid value 'hyphen' for '--separator'`.

use assert_cmd::Command;

/// 12-word all-`abandon` BIP-39 vector. Fed on **stdin** wherever the tool has a
/// stdin channel, never on argv: P3's next row refuses argv-borne secret
/// material, and a test written against the argv channel would have to be
/// rewritten one commit later.
const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// One complete, exit-0-today invocation per `--group-size` carrier.
struct Carrier {
    name: &'static str,
    args: Vec<String>,
    stdin: String,
}

fn args(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// The artifact lines of a stdout capture, with any `ms1: ` / `mk1: ` / `md1: `
/// label prefix stripped. `#` comments and blank separators are not artifacts.
fn artifact_bodies(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter_map(|l| {
            let body = match l.split_once(": ") {
                Some(("ms1" | "mk1" | "md1", rest)) => rest,
                _ => l,
            };
            let hrp: String = body.chars().take(3).collect();
            matches!(hrp.as_str(), "ms1" | "mk1" | "md1").then_some(body)
        })
        .collect()
}

fn carriers() -> Vec<Carrier> {
    // A real 2-of-3 codex32 split, produced through the stdin channel, so the
    // `combine` carrier below has genuine shares to recombine.
    let split = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "ms-shares",
            "split",
            "--from",
            "phrase=-",
            "--threshold",
            "2",
            "--shares",
            "3",
            "--group-size",
            "0",
        ])
        .write_stdin(PHRASE)
        .assert()
        .success();
    let shares: Vec<String> = String::from_utf8(split.get_output().stdout.clone())
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(shares.len(), 3, "fixture: expected three ms1 shares");

    vec![
        Carrier {
            name: "bundle",
            args: args(&[
                "bundle",
                "--slot",
                "@0.phrase=-",
                "--network",
                "mainnet",
                "--template",
                "bip84",
            ]),
            stdin: PHRASE.to_string(),
        },
        Carrier {
            name: "convert",
            args: args(&["convert", "--from", "phrase=-", "--to", "ms1"]),
            stdin: PHRASE.to_string(),
        },
        Carrier {
            name: "ms-shares-split",
            args: args(&[
                "ms-shares",
                "split",
                "--from",
                "phrase=-",
                "--threshold",
                "2",
                "--shares",
                "3",
            ]),
            stdin: PHRASE.to_string(),
        },
        Carrier {
            // `ms-shares combine` needs K >= 2 shares and accepts **at most
            // one** `-`, so its second share cannot avoid argv. That is a real
            // gap in the tool's private channels, filed rather than papered
            // over — P3's argv row reaches this invocation through
            // `--allow-argv-secret` rather than pretending a channel exists,
            // and adds that flag to this line when it lands.
            name: "ms-shares-combine",
            args: {
                let mut a = args(&["ms-shares", "combine", "--share"]);
                a.push(shares[0].clone());
                a.extend(args(&["--share", "-", "--to", "ms1"]));
                a
            },
            stdin: shares[1].clone(),
        },
    ]
}

/// The subcommands the BINARY says carry `--group-size`, read out of its own
/// `gui-schema` JSON rather than out of this file's imagination.
fn schema_group_size_carriers() -> Vec<(String, Option<String>, Option<String>)> {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("gui-schema")
        .assert()
        .success();
    let json: serde_json::Value =
        serde_json::from_slice(&out.get_output().stdout).expect("gui-schema emits JSON");
    let mut carriers = Vec::new();
    for sub in json["subcommands"].as_array().expect("subcommands array") {
        let name = sub["name"].as_str().unwrap_or_default().to_string();
        let mut gs = None;
        let mut sep = None;
        for flag in sub["flags"].as_array().into_iter().flatten() {
            // `--group-size` is a `number` kind, so the schema emits a JSON
            // NUMBER; `--separator` is `text`, so it emits a JSON string.
            // Stringify either shape rather than assuming one — reading
            // `.as_str()` alone would report `<null>` for the numeric flag and
            // make this gate pass against an absent default.
            let default = match &flag["default_value"] {
                serde_json::Value::Null => None,
                serde_json::Value::String(s) => Some(s.clone()),
                other => Some(other.to_string()),
            };
            match flag["name"].as_str() {
                Some("--group-size") => gs = Some(default.unwrap_or_else(|| "<null>".into())),
                Some("--separator") => sep = Some(default.unwrap_or_else(|| "<null>".into())),
                _ => {}
            }
        }
        if gs.is_some() {
            carriers.push((name, gs, sep));
        }
    }
    carriers.sort();
    carriers
}

#[test]
fn grouping_carrier_set_is_exactly_what_the_schema_declares() {
    let schema: Vec<String> = schema_group_size_carriers()
        .into_iter()
        .map(|(n, _, _)| n)
        .collect();
    let mut covered: Vec<String> = carriers().into_iter().map(|c| c.name.to_string()).collect();
    covered.sort();
    assert_eq!(
        schema, covered,
        "the behavioural table in this file must cover EVERY --group-size \
         carrier the binary declares; a new carrier is a new default to flip, \
         not a new exemption"
    );
}

/// The declared default, read from the binary. This is the half the
/// `mnemonic-gui` drift gate compares against, so it is asserted here at the
/// source rather than only downstream.
#[test]
fn declared_group_size_default_is_zero_everywhere() {
    let carriers = schema_group_size_carriers();
    assert!(
        !carriers.is_empty(),
        "gui-schema declared no --group-size flag"
    );
    for (name, gs, sep) in carriers {
        assert_eq!(
            gs.as_deref(),
            Some("0"),
            "{name}: --group-size must default to 0 (unbroken) after §6c"
        );
        assert_eq!(
            sep.as_deref(),
            Some("space"),
            "{name}: --separator's default is unchanged; only its VOCABULARY narrows"
        );
    }
}

/// The behavioural half: stdout carries no separator by default.
#[test]
fn default_output_is_unbroken_on_every_carrier() {
    for c in carriers() {
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args(&c.args)
            .write_stdin(c.stdin)
            .assert()
            .success();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        let bodies = artifact_bodies(&stdout);
        assert!(
            !bodies.is_empty(),
            "{}: no artifact line found on stdout:\n{stdout}",
            c.name
        );
        for body in bodies {
            assert!(
                !body.contains(' ') && !body.contains('-') && !body.contains(','),
                "{}: default output must be UNBROKEN; got {body:?}",
                c.name
            );
        }
    }
}

/// `--group-size 5` is still available — the flip changes the DEFAULT, not the
/// capability. Without this control, deleting the flag outright would pass the
/// test above.
#[test]
fn explicit_group_size_five_still_groups() {
    for c in carriers() {
        let mut a = c.args.clone();
        a.extend(args(&["--group-size", "5"]));
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args(&a)
            .write_stdin(c.stdin)
            .assert()
            .success();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        let bodies = artifact_bodies(&stdout);
        assert!(!bodies.is_empty(), "{}: no artifact line", c.name);
        assert!(
            bodies.iter().all(|b| b.contains(' ')),
            "{}: --group-size 5 must still group every artifact line; got:\n{stdout}",
            c.name
        );
    }
}

/// §6c: `hyphen` and `comma` are retired on every carrier, keyword and literal
/// alike.
///
/// The invocations are otherwise complete and exit 0 today, so the exit code
/// moves *because of the separator* — and the message is asserted so a
/// coincidentally-non-zero exit cannot pass for a refusal.
#[test]
fn retired_separator_values_are_refused_on_every_carrier() {
    for c in carriers() {
        for retired in ["hyphen", "comma", "-", ","] {
            let mut a = c.args.clone();
            a.extend(args(&["--separator", retired]));
            let out = Command::cargo_bin("mnemonic")
                .unwrap()
                .args(&a)
                .write_stdin(c.stdin.clone())
                .assert()
                .failure();
            let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
            assert!(
                stderr.contains(&format!("invalid value '{retired}' for '--separator")),
                "{}: --separator {retired} must be REFUSED by the value parser; \
                 stderr was:\n{stderr}",
                c.name
            );
            assert!(
                stderr.contains("whitespace"),
                "{}: the refusal must name what replaced the retired keywords; \
                 stderr was:\n{stderr}",
                c.name
            );
        }
    }
}

/// The control: `space` and the literal `" "` still parse and still group.
#[test]
fn whitespace_separator_still_accepted_on_every_carrier() {
    for c in carriers() {
        for accepted in ["space", " "] {
            let mut a = c.args.clone();
            a.extend(args(&["--separator", accepted, "--group-size", "5"]));
            let out = Command::cargo_bin("mnemonic")
                .unwrap()
                .args(&a)
                .write_stdin(c.stdin.clone())
                .assert()
                .success();
            let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
            assert!(
                artifact_bodies(&stdout).iter().all(|b| b.contains(' ')),
                "{}: --separator {accepted:?} must still space-group; got:\n{stdout}",
                c.name
            );
        }
    }
}

/// **THE CONTROL THAT PINS THE DECLINE.** §6a's stdout rule is scoped to
/// `encode` by an explicit table `mnemonic` is not in, so `bundle`'s three `#`
/// kind comments and three blank separators STAY. An implementer who read §4 as
/// absolute — "no non-artifact lines on stdout, ever" — goes RED here rather
/// than silently breaking a shipped machine-readable surface. Filed as F-295.
#[test]
fn bundle_stdout_keeps_its_comment_headers_and_blank_separators() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
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
    let lines: Vec<&str> = stdout.lines().collect();
    let comments = lines.iter().filter(|l| l.starts_with('#')).count();
    let blanks = lines.iter().filter(|l| l.trim().is_empty()).count();
    let artifacts = artifact_bodies(&stdout).len();
    assert_eq!(
        (lines.len(), comments, blanks, artifacts),
        (12, 3, 3, 6),
        "bundle's stdout shape is (lines, #-comments, blanks, artifacts); \
         got:\n{stdout}"
    );
}

/// **THE CORPUS, ASSERTED UNCHANGED.** `design/display-grouping-vectors.tsv` is
/// byte-identical across `descriptor-mnemonic`, `mnemonic-key`,
/// `mnemonic-secret` and `mnemonic-toolkit`, and it contains rows keyed `hyphen`
/// and `comma`. It survives §6c because its consumers are **codec-level**: the
/// conformance test maps the keyword to a `char` itself, and the function under
/// test takes a `char` and has no keyword vocabulary at all. A narrowing applied
/// one layer too deep — in `render_grouped` rather than in `parse_separator` —
/// goes RED here and in `display_grouping_conformance`.
#[test]
fn display_grouping_corpus_is_untouched() {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../design/display-grouping-vectors.tsv"
    ))
    .expect("the shared display-grouping corpus is tracked in this repo");
    let digest = Sha256::digest(&bytes);
    assert_eq!(
        format!("{digest:x}"),
        "7147b0ecc8cf175c41b2ade612d8dc4c6e523974f39188485ee68b2f99cc10ad",
        "the four-repo display-grouping corpus must not move; §6c narrows the \
         CLI's --separator vocabulary, not the codec's char-taking renderer"
    );
    let text = String::from_utf8(bytes).unwrap();
    assert!(
        text.contains("hyphen") && text.contains("comma"),
        "the corpus still carries hyphen and comma rows -- that is the point of \
         this control"
    );
}
