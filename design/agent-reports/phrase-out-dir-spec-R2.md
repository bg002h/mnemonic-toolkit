# R2 review — `design/SPEC_convert_phrase_out_dir.md` at `87a8d07`

Reviewer: R0 architect, round 3. Scope: did the R1 fold fix R1's three
Importants, did it add a new defect, and are the folded minors right? Settled
rulings are not reopened. I read the diff `f374386..87a8d07` against the code at
`d72a18b` and the vendored crates; nothing was run.

**Verdict: NOT GREEN — 0 Critical, 1 Important (new in the fold), 4 Minor.**
The one Important is a one-sentence fix, and its re-review can be mechanical.

## R1 Importants

| R1 | fixed? | note |
|---|---|---|
| I-1 temp cleanup, fallback truncation | **partly** | Step 0 (check the name before any temp) and step 3 (unlink `T` on every outcome of step 2) fix the EEXIST leaks. Step 4 unlinks `N` on any error after its `create_new`, so the fallback can no longer leave a truncated file under the real name. Both fsyncs are stated. **A failure inside step 1 itself is still uncovered** (new I-1 below). |
| I-2 owner-only on mount-moded filesystems | **yes** | Step 5 `fstat`s the handle and warns on group/other bits, and the warning covers the "already saved" branch. On the hard-link path the handle is `T`'s, which is the same inode as `N`, so the `fstat` is valid after the `unlink(T)`. |
| I-3 Windows UTC vs the ruling | **yes** | `GetLocalTime` via the vendored windows-sys 0.61.2. glibc/musl "UTC with success" is accepted with no note, and the `tm_gmtoff == 0` trap is named. See m-3 on wording. |

Folded minors checked and right:
- the temp name `seed-tmp-<pid>-<k>.txt`: `create_new`, counter, cap 16 then `Io`; a planted symlink only uses up a slot;
- the open with `O_NOFOLLOW|O_NONBLOCK` + `fstat` re-check;
- the visible temp and its inclusion in the stale-copy note;
- `schema_version` `"2"` only under the flag, byte-identical `"1"` without it;
- the stderr line with both fingerprints (the second is already in the name, so nothing new leaks);
- the GUI conditional for both flags;
- the template-free fingerprint wording.

---

## Important

### I-1. A write or fsync failure in step 1 leaves `T` behind, but the spec says no path does

Step 1: "Create `T` … Write, `fsync(T)`." Step 3 unlinks `T` "on **every**
outcome of step 2", and then: "No path leaves `T` behind except a kill." An
error from the write or the `fsync` in step 1 never reaches step 2. So `T`
stays, mode 0600, holding the complete words or a prefix of them. A full stick
(ENOSPC), EIO or EDQUOT is exactly the failure R1 I-1 named, and on a small
FAT/exFAT stick step 1's write is where ENOSPC lands first. The claim in step 3
is therefore false as written. An implementer who follows the steps literally
propagates the error with `?` and leaves the file.

Fix, one sentence in step 1: "on any error after `T` was created, `unlink(T)`
before exiting". Better, state the invariant once: "once created, `T` is
unlinked before `convert` returns, on every path". Extend the §8 row
"fallback write failing mid-file" with "step-1 write failing (seam) → no
`seed-tmp-*` left".

---

## Minor

- **m-1 The fallback errno list is Unix-only, and Windows FAT sticks miss it.**
  `std::fs::hard_link` on Windows (`CreateHardLinkW`) fails on FAT/exFAT with
  Windows error codes (`ERROR_INVALID_FUNCTION` 1, `ERROR_NOT_SUPPORTED` 50),
  not errnos. Under step 4's "any other `hard_link` errno is `Io`", saving to a
  USB stick on Windows fails outright, and C5 rules that Windows writes work.
  Linux FUSE can also return `ENOSYS` for an unimplemented `link`. The simpler
  rule is correct on every platform: fall back on **any** `hard_link` error
  except EEXIST. The fallback is `O_CREAT|O_EXCL`, so it is safe whatever the
  cause. (`EXDEV` in the list is harmless: `T` and `N` share a directory.)
- **m-2 `O_NOFOLLOW` has no Windows meaning.** The compare-branch open is
  Unix-specific. On Windows `symlink_metadata` still catches a symlink at the
  name, but the open itself follows reparse points. Say the Windows build
  relies on the `symlink_metadata` check and the post-open regular-file
  re-check alone, or uses `FILE_FLAG_OPEN_REPARSE_POINT`. Creating symlinks on
  Windows needs Developer Mode or admin, so the exposure is small.
- **m-3 "Only if the call fails" reads as covering `GetLocalTime`, which
  returns `void` and cannot fail.** The note can fire only on Unix, when
  `localtime_r` returns NULL. Reword so the build does not invent a Windows
  failure check.
- **m-4 windows-sys details.** `GetLocalTime` needs the
  `Win32_System_SystemInformation` feature, and its `SYSTEMTIME` argument
  needs `Win32_Foundation`. The new direct-dependency edge **does** change
  `Cargo.lock` (mnemonic-toolkit's dependency list), even though the version
  is already locked. "Already locked" should not be read as "no lock change"
  by the `--locked` CI guard at `.github/workflows/rust.yml` ("Verify
  Cargo.lock is up to date").
