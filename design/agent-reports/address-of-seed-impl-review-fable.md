# address-of-seed: final implementation review (fable, software-developer lens)

- Scope: worktree `/home/user/wt-address-search`, branch `address-of-seed`,
  `40b35942..021dfe73`, against `design/SPEC_xpub_search_address_of_seed.md` r3
  (approved 2026-11-26, §10 recommendations 1-9) and the r1 review
  (`address-of-seed-impl-review-r1.md`, folded in `1806c12c..021dfe73`).
- Reviewer: independent adversarial review (Fable 5.2). Source not modified.
- Date: 2026-11-26.

**Verdict: RED. 0 Critical, 2 Important, 6 Minor, 7 Nit.**

Both Importants are narrow and have small fixes. Every r1 Critical and
Important fix holds under the tests below.

## Gate

| step | result |
| --- | --- |
| `cargo test --locked --workspace --no-fail-fast` (once, log at `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/aos-fable.log`) | exit 0; 4683 passed, 0 failed, 33 ignored (23 result lines). 80 `address_of_seed` rows and 6 `seed_search::tests` rows ran |
| `cargo clippy --locked --all-targets -- -D warnings` | exit 0 |
| `cargo fmt --check` | exit 0 |

Not run: the manual's `make lint` (no `md`/`ms`/`mk` binaries, markdownlint or
cspell in this container; I compared the flag table with `--help` by hand and
every flag is present), and the `#[ignore]`d Bitcoin Core rows.

## What was exercised

Harness: `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/fb/`
(`lib.py`, `lib2.py`, `t1.py` to `t18.py`, `perf.py`).

- Targets come from r1's independent oracle (rust-miniscript at the pin, no
  toolkit code).
- For every wallet that printed a command, the harness ran the command with the
  secrets on stdin or `@env:`, read the minted keys back from the `bundle` JSON
  (`multisig.cosigners[]`, or the mk1 through `inspect`), and re-derived each
  reported (alternative, index) with the oracle. This is the "MINT-OK" check.
- Every run was grepped on stdout and stderr for the phrases, ms1 strings,
  entropy and the passphrase.

About 330 runs. All of the following found the wallet, with the expected exit
code and MINT-OK on every printed command (twins included), except where a
finding below says otherwise.

- **Single-sig:** `pkh`, `sh(wpkh)`, `wpkh`, `tr` at BIP-44/49/84/86/48/87 and
  `m/45'`, both chains, index 19. Mainnet, testnet, signet, regtest. Passphrase
  by env and stdin. Two seeds without a descriptor.
- **Multisig shapes, every seed order (3 seeds, 6 orders, 3 targets each):**
  - `wsh`, `sh`, `sh(wsh)` with `sortedmulti` and `multi`;
  - `tr(NUMS,sortedmulti_a|multi_a)`;
  - `tr(@0,multi_a(..))` and `tr(@0,sortedmulti_a(..))` (key-path internal key);
  - `or_d(multi,and_v(v:pkh,older))`, `andor`, `thresh`;
  - a tap tree with mixed leaves; `sortedmulti` inside a combinator;
  - bare `/*`, `<0;1;2>`, per-slot `<0;1>` beside `<2;3>` (U1).
- **Slot-order shortcut:**
  - Refugium seats with mixed rounds, all 6 seed orders, sorted and unsorted,
    `wsh` and `tr` (r1 C1);
  - one seed plus cosigners, unplaced and placed;
  - `[fp]@N` and `[fp/path]@N` pins;
  - 3-of-5 with 2 and with 3 seeds;
  - `--independent-paths` in all 6 orders, with a path pin, with a cosigner;
  - a literal xpub beside `@N` (r1 I2).
- **Twin check:** the verdict was compared with the oracle for all 6 `multi`
  orders of a 3-key wallet and for 7 two-target `sortedmulti` pairs: all
  agree. Also `v:multi` and `v:multi_a` inside `and_v`, two fragments, keyed
  mode, M6.
- **Symmetry merge:** L1 (pairing found, 7 merged forms), L2, nested tap trees
  (sibling swap merged, non-siblings not), the tree written the other way
  round.
- **Keyed:** BIP-380 checksums (good, wrong, empty; r1 I3), apostrophe origins,
  bare xpubs, a stated `00000000`, a Refugium-seat origin, account 25, `tr` with
  NUMS and with a key internal, BIP-388 policy JSON, `--descriptor-file`.
- **Cosigners:** mk1 chunks one per flag and interleaved between two cards
  (r1 I6), joined, placed, reversed, upper case, a 3-chunk card; bare xpub
  (B1, B2).
- **Intake:** ms1 (stdin, positional, `--from`), entropy, seedqr, Spanish,
  keyless md1 (chunks and `md1=-`).
- **Refusals and fuzz:** 48 malformed invocations. No panic, no abnormal exit.

r1 I1 (keyless md1 with divergent origins) cannot be minted from the CLI. I
relied on `i1_keyless_md1_divergent_paths_either_seed_order`, which passes, and
on reading `slot_orders` (`seed_search.rs:522-543`).

**Secrets:** no seed word pair, ms1 string, entropy or passphrase appeared on
any stream or in JSON in any run. That includes inline values under
`--allow-argv-secret`, every malformed-secret error, a corrupted ms1 (the
auto-repair path prints nothing), and private keys in `--descriptor`,
`--descriptor-file`, `--cosigner` and policy JSON (refused, not echoed).

## Critical

None.

## Important

### I1. A malformed `--add-path` is dropped silently and then reported as searched (false "no match")

`candidate_paths.rs:182-193` (and the count at `:216-219`) discards a template
that does not parse (`DerivationPath::from_str(..).ok()`). `address_of_seed.rs`
`standard_paths` (`:893-903`) keeps the raw string in `add`, and the no-match
report prints it under `--add-path:` (`:3011-3013`; JSON `searched.add_paths`).
The user is told the path was searched when it was not.

Reproduction: the wallet is `wpkh` at `m/0'` (the path the tool's own hint
suggests), receive 0 = `bc1qgv52mt89gpev6p56huggl970sppqkgftxakv7f` (oracle).

```
mnemonic xpub-search address-of-seed --phrase @env:SA \
  --target-address bc1qgv52mt89gpev6p56huggl970sppqkgftxakv7f --add-path "m/0’"
# exit 4
# no match.
# searched: 6440 child keys on 161 paths, index window [0, 20); 0 wallet(s) match
#   --add-path: m/0’
```

- The same happens with `m/0''`, `m/0H`, `M/0'`, `m / 0'`, `m/0'/`, `garbage`
  and `m/2147483648'`.
- With `m/0'`, `0'` or `m/0h` the wallet is found (162 paths).
- The only sign is "161 paths" instead of 162.

A typographic apostrophe from a pasted path is the likely real case. The drop
is pre-existing in the shared path builder (the sibling modes inherit it), but
this verb is the one that prints the searched set as a certificate (spec §5:
"The report prints exactly what was searched").

**Fix.** Validate every `--add-path` at intake and refuse one that yields no
path (exit 2, naming the template); for an `account` template, check it at
`min_account`. Best done in `candidate_paths.rs` so the siblings get it too.
Add a row: a malformed `--add-path` exits 2.

### I2. The twin check does not look past index 100,000: a fitting twin is missed and the run exits 0

`Analysis::coverage` (`address_of_seed.rs:2002`) scans
`0..gap_limit.min(100_000)`. A matched target at index 100,000 or above is
never in the twin's coverage, so `want.is_subset(&cov)` is false (`:2040`) and
the result is "twin: none" and exit 0. Spec §5: exit 0 means "no twin fits".
The cache ceiling allows such windows (keyed mode, or a narrowed account
window).

Reproduction: keyed `wsh(sortedmulti(2,A,Z))` at `m/48'/0'/0'/2'`, one target at
receive 100003. The oracle confirms `wsh(multi(2,A,Z))` gives the same address
there.

```
... --phrase @env:SA --descriptor '<keyed descriptor>' \
    --target-address <receive 100003> --gap-limit 100010 --external-only
# exit 0, result "match", reasons []          (wrong: the twin fits)
# control, target at receive 99990: exit 4, reasons ["twin"]
```

The same cap hides the sorted twin of an unsorted template. High indices are
rare, but the failure is a false exit 0 on the one check the exit code exists
for.

**Fix.** Evaluate the twin at each matched hit's own (alternative, index)
(spec: "one script per target"), which needs no window scan. Keep a window
scan only for "the twin covers more targets", bounded by `gap_limit`. If any
cap stays, a hit beyond it must give exit 4, not "none". Add a row with a
target above 100,000.

## Minor

- **M1. A repeated `--target-address` gives a false `unmatched` (exit 4).**
  `target_map` (`address_of_seed.rs:464-468`) keeps one index per script, so the
  other copy is never matched and `union != all` (`:2233`).
  Repro: S1's address twice gives `result: "unverified"`,
  `reasons: ["unmatched"]`, and the address is listed as both matched and
  "not found". The same address in upper-case bech32 does the same.
  Fix: dedupe targets by scriptPubKey at intake.
- **M2. The same `--add-path` twice gives a false `split` (exit 4).**
  `--add-path "m/0'" --add-path "m/0'"` prints two identical wallets with the
  same descriptor and `reasons: ["split"]`. r1 M4 was fixed only against the
  seat list (`:895-903`). Fix: dedupe `--add-path` entries by parsed path
  (after account substitution).
- **M3. A sorted template with `--refugium-seats` and 7 or more keys trips
  `cap` spuriously.** The r1 C1 fix turns the collapse off for every segment,
  shared included (`seed_search.rs:535-542`), so a wallet at a standard path
  is hit once per slot order and target.
  Repro: 4-of-7 `wsh(sortedmulti)`, one seed and six cosigners at
  `m/48'/0'/x'/2'`, two targets, `--refugium-seats`: 5040 orders, 34,070,400
  scripts, 39 s, `matches_total` 10080, exit 4 `reasons: ["cap"]`,
  `merged_forms` 4999 (an artefact of the cut). Without the flag: exit 0 in
  under a second. With 8 keys one target is enough.
  Fix: give each segment its own `orders` (r1's first suggestion) so the
  shared and independent segments keep the collapse.
- **M4. The ceilings bound neither memory nor the time before the estimate,
  and the estimate is printed after most of the work.**
  - `early_ceiling` (`:954-983`) passes `--number-of-accounts 3000000
    --gap-limit 1 --external-only` (24 million paths). The run then
    materialises a `PathInfo` list per seed and aborts: "memory allocation of
    21 bytes failed", SIGABRT, 7.3 GB RSS in 11 s under an 8 GB limit, with
    nothing printed.
  - `derive_accounts` (`:1144-1161`) is single-threaded and uncounted.
    `--number-of-accounts 20000 --gap-limit 2` takes 20.8 s wall, and the line
    printed at the end of that wait says "estimated 0.1 s".
  - The estimate line (`:1833`) is written after `derive_accounts` and
    `ChildCache::build` (`:1795`). The manual says "Before it starts, stderr
    states the candidate count, the child-key cache and the estimated time".
  - The twin check's window scan is not in the estimate either (10 to 20 s at
    a 100,000 window).

  Fix: print the counts before deriving; include derivation and cache build in
  the estimate and in progress; derive accounts in parallel; add a ceiling on
  paths (or state the memory cost) so the refusal comes before the allocation.
- **M5. Common keyed exports are refused with a message about `@0`, which the
  user never typed.**
  - A Bitcoin Core descriptor (`wpkh([fp/84h/0h/0h]xpub…/0/*)`) is refused:
    "@0: derivation steps after the placeholder are not representable in
    md1 …".
  - A descriptor that uses one key at two use-sites (`…/<0;1>/*` and
    `…/<2;3>/*`, Liana's recovery path) is refused: "error: @0 appears with
    inconsistent path/multipath/hardening/fingerprint".

  Both follow `bundle`'s limits (spec §2), so the refusal is right. The text
  should name the key and say the shape cannot be put on a card. The second
  message also still carries the doubled "error:" (r1 N2), as does
  "@1 not present; placeholders must be dense 0..n".
- **M6. Two `--cosigner @1=<xpub>` values for one slot fail with
  "base58check decode: decode" (exit 1).** Placed values of one slot are joined
  with a space as if they were mk1 chunks (`:830`, `:856-858`). Expected:
  "@1 is given two keys", exit 2. Fix: join only mk1 values.

## Nit

- **N1.** The manual's exit-0 row says "exactly one wallet". Several seeds
  without a descriptor exit 0 with one wallet per seed (spec §5, and what the
  binary does).
- **N2.** JSON `slots[].template` also takes `cosigner`, `stated` and `md1`,
  which spec §6's list does not name. A keyed-mode seed placed through the
  standard list reports `bipNN`, not `keyed`. The GUI consumes this shape.
- **N3.** A bad `--from seedqr=` echoes a character and its position in the
  secret ("invalid character at position 47: 'x'"). r1 N3 was fixed for
  entropy only.
- **N4.** In the M6 report, the "they differ at" block lists receive 1 for the
  twin, which is the user's own second target, and then advises rerunning
  with that address as a second target. The third wallet carries no
  `distinguishing`.
- **N5.** With `--refugium-seats`, a wallet with one seed at a seat path and
  another at a standard path is not searched, even with
  `--independent-paths` (no-match, 28,240 scripts). This follows spec §4; one
  line in the manual or the no-match hint would help.
- **N6.** `--descriptor '@0/<0;1>/*'` (no wrapper) fails with "unrecognized
  name 'xpub6Bem…'", echoing the internal synthetic key.
- **N7.** Code-reasoned: `fragment_twin` returns `None` for a sorted fragment
  with an argument that is not a slot (a raw public key),
  `address_of_seed.rs:2122-2125`, so that fragment gets no twin check. Such
  templates are rare; a line in "twin:" would make the skip visible.

## r1 fixes: do they hold?

| r1 | status |
| --- | --- |
| C1 Refugium with a sorted template | Holds: 24 runs over all seed orders, plus cosigner, pin and 3-of-5 variants. Side effect: M3 |
| I1 keyless md1, divergent origins | Holds by the suite row and by reading `slot_orders`; not reproducible from the CLI |
| I2 literal xpub beside `@N` | Holds, sorted and unsorted; the lifted key is renumbered after the placeholders and the minted wallet matches |
| I3 checksum on a keyed descriptor | Holds; a wrong and an empty checksum are refused; a checksum on an `@N` template is accepted |
| I4, I5 test rows | `run_printed` now mints and the rows compare addresses; 80 rows run |
| I6 unplaced mk1 chunks | Holds, including chunks of two cards interleaved and a 3-chunk card |
| M1 `no_match` reasons empty | Holds |
| M3 positional ms1 warning | Holds (stderr warning) |
| M4 seat path beside `--add-path` | Holds for seats; the same defect remains between `--add-path` values (M2) |
| M5, M7, M8 | Hold |
| N2 doubled "error:" | Partly: still present on lexer errors (M5) |
| N6 network-mismatch wording | Open by FOLLOWUP `address-of-seed-network-mismatch-wording` |

## Lockstep

- The GUI schema mirror and the GUI manual are deferred by FOLLOWUPs
  `address-of-seed-gui-lockstep`, `address-of-seed-manual-gui` and
  `address-of-seed-lockstep-companions`.
- CLAUDE.md's mirror rule means the merge waits for them. Nothing in this
  branch enforces that.
