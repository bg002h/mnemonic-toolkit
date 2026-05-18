//! `mnemonic derive-child` subcommand — BIP-85 deterministic derivation.
//!
//! Realizes `design/SPEC_derive_child_v0_7.md` §2 (grammar), §3 (primitive),
//! §4 (in-scope apps), §5 (out-of-scope refusal), §7 (refusal taxonomy)
//! plus v0.8 extensions: §3 phrase-master input, language-code dispatch,
//! testnet emission, stdin-master sentinel.

use crate::bip85;
use crate::cmd::convert::{parse_from_input, read_stdin_to_string, FromInput, NodeType};
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use bip39::Mnemonic;
use bitcoin::bip32::Xpriv;
use bitcoin::NetworkKind;
use clap::Args;
use std::io::{Read, Write};
use std::str::FromStr;

#[derive(Args, Debug, Clone)]
pub struct DeriveChildArgs {
    /// Master source — shape `<node>=<value>`.
    ///
    /// `<node>` is one of:
    ///   xprv    BIP-32 extended private key (secret)
    ///   phrase  BIP-39 mnemonic (secret; combine with --passphrase
    ///           and --language)
    ///
    /// `<value>` is the node's text form, or `-` to read from stdin.
    #[arg(
        long = "from",
        value_parser = parse_from_input,
        required = true,
        verbatim_doc_comment,
    )]
    pub from: FromInput,

    /// BIP-85 application. Accepted values:
    ///
    ///   bip39             BIP-85 application 39' — derives a child
    ///                     BIP-39 mnemonic of `--length` words (12 /
    ///                     18 / 24)
    ///   hd-seed           BIP-85 application 32' — derives a 32-byte
    ///                     BIP-32 seed (`--length 0`)
    ///   xprv              BIP-85 application 32' alt — derives a
    ///                     child xprv directly (`--length 0`)
    ///   hex               BIP-85 application 128169' — derives
    ///                     `--length` bytes of hex (16..=64)
    ///   password-base64   BIP-85 application 707764' — derives
    ///                     `--length` chars of base64 password (20..=86)
    ///   password-base85   BIP-85 application 707785' — derives
    ///                     `--length` chars of base85 password (10..=80)
    ///   dice              BIP-85 application 89101' — derives
    ///                     `--length` dice rolls (1..=99 each
    ///                     `--dice-sides`-sided); REFUSED at runtime
    ///   rsa               BIP-85 application 828365' — RSA key
    ///                     derivation; REFUSED at runtime
    ///                     (RUSTSEC-2023-0071)
    ///   rsa-gpg           BIP-85 application 828366' — RSA-GPG;
    ///                     REFUSED at runtime
    #[arg(long = "application", required = true, verbatim_doc_comment)]
    pub application: String,

    /// Per-app `--length` validator (range varies; see SPEC §4).
    /// Required at clap level for grammar-uniformity (SPEC §2). For
    /// `hd-seed` / `xprv` the value is irrelevant unless non-zero, in which
    /// case the SPEC §7 not-applicable refusal fires; pass `--length 0` as
    /// the sentinel to satisfy clap without triggering the refusal.
    #[arg(long = "length", required = true)]
    pub length: u32,

    /// Hardened child index (`0..2^31`).
    #[arg(long = "index", required = true)]
    pub index: u32,

    /// SPEC v0.8 §4 — network for emitted `--application <hd-seed|xprv>`.
    /// Defaults to mainnet (matches BIP-85 §"Test Vectors"). Testnet emits
    /// `c…` WIF and `tprv…` xprv prefixes.
    #[arg(long)]
    pub network: Option<CliNetwork>,

    /// SPEC v0.8 §4 — BIP-39 wordlist + BIP-85 language code for
    /// `--application bip39`. Defaults to English (BIP-85 code 0).
    #[arg(long)]
    pub language: Option<CliLanguage>,

    /// SPEC v0.8 §3 — BIP-39 mnemonic extension passphrase, used only
    /// when `--from phrase=…`. Empty by default. Ignored on `--from xprv=…`.
    #[arg(long)]
    pub passphrase: Option<String>,

    /// SPEC v0.9.0 §1 item 1 — read `--passphrase` from stdin (raw,
    /// preserving NULL bytes; strips a single trailing `\r?\n`).
    /// Mutually exclusive with `--passphrase` AND with any
    /// `--from <node>=-` (single stdin per invocation).
    /// Mirrors `convert.rs:181` precedent.
    #[arg(long = "passphrase-stdin", conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// SPEC v0.8 §4 — number of sides for `--application dice`. Required
    /// when `--application dice`; ignored otherwise. Range: 2..=2^32-1.
    #[arg(long = "dice-sides")]
    pub dice_sides: Option<u32>,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &DeriveChildArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // v0.26.0 §3 — resolve `@env:<VAR>` sentinels on `--passphrase` and
    // secret-bearing `--from <node>=` values before argv-leakage advisory
    // and downstream consumption.
    let env_resolved_owned;
    let args: &DeriveChildArgs = if needs_env_sentinel_resolution(args) {
        env_resolved_owned = resolve_env_sentinels(args)?;
        &env_resolved_owned
    } else {
        args
    };

    // SPEC v0.9.0 §1 item 1 — argv-leakage closure (advisories first).
    emit_secret_in_argv_advisories(args, stderr);

    // SPEC v0.9.0 §1 item 1 — single-stdin-per-invocation. `--from <node>=-`
    // and `--passphrase-stdin` both want stdin; refuse the combination.
    if args.passphrase_stdin && args.from.value == "-" {
        return Err(ToolkitError::BadInput(
            "--passphrase-stdin cannot be used with --from <node>=- (single stdin per invocation)"
                .into(),
        ));
    }

    // SPEC §2 + v0.8 §3 — `--from` accepts xprv= or phrase=. Stdin via `=-`.
    // SPEC v0.9.0 §1 item 2 — wrap the OWNED secret string in Zeroizing so
    // it scrubs on drop after seed derivation.
    let from_value: zeroize::Zeroizing<String> = if args.from.value == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(args.from.value.clone())
    };

    // SPEC v0.9.0 §1 item 1 — read --passphrase from stdin when set.
    // Preserves NULL bytes; strips a single trailing `\r?\n`.
    // SPEC v0.9.0 §1 item 2 — wrap the OWNED stdin buffer in Zeroizing so
    // the BIP-39 passphrase scrubs on drop at function exit.
    let stdin_passphrase: Option<zeroize::Zeroizing<String>> = if args.passphrase_stdin {
        let mut buf: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(String::new());
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        Some(buf)
    } else {
        None
    };

    // Cycle B Phase 3a Site 1 — pin secret-bearing heap pages for the
    // remainder of the handler scope. from_value and stdin_passphrase are
    // both Zeroizing<String>; we pin the underlying String's heap data
    // (via .as_bytes()), and rely on Zeroizing for scrub-on-drop.
    let _pin_from = mnemonic_toolkit::mlock::pin_pages_for(from_value.as_bytes());
    let _pin_pp = stdin_passphrase
        .as_ref()
        .map(|s| mnemonic_toolkit::mlock::pin_pages_for(s.as_bytes()));

    let master = match args.from.node {
        NodeType::Xprv => Xpriv::from_str(&from_value)
            .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?,
        NodeType::Phrase => {
            // SAFETY: third-party-blocked — `bip39::Mnemonic` +
            // `bitcoin::bip32::Xpriv` have no Drop+Zeroize. FOLLOWUPS:
            // `rust-bip39-mnemonic-zeroize-upstream`,
            // `rust-bitcoin-xpriv-zeroize-upstream`.
            let language = args.language.unwrap_or_default();
            let mnemonic = Mnemonic::parse_in(language.into(), from_value.as_str())
                .map_err(ToolkitError::Bip39)?;
            let passphrase: &str = stdin_passphrase
                .as_ref()
                .map(|z| z.as_str())
                .or(args.passphrase.as_deref())
                .unwrap_or("");
            let seed = crate::derive_slot::derive_master_seed(&mnemonic, passphrase);
            // BIP-85 spec test vectors are network-agnostic at the entropy
            // level; the master xprv's network field doesn't affect any
            // BIP-85 derivation byte. Use Main as a stable internal default;
            // emission-side network is driven by `--network` per SPEC §4.
            // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy
            // + no Drop; FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
            Xpriv::new_master(NetworkKind::Main, &seed[..])
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?
        }
        _ => {
            return Err(ToolkitError::BadInput(format!(
                "derive-child: --from must be xprv=<master-xprv> or phrase=<mnemonic>; got {}",
                args.from.node.as_str(),
            )))
        }
    };
    if (args.passphrase.is_some() || args.passphrase_stdin) && args.from.node != NodeType::Phrase {
        let _ = writeln!(
            stderr,
            "warning: --passphrase ignored on --from xprv (no BIP-39 mnemonic to extend)",
        );
    }

    let emit_network = args.network.unwrap_or(CliNetwork::Mainnet).network_kind();

    // SPEC §5 + §7 — out-of-scope apps surface byte-exact refusal here.
    // v0.8: `dice` lifted to in-scope; `rsa` and `rsa-gpg` remain deferred
    // per Phase 6 RSA-crate security spike (RUSTSEC-2023-0071 unpatched).
    match args.application.as_str() {
        "rsa" | "rsa-gpg" => return Err(ToolkitError::DeriveChildUnsupportedApp),
        _ => {}
    }

    let output = match args.application.as_str() {
        "bip39" => {
            let words = args.length;
            if !matches!(words, 12 | 15 | 18 | 21 | 24) {
                return Err(ToolkitError::DeriveChildLengthOutOfRange {
                    app: "bip39",
                    length: words,
                    valid_text: "12 | 15 | 18 | 21 | 24 words",
                });
            }
            let cli_lang = args.language.unwrap_or_default();
            let (lang_code, bip39_lang) = resolve_bip85_language(cli_lang)?;
            bip85::format_bip39_phrase(&master, lang_code, bip39_lang, words, args.index)?
        }
        "hd-seed" => {
            reject_length(args.length)?;
            bip85::format_hd_seed_wif(&master, args.index, emit_network)?
        }
        "xprv" => {
            reject_length(args.length)?;
            bip85::format_xprv_child(&master, args.index, emit_network)?
        }
        "hex" => {
            let n = args.length;
            if !(16..=64).contains(&n) {
                return Err(ToolkitError::DeriveChildLengthOutOfRange {
                    app: "hex",
                    length: n,
                    valid_text: "16..=64 bytes",
                });
            }
            bip85::format_hex_bytes(&master, n, args.index)?
        }
        "password-base64" => {
            let n = args.length;
            if !(20..=86).contains(&n) {
                return Err(ToolkitError::DeriveChildLengthOutOfRange {
                    app: "password-base64",
                    length: n,
                    valid_text: "20..=86 chars",
                });
            }
            bip85::format_password_base64(&master, n, args.index)?
        }
        "password-base85" => {
            let n = args.length;
            if !(10..=80).contains(&n) {
                return Err(ToolkitError::DeriveChildLengthOutOfRange {
                    app: "password-base85",
                    length: n,
                    valid_text: "10..=80 chars",
                });
            }
            bip85::format_password_base85(&master, n, args.index)?
        }
        "dice" => {
            let rolls = args.length;
            if rolls < 1 {
                return Err(ToolkitError::DeriveChildLengthOutOfRange {
                    app: "dice",
                    length: rolls,
                    valid_text: "rolls >= 1",
                });
            }
            let sides = args.dice_sides.ok_or_else(|| {
                ToolkitError::BadInput(
                    "--application dice requires --dice-sides <N> (number of sides; >=2)".into(),
                )
            })?;
            bip85::format_dice_rolls(&master, sides, rolls, args.index)?
        }
        other => {
            return Err(ToolkitError::BadInput(format!(
                "derive-child: --application {other:?} is not recognized; \
                 expected one of: bip39, hd-seed, xprv, hex, password-base64, \
                 password-base85, dice (or out-of-scope: rsa, rsa-gpg)",
            )));
        }
    };

    writeln!(stdout, "{output}").ok();
    // SPEC §4 — every in-scope app emits secret material; warn on stdout.
    let _ = writeln!(
        stderr,
        "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')",
    );
    Ok(())
}

/// SPEC §7 — `hd-seed` / `xprv` ignore `--length 0` (sentinel for grammar-
/// uniformity); any non-zero value triggers the not-applicable refusal.
fn reject_length(length: u32) -> Result<(), ToolkitError> {
    if length != 0 {
        return Err(ToolkitError::DeriveChildLengthNotApplicable);
    }
    Ok(())
}

/// SPEC v0.8 §4 — map `CliLanguage` → (BIP-85 path language code,
/// `bip39::Language` for wordlist selection). BIP-85 language codes per
/// <https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki> §"Application: BIP39":
///
/// | Code | Language               |
/// |------|------------------------|
/// | 0    | English                |
/// | 1    | Japanese               |
/// | 2    | Korean                 |
/// | 3    | Spanish                |
/// | 4    | Chinese (Simplified)   |
/// | 5    | Chinese (Traditional)  |
/// | 6    | French                 |
/// | 7    | Italian                |
/// | 8    | Czech                  |
///
/// Portuguese (BIP-39 wordlist, but no BIP-85 code assigned) is refused.
fn resolve_bip85_language(lang: CliLanguage) -> Result<(u32, bip39::Language), ToolkitError> {
    Ok(match lang {
        CliLanguage::English => (0, bip39::Language::English),
        CliLanguage::Japanese => (1, bip39::Language::Japanese),
        CliLanguage::Korean => (2, bip39::Language::Korean),
        CliLanguage::Spanish => (3, bip39::Language::Spanish),
        CliLanguage::SimplifiedChinese => (4, bip39::Language::SimplifiedChinese),
        CliLanguage::TraditionalChinese => (5, bip39::Language::TraditionalChinese),
        CliLanguage::French => (6, bip39::Language::French),
        CliLanguage::Italian => (7, bip39::Language::Italian),
        CliLanguage::Czech => (8, bip39::Language::Czech),
        CliLanguage::Portuguese => {
            return Err(ToolkitError::BadInput(
                "--language portuguese is not assigned a BIP-85 path code; only english, japanese, korean, spanish, simplified-chinese, traditional-chinese, french, italian, czech are supported for --application bip39".into(),
            ))
        }
    })
}

// ============================================================================
// SPEC v0.9.0 §1 item 1 — argv-leakage closure helpers
// ============================================================================

/// Per-occurrence `secret-in-argv` stderr advisory emission for
/// `derive-child`. The two inline-secret sites are `--from <node>=<inline>`
/// (xprv or phrase) and `--passphrase <inline>`; each fires its own
/// advisory if the inline form is used.
fn emit_secret_in_argv_advisories<E: Write>(args: &DeriveChildArgs, stderr: &mut E) {
    use crate::secret_advisory::secret_in_argv_warning;
    if args.from.value != "-" {
        let node = args.from.node.as_str();
        let flag = format!("--from {node}=");
        let alt = format!("--from {node}=-");
        secret_in_argv_warning(stderr, &flag, &alt);
    }
    if args.passphrase.is_some() {
        secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin");
    }
}

/// v0.26.0 §3 — cheap pre-check for `@env:` sentinels on `derive-child`'s
/// secret-bearing flag surfaces (`--passphrase`, secret-bearing
/// `--from <node>=`).
fn needs_env_sentinel_resolution(args: &DeriveChildArgs) -> bool {
    let pp = args
        .passphrase
        .as_deref()
        .map(|v| v.starts_with("@env:"))
        .unwrap_or(false);
    let from = args.from.node.is_secret_bearing() && args.from.value.starts_with("@env:");
    pp || from
}

/// v0.26.0 §3 — resolve `@env:<VAR>` sentinels across `derive-child`'s
/// secret-bearing flag surfaces.
fn resolve_env_sentinels(args: &DeriveChildArgs) -> Result<DeriveChildArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    if let Some(pp) = owned.passphrase.as_ref() {
        owned.passphrase = Some(resolve_env_var_sentinel(pp, "--passphrase")?);
    }
    if owned.from.node.is_secret_bearing() {
        let flag = format!("--from {}=", owned.from.node.as_str());
        owned.from.value = resolve_env_var_sentinel(&owned.from.value, &flag)?;
    }
    Ok(owned)
}

// ============================================================================
// Path B-lite Site 1 — derive-child handler-scope pin coverage.
//
// Tests assert `attempts_for_test() > baseline` after a production code path
// that should pin. record_attempt fires unconditionally on every
// pin_pages_for call (mlock.rs:97), independent of the FAIL_MODE harness
// and cfg(test) gating (per-crate-not-per-build, RFC 1604).
//
// derive_child::run is the canonical Site 1 exemplar; bundle / verify_bundle
// / convert handlers follow the same pattern (R1 reviewer verifies them by
// reading source — they're 1-3 pin lines each per SPEC §2 row 5).
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test BIP-32 master xprv (BIP-32 §"Test Vectors" m at depth 0).
    /// Stable; deterministically derives to a valid bip39 entropy at index 0.
    const TEST_XPRV: &str =
        "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";

    /// Site 1 — `derive_child::run` pins `from_value` and `stdin_passphrase`
    /// after they're bound (post `derive_child.rs:122`). The cascading
    /// Site 4 pin inside `format_bip39_phrase` also fires along this path.
    /// The test only asserts `> baseline`, so any pin call along the path
    /// passes — R1 reviewer separately verifies Site 1 pin specifically by
    /// reading source.
    #[test]
    fn site_1_derive_child_run_invokes_pin() {
        use std::io::Cursor;
        let args = DeriveChildArgs {
            from: FromInput {
                node: NodeType::Xprv,
                value: TEST_XPRV.to_string(),
            },
            application: "bip39".to_string(),
            length: 12,
            index: 0,
            network: None,
            language: None,
            passphrase: None,
            passphrase_stdin: false,
            dice_sides: None,
        };
        let mut stdin = Cursor::new(Vec::<u8>::new());
        let mut stdout = Vec::<u8>::new();
        let mut stderr = Vec::<u8>::new();
        let baseline = mnemonic_toolkit::mlock::attempts_for_test();
        let _ = run(&args, &mut stdin, &mut stdout, &mut stderr);
        assert!(
            mnemonic_toolkit::mlock::attempts_for_test() > baseline,
            "derive_child::run must invoke pin_pages_for along the cmd-handler path; \
             attempts counter did not increment",
        );
    }
}
