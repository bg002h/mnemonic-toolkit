# FIX-tk-env-dash-stdin-order: an `@env:` value of `-` must not switch the source to stdin (bughunt-1119 bh-tk-I5)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: **small** (one Opus review round).
Base: origin/master `04c56bfa`. Key: bh-tk-I5 (grade I, CONFIRMED; wrong seed, exit 0). Sources:
`design/agent-reports/bug-hunt-2026-11-19.md` section I5; coordinator `agent-reports/bughunt-1119/grade-tk.md` L29,
L53; `triage-tk-t2.md` L9, L17. No T-ruling names this row; the grader's FIX line (grade-tk.md L53) is the scope:
"decide stdin from the value as written, before `@env:` resolution, in derive-child and convert, and grep every
`== "-"` check after `resolve_env_sentinels`". The precedent is F-687 (`--passphrase`, every subcommand) and F-689
(`verify-bundle --ms1`): "a variable whose value is `-` is that literal value, not stdin".

**Status: complete.** No STOP. Two holder-visible texts are PROPOSED (architect confirms): the manual sentence (4.1)
and the CHANGELOG entry (4.2). Every refusal a GREEN row prints is an existing text.

## 1. The rule

R-1. On every secret-bearing value that accepts both `-` (stdin) and `@env:VAR`, whether the value is read from stdin
is decided from the value **as written** on the command line, before `@env:` resolution. `@env:VAR` always yields
the value of `VAR`; that value is never stdin, even when it is exactly `-`. It then goes where an inline value goes
(the same parsers, the same errors).

R-2. The single-stdin guards (the "one stdin per invocation" refusals and stdin counts) count only values written as
`-`. An `@env:` value of `-` is not a stdin reader.

R-3. The argv-secret advisory `warning: secret material on argv (--from <node>=) — ...` on `seed-xor split`,
`ms-shares split` and `slip39 split` is judged on the value as written: it fires for an inline value only, not for
`-` or `@env:VAR` (as every `combine`, `convert`, `bundle`, `addresses` and `restore` already do). Same root (the
check runs on the resolved value) and the same lines as R-1, so it rides this FIX.

R-4 (unchanged, pinned). Text read from stdin is never `@env:`-resolved.

## 2. Defect at `04c56bfa` (measured; section 5 lists the rows)

Every site below resolves `@env:` first and then tests the **resolved** value for `-`:

| Command | `@env:` resolution | `-` tests on the resolved value |
|---|---|---|
| `convert --from` | `cmd/convert.rs:838-844` (`resolve_env_sentinels`, L1974-2000) | L876 (`primary_uses_stdin`, single-stdin guards L879-899), L926 (read) |
| `derive-child --from` | `cmd/derive_child.rs:126-132` (L395-409) | L136 (guard), L145 (read) |
| `bundle --slot @N.<secret>=` | `cmd/bundle.rs:243-249` (L2991-3010) | L251 `needs_stdin_substitution` (L2969-2971), L3024 `apply_stdin_substitutions` and its guard L3025-3030, via `SlotInput::is_stdin_sentinel` (`slot_input.rs:114-116`) and `apply_slot_stdin` (`slot_input.rs:208-216`) |
| `verify-bundle --slot @N.<secret>=` | `cmd/verify_bundle.rs:348-354` (L2152-2174) | L440 `needs_stdin_substitution` (L2125-2127), L2185 `apply_stdin_substitutions`, same `slot_input.rs` helpers. (Its `refuse_second_stdin` at L318-342 and `--ms1 -` at L312, L377 already use the value as written.) |
| `seed-xor split --from` | `cmd/seed_xor.rs:110-116` (L141-150) | L176 (advisory), L180 (read) |
| `seed-xor combine --share` | `cmd/seed_xor.rs:129-135` (L152-162) | L276 (stdin count), L296 (read) |
| `ms-shares split --from` | `cmd/ms_shares.rs:156-162` | L268 (advisory), L279 (read) |
| `ms-shares combine --share` | `cmd/ms_shares.rs:174-180` | L362 (stdin count), L373 (read) |
| `slip39 split --from` | `cmd/slip39.rs:218-224` (L248-263) | L375 (stdin count), L399 (advisory), L428 (read) |
| `slip39 combine --share` | `cmd/slip39.rs:237-243` (L265-279) | L583 (stdin count), L605 (read) |

Result: with `E=-` and a valid input on stdin, all ten commands use the stdin input and exit 0 (R1-R10); with a
second stdin reader they refuse with a single-stdin error that names a `-` the holder never wrote (S1-S4); the three
splits print the argv advisory for an `@env:` source (A1-A3).

Not affected (surveyed: every `resolve_env_var_sentinel` / `resolve_env_sentinels` caller and every `== "-"` in
`crates/mnemonic-toolkit/src`): `addresses` (`cmd/addresses.rs:117`, decided as written), `restore`
(`cmd/restore.rs:385`, L866, L1482, L4129), every `--passphrase` / `--bip38-passphrase` / `--decrypt-password` (F-687,
`passphrase_input.rs`), `verify-bundle --ms1` (F-689), `import-wallet` (no stdin on `--ms1` / `--slot`; its `-` tests
are file paths), `xpub-search` (`--phrase` has no `-`; `--descriptor-from` is not secret and not `@env:`), and the
commands without `@env:` (`final-word`, `seedqr`, `word-card`, `repair`, `export-wallet`, `electrum-decrypt`
ciphertext).

## 3. Spec delta (what the builder changes)

Behaviour is the spec; the code shape is the builder's. In each command of section 2, take every stdin decision
(guard, count, advisory, read) from the value as written, captured before the resolution call, and thread it to the
later tests. For `--slot` (bundle, verify-bundle) the slot that reads stdin must be identified before resolution
(for example by its position), because after resolution a `-` value is ambiguous; `apply_slot_stdin` and
`is_stdin_sentinel` must not be the deciders on resolved slots. The resolution functions keep resolving every
`@env:` value as today (their errors, `EnvVarMissing`, are unchanged).

What does not change: every refusal text and exit code; a literal `-` still reads stdin (C1, C3); a valid `@env:`
value still works (C2); stdin text is not resolved (C4); `--passphrase` handling; flags; JSON.

Consequences, decided by R-1 (existing behaviour of the literal value, measured with the capture build in 5):
- A phrase source of `-` is a one-word phrase: `error: BIP-39 word count 1 invalid (must be 12, 15, 18, 21, or 24)`,
  exit 1 (convert, derive-child, bundle, verify-bundle, seed-xor split), or the command's own word-count error
  (`ms-shares split`, `slip39 split`, `seed-xor combine`, `slip39 combine`).
- An ms1 share of `-` is separator-only, so `ms-shares combine` drops it as it drops any empty share
  (`cmd/ms_shares.rs:394-398`) and counts the rest (R8: `ms1 not enough shares: have 1, need 2`). This is today's
  behaviour for `--share @env:V` with `V=--`; not changed here (FOLLOWUPS F2).

## 4. Manual and CHANGELOG lockstep

### 4.1 Manual (`docs/manual/src/40-cli-reference/41-mnemonic.md`), PROPOSED (architect confirms)

After the paragraph at L79-82 ("Only **one** input per invocation can come from stdin. ... or `--allow-argv-secret`.")
insert one blank line and this paragraph:

    `@env:VAR`, on every flag that accepts it, means the value of `VAR`: a variable whose value is
    `-` is that one-character value, never a request to read stdin.

No flag changes; `docs/manual/tests/lint.sh` flag coverage is unaffected (the builder runs it anyway, gate 8).

### 4.2 CHANGELOG, PROPOSED (architect confirms)

Insert after `## [Unreleased]` and its blank line (`CHANGELOG.md:9-10`), above the existing
`### A failed stdout write no longer exits 0` block; no version bump:

```
### An `@env:` value of `-` no longer reads stdin

- **`convert`, `derive-child`, `bundle --slot`, `verify-bundle --slot`, `seed-xor`, `ms-shares`, `slip39`**: a
  secret given as `@env:VAR` whose value is exactly `-` was read from stdin instead, so a valid input on stdin was
  used in place of the named variable (exit 0). The value of `VAR` is now always the value, as for `--passphrase`
  since F-687; a `-` value is now handled like any other inline value: refused by the parser, or on
  `ms-shares combine` dropped as an empty share. The split subcommands no longer print the argv-secret warning for
  an `@env:VAR` source.
```

## 5. RED rows (run at `04c56bfa`, signet, public test phrase only)

Command for every row: `design/fix-tk-env-dash-stdin-order-red.sh target/debug/mnemonic` (committed with this doc;
it states what it does not cover). It prints `<row> rc=<exit> out=<first stdout line, 40 chars> err=<last stderr
line>`. Every R/S row sets `E=-` and writes `@env:E`. `P` = the BIP-39 vector `abandon` x11 + `about`; `X`, `M1`,
`M2`, `L1`, `L2` are shares of `P` and the cards are `bundle --template bip84 --network signet` on `P` (listed in the
script). `WC` = `error: BIP-39 word count 1 invalid (must be 12, 15, 18, 21, or 24)`. `ADV` = `warning: secret
material on argv (--from phrase=) — pipe via --from phrase=- to avoid /proc/$PID/cmdline exposure`.

GREEN values were measured with a capture build (`env_sentinel.rs` returning `" -"` for a `-` variable, so every
site sees a non-`-` value that every parser trims back to `-`; not committed). The builder's GREEN run must print
exactly the GREEN column.

| Row | Command (stdin) | At `04c56bfa` (measured) | GREEN |
|---|---|---|---|
| R1 | `convert --from phrase=@env:E --to entropy` (P) | rc=0, out `entropy: 0000...` | rc=1, out empty, err `WC` |
| R2 | `derive-child --from phrase=@env:E --application hex --length 16 --index 0` (P) | rc=0, out `866a12e42e31a09aa8ca4f25a02e999e` | rc=1, out empty, err `WC` |
| R3 | `bundle --template bip84 --network signet --slot @0.phrase=@env:E` (P) | rc=0, out `# ms1 (entropy, BCH-checksummed)` | rc=1, out empty, err `WC` |
| R4 | `verify-bundle --allow-argv-secret --template bip84 --network signet --ms1 MS --mk1 K1 K2 --md1 D1 D2 D3 --slot @0.phrase=@env:E` (P) | rc=0, out `ms1_decode: ok decoded successfully` | rc=1, out empty, err `WC` |
| R5 | `seed-xor split --from phrase=@env:E --shares 2 --deterministic-from-master` (P) | rc=0, out `throw enter benefit planet indoor happy ` | rc=1, out empty, err `WC` |
| R6 | `seed-xor combine --allow-argv-secret --shares 2 --share phrase=@env:E --share phrase=X` (X) | rc=0, out `abandon abandon abandon abandon abandon ` | rc=1, out empty, err `error: seed-xor combine: share at position 0 has invalid BIP-39 checksum (not a parseable mnemonic in --language english): mnemonic has an invalid word count: 1. Word count must be 12, 15, 18, 21, or 24` |
| R7 | `ms-shares split --from phrase=@env:E --threshold 2 --shares 3` (P) | rc=0, out an `ms12...` share (random) | rc=1, out empty, err `error: ms-shares split: input phrase must be 12/15/18/21/24 words; got 1` |
| R8 | `ms-shares combine --allow-argv-secret --share @env:E --share M2` (M1) | rc=0, out `abandon abandon abandon abandon abandon ` | rc=1, out empty, err `error: ms1 not enough shares: have 1, need 2` |
| R9 | `slip39 split --from phrase=@env:E --group-threshold 1 --group 3,2` (P) | rc=0, out a SLIP-39 share (random) | rc=1, out empty, err `error: slip39 split: input phrase must be 12/15/18/21/24 words; got 1` |
| R10 | `slip39 combine --allow-argv-secret --share @env:E --share L2` (L1) | rc=0, out `00000000000000000000000000000000` | rc=1, out empty, err `error: slip39 combine: share at position 0: word at index 0 not in SLIP-39 wordlist` |
| S1 | `derive-child --from phrase=@env:E --passphrase-stdin --application hex --length 16 --index 0` (empty) | rc=1, err `error: --passphrase-stdin cannot be used with --from <node>=- (single stdin per invocation)` | rc=1, out empty, err `WC` |
| S2 | `convert --from phrase=@env:E --passphrase-stdin --to entropy` (empty) | rc=1, err `error: --passphrase-stdin cannot coexist with --from <node>=- (a single stdin cannot serve both); supply the value-bearing source via argv` | rc=1, out empty, err `WC` |
| S3 | `bundle --template bip84 --network signet --slot @0.phrase=@env:E --passphrase-stdin` (empty) | rc=1, err `error: --passphrase-stdin cannot be used with --slot @N.<secret>=- (single stdin per invocation)` | rc=1, out empty, err `WC` |
| S4 | `seed-xor combine --shares 2 --share phrase=@env:E --share phrase=@env:E` (X) | rc=1, err `error: seed-xor combine: at most one --share value may be \`-\` (single stdin per invocation)` | rc=1, out empty, err as R6 GREEN |
| S5 | `slip39 split --from phrase=@env:E --passphrase-stdin --group-threshold 1 --group 3,2` (empty) | rc=1, err `error: slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)` | rc=1, out empty, err `error: slip39 split: input phrase must be 12/15/18/21/24 words; got 1` |
| S6 | `slip39 combine --share @env:E --share @env:E` (L1) | rc=1, err `error: slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)` | rc=1, out empty, err `error: slip39 combine: share at position 0: word at index 0 not in SLIP-39 wordlist` |
| S7 | `ms-shares combine --share @env:E --share @env:E` (M1) | rc=1, err `error: ms-shares combine: at most one stdin consumer per invocation (across --share)` | rc=1, out empty, err `error: ms-shares combine: at least one --share required` |
| S8 | `bundle --template wsh-sortedmulti --threshold 1 --network signet --slot @0.phrase=- --slot @1.phrase=@env:E` (P) | rc=1, err `error: at most one --slot @N.<secret>=- per invocation (single stdin per invocation)` | rc=1, out empty, err `WC` |
| S9 | `verify-bundle --allow-argv-secret --template bip84 --network signet --ms1 MS --mk1 K1 K2 --md1 D1 D2 D3 --slot @0.phrase=@env:E --passphrase-stdin` (empty) | rc=1, err `error: --passphrase-stdin cannot be used with --slot @N.<secret>=- (single stdin per invocation)` | rc=1, out empty, err `WC` |
| A1 | `seed-xor split --from phrase=@env:G --shares 2 --deterministic-from-master` (none; `G=P`) | rc=0, `argv-advisory=1` (the line is `ADV`) | rc=0, `argv-advisory=0` |
| A2 | `ms-shares split --from phrase=@env:G --threshold 2 --shares 3` (none) | rc=0, `argv-advisory=1` | rc=0, `argv-advisory=0` |
| A3 | `slip39 split --from phrase=@env:G --group-threshold 1 --group 3,2` (none) | rc=0, `argv-advisory=1` | rc=0, `argv-advisory=0` |
| C1 | `convert --from phrase=- --to entropy` (P) | rc=0, out `entropy: 0000...` | same |
| C2 | `convert --from phrase=@env:G --to entropy` (none) | rc=0, out `entropy: 0000...` | same |
| C3 | `bundle --template bip84 --network signet --slot @0.phrase=-` (P) | rc=0, out `# ms1 (entropy, BCH-checksummed)` | same |
| C4 | `convert --from phrase=- --to entropy` (stdin `@env:G`) | rc=1, err `WC` | same |
| C5 | `bundle --template bip84 --network signet --slot @0.phrase=-` (stdin `@env:G`) | rc=1, err `WC` | same |
| C6 | `ms-shares split --allow-argv-secret --from "phrase=$P" --threshold 2 --shares 3` (none) | rc=0, `argv-advisory=1` | same |

The A rows print `argv-advisory=<count of stderr lines containing "secret material on argv">`. The capture build
does not fix R-3, so A1-A3 GREEN is the rule, not a measurement; inline `--from phrase=<P>` keeps its advisory
(`tests/cli_slip39_advisories.rs` row 1a).

Measured output of the script at `04c56bfa` (stderr warnings elided to the last line, as the script prints):

    R1-convert rc=0 out=entropy: 0000000000000000000000000000000 err=warning: stdout carries private key material ...
    R2-derive-child rc=0 out=866a12e42e31a09aa8ca4f25a02e999e err=warning: stdout carries private key material ...
    R3-bundle-slot rc=0 out=# ms1 (entropy, BCH-checksummed) err=warning: stdout carries private key material ...
    R4-verify-bundle-slot rc=0 out=ms1_decode: ok decoded successfully err=
    R5-seed-xor-split rc=0 out=throw enter benefit planet indoor happy  err=warning: Seed XOR shares on stdout ...
    R6-seed-xor-combine rc=0 out=abandon abandon abandon abandon abandon  err=warning: combined phrase is secret ...
    R7-ms-shares-split rc=0 out=ms12cd0lqnwufuhd4n0c52u6pqx0jeyyasxmq0rs err=note: each share is secret material ...
    R8-ms-shares-combine rc=0 out=abandon abandon abandon abandon abandon  err=note: verify the recovered wallet's ...
    R9-slip39-split rc=0 out=voter aluminum academic acid camera reun err=note: each share is secret material ...
    R10-slip39-combine rc=0 out=00000000000000000000000000000000 err=note: verify the recovered wallet's ...
    S1..S4 rc=1 with the single-stdin errors above
    A1-seed-xor-split-env-source rc=0 argv-advisory=1   (A2, A3 the same)
    C1..C3 rc=0; C4-stdin-text-not-env-resolved rc=1 out= err=WC

## 6. Tests

New, one test function per row, named exactly (TRACE), in `crates/mnemonic-toolkit/tests/cli_env_dash_literal.rs`
(runs `env!("CARGO_BIN_EXE_mnemonic")` / `assert_cmd`, sets the variable with `.env(...)`, never the process env):
`r1_convert_env_dash_is_literal`, `r2_derive_child_env_dash_is_literal`, `r3_bundle_slot_env_dash_is_literal`,
`r4_verify_bundle_slot_env_dash_is_literal`, `r5_seed_xor_split_env_dash_is_literal`,
`r6_seed_xor_combine_env_dash_is_literal`, `r7_ms_shares_split_env_dash_is_literal`,
`r8_ms_shares_combine_env_dash_is_literal`, `r9_slip39_split_env_dash_is_literal`,
`r10_slip39_combine_env_dash_is_literal`, `s1_derive_child_env_dash_not_a_stdin_reader`,
`s2_convert_env_dash_not_a_stdin_reader`, `s3_bundle_slot_env_dash_not_a_stdin_reader`,
`s4_seed_xor_combine_two_env_dashes_not_stdin_readers`, `s5_slip39_split_env_dash_not_a_stdin_reader`,
`s6_slip39_combine_two_env_dashes_not_stdin_readers`, `s7_ms_shares_combine_two_env_dashes_not_stdin_readers`,
`s8_bundle_literal_slot_and_env_dash_slot`, `s9_verify_bundle_slot_env_dash_not_a_stdin_reader`,
`a1_seed_xor_split_env_source_no_argv_advisory`,
`a2_ms_shares_split_env_source_no_argv_advisory`, `a3_slip39_split_env_source_no_argv_advisory`,
`c1_convert_literal_dash_reads_stdin`, `c2_convert_env_phrase_works`, `c3_bundle_slot_literal_dash_reads_stdin`,
`c4_stdin_text_is_not_env_resolved`, `c5_bundle_stdin_text_is_not_env_resolved`,
`c6_ms_shares_split_inline_keeps_argv_advisory`.
Each R and S test asserts the exit code, an empty stdout, and the GREEN stderr line of section 5 as a whole line of
stderr. R7 and R9 must not pin share text (random). A1-A3 assert rc=0 and that no stderr line contains
`secret material on argv`.

Existing tests that change: none found. Checked at `04c56bfa`: no test sets an environment variable to `-`
(`grep -rnE '\.env\("[A-Z_0-9]+", *"-"\)|set_var\([^)]*"-"\)' crates/mnemonic-toolkit/{tests,src}` prints nothing),
and no test pins `secret material on argv (--from` for an `@env:` source on a split (`tests/cli_slip39_advisories.rs`
rows 1a/1b use inline values, which keep the advisory). The full suite in the gate is the check.

Source files this FIX changes (for build ordering): `crates/mnemonic-toolkit/src/cmd/convert.rs`,
`cmd/derive_child.rs`, `cmd/bundle.rs`, `cmd/verify_bundle.rs`, `cmd/seed_xor.rs`, `cmd/ms_shares.rs`,
`cmd/slip39.rs`, `crates/mnemonic-toolkit/src/slot_input.rs` (if the builder changes the slot helpers),
`crates/mnemonic-toolkit/tests/cli_env_dash_literal.rs` (new), `design/fix-tk-env-dash-stdin-order-red.sh`,
`docs/manual/src/40-cli-reference/41-mnemonic.md`, `CHANGELOG.md`. Overlap with tk #15 (`fix/tk-stdout-write-errors`,
merged in `04c56bfa`'s history): none pending. Batch-3 tk rows #24 (coldcard multisig) and #25 (derive-child master
depth): #25 likely touches `cmd/derive_child.rs`; build after or rebase.

## 7. Gate (builder, one run to a file, then grep)

    export PATH=$HOME/.cargo/bin:$PATH; cargo --version   # 1.85.0
    cargo +1.95.0 fmt --all -- --check && cargo clippy --locked --all-targets -- -D warnings \
      && cargo nextest run --locked --workspace --build-jobs 6 -j 6 > gate.log 2>&1; grep -E '^\s+(PASS|FAIL)|Summary' gate.log | tail -5
    cargo build --locked --bin mnemonic && design/fix-tk-env-dash-stdin-order-red.sh target/debug/mnemonic
    make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk

Plus every other job of `.github/workflows/rust.yml`, `manual.yml`, `changelog-check.yml` locally or green on the PR
head (RULES.md build gate). Not covered by this gate: a terminal stdin (every row pipes stdin), Windows, and the
`mnemonic-gui` front end (it passes argv through; no GUI change).

## 8. FOLLOWUPS

Closes: none (bh-tk-I5 has no line in `design/FOLLOWUPS.md` at `04c56bfa`; grep for `bh-tk-I5`, `@env.*stdin`,
`env-dash`). Files, after the PR merges (direct-push; not in the PR):

- F1 `- [tk-verify-bundle-ms1-env-separator-only-watch-only] status=open grade=M kind=input owner=none filed=2026-11-22 src=design/FIX-tk-env-dash-stdin-order.md#8 :: verify-bundle --ms1 @env:VAR whose value is separator-only (e.g. "-") strips to "" and silently selects the watch-only cosigner sentinel; F-689 refuses this for --ms1 - stdin but not for @env:. Measured at 04c56bfa: notice "cosigner[0] marked watch-only via empty --ms1 sentinel".`
- F2 `- [tk-ms-shares-combine-separator-only-share-dropped] status=open grade=M kind=input owner=none filed=2026-11-22 src=design/FIX-tk-env-dash-stdin-order.md#3 :: ms-shares combine drops a --share value that is empty or separator-only (inline, @env: or stdin) without a word (cmd/ms_shares.rs:394-398); with enough other shares it recovers from those and the named source is ignored. Refuse a named-but-empty share instead.`
