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

    // `inspect` has no indel mode; always keep the strict typed-flag HRP gate.
    let groups = repair::resolve_groups(args, "inspect", stdin, stderr, false)?;
    let mut kinds: Vec<crate::secret_advisory::OutputClass> = Vec::new();

    // P2.3 (pathless partial-decode): the unresolved-origin `@N` indices of the
    // md1 card if it decodes partial. `resolve_groups` collapses all `--md1`
    // values into a SINGLE md1 group, so there is at most one md1 card per
    // invocation (no multi-card ambiguity). A non-empty value forces exit 4
    // (VERIFY-ME) + a stderr note after the loop.
    let mut partial_indices: Option<Vec<u8>> = None;

    // Emit per-kind reports in fixed (ms1, mk1, md1) order for deterministic
    // output regardless of CLI arg ordering.
    for (kind, chunks) in &groups {
        kinds.push(crate::secret_advisory::card_kind_class(*kind));
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

        // P2.3: record an unresolved-origin md1 (dead card) for the exit-4 +
        // stderr-note handling below. Computed BEFORE the emit borrow so the
        // marker/JSON render (which re-queries `d`) stays in the emit fns.
        if let InspectPayload::Md1(d) = &payload {
            let unres = d.unresolved_origin_indices();
            if !unres.is_empty() {
                partial_indices = Some(unres);
            }
        }

        if args.json {
            emit_inspect_json(&payload, args.reveal_secret, stdout)?;
        } else {
            emit_inspect_text(&payload, args.reveal_secret, stdout)?;
        }
    }

    // Output-class advisory: worst class over all card kinds inspected to stdout.
    // Supersedes D9 ms1-only gate: mk1→WatchOnly, md1→Template, ms1→PrivateKeyMaterial.
    if let Some(c) = crate::secret_advisory::worst_class_on_stdout(&kinds) {
        crate::secret_advisory::emit_output_class_advisory(c, stderr);
    }

    // P2.3: a dead md1 card partial-decoded — emit the stderr note (mirrors
    // md-cli's `md inspect`) + exit 4 (VERIFY-ME). The template + marker are
    // already on stdout; the origin is genuinely unspecified on this backup.
    if let Some(unres) = &partial_indices {
        crate::cmd::md1_partial::emit_partial_stderr_note(unres, stderr);
        return Ok(4);
    }

    Ok(0)
}

/// Decoded card payload (variant per kind).
///
/// wave2 T2 Site B (v0.71.0): the `Ms1` variant carries the decoded master-seed
/// entropy as a `Zeroizing<Vec<u8>>` (was a bare `ms_codec::Payload` held for
/// the whole handler scope and dropped un-scrubbed). The small display bits the
/// emit fns need (`kind` / `language`) are read off the bare `Payload` at decode
/// time, BEFORE the husk drops.
pub enum InspectPayload {
    Ms1 {
        tag: ms_codec::Tag,
        /// Decoded entropy bytes, scrub-on-drop. Raw `Zeroizing<Vec<u8>>` (not
        /// `SecretString`) — these are raw entropy BYTES, never Debug-printed.
        entropy: zeroize::Zeroizing<Vec<u8>>,
        /// `PayloadKind` (Copy) — rendered `{:?}` as `payload_kind`.
        kind: ms_codec::PayloadKind,
        /// `Some(wire_lang_code)` for `Mnem`, `None` for `Entr`.
        language: Option<u8>,
    },
    Mk1(mk_codec::KeyCard),
    Md1(md_codec::Descriptor),
}

fn decode_card(kind: CardKind, chunks: &[&str]) -> Result<InspectPayload, ToolkitError> {
    match kind {
        CardKind::Ms1 => {
            let (tag, payload) = ms_codec::decode(chunks[0])?;
            // Read the small display bits BEFORE moving the entropy out (the
            // move-match consumes `payload`).
            let payload_kind = payload.kind();
            let language = match &payload {
                ms_codec::Payload::Mnem { language, .. } => Some(*language),
                _ => None,
            };
            // wave2 T2 Site B: move the entropy out of the bare `Payload` into a
            // `Zeroizing<Vec<u8>>` so the decoded master-seed entropy scrubs on
            // drop. The `_` arm covers `Payload`'s `#[non_exhaustive]` variants.
            let entropy: zeroize::Zeroizing<Vec<u8>> = match payload {
                ms_codec::Payload::Entr(b) => zeroize::Zeroizing::new(b),
                ms_codec::Payload::Mnem { entropy, .. } => zeroize::Zeroizing::new(entropy),
                ref other => zeroize::Zeroizing::new(other.as_bytes().to_vec()),
            };
            Ok(InspectPayload::Ms1 {
                tag,
                entropy,
                kind: payload_kind,
                language,
            })
        }
        CardKind::Mk1 => Ok(InspectPayload::Mk1(mk_codec::decode(chunks)?)),
        // P2.3 (pathless partial-decode): opt the md1 decode into the
        // partial-allowing entry. A `canonical_origin == None` card with an
        // elided-and-unresolvable origin (a "dead card") now decodes Ok (with
        // non-empty `unresolved_origin_indices()`) instead of hard-rejecting
        // `MissingExplicitOrigin` — the render path then marks the origin
        // unspecified + exits 4 (see `emit_inspect_*`). Every other decode
        // check (per-chunk BCH, cross-chunk content-id oracle) stays enforced.
        // Intake is CHUNK-FORM only (unchanged); a plain single-string md1 still
        // hits the pre-existing `unsupported version 2` gap (FOLLOWUP
        // `toolkit-inspect-nonchunked-md1-intake-gap`).
        CardKind::Md1 => Ok(InspectPayload::Md1(md_codec::reassemble_with_opts(
            chunks,
            md_codec::DecodeOpts::partial(),
        )?)),
    }
}

fn emit_inspect_text<W: Write>(
    payload: &InspectPayload,
    reveal_secret: bool,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    match payload {
        InspectPayload::Ms1 {
            tag,
            entropy,
            kind,
            language,
        } => {
            let tag_str = std::str::from_utf8(tag.as_bytes()).unwrap_or("<non-utf8>");
            let bytes: &[u8] = entropy;
            let bit_strength = bytes.len() * 8;
            writeln!(stdout, "kind: ms1").map_err(ToolkitError::Io)?;
            writeln!(stdout, "tag: {tag_str}").map_err(ToolkitError::Io)?;
            writeln!(stdout, "payload_kind: {kind:?}").map_err(ToolkitError::Io)?;
            // ms mnem Phase 3 Step 6: surface language for mnem cards.
            if let Some(lang_code) = language {
                let lang_name = ms_codec::consts::MNEM_LANGUAGE_NAMES
                    .get(*lang_code as usize)
                    .copied()
                    .unwrap_or("unknown");
                writeln!(stdout, "language: {lang_name}").map_err(ToolkitError::Io)?;
            }
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
            // v0.75.0: lead the md1 body with the canonical keyless `@N`
            // wallet-policy template (md-inspect parity; `md_codec` and `md`
            // share the renderer → byte-identical). N1 conscious trade-off —
            // md1 uniquely leads with `template:` vs ms1/mk1 leading with `kind:`.
            writeln!(stdout, "template: {}", md_codec::descriptor_to_template(d)?)
                .map_err(ToolkitError::Io)?;
            // P2.3 (partial only): a dead card renders its (origin-independent)
            // template PLUS an explicit unspecified-origin marker — NEVER a fake
            // `m/` path. Byte-identical to `md decode`/`md inspect` (cross-binary
            // parity). Placed right after `template:`, mirroring md-cli.
            if !d.unresolved_origin_indices().is_empty() {
                writeln!(
                    stdout,
                    "{}",
                    crate::cmd::md1_partial::ORIGIN_UNSPECIFIED_MARKER
                )
                .map_err(ToolkitError::Io)?;
            }
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
/// v0.75.0 → `"2"`: the md1 body gained a `template` field (the keyless `@N`
/// wallet-policy template). `schema_version` is the SHARED top-level
/// `InspectEnvelope` field, so ms1/mk1 `--json` envelopes also report `"2"`
/// even though their bodies are unchanged (the inspect `--json` contract
/// versions as a whole). `mnemonic repair` carries its OWN, independent
/// `schema_version` (still `"1"`) — this bump does not touch it.
pub const INSPECT_SCHEMA_VERSION: &str = "2";

/// v0.27.0 top-level wrapper. Always emits `schema_version` (now `"2"`, v0.75.0)
/// at the top level; the per-kind body is `#[serde(flatten)]`'d so the resulting
/// JSON has shape `{"schema_version":"2","kind":"<kind>",...}`. Mirrors
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
        /// ms mnem Phase 3 Step 6: Some(name) for mnem cards; None for entr cards.
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<&'a str>,
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
        /// v0.75.0: the canonical keyless `@N` wallet-policy template, rendered
        /// by `md_codec::descriptor_to_template` (identical to the text-form
        /// `template:` line and to `md decode`). Bumps the shared envelope's
        /// `schema_version` to `"2"`.
        template: String,
        placeholder_count: u8,
        tree_tag: String,
        wallet_policy_mode: bool,
        path_decl_shape: &'static str,
        /// P2.3 (pathless partial-decode): present ONLY on a dead card (elided,
        /// unresolvable origin). Additive — omitted on canonical/explicit-origin
        /// cards so their JSON stays byte-identical (schema_version unchanged).
        #[serde(skip_serializing_if = "Option::is_none")]
        partial: Option<crate::format::PartialDecodeInfo>,
    },
}

fn emit_inspect_json<W: Write>(
    payload: &InspectPayload,
    reveal_secret: bool,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    let body = match payload {
        InspectPayload::Ms1 {
            tag,
            entropy,
            kind,
            language,
        } => {
            let tag_str = std::str::from_utf8(tag.as_bytes()).unwrap_or("<non-utf8>");
            let bytes: &[u8] = entropy;
            // ms mnem Phase 3 Step 6: surface language name for mnem cards.
            let language = (*language).and_then(|code| {
                ms_codec::consts::MNEM_LANGUAGE_NAMES
                    .get(code as usize)
                    .copied()
            });
            InspectJson::Ms1 {
                tag: tag_str,
                payload_kind: format!("{kind:?}"),
                language,
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
        InspectPayload::Md1(d) => {
            let unres = d.unresolved_origin_indices();
            InspectJson::Md1 {
                // v0.75.0: identical render to the text-form `template:` line.
                template: md_codec::descriptor_to_template(d)?,
                placeholder_count: d.n,
                tree_tag: format!("{:?}", d.tree.tag),
                wallet_policy_mode: d.is_wallet_policy(),
                path_decl_shape: path_decl_shape(d),
                // P2.3: additive partial marker on a dead card only.
                partial: (!unres.is_empty())
                    .then(|| crate::format::PartialDecodeInfo::missing_explicit_origin(unres)),
            }
        }
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
            language: None,
            byte_length: 16,
            bit_strength: 128,
            entropy_hex: None,
        };
        let envelope = InspectEnvelope {
            schema_version: INSPECT_SCHEMA_VERSION,
            body,
        };
        let v = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(v["schema_version"], "2");
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
        assert_eq!(v["schema_version"], "2");
        assert_eq!(v["kind"], "mk1");
        assert_eq!(v["policy_id_stub_count"], 2);
        assert_eq!(v["origin_fingerprint"], "aabbccdd");
        assert_eq!(v["origin_path"], "m/84'/0'/0'");
        assert_eq!(v["xpub"], "xpub6...");
    }

    #[test]
    fn inspect_envelope_md1_serializes_schema_version_and_flattens_body() {
        let body = InspectJson::Md1 {
            template: "wpkh(@0/<0;1>/*)".to_string(),
            placeholder_count: 1,
            tree_tag: "Wpkh".to_string(),
            wallet_policy_mode: true,
            path_decl_shape: "Shared",
            partial: None,
        };
        let envelope = InspectEnvelope {
            schema_version: INSPECT_SCHEMA_VERSION,
            body,
        };
        let v = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(v["schema_version"], "2");
        assert_eq!(v["kind"], "md1");
        assert_eq!(v["template"], "wpkh(@0/<0;1>/*)");
        assert_eq!(v["placeholder_count"], 1);
        assert_eq!(v["tree_tag"], "Wpkh");
        assert_eq!(v["wallet_policy_mode"], true);
        assert_eq!(v["path_decl_shape"], "Shared");
    }
}
