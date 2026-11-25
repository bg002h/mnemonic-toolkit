# FIX-tk-unbounded-args: `derive-child` dice and `xpub-search` refuse or stream huge argument values

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24. Track: small (one Opus review round). Findings (bughunt-1119
regrade, graded I, refutes upheld): bh-tk-mn7, bh-tk-mn11, bh-tk-mn18
(coordinator `agent-reports/bughunt-1119-mn-regrade/tk-grade.md:25-60`, `tk-refute-bh-tk-mn7.md`,
`tk-refute-bh-tk-mn18.md`). Base: origin/master e30985a5. Closes the ruled FOLLOWUPS entries
`derive-child-dice-length-cap` and `xpub-search-account-window-2p31-panic` (their ruled texts are used verbatim) and
`xpub-search-max-account-unbounded-allocation`.

## 1. Defects

- **mn7.** `mnemonic derive-child --application dice --length N` checks only `N >= 1`
  (`cmd/derive_child.rs:299-307`); the help (`derive_child.rs:56`) documents `1..=2^31-1`. `bip85.rs:254` runs
  `Vec::with_capacity(rolls as usize)` of `String` (24 bytes each). `--length 2000000000` asks 48 GB and the process
  dies by SIGABRT (exit 134) with `memory allocation of 48000000000 bytes failed`, no refusal.
- **mn11.** `xpub-search path-of-xpub` / `passphrase-of-xpub` / `account-of-descriptor` take `--min-account`,
  `--number-of-accounts`, `--max-account` as `u32` with no check. Any account at or above 2^31 reaches
  `template.rs:77-78` (`.expect("template paths are well-formed by construction")`) or
  `cmd/xpub_search/descriptor_intake.rs:434` (`.expect("BIP-48 default path well-formed")`): a panic, exit 101,
  blaming the template, not the argument.
- **mn18.** `cmd/xpub_search/candidate_paths.rs:62` builds the whole candidate set up front:
  `Vec::with_capacity(n * 7 + add_paths.len())` of `CandidatePath` (56 bytes each). `--max-account 2147483647`, a
  valid hardened account, asks 841 813 590 016 bytes and aborts (exit 134). Any window of about 10^8 accounts or more
  does the same. The mn11 window refusal does not cover it (the window `0..=2^31-1` is valid).

## 2. Decisions

### 2.1 `--length` for `--application dice`: `1..=1000000`

The accepted range is `1..=1000000` rolls (the ruling in FOLLOWUPS `derive-child-dice-length-cap`). Why that value:

- BIP-85 (bitcoin/bips `bip-0085.mediawiki`, DICE, fetched 2026-11-24, lines 395-398) puts `{rolls}'` in the
  hardened derivation path `m/83696968'/89101'/{sides}'/{rolls}'/{index}'` and states `1 <= rolls <= 2^32 - 1`. So the
  BIP sets no useful size limit; the cap is the tool's.
- A dice secret needs few rolls: 256 bits is 100 d6 rolls; the BIP's own vector is 10 rolls. 10^6 is four orders of
  magnitude above any secret use.
- 10^6 rolls are cheap: measured on the base binary under `ulimit -v 4000000`, `--length 1000000` exits 0 and prints
  2 000 000 bytes at `--dice-sides 6` and 10 483 055 bytes at `--dice-sides 2147483647` (the largest accepted die).
  The roll vector is at most about 1e6 x 40 bytes = 40 MB.
- Reproducibility note (for the architect; not a STOP): a length in `1000001..=2147483647` was accepted before when
  memory allowed (R1 below shows 1 000 001 rolls at exit 0), and `{rolls}` is part of the derivation path, so such a
  sequence cannot be re-derived with a release carrying this fix. No plausible secret uses more than 10^6 rolls; the
  ruling stands.

### 2.2 Account window: refuse at 2^31 (mn11)

BIP-32 hardened indexes are `0..=2^31-1` (`i ≥ 2^31` is the hardened bit; the account component `account'` is the
31-bit value). The window `[min, end)` with `end = max(min + number, max + 1)` must stay at or below 2^31. Refused
(never clamped), with the ruled text, exit 1, when any of:

- `--min-account` ≥ 2147483648, or
- `--max-account` ≥ 2147483648, or
- `--min-account + --number-of-accounts` > 2147483648 (sum computed in `u64`).

Accepted edges: `--min-account 2147483646 --number-of-accounts 2` (accounts 2147483646, 2147483647);
`--max-account 2147483647`; `--min-account 2147483647 --number-of-accounts 1`; `--min-account 2147483648` is refused
even with `--number-of-accounts 0` (empty window).

### 2.3 Candidate set: stream, do not cap (mn18)

The candidate set becomes lazy: the order, contents and the printed count stay byte-identical, memory is constant in the
window size. A valid window stays usable; a holder who asks for the full hardened range gets a long search (it is
bounded only by time, which the holder chose), not an abort. No cap, no new advice text.

## 3. Holder-visible behaviour

### 3.1 Refusal texts (stderr, one line, nothing on stdout)

D1, `derive-child --application dice`, `--length` outside `1..=1000000` (includes `0`), exit 2 (unchanged code,
variant `DeriveChildLengthOutOfRange`). Ruled text (FOLLOWUPS `derive-child-dice-length-cap`):

~~~text
error: --length <N> out of range for --application dice (valid: 1..=1000000 rolls)
~~~

`<N>` is the value as given in decimal. This replaces `(valid: rolls >= 1)` for `--length 0`. The refusal fires before
`--dice-sides` is read (the present order: a missing `--dice-sides` with a bad `--length` still prints D1).

W1, `xpub-search path-of-xpub | passphrase-of-xpub | account-of-descriptor`, window rule of 2.2, exit 1 (variant
`BadInput`). Ruled text (FOLLOWUPS `xpub-search-account-window-2p31-panic`):

~~~text
error: --min-account / --max-account / --number-of-accounts: the account window must stay below 2147483648 (2^31): BIP-32 hardened account indexes are 31 bits
~~~

W1 is the first thing each of the three commands does: before the one-stdin check, before any stdin read, before
`passphrase-of-xpub`'s unconditional `note: passphrase verification searches …` advisory, before the argv-passphrase
warning. So stderr is exactly the W1 line. It applies also to `passphrase-of-xpub --passphrase-candidates-file` (the
check precedes the candidates-file dispatch at `passphrase_of_xpub.rs:297`).

### 3.2 Help texts (PROPOSED, architect confirms)

H1, `derive_child.rs:56`, the dice entry of `--application`'s list: `` `--length` rolls (1..=2^31-1) of a `` becomes
`` `--length` rolls (1..=1000000) of a ``. Rendered `app_entry("dice")`:

~~~text
dice BIP-85 application 89101' — derives `--length` rolls (1..=1000000) of a `--dice-sides`-sided die
~~~

H2, the `--max-account` doc comment, identical in `path_of_xpub.rs:94-95`, `account_of_descriptor.rs:106-107`,
`passphrase_of_xpub.rs:125-126`. Old: ``Optional upper bound. Effective end is `max(min_account + number_of_accounts,
max_account + 1)`.`` New (one sentence appended):

~~~text
Optional upper bound. Effective end is `max(min_account + number_of_accounts, max_account + 1)`. Every account in the window must be below 2147483648 (2^31).
~~~

(the doc-comment line breaks are the builder's; clap rewraps). `--min-account` and `--number-of-accounts` help are
unchanged.

### 3.3 Unchanged

Every accepted input prints byte-identical output: candidate order, `searched: N candidate paths`,
`searched: N candidates × C cosigners`, `searched: N candidate paths per passphrase`, `paths searched=N` in the
no-match error, every `--json` `searched_count`. BIP-85 dice KATs unchanged. Exit codes 0 / 4 unchanged.

## 4. Code delta (production)

1. `cmd/derive_child.rs:299-307`: the dice check becomes `!(1..=1_000_000).contains(&rolls)` with
   `valid_text: "1..=1000000 rolls"`. Name the bound `pub(crate) const DICE_MAX_ROLLS: u32 = 1_000_000;` in `bip85.rs`
   and use it in both places below.
2. `bip85.rs:254`: `Vec::with_capacity((rolls.min(1024)) as usize)` (defence in depth; the doc comment at :224 adds
   "the CLI caps rolls at `DICE_MAX_ROLLS`"). `format_dice_rolls` itself adds no refusal.
3. `cmd/xpub_search/candidate_paths.rs`:
   - new `pub fn check_account_window(min_account: u32, number_of_accounts: u32, max_account: Option<u32>) ->
     Result<(), ToolkitError>`: returns `BadInput` with the W1 text (without the `error: ` prefix, which Display adds)
     on the 2.2 conditions; `Ok(())` otherwise.
   - `build_candidate_paths` keeps its arguments and returns a new `pub struct CandidatePaths` (fields: `lo: u32`,
     `hi: u32`, `add_paths: Vec<String>`, `network: CliNetwork`; `hi` computed exactly as today, :52-60), not a `Vec`.
     It stays infallible; its doc states the precondition "the caller ran `check_account_window`".
   - `CandidatePaths::iter(&self) -> impl Iterator<Item = CandidatePath> + '_` yields exactly the sequence the old
     `Vec` held, in the same order (4 single-sig templates × accounts, then 3 BIP-48 script types × accounts, then each
     `--add-path` in order, malformed substitutions skipped as today). No allocation proportional to `hi - lo`.
   - `CandidatePaths::len(&self) -> usize` equals `self.iter().count()` without building candidates: `7 * (hi - lo)`
     plus, per add-path, `1`/`0` for a token-free path by whether it parses, and for a token path the number of
     accounts in `lo..hi` whose substituted string parses (a counting loop; parse validity can depend on the account,
     e.g. `m/1account'` → `m/12147483647'` overflows).
4. `path_search.rs:26-28` `match_xpub_against_paths(master, candidates: &CandidatePaths, target)` iterates
   `candidates.iter()`; `account_search.rs:27-30` `match_descriptor_against_seed(..., candidates: &CandidatePaths)`
   likewise (one fresh `iter()` per cosigner). `passphrase_search.rs:84` passes `&candidates` per line (one fresh
   `iter()` per passphrase).
5. `check_account_window(args.min_account, args.number_of_accounts, args.max_account)?;` as the first statement of
   `run_path_of_xpub` (`path_of_xpub.rs:228`, before `refuse_second_stdin`), `run_account_of_descriptor`
   (`account_of_descriptor.rs:238`), `run_passphrase_of_xpub` (`passphrase_of_xpub.rs:271`, before the
   `STDERR_ADVISORY` write).
6. Help: H1, H2.

`template.rs:77-78` and `descriptor_intake.rs:434` keep their `expect`s (now unreachable from the CLI).

## 5. RED rows

Run on the base (e30985a5, `target/debug/mnemonic`, `cargo build --locked`) by the author. Signet only: the master is
the public BIP-39 vector `abandon ×11 about` (on stdin); `X` = its signet `m/84'/1'/0'` tpub
`tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M`; `T`
= its signet master tprv from `mnemonic convert --from phrase=- --to xprv --path m --network signet`. Every abort or
panic row runs as `timeout 60 sh -c 'ulimit -v 4000000; exec "$@"' sh mnemonic …` (4 GB address space, 60 s), so a
pre-fix run fails fast on any host and never touches swap. Integration rows build the same bound in-test (spawn
`sh -c 'ulimit -v 4000000 && exec "$0" "$@"'`, the pattern of `bughunt_s4.rs:59-80`, with a deadline).

New file `crates/mnemonic-toolkit/tests/bughunt_1119_unbounded_args.rs` unless noted. Row names are the test names.

| Row | Command (stdin) | Base output (observed) | Expected after fix |
|---|---|---|---|
| R1 `bh_tk_mn7_dice_length_over_cap_is_refused` | `derive-child --from xprv=- --application dice --dice-sides 6 --length 1000001 --index 0` (`T`) | exit 0; stdout 2 000 002 bytes (1 000 001 rolls) | exit 2; stdout empty; stderr exactly `error: --length 1000001 out of range for --application dice (valid: 1..=1000000 rolls)` |
| R2 `bh_tk_mn7_dice_length_zero_names_the_cap` | same, `--length 0` | exit 2; `error: --length 0 out of range for --application dice (valid: rolls >= 1)` | exit 2; `error: --length 0 out of range for --application dice (valid: 1..=1000000 rolls)` |
| R3 `bughunt_tk_15_dice_huge_length_is_refused_cleanly` (existing, `bughunt_s4.rs:59`, un-ignore) | same, `--length 2000000000`, ulimit | `memory allocation of 48000000000 bytes failed`; exit 134 (SIGABRT) | exit 2; stderr exactly D1 with `2000000000` (assertion added) |
| G1 `bh_tk_mn7_dice_length_at_cap_succeeds` (guard, green on base) | same, `--length 1000000` | exit 0; 1 000 000 comma-separated rolls | unchanged |
| R4 `bughunt_tk_32_path_of_xpub_min_account_2p31_no_panic` (existing, `bughunt_s4.rs:306`, un-ignore) | `xpub-search path-of-xpub --phrase-stdin --network signet --target-xpub X --min-account 2147483648` | panic `template.rs:78:14 … InvalidChildNumber(2147483648)`; exit 101 | exit 1; stderr exactly W1 (assertion added) |
| R5 `bh_tk_mn11_path_of_xpub_window_crossing_2p31_is_refused` | same, `--min-account 2147483647 --number-of-accounts 2` | same panic; exit 101 | exit 1; W1 |
| R6 `bh_tk_mn11_path_of_xpub_max_account_2p31_is_refused` | same, `--max-account 2147483648`, ulimit | `memory allocation of 841813590408 bytes failed`; exit 134 | exit 1; W1 |
| R7 `bughunt_tk_32_account_of_descriptor_zero_window_2p31_no_panic` (existing, `bughunt_s4.rs:326`, un-ignore) | `xpub-search account-of-descriptor --phrase-stdin --network signet --descriptor 'wsh(sortedmulti(1,X/<0;1>/*,X/<0;1>/*))' --min-account 2147483648 --number-of-accounts 0` | panic `descriptor_intake.rs:434:34 BIP-48 default path well-formed: InvalidChildNumber(2147483648)`; exit 101 | exit 1; W1 (assertion added) |
| R8 `bh_tk_mn11_passphrase_of_xpub_min_account_2p31_is_refused` | `xpub-search passphrase-of-xpub --phrase-stdin --network signet --passphrase "" --target-xpub X --min-account 2147483648` | advisory note, argv warning, then panic `template.rs:78:14`; exit 101 | exit 1; stderr exactly the W1 line (no note, no warning) |
| G2 `bh_tk_mn11_window_ending_at_2p31_is_accepted` (guard) | path-of-xpub, `--min-account 2147483646 --number-of-accounts 2` | exit 4; `error: no match in searched set: mode=path-of-xpub, paths searched=14; widen the range …` | unchanged |
| G3 `bh_tk_mn18_add_path_count_unchanged` (guard) | path-of-xpub, `--number-of-accounts 3 --add-path "m/1account'" --add-path "m/9'/account"` | exit 0; `match: m/84'/1'/0'  (template=bip84, account=0)`; `searched: 27 candidate paths` | unchanged |
| R9 `bh_tk_mn18_path_of_xpub_full_hardened_window_streams` | path-of-xpub, `--max-account 2147483647`, ulimit | `memory allocation of 841813590016 bytes failed`; exit 134 | still running after 10 s (the test kills it); stderr has no `memory allocation` |
| R10 `bh_tk_mn18_account_of_descriptor_full_hardened_window_streams` | account-of-descriptor (R7's descriptor), `--max-account 2147483647`, ulimit | `memory allocation of 841813590016 bytes failed`; exit 134 | as R9 |
| R11 `bh_tk_mn18_passphrase_of_xpub_full_hardened_window_streams` | passphrase-of-xpub, `--passphrase "" --max-account 2147483647`, ulimit | note, warning, `memory allocation of 841813590016 bytes failed`; exit 134 | as R9 (the note and warning stay) |

R9-R11: the test spawns under the ulimit, polls `try_wait` every 100 ms, asserts the child has not exited at 10 s, kills
it, and asserts stderr lacks `memory allocation`. On the base the child exits 134 within a second.

Unit rows (in `cmd/xpub_search/candidate_paths.rs` `mod tests`; RED on base = they do not compile, as
`CandidatePaths` / `check_account_window` do not exist):

- U1 `candidate_paths_len_matches_iter_count`: for `(0,20,None,[])`, `(5,3,None,[])`, `(0,5,Some(50),[])`,
  `(0,3,None,["m/1account'","m/9'/account","m/86'/0'/0'","garbage"])`, `(2147483646,2,None,["m/1account'"])`:
  `len() == iter().count()`; the last is 14.
- U2 `candidate_paths_full_hardened_window_is_lazy`: `build_candidate_paths(0,20,Some(2147483647),&[],Mainnet)`:
  `len() == 15032385536` (7 × 2^31) and the first three items are `bip44` accounts 0, 1, 2 (`44'/0'/0'` …).
- U3 `check_account_window_boundaries`: Ok for `(0,20,None)`, `(2147483646,2,None)`, `(2147483647,1,None)`,
  `(0,0,Some(2147483647))`, `(0,u32::MAX>>1,None)` (= 2^31-1 accounts from 0); Err with the W1 message for
  `(2147483648,0,None)`, `(2147483647,2,None)`, `(0,20,Some(2147483648))`, `(1,2147483648,None)`, `(0,u32::MAX,None)`.

## 6. Tests that change

- `tests/bughunt_s4.rs`: `bughunt_tk_15_dice_huge_length_is_refused_cleanly`,
  `bughunt_tk_32_path_of_xpub_min_account_2p31_no_panic`, `bughunt_tk_32_account_of_descriptor_zero_window_2p31_no_panic`:
  drop `#[ignore]`, add the exit-code and exact-stderr assertions of R3, R4, R7. Their comments drop "OPEN product
  question".
- `tests/cli_printed_text_drift.rs`: `d3_derive_child_help_dice_is_runnable_with_bip85_range` (:336) expects H1;
  `d6_derive_child_dice_bounds_match_binary` (:359): `dice("6", "2147483648")` becomes `dice("6", "1000001")` (rolls
  over the cap, nonzero) plus `assert_eq!(dice("6", "1000000"), 0, "rolls at cap")`, and
  `h.find("rolls (1..=2^31-1)")` becomes `h.find("rolls (1..=1000000)")`; `b3_manual_bip85_application_facts` (:500)
  expects ``| `dice` | deterministic dice rolls | `1..=1000000` |``.
- `cmd/xpub_search/candidate_paths.rs` unit tests (:163-243, 7 tests): index `candidates[i]` / `.iter().filter` over
  `candidates.iter()` (collect to a `Vec` in the test where indexing is needed); expected values unchanged.
- No other test changes (`git grep build_candidate_paths` / `match_xpub_against_paths` /
  `match_descriptor_against_seed`: only the files in section 4; `tests/cli_xpub_search_passphrase_of_xpub.rs:6`
  names `match_xpub_against_paths` in a comment only).

## 7. Manual delta (toolkit manual, lockstep; PROPOSED, architect confirms)

- `docs/manual/src/40-cli-reference/41-mnemonic.md:1965` `--length` row. Old cell
  ``application-specific size; pass `0` for `hd-seed` and `xprv` `` → new cell
  ``application-specific size; pass `0` for `hd-seed` and `xprv`; `dice`: `1..=1000000` rolls (exit 2 otherwise)``.
- `41-mnemonic.md:4135` (path-of-xpub) and `:4494` (passphrase-of-xpub) `--max-account` rows. Old cell
  ``optional upper bound; effective end is `max(min_account + number_of_accounts, max_account + 1)` `` → new cell
  ``optional upper bound; effective end is `max(min_account + number_of_accounts, max_account + 1)`; every account in the window must be below `2^31` (exit 1 otherwise)``.
  account-of-descriptor's row (:4276) says "same as `path-of-xpub`" and is unchanged.
- `docs/manual/src/30-workflows/38-bip85-children.md:69`:
  ``| `dice` | deterministic dice rolls | `1..=2^31-1` |`` → ``| `dice` | deterministic dice rolls | `1..=1000000` |``.
- Not changed: `docs/manual-gui` (pinned to the GUI's release CLI, precedent FIX_bughunt_tk_g2 5.6; its
  `46-derive-child.md:128` dice paragraph and the `--max-account` sections of `4d`/`4e`/`4f` get the texts at the
  next GUI pin; filed as a FOLLOWUPS line in the author report). No flag added or removed, so the GUI schema mirror and
  `gui-schema` output are unchanged (it carries no help text).

## 8. CHANGELOG (`## [Unreleased]`; PROPOSED, architect confirms)

~~~text
### Huge `--length` and account-window values are refused or streamed, not aborted

- **`derive-child --application dice`**: `--length` is capped at 1000000 rolls. A larger value is refused (exit 2,
  `--length <N> out of range for --application dice (valid: 1..=1000000 rolls)`). Before, a value near 2^31 aborted
  on a failed memory allocation (exit 134), and lengths up to 2^31-1 were documented. (bh-tk-mn7)
- **`xpub-search path-of-xpub`, `passphrase-of-xpub`, `account-of-descriptor`**: an account window reaching 2^31
  (`--min-account` or `--max-account` ≥ 2147483648, or `--min-account + --number-of-accounts` > 2147483648) is refused
  (exit 1). Before, it panicked (exit 101). (bh-tk-mn11)
- **`xpub-search`**: candidate paths are generated as the search runs, so a large valid window (up to
  `--max-account 2147483647`) searches instead of aborting on a memory allocation (exit 134). Printed counts and
  results are unchanged. (bh-tk-mn18)
~~~

## 9. Gate

`bash /scratch/code/shibboleth/refugium-coordinator/coordinator/scripts/build-gate.sh <builder worktree> origin/master --rows 'test(/bh_tk_mn(7|11|18)_|bughunt_tk_(15|32)_|candidate_paths_len_matches_iter_count|candidate_paths_full_hardened_window_is_lazy|check_account_window_boundaries/)' --rows-expect 17`

17 = R1-R11 (11, of which R3, R4, R7 are the three un-ignored `bughunt_s4.rs` rows) + G1-G3 (3) + U1-U3 (3). The
regex is unanchored because nextest matches unit tests by their module path. On the base no other test matches it
(checked: only the three `bughunt_s4.rs` rows, ignored). The drift tests d3/d6/b3 run in the full nextest step.

It runs the `[mnemonic-toolkit]` conf (fmt, clippy `-D warnings`, `cargo nextest run --locked --workspace`, doc
tests) and the mutation gate on the diff. Not covered by it: the `manual` CI job (`make -C docs/manual audit`, with
`verify-examples` and the doc-flag-lint mutation self-test) and `manual-gui`; run both, locally or as green CI on the
head. R9-R11 each hold a 10 s wait; they run in parallel under nextest.

## 10. FOLLOWUPS

None appended in the PR. Closures and the manual-gui line are listed in the author report
(`coordinator/agent-reports/fix-doc-bh-tk-unbounded-args.md`) for a direct push after merge.
