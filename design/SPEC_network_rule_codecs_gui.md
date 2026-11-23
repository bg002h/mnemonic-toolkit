# SPEC: one network-contradiction rule for every tool, section 6 and units 6, 8d, 8e, 8f, 8h

Normative rule, texts and vectors: SPEC_network_rule.md (Spec A) at 36c1f155; this spec adds units only and never changes §2/§3.
Spec map: A = SPEC_network_rule.md (§1-§10, §13, units 0, 1, 7); B = SPEC_network_rule_tk_rest.md (units 2, 3, 8a, 8b); C = SPEC_network_rule_mr.md (units 4, 5, 8c, 8g); D = SPEC_network_rule_codecs_gui.md (§6, units 6, 8d, 8e, 8f, 8h).
Units here: 6, 8d, 8e, 8f, 8h. Machine checks: `scripts/spec-network-rule-gate.sh` reads all four specs.

## 6. GUI and device design

### 6.1 mnemonic-gui
- It sets MNEMONIC_NETWORK_JSON=1 in runner::run_plan_with_env_prefix (the Run-button path) and in run_with_stdin, beside their MNEMONIC_FORCE_TTY calls ({{mnemonic-gui@922a0b383:src/runner.rs:191}} "MNEMONIC_FORCE_TTY", {{mnemonic-gui@922a0b383:src/runner.rs:283}} "MNEMONIC_FORCE_TTY"). The tree-form validate and --emit-spec callers (tree_form.rs) drop the machine line (the last stderr line when it starts with {"kind":"network_mismatch") before showing stderr in the strip; tree_form.rs validate_argv appends the form's --network value when one is chosen (the verdict now depends on it), and appends nothing when none is chosen; a validate that exits 8 shows MG-BANNER above the strip with the same Override rule.
- A run that ends with exit 8 shows a red banner over the output pane:
- **MG-BANNER**: `Network mismatch: nothing was produced.` with, under it,
  the CLI's first stderr line verbatim (flags included: D6, mnemonic-gui may
  show CLI flags, its form fields are named by them). The stderr pane stays as today except that the machine line (the last stderr line, when it starts with {"kind":"network_mismatch") is not shown in it nor in any copy of stderr; the line stays in RunResult.stderr for the banner logic.
- The HARD/SOFT signal is the machine line's top `class` (N10), never the
  prose. `soft` adds one button to the banner:
- **MG-OVERRIDE**: the button label `Override…`. No machine line, or
  `class` `hard`, or invalid JSON: no button (fail closed).
- **MG-DIALOG** (modal; extension point the existing confirm window):
  title `Override a network protection`; body, filled from the machine
  line's first soft contradiction:

      This key's path says <A> (coin type <c>h).
      <b-line>
          <a.what>
      If you go on:
      - Addresses and files are made for <B> from a key at a <A> path.
      - Coins sent there may not show up in your signer or wallet app. Many signers look only at <B> paths on <B> and some refuse to sign for this path.
      - Recovery software that searches the standard paths will not find the coins.
      - Whoever restores this wallet later has to make the same override, and has to know to.
      If the path is a mistake, press Cancel and export the key again from your signer.
      To go on, type this sentence exactly:
          I acknowledge I may lose funds by overriding this protection
      [ text field ]
      [ Cancel ]   [ Override and run once ]

  `<A>` and `<B>` are `TEST NETWORK` or `MAINNET` from `a.network` and
  `b.network`. `<b-line>` is `The key and this run say <B>.` when `b.what`
  names a key (`key @i`, a flag such as `--key @0`); `This run says <B>; the key carries no network of its own.` when `b.what` is `this run` and the argv has `--network`; `This run says MAINNET: no network was chosen, and mainnet is the default; the key carries no network of its own.` when `b.what` is `this run` and the argv has no `--network`; and `Another key's path says <B>.`
  when `b.what` names a path. With several soft contradictions the dialog lists every
  `a.what` under the first and the sentence is typed once.
- Mechanics: "Override and run once" is disabled until the field equals the
  sentence after trimming leading and trailing spaces, compared
  case-insensitively, with no other tolerance. Cancel has focus. A paste
  into the field clears it (keystrokes only). The button re-runs the SAME
  argv once with `--accept-coin-type-mismatch` inserted where
  `--allow-argv-secret` goes ({{mnemonic-gui@922a0b383:src/form/channels/mod.rs:1039}} "argv.insert(at, ALLOW_ARGV_SECRET.to_string());"),
  so before any `--`. The flag is never stored, never a form field, and is
  dropped when that run ends or any field changes. The Override button and its banner exist only for the form state that made the refused run: any field change, tab or subcommand change, or a new Run removes them. The re-run is assembled from that unchanged form state through the normal Run path (the secrets confirm and private channels apply as for any run); no RunPlan, argv or secret payload of the refused run is kept for it beyond what RunResult already holds. Before spawning, the assembled argv with the new flag removed must equal the refused run's RunResult.argv; if not, nothing runs and the banner is removed. The secrets confirm
  still appears after it if the run carries secrets.
- **MG-AMBER**: after the overridden run an amber banner that stays until
  the next run: `Run with a network protection overridden` and the CLI's
  `warning:` line (TX-S-OK). Exit 0 does not turn it green.
- `--accept-coin-type-mismatch` is a GUI-managed flag
  ({{mnemonic-gui@922a0b383:src/form/invocation.rs:64}} "pub fn is_gui_managed_flag(name: &str) -> bool {"):
  mirrored in the schema, never rendered, never emitted from form state.
- The network dropdown needs no change: no flag and `mainnet` mean the same
  (D1); FOLLOWUP `network-dropdown-default-forces-explicit-mainnet` is moot.

### 6.2 mr-gui (refugium-wallet; signet-only)
mr-gui never shows a CLI flag and offers no override while signet-only (D6).
mr-codec's api calls (create, verify, export, address, member_policy, record_plates) put no network text (TX-H1 to TX-H5, TX-H4C, TX-H4P, TX-S, TX-S-OK, TX-P, TX-PX, TX-PX-END, TX-N, TX-T, TX-R) into the Outcome's stderr or a Failure message: the mr CLI layer renders those from the typed result (unit 5 of Spec C). The panel "What mr printed" therefore carries none of them; GUI-N and GUI-R are the app's own lines, and a set-aside stray plate shows nothing. nr8g_no_cli_flag_in_any_text scans every text mr-gui can show, the panel included.
mr-codec returns every contradiction as a typed error (class, text id, the
two sources); mr-gui stops matching mr's sentences for them
({{refugium-wallet@b58091825:crates/refugium-view/src/create/refusal.rs:92}} "not at this seat's path").
- **GUI-H**: `These keys are mainnet keys. This version of Refugium works on signet only. Export each key again with the signer set to a test network, or use mr on the command line for a mainnet wallet.`
- **GUI-P**: `These keys were exported at the mainnet path (…/0h/…). On signet each seat's path is …/1h/…. Export each key again at the path shown beside its seat.`
- **GUI-N**: `This card names no network. Refugium reads it on signet; this version cannot read a mainnet wallet.`
- **GUI-S0**: `This card's key paths use coin type 0h, the path mainnet wallets use. This version of Refugium does not read such a card. If its wallet is on signet, read it with mr on the command line; if it is a mainnet wallet, this version cannot read it.`
- **GUI-R**: in the Create flow's FinishBlock, in its own box above the
  page's buttons, when any seat's key came from a sitting's hand-back:
  `This rehearsal used your real seeds. Everything it engraved or printed from them is as secret as your real kit: destroy it, do not discard it.`

### 6.3 SeedHammer (firmware owner's handoff; D7)
- **SH-S** (coin-type warning screen): title `Network mismatch`; body `This key is a <KCLASS> key (<pfx>), but its path <path> is a <PCLASS> path (coin type <c>h). A signer or recovery tool that looks only at <KCLASS> paths will not find coins sent here.`; `<KCLASS>`/`<PCLASS>` are `MAINNET` or `TEST NETWORK`. Use ConfirmWarningScreen (Back = Button1, hold = Button3). The body ends with the line "Hold button to confirm.", as childSeedWarning's does. (<pfx>) is omitted, with its space, when the source has no version prefix (mk1 card, UR use-info). One screen per mismatching key, in key order, each needing its own hold; Back on any screen abandons the flow. Entry points: descriptor load (bip380 parse result), verify_address.go before address.Find, key: record accept, mk1 seat in composer_sources.go, UR hdkey accept. Line breaks are the firmware's.
- **SH-S-RUN** (coin type against the device's mainnet run where the key carries no version: md1 policy address check, policy_address.go and md1_expand.go): title `Network mismatch`; body `This card's path <path> is a TEST NETWORK path (coin type 1h), but this device shows mainnet addresses here. Addresses shown would be wrong for a test wallet.`; buttons as SH-S.
- **SH-K**: `This record holds a test-network key (tpub). This record takes mainnet keys.`
- **SH-L**: under the card in the composer list: `test-network key (tpub): not offered here`.
- **SH-RT**: `This is a regtest address. Regtest addresses are not supported.` Trigger: the candidate, lower-cased, starts with bcrt1 (checked in gui/verify_address.go before address.Find, which is unchanged); today's screen for it is "Invalid address.". SH-K shows as the result screen of the flow that read the record (the sysw classifier returns a new sysw.ClassKeyTestnet, admitted by no site, listed in sysw_admit_oracle_test.go). SH-L: composerSource gains a refused string; a source with it set is drawn in the pick list with that line under its label and skipped by selection (composer_seat.go). SH-K and SH-L write (tpub) only for a key: record; for an mk1 card the parenthesis is omitted.

The handoff names the three readers that apply SH-L: buildCosignerCards, the verify card gather, the composer list.

## 11. FIX units with RED rows

The column legend of the RED-row tables is in SPEC_network_rule.md (Spec A) §11.

### Unit 6: me pack and bundle (mnemonic-engrave)
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R6.1 | me | - | `me sysw pack --as descriptor --in d_t_c0.txt --out $W/p1.bin --no-passphrase` | 0 | - | `address 0: tb1qp5wfc` | `coin type` | 8 | TX-S (key form, no `--network` remedy); with the flag 0 and TX-S-OK; with the flag stderr has `the output is for a test network` |
| R6.2 | me | - | `me sysw pack --as descriptor --in d_x_c1.txt --out $W/p2.bin --no-passphrase` | 0 | - | `address 0: bc1qp5wfc` | `coin type` | 8 | TX-S |
| R6.3 | me | - | `me sysw pack --as descriptor --in d_mix.txt --out $W/p3.bin --no-passphrase` | 3 | - | `while key 0 is` | - | 8 | TX-H2 |
| R6.4 | me | - | `me bundle --in cards.mix_mk_main_md_sig --manifest $W/m4.json` | 0 | - | - | `coin type` | 8 | TX-S naming the md1's coin type and the mk1's key |
| R6.5 | me | - | `me bundle --in cards.mix_mk_sig_md_main --manifest $W/m5.json` | 0 | - | - | `coin type` | 8 | TX-S |
| R6.6 | me | - | `me bundle --in cards.sig --manifest $W/m6.json` | 0 | - | - | `coin type` | 0 | unchanged (control) |
| R6.7 | me | - | `me sysw pack --as descriptor --in d_t_noorigin.txt --out $W/p7.bin --no-passphrase` | 0 | - | `address 0: tb1qp5wfc` | `coin type` | 0 | unchanged (control: no origin block, so no coin type) |

The SOFT check reads a key's coin type only when Key.origin_explicit is true (the holder wrote the [fingerprint/path] block); the implied path in Key.origin is no source (N1), and a key with no origin block never raises TX-S (R6.7).

TRACE rows: `nr6_pack_coin_type_soft_refused_and_overridden` (R6.1, R6.2),
`nr6_pack_mixed_keys_exit_8` (R6.3), `nr6_bundle_md1_mk1_coin_type_refused`
(R6.4, R6.5), `nr6_bundle_consistent_unchanged` (R6.6),
`nr6_exit_code_vocabulary_has_8`, `nr6_pack_implied_path_no_source` (R6.7), `nr6_bundle_mk1_against_mk1_exit_8`, `nr6_pack_keyed_md1_coin_types_path_form` (the keyed md1 check needs an md-codec decode in sysw::admit_check), `nr6_bundle_mk1_own_coin_type_soft`; fixtures owed by unit 6: `me/cards.mk_main_mk_test`, `me/cards.md_coins_mixed`, `me/cards.mk_own_coin1_xpub`. The coin-type check is a separate step after admit and after the identification block: the block prints in full (with address 0:, for the key's own network) on a refused and on an overridden run, then TX-S (exit 8, nothing written) or TX-S-OK (exit 0) follows; the compare prompt is kept. R6.1 after: stderr still has address 0: tb1qp5wfc, then TX-S.

### Unit 8: the rest (one PR each, in this order)

The order of the sub-units is in SPEC_network_rule.md (Spec A), Unit 8.

8d md and mk coin type (descriptor-mnemonic, mnemonic-key + manual):
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R8d.1 | fx | - | `md address --template "wpkh(@0/<0;1>/*)" --key "@0=[73c5da0a/84h/1h/0h]$X" --network mainnet` | 0 | `bc1qcr8te4k` | - | `coin type` | 8 | TX-S |
| R8d.2 | fx | - | `md address --template "wpkh(@0/<0;1>/*)" --key "@0=[73c5da0a/84h/0h/0h]$T" --network testnet` | 0 | `tb1q6rz28mc` | - | `coin type` | 8 | TX-S |
| R8d.3 | fx | - | `mk encode --xpub $X --origin-path "m/84'/1'/0'" --origin-fingerprint 73c5da0a --policy-id-stub 00000000` | 0 | - | `mk1qp` | `coin type` | 8 | TX-S; with the flag 0 |
| R8d.4 | fx | - | `mk encode --xpub $T --origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a --policy-id-stub 00000000` | 0 | - | `mk1qp` | `coin type` | 8 | TX-S |
| R8d.5 | fx | - | `md address $(cat md_keyed_nopath.txt) --count 1` | 0 | `bc1qcr8te4k` | - | `names a network` | 0 | stderr has TX-N |
| R8d.6 | fx | - | `md address $(cat md_keyed_nopath.txt) --count 1 --network signet` | 0 | `tb1qcr8te4k` | - | `test network` | 0 | stderr has TX-T (`this card names no network`) |
| R8d.7 | fx | - | `md descriptor $(cat md_keyed_nopath.txt)` | 0 | - | - | `names a network` | 0 | stderr has TX-N |
| R8d.8 | fx | - | `mk encode --xpub $T --origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a --policy-id-stub 00000000 --json` | 0 | `mk1_strings` | - | `coin type` | 8 | stdout is the envelope with `"kind":"NetworkMismatch"` and `"exit_code":8`; stderr TX-S |

TRACE rows: `nr8d_md_key_coin_type_soft` (R8d.1, R8d.2),
`nr8d_mk_encode_coin_type_soft` (R8d.3, R8d.4),
`nr8d_md_no_source_tx_n_tx_t` (R8d.5, R8d.6, R8d.7; leg: `--network mainnet`
keeps the note), `nr8d_md_keyed_card_coin_type_soft`,
`nr8d_md_path_shortcut_follows_network`, `nr8d_mk_encode_json_envelope_exit_8` (R8d.8), `nr8d_md_decompose_coin_type_soft_commands_carry_flag`, `nr8d_mk_encode_batch_mixed_keys_refused` (fixture owed: `fx/recs_mixed.txt`, one xpub and one tpub record).

8e mt (mnemonic-transaction):
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R8e.1 | mt | - | `mt inspect --in mt1.txt --bitcoin-cli /nonexistent` | 0 | `addresses shown as MAINNET` | - | `names a network` | 0 | stderr has TX-N-MT |
| R8e.2 | mt | - | `mt inspect --in mt1.txt --bitcoin-cli /nonexistent --json` | 0 | `bc1q0xmdy` | - | `names a network` | 0 | JSON has `"network":"mainnet","network_source":"default"`; stderr TX-N-MT |
| R8e.3 | mt | - | `CHAIN=testnet4 mt inspect --in mt1.txt --bitcoin-cli ./fakecli.sh` | 0 | `bc1q0xmdy` | - | - | 0 | `tb1q0xmdy...` on the head build (closed by reinstall, no code change) |
| R8e.4 | mt | - | `CHAIN=signet mt inspect --in mt1.txt --bitcoin-cli ./fakecli.sh` | 0 | `tb1q0xmdy` | - | `names a network` | 0 | unchanged (control) |
| R8e.5 | mt | - | `mt decode --in mt1.txt --bitcoin-cli /nonexistent` | 0 | `^0200` | `no node to ask` | `names a network` | 0 | stderr has TX-N-MT |
| R8e.6 | mt | - | `mt inspect --in tx.hex --bitcoin-cli /nonexistent` | 0 | `no node to ask` | - | `names a network` | 0 | stderr has TX-N-MT |
| R8e.7 | mt | - | `mt inspect --in mt1.txt --bitcoin-cli ./sheb.sh` | 0 | `tb1q0xmdy` | - | `names a network` | 0 | unchanged (control: the remedy's file shape, with fakecli.sh standing in for bitcoin-cli -signet) |

TRACE rows: `nr8e_no_node_tx_n_mt` (R8e.1), `nr8e_json_network_fields`
(R8e.2), `nr8e_node_chain_unchanged` (R8e.4). The report's own note (addresses shown as MAINNET — no node to ask) stays in the report; TX-N-MT is an addition on stderr. JSON: "network" and "network_source" are inserted after "set_prefix"; network is mainnet, testnet, testnet4, signet or regtest (Core's test written testnet); network_source is "node", "default" or "node-unreadable"; warnings is unchanged. TRACE: nr8e_decode_and_raw_inspect_tx_n_mt (R8e.5, R8e.6), nr8e_remedy_script_shape (R8e.7).

8f mnemonic-gui (needs 8a, 8b (Spec B), 8d released and pinned): MG-BANNER,
MG-OVERRIDE, MG-DIALOG, MG-AMBER, `MNEMONIC_NETWORK_JSON=1`. No RED row runs
a GUI; TRACE rows: `nr8f_exit_8_red_banner`, `nr8f_soft_override_button_only_on_class_soft`,
`nr8f_no_machine_line_no_button`, `nr8f_typed_sentence_enables_once`,
`nr8f_paste_clears_field`, `nr8f_flag_inserted_before_double_dash`,
`nr8f_flag_dropped_after_run`, `nr8f_amber_banner_after_override`,
`nr8f_schema_flag_gui_managed`, `nr8f_mg_dialog_b_line_per_form` (key
form, run form, path form; fixture machine lines; legs "run form, argv with `--network signet`" and "run form, argv with no `--network`"), `nr8f_validate_argv_carries_network`, `nr8f_strip_drops_machine_line`.

8h SeedHammer firmware (another owner's handoff): SH-S, SH-K, SH-L, SH-RT.
TRACE rows: `nr8h_coin_type_screen_default_back`,
`nr8h_hold_goes_on`, `nr8h_tpub_key_record_sh_k`,
`nr8h_test_mk1_listed_sh_l`, `nr8h_regtest_candidate_sh_rt`.

## 12. Build impact

7. Unit 6: mnemonic-engrave `crates/me-cli/src/descriptor/admit.rs`,
   `bundle.rs`, `main.rs` (flag, exit 8), `crates/me-cli/src/sysw/composer_records.rs`; tests:
   `the_exit_code_vocabulary_is_one_vocabulary`, MixedNetwork exit pins.
12. Unit 8d: descriptor-mnemonic `md-cli` address/descriptor/encode
   (coin type, TX-N/TX-T replacing the note at
   {{descriptor-mnemonic@60956b858:crates/md-cli/src/cmd/address.rs:112}} "an md1 card does not record its network"),
   path shortcuts; mnemonic-key `mk-cli` encode/address; closes FOLLOWUPS
   `md-descriptor-keyed-card-network-silent`,
   `md-path-shortcut-coin-type-mainnet`; manual 42-md, 44-mk-cli.
13. Unit 8e: mnemonic-transaction `crates/mt-cli/src/report.rs`
   ({{mnemonic-transaction@88ac6d9e0:crates/mt-cli/src/report.rs:437}} "no node to ask"),
   JSON fields; tests `crates/mt-cli/tests/inspect.rs`; mt's own manual.
14. Unit 8f: mnemonic-gui `src/app_window.rs`, `src/runner.rs`,
   `src/form/channels/mod.rs`, `src/form/invocation.rs`, `src/form/tree_form.rs`, schema mirror;
   toolkit `docs/manual-gui`.
16. Unit 8h: seedhammer firmware (owner's schedule).
