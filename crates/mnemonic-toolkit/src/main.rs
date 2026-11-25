//! `mnemonic` — engraving-bundle CLI for the m-format star. The code lives in the
//! library (`mnemonic_toolkit::run`); see `src/cli.rs`.

use mnemonic_toolkit::zero_free::ZeroFree;
use std::alloc::System;

/// Every block the binary frees is zeroed first (secmn T2).
#[global_allocator]
static ALLOC: ZeroFree<System> = ZeroFree(System);

fn main() -> std::process::ExitCode {
    mnemonic_toolkit::run()
}

/// The binary's allocator row: this test binary is built from this file, so a
/// plain `Vec` goes through the installed global allocator. The freed block
/// is read back through `/proc/self/mem` (a file read, not a Rust read of
/// freed memory), with a live guard block after it so it stays a heap chunk;
/// bytes `[32, N)` are checked, clear of glibc's free-list words. Linux only
/// (`/proc`).
#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::fs::File;
    use std::hint::black_box;
    use std::os::unix::fs::FileExt;

    const N: usize = 4096;
    const MARK: u8 = 0xA5;

    /// A block the binary frees holds none of its bytes past glibc's words.
    #[test]
    fn zf_main_installs_zero_free() {
        let mem = File::open("/proc/self/mem").unwrap();
        let block = black_box(vec![MARK; N]);
        let guard = black_box(vec![1u8; 64]);
        let addr = block.as_ptr() as usize;
        drop(block);
        let mut buf = [0u8; N];
        mem.read_exact_at(&mut buf, addr as u64).unwrap();
        drop(guard);
        let left = buf[32..].iter().filter(|&&b| b == MARK).count();
        assert_eq!(left, 0, "freed block keeps {left} marker bytes");
    }
}
