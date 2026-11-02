# Bug-hunt grading pass G2, mnemonic-toolkit (tk): slices S3, S4, S5

Opus grading pass under RULES.md "Triage of reported bugs". Inputs: hunt report
`bughunt-2026-10-31-tk.md`, triage reports `bughunt-triage-tk-s{3,4,5}.md`, RED rows at
`a9f781b2`, evidence in `/scratch/cache/bughunt/tk/{s3,s4,s5}/`. No new hunting, no re-triage.
Production code and tests in this worktree are untouched.

**Outcome (33 CONFIRMED): 0 C, 17 I, 14 M, 2 N. X5 FALSE POSITIVE accepted (its residual is filed M).**

- I (17): #16 #17 #18 #19 #20 #29 #31 #49 #57 (S3); #26 #28 #56 (S4); #50 #51 X1 X3 X4 (S5).
- M (14): #21 #22 #23 #30 #58 (S3); #15 #24 #25 #27 #32 #33 #34 #48 (S4); X2 (S5). #30 is closed in the
  PR anyway by #31's fix (same patch).
- N (2): #35 #36.
- I disagree with the triage's proposal on 5 grades: #18 M->I, #31 M->I, X4 M->I, #15 I->M, #32 I->M. On #21
  I agree with M but reject the fix direction. Six fix patches need amending before they can ship: fix-18,
  fix-19, fix-20, fix-28, fix-56 and fix-x3 (also small amendments to fix-16, fix-51 and fix-x4). Details below.

## What I checked myself (beyond Settled)

- **Combined fix gate (machine-checked).** Scratch worktree `/scratch/code/shibboleth/tk-worktrees/grade-g2-scratch`
  (detached at `a9f781b2`). I applied all 17 C/I patches together (`fix-16 17 18 19 20 29 30 49 57`, `fix-26 28 56`,
  `fix-50 51 x1 x3 x4`). All applied cleanly. Full `cargo nextest run --locked --workspace` (cargo 1.85.0),
  log `/scratch/cache/bughunt/tk/g2/nextest-allfix.log`: 4148 run, 4099 passed, 49 failed. All 18 C/I rows pass.
  Five existing tests flip:
  `cell_3_sparrow_multi_2of3_wsh_sortedmulti_byte_exact` and `cell_5_sparrow_tr_multi_a_nums_2of3_byte_exact` (#20),
  `descriptor_builder::gate::tests::repeated_digest_cap_agrees_with_enumerate` (#28), and
  `e3a_testnet_tprv_mainnet_network_rejects` / `e3b_testnet_tprv_testnet_network_agrees_ok` (#56).
  `cargo clippy -D warnings` failed: fix-18 leaves `key_field_to_line` and `format_path_no_m_prefix` dead
  (`clippy-allfix.log`). `cargo fmt --check` failed on the raw patches (`fmt-allfix.log`: repair.rs:680,
  bitcoin_core.rs:909, coldcard_multisig.rs:494, sparrow.rs:648). The PR runs `cargo fmt`.
- **Prototype of my amendments.** I added the #18 dead-code deletion, the #28 gate dedupe and test rewrite, and the
  #56 message text and test vector. Targeted run: 82/82 pass (gate tests, `cli_compare_cost`, e3a/e3b, the #28
  and #56 rows). Full suite (`nextest-proto.log`): 4148 run, 4102 passed, 46 failed. The only non-row failures
  left are cells 3 and 5 (#20 goldens). The other 44 failures are the deferred M/N rows of both grading halves.
  Clippy `-D warnings` is clean (`clippy-proto.log`). Combined diff: `/scratch/cache/bughunt/tk/g2/allfix-proto.diff`.
  This is evidence, not the build: it does NOT carry my amendments to #16 (regex form), #19, #20, #51, X3 or X4.
- Post-fix holder output, executed on the scratch binary: #19 (exit 0, `threshold=2`, `mainnet`), #29, #31,
  #49, #57 (exact texts in the Fixes section), #51, X1 (ms1 and mk1), X3 (damaged share exit 4 with report,
  clean share exit 0), #56.
- External sources (fetched 2026-11-01):
  - **Coldcard BSMS signer** (`shared/bsms.py` at `2b2e8083`, the parent of commit `e4e1844f` "remove BSMS").
    Line 849: `xfp = xfp2str(settings.get('xfp', 0))`. Line 894: `desc_type_key = "[%s%s]%s" % (xfp, dd[1:], ext_key)`.
    `utils.py:168-171` `xfp2str` returns `... .decode().upper()`. So a real signer writes an UPPERCASE fingerprint
    on line 3, and its coordinator side (lines 684-695) verifies over `key_exp` as written.
  - **Coldcard multisig** (`chains.py:406-409`): `MS_STD_DERIVATIONS` includes `("p2sh", "m/45h", AF_P2SH)`.
    `descriptor.py:195-196` serialises every key as `[xfp/path]xpub/<0;1>/*` whatever the derivation.
  - **drongo** (Sparrow) `Keystore.java:21-22`: `DEFAULT_LABEL = "Keystore 1"`, `MAX_LABEL_LENGTH = 16`.
    Lines 384-386: `if(label.replace(" ", "").length() > MAX_LABEL_LENGTH) throw ... "Label too long"`.
    Reached from `Wallet.checkWallet()` (cached `Wallet.java:2362-2371`).
  - **BIP-388** (cached copy). Lines 149-151: a `KEY` is "a `KP` expression *always* followed by" `/**` or
    `/<NUM;NUM>/*`. Line 310 is in the invalid-templates list: "Expression with a non-KP key present".
  - **BIP-93** (cached copy). Line 82: "a codex32 string MUST be entirely uppercase or entirely lowercase".
    Line 85: "uppercase SHOULD be used for handwritten codex32 strings". Line 513: "it's also valid to use all uppercase".
- **Plate case.** `mnemonic-engrave/design/CONTINUITY_2026-08-05b.md:164-165`: "Seed, codex32 and SLIP-39 goldens
  ... they engrave upper case". The SeedHammer keypad also uppercases typed codex32 input
  (`IMPLEMENTATION_PLAN_firmware_pr1_codex32.md:80-81`). The toolkit manual (`41-mnemonic.md:3977-3980`) claims
  "Card intake is case-tolerant ... all-uppercase cards ... decode end-to-end".
- **GUI `required` is a marker only.** `mnemonic-gui src/form/render_emit.rs:577` adds a "(required)" marker.
  `app_window.rs:1088-1090` enables Run without any required-field check, and `conditional.rs:1243-1254`
  `run_blocker` blocks only `ms hashlock`. So #33/#34 never stop a run.
- The GUI help for `repair --ms1` (`mnemonic-gui src/schema/mnemonic.rs:2545-2547`) says "`@env:VAR` (read by the GUI ...)".
  This confirms the triage's X2 correction.
- `ms-codec` (vendored) `decode.rs:47-50`: the pre-dispatch length error passes `allowed: VALID_STR_LENGTHS`
  ("report the entr set as the primary allowed set").
- A Coldcard `ccxp-*.json` multisig xpub export writes `"account": "%d"` as a string (`multisig.py:1648`).
  It is refused earlier (`missing or non-string top-level \`chain\``, executed before and after fix-29), so
  #29's refusal of string accounts changes no real Coldcard file.

**Grading principle used throughout.** Precedent: the ms grading pass, #3, #6 and #8.

- A wrong value in a funds-relevant result stays C/I however rare. Funds-relevant means a threshold, key origin,
  fingerprint, address, descriptor, or a validity verdict the holder acts on.
- A wrong cosmetic detail is M when the main result is right: a label, a placeholder list, an advisory notice,
  or an index inside a correct refusal.
- Where the likelihood test fails only on (3), the usual grades apply. For an abort or panic that prints nothing
  and changes no data, the usual grade is M.

## Grades

| # | final grade | triage proposal | agree? | row tests the defect? | likelihood conditions / why |
|---|---|---|---|---|---|
| 16 | **I** | I | yes | Yes: `sortedmulti_a` and `multi_a` blobs, asserts JSON `bundle.multisig.threshold == 2`. | The import reports a 2-of-3 taproot wallet as threshold `none` (text) or `3` (JSON), exit 0. That is a wrong statement of the spending policy: a holder who believes all 3 keys are needed may store two keys together. Not C: cards and `bundle --import-json` read K from the tree (triage, executed), and `export-wallet --from-import-json` refuses taproot envelopes (`export_wallet.rs` §2.4 gate), so no artefact carries the 3. |
| 17 | **I** | I | yes | Yes: the repo's own 2-of-3 taproot Sparrow fixture plus its `sortedmulti_a` rewrite. | Same as #16, Sparrow importer. |
| 18 | **I** (raised) | M | **no** | Yes: in-test signed records. The control passes, and the uppercase and `h` sub-cases fail with "pubkey mismatch". The `[fp/]` sub-case is a live regression pin (see FP section). | Not Minor. (1) fails: a valid signature is reported as forged, which is a wrong verdict on an authenticity check. In strict mode exit 2 reads "signature verification failed"; in the default mode exit 4 reads `signature_verified:false`. (3) fails: one ordinary condition. Coldcard's BSMS signer wrote the fingerprint UPPERCASE (`bsms.py:849,894` + `utils.py:171`), so every Round-1 record that firmware made is rejected. BIP-129 line 81 signs "the first four lines" as written. |
| 19 | **I** | I (borderline M) | yes | Yes: `Derivation: m/45'`, `Format: P2SH`, asserts exit 0 and `threshold=2`. | Fails closed with no printed command, but needs only one unusual condition: a legacy BIP-45 P2SH multisig. That condition is Coldcard's own standard path for account 0 (`chains.py:407`). So the usual grade applies: a supported format's standard variant is refused (a missing case). |
| 20 | **I** | I | yes (fix differs) | Yes: asserts three distinct keystore labels. | The toolkit writes, with exit 0, a file that its target refuses on every multisig export ("Wallet keystores have duplicate labels"). No unusual condition. fix-20 is insufficient: the default name `wsh-sortedmulti-0` is 17 non-space chars, so `"{name} {i}"` still trips drongo's 16-char "Label too long". |
| 21 | **M** | M | grade yes; **fix rejected** | No as specified. It detects the template/descriptor inconsistency, but its expected outcome (the `--descriptor` path emits `tr(<NUMS hex>,…)`) is a BIP-388-invalid policy (lines 149-151, 310). Rewrite it when the follow-up is decided. | The `--descriptor` refusal is correct by BIP-388: a literal key is not a key placeholder. Only its wording is imprecise ("must end in `/<0;1>/*`" names the NUMS key instead of saying a literal key cannot be a placeholder). It fails closed (exit 1, no data). Conditions: (a) taproot multisig with a NUMS internal key, and (b) entered through `--descriptor` instead of `--template`. Do not ship fix-21. Why the `--template` path emits the literal NUMS hex is a separate product question (Deferred). |
| 22 | **M** | M | yes | Yes: asserts `devices.len() == 3`. | Does not fail closed, so usual grades apply. Each entry is a literal `"unknown"` placeholder that carries no information. Specter reads the list behind a length guard (`wallet_importer.py:153,157`), so the short list changes no import. Wrong placeholder metadata, so M. |
| 23 | **M** | M | yes | Yes: control `Vault-2 -> Vault`, then three digit-ending names. | Usual grades: descriptor, keys and threshold are right; only the free-text label lifted from cosigner 1 loses trailing digits. Conditions for harm: (a) an Electrum multisig whose first keystore label ends in digits not preceded by `-`, and (b) the name is reused downstream (the export name lift). |
| 29 | **I** | I | yes | Yes: `"5"`, `-1` and `1.5` each produce `[…/44'/0'/0']`. | A wrong key origin (account rewritten to 0), exit 0, contrary to the code's own rule (`coldcard.rs:240-245`). A signer matching origins cannot sign. |
| 30 | **M** (closed in the PR by #31's fix) | M | yes | Yes: asserts no panic and exit 2. | Fails closed in substance: exit 101, nothing printed, no data. Conditions: (a) a hand-edited or corrupted `xfp` holding a non-ASCII char, and (b) a UTF-8 length of exactly 8 bytes. The same one-line hex check as #31 fixes it, so the row passes in the PR; no FOLLOWUPS entry. |
| 31 | **I** (raised) | M | **no** | Yes (permissive: asserts only non-zero; tighten). | The input silently bypasses validation and becomes a wrong master fingerprint in the imported descriptor's origin, exit 0. Settled: a silently ignored input or wrong data stays C/I however rare. The triage's "garbage in has no correct output" misses the point: the correct output is the refusal. A `+` typed over a pair's leading digit (`+436D724`) yields a plausible wrong fingerprint. |
| 49 | **I** | I (C arguable) | yes | Yes: a master `xpub` plus only a `bip48_2` block; asserts the master is not emitted. | Wrong addresses silently (master xpub labelled `[fp/44'/0'/123']`), exit 0. Not C: no Coldcard firmware writes this shape (every generic export carries bip44/49/84, `generic-wallet-export.md`), and the depth-0 marker is visible in the output. |
| 57 | **I** | I | yes | Yes: asserts the `wsh(pk)` + `wsh(pkh)` pair is not merged. | The change script is silently dropped, exit 0: a wrong descriptor. Rare (Core does not produce it), but a wrong result stays I. |
| 58 | **M** | M | grade yes; conditions differ | Yes: asserts `descriptors[2]` and not `[1]`. | Fails closed (exit 2, correct reason). The quoted bad checksum (`invalid checksum aaaaaaaa`) lets the holder find the entry. The triage's condition (a), "a merge in the blob", is the normal Core shape, not unusual. So only (b), a corrupt or unsupported entry after a merged pair, remains, and the usual grades apply: a wrong index inside a correct refusal is M. |
| 15 | **M** (lowered) | I (borderline) | **no** | Yes: SIGABRT under `ulimit -v 4000000`. | Fails closed in substance: abort, no output, no secret, no data. On a large box it prints the rolls that were asked for. Only one unusual condition, (a) a dice `--length` of roughly 10^8 or more (a typo or a deliberate absurd value), so the usual grades apply, and the usual grade for an abort on an absurd argument is M (ms #6). |
| 24 | **M** | M | yes | Yes: asserts the notice names `/1'`. | Search results are right (the match line shows `m/48'/0'/0'/1'`); only the info notice says `/2'`. Conditions: (a) a nested-segwit multisig descriptor, and (b) origin-elided cosigners. |
| 25 | **M** | M | yes | Yes: `MK=… --from minikey=@env:MK`. | Fails closed. Secret-channel class (operator ruling), so never C/I. Conditions: (a) a Casascius mini-key input, and (b) the `@env:` channel (`minikey=-` works). |
| 26 | **I** | I | yes | Yes: the address changes with `--passphrase`, yet the warning says it is ignored. | False advice on a funds-address command: the holder is told the address belongs to the no-passphrase wallet. Every phrase/seedqr/entropy -> address run with a passphrase hits it. Not fail-closed. |
| 27 | **M** | M | yes | Yes: `sha256` and `hash256` leaves under `tr(NUMS, …)`. | Fails closed (exit 3) with a misattributed message. Advisory cost tool, no data. Conditions: (a) a hash-locked tap leaf with a 64-hex digest, and (b) `--descriptor tr(…)` input (`--miniscript` works). |
| 28 | **I** | I (borderline M) | yes | Yes: or_i with one digest; asserts 2 rows and that `--max-conditions 8` passes. | compare-cost's result (its `conditions` list) names two distinct preimages `h0`/`h1` for one digest and doubles the count. Its cap precheck also falsely refuses. A wrong result, and no fail-closed path for the listing. The manual documents the leaf model (`41-mnemonic.md:4583-4584`), so the docs change with the fix. |
| 32 | **M** (lowered; both rows) | I (borderline M) | **no** | Yes (32a `template.rs:78`; 32b `descriptor_intake.rs:434`). | Fails closed in substance: exit 101, the panic names `InvalidChildNumber(2147483648)`, no output. 32a has one unusual condition, (a) `--min-account` of 2^31 or more; 32b adds (b) `--number-of-accounts 0`. Usual grade for a panic on an absurd argument: M. |
| 33 | **M** | M | yes | Yes: evaluates the schema rule against clap's real demand. | The GUI's "(required)" on `--template` is only a marker; Run is not blocked (see What I checked), and the same GUI form's `--import-json` help says "Mutually exclusive with --template". Obeying the marker gives clap's clean conflict refusal. Conditions: (a) GUI user in `bundle --import-json` mode, and (b) the user fills `--template` because of the marker despite the flag's own help. |
| 34 | **M** | M | yes | Yes: three alternatives. | GUI marker only, fails closed. A keyed bundle really needs `--template` at runtime and the CLI says so. Conditions: (a) GUI user, (b) a keyless single-sig template bundle, and (c) the holder does not know the template. |
| 35 | **N** | N | yes | Yes. | One advisory note missing; every number is right. |
| 36 | **N** | N | yes | Yes (mapper row). | Latent: no input reaches the self-contradictory form (unreachability accepted, see FP section). |
| 48 | **M** | M | yes | Yes: `legacy` and `auto` with `sig + "\n"`. | Fails closed with a refusal, not an INVALID verdict. Conditions: (a) a legacy P2PKH signature, and (b) a literal leading or trailing blank reaches argv (`$(cat f)` strips it). |
| 56 | **I** | I (C arguable) | yes | Yes: a depth-3 xprv; asserts refusal and no `sp1`. | A wrong receive address, silently, under a documented contract ("the xprv IS the master"). Not C: the scan/spend keys are printed and derive from the holder's own seed at a fixed path. The toolkit's own test suite feeds a depth-3 tprv (`cli_network_fail_open.rs:62-63`), which shows how easily it happens. |
| 50 | **I** | I | yes | Yes: repeated `--from`, two stdin lines, and three md1 chunks, each compared byte-for-byte with the joined form. | The help's own advice ("repeat the flag; chunks are auto-grouped by HRP", `word_card.rs:61-62`; manual `:5013`) fails as printed for every multi-chunk card: a printed instruction that fails. |
| 51 | **I** | I | yes | Yes: corrupt second ms1 -> exit 0 (and a valid 256-bit second card is not shown). | A check silently does not run: the second card is neither decoded nor checked, exit 0. Not fail-closed. |
| X1 | **I** | I | yes | Yes: mixed-case ms1/md1/mk1; only mk1 is accepted. | `repair` gives a wrong validity verdict (exit 0, mixed-case string echoed as valid). The mk1 BIP says mixed case "MUST be rejected", and `inspect`/`convert`/`mk repair` refuse the same string. The holder re-engraves from `repair`'s verdict, so a wrong OK verdict is I (ms #5). Not fail-closed. |
| X2 | **M** | M | yes | Yes. | Fails closed with a clear-enough message. No toolkit text advertises `@env:` for `repair`/`inspect`, and the GUI reads the variable itself. Secret-channel class. Conditions: (a) an undocumented syntax on this verb, and (b) the holder expects parity with `verify-bundle`. |
| X3 | **I** | I | yes | Yes: a damaged share wants exit 4 with the clean share; a clean share wants exit 0. | The printed advice "use `ms combine` to recombine K shares" fails as printed for a damaged share (BCH invalid, exit 1), and `repair` refuses even a clean share. A printed command that fails stays I. |
| X4 | **I** (raised) | M | **no** | Yes: lowercase baseline exits 5; all-capitals ms1/mk1/md1 exit 2. | All-capitals is the normal plate form: engraved codex32 plates are upper case (`mnemonic-engrave` CONTINUITY note), BIP-93 line 85 says handwritten strings SHOULD be uppercase, and the manual promises case-tolerant intake. So the triage's condition (c) is typical. (a) an indel and (b) `--max-indel` are one condition: `--max-indel` is the documented remedy for an indel. (3) fails, and the usual grade for a recovery feature that does not work on the normal plate form, with a message that points away from the cause ("more than 1 inserted/dropped characters, or a different error class"), is I. The ms FIX review graded the same item I-1 (FOLLOWUPS `toolkit-repair-indel-search-rejects-capitals`). |

## False positives and dropped sub-cases

- **X5: FALSE POSITIVE, accepted.** The plant (`plant-x5.patch`, `ms1_vec.truncate(1)` after `expand_dashes`,
  `repair.rs:361`) makes the row fail at its own assertion: "second stdin line silently ignored: exit 0, stdout:
  ms10entrsq…34v7f". That is the reported mechanism, for the right reason. Where the real code differs:
  `repair.rs:360` `expand_dashes` replaces `-` with every stdin line, and the Ms1 arm of `repair_card`
  (`repair.rs:1208`) repairs each one. The row stays as a passing regression pin. The residual (a card wrapped over
  two lines is read as two strings, exit 2 "too many errors") is real and filed M (Deferred).
- **#18, empty path `[fp/]`: dropped sub-claim accepted.** The plant (`format_path_no_m_prefix` returns `"m"`)
  makes that sub-case fail, so the sub-case is live. The real rebuild gives `[fp/]` because vendored bitcoin 0.32
  `DerivationPath` Display prints no `m/` (`vendor/bitcoin/src/bip32.rs:459-470`). The point is moot after the fix:
  line 3 is kept as written, and the helper is deleted.
- **#36, unreachability accepted.** The only producer that reports the mnem set (`ms-codec decode.rs:62`) sits
  behind `envelope::discriminate`, which validates entropy length first. The triage's 96-string scan found no
  reaching input. The reachable cousin ("length 52 not in v0.1 set [50, …]", which omits the mnem lengths) is
  ms-codec's own choice at `decode.rs:47-50`. fix-36 would not change it, so it is routed to ms (Deferred), not
  graded here.
- **#24, hunter's impact claim corrected (accepted).** The search covers the right paths; only the notice is
  wrong. **#28:** the hunter's `or_d` input is not valid miniscript; the row's `or_i` is right. **#32:** the first
  panic site is `template.rs:78`, not `candidate_paths.rs:172`. All three accepted.
- **#21, `build-descriptor` claim:** not tested by the triage and not verified by me. Moot, since fix-21 is rejected.
- **#16, hunter's "check `bsms::extract_threshold` too":** it already refuses taproot before the regex
  (`bsms.rs:470-472`, `BsmsTaprootImportRefused`). `specter.rs:427` and `descriptor.rs:229` already carry the `_a`
  forms. No change there.
- **X2, the "GUI claims it" premise:** the triage's correction is accepted (verified at `mnemonic-gui
  src/schema/mnemonic.rs:2545-2547`: "read by the GUI").

## Product rulings

Exit codes follow `ToolkitError::exit_code` (`error.rs:640-700`): `BadInput` 1, `ImportWalletParse` 2,
`SilentPayment` 1, `Repair` 2, `DeriveChildLengthOutOfRange` 2. The binary prefixes every message with `error: `.

**S3**

- **#29.** Refuse. A present `account` that is not a JSON non-negative integer is refused wherever it appears,
  including when a bipNN block makes it unused. That covers a string (even `"5"`), a negative number, a fraction,
  `null` and a boolean. Text: ``error: import-wallet: coldcard: parse error: `account` must be a non-negative
  integer, got <v>``, where `<v>` is serde_json's rendering (`"5"`, `-1`, `1.5`, `null`). Exit 2. Do not accept
  `"5"` as 5: a backup tool does not guess an index.
- **#21.** Neither of the triage's options. The `--descriptor` path keeps refusing (BIP-388 lines 149-151, 310).
  Follow-up text for the refusal (exit 1 unchanged): ``error: --format bip388 cannot express the literal key
  <hex> (a taproot internal key with no origin, e.g. the BIP-341 NUMS key): a BIP-388 wallet policy allows only
  @N key placeholders; use --format descriptor or --format bitcoin-core for this wallet``. Whether the `--template
  tr-*-multi-a --taproot-internal-key nums --format bip388` path should keep emitting the literal NUMS hex (invalid
  per line 310) goes to the architect with the next export-wallet spec. It is not in this PR.
- **#49.** Refuse only in the legacy-fallback arm, as fix-49 does. When a bip44/49/84/86 block exists, the
  top-level master `xpub` is not used and nothing is refused.
- **#20.** Every keystore's label is `Keystore <n>`, n = 1-based slot order; single-sig is `Keystore 1`. This is
  drongo's own default (`Keystore.DEFAULT_LABEL`). It is always unique and always 16 non-space chars or fewer.
  Not `"{name} {i}"` and not `"{name}-{i}"`: names exceed Sparrow's 16-char cap (default `wsh-sortedmulti-0` is 17).
  The top-level `name` is unchanged, and the toolkit's Sparrow importer reads only `name` (`sparrow.rs:85`), so
  round-trip is unaffected.
- **#22** (M follow-up). In `--descriptor` mode `devices` lists one `"unknown"` per descriptor key, except keys
  that are origin-less single keys (the NUMS internal key is not a device); at least 1. A taproot internal key that
  is a cosigner (has an origin) counts.
- **#19.** BIP-45 mode applies only when EVERY cosigner path is exactly `m/45'` (one hardened component, index 45).
  The network then comes from cosigner 0's xpub version (`xpub` -> mainnet, `tpub` -> testnet, matching the
  coin-type 0/1 mapping of `network_from_path`). The coin-type loop is skipped, and the existing
  `assert_slots_network_agrees` checks every xpub. If some but not all cosigners are at `m/45'`, behaviour is
  unchanged (today's coin-type refusal).
- **#57.** Refuse, as fix-57 does. "Import separately" is not available: each entry alone is a fixed-step single
  path, which the toolkit refuses anyway. The holder sees the existing fixed-step refusal for `descriptors[0]`
  (text in Fixes).

**S4**

- **#15** (M follow-up). Dice rolls are capped at 1..=1000000. Text: ``error: --length <N> out of range for
  --application dice (valid: 1..=1000000 rolls)``, exit 2. `Vec::with_capacity(rolls.min(1024))`.
- **#24** (M follow-up). `sh(wsh(...))` -> `1'` (fix-24). Plain `sh(...)` keeps today's `2'`: BIP-48 defines no
  legacy-P2SH script type, and whether `bundle`'s `Sh -> 1'` is right is a separate question. Keep the notice.
- **#32** (M follow-up). Refuse, never clamp. Text: ``error: --min-account / --max-account / --number-of-accounts:
  the account window must stay below 2147483648 (2^31): BIP-32 hardened account indexes are 31 bits``, exit 1.
  It fires when `--min-account` >= 2^31, `--max-account` >= 2^31, or `--min-account + --number-of-accounts` > 2^31.
- **#33/#34** (M follow-ups). The schema mirrors clap's `required_unless_present_any` exactly.
  - `bundle`: add `--import-json` to the `--template` rule's `any_of` (fix-33). Its rationale becomes "--template
    is required unless --descriptor, --descriptor-file or --import-json is supplied (bundle's primary-mode
    selection invariant)."
  - `verify-bundle`: add `--md1`, `--bundle-json` and `positional_present 0` (fix-34). The CLI's own runtime
    refusal covers keyed bundles that lack `--template`.
  - GUI lockstep in `mnemonic-gui src/form/conditional.rs`: `bundle()` and `verify_bundle()` get the same
    conditions, plus their doc-comment clap citations.
  - Toolkit test that changes: `cli_gui_schema_conditional_rules.rs::bundle_template_required_unless_uses_not_any_of_predicate`
    (`predicates.len()` 2 -> 3, and assert `--import-json`).
- **#36** (N follow-up). Fix the mapper so it prints the error's `allowed` field (fix-36). Keep ms-codec's
  post-dispatch check, which is that crate's defence in depth.
- **#56.** Refuse, and name what to supply (text in Fixes).
- **#48** (M follow-up). Trim leading and trailing whitespace only, as the BIP-322 path does (`verify_message.rs:212`).
  No interior whitespace or line-wrapped base64: a legacy signature is 88 characters and no signer wraps it.

**S5**

1. **#50.** If every `--from` value decodes as a whole card, nothing changes. Otherwise single-chunk values are
   joined per HRP in order of first appearance, and values that already hold whitespace-joined chunks stay as they
   are (fix-50). Several multi-chunk cards of one HRP given chunk-by-chunk are therefore refused by the codec's own
   error; pass each one joined. `--raid` is unchanged. No help or manual text change: both already state this.
2. **#51.** Refuse a second ms1 (text in Fixes). Reporting several ms1 would need a new output shape.
3. **X1.** Refuse a mixed-case chunk outright, for every kind, with md-codec's existing wording. BIP-93 line 159's
   erasure reading is optional. The erasure-fill spec may revisit it, but must keep one rule for all kinds.
4. **X2** (M, for the erasure-fill spec). Resolve `@env:VAR` for `--ms1` on `repair` and `inspect` (fix-x2), and do
   not fire the argv advisory for an `@env:` value. No GUI change: the GUI already reads the variable.
5. **X3.** Correction lives in toolkit `mnemonic repair` only (verb placement: `ms repair` will be removed), with no
   ms-codec change. A corrected share exits 4 (VERIFY-ME, like every ms1 correction, `cli_repair.rs` cell 9) and a
   clean share exits 0. It prints a share-specific UNVERIFIED line instead of the failing `ms combine` advice (text
   in Fixes).
6. **X4.** A corrected all-capitals card is printed in lowercase (BIP-93 line 513, canonical lowercase), as the
   non-indel path already does.
7. **X5 residual** (M, for the erasure-fill spec). When `--ms1 -` reads 2 or more non-empty lines, join them only if
   no line alone has a valid ms1 length and their concatenation does. Otherwise each line stays its own string, as
   today. The manual's "use `-` to read one chunk from stdin" (`41-mnemonic.md:3382`, `:3895`) is decided in the
   same spec.

## Fixes for the PR (C/I only)

General: apply the named patches, then `cargo fmt` (the raw patches are not fmt-clean). The build gate must include
clippy `-D warnings`.

Manual means `docs/manual/src/40-cli-reference/41-mnemonic.md`. These RED rows accept alternatives and must be
tightened to the ruled outcome: #20, #29, #30, #31, #49, #51, #56, #57, X1, X3, X4.

- **#16:** fix-16, but write the regex as `(?:thresh|multi|sortedmulti|multi_a|sortedmulti_a)\((\d+)\s*,`
  (identical to `specter.rs:427`).
  - Holder sees `bundles[0].threshold=2` (was `none`) and JSON `bundle.multisig.threshold: 2` (was 3). Exit 0 unchanged.
  - Tests that change: none. Manual: none.
- **#17:** the same regex in `sparrow.rs:651`. Same effect; no tests or manual change.
- **#18:** fix-18, and delete `key_field_to_line` and `format_path_no_m_prefix` with their doc comments from
  `bsms_round1.rs` (clippy `-D warnings` fails otherwise; executed).
  - No text change. TV-1/TV-2 unit tests are unchanged and pass. Tests that change: none. Manual: none.
- **#19:** fix-19, with `let bip45 = resolved.iter().all(|c| is_bip45_root(&c.path));` in place of the cosigner-0-only test.
  - Executed result: exit 0, `bundles[0].network=mainnet`, `bundles[0].threshold=2`, descriptor
    `sh(sortedmulti(2,[34a3a4f1/45']xpub…/<0;1>/*,…))`. This equals Coldcard's own rendering.
  - JSON `path_family` stays `"bip87"` with the existing non-canonical-purpose notice (designed, `import_wallet.rs:2134-2140`).
  - Tests that change: none. Manual: none.
  - RED rows: add `[34a3a4f1/45']` to the assertion. Add a row where cosigner 0 is at `m/45'` and cosigner 1 at
    `m/48'/0'/0'/2'`: exit 2, ``derivation path has only 1 components; need ≥2 for BIP-48 coin-type inference``.
- **#20:** replace fix-20's scheme with `label: format!("Keystore {}", i + 1)` for every keystore (single-sig too).
  The `String` field change in fix-20 stays.
  - Tests that change: `cli_export_wallet_sparrow.rs` cells 1, 2, 3 and 5. Their goldens
    `tests/export_wallet/sparrow_single_wpkh.json`, `sparrow_single_tr.json`, `sparrow_multi_2of3_wsh_sortedmulti.json`
    and `sparrow_tr_multi_a_nums_2of3.json` get every keystore `"label"` set to `Keystore <n>`. Cells 3 and 5 were
    executed flipping; cells 1 and 2 flip by the single-sig rule.
  - The import fixture `tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json` is an input and does not change.
  - Manual: none (labels are not documented). RED row: labels equal `["Keystore 1","Keystore 2","Keystore 3"]`.
- **#29:** fix-29 as is.
  - Text (executed): ``error: import-wallet: coldcard: parse error: `account` must be a non-negative integer, got "5"``. Exit 2.
  - Tests that change: none.
  - Manual: add a row to the import-wallet `### Refusals` table: ``| Coldcard generic JSON: `account` present but
    not a non-negative JSON integer (`"5"`, `-1`, `1.5`) | exit 2 `ImportWalletParse` — `import-wallet: coldcard:
    parse error: `account` must be a non-negative integer, got <value>` |``
- **#31 (closes #30):** fix-30 as is.
  - Text (executed): ``error: import-wallet: coldcard: parse error: `xfp` must contain only hex digits``. Exit 2.
    The 8-byte `aéééb` hits this instead of panicking.
  - Tests that change: none.
  - Manual row: ``| Coldcard generic JSON: `xfp` holds a character that is not a hex digit | exit 2
    `ImportWalletParse` — `import-wallet: coldcard: parse error: `xfp` must contain only hex digits` |``
- **#49:** fix-49 as is.
  - Text (executed): ``error: import-wallet: coldcard: parse error: top-level `xpub` is a depth-0 master xpub, not
    an account xpub; no bip44/bip49/bip84/bip86 block to import``. Exit 2.
  - Legacy depth-3 top-level xpubs still import. Tests that change: none.
  - Manual row: ``| Coldcard generic JSON: no `bip44`/`bip49`/`bip84`/`bip86` block and a depth-0 (master)
    top-level `xpub` | exit 2 `ImportWalletParse` — `… top-level `xpub` is a depth-0 master xpub, not an account
    xpub; no bip44/bip49/bip84/bip86 block to import` |``
- **#57:** fix-57 as is.
  - Holder sees (executed): ``error: import-wallet: bitcoin-core: parse error: descriptors[0]: @0: derivation steps
    after the placeholder are not representable in md1; the use-site path must be a multipath `/<a;b>/*` (or bare
    `/*`) as the final step — a fixed single step like `/0/*` is un-representable (found residue near `/0/*))`)``. Exit 2.
  - Ordinary pairs still merge. Tests that change: none.
  - Manual: after the sentence ending "since distinct keys are different wallets." (`:1816-1818`), add: "A same-key pair whose script bodies differ
    (e.g. `wsh(pk(…/0/*))` + `wsh(pkh(…/1/*))`) is not merged either; its entries then hit the fixed-step refusal
    (exit 2)." Also add a Refusals row: ``| Bitcoin Core same-key receive/change pair whose script bodies differ |
    exit 2 `ImportWalletParse` — not merged; the first entry gets the fixed-step refusal above |``
- **#26:** fix-26 as is. `convert --from phrase|seedqr|entropy=… --to address --passphrase …` no longer prints
  `warning: --passphrase ignored on this edge (not a PBKDF2-bearing conversion)`. Tests that change: none. Manual: none.
- **#28:** fix-28, plus three changes in `descriptor_builder/gate.rs`:
  - (a) `hash_and_timelock_counts` counts distinct (hash kind, digest) pairs, not leaves. The prototype uses
    `(u8, h.to_string())` in a `Vec` with a `contains` check.
  - (b) The doc comments on `check_cap` (`:555-565`) and `hash_and_timelock_counts` (`:610-618`) say digests are
    deduped on both sides.
  - (c) Rewrite `repeated_digest_cap_agrees_with_enumerate` on the same shape, raw = 2^(1+1) = 4. Gate: Ok at cap 4,
    `OverEnvelope` at cap 3. Enumerate: `run(4)` Ok, `run(3)` `ConditionsTooMany`. Drop the "buggy dedup would have
    PASSED at cap 4" block.
  - Holder sees one `preimage(h<i>)` per distinct digest. For `or_i(and_v(v:pk(A),sha256(H)),and_v(v:pk(B),sha256(H)))`
    the rows are `A + preimage(h0)` and `B + preimage(h0)`, and `--max-conditions 8` passes.
  - Tests that change: that gate test only (prototype: 82/82 targeted pass, full suite clean).
  - Manual `:4583-4584`: "`preimage(h<i>)` in AST-order, one per `sha256` / `hash256` / `ripemd160` / `hash160`
    leaf supplied." becomes "`preimage(h<i>)` in AST-order, one per distinct hash digest (a digest repeated in
    several `sha256` / `hash256` / `ripemd160` / `hash160` leaves is one preimage)."
- **#56:** fix-56 at the same position (before the passphrase warning and the network check), but the inner message
  must not repeat the prefix. fix-56 as written prints `silent-payment: silent-payment:` (executed).
  - Text (executed): ``error: silent-payment: the xprv/tprv is at depth 3, not a master key (depth 0); supply the
    master xprv/tprv, or the seed phrase, ms1 or entropy it came from``. Exit 1.
  - Tests that change: `tests/cli_network_fail_open.rs`. `TESTNET_TPRV` (`:62-63`, an m/84'/1'/0' account tprv,
    depth 3, used by e3a/e3b) becomes the depth-0 testnet master of the abandon…about vector,
    `tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd`.
    I computed it from the BIP-39 seed; its mainnet form equals `cli_silent_payment.rs` `ROOT_XPRV`. Its doc comment
    becomes "Testnet master tprv (depth 0) of the abandon…about vector." With this, e3a/e3b pass (executed).
  - Manual `:3135` `--secret` row: append "A non-master xprv/tprv (depth ≠ 0, e.g. an account-level key) is refused (exit 1)."
- **#50:** fix-50 as is. No text change. Tests that change: none. Manual: none (it already says this).
- **#51:** fix-51 with this text: `format!("inspect: expected 1 ms1 card, got {}; inspect each ms1 card in its own run", chunks.len())`.
  - Holder sees ``error: inspect: expected 1 ms1 card, got 2; inspect each ms1 card in its own run``. Exit 1.
  - Tests that change: none.
  - Manual inspect `--ms1` row (`:3895`): append "; a second ms1 (positional or stdin line) is refused (exit 1)".
- **X1:** fix-x1 as is.
  - Text (executed, all kinds): ``error: repair: chunk <i> parse failed before correction could run: string mixes
    upper and lower case (BIP-173 forbids mixed case)``. Exit 2. For ms1 this replaces `InvalidCase(Lower, 'E')`.
  - Tests that change: none (the `cli_hrp_case_insensitive.rs` mixed-case cells drive `inspect`).
  - Manual, repair section, after the Exit codes table: "Case: an all-lowercase or all-capitals chunk is accepted
    (corrected output is lowercase); a chunk that mixes upper and lower case is refused (exit 2) for `ms1`, `mk1`
    and `md1` (BIP-173, BIP-93)." Also fix the stale `:3981` "`mk1`/`ms1` reject mixed-case input, `md1` is lenient"
    to "`mk1`/`ms1`/`md1` reject mixed-case input". md1 has refused since md-codec 0.35.3 (`cli_hrp_case_insensitive.rs:594-609`).
- **X3:** fix-x3, plus four amendments:
  - (a) Bound-check every position (`p < symbols.len()`, else `TooManyErrors { bound: 8 }`).
  - (b) Re-verify after correction: the polymod of the corrected symbols must equal `MS_REGULAR_CONST`, else
    `TooManyErrors { bound: 8 }` (as ms-codec `decode.rs:267-291`).
  - (c) The corrected or clean string must decode to `Err(IsShareNotSingleString { .. })` under `ms_codec::decode`.
    Any other outcome is `PostCorrectionDecodeFailed` carrying that error's text.
  - (d) When any corrected chunk is a share, the `SetVerify::Unverified` reason is: `correction UNVERIFIED — a
    corrected K-of-N share cannot be self-verified; recombine it with the other shares (mnemonic ms-shares combine
    --share <SHARE> --share <SHARE> …) and confirm the recovered seed's address/xpub against a known-good copy
    before use; BIP-93 recommends confirming a corrected codex32 string`. The seed-card text assumes an address can
    be derived from the one card, and one share cannot do that. The advised command runs as printed (executed with
    two shares, exit 0).
  - Output: a damaged share exits 4 with `# Repair report` / `#   ms1 chunk 0: 1 correction at position <p>:
    '<was>' -> '<now>'` and the clean share on stdout. A clean share exits 0, echoed.
  - Tests that change: none.
  - Manual, repair section: "A K-of-N `ms1` share (threshold 2–9) is corrected like any `ms1` (exit 4, VERIFY-ME; a
    clean share exits 0); recombine the corrected shares with `mnemonic ms-shares combine`."
- **X4:** fix-x4, but lowercase a chunk only when it contains no ASCII lowercase letter. A mixed-case chunk must
  stay as typed so it still meets X1's refusal and is not normalised by the indel search.
  - Output is lowercase, exit 5 (as `with-fix-x4.log`). Tests that change: none.
  - Closes FOLLOWUPS `toolkit-repair-indel-search-rejects-capitals`. Manual: the X1 case sentence covers it.
  - The ms primer's "retype in lowercase" (mnemonic-secret FIX 5.6.6) stays valid; no ms change needed.

## Deferred (toolkit `design/FOLLOWUPS.md`, status `deferred-minor`; each RED row gets `#[ignore = "deferred-minor: <slug>"]`)

- `bip388-literal-nums-key-refusal-text` (#21): the `--descriptor` bip388 refusal names the NUMS key as a bad
  use-site; use the ruled text. Architect question for the next export-wallet spec: should `--template … nums
  --format bip388` keep emitting a literal key (BIP-388 line 310)? The RED row must be rewritten, not just un-ignored.
- `specter-export-descriptor-mode-device-count` (#22): `devices` has 1 entry for an N-key `--descriptor` wallet;
  count keys per the ruling.
- `electrum-import-wallet-name-trailing-digits` (#23): strip only a `-<digits>` suffix (fix-23).
- `core-import-error-index-after-merge` (#58): after a receive/change merge, errors name the merged-list index;
  carry the original index (fix-58).
- `derive-child-dice-length-cap` (#15): a huge dice `--length` aborts on allocation; cap at 1..=1000000 per the ruling.
- `xpub-search-sh-wsh-default-origin-notice` (#24): the notice says `/2'` for `sh(wsh(...))`; use `1'` (fix-24).
- `convert-minikey-env-sentinel` (#25): `--from minikey=@env:VAR` is not resolved; use `is_argv_secret_bearing` (fix-25).
- `compare-cost-tr-leaf-hash-arg-inflated` (#27): `02` is glued onto 64-hex hash arguments in tap leaves (fix-27).
- `xpub-search-account-window-2p31-panic` (#32): panic for account windows of 2^31 or more; refuse per the ruling.
- `gui-schema-bundle-import-json-template-required` (#33): add `--import-json` to the rule; GUI `bundle()` in lockstep.
- `gui-schema-verify-bundle-template-required` (#34): add `--md1`/`--bundle-json`/positional; GUI `verify_bundle()` in lockstep.
- `verify-message-legacy-signature-trim` (#48): trim the legacy signature as the BIP-322 path does (fix-48).
- `toolkit-repair-inspect-ms1-env-sentinel` (X2): resolve `@env:` for `--ms1` on `repair`/`inspect`; no argv
  advisory for it. Owning phase: the erasure-fill spec.
- `toolkit-repair-ms1-stdin-second-line-ignored` (existing entry; re-scope to the X5 residual): a card wrapped over
  two stdin lines is read as two strings ("too many errors"); join per ruling 7. Owning phase: the erasure-fill spec.
- `compare-cost-concrete-keys-mixed-input` (#35, N): mixed abstract and hex input omits the concrete-keys note (fix-35).
- `friendly-ms1-length-set-uses-allowed` (#36, N): print the error's `allowed` set (fix-36).

Routed, not graded (the triage's "noticed" items, unverified by me; each needs its own triage):

- Toolkit Specter export writes `devices` as strings while Specter indexes `devices[i]["label"]`
  (`wallet_importer.py:154,160`). If confirmed, every Specter import of unknown keys may raise TypeError.
- `xpub-search --max-account 2147483648` aborts on a huge allocation.
- ms-codec's pre-dispatch length error lists only the entr set (`decode.rs:47-50`). That is an ms item.

Scratch worktree `/scratch/code/shibboleth/tk-worktrees/grade-g2-scratch` (detached, scratch patches only) is
left in place: RULES.md reserves worktree removal for Brian's typed go-ahead. Evidence:
`/scratch/cache/bughunt/tk/g2/`.
