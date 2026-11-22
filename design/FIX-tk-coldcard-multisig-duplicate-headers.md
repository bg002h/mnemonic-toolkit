# FIX-tk-coldcard-multisig-duplicate-headers: a repeated Coldcard header must not change the wallet (bughunt-1119 bh-tk-I1)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: **small** (one Opus review round). Funds path (the
quorum, script wrapper and origin fingerprint of an imported multisig wallet); the grader calls it class B
(`grade-tk.md` L51), the architect table row 24 says small with no blocker.
Base: origin/master `04c56bfa`. Key: bh-tk-I1 (grade I, CONFIRMED; P4/B4 keep it I). Sources:
`design/agent-reports/bug-hunt-2026-11-19.md` section I1 (L13-L17); coordinator
`agent-reports/bughunt-1119/grade-tk.md` L25, L51; `triage-tk-t1.md` L12, L18; `xref-tk.md` L15.

**Status: STOPs ruled (architect-bh-batch34 #24): STOP-1 A, STOP-2 A plus Rule C, STOP-3 A (grade I).** Every new holder-visible text is confirmed.

## 1. Scope

`mnemonic import-wallet` reads a Coldcard multisig setup file (`--format coldcard-multisig`, auto-sniff, and the
`multisig_file` field of `--format jade`, which calls the same parser: `wallet_import/jade.rs` L103-L125). The parser
is `wallet_import/coldcard_multisig.rs::parse_text` (L172). Every rule below lives in that one function, so all
three entry points get it.

External fact (checked 2026-11-22 against Coldcard `firmware` master, `shared/multisig.py`): `from_simple_text`
lowercases each label and lets a repeated `name`/`policy`/`format`/`derivation` line overwrite the earlier one; it
ignores a label `xfp`; `render_export` writes `Name:` and `Policy:` once, `Format:` at most once, and a
`Derivation:` line only immediately before the key line whose path changes. No Coldcard export repeats a header or
writes two `Derivation:` lines in a row. The toolkit's own emit (`wallet_export/coldcard.rs` L376-L395) does not
either.

## 2. Defect at `04c56bfa` (measured; section 5 lists the rows)

1. L225-L235: `Name`, `Policy`, `Format` and `XFP` each assign `Some(..)` on every occurrence, so the last line
   wins. A second `Policy: 1 of 3` turns a 2-of-3 file into `sortedmulti(1,` with exit 0 (R1, R3, R4); a second
   `Format: P2SH-P2WSH` turns `wsh(` into `sh(wsh(` (R5); a second `XFP:` changes a cosigner's origin fingerprint
   (R7); a second `Name:` renames the wallet (R8). Only the ordinary watch-only note reaches stderr.
2. L218-L219 match keys case-sensitively, so a `policy: 1 of 3` line is kept as a dropped field and the toolkit
   imports 2-of-3, while Coldcard reads the same file as 1-of-3 (R11).
3. L237-L248 (new finding, not in the grade): a `Derivation:` line becomes the shared path only if it is the first;
   every `Derivation:` is also staged for the next cosigner. Two `Derivation:` lines with no cosigner line between
   them give cosigner 1 the second path and cosigners 2..N the first (R15:
   `paths=48'/0'/9'/2',48'/0'/0'/2',48'/0'/0'/2'`); Coldcard applies the second path to all three. Addresses are
   unchanged (the xpubs carry them); the key origins, and so the signing metadata, differ from what Coldcard reads.
4. L240-L242, L348-L351: a cosigner line with no `Derivation:` line of its own takes the FIRST `Derivation:` path, while Coldcard (`from_simple_text` L698, L720) takes the most recent one. Coldcard's `render_export` writes `Derivation:` only when the path changes, so its export of a wallet whose paths are A, B, B gives the third cosigner A here and B on the device (R18), exit 0.

## 3. Spec delta (what the builder changes)

All in `parse_text`'s line loop (L211-L305), before the existing `match key` assigns anything. The parser stays
single-pass: the first refusal in line order is the one printed. Line numbers are the existing `line_no` (1-based,
counting blank and comment lines, after CRLF to LF). Exit code 2 (`ToolkitError::ImportWalletParse`, `error.rs`
L673), stdout empty, no envelope; this is the code of every other coldcard-multisig parse refusal (SPEC §2.4).

**Rule H (singleton headers; STOP-1, STOP-2).** `Name`, `Policy`, `Format` and `XFP` are singleton headers. A key
line whose key equals one of them ignoring ASCII case is an occurrence of that header. The first occurrence records
its line number. Any later occurrence is refused, whatever its value and before its value is parsed (so a malformed
second `Policy:` gets this refusal, not the shape error). The first occurrence is otherwise handled as today: an
exact-case key is parsed; a case-variant key (for example `policy:`) is still kept in `dropped_fields` and a file
whose only `Policy` line is `policy:` is still refused as `missing \`Policy:\` header`. Text, with `<L>` the
repeated line, `<F>` the first, and `<Header>` the canonical spelling `Name`, `Policy`, `Format` or `XFP`:

    error: import-wallet: coldcard-multisig: parse error: line <L>: repeated `<Header>:` header (first given at line <F>); a setup file gives each header once

Confirmed (architect-bh-batch34). The detail string (without `error: `) is the `ImportWalletParse` payload, in the
same shape as L297-L301.

**Rule D (two `Derivation:` lines with no cosigner between; STOP-3).** A `Derivation:` line (exact case, as today)
is refused when the previous `Derivation:` line has not yet been taken by a cosigner line (a `<XFP>: <xpub>` line or a
bare xpub). Blank, comment and unknown-key lines do not count as cosigner lines. Equivalently: refuse when
`pending_per_cosigner_path` is `Some` at L237; record the line number wherever L247 stages the path. Text:

    error: import-wallet: coldcard-multisig: parse error: line <L>: `Derivation:` line follows the one at line <F> with no cosigner line between them

Confirmed (architect-bh-batch34). Per-cosigner `Derivation:` lines, each followed by its key line, stay accepted (R13).

**Rule C (case-variant Derivation key; STOP-2 ruling).** A key line whose key equals `Derivation` ignoring ASCII case but is not exactly `Derivation` is refused, checked after Rule H and before Rule D (Coldcard lowercases labels and reads it as a path line; this parser would drop it). Text, with `<key>` the key as written:

    error: import-wallet: coldcard-multisig: parse error: line <L>: `<key>:` must be spelled `Derivation:`

Confirmed (architect-bh-batch34). A lone `xfp:` key stays a dropped field (Coldcard ignores an `xfp` label).

**Rule P (most recent path).** Every `Derivation:` line sets the current path. A cosigner line (`<XFP>: <xpub>` or bare xpub) with no pending per-line path takes the current path at that line, not the first path of the file: keep a `current_derivation`, set it on every accepted `Derivation:` line, and record `per_line_path = pending.take().or_else(|| current_derivation.clone())` in the `<XFP>:` arm and `per_line_path = current_derivation.clone()` in the L285-L295 bare-xpub arm. `shared_derivation` keeps its role only for cosigners before any `Derivation:` line.

Nothing else changes: the sniff (L139-L152), the cosigner-count check (L324) and the XFP truth table (§11.4.1).

**SPEC lockstep:** `design/SPEC_wallet_import_v0_28_0.md` §11.4 "Parse contract" (L417) gains, after its first
sentence: "Each of `Name:`, `Policy:`, `Format:` and `XFP:` appears at most once; a repeat, compared ignoring the
key's ASCII case and whatever its value, is `ImportWalletParse` (exit 2) naming both line numbers. Two `Derivation:`
lines with no cosigner line between them are refused the same way. A `Derivation` key spelled in another letter case is refused. A cosigner line takes the most recent `Derivation:` path."

## 4. Product points (ruled A by architect-bh-batch34)

B4 decides the grade only; B2 (repeated key records refused, exit 64) is ruled for `mk encode` key records, not
for setup-file headers; B1 (normalise BOM and CRLF only) is ruled for near-canonical card input. None decides these.

- **STOP-1: does an identical repeat refuse, and is `Name:` in the rule?** The bug report and grader ask to refuse a
  second *different* `Policy:`/`Format:`/`XFP:`. Options: (A) refuse every repeat of `Name:`, `Policy:`, `Format:`,
  `XFP:`, same value or not (Rule H as written; R8-R10 refuse); (B) refuse only a repeat whose parsed value differs
  (`2-of-3` after `2 of 3` accepted), `Name:` excluded; then the text becomes "line <L>: `<Header>:` header
  `<value>` contradicts line <F> (`<first value>`)" and R8-R10 stay exit 0; (C) as B with `Name:` included.
  **Recommend A:** no Coldcard or toolkit export repeats a header, it is one rule with one text (the same choice B2
  made for key records), and a second `Name:` is as much a sign of a spliced file as a second `Policy:`. **Ruled A (architect-bh-batch34).**
- **STOP-2: case-variant keys (`policy: 1 of 3`, R11).** Options: (A) count a case-variant key as an occurrence for
  the repeat check only (Rule H as written); (B) also refuse a lone case-variant header key; (C) leave R11 out and
  file a FOLLOWUPS line. **Recommend A:** it closes the one case where Coldcard and the toolkit read different
  quorums from one file, and changes nothing for a file without a repeat (B would refuse lone `xfp:` lines that
  Coldcard ignores). **Ruled A (architect-bh-batch34).** Extended by Rule C to a case-variant `Derivation` key.
- **STOP-3: two `Derivation:` lines with no cosigner between (R15; new finding).** Options: (A) refuse (Rule D, in
  this FIX); (B) apply the later path to every following cosigner, as Coldcard does; (C) leave it and file a
  FOLLOWUPS line (proposed grade I: the origins of N-1 cosigners differ from the device's reading, addresses do
  not). **Recommend A:** no export writes that shape, refusal keeps one reading, and it is the same root (a repeated
  header silently resolved). **Ruled A (architect-bh-batch34).** Grade I (P4).

## 5. RED rows (run at `04c56bfa`; public xpubs from committed fixtures, no seed)

Command: `cargo build --locked --bin mnemonic && design/fix-tk-coldcard-dup-headers-red.sh target/debug/mnemonic`
(run from the repo root). Base fixture `tests/export_wallet/coldcard_multisig_2of3_wsh.txt` (7 lines: Name L1,
Policy L2, Derivation L3, Format L4, three `<XFP>: <xpub>` lines). Every refused row expects `out=0` and empty
quorum, wrap, fp0, paths and name columns; `err=` shows the last stderr line, which is the whole refusal.

| Row | Input | At `04c56bfa` (seen) | Expected after the fix |
|---|---|---|---|
| R1 | fixture + `Policy: 1 of 3` (L8), `--format coldcard-multisig` | rc=0 `quorum=sortedmulti(1` | rc=2 `line 8: repeated \`Policy:\` header (first given at line 2)` |
| R2 | `Policy: 1 of 3` (L1) + fixture | rc=0 `sortedmulti(2` | rc=2 `line 3: repeated \`Policy:\` header (first given at line 1)` |
| R3 | as R1, auto-sniff (no `--format`) | rc=0 `sortedmulti(1` | rc=2, R1's line |
| R4 | as R1 inside jade `multisig_file`, `--format jade` | rc=0 `sortedmulti(1` | rc=2, R1's line |
| R5 | fixture + `Format: P2SH-P2WSH` (L8) | rc=0 `wrap=sh(wsh(` | rc=2 `line 8: repeated \`Format:\` header (first given at line 4)` |
| R6 | `Format: P2SH-P2WSH` (L1) + fixture | rc=0 `wsh(` | rc=2 `line 5: repeated \`Format:\` header (first given at line 1)` |
| R7 | per-cosigner file, bare first xpub, `XFP: 0000AAAA` L4 + `XFP: 0000BBBB` L5 | rc=0 `fp0=0000bbbb` | rc=2 `line 5: repeated \`XFP:\` header (first given at line 4)` |
| R8 | fixture + `Name: other-wallet` (L8) | rc=0 `name="other-wallet"` | rc=2 `line 8: repeated \`Name:\` header (first given at line 1)` (STOP-1 B: rc=0) |
| R9 | fixture + `Policy: 2 of 3` (L8) | rc=0 `sortedmulti(2` | rc=2, R1's line (STOP-1 B/C: rc=0) |
| R10 | fixture + `Policy: 2-of-3` (L8) | rc=0 `sortedmulti(2` | rc=2, R1's line (STOP-1 B/C: rc=0) |
| R11 | fixture + `policy: 1 of 3` (L8) | rc=0 `sortedmulti(2` (Coldcard reads 1-of-3) | rc=2, R1's line (STOP-2 C: rc=0) |
| R12 | fixture unchanged (control) | rc=0 `sortedmulti(2` `wsh(` `fp0=5436d724` | unchanged |
| R13 | per-cosigner shape, three `Derivation:` + `<XFP>: <xpub>` pairs (control) | rc=0 paths `48'/0'/0'/2',48'/0'/1'/2',48'/0'/2'/2'` | unchanged |
| R14 | R7 with one `XFP: 0000AAAA` (control) | rc=0 `fp0=0000aaaa` | unchanged |
| R15 | fixture with `Derivation: m/48'/0'/9'/2'` inserted as L4 | rc=0 `paths=48'/0'/9'/2',48'/0'/0'/2',48'/0'/0'/2'` | rc=2 `line 4: \`Derivation:\` line follows the one at line 3 with no cosigner line between them` (STOP-3 B/C: differs) |
| R16 | fixture with `derivation: m/48'/0'/9'/2'` inserted as L4 | rc=0 paths=48'/0'/0'/2',48'/0'/0'/2',48'/0'/0'/2' (Coldcard reads 9' for all) | rc=2 `` line 4: `derivation:` must be spelled `Derivation:` `` |
| R17 | R13 with its L6 key spelled `derivation:` | rc=0 paths=48'/0'/0'/2',48'/0'/0'/2',48'/0'/2'/2' (Coldcard reads 1' for cosigner 2) | rc=2 `` line 6: `derivation:` must be spelled `Derivation:` `` |
| R18 | Coldcard render_export shape, paths A, B, B (`Derivation: m/48'/0'/0'/2'`, key 1, `Derivation: m/48'/0'/1'/2'`, keys 2 and 3) | rc=0 paths=48'/0'/0'/2',48'/0'/1'/2',48'/0'/0'/2' | rc=0 paths=48'/0'/0'/2',48'/0'/1'/2',48'/0'/1'/2' |

The builder appends R16 and R17 to `design/fix-tk-coldcard-dup-headers-red.sh` (and changes its header to "rows R1-R17"):

    TWODERLC=$(sed -n 1,3p "$FX"; printf 'derivation: m/48'"'"'/0'"'"'/9'"'"'/2'"'"'\n'; sed -n '4,$p' "$FX")
    text R16-derivation-lowercase-after-shared  "${TWODERLC}
    "
    text R17-derivation-lowercase-per-cosigner  "$(printf '%s\n' "$PERCOS" | sed '6s/^Derivation:/derivation:/')
    "

Each expected refusal is the full stderr line `error: import-wallet: coldcard-multisig: parse error: <detail>` with
the Rule H, Rule C or Rule D detail. The author applied a scratch implementation of the recommended options (not
committed) and saw exactly the "Expected" column for R1-R15. R16 and R17: measured by the architect (architect-bh-batch34 #24) with Rule C. R18: measured by the r1 reviewer on a scratch copy (review-fix-bh-tk-coldcard-dup-headers-r1).

## 6. Tests

Tests that change: **none**. Measured: with the scratch implementation, `cargo nextest run --locked -p
mnemonic-toolkit` ran 4208 tests, 4208 passed, 37 skipped.

New tests, in `crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard_multisig.rs` (they spawn the binary; blobs
built from the fixture text in the test, as the RED script does). Each refusal test asserts exit 2, empty stdout,
and that stderr contains the row's full detail string (line numbers included). Names carry the row:
`dup_headers_r1_policy_conflict_appended`, `dup_headers_r2_policy_conflict_prepended`,
`dup_headers_r3_policy_conflict_autosniff`, `dup_headers_r4_policy_conflict_jade`,
`dup_headers_r5_format_conflict_appended`, `dup_headers_r6_format_conflict_prepended`,
`dup_headers_r7_xfp_conflict`, `dup_headers_r8_name_conflict`, `dup_headers_r9_policy_identical_repeat`,
`dup_headers_r10_policy_repeat_other_spelling`, `dup_headers_r11_policy_lowercase_key_repeat`,
`dup_headers_r12_control_original_fixture` (exit 0, stdout contains `sortedmulti(2,`),
`dup_headers_r13_control_per_cosigner_derivation` (exit 0, the three paths of R13),
`dup_headers_r14_control_xfp_single` (exit 0, stdout contains `[0000aaaa/48'/0'/0'/2']`),
`dup_headers_r15_derivation_twice_before_cosigners`, `dup_headers_r16_derivation_lowercase_after_shared`, `dup_headers_r17_derivation_lowercase_per_cosigner`, `dup_headers_r18_render_export_path_change_carries_forward`.

## 7. Manual and changelog lockstep

No flag, verb or `--help` text changes, so `docs/manual/src/40-cli-reference/` and the GUI manual
(`docs/manual-gui/src/40-mnemonic/4c-import-wallet.md` L103-L106 lists no refusals) are unchanged.

- `docs/manual/src/45-foreign-formats.md`: after L634 (end of the `m/45'` paragraph, before `### XFP-header
  policy`), a new paragraph: "Each of `Name:`, `Policy:`, `Format:` and `XFP:` appears once. A file that repeats one of them (in any letter case, even with the same value), or that has two `Derivation:` lines with no cosigner line between them, is refused (exit 2) with both line numbers; a `Derivation:` key spelled in another letter case is refused (exit 2) with its line number. Re-export the file from the device."
- `CHANGELOG.md` `## [Unreleased]`, new subsection: "### A repeated Coldcard multisig header is refused" with
  "- **`import-wallet` (`coldcard-multisig`, auto-detected, and `jade`)**: a setup file that repeats `Name:`, `Policy:`, `Format:` or `XFP:`, or has two `Derivation:` lines with no cosigner line between them, now exits 2 naming both lines, and one that spells `Derivation:` in another letter case exits 2 naming that line. The last line used to win silently, so a second `Policy: 1 of 3` imported a 2-of-3 file as 1-of-3.
  - a cosigner line after a later `Derivation:` line now takes that path, as the device does (it used to take the first)"

## 8. Gate (builder, one run to a file, then grep)

    export PATH=$HOME/.cargo/bin:$PATH; cargo --version   # 1.85.0
    cargo +1.95.0 fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings \
      && cargo nextest run --locked --workspace --build-jobs 4 -j 4 > gate.log 2>&1; grep -E '^\s+(PASS|FAIL)|Summary' gate.log | tail -5
    cargo build --locked --bin mnemonic && design/fix-tk-coldcard-dup-headers-red.sh target/debug/mnemonic
    make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk

Plus every other job of `.github/workflows/rust.yml`, `manual.yml`, `changelog-check.yml` locally or green on the
PR head (RULES.md build gate). Not covered: library callers of `parse_text` outside the CLI, other import formats
(bsms, sparrow, specter, electrum), the cosigner-count predicate (bug-hunt L101, separate key), Windows.

## 9. Files the builder changes (for build ordering)

- `crates/mnemonic-toolkit/src/wallet_import/coldcard_multisig.rs`
- `crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard_multisig.rs`
- `design/SPEC_wallet_import_v0_28_0.md`
- `docs/manual/src/45-foreign-formats.md`
- `CHANGELOG.md`
- `design/fix-tk-coldcard-dup-headers-red.sh`

Neighbour #15 (`fix/tk-stdout-write-errors`) also edits `CHANGELOG.md` `[Unreleased]`; no other overlap.

## 10. FOLLOWUPS

Closes: none (bh-tk-I1 has no line in `design/FOLLOWUPS.md` at `04c56bfa`; grep for `bh-tk-I1`, `duplicate`,
`repeated`, `Policy:`). Files: none (STOP-2 and STOP-3 ruled A, architect-bh-batch34).
