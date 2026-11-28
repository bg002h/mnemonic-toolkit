# R1 re-review: IMPLEMENTATION_PLAN_network_rule_tk.md r1 (commit c8a3f61d)

Reviewer: independent adversarial architect (Claude Opus 5.5), 2026-11-28.
Scope, as briefed: did the fold fix each R0 finding
(`design/agent-reports/network-rule-tk-plan-r0.md`), and did it introduce new
defects? The new parts checked are the SectionKey model, merge rule B5, rule B1,
step 8a-0, the per-parser import-wallet table and the verify-bundle check site.
Settled parts were not re-audited. The fixture commands were re-run with the
cached `target/debug/mnemonic` (no rebuild). New cites were opened at
d72a18b1.

Counts: **Critical 0, Important 2, Minor 7, Nit 1.**

All 10 R0 Importants are fixed in the plan text. I-4 now rests on Brian's
ruling: step 8a-0 honours `--origin`, with a refusal as the alternative. The
two new Importants come from the fold itself.

## Disposition of the R0 findings

| R0 | fixed? | evidence |
|---|---|---|
| I-1 | yes | `SectionKey` is defined (§2.1). B3 puts a stated file's keys only inside `Stated`. `envelope_source` and the vector harness (§2.5) use the same model. A12 is recorded. With B3, R1.13 gives TX-H4 alone and R1.21 gives TX-H4C alone. |
| I-2 | yes | `Key.coins: Vec<KeyCoin>`, and `PathWho::Named("key @N")` binds the md1 slot coin to that slot's cosigner (§3 unit 2). R2.4 under B4 gives exactly the §3.0 literal. A13 asks the spec owner for the TX-S wording when the path and the key have different names. |
| I-3 | yes | B5 merges key-form TX-S lines that share `<pclass>` and `<kclass>`. The R3.6 line-2 literal matches the §3 key form with its flag sentence dropped. |
| I-4 | yes, pending ruling | The site is named (`restore.rs:524-527`, the single-sig `run`). Step 8a-0 adds its own test, and the alternative refusal is named. The unit-2 template line is corrected (the template carries no origin, B1). |
| I-5 | yes | Unit 3 has a per-parser table, and I checked its cites (bsms `:402`/`:431`, bitcoin_core `:852`, descriptor `:177-222`, sparrow `:598`, specter `:377`, electrum `:786`/`:934`/`:352`/`:996`, coldcard_multisig `:532`/`:549`/`:803`). Fixtures: `d_mixcoin.txt` exits 2 today with "cosigner 1 has coin-type 1, cosigner 0 has coin-type 0" (re-run); `d_x1.txt` exits 2 today with NetworkMismatch (re-run). **New defect: see R1-I1.** |
| I-6 | yes | `vb_signet_bip84.txt` re-run: `--network mainnet --template bip84 --slot @0.phrase=$P --mk1 .. --md1 ..` exits 4 today with the mk1 and md1 xpub/path/origin rows failing. The seed slot is no source (B1), so the mk1 card is the only HARD source and the §3.0 literal follows. The one check sits before the mode dispatch, covering all six modes, and no stdout is written before it (`verify_bundle.rs:260-497` has no stdout write). **New defect in the TX-V leg: see R1-I2.** |
| I-7 | yes | `envelope_source` is shared with `bundle --import-json` (TX-H4, TX-H4C, TX-S). A new plan test covers it, A16 records it, and a FOLLOWUP is filed for the table gap. |
| I-8 | yes | B1 makes derived paths no source, and a one-line leg on R1.12 pins it. |
| I-9 | yes | A3 is settled on Spec A §3. The R8a.11 and R8a.12 literals match §3's key form and run form. |
| I-10 | yes | The census is in `test-census-tk.md`, with a step-0 rule (never delete or loosen a flipped test). The as1-i4 leg is re-run: `restore $(mdargs m_signet.txt) --cosigner @0=<xcos l1> --cosigner @1=<xcos l2>` exits 0 "cross-checked" today, so after the change it is a SOFT-only run. Only the FOLLOWUPS whose legs are built are closed. |
| M-1..M-16, N-1..N-4 | yes | Each was checked in the text: A9 corrected; §4.2 exit tables listed; §5 cites (`convert.rs:1287-1289`); both `--expect-xpub` compares; the trailer after `report_at_exit`; sections ordered by name; types defined and TX-R returned by `evaluate`; the re-export; B2; `seed_only` on template completion; the bsms guard kept; manual-gui; `nr8f_schema_flag_gui_managed`; the PR deviation recorded; B5/B6 for prefixes and several addresses; §3.0 literals; corrected cites; `SHIB`; recipes; the #228/#229 cites. |

I checked every §3.0 literal against the Spec A §3 templates (TX-H1 both
directions, TX-H2, TX-H3, TX-H4, TX-H4C, the TX-S key and run forms, and the
drops of B6). Each matches the spec's words for its fields.

## New findings

### Important

**R1-I1: Deleting bsms's `network_from_origins` breaks its first-address audit, which still needs a network.**
Unit 3's table deletes bsms's `network_from_origins` (`bsms.rs:402`), and
`p.network` becomes a placeholder `bitcoin::Network::Bitcoin` "until the
central check". But the parser uses that inferred network inside itself before
it returns. At `bsms.rs:313`, `derive_first_address(&parsed, network)` computes
the first address and compares it with the Round-2 audit's declared
`first_address`; a mismatch prints `warning: import-wallet: bsms: first-address mismatch ...`.
With the placeholder, every correct test-network BSMS blob imported with
`--network testnet`/`signet` gets a false mismatch warning (a computed `bc1...`
against a declared `tb1...`). A real mismatch on a test blob is then lost among
false ones. The other parsers only store `network` (descriptor `:107`/`:125`,
bitcoin_core `:724`/`:763`, sparrow `:503`, specter `:320`, coldcard_multisig
`:614`/`:631`). Electrum's synthesized derivation (`electrum.rs:569`, `:633`)
stays correct only because the table keeps `network_from_xpub_neutral`.
Fix:
- Pass the run network into `Parser::parse` (it is known before the dispatch),
  or move the bsms first-address audit after the central check, using the run
  network.
- Add a test: a test-network BSMS blob on `--network testnet` prints no
  first-address warning.
- Also note for the spec owner: the declared first address is itself an N1 HARD
  source (an address HRP) that Spec A §4.1 does not list. Put it in a FOLLOWUP
  beside A16, or make it a source.

**R1-I2: The TX-V condition prints TX-V's claim on a run whose only failing row is not a coin-type or key-version row.**
The plan's `nr8a_verify_bundle_cause_line` reads "rows fail and no source
contradicts the run: TX-V after the rows, exit 4". The fixture README names
"the signet leg (rc 4 today only on the empty-ms1 row)" as the TX-V control.
Re-run, the signet leg fails only on `ms1_decode` (no `--ms1` was given),
`result: mismatch`. TX-V's text is "every mismatch above is a coin-type or
key-version row: the bundle may be for another network than --network <net>".
Printed on that leg, it is false and sends the user to the wrong fix.
Fix:
- Define TX-V's condition by the failing rows: at least one row fails, and every
  failing row is one of the coin-type or key-version rows. Name them
  (`mk1_xpub_match`, `mk1_path_match`, `md1_xpub_match`, `md1_origin_match` and
  whatever else the modes emit).
- Pin both sides with literals:
  - the signet `vb` leg (an ms1-only failure) prints no TX-V;
  - a leg whose only failures are coin/version rows and that has no source
    prints TX-V. An example: a keyless template md1 and a seed slot verified on
    the other network.
- Pass `--ms1` in the mainnet leg too, so its rows are only the network ones
  (it exits 8 there anyway, but the census and "today" text stay clean).

### Minor

- **R1-M1: Card decoding at the verify-bundle check site.** One check before
  `:497` must decode the `--mk1` and `--md1` cards, but the modes own decode
  failures and auto-repair (with RepairShortCircuit output). Say that the check
  decodes best-effort: a card that does not decode is no source, and the mode
  then handles it as today. Otherwise a corrupt card is refused by the new code
  with a new error before auto-repair runs.
- **R1-M2: B6 merges several `--target-address` values with no spec form.**
  TX-H3 names the whole address, and §3 gives TX-H3 no plural. Print one TX-H3
  per address (input order), or raise it with the spec owner. Do not invent a
  plural.
- **R1-M3: The post-run TX-R Input (§5) does not say `reads_kept = false`.**
  If it inherits `true` with no sources, it prints a second TX-N/TX-T. State the
  field values (`sources: []`, `reads_kept: false`, `seed_only: true`).
- **R1-M4: Step 8a-0 lands unconditionally.** §6 schedules it as commit 6
  regardless of the ruling. Gate that commit on Brian's answer. Also settle how
  `--origin` interacts with `--account` (a non-default `--account` beside
  `--origin`: refuse, or ignore with a note) and with `--expect-xpub`.
- **R1-M5: The bsms guard (M-11) cannot render its line.** "Build a one-key
  `Input` and return `NetworkRule`" runs inside an emitter, which has no
  `RunNet.given`, no raw `pfx` and no `<who>`, so the line it renders would be
  wrong. Since every caller checks first, make it an internal invariant error
  (for example `debug_assert` plus a typed bug error), not a user-facing TX-H1.
- **R1-M6: Census quality.** The census lists helper functions as tests
  (`cli_bip388_validity.rs::ka`, `::kb`, `::ka_other_origin`), and some entries
  have an empty flag class (`p25_controls_...`,
  `xpub_to_address_p2pkh_explicit_script_type_v0_26_0`). The step-0 rule still
  works, but tidy those entries so "on the list" is checkable.
- **R1-M7: Non-dominant Coldcard sections.** B2/B3 compare the coin of every
  Coldcard section, but the parser imports only the dominant one
  (`coldcard.rs:430-431`; bip48_* ignored). Say whether a non-imported section's
  coin/key disagreement prints TX-S (literally yes under B2). Record it as a
  reading (A17), or limit TX-S to imported sections.

### Nit

- **R1-N1:** The verify-bundle site is "before `:503`"; the
  `if !args.md1.is_empty()` dispatch is at `verify_bundle.rs:497`.

## Verdict

Not GREEN until R1-I1 and R1-I2 are folded. Both are narrow. R1-I1 is a
plumbing change (the run network into the parser, or the audit moved after the
check). R1-I2 is the TX-V condition plus two legs. A scoped re-check of those
two folds is enough; nothing else needs another round.
