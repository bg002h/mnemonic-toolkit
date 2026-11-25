# FIX: a failed `seedqr --json-out` replacement destroys the previous file (bh-tk-I3, I); bh-tk-C1 (I, secret) is PR #211

Model: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24. Bug hunt 1119 FIX table row #14 (`FIX-tk-secret-file-writes`),
small-fix track (architect T38). Branch `fix/tk-secret-file-writes`, base `origin/master` `7e87e067`. This document's
commit changes no production code, test or manual; the builder writes them. The RED test file is the sidecar
`design/FIX-tk-secret-file-writes/cli_seedqr_json_out_replace.rs.txt` (263 lines, sha256
`2fb694e34e31b513556615ff1b6c39711f1a6f84c1cac46f026ba5ee3757cd9e`, rustfmt-clean, clippy `-D warnings` clean).

Binding inputs: architect T5 (one safe writer, mnemonic-io-lib 0.1.2), T38 (small-fix), P5 (grade I); architect Q3 of
`architect-secret-ci-decisions-1124` (seedqr writes through `write_private`; the four sibling `--json-out` writers
reach 0600 in file-output unit U6); file-output convention R2/R4/R5, G1 and U6
(`architect-file-output-convention-1123.md:19-22,38,52`); the 1124 batch overrides (registry pin
`mnemonic-io-lib = "0.1.2"`).

## 1. Keys, final grades, and what closes where

| key | consultant | final grade | closes in |
|---|---|---|---|
| bh-tk-C1 (secret) | Critical | **Important** (secret-handling; listed, non-gating in alpha; grade-tk.md:24) | **PR #211** (`build/sec-tk-seedqr-json-out` `b4e42de2`, FIX `design/FIX_tk_seedqr_json_out_secret_file.md`): `seedqr.rs:309,337` become `mnemonic_io_lib::write::write_private`, file 0600. Not re-specified here. |
| bh-tk-I3 | Important | **Important** (data loss of the previous backup; grade-tk.md:27, P5) | **this FIX**: the io-lib pin moves 0.1.0 -> 0.1.2, whose `write_private` replaces a regular file through a temp file and a rename. |

C1 paths #211 does not cover: **none.** `git grep -n "fs::write\|File::create\|OpenOptions"` in `cmd/seedqr.rs` at
`7e87e067` finds only `:309` (decode) and `:337` (encode), both in #211. seedqr reads its secret from argv or stdin only
(`--from seedqr=<VALUE|->`, `--from phrase=<VALUE|->`, `--digits`), never from a file, so no `--json-out` can name an
input (convention Q2 has no case here).

Not in this FIX (decided elsewhere, listed so a reviewer can check the split):
- The sibling `--json-out` writers `final_word.rs:177`, `seed_xor.rs:460,485`, `slip39.rs:916,945`,
  `electrum_decrypt.rs:167` and the `--output` writers `restore.rs:775,1167,1693,3158,4447`,
  `export_wallet.rs:843,1125` keep `std::fs::write` here. Their mode (R4), their atomic replace (R2, G1 "toolkit
  rows 9-10"), the advisory removal (`secret_advisory.rs:47-66`), the `--output` -> `--out` rename and
  `write_private_new` for split are unit U6 (architect Q3: "The four siblings reach 0600 in file-output unit U6").
  The truncate-then-fail mechanism applies to them by construction (grade-tk.md:18); it was not run here.
- The `--json-out` failure wording (`tk-file-output-write-error-text`, M, open) and the R5 "is unchanged" state
  sentence: this FIX keeps the seedqr error text byte-for-byte (section 3). See open point O2.
- The convention Q3 replace notice needs io-lib 0.1.3's `Written` outcome (as mt #12 deferred it).

## 2. Precondition and overlap with PR #211

- This FIX builds on #211's two-line change. The builder starts after #211 has merged to `master` and bases the build
  branch on that `origin/master`. Today `master` (`7e87e067`) still has `std::fs::write` at `seedqr.rs:309,337`; a pin
  bump alone there fixes nothing (measured: section 6 rows run on `7e87e067` and on `b4e42de2` give the same 6 RED).
  If #211 has not merged when the build is dispatched, the builder stops and reports (open point O1).
- Files both touch: `CHANGELOG.md` ([Unreleased]; this FIX edits one clause of #211's entry, section 4.3) and
  `docs/manual/src/40-cli-reference/41-mnemonic.md` (the two seedqr `--json-out` bullets #211 extends; this FIX appends
  one sentence after #211's). `crates/mnemonic-toolkit/src/cmd/seedqr.rs`: #211 only; this FIX does not touch it.
- The builder runs `cargo search mnemonic-io-lib --limit 1` first. On 2026-11-24 it printed
  `mnemonic-io-lib = "0.1.0"`: 0.1.2 is NOT yet on crates.io. Until it is, the builder does every step that needs no
  build and stops (1124 overrides). The GREEN prototype of section 7 used a local `[patch.crates-io]` path to the
  io-lib tree of mnemonic-engrave `e09d538d` (crate version 0.1.2); that patch is not part of the fix.

## 3. Holder-visible behaviour after the fix (`seedqr decode|encode --json-out P`)

Source of the behaviour: mnemonic-io-lib 0.1.2 `write_private` (`crates/mnemonic-io-lib/src/write.rs` at
mnemonic-engrave `e09d538d`, doc :78-92, `classify` :183-214, `write_atomic` :234-280, preflight in `fill_and_rename` :290-299).

| P | before (`7e87e067`; #211 the same unless noted) | after |
|---|---|---|
| absent, write succeeds | exit 0, P holds the envelope (0644 at master, 0600 with #211) | the same, 0600; P is created as `P.<pid>.<n>.tmp` (0600) in P's folder, synced, renamed to P, folder synced |
| existing regular file, write succeeds | exit 0, P rewritten in place | exit 0, P replaced by rename, 0600 (an existing P is first set 0600, preflight :293-297) |
| existing P, write fails (full disk, file-size limit) | exit 1, **P emptied** (measured: 0 bytes) | exit 1, **P byte-identical**, no temp left in the folder |
| absent P, write fails | exit 1, **an empty P is left** | exit 1, no P and no temp |
| P has a second hard-link name Q | P and Q both get the envelope (one inode) | P gets the envelope (new inode); Q keeps the old bytes |
| writable P in a folder the holder cannot write | exit 0, P rewritten in place | exit 1, `Permission denied (os error 13)`, P unchanged (the temp cannot be created) |
| P a symlink to a regular file | target rewritten, link kept | the same (target replaced by rename, link stays a link) |
| P read-only (0444) | exit 1, `Permission denied (os error 13)`, P unchanged | the same (measured on the prototype: rc 1, 444, `old`) |
| P a file the holder can write but does not own | #211: exit 1, `Operation not permitted (os error 1)`, P left **empty** | exit 1, `Operation not permitted (os error 1)`, P **unchanged** (preflight `set_permissions` fails before any write; from the code, not run: needs a second uid) |
| `/dev/null`, `/dev/stdout` (pipe or terminal), a FIFO | #211 table rows | unchanged: these are written directly as in 0.1.0 (`write_direct` :101-121); measured on the prototype: `/dev/null` rc 1 `Operation not permitted (os error 1)`, `/dev/stdout` into a pipe rc 0 with the envelope |
| P in a folder that does not exist | exit 1, `No such file or directory (os error 2)` | the same (measured) |
| process killed after the temp is created and before the rename | (no temp) | `P.<pid>.<n>.tmp` (0600, partial) is left next to P; P is unchanged (T5 "crash leftover"; not run here, io-lib FIX #7 owns its row) |

Unchanged: exit codes (`ToolkitError::BadInput`, exit 1), every printed text (stdout stays empty with `--json-out`;
the error line stays `error: seedqr: <decode|encode>: json-out write to "<P>": <os error>`, `seedqr.rs:310,338`), help,
flags, the envelope bytes. A P whose file name is over 200 bytes gets the temp name
`mnemonic-io-lib.<pid>.<n>.tmp` (`temp_name`, write.rs :221-232).

## 4. Delta

### 4.1 Code and dependency (GREEN commit)
1. `crates/mnemonic-toolkit/Cargo.toml:99`: `mnemonic-io-lib = "0.1.0"` becomes `mnemonic-io-lib = "0.1.2"`. The
   comment above it (:95-98, registry pin, `remedy` path) stays true and stays.
2. `Cargo.lock`: `cargo update -p mnemonic-io-lib --precise 0.1.2` (registry source and checksum lines; nothing else
   moves; in the prototype only that one stanza changed).
3. `vendor/mnemonic-io-lib/`: re-vendored from the registry 0.1.2 crate with `cargo vendor --locked vendor` (the
   repro path, `design/IMPLEMENTATION_PLAN_reproducible_builds_musl.md:82`); `bash ci/repro/vendor-freshness.sh` green
   (check 3 registry provenance needs the published checksum). 0.1.0 -> 0.1.2 changes behaviour only in `write.rs`;
   `channel.rs` and `lib.rs` differ in doc comments only, and toolkit's other io-lib calls
   (`remedy::history_purge_block`, `history_purge_recipes`, `argv_guard.rs:564,810`) are byte-identical.
4. No source change. `seedqr.rs` already calls `write_private` after #211.

### 4.2 Toolkit manual (`docs/manual/src/40-cli-reference/41-mnemonic.md`), PROPOSED (architect confirms)
Append to each of the two seedqr `--json-out` bullets (decode ~:2853, encode ~:2861 after #211), after #211's sentence
"On Unix the file is written owner-only (mode 0600), an existing file's mode included.":

> If the write fails, an existing file is left as it was and no new file is made.

GUI manual (`docs/manual-gui/src/40-mnemonic/4d-seedqr-encode.md:85-93`, `4e-seedqr-decode.md:90-96`): no change (no
flag or help change; those sections describe no write mechanics; #211 made the same choice). GUI schema mirror: no change.

### 4.3 CHANGELOG ([Unreleased]), PROPOSED (architect confirms)
(a) In #211's entry "`seedqr --json-out` writes its file owner-only", replace the clause
"and a refused file you do not own is left empty." with "and the file is left as it was." (0.1.2 refuses before any
write, section 3; the #211 clause would be false after this FIX.)

(b) New entry, placed directly after #211's:

> ### `seedqr --json-out` keeps the old file when a write fails
>
> - **`seedqr decode`, `seedqr encode`**: `--json-out PATH` now writes a temporary file next to PATH
>   (`PATH.<pid>.<n>.tmp`, mode 0600) and renames it over PATH (mnemonic-io-lib 0.1.2). A write that fails (a full
>   disk, a file-size limit) leaves an existing PATH byte-for-byte as it was and makes no file where there was none
>   (before: PATH was emptied, and a failed first write left an empty file). PATH in a folder you cannot write is
>   now refused (exit 1, `Permission denied`; before: written in place). A PATH with a second hard-link name is
>   replaced at PATH only; the other name keeps the old contents. A run killed mid-write can leave the temporary
>   file behind.

## 5. Tests that change
- New: `crates/mnemonic-toolkit/tests/cli_seedqr_json_out_replace.rs` = the sidecar, byte-identical (RED commit).
- Existing tests: **none change.** Measured: the full workspace suite with the prototype (section 7) is
  `4459 tests run: 4459 passed, 37 skipped` (= #211's 4451 + the 8 new rows); #211's
  `cli_seedqr_json_out_mode` 4 rows pass unchanged.

## 6. RED rows (run; signet-irrelevant: the public BIP-39 test phrase `abandon x11 about`, SeedQR `0000...0003`)
Command: `cargo nextest run --locked -p mnemonic-toolkit --test cli_seedqr_json_out_replace --no-fail-fast --build-jobs 6 -j 6`
with the sidecar copied to `crates/mnemonic-toolkit/tests/cli_seedqr_json_out_replace.rs`. Failing rows run under
`sh -c 'umask 022; ulimit -f 0; trap '' XFSZ; exec ...'`.

| row (test name) | expected after | seen on `7e87e067` (master) | seen on `b4e42de2` (#211) |
|---|---|---|---|
| `i3_seedqr_encode_failed_replace_keeps_previous_file` | rc 1; last stderr line `error: seedqr: encode: json-out write to "<P>": File too large (os error 27)`; stdout empty; P == `PREVIOUS-BACKUP-BYTES\n`; folder holds only `seed.json` | FAIL `previous file destroyed by a failed replacement`, `left: []` | FAIL, same |
| `i3_seedqr_decode_failed_replace_keeps_previous_file` | the same with `decode` | FAIL, same | FAIL, same |
| `i3_seedqr_encode_failed_create_leaves_no_file` | rc 1; same last line; folder empty | FAIL `a failed create left a file`, `left: ["seed.json"]` | FAIL, same |
| `i3_seedqr_decode_failed_create_leaves_no_file` | the same with `decode` | FAIL, same | FAIL, same |
| `i3_seedqr_encode_hard_link_other_name_keeps_old_bytes` | rc 0; P holds the envelope; Q == previous bytes; inodes differ | FAIL `the other hard-link name was rewritten` (Q holds the envelope) | FAIL, same |
| `i3_seedqr_encode_readonly_folder_refused_file_unchanged` | rc 1; last line `... json-out write to "<P>": Permission denied (os error 13)`; P unchanged; folder holds only `seed.json` (skips with a note where the folder mode is not enforced, e.g. root) | FAIL `exit`, `left: 0 right: 1` | FAIL, same |
| `i3_seedqr_encode_symlink_target_replaced_link_kept` (guard) | rc 0; link still a symlink; target holds the envelope | PASS | PASS |
| `i3_seedqr_encode_retry_after_failure_writes_envelope` (guard) | failed run rc 1, then rc 0, stdout empty, envelope, folder holds only `seed.json` | PASS | PASS |

Totals: master `8 tests run: 2 passed, 6 failed`; #211 `8 tests run: 2 passed, 6 failed`. Every RED fails on the
defect assertion; the text-pin assertions before it passed, so the error lines are as pinned.

## 7. GREEN (prototype, run then reverted)
`b4e42de2` + `crates/mnemonic-toolkit/Cargo.toml` pin `"0.1.2"` + a root `[patch.crates-io] mnemonic-io-lib = { path =
<e09d538d io-lib tree> }` + `cargo update -p mnemonic-io-lib --offline` (lock: that stanza only) + the sidecar:
`cargo nextest run --locked --workspace --no-fail-fast --build-jobs 6 -j 6` -> `4459 tests run: 4459 passed, 37
skipped`, all 8 new rows PASS (the read-only-folder row ran; uid 1000). The builder's GREEN uses the registry crate,
not the patch.

## 8. Gate (builder, on the exact head; `CARGO_BUILD_JOBS=6`, host-load wait before each cargo call)
`cargo fmt --all -- --check`; `cargo clippy --locked --workspace --all-targets -- -D warnings`;
`cargo nextest run --locked --workspace --no-fail-fast --build-jobs 6 -j 6` (expect 4459 passed, 37 skipped on #211's
base, plus whatever merged since); `cargo test --locked --workspace --doc`; `bash ci/repro/vendor-freshness.sh`;
`bash ci/repro/vendor-freshness.test.sh`; `make -C docs/manual audit` (with the sibling binaries the manual job pins);
`bash ci/doc-flag-lint.test.sh`; `bash ci/doc-gate-guard.test.sh`. Not local: the release repro double-build,
macOS and the cross-check legs (CI).

## 9. Coverage map (rule -> row)
- Failed replace keeps the old file, encode/decode -> rows 1, 2.
- Failed create leaves nothing -> rows 3, 4.
- No temp left on failure -> rows 1, 2 (folder listing); on success -> row 8.
- Hard link replaced at the given name only (T5) -> row 5.
- Folder without write permission fails closed (T5) -> row 6.
- Symlink: target replaced, link kept (R2) -> row 7.
- Error text and exit unchanged -> rows 1-4, 6 (last-line pins).
- Mode 0600 -> #211's `cli_seedqr_json_out_mode` (unchanged, passes).
- Not-owned file left unchanged; kill leftover -> from the code (section 3), no row (open point O3).

## 10. FOLLOWUPS
Closes nothing in `design/FOLLOWUPS.md` (bh-tk-I3 is not filed there). `tk-file-output-write-error-text` stays open.

## 11. Open points (for the architect; the rows above do not depend on them)
- O1: build order. This FIX assumes #211 merges first (section 2); the alternative is to stack the build on #211's
  branch.
- O2: wording. This FIX keeps `error: seedqr: <verb>: json-out write to "<P>": <e>`. md #9 replaced its `--out`
  failure text with T6's `md: cannot write the output: <error>`; mt #12 proposes T6 plus an R5 state sentence
  ("<path> is unchanged." / "Nothing was written."), under architect review (architect-mt-output-writes-1124). If R5
  is to reach tk seedqr in this FIX rather than U6, the text and the 5 pinned last lines change.
- O3: rows for the not-owned file and the killed-run leftover are not in the sidecar (a second uid; a kill between
  temp creation and rename). io-lib's own tests own the latter (T5).
