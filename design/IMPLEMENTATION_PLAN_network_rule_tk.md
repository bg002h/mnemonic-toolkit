# IMPLEMENTATION PLAN: the network rule in mnemonic-toolkit (units 0, 1, 2, 3, 7, 8a, 8b-toolkit)

Status: plan r0 (author: Claude Opus 5.5, 2026-11-28). Awaiting R0 review.
Normative: `design/SPEC_network_rule.md` (Spec A: §1-§13, units 0, 1, 7) and
`design/SPEC_network_rule_tk_rest.md` (Spec B: units 2, 3, 8a, 8b). Where this
plan and a spec disagree, the spec wins and the plan is the defect. This plan
never restates a §3 text: the implementer copies each text verbatim from Spec A
§3 into one module (`network_rule::texts`) and tests pin them.

## 1. Baseline and base branch

- Baseline: `origin/master` **d72a18b1** (FOLLOWUPS: 32 owed-to-build lines, #198).
  Every `file:line` below is at d72a18b1 and was grep-checked when written.
- Branch: `claude/project-thread-cbnpjn`, based on master. One PR.
- Open PRs that touch the same files (none builds the rule):
  - #227 bug-hunt fixes (base master) and the stack on it: #228 exit-code bit
    flags, #229 `--scheme` rename (hidden `--template`/`--archetype` aliases),
    #230 address-of-seed. #228 keeps **8** for a network mismatch (its
    `SPEC_constellation_exit_codes.md` decision 1: bit 8 = REFUSED, network
    mismatch is a member, told apart by the N10 machine line), so this plan's
    exit 8 is compatible. #228 rewrites `error.rs::exit_code` wholesale and
    moves clap usage to 64; #229 renames flags the RED rows type (`--template`,
    `--archetype`), and its hidden aliases keep the RED commands valid.
  - Consequence: this PR rebases onto whichever of #227/#228/#229 merges
    first. Expected conflicts: `error.rs` (exit_code/kind/message match blocks),
    `cli.rs`, `export_wallet.rs`, `restore.rs`, `build_descriptor.rs`, the
    manual's 41-mnemonic.md. Keep every edit to those match blocks one line per
    variant, alphabetical (toolkit CLAUDE.md rule) so the rebase is mechanical.
- Out of scope (deferred, later PRs after this one is reviewed): md and mk parts
  of 8b (`md-cli` parse/keys.rs, decompose, seat; `mk-cli` address.rs; their
  unit-0 parts) and units 4, 5, 6, 8c-8h. mnemonic-gui's GUI-managed
  pieces (Spec D §6.1, unit 8f) except the minimal schema-mirror entry (§4.3).

## 2. Unit 0: the rule as code

### 2.1 Module

New `crates/mnemonic-toolkit/src/network_rule.rs` (pub, so
`tests/cli_network_rule_vectors.rs` and unit tests reach it through the lib;
`lib.rs` declares `pub mod network_rule;`). `network.rs` keeps `CliNetwork`;
`assert_network_agrees` and `network_kind_name` are deleted at the end of 8b
when no caller is left (Spec A unit 0: "when no site is left, `NetworkMismatch`
is deleted").

```rust
pub const ACCEPT_COIN_TYPE_FLAG: &str = "--accept-coin-type-mismatch"; // N7, one constant
pub const EXIT_NETWORK_MISMATCH: u8 = 8;                               // N9
pub const MACHINE_ENV: &str = "MNEMONIC_NETWORK_JSON";                 // N10

/// The run network (N2). `given` drives the `<run>` text.
pub struct RunNet { pub net: CliNetwork, pub given: bool }

/// A source (N1) at the grain it can express (N3).
pub enum Grain {
    Class(Class),                 // Main | Test: version bytes, WIF, coin type, base58, CC chain, UR
    Hrp(Hrp),                     // Bc | Tb | Bcrt
    Exact(ExactNet),              // Main | Test(=testnet, testnet3) | Testnet4 | Signet | Regtest
}
pub enum SourceKind {
    /// Extended key. `pfx` = prefix AS TYPED (xpub/tpub/zpub/vpub/...), `who` per §3.
    Key { who: String, pfx: String, class: Class, path_coin: Option<CoinRead> },
    /// Origin coin type with no key version (raw pubkey, seed slot, --path, md1 slot).
    Coin { who: String, coin: CoinRead },   // who renders "<who>'s path <path>" or "the path <p> given with --path"
    Address { who: String, address: String, grain: Grain },  // TX-H3; Hrp or Class(b58)
    Wif { class: Class, version: u8 },                       // TX-H5 (<n> 128 / 239)
    Stated { file: String, stated: StatedText, grain: Grain, keys: Vec<SectionKey> }, // TX-H4 / TX-H4C
}
/// Coin read from an origin path ONLY when N1 SOFT's shape holds
/// (purpose in {44,48,49,84,86,87,352,270028}, coin in {0,1}); else None (no source).
pub struct CoinRead { pub path_text: String /* m/84h/1h/0h, always h */, pub coin: u32 }
pub fn coin_of_path(path: &bitcoin::bip32::DerivationPath) -> Option<CoinRead>;

pub struct Input {
    pub run: RunNet,
    pub has_network_flag: bool,   // always true for the toolkit's verbs
    pub sources: Vec<SourceKind>, // in input order (N8 "by input order")
    pub reads_kept: bool,         // N6
    pub seed_only: bool,          // N6 exemption (a seed typed now, nothing kept)
    pub secret_out: bool,         // N12 (decided by the TX-R hook, §5; vector rows set it)
    pub accept_coin_type: bool,   // N7
    pub thing: Thing,             // TX-T <thing>: Card | Key | Descriptor
}
pub enum LineClass { Hard, Soft, Notice }
pub struct Line {
    pub class: LineClass,
    pub text_id: &'static str,    // "TX-H1".."TX-H5","TX-H4C","TX-S","TX-S-OK","TX-N","TX-T","TX-R"
    pub text: String,             // the whole rendered line, prefix included
    pub a: Who, pub b: Who,       // N10 a/b (what, network at the compared grain)
}
pub enum Verdict {
    Refuse { lines: Vec<Line> },       // exit 8; every line goes to stderr; nothing to stdout
    Proceed { warnings: Vec<Line> },   // TX-S-OK / TX-N / TX-T (TX-R comes from the hook)
}
pub fn evaluate(input: &Input) -> Verdict;
pub fn machine_line(lines: &[Line]) -> String;   // N10 compact JSON, serde_json::to_string
```

`evaluate` implements, in this order: N3 compare (class/HRP/exact at the
common grain; `--network testnet` equals both `test` and `testnet4`; a
class-grain source never contradicts a test chain), N4 HARD set, N8 merge
(one-class sources that agree with each other and disagree with the run are ONE
contradiction with the plural `<who>`), N8 suppressions (stated+keys agree but
disagree with run: only TX-H4; a self-contradicting file: only TX-H4C, no run
compare, no TX-S for its coins; coin agreeing with its own key's version when
that key is on a HARD line: no line; coin disagreeing with its own key and the
run: one TX-S key form), N8 order (HARD: H4C, H4, H4P, H2, H1, H3, H5; then
SOFT; within an id by input order), N8 drops (any HARD present: every SOFT line
drops its last sentence; TX-H2 present: each TX-H1 drops its first remedy
sentence; key+address of one run disagree: TX-H3's `Rerun with <remedy>, or `
drops and TX-H1 drops its first remedy sentence; address agreeing with the run:
no TX-H3), N7 (SOFT only + accept: Proceed with one TX-S-OK per overridden
contradiction; accept with nothing to override: no line), N6 (no source,
`reads_kept`, not `seed_only`: TX-N with `given=false`, TX-T with a test
network, nothing with `--network mainnet`).

Text rendering lives in `network_rule::texts`: one `fn` per text id taking typed
fields, plus `plural_who(&[String])` ("keys @0 and @1", "keys @0, @1 and @2";
the restore cosigner form "--cosigner @0's and @1's mk1 cards"; "jade keys @0,
@1 and @2"; TX-H4C "its bip44, bip49 and bip84 keys"). Network names: `mainnet`,
`signet`, `testnet`, `regtest` (Spec A §1); a no-flag class-derived `<net>` in
TX-S/TX-S-OK is "mainnet" / "a test network" (§3 placeholder note). The prefix
`<P>` is `error:` for every refusal line; warnings start `warning:`.

### 2.2 Error variant and printing

- `error.rs`: new variant `ToolkitError::NetworkRule { lines: Vec<network_rule::Line> }`,
  inserted alphabetically (after `NetworkMismatch`, before `NostrKeyParse`) in
  the enum and in every match block (`exit_code` => 8, `kind` =>
  "NetworkRule", `message` => lines joined with `\n` minus the leading
  `error: ` of the first, `details` => None). `NetworkMismatch` (2) and
  `ImportWalletNetworkClassMismatch` (1) stay untouched in unit 0 (Spec A unit
  0: "No refusal text or exit code of an existing arm changes in unit 0").
- `cli.rs` `run()` (cli.rs:257-267, the `match result`): add an arm before the
  generic `Err(e)` for `Err(ToolkitError::NetworkRule { lines })`: write each
  `line.text` to stderr, then, when `std::env::var(MACHINE_ENV) == Ok("1")`,
  `machine_line(&lines)` as the LAST stderr line; code = 8. Do not use the
  generic `writeln!("{}", e)` (it would add a second `error: `).
- Stdout is the unbuffered `OutSink` (out_sink.rs:1-60): a check that runs after
  the first `writeln!(stdout, ..)` cannot honour "nothing on stdout". Every arm
  below therefore calls `evaluate` before its first stdout/file write; tests
  assert `stdout.is_empty()` on every exit-8 leg.

### 2.3 Warnings on success paths

A verb that gets `Verdict::Proceed { warnings }` writes each `warning.text` to
its `stderr` immediately (before its own output, matching how the rows read
"stderr has TX-N"). N10's line prints only on exit 8 (never on Proceed).

### 2.4 "Given or default" (`<run>` text)

`<run>` needs to know whether `--network` was typed. Verbs with
`Option<CliNetwork>` (addresses, convert, derive-child, restore, import-wallet,
build-descriptor, address-of-xpub) know. Verbs with a clap default
(`export_wallet.rs:294-295`, `path_of_xpub.rs:83-84`,
`account_of_descriptor.rs:95-96`, `passphrase_of_xpub.rs:114-115`,
`silent_payment.rs:53-54`, `nostr.rs:96-97`) do not. Plan: keep the clap
default (so `gui-schema`'s `default_value` and the GUI's D33 suppression are
unchanged) and add `#[arg(skip)] pub network_given: bool` to those six Args
structs; `cli.rs` switches from `Cli::try_parse()` to
`Cli::command().try_get_matches()` + `Cli::from_arg_matches(&m)` (same error
handling), walks to the leaf subcommand's `ArgMatches`, and sets
`network_given = leaf.value_source("network") == Some(ValueSource::CommandLine)`.
`bundle.rs:23-24` and `verify_bundle.rs:25-26` require `--network` (always
given). The `#[arg(skip)]` field is invisible to clap, `gen-man` and
`gui-schema`; `cli_gui_schema*` tests must stay byte-identical (check).

### 2.5 Vectors

- Vendored copy `crates/mnemonic-toolkit/tests/data/network_rule_vectors.tsv`
  (byte copy of `design/network-rule/vectors.tsv`, 62 lines = header + 61 rows).
- `nr0_vectors_tsv_hash_pinned`: sha256 of the vendored file equals the sha256
  of `design/network-rule/vectors.tsv` read from the repo (path relative to
  `CARGO_MANIFEST_DIR/../../`) AND equals a hard-coded constant (so a spec
  change to the table fails the test until the copy and constant move).
- `nr0_vectors_tsv_every_row`: for every row with `tool` in {`any`,
  `mnemonic`} (V01-V25, V28-V33, V42-V46, V60), build an `Input` and assert
  `class`, `text` (ids in print order) and `exit`. Column mapping:
  `run_network` none -> `RunNet{Mainnet, given:false}`, else given;
  `override` -> `accept_coin_type`; `sources`: `key=main|test` -> `Key{who:"key @i"
  (i = running key index), pfx xpub|tpub}`; `coin=c` directly after a `key=` is
  that key's `path_coin` (`m/84h/<c>h/0h`), a `coin=` with no key before it is a
  `Coin{who:"key @j"}` raw-pubkey coin; `path=other` -> a key/coin path of no N1
  shape (`m/45h/0h`) = no source; `hrp=` -> `Address` with Hrp grain;
  `b58=` -> `Address` Class grain; `wif=` -> `Wif`; `stated=<exact>` ->
  `Stated` Exact grain; `stated2=main|test` -> `Stated` Class grain (a Coldcard
  `chain`) whose following `key=` entries are its section keys;
  `reads=kept` -> `reads_kept`; `reads=seed` -> `seed_only`; `out=secret` ->
  `secret_out` (the TX-R decision is the hook's, §5: the test calls the same
  `tx_r_due(run, secret_out, seed_input)` predicate). `class` column: hard if
  any Hard line, hard+soft if both, soft if only Soft (or TX-S-OK), no-source
  if TX-N/TX-T, none otherwise. The whole-text tests live in the arm units.
- `nr0_exit_code_8_is_network_mismatch`: `ToolkitError::NetworkRule{..}.exit_code() == 8`
  and `kind()`; `EXIT_NETWORK_MISMATCH == 8`.
- `nr0_machine_line_shape`: the §5 R1.4 example byte-exact, R1.14's two
  contradictions with top `"class":"hard"`; no line without the variable.
- Unit tests in `network_rule.rs` pin every §3 text with fixed fields (plural,
  drop rules, TX-S three forms, TX-H4C Coldcard and signer forms).

## 3. Arms, per unit

Common: "check" = build `Input`, call `evaluate`, on `Refuse` return
`Err(NetworkRule{lines})`, on `Proceed` print warnings, continue. Each arm adds
`#[arg(long = "accept-coin-type-mismatch")] pub accept_coin_type_mismatch: bool`
where §4 marks "flag" yes or SOFT only (list in §4.1 below). Tests go in new
files `tests/cli_network_rule_<verb>.rs`, each listed in `tests/all.rs` with a
`#[path]` line (all.rs aggregates the integration tests into one binary). Test
fn names are the spec's TRACE names exactly; each runs its RED rows' commands
(Spec A §11 gate variables `$X $T $P $T1 $T2 $X1 $X2 $X1c1` copied as consts,
`mdargs`/`cos` reimplemented in Rust over the fixture files, fixtures read from
`design/network-rule/fx/`), asserts exit, stdout (empty on 8), and the stderr
text by its full rendered string from `network_rule::texts`.

A shared helper `network_rule::descriptor_sources(raw: &str,
desc: &Descriptor<DescriptorPublicKey>) -> Vec<SourceKind>` yields keys in
BIP-388 `keys_info` order (first appearance, deduplicated), `who = "key @i"`,
`pfx` read from the RAW key token as typed (miniscript normalizes SLIP-132 and
loses it), `path_coin` from each key's origin; `Single` keys yield only a
`Coin` when their origin has an N1 coin. Used by export-wallet, bundle
`--descriptor`, build-descriptor, xpub-search account-of-descriptor,
import-wallet descriptor-shaped parsers.

### Unit 1: export-wallet (Spec A §11 unit 1; §4.1 rows 1-9, R1.1-R1.21)

- `--descriptor` arm: `export_wallet.rs:581` (`let canonical = if let Some(desc)`),
  after the lenient parse at `:621`. Sources: `descriptor_sources`. `thing =
  Descriptor`, `reads_kept = true`. Check before `emit_payload` (`:830`). Covers
  every `--format` (bsms's own guard `wallet_export/bsms.rs:133` becomes
  unreachable for this arm; delete it in 8b with the last `assert_network_agrees`).
- `--template/--slot` arm: replace the loop at `export_wallet.rs:775-793`.
  Sources: per slot, `Key{who:"--slot @N.xpub"}` and `Key{who:"--slot
  @N.master_xpub"}` (R1.20: TX-H2 between them, then TX-H1 for master_xpub),
  plus the slot's `@N.path=` coin (R8a.11 is 8a: TX-S key form). The raw `pfx`
  comes from the `--slot` value as typed (`args.slot`).
- `--from-import-json` (`run_from_import_json`, `:853`): replace `:923-931`.
  Sources: `Stated{file: <path as given>, stated: bundle.network, grain: Exact,
  keys: envelope keys "bundle.descriptor"}` plus each key `key @i` (index in
  `bundle.descriptor`) with its origin coin (R1.19). Run = `args.network` +
  `network_given` — the envelope no longer decides the run (R1.13, R1.15: today
  the envelope wins). Self-contradiction (R1.21) -> TX-H4C only,
  `<section>` = `bundle.descriptor`, `<device>` = `signer`. Then emit with the
  run network (they agree after the check).
- Tests that flip: `tests/cli_network_fail_open.rs::e4d_descriptor_concrete_passthrough_ok`
  (now a refusal) and the E4 `assert_mismatch` legs (exit 2 -> 8, new text);
  `tests/cli_snet_convert_export_network_mismatch.rs` export legs.
- TRACE: the 13 `nr1_*` names in Spec A §11 unit 1 (copy exactly).

### Unit 7: build-descriptor (Spec A §11 unit 7, R7.1-R7.10; erratum E1)

- One check after the preset/spec lowers and the gate passes, before any output,
  in every mode: preset path between `build_descriptor.rs:212`
  (`emit_allow_notes`) and `:214` (`if args.emit_spec`); spec path at the
  equivalent point after `read_spec`/gate. Sources: `descriptor_sources` over
  the validated descriptor; `key @i` in keys_info order (`--key`, then
  `--recovery-key`, then `--final-key`; a `--spec` tree in tree order — confirm
  the lowered descriptor's first-appearance order equals this; if it doesn't,
  that is an ambiguity, §7 A6).
- Delete `infer_descriptor_network_kind` (`:398-421`) and the warn block
  (`:429-460`); preview network = run network. `--json`: no `{diagnostics}`
  envelope on a network refusal. `--emit-spec` help (`:102`) "is accepted and
  ignored" -> "is accepted and checked against the keys".
- Erratum E1 banner at the top of `design/BRAINSTORM_cycle5_snet_network_invariant.md`
  (the spec says "written now"; its first lines already mention an erratum: check it reads as E1, add if not).
- Tests that flip: `tests/cli_snet_build_descriptor_network_warn.rs`
  `l1_tpub_with_network_mainnet_warns_not_rejects` (:45) and
  `l1_tpub_network_omitted_infers_testnet_preview` (:65) reverse to exit 8.
- TRACE: `nr7_build_descriptor_keys_against_run_refused`,
  `nr7_build_descriptor_mixed_keys_refused`, `nr7_build_descriptor_coin_type_soft`,
  `nr7_build_descriptor_consistent_unchanged`, `nr7_build_descriptor_every_output_mode_refused`
  (+ R7.10 control as a leg of `..._consistent_unchanged`).

### Unit 2: restore `--md1` (Spec B unit 2, R2.1-R2.9)

- Keyed wallet-policy md1 -> `run_multisig` (`restore.rs:3882`). Check after
  step 4 builds `cosigners` (`:4023-4060`) and before step 5/6 output. Sources:
  each md1 slot's origin coin as `Coin{who:"key @N"}` (key bytes carry no
  version; run form); each `--cosigner @N=<mk1>` decoded in a pre-pass (hoist
  the parse at `:4240-4275` before the check) as `Key{who:"--cosigner @N's mk1
  card"}` with `path_coin` = the mk1's own origin coin; `--cosigner @N=<xpub>`
  as `Key{who:"--cosigner @N's xpub"}`. Spec A §4.1 "Restore md1 rules": slot
  md1 coin vs a cosigner key's version -> TX-S key form for that slot; when the
  key also disagrees with the run, only TX-H1. `--from` does not exempt
  (R2.9). `reads_kept = true`, `thing = Card`.
- Keyless single-sig template (`run_singlesig_template_completion`, `:825`):
  template origin coin vs run (R2.9: SOFT run form even with `--from`).
  Keyless multisig template (`run_multisig_template_completion`, `:1540`): each
  `--cosigner` mk1: version vs run (TX-H1) and own coin vs own key (TX-S key
  form, 8a's `nr8a_mk1_own_coin_type_soft` restore leg).
- Fixture owed `design/network-rule/fx/m_bip45.txt`: a keyed md1 at `m/45'/...`
  (no N1 coin) for `nr2_restore_md1_no_source_tx_n_tx_t` (TX-N; TX-T with
  `--network signet`). Make it with the toolkit from the public phrases `$P` and
  "legal winner thank year wave sausage worth useful legal winner thank yellow":
  `mnemonic bundle --network mainnet --descriptor "wsh(sortedmulti(2,[fp/45h]xpubA/<0;1>/*,[fp/45h]xpubB/<0;1>/*))"`
  with the two xpubs from `mnemonic convert --to xpub --path m/45h`; keep only
  the md1 lines in the file (record the commands in `fx/README` or the test).
- TRACE: the four `nr2_*` names in Spec B.

### Unit 3: import-wallet Coldcard and Jade (Spec B unit 3, R3.1-R3.10)

- Parsers stop refusing on network: remove the `assert_slots_network_agrees`
  calls (`pipeline.rs:149` callers: `coldcard_multisig.rs:612`,
  `bitcoin_core.rs:722`, `bsms.rs:272`, `descriptor.rs:105`, `electrum.rs:412`,
  `sparrow.rs:452`, `specter.rs:267`) and the coin-heterogeneity refusal is
  kept only where it is not a network question. Each parser instead fills new
  `ParsedImport` fields (`wallet_import/mod.rs:315`): `stated:
  Option<(StatedText, Class, Vec<(section, pfx, Class)>)>` (Coldcard `chain`)
  and `key_sources: Vec<SourceKind>` (raw pfx, who, path coin).
- `cmd/import_wallet.rs`: one check after the parse dispatch
  (`import_wallet.rs:1196-1212`), replacing the override block
  `:1214-1245` (`ImportWalletNetworkClassMismatch`). Run = `args.network` or
  mainnet (D1; R3.9: a test blob with no flag refuses). After the check, set
  every `p.network` to the run network.
- Coldcard (`wallet_import/coldcard.rs`): accept `XRT` in sniff (`:155-160`)
  and parse (`:216-224`); chain -> `Stated` Class grain with `stated` text
  `chain BTC (mainnet)` / `chain XTN (a test network)` / `chain XRT (a test
  network)`; sections in file key order (`bip44`, `bip49`, `bip84`, ...,
  legacy top-level `xpub`), `<device>` = `Coldcard`. Update docs at
  `:12`, `:32`, `:37`, `:67`, `:135`, `:184`, the coin-type mapping `:276`
  (XRT -> coin 1).
- Jade (`jade.rs`) delegates to coldcard-multisig; who = `jade key @N` (the
  file's key-line order) for a Jade file, `coldcard-multisig key @N` for a
  Coldcard multisig file. Thread a `who_prefix` into the shared parser.
- Tests that flip: `tests/cli_import_wallet_snet_network_mismatch.rs` (exit
  2 -> 8, texts), `tests/cli_import_wallet_network_override.rs`
  (`testnet_blob_default_network_is_testnet` :44 becomes a refusal;
  `*_refused` legs exit 1 -> 8; `testnet_blob_override_to_signet`/`_regtest`
  stay 0).
- TRACE: the four `nr3_*` names in Spec B.

### Unit 8a: the toolkit remainder (Spec B 8a, R8a.1-R8a.24)

| arm | site (d72a18b1) | sources / who | TRACE |
|---|---|---|---|
| bundle `--descriptor` | `bundle.rs:286`/`:348` descriptor arm (check before the first card) | `descriptor_sources` | `nr8a_bundle_descriptor_refused` |
| bundle `--slot @N.wif=` | slot resolution, `bundle.rs:539` `resolve_slots` (WIF kind `slot_input.rs:49`) | `Wif` | `nr8a_bundle_slot_wif_refused` (+legs signet, regtest) |
| bundle/export `--slot @N.path=` | `resolve_slots` | `Coin` run form (seed slot) or key `path_coin` (xpub slot); who "the path <p> given with --slot @N.path"? see §7 A3 | `nr8a_slot_path_coin_type_soft` |
| convert key sources | `convert.rs` after primary decode (step 2, `:945`) and before compute (`:1085`) | `Key{who:"--from xpub"/"--from xprv"/"--from mk1"}`, mk1 own coin | `nr8a_convert_every_key_source_refused`, `nr8a_convert_xpub_to_xpub_refused`, `nr8a_mk1_own_coin_type_soft` |
| convert seed + `--path` | same point | `Coin{who:"the path <p> given with --path"}` | `nr8a_convert_path_coin_type_soft` (R8a.3, .4, .17) |
| restore `--origin` | `restore.rs:965` `explicit_origin` and single-sig `--origin` | Coin run form | `nr8a_restore_origin_coin_type_soft` |
| restore `--expect-xpub` | `restore.rs:409`/`:617`; check BEFORE the expect compare (`:626-629`); `--allow-mismatch` does not pass it | `Key{who:"--expect-xpub"}` | `nr8a_restore_expect_xpub_network_first` |
| xpub-search path/passphrase-of-xpub | `path_of_xpub.rs`, `passphrase_of_xpub.rs` before the search | `Key{who:"--target-xpub"}` | `nr8a_xpub_search_refuses_before_search` |
| xpub-search address-of-xpub | `address_of_xpub.rs:213-227` (replace; no inference arm any more) | `Key{"--xpub"}`, `Address{"--target-address"}` (N8 key+address rule) | same |
| xpub-search account-of-descriptor | `account_of_descriptor.rs:232` before intake/search | `descriptor_sources` (`key @0`), `--descriptor-from` too | `nr8a_xpub_search_account_of_descriptor_key_refused` |
| nostr `--pubkey` / `--secret*` | `nostr.rs:161` and the secret branch `:202` | none; `reads_kept=true`, `thing=Key`; TX-N/TX-T after the key advisory, never TX-R | `nr8a_nostr_pubkey_prints_tx_n`, `nr8a_nostr_secret_prints_tx_n` |
| derive-child | `derive_child.rs:230` (`emit_network`), check before any output | `Key{who:"--from xprv"}` | `nr8a_derive_child_tprv_no_flag_refused` |
| import-wallet coin flag | unit 3's check, now with the flag | key `path_coin` | `nr8a_import_wallet_coin_type_flag` |
| verify-bundle | `verify_bundle.rs` before the rows; mk1 version (TX-H1 `the mk1 card` / `mk1 card @N`) then md1 coin (TX-S run form); TX-V after mismatch rows when no source contradicts (exit 4 kept) | | `nr8a_verify_bundle_network_refused_exit_8`, `nr8a_verify_bundle_cause_line` |
| TX-R | §5 | | `nr8a_tx_r_on_secret_output_test_run` (R8a.21; legs derive-child, bundle, silent-payment) |

Fixtures owed by 8a (make from public data, commit under `design/network-rule/fx/`):
- a verify-bundle input whose mk1 is a test-network key checked under
  `--network mainnet`: build with `mnemonic bundle --network signet --template
  bip84 --slot @0.phrase=$P` is refused upstream ("slot count 0 out of
  range", Spec B note), so use `--slot @0.xpub=$T --slot @0.fingerprint=73c5da0a`
  on `--network signet` and verify its md1+mk1 with `--network mainnet`;
- an import-wallet descriptor blob with an xpub at coin 1 (`[73c5da0a/84h/1h/0h]$X`)
  for `nr8a_import_wallet_coin_type_flag`.

### Unit 8b (toolkit part only): existing refusals to exit 8 and one wording

- `addresses.rs:158-214`: the xpub arm refuses by the rule with or without
  `--network` (R8b.1, R8b.2); the help (`:54-55`) "Defaults to the xpub's
  version bytes ..." becomes "Defaults to mainnet; an xpub of another class is
  refused".
- `convert.rs` xpub->address (R8b.4) is covered by 8a's convert check;
  `--xpub-prefix` (`:1212-1235`) and wif->xpub move to `NetworkRule` (TX-H1 /
  TX-H5).
- `synthesize.rs:645-655` and `:888-898` (`CosignerSpec`, exit 1) for
  `bundle --slot @N.xpub=`: TX-H1, exit 8.
- bundle `--import-json` (`bundle.rs:2116-2137`): TX-H4.
- restore `--search-address` (`restore.rs:3171-3175`): TX-H3 (multisig template
  completion only).
- silent-payment (`silent_payment.rs:147-151`): TX-H1; TX-R when N12 holds.
- verify-bundle watch-only slot: TX-H1.
- `wallet_export/bsms.rs:133`: unreachable after unit 1; delete.
- Then delete `assert_network_agrees`, `network_kind_name`,
  `ToolkitError::NetworkMismatch` and `ImportWalletNetworkClassMismatch` and
  their tests (network.rs tests, error.rs:1157 test); update the
  `cli_network_fail_open.rs` `assert_mismatch` helper (`:34`) to exit 8 and the
  new texts; `e1c_testnet_tpub_to_address_no_network_infers_ok` (`:161`) flips.
- TRACE: `nr8b_inferring_arms_refuse_without_flag` (R8b.1, R8b.4; R8b.5 is
  mk's: deferred, note it in the test's doc), `nr8b_existing_refusals_exit_8_one_wording`
  (R8b.2 plus one leg per "stays" arm above; R8b.3 md and R8b.6 mk deferred).
  `nr8b_md_from_mk1_version_refused` is md's: deferred.

## 4. The flag, the manual and the GUI mirror

### 4.1 Verbs that gain `--accept-coin-type-mismatch`

export-wallet, restore, import-wallet, build-descriptor (units 1, 2, 3, 7);
convert, bundle, xpub-search account-of-descriptor, verify-bundle (8a).
(§4.1 flag column yes / SOFT only. Not: addresses, derive-child, nostr,
silent-payment, path/passphrase/address-of-xpub: no SOFT arm.) Clap: a plain
`bool` with `long = "accept-coin-type-mismatch"`, no value; one-paragraph doc
comment (cli.rs note: a blank line flips `--help` layout) — "Go on when an
origin path's coin type (0h or 1h) disagrees with the key or the network; the
output is for the run's network, paths are kept, and a warning is printed each
time."

### 4.2 Manual (`docs/manual/src/40-cli-reference/41-mnemonic.md`)

Per verb: the flag row in its `### Flags` table (export-wallet `:1187`,
restore `:1369`, import-wallet `:1710`, build-descriptor, convert `:1133`,
bundle `:370`, verify-bundle `:904`, xpub-search account-of-descriptor); each
verb's exit table gains row `8` with Spec A §8's sentence; the import-wallet
exit table (`:1771`) loses the exit-1 class-mismatch row; addresses' and
build-descriptor's changed help text. `make -C docs/manual lint` step 4
(flag-coverage) needs the flag documented; `cli-subcommands.list` lists
subcommands only (check). `transcripts/` that show changed stderr/exit:
regenerate the ones whose commands now refuse (grep transcripts for `tpub` on
mainnet runs).

### 4.3 mnemonic-gui (paired PR, separate repo)

The toolkit CLAUDE.md mirror invariant requires `mnemonic-gui/src/schema/mnemonic.rs`
to gain the flag on the same verbs. `schema_mirror` compares flag NAMES against
the PINNED binary, so it stays green until a pin bump. Minimal paired change:
add a `--accept-coin-type-mismatch` bool `FlagSchema` to each verb above, and
add it to `form::invocation::is_gui_managed_flag` (`invocation.rs:64`) so the
form never renders or emits it (Spec D §6.1 makes it GUI-managed from the first
mirror commit; MG-OVERRIDE/dialog are 8f). One small PR on mnemonic-gui,
opened after this one, citing it.

## 5. TX-R post-run hook

In `cli.rs::run` after the verb returns `Ok`, before `finish_exit`: if
`tx_r_due(run_net, stdout_class_was_private, input_had_secret)` print TX-R on
stderr as the last line. Data flow: a small `network_rule::RunRecord`
(thread-local `Cell`s, set by the verb): `run_net: Option<CliNetwork>`,
`secret_input: bool` (seed, ms1, xprv/tprv, WIF), `private_out: bool` (set
where the verb calls `emit_output_class_advisory(PrivateKeyMaterial, ..)`:
`bundle.rs:1112`, `convert.rs:1299`, `derive_child.rs:317`/`:334`,
`import_wallet.rs:1394`, `silent_payment.rs:367`). Verbs: bundle, convert,
derive-child, import-wallet, silent-payment. nostr never sets it (Spec B 8a).
Fires only on a test run network.

## 6. Commit sequence and gates

1. unit 0: RED `nr0_*` (helper missing) -> GREEN `network_rule.rs`, error
   variant, cli.rs printing and `network_given` plumbing, vendored tsv.
2. unit 1, 3. unit 7, 4. unit 2, 5. unit 3, 6. unit 8a (one commit per verb
   group is fine), 7. unit 8b toolkit + deletions, 8. manual + help text,
   9. FOLLOWUPS: close (status=resolved <sha>) the `netrule-r3-*` lines with
   `repo=mnemonic-toolkit` these units satisfy: as1-i2, as1-i3, as1-i4,
   as2-i1..i4, b-i1..i4 (design/FOLLOWUPS.md:6846-6857); leave as1-i1 (mr tip
   clause), as1-i5 and c-i1 (unit order: open until 8h). Add a FOLLOWUP for the
   deferred md/mk 8b parts and for the mnemonic-gui paired change.
   Each code commit: its TRACE tests first (fail), then code (pass), in one
   commit or a RED/GREEN pair.
- Gate at each commit (4 cores; nextest is NOT installed here, so
  `cargo test --locked -p mnemonic-toolkit --test all -- <names>` for the
  phase's rows, and the whole suite once at the end, captured to a file):
  `cargo test --locked --workspace`, `cargo test --locked --doc`,
  `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
  `RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps`,
  `bash scripts/spec-network-rule-gate.sh --no-red` (cites/ids/vectors still
  hold), `make -C docs/manual lint` if its binaries can be built here.
  Never `--release`.
- Before the PR: one whole-diff adversarial review
  (`design/agent-reports/whole-diff-network-rule-tk.md`).

## 7. Ambiguities and proposed resolutions

- A1 `<run>` given-vs-default on clap-default verbs (§2.4): resolved by
  `#[arg(skip)] network_given` + `value_source`. Alternative (switch to
  `Option<CliNetwork>`) changes `gui-schema` defaults; rejected.
- A2 `<pfx>` "as typed": miniscript and the toolkit's SLIP-132 normalizer
  rewrite zpub/vpub to xpub/tpub before parse. Proposal: read pfx from the raw
  input token (first 4 chars of the key string after the origin bracket).
- A3 `<who>` for `--slot @N.path=` and convert `--path`: §3 says the path flag
  source "is written `the path <path> given with --path` (--origin,
  --slot @N.path)". Proposal: `the path m/84h/0h/0h given with --slot @0.path`
  and `... given with --origin`, i.e. the flag name substituted; when the slot
  also has an xpub, the key form uses `--slot @0.xpub's path ...`? The spec's
  R8a.11 says "TX-S (key form)": proposal `<who>` = `--slot @0.xpub`, path
  text from `@0.path`. Reviewer to confirm.
- A4 import-wallet parsers other than Coldcard/Jade/coldcard-multisig (bsms,
  bitcoin-core, descriptor, electrum, sparrow, specter): §3 names no `<who>`.
  Proposal: `key @i` (descriptor key order) for descriptor-bearing formats.
  Their blobs' own network fields (Core's `listdescriptors` has none; Sparrow
  and Specter carry a network/chain in some exports) are NOT N1 sources in the
  spec; proposal: do not add them (a later spec PR may).
- A5 Removing every parser's network refusal (§3 unit 3) changes behaviour for
  bsms/bitcoin-core/electrum/sparrow/specter/descriptor blobs too (exit 2 ->
  8, D1 run network). Spec A §4.1 row "import-wallet, every parser" covers
  this; flagged because it is wider than "Coldcard and Jade".
- A6 build-descriptor `key @i` order for `--spec` trees: "tree order" vs the
  lowered descriptor's first-appearance order; proposal: use keys_info order of
  the policy the gate validated (the BIP-388 order), which is what §11 unit 7
  says ("the key's index in the policy's BIP-388 keys_info").
- A7 Vector harness conventions (coin= binds to the preceding key=;
  `stated2` keys follow it; `path=other` = m/45h) are this plan's reading of
  §7; the spec gives no per-source binding. Reviewer to confirm.
- A8 TX-N on export-wallet raw-pubkey descriptors prints on every format,
  including `--output FILE` (file written, TX-N on stderr). OK per R1.16.
- A9 `bundle --import-json` today compares exact names with a same-class
  allowance (`bundle.rs:2118-2137`: an envelope recorded `testnet` accepts
  `--network signet`); N3 says a stated exact name compares at exact grain, and
  `testnet` equals `test` and `testnet4` only. So signet against a `testnet`
  envelope becomes TX-H4. Proposal: follow N3 (refuse), note in the PR.
- A10 FOLLOWUP lines with `repo=mnemonic-toolkit` that are really mr's (as1-i1,
  c-i1): left open, not closed by this PR.
- A11 `nr0_vectors_tsv_hash_pinned` "at the spec's rev": the spec header says
  r2 fold, Spec B says "Spec A at 36c1f155". Proposal: pin the hash of the file
  at d72a18b1 (the merged spec).
