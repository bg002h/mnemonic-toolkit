//! `mnemonic` — engraving-bundle CLI for the m-format star. The code lives in the
//! library (`mnemonic_toolkit::run`); see `src/cli.rs`.

fn main() -> std::process::ExitCode {
    mnemonic_toolkit::run()
}

/// A closed fd 0, 1 or 2 at exec (`mnemonic ... >&-`). std's runtime opens `/dev/null` read-write on each
/// closed standard fd before `main`, so by `main` a closed stdout looks like a writable one and its output
/// is lost with exit 0. This runs earlier, from the executable's initializer list: it records a closed
/// fd 1 (the stdout sink then reports `EBADF` on the first write, like a read-only stdout; a verb that
/// writes nothing to stdout is unaffected) and plugs each closed fd with `/dev/null` itself (the lowest
/// free fd, so it lands on that number and stays open for the whole run), so no file `mnemonic` opens
/// later (an `--output` file) can become stdout. Same probe as ms-cli's and mt-cli's. A target without
/// such a list keeps std's plug only.
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly",
    target_os = "macos"
))]
mod closed_fds {
    extern "C" fn probe() {
        for fd in 0..=2 {
            // SAFETY: fcntl(F_GETFD) and open() of a NUL-terminated literal take no other pointer; a bad fd
            // or a failed open returns -1.
            unsafe {
                if libc::fcntl(fd, libc::F_GETFD) != -1
                    || std::io::Error::last_os_error().raw_os_error() != Some(libc::EBADF)
                {
                    continue;
                }
                if fd == 1 {
                    mnemonic_toolkit::mark_stdout_closed_at_start();
                }
                let n = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
                if n >= 0 && n != fd {
                    libc::close(n);
                }
            }
        }
    }

    #[used]
    #[cfg_attr(not(target_os = "macos"), link_section = ".init_array")]
    #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
    static PROBE: extern "C" fn() = probe;
}
