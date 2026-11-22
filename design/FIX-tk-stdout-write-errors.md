# FIX-tk-stdout-write-errors: a failed stdout write must not exit 0 (bughunt-1119 bh-tk-I2)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: **small** (one Opus review round).
Base: origin/master `dbac472f`. Keys: bh-tk-I2 (grade I, CONFIRMED). Ruling: architect T6 (decided; quoted in
section 1). Sources: `design/agent-reports/bug-hunt-2026-11-19.md` section I2; coordinator
`agent-reports/bughunt-1119/grade-tk.md` L26, L50; `triage-tk-t1.md` L13.

**Status: one open question (Q1, section 7).** It decides only what exit 4 becomes when stdout fails. Every other
row is decided; rows R16 and R21 carry both outcomes.

## 1. The contract (T6, decided)

T6: every tool takes md's published contract (toolkit manual `42-md.md:799`): a closed pipe stops output and keeps
the verb's own code; any other stdout write error prints `<tool>: cannot write the output: <error>` and exits 1; a
verb already failing keeps its code.

For `mnemonic` that is:

- `<tool>` is `mnemonic` (the binary name). The line is exactly
  `mnemonic: cannot write the output: <error>`, where `<error>` is `std::io::Error`'s `Display` (for example
  `No space left on device (os error 28)`). It names no output content, so no secret reaches stderr.
- Scope: stdout only, for every verb and for clap's `--help`/`--version`. File outputs (`--out`, `--json-out`, ...)
  are out of scope for this row; see FOLLOWUPS line F1 in section 9.

## 2. Defect at `dbac472f` (measured; section 5 lists the rows)

1. 34 stdout writes discard their result with `.ok()`:
   `grep -rnE '(writeln!|write!)\(stdout|to_writer(_pretty)?\(&mut \*stdout' crates/mnemonic-toolkit/src | grep '\.ok()'`
   prints 34 lines: `cmd/bundle.rs` 17 (L1020-L1084), `cmd/verify_bundle.rs` 10 (L684-L695, L2061-L2072),
   `cmd/convert.rs` 3 (L1237, L1238, L1251), `cmd/gui_schema.rs` 3 (L1325, L1333, L1334), `cmd/derive_child.rs` 1
   (L306). Every multi-line `.ok()` in `src/` is a stderr write (checked: cmd/repair.rs L141/L152/L250,
   repair.rs L2091/L2099, cmd/verify_bundle.rs L1352-L1362/L1454-L1464/L3706/L3723/L3739/L3895/L3989).
   Result: `derive-child`, `convert` (text and `--json`), `bundle`, `gui-schema` and `verify-bundle` exit 0 (or their
   verdict code) with nothing written (R1-R5, R13, R14, R17).
2. Verbs that propagate with `?` turn the error into `ToolkitError::Io` and print `error: I/O error: <error>` with
   exit 1 (`main.rs:289-292`, `error.rs:677`). That is right for a full disk but wrong for a closed pipe: T6 keeps the
   verb's own code there (R7).
3. `main.rs:221` passes `io::stdout()` straight to the verbs and never flushes it; std drops a failed buffered flush
   at exit silently.
4. `main.rs:214-216`: `e.print().ok()` discards a failed `--help`/`--version` write (R9, R10).
5. std's `Stdout` reports `EBADF` as a successful write, so a read-only stdout (`1</dev/null`) loses the output with
   exit 0 (R11, R12).

## 3. Spec delta (what the builder changes)

### 3.1 One stdout sink (new module, `crates/mnemonic-toolkit/src/out_sink.rs`, declared in `main.rs`)

A writer that wraps `io::stdout()` and is the only thing `main` hands the verbs as `stdout` (`main.rs:221`; every
verb's `run` already takes a generic `W: Write`, `gen_man::run` takes `&mut impl Write`; no verb signature changes).
Its behaviour, which is the spec (code shape is the builder's):

- `write`/`write_all`: after the first failure, write nothing and return `Ok`. On a failure, record it: an
  `ErrorKind::BrokenPipe` error only marks the sink closed; any other error is kept (the first one only).
- It returns `Ok` even on the failing call, so a verb never sees a stdout error, runs to its own end, and keeps its
  own code and its own stderr lines (the md precedent, `descriptor-mnemonic` `crates/md-cli/src/out.rs`).
- `flush`: same rule. `main` flushes the sink once after the verb returns (both `Ok` and `Err`), before the exit
  code is mapped.
- Read-only stdout (unix only): at the first write, and before clap prints `--help`/`--version`, check fd 1 once
  with `fcntl(1, F_GETFL)` (`libc` is already a dependency, `Cargo.toml:117`). If it succeeds and the access mode is
  neither `O_WRONLY` nor `O_RDWR`, record `io::Error::from_raw_os_error(libc::EBADF)` as the error and write nothing.
  A failing `fcntl` is not recorded. Non-unix: no check (same as md). Guard it with `#[cfg(unix)]` so the
  binary still builds for the release windows row (`release.yml` `x86_64-pc-windows-msvc`).
- A stdout closed outright (`>&-`) is not an error: std reopens fd 1 on `/dev/null` at start-up, so the output is
  discarded and the code is the verb's own (R18, a guard row, already GREEN at `dbac472f`; md behaves the same).
- Stderr writes are unchanged: a failed stderr write is ignored, as today.

### 3.2 `main` maps the exit code once (`main.rs:281-300`)

After the verb returns and the sink is flushed:

- No recorded error (including a closed pipe): exit code unchanged from today, no new stderr line.
- A recorded error: first the lines `main` prints today (for `Err(e)`: `{e}` at `main.rs:291`; for
  `RepairShortCircuit` nothing), then the one line `mnemonic: cannot write the output: <error>` on stderr, then
  `mlock::report_at_exit()` as today. The exit code becomes:

| Verb result | Code when stdout failed |
|---|---|
| `Ok(0)` | `1` |
| `Ok(5)` or `Err(RepairShortCircuit { exit_code: 5 })` (`REPAIR_APPLIED`: the corrected chunks on stdout are the result) | `1` |
| exit 4, any source (`Ok(4)` or an `Err` whose `exit_code()` is 4) | **Q1** (section 7) |
| any other code (`1`, `2`, `3`, `64`, every other `Err(e)`) | kept: the verb was already failing |

The order (the verb's own error line before the write-error line) is md's (`md-cli/src/main.rs:924-936`: the verb's
`eprintln!` runs before `out::exit_code`). PROPOSED (architect confirms) only in so far as tk has not stated it
before; it is not new wording.

### 3.3 `--help` / `--version` (`main.rs:211-217`)

Replace `e.print().ok()`: when clap's error is a stdout display (`!e.use_stderr()`), run the read-only check first
(skip printing when it records EBADF); if `e.print()` returns an error, record it in the sink's state (BrokenPipe
only closes). Then the exit code is today's (`0` for help/version, `64` otherwise) passed through the 3.2 mapping, so
`mnemonic --help > /dev/full` exits 1 with the contract line and `mnemonic --help | head -1` exits 0. A clap usage
error (`use_stderr()`) is printed on stderr as today and exits 64; its print failure stays ignored.

### 3.4 The 34 discarded stdout writes

Replace each `.ok()` in the section 2 item 1 list with `?` (for `serde_json::to_writer`, map the
`serde_json::Error` into `ToolkitError::Io` via `std::io::Error::from`; serde_json implements
`From<serde_json::Error> for io::Error`). Under the sink these never fire in the binary; they stop each verb hiding a
write error from a direct caller (the bin unit tests that pass a `Vec<u8>` or a failing writer), and R22 keeps the
idiom from returning. Fix the stale comment at `gui_schema.rs:1330-1332` ("Match the `.ok()` pattern ...") to say the
write is propagated. No other text changes.

### 3.5 What does not change

Every verb's stdout bytes and stderr lines when stdout works; all exit codes when stdout works; `gui-schema` output
(no flag added or removed, so no `mnemonic-gui` schema-mirror or `docs/manual-gui` lockstep); the argv guard
(`main.rs:202-209`) and the non-UTF-8 refusal (`main.rs:193-199`), which print on stderr only.

## 4. Manual lockstep (toolkit `docs/manual/src/40-cli-reference/41-mnemonic.md`)

Append after the last line (today L5215), mirroring `42-md.md:797-799`, `43-ms.md:896-898`, `44-mk-cli.md:621`:
a blank line, `---`, a blank line, `## When stdout fails`, a blank line, then this paragraph (PROPOSED (architect
confirms); it is md's sentence with `mnemonic` and a tk example, plus the exit-4 clause Q1 decides):

> A reader that closes the pipe early (`mnemonic addresses … | head -3`) only stops the output: the exit code is the
> verb's own. Any other write error on stdout (a full disk, `> /dev/full`), from a verb or `--help`, prints
> `mnemonic: cannot write the output: <error>` on stderr and exits 1; a verb that was already failing keeps its own
> code.

Q1 answer (A) adds nothing (exit 4 is "already failing"). Answer (B) or (C) appends one sentence, PROPOSED:
(B) `` `repair`'s and `inspect`'s exit 4 (the candidates or the partial decode on stdout are the result) also become 1. ``
(C) `` Exit 4 and exit 5 also become 1. `` (Exit 5 becomes 1 in every answer; md and mk state it the same way, under
"a verb that was already failing", so no sentence is needed for 5.)

No flag changes, so `docs/manual/tests/lint.sh` flag coverage is unaffected; the builder runs it anyway (gate, 8).
CHANGELOG: the `changelog-check` job wants a section; the builder adds the entry per the repo's release process.

## 5. RED rows (run at `dbac472f`, signet, public test phrases only)

Command for every row: `design/fix-tk-stdout-write-errors-red.sh target/debug/mnemonic` (committed with this doc; it
states what it does not cover). It prints `<row> rc=<exit> stderr=<last stderr line>`. `P` = the BIP-39 vector
`abandon` x11 + `about`; `Z` = `zoo` x11 + `wrong`; the cards are `bundle --template bip84 --network signet` on `P`
(listed in the script). `W` below = `mnemonic: cannot write the output: No space left on device (os error 28)`.

| Row | Command (stdin; stdout) | At `dbac472f` (measured) | GREEN |
|---|---|---|---|
| R1 | `derive-child --from phrase=- --application hex --length 16 --index 0` (P; `>/dev/full`) | rc=0 | rc=1, last stderr line `W` |
| R2 | `convert --from phrase=- --to entropy` (P; `>/dev/full`) | rc=0 | rc=1, `W` |
| R3 | `convert --from phrase=- --to entropy --json` (P; `>/dev/full`) | rc=0 | rc=1, `W` |
| R4 | `bundle --template bip84 --network signet --slot @0.phrase=-` (P; `>/dev/full`) | rc=0 | rc=1, `W` |
| R5 | `gui-schema` (`>/dev/full`) | rc=0, stderr empty | rc=1, stderr exactly `W` |
| R6 | `addresses --from phrase=- --address-type p2wpkh --network signet --count 3` (P; `>/dev/full`) | rc=1, `error: I/O error: No space left on device (os error 28)` | rc=1, `W`; no line starts `error: I/O error` |
| R7 | same, `--count 2000`, stdout piped to `head -c 1` | rc=1, `error: I/O error: Broken pipe (os error 32)` | rc=0; no `cannot write` and no `I/O error` line |
| R8 | `gui-schema`, piped to `head -c 1` | rc=0 | rc=0, stderr empty (guard) |
| R9 | `--help` (`>/dev/full`) | rc=0, stderr empty | rc=1, stderr exactly `W` |
| R10 | `--version` (`>/dev/full`) | rc=0, stderr empty | rc=1, stderr exactly `W` |
| R11 | `gui-schema` (`1</dev/null`) | rc=0, stderr empty | rc=1, stderr exactly `mnemonic: cannot write the output: Bad file descriptor (os error 9)` |
| R12 | `convert --from phrase=- --to entropy` (P; `1</dev/null`) | rc=0 | rc=1, last line as R11 |
| R13 | `verify-bundle --allow-argv-secret --template bip84 --network signet --ms1 … --mk1 … --md1 … --slot @0.phrase=-` (P; `>/dev/full`) | rc=0, stderr empty | rc=1, stderr exactly `W` |
| R14 | R13 with stdin Z (mismatch, exit 4) (`>/dev/full`) | rc=4, stderr empty | Q1 (A),(B): rc=4, stderr exactly `W`. (C): rc=1, `W` |
| R15 | `repair --mk1 <K1 with its last char f→q> <K2>` (`>/dev/full`) | rc=1, `error: I/O error: …(os error 28)` | rc=1, `W`; no `error: I/O error` line |
| R16 | `repair --ms1 -` (stdin: the ms1 card, last char f→q; exit 4 when stdout works) (`>/dev/full`) | rc=1, `error: I/O error: …(os error 28)` | Q1 (A): rc=4, `W`. (B),(C): rc=1, `W`. No `error: I/O error` line in any answer |
| R17 | R1 into a regular file under `sh -c 'trap "" XFSZ; ulimit -f 0; exec …'` (the consultant's case) | rc=0, bytes=0 | rc=1, bytes=0, `mnemonic: cannot write the output: File too large (os error 27)` |
| R18 | `gui-schema >&-` | rc=0, stderr empty | rc=0, stderr empty (guard: std reopens fd 1 on /dev/null) |

Measured output at `dbac472f`, verbatim except the long advisory line cut to its start (`warning: stdout carries
private key material (can spend) — …` is R1-R4's and R12's last line):

    R1-derive-child-full rc=0   R2-convert-text-full rc=0   R3-convert-json-full rc=0   R4-bundle-full rc=0
    R5-gui-schema-full rc=0 stderr=
    R6-addresses-full rc=1 stderr=error: I/O error: No space left on device (os error 28)
    R7-addresses-closed-pipe rc=1 stderr=error: I/O error: Broken pipe (os error 32)
    R8-gui-schema-closed-pipe rc=0 stderr=
    R9-help-full rc=0 stderr=          R10-version-full rc=0 stderr=
    R11-gui-schema-read-only rc=0 stderr=          R12-convert-read-only rc=0
    R13-verify-bundle-ok-full rc=0 stderr=          R14-verify-bundle-mismatch4-full rc=4 stderr=
    R15-repair-applied5-full rc=1 stderr=error: I/O error: No space left on device (os error 28)
    R16-repair-unverified4-full rc=1 stderr=error: I/O error: No space left on device (os error 28)
    R17-derive-child-fsize0 rc=0 bytes=0 stderr=
    R18-gui-schema-closed-fd rc=0 stderr=

In-crate rows (no binary run; the builder sees each fail before the fix, by test, not by text):

| Row | What | RED at `dbac472f` | GREEN |
|---|---|---|---|
| R19 | sink over a writer that accepts 4 bytes then returns `ENOSPC`: write 10 bytes, then 10 more | does not compile (no sink) | first write returns `Ok`, inner holds 4 bytes, recorded error is `os error 28`, second write reaches the inner writer 0 times |
| R20 | sink over a writer whose writes succeed and whose `flush` returns `ENOSPC` | does not compile | writes `Ok`, `flush` `Ok`, recorded error `os error 28` |
| R21 | exit mapping table: no error: every code kept; BrokenPipe only: codes 0,1,2,3,4,5,64 kept; ENOSPC: 0→1, 5→1, 1/2/3/64 kept, 4 per Q1 | does not compile | as stated |
| R22 | source scan of `crates/mnemonic-toolkit/src`: no line matches `(writeln!\|write!)\(stdout` or `to_writer(_pretty)?\(&mut \*stdout` and contains `.ok()` | FAILS, lists the 34 sites | passes |

## 6. Tests

New, one test function per row, named exactly (TRACE):

- `crates/mnemonic-toolkit/tests/stdout_write_errors.rs`, `#![cfg(unix)]`, runs `env!("CARGO_BIN_EXE_mnemonic")`
  with `Stdio` from `/dev/full`, `/dev/null` opened read-only, a regular file under `sh -c` (R17), a pipe whose
  reader is dropped after one byte (R7, R8):
  `r1_derive_child_dev_full_exits_1`, `r2_convert_text_dev_full_exits_1`, `r3_convert_json_dev_full_exits_1`,
  `r4_bundle_dev_full_exits_1`, `r5_gui_schema_dev_full_exits_1`, `r6_addresses_dev_full_prints_the_contract_line`,
  `r7_addresses_closed_pipe_keeps_exit_0`, `r8_gui_schema_closed_pipe_keeps_exit_0`, `r9_help_dev_full_exits_1`,
  `r10_version_dev_full_exits_1`, `r11_gui_schema_read_only_stdout_exits_1`,
  `r12_convert_read_only_stdout_exits_1`, `r13_verify_bundle_ok_dev_full_exits_1`,
  `r14_verify_bundle_mismatch_dev_full`, `r15_repair_applied_dev_full_exits_1`,
  `r16_repair_unverified_dev_full`, `r17_derive_child_fsize0_exits_1`, `r18_gui_schema_closed_fd_keeps_exit_0`.
  R7 must not be flaky: the reader closes after the first byte and `--count 2000` writes far more than a pipe buffer
  (64 KiB on Linux); assert only rc and the absence of the two lines.
- `crates/mnemonic-toolkit/src/out_sink.rs` `#[cfg(test)]`: `r19_sink_prefix_then_error_records_first_error_and_stops`,
  `r20_sink_flush_only_failure_is_recorded`, `r21_exit_code_mapping`. The sink's state must be per instance (not a
  process static) or R19-R21 must be one test, as md's `out.rs` does; the builder picks.
- `crates/mnemonic-toolkit/tests/stdout_write_errors.rs`: `r22_no_discarded_stdout_write_in_src` (reads the files
  under `src/` with `std::fs`, no shell).

Existing tests that change: none. Checked: no test under `crates/` or script under `scripts/` names `/dev/full`,
`Broken pipe`, `BrokenPipe`, `os error 28` or `cannot write the output`, and no test pins `error: I/O error:` on a
stdout failure (`grep -rn` at `dbac472f`).

## 7. Open question (STOP)

**Q1. When stdout fails, does tk's exit 4 become 1?** T6 keeps the code of "a verb already failing" and md turns
"a code that says the output is the result" into 1. tk's exit 4 is both kinds: a failing verdict for
`verify-bundle`/`restore`/`import-wallet` mismatch (`error.rs` `BundleMismatch`, `RestoreMismatch`,
`ImportWalletSeedMismatch`, `Ok(4)` at `verify_bundle.rs:701,983`, `import_wallet.rs:447,1408`) and `xpub-search`
no-match; and "the output is the result, verify it" for `repair` (candidates printed, manual 41-mnemonic.md L3464)
and `inspect` partial decode (`inspect.rs:177`, a mirror of md's partial-decode 4, which md turns into 1). mk keeps
its verify-mismatch 4. Today a `?`-propagating verb's 4 already becomes 1 on a full disk (R16), and an `.ok()` verb's 4
stays 4 (R14).

- (A) **Recommended.** Keep 4 everywhere. tk's manual defines 4 as VERIFY-ME/mismatch, a "do not trust" result that
  no caller reads as success, so the false-success defect is closed without it; one rule, no per-verb table. Changes
  R16 from today's 1 to 4.
- (B) 4 becomes 1 for `repair` and `inspect` only (md's reading); `main` keys on the verb. Adds the 4.(B) sentence.
- (C) 4 becomes 1 everywhere (md's code list 0/4/5 literally). Turns a mismatch verdict into 1.

## 8. Gate (builder, one run to a file, then grep)

    export PATH=$HOME/.cargo/bin:$PATH; cargo --version   # 1.85.0
    cargo +1.95.0 fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings \
      && cargo nextest run --locked --workspace --build-jobs 6 -j 6 > gate.log 2>&1; grep -E '^\s+(PASS|FAIL)|Summary' gate.log | tail -5
    cargo build --locked --bin mnemonic && design/fix-tk-stdout-write-errors-red.sh target/debug/mnemonic
    cargo check --locked --bin mnemonic --target x86_64-pc-windows-msvc   # if the target is installed; else the release CI row
    make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=… MS_BIN=… MK_BIN=…

Plus every other job of `.github/workflows/rust.yml`, `manual.yml`, `changelog-check.yml` locally or green on the PR
head (RULES.md build gate). Not covered by this gate: Windows behaviour of the sink (no read-only check there, as md),
stderr write failures, file outputs.

## 9. FOLLOWUPS

Closes: none (bh-tk-I2 has no line in `design/FOLLOWUPS.md` at `dbac472f`; grep for `stdout`, `write error`,
`dev/full`, `bh-tk-I2`). Files, after the PR merges (direct-push; not in the PR):

- F1 `- [tk-file-output-write-error-text] status=open grade=M kind=ux owner=none filed=2026-11-22 src=design/FIX-tk-stdout-write-errors.md#1 :: T6 also names --out: tk file outputs (--out, --json-out, gen-man --out) print "error: I/O error: <e>" on a write error, not "mnemonic: cannot write the output: <e>"; align the wording per flag.`
