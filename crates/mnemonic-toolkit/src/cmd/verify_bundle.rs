//! `mnemonic verify-bundle` subcommand.
//!
//! Realizes SPEC §2.2 + §5.4. Full mode runs 5 checks; watch-only
//! runs 4 checks; check failures stay in §5.4 with result:mismatch
//! per SPEC §5.4 routing rule (only pre-decode failures escape to
//! the §5.5 error envelope).

use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::format::{VerifyBundleJson, VerifyCheck};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::{check_no_concurrent_stdin, parse_master_fingerprint, read_phrase_input};
use crate::synthesize::xpub_to_65;
use crate::template::CliTemplate;
use bitcoin::bip32::Xpub;
use clap::Args;
use std::io::Write;
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

    #[arg(long)]
    pub ms1: Option<String>,

    #[arg(long, num_args = 1.., required = true)]
    pub mk1: Vec<String>,

    #[arg(long, num_args = 1.., required = true)]
    pub md1: Vec<String>,

    #[arg(long)]
    pub json: bool,
}

pub fn run<W: Write>(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    use crate::cmd::bundle::mode_text;

    let xpub_arg = args.xpub.as_deref();
    let phrase_arg = args.phrase.as_deref();

    // SPEC §6.6 mode-violation pre-checks (mirror bundle.rs).
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

    let mut checks: Vec<VerifyCheck> = Vec::new();

    if xpub_arg.is_some() {
        // Watch-only mode (SPEC §2.2.2): 4 checks.
        run_watch_only(args, &mut checks)?;
    } else if phrase_arg.is_some() {
        // Full mode (SPEC §2.2.1): 5 checks.
        check_no_concurrent_stdin(phrase_arg, args.passphrase.as_deref())?;
        run_full(args, stdin, &mut checks)?;
    } else {
        return Err(ToolkitError::BadInput("expected --phrase or --xpub".into()));
    }

    let any_fail = checks.iter().any(|c| c.result == "fail");
    let result = if any_fail { "mismatch" } else { "ok" };

    if args.json {
        let json = VerifyBundleJson {
            schema_version: "1",
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

    let acc =
        crate::derive::derive_full(&phrase, &passphrase, language, args.network, args.template)?;

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
            let expected_path = args.template.derivation_path(args.network);
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

fn run_watch_only(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
) -> Result<(), ToolkitError> {
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

    // Check 1: mk1 parses + BCH valid.
    let mk1_strs: Vec<&str> = args.mk1.iter().map(|s| s.as_str()).collect();
    let mk_card = match mk_codec::decode(&mk1_strs) {
        Ok(c) => {
            checks.push(VerifyCheck {
                name: "mk1_decode",
                result: "ok",
                detail: "decoded successfully".into(),
            });
            Some(c)
        }
        Err(e) => {
            checks.push(VerifyCheck {
                name: "mk1_decode",
                result: "fail",
                detail: format!("{:?}", e),
            });
            None
        }
    };

    // Check 2: md1 parses + BCH valid.
    let md1_strs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    let md_desc = match md_codec::chunk::reassemble(&md1_strs) {
        Ok(d) => {
            checks.push(VerifyCheck {
                name: "md1_decode",
                result: "ok",
                detail: "decoded successfully".into(),
            });
            Some(d)
        }
        Err(e) => {
            checks.push(VerifyCheck {
                name: "md1_decode",
                result: "fail",
                detail: format!("{:?}", e),
            });
            None
        }
    };

    // Check 3: stub linkage.
    if let (Some(card), Some(desc)) = (mk_card.as_ref(), md_desc.as_ref()) {
        match md_codec::compute_wallet_policy_id(desc) {
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
                    detail: format!("policy_id: {:?}", e),
                });
            }
        }
    } else {
        checks.push(VerifyCheck {
            name: "stub_linkage",
            result: "skipped",
            detail: "decode failed".into(),
        });
    }

    // Check 4: optional xpub/fp match.
    if let Some(card) = mk_card.as_ref() {
        let xpub_match = card.xpub == supplied_xpub;
        checks.push(VerifyCheck {
            name: "mk1_xpub_match",
            result: if xpub_match { "ok" } else { "fail" },
            detail: if xpub_match {
                "matches --xpub".into()
            } else {
                "differs from --xpub".into()
            },
        });
        let fp_match = card.origin_fingerprint == Some(supplied_fp);
        checks.push(VerifyCheck {
            name: "mk1_fingerprint_match",
            result: if fp_match { "ok" } else { "fail" },
            detail: if fp_match {
                "matches --master-fingerprint".into()
            } else {
                "differs from --master-fingerprint".into()
            },
        });
    } else {
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
    }

    Ok(())
}
