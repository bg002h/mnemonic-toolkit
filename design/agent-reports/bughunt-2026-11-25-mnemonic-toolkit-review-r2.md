# mnemonic-toolkit bug-hunt fold: independent adversarial review, round 2

Branch `claude/project-thread-utm1aa`, range `6e304aa..4eaf96a` (11 commits). Clone `/home/user/mnemonic-toolkit`, read-only: no tracked file was edited, and `git status` is clean. I rebuilt `target/debug/mnemonic` at HEAD (`cargo build --locked -j 2`, `CARGO_INCREMENTAL=0`). Probes ran from the scratchpad (`r2t/`, `r2py/`).

**Verdict: 0 Critical, 2 Important, 6 Minor, 5 Nit.**

- The three r1 Importants are fixed as r1 scoped them:
  - I1: the budget is now exact.
  - I2: refused on canonical shapes.
  - I3: every emitter's derivation string is fixed. Coldcard's `account` field is acceptable as Minor.
- Every derivation and address the ruling produces matches an independent BIP-32 derivation (Python, below), on mainnet and testnet.
- The m/45' route works end to end. Its quoting is safe, and it never echoes a secret.
- The two Importants are both side effects of moving the default:
  - **xpub-search** cannot find keys at the new BIP-87 default, and the PR says it agrees with `bundle`.
  - **A foreign xpub slot** on a non-canonical shape gets the BIP-87 origin without a header check. A Sparrow-style `48'/…/3'` taproot cosigner key is therefore recorded at `87'/…`, a false origin on the card.

## What I ran (targeted only, as asked)

- **Integration tests** (`tests/all.rs` binary, from the crate dir), all pass:

  | Module | Passed |
  |---|---|
  | `bughunt_1125` (incl. `_r1`) | 29 |
  | `cli_bundle_h12_taproot_origin` | 4 |
  | `cli_non_canonical_descriptor` | 21 |
  | `cli_concrete_nonranged_xpub_reject` | 11 |
  | `cli_restore_multisig_general` | 28 |
  | `cli_shwpkh_canonical_flip` | 11 |
  | `cli_slot_path_reaches_the_card` | 3 |
  | `cli_unrestorable_shape_advisory` | 13 |
  | `bughunt_s2` | 12 |
  | `bughunt_s4` | 7 (6 ignored) |
  | `cli_verify_md1_origin_match` | 3 |
  | `cli_verify_bundle_multi_cosigner_mk1` | 8 |
  | `cli_export_wallet*` | 191 |

- **wc-codec `parity_budget`:** 9 of 9 pass.
- **Lib unit tests:**
  - `default_origin_family_follows_the_bip_defaults_ruling`
  - `rerun_command_never_echoes_an_inline_secret`
  - the two `descriptor_intake` default tests

  All pass.
- **Independent oracle:**
  - `scratchpad/r2py/b32.py` is a pure-Python BIP-32 implementation (secp256k1, HMAC-SHA512, Base58Check). It reproduces the BIP-44 test vector `xpub6BosfCnifzxc…`.
  - `r2py/tr.py` adds the BIP-341 tweak, a TapLeaf for `multi_a`, and bech32m.

## Q1: are the r1 Importants fixed?

### I1 (word-card budget): fixed, and the stated budget is now exact

- **The help text** (`word_card.rs:81-89`) now gives the costs:
  - checkpoint word = block + 1, or block + 2 for a CRC-only change;
  - parity word = 2;
  - data word = its block;
  - one error needs `m` ≥ block + 2.
- **The manual and manual-gui** now say `10` and `20` for a 2-chunk mk1 card.
- **The tests:**
  - `checkpoint_substitution_costs_block_plus_one_or_plus_two` tries all 2047 substitutions of the first checkpoint word at block + 1 and at block + 2. It asserts that exactly the 31 CRC-only values refuse at + 1, that all repair at + 2, and that no payload is ever wrong.
  - Every non-header position is also tried at stride + 2 with a breaking delta and a CRC-only delta.
  - There is a pair test for 2 × (block + 2).
- **Can a single substitution still exceed block + 2?** I found none, and none is expected:
  - A data-word slip whose block CRC still passes (1/32) is an undetected RS error. It costs 2, which is never more than block + 2.
  - A checkpoint slip that changes the marker or index costs block + 1 (the exhaustive test).
  - A parity slip costs 2.
  - Ledger and stop words cost nothing (r1's probe).
  - "Any two in different blocks" at 20 is therefore also an upper bound.

### I2 (inline `[fp]@N`): fixed for canonical shapes

- `wpkh([73c5da0a]@0/<0;1>/*)` and `wsh(sortedmulti(2,[73c5da0a]@0/…,@1/…))` exit 2 with the new text.
- `[fp]@N` together with `--slot @N.path=` is still accepted.
- The canonical notice now says "no origin path supplied".
- The same rewrite survives on **non-canonical** shapes (Minor M6 below).

### I3 (emitters substitute the template default): fixed for every derivation string

- Sparrow, Electrum (standard and multisig) and Coldcard single-sig now use `ResolvedSlot::origin_path_m()`.
- I checked these cases:
  - **Typed path** (`@0.path=m/84'/0'/7'`): all three write `m/84'/0'/7'`.
  - **No path** (default bound through the header check): all three write `m/84'/0'/0'`. The default still reaches the file, because the slot's path is now populated upstream.
  - **Master key:** `m`.
- No other emitter calls `origin_path_str`. Jade and Specter go through `key_origin_str`. `bip388` passes `inputs.account` only to the template renderer.
- **The Coldcard `account` field** still prints `inputs.account`. A file with `"deriv": "m/84'/0'/7'"` therefore says `"account": 0` (probe). This is acceptable as **Minor**:
  - The fields that select the key (`deriv`, `xpub`, `xfp`) are now right.
  - `account` is a descriptive field of Coldcard's own export.
  - A consumer that rebuilds a path from `account` instead of reading `deriv` fails closed: it derives a different key and cannot sign. It never shows wrong addresses.
  - It is logged at M (`tk-bh1125-r1-i3-coldcard-account-field`).
  - Fix: take `account` from the third component of the slot path when the purpose and coin match the template, else refuse.

## Q2: are addresses and origins right? (checked independently)

Seeds: A = `abandon…about`, B = `zoo…wrong`, and C1 = `legal winner…yellow` for the restore golden.

| Check | Toolkit | Independent (Python BIP-32 / BIP-341) |
|---|---|---|
| A at m/87'/0'/0' | `xpub6DBjiYnc4ewKti13…` | identical |
| A at m/87'/1'/0' (testnet tr bundle) | `tpubDCChhoz5Qdrkn7Z7…` | identical |
| A at m/48'/0'/0'/1' (sh(wsh) default, mainnet) | `xpub6DkFAXWQ2dHx…` | identical (testnet `tpubDFH9dgzveyD8…` identical) |
| A at m/45' | `xpub68jrRzQopSUSiczu…` (convert) | identical; restore prints the same key with parent fingerprint 00000000 (pre-existing compact-form rendering, not in this diff) |
| bare `sh(sortedmulti(2,A,B))` default, mainnet, first receive | `3QWZxAxBAc7CheNe9MvbAjPECogbREKvXE` | identical (87'/0'/0') |
| same, testnet | `2MwgXuhbvuf6VBpou1t7dcYJoDcvZLgWF6V` | identical (87'/1'/0') |
| same via the printed m/45' command | `3Q6xNscq5ukuyYEBY2wiGATs6HrsWGmEnw` | identical (m/45'/{0}/0) |
| `DIVERGENT_TR_MULTI_A_CHAIN0_IDX0_GOLDEN` (re-captured) | `bc1psew7pgfzc99qec0s9lrdddj0qp00y49hjmj98mh9m9z4rhxmwl6s9w0q0e` | identical at 87'/0'/0' (NUMS H, multi_a(2, C0/0/0, C1/2/0)); the old value `bc1p6ufc…` reproduces at 48'/0'/0'/3', so the change is exactly the ruling |

Answers to the specific questions:

- **Is 87'/1' right on testnet?** Yes. BIP-87: "Coin type `0` for mainnet and `1` for testnets (testnet, regtest, and signet)". `CliNetwork::coin_type` returns 1 for testnet, signet and regtest (`network.rs:22-27`).
- **Does xpub-search find `sh(wsh)` keys at 1'?** Yes. With A's `48'/0'/0'/1'` key it prints `match: cosigner @0  m/48'/0'/0'/1'  (template=bip48-sh-wsh, account=0)`. But it always did: the default family in `descriptor_intake.rs` only changes the notice. The search runs over `candidate_paths.rs`, which never contained 87' or 45'. See I-A.
- **Does the printed m/45' command mint at m/45', and does verify-bundle accept it?**
  - Yes. Run as printed (secrets put back), it mints `@0: [73c5da0a/45']`, `@1: [3f635a63/45']` (`md decode`).
  - `restore --from` verifies @0 at `[45']`.
  - `verify-bundle` returns `result: ok` **only when it is given the same `--slot @N.path=m/45'`**. Without the path it fails with an xpub/path mismatch, and nothing tells the user this (M3).
  - The r1 test `b2_bare_sh_multisig_defaults_to_bip87_with_a_runnable_m45_command` really runs the printed line through `sh -c` and verifies.
  - The existing override was sufficient; no new flag was needed.
- **Is the shell quoting safe?** Yes:
  - `shell_quote` leaves an argument bare only when it consists entirely of `[A-Za-z0-9@%+=:,./_-]`. Everything else is single-quoted, with `'` written as `'\''`.
  - That neutralises `$`, backticks, `;`, `|`, `&`, globs, `~`, `#`, `!` and newlines for any POSIX shell.
  - Slot values, descriptors and fingerprints cannot inject.
- **Does it ever print a secret?** No:
  - The redaction is fail-closed. Only the `xpub`, `master_xpub`, `fingerprint` and `path` subkeys and the `=-` form are kept, and `--passphrase` is always replaced. This covers both `--slot X` and `--slot=X`.
  - Every other flag `bundle` takes is non-secret.
  - A literal xprv in `--descriptor` is refused by the lexer before the notice ("public keys must be 64, 66 or 130 characters").
  - An xprv in an `xpub` or `master_xpub` slot is refused before the notice (exit 1 or 2), and stderr does not contain it.

## Q3: newly refused or newly accepted

- **Bare `@N` refusal (B4) does not touch reading:**
  - A bundle minted as `wsh(and_v(v:pk(@0/*),pk(@1/*)))` still passes `verify-bundle --descriptor 'wsh(and_v(v:pk(@0),pk(@1)))'`. The bare form is accepted in verify: `result: ok`.
  - `b4_explicit_derivations_still_mint_and_restore` passes.
  - `restore` and `inspect` take md1 only and never reach the lexer.
  - `md encode` confirms the premise: a bare `@0` "would describe `@0/*`".
- **Rewritten test files:**
  - Across the 20 changed test files, the only golden literal that changed is `DIVERGENT_TR_MULTI_A_CHAIN0_IDX0_GOLDEN`. That change comes from the ruling (B3), not from the `@N` → `@N/*` rewrite, and I reproduced both its old and new values independently.
  - Every other edit is `@N` → `@N/*`, which mints the same card.
  - So the claim holds: the bare-`@N` rewrite left every golden byte-identical.
- **Newly accepted and wrong:** I-B below. A foreign xpub's real origin is replaced by the new BIP-87 default with exit 0.

## Findings

### Important

**I-A: `xpub-search` never searches the new default, BIP-87 `m/87'/c'/a'` (nor m/45'). Its notice says the opposite, and the PR claims agreement.**

- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/xpub_search/candidate_paths.rs:114-121` (`SINGLE_SIG` 44/49/84/86, `MULTISIG` = BIP-48 1'/2'/3' only) and `:127-182`.
  - `descriptor_intake.rs:304-320` (`default_family`, used only for the notice; `derivation_path_anno` is never consumed, see the struct doc at :33).
- **Repro:**
  - Run `xpub-search account-of-descriptor --phrase-stdin` with seed A and `tr(<B 87'>/<0;1>/*,sortedmulti_a(1,<A 87'>/<0;1>/*))`, or with bare `sh(sortedmulti(1,<A 87'>…,<B 87'>…))`.
  - It prints `info: …defaulting origin path for @0,@1 to m/87'/0'/0' (BIP-87)`, then `error: no match in searched set … paths searched=28`. A's key **is** at 87'/0'/0'.
  - A's m/45' key also gives no match.
  - `sh(wsh)` at 1' matches, but that worked before this branch.
- **Why it is Important:**
  - It is a regression for taproot. Before this fold, `bundle --descriptor tr(...multi_a...)` minted at 48'/c'/a'/3', which xpub-search searched (`bip48-tr-multi-a`). The same card minted now is not findable without `--add-path`.
  - The ruling says "bundle+verify-bundle+xpub-search together".
  - The PR body (B1, B3: "bundle, verify-bundle and xpub-search agree") and FOLLOWUPS `tk-bh1125-decision-xpub-search-sh-wsh` (resolved: "share `template::DefaultOriginFamily`") describe the notice, not the search.
  - The new test `b1_b2_b3_xpub_search_default_follows_the_ruling` asserts only the notice text, not a match. It even builds every descriptor from 87' keys and never checks that they are found.
  - Template-mode BIP-87 cards (`wsh-sortedmulti`, `tr-multi-a`) were never findable either (pre-existing). This fold makes BIP-87 the default in descriptor mode too.
- **Failure scenario:** a user holds a taproot multisig card minted by this release and wants to confirm which account a seed is in. `xpub-search` names m/87'/0'/0' and then says no match: a false negative from the recovery tool.
- **Fix:**
  - Add a `bip87` family `m/87'/<coin>'/<a>'` (accounts lo..hi) and a single `bip45` `m/45'` candidate to `CandidatePaths::iter` and `len`.
  - Make the b1/b2/b3 test assert `match: cosigner @0  m/87'/0'/0'` (and `m/45'` for a 45' key).
  - Update the manual's xpub-search candidate list (`41-mnemonic.md` §xpub-search and manual-gui `4d-*`).
  - Correct the PR body and reopen the FOLLOWUP.

**I-B: a foreign `@N.xpub=` slot on a non-canonical shape now gets the BIP-87 origin without the toolkit#5 header check. Taproot cosigner xpubs at the Sparrow / previous-default `48'/c'/a'/3'` are recorded at `87'/c'/a'`, a false origin, with exit 0.**

- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:1860-1868`. `check_defaulted_xpub_origin` runs only `if bound_paths.canonical_filled.contains(&idx)`, never for `bound_paths.defaulted`. The same applies to the shared verify path.
- **Repro:**

  ```
  bundle --network mainnet --descriptor 'tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))' \
    --slot @0.phrase=<A> --slot @1.xpub=<B at m/48'/0'/0'/3', depth 4> \
    --slot @1.fingerprint=3f635a63
  ```

  - It exits 0 with `info: … defaulting to m/87'/0'/0' (BIP-87)`.
  - `md decode` shows `@1: [3f635a63/87'/0'/0']` for a depth-4 key whose child number is 3'.
  - The same happens with bare `sh(sortedmulti)` and a `48'/0'/0'/1'` xpub. That case gets the P2SH warning, which says "derived at" although an xpub is not derived.
- **Why it is Important:**
  - Addresses are right, because the xpub is used. But the card's origin, and every PSBT and wallet file built from it, claims `87'/0'/0'` for that key. The cosigner's hardware signer derives 87'/0'/0'/0/i, does not find the pubkey, and refuses to sign. With 2-of-2, or with several such keys, the funds are stuck until someone diagnoses the origin.
  - This is the toolkit#5 bug class, which the branch fixed for template mode and canonical shapes.
  - The FOLLOWUP `tk-bh1125-noncanonical-xpub-header` logged it as pre-existing at M. However, this fold moved the default **away** from where real foreign taproot multisig xpubs live: Sparrow, Coldcard and Jade export 48'/…/3'. Before, the common case was accidentally right; now it is wrong. The same applies to anyone whose bare-P2SH cosigner xpub sat at the old 1' default.
- **Fix:**
  - Run `check_defaulted_xpub_origin(idx, &xpub, &anno_path)` also when `bound_paths.defaulted.contains(&(idx as u8))`, in Emit and Verify alike. The depth 4 ≠ 3 case and the child-number 3' ≠ 0' case then refuse, asking for `--slot @N.path=`.
  - Add a row: a tr default plus a `48'/0'/0'/3'` xpub refuses; an `87'/0'/0'` xpub is accepted.
  - Promote the FOLLOWUP to I and close it in this PR.

### Minor

**M1: Coldcard generic-JSON `account` field.**
- Where: `wallet_export/coldcard.rs:222`.
- What: it is still `inputs.account`, while `deriv` is the slot's path (`"account": 0` beside `m/84'/0'/7'`).
- Why Minor: the key-selecting fields are right, and the failure is fail-closed.
- Status: already logged at M, and that grade is accepted.

**M2: the manual's m/45' example and BIP-87 quote are inaccurate.**
- **Broken example command** (`docs/manual/src/40-cli-reference/41-mnemonic.md:698`):
  - It ends `--slot '@0.path=m/45'\''`, which leaves an unterminated quote when pasted.
  - The tool prints `'@0.path=m/45'\'''`.
  - The example also quotes `'@0.phrase=-'`, which the tool prints bare.
  - Fix: copy a real line.
- **Quote not in BIP-87:**
  - `41-mnemonic.md:675` attributes "the same derivation path may be used for all multisig script types" to BIP-87. The doc comment at `tests/cli_bundle_h12_taproot_origin.rs:7-9` does the same.
  - That sentence is not in BIP-87 (checked against `scratchpad/bips/bip-0087.mediawiki`).
  - Fix: use a real BIP-87 sentence, for example the Backwards Compatibility sentence `template.rs` already quotes: "Any script that is supported by descriptors (and the specific wallet implementation) is compatible with this BIP." This breaks the repo rule that external protocol facts are checked against source text.

**M3: the m/45' alternative has gaps the warning does not mention.**
- **`--account` is ignored:**
  - With `--account 1`, the printed command mints a bundle byte-identical to `--account 0` (probe: `md1` equal), because m/45' has no account level.
  - Yet the same warning, one line earlier, says "For a second wallet from the same seed, use a different --account."
  - Mainnet and testnet also share the m/45' key.
- **verify-bundle needs the path too:** it accepts the m/45' bundle only with the same `--slot @N.path=m/45'`. Re-verifying with the original slot set fails with xpub and path mismatches, and neither the warning nor the manual says this.
- **Fix:** add one line under the printed command:

  ```
  m/45' has no coin or account level (every --account and network gives the same keys); pass the same --slot @N.path=m/45' to verify-bundle.
  ```

  Mirror it in `41-mnemonic.md`.

**M4: the manual claims old cards "still decode and verify as before"** (`41-mnemonic.md:709-710`).
- `verify-bundle --descriptor` re-run with the original command line on a pre-ruling taproot bundle (or a bare-sh bundle at 1') now rebuilds at 87' and fails (`mk1_xpub_match … fail`, `md1_xpub_match: fail`). I probed this with a bundle minted at `48'/0'/0'/3'`.
- `restore` is unaffected.
- The owner says there are no existing card holders, so this is Minor.
- Fix: say that verify-bundle of such a card needs `--slot @N.path=m/48'/<c>'/<a>'/3'` (or `/1'`). Optionally, add the pre-2026-11-25 default to verify-bundle's info line when the default-rebuilt xpubs mismatch.

**M5 (Q4): BIP-87 for non-multisig `tr` trees and non-multisig `sh(...)`. Keep the default, but tell the user about key reuse.**
- **What BIP-87 says:** "it is best to restrict this script-agnostic hierarchy to multisignature wallets only". Its own footnote gives the reason: "With this proposed hierarchy, the user would either have to now backup additional data (the descriptor), or the wallet would have to attempt all script types for every account level when restoring."
- **Why the reason does not apply here:** every toolkit bundle *is* that descriptor backup (the md1 card carries the origin).
- **Why refusing would be inconsistent:**
  - Non-multisig `wsh(...)` shapes already default to BIP-48 2', which is also a multisig-only BIP.
  - A refusal would break the documented recipe `tr(NUMS, and_v(v:pk(@0/*), after(N)))` (`41-mnemonic.md` ~946) and `cli_non_canonical_descriptor::tr_nums_*`.
- **The real cost is key reuse:**
  - The same `87'/c'/a'` key is the default for template `wsh-sortedmulti`, template `tr-multi-a`, descriptor `tr(...)` and descriptor bare `sh(...)`. One seed at one account therefore uses one xpub for ECDSA and Schnorr and across script types.
  - BIP-87 asks for exactly the opposite: "It is crucial that this level is increased for each new wallet joined … This prevents key reuse - across ECDSA and Schnorr signatures, across different script types".
  - Only the bare-sh warning mentions `--account`.
- **Recommendation:** keep the BIP-87 default; do not refuse. Append the `--account` sentence to the info line of every BIP-87 default. For a shape with a single distinct key, raise it to `warning:` and add "BIP-87 is written for multisig; this card records the path, so restore does not depend on it". Keep the FOLLOWUP open for Brian.

**M6: a non-canonical `[fp]@N` is silently rewritten to the default** (I2's residual; pre-existing since v0.19).
- **Where:** `bundle.rs` ~1687 (the I2 refusal is gated `if !is_non_canonical`). `bind_descriptor_mode_paths` treats an empty inline path as "no origin".
- **Repro:** `sh(multi(1,[73c5da0a]@0/<0;1>/*,@1/<0;1>/*))` with seeds exits 0. It prints "no origin supplied for @0,@1" and records `[73c5da0a/87'/0'/0']`, although the operator wrote the master key of 73c5da0a.
- **Why Minor:** the card is self-consistent: the seed is derived at the recorded path.
- **Fix:**
  - Refuse an inline `[fp]@N` with no path on every shape, or honour it as `m` on non-canonical shapes, where an explicit empty origin is recordable.
  - The printed m/45' command composes correctly with such an `[fp]@N` (probe: `[73c5da0a/45']`), so no change is needed there.

### Nit

- **N1:** The redaction replaces non-secret forms too:
  - `--passphrase -` and `--passphrase @env:VAR` become `<your passphrase>`, which steers the user to type the passphrase on argv.
  - A slot value of `@env:VAR` is redacted the same way.
  - Keep `-` and `@env:` forms, as is already done for `=-`.
- **N2:** `rerun_command_with` uses `to_string_lossy`, so a non-UTF-8 argument is printed altered. The quoting is POSIX only; the GUI ships Windows builds, where `'\''` is wrong in cmd.exe and PowerShell. Say "POSIX shell" in the warning.
- **N3:** For canonical literal-xpub descriptors, xpub-search's notice says "non-canonical descriptor" and names a multisig path. `sh(wpkh(xpub))` now reads "… m/87'/0'/0' (BIP-87)". This is pre-existing for wpkh (2'), and the search is unaffected.
- **N4:** The bare-sh warning says "derived at" for xpub slots, which are not derived.
- **N5:** `docs/manual/src/30-workflows/37-wallet-export.md:149-155`, edited in this diff (`@0` → `@0/*`), shows `export-wallet --descriptor 'tr(NUMS,sortedmulti_a(2,@0/*,…))' --slot @N.xpub=…`. export-wallet refuses that ("accepts only concrete descriptors with inline keys; for keyless @N templates use --template"). The example was already broken before this branch, but the fold touched it without noticing.

## Q5: manual, manual-gui and FOLLOWUPS

- **Bare-sh warning text:** the manual's quoted line (`41-mnemonic.md:696`) is byte-identical to the binary's (diffed). The command line below it is not (M2).
- **Info notices and refusal texts:**
  - The canonical "no origin path supplied" info line and the B4 refusal text match the binary.
  - The I2 refusal is paraphrased in the manual, not quoted.
  - The xpub-search notice in `41-mnemonic.md:4396` and manual-gui `4d-*` matches the binary's text, but describes a default the search does not use (I-A).
- **Word-card budget** (manual, manual-gui `4n-word-card.md`): correct (`10` / `20`).
- **45-mr.md:** matches refugium 78d4a7c83 (`UNKNOWN_PROCEDURE`, `` `ITEM` value <k> (not echoed) is not an item``).
- **ms `--hex --language` lines:** consistent with ms#2.
- **FOLLOWUPS:** every r1 Minor (M1–M5) is logged at M, and the nits at N (with N1, N2, N4 marked resolved; I re-checked N2's comment in `verify_bundle.rs:1664`). The I3 residual is logged at M, and the decisions and cross-repo companions are present. Two need a change:
  - `tk-bh1125-decision-xpub-search-sh-wsh` is marked resolved on a notice-only change (I-A).
  - `tk-bh1125-noncanonical-xpub-header` should be I (I-B).

## Q6: PR body (`scratchpad/prbodies/mnemonic-toolkit.md`)

- **Header and footer:**
  - It starts with the attribution comment and the "Requested by" line.
  - It ends with the Claude Code footer followed by the session URL.
- **Inaccurate:**
  - B1 and B3 say xpub-search agrees with bundle. True of the notice; false of the search (I-A).
  - B3's test column cites `bughunt_1125_r1::b3_*`, but the xpub-search row only asserts text.
- **Merge order:**
  - It states that this PR goes before refugium's PR (`ci/toolkit-manual.rev`) and after ms#2. Both are correct.
  - It omits that the manual-gui G1 pages (1761da8) describe the unmerged mnemonic-gui G1 branch. Either merge after that GUI PR, or say the text runs ahead of it. The FOLLOWUP `tk-bh1125-gui-g1-manual` notes this, but the merge-order paragraph does not.
- **Tests:** the counts match the implementer's run. I did not rerun the full suite, per the brief.

## Summary of fixes

1. **I-A:** add BIP-87 (accounts) and m/45' to `xpub_search/candidate_paths.rs`, and make the b1–b3 test assert a match.
2. **I-B:** apply `check_defaulted_xpub_origin` to `bound_paths.defaulted` xpub slots (`bundle.rs:1865`), in emit and verify, with a test row.
3. **Minors:**
   - Fix the manual's command line and BIP-87 quote.
   - Add the m/45' account and verify-bundle line.
   - Correct "verifies as before".
   - Add the `--account` reuse advice to every BIP-87 notice.
   - Refuse or honour a non-canonical `[fp]@N`.
