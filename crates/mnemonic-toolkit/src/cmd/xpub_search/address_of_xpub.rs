//! `mnemonic xpub-search address-of-xpub` — P3 mode.
//!
//! SPEC: plan §5 (P3 address-of-xpub).
//! - Input: xpub (any SLIP-0132 single-sig prefix) OR mk1 card carrying an
//!   xpub. Multisig SLIP-0132 prefixes (`Ypub`/`Zpub`/`Upub`/`Vpub`) are
//!   refused — use account-of-descriptor for multisig.
//! - Target addresses: repeatable `--target-address <ADDR>` (≥1 required).
//! - Scan: chain ∈ {0, 1} (or {0} with `--external-only`) × index ∈
//!   [0, gap_limit). First-match-per-target wins.
//! - Address-type inference: SLIP-0132 prefix → script_type; neutral xpub/tpub
//!   requires explicit `--address-type`. `--address-type` overrides.
//! - JSON envelope: `{"schema_version":"1","mode":"address-of-xpub", ...}`.
//! - Exit: 0 = all matched; 4 = any unmatched; 1 = bad input.
//!
//! P3 has NO seed material; auto-fire BCH repair does NOT apply.

use super::address_search::{scan_xpub_for_addresses, AddressMatch, AddressMatchKind};
use super::target_intake::resolve_target_xpub;
use super::{XpubSearchEnvelope, XpubSearchJson};
use crate::cmd::convert::{parse_script_type_arg, ScriptType};
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use bitcoin::secp256k1::Secp256k1;
use clap::Args;
use serde::Serialize;
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct AddressOfXpubArgs {
    /// Parent xpub. Accepts any SLIP-0132 single-sig prefix (xpub / tpub /
    /// ypub / upub / zpub / vpub) or an mk1 bech32 card carrying an xpub.
    /// Multisig SLIP-0132 prefixes (Ypub / Zpub / Upub / Vpub) are refused —
    /// use `xpub-search account-of-descriptor` for multisig.
    #[arg(long, value_name = "XPUB-OR-MK1", conflicts_with = "xpub_stdin")]
    pub xpub: Option<String>,

    /// Read parent xpub from stdin (single line, trailing newline stripped).
    #[arg(long, conflicts_with = "xpub")]
    pub xpub_stdin: bool,

    /// Target address. Repeatable; at least one required.
    #[arg(long, value_name = "ADDR", required = true)]
    pub target_address: Vec<String>,

    /// Per-chain BIP-44 gap-limit window. Scan covers `0..gap_limit` indices
    /// on each chain. Default 20.
    #[arg(long, default_value_t = 20)]
    pub gap_limit: u32,

    /// Restrict the scan to the external (receive) chain only; skip internal
    /// (change) chain. Default: scan both chains.
    #[arg(long)]
    pub external_only: bool,

    /// Explicit script-type for address rendering. Required when the parent
    /// xpub uses a neutral SLIP-0132 prefix (xpub / tpub) or when overriding
    /// the prefix-inferred type. Accepts `p2pkh` / `p2sh-p2wpkh` / `p2wpkh` /
    /// `p2tr`.
    #[arg(long, value_name = "TYPE", value_parser = parse_script_type_arg)]
    pub address_type: Option<ScriptType>,

    /// Network selector. Defaults to network inferred from the xpub version
    /// byte (mainnet ↔ xpub/ypub/Ypub/zpub/Zpub; testnet ↔ tpub/upub/Upub/
    /// vpub/Vpub). `--network signet` or `--network regtest` overrides for
    /// disambiguation (the version byte collapses test/signet/regtest).
    #[arg(long)]
    pub network: Option<CliNetwork>,

    /// Emit a JSON envelope on stdout instead of text-form report.
    #[arg(long)]
    pub json: bool,
}

/// Per-target result entry in the JSON envelope.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum AddressResultJson {
    Match {
        target: String,
        result: &'static str, // "match"
        chain: &'static str,
        index: u32,
        script_type: &'static str,
    },
    NoMatch {
        target: String,
        result: &'static str, // "no_match"
        scanned_external: u32,
        scanned_internal: u32,
    },
}

/// Per-mode JSON body for address-of-xpub.
#[derive(Debug, Serialize)]
pub struct AddressOfXpubResult {
    pub results: Vec<AddressResultJson>,
    pub xpub_canonical: String,
    /// SLIP-0132 prefix the input was normalized from, or `null` for already-
    /// canonical xpub/tpub or mk1-card input.
    pub xpub_variant: Option<&'static str>,
    pub gap_limit: u32,
}

/// Map a ScriptType to its kebab-case JSON tag.
fn script_type_str(st: ScriptType) -> &'static str {
    match st {
        ScriptType::P2pkh => "p2pkh",
        ScriptType::P2wpkh => "p2wpkh",
        ScriptType::P2shP2wpkh => "p2sh-p2wpkh",
        ScriptType::P2tr => "p2tr",
    }
}

/// Infer script-type from a SLIP-0132 variant signal, or return None when
/// the variant carries no signal (neutral xpub/tpub) and the user must
/// supply --address-type explicitly. Multisig variants are refused upstream
/// (see `refuse_multisig_prefix`).
fn script_type_from_variant(variant: Option<&'static str>) -> Option<ScriptType> {
    match variant {
        Some("ypub") | Some("upub") => Some(ScriptType::P2shP2wpkh),
        Some("zpub") | Some("vpub") => Some(ScriptType::P2wpkh),
        // Neutral xpub/tpub (None) and unknown variants → no signal.
        _ => None,
    }
}

/// Refuse multisig SLIP-0132 prefixes with a pointer to account-of-descriptor.
fn refuse_multisig_prefix(variant: &str) -> ToolkitError {
    ToolkitError::BadInput(format!(
        "address-of-xpub is single-sig only; the {variant} prefix is a multisig SLIP-0132 \
         variant. Multisig address derivation requires the full descriptor — use \
         `xpub-search account-of-descriptor` to find the matching account."
    ))
}

/// Read the xpub value from stdin or from the `--xpub` arg.
fn read_xpub_value<R: Read>(args: &AddressOfXpubArgs, stdin: &mut R) -> Result<String, ToolkitError> {
    if args.xpub_stdin {
        let mut buf = String::new();
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
        let trimmed = buf.trim();
        if trimmed.is_empty() {
            return Err(ToolkitError::BadInput(
                "stdin empty; expected an xpub or mk1 card".into(),
            ));
        }
        Ok(trimmed.to_string())
    } else if let Some(v) = &args.xpub {
        Ok(v.clone())
    } else {
        Err(ToolkitError::BadInput(
            "supply --xpub <VALUE> or --xpub-stdin".into(),
        ))
    }
}

pub fn run_address_of_xpub<R: Read, W: Write, E: Write>(
    args: &AddressOfXpubArgs,
    stdin: &mut R,
    stdout: &mut W,
    _stderr: &mut E,
    _no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    // 1) Read xpub value.
    let xpub_value = read_xpub_value(args, stdin)?;

    // 2) Detect multisig SLIP-0132 prefix BEFORE resolve_target_xpub.
    //    `slip0132::normalize_xpub_prefix` ACCEPTS the multisig variants
    //    (Ypub/Zpub/Upub/Vpub) and silently maps them to neutral xpub/tpub —
    //    so without this short-circuit the input would normalize through and
    //    we'd derive single-sig addresses from a multisig-cosigner xpub
    //    (semantically wrong; the full descriptor is required for multisig
    //    address materialization). This short-circuit is the LOAD-BEARING
    //    refusal point; the pointer to `account-of-descriptor` here is the
    //    user's path forward.
    if !xpub_value.starts_with("mk1") {
        // The multisig prefixes start with a capital letter (uppercase ASCII).
        // bech32 mk1 cards start with "mk1" so this short-circuit is safe.
        if let Some(variant) = detect_multisig_prefix(&xpub_value) {
            return Err(refuse_multisig_prefix(variant));
        }
    }

    // 3) Resolve xpub via shared mk1-or-slip0132 dispatcher.
    let (xpub, variant) = resolve_target_xpub(&xpub_value)?;
    let xpub_canonical = xpub.to_string();

    // 4) Resolve script-type: explicit --address-type wins; else infer from
    //    variant; else refuse.
    let script_type = match args.address_type {
        Some(st) => st,
        None => match script_type_from_variant(variant) {
            Some(st) => st,
            None => {
                return Err(ToolkitError::BadInput(format!(
                    "xpub has no SLIP-0132 single-sig prefix signal{} — supply \
                     --address-type <p2pkh|p2sh-p2wpkh|p2wpkh|p2tr>.",
                    match variant {
                        Some(v) => format!(" ({v} is recognized but does not pin a script-type)"),
                        None => String::new(),
                    }
                )));
            }
        },
    };

    // 5) Network resolution: explicit --network wins; else inferred from the
    //    xpub version byte.
    let network = args.network.unwrap_or_else(|| network_from_xpub(&xpub));

    // 6) Scan.
    let secp = Secp256k1::verification_only();
    let scan_internal = !args.external_only;
    let matches = scan_xpub_for_addresses(
        &xpub,
        &args.target_address,
        args.gap_limit,
        scan_internal,
        script_type,
        network,
        &secp,
    );

    // 7) Build per-target JSON results.
    let mut all_matched = true;
    let results_json: Vec<AddressResultJson> = matches
        .iter()
        .map(|m| match &m.result {
            AddressMatchKind::Match { chain, index } => AddressResultJson::Match {
                target: m.target.clone(),
                result: "match",
                chain,
                index: *index,
                script_type: script_type_str(m.script_type),
            },
            AddressMatchKind::NoMatch {
                scanned_external,
                scanned_internal,
            } => {
                all_matched = false;
                AddressResultJson::NoMatch {
                    target: m.target.clone(),
                    result: "no_match",
                    scanned_external: *scanned_external,
                    scanned_internal: *scanned_internal,
                }
            }
        })
        .collect();

    // 8) Emit (text or JSON).
    if args.json {
        let envelope = XpubSearchEnvelope {
            schema_version: "1",
            body: XpubSearchJson::AddressOfXpub(AddressOfXpubResult {
                results: results_json,
                xpub_canonical,
                xpub_variant: variant,
                gap_limit: args.gap_limit,
            }),
        };
        let body = serde_json::to_string(&envelope).map_err(|e| {
            ToolkitError::BadInput(format!("address-of-xpub JSON serialize: {e}"))
        })?;
        writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
    } else {
        for m in &matches {
            emit_text_line(stdout, m, args.gap_limit, scan_internal)?;
        }
        let n_targets = matches.len();
        let n_matched = matches
            .iter()
            .filter(|m| matches!(m.result, AddressMatchKind::Match { .. }))
            .count();
        writeln!(
            stdout,
            "targets: {n_targets}; matched: {n_matched}; unmatched: {}",
            n_targets - n_matched
        )
        .map_err(ToolkitError::Io)?;
    }

    if all_matched {
        Ok(0)
    } else {
        // Aggregate count of candidate-comparisons performed for the no-match
        // diagnostic. Formula: matches.len() (= n_targets) × gap_limit × chains.
        // Per-target unique candidates are reported in AddressResultJson's
        // scanned_external / scanned_internal fields. See SPEC `searched` semantic
        // notes on ToolkitError::XpubSearchNoMatch.
        let total_scanned = matches
            .iter()
            .map(|_| (args.gap_limit * if scan_internal { 2 } else { 1 }) as usize)
            .sum::<usize>();
        Err(ToolkitError::XpubSearchNoMatch {
            mode: "address-of-xpub",
            searched: total_scanned,
        })
    }
}

fn emit_text_line<W: Write>(
    stdout: &mut W,
    m: &AddressMatch,
    gap_limit: u32,
    scan_internal: bool,
) -> Result<(), ToolkitError> {
    match &m.result {
        AddressMatchKind::Match { chain, index } => writeln!(
            stdout,
            "match: {} → {}/{}  (script_type={}, chain={}, index={})",
            m.target,
            if *chain == "external" { 0 } else { 1 },
            index,
            script_type_str(m.script_type),
            chain,
            index,
        )
        .map_err(ToolkitError::Io),
        AddressMatchKind::NoMatch { .. } => writeln!(
            stdout,
            "no match: {} (searched 0/0..{} {})",
            m.target,
            gap_limit.saturating_sub(1),
            if scan_internal {
                format!("+ 1/0..{}", gap_limit.saturating_sub(1))
            } else {
                "external-only".to_string()
            }
        )
        .map_err(ToolkitError::Io),
    }
}

/// Detect a multisig SLIP-0132 prefix by base58check decoding the version
/// bytes. Returns the variant name (`"Ypub"`, `"Zpub"`, `"Upub"`, `"Vpub"`)
/// or None for non-multisig / non-SLIP-0132 inputs.
fn detect_multisig_prefix(s: &str) -> Option<&'static str> {
    let raw = bitcoin::base58::decode_check(s).ok()?;
    if raw.len() != 78 {
        return None;
    }
    let prefix: [u8; 4] = raw[0..4].try_into().ok()?;
    match prefix {
        // SLIP-0132 mainnet multisig
        [0x02, 0x95, 0xB4, 0x3F] => Some("Ypub"),
        [0x02, 0xAA, 0x7E, 0xD3] => Some("Zpub"),
        // SLIP-0132 testnet multisig
        [0x02, 0x42, 0x89, 0xEF] => Some("Upub"),
        [0x02, 0x57, 0x54, 0x83] => Some("Vpub"),
        _ => None,
    }
}

/// Mirror of `cmd/convert.rs::network_from_xpub` (private there). Kept inline
/// so P3 doesn't depend on convert.rs internal visibility.
fn network_from_xpub(xpub: &bitcoin::bip32::Xpub) -> CliNetwork {
    match xpub.network {
        bitcoin::NetworkKind::Main => CliNetwork::Mainnet,
        bitcoin::NetworkKind::Test => CliNetwork::Testnet,
    }
}
