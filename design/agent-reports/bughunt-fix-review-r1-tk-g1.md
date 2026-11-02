**⚠ 4 Important (I-1 #46 remedy fails when piped; I-2 #37 expected-ms1 computation unspecified; I-3 #45 refuses signet/regtest default envelopes and mislabels; I-4 manual 5.2 places #3 under a wallet-changing Remedy)**

# FIX review r1: tk bug-hunt G1 (`design/FIX_bughunt_tk_g1.md` at 7115c685)

Reviewer: independent adversarial, Opus (xhigh). Scratch worktree `review-fix-r1-g1` (detached 7115c685) with the author's reference patch applied to build a fixed binary; a second build of unmodified 7115c685 for "current". First round: no finding came from a previous fix commit.

## Findings (appended as found)


### I-1 (Important, advice lens) #46 remedy "then pass it as --slot @<N>.phrase=-" fails when followed literally

- **Location:** FIX 2.22 first bullet (line 1230) and its transcripts (1246, 1258); RED row `bughunt_tk_46_…` (1673); manual 5.1 new text (1429); section 11 item 1 (1803).
- **Failure.** `convert --to phrase` prints `phrase: <words>`, so "it" (convert's output) carries a label. A holder who pipes convert into bundle, the natural reading of "it reads the value from stdin, then pass it as … =-", gets a misleading refusal that never names the label. Run on the patched build:
  `printf "%s\n" "$E0" | mnemonic convert --from entropy=- --to phrase | mnemonic bundle --network mainnet --import-json env-3of3.json --slot @0.phrase=- --json` exits 1 with `error: --slot @0.phrase=: BIP-39 parse error: mnemonic has an invalid word count: 13. Word count must be 12, 15, 18, 21, or 24`. It fails closed, but the printed remedy chain does not work as written and the path needs only one condition (piping), so the likelihood test leaves it Important.
- **Fix text.** In the 2.22 seed-subkey bullet, both transcripts and the RED row's expected stderr (with `entropy`/`ms1`, `0` substituted), replace
  `Get the phrase with: mnemonic convert --from <subkey>=- --to phrase (it reads the value from stdin), then pass it as --slot @<N>.phrase=-`
  with
  `Get the phrase with: mnemonic convert --from <subkey>=- --to phrase (it reads the value from stdin and prints "phrase: <words>"), then give the words alone, without the "phrase: " label, on stdin to --slot @<N>.phrase=-`
  In manual 5.1 replace `get the phrase of a seed card with `mnemonic convert --from <subkey>=- --to phrase` and overlay it with `--slot @N.phrase=-`.` with `get the phrase of a seed card with `mnemonic convert --from <subkey>=- --to phrase` (it prints `phrase: <words>`) and give the words alone, without the `phrase: ` label, on stdin to `--slot @N.phrase=-`.` Section 11 item 1 quotes the new text.
- **Run as printed (patched build, fixtures `fx/`).** Step 1: `printf '%s\n' "$E0" | mnemonic convert --from entropy=- --to phrase` exits 0 with `phrase: abandon … about`. Step 2, the words alone: `printf '%s\n' "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" | mnemonic bundle --network mainnet --import-json env-3of3.json --slot @0.phrase=- --json` exits 0 with `"mode":"full"`. (A working one-line alternative: `… convert … --json | jq -r '.to[0].value' | mnemonic bundle … --slot @0.phrase=-` exits 0, but it needs jq.)

### N-1 (Nit) Line 1321 says "The reference patch implements all of it", but the patch has no `crates/wc-codec/src/lib.rs` hunk
The two rustdoc edits that 2.24, section 3 item 7 and the build-impact item 7 name are missing from the reference patch (`lib.rs:69-70`, `lib.rs:146-148` are unchanged with it applied). The edits are fully specified in 2.24, so a builder working from the document writes them. Fix: drop the claim or add the hunk to the reference patch.

### I-2 (Important, builder must choose) #37: FIX 2.10 does not say how the expected ms1 string is computed, and the obvious shortcut gives a false `mismatch` for every non-English seed

- **Location:** FIX 2.10 "Behaviour" (line 834), the sentence "The expected card is the canonical seed slot's (`resolved`, which keeps the entropy under `--origin`), never `resolved_for_synth`."
- **Gap.** `ms1_card_checks` compares strings (`supplied_ms1 == expected_ms1`, `verify_bundle.rs:2344` patched). An ms1 card carries the wordlist language: `synthesize_unified` emits `Payload::Mnem{language,…}` for a non-English seed and `Payload::Entr` only for English (`synthesize.rs:1152-1170`). The document names the slot but not the call, so a builder must choose how to produce "the canonical seed slot's card". The direct reading, encoding `canonical_slot.entropy` as `ms_codec::Payload::Entr`, passes every RED row (all English). It then prints `✗ ms1_entropy_match: expected ms1 bytes differ from supplied` (exit 4) for the holder's own correct card whenever the seed is Japanese, Spanish and so on. That is a wrong verdict on a correct seed card.
- **Measured (reference patch).** A Japanese seed (16 zero bytes) gives `ms1[0]` = `ms10entrsqgqs…` (a Mnem payload), not `M0`'s `ms10entrsqqqq…`. The reference verifies it correctly: exit 0 with `✓ ms1_entropy_match: ms1 byte-identical`.
- **Fix text.** In 2.10 "Behaviour", after "never `resolved_for_synth`." insert:
  "Compute it as `ms1[0]` of a second `crate::synthesize::synthesize_unified(&resolved, template, 1, args.network, args.privacy_preserving, args.language.unwrap_or_default().into(), crate::synthesize::Md1Form::Template)` call. Use the same arguments as the existing `expected` call, with `resolved` in place of `resolved_for_synth`, and never encode the entropy directly. The card carries the wordlist language (`Mnem` for a non-English seed), and the compare is by string."
  Add a control assertion to the RED row `bughunt_tk_37_…`. JP is the Japanese phrase of 16 zero bytes (`あいこくしん` ×11 then `あおぞら`). First run `mnemonic bundle --network mainnet --template bip84 --md1-form template --group-size 0 --language japanese --slot @0.phrase=@env:JP --json`. Then run `mnemonic verify-bundle --network mainnet --language japanese --slot @0.phrase=@env:JP --ms1 - <its mk1, md1>` with that bundle's `ms1[0]` on stdin. Expected: exit 0, and stderr contains `✓ ms1_entropy_match: ms1 byte-identical`.
- **Run as printed** (patched build): the bundle exits 0 and the verify exits 0 with that line.

### I-3 (Important, missing case) #45: a signet or regtest envelope from a default `import-wallet` is now refused, and the remedy relabels the bundle `testnet`

- **Location:** FIX 2.21 "Behaviour" (line 1200); manual 5.1 (line 1429); the RED row for #45.
- **Failure.** `import-wallet` without `--network` records every coin-type-1 wallet as `testnet`. Its own `--network` help says it exists "to disambiguate signet/regtest from the coin-type-1→testnet collapse". So a signet holder's default envelope says `testnet`. Today `bundle --network signet --import-json env-testnet.json --json` exits 0 and the bundle says `"network":"signet"`; the same holds for regtest (measured on the current build). After the fix it exits 1: `error: --network signet disagrees with the --import-json envelope's bundle.network (testnet); re-run with --network testnet`. Following that remedy gives a bundle labelled `"network":"testnet"` for a signet wallet. The cards are byte-identical across testnet, signet and regtest (measured: same sha256 of the JSON without `network`), so no key or address is wrong. But the fix newly refuses a legitimate, common flow (this project's test seeds are signet only) and prints wrong data. The path needs one condition (a signet or regtest wallet imported with the default), so it is not Minor. The document does not decide this case.
- **Fix text.** In 2.21 "Behaviour", after the sentence ending `` `regtest`). ``, add:
  "When the envelope's `bundle.network` is `testnet` and `--network` is `signet` or `regtest` (an envelope imported without `--network` records every coin-type-1 wallet as testnet), the text is instead `--network <flag> disagrees with the --import-json envelope's bundle.network (testnet); if the wallet is on <flag>, re-create the envelope with import-wallet --network <flag> from the same blob, otherwise re-run with --network testnet`."
  Add the transcript and RED assertion. `mnemonic bundle --network signet --import-json env-testnet.json --json` gives exit 1, nothing on stdout, and stderr equal to that text with `<flag>` = `signet`. The remedy `mnemonic import-wallet --format bsms --blob tests/fixtures/wallet_import/bsms-testnet-2of2.txt --network signet --json > env-signet.json`, then `mnemonic bundle --network signet --import-json env-signet.json --json`, gives exit 0 with `"network":"signet"`.
  In manual 5.1, after `a disagreement is refused (exit 1).` insert ` An envelope imported without `--network` records a signet or regtest wallet as testnet: re-create it with `import-wallet --network signet` (or `regtest`).`
  If the architect prefers to keep today's flow instead, the alternative is to refuse only a cross-class disagreement (mainnet against the testnet class) and honour a same-class `--network` as `import-wallet --network` does. That is a design choice; the text above keeps the grade's ruled refusal.
- **Run as printed (patched build).** `import-wallet --format bsms --blob <crate>/tests/fixtures/wallet_import/bsms-testnet-2of2.txt --network signet --json` exits 0, and the bundle from it exits 0 with `{"network":"signet"}`. The same with `regtest` gives `{"network":"regtest"}`. The reverse (`--network testnet` on the signet envelope) is refused with `re-run with --network signet`, which is correct.

### I-4 (Important, advice lens) Manual 5.2 puts the new #3 paragraph directly above a **Remedy** that, read as applying to it, rewrites the holder's wallet into another one

- **Location:** FIX 5.2 (lines 1432-1454). The insertion point is "before the `Remedy:` paragraph" in `41-mnemonic.md` (currently line 296-298).
- **Failure.** FIX 2.16 says "The text gives no 'add `/*`' remedy on purpose: that describes another wallet." The manual edit, however, inserts the new paragraph immediately above `**Remedy:** rewrite the use-site path as the explicit multipath form — … one `/<0;1>/*` entry …, or use the `/**` shorthand for the same thing.` A holder refused for `wpkh([fp/84'/0'/0']xpub/<0;1>)` reads the section top-down, takes the Remedy as applying to their case and adds `/*` (or `/**`). They then get an md1 for `xpub/0/*` and `xpub/1/*`, a different address set from the two keys `xpub/0` and `xpub/1` their wallet uses (BIP-389 non-ranged multipath). The tool accepts it (exit 0), so the backup card is for another wallet. That is a wrong descriptor reached by following the manual. It does not fail closed, so it stays Important however rare the shape is.
- **Fix text.** Replace FIX 5.2 with this edit to `docs/manual/src/40-cli-reference/41-mnemonic.md`, after the Remedy, so it cannot read as covered by it. Replace, once (the anchor occurs once at 7115c685, checked):
  ~~~text
  the `/**` shorthand for the same thing.

  **Sparrow is unaffected.**
  ~~~
  with:
  ~~~text
  the `/**` shorthand for the same thing.

  A **multipath with no trailing wildcard** — `@0/<0;1>` (or an inline
  `[fp/path]xpub/<0;1>`) with no `/*` after it — names a non-ranged wallet
  that md1 cannot encode either. It used to be encoded silently as the
  ranged `/<0;1>/*` wallet; it is now refused (exit 2, `DescriptorParse`) on
  the same surfaces. Bare `@N`, with no use-site at all, is unaffected. The
  remedy above does **not** apply: adding `/*` describes a different wallet
  (`/<0;1>/*` ranges over `/0/i` and `/1/i`; `/<0;1>` is the two keys `/0`
  and `/1`). md1 has no card for it; keep the full descriptor.

  **Sparrow is unaffected.**
  ~~~
  Change 5.2's title to "(a new paragraph after the `Remedy:` paragraph)".

### M-1 (Minor) The new spec 4.1 (b) and (c) sentences on `--expect-wallet-id` leave out the `--origin` exception that 2.13 rules

- **Location:** FIX 4.1 (b, c) new text (lines 1363-1364).
- **What the text says.** (c) says "an empty, odd-length or non-hex, or longer value is refused (`BadInput`, exit 1) on `restore` and `verify-bundle` alike". But 2.13 rules that the decode is "not under `--origin`, which still prints the existing notice and checks nothing". (b) says "The `UNVERIFIED` banner prints only when no reference (`--expect-wallet-id`, …) is given". Under `--origin`, however, a given `--expect-wallet-id` is not read, yet it still suppresses the banner.
- **Measured (patched build).**
  - `printf '%s\n' "$PA" | mnemonic restore --network mainnet --md1 md1fvwrwqqpqqgqpsqqq3uaau4ctxyl7 --from phrase=- --origin "m/84'/0'/5'" --expect-wallet-id ''` exits 0. The same holds with `zz` or `deadbeef`. stderr has only `notice: --expect-wallet-id is not checked when --origin overrides the canonical account path …` and no `UNVERIFIED` line.
  - Without `--expect-wallet-id`, the same command prints the `UNVERIFIED:` banner.
  - `verify-bundle … --origin … --expect-wallet-id ''` prints the same notice.
  - The banner behaviour is the same on the unmodified build, so it is pre-existing. The new sentence writes it into the spec without the exception.
- **Likelihood.** No wrong wallet is printed: it is the `--origin` wallet the holder asked for, and the notice says the id is unchecked. No printed command fails. The path needs two conditions: (1) `--origin` and (2) `--expect-wallet-id` as the only reference, or a malformed value.
- **Fix text.**
  - In (c), replace `on `restore` and `verify-bundle` alike (FIX_bughunt_tk_g1 #47).` with `on `restore` and `verify-bundle` alike, except under `--origin`, where the value is not read and a notice says it is not checked (FIX_bughunt_tk_g1 #47).`
  - In (b), after `is given.` append ` Under `--origin` a given `--expect-wallet-id` is not checked (a notice says so) and the banner does not print.`

## Case table (every fix; outputs from the patched build unless "cur")

The author's 90 cases (`cases.py`), re-run on my two builds, match the document on every case (cur and fix). All 538 transcript lines and the JSON details match. Additional cells I ran are listed below; "OK" means the result matches the document's ruling.

| fix | cells run | result |
|---|---|---|
| #1 | 01 | OK |
| #2 | 02a-f; zpub `--expect-xpub` (pre-existing in seed-only restore as well) | OK |
| #10 | 10, 10b, 10c; RED row 10b runs 1/2/3 as printed (27 lines, `9 assignments match prefix a693`, the file is byte-equal, the JSON has `match_count` 9 and 9 candidates and no `wallets` key) | OK |
| #40 | 40a, 40b; RED row 40b as printed (golden pair; cur gives `bc1q2a4vm7…`) | OK |
| #41 | 41, 41b-d (all three advice branches) | OK |
| #42 | 42, 42b | OK |
| #53 | 53 | OK |
| #38 | 38a-d (the `sortedmulti_a` `/<2;3>/*` first receive address is `bc1pv963…`) | OK |
| #11 / 11b | 11, 11b1-4, 11p, 11q, 11r1-4; the refusal comes before the seed read | OK |
| #37 | 37a-c; Japanese seed (Mnem payload) → exit 0; `--bundle-json` route (own card exit 0, other seed's card exit 4); `--slot @0.phrase=`, `.entropy=`, `.ms1=` and `.seedqr=` all verify (manual 5.3) | OK; the expected-card computation is unspecified → **I-2** |
| #39 | 39a-c | OK |
| #44 | 44a-c | OK |
| #47 | 47a-d; keyed bundle `''` and `zz` are ignored (exit 0, as the manual says "ignored otherwise"); `--origin` cells | OK; spec wording → **M-1** |
| #52 | 52a-b | OK |
| #12 / #13 / #43 | 12k1-8, 13k1-8, 43 | OK |
| #3 | 03a-d | OK; manual placement → **I-4** |
| #4 | 04a-b; `is_cjk` has 29 ranges, identical to `electrum/mnemonic.py` 40-70 at `e1099925` (raw source) | OK |
| #5 | 05, 05b, 05r; `tr-multi-a` and `tr-sortedmulti-a` still hit the older refusal first (exit 1 on both builds), so manual 5.7's "every taproot template" holds; `wallet.py:4241` checked against raw source | OK |
| #6 | 06a-c | OK |
| #9 | 09 | OK |
| #45 | 45a-b; signet/regtest envelope from a default `import-wallet` | **I-3** |
| #46 | 46a-g; convert piped into bundle | **I-1** |
| #54 | 54, 54t; 9.3's five GUI transcripts are exactly the files with `"path_family":"bip87"` (all descriptor mode) | OK |
| #8 | tightened row as a scratch test: `u_slots` 8 → `InvalidParams`, 7 round-trips | OK |

- **RED rows (section 6).** Every existing row name exists in `bughunt_s1.rs`, `bughunt_s2.rs` or the wc-codec `bughunt_s2.rs`, and no G1 test in those files is missing from the table (28 = 28). The six NEW rows are absent, as expected. Every quoted assertion string was matched against the patched build's output (a scripted check of 44 substrings): all match.
- **Anchors (sections 4 and 5).** All 14 manual and 7 spec old-text anchors occur exactly once in their target file. The `41-bundle-inheritance-json.out` hit under `docs/quickstart/transcripts` is the same file through a symlink.

## Design concerns (ungraded)

- The 11b `restore` refusal does not name the card's actual use-site.
- `--expect-xpub` does not accept SLIP-132 forms (`zpub…`); this is pre-existing.
- Under `--origin` with `--expect-wallet-id` as the only reference, there is neither a check nor an `UNVERIFIED` banner (M-1's behaviour); the architect may want the banner back there.

## Accounting

First round (r1): 0 of these findings came from a previous fix commit (none exists; the document is the author's first submission).

C / I / M / N: 0 / 4 / 1 / 1
r1: 0C/4I/1M/1N
