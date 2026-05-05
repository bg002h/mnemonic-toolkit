//! `mnemonic verify-bundle` subcommand.
//!
//! Realizes SPEC §2.2 + §5.4. Both full and watch-only emit the
//! fixed 9-element `checks` array in SPEC §5.4 order; watch-only
//! marks entropy + path-rederivation `skipped` (SPEC §2.2.2). Check
//! failures stay in §5.4 with `result: "mismatch"` per the §5.4
//! routing rule (only pre-decode failures escape to the §5.5 error
//! envelope).

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

pub fn run<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
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
fn watch_only_checks(
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
        let acc = derive_full(TREZOR_24, "", CliLanguage::English, net, tpl).unwrap();
        let bundle = synthesize_full(
            &acc.entropy,
            acc.master_fingerprint,
            acc.account_xpub,
            tpl,
            net,
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
        let mk1_strs: Vec<&str> = bundle.mk1.iter().map(|s| s.as_str()).collect();
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
        let mk1_strs: Vec<&str> = bundle.mk1.iter().map(|s| s.as_str()).collect();
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
        let mk1_strs: Vec<&str> = bundle.mk1.iter().map(|s| s.as_str()).collect();
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
        let mk1_strs: Vec<&str> = bundle.mk1.iter().map(|s| s.as_str()).collect();
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
            ms1: None,
            mk1: vec!["mk1placeholder".into()],
            md1: vec!["md1placeholder".into()],
            json: false,
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
