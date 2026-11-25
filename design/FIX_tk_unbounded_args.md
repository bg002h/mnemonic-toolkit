# FIX-tk-unbounded-args: `derive-child` dice and `xpub-search` refuse or stream huge argument values

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24. Track: small (one Opus review round). Findings (bughunt-1119
regrade, graded I, refutes upheld): bh-tk-mn7, bh-tk-mn11, bh-tk-mn18
(coordinator `agent-reports/bughunt-1119-mn-regrade/tk-grade.md:25-60`, `tk-refute-bh-tk-mn7.md`,
`tk-refute-bh-tk-mn18.md`). Base: origin/master e30985a5. Closes the ruled FOLLOWUPS entries
`derive-child-dice-length-cap` and `xpub-search-account-window-2p31-panic` (`xpub-search-account-window-2p31-panic`'s ruled text
is used verbatim; `derive-child-dice-length-cap`'s cap is superseded by the 2026-11-24 architect ruling, 2.1) and
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

### 2.1 `--length` for `--application dice`: `1..=2^31-1`, rolls written as drawn

Architect ruling 2026-11-24 (`coordinator/agent-reports/architect-tk-dice-cap-1124.md`), replacing the cap
`1..=1000000` of FOLLOWUPS `derive-child-dice-length-cap`. The accepted range is `1..=2147483647`, the range the help
(`derive_child.rs:56`) and manual ch.38 (`38-bip85-children.md:69`) already state.

- BIP-85 (bitcoin/bips `bip-0085.mediawiki`, DICE, fetched 2026-11-24, lines 395 and 398) puts `{rolls}'` in the
  hardened path `m/83696968'/89101'/{sides}'/{rolls}'/{index}'` and allows `1 <= rolls <= 2^32 - 1`; the hardened
  element ends derivation at 2^31-1. The rolls count is part of the child: a cap below 2^31-1 would make a child that
  every release since v0.8.0 derived (they check only `rolls >= 1`) underivable here, and a shorter `--length` derives
  an unrelated sequence, not a prefix.
- The defect is memory, not the count: `bip85.rs:254` reserves `rolls` × 24 bytes before the first roll. The rolls are
  now written to stdout as they are drawn, so memory is constant in `--length`; output size and time are the holder's
  choice, as for the candidate set (2.3). Every accepted length prints the same bytes as before.
- Refused, exit 2 (D1): `--length 0`, and `--length 2147483648` or more (before: a BIP-32 error, exit 1).
- A closed stdout (`| head`) does not end the run early: under T6 (`out_sink.rs:1-7`) later writes are dropped and the
  verb runs to its end, as every verb does.

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

D1, `derive-child --application dice`, `--length` outside `1..=2147483647` (`0`, or `2147483648` and above), exit 2
(unchanged code, variant `DeriveChildLengthOutOfRange`):

~~~text
error: --length <N> out of range for --application dice (valid: 1..=2^31-1 rolls)
~~~

`<N>` is the value as given in decimal. This replaces `(valid: rolls >= 1)` for `--length 0` and the BIP-32 child-number
error (exit 1) for `2147483648` and above. The refusal fires before `--dice-sides` is read (the present order).

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
`derive-child --application dice` stdout and stderr are byte-identical for every accepted length (R1).

## 4. Code delta (production)

1. `cmd/derive_child.rs:299-307`: the dice check becomes `!(1..=bip85::DICE_MAX_ROLLS).contains(&rolls)` with
   `valid_text: "1..=2^31-1 rolls"`; `pub(crate) const DICE_MAX_ROLLS: u32 = 2_147_483_647;` in `bip85.rs`.
2. `bip85.rs:214-280`: `format_dice_rolls` is removed and replaced by `pub(crate) fn write_dice_rolls<W: std::io::Write +
   ?Sized>(master: &Xpriv, sides: u32, rolls: u32, index: u32, out: &mut W) -> Result<(), ToolkitError>`: the same two
   checks, `derive_entropy`, the same SHAKE256 loop; each accepted roll is written at once (`,` before every roll but the
   first, then the decimal value with `write!`), a write error is `ToolkitError::Io`; no `Vec`, no `String`, no joined
   output; the per-roll read buffer is a stack `[u8; 4]` sliced to `bytes_per_roll`. Nothing is written before the checks
   and `derive_entropy` succeed. Its doc comment states the bound `rolls <= DICE_MAX_ROLLS` (the CLI's) and drops
   "`1 <= rolls <= 2^32 - 1`". The dice arm of `derive_child.rs:298-313` calls it with `stdout`, then `writeln!(stdout)`,
   then the same `emit_output_class_advisory(OutputClass::PrivateKeyMaterial, stderr)`, and returns `Ok(())`.
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
| R1 `bh_tk_mn7_dice_long_child_streams_byte_identical` | `derive-child --from xprv=- --application dice --dice-sides 6 --length 10000000 --index 0`, `ulimit -v 200000`, deadline 60 s | rc 134; stderr `memory allocation of 240000000 bytes failed`; stdout empty | exit 0; stdout 20000000 bytes, sha256 `2d14b9d9f52ab6379a461cd7d41c6bc01a36d69c4aa1c8727d1b9d39d3a62c59` (= base's stdout for the same command with no ulimit, measured); stderr exactly the one advisory line `warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '\| age -e ...')`. 0.43 s |
| R2 `bh_tk_mn7_dice_length_outside_range_is_refused` | same, no ulimit, `--length 0`; then `--length 4294967295` | 0: rc 2, `error: --length 0 out of range for --application dice (valid: rolls >= 1)`; 4294967295: rc 1, `error: BIP-32 error: child number 4294967295 is invalid (not within [0, 2^31 - 1])` | each: exit 2; stdout empty; stderr exactly `error: --length <N> out of range for --application dice (valid: 1..=2^31-1 rolls)` with N = 0, 4294967295 |
| R3 `bughunt_tk_15_dice_huge_length_is_refused_cleanly` (existing, `bughunt_s4.rs:59`, un-ignore) | keeps its argv form and ulimit 4000000; `--length 2000000000` becomes `--length 2147483648` | rc 1; stderr `warning: secret material on argv (--from xprv=) — pipe via --from xprv=- to avoid /proc/$PID/cmdline exposure` then `error: BIP-32 error: child number 2147483648 is invalid (not within [0, 2^31 - 1])` | exit 2; stdout empty; stderr exactly the same warning line then `error: --length 2147483648 out of range for --application dice (valid: 1..=2^31-1 rolls)` |
| R12 `bh_tk_mn7_dice_max_length_streams_at_once` | `--length 2147483647`, `ulimit -v 200000`; a thread reads stdout, the test waits `recv_timeout(60 s)` for the first 63 bytes, then kills and waits the child | rc 134; `memory allocation of 51539607528 bytes failed`; stdout empty | the first 63 stdout bytes are exactly `0,5,0,4,0,2,0,0,2,5,5,1,1,4,0,4,0,0,4,5,0,1,3,3,1,4,3,0,4,4,3,5` (= the oracle's first 32 rolls of `m/83696968'/89101'/6'/2147483647'/0'`). 0.004 s |
| R4 `bughunt_tk_32_path_of_xpub_min_account_2p31_no_panic` (existing, `bughunt_s4.rs:306`, un-ignore) | `xpub-search path-of-xpub --phrase-stdin --network signet --target-xpub X --min-account 2147483648` | panic `template.rs:78:14 … InvalidChildNumber(2147483648)`; exit 101 | exit 1; stderr exactly W1 (assertion added) |
| R5 `bh_tk_mn11_path_of_xpub_window_crossing_2p31_is_refused` | same, `--min-account 2147483647 --number-of-accounts 2` | same panic; exit 101 | exit 1; W1 |
| R6 `bh_tk_mn11_path_of_xpub_max_account_2p31_is_refused` | same, `--max-account 2147483648`, ulimit | `memory allocation of 841813590408 bytes failed`; exit 134 | exit 1; W1 |
| R7 `bughunt_tk_32_account_of_descriptor_zero_window_2p31_no_panic` (existing, `bughunt_s4.rs:326`, un-ignore) | `xpub-search account-of-descriptor --phrase-stdin --network signet --descriptor 'wsh(sortedmulti(1,X/<0;1>/*,X/<0;1>/*))' --min-account 2147483648 --number-of-accounts 0` | panic `descriptor_intake.rs:434:34 BIP-48 default path well-formed: InvalidChildNumber(2147483648)`; exit 101 | exit 1; W1 (assertion added) |
| R8 `bh_tk_mn11_passphrase_of_xpub_min_account_2p31_is_refused` | `xpub-search passphrase-of-xpub --phrase-stdin --network signet --passphrase "" --target-xpub X --min-account 2147483648` | advisory note, argv warning, then panic `template.rs:78:14`; exit 101 | exit 1; stderr exactly the W1 line (no note, no warning) |
| G2 `bh_tk_mn11_window_ending_at_2p31_is_accepted` (guard) | path-of-xpub, `--min-account 2147483646 --number-of-accounts 2` | exit 4; `error: no match in searched set: mode=path-of-xpub, paths searched=14; widen the range …` | unchanged |
| G3 `bh_tk_mn18_add_path_count_unchanged` (guard) | path-of-xpub, `--number-of-accounts 3 --add-path "m/1account'" --add-path "m/9'/account"` | exit 0; `match: m/84'/1'/0'  (template=bip84, account=0)`; `searched: 27 candidate paths` | unchanged |
| O1 `bh_tk_mn18_order_bip48_before_add_path` (guard, green on base) | `xpub-search path-of-xpub --phrase-stdin --network signet --target-xpub tpubDEYM1BmQ5rp2PWKvCgvQxNeUrEv8gu5819xRdmu6S23fYpS8x2icwAeoVaBTLyN3fGWJQcWoaiKMduTXWKtG9bXNpVrZPRF7XVxrANtAEcR --number-of-accounts 2 --add-path "m/48'/1'/account'/2'"` (target = `m/48'/1'/1'/2'`; phrase `abandon ×11 about` on stdin) | exit 0; stdout exactly `match: m/48'/1'/1'/2'  (template=bip48-wsh, account=1)` / `target-xpub: tpubDEYM1BmQ5rp2PWKvCgvQxNeUrEv8gu5819xRdmu6S23fYpS8x2icwAeoVaBTLyN3fGWJQcWoaiKMduTXWKtG9bXNpVrZPRF7XVxrANtAEcR` / `searched: 16 candidate paths` | unchanged |
| O3 `bh_tk_mn18_order_add_paths_in_argv_order` (guard, green on base) | same, `--target-xpub tpubDCChhoz5QdrkoaTfhN9wz9UwMkRvBNezCAcbLJ3yqswVCnGtCvuBtmub29oFk2CEty4eu15WDGa7aJ3QcHKp9JtggPakJ8JaFz4HHtCnA65 --number-of-accounts 2 --add-path "m/87'/1'/1'" --add-path "m/87'/1'/account'"` (target = `m/87'/1'/1'`) | exit 0; stdout exactly `match: m/87'/1'/1'  (template=m/87'/1'/1', account=n/a)` / `target-xpub: tpubDCChhoz5QdrkoaTfhN9wz9UwMkRvBNezCAcbLJ3yqswVCnGtCvuBtmub29oFk2CEty4eu15WDGa7aJ3QcHKp9JtggPakJ8JaFz4HHtCnA65` / `searched: 17 candidate paths` (the reverse argv order gives `template=m/87'/1'/account', account=1`) | unchanged |
| O5 `bh_tk_mn18_order_templates_outer_accounts_inner` (guard, green on base) | same target as O3, `--number-of-accounts 2 --add-path "m/87'/1'/account'" --add-path "m/87'/1'/account1'"` | exit 0; stdout exactly `match: m/87'/1'/1'  (template=m/87'/1'/account', account=1)` / `target-xpub: tpubDCChhoz5QdrkoaTfhN9wz9UwMkRvBNezCAcbLJ3yqswVCnGtCvuBtmub29oFk2CEty4eu15WDGa7aJ3QcHKp9JtggPakJ8JaFz4HHtCnA65` / `searched: 18 candidate paths` (an account-major iterator prints `template=m/87'/1'/account1', account=0`) | unchanged |
| R9 `bh_tk_mn18_path_of_xpub_full_hardened_window_streams` | path-of-xpub, `--target-xpub X44`, `--max-account 2147483647`, ulimit, 60 s deadline | rc 134; `memory allocation of 841813590016 bytes failed` | rc 0 within the deadline; stdout as in the paragraph below (`searched: 15032385536 candidate paths`); also with `--json`: `searched_count` = 15032385536 |
| R10 `bh_tk_mn18_account_of_descriptor_full_hardened_window_streams` | account-of-descriptor, descriptor `wsh(sortedmulti(1,X44/<0;1>/*,X44/<0;1>/*))`, `--max-account 2147483647`, ulimit, 60 s deadline | rc 134; `memory allocation of 841813590016 bytes failed` | rc 0 within the deadline; stdout as in the paragraph below (`searched: 15032385536 candidates × 2 cosigners`) |
| R11 `bh_tk_mn18_passphrase_of_xpub_full_hardened_window_streams` | passphrase-of-xpub, `--passphrase ""`, `--target-xpub X44`, `--max-account 2147483647`, ulimit, 60 s deadline | rc 134 after the note and warning; `memory allocation of 841813590016 bytes failed` | rc 0 within the deadline; stdout as in the paragraph below (`searched: 15032385536 candidate paths`) |

R9/R10/R11: as above but `--target-xpub X44` (R10: the descriptor `wsh(sortedmulti(1,X44/<0;1>/*,X44/<0;1>/*))`),
`--max-account 2147483647`, ulimit, a 60 s deadline. `X44` = `tpubDC5FSnBiZDMmhiuCmWAYsLwgLYrrT9rAqvTySfuCCrgsWz8wxMXUS9Tb9iVMvcRbvFcAHGkMD5Kx8koh4GquNGNTfohfk7pgjhaPCdXpoba` (signet `m/44'/1'/0'` of `abandon ×11 about`, from
`mnemonic convert --from phrase=- --to xpub --path "m/44'/1'/0'" --network signet`). Base: rc 134, `memory allocation of 841813590016 bytes failed`.
After: rc 0 within the deadline; stdout exactly the 0..20 output (no `--max-account`; observed on base: `match: m/44'/1'/0'  (template=bip44, account=0)`,
`target-xpub: X44`, `searched: 140 candidate paths`; for account-of-descriptor `match: cosigner @0  m/44'/1'/0'  (template=bip44, account=0)`,
`match: cosigner @1  m/44'/1'/0'  (template=bip44, account=0)`, `cosigners total: 2; matched: 2`,
`searched: 140 candidates × 2 cosigners`) with `140` replaced by `15032385536`
(R9, R11: `searched: 15032385536 candidate paths`; R10: `searched: 15032385536 candidates × 2 cosigners`); R9 also
run with `--json`: `searched_count` = 15032385536.

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

- `tests/bughunt_s4.rs`: `bughunt_tk_32_path_of_xpub_min_account_2p31_no_panic`,
  `bughunt_tk_32_account_of_descriptor_zero_window_2p31_no_panic`: drop `#[ignore]`, add the exit-code and exact-stderr
  assertions of R4, R7. Their comments drop "OPEN product question".
- `tests/bughunt_s4.rs` `bughunt_tk_15_…`: un-ignore, `--length 2147483648`, the R3 assertions; comment drops "OPEN product question".
  R3 keeps its argv form (`--allow-argv-secret derive-child --from xprv=<MASTER_XPRV>`); after the fix its stderr is exactly two
  lines: `warning: secret material on argv (--from xprv=) — pipe via --from xprv=- to avoid /proc/$PID/cmdline exposure` then
  `error: --length 2147483648 out of range for --application dice (valid: 1..=2^31-1 rolls)`; assert exit code 2 and that whole stderr.
  This also closes FOLLOWUPS `dice-length-red-row-ulimit-false-pass`.
- `src/bip85.rs` tests: `t1_format_fns_return_secret_string` drops `_f7` (doc "7 `format_*`" → "6"); `dice_d6_10_rolls_matches_spec`,
  `dice_d2_rolls_in_range`, `dice_d256_rolls_in_range`, `dice_sides_too_small_refused` call `write_dice_rolls` into a
  `Vec<u8>` (expected values unchanged); `internal_encode_dice_scratch_is_zeroizing`: its dice assertion and the dice half
  of its doc comment are replaced by

  ~~~rust
          // dice: no per-roll aggregate; the rolls are written as they are drawn.
          for anchor in [format!("{}.push(trial", "out"), format!("{}.join(", "out")] {
              assert!(
                  !src.contains(&anchor),
                  "dice: rolls must be written as drawn; found `{anchor}`"
              );
          }
  ~~~

  Run: on fixB PASS (`/scratch/cache/arch-tkdice-1124-zt-green.log`); on base code FAIL `dice: rolls must be written as
  drawn; found `out.push(trial`` (`…-zt-red.log`).
- `tests/cli_derive_child.rs:722` `dice_d6_10_rolls_reference_vector`: unchanged (passes on fixB).
- `tests/cli_printed_text_drift.rs`: d3, d6 and b3 stay as they are (they pass on fixB; d6's `dice("6", "2147483648")` is nonzero as exit 2).
- `cmd/xpub_search/candidate_paths.rs` unit tests (:163-243, 7 tests): index `candidates[i]` / `.iter().filter` over
  `candidates.iter()` (collect to a `Vec` in the test where indexing is needed); expected values unchanged.
- No other test changes (`git grep build_candidate_paths` / `match_xpub_against_paths` /
  `match_descriptor_against_seed`: only the files in section 4; `tests/cli_xpub_search_passphrase_of_xpub.rs:6`
  names `match_xpub_against_paths` in a comment only).

## 7. Manual delta (toolkit manual, lockstep; PROPOSED, architect confirms)

- `41-mnemonic.md:4135` (path-of-xpub) and `:4494` (passphrase-of-xpub) `--max-account` rows. Old cell
  ``optional upper bound; effective end is `max(min_account + number_of_accounts, max_account + 1)` `` → new cell
  ``optional upper bound; effective end is `max(min_account + number_of_accounts, max_account + 1)`; every account in the window must be below `2^31` (exit 1 otherwise)``.
  account-of-descriptor's row (:4276) says "same as `path-of-xpub`" and is unchanged.
- Not changed: `docs/manual-gui` (pinned to the GUI's release CLI, precedent FIX_bughunt_tk_g2 5.6; its
  `--max-account` sections of `4d`/`4e`/`4f` get the texts at the
  next GUI pin; filed as a FOLLOWUPS line in the author report). No flag added or removed, so the GUI schema mirror and
  `gui-schema` output are unchanged (it carries no help text).

## 8. CHANGELOG (`## [Unreleased]`; PROPOSED, architect confirms)

~~~text
### Huge `--length` and account-window values are refused or streamed, not aborted

- **`derive-child --application dice`**: the rolls are written as they are drawn, so every `--length` in
  `1..=2^31-1` runs in constant memory and prints the same rolls as before. Before, the whole sequence was built in
  memory first, and a large `--length` aborted on a failed memory allocation (exit 134), so a long dice child could not
  be re-derived on a computer with less memory. `--length 0` and `--length` 2147483648 or more are refused
  (exit 2, `--length <N> out of range for --application dice (valid: 1..=2^31-1 rolls)`); 2147483648 and more failed
  with a BIP-32 error (exit 1) before. (bh-tk-mn7)
- **`xpub-search path-of-xpub`, `passphrase-of-xpub`, `account-of-descriptor`**: an account window reaching 2^31
  (`--min-account` or `--max-account` ≥ 2147483648, or `--min-account + --number-of-accounts` > 2147483648) is refused
  (exit 1). Before, it panicked (exit 101). (bh-tk-mn11)
- **`xpub-search`**: candidate paths are generated as the search runs, so a large valid window (up to
  `--max-account 2147483647`) searches instead of aborting on a memory allocation (exit 134). Printed counts and
  results are unchanged. (bh-tk-mn18)
~~~

## 9. Gate

`bash /scratch/code/shibboleth/refugium-coordinator/coordinator/scripts/build-gate.sh <builder worktree> origin/master --rows 'test(/bh_tk_mn(7|11|18)_|bughunt_tk_(15|32)_|candidate_paths_len_matches_iter_count|candidate_paths_full_hardened_window_is_lazy|check_account_window_boundaries/)' --rows-expect 20`

20 = R1-R12 (12, of which R3, R4, R7 are the three un-ignored `bughunt_s4.rs` rows) + G2-G3 and O1/O3/O5 (5) + U1-U3 (3). The
regex is unanchored because nextest matches unit tests by their module path. On the base no other test matches it
(checked: only the three `bughunt_s4.rs` rows, ignored). The drift tests d3/d6/b3 run in the full nextest step.

It runs the `[mnemonic-toolkit]` conf (fmt, clippy `-D warnings`, `cargo nextest run --locked --workspace`, doc
tests) and the mutation gate on the diff. Not covered by it: the `manual` CI job (`make -C docs/manual audit`, with
`verify-examples` and the doc-flag-lint mutation self-test) and `manual-gui`; run both, locally or as green CI on the
head.

## 10. FOLLOWUPS

None appended in the PR. Closures and the manual-gui line are listed in the author report
(`coordinator/agent-reports/fix-doc-bh-tk-unbounded-args.md`) for a direct push after merge.
