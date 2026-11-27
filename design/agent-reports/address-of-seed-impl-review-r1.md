# address-of-seed: implementation review r1 (whole diff)

- Scope: branch `address-of-seed`, `40b35942..2a6b4ddb` (6728c1ab, 8238d2ef,
  3055a0b4, 2a6b4ddb), against `design/SPEC_xpub_search_address_of_seed.md` r3
  (approved 2026-11-26, §10 recommendations 1-9).
- Reviewer: independent adversarial review (Opus). Source not modified.
- Date: 2026-11-26.

**Verdict: RED. 1 Critical, 6 Important, 11 Minor, 8 Nit.**

## What was run

- `cargo build --locked -p mnemonic-toolkit` (dev profile, opt-level 2).
- `cargo test --locked -p mnemonic-toolkit --test all -- address_of_seed`:
  52 passed, 0 failed.
- `cargo test --locked -p mnemonic-toolkit --test all -- lint_ gui_schema every_test_file`:
  99 passed, 2 ignored.
- `cargo test --locked -p mnemonic-toolkit --lib -- seed_search address_of_seed`:
  6 passed.
- Logs are in the session scratchpad
  (`/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/`:
  `test_aos.log`, `lint.log`, `unit.log`).
- **An independent address oracle.** A scratch crate (`scratchpad/oracle`) uses
  the vendored rust-miniscript at the toolkit's pin directly, with no toolkit
  code, to derive `[fp/path]xpub` from a phrase and an address from a concrete
  descriptor. `scratchpad/check.py` runs every printed `bundle_command` (exit
  status) and then rebuilds the concrete descriptor from the printed descriptor
  text, the oracle's xpubs for each `[fp/path]@N` and the `@N.xpub=` of each
  cosigner slot. It compares the address at each reported (alternative, index)
  with the target. For the twins of M1, U1 and N1 and for M4, the minted md1
  was also read back with `restore --md1`. Its first receive address equals
  the target (it confirms that `bundle` keeps the `@1`-before-`@0` order of the
  printed twin).
- End to end, with the "abandon … about" phrase via stdin and `@env:`:
  - S1-S10, P1, T1 (testnet, signet), regtest, H1 (`536870911'` and
    `536870912'`): every exit code matched the spec, every printed command
    exited 0, and every target was re-derived OK.
  - M1, M2, M3, M4, M5, M6, R1, R2, U1, N1, K1, B1, B2, L1, M1p (by the test),
    the mk1 cosigner (placed, two chunks), ms1 (`--ms1-stdin`, `--from ms1=`,
    positional), entropy, seedqr, Spanish (NFC and NFD), a keyless md1
    (single-sig template form), and `/**`: every exit code matched the spec.
    Every printed command exited 0, and every target was re-derived OK.
  - B2 and H1 (`536870912'`) print no command, as the spec says.

## Critical

### C1. `--refugium-seats` with a sorted multisig misses the wallet unless the seeds are given in seat order

`seed_search.rs:477` computes the seat as `32 r + slot`, from the slot the
seed sits in. `address_of_seed.rs:1306-1311` still applies the sorted collapse
(one slot order, seed i in free slot i), and the Refugium segment is added at
`address_of_seed.rs:1331-1333`. So under the collapse, seed 0 is only ever
tried at seats ≡ 0 (mod 32), seed 1 at seats ≡ 1, and so on.

Pruning 4's soundness argument ("a slot contributes nothing but its use-site
path") is false here: in the seat segment the slot also selects the path. The
spec says as much ("the path fixes the slot", §3; "f! × 8^k
assignments-and-tuples", §4). A user who does not know which seed holds seat 0
(the normal case) gets **no match** and the full "searched … Refugium seats:
included" certificate of absence. With k seeds in random order, the miss
probability is 1 − 1/k!.

Reproduction: wallet `wsh(sortedmulti(2, Z@m/270028'/0'/0', A@m/270028'/0'/1'))`,
receive 0 = `bc1qvcvnuw89qe8e50tq6l40mqu752ahzdxhk0g6gh9lcfu7spadgfuscktmns`
(derived by the independent oracle):

```
mnemonic xpub-search address-of-seed --from phrase=@env:SA --from phrase=@env:SZ \
  --descriptor 'wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))' --refugium-seats \
  --target-address bc1qvcvnuw89qe8e50tq6l40mqu752ahzdxhk0g6gh9lcfu7spadgfuscktmns
# exit 4, result no_match, slot_orders 1
```

- With the seeds swapped (`--from phrase=@env:SZ --from phrase=@env:SA`) the
  wallet is found (exit 4, twin).
- `tr(NUMS,sortedmulti_a(...))` misses the same way.
- The unsorted `wsh(multi(...))` equivalent is found (exit 0), because it
  searches f! orders.

This is exactly the "2-seat Refugium multisig" row the implementer reports as
untested.

**Fix.** Do not collapse slot orders in a `PathTuples::Refugium` segment (give
each segment its own `orders`). Alternatively, enumerate the seat per seed with
the slot left free, then check that the seat residues form a permutation of the
slots. Add the §9 row "a 2-seat Refugium-shaped descriptor found per seat" for
sorted and unsorted templates and both seed orders. Add a reference test whose
expected hits come from an independent brute force over (seed → slot, seat)
assignments, not from `Space::decode` (see I6).

## Important

### I1. A keyless md1 with divergent per-slot origin paths has the same defect as C1 (code-reasoned)

`md1_template` (`address_of_seed.rs:699-744`) turns the card's per-slot origin
paths into slot path pins (`address_of_seed.rs:1203-1213`, `1256-1279`). A
keyless card has no fingerprints (the template form strips them), so no seed is
pinned to a slot: the pin binds the path to the **slot**, not to a seed.

Under the sorted collapse, seed i only ever sits in free slot i. So a card
recording `@0` at `m/48'/0'/0'/2'` and `@1` at `m/87'/0'/2'` (M4's wallet) is
found only if the seeds are given in the card's slot order. The spec's argument
"path pins do not break this: a `[fp/path]@N` ties a seed to a path" holds only
for fingerprint pins.

I could not mint a divergent keyless md1 at the toolkit's md-codec pin to
reproduce it: the sibling `md encode` emits a version the pin refuses, and
`bundle --md1-form template` is single-sig. It is the same mechanism as C1.

**Fix.** Disable the collapse when any slot path pin is not tied to a pinned
key (or when the pinned paths differ by slot). Add a test.

### I2. A template that mixes `@N` with a literal xpub silently searches nothing and reports "no match"

`run_address_of_seed` routes any text containing `@\d` to `template_job`
(`address_of_seed.rs:560`). The literal xpub stays a `DescriptorPublicKey::XPub`.
`KeyTranslator::pk` returns `Err(())` for it, so `Template::eval` returns
`None` for every candidate (`seed_search.rs:213`, `223-238`). With one `@N`,
`n == 1` also makes it "single-sig".

The run prints the full no-match certificate ("searched: 6440 child keys on
161 paths … slot orders: 1") for a wallet that is in the space. Writing your
own key as `@0` and pasting the cosigner's xpub inline is a natural way to type
this.

Reproduction (Z's `[3f635a63/48h/0h/0h/2h]xpub…` inline):

```
… --phrase @env:SA --descriptor 'wsh(sortedmulti(2,@0/<0;1>/*,<Z xpub>/<0;1>/*))' \
  --target-address bc1qkpcfs6lhqh3rx63we7435nf3hqpghkdtmmmvln0yrssv0ulhmngs92dens
# exit 4, "no match."
```

**Fix.** Refuse a template with both `@N` placeholders and literal extended
keys (exit 2, pointing at `--cosigner @N=`), or lift each literal key to a
pinned cosigner slot. Also make `eval` failure loud: a `debug_assert`, plus a
pre-scan check that one `eval` on the cache succeeds.

### I3. A keyed descriptor with a valid BIP-380 checksum is refused as "invalid checksum"

`keyed_job` replaces the keys with `@i` and then hands the text to
`Template::parse`, checksum included (`address_of_seed.rs:1388-1445`).
miniscript then checks the original checksum against the rewritten text.

Every coordinator export (Core `listdescriptors`, Sparrow, Specter) carries a
checksum:

```
--descriptor 'wsh(multi(2,<A xpub>/<0;1>/*,<Z xpub>/<0;1>/*))#j9wpt054'
# (checksum verified with rust-miniscript)
# exit 2: "descriptor parse failed: invalid checksum j9wpt054; expected l9v0jjuh"
```

The same descriptor without `#…` exits 0. The error message also tells the user
their correct checksum is wrong.

**Fix.** Verify the checksum on the original text (miniscript or
`descriptor_intake.rs`), strip it, and then templatise. Add a test row with a
checksummed keyed descriptor.

### I4. Tests do not assert what they claim, and the "printed command, run" row is weakened

- `run_printed` (`tests/cli_xpub_search_address_of_seed.rs:1180`) is documented
  as "mints a wallet whose first receive address is the found wallet's
  (checked through `restore --md1`)". It only asserts exit 0, and the callers
  check `out.contains("md1")` (`:1217`, `:1225`).
- The command is run for S1, M3, R2 and B1 only. Spec §9 asks for "every match
  that prints a command" and address equality, including the twins: a twin's
  command is the one most likely to be built wrong.
- `cosigner_as_mk1` (`:801`) does not use an mk1 at all; its comment admits
  this.
- `parallel_scan_equals_reference` (`seed_search.rs:1051`) compares the
  parallel scan with a "reference" that shares `Space::decode` and `eval_rank`.
  It proves the threading, not the enumeration. It never builds a Refugium
  segment, a collapse (`orders: 1`), pins or cosigners, so C1 is invisible to
  it. It also passes the step value as the cache step index
  (`slot_alts[slot][alt].unwrap() as usize`), which only works because the
  steps are 0 and 1.
- `m4_independent_paths` asserts no exit code.
- `refusals_exit_2`'s "above the ceiling" case does not check which refusal
  fired (it is the cache ceiling, not the 10^15 count).

**Fix.**
- Make `run_printed` compare the minted wallet's addresses at each matched
  (alternative, index) with an independent oracle (rust-miniscript in the
  test, as `mint` already does), and run it for every printed command,
  including the twins.
- Rename or replace `cosigner_as_mk1` with a real mk1 test.
- Rebuild the engine reference as an independent brute force over
  (seed, path, slot) assignments, and include `Refugium`, `orders: 1` and pins.

### I5. Spec §9 rows missing, beyond the ones the implementer reports

Not tested at all:
- mk1 cosigner (single- and multi-chunk, placed and unplaced, one unplaced
  cosigner across 3 slots);
- ms1 intake;
- a non-English phrase;
- keyless md1 pins and the template-as-text hint;
- the 2-seat Refugium descriptor (hides C1);
- a keyed descriptor at a Refugium-seat origin;
- the tap-tree sibling-swap merge note;
- the `bc1p` internal-key hint;
- `/**` end to end;
- the second stdin reader via `--descriptor-from md1=-`;
- a JSON snapshot of each `unverified` reason and of `no_match` (it would have
  caught M1 below);
- the spec's "grep all three streams for every seed word and the passphrase"
  (the redaction tests grep stdout only for the words, stderr only for a
  doubled word, and never for the passphrase on stderr);
- the argv-guard table for `--from`, `--ms1` and `--passphrase` on this verb
  (only `--phrase` is tested).

I ran ms1, non-English, a keyless md1, mk1 placed and `/**` by hand: they work.
The unplaced multi-chunk mk1 does not (I6).

**Fix.** Add the rows.

### I6. Unplaced multi-chunk mk1 cosigners are refused, not grouped by chunk-set id (spec §2)

`intake_cosigners` decodes each unplaced `--cosigner` value alone
(`address_of_seed.rs:798`):

```
--cosigner <mk1 chunk 1> --cosigner <mk1 chunk 2>
# exit 2: "mk1 chunked-header malformed: received 1 chunks, header declares total_chunks = 2"
```

It works only when every chunk is packed into one space-separated value
(undocumented in `--help` and in the manual). It is filed as FOLLOWUP
`address-of-seed-mk1-unplaced-chunk-grouping`, but it is an approved-spec
behaviour the implementer's deviation list does not name.

**Fix.** Group the unplaced mk1 values by the chunk-set id in their headers
before decoding. Until then, document the one-value form in `--help` and in
the manual.

## Minor

- **M1. JSON `no_match` carries `reasons: ["unmatched"]`.** Spec §6:
  "`reasons` is empty for the other two" (normative, and the GUI consumes it).
  `address_of_seed.rs:2098-2099` adds `unmatched` before `result` is decided
  (`:2240-2246`). Repro: S10 without `--refugium-seats --json`. Fix: clear
  `reasons` when `found` is empty.
- **M2. A match at `--add-path m` exits 0 with a command `bundle` refuses.**
  The printed descriptor is `wpkh([73c5da0a/]@0/<0;1>/*)`, and `bundle` exits 2
  ("invalid child number format"). `path_h` of an empty path gives an empty
  string (`address_of_seed.rs:345`, `2429`). Spec §1 puts the master in "not
  covered", and exit 0 promises "mintable as printed". Fix: refuse `m` at
  `--add-path`, or report it with no command (exit 4).
- **M3. A positional ms1 on argv is accepted silently.** There is no guard
  refusal and no `secret_in_argv_warning` (exit 0). `--ms1 <inline>` is
  refused. The manual row says "Inline material is refused by the argv guard"
  for `<MS1>` too. This is a pre-existing gap of the `seed_intake.rs` family
  (path-of-xpub does the same), inherited by a verb whose brief is "a leaked
  seed is worse". Fix: fire the positional advisory (as `repair`/`inspect` do),
  correct the manual row, and file a FOLLOWUP for guard coverage of the
  positional.
- **M4. `--refugium-seats` plus an `--add-path` equal to a seat path gives a
  false `split`** (exit 4, two wallets for one path). Repro: S10 with
  `--refugium-seats --add-path "m/270028'/0'/33'"`. The seat list is not
  deduplicated against `--add-path` (`:1103`, `:1170`, `:1283`). Fix: dedupe
  `--add-path` against the seat set too.
- **M5. A bare cosigner's per-target path prints as `m/0/0`.** It comes from
  `child_path` on an empty path (`:349`, `:1052`), in text and in JSON
  `targets[].paths`, so it reads as a real full path from a master. Spec §6:
  each per-key path is "the real full path". Fix: print
  `<origin unknown>/0/0`, or null in JSON.
- **M6. The 2^26 child-key cache ceiling is undocumented.** `--gap-limit` help
  says "At most 2^31". The manual's ceiling paragraph names only 10^15. At the
  defaults (161 paths, 2 chains), a `--gap-limit` above about 208,000 is
  refused for a single seed. It is an acceptable deviation (FOLLOWUP
  `address-of-seed-cache-ceiling`), but help and the manual must say so.
- **M7. `--independent-paths` with a path-pinned seed repeats every assignment
  C times.** The pinned seed's digit is ignored (`seed_search.rs:483-492`), so
  the duplicates inflate `matches_total` and can trip `cap` spuriously. Fix:
  leave path-pinned seeds out of `Independent.seeds`.
- **M8. Text numbering mismatch.** In M6's text, the leading super-twin says
  "twin (of wallet 1)", while the found wallet it twins is printed and later
  labelled as wallet 2 (`:2591` uses the found index, not the output
  position).
- **M9. Lockstep sites from spec §8 not done.** No `cli_network_fail_open.rs`
  per-verb row, no `cli_xpub_search_env_var_sentinel.rs` entry. The cross-repo
  companions (descriptor-mnemonic `address-of-seed-md1-path-step-range`, and
  mnemonic-gui) are "owed", not filed, although `/home/user/descriptor-mnemonic`
  is on this machine. The GUI paired PR and the GUI manual are deferred by
  FOLLOWUP, which is acceptable only if the merge waits for them per the GUI
  mirror invariant.
- **M10. The keyed-mode placement of a key with no origin skips the seat paths
  and does not report it.** Keys without an origin are compared against the
  standard list only, never the seat paths, even with `--refugium-seats`. The
  refusal message names "the standard paths", which is accurate, but a
  Refugium keyed export without origins is refused.
- **M11. Bare-cosigner "header consistent" is checked by depth and child
  number only.** For BIP-48 the last step is the script type, so an xpub at
  account 1 (`m/48'/0'/1'/2'`) "fits" `m/48'/0'/0'/2'` and the command records
  the wrong origin. It is marked "assumed", exit 4. The spec's "any xpub at
  account ≠ 0 is not consistent" is true only for BIP-87. Note it in the
  output ("the account level cannot be read from the xpub").

## Nit

- **N1.** `XpubSearchJson::AddressOfSeed` is appended last
  (`mod.rs:140`); spec §8 says alphabetical (after `AddressOfXpub` /
  `AccountOfDescriptor`).
- **N2.** Refusal text reads "xpub-search address-of-seed: error: …" (double
  "error:") when wrapping `refuse_second_stdin` and the lexer errors.
- **N3.** `--from entropy=` hex errors echo one character and the position of
  the secret value ("Invalid character 'g' at position 29", `:670`).
- **N4.** The twin line differs from the spec sample: "none (no multi order
  fits every target)" instead of "none (multi orders differ: Z,A at index 0,
  A,Z at index 1)".
- **N5.** JSON `targets[].step` is slot 0's step only (`:2367`); with
  per-slot multipaths (U1) slot 1's step is not reported.
- **N6.** The `NetworkMismatch` text for a target address says "xpub version
  bytes distinguish …".
- **N7.** Seeds of different BIP-39 languages cannot be combined (one
  `--language`); worth one manual line.
- **N8.** The private-key regex (`[a-zA-Z]prv` + 20 base58 characters) can
  match inside an mk1 bech32 string (about 0.3% of long cards), refusing a
  public cosigner. It fails closed. Restrict the scan to non-mk1 values.

## Judgement on the implementer's reported deviations

| deviation | judgement |
| --- | --- |
| Sibling engine module (`seed_search.rs`) reusing `permutation_search` (`unrank_permutation`, `factorial`, `cap_decision`, `search_threads`) rather than extending the engine in place | Acceptable. It meets §4.1's mode (collect-all, no early exit, capped and counted list, structure-once evaluator, (path tuple, order) unrank). But its equality test is self-referential (I4) and missed C1. |
| 2^26 child-key ceiling | Acceptable as a v1 limit, refused fail-closed with flags named; FOLLOWUP filed. Help and the manual must state it (M6). |
| Keyed md1 refused | Acceptable deviation from §2 with a FOLLOWUP and a clear message. |
| No tests for mk1, ms1, md1, non-English, 2-seat Refugium | **Not acceptable.** The 2-seat Refugium gap hid C1. The mk1 gap hid I6 (the approved unplaced-chunk behaviour is absent). The others work when run by hand (above) but must be pinned. |

## §10 decisions

- **1** (shared default, `--independent-paths` opt-in): implemented. The
  no-match hint does not print "the independent command and its estimated
  time"; it names the flag only (minor).
- **2** (20 accounts, gap 20, both chains): implemented.
- **3** (`--refugium-seats` opt-in, 8 per slot or 256 for one key):
  implemented, but broken with sorted templates (C1).
- **4** (repeatable `--from`): implemented.
- **5** (one seed in two slots refused): implemented, FOLLOWUP filed.
- **6** (twin exit 4 with three distinguishing indices, keyed too):
  implemented. D1 addresses match the spec sample exactly.
- **7** (no-match exit 4): implemented.
- **8** (bare cosigner assumed or placeholder, exit 4): implemented
  (B1 runs, exit 0; B2 null command plus placeholder).
- **9** (accounts ≥ 2^29: no command, exit 4): implemented (H1).

## Security summary

- The argv guard covers `--phrase`, `--ms1`, `--passphrase` and every repeated
  `--from <secret>=`, since the guard walks every token.
- Positional ms1 is not covered (M3).
- The printed commands are built from references only:
  - inline seeds become `-` or `@env:MNEMONIC_SEED_<i>`, and an inline
    passphrase becomes `@env:MNEMONIC_PASSPHRASE`, as checked by hand;
  - `redact_and_quote` runs as a second check;
  - the redaction note goes to stderr in text mode and to `redaction_note` in
    JSON.
- No seed word or passphrase appeared on stdout or stderr in any run here.
- Worker threads see only xpubs and child public keys.
- The masters are dropped before the scan.
- The passphrase and the seed are `Zeroizing`.
- The `bip39::Mnemonic` values live through the scan and the report. bip39's
  `zeroize` feature is not enabled in the lock, which is pre-existing and
  toolkit-wide.
- The master `Xpriv`s are not zeroized either (same as path-of-xpub).
