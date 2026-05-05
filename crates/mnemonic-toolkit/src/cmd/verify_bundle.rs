//! `mnemonic verify-bundle` subcommand.
//!
//! Realizes SPEC §2.2 + §5.4. Both full and watch-only emit the
//! fixed 9-element `checks` array in SPEC §5.4 order; watch-only
//! marks entropy + path-rederivation `skipped` (SPEC §2.2.2). Check
//! failures stay in §5.4 with `result: "mismatch"` per the §5.4
//! routing rule (only pre-decode failures escape to the §5.5 error
//! envelope).

use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::format::{chunk_set_id_extract, VerifyBundleJson, VerifyCheck};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::{
    check_no_concurrent_stdin, parse_cosigner_spec, parse_cosigners_file, parse_master_fingerprint,
    read_phrase_input, CosignerSpec, MultisigPathFamily,
};
use crate::synthesize::xpub_to_65;
use crate::template::CliTemplate;
use bitcoin::bip32::Xpub;
use clap::Args;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

// SPEC §6.6 mode-violation symmetry mirrored from bundle.rs:
// clap-level mutual exclusion is ONLY --phrase ↔ --xpub; all other
// xpub-mode-incompatible flag rejections are runtime checks emitting
// byte-exact §6.6 strings via ToolkitError::ModeViolation (exit 2).

#[derive(Args, Debug)]
pub struct VerifyBundleArgs {
    #[arg(long, conflicts_with = "xpub")]
    pub phrase: Option<String>,

    #[arg(long, conflicts_with = "phrase")]
    pub xpub: Option<String>,

    #[arg(long = "master-fingerprint")]
    pub master_fingerprint: Option<String>,

    #[arg(long)]
    pub network: CliNetwork,

    #[arg(long)]
    pub template: CliTemplate,

    #[arg(long)]
    pub language: Option<CliLanguage>,

    #[arg(long)]
    pub passphrase: Option<String>,

    /// BIP-32 account index (default 0). Non-zero values produce md1 with
    /// PathDeclPaths::Divergent per SPEC §4.2.
    #[arg(long, default_value = "0")]
    pub account: u32,

    #[arg(long)]
    pub ms1: Option<String>,

    #[arg(long, num_args = 1.., required = true)]
    pub mk1: Vec<String>,

    #[arg(long, num_args = 1.., required = true)]
    pub md1: Vec<String>,

    #[arg(long)]
    pub json: bool,

    /// v0.2 multisig watch-only: per-cosigner spec `<xpub>:<fp>:<path>`. Repeatable.
    #[arg(long, action = clap::ArgAction::Append)]
    pub cosigner: Vec<String>,

    /// v0.2 multisig watch-only: bulk cosigners via JSON file.
    #[arg(long = "cosigners-file")]
    pub cosigners_file: Option<PathBuf>,

    /// v0.2 multisig path family (default: bip87).
    #[arg(long = "multisig-path-family", value_enum)]
    pub multisig_path_family: Option<MultisigPathFamily>,

    /// v0.2 privacy mode: expect mk1 omits master fingerprint.
    #[arg(long, default_value = "false")]
    pub privacy_preserving: bool,

    /// v0.2 multisig threshold K (1 ≤ K ≤ N ≤ 16).
    #[arg(long)]
    pub threshold: Option<u8>,

    /// v0.2 multisig cosigner count N.
    #[arg(long = "cosigner-count")]
    pub cosigner_count: Option<usize>,
}

pub fn run<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    use crate::cmd::bundle::mode_text;

    let xpub_arg = args.xpub.as_deref();
    let phrase_arg = args.phrase.as_deref();
    let multisig = args.template.is_multisig();
    let cosigner_present = !args.cosigner.is_empty();
    let cosigners_file_present = args.cosigners_file.is_some();

    // SPEC §6.6 v0.2 NEW mode-violation pre-checks (mirror bundle.rs).
    if xpub_arg.is_some() && (cosigner_present || cosigners_file_present) {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--cosigner/--cosigners-file",
            message: mode_text::XPUB_AND_COSIGNER,
        });
    }
    if cosigner_present && cosigners_file_present {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only-multisig",
            flag: "--cosigners-file",
            message: mode_text::COSIGNER_AND_COSIGNERS_FILE,
        });
    }
    if args.threshold.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--threshold",
            message: mode_text::THRESHOLD_WITHOUT_MULTISIG,
        });
    }
    if args.cosigner_count.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--cosigner-count",
            message: mode_text::COSIGNER_COUNT_WITHOUT_MULTISIG,
        });
    }
    if args.multisig_path_family.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--multisig-path-family",
            message: mode_text::PATH_FAMILY_WITHOUT_MULTISIG,
        });
    }
    if args.privacy_preserving && xpub_arg.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--privacy-preserving",
            message: mode_text::PRIVACY_WITH_XPUB,
        });
    }

    // SPEC §6.6 single-sig mode-violation pre-checks (mirror bundle.rs).
    if xpub_arg.is_some() && args.passphrase.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--passphrase",
            message: mode_text::PASSPHRASE_WITH_XPUB,
        });
    }
    if xpub_arg.is_some() && args.language.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--language",
            message: mode_text::LANGUAGE_WITH_XPUB,
        });
    }
    if xpub_arg.is_some() && args.master_fingerprint.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--xpub",
            message: mode_text::XPUB_NEEDS_FINGERPRINT,
        });
    }
    if xpub_arg.is_none() && args.master_fingerprint.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "full",
            flag: "--master-fingerprint",
            message: mode_text::FINGERPRINT_WITHOUT_XPUB,
        });
    }
    if xpub_arg == Some("-") {
        return Err(ToolkitError::BadInput(mode_text::XPUB_STDIN.to_string()));
    }

    // v0.2 multisig verify dispatch.
    if multisig {
        let mut checks: Vec<VerifyCheck> = Vec::new();
        run_multisig(args, &mut checks, stderr)?;
        let any_fail = checks.iter().any(|c| c.result == "fail");
        let result = if any_fail { "mismatch" } else { "ok" };
        if args.json {
            let json = VerifyBundleJson {
                schema_version: "2",
                result,
                checks,
            };
            serde_json::to_writer(&mut *stdout, &json).ok();
            writeln!(stdout).ok();
        } else {
            for c in &checks {
                writeln!(stdout, "{}: {} {}", c.name, c.result, c.detail).ok();
            }
            writeln!(stdout, "result: {}", result).ok();
        }
        return Ok(if any_fail { 4 } else { 0 });
    }

    let mut checks: Vec<VerifyCheck> = Vec::new();

    if xpub_arg.is_some() {
        // Watch-only mode (SPEC §2.2.2): emits the §5.4 9-element array
        // with entropy + path-rederivation marked `skipped`.
        run_watch_only(args, &mut checks, stderr)?;
    } else if phrase_arg.is_some() {
        // Full mode (SPEC §2.2.1): emits the §5.4 9-element array.
        check_no_concurrent_stdin(phrase_arg, args.passphrase.as_deref())?;
        run_full(args, stdin, &mut checks)?;
    } else {
        return Err(ToolkitError::BadInput("expected --phrase or --xpub".into()));
    }

    let any_fail = checks.iter().any(|c| c.result == "fail");
    let result = if any_fail { "mismatch" } else { "ok" };

    if args.json {
        // v0.2: schema_version "2"; single-sig checks shape unchanged from v0.1
        // (multisig array shape comes in Phase C).
        let json = VerifyBundleJson {
            schema_version: "2",
            result,
            checks,
        };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        for c in &checks {
            writeln!(stdout, "{}: {} {}", c.name, c.result, c.detail).ok();
        }
        writeln!(stdout, "result: {}", result).ok();
    }

    Ok(if any_fail { 4 } else { 0 })
}

fn run_full(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    checks: &mut Vec<VerifyCheck>,
) -> Result<(), ToolkitError> {
    let phrase = read_phrase_input(args.phrase.as_deref(), stdin)?;
    let passphrase = args.passphrase.clone().unwrap_or_default();
    let language = args.language.unwrap_or_default();

    let acc = crate::derive::derive_full(
        &phrase,
        &passphrase,
        language,
        args.network,
        args.template,
        args.account,
    )?;

    // Check 1: ms1 entropy match.
    if let Some(ms1) = args.ms1.as_deref() {
        match ms_codec::decode(ms1) {
            Ok((_tag, payload)) => {
                if let ms_codec::Payload::Entr(e) = payload {
                    if e == acc.entropy {
                        checks.push(VerifyCheck {
                            name: "ms1_entropy_match",
                            result: "ok",
                            detail: "entropy bytes match".into(),
                        });
                    } else {
                        checks.push(VerifyCheck {
                            name: "ms1_entropy_match",
                            result: "fail",
                            detail: format!("decoded {}-byte entropy != derived", e.len()),
                        });
                    }
                } else {
                    checks.push(VerifyCheck {
                        name: "ms1_entropy_match",
                        result: "fail",
                        detail: "decoded ms1 payload is not Entr".into(),
                    });
                }
            }
            Err(e) => {
                checks.push(VerifyCheck {
                    name: "ms1_entropy_match",
                    result: "fail",
                    detail: format!("ms1 decode: {:?}", e),
                });
            }
        }
    } else {
        checks.push(VerifyCheck {
            name: "ms1_entropy_match",
            result: "skipped",
            detail: "no --ms1 supplied".into(),
        });
    }

    // Check 2: mk1 decode + xpub/fp/path match.
    let mk1_strs: Vec<&str> = args.mk1.iter().map(|s| s.as_str()).collect();
    match mk_codec::decode(&mk1_strs) {
        Ok(card) => {
            checks.push(VerifyCheck {
                name: "mk1_decode",
                result: "ok",
                detail: "decoded successfully".into(),
            });
            let xpub_match = card.xpub == acc.account_xpub;
            checks.push(VerifyCheck {
                name: "mk1_xpub_match",
                result: if xpub_match { "ok" } else { "fail" },
                detail: if xpub_match {
                    "xpub matches".into()
                } else {
                    "xpub does not match derived".into()
                },
            });
            let fp_match = card.origin_fingerprint == Some(acc.master_fingerprint);
            checks.push(VerifyCheck {
                name: "mk1_fingerprint_match",
                result: if fp_match { "ok" } else { "fail" },
                detail: if fp_match {
                    "fp matches".into()
                } else {
                    "master fingerprint does not match".into()
                },
            });
            let expected_path = args.template.derivation_path(args.network, args.account);
            let path_match = card.origin_path == expected_path;
            checks.push(VerifyCheck {
                name: "mk1_path_match",
                result: if path_match { "ok" } else { "fail" },
                detail: if path_match {
                    "path matches".into()
                } else {
                    format!("expected {}, got {}", expected_path, card.origin_path)
                },
            });

            // Check 3+5: md1 decode + cross-binding.
            verify_md1_and_stub(args, &card, checks);
        }
        Err(e) => {
            checks.push(VerifyCheck {
                name: "mk1_decode",
                result: "fail",
                detail: format!("{:?}", e),
            });
            checks.push(VerifyCheck {
                name: "mk1_xpub_match",
                result: "skipped",
                detail: "mk1 decode failed".into(),
            });
            checks.push(VerifyCheck {
                name: "mk1_fingerprint_match",
                result: "skipped",
                detail: "mk1 decode failed".into(),
            });
            checks.push(VerifyCheck {
                name: "mk1_path_match",
                result: "skipped",
                detail: "mk1 decode failed".into(),
            });

            // Try md1 anyway for diagnostic completeness.
            verify_md1_only(args, checks);
        }
    }

    Ok(())
}

fn verify_md1_and_stub(
    args: &VerifyBundleArgs,
    card: &mk_codec::KeyCard,
    checks: &mut Vec<VerifyCheck>,
) {
    let md1_strs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    match md_codec::chunk::reassemble(&md1_strs) {
        Ok(desc) => {
            checks.push(VerifyCheck {
                name: "md1_decode",
                result: "ok",
                detail: "decoded successfully".into(),
            });
            let wp = desc.is_wallet_policy();
            checks.push(VerifyCheck {
                name: "md1_wallet_policy",
                result: if wp { "ok" } else { "fail" },
                detail: if wp {
                    "wallet-policy mode confirmed".into()
                } else {
                    "descriptor is template-only (no pubkeys TLV)".into()
                },
            });

            if wp {
                let xpub_65_expected = xpub_to_65(&card.xpub);
                let xpub_match = desc
                    .tlv
                    .pubkeys
                    .as_ref()
                    .and_then(|v| v.first())
                    .map(|(_, b)| b == &xpub_65_expected)
                    .unwrap_or(false);
                checks.push(VerifyCheck {
                    name: "md1_xpub_match",
                    result: if xpub_match { "ok" } else { "fail" },
                    detail: if xpub_match {
                        "65-byte xpub matches mk1's xpub".into()
                    } else {
                        "md1 xpub differs from mk1's".into()
                    },
                });
            } else {
                checks.push(VerifyCheck {
                    name: "md1_xpub_match",
                    result: "skipped",
                    detail: "not in wallet-policy mode".into(),
                });
            }

            match md_codec::compute_wallet_policy_id(&desc) {
                Ok(pid) => {
                    let stub_match = card.policy_id_stubs.first().copied().unwrap_or([0u8; 4])[..]
                        == pid.as_bytes()[..4];
                    checks.push(VerifyCheck {
                        name: "stub_linkage",
                        result: if stub_match { "ok" } else { "fail" },
                        detail: if stub_match {
                            "policy_id_stub[0..4] matches mk1's stub[0]".into()
                        } else {
                            "stub linkage broken".into()
                        },
                    });
                }
                Err(e) => {
                    checks.push(VerifyCheck {
                        name: "stub_linkage",
                        result: "fail",
                        detail: format!("policy_id compute: {:?}", e),
                    });
                }
            }
        }
        Err(e) => {
            checks.push(VerifyCheck {
                name: "md1_decode",
                result: "fail",
                detail: format!("{:?}", e),
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy",
                result: "skipped",
                detail: "md1 decode failed".into(),
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match",
                result: "skipped",
                detail: "md1 decode failed".into(),
            });
            checks.push(VerifyCheck {
                name: "stub_linkage",
                result: "skipped",
                detail: "md1 decode failed".into(),
            });
        }
    }
}

fn verify_md1_only(args: &VerifyBundleArgs, checks: &mut Vec<VerifyCheck>) {
    let md1_strs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    match md_codec::chunk::reassemble(&md1_strs) {
        Ok(desc) => {
            checks.push(VerifyCheck {
                name: "md1_decode",
                result: "ok",
                detail: "decoded successfully".into(),
            });
            let wp = desc.is_wallet_policy();
            checks.push(VerifyCheck {
                name: "md1_wallet_policy",
                result: if wp { "ok" } else { "fail" },
                detail: "".into(),
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match",
                result: "skipped",
                detail: "mk1 decode failed; no reference xpub".into(),
            });
            checks.push(VerifyCheck {
                name: "stub_linkage",
                result: "skipped",
                detail: "mk1 decode failed".into(),
            });
        }
        Err(e) => {
            checks.push(VerifyCheck {
                name: "md1_decode",
                result: "fail",
                detail: format!("{:?}", e),
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy",
                result: "skipped",
                detail: "".into(),
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match",
                result: "skipped",
                detail: "".into(),
            });
            checks.push(VerifyCheck {
                name: "stub_linkage",
                result: "skipped",
                detail: "".into(),
            });
        }
    }
}

fn run_watch_only<E: Write>(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // SPEC §2.2.2 watch-only-cannot-verify-path warning. Emitted before any
    // parse error so the user always sees it, even if --xpub fails to parse.
    writeln!(
        stderr,
        "warning: watch-only verify-bundle does not verify --xpub is actually at the"
    )
    .ok();
    writeln!(
        stderr,
        "warning: claimed BIP path m/<purpose>'/<coin>'/0' (no master seed available"
    )
    .ok();
    writeln!(
        stderr,
        "warning: for re-derivation). Use --phrase mode for end-to-end verification."
    )
    .ok();

    let xpub_str = args.xpub.as_deref().expect("xpub set in watch-only mode");
    let fp_str = args
        .master_fingerprint
        .as_deref()
        .ok_or_else(|| ToolkitError::BadInput("--xpub requires --master-fingerprint".into()))?;
    let supplied_xpub = Xpub::from_str(xpub_str)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::XpubParse(format!("{}", e))))?;
    let supplied_fp = parse_master_fingerprint(fp_str)?;

    if supplied_xpub.network != args.network.network_kind() {
        return Err(ToolkitError::NetworkMismatch {
            xpub_network: if supplied_xpub.network == bitcoin::NetworkKind::Main {
                "mainnet"
            } else {
                "testnet/signet/regtest"
            },
            expected: args.network.human_name(),
        });
    }

    let mk1_strs: Vec<&str> = args.mk1.iter().map(|s| s.as_str()).collect();
    let mk_decode = mk_codec::decode(&mk1_strs).map_err(|e| format!("{:?}", e));

    let md1_strs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    let md_decode = md_codec::chunk::reassemble(&md1_strs).map_err(|e| format!("{:?}", e));

    let emitted = watch_only_checks(
        &supplied_xpub,
        supplied_fp,
        mk_decode.as_ref(),
        md_decode.as_ref(),
    );
    checks.extend(emitted);
    Ok(())
}

/// Emit the SPEC §5.4 9-element checks array for watch-only mode.
///
/// Order is fixed (SPEC §5.4 lines 538-548); entropy and path-rederivation
/// are `skipped` per SPEC §2.2.2 (no master seed in watch-only). Decode
/// failures cascade: mk1 fail skips its 3 mk1_*_match deps; md1 fail
/// skips wallet_policy + md1_xpub_match + stub_linkage.
pub(crate) fn watch_only_checks(
    supplied_xpub: &Xpub,
    supplied_fp: bitcoin::bip32::Fingerprint,
    mk_decode: Result<&mk_codec::KeyCard, &String>,
    md_decode: Result<&md_codec::Descriptor, &String>,
) -> Vec<VerifyCheck> {
    let mut out: Vec<VerifyCheck> = Vec::with_capacity(9);

    // 1. ms1_entropy_match — always skipped (no entropy in watch-only).
    out.push(VerifyCheck {
        name: "ms1_entropy_match",
        result: "skipped",
        detail: "watch-only mode: no entropy known to toolkit".into(),
    });

    // 2. mk1_decode.
    match mk_decode {
        Ok(_) => out.push(VerifyCheck {
            name: "mk1_decode",
            result: "ok",
            detail: "decoded successfully".into(),
        }),
        Err(e) => out.push(VerifyCheck {
            name: "mk1_decode",
            result: "fail",
            detail: e.clone(),
        }),
    }

    // 3. mk1_xpub_match.
    match mk_decode {
        Ok(card) => {
            let m = &card.xpub == supplied_xpub;
            out.push(VerifyCheck {
                name: "mk1_xpub_match",
                result: if m { "ok" } else { "fail" },
                detail: if m {
                    "matches --xpub".into()
                } else {
                    "differs from --xpub".into()
                },
            });
        }
        Err(_) => out.push(VerifyCheck {
            name: "mk1_xpub_match",
            result: "skipped",
            detail: "mk1 decode failed".into(),
        }),
    }

    // 4. mk1_fingerprint_match.
    match mk_decode {
        Ok(card) => {
            let m = card.origin_fingerprint == Some(supplied_fp);
            out.push(VerifyCheck {
                name: "mk1_fingerprint_match",
                result: if m { "ok" } else { "fail" },
                detail: if m {
                    "matches --master-fingerprint".into()
                } else {
                    "differs from --master-fingerprint".into()
                },
            });
        }
        Err(_) => out.push(VerifyCheck {
            name: "mk1_fingerprint_match",
            result: "skipped",
            detail: "mk1 decode failed".into(),
        }),
    }

    // 5. mk1_path_match — always skipped in watch-only (SPEC §2.2.2).
    out.push(VerifyCheck {
        name: "mk1_path_match",
        result: "skipped",
        detail: "watch-only mode: path verification requires master seed (SPEC §2.2.2)".into(),
    });

    // 6. md1_decode.
    match md_decode {
        Ok(_) => out.push(VerifyCheck {
            name: "md1_decode",
            result: "ok",
            detail: "decoded successfully".into(),
        }),
        Err(e) => out.push(VerifyCheck {
            name: "md1_decode",
            result: "fail",
            detail: e.clone(),
        }),
    }

    // 7. md1_wallet_policy.
    match md_decode {
        Ok(desc) => {
            let wp = desc.is_wallet_policy();
            out.push(VerifyCheck {
                name: "md1_wallet_policy",
                result: if wp { "ok" } else { "fail" },
                detail: if wp {
                    "wallet-policy mode confirmed".into()
                } else {
                    "descriptor is template-only (no pubkeys TLV)".into()
                },
            });
        }
        Err(_) => out.push(VerifyCheck {
            name: "md1_wallet_policy",
            result: "skipped",
            detail: "md1 decode failed".into(),
        }),
    }

    // 8. md1_xpub_match — substantive in watch-only: compare 65-byte
    // form of supplied --xpub against md1's pubkeys[0].
    match md_decode {
        Ok(desc) => {
            if desc.is_wallet_policy() {
                let xpub_65 = xpub_to_65(supplied_xpub);
                let m = desc
                    .tlv
                    .pubkeys
                    .as_ref()
                    .and_then(|v| v.first())
                    .map(|(_, b)| b == &xpub_65)
                    .unwrap_or(false);
                out.push(VerifyCheck {
                    name: "md1_xpub_match",
                    result: if m { "ok" } else { "fail" },
                    detail: if m {
                        "65-byte xpub matches --xpub".into()
                    } else {
                        "md1 xpub differs from --xpub".into()
                    },
                });
            } else {
                out.push(VerifyCheck {
                    name: "md1_xpub_match",
                    result: "skipped",
                    detail: "not in wallet-policy mode".into(),
                });
            }
        }
        Err(_) => out.push(VerifyCheck {
            name: "md1_xpub_match",
            result: "skipped",
            detail: "md1 decode failed".into(),
        }),
    }

    // 9. stub_linkage.
    match (mk_decode, md_decode) {
        (Ok(card), Ok(desc)) => match md_codec::compute_wallet_policy_id(desc) {
            Ok(pid) => {
                let stub_match = card.policy_id_stubs.first().copied().unwrap_or([0u8; 4])[..]
                    == pid.as_bytes()[..4];
                out.push(VerifyCheck {
                    name: "stub_linkage",
                    result: if stub_match { "ok" } else { "fail" },
                    detail: if stub_match {
                        "policy_id_stub[0..4] matches mk1's stub[0]".into()
                    } else {
                        "stub linkage broken".into()
                    },
                });
            }
            Err(e) => out.push(VerifyCheck {
                name: "stub_linkage",
                result: "fail",
                detail: format!("policy_id compute: {:?}", e),
            }),
        },
        _ => out.push(VerifyCheck {
            name: "stub_linkage",
            result: "skipped",
            detail: "decode failed".into(),
        }),
    }

    out
}

/// Multisig verify-bundle entry. Implements SPEC §2.2.1 multisig grouping +
/// stub-list mismatch detection. Phase C scope: happy path + stub-list mismatch.
/// Per-cosigner check enumeration (3 + 6N) and watch-only-vs-full split detail
/// are deferred to Phase D.
fn run_multisig<E: Write>(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let xpub_arg = args.xpub.as_deref();
    let phrase_arg = args.phrase.as_deref();
    let cosigner_present = !args.cosigner.is_empty();
    let cosigners_file_present = args.cosigners_file.is_some();
    let watch_only_multi = cosigner_present || cosigners_file_present;

    if watch_only_multi {
        // SPEC §2.2.2 multisig watch-only stderr warning.
        writeln!(
            stderr,
            "warning: watch-only multisig verify-bundle does not verify --cosigner xpubs are at the"
        )
        .ok();
        writeln!(
            stderr,
            "warning: claimed BIP path (no per-cosigner master seed available for re-derivation)."
        )
        .ok();
        writeln!(
            stderr,
            "warning: Use --phrase mode for end-to-end verification of self-multisig backups."
        )
        .ok();
    }
    let _ = xpub_arg;
    let _ = phrase_arg;

    // SPEC §2.2.1 step 1: group --mk1 chunks by chunk_set_id (Chunked) or per-string (SingleString).
    // Build groups: Vec<Vec<&str>>, where each group is a per-cosigner card-set.
    use std::collections::BTreeMap;
    let mut chunked_groups: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
    let mut single_groups: Vec<Vec<&str>> = Vec::new();
    for s in &args.mk1 {
        match chunk_set_id_extract(s) {
            Some(csi) => chunked_groups.entry(csi).or_default().push(s.as_str()),
            None => single_groups.push(vec![s.as_str()]),
        }
    }
    let groups: Vec<Vec<&str>> = chunked_groups.into_values().chain(single_groups).collect();

    // Decode each group.
    let mut decoded: Vec<mk_codec::KeyCard> = Vec::with_capacity(groups.len());
    let mut decode_errors: Vec<String> = Vec::with_capacity(groups.len());
    for g in &groups {
        match mk_codec::decode(g) {
            Ok(c) => {
                checks.push(VerifyCheck {
                    name: "mk1_decode",
                    result: "ok",
                    detail: format!("group of {} chunks decoded", g.len()),
                });
                decoded.push(c);
                decode_errors.push(String::new());
            }
            Err(e) => {
                checks.push(VerifyCheck {
                    name: "mk1_decode",
                    result: "fail",
                    detail: format!("{:?}", e),
                });
                decode_errors.push(format!("{:?}", e));
            }
        }
    }

    // Per SPEC §2.2.1 step 5b: stub-list consistency across all decoded cards.
    if decoded.len() >= 2 {
        let first = &decoded[0].policy_id_stubs;
        let all_match = decoded[1..].iter().all(|c| &c.policy_id_stubs == first);
        checks.push(VerifyCheck {
            name: "mk1_stub_list_consistent",
            result: if all_match { "ok" } else { "fail" },
            detail: if all_match {
                "all decoded cards share the same policy_id_stubs list".into()
            } else {
                "policy_id_stubs lists differ across cards; mixed bundle".into()
            },
        });
    }

    // SPEC §2.2.1 step 5: group-count vs expected-N (when known).
    let expected_n = if watch_only_multi {
        // Resolve cosigner specs solely to learn expected N.
        let cosigners: Vec<CosignerSpec> = if let Some(file) = &args.cosigners_file {
            parse_cosigners_file(file)?
        } else {
            let mut out = Vec::with_capacity(args.cosigner.len());
            for (i, s) in args.cosigner.iter().enumerate() {
                out.push(parse_cosigner_spec(s, i)?);
            }
            out
        };
        Some(cosigners.len())
    } else {
        args.cosigner_count
    };
    if let Some(n) = expected_n {
        let m = groups.len();
        checks.push(VerifyCheck {
            name: "mk1_group_count",
            result: if m == n { "ok" } else { "fail" },
            detail: if m == n {
                format!("{} cosigner card-sets (matches N={})", m, n)
            } else {
                format!("expected {} cosigner card-sets; got {}", n, m)
            },
        });
    }

    // md1 decode.
    let md1_strs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    match md_codec::chunk::reassemble(&md1_strs) {
        Ok(desc) => {
            checks.push(VerifyCheck {
                name: "md1_decode",
                result: "ok",
                detail: "decoded successfully".into(),
            });
            let wp = desc.is_wallet_policy();
            checks.push(VerifyCheck {
                name: "md1_wallet_policy",
                result: if wp { "ok" } else { "fail" },
                detail: if wp {
                    "wallet-policy mode confirmed".into()
                } else {
                    "descriptor is template-only (no pubkeys TLV)".into()
                },
            });
            if let Ok(pid) = md_codec::compute_wallet_policy_id(&desc) {
                let expected_stub: [u8; 4] = pid.as_bytes()[..4].try_into().unwrap();
                let stub_match = if let Some(first) = decoded.first() {
                    first.policy_id_stubs.iter().any(|s| *s == expected_stub)
                } else {
                    false
                };
                checks.push(VerifyCheck {
                    name: "stub_linkage",
                    result: if stub_match { "ok" } else { "fail" },
                    detail: if stub_match {
                        "descriptor's policy_id stub appears in mk1 stubs list".into()
                    } else {
                        "descriptor's policy_id stub not in mk1 stubs list".into()
                    },
                });
            } else {
                checks.push(VerifyCheck {
                    name: "stub_linkage",
                    result: "fail",
                    detail: "policy_id compute failed".into(),
                });
            }
        }
        Err(e) => {
            checks.push(VerifyCheck {
                name: "md1_decode",
                result: "fail",
                detail: format!("{:?}", e),
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy",
                result: "skipped",
                detail: "md1 decode failed".into(),
            });
            checks.push(VerifyCheck {
                name: "stub_linkage",
                result: "skipped",
                detail: "md1 decode failed".into(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod watch_only_tests {
    use super::*;
    use crate::derive::derive_full;
    use crate::language::CliLanguage;
    use crate::synthesize::{synthesize_full, Bundle};
    use crate::template::CliTemplate;

    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    /// SPEC §5.4 fixed name order, asserted by every emitted-array test.
    const SPEC_NAMES: [&str; 9] = [
        "ms1_entropy_match",
        "mk1_decode",
        "mk1_xpub_match",
        "mk1_fingerprint_match",
        "mk1_path_match",
        "md1_decode",
        "md1_wallet_policy",
        "md1_xpub_match",
        "stub_linkage",
    ];

    fn fixture_bundle() -> (Bundle, Xpub, bitcoin::bip32::Fingerprint) {
        let net = CliNetwork::Mainnet;
        let tpl = CliTemplate::Bip84;
        let acc = derive_full(TREZOR_24, "", CliLanguage::English, net, tpl, 0).unwrap();
        let bundle = synthesize_full(
            &acc.entropy,
            acc.master_fingerprint,
            acc.account_xpub,
            tpl,
            net,
            0,
        )
        .unwrap();
        (bundle, acc.account_xpub, acc.master_fingerprint)
    }

    fn assert_spec_order(checks: &[VerifyCheck]) {
        assert_eq!(
            checks.len(),
            9,
            "watch-only must emit exactly 9 checks per SPEC §5.4"
        );
        for (i, c) in checks.iter().enumerate() {
            assert_eq!(
                c.name, SPEC_NAMES[i],
                "check[{i}] name out of SPEC §5.4 order"
            );
        }
    }

    #[test]
    fn happy_path_emits_9_checks_in_spec_order() {
        let (bundle, xpub, fp) = fixture_bundle();
        let mk1_v = bundle.mk1.as_single().unwrap();
        let mk1_strs: Vec<&str> = mk1_v.iter().map(|s| s.as_str()).collect();
        let card = mk_codec::decode(&mk1_strs).unwrap();
        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let desc = md_codec::chunk::reassemble(&md1_strs).unwrap();

        let checks = watch_only_checks(&xpub, fp, Ok(&card), Ok(&desc));
        assert_spec_order(&checks);

        // Watch-only-skipped entries:
        assert_eq!(checks[0].result, "skipped"); // ms1_entropy_match
        assert_eq!(checks[4].result, "skipped"); // mk1_path_match

        // Substantive checks all pass:
        assert_eq!(checks[1].result, "ok"); // mk1_decode
        assert_eq!(checks[2].result, "ok"); // mk1_xpub_match
        assert_eq!(checks[3].result, "ok"); // mk1_fingerprint_match
        assert_eq!(checks[5].result, "ok"); // md1_decode
        assert_eq!(checks[6].result, "ok"); // md1_wallet_policy
        assert_eq!(checks[7].result, "ok"); // md1_xpub_match
        assert_eq!(checks[8].result, "ok"); // stub_linkage
    }

    #[test]
    fn mk1_decode_fail_cascades_to_three_skipped() {
        let (bundle, xpub, fp) = fixture_bundle();
        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let desc = md_codec::chunk::reassemble(&md1_strs).unwrap();
        let err = "synthetic mk decode error".to_string();

        let checks = watch_only_checks(&xpub, fp, Err(&err), Ok(&desc));
        assert_spec_order(&checks);
        assert_eq!(checks[1].result, "fail"); // mk1_decode
        assert_eq!(checks[2].result, "skipped"); // mk1_xpub_match
        assert_eq!(checks[3].result, "skipped"); // mk1_fingerprint_match
        assert_eq!(checks[4].result, "skipped"); // mk1_path_match (always skipped anyway)
        assert_eq!(checks[5].result, "ok"); // md1_decode still ok
        assert_eq!(checks[6].result, "ok"); // md1_wallet_policy
        assert_eq!(checks[7].result, "ok"); // md1_xpub_match (compares against supplied xpub)
        assert_eq!(checks[8].result, "skipped"); // stub_linkage needs both
    }

    #[test]
    fn md1_decode_fail_cascades_to_three_skipped() {
        let (bundle, xpub, fp) = fixture_bundle();
        let mk1_v = bundle.mk1.as_single().unwrap();
        let mk1_strs: Vec<&str> = mk1_v.iter().map(|s| s.as_str()).collect();
        let card = mk_codec::decode(&mk1_strs).unwrap();
        let err = "synthetic md decode error".to_string();

        let checks = watch_only_checks(&xpub, fp, Ok(&card), Err(&err));
        assert_spec_order(&checks);
        assert_eq!(checks[1].result, "ok"); // mk1_decode
        assert_eq!(checks[2].result, "ok"); // mk1_xpub_match
        assert_eq!(checks[3].result, "ok"); // mk1_fingerprint_match
        assert_eq!(checks[5].result, "fail"); // md1_decode
        assert_eq!(checks[6].result, "skipped"); // md1_wallet_policy
        assert_eq!(checks[7].result, "skipped"); // md1_xpub_match
        assert_eq!(checks[8].result, "skipped"); // stub_linkage
    }

    #[test]
    fn xpub_mismatch_fails_both_xpub_checks() {
        let (bundle, _correct_xpub, fp) = fixture_bundle();
        let mk1_v = bundle.mk1.as_single().unwrap();
        let mk1_strs: Vec<&str> = mk1_v.iter().map(|s| s.as_str()).collect();
        let card = mk_codec::decode(&mk1_strs).unwrap();
        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let desc = md_codec::chunk::reassemble(&md1_strs).unwrap();

        // Substitute a different xpub: derive bip44 instead of bip84.
        let other_acc = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip44,
            0,
        )
        .unwrap();
        let wrong_xpub = other_acc.account_xpub;

        let checks = watch_only_checks(&wrong_xpub, fp, Ok(&card), Ok(&desc));
        assert_spec_order(&checks);
        assert_eq!(checks[2].result, "fail"); // mk1_xpub_match
        assert_eq!(checks[7].result, "fail"); // md1_xpub_match
        assert_eq!(checks[3].result, "ok"); // fingerprint still matches (master_fingerprint is path-independent)
        assert_eq!(checks[8].result, "ok"); // stub_linkage still holds (mk's stub binds md, not xpub)
    }

    #[test]
    fn fingerprint_mismatch_fails_only_fingerprint_check() {
        let (bundle, xpub, _correct_fp) = fixture_bundle();
        let mk1_v = bundle.mk1.as_single().unwrap();
        let mk1_strs: Vec<&str> = mk1_v.iter().map(|s| s.as_str()).collect();
        let card = mk_codec::decode(&mk1_strs).unwrap();
        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let desc = md_codec::chunk::reassemble(&md1_strs).unwrap();
        let wrong_fp = bitcoin::bip32::Fingerprint::from([0xDE, 0xAD, 0xBE, 0xEF]);

        let checks = watch_only_checks(&xpub, wrong_fp, Ok(&card), Ok(&desc));
        assert_spec_order(&checks);
        assert_eq!(checks[3].result, "fail"); // mk1_fingerprint_match
        assert_eq!(checks[2].result, "ok");
        assert_eq!(checks[7].result, "ok");
    }

    /// SPEC §2.2.2: watch-only verify-bundle MUST emit the 3-line
    /// path-rederivation warning to stderr. The warning is emitted at the
    /// top of `run_watch_only` BEFORE any parse error so the user always
    /// sees it, even when --xpub fails to parse.
    #[test]
    fn watch_only_emits_spec_2_2_2_warning_to_stderr() {
        let mut stdin = std::io::empty();
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let args = VerifyBundleArgs {
            phrase: None,
            xpub: Some("xpub6BadInvalidShortString".into()),
            master_fingerprint: Some("deadbeef".into()),
            network: CliNetwork::Mainnet,
            template: CliTemplate::Bip84,
            language: None,
            passphrase: None,
            account: 0,
            ms1: None,
            mk1: vec!["mk1placeholder".into()],
            md1: vec!["md1placeholder".into()],
            json: false,
            cosigner: Vec::new(),
            cosigners_file: None,
            multisig_path_family: None,
            privacy_preserving: false,
            threshold: None,
            cosigner_count: None,
        };
        // run() will fail at xpub parse, but the §2.2.2 warning should
        // already be on stderr.
        let _ = run(&args, &mut stdin, &mut stdout, &mut stderr);
        let stderr_text = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_text.contains("watch-only verify-bundle does not verify"),
            "missing line 1 of §2.2.2 warning; got: {stderr_text:?}"
        );
        assert!(
            stderr_text.contains("BIP path m/<purpose>'/<coin>'/0'"),
            "missing line 2 of §2.2.2 warning; got: {stderr_text:?}"
        );
        assert!(
            stderr_text.contains("Use --phrase mode for end-to-end verification."),
            "missing line 3 of §2.2.2 warning; got: {stderr_text:?}"
        );
    }
}
