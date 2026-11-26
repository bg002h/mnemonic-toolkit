//! `mnemonic bundle` subcommand.
//!
//! Realizes SPEC §2.1 (full + watch-only modes), §5.1 (multi-section
//! stdout), §5.2 (engraving card stderr), §5.3 (JSON schema).

use crate::display_grouping::render_grouped;
use crate::error::ToolkitError;
use crate::format::{BundleJson, CosignerEntry, MkField, MultisigInfo};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::MultisigPathFamily;
use crate::synthesize::Bundle;
use crate::template::CliTemplate;
use clap::Args;
use mnemonic_toolkit::mlock::pin_pages_for;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Args, Debug, Clone)]
pub struct BundleArgs {
    #[arg(long)]
    pub network: CliNetwork,

    /// Pre-built template name (single-sig or multisig). Mutually-required-one-of
    /// with --descriptor / --descriptor-file / --import-json (clap-level + runtime
    /// pre-check; v0.27.0 added the --import-json branch).
    #[arg(long, required_unless_present_any = ["descriptor", "descriptor_file", "import_json"])]
    pub template: Option<CliTemplate>,

    /// User-supplied BIP-388 descriptor (v0.3 §2.1.10). Mutually-required-one-of
    /// with --template / --descriptor-file. XOR with --descriptor-file (clap conflicts).
    /// (v0.49.0) Also accepts a BIP-388 wallet-policy JSON `{name,
    /// description_template, keys_info}` (auto-detected by a leading `{`),
    /// expanded to the concrete descriptor — the inverse of `export-wallet
    /// --format bip388`; origin-annotated `keys_info` required.
    #[arg(long, conflicts_with = "descriptor_file")]
    pub descriptor: Option<String>,

    /// User-supplied BIP-388 descriptor file (v0.3 §2.1.10). Single-line UTF-8;
    /// trailing newline tolerated. XOR with --descriptor (clap conflicts).
    #[arg(long = "descriptor-file")]
    pub descriptor_file: Option<PathBuf>,

    #[arg(long)]
    pub language: Option<CliLanguage>,

    /// BIP-39 mnemonic-extension passphrase ("25th word"). Empty
    /// (default) is the common case. `-` reads it from stdin (same as `--passphrase-stdin`); `@env:VAR`
    /// reads it from an environment variable. Any other value is taken
    /// literally and emits an argv-leakage advisory.
    #[arg(long)]
    pub passphrase: Option<String>,

    /// SPEC v0.9.0 §1 item 1 — read `--passphrase` from stdin (raw,
    /// preserving NULL bytes; strips a single trailing `\r?\n`).
    /// Mutually exclusive with `--passphrase` AND with any
    /// `--slot @N.<secret>=-` (single stdin per invocation).
    /// Mirrors `convert.rs:181` precedent.
    #[arg(long = "passphrase-stdin", conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// BIP-32 account index (default 0). Non-zero values produce md1 with
    /// PathDeclPaths::Divergent per SPEC §4.2.
    #[arg(long, default_value = "0", value_parser = crate::parse::parse_account_arg)]
    pub account: u32,

    /// Emit a single JSON object on stdout instead of the multi-line
    /// `ms1: ... / mk1: ... / md1: ...` text form.
    #[arg(long)]
    pub json: bool,

    /// Suppress the human-readable engraving-card panel on stderr.
    /// The stdout `ms1` / `mk1` / `md1` output is unchanged. Use for
    /// piping into other tooling.
    #[arg(long = "no-engraving-card")]
    pub no_engraving_card: bool,

    /// Insert a separator every N characters in the emitted ms1/mk1/md1 strings
    /// (0 = unbroken). SPEC §3. Display only; --json + verify-bundle stay unbroken.
    #[arg(long, default_value_t = 0)]
    pub group_size: u16,

    /// Separator: `space` (keyword) or the literal " ". Whitespace only --
    /// `hyphen` and `comma` were retired by
    /// SPEC_constellation_cli_uniformity 6c. SPEC §5.
    #[arg(long, default_value = "space", value_parser = crate::display_grouping::parse_separator)]
    pub separator: char,

    /// v0.2 multisig path family (default: bip87).
    #[arg(long = "multisig-path-family", value_enum)]
    pub multisig_path_family: Option<MultisigPathFamily>,

    /// v0.2 privacy mode: suppress master fingerprint from the
    /// emitted mk1 cards + engraving card.
    #[arg(long, default_value = "false")]
    pub privacy_preserving: bool,

    /// v0.2 self-check: after emission, re-parse the emitted bundle
    /// and verify it round-trips to the same xpubs + policy.
    #[arg(long, default_value = "false")]
    pub self_check: bool,

    /// v0.2 multisig threshold K (1 ≤ K ≤ N ≤ 16). Required for
    /// multisig templates (`wsh-*`, `sh-wsh-*`, `tr-*`); refused
    /// under single-sig templates.
    #[arg(long)]
    pub threshold: Option<u8>,

    /// v0.4 unified slot input. Repeating flag — one occurrence per
    /// (slot, subkey) tuple.
    ///
    /// Grammar: `@N.<subkey>=<value>`, where N is the slot index
    /// (u8) and `<subkey>` is one of:
    ///   phrase       BIP-39 mnemonic (secret)
    ///   seedqr       48 or 96 ASCII digits encoding a BIP-39 phrase
    ///                (secret; decoded inline via seedqr::decode)
    ///   entropy      raw entropy hex (secret)
    ///   ms1          BIP-93 codex32 secret (entropy or mnemonic;
    ///                language-preserving) (secret)
    ///   xpub         BIP-32 extended public key
    ///   master_xpub  depth-0 master xpub (Coldcard singlesig only;
    ///                see SPEC_export_wallet §5.1)
    ///   fingerprint  4-byte master fingerprint (hex)
    ///   path         BIP-32 derivation path
    ///   wif          Wallet Import Format private key (secret)
    ///   xprv         BIP-32 extended private key (secret)
    ///
    /// `<value>` is the subkey's text form, or `-` to read from
    /// stdin. Single-sig templates expect `@0` only; multisig
    /// templates expect `@0..@N-1`.
    #[arg(
        long = "slot",
        action = clap::ArgAction::Append,
        value_parser = crate::slot_input::parse_slot_input,
        verbatim_doc_comment,
    )]
    pub slot: Vec<crate::slot_input::SlotInput>,

    /// v0.27.0 — synthesize a bundle from an `import-wallet --json`
    /// envelope rather than from `--template` / `--descriptor`. Accepts a
    /// file path or `-` to read the envelope from stdin. The envelope's
    /// `bundle.descriptor` carries the source-of-truth descriptor; the
    /// envelope's `bundle.mk1` chunks decode to per-cosigner xpubs +
    /// fingerprints + paths. Mutually exclusive with `--template`,
    /// `--descriptor`, `--descriptor-file`.
    ///
    /// Seed overlay (`--slot @N.phrase=`) continues to apply to
    /// cosigners where the envelope's `ms1[N] == ""` sentinel (watch-
    /// only). (`--ms1` is import-wallet's input surface, not bundle's;
    /// envelope-derived entropy arrives pre-attached as the envelope's
    /// `ms1[N] != ""`.) Supplying `--slot @N.phrase=` for a cosigner
    /// with non-empty envelope `ms1[N]` is `BadInput` (conflict).
    #[arg(
        long = "import-json",
        value_name = "FILE|-",
        conflicts_with_all = ["template", "descriptor", "descriptor_file"],
    )]
    pub import_json: Option<String>,

    /// #28 phase 1 — what the md1 card encodes. `policy` (default) = the FULL
    /// keyed wallet-policy md1 (pre-#28 behavior, identifies THIS wallet).
    /// `template` = a KEYLESS, fingerprint-stripped, canonical-origin-elided
    /// single-sig template md1 — a backup of the wallet TYPE, byte-identical
    /// across all users of that type ("one engraving for thousands"). The
    /// account/origin is supplied at restore (`--account`/`--origin`); the
    /// specific wallet is identified out-of-band by the `WalletPolicyId`
    /// printed on stderr. REQUIRES a canonical single-sig shape (bip44/84/86);
    /// multisig / non-canonical / bip49-nested-segwit are refused (use
    /// `--md1-form=policy`).
    #[arg(long = "md1-form", value_enum, default_value_t = crate::synthesize::Md1Form::Policy)]
    pub md1_form: crate::synthesize::Md1Form,

    /// v0.27.0 — pick a specific entry from a multi-entry envelope
    /// array (e.g., Bitcoin Core `listdescriptors` blob with multiple
    /// descriptors). Required when the envelope has > 1 entry; optional
    /// (and rejected if supplied) for single-entry envelopes? — actually
    /// SPEC §3.6 leaves single-entry usage unrestricted; passing an
    /// index for a single-entry envelope just requires `0`. Out-of-
    /// range → exit 2.
    #[arg(long = "import-json-index", value_name = "N", requires = "import_json")]
    pub import_json_index: Option<usize>,
}

/// SPEC §6.6 byte-exact mode-violation strings. Pinned for integration tests.
pub mod mode_text {
    pub const THRESHOLD_WITHOUT_MULTISIG: &str = "--threshold is meaningful only with a multisig --template; single-sig templates ignore threshold.";
    pub const PATH_FAMILY_WITHOUT_MULTISIG: &str =
        "--multisig-path-family is meaningful only with a multisig --template.";

    // v0.3 NEW rows (SPEC §6.9). Byte-exact.
    pub const DESCRIPTOR_AND_TEMPLATE: &str = "--descriptor and --template are mutually exclusive; pick descriptor passthrough or template, not both.";
    pub const DESCRIPTOR_AND_DESCRIPTOR_FILE: &str = "--descriptor and --descriptor-file are mutually exclusive; supply the descriptor inline or via file, not both.";
    pub const DESCRIPTOR_WITH_THRESHOLD: &str = "--threshold is meaningful only with a multisig --template; descriptor mode encodes K directly.";
    pub const DESCRIPTOR_WITH_PATH_FAMILY: &str = "--multisig-path-family is meaningful only with --template; descriptor mode encodes paths directly via @i/path syntax.";
    pub const DESCRIPTOR_WITH_NONZERO_ACCOUNT: &str = "--account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path.";
}

/// Exit status of a successful `bundle` whose cards record an origin path the
/// user did not supply (Brian's ruling of 2026-11-26,
/// `tk-bh1125-decision-descriptor-vs-template-multisig`): the cards are
/// written in full, and 4 says "verify before trusting".
pub const EXIT_CHOSEN_ORIGIN: u8 = 4;

/// Returns the exit status: 0, or [`EXIT_CHOSEN_ORIGIN`].
pub fn run<W: Write, E: Write>(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // SPEC v0.9.0 §1 item 1 — argv-leakage closure. Run BEFORE any
    // dispatch logic so the advisory fires uniformly regardless of
    // downstream success/error. v0.26.0 §I1 fold: emit BEFORE
    // `@env:` sentinel resolution so the advisory sees the literal
    // sentinel string and can skip values that already used the
    // env-var leak-mitigation channel (sentinel-bearing flags do NOT
    // get the warning — the user already opted out of argv exposure).
    emit_secret_in_argv_advisories(args, stderr);
    // F-687: `--passphrase -` reads stdin like `--passphrase-stdin`. Decided
    // from the value as written, before `@env:` resolution below.
    let pp_reads_stdin =
        crate::passphrase_input::reads_stdin(args.passphrase.as_deref(), args.passphrase_stdin);
    // F-687 fold 1 (M1, M4): `--import-json -`, and a `--descriptor-file` /
    // `--import-json` path that IS stdin, read stdin too. (Slot stdin is
    // refused in `apply_stdin_substitutions`.)
    crate::passphrase_input::refuse_second_stdin(&[
        (
            pp_reads_stdin,
            crate::passphrase_input::stdin_spelling(args.passphrase_stdin),
        ),
        (
            args.import_json.as_deref().is_some_and(|v| {
                v == "-" || crate::passphrase_input::path_is_stdin(std::path::Path::new(v))
            }),
            "--import-json -",
        ),
        (
            args.descriptor_file
                .as_deref()
                .is_some_and(crate::passphrase_input::path_is_stdin),
            "--descriptor-file /dev/stdin",
        ),
    ])?;

    // v0.26.0 §3 — resolve `@env:<VAR>` sentinels before downstream
    // consumption. Skipped when no sentinel is present to avoid an
    // unnecessary `args.clone()`.
    // bh-tk-I5 (R-1): which slots read stdin is decided from the values as
    // written, before `@env:` resolution (a variable whose value is `-` is that
    // literal value, not stdin).
    let slot_stdin_idxs = crate::slot_input::stdin_slot_positions(&args.slot);
    let env_resolved_owned;
    let args: &BundleArgs = if needs_env_sentinel_resolution(args) {
        env_resolved_owned = resolve_env_sentinels(args)?;
        &env_resolved_owned
    } else {
        args
    };
    let synthetic_args;
    let args: &BundleArgs = if needs_stdin_substitution(pp_reads_stdin, &slot_stdin_idxs) {
        synthetic_args = apply_stdin_substitutions(args, stdin, pp_reads_stdin, &slot_stdin_idxs)?;
        &synthetic_args
    } else {
        args
    };

    // Cycle B Phase 3a Site 1 — pin argv-string secret heap pages for the
    // remainder of the handler scope. Lands AFTER apply_stdin_substitutions
    // so the pin covers the post-substitution buffers (per SPEC §4 P3a).
    let _pin_passphrase = args
        .passphrase
        .as_ref()
        .map(|p| mnemonic_toolkit::mlock::pin_pages_for(p.as_bytes()));
    let _pin_slot_values: Vec<_> = args
        .slot
        .iter()
        .map(|s| mnemonic_toolkit::mlock::pin_pages_for(s.value.as_bytes()))
        .collect();

    // v0.27.0 — `--import-json` dispatch short-circuits before template /
    // descriptor mode pre-checks. The envelope carries everything needed
    // for a fresh `synthesize_descriptor` pass; the only relevant user
    // flags downstream are `--slot @N.phrase=` (seed overlay),
    // `--privacy-preserving`, `--json`, `--self-check`, `--no-engraving-card`.
    // Mutex with --template / --descriptor / --descriptor-file is enforced
    // by clap (`conflicts_with_all` on the `--import-json` arg).
    if args.import_json.is_some() {
        return bundle_run_from_import_json(args, stdin, stdout, stderr).map(|()| 0);
    }

    let descriptor_mode = args.descriptor.is_some() || args.descriptor_file.is_some();
    let multisig_template = args
        .template
        .as_ref()
        .map(|t| t.is_multisig())
        .unwrap_or(false);

    // SPEC §6.6 / §6.9 retained mode-violation pre-checks.
    if descriptor_mode && args.template.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--template",
            message: mode_text::DESCRIPTOR_AND_TEMPLATE,
        });
    }
    if args.descriptor.is_some() && args.descriptor_file.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--descriptor-file",
            message: mode_text::DESCRIPTOR_AND_DESCRIPTOR_FILE,
        });
    }
    if descriptor_mode && args.threshold.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--threshold",
            message: mode_text::DESCRIPTOR_WITH_THRESHOLD,
        });
    }
    if descriptor_mode && args.multisig_path_family.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--multisig-path-family",
            message: mode_text::DESCRIPTOR_WITH_PATH_FAMILY,
        });
    }
    // v0.19.0 SPEC §4.12.g — `DESCRIPTOR_WITH_NONZERO_ACCOUNT` guard is
    // canonicity-gated. The check moved into `bundle_run_unified_descriptor`
    // post-parse so canonical descriptors still refuse `--account != 0`
    // (canonical_origin's per-shape default supplies the path; user-supplied
    // account is redundant), while non-canonical descriptors consume
    // `--account N` for §4.12.b default-path inference. Pre-bundle_run_unified
    // site retains the other descriptor-mode guards (--template, --threshold,
    // --multisig-path-family) which apply uniformly regardless of canonicity.
    if args.threshold.is_some() && !multisig_template && !descriptor_mode {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--threshold",
            message: mode_text::THRESHOLD_WITHOUT_MULTISIG,
        });
    }
    if args.multisig_path_family.is_some() && !multisig_template && !descriptor_mode {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--multisig-path-family",
            message: mode_text::PATH_FAMILY_WITHOUT_MULTISIG,
        });
    }

    if descriptor_mode {
        // Read the descriptor body here (the read inside bundle_run_unified_descriptor
        // is off the Concrete early-fork path).
        let body = match (&args.descriptor, &args.descriptor_file) {
            (Some(s), None) => s.clone(),
            (None, Some(p)) => std::fs::read_to_string(p)
                .map_err(|e| {
                    ToolkitError::DescriptorParse(format!("--descriptor-file {}: {e}", p.display()))
                })?
                .trim_end()
                .to_string(),
            _ => unreachable!("DESCRIPTOR_AND_DESCRIPTOR_FILE guard above rules out both"),
        };
        // md1 cards are engraved descriptor-mnemonic CARDS, not raw descriptors —
        // refuse with a surface-pointing message instead of letting an md1 card
        // fall into classify_descriptor_form's misleading "keyless script" arm
        // (`export-wallet-bundle-descriptor-md1-clearer-error`). Covers both
        // --descriptor and --descriptor-file (body already merged above).
        crate::wallet_import::pipeline::reject_md1_card(&body, "bundle --descriptor")?;
        // BIP-388 wallet-policy JSON intake: expand to a concrete descriptor
        // BEFORE classify (a raw policy JSON trips classify's (true,true)
        // mixed-form error — its description_template matches the @N probe AND
        // its keys_info matches the key_regex). Expanded body → Concrete → the
        // watch-only path (a policy carries no secrets). Requires origin-
        // annotated keys_info (the slot-resolver needs [fp/path]xpub).
        let body = if crate::wallet_import::pipeline::is_bip388_policy_shape(&body) {
            crate::wallet_import::pipeline::expand_bip388_policy(&body)?
        } else {
            body
        };
        use crate::wallet_import::pipeline::{classify_descriptor_form, DescriptorForm};
        if classify_descriptor_form(&body)? == DescriptorForm::Concrete {
            return bundle_run_concrete_descriptor(args, body, stdout, stderr).map(|()| 0);
        }
        // AtN: fall through to bundle_run_unified (re-reads the file as today).
    }

    bundle_run_unified(args, stdin, stdout, stderr)
}
// ============================================================================
// v0.4.1 Phase H.5: unified --slot-driven dispatch.
// ============================================================================

use crate::bundle_unified::{detect_bundle_mode, BundleMode};
use crate::slot_input::{SlotInput, SlotSubkey};
use crate::synthesize::{synthesize_unified, ResolvedSlot};
use bitcoin::bip32::{DerivationPath, Fingerprint};
use bitcoin::secp256k1::Secp256k1;

/// v0.5.1 entry point — `--slot`-driven dispatch is the sole shape.
/// Routes through SPEC §6.6.b validate_slot_set + §3.3 detect_bundle_mode +
/// `synthesize_unified`.
fn bundle_run_unified<W: Write, E: Write>(
    args: &BundleArgs,
    _stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    use crate::bundle_unified::{pre_check_template_n, pre_check_threshold};
    use crate::slot_input::validate_slot_set;

    let slots = args.slot.clone();
    validate_slot_set(&slots)?;

    let mode = detect_bundle_mode(&slots)?;
    let n = slots
        .iter()
        .map(|s| s.index as usize)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);

    // SPEC §6.6 row 9, 9.5, 10, 11.
    let template_str = args.template.map(|t| t.human_name());
    let multisig_template =
        template_str.filter(|_| args.template.map(|t| t.is_multisig()).unwrap_or(false));
    pre_check_threshold(args.threshold, n, multisig_template)?;
    if let Some(t) = args.template {
        pre_check_template_n(t.human_name(), t.is_multisig(), n)?;
    } else if args.descriptor.is_none() && args.descriptor_file.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "unified-slot",
            flag: "--template / --descriptor",
            message: "missing --template or --descriptor",
        });
    }

    // v0.4.2 Phase L: descriptor mode under unified --slot dispatch.
    if args.descriptor.is_some() || args.descriptor_file.is_some() {
        return bundle_run_unified_descriptor(args, &slots, mode, stdout, stderr);
    }

    let template = args
        .template
        .ok_or_else(|| ToolkitError::BadInput("--template required for --slot dispatch".into()))?;

    // Bug hunt 2026-11-25 X4: a `@N.wif=` slot is one fixed key, not an HD key.
    // Minting it wrote the WIF's public key as a depth-0 "xpub" with an all-zero
    // chain code under the template's ranged `/<0;1>/*` use-site, so every
    // address on the cards (and from `restore` / `md address`) is a CHILD of
    // that fake chain code — not the WIF key's own address, and invisible to any
    // wallet that imports the WIF. Every template is ranged, so refuse at mint.
    // (`verify-bundle` still reads an existing WIF bundle unchanged; descriptor
    // mode already refused `wif`.)
    if let Some(w) = slots.iter().find(|s| s.subkey == SlotSubkey::Wif) {
        let idx = w.index;
        return Err(ToolkitError::BadInput(format!(
            "--slot @{idx}.wif is refused: a WIF is a single private key with no chain code, but \
             every --template derives ranged child addresses (/<0;1>/*), so the cards would \
             describe addresses that are children of an all-zero chain code, not the WIF key's \
             own address, and that no wallet importing this WIF would ever watch. Back up the \
             seed (phrase / entropy / ms1) instead, or sweep the WIF's funds into a seed wallet."
        )));
    }

    // Fable review I1 (bug hunt 2026-11-25): a typed `@N.path=m` (the master
    // node) would mint a keyed card with an EMPTY origin, which the wire reads
    // as "canonical origin, elided" for the template shapes, and which a
    // non-canonical card cannot carry at all. Refused at mint only, with the
    // descriptor-mode text; `resolve_slots` is shared with verify-bundle and
    // export-wallet, which keep reading existing input unchanged.
    for s in slots.iter().filter(|s| s.subkey == SlotSubkey::Path) {
        let empty = DerivationPath::from_str(&s.value)
            .map(|p| p.is_empty())
            .unwrap_or(false);
        if empty {
            let i = s.index;
            return Err(ToolkitError::SlotInputViolation {
                kind: "conflict",
                message: format!(
                    "slot @{i}: `--slot @{i}.path=m` (the master node) cannot be recorded on a \
                     card for this template: an empty origin on the card means the shape's \
                     canonical origin. Give the key's real origin path."
                ),
            });
        }
    }

    // FOLLOWUP `multisig-tr-bip48-script-type-3-policy` (bless + warn): taproot
    // multisig under --multisig-path-family bip48 derives at the non-standard
    // m/48'/.../3'. Honor it, but advise on stderr at creation time.
    if let Some(w) = template
        .bip48_nonstandard_script_type_warning(args.multisig_path_family.unwrap_or_default())
    {
        let _ = writeln!(stderr, "{w}");
    }

    // Resolve slots into ResolvedSlot vec.
    let (resolved, slip0132_signals) = resolve_slots(
        &slots,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
        args.multisig_path_family.unwrap_or_default(),
    )?;

    // SPEC §4.11.b BIP-388 distinct-key check on resolved slots.
    check_resolved_slots_distinctness(&resolved)?;

    let threshold = args.threshold.unwrap_or(n as u8);

    // Mode-specific synthesis.
    let bundle = match mode {
        BundleMode::SingleSigFull
        | BundleMode::SingleSigWatchOnly
        | BundleMode::MultisigMultiSource
        | BundleMode::MultisigWatchOnly
        | BundleMode::MultisigHybrid => synthesize_unified(
            &resolved,
            template,
            threshold,
            args.network,
            args.privacy_preserving,
            args.language.unwrap_or_default().into(),
            args.md1_form,
        )?,
    };

    // Brian's ruling of 2026-11-26: a multisig template with no
    // --multisig-path-family records the BIP-87 default, a path the user did
    // not choose. Say so, print the command that chooses it, and exit 4. A
    // single-sig template names its own path (bip84 is m/84'/...), and a slot
    // with a typed `@N.path=` takes no default.
    let chosen_origin = if template.is_multisig() && args.multisig_path_family.is_none() {
        let defaulted: Vec<u8> = (0..n as u8)
            .filter(|i| {
                !slots
                    .iter()
                    .any(|s| s.index == *i && s.subkey == SlotSubkey::Path)
            })
            .collect();
        emit_template_family_warning(stderr, &defaulted, template, args.network, args.account)?
    } else {
        false
    };

    // Emit (reuse legacy text/JSON renderer; engraving card omitted for now;
    // unified card lands in Phase I).
    emit_unified(
        args,
        &bundle,
        &resolved,
        mode,
        &slip0132_signals,
        stdout,
        stderr,
    )?;

    if args.self_check {
        let expected_entropy: Vec<Option<&[u8]>> = resolved
            .iter()
            .map(|r| r.entropy.as_deref().map(|v| v.as_slice()))
            .collect();
        self_check_bundle(&bundle, args, &expected_entropy)?;
    }
    Ok(if chosen_origin { EXIT_CHOSEN_ORIGIN } else { 0 })
}

/// v0.4.1 H.5 BIP-388 distinct-key check on ResolvedSlot vector. Mirrors
/// `check_key_vector_distinctness` for the unified path; comparison key is
/// `(xpub.to_string(), path)` on the TYPED `DerivationPath` (v0.5 §4.11.b
/// deliberate-reversal: `h`/`'`-notation folds, so `48h/..` and `48'/..`
/// collide — converges with the descriptor-mode twin per
/// `SPEC_path_raw_bracketed_bare_unification.md` Amendment A2).
fn check_resolved_slots_distinctness(slots: &[ResolvedSlot]) -> Result<(), ToolkitError> {
    for i in 0..slots.len() {
        for j in (i + 1)..slots.len() {
            if slots[i].xpub.to_string() == slots[j].xpub.to_string()
                && slots[i].path == slots[j].path
            {
                return Err(ToolkitError::Bip388Distinctness {
                    i: i as u8,
                    j: j as u8,
                });
            }
        }
    }
    Ok(())
}

/// Bug hunt 2026-11-25 toolkit#5: refuse a DEFAULTED origin path that the
/// xpub's own BIP-32 header contradicts (depth differs from the path length, or
/// the child number differs from the path's last component). Only called when
/// no `@N.path=` was given; a typed path is not checked here. The refusal
/// suggests no example path (FOLLOWUPS `tk-bh1125-r3-m1-refusal-example-path`):
/// a typed path is not checked against the header, so a copied example would
/// mint a card with a wrong origin.
pub(crate) fn check_defaulted_xpub_origin(
    idx: u8,
    xpub: &bitcoin::bip32::Xpub,
    default_path: &DerivationPath,
) -> Result<(), ToolkitError> {
    let comps: Vec<bitcoin::bip32::ChildNumber> = default_path.into_iter().copied().collect();
    let depth_ok = xpub.depth as usize == comps.len();
    let child_ok = comps.last() == Some(&xpub.child_number);
    if depth_ok && child_ok {
        return Ok(());
    }
    Err(ToolkitError::SlotInputViolation {
        kind: "conflict",
        message: format!(
            "slot @{idx}: no `@{idx}.path=` was given, and the default origin m/{default_path} \
             does not match the xpub's own header (depth {}, child number {}); the key does not \
             live at the default path. Supply the key's real origin with \
             `--slot @{idx}.path=m/...`.",
            xpub.depth, xpub.child_number
        ),
    })
}

/// Resolve slot inputs into ResolvedSlot vec.
/// Supported subkey shapes:
/// - {phrase} → BIP-39 derive entropy + seed + master_xpriv → xpub at template
///   path + master_fingerprint + path.
/// - {xpub, fingerprint, path} → parse all three directly.
/// - {entropy} / {wif} / {xprv-rejected} per slot_input.rs validity matrix.
///
/// Returns `(resolved, slip0132_signals)`. The signals vec carries `(slot_idx,
/// variant)` pairs in slot-index ascending order (BTreeMap iteration) for any
/// `xpub` slots whose input was a SLIP-0132 prefix variant; `emit_unified`
/// uses them to emit the SPEC §5.5.a info-line.
#[allow(clippy::type_complexity)]
pub(crate) fn resolve_slots(
    slots: &[SlotInput],
    template: CliTemplate,
    network: CliNetwork,
    account: u32,
    language: Option<CliLanguage>,
    passphrase: Option<&str>,
    multisig_path_family: MultisigPathFamily,
) -> Result<(Vec<ResolvedSlot>, Vec<(u8, &'static str)>), ToolkitError> {
    use std::collections::BTreeMap;
    let mut by_index: BTreeMap<u8, Vec<&SlotInput>> = BTreeMap::new();
    for s in slots {
        by_index.entry(s.index).or_default().push(s);
    }
    let by_index_len = by_index.len();
    let secp = Secp256k1::new();
    let mut out: Vec<ResolvedSlot> = Vec::with_capacity(by_index_len);
    // SPEC v0.6.2 §5.5.a — accumulate SLIP-0132 input-normalization signals
    // for the emit_unified info-line. BTreeMap iteration is slot-index
    // ascending → no re-sort needed downstream.
    let mut slip0132_signals: Vec<(u8, &'static str)> = Vec::new();
    // F3 fix: for multisig templates the per-cosigner derivation path comes
    // from `--multisig-path-family` (BIP-87 default → m/87'/coin'/account';
    // BIP-48 → m/48'/coin'/account'/script') — NOT `template.derivation_path`,
    // which returns the BIP-87 fallback for ALL multisig templates and so
    // silently ignored the flag for seed/entropy slots. For BIP-87 the path is
    // identical, so every pre-fix default-family bundle is byte-unchanged. For
    // single-sig this is None and the template path is used as before.
    let multisig_acct_path: Option<DerivationPath> = if template.is_multisig() {
        let script_type = template.bip48_script_type().unwrap_or(0);
        let p = multisig_path_family.default_origin_path(network, account, script_type);
        Some(DerivationPath::from_str(&p).expect("family origin paths are well-formed"))
    } else {
        None
    };
    for (idx, slot_inputs) in by_index {
        let subkeys: std::collections::BTreeSet<SlotSubkey> =
            slot_inputs.iter().map(|s| s.subkey).collect();
        // Bug hunt 2026-11-25 toolkit#4: the slot grammar admits `@N.path=` /
        // `@N.fingerprint=` beside a seed (`phrase` / `seedqr` / `ms1`) for
        // non-canonical DESCRIPTOR mode only. Template mode derives the seed at
        // the `--template` / `--account` / `--multisig-path-family` path and has
        // nowhere to put a typed path or fingerprint, so it used to drop them
        // silently (a request for account 5 produced account-0 cards). Refuse
        // instead: a path the operator supplies must reach the artifact or be
        // refused, never be silently dropped.
        let seed_bearing = [
            SlotSubkey::Phrase,
            SlotSubkey::Seedqr,
            SlotSubkey::Ms1,
            SlotSubkey::Entropy,
        ]
        .iter()
        .any(|k| subkeys.contains(k));
        if seed_bearing
            && (subkeys.contains(&SlotSubkey::Path) || subkeys.contains(&SlotSubkey::Fingerprint))
        {
            return Err(ToolkitError::SlotInputViolation {
                kind: "conflict",
                message: format!(
                    "slot @{idx}: --template mode ignores `@{idx}.path=` / `@{idx}.fingerprint=` \
                     beside a seed (the key is derived at the template's path and the seed's own \
                     master fingerprint); remove them, and choose the account with --account \
                     (and --multisig-path-family for multisig). A per-key path on a seed slot is \
                     only accepted with a non-canonical --descriptor."
                ),
            });
        }
        if subkeys.contains(&SlotSubkey::Phrase) || subkeys.contains(&SlotSubkey::Seedqr) {
            // v0.31.3 — Seedqr digest = secret-bearing materialization of
            // a BIP-39 phrase. Decode at slot-emit time then dispatch
            // identically to the Phrase path. `is_legal_set` refuses
            // co-occurrence of Phrase + Seedqr in the same slot, so the
            // owned-String binding is unambiguous.
            let decoded_phrase: String;
            let phrase: &str = if subkeys.contains(&SlotSubkey::Seedqr) {
                let digits = slot_inputs
                    .iter()
                    .find(|s| s.subkey == SlotSubkey::Seedqr)
                    .map(|s| &*s.value)
                    .expect("contains() asserts presence");
                decoded_phrase = mnemonic_toolkit::seedqr::decode(digits).map_err(|e| {
                    crate::cmd::seedqr::map_seedqr_error(e, &format!("slot @{idx} decode"))
                })?;
                &decoded_phrase
            } else {
                slot_inputs
                    .iter()
                    .find(|s| s.subkey == SlotSubkey::Phrase)
                    .map(|s| &*s.value)
                    .expect("contains() asserts presence")
            };
            let lang = language.unwrap_or_default();
            let pass = passphrase.unwrap_or("");
            let acc = match &multisig_acct_path {
                Some(p) => crate::derive::derive_full_at_path(phrase, pass, lang, network, p)?,
                None => crate::derive::derive_full(phrase, pass, lang, network, template, account)?,
            };
            // v0.10.1: DerivedAccount.entropy is Zeroizing<Vec<u8>>; the
            // hand-rolled impl Drop is gone. `into_parts` remains the
            // canonical consuming-move path (returns bare Vec<u8> per the
            // caller-wrap contract — re-wrap below at the ResolvedSlot
            // ctor boundary).
            let (entropy, fingerprint, xpub, path) = acc.into_parts();
            let entropy_pin = Some(Rc::new(pin_pages_for(&entropy[..])));
            out.push(ResolvedSlot {
                xpub,
                fingerprint,
                path,
                // v0.10.1: ResolvedSlot.entropy migrated to Option<Zeroizing<Vec<u8>>>.
                entropy: Some(zeroize::Zeroizing::new(entropy)),
                master_xpub: None,
                language: None,
                _entropy_pin: entropy_pin,
            });
        } else if subkeys.contains(&SlotSubkey::Xpub) {
            let xpub_str = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Xpub)
                .map(|s| &*s.value)
                .expect("contains() asserts presence");
            // SPEC v0.6.1 §11 — accept SLIP-0132 prefix variants on input.
            let (xpub_str, input_variant) = crate::slip0132::normalize_xpub_prefix(xpub_str)?;
            if let Some(v) = input_variant {
                slip0132_signals.push((idx, v));
            }
            let xpub = bitcoin::bip32::Xpub::from_str(&xpub_str)
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            let fp_str = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Fingerprint)
                .map(|s| &*s.value);
            let fingerprint = match fp_str {
                Some(s) => Fingerprint::from_str(s).map_err(|e| {
                    ToolkitError::BadInput(format!("--slot @{idx}.fingerprint parse: {e}"))
                })?,
                None => Fingerprint::default(),
            };
            let path = match slot_inputs.iter().find(|s| s.subkey == SlotSubkey::Path) {
                Some(p) => DerivationPath::from_str(&p.value).map_err(|e| {
                    ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}"))
                })?,
                None => {
                    // v0.5.1: Path absent → fall back to the default origin path
                    // so xpub-only watch-only slots verify against fixtures built
                    // at the same path. F3 fix: for multisig templates this is the
                    // --multisig-path-family path, not the BIP-87-only template
                    // fallback.
                    let default_path = match &multisig_acct_path {
                        Some(p) => p.clone(),
                        None => template.derivation_path(network, account),
                    };
                    // Bug hunt 2026-11-25 toolkit#5: the default is a GUESS, so
                    // refuse it when the xpub's own header says the key does
                    // not live there (depth or child number disagree) — e.g. a
                    // BIP-48 `m/48'/0'/0'/2'` hardware-wallet xpub under the
                    // BIP-87 `m/87'/0'/0'` multisig default. Recording the guess
                    // would put an origin on the cards and exports at which no
                    // signer finds this key. A path the operator TYPES keeps the
                    // v0.37.10 tolerance (no depth check), so the refusal only
                    // asks for that path to be stated.
                    check_defaulted_xpub_origin(idx, &xpub, &default_path)?;
                    default_path
                }
            };
            // v0.8.2 SPEC §5.1 — parse the optional `@N.master_xpub=` subkey
            // into a depth-0 Xpub. Only emitted by `--format coldcard`
            // singlesig (other formats silently ignore the slot per the
            // per-format ignored-input contract).
            let master_xpub = match slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::MasterXpub)
            {
                Some(m) => {
                    let (mx_str, _variant) = crate::slip0132::normalize_xpub_prefix(&m.value)?;
                    let mx = bitcoin::bip32::Xpub::from_str(&mx_str).map_err(|e| {
                        ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
                    })?;
                    Some(mx)
                }
                None => None,
            };
            out.push(ResolvedSlot {
                xpub,
                fingerprint,
                path,
                entropy: None,
                master_xpub,
                language: None,
                _entropy_pin: None,
            });
        } else if subkeys.contains(&SlotSubkey::Entropy) {
            // K.1: {entropy} — byte-identical to phrase resolution for the same
            // underlying entropy via the shared derive_slot helper.
            let entropy_hex = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Entropy)
                .map(|s| &*s.value)
                .expect("contains() asserts presence");
            let entropy_bytes = hex::decode(entropy_hex).map_err(|e| {
                ToolkitError::BadInput(format!("--slot @{idx}.entropy hex-decode: {e}"))
            })?;
            let lang = language.unwrap_or_default();
            let lang_bip39: bip39::Language = lang.into();
            let pass = passphrase.unwrap_or("");
            let acc = match &multisig_acct_path {
                Some(p) => crate::derive_slot::derive_bip32_from_entropy_at_path(
                    &entropy_bytes,
                    pass,
                    lang_bip39,
                    network,
                    p,
                )?,
                None => crate::derive_slot::derive_bip32_from_entropy(
                    &entropy_bytes,
                    pass,
                    lang_bip39,
                    network,
                    template,
                    account,
                )?,
            };
            // v0.10.1: `into_parts` returns bare Vec<u8> per caller-wrap
            // contract (Zeroizing-drives-scrub semantics live on the field).
            // The derived `entropy` is discarded here (the user-supplied
            // `entropy_bytes` is the canonical buffer for this slot);
            // the Drop on `acc` will scrub the now-orphaned husk.
            let (_acc_entropy, fingerprint, xpub, path) = acc.into_parts();
            let entropy_pin = Some(Rc::new(pin_pages_for(&entropy_bytes[..])));
            out.push(ResolvedSlot {
                xpub,
                fingerprint,
                path,
                // v0.10.1: ResolvedSlot.entropy migrated to Option<Zeroizing<Vec<u8>>>.
                entropy: Some(zeroize::Zeroizing::new(entropy_bytes)),
                master_xpub: None,
                language: None,
                _entropy_pin: entropy_pin,
            });
        } else if subkeys.contains(&SlotSubkey::Ms1) {
            // v0.41.0 — raw `ms1` codex32 secret. Decode + apply the
            // wire-language policy via the shared `slot_ms1` helper, then
            // derive through the SAME entropy spine the Entropy arm uses.
            // `emit_language` rides onto `ResolvedSlot.language` so the
            // re-emitted card preserves the wire language (LOAD-BEARING for
            // the verify-bundle whole-card round-trip).
            let value = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Ms1)
                .map(|s| &*s.value)
                .expect("contains() asserts presence");
            let res = crate::slot_ms1::resolve_ms1_slot(value, language, idx)?;
            let pass = passphrase.unwrap_or("");
            let acc = match &multisig_acct_path {
                Some(p) => crate::derive_slot::derive_bip32_from_entropy_at_path(
                    &res.entropy,
                    pass,
                    res.derive_language,
                    network,
                    p,
                )?,
                None => crate::derive_slot::derive_bip32_from_entropy(
                    &res.entropy,
                    pass,
                    res.derive_language,
                    network,
                    template,
                    account,
                )?,
            };
            // Discard the derived-account entropy husk (the helper-supplied
            // `res.entropy` is the canonical buffer for this slot); the Drop
            // on `acc` scrubs it.
            let (_acc_entropy, fingerprint, xpub, path) = acc.into_parts();
            // M4: bind the pin to a LOCAL before moving `res.entropy` — struct
            // fields eval left-to-right, `entropy:` precedes `_entropy_pin:`,
            // so an inline `pin_pages_for(&res.entropy[..])` would move-then-borrow.
            let entropy_pin = Some(Rc::new(pin_pages_for(&res.entropy[..])));
            out.push(ResolvedSlot {
                xpub,
                fingerprint,
                path,
                entropy: Some(res.entropy),
                master_xpub: None,
                language: res.emit_language,
                _entropy_pin: entropy_pin,
            });
        } else if subkeys.contains(&SlotSubkey::Wif) {
            // K.3 (v0.4.2) + R (v0.4.3): {wif} — degenerate single-key. Parse
            // WIF; use its public point as a depth-0 xpub with zero chain code
            // (BIP-32 framing accepts depth-0 with sentinel chain code;
            // non-derivable but the wallet policy slot just needs a stable
            // pubkey). v0.4.3 R: lifted the v0.4.2 single-sig-only guard;
            // wif slots are now legal in multisig contexts. BIP-388
            // distinctness applies normally — same WIF supplied for two slots
            // → identical pubkey + empty path → row 13 collision.
            let _ = by_index_len; // by_index_len no longer guards; multi-wif allowed.
            let wif_str = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Wif)
                .map(|s| &*s.value)
                .expect("contains() asserts presence");
            let priv_key = bitcoin::PrivateKey::from_wif(wif_str)
                .map_err(|e| ToolkitError::BadInput(format!("--slot @{idx}.wif parse: {e}")))?;
            // Bug hunt 2026-11-25 toolkit#8: a WIF carries its own network byte;
            // refuse one that disagrees with --network (the xpub arm already
            // checks), instead of re-stamping a mainnet key as a testnet wallet.
            crate::network::assert_network_agrees(
                priv_key.network,
                network.network_kind(),
                "--slot @N.wif",
            )?;
            let pubkey = priv_key.public_key(&secp);
            // Build a depth-0 xpub from the WIF's pubkey + zero chain code.
            // The KeyCard accepts this via the standard mk-codec encoder; the
            // resulting bundle's mk1 carries the wif's pubkey verbatim.
            let xpub = bitcoin::bip32::Xpub {
                network: network.network_kind(),
                depth: 0,
                parent_fingerprint: Fingerprint::default(),
                child_number: bitcoin::bip32::ChildNumber::Normal { index: 0 },
                public_key: pubkey.inner,
                chain_code: bitcoin::bip32::ChainCode::from([0u8; 32]),
            };
            // wif slots are secret-bearing for signing but ms-codec ENTR encoding
            // takes BIP-39 entropy bytes, not raw WIF bytes. v0.4.2 emits an
            // empty-string ms1 sentinel for wif slots — analogous to the xprv
            // case. Document in SPEC §5.8 amendment block.
            out.push(ResolvedSlot {
                xpub,
                fingerprint: Fingerprint::default(),
                path: DerivationPath::default(),
                entropy: None,
                master_xpub: None,
                language: None,
                _entropy_pin: None,
            });
        } else if subkeys.contains(&SlotSubkey::Xprv) {
            // K.2: {xprv} — REJECTED in v0.4.2 per impl plan r1 review C-1.
            // Resolution requires ms-codec XPRV-tag support (cross-repo cycle).
            return Err(ToolkitError::BadInput(format!(
                "--slot @{idx}.xprv not supported in v0.4.2; deferred to v0.5+ \
                pending ms-codec XPRV-tag extension. See FOLLOWUP \
                `unified-slot-xprv-resolution-needs-ms-codec-extension`."
            )));
        } else {
            return Err(ToolkitError::BadInput(format!(
                "slot @{idx} subkey set {:?} not supported by resolve_slots; \
                this should have been caught by validate_slot_set",
                subkeys.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            )));
        }
    }
    Ok((out, slip0132_signals))
}

/// v0.4.1 unified-path emit: reuses the existing emit() / emit_multisig() text
/// rendering by adapting ResolvedSlot back into the shapes those functions
/// expect. Engraving card omitted in v0.4.1 unified path (Phase I lands the
/// unified card across both paths).
fn emit_unified<W: Write, E: Write>(
    args: &BundleArgs,
    bundle: &Bundle,
    resolved: &[ResolvedSlot],
    mode: BundleMode,
    slip0132_signals: &[(u8, &'static str)],
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let _ = mode;
    // SPEC v0.6.1 §11 + v0.6.2 §5.5.a — informational notes for SLIP-0132
    // input normalization. Slot-index ascending; both callers accumulate in
    // ascending order (BTreeMap iteration in resolve_slots; 0..n range in
    // bundle_run_unified_descriptor) so no re-sort needed here. Emitted
    // unconditionally of --json (stderr advisories follow §5.5.a).
    for (_idx, variant) in slip0132_signals.iter() {
        let _ = writeln!(
            stderr,
            "{}",
            crate::slip0132::render_slip0132_info_line(variant)
        );
    }
    // ms mnem Phase 3 Step 6: re-keyed advisory — suppress iff EVERY secret-bearing
    // slot emits a self-describing `mnem` card (i.e. its effective language is
    // non-English and the card is therefore self-describing). Fire only when at
    // least one secret-bearing slot's effective language is English AND the run
    // context is non-English (that slot emits `entr`, which is language-losing
    // in a non-English run context).
    // Effective language per slot: slot.language.unwrap_or(run_language).
    // A slot emits entr iff effective_lang == English.
    // Advisory fires iff: run_language is non-English AND some secret slot's
    //   effective_lang == English (→ that slot emits entr, losing the run language).
    if bundle.any_secret_bearing() {
        let run_lang: bip39::Language = args.language.unwrap_or_default().into();
        let any_slot_emits_entr_non_english_run = run_lang != bip39::Language::English
            && resolved.iter().any(|s| {
                s.entropy.is_some() && s.language.unwrap_or(run_lang) == bip39::Language::English
            });
        if any_slot_emits_entr_non_english_run {
            if let Some(msg) = crate::language::non_english_seed_advisory(
                args.language.unwrap_or_default(),
                "an ms1 card",
            ) {
                let _ = writeln!(stderr, "{msg}");
            }
        }
    }
    let n = resolved.len();
    let mode_str = if bundle.any_secret_bearing() {
        "full"
    } else {
        "watch-only"
    };
    // v0.4.2 Phase M reconciliation: legacy emit_*/descriptor_mode_emit
    // emitted origin_path with "m/" prefix (md-codec OriginPath rendering).
    // Unified path uses bitcoin DerivationPath::to_string() which omits the
    // "m/" prefix in current bitcoin lib version. Normalize for backward-
    // compatibility with cli_json_envelopes / cli_descriptor_mode tests.
    fn normalize_origin_path(p: &str) -> String {
        if p.is_empty() || p == "m" {
            "m".to_string()
        } else if p.starts_with("m/") {
            p.to_string()
        } else {
            format!("m/{}", p)
        }
    }

    // v0.5 Phase E: absent paths emit null in JSON (was Some("m") via the
    // normalize_origin_path "" → "m" branch). The empty-string sentinel
    // (v0.37.9 — from `ResolvedSlot::origin_path_bare()` for a default path) is
    // the SPEC §4.11.b absent-path marker; null is the JSON wire-format absent.
    fn origin_path_for_json(bare_path: &str) -> Option<String> {
        if bare_path.is_empty() {
            None
        } else {
            Some(normalize_origin_path(bare_path))
        }
    }

    if args.json {
        let template = args.template.map(|t| t.human_name());
        let (multisig_info, origin_path, origin_paths) = if n == 1 {
            (
                None,
                origin_path_for_json(&resolved[0].origin_path_bare()),
                None,
            )
        } else {
            let cosigners: Vec<CosignerEntry> = resolved
                .iter()
                .enumerate()
                .map(|(i, s)| CosignerEntry {
                    index: i,
                    master_fingerprint: if args.privacy_preserving {
                        None
                    } else {
                        Some(s.fingerprint.to_string().to_lowercase())
                    },
                    origin_path: s.origin_path_bare(),
                    xpub: s.xpub.to_string(),
                })
                .collect();
            // M7 (cycle-13): report the real multisig threshold K, not the
            // cosigner count N. FIX-V7 ruling 2: template mode keeps `--threshold` (default N);
            // every other mode reports K only for a plain k-of-n descriptor
            // and null for a general policy.
            let threshold: Option<u8> = if args.template.is_some() {
                Some(args.threshold.unwrap_or(n as u8))
            } else {
                let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
                md_codec::chunk::reassemble(&md1_strs)
                    .ok()
                    .and_then(|d| plain_multisig_threshold(&d.tree))
            };
            // r1 review I-1 fix: derive path_family from --multisig-path-family
            // (defaults to bip87 when unset). Hardcoded "bip87" was wrong for
            // sh-wsh-* templates (which require bip48) and broke SPEC §5.6
            // cross-schema invariant for BIP-48 recovery tooling.
            let info = MultisigInfo {
                template: template.unwrap_or("descriptor"),
                threshold,
                cosigner_count: n,
                // FIX #54: SPEC v0.3 §5.3 — descriptor mode reports "descriptor".
                path_family: if template.is_none() {
                    "descriptor"
                } else {
                    args.multisig_path_family.unwrap_or_default().human_name()
                },
                cosigners: cosigners.clone(),
            };
            let paths: Vec<String> = cosigners.iter().map(|c| c.origin_path.clone()).collect();
            let all_same = paths.windows(2).all(|w| w[0] == w[1]);
            if all_same {
                (Some(info), paths.first().cloned(), None)
            } else {
                (Some(info), None, Some(paths))
            }
        };
        let master_fp = if n == 1 && !args.privacy_preserving {
            Some(resolved[0].fingerprint.to_string().to_lowercase())
        } else {
            None
        };
        // v0.4.2 Phase L: descriptor presence is mutually exclusive with
        // template. emit_unified is called from both paths; pick the right
        // field based on which arg was supplied.
        let descriptor_field: Option<String> = match (&args.descriptor, &args.descriptor_file) {
            (Some(s), None) => Some(s.clone()),
            (None, Some(p)) => std::fs::read_to_string(p)
                .ok()
                .map(|s| s.trim_end().to_string()),
            _ => None,
        };
        let json = BundleJson {
            schema_version: "4",
            mode: mode_str,
            network: args.network.human_name(),
            template: if descriptor_field.is_some() {
                None
            } else {
                template
            },
            descriptor: descriptor_field,
            account: args.account,
            origin_path,
            origin_paths,
            master_fingerprint: master_fp,
            ms1: bundle.ms1.clone(),
            mk1: bundle.mk1.clone(),
            md1: bundle.md1.clone(),
            multisig: multisig_info,
            privacy_preserving: args.privacy_preserving,
        };
        serde_json::to_writer(&mut *stdout, &json)
            .map_err(|e| ToolkitError::Io(std::io::Error::from(e)))?;
        writeln!(stdout).map_err(ToolkitError::Io)?;
    } else {
        // Schema-4 text mode: emit per-slot ms1 sections (skip empty sentinels).
        // v0.4.2 Phase M reconciliation: when ALL ms1 entries are empty, emit
        // an "omitted" marker line for backward-compatibility with v0.3
        // legacy text-mode output. The marker text varies by mode.
        let any_non_empty = bundle.ms1.iter().any(|s| !s.is_empty());
        if !any_non_empty {
            let marker = if args.descriptor.is_some() || args.descriptor_file.is_some() {
                "# ms1 (omitted — descriptor watch-only mode)"
            } else if n > 1 {
                "# ms1 (omitted — multisig watch-only mode)"
            } else {
                "# ms1 (omitted — xpub-only mode)"
            };
            writeln!(stdout, "{marker}").map_err(ToolkitError::Io)?;
            writeln!(stdout).map_err(ToolkitError::Io)?;
        }
        // mstring display-grouping (SPEC §6): print-once, flag-controlled
        // (default space/5). --json + verify-bundle forensics stay unbroken.
        let gs = args.group_size as usize;
        let sep = args.separator;
        for (i, ms) in bundle.ms1.iter().enumerate() {
            if ms.is_empty() {
                continue;
            }
            if n > 1 {
                writeln!(stdout, "# ms1[{i}] (entropy, BCH-checksummed)")
                    .map_err(ToolkitError::Io)?;
            } else {
                writeln!(stdout, "# ms1 (entropy, BCH-checksummed)").map_err(ToolkitError::Io)?;
            }
            writeln!(stdout, "{}", render_grouped(ms, gs, sep)).map_err(ToolkitError::Io)?;
            writeln!(stdout).map_err(ToolkitError::Io)?;
        }
        match &bundle.mk1 {
            MkField::Single(mk1) => {
                writeln!(stdout, "# mk1 (xpub + origin)").map_err(ToolkitError::Io)?;
                for s in mk1 {
                    writeln!(stdout, "{}", render_grouped(s, gs, sep)).map_err(ToolkitError::Io)?;
                }
                writeln!(stdout).map_err(ToolkitError::Io)?;
            }
            MkField::Multi(per_cosigner) => {
                for (i, chunks) in per_cosigner.iter().enumerate() {
                    writeln!(stdout, "# mk1[{}] (cosigner {} xpub + origin)", i, i)
                        .map_err(ToolkitError::Io)?;
                    for s in chunks {
                        writeln!(stdout, "{}", render_grouped(s, gs, sep))
                            .map_err(ToolkitError::Io)?;
                    }
                    writeln!(stdout).map_err(ToolkitError::Io)?;
                }
            }
        }
        // v0.4.2 Phase M reconciliation: legacy emit_multisig prefixed
        // "multisig" to the md1 header for n>1; preserve.
        let md1_header = if n > 1 {
            "# md1 (multisig wallet policy)"
        } else {
            "# md1 (wallet policy)"
        };
        writeln!(stdout, "{md1_header}").map_err(ToolkitError::Io)?;
        for s in &bundle.md1 {
            writeln!(stdout, "{}", render_grouped(s, gs, sep)).map_err(ToolkitError::Io)?;
        }
        writeln!(stdout).map_err(ToolkitError::Io)?;
        // v0.4.1 Phase I: emit unified engraving card to stderr unless suppressed.
        if !args.no_engraving_card {
            let card = build_unified_card(args, bundle, resolved);
            write!(stderr, "{}", card).ok();
        }
    }
    // #28 phase 1 (D7) — for the keyless template form, the engraved md1 does
    // NOT identify the user's specific wallet (it is identical across all users
    // of the type). Print the per-wallet `WalletPolicyId` on STDERR so the user
    // can record it with the engraving. Advisory-only; stdout (the cards) is
    // unchanged. Computed from the FULLY-KEYED, EXPLICIT-origin, presence-`0b11`
    // descriptor (NOT the elided template — origin-significant).
    if args.md1_form.is_template() {
        emit_template_wallet_id_advisory(args, bundle, resolved, stderr);
    }
    // SPEC §5.5.a: output-class advisory — PrivateKeyMaterial when any ms1
    // slot is non-empty (BIP-39 entropy on stdout); WatchOnly otherwise
    // (all ms1 == "" sentinels per §5.8). Always fires (Option never None).
    let cls = if bundle.any_secret_bearing() {
        crate::secret_advisory::OutputClass::PrivateKeyMaterial
    } else {
        crate::secret_advisory::OutputClass::WatchOnly
    };
    crate::secret_advisory::emit_output_class_advisory(cls, stderr);
    Ok(())
}

/// #28 phase 1 (D7) — print the per-wallet `WalletPolicyId` for a keyless
/// single-sig template bundle on stderr. Best-effort advisory: any failure to
/// recover the type / key is silently skipped (the cards on stdout are the
/// authoritative artifact; the canonical gate already guaranteed a
/// single-sig template reached here). The type comes from the engraved md1's
/// tree shape; the xpub + fingerprint from the resolved slot; the network +
/// account from args. The `WalletPolicyId` is computed via the SHARED
/// `wallet_policy_id_for_singlesig` helper — the SAME preimage `restore
/// --expect-wallet-id` recomputes (the D7 round-trip invariant).
fn emit_template_wallet_id_advisory<E: Write>(
    args: &BundleArgs,
    bundle: &Bundle,
    resolved: &[ResolvedSlot],
    stderr: &mut E,
) {
    // Recover the engraved (keyless) template descriptor.
    let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
    let Ok(template_desc) = md_codec::chunk::reassemble(&md1_strs) else {
        return;
    };

    // Compute the order-sensitive WalletPolicyId. Single-sig uses the shared
    // `wallet_policy_id_for_singlesig` helper (the exact preimage `restore
    // --expect-wallet-id` recomputes); multisig/general re-injects the resolved
    // keys into the template tree (`wallet_policy_id_for_template`).
    let id = if template_desc.n == 1 {
        let Some(template) = crate::synthesize::cli_template_from_tree(&template_desc.tree) else {
            return;
        };
        let Some(slot) = resolved.first() else {
            return;
        };
        match crate::synthesize::wallet_policy_id_for_singlesig(
            template,
            args.network,
            &slot.xpub,
            slot.fingerprint,
            args.account,
        ) {
            Ok(id) => id,
            Err(_) => return,
        }
    } else {
        match crate::synthesize::wallet_policy_id_for_template(&template_desc, resolved) {
            Ok(id) => id,
            Err(_) => return,
        }
    };
    let bytes = id.as_bytes();
    let hex = hex::encode(bytes);
    let prefix = hex::encode(&bytes[..4]);
    let phrase = id.to_phrase().ok().map(|p| p.to_string());

    let _ = writeln!(
        stderr,
        "notice: this is a SHAREABLE TEMPLATE — the engraved md1 does not identify YOUR wallet."
    );
    let _ = writeln!(
        stderr,
        "        record this wallet-id with the engraving (as many bytes as you wish):"
    );
    let _ = writeln!(stderr, "        wallet-id (hex):    {hex}");
    let _ = writeln!(
        stderr,
        "        wallet-id (prefix): {prefix}  (4-byte convenience prefix)"
    );
    if let Some(phrase) = phrase {
        let _ = writeln!(stderr, "        wallet-id (words):  {phrase}");
    }

    // #28 phase 2 (SPEC §3.4) — the loud order-dependent warning. For an
    // ORDER-DEPENDENT multisig/general shape (anything but sortedmulti*), the
    // N!  assignment count, the "only one assignment reproduces this wallet"
    // line, and (for a general policy) the asymmetric-semantics caveat. Softened
    // for sortedmulti* (order-independent) and skipped for single-sig (N=1).
    if template_desc.n >= 2 {
        emit_template_order_warning(&template_desc, stderr);
    }

    let _ = writeln!(
        stderr,
        "        restore with: restore --md1 <template> --from <seed> --account {} [--expect-wallet-id {prefix}]",
        args.account
    );
}

/// #28 phase 2 (SPEC §3.4) — the loud N!/asymmetric-spending-role warning at
/// multisig/general template emit. `template_desc` is the keyless template.
fn emit_template_order_warning<E: Write>(template_desc: &md_codec::Descriptor, stderr: &mut E) {
    let n = template_desc.n as u64;
    if crate::synthesize::is_order_independent_shape(&template_desc.tree) {
        // sortedmulti* — order-independent: any assignment yields the same
        // wallet, so the warning is softened.
        let _ = writeln!(
            stderr,
            "note: this is an ORDER-INDEPENDENT (sortedmulti) template — the cosigner key order \
             does not change the wallet, so any assignment of keys to slots reproduces it."
        );
        return;
    }
    // Order-dependent: exactly one of N! assignments reproduces the wallet.
    // `checked` guards a pathological descriptor with a huge placeholder count
    // (a panic in an advisory must never abort the bundle emit); n can reach 32
    // (md1's key cap, via `--descriptor --md1-form template`), and 21! overflows
    // u64, so the overflow arm is live for 21..=32 placeholders.
    let factorial: Option<u64> = (1..=n).try_fold(1u64, |acc, k| acc.checked_mul(k));
    let count = match factorial {
        Some(f) => format!("{n}! = {f}"),
        None => format!("{n}! (astronomically many)"),
    };
    let _ = writeln!(
        stderr,
        "warning: this is an ORDER-DEPENDENT template with {n} distinct cosigner slots — there are \
         {count} possible key→slot assignments and only one assignment reproduces this wallet."
    );
    // A general policy (non-canonical wrapper, e.g. wsh(or_i(...))) has
    // asymmetric branch semantics — a wrong assignment silently changes each
    // key's SPENDING ROLE, not just the receive address.
    if md_codec::canonical_origin::canonical_origin(&template_desc.tree).is_none() {
        let _ = writeln!(
            stderr,
            "         this is a GENERAL POLICY: a wrong assignment changes each key's SPENDING ROLE \
             (timelock branch, threshold membership), not just the address."
        );
    }
    let _ = writeln!(
        stderr,
        "         record the wallet-id above and/or a known receive address to complete safely."
    );
}

/// Extract the multisig threshold K from a descriptor's tree, if a
/// multi-family operator (`multi` / `sortedmulti` / `multi_a` /
/// `sortedmulti_a`) or `thresh` is present. Returns `None` for pure
/// single-sig descriptors. Walks `wsh(...)`, `sh(...)`, and `tr(IK, ...)`
/// wrappings to reach the inner threshold-bearing node.
pub(crate) fn extract_multisig_threshold(node: &md_codec::tree::Node) -> Option<u8> {
    use md_codec::tree::Body;
    match &node.body {
        Body::MultiKeys { k, .. } => Some(*k),
        Body::Variable { k, .. } => Some(*k),
        Body::Children(children) => children.iter().find_map(extract_multisig_threshold),
        Body::Tr {
            tree: Some(inner), ..
        } => extract_multisig_threshold(inner),
        _ => None,
    }
}

/// FIX-V7 (grade 2026-11-03 rulings 1-3): the threshold K of a PLAIN k-of-n
/// descriptor, or `None` when the descriptor is a general policy (or has no
/// k-of-n at all). Plain means the only child under `wsh`, `sh` or
/// `sh(wsh)`, or the only leaf of a `tr` whose internal key is the NUMS
/// point, is `multi` / `sortedmulti` / `multi_a` / `sortedmulti_a` or a
/// pure-key `thresh` (every child `pk(K)` or `pkh(K)` with only `s:` / `a:`
/// wrappers). A `Slot` or `LianaUnspendable` internal key is general.
pub(crate) fn plain_multisig_threshold(root: &md_codec::tree::Node) -> Option<u8> {
    use md_codec::tag::Tag;
    use md_codec::tree::{Body, InternalKey};
    fn only_child(node: &md_codec::tree::Node) -> Option<&md_codec::tree::Node> {
        match &node.body {
            Body::Children(c) if c.len() == 1 => Some(&c[0]),
            _ => None,
        }
    }
    fn pure_key(node: &md_codec::tree::Node) -> bool {
        let mut cur = node;
        while matches!(cur.tag, Tag::Swap | Tag::Alt) {
            match only_child(cur) {
                Some(c) => cur = c,
                None => return false,
            }
        }
        if cur.tag == Tag::Check {
            match only_child(cur) {
                Some(c) => cur = c,
                None => return false,
            }
        }
        matches!(
            (&cur.tag, &cur.body),
            (Tag::PkK | Tag::PkH, Body::KeyArg { .. })
        )
    }
    fn kofn(node: &md_codec::tree::Node) -> Option<u8> {
        match (&node.tag, &node.body) {
            (
                Tag::Multi | Tag::SortedMulti | Tag::MultiA | Tag::SortedMultiA,
                Body::MultiKeys { k, .. },
            ) => Some(*k),
            (Tag::Thresh, Body::Variable { k, children }) if children.iter().all(pure_key) => {
                Some(*k)
            }
            _ => None,
        }
    }
    match (&root.tag, &root.body) {
        (Tag::Wsh, _) => kofn(only_child(root)?),
        (Tag::Sh, _) => {
            let inner = only_child(root)?;
            if inner.tag == Tag::Wsh {
                kofn(only_child(inner)?)
            } else {
                kofn(inner)
            }
        }
        (
            Tag::Tr,
            Body::Tr {
                internal_key: InternalKey::NumsPoint,
                tree: Some(leaf),
            },
        ) if leaf.tag != Tag::TapTree => kofn(leaf),
        _ => None,
    }
}

/// #28 phase 1 — the 4-byte bundle-binding stub for a reassembled md1
/// descriptor, FORM-AWARE. A keyed wallet-policy md1 roots its binding on
/// `WalletPolicyId` (pre-#28); a keyless template md1 (`--md1-form=template`)
/// roots it on the key-stable `WalletDescriptorTemplateId` — discriminated by
/// `is_wallet_policy()` on the descriptor itself. Returns `None` if the id
/// computation fails. This is the single source of truth for the stub used by
/// both the engraving-card display labels and `self_check_bundle`.
pub(crate) fn bundle_binding_stub(d: &md_codec::Descriptor) -> Option<[u8; 4]> {
    let bytes = if d.is_wallet_policy() {
        *md_codec::compute_wallet_policy_id(d).ok()?.as_bytes()
    } else {
        *md_codec::compute_wallet_descriptor_template_id(d)
            .ok()?
            .as_bytes()
    };
    Some([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// v0.4.1 Phase I helper: assemble `BundleInputForCard` from the unified
/// dispatch's `ResolvedSlot` vec + `Bundle` + args, then render via
/// `engraving_card_unified`.
fn build_unified_card(args: &BundleArgs, bundle: &Bundle, resolved: &[ResolvedSlot]) -> String {
    use crate::format::{
        engraving_card_unified, BundleInputForCard, SlotCardBlock, TemplateOrDescriptor,
    };
    use crate::synthesize::derive_mk1_chunk_set_id_for_slot;

    let n = resolved.len() as u8;
    let template_str: &'static str = args
        .template
        .map(|t| t.human_name())
        .unwrap_or("descriptor");

    // Compute md1 chunk_set_id from the descriptor's policy_id (re-extracted
    // from the encoded md1 strings to avoid threading the policy_id through
    // the synthesis output). The reassembled Descriptor is reused below to
    // extract the multisig threshold K — args.threshold is None on the
    // --import-json descriptor-mode path, so we must read K from the
    // descriptor body itself rather than fall back to N.
    let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
    let reassembled_descriptor = md_codec::chunk::reassemble(&md1_strs).ok();
    // #28 phase 1 — the binding stub roots on `WalletPolicyId` for a keyed
    // wallet-policy md1, but on the key-stable `WalletDescriptorTemplateId` for
    // a keyless template md1 (`--md1-form=template`). Discriminate on what the
    // engraved md1 actually IS (`is_wallet_policy`), so the displayed stub
    // matches what `synthesize` emitted in the mk1/md1 strings.
    let bundle_stub_2hex = |d: &md_codec::Descriptor| -> Option<String> {
        bundle_binding_stub(d).map(|s| format!("{:02x}{:02x}", s[0], s[1]))
    };
    let md1_chunk_set_id = reassembled_descriptor
        .as_ref()
        .and_then(bundle_stub_2hex)
        .unwrap_or_else(|| "????".to_string());
    // FIX-V7 ruling 1: template mode keeps `--threshold` (default N); every
    // other mode prints K only for a plain k-of-n descriptor.
    let card_threshold: Option<u8> = if args.template.is_some() {
        args.threshold.or(if n > 1 { Some(n) } else { None })
    } else {
        reassembled_descriptor
            .as_ref()
            .and_then(|d| plain_multisig_threshold(&d.tree))
    };

    let per_slot: Vec<SlotCardBlock> = resolved
        .iter()
        .enumerate()
        .map(|(i, s)| {
            // ms1 and mk1 for slot i share the SLOT-aware chunk_set_id
            // (audit I10 / n1-vs-nge2): base = policy-stub-derived, XOR slot i.
            // The leading 15 bits are shared across all slots (bundle-binding
            // prefix); the slot (0..=31) XORs into bits 4..0, so the 5th hex char
            // varies by slot and, for slots 16..=31, so does the lowest bit of the
            // 4th — matching what synthesize actually emits per cosigner.
            // #28 — same WalletPolicyId-vs-template-id discrimination as the
            // md1 prefix above; reuse the reassembled descriptor's binding stub.
            let slot_csi_5hex = match reassembled_descriptor
                .as_ref()
                .and_then(bundle_binding_stub)
            {
                Some(stub) => {
                    format!("{:05x}", derive_mk1_chunk_set_id_for_slot(&stub, i as u32))
                }
                None => "?????".to_string(),
            };
            let ms1_card_id = if bundle.ms1.get(i).map(|s| !s.is_empty()).unwrap_or(false) {
                Some(slot_csi_5hex.clone())
            } else {
                None
            };
            SlotCardBlock {
                index: i as u8,
                ms1_card_id,
                mk1_card_id: slot_csi_5hex,
                fingerprint: if args.privacy_preserving {
                    None
                } else {
                    Some(s.fingerprint.to_string().to_lowercase())
                },
                origin_path: match s.origin_path_bare() {
                    p if p.is_empty() => None,
                    p => Some(p),
                },
            }
        })
        .collect();

    let input = BundleInputForCard {
        network: args.network.human_name(),
        template_or_descriptor: TemplateOrDescriptor::Template(template_str),
        threshold: card_threshold,
        n,
        language: args.language.map(|l| l.human_name()),
        passphrase_used: args
            .passphrase
            .as_ref()
            .map(|p| !p.is_empty())
            .unwrap_or(false),
        privacy_preserving: args.privacy_preserving,
        per_slot,
        md1_chunk_set_id,
    };

    engraving_card_unified(&input)
}

// ============================================================================
// v0.4.2 Phase L — descriptor mode under unified --slot dispatch.
// ============================================================================

use crate::parse_descriptor::{lex_placeholders, parse_descriptor, resolve_placeholders};
use crate::synthesize::{synthesize_descriptor, CosignerKeyInfo};
use bip39::Mnemonic as Bip39Mnemonic;
use bitcoin::bip32::{Xpriv as BipXpriv, Xpub as BipXpub};
use md_codec::origin_path::PathDeclPaths;

/// v0.4.2 Phase L entry point. Reached when args.descriptor / descriptor_file
/// is supplied alongside --slot. Resolves each slot per its subkey set against
/// the per-@i annotation path from the parsed descriptor, then routes through
/// the existing synthesize_descriptor pipeline.
///
/// Phase N (binding-type merge) collapses the legacy CosignerKeyInfo into
/// ResolvedSlot; v0.4.2 Phase L continues to construct CosignerKeyInfo as a
/// bridge so synthesize_descriptor's existing signature is preserved.
fn bundle_run_unified_descriptor<W: Write, E: Write>(
    args: &BundleArgs,
    slots: &[crate::slot_input::SlotInput],
    _mode: BundleMode,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    use std::collections::BTreeMap;

    let descriptor_str = match (&args.descriptor, &args.descriptor_file) {
        (Some(s), None) => s.clone(),
        (None, Some(p)) => std::fs::read_to_string(p)
            .map_err(|e| {
                ToolkitError::DescriptorParse(format!("--descriptor-file {}: {e}", p.display()))
            })?
            .trim_end()
            .to_string(),
        _ => unreachable!("clap conflicts_with rules out both / pre-checks rule out neither"),
    };
    // SPEC bip388-double-star-shorthand-support §5 — this is the AtN
    // direct-lex chokepoint (bypasses `parse_descriptor`'s own expansion at
    // its top): expand a literal `/**` once here so it covers BOTH the
    // `lex_placeholders` call below AND every subsequent `parse_descriptor`
    // call in this function (idempotent — parse_descriptor re-expands a
    // no-op).
    let descriptor_str =
        crate::parse_descriptor::expand_literal_double_star(&descriptor_str).into_owned();

    let occs = lex_placeholders(&descriptor_str)?;
    // Review r1 B4 (Brian's BIP-defaults ruling 4): a bare `@N` with no
    // derivation after it is refused at mint. BIP-388: each key placeholder
    // is "always followed by" `/**` or `/<M;N>/*`; md1 would record it as
    // `@N/*`, a different set of keys from the one typed. Only minting is
    // refused: decoding, verifying and restoring a card are unchanged.
    if let Some(occ) = occs.iter().find(|o| !o.wildcard) {
        let i = occ.i;
        return Err(ToolkitError::DescriptorParse(format!(
            "@{i} has no derivation after it: a bare `@{i}` would be recorded as `@{i}/*`, \
             which names different keys from the ones typed. Write `@{i}/<0;1>/*` (receive \
             addresses on 0, change on 1, as most wallets do), or `@{i}/*` if this wallet \
             really uses a single chain."
        )));
    }
    let mut resolved_placeholders = resolve_placeholders(&occs)?;
    let n = resolved_placeholders.n as usize;

    // Wave-4 L1: the exact-coverage gate now lives inside the shared
    // `bind_descriptor_mode_paths` (called below). Folding it there makes the
    // cycle-11b L24 guard-drift (bundle had it, verify omitted it) structurally
    // impossible. The gate runs AFTER the canonicity probe + the emit-only
    // §4.12.g / §6.6-row-4 refusals — for doubly-malformed input (over-`n` slot
    // vec AND a parse-failing descriptor / `--account != 0` on canonical / a
    // row-4 conflict) this re-orders WHICH exit-≠0 refusal surfaces, no funds /
    // accept-set / exit-class impact (SPEC §3.2 / §5 item 2).

    // v0.19.0 SPEC §4.12 — early canonicity classification. Probe-parse the
    // descriptor (empty keys/fingerprints — only the tree is consulted) so
    // canonicity is known before slot binding. The full parse_descriptor
    // call later (line ~1112) re-runs with populated keys/fingerprints; the
    // probe-parse is cheap because rust-miniscript caches nothing per-call
    // and the substituted form is small.
    let canonicity_probe = parse_descriptor(&descriptor_str, &[], &[])?;
    let is_non_canonical =
        md_codec::canonical_origin::canonical_origin(&canonicity_probe.tree).is_none();

    // v0.19.0 SPEC §4.12.g — DESCRIPTOR_WITH_NONZERO_ACCOUNT canonicity-gated.
    if !is_non_canonical && args.account != 0 {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--account",
            message: mode_text::DESCRIPTOR_WITH_NONZERO_ACCOUNT,
        });
    }

    // v0.19.0 SPEC §6.6 row 4 canonical-mode rejection of [Phrase, Path] /
    // [Phrase, Fingerprint, Path] subkey sets. Phase 2 slot grammar accepts
    // these pairs structurally; canonical descriptors refuse them here.
    if !is_non_canonical {
        let mut by_index_check: std::collections::BTreeMap<u8, Vec<&crate::slot_input::SlotInput>> =
            std::collections::BTreeMap::new();
        for s in slots {
            by_index_check.entry(s.index).or_default().push(s);
        }
        for (idx, slot_inputs) in &by_index_check {
            let subkeys: std::collections::BTreeSet<crate::slot_input::SlotSubkey> =
                slot_inputs.iter().map(|s| s.subkey).collect();
            let has_phrase = subkeys.contains(&crate::slot_input::SlotSubkey::Phrase);
            let has_seedqr = subkeys.contains(&crate::slot_input::SlotSubkey::Seedqr);
            let has_ms1 = subkeys.contains(&crate::slot_input::SlotSubkey::Ms1);
            let has_path = subkeys.contains(&crate::slot_input::SlotSubkey::Path);
            if (has_phrase || has_seedqr || has_ms1) && has_path {
                return Err(ToolkitError::SlotInputViolation {
                    kind: "conflict",
                    message: format!(
                        "slot @{idx} has both secret-bearing input and watch-only input; pick one per slot."
                    ),
                });
            }
        }
    }

    // v0.19.0 SPEC §4.12.b — default-path inference for non-canonical
    // descriptors: each `@N` with no origin and no `--slot @N.path=` gets the
    // family default (`template::DefaultOriginFamily`, Brian's 2026-11-25
    // ruling). The notice below names the path actually used.
    let default_family = crate::template::DefaultOriginFamily::for_md_tree(&canonicity_probe.tree);
    // Wave-4 L1: gate + H12 inference + new_paths build + per-slot override loop
    // (incl. the emit-mode-gated §6.6 row-19 path-mismatch refusal) + F4 collapse
    // now live in the SHARED `bind_descriptor_mode_paths`. `Emit` mode enforces
    // row-19. Returns the `defaulted_indices` for the notice below.
    // Review r1 I2 (bug hunt 2026-11-25): an inline fingerprint-only origin
    // `[fp]@N` says, per BIP-380, that the key IS the master key of `fp`. A
    // canonical card cannot record that (an empty origin on the wire means the
    // shape's canonical origin), and filling the canonical origin would rewrite
    // what the operator typed. Refuse, exactly as `--slot @N.path=m` is refused.
    if !is_non_canonical {
        use md_codec::origin_path::PathDeclPaths as P;
        for (i, fp) in resolved_placeholders.fingerprint_annos.iter().enumerate() {
            let Some(fp) = fp else { continue };
            let path_empty = match &resolved_placeholders.path_decl.paths {
                P::Shared(op) => op.components.is_empty(),
                P::Divergent(v) => v[i].components.is_empty(),
            };
            let slot_path = slots
                .iter()
                .any(|s| s.index as usize == i && s.subkey == crate::slot_input::SlotSubkey::Path);
            if path_empty && !slot_path {
                // A one-key BIP-48 shape has no canonical fill to fall back
                // on (rulings2-r1 I-1), so do not advise dropping the brackets.
                let drop_hint = if canonical_fill_is_bip48(&canonicity_probe.tree)
                    && !crate::template::tree_has_multiple_keys(&canonicity_probe.tree)
                {
                    ""
                } else {
                    ", or drop the brackets to use the canonical origin"
                };
                return Err(ToolkitError::SlotInputViolation {
                    kind: "conflict",
                    message: format!(
                        "slot @{i}: the descriptor gives `[{fp}]@{i}` (a fingerprint with no path), \
                         which says the key is the master key of {fp}. The master node cannot be \
                         recorded on a card for this descriptor shape: an empty origin on the card \
                         means the shape's canonical origin. Give the key's real origin, \
                         `[{fp}/<path>]@{i}`{drop_hint}."
                    ),
                });
            }
        }
    }
    let bound_paths = bind_descriptor_mode_paths(
        DescriptorBindMode::Emit,
        slots,
        &mut resolved_placeholders.path_decl,
        n,
        is_non_canonical,
        &canonicity_probe.tree,
        args.network,
        args.account,
    )?;

    // tk-bh1125-tr-tree-bip87-scope and tk-bh1125-rr1-m3-bip48-thresh-scope
    // (Brian's rulings, 2026-11-26): the BIP-87 and BIP-48 defaults are for
    // multisig, so a shape with fewer than two keys that would take either
    // default is refused instead of minted with an origin nobody chose.
    check_default_origin_scope(
        &bound_paths.defaulted,
        slots,
        default_family,
        &canonicity_probe.tree,
        false,
    )?;
    // Review rulings2-r1 I-1: the canonical BIP-48 fill of wsh / sh(wsh) of
    // multi / sortedmulti is the same multisig hierarchy, so a one-key
    // `wsh(multi(1,@0))` with no origin is refused like every other one-key
    // tree. Mint-time admission only: decode, verify-bundle and restore of an
    // existing card keep md-codec's canonical meaning.
    if canonical_fill_is_bip48(&canonicity_probe.tree) {
        check_default_origin_scope(
            &bound_paths.canonical_filled,
            slots,
            default_family,
            &canonicity_probe.tree,
            true,
        )?;
    }

    // Resolve each @i slot using the per-@i annotation path from the descriptor.
    let secp = Secp256k1::new();
    let mut by_index: BTreeMap<u8, Vec<&crate::slot_input::SlotInput>> = BTreeMap::new();
    for s in slots {
        by_index.entry(s.index).or_default().push(s);
    }

    let mut cosigners: Vec<CosignerKeyInfo> = Vec::with_capacity(n);
    let mut keys: Vec<crate::parse_descriptor::ParsedKey> = Vec::with_capacity(n);
    let mut fingerprints: Vec<crate::parse_descriptor::ParsedFingerprint> = Vec::with_capacity(n);
    // SPEC v0.6.2 §5.5.a — accumulate SLIP-0132 input-normalization signals.
    // The 0..n range loop walks slots in ascending order natively → no re-sort.
    let mut slip0132_signals: Vec<(u8, &'static str)> = Vec::new();

    for idx in 0..(n as u8) {
        let slot_inputs = by_index
            .get(&idx)
            .ok_or_else(|| ToolkitError::SlotInputViolation {
                kind: "gap",
                message: format!("--slot @{idx} missing for descriptor with n={n} placeholders"),
            })?;
        let subkeys: std::collections::BTreeSet<crate::slot_input::SlotSubkey> =
            slot_inputs.iter().map(|s| s.subkey).collect();

        // Per-@i annotation path from descriptor.
        let anno_path: bitcoin::bip32::DerivationPath = match &resolved_placeholders.path_decl.paths
        {
            PathDeclPaths::Shared(op) => origin_to_derivation_path(op)?,
            PathDeclPaths::Divergent(v) => origin_to_derivation_path(&v[idx as usize])?,
        };
        let anno_fp: Option<bitcoin::bip32::Fingerprint> =
            resolved_placeholders.fingerprint_annos[idx as usize];

        // v0.41.0 — 5-tuple widening (Plan-R0-I1): the 5th element carries the
        // per-slot emit language (Some(wire) for a mnem ms1; None for every
        // pre-ms1 arm) so the single shared push can stamp
        // `CosignerKeyInfo.language` — LOAD-BEARING for the verify-bundle
        // whole-card round-trip.
        let (xpub, fingerprint, path, ent_opt, emit_lang): (
            BipXpub,
            Fingerprint,
            DerivationPath,
            Option<Vec<u8>>,
            Option<bip39::Language>,
        ) = if subkeys.contains(&crate::slot_input::SlotSubkey::Phrase) {
            // SAFETY: third-party-blocked — `bip39::Mnemonic` +
            // `bitcoin::bip32::Xpriv` have no Drop+Zeroize. FOLLOWUPS:
            // `rust-bip39-mnemonic-zeroize-upstream`,
            // `rust-bitcoin-xpriv-zeroize-upstream`. The passphrase clone,
            // entropy Vec, and seed buffer are all `Zeroizing`-wrapped.
            let phrase = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Phrase)
                .map(|s| &*s.value)
                .expect("contains() asserts presence");
            let language = args.language.unwrap_or_default();
            let passphrase: zeroize::Zeroizing<String> =
                zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());
            let mnemonic =
                Bip39Mnemonic::parse_in(language.into(), phrase).map_err(ToolkitError::Bip39)?;
            let entropy = zeroize::Zeroizing::new(mnemonic.to_entropy());
            let seed = crate::derive_slot::derive_master_seed(&mnemonic, &passphrase);
            let master = BipXpriv::new_master(args.network.network_kind(), &seed[..])
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            let master_fp = master.fingerprint(&secp);
            // Cross-check fingerprint annotation if present.
            if let Some(anno) = anno_fp {
                if anno != master_fp {
                    return Err(ToolkitError::DescriptorParse(format!(
                        "--slot @{idx}.phrase derives master fingerprint {master_fp} but descriptor @{idx} annotation specifies {anno}"
                    )));
                }
            }
            // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy
            // + no Drop; FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
            let acct_xpriv = master
                .derive_priv(&secp, &anno_path)
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            let xpub = BipXpub::from_priv(&secp, &acct_xpriv);
            (
                xpub,
                master_fp,
                anno_path.clone(),
                Some((*entropy).clone()),
                None,
            )
        } else if subkeys.contains(&crate::slot_input::SlotSubkey::Xpub) {
            let xpub_str = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Xpub)
                .map(|s| &*s.value)
                .expect("contains() asserts presence");
            // SPEC v0.6.1 §11 — accept SLIP-0132 prefix variants on input.
            let (xpub_str, input_variant) = crate::slip0132::normalize_xpub_prefix(xpub_str)?;
            if let Some(v) = input_variant {
                slip0132_signals.push((idx, v));
            }
            let xpub = BipXpub::from_str(&xpub_str)
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            // Explicit `--slot @N.fingerprint=` (if supplied).
            let slot_fp: Option<Fingerprint> = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Fingerprint)
                .map(|s| {
                    Fingerprint::from_str(&s.value).map_err(|e| {
                        ToolkitError::BadInput(format!("--slot @{idx}.fingerprint parse: {e}"))
                    })
                })
                .transpose()?;
            // H7 (cycle-2): if BOTH a descriptor `@N` fingerprint annotation
            // (prefix `[fp/path]@N` or suffix `@N[fp/path]`) AND an explicit
            // `--slot @N.fingerprint=` are present, they MUST AGREE. The
            // phrase/entropy arm's master-fp cross-check (`:1617`) never reaches
            // the xpub slot, and the `.or(anno_fp)` below silently lets the slot
            // value win — so without this check a prefix-anno fp that DISAGREES
            // with `--slot @N.fingerprint=` is silently dropped (funds-safety:
            // the user's stated origin fingerprint is ignored). Refuse on
            // mismatch, modeled on the `:1618-1620` compare-and-`DescriptorParse`
            // shape and the Row-19 inline-path-vs-`--slot @N.path=` precedent.
            if let (Some(anno), Some(slot)) = (anno_fp, slot_fp) {
                if anno != slot {
                    return Err(ToolkitError::DescriptorParse(format!(
                        "--slot @{idx}.fingerprint specifies {slot} but descriptor @{idx} annotation specifies {anno}"
                    )));
                }
            }
            let fp = slot_fp.or(anno_fp).unwrap_or_default();
            let path = match slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Path)
            {
                Some(p) => DerivationPath::from_str(&p.value).map_err(|e| {
                    ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}"))
                })?,
                None => {
                    // Bug hunt 2026-11-25 toolkit#1: a canonical-shape origin the
                    // operator did not give is only a guess; use it for an xpub
                    // only when the xpub's own header agrees (as toolkit#5 does
                    // for template mode), else ask for `@N.path=`.
                    // Review r2 I-B: the same holds for a non-canonical shape's
                    // default (BIP-48 or BIP-87): a foreign key at another path
                    // (e.g. a 48'/../3' taproot cosigner) must not be recorded at
                    // the default.
                    if bound_paths.canonical_filled.contains(&idx)
                        || bound_paths.defaulted.contains(&idx)
                    {
                        check_defaulted_xpub_origin(idx, &xpub, &anno_path)?;
                    }
                    anno_path.clone()
                }
            };
            (xpub, fp, path, None, None)
        } else if subkeys.contains(&crate::slot_input::SlotSubkey::Entropy) {
            let entropy_hex = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Entropy)
                .map(|s| &*s.value)
                .expect("contains() asserts presence");
            // SAFETY: third-party-blocked — `bip39::Mnemonic` +
            // `bitcoin::bip32::Xpriv` have no Drop+Zeroize. FOLLOWUPS:
            // `rust-bip39-mnemonic-zeroize-upstream`,
            // `rust-bitcoin-xpriv-zeroize-upstream`.
            let entropy_bytes = zeroize::Zeroizing::new(hex::decode(entropy_hex).map_err(|e| {
                ToolkitError::BadInput(format!("--slot @{idx}.entropy hex-decode: {e}"))
            })?);
            let language = args.language.unwrap_or_default();
            let passphrase: zeroize::Zeroizing<String> =
                zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());
            let mnemonic = Bip39Mnemonic::from_entropy_in(language.into(), &entropy_bytes[..])
                .map_err(ToolkitError::Bip39)?;
            let seed = crate::derive_slot::derive_master_seed(&mnemonic, &passphrase);
            let master = BipXpriv::new_master(args.network.network_kind(), &seed[..])
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            let master_fp = master.fingerprint(&secp);
            // Scratch fix (report #6): same cross-check as the Phrase arm.
            if let Some(anno) = anno_fp {
                if anno != master_fp {
                    return Err(ToolkitError::DescriptorParse(format!(
                        "--slot @{idx}.entropy derives master fingerprint {master_fp} but descriptor @{idx} annotation specifies {anno}"
                    )));
                }
            }
            // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy
            // + no Drop; FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
            let acct_xpriv = master
                .derive_priv(&secp, &anno_path)
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            let xpub = BipXpub::from_priv(&secp, &acct_xpriv);
            (
                xpub,
                master_fp,
                anno_path.clone(),
                Some((*entropy_bytes).clone()),
                None,
            )
        } else if subkeys.contains(&crate::slot_input::SlotSubkey::Ms1) {
            // v0.41.0 — raw `ms1` codex32 secret cosigner. Decode + apply the
            // wire-language policy via the shared helper, then derive the
            // cosigner key at the descriptor-annotated `anno_path`. This loop
            // has NO bare `pass`/`network` locals (M-A): use `args.network` and
            // build the passphrase from `args.passphrase` like the Phrase /
            // Entropy arms above.
            let value = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Ms1)
                .map(|s| &*s.value)
                .expect("contains() asserts presence");
            let res = crate::slot_ms1::resolve_ms1_slot(value, args.language, idx)?;
            let passphrase: zeroize::Zeroizing<String> =
                zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());
            let acc = crate::derive_slot::derive_bip32_from_entropy_at_path(
                &res.entropy,
                &passphrase,
                res.derive_language,
                args.network,
                &anno_path,
            )?;
            let (_acc_entropy, master_fp, xpub, _path) = acc.into_parts();
            // Scratch fix (report #6): same cross-check as the Phrase arm.
            if let Some(anno) = anno_fp {
                if anno != master_fp {
                    return Err(ToolkitError::DescriptorParse(format!(
                        "--slot @{idx}.ms1 derives master fingerprint {master_fp} but descriptor @{idx} annotation specifies {anno}"
                    )));
                }
            }
            (
                xpub,
                master_fp,
                anno_path.clone(),
                Some((*res.entropy).clone()),
                res.emit_language,
            )
        } else {
            return Err(ToolkitError::BadInput(format!(
                "--slot @{idx} subkey set {:?} not supported in descriptor mode in v0.4.2 \
                (xprv → v0.5+; wif → multisig FOLLOWUP; partial xpub may be supported but \
                requires full descriptor binding context — file a FOLLOWUP if needed)",
                subkeys.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            )));
        };

        // v0.4.3 Phase N: per-slot entropy goes on the ResolvedSlot directly.
        // v0.10.1: CosignerKeyInfo.entropy (aliased ResolvedSlot.entropy) is
        // Option<Zeroizing<Vec<u8>>>; wrap at the field-write boundary.
        let entropy = ent_opt.clone().map(zeroize::Zeroizing::new);
        let entropy_pin = entropy.as_ref().map(|e| Rc::new(pin_pages_for(&e[..])));
        cosigners.push(CosignerKeyInfo {
            xpub,
            fingerprint,
            path,
            entropy,
            master_xpub: None,
            // v0.41.0 — per-slot emit language (Some(wire) for a mnem ms1
            // cosigner; None for every other arm). Drives the mnem-vs-entr
            // re-emit so the verify-bundle whole-card compare round-trips.
            language: emit_lang,
            _entropy_pin: entropy_pin,
        });

        keys.push(crate::parse_descriptor::ParsedKey {
            i: idx,
            payload: crate::synthesize::xpub_to_65(&xpub),
        });
        fingerprints.push(crate::parse_descriptor::ParsedFingerprint {
            i: idx,
            fp: fingerprint.to_bytes(),
        });
    }

    // SPEC §4.11.b BIP-388 distinct-key check (use bridging path: cosigners
    // already carry the typed path + entropy per slot post-v0.4.3 N alias merge).
    let dummy_binding = crate::parse_descriptor::DescriptorBinding {
        keys: keys.clone(),
        fingerprints: fingerprints.clone(),
        cosigners: cosigners.clone(),
    };
    crate::parse_descriptor::check_key_vector_distinctness(&dummy_binding)?;

    // Build md-codec Descriptor + synthesize.
    let mut descriptor = parse_descriptor(&descriptor_str, &keys, &fingerprints)?;

    // v0.19.0 SPEC §4.12.b/c — propagate the locally-mutated `path_decl`
    // (default-inference + slot-path overrides applied above) into the
    // freshly-parsed `MdDescriptor`. `parse_descriptor` re-runs
    // `resolve_placeholders` internally and would otherwise reset the
    // path_decl to its descriptor-string-derived form (empty Shared for
    // bare-`@N` non-canonical descriptors), losing the default-inference
    // mutation that `md_codec::validate_explicit_origin_required` needs to
    // accept the wire.
    //
    // PROPAGATED FOR CANONICAL SHAPES TOO, since 2026-09-19. This was the third
    // and last gate keyed on `is_non_canonical` that together dropped every
    // `--slot @N.path=` on a canonical descriptor: the binder returned early,
    // the override loop skipped xpub slots, and this refused to copy the
    // result back. Each guard was written for default-INFERENCE, which is
    // genuinely non-canonical-only; each also silently took path BINDING with
    // it.
    //
    // Unconditional is safe, checked case by case. Canonical WITH inline
    // origins: `resolve_placeholders` already put them in `path_decl`, so this
    // copies them onto themselves. Canonical with neither inline origins nor
    // slot paths: both sides are the empty `Shared`, also a no-op. Canonical
    // with slot paths: the operator's paths finally reach the card, which is
    // the whole point.
    descriptor.path_decl.paths = resolved_placeholders.path_decl.paths.clone();

    let run_language: bip39::Language = args.language.unwrap_or_default().into();
    let bundle = synthesize_descriptor(
        &descriptor,
        &cosigners,
        args.privacy_preserving,
        run_language,
        args.md1_form,
    )?;

    // Reuse emit_unified renderer (resolved must be reconstructed as
    // ResolvedSlot vec for engraving card; entropy field tracks per-slot).
    // SPEC §5.8 per-slot emission: clone entropy from each cosigners[i] so the
    // engraving-card cosigner-summary block reflects ms1 emission for every
    // phrase-bearing slot (not just @0). master_xpub: None preserves the
    // descriptor-mode invariant established above at the cosigner push.
    // ms mnem Phase 3 C1 fix: populate language on each resolved slot so that
    // emit_unified's advisory model (slot.language.unwrap_or(run_lang)) agrees
    // with the actual emitted card kind — non-English descriptor-@N slots now
    // emit mnem, so the advisory is correctly suppressed for them.
    let resolved_slots: Vec<ResolvedSlot> = cosigners
        .iter()
        .map(|c| {
            let entropy_pin = c.entropy.as_ref().map(|e| Rc::new(pin_pages_for(&e[..])));
            // Slot language: None slots inherit run_language (same unwrap_or as
            // synthesize_descriptor). This makes the advisory model in emit_unified
            // agree with the actual emitted card kind.
            let slot_language = if c.entropy.is_some() {
                Some(c.language.unwrap_or(run_language))
            } else {
                None // watch-only slots have no entropy → advisory model skips them
            };
            ResolvedSlot {
                xpub: c.xpub,
                fingerprint: c.fingerprint,
                path: c.path.clone(),
                entropy: c.entropy.clone(),
                master_xpub: None,
                language: slot_language,
                _entropy_pin: entropy_pin,
            }
        })
        .collect();

    // v0.19.0 SPEC §4.12.d — stderr info notice on default-path application.
    // Printed BEFORE the bundle to surface the assumption legibly. Suppressed
    // when no `@N` received the default (defaulted_indices is empty).
    // Seed-bound slots among the defaulted ones: the printed m/45' command
    // overrides only those (an xpub slot's key does not move with its path).
    //
    // Brian's ruling of 2026-11-26: either way (family default, or a canonical
    // shape's canonical origin, bug hunt 2026-11-25 toolkit#1) the card records
    // a path the user did not supply, so one warning says so, the command that
    // states it explicitly follows, and the exit status is 4.
    let chosen_slot = |i: u8| {
        use crate::slot_input::SlotSubkey as K;
        let subkeys: Vec<K> = slots
            .iter()
            .filter(|s| s.index == i)
            .map(|s| s.subkey)
            .collect();
        let seed = subkeys
            .iter()
            .any(|k| matches!(k, K::Phrase | K::Seedqr | K::Entropy | K::Ms1));
        ChosenSlot {
            idx: i,
            fingerprint: resolved_slots[i as usize].fingerprint,
            seed,
            // The slot grammar refuses `@N.path=` beside `@N.entropy=`, and a
            // canonical shape refuses it beside any seed (SPEC §6.6 row 4): for
            // those the path can only be stated inline, `[fp/path]@N`.
            inline_only: subkeys.contains(&K::Entropy) || (seed && !is_non_canonical),
        }
    };
    let chosen_origin = if !bound_paths.defaulted.is_empty() {
        let chosen: Vec<ChosenSlot> = bound_paths
            .defaulted
            .iter()
            .map(|i| chosen_slot(*i))
            .collect();
        emit_chosen_origin_warning(
            stderr,
            &chosen,
            &default_family.derivation_path(args.network, args.account),
            ChosenWhy::Family {
                family: default_family,
                // Bare P2SH only: sh(wsh(...)) keeps BIP-48 1' and the plain warning.
                bare_sh: canonicity_probe.tree.tag == md_codec::tag::Tag::Sh
                    && matches!(default_family, crate::template::DefaultOriginFamily::Bip87),
            },
            args.descriptor_file.is_some(),
        )?
    } else if !bound_paths.canonical_filled.is_empty() {
        let filled = &bound_paths.canonical_filled;
        let op = match &descriptor.path_decl.paths {
            PathDeclPaths::Shared(op) => op.clone(),
            PathDeclPaths::Divergent(v) => v[filled[0] as usize].clone(),
        };
        let chosen: Vec<ChosenSlot> = filled.iter().map(|i| chosen_slot(*i)).collect();
        emit_chosen_origin_warning(
            stderr,
            &chosen,
            &origin_to_derivation_path(&op)?,
            ChosenWhy::Canonical,
            args.descriptor_file.is_some(),
        )?
    } else {
        false
    };

    // SPEC_older_timelock_advisory Task 5 (Adapter-A, Site 1) — surface
    // BIP-68 consensus-masked older() operands on the finalized MdDescriptor
    // before the bundle is emitted. Non-blocking (best-effort stderr).
    let adv = crate::timelock_advisory::older_advisories_tree(&descriptor);
    crate::timelock_advisory::emit_advisories(&adv, stderr);

    // C1 (Site 1) — non-blocking advisory for shapes restore --md1 can't
    // reconstruct (sortedmulti-in-combinator / use-site overrides / hardened
    // wildcard). The engraved card is still emitted (a faithful backup).
    let unrest = crate::unrestorable_advisory::unrestorable_advisories(&descriptor);
    crate::unrestorable_advisory::emit_advisories(&unrest, stderr);
    // Cycle Y (Site 1) — LOUD funds-safety warning for a RESTORABLE-but-no-precedent
    // shape (a CUSTOM use-site on a tr(NUMS,multi_a) card). Distinct from the calm
    // advisory above; the card still restores (proceed-and-warn).
    let fs = crate::unrestorable_advisory::funds_safety_advisories(&descriptor);
    crate::unrestorable_advisory::emit_funds_safety_advisories(&fs, stderr);

    emit_unified(
        args,
        &bundle,
        &resolved_slots,
        BundleMode::SingleSigFull,
        &slip0132_signals,
        stdout,
        stderr,
    )?;

    if args.self_check {
        let expected_entropy: Vec<Option<&[u8]>> = resolved_slots
            .iter()
            .map(|r| r.entropy.as_deref().map(|v| v.as_slice()))
            .collect();
        self_check_bundle(&bundle, args, &expected_entropy)?;
    }

    Ok(if chosen_origin { EXIT_CHOSEN_ORIGIN } else { 0 })
}

/// Theme-A Phase 3a entry point — `bundle --descriptor <CONCRETE>`. Accepts a
/// bare-concrete descriptor (inline `[fp/path]xpub` keys, no `@N` placeholders)
/// and synthesizes a watch-only bundle without any `--slot` inputs. SPEC §3.2.
fn bundle_run_concrete_descriptor<W: Write, E: Write>(
    args: &BundleArgs,
    body: String,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    use crate::wallet_import::pipeline::descriptor_concrete_to_resolved_slots;
    let body_no_csum =
        crate::wallet_import::json_envelope::descriptor_body_no_csum(&body, "--descriptor")?;
    let (descriptor, resolved_slots) = descriptor_concrete_to_resolved_slots(body_no_csum)?;

    // SPEC_older_timelock_advisory Task 5 (Adapter-A, Site 3) — surface
    // BIP-68 consensus-masked older() operands on the finalized concrete-key
    // MdDescriptor before the bundle is emitted. Non-blocking (best-effort
    // stderr). `descriptor` is passed by reference downstream, never moved.
    let adv = crate::timelock_advisory::older_advisories_tree(&descriptor);
    crate::timelock_advisory::emit_advisories(&adv, stderr);

    // C1 (Site 3) — non-blocking unrestorable-shape advisory (concrete-key path).
    let unrest = crate::unrestorable_advisory::unrestorable_advisories(&descriptor);
    crate::unrestorable_advisory::emit_advisories(&unrest, stderr);
    // Cycle Y (Site 3) — LOUD funds-safety warning for a CUSTOM use-site on a
    // tr(NUMS,multi_a) card (concrete-key path).
    let fs = crate::unrestorable_advisory::funds_safety_advisories(&descriptor);
    crate::unrestorable_advisory::emit_funds_safety_advisories(&fs, stderr);

    // BIP-388 distinctness check — a pasted descriptor is untrusted, unlike
    // the mk1-sourced path in bundle_run_from_import_json.
    check_resolved_slots_distinctness(&resolved_slots)?;

    // Concrete-descriptor mode is always watch-only (no phrase/entropy input);
    // run_language is irrelevant but must be supplied — English is the safe default.
    let bundle = synthesize_descriptor(
        &descriptor,
        &resolved_slots,
        args.privacy_preserving,
        bip39::Language::English,
        args.md1_form,
    )?;
    let n = resolved_slots.len();
    let any_secret = resolved_slots.iter().any(|s| s.entropy.is_some()); // always false here
    let any_watch = resolved_slots.iter().any(|s| s.entropy.is_none());
    let mode = match (n, any_secret, any_watch) {
        (1, true, _) => BundleMode::SingleSigFull,
        (1, false, _) => BundleMode::SingleSigWatchOnly,
        (_, true, true) => BundleMode::MultisigHybrid,
        (_, true, false) => BundleMode::MultisigMultiSource,
        (_, false, _) => BundleMode::MultisigWatchOnly,
    };

    // slip0132 signals are not applicable to a bare-concrete descriptor
    // (the inline xpub prefix is canonical per SPEC §5.3).
    emit_unified(args, &bundle, &resolved_slots, mode, &[], stdout, stderr)?;

    if args.self_check {
        let expected_entropy: Vec<Option<&[u8]>> = resolved_slots
            .iter()
            .map(|r| r.entropy.as_deref().map(|v| v.as_slice()))
            .collect();
        self_check_bundle(&bundle, args, &expected_entropy)?;
    }

    Ok(())
}

/// v0.27.0 Phase 5 entry point — `bundle --import-json <FILE|->`. Consumes
/// an `import-wallet --json` envelope (SPEC §3.2 wire shape; Phase 4 ship)
/// and synthesizes a fresh bundle. Per plan §3.6.
///
/// Pipeline:
/// 1. Read envelope (file or stdin) → parse via `parse_import_json_envelopes`.
/// 2. Extract descriptor (`bundle.descriptor`) + decode `bundle.mk1` chunks
///    into `Vec<ResolvedSlot>` per §3.6.1.
/// 3. Decode envelope's `bundle.ms1[i] != ""` entries → attach entropy to
///    `resolved_slots[i]` (envelope-derived seed-bearing state).
/// 4. Apply user seed overlay (`--slot @N.phrase=`) on slots where
///    envelope `ms1[i] == ""`; conflict-on-non-empty-envelope-ms1
///    is `BadInput` exit 1 (per plan Q5 / §3.1.3). (`--ms1` is
///    import-wallet's input surface, not bundle's — see the asymmetry
///    note after the overlay loop in `bundle_run_from_import_json`.)
/// 5. Parse descriptor via `concrete_keys_to_placeholders` →
///    `parse_descriptor::parse_descriptor` (same path bundle_run_unified_descriptor
///    follows post-substitute).
/// 6. `synthesize_descriptor(descriptor, resolved_slots, privacy_preserving)`
///    → `Bundle`.
/// 7. Determine `BundleMode` from `resolved_slots` entropy state +
///    cosigner count; route through `emit_unified` (existing
///    text/JSON/engraving renderer).
fn bundle_run_from_import_json<W: Write, E: Write>(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    use crate::wallet_import::json_envelope::{
        cli_network_from_str, descriptor_body_no_csum, envelope_to_resolved_slots,
        parse_import_json_envelopes, read_import_json_arg,
    };
    use crate::wallet_import::pipeline::concrete_keys_to_placeholders;
    use bitcoin::secp256k1::Secp256k1;
    use zeroize::Zeroizing;

    let value = args
        .import_json
        .as_ref()
        .expect("caller checked --import-json.is_some()");
    let raw = read_import_json_arg(value, stdin, "--import-json")?;
    let envelope = parse_import_json_envelopes(&raw, args.import_json_index, "--import-json")?;

    // §3.6 — envelope.bundle.descriptor is the source-of-truth for the
    // descriptor; descriptor-mode synthesis applies. v0.27.0 wallet-
    // import path always emits Some.
    let descriptor_with_csum = envelope.bundle.descriptor.as_deref().ok_or_else(|| {
        ToolkitError::BadInput(
            "--import-json: envelope.bundle.descriptor is null; v0.27.0 wallet-import \
                 path always emits the descriptor string verbatim"
                .to_string(),
        )
    })?;
    let descriptor_body = descriptor_body_no_csum(descriptor_with_csum, "--import-json")?;

    // Decode mk1 chunks per §3.6.1 → ResolvedSlots (entropy=None). v0.27.1
    // Phase 2 I5 fold: stderr carries the origin_fingerprint substitution
    // NOTICE if any mk1 card omits the master fingerprint.
    let mut resolved_slots = envelope_to_resolved_slots(&envelope, stderr)?;

    // Network: env-derived. Used for entropy → xpub derivation in the
    // seed-overlay step + as the cross-check against args.network when
    // the user supplied one explicitly (--network defaults silently to
    // mainnet on the bundle subcommand; we use envelope.network as the
    // source-of-truth for the consumer path).
    let envelope_network = cli_network_from_str(&envelope.bundle.network)?;
    // FIX #45: the required --network must agree with the envelope.
    if args.network != envelope_network {
        let flag = args.network.human_name();
        let env = envelope_network.human_name();
        // An envelope imported without `--network` records every coin-type-1
        // wallet as testnet, so signet/regtest against it gets the longer text.
        let same_class = matches!(envelope_network, CliNetwork::Testnet)
            && matches!(args.network, CliNetwork::Signet | CliNetwork::Regtest);
        return Err(ToolkitError::BadInput(if same_class {
            format!(
                "--network {flag} disagrees with the --import-json envelope's bundle.network \
                 ({env}); if the wallet is on {flag}, re-create the envelope with import-wallet \
                 --network {flag} from the same blob, otherwise re-run with --network {env}"
            )
        } else {
            format!(
                "--network {flag} disagrees with the --import-json envelope's bundle.network \
                 ({env}); re-run with --network {env}"
            )
        }));
    }
    let secp = Secp256k1::new();

    // §3.6 — decode envelope's ms1[i] != "" entries first (envelope-
    // declared seed-bearing slots). These are equivalent to v0.26.0
    // "full" import-wallet's seed-overlay-on-emit state.
    let n = resolved_slots.len();
    if envelope.bundle.ms1.len() != n {
        return Err(ToolkitError::BadInput(format!(
            "--import-json: envelope.bundle.ms1 length {} disagrees with mk1 cosigner count {n}",
            envelope.bundle.ms1.len()
        )));
    }
    for (i, ms1_str) in envelope.bundle.ms1.iter().enumerate() {
        if ms1_str.is_empty() {
            continue;
        }
        let (_tag, payload) = ms_codec::decode(ms1_str).map_err(|e| {
            ToolkitError::BadInput(format!(
                "--import-json: envelope.bundle.ms1[{i}] decode failed: {e:?}"
            ))
        })?;
        // ms mnem Phase 3 (R2-I6): bind both Entr and Mnem payloads.
        // Populate slot.language from the wire for mnem cards (emit-only path —
        // this flow does NOT re-derive; xpub came from the envelope's mk1 chunk).
        //
        // C2 regression fix: an Entr wire card is language-AGNOSTIC and MUST stay
        // Entr on re-emit regardless of --language. Set language=English explicitly
        // so synthesize_descriptor's unwrap_or(run_language) never fabricates a
        // non-English language for it. An entr card emitted with English language
        // produces the Entr payload (synthesize_descriptor's emit_lang == English
        // branch → Payload::Entr), preserving the wire shape faithfully.
        let (entropy_bytes, slot_lang) = match payload {
            ms_codec::Payload::Entr(bytes) => (Zeroizing::new(bytes), bip39::Language::English),
            ms_codec::Payload::Mnem {
                entropy,
                language: wire_lang,
                ..
            } => {
                let lang = crate::language::wire_code_to_bip39(wire_lang).map_err(|e| {
                    ToolkitError::BadInput(format!("--import-json: envelope.bundle.ms1[{i}] {e}"))
                })?;
                (Zeroizing::new(entropy), lang)
            }
            _ => {
                return Err(ToolkitError::BadInput(format!(
                    "--import-json: envelope.bundle.ms1[{i}] payload is not entropy"
                )));
            }
        };
        resolved_slots[i].entropy = Some(entropy_bytes);
        resolved_slots[i].language = Some(slot_lang);
    }

    // FIX #46: refuse every overlay this path does not apply.
    if let Some(s) = args
        .slot
        .iter()
        .find(|s| s.subkey != crate::slot_input::SlotSubkey::Phrase)
    {
        use crate::slot_input::SlotSubkey as K;
        let sub = s.subkey.as_str();
        return Err(ToolkitError::BadInput(match s.subkey {
            K::Entropy | K::Ms1 | K::Seedqr => format!(
                "--slot @{n}.{sub}=: not supported with --import-json; only --slot @N.phrase= can \
                 add a seed to a watch-only cosigner. Get the phrase with: mnemonic convert \
                 --from {sub}=- --to phrase (it reads the value from stdin and prints \"phrase: <words>\"), \
                 then give the words alone, without the \"phrase: \" label, on stdin to \
                 --slot @{n}.phrase=-",
                n = s.index
            ),
            _ => format!(
                "--slot @{}.{sub}=: not supported with --import-json; the envelope already \
                 supplies every cosigner's key, fingerprint and path (only --slot @N.phrase= is \
                 accepted)",
                s.index
            ),
        }));
    }

    // §3.6 + Q5 — apply user seed overlay (--slot @N.phrase=) on
    // watch-only cosigners. Conflict on non-empty envelope ms1[i] is
    // BadInput (exit 1). (--ms1 is import-wallet's input surface, not
    // bundle's — see the asymmetry note after this loop.)
    for (i, user_ms1) in args.slot.iter().filter_map(|s| {
        if s.subkey == crate::slot_input::SlotSubkey::Phrase {
            // cycle-14 (L22): `.value` is a SecretString; Deref to `&str` for
            // the downstream bip39 parse (no AsRef<str> on &SecretString).
            Some((s.index as usize, &*s.value))
        } else {
            None
        }
    }) {
        if i >= n {
            return Err(ToolkitError::BadInput(format!(
                "--slot @{i}.phrase=: cosigner index out of range (envelope has {n} cosigners)"
            )));
        }
        if !envelope.bundle.ms1[i].is_empty() {
            return Err(ToolkitError::BadInput(format!(
                "--slot @{i}.phrase=: envelope already carries entropy for cosigner {i} \
                 (bundle.ms1[{i}] != \"\"); supply overlay only for watch-only slots"
            )));
        }
        // Resolve phrase → entropy → derive xpub at resolved_slots[i].path → verify match.
        let language = args.language.unwrap_or_default();
        // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+Zeroize.
        // FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`.
        let mnemonic = bip39::Mnemonic::parse_in(language.into(), user_ms1).map_err(|e| {
            ToolkitError::BadInput(format!("--slot @{i}.phrase=: BIP-39 parse error: {e}"))
        })?;
        let entropy = Zeroizing::new(mnemonic.to_entropy());
        let passphrase: Zeroizing<String> =
            Zeroizing::new(args.passphrase.clone().unwrap_or_default());
        let seed = crate::derive_slot::derive_master_seed(&mnemonic, &passphrase);
        // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy + no
        // Drop; FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
        let master = BipXpriv::new_master(envelope_network.network_kind(), &seed[..])
            .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
        // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy + no
        // Drop; FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
        let child = master
            .derive_priv(&secp, &resolved_slots[i].path)
            .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
        let derived_xpub = BipXpub::from_priv(&secp, &child);
        // Path-only-mk1-encoded xpub may differ from the user's derivation
        // by reconstructed depth+child_number — compare via (parent_fp,
        // chain_code, public_key) tuple per the Phase 4 holistic-review
        // mk-codec depth/child reconstruction note.
        if derived_xpub.parent_fingerprint != resolved_slots[i].xpub.parent_fingerprint
            || derived_xpub.chain_code != resolved_slots[i].xpub.chain_code
            || derived_xpub.public_key != resolved_slots[i].xpub.public_key
        {
            return Err(ToolkitError::ImportWalletSeedMismatch {
                cosigner_index: i,
                derived_xpub: derived_xpub.to_string(),
                blob_xpub: resolved_slots[i].xpub.to_string(),
                path: resolved_slots[i].origin_path_bare(),
            });
        }
        resolved_slots[i].entropy = Some(entropy);
        // C2 regression fix: set language explicitly from the user's --language
        // so synthesize_descriptor's unwrap_or(run_language) is moot for this
        // slot — the slot already carries the correct target language.
        resolved_slots[i].language = Some(language.into());
    }

    // --ms1 positional overlay (vs --slot @N.phrase=). Repeat with the
    // ms_codec entropy decode + same xpub-equivalence check.
    // BundleArgs doesn't currently expose --ms1 (that's import-wallet's
    // surface); the seed-overlay channel on `bundle --import-json` is
    // --slot @N.phrase= per the existing bundle subcommand convention.
    // Document the asymmetry in the import-wallet --json envelope's
    // overlay model — `--ms1` is the import-wallet-side surface and
    // already attaches entropy via Phase 4 emit → bundle.ms1[i] != "".
    // No `--ms1` arg on BundleArgs to handle here.

    // §3.6 — parse descriptor: concrete-keys → @N placeholders → md_codec.
    let (placeholder_form, parsed_keys, parsed_fps) =
        concrete_keys_to_placeholders(descriptor_body)?;
    let descriptor =
        crate::parse_descriptor::parse_descriptor(&placeholder_form, &parsed_keys, &parsed_fps)
            .map_err(|e| {
                ToolkitError::DescriptorParse(format!(
                    "--import-json: descriptor re-parse failed: {}",
                    e.message()
                ))
            })?;

    crate::wallet_import::json_envelope::check_envelope_consistency(
        &envelope,
        descriptor_body,
        &descriptor,
        &resolved_slots,
        envelope_network,
        true,
        "--import-json",
    )?;

    // Synthesize. run_language defaults to English; import-json slots that
    // carry Some(wire_lang) override via unwrap_or in synthesize_descriptor,
    // so import-json behavior is unchanged by this parameter.
    let run_language_import: bip39::Language = args.language.unwrap_or_default().into();
    let bundle = synthesize_descriptor(
        &descriptor,
        &resolved_slots,
        args.privacy_preserving,
        run_language_import,
        args.md1_form,
    )?;

    // SPEC_older_timelock_advisory Task 5 (Adapter-A, Site 2) — surface
    // BIP-68 consensus-masked older() operands on the re-parsed import-json
    // MdDescriptor before the bundle is emitted. Non-blocking (best-effort).
    let adv = crate::timelock_advisory::older_advisories_tree(&descriptor);
    crate::timelock_advisory::emit_advisories(&adv, stderr);

    // C1 (Site 2) — non-blocking unrestorable-shape advisory (import-json path).
    let unrest = crate::unrestorable_advisory::unrestorable_advisories(&descriptor);
    crate::unrestorable_advisory::emit_advisories(&unrest, stderr);
    // Cycle Y (Site 2) — LOUD funds-safety warning for a CUSTOM use-site on a
    // tr(NUMS,multi_a) card (import-json path).
    let fs = crate::unrestorable_advisory::funds_safety_advisories(&descriptor);
    crate::unrestorable_advisory::emit_funds_safety_advisories(&fs, stderr);

    // Determine BundleMode from resolved_slots state.
    let any_secret = resolved_slots.iter().any(|s| s.entropy.is_some());
    let any_watch = resolved_slots.iter().any(|s| s.entropy.is_none());
    let mode = match (n, any_secret, any_watch) {
        (1, true, _) => BundleMode::SingleSigFull,
        (1, false, _) => BundleMode::SingleSigWatchOnly,
        (_, true, true) => BundleMode::MultisigHybrid,
        (_, true, false) => BundleMode::MultisigMultiSource,
        (_, false, _) => BundleMode::MultisigWatchOnly,
    };

    // Emit. emit_unified derives its `descriptor` field from
    // `args.descriptor` / `args.descriptor_file`; the --import-json path
    // doesn't populate either, so we clone the args struct and inject
    // the envelope's descriptor (including #<csum>) so the emit-side
    // `descriptor` field round-trips faithfully through the new envelope.
    // This mirrors `cmd::bundle::run`'s descriptor-mode dispatch
    // pre-conditions; clap-mutex `conflicts_with_all` is bypassed at
    // this layer (the synthetic args is internal, not user-facing).
    let mut emit_args = args.clone();
    emit_args.descriptor = Some(descriptor_with_csum.to_string());
    // slip0132 signals are not applicable to envelope-sourced bundles
    // (the envelope's xpub field is canonical per SPEC §5.3).
    emit_unified(
        &emit_args,
        &bundle,
        &resolved_slots,
        mode,
        &[],
        stdout,
        stderr,
    )?;

    if args.self_check {
        let expected_entropy: Vec<Option<&[u8]>> = resolved_slots
            .iter()
            .map(|r| r.entropy.as_deref().map(|v| v.as_slice()))
            .collect();
        self_check_bundle(&bundle, args, &expected_entropy)?;
    }

    Ok(())
}

/// Distinguishes the emit (`bundle`) caller from the read-only round-trip
/// (`verify-bundle`) caller of [`bind_descriptor_mode_paths`]. The ONLY
/// behavioral difference inside the shared fn is the §6.6 row-19 inline-vs-slot
/// path-mismatch refusal: emit refuses; verify-bundle is read-only and silently
/// accepts (it later compares the re-derived md1 byte-for-byte, so a genuinely
/// inconsistent inline/slot path surfaces as a md1 mismatch, not a refusal). All
/// other emit-only refusals (§4.12.g `--account`, §6.6-row-4 `[Phrase,Path]`)
/// and the stderr default-inference notice stay at the bundle CALL SITE and
/// never enter the shared fn — importing any of them here would change
/// verify-bundle's accept-set.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum DescriptorBindMode {
    /// `bundle --descriptor` — emit; enforce the row-19 path-mismatch refusal.
    Emit,
    /// `verify-bundle --descriptor` — read-only round-trip; skip row-19.
    Verify,
}

/// Wave-4 L1 SHARED descriptor-mode path binding for non-canonical `@N`
/// descriptors, consumed by BOTH `bundle_run_unified_descriptor` (emit) and
/// `verify_bundle::descriptor_mode_verify_run` (round-trip cross-check). Folds
/// the cycle-11b L24 exact-coverage gate + H12 default-script-type inference +
/// `new_paths` default-path build + per-slot `--slot @N.path=` override loop +
/// F4 identical-path `Shared`-collapse into ONE site so the two callers can
/// never drift (the L24 panic was exactly that drift: verify omitted the gate
/// → OOB write).
///
/// MUTATES `path_decl.paths` in place (default-inference + overrides + F4
/// collapse). RETURNS the `defaulted_indices` (the `@N` that received the
/// default path) for the caller's use — `bundle` feeds it to
/// `emit_default_path_notice`; `verify-bundle` discards it.
///
/// Inputs are already-resolved so each caller keeps its OWN error mapping for
/// the lex/resolve/probe steps (bundle → `DescriptorParse`; verify-bundle →
/// `DescriptorReparseFailed`) — those steps deliberately stay at the call site,
/// which is why this takes `n` + `is_non_canonical` + `root_tree` rather than
/// the descriptor string.
///
/// EMIT-ONLY concerns that do NOT belong here (caller-side): the §4.12.g
/// `--account != 0` refusal, the §6.6-row-4 `[Phrase,Path]` refusal, and the
/// stderr notice.
///
/// The arg count (8) is deliberate: passing already-resolved `n` /
/// `is_non_canonical` / `root_tree` keeps the lex/resolve/probe steps (and their
/// per-caller error mapping) at the call site, which is the whole point of the
/// dedup boundary (SPEC §3.1). Bundling them into a struct would only relocate,
/// not reduce, the coupling.
#[allow(clippy::too_many_arguments)]
pub(crate) fn bind_descriptor_mode_paths(
    mode: DescriptorBindMode,
    slots: &[crate::slot_input::SlotInput],
    path_decl: &mut md_codec::origin_path::PathDecl,
    n: usize,
    is_non_canonical: bool,
    root_tree: &md_codec::tree::Node,
    network: crate::network::CliNetwork,
    account: u32,
) -> Result<BoundDescriptorPaths, ToolkitError> {
    use md_codec::origin_path::{OriginPath, PathDeclPaths};

    // cycle-11b L24 — exact-coverage bounds gate. `validate_slot_set` checks
    // contiguity (`0..=max_idx`) only, NOT range-vs-`n`; a contiguous slot set
    // whose max index exceeds `n` would reach the per-slot override loop and
    // OOB-write `new_paths[idx]` (panic). Folding the gate here makes the L24
    // drift (verify omitted it) structurally impossible — one gate, both
    // callers. Catches the over-`n` AND under-`n` cases.
    let covered = slots
        .iter()
        .map(|s| s.index as usize + 1)
        .max()
        .unwrap_or(0);
    if covered != n {
        return Err(ToolkitError::DescriptorParse(format!(
            "descriptor has n={n} placeholders but --slot vec covers {covered} slots"
        )));
    }

    let mut defaulted_indices: Vec<u8> = Vec::new();
    // CANONICAL DESCRIPTORS SKIP DEFAULT-INFERENCE, NOT PATH BINDING.
    //
    // This used to `return` here. The comment explained only the first half --
    // "no default-inference" -- and the early return also skipped the
    // `--slot @N.path=` override loop below, so for a canonical descriptor
    // every supplied path was accepted, used for derivation, and then dropped
    // on the floor. The emitted card carried one shared EMPTY origin and
    // recorded where NO key lived.
    //
    // It hid for two reasons: every slot dropped its path identically, so
    // `verify-bundle` round-tripped the card to itself, and addresses derive
    // from the xpubs a card CARRIES rather than the origin it DECLARES, so
    // every address check passed either way. md-codec 0.43's
    // `OriginKeyContradiction` is what finally asked the question.
    //
    // So: default-inference stays gated on `is_non_canonical` (a canonical
    // shape has a canonical origin and must not be second-guessed), and
    // everything below it runs for both.

    // Default origin family (Brian's BIP-defaults ruling, 2026-11-25; see
    // `template::DefaultOriginFamily`): sh(wsh) → BIP-48 1', wsh → BIP-48 2',
    // any other sh(...) and every tr(...) → BIP-87 m/87'/c'/a'. Derived from
    // the SAME canonicity-probe tree both callers pass, so emit and verify
    // agree. (H12 used BIP-48 3' for taproot, which no BIP defines.)
    let default_path =
        crate::template::DefaultOriginFamily::for_md_tree(root_tree).origin_path(network, account);

    let mut new_paths: Vec<OriginPath> = match &path_decl.paths {
        PathDeclPaths::Shared(op) => {
            if op.components.is_empty() {
                if is_non_canonical {
                    // All slots default.
                    defaulted_indices.extend(0..(n as u8));
                    (0..n).map(|_| default_path.clone()).collect()
                } else {
                    // Canonical with no inline origins: leave them EMPTY here
                    // and let the override loop below fill in whatever the
                    // operator supplied. Defaulting a canonical shape would
                    // invent an origin the descriptor never claimed.
                    (0..n)
                        .map(|_| OriginPath {
                            components: Vec::new(),
                        })
                        .collect()
                }
            } else {
                // Shared non-empty: no defaulting; lift to Divergent for
                // uniform downstream handling.
                (0..n).map(|_| op.clone()).collect()
            }
        }
        PathDeclPaths::Divergent(v) => v
            .iter()
            .enumerate()
            .map(|(i, op)| {
                // THE SAME GATE AS THE `Shared` ARM ABOVE, and its absence here
                // was a Critical found in whole-diff review (2026-09-19).
                //
                // Ungating this function for canonical shapes let a canonical
                // descriptor with PARTIAL inline origins --
                // `wsh(sortedmulti(2,[fp/48'/0'/0'/2']@0,@1))` -- reach this
                // arm with one empty entry, and the arm INVENTED an origin for
                // it. For a phrase/entropy/ms1 slot the key is then DERIVED at
                // the invented path, so the same command line produced a
                // different wallet than the shipped binary:
                //
                //   shipped: @1 [3f635a63/m]               id 7dd635d7...
                //   broken:  @1 [3f635a63/48'/0'/0'/2']    id a28485fe...
                //
                // and a different first address
                // (bc1qq3p989... vs bc1qt2yw8z4...). Default-INFERENCE is
                // non-canonical-only; a canonical shape's un-annotated slot
                // must be left exactly as the operator left it.
                if op.components.is_empty() && is_non_canonical {
                    defaulted_indices.push(i as u8);
                    default_path.clone()
                } else {
                    op.clone()
                }
            })
            .collect(),
    };

    // Apply per-slot `--slot @N.path=` overrides (phrase-bearing slots only;
    // the Xpub branch in the binding loop has its own path-override handling).
    let mut by_index_path: std::collections::BTreeMap<u8, &crate::slot_input::SlotInput> =
        std::collections::BTreeMap::new();
    for s in slots {
        if s.subkey == crate::slot_input::SlotSubkey::Path {
            by_index_path.insert(s.index, s);
        }
    }
    for (idx, slot_path) in &by_index_path {
        // XPUB-BEARING SLOTS ROUTE THROUGH HERE TOO, and used not to.
        //
        // The note said they were "handled by the per-slot binding loop's own
        // override logic". That loop does read `--slot @N.path=` -- but only
        // into the ResolvedSlot it derives with. It never writes the path back
        // into `path_decl`, which is what becomes the CARD's origin. So an
        // `@N.xpub=` + `@N.path=` slot derived correctly and then engraved a
        // card that did not say where the key came from.
        //
        // Every slot kind that can supply a path now folds it into the same
        // place. The Xpub branch's own handling is unaffected: it still uses
        // the path for derivation, and the row-19 mismatch guard below still
        // refuses a slot path that contradicts an inline one.
        let user_path = DerivationPath::from_str(&slot_path.value)
            .map_err(|e| ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}")))?;
        let user_origin = derivation_path_to_origin(&user_path);
        // §6.6 row 19 — EMIT-ONLY: inline `[fp/path]@N` AND `--slot @N.path=`
        // both supplied, non-empty, and differ → refuse. verify-bundle skips
        // it (read-only; a genuine conflict surfaces downstream as a md1
        // byte-mismatch, never as a refusal — preserves its accept-set).
        if mode == DescriptorBindMode::Emit
            && !defaulted_indices.contains(idx)
            && !new_paths[*idx as usize].components.is_empty()
            && new_paths[*idx as usize] != user_origin
        {
            let inline_path = origin_to_derivation_path(&new_paths[*idx as usize])?;
            return Err(ToolkitError::SlotInputViolation {
                kind: "path-mismatch",
                message: format!(
                    "slot @{idx} path mismatch: --slot says {user_path}, descriptor inline [.../{inline_path}] disagrees; supply consistent values or remove one source."
                ),
            });
        }
        new_paths[*idx as usize] = user_origin;
        // Slot-supplied path takes precedence; if it was a default, remove from
        // the notice list.
        defaulted_indices.retain(|i| i != idx);
    }

    // Bug hunt 2026-11-25 toolkit#1 (Brian's ruling on the empty origin of a
    // KEYED card): a canonical shape whose `@N` still has no origin here (no
    // inline `[fp/path]@N`, no `--slot @N.path=`) used to be minted with an
    // EMPTY origin, and a seed slot was then derived at the MASTER node. An
    // empty origin on the wire means "the canonical origin, elided", so that
    // card described a wallet nobody expects (m/0/i addresses) and put the
    // master xpub on public cards. Fill the canonical origin of the shape
    // instead (md-codec `canonical_origin`, coin from --network), write it on
    // the card explicitly, and report the slots so the caller can say so.
    //
    // A slot whose operator TYPED an empty path (`--slot @N.path=m`) is not
    // filled: in Emit mode it is refused (the master node cannot be written
    // on a canonical card, the wire would read it as the canonical origin);
    // Verify mode stays read-only and leaves it as typed.
    let mut canonical_filled: Vec<u8> = Vec::new();
    if !is_non_canonical {
        if let Some(canon) = canonical_origin_for_network(root_tree, network) {
            for (i, op) in new_paths.iter_mut().enumerate() {
                if !op.components.is_empty() {
                    continue;
                }
                if by_index_path.contains_key(&(i as u8)) {
                    if mode == DescriptorBindMode::Emit {
                        return Err(ToolkitError::SlotInputViolation {
                            kind: "conflict",
                            message: format!(
                                "slot @{i}: `--slot @{i}.path=m` (the master node) cannot be recorded on a \
                                 card for this descriptor shape: an empty origin on the card means the \
                                 shape's canonical origin. Give the key's real origin path."
                            ),
                        });
                    }
                    continue;
                }
                *op = canon.clone();
                canonical_filled.push(i as u8);
            }
        }
    }

    // F4 — collapse identical inferred per-`@N` paths to `Shared` for
    // cross-start byte-convergence with the explicit-origin / wallet-import
    // path. `new_paths` is non-empty (n >= 1 enforced upstream); a 1-element
    // vec is trivially all-same.
    let all_same = new_paths.windows(2).all(|w| w[0] == w[1]);
    path_decl.paths = if all_same {
        PathDeclPaths::Shared(new_paths[0].clone())
    } else {
        PathDeclPaths::Divergent(new_paths)
    };

    Ok(BoundDescriptorPaths {
        defaulted: defaulted_indices,
        canonical_filled,
    })
}

/// Brian's rulings of 2026-11-26 (`tk-bh1125-tr-tree-bip87-scope`,
/// `tk-bh1125-rr1-m3-bip48-thresh-scope`): refuse to mint when an `@N` would
/// take a family default origin (BIP-87, or BIP-48 for wsh / sh(wsh)) on a
/// shape that references fewer than two distinct keys
/// (`template::tree_has_multiple_keys`). BIP-87: "it is best to restrict this
/// script-agnostic hierarchy to multisignature wallets only"; BIP-48 is a
/// multisig hierarchy too. No example path is suggested: the right one depends
/// on where the key really lives (see FOLLOWUPS
/// `tk-bh1125-r3-m1-refusal-example-path`). Canonical shapes never take a
/// family default; the bundle caller applies this same check to the
/// canonical BIP-48 fill of wsh / sh(wsh) of multi / sortedmulti
/// ([`canonical_fill_is_bip48`]), which a one-of-one `multi` would otherwise
/// take. The single-sig canonical fills (BIP-44/49/84/86) are not affected.
fn check_default_origin_scope(
    defaulted: &[u8],
    slots: &[crate::slot_input::SlotInput],
    family: crate::template::DefaultOriginFamily,
    root_tree: &md_codec::tree::Node,
    canonical_shape: bool,
) -> Result<(), ToolkitError> {
    if defaulted.is_empty() || crate::template::tree_has_multiple_keys(root_tree) {
        return Ok(());
    }
    let family_text = match family {
        crate::template::DefaultOriginFamily::Bip87 => "BIP-87 (m/87'/...)",
        crate::template::DefaultOriginFamily::Bip48 { .. } => "BIP-48 (m/48'/...)",
    };
    let idx_list = defaulted
        .iter()
        .map(|i| format!("@{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let mut message = format!(
        "no origin was given for {idx_list}, and this descriptor holds only one key. \
         {family_text} is for wallets with two or more keys, so no default origin is \
         assumed. Give each key's real origin with --slot @N.path=m/... or inline \
         [fp/path]@N in the descriptor."
    );
    // The slot grammar refuses `@N.path=` beside `@N.entropy=`, so name only
    // the inline form for those slots (never advise a refused input).
    let entropy_only: Vec<String> = defaulted
        .iter()
        .filter(|i| {
            slots
                .iter()
                .any(|s| s.index == **i && s.subkey == crate::slot_input::SlotSubkey::Entropy)
        })
        .map(|i| format!("@{i}"))
        .collect();
    if !entropy_only.is_empty() {
        message.push_str(&format!(
            " An @N.entropy= slot ({}) takes only the inline form.",
            entropy_only.join(",")
        ));
    }
    // A canonical shape refuses `@N.path=` beside a phrase / SeedQR / ms1
    // slot (SPEC v0.19.0 §6.6 row 4), so those take only the inline form too.
    if canonical_shape {
        use crate::slot_input::SlotSubkey as K;
        let seed_only: Vec<String> = defaulted
            .iter()
            .filter(|i| {
                slots
                    .iter()
                    .any(|s| s.index == **i && matches!(s.subkey, K::Phrase | K::Seedqr | K::Ms1))
            })
            .map(|i| format!("@{i}"))
            .collect();
        if !seed_only.is_empty() {
            message.push_str(&format!(
                " On this descriptor shape a seed slot ({}) takes only the inline form.",
                seed_only.join(",")
            ));
        }
    }
    Err(ToolkitError::SlotInputViolation {
        kind: "missing-origin",
        message,
    })
}

/// Whether `tree`'s canonical origin (md-codec `canonical_origin`) is the
/// BIP-48 multisig hierarchy (`m/48'/...`, wsh / sh(wsh) of multi /
/// sortedmulti), as opposed to a single-sig BIP-44/49/84/86 origin.
pub(crate) fn canonical_fill_is_bip48(tree: &md_codec::tree::Node) -> bool {
    md_codec::canonical_origin::canonical_origin(tree).is_some_and(|op| {
        op.components
            .first()
            .is_some_and(|c| c.hardened && c.value == 48)
    })
}

/// What [`bind_descriptor_mode_paths`] chose on the operator's behalf.
#[derive(Debug, Default)]
pub(crate) struct BoundDescriptorPaths {
    /// Non-canonical `@N` given the family default origin (BIP-48 or BIP-87,
    /// `template::DefaultOriginFamily`).
    pub defaulted: Vec<u8>,
    /// Canonical-shape `@N` given the shape's canonical origin (bug hunt
    /// 2026-11-25 toolkit#1).
    pub canonical_filled: Vec<u8>,
}

/// md-codec's `canonical_origin` for `tree` with the coin-type component set
/// from `network` (md-codec's table is written for mainnet, coin `0'`). `None`
/// for a non-canonical shape.
pub(crate) fn canonical_origin_for_network(
    tree: &md_codec::tree::Node,
    network: crate::network::CliNetwork,
) -> Option<md_codec::origin_path::OriginPath> {
    let mut op = md_codec::canonical_origin::canonical_origin(tree)?;
    if let Some(coin) = op.components.get_mut(1) {
        coin.value = network.coin_type();
    }
    Some(op)
}

/// Convert a `bitcoin::bip32::DerivationPath` to `md_codec::origin_path::OriginPath`.
/// Used to fold `--slot @N.path=` user input into `path_decl.paths` for
/// non-canonical-descriptor default-inference override. Public for the
/// same symmetric-verify-bundle reason as `compute_default_origin_path`.
pub fn derivation_path_to_origin(dp: &DerivationPath) -> md_codec::origin_path::OriginPath {
    use bitcoin::bip32::ChildNumber;
    use md_codec::origin_path::{OriginPath, PathComponent};
    OriginPath {
        components: dp
            .into_iter()
            .map(|c| match c {
                ChildNumber::Normal { index } => PathComponent {
                    hardened: false,
                    value: *index,
                },
                ChildNumber::Hardened { index } => PathComponent {
                    hardened: true,
                    value: *index,
                },
            })
            .collect(),
    }
}

/// An `@N` whose card origin the toolkit chose (Brian's ruling of 2026-11-26,
/// `tk-bh1125-decision-descriptor-vs-template-multisig`).
pub(crate) struct ChosenSlot {
    pub idx: u8,
    /// The key's master fingerprint, for the inline `[fp/path]@N` form.
    pub fingerprint: Fingerprint,
    /// A seed slot: its key moves with its path (the m/45' alternative).
    pub seed: bool,
    /// `--slot @N.path=` is refused for this slot, so only the inline form
    /// states its path.
    pub inline_only: bool,
}

/// Why the toolkit chose the origin.
pub(crate) enum ChosenWhy {
    /// A canonical shape's canonical origin (bug hunt 2026-11-25 toolkit#1).
    Canonical,
    /// A non-canonical shape's family default (v0.19.0 SPEC §4.12.b, Brian's
    /// 2026-11-25 BIP-defaults ruling). `bare_sh`: a legacy bare P2SH multisig,
    /// for which BIP-48 assigns no path.
    Family {
        family: crate::template::DefaultOriginFamily,
        bare_sh: bool,
    },
}

/// `[fp/path]` for `chosen` (the descriptor's inline origin form).
fn inline_origin_text(fingerprint: Fingerprint, path: &DerivationPath) -> String {
    let fp = fingerprint.to_string().to_lowercase();
    if path.is_empty() {
        format!("[{fp}]")
    } else {
        format!("[{fp}/{path}]")
    }
}

/// Write the inline origin `[fp/path]` in front of every `@idx` placeholder of
/// `desc`, replacing an inline origin already there (a fingerprint-only one).
pub(crate) fn rewrite_inline_origin(desc: &str, idx: u8, origin: &str) -> String {
    let needle = format!("@{idx}");
    let bytes = desc.as_bytes();
    let mut out = String::with_capacity(desc.len() + 32);
    let mut last = 0;
    let mut from = 0;
    while let Some(off) = desc[from..].find(&needle) {
        let at = from + off;
        let after = at + needle.len();
        from = after;
        if bytes.get(after).is_some_and(|b| b.is_ascii_digit()) {
            continue; // `@1` inside `@12`
        }
        let start = if at > 0 && bytes[at - 1] == b']' {
            desc[..at].rfind('[').unwrap_or(at)
        } else {
            at
        };
        out.push_str(&desc[last..start]);
        out.push_str(origin);
        out.push_str(&needle);
        last = after;
    }
    out.push_str(&desc[last..]);
    out
}

/// One printed command line that states `path_for(slot)` for every chosen
/// slot: `--slot @N.path=` where the grammar takes it, inline `[fp/path]@N` in
/// `--descriptor` otherwise. `Err(forms)`: the inline forms to write by hand,
/// when the descriptor comes from `--descriptor-file`.
fn explicit_origin_command(
    chosen: &[ChosenSlot],
    path_for: &dyn Fn(&ChosenSlot) -> DerivationPath,
    from_file: bool,
) -> (Option<(String, bool)>, Vec<String>) {
    let mut extra: Vec<String> = Vec::new();
    let mut inline: Vec<(u8, String)> = Vec::new();
    for c in chosen {
        let p = path_for(c);
        if c.inline_only {
            inline.push((c.idx, inline_origin_text(c.fingerprint, &p)));
        } else {
            extra.push("--slot".into());
            extra.push(format!("@{}.path=m/{p}", c.idx));
        }
    }
    if from_file && !inline.is_empty() {
        let forms = inline
            .iter()
            .map(|(i, o)| format!("{o}@{i}"))
            .collect::<Vec<_>>();
        let cmd = (!extra.is_empty()).then(|| rerun_command_with(&extra, None));
        return (cmd, forms);
    }
    let rewrite = |d: &str| {
        inline.iter().fold(d.to_string(), |acc, (i, o)| {
            rewrite_inline_origin(&acc, *i, o)
        })
    };
    let cmd = rerun_command_with(&extra, (!inline.is_empty()).then_some(&rewrite as _));
    (Some(cmd), Vec::new())
}

/// Write `cmd` (and the inline forms for a descriptor file) after `lead`.
fn write_explicit<E: Write>(
    stderr: &mut E,
    lead: &str,
    cmd: Option<&(String, bool)>,
    forms: &[String],
    redacted: &mut bool,
) -> Result<(), ToolkitError> {
    let w = |e: std::io::Error| ToolkitError::BadInput(format!("stderr write: {e}"));
    if !forms.is_empty() {
        let tail = if cmd.is_some() { ", and run:" } else { "." };
        writeln!(
            stderr,
            "warning: {lead} write {} in the descriptor file{tail}",
            forms.join(", ")
        )
        .map_err(w)?;
    } else {
        writeln!(stderr, "warning: {lead} run:").map_err(w)?;
    }
    if let Some((c, r)) = cmd {
        writeln!(stderr, "  {c}").map_err(w)?;
        *redacted |= *r;
    }
    Ok(())
}

const REDACTED_NOTE: &str = "warning: secret values typed on the command line are not repeated \
     above; put yours back in place of each <...>, or pass one as =- on stdin.";

/// Brian's ruling of 2026-11-26 (`tk-bh1125-decision-descriptor-vs-template-
/// multisig`): descriptor mode recorded `path` for the `chosen` slots, a path
/// the user did not supply. One warning names the slots and the path and asks
/// for verification; the command that states the path explicitly follows
/// (`--slot @N.path=`, or inline `[fp/path]@N` where the slot grammar refuses
/// that). For a legacy bare-P2SH `sh(...)` (BIP-87 default, v0.19.0 SPEC
/// §4.12.d) the warning also lists where other wallets put those keys, and a
/// second command mints the same bundle at `m/45'`. Returns whether anything
/// was chosen (the caller exits 4).
fn emit_chosen_origin_warning<E: Write>(
    stderr: &mut E,
    chosen: &[ChosenSlot],
    path: &DerivationPath,
    why: ChosenWhy,
    from_file: bool,
) -> Result<bool, ToolkitError> {
    if chosen.is_empty() {
        return Ok(false);
    }
    let w = |e: std::io::Error| ToolkitError::BadInput(format!("stderr write: {e}"));
    let idx_list = chosen
        .iter()
        .map(|c| format!("@{}", c.idx))
        .collect::<Vec<_>>()
        .join(",");
    let verify = "Verify that your wallet keeps these keys there (compare the first address \
                  with it) before trusting the cards.";
    let bare_sh = matches!(why, ChosenWhy::Family { bare_sh: true, .. });
    let line = match &why {
        ChosenWhy::Canonical => format!(
            "warning: no origin was given for {idx_list}; the toolkit chose m/{path} (this \
             descriptor shape's canonical origin) and the cards record it. {verify}"
        ),
        ChosenWhy::Family {
            family,
            bare_sh: true,
        } => format!(
            "warning: no origin was given for {idx_list} in a legacy P2SH descriptor (sh(...)); \
             the toolkit chose m/{path} ({}) and the cards record it. BIP-48 assigns no path \
             to this script type, and other wallets place legacy P2SH multisig keys elsewhere \
             (for example m/45', m/45'/0, m/45'/0'/0'). {verify} For a second wallet from the \
             same seed, use a different --account.",
            family.label()
        ),
        ChosenWhy::Family { family, .. } => {
            let mut l = format!(
                "warning: no origin was given for {idx_list}; the toolkit chose m/{path} ({}) \
                 and the cards record it. {verify}",
                family.label()
            );
            // Review r2 M5: m/87'/c'/a' is also the default of the multisig
            // templates and of every other bare-sh / tr descriptor, so one seed
            // at one account would reuse one key across script types, which
            // BIP-87 asks wallets to avoid ("the wallet must increment the
            // `account'` level. This prevents key reuse - across ECDSA and
            // Schnorr signatures, across different script types").
            if matches!(family, crate::template::DefaultOriginFamily::Bip87) {
                l.push_str(
                    " The same BIP-87 key serves every script type at one account: use a \
                     different --account for each wallet from this seed.",
                );
            }
            l
        }
    };
    writeln!(stderr, "{line}").map_err(w)?;

    let mut redacted = false;
    let (cmd, forms) = explicit_origin_command(chosen, &|_| path.clone(), from_file);
    write_explicit(
        stderr,
        "to state this path explicitly,",
        cmd.as_ref(),
        &forms,
        &mut redacted,
    )?;

    if bare_sh && chosen.iter().any(|c| c.seed) {
        // A seed slot's key moves with its path; an xpub slot's does not, so
        // it keeps the path it was minted at.
        let m45: DerivationPath =
            vec![bitcoin::bip32::ChildNumber::from_hardened_idx(45).expect("below 2^31")].into();
        let (cmd, forms) = explicit_origin_command(
            chosen,
            &|c| if c.seed { m45.clone() } else { path.clone() },
            from_file,
        );
        write_explicit(
            stderr,
            "to mint this bundle at m/45' instead,",
            cmd.as_ref(),
            &forms,
            &mut redacted,
        )?;
        // Review r2 M3: m/45' has no coin or account level, and verify-bundle
        // rebuilds the card from the same slot set.
        writeln!(
            stderr,
            "warning: m/45' has no coin or account level (every --account and network gives the \
             same keys); pass the same m/45' origins to verify-bundle."
        )
        .map_err(w)?;
    }
    if redacted {
        writeln!(stderr, "{REDACTED_NOTE}").map_err(w)?;
    }
    Ok(true)
}

/// Brian's ruling of 2026-11-26: template mode, a multisig template, no
/// `--multisig-path-family`: the `defaulted` slots were given the BIP-87
/// default (the family default), which the user did not choose. Returns
/// whether anything was chosen (the caller exits 4).
fn emit_template_family_warning<E: Write>(
    stderr: &mut E,
    defaulted: &[u8],
    template: CliTemplate,
    network: CliNetwork,
    account: u32,
) -> Result<bool, ToolkitError> {
    if defaulted.is_empty() {
        return Ok(false);
    }
    let w = |e: std::io::Error| ToolkitError::BadInput(format!("stderr write: {e}"));
    let family = MultisigPathFamily::default();
    let path =
        family.default_origin_path(network, account, template.bip48_script_type().unwrap_or(0));
    let idx_list = defaulted
        .iter()
        .map(|i| format!("@{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let label = match family {
        MultisigPathFamily::Bip87 => "BIP-87",
        MultisigPathFamily::Bip48 => "BIP-48",
    };
    writeln!(
        stderr,
        "warning: --multisig-path-family was not given, so the toolkit chose {label} ({path}) \
         for {idx_list} and the cards record it. Verify that your wallet keeps these keys there \
         (compare the first address with it) before trusting the cards."
    )
    .map_err(w)?;
    let (cmd, redacted) = rerun_command_with(
        &[
            "--multisig-path-family".to_string(),
            family.human_name().to_string(),
        ],
        None,
    );
    writeln!(stderr, "warning: to state this choice explicitly, run:").map_err(w)?;
    writeln!(stderr, "  {cmd}").map_err(w)?;
    if redacted {
        writeln!(stderr, "{REDACTED_NOTE}").map_err(w)?;
    }
    Ok(true)
}

/// This process's own command line, as `mnemonic ...`, with `extra` appended
/// and the `--descriptor` value passed through `descriptor`, quoted for a
/// POSIX shell. Secret values typed inline are replaced by a `<...>`
/// placeholder and never echoed (see [`redact_and_quote`]); the `bool` says
/// whether any was.
pub(crate) fn rerun_command_with(
    extra: &[String],
    descriptor: Option<&dyn Fn(&str) -> String>,
) -> (String, bool) {
    let argv: Vec<String> = std::env::args_os()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    redact_and_quote(&argv, extra, descriptor)
}

/// Slot subkeys whose values are public and may be repeated in a printed
/// command. Fail-closed: every other subkey's inline value (a phrase, seedqr,
/// entropy, ms1, wif or xprv, or any subkey added later) is redacted.
const PUBLIC_SLOT_SUBKEYS: &[&str] = &["xpub", "master_xpub", "fingerprint", "path"];

/// A value that names where a secret comes from instead of holding it: stdin
/// (`-`) or an environment variable (`@env:VAR`). Repeated as typed.
fn is_secret_reference(value: &str) -> bool {
    value == "-" || value.starts_with("@env:")
}

fn redact_slot_value(v: &str, redacted: &mut bool) -> String {
    // `@N.<subkey>=<value>`
    if let Some((key, value)) = v.split_once('=') {
        let public = key
            .split_once('.')
            .is_some_and(|(_, sub)| PUBLIC_SLOT_SUBKEYS.contains(&sub));
        if public || is_secret_reference(value) {
            return v.to_string();
        }
        *redacted = true;
        return format!("{key}=<your {key}>");
    }
    v.to_string()
}

/// `restore --from <node>=<value>`: every node there is a seed.
fn redact_from_value(v: &str, redacted: &mut bool) -> String {
    match v.split_once('=') {
        Some((_, value)) if is_secret_reference(value) => v.to_string(),
        Some((node, _)) => {
            *redacted = true;
            format!("{node}=<your {node}>")
        }
        None => v.to_string(),
    }
}

/// `argv` (argv[0] printed as `mnemonic`) with `extra` appended, quoted for a
/// POSIX shell. Secret values typed inline (`--slot @N.<phrase|seedqr|entropy|
/// ms1|wif|xprv>=<value>`, `--from <node>=<value>` and `--passphrase <value>`)
/// are replaced by a `<...>` placeholder and never echoed; `-` and `@env:VAR`
/// are kept. `descriptor` rewrites the `--descriptor` value.
pub(crate) fn redact_and_quote(
    argv: &[String],
    extra: &[String],
    descriptor: Option<&dyn Fn(&str) -> String>,
) -> (String, bool) {
    let mut redacted = false;
    let mut out: Vec<String> = Vec::with_capacity(argv.len() + extra.len());
    out.push("mnemonic".into());
    let mut i = 1;
    while i < argv.len() {
        let a = &argv[i];
        let next = argv.get(i + 1);
        match (a.as_str(), next) {
            ("--slot", Some(v)) => {
                out.push(a.clone());
                out.push(redact_slot_value(v, &mut redacted));
                i += 2;
                continue;
            }
            ("--from", Some(v)) => {
                out.push(a.clone());
                out.push(redact_from_value(v, &mut redacted));
                i += 2;
                continue;
            }
            ("--passphrase", Some(v)) => {
                out.push(a.clone());
                if is_secret_reference(v) {
                    out.push(v.clone());
                } else {
                    out.push("<your passphrase>".into());
                    redacted = true;
                }
                i += 2;
                continue;
            }
            ("--descriptor", Some(v)) => {
                out.push(a.clone());
                out.push(descriptor.map_or_else(|| v.clone(), |f| f(v)));
                i += 2;
                continue;
            }
            _ => {}
        }
        if let Some(v) = a.strip_prefix("--slot=") {
            out.push(format!("--slot={}", redact_slot_value(v, &mut redacted)));
        } else if let Some(v) = a.strip_prefix("--from=") {
            out.push(format!("--from={}", redact_from_value(v, &mut redacted)));
        } else if let Some(v) = a.strip_prefix("--passphrase=") {
            if is_secret_reference(v) {
                out.push(a.clone());
            } else {
                out.push("--passphrase=<your passphrase>".into());
                redacted = true;
            }
        } else if let Some(v) = a.strip_prefix("--descriptor=") {
            out.push(format!(
                "--descriptor={}",
                descriptor.map_or_else(|| v.to_string(), |f| f(v))
            ));
        } else {
            out.push(a.clone());
        }
        i += 1;
    }
    out.extend(extra.iter().cloned());
    let quoted: Vec<String> = out.iter().map(|a| shell_quote(a)).collect();
    (quoted.join(" "), redacted)
}

/// Quote one argument for a POSIX shell: bare when it is plainly safe,
/// otherwise single-quoted with each embedded `'` written as `'\''`.
fn shell_quote(a: &str) -> String {
    let safe = !a.is_empty()
        && a.chars()
            .all(|c| c.is_ascii_alphanumeric() || "@%+=:,./-_".contains(c));
    if safe {
        a.to_string()
    } else {
        format!("'{}'", a.replace('\'', "'\\''"))
    }
}

/// Convert a md-codec OriginPath to bitcoin::bip32::DerivationPath. Required
/// because the resolved descriptor placeholder carries the path in md-codec
/// shape but the binding logic operates on bitcoin types. Public for
/// symmetric verify-bundle (§4.11.c) which mirrors the descriptor-mode
/// binding loop and needs the same conversion.
pub fn origin_to_derivation_path(
    op: &md_codec::origin_path::OriginPath,
) -> Result<DerivationPath, ToolkitError> {
    let s = if op.components.is_empty() {
        "m".to_string()
    } else {
        let mut s = String::from("m");
        for c in &op.components {
            s.push('/');
            s.push_str(&c.value.to_string());
            if c.hardened {
                s.push('\'');
            }
        }
        s
    };
    DerivationPath::from_str(&s).map_err(|e| {
        ToolkitError::DescriptorParse(format!("descriptor @N annotation path parse failed: {e}"))
    })
}

pub fn self_check_bundle(
    bundle: &Bundle,
    args: &BundleArgs,
    expected_entropy: &[Option<&[u8]>],
) -> Result<(), ToolkitError> {
    // md1 decode.
    let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
    let desc =
        md_codec::chunk::reassemble(&md1_strs).map_err(|e| ToolkitError::BundleMismatch {
            card: "self-check[md1_decode]".into(),
            message: format!("{:?}", e),
        })?;

    // #28 phase 1 — a keyless single-sig template md1 (`--md1-form=template`)
    // is keyless by construction, so the `is_wallet_policy` gate, the
    // pubkeys-absent refusal, and `check_mk1_xpub_binding` (which binds mk1
    // xpubs against the md1's pubkeys TLV) are STRUCTURALLY inapplicable. The
    // surviving checks — stub-coherence (rooted on the template id), mk1
    // origin/fingerprint, and ms1 parity — still apply. We discriminate on
    // `is_wallet_policy()` of the reassembled md1, the same discriminator
    // `bundle_binding_stub` uses, so the expected stub matches what
    // `synthesize` emitted.
    let template_form = !desc.is_wallet_policy();

    let expected_stub: [u8; 4] =
        bundle_binding_stub(&desc).ok_or_else(|| ToolkitError::BundleMismatch {
            card: "self-check[stub_linkage]".into(),
            message: "binding-id compute failed".into(),
        })?;

    // Audit M1 — slot-exact mk1-xpub ↔ descriptor binding (defense-in-depth).
    // Every keyed call site reaches here via synthesize_descriptor/
    // synthesize_unified, which build tlv.pubkeys AND the per-cosigner cards
    // from the SAME slots vector in the same order — so card @i must carry
    // EXACTLY the descriptor's @i pubkey. Order-agnostic membership would miss
    // a cross-slot card swap (it passes membership + the stub-equality check
    // below). For the keyless TEMPLATE form there are no md1 pubkeys to bind
    // against (the seed derives the key at restore) — skip this check.
    let pubkeys: Option<&[(u8, [u8; 65])]> = if template_form {
        None
    } else {
        let Some(pk) = desc.tlv.pubkeys.as_deref() else {
            return Err(ToolkitError::BundleMismatch {
                card: "self-check[mk1_xpub_binding]".into(),
                message: "descriptor tlv.pubkeys is absent — cannot bind mk1 xpubs to the policy"
                    .into(),
            });
        };
        Some(pk)
    };

    match &bundle.mk1 {
        MkField::Single(mk1) => {
            let mk1_strs: Vec<&str> = mk1.iter().map(|s| s.as_str()).collect();
            let card = mk_codec::decode(&mk1_strs).map_err(|e| ToolkitError::BundleMismatch {
                card: "self-check[mk1_decode]".into(),
                message: format!("{:?}", e),
            })?;
            if let Some(pubkeys) = pubkeys {
                check_mk1_xpub_binding(pubkeys, 0, &card, "self-check[mk1_xpub_binding]".into())?;
            }
            if !card.policy_id_stubs.iter().any(|s| *s == expected_stub) {
                return Err(ToolkitError::BundleMismatch {
                    card: "self-check[stub_linkage]".into(),
                    message: "mk1 policy_id_stubs do not include descriptor's stub".into(),
                });
            }
            if !args.privacy_preserving && card.origin_fingerprint.is_none() {
                return Err(ToolkitError::BundleMismatch {
                    card: "self-check[mk1_fingerprint_match]".into(),
                    message: "mk1 missing origin_fingerprint but --privacy-preserving not set"
                        .into(),
                });
            }
            if args.privacy_preserving && card.origin_fingerprint.is_some() {
                return Err(ToolkitError::BundleMismatch {
                    card: "self-check[mk1_fingerprint_match]".into(),
                    message: "mk1 has origin_fingerprint but --privacy-preserving was set".into(),
                });
            }
        }
        MkField::Multi(per_cosigner) => {
            // Decode each card-set; verify all share the same stubs list.
            let mut decoded_cards: Vec<mk_codec::KeyCard> = Vec::with_capacity(per_cosigner.len());
            for (i, chunks) in per_cosigner.iter().enumerate() {
                let strs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
                let card = mk_codec::decode(&strs).map_err(|e| ToolkitError::BundleMismatch {
                    card: format!("self-check[mk1_decode[{}]]", i),
                    message: format!("{:?}", e),
                })?;
                decoded_cards.push(card);
            }
            // Audit M1 — slot-exact binding per cosigner card (see above). The
            // keyless template form is single-sig (MkField::Single), so this
            // Multi arm only runs for keyed multisig where pubkeys is Some;
            // skip defensively if absent (template form never reaches here).
            if let Some(pubkeys) = pubkeys {
                for (i, c) in decoded_cards.iter().enumerate() {
                    check_mk1_xpub_binding(
                        pubkeys,
                        i,
                        c,
                        format!("self-check[mk1_xpub_binding[{i}]]"),
                    )?;
                }
            }
            let first_stubs = &decoded_cards[0].policy_id_stubs;
            for (i, c) in decoded_cards.iter().enumerate().skip(1) {
                if &c.policy_id_stubs != first_stubs {
                    return Err(ToolkitError::BundleMismatch {
                        card: format!("self-check[stub_linkage[{}]]", i),
                        message: "policy_id_stubs differ across cosigner cards".into(),
                    });
                }
            }
            if !first_stubs.iter().any(|s| *s == expected_stub) {
                return Err(ToolkitError::BundleMismatch {
                    card: "self-check[stub_linkage]".into(),
                    message: "mk1 policy_id_stubs do not include descriptor's stub".into(),
                });
            }
        }
    }

    // ms1 per-slot validation (FOLLOWUP self-check-ms1-iteration). The oracle
    // is `expected_entropy[i] = resolved_slots[i].entropy` (the EXACT predicate
    // that drove emission at synthesize.rs::synthesize_descriptor) — NOT
    // `args.slot`, which records what the user supplied (import-json envelopes
    // and wif slots would false-trip). For each slot: (a) emptiness parity —
    // `ms1[i]` non-empty iff the slot is entropy-bearing (catches the @0-only
    // emission reversion); (b) for entropy-bearing slots, the emitted ms1 must
    // decode AND round-trip to the slot's source entropy.
    if expected_entropy.len() != bundle.ms1.len() {
        return Err(ToolkitError::BundleMismatch {
            card: "self-check[ms1_length_mismatch]".into(),
            message: format!(
                "expected_entropy len {} != ms1 len {}",
                expected_entropy.len(),
                bundle.ms1.len()
            ),
        });
    }
    for (i, ms) in bundle.ms1.iter().enumerate() {
        let expected = expected_entropy[i];
        if ms.is_empty() != expected.is_none() {
            return Err(ToolkitError::BundleMismatch {
                card: format!("self-check[ms1_parity[{i}]]"),
                message: format!(
                    "ms1[{i}] is {} but slot {i} is {}",
                    if ms.is_empty() { "empty" } else { "populated" },
                    if expected.is_none() {
                        "watch-only (no source entropy)"
                    } else {
                        "entropy-bearing"
                    }
                ),
            });
        }
        let Some(expected_bytes) = expected else {
            continue; // watch-only "" sentinel — correct, nothing to decode.
        };
        let (_tag, payload) = ms_codec::decode(ms).map_err(|e| ToolkitError::BundleMismatch {
            card: format!("self-check[ms1_decode[{i}]]"),
            message: format!("{e:?}"),
        })?;
        // wave2 T2 Site A: move the decoded master-seed entropy OUT of the bare
        // `Payload` husk into a fn-local `Zeroizing<Vec<u8>>` so it scrubs at
        // the end of this iteration (was dropped un-scrubbed). The `_` arm
        // covers `Payload`'s `#[non_exhaustive]` future variants.
        let self_check_entropy: zeroize::Zeroizing<Vec<u8>> = match payload {
            ms_codec::Payload::Entr(b) => zeroize::Zeroizing::new(b),
            ms_codec::Payload::Mnem { entropy, .. } => zeroize::Zeroizing::new(entropy),
            ref other => zeroize::Zeroizing::new(other.as_bytes().to_vec()),
        };
        if self_check_entropy.as_slice() != expected_bytes {
            return Err(ToolkitError::BundleMismatch {
                card: format!("self-check[ms1_entropy[{i}]]"),
                message: format!(
                    "ms1[{i}] decodes to entropy that does not match the slot's source entropy"
                ),
            });
        }
    }

    Ok(())
}

/// Audit M1 — assert the decoded mk1 card at `slot` carries EXACTLY the
/// descriptor's `@slot` pubkey (65-byte chain_code‖compressed-pubkey form,
/// the same comparison verify_bundle's cosigner mapping uses). The failure
/// message names the slot + a truncated xpub (public material).
fn check_mk1_xpub_binding(
    pubkeys: &[(u8, [u8; 65])],
    slot: usize,
    card: &mk_codec::KeyCard,
    card_label: String,
) -> Result<(), ToolkitError> {
    let want = crate::synthesize::xpub_to_65(&card.xpub);
    if !pubkeys
        .iter()
        .any(|(s, b)| *s as usize == slot && *b == want)
    {
        let xpub_str = card.xpub.to_string();
        let trunc = &xpub_str[..xpub_str.len().min(12)];
        return Err(ToolkitError::BundleMismatch {
            card: card_label,
            message: format!(
                "mk1 card @{slot} carries xpub {trunc}… which is not the descriptor's @{slot} pubkey"
            ),
        });
    }
    Ok(())
}

// ============================================================================
// SPEC v0.9.0 §1 item 1 — argv-leakage closure helpers
// ============================================================================

/// Per-occurrence `secret-in-argv` stderr advisory emission. One advisory
/// per inline-secret site (per (flag, slot-index) pair) so the user sees
/// every leak site, not just the first. Emits to stderr only — no
/// argv-leakage data is logged or persisted.
fn emit_secret_in_argv_advisories<E: std::io::Write>(args: &BundleArgs, stderr: &mut E) {
    use crate::secret_advisory::secret_in_argv_warning;
    for s in &args.slot {
        if s.subkey.is_secret_bearing() && !s.is_stdin_sentinel() && !s.value.starts_with("@env:") {
            let flag = format!("--slot @{}.{}=", s.index, s.subkey.as_str());
            let alt = format!("--slot @{}.{}=-", s.index, s.subkey.as_str());
            secret_in_argv_warning(stderr, &flag, &alt);
        }
    }
    // F-687: one note for a literal argv passphrase; none for `-` (stdin)
    // or `@env:VAR`.
    crate::passphrase_input::emit_argv_note(
        args.passphrase.as_deref(),
        args.passphrase_stdin,
        stderr,
    );
}

/// Does the current invocation require stdin consumption for slot_stdin
/// or passphrase_stdin? Returns false when no stdin work is needed,
/// letting `run()` skip the clone-into-synthetic step.
fn needs_stdin_substitution(pp_reads_stdin: bool, slot_stdin_idxs: &[usize]) -> bool {
    pp_reads_stdin || !slot_stdin_idxs.is_empty()
}

/// v0.26.0 §3 — cheap pre-check for `@env:` sentinels on `bundle`'s
/// secret-bearing flag surfaces (`--passphrase`, secret-bearing `--slot`).
fn needs_env_sentinel_resolution(args: &BundleArgs) -> bool {
    let pp = args
        .passphrase
        .as_deref()
        .map(|v| v.starts_with("@env:"))
        .unwrap_or(false);
    let slot = args
        .slot
        .iter()
        .any(|s| s.subkey.is_secret_bearing() && s.value.starts_with("@env:"));
    pp || slot
}

/// v0.26.0 §3 — resolve `@env:<VAR>` sentinels across `bundle`'s
/// secret-bearing flag surfaces. Non-secret slot subkeys are NOT resolved
/// per SPEC §3.2 (opt-in per-callsite).
fn resolve_env_sentinels(args: &BundleArgs) -> Result<BundleArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    if let Some(pp) = owned.passphrase.as_ref() {
        // F-687: only `@env:` resolves here; `-` is stdin (substituted later).
        if pp.starts_with("@env:") {
            owned.passphrase = Some(crate::passphrase_input::resolve_env(pp)?);
        }
    }
    for s in owned.slot.iter_mut() {
        if s.subkey.is_secret_bearing() {
            let flag = format!("--slot @{}.{}=", s.index, s.subkey.as_str());
            // cycle-14 (L22): the @env: sentinel resolves to the ACTUAL secret
            // phrase — re-wrap into SecretString so it scrubs on drop.
            s.value =
                crate::secret_string::SecretString::new(resolve_env_var_sentinel(&s.value, &flag)?);
        }
    }
    Ok(owned)
}

/// Clone `args` into an owned `BundleArgs` and apply the stdin
/// substitution(s) (single-stdin-per-invocation: at most one of
/// `--passphrase-stdin` OR `--slot @N.<secret>=-` may be present).
/// `pp_reads_stdin` is decided from `--passphrase` AS WRITTEN (F-687: `-` or
/// `--passphrase-stdin`), before `@env:` resolution — a variable whose value
/// is `-` is that literal passphrase, not a request to read stdin.
fn apply_stdin_substitutions(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    pp_reads_stdin: bool,
    slot_stdin_idxs: &[usize],
) -> Result<BundleArgs, ToolkitError> {
    let mut owned = args.clone();
    // bh-tk-I5 (R-1): the stdin slots were identified from the values as
    // written, before `@env:` resolution.
    let has_slot_stdin = !slot_stdin_idxs.is_empty();
    if pp_reads_stdin && has_slot_stdin {
        return Err(ToolkitError::BadInput(format!(
            "{} cannot be used with --slot @N.<secret>=- (single stdin per invocation)",
            crate::passphrase_input::stdin_spelling(owned.passphrase_stdin)
        )));
    }
    if pp_reads_stdin {
        owned.passphrase = Some(crate::passphrase_input::read_stdin_passphrase(stdin)?);
    } else if has_slot_stdin {
        crate::slot_input::apply_slot_stdin_at(&mut owned.slot, slot_stdin_idxs, stdin)?;
    }
    Ok(owned)
}

#[cfg(test)]
mod self_check_ms1_tests {
    use super::*;

    // All-zero-entropy 24-word vector (mirrors synthesize.rs tests).
    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    fn minimal_bundle_args() -> BundleArgs {
        BundleArgs {
            network: CliNetwork::Mainnet,
            template: None,
            descriptor: None,
            descriptor_file: None,
            language: None,
            passphrase: None,
            passphrase_stdin: false,
            account: 0,
            json: false,
            no_engraving_card: true,
            group_size: 5,
            separator: ' ',
            multisig_path_family: None,
            privacy_preserving: false,
            self_check: false,
            threshold: None,
            slot: vec![],
            md1_form: crate::synthesize::Md1Form::Policy,
            import_json: None,
            import_json_index: None,
        }
    }

    /// A real 2-of-3 self-multisig bundle (md1/mk1/ms1 all valid + decodable).
    /// A SECRET-BEARING 2-of-3 from one seed at accounts 0..3, built through
    /// `synthesize_unified` — the PRODUCTION path.
    ///
    /// Was `synthesize_multisig_full`, now deleted: it was `pub`, called from
    /// nothing outside `#[cfg(test)]`, and it replicated one xpub across every
    /// slot — a degenerate `multi(k, K, K, K)` one signer satisfies k times.
    ///
    /// NOT `synthesize_multisig_watch_only`, which was the obvious swap and is
    /// wrong here: the ms1 self-check cells below assert a NON-EMPTY ms1 per
    /// cosigner, and the watch-only helper emits empty sentinels. Routing
    /// through `synthesize_unified` keeps the secret half and has the side
    /// benefit that this fixture now exercises the path the CLI actually takes.
    fn multisig_bundle() -> Bundle {
        use crate::synthesize::{synthesize_unified, Md1Form, ResolvedSlot};
        use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
        use bitcoin::secp256k1::Secp256k1;
        use std::str::FromStr;
        let m = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let entropy = m.to_entropy();
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &m.to_seed("")).unwrap();
        let slots: Vec<ResolvedSlot> = (0..3u32)
            .map(|acct| {
                let path = DerivationPath::from_str(&format!("48'/0'/{acct}'/2'")).unwrap();
                ResolvedSlot {
                    xpub: Xpub::from_priv(&secp, &master.derive_priv(&secp, &path).unwrap()),
                    fingerprint: master.fingerprint(&secp),
                    path,
                    entropy: Some(zeroize::Zeroizing::new(entropy.clone())),
                    master_xpub: None,
                    language: None,
                    _entropy_pin: None,
                }
            })
            .collect();
        synthesize_unified(
            &slots,
            CliTemplate::WshSortedMulti,
            2,
            CliNetwork::Mainnet,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap()
    }

    /// RED→GREEN: self-check must DETECT a regressed ms1 emission (the @0-only
    /// reversion: ms1[0] populated, ms1[1+] wrongly cleared). RED against the
    /// pre-fix self_check_bundle (which ignores ms1 → returns Ok).
    /// The 3 cosigners' source entropy (self-multisig → all the TREZOR_24 seed).
    fn expected_entropy_bytes() -> Vec<u8> {
        bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24)
            .unwrap()
            .to_entropy()
    }

    #[test]
    fn self_check_detects_at0_only_ms1_regression() {
        let args = minimal_bundle_args();
        let e = expected_entropy_bytes();
        let expected: Vec<Option<&[u8]>> = vec![Some(&e[..]), Some(&e[..]), Some(&e[..])];

        // Sanity: a correct full-mode multisig bundle self-checks Ok.
        let good = multisig_bundle();
        assert_eq!(good.ms1.len(), 3);
        assert!(
            good.ms1.iter().all(|s| !s.is_empty()),
            "fixture must emit a non-empty ms1 per cosigner"
        );
        assert!(
            self_check_bundle(&good, &args, &expected).is_ok(),
            "a correct full-mode multisig must pass self-check"
        );

        // Regress: clear ms1[1] (the @0-only emission reversion).
        let mut bad = multisig_bundle();
        bad.ms1[1] = String::new();
        let r = self_check_bundle(&bad, &args, &expected);
        assert!(
            r.is_err(),
            "self-check MUST detect the @0-only ms1 regression (ms1[1] cleared); got {r:?}"
        );
    }

    /// GREEN guard: a watch-only shape (all-empty ms1, no entropy-bearing slots)
    /// must PASS self-check (the check must not be over-eager).
    #[test]
    fn self_check_passes_watch_only_all_empty_ms1() {
        let args = minimal_bundle_args();
        let mut b = multisig_bundle();
        for s in b.ms1.iter_mut() {
            *s = String::new();
        }
        let expected: Vec<Option<&[u8]>> = vec![None, None, None];
        assert!(
            self_check_bundle(&b, &args, &expected).is_ok(),
            "all-empty ms1 with no entropy-bearing slots (watch-only) must pass self-check"
        );
    }

    /// GREEN guard: the entropy round-trip catches a WRONG-entropy ms1 (an ms1
    /// that decodes to a different secret than the slot's source entropy).
    #[test]
    fn self_check_detects_wrong_entropy_ms1() {
        let args = minimal_bundle_args();
        let bundle = multisig_bundle();
        // Claim a DIFFERENT expected entropy than what the ms1 actually encodes.
        let wrong = [0xABu8; 32];
        let expected: Vec<Option<&[u8]>> =
            vec![Some(&wrong[..]), Some(&wrong[..]), Some(&wrong[..])];
        let r = self_check_bundle(&bundle, &args, &expected);
        assert!(
            r.is_err(),
            "self-check MUST detect ms1 that round-trips to the wrong entropy; got {r:?}"
        );
    }

    /// T4b (cycle-14, L22) — the `@env:` write-back resolves the sentinel to
    /// the ACTUAL secret phrase and stores it into `SlotInput.value` (now a
    /// `SecretString`), byte-identically to before. Fences the
    /// `s.value = SecretString::new(resolve_env_var_sentinel(...)?)` re-wrap.
    #[test]
    fn env_sentinel_write_back_into_secret_string() {
        use crate::slot_input::{parse_slot_input, SlotSubkey};
        let var = "CYCLE14_BUNDLE_PHRASE_TEST";
        // SAFETY: single-threaded test; unique var name; removed before return.
        std::env::set_var(var, TREZOR_24);
        let mut args = minimal_bundle_args();
        args.slot = vec![parse_slot_input(&format!("@0.phrase=@env:{var}")).unwrap()];
        let resolved = resolve_env_sentinels(&args).unwrap();
        std::env::remove_var(var);
        assert_eq!(resolved.slot.len(), 1);
        assert_eq!(resolved.slot[0].subkey, SlotSubkey::Phrase);
        // Deref<Target=str>: the resolved value is the env-var phrase verbatim.
        assert_eq!(&*resolved.slot[0].value, TREZOR_24);
        // Debug of the resolved slot must NOT leak the resolved secret.
        let dbg = format!("{:?}", resolved.slot[0]);
        assert!(
            !dbg.contains(TREZOR_24),
            "resolved @env: SlotInput Debug leaked the secret: {dbg}"
        );
    }
}

#[cfg(test)]
mod self_check_mk1_xpub_binding_tests {
    //! Audit M1 — `self_check_bundle` must bind each decoded mk1 card's xpub
    //! to the descriptor's `tlv.pubkeys` SLOT-EXACTLY (card @i ↔ pair
    //! `(i, xpub_to_65)`). v0.47.4 pattern: synthesize-then-mutate (re-encode
    //! an mk1 with a DIFFERENT xpub but the same chunk_set_id so the mutation
    //! is isolated to the xpub bytes). Greens: the existing happy-path / wif
    //! cells in `tests/cli_self_check.rs` discriminate against over-eagerness.

    use super::*;
    use crate::parse::CosignerSpec;
    use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;

    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    const BIP39_TEST_2: &str =
        "legal winner thank year wave sausage worth useful legal winner thank yellow";

    fn minimal_bundle_args() -> BundleArgs {
        BundleArgs {
            network: CliNetwork::Mainnet,
            template: None,
            descriptor: None,
            descriptor_file: None,
            language: None,
            passphrase: None,
            passphrase_stdin: false,
            account: 0,
            json: false,
            no_engraving_card: true,
            group_size: 5,
            separator: ' ',
            multisig_path_family: None,
            privacy_preserving: false,
            self_check: false,
            threshold: None,
            slot: vec![],
            md1_form: crate::synthesize::Md1Form::Policy,
            import_json: None,
            import_json_index: None,
        }
    }

    /// Single-sig full-mode bundle + its source entropy.
    fn single_bundle() -> (Bundle, Vec<u8>) {
        let acc = crate::derive::derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
            0,
        )
        .unwrap();
        let (entropy, fp, xpub, _path) = acc.into_parts();
        let bundle = crate::synthesize::synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        (bundle, entropy)
    }

    /// A 2-of-2 multisig with DISTINCT per-slot xpubs. NB: NOT
    /// the deleted `synthesize_multisig_full` — that was self-multisig (all N
    /// xpubs byte-identical), which would make a cross-slot card swap
    /// undetectable by construction. This note is why that function is gone:
    /// the knowledge lived here, in a helper that routed around it, instead of
    /// being fixed at the source. Watch-only (ms1 = ["", ""]) keeps the fixture
    /// small.
    fn distinct_xpub_multisig_bundle() -> Bundle {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        let cosigner = |phrase: &str| {
            let m = bip39::Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
            let master =
                Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &m.to_seed("")).unwrap();
            CosignerSpec {
                xpub: Xpub::from_priv(&secp, &master.derive_priv(&secp, &path).unwrap()),
                master_fingerprint: master.fingerprint(&secp),
                path: Some(path.clone()),
            }
        };
        crate::synthesize::synthesize_multisig_watch_only(
            &[cosigner(TREZOR_24), cosigner(BIP39_TEST_2)],
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            0,
            MultisigPathFamily::default(),
            false,
        )
        .unwrap()
    }

    /// Re-encode the mk1 card-set `chunks` with its xpub swapped for a
    /// FOREIGN one (derived from an unrelated seed at the card's own
    /// origin_path, so depth/child_number stay consistent with the
    /// origin-path round-trip invariant) keeping the SAME chunk_set_id
    /// (`derive_mk1_chunk_set_id_for_slot(stub, slot)` — what synthesis
    /// used), so the mutation is isolated to the xpub bytes.
    fn reencode_with_foreign_xpub(chunks: &[String], slot: u32) -> Vec<String> {
        let strs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let card = mk_codec::decode(&strs).unwrap();
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &[7u8; 32]).unwrap();
        let foreign = Xpub::from_priv(
            &secp,
            &master.derive_priv(&secp, &card.origin_path).unwrap(),
        );
        assert_ne!(
            crate::synthesize::xpub_to_65(&foreign),
            crate::synthesize::xpub_to_65(&card.xpub),
            "fixture bug: foreign xpub must differ from the card's"
        );
        let csi =
            crate::synthesize::derive_mk1_chunk_set_id_for_slot(&card.policy_id_stubs[0], slot);
        let mutated = mk_codec::KeyCard::new(
            card.policy_id_stubs.clone(),
            card.origin_fingerprint,
            card.origin_path.clone(),
            foreign,
        );
        mk_codec::encode_with_chunk_set_id(&mutated, csi).unwrap()
    }

    /// Assert the self-check failure is the xpub-binding one (right-reason
    /// guard: a decode or stub failure would carry a different card label).
    fn assert_binding_failure(r: Result<(), ToolkitError>, expect_label: &str) {
        match r {
            Err(ToolkitError::BundleMismatch { ref card, .. }) if card.contains("mk1_xpub_binding") => {
                assert!(
                    card.contains(expect_label),
                    "binding failure must name the slot: card={card}, want {expect_label}"
                );
            }
            other => panic!(
                "self-check MUST fail with the mk1_xpub_binding mismatch ({expect_label}); got {other:?}"
            ),
        }
    }

    /// RED (M1, Single branch): an mk1 re-encoded with a different xpub but
    /// the same chunk_set_id passes md1/stub/fingerprint checks today — the
    /// xpub↔descriptor binding must catch it.
    #[test]
    fn self_check_detects_single_mk1_xpub_descriptor_mismatch() {
        let args = minimal_bundle_args();
        let (good, entropy) = single_bundle();
        let expected: Vec<Option<&[u8]>> = vec![Some(&entropy[..])];
        assert!(
            self_check_bundle(&good, &args, &expected).is_ok(),
            "fixture must pass self-check unmutated"
        );

        let mut bad = good;
        let MkField::Single(ref chunks) = bad.mk1 else {
            panic!("single-sig fixture must emit MkField::Single");
        };
        bad.mk1 = MkField::Single(reencode_with_foreign_xpub(chunks, 0));
        assert_binding_failure(
            self_check_bundle(&bad, &args, &expected),
            "mk1_xpub_binding",
        );
    }

    /// RED (M1, Multi branch): same mutation on one cosigner card of a
    /// distinct-xpub multisig.
    #[test]
    fn self_check_detects_multi_mk1_xpub_descriptor_mismatch() {
        let args = minimal_bundle_args();
        let good = distinct_xpub_multisig_bundle();
        let expected: Vec<Option<&[u8]>> = vec![None, None];
        assert!(
            self_check_bundle(&good, &args, &expected).is_ok(),
            "fixture must pass self-check unmutated"
        );

        let mut bad = good;
        let MkField::Multi(ref mut per_cosigner) = bad.mk1 else {
            panic!("multisig fixture must emit MkField::Multi");
        };
        per_cosigner[1] = reencode_with_foreign_xpub(&per_cosigner[1], 1);
        assert_binding_failure(
            self_check_bundle(&bad, &args, &expected),
            "mk1_xpub_binding[1]",
        );
    }

    /// RED (M1, slot-exactness): swapping two cosigner card-sets keeps every
    /// xpub a MEMBER of tlv.pubkeys and every stubs-list equal — only a
    /// slot-exact pairing detects it. This is the case slot-exactness exists
    /// for; order-agnostic membership would pass.
    #[test]
    fn self_check_detects_multi_cross_slot_card_swap() {
        let args = minimal_bundle_args();
        let expected: Vec<Option<&[u8]>> = vec![None, None];

        let mut bad = distinct_xpub_multisig_bundle();
        let MkField::Multi(ref mut per_cosigner) = bad.mk1 else {
            panic!("multisig fixture must emit MkField::Multi");
        };
        per_cosigner.swap(0, 1);
        assert_binding_failure(
            self_check_bundle(&bad, &args, &expected),
            "mk1_xpub_binding[0]",
        );
    }

    // Brian's BIP-defaults ruling (2026-11-25): the default origin family
    // for a non-canonical descriptor.
    #[test]
    fn default_origin_family_follows_the_bip_defaults_ruling() {
        use crate::template::DefaultOriginFamily as F;
        let tree = |d: &str| parse_descriptor(d, &[], &[]).unwrap().tree;
        let cases = [
            ("sh(wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)))", "m/48'/0'/0'/1'"),
            (
                "wsh(and_v(v:pk(@0/<0;1>/*),pk(@1/<0;1>/*)))",
                "m/48'/0'/0'/2'",
            ),
            ("sh(multi(2,@0/<0;1>/*,@1/<0;1>/*))", "m/87'/0'/0'"),
            ("sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))", "m/87'/0'/0'"),
            ("tr(@0/<0;1>/*,multi_a(1,@1/<0;1>/*))", "m/87'/0'/0'"),
        ];
        for (d, want) in cases {
            assert_eq!(
                F::for_md_tree(&tree(d)).path_string(CliNetwork::Mainnet, 0),
                want,
                "{d}"
            );
        }
        assert_eq!(F::Bip87.path_string(CliNetwork::Testnet, 3), "m/87'/1'/3'");
    }

    #[test]
    fn rerun_command_never_echoes_an_inline_secret() {
        let argv: Vec<String> = [
            "mnemonic",
            "--allow-argv-secret",
            "bundle",
            "--descriptor",
            "sh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
            "--slot",
            "@0.phrase=abandon abandon about",
            "--slot=@1.ms1=ms10secret",
            "--slot",
            "@2.phrase=-",
            "--slot",
            "@3.xpub=xpubPUBLIC",
            "--slot",
            "@3.fingerprint=b8688df1",
            "--slot",
            "@4.newsecret=topsecret",
            "--passphrase",
            "hunter2",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let (cmd, redacted) =
            redact_and_quote(&argv, &["--slot".into(), "@0.path=m/45'".into()], None);
        assert!(redacted);
        for secret in ["abandon", "ms10secret", "hunter2", "topsecret"] {
            assert!(!cmd.contains(secret), "fail-closed: {cmd}");
        }
        assert!(cmd.contains("@2.phrase=-"), "stdin form kept: {cmd}");
        assert!(
            cmd.contains("@3.xpub=xpubPUBLIC"),
            "public value kept: {cmd}"
        );
        assert!(
            cmd.contains("@3.fingerprint=b8688df1"),
            "public value kept: {cmd}"
        );
        assert!(cmd.ends_with("--slot '@0.path=m/45'\\'''"), "{cmd}");
        assert!(
            cmd.contains("'sh(multi(2,@0/<0;1>/*,@1/<0;1>/*))'"),
            "{cmd}"
        );
    }
}

#[cfg(test)]
mod bughunt_1125_wif_network_tests {
    //! Bug hunt 2026-11-25 toolkit#8: a WIF's own network byte must agree with
    //! `--network` (the xpub arm already checked). `bundle` now refuses every
    //! WIF slot (X4) before `resolve_slots`, so this guards the remaining
    //! caller, `verify-bundle`.
    use super::*;
    use crate::slot_input::SlotInput;

    const MAINNET_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";

    fn wif_slot(wif: &str) -> Vec<SlotInput> {
        vec![SlotInput {
            index: 0,
            subkey: SlotSubkey::Wif,
            value: crate::secret_string::SecretString::new(wif.to_string()),
        }]
    }

    fn resolve(network: CliNetwork) -> Result<(), ToolkitError> {
        resolve_slots(
            &wif_slot(MAINNET_WIF),
            CliTemplate::Bip84,
            network,
            0,
            None,
            None,
            MultisigPathFamily::default(),
        )
        .map(|_| ())
    }

    #[test]
    fn mainnet_wif_under_testnet_is_refused() {
        let e = resolve(CliNetwork::Testnet).expect_err("network mismatch must refuse");
        assert!(
            matches!(e, ToolkitError::NetworkMismatch { .. }),
            "got {e:?}"
        );
    }

    #[test]
    fn mainnet_wif_under_mainnet_still_resolves() {
        resolve(CliNetwork::Mainnet).expect("matching network resolves as before");
    }
}
