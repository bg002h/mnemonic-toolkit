//! `mnemonic inspect` subcommand — describe the contents of an m-format card.
//!
//! Realizes `design/IMPLEMENTATION_PLAN_repair_v0_22.md` §2.3. Decodes via
//! the sibling-codec public APIs and emits a human-readable summary:
//!   - `ms1`: tag, payload kind, byte length, bit_strength (= 8·bytes).
//!            Entropy hex withheld by default; `--reveal-secret` opts in.
//!   - `mk1`: policy-id-stub count, origin fingerprint (or `<absent>`),
//!            origin path, xpub.
//!   - `md1`: placeholder count (`n`), root-tree tag, wallet-policy mode,
//!            path-decl shape (single/divergent).
//!
//! Phase 4 ships WITHOUT auto-fire: a decode failure surfaces as a typed
//! `ToolkitError`. Phase 5 adds the `?`-propagating
//! `try_repair_and_short_circuit` call so that a corrupted card auto-repairs
//! instead of failing loudly.

use crate::error::ToolkitError;
use crate::repair::CardKind;
use clap::{ArgGroup, Args};
use std::io::{Read, Write};

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("kind")
        .args(["ms1", "mk1", "md1"])
        .required(true)
        .multiple(true),
))]
pub struct InspectArgs {
    /// Single ms1 chunk to inspect. Use `-` to read one chunk from stdin.
    /// Mutually exclusive with --mk1 / --md1.
    #[arg(long, value_name = "MS1", conflicts_with_all = ["mk1", "md1"])]
    pub ms1: Option<String>,

    /// One or more mk1 chunks to inspect (repeating flag). Use `-` to
    /// read chunks from stdin (one per line). Mutually exclusive with
    /// --ms1 / --md1.
    #[arg(long, value_name = "MK1", conflicts_with_all = ["ms1", "md1"])]
    pub mk1: Vec<String>,

    /// One or more md1 chunks to inspect (repeating flag). Use `-` to
    /// read chunks from stdin (one per line). Mutually exclusive with
    /// --ms1 / --mk1.
    #[arg(long, value_name = "MD1", conflicts_with_all = ["ms1", "mk1"])]
    pub md1: Vec<String>,

    /// Emit a single JSON envelope on stdout instead of the text-form report.
    #[arg(long)]
    pub json: bool,

    /// Reveal the ms1 entropy hex on stdout. Default suppresses it (the
    /// summary stays at length / bit-strength). No effect for mk1 / md1
    /// (those payloads are not BIP-39 entropy and carry no secret material).
    #[arg(long)]
    pub reveal_secret: bool,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &InspectArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    let (kind, chunks) = resolve_kind_and_chunks(args, stdin)?;
    let chunks_ref: Vec<&str> = chunks.iter().map(String::as_str).collect();

    let payload = match decode_card(kind, &chunks_ref) {
        Ok(p) => p,
        Err(orig) => {
            // v0.22.0 auto-fire — same pattern as `cmd/convert.rs`. On a
            // sibling-codec decode failure, attempt BCH correction and
            // short-circuit with exit 5. Falls through to typed `orig` if
            // repair fails or the error wasn't a decode-class failure.
            if !no_auto_repair {
                let is_codec_decode_err = matches!(
                    &orig,
                    ToolkitError::MsCodec(_)
                        | ToolkitError::MkCodec(_)
                        | ToolkitError::MdCodec(_)
                );
                if is_codec_decode_err {
                    crate::repair::try_repair_and_short_circuit(
                        kind, &chunks, stdout, stderr,
                    )?;
                }
            }
            return Err(orig);
        }
    };

    if args.json {
        emit_inspect_json(&payload, args.reveal_secret, stdout)?;
    } else {
        emit_inspect_text(&payload, args.reveal_secret, stdout)?;
    }

    // Secret-on-stdout discipline mirrors `cmd/repair.rs`: ms1 entropy is
    // sensitive even when only the bit-strength summary is on stdout (we
    // already write a length-hint to stdout). Warn whenever a Ms1 is being
    // inspected to a non-secret stream.
    if matches!(kind, CardKind::Ms1) {
        crate::secret_advisory::secret_on_stdout_warning(kind, stderr);
    }

    Ok(0)
}

/// Decoded card payload (variant per kind).
pub enum InspectPayload {
    Ms1 {
        tag: ms_codec::Tag,
        payload: ms_codec::Payload,
    },
    Mk1(mk_codec::KeyCard),
    Md1(md_codec::Descriptor),
}

fn decode_card(kind: CardKind, chunks: &[&str]) -> Result<InspectPayload, ToolkitError> {
    match kind {
        CardKind::Ms1 => {
            let (tag, payload) = ms_codec::decode(chunks[0])?;
            Ok(InspectPayload::Ms1 { tag, payload })
        }
        CardKind::Mk1 => Ok(InspectPayload::Mk1(mk_codec::decode(chunks)?)),
        CardKind::Md1 => Ok(InspectPayload::Md1(md_codec::reassemble(chunks)?)),
    }
}

fn resolve_kind_and_chunks<R: Read>(
    args: &InspectArgs,
    stdin: &mut R,
) -> Result<(CardKind, Vec<String>), ToolkitError> {
    let (kind, raw): (CardKind, Vec<String>) = if let Some(ms) = &args.ms1 {
        (CardKind::Ms1, vec![ms.clone()])
    } else if !args.mk1.is_empty() {
        (CardKind::Mk1, args.mk1.clone())
    } else if !args.md1.is_empty() {
        (CardKind::Md1, args.md1.clone())
    } else {
        return Err(ToolkitError::BadInput(
            "inspect: exactly one of --ms1 / --mk1 / --md1 is required".into(),
        ));
    };

    let dash_count = raw.iter().filter(|s| s.as_str() == "-").count();
    if dash_count == 0 {
        return Ok((kind, raw));
    }
    if dash_count > 1 {
        return Err(ToolkitError::BadInput(
            "inspect: at most one `-` (stdin) value across all inspect flags".into(),
        ));
    }

    let mut buf = String::new();
    stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
    let stdin_chunks: Vec<String> = buf
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    if stdin_chunks.is_empty() {
        return Err(ToolkitError::BadInput(
            "inspect: stdin (`-`) yielded no non-blank chunks".into(),
        ));
    }

    let mut out: Vec<String> = Vec::with_capacity(raw.len() - 1 + stdin_chunks.len());
    for c in raw {
        if c == "-" {
            out.extend(stdin_chunks.iter().cloned());
        } else {
            out.push(c);
        }
    }
    Ok((kind, out))
}

fn emit_inspect_text<W: Write>(
    payload: &InspectPayload,
    reveal_secret: bool,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    match payload {
        InspectPayload::Ms1 { tag, payload } => {
            let tag_str = std::str::from_utf8(tag.as_bytes()).unwrap_or("<non-utf8>");
            let bytes = payload.as_bytes();
            let bit_strength = bytes.len() * 8;
            writeln!(stdout, "kind: ms1").map_err(ToolkitError::Io)?;
            writeln!(stdout, "tag: {tag_str}").map_err(ToolkitError::Io)?;
            writeln!(stdout, "payload_kind: {:?}", payload.kind()).map_err(ToolkitError::Io)?;
            writeln!(stdout, "byte_length: {}", bytes.len()).map_err(ToolkitError::Io)?;
            writeln!(stdout, "bit_strength: {bit_strength}").map_err(ToolkitError::Io)?;
            if reveal_secret {
                writeln!(stdout, "entropy_hex: {}", hex::encode(bytes))
                    .map_err(ToolkitError::Io)?;
            } else {
                writeln!(
                    stdout,
                    "entropy_hex: <suppressed; pass --reveal-secret to print>"
                )
                .map_err(ToolkitError::Io)?;
            }
        }
        InspectPayload::Mk1(card) => {
            writeln!(stdout, "kind: mk1").map_err(ToolkitError::Io)?;
            writeln!(
                stdout,
                "policy_id_stub_count: {}",
                card.policy_id_stubs.len()
            )
            .map_err(ToolkitError::Io)?;
            match card.origin_fingerprint {
                Some(fp) => writeln!(stdout, "origin_fingerprint: {fp}"),
                None => writeln!(stdout, "origin_fingerprint: <absent (privacy-preserving)>"),
            }
            .map_err(ToolkitError::Io)?;
            writeln!(stdout, "origin_path: m/{}", card.origin_path).map_err(ToolkitError::Io)?;
            writeln!(stdout, "xpub: {}", card.xpub).map_err(ToolkitError::Io)?;
        }
        InspectPayload::Md1(d) => {
            writeln!(stdout, "kind: md1").map_err(ToolkitError::Io)?;
            writeln!(stdout, "placeholder_count: {}", d.n).map_err(ToolkitError::Io)?;
            writeln!(stdout, "tree_tag: {:?}", d.tree.tag).map_err(ToolkitError::Io)?;
            writeln!(stdout, "wallet_policy_mode: {}", d.is_wallet_policy())
                .map_err(ToolkitError::Io)?;
            writeln!(stdout, "path_decl_shape: {}", path_decl_shape(d))
                .map_err(ToolkitError::Io)?;
        }
    }
    Ok(())
}

fn path_decl_shape(d: &md_codec::Descriptor) -> &'static str {
    match &d.path_decl.paths {
        md_codec::PathDeclPaths::Shared(_) => "Shared",
        md_codec::PathDeclPaths::Divergent(_) => "Divergent",
    }
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum InspectJson<'a> {
    Ms1 {
        tag: &'a str,
        payload_kind: String,
        byte_length: usize,
        bit_strength: usize,
        entropy_hex: Option<String>,
    },
    Mk1 {
        policy_id_stub_count: usize,
        origin_fingerprint: Option<String>,
        origin_path: String,
        xpub: String,
    },
    Md1 {
        placeholder_count: u8,
        tree_tag: String,
        wallet_policy_mode: bool,
        path_decl_shape: &'static str,
    },
}

fn emit_inspect_json<W: Write>(
    payload: &InspectPayload,
    reveal_secret: bool,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    let envelope = match payload {
        InspectPayload::Ms1 { tag, payload } => {
            let tag_str = std::str::from_utf8(tag.as_bytes()).unwrap_or("<non-utf8>");
            let bytes = payload.as_bytes();
            InspectJson::Ms1 {
                tag: tag_str,
                payload_kind: format!("{:?}", payload.kind()),
                byte_length: bytes.len(),
                bit_strength: bytes.len() * 8,
                entropy_hex: if reveal_secret {
                    Some(hex::encode(bytes))
                } else {
                    None
                },
            }
        }
        InspectPayload::Mk1(card) => InspectJson::Mk1 {
            policy_id_stub_count: card.policy_id_stubs.len(),
            origin_fingerprint: card.origin_fingerprint.map(|fp| fp.to_string()),
            origin_path: format!("m/{}", card.origin_path),
            xpub: card.xpub.to_string(),
        },
        InspectPayload::Md1(d) => InspectJson::Md1 {
            placeholder_count: d.n,
            tree_tag: format!("{:?}", d.tree.tag),
            wallet_policy_mode: d.is_wallet_policy(),
            path_decl_shape: path_decl_shape(d),
        },
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("inspect JSON serialize: {e}")))?;
    writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
    Ok(())
}
