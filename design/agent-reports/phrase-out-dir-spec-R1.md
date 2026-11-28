# R1 review — `design/SPEC_convert_phrase_out_dir.md` at `f374386`

Reviewer: R0 architect, round 2. Scope: did the fold fix every R0 Critical
and Important finding, and did the fold add a new defect? Settled rulings and
folded minors are not reopened unless the fold made them wrong. Code baseline
is still `d72a18b`. Nothing was run beyond reading code: the new claims are
about mechanism, and the all-zero-vector facts were measured in R0.

**Verdict: NOT GREEN — 0 Critical, 3 Important (all new, all in the fold), 10 Minor.**

## Fold check, R0 finding by finding

| R0 | fixed? | note |
|---|---|---|
| C-1 `(Phrase, Phrase)` refused | **yes** | §2.1 admits it only under `--phrase-out-dir`. Without the flag the refusal stays byte-identical. RED rows are on both sides (§8). |
| I-1 `write_private` replaces and follows links | **mostly** | `write_private` is dropped. `create_new` temp + `hard_link` + `create_new` fallback gives no replace and follows no symlink at the final name, and EEXIST is examined with `symlink_metadata`. The new mechanism has holes (new I-1, I-2 below). |
| I-2 fingerprint derivation | **yes** | §3: the master fingerprint of the written words with `""`, and needs neither `--template` nor `--network`. The ms1 arm is covered. The warning rule is in §2.4 with RED rows. One wording nit is m-8. |
| I-3 Mnem language, suffix spelling | **yes** | The suffix follows the words' real language; the toolkit spelling is chosen; the ms-cli spelling is documented. |
| I-4 group-size refusal | **yes** | Accepted, with a RED row. |
| I-5 ordering, duplicates, JSON | **yes** | §2.2 orders the effects, duplicates are covered, and the JSON is opt-in. See m-7 on the "additive" label. |
| I-6 F-579 panic | **yes** | Ported first, with a FOLLOWUP. |
| I-7 exit table | **yes** | §2.5. Clap `requires` → 64 matches the measured clap remap. `PhraseFileExists` sorts correctly between `NostrKeyParse` and `Render`. |
| I-8 lockstep | **yes** | Both flags, `Path { stdio_sentinel: false }`, conditional.rs, manuals, FOLLOWUPs. See m-10. |

---

## Important (new, introduced by the fold)

### I-1. The new write steps leave plaintext copies, or a truncated seed file under the real name, on paths the spec does not clean up

Read literally, §4 steps 1-3:

- **The temp outlives the EEXIST paths.** Step 1 writes the words to
  `DIR/.<name>.<pid>.tmp` before the name is known to be free. Step 2 says
  "then unlink the temp" only after a successful `hard_link`. On EEXIST,
  whether "already saved" or `PhraseFileExists`, nothing says to unlink it.
  An "already saved" re-run would then leave one more hidden 0600 copy of the
  words each time. The old `write_private` removed its temp on every error
  (`vendor/mnemonic-io-lib/src/write.rs:416-420`); the new text dropped that
  property. Fix: "on every outcome after step 1, success, already saved,
  refusal or I/O error, unlink the temp". Better still, examine the name with
  `symlink_metadata` *before* step 1, so the common "already saved" re-run
  never writes the words a second time. The `hard_link` EEXIST stays the
  authority.
- **The fallback writes in place with no cleanup.** Step 3 opens
  `DIR/<name>` with `create_new` and writes to it directly. Step 3 targets
  FAT/exFAT sticks, which are small and fill up. There a short write (ENOSPC,
  EIO, a pulled stick) leaves a **truncated seed file at the canonical name**.
  The spec says no fsync for this path and no unlink on failure. The next run
  then hits EEXIST with different bytes and reports `PhraseFileExists`, which
  the user will read as a fingerprint collision or a tampered file, not as
  "your last save failed". Fix: on any error after the `create_new` succeeds,
  unlink `DIR/<name>`. That is safe: `O_EXCL` made it ours, and a racer who
  swaps it needs write access to `DIR`, which the world-writable warning
  covers. Then fsync the file before reporting success, and fsync `DIR`.
  Add a RED row: injected write failure on the fallback leaves no file at the
  name and no temp.

### I-2. "Owner-only" is not true on the filesystems the fallback targets

§4 relies on `mode(0o600)`. On Linux, vfat, exfat, ntfs3 and most SMB and FUSE
mounts take permissions from mount options (`uid=`, `fmask=`/`umask=`,
`file_mode=`), not from `open`'s mode. A typical desktop auto-mount gives
0644 or 0755, and `chmod` is ignored or fails. Those are exactly the
"hard links unsupported" filesystems step 3 is written for, so the tool would
report `saved phrase to …` for a seed file that every local user can read.
The spec gives Windows a warning (C5) but does not cover this Unix case.

Fix: after the write, `fstat` the open handle. If `mode & 0o077 != 0`, or the
owner is not the effective uid, print the same class of warning as Windows
(`warning: file permissions not restricted on this filesystem (mode 0NNN);
protect <path>`). Use one predicate for both, so the C5 pure-function test
covers it. The same check belongs on the "already saved" branch: an
identical file that the user chmod'ed to 0644, or hand-wrote at 0644, is
reported as saved, so warn there too. Add RED rows: a world-readable identical
file gives "already saved" plus a warning, and a mode-check seam is used for
the FAT case.

### I-3. Windows is always UTC, which contradicts the ruling "use local time when available"

§3: "On Windows, or if `localtime_r` fails: the UTC date … and stderr says
`note: local time zone unavailable`". Brian ruled *"No to utc, use local time
when available."* Local time is always available on Windows (`GetLocalTime`).
The spec makes every Windows run UTC and prints a note claiming the zone is
unavailable, which is false. No new crate is needed: `windows-sys` 0.61.2 is
already in `Cargo.lock` and `vendor/` (`vendor/windows-sys/Cargo.toml:15-16`),
and `GetLocalTime` is in its `Win32_System_SystemInformation` module
(`vendor/windows-sys/src/Windows/Win32/System/SystemInformation/mod.rs`).
Adding it as a `[target.'cfg(windows)'.dependencies]` entry with that feature
changes no locked version. It does need the `cargo vendor`/freshness check,
and the "No new dependency" sentence changes to "no new crate". The
alternative is to put the Windows-UTC choice in front of Brian as a deviation
from his ruling. The spec must not state it as settled.

Related: glibc and musl `localtime_r` do not fail when no zone is configured
(no `TZ`, no `/etc/localtime`). They return UTC and report success, so the
"unavailable" note will almost never fire on Unix. That is acceptable (it is
the machine's configured local time), but §3 should say so, so that the build
does not try to detect it from `tm_gmtoff == 0`, which is a legitimate local
offset in the UK half the year.

---

## Minor

- **m-1 Temp-name collision is unspecified.** `.<name>.<pid>.tmp` with
  `create_new` hits EEXIST when a temp from a killed run with a reused pid is
  still there, or when one is planted in a shared `DIR`. Say what happens:
  retry with a counter, as `write_private` does (`.<name>.<pid>.<n>.tmp`,
  bounded), or refuse with `Io`. Otherwise the build picks.
- **m-2 Race between `symlink_metadata` and `open` on the compare branch.**
  `lstat` says regular and ≤ 1 KiB, then the entry is swapped for a FIFO or a
  symlink before the `open`. Then "a FIFO is never opened" fails (the open
  blocks), or a symlink target gets read. Open with `O_NOFOLLOW | O_NONBLOCK`,
  then `fstat` the handle and re-check regular and size; the handle is the
  authority. Only a read is at stake, but the spec makes an absolute claim.
- **m-3 Fallback trigger errnos.** Name them: EPERM (vfat on Linux),
  EOPNOTSUPP/ENOTSUP, ENOSYS (some FUSE), EMLINK. Any other `link` error is
  `Io`. EEXIST is never a fallback trigger.
- **m-4 The leading-dot temp hides the leftover the manual tells users to
  find.** m-3's mitigation was "the name carries the fingerprint, so it can be
  found", but a dotfile is hidden from `ls` and file managers. The
  `seed-*-<fp>*.txt` stale-copy note (§4) does not match `.seed-…tmp`
  either. Extend the note's glob to `.seed-*-<fp>*.tmp` and say "a leftover
  temporary copy exists: <path>". Or drop the dot; the reason for it is not
  given.
- **m-5 Two fsyncs are missing from the mechanism text.** After `hard_link`,
  "fsync DIR" is stated. Unlinking the temp is a second directory change, so
  order it link → fsync DIR → unlink temp (→ fsync again or accept). The
  fallback path names no fsync at all; this is folded into I-1.
- **m-6 "single-threaded".** `restore` and `permutation_search` use
  `std::thread::scope` (`src/cmd/restore.rs:2728`,
  `src/permutation_search.rs:662, 1341`). The soundness argument actually
  rests on this: *no other thread is live when `convert` calls
  `localtime_r`, and no non-test code calls `setenv`* (true at `d72a18b`).
  Reword it so a later change to `convert` that adds threads sees the
  constraint.
- **m-7 The "additive" JSON label contradicts repo precedent.** Additive
  fields have bumped `schema_version` here before: `INSPECT_SCHEMA_VERSION`
  1→2 (CHANGELOG :1547) and `WORD_CARD_SCHEMA_VERSION` 1→2 (FOLLOWUPS :88). The
  new entry also *removes* `value` under the flag, so a consumer that reads
  every `to[].value` sees a missing field. Opt-in keeps old invocations
  unchanged, so staying at "1" is defensible, but state it as a deliberate
  departure from those precedents. `ConvertJsonEntry.value`
  (`convert.rs:464-468`) becomes `Option` with `skip_serializing_if`. Also say
  how many JSON entries `--to phrase,phrase` emits (one or two).
- **m-8 "same derivation function `--to fingerprint` uses".** That function is
  `derive_bip32_from_entropy(…, network, template, account)`, which needs the
  template and network that §3 says are not needed. Say "the same seed
  derivation (`Mnemonic::to_seed` with NFKD) and the same master-key
  fingerprint". The intent is clear, but the literal text is self-contradicting.
- **m-9 The stderr success line with the flag.** §2.3 gives only
  `(fingerprint <fp>, without passphrase)`. State the line when
  `--name-with-passphrase-fp` is set; printing `<fp-pass>` on stderr is
  already in the name, so there is no new leak.
- **m-10 GUI conditional.** Besides showing `--name-with-passphrase-fp` only
  when `--phrase-out-dir` is set, `--phrase-out-dir` should show only when
  `--to` includes `phrase`. Otherwise the form builds argv that is always a
  `BadInput`.
