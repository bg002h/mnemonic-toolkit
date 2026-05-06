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

    /// Template name. Mutually-required-one-of with --descriptor / --descriptor-file.
    #[arg(long, required_unless_present_any = ["descriptor", "descriptor_file"])]
    pub template: Option<CliTemplate>,

    /// User-supplied descriptor (v0.3 §5.7 verify-bundle re-parse path).
    #[arg(long, conflicts_with = "descriptor_file")]
    pub descriptor: Option<String>,

    /// User-supplied descriptor file (single-line UTF-8).
    #[arg(long = "descriptor-file")]
    pub descriptor_file: Option<PathBuf>,

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

impl VerifyBundleArgs {
    fn template_unchecked(&self) -> CliTemplate {
        self.template
            .expect("template-mode dispatch contract — descriptor-mode escapes earlier")
    }
}

pub fn run<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    use crate::cmd::bundle::mode_text;

    // v0.3 descriptor-mode dispatch (escapes before template_unchecked).
    let descriptor_mode = args.descriptor.is_some() || args.descriptor_file.is_some();
    if descriptor_mode && args.template.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--template",
            message: mode_text::DESCRIPTOR_AND_TEMPLATE,
        });
    }
    if descriptor_mode {
        return descriptor_mode_verify_run(args, stdin, stdout);
    }

    let xpub_arg = args.xpub.as_deref();
    let phrase_arg = args.phrase.as_deref();
    let multisig = args.template_unchecked().is_multisig();
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
                schema_version: "4",
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
            schema_version: "4",
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
        args.template_unchecked(),
        args.account,
    )?;

    // Check 1: ms1 entropy match.
    if let Some(ms1) = args.ms1.as_deref() {
        match ms_codec::decode(ms1) {
            Ok((_tag, payload)) => {
                if let ms_codec::Payload::Entr(e) = payload {
                    if e == acc.entropy {
                        checks.push(VerifyCheck {
                            name: "ms1_entropy_match".into(),
                            result: "ok",
                            detail: "entropy bytes match".into(),
                        });
                    } else {
                        checks.push(VerifyCheck {
                            name: "ms1_entropy_match".into(),
                            result: "fail",
                            detail: format!("decoded {}-byte entropy != derived", e.len()),
                        });
                    }
                } else {
                    checks.push(VerifyCheck {
                        name: "ms1_entropy_match".into(),
                        result: "fail",
                        detail: "decoded ms1 payload is not Entr".into(),
                    });
                }
            }
            Err(e) => {
                checks.push(VerifyCheck {
                    name: "ms1_entropy_match".into(),
                    result: "fail",
                    detail: format!("ms1 decode: {:?}", e),
                });
            }
        }
    } else {
        checks.push(VerifyCheck {
            name: "ms1_entropy_match".into(),
            result: "skipped",
            detail: "no --ms1 supplied".into(),
        });
    }

    // Check 2: mk1 decode + xpub/fp/path match.
    let mk1_strs: Vec<&str> = args.mk1.iter().map(|s| s.as_str()).collect();
    match mk_codec::decode(&mk1_strs) {
        Ok(card) => {
            checks.push(VerifyCheck {
                name: "mk1_decode".into(),
                result: "ok",
                detail: "decoded successfully".into(),
            });
            let xpub_match = card.xpub == acc.account_xpub;
            checks.push(VerifyCheck {
                name: "mk1_xpub_match".into(),
                result: if xpub_match { "ok" } else { "fail" },
                detail: if xpub_match {
                    "xpub matches".into()
                } else {
                    "xpub does not match derived".into()
                },
            });
            let fp_match = card.origin_fingerprint == Some(acc.master_fingerprint);
            checks.push(VerifyCheck {
                name: "mk1_fingerprint_match".into(),
                result: if fp_match { "ok" } else { "fail" },
                detail: if fp_match {
                    "fp matches".into()
                } else {
                    "master fingerprint does not match".into()
                },
            });
            let expected_path = args
                .template_unchecked()
                .derivation_path(args.network, args.account);
            let path_match = card.origin_path == expected_path;
            checks.push(VerifyCheck {
                name: "mk1_path_match".into(),
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
                name: "mk1_decode".into(),
                result: "fail",
                detail: format!("{:?}", e),
            });
            checks.push(VerifyCheck {
                name: "mk1_xpub_match".into(),
                result: "skipped",
                detail: "mk1 decode failed".into(),
            });
            checks.push(VerifyCheck {
                name: "mk1_fingerprint_match".into(),
                result: "skipped",
                detail: "mk1 decode failed".into(),
            });
            checks.push(VerifyCheck {
                name: "mk1_path_match".into(),
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
                name: "md1_decode".into(),
                result: "ok",
                detail: "decoded successfully".into(),
            });
            let wp = desc.is_wallet_policy();
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
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
                    name: "md1_xpub_match".into(),
                    result: if xpub_match { "ok" } else { "fail" },
                    detail: if xpub_match {
                        "65-byte xpub matches mk1's xpub".into()
                    } else {
                        "md1 xpub differs from mk1's".into()
                    },
                });
            } else {
                checks.push(VerifyCheck {
                    name: "md1_xpub_match".into(),
                    result: "skipped",
                    detail: "not in wallet-policy mode".into(),
                });
            }

            match md_codec::compute_wallet_policy_id(&desc) {
                Ok(pid) => {
                    let stub_match = card.policy_id_stubs.first().copied().unwrap_or([0u8; 4])[..]
                        == pid.as_bytes()[..4];
                    checks.push(VerifyCheck {
                        name: "stub_linkage".into(),
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
                        name: "stub_linkage".into(),
                        result: "fail",
                        detail: format!("policy_id compute: {:?}", e),
                    });
                }
            }
        }
        Err(e) => {
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                result: "fail",
                detail: format!("{:?}", e),
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                result: "skipped",
                detail: "md1 decode failed".into(),
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match".into(),
                result: "skipped",
                detail: "md1 decode failed".into(),
            });
            checks.push(VerifyCheck {
                name: "stub_linkage".into(),
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
                name: "md1_decode".into(),
                result: "ok",
                detail: "decoded successfully".into(),
            });
            let wp = desc.is_wallet_policy();
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                result: if wp { "ok" } else { "fail" },
                detail: "".into(),
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match".into(),
                result: "skipped",
                detail: "mk1 decode failed; no reference xpub".into(),
            });
            checks.push(VerifyCheck {
                name: "stub_linkage".into(),
                result: "skipped",
                detail: "mk1 decode failed".into(),
            });
        }
        Err(e) => {
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                result: "fail",
                detail: format!("{:?}", e),
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                result: "skipped",
                detail: "".into(),
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match".into(),
                result: "skipped",
                detail: "".into(),
            });
            checks.push(VerifyCheck {
                name: "stub_linkage".into(),
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
        args.privacy_preserving,
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
    privacy_preserving: bool,
) -> Vec<VerifyCheck> {
    let mut out: Vec<VerifyCheck> = Vec::with_capacity(9);

    // 1. ms1_entropy_match — always skipped (no entropy in watch-only).
    out.push(VerifyCheck {
        name: "ms1_entropy_match".into(),
        result: "skipped",
        detail: "watch-only mode: no entropy known to toolkit".into(),
    });

    // 2. mk1_decode.
    match mk_decode {
        Ok(_) => out.push(VerifyCheck {
            name: "mk1_decode".into(),
            result: "ok",
            detail: "decoded successfully".into(),
        }),
        Err(e) => out.push(VerifyCheck {
            name: "mk1_decode".into(),
            result: "fail",
            detail: e.clone(),
        }),
    }

    // 3. mk1_xpub_match.
    match mk_decode {
        Ok(card) => {
            let m = &card.xpub == supplied_xpub;
            out.push(VerifyCheck {
                name: "mk1_xpub_match".into(),
                result: if m { "ok" } else { "fail" },
                detail: if m {
                    "matches --xpub".into()
                } else {
                    "differs from --xpub".into()
                },
            });
        }
        Err(_) => out.push(VerifyCheck {
            name: "mk1_xpub_match".into(),
            result: "skipped",
            detail: "mk1 decode failed".into(),
        }),
    }

    // 4. mk1_fingerprint_match. SPEC §2.1.8: --privacy-preserving relaxes
    // this check to `skipped` (mk1 omits fingerprint by design).
    if privacy_preserving {
        out.push(VerifyCheck {
            name: "mk1_fingerprint_match".into(),
            result: "skipped",
            detail: "privacy-preserving mode; fingerprint suppressed".into(),
        });
    } else {
        match mk_decode {
            Ok(card) => {
                let m = card.origin_fingerprint == Some(supplied_fp);
                out.push(VerifyCheck {
                    name: "mk1_fingerprint_match".into(),
                    result: if m { "ok" } else { "fail" },
                    detail: if m {
                        "matches --master-fingerprint".into()
                    } else {
                        "differs from --master-fingerprint".into()
                    },
                });
            }
            Err(_) => out.push(VerifyCheck {
                name: "mk1_fingerprint_match".into(),
                result: "skipped",
                detail: "mk1 decode failed".into(),
            }),
        }
    }

    // 5. mk1_path_match — always skipped in watch-only (SPEC §2.2.2).
    out.push(VerifyCheck {
        name: "mk1_path_match".into(),
        result: "skipped",
        detail: "watch-only mode: path verification requires master seed (SPEC §2.2.2)".into(),
    });

    // 6. md1_decode.
    match md_decode {
        Ok(_) => out.push(VerifyCheck {
            name: "md1_decode".into(),
            result: "ok",
            detail: "decoded successfully".into(),
        }),
        Err(e) => out.push(VerifyCheck {
            name: "md1_decode".into(),
            result: "fail",
            detail: e.clone(),
        }),
    }

    // 7. md1_wallet_policy.
    match md_decode {
        Ok(desc) => {
            let wp = desc.is_wallet_policy();
            out.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                result: if wp { "ok" } else { "fail" },
                detail: if wp {
                    "wallet-policy mode confirmed".into()
                } else {
                    "descriptor is template-only (no pubkeys TLV)".into()
                },
            });
        }
        Err(_) => out.push(VerifyCheck {
            name: "md1_wallet_policy".into(),
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
                    name: "md1_xpub_match".into(),
                    result: if m { "ok" } else { "fail" },
                    detail: if m {
                        "65-byte xpub matches --xpub".into()
                    } else {
                        "md1 xpub differs from --xpub".into()
                    },
                });
            } else {
                out.push(VerifyCheck {
                    name: "md1_xpub_match".into(),
                    result: "skipped",
                    detail: "not in wallet-policy mode".into(),
                });
            }
        }
        Err(_) => out.push(VerifyCheck {
            name: "md1_xpub_match".into(),
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
                    name: "stub_linkage".into(),
                    result: if stub_match { "ok" } else { "fail" },
                    detail: if stub_match {
                        "policy_id_stub[0..4] matches mk1's stub[0]".into()
                    } else {
                        "stub linkage broken".into()
                    },
                });
            }
            Err(e) => out.push(VerifyCheck {
                name: "stub_linkage".into(),
                result: "fail",
                detail: format!("policy_id compute: {:?}", e),
            }),
        },
        _ => out.push(VerifyCheck {
            name: "stub_linkage".into(),
            result: "skipped",
            detail: "decode failed".into(),
        }),
    }

    out
}

/// Per-cosigner expected value for multisig verify cross-checks. Source varies
/// by mode (full = derived from --phrase; watch-only = supplied cosigner spec).
struct ExpectedCosigner {
    xpub: Xpub,
    master_fingerprint: bitcoin::bip32::Fingerprint,
    path: bitcoin::bip32::DerivationPath,
}

/// Multisig verify-bundle entry. Implements SPEC §2.2.1 multisig grouping +
/// SPEC §5.4 `3 + 6N` per-cosigner check enumeration.
///
/// Total checks emitted: `3 + 6N` for N cosigners, in this order:
///   ms1_entropy_match (1)
///   mk1_decode[i]            i ∈ 0..N (N)
///   mk1_xpub_match[i]        i ∈ 0..N (N)
///   mk1_fingerprint_match[i] i ∈ 0..N (N)
///   mk1_path_match[i]        i ∈ 0..N (N)
///   md1_decode (1)
///   md1_wallet_policy (1)
///   md1_xpub_match[i]        i ∈ 0..N (N)
///   stub_linkage[i]          i ∈ 0..N (N)
///
/// Cosigner association: per SPEC §2.2.1 step 6, decoded cards' xpubs are
/// matched against md1's `tlv.pubkeys` to determine each card's cosigner index.
/// In self-multisig (full mode) all N pubkeys are byte-identical and the mapping
/// is positional in decode order. Cards whose xpub is absent from `tlv.pubkeys`
/// fail their xpub_match check (other per-i checks remain `skipped`).
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
    let _ = xpub_arg;

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

    // 1. Resolve expected cosigners (mode-dependent).
    let expected: Vec<ExpectedCosigner> = if watch_only_multi {
        let specs: Vec<CosignerSpec> = if let Some(file) = &args.cosigners_file {
            parse_cosigners_file(file)?
        } else {
            let mut out = Vec::with_capacity(args.cosigner.len());
            for (i, s) in args.cosigner.iter().enumerate() {
                out.push(parse_cosigner_spec(s, i)?);
            }
            out
        };
        let path_family = args.multisig_path_family.unwrap_or_default();
        let script_type = args.template_unchecked().bip48_script_type().unwrap_or(0);
        let default_path_str =
            path_family.default_origin_path(args.network, args.account, script_type);
        let default_path = bitcoin::bip32::DerivationPath::from_str(&default_path_str)
            .map_err(|e| ToolkitError::BadInput(format!("default path parse: {}", e)))?;
        specs
            .into_iter()
            .map(|c| ExpectedCosigner {
                xpub: c.xpub,
                master_fingerprint: c.master_fingerprint,
                path: c.path.unwrap_or_else(|| default_path.clone()),
            })
            .collect()
    } else if phrase_arg.is_some() {
        check_no_concurrent_stdin(phrase_arg, args.passphrase.as_deref())?;
        let cosigner_count = args
            .cosigner_count
            .ok_or_else(|| ToolkitError::MultisigConfig {
                message: "--cosigner-count required for full-mode multisig verify".into(),
            })?;
        let path_family = args.multisig_path_family.unwrap_or_default();
        let language = args.language.unwrap_or_default();
        let passphrase = args.passphrase.clone().unwrap_or_default();
        let mnemonic = bip39::Mnemonic::parse_in(language.into(), phrase_arg.unwrap_or(""))
            .map_err(ToolkitError::Bip39)?;
        let seed = mnemonic.to_seed(&passphrase);
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let master = bitcoin::bip32::Xpriv::new_master(args.network.network_kind(), &seed)
            .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
        let master_fp = master.fingerprint(&secp);
        let script_type = args.template_unchecked().bip48_script_type().unwrap_or(0);
        let path_str = path_family.default_origin_path(args.network, args.account, script_type);
        let path = bitcoin::bip32::DerivationPath::from_str(&path_str)
            .map_err(|e| ToolkitError::BadInput(format!("path parse: {}", e)))?;
        let xpriv = master
            .derive_priv(&secp, &path)
            .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
        let xpub = Xpub::from_priv(&secp, &xpriv);
        // Self-multisig: all N expected (xpub, fp, path) are identical.
        (0..cosigner_count)
            .map(|_| ExpectedCosigner {
                xpub,
                master_fingerprint: master_fp,
                path: path.clone(),
            })
            .collect()
    } else {
        return Err(ToolkitError::BadInput(
            "multisig verify-bundle requires --phrase (full) or --cosigner/--cosigners-file (watch-only)".into(),
        ));
    };
    let n = expected.len();

    // 2. SPEC §2.2.1 step 1: group --mk1 chunks by chunk_set_id (Chunked) or
    //    per-string (SingleString).
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

    // 3. Decode each group; record per-group decode result.
    let mut decoded_opts: Vec<Option<mk_codec::KeyCard>> = Vec::with_capacity(groups.len());
    let mut decode_errors: Vec<Option<String>> = Vec::with_capacity(groups.len());
    for g in &groups {
        match mk_codec::decode(g) {
            Ok(c) => {
                decoded_opts.push(Some(c));
                decode_errors.push(None);
            }
            Err(e) => {
                decoded_opts.push(None);
                decode_errors.push(Some(format!("{:?}", e)));
            }
        }
    }

    // 4. md1 decode (needed early for cosigner association).
    let md1_strs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    let md_decoded = md_codec::chunk::reassemble(&md1_strs);

    // 5. Build cosigner-index → decoded-card mapping by xpub-against-tlv.pubkeys.
    //    SPEC §2.2.1 step 6. In self-multisig all N pubkeys are byte-identical;
    //    fall back to positional mapping (group i → cosigner i) when the lookup
    //    is ambiguous (multiple equal entries).
    let mut card_for_cosigner: Vec<Option<&mk_codec::KeyCard>> = vec![None; n];
    if let Ok(desc) = md_decoded.as_ref() {
        if let Some(pubkeys) = desc.tlv.pubkeys.as_ref() {
            // Build a quick lookup: 65-byte pubkey → cosigner index list.
            for (i, card_opt) in decoded_opts.iter().enumerate() {
                if let Some(card) = card_opt {
                    let want = crate::synthesize::xpub_to_65(&card.xpub);
                    // Prefer the i-th slot if it matches (covers self-multisig).
                    if let Some((_, b)) = pubkeys.get(i) {
                        if b == &want && card_for_cosigner[i].is_none() {
                            card_for_cosigner[i] = Some(card);
                            continue;
                        }
                    }
                    // Otherwise scan for the first unfilled matching slot.
                    if let Some((idx, _)) = pubkeys
                        .iter()
                        .find(|(slot, b)| b == &want && card_for_cosigner[*slot as usize].is_none())
                    {
                        card_for_cosigner[*idx as usize] = Some(card);
                    }
                }
            }
        }
    }
    // Fallback: when md1 decode failed OR pubkeys absent, use positional mapping.
    if md_decoded.is_err()
        || md_decoded
            .as_ref()
            .map(|d| d.tlv.pubkeys.is_none())
            .unwrap_or(false)
    {
        for (i, slot) in card_for_cosigner.iter_mut().enumerate().take(n) {
            if let Some(Some(c)) = decoded_opts.get(i) {
                *slot = Some(c);
            }
        }
    }

    // 6. ms1_entropy_match — full-multisig substantive; watch-only skipped.
    if watch_only_multi {
        checks.push(VerifyCheck {
            name: "ms1_entropy_match".into(),
            result: "skipped",
            detail: "watch-only multisig: no entropy known to toolkit".into(),
        });
    } else if let Some(ms1) = args.ms1.as_deref() {
        let language = args.language.unwrap_or_default();
        // BIP-39 entropy is passphrase-independent; passphrase affects only seed derivation.
        let mnemonic = bip39::Mnemonic::parse_in(language.into(), phrase_arg.unwrap_or(""))
            .map_err(ToolkitError::Bip39)?;
        let want_entropy = mnemonic.to_entropy();
        match ms_codec::decode(ms1) {
            Ok((_t, ms_codec::Payload::Entr(e))) => {
                let ok = e == want_entropy;
                checks.push(VerifyCheck {
                    name: "ms1_entropy_match".into(),
                    result: if ok { "ok" } else { "fail" },
                    detail: if ok {
                        "entropy bytes match".into()
                    } else {
                        format!("decoded {}-byte entropy != derived", e.len())
                    },
                });
            }
            Ok((_t, _)) => checks.push(VerifyCheck {
                name: "ms1_entropy_match".into(),
                result: "fail",
                detail: "decoded ms1 payload is not Entr".into(),
            }),
            Err(e) => checks.push(VerifyCheck {
                name: "ms1_entropy_match".into(),
                result: "fail",
                detail: format!("ms1 decode: {:?}", e),
            }),
        }
    } else {
        checks.push(VerifyCheck {
            name: "ms1_entropy_match".into(),
            result: "skipped",
            detail: "no --ms1 supplied".into(),
        });
    }

    // 7. Per-cosigner mk1_decode[i].
    for (i, slot) in card_for_cosigner.iter().enumerate().take(n) {
        match slot {
            Some(_) => checks.push(VerifyCheck {
                name: format!("mk1_decode[{}]", i),
                result: "ok",
                detail: format!("cosigner[{}] decoded", i),
            }),
            None => {
                // If we have a decode error at index i (positional), surface it.
                let detail = decode_errors
                    .get(i)
                    .and_then(|e| e.clone())
                    .unwrap_or_else(|| format!("no card associated with cosigner[{}]", i));
                checks.push(VerifyCheck {
                    name: format!("mk1_decode[{}]", i),
                    result: "fail",
                    detail,
                });
            }
        }
    }

    // 8. Per-cosigner mk1_xpub_match[i].
    for (i, slot) in card_for_cosigner.iter().enumerate().take(n) {
        match slot {
            Some(card) => {
                let m = card.xpub == expected[i].xpub;
                checks.push(VerifyCheck {
                    name: format!("mk1_xpub_match[{}]", i),
                    result: if m { "ok" } else { "fail" },
                    detail: if m {
                        format!("cosigner[{}] xpub matches", i)
                    } else {
                        format!("cosigner[{}] xpub does not match expected", i)
                    },
                });
            }
            None => checks.push(VerifyCheck {
                name: format!("mk1_xpub_match[{}]", i),
                result: "skipped",
                detail: format!("cosigner[{}] mk1 decode failed", i),
            }),
        }
    }

    // 9. Per-cosigner mk1_fingerprint_match[i] (skipped under --privacy-preserving).
    for (i, slot) in card_for_cosigner.iter().enumerate().take(n) {
        if args.privacy_preserving {
            checks.push(VerifyCheck {
                name: format!("mk1_fingerprint_match[{}]", i),
                result: "skipped",
                detail: "privacy-preserving mode; fingerprint suppressed".into(),
            });
            continue;
        }
        match slot {
            Some(card) => {
                let m = card.origin_fingerprint == Some(expected[i].master_fingerprint);
                checks.push(VerifyCheck {
                    name: format!("mk1_fingerprint_match[{}]", i),
                    result: if m { "ok" } else { "fail" },
                    detail: if m {
                        format!("cosigner[{}] fp matches", i)
                    } else {
                        format!("cosigner[{}] fp does not match expected", i)
                    },
                });
            }
            None => checks.push(VerifyCheck {
                name: format!("mk1_fingerprint_match[{}]", i),
                result: "skipped",
                detail: format!("cosigner[{}] mk1 decode failed", i),
            }),
        }
    }

    // 10. Per-cosigner mk1_path_match[i].
    //     Watch-only: substantive (compares card.origin_path against supplied/family path).
    //     Full: substantive (compares against derived path from --phrase + family).
    for (i, slot) in card_for_cosigner.iter().enumerate().take(n) {
        match slot {
            Some(card) => {
                let m = card.origin_path == expected[i].path;
                checks.push(VerifyCheck {
                    name: format!("mk1_path_match[{}]", i),
                    result: if m { "ok" } else { "fail" },
                    detail: if m {
                        format!("cosigner[{}] path matches", i)
                    } else {
                        format!(
                            "cosigner[{}] expected {}, got {}",
                            i, expected[i].path, card.origin_path
                        )
                    },
                });
            }
            None => checks.push(VerifyCheck {
                name: format!("mk1_path_match[{}]", i),
                result: "skipped",
                detail: format!("cosigner[{}] mk1 decode failed", i),
            }),
        }
    }

    // 11. md1_decode + md1_wallet_policy.
    let (md_ok, wp_ok) = match md_decoded.as_ref() {
        Ok(desc) => {
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                result: "ok",
                detail: "decoded successfully".into(),
            });
            let wp = desc.is_wallet_policy();
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                result: if wp { "ok" } else { "fail" },
                detail: if wp {
                    "wallet-policy mode confirmed".into()
                } else {
                    "descriptor is template-only (no pubkeys TLV)".into()
                },
            });
            (true, wp)
        }
        Err(e) => {
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                result: "fail",
                detail: format!("{:?}", e),
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                result: "skipped",
                detail: "md1 decode failed".into(),
            });
            (false, false)
        }
    };

    // 12. Per-cosigner md1_xpub_match[i] — compare expected xpub's 65-byte form
    //     against md1's tlv.pubkeys[i].
    for (i, exp) in expected.iter().enumerate().take(n) {
        if !md_ok || !wp_ok {
            checks.push(VerifyCheck {
                name: format!("md1_xpub_match[{}]", i),
                result: "skipped",
                detail: if !md_ok {
                    "md1 decode failed".into()
                } else {
                    "not in wallet-policy mode".into()
                },
            });
            continue;
        }
        let desc = md_decoded.as_ref().unwrap();
        let want = crate::synthesize::xpub_to_65(&exp.xpub);
        let m = desc
            .tlv
            .pubkeys
            .as_ref()
            .and_then(|v| v.iter().find(|(slot, _)| *slot as usize == i))
            .map(|(_, b)| b == &want)
            .unwrap_or(false);
        checks.push(VerifyCheck {
            name: format!("md1_xpub_match[{}]", i),
            result: if m { "ok" } else { "fail" },
            detail: if m {
                format!("cosigner[{}] md1 xpub matches expected", i)
            } else {
                format!("cosigner[{}] md1 xpub differs from expected", i)
            },
        });
    }

    // 13. Per-cosigner stub_linkage[i].
    //     Each card's policy_id_stubs list must contain the descriptor's
    //     computed policy_id stub. Failure → fail; missing decode → skipped.
    let descriptor_stub: Option<[u8; 4]> = md_decoded
        .as_ref()
        .ok()
        .and_then(|d| md_codec::compute_wallet_policy_id(d).ok())
        .map(|pid| {
            let mut s = [0u8; 4];
            s.copy_from_slice(&pid.as_bytes()[..4]);
            s
        });
    for (i, slot) in card_for_cosigner.iter().enumerate().take(n) {
        match (slot, descriptor_stub) {
            (Some(card), Some(want)) => {
                let m = card.policy_id_stubs.iter().any(|s| *s == want);
                checks.push(VerifyCheck {
                    name: format!("stub_linkage[{}]", i),
                    result: if m { "ok" } else { "fail" },
                    detail: if m {
                        format!("cosigner[{}] stub matches descriptor's policy_id", i)
                    } else {
                        format!("cosigner[{}] stub linkage broken", i)
                    },
                });
            }
            _ => checks.push(VerifyCheck {
                name: format!("stub_linkage[{}]", i),
                result: "skipped",
                detail: format!("cosigner[{}] decode failed", i),
            }),
        }
    }

    Ok(())
}

/// Phase D descriptor-mode verify: re-run the descriptor pipeline to build the
/// expected Bundle, then compare each card against the supplied --ms1/--mk1/--md1.
/// Emits the same VerifyBundleJson schema as template-mode verify (per SPEC §5.7
/// the check schema is structurally unchanged; only the source of truth differs).
fn descriptor_mode_verify_run<W: Write>(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    use crate::parse_descriptor::{
        bind_descriptor_keys, check_key_vector_distinctness, lex_placeholders, parse_descriptor,
        resolve_placeholders,
    };
    use crate::synthesize::synthesize_descriptor;

    let descriptor_str = match (&args.descriptor, &args.descriptor_file) {
        (Some(s), None) => s.clone(),
        (None, Some(p)) => std::fs::read_to_string(p)
            .map_err(|e| ToolkitError::DescriptorReparseFailed {
                detail: format!("--descriptor-file {}: {e}", p.display()),
            })?
            .trim_end()
            .to_string(),
        _ => unreachable!("clap conflicts_with rules out both"),
    };

    let occs =
        lex_placeholders(&descriptor_str).map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;
    let resolved =
        resolve_placeholders(&occs).map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;

    let phrase_owned: Option<String> = if args.phrase.is_some() {
        Some(read_phrase_input(args.phrase.as_deref(), stdin)?)
    } else {
        None
    };
    let passphrase = args.passphrase.clone().unwrap_or_default();
    let language = args.language.unwrap_or_default();

    let cosigner_specs: Vec<CosignerSpec> = if !args.cosigner.is_empty() {
        args.cosigner
            .iter()
            .enumerate()
            .map(|(i, s)| parse_cosigner_spec(s, i))
            .collect::<Result<Vec<_>, _>>()?
    } else if let Some(p) = args.cosigners_file.as_ref() {
        parse_cosigners_file(p)?
    } else {
        Vec::new()
    };

    let binding = bind_descriptor_keys(
        &resolved,
        args.network,
        phrase_owned.as_deref(),
        &passphrase,
        language,
        args.xpub.as_deref(),
        args.master_fingerprint.as_deref(),
        &cosigner_specs,
    )?;

    // SPEC §4.11.c symmetric verify-bundle enforcement: re-wrap to the verify-bundle
    // exit-4 variant so v0.2 self-multisig artifacts fail with the §4.11.c stderr.
    if let Err(ToolkitError::Bip388Distinctness { .. }) = check_key_vector_distinctness(&binding) {
        return Err(ToolkitError::Bip388VerifyDistinctness);
    }

    let descriptor = parse_descriptor(&descriptor_str, &binding.keys, &binding.fingerprints)
        .map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;
    let expected = synthesize_descriptor(
        &descriptor,
        &binding.cosigners,
        binding.entropy.as_deref(),
        args.privacy_preserving,
    )?;

    // Build the v0.3 §5.7 check ladder. For descriptor mode we use direct
    // bundle-cell comparison: ms1 string equality, mk1 string equality, md1
    // string equality. SPEC §5.7 conservatively emits a 3-element ladder for
    // descriptor mode (full 9 / 3+6N check schema + per-cell forensics land in
    // v0.4.1 per FOLLOWUPS `verify-bundle-9-3plus6n-forensics`).
    let mut checks: Vec<VerifyCheck> = Vec::new();

    // Check 1: ms1 entropy match (skipped if no --ms1 supplied or watch-only).
    // v0.4.1 H.1 shim: Bundle.ms1 is now Vec<String> (schema-4); descriptor
    // mode binds entropy at @0 only, so ms1[0] is the secret. The shim takes
    // the first non-empty element. Behavior diverges from v0.4.0 for the
    // impossible Some("") case (now routes to "skipped" rather than "fail";
    // synthesis never produced Some("") under v0.4.0). Phase J supersedes
    // this with the full per-slot ms1 check.
    if let Some(supplied_ms1) = args.ms1.as_deref() {
        match expected.ms1.first().map(|s| s.as_str()).filter(|s| !s.is_empty()) {
            Some(exp) if exp == supplied_ms1 => checks.push(VerifyCheck {
                name: "ms1_entropy_match".into(),
                result: "ok",
                detail: "ms1 byte-identical".into(),
            }),
            Some(_) => checks.push(VerifyCheck {
                name: "ms1_entropy_match".into(),
                result: "fail",
                detail: "expected ms1 bytes differ from supplied".into(),
            }),
            None => checks.push(VerifyCheck {
                name: "ms1_entropy_match".into(),
                result: "skipped",
                detail: "watch-only descriptor mode (no entropy expected)".into(),
            }),
        }
    } else {
        checks.push(VerifyCheck {
            name: "ms1_entropy_match".into(),
            result: "skipped",
            detail: "no --ms1 supplied".into(),
        });
    }

    // Check 2: mk1 byte-equality (per-card for multisig).
    let supplied_mk1 = &args.mk1;
    let expected_mk1: Vec<String> = match &expected.mk1 {
        crate::format::MkField::Single(v) => v.clone(),
        crate::format::MkField::Multi(per) => per.iter().flatten().cloned().collect(),
    };
    let mk1_match = expected_mk1.len() == supplied_mk1.len()
        && expected_mk1
            .iter()
            .zip(supplied_mk1.iter())
            .all(|(a, b)| a == b);
    checks.push(VerifyCheck {
        name: "mk1_match".into(),
        result: if mk1_match { "ok" } else { "fail" },
        detail: if mk1_match {
            "mk1 byte-identical".into()
        } else {
            format!(
                "expected {} chunks, got {}",
                expected_mk1.len(),
                supplied_mk1.len()
            )
        },
    });

    // Check 3: md1 byte-equality.
    let md1_match = expected.md1 == args.md1;
    checks.push(VerifyCheck {
        name: "md1_match".into(),
        result: if md1_match { "ok" } else { "fail" },
        detail: if md1_match {
            "md1 byte-identical".into()
        } else {
            format!(
                "expected {} chunks, got {}",
                expected.md1.len(),
                args.md1.len()
            )
        },
    });

    let any_fail = checks.iter().any(|c| c.result == "fail");
    let result_str = if any_fail { "mismatch" } else { "ok" };
    if args.json {
        let json = VerifyBundleJson {
            schema_version: "4",
            result: result_str,
            checks,
        };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        writeln!(stdout, "verify-bundle: {}", result_str).ok();
        for c in &checks {
            writeln!(stdout, "  - {} [{}]: {}", c.name, c.result, c.detail).ok();
        }
    }
    Ok(if any_fail { 4 } else { 0 })
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

        let checks = watch_only_checks(&xpub, fp, Ok(&card), Ok(&desc), false);
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

        let checks = watch_only_checks(&xpub, fp, Err(&err), Ok(&desc), false);
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

        let checks = watch_only_checks(&xpub, fp, Ok(&card), Err(&err), false);
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

        let checks = watch_only_checks(&wrong_xpub, fp, Ok(&card), Ok(&desc), false);
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

        let checks = watch_only_checks(&xpub, wrong_fp, Ok(&card), Ok(&desc), false);
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
            template: Some(CliTemplate::Bip84),
            descriptor: None,
            descriptor_file: None,
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
