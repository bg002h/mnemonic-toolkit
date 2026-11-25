//! The zero-on-free allocator (secmn tk-3; architect-secret-mn-decisions-1124 T2;
//! a port of refugium-wallet crates/mr-gui/src/zero_free.rs@3965e5f7).
//! `ZeroFree` wraps an allocator: it writes zeros over every block before
//! it frees it, and resizes a block by allocating the new one, copying the
//! kept bytes, then zeroing and freeing the old one (never in place). So a
//! copy of a secret that Rust code in the toolkit makes and frees (the CLI,
//! clap, bip38/scrypt, serde_json) does not stay in freed memory. `main.rs`
//! of the `mnemonic` binary installs `ZeroFree<System>` as the global
//! allocator. It does not wipe live blocks, the stack, or memory that C or
//! C++ code allocates outside it.

use std::alloc::{GlobalAlloc, Layout};

/// An allocator that zeroes every block it frees and moves every resize.
pub struct ZeroFree<A: GlobalAlloc>(pub A);

/// Writes zeros over the `len` bytes at `p`. The write is kept: the
/// optimization barrier makes the compiler treat the bytes as read after it,
/// so it cannot drop the write as dead before the free (the same memset plus
/// barrier glibc's `explicit_bzero` is).
///
/// # Safety
/// `p` holds `len` writable bytes.
unsafe fn wipe(p: *mut u8, len: usize) {
    // SAFETY: the caller guarantees `p` holds `len` writable bytes.
    unsafe { std::ptr::write_bytes(p, 0, len) };
    // SAFETY: as above; the bytes were just written, so they are initialized.
    zeroize::optimization_barrier(unsafe { std::slice::from_raw_parts(p, len) });
}

// SAFETY: `alloc` and `alloc_zeroed` defer to the inner allocator; `dealloc`
// only writes zeros into the block it is given before the inner allocator
// frees it; `realloc` is the inner `alloc`, a copy of the kept bytes and this
// `dealloc`, and a failed `alloc` leaves the old block live and unchanged.
unsafe impl<A: GlobalAlloc> GlobalAlloc for ZeroFree<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's guarantees for `layout` pass through.
        unsafe { self.0.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's guarantees for `layout` pass through.
        unsafe { self.0.alloc_zeroed(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` is a live block of this allocator with `layout`, so it
        // holds `layout.size()` writable bytes until it is freed below.
        unsafe { wipe(ptr, layout.size()) };
        // SAFETY: `ptr` was allocated by the inner allocator with `layout`.
        unsafe { self.0.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller guarantees `new_size` is non-zero and, rounded up
        // to `layout.align()`, fits an isize; `layout.align()` is valid.
        let new = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        // SAFETY: `new` has a non-zero size.
        let q = unsafe { self.0.alloc(new) };
        if !q.is_null() {
            // SAFETY: `ptr` holds `layout.size()` bytes and `q` holds
            // `new_size`; both are live, so they do not overlap.
            unsafe { std::ptr::copy_nonoverlapping(ptr, q, layout.size().min(new_size)) };
            // SAFETY: `ptr` is a live block of this allocator with `layout`.
            unsafe { self.dealloc(ptr, layout) };
        }
        // A null `q` leaves `ptr` live and unchanged, as `GlobalAlloc` requires.
        q
    }
}

/// The zero-on-free rows . Each row
/// drives a `ZeroFree` over a recording allocator that, at its own `dealloc`,
/// counts the nonzero bytes of the WHOLE block it is given, before it frees
/// it: so every byte of the block is checked, the first ones included.
#[cfg(test)]
mod tests {
    use super::ZeroFree;
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    const N: usize = 4096;
    const MARK: u8 = 0xA5;

    /// The inner allocator: `System`, recording every free and every resize.
    #[derive(Default)]
    struct Rec {
        /// When set, every `alloc` fails (returns null).
        fail: AtomicBool,
        /// Blocks freed.
        frees: AtomicUsize,
        /// Size of the last block freed.
        last_size: AtomicUsize,
        /// Nonzero bytes the last freed block held when it was freed.
        last_nonzero: AtomicUsize,
        /// Calls of this allocator's `realloc`.
        reallocs: AtomicUsize,
    }

    // SAFETY: every block comes from `System`; `dealloc` reads the block's
    // `layout.size()` bytes, all written by the test, before it frees it.
    unsafe impl GlobalAlloc for Rec {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            if self.fail.load(Ordering::SeqCst) {
                return std::ptr::null_mut();
            }
            // SAFETY: the caller's guarantees for `layout` pass through.
            unsafe { System.alloc(layout) }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            // SAFETY: `ptr` is live with `layout.size()` initialized bytes.
            let block = unsafe { std::slice::from_raw_parts(ptr, layout.size()) };
            let nonzero = block.iter().filter(|&&b| b != 0).count();
            self.last_size.store(layout.size(), Ordering::SeqCst);
            self.last_nonzero.store(nonzero, Ordering::SeqCst);
            self.frees.fetch_add(1, Ordering::SeqCst);
            // SAFETY: `ptr` was allocated by `System` with `layout`.
            unsafe { System.dealloc(ptr, layout) }
        }
        /// A resize as an allocator that moves does it: a new block, the kept
        /// bytes copied, the old block freed (and so checked) as it is.
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            self.reallocs.fetch_add(1, Ordering::SeqCst);
            // SAFETY: the caller's guarantees pass through.
            let new = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
            // SAFETY: `new` has a non-zero size.
            let q = unsafe { self.alloc(new) };
            if !q.is_null() {
                // SAFETY: both blocks are live and hold at least the copied bytes.
                unsafe { std::ptr::copy_nonoverlapping(ptr, q, layout.size().min(new_size)) };
                // SAFETY: `ptr` is live with `layout`.
                unsafe { self.dealloc(ptr, layout) };
            }
            q
        }
    }

    /// A live block of `N` bytes of `MARK` from `zf`.
    fn marked(zf: &ZeroFree<Rec>) -> (*mut u8, Layout) {
        let l = Layout::from_size_align(N, 8).unwrap();
        // SAFETY: non-zero size.
        let p = unsafe { zf.alloc(l) };
        assert!(!p.is_null());
        // SAFETY: `p` holds `N` writable bytes.
        unsafe { p.write_bytes(MARK, N) };
        (p, l)
    }

    /// Whether the `len` bytes at `p` are all `MARK`.
    fn all_marked(p: *const u8, len: usize) -> bool {
        // SAFETY: the caller passes a live block of at least `len` bytes.
        unsafe { std::slice::from_raw_parts(p, len) }
            .iter()
            .all(|&b| b == MARK)
    }

    /// A freed block holds no byte of what it held: every byte reads zero
    /// when the inner allocator frees it.
    #[test]
    fn zf_free_zeroes_whole_block() {
        let zf = ZeroFree(Rec::default());
        let (p, l) = marked(&zf);
        // SAFETY: `p` is a live block of `zf` with `l`.
        unsafe { zf.dealloc(p, l) };
        let (size, left) = (
            zf.0.last_size.load(Ordering::SeqCst),
            zf.0.last_nonzero.load(Ordering::SeqCst),
        );
        assert_eq!(zf.0.frees.load(Ordering::SeqCst), 1, "frees");
        assert_eq!(size, N, "freed size");
        assert_eq!(left, 0, "freed block keeps {left} of {N} bytes nonzero");
    }

    /// A grown block moves and keeps its bytes; the old block reads zero
    /// when it is freed, and the inner allocator never resizes.
    #[test]
    fn zf_realloc_grow_zeroes_old() {
        let zf = ZeroFree(Rec::default());
        let (p, l) = marked(&zf);
        // SAFETY: `p` is a live block of `zf` with `l`; the size is non-zero.
        let q = unsafe { zf.realloc(p, l, 16 * N) };
        assert!(!q.is_null(), "the grow failed");
        let left = zf.0.last_nonzero.load(Ordering::SeqCst);
        let (frees, reallocs) = (
            zf.0.frees.load(Ordering::SeqCst),
            zf.0.reallocs.load(Ordering::SeqCst),
        );
        let kept = all_marked(q, N);
        // SAFETY: `q` is a live block of `zf` with the grown layout.
        unsafe { zf.dealloc(q, Layout::from_size_align(16 * N, 8).unwrap()) };
        assert_eq!(frees, 1, "the old block was not freed");
        assert_eq!(left, 0, "old block keeps {left} of {N} bytes nonzero");
        assert_eq!(reallocs, 0, "the inner allocator resized the block");
        assert!(kept, "contents not copied");
    }

    /// A shrunk block moves and keeps its head; the old block reads zero
    /// when it is freed, its tail included, and the inner allocator never
    /// resizes (an in-place shrink would leave the tail unzeroed).
    #[test]
    fn zf_realloc_shrink_zeroes_old() {
        let zf = ZeroFree(Rec::default());
        let (p, l) = marked(&zf);
        // SAFETY: `p` is a live block of `zf` with `l`; the size is non-zero.
        let q = unsafe { zf.realloc(p, l, 100) };
        assert!(!q.is_null(), "the shrink failed");
        let left = zf.0.last_nonzero.load(Ordering::SeqCst);
        let (frees, reallocs) = (
            zf.0.frees.load(Ordering::SeqCst),
            zf.0.reallocs.load(Ordering::SeqCst),
        );
        let kept = all_marked(q, 100);
        // SAFETY: `q` is a live block of `zf` with the shrunk layout.
        unsafe { zf.dealloc(q, Layout::from_size_align(100, 8).unwrap()) };
        assert_eq!(frees, 1, "the old block was not freed");
        assert_eq!(left, 0, "old block keeps {left} of {N} bytes nonzero");
        assert_eq!(reallocs, 0, "the inner allocator resized the block");
        assert!(kept, "contents not copied");
    }

    /// A resize that cannot be met returns null and leaves the old block live
    /// and unchanged (a guard row: GREEN before the fix too).
    #[test]
    fn zf_realloc_fail_keeps_block() {
        let zf = ZeroFree(Rec::default());
        let (p, l) = marked(&zf);
        zf.0.fail.store(true, Ordering::SeqCst);
        // SAFETY: `p` is a live block of `zf` with `l`; the size is non-zero.
        let q = unsafe { zf.realloc(p, l, 2 * N) };
        zf.0.fail.store(false, Ordering::SeqCst);
        let frees = zf.0.frees.load(Ordering::SeqCst);
        assert!(q.is_null(), "a resize that cannot be met succeeded");
        assert_eq!(frees, 0, "the old block was freed when the resize failed");
        let kept = all_marked(p, N);
        // SAFETY: a failed realloc leaves `p` live with `l`.
        unsafe { zf.dealloc(p, l) };
        assert!(kept, "the block changed when the resize failed");
    }
}
