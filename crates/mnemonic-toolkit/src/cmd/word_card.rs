//! `mnemonic word-card` subcommand — encode an `mk1` / `md1` card as an
//! engravable BIP-39 **Word Card** (with optional RAID recovery plates), or
//! `--decode` a Word Card back to its `m*1` / xpub / descriptor.
//!
//! Realizes `design/IMPLEMENTATION_PLAN_word_card_encoding.md` §6.2 (toolkit CLI
//! surface) + §7 P6. The value engine lives in the `wc-codec` crate; the
//! canonical-payload adapter (`crate::word_card_adapter`) bridges the sibling
//! codecs to it.
//!
//! # PUBLIC material — NOT secrets
//!
//! Word Cards carry the xpub (`mk1`) / descriptor (`md1`) — **watch-only,
//! public-ish** material, NOT spending secrets (plan §8). So the `--from`
//! argument is NOT secret-classified (it is intentionally absent from the
//! argv-secret taxonomy in `crate::secrets` / `crate::secret_taxonomy`), and the
//! `ms1` entropy card is intentionally NOT word-card-able.
//!
//! # Encode vs decode
//!
//! - **encode (default):** `--from <mk1|md1>` (repeatable; `-` reads stdin, one
//!   per line). With `--raid 1|2`, the supplied `--from` values must all be
//!   `mk1` xpub cards forming an array; the output is `n` data plates + `r`
//!   recovery plates.
//! - **decode:** `--decode <WORD>...` (or `-` / stdin) recovers the payload and
//!   re-emits the `m*1` / xpub / descriptor with a repair + truncation report.
//!   With multiple `--decode-plate` groups, RAID-reconstruct the array.

use crate::error::ToolkitError;
use clap::Args;
use std::io::{Read, Write};
use wc_codec::{EncodeOpts, SourceKind};

use crate::word_card_adapter::{canonical_to_recovered, string_to_canonical, RecoveredCard};

/// The always-on advisory attached to a `*recovered` (MDS-solved) RAID plate
/// (constellation-eval **F2** part (c)). A plate reconstructed from RAID parity
/// carries NO integrity tag of its own (the tag died with the lost plate), so a
/// same-quorum plate mix that slips past the array-id / spare-parity guards
/// cannot ALWAYS be caught in-band (legacy arrays, r=1-with-1-missing). The human
/// must independently confirm the reconstructed xpub. Fires ONLY on MDS-solved
/// plates — never on an all-present decode (G4).
const RECOVERED_XPUB_ADVISORY: &str =
    "reconstructed from RAID parity — independently verify this xpub against your \
     other records before trusting it (it carries no integrity tag of its own)";

/// The `--json` schema version for the `word-card` envelope. Bumped on any
/// wire-shape change. NOTE (lockstep): the `word-card` `--json` wire-shape is NOT
/// `schema_mirror`-gated (that gate is clap-flag-NAME parity only) — GUI
/// consumers self-update via the paired-PR rule (plan §7 P6 / `CLAUDE.md`).
///
/// - `"1"`: initial P6 shape.
/// - `"2"` (F2): a `verify_advisory` string is added to each RAID-decode plate
///   that was reconstructed via the MDS solve (`reconstructed: true`) — a loud
///   "independently verify this reconstructed xpub" mitigation.
pub const WORD_CARD_SCHEMA_VERSION: &str = "2";

#[derive(Args, Debug)]
pub struct WordCardArgs {
    /// Source `m*1` card to encode into a Word Card: an `mk1` xpub card or an
    /// `md1` descriptor card. Repeating flag — supply ONE per `mk1`/`md1` (for a
    /// multi-chunk card, pass all chunks joined OR repeat the flag; chunks are
    /// auto-grouped by HRP). Use `-` to read one card per line from stdin. With
    /// `--raid 1|2`, supply the `n` `mk1` data cards (one `--from` each).
    /// PUBLIC material (xpub / descriptor) — not a secret.
    #[arg(long, value_name = "MK1|MD1")]
    pub from: Vec<String>,

    /// Decode mode: recover the payload from an engraved Word Card. The words
    /// come from the positional `<WORD>...` list, or `-`/stdin (whitespace-
    /// separated). For a RAID array, repeat `--decode-plate` once per plate.
    #[arg(long)]
    pub decode: bool,

    /// One RAID plate's word list for `--decode` reconstruction (repeating flag;
    /// each occurrence is one plate's whitespace-separated words). Supply the
    /// surviving `≥ n` plates of an `n + r` array to reconstruct a lost data
    /// plate. Mutually exclusive with the positional `<WORD>...` single-card form.
    #[arg(long, value_name = "WORDS", conflicts_with = "words")]
    pub decode_plate: Vec<String>,

    /// Reed–Solomon parity words `m` to append (the repair budget; corrects
    /// `⌊m/2⌋` substitutions / fills `m` erasures). Default 0 (detection only).
    /// Mutually exclusive with `--parity-pct`.
    #[arg(long, value_name = "N", conflicts_with = "parity_pct")]
    pub parity_words: Option<usize>,

    /// Reed–Solomon parity as a PERCENTAGE of the data-symbol count `K`
    /// (`m = ceil(K * pct / 100)`), an alternative to `--parity-words`. E.g.
    /// `--parity-pct 25` ≈ a 25% redundancy budget. Mutually exclusive with
    /// `--parity-words`.
    #[arg(long, value_name = "PCT", conflicts_with = "parity_words")]
    pub parity_pct: Option<u8>,

    /// RAID recovery tier: `0` = no RAID (a single solo card; default), `1` =
    /// one XOR recovery plate (RAID-5, survives any 1 lost plate), `2` = two
    /// recovery plates (RAID-6, survives any 2). RAID requires `≥ 2` `mk1` data
    /// cards via repeated `--from`. (The construction admits `r ≥ 3`; only 0/1/2
    /// are surfaced — plan §4.6 / §9.)
    #[arg(long, value_name = "0|1|2", default_value_t = 0)]
    pub raid: u8,

    /// Integrity-tag bit width `t` (the non-linear SHA-256 truncation that
    /// catches an RS miscorrection at `≤ 2⁻ᵗ`). Default 44 (4 words); min 33.
    #[arg(long, value_name = "BITS", default_value_t = wc_codec::DEFAULT_INTEGRITY_BITS)]
    pub integrity_bits: u8,

    /// Emit a single JSON envelope on stdout instead of the text-form report.
    #[arg(long)]
    pub json: bool,

    /// Positional Word-Card words for `--decode` (a single solo card). Each value
    /// is one BIP-39 word; or pass `-` to read whitespace-separated words from
    /// stdin. Ignored in encode mode.
    #[arg(value_name = "WORD", num_args = 0..)]
    pub words: Vec<String>,
}

/// Compute the parity-word count from `--parity-words` / `--parity-pct` against
/// the data-symbol count `k` of a payload. `--parity-pct P` ⇒ `ceil(k·P/100)`.
fn resolve_parity_words(args: &WordCardArgs, k: usize) -> usize {
    if let Some(n) = args.parity_words {
        n
    } else if let Some(pct) = args.parity_pct {
        // ceil(k * pct / 100)
        (k * pct as usize).div_ceil(100)
    } else {
        0
    }
}

/// The number of `GF(2¹¹)` data symbols `K` a canonical payload occupies (the RS
/// `K` that `--parity-pct` is taken against): `ceil((payload_bits + t) / 11)`.
fn data_symbol_count(payload_bits: usize, integrity_bits: u8) -> usize {
    (payload_bits + integrity_bits as usize).div_ceil(11)
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &WordCardArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    if args.decode {
        run_decode(args, stdin, stdout, stderr)
    } else {
        run_encode(args, stdin, stdout, stderr)
    }
}

// ===========================================================================
// Encode.
// ===========================================================================

fn run_encode<R: Read, W: Write, E: Write>(
    args: &WordCardArgs,
    stdin: &mut R,
    stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let sources = collect_from_sources(&args.from, stdin)?;
    if sources.is_empty() {
        return Err(ToolkitError::BadInput(
            "word-card: encode requires at least one --from <mk1|md1> card (or `-` for stdin)"
                .into(),
        ));
    }
    if args.integrity_bits < wc_codec::MIN_INTEGRITY_BITS {
        return Err(ToolkitError::BadInput(format!(
            "word-card: --integrity-bits {} below the {}-bit floor",
            args.integrity_bits,
            wc_codec::MIN_INTEGRITY_BITS
        )));
    }

    if args.raid == 0 {
        run_encode_solo(args, &sources, stdout)
    } else {
        run_encode_raid(args, &sources, stdout)
    }
}

/// Encode each `--from` card as an independent solo Word Card.
fn run_encode_solo<W: Write>(
    args: &WordCardArgs,
    sources: &[String],
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    let mut cards: Vec<EncodedCardJson> = Vec::with_capacity(sources.len());
    for s in sources {
        let cp = string_to_canonical(s)?;
        let k = data_symbol_count(cp.payload_bits, args.integrity_bits);
        let opts = EncodeOpts {
            parity_words: resolve_parity_words(args, k),
            integrity_bits: args.integrity_bits,
            ..Default::default()
        };
        let words = wc_codec::encode(cp.kind, &cp.bytes, cp.payload_bits, &opts)
            .map_err(ToolkitError::from)?;
        cards.push(EncodedCardJson {
            role: "solo",
            index: 0,
            source_kind: source_kind_str(cp.kind),
            word_count: words.len(),
            words: words.iter().map(|w| w.to_string()).collect(),
        });
    }

    if args.json {
        emit_encode_json(&cards, args.raid, stdout)?;
    } else {
        emit_encode_text(&cards, stdout)?;
    }
    Ok(0)
}

/// Encode an `n`-card `mk1` array into `n` data plates + `r` recovery plates.
fn run_encode_raid<W: Write>(
    args: &WordCardArgs,
    sources: &[String],
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    if !(1..=2).contains(&args.raid) {
        return Err(ToolkitError::BadInput(format!(
            "word-card: --raid must be 0, 1, or 2 (got {})",
            args.raid
        )));
    }
    if sources.len() < 2 {
        return Err(ToolkitError::BadInput(format!(
            "word-card: --raid {} needs at least 2 mk1 data cards via repeated --from (got {})",
            args.raid,
            sources.len()
        )));
    }

    // All RAID data cards must be mk1 (xpub arrays only — plan §4.6); collect the
    // canonical payloads + the array-id seed (concatenated ordered cosigner
    // master fingerprints).
    let mut payloads: Vec<(Vec<u8>, usize)> = Vec::with_capacity(sources.len());
    let mut array_id_seed: Vec<u8> = Vec::new();
    for s in sources {
        let cp = string_to_canonical(s)?;
        if cp.kind != SourceKind::Mk1Xpub {
            return Err(ToolkitError::BadInput(
                "word-card: --raid arrays are mk1 xpub cards only (md1 is a single descriptor; \
                 use --raid 0)"
                    .into(),
            ));
        }
        // The array-id seed is the ordered concatenation of each card's master
        // fingerprint (4 bytes each); a privacy-mode card (no fingerprint)
        // contributes 4 zero bytes so the seed length stays deterministic.
        let chunk_tokens: Vec<&str> = s.split_whitespace().collect();
        let card = mk_codec::decode(&chunk_tokens).map_err(ToolkitError::from)?;
        match card.origin_fingerprint {
            Some(fp) => array_id_seed.extend_from_slice(&fp[..]),
            None => array_id_seed.extend_from_slice(&[0u8; 4]),
        }
        payloads.push((cp.bytes, cp.payload_bits));
    }

    // For RAID, parity_words applies per-plate; resolve against the widest plate.
    let max_bits = payloads.iter().map(|(_, b)| *b).max().unwrap_or(0);
    let k = data_symbol_count(max_bits, args.integrity_bits);
    let opts = EncodeOpts {
        parity_words: resolve_parity_words(args, k),
        integrity_bits: args.integrity_bits,
        ..Default::default()
    };

    let plates = wc_codec::raid_encode(&payloads, &array_id_seed, args.raid, &opts)
        .map_err(ToolkitError::from)?;

    let cards: Vec<EncodedCardJson> = plates
        .iter()
        .map(|p| EncodedCardJson {
            role: plate_role_str(p.role),
            index: p.index,
            source_kind: source_kind_str(SourceKind::Mk1Xpub),
            word_count: p.words.len(),
            words: p.words.iter().map(|w| w.to_string()).collect(),
        })
        .collect();

    if args.json {
        emit_encode_json(&cards, args.raid, stdout)?;
    } else {
        emit_encode_text(&cards, stdout)?;
    }
    Ok(0)
}

// ===========================================================================
// Decode.
// ===========================================================================

fn run_decode<R: Read, W: Write, E: Write>(
    args: &WordCardArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    if !args.decode_plate.is_empty() {
        run_decode_raid(args, stdin, stdout, stderr)
    } else {
        run_decode_solo(args, stdin, stdout)
    }
}

/// Decode a single solo Word Card from the positional `<WORD>...` / stdin.
fn run_decode_solo<R: Read, W: Write>(
    args: &WordCardArgs,
    stdin: &mut R,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    let words = collect_words(&args.words, stdin)?;
    if words.is_empty() {
        return Err(ToolkitError::BadInput(
            "word-card: --decode requires the Word-Card words (positional <WORD>... or `-` stdin)"
                .into(),
        ));
    }
    let word_refs: Vec<&str> = words.iter().map(String::as_str).collect();
    let decoded = wc_codec::decode(&word_refs).map_err(ToolkitError::from)?;
    let recovered = canonical_to_recovered(decoded.kind, &decoded.payload, decoded.payload_bits)?;

    let report = DecodeReportJson {
        source_kind: source_kind_str(decoded.kind),
        truncated: decoded.truncated,
        erasures_filled: decoded.repair.erasures_filled,
        recovered: recovered_json(&recovered),
    };

    if args.json {
        emit_decode_json(&report, stdout)?;
    } else {
        emit_decode_text(&report, stdout)?;
    }
    Ok(0)
}

/// RAID-reconstruct an array from the surviving `--decode-plate` word lists.
fn run_decode_raid<R: Read, W: Write, E: Write>(
    args: &WordCardArgs,
    _stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // Each --decode-plate value is one plate's whitespace-separated words.
    let plate_word_sets: Vec<Vec<String>> = args
        .decode_plate
        .iter()
        .map(|p| split_words(p))
        .collect::<Vec<_>>();
    let plate_refs: Vec<Vec<&str>> = plate_word_sets
        .iter()
        .map(|set| set.iter().map(String::as_str).collect())
        .collect();

    let recovery = wc_codec::raid_reconstruct(&plate_refs).map_err(ToolkitError::from)?;

    // Each recovered (bytes, bits) is an mk1 xpub payload; rebuild the card.
    let mut recovered_cards: Vec<RecoveredJson> = Vec::with_capacity(recovery.payloads.len());
    for (i, (bytes, bits)) in recovery.payloads.iter().enumerate() {
        let rc = canonical_to_recovered(SourceKind::Mk1Xpub, bytes, *bits)?;
        let mut j = recovered_json(&rc);
        j.array_index = Some(i);
        let was_reconstructed = recovery.reconstructed.contains(&i);
        j.reconstructed = Some(was_reconstructed);
        // (c, F2) MDS-solved plates carry the loud verify-this-xpub advisory —
        // never an all-present plate (G4).
        if was_reconstructed {
            j.verify_advisory = Some(RECOVERED_XPUB_ADVISORY.to_string());
        }
        recovered_cards.push(j);
    }

    let report = RaidDecodeReportJson {
        n: recovery.payloads.len(),
        reconstructed: recovery.reconstructed.clone(),
        plates: recovered_cards,
    };

    if args.json {
        let envelope = RaidDecodeEnvelope {
            schema_version: WORD_CARD_SCHEMA_VERSION,
            mode: "raid-decode",
            body: &report,
        };
        let s = serde_json::to_string(&envelope)
            .map_err(|e| ToolkitError::BadInput(format!("word-card JSON serialize: {e}")))?;
        writeln!(stdout, "{s}").map_err(ToolkitError::Io)?;
    } else {
        writeln!(
            stdout,
            "raid-reconstruct: n={}, reconstructed={:?}",
            report.n, report.reconstructed
        )
        .map_err(ToolkitError::Io)?;
        for p in &report.plates {
            let recovered = p.reconstructed == Some(true);
            writeln!(
                stdout,
                "  [{}{}] xpub: {}",
                p.array_index.unwrap_or(0),
                if recovered { " *recovered" } else { "" },
                p.xpub.as_deref().unwrap_or("<none>"),
            )
            .map_err(ToolkitError::Io)?;
            // (c, F2) loud advisory directly under each *recovered plate.
            if recovered {
                writeln!(stdout, "      ! verify: {RECOVERED_XPUB_ADVISORY}")
                    .map_err(ToolkitError::Io)?;
            }
        }
    }

    // (c, F2) A single loud stderr advisory whenever ANY plate was MDS-solved,
    // regardless of --json (so a piped-JSON consumer's operator still sees it).
    if !report.reconstructed.is_empty() {
        writeln!(
            stderr,
            "word-card: WARNING — plate(s) {:?} were reconstructed from RAID parity; \
             independently verify each *recovered xpub against your other records \
             before trusting it.",
            report.reconstructed
        )
        .map_err(ToolkitError::Io)?;
    }
    Ok(0)
}

// ===========================================================================
// I/O helpers.
// ===========================================================================

/// Resolve the `--from` source list, expanding a `-` entry to stdin lines.
fn collect_from_sources<R: Read>(
    from: &[String],
    stdin: &mut R,
) -> Result<Vec<String>, ToolkitError> {
    let mut out = Vec::new();
    for f in from {
        if f == "-" {
            let mut buf = String::new();
            stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
            for line in buf.lines() {
                let t = line.trim();
                if !t.is_empty() {
                    out.push(t.to_string());
                }
            }
        } else {
            out.push(f.clone());
        }
    }
    Ok(out)
}

/// Resolve the decode word list, expanding a `-` entry to stdin (whitespace-
/// separated).
fn collect_words<R: Read>(words: &[String], stdin: &mut R) -> Result<Vec<String>, ToolkitError> {
    if words.iter().any(|w| w == "-") {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
        Ok(split_words(&buf))
    } else {
        // Allow a single positional arg that itself contains spaces (a quoted
        // whole card) OR many single-word args.
        let joined = words.join(" ");
        Ok(split_words(&joined))
    }
}

/// Split a string into BIP-39 words on any whitespace.
fn split_words(s: &str) -> Vec<String> {
    s.split_whitespace().map(|w| w.to_string()).collect()
}

// ===========================================================================
// JSON / text emit.
// ===========================================================================

#[derive(serde::Serialize)]
struct EncodedCardJson {
    role: &'static str,
    index: usize,
    source_kind: &'static str,
    word_count: usize,
    words: Vec<String>,
}

#[derive(serde::Serialize)]
struct EncodeEnvelope<'a> {
    schema_version: &'static str,
    mode: &'static str,
    raid: u8,
    cards: &'a [EncodedCardJson],
}

fn emit_encode_json<W: Write>(
    cards: &[EncodedCardJson],
    raid: u8,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    let envelope = EncodeEnvelope {
        schema_version: WORD_CARD_SCHEMA_VERSION,
        mode: "encode",
        raid,
        cards,
    };
    let s = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("word-card JSON serialize: {e}")))?;
    writeln!(stdout, "{s}").map_err(ToolkitError::Io)?;
    Ok(())
}

fn emit_encode_text<W: Write>(
    cards: &[EncodedCardJson],
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    for card in cards {
        // Each plate: a labeled header line, then the space-joined word list.
        writeln!(
            stdout,
            "# {} plate [{}] ({}, {} words)",
            card.role, card.index, card.source_kind, card.word_count
        )
        .map_err(ToolkitError::Io)?;
        writeln!(stdout, "{}", card.words.join(" ")).map_err(ToolkitError::Io)?;
    }
    Ok(())
}

#[derive(serde::Serialize, Default)]
struct RecoveredJson {
    /// `"mk1"` | `"md1"`.
    kind: &'static str,
    /// The re-emitted `m*1` string(s) (mk1 = chunk vec; md1 = single string).
    mstring: Vec<String>,
    /// The xpub identity (mk1 only).
    #[serde(skip_serializing_if = "Option::is_none")]
    xpub: Option<String>,
    /// The count of bound-policy stubs on the rebuilt mk1 card (mk1 only).
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_id_stub_count: Option<usize>,
    /// The descriptor text (md1 only).
    #[serde(skip_serializing_if = "Option::is_none")]
    descriptor: Option<String>,
    /// RAID: this plate's data-array index (decode-raid only).
    #[serde(skip_serializing_if = "Option::is_none")]
    array_index: Option<usize>,
    /// RAID: `true` iff this plate was reconstructed via the MDS solve.
    #[serde(skip_serializing_if = "Option::is_none")]
    reconstructed: Option<bool>,
    /// RAID (F2, schema `"2"`): present ONLY on a `*recovered` (MDS-solved) plate
    /// — a loud "independently verify this reconstructed xpub" advisory. Absent on
    /// an all-present decode (G4).
    #[serde(skip_serializing_if = "Option::is_none")]
    verify_advisory: Option<String>,
}

fn recovered_json(rc: &RecoveredCard) -> RecoveredJson {
    match rc {
        RecoveredCard::Mk1 { card, mk1, xpub } => RecoveredJson {
            kind: "mk1",
            mstring: mk1.clone(),
            xpub: Some(xpub.clone()),
            // Surface the rebuilt card's bound-policy stub count (mirrors
            // `inspect`'s mk1 summary) so the human report shows how many
            // policies this xpub serves.
            policy_id_stub_count: Some(card.policy_id_stubs.len()),
            descriptor: None,
            array_index: None,
            reconstructed: None,
            verify_advisory: None,
        },
        RecoveredCard::Md1 { descriptor, md1 } => RecoveredJson {
            kind: "md1",
            mstring: md1.clone(),
            xpub: None,
            policy_id_stub_count: None,
            // The decoded canonical descriptor as a debug-rendered identity (the
            // human-readable descriptor text). md_codec exposes the structured
            // `Descriptor`; we surface the re-emitted md1 string as the identity
            // and a short tag for the human report.
            descriptor: Some(format!("{:?}", descriptor.tree.tag)),
            array_index: None,
            reconstructed: None,
            verify_advisory: None,
        },
    }
}

#[derive(serde::Serialize)]
struct DecodeReportJson {
    source_kind: &'static str,
    truncated: bool,
    erasures_filled: usize,
    recovered: RecoveredJson,
}

#[derive(serde::Serialize)]
struct DecodeEnvelope<'a> {
    schema_version: &'static str,
    mode: &'static str,
    #[serde(flatten)]
    body: &'a DecodeReportJson,
}

fn emit_decode_json<W: Write>(
    report: &DecodeReportJson,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    let envelope = DecodeEnvelope {
        schema_version: WORD_CARD_SCHEMA_VERSION,
        mode: "decode",
        body: report,
    };
    let s = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("word-card JSON serialize: {e}")))?;
    writeln!(stdout, "{s}").map_err(ToolkitError::Io)?;
    Ok(())
}

fn emit_decode_text<W: Write>(
    report: &DecodeReportJson,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    writeln!(stdout, "source_kind: {}", report.source_kind).map_err(ToolkitError::Io)?;
    writeln!(stdout, "truncated: {}", report.truncated).map_err(ToolkitError::Io)?;
    writeln!(stdout, "erasures_filled: {}", report.erasures_filled).map_err(ToolkitError::Io)?;
    if let Some(xpub) = &report.recovered.xpub {
        writeln!(stdout, "xpub: {xpub}").map_err(ToolkitError::Io)?;
    }
    if let Some(c) = report.recovered.policy_id_stub_count {
        writeln!(stdout, "policy_id_stub_count: {c}").map_err(ToolkitError::Io)?;
    }
    if let Some(desc) = &report.recovered.descriptor {
        writeln!(stdout, "descriptor_tag: {desc}").map_err(ToolkitError::Io)?;
    }
    for s in &report.recovered.mstring {
        writeln!(stdout, "{}: {}", report.recovered.kind, s).map_err(ToolkitError::Io)?;
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct RaidDecodeReportJson {
    n: usize,
    reconstructed: Vec<usize>,
    plates: Vec<RecoveredJson>,
}

#[derive(serde::Serialize)]
struct RaidDecodeEnvelope<'a> {
    schema_version: &'static str,
    mode: &'static str,
    #[serde(flatten)]
    body: &'a RaidDecodeReportJson,
}

// ===========================================================================
// Small mappers.
// ===========================================================================

fn source_kind_str(k: SourceKind) -> &'static str {
    match k {
        SourceKind::Mk1Xpub => "mk1",
        SourceKind::Md1Descriptor => "md1",
    }
}

fn plate_role_str(role: wc_codec::PlateRole) -> &'static str {
    match role {
        wc_codec::PlateRole::Data => "data",
        wc_codec::PlateRole::ParityA => "recovery-a",
        wc_codec::PlateRole::ParityB => "recovery-b",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_symbol_count_matches_ceil_formula() {
        // K = ceil((payload_bits + t) / 11). The canonical mk1 numbers from the
        // plan §4.1: payload 73 B = 584 bits, t = 44 ⇒ K = ceil(628/11) = 58.
        assert_eq!(data_symbol_count(584, 44), 58);
        // md1 bit-precise example: 100 bits + 44 = 144 ⇒ ceil(144/11) = 14.
        assert_eq!(data_symbol_count(100, 44), 14);
    }

    #[test]
    fn parity_pct_is_ceil_of_k_times_pct() {
        let args = WordCardArgs {
            from: vec![],
            decode: false,
            decode_plate: vec![],
            parity_words: None,
            parity_pct: Some(25),
            raid: 0,
            integrity_bits: 44,
            json: false,
            words: vec![],
        };
        // K = 58, 25% ⇒ ceil(58 * 25 / 100) = ceil(14.5) = 15.
        assert_eq!(resolve_parity_words(&args, 58), 15);
    }

    #[test]
    fn parity_words_takes_precedence_when_set() {
        let args = WordCardArgs {
            from: vec![],
            decode: false,
            decode_plate: vec![],
            parity_words: Some(12),
            parity_pct: None,
            raid: 0,
            integrity_bits: 44,
            json: false,
            words: vec![],
        };
        assert_eq!(resolve_parity_words(&args, 58), 12);
    }

    #[test]
    fn split_words_splits_on_any_whitespace() {
        assert_eq!(
            split_words("abandon  ability\nable\tabout"),
            vec!["abandon", "ability", "able", "about"]
        );
    }

    #[test]
    fn source_kind_strings() {
        assert_eq!(source_kind_str(SourceKind::Mk1Xpub), "mk1");
        assert_eq!(source_kind_str(SourceKind::Md1Descriptor), "md1");
    }
}
