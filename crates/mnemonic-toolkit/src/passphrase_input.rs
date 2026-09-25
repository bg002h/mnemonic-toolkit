//! F-687 — **the one rule for the value of `--passphrase`**, on every
//! subcommand that declares it.
//!
//! | value                    | the passphrase is                          | stderr note |
//! |--------------------------|--------------------------------------------|-------------|
//! | absent                   | the empty string (BIP-39 default)          | none        |
//! | `--passphrase-stdin`     | stdin, minus ONE trailing `\n` / `\r\n`    | none        |
//! | `-`                      | stdin, byte-identical to `--passphrase-stdin` | none     |
//! | `@env:VAR`               | `$VAR` minus ONE trailing `\n` / `\r\n`; unset → error naming `VAR`; set-but-empty → the empty passphrase | none |
//! | anything else            | that literal string, verbatim              | one line    |
//!
//! **One byte rule for the three private forms** ([`strip_one_newline`]).
//! Before F-687, `@env:VAR` was verbatim while stdin stripped a newline, so
//! `PP=$'TREZOR\n'` derived `48efb44f` where the same passphrase on stdin
//! derived `b4e3f5ed`.
//!
//! Before F-687 the rule was re-implemented at each site, and the copies
//! disagreed: `-` was the literal one-character passphrase everywhere, and
//! `@env:VAR` was literal on `silent-payment` while resolving elsewhere — a
//! different wallet at exit 0. Operator ruling 2026-09-25: `-` is stdin and
//! `@env:VAR` is the environment, everywhere; nothing is refused; a literal
//! argv passphrase keeps working and gets one stderr note.
//!
//! **A literal `-` passphrase is no longer expressible on argv.** It still is
//! through `--passphrase-stdin` / `--passphrase -` (pipe a lone `-`) or
//! `@env:VAR`. The same holds for a literal passphrase beginning `@env:`.
//!
//! `--passphrase -` together with `--passphrase-stdin` is refused by clap
//! (`conflicts_with`, exit 64) on every site; [`resolve`] refuses it too, as a
//! backstop, so no caller can ever read stdin twice.

use std::io::{Read, Write};

use zeroize::Zeroizing;

use crate::error::ToolkitError;

/// Where the value of `--passphrase` comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PassphraseSource {
    /// Neither `--passphrase` nor `--passphrase-stdin`.
    Absent,
    /// `--passphrase-stdin`, or `--passphrase -`.
    Stdin,
    /// `--passphrase @env:VAR`.
    Env,
    /// Any other `--passphrase` value: the passphrase itself, on argv.
    Argv,
}

/// Classify. Pure: reads nothing, resolves nothing.
pub(crate) fn source(value: Option<&str>, stdin_flag: bool) -> PassphraseSource {
    if stdin_flag {
        return PassphraseSource::Stdin;
    }
    match value {
        None => PassphraseSource::Absent,
        Some("-") => PassphraseSource::Stdin,
        Some(v) if v.starts_with("@env:") => PassphraseSource::Env,
        Some(_) => PassphraseSource::Argv,
    }
}

/// Does the passphrase consume stdin? Every single-stdin guard asks THIS,
/// never `args.passphrase_stdin` alone — otherwise `--passphrase -` beside
/// `--from phrase=-` would read stdin twice.
pub(crate) fn reads_stdin(value: Option<&str>, stdin_flag: bool) -> bool {
    source(value, stdin_flag) == PassphraseSource::Stdin
}

/// How the operator spelled the stdin passphrase channel, for single-stdin
/// refusals: `--passphrase-stdin` or `--passphrase -`. Existing messages keep
/// their `--passphrase-stdin` wording; the `-` form names itself.
pub(crate) fn stdin_spelling(stdin_flag: bool) -> &'static str {
    if stdin_flag {
        "--passphrase-stdin"
    } else {
        "--passphrase -"
    }
}

/// Is this input PATH really stdin? `/dev/stdin`, `/dev/fd/0` and
/// `/proc/self/fd/0` by name, and on Unix anything that resolves to the same
/// file as fd 0 (same device + inode). Opening such a path reads the SAME
/// stream as a stdin passphrase, so it counts as a second stdin reader —
/// before F-687 fold 1, `silent-payment --secret-file /dev/stdin
/// --passphrase -` let the secret read drain stdin and derived with the EMPTY
/// passphrase at exit 0 (review M1).
pub(crate) fn path_is_stdin(path: &std::path::Path) -> bool {
    if matches!(
        path.to_str(),
        Some("/dev/stdin" | "/dev/fd/0" | "/proc/self/fd/0")
    ) {
        return true;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let (Ok(a), Ok(b)) = (std::fs::metadata(path), std::fs::metadata("/dev/stdin")) {
            return a.dev() == b.dev() && a.ino() == b.ino();
        }
    }
    false
}

/// One stdin per invocation: refuse when more than one of `readers` (each a
/// `(reads_stdin, name)` pair) consumes stdin. The second reader would
/// otherwise see an empty stream — for a passphrase, silently the empty
/// passphrase, i.e. a different wallet.
pub(crate) fn refuse_second_stdin(readers: &[(bool, &str)]) -> Result<(), ToolkitError> {
    let names: Vec<&str> = readers
        .iter()
        .filter(|(r, _)| *r)
        .map(|(_, n)| *n)
        .collect();
    if names.len() > 1 {
        return Err(ToolkitError::BadInput(format!(
            "{} all read stdin; only one input can come from stdin per invocation",
            names.join(" and ")
        )));
    }
    Ok(())
}

/// The note printed once for a literal argv passphrase. Never carries the
/// value. Byte-identical to `ms`'s `advisory::PASSPHRASE_ARGV_NOTE`.
pub(crate) const ARGV_NOTE: &str = "warning: secret material on argv (--passphrase) \u{2014} \
read it privately with --passphrase - or --passphrase-stdin (stdin), \
or --passphrase @env:VAR (environment variable)";

/// Emit [`ARGV_NOTE`] iff the passphrase is literal on argv. Best-effort, like
/// every advisory: a closed stderr is not an error.
pub(crate) fn emit_argv_note<E: Write>(value: Option<&str>, stdin_flag: bool, stderr: &mut E) {
    if source(value, stdin_flag) == PassphraseSource::Argv {
        let _ = writeln!(stderr, "{ARGV_NOTE}");
    }
}

/// Remove exactly ONE trailing `\n` (and a `\r` before it), nothing else.
/// The one byte rule for every private passphrase channel — stdin
/// (`--passphrase-stdin`, `--passphrase -`) and `@env:VAR` — because shells,
/// editors and `echo` add that newline and it is never meant. A second
/// newline, interior newlines and all other whitespace are kept: a passphrase
/// differing by one byte is a different wallet.
pub(crate) fn strip_one_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

/// Resolve `--passphrase @env:VAR`: the variable's value under the stdin
/// byte rule ([`strip_one_newline`]). Unset / invalid name → the toolkit's
/// `EnvVarMissing` error naming `VAR`. Every subcommand's `@env:` pre-pass
/// calls THIS for `--passphrase` — never `resolve_env_var_sentinel` directly,
/// which is verbatim and serves the other secret flags.
pub(crate) fn resolve_env(value: &str) -> Result<String, ToolkitError> {
    let mut v = crate::env_sentinel::resolve_env_var_sentinel(value, "--passphrase")?;
    strip_one_newline(&mut v);
    Ok(v)
}

/// Read a passphrase from stdin, preserving every byte except ONE trailing
/// `\n` (and a `\r` before it). This is the byte rule `--passphrase-stdin` has
/// always had; `--passphrase -` shares it by calling the same function.
pub(crate) fn read_stdin_passphrase<R: Read + ?Sized>(
    stdin: &mut R,
) -> Result<String, ToolkitError> {
    // wave2 T4: scrub the read_to_string SCRATCH buffer (passphrase material).
    let mut buf = Zeroizing::new(String::new());
    stdin
        .read_to_string(&mut buf)
        .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
    strip_one_newline(&mut buf);
    Ok(buf.to_string())
}

/// A `--passphrase` handed to a shared helper: either still AS WRITTEN (the
/// helper applies the rule and emits the note), or already RESOLVED by a
/// caller that substituted stdin / expanded `@env:` and emitted the note
/// itself. The resolved form must never go through [`resolve`] again: its
/// value would be re-read as written — `-` as stdin, a second argv note —
/// and, before F-687, `verify-bundle`'s template path re-read an
/// already-drained stdin and derived with the EMPTY passphrase (exit 4, a
/// false NO MATCH, on a matching bundle).
#[derive(Debug, Clone, Copy)]
pub(crate) enum PassphraseArg<'a> {
    AsWritten {
        value: Option<&'a str>,
        stdin_flag: bool,
    },
    Resolved {
        value: Option<&'a str>,
        /// The resolution consumed stdin (for the single-stdin guard).
        consumed_stdin: bool,
    },
}

impl PassphraseArg<'_> {
    /// Does (or did) this passphrase consume stdin?
    pub(crate) fn reads_stdin(&self) -> bool {
        match *self {
            PassphraseArg::AsWritten { value, stdin_flag } => reads_stdin(value, stdin_flag),
            PassphraseArg::Resolved { consumed_stdin, .. } => consumed_stdin,
        }
    }

    /// The spelling to name in a single-stdin refusal.
    pub(crate) fn stdin_spelling(&self) -> &'static str {
        match *self {
            PassphraseArg::AsWritten { stdin_flag, .. } => stdin_spelling(stdin_flag),
            PassphraseArg::Resolved { .. } => "--passphrase-stdin / --passphrase -",
        }
    }

    /// Emit the argv note — only for an AS-WRITTEN literal.
    pub(crate) fn emit_argv_note<E: Write>(&self, stderr: &mut E) {
        if let PassphraseArg::AsWritten { value, stdin_flag } = *self {
            emit_argv_note(value, stdin_flag, stderr);
        }
    }

    /// The passphrase (absent = empty).
    pub(crate) fn resolve_or_empty<R: Read + ?Sized>(
        &self,
        stdin: &mut R,
    ) -> Result<Zeroizing<String>, ToolkitError> {
        match *self {
            PassphraseArg::AsWritten { value, stdin_flag } => {
                resolve_or_empty(value, stdin_flag, stdin)
            }
            PassphraseArg::Resolved { value, .. } => {
                Ok(Zeroizing::new(value.unwrap_or("").to_string()))
            }
        }
    }
}

/// Resolve `--passphrase` / `--passphrase-stdin` to the passphrase, or `None`
/// when neither was given. Does NOT emit the argv note — callers emit it with
/// [`emit_argv_note`] at the point their other advisories go out, so the
/// stderr order of each subcommand is unchanged.
pub(crate) fn resolve<R: Read + ?Sized>(
    value: Option<&str>,
    stdin_flag: bool,
    stdin: &mut R,
) -> Result<Option<Zeroizing<String>>, ToolkitError> {
    if stdin_flag && value.is_some() {
        return Err(ToolkitError::BadInput(
            "--passphrase and --passphrase-stdin cannot both be given (one passphrase, one stdin)"
                .into(),
        ));
    }
    Ok(match source(value, stdin_flag) {
        PassphraseSource::Absent => None,
        PassphraseSource::Stdin => Some(Zeroizing::new(read_stdin_passphrase(stdin)?)),
        PassphraseSource::Env => Some(Zeroizing::new(resolve_env(value.unwrap_or(""))?)),
        PassphraseSource::Argv => Some(Zeroizing::new(value.unwrap_or("").to_string())),
    })
}

/// [`resolve`], with absent meaning the empty passphrase.
pub(crate) fn resolve_or_empty<R: Read + ?Sized>(
    value: Option<&str>,
    stdin_flag: bool,
    stdin: &mut R,
) -> Result<Zeroizing<String>, ToolkitError> {
    Ok(resolve(value, stdin_flag, stdin)?.unwrap_or_else(|| Zeroizing::new(String::new())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(value: Option<&str>, stdin_flag: bool, input: &[u8]) -> Result<Option<String>, String> {
        let mut cur = std::io::Cursor::new(input.to_vec());
        resolve(value, stdin_flag, &mut cur)
            .map(|o| o.map(|z| z.to_string()))
            .map_err(|e| format!("{e:?}"))
    }

    #[test]
    fn dash_is_stdin_and_matches_the_stdin_flag_byte_for_byte() {
        for input in [
            &b"TREZOR"[..],
            b"TREZOR\n",
            b"TREZOR\r\n",
            b"TREZOR\n\n",
            b" pw \n",
            b"",
            b"-",
        ] {
            assert_eq!(
                r(Some("-"), false, input),
                r(None, true, input),
                "{input:?}"
            );
        }
        assert_eq!(r(Some("-"), false, b"TREZOR\n"), Ok(Some("TREZOR".into())));
        // Only ONE newline is stripped: a second is part of the passphrase.
        assert_eq!(
            r(Some("-"), false, b"TREZOR\n\n"),
            Ok(Some("TREZOR\n".into()))
        );
    }

    #[test]
    fn env_resolves_and_unset_names_the_variable() {
        let var = "MNEMONIC_F687_UNIT_SET";
        std::env::set_var(var, "TREZOR");
        assert_eq!(
            r(Some(&format!("@env:{var}")), false, b"ignored"),
            Ok(Some("TREZOR".into()))
        );
        std::env::remove_var(var);
        let err = r(Some("@env:MNEMONIC_F687_UNIT_NEVER_SET"), false, b"").unwrap_err();
        assert!(err.contains("MNEMONIC_F687_UNIT_NEVER_SET"), "{err}");
    }

    #[test]
    fn env_strips_exactly_one_trailing_newline_like_stdin() {
        let var = "MNEMONIC_F687_UNIT_NL";
        for (val, want) in [
            ("TREZOR\n", "TREZOR"),
            ("TREZOR\r\n", "TREZOR"),
            ("TREZOR\n\n", "TREZOR\n"),
            ("TRE\nZOR", "TRE\nZOR"),
            (" TREZOR ", " TREZOR "),
        ] {
            std::env::set_var(var, val);
            let env = r(Some(&format!("@env:{var}")), false, b"");
            let stdin = r(Some("-"), false, val.as_bytes());
            assert_eq!(env, Ok(Some(want.to_string())), "{val:?}");
            assert_eq!(env, stdin, "{val:?}");
        }
        std::env::remove_var(var);
    }

    #[test]
    fn env_set_but_empty_is_the_empty_passphrase() {
        let var = "MNEMONIC_F687_UNIT_EMPTY";
        std::env::set_var(var, "");
        assert_eq!(
            r(Some(&format!("@env:{var}")), false, b""),
            Ok(Some(String::new()))
        );
        std::env::remove_var(var);
    }

    #[test]
    fn a_literal_is_itself_and_only_it_gets_the_note() {
        assert_eq!(r(Some("TREZOR"), false, b"x"), Ok(Some("TREZOR".into())));
        assert_eq!(r(None, false, b"x"), Ok(None));
        for (v, flag, noted) in [
            (Some("TREZOR"), false, true),
            (Some("-x"), false, true),
            (Some(""), false, true),
            (Some("-"), false, false),
            (Some("@env:PP"), false, false),
            (None, true, false),
            (None, false, false),
        ] {
            let mut e = Vec::new();
            emit_argv_note(v, flag, &mut e);
            let s = String::from_utf8(e).unwrap();
            assert_eq!(s.lines().count(), usize::from(noted), "{v:?} {flag}");
            if let Some(v) = v.filter(|v| !v.is_empty()) {
                assert!(
                    !s.contains(v) || v == "-",
                    "the note must not echo the value"
                );
            }
        }
    }

    #[test]
    fn both_stdin_forms_together_are_refused() {
        assert!(r(Some("-"), true, b"TREZOR").is_err());
        assert!(reads_stdin(Some("-"), false));
        assert!(reads_stdin(None, true));
        assert!(!reads_stdin(Some("@env:PP"), false));
        assert!(!reads_stdin(Some("TREZOR"), false));
    }
}
