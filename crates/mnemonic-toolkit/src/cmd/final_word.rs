//! `mnemonic final-word` subcommand — BIP-39 last-word completer.
//!
//! Realizes `design/SPEC_final_word_v0_11_0.md` §2.2. Wraps the library
//! at `mnemonic_toolkit::final_word` with Cycle A/B secret-memory
//! discipline:
//!   - argv-leakage advisory for inline `--from phrase=<value>`
//!   - `Zeroizing<String>` for the parsed partial
//!   - mlock pin on the parsed partial bytes (Cycle B Phase 3a Site 1
//!     pattern; cite `cmd/derive_child.rs:124-131`)
//!   - SPEC §2.6 stdout-on-TTY + world-readable `--json-out` advisories

use crate::cmd::convert::{parse_from_input, read_stdin_to_string, FromInput, NodeType};
use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::secret_advisory::secret_in_argv_warning;
use clap::Args;
use mnemonic_toolkit::final_word::{
    final_word_candidates, FinalWordError, FinalWordLanguage,
};
use std::io::{IsTerminal, Read, Write};

#[derive(Args, Debug)]
pub struct FinalWordArgs {
    /// Partial phrase as `phrase=<n-1 words>` (inline) or `phrase=-`
    /// (read from stdin). The partial must have 11, 14, 17, 20, or 23
    /// words; the target N = K + 1 ∈ {12, 15, 18, 21, 24}.
    ///
    /// Inline form emits an argv-leakage advisory (`/proc/$PID/cmdline`
    /// exposure); prefer `phrase=-` for sensitive input.
    #[arg(
        long = "from",
        value_name = "phrase=<value-or-->",
        value_parser = parse_from_input,
        required = true,
    )]
    pub from: FromInput,

    /// BIP-39 language. Defaults to english.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Write a versioned JSON envelope to this path (side-effect; does
    /// NOT replace stdout). The plain candidate list is still emitted to
    /// stdout. On Unix the resulting file inherits the process umask;
    /// a world-readable result raises a SPEC §2.6 stderr advisory.
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &FinalWordArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    if args.from.node != NodeType::Phrase {
        return Err(ToolkitError::BadInput(format!(
            "final-word --from only accepts phrase=<value> or phrase=-; got {}=",
            args.from.node.as_str(),
        )));
    }

    // Cycle A — argv-leakage advisory for inline secret.
    if args.from.value != "-" {
        secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-");
    }

    // Cycle A — wrap the OWNED partial in Zeroizing before any further work.
    let partial: zeroize::Zeroizing<String> = if args.from.value == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(args.from.value.clone())
    };

    // Cycle B Phase 3a Site 1 — pin the partial's heap pages.
    let _pin_partial = mnemonic_toolkit::mlock::pin_pages_for(partial.as_bytes());

    let language = map_language(args.language);

    let candidates = final_word_candidates(partial.as_str(), language)
        .map_err(map_final_word_error)?;

    // Plain stdout: one candidate per line, sorted (library guarantees sort).
    for word in &candidates {
        writeln!(stdout, "{word}")
            .map_err(|e| ToolkitError::BadInput(format!("stdout write: {e}")))?;
    }

    if let Some(path) = &args.json_out {
        write_json_envelope(
            path,
            args.language.human_name(),
            &candidates,
            partial_word_count(&partial),
            stderr,
        )?;
    }

    // SPEC §2.6 — stdout-on-TTY advisory. Fires only when stdout is a
    // real terminal and we actually emitted candidate words.
    if !candidates.is_empty() && std::io::stdout().is_terminal() {
        let _ = writeln!(
            stderr,
            "warning: candidate list is secret material — pairing the partial phrase with any candidate yields a valid seed phrase; do not paste this output into untrusted tools",
        );
    }

    Ok(0)
}

fn partial_word_count(partial: &str) -> usize {
    partial.split_whitespace().count()
}

fn map_language(l: CliLanguage) -> FinalWordLanguage {
    match l {
        CliLanguage::English => FinalWordLanguage::English,
        CliLanguage::SimplifiedChinese => FinalWordLanguage::SimplifiedChinese,
        CliLanguage::TraditionalChinese => FinalWordLanguage::TraditionalChinese,
        CliLanguage::Czech => FinalWordLanguage::Czech,
        CliLanguage::French => FinalWordLanguage::French,
        CliLanguage::Italian => FinalWordLanguage::Italian,
        CliLanguage::Japanese => FinalWordLanguage::Japanese,
        CliLanguage::Korean => FinalWordLanguage::Korean,
        CliLanguage::Portuguese => FinalWordLanguage::Portuguese,
        CliLanguage::Spanish => FinalWordLanguage::Spanish,
    }
}

fn map_final_word_error(e: FinalWordError) -> ToolkitError {
    match e {
        FinalWordError::BadWordCount(0) => ToolkitError::BadInput(
            "final-word: empty partial phrase; need 11/14/17/20/23 words \
             for a target of 12/15/18/21/24"
                .into(),
        ),
        FinalWordError::BadWordCount(got) => ToolkitError::BadInput(format!(
            "final-word: got {got} words; expected one of [11, 14, 17, 20, 23] \
             (target = K+1 must be in {{12,15,18,21,24}})",
        )),
        FinalWordError::UnknownWord { position } => ToolkitError::BadInput(format!(
            "final-word: unknown BIP-39 word at position {position} \
             (not in selected wordlist; did you pick the right --language?)",
        )),
    }
}

#[derive(serde::Serialize)]
struct FinalWordJson<'a> {
    schema_version: &'static str,
    language: &'static str,
    partial_word_count: usize,
    target_word_count: usize,
    candidate_count: usize,
    candidates: &'a [&'static str],
}

fn write_json_envelope<E: Write>(
    path: &std::path::Path,
    language_name: &'static str,
    candidates: &[&'static str],
    partial_word_count: usize,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let envelope = FinalWordJson {
        schema_version: "1",
        language: language_name,
        partial_word_count,
        target_word_count: partial_word_count + 1,
        candidate_count: candidates.len(),
        candidates,
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("--json-out serialize: {e}")))?;
    std::fs::write(path, &body)
        .map_err(|e| ToolkitError::BadInput(format!("--json-out write {}: {e}", path.display())))?;

    // SPEC §2.6 row 3 — world-readable permission-mode advisory.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mode = meta.permissions().mode();
            if mode & 0o077 != 0 {
                let _ = writeln!(
                    stderr,
                    "warning: --json-out {} inherits umask (file may be world-readable, mode {:o}); consider --json-out /dev/stdout or chmod 0600 the path before invoking",
                    path.display(),
                    mode & 0o777,
                );
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (stderr, path); // suppress unused warnings on non-Unix
    }

    Ok(())
}
