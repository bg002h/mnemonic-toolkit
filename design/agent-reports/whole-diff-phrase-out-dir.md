# Whole-diff adversarial review — `convert --phrase-out-dir` / `--name-with-passphrase-fp`

- Diff: `git diff 5a89a9b eadbbcd` (branch `claude/project-thread-oyjycn`, 5 commits), read in worktree `/home/user/wt-phrase-out-dir`.
- Spec: `design/SPEC_convert_phrase_out_dir.md` (R3 GREEN; its design was not re-argued here).
- Reviewer: independent whole-diff execution review, 2026-11-28.

**Verdict: 0 Critical / 1 Important / 9 Minor.** The code is correct on every seed-touching point I was asked to check. The one Important is lockstep work that must land before merge, not a code defect.

## What I ran (and did not run)

- `cargo build -p mnemonic-toolkit --bin mnemonic`, then `cargo test --test all -- cli_convert_phrase_out_dir`: **28/28 pass**.
- `cargo test --lib -- phrase_file phrase_out_dir error::tests`: **25/25 pass** (11 `phrase_file` units, 2 `derive_slot` units, the error-table rows).
- `cargo test --test all -- convert lint every_test_file`: **321 pass, 0 fail, 1 ignored** (this covers `lint_zeroize_discipline` with floor 41).
- `cargo fmt --check`: clean. `cargo clippy --all-targets -D warnings`: clean (it reused cached results). `bash ci/repro/vendor-freshness.sh`: OK (4/4).
- **Independent fingerprint check.** I wrote a standalone Python script (PBKDF2, HMAC-SHA512, pure secp256k1 and HASH160) that uses no toolkit code. It gives `73c5da0a` for `abandon×11 about` with an empty passphrase, `b4e3f5ed` with `TREZOR`, and `0ed2c5a4` for the Japanese file's bytes exactly as the binary wrote them. All three match the test constants and the file names.
- **F-579 fixture check.** sha256(31×00‖06)[0] = `d1` = the low 8 bits of `surface`'s index (1745). So `abandon×23 surface` is a valid English phrase. `surface` and `abandon` are in both the English and the French vendored lists, and `all-languages` is enabled, so the old `language_of_iter(..).unwrap()` path really was reachable.
- **F-579 equivalence sweep.** I took 16 public BIP-39 vector entropies (16/24/32 bytes, including the `7f…`, `80…`, `ff…`, `9e885d95…`, `68a79eac…`, `f585c11a…` vectors) and ran entropy → phrase → entropy through the new `phrase` arm in all 10 languages. **0 mismatches** out of 160. `entropy_in_language` is a line-for-line copy of vendored `bip39::Mnemonic::to_entropy_array` (`vendor/bip39/src/lib.rs:574-604`). The only change is that the language is passed in instead of re-detected.
- Manual probes: JSON with entropy/phrase on argv (`from_value` is redacted, the phrase entry has no `value`), all 10 language suffixes (they come out as the toolkit's `--language` spellings), an ms1 `entr` card with `--language japanese` (`-japanese`, fingerprint of the Japanese words), and `--to phrase,ms1` with an existing identical file (`already_saved:true`).
- **Not run:**
  - Windows compile (`cargo check --target x86_64-pc-windows-gnu` stops at cc-rs because there is no mingw). I checked the Windows code by reading it against vendored `windows-sys` 0.61.2 instead. `GetLocalTime(*mut Foundation::SYSTEMTIME)` is at `SystemInformation/mod.rs:8` and `SYSTEMTIME` is in `Foundation`, so the code is consistent with them.
  - EACCES probes as a non-root user. The privilege drop was refused in this sandbox, and root bypasses directory modes, so I reasoned about those paths from the code (m-1).

## Checklist from the brief

| item | result |
|---|---|
| A temp file with the words left behind on a non-kill path | **No.** In `phrase_file.rs:432-494`, every return after `T` is created either unlinks it first (461-464 on a write/fsync error; 480 on every `hard_link` outcome) or comes before `T` exists. Nothing between creating `T` and unlinking it can panic. The only remaining way to leave `T` behind is an `unlink` failure, and then the error names `T` (482-493). |
| Replaces, or follows a symlink at, the final name | **No.** Step 0 uses `symlink_metadata` (368). `hard_link` (476) refuses on EEXIST and does not follow a symlink at `N`. The fallback is `create_new` (`O_CREAT\|O_EXCL`, 324-333), which refuses a dangling link. The compare open uses `O_NOFOLLOW` (342) and re-checks with `fstat` (387-392). The tests cover dangling links, live links (including one whose target holds the very same words), directories and FIFOs. |
| FIFO never opened blocking | **Yes.** A FIFO fails `is_file()` (380) before any open, and a FIFO swapped in during the race is opened with `O_NONBLOCK` (342), then refused by the `fstat` re-check. The test would hang if this were wrong. |
| Mode 0600 whatever the umask | **Yes, as far as a umask can allow.** Both creates use `mode(0o600)`, and a umask can only clear bits, so group and other bits can never appear. The `umask 000` row passes. The fallback test checks 0600 too. The step-5 `fstat` warning catches filesystems that ignore the mode. |
| Fallback unlinks its own partial file | **Yes** (525-528). Test `fallback_write_failing_mid_file_leaves_nothing` writes half the bytes, then fails, and finds the directory empty. |
| phrase→phrase only under the flag, and the refusal otherwise byte-identical | **Yes.** `convert.rs:1090-1092` skips `classify_edge` only for `(Phrase, Phrase)` when `out_dir.is_some()`. The refusal text is still built by base `convert.rs:476`, and the test pins the exact stderr and exit 2. |
| Fingerprint equals `--to fingerprint` with no passphrase, comes from the words written, card language wins, passphrase fingerprint only with the flag | **Yes.** `save_phrase_file` (`convert.rs:1326-1333`) hashes the very `words` it writes, in `phrase_language`, which for ms1 is `payload_lang` (`convert.rs:1987`). The passphrase is `Some` only under the flag (1308). The values were checked independently (above). |
| No stdout byte before the write succeeds | **Yes.** The only earlier stdout writer is the auto-repair short-circuit, which returns. 8.b (1299-1312) comes before 9) Emit. `expect_exists_refusal` runs with `--to ms1` and asserts that stdout is empty. |
| JSON v2 only under the flag | **Yes** (1347). There is a byte-identical v1 test without the flag. Exactly one phrase entry is inserted, at the first phrase target's index. That index is correct after removal, because the outputs before it are unchanged. |
| Warnings and exit codes against §2.5 | They match (64 / 1 / 1 / 1 / 1 / 1 / 2). One wrong error variant: m-1. |
| Zeroizing | The phrase strings taken out of `outputs` are wrapped in `Zeroizing` as they are moved (duplicates included). `bytes` is `Zeroizing` with exact capacity, so it never reallocates. The compare buffer is `Zeroizing` and mlock-pinned. The PBKDF2 seed goes through `derive_master_seed` (`Zeroizing`). `Mnemonic` and `Xpriv` are documented third-party-blocked, as at the existing sites. |
| False-PASS tests | None found. See m-3 and m-6 for rows weaker than the spec's wording. |

## Critical

None.

## Important

### I-1. The mnemonic-gui schema-mirror lockstep is owed and is not in this diff (spec §7; toolkit CLAUDE.md "GUI schema-mirror coverage")

The spec §7 lists `mnemonic-gui/src/schema/mnemonic.rs` `CONVERT_FLAGS` (both flags, `--phrase-out-dir` as `Path { stdio_sentinel: false }`), the `src/form/conditional.rs` visibility rules, and the pinned-upstream bump. CLAUDE.md says this "MUST" happen in the same PR or a paired sibling PR. The GUI manual work-around (a non-schema anchor `#convert-cli-phrase-out-dir`, `44-convert.md`) is reasonable for keeping the orphan check green, but it confirms the mirror has not been updated. The toolkit `manual-mr`/GUI lint is a lagging gate and will not catch this.

**Must happen before the PR is offered for merge:** open the paired mnemonic-gui PR.

## Minor

### m-1. A step-0 examine error is reported as "already exists" (`phrase_file.rs:371`)

`symlink_metadata` failing with anything but NotFound (for example EACCES on a `DIR` without search permission) becomes `PhraseFileExists`. The user then reads "`<path>` already exists and is not these words (cannot examine it: Permission denied)". Nothing exists there. Spec §2.5 puts "any other I/O failure" under `Io`. The exit code is 1 either way, so only the message and the `kind` are wrong. Fix: map it to `io_err(dir, "cannot examine the name", e)`.

### m-2. An `unlink(T)` failure after a successful link reports failure, although the seed was saved (`phrase_file.rs:480-494`)

If `hard_link` succeeded but removing `T` fails, the function returns `Io` (exit 1) before it reaches `Ok(Written)`. `N` now holds the words, no "saved phrase to …" line prints, and the message only says to delete `T`. It is safe (nothing is replaced and no stdout output appears), but the user is told it failed. A re-run would say `already saved`. Suggest wording such as "saved to N, but the temporary copy T could not be removed; delete it", or returning success together with a warning.

### m-3. The ms1 repair row cannot reach exit 5 (`tests/cli_convert_phrase_out_dir.rs:755-777`)

Spec §8 says "ms1 auto-repair exit 5 → no file". Since v0.86.0 an ms1 correction is demoted (`repair.rs:~2104`), so `convert --from ms1` never short-circuits to 5. The row asserts `!= 0` and that no file was written, and its comment says so. The structure still holds (the short-circuit returns before 8.b). This is an acceptable deviation, because the spec's premise is out of date. The row stands as a "decode failure → no file" row.

### m-4. A shared-`DIR` swap window between creating `T` and linking it (`phrase_file.rs:466-476`)

`tf` is closed before `hard_link(&t, &n)`, and nothing afterwards checks that `N` is the inode that was written. In a non-sticky directory that another user can write to, that user can rename `T` away and put their own file at `T`'s path before the link. `N` then holds the attacker's bytes (for example a valid phrase they control), and stderr still prints "saved phrase … (fingerprint <the user's fp>)". The reported `mode` also comes from `tf`, not `N`.

The spec accepts writable `DIR`s with only a warning, and the warning does fire. But this outcome is worse than the warning's text ("delete the file or plant names"), because it lets someone substitute a seed. The hardening is cheap:
- keep `tf` open across the link;
- after `Ok(())`, compare `symlink_metadata(&n)` `(dev, ino)` against `tf.metadata()`;
- if they differ, refuse, and do not report success.

### m-5. The writable-`DIR` warning ignores the sticky bit (`phrase_file.rs:201-219`)

For a `1777` directory such as `/tmp`, the warning says others "could delete the saved file". With the sticky bit set they cannot, though they can still plant names. Suggest a second wording when `mode & 0o1000` is set.

### m-6. The F-579 unit row checks equality with `to_entropy()` for one 12-word English phrase only (`derive_slot.rs:588-598`)

The algorithm is the crate's own, and my 160-case sweep agrees with it. A non-English 24-word equality case would pin it, for example the Japanese rendering of the `ff…×32` vector compared against `to_entropy()` on a phrase that is not ambiguous.

### m-7. Help text gaps (`convert.rs:367-379`)

- The name pattern in `--phrase-out-dir`'s help omits `[-<fp-pass>]`. The manual, the CHANGELOG and the spec all include it.
- "owner-only (0600)" is not true on Windows. The Windows warning says so at run time, but the help could add "(Unix)".

### m-8. Windows was not compile-checked in this review

There is no mingw in the container. The code matches vendored `windows-sys` 0.61.2 when read against it (see above), and the `#[cfg_attr(windows, allow(dead_code))]` attributes look right. Make sure CI's Windows job (or Brian's machine) builds it before release.

### m-9. Listed deviations, all acceptable

- No version bump: the CHANGELOG entry is under `[Unreleased]`, and the commit says releases are cut by `cut-release.sh`.
- The GUI manual uses a prose section until the mirror lands (see I-1).
- The UTC-fallback seam is tested on `resolve_date`, not on `today()`.
- The Written-path 0644 warning is covered by the pure-function row plus the already-saved row, with no vfat seam on the write path.
- The FOLLOWUPS `f579-to-entropy-remaining-sites` and `io-lib-write-private-new-hoist` are filed. The io-lib companion entry in mnemonic-engrave is marked "owed when scheduled", which is acceptable.

## Points I checked and found no defect

- `entropy_in_language` runs in the caller's parsed language. The only reachable uses are the `convert` phrase and seedqr arms. `seedqr::encode_compact` (`seedqr.rs:222`) still calls `to_entropy()`, but `convert` does not call it (it is in the remaining-sites FOLLOWUP).
- `--name-with-passphrase-fp` suppresses the "ignored" warning only through `edge_uses_passphrase` (1121). An empty or absent passphrase is refused before any compute (1084-1090).
- No production code path reaches `Faults`: `save_phrase_file` passes `Faults::default()` (1355), and there is no env seam.
- Writing the file before stdout makes failures safe: compute errors and `--xpub-prefix` network errors (8.a) all happen before 8.b.
- `stale_copy_notes` skips today's own name. Its matcher requires a full `YYYY-MM-DD-` date and stops `<fp>` at `-` or `.`, and its unit rows cover a longer hex fingerprint and the temp name.
- The Japanese file is written with rust-bip39's NFKD bytes and ASCII spaces. `bundle --language japanese` reads it back with the same fingerprint, and so does the ms-cli reader emulation.
- `lint_zeroize_discipline` rows were added and not weakened: floor 40 → 41, with evidence strings present in the source.
