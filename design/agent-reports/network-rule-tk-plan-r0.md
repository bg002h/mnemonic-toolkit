# R0 review: IMPLEMENTATION_PLAN_network_rule_tk.md (plan commit 66a798a0 on master d72a18b1)

Reviewer: independent adversarial architect (Claude Opus 5.5), 2026-11-28.
Scope: the plan against Spec A (`design/SPEC_network_rule.md`), Spec B
(`design/SPEC_network_rule_tk_rest.md`), `design/network-rule/vectors.tsv`,
the fixtures and `scripts/spec-network-rule-gate.sh`, and the source tree at
d72a18b1. Nothing was edited in `src/` or in the plan.

How the checks were run: every `file:line` cite in the plan was opened at that
line; current behaviour was checked with the cached `target/debug/mnemonic`
(built from this tree) on the public test data the gate uses (`$P`, `$X`, `$T`,
the `fx/` fixtures); the spec gate was run with `--no-red` (it needs
`SHIB=/home/user` on this box: GATE GREEN, 28 cites, 32 text ids, 61 vectors).

Counts: **Critical 0, Important 10, Minor 16, Nit 4.**

Verdict: **not GREEN.** The unit-0 helper's data model cannot express three
bindings that the spec's RED rows depend on (I-1, I-2, I-3), one arm sits at
the wrong site and meets a flag the code ignores (I-4), the import-wallet parser
change is underspecified so that the rule may never run (I-5), one fixture
recipe fails today (I-6), and one N4 contradiction is still missing from the
arms (I-7).

What holds up (checked, no finding): N8 order (H4C, H4, H4P, H2, H1, H3, H5, then
SOFT); the drop rules (flag sentence when any HARD; TX-H1 rerun sentence with
TX-H2; TX-H3/TX-H1 rerun clauses for a disagreeing key and address; no TX-H3 for
an address that agrees with the run); TX-H4C suppressing every run compare and
every TX-S inside the file; a coin that agrees with its own key named on a HARD
line printing nothing; N6 (TX-N with no flag, TX-T with a test flag, nothing on
`--network mainnet`, nothing for a seed typed now); N7 (TX-S-OK once per
overridden contradiction, the flag with nothing to override prints nothing); the
N10 shape and `class:"soft"` only when every contradiction is soft. Every
`any`/`mnemonic` vector row (V01-V25, V28-V33, V42-V46, V60: 37 rows) runs
through the helper as the plan maps the columns, given I-1's fix for the
`stated2` rows. Every RED row R1.1-R1.21, R2.1-R2.9, R3.1-R3.10, R7.1-R7.10,
R8a.1-R8a.24, R8b.1/2/4 maps to a TRACE name the plan carries; the 13 `nr1_*`,
4 `nr2_*`, 4 `nr3_*`, 5 `nr7_*`, 18 `nr8a_*` and the two toolkit `nr8b_*` names
are all present (unit 1's by reference to Spec A; spelling them out in the plan
would cost nothing). The md/mk parts of 8b (R8b.3, R8b.5, R8b.6,
`nr8b_md_from_mk1_version_refused`) are rightly left out. The cites I checked
all hold to within a line or two (exceptions in M-3, M-4, N-1). Clap defaults
are as the plan says (`export_wallet.rs:294`, `path_of_xpub.rs:83`,
`account_of_descriptor.rs:95`, `passphrase_of_xpub.rs:114`, `silent_payment.rs:53`,
`nostr.rs:96` carry defaults; `bundle.rs:23` and `verify_bundle.rs:25` are
required; the rest are `Option`). No Args struct is built by a struct literal
anywhere, so `#[arg(skip)] network_given` breaks nothing. vectors.tsv is the same
file at 36c1f155 and d72a18b1, so A11's pin is right.

---

## Important

### I-1 The keys of a stated file are listed twice, and nothing ties them to the file
Plan §2.1 `SourceKind::Stated { keys: Vec<SectionKey> }` (the type is never
defined), and §3 unit 1 `--from-import-json`: "Stated{... keys: envelope keys
"bundle.descriptor"} **plus** each key `key @i` ... with its origin coin".
`Key` has no owner field, so `evaluate` cannot tell that `key @0` belongs to the
envelope. Run R1.13 (env_t.json says testnet, tpub keys, `--network mainnet`)
through that model and it gives TX-H4 **and** TX-H1 for `key @0`. N8 says "only
TX-H4 prints", and R1.13's after leg is TX-H4 alone. R1.21 (TX-H4C only, "not
compared with the run") fails the same way. The vector harness (§2.5) instead
puts `stated2`'s keys only inside `Stated.keys`, so V32 and V33 would pass while
the arm tests fail. That is two different models. V33 (`stated2=main;key=main;coin=1`)
also needs the coin bound to a section key, and the plan never gives
`SectionKey` a coin.
Fix: define `SectionKey { section: String /* TX-H4C <section> */, who: String /* TX-S <who>: "key @i" for an envelope, the section name for Coldcard */, pfx, class, path_coin: Option<CoinRead> }`.
State that a stated file's keys appear only in `Stated.keys` and never again as
`Key` sources. Use that one model in both the vector harness and the arms.
Separately, decide whether a coin that agrees with its own key inside a TX-H4
file (the file agrees with itself but disagrees with the run) prints a TX-S. The
spec's "key named on a HARD line" does not literally cover it. Treat the keys of
a TX-H4 file as named on that line (no TX-S), and record the reading as A12.

### I-2 restore `--md1` + `--cosigner`: a slot has two coin types and the model holds one
Plan §3 unit 2 makes each md1 slot coin a stand-alone `Coin{who:"key @N"}`
(run form) and each cosigner a `Key{who:"--cosigner @N's mk1 card", path_coin = the mk1's own coin}`.
Spec A §4.1 "Restore md1 rules" compares the slot's md1 coin with **that
cosigner's key version**: TX-S key form, or nothing when the key is on TX-H1.
With a stand-alone `Coin`, R2.4 (m_signet coin 1, tpub cosigners, no flag)
prints TX-H1 plus a TX-S run form (`key @0's path m/87h/1h/0h ... but this run is on mainnet`).
R2.4's after leg says "TX-H1 ..., nothing else". `Key.path_coin` is one
`Option`, so it cannot carry both the md1 slot's coin and the mk1's own coin.
The key-form text also needs two different `<who>`s here (path `key @0`, key
`--cosigner @0's mk1 card`), and §3's single `<who>` does not give that wording.
Fix: give a key a list of coins, each with its own `who` for the path
(`Vec<(who, CoinRead)>`), or give `Coin` an `own_key: Option<usize>`. Make a
slot's md1 coin bind to the cosigner that fills the slot. Record the TX-S wording
for "path from the md1, key from the mk1" as an ambiguity for the spec owner;
the closest normative form is me's "bundle form" of TX-S. The R2.4 and R2.7
tests must pin that nothing else prints.

### I-3 TX-S key-form merging is unspecified, and R3.6/R3.10 need it
Plan §2.1 merges only "one-class sources that agree with each other and
disagree with the run". In R3.6 (jade_T_0.json: three tpubs under one
`Derivation: m/48'/0'/0'/2'`, no flag), the three coin-0 paths agree with the
run and disagree with their own keys. The plan's rule does not merge them, so it
gives three TX-S lines. Spec B says "then TX-S", and §3 gives plural forms that
exist only for the key form ("`<who> is a <kclass> key` becomes
`keys @0 and @1 are <kclass> keys`"). That implies one merged line:
`jade keys @0, @1 and @2's paths m/48h/0h/0h/2h, m/48h/0h/0h/2h and m/48h/0h/0h/2h are mainnet paths (coin type 0h), but jade keys @0, @1 and @2 are test-network keys (tpub). Nothing was written. If the path is a mistake, ...`
Since the test pins the full stderr, the implementer cannot guess.
Fix: add the merge rule. Key-form TX-S contradictions with the same `<pclass>`
and the same `<kclass>` are one line. Run-form TX-S lines merge per coin class
(Spec A §4.1 "one TX-S run-form line per coin class"). Merging never crosses
text ids. If the plan's authors read it otherwise, raise it with the spec owner
before the R3.6 test is written.

### I-4 restore `--origin` (R8a.6): wrong site, and single-sig restore ignores `--origin`
The plan's 8a row puts the check at "`restore.rs:965` `explicit_origin` and
single-sig `--origin`". Line 965 is in `run_singlesig_template_completion`
(keyless md1 template), where `--origin` is used. R8a.6 runs the **plain
single-sig** arm (`restore --from phrase= --origin m/84h/0h/0h --network testnet --template bip84`,
no `--md1`). That arm never reads `args.origin`: the only reads are
`restore.rs:965` and `:1578`. The help says so (`restore.rs:127-131`: "Only
meaningful for keyless single-sig template md1 restore; ignored otherwise").
Measured: `--origin m/84h/0h/0h` and `--origin m/84h/5h/0h` on testnet both
print `wpkh([73c5da0a/84'/1'/0']tpubDC8msF...)`, rc 0.

So the TX-S refusal has no code site in the plan. Worse, with
`--accept-coin-type-mismatch` the run would print TX-S-OK ("the path
m/84h/0h/0h given with --origin is a mainnet path; the output is for testnet")
over a wallet at `m/84'/1'/0'`, a path the user never gave. That breaks N7's
"every path kept as written" and misleads the user about which keys were made.
Fix: name the site in `run` (after `parse_from_input`, before the derivation
around `restore.rs:530`). Add an ambiguity entry and get a ruling. Either
single-sig restore derives at `--origin` when it is given (as template
completion does), or `--origin` without `--md1` is refused as a mode violation,
which would make R8a.6's after leg a different refusal. Do not ship the override
leg as it stands. The plan's unit-2 line "Keyless single-sig template ...
template origin coin vs run" is also wrong: that template carries no origin (the
coin comes from `--network` via `derive_bip32_from_entropy`). Its only source is
`--origin`, which is the 8a arm.

### I-5 import-wallet: the parsers' coin-type refusals run first, and the plan leaves them unresolved
Plan §3 unit 3: "the coin-heterogeneity refusal is kept only where it is not a
network question". Every origin-bearing parser infers the network from coin
types and refuses before any network check. For example
`wallet_import/descriptor.rs:177-204` refuses "cosigner {i} has coin-type {ct},
cosigner 0 has coin-type {first}" (ImportWalletParse), "unsupported coin-type
{other} ... only 0 and 1", and "origin path has only {n} components". Specter,
sparrow, bsms, bitcoin-core and coldcard_multisig (`network_from_path`,
`:805`) have the same refusals. These **are** network questions: they exist
only to infer `p.network`, which the run now decides. If they stay, a blob with
keys at mixed coin types is refused with exit 2 (ImportWalletParse) before the
rule runs. That contradicts Spec A §4.1 "import-wallet, every parser: key
version against the blob's coin type → TX-S, 8/0, flag". If they go, blobs at
BIP-45 or short origins become importable, a widening N1 permits ("no source: no
check").
Fix: list, per parser, each `network_from_*` refusal and whether it is deleted,
and say what `p.network` holds before the check (a placeholder overwritten by
the run network). Delete the heterogeneity and coin-not-in-{0,1} refusals, and
keep the "need ≥2 components" refusal only if something other than network
inference needs it. Add a test: a descriptor blob with an xpub at coin 0 and one
at coin 1 on mainnet gives TX-S for the coin-1 key, exit 8, and with the flag
exit 0 and TX-S-OK. Fold this into `nr8a_import_wallet_coin_type_flag`.

### I-6 The verify-bundle fixture recipe fails today, and every route adds a second source
Plan §3 8a fixtures: make a watch-only bundle with
`bundle --network signet --template bip84 --slot @0.xpub=$T --slot @0.fingerprint=73c5da0a`,
then "verify its md1+mk1 with `--network mainnet`". Measured: positional
md1+mk1 with `--template bip84` exits 1 with `error: slot count 0 out of range 1..=16`
on **both** signet and mainnet. That is the very error Spec B warns about.
`--bundle-json` fails the same way, and without `--template` it exits 2. The
only route that works adds `--slot @0.xpub=$T --slot @0.fingerprint=...`, and on
mainnet that refuses through synthesize's `CosignerSpec`
(`--cosigner[0]: xpub network Test does not match --network mainnet`, exit 1).
That is the 8b "watch-only slot" arm, not the mk1-card arm. A tpub `--slot` plus
a tpub mk1 card is two test sources against the run, so N8 merges them into one
plural TX-H1 ("--slot @0.xpub and the mk1 card are ..."), a form no row pins.
The verify-bundle row also names no code site among the six verify modes
(`verify_singlesig_template`, `verify_multisig_template`, `run_full`,
`run_watch_only`, `run_multisig`, `descriptor_mode_verify_run`).
Fix: build and run the fixture before writing the plan's text. Pick a mode in
which the mk1 card is the only key source (e.g. a seed-slot multisig bundle made
on signet, verified with `--slot @N.phrase=` for the own slot and the foreign
mk1 at coin 1). Name the site in each mode the rows reach, say whether the
`--slot` key and the mk1 card merge, and check TX-V's condition with the flag.

### I-7 bundle `--import-json`: an envelope that contradicts itself still passes
Plan §3 8b gives `bundle --import-json` only TX-H4 (the envelope's network
against the run), as Spec A §4.1's row does. N4 is unconditional: the envelope's
stated network against its own keys is a HARD contradiction (TX-H4C), and its
keys' coin types are SOFT (as R1.19 for export-wallet). Measured:
`bundle --import-json env_xt.json --network testnet` (envelope says testnet, key
is mainnet `xpub6CatWd...`) exits 0 and re-emits md1 and mk1 cards for it today.
The plan keeps that. The per-arm table is outside the gate's coverage (Spec A
§13, "completeness of the per-arm table"), so this is a table gap and N4 still
binds.
Fix: give `bundle --import-json` the same check as `export-wallet --from-import-json`
(one helper building `Stated{keys}` from an envelope, used by both arms), with a
leg each for TX-H4C (env_xt.json) and TX-S (env_x_c1.json). Or, if the
coordinator rules it out of scope, file a FOLLOWUP that cites N4 and record the
decision.

### I-8 The default (synthesized) origin path must not be a source, and the plan never says so
Several arms build an origin path from the run network when none is typed:
export-wallet/bundle `--template/--slot` with no `@N.path=` (`resolve_slots`,
`synthesize.rs:657` `default_origin_path(network, ...)`), restore single-sig and
template completion (`derive_bip32_from_entropy(.., network, ..)`), convert
`--template` without `--path`, and Coldcard's legacy top-level xpub path built
from `chain` (`coldcard.rs:276-286`). If such a path is read as an N1 coin, R1.12
(`--slot @0.xpub=$T`, no flag) gets TX-H1 **plus** a TX-S key form (coin 0 from
the mainnet default against a tpub, the V15 shape). R1.12 and R1.20 say TX-H1
only. R8a.11's after leg (TX-S key form) is about a **typed** `@0.path=`.
Fix: one sentence in §2.1, "only a path the input carries (typed or read from a
kept artifact) is a coin source; a path the toolkit derives from `--network` or
from a file's `chain` is not". Add a regression leg to
`nr1_export_wallet_slot_key_refused_exit_8` that pins "exactly one line".

### I-9 A3's proposed `<who>` for path flags contradicts §3
A3 proposes the key form for `--slot @N.path=` with an xpub slot as
"`--slot @0.xpub`'s path ...". Spec A §3, `<who>` by arm, is explicit: for "a
path flag as the TX-S source (convert --path, restore --origin, --slot
@N.path)", the phrase "`<who>'s path <path>`" is written "`the path <path> given with --path`"
(with `--origin` / `--slot @N.path`). That holds in both TX-S forms. So R8a.11
reads `the path m/84h/0h/0h given with --slot @0.path is a mainnet path (coin type 0h), but --slot @0.xpub is a test-network key (tpub). ...`,
and the N10 `a.what` is `the path m/84h/0h/0h given with --slot @0.path`. A3's
run-form reading is right; the key-form proposal is not.
Fix: settle A3 on the §3 reading and pin R8a.11 and R8a.12 as literal strings.

### I-10 No census of the tests the rule flips; as1-i4 is closed without its fixture
(a) The plan names six existing test files that flip. Test files that use test
material number 36 (`grep -l "tpub\|tprv" tests/*.rs`), among them
`cli_addresses.rs`, `cli_convert_*.rs`, `cli_derive_child.rs`, `cli_restore*.rs`,
`cli_export_wallet_*.rs`, `cli_import_wallet_*.rs`,
`cli_xpub_search_address_of_xpub.rs`, `cli_output_class.rs` and
`prop_backup_restore_roundtrip.rs`. Many run test keys with no `--network`, which
D1 now refuses. Without a census an implementer meets dozens of red tests
mid-unit, with nothing saying which flip by the spec and which are regressions.
The easy fix, loosening assertions, is the dangerous one.
Fix: add a step 0. Run the whole suite once with a stub that refuses
everywhere, or better, grep. List each test the rule flips with the spec row
that flips it. At each unit, check that every newly red test is on the list and
is changed to the spec's text and exit, never deleted or loosened.
(b) §6 step 9 closes `netrule-r3-as1-i4-restore-cosigner-soft-flag`
(`design/FOLLOWUPS.md:6848`: "a SOFT-only run passes with
--accept-coin-type-mismatch and TX-S-OK ...; TRACE leg needs a new fixture").
Unit 2 lists only `fx/m_bip45.txt`; the SOFT-only cosigner fixture (an md1 slot
coin against a cosigner key that agrees with the run) and its leg in
`nr2_restore_md1_cosigner_version_refused_flag_ignored` are missing.
Fix: add the fixture and the leg, or leave as1-i4 open.

---

## Minor

- **M-1 A9 misstates today's behaviour.** `bundle.rs:2116-2137` does not
  accept `--network signet` against a `testnet` envelope. It refuses both cases
  (BadInput, exit 1) and only words the same-class case longer. Following N3
  therefore keeps a refusal; it widens nothing. Correct A9.
- **M-2 §4.2's manual claim is false.** The import-wallet exit table
  (`41-mnemonic.md:1771-1780`) has no class-mismatch row: row `1` lists
  AmbiguousFormat, FormatMismatch and EnvVarMissing. Drop "loses the exit-1
  class-mismatch row" and add the `8` row. Also, only a few verbs have exit
  tables (import-wallet; the xpub-search pages at :4201, :4319, :4438; repair),
  so "each verb's exit table gains row 8" needs a list.
- **M-3 TX-R data-flow cites.** `convert.rs:1299` is `fn convert_target_class`.
  The advisory call is at `:1289`, and convert computes the worst class over its
  outputs, which is where `private_out` must be set. Bundle `:1112`, derive-child
  `:316`/`:333`, import-wallet `:1394`, silent-payment `:366` are right.
- **M-4 restore `--expect-xpub` sites.** The compare is `restore.rs:607-624`,
  not `:626-629`, and template completion has a second compare at `:1044`
  (R8a.22-style, with `--md1`). List both; the check precedes both.
- **M-5 "Last line on stderr" against `finish_exit`.** `cli.rs::finish_exit`
  runs `mlock::report_at_exit()`, which can print two stderr lines after the
  verb, so after TX-R (N12, "last line of the run") and after the N10 machine
  line ("the last on stderr"). Print TX-R and the machine line after
  `report_at_exit`, or move `report_at_exit` ahead of them.
- **M-6 Coldcard "file key order".** `serde_json` is built without
  `preserve_order` (no indexmap in `Cargo.lock`), so `Map` iterates
  alphabetically. Do not turn the feature on: it would reorder every JSON output
  the toolkit writes. Order sections by name (bip44 < bip48_1 < bip48_2 < bip49 < bip84 < bip86 < xpub)
  and record that as the reading of "the file's key order".
- **M-7 Types the plan names but never defines.** `Who`, `SectionKey`
  (see I-1) and `StatedText`. `Input.secret_out` is never used, because "TX-R
  comes from the hook", while Spec A unit 0 says the helper takes `secret_out`
  and returns the lines including TX-R. Either have `evaluate` return TX-R when
  `secret_out` holds and a test run is on (the hook then prints what the helper
  returns, last), or drop the field and say the predicate is the spec's helper
  output.
- **M-8 Module location.** Spec A §11 unit 0 and §12 put the helper in
  `network.rs` ("the helper is pub in network.rs"). The plan's
  `network_rule.rs` is harmless, but it differs from the spec: note it, or
  re-export from `network.rs`.
- **M-9 A coin compared with a stated network.** Say that a coin is compared
  with its own key if it has one, else with the run, and never separately with a
  `Stated` source (V33 must give one TX-S, not two).
- **M-10 Keyless template completion is a seed-typed-now run.** Spec B:
  "restore --md1 of a keyless template with --from ... (N6): no TX-N/TX-T". The
  plan sets `reads_kept = true` only for the keyed arm. Set `seed_only` (or
  `reads_kept = false`) explicitly on both template-completion arms.
- **M-11 Deleting `wallet_export/bsms.rs:133`.** The bsms emitter is reached
  from export-wallet's three arms and from restore's `--format` paths
  (`restore.rs:3208` mirrors `EmitInputs`). Keep the guard, moved to
  `NetworkRule`, or show that every caller has checked first.
- **M-12 Help and manual text that still say "inferred".** Besides
  `addresses.rs:54`, `xpub_search/address_of_xpub.rs:62` ("Defaults to network
  inferred from the xpub version") and manual `41-mnemonic.md:4369` change. The
  toolkit CLAUDE.md "GUI manual lockstep" hazard also covers `docs/manual-gui`
  (`4g-xpub-search-address-of-xpub.md:31` "default inferred from the xpub",
  `4e-build-descriptor.md`), and §4.2 does not mention it.
- **M-13 mnemonic-gui paired PR.** Spec A §12 item 2 and Spec D :54 say the
  first mirror commit carries `nr8f_schema_flag_gui_managed`; add it to §4.3.
- **M-14 One PR.** Spec A §11 "Unit 8: the rest (one PR each, in this order)".
  The plan ships 0, 1, 2, 3, 7, 8a and 8b-tk as one PR. Split 8a and 8b-tk out,
  or record the coordinator's waiver in §1.
- **M-15 Several targets, several importers.** address-of-xpub's
  `--target-address` is a `Vec` (`address_of_xpub.rs:43`): say how several
  addresses merge (N8) and how the key+address drop applies per address.
  Bitcoin Core `listdescriptors` yields several `ParsedImport`s: say whether
  `key @i` counts within each descriptor, and whether the check runs before or
  after `--select-descriptor`. Say how a plural line writes mixed prefixes
  (`(tpub)` vs `(tpub, vpub)`).
- **M-16 Tests rendered by the code under test.** §3 "asserts ... the stderr
  text by its full rendered string from `network_rule::texts`" is circular for
  wording. Keep that, but make the unit tests' expected strings literals copied
  from §3, and make every after leg the spec quotes (R1.19, R1.20, R1.21, R2.4,
  R3.1, R3.2, R3.6, R8a.8, R8a.24, R8a.7) a literal in its arm test.

## Nit

- **N-1** `cli.rs:257-267` is `:255-268`; build-descriptor's `--emit-spec`
  sentence is `:100-101`, not `:102`.
- **N-2** The gate needs `SHIB=/home/user` on this box (no
  `/scratch/code/shibboleth`); say so in §6 or the per-commit gate fails on
  cites.
- **N-3** The `m_bip45.txt` recipe: `convert --to fingerprint` needs
  `--template`, and the `[fp/45h]` placeholders must be real fingerprints.
  Spell the commands out (bundle fails "master fingerprint should be 8
  characters long" when they are not).
- **N-4** The claims about open PRs #227-#230 (exit 8 kept by #228, hidden
  aliases in #229) were not verifiable from the tree. Cite the commit or file
  that says so.

---

## Answers to the brief's direct questions

- **Stdout before the check / a file written on exit 8:** none found on the
  arms the plan names, provided each check sits where §3 puts it (before
  `emit_payload` at `export_wallet.rs:830`, before step 5 at
  `restore.rs:4073`, before `:212`/`:243` in build-descriptor, before
  `convert.rs:1085`). The gaps are the sites the plan does not name: I-4
  (single-sig restore) and I-6 (verify-bundle modes). Only RepairShortCircuit
  output (convert auto-repair at `convert.rs:1133`) reaches stdout first, and it
  is no network output.
- **Funds-relevant slips that remain:** I-7 (bundle `--import-json`
  self-contradiction, exit 0 today and after); I-4's override leg (output at a
  path the user did not give, under a warning that says otherwise); I-5 if the
  parser refusals are kept in the wrong order (no wrong output, but the wrong
  refusal and no flag).
- **Scope:** nothing out of scope is included. The md/mk parts of 8b are
  correctly deferred. Missing from scope: I-7 (or its FOLLOWUP), the as1-i4 leg
  (I-10b) and `docs/manual-gui` (M-12).
