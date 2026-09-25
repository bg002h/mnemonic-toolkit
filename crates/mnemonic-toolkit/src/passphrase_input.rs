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

//!
//! ## F-687b (operator rulings 2026-09-25, "1) yes 2) yes 3) yes")
//!
//! 1. The SAME rule governs every password- or passphrase-like secret flag,
//!    through this module (never a copy): `--passphrase`,
//!    `--bip38-passphrase` and `--decrypt-password` ([`SecretFlag`]).
//! 2. A private channel (`-`, the `*-stdin` flag, `@env:VAR`) that yields an
//!    EMPTY value prints one stderr warning ([`empty_warning`]) and proceeds.
//! 3. When a stdin secret is read and stdin is a terminal, a prompt goes to
//!    stderr first, echo is switched off where the terminal allows it, and one
//!    line is read (a terminal user presses Enter, not Ctrl-D).

use std::io::{Read, Write};

use zeroize::Zeroizing;

use crate::error::ToolkitError;

/// A password- or passphrase-like secret flag that follows this module's rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SecretFlag {
    /// The value flag, e.g. `--passphrase`.
    pub flag: &'static str,
    /// Its boolean stdin twin, e.g. `--passphrase-stdin`.
    pub stdin_flag: &'static str,
    /// What the value is, for the empty warning.
    pub noun: &'static str,
    /// The terminal prompt (ruling 3).
    pub prompt: &'static str,
}

/// `--passphrase`: BIP-39 (or, on `slip39`, SLIP-39) passphrase.
pub(crate) const PASSPHRASE: SecretFlag = SecretFlag {
    flag: "--passphrase",
    stdin_flag: "--passphrase-stdin",
    noun: "passphrase",
    prompt: "Enter passphrase: ",
};

/// `convert --bip38-passphrase`: the BIP-38 Scrypt passphrase.
pub(crate) const BIP38_PASSPHRASE: SecretFlag = SecretFlag {
    flag: "--bip38-passphrase",
    stdin_flag: "--bip38-passphrase-stdin",
    noun: "BIP-38 passphrase",
    prompt: "Enter BIP-38 passphrase: ",
};

/// `verify-bundle --ms1 -` (F-689): read through [`read_stdin_raw`] so a
/// terminal gets the same prompt and echo-off. Not an argv/`@env:` flag of
/// this module (its empty value is refused, not warned).
pub(crate) const MS1: SecretFlag = SecretFlag {
    flag: "--ms1",
    stdin_flag: "--ms1 -",
    noun: "ms1",
    prompt: "Enter ms1: ",
};

/// `import-wallet` / `electrum-decrypt --decrypt-password`.
pub(crate) const DECRYPT_PASSWORD: SecretFlag = SecretFlag {
    flag: "--decrypt-password",
    stdin_flag: "--decrypt-password-stdin",
    noun: "decryption password",
    prompt: "Enter decryption password: ",
};

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

/// [`stdin_spelling`] for any [`SecretFlag`].
pub(crate) fn stdin_spelling_for(f: &SecretFlag, stdin_flag: bool) -> String {
    if stdin_flag {
        f.stdin_flag.to_string()
    } else {
        format!("{} -", f.flag)
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

/// The note printed once for a literal argv value of `f`. Never carries the
/// value. For `--passphrase` it is byte-identical to ms's `ARGV_NOTE`.
pub(crate) fn argv_note(f: &SecretFlag) -> String {
    format!(
        "warning: secret material on argv ({flag}) \u{2014} read it privately with \
         {flag} - or {stdin} (stdin), or {flag} @env:VAR (environment variable)",
        flag = f.flag,
        stdin = f.stdin_flag
    )
}

/// Emit [`argv_note`] iff the value of `f` is literal on argv. Best-effort,
/// like every advisory: a closed stderr is not an error.
pub(crate) fn emit_argv_note_for<E: Write>(
    f: &SecretFlag,
    value: Option<&str>,
    stdin_flag: bool,
    stderr: &mut E,
) {
    if source(value, stdin_flag) == PassphraseSource::Argv {
        let _ = writeln!(stderr, "{}", argv_note(f));
    }
}

/// [`emit_argv_note_for`] for `--passphrase`.
pub(crate) fn emit_argv_note<E: Write>(value: Option<&str>, stdin_flag: bool, stderr: &mut E) {
    emit_argv_note_for(&PASSPHRASE, value, stdin_flag, stderr)
}

/// Ruling 2: the one line printed when a PRIVATE channel yields an empty
/// value. `origin` is `stdin` or `environment variable VAR`. Byte-identical
/// to ms's `empty_warning`.
pub(crate) fn empty_warning(f: &SecretFlag, origin: &str) -> String {
    format!(
        "warning: {} from {origin} is empty; proceeding with the EMPTY {}",
        f.flag, f.noun
    )
}

/// Print [`empty_warning`] iff `value` is empty. Goes to the PROCESS stderr:
/// it is emitted at the one place each value is read, which is below the
/// layer that carries a `stderr` writer on some paths (`@env:` pre-passes).
fn warn_if_empty(f: &SecretFlag, origin: &str, value: &str) {
    if value.is_empty() {
        let _ = writeln!(std::io::stderr(), "{}", empty_warning(f, origin));
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
    resolve_env_for(&PASSPHRASE, value)
}

/// [`resolve_env`] for any [`SecretFlag`]; warns once if the value is empty.
pub(crate) fn resolve_env_for(f: &SecretFlag, value: &str) -> Result<String, ToolkitError> {
    let mut v = crate::env_sentinel::resolve_env_var_sentinel(value, f.flag)?;
    strip_one_newline(&mut v);
    let var = value.strip_prefix("@env:").unwrap_or(value);
    warn_if_empty(f, &format!("environment variable {var}"), &v);
    Ok(v)
}

/// Is the PROCESS stdin a terminal? Never in unit tests (`cfg(test)`), whose
/// readers are in-memory cursors even when `cargo test` runs on a terminal.
fn stdin_is_terminal() -> bool {
    use std::io::IsTerminal;
    !cfg!(test) && std::io::stdin().is_terminal()
}

/// Echo off on fd 0 for the life of the guard (Unix). `ECHONL` keeps the
/// Enter visible, so the cursor moves on without showing the secret. The
/// saved mode is restored on drop (including on an error return) AND, while
/// the guard is live, by a handler for SIGINT/SIGTERM/SIGHUP/SIGQUIT that
/// restores the mode, resets the signal to its default and re-raises it, so
/// Ctrl-C at the prompt exits with the conventional signal status and a
/// working terminal. A signal the process inherited as IGNORED stays ignored.
struct EchoOff {
    #[cfg(unix)]
    saved: Option<libc::termios>,
    #[cfg(unix)]
    old_actions: Option<[libc::sigaction; 4]>,
}

#[cfg(unix)]
mod echo_signal {
    use std::sync::atomic::{AtomicBool, Ordering};

    /// The mode to restore from the handler. Written BEFORE `ACTIVE` is set
    /// and read only while it is set.
    static mut SAVED: std::mem::MaybeUninit<libc::termios> = std::mem::MaybeUninit::uninit();
    static ACTIVE: AtomicBool = AtomicBool::new(false);
    pub(super) const SIGNALS: [libc::c_int; 4] =
        [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGQUIT];

    /// Async-signal-safe: tcsetattr, signal and raise only.
    extern "C" fn restore_and_reraise(sig: libc::c_int) {
        if ACTIVE.swap(false, Ordering::SeqCst) {
            // SAFETY: SAVED was fully written before ACTIVE was set.
            unsafe {
                libc::tcsetattr(0, libc::TCSANOW, (*std::ptr::addr_of!(SAVED)).as_ptr());
            }
        }
        // SAFETY: default disposition, then deliver the same signal again.
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
            libc::raise(sig);
        }
    }

    /// Install the handlers; returns the previous actions.
    ///
    /// # Safety
    /// Single-threaded use around one prompt at a time.
    pub(super) unsafe fn arm(saved: &libc::termios) -> [libc::sigaction; 4] {
        (*std::ptr::addr_of_mut!(SAVED)).write(*saved);
        ACTIVE.store(true, Ordering::SeqCst);
        let mut old: [libc::sigaction; 4] = std::mem::zeroed();
        for (i, s) in SIGNALS.iter().enumerate() {
            libc::sigaction(*s, std::ptr::null(), &mut old[i]);
            if old[i].sa_sigaction == libc::SIG_IGN {
                continue;
            }
            let mut sa: libc::sigaction = std::mem::zeroed();
            sa.sa_sigaction = restore_and_reraise as extern "C" fn(libc::c_int) as usize;
            libc::sigemptyset(&mut sa.sa_mask);
            libc::sigaction(*s, &sa, std::ptr::null_mut());
        }
        old
    }

    /// Put the previous actions back.
    ///
    /// # Safety
    /// `old` is what [`arm`] returned.
    pub(super) unsafe fn disarm(old: &[libc::sigaction; 4]) {
        ACTIVE.store(false, Ordering::SeqCst);
        for (i, s) in SIGNALS.iter().enumerate() {
            libc::sigaction(*s, &old[i], std::ptr::null_mut());
        }
    }
}

impl EchoOff {
    /// `(guard, echo_disabled)`.
    fn new() -> (Self, bool) {
        #[cfg(unix)]
        {
            // SAFETY: tcgetattr/tcsetattr on fd 0 with a zeroed, then
            // kernel-filled, termios; the handlers are armed before echo goes
            // off, so no window exists with echo off and no handler.
            unsafe {
                let mut t: libc::termios = std::mem::zeroed();
                if libc::tcgetattr(0, &mut t) == 0 {
                    let saved = t;
                    let old = echo_signal::arm(&saved);
                    t.c_lflag &= !libc::ECHO;
                    t.c_lflag |= libc::ECHONL;
                    if libc::tcsetattr(0, libc::TCSANOW, &t) == 0 {
                        return (
                            Self {
                                saved: Some(saved),
                                old_actions: Some(old),
                            },
                            true,
                        );
                    }
                    echo_signal::disarm(&old);
                }
            }
            (
                Self {
                    saved: None,
                    old_actions: None,
                },
                false,
            )
        }
        #[cfg(not(unix))]
        {
            (Self {}, false)
        }
    }
}

impl Drop for EchoOff {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            if let Some(t) = self.saved.take() {
                // SAFETY: restores the attributes read in `new`. Restore
                // FIRST, then disarm: a signal in between restores again.
                unsafe {
                    libc::tcsetattr(0, libc::TCSANOW, &t);
                }
            }
            if let Some(old) = self.old_actions.take() {
                // SAFETY: `old` came from `arm`.
                unsafe { echo_signal::disarm(&old) };
            }
        }
    }
}

/// Read ONE line (up to and including the first `\n`, or EOF) byte by byte,
/// so nothing past the line is consumed.
fn read_line<R: Read + ?Sized>(stdin: &mut R, buf: &mut Vec<u8>) -> std::io::Result<()> {
    let mut b = [0u8; 1];
    loop {
        match stdin.read(&mut b) {
            Ok(0) => return Ok(()),
            Ok(_) => {
                buf.push(b[0]);
                if b[0] == b'\n' {
                    return Ok(());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

/// Read a passphrase from stdin, preserving every byte except ONE trailing
/// `\n` (and a `\r` before it). This is the byte rule `--passphrase-stdin` has
/// always had; `--passphrase -` shares it by calling the same function.
pub(crate) fn read_stdin_passphrase<R: Read + ?Sized>(
    stdin: &mut R,
) -> Result<String, ToolkitError> {
    read_stdin_secret(&PASSPHRASE, stdin)
}

/// [`read_stdin_passphrase`] for any [`SecretFlag`]. On a terminal (ruling
/// 3): prompt on stderr, echo off where possible, read one line. Otherwise
/// read to EOF, no prompt. Warns once if the result is empty (ruling 2).
pub(crate) fn read_stdin_secret<R: Read + ?Sized>(
    f: &SecretFlag,
    stdin: &mut R,
) -> Result<String, ToolkitError> {
    let buf = read_stdin_raw(f, stdin)?;
    warn_if_empty(f, "stdin", &buf);
    Ok(buf.to_string())
}

/// The reading half of [`read_stdin_secret`], without the empty warning, for
/// a stdin secret whose EMPTY value is refused rather than proceeded with
/// (`verify-bundle --ms1 -`). Terminal: prompt, echo off, one line; the
/// cursor is moved to a fresh line if the input did not end in one (Ctrl-D
/// at the prompt), so any following message starts on its own line. Pipe:
/// read to EOF. Either way ONE trailing `\n` / `\r\n` is stripped.
pub(crate) fn read_stdin_raw<R: Read + ?Sized>(
    f: &SecretFlag,
    stdin: &mut R,
) -> Result<Zeroizing<String>, ToolkitError> {
    let err = |e: std::io::Error| ToolkitError::BadInput(format!("stdin read: {e}"));
    // wave2 T4: scrub the SCRATCH buffer (passphrase material).
    let mut buf = Zeroizing::new(String::new());
    if stdin_is_terminal() {
        let (_echo, hidden) = EchoOff::new();
        let mut e = std::io::stderr();
        let _ = write!(
            e,
            "{}{}",
            f.prompt,
            if hidden {
                ""
            } else {
                "(input will be visible) "
            }
        );
        let _ = e.flush();
        let mut bytes = Zeroizing::new(Vec::new());
        read_line(stdin, &mut bytes).map_err(err)?;
        if bytes.last() != Some(&b'\n') {
            let _ = writeln!(e);
        }
        *buf = String::from_utf8(bytes.to_vec())
            .map_err(|_| ToolkitError::BadInput(format!("{}: stdin is not valid UTF-8", f.flag)))?;
    } else {
        stdin.read_to_string(&mut buf).map_err(err)?;
    }
    strip_one_newline(&mut buf);
    Ok(buf)
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
    resolve_for(&PASSPHRASE, value, stdin_flag, stdin)
}

/// [`resolve`] for any [`SecretFlag`].
pub(crate) fn resolve_for<R: Read + ?Sized>(
    f: &SecretFlag,
    value: Option<&str>,
    stdin_flag: bool,
    stdin: &mut R,
) -> Result<Option<Zeroizing<String>>, ToolkitError> {
    if stdin_flag && value.is_some() {
        return Err(ToolkitError::BadInput(format!(
            "{} and {} cannot both be given (one value, one stdin)",
            f.flag, f.stdin_flag
        )));
    }
    Ok(match source(value, stdin_flag) {
        PassphraseSource::Absent => None,
        PassphraseSource::Stdin => Some(Zeroizing::new(read_stdin_secret(f, stdin)?)),
        PassphraseSource::Env => Some(Zeroizing::new(resolve_env_for(f, value.unwrap_or(""))?)),
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
    fn notes_and_warnings_per_flag() {
        assert_eq!(
            argv_note(&PASSPHRASE),
            "warning: secret material on argv (--passphrase) \u{2014} read it privately with \
             --passphrase - or --passphrase-stdin (stdin), or --passphrase @env:VAR \
             (environment variable)"
        );
        assert!(argv_note(&BIP38_PASSPHRASE).contains("--bip38-passphrase-stdin (stdin)"));
        assert!(argv_note(&DECRYPT_PASSWORD).contains("--decrypt-password @env:VAR"));
        assert_eq!(
            empty_warning(&PASSPHRASE, "stdin"),
            "warning: --passphrase from stdin is empty; proceeding with the EMPTY passphrase"
        );
        assert_eq!(
            stdin_spelling_for(&BIP38_PASSPHRASE, false),
            "--bip38-passphrase -"
        );
    }

    #[test]
    fn read_line_stops_at_the_first_newline() {
        let mut cur = std::io::Cursor::new(b"TREZOR\nrest".to_vec());
        let mut v = Vec::new();
        read_line(&mut cur, &mut v).unwrap();
        assert_eq!(v, b"TREZOR\n");
        let mut rest = String::new();
        cur.read_to_string(&mut rest).unwrap();
        assert_eq!(rest, "rest");
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
