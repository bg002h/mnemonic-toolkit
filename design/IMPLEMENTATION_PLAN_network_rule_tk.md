# IMPLEMENTATION PLAN: the network rule in mnemonic-toolkit (units 0, 1, 2, 3, 7, 8a, 8b-toolkit)

Status: plan r2 (author: Claude Opus 5.5, 2026-11-28). r0 at 66a798a0; r1
(c8a3f61d) folds R0 review `design/agent-reports/network-rule-tk-plan-r0.md`
(0C/10I/16M/4N) and the coordinator's rulings on it; r2 folds R1 review
`design/agent-reports/network-rule-tk-plan-r1.md` (0C/2I/7M/1N) and the
coordinator's rulings on that (§8 lists every finding's disposition, R0 and R1).
Normative: `design/SPEC_network_rule.md` (Spec A: §1-§13, units 0, 1, 7) and
`design/SPEC_network_rule_tk_rest.md` (Spec B: units 2, 3, 8a, 8b). Where this
plan and a spec disagree, the spec wins and the plan is the defect. The §3 text
templates live once in code (`network_rule::texts`); tests pin the spec-quoted
legs with LITERAL strings (§3.0), never with the code's own renderer.

## 1. Baseline, base branch, PR shape

- Baseline: `origin/master` **d72a18b1** (FOLLOWUPS: 32 owed-to-build lines, #198).
  Every `file:line` below is at d72a18b1 and was grep-checked when written.
- Branch `claude/project-thread-cbnpjn`, based on master.
- **One PR** for units 0, 1, 2, 3, 7, 8a and 8b-toolkit. Spec A §11 "Unit 8:
  the rest (one PR each, in this order)" asks for one PR per unit; Brian chose
  one PR for this round (via the coordinator, 2026-11-28). The PR body records
  the deviation; commits stay one per unit so review can still go unit by unit.
- Open PRs on the same files (none builds the rule):
  - #227 bug-hunt fixes (base master), and stacked on it #228 exit codes, #229
    `--scheme` rename, #230 address-of-seed.
  - #228 keeps 8 for a network mismatch: its spec
    `mnemonic-engrave:design/SPEC_constellation_exit_codes.md` on branch
    `claude/project-thread-o57s2k`, decision 1 (lines 90-100: "bit 8 becomes
    REFUSED ... with network mismatch as one of its members ... Its
    machine-readable stderr line (the network rule's N10) is what tells it apart")
    and its table row at line 371. #228 rewrites `error.rs::exit_code`.
  - #229 (title: "named wallet shapes are --scheme (--template / --archetype
    kept as hidden aliases)") renames flags the RED rows type; the aliases keep
    the commands valid.
  - This PR rebases onto whichever of #227/#228/#229 merges first. Expected
    conflicts: `error.rs` match blocks, `cli.rs`, `export_wallet.rs`,
    `restore.rs`, `build_descriptor.rs`, `docs/manual/.../41-mnemonic.md`. One
    line per variant, alphabetical, keeps the rebase mechanical.
- Out of scope, deferred to later PRs after this one is reviewed: the md and
  mk parts of 8b (md-cli parse/keys.rs, decompose, seat; mk-cli address.rs;
  their unit-0 parts; rows R8b.3, R8b.5, R8b.6, TRACE
  `nr8b_md_from_mk1_version_refused`), units 4, 5, 6, 8c-8h. mnemonic-gui: only
  the minimal paired mirror change (§4.3).

## 2. Unit 0: the rule as code

### 2.1 Module and types

Spec A §11/§12 put the helper in `network.rs`. This plan puts it in a new
`crates/mnemonic-toolkit/src/network_rule.rs` (`pub mod network_rule;` in
`lib.rs`) and re-exports it from `network.rs` (`pub use crate::network_rule::*;`
inside a `pub mod rule` there, or a plain re-export of `evaluate`), so the
spec's "pub in network.rs" holds for callers. `network.rs` keeps `CliNetwork`.

```rust
pub const ACCEPT_COIN_TYPE_FLAG: &str = "--accept-coin-type-mismatch"; // N7
pub const EXIT_NETWORK_MISMATCH: u8 = 8;                               // N9
pub const MACHINE_ENV: &str = "MNEMONIC_NETWORK_JSON";                 // N10

pub enum Class { Main, Test }
pub enum Hrp { Bc, Tb, Bcrt }
pub enum ExactNet { Main, Test /* testnet, testnet3 */, Testnet4, Signet, Regtest }
pub enum Grain { Class(Class), Hrp(Hrp), Exact(ExactNet) }

/// N10 `a`/`b`: the text's own name for a source and its network at the grain compared.
pub struct Who { pub what: String, pub network: &'static str } // main|test|signet|testnet|testnet4|regtest

/// The run network (N2). `given` drives `<run>`.
pub struct RunNet { pub net: CliNetwork, pub given: bool }

/// A coin type read from a path (N1 SOFT shape only).
pub struct CoinRead { pub path_text: String /* m/84h/1h/0h */, pub coin: u32 }
/// Some only for purpose in {44,48,49,84,86,87,352,270028} and coin in {0,1}.
pub fn coin_of_path(p: &bitcoin::bip32::DerivationPath) -> Option<CoinRead>;

/// One coin a key carries, with the `<who>` its path is named by.
pub struct KeyCoin {
    pub path_who: PathWho,   // how "<who>'s path <path>" renders
    pub coin: CoinRead,
}
pub enum PathWho {
    OwnKey,                  // "<key who>'s path <path>" (origin of the key itself)
    Named(String),           // another name, e.g. "key @0" (md1 slot) -> "key @0's path <path>"
    Flag(String),            // "the path <path> given with --slot @0.path" (Spec A §3 <who> by arm)
}

/// A key of a stated file (Coldcard section, envelope key). Appears ONLY here,
/// never also as a top-level `Key` (I-1).
pub struct SectionKey {
    pub section: String,     // TX-H4C <section>: "bip84", "bip48_2", "xpub", "bundle.descriptor"
    pub who: String,         // TX-S <who>: "key @i" (envelope) / section name (Coldcard)
    pub pfx: String, pub class: Class,
    pub coins: Vec<KeyCoin>,
}
/// TX-H4 <stated>: an exact name ("testnet") or a two-valued statement
/// ("chain BTC (mainnet)", "chain XTN (a test network)", "chain XRT (a test network)").
pub struct StatedText { pub text: String, pub grain: Grain }
pub enum Device { Coldcard, Signer }

pub enum Source {
    Key { who: String, pfx: String /* as typed, A2 */, class: Class, coins: Vec<KeyCoin> },
    Coin { coin: KeyCoin },                 // a coin with no key version: always against the run
    Address { who: String, address: String, grain: Grain },     // TX-H3
    Wif { class: Class, version: u8 },                          // TX-H5
    Stated { file: String, stated: StatedText, device: Device, keys: Vec<SectionKey> }, // TX-H4 / H4C
}

pub enum Thing { Card, Key, Descriptor }    // TX-T <thing>
pub struct Input {
    pub run: RunNet,
    pub sources: Vec<Source>,   // input order (N8 "by input order")
    pub reads_kept: bool,       // N6
    pub seed_only: bool,        // N6 exemption: the run only derives from a seed typed now
    pub secret_out: bool,       // N12: test run + secret input + PrivateKeyMaterial on stdout
    pub accept_coin_type: bool, // N7
    pub thing: Thing,
}
pub enum LineClass { Hard, Soft, Notice }
pub struct Line { pub class: LineClass, pub text_id: &'static str, pub text: String, pub a: Who, pub b: Who }
pub enum Verdict { Refuse { lines: Vec<Line> }, Proceed { warnings: Vec<Line> } }
pub fn evaluate(input: &Input) -> Verdict;
pub fn machine_line(lines: &[Line]) -> String;  // N10, serde_json::to_string (no spaces)
```

Binding and comparison rules `evaluate` implements (each has unit tests):

- **B1 (derived paths are no source, I-8).** Only a path the input carries
  (typed, or read from a kept artifact) becomes a `KeyCoin`/`Coin`. A path the
  toolkit derives from `--network`, a template default (`synthesize.rs:657`
  `default_origin_path`, `resolve_slots` with no `@N.path=`, restore's
  `derive_bip32_from_entropy`, convert `--template` without `--path`) or from a
  file's `chain` (`coldcard.rs:276-286`, the legacy top-level xpub path) is
  never a source.
- **B2 (a coin compares with its own key, M-9).** A `KeyCoin` is compared only
  with the class of the key that carries it. A `Coin` (no key) is compared only
  with the run. Neither is ever compared with a `Stated` network.
- **B3 (stated files own their keys, I-1).** A `Stated` source's keys are
  compared with its statement first. If any disagrees: TX-H4C only (one line per
  file; several disagreeing sections of one class merge, "its bip44, bip49 and
  bip84 keys are test-network keys (tpub)"), no compare with the run, no TX-S
  for coins inside it (N8). If all agree: the statement is compared with the run
  (TX-H4); its keys are not also compared with the run. A coin inside a file
  whose statement prints TX-H4 prints no TX-S when it agrees with its own key
  (the file's keys are "named on" that HARD line: reading A12); a coin that
  disagrees with its own key prints TX-S after the HARD lines (V33 when no HARD).
- **B4 (a coin agreeing with its key on a HARD line prints nothing).** As N8.
  A coin disagreeing with its own key AND the run: one TX-S key form.
- **B5 (merging, I-3).** One contradiction, one line:
  - HARD key/run: keys of one class that agree with each other and disagree
    with the run merge (plural `<who>`).
  - TX-S key form: contradictions with the same `<pclass>` and the same
    `<kclass>` merge into one line (plural per Spec A §3: "`keys @0 and @1's
    paths <path0> and <path1> are <pclass> paths`", "`keys @0 and @1 are
    <kclass> keys`"); paths listed in key order, repeated when equal.
  - TX-S run form: one line per coin class (Spec A §4.1 restore rule).
  - Merging never crosses text ids. `<pfx>` of a merged line: the prefixes in
    key order, distinct values joined with ", " (`(tpub)`, `(tpub, vpub)`) (M-15).
- **B6 (order and drops).** N8 order (HARD: H4C, H4, H4P, H2, H1, H3, H5; then
  SOFT; within an id by input order). Any HARD printed: every SOFT line drops its
  last (flag) sentence. TX-H2 printed: each TX-H1 drops its first remedy
  sentence. A key and an address of the run that disagree with each other: TX-H3
  drops `Rerun with <remedy>, or ` and TX-H1 drops its first remedy sentence;
  an address that agrees with the run prints no TX-H3. Several `--target-address`
  values: each address is its own source and prints its own TX-H3, in input
  order; addresses never merge, because §3 gives TX-H3 no plural (R1-M2).
- **B7 (N6/N7/N12).** No source at all, `reads_kept`, not `seed_only`: TX-N
  (`given == false`), TX-T (test net), nothing (`--network mainnet`). SOFT only
  + `accept_coin_type`: Proceed with one TX-S-OK per overridden (merged)
  contradiction; the flag with nothing to override: no line. `secret_out` on a
  test run: TX-R appended as the LAST warning (M-7: the helper returns it, the
  hook prints it).
- Network names (Spec A §1): `mainnet`, `signet`, `testnet`, `regtest`; a
  class-derived `<net>` with no flag is "mainnet" / "a test network". `<P>` is
  `error:`; warnings start `warning:`.

### 2.2 Error variant and printing

- `error.rs`: `ToolkitError::NetworkRule { lines: Vec<network_rule::Line> }`,
  alphabetical after `NetworkMismatch`, in the enum and in `exit_code` (8),
  `kind` ("NetworkRule"), `message` (lines joined `\n`, first line's `error: `
  stripped), `details` (None). `NetworkMismatch` (2) and
  `ImportWalletNetworkClassMismatch` (1) stay until 8b deletes them.
- `cli.rs::run` (`match result`, cli.rs:255-268): a new arm before the generic
  `Err(e)` for `NetworkRule { lines }` that stores the lines, prints nothing yet,
  code 8. `finish_exit` (cli.rs:277-287) runs `mlock::report_at_exit()`, which
  can print stderr lines; to keep "the last on stderr" (N10) and "the last line
  of the run" (N12), the refusal lines, then the machine line (only when
  `MNEMONIC_NETWORK_JSON` is `1`), and TX-R on success, print AFTER
  `report_at_exit` (M-5): `finish_exit` takes a `trailer: Vec<String>`.
- Stdout is the unbuffered `OutSink`: each arm calls `evaluate` before its first
  stdout or file write. Every exit-8 test asserts stdout is empty.

### 2.3 Warnings on success paths

`Verdict::Proceed { warnings }`: the verb prints TX-S-OK/TX-N/TX-T on stderr at
once (before its output). TX-R is held and printed by the trailer (§5).

### 2.4 Given or default

Verbs with `Option<CliNetwork>` know whether `--network` was typed. Six verbs
carry a clap default: `export_wallet.rs:294-295`, `path_of_xpub.rs:83-84`,
`account_of_descriptor.rs:95-96`, `passphrase_of_xpub.rs:114-115`,
`silent_payment.rs:53-54`, `nostr.rs:96-97`. Each gets `#[arg(skip)] pub
network_given: bool`. `cli.rs` parses with `Cli::command().try_get_matches()` +
`Cli::from_arg_matches` (same error handling as `try_parse`), walks to the leaf
subcommand's `ArgMatches` (xpub-search is nested) and sets `network_given =
value_source("network") == Some(ValueSource::CommandLine)`. No Args struct is
built by a struct literal (R0 checked), and the skip field is invisible to
clap, `gen-man` and `gui-schema` (the `cli_gui_schema*` tests must stay
byte-identical). `bundle.rs:23-24` and `verify_bundle.rs:25-26` require
`--network`.

### 2.5 Vectors

- `crates/mnemonic-toolkit/tests/data/network_rule_vectors.tsv`: byte copy of
  `design/network-rule/vectors.tsv` (header + 61 rows; identical at 36c1f155 and
  d72a18b1).
- `nr0_vectors_tsv_hash_pinned`: sha256 of the copy == sha256 of the design file
  == a hard-coded constant.
- `nr0_vectors_tsv_every_row`: rows with `tool` in {any, mnemonic} (V01-V25,
  V28-V33, V42-V46, V60; 37 rows) through `evaluate`, with the SAME source model
  the arms use: `key=c` -> `Source::Key{who:"key @i", pfx: xpub|tpub, coins: []}`;
  `coin=c` directly after a `key=` -> a `KeyCoin{OwnKey}` on that key; `coin=`
  with no key before it -> `Source::Coin{Named("key @j")}` (raw-pubkey shape);
  `path=other` -> a key path of no N1 shape (`m/45h`), no coin; `hrp=`/`b58=` ->
  `Address`; `wif=` -> `Wif`; `stated=<n>` -> `Stated` Exact grain with no keys;
  `stated2=c` -> `Stated` Class grain (`chain BTC (mainnet)` / `chain XTN (a
  test network)`) and every `key=`/`coin=` AFTER it are its `SectionKey`s
  (`section` `bip84`, `who` `bip84`); `reads=kept` -> `reads_kept`;
  `reads=seed` -> `seed_only`; `out=secret` -> `secret_out`. Assert `class`
  (hard / hard+soft / soft incl. TX-S-OK / no-source for TX-N/TX-T / none), the
  text ids in print order, and exit (8 iff any Refuse). These conventions are
  this plan's reading of §7 (A7).
- `nr0_exit_code_8_is_network_mismatch`, `nr0_machine_line_shape` (the §5
  R1.4 example byte-exact; R1.14: two contradictions, top `"class":"hard"`; no
  line without the variable).
- Unit tests in `network_rule.rs`: every §3 template with fixed fields, each
  expected string a literal copied from Spec A §3 (M-16).

## 3. Arms, per unit

Common: build `Input`, `evaluate`, `Refuse` -> `Err(NetworkRule{lines})`,
`Proceed` -> print warnings, go on. `#[arg(long = "accept-coin-type-mismatch")]
pub accept_coin_type_mismatch: bool` goes on the verbs in §4.1. New test files
`tests/cli_network_rule_<verb>.rs`, each listed in `tests/all.rs` with a
`#[path]` line (`every_test_file_is_listed`, all.rs:267, fails otherwise). TRACE
test fn names are the spec's, exactly. Gate variables `$X $T $P $T1 $T2 $X1 $X2
$X1c1` copied from `scripts/spec-network-rule-gate.sh:72-80` as consts;
`mdargs`/`cos` reimplemented over the fixture files.

Shared helpers:
- `descriptor_sources(raw, &Descriptor<DescriptorPublicKey>) -> Vec<Source>`:
  keys in BIP-388 keys_info order (first appearance, deduplicated), `who = "key @i"`,
  `pfx` from the RAW key token as typed (A2), the key's origin coin as
  `KeyCoin{OwnKey}` (B1: only an origin present in the input). A `Single` key
  yields only a `Coin{Named("key @i")}` when its origin has an N1 coin.
- `envelope_source(path_as_given, &envelope) -> Source::Stated`: statement =
  `bundle.network` (Exact), `device = Signer`, keys = the envelope descriptor's
  keys as `SectionKey{section:"bundle.descriptor", who:"key @i", coins}`. Used by
  export-wallet `--from-import-json` AND bundle `--import-json` (I-7).
- `mk1_key_source(who, &mk1)`: version class, `pfx` from the decoded xpub's
  prefix, the card's own origin coin as `KeyCoin{OwnKey}`.

### 3.0 Literal expected strings (the spec-quoted legs)

Tests assert these byte-for-byte (one line each; `\` marks no line break).
They are built from Spec A §3 by hand; a reviewer checks each against §3.

- R1.13: `error: network mismatch: env_t.json says network testnet, but --network is mainnet. Nothing was written. Rerun with --network testnet, or use the file made for mainnet.`
- R1.19: `error: network mismatch (coin type): key @0's path m/84h/1h/0h is a test-network path (coin type 1h), but key @0 is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time.`
- R1.20 line 1: `error: network mismatch: --slot @0.xpub is a mainnet key (xpub), but --slot @0.master_xpub is a test-network key (tpub). Nothing was written. Every key of one wallet is for one network: export the odd key again.`
- R1.20 line 2: `error: network mismatch: --slot @0.master_xpub is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a mainnet wallet, export the key again with the signer set to mainnet.`
- R1.21: `error: network mismatch: env_xt.json says testnet, but its bundle.descriptor key is a mainnet key (xpub). Nothing was written. No --network repairs a file that contradicts itself: export the file again from the signer with its network set for the wallet's network.`
- R2.4 (the only line): `error: network mismatch: --cosigner @0's and @1's mk1 cards are test-network keys (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.`
- R3.1 / R3.5: `error: network mismatch: cc_BTC_T_1.json says chain BTC (mainnet), but its bip84 key is a test-network key (tpub). Nothing was written. No --network repairs a file that contradicts itself: export the file again from the Coldcard with its network set for the wallet's network.`
- R3.2: as R3.1 with `cc_BTC_T_0.json`, the only stderr line before the machine line.
- R3.6 line 1: `error: network mismatch: jade keys @0, @1 and @2 are test-network keys (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.`
- R3.6 line 2: `error: network mismatch (coin type): jade keys @0, @1 and @2's paths m/48h/0h/0h/2h, m/48h/0h/0h/2h and m/48h/0h/0h/2h are mainnet paths (coin type 0h), but jade keys @0, @1 and @2 are test-network keys (tpub). Nothing was written. If the path is a mistake, export the key again at the test-network path (coin type 1h).`
- R8a.7: `error: network mismatch: --target-xpub is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.`
- R8a.8: `error: network mismatch: --target-address tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl is a signet or testnet address, but this run is on mainnet (no --network given). Nothing was written. Give an address of mainnet.`
- R8a.11: `error: network mismatch (coin type): the path m/84h/0h/0h given with --slot @0.path is a mainnet path (coin type 0h), but --slot @0.xpub is a test-network key (tpub). Nothing was written. If the path is a mistake, export the key again at the test-network path (coin type 1h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for signet and this warning is printed every time.`
- R8a.12: `error: network mismatch (coin type): the path m/84h/0h/0h given with --slot @0.path is a mainnet path (coin type 0h), but --network is signet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. To use this path on signet as it is, add --accept-coin-type-mismatch: the output is then for signet and this warning is printed every time.`
- R8a.24: `error: network mismatch (coin type): --from mk1's path m/84h/1h/0h is a test-network path (coin type 1h), but --from mk1 is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time.`
- verify-bundle (fixture `vb_signet_bip84.txt`, `--network mainnet`): `error: network mismatch: the mk1 card is a test-network key (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.`

Open wording questions for the spec owner (each also in §7): A13 (the TX-S
key form where the path's `<who>` and the key's `<who>` differ, restore's md1
slot against a cosigner card), A14 (plural "export the key again" stays
singular in R2.4/R3.6 above, because §3's plural rules name only the `<who>`
and the verb).

### Step 0 (before unit 0): the test census

`design/network-rule/test-census-tk.md` (committed with this plan) lists the
90 integration-test functions that carry test-network material (r2: 12
helper functions that r1 listed as tests now sit in their own section, each
with the tests that call it; R1-M6), the 5 with a
mainnet key on a test `--network`, and the src unit tests that pin today's
refusals, each with its flag class and the expected outcome. At every unit, the
implementer runs the touched test files once (captured to a file) and checks
that every newly red test is on the census, or is explained by a spec row and
then added to the census with that row. A flipped test is changed to the spec's
exit and literal text; it is never deleted, skipped or loosened. A red test that
neither is listed nor is explained by a spec row is a regression and is fixed in
the code.

### Unit 1: export-wallet (Spec A unit 1; R1.1-R1.21)

- `--descriptor` arm: `export_wallet.rs:581`, after the lenient parse (`:621`):
  `descriptor_sources`, `thing = Descriptor`, `reads_kept = true`. Check before
  `emit_payload` (`:830`). Every `--format`.
- `--template/--slot` arm: replaces the loop `:775-793`. Per slot: `Key{who:
  "--slot @N.xpub"}`, `Key{who:"--slot @N.master_xpub"}` (R1.20), and a typed
  `@N.path=` as `KeyCoin{Flag("--slot @N.path")}` on the slot's xpub (R8a.11,
  key form) or, for a seed slot, `Coin{Flag("--slot @N.path")}` (R8a.12, run
  form). No typed path: no coin (B1; R1.12 prints exactly one line, a regression
  leg pins it). `pfx` from the `--slot` value as typed.
- `--from-import-json` (`:853`): replaces `:923-931` with `envelope_source`
  (I-1: the keys are inside `Stated`, never also top-level). Run =
  `args.network` + `network_given`; the envelope no longer decides (R1.13,
  R1.15). R1.21: TX-H4C only.
- Tests that flip: see the census; named in the spec: `cli_network_fail_open.rs`
  `e4d_descriptor_concrete_passthrough_ok` (:493) and the E4 legs,
  `cli_snet_convert_export_network_mismatch.rs` export legs,
  `cli_export_wallet_from_import_json.rs`.
- TRACE (Spec A §11 unit 1): `nr1_export_wallet_main_key_on_testnet_refused`,
  `nr1_export_wallet_test_key_on_mainnet_refused`,
  `nr1_export_wallet_test_key_no_flag_refused`,
  `nr1_export_wallet_coin_type_soft_refused_and_overridden`,
  `nr1_export_wallet_main_key_on_regtest_refused`,
  `nr1_export_wallet_consistent_controls_unchanged`,
  `nr1_export_wallet_mixed_keys_refused`,
  `nr1_export_wallet_slot_key_refused_exit_8` (+ the B1 one-line leg),
  `nr1_export_wallet_import_json_against_run_refused`,
  `nr1_export_wallet_raw_keys_no_source_tx_n_tx_t`,
  `nr1_export_wallet_raw_key_coin_type_soft`,
  `nr1_export_wallet_import_json_coin_type_soft`,
  `nr1_export_wallet_import_json_contradicts_itself`.
- `wallet_export/bsms.rs:133`: keep the guard (bsms is also reached from
  restore's `--format`, `restore.rs:3213` `build_import_payload`), but as an
  internal invariant, not a user-facing line (R1-M5): the emitter has no
  `RunNet.given`, raw `<pfx>` or `<who>`, so it cannot render TX-H1. It returns
  `ToolkitError::BadInput("internal: bsms export reached with a key of another
  network than the run; this is a bug")` (exit 1; no new variant, and no
  `debug_assert!`, so a unit test can reach it). Every caller (export-wallet's
  arms after unit 1, restore `--format` after unit 2) runs the rule first, so it
  is unreachable from the CLI; a unit test calls the emitter directly with a
  test key on a mainnet run and asserts that text.

### Unit 7: build-descriptor (Spec A unit 7; R7.1-R7.10; erratum E1)

- One check after the preset/spec lowers and the gate passes, before any output,
  in every mode: preset path between `build_descriptor.rs:212`
  (`emit_allow_notes`) and `:214` (`if args.emit_spec`); spec path at the same
  point after its gate; before `:243` and the human/`--format`/`--json` emits.
  Sources: `descriptor_sources` over the validated descriptor, `key @i` in the
  policy's BIP-388 keys_info order (Spec A unit 7; for presets `--key`, then
  `--recovery-key`, then `--final-key`) (A6).
- Delete `infer_descriptor_network_kind` (`:398-421`) and the warn block
  (`:429-460`); preview network = run network. `--json`: no `{diagnostics}`
  envelope on exit 8. `--emit-spec` help (`:100-101`): "is accepted and
  ignored" -> "is accepted and checked against the keys".
- Erratum E1: the banner at the top of
  `design/BRAINSTORM_cycle5_snet_network_invariant.md`; check its text is E1's,
  write it if not.
- Flip: `cli_snet_build_descriptor_network_warn.rs`
  `l1_tpub_with_network_mainnet_warns_not_rejects` (:45),
  `l1_tpub_network_omitted_infers_testnet_preview` (:65).
- TRACE: `nr7_build_descriptor_keys_against_run_refused`,
  `nr7_build_descriptor_mixed_keys_refused`, `nr7_build_descriptor_coin_type_soft`,
  `nr7_build_descriptor_consistent_unchanged` (+R7.10 leg),
  `nr7_build_descriptor_every_output_mode_refused`.

### Unit 2: restore `--md1` (Spec B unit 2; R2.1-R2.9)

- Keyed wallet-policy md1 -> `run_multisig` (`restore.rs:3882`). Pre-pass:
  parse every `--cosigner @N=` value (hoisted from `:4235-4290`) before the
  check. Check after step 4 builds `cosigners` (`:4023-4060`), before step 5
  (`:4073`). Sources (I-2):
  - slot N with a `--cosigner @N` card: ONE `Key{who:"--cosigner @N's mk1 card"}`
    (or `"--cosigner @N's xpub"`) whose `coins` are the md1 slot's origin coin
    as `KeyCoin{Named("key @N")}` and, for an mk1, the card's own origin coin as
    `KeyCoin{OwnKey}`. So R2.4 (tpub cards, coin-1 md1, no flag) prints only the
    merged TX-H1 (B4: both coins agree with their tpub key, named on the HARD
    line). R2.7 likewise (main keys, coin-0 md1, `--network signet`).
  - slot N with no cosigner: `Coin{Named("key @N")}` (run form; one line per
    coin class, B5).
  - `reads_kept = true`, `thing = Card`. `--from` does not exempt (R2.9).
- Keyless single-sig template (`run_singlesig_template_completion`, `:825`):
  the template carries no origin (the coin comes from `--network`, B1), so its
  only source is `--origin` (the 8a arm). `seed_only = true` (M-10).
- Keyless multisig template (`run_multisig_template_completion`, `:1540`):
  each `--cosigner` mk1 via `mk1_key_source` (version vs run: TX-H1; own coin
  vs own key: TX-S key form). `seed_only = true` when no cosigner is given.
- Fixtures (made, committed, recipes in `fx/README-owed-fixtures.md`):
  `fx/m_bip45.txt` (keyed md1 at m/45', no N1 coin) for
  `nr2_restore_md1_no_source_tx_n_tx_t`; `fx/m_signet_xcos.txt` (the two
  m_signet cosigner keys re-encoded as xpub, same key data) for the SOFT-only
  leg of `nr2_restore_md1_cosigner_version_refused_flag_ignored` (I-10b,
  FOLLOWUP as1-i4): `restore $(mdargs m_signet.txt) --cosigner @0=<l1>
  --cosigner @1=<l2> --count 1` today rc 0 "cross-checked"; after: exit 8, one
  merged TX-S key form (A13 wording); with the flag exit 0, TX-S-OK, stdout as
  today.
- TRACE: `nr2_restore_md1_coin_type_against_run_refused`,
  `nr2_restore_md1_cosigner_version_refused_flag_ignored`,
  `nr2_restore_md1_controls_unchanged`, `nr2_restore_md1_no_source_tx_n_tx_t`.

### Unit 3: import-wallet (Spec B unit 3; R3.1-R3.10; every parser per Spec A §4.1)

What each parser does after this unit (I-5). `p.network` is a placeholder
(`bitcoin::Network::Bitcoin`) until the central check sets it to the run network.

| parser | refusal today | after |
|---|---|---|
| bsms | `network_from_origins` (`bsms.rs:402-429`): cosigners' coin types differ; coin not in {0,1} | deleted (network inference); each key's coin -> `KeyCoin` (B1: a coin outside {0,1} is no source) |
| bsms | `coin_type_from_path` (`:431-450`): fewer than 2 components; coin step not hardened | deleted (only network inference uses it) |
| bsms | `assert_slots_network_agrees` (`:272`) | deleted; central check |
| bsms | first-address audit (`derive_first_address(&parsed, network)`, `:313`; WARNING on mismatch) | moved after the central check, run on the RUN network (R1-I1; below) |
| bitcoin-core | `network_from_origins` / `coin_type_from_path` (`bitcoin_core.rs:852-897`), same four refusals | deleted, as bsms |
| bitcoin-core | `assert_slots_network_agrees` (`:722`) | deleted |
| descriptor | `network_from_origins` / `coin_type_from_path` (`descriptor.rs:177-222`) | deleted, as bsms |
| descriptor | `assert_slots_network_agrees` (`:105`) | deleted |
| sparrow | `network_from_origins` / `coin_type_from_path` (`sparrow.rs:598-641`) | deleted |
| sparrow | `assert_slots_network_agrees` (`:452`) | deleted |
| specter | `network_from_origins` / `coin_type_from_path` (`specter.rs:377-420`) | deleted |
| specter | `assert_slots_network_agrees` (`:267`) | deleted |
| electrum | cosigners' coin types differ / coin not in {0,1} (`electrum.rs:786-805`) | deleted; each key's coin -> `KeyCoin` |
| electrum | cosigner coin step not hardened (`:934-945`) | deleted with the inference |
| electrum | `network_from_xpub_neutral` (`:996`) | kept as the key's class (a key source), not a refusal |
| electrum | `assert_slots_network_agrees` (`:412`) | deleted |
| electrum | imported-addresses wallet refused (`:352`) | kept (not a network question) |
| coldcard-multisig | `network_from_path` / `coin_type_index` (`coldcard_multisig.rs:803-835`) and heterogeneity (`:549-566`) | deleted; BIP-45 branch (`:532-547`) goes too (the run decides) |
| coldcard-multisig | `assert_slots_network_agrees` (`:612`) | deleted |
| coldcard (generic JSON) | `chain` must be BTC or XTN (sniff `:155-160`, parse `:216-224`) | accepts XRT too; any other value stays refused (shape) |
| every parser | descriptor/format shape, missing fields, xprv forbidden, watch-only | kept |

- bsms first-address audit (R1-I1). The audit at `bsms.rs:280-350` derives the
  first address on the parser's inferred network; with the inference deleted
  it would run on the placeholder and warn falsely on every correct test blob.
  So it moves out of `BsmsParser::parse` into
  `wallet_import::bsms::first_address_audit(p: &ParsedImport, run:
  bitcoin::Network, stderr)`, called by `cmd/import_wallet.rs` right after the
  central check passes for that `ParsedImport` (and so after `p.network` is the
  run network), for every shape whose provenance carries a declared address
  (`p.bsms_audit()`: the 6-line shape and the 4-line shape). The `Parser` trait
  signature does not change. Its warning and notice texts, the taproot skip and
  the `/**` expansion move unchanged. A refused run never reaches it (the
  refusal prints before any audit line). New tests:
  `nr3_bsms_first_address_audit_uses_run_network` (a 6-line test-network blob
  built with the file's `build_bsms_6line` helper from tpubs at
  `m/48'/1'/0'/2'`, its declared address the `tb1q...` the toolkit derives on
  testnet; `--network testnet`: exit 0, stderr has no "first-address
  mismatch"), and the existing `bsms_first_address_mismatch_warning` (mainnet)
  stays green unchanged. The declared first address is itself an address HRP,
  an N1 HARD source that Spec A §4.1's import-wallet row does not list; it is
  NOT made a source in this PR (N4: the §4.1 table binds) and is filed as a
  FOLLOWUP for the spec owner beside A16 (A18).
- Every parser fills new `ParsedImport` fields (`wallet_import/mod.rs:315`):
  `key_sources: Vec<Source>` (who per format: `jade key @N`,
  `coldcard-multisig key @N` in the file's key-line order; `key @i` for
  descriptor-bearing formats, counted within each `ParsedImport`, A4) and
  `stated: Option<Source::Stated>` (Coldcard generic JSON only; sections ordered
  by name, `bip44 < bip48_1 < bip48_2 < bip49 < bip84 < bip86 < xpub`, M-6).
- `cmd/import_wallet.rs`: one check per `ParsedImport` after the parse dispatch
  (`import_wallet.rs:1196-1212`) and BEFORE `--select-descriptor` filtering
  (every descriptor of a Core blob is checked; M-15), replacing the override
  block `:1214-1245` (`ImportWalletNetworkClassMismatch`). Run = `args.network`
  or mainnet (D1). Lines of several `ParsedImport`s concatenate in blob order.
  After the check, every `p.network` = the run network.
- Coldcard: chain -> `Stated` Class grain, `chain XRT (a test network)` new;
  `<device>` Coldcard; the `:276` coin mapping gains XRT -> 1 (a derived path,
  B1). Doc comments `:12`, `:32`, `:37`, `:67`, `:135`, `:184` updated.
- Jade (`jade.rs:103`) delegates to coldcard-multisig with a `who_prefix`.
- Flip (census): `cli_import_wallet_snet_network_mismatch.rs`,
  `cli_import_wallet_network_override.rs` (`testnet_blob_default_network_is_testnet`
  :44 refuses; `*_refused` exit 1 -> 8; signet/regtest overrides stay 0),
  `cli_import_wallet_bitcoin_core.rs`, `cli_import_wallet_bsms.rs`, parser unit
  tests that assert the deleted refusals (they become positive parse tests).
- TRACE: `nr3_coldcard_chain_against_key_refused`,
  `nr3_coldcard_consistent_unchanged`, `nr3_jade_names_jade_and_exits_8`,
  `nr3_import_test_blob_no_flag_refused`. Plan test (no TRACE row):
  `nr3_bsms_first_address_audit_uses_run_network`.
- Coldcard sections the parser does not import (R1-M7). The parser imports
  only the dominant section (`coldcard.rs:430-431`; `bip48_*` ignored). Every
  section's key still counts for TX-H4 and TX-H4C (the file's statement and its
  keys: a file that contradicts itself is refused whichever section is
  imported). The key-form TX-S (a section's path coin against that section's
  key) is evaluated only for the imported section, since no other section's
  path reaches the output. Reading A17.

### Unit 8a: the toolkit remainder (Spec B 8a; R8a.1-R8a.24)

| arm | site (d72a18b1) | sources / who | TRACE |
|---|---|---|---|
| bundle `--descriptor` | `bundle.rs:286`/`:348` descriptor arm, before the first card | `descriptor_sources` | `nr8a_bundle_descriptor_refused` |
| bundle `--slot @N.wif=` | `resolve_slots` (`bundle.rs:539`; WIF kind `slot_input.rs:49`) | `Wif` | `nr8a_bundle_slot_wif_refused` (+ legs signet, regtest) |
| bundle/export `--slot @N.path=` | `resolve_slots` | `KeyCoin{Flag("--slot @N.path")}` on the xpub, or `Coin{Flag(..)}` for a seed slot (I-9) | `nr8a_slot_path_coin_type_soft` (R8a.11, R8a.12 literals) |
| bundle `--import-json` | `bundle.rs:2116-2137` (replaces the exact-name compare) | `envelope_source` (TX-H4, TX-H4C, TX-S) | legs in `nr8b_existing_refusals_exit_8_one_wording` (TX-H4) and new plan test `nr8a_bundle_import_json_contradicts_itself` (env_xt.json TX-H4C; env_x_c1.json TX-S) (I-7) |
| convert key sources | `convert.rs` after the primary value is read (`:945`) and before compute (`:1085`) | `Key{who:"--from xpub"|"--from xprv"|"--from mk1"}`, mk1 own coin | `nr8a_convert_every_key_source_refused`, `nr8a_convert_xpub_to_xpub_refused`, `nr8a_mk1_own_coin_type_soft` |
| convert seed + `--path` | same point | `Coin{Flag("--path")}` | `nr8a_convert_path_coin_type_soft` (R8a.3, .4, .17) |
| restore `--origin` single-sig (**step 8a-0**, below) | `restore.rs` single-sig `run`, after `templates` (`:524-527`), before the derivation loop | `Coin{Flag("--origin")}` | `nr8a_restore_origin_coin_type_soft` |
| restore `--origin` template completion | `restore.rs:965` (`explicit_origin`), `:1578` | `Coin{Flag("--origin")}` | same TRACE, template leg |
| restore `--expect-xpub` | single-sig compare `restore.rs:607-624` and template-completion compare `:1044`; the check precedes both; `--allow-mismatch` does not pass it | `Key{who:"--expect-xpub"}` | `nr8a_restore_expect_xpub_network_first` |
| xpub-search path/passphrase-of-xpub | `path_of_xpub.rs`, `passphrase_of_xpub.rs`, before the search | `Key{who:"--target-xpub"}` | `nr8a_xpub_search_refuses_before_search` |
| xpub-search address-of-xpub | `address_of_xpub.rs:213-227` (replaced; no inference arm) | `Key{"--xpub"}`, one `Address{"--target-address"}` per value, one TX-H3 each (B6) | same |
| xpub-search account-of-descriptor | `account_of_descriptor.rs:232`, before intake/search | `descriptor_sources` (`key @0`), `--descriptor-from` too | `nr8a_xpub_search_account_of_descriptor_key_refused` |
| nostr `--pubkey` / `--secret*` | `nostr.rs:161`, secret branch `:202` | none; `reads_kept`, `thing = Key`; after the key advisory; never TX-R | `nr8a_nostr_pubkey_prints_tx_n`, `nr8a_nostr_secret_prints_tx_n` |
| derive-child | `derive_child.rs:230`, before any output | `Key{who:"--from xprv"}` | `nr8a_derive_child_tprv_no_flag_refused` |
| import-wallet coin flag | unit 3's check with the flag | `KeyCoin`s | `nr8a_import_wallet_coin_type_flag` (fixtures `d_x1.txt`, `d_mixcoin.txt` with `--format descriptor`: TX-S for the coin-1 key, exit 8; with the flag exit 0, TX-S-OK) |
| verify-bundle | `verify_bundle.rs::run`, after args are resolved (bundle-json load, positional HRP routing) and immediately before the first mode dispatch, `if !args.md1.is_empty()` at `:497` (R1-N1); one check covers every mode it then dispatches to (`verify_singlesig_template` :517, `verify_multisig_template` :536, `descriptor_mode_verify_run` :551, `run_multisig` :621, `run_full` :635, `run_watch_only` :644) | each `--mk1` card via `mk1_key_source` (`the mk1 card`, or `mk1 card @N` when several); each decoded md1 slot coin bound to the mk1 of that slot as `KeyCoin{Named("key @N")}` (I-2's binding), else `Coin`; a `--slot @N.xpub` key is a separate `Key{who:"--slot @N.xpub"}` and merges with the card of the same class (B5). Decoding here is best-effort (R1-M1): a card that does not decode (or needs auto-repair) is no source, and the mode then decodes, repairs and reports it exactly as today (RepairShortCircuit output unchanged); the check never refuses on a decode failure | `nr8a_verify_bundle_network_refused_exit_8` (fixture `vb_signet_bip84.txt`: `--network mainnet --template bip84 --slot @0.phrase=$P --mk1 .. --md1 ..`, today rc 4; after: exit 8, the literal in §3.0, stdout empty), `nr8a_verify_bundle_cause_line` (TX-V, R1-I2: see the condition below the table; legs `vb_mainnet_coin5.txt` prints TX-V, the signet `vb` leg without `--ms1` prints none; with the flag on a SOFT-only run: TX-S-OK, then the rows as today, and TX-V by the same condition) |
| TX-R | §5 | | `nr8a_tx_r_on_secret_output_test_run` (R8a.21; legs derive-child, bundle, silent-payment) |

**TX-V condition (R1-I2).** TX-V is printed after the rows, exit 4 unchanged,
when (a) the run was not refused (no HARD line, and no unaccepted SOFT line),
(b) at least one row failed, and (c) every failed row is one of the four
network-sensitive rows `mk1_xpub_match`, `mk1_path_match`, `md1_xpub_match`,
`md1_origin_match` (the only rows whose expected value depends on the run
network: the key version and the coin step of the derived path; checked
against every row name `verify_bundle.rs` emits at d72a18b1: `ms1_decode`,
`ms1_entropy_match`, `mk1_decode`, `mk1_fingerprint_match`,
`mk1_template_stub_bind`, `md1_decode`, `md1_wallet_policy`,
`md1_template_match`, `wallet_id_match`, `wallet_completed` are not, since a
master fingerprint, entropy, a policy shape or a decode do not change with the
network). Any other failed row suppresses TX-V. The check reads the
mode's row list once the mode has built it, and TX-V prints after the rows and
the result line. Legs, all with literal strings:
- `nr8a_verify_bundle_cause_line` positive: fixture `vb_mainnet_coin5.txt`
  (a mainnet bundle whose key is at `m/84'/5'/0'`; coin 5 is no source, B1,
  and the key is a mainnet xpub, so no source contradicts the run).
  `verify-bundle --network mainnet --template bip84 --slot @0.phrase=$P --ms1
  <ms1 of vb_signet_bip84.txt> --mk1 .. --md1 ..`: today rc 4 with exactly the
  four rows failing; after: the same rows, then `note: every mismatch above is
  a coin-type or key-version row: the bundle may be for another network than
  --network mainnet; rerun with the network the bundle was made for.`, exit 4.
- negative: `vb_signet_bip84.txt` with `--network signet` and no `--ms1`: only
  `ms1_decode` fails (re-run today: rc 4); after: the same rows, no TX-V, exit 4.
- The mainnet refusal leg (`nr8a_verify_bundle_network_refused_exit_8`) passes
  `--ms1` too, so its today rows are only the four network rows (re-run: rc 4,
  those four failing).

**Step 8a-0: single-sig restore honours `--origin` (pending Brian's ruling;
alternative: refuse `--origin` on single-sig restore).** Today the plain
single-sig arm ignores `--origin` (help `restore.rs:127-131`: "ignored
otherwise"; R0 measured `--origin m/84h/5h/0h` printing the `m/84'/1'/0'`
wallet). With the rule, an override would print TX-S-OK about a path the wallet
was not made at. So: when `--origin` is given without `--md1`, require exactly
one `--template` (as `--expect-xpub` does, `:409`) and derive that template's
account at the given origin path (the same derivation template completion uses
at `:965-1030`); the JSON/text show the path as written. Own commit, own test
`nr8a0_restore_single_sig_derives_at_origin` (phrase `$P`, `--template bip84
--origin m/84h/1h/5h --network testnet`: the printed origin is `84'/1'/5'` and
the key equals `convert --from phrase=$P --to xpub --path m/84h/1h/5h --network
testnet`). Help text and manual change with it.
Proposal for its neighbours (R1-M4):
- `--account` beside `--origin`: `--origin` already names the account step, so
  a `--account` typed on the command line (clap `value_source` is
  `CommandLine`; the default `0` does not count) beside `--origin` in
  single-sig restore is a `ModeViolation` (exit 2): `error: restore: --origin
  gives the whole path, account included; drop --account or drop --origin.`
  Template completion keeps today's behaviour (`--origin` wins silently,
  `restore.rs:965-988`); aligning it is filed as a FOLLOWUP, not changed here.
- `--expect-xpub` beside `--origin`: compared with the key derived at
  `--origin` (the key restore prints), after the network check as for every
  `--expect-xpub` (R8a.22, R8a.23).
- `--count`, `--json` and the text output are unchanged except that the
  origin shown is the given path.
Gating: step 8a-0 is the LAST code commit before the manuals (§6), so that
when Brian answers it is kept, dropped, or swapped for the alternative without
touching another commit. If Brian picks the alternative: `--origin` without
`--md1` is a `ModeViolation` (exit 2) and R8a.6's after leg becomes that
refusal; the coordinator owns the spec erratum. If he drops it: R8a.6's leg
stays as the spec writes it and the restore `--origin` single-sig row of the
table above goes with the commit.

### Unit 8b (toolkit part): existing refusals to exit 8 and one wording

- `addresses.rs:158-214`: the xpub arm refuses by the rule with or without
  `--network` (R8b.1, R8b.2). Help `:54-55` -> "Defaults to mainnet; an xpub
  of another class is refused".
- `convert.rs` `--xpub-prefix` (`:1212-1235`) and wif->xpub: TX-H1 / TX-H5.
- `synthesize.rs:645-655` and `:888-898` (`CosignerSpec`, exit 1),
  bundle `--slot @N.xpub=`: TX-H1.
- restore `--search-address` (`restore.rs:3171-3175`): TX-H3 (multisig template
  completion only).
- silent-payment (`silent_payment.rs:147-151`): TX-H1; TX-R when N12 holds.
- verify-bundle watch-only slot: TX-H1 (the unit-8a check covers it).
- `address_of_xpub.rs:62` help ("Defaults to network inferred from the xpub
  version") -> "Defaults to mainnet; an xpub of another class is refused" (M-12).
- Then delete `assert_network_agrees`, `network_kind_name`,
  `ToolkitError::NetworkMismatch`, `ImportWalletNetworkClassMismatch`,
  `pipeline::assert_slots_network_agrees` and their tests (network.rs tests,
  error.rs:1157); `cli_network_fail_open.rs::assert_mismatch` (:34) -> exit 8
  and the literal texts; `e1c_testnet_tpub_to_address_no_network_infers_ok`
  (:161) flips.
- TRACE: `nr8b_inferring_arms_refuse_without_flag` (R8b.1, R8b.4; R8b.5 is
  mk's, deferred), `nr8b_existing_refusals_exit_8_one_wording` (R8b.2 plus one
  leg per arm above and bundle `--import-json` TX-H4; R8b.3 md and R8b.6 mk
  deferred).

## 4. The flag, the manual and the GUI mirror

### 4.1 Verbs that gain `--accept-coin-type-mismatch`

export-wallet, restore, import-wallet, build-descriptor (units 1, 2, 3, 7);
convert, bundle, xpub-search account-of-descriptor, verify-bundle (8a). Not:
addresses, derive-child, nostr, silent-payment, path/passphrase/address-of-xpub
(no SOFT arm). A plain `bool`, `long = "accept-coin-type-mismatch"`, one
paragraph of doc (a blank line would flip `--help` layout): "Go on when an
origin path's coin type (0h or 1h) disagrees with its key or the network; the
output is for the run's network, every path is kept as written, and a warning
is printed each time."

### 4.2 Manuals

- `docs/manual/src/40-cli-reference/41-mnemonic.md`: the flag row in each
  verb's `### Flags` table (bundle `:370`, verify-bundle `:904`, convert
  `:1133`, export-wallet `:1187`, restore `:1369`, import-wallet `:1710`,
  build-descriptor, xpub-search account-of-descriptor). Row `8` with Spec A §8's
  sentence in every exit table the chapter has: import-wallet `:1771` (it has
  no class-mismatch row today: add 8, change nothing else; M-2), xpub-search
  path-of-xpub `:4205`, account-of-descriptor `:4323`, address-of-xpub `:4442`,
  passphrase-of-xpub `:4612`, and the chapter's other exit tables (`:2994`,
  `:3349`, `:3477`: add 8 only where that verb can refuse on the network).
  Changed help sentences (addresses, address-of-xpub `:4369` "inferred",
  build-descriptor `--emit-spec`, restore `--origin` per step 8a-0).
- `docs/manual-gui` (GUI manual lockstep hazard, M-12):
  `4g-xpub-search-address-of-xpub.md:31` ("default inferred from the xpub") and
  `4e-build-descriptor.md` (the inference/warning text).
- `make -C docs/manual lint` (flag-coverage) needs the flag documented on each
  verb; transcripts whose commands now refuse are regenerated.

### 4.3 mnemonic-gui (paired PR, separate repo)

Toolkit CLAUDE.md's mirror invariant: `mnemonic-gui/src/schema/mnemonic.rs`
gains a bool `--accept-coin-type-mismatch` `FlagSchema` on each verb in §4.1,
and `form::invocation::is_gui_managed_flag` (`invocation.rs:64`) returns true
for it, so the form never renders or emits it (Spec D §6.1: GUI-managed from
the first mirror commit). That commit also lands
`nr8f_schema_flag_gui_managed` (Spec A §12 item 2; M-13). `schema_mirror`
compares names against the PINNED binary, so it stays green until a pin bump.
MG-OVERRIDE and its dialog are 8f. One small PR on mnemonic-gui, opened after
this one, citing it.

## 5. TX-R hook

`network_rule::RunRecord` (thread-local `Cell`s, set by the verb):
`run_net`, `secret_input` (a seed, ms1, xprv/tprv or WIF was read),
`private_out` (set where the verb's worst stdout class is `PrivateKeyMaterial`:
`bundle.rs:1112`, convert's `worst_class_on_stdout` at `convert.rs:1287-1289`,
`derive_child.rs:316`/`:333`, `import_wallet.rs:1394`, `silent_payment.rs:366`).
After the verb returns Ok, `cli.rs` builds a sourceless `Input{run: <the
verb's RunNet>, sources: vec![], reads_kept: false, seed_only: true,
accept_coin_type: false, thing: <any; unused>, secret_out: secret_input &&
private_out}` (R1-M3: `reads_kept: false` and `seed_only: true` so B7 prints no
second TX-N/TX-T) and lets `evaluate` return TX-R alone (B7); the
trailer prints it after `report_at_exit` (§2.2). Verbs: bundle, convert,
derive-child, import-wallet, silent-payment. nostr never sets `secret_input`.

## 6. Commit sequence and gates

0. This plan, the census, the fixtures (done with r1).
1. unit 0. 2. unit 1. 3. unit 7. 4. unit 2. 5. unit 3. 6. unit 8a. 7. unit
   8b-toolkit + deletions. 8. step 8a-0 (the last code commit, gated on Brian's
   ruling: kept, dropped or swapped for the alternative refusal; R1-M4).
   9. manuals + help text. 10. FOLLOWUPS.
   Each code commit has its TRACE tests failing first, then passing.
- FOLLOWUPS (`design/FOLLOWUPS.md`): mark `status=resolved <sha>` ONLY for lines
  whose leg is built and green in this PR: as1-i2 (:6846), as1-i3 (:6847),
  as1-i4 (:6848, with the m_signet_xcos leg), as2-i1..i4 (:6850-6853),
  b-i1..i4 (:6854-6857; b-i4's mr half stays: close it only if the line scopes
  to the toolkit, else note "toolkit half resolved"). Leave as1-i1, as1-i5,
  c-i1 open (A10). File new lines: the md/mk 8b parts; the mnemonic-gui paired
  change; A12-A14 for the spec owner; bundle `--import-json` table gap (I-7);
  the bsms declared first address as an unlisted address source (A18, R1-I1);
  restore template completion's silent `--account` beside `--origin` (R1-M4).
- Gate per commit (4 cores; `cargo nextest` is NOT installed, so use
  `cargo test --locked -p mnemonic-toolkit --test all -- <names>` for the unit's
  rows, and the full suite once per unit captured to a file and grepped):
  `cargo test --locked --workspace`, `cargo test --locked --doc`,
  `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
  `RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps
  --document-private-items --all-features`,
  `SHIB=/home/user bash scripts/spec-network-rule-gate.sh --no-red` (the cites
  need the checkouts under `$SHIB`; N-2), `make -C docs/manual lint` when its
  binaries build here. Never `--release`.
- Before the PR: a whole-diff adversarial review
  (`design/agent-reports/whole-diff-network-rule-tk.md`).

## 7. Ambiguities (for the reviewer and Brian)

- A1 Typed or default `--network`: `#[arg(skip)] network_given` +
  `value_source`; switching to `Option` would change `gui-schema` defaults.
- A2 `<pfx>` as typed: read from the raw input token (miniscript and the
  SLIP-132 normalizer rewrite zpub/vpub).
- A3 Path-flag `<who>`: settled on Spec A §3 (I-9): "the path <p> given with
  <flag>" in both TX-S forms (R8a.11, R8a.12 literals in §3.0).
- A4 import-wallet formats other than Jade/coldcard-multisig/Coldcard: `<who>`
  `key @i` within each `ParsedImport`. Bitcoin Core blobs can carry a `chain`
  field (`bitcoin_core.rs:86`) and Sparrow/Specter exports a network; N1 lists
  none of them, so they are not sources here.
- A5 Unit 3 changes every import format (exit 2 -> 8; D1 run network; the
  coin-type inference refusals are gone, §3 unit 3 table). Spec A §4.1 "every
  parser" covers this; flagged because it is wider than Coldcard and Jade.
- A6 build-descriptor `key @i`: BIP-388 keys_info order of the validated policy.
- A7 Vector harness conventions (§2.5): coin binds to the key before it,
  `stated2`'s keys follow it, `path=other` is m/45h.
- A8 TX-N on export-wallet raw-pubkey descriptors on every format, also with
  `--output FILE` (R1.16).
- A9 `bundle --import-json` today refuses both an exact-name mismatch and a
  same-class one (BadInput, exit 1; `bundle.rs:2118-2137`). Under N3 it stays a
  refusal (TX-H4, exit 8): nothing widens (corrected per M-1).
- A10 `repo=mnemonic-toolkit` FOLLOWUPS that are mr's or about unit order
  (as1-i1, as1-i5, c-i1) stay open.
- A11 vectors.tsv hash pinned at d72a18b1 (same file at 36c1f155).
- A12 A coin inside a TX-H4 file that agrees with its own key prints no TX-S
  (the file's keys count as named on the TX-H4 line).
- A13 TX-S key form when the path and the key have different names (restore:
  md1 slot path, cosigner card key; verify-bundle: md1 slot, mk1 card). Proposal:
  `<P> network mismatch (coin type): key @0's path m/87h/1h/0h is a test-network
  path (coin type 1h), but --cosigner @0's mk1 card is a mainnet key (xpub).
  Nothing was written. If the path is a mistake, make the md1 card again at the
  mainnet path (coin type 0h). To use this key as it is, add ...`. me's "bundle
  form" and mr's "card form" are the nearest normative texts; the spec owner
  rules (FOLLOWUP).
- A14 Plural TX-H1 keeps "export the key again" singular (only `<who>` and the
  verb take the plural in §3).
- A15 Step 8a-0 (`--origin` on single-sig restore) is a behaviour change
  pending Brian's ruling; the alternative is a refusal (§3 8a).
- A16 `bundle --import-json` gains TX-H4C and TX-S (I-7), beyond Spec A §4.1's
  row; N4 binds. Filed as a table-gap FOLLOWUP for the spec owner.
- A17 Coldcard sections not imported: they count for TX-H4/TX-H4C, not for the
  key-form TX-S (unit 3; R1-M7). Spec owner to confirm.
- A18 A BSMS blob's declared first address is an address HRP the §4.1
  import-wallet row does not list as a source. Not a source here; the audit
  runs on the run network after the check (R1-I1). FOLLOWUP for the spec owner.
- A19 TX-V's "every mismatch above is a coin-type or key-version row" is
  read as the four rows named in unit 8a (R1-I2). Spec owner to confirm the
  row set.

## 8. Review folds

### R0 fold (review `design/agent-reports/network-rule-tk-plan-r0.md`)

| id | disposition |
|---|---|
| I-1 | folded: `SectionKey` defined; a stated file's keys live only inside `Stated` (B3); the vector harness and arms share the model (§2.1, §2.5, `envelope_source`); A12 recorded |
| I-2 | folded: a key carries `Vec<KeyCoin>`; the md1 slot coin binds to that slot's cosigner key (§3 unit 2); R2.4 prints TX-H1 only (literal §3.0); A13 for the wording |
| I-3 | folded: B5 merge rules (key-form TX-S by pclass+kclass, run form per coin class); R3.6 literal |
| I-4 | folded: step 8a-0, single-sig restore honours `--origin` (pending Brian; alternative refusal); site named; unit-2 template line corrected |
| I-5 | folded: per-parser table in unit 3; inference refusals deleted, shape refusals kept; `p.network` placeholder; I-5 leg via `fx/d_mixcoin.txt` |
| I-6 | folded: fixture `fx/vb_signet_bip84.txt` built and run (today rc 4); the site is one check in `verify_bundle.rs::run` before `:503` (r2: `:497`, R1-N1), covering all six modes; seed slot is no source, so the mk1 card is alone |
| I-7 | folded: `envelope_source` shared with bundle `--import-json` (TX-H4, TX-H4C, TX-S); new plan test; A16 |
| I-8 | folded: B1 (derived paths are no source); R1.12 one-line leg |
| I-9 | folded: A3 settled on Spec A §3; R8a.11/R8a.12 literals |
| I-10 | folded: census `design/network-rule/test-census-tk.md` + step 0 rule; as1-i4 fixture `fx/m_signet_xcos.txt` and its leg; FOLLOWUPS close only built legs |
| M-1 | folded (A9 corrected) |
| M-2 | folded (§4.2: no class-mismatch row; exit tables listed) |
| M-3 | folded (§5 cites) |
| M-4 | folded (both `--expect-xpub` compares) |
| M-5 | folded (§2.2 trailer after `report_at_exit`) |
| M-6 | folded (sections by name, B-order in unit 3) |
| M-7 | folded (types defined; `evaluate` returns TX-R) |
| M-8 | folded (module noted, re-export from `network.rs`) |
| M-9 | folded (B2) |
| M-10 | folded (`seed_only` on both template-completion arms) |
| M-11 | folded (bsms guard kept, moved to `NetworkRule`) |
| M-12 | folded (address-of-xpub help, manual :4369, `docs/manual-gui`) |
| M-13 | folded (§4.3) |
| M-14 | folded (§1: single PR by Brian's choice, noted) |
| M-15 | folded (B5 mixed prefixes, B6 several addresses, unit 3 per-`ParsedImport` count and check before `--select-descriptor`) |
| M-16 | folded (§3.0 literals; unit tests use literals) |
| N-1 | folded (cites corrected) |
| N-2 | folded (§6 `SHIB=/home/user`) |
| N-3 | folded (recipes spelled out in `fx/README-owed-fixtures.md`, fixture built) |
| N-4 | folded (§1 cites the #228 spec file and the #229 title) |

### R1 fold (review `design/agent-reports/network-rule-tk-plan-r1.md`)

| id | disposition |
|---|---|
| R1-I1 | folded: the bsms first-address audit moves out of `BsmsParser::parse` into `first_address_audit`, run by import-wallet after the central check on the run network (trait unchanged); test `nr3_bsms_first_address_audit_uses_run_network`; the declared address as an unlisted source is FOLLOWUP + A18 |
| R1-I2 | folded: TX-V's condition is by failed rows (at least one fails, every failed row is one of `mk1_xpub_match`, `mk1_path_match`, `md1_xpub_match`, `md1_origin_match`, and the run was not refused); positive leg fixture `fx/vb_mainnet_coin5.txt` (re-run today: rc 4, those four rows), negative leg the signet `vb` run without `--ms1` (only `ms1_decode` fails); the mainnet refusal leg passes `--ms1`; A19 |
| R1-M1 | folded: best-effort decode at the verify-bundle site; a card that does not decode is no source and the mode handles it as today |
| R1-M2 | folded: one TX-H3 per `--target-address`, input order, no plural (B6, unit 8a table) |
| R1-M3 | folded: §5 states every `Input` field of the post-run hook (`reads_kept: false`, `seed_only: true`, `sources: []`) |
| R1-M4 | folded: step 8a-0 is the last code commit before the manuals, gated on Brian; `--account` typed beside `--origin` on single-sig restore is a ModeViolation (exit 2), template completion unchanged with a FOLLOWUP; `--expect-xpub` compares with the key at `--origin` |
| R1-M5 | folded: the bsms emitter guard is an internal invariant (`BadInput("internal: ...")`, reachable only by a direct unit test), not a TX-H1 |
| R1-M6 | folded: census tidied; 12 helpers moved to their own section with their callers; the five empty flag classes filled (p25_controls TESTFLAG, xpub_to_address_p2pkh MAINKEY+TESTFLAG(testnet), e1c/general_multipath/l1_tpub_network_omitted NOFLAG); 90 tests, 5 mainnet-key-on-test-flag tests |
| R1-M7 | folded: non-imported Coldcard sections count for TX-H4/TX-H4C, not for the key-form TX-S; A17 |
| R1-N1 | folded: the verify-bundle dispatch cite is `:497` |

## 9. Implementation notes (implementer, 2026-11-28)

Readings taken where neither the plan nor the specs settle a point, and
deviations from the plan's text. Each is listed for the whole-diff review.

- **Clippy at unit 3.** Units 0-2/7 left two clippy errors in
  `network_rule.rs` (`too_many_arguments` on `texts::s_key`; items after the
  test module). Fixed in the unit-3 commit (an `allow` on `s_key`; the test
  module moved to the end of the file). No behaviour change.
- **Unit 3: deletions moved forward.** `ToolkitError::ImportWalletNetworkClassMismatch`
  and `pipeline::assert_slots_network_agrees` are deleted in unit 3, not 8b:
  import-wallet is their only caller and owns their arm (Spec A §11 unit 0:
  the old variants keep their codes "until the unit that owns their arm").
- **Unit 3: who builds the Jade `<who>`.** The plan has `jade.rs` delegate
  with a `who_prefix`; the code names the keys in `cmd/import_wallet.rs` from
  the resolved `--format` (`jade` -> `jade key @N`, `coldcard-multisig` ->
  `coldcard-multisig key @N`, else `key @i`). Same texts; the parsers stay
  free of rule texts.
- **Unit 3: `<file>` for a blob on stdin** (`--blob -`) is `the blob on
  stdin` (TX-H4/TX-H4C name a file "as given"; `-` names none).
- **Unit 3: Electrum's null-derivation path.** A cosigner with a null
  derivation gets a BIP-48 path synthesized from its own prefix (L18). By B1
  that path is derived, not carried; it is still passed to the rule, which is
  harmless because it agrees with its own key by construction.
- **Unit 3: RED check.** The TRACE tests were run against the pre-unit-3 src
  (src stashed, tests kept): `nr3_coldcard_chain_against_key_refused`,
  `nr3_jade_names_jade_and_exits_8`, `nr3_import_test_blob_no_flag_refused`
  and `nr8a_import_wallet_coin_type_flag` fail there; the control
  `nr3_coldcard_consistent_unchanged` and the plan test
  `nr3_bsms_first_address_audit_uses_run_network` pass there (the old parser
  inferred testnet from the coin type; the test guards the move of the audit
  after the placeholder network, which is the R1-I1 regression). R2-M2 is in
  the test: `build_bsms_6line` writes a checksummed descriptor line.
- **Unit 3: the 8a import-wallet flag leg** (`nr8a_import_wallet_coin_type_flag`)
  is written and committed with unit 3, since it exercises unit 3's check.
