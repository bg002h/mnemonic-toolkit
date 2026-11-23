# FIX-tk-word-card-mixed-stdin: `-` must be the only positional word of `word-card --decode` (bughunt-1119 bh-tk-I7)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: **small** (one Opus review round). Not a funds path
(the decoded card is still integrity-checked; the defect is input silently dropped at exit 0).
Base: origin/master `b57187b3`. Key: bh-tk-I7 (grade I, lowest end; CONFIRMED). Architect table row 54, no blocker.
Sources: `design/agent-reports/bug-hunt-2026-11-19.md` section I7 (L49-L53); coordinator
`agent-reports/bughunt-1119/grade-tk.md` L31, L55; `triage-tk-t2.md` L11, L181-L205; `xref-tk.md` L21.

**Status: no STOPs. Two PROPOSED texts (architect confirms): the refusal line (section 3) and the help/manual
wording (section 7).**

## 1. Scope

`mnemonic word-card --decode <WORD>...` resolves its words in `cmd/word_card.rs::collect_words` (L490-L501). If any
positional is `-`, it reads stdin and drops every other positional (L491-L494). The fix refuses that mix before
stdin is read, as the bug report asks ("Reject mixed or repeated dashes before reading", L53). Untouched: encode
mode (positionals are documented as ignored there, L112-L113), `--from -` (`collect_from_sources`, L466-L486, which
is meant to mix `-` with other values), `--decode-plate` (clap `conflicts_with = "words"`, L78).

## 2. Defect at `b57187b3` (measured; section 5 lists the rows)

`word-card --decode nonsense -` with a valid card on stdin exits 0 and prints the card's xpub; so do `- nonsense`,
`- -`, and a full card given as positionals plus `-`. In each case the positional words never reach the decoder.

## 3. Spec delta (what the builder changes)

Rule A. In `collect_words` (L490), inside the `if words.iter().any(|w| w == "-")` branch and before
`stdin.read_to_string`, when `words.len() != 1` return
`ToolkitError::BadInput(format!("word-card: `-` (read the words from stdin) must be the only positional <WORD>; got {} positional values", words.len()))`.

Printed on stderr (via the existing error path, measured with a scratch build):

    error: word-card: `-` (read the words from stdin) must be the only positional <WORD>; got 2 positional values

**PROPOSED (architect confirms).** Exit 1 (`BadInput`, `error.rs` L640), stdout empty, also under `--json` (the
existing `BadInput` path prints the plain line on stderr and nothing on stdout; measured with R9's refusal under
`--json`). `N` is `words.len()`, the count of positional values including every `-`; it is always 2 or more here.
The positional words are not echoed (a holder's card words stay off stderr; they are public, but the count is enough).

Rule B. Nothing else changes: a lone `-` reads stdin as before (empty stdin still gives the existing
`--decode requires the Word-Card words` refusal, exit 1); word-per-arg and one quoted whole-card arg still join and
split as before; a `-` inside a quoted arg is not a dash positional and stays a word (it then fails as an unknown
BIP-39 word, exit 2, as today).

## 4. Product points

None open. The bug report (L53) decides refuse-before-read, mixed in either order and repeated dashes; the grader
(L55) decides the site. The refusal text and the help/manual wording are new holder-visible text, marked PROPOSED.

## 5. RED rows (run at `b57187b3`; public mk1 xpub card, no seed)

Command: `cargo build --locked --bin mnemonic && design/fix-tk-word-card-mixed-stdin-red.sh target/debug/mnemonic`
(repo root). `CARD` = the 92-word card the script encodes from the `cli_word_card.rs` `MK1` constant with
`--parity-words 4`; its xpub is `xpub6CatWdiZ…PW6V` (`MK1_XPUB`). `REFUSAL(N)` = the section 3 line with `got N`.

| Row | Positionals after `--decode` (stdin) | At `b57187b3` (seen) | Expected after the fix |
|---|---|---|---|
| R1 | `--json nonsense -` (CARD) | rc=0 out=479 xpub printed | rc=1 out=0, err=REFUSAL(2) |
| R2 | `nonsense -` (CARD) | rc=0 out=398 xpub printed | rc=1 out=0, err=REFUSAL(2) |
| R3 | `- nonsense` (CARD) | rc=0 out=398 xpub printed | rc=1 out=0, err=REFUSAL(2) |
| R4 | `- -` (CARD) | rc=0 out=398 xpub printed | rc=1 out=0, err=REFUSAL(2) |
| R5 | the 92 card words then `-` (CARD) | rc=0 out=398 xpub printed | rc=1 out=0, err=REFUSAL(93) |
| R6 | `-` (CARD), control | rc=0 out=398 xpub | unchanged |
| R7 | the 92 card words, one per arg (empty), control | rc=0 out=398 xpub | unchanged |
| R8 | CARD as one quoted arg (empty), control | rc=0 out=398 xpub | unchanged |
| R9 | `-` (empty), control | rc=1 out=0 err=`error: word-card: --decode requires the Word-Card words (positional <WORD>... or \`-\` stdin)` | unchanged |
| R10 | unit: `collect_words(["nonsense","-"])` and `(["-","-"])` with a reader that panics on `read` | FAIL: panicked `stdin was read` (word_card.rs test line) | PASS: `BadInput` equal to REFUSAL(2), reader untouched |

R1-R9 run by the script; R10 is the unit test of section 6, run by the author as a scratch addition
(`cargo nextest run --locked -p mnemonic-toolkit --bin mnemonic -E 'test(mixed_stdin_r10)'`: 1 run, 1 failed).
The author applied Rule A as a scratch change (not committed) and saw exactly the "Expected" column for R1-R10.

## 6. Tests

Tests that change: **none** (every existing `word-card --decode` test passes a lone `-`: `cli_word_card.rs` L75,
L109, L365, L389, L402, L444, L517). Measured with the scratch Rule A plus R10: `cargo nextest run --locked -p
mnemonic-toolkit --build-jobs 2 -j 2` ran 4235 tests, 4235 passed, 37 skipped.

New tests (names carry the row):
- `crates/mnemonic-toolkit/tests/cli_word_card.rs` (spawn the binary; CARD built with the file's
  `encode_solo_words(MK1, 4)` helper, L43): `mixed_stdin_r1_junk_then_dash_json_refused`,
  `mixed_stdin_r2_junk_then_dash_refused`, `mixed_stdin_r3_dash_then_junk_refused`,
  `mixed_stdin_r4_dash_twice_refused`, `mixed_stdin_r5_full_card_args_plus_dash_refused` (each: exit 1, empty
  stdout, stderr contains REFUSAL(N) for its row's N), `mixed_stdin_r7_word_per_arg_control` and
  `mixed_stdin_r8_one_quoted_arg_control` (exit 0, stdout contains `MK1_XPUB`). R6 and R9 are already covered
  (`mk1_e2e_round_trip_recovers_same_xpub_not_literal_string` L71; R9 by the existing empty-words refusal) and get no
  new test.
- `crates/mnemonic-toolkit/src/cmd/word_card.rs` `mod tests`: `mixed_stdin_r10_refused_before_reading_stdin`
  (a `Read` impl whose `read` panics; both inputs return `ToolkitError::BadInput` with REFUSAL(2) exactly).

## 7. Help, manual and changelog lockstep (PROPOSED texts, architect confirms)

- `word_card.rs` L110-L112, the `words` doc comment (shown by `--help`), becomes: "Positional Word-Card words for
  `--decode` (a single solo card). Each value is one BIP-39 word; or pass `-` alone to read whitespace-separated
  words from stdin (`-` with any other <WORD> is refused). Ignored in encode mode."
- `docs/manual/src/40-cli-reference/41-mnemonic.md` L5106, the `--decode` row, "Words come from the positional
  `<WORD>...` list or `-`/stdin (whitespace-separated)." becomes "Words come from the positional `<WORD>...` list or
  a lone `-` (stdin, whitespace-separated; `-` with any other `<WORD>` is refused, exit 1)."
- Same file L5214, the exit-1 row: "bad input (no `--from`, non-`mk1` card under `--raid`, `--integrity-bits` below
  floor, I/O error)" becomes "bad input (no `--from`, non-`mk1` card under `--raid`, `--integrity-bits` below floor,
  `-` mixed with other `<WORD>` values, I/O error)".
- `CHANGELOG.md` `## [Unreleased]`, new subsection "### `word-card --decode` refuses `-` mixed with other words" with
  "- **`word-card --decode`**: `-` (read the words from stdin) given together with any other positional word, or
  twice, now exits 1 before reading stdin. It used to read stdin and silently drop the other words."
- No flag changes, so no `mnemonic-gui` schema-mirror change and no `docs/manual/tests/lint.sh` flag delta.
- GUI manual (`docs/manual-gui/src/40-mnemonic/4n-word-card.md` L57-L58, L131-L137 Refusals): **unchanged**. It
  documents the GUI's pinned toolkit tier, and the GUI form passes words as positionals, never stdin. The reviewer
  checks this reading; if the architect wants the Refusals row now, it is: "| `-` together with other positional
  `<WORD>` values | `-` reads every word from stdin, so it must be the only positional word |".

## 8. Gate (builder, one run to a file, then grep)

    export PATH=$HOME/.cargo/bin:$PATH; cargo --version   # 1.85.0
    cargo +1.95.0 fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings \
      && cargo nextest run --locked --workspace --build-jobs 2 -j 2 > gate.log 2>&1; grep -E '^\s+FAIL|Summary' gate.log | tail -5
    cargo build --locked --bin mnemonic && design/fix-tk-word-card-mixed-stdin-red.sh target/debug/mnemonic
    make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk

Plus every other job of `.github/workflows/rust.yml` and `manual.yml` locally or green on the PR head (RULES.md build
gate). Not covered: encode-mode positionals, `--from -`, `--decode-plate`, a stdin read error, Windows.

## 9. Files the builder changes (for build ordering)

- `crates/mnemonic-toolkit/src/cmd/word_card.rs`
- `crates/mnemonic-toolkit/tests/cli_word_card.rs`
- `docs/manual/src/40-cli-reference/41-mnemonic.md`
- `CHANGELOG.md`
- `design/fix-tk-word-card-mixed-stdin-red.sh` (committed with this document; no change expected)

Neighbours: #15 (`fix/tk-stdout-write-errors`) and the other tk rows edit `CHANGELOG.md` `[Unreleased]`;
#36 (`FIX-tk-word-card-stop-sign-recovery`) fixes `crates/wc-codec/src/pipeline.rs` (triage-tk-t2 L311) and may add
tests to `cli_word_card.rs` (append-only; order the builds).

## 10. FOLLOWUPS

None closed (no FOLLOWUPS line records bh-tk-I7 at `b57187b3`), none filed.
