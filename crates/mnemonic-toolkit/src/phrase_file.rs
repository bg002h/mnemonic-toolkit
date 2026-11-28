//! `mnemonic convert --phrase-out-dir`: save BIP-39 words to a new owner-only
//! file that never replaces anything (`design/SPEC_convert_phrase_out_dir.md`
//! §3, §4).
//!
//! `mnemonic_io_lib::write::write_private` replaces and follows symlinks, so it
//! is not used here. FOLLOWUP `io-lib-write-private-new-hoist` moves this
//! mechanism into mnemonic-io-lib.

use crate::error::ToolkitError;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

/// Largest entry at the name that is read and compared (spec §4). The longest
/// 24-word phrase is well under it.
pub(crate) const COMPARE_CAP: u64 = 1024;

/// Temp-name attempts before giving up (spec §4 step 1).
const TEMP_TRIES: u32 = 16;

// ---------------------------------------------------------------------------
// §3 file name
// ---------------------------------------------------------------------------

/// `seed-<YYYY-MM-DD>-<fp>[-<fp-pass>][-<language>].txt`.
pub(crate) fn file_name(
    date: &str,
    fp: &str,
    fp_pass: Option<&str>,
    language: Option<&str>,
) -> String {
    let mut s = format!("seed-{date}-{fp}");
    if let Some(p) = fp_pass {
        s.push('-');
        s.push_str(p);
    }
    if let Some(l) = language {
        s.push('-');
        s.push_str(l);
    }
    s.push_str(".txt");
    s
}

/// A calendar date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ymd {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

impl Ymd {
    pub(crate) fn render(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// The UTC date of `secs` seconds since the Unix epoch (civil-from-days,
/// H. Hinnant's algorithm).
// Windows reads local time with `GetLocalTime`, which cannot fail.
#[cfg_attr(windows, allow(dead_code))]
pub(crate) fn utc_ymd(secs: i64) -> Ymd {
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    Ymd {
        year: if m <= 2 { y + 1 } else { y },
        month: m,
        day: d,
    }
}

/// The note printed when the local zone could not be read (spec §3).
// Windows reads local time with `GetLocalTime`, which cannot fail.
#[cfg_attr(windows, allow(dead_code))]
pub(crate) const UTC_FALLBACK_NOTE: &str = "note: local time zone unavailable; file date is UTC";

/// Resolve the file date from `now` (seconds since the epoch) through the
/// local-time call `local` (the test seam). `None` from `local` means the call
/// failed: the UTC date is used and the note is returned. A call that succeeds
/// with a UTC zone is a success (no note); an offset of 0 is never read as
/// "unavailable".
// Windows reads local time with `GetLocalTime`, which cannot fail.
#[cfg_attr(windows, allow(dead_code))]
pub(crate) fn resolve_date(
    now: i64,
    local: impl FnOnce(i64) -> Option<Ymd>,
) -> (String, Option<&'static str>) {
    match local(now) {
        Some(d) => (d.render(), None),
        None => (utc_ymd(now).render(), Some(UTC_FALLBACK_NOTE)),
    }
}

/// Today's date in the machine's local zone, and the UTC-fallback note if one
/// applies.
#[cfg(unix)]
pub(crate) fn today() -> (String, Option<&'static str>) {
    // SAFETY: `time(NULL)` only reads the clock.
    let now = unsafe { libc::time(std::ptr::null_mut()) } as i64;
    resolve_date(now, unix_localtime)
}

/// `localtime_r`. Sound here because no other thread is live while `convert`
/// runs and no non-test code calls `setenv` (spec §3). A later change that adds
/// threads to `convert` must revisit this.
#[cfg(unix)]
fn unix_localtime(now: i64) -> Option<Ymd> {
    let t = now as libc::time_t;
    // SAFETY: `tm` is plain data; `localtime_r` writes it or returns NULL.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::localtime_r(&t, &mut tm) };
    if r.is_null() {
        return None;
    }
    Some(Ymd {
        year: i64::from(tm.tm_year) + 1900,
        month: (tm.tm_mon + 1) as u32,
        day: tm.tm_mday as u32,
    })
}

/// `GetLocalTime` cannot fail, so Windows never prints the note.
#[cfg(windows)]
pub(crate) fn today() -> (String, Option<&'static str>) {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    // SAFETY: `GetLocalTime` fills the caller's SYSTEMTIME.
    let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut st) };
    let d = Ymd {
        year: i64::from(st.wYear),
        month: u32::from(st.wMonth),
        day: u32::from(st.wDay),
    };
    (d.render(), None)
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn today() -> (String, Option<&'static str>) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    resolve_date(now, |_| None)
}

// ---------------------------------------------------------------------------
// warnings
// ---------------------------------------------------------------------------

/// Which platform the permission decision is made for (a parameter so the
/// Windows branch is tested on Linux, spec §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Platform {
    Unix,
    Other,
}

impl Platform {
    pub(crate) fn current() -> Self {
        if cfg!(unix) {
            Platform::Unix
        } else {
            Platform::Other
        }
    }
}

/// The step-5 warning (spec §4): on Unix when the written (or already-saved)
/// file has group or other bits set; always on a platform with no mode bits.
pub(crate) fn permissions_warning(
    platform: Platform,
    mode: Option<u32>,
    path: &Path,
) -> Option<String> {
    match platform {
        Platform::Other => Some(format!(
            "warning: file permissions not restricted on this platform; protect {}",
            path.display()
        )),
        Platform::Unix => match mode {
            Some(m) if m & 0o077 != 0 => Some(format!(
                "warning: file permissions not restricted on this filesystem; protect {}",
                path.display()
            )),
            _ => None,
        },
    }
}

/// `DIR` group- or world-writable (spec §4 `DIR` edges).
pub(crate) fn dir_writable_warning(dir: &Path) -> Option<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let md = fs::metadata(dir).ok()?;
        if md.permissions().mode() & 0o022 != 0 {
            return Some(format!(
                "warning: {} is writable by other users; they could delete the saved file or plant names in it",
                dir.display()
            ));
        }
        None
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
        None
    }
}

/// Notes for a stale copy of the same seed (another day's `seed-*-<fp>*.txt`)
/// and any leftover `seed-tmp-*.txt` in `DIR` (spec §4). `own` is today's name,
/// which is the "already saved" case and is not noted.
pub(crate) fn stale_copy_notes(dir: &Path, fp: &str, own: &str) -> Vec<String> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = rd
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    let mut notes = Vec::new();
    for n in names {
        if n == own {
            continue;
        }
        if n.starts_with("seed-tmp-") && n.ends_with(".txt") {
            notes.push(format!(
                "note: a leftover temporary copy of a seed exists: {}",
                dir.join(&n).display()
            ));
        } else if is_same_seed_name(&n, fp) {
            notes.push(format!(
                "note: {} already holds this seed (saved on another day or with other name parts)",
                dir.join(&n).display()
            ));
        }
    }
    notes
}

/// `seed-YYYY-MM-DD-<fp>….txt`.
fn is_same_seed_name(name: &str, fp: &str) -> bool {
    let Some(rest) = name.strip_prefix("seed-") else {
        return false;
    };
    if !name.ends_with(".txt") || rest.len() < 11 || !rest.is_char_boundary(11) {
        return false;
    }
    let (date, tail) = rest.split_at(11);
    let db = date.as_bytes();
    let date_ok = db[4] == b'-'
        && db[7] == b'-'
        && db[10] == b'-'
        && db
            .iter()
            .enumerate()
            .all(|(i, c)| matches!(i, 4 | 7 | 10) || c.is_ascii_digit());
    date_ok
        && tail.starts_with(fp)
        && matches!(tail.as_bytes().get(fp.len()), Some(b'-') | Some(b'.'))
}

// ---------------------------------------------------------------------------
// §4 the write
// ---------------------------------------------------------------------------

/// Fault injection for the write path (tests only set these; spec §8 seams).
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Faults {
    /// Fail step 1's write after creating `T`.
    pub temp_write: bool,
    /// `hard_link` reports "unsupported" (FAT/exFAT stand-in).
    pub no_hard_link: bool,
    /// Fail step 4's write after writing half of the bytes to `N`.
    pub fallback_write: bool,
}

/// What happened at the name.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// A new file was written; its mode from `fstat` (Unix).
    Written { mode: Option<u32> },
    /// The same bytes were already there, untouched.
    AlreadySaved { mode: Option<u32> },
}

fn io_err(dir: &Path, what: &str, e: io::Error) -> ToolkitError {
    ToolkitError::Io(io::Error::new(
        e.kind(),
        format!("--phrase-out-dir {}: {what}: {e}", dir.display()),
    ))
}

fn exists(path: &Path, reason: impl Into<String>) -> ToolkitError {
    ToolkitError::PhraseFileExists {
        path: path.display().to_string(),
        reason: reason.into(),
    }
}

#[cfg(unix)]
fn handle_mode(f: &File) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    f.metadata().ok().map(|m| m.permissions().mode() & 0o7777)
}

#[cfg(not(unix))]
fn handle_mode(_f: &File) -> Option<u32> {
    None
}

fn create_new_0600(path: &Path) -> io::Result<File> {
    let mut o = OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    o.open(path)
}

/// Open an existing entry for the compare without following a link.
fn open_no_follow(path: &Path) -> io::Result<File> {
    let mut o = OpenOptions::new();
    o.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_FLAG_OPEN_REPARSE_POINT: a reparse point is opened as itself
        // and then refused by the regular-file check below.
        o.custom_flags(0x0020_0000);
    }
    o.open(path)
}

fn fsync_dir(dir: &Path) {
    // Best effort: some filesystems refuse a directory fsync (EINVAL), and
    // Windows cannot open a directory as a file. The data itself was fsynced.
    #[cfg(unix)]
    if let Ok(d) = File::open(dir) {
        let _ = d.sync_all();
    }
    #[cfg(not(unix))]
    let _ = dir;
}

/// Examine an entry already at `path` (spec §4 "An entry already at the
/// name"). `Ok(None)` when nothing is there.
pub(crate) fn check_existing(path: &Path, bytes: &[u8]) -> Result<Option<Outcome>, ToolkitError> {
    let md = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(exists(path, format!("cannot examine it: {e}"))),
    };
    let ft = md.file_type();
    if ft.is_symlink() {
        return Err(exists(path, "it is a symbolic link"));
    }
    if ft.is_dir() {
        return Err(exists(path, "it is a directory"));
    }
    if !ft.is_file() {
        return Err(exists(path, "it is not a regular file"));
    }
    if md.len() > COMPARE_CAP {
        return Err(exists(path, "it holds different contents"));
    }
    let mut f = open_no_follow(path).map_err(|e| exists(path, format!("cannot open it: {e}")))?;
    let fmd = f
        .metadata()
        .map_err(|e| exists(path, format!("cannot examine it: {e}")))?;
    if !fmd.file_type().is_file() || fmd.len() > COMPARE_CAP {
        return Err(exists(path, "it changed while being examined"));
    }
    // Zero-filled first so the pin covers initialised memory.
    let mut buf: Zeroizing<Vec<u8>> = Zeroizing::new(vec![0u8; COMPARE_CAP as usize + 1]);
    let _pin = mnemonic_toolkit::mlock::pin_pages_for(&buf);
    let mut len = 0usize;
    loop {
        if len == buf.len() {
            return Err(exists(path, "it holds different contents"));
        }
        match f.read(&mut buf[len..]) {
            Ok(0) => break,
            Ok(k) => len += k,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(exists(path, format!("cannot read it: {e}"))),
        }
    }
    if &buf[..len] == bytes {
        Ok(Some(Outcome::AlreadySaved {
            mode: handle_mode(&f),
        }))
    } else {
        Err(exists(path, "it holds different contents"))
    }
}

/// Write `bytes` to `dir/name` without ever replacing anything (spec §4 steps
/// 0-4). Step 5 (the mode warning) is the caller's.
pub(crate) fn write_new(
    dir: &Path,
    name: &str,
    bytes: &[u8],
    faults: Faults,
) -> Result<Outcome, ToolkitError> {
    let n = dir.join(name);

    // Step 0.
    if let Some(o) = check_existing(&n, bytes)? {
        return Ok(o);
    }

    // Step 1.
    let pid = std::process::id();
    let mut temp: Option<(PathBuf, File)> = None;
    for k in 0..TEMP_TRIES {
        let t = dir.join(format!("seed-tmp-{pid}-{k}.txt"));
        match create_new_0600(&t) {
            Ok(f) => {
                temp = Some((t, f));
                break;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(io_err(dir, "cannot create a temporary file", e)),
        }
    }
    let Some((t, mut tf)) = temp else {
        return Err(io_err(
            dir,
            "cannot create a temporary file",
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("seed-tmp-{pid}-0..{} all exist", TEMP_TRIES - 1),
            ),
        ));
    };
    let written = if faults.temp_write {
        Err(io::Error::other("injected step-1 write failure"))
    } else {
        tf.write_all(bytes).and_then(|()| tf.sync_all())
    };
    if let Err(e) = written {
        drop(tf);
        let _ = fs::remove_file(&t);
        return Err(io_err(dir, "cannot write the temporary file", e));
    }
    let mode = handle_mode(&tf);
    drop(tf);

    // Step 2.
    let linked = if faults.no_hard_link {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "injected: hard links unsupported",
        ))
    } else {
        fs::hard_link(&t, &n)
    };

    // Step 3: unlink T on every outcome of step 2.
    let unlinked = fs::remove_file(&t);
    fsync_dir(dir);
    if let Err(e) = unlinked {
        if e.kind() != io::ErrorKind::NotFound {
            // Only reachable when unlink itself fails; say where the copy is.
            return Err(io_err(
                dir,
                &format!(
                    "cannot remove the temporary copy {}; delete it",
                    t.display()
                ),
                e,
            ));
        }
    }

    match linked {
        Ok(()) => return Ok(Outcome::Written { mode }),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            // A racer won: examine what is there.
            return match check_existing(&n, bytes)? {
                Some(o) => Ok(o),
                None => Err(exists(&n, "it changed during the write")),
            };
        }
        Err(_) => {} // step 4
    }

    // Step 4: exclusive create on the real name.
    let mut nf = match create_new_0600(&n) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            return match check_existing(&n, bytes)? {
                Some(o) => Ok(o),
                None => Err(exists(&n, "it changed during the write")),
            };
        }
        Err(e) => return Err(io_err(dir, "cannot create the file", e)),
    };
    let written = if faults.fallback_write {
        nf.write_all(&bytes[..bytes.len() / 2])
            .and_then(|()| Err(io::Error::other("injected step-4 write failure")))
    } else {
        nf.write_all(bytes).and_then(|()| nf.sync_all())
    };
    if let Err(e) = written {
        drop(nf);
        let _ = fs::remove_file(&n);
        return Err(io_err(dir, "cannot write the file", e));
    }
    let mode = handle_mode(&nf);
    drop(nf);
    fsync_dir(dir);
    Ok(Outcome::Written { mode })
}

/// Validate `--phrase-out-dir <DIR>` (spec §2.5).
pub(crate) fn check_dir(dir: &Path) -> Result<(), ToolkitError> {
    if dir.as_os_str() == "-" {
        return Err(ToolkitError::BadInput(
            "--phrase-out-dir -: stdout is not accepted; give an existing directory".into(),
        ));
    }
    match fs::metadata(dir) {
        Ok(m) if m.is_dir() => Ok(()),
        Ok(_) => Err(ToolkitError::BadInput(format!(
            "--phrase-out-dir {}: not a directory",
            dir.display()
        ))),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(ToolkitError::BadInput(format!(
            "--phrase-out-dir {}: no such directory (it is never created)",
            dir.display()
        ))),
        Err(e) => Err(ToolkitError::BadInput(format!(
            "--phrase-out-dir {}: {e}",
            dir.display()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: &[u8] = b"abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about\n";

    fn entries(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        v.sort();
        v
    }

    #[test]
    fn name_with_injected_date() {
        assert_eq!(
            file_name("2026-11-28", "73c5da0a", None, None),
            "seed-2026-11-28-73c5da0a.txt"
        );
        assert_eq!(
            file_name("2026-11-28", "73c5da0a", Some("b4e3f5ed"), Some("japanese")),
            "seed-2026-11-28-73c5da0a-b4e3f5ed-japanese.txt"
        );
    }

    #[test]
    fn utc_civil_from_days() {
        assert_eq!(utc_ymd(0).render(), "1970-01-01");
        assert_eq!(utc_ymd(951_782_400).render(), "2000-02-29");
        // 2026-11-28T23:59:59Z
        assert_eq!(utc_ymd(1_795_910_399).render(), "2026-11-28");
        assert_eq!(utc_ymd(1_795_910_400).render(), "2026-11-29");
        assert_eq!(utc_ymd(-1).render(), "1969-12-31");
    }

    #[test]
    fn local_time_failing_gives_utc_and_the_note() {
        let (d, note) = resolve_date(1_795_910_399, |_| None);
        assert_eq!(d, "2026-11-28");
        assert_eq!(note, Some(UTC_FALLBACK_NOTE));
    }

    #[test]
    fn utc_zone_with_success_gives_no_note() {
        // A configured-UTC (or unconfigured glibc/musl) zone: success, offset 0.
        let (d, note) = resolve_date(1_795_910_399, |t| Some(utc_ymd(t)));
        assert_eq!(d, "2026-11-28");
        assert_eq!(note, None);
        // A local zone east of UTC moves the date; still no note.
        let (d, note) = resolve_date(1_795_910_399, |t| Some(utc_ymd(t + 3600)));
        assert_eq!(d, "2026-11-29");
        assert_eq!(note, None);
    }

    #[test]
    fn windows_warning_is_a_pure_platform_decision() {
        let p = Path::new("/d/seed.txt");
        let w = permissions_warning(Platform::Other, None, p).unwrap();
        assert_eq!(
            w,
            "warning: file permissions not restricted on this platform; protect /d/seed.txt"
        );
        assert!(permissions_warning(Platform::Unix, Some(0o600), p).is_none());
        let w = permissions_warning(Platform::Unix, Some(0o644), p).unwrap();
        assert!(w.contains("not restricted on this filesystem"), "{w}");
        assert!(permissions_warning(Platform::Unix, Some(0o640), p).is_some());
    }

    #[test]
    fn same_seed_name_matcher() {
        assert!(is_same_seed_name(
            "seed-2026-11-27-73c5da0a.txt",
            "73c5da0a"
        ));
        assert!(is_same_seed_name(
            "seed-2026-11-27-73c5da0a-b4e3f5ed.txt",
            "73c5da0a"
        ));
        assert!(!is_same_seed_name(
            "seed-2026-11-27-73c5da0b.txt",
            "73c5da0a"
        ));
        assert!(!is_same_seed_name("seed-tmp-1-0.txt", "73c5da0a"));
        assert!(!is_same_seed_name(
            "seed-2026-11-27-73c5da0aff.txt",
            "73c5da0a"
        ));
    }

    #[test]
    fn step1_write_failing_leaves_no_temp() {
        let d = tempfile::tempdir().unwrap();
        let faults = Faults {
            temp_write: true,
            ..Faults::default()
        };
        let e = write_new(d.path(), "seed-x.txt", WORDS, faults).unwrap_err();
        assert_eq!(e.exit_code(), 1);
        assert!(matches!(e, ToolkitError::Io(_)), "{e:?}");
        assert!(entries(d.path()).is_empty(), "{:?}", entries(d.path()));
    }

    #[test]
    fn hard_link_unsupported_uses_the_fallback() {
        let d = tempfile::tempdir().unwrap();
        let faults = Faults {
            no_hard_link: true,
            ..Faults::default()
        };
        let o = write_new(d.path(), "seed-x.txt", WORDS, faults).unwrap();
        assert!(matches!(o, Outcome::Written { .. }));
        assert_eq!(entries(d.path()), vec!["seed-x.txt".to_string()]);
        assert_eq!(fs::read(d.path().join("seed-x.txt")).unwrap(), WORDS);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let m = fs::metadata(d.path().join("seed-x.txt"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(m & 0o777, 0o600);
        }
    }

    #[test]
    fn fallback_write_failing_mid_file_leaves_nothing() {
        let d = tempfile::tempdir().unwrap();
        let faults = Faults {
            no_hard_link: true,
            fallback_write: true,
            ..Faults::default()
        };
        let e = write_new(d.path(), "seed-x.txt", WORDS, faults).unwrap_err();
        assert!(matches!(e, ToolkitError::Io(_)), "{e:?}");
        assert!(entries(d.path()).is_empty(), "{:?}", entries(d.path()));
    }

    #[test]
    fn temp_name_collision_takes_the_next_k() {
        let d = tempfile::tempdir().unwrap();
        let planted = d
            .path()
            .join(format!("seed-tmp-{}-0.txt", std::process::id()));
        fs::write(&planted, b"planted").unwrap();
        let o = write_new(d.path(), "seed-x.txt", WORDS, Faults::default()).unwrap();
        assert!(matches!(o, Outcome::Written { .. }));
        // The planted temp is untouched; our own temp (k = 1) is gone.
        assert_eq!(fs::read(&planted).unwrap(), b"planted");
        let mut want = vec![
            "seed-x.txt".to_string(),
            format!("seed-tmp-{}-0.txt", std::process::id()),
        ];
        want.sort();
        assert_eq!(entries(d.path()), want);
    }

    #[test]
    fn identical_entry_is_already_saved_and_different_is_refused() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("seed-x.txt"), WORDS).unwrap();
        let o = write_new(d.path(), "seed-x.txt", WORDS, Faults::default()).unwrap();
        assert!(matches!(o, Outcome::AlreadySaved { .. }));
        let e = write_new(d.path(), "seed-x.txt", b"other\n", Faults::default()).unwrap_err();
        assert!(matches!(e, ToolkitError::PhraseFileExists { .. }), "{e:?}");
        assert_eq!(entries(d.path()), vec!["seed-x.txt".to_string()]);
    }
}
