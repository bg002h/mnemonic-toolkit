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
use crate::repair::{self, CardArgs, CardKind};
use clap::{ArgGroup, Args};
use std::io::{Read, Write};

#[derive(Args, Debug)]
#[command(group(
    // v0.24.0 §2.C.1 (D35 fold) — drop cross-HRP `conflicts_with_all` on
    // the three flag args. Cards self-identify by HRP; mixed-HRP invocations
    // are valid (`mnemonic inspect ms1xxx mk1yyy md1zzz`).
    ArgGroup::new("kind")
        .args(["ms1", "mk1", "md1"])
        .required(false)
        .multiple(true),
))]
pub struct InspectArgs {
    /// Single ms1 chunk to inspect. Use `-` to read one chunk from stdin.
    /// May be combined with --mk1 / --md1 per D35.
    #[arg(long, value_name = "MS1")]
    pub ms1: Option<String>,

    /// One or more mk1 chunks to inspect (repeating flag). Use `-` to
    /// read chunks from stdin (one per line). May be combined with
    /// --ms1 / --md1 per D35.
    #[arg(long, value_name = "MK1")]
    pub mk1: Vec<String>,

    /// One or more md1 chunks to inspect (repeating flag). Use `-` to
    /// read chunks from stdin (one per line). May be combined with
    /// --ms1 / --mk1 per D35.
    #[arg(long, value_name = "MD1")]
    pub md1: Vec<String>,

    /// Emit a single JSON envelope on stdout instead of the text-form report.
    #[arg(long)]
    pub json: bool,

    /// Reveal the ms1 entropy hex on stdout. Default suppresses it (the
    /// summary stays at length / bit-strength). No effect for mk1 / md1
    /// (those payloads are not BIP-39 entropy and carry no secret material).
    #[arg(long)]
    pub reveal_secret: bool,

    /// v0.24.0 §2.C.1 — positional `<STRING>...` intake. Each value
    /// self-identifies by HRP prefix (`ms1` / `mk1` / `md1`) and is routed
    /// to the same internal storage as the matching typed flag. Unknown
    /// HRPs are rejected with `ToolkitError::UnknownHrp`. At least one of
    /// {--ms1, --mk1, --md1, positional} is required.
    #[arg(
        value_name = "STRING",
        num_args = 0..,
        required_unless_present_any = ["ms1", "mk1", "md1"],
    )]
    pub extra_strings: Vec<String>,
}

impl CardArgs for InspectArgs {
    fn ms1(&self) -> Option<&String> {
        self.ms1.as_ref()
    }
    fn mk1(&self) -> &[String] {
        &self.mk1
    }
    fn md1(&self) -> &[String] {
        &self.md1
    }
    fn extra_strings(&self) -> &[String] {
        &self.extra_strings
    }
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &InspectArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    // v0.25.0 §2.B — TTY-conditional auto-fire (mirrors verify_bundle's
    // v0.22.1 D18 contract). See `crate::repair::resolve_no_auto_repair` for
    // the full public-API contract: `MNEMONIC_FORCE_TTY={0,1}` forces the
    // gate; unset → runtime `is_terminal()` detection. Computed once at
    // function entry and threaded to the downstream auto-fire call so
    // piped consumers (no TTY) see the typed decode error instead of
    // exit 5 short-circuit.
    let effective_no_auto_repair = crate::repair::resolve_no_auto_repair(no_auto_repair);

    let groups = repair::resolve_groups(args, "inspect", stdin)?;
    let mut any_ms1 = false;

    // Emit per-kind reports in fixed (ms1, mk1, md1) order for deterministic
    // output regardless of CLI arg ordering.
    for (kind, chunks) in &groups {
        if matches!(kind, CardKind::Ms1) {
            any_ms1 = true;
        }
        let chunks_ref: Vec<&str> = chunks.iter().map(String::as_str).collect();

        let payload = match decode_card(*kind, &chunks_ref) {
            Ok(p) => p,
            Err(orig) => {
                // v0.22.0 auto-fire — same pattern as `cmd/convert.rs`. On a
                // sibling-codec decode failure, attempt BCH correction and
                // short-circuit with exit 5. Falls through to typed `orig` if
                // repair fails or the error wasn't a decode-class failure.
                // v0.25.0 §2.B — use `effective_no_auto_repair` (TTY-aware)
                // so piped invocations skip auto-fire by default; see top-of-fn.
                if !effective_no_auto_repair {
                    let is_codec_decode_err = matches!(
                        &orig,
                        ToolkitError::MsCodec(_)
                            | ToolkitError::MkCodec(_)
                            | ToolkitError::MdCodec(_)
                    );
                    if is_codec_decode_err {
                        // v0.22.1 D20: pass args.json so the auto-fire emits
                        // a JSON envelope on stdout when --json was requested.
                        crate::repair::try_repair_and_short_circuit(
                            *kind, chunks, stdout, stderr, args.json,
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
    }

    // Secret-on-stdout discipline mirrors `cmd/repair.rs`: ms1 entropy is
    // sensitive even when only the bit-strength summary is on stdout (we
    // already write a length-hint to stdout). Warn whenever a Ms1 is being
    // inspected to a non-secret stream.
    if any_ms1 {
        crate::secret_advisory::secret_on_stdout_warning(CardKind::Ms1, stderr);
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

/// v0.27.0 inspect-envelope schema. Bumped with each wire-shape change.
pub const INSPECT_SCHEMA_VERSION: &str = "1";

/// v0.27.0 top-level wrapper. Always emits `schema_version: "1"` at the top
/// level; the per-kind body is `#[serde(flatten)]`'d so the resulting JSON
/// has shape `{"schema_version":"1","kind":"<kind>",...}`. Mirrors
/// `XpubSearchEnvelope` precedent (`crate::cmd::xpub_search::mod`).
#[derive(serde::Serialize)]
struct InspectEnvelope<'a> {
    schema_version: &'static str,
    #[serde(flatten)]
    body: InspectJson<'a>,
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
    let body = match payload {
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
    let envelope = InspectEnvelope {
        schema_version: INSPECT_SCHEMA_VERSION,
        body,
    };
    let body_str = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("inspect JSON serialize: {e}")))?;
    writeln!(stdout, "{body_str}").map_err(ToolkitError::Io)?;
    Ok(())
}

#[cfg(test)]
mod inspect_envelope_tests {
    //! Unit cell: `InspectEnvelope` serde round-trip. Pins the
    //! `#[serde(flatten)]` + inner `tag = "kind"` shape against accidental
    //! breakage. Mirrors `XpubSearchEnvelope` precedent.

    use super::*;

    #[test]
    fn inspect_envelope_ms1_serializes_schema_version_and_flattens_body() {
        let body = InspectJson::Ms1 {
            tag: "entr",
            payload_kind: "Entr16".to_string(),
            byte_length: 16,
            bit_strength: 128,
            entropy_hex: None,
        };
        let envelope = InspectEnvelope {
            schema_version: INSPECT_SCHEMA_VERSION,
            body,
        };
        let v = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(v["schema_version"], "1");
        assert_eq!(v["kind"], "ms1");
        assert_eq!(v["tag"], "entr");
        assert_eq!(v["byte_length"], 16);
        assert_eq!(v["bit_strength"], 128);
        assert!(v["entropy_hex"].is_null());
    }

    #[test]
    fn inspect_envelope_mk1_serializes_schema_version_and_flattens_body() {
        let body = InspectJson::Mk1 {
            policy_id_stub_count: 2,
            origin_fingerprint: Some("aabbccdd".to_string()),
            origin_path: "m/84'/0'/0'".to_string(),
            xpub: "xpub6...".to_string(),
        };
        let envelope = InspectEnvelope {
            schema_version: INSPECT_SCHEMA_VERSION,
            body,
        };
        let v = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(v["schema_version"], "1");
        assert_eq!(v["kind"], "mk1");
        assert_eq!(v["policy_id_stub_count"], 2);
        assert_eq!(v["origin_fingerprint"], "aabbccdd");
        assert_eq!(v["origin_path"], "m/84'/0'/0'");
        assert_eq!(v["xpub"], "xpub6...");
    }

    #[test]
    fn inspect_envelope_md1_serializes_schema_version_and_flattens_body() {
        let body = InspectJson::Md1 {
            placeholder_count: 1,
            tree_tag: "Wpkh".to_string(),
            wallet_policy_mode: true,
            path_decl_shape: "Shared",
        };
        let envelope = InspectEnvelope {
            schema_version: INSPECT_SCHEMA_VERSION,
            body,
        };
        let v = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(v["schema_version"], "1");
        assert_eq!(v["kind"], "md1");
        assert_eq!(v["placeholder_count"], 1);
        assert_eq!(v["tree_tag"], "Wpkh");
        assert_eq!(v["wallet_policy_mode"], true);
        assert_eq!(v["path_decl_shape"], "Shared");
    }
}
