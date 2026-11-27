# R0 review r1 (opus): SPEC `xpub-search address-of-seed`

**Reviewed:** `/mnt/project-files/address-search/SPEC_xpub_search_address_of_seed.md` (360 lines, DRAFT 2026-11-26).
**Against:** mnemonic-toolkit `claude/project-thread-utm1aa` at `40b35942`. The binary `target/debug/mnemonic` 0.107.1 was built at `93b804e5`. The only change from there to HEAD is to `design/FOLLOWUPS.md`, so the binary matches the code.
**Read-only:** I did not edit the spec or the repo. Scratch files are under `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/`: `derive.py` is a from-scratch secp256k1/BIP-32/bech32/base58 implementation, `vec.py` checks the spec's vectors and `alias.py` shows the C1 alias.

**Verdict: RED.** 1 Critical, 7 Important, 15 Minor, 7 Nit.

## What was checked and holds

- **Every address vector, S1 to S10 and M1 to M4, recomputed independently.** All 14 match (`vec.py`). The fingerprints `73c5da0a` (abandon) and `3f635a63` (zoo) also match.
  - S1, S2 and S3 agree with the BIP texts, fetched from `bitcoin/bips` master: BIP-84 lines 80 and 90, BIP-86 line 100.
  - M2 is exactly right: `multi(2,zoo,abandon)` at 0/0 equals M1's sortedmulti address. That is the root of C1.
- **The path set.** `candidate_paths.rs` yields 4 single-sig families × accounts, then BIP-48 1'/2'/3' × accounts, then BIP-87 × accounts, then `m/45'` once, then each `--add-path`. That is 8n+1, or 161 at n = 20 (unit test `standard_templates_at_account_0_default_range`). There is no dedupe today, so §3's dedupe is new behaviour.
- **Code citations.**
  - The Refugium paths exist only in `restore.rs:3961` (`empty_origin_search_paths`), with n in 0..=255.
  - Refugium `succ.rs:325` `seat_path` = `m/270028'/coin'/(32r+i)'` (confirmed).
  - The other cited functions are at `parse_descriptor.rs:65` (`lex_placeholders`), `parse_descriptor.rs:283` (`resolve_placeholders`), `passphrase_input.rs:192` (`refuse_second_stdin`), `bundle.rs:3410` (`rerun_command_with`) and `bundle.rs:3463` (`redact_and_quote`).
  - `descriptor_intake.rs:159-163` does refuse `@N`.
  - The engine has had no time ceiling since 0.99.0 (CHANGELOG line 1126; `cap_decision`).
  - The exit codes in the "today" column are right: BadInput and Bip39 exit 1, DescriptorParse 2, NetworkMismatch 2, EnvVarMissing 1, XpubSearchNoMatch 4.
- **Measurements re-run on this box.** `address-of-xpub` deriving 100,000 P2WPKH took 5.02 s (spec: 5.1 s), and the same with P2TR took 9.47 s (spec: 9.7 s). The 7.4 µs figure is in `SPEC_bundle_md1_template_multisig` §6.3.
- **All cost-table arithmetic.** Every row checks out, for example 161³ × 40 = 166.9 M → 20.6 min, 5040 × 6440 = 32.5 M → 4.0 min, 417 × 2000 = 834,000 → 20.9 s or 59 s with P2TR, and 8001 × 40 = 320,040.
- **The printed `bundle` forms, run with the abandon/zoo phrases through `-` and `@env:`.** Each of these exits 0:
  - single-sig `pkh`, `sh(wpkh)`, `wpkh` and `tr`, at standard paths and at the non-standard pairs S6, S7, S9 and S10;
  - an origin of `m/45'`, and `h` hardened steps in both the descriptor and `@N.path`;
  - two seeds with sortedmulti on a shared path, and M4 with independent paths;
  - a seed plus a cosigner given as `@1.xpub`, `@1.fingerprint` and `@1.path`;
  - `--passphrase -` and `--passphrase @env:` (with the passphrase fingerprint `b4e3f5ed`);
  - `@0.ms1=@env:`, and testnet;
  - `tr(NUMS,sortedmulti_a/multi_a/{pk,pk})`, `sh(multi)` and `sh(wsh(multi))`.
- **Template versus descriptor md1.** The md1 from `--template bipNN --account N` is byte-identical to the inline-origin descriptor form for bip44/3, bip49/0, bip84/0, bip84/3 and bip86/2.

---

## Critical

### C1. A `multi`/`sortedmulti` alias gives exit 0 for the wrong wallet, and test row M2 pins it as correct (§5 exit 0, §9 M2, §4 pruning 4)

**Evidence.** The M1 target `bc1qkpcfs6…92dens` (from `wsh(sortedmulti(2,A,Z))`) is matched by template `wsh(multi(2,@0,@1))` with @0 = zoo and @1 = abandon. That is the spec's own row M2, with an expected result of "exactly @0 = zoo, @1 = abandon", which is exit 0 under §5.

The two wallets are different. `alias.py` shows that over receive indices 0..19:
- they agree at 0, 2, 4, 6, 7, 10, 12, 13, 14, 18 and 19;
- they differ at 1, 3, 5, 8, 9, 11, 15, 16 and 17.

So a user with a sortedmulti wallet who passes a `multi` template, which is common because users rarely know the difference, gets exit 0 and a printed `bundle` command. That command mints cards whose wallet shows about half of the real addresses. A restore from those cards misses funds, and the exit code tells the user it is safe to trust. The reverse also happens: a `multi(A,Z)` wallet whose target sits at index 1 is matched by a sortedmulti template.

This contradicts Brian's ruling that exit 4 means "verify before trusting". A single target address cannot tell apart two wallets that coincide at that index.

**Fix.**
1. For every match from a `multi`/`multi_a` template, also evaluate its sorted twin over every target, and the reverse for a sorted template.
2. If the twin also matches all targets, the result is ambiguous (exit 4). Print both wallets, with a note asking for a second address at a different index.
3. Say this generally in §5: exit 0 means "unique among the scripts the search considered", and only when no alias template is also consistent.
4. Change M2's expected result to exit 4 (sorted twin also matches). Add a row: with two targets at indices 0 and 1, `multi` with Z,A gives no match and sortedmulti gives exit 0.
5. Recommend two or more targets for multisig in the no-match and ambiguous text.

---

## Important

### I1. The engine reuse is misdescribed and underspecified (§1 "reuses the same engine", §4 "the engine removes symmetric orders it can prove", §5 "the engine's funds-safety contract")

**Evidence** (`permutation_search.rs`):
- **No full scan by default.** The engine does not scan the whole space: `stop_at = 2` short-circuits at the second match (lines 1151-1158, 1270-1276). `SearchOutcome::Ambiguous` carries no assignments. Only `collect_all = true` (line 1267) disables the short-circuit and returns `Enumerated { assignments }`. §5's "every finding is printed" therefore needs `collect_all`, and the spec never says so.
- **No path dimension.** The engine has no path-tuple dimension. `Enumeration` covers `FullPermutation`, `OwnAnchored` and `OptIn` over a key pool, and `SearchMode::Address` packs only chain and index (`flatten`, line 236). The space here is "slot orders × path tuples × chains × gap", so it needs either a new `Enumeration` variant or a pool of (seed, path) entries with a constraint of one entry per seed. `OptIn`/`p_count` over a pool of k×C entries would also produce self-multisig assignments, which §1 says v1 refuses.
- **No general symmetry removal.** The only symmetry the engine removes is the `sorted` flag (dropping the N! factor). It has no general symmetry analysis.

**Fix.**
1. State the engine mode: `collect_all = true`, then merge candidates with identical scripts across the window (§4 pruning 5) after the scan.
2. Specify the new enumeration over (path tuple, permutation) and its unrank order, and its interaction with `early_exit`, which must stay false.
3. Delete "the engine removes symmetric orders it can prove". Pruning 4 (sorted) is the only one.

### I2. The cost model leaves out the per-candidate taproot tweak for `tr` multisig (§4 multisig table and "Defaults that keep a run under a minute")

**Evidence.** The 7.4 µs per candidate comes from the wsh cost: `translate_pk` plus one sha256 (multisig spec §6.3). A `tr(NUMS, multi_a(...))` candidate also needs a TapLeaf hash and a TapTweak EC multiply. The spec's own measurement puts that multiply at 46 µs (the P2TR row), so a candidate costs roughly 50-55 µs, about 7 times the wsh figure.

The claim "every shared-path multisig up to 6 unsorted keys fits" in under a minute is therefore false for taproot. For example, `multi_a` with 6 keys is 4.6 M candidates × ~53 µs ≈ 4 min on one thread.

**Fix.** Add a taproot row per candidate (measure it). Restate the under-a-minute envelope per script family. The runtime calibration (`calibrate_per_candidate`) will catch the difference at run time, but the defaults decision in question 2 rests on this table.

### I3. The keyed-descriptor fast path through `account_search` misses keys whose origin the descriptor states (§2 "A keyed descriptor or keyed md1 … matched to a key by `account_search`")

**Evidence.** `account_search::match_descriptor_against_seed` only tries `CandidatePaths`, which is the 161-path set, and returns the first match per cosigner. A keyed descriptor whose key origin lies outside that set (a Refugium seat `m/270028'/…`, an `--add-path` account, account 25) leaves the seed unplaced, even though the descriptor states `[fp/path]`.

The addresses of a keyed descriptor do not need the seeds at all. The seeds are only needed to write the `bundle` command.

**Fix.**
1. For keyed input, derive the target addresses straight from the descriptor's keys and confirm the targets with no search.
2. Place each seed by matching its master fingerprint to the stated `[fp/path]` and deriving at that path. Check that the xpub there equals the descriptor's key, and fall back to the candidate set only when the origin is empty.
3. Specify what happens to a seed that matches no key: refuse (exit 2) or report.

### I4. Skipping the slot search for `sortedmulti` is only correct when every slot has the same use-site path and nothing is pinned per slot (§4 pruning 4)

**Evidence.** `resolve_placeholders` returns `use_site_path_overrides`, so per-`@N` multipaths are a supported shape. `bundle` accepts `wsh(sortedmulti(2,[…]@0/<0;1>/*,[…]@1/<2;3>/*))` and exits 0 (run). Swapping keys between slots then changes which child each key contributes, so the addresses change. Mixed `[fp/path]@N` pins break the symmetry the same way.

BIP-387 line 58 also ties the symmetry to "the same child index". The pruning as written would skip the orders that are needed and miss the wallet.

**Fix.** Apply pruning 4 only when every key sits in sorted fragments with identical use-site paths, and the slots are unpinned or pinned to the same path. Otherwise search f! as usual.

### I5. The chain model assumes `<0;1>`, but a descriptor template can say otherwise (§2, §4 pruning 2, §6 `chain:` and `path:`, `--external-only`)

**Evidence.**
- `bundle` accepts `wpkh([…]@0/*)` with no chain step (run: exit 0).
- The lexer carries `multipath_alts` (`PlaceholderOccurrence`), so `<2;3>` and other alternatives are legal.
- Hardened wildcards (`/*h`) also lex (`wildcard_hardened`).

The spec scans chains 0 and 1, prints `path: …/0/0`, and gives `--external-only` a meaning that depends on `<0;1>`.

**Fix.**
1. Derive the chains from the template's multipath alternatives, so that "chain" means "alternative k". For a bare `/*`, use a one-dimension index with no chain.
2. Print the real full path.
3. Refuse `/*h` explicitly with exit 2, since §1 puts hardened leaves out of scope.

### I6. The `m/45'` candidate cannot reproduce a BIP-45 wallet, yet the no-match report will claim BIP-45 was searched (§3, §5 no-match report)

**Evidence.** BIP-45 line 40 gives `m / purpose' / cosigner_index / change / address_index`, and lines 60-85 say each cosigner has its own `cosigner_index` branch. A real BIP-45 (Copay-era) wallet therefore sits at `m/45'/i/<0;1>/*`.

The search tries `@N/<0;1>/*` directly under `m/45'`, which matches no BIP-45 wallet. `bundle` cannot express the extra fixed step either: it refuses `/0/*` with "a fixed single step … is un-representable" (run). Today the `m/45'` entry exists so the search covers what `bundle` mints, not BIP-45 wallets.

**Fix.** In the searched-set report, name the entry "`m/45'` (the `bundle` bare-sh default, not BIP-45 cosigner branches)". Add BIP-45 cosigner-index wallets to "Not covered in v1" with a FOLLOWUP, since md1 cannot represent them.

### I7. The GUI lockstep leaves out the GUI's secret classification and invocation mapping, where this verb family has already leaked once (§8)

**Evidence.**
- mnemonic-gui `src/secrets.rs:473-477` records that xpub-search's `--phrase` was `secret: false` and persisted plaintext, a live leak (FOLLOWUP `xpub-search-inline-phrase-not-secret-classified`).
- `src/persistence.rs:84-95` drops values only by secret name.
- `src/form/invocation.rs:27,888-896` lists every xpub-search subcommand by hand.
- The new verb adds a repeatable `--from <node>=` (a NodeValueComposite) plus `--phrase`, `--ms1` and `--passphrase`.
- The toolkit's GUI manual also has `docs/manual-gui/tests/expected_gui_schema_inventory.json` (it has an `xpub-search-address-of-xpub` entry) and `docs/manual-gui/pinned-upstream.toml`. §8 lists neither.

**Fix.**
1. Add to §8: in the GUI PR, every secret-bearing flag of the new verb is `secret: true` (including the seed-only `--from` composite); add the `invocation.rs` nesting entry; and add a persistence test proving that no seed or passphrase reaches `state.json`.
2. Add the manual-gui inventory JSON and the `pinned-upstream.toml` bump to the toolkit side.

---

## Minor

- **M1 (§2 "`--descriptor` takes what `bundle --descriptor` takes: … or md1 chunks").** This is false. `bundle --descriptor <md1>` exits 2 with "md1 cards are not accepted on --descriptor" (run). The md1 intake comes from `descriptor_intake.rs`, which in turn refuses `@N`. Restate it as: templates through `lex_placeholders`, and keyed descriptors, BIP-388 JSON and md1 through `descriptor_intake`.
- **M2 (§6 "`bundle` refuses it beside a seed slot").** The refusal is per slot ("slot @0 has both secret-bearing input and watch-only input"). A cosigner slot's `@1.path=` beside a seed slot is accepted (run: exit 0), and §6's own cosigner form relies on that. Reword.
- **M3 (§6 bare cosigner xpub "leaves `<…>` placeholders for the last two").** The user does not know these values, so the command can never be run as printed. Without them, `bundle` defaults the origin, warns, and exits 4 (run). Omit the two slots, and print a note that `bundle` will choose the canonical origin and that it must be verified.
- **M4 (§4 pruning 3, §2 Network).** Only mainnet prefixes are named. Classify by the decoded scriptPubKey, not by prefix. Base58 `m`/`n`/`2` are shared by testnet, signet and regtest, and `tb1` by testnet and signet. Say which `--network` values accept which forms, and add a testnet row to the tests.
- **M5 (§5 no-match text).** Only `3…` gets the "a script the search was not given" note. `bc1p` needs it too: taproot script wallets, and multisig whose internal key is BIP-341's recommended `H + rG` (BIP-341 line 157) or any key other than the toolkit's NUMS, cannot be found without the exact descriptor.
- **M6 (§3, §1 Not covered).** Common BIP-39 paths outside the set:
  - `m` (the master key; restore's set includes it);
  - `m/0'` (BIP-32 default account; breadwallet, MultiBit HD);
  - Samourai's high accounts (2147483644'-2147483646').

  Also, Electrum-format seeds are not BIP-39. List these in the no-match hint as `--add-path` suggestions, or include `m` and `m/0'`.
- **M7 (§2 and §5 refusals).** Single-sig with two or more `--from` seeds is unspecified. Either each seed is searched on its own (then "more keys than slots" must not fire), or it is refused. The same goes for `--cosigner` without `--descriptor`. Specify both.
- **M8 (§3 dedupe, §4 cache).** "Deduplicated by path before the search" cannot use a set over the lazy window, which can be up to 8 × 2³¹ entries (`candidate_paths_full_hardened_window_is_lazy`). Dedupe `--add-path` expansions against the standard families analytically. Also bound `--gap-limit` below 2³¹ (non-hardened indexes), and note that the cache of `2 × gap` keys is O(gap) memory per seed and path.
- **M9 (§5 "Refusals … exit 2").** xpub-search's existing refusals are mostly `BadInput`, which exits 1. Name the new or reused exit-2 variant (for example `ModeViolation`, or a new `AddressOfSeedRefusal`, in alphabetical order).
- **M10 (§6 command building, security).** Do not put real secrets into a synthetic `Vec<String>` (ordinary, unzeroized memory) only to redact them afterwards. Build the argv with the reference (`-`, `@env:`) or the placeholder directly, then run `redact_and_quote` as a fail-closed second check.
  - `REDACTED_NOTE` (`bundle.rs:3244`) says "put yours back … or pass one as =- on stdin". Putting a secret back on argv is refused by the guard, and only one value can come from stdin.
  - For this verb, prefer printing `@N.phrase=-` for one seed and `@env:` for the rest, or extend the note to name `@env:VAR`.
- **M11 (§10 Q4, evidence for the ruling).** In mnemonic-refugium, rounds are per seat. `KitCard::seat_derivation_path` uses `self.seats[i].round` (`kits.rs:1502-1515`), and `new_round` (`succ.rs`) sets each seat's round on its own. The seat is the slot (SPEC_refugium line 92: "i the seat (stored `@i`)").
  - So the round is not shared, and the search for slot @i should use only `{32r + i : r ∈ 0..7}`, which is 8 paths per slot, not 256.
  - The path also fixes the slot, which collapses the slot-order search.
  - Separately, check whether any Refugium wallet is single-key at all, which question 4 assumes.
- **M12 (§7).** The argv guard does not cover `--descriptor` or `--descriptor-file`. A keyed descriptor carrying `xprv` or WIF material would be read and could be echoed in an error. Refuse private keys in descriptor intake (exit 2), and never echo the input descriptor.
- **M13 (§5 symmetry).** BIP-341 sorts the two children at every TapBranch (lines 75-76), so any sibling swap at any depth gives the same output key, not only in a two-leaf tree. Also say which of the equivalent md1s the printed command uses, since their wallet-policy ids differ.
- **M14 (§2 `--cosigner <xpub|mk1>`, slot searched).** A multi-chunk mk1 given without a slot has no way to group its chunks (restore groups them by repeating `@N=`). Define the grouping, for example repeating `--cosigner` with an explicit chunk set, or reassembling chunks by their header.
- **M15 (§9 test plan gaps).** Missing rows:
  - a taproot multisig vector (`tr(NUMS,multi_a)` and `tr(NUMS,sortedmulti_a)`);
  - `sh(wsh(multi))` and bare `sh(multi)` targets (`3…`);
  - testnet and regtest;
  - intake by ms1, entropy and seedqr;
  - non-English phrases;
  - a printed command with a passphrase, where the fingerprint becomes `b4e3f5ed` for abandon/TREZOR (run: the printed `[73c5da0a/…]` would fail with "derives master fingerprint b4e3f5ed");
  - the C1 alias row;
  - an I4 row with differing multipaths;
  - keyed descriptors at non-standard origins (I3);
  - an unplaced cosigner across 3 slots.

  M4 with `--independent-paths` is about 1.04 M candidates. Narrow the accounts in that test so the debug-profile suite stays fast.

## Nit

- **N1 (§8).** Give full paths: `docs/manual-gui/src/75-gui-forms/751-mnemonic.md` and `docs/manual-gui/src/90-appendices/99-index-table.md`.
- **N2 (§5 table).** A malformed phrase is the `Bip39` variant (exit 1), not `BadInput`.
- **N3 (§6 JSON).** `seed_index` is 1-based while `slot` is 0-based. Pick one. `searched.candidates` counts child keys in single-sig but candidate scripts in multisig, so name them apart.
- **N4 (§5 table).** Cite where the "draft exit-code spec" lives. It is not in `design/` or the shared folder.
- **N5 (§4).** "Account xpub (3 or 4 hardened steps)" leaves out `m/45'`, which is 1 step.
- **N6 (§2, §4).** `--gap-limit` is an index window `[0, N)`, not BIP-44's "N consecutive unused" rule. Say so in help and output, as `address-of-xpub` does implicitly.
- **N7 (§6).** `bundle`'s redaction note goes to stderr. Say where it goes for this verb in text and JSON mode (JSON already has `bundle_command_redacted`).

## On the open questions (§10)

- **Q1 (shared by default, `--independent-paths` opt-in).** Sound.
- **Q2 (20 accounts, gap 20, both chains).** Sound, but restate the timing claim per script family (I2).
- **Q3 (no early stop).** Sound and needed. Make it `collect_all` (I1). It is also what lets the C1 twin check and the symmetry merge work.
- **Q4 (Refugium).** Answered by evidence in M11: per-seat rounds, and the seat is the slot.
- **Q5 (inline origin always).** Sound. The md1 identity was confirmed for five standard pairs.
- **Q6 and Q7.** Sound. The FOLLOWUP `tk-bh1125-r1-m3-self-multisig-two-matches` exists (`FOLLOWUPS.md:6879`).
- **Q8 (no-match exit 4 for now).** Acceptable only because `result` tells them apart in JSON. Say in the manual's exit table that exit 4 has two meanings until the exit-code spec lands.
- **Missing question.** C1's alias policy: with one target, should a `multi` match be reported as ambiguous with its sorted twin by default? This review recommends yes.
