# mnemonic-toolkit bug-hunt fix branch — independent adversarial review, round 1

Branch `claude/project-thread-utm1aa` (7 commits, `e6fd50c..6e304aa`) against `origin/master`. Clone `/home/user/mnemonic-toolkit`, read-only. No tracked file was edited. Two throw-away probes were created and deleted again: a wc-codec test file and a toolkit example that empties a card's origins. `git status` is clean.

**Verdict: 0 Critical, 3 Important, 5 Minor, 4 Nit.**

- No derivation, origin or address defect was found in the diff. Every canonical shape on both networks derives and records exactly the BIP-44/49/84/86/48 key, checked against the published test vectors or an independent BIP-48 derivation.
- One Important finding is in the diff's own deliverable: the toolkit#3 parity budget is still not exact.
- One Important finding is a user-typed origin that is silently rewritten: an inline `[fp]@N` with no path.
- One Important finding is outside the diff: the toolkit#2 bug class survives in the Sparrow, Electrum and Coldcard emitters.

## What I ran

- Built once with `cargo test --locked -j 2 -p mnemonic-toolkit --test all --no-run`; disk was 13–14 GB free throughout.
- `bughunt_1125`: 18/18 pass.
- Changed existing modules, run from the crate dir because the fixtures are cwd-relative. All pass:

  | Module | Passed |
  |---|---|
  | `cli_shwpkh_canonical_flip` | 11 |
  | `cli_slot_path_reaches_the_card` | 3 |
  | `cli_unified_slot` | 10 |
  | `cli_restore_multisig_general` | 28 |
  | `cli_bundle_slip0132_info` | 9 |
  | `cli_argv_leakage` | 9 |
  | `cli_bundle_origin_path_canon` | 4 |
  | `cli_self_check` | 2 |
  | `bughunt_s2` | 12 |
  | `cli_concrete_nonranged_xpub_reject` | 11 |
  | `cli_export_wallet_*` | 169 |
  | `export_wallet` | 220 |
  | `cli_verify_bundle` | 84 |
  | `cli_restore` | 178 |

- Not run: the full workspace suite, clippy and fmt. These are the implementer's claims and I did not re-verify them.
- Probes used the built `mnemonic` binary and `/home/user/descriptor-mnemonic/target/debug/md` as an independent address oracle (`md address`, `md decode`).

## Q1 — #1 mint: canonical shapes, mainnet and testnet

**Single-sig.** I minted each shape with `abandon … about`, using `bundle --descriptor '<shape>(@0/<0;1>/*)' --slot @0.phrase=…`, then derived the first receive address with `restore --md1 … --from` or `md address`. Every result is the BIP test vector:

| Shape | Origin written | First address | Vector |
|---|---|---|---|
| pkh mainnet | 44'/0'/0' | 1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA | BIP-44 ✓ |
| sh(wpkh) mainnet | 49'/0'/0' | 37VucYSaXLCAsxYyAPfbSi9eh4iEcbShgf | BIP-49 ✓ |
| wpkh mainnet | 84'/0'/0' | bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu | BIP-84 ✓ |
| tr mainnet | 86'/0'/0' | bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr | BIP-86 ✓ |
| pkh testnet | 44'/1'/0' | mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV | ✓ |
| sh(wpkh) testnet | 49'/1'/0' | 2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2 | BIP-49 ✓ |
| wpkh testnet | 84'/1'/0' | tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl | BIP-84 ✓ |
| tr testnet | 86'/1'/0' | tb1p8wpt9v4frpf3tkn0srd97pksgsxc5hs52lafxwru9kgeephvs7rqlqt9zj | BIP-86 ✓ |

**Multisig.** I minted 2-of-2 cards with seeds A and B, then compared them with `md address --template … --key @i=<convert --path m/48'/c'/0'/{2,1}' xpub>`, an independent derivation:

| Shape | Network | Card | Independent |
|---|---|---|---|
| wsh(sortedmulti) | mainnet | bc1qsks3qr92vdnr80q6y9vv6h4qwlzza9w8ts2pjp74wjj6ahvud5dsc3vhxe | identical |
| sh(wsh(sortedmulti)) | mainnet | 36s1UDQCExPVocyN48jYVAq2GxgBkPQxdN | identical |
| wsh(sortedmulti) | testnet | tb1q8ehr5wd4f8v24vhrcvhsrmc9wrgm05ts2ux5l26ec4vg9lrhnc2sevm3ye | identical |
| sh(wsh(sortedmulti)) | testnet | 2NE9iKakaTBaYTs4CQZk3Zovs3yEP9hLxwP | identical |

The `info:` line names 48'/c'/0'/2' or /1' correctly, and `md decode` shows the written origins.

**Explicit paths.**
- An inline `[fp/48'/0'/3'/2']@0` survives, and only the bare `@1` is filled.
- A `--slot @N.path=` with a seed on a canonical shape was already refused before this branch (`bundle.rs:1655`).
- `@N.path=m` is refused (exit 2).
- **Exception (I2 below):** an inline fingerprint-only origin `[fp]@N` is silently rewritten.

**Non-canonical and template mode are unchanged.**
- The fill is gated on `!is_non_canonical` (`bundle.rs:2716`).
- `--template bip84` and `wsh-sortedmulti` (bip87) cards still carry explicit `84'/c'/0'` and `87'/c'/0'` origins, and their addresses are unchanged (BIP-84 vector again).
- `verify-bundle` of fresh template bundles (single-sig phrase, multisig xpub, hybrid) returns `result: ok`.
- `verify-bundle --descriptor` of fresh canonical wpkh and sh(wsh) bundles returns `result: ok`: Verify mode fills the same origin.
- `--account != 0` is refused for canonical descriptors, so the fill dropping `--account` is not reachable (`bundle.rs:1627`).
- md-codec's encoder does not elide an explicit origin (`vendor/md-codec/src/encode.rs`), so the written origin really is on the wire.

## Q2 — #1/#2 read

- **The match is exact.** `xpub_to_65(xpub) == key65` compares chain code and public key (`restore.rs`, empty-origin block after the own-position loop).
- **The fingerprint is checked after a match.** A mismatch with the card is routed to the existing mismatch path.
- **Coin follows `--network`.** I took an empty-origin override card holding B's 48'/0'/0'/2' key and restored it with `--network testnet`. Restore refused with exit 1 and a message naming `--network`. With mainnet, restore found 48'/0'/0'/2' and warned.
- **Single-sig empty-origin cards (faithful arm).**
  - Without a seed: the key prints as `[73c5da0a]tpub…` with the warning, and the address is unchanged.
  - With a seed: restore finds `m/84'/1'/0'`, warns, and prints `[73c5da0a/84'/1'/0']`.
  - Override (faithful-arm) multisig cards behave the same way.
- **The own-position loop skips empty-origin cosigners**, so a master-key match is never accepted silently as if the card said `m`.
- **More than one match is refused.** This also covers a seed that is two empty-origin cosigners (M3).
- **The no-seed path guesses nothing.** `key_origin_str` is now `slot.bracketed_origin()` only (`wallet_export/pipeline.rs:45`); the template fallback is gone. That function no longer fills a default anywhere. Other emitters still do (I3).
- **Existing cards read with unchanged addresses.**
  - Addresses are computed before the search from the card's xpubs, and they do not depend on the origin.
  - The pre-change mk1 fixtures (`PRE_X4_WIF_MK1`, `PRE_TK1_NO_ORIGIN_MK1`) still decode through `inspect`.
  - Template-mode cards are untouched.

## Q3 — #5 / #4 / X4 / #8 / #6: legitimate input newly refused? secret leaks?

- **#5 (`check_defaulted_xpub_origin`, `bundle.rs:550`).** It compares depth with the path length and the child number (hardened flag included) with the last component.
  - An xpub for account N under `--account N` passes.
  - SLIP-0132 zpub/ypub forms keep their header, and `cli_bundle_slip0132_info` passes.
  - A testnet tpub at 84'/1'/0' under `--network testnet` passes.
  - Only a genuinely contradictory header is refused. Fixture xpubs I decoded: `xpub6FQya…` and `xpub6DnEB…` are depth 4 with child 2', so the old `[fp/87'/0'/0']` in the Sparrow fixture was a wrong origin. `xpub6Buxw…` is depth 3 with child 0', so it still takes 87'.
  - Residual limitation (M4): purpose and coin cannot be checked.
- **#4.** `@N.fingerprint=` beside a seed was legal only together with `Path` (`slot_input.rs::is_legal_set`), and template mode never read either. Refusing them removes no working check.
- **verify-bundle of existing good bundles is OK** (three template cases and two canonical descriptor cases tested). A bundle minted earlier with a contradictory xpub default, or with a seed-slot path, now refuses with exit 2. That is decision 5 in the PR body, and the refusal is arguably correct because such a card recorded a wrong origin.
- **X4 / #8.** No legitimate mint is lost, because a WIF under a ranged template never described the WIF's own address. `verify-bundle` still reads WIF bundles. `export-wallet` already refused `wif` as secret.
- **#6.** `parse_account_arg` is on all seven `--account` flags. The `restore` list is validated per element. 2^31−1 still parses.
- **No secret leaks.** I checked every new refusal: the #4 message (no values), #5 (depth and child only), X4 (no WIF), #8 (`NetworkMismatch` carries names and context only, `network.rs:94`) and #6 (the account number).

## Q4 — #3 budget: is the stated text now exact? **No** (I1)

I probed the K=58 (73-byte mk1) card from `parity_budget.rs`: blocks of [8 × 7, 2], m ∈ {0, 1, 2, 8, 9, 10}, every word position, then all 2047 substitutions on chosen words:

- **Header words 0–4:** never repair. ✓
- **Ledger words 5–10 and the two stop words:** survive at any m, even m=0. The text does not mention them; harmless.
- **Data words:** repair at m = block size. ✓
- **Parity words:** cost 2. ✓ (m=1 fails, and 2 parity errors plus one block need block + 4.)
- **Checkpoint words: wrong.**
  - At m=9 (block + 1), checkpoint word 19 repairs 2016 of 2047 substitutions and **refuses 31**; word 46 is the same.
  - At m=10, all 2047 repair.
  - The 31 are the substitutions that keep the `0b101` marker and the block index and change only the CRC-5 field (`sync.rs:197`). The word still parses as a checkpoint, so it is an undetected RS error costing 2, on top of the erased block. The real cost is block + 2.
- No wrong payload in any trial.

## Findings

### Important

**I1 — The toolkit#3 parity budget is still not exact. A checkpoint word can cost block + 2, so "m = block size + 1 survives any one mis-stamped non-header word" is false.**

- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/word_card.rs:81-88` (help: "a mis-stamped checkpoint word its block plus itself … m must exceed the block size").
  - `docs/manual/src/40-cli-reference/41-mnemonic.md`, *Repair budget*: "costs the block size plus one … So `m` = block size + 1 survives one mis-stamped word anywhere outside the header … `--parity-words 9` survives any one".
  - `docs/manual-gui/src/40-mnemonic/4n-word-card.md:84-90` ("for a 2-chunk `mk1` card, `9`", and step 4 "say `9`").
  - `crates/wc-codec/tests/parity_budget.rs`: `checkpoint_substitution_costs_block_plus_one`, `block_size_plus_one_survives_any_single_non_header_word`.
- **Failure scenario:** an engraver follows the manual and stamps a 2-chunk mk1 word card with `--parity-words 9`. One slip turns a checkpoint word into another word with the same marker and index (31/2047 of substitutions on that word). The decode refuses, so the card the manual promised survives one error does not. Fail-safe: never a wrong payload.
- **Why the tests miss it:** both tests substitute with a single delta (`+1029`), which always breaks the marker or index.
- **Fix:** state that a checkpoint word costs block + 1, or block + 2 when only its CRC field changes; make the single-error guarantee m ≥ block + 2 (10 for this card, and 18 → 20 for "any two"); test every delta on a checkpoint word.

**I2 — An inline fingerprint-only origin `[fp]@N` on a canonical shape is silently rewritten to the canonical origin, and the notice says "no origin supplied".**

- **Where:** the fill loop at `crates/mnemonic-toolkit/src/cmd/bundle.rs` (~2716-2738, `if !op.components.is_empty() { continue; }`). It cannot tell `[fp]@N` (BIP-380: the key is the master of `fp`) from a bare `@N`.
- **Repro:**

  ```
  bundle --network mainnet --descriptor 'wpkh([73c5da0a]@0/<0;1>/*)' --slot @0.phrase=<A>
  ```

  It exits 0, prints `info: no origin supplied for @0; … m/84'/0'/0'`, and the card carries `[73c5da0a/84'/0'/0']`. `wsh(sortedmulti(2,[73c5da0a]@0/…,[b8688df1]@1/…))` behaves the same way and gets 48'/0'/0'/2'.
- **Before this branch** the key was derived at `m`.
- **Inconsistent with its own sibling:** the explicit `--slot @N.path=m` (the same request through another channel) is refused with "cannot be recorded".
- **Risk:** the card is self-consistent, so no wrong address. But an operator-typed origin is replaced without being told, and the stderr text is false. This is exactly Q1's "explicitly supplied path overridden".
- **Fix:** in Emit mode, when the inline placeholder carried a fingerprint with an empty path, refuse the same way as `@N.path=m`.

**I3 (pre-existing, outside the diff, same class as toolkit#2) — the Sparrow, Electrum and Coldcard emitters still substitute the template default origin.**

- **Where:**
  - `wallet_export/sparrow.rs:245-262`: single-sig always uses `template.origin_path_str`; multisig falls back to it when the path is empty.
  - `wallet_export/electrum.rs:122` (single-sig always) and `:175-185` (multisig fallback).
  - `wallet_export/coldcard.rs:201` (single-sig `deriv`).
- **Repro (single-sig):**

  ```
  export-wallet --format sparrow|electrum|coldcard --template bip84 \
    --slot @0.xpub=<A's m/84'/0'/7' xpub> --slot @0.fingerprint=73c5da0a \
    --slot @0.path=m/84'/0'/7'
  ```

  All three write `derivation`/`deriv` `m/84'/0'/0'`. The typed origin is ignored, and a signer asked for 84'/0'/0' derives a different key; the wallet cannot sign until fixed.
- **Repro (multisig, after this branch):** `--template wsh-sortedmulti` with `@0.path=m` on a master xpub. `bitcoin-core` now prints `[73c5da0a]`, while Sparrow and Electrum print `"derivation": "m/87'/0'/0'"` for the same slot.
- **Why it is in this review:** it is not a regression (`key_origin_str` used to agree with the default), but the PR's toolkit#2 claim ("never substitutes the template default") holds only for `key_origin_str`. Fold it, or file a FOLLOWUP before claiming toolkit#2 closed.

### Minor

- **M1 — `restore --json` hides that an origin was found by search, not read.**
  - `cosigner_keys[].origin` is the found path and `note` is `verified`; an unresolved key has `origin: ""`, which a consumer may read as `m`. The loud warning is stderr-only. Location: `restore.rs`, the `--json` envelope (~4590).
  - Scenario: mnemonic-gui or a script imports the JSON descriptor and never surfaces that the path is not on the card.
  - Fix: add an `origin_source: "card" | "search" | "unknown"` field.
- **M2 — The no-seed rendering prints `[fp]xpub661My…`.** That is a depth-0 serialization that claims to be the master of `fp`, while hash160 of the key is not `fp`. It is internally contradictory to signers. This is the PR's decision item 4; I recommend omitting the descriptor or refusing rather than printing it with exit 0.
- **M3 — The "more than one match" refusal is misleading for a self-multisig card.** If one seed holds two empty-origin cosigners at different paths, restore refuses with "more than one path of this seed produces an empty-origin key". That is conservative, but the user cannot complete the restore at all. Restore could resolve each cosigner independently when each has exactly one match.
- **M4 — #5 cannot see purpose or coin.** A BIP-49 account xpub (depth 3, child 0') under `--template bip84` with no path is still accepted and exported as `wpkh([00000000/84'/0'/0']…)`, which I verified. This is pre-existing and documented as a depth/child check, but the manual's "takes the default only when the header agrees" may be read as "only when correct".
- **M5 — The rebuilt descriptor skips the round-trip guard.** After a successful search, `descriptor = build_desc(&slots, &d)` is rebuilt but not re-run through the parse→print round-trip guard, which ran on the pre-search string. Low risk: the same builders are used.

### Nit

- **N1.** `tests/cli_shwpkh_canonical_flip.rs`: the doc comment on `non_canonical_descriptor_still_gets_default_path_notice` still says flip #1 asserts the notice is ABSENT for canonical shapes. That is now false; `DEFAULT_PATH_NOTICE` matches both notices.
- **N2.** `verify_bundle.rs` ~1662: the comment "`defaulted_indices` is discarded: verify-bundle emits no notice" is stale.
- **N3.** `_network` / `_account` are kept as dead parameters in `bip388::format_bip388_wallet_policy` and `pipeline::build_descriptor_string_inner`; they could be removed.
- **N4.** The `emit_canonical_origin_notice` text "no origin supplied" is wrong when an inline `[fp]` was given (part of I2).

## Q5 — Do the tests assert what they claim, and are the changed expectations justified?

**New tests that fail without the fix** (by inspection of the pre-fix behaviour):
- tk1 tests: the origin used to be empty or master.
- tk2 no-seed: the pre-fix output printed `[fp/87'/0'/0']` through the `key_origin_str` fallback.
- tk2 with a seed: there was no search before; the old code matched at the empty path, i.e. `m`, and the 48' key would not match.
- tk4 and tk5: no refusal before.
- tk6: panicked before (not exit 64).
- X4: minted before.
- The #8 unit tests: `Ok` before.

The `parity_budget.rs` tests are characterization tests (the decoder is unchanged), and two of them over-claim (I1).

**Changed expectations, each checked:**
- **`DIVERGENT_WSH_MULTI_CHAIN0_IDX0_GOLDEN`** re-captured as `bc1qrce3xdgthdumu3u6h2kucc0mnxlwat3sk5yvznj7avmks6ghv5mq0rff63`. I recomputed it independently from `convert --path m/48'/0'/0'/2'` xpubs of C0 and C1 with `md address --template 'wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))'`, and it is **identical**. The old value was the master-key wallet. Justified.
- **Sparrow fixture** `sparrow_tr_multi_a_nums_2of3.json`: 87' → 48'/0'/0'/2' for two cosigners whose xpub headers are depth 4 / child 2'. Keys unchanged. Justified: the old fixture pinned a wrong origin.
- **`cli_export_wallet_*` and `bughunt_s2`:** `@N.path=m/48'/0'/0'/2'` was added only to refusal-path tests or non-byte-exact tests, so they still reach the refusal under test. No other golden output changed.
- **`cli_shwpkh_canonical_flip` flips #1 and #4, and `cli_slot_path_reaches_the_card`:** they reverse deliberate prior pins (empty or master). This is per Brian's ruling, with the SPEC amendment added. Justified, and the new assertions pin 49'/84'/48' plus the notice.
- **WIF success tests** were turned into refusal tests, and the decode coverage is kept by pre-change fixtures. Acceptable. I did not independently verify that the fixture strings were captured from the pre-change binary, but they decode and show the claimed origins.
