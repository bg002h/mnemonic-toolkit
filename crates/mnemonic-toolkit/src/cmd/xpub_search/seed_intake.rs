//! Seed intake helper for `xpub-search` modes (P1/P2/P4).
//!
//! Polymorphic seed shape parser:
//!   - `--phrase <bip39>`
//!   - `--phrase-stdin`
//!   - `--ms1 <bech32>`
//!   - `--ms1-stdin`
//!   - positional `<STRING>` (ms1 HRP only; BIP-39 phrase positional is
//!     refused with a clear error)
//!
//! Auto-fire BCH repair applies ONLY to the `--ms1` decode-failure path,
//! TTY-gated via `crate::repair::resolve_no_auto_repair`. `--phrase` BIP-39
//! parse failure routes directly to exit 1 (no BCH primitive for plain text).

use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::repair::{self, CardKind};
use crate::secret_advisory::secret_in_argv_warning;
use bip39::Mnemonic;
use std::io::{Read, Write};
use zeroize::Zeroizing;

/// Read-only accessor over a `*Args` struct's seed-intake fields. Each
/// per-mode struct (P1: `PathOfXpubArgs`, future P2/P4: `*Args`) implements
/// this trait so `resolve_seed` works uniformly across modes.
pub trait SeedIntakeArgs {
    fn phrase(&self) -> Option<&str>;
    fn phrase_stdin(&self) -> bool;
    fn ms1(&self) -> Option<&str>;
    fn ms1_stdin(&self) -> bool;
    fn positional(&self) -> &[String];
    fn language(&self) -> CliLanguage;
}

/// Resolve the seed-intake mutex + parse the chosen value into a
/// `bip39::Mnemonic`. Emits argv-leak advisory when inline `--phrase` /
/// `--ms1` are used. On `--ms1` decode failure, route through the BCH
/// auto-fire short-circuit (TTY-gated).
pub fn resolve_seed<A, R, E>(
    args: &A,
    stdin: &mut R,
    stdout: &mut impl Write,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<Mnemonic, ToolkitError>
where
    A: SeedIntakeArgs,
    R: Read,
    E: Write,
{
    // 1) Mutex check across (phrase / phrase-stdin / ms1 / ms1-stdin /
    //    positional). Exactly one must be present. clap's
    //    `conflicts_with_all` + `required_unless_present_any` covers most of
    //    this at clap-parse time, but we double-check here for defense-in-
    //    depth (positional + flag mixed-form would slip past clap).
    let modes_active = [
        args.phrase().is_some(),
        args.phrase_stdin(),
        args.ms1().is_some(),
        args.ms1_stdin(),
        !args.positional().is_empty(),
    ];
    let n_active = modes_active.iter().filter(|b| **b).count();
    if n_active == 0 {
        return Err(ToolkitError::BadInput(
            "supply one of --phrase / --phrase-stdin / --ms1 / --ms1-stdin / <positional ms1 card>"
                .into(),
        ));
    }
    if n_active > 1 {
        return Err(ToolkitError::BadInput(
            "exactly one seed-intake mode allowed: --phrase / --phrase-stdin / --ms1 / --ms1-stdin / <positional ms1 card>"
                .into(),
        ));
    }

    // 2) Resolve to (kind, value). Source values carry `Zeroizing<String>`
    //    so the phrase/ms1 heap buffer scrubs on drop (plan §3.6 secret
    //    hygiene; mirrors `derive_child.rs:127-131` precedent).
    enum Source {
        Phrase(Zeroizing<String>),
        Ms1(Zeroizing<String>),
    }

    let source: Source = if let Some(p) = args.phrase() {
        secret_in_argv_warning(stderr, "--phrase", "--phrase-stdin");
        Source::Phrase(Zeroizing::new(normalize_phrase(p)))
    } else if args.phrase_stdin() {
        let mut buf: Zeroizing<String> = Zeroizing::new(String::new());
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
        Source::Phrase(Zeroizing::new(normalize_phrase(&buf)))
    } else if let Some(m) = args.ms1() {
        secret_in_argv_warning(stderr, "--ms1", "--ms1-stdin");
        // HRP validation (strict; v0.24.0 D34 pattern).
        crate::repair::validate_flag_hrp("--ms1", "ms", m)?;
        Source::Ms1(Zeroizing::new(m.to_string()))
    } else if args.ms1_stdin() {
        let mut buf: Zeroizing<String> = Zeroizing::new(String::new());
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
        let trimmed: Zeroizing<String> = Zeroizing::new(buf.trim().to_string());
        crate::repair::validate_flag_hrp("--ms1-stdin", "ms", &trimmed)?;
        Source::Ms1(trimmed)
    } else {
        // Positional intake — exactly one element (clap shape enforces this
        // via the count check above; we accept >= 1 but reject BIP-39).
        let positional = args.positional();
        if positional.len() > 1 {
            return Err(ToolkitError::BadInput(format!(
                "xpub-search accepts a single positional ms1 card; got {} arguments",
                positional.len()
            )));
        }
        let v = &positional[0];
        // HRP-autodetect: only ms1 is accepted positionally. BIP-39 phrase
        // text has no HRP and is rejected with a clear pointer to --phrase.
        match repair::classify_hrp_prefix(v) {
            Ok(CardKind::Ms1) => Source::Ms1(Zeroizing::new(v.clone())),
            Ok(CardKind::Mk1) | Ok(CardKind::Md1) => {
                return Err(ToolkitError::BadInput(format!(
                    "xpub-search seed-intake positional requires an ms1 card; got HRP for `{}` \
                     (only ms1 carries BIP-39 entropy)",
                    classify_hrp_str(v)
                )));
            }
            Err(_) => {
                return Err(ToolkitError::BadInput(
                    "positional argument is neither an ms1 card nor a recognized HRP prefix; \
                     BIP-39 phrase positional is not supported (no HRP for autodetect) — \
                     pass the phrase via --phrase or --phrase-stdin"
                        .into(),
                ));
            }
        }
    };

    // 3) Pin the source heap-buffer for the remainder of resolve_seed scope
    //    (mirrors `derive_child.rs:157-160` precedent). The owned-String
    //    inside Zeroizing keeps its heap-data pointer stable across this
    //    function's lifetime; the pin captured here covers the parse step.
    let _source_pin = match &source {
        Source::Phrase(p) => mnemonic_toolkit::mlock::pin_pages_for(p.as_bytes()),
        Source::Ms1(m) => mnemonic_toolkit::mlock::pin_pages_for(m.as_bytes()),
    };

    // 4) Parse the source into a Mnemonic.
    match source {
        Source::Phrase(p) => {
            // Mnemonic::parse_in handles BIP-39 word-validation. No
            // auto-fire applies — there's no BCH primitive for plain text.
            Mnemonic::parse_in(args.language().into(), p.as_str()).map_err(ToolkitError::Bip39)
        }
        Source::Ms1(card) => {
            match ms_codec::decode(card.as_str()) {
                Ok((_tag, payload)) => {
                    // Entropy bytes wrapped in Zeroizing to scrub on drop at
                    // function exit. Mirrors `derive_slot.rs:77-84` precedent.
                    let entropy: Zeroizing<Vec<u8>> = Zeroizing::new(payload.as_bytes().to_vec());
                    let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
                    Mnemonic::from_entropy_in(args.language().into(), &entropy[..])
                        .map_err(ToolkitError::Bip39)
                }
                Err(decode_err) => {
                    // Auto-fire BCH repair on decode failure (TTY-gated).
                    let effective_no_auto_repair =
                        crate::repair::resolve_no_auto_repair(no_auto_repair);
                    if !effective_no_auto_repair {
                        // Single-chunk repair attempt; emits exit-5 short-circuit
                        // via Err(RepairShortCircuit) when correction succeeds.
                        crate::repair::try_repair_and_short_circuit(
                            CardKind::Ms1,
                            &[card.as_str().to_string()],
                            stdout,
                            stderr,
                            false, // text-form report (caller's --json is mode-level)
                        )?;
                    }
                    Err(ToolkitError::from(decode_err))
                }
            }
        }
    }
}

fn normalize_phrase(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn classify_hrp_str(s: &str) -> &'static str {
    if s.starts_with("ms1") {
        "ms"
    } else if s.starts_with("mk1") {
        "mk"
    } else if s.starts_with("md1") {
        "md"
    } else {
        "<unknown>"
    }
}
