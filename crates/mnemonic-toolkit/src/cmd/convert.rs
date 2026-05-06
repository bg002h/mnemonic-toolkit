//! `mnemonic convert` subcommand — single-format conversion utility.
//!
//! Realizes `design/SPEC_convert_v0_6.md`.

use crate::derive_slot::{derive_bip32_at_path, derive_bip32_from_entropy};
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::slip0132::{
    apply_xpub_prefix, normalize_xpub_prefix, parse_xpub_prefix_arg, XpubPrefix,
};
use crate::template::CliTemplate;
use bip39::Mnemonic;
use bitcoin::bip32 as bip32;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::PrivateKey;
use clap::Args;
use serde::Serialize;
use std::io::{Read, Write};
use std::str::FromStr;

// ============================================================================
// SPEC §1 nodes
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Phrase,
    Entropy,
    Xpub,
    Xprv,
    Wif,
    Fingerprint,
    Path,
    Ms1,
    Mk1,
}

impl NodeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Phrase => "phrase",
            Self::Entropy => "entropy",
            Self::Xpub => "xpub",
            Self::Xprv => "xprv",
            Self::Wif => "wif",
            Self::Fingerprint => "fingerprint",
            Self::Path => "path",
            Self::Ms1 => "ms1",
            Self::Mk1 => "mk1",
        }
    }

    pub fn from_token(t: &str) -> Option<Self> {
        Some(match t {
            "phrase" => Self::Phrase,
            "entropy" => Self::Entropy,
            "xpub" => Self::Xpub,
            "xprv" => Self::Xprv,
            "wif" => Self::Wif,
            "fingerprint" => Self::Fingerprint,
            "path" => Self::Path,
            "ms1" => Self::Ms1,
            "mk1" => Self::Mk1,
            _ => return None,
        })
    }

    pub fn is_secret_bearing(self) -> bool {
        matches!(
            self,
            Self::Phrase | Self::Entropy | Self::Xprv | Self::Wif | Self::Ms1
        )
    }

    pub fn is_side_input_only(self) -> bool {
        matches!(self, Self::Path | Self::Fingerprint)
    }
}

// ============================================================================
// SPEC §5 grammar — `--from <node>=<value>`
// ============================================================================

#[derive(Debug, Clone)]
pub struct FromInput {
    pub node: NodeType,
    pub value: String,
}

pub fn parse_from_input(s: &str) -> Result<FromInput, String> {
    let eq = s
        .find('=')
        .ok_or_else(|| format!("--from must have shape <node>=<value>; got {:?}", s))?;
    let (token, after) = s.split_at(eq);
    let value = &after[1..];
    if token.is_empty() {
        return Err(format!("--from missing node name before '='; got {:?}", s));
    }
    let node = NodeType::from_token(token).ok_or_else(|| {
        format!(
            "unknown --from node {:?}; expected one of: phrase, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1",
            token
        )
    })?;
    if value.is_empty() {
        return Err(format!(
            "--from {} value is empty; supply a non-empty value (or '-' to read from stdin)",
            node.as_str()
        ));
    }
    Ok(FromInput {
        node,
        value: value.to_string(),
    })
}

// ============================================================================
// CLI args
// ============================================================================

#[derive(Args, Debug)]
pub struct ConvertArgs {
    #[arg(long = "from", action = clap::ArgAction::Append, value_parser = parse_from_input, required = true)]
    pub from: Vec<FromInput>,

    #[arg(long, action = clap::ArgAction::Append, required = true)]
    pub to: Vec<String>,

    #[arg(long)]
    pub network: Option<CliNetwork>,

    #[arg(long)]
    pub template: Option<CliTemplate>,

    #[arg(long)]
    pub path: Option<String>,

    #[arg(long)]
    pub language: Option<CliLanguage>,

    #[arg(long)]
    pub passphrase: Option<String>,

    #[arg(long, default_value = "0")]
    pub account: u32,

    #[arg(long)]
    pub fingerprint: Option<String>,

    /// SPEC v0.6.1 §11.a — emit `xpub` targets with a SLIP-0132 prefix.
    /// Requires explicit `--network` when non-default (`xpub`).
    #[arg(long = "xpub-prefix", value_parser = parse_xpub_prefix_arg)]
    pub xpub_prefix: Option<XpubPrefix>,

    #[arg(long)]
    pub json: bool,
}

// ============================================================================
// SPEC §6 JSON envelope
// ============================================================================

#[derive(Serialize)]
struct ConvertJson<'a> {
    schema_version: &'a str,
    from_node: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_value: Option<&'a str>,
    to: Vec<ConvertJsonEntry<'a>>,
}

#[derive(Serialize)]
struct ConvertJsonEntry<'a> {
    node: &'a str,
    value: String,
}

// ============================================================================
// SPEC §3 / §4 refusal taxonomy
// ============================================================================

fn refusal_one_way(from: NodeType, to: NodeType) -> ToolkitError {
    ToolkitError::ConvertRefusal(format!(
        "--to {} is cryptographically unrecoverable from --from {} (one-way derivation barrier)",
        to.as_str(),
        from.as_str(),
    ))
}

fn refusal_sibling_pivot(from: NodeType, to: NodeType) -> ToolkitError {
    ToolkitError::ConvertRefusal(format!(
        "--from {} --to {} is a sibling-format pivot, not a single-format conversion. Use 'mnemonic bundle' instead.",
        from.as_str(),
        to.as_str(),
    ))
}

fn refusal_xpub_to_mk1() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--to mk1 requires a policy descriptor binding (mk1 cards bind xpubs to specific policies via policy_id_stubs). Use 'mnemonic bundle --slot @0.xpub=... --template ...' to emit a complete bundle.".into(),
    )
}

fn refusal_phrase_entropy_to_wif_no_path() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--to wif requires explicit --path; supply a BIP-32 path producing a leaf privkey (the toolkit does not auto-default a path from --template/--account).".into(),
    )
}

fn refusal_xpub_prefix_no_network() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--xpub-prefix <variant> requires explicit --network (cannot infer mainnet vs. testnet swap from defaults).".into(),
    )
}

fn refusal_wif_with_path() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--from wif does not retain a chain code; --path-driven derivation is impossible.".into(),
    )
}

/// Direct edges supported per SPEC §2.
/// Used as the negative-space check for the catch-all refusal: any (from, to)
/// NOT in this set is a one-way barrier.
fn is_supported_direct_edge(from: NodeType, to: NodeType) -> bool {
    use NodeType::*;
    matches!(
        (from, to),
        (Phrase, Entropy)
            | (Entropy, Phrase)
            | (Phrase, Xpub)
            | (Phrase, Xprv)
            | (Phrase, Fingerprint)
            | (Phrase, Ms1)
            | (Phrase, Wif)        // SPEC-A v0.6.1
            | (Entropy, Xpub)
            | (Entropy, Xprv)
            | (Entropy, Fingerprint)
            | (Entropy, Ms1)
            | (Entropy, Wif)       // SPEC-A v0.6.1
            | (Xprv, Xpub)
            | (Xprv, Fingerprint)
            | (Xpub, Fingerprint)
            | (Xpub, Xpub)         // SPEC v0.6.1 §2 — encoding-only normalization (§11/§11.a primitive)
            | (Wif, Xpub)
            | (Wif, Fingerprint)
            | (Ms1, Entropy)
            | (Ms1, Phrase)
            | (Mk1, Xpub)
            | (Mk1, Fingerprint)
            | (Mk1, Path)
    )
}

/// Returns Some(refusal) for a refused (from, to) edge; None when permitted.
fn classify_edge(from: NodeType, to: NodeType) -> Option<ToolkitError> {
    use NodeType::*;

    // §3.c distinct xpub→mk1 message.
    if from == Xpub && to == Mk1 {
        return Some(refusal_xpub_to_mk1());
    }

    // §3.c sibling pivots between codec formats.
    let codec_set = [Ms1, Mk1];
    if codec_set.contains(&from) && codec_set.contains(&to) && from != to {
        return Some(refusal_sibling_pivot(from, to));
    }

    // §3.a/§4 catch-all: any non-supported edge is a one-way barrier.
    if !is_supported_direct_edge(from, to) {
        return Some(refusal_one_way(from, to));
    }

    None
}

// ============================================================================
// SPEC §5.a stdin
// ============================================================================

fn read_stdin_to_string<R: Read>(stdin: &mut R) -> Result<String, ToolkitError> {
    let mut buf = String::new();
    stdin
        .read_to_string(&mut buf)
        .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
    Ok(buf.trim().to_string())
}

// ============================================================================
// dispatch entry
// ============================================================================

pub fn run<R: Read, W: Write, E: Write>(
    args: &ConvertArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // 1) Single-from-value constraint (§5).
    let mut primaries: Vec<&FromInput> = args
        .from
        .iter()
        .filter(|f| !f.node.is_side_input_only())
        .collect();
    if primaries.is_empty() {
        return Err(ToolkitError::BadInput(
            "--from requires at least one primary value-bearing node (phrase, entropy, xpub, xprv, wif, ms1, mk1)".into(),
        ));
    }
    if primaries.len() > 1 {
        return Err(ToolkitError::BadInput(format!(
            "--from accepts at most one primary value-bearing node in v0.6; got {} ({})",
            primaries.len(),
            primaries
                .iter()
                .map(|f| f.node.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )));
    }
    let primary = primaries.pop().unwrap();

    // 2) Stdin if `--from <node>=-`.
    let primary_value = if primary.value == "-" {
        read_stdin_to_string(stdin)?
    } else {
        primary.value.clone()
    };

    // 3) Parse `--to`.
    let mut targets: Vec<NodeType> = Vec::new();
    for chunk in &args.to {
        for tok in chunk.split(',') {
            let t = tok.trim();
            if t.is_empty() {
                return Err(ToolkitError::BadInput(format!(
                    "--to value contains an empty token; got {:?}",
                    chunk
                )));
            }
            let n = NodeType::from_token(t).ok_or_else(|| {
                ToolkitError::BadInput(format!(
                    "unknown --to node {:?}; expected one of: phrase, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1",
                    t
                ))
            })?;
            targets.push(n);
        }
    }
    if targets.is_empty() {
        return Err(ToolkitError::BadInput(
            "--to requires at least one node".into(),
        ));
    }

    // 4) §3 refusal pre-check.
    for &t in &targets {
        if let Some(e) = classify_edge(primary.node, t) {
            return Err(e);
        }
    }

    // 5) §4 WIF + --path guard.
    if primary.node == NodeType::Wif && args.path.is_some() {
        return Err(refusal_wif_with_path());
    }

    // 5.a) SPEC §11.a — `--xpub-prefix` (non-default) requires explicit `--network`.
    if let Some(prefix) = args.xpub_prefix {
        if !prefix.is_default() && args.network.is_none() {
            return Err(refusal_xpub_prefix_no_network());
        }
    }

    // 6) §8 --passphrase warning when not on PBKDF2 edge.
    //    SPEC-A v0.6.1: `Wif` joins the PBKDF2-bearing target set so
    //    `--from phrase --to wif --passphrase x` does NOT spuriously
    //    fire the ignored-passphrase warning (phrase → seed → master
    //    → derive at path → leaf privkey → WIF traverses PBKDF2).
    let edge_uses_pbkdf2 = matches!(primary.node, NodeType::Phrase | NodeType::Entropy)
        && targets.iter().any(|t| {
            matches!(
                t,
                NodeType::Xpub | NodeType::Xprv | NodeType::Fingerprint | NodeType::Wif
            )
        });
    if args.passphrase.is_some() && !edge_uses_pbkdf2 {
        let _ = writeln!(
            stderr,
            "warning: --passphrase ignored on this edge (not a PBKDF2-bearing conversion)",
        );
    }

    // 7) §2 wif→xpub sentinel warning (chain-code zeroed; not BIP-32 derivable).
    if primary.node == NodeType::Wif && targets.iter().any(|t| *t == NodeType::Xpub) {
        let _ = writeln!(
            stderr,
            "warning: wif → xpub emits a depth-0 sentinel with a zeroed chain code; this xpub is not BIP-32 derivable",
        );
    }

    // 8) Compute outputs.
    let mut outputs = compute_outputs(primary.node, &primary_value, &targets, args)?;

    // 8.a) SPEC §11.a — apply --xpub-prefix to xpub-typed outputs. The flag
    //      is silently ignored when no xpub target is present (per §11.a).
    if let Some(prefix) = args.xpub_prefix {
        if !prefix.is_default() {
            // §5.a refusal already enforced --network presence above; safe to
            // unwrap_or default for the swap-target lookup.
            let network = args.network.unwrap_or(CliNetwork::Mainnet);
            for (node, value) in outputs.iter_mut() {
                if *node == NodeType::Xpub {
                    let xpub = bip32::Xpub::from_str(value)
                        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
                    *value = apply_xpub_prefix(&xpub, prefix, network);
                }
            }
        }
    }

    // 8) Emit.
    if args.json {
        let from_value = if primary.node.is_secret_bearing() {
            None
        } else {
            Some(primary_value.as_str())
        };
        let entries: Vec<ConvertJsonEntry> = outputs
            .iter()
            .map(|(node, value)| ConvertJsonEntry {
                node: node.as_str(),
                value: value.clone(),
            })
            .collect();
        let env = ConvertJson {
            schema_version: "1",
            from_node: primary.node.as_str(),
            from_value,
            to: entries,
        };
        serde_json::to_writer(&mut *stdout, &env).ok();
        writeln!(stdout).ok();
    } else {
        for (node, value) in &outputs {
            writeln!(stdout, "{}: {}", node.as_str(), value).ok();
        }
    }

    // 9) §7 secret-on-stdout warning.
    if outputs.iter().any(|(n, _)| n.is_secret_bearing()) {
        let _ = writeln!(
            stderr,
            "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')",
        );
    }

    Ok(0)
}

// ============================================================================
// edge dispatch
// ============================================================================

type Output = (NodeType, String);

fn compute_outputs(
    from: NodeType,
    value: &str,
    targets: &[NodeType],
    args: &ConvertArgs,
) -> Result<Vec<Output>, ToolkitError> {
    use NodeType::*;
    let language = args.language.unwrap_or_default();
    let passphrase = args.passphrase.as_deref().unwrap_or("");
    let network = args.network.unwrap_or(CliNetwork::Mainnet);
    let secp = Secp256k1::new();

    match from {
        Phrase | Entropy => {
            // BIP-39 source — derive once, project.
            let entropy: Vec<u8> = if from == Phrase {
                let m = Mnemonic::parse_in(language.into(), value)
                    .map_err(ToolkitError::Bip39)?;
                m.to_entropy()
            } else {
                hex::decode(value).map_err(|e| {
                    ToolkitError::BadInput(format!("--from entropy hex-decode: {e}"))
                })?
            };

            let needs_derive = targets
                .iter()
                .any(|t| matches!(t, Xpub | Xprv | Fingerprint));
            let derived = if needs_derive {
                let template = args.template.ok_or_else(|| {
                    ToolkitError::BadInput(
                        "--template is required for derivation targets (xpub/xprv/fingerprint)".into(),
                    )
                })?;
                Some(derive_bip32_from_entropy(
                    &entropy, passphrase, language, network, template, args.account,
                )?)
            } else {
                None
            };

            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Phrase => Mnemonic::from_entropy_in(language.into(), &entropy)
                        .map_err(ToolkitError::Bip39)?
                        .to_string(),
                    Entropy => hex::encode(&entropy),
                    Xpub => derived.as_ref().unwrap().account_xpub.to_string(),
                    Xprv => derived.as_ref().unwrap().account_xpriv.to_string(),
                    Fingerprint => derived
                        .as_ref()
                        .unwrap()
                        .master_fingerprint
                        .to_string()
                        .to_lowercase(),
                    Ms1 => ms_codec::encode(
                        ms_codec::Tag::ENTR,
                        &ms_codec::Payload::Entr(entropy.clone()),
                    )
                    .map_err(ToolkitError::from)?,
                    Wif => {
                        // SPEC-A v0.6.1: phrase/entropy → wif requires explicit
                        // --path. `needs_derive` deliberately does NOT include
                        // Wif, so --template is not required for this edge.
                        let path_str = args.path.as_deref().ok_or_else(refusal_phrase_entropy_to_wif_no_path)?;
                        let path = bip32::DerivationPath::from_str(path_str)
                            .map_err(|e| ToolkitError::BadInput(format!("--path parse: {e}")))?;
                        let leaf_xpriv = derive_bip32_at_path(
                            &entropy, passphrase, language, network, &path,
                        )?;
                        // BIP-32 §4 mandates compressed pubkeys for derived
                        // keys; WIF compression follows the BIP-32 contract.
                        let pk = PrivateKey {
                            compressed: true,
                            network: network.network_kind(),
                            inner: leaf_xpriv.private_key,
                        };
                        pk.to_wif()
                    }
                    Path => return Err(ToolkitError::BadInput(
                        "--to path is informational; not emitted as a value".into(),
                    )),
                    Mk1 => unreachable!("classify_edge intercepts (Phrase|Entropy, Mk1) as one-way barrier"),
                };
                out.push((t, v));
            }
            Ok(out)
        }
        Xprv => {
            let xprv = bip32::Xpriv::from_str(value)
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
            let xpub = bip32::Xpub::from_priv(&secp, &xprv);
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Xpub => xpub.to_string(),
                    Fingerprint => xpub.fingerprint().to_string().to_lowercase(),
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from xprv --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok(out)
        }
        Xpub => {
            // SPEC v0.6.1 §11 — accept SLIP-0132 prefix variants on input.
            let value = normalize_xpub_prefix(value)?;
            let xpub = bip32::Xpub::from_str(&value)
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Fingerprint => xpub.fingerprint().to_string().to_lowercase(),
                    // SPEC v0.6.1 §2 — encoding-only normalization. Default
                    // emit is the neutral xpub/tpub; any --xpub-prefix swap
                    // happens in run() after compute_outputs.
                    Xpub => xpub.to_string(),
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from xpub --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok(out)
        }
        Wif => {
            let pk = PrivateKey::from_wif(value)
                .map_err(|e| ToolkitError::BadInput(format!("--from wif parse: {e}")))?;
            let pubkey = pk.public_key(&secp);
            let sentinel_xpub = bip32::Xpub {
                network: network.network_kind().into(),
                depth: 0,
                parent_fingerprint: bip32::Fingerprint::default(),
                child_number: bip32::ChildNumber::Normal { index: 0 },
                public_key: pubkey.inner,
                chain_code: bip32::ChainCode::from([0u8; 32]),
            };
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Xpub => sentinel_xpub.to_string(),
                    Fingerprint => sentinel_xpub.fingerprint().to_string().to_lowercase(),
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from wif --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok(out)
        }
        Ms1 => {
            let (_tag, payload) = ms_codec::decode(value).map_err(ToolkitError::from)?;
            let entropy = match payload {
                ms_codec::Payload::Entr(bytes) => bytes,
                _ => {
                    return Err(ToolkitError::BadInput(
                        "ms1 decoded to a non-Entr payload; v0.1 ms-codec emits only Entr".into(),
                    ))
                }
            };
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Entropy => hex::encode(&entropy),
                    Phrase => Mnemonic::from_entropy_in(language.into(), &entropy)
                        .map_err(ToolkitError::Bip39)?
                        .to_string(),
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from ms1 --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok(out)
        }
        Mk1 => {
            let tokens: Vec<&str> = value.split_whitespace().collect();
            let card = mk_codec::decode(&tokens).map_err(ToolkitError::from)?;
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Xpub => card.xpub.to_string(),
                    Fingerprint => card
                        .origin_fingerprint
                        .map(|f| f.to_string().to_lowercase())
                        .ok_or_else(|| {
                            ToolkitError::BadInput(
                                "mk1 card has no origin_fingerprint; cannot project --to fingerprint".into(),
                            )
                        })?,
                    Path => card.origin_path.to_string(),
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from mk1 --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok(out)
        }
        Fingerprint | Path => Err(ToolkitError::BadInput(format!(
            "--from {} is not a primary value-bearing node",
            from.as_str()
        ))),
    }
}
