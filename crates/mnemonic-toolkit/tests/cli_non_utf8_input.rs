//! Final review I3: input bytes that are not UTF-8 are an invalid input
//! (exit 2, "the input is not UTF-8 text"), at every read site; any other
//! read failure stays an I/O failure (32). One row per verb and read site.

use assert_cmd::Command;

const BAD: &[u8] = b"\xff\xfe abc\n";
const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

/// Where the bytes go: piped on stdin, or in the file named by `{FILE}`.
#[derive(Clone, Copy)]
enum Via {
    Stdin,
    File,
}

fn rows() -> Vec<(&'static str, Via, Vec<&'static str>)> {
    use Via::*;
    vec![
        (
            "convert --from phrase=-",
            Stdin,
            vec![
                "convert",
                "--from",
                "phrase=-",
                "--to",
                "xpub",
                "--template",
                "bip84",
            ],
        ),
        (
            "convert --passphrase -",
            Stdin,
            vec![
                "convert",
                "--from",
                "phrase={PHRASE}",
                "--to",
                "xpub",
                "--template",
                "bip84",
                "--passphrase",
                "-",
            ],
        ),
        ("compare-cost stdin", Stdin, vec!["compare-cost"]),
        (
            "build-descriptor --spec -",
            Stdin,
            vec!["build-descriptor", "--spec", "-"],
        ),
        (
            "build-descriptor --spec FILE",
            File,
            vec!["build-descriptor", "--spec", "{FILE}"],
        ),
        (
            "bundle --descriptor-file",
            File,
            vec![
                "bundle",
                "--network",
                "mainnet",
                "--descriptor-file",
                "{FILE}",
            ],
        ),
        (
            "bundle --import-json",
            File,
            vec!["bundle", "--network", "mainnet", "--import-json", "{FILE}"],
        ),
        (
            "bundle --slot @0.phrase=-",
            Stdin,
            vec![
                "bundle",
                "--network",
                "mainnet",
                "--template",
                "bip84",
                "--slot",
                "@0.phrase=-",
            ],
        ),
        (
            "verify-bundle --descriptor-file",
            File,
            vec![
                "verify-bundle",
                "--network",
                "mainnet",
                "--descriptor-file",
                "{FILE}",
                "md1qq",
            ],
        ),
        (
            "verify-bundle --bundle-json",
            File,
            vec![
                "verify-bundle",
                "--network",
                "mainnet",
                "--bundle-json",
                "{FILE}",
            ],
        ),
        (
            "export-wallet --from-import-json",
            File,
            vec![
                "export-wallet",
                "--from-import-json",
                "{FILE}",
                "--format",
                "electrum",
                "--output",
                "-",
            ],
        ),
        (
            "import-wallet --bsms-encryption-token",
            File,
            vec![
                "import-wallet",
                "--format",
                "bsms",
                "--blob",
                "-",
                "--bsms-encryption-token",
                "{FILE}",
            ],
        ),
        (
            "electrum-decrypt --decrypt-password-file",
            File,
            vec![
                "electrum-decrypt",
                "--ciphertext",
                "AAAA",
                "--decrypt-password-file",
                "{FILE}",
            ],
        ),
        (
            "xpub-search path-of-xpub --phrase-stdin",
            Stdin,
            vec![
                "xpub-search",
                "path-of-xpub",
                "--phrase-stdin",
                "--target-xpub",
                "{XPUB}",
            ],
        ),
        (
            "xpub-search path-of-xpub --ms1-stdin",
            Stdin,
            vec![
                "xpub-search",
                "path-of-xpub",
                "--ms1-stdin",
                "--target-xpub",
                "{XPUB}",
            ],
        ),
        (
            "xpub-search account-of-descriptor --descriptor-from literal=-",
            Stdin,
            vec![
                "xpub-search",
                "account-of-descriptor",
                "--phrase",
                "{PHRASE}",
                "--descriptor-from",
                "literal=-",
            ],
        ),
        (
            "xpub-search address-of-xpub --xpub-stdin",
            Stdin,
            vec![
                "xpub-search",
                "address-of-xpub",
                "--xpub-stdin",
                "--target-address",
                "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
                "--address-type",
                "p2wpkh",
            ],
        ),
        (
            "xpub-search passphrase-of-xpub --passphrase-candidates-file",
            File,
            vec![
                "xpub-search",
                "passphrase-of-xpub",
                "--phrase",
                "{PHRASE}",
                "--passphrase-candidates-file",
                "{FILE}",
                "--target-xpub",
                "{XPUB}",
            ],
        ),
    ]
}

#[test]
fn final_i3_non_utf8_input_is_2_at_every_read_site() {
    let dir = tempfile::tempdir().expect("tempdir");
    let file = dir.path().join("bad.bin");
    std::fs::write(&file, BAD).expect("write");
    let file = file.to_str().expect("utf-8 path").to_owned();
    let mut failures = Vec::new();
    for (name, via, args) in rows() {
        let args: Vec<String> = args
            .iter()
            .map(|a| {
                a.replace("{FILE}", &file)
                    .replace("{PHRASE}", PHRASE)
                    .replace("{XPUB}", XPUB)
            })
            .collect();
        let mut c = Command::cargo_bin("mnemonic").expect("binary built");
        c.arg("--allow-argv-secret").args(&args);
        match via {
            Via::Stdin => c.write_stdin(BAD),
            Via::File => c.write_stdin(&b""[..]),
        };
        let out = c.output().expect("spawn");
        let stderr = String::from_utf8_lossy(&out.stderr);
        if out.status.code() != Some(2) || !stderr.contains("the input is not UTF-8 text") {
            failures.push(format!(
                "{name}: exit {:?}, stderr {stderr:?}",
                out.status.code()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "rows that failed:\n{}",
        failures.join("\n")
    );
}

/// The other half of the mapping: a file that cannot be read for a reason
/// other than its bytes is still an I/O failure (32).
#[test]
fn final_i3_a_missing_file_stays_32() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("missing.txt");
    let out = Command::cargo_bin("mnemonic")
        .expect("binary built")
        .args(["bundle", "--network", "mainnet", "--descriptor-file"])
        .arg(&missing)
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(32),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
