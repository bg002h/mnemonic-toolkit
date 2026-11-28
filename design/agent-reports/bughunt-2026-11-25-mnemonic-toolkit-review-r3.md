# mnemonic-toolkit bug-hunt fold: independent review, round 3 (fold of r2)

Branch `claude/project-thread-utm1aa`, fold range `4eaf96a..14e85b3`:

| Commit | Content |
|---|---|
| f89d716 | I-A |
| 9015ce1 | I-B |
| 94e8055 | M5, M2, M3 |
| 4184500 | report and FOLLOWUPS |
| 14e85b3 | clippy allow |

How I worked:
- Read-only. No tracked file was edited, and only this report was added.
- I built at HEAD with `CARGO_INCREMENTAL=0 -j 2`, never `--release`.
- I probed with `target/debug/mnemonic`, and checked keys with the independent Python BIP-32 oracle (`scratchpad/r2py/b32.py`).
- I checked the BIP quotes against `scratchpad/bips/bip-0087.mediawiki` and `bip-0045.mediawiki`.

**Verdict: 0 Critical, 0 Important, 3 Minor, 4 Nit.**

- **I-A and I-B are fixed as r2 scoped them:**
  - The search now finds keys at the BIP-87 default and at m/45'.
  - A foreign xpub whose header contradicts a defaulted origin is refused, in both `bundle` and `verify-bundle`.
- **M5, M2 and M3 are fixed as claimed.**
  - M2: the manual's m/45' block is byte-identical to the binary (only argv0 differs).
  - M2: the BIP quotes are real.
- **I found no new defect of the classes asked about:**
  - The path order does not change which match is reported, because the purposes are disjoint.
  - No valid header-consistent xpub is refused.
  - I found no bypass of the new check.
  - The new tests assert what they claim.
- **The Minors are about text:**
  - The refusal's example path is wrong for the new shapes.
  - Stale counts remain in the manual.
  - The header check cannot see the purpose (a known, logged limitation).

## Tests run (targeted, plus the one `all` binary since it was built)

- **Lib:** `cargo test --locked -j 2 --lib candidate_paths`: 11/11 pass, including the new `bip87_accounts_and_bip45_are_searched` and `candidate_paths_len_matches_iter_count`.
- **Integration** (`--test all`, filtered), all pass:

  | Filter | Passed |
  |---|---|
  | `bughunt_1125` | 31 |
  | `bughunt_1119` | 14 |
  | `cli_bundle_h12_taproot_origin` | 4 |
  | `cli_non_canonical_descriptor` | 21 |
  | `cli_xpub_search` | 88 (3 ignored) |
  | `cli_shwpkh_canonical_flip` | 11 |
  | `cli_verify_bundle` | 84 |
  | `cli_slot_path_reaches_the_card` | 3 |
  | `cli_unrestorable_shape_advisory` | 13 |
  | `cli_restore_multisig_general` | 28 |
  | `cli_concrete_nonranged_xpub_reject` | 11 |

- **Whole `all` binary, once:** 2939 passed, 0 failed, 19 ignored. I ran it because I-B changes the accept set of every non-canonical xpub slot.
- **Format:** `cargo fmt --check -p mnemonic-toolkit` is clean.

## I-A: xpub-search did not search BIP-87 / m/45'. FIXED

`candidate_paths.rs:162-204` adds two families:
- `bip87` m/87'/<coin>'/<a>' over the account range. It uses `MultisigPathFamily::Bip87.default_origin_path`, the same coin as `bundle`: `coin_type()`.
- `bip45` m/45', once.

The order is single-sig, BIP-48, BIP-87, BIP-45, then add-paths. `len()` is `8n + 1 + add-paths`, which matches `iter()`; the unit test checks that against garbage add-paths too.

**Order safety:** `match_xpub_against_paths` returns the first byte-equal child.
- 87' and 45' differ in purpose from every earlier family, so no earlier candidate can produce the same key (short of a BIP-32 collision).
- Adding them cannot change which match is reported for a key the old set found.
- Only add-paths moved later. An add-path that duplicates 87' is now reported as `bip87` (Nit n1).

**Tests:**
- `b1_b2_b3_xpub_search_default_follows_the_ruling` now builds each descriptor from keys at the real `key_path` and asserts exit 0 plus `match: cosigner @0|@1  <key_path>`. It covers:
  - sh(wsh) 1';
  - bare sh at 87' and at m/45';
  - tr multi_a at 87' (the matched key is @1, because @0 is B's internal key);
  - wsh 2'.

  This closes r2's complaint that the test checked only the notice.
- `path_of_xpub_default_set_covers_bip87_and_bip45` asserts the path, the template (`bip87` / `bip45`) and the account (`2` / `null`).
- The old add-path tests moved from 87' to 88' so they still test add-path. The `bughunt_1119` order rows were re-pointed to 88' with recomputed counts. I checked 19/20/21 = 8·2 + 1 + the add-paths in front.

**Manual:**
- The candidate-list text (`41-mnemonic.md:4362-4366`) is accurate.
- The template-name list includes `bip87` / `bip45`, and `account` is `null` for bip45.
- Stale counts are in Minor m2.

## I-B: foreign xpub header on defaulted non-canonical slots. FIXED

`bundle.rs:1869-1873` and `verify_bundle.rs:1850-1860` call `check_defaulted_xpub_origin` when the index is in `canonical_filled` or in `defaulted`.

`defaulted` is computed after `--slot @N.path=` overrides are removed (`bundle.rs:2763`), so a typed path is never second-guessed. `anno_path` at that point is the bound default.

Probes:

| Input | Result |
|---|---|
| tr(NUMS,multi_a) + B at 48'/0'/0'/3' | exit 2, "depth 4, child number 3'" |
| the same with an inline fp-only `[3f635a63]@1` (also defaulted) | exit 2, so not a bypass |
| bare sh + B at m/45' | exit 2, "depth 1, child number 45'" (but see m1) |
| bare sh + B at m/45' with `--slot @1.path=m/45'` (+ `@0.path=m/45'`) | mints |
| B at 87'/0'/0' | mints. Card origins `87'/0'/0'` ×2 (test `r2_ib_foreign_xpub_at_the_bip87_default_still_mints`) |
| manual's bare-sh example (@1 xpub at 87'/0'/0') | still exit 0 |

**Other routes checked:**
- No other route mints a descriptor-mode default.
  - `DefaultOriginFamily::for_md_tree` is used only by bundle, verify-bundle and the shared binder.
  - The `synthesize.rs:1713` mirror is a test fixture.
- `master_xpub` is a template-mode Coldcard subkey and does not reach this arm.
- The SLIP-132 normalisation keeps the header.

**Tests:** `r2_ib_foreign_xpub_whose_header_contradicts_the_default_is_refused` covers both shapes (tr at 48'/3', bare sh at 48'/1'). It asserts:
- mint exits 2 with empty stdout and the header message;
- mint with the real path succeeds;
- verify-bundle without the path exits 2 with the same message;
- verify-bundle with the path exits 0.

The assertions are real.

**Real-world xpubs:** the check refuses only when depth or child number disagrees with the default. A header-consistent account xpub (Sparrow, Coldcard and Jade export real BIP-32 headers; mk1-rebuilt keys take their header from the origin) is refused only if it really does not sit at the default. The remedy is stated (`--slot @N.path=`). It fails closed.

## M5, M3, M2: done as claimed

**M5** (`bundle.rs:2900-2933`):
- Every BIP-87 default (non-bare-sh) appends "The same BIP-87 key serves every script type at one account: use a different --account for each wallet from this seed."
- `n == 1` raises the line to `warning:` and adds "BIP-87 is written for multisig; this card records the path, so restore does not depend on it."
- `n` counts distinct placeholders, so that matches r2's "single distinct key".
- Bare sh keeps its own warning, which already carries the `--account` sentence.
- The code comment's BIP-87 quote matches lines 105-106 of the BIP.
- Tests: `b3_*` and `tr_nums_sentinel_substitution_emits_bundle` assert the new text.
- The FOLLOWUP stays open for Brian, as r2 asked.

**M3** (`bundle.rs:2958-2965`): the line "m/45' has no coin or account level … pass the same --slot @N.path=m/45' to verify-bundle." prints right after the command. It is asserted in `b2_*` and mirrored in `41-mnemonic.md:709-714`. It is true: BIP-45 is "m / 45' / *", and r2 already verified that verify-bundle needs the path.

**M2:**
- **The example command** (`41-mnemonic.md:706-709`): I ran the manual's command with phrase A on stdin. The stderr is identical line for line except argv0 and the trailing stdout-secret warning. `'@0.path=m/45'\'''` is a well-formed POSIX word that yields `@0.path=m/45'`.
- **The BIP-87 quote** (`41-mnemonic.md:675-676`, `cli_bundle_h12_taproot_origin.rs:6-9`): "Any script that is supported by descriptors (and the specific wallet implementation) is compatible with this BIP." is BIP-87 line 152.
- **The BIP-45 quotes** in `candidate_paths.rs:129-132` are all real (lines 53, 56, 67-68):
  - "m / 45' / *"
  - "Hardened derivation is used at this level"
  - "Only the hardened purpose extended public key is shared"

**FOLLOWUPS:**
- `tk-bh1125-decision-xpub-search-sh-wsh` is honestly re-described and resolved by f89d716.
- `tk-bh1125-noncanonical-xpub-header` is raised to I and resolved by 9015ce1.
- r2's M4, M6 and nits are logged as open.

## Findings

### Critical
None.

### Important
None.

### Minor

**m1: the refusal's example path is wrong for every shape I-B newly reaches, and a typed path is not checked.**
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:562-567`.
- **What:** the message always says "(for example m/48'/0'/0'/2' for a BIP-48 P2WSH cosigner xpub)". I-B now fires it for taproot and bare-sh defaults, where that path is never right. Probe: a bare sh with an m/45' xpub prints that example beside "depth 1, child number 45'".
- **Why it matters:** `check_defaulted_xpub_origin` runs only when no path is typed. A user who copies the example therefore mints a card recording `48'/0'/0'/2'` for a depth-1 m/45' key. That is the same false-origin class I-B fixed, now one copy-paste away.
- **Fix:** derive the hint from the header:
  - depth 1 with child 45' gives `m/45'`;
  - depth 4 with child 3' or 1' gives `m/48'/<coin>'/<acct>'/3'` (or `/1'`);
  - depth 3 gives a BIP-87 or BIP-44-style path.

  Or drop the concrete example. Optionally refuse a typed `@N.path=` whose length or last component contradicts the xpub header, which the same function could do.

**m2: stale candidate counts and the stale advisory next to the updated text.**
- **Main manual** (`docs/manual/src/40-cli-reference/41-mnemonic.md`): the fold changed the text examples to 161, but the JSON examples in the same sections still say `"searched_count": 140` (lines 4306, 4319, 4439 `searched_count_per_cosigner`, 4698, 4714, 4731). The passphrase-of-xpub text example still says `searched: 140 candidate paths` (4660, 4681).
- **Advisory:** the passphrase-of-xpub advisory (`crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs:260-263`, quoted at manual 4636 and 4763) still says it searches "the standard BIP-44/49/84/86 + BIP-48 templates". That is now incomplete: it omits BIP-87 and m/45'.
- **Manual-gui** documents the GUI-pinned toolkit: CI's verify-examples installs `mnemonic-toolkit-v0.105.1` (`.github/workflows/manual-gui.yml:246-248`). The fold's manual-gui edits therefore run ahead of the pin:
  - `4d-…:271` and `4f-…:229` say 161;
  - the intros of `4e` and `4f` name BIP-87 / BIP-45.

  Meanwhile page 4e's included transcript (`transcripts/4e-xpub-search-passphrase-advisory.out`) still shows 140, which is correct for v0.105.1. So the same page now contradicts itself.
- **CI:** not broken, because the transcript is gated against v0.105.1.
- **Fix:**
  - Update the remaining main-manual counts and the advisory string (and its two manual quotes).
  - Either revert the manual-gui edits until the GUI pin moves, or log them with `tk-bh1125-gui-g1-manual`-style "runs ahead of the pin" wording.

**m3: the header check cannot see purpose or coin, and the new manual paragraph does not say so.**
- **Probe:** tr(NUMS,multi_a) with B's `m/86'/0'/0'` xpub (depth 3, child 0') and no path is accepted, and recorded at `87'/0'/0'`.
- **Status:** this limitation is known and logged (`tk-bh1125-r1-m4-header-check-blind-to-purpose`). I-B now extends the same check to non-canonical shapes.
- **Where the text falls short:** `41-mnemonic.md:683-689` says a slot "takes the default only when the xpub's own header agrees". That is literally true, but it reads as a guarantee.
- **Fix:** add "the header cannot show the purpose or coin, so a depth-3 key at another purpose still passes; give `--slot @N.path=` for every foreign key". That is the r1 M4 docs fix, applied to this paragraph too.

### Nit

- **n1:** an `--add-path` equal to a BIP-87 path (for example `m/87'/0'/account'`) is now matched by the built-in `bip87` family first. The JSON `template` field changes from the literal string to `"bip87"` for the same command (`candidate_paths.rs:204`). The path and account are unchanged. Worth one line in the manual's template-name paragraph.
- **n2:** `bip45` is searched after all account families (`candidate_paths.rs:204`). With a wide window (`--max-account 2147483647`, 8·2^31 candidates first) m/45' is effectively never reached. Putting `bip45` first costs one derivation and cannot change any other reported match, because the purposes are disjoint.
- **n3:** verify-bundle now also refuses `canonical_filled` header mismatches (`verify_bundle.rs:1854`). Before, it reported mismatch rows, so the accept set on canonical shapes widened slightly beyond I-B. This is consistent with open decision `tk-bh1125-decision-verify-bundle-strict` and with bundle's own refusal, but the commit and FOLLOWUP do not mention it.
- **n4:** in the bare-sh output, the redaction note "secret values … are not repeated above" now follows the new M3 line rather than the command (`bundle.rs:2966-2973`). Move the M3 line after the redaction note.
