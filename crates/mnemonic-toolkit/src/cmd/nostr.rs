//! `mnemonic nostr` — wrap an existing nostr key (`npub`/`nsec`) as Bitcoin
//! addresses, descriptors, and (for `nsec`) a WIF. See
//! `design/BRAINSTORM_v0_34_0_nostr_key_wrappers.md`.

use crate::cmd::convert::ScriptType;
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::secret_advisory::{secret_in_argv_warning, secret_on_stdout_warning_unconditional};
use clap::{ArgGroup, Args};
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImportMode {
    ReadOnly,
}

/// `--import` value parser. Only `readonly` is supported in v0.34.2; `spending`
/// and `both` are reserved (forward-compatible) and rejected with a clear note.
fn parse_import_mode(s: &str) -> Result<ImportMode, String> {
    match s {
        "readonly" => Ok(ImportMode::ReadOnly),
        "spending" | "both" => Err(
            "--import: 'spending'/'both' is deferred to a future cycle; only 'readonly' is supported"
                .into(),
        ),
        other => Err(format!("--import must be 'readonly'; got {other:?}")),
    }
}

/// Build the read-only `importdescriptors` recipe from the rows' descriptors
/// (one non-ranged watch-only entry per script type), or `None` if `--import`
/// was not given.
fn build_import_recipe(args: &NostrArgs, rows: &[OutputRow]) -> Option<serde_json::Value> {
    if args.import == Some(ImportMode::ReadOnly) {
        let descs: Vec<String> = rows.iter().map(|r| r.descriptor.clone()).collect();
        Some(crate::wallet_export::import_array_single(&descs, args.timestamp.0))
    } else {
        None
    }
}

/// Emit the `import:` line (compact `importdescriptors '<json>'`) when present.
fn emit_import_line<W: Write>(
    stdout: &mut W,
    recipe: &Option<serde_json::Value>,
) -> Result<(), ToolkitError> {
    if let Some(recipe) = recipe {
        let line = serde_json::to_string(recipe)
            .map_err(|e| ToolkitError::BadInput(format!("nostr: import recipe serialize: {e}")))?;
        writeln!(stdout, "  import:      importdescriptors '{line}'").map_err(ToolkitError::Io)?;
    }
    Ok(())
}

#[derive(Args, Debug)]
#[command(group(
    // Exactly one key input is required. `--secret-stdin` is a bool: `false`
    // does not count as present, so the group fires only when it is `true`.
    ArgGroup::new("key")
        .required(true)
        .multiple(false)
        .args(["pubkey", "secret", "secret_file", "secret_stdin"]),
))]
pub struct NostrArgs {
    /// Public key: `npub1…` (NIP-19) or 64-hex x-only. Watch-only outputs.
    #[arg(long)]
    pub pubkey: Option<String>,

    /// Secret key: `nsec1…` (NIP-19) or 64-hex scalar. Adds WIF. SECRET — leaks via argv.
    #[arg(long)]
    pub secret: Option<String>,

    /// Read the secret key from a file (avoids argv exposure).
    #[arg(long = "secret-file")]
    pub secret_file: Option<std::path::PathBuf>,

    /// Read the secret key from stdin.
    #[arg(long = "secret-stdin")]
    pub secret_stdin: bool,

    /// Address/descriptor script type. Defaults to `p2tr` when neither this nor
    /// `--all-script-types` is given.
    #[arg(long = "script-type", value_parser = crate::cmd::convert::parse_script_type_arg, conflicts_with = "all_script_types")]
    pub script_type: Option<ScriptType>,

    /// Emit descriptor + address for all four script types.
    #[arg(long = "all-script-types")]
    pub all_script_types: bool,

    /// Bitcoin network (affects address HRP + WIF version byte).
    // Do NOT add a Default/#[default] derive to CliNetwork; default_value_t renders via ValueEnum.
    #[arg(long, value_enum, default_value_t = CliNetwork::Mainnet)]
    pub network: CliNetwork,

    /// Emit JSON instead of the human-readable block.
    #[arg(long)]
    pub json: bool,

    /// Emit a ready-to-paste Bitcoin Core `importdescriptors` recipe for the
    /// derived address(es). `readonly` = watch-only (the pubkey descriptor).
    /// `spending`/`both` are reserved (future cycle).
    #[arg(long, value_parser = parse_import_mode)]
    pub import: Option<ImportMode>,

    /// Bitcoin Core `importdescriptors` rescan anchor: `now` or unix seconds.
    /// Default `0` (rescan from genesis to discover an existing key's funds).
    /// Only used with `--import`.
    #[arg(long, value_parser = crate::cmd::export_wallet::parse_timestamp, default_value = "0")]
    pub timestamp: crate::cmd::export_wallet::TimestampArgValue,
}

/// One row of output (per script type) in the JSON envelope.
#[derive(serde::Serialize)]
struct OutputRow {
    script_type: String,
    descriptor: String,
    address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    electrum: Option<String>,
}

/// Top-level JSON envelope for `--json`.
#[derive(serde::Serialize)]
struct NostrJson {
    kind: &'static str, // "public" | "secret"
    x_only: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    wif: Option<String>,
    outputs: Vec<OutputRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    import: Option<serde_json::Value>,
}

// Signature MUST match the sibling pattern (by-ref args, Result<u8>); the
// dispatch is `match &cli.command`. Verify against cmd/electrum_decrypt.rs.
pub fn run<R: Read, W: Write, E: Write>(
    args: &NostrArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let types: Vec<ScriptType> = if args.all_script_types {
        vec![ScriptType::P2tr, ScriptType::P2wpkh, ScriptType::P2shP2wpkh, ScriptType::P2pkh]
    } else {
        vec![args.script_type.unwrap_or(ScriptType::P2tr)]
    };

    if let Some(p) = args.pubkey.as_deref() {
        let xonly = crate::nostr::decode_npub(p)?;

        // Build rows once; used by both render paths.
        let mut rows: Vec<OutputRow> = Vec::with_capacity(types.len());
        for st in &types {
            rows.push(OutputRow {
                script_type: st.as_str().to_owned(),
                descriptor: crate::nostr::descriptor_for(xonly, *st)?,
                address: crate::nostr::address_for(&secp, xonly, *st, args.network).to_string(),
                electrum: None,
            });
        }

        let import_recipe = build_import_recipe(args, &rows);

        if args.json {
            let envelope = NostrJson {
                kind: "public",
                x_only: xonly.to_string(),
                wif: None,
                outputs: rows,
                import: import_recipe.clone(),
            };
            serde_json::to_writer_pretty(&mut *stdout, &envelope)
                .map_err(|e| ToolkitError::BadInput(format!("nostr: json serialize: {e}")))?;
            writeln!(stdout).map_err(ToolkitError::Io)?;
        } else {
            writeln!(stdout, "nostr key (public)").map_err(ToolkitError::Io)?;
            writeln!(stdout, "  x-only:      {xonly}").map_err(ToolkitError::Io)?;
            for row in &rows {
                writeln!(stdout, "  script-type: {}", row.script_type).map_err(ToolkitError::Io)?;
                writeln!(stdout, "  descriptor:  {}", row.descriptor).map_err(ToolkitError::Io)?;
                writeln!(stdout, "  address:     {}", row.address).map_err(ToolkitError::Io)?;
            }
            emit_import_line(stdout, &import_recipe)?;
        }
        return Ok(0);
    }

    // Resolve a secret from --secret / --secret-file / --secret-stdin.
    let secret_input: Option<zeroize::Zeroizing<String>> = if let Some(s) = &args.secret {
        secret_in_argv_warning(stderr, "--secret", "--secret-stdin");
        Some(zeroize::Zeroizing::new(s.clone()))
    } else if let Some(path) = &args.secret_file {
        Some(zeroize::Zeroizing::new(std::fs::read_to_string(path).map_err(ToolkitError::Io)?.trim().to_string()))
    } else if args.secret_stdin {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
        Some(zeroize::Zeroizing::new(buf.trim().to_string()))
    } else {
        None
    };

    if let Some(sec) = secret_input {
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(sec.as_bytes());
        let raw = crate::nostr::decode_nsec(&sec)?;
        let (norm, negated) = crate::nostr::normalize_to_even_y(&secp, raw);
        if negated {
            writeln!(stderr, "notice: nostr: secret normalized to even-y (BIP-340) for address consistency").map_err(ToolkitError::Io)?;
        }
        let (xonly, _) = norm.x_only_public_key(&secp);
        let wif = crate::nostr::wif_for(&norm, args.network);

        // Build rows once; used by both render paths.
        let mut rows: Vec<OutputRow> = Vec::with_capacity(types.len());
        for st in &types {
            rows.push(OutputRow {
                script_type: st.as_str().to_owned(),
                descriptor: crate::nostr::descriptor_for(xonly, *st)?,
                address: crate::nostr::address_for(&secp, xonly, *st, args.network).to_string(),
                electrum: crate::nostr::electrum_prefix(*st).map(|p| format!("{p}{wif}")),
            });
        }

        let import_recipe = build_import_recipe(args, &rows);

        if args.json {
            let envelope = NostrJson {
                kind: "secret",
                x_only: xonly.to_string(),
                wif: Some(wif.clone()),
                outputs: rows,
                import: import_recipe.clone(),
            };
            serde_json::to_writer_pretty(&mut *stdout, &envelope)
                .map_err(|e| ToolkitError::BadInput(format!("nostr: json serialize: {e}")))?;
            writeln!(stdout).map_err(ToolkitError::Io)?;
        } else {
            writeln!(stdout, "nostr key (secret)").map_err(ToolkitError::Io)?;
            writeln!(stdout, "  x-only:      {xonly}").map_err(ToolkitError::Io)?;
            for row in &rows {
                writeln!(stdout, "  script-type: {}", row.script_type).map_err(ToolkitError::Io)?;
                writeln!(stdout, "  descriptor:  {}", row.descriptor).map_err(ToolkitError::Io)?;
                writeln!(stdout, "  address:     {}", row.address).map_err(ToolkitError::Io)?;
                if let Some(elec) = &row.electrum {
                    writeln!(stdout, "  electrum:    {elec}").map_err(ToolkitError::Io)?;
                }
            }
            writeln!(stdout, "  wif:         {wif}").map_err(ToolkitError::Io)?;
            emit_import_line(stdout, &import_recipe)?;
        }
        secret_on_stdout_warning_unconditional(stderr);
        return Ok(0);
    }

    Err(ToolkitError::NostrKeyParse(
        "exactly one of --pubkey / --secret / --secret-file / --secret-stdin is required".into(),
    ))
}
