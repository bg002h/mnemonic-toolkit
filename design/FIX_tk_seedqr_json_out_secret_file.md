# FIX: `mnemonic seedqr decode|encode --json-out` writes the phrase to a umask-mode (0644) file (I)

Model: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24. Small-fix track (RULES.md "Authors"). Branch
`fix/sec-tk-seedqr-json-out`, base `origin/master` `7ff7de83` (no change to any file this document cites since
`cadbfc93`, the sha the ruling and the recon cite: `git diff --stat cadbfc93 7ff7de83 -- crates/mnemonic-toolkit/src/cmd/seedqr.rs
docs/manual/src/40-cli-reference/41-mnemonic.md crates/mnemonic-toolkit/Cargo.toml vendor/mnemonic-io-lib
crates/mnemonic-toolkit/tests/cli_seedqr.rs` is empty). This document's commit changes no production code and adds
no test file; the builder writes them.

Ruling (binding): architect Q3, `coordinator/agent-reports/architect-secret-ci-decisions-1124.md:31-39`, option
(b): seedqr writes the file 0600 now through the existing `mnemonic_io_lib::write::write_private`, with no
advisory line; the four sibling `--json-out` writers reach 0600 later in file-output unit U6
(`architect-file-output-convention-1123.md:37`). Recon: `coordinator/agent-reports/secret-ci-recon-1124/mnemonic-toolkit.md`
"FIX QUEUE" (:33-42).

**One STOP is open (section 11): the boundary test `p3_declined_crate_items` fails under the ruled change, and no
ruling says how it changes.** Everything else below follows the ruling.

**How this document was checked.** The code change and the manual change were applied in this worktree as a
scratch change and reverted. They are saved as reference patches:
`/scratch/cache/tk-seedqr-json-out-reference.patch` (sha256
`6edaab2524e5cf20048ac06b153a9219645f17d0d666c03a868eeefdb329abfc`, `seedqr.rs` +2 -2) and
`/scratch/cache/tk-seedqr-json-out-manual.patch` (sha256
`b1f8417f7c360a8e41a6fa0b239d0ce2414a3f83bd704863f73cb86d4a350925`, `41-mnemonic.md` +2 -2); both pass `git apply
--check` at `7ff7de83`. The RED test file of section 6 was written and run; it is saved at
`/scratch/cache/tk-seedqr-json-out-red-test.rs` (sha256
`087d7f5d65cc56c93db876c4b52a8e03f91047726798cb272073d93aff5bc9a5`, 123 lines; `cargo fmt` wants two argument
arrays wrapped, the builder runs `cargo fmt`). Every "today" cell was run against a debug `mnemonic` built from
`7ff7de83`; every "after" cell against the same tree with the code patch. The full workspace nextest suite was run
once with the code patch and the test file (section 8).

## 1. Summary

`seedqr decode --json-out P` and `seedqr encode --json-out P` write a JSON envelope holding the full BIP-39 phrase
and the SeedQR digits with `std::fs::write` (`crates/mnemonic-toolkit/src/cmd/seedqr.rs:309` and `:337` at
`7ff7de83`; the ruling cites them as :308 and :336, one line early). Under umask 022 a fresh file is 0644, an
existing file keeps its mode, and nothing is printed about it (the stdout class advisory at `seedqr.rs:215-220` and
`:282-287` fires only when `--json-out` is absent). Measured today: exit 0, `stat` 644 for decode and encode.

The fix replaces each `std::fs::write(path, json)` with `mnemonic_io_lib::write::write_private(path,
json.as_bytes())`. `write_private` (`vendor/mnemonic-io-lib/src/write.rs:45-65` at `7ff7de83`, crate 0.1.0, already a
dependency at `crates/mnemonic-toolkit/Cargo.toml:99`, `Cargo.lock:735-738`) opens with write+create+truncate and
mode 0600 (:48-55), then calls `set_permissions(0600)` on the open file (:59-63), then writes the bytes (:64). No new
flag, type, command, error text or public API; no spec owns the file mode. Small-fix track.

## 2. Holder-visible behaviour (after the fix)

- `seedqr decode --json-out P` and `seedqr encode --json-out P`, P absent: P is created mode 0600 whatever the
  umask. Exit 0, stdout empty, envelope bytes unchanged.
- P an existing regular file the holder owns, at any mode that allows the holder to write (e.g. 0644): its mode
  becomes 0600, the contents are replaced (truncated first, so a shorter envelope leaves no tail). Exit 0.
- P a symlink to such a file: the link is followed (no `O_NOFOLLOW` in `write.rs:48-55`); the target becomes 0600
  and holds the envelope. Measured: target 644 → 600, rc 0.
- No new stderr line. The class advisory still fires only on the stdout path. Help text: no change.
- Error texts unchanged: `seedqr: decode: json-out write to {path:?}: {e}` and the `encode` twin (`seedqr.rs:310`,
  `:338`), `ToolkitError::BadInput`, exit 1 (`error.rs:643`).
- Changed outcomes on paths that are not a holder-owned regular file (measured on the patched binary unless marked
  "from the code"):

| `--json-out` | today (`7ff7de83`) | after |
|---|---|---|
| `/dev/null` (root-owned, 0666) | rc 0 | rc 1; stderr last line `error: seedqr: decode: json-out write to "/dev/null": Operation not permitted (os error 1)` (the `fchmod` at `write.rs:62` is refused) |
| `/dev/stdout`, stdout a pipe | rc 0, envelope on the pipe | rc 0, envelope on the pipe (unchanged) |
| `/dev/stdout`, stdout a terminal the holder owns | rc 0 | rc 0; the terminal device's mode goes 620 -> 600 (measured on a pty, architect-tk-seedqr-stops-1124 Q2) |
| an existing file the holder can write but does not own | rc 0, written | rc 1, `Operation not permitted (os error 1)`; the file is left empty, owner and mode unchanged (measured in a user namespace, architect-tk-seedqr-stops-1124 Q2). Today the envelope lands in the other user's file. |
| an existing read-only (0444) file | rc 1, `Permission denied (os error 13)` | the same (measured: rc 1, file untouched, 444) |

Accepted for this FIX (architect-tk-seedqr-stops-1124 Q2); the /dev/null and terminal rows close with iolib-direct-chmod-non-regular.

## 3. Code delta, `crates/mnemonic-toolkit/src/cmd/seedqr.rs` (the reference patch)

1. `emit_decode_output`, :309: `std::fs::write(path, json).map_err(|e| {` becomes
   `mnemonic_io_lib::write::write_private(path, json.as_bytes()).map_err(|e| {`.
2. `emit_encode_output`, :337: the same replacement.

Nothing else: the `map_err` closures and their texts, the advisory gates at :215 and :282, and the function
signatures stay. No `stderr` threading (the advisory route of the recon's minimum fix is not taken; the ruling
says no warning line).

## 4. Spec delta

No spec states the seedqr `--json-out` file mode. `design/BRAINSTORM_v0_30_0_seedqr.md:63` says the flag "mirrors
`seed-xor split` … + `slip39 split`"; it is a closed brainstorm record and is not edited (the mirror was of the flag
shape and the empty stdout, which do not change). The holder-facing statement is the manual, section 5.

## 5. Manual, help and GUI delta (lockstep, RULES "Builds and tests")

`docs/manual/src/40-cli-reference/41-mnemonic.md` at `7ff7de83` (the manual patch):

5.1. :2853, old (whole line):

> - `--json-out <PATH>`: emit a JSON envelope at PATH instead of plain text on stdout.

new:

> - `--json-out <PATH>`: emit a JSON envelope at PATH instead of plain text on stdout. On Unix the file is written owner-only (mode 0600), an existing file's mode included.

5.2. :2861, old (whole line):

> - `--json-out <PATH>`: emit a JSON envelope at PATH instead of plain text on stdout. The envelope's `variant` field reflects the selected variant; the `digits` field holds the payload (decimal for standard, hex for compact).

new:

> - `--json-out <PATH>`: emit a JSON envelope at PATH instead of plain text on stdout. The envelope's `variant` field reflects the selected variant; the `digits` field holds the payload (decimal for standard, hex for compact). On Unix the file is written owner-only (mode 0600), an existing file's mode included.

The appended sentence is the ruling's text verbatim (`architect-secret-ci-decisions-1124.md:34`).

- Help text (`seedqr.rs:76`, `:97` doc comments): no change (ruling). No flag added or removed, so
  `docs/manual/tests/lint.sh` and the GUI schema mirror are unaffected.
- mnemonic-gui: `src/schema/mnemonic.rs:2334-2342@7e1b498` (seedqr `--json-out`, help "Side-effect: write versioned
  JSON envelope to PATH.") states no mode: no change.
- The sibling world-readable advisory rows (41-mnemonic.md:2004, :2057, :2108, :2235, :2555, :2609) are
  `electrum-decrypt`, `final-word`, `seed-xor` and `slip39`: unchanged here (U6).

## 6. RED rows

New file `crates/mnemonic-toolkit/tests/cli_seedqr_json_out_mode.rs`, `#![cfg(unix)]` (each `tests/*.rs` is its own
binary; nothing to register). Each row runs the binary through `sh -c 'umask 022; exec "$0" "$@"' <mnemonic>
--allow-argv-secret <args>` (`assert_cmd::cargo::cargo_bin("mnemonic")`, the pattern of
`tests/cli_liana_template_refusal.rs:71`), in a `tempfile::tempdir()`. Note: the existing `cli_seedqr.rs` JSON
rows use `NamedTempFile`, which pre-creates the file at 0600, so they cannot see the defect.

Constants: `PHRASE_12` = BIP-39 `abandon` x11 `about`, `DIGITS_12` =
`000000000000000000000000000000000000000000000003` (as `tests/cli_seedqr.rs:6-8`; a network-free BIP-39 test
vector, used here only as signet test data).

Every row asserts, in order: exit 0; stdout empty; the file parses as the envelope with `operation` =
decode/encode, `phrase` = PHRASE_12, `digits` = DIGITS_12; stderr contains neither `world-readable` nor `umask`;
`mode & 0o077 == 0`; `mode & 0o777 == 0o600`.

| row | args | setup | today (`7ff7de83`) | wrong implementation it catches |
|---|---|---|---|---|
| `seedqr_decode_json_out_fresh_file_0600` | `seedqr decode --from seedqr=DIGITS_12 --json-out <dir>/decode.json` | none | fails: `assertion left == right failed: mode is 644` | decode left on `std::fs::write` |
| `seedqr_encode_json_out_fresh_file_0600` | `seedqr encode --from phrase=- --json-out <dir>/encode.json`, stdin PHRASE_12 | none | fails: `mode is 644` | encode left on `std::fs::write` |
| `seedqr_decode_json_out_existing_0644_tightened` | as the decode row | file pre-written `old`, `chmod 0644`, control assert 0644 | fails: `mode is 644` | `OpenOptions::mode(0o600)` alone (binds on create only) |
| `seedqr_encode_json_out_existing_0644_tightened` | as the encode row | as above | fails: `mode is 644` | the same, encode side |

Run on `7ff7de83` with production code untouched (`cargo nextest run --locked -p mnemonic-toolkit --test
cli_seedqr_json_out_mode --build-jobs 6 -j 6`, log `/scratch/cache/seedqr-red.log`):

```
FAIL seedqr_decode_json_out_existing_0644_tightened  panicked at crates/mnemonic-toolkit/tests/cli_seedqr_json_out_mode.rs:83:5: assertion `left == right` failed: mode is 644
FAIL seedqr_encode_json_out_existing_0644_tightened  ... :83:5 ... mode is 644
FAIL seedqr_encode_json_out_fresh_file_0600          ... :83:5 ... mode is 644
FAIL seedqr_decode_json_out_fresh_file_0600          ... :83:5 ... mode is 644
Summary 4 tests run: 0 passed, 4 failed
```

Line 83 is the `mode & 0o077 == 0` assertion, so the exit, stdout, envelope and no-advisory assertions before it
passed: each row fails for the stated reason only. With the code patch all four pass (section 8 run: `PASS` x4).

Manual check of today's behaviour (umask 022, unpatched binary): fresh decode → `644`, fresh encode → `644`, a
symlink to a 0644 file → target stays `644`, all rc 0, stderr only the argv warning for the inline decode.

## 7. Sentence coverage

| sentence | rows |
|---|---|
| manual 5.1 "written owner-only (mode 0600)" | `seedqr_decode_json_out_fresh_file_0600` |
| manual 5.1 "an existing file's mode included" | `seedqr_decode_json_out_existing_0644_tightened` |
| manual 5.2 (both clauses) | `seedqr_encode_json_out_fresh_file_0600`, `seedqr_encode_json_out_existing_0644_tightened` |
| ruling "no advisory" | the stderr assertion in all four rows |
| envelope unchanged | the envelope assertion in all four rows; existing `cli_seedqr.rs` JSON rows |

## 8. Tests that change

Found by one `cargo nextest run --locked --workspace --build-jobs 6 -j 6 --no-fail-fast` with the code patch and the
RED file applied (log `/scratch/cache/seedqr-green.log`): `4451 tests run: 4450 passed, 1 failed, 37 skipped`. The
one failure:

1. `crates/mnemonic-toolkit/tests/p3_declined_crate_items.rs::the_binary_names_only_the_adopted_crate_items`
   (:25-69). It scans `src/` and allows `mnemonic_io_lib::` only before `remedy::`; its message:
   `P3's boundary table adopts `remedy` and NOTHING else from mnemonic-io-lib. These lines reach a declined item; if
   that is intended it is a boundary change and belongs in a plan, not in a tidy-up:` followed by
   `cmd/seedqr.rs:309: …write_private…` and `cmd/seedqr.rs:337: …`. The boundary table it enforces is
   `mnemonic-engrave/design/IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md:704@bc90d311` (`write_private` … `mnemonic`:
   "DECLINE — no `--out` in P3's row"). It changes by architect-tk-seedqr-stops-1124 Q1: it allows the one item `write::write_private` by exact name and is renamed `the_binary_names_only_the_adopted_crate_items`; the patch is design/FIX_tk_seedqr_json_out_secret_file.p3-test.patch. The adoption is the one SPEC_io_lib_safe_write.md section 9 (tk row) already plans; P3's decline was "no `--out` in P3's row" (IMPLEMENTATION_PLAN_P3_md_mk_mnemonic.md:704@bc90d311).

No other test changes. No golden or snapshot file changes.

## 9. Gate (the builder runs all of it; capture each run once to a file and grep it)

`PATH=$HOME/.cargo/bin:$PATH`; `cargo --version` must be 1.85.0 (`rust-toolchain.toml`); never `--release`.

1. RED commit: the section 6 test file (after `cargo fmt`), production code untouched. `cargo nextest run --locked
   -p mnemonic-toolkit --test cli_seedqr_json_out_mode --no-fail-fast`: **4 run, 4 failed**, each `mode is 644`.
2. GREEN commit: sections 3 and 5, and the STOP 1 answer. `cargo fmt --all -- --check`; `cargo clippy --locked
   --all-targets --workspace -- -D warnings`; `cargo nextest run --locked --workspace --no-fail-fast` 0 failed;
   `cargo test --locked --doc --workspace`; the `cli_seedqr_json_out_mode` filter **4 run, 4 passed**.
3. Docs CI jobs on the head (locally where possible, else green CI): `manual.yml` (incl. `docs/manual/tests/lint.sh`),
   `manual-gui.yml`, `technical-manual.yml`, `quickstart.yml`, `examples.yml`; the rest of `rust.yml` and
   `fuzz-smoke.yml` as green CI.
4. `coordinator/scripts/mutation-gate.sh <worktree> 7ff7de83` after step 2; its missed list goes in the review brief.

Not covered: macOS (`#![cfg(unix)]` runs there, but no macOS CI job is claimed here); the terminal and not-owned
rows of section 2 (measured once by the architect, not tested).

## 10. Review class

`coordinator/scripts/review-class.sh <worktree> origin/master` on a scratch commit holding the code patch, the
manual patch and the RED file (output `/scratch/cache/seedqr-review-class.txt`): **A** (holder-facing text:
`docs/manual/src/40-cli-reference/41-mnemonic.md`), C (`cmd/seedqr.rs`), D (the test file); "REVIEW: s1 Opus
executed advice review …; s2 Sonnet tests-can-fail against mutation-gate survivors." The ruling said class B
(`architect-secret-ci-decisions-1124.md:34`); the script's A is higher and the script's rule is "Override upward by
judgement, never downward", so the class is **A**.

## 11. STOPs (points no ruling decides)

Resolved: architect-tk-seedqr-stops-1124 Q1 (allow exactly write::write_private).
Resolved: architect-tk-seedqr-stops-1124 Q2 (accept; library line filed).

## 12. Secret rows

Close on the GREEN commit (the ruling, `architect-secret-ci-decisions-1124.md:36`): `bh1119-tk-c1-seedqr-json-out-mode`
(row 270), `bh-tk-c1` (274), `bh-tk-c1b` (275), `tk-bh-c1` (278).

Row 399 `tk-json-out-umask-secret` (M) stays open (closes with U6) and narrows. Its summary cell, old:

> --json-out writes seed candidates, shares and decrypted phrases at the umask (644 measured, M3 T1) with an advisory only; the same std::fs::write is at slip39.rs:916,945, seed_xor.rs:460,485, seedqr.rs:309,337, electrum_decrypt.rs:167

new (only `, seedqr.rs:309,337` removed):

> --json-out writes seed candidates, shares and decrypted phrases at the umask (644 measured, M3 T1) with an advisory only; the same std::fs::write is at slip39.rs:916,945, seed_xor.rs:460,485, electrum_decrypt.rs:167
