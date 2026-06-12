//! `--passphrase-candidates-file` candidate-list scan engine for
//! `xpub-search passphrase-of-xpub` (FOLLOWUP `xpub-search-passphrase-bruteforce`,
//! file-only scope). Streams a text file (one candidate per line), loops the
//! existing `derive_master_seed` → `match_xpub_against_paths` oracle, aborts on
//! first match, and reports the matching FILE LINE (the passphrase only in
//! `--json`). See design/SPEC_xpub_search_passphrase_candidates_file.md.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use bip39::Mnemonic;
use bitcoin::bip32::Xpriv;
use zeroize::Zeroizing;

use super::candidate_paths::build_candidate_paths;
use super::passphrase_of_xpub::{PassphraseOfXpubArgs, PassphraseOfXpubResult};
use super::path_search::match_xpub_against_paths;
use super::target_intake::resolve_target_xpub;
use super::{XpubSearchEnvelope, XpubSearchJson};
use crate::derive_slot::derive_master_seed;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::synthesize::xpub_to_65;

/// Run the candidate-file scan. The `mnemonic` is already resolved once by the
/// caller; only the per-candidate `derive_master_seed` + match re-runs.
pub(super) fn run_candidate_scan<W: Write, E: Write>(
    args: &PassphraseOfXpubArgs,
    mnemonic: &Mnemonic,
    path: &Path,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // (impl-review I1 / SPEC §2) One-line runtime sensitivity advisory — the
    // compensating control for classifying `--passphrase-candidates-file` as a
    // non-secret PATH flag. Non-fatal if stderr is unreachable.
    let _ = writeln!(
        stderr,
        "note: {} holds candidate passphrases — treat as sensitive",
        path.display()
    );

    // Resolve target xpub + the per-passphrase candidate-path window (same as
    // the single-passphrase path).
    let (target_xpub, target_variant) = resolve_target_xpub(&args.target_xpub)?;
    let target_xpub_65 = xpub_to_65(&target_xpub);
    let target_xpub_canonical = target_xpub.to_string();
    let candidates = build_candidate_paths(
        args.min_account,
        args.number_of_accounts,
        args.max_account,
        &args.add_path,
        args.network,
    );
    let searched_count = candidates.len();

    let file = std::fs::File::open(path).map_err(|e| {
        ToolkitError::BadInput(format!(
            "--passphrase-candidates-file {}: {e}",
            path.display()
        ))
    })?;
    let reader = BufReader::new(file);

    let mut candidates_tried: usize = 0usize;
    for (idx, line) in reader.lines().enumerate() {
        // `BufRead::lines` drops `\n`; strip a trailing `\r` for CRLF. NO other
        // trim — a passphrase is an exact byte string.
        let mut raw = line.map_err(|e| {
            ToolkitError::BadInput(format!("--passphrase-candidates-file read: {e}"))
        })?;
        if raw.ends_with('\r') {
            raw.pop();
        }
        if raw.is_empty() {
            continue; // blank-line skip
        }
        let line_no = idx + 1; // 1-indexed FILE line
        let candidate: Zeroizing<String> = Zeroizing::new(raw); // (I-r3) owned secret
        candidates_tried += 1;

        let seed = derive_master_seed(mnemonic, candidate.as_str()); // Zeroizing<[u8;64]>
        let master_xprv = Xpriv::new_master(args.network.network_kind(), &seed[..])
            .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
        if let Some(m) = match_xpub_against_paths(&master_xprv, &candidates, &target_xpub_65) {
            return emit_match(
                args,
                stdout,
                &m,
                line_no,
                &candidate,
                target_xpub_canonical,
                target_variant,
                searched_count,
            );
        }
    }

    // Exhausted: no candidate matched.
    if args.json {
        let envelope = XpubSearchEnvelope {
            schema_version: "1",
            body: XpubSearchJson::PassphraseOfXpub(PassphraseOfXpubResult::NoMatch {
                target_xpub_canonical,
                target_xpub_variant: target_variant,
                searched_count,
                candidates_tried: Some(candidates_tried),
            }),
        };
        let body = serde_json::to_string(&envelope)
            .map_err(|e| ToolkitError::BadInput(format!("passphrase scan JSON serialize: {e}")))?;
        writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
    }
    Err(ToolkitError::XpubSearchPassphraseCandidatesExhausted { candidates_tried })
}

#[allow(clippy::too_many_arguments)]
fn emit_match<W: Write>(
    args: &PassphraseOfXpubArgs,
    stdout: &mut W,
    m: &super::path_search::MatchedPath,
    line_no: usize,
    candidate: &Zeroizing<String>,
    target_xpub_canonical: String,
    target_variant: Option<&'static str>,
    searched_count: usize,
) -> Result<u8, ToolkitError> {
    if args.json {
        // `--json` opt-in: the matching passphrase IS emitted (machine use).
        let envelope = XpubSearchEnvelope {
            schema_version: "1",
            body: XpubSearchJson::PassphraseOfXpub(PassphraseOfXpubResult::Match {
                path: format!("m/{}", m.path),
                template: m.template_name.clone(),
                account: m.account,
                target_xpub_canonical,
                target_xpub_variant: target_variant,
                searched_count,
                matched_candidate_line: Some(line_no),
                matched_passphrase: Some(candidate.as_str().to_string()),
            }),
        };
        let body = serde_json::to_string(&envelope)
            .map_err(|e| ToolkitError::BadInput(format!("passphrase scan JSON serialize: {e}")))?;
        writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
    } else {
        // Default text: report the FILE LINE, NOT the passphrase (the secret is
        // already in the user's file; don't echo it to stdout/scrollback).
        writeln!(
            stdout,
            "match: candidate on line {line_no} derives the target xpub at m/{} \
             (template={}, account={})",
            m.path,
            m.template_name,
            m.account
                .map(|a| a.to_string())
                .unwrap_or_else(|| "n/a".to_string()),
        )
        .map_err(ToolkitError::Io)?;
        writeln!(
            stdout,
            "target-xpub: {}{}",
            target_xpub_canonical,
            match target_variant {
                Some(v) => format!(" (normalized from {v}; variant={v})"),
                None => String::new(),
            },
        )
        .map_err(ToolkitError::Io)?;
        writeln!(
            stdout,
            "searched: {searched_count} candidate paths per passphrase"
        )
        .map_err(ToolkitError::Io)?;
    }
    Ok(0)
}
