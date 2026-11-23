# SPEC: one network-contradiction rule for every tool, units 4, 5, 8c, 8g

Normative rule, texts and vectors: SPEC_network_rule.md (Spec A) at <A-COMMIT>; this spec adds units only and never changes §2/§3.
Spec map: A = SPEC_network_rule.md (§1-§10, §13, units 0, 1, 7); B = SPEC_network_rule_tk_rest.md (units 2, 3, 8a, 8b); C = SPEC_network_rule_mr.md (units 4, 5, 8c, 8g); D = SPEC_network_rule_codecs_gui.md (§6, units 6, 8d, 8e, 8f, 8h).
Units here: 4, 5, 8c, 8g. Machine checks: `scripts/spec-network-rule-gate.sh` reads all four specs.

## 11. FIX units with RED rows

The column legend of the RED-row tables is in SPEC_network_rule.md (Spec A) §11.

### Unit 4: mr upgrade key version (mnemonic-refugium + erratum E3 + toolkit manual 45-mr)
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R4.1 | fx | - | `sh up.sh 0 0488b21e signet` | 0 | `network: signet` | `carry coin type 0` | `network mismatch` | 8 | TX-H1 (keys @0 and @1 mainnet keys, `--network is signet`); no kit |
| R4.2 | fx | - | `sh up.sh 1 0488b21e signet` | 0 | `xpub6` | - | `network mismatch` | 8 | TX-H1, then TX-S without its flag sentence (xpub keys at coin-1 paths, as V15's mirror); no kit |
| R4.3 | fx | - | `sh up.sh 0 043587cf main` | 0 | `tpub` | - | `network mismatch` | 8 | TX-H1 (`--network is main`), then TX-S without its flag sentence (tpub keys at coin-0 paths, as V15) |
| R4.5 | fx | - | `sh up.sh 1 043587cf signet` | 0 | `network: signet` | - | `network mismatch` | 0 | unchanged except TX-R last (unit 5) |

TRACE rows: `nr4_upgrade_key_version_refused` (R4.1, R4.2, R4.3),
`nr4_upgrade_consistent_unchanged` (R4.5). Sources in upgrade: the version of each --keys plate or record and of each --hot-key record; the --witness-descriptor's keys are not counted (they must equal the given keys). <who> is key @i for a --keys entry bound to seat i (plural keys @0 and @1, N8) and --hot-key record <k> for a hot-key record. The refusal is raised in uprun::seat_keys after the seeds prompt, as create's is, before anything reaches stdout. Today create checks --keys and --hot-key (hotrun.rs:845, create only) and upgrade checks neither. TRACE: nr4_upgrade_hot_key_version_refused (fixture owed: up.sh with a tpub --hot-key on main). R4.5's unit-4 test expects no TX-R; unit 5 adds it.

### Unit 5: mr no-network and rehearsal lines (mnemonic-refugium + mr spec §7 + manual 45-mr, 46-mr-sitting)
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R5.1 | fx | seeds.txt | `mr address $(cat card_sig.txt) --seeds - --epoch 0 --slot 0 --index 0 --height 900000 --time 1760000000` | 0 | `bc1qk4d5x` | - | `names a network` | 0 | same address; stderr has TX-N |
| R5.2 | fx | seeds.txt | `mr export $(cat card_sig.txt) --seeds - --height 900000 --time 1760000000` | 0 | `network: main, coin type 0` | - | `names a network` | 0 | stderr has TX-N |
| R5.3 | fx | - | `mr verify $(cat card_sig.txt)` | 0 | - | - | `names a network` | 0 | stderr has TX-N |
| R5.4 | fx | seeds.txt | `mr address $(cat card_sig.txt) --network signet --seeds - --epoch 0 --slot 0 --index 0 --height 250000 --time 1760000000` | 0 | `tb1qu6m7m` | - | `test network` | 0 | stderr has TX-T |
| R5.5 | fx | - | `sh cr.sh signet 1 043587cf` | 0 | - | - | `This rehearsal used your real seeds` | 0 | TX-R is the last stderr line |
| R5.6 | fx | - | `sh cr.sh main 0 0488b21e` | 0 | - | - | `This rehearsal` | 0 | unchanged: no TX-R on main (control) |
| R5.7 | fx | - | `mr lost-or-stolen $(cat card_sig.txt) @1 --held @0 --height 900000 --time 1760000000` | 0 | `export its key at m/270028'/0'/` | - | `names a network` | 0 | stdout unchanged; stderr has TX-N |

TRACE rows: `nr5_no_flag_read_prints_tx_n` (R5.1, R5.2, R5.3),
`nr5_test_flag_read_prints_tx_t` (R5.4),
`nr5_create_with_seeds_on_signet_prints_tx_r_last` (R5.5; leg: `upgrade`
with `--seeds` on signet, R4.5),
`nr5_create_on_main_prints_no_tx_r` (R5.6),
`nr5_tlv_card_no_flag_reads_mainnet_tx_n` (erratum E5; RED: exit 1
`TlvHandoutNetworkNotGiven`),
`nr5_sitting_brought_seed_test_plan_prints_tx_r` (mr-sitting practice
harness; RED: no line), `nr5_sitting_drawn_seeds_only_no_tx_r`,
`nr5_lost_or_stolen_no_flag_prints_tx_n` (R5.7).

mr-codec api (crates/mr-codec/src/api/typed.rs, api.rs): FailureKind::NetworkMismatch { class: Hard|Soft, contradictions: Vec<Contradiction> } with Contradiction { text_id: &'static str, a: Source, b: Source } and Source { what: String, network: String }, exactly N10's fields, set with io.record_failure on every exit-8 refusal of units 4, 5 and 8c (mr-sitting excepted); Io gains fn card_names_no_network(&self) -> bool (set where TX-N or TX-T would print) and fn stray_plates(&self) -> &[(usize, usize)] (plate k, seat i; set where TX-P would print); the request structs gain accept_coin_type_mismatch: bool (default false; mr-gui always passes false). TRACE: nr5_typed_network_failure_matches_machine_line.

### Unit 8: the rest (one PR each, in this order)

The order of the sub-units is in SPEC_network_rule.md (Spec A), Unit 8.

8c mr SOFT and plates (mnemonic-refugium + errata E2 + manual 45-mr; needs 8d's (Spec D) mk released, since the kit prints `mk encode ... --accept-coin-type-mismatch`):
| row | dir | stdin | command | today rc | today stdout has | today stderr has | today stderr lacks | after rc | after |
|---|---|---|---|---|---|---|---|---|---|
| R8c.1 | fx | - | `sh up.sh 0 043587cf signet` | 0 | `network: signet` | `carry coin type 0` | `network mismatch` | 8 | TX-S (key form, plural: `mr: network mismatch (coin type): keys @0 and @1's paths m/48'/0'/0'/2' and m/48'/0'/0'/2' are mainnet paths (coin type 0'), but keys @0 and @1 are test-network keys (tpub). ...`; the sources are the `--keys` records' versions and the md1's paths); with `--accept-coin-type-mismatch` 0, TX-S-OK, and each printed command line of `mr create`, `upgrade`, `verify`, `decode`, `member`, `address`, `current`, `export`, `lost-or-stolen` and of `mk encode` (a line a holder runs, template placeholders included; not a prose mention such as `mr --help` or a bare `mk encode`) ends with ` --accept-coin-type-mismatch`; the test counts such lines in stdout and fails on any without it |
| R8c.2 | fx | - | `mr export $(cat card_sig.txt) --keys "$(sh plate.sh 0 0488b21e 0)" "$(sh plate.sh 0 0488b21e 1)" "$(sh plate.sh 1 043587cf 0)" --height 900000 --time 1760000000` | 0 | `rule: @0=plate 1, @1=plate 2` | `this plate is for a test network` | `from a rehearsal` | 0 | TX-P (`plate 3 (m/270028'/1'/0') is a test-network key (tpub) from a rehearsal; it was not used. Seat @0 is filled by plate 1.`) in place of today's two lines; stdout unchanged |
| R8c.3 | fx | - | `mr export $(cat card_sig.txt) --keys "$(sh plate.sh 0 0488b21e 0)" "$(sh plate.sh 1 043587cf 1)" --height 900000 --time 1760000000` | 1 | `seat @1: missing` | `nothing to export` | `network mismatch` | 8 | TX-PX (seat @1, plate 2 (m/270028'/1'/1'), from a rehearsal) then TX-PX-END; stdout empty |
| R8c.4 | fx | - | `sh crs.sh main 1 0488b21e` | 0 | `card digest` | - | `coin type` | 8 | TX-S (the `--sibling-keys` record's coin-1 path m/84'/1'/0' against its xpub, as V09); stdout empty; with `--accept-coin-type-mismatch` 0, TX-S-OK, and each printed command line of `mr create`, `upgrade`, `verify`, `decode`, `member`, `address`, `current`, `export`, `lost-or-stolen` and of `mk encode` (a line a holder runs, template placeholders included; not a prose mention such as `mr --help` or a bare `mk encode`) ends with ` --accept-coin-type-mismatch`; the test counts such lines in stdout and fails on any without it |
| R8c.5 | fx | - | `sh cr.sh main 0 043587cf` | 1 | - | `record 1's xpub is for signet, testnet and regtest` | `network mismatch` | 8 | TX-H1 (`key @0`, `--network is main`); stdout empty |
| R8c.6 | fx | - | `sh cr.sh signet 1 0488b21e` | 1 | - | `record 1's xpub is for mainnet` | `network mismatch` | 8 | TX-H1 (main key, `--network is signet`); stdout empty |
| R8c.7 | fx | - | `sh crs.sh main 1 043587cf` | 0 | `card digest` | - | `network mismatch` | 8 | TX-H1 (`--sibling-keys record 1 is a test-network key (tpub)`, `--network is main`); no coin line (its coin 1 agrees with its key, N8); stdout empty, no kit |
| R8c.8 | fx | - | `sed 's/^network signet/network testnet/' sit_plan.txt > $W/pt.txt && mr-sitting verify --plan $W/pt.txt --hand-back sit_hb.txt` | 2 | - | `plan-sha256 differs` | `network mismatch` | 8 | TX-H4P (`mr-sitting: network mismatch: sit_hb.txt says network signet, but the plan says network testnet. Nothing was written. Use the hand-back written from this plan, or give the plan it was written from with --plan.`) |
| R8c.9 | fx | - | `sh upx.sh . 0488b21e` | 1 | `network: signet` | `another network than Signet` | `network mismatch` | 8 | stdout empty; TX-PX (`seat @0 has no signet key: plate 1 (m/48'/1'/0'/2') is a mainnet key (xpub)`), TX-PX-END |
| R8c.10 | fx | - | `sh upx.sh . 043587cf` | 0 | `wsh` | - | `network mismatch` | 0 | unchanged (control) |

TRACE rows: `nr8c_upgrade_coin_type_soft_and_kit_carries_flag` (R8c.1; leg: each printed `mk encode` line, run with the 8d (Spec D) mk, exits 0),
`nr8c_create_sibling_hot_bundle_coin_type_soft` (R8c.4; legs: `--hot-key`, `--bundle-card` md1),
`nr8c_explicit_path_card_read_coin_type_soft_per_key` (M-1 leg: one coin-0
and one coin-1 path on one card),
`nr8c_stray_plate_set_aside_tx_p` (R8c.2; leg: the mirror on signet),
`nr8c_stray_plate_only_key_refused_tx_px` (R8c.3, R8c.9, R8c.10; legs: `verify`,
`address`, a stray `[fingerprint/path]xpub` record named `record <k>`, and a `verify` leg on the explicit-path card of R8c.9), `nr8c_mr_sitting_network_refusals_exit_8` (R8c.8).
plates::coin_of is replaced by a function that returns a coin only for a path of the shape purpose'/coin'/... with purpose in N1's list and coin in {0, 1}; every caller (seating included: create.rs:1190, seat.rs:863, seatkey.rs:383, hotrecord.rs:282, plates.rs:1657, seating.rs:4636) uses it. TRACE: nr8c_other_purpose_path_no_source (a card at m/45'/1'/0' read with no flag: rc 0, no TX-S, TX-N on stderr; fixture owed).
8c TRACE: `nr8c_create_key_version_hard` (R8c.5-R8c.7; legs `--hot-key`, `--bundle-card` mk1, and the other-coin seat path row "today's text kept", exit 8), `nr8c_lost_or_stolen_coin_type_soft_per_card`.

8g refugium-wallet (needs the mr-codec pin past units 4, 5, 8c): GUI-H,
GUI-P, GUI-N, GUI-S0, GUI-R and the typed error; it consumes the typed error, card_names_no_network and stray_plates of unit 5. The api keeps the CLI's code: a network refusal is code 8 in api::Outcome. refugium-view maps it: mr_panel.rs CallState::of_code gains a state NetworkRefused for code 8 (the refused state, Continue disabled, REFUSED_REASON); restore/mod.rs and create/mod.rs take_result and the path-line match treat 8 as they treat 1 and take the lead from the typed error (text id, never the printed sentence); no network refusal reaches USAGE_TEXT. TRACE: nr8g_exit_8_is_refused_not_usage. A saved draft of another network: create/draft.rs from_draft and restore/draft.rs return a refusal reason (WrongNetwork or Other) instead of bool; the saveload dialog shows GUI-H's second sentence under T14 for WrongNetwork. TRACE: nr8g_saved_draft_other_network_names_signet_only. TRACE rows:
`nr8g_mainnet_records_gui_h`, `nr8g_other_coin_path_gui_p_no_flag`,
`nr8g_card_names_no_network_gui_n`, `nr8g_coin0_explicit_card_gui_s0`,
`nr8g_finish_from_handback_gui_r`, `nr8g_no_cli_flag_in_any_text`. Placement: GUI-H and GUI-P are row_lead rows 1a and 1b in refusal.rs (constants LEAD_MAINNET_RECORDS and LEAD_OTHER_COIN_PATH in texts.rs and menu/mint.rs; the §8.3 table of design/SPEC_ui_refugium_wallet.md gains both rows in the same commit), each with Lead.step equal to LEAD_WRONG_PATH's step for the entry (3; 4 in build-my-own); GUI-S0 is the lead of the typed SOFT refusal, at the card's step; GUI-N is a notice under the card in restore/ (Restore step 1) and in balance/read.rs; GUI-R is a notice in FinishBlock (ui/round/create.slint) above the "Open your Refugium" button. GUI-P's ellipses are literal.

## 12. Build impact

5. Unit 4: mnemonic-refugium `crates/mr-codec/src/app/create/uprun.rs`
   (the `create` check), spec erratum E3; tests: new
   `upgrade` network rows; toolkit manual 45-mr `upgrade`.
6. Unit 5: mnemonic-refugium `crates/mr-codec/src/card.rs` read path
   (TX-N/TX-T), `network.rs` (`check_tlv_handout` retired, erratum E5),
   `app/create.rs` and `uprun.rs` (TX-R), mr-sitting
   `src/session.rs` (TX-R after ST-END-02); mr spec §7; tests: new rows,
   any golden of a read verb's stderr; manual 45-mr §`--network`,
   46-mr-sitting ST-END.
11. Unit 8c: mnemonic-refugium `card.rs` (`Warning::CoinType` becomes the
   SOFT refusal, per key), `network.rs` (`paths_coin` per key),
   `plates.rs` (N13, records included), `app/create.rs` (create's `--keys`/`--hot-key`/`--sibling-keys`/`--bundle-card` key version HARD; create and upgrade
   SOFT, kit commands carry the flag), mr-sitting exit codes; errata E2; tests:
   `pr54_export_coin_type_warning_tagged_per_card` rewritten, plate
   set-aside pins; manual 45-mr, 46-mr-sitting exit tables.
15. Unit 8g: refugium-wallet `crates/refugium-view/src/create/refusal.rs`,
   `restore/` lead files, `create/finish.rs`, `ui/round/create.slint`, `ui/bridge.slint`,
   `crates/mr-gui/src/create_round.rs`, `balance/read.rs`, `balance/texts.rs`,
   `design/SPEC_ui_refugium_wallet.md`, `create/draft.rs`, `restore/draft.rs`,
   `saveload/model.rs`, mr-codec pin bump; consumes the typed error,
   card_names_no_network and stray_plates of unit 5.
