//! `mnemonic verify-bundle` subcommand.
//!
//! Realizes SPEC §2.2 + §5.4. Both full and watch-only emit the
//! fixed 9-element `checks` array in SPEC §5.4 order; watch-only
//! marks entropy + path-rederivation `skipped` (SPEC §2.2.2). Check
//! failures stay in §5.4 with `result: "mismatch"` per the §5.4
//! routing rule (only pre-decode failures escape to the §5.5 error
//! envelope).

use crate::error::ToolkitError;
use crate::format::{chunk_set_id_extract, VerifyBundleJson, VerifyCheck};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::MultisigPathFamily;
use crate::slot_input::SlotInput;
use crate::template::CliTemplate;
use clap::Args;
use mnemonic_toolkit::mlock::pin_pages_for;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Args, Debug, Clone)]
pub struct VerifyBundleArgs {
    #[arg(long)]
    pub network: CliNetwork,

    /// Template name. Mutually-required-one-of with --descriptor /
    /// --descriptor-file. #28: ALSO optional when `--md1` is present — a keyless
    /// single-sig TEMPLATE md1 carries its own type (the type is read from the
    /// md1 tree); a keyed wallet-policy md1 still requires `--template` (guarded
    /// at runtime with a clear error, not a clap-required panic).
    #[arg(long, required_unless_present_any = ["descriptor", "descriptor_file", "md1", "bundle_json", "extra_strings"])]
    pub template: Option<CliTemplate>,

    /// User-supplied descriptor (v0.3 §5.7 verify-bundle re-parse path).
    #[arg(long, conflicts_with = "descriptor_file")]
    pub descriptor: Option<String>,

    /// User-supplied descriptor file (single-line UTF-8).
    #[arg(long = "descriptor-file")]
    pub descriptor_file: Option<PathBuf>,

    #[arg(long)]
    pub language: Option<CliLanguage>,

    /// BIP-39 mnemonic-extension passphrase used during the
    /// original `mnemonic bundle` emission. Empty (default) is the
    /// common case. Mutually exclusive with `--passphrase-stdin`.
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
    #[arg(long, default_value = "0")]
    pub account: u32,

    /// #28 phase 1 — explicit origin derivation path for verifying + recomposing
    /// a keyless single-sig TEMPLATE bundle (`bundle --md1-form=template`).
    /// Overrides the template's canonical `m/<purpose>'/<coin>'/<account>'`
    /// default with an arbitrary BIP-32 path (e.g. `m/84'/0'/7'`), mirroring
    /// `restore --origin`. Only meaningful for a keyless single-sig template
    /// bundle; ignored otherwise. When supplied, `--expect-wallet-id` is NOT
    /// checked (the canonical-origin id is a different preimage — same carve-out
    /// as `restore`).
    #[arg(long = "origin")]
    pub origin: Option<String>,

    /// #28 phase 1 — expected `WalletPolicyId` (hex prefix) for a keyless
    /// single-sig TEMPLATE bundle. When set, verify-bundle recomputes the
    /// `WalletPolicyId` from the completed (fully-keyed, explicit-origin)
    /// wallet and matches its leading bytes; a mismatch is reported as a failed
    /// check (overall `mismatch`, exit 4). Only meaningful for a template
    /// bundle (`bundle --md1-form=template` output); ignored otherwise. NOT
    /// checked when `--origin` overrides the canonical account path.
    #[arg(long = "expect-wallet-id")]
    pub expect_wallet_id: Option<String>,

    /// P4 — RANGE fallback for the OWN seed's account when the exact account is
    /// unknown: derive the own seed at every account in `0..K` and let the
    /// multisig-template OWN-ACCOUNT SUBSET-SEARCH select the account actually
    /// used (own-only — the `--cosigner` cards must be EXACT; over-supply
    /// cosigners with `--search-cosigner-subset`). NEW on verify-bundle (mirrors
    /// `restore --own-account-max`). Mutually exclusive with `--account` (clap
    /// `conflicts_with` — `--own-account-max K` ALONE passes; the scalar
    /// `--account` default is ignored). `K ≤ 256`. Threaded into the SAME shared
    /// `complete_multisig_template` engine restore uses (verify == restore).
    #[arg(long = "own-account-max", conflicts_with = "account")]
    pub own_account_max: Option<u32>,

    /// P4 — OPT-IN bounded cosigner-subset search. By default (OFF) a multisig
    /// template completion requires the supplied `--cosigner` cards to be EXACT
    /// (own-only — over-supplying cosigners refuses). With this flag the operator
    /// MAY over-supply `--cosigner` cards (unsure which/how many cosigners
    /// belong); the search resolves the correct cosigner subset too. NEW on
    /// verify-bundle (mirrors `restore --search-cosigner-subset`). The space
    /// grows, so a LONGER `--expect-wallet-id` prefix may be needed; bounded by
    /// the §6 hard ceiling + the adaptive time-cap. Mutually exclusive with
    /// `--cosigner @N=`. Threaded into the SAME shared completion engine restore
    /// uses (verify == restore).
    #[arg(long = "search-cosigner-subset")]
    pub search_cosigner_subset: bool,

    /// #28 phase 2 — the operator's OWN seed for completing a keyless multisig
    /// (or general policy) TEMPLATE bundle (`bundle --md1-form=template`, n≥2).
    /// Same grammar and semantics as `restore --from` (an `ms1=`, `phrase=`,
    /// `entropy=` or `seedqr=` source; `@env:VAR` or stdin supported). REQUIRED to
    /// complete a multisig template (the template carries no keys); ignored for a
    /// single-sig template or keyed wallet-policy bundle. The own key is derived
    /// at `--account` (a single own account for verify; multi-own-account
    /// completion is restore-only this cycle) honoring `--origin`.
    #[arg(long = "from")]
    pub from: Option<String>,

    /// #28 phase 2 — an UNASSIGNED cosigner key (`mk1`/xpub) for completing a
    /// keyless MULTISIG / general TEMPLATE bundle; repeat per cosigner card. Same
    /// grammar + semantics as `restore --cosigner`: the bare form is search-
    /// placed; `@N=<mk1|xpub>` assigns it explicitly. The mk1 carries each
    /// cosigner's origin. Only meaningful with `--from` + a keyless multisig
    /// template; ignored otherwise. (Distinct from `--mk1`, which supplies the
    /// engraved template STUB cards the binding check validates.)
    #[arg(long = "cosigner")]
    pub cosigner: Vec<String>,

    /// #28 phase 2 — a known receive (or change) ADDRESS of the wallet; triggers
    /// ADDRESS-SEARCH for a multisig template completion (mirrors
    /// `restore --search-address`). Recommended over `--expect-wallet-id`
    /// (full-scriptPubKey match — collision-free).
    #[arg(long = "search-address")]
    pub search_address: Option<String>,

    /// #28 phase 2 — inclusive lower address index for `--search-address`
    /// (default 0; mirrors `restore`).
    #[arg(long = "search-addr-min", default_value_t = 0)]
    pub search_addr_min: u32,

    /// #28 phase 2 — exclusive upper address index for `--search-address`
    /// (default 20; mirrors `restore`).
    #[arg(long = "search-addr-max", default_value_t = 20)]
    pub search_addr_max: u32,

    /// #28 phase 2 — which BIP-32 change-chain branch(es) `--search-address`
    /// scans: `receive` (0, default), `change` (1), or `both` (mirrors
    /// `restore`).
    #[arg(long = "search-chain", value_enum, default_value_t = crate::cmd::restore::CliSearchChain::Receive)]
    pub search_chain: crate::cmd::restore::CliSearchChain,

    /// #28 phase 2 — override the 1-hour search-time ceiling for a multisig
    /// template completion (mirrors `restore --accept-search-time`). Must be ≥
    /// the printed exhaustive-time estimate (a forced acknowledgment). Accepts a
    /// humantime duration (e.g. `2h`, `90min`).
    #[arg(long = "accept-search-time")]
    pub accept_search_time: Option<String>,

    /// Per-slot `ms1` card(s) to verify. Single-sig: supply once
    /// (`--ms1 <s>`). Multisig: repeat per slot — `--ms1 <s1>
    /// --ms1 <s2>` for full-path. For watch-only cosigners, two
    /// equivalent forms are accepted per SPEC §5.8 (v0.25.1 restored
    /// the empty-string sentinel that v0.24.0 §2.C.1 accidentally
    /// broke): (1) **flag omission** — supply `--ms1` only for the
    /// full-path cosigners; positional vec naturally stops at the
    /// last full-path index (`--ms1 <s0>` skips cosigners 1+). Works
    /// only for trailing cosigners. (2) **empty-string sentinel
    /// `--ms1 ""`** — each `""` value marks the positionally-aligned
    /// cosigner as watch-only; required for middle-cosigner skips
    /// (`--ms1 <s0> --ms1 "" --ms1 <s2>`); emits a one-line stderr
    /// NOTICE per skipped cosigner. Mutually exclusive with
    /// `--bundle-json`.
    #[arg(long, action = clap::ArgAction::Append, conflicts_with = "bundle_json")]
    pub ms1: Vec<String>,

    /// The `mk1` xpub card(s) to verify. Single-sig: one `--mk1`.
    /// Multisig: one `--mk1` per cosigner, in slot order. Mutually
    /// exclusive with `--bundle-json`.
    #[arg(long, num_args = 1.., required_unless_present_any = ["bundle_json", "extra_strings"], conflicts_with = "bundle_json")]
    pub mk1: Vec<String>,

    /// The `md1` wallet-policy card(s) to verify. Single-sig
    /// templates emit one md1; multisig templates emit one md1
    /// total (the policy is shared). Mutually exclusive with
    /// `--bundle-json`.
    #[arg(long, num_args = 1.., required_unless_present_any = ["bundle_json", "extra_strings"], conflicts_with = "bundle_json")]
    pub md1: Vec<String>,

    /// v0.4.3 Phase Q: read supplied ms1/mk1/md1 cards from a JSON envelope
    /// file (the output of `bundle --json`). Mutually exclusive with the
    /// explicit --ms1/--mk1/--md1 triplet. Re-derivation flags (`--slot`)
    /// are STILL required to compute the expected bundle.
    #[arg(long = "bundle-json", conflicts_with_all = ["ms1", "mk1", "md1"])]
    pub bundle_json: Option<PathBuf>,

    /// Emit a single JSON object on stdout instead of the multi-line
    /// `OK / mismatch` text form. The JSON envelope includes
    /// per-slot match details for multisig verifications.
    #[arg(long)]
    pub json: bool,

    /// v0.2 multisig path family (default: bip87).
    #[arg(long = "multisig-path-family", value_enum)]
    pub multisig_path_family: Option<MultisigPathFamily>,

    /// v0.2 privacy mode: expect mk1 omits master fingerprint.
    #[arg(long, default_value = "false")]
    pub privacy_preserving: bool,

    /// v0.2 multisig threshold K (1 ≤ K ≤ N ≤ 16).
    #[arg(long)]
    pub threshold: Option<u8>,

    /// v0.4 unified slot input. Repeating flag — see `BundleArgs::slot`
    /// for grammar.
    #[arg(long = "slot", action = clap::ArgAction::Append, value_parser = crate::slot_input::parse_slot_input)]
    pub slot: Vec<SlotInput>,

    /// v0.24.0 §2.C.1 — positional `<STRING>...` intake. Each value
    /// self-identifies by HRP prefix (`ms1` / `mk1` / `md1`) and is routed
    /// to the same internal storage as the matching typed flag. Unknown
    /// HRPs are rejected with `ToolkitError::UnknownHrp`. Mutually
    /// exclusive with `--bundle-json` (per I3 fold — preserves the
    /// existing `--bundle-json XOR cards-group` mutex).
    #[arg(
        value_name = "STRING",
        num_args = 0..,
        conflicts_with = "bundle_json",
    )]
    pub extra_strings: Vec<String>,
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
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    use crate::cmd::bundle::mode_text;

    // v0.22.1 D18 — TTY-conditional auto-fire. See
    // `crate::repair::resolve_no_auto_repair` (v0.25.0 §2.A D4 fold) for the
    // full public-API contract: `MNEMONIC_FORCE_TTY={0,1}` forces the gate;
    // unset → runtime `is_terminal()` detection.
    let effective_no_auto_repair = crate::repair::resolve_no_auto_repair(no_auto_repair);
    let json_context = args.json;

    // SPEC v0.9.0 §1 item 1 — argv-leakage closure. Run BEFORE bundle-json
    // intake so the advisory fires uniformly even on the synthetic-args
    // intake path. v0.26.0 §I1 fold: emit BEFORE `@env:` sentinel
    // resolution; sentinel-bearing flag values are skipped (user opted
    // into the env-var leak-mitigation channel).
    emit_secret_in_argv_advisories(args, stderr);

    // v0.26.0 §3 — resolve `@env:<VAR>` sentinels before HRP validation
    // + downstream consumption. Owned-args shadowing keeps the diff
    // localized; clones the original `args` only if any sentinel
    // actually needed substitution.
    let env_resolved_owned;
    let args: &VerifyBundleArgs = if needs_env_sentinel_resolution(args) {
        env_resolved_owned = resolve_env_sentinels(args)?;
        &env_resolved_owned
    } else {
        args
    };

    // mstring display-grouping (SPEC §3.2): strip display separators from the
    // --ms1/--mk1/--md1 flag values up-front (each flag value is ONE chunk, so a
    // full strip is safe) so a grouped or unbroken card both re-ingest — and the
    // stripped (canonical) form reaches decode, the forensic `supplied == expected`
    // equality, AND the `expected`/`actual` JSON fields uniformly (R0-r1 C4).
    // NOT `--bundle-json` (that path is canonical-unbroken by construction).
    let stripped_owned;
    let args: &VerifyBundleArgs = if args
        .ms1
        .iter()
        .chain(&args.mk1)
        .chain(&args.md1)
        .any(|v| v.chars().any(crate::display_grouping::is_display_separator))
    {
        let mut a = args.clone();
        let strip = crate::display_grouping::strip_display_separators;
        a.ms1 = a.ms1.iter().map(|s| strip(s)).collect();
        a.mk1 = a.mk1.iter().map(|s| strip(s)).collect();
        a.md1 = a.md1.iter().map(|s| strip(s)).collect();
        stripped_owned = a;
        &stripped_owned
    } else {
        args
    };

    // v0.24.0 §2.C.1 (D34/I5 fold) — strict per-flag HRP validation across
    // verify-bundle's typed `--ms1` / `--mk1` / `--md1` flag args. Mirrors
    // the same gate in `cmd::repair::run` + `cmd::inspect::run` so all three
    // subcommands enforce mismatched-HRP rejection uniformly (architect
    // review C1 fold — previously verify-bundle dropped through to sibling
    // codec parse errors with no flag-name attribution).
    for (idx, v) in args.ms1.iter().enumerate() {
        crate::repair::validate_flag_hrp("--ms1", "ms", v)?;
        // v0.25.1 fix: empty-string positional watch-only sentinel per SPEC §5.8.
        // Emit an expressive NOTICE so the user sees the intent (catches the
        // accidental-empty-from-unset-shell-variable footgun while preserving
        // the intentional middle / trailing-cosigner skip convention).
        if v.is_empty() {
            let _ = writeln!(
                stderr,
                "notice: cosigner[{idx}] marked watch-only via empty `--ms1` \
                 sentinel (SPEC §5.8); no seed will be derived for this slot"
            );
        }
    }
    for v in &args.mk1 {
        crate::repair::validate_flag_hrp("--mk1", "mk", v)?;
    }
    for v in &args.md1 {
        crate::repair::validate_flag_hrp("--md1", "md", v)?;
    }

    let stdin_synth;
    let args: &VerifyBundleArgs = if needs_stdin_substitution(args) {
        stdin_synth = apply_stdin_substitutions(args, stdin)?;
        &stdin_synth
    } else {
        args
    };

    // v0.24.0 §2.C.1 — positional `<STRING>...` intake. Route each
    // positional value to the matching typed-flag bucket (ms1/mk1/md1)
    // by HRP prefix. Unknown HRPs return `ToolkitError::UnknownHrp`.
    // Mutually exclusive with `--bundle-json` at clap-parse time
    // (per I3 fold; `conflicts_with = "bundle_json"` on the
    // `extra_strings` arg).
    let positional_synth;
    let args: &VerifyBundleArgs = if !args.extra_strings.is_empty() {
        positional_synth = apply_positional_hrp_autodetect(args)?;
        &positional_synth
    } else {
        args
    };

    // v0.4.3 Phase Q: --bundle-json intake. Load JSON envelope, extract
    // ms1/mk1/md1 into a synthetic VerifyBundleArgs, then continue dispatch
    // as if the user had supplied --ms1/--mk1/--md1 directly.
    let synthetic_args;
    let args = if args.bundle_json.is_some() {
        synthetic_args = load_bundle_json_into_args(args)?;
        &synthetic_args
    } else {
        args
    };

    // Cycle B Phase 3a Site 1 — pin argv-string secret heap pages for the
    // remainder of the handler scope. Lands AFTER both apply_stdin_substitutions
    // and load_bundle_json_into_args returns so the pin covers the final
    // post-substitution buffers (per SPEC §4 P3a).
    let _pin_passphrase = args
        .passphrase
        .as_ref()
        .map(|p| pin_pages_for(p.as_bytes()));
    let _pin_slot_values: Vec<_> = args
        .slot
        .iter()
        .map(|s| pin_pages_for(s.value.as_bytes()))
        .collect();

    // #28 phase 1 — keyless SINGLE-SIG TEMPLATE bundle short-circuit. A
    // template-form md1 (`bundle --md1-form=template`) is keyless + single-sig +
    // canonical-origin-elided; the policy-form expected-bundle reconstruction
    // below would never match it (different stub, no md1 pubkeys to bind). The
    // dedicated path binds the cards via the template-id stub, then completes +
    // recomposes the watch-only single-sig wallet from the supplied seed
    // (`--slot @0.<secret>=` + `--account`/`--origin`). Escapes before the
    // template/descriptor dispatch (a template bundle may omit `--template`).
    if !args.md1.is_empty() {
        let md1_refs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
        if let Ok(d) = md_codec::chunk::reassemble(&md1_refs) {
            let is_singlesig_template = !d.is_wallet_policy()
                && d.n == 1
                && md_codec::canonical_origin::canonical_origin(&d.tree).is_some()
                && crate::synthesize::cli_template_from_tree(&d.tree).is_some();
            if is_singlesig_template {
                return verify_singlesig_template(&d, args, stdin, stdout, stderr, json_context);
            }
            // #28 phase 2 — keyless MULTISIG / general TEMPLATE bundle (n≥2,
            // canonical OR general — same routing as restore P3b). The supplied
            // `--from` seed + `--cosigner` keys complete a concrete watch-only
            // wallet via the SHARED completion engine; verify-bundle then asserts
            // the card↔template-id binding + surfaces the completed id/address.
            // WITHOUT `--from` this still routes here and refuses (floor 1(i)),
            // naming `--from`.
            let is_multisig_template = !d.is_wallet_policy() && d.n >= 2;
            if is_multisig_template {
                return verify_multisig_template(&d, args, stdin, stdout, stderr, json_context);
            }
        }
    }

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
        return descriptor_mode_verify_run(
            args,
            stdin,
            stdout,
            stderr,
            effective_no_auto_repair,
            json_context,
        );
    }

    // #28 — `--template` is now clap-optional when `--md1`/`--bundle-json`/
    // positionals are present (to allow a keyless template md1 to carry its own
    // type). The template-form short-circuit above already handled the keyless
    // single-sig template; any md1 reaching here is a keyed wallet-policy md1
    // (or a non-template keyless shape) that DOES need an explicit `--template`.
    if args.template.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "verify-bundle",
            flag: "--template",
            message: "--template is required (the supplied md1 is not a keyless single-sig \
                      template that carries its own type); supply --template <bip44|…> or \
                      --descriptor",
        });
    }
    let multisig = args.template_unchecked().is_multisig();

    if args.threshold.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--threshold",
            message: mode_text::THRESHOLD_WITHOUT_MULTISIG,
        });
    }
    if args.multisig_path_family.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--multisig-path-family",
            message: mode_text::PATH_FAMILY_WITHOUT_MULTISIG,
        });
    }

    // FOLLOWUP `multisig-tr-bip48-script-type-3-policy` (bless + warn): mirror
    // the bundle/export-wallet advisory so re-deriving a taproot+bip48 bundle
    // under verify-bundle surfaces the same non-standard m/48'/.../3' notice.
    // Fires once here (template-mode only; descriptor mode escaped at the top
    // and refuses --multisig-path-family).
    if let Some(w) = args
        .template_unchecked()
        .bip48_nonstandard_script_type_warning(args.multisig_path_family.unwrap_or_default())
    {
        let _ = writeln!(stderr, "{w}");
    }

    crate::slot_input::validate_slot_set(&args.slot)?;
    let n = args
        .slot
        .iter()
        .map(|s| s.index as usize)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    let template_str = args.template.map(|t| t.human_name());
    let multisig_template_name = template_str.filter(|_| multisig);
    crate::bundle_unified::pre_check_threshold(args.threshold, n, multisig_template_name)?;
    if let Some(t) = args.template {
        crate::bundle_unified::pre_check_template_n(t.human_name(), t.is_multisig(), n)?;
    }

    let mut checks: Vec<VerifyCheck> = Vec::new();
    if multisig {
        run_multisig(
            args,
            &mut checks,
            stdout,
            stderr,
            effective_no_auto_repair,
            json_context,
        )?;
    } else {
        let secret_bearing_at_0 = args
            .slot
            .iter()
            .any(|s| s.index == 0 && s.subkey.is_secret_bearing());
        if secret_bearing_at_0 {
            run_full(
                args,
                &mut checks,
                stdout,
                stderr,
                effective_no_auto_repair,
                json_context,
            )?;
        } else {
            run_watch_only(
                args,
                &mut checks,
                stdout,
                stderr,
                effective_no_auto_repair,
                json_context,
            )?;
        }
    }

    let any_fail = checks.iter().any(|c| !c.passed);
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
            let status = if c.passed { "ok" } else { "fail" };
            if c.detail.is_empty() {
                writeln!(stdout, "{}: {}", c.name, status).ok();
            } else {
                writeln!(stdout, "{}: {} {}", c.name, status, c.detail).ok();
            }
        }
        writeln!(stdout, "result: {}", result).ok();
    }

    Ok(if any_fail { 4 } else { 0 })
}

/// #28 phase 1 — verify + recompose a keyless SINGLE-SIG TEMPLATE bundle.
///
/// The supplied md1 (`d`, already reassembled + classified) is keyless,
/// single-sig, canonical-origin-elided. We:
///  1. derive the type from the md1 tree, resolve the seed slot
///     (`--slot @0.<secret>=`) to the keyed xpub (+ `--account`/`--origin`);
///  2. re-synthesize the EXPECTED bundle in TEMPLATE form and compare the
///     supplied md1/mk1/ms1 against it — this binds the cards via the
///     template-id stub (a card swap or policy/template cross-mix fails);
///  3. recompose the concrete watch-only single-sig wallet (descriptor +
///     first-receive) and print it;
///  4. honor `--expect-wallet-id` (D7 recompute-and-match).
///
/// A seed slot is REQUIRED (the template is keyless — without the seed there is
/// no wallet to recompose); a missing `--slot @0.<secret>=` is refused.
fn verify_singlesig_template<W: Write, E: Write>(
    d: &md_codec::Descriptor,
    args: &VerifyBundleArgs,
    _stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
    json_context: bool,
) -> Result<u8, ToolkitError> {
    use crate::synthesize::ResolvedSlot;
    use std::str::FromStr;

    let template = crate::synthesize::cli_template_from_tree(&d.tree)
        .ok_or_else(|| ToolkitError::BadInput("template md1 tree is not single-sig".into()))?;

    // Require a secret-bearing @0 slot (the seed).
    let secret_bearing_at_0 = args
        .slot
        .iter()
        .any(|s| s.index == 0 && s.subkey.is_secret_bearing());
    if !secret_bearing_at_0 {
        return Err(ToolkitError::ModeViolation {
            mode: "verify-bundle",
            flag: "--md1",
            message: "verifying a keyless single-sig TEMPLATE bundle requires the seed \
                      (`--slot @0.phrase=…` / `.ms1=` / `.entropy=` / `.seedqr=`) to recompose \
                      the watch-only wallet — the template carries no key",
        });
    }

    // Resolve the seed slot → keyed xpub at the template origin (+ account).
    let (resolved, _signals) = crate::cmd::bundle::resolve_slots(
        &args.slot,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
        args.multisig_path_family.unwrap_or_default(),
    )?;
    let canonical_slot = resolved
        .first()
        .ok_or_else(|| ToolkitError::BadInput("no @0 slot resolved".into()))?;

    // #28 phase 1 (R0 I2) — `--origin` override. Mirrors `restore --origin`: the
    // template is account/origin-agnostic (byte-identical md1), so a custom
    // origin is supplied at verify time. Re-derive the seed slot's xpub at the
    // explicit BIP-32 path (via the SAME `derive_bip32_from_entropy_at_path`
    // wrapper restore uses) so the recompose + the expected-bundle card
    // comparison both reflect the override origin. The canonical-account
    // `resolve_slots` result is used when `--origin` is absent.
    let origin_slot: Option<ResolvedSlot> = match args.origin.as_deref() {
        Some(origin_str) => {
            let entropy = canonical_slot.entropy.as_ref().ok_or_else(|| {
                ToolkitError::BadInput(
                    "verify-bundle --origin requires a secret-bearing seed slot (the override \
                     re-derives the key at the supplied path)"
                        .into(),
                )
            })?;
            let path = bitcoin::bip32::DerivationPath::from_str(
                origin_str.trim_start_matches("m/").trim_start_matches('m'),
            )
            .or_else(|_| bitcoin::bip32::DerivationPath::from_str(origin_str))
            .map_err(|e| ToolkitError::BadInput(format!("--origin {origin_str}: {e}")))?;
            let passphrase = args.passphrase.as_deref().unwrap_or("");
            let derive_language: bip39::Language = canonical_slot
                .language
                .unwrap_or_else(|| args.language.unwrap_or_default().into());
            let acct = crate::derive_slot::derive_bip32_from_entropy_at_path(
                entropy,
                passphrase,
                derive_language,
                args.network,
                &path,
            )?;
            Some(ResolvedSlot {
                xpub: acct.account_xpub,
                fingerprint: acct.master_fingerprint,
                path: acct.account_path.clone(),
                entropy: None,
                master_xpub: None,
                language: None,
                _entropy_pin: None,
            })
        }
        None => None,
    };
    // The effective slot + single-element resolved vec for card-synthesis: the
    // override slot when `--origin` is set, else the canonical resolution.
    let resolved_for_synth: Vec<ResolvedSlot> = match &origin_slot {
        Some(s) => vec![s.clone()],
        None => resolved.clone(),
    };
    let slot: &ResolvedSlot = origin_slot.as_ref().unwrap_or(canonical_slot);

    // (2) Re-synthesize the EXPECTED bundle in TEMPLATE form; compare cards.
    let expected = crate::synthesize::synthesize_unified(
        &resolved_for_synth,
        template,
        1,
        args.network,
        args.privacy_preserving,
        args.language.unwrap_or_default().into(),
        crate::synthesize::Md1Form::Template,
    )?;
    let md1_match = expected.md1 == args.md1;
    let mk1_match = match &expected.mk1 {
        crate::format::MkField::Single(chunks) => chunks == &args.mk1,
        crate::format::MkField::Multi(_) => false,
    };
    let mut checks: Vec<VerifyCheck> = Vec::new();
    checks.push(VerifyCheck {
        name: "md1_template_match".into(),
        passed: md1_match,
        detail: if md1_match {
            "supplied md1 matches the expected keyless single-sig template".into()
        } else {
            "supplied md1 does NOT match the expected template for this seed/type".into()
        },
        ..Default::default()
    });
    checks.push(VerifyCheck {
        name: "mk1_template_stub_bind".into(),
        passed: mk1_match,
        detail: if mk1_match {
            "supplied mk1 binds via the template-id stub".into()
        } else {
            "supplied mk1 does NOT bind via the template-id stub (card mismatch or policy/template cross-mix)".into()
        },
        ..Default::default()
    });

    // (3) Recompose the concrete watch-only wallet.
    let descriptor = crate::wallet_export::build_descriptor_string(
        template,
        std::slice::from_ref(slot),
        1,
        args.network,
        args.account,
        None,
    )?;
    let secp = bitcoin::secp256k1::Secp256k1::verification_only();
    let script_type = crate::cmd::convert::script_type_from_template(template)
        .expect("single-sig template has a script type");
    let chain = bitcoin::bip32::ChildNumber::from_normal_idx(0).unwrap();
    let leaf = bitcoin::bip32::ChildNumber::from_normal_idx(0).unwrap();
    let dp: bitcoin::bip32::DerivationPath = vec![chain, leaf].into();
    let first_recv = slot
        .xpub
        .derive_pub(&secp, &dp)
        .map(|child| {
            crate::address_render::render_address_from_xpub(
                &secp,
                &child,
                script_type,
                args.network,
            )
        })
        .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;

    // (4) --expect-wallet-id (D7 recompute-and-match), via the SHARED helper.
    // Skipped under `--origin` (same carve-out as `restore`): D7 was computed
    // for the canonical `m/<purpose>'/<coin>'/account'` origin, so an explicit
    // override is a DIFFERENT preimage — matching it against the canonical id
    // would spuriously fail. Notice + skip rather than refuse.
    if let Some(prefix_hex) = args.expect_wallet_id.as_deref() {
        if origin_slot.is_some() {
            let _ = writeln!(
                stderr,
                "notice: --expect-wallet-id is not checked when --origin overrides the canonical \
                 account path (the wallet-id was computed for the canonical origin)."
            );
        } else {
            let prefix = hex::decode(prefix_hex.trim())
                .map_err(|e| ToolkitError::BadInput(format!("--expect-wallet-id hex: {e}")))?;
            if prefix.len() < 4 {
                let _ = writeln!(
                    stderr,
                    "advisory: --expect-wallet-id prefix is only {} byte(s); ≥4 recommended.",
                    prefix.len()
                );
            }
            let id = crate::synthesize::wallet_policy_id_for_singlesig(
                template,
                args.network,
                &slot.xpub,
                slot.fingerprint,
                args.account,
            )?;
            let ok =
                id.as_bytes().len() >= prefix.len() && id.as_bytes()[..prefix.len()] == prefix[..];
            checks.push(VerifyCheck {
                name: "wallet_id_match".into(),
                passed: ok,
                detail: if ok {
                    "completed wallet matches --expect-wallet-id".into()
                } else {
                    "completed wallet does NOT match --expect-wallet-id".into()
                },
                ..Default::default()
            });
        }
    }

    // ---- Emit verdict -------------------------------------------------------
    let any_fail = checks.iter().any(|c| !c.passed);
    if json_context {
        let result = if any_fail { "mismatch" } else { "ok" };
        let json = serde_json::json!({
            "result": result,
            "mode": "single-sig-template",
            "wallet_type": template.human_name(),
            "descriptor": descriptor,
            "first_receive": first_recv,
            "checks": checks.iter().map(|c| serde_json::json!({
                "name": c.name, "passed": c.passed, "detail": c.detail,
            })).collect::<Vec<_>>(),
        });
        writeln!(stdout, "{}", serde_json::to_string(&json).unwrap()).map_err(ToolkitError::Io)?;
    } else {
        for c in &checks {
            writeln!(
                stderr,
                "{} {}: {}",
                if c.passed { "✓" } else { "✗" },
                c.name,
                c.detail
            )
            .map_err(ToolkitError::Io)?;
        }
        if any_fail {
            writeln!(stdout, "mismatch").map_err(ToolkitError::Io)?;
        } else {
            writeln!(stdout, "OK (single-sig template recomposed)").map_err(ToolkitError::Io)?;
            writeln!(stdout, "wallet type: {}", template.human_name()).map_err(ToolkitError::Io)?;
            writeln!(stdout, "descriptor:  {descriptor}").map_err(ToolkitError::Io)?;
            writeln!(stdout, "first recv:  {first_recv}").map_err(ToolkitError::Io)?;
        }
    }

    Ok(if any_fail { 4 } else { 0 })
}

/// #28 phase 2 — verify + recompose a keyless MULTISIG / general TEMPLATE bundle.
///
/// The supplied `--md1` (`d`, already reassembled + classified keyless n≥2) is a
/// `bundle --md1-form=template` policy. We:
///  1. require `--from` (the operator's own seed — the template is keyless);
///  2. run the SAME completion engine `restore` uses
///     (`complete_multisig_template`): `--from` own slot(s) + `--cosigner` keys →
///     per-slot origin BUILD → the floors → the search → the unique key→slot
///     assignment + a fresh, fully-keyed descriptor (NEVER the carried
///     `path_decl` — the C1 invariant);
///  3. assert the card↔template-id BINDING:
///     - `md1_template_match`: the supplied keyless `--md1`'s
///       `WalletDescriptorTemplateId` (key-invariant) equals the completed
///       wallet's — i.e. the engraved md1 IS this wallet's template;
///     - `mk1_template_stub_bind`: each supplied `--mk1[slot]` template STUB
///       card's `chunk_set_id` equals
///       `derive_mk1_chunk_set_id_for_slot(template_id_stub, slot)`;
///  4. recompose the watch-only wallet (descriptor + first receive) via the
///     IDENTICAL engine restore emits with (funds-safety parity);
///  5. surface the completed `WalletPolicyId` + first address as a `VerifyCheck`.
///
/// `--expect-wallet-id` / `--search-address` are consumed BY the completion
/// engine (the search target), so a UNIQUE outcome already implies the completed
/// id/address matches; on NO-MATCH / AMBIGUOUS / any floor the engine RETURNS the
/// (refuse) error (exit 2/4), never a silent OK.
fn verify_multisig_template<W: Write, E: Write>(
    d: &md_codec::Descriptor,
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
    json_context: bool,
) -> Result<u8, ToolkitError> {
    use crate::cmd::restore::{
        candidate_descriptor_string, complete_multisig_template, resolve_template_completion_seed,
        MultisigCompletionCtx,
    };
    use std::str::FromStr;

    let network = args.network;

    // (1) Floor 1(i): `--from` REQUIRED + resolve the seed (SHARED with restore).
    let no_from = ToolkitError::ModeViolation {
        mode: "verify-bundle",
        flag: "--md1",
        message: "verifying a keyless MULTISIG TEMPLATE bundle requires the operator's own seed \
                  via --from <seed> (the template carries no keys; the seed derives your cosigner \
                  key, and --cosigner <mk1> supplies the others). Supply \
                  --from ms1=…/phrase=…/entropy=…/seedqr=…",
    };
    let seed = resolve_template_completion_seed(
        args.from.as_deref(),
        no_from,
        args.passphrase.as_deref(),
        args.passphrase_stdin,
        args.language,
        stdin,
        stderr,
    )?;

    // The explicit own-origin override (#28 phase-1 `--origin`, reused).
    let explicit_own_origin = match args.origin.as_deref() {
        Some(s) => Some(
            bitcoin::bip32::DerivationPath::from_str(
                s.trim_start_matches("m/").trim_start_matches('m'),
            )
            .or_else(|_| bitcoin::bip32::DerivationPath::from_str(s))
            .map_err(|e| ToolkitError::BadInput(format!("--origin {s}: {e}")))?,
        ),
        None => None,
    };

    // (2) Build the NEUTRAL ctx + run the SHARED completion engine. For the
    // verify surface `--account` is a single own account (multi-own-account
    // completion is restore-only this cycle).
    let ctx = MultisigCompletionCtx {
        entropy: &seed.entropy,
        passphrase: &seed.passphrase,
        derive_language: seed.derive_language,
        own_accounts: vec![args.account],
        explicit_own_origin,
        cosigner_specs: &args.cosigner,
        // P4 — verify-bundle now exposes `--own-account-max` / `--search-cosigner-
        // subset` (both NEW NAMES here) and threads them into the SHARED
        // `complete_multisig_template` engine restore uses, so verify == restore
        // over the subset-search (own pool from `0..K`, the opt-in cosigner
        // stratification, the ceilings, the §5a gates — all inherited).
        // `--account` stays SCALAR on verify-bundle (own-only-via-range covers
        // the over-supply there); `own_accounts` is still the single scalar.
        own_account_max: args.own_account_max,
        search_cosigner_subset: args.search_cosigner_subset,
        expect_wallet_id: args.expect_wallet_id.clone(),
        search_address: args.search_address.clone(),
        search_addr_min: args.search_addr_min,
        search_addr_max: args.search_addr_max,
        search_chain: args.search_chain,
        accept_search_time: args.accept_search_time.clone(),
        network,
    };
    let outcome = complete_multisig_template(d, &ctx, stderr)?;

    // (3) Binding: the supplied md1 + mk1 STUB cards bind to the recomposed
    // wallet's key-invariant `WalletDescriptorTemplateId`.
    let completed_template_id = md_codec::compute_wallet_descriptor_template_id(&outcome.completed)
        .map_err(ToolkitError::from)?;
    let supplied_template_id =
        md_codec::compute_wallet_descriptor_template_id(d).map_err(ToolkitError::from)?;
    let md1_match = completed_template_id.as_bytes() == supplied_template_id.as_bytes();

    let mut checks: Vec<VerifyCheck> = Vec::new();
    checks.push(VerifyCheck {
        name: "md1_template_match".into(),
        passed: md1_match,
        detail: if md1_match {
            "supplied md1 matches the recomposed wallet's keyless template (WalletDescriptorTemplateId)".into()
        } else {
            "supplied md1 does NOT match the recomposed wallet's template (card mismatch or policy/template cross-mix)".into()
        },
        ..Default::default()
    });

    // `mk1_template_stub_bind`: the supplied --mk1 template stubs' chunk_set_ids
    // must equal `derive_mk1_chunk_set_id_for_slot(template_stub, slot)` for the
    // recomposed wallet's template-id. The stubs are slot-indexed; group them by
    // their declared csi (each cosigner's chunks share one csi), then check the
    // SET of csis equals the expected per-slot set. Empty --mk1 → not checked.
    let mk1_bind =
        check_mk1_template_stubs(&args.mk1, completed_template_id.as_bytes(), d.n as usize);
    if let Some(passed) = mk1_bind {
        checks.push(VerifyCheck {
            name: "mk1_template_stub_bind".into(),
            passed,
            detail: if passed {
                "supplied mk1 template stub(s) bind via the template-id stub".into()
            } else {
                "supplied mk1 stub(s) do NOT bind via the template-id stub (card mismatch or policy/template cross-mix)".into()
            },
            ..Default::default()
        });
    }

    // (4) Recompose the watch-only wallet via the IDENTICAL restore engine.
    let descriptor = candidate_descriptor_string(&outcome.completed, network)?;
    let parsed = miniscript::Descriptor::<miniscript::DescriptorPublicKey>::from_str(&descriptor)
        .map_err(|e| {
        ToolkitError::DescriptorParse(format!("completed descriptor parse: {e}"))
    })?;
    let first_recv =
        crate::derive_address::derive_receive_addresses(&parsed, 1, network.to_bitcoin_network())?
            .into_iter()
            .next()
            .ok_or_else(|| ToolkitError::DescriptorParse("no first receive address".into()))?;

    // (5) Surface the completed WalletPolicyId + first address (funds-safety).
    let completed_id =
        md_codec::compute_wallet_policy_id(&outcome.completed).map_err(ToolkitError::from)?;
    let completed_id_hex = hex::encode(completed_id.as_bytes());
    checks.push(VerifyCheck {
        name: "wallet_completed".into(),
        passed: true,
        detail: format!("completed WalletPolicyId {completed_id_hex}; first receive {first_recv}"),
        ..Default::default()
    });

    let own_pos = outcome.own_position();

    // ---- Emit verdict (mirror verify_singlesig_template's envelope) ---------
    let any_fail = checks.iter().any(|c| !c.passed);
    if json_context {
        let result = if any_fail { "mismatch" } else { "ok" };
        let json = serde_json::json!({
            "result": result,
            "mode": "multisig-template",
            "wallet_policy_id": completed_id_hex,
            "own_position": own_pos,
            "descriptor": descriptor,
            "first_receive": first_recv,
            "checks": checks.iter().map(|c| serde_json::json!({
                "name": c.name, "passed": c.passed, "detail": c.detail,
            })).collect::<Vec<_>>(),
        });
        writeln!(stdout, "{}", serde_json::to_string(&json).unwrap()).map_err(ToolkitError::Io)?;
    } else {
        for c in &checks {
            writeln!(
                stderr,
                "{} {}: {}",
                if c.passed { "✓" } else { "✗" },
                c.name,
                c.detail
            )
            .map_err(ToolkitError::Io)?;
        }
        if any_fail {
            writeln!(stdout, "mismatch").map_err(ToolkitError::Io)?;
        } else {
            writeln!(stdout, "OK (multisig template recomposed)").map_err(ToolkitError::Io)?;
            writeln!(stdout, "wallet-policy-id: {completed_id_hex}").map_err(ToolkitError::Io)?;
            writeln!(stdout, "descriptor:  {descriptor}").map_err(ToolkitError::Io)?;
            writeln!(stdout, "first recv:  {first_recv}").map_err(ToolkitError::Io)?;
            if let Some(p) = own_pos {
                writeln!(stdout, "your seed completes cosigner slot @{p}")
                    .map_err(ToolkitError::Io)?;
            }
        }
    }

    Ok(if any_fail { 4 } else { 0 })
}

/// `mk1_template_stub_bind`: validate the supplied `--mk1` TEMPLATE stub cards
/// bind to `template_id` (the recomposed wallet's `WalletDescriptorTemplateId`).
/// Each cosigner's mk1 chunks declare a `chunk_set_id`; for slot `i` the expected
/// csi is `derive_mk1_chunk_set_id_for_slot(template_id[0..4], i)`. The supplied
/// stubs are slot-ordered, so the SET of supplied csis must equal the expected
/// per-slot set `{ csi(i) : i in 0..n }`. Returns `None` when no `--mk1` is
/// supplied (the check is skipped — completion still gates funds-safety).
fn check_mk1_template_stubs(mk1: &[String], template_id: &[u8], n: usize) -> Option<bool> {
    if mk1.is_empty() {
        return None;
    }
    let stub: [u8; 4] = [
        template_id[0],
        template_id[1],
        template_id[2],
        template_id[3],
    ];
    let expected: std::collections::BTreeSet<u32> = (0..n as u32)
        .map(|i| crate::synthesize::derive_mk1_chunk_set_id_for_slot(&stub, i))
        .collect();
    let supplied: std::collections::BTreeSet<u32> =
        mk1.iter().filter_map(|s| chunk_set_id_extract(s)).collect();
    // Every supplied stub csi must be one of the expected per-slot csis, and the
    // supplied set must cover all n slots (one stub per cosigner).
    Some(supplied == expected)
}

fn run_full<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
    json_context: bool,
) -> Result<(), ToolkitError> {
    let template = args.template_unchecked();
    // verify-bundle does not surface SLIP-0132 input-normalization signals.
    // SPEC `design/SPEC_convert_v0_6.md` §11 v0.7 amendment (Option B): checker
    // semantics suppress info-lines to avoid breaking script callers parsing
    // VERIFIED/MISMATCH stderr line-by-line.
    let (resolved, _slip0132_signals) = crate::cmd::bundle::resolve_slots(
        &args.slot,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
        args.multisig_path_family.unwrap_or_default(),
    )?;
    let n = resolved.len() as u8;
    let threshold = args.threshold.unwrap_or(n);
    let expected = crate::synthesize::synthesize_unified(
        &resolved,
        template,
        threshold,
        args.network,
        args.privacy_preserving,
        args.language.unwrap_or_default().into(),
        crate::synthesize::Md1Form::Policy,
    )?;
    let supplied = SuppliedCards {
        ms1: &args.ms1,
        mk1: &args.mk1,
        md1: &args.md1,
    };
    // v0.25.0 §2.D — ms1-driven parent_fingerprint check at depth ≥ 2.
    // Extends v0.24.0 D30 to the depth-≥-2 blind spot. Full-path single-sig:
    // ms1 supplied → derive parent xpub from seed; mismatch → stderr warning.
    emit_full_path_parent_fingerprint_check(
        &args.ms1,
        &args.mk1,
        &args.md1,
        false,
        args.passphrase.as_deref(),
        args.language,
        args.network,
        stderr,
    );
    checks.extend(emit_verify_checks(
        &expected,
        &supplied,
        false,
        no_auto_repair,
        json_context,
        stdout,
        stderr,
    )?);
    Ok(())
}

fn run_watch_only<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
    json_context: bool,
) -> Result<(), ToolkitError> {
    // SPEC §2.2.2 watch-only-cannot-verify-path warning. Emitted before any
    // parse error so the user always sees it, even if the supplied xpub is
    // malformed.
    writeln!(
        stderr,
        "warning: watch-only verify-bundle does not verify --slot @0.xpub= is actually at the"
    )
    .ok();
    writeln!(
        stderr,
        "warning: claimed BIP path m/<purpose>'/<coin>'/0' (no master seed available"
    )
    .ok();
    writeln!(
        stderr,
        "warning: for re-derivation). Use --slot @0.phrase= mode for end-to-end verification."
    )
    .ok();

    // v0.24.0 D30 — defense-in-depth cross-check between supplied mk1 xpub
    // fields and md1's claimed OriginPath. Warns (not errors) on mismatch.
    emit_watch_only_xpub_path_cross_check(&args.mk1, &args.md1, false, stderr);

    // v0.25.0 §2.D — watch-only NOTICE at depth ≥ 2 (no ms1 → cannot derive
    // parent xpub; per BIP-32 child→parent one-wayness).
    emit_full_path_parent_fingerprint_check(
        &args.ms1,
        &args.mk1,
        &args.md1,
        false,
        args.passphrase.as_deref(),
        args.language,
        args.network,
        stderr,
    );

    let template = args.template_unchecked();
    // verify-bundle does not surface SLIP-0132 input-normalization signals.
    // SPEC `design/SPEC_convert_v0_6.md` §11 v0.7 amendment (Option B): checker
    // semantics suppress info-lines to avoid breaking script callers parsing
    // VERIFIED/MISMATCH stderr line-by-line.
    let (resolved, _slip0132_signals) = crate::cmd::bundle::resolve_slots(
        &args.slot,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
        args.multisig_path_family.unwrap_or_default(),
    )?;
    let n = resolved.len() as u8;
    let threshold = args.threshold.unwrap_or(n);
    let expected = crate::synthesize::synthesize_unified(
        &resolved,
        template,
        threshold,
        args.network,
        args.privacy_preserving,
        args.language.unwrap_or_default().into(),
        crate::synthesize::Md1Form::Policy,
    )?;
    let supplied = SuppliedCards {
        ms1: &args.ms1,
        mk1: &args.mk1,
        md1: &args.md1,
    };
    checks.extend(emit_verify_checks(
        &expected,
        &supplied,
        false,
        no_auto_repair,
        json_context,
        stdout,
        stderr,
    )?);
    Ok(())
}

/// Multisig verify-bundle entry. Synthesizes the expected Bundle (full or
/// watch-only) and dispatches to `emit_verify_checks(... is_multisig: true)`,
/// which emits the SPEC §5.7 `3 + 6N` schema in this order:
///
///   For each cosigner i ∈ 0..N (interleaved by slot):
///     ms1_decode[i], ms1_entropy_match[i],
///     mk1_decode[i], mk1_xpub_match[i],
///     mk1_fingerprint_match[i], mk1_path_match[i].
///   Then 3 shared md1 checks:
///     md1_decode, md1_wallet_policy, md1_xpub_match.
///
/// Watch-only / wif slots (`expected.ms1[i] == ""`) short-circuit ms1_decode[i]
/// and ms1_entropy_match[i] with `passed: true + decode_error: "skipped: watch-only slot"`.
fn run_multisig<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
    json_context: bool,
) -> Result<(), ToolkitError> {
    let any_secret = args.slot.iter().any(|s| s.subkey.is_secret_bearing());
    let any_watch_only = args.slot.iter().any(|s| s.subkey.is_watch_only());
    let watch_only_multi = !any_secret && any_watch_only;

    if watch_only_multi {
        // SPEC §2.2.2 multisig watch-only stderr warning.
        writeln!(
            stderr,
            "warning: watch-only multisig verify-bundle does not verify --slot xpubs are at the"
        )
        .ok();
        writeln!(
            stderr,
            "warning: claimed BIP path (no per-cosigner master seed available for re-derivation)."
        )
        .ok();
        writeln!(
            stderr,
            "warning: Use --slot @N.phrase= mode for end-to-end verification of self-multisig backups."
        )
        .ok();

        // v0.24.0 D30 — defense-in-depth cross-check between supplied mk1
        // xpub fields and md1's claimed OriginPath, per-cosigner.
        emit_watch_only_xpub_path_cross_check(&args.mk1, &args.md1, true, stderr);
    }

    // v0.25.0 §2.D — ms1-driven parent_fingerprint check at depth ≥ 2.
    // Fires regardless of watch_only_multi: full-path multisig has ms1 for
    // every cosigner (warning on mismatch); partial-watch-only multisig has
    // ms1 for some cosigners (warning on those; notice on the empty/missing
    // ones).
    emit_full_path_parent_fingerprint_check(
        &args.ms1,
        &args.mk1,
        &args.md1,
        true,
        args.passphrase.as_deref(),
        args.language,
        args.network,
        stderr,
    );

    let template = args.template_unchecked();
    // verify-bundle does not surface SLIP-0132 input-normalization signals.
    // SPEC `design/SPEC_convert_v0_6.md` §11 v0.7 amendment (Option B): checker
    // semantics suppress info-lines to avoid breaking script callers parsing
    // VERIFIED/MISMATCH stderr line-by-line.
    let (resolved, _slip0132_signals) = crate::cmd::bundle::resolve_slots(
        &args.slot,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
        args.multisig_path_family.unwrap_or_default(),
    )?;
    let n = resolved.len() as u8;
    let threshold = args.threshold.unwrap_or(1);
    let expected = crate::synthesize::synthesize_unified(
        &resolved,
        template,
        threshold,
        args.network,
        args.privacy_preserving,
        args.language.unwrap_or_default().into(),
        crate::synthesize::Md1Form::Policy,
    )?;
    let _ = n;

    let supplied = SuppliedCards {
        ms1: &args.ms1,
        mk1: &args.mk1,
        md1: &args.md1,
    };

    checks.extend(emit_verify_checks(
        &expected,
        &supplied,
        true,
        no_auto_repair,
        json_context,
        stdout,
        stderr,
    )?);
    Ok(())
}

/// Phase D descriptor-mode verify: re-run the descriptor pipeline to build the
/// expected Bundle, then compare each card against the supplied --ms1/--mk1/--md1.
/// Emits the same VerifyBundleJson schema as template-mode verify (per SPEC §5.7
/// the check schema is structurally unchanged; only the source of truth differs).
fn descriptor_mode_verify_run<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    _stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
    json_context: bool,
) -> Result<u8, ToolkitError> {
    use crate::parse_descriptor::{
        check_key_vector_distinctness, lex_placeholders, parse_descriptor, resolve_placeholders,
        DescriptorBinding, ParsedFingerprint, ParsedKey,
    };
    use crate::synthesize::{xpub_to_65, CosignerKeyInfo};

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

    // BIP-388 wallet-policy intake (mirror bundle.rs:319): a leading-`{` policy
    // JSON expands to a concrete descriptor BEFORE classify — a raw policy trips
    // BOTH classify's @N and key-regex probes (the v0.49.0 ordering invariant).
    // verify-bundle is read-only → no policy-name preservation (unlike
    // export-wallet). Brings verify-bundle's --descriptor intake into parity with
    // bundle/export-wallet (FOLLOWUP verify-bundle-bip388-policy-intake).
    let descriptor_str = if crate::wallet_import::pipeline::is_bip388_policy_shape(&descriptor_str)
    {
        crate::wallet_import::pipeline::expand_bip388_policy(&descriptor_str)?
    } else {
        descriptor_str
    };

    // SPEC bip388-double-star-shorthand-support §5 — single per-command
    // chokepoint, placed AFTER the BIP-388 JSON expansion (which never
    // re-introduces `/**`) and BEFORE the Concrete/AtN split just below, so
    // both forks see the expanded string: the Concrete fork's
    // `descriptor_concrete_to_resolved_slots` → `parse_descriptor` (already
    // covered by parse_descriptor's own top-of-function expansion — this is
    // belt-and-suspenders, idempotent) AND the AtN fork's direct
    // `lex_placeholders` call below (NOT otherwise covered).
    let descriptor_str =
        crate::parse_descriptor::expand_literal_double_star(&descriptor_str).into_owned();

    // A1 P3b — bare-concrete fork: if the descriptor contains real xpubs (no
    // @N placeholders), route directly to the concrete-to-resolved-slots helper
    // and bypass the @N lex/resolve/slot-binding machinery entirely.
    {
        use crate::wallet_import::pipeline::{
            classify_descriptor_form, descriptor_concrete_to_resolved_slots, DescriptorForm,
        };
        if classify_descriptor_form(&descriptor_str)? == DescriptorForm::Concrete {
            let body_no_csum = crate::wallet_import::json_envelope::descriptor_body_no_csum(
                &descriptor_str,
                "--descriptor",
            )?;
            let (descriptor, cosigners) = descriptor_concrete_to_resolved_slots(body_no_csum)?;
            // BIP-388 distinctness: verify-bundle uses the exit-4 variant.
            if dup_xpub_path(&cosigners) {
                return Err(ToolkitError::Bip388VerifyDistinctness);
            }
            return verify_emit_from_expected(
                args,
                descriptor,
                &cosigners,
                no_auto_repair,
                json_context,
                stdout,
                stderr,
            );
        }
    }

    let occs =
        lex_placeholders(&descriptor_str).map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;
    let mut descriptor_resolved =
        resolve_placeholders(&occs).map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;
    let n = descriptor_resolved.n as usize;

    crate::slot_input::validate_slot_set(&args.slot)?;

    // Wave-4 L1: the cycle-11b L24 exact-coverage gate (formerly hand-copied
    // here, carrying an S-VERIFY fold-comment citing this dedup) now lives
    // inside the shared `bind_descriptor_mode_paths` called below — one gate,
    // both callers, drift structurally impossible.

    // v0.19.0 SPEC §4.12 — canonicity-aware verify-bundle round-trip.
    // Mirror bundle.rs's descriptor-mode binding logic so default-inferred
    // bundles round-trip correctly. Without this, verify-bundle would
    // re-derive xpubs at the template path (BIP-84 default) instead of
    // the inferred BIP-48 cosigner path, and md-codec's
    // validate_explicit_origin_required would refuse the wire.
    let canonicity_probe = parse_descriptor(&descriptor_str, &[], &[]).map_err(|e| {
        ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        }
    })?;
    let is_non_canonical =
        md_codec::canonical_origin::canonical_origin(&canonicity_probe.tree).is_none();

    // Wave-4 L1: gate + default-inference + per-slot path-override + F4 collapse
    // now run in the SHARED `bind_descriptor_mode_paths` (the same fn bundle's
    // emit path calls), so the two sides cannot drift. `Verify` mode SKIPS the
    // emit-only §6.6 row-19 path-mismatch refusal (verify is read-only — a
    // genuine inline/slot conflict surfaces downstream as a md1 byte-mismatch,
    // never as a refusal, preserving verify's accept-set). The returned
    // `defaulted_indices` is discarded: verify-bundle emits no notice, and the
    // sole consumed output (`path_decl.paths`) is byte-identical to the prior
    // hand-copied block (only the discarded bookkeeping differs).
    let _defaulted = crate::cmd::bundle::bind_descriptor_mode_paths(
        crate::cmd::bundle::DescriptorBindMode::Verify,
        &args.slot,
        &mut descriptor_resolved.path_decl,
        n,
        is_non_canonical,
        &canonicity_probe.tree.tag,
        args.network,
        args.account,
    )?;

    // Per-slot descriptor-mode binding loop using mutated path_decl as the
    // per-`@N` anno_path source. Mirror of bundle.rs:939-1099.
    use bitcoin::bip32::{Xpriv as BipXpriv, Xpub as BipXpub};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;
    let mut by_index_inputs: std::collections::BTreeMap<u8, Vec<&crate::slot_input::SlotInput>> =
        std::collections::BTreeMap::new();
    for s in &args.slot {
        by_index_inputs.entry(s.index).or_default().push(s);
    }
    let secp = Secp256k1::new();
    let mut keys: Vec<ParsedKey> = Vec::with_capacity(n);
    let mut fingerprints: Vec<ParsedFingerprint> = Vec::with_capacity(n);
    let mut cosigners: Vec<CosignerKeyInfo> = Vec::with_capacity(n);

    for idx in 0..(n as u8) {
        let slot_inputs =
            by_index_inputs
                .get(&idx)
                .ok_or_else(|| ToolkitError::DescriptorReparseFailed {
                    detail: format!("--slot @{idx} missing for descriptor with n={n} placeholders"),
                })?;
        let subkeys: std::collections::BTreeSet<crate::slot_input::SlotSubkey> =
            slot_inputs.iter().map(|s| s.subkey).collect();

        let anno_path: bitcoin::bip32::DerivationPath = match &descriptor_resolved.path_decl.paths {
            md_codec::origin_path::PathDeclPaths::Shared(op) => {
                crate::cmd::bundle::origin_to_derivation_path(op)?
            }
            md_codec::origin_path::PathDeclPaths::Divergent(v) => {
                crate::cmd::bundle::origin_to_derivation_path(&v[idx as usize])?
            }
        };

        // v0.41.0 — 5-tuple widening (Plan-R0-I1 / R0-M-C): the 5th element
        // carries the per-slot emit language (Some(wire) for a mnem ms1 cosigner;
        // None otherwise). LOAD-BEARING — verify-bundle compares whole emitted
        // card strings, so the re-emitted card must preserve the wire language.
        let (xpub, fingerprint, path, ent_opt, emit_lang): (
            BipXpub,
            bitcoin::bip32::Fingerprint,
            bitcoin::bip32::DerivationPath,
            Option<Vec<u8>>,
            Option<bip39::Language>,
        ) = if subkeys.contains(&crate::slot_input::SlotSubkey::Phrase)
            || subkeys.contains(&crate::slot_input::SlotSubkey::Seedqr)
        {
            // v0.31.3 — Seedqr materialization. Decode at slot-emit
            // time; dispatch to the same materialization path as Phrase.
            let decoded_phrase: String;
            let phrase: &str = if subkeys.contains(&crate::slot_input::SlotSubkey::Seedqr) {
                let digits = slot_inputs
                    .iter()
                    .find(|s| s.subkey == crate::slot_input::SlotSubkey::Seedqr)
                    .map(|s| &*s.value)
                    .expect("contains() asserts presence");
                decoded_phrase = mnemonic_toolkit::seedqr::decode(digits).map_err(|e| {
                    crate::cmd::seedqr::map_seedqr_error(e, &format!("slot @{idx} decode"))
                })?;
                &decoded_phrase
            } else {
                slot_inputs
                    .iter()
                    .find(|s| s.subkey == crate::slot_input::SlotSubkey::Phrase)
                    .map(|s| &*s.value)
                    .expect("contains() asserts presence")
            };
            let language = args.language.unwrap_or_default();
            let passphrase: zeroize::Zeroizing<String> =
                zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());
            let mnemonic =
                bip39::Mnemonic::parse_in(language.into(), phrase).map_err(ToolkitError::Bip39)?;
            let entropy = zeroize::Zeroizing::new(mnemonic.to_entropy());
            let seed = crate::derive_slot::derive_master_seed(&mnemonic, &passphrase);
            let master = BipXpriv::new_master(args.network.network_kind(), &seed[..])
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            let master_fp = master.fingerprint(&secp);
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
            let (xpub_str, _) = crate::slip0132::normalize_xpub_prefix(xpub_str)?;
            let xpub = BipXpub::from_str(&xpub_str)
                .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
            let fp = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Fingerprint)
                .and_then(|s| bitcoin::bip32::Fingerprint::from_str(&s.value).ok())
                .unwrap_or_default();
            let path = match slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Path)
            {
                Some(p) => bitcoin::bip32::DerivationPath::from_str(&p.value).map_err(|e| {
                    ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}"))
                })?,
                None => anno_path.clone(),
            };
            (xpub, fp, path, None, None)
        } else if subkeys.contains(&crate::slot_input::SlotSubkey::Entropy) {
            // v0.43.1 — raw-`entropy` cosigner in descriptor verify-bundle mode
            // (FOLLOWUP `verify-bundle-descriptor-entropy-slot-gap`). Mirror of
            // the bundle-loop Entropy arm (bundle.rs:1438): hex-decode, then
            // derive at the descriptor-annotated `anno_path` via the shared
            // helper. emit_lang = None — raw entropy carries no BIP-39 wire
            // language (symmetric with the bundle Entropy arm, which returns None
            // as its 5th element). Placement mirrors the bundle loop's
            // Xpub→Entropy→Ms1 order; precedence is moot (`is_legal_set` forbids
            // `[Entropy, *]` co-occurrence).
            let entropy_hex = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Entropy)
                .map(|s| &*s.value)
                .expect("contains() asserts presence");
            // SAFETY: third-party-blocked — `bip39::Mnemonic` + `Xpriv` have no
            // Drop+Zeroize. FOLLOWUPS: `rust-bip39-mnemonic-zeroize-upstream`,
            // `rust-bitcoin-xpriv-zeroize-upstream`. The decoded entropy is held
            // in `Zeroizing` and the returned `ent_opt` is re-pinned below.
            let entropy_bytes = zeroize::Zeroizing::new(hex::decode(entropy_hex).map_err(|e| {
                ToolkitError::BadInput(format!("--slot @{idx}.entropy hex-decode: {e}"))
            })?);
            let language = args.language.unwrap_or_default();
            let passphrase: zeroize::Zeroizing<String> =
                zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());
            let acc = crate::derive_slot::derive_bip32_from_entropy_at_path(
                &entropy_bytes,
                &passphrase,
                language.into(),
                args.network,
                &anno_path,
            )?;
            let (_acc_entropy, master_fp, xpub, _path) = acc.into_parts();
            (
                xpub,
                master_fp,
                anno_path.clone(),
                Some((*entropy_bytes).clone()),
                None,
            )
        } else if subkeys.contains(&crate::slot_input::SlotSubkey::Ms1) {
            // v0.41.0 — raw `ms1` codex32 secret cosigner in descriptor
            // verify-bundle mode. (SPEC-R0-I1: this loop has NO Entropy arm to
            // mirror; derive inline via the shared `slot_ms1` helper +
            // `derive_slot::derive_bip32_from_entropy_at_path` at the
            // descriptor-annotated `anno_path`.) Use `args.network` + the loop's
            // `args.passphrase` accessor (R0-M-A).
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
            (
                xpub,
                master_fp,
                anno_path.clone(),
                Some((*res.entropy).clone()),
                res.emit_language,
            )
        } else {
            return Err(ToolkitError::DescriptorReparseFailed {
                detail: format!(
                    "--slot @{idx} subkey set {:?} not supported in descriptor verify-bundle path",
                    subkeys.iter().map(|s| s.as_str()).collect::<Vec<_>>()
                ),
            });
        };

        let entropy = ent_opt.map(zeroize::Zeroizing::new);
        let entropy_pin = entropy.as_ref().map(|e| Rc::new(pin_pages_for(&e[..])));
        cosigners.push(CosignerKeyInfo {
            xpub,
            fingerprint,
            path,
            entropy,
            master_xpub: None,
            // v0.41.0 — per-slot emit language for the mnem-vs-entr re-emit; must
            // match the engraved card for the whole-card verify compare.
            language: emit_lang,
            _entropy_pin: entropy_pin,
        });
        keys.push(ParsedKey {
            i: idx,
            payload: xpub_to_65(&xpub),
        });
        fingerprints.push(ParsedFingerprint {
            i: idx,
            fp: fingerprint.to_bytes(),
        });
    }

    let binding = DescriptorBinding {
        keys: keys.clone(),
        fingerprints: fingerprints.clone(),
        cosigners: cosigners.clone(),
    };

    // SPEC §4.11.c symmetric verify-bundle enforcement: re-wrap to the verify-bundle
    // exit-4 variant so v0.2 self-multisig artifacts fail with the §4.11.c stderr.
    if let Err(ToolkitError::Bip388Distinctness { .. }) = check_key_vector_distinctness(&binding) {
        return Err(ToolkitError::Bip388VerifyDistinctness);
    }

    let mut descriptor = parse_descriptor(&descriptor_str, &keys, &fingerprints).map_err(|e| {
        ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        }
    })?;
    // v0.19.0 SPEC §4.11.c symmetric verify-bundle — propagate the
    // mutated path_decl into the freshly-parsed MdDescriptor so md-codec
    // wire validation passes for default-inferred non-canonical bundles.
    // Mirror of bundle.rs:1260-1262.
    if is_non_canonical {
        descriptor.path_decl.paths = descriptor_resolved.path_decl.paths.clone();
    }
    verify_emit_from_expected(
        args,
        descriptor,
        &cosigners,
        no_auto_repair,
        json_context,
        stdout,
        stderr,
    )
}

/// Form-agnostic tail shared by both the @N path and the concrete-descriptor
/// path: synthesize the expected Bundle from the already-resolved descriptor +
/// cosigners, then emit the verify checks and write the result.
fn verify_emit_from_expected<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    descriptor: md_codec::Descriptor,
    cosigners: &[crate::synthesize::ResolvedSlot],
    no_auto_repair: bool,
    json_context: bool,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    use crate::synthesize::synthesize_descriptor;
    // SPEC_older_timelock_advisory Task 6 — non-blocking consensus-masked older()
    // advisory (Adapter A). Hooked in the form-agnostic tail so it fires for BOTH
    // the @N-placeholder verify path AND the bare-concrete-descriptor fork (both
    // funnel `descriptor` here as the real, already-canonicalized md-codec parse),
    // before the verify-emit. Read-only: advisory only, never alters the verdict.
    let adv = crate::timelock_advisory::older_advisories_tree(&descriptor);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
    // run_language for verify-bundle: use --language (defaulting to English).
    // cosigners[i].language is None in verify-bundle paths (slots come from
    // mk1 decode + phrase input, not from an ms1 mnem payload). The unwrap_or
    // in synthesize_descriptor correctly falls back to run_language for those
    // slots, matching the emit semantics of the original bundle --descriptor call.
    let run_language: bip39::Language = args.language.unwrap_or_default().into();
    let expected = synthesize_descriptor(
        &descriptor,
        cosigners,
        args.privacy_preserving,
        run_language,
        crate::synthesize::Md1Form::Policy,
    )?;

    // SPEC §5.7: descriptor-mode emits the same 9 / 3+6N schema as template-mode.
    // is_multisig := descriptor.n > 1.
    let supplied = SuppliedCards {
        ms1: &args.ms1,
        mk1: &args.mk1,
        md1: &args.md1,
    };
    let checks = emit_verify_checks(
        &expected,
        &supplied,
        descriptor.n > 1,
        no_auto_repair,
        json_context,
        stdout,
        stderr,
    )?;

    let any_fail = checks.iter().any(|c| !c.passed);
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
        for c in &checks {
            let status = if c.passed { "ok" } else { "fail" };
            if c.detail.is_empty() {
                writeln!(stdout, "{}: {}", c.name, status).ok();
            } else {
                writeln!(stdout, "{}: {} {}", c.name, status, c.detail).ok();
            }
        }
        writeln!(stdout, "result: {}", result_str).ok();
    }
    Ok(if any_fail { 4 } else { 0 })
}

/// Returns true if any two slots share the same (xpub, path) pair.
/// Used by the concrete-descriptor verify-bundle fork to enforce BIP-388
/// distinctness (verify-bundle exits 4, not 2).
fn dup_xpub_path(slots: &[crate::synthesize::ResolvedSlot]) -> bool {
    for i in 0..slots.len() {
        for j in (i + 1)..slots.len() {
            if slots[i].xpub.to_string() == slots[j].xpub.to_string()
                && slots[i].path == slots[j].path
            {
                return true;
            }
        }
    }
    false
}

// ============================================================================
// SPEC v0.9.0 §1 item 1 — argv-leakage closure helpers (mirror bundle.rs)
// ============================================================================

/// Per-occurrence `secret-in-argv` stderr advisory emission for
/// `verify-bundle`. Mirrors `cmd/bundle.rs` shape (one advisory per
/// (flag, slot-index) site).
fn emit_secret_in_argv_advisories<E: std::io::Write>(args: &VerifyBundleArgs, stderr: &mut E) {
    use crate::secret_advisory::secret_in_argv_warning;
    for s in &args.slot {
        if s.subkey.is_secret_bearing() && !s.is_stdin_sentinel() && !s.value.starts_with("@env:") {
            let flag = format!("--slot @{}.{}=", s.index, s.subkey.as_str());
            let alt = format!("--slot @{}.{}=-", s.index, s.subkey.as_str());
            secret_in_argv_warning(stderr, &flag, &alt);
        }
    }
    // v0.26.0 §I1 fold: `--passphrase @env:VAR` is the leak-mitigation
    // channel; do not emit the argv-leak warning for sentinel values.
    if let Some(pp) = args.passphrase.as_deref() {
        if !pp.starts_with("@env:") {
            secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin");
        }
    }
}

fn needs_stdin_substitution(args: &VerifyBundleArgs) -> bool {
    args.passphrase_stdin || args.slot.iter().any(|s| s.is_stdin_sentinel())
}

/// v0.26.0 §3 — cheap pre-check for `@env:` sentinels across `verify-bundle`'s
/// secret-bearing flag surfaces (`--ms1`, `--passphrase`, secret `--slot` values).
/// Returning false avoids the `args.clone()` in the common case where no
/// sentinel is in play.
fn needs_env_sentinel_resolution(args: &VerifyBundleArgs) -> bool {
    let pp = args
        .passphrase
        .as_deref()
        .map(|v| v.starts_with("@env:"))
        .unwrap_or(false);
    let ms1 = args.ms1.iter().any(|v| v.starts_with("@env:"));
    let slot = args
        .slot
        .iter()
        .any(|s| s.subkey.is_secret_bearing() && s.value.starts_with("@env:"));
    pp || ms1 || slot
}

/// v0.26.0 §3 — resolve `@env:<VAR>` sentinels across `verify-bundle`'s
/// secret-bearing flag surfaces. Non-secret flag values (`--mk1`, `--md1`,
/// non-secret slot subkeys, `--network`, etc.) are NOT resolved per SPEC
/// §3.2 (opt-in per-callsite). On any resolution failure, returns the
/// `EnvVarMissing` error with the offending flag name for stderr attribution.
fn resolve_env_sentinels(args: &VerifyBundleArgs) -> Result<VerifyBundleArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    if let Some(pp) = owned.passphrase.as_ref() {
        owned.passphrase = Some(resolve_env_var_sentinel(pp, "--passphrase")?);
    }
    for v in owned.ms1.iter_mut() {
        *v = resolve_env_var_sentinel(v, "--ms1")?;
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

fn apply_stdin_substitutions(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
) -> Result<VerifyBundleArgs, ToolkitError> {
    let mut owned = args.clone();
    let has_slot_stdin = owned.slot.iter().any(|s| s.is_stdin_sentinel());
    if owned.passphrase_stdin && has_slot_stdin {
        return Err(ToolkitError::BadInput(
            "--passphrase-stdin cannot be used with --slot @N.<secret>=- (single stdin per invocation)"
                .into(),
        ));
    }
    if owned.passphrase_stdin {
        let mut buf = String::new();
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        owned.passphrase = Some(buf);
    } else if has_slot_stdin {
        crate::slot_input::apply_slot_stdin(&mut owned.slot, stdin)?;
    }
    Ok(owned)
}

/// v0.4.3 Phase Q: load a `bundle --json` envelope file and synthesize
/// a VerifyBundleArgs with the extracted ms1/mk1/md1 vecs populated. Other
/// args (re-derivation flags --slot/--phrase/etc) are preserved from the
/// caller's args. v0.5: schema-version peek-and-reject deleted; envelopes
/// that don't match the v0.5 schema-4 shape fail at the underlying field
/// extraction (serde-style errors).
fn load_bundle_json_into_args(args: &VerifyBundleArgs) -> Result<VerifyBundleArgs, ToolkitError> {
    let path = args
        .bundle_json
        .as_ref()
        .expect("caller checked bundle_json.is_some()");
    let raw = std::fs::read_to_string(path)
        .map_err(|e| ToolkitError::BadInput(format!("--bundle-json {}: {e}", path.display())))?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
        ToolkitError::BadInput(format!("--bundle-json {} parse: {e}", path.display()))
    })?;
    // Extract ms1 (MsField = Vec<String>) + mk1 (MkField — flat or nested) + md1 (Vec<String>).
    let ms1: Vec<String> = v["ms1"]
        .as_array()
        .ok_or_else(|| ToolkitError::BadInput("--bundle-json ms1 field is not an array".into()))?
        .iter()
        .map(|s| s.as_str().unwrap_or("").to_string())
        .collect();
    // mk1 may be flat (Vec<String>) or nested (Vec<Vec<String>>); both flatten
    // into a single Vec<String> for verify-bundle's --mk1 vec semantics.
    let mk1: Vec<String> = match &v["mk1"] {
        serde_json::Value::Array(arr) => {
            let mut flat = Vec::new();
            for item in arr {
                match item {
                    serde_json::Value::String(s) => flat.push(s.clone()),
                    serde_json::Value::Array(inner) => {
                        for s in inner {
                            if let Some(t) = s.as_str() {
                                flat.push(t.to_string());
                            }
                        }
                    }
                    _ => {
                        return Err(ToolkitError::BadInput(
                            "--bundle-json mk1 element is neither string nor array".into(),
                        ))
                    }
                }
            }
            flat
        }
        _ => {
            return Err(ToolkitError::BadInput(
                "--bundle-json mk1 field is not an array".into(),
            ))
        }
    };
    let md1: Vec<String> = v["md1"]
        .as_array()
        .ok_or_else(|| ToolkitError::BadInput("--bundle-json md1 field is not an array".into()))?
        .iter()
        .map(|s| s.as_str().unwrap_or("").to_string())
        .collect();
    // Construct synthetic args: clone everything from caller, override the
    // card-input fields. bundle_json field is cleared to avoid recursion.
    Ok(VerifyBundleArgs {
        ms1,
        mk1,
        md1,
        bundle_json: None,
        ..args.clone()
    })
}

/// v0.24.0 §2.C.1 — route `extra_strings` positional values to the
/// typed-flag buckets (ms1/mk1/md1) by HRP prefix. Returns a synthetic
/// `VerifyBundleArgs` with the positional values merged into the
/// existing flag-form vectors (flag-form first, then positional). The
/// `extra_strings` field is cleared on the synthetic args.
///
/// Unknown HRPs return `ToolkitError::UnknownHrp` per D34/I5 (toolkit-
/// internal validation; not a clap parser callback).
///
/// Mutual exclusion with `--bundle-json` is enforced at clap-parse time
/// by the `conflicts_with = "bundle_json"` attribute on `extra_strings`.
fn apply_positional_hrp_autodetect(
    args: &VerifyBundleArgs,
) -> Result<VerifyBundleArgs, ToolkitError> {
    let mut ms1 = args.ms1.clone();
    let mut mk1 = args.mk1.clone();
    let mut md1 = args.md1.clone();
    for s in &args.extra_strings {
        match crate::repair::classify_hrp_prefix(s)? {
            crate::repair::CardKind::Ms1 => ms1.push(s.clone()),
            crate::repair::CardKind::Mk1 => mk1.push(s.clone()),
            crate::repair::CardKind::Md1 => md1.push(s.clone()),
        }
    }
    Ok(VerifyBundleArgs {
        ms1,
        mk1,
        md1,
        extra_strings: Vec::new(),
        ..args.clone()
    })
}

// ============================================================================
// v0.4.4 Phase P — emit_verify_checks helper (SPEC §5.7 9 / 3+6N + forensics).
// ============================================================================

use crate::synthesize::Bundle;

/// User-supplied --ms1/--mk1/--md1 vectors packaged for the helper.
/// `mk1[i]` is the mk1 card for cosigner @i (0-indexed); `len(mk1) == N` expected.
pub struct SuppliedCards<'a> {
    pub ms1: &'a [String],
    pub mk1: &'a [String],
    pub md1: &'a [String],
}

/// Cycle F (`ms1-repair-demote-to-candidate`) C1 — ground-truth compare.
///
/// On a supplied-ms1 decode failure, attempt a PURE (non-advisory)
/// substitution-correction via a DIRECT `repair_card` call — deliberately
/// bypassing `try_repair_and_short_circuit`, which would either short-
/// circuit the whole run (pre-Cycle-F ms1 semantics) or (post-Cycle-F) emit
/// the standalone-inline I2 advisory; neither is appropriate here, since the
/// candidate is surfaced via the `ms1_decode`/`ms1_entropy_match` check rows
/// instead (SPEC §0.3).
///
/// Returns `Some(true)` when a correction was applied AND it byte-matches
/// `expected_ms1` (the user's TYPED seed — the ground truth: recovered,
/// confirmed). Returns `Some(false)` when a correction was applied but does
/// NOT match (the wrong-bundle case — SPEC §5.5 funds anchor: the candidate
/// must NOT be blessed). Returns `None` when no correction could be applied
/// (uncorrectable, or the failure wasn't a repair-class error) — the caller
/// falls back to the original decode-error check rows unchanged.
///
/// The corrected string is held in `Zeroizing` (secret-hygiene, proactive —
/// SPEC §8 risk 6 / G5) even though it is only ever compared, never emitted.
fn ms1_ground_truth_compare(supplied_ms1: &str, expected_ms1: &str) -> Option<bool> {
    let outcome =
        crate::repair::repair_card(crate::repair::CardKind::Ms1, &[supplied_ms1.to_string()])
            .ok()?;
    if outcome.repairs.is_empty() {
        // Decoded-and-clean is unreachable from the caller (which only
        // invokes this after `ms_codec::decode` already failed), but guard
        // defensively rather than assume — fall back to the original error.
        return None;
    }
    let corrected = zeroize::Zeroizing::new(
        outcome
            .corrected_chunks
            .first()
            .cloned()
            .unwrap_or_default(),
    );
    Some(corrected.as_str() == expected_ms1)
}

/// SPEC §5.7 verify-bundle check emission. Returns the 9-check array (single-sig)
/// or 3+6N (multisig) per the SPEC's check-name ordering. Forensic fields
/// populated per SPEC §5.7 rules: pass → all None; string-mismatch → expected/
/// actual/diff_byte_offset; decode-failure → decode_error; watch-only short-
/// circuit → passed: true + decode_error: "skipped: watch-only slot".
///
/// `expected.ms1[i].is_empty()` discriminates watch-only slots per SPEC §5.7
/// (the §5.8 MsField wire-format defines the empty-string sentinel; §5.7
/// specifies the watch-only short-circuit semantics in verify-bundle).
/// `is_multisig` selects the 9 vs 3+6N schema.
///
pub fn emit_verify_checks(
    expected: &Bundle,
    supplied: &SuppliedCards,
    is_multisig: bool,
    no_auto_repair: bool,
    json_context: bool,
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> Result<Vec<VerifyCheck>, ToolkitError> {
    if is_multisig {
        return emit_multisig_checks(
            expected,
            supplied,
            no_auto_repair,
            json_context,
            stdout,
            stderr,
        );
    }
    let mut checks = Vec::with_capacity(9);
    let watch_only = expected.ms1.first().map(|s| s.is_empty()).unwrap_or(true);

    // 1. ms1_decode + 2. ms1_entropy_match — both pass-vacuously for watch-only.
    if watch_only {
        checks.push(VerifyCheck {
            name: "ms1_decode".into(),
            passed: true,
            detail: "skipped: watch-only slot".into(),
            decode_error: Some("skipped: watch-only slot".into()),
            ..Default::default()
        });
        checks.push(VerifyCheck {
            name: "ms1_entropy_match".into(),
            passed: true,
            detail: "skipped: watch-only slot".into(),
            decode_error: Some("skipped: watch-only slot".into()),
            ..Default::default()
        });
    } else {
        let supplied_ms1 = supplied.ms1.first().map(|s| s.as_str()).unwrap_or("");
        let expected_ms1 = expected.ms1.first().map(|s| s.as_str()).unwrap_or("");
        match ms_codec::decode(supplied_ms1) {
            Ok(_) => {
                checks.push(VerifyCheck {
                    name: "ms1_decode".into(),
                    passed: true,
                    detail: "decoded successfully".into(),
                    ..Default::default()
                });
                if supplied_ms1 == expected_ms1 {
                    checks.push(VerifyCheck {
                        name: "ms1_entropy_match".into(),
                        passed: true,
                        detail: "ms1 byte-identical".into(),
                        ..Default::default()
                    });
                } else {
                    let diff = VerifyCheck::diff_offset(expected_ms1, supplied_ms1);
                    checks.push(VerifyCheck {
                        name: "ms1_entropy_match".into(),
                        passed: false,
                        detail: "expected ms1 bytes differ from supplied".into(),
                        expected: Some(expected_ms1.to_string()),
                        actual: Some(supplied_ms1.to_string()),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
            }
            Err(e) => {
                // v0.22.1 Phase 4 site #1 → Cycle F C1 (SPEC §0.4/§3) — a
                // direct, non-advisory `repair_card` call replaces the
                // short-circuiting helper here: the corrected candidate is
                // compared against the user's TYPED seed (`expected_ms1`)
                // and surfaced via the check rows, never silently applied.
                let ground_truth = if !no_auto_repair {
                    ms1_ground_truth_compare(supplied_ms1, expected_ms1)
                } else {
                    None
                };
                match ground_truth {
                    Some(true) => {
                        // Match — the repair recovered the right card,
                        // confirmed by the ground truth the user typed.
                        checks.push(VerifyCheck {
                            name: "ms1_decode".into(),
                            passed: true,
                            detail: "recovered via auto-repair, confirmed against expected seed"
                                .into(),
                            ..Default::default()
                        });
                        checks.push(VerifyCheck {
                            name: "ms1_entropy_match".into(),
                            passed: true,
                            detail: "recovered via auto-repair, confirmed against expected seed"
                                .into(),
                            ..Default::default()
                        });
                    }
                    Some(false) => {
                        // Mismatch (SPEC §5.5 funds anchor / wrong-bundle
                        // attack) — a failed check row, full table, exit 4.
                        // REDACTED: no seed bytes, diff_byte_offset pinned
                        // None (G5 — the offset would leak positional info
                        // about two secret strings).
                        checks.push(VerifyCheck {
                            name: "ms1_decode".into(),
                            passed: true,
                            detail: "recovered via auto-repair (unconfirmed)".into(),
                            ..Default::default()
                        });
                        checks.push(VerifyCheck {
                            name: "ms1_entropy_match".into(),
                            passed: false,
                            detail: "auto-repair candidate did not match the expected seed — \
                                this card is not a card for this seed"
                                .into(),
                            ..Default::default()
                        });
                    }
                    None => {
                        // Uncorrectable, or auto-repair disabled — original
                        // decode-error behavior, unchanged.
                        let err_msg = format!("{:?}", e);
                        checks.push(VerifyCheck {
                            name: "ms1_decode".into(),
                            passed: false,
                            detail: err_msg.clone(),
                            decode_error: Some(err_msg),
                            ..Default::default()
                        });
                        checks.push(VerifyCheck {
                            name: "ms1_entropy_match".into(),
                            passed: true,
                            detail: "ms1 decode failed; entropy match cannot run".into(),
                            decode_error: Some("skipped: ms1 decode failed".into()),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    // 3. mk1_decode — must succeed for checks 4/5/6 to run.
    let mk1_strs: Vec<&str> = supplied.mk1.iter().map(|s| s.as_str()).collect();
    let mk_card_result = mk_codec::decode(&mk1_strs);
    match &mk_card_result {
        Ok(_) => {
            checks.push(VerifyCheck {
                name: "mk1_decode".into(),
                passed: true,
                detail: "decoded successfully".into(),
                ..Default::default()
            });
        }
        Err(e) => {
            // v0.22.1 Phase 4 site #2 — auto-fire on supplied mk1 (single-sig) decode-fail.
            if !no_auto_repair {
                let chunks: Vec<String> = supplied.mk1.to_vec();
                crate::repair::try_repair_and_short_circuit(
                    crate::repair::CardKind::Mk1,
                    &chunks,
                    stdout,
                    stderr,
                    json_context,
                )?;
            }
            let err_msg = format!("{:?}", e);
            checks.push(VerifyCheck {
                name: "mk1_decode".into(),
                passed: false,
                detail: err_msg.clone(),
                decode_error: Some(err_msg),
                ..Default::default()
            });
            // 4/5/6 cascade-skipped.
            for n in &["mk1_xpub_match", "mk1_fingerprint_match", "mk1_path_match"] {
                checks.push(VerifyCheck {
                    name: (*n).into(),
                    passed: true,
                    detail: "mk1 decode failed; check cannot run".into(),
                    decode_error: Some("skipped: mk1 decode failed".into()),
                    ..Default::default()
                });
            }
            // Try md1 anyway for diagnostic completeness.
            emit_md1_checks(
                expected,
                supplied,
                &mut checks,
                no_auto_repair,
                json_context,
                stdout,
                stderr,
            )?;
            return Ok(checks);
        }
    }
    let mk_card = mk_card_result.expect("Ok branch handled above");

    // expected.mk1 is MkField::Single for single-sig. Caller invariant: only
    // multisig dispatch passes MkField::Multi (handled in emit_multisig_checks).
    let expected_mk1_strs: Vec<&str> = match &expected.mk1 {
        crate::format::MkField::Single(v) => v.iter().map(|s| s.as_str()).collect(),
        crate::format::MkField::Multi(_) => {
            unreachable!("single-sig branch reached MkField::Multi — caller invariant violation")
        }
    };
    let exp_card = mk_codec::decode(&expected_mk1_strs).expect("expected bundle is well-formed");

    // 4. mk1_xpub_match.
    let exp_xpub = exp_card.xpub.to_string();
    let act_xpub = mk_card.xpub.to_string();
    if exp_xpub == act_xpub {
        checks.push(VerifyCheck {
            name: "mk1_xpub_match".into(),
            passed: true,
            detail: "xpub matches".into(),
            ..Default::default()
        });
    } else {
        let diff = VerifyCheck::diff_offset(&exp_xpub, &act_xpub);
        checks.push(VerifyCheck {
            name: "mk1_xpub_match".into(),
            passed: false,
            detail: "xpub does not match".into(),
            expected: Some(exp_xpub),
            actual: Some(act_xpub),
            diff_byte_offset: Some(diff),
            decode_error: None,
        });
    }

    // 5. mk1_fingerprint_match.
    let exp_fp = exp_card
        .origin_fingerprint
        .map(|f| f.to_string())
        .unwrap_or_default();
    let act_fp = mk_card
        .origin_fingerprint
        .map(|f| f.to_string())
        .unwrap_or_default();
    if exp_fp == act_fp {
        checks.push(VerifyCheck {
            name: "mk1_fingerprint_match".into(),
            passed: true,
            detail: "fingerprint matches".into(),
            ..Default::default()
        });
    } else {
        let diff = VerifyCheck::diff_offset(&exp_fp, &act_fp);
        checks.push(VerifyCheck {
            name: "mk1_fingerprint_match".into(),
            passed: false,
            detail: "fingerprint does not match".into(),
            expected: Some(exp_fp),
            actual: Some(act_fp),
            diff_byte_offset: Some(diff),
            decode_error: None,
        });
    }

    // 6. mk1_path_match.
    let exp_path = exp_card.origin_path.to_string();
    let act_path = mk_card.origin_path.to_string();
    if exp_path == act_path {
        checks.push(VerifyCheck {
            name: "mk1_path_match".into(),
            passed: true,
            detail: "path matches".into(),
            ..Default::default()
        });
    } else {
        let diff = VerifyCheck::diff_offset(&exp_path, &act_path);
        checks.push(VerifyCheck {
            name: "mk1_path_match".into(),
            passed: false,
            detail: "path does not match".into(),
            expected: Some(exp_path),
            actual: Some(act_path),
            diff_byte_offset: Some(diff),
            decode_error: None,
        });
    }

    // 7+8+9: md1.
    emit_md1_checks(
        expected,
        supplied,
        &mut checks,
        no_auto_repair,
        json_context,
        stdout,
        stderr,
    )?;

    Ok(checks)
}

/// SPEC §5.7 multisig 3+6N emission.
///
/// Output ordering: 6N per-cosigner first (interleaved by slot), then 3 shared
/// md1 checks. For each cosigner i in 0..N:
///   ms1_decode[i], ms1_entropy_match[i],
///   mk1_decode[i], mk1_xpub_match[i], mk1_fingerprint_match[i], mk1_path_match[i].
/// Then shared: md1_decode, md1_wallet_policy, md1_xpub_match.
///
/// Watch-only / wif slots (where `expected.ms1[i].is_empty()`): the two ms1
/// checks short-circuit with `passed: true + decode_error: "skipped: watch-only slot"`.
/// Per-cosigner mapping outcome. v0.5 SPEC §5.7 mk1-mapping diagnostic.
/// Precedence when multiple modes apply: `XpubNotInPolicy > DecodeFailed > NotSupplied`.
#[derive(Debug)]
enum MappingFailure {
    NotSupplied,
    DecodeFailed(String),
    XpubNotInPolicy,
}

fn emit_multisig_checks(
    expected: &Bundle,
    supplied: &SuppliedCards,
    no_auto_repair: bool,
    json_context: bool,
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> Result<Vec<VerifyCheck>, ToolkitError> {
    let n = expected.ms1.len();
    let mut checks: Vec<VerifyCheck> = Vec::with_capacity(6 * n + 3);

    // Decode expected.mk1 per-cosigner. expected.mk1 is MkField::Multi(Vec<Vec<String>>)
    // for multisig; on legacy single-element MkField::Single(v) self-multisig
    // bundles, treat the single card as cosigner-0 and emit "missing card"
    // failures for the remaining cosigners.
    let expected_mk1_per_cos: Vec<Option<mk_codec::KeyCard>> = match &expected.mk1 {
        crate::format::MkField::Multi(per_cosigner) => per_cosigner
            .iter()
            .map(|chunks| {
                let strs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
                mk_codec::decode(&strs).ok()
            })
            .collect(),
        crate::format::MkField::Single(v) => {
            let strs: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
            let mut out = vec![mk_codec::decode(&strs).ok()];
            out.resize_with(n, || None);
            out
        }
    };

    // Group supplied.mk1 by chunk_set_id; remember per-group decode outcome
    // (Ok(card) or Err(message)) so the mapping diagnostic can distinguish
    // DecodeFailed from NotSupplied.
    use std::collections::BTreeMap;
    let mut chunked: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
    let mut singles: Vec<Vec<&str>> = Vec::new();
    for s in supplied.mk1 {
        match chunk_set_id_extract(s) {
            Some(csi) => chunked.entry(csi).or_default().push(s.as_str()),
            None => singles.push(vec![s.as_str()]),
        }
    }
    let groups: Vec<Vec<&str>> = chunked.into_values().chain(singles).collect();
    // v0.22.1 Phase 4 site #5 — auto-fire on per-group supplied mk1 decode-fail.
    // The closure-return is `Result<Result<KeyCard, String>, ToolkitError>`:
    // outer Result threads RepairShortCircuit via `?` after collect; inner
    // Result preserves the per-group diagnostic message for MappingFailure.
    let supplied_decoded: Vec<Result<mk_codec::KeyCard, String>> = groups
        .iter()
        .map(
            |g| -> Result<Result<mk_codec::KeyCard, String>, ToolkitError> {
                match mk_codec::decode(g) {
                    Ok(card) => Ok(Ok(card)),
                    Err(e) => {
                        if !no_auto_repair {
                            let chunk_strs: Vec<String> =
                                g.iter().map(|s| (*s).to_string()).collect();
                            crate::repair::try_repair_and_short_circuit(
                                crate::repair::CardKind::Mk1,
                                &chunk_strs,
                                stdout,
                                stderr,
                                json_context,
                            )?;
                        }
                        Ok(Err(format!("{:?}", e)))
                    }
                }
            },
        )
        .collect::<Result<Vec<_>, _>>()?;

    // Decode supplied.md1 once for cosigner-mapping by tlv.pubkeys.
    // v0.22.1 Phase 4 site #6 — auto-fire on supplied md1 (multisig) decode-fail.
    let supplied_md1_strs: Vec<&str> = supplied.md1.iter().map(|s| s.as_str()).collect();
    let supplied_md_decoded = md_codec::chunk::reassemble(&supplied_md1_strs);
    if supplied_md_decoded.is_err() && !no_auto_repair {
        let chunks: Vec<String> = supplied.md1.to_vec();
        crate::repair::try_repair_and_short_circuit(
            crate::repair::CardKind::Md1,
            &chunks,
            stdout,
            stderr,
            json_context,
        )?;
    }

    // B.2: positional fallback condition refactored to match for clarity.
    let needs_positional_fallback = match supplied_md_decoded.as_ref() {
        Err(_) => true,
        Ok(d) => d.tlv.pubkeys.is_none(),
    };

    // Map decoded supplied groups → cosigner positions, tracking failure modes.
    // B.4: Vec<Result<&KeyCard, MappingFailure>> with precedence enforcement.
    let mut card_for_cosigner: Vec<Result<&mk_codec::KeyCard, MappingFailure>> =
        (0..n).map(|_| Err(MappingFailure::NotSupplied)).collect();

    if !needs_positional_fallback {
        let desc = supplied_md_decoded
            .as_ref()
            .expect("Ok per needs_positional_fallback");
        let pubkeys = desc
            .tlv
            .pubkeys
            .as_ref()
            .expect("Some per needs_positional_fallback");
        // First pass: place decoded groups into matching cosigner slots by xpub.
        for (gi, decode_res) in supplied_decoded.iter().enumerate() {
            if let Ok(card) = decode_res {
                let want = crate::synthesize::xpub_to_65(&card.xpub);
                // Prefer slot gi if it matches.
                if let Some((_, b)) = pubkeys.get(gi) {
                    if b == &want
                        && matches!(card_for_cosigner[gi], Err(MappingFailure::NotSupplied))
                    {
                        card_for_cosigner[gi] = Ok(card);
                        continue;
                    }
                }
                // Otherwise scan for first unfilled matching slot.
                if let Some((idx, _)) = pubkeys.iter().find(|(slot, b)| {
                    b == &want
                        && matches!(
                            card_for_cosigner[*slot as usize],
                            Err(MappingFailure::NotSupplied)
                        )
                }) {
                    card_for_cosigner[*idx as usize] = Ok(card);
                } else {
                    // Decoded successfully but xpub not in any policy slot.
                    // Promote any NotSupplied slot to XpubNotInPolicy (precedence).
                    for slot in card_for_cosigner.iter_mut() {
                        if matches!(slot, Err(MappingFailure::NotSupplied)) {
                            *slot = Err(MappingFailure::XpubNotInPolicy);
                            break;
                        }
                    }
                }
            }
        }
        // Second pass: any remaining group with DecodeFailed promotes a NotSupplied slot.
        // Precedence: XpubNotInPolicy > DecodeFailed > NotSupplied.
        for decode_res in &supplied_decoded {
            if let Err(msg) = decode_res {
                for slot in card_for_cosigner.iter_mut() {
                    if matches!(slot, Err(MappingFailure::NotSupplied)) {
                        *slot = Err(MappingFailure::DecodeFailed(msg.clone()));
                        break;
                    }
                }
            }
        }
    } else {
        // Positional fallback: position-i decoded card → Ok; per-position decode error → DecodeFailed.
        for (i, slot) in card_for_cosigner.iter_mut().enumerate().take(n) {
            match supplied_decoded.get(i) {
                Some(Ok(c)) => *slot = Ok(c),
                Some(Err(msg)) => *slot = Err(MappingFailure::DecodeFailed(msg.clone())),
                None => {} // stays NotSupplied
            }
        }
    }

    // 6N per-cosigner emission.
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        let exp_ms1 = expected.ms1.get(i).map(|s| s.as_str()).unwrap_or("");
        let watch_only_slot = exp_ms1.is_empty();
        let sup_ms1 = supplied.ms1.get(i).map(|s| s.as_str());

        // SPEC §5.7 four-case ms1_decode[i] + ms1_entropy_match[i].
        if watch_only_slot {
            // Case 1: watch-only slot — pass-vacuously regardless of supplied.
            checks.push(VerifyCheck {
                name: format!("ms1_decode[{}]", i),
                passed: true,
                detail: "skipped: watch-only slot".into(),
                decode_error: Some("skipped: watch-only slot".into()),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: format!("ms1_entropy_match[{}]", i),
                passed: true,
                detail: "skipped: watch-only slot".into(),
                decode_error: Some("skipped: watch-only slot".into()),
                ..Default::default()
            });
        } else if let Some(s) = sup_ms1.filter(|s| !s.is_empty()) {
            match ms_codec::decode(s) {
                Ok(_) => {
                    // Case 2: full-mode, supplied present, decodes Ok.
                    checks.push(VerifyCheck {
                        name: format!("ms1_decode[{}]", i),
                        passed: true,
                        detail: format!("cosigner[{}] ms1 decoded", i),
                        ..Default::default()
                    });
                    if s == exp_ms1 {
                        checks.push(VerifyCheck {
                            name: format!("ms1_entropy_match[{}]", i),
                            passed: true,
                            detail: format!("cosigner[{}] ms1 byte-identical", i),
                            ..Default::default()
                        });
                    } else {
                        let diff = VerifyCheck::diff_offset(exp_ms1, s);
                        checks.push(VerifyCheck {
                            name: format!("ms1_entropy_match[{}]", i),
                            passed: false,
                            detail: format!("cosigner[{}] ms1 differs", i),
                            expected: Some(exp_ms1.to_string()),
                            actual: Some(s.to_string()),
                            diff_byte_offset: Some(diff),
                            decode_error: None,
                        });
                    }
                }
                Err(e) => {
                    // v0.22.1 Phase 4 site #7 → Cycle F C1 — per-cosigner
                    // ground-truth compare (mirrors the single-sig site
                    // above; see `ms1_ground_truth_compare` doc-comment).
                    let ground_truth = if !no_auto_repair {
                        ms1_ground_truth_compare(s, exp_ms1)
                    } else {
                        None
                    };
                    match ground_truth {
                        Some(true) => {
                            checks.push(VerifyCheck {
                                name: format!("ms1_decode[{}]", i),
                                passed: true,
                                detail: format!(
                                    "cosigner[{}] recovered via auto-repair, confirmed against \
                                        expected seed",
                                    i
                                ),
                                ..Default::default()
                            });
                            checks.push(VerifyCheck {
                                name: format!("ms1_entropy_match[{}]", i),
                                passed: true,
                                detail: format!(
                                    "cosigner[{}] recovered via auto-repair, confirmed against \
                                        expected seed",
                                    i
                                ),
                                ..Default::default()
                            });
                        }
                        Some(false) => {
                            // Mismatch (SPEC §5.5 multisig analogue) — REDACTED
                            // (G5): no seed bytes, diff_byte_offset pinned None.
                            checks.push(VerifyCheck {
                                name: format!("ms1_decode[{}]", i),
                                passed: true,
                                detail: format!(
                                    "cosigner[{}] recovered via auto-repair (unconfirmed)",
                                    i
                                ),
                                ..Default::default()
                            });
                            checks.push(VerifyCheck {
                                name: format!("ms1_entropy_match[{}]", i),
                                passed: false,
                                detail: format!(
                                    "cosigner[{}] auto-repair candidate did not match the \
                                        expected seed — this card is not a card for this seed",
                                    i
                                ),
                                ..Default::default()
                            });
                        }
                        None => {
                            // Case 3: full-mode, supplied present, decodes Err.
                            let err_msg = format!("{:?}", e);
                            checks.push(VerifyCheck {
                                name: format!("ms1_decode[{}]", i),
                                passed: false,
                                detail: err_msg.clone(),
                                decode_error: Some(err_msg),
                                ..Default::default()
                            });
                            checks.push(VerifyCheck {
                                name: format!("ms1_entropy_match[{}]", i),
                                passed: true,
                                detail: format!(
                                    "cosigner[{}] ms1 decode failed; entropy match cannot run",
                                    i
                                ),
                                decode_error: Some("skipped: ms1 decode failed".into()),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        } else {
            // Case 4: full-mode, supplied absent. v0.5 SPEC §5.7 — passed: false.
            checks.push(VerifyCheck {
                name: format!("ms1_decode[{}]", i),
                passed: false,
                detail: format!(
                    "cosigner[{}] ms1 expected (full-mode bundle) but not supplied",
                    i
                ),
                decode_error: Some(format!(
                    "error: ms1[{}] expected (full-mode bundle) but not supplied",
                    i
                )),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: format!("ms1_entropy_match[{}]", i),
                passed: false,
                detail: format!("cosigner[{}] ms1 not supplied", i),
                decode_error: Some(format!("skipped: ms1[{}] not supplied", i)),
                ..Default::default()
            });
        }

        // mk1_decode[i] + mk1_xpub_match[i] + mk1_fingerprint_match[i] + mk1_path_match[i].
        let sup_card_result = &card_for_cosigner[i];
        let exp_card = expected_mk1_per_cos.get(i).and_then(|o| o.as_ref());
        match (sup_card_result, exp_card) {
            (Ok(sup), Some(exp)) => {
                checks.push(VerifyCheck {
                    name: format!("mk1_decode[{}]", i),
                    passed: true,
                    detail: format!("cosigner[{}] mk1 decoded", i),
                    ..Default::default()
                });
                let exp_x = exp.xpub.to_string();
                let act_x = sup.xpub.to_string();
                if exp_x == act_x {
                    checks.push(VerifyCheck {
                        name: format!("mk1_xpub_match[{}]", i),
                        passed: true,
                        detail: format!("cosigner[{}] xpub matches", i),
                        ..Default::default()
                    });
                } else {
                    let diff = VerifyCheck::diff_offset(&exp_x, &act_x);
                    checks.push(VerifyCheck {
                        name: format!("mk1_xpub_match[{}]", i),
                        passed: false,
                        detail: format!("cosigner[{}] xpub mismatch", i),
                        expected: Some(exp_x),
                        actual: Some(act_x),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
                let exp_fp = exp
                    .origin_fingerprint
                    .map(|f| f.to_string())
                    .unwrap_or_default();
                let act_fp = sup
                    .origin_fingerprint
                    .map(|f| f.to_string())
                    .unwrap_or_default();
                if exp_fp == act_fp {
                    checks.push(VerifyCheck {
                        name: format!("mk1_fingerprint_match[{}]", i),
                        passed: true,
                        detail: format!("cosigner[{}] fingerprint matches", i),
                        ..Default::default()
                    });
                } else {
                    let diff = VerifyCheck::diff_offset(&exp_fp, &act_fp);
                    checks.push(VerifyCheck {
                        name: format!("mk1_fingerprint_match[{}]", i),
                        passed: false,
                        detail: format!("cosigner[{}] fingerprint mismatch", i),
                        expected: Some(exp_fp),
                        actual: Some(act_fp),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
                let exp_p = exp.origin_path.to_string();
                let act_p = sup.origin_path.to_string();
                if exp_p == act_p {
                    checks.push(VerifyCheck {
                        name: format!("mk1_path_match[{}]", i),
                        passed: true,
                        detail: format!("cosigner[{}] path matches", i),
                        ..Default::default()
                    });
                } else {
                    let diff = VerifyCheck::diff_offset(&exp_p, &act_p);
                    checks.push(VerifyCheck {
                        name: format!("mk1_path_match[{}]", i),
                        passed: false,
                        detail: format!("cosigner[{}] path mismatch", i),
                        expected: Some(exp_p),
                        actual: Some(act_p),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
            }
            (Err(failure), _) => {
                // SPEC §5.7 mk1-mapping diagnostic: distinguish three failure modes.
                let (detail, decode_error) = match failure {
                    MappingFailure::NotSupplied => (
                        format!("cosigner[{}] mk1 not supplied", i),
                        format!("skipped: mk1[{}] not supplied", i),
                    ),
                    MappingFailure::DecodeFailed(msg) => {
                        (format!("cosigner[{}] mk1 decode failed", i), msg.clone())
                    }
                    MappingFailure::XpubNotInPolicy => (
                        format!(
                            "cosigner[{}] supplied mk1 card xpub absent from descriptor policy",
                            i
                        ),
                        "supplied mk1 card xpub absent from descriptor policy".to_string(),
                    ),
                };
                checks.push(VerifyCheck {
                    name: format!("mk1_decode[{}]", i),
                    passed: false,
                    detail,
                    decode_error: Some(decode_error),
                    ..Default::default()
                });
                // Cascade-skip dependent checks: passed=true (vacuous-skip; no oracle).
                for nm in &["mk1_xpub_match", "mk1_fingerprint_match", "mk1_path_match"] {
                    checks.push(VerifyCheck {
                        name: format!("{}[{}]", nm, i),
                        passed: true,
                        detail: format!("cosigner[{}] mk1 decode failed; cannot evaluate", i),
                        decode_error: Some(format!("skipped: mk1[{}] decode failed", i)),
                        ..Default::default()
                    });
                }
            }
            (Ok(_), None) => {
                // Expected card unavailable (legacy MkField::Single beyond i=0): treat as
                // unknown — supplied card decoded but no comparison oracle.
                checks.push(VerifyCheck {
                    name: format!("mk1_decode[{}]", i),
                    passed: true,
                    detail: format!("cosigner[{}] mk1 decoded; no expected oracle", i),
                    ..Default::default()
                });
                for nm in &["mk1_xpub_match", "mk1_fingerprint_match", "mk1_path_match"] {
                    checks.push(VerifyCheck {
                        name: format!("{}[{}]", nm, i),
                        passed: true,
                        detail: format!("cosigner[{}] no expected mk1 oracle", i),
                        decode_error: Some(format!("skipped: expected mk1[{}] not available", i)),
                        ..Default::default()
                    });
                }
            }
        }
    }

    // 3 shared md1 checks.
    let expected_md1_strs: Vec<&str> = expected.md1.iter().map(|s| s.as_str()).collect();
    let expected_md_decoded =
        md_codec::chunk::reassemble(&expected_md1_strs).expect("expected bundle is well-formed");

    match supplied_md_decoded.as_ref() {
        Ok(desc) => {
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                passed: true,
                detail: "decoded successfully".into(),
                ..Default::default()
            });
            let wp = desc.is_wallet_policy();
            if wp {
                checks.push(VerifyCheck {
                    name: "md1_wallet_policy".into(),
                    passed: true,
                    detail: "wallet-policy mode confirmed".into(),
                    ..Default::default()
                });
                // md1_xpub_match — H1 (cycle-1): widened from a sorted-pubkey-
                // multiset-only compare (B.3 SPEC §5.7) to ALSO require a
                // STRUCTURAL decoded-policy compare. The legacy multiset gate
                // GREEN-lit a supplied md1 that reconstructs a DIFFERENT wallet
                // (wrong threshold / sorted-vs-unsorted / script-type-wrapper /
                // divergent change-chain multipath) whenever its cosigner pubkey
                // SET matched — a false-assurance hole. The widened gate compares
                // the derived `md_codec::Descriptor` structure:
                //   - `tree ==`            → Tag (Multi/SortedMulti/Tr/…) + threshold
                //                            `k` + script-type wrapper + nesting
                //                            (distinguishes `sh(multi)` from
                //                            `sh(wsh(multi))`) + index-aware per-slot
                //                            binding (`Body::MultiKeys.indices` order).
                //   - `use_site_path ==`   → the change-chain / multipath
                //                            alternatives (`<0;1>` vs `<2;3>`,
                //                            presence/count) that fix the WATCHED
                //                            ADDRESS SET (md-codec `derive_address`).
                //   - per-`@N` `tlv.use_site_path_overrides ==` → the per-cosigner
                //                            override paths that ALSO fix the
                //                            address set (the #25/#26 silent-wrong-
                //                            address class).
                // Origins (`path_decl` + the origin/fingerprint TLV columns) are
                // EXCLUDED: they carry legitimate elision/canonicalization variance
                // (bug-hunt L14) and binding them would false-FAIL a legit
                // origin-elided descriptor-mode backup — the exact class the v0.5.0
                // B.3 multiset change exists to tolerate.
                //
                // FALSE-FAIL SAFETY BASIS — the DECODE BOUNDARY, not field stability:
                // both operands here are DECODED md1 (`expected_md_decoded` /
                // `desc`). md-codec's decoder enforces canonical form on every
                // `Descriptor` (ascending placeholder first-occurrences; rejected
                // `@0`/baseline-redundant overrides; ascending TLV idx column), so
                // two semantically-identical wallets decode to byte-identical `tree`
                // labels AND `use_site_path_overrides` `@N` keys. Hence `==` over
                // these decoded fields cannot false-fail a legitimately-equal
                // wallet. (The baseline `use_site_path` is additionally intrinsically
                // stable — it has no in-field canonicalization; the override map IS
                // canonicalization-touched but is made `==`-safe by this same decode
                // boundary. Per plan-doc R0 round-2 m-NEW-1: the precise basis is
                // decode-boundary canonicalization, NOT `validate_multipath_consistency`,
                // which only checks alt-COUNT equality.)
                let policy_match = expected_md_decoded.tree == desc.tree
                    && expected_md_decoded.use_site_path == desc.use_site_path
                    && expected_md_decoded.tlv.use_site_path_overrides
                        == desc.tlv.use_site_path_overrides;
                let exp_pubs: Vec<[u8; 65]> = expected_md_decoded
                    .tlv
                    .pubkeys
                    .as_ref()
                    .map(|v| v.iter().map(|(_, b)| *b).collect())
                    .unwrap_or_default();
                let act_pubs: Vec<[u8; 65]> = desc
                    .tlv
                    .pubkeys
                    .as_ref()
                    .map(|v| v.iter().map(|(_, b)| *b).collect())
                    .unwrap_or_default();
                let mut exp_sorted = exp_pubs.clone();
                let mut act_sorted = act_pubs.clone();
                exp_sorted.sort();
                act_sorted.sort();
                let pubkeys_match = exp_sorted == act_sorted;
                // NAME unchanged (`md1_xpub_match`) per Q-WIRE — only the `passed`
                // predicate widens; the `--json` `checks[]` wire-shape is identical.
                if policy_match && pubkeys_match {
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: true,
                        detail: format!(
                            "all {} pubkeys match expected (multiset) and decoded policy \
                             (tree + use-site path) matches",
                            exp_pubs.len()
                        ),
                        ..Default::default()
                    });
                } else if !policy_match {
                    // Structural divergence (wrong threshold / sorted-vs-unsorted /
                    // script-type-wrapper / divergent change-chain multipath /
                    // per-`@N` use-site override). Reconstructs a DIFFERENT wallet
                    // even if the pubkey SET matched.
                    let mut classes: Vec<&str> = Vec::new();
                    if expected_md_decoded.tree != desc.tree {
                        classes.push("policy tree (threshold/script-type/order)");
                    }
                    if expected_md_decoded.use_site_path != desc.use_site_path {
                        classes.push("use-site path (change-chain/multipath)");
                    }
                    if expected_md_decoded.tlv.use_site_path_overrides
                        != desc.tlv.use_site_path_overrides
                    {
                        classes.push("per-cosigner use-site override");
                    }
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: false,
                        detail: format!(
                            "md1 reconstructs a DIFFERENT wallet — structural policy mismatch: {}",
                            classes.join(", ")
                        ),
                        expected: Some(format!("{:?}", expected_md_decoded.tree)),
                        actual: Some(format!("{:?}", desc.tree)),
                        diff_byte_offset: None,
                        decode_error: None,
                    });
                } else {
                    let exp_hex = exp_pubs
                        .iter()
                        .map(hex::encode)
                        .collect::<Vec<_>>()
                        .join(",");
                    let act_hex = act_pubs
                        .iter()
                        .map(hex::encode)
                        .collect::<Vec<_>>()
                        .join(",");
                    let diff = VerifyCheck::diff_offset(&exp_hex, &act_hex);
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: false,
                        detail: "md1 pubkeys differ from expected set".into(),
                        expected: Some(exp_hex),
                        actual: Some(act_hex),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
            } else {
                checks.push(VerifyCheck {
                    name: "md1_wallet_policy".into(),
                    passed: false,
                    detail: "descriptor is template-only (no pubkeys TLV)".into(),
                    decode_error: Some("not in wallet-policy mode".into()),
                    ..Default::default()
                });
                checks.push(VerifyCheck {
                    name: "md1_xpub_match".into(),
                    passed: true,
                    detail: "skipped: not in wallet-policy mode".into(),
                    decode_error: Some("skipped: not in wallet-policy mode".into()),
                    ..Default::default()
                });
            }
        }
        Err(e) => {
            let err_msg = format!("{:?}", e);
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                passed: false,
                detail: err_msg.clone(),
                decode_error: Some(err_msg),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                passed: true,
                detail: "skipped: md1 decode failed".into(),
                decode_error: Some("skipped: md1 decode failed".into()),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match".into(),
                passed: true,
                detail: "skipped: md1 decode failed".into(),
                decode_error: Some("skipped: md1 decode failed".into()),
                ..Default::default()
            });
        }
    }

    Ok(checks)
}

/// Emit md1_decode + md1_wallet_policy + md1_xpub_match (checks 7-9 of SPEC §5.7).
fn emit_md1_checks(
    expected: &Bundle,
    supplied: &SuppliedCards,
    checks: &mut Vec<VerifyCheck>,
    no_auto_repair: bool,
    json_context: bool,
    stdout: &mut dyn std::io::Write,
    stderr: &mut dyn std::io::Write,
) -> Result<(), ToolkitError> {
    let supplied_md1: Vec<&str> = supplied.md1.iter().map(|s| s.as_str()).collect();
    match md_codec::chunk::reassemble(&supplied_md1) {
        Ok(desc) => {
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                passed: true,
                detail: "decoded successfully".into(),
                ..Default::default()
            });
            let wp = desc.is_wallet_policy();
            if wp {
                checks.push(VerifyCheck {
                    name: "md1_wallet_policy".into(),
                    passed: true,
                    detail: "wallet-policy mode confirmed".into(),
                    ..Default::default()
                });
                // 9. md1_xpub_match — compare descriptor's first pubkey to expected mk1's xpub.
                let expected_md1: Vec<&str> = expected.md1.iter().map(|s| s.as_str()).collect();
                let exp_desc = md_codec::chunk::reassemble(&expected_md1)
                    .expect("expected bundle is well-formed");
                let exp_xpub = exp_desc
                    .tlv
                    .pubkeys
                    .as_ref()
                    .and_then(|v| v.first())
                    .map(|(_, b)| *b);
                let act_xpub = desc
                    .tlv
                    .pubkeys
                    .as_ref()
                    .and_then(|v| v.first())
                    .map(|(_, b)| *b);
                let xpub_match = exp_xpub == act_xpub;
                if xpub_match {
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: true,
                        detail: "65-byte xpub matches expected".into(),
                        ..Default::default()
                    });
                } else {
                    let exp_hex = exp_xpub.map(hex::encode).unwrap_or_default();
                    let act_hex = act_xpub.map(hex::encode).unwrap_or_default();
                    let diff = VerifyCheck::diff_offset(&exp_hex, &act_hex);
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: false,
                        detail: "md1 xpub differs from expected".into(),
                        expected: Some(exp_hex),
                        actual: Some(act_hex),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
            } else {
                checks.push(VerifyCheck {
                    name: "md1_wallet_policy".into(),
                    passed: false,
                    detail: "descriptor is template-only (no pubkeys TLV)".into(),
                    decode_error: Some("not in wallet-policy mode".into()),
                    ..Default::default()
                });
                checks.push(VerifyCheck {
                    name: "md1_xpub_match".into(),
                    passed: true,
                    detail: "skipped: not in wallet-policy mode".into(),
                    decode_error: Some("skipped: not in wallet-policy mode".into()),
                    ..Default::default()
                });
            }
        }
        Err(e) => {
            // v0.22.1 Phase 4 site #8 — auto-fire on supplied md1 decode-fail.
            if !no_auto_repair {
                let chunks: Vec<String> = supplied.md1.to_vec();
                crate::repair::try_repair_and_short_circuit(
                    crate::repair::CardKind::Md1,
                    &chunks,
                    stdout,
                    stderr,
                    json_context,
                )?;
            }
            let err_msg = format!("{:?}", e);
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                passed: false,
                detail: err_msg.clone(),
                decode_error: Some(err_msg),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                passed: true,
                detail: "skipped: md1 decode failed".into(),
                decode_error: Some("skipped: md1 decode failed".into()),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match".into(),
                passed: true,
                detail: "skipped: md1 decode failed".into(),
                decode_error: Some("skipped: md1 decode failed".into()),
                ..Default::default()
            });
        }
    }
    Ok(())
}

// ============================================================================
// v0.24.0 sub-item 1 — D30 watch-only xpub↔path cross-check.
// ============================================================================

/// Watch-only defense-in-depth cross-check between supplied mk1 cards and the
/// supplied md1 card. Operates entirely on the decoded structs (no seed
/// required); emits stderr WARNING lines for each detected inconsistency.
///
/// Closes `verify-bundle-watch-only-xpub-path-internal-consistency` (D30
/// tier upgrade from `v1+` to `v0.24.0`). Distinct from the existing
/// "compare each card against a synthesized expected Bundle" path: that path
/// holds when the user-supplied template + slots match the cards' origin;
/// the cross-check below is independent of the synthesized expectation and
/// catches mk1↔md1 internal inconsistency even when both cards happen to
/// agree with the synthesized Bundle (e.g. via tampering on both sides).
///
/// Three cross-checks per cosigner (all on already-decoded fields):
///   1. mk1.xpub.depth == md1 OriginPath length.
///   2. mk1.xpub.child_number == md1 OriginPath last component
///      (value + hardened bit).
///   3. mk1.xpub.parent_fingerprint sanity: at depth 0 it must be all-zeros
///      (BIP-32 master invariant); at depth 1 it must equal mk1's claimed
///      origin_fingerprint (the master fingerprint) when the latter is
///      supplied. Deeper paths skip this check (would require deriving the
///      parent xpub, which the watch-only path cannot do).
///
/// Failure mode: stderr WARNING (not hard error). Matches existing watch-only
/// stderr disclaimer pattern (see `run_watch_only` and `run_multisig`'s
/// watch-only branch). The verify-bundle exit code is unchanged.
fn emit_watch_only_xpub_path_cross_check<E: std::io::Write>(
    supplied_mk1: &[String],
    supplied_md1: &[String],
    is_multisig: bool,
    stderr: &mut E,
) {
    // Decode md1; bail silently on failure — the regular `md1_decode` check
    // path will surface decode errors via the VerifyCheck schema.
    let md1_strs: Vec<&str> = supplied_md1.iter().map(|s| s.as_str()).collect();
    let desc = match md_codec::chunk::reassemble(&md1_strs) {
        Ok(d) => d,
        Err(_) => return,
    };

    // Map of cosigner index → md1's OriginPath. Use TLV
    // origin_path_overrides if present (per-`@N` override), else path_decl.
    let n = desc.n as usize;
    let md_path_for = |idx: usize| -> Option<md_codec::origin_path::OriginPath> {
        if let Some(overrides) = &desc.tlv.origin_path_overrides {
            if let Some((_, op)) = overrides.iter().find(|(i, _)| *i as usize == idx) {
                return Some(op.clone());
            }
        }
        match &desc.path_decl.paths {
            md_codec::origin_path::PathDeclPaths::Shared(op) => Some(op.clone()),
            md_codec::origin_path::PathDeclPaths::Divergent(v) => v.get(idx).cloned(),
        }
    };

    // Map of cosigner index → claimed master fingerprint (TLV fingerprints,
    // wallet-policy mode only).
    let md_fp_for = |idx: usize| -> Option<[u8; 4]> {
        desc.tlv.fingerprints.as_ref().and_then(|v| {
            v.iter()
                .find(|(i, _)| *i as usize == idx)
                .map(|(_, fp)| *fp)
        })
    };

    // Decode supplied mk1 cards. For multisig, group by chunk_set_id (mirrors
    // emit_multisig_checks's grouping logic at the top of that function).
    let mk_cards: Vec<(usize, mk_codec::KeyCard)> = if is_multisig {
        use std::collections::BTreeMap;
        let mut chunked: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
        let mut singles: Vec<Vec<&str>> = Vec::new();
        for s in supplied_mk1 {
            match chunk_set_id_extract(s) {
                Some(csi) => chunked.entry(csi).or_default().push(s.as_str()),
                None => singles.push(vec![s.as_str()]),
            }
        }
        let groups: Vec<Vec<&str>> = chunked.into_values().chain(singles).collect();
        let mut out: Vec<(usize, mk_codec::KeyCard)> = Vec::new();
        // Map each decoded card to its cosigner index via md1.tlv.pubkeys
        // when in wallet-policy mode, else positional.
        let pubkeys = desc.tlv.pubkeys.as_ref();
        let mut assigned = vec![false; n];
        for (gi, g) in groups.iter().enumerate() {
            let card = match mk_codec::decode(g) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let want = crate::synthesize::xpub_to_65(&card.xpub);
            let mut placed_idx: Option<usize> = None;
            if let Some(pubs) = pubkeys {
                if let Some((slot, _)) = pubs.iter().find(|(slot, b)| {
                    b == &want && (*slot as usize) < n && !assigned[*slot as usize]
                }) {
                    placed_idx = Some(*slot as usize);
                }
            }
            if placed_idx.is_none() && gi < n && !assigned[gi] {
                placed_idx = Some(gi);
            }
            if let Some(idx) = placed_idx {
                assigned[idx] = true;
                out.push((idx, card));
            }
        }
        out
    } else {
        match mk_codec::decode(&mk1_strs_to_str_refs(supplied_mk1)) {
            Ok(card) => vec![(0, card)],
            Err(_) => Vec::new(),
        }
    };

    for (i, card) in &mk_cards {
        let md_path = match md_path_for(*i) {
            Some(p) => p,
            None => continue,
        };

        // Check 1 (overlap-prefix, v0.37.10): compare the decoded mk1 origin_path
        // against md1's origin on min(len). One is a prefix of the other by
        // construction (3→4 truncate: mk1 ⊆ md1; 4→3 extend: md1 ⊆ mk1; 4→4: equal),
        // so a depth difference is the legitimate truncation/extension/under-
        // annotation shape — NOT flagged. Only a genuine disagreement on the shared
        // prefix is an inconsistency. This subsumes the old depth + terminal-child
        // checks (the mk1 path's length is xpub.depth and its terminal is
        // xpub.child_number, by the mk-codec 0.4.0 encode guard).
        let d = card.xpub.depth as usize;
        let mk_comps: Vec<bitcoin::bip32::ChildNumber> =
            card.origin_path.into_iter().copied().collect();
        // zip stops at the shorter (= the overlap = min(len)); compare each shared
        // component. enumerate gives the 0-based index for the warning message.
        for (k, (mk_c, md_c)) in mk_comps.iter().zip(md_path.components.iter()).enumerate() {
            let (mi, mh) = match *mk_c {
                bitcoin::bip32::ChildNumber::Normal { index } => (index, false),
                bitcoin::bip32::ChildNumber::Hardened { index } => (index, true),
            };
            if mi != md_c.value || mh != md_c.hardened {
                writeln!(
                    stderr,
                    "warning: cosigner[{}] mk1 origin-path component #{} ({}{}) does not match md1 ({}{}); cards are internally inconsistent",
                    i,
                    k + 1,
                    mi,
                    if mh { "'" } else { "" },
                    md_c.value,
                    if md_c.hardened { "'" } else { "" },
                )
                .ok();
                break; // one warning per cosigner
            }
        }

        // Check 2: parent_fingerprint structural sanity, keyed off the xpub's OWN
        // depth d (NOT md_depth). Depth >= 2 is verified by
        // emit_full_path_parent_fingerprint_check (needs ms1 to derive the parent).
        let pfp = card.xpub.parent_fingerprint.to_bytes();
        if d == 0 {
            // Master xpub MUST have all-zero parent_fingerprint per BIP-32.
            if pfp != [0u8; 4] {
                writeln!(
                    stderr,
                    "warning: cosigner[{}] mk1 xpub parent_fingerprint ({}) is non-zero at depth 0 (expected 00000000); cards are internally inconsistent",
                    i, hex::encode(pfp)
                )
                .ok();
            }
        } else if d == 1 {
            // At depth 1, parent IS the master. Cross-check against the master
            // fingerprint claimed by md1 (TLV fingerprints) or mk1 (origin_fingerprint).
            let claimed_master_fp =
                md_fp_for(*i).or_else(|| card.origin_fingerprint.map(|f| f.to_bytes()));
            if let Some(master_fp) = claimed_master_fp {
                if pfp != master_fp {
                    writeln!(
                        stderr,
                        "warning: cosigner[{}] mk1 xpub parent_fingerprint ({}) does not match claimed master fingerprint ({}) at depth 1; cards are internally inconsistent",
                        i,
                        hex::encode(pfp),
                        hex::encode(master_fp),
                    )
                    .ok();
                }
            }
        }
        // Deeper paths (depth >= 2) skip here; emit_full_path_parent_fingerprint_check
        // derives the parent from the seed (ms1) when available.
    }
}

/// Single-sig mk1 decode helper for `emit_watch_only_xpub_path_cross_check`.
/// Pulled into a free function to dodge a borrow-checker issue caused by
/// constructing the `Vec<&str>` inline at the match arm.
fn mk1_strs_to_str_refs(v: &[String]) -> Vec<&str> {
    v.iter().map(|s| s.as_str()).collect()
}

// ============================================================================
// v0.25.0 §2.D Tranche #1 — ms1-driven parent_fingerprint check at depth ≥ 2.
// ============================================================================

/// Defense-in-depth check that extends v0.24.0's `emit_watch_only_xpub_path_cross_check`
/// at depth ≥ 2, where the parent xpub cannot be recovered from the supplied
/// mk1 alone (BIP-32 child→parent derivation is one-way). For each cosigner
/// with `path.len() >= 2`:
///
/// * **Full-path mode (ms1 supplied + non-empty):** decode ms1 → BIP-39
///   mnemonic in the bundle's language → master seed (passphrase-aware) →
///   master xpriv at the bundle's network → derive parent xpriv at the
///   `path[..N-1]` prefix → compute the parent xpub's fingerprint → compare
///   against the claimed `mk1.xpub.parent_fingerprint`. Emit a stderr WARNING
///   on mismatch.
/// * **Watch-only mode (ms1 absent / empty for this cosigner):** emit a
///   stderr NOTICE marking the parent_fingerprint as unverified-by-design
///   (cryptographic ceiling per BIP-32 child→parent one-wayness; no seed →
///   no derivation possible).
///
/// Failure mode: stderr WARNING / NOTICE (not hard error). The verify-bundle
/// exit code and `result: ok / mismatch` verdict are UNCHANGED — matches the
/// permissive-input / expressive-output philosophy + the existing v0.24.0
/// cross-check pattern.
///
/// Closes FOLLOWUP `verify-bundle-xpub-parent-fingerprint-derivation` (the
/// original "derive parent from mk1" framing was structurally impossible;
/// corrected to ms1-driven derivation, with explicit wontfix partition for
/// the watch-only ceiling).
#[allow(clippy::too_many_arguments)]
fn emit_full_path_parent_fingerprint_check<E: std::io::Write>(
    supplied_ms1: &[String],
    supplied_mk1: &[String],
    supplied_md1: &[String],
    is_multisig: bool,
    passphrase: Option<&str>,
    // `Some(x)` = user explicitly supplied `--language x`; `None` = defaulted.
    language_opt: Option<CliLanguage>,
    network: CliNetwork,
    stderr: &mut E,
) {
    let language = language_opt.unwrap_or_default();
    use bitcoin::bip32::{Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;

    // Decode md1; bail silently on failure — regular `md1_decode` check path
    // surfaces decode errors via the VerifyCheck schema.
    let md1_strs: Vec<&str> = supplied_md1.iter().map(|s| s.as_str()).collect();
    let desc = match md_codec::chunk::reassemble(&md1_strs) {
        Ok(d) => d,
        Err(_) => return,
    };

    // Map of cosigner index → md1's OriginPath. Mirrors the lookup pattern in
    // `emit_watch_only_xpub_path_cross_check`.
    let n = desc.n as usize;
    let md_path_for = |idx: usize| -> Option<md_codec::origin_path::OriginPath> {
        if let Some(overrides) = &desc.tlv.origin_path_overrides {
            if let Some((_, op)) = overrides.iter().find(|(i, _)| *i as usize == idx) {
                return Some(op.clone());
            }
        }
        match &desc.path_decl.paths {
            md_codec::origin_path::PathDeclPaths::Shared(op) => Some(op.clone()),
            md_codec::origin_path::PathDeclPaths::Divergent(v) => v.get(idx).cloned(),
        }
    };

    // Decode supplied mk1 cards, grouping by chunk_set_id for multisig
    // (mirror `emit_watch_only_xpub_path_cross_check`'s grouping logic).
    let mk_cards: Vec<(usize, mk_codec::KeyCard)> = if is_multisig {
        use std::collections::BTreeMap;
        let mut chunked: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
        let mut singles: Vec<Vec<&str>> = Vec::new();
        for s in supplied_mk1 {
            match chunk_set_id_extract(s) {
                Some(csi) => chunked.entry(csi).or_default().push(s.as_str()),
                None => singles.push(vec![s.as_str()]),
            }
        }
        let groups: Vec<Vec<&str>> = chunked.into_values().chain(singles).collect();
        let mut out: Vec<(usize, mk_codec::KeyCard)> = Vec::new();
        let pubkeys = desc.tlv.pubkeys.as_ref();
        let mut assigned = vec![false; n];
        for (gi, g) in groups.iter().enumerate() {
            let card = match mk_codec::decode(g) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let want = crate::synthesize::xpub_to_65(&card.xpub);
            let mut placed_idx: Option<usize> = None;
            if let Some(pubs) = pubkeys {
                if let Some((slot, _)) = pubs.iter().find(|(slot, b)| {
                    b == &want && (*slot as usize) < n && !assigned[*slot as usize]
                }) {
                    placed_idx = Some(*slot as usize);
                }
            }
            if placed_idx.is_none() && gi < n && !assigned[gi] {
                placed_idx = Some(gi);
            }
            if let Some(idx) = placed_idx {
                assigned[idx] = true;
                out.push((idx, card));
            }
        }
        out
    } else {
        match mk_codec::decode(&mk1_strs_to_str_refs(supplied_mk1)) {
            Ok(card) => vec![(0, card)],
            Err(_) => Vec::new(),
        }
    };

    let secp = Secp256k1::new();

    for (i, card) in &mk_cards {
        let md_path = match md_path_for(*i) {
            Some(p) => p,
            None => continue,
        };
        // Keyed off the xpub's OWN depth d (v0.37.10): the mk1 card's parent is at
        // depth d-1, not md_depth-1 (md1 may be deeper/shallower than the xpub).
        let d = card.xpub.depth as usize;
        if d < 2 {
            // Depth 0/1 handled by `emit_watch_only_xpub_path_cross_check`'s Check 2.
            continue;
        }

        let ms1_str = supplied_ms1.get(*i).map(|s| s.as_str()).unwrap_or("");

        if ms1_str.is_empty() {
            // Watch-only at depth ≥ 2: emit expressive notice (cryptographic
            // ceiling per BIP-32 child→parent one-wayness).
            writeln!(
                stderr,
                "notice: cosigner[{}] mk1 parent_fingerprint at depth {} unverified (requires ms1 to derive parent xpub)",
                i,
                d
            )
            .ok();
            continue;
        }

        // Full-path: ms1 supplied — derive parent xpub from seed.
        // ms mnem Phase 3 (R2-I7): widen match to bind BOTH Entr and Mnem payloads;
        // a mnem cosigner card previously silently `continue`d, skipping the cross-check.
        let (entropy, card_lang) = match ms_codec::decode(ms1_str) {
            Ok((_tag, ms_codec::Payload::Entr(bytes))) => (bytes, language.into()),
            Ok((
                _tag,
                ms_codec::Payload::Mnem {
                    language: wire_lang,
                    entropy,
                },
            )) => {
                // Per-card wire language wins over run-level --language.
                let lang = match crate::language::wire_code_to_bip39(wire_lang) {
                    Ok(l) => l,
                    Err(_) => continue, // invalid wire code — skip silently
                };
                // Wire-wins note: emit if --language was explicit AND differs.
                if let Some(cli_lang) = language_opt {
                    let cli_bip39: bip39::Language = cli_lang.into();
                    if cli_bip39 != lang {
                        let wire_name = ms_codec::consts::MNEM_LANGUAGE_NAMES
                            .get(wire_lang as usize)
                            .copied()
                            .unwrap_or("unknown");
                        let _ = writeln!(
                            stderr,
                            "note: cosigner[{i}] ms1 carries wordlist language {wire_name}; \
                             ignoring --language {}",
                            cli_lang.human_name()
                        );
                    }
                }
                (entropy, lang)
            }
            // ms1 didn't decode — the regular ms1_decode check surfaces errors
            // via VerifyCheck; skip silently here so we don't double-report.
            Err(_) => continue,
            // Forward-compat: unknown future payload kinds — skip silently.
            Ok(_) => continue,
        };
        let entropy: Vec<u8> = entropy;

        // entropy → mnemonic → seed → master xpriv. Mirrors descriptor-mode
        // verify path at `derive_slot::derive_bip32_from_entropy`.
        let mnemonic = match bip39::Mnemonic::from_entropy_in(card_lang, &entropy) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
        let master = match Xpriv::new_master(network.network_kind(), &seed[..]) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // Convert md1 OriginPath → bitcoin DerivationPath, then truncate to the
        // xpub's PARENT level (full[..d-1]), not md_depth-1: the mk1 card's xpub is
        // at depth d, so its parent is at depth d-1. d == full.len()+1 (the 4→3 leaf
        // one below md1's origin) is valid — full[..d-1] = all of full = the parent.
        let full_path = match crate::cmd::bundle::origin_to_derivation_path(&md_path) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let full_components: Vec<bitcoin::bip32::ChildNumber> =
            full_path.into_iter().copied().collect();
        if d - 1 > full_components.len() {
            // The xpub claims a node ≥2 levels below md1's origin; can't form the
            // parent prefix. (Check 1's overlap-prefix already covers consistency.)
            continue;
        }
        let parent_components: Vec<bitcoin::bip32::ChildNumber> = full_components[..d - 1].to_vec();
        let parent_path = bitcoin::bip32::DerivationPath::from(parent_components);

        let parent_xpriv = match master.derive_priv(&secp, &parent_path) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let parent_xpub = Xpub::from_priv(&secp, &parent_xpriv);
        let derived_fp = parent_xpub.fingerprint().to_bytes();
        let claimed_fp = card.xpub.parent_fingerprint.to_bytes();

        if derived_fp != claimed_fp {
            writeln!(
                stderr,
                "warning: cosigner[{}] mk1 xpub parent_fingerprint ({}) does not match derived parent fingerprint ({}) from ms1 at depth {}; cards are internally inconsistent",
                i,
                hex::encode(claimed_fp),
                hex::encode(derived_fp),
                d
            )
            .ok();
        }
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;
    use crate::format::MkField;
    use crate::network::CliNetwork;
    use crate::synthesize::synthesize_full;
    use crate::template::CliTemplate;
    use bip39::Mnemonic;
    use bitcoin::bip32::{Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;

    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    fn synth_full_bundle() -> Bundle {
        synth_full_bundle_for(TREZOR_24)
    }

    /// Cycle F — parameterized variant of `synth_full_bundle` for building a
    /// SECOND, DISTINCT wallet's bundle (the §5.5 wrong-bundle fixtures need
    /// two independently-derived bundles).
    fn synth_full_bundle_for(phrase: &str) -> Bundle {
        let m = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
        let entropy = m.to_entropy();
        let seed = m.to_seed("");
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        let fp = master.fingerprint(&secp);
        let path = CliTemplate::Bip84.derivation_path(CliNetwork::Mainnet, 0);
        let acct_xpriv = master.derive_priv(&secp, &path).unwrap();
        let xpub = Xpub::from_priv(&secp, &acct_xpriv);
        synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap()
    }

    #[test]
    fn helper_singlesig_full_emits_9_checks_in_spec_order() {
        let expected = synth_full_bundle();
        let supplied_ms1 = expected.ms1.clone();
        let supplied_mk1 = match &expected.mk1 {
            MkField::Single(v) => v.clone(),
            MkField::Multi(_) => panic!("expected single-sig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let mut _test_so: Vec<u8> = Vec::new();
        let mut _test_se: Vec<u8> = Vec::new();
        let checks = emit_verify_checks(
            &expected,
            &supplied,
            false,
            true,
            false,
            &mut _test_so,
            &mut _test_se,
        )
        .unwrap();
        assert_eq!(
            checks.len(),
            9,
            "single-sig must emit 9 checks per SPEC §5.7"
        );
        let names: Vec<&str> = checks.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "ms1_decode",
                "ms1_entropy_match",
                "mk1_decode",
                "mk1_xpub_match",
                "mk1_fingerprint_match",
                "mk1_path_match",
                "md1_decode",
                "md1_wallet_policy",
                "md1_xpub_match",
            ],
            "checks must be in SPEC §5.7 order"
        );
        assert!(
            checks.iter().all(|c| c.passed),
            "happy-path bundle must produce all-passed checks"
        );
    }

    #[test]
    fn helper_singlesig_tampered_mk1_populates_forensics() {
        let expected = synth_full_bundle();
        let supplied_ms1 = expected.ms1.clone();
        // Tamper: replace the last char with '0'.
        let supplied_mk1: Vec<String> = match &expected.mk1 {
            MkField::Single(v) => v
                .iter()
                .map(|s| {
                    let mut s = s.clone();
                    s.pop();
                    s.push('0');
                    s
                })
                .collect(),
            MkField::Multi(_) => panic!("expected single-sig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let mut _test_so: Vec<u8> = Vec::new();
        let mut _test_se: Vec<u8> = Vec::new();
        let checks = emit_verify_checks(
            &expected,
            &supplied,
            false,
            true,
            false,
            &mut _test_so,
            &mut _test_se,
        )
        .unwrap();
        // Either mk1_decode fails (BCH checksum mismatch) OR mk1_xpub_match fails.
        let mk1_decode = checks
            .iter()
            .find(|c| c.name == "mk1_decode")
            .expect("mk1_decode present");
        if !mk1_decode.passed {
            assert!(
                mk1_decode.decode_error.is_some(),
                "decode-failure must populate decode_error"
            );
        }
    }

    #[test]
    fn helper_singlesig_watch_only_short_circuits_ms1() {
        let mut expected = synth_full_bundle();
        // Convert to watch-only by emptying ms1[0].
        expected.ms1[0].clear();
        let supplied_ms1: Vec<String> = vec!["".into()];
        let supplied_mk1 = match &expected.mk1 {
            MkField::Single(v) => v.clone(),
            MkField::Multi(_) => panic!("expected single-sig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let mut _test_so: Vec<u8> = Vec::new();
        let mut _test_se: Vec<u8> = Vec::new();
        let checks = emit_verify_checks(
            &expected,
            &supplied,
            false,
            true,
            false,
            &mut _test_so,
            &mut _test_se,
        )
        .unwrap();
        assert_eq!(checks.len(), 9);
        // ms1_decode and ms1_entropy_match are skipped per SPEC §5.7.
        let ms1_decode = &checks[0];
        let ms1_match = &checks[1];
        assert!(ms1_decode.passed);
        assert!(ms1_match.passed);
        assert_eq!(
            ms1_decode.decode_error.as_deref(),
            Some("skipped: watch-only slot")
        );
        assert_eq!(
            ms1_match.decode_error.as_deref(),
            Some("skipped: watch-only slot")
        );
        // mk1 + md1 substantive checks all pass.
        for c in &checks[2..] {
            assert!(c.passed, "{} should pass on watch-only happy path", c.name);
        }
    }

    #[test]
    fn helper_multisig_watch_only_emits_3plus6n_checks_in_spec_order() {
        use crate::parse::{CosignerSpec, MultisigPathFamily};
        use crate::synthesize::synthesize_multisig_watch_only;
        use bitcoin::bip32::DerivationPath;
        // Derive 2 distinct cosigner xpubs at the canonical BIP-48 depth-4 path
        // from 2 distinct mnemonic seeds. Distinct xpubs → distinct chunk_set_ids
        // (avoids the legacy self-multisig csi-collision case which is out of
        // scope for SPEC §5.7).
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        let m_a = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let seed_a = m_a.to_seed("");
        let master_a = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed_a).unwrap();
        let xpriv_a = master_a.derive_priv(&secp, &path).unwrap();
        let xpub_a = Xpub::from_priv(&secp, &xpriv_a);
        let fp_a = master_a.fingerprint(&secp);
        let m_b = Mnemonic::parse_in(
            bip39::Language::English,
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
        )
        .unwrap();
        let seed_b = m_b.to_seed("");
        let master_b = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed_b).unwrap();
        let xpriv_b = master_b.derive_priv(&secp, &path).unwrap();
        let xpub_b = Xpub::from_priv(&secp, &xpriv_b);
        let fp_b = master_b.fingerprint(&secp);
        let cosigners = vec![
            CosignerSpec {
                xpub: xpub_a,
                master_fingerprint: fp_a,
                path: Some(path.clone()),
            },
            CosignerSpec {
                xpub: xpub_b,
                master_fingerprint: fp_b,
                path: Some(path.clone()),
            },
        ];
        let n: usize = 2;
        let expected = synthesize_multisig_watch_only(
            &cosigners,
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            0,
            MultisigPathFamily::default(),
            false,
        )
        .unwrap();
        let supplied_ms1 = expected.ms1.clone();
        let supplied_mk1: Vec<String> = match &expected.mk1 {
            MkField::Multi(per_cos) => per_cos.iter().flat_map(|v| v.iter().cloned()).collect(),
            MkField::Single(_) => panic!("expected multisig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let mut _test_so: Vec<u8> = Vec::new();
        let mut _test_se: Vec<u8> = Vec::new();
        let checks = emit_verify_checks(
            &expected,
            &supplied,
            true,
            true,
            false,
            &mut _test_so,
            &mut _test_se,
        )
        .unwrap();
        assert_eq!(
            checks.len(),
            6 * n + 3,
            "multisig must emit 3+6N checks per SPEC §5.7 (N={n})"
        );
        let names: Vec<&str> = checks.iter().map(|c| c.name.as_str()).collect();
        // First 6N: per-cosigner [i]-indexed.
        let mut expected_names: Vec<String> = Vec::new();
        for i in 0..n {
            expected_names.push(format!("ms1_decode[{i}]"));
            expected_names.push(format!("ms1_entropy_match[{i}]"));
            expected_names.push(format!("mk1_decode[{i}]"));
            expected_names.push(format!("mk1_xpub_match[{i}]"));
            expected_names.push(format!("mk1_fingerprint_match[{i}]"));
            expected_names.push(format!("mk1_path_match[{i}]"));
        }
        // Last 3: shared md1.
        expected_names.push("md1_decode".into());
        expected_names.push("md1_wallet_policy".into());
        expected_names.push("md1_xpub_match".into());
        let expected_names_ref: Vec<&str> = expected_names.iter().map(String::as_str).collect();
        assert_eq!(names, expected_names_ref, "SPEC §5.7 ordering");
        // The fixture uses two distinct mnemonic seeds → two distinct cosigner
        // xpubs → two distinct chunk_set_ids; mk_codec grouping works correctly.
        // Per-cell forensic content on the chunked multi-card path is fully
        // exercised by cli_bundle_multisig.rs / cli_verify_bundle_*.rs end-to-end.
        // This unit test asserts the helper's structural contract (3+6N name
        // vec + ms1_decode happy-path) only.
        let ms1_decode_passed = checks
            .iter()
            .filter(|c| c.name.starts_with("ms1_decode"))
            .all(|c| c.passed);
        assert!(
            ms1_decode_passed,
            "ms1_decode[i] must pass on byte-identical happy path"
        );
    }

    #[test]
    fn helper_multisig_full_emits_3plus6n_checks_in_spec_order() {
        // B.1: full-mode multisig fixture. Reuses watch-only synthesis for the
        // mk1+md1 (distinct cosigners → distinct chunk_set_ids → grouping works)
        // then manually populates expected.ms1 with two distinct non-empty ms1
        // strings derived from synthesize_full(seed_a/seed_b). The unit-test
        // scope is emit_multisig_checks behavior in isolation, not synthesis.
        use crate::parse::{CosignerSpec, MultisigPathFamily};
        use crate::synthesize::{synthesize_full, synthesize_multisig_watch_only};
        use bitcoin::bip32::DerivationPath;
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        let m_a = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let entropy_a = m_a.to_entropy();
        let seed_a = m_a.to_seed("");
        let master_a = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed_a).unwrap();
        let xpriv_a = master_a.derive_priv(&secp, &path).unwrap();
        let xpub_a = Xpub::from_priv(&secp, &xpriv_a);
        let fp_a = master_a.fingerprint(&secp);
        let m_b = Mnemonic::parse_in(
            bip39::Language::English,
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
        )
        .unwrap();
        let entropy_b = m_b.to_entropy();
        let seed_b = m_b.to_seed("");
        let master_b = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed_b).unwrap();
        let xpriv_b = master_b.derive_priv(&secp, &path).unwrap();
        let xpub_b = Xpub::from_priv(&secp, &xpriv_b);
        let fp_b = master_b.fingerprint(&secp);
        let cosigners = vec![
            CosignerSpec {
                xpub: xpub_a,
                master_fingerprint: fp_a,
                path: Some(path.clone()),
            },
            CosignerSpec {
                xpub: xpub_b,
                master_fingerprint: fp_b,
                path: Some(path.clone()),
            },
        ];
        let n: usize = 2;
        let mut expected = synthesize_multisig_watch_only(
            &cosigners,
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            0,
            MultisigPathFamily::default(),
            false,
        )
        .unwrap();
        // Manually populate per-cosigner ms1 with non-empty strings (full-mode shape).
        let bundle_a = synthesize_full(
            &entropy_a,
            fp_a,
            xpub_a,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        let bundle_b = synthesize_full(
            &entropy_b,
            fp_b,
            xpub_b,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        expected.ms1 = vec![bundle_a.ms1[0].clone(), bundle_b.ms1[0].clone()];
        let supplied_ms1 = expected.ms1.clone();
        let supplied_mk1: Vec<String> = match &expected.mk1 {
            MkField::Multi(per_cos) => per_cos.iter().flat_map(|v| v.iter().cloned()).collect(),
            MkField::Single(_) => panic!("expected multisig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let mut _test_so: Vec<u8> = Vec::new();
        let mut _test_se: Vec<u8> = Vec::new();
        let checks = emit_verify_checks(
            &expected,
            &supplied,
            true,
            true,
            false,
            &mut _test_so,
            &mut _test_se,
        )
        .unwrap();
        assert_eq!(
            checks.len(),
            6 * n + 3,
            "multisig must emit 3+6N checks (N={n})"
        );
        // Substantive ms1 happy-path: case 2 (decodes Ok + byte-equal) for both slots.
        for i in 0..n {
            let dec = checks
                .iter()
                .find(|c| c.name == format!("ms1_decode[{i}]"))
                .unwrap();
            assert!(dec.passed, "case 2 ms1_decode[{i}] must pass");
            let mat = checks
                .iter()
                .find(|c| c.name == format!("ms1_entropy_match[{i}]"))
                .unwrap();
            assert!(
                mat.passed,
                "case 2 ms1_entropy_match[{i}] must pass on byte-identical"
            );
        }
    }

    #[test]
    fn helper_multisig_missing_ms1_emits_passed_false_per_spec_5_7_case_4() {
        // B.5: SPEC §5.7 case 4 — full-mode bundle with no supplied ms1 → passed=false.
        use crate::parse::{CosignerSpec, MultisigPathFamily};
        use crate::synthesize::{synthesize_full, synthesize_multisig_watch_only};
        use bitcoin::bip32::DerivationPath;
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        let m_a = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let entropy_a = m_a.to_entropy();
        let seed_a = m_a.to_seed("");
        let master_a = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed_a).unwrap();
        let xpriv_a = master_a.derive_priv(&secp, &path).unwrap();
        let xpub_a = Xpub::from_priv(&secp, &xpriv_a);
        let fp_a = master_a.fingerprint(&secp);
        let m_b = Mnemonic::parse_in(
            bip39::Language::English,
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
        )
        .unwrap();
        let entropy_b = m_b.to_entropy();
        let seed_b = m_b.to_seed("");
        let master_b = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed_b).unwrap();
        let xpriv_b = master_b.derive_priv(&secp, &path).unwrap();
        let xpub_b = Xpub::from_priv(&secp, &xpriv_b);
        let fp_b = master_b.fingerprint(&secp);
        let cosigners = vec![
            CosignerSpec {
                xpub: xpub_a,
                master_fingerprint: fp_a,
                path: Some(path.clone()),
            },
            CosignerSpec {
                xpub: xpub_b,
                master_fingerprint: fp_b,
                path: Some(path.clone()),
            },
        ];
        let mut expected = synthesize_multisig_watch_only(
            &cosigners,
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            0,
            MultisigPathFamily::default(),
            false,
        )
        .unwrap();
        let bundle_a = synthesize_full(
            &entropy_a,
            fp_a,
            xpub_a,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        let bundle_b = synthesize_full(
            &entropy_b,
            fp_b,
            xpub_b,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        expected.ms1 = vec![bundle_a.ms1[0].clone(), bundle_b.ms1[0].clone()];
        // Supply EMPTY ms1 to trigger case 4.
        let supplied_ms1: Vec<String> = vec![];
        let supplied_mk1: Vec<String> = match &expected.mk1 {
            MkField::Multi(per_cos) => per_cos.iter().flat_map(|v| v.iter().cloned()).collect(),
            MkField::Single(_) => panic!("expected multisig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let mut _test_so: Vec<u8> = Vec::new();
        let mut _test_se: Vec<u8> = Vec::new();
        let checks = emit_verify_checks(
            &expected,
            &supplied,
            true,
            true,
            false,
            &mut _test_so,
            &mut _test_se,
        )
        .unwrap();
        for i in 0..2 {
            let dec = checks
                .iter()
                .find(|c| c.name == format!("ms1_decode[{i}]"))
                .unwrap();
            assert!(
                !dec.passed,
                "case 4 ms1_decode[{i}] must fail (passed=false)"
            );
            assert_eq!(
                dec.decode_error.as_deref().unwrap(),
                &format!("error: ms1[{i}] expected (full-mode bundle) but not supplied")
            );
            let mat = checks
                .iter()
                .find(|c| c.name == format!("ms1_entropy_match[{i}]"))
                .unwrap();
            assert!(!mat.passed, "case 4 ms1_entropy_match[{i}] must fail");
        }
    }

    // ───────────────────────── H1 (cycle-1) ─────────────────────────
    //
    // H1 — `md1_xpub_match` must FAIL for a supplied md1 that reconstructs a
    // DIFFERENT wallet than the engraved bundle, even when the cosigner
    // pubkey-SET is identical (so the legacy sorted-multiset compare wrongly
    // GREEN-lit it). The widened gate ALSO requires a structural decoded-policy
    // compare (`tree == && use_site_path ==` + per-`@N`
    // `tlv.use_site_path_overrides`), keeping the subordinate pubkey-set check.
    // Origins (`path_decl` + origin/fingerprint TLV) are EXCLUDED (legitimate
    // elision/canonicalization variance — bug-hunt L14). See
    // design/IMPLEMENTATION_PLAN_cycle1_critical_fixes.md §6.3 / R0 round-2.

    use crate::parse::{CosignerSpec, MultisigPathFamily};
    use crate::synthesize::synthesize_multisig_watch_only;
    use bitcoin::bip32::DerivationPath;

    /// Build N distinct cosigner specs at the canonical BIP-48 depth-4 path
    /// from N distinct mnemonics. The 65-byte md1 pubkeys derive ONLY from the
    /// cosigner xpubs (chain_code||pubkey), so they are template-/threshold-
    /// independent — a divergent-policy bundle synthesized from the SAME
    /// cosigners carries the SAME pubkey multiset (the precise condition under
    /// which the legacy multiset-only gate false-GREENed).
    fn h1_cosigners(phrases: &[&str]) -> Vec<CosignerSpec> {
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        phrases
            .iter()
            .map(|p| {
                let m = Mnemonic::parse_in(bip39::Language::English, *p).unwrap();
                let seed = m.to_seed("");
                let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
                let xpriv = master.derive_priv(&secp, &path).unwrap();
                let xpub = Xpub::from_priv(&secp, &xpriv);
                let fp = master.fingerprint(&secp);
                CosignerSpec {
                    xpub,
                    master_fingerprint: fp,
                    path: Some(path.clone()),
                }
            })
            .collect()
    }

    const H1_P1: &str = TREZOR_24;
    const H1_P2: &str =
        "legal winner thank year wave sausage worth useful legal winner thank yellow";
    const H1_P3: &str =
        "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

    /// Synthesize a watch-only multisig bundle from the SAME 3 cosigners with a
    /// chosen `(template, threshold)`. The cosigner pubkey multiset is identical
    /// across all `(template, threshold)` choices.
    fn h1_bundle(template: CliTemplate, threshold: u8) -> Bundle {
        let cosigners = h1_cosigners(&[H1_P1, H1_P2, H1_P3]);
        synthesize_multisig_watch_only(
            &cosigners,
            CliNetwork::Mainnet,
            template,
            threshold,
            0,
            MultisigPathFamily::default(),
            false,
        )
        .unwrap()
    }

    /// Run `emit_verify_checks` with `expected`'s ms1/mk1 but a SUPPLIED md1
    /// drawn from a DIFFERENT policy (same cosigners). Returns the
    /// `md1_xpub_match` check.
    fn h1_run(expected: &Bundle, supplied_md1: &[String]) -> VerifyCheck {
        let supplied_ms1 = expected.ms1.clone();
        let supplied_mk1: Vec<String> = match &expected.mk1 {
            MkField::Multi(per_cos) => per_cos.iter().flat_map(|v| v.iter().cloned()).collect(),
            MkField::Single(_) => panic!("expected multisig"),
        };
        let supplied_md1_v = supplied_md1.to_vec();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1_v,
        };
        let mut so: Vec<u8> = Vec::new();
        let mut se: Vec<u8> = Vec::new();
        let checks =
            emit_verify_checks(expected, &supplied, true, true, false, &mut so, &mut se).unwrap();
        checks
            .into_iter()
            .find(|c| c.name == "md1_xpub_match")
            .expect("md1_xpub_match present")
    }

    /// Re-encode `expected`'s md1 with a mutated `use_site_path` baseline (same
    /// `.tree`, same pubkeys) — for the C-PLAN-1 multipath-divergence case.
    fn h1_remap_use_site_path(
        md1: &[String],
        new_use_site: md_codec::use_site_path::UseSitePath,
    ) -> Vec<String> {
        let refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
        let mut desc = md_codec::chunk::reassemble(&refs).expect("expected md1 decodes");
        desc.use_site_path = new_use_site;
        md_codec::chunk::split(&desc).expect("re-encode mutated descriptor")
    }

    /// H1 case 1 — wrong threshold: engraved `wsh(sortedmulti(2,A,B,C))`,
    /// supplied `wsh(sortedmulti(1,A,B,C))` (1-of-3 anyone-spends). Same pubkey
    /// SET ⇒ legacy multiset gate passed; widened gate MUST fail.
    #[test]
    fn h1_wrong_threshold_fails() {
        let expected = h1_bundle(CliTemplate::WshSortedMulti, 2);
        let supplied = h1_bundle(CliTemplate::WshSortedMulti, 1);
        let c = h1_run(&expected, &supplied.md1);
        assert!(
            !c.passed,
            "wrong-threshold (1-of-3 vs 2-of-3) md1 must FAIL md1_xpub_match"
        );
    }

    /// H1 case 2 — sorted-vs-unsorted: engraved `wsh(sortedmulti(2,…))`,
    /// supplied unsorted `wsh(multi(2,…))` (different Tag → consensus-different).
    #[test]
    fn h1_sorted_vs_unsorted_fails() {
        let expected = h1_bundle(CliTemplate::WshSortedMulti, 2);
        let supplied = h1_bundle(CliTemplate::WshMulti, 2);
        let c = h1_run(&expected, &supplied.md1);
        assert!(
            !c.passed,
            "sorted-vs-unsorted (SortedMulti vs Multi) md1 must FAIL md1_xpub_match"
        );
    }

    /// H1 case 3 — script-type wrapper: engraved `wsh(sortedmulti(2,…))`,
    /// supplied `sh(wsh(sortedmulti(2,…)))` (P2SH-nested — different address
    /// type). The `sh(multi)`/`sh(wsh(multi))` trap: `tree ==` distinguishes
    /// the nested body that a root-tag-only check would miss.
    #[test]
    fn h1_script_type_wrapper_fails() {
        let expected = h1_bundle(CliTemplate::WshSortedMulti, 2);
        let supplied = h1_bundle(CliTemplate::ShWshSortedMulti, 2);
        let c = h1_run(&expected, &supplied.md1);
        assert!(
            !c.passed,
            "wsh vs sh(wsh(...)) md1 must FAIL md1_xpub_match (script-type/nesting)"
        );
    }

    /// H1 case 4 — C-PLAN-1: identical `.tree`, DIVERGENT `use_site_path`
    /// multipath (`<0;1>` vs `<2;3>` change-chains → DIFFERENT watched-address
    /// set). This is the gap a `.tree`-only gate would have GREEN-lit.
    #[test]
    fn h1_multipath_divergence_fails() {
        use md_codec::use_site_path::{Alternative, UseSitePath};
        let expected = h1_bundle(CliTemplate::WshSortedMulti, 2);
        // <2;3>/* — same .tree, same pubkeys, different change-chains.
        let divergent = UseSitePath {
            multipath: Some(vec![
                Alternative {
                    hardened: false,
                    value: 2,
                },
                Alternative {
                    hardened: false,
                    value: 3,
                },
            ]),
            wildcard_hardened: false,
        };
        let supplied_md1 = h1_remap_use_site_path(&expected.md1, divergent);
        let c = h1_run(&expected, &supplied_md1);
        assert!(
            !c.passed,
            "use_site_path-divergent (<0;1> vs <2;3>) md1 must FAIL md1_xpub_match — \
             a .tree-only gate would have false-GREENed this different-address wallet"
        );
    }

    /// H1 case 4b — C-PLAN-1 variant: multipath PRESENCE/COUNT divergence
    /// (`<0;1>/*` vs bare `/*`, no multipath). Same `.tree`, same pubkeys.
    #[test]
    fn h1_multipath_presence_divergence_fails() {
        use md_codec::use_site_path::UseSitePath;
        let expected = h1_bundle(CliTemplate::WshSortedMulti, 2);
        let bare = UseSitePath {
            multipath: None,
            wildcard_hardened: false,
        };
        let supplied_md1 = h1_remap_use_site_path(&expected.md1, bare);
        let c = h1_run(&expected, &supplied_md1);
        assert!(
            !c.passed,
            "multipath presence/count divergence (<0;1>/* vs bare /*) must FAIL md1_xpub_match"
        );
    }

    /// H1 clean-negative 1 — genuine match: byte-identical engraved md1 must
    /// still PASS (no over-rejection). Identical `.tree` AND `use_site_path`.
    #[test]
    fn h1_genuine_match_passes() {
        let expected = h1_bundle(CliTemplate::WshSortedMulti, 2);
        let supplied_md1 = expected.md1.clone();
        let c = h1_run(&expected, &supplied_md1);
        assert!(
            c.passed,
            "genuine matching md1 must PASS md1_xpub_match (no over-rejection)"
        );
    }

    /// H1 clean-negative 2 — origin-DIVERGENT-but-policy-equal: the SAME
    /// `.tree` / `use_site_path` / pubkey SET with a DIFFERENT per-cosigner
    /// origin path must still PASS. Origins are EXCLUDED from the gate (L14
    /// elision/canonicalization brittleness), so changing only the origin does
    /// NOT false-fail. (Binding origins would re-introduce the v0.5.0-era
    /// origin-elision false-FAIL the B.3 multiset change exists to avoid.)
    /// Re-encoded via decode → mutate `path_decl` origin → re-split; `.tree`,
    /// `use_site_path`, and pubkeys are UNCHANGED — only the origin differs.
    #[test]
    fn h1_origin_divergent_but_policy_equal_passes() {
        use md_codec::origin_path::{OriginPath, PathComponent, PathDeclPaths};
        let expected = h1_bundle(CliTemplate::WshSortedMulti, 2);
        let refs: Vec<&str> = expected.md1.iter().map(|s| s.as_str()).collect();
        let mut desc = md_codec::chunk::reassemble(&refs).expect("expected md1 decodes");
        // A DIFFERENT-but-valid origin: m/48'/0'/5'/2' (account index 0' → 5').
        let alt_origin = OriginPath {
            components: vec![
                PathComponent {
                    hardened: true,
                    value: 48,
                },
                PathComponent {
                    hardened: true,
                    value: 0,
                },
                PathComponent {
                    hardened: true,
                    value: 5,
                },
                PathComponent {
                    hardened: true,
                    value: 2,
                },
            ],
        };
        desc.path_decl.paths = match desc.path_decl.paths {
            PathDeclPaths::Shared(_) => PathDeclPaths::Shared(alt_origin),
            PathDeclPaths::Divergent(v) => {
                PathDeclPaths::Divergent(v.iter().map(|_| alt_origin.clone()).collect())
            }
        };
        // Also clear origin-bearing TLV columns (fingerprints/overrides) — all
        // origin-category fields the gate must IGNORE. `.tree`/`use_site_path`/
        // pubkeys remain identical to `expected`.
        desc.tlv.fingerprints = None;
        desc.tlv.origin_path_overrides = None;
        let supplied_md1 =
            md_codec::chunk::split(&desc).expect("re-encode origin-mutated descriptor");
        let c = h1_run(&expected, &supplied_md1);
        assert!(
            c.passed,
            "origin-divergent-but-policy-equal md1 must PASS md1_xpub_match (origins EXCLUDED)"
        );
    }

    // ========================================================================
    // Cycle F (`ms1-repair-demote-to-candidate`) — C1 ground-truth compare
    // (SPEC §0.4/§3/§5.4/§5.5/§8.6). Unit-level coverage of
    // `emit_verify_checks`'s ms1 decode-failure branch, complementing the
    // CLI-level cells in `tests/cli_ms1_repair_demote.rs` and the flipped
    // `tests/cli_auto_repair.rs::cell_27`/`cell_30`.
    // ========================================================================

    /// Deterministically flip the bech32 char at data-part index `pos` (mirrors
    /// the `flip_at` helper duplicated across the integration test suite).
    fn cycle_f_flip_at(chunk: &str, pos: usize) -> String {
        const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
        let sep = chunk.rfind('1').unwrap();
        let (prefix, rest) = chunk.split_at(sep + 1);
        let mut chars: Vec<char> = rest.chars().collect();
        let was = chars[pos];
        let was_idx = ALPHABET.find(was).unwrap();
        chars[pos] = ALPHABET.chars().nth((was_idx + 1) % 32).unwrap();
        let mut out = String::from(prefix);
        for c in chars {
            out.push(c);
        }
        out
    }

    /// §5.4 MATCH — a decode-failing ms1 whose auto-repair correction
    /// byte-matches `expected.ms1[0]` (the SAME wallet's own card) → both
    /// `ms1_decode` and `ms1_entropy_match` PASS, noted as recovered +
    /// confirmed against the expected seed; no forensic fields populated.
    #[test]
    fn cycle_f_ms1_ground_truth_match_recovers_and_confirms() {
        let expected = synth_full_bundle();
        let clean_ms1 = expected.ms1[0].clone();
        let supplied_ms1: Vec<String> = vec![cycle_f_flip_at(&clean_ms1, 17)];
        let supplied_mk1 = match &expected.mk1 {
            MkField::Single(v) => v.clone(),
            MkField::Multi(_) => panic!("expected single-sig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let mut _test_so: Vec<u8> = Vec::new();
        let mut _test_se: Vec<u8> = Vec::new();
        let checks = emit_verify_checks(
            &expected,
            &supplied,
            false,
            false,
            false,
            &mut _test_so,
            &mut _test_se,
        )
        .unwrap();
        let ms1_decode = checks.iter().find(|c| c.name == "ms1_decode").unwrap();
        let ms1_match = checks
            .iter()
            .find(|c| c.name == "ms1_entropy_match")
            .unwrap();
        assert!(ms1_decode.passed, "{ms1_decode:?}");
        assert!(ms1_match.passed, "{ms1_match:?}");
        assert!(ms1_decode
            .detail
            .contains("recovered via auto-repair, confirmed against expected seed"));
        assert!(ms1_match
            .detail
            .contains("recovered via auto-repair, confirmed against expected seed"));
        assert!(ms1_match.expected.is_none());
        assert!(ms1_match.actual.is_none());
        assert!(ms1_match.diff_byte_offset.is_none());
    }

    /// §5.5 (FUNDS ANCHOR) MISMATCH — a decode-failing ms1 whose auto-repair
    /// correction resolves to a DIFFERENT wallet's clean card (NOT
    /// `expected.ms1[0]`) → `ms1_decode` reports "recovered (unconfirmed)"
    /// but `ms1_entropy_match` FAILS with the redacted detail — never
    /// blessed merely because the candidate happens to decode.
    #[test]
    fn cycle_f_ms1_ground_truth_mismatch_fails_and_redacts() {
        let expected = synth_full_bundle(); // wallet E (TREZOR_24)
        let other = synth_full_bundle_for(
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
        ); // wallet A — a DIFFERENT seed
        let other_ms1 = other.ms1[0].clone();
        let supplied_ms1: Vec<String> = vec![cycle_f_flip_at(&other_ms1, 17)];
        // mk1/md1 are irrelevant to this assertion (ms1 check is independent
        // of the mk1/md1 checks) — reuse `expected`'s own so decode succeeds
        // and doesn't distract from the ms1 assertions.
        let supplied_mk1 = match &expected.mk1 {
            MkField::Single(v) => v.clone(),
            MkField::Multi(_) => panic!("expected single-sig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let mut _test_so: Vec<u8> = Vec::new();
        let mut _test_se: Vec<u8> = Vec::new();
        let checks = emit_verify_checks(
            &expected,
            &supplied,
            false,
            false,
            false,
            &mut _test_so,
            &mut _test_se,
        )
        .unwrap();
        let ms1_match = checks
            .iter()
            .find(|c| c.name == "ms1_entropy_match")
            .unwrap();
        assert!(
            !ms1_match.passed,
            "a candidate that does not match the typed seed must NOT be blessed"
        );
        assert!(ms1_match
            .detail
            .contains("this card is not a card for this seed"));
        // §8.6/G5 secret-hygiene redaction: no seed bytes, no offset.
        assert!(ms1_match.expected.is_none(), "{ms1_match:?}");
        assert!(ms1_match.actual.is_none(), "{ms1_match:?}");
        assert!(ms1_match.diff_byte_offset.is_none(), "{ms1_match:?}");
        for c in &checks {
            assert!(
                !c.detail.contains(&other_ms1),
                "leaked corrected seed: {c:?}"
            );
            assert!(
                !c.detail.contains(&expected.ms1[0]),
                "leaked expected seed: {c:?}"
            );
            if let Some(e) = &c.expected {
                assert!(!e.contains(&other_ms1) && !e.contains(&expected.ms1[0]));
            }
            if let Some(a) = &c.actual {
                assert!(!a.contains(&other_ms1) && !a.contains(&expected.ms1[0]));
            }
        }
        // Overall run-level property: a mismatch is a failed CHECK ROW, not
        // an abort — the full check array still emits (G3).
        assert_eq!(
            checks.len(),
            9,
            "mismatch must not truncate the check table"
        );
    }

    /// §5.8 — `no_auto_repair: true` suppresses the ground-truth compare
    /// entirely: a decode-failing ms1 (even one that WOULD recover to the
    /// expected seed) falls back to the legacy decode-error check rows.
    #[test]
    fn cycle_f_no_auto_repair_suppresses_ground_truth_compare() {
        let expected = synth_full_bundle();
        let clean_ms1 = expected.ms1[0].clone();
        let supplied_ms1: Vec<String> = vec![cycle_f_flip_at(&clean_ms1, 17)];
        let supplied_mk1 = match &expected.mk1 {
            MkField::Single(v) => v.clone(),
            MkField::Multi(_) => panic!("expected single-sig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let mut _test_so: Vec<u8> = Vec::new();
        let mut _test_se: Vec<u8> = Vec::new();
        // no_auto_repair = true.
        let checks = emit_verify_checks(
            &expected,
            &supplied,
            false,
            true,
            false,
            &mut _test_so,
            &mut _test_se,
        )
        .unwrap();
        let ms1_decode = checks.iter().find(|c| c.name == "ms1_decode").unwrap();
        let ms1_match = checks
            .iter()
            .find(|c| c.name == "ms1_entropy_match")
            .unwrap();
        assert!(!ms1_decode.passed, "{ms1_decode:?}");
        assert!(
            ms1_decode.decode_error.is_some(),
            "legacy decode-error path must populate decode_error when auto-repair is disabled"
        );
        assert!(!ms1_decode.detail.contains("recovered"), "{ms1_decode:?}");
        assert!(ms1_match.passed, "skipped-vacuously: {ms1_match:?}");
    }
}
