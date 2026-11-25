//! **Creating a file that a bearer artifact may be written into.**
//!
//! One writing function (and `same_file`, below). It is here rather than in [`super::channel`] or [`super::fd`]
//! because both of those declare narrower contracts than an effectful write:
//! `channel` is *"classification ONLY"* and says in as many words that
//! `destination` **never touches a path**, and `fd` is *"what was measured about
//! stdout"* — the read side of one file descriptor. Widening either header to
//! admit a path-writer would cost more than a seventh module does.
//!
//! ## Why `0o600` here is mechanism, when `0o044` and `0o077` are not
//!
//! [`super::fd`] refuses to publish a *disqualifying* mask because `me` rules
//! `0o044` and `mt` rules `0o077`, and the crate does not get to settle which
//! of them is right. **That is a disagreement about somebody else's file.**
//!
//! The mode a tool creates *its own* output at is not that question, and the two
//! consumers do not disagree about it: a file this crate's callers create for a
//! bearer artifact is owner-only. So `0o600` is a constant here rather than a
//! parameter. If a third consumer ever needs a different creation mode, that is
//! the moment to add one — not before, and a parameter added speculatively would
//! be the first place a caller could weaken the mode by accident.
//!
//! ## Is this output one of my inputs
//!
//! [`same_file`] answers that question for the callers that read and write in
//! one run (`--out X` with `X` also an input, or stdin or stdout redirected to
//! it). It is mechanism only: it returns the index of the first matching input
//! and the caller decides the refusal.
//!
//! ## F-244 — the half that is easy to leave out
//!
//! `OpenOptions::mode()` binds **on create only**. Open an existing file with it
//! and the mode on disk is untouched, so `me sysw pack --out stale.bin` over a
//! target already at `0644` left it at `0644` — and that is the case an operator
//! re-running a command actually hits, not an exotic one.
//!
//! The fix is the `set_permissions` call on the **open temp file**, made before
//! any byte is written, and the temp is created `0o600` and renamed over the
//! old file. A path-based `set_permissions` names the file a second time, and
//! between the two calls the name can be made to point somewhere else; a handle
//! cannot be redirected once it is open.

use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[cfg(test)]
use std::cell::Cell;

/// The steps a test can make fail (test builds only).
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    CreateTemp,
    Preflight,
    Chmod,
    Write,
    Sync,
    Rename,
    DirSync,
}

#[cfg(test)]
thread_local! {
    static FAULT: Cell<Option<Step>> = const { Cell::new(None) };
}

/// Return the injected error if a test asked for `step` to fail.
#[cfg(test)]
fn fault(step: Step) -> io::Result<()> {
    if FAULT.with(Cell::get) == Some(step) {
        return Err(io::Error::other(format!("injected at {step:?}")));
    }
    Ok(())
}

/// Write `bytes` to `path`, creating or replacing it owner-only.
///
/// For a regular file (or an absent path) the bytes go to a sibling temp file,
/// `<name>.<pid>.<n>.tmp`, created `0o600`, synced, and renamed over the final
/// name, so any error leaves the old file byte-identical. A hard link is
/// replaced at the given name only. Symlinks are resolved on the last
/// component and the target is replaced; the link stays a link. A rename that
/// fails (cross-device, bind mount, busy) is a refusal: there is no copy
/// fallback.
///
/// A standard-stream alias (Unix) names a descriptor N of THIS process:
/// `/dev/stdin`, `/dev/stdout`, `/dev/stderr`, `/dev/fd/N`, or
/// `/proc/{self,thread-self,<own pid>}/fd/N` (also `.../task/T/fd/N`), spelled
/// so as given, after its parent is canonicalised, or at a symlink hop. The
/// bytes go INTO that stream and nothing is truncated: `--out /dev/stdout >> f`
/// appends to `f`, and `{ a; tool --out /dev/fd/3; b; } 3> f` keeps all three
/// in order. Descriptor N is duplicated (its own offset and `O_APPEND`);
/// stdout is flushed and locked first. `0o600` is set on the handle first. A
/// failed write can leave part of `bytes` in the stream at its offset (after
/// the earlier contents when it appends). Any
/// other `/proc/` path (`/proc/self/cwd/x`, another process's `/proc/<pid>/`)
/// is a file name like any other: a regular file there is replaced ATOMIC.
///
/// A FIFO, a device, a directory or anything the library cannot classify is
/// written directly, as before: open, `0o600` on the handle, truncate if a
/// regular file, write.
///
/// Errors are the `std::io::Error` of the system call that failed, unmodified.
/// On non-Unix there are no mode calls and no directory sync.
pub fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    #[cfg(unix)]
    {
        if let Some(fd) = stdio_alias_of(path) {
            return write_alias(fd, bytes);
        }
    }
    match classify(path) {
        Some((final_path, replace)) => write_atomic(&final_path, replace, bytes),
        None => write_direct(path, bytes),
    }
}

/// DIRECT: open in place, `0o600` on the handle, truncate a regular file, write.
fn write_direct(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut opts = OpenOptions::new();
    opts.write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    // F-244. `opts.mode()` above did nothing if `path` already existed, so the
    // mode is set a second time on the OPEN FILE -- see the module header for
    // why the handle and not the path.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    if f.metadata()?.is_file() {
        f.set_len(0)?;
    }
    f.write_all(bytes)
}

/// True when `path` as given names a descriptor of this process (Unix only).
fn is_stdio_alias(path: &Path) -> bool {
    #[cfg(unix)]
    {
        alias_fd(path).is_some()
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        false
    }
}

/// The descriptor `p` names: as given, or as its canonical parent joined with
/// its name (`/dev/./stdout`, or `d/1` with `d` -> `/dev/fd`).
#[cfg(unix)]
fn spelled_alias(p: &Path) -> Option<i32> {
    if let Some(n) = alias_fd(p) {
        return Some(n);
    }
    let dir = match p.parent() {
        Some(d) if !d.as_os_str().is_empty() => d,
        _ => Path::new("."),
    };
    // The name itself must exist: `/dev/fd/1/.` has file_name `1` but is
    // ENOTDIR to the kernel.
    fs::symlink_metadata(p).ok()?;
    alias_fd(&fs::canonicalize(dir).ok()?.join(p.file_name()?))
}

/// The descriptor `path` reaches: itself, or a hop of its last component's
/// symlink chain (the walk of `resolve_final`, at most 40 hops).
#[cfg(unix)]
fn stdio_alias_of(path: &Path) -> Option<i32> {
    let mut p = path.to_path_buf();
    for _ in 0..=40 {
        if let Some(n) = spelled_alias(&p) {
            return Some(n);
        }
        if !fs::symlink_metadata(&p).ok()?.file_type().is_symlink() {
            return None;
        }
        let link = fs::read_link(&p).ok()?;
        p = if link.is_absolute() {
            link
        } else {
            p.parent()?.join(link)
        };
    }
    None
}

/// The descriptor of THIS process `p` names as spelled (spec section 3a): 0,
/// 1, 2 for `/dev/stdin|stdout|stderr`; N for `/dev/fd/N` and `/proc/W/fd/N`,
/// W = `self`, `thread-self`, this pid, `self/task/T` or `<this pid>/task/T`
/// (`thread-self` canonicalises to `<pid>/task/<tid>`). N and T are digits,
/// and the name must exist, so a closed N, another process's thread T or
/// `/dev/fd/01` is `None`. So is every other path: `/proc/self/cwd/x` and
/// `/proc/<another pid>/fd/N` are file names.
#[cfg(unix)]
fn alias_fd(p: &Path) -> Option<i32> {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let s = p.to_str()?;
    match s {
        "/dev/stdin" => return Some(0),
        "/dev/stdout" => return Some(1),
        "/dev/stderr" => return Some(2),
        _ => {}
    }
    let n = match s.strip_prefix("/dev/fd/") {
        Some(n) => n,
        None => {
            let (who, n) = s.strip_prefix("/proc/")?.rsplit_once("/fd/")?;
            let who = match who.split_once("/task/") {
                Some((w, t)) if digits(t) && w != "thread-self" => w,
                Some(_) => return None,
                None => who,
            };
            // This process's pid as THIS /proc numbers it: getpid() differs in a
            // pid namespace that kept its parent's /proc.
            let me = fs::read_link("/proc/self").ok()?;
            if who != "self" && who != "thread-self" && Path::new(who) != me {
                return None;
            }
            n
        }
    };
    if !digits(n) {
        return None;
    }
    let fd = n.parse().ok()?;
    fs::symlink_metadata(p).ok()?;
    Some(fd)
}

/// ALIAS: into the stream, never truncated (spec section 3a). Descriptor `fd`
/// is duplicated, so the write lands at the stream's own offset and keeps its
/// `O_APPEND`, whatever `fd` is.
#[cfg(unix)]
fn write_alias(fd: i32, bytes: &[u8]) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::io::{AsFd, BorrowedFd};
    // Held across the write: nothing else here prints to stdout in between.
    let mut out = io::stdout().lock();
    if fd != 1 {
        // N may share stdout's file (`>f 3>&1`): what the caller already
        // printed lands first. A broken stdout is the caller's own print's
        // error to report, not this write's.
        let _ = out.flush();
    }
    let owned = match fd {
        0 => io::stdin().as_fd().try_clone_to_owned()?,
        1 => {
            // What the caller already printed lands before the bytes.
            out.flush()?;
            out.as_fd().try_clone_to_owned()?
        }
        2 => io::stderr().as_fd().try_clone_to_owned()?,
        // SAFETY: `alias_fd` saw `fd`'s name exist, so it was open; the borrow
        // lives for this one fcntl(F_DUPFD_CLOEXEC), which closes nothing. If
        // another thread closed `fd` since, the dup fails with EBADF, or, if the
        // number was reused, duplicates the new file (as std's own stdio would).
        n => unsafe { BorrowedFd::borrow_raw(n) }.try_clone_to_owned()?,
    };
    let mut f = File::from(owned);
    // A descriptor not open for writing (an input, `/dev/stdin < f`) is
    // refused before its file's mode is touched.
    if !open_for_writing(&f)? {
        return Err(io::Error::from_raw_os_error(9)); // EBADF, as write(2) says
    }
    f.set_permissions(fs::Permissions::from_mode(0o600))?;
    f.write_all(bytes)
}

/// `fcntl(F_GETFL)` access mode is `O_WRONLY` or `O_RDWR`. Declared against
/// the C library std already links (no dependency; spec section 2 item 4).
/// `F_GETFL` = 3, `O_ACCMODE` = 3, `O_RDONLY` = 0 on Linux and Apple.
#[cfg(unix)]
fn open_for_writing(f: &File) -> io::Result<bool> {
    use std::os::raw::c_int;
    use std::os::unix::io::AsRawFd;
    extern "C" {
        fn fcntl(fd: c_int, cmd: c_int, ...) -> c_int;
    }
    // SAFETY: the descriptor belongs to `f`, which outlives the call; F_GETFL
    // takes no third argument and only reads the descriptor's flags.
    let fl = unsafe { fcntl(f.as_raw_fd(), 3) };
    if fl < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(fl & 3 != 0)
}

#[cfg(unix)]
fn dev_ino(m: &fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (m.dev(), m.ino())
}

/// Walk the last component's symlink chain (spec section 3 step 2). `None`
/// means DIRECT. `NotFound` ends the walk (an absent path or a dangling link's
/// target), which is the final name.
fn resolve_final(path: &Path) -> Option<PathBuf> {
    let mut p = path.to_path_buf();
    let mut hops = 0u32;
    loop {
        match fs::symlink_metadata(&p) {
            Ok(m) if m.file_type().is_symlink() => {
                hops += 1;
                if hops > 40 {
                    return None;
                }
                let link = fs::read_link(&p).ok()?;
                p = if link.is_absolute() {
                    link
                } else {
                    p.parent()?.join(link)
                };
                if is_stdio_alias(&p) {
                    return None;
                }
            }
            Ok(_) => break,
            Err(e) if e.kind() == io::ErrorKind::NotFound => break,
            Err(_) => return None,
        }
    }
    p.file_name()?;
    Some(p)
}

/// Steps 1 to 3. `Some((final name, replace))` is ATOMIC; `None` is DIRECT.
fn classify(path: &Path) -> Option<(PathBuf, bool)> {
    if is_stdio_alias(path) {
        return None;
    }
    #[cfg(unix)]
    let mut ident = None;
    let replace = match fs::metadata(path) {
        Ok(m) if m.is_file() => {
            #[cfg(unix)]
            {
                ident = Some(dev_ino(&m));
            }
            true
        }
        Ok(_) => return None,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(_) => return None,
    };
    let fin = resolve_final(path)?;
    if replace {
        let m = fs::metadata(&fin).ok()?;
        if !m.is_file() {
            return None;
        }
        #[cfg(unix)]
        if Some(dev_ino(&m)) != ident {
            return None;
        }
    } else {
        match fs::symlink_metadata(&fin) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            _ => return None,
        }
    }
    Some((fin, replace))
}

/// The temp name: `<name>.<pid>.<n>.tmp`, or `mnemonic-io-lib.<pid>.<n>.tmp`
/// when `name` is over 200 bytes (a suffixed name would exceed NAME_MAX).
fn temp_name(name: &OsStr, pid: u32, n: u32) -> OsString {
    let base: &OsStr = if name.as_encoded_bytes().len() <= 200 {
        name
    } else {
        OsStr::new("mnemonic-io-lib")
    };
    let mut out = base.to_os_string();
    out.push(format!(".{pid}.{n}.tmp"));
    out
}

/// ATOMIC (spec section 3 steps 4 to 11).
fn write_atomic(fin: &Path, replace: bool, bytes: &[u8]) -> io::Result<()> {
    let dir: &Path = match fin.parent() {
        Some(d) if !d.as_os_str().is_empty() => d,
        _ => Path::new("."),
    };
    let name = fin.file_name().unwrap_or_default();
    let pid = std::process::id();

    // Step 4: create the temp.
    #[cfg(test)]
    fault(Step::CreateTemp)?;
    let mut n = 0u32;
    let (temp, mut handle) = loop {
        let t = dir.join(temp_name(name, pid, n));
        let mut opts = OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(&t) {
            Ok(f) => break (t, f),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists && n < 99 => n += 1,
            Err(e) => return Err(e),
        }
    };

    // Steps 5 to 8; any error removes the temp (step 9).
    let r = fill_and_rename(fin, replace, bytes, &temp, &mut handle);
    drop(handle);
    if let Err(e) = r {
        let _ = fs::remove_file(&temp);
        return Err(e);
    }

    // Step 10: the directory sync, whose error is ignored.
    #[cfg(unix)]
    {
        let _ = (|| -> io::Result<()> {
            #[cfg(test)]
            fault(Step::DirSync)?;
            File::open(dir)?.sync_all()
        })();
    }
    Ok(())
}

fn fill_and_rename(
    fin: &Path,
    replace: bool,
    bytes: &[u8],
    temp: &Path,
    handle: &mut File,
) -> io::Result<()> {
    // Step 5: preflight (REPLACE only).
    if replace {
        #[cfg(test)]
        fault(Step::Preflight)?;
        let f = OpenOptions::new().write(true).open(fin)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            f.set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        drop(f);
    }
    // Step 6: owner-only on the temp handle.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        #[cfg(test)]
        fault(Step::Chmod)?;
        handle.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    // Step 7: write, sync.
    #[cfg(test)]
    fault(Step::Write)?;
    handle.write_all(bytes)?;
    #[cfg(test)]
    fault(Step::Sync)?;
    sync_temp(handle)?;
    // Step 8: rename over the final name; no copy fallback.
    #[cfg(test)]
    fault(Step::Rename)?;
    fs::rename(temp, fin)
}

/// Step 7's sync. On Apple targets std's `sync_all` is `fcntl(F_FULLFSYNC)`
/// with no fallback, and SMB and some FUSE volumes refuse it with ENOTSUP
/// (45); there a plain `fsync(2)` runs instead and its result stands, as Go
/// (go#64215) and SQLite do. Any other error is returned unchanged.
fn sync_temp(handle: &File) -> io::Result<()> {
    full_sync_or_plain(cfg!(target_vendor = "apple"), handle.sync_all(), || {
        plain_fsync(handle)
    })
}

/// The fallback rule of `sync_temp`, separated so every platform can test it.
fn full_sync_or_plain(
    apple: bool,
    full: io::Result<()>,
    plain: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    match full {
        Err(e) if apple && e.raw_os_error() == Some(45) => plain(),
        r => r,
    }
}

/// `fsync(2)` itself: std exposes no plain fsync on Apple targets, and the
/// crate takes no dependency (spec section 2 item 4), so it is declared here
/// against libSystem, which std already links.
#[cfg(target_vendor = "apple")]
fn plain_fsync(handle: &File) -> io::Result<()> {
    use std::os::raw::c_int;
    use std::os::unix::io::AsRawFd;
    extern "C" {
        fn fsync(fd: c_int) -> c_int;
    }
    loop {
        // SAFETY: the descriptor belongs to `handle`, which outlives the call.
        if unsafe { fsync(handle.as_raw_fd()) } == 0 {
            return Ok(());
        }
        let e = io::Error::last_os_error();
        if e.kind() != io::ErrorKind::Interrupted {
            return Err(e);
        }
    }
}

/// Off Apple targets `sync_all` is already the plain sync (never reached:
/// `full_sync_or_plain` falls back only on Apple).
#[cfg(not(target_vendor = "apple"))]
fn plain_fsync(handle: &File) -> io::Result<()> {
    handle.sync_all()
}

/// One side of a "same file" question.
#[derive(Debug, Clone, Copy)]
pub enum Endpoint<'a> {
    /// A path (symlinks followed; an absent path is compared by where it
    /// would be created).
    Path(&'a Path),
    /// This process's standard input.
    Stdin,
    /// This process's standard output.
    Stdout,
}

/// What an endpoint reduces to.
enum Identity {
    #[cfg(unix)]
    Id(u64, u64),
    #[cfg(not(unix))]
    Canon(PathBuf),
    Planned(PathBuf),
    NonRegular,
    Unknown,
}

fn identity_matches(a: &Identity, b: &Identity) -> bool {
    match (a, b) {
        #[cfg(unix)]
        (Identity::Id(d1, i1), Identity::Id(d2, i2)) => d1 == d2 && i1 == i2,
        #[cfg(not(unix))]
        (Identity::Canon(x), Identity::Canon(y)) => x == y,
        // Byte-equal, not component-equal: `Path` equality would fold `a/./x` into `a/x`.
        (Identity::Planned(x), Identity::Planned(y)) => x.as_os_str() == y.as_os_str(),
        _ => false,
    }
}

fn path_identity(p: &Path) -> Identity {
    match fs::metadata(p) {
        Ok(m) if m.is_file() => {
            #[cfg(unix)]
            {
                let (d, i) = dev_ino(&m);
                Identity::Id(d, i)
            }
            #[cfg(not(unix))]
            {
                match fs::canonicalize(p) {
                    Ok(c) => Identity::Canon(c),
                    Err(_) => Identity::Unknown,
                }
            }
        }
        Ok(_) => Identity::NonRegular,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let Some(fin) = resolve_final(p) else {
                return Identity::Unknown;
            };
            let Some(name) = fin.file_name() else {
                return Identity::Unknown;
            };
            let dir: &Path = match fin.parent() {
                Some(d) if !d.as_os_str().is_empty() => d,
                _ => Path::new("."),
            };
            match fs::canonicalize(dir) {
                Ok(c) => Identity::Planned(c.join(name)),
                Err(_) => Identity::Unknown,
            }
        }
        Err(_) => Identity::Unknown,
    }
}

#[cfg(unix)]
fn fd_identity(owned: io::Result<std::os::fd::OwnedFd>) -> Identity {
    match owned.map(File::from).and_then(|f| f.metadata()) {
        Ok(m) if m.is_file() => {
            let (d, i) = dev_ino(&m);
            Identity::Id(d, i)
        }
        Ok(_) => Identity::NonRegular,
        Err(_) => Identity::Unknown,
    }
}

fn endpoint_identity(e: Endpoint<'_>) -> Identity {
    match e {
        Endpoint::Path(p) => path_identity(p),
        #[cfg(unix)]
        Endpoint::Stdin => {
            use std::os::fd::AsFd;
            fd_identity(io::stdin().as_fd().try_clone_to_owned())
        }
        #[cfg(unix)]
        Endpoint::Stdout => {
            use std::os::fd::AsFd;
            fd_identity(io::stdout().as_fd().try_clone_to_owned())
        }
        #[cfg(not(unix))]
        Endpoint::Stdin | Endpoint::Stdout => Identity::Unknown,
    }
}

/// The index of the FIRST entry of `inputs` that is the same file as `out`, or
/// `None`. Never an error: a lookup that fails matches nothing, because the
/// write or read that follows reports it.
///
/// A regular file is identified by `(dev, ino)` on Unix and by its canonical
/// path on Windows; an absent path by its canonical directory plus its final
/// name. A pipe, tty or device (`NonRegular`) and a failed lookup (`Unknown`)
/// never match anything, so a terminal shared by stdin and stdout is never "the
/// same file". Limits: two absent paths that differ only in letter case on a
/// case-insensitive filesystem do not match; on Windows a hard link and a
/// redirected stdin or stdout are not detected (std exposes no stable file id).
pub fn same_file(out: Endpoint<'_>, inputs: &[Endpoint<'_>]) -> Option<usize> {
    let o = endpoint_identity(out);
    if matches!(o, Identity::NonRegular | Identity::Unknown) {
        return None;
    }
    inputs
        .iter()
        .position(|i| identity_matches(&o, &endpoint_identity(*i)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output, Stdio};

    #[cfg(unix)]
    fn mode_of(p: &std::path::Path) -> u32 {
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    /// Sorted entry names of `d`.
    fn names(d: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(d)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        v.sort();
        v
    }

    /// The child harness: the test binary re-executes itself with
    /// `--exact write::tests::child --nocapture`. `ulimit` is `None` (no `sh`),
    /// `Some(true)` (`sh`, SIGXFSZ ignored, `ulimit -f 1`) or `Some(false)`
    /// (`sh`, `ulimit -f 1`, signal left at its default).
    fn run_child(row: &str, target: &Path, stdout: Stdio, ulimit: Option<bool>) -> Output {
        let exe = std::env::current_exe().unwrap();
        let mut cmd = match ulimit {
            None => Command::new(&exe),
            Some(trap) => {
                let mut c = Command::new("sh");
                let script = if trap {
                    "trap '' XFSZ; ulimit -f 1; exec \"$0\" \"$@\""
                } else {
                    "ulimit -f 1; exec \"$0\" \"$@\""
                };
                c.arg("-c").arg(script).arg(&exe);
                c
            }
        };
        cmd.args(["--exact", "write::tests::child", "--nocapture"])
            .env("MNEMONIC_IO_LIB_CHILD", row)
            .env("MNEMONIC_IO_LIB_TARGET", target)
            .stdin(Stdio::piped())
            .stdout(stdout)
            .stderr(Stdio::piped());
        cmd.output().unwrap()
    }

    fn show(r: Option<usize>) -> String {
        match r {
            Some(i) => format!("Some({i})"),
            None => "None".to_string(),
        }
    }

    /// Not a row: the body of the child processes the rows above start. It
    /// returns at once when `MNEMONIC_IO_LIB_CHILD` is unset.
    #[test]
    fn child() {
        let Ok(row) = std::env::var("MNEMONIC_IO_LIB_CHILD") else {
            return;
        };
        let target = PathBuf::from(std::env::var_os("MNEMONIC_IO_LIB_TARGET").unwrap_or_default());
        let r: std::io::Result<String> = match row.as_str() {
            "W1" | "W2" => write_private(&target, &[b'N'; 4096]).map(|()| "Ok".to_string()),
            "W10" => write_private(Path::new("/dev/stdout"), b"via-dev-stdout")
                .map(|()| "Ok".to_string()),
            "W17" => write_private(Path::new("/dev/stdout"), b"via").map(|()| "Ok".to_string()),
            "W19" => write_private(Path::new("/dev/fd/1"), b"via").map(|()| "Ok".to_string()),
            "W20" => write_private(Path::new("/dev/stdout"), b"NEW\n").map(|()| "Ok".to_string()),
            "W21" => write_private(Path::new("/dev/fd/1"), b"NEW\n").map(|()| "Ok".to_string()),
            "W22" => {
                print!("before-the-write|");
                let r = write_private(Path::new("/dev/stdout"), b"via-dev-stdout\n");
                println!("after-the-write");
                r.map(|()| "Ok".to_string())
            }
            "W23" | "W24" | "W29" => write_private(&target, b"NEW\n").map(|()| "Ok".to_string()),
            "W25" => {
                std::env::set_current_dir(target.parent().unwrap()).unwrap();
                write_private(
                    &Path::new("/proc/self/cwd").join(target.file_name().unwrap()),
                    b"NEW\n",
                )
                .map(|()| "Ok".to_string())
            }
            "W28" => write_private(Path::new("/dev/fd/3"), b"NEW\n").map(|()| "Ok".to_string()),
            "W30" => write_private(Path::new("/dev/fd/01"), b"NEW\n").map(|()| "Ok".to_string()),
            "W31" => write_private(Path::new("/dev/fd/3"), b"NEW\n").map(|()| "Ok".to_string()),
            "W32" => write_private(&target, b"NEW\n").map(|()| "Ok".to_string()),
            "W33" => {
                print!("before-the-write|");
                let r = write_private(Path::new("/dev/fd/3"), b"NEW\n");
                println!("after-the-write");
                r.map(|()| "Ok".to_string())
            }
            "W34" => write_private(Path::new("/dev/fd/1/."), b"NEW\n").map(|()| "Ok".to_string()),
            "S7" => Ok(show(same_file(
                Endpoint::Path(&target),
                &[Endpoint::Stdout],
            ))),
            "S8" => Ok(show(same_file(Endpoint::Stdout, &[Endpoint::Stdin]))),
            "S11" => Ok(show(same_file(Endpoint::Path(&target), &[Endpoint::Stdin]))),
            other => panic!("unknown child row {other}"),
        };
        match r {
            Ok(v) => eprintln!("child-result: {v}"),
            Err(e) => {
                match e.raw_os_error() {
                    Some(n) => eprintln!("child-result: Err raw_os_error={n}"),
                    None => eprintln!("child-result: Err raw_os_error=none"),
                }
                std::process::exit(3);
            }
        }
    }

    const OLD_CARD: &[u8] = b"OLD-VALID-CARD\n";

    /// W1: ENOSPC-class failure (EFBIG via `ulimit -f 1`) keeps the old file.
    #[cfg(unix)]
    #[test]
    fn a_failed_write_leaves_the_old_file_byte_identical() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, OLD_CARD).unwrap();
        let out = run_child("W1", &t, Stdio::piped(), Some(true));
        let err = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(
            err.contains("child-result: Err raw_os_error=27"),
            "child stderr: {err}"
        );
        assert_eq!(out.status.code(), Some(3), "child stderr: {err}");
        assert_eq!(
            std::fs::read(&t).unwrap(),
            OLD_CARD,
            "the old file must be byte-identical"
        );
        assert_eq!(names(d.path()), vec!["card.txt".to_string()]);
    }

    /// W2: a run killed mid-write (SIGXFSZ) keeps the old file and leaves one
    /// named owner-only temp.
    #[cfg(unix)]
    #[test]
    fn a_killed_run_leaves_the_old_file_and_a_named_owner_only_leftover() {
        use std::os::unix::process::ExitStatusExt;
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, OLD_CARD).unwrap();
        let out = run_child("W2", &t, Stdio::piped(), Some(false));
        assert_eq!(out.status.signal(), Some(25), "status: {:?}", out.status);
        assert_eq!(std::fs::read(&t).unwrap(), OLD_CARD);
        let others: Vec<String> = names(d.path())
            .into_iter()
            .filter(|n| n != "card.txt")
            .collect();
        assert_eq!(others.len(), 1, "entries: {others:?}");
        let n = &others[0];
        let mid = n
            .strip_prefix("card.txt.")
            .and_then(|r| r.strip_suffix(".tmp"))
            .unwrap_or_else(|| panic!("leftover name {n}"));
        let parts: Vec<&str> = mid.split('.').collect();
        assert!(
            parts.len() == 2
                && parts
                    .iter()
                    .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())),
            "leftover name {n}"
        );
        let lp = d.path().join(n);
        assert_eq!(mode_of(&lp), 0o600);
        let body = std::fs::read(&lp).unwrap();
        assert_eq!(body.len(), 512);
        assert!(body.iter().all(|b| *b == b'N'));
    }

    /// W3: a hard-linked target is replaced at the given name only.
    #[cfg(unix)]
    #[test]
    fn a_hard_linked_target_is_replaced_at_that_name_only() {
        use std::os::unix::fs::MetadataExt;
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::write(&a, b"old").unwrap();
        std::fs::hard_link(&a, &b).unwrap();
        write_private(&a, b"new").unwrap();
        assert_eq!(std::fs::read(&a).unwrap(), b"new");
        assert_eq!(std::fs::read(&b).unwrap(), b"old");
        assert_eq!(std::fs::metadata(&a).unwrap().nlink(), 1);
        assert_eq!(std::fs::metadata(&b).unwrap().nlink(), 1);
    }

    /// W4: an unwritable directory fails closed before the old file is touched.
    #[cfg(unix)]
    #[test]
    fn an_unwritable_directory_fails_closed_before_the_old_file_is_touched() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, b"old").unwrap();
        std::fs::set_permissions(&t, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
        let probe = d.path().join("probe");
        if std::fs::File::create(&probe).is_ok() {
            std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
            panic!("precondition: the directory is still writable (running as root?)");
        }
        let before = names(d.path());
        let r = write_private(&t, b"new");
        let after = names(d.path());
        let mode = mode_of(&t);
        let bytes = std::fs::read(&t).unwrap();
        std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let e = r.expect_err("a directory the holder cannot write must refuse");
        assert_eq!(e.raw_os_error(), Some(13));
        assert_eq!(bytes, b"old");
        assert_eq!(mode, 0o644);
        assert_eq!(before, after);
    }

    /// W5: an absent target is created owner-only with no leftover.
    #[test]
    fn an_absent_target_is_created_owner_only_with_no_leftover() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        write_private(&t, b"bytes").unwrap();
        assert_eq!(std::fs::read(&t).unwrap(), b"bytes");
        #[cfg(unix)]
        assert_eq!(mode_of(&t), 0o600);
        assert_eq!(names(d.path()), vec!["card.txt".to_string()]);
    }

    /// W6: a read-only target is refused and kept.
    #[cfg(unix)]
    #[test]
    fn a_read_only_target_is_refused_and_kept() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, b"old").unwrap();
        std::fs::set_permissions(&t, std::fs::Permissions::from_mode(0o400)).unwrap();
        let e = write_private(&t, b"new").expect_err("a read-only target must refuse");
        assert_eq!(e.raw_os_error(), Some(13));
        assert_eq!(std::fs::read(&t).unwrap(), b"old");
        assert_eq!(names(d.path()), vec!["card.txt".to_string()]);
    }

    /// W7: a symlink is written through and stays a link.
    #[cfg(unix)]
    #[test]
    fn a_symlink_is_written_through_and_stays_a_link() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("A");
        let b = d.path().join("B");
        std::fs::create_dir(&a).unwrap();
        std::fs::create_dir(&b).unwrap();
        std::fs::write(b.join("real"), b"old").unwrap();
        let link = a.join("link");
        std::os::unix::fs::symlink("../B/real", &link).unwrap();
        write_private(&link, b"new").unwrap();
        assert_eq!(std::fs::read(b.join("real")).unwrap(), b"new");
        assert_eq!(mode_of(&b.join("real")), 0o600);
        assert!(std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(
            std::fs::read_link(&link).unwrap(),
            PathBuf::from("../B/real")
        );
        assert_eq!(names(&a), vec!["link".to_string()]);
        assert_eq!(names(&b), vec!["real".to_string()]);
    }

    /// W8: a dangling symlink creates its target owner-only.
    #[cfg(unix)]
    #[test]
    fn a_dangling_symlink_creates_its_target_owner_only() {
        let d = tempfile::tempdir().unwrap();
        let link = d.path().join("link");
        std::os::unix::fs::symlink("missing", &link).unwrap();
        write_private(&link, b"bytes").unwrap();
        let m = d.path().join("missing");
        assert_eq!(std::fs::read(&m).unwrap(), b"bytes");
        assert_eq!(mode_of(&m), 0o600);
        assert!(std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(
            names(d.path()),
            vec!["link".to_string(), "missing".to_string()]
        );
    }

    /// W9: a FIFO target is written directly (run under `timeout 60`: a wrong
    /// classification can hang here).
    #[cfg(unix)]
    #[test]
    fn a_fifo_target_is_written_directly() {
        use std::io::Read;
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("pipe");
        let st = Command::new("mkfifo").arg(&t).status().unwrap();
        assert!(st.success(), "mkfifo failed");
        let rp = t.clone();
        let reader = std::thread::spawn(move || {
            let mut v = Vec::new();
            std::fs::File::open(rp)
                .unwrap()
                .read_to_end(&mut v)
                .unwrap();
            v
        });
        write_private(&t, b"fifo-bytes").unwrap();
        assert_eq!(reader.join().unwrap(), b"fifo-bytes");
        assert_eq!(names(d.path()), vec!["pipe".to_string()]);
    }

    /// W10: `/dev/stdout` on a pipe is written directly.
    #[cfg(unix)]
    #[test]
    fn dev_stdout_on_a_pipe_is_written_directly() {
        let d = tempfile::tempdir().unwrap();
        let out = run_child("W10", d.path(), Stdio::piped(), None);
        let so = String::from_utf8_lossy(&out.stdout).to_string();
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(so.contains("via-dev-stdout"), "child stdout: {so}");
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
    }

    /// W11: a 250-byte file name is written.
    #[test]
    fn a_250_byte_file_name_is_written() {
        let d = tempfile::tempdir().unwrap();
        let name = "a".repeat(250);
        let t = d.path().join(&name);
        write_private(&t, b"bytes").unwrap();
        assert_eq!(std::fs::read(&t).unwrap(), b"bytes");
        assert_eq!(names(d.path()), vec![name]);
    }

    /// W17: `/dev/stdout` on a regular file is written into the open stream (same inode, 0600; since 0.1.3 never truncated, so libtest's own lines may come first).
    #[cfg(unix)]
    #[test]
    fn dev_stdout_on_a_regular_file_is_written_directly() {
        use std::os::unix::fs::MetadataExt;
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        std::fs::write(&f, b"OLD").unwrap();
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
        let ino = std::fs::metadata(&f).unwrap().ino();
        let sf = std::fs::OpenOptions::new().write(true).open(&f).unwrap();
        let out = run_child("W17", &f, Stdio::from(sf), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
        assert_eq!(std::fs::metadata(&f).unwrap().ino(), ino);
        assert!(String::from_utf8_lossy(&std::fs::read(&f).unwrap()).contains("via"));
        assert_eq!(mode_of(&f), 0o600);
        assert_eq!(names(d.path()), vec!["f".to_string()]);
    }

    /// W19: `/dev/fd/1` on a regular file is written into the open stream (same inode; never truncated).
    #[cfg(unix)]
    #[test]
    fn dev_fd_1_on_a_regular_file_is_written_directly() {
        use std::os::unix::fs::MetadataExt;
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        std::fs::write(&f, b"OLD").unwrap();
        let ino = std::fs::metadata(&f).unwrap().ino();
        let sf = std::fs::OpenOptions::new().write(true).open(&f).unwrap();
        let out = run_child("W19", &f, Stdio::from(sf), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
        assert_eq!(
            std::fs::metadata(&f).unwrap().ino(),
            ino,
            "inode replaced via /dev/fd/1"
        );
        assert!(String::from_utf8_lossy(&std::fs::read(&f).unwrap()).contains("via"));
    }

    /// W20-W24 helper: `row` with stdout = `f` opened for APPEND (the shell's
    /// `>>`), `f` holding `EARLIER\n` at 0644; returns `f`'s text.
    #[cfg(unix)]
    fn appended_run(row: &str, target: &Path, f: &Path) -> String {
        std::fs::write(f, b"EARLIER\n").unwrap();
        std::fs::set_permissions(f, std::fs::Permissions::from_mode(0o644)).unwrap();
        let sf = std::fs::OpenOptions::new().append(true).open(f).unwrap();
        let out = run_child(row, target, Stdio::from(sf), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
        String::from_utf8_lossy(&std::fs::read(f).unwrap()).to_string()
    }

    /// W20: `--out /dev/stdout >> f` keeps f's earlier lines (triage-io-lib-dev-stdout-append).
    #[cfg(unix)]
    #[test]
    fn dev_stdout_appended_to_a_file_keeps_its_earlier_lines() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let s = appended_run("W20", &f, &f);
        assert!(s.starts_with("EARLIER\n"), "earlier lines lost: {s:?}");
        assert!(s.contains("NEW\n"), "{s:?}");
        assert_eq!(mode_of(&f), 0o600);
        assert_eq!(names(d.path()), vec!["f".to_string()]);
    }

    /// W21: the same through `/dev/fd/1`.
    #[cfg(unix)]
    #[test]
    fn dev_fd_1_appended_to_a_file_keeps_its_earlier_lines() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let s = appended_run("W21", &f, &f);
        assert!(s.starts_with("EARLIER\n"), "earlier lines lost: {s:?}");
        assert!(s.contains("NEW\n"), "{s:?}");
        assert_eq!(names(d.path()), vec!["f".to_string()]);
    }

    /// W22: `{ printf before; tool --out /dev/stdout; echo after; } > f` keeps all three, in order.
    #[cfg(unix)]
    #[test]
    fn dev_stdout_keeps_the_stream_order_in_a_redirected_file() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let sf = std::fs::File::create(&f).unwrap();
        let out = run_child("W22", &f, Stdio::from(sf), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
        let s = String::from_utf8_lossy(&std::fs::read(&f).unwrap()).to_string();
        let at = |m: &str| {
            s.find(m)
                .unwrap_or_else(|| panic!("{m:?} missing from {s:?}"))
        };
        assert!(at("before-the-write|") < at("via-dev-stdout\n"), "{s:?}");
        assert!(at("via-dev-stdout\n") < at("after-the-write\n"), "{s:?}");
    }

    /// W23: a symlink to `/dev/stdout` is the alias, not a file to truncate.
    #[cfg(unix)]
    #[test]
    fn a_link_to_dev_stdout_appended_to_a_file_keeps_its_earlier_lines() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let l = d.path().join("l");
        std::os::unix::fs::symlink("/dev/stdout", &l).unwrap();
        let s = appended_run("W23", &l, &f);
        assert!(s.starts_with("EARLIER\n"), "earlier lines lost: {s:?}");
        assert!(s.contains("NEW\n"), "{s:?}");
        assert!(std::fs::symlink_metadata(&l)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(names(d.path()), vec!["f".to_string(), "l".to_string()]);
    }

    /// W24: `d/1` with `d` -> `/dev/fd` is the alias it spells, not ATOMIC over stdout's file.
    #[cfg(unix)]
    #[test]
    fn a_name_under_a_link_to_dev_fd_appended_to_a_file_keeps_its_earlier_lines() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let fd = d.path().join("fd");
        std::os::unix::fs::symlink("/dev/fd", &fd).unwrap();
        let s = appended_run("W24", &fd.join("1"), &f);
        assert!(s.starts_with("EARLIER\n"), "earlier lines lost: {s:?}");
        assert!(s.contains("NEW\n"), "{s:?}");
        assert_eq!(names(d.path()), vec!["f".to_string(), "fd".to_string()]);
    }

    /// W25-W27 helper: `f` holds `OLD_CARD` at 0644; returns its inode.
    #[cfg(target_os = "linux")]
    fn old_card(f: &Path) -> u64 {
        use std::os::unix::fs::MetadataExt;
        std::fs::write(f, OLD_CARD).unwrap();
        std::fs::set_permissions(f, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::fs::metadata(f).unwrap().ino()
    }

    /// W25-W27 helper: `f` is exactly the new bytes, 0600, and a new inode (temp+rename).
    #[cfg(target_os = "linux")]
    fn assert_replaced(f: &Path, ino: u64) {
        use std::os::unix::fs::MetadataExt;
        assert_eq!(
            String::from_utf8_lossy(&std::fs::read(f).unwrap()),
            "NEW\n",
            "not replaced by exactly the new bytes"
        );
        assert_eq!(mode_of(f), 0o600);
        assert_ne!(
            std::fs::metadata(f).unwrap().ino(),
            ino,
            "written in place, not temp+rename"
        );
    }

    /// W25: a `/proc/self/cwd/...` path is a file name, not an alias: ATOMIC.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_proc_self_cwd_path_onto_a_file_is_replaced_atomically() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let ino = old_card(&f);
        let out = run_child("W25", &f, Stdio::null(), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
        assert_replaced(&f, ino);
        assert_eq!(names(d.path()), vec!["f".to_string()]);
    }

    /// W26: a `/proc/self/root/...` path is a file name, not an alias: ATOMIC.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_proc_self_root_path_onto_a_file_is_replaced_atomically() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let ino = old_card(&f);
        write_private(
            &Path::new("/proc/self/root").join(f.strip_prefix("/").unwrap()),
            b"NEW\n",
        )
        .unwrap();
        assert_replaced(&f, ino);
        assert_eq!(names(d.path()), vec!["f".to_string()]);
    }

    /// W27: a link to a `/proc/self/root/...` path is walked to the real file, which is replaced.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_link_to_a_proc_self_root_path_is_written_through_and_stays_a_link() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let l = d.path().join("l");
        let ino = old_card(&f);
        std::os::unix::fs::symlink(
            Path::new("/proc/self/root").join(f.strip_prefix("/").unwrap()),
            &l,
        )
        .unwrap();
        write_private(&l, b"NEW\n").unwrap();
        assert_replaced(&f, ino);
        assert!(std::fs::symlink_metadata(&l)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(names(d.path()), vec!["f".to_string(), "l".to_string()]);
    }

    /// W28: `{ echo a >&3; tool --out /dev/fd/3; echo b >&3; } 3> f` keeps all three, in order.
    #[cfg(unix)]
    #[test]
    fn dev_fd_3_keeps_the_stream_order_in_a_redirected_file() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        std::fs::write(&f, b"EARLIER\n").unwrap();
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
        let out = Command::new("sh")
            .arg("-c")
            .arg("{ echo a >&3; \"$0\" --exact write::tests::child --nocapture; echo b >&3; } 3>\"$1\"")
            .arg(std::env::current_exe().unwrap())
            .arg(&f)
            .env("MNEMONIC_IO_LIB_CHILD", "W28")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
        assert_eq!(
            String::from_utf8_lossy(&std::fs::read(&f).unwrap()),
            "a\nNEW\nb\n"
        );
        assert_eq!(mode_of(&f), 0o600);
    }

    /// W29: `d/1` with `d` -> `/proc/thread-self/fd` is the alias it spells (guard).
    #[cfg(target_os = "linux")]
    #[test]
    fn a_name_under_a_link_to_proc_thread_self_fd_appended_to_a_file_keeps_its_earlier_lines() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let fd = d.path().join("fd");
        std::os::unix::fs::symlink("/proc/thread-self/fd", &fd).unwrap();
        let s = appended_run("W29", &fd.join("1"), &f);
        assert!(s.starts_with("EARLIER\n"), "earlier lines lost: {s:?}");
        assert!(s.contains("NEW\n"), "{s:?}");
        assert_eq!(names(d.path()), vec!["f".to_string(), "fd".to_string()]);
    }

    /// W30: a descriptor name the kernel lacks (`/dev/fd/01`) is not an alias: the normal path's ENOENT.
    #[cfg(unix)]
    #[test]
    fn a_descriptor_name_the_kernel_lacks_is_refused() {
        let out = run_child("W30", Path::new(""), Stdio::piped(), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(
            se.contains("child-result: Err raw_os_error=2"),
            "child stderr: {se}"
        );
        assert!(!String::from_utf8_lossy(&out.stdout).contains("NEW"));
    }

    /// W31: a descriptor open read-only (`3< f`) is refused before its file's mode changes.
    #[cfg(unix)]
    #[test]
    fn a_read_only_descriptor_is_refused_before_its_mode_changes() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        std::fs::write(&f, OLD_CARD).unwrap();
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
        let out = Command::new("sh")
            .arg("-c")
            .arg("\"$0\" --exact write::tests::child --nocapture 3<\"$1\"")
            .arg(std::env::current_exe().unwrap())
            .arg(&f)
            .env("MNEMONIC_IO_LIB_CHILD", "W31")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(
            se.contains("child-result: Err raw_os_error=9"),
            "child stderr: {se}"
        );
        assert_eq!(std::fs::read(&f).unwrap(), OLD_CARD);
        assert_eq!(mode_of(&f), 0o644, "an input's mode was changed");
    }

    /// W32: in a pid namespace that kept its parent's `/proc` (getpid() is 1,
    /// `/proc/self` is the outer pid), `d/1` with `d` -> `/dev/fd` is still the
    /// alias. Needs unprivileged user namespaces; says SKIP when they are off.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_name_under_a_link_to_dev_fd_in_a_pid_namespace_keeps_its_earlier_lines() {
        let ok = Command::new("unshare")
            .args(["-Urpf", "--kill-child", "true"])
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            eprintln!("W32 SKIP: unshare -Urpf unavailable");
            return;
        }
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let fd = d.path().join("fd");
        std::os::unix::fs::symlink("/dev/fd", &fd).unwrap();
        std::fs::write(&f, b"EARLIER\n").unwrap();
        let out = Command::new("unshare")
            .args(["-Urpf", "--kill-child", "sh", "-c"])
            .arg("\"$0\" --exact write::tests::child --nocapture >>\"$1\"")
            .arg(std::env::current_exe().unwrap())
            .arg(&f)
            .env("MNEMONIC_IO_LIB_CHILD", "W32")
            .env("MNEMONIC_IO_LIB_TARGET", fd.join("1"))
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
        let s = String::from_utf8_lossy(&std::fs::read(&f).unwrap()).to_string();
        assert!(s.starts_with("EARLIER\n"), "earlier lines lost: {s:?}");
        assert!(s.contains("NEW\n"), "{s:?}");
    }

    /// W33: `tool --out /dev/fd/3 >f 3>&1` keeps the stream order: stdout is flushed for every N.
    #[cfg(unix)]
    #[test]
    fn dev_fd_3_onto_stdouts_file_keeps_the_stream_order() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let out = Command::new("sh")
            .arg("-c")
            .arg("\"$0\" --exact write::tests::child --nocapture >\"$1\" 3>&1")
            .arg(std::env::current_exe().unwrap())
            .arg(&f)
            .env("MNEMONIC_IO_LIB_CHILD", "W33")
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Ok"), "child stderr: {se}");
        let s = String::from_utf8_lossy(&std::fs::read(&f).unwrap()).to_string();
        assert!(
            s.contains("before-the-write|NEW\nafter-the-write"),
            "order lost: {s:?}"
        );
    }

    /// W34: `/dev/fd/1/.`, a name the kernel lacks (ENOTDIR), is not an alias.
    #[cfg(unix)]
    #[test]
    fn a_descriptor_name_with_a_trailing_dot_is_refused() {
        let out = run_child("W34", Path::new(""), Stdio::piped(), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(
            se.contains("child-result: Err raw_os_error=20"),
            "child stderr: {se}"
        );
        assert!(!String::from_utf8_lossy(&out.stdout).contains("NEW"));
    }

    /// W35: another process's `/proc/<pid>/fd/N` is a file name, never this process's descriptor N.
    #[cfg(target_os = "linux")]
    #[test]
    fn another_processs_descriptor_path_is_not_an_alias() {
        use std::io::{BufRead, BufReader};
        use std::os::unix::io::AsRawFd;
        let d = tempfile::tempdir().unwrap();
        let (f, g) = (d.path().join("f"), d.path().join("g"));
        std::fs::write(&f, OLD_CARD).unwrap();
        let h = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&g)
            .unwrap();
        let n = h.as_raw_fd();
        assert!(n <= 9, "sh redirects only 0-9, got {n}");
        let mut c = Command::new("sh")
            .arg("-c")
            .arg(format!("exec {n}>>\"$0\"; echo ready >&2; read x"))
            .arg(&f)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut line = String::new();
        BufReader::new(c.stderr.take().unwrap())
            .read_line(&mut line)
            .unwrap();
        assert_eq!(line, "ready\n");
        let r = write_private(Path::new(&format!("/proc/{}/fd/{n}", c.id())), b"NEW\n");
        let _ = c.kill();
        let _ = c.wait();
        assert!(r.is_ok(), "{r:?}");
        assert_eq!(
            std::fs::read(&g).unwrap(),
            b"",
            "this process's fd {n} was written"
        );
        assert_eq!(
            std::fs::read(&f).unwrap(),
            b"NEW\n",
            "the file behind the name was not replaced"
        );
    }

    /// W36: write_private through `/dev/fd/N` leaves the caller's descriptor open (a dup, never a close).
    #[cfg(unix)]
    #[test]
    fn an_alias_write_leaves_the_callers_descriptor_open() {
        use std::os::unix::io::AsRawFd;
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        let h = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&f)
            .unwrap();
        let n = h.as_raw_fd();
        let mut spellings = vec![format!("/dev/fd/{n}")];
        #[cfg(target_os = "linux")]
        {
            let pid = std::process::id();
            let ts = std::fs::read_link("/proc/thread-self").unwrap();
            spellings.extend([
                format!("/proc/self/fd/{n}"),
                format!("/proc/thread-self/fd/{n}"),
                format!("/proc/{pid}/fd/{n}"),
                format!(
                    "/proc/self/task/{}/fd/{n}",
                    ts.file_name().unwrap().to_str().unwrap()
                ),
                format!("/proc/{}/fd/{n}", ts.display()),
            ]);
        }
        let mut want = Vec::new();
        for p in spellings {
            write_private(Path::new(&p), p.as_bytes()).unwrap();
            want.extend_from_slice(p.as_bytes());
            assert!(
                h.metadata().is_ok(),
                "descriptor closed by write_private via {p}"
            );
            assert_eq!(std::fs::read(&f).unwrap(), want, "{p}");
        }
    }

    /// **THE GATE, and the pre-existing target is the whole of it.**
    ///
    /// The fresh-file half passes under any implementation that sets a mode at
    /// all, so on its own it proves nothing. The `0644` half is what fails
    /// against `OpenOptions::mode()` alone — measured: without the
    /// `set_permissions` call this assertion reports `0o644` — which makes it a
    /// gate on F-244 rather than a restatement of the doc comment.
    ///
    /// It also pins the CONTENTS, because a function that tightened the mode and
    /// wrote nothing would satisfy a permissions-only test.
    #[cfg(unix)]
    #[test]
    fn an_existing_world_readable_target_is_tightened_not_inherited() {
        let d = tempfile::tempdir().unwrap();

        let fresh = d.path().join("fresh.bin");
        write_private(&fresh, b"new bytes").unwrap();
        assert_eq!(
            mode_of(&fresh),
            0o600,
            "a file this function creates is owner-only"
        );

        let stale = d.path().join("stale.bin");
        std::fs::write(&stale, b"old").unwrap();
        std::fs::set_permissions(&stale, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            mode_of(&stale),
            0o644,
            "the control: the target really is 0644 before the call"
        );

        write_private(&stale, b"new bytes").unwrap();
        assert_eq!(
            mode_of(&stale),
            0o600,
            "F-244: `0o600` binds on CREATE, so an implementation that only passes \
             it to OpenOptions leaves this at 0644 and reports success"
        );
        assert_eq!(
            std::fs::read(&stale).unwrap(),
            b"new bytes",
            "the new contents must be there -- tightening a file this function \
             failed to write would pass a mode-only assertion"
        );
    }

    /// A **shrinking** overwrite leaves no tail of the previous file.
    ///
    /// Dropping the `set_len(0)` is the tidy-up that looks harmless: the mode
    /// assertions above all still pass, and a smaller JSON manifest written over
    /// a larger one silently acquires trailing bytes after its closing brace.
    #[cfg(unix)]
    #[test]
    fn a_shrinking_overwrite_leaves_no_stale_tail() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("manifest.json");
        write_private(&p, b"{\"a\":1,\"padding\":\"xxxxxxxxxxxxxxxxxxxx\"}").unwrap();
        write_private(&p, b"{\"a\":1}").unwrap();
        assert_eq!(
            std::fs::read(&p).unwrap(),
            b"{\"a\":1}",
            "without set_len(0) the tail of the longer write survives"
        );
    }

    /// W12: the temp name is the target name or the fallback.
    #[test]
    fn the_temp_name_is_the_target_name_or_the_fallback() {
        let n200 = "a".repeat(200);
        let n201 = "a".repeat(201);
        assert_eq!(
            temp_name(std::ffi::OsStr::new(&n200), 7, 3),
            std::ffi::OsString::from(format!("{n200}.7.3.tmp"))
        );
        assert_eq!(
            temp_name(std::ffi::OsStr::new(&n201), 7, 3),
            std::ffi::OsString::from("mnemonic-io-lib.7.3.tmp")
        );
    }

    /// W13: a failure at each step removes the temp and keeps the old file.
    #[test]
    fn a_failure_at_each_step_removes_the_temp_and_keeps_the_old_file() {
        #[cfg(unix)]
        let steps = [
            Step::CreateTemp,
            Step::Preflight,
            Step::Chmod,
            Step::Write,
            Step::Sync,
            Step::Rename,
        ];
        #[cfg(not(unix))]
        let steps = [
            Step::CreateTemp,
            Step::Preflight,
            Step::Write,
            Step::Sync,
            Step::Rename,
        ];
        for step in steps {
            let d = tempfile::tempdir().unwrap();
            let t = d.path().join("card.txt");
            std::fs::write(&t, b"old").unwrap();
            FAULT.with(|c| c.set(Some(step)));
            let r = write_private(&t, b"new");
            FAULT.with(|c| c.set(None));
            let e = r.expect_err("the injected step must fail");
            assert_eq!(e.to_string(), format!("injected at {step:?}"));
            assert_eq!(std::fs::read(&t).unwrap(), b"old", "{step:?}");
            assert_eq!(names(d.path()), vec!["card.txt".to_string()], "{step:?}");
        }
    }

    /// W14: a directory-sync failure after the rename is not reported.
    #[cfg(unix)]
    #[test]
    fn a_directory_sync_failure_after_the_rename_is_not_reported() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, b"old").unwrap();
        FAULT.with(|c| c.set(Some(Step::DirSync)));
        let r = write_private(&t, b"new");
        FAULT.with(|c| c.set(None));
        r.unwrap();
        assert_eq!(std::fs::read(&t).unwrap(), b"new");
        assert_eq!(names(d.path()), vec!["card.txt".to_string()]);
    }

    /// W18: ENOTSUP from the full sync falls back to a plain fsync on Apple
    /// targets only; every other error, and every error elsewhere, stands.
    #[test]
    fn enotsup_from_the_full_sync_falls_back_to_fsync_on_apple_only() {
        let os = |n| Err(std::io::Error::from_raw_os_error(n));
        let called = Cell::new(0);
        let plain = |r: std::io::Result<()>| {
            let c = &called;
            move || {
                c.set(c.get() + 1);
                r
            }
        };
        let code = |r: std::io::Result<()>| r.err().and_then(|e| e.raw_os_error());
        // Apple, ENOTSUP: the plain fsync runs and its result stands.
        assert!(full_sync_or_plain(true, os(45), plain(Ok(()))).is_ok());
        assert_eq!(
            code(full_sync_or_plain(true, os(45), plain(os(5)))),
            Some(5)
        );
        assert_eq!(called.get(), 2);
        // Apple, any other error or success: no fallback.
        assert_eq!(
            code(full_sync_or_plain(true, os(5), plain(Ok(())))),
            Some(5)
        );
        assert_eq!(
            code(full_sync_or_plain(true, os(22), plain(Ok(())))),
            Some(22)
        );
        assert!(full_sync_or_plain(true, Ok(()), plain(os(5))).is_ok());
        // Elsewhere, ENOTSUP is an error like any other.
        assert_eq!(
            code(full_sync_or_plain(false, os(45), plain(Ok(())))),
            Some(45)
        );
        assert_eq!(called.get(), 2);
        // The real sync on this platform succeeds on a local file.
        let d = tempfile::tempdir().unwrap();
        let f = File::create(d.path().join("t")).unwrap();
        sync_temp(&f).unwrap();
    }

    /// W15: a stale leftover is skipped and left alone.
    #[cfg(unix)]
    #[test]
    fn a_stale_leftover_is_skipped_and_left_alone() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, b"old").unwrap();
        let stale = d
            .path()
            .join(format!("card.txt.{}.0.tmp", std::process::id()));
        std::fs::write(&stale, b"stale").unwrap();
        write_private(&t, b"new").unwrap();
        assert_eq!(std::fs::read(&t).unwrap(), b"new");
        assert_eq!(std::fs::read(&stale).unwrap(), b"stale");
    }

    /// W16: the temp-name search stops after 100 taken names.
    #[cfg(unix)]
    #[test]
    fn the_temp_name_search_stops_after_100_taken_names() {
        let d = tempfile::tempdir().unwrap();
        let t = d.path().join("card.txt");
        std::fs::write(&t, b"old").unwrap();
        let pid = std::process::id();
        for n in 0..100 {
            std::fs::write(d.path().join(format!("card.txt.{pid}.{n}.tmp")), b"taken").unwrap();
        }
        let e = write_private(&t, b"new").expect_err("100 taken names must refuse");
        assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&t).unwrap(), b"old");
        for n in 0..100 {
            let f = d.path().join(format!("card.txt.{pid}.{n}.tmp"));
            assert_eq!(std::fs::read(&f).unwrap(), b"taken");
        }
        assert_eq!(names(d.path()).len(), 101);
    }

    /// S1: the same path twice is the same file.
    #[test]
    fn the_same_path_twice_is_the_same_file() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("f");
        std::fs::write(&p, b"x").unwrap();
        assert_eq!(
            same_file(Endpoint::Path(&p), &[Endpoint::Path(&p)]),
            Some(0)
        );
    }

    /// S2: a hard link is the same file.
    #[cfg(unix)]
    #[test]
    fn a_hard_link_is_the_same_file() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::write(&a, b"x").unwrap();
        std::fs::hard_link(&a, &b).unwrap();
        assert_eq!(
            same_file(Endpoint::Path(&a), &[Endpoint::Path(&b)]),
            Some(0)
        );
    }

    /// S3: a symlink is the same file as its target.
    #[test]
    fn a_symlink_is_the_same_file() {
        let d = tempfile::tempdir().unwrap();
        let real = d.path().join("real");
        std::fs::write(&real, b"x").unwrap();
        #[cfg(unix)]
        {
            let link = d.path().join("link");
            std::os::unix::fs::symlink("real", &link).unwrap();
            assert_eq!(
                same_file(Endpoint::Path(&link), &[Endpoint::Path(&real)]),
                Some(0)
            );
        }
    }

    /// S4: two files with equal bytes are not the same file.
    #[test]
    fn two_files_with_equal_bytes_are_not_the_same() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::write(&a, b"same").unwrap();
        std::fs::write(&b, b"same").unwrap();
        assert_eq!(same_file(Endpoint::Path(&a), &[Endpoint::Path(&b)]), None);
    }

    /// S5: an absent output matches an absent input at the same resolved path.
    #[test]
    fn an_absent_output_matches_an_absent_input_at_the_same_resolved_path() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path().join("dir");
        std::fs::create_dir(&dir).unwrap();
        let out = dir.join(".").join("x");
        let y = dir.join("y");
        let x = dir.join("x");
        assert_eq!(
            same_file(
                Endpoint::Path(&out),
                &[Endpoint::Path(&y), Endpoint::Path(&x)]
            ),
            Some(1)
        );
        assert_eq!(same_file(Endpoint::Path(&out), &[Endpoint::Path(&y)]), None);
    }

    /// S6: the first matching input is reported.
    #[test]
    fn the_first_matching_input_is_reported() {
        let d = tempfile::tempdir().unwrap();
        let out = d.path().join("out");
        let other = d.path().join("other");
        std::fs::write(&out, b"x").unwrap();
        std::fs::write(&other, b"y").unwrap();
        assert_eq!(
            same_file(
                Endpoint::Path(&out),
                &[
                    Endpoint::Path(&other),
                    Endpoint::Path(&out),
                    Endpoint::Path(&out)
                ]
            ),
            Some(1)
        );
    }

    /// S7: stdout redirected to the output file is the same file.
    #[cfg(unix)]
    #[test]
    fn stdout_redirected_to_the_output_file_is_the_same_file() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        std::fs::write(&f, b"x").unwrap();
        let sf = std::fs::OpenOptions::new().write(true).open(&f).unwrap();
        let out = run_child("S7", &f, Stdio::from(sf), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Some(0)"), "child stderr: {se}");
    }

    /// S8: stdin and stdout on pipes never match.
    #[cfg(unix)]
    #[test]
    fn stdin_and_stdout_on_pipes_never_match() {
        let d = tempfile::tempdir().unwrap();
        let out = run_child("S8", d.path(), Stdio::piped(), None);
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: None"), "child stderr: {se}");
    }

    /// S11: stdin redirected from the output file is the same file.
    #[cfg(unix)]
    #[test]
    fn stdin_redirected_from_the_output_file_is_the_same_file() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("f");
        std::fs::write(&f, b"x").unwrap();
        let sf = std::fs::File::open(&f).unwrap();
        let exe = std::env::current_exe().unwrap();
        let out = Command::new(exe)
            .args(["--exact", "write::tests::child", "--nocapture"])
            .env("MNEMONIC_IO_LIB_CHILD", "S11")
            .env("MNEMONIC_IO_LIB_TARGET", &f)
            .stdin(Stdio::from(sf))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        let se = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(se.contains("child-result: Some(0)"), "child stderr: {se}");
    }

    /// S9: a non-regular path never matches (metadata only, never written).
    #[cfg(unix)]
    #[test]
    fn a_non_regular_path_never_matches() {
        let n = Path::new("/dev/null");
        assert_eq!(same_file(Endpoint::Path(n), &[Endpoint::Path(n)]), None);
    }

    /// S10: an unreadable lookup never matches.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_lookup_never_matches() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path().join("d");
        std::fs::create_dir(&dir).unwrap();
        let x = dir.join("x");
        std::fs::write(&x, b"x").unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o000)).unwrap();
        if std::fs::metadata(&x).is_ok() {
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
            panic!("precondition: the directory is still readable (running as root?)");
        }
        let r = same_file(Endpoint::Path(&x), &[Endpoint::Path(&x)]);
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(r, None);
    }
}
