# FIX sec-multipath-duplicate-refusal: refuse a multipath tuple that repeats a value (BIP-389)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-23. Track: **small** (no new flag, type, command or public
API; one new refusal text in an existing error variant; one spec paragraph, one manual paragraph and two
manual table rows). Base: origin/master `68ea05e0` (toolkit 0.107.1, md-codec pin `cf35d61a` = md-codec 0.47.0).
Unit: security sweep `multipath-duplicate-refusal` (coordinator `reports/security-sweep/grade/routing.md`
L43; graded `agent-reports/grade-security-sweep-toolkit-gui.md` L58, R02 tsv108/tsv109, regraded M to I).
RED probe: `design/fix-sec-multipath-duplicate-refusal-red.sh <mnemonic-binary>` (committed with this FIX).

## 1. Defect at `68ea05e0` (measured, section 5)

`lex_placeholders` (`crates/mnemonic-toolkit/src/parse_descriptor.rs` L61) validates each `;`-separated value
of a `/<...>` multipath body (L147-L179: hardened refused, non-integer refused) but never compares the
values with each other. A tuple that repeats a value (`/<0;0>/*`, `/<1;1>/*`, `/<0;1;0>/*`, `/<1;01>/*`) is
accepted, encoded and engraved with exit 0 (R1-R5), imported (R6), verified (R7: verify-bundle reaches the
card comparison instead of refusing the descriptor) and classified `canonical` (R10). md-codec 0.47.0 does not
catch it either: `UseSitePath::write` (`use_site_path.rs` L95-L98) checks only the alt count 2..=9.

External facts (checked 2026-11-23 against the raw BIP texts in mnemonic-refugium
`scripts/factcheck/sources/`):
- BIP-389 `bip-0389.mediawiki` L50: "Duplicate <tt>NUM</tt>s within a tuple are not allowed."
- BIP-388 `bip-0388.mediawiki` L153: "a string of the form <tt>/<NUM;NUM>/*</tt>, for two distinct decimal
  numbers <tt>NUM</tt> representing unhardened derivations".

Holder harm (why refuse, not warn; Brian's ruling "warn-and-proceed over refusals where no funds risk" does not
apply): with `<0;0>` the card's receive and change chains are one chain. The likely source is a typo for
`<0;1>`; the engraved card then lacks the change chain, so change sent to chain 1 by the real wallet is not
found from the card. A BIP-389 parser must refuse the descriptor the card restores to. The routing row and the
grade both say refuse.

## 2. Layer decision: the toolkit lexer (md-codec gets a companion FOLLOWUP)

The check goes in the toolkit's `lex_placeholders` multipath-body validator, not in the md-codec encoder.
Reasons:
1. The lexer is the single chokepoint for every toolkit descriptor-text surface: `parse_descriptor` (L970)
   calls it, and `bundle`, `verify-bundle`, every `import-wallet` format that reaches `parse_descriptor`
   (bsms, bitcoin-core, coldcard, coldcard-multisig, descriptor, electrum, sparrow, specter),
   `export-wallet --from-import-json` (`export_wallet.rs` L1024) and `gui-schema --classify-descriptor`
   (`gui_schema.rs` L1319) call `parse_descriptor`. The concrete-key form is rewritten to `@N` first
   (`concrete_keys_to_placeholders`), so one check covers both forms (R1, R2).
2. The validator already owns the per-value rules for the same body (H13 hardened, non-integer), so the
   new rule sits with its siblings and keeps their precedence.
3. An md-codec fix needs an md-codec release and a toolkit git-pin bump (`Cargo.toml` L73), and changes
   md-codec's encoder validity (the decoder must keep reading cards already engraved): cross-repo, full track. md-codec does hold BIP rules and
   `md encode` has the same defect (md 0.20.7: `md encode 'wpkh(@0/<0;0>/*)'` exits 0 and `md decode` returns
   `wpkh(@0/<0;0>/*)`), so a companion FOLLOWUP is proposed for md-codec (section 8). When md-codec refuses,
   the toolkit lexer still refuses first, with this text.

## 3. Spec delta (what the builder changes)

**Rule D (duplicate value).** In `lex_placeholders`, inside the multipath-body validator, after every value of
the body has passed the existing per-value checks (hardened, bare unsigned integer), refuse the body when two
values are equal as parsed `u32` numbers. Scan left to right; the refusal names the first value that equals an
earlier value. The refusal is a `ToolkitError::DescriptorParse` (exit 2) with this exact payload, where `<i>` is
the `@N` index, `<body>` is the body exactly as captured (the text between `<` and `>`) and `<v>` is the
repeated value in decimal without leading zeros:

    @<i> multipath `<<body>>` repeats the value <v>; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)

Example (R1): ``@0 multipath `<0;0>` repeats the value 0; BIP-389 forbids duplicate values in a multipath
tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)``.

Decided edge cases:
- Comparison is by value: `<1;01>` repeats 1, and the message shows the body as typed (`<1;01>`) and `1` (R4).
- Non-adjacent repeats are refused: `<0;1;0>` names 0 (R3); `<1;0;0;1>` names 0, the first value equal to an
  earlier one (unit row U3).
- Precedence inside one body: a hardened or non-integer value keeps its existing refusal, because the per-value
  checks run first (`<0;0'>` gives the hardened text, G3; `<0;x;0>` gives the non-integer text, G4).
- Precedence against later checks of the same placeholder: Rule D is part of the validator, so it runs before
  the residue check (L184-L217) and the no-wildcard check (L218-L227), as the hardened check does (SPEC
  cycleA §5 "AFTER the multipath validator"). `@0/<0;0>` (R8) and `@0/<0;0>/0/*` (R9) get the Rule D text.
- Several placeholders: occurrences are lexed in text order; the first offending occurrence is reported (R5
  names `@1` when `@0` is `<0;1>`).
- Distinct tuples are unchanged: `<0;1>`, `<1;0>`, `<0;1;2>` accepted (G1, G2, U10). A single-value `<5>` keeps
  md-codec's `md1 multipath alt-count 1 out of range (2..=9)` refusal (not a duplicate).
- Different placeholders may use the same tuple (`@0/<0;1>/*,@1/<0;1>/*`): Rule D is within one tuple only.
- Old cards: an md1 card already engraved with `<0;0>` still decodes (md1 decode is md-codec's, untouched);
  `verify-bundle --descriptor` with that descriptor now refuses (Brian's ruling: no backward-compatibility
  work for old files yet). `export-wallet --from-import-json` on an older envelope whose descriptor repeats a
  value exits 1 (`BadInput`) with the existing prefix `--from-import-json: cannot encode the envelope
  descriptor to derive K: ` followed by the Rule D payload.

Holder-visible surfaces (stdout empty on every refusal):

| Surface | Exit | Stderr |
|---|---|---|
| `bundle --descriptor`, `verify-bundle --descriptor`, `gui-schema --classify-descriptor` | 2 | `error: ` + payload |
| `import-wallet --format descriptor` | 2 | `error: import-wallet: descriptor: parse error: ` + payload |
| other `import-wallet` formats | 2 | `error: ` + the format's existing parse-error prefix + payload |
| `export-wallet --from-import-json` | 1 | `error: --from-import-json: cannot encode the envelope descriptor to derive K: ` + payload |

No new error variant, exit code, flag or `--json` field.

**SPEC lockstep.** `design/SPEC_cycleA_descriptor_use_site_collapse.md` §5 gains, after its last paragraph
(the "**Does NOT catch**" paragraph, L120-L122), this paragraph:

> **Duplicate multipath values (FIX sec-multipath-duplicate-refusal, 2026-11-23).** The multipath-body
> validator also refuses a tuple that repeats a value (BIP-389: "Duplicate NUMs within a tuple are not
> allowed"), comparing parsed values after the per-value checks: `<0;0>`, `<0;1;0>` and `<1;01>` are refused,
> and a hardened or non-integer value keeps its own refusal. Being part of the validator, it runs before the
> residue check and the no-wildcard check. Text: `design/FIX_sec_multipath_duplicate_refusal.md` §3.

**Code comment.** The validator comment (L150-L159) that lists what it accepts ("accepting valid `<0;1>` /
`<0;1;2>`") gains "and refuses a repeated value (`<0;0>`, BIP-389)". The `lex_placeholders` regex comment
(L64-L96) is otherwise unchanged.

## 4. Manual lockstep (toolkit manual, `docs/manual/src/40-cli-reference/41-mnemonic.md`)

No flag changes. Printed text changes, so the manual gains:

1. In "Non-representable use-site steps" (L280), a new paragraph after the paragraph that ends "md1 has no card
   for it; keep the full descriptor." (L329) and before "**Sparrow is unaffected.**":

   > A **multipath that repeats a value** — `<0;0>`, `<1;1>`, `<0;1;0>` (values compare as numbers, so
   > `<1;01>` repeats 1) — is invalid under BIP-389, which forbids duplicate values in a tuple; with `<0;0>`
   > the receive and change chains would be one chain. It is refused (exit 2, `DescriptorParse`; for
   > `import-wallet`, exit 2 `ImportWalletParse`) on the same surfaces, before the fixed-step and
   > no-wildcard checks above. **Remedy:** copy the tuple from the wallet's own exported descriptor; most
   > wallets use `/<0;1>/*` (receive 0, change 1). A guessed tuple describes a different wallet.

2. In the `mnemonic bundle` refusal table, a row after the fixed-step row (L878):

   | `@N`/key multipath repeats a value (`<0;0>`, `<0;1;0>`) | refused, exit 2 (`DescriptorParse`) — ``error: @N multipath `<0;0>` repeats the value 0; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)``; see [Non-representable use-site steps](#non-representable-use-site-steps) |

3. In the `mnemonic import-wallet` refusal table, a row after the fixed-step row (L1897):

   | Descriptor's key placeholder multipath repeats a value (`<0;0>`, `<0;1;0>`) | exit 2 `ImportWalletParse` — the `bundle` text after the format's parse-error prefix (``import-wallet: descriptor: parse error: @0 multipath `<0;0>` repeats the value 0; …``); see [Non-representable use-site steps](#non-representable-use-site-steps) |

The GUI manual (`docs/manual-gui`) and the mnemonic-gui schema mirror are unaffected (no flag, option or
dropdown change). md-cli text is unchanged by this FIX.

**CHANGELOG** (`CHANGELOG.md`, under `## [Unreleased]`, a new subsection):

    ### A multipath tuple that repeats a value is refused

    - **`bundle`, `verify-bundle`, `import-wallet`, `gui-schema --classify-descriptor`**: a descriptor whose
      multipath tuple repeats a value (`/<0;0>/*`, `/<0;1;0>/*`) now exits 2 with ``@0 multipath `<0;0>` repeats
      the value 0; BIP-389 forbids duplicate values in a multipath tuple, …``. BIP-389 does not allow duplicate
      values in a tuple; such a descriptor used to be engraved with exit 0, its receive and change chains the
      same chain. `export-wallet --from-import-json` refuses such an envelope (exit 1).

## 5. RED rows (run on unmodified `68ea05e0`)

Test file for the CLI rows: new `crates/mnemonic-toolkit/tests/cli_multipath_duplicate_refusal.rs` (signet
tpubs from committed fixtures, no seed; `T84` = the `[73c5da0a/84'/1'/0']` tpub, `TA`/`TC` = the
`[73c5da0a/48'/1'/0'/2']` and `[28645006/48'/1'/0'/2']` tpubs). The lexer rows are unit tests in a new
`#[cfg(test)] mod multipath_duplicate_tests` at the end of `src/parse_descriptor.rs` (`lex_placeholders` is not
public outside `cfg(fuzzing)`, `lib.rs` L178-L179); they compare `e.message()` with the payload, and `Ok` as
`Ok(<n> occurrence(s))`. Each CLI row asserts exit code, empty stdout and the whole stderr (`error: ` + text +
newline). `P(i, body, v)` is the Rule D payload.

The 16 tests were written as scratch, run on `68ea05e0` with `cargo nextest run --locked -p mnemonic-toolkit
--build-jobs 4 -j 4 --no-fail-fast -E 'test(/multipath_duplicate|duplicate_multipath|distinct_multipath|_precedes_/)'`
(16 matched: "16 tests run: 4 passed, 12 failed, 4306 skipped"), then removed; the builder re-creates them from
this table.

| Row | Test function | Input | At `68ea05e0` (measured) | After the fix |
|---|---|---|---|---|
| U1 | `lex_rejects_duplicate_multipath_value_pair` | `wpkh(@0/<0;0>/*)` | FAIL: left `"Ok(1 occurrence(s))"` | `P(0,"0;0",0)` |
| U2 | `lex_rejects_duplicate_multipath_value_nonadjacent` | `wpkh(@0/<0;1;0>/*)` | FAIL: `"Ok(1 occurrence(s))"` | `P(0,"0;1;0",0)` |
| U3 | `lex_rejects_duplicate_multipath_value_first_repeat_named` | `wpkh(@0/<1;0;0;1>/*)` | FAIL: `"Ok(1 occurrence(s))"` | `P(0,"1;0;0;1",0)` |
| U4 | `lex_rejects_duplicate_multipath_value_leading_zero` | `wpkh(@0/<1;01>/*)` | FAIL: `"Ok(1 occurrence(s))"` | `P(0,"1;01",1)` |
| U5 | `lex_rejects_duplicate_multipath_value_second_placeholder` | `wsh(multi(2,@0/<0;1>/*,@1/<2;2>/*))` | FAIL: `"Ok(2 occurrence(s))"` | `P(1,"2;2",2)` |
| U6 | `lex_duplicate_refusal_precedes_no_wildcard_refusal` | `wpkh(@0/<0;0>)` | FAIL: left = the no-wildcard text ``@0: a multipath use-site `/<a;b>` with no trailing `/*` …`` | `P(0,"0;0",0)` |
| U7 | `lex_duplicate_refusal_precedes_residue_refusal` | `wpkh(@0/<0;0>/0/*)` | FAIL: left = the residue text ``@0: derivation steps after the placeholder are not representable in md1; …`` | `P(0,"0;0",0)` |
| U8 | `lex_hardened_refusal_precedes_duplicate_refusal` (guard) | `wpkh(@0/<0;0'>/*)` | PASS | unchanged: ``@0 multipath alternative `0'` is hardened; …`` |
| U9 | `lex_non_integer_refusal_precedes_duplicate_refusal` (guard) | `wpkh(@0/<0;x;0>/*)` | PASS | unchanged: ``@0 multipath alt `x` is not a bare unsigned integer`` |
| U10 | `lex_distinct_multipath_values_still_accepted` (guard) | `<0;1>`, `<0;1;2>`, `<1;0>` | PASS | unchanged: `Ok(1 occurrence(s))` each |
| C1 | `bundle_concrete_duplicate_multipath_refused` | `bundle --network signet --descriptor "wpkh([73c5da0a/84h/1h/0h]T84/<0;0>/*)" --slot @0.xpub=T84` | FAIL: left `Some(0)`, right `Some(2)` | exit 2, stdout empty, stderr `error: P(0,"0;0",0)` |
| C2 | `bundle_template_duplicate_multipath_refused` | `bundle … --descriptor "wpkh([73c5da0a/84h/1h/0h]@0/<1;1>/*)" --slot @0.xpub=T84` | FAIL: `Some(0)` vs `Some(2)` | exit 2, `error: P(0,"1;1",1)` |
| C3 | `bundle_multisig_second_key_duplicate_multipath_refused` | `wsh(sortedmulti(2,[73c5da0a/48h/1h/0h/2h]TA/<0;1>/*,[28645006/48h/1h/0h/2h]TC/<2;2>/*))`, both slots `.xpub` | FAIL: `Some(0)` vs `Some(2)` | exit 2, `error: P(1,"2;2",2)` |
| C4 | `import_wallet_descriptor_duplicate_multipath_refused` | `import-wallet --network signet --format descriptor --blob <file holding the C1 descriptor>` | FAIL: `Some(0)` vs `Some(2)` | exit 2, `error: import-wallet: descriptor: parse error: P(0,"0;0",0)` |
| C5 | `verify_bundle_duplicate_multipath_refused` | `verify-bundle --network signet --mk1 '' --md1 '' --descriptor <C1 descriptor> --slot @0.xpub=T84` | FAIL: `Some(4)` vs `Some(2)` | exit 2, `error: P(0,"0;0",0)` |
| C6 | `bundle_distinct_multipath_still_accepted` (guard) | C1 with `/<0;1>/*` and with `/<0;1;2>/*` | PASS | unchanged: exit 0, stdout carries md1 |

RED probe output at `68ea05e0` (`design/fix-sec-multipath-duplicate-refusal-red.sh
target/debug/mnemonic`, verbatim; `out` is stdout bytes, `err` the first stderr line):

    R1 rc=0 out=491 err=# === Wallet bundle: descriptor, signet ===
    R2 rc=0 out=491 err=# === Wallet bundle: descriptor, signet ===
    R3 rc=0 out=493 err=# === Wallet bundle: descriptor, signet ===
    R4 rc=0 out=491 err=# === Wallet bundle: descriptor, signet ===
    R5 rc=0 out=907 err=# === Wallet bundle: descriptor, signet ===
    R6 rc=0 out=333 err=note: stdout is watch-only — public keys only, cannot spend
    R7 rc=4 out=563 err=
    R8 rc=2 out=0 err=error: @0: a multipath use-site `/<a;b>` with no trailing `/*` (a non-ranged multipath) is not representable in md1, which encodes only a ranged `/<a;b>/*` or bare `/*` use-site
    R9 rc=2 out=0 err=error: @0: derivation steps after the placeholder are not representable in md1; the use-site path must be a multipath `/<a;b>/*` (or bare `/*`) as the final step — a fixed single step like `/0/*` is un-representable (found residue near `/0/*)`)
    R10 rc=0 out=10 err=
    G1 rc=0 out=491 err=# === Wallet bundle: descriptor, signet ===
    G2 rc=0 out=493 err=# === Wallet bundle: descriptor, signet ===
    G3 rc=2 out=0 err=error: @0 multipath alternative `0'` is hardened; hardened derivation is impossible on a watch-only (xpub) card, so such a descriptor is un-restorable
    G4 rc=2 out=0 err=error: @0 multipath alt `x` is not a bare unsigned integer

Expected after the fix (R10's stdout at `68ea05e0` is `canonical`):

    R1 rc=2 out=0 err=error: @0 multipath `<0;0>` repeats the value 0; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    R2 rc=2 out=0 err=error: @0 multipath `<1;1>` repeats the value 1; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    R3 rc=2 out=0 err=error: @0 multipath `<0;1;0>` repeats the value 0; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    R4 rc=2 out=0 err=error: @0 multipath `<1;01>` repeats the value 1; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    R5 rc=2 out=0 err=error: @1 multipath `<2;2>` repeats the value 2; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    R6 rc=2 out=0 err=error: import-wallet: descriptor: parse error: @0 multipath `<0;0>` repeats the value 0; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    R7 rc=2 out=0 err=error: @0 multipath `<0;0>` repeats the value 0; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    R8 rc=2 out=0 err=error: @0 multipath `<0;0>` repeats the value 0; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    R9 rc=2 out=0 err=error: @0 multipath `<0;0>` repeats the value 0; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    R10 rc=2 out=0 err=error: @0 multipath `<0;0>` repeats the value 0; BIP-389 forbids duplicate values in a multipath tuple, so this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
    G1-G4 unchanged.

R3, R4, R8 and R9 are covered in the test suite by the unit rows U2, U4, U6, U7; R10 is probe-only (the
classify path calls the same `parse_descriptor`).

## 6. Tests that change

None. A grep of `crates/`, `fuzz/` and `docs/` at `68ea05e0` for `<0;0>`, `<1;1>`, `<2;2>` finds no use; the
property tests build `/<0;1>/*` only (`prop_backup_restore_roundtrip.rs` L462, L563-L583;
`prop_template_completion_roundtrip.rs` L127); the bitcoin-core receive/change merge
(`wallet_import/bitcoin_core.rs` L503) merges only differing steps; the fuzz target
`fuzz_targets/descriptor_parse.rs` discards the result. Existing refusal tests keep their texts (U8, U9, and
`cli_verify_bundle_exit_tier.rs` `t1_lex_reject_exits_2`).

## 7. Out of scope (proposed FOLLOWUPS, section 8)

- md-codec encoder (the decoder keeps reading already-engraved cards), and md-cli `md encode` (companion line).
- `export-wallet --descriptor` (`parse_descriptor_lenient`, miniscript parse): measured at `68ea05e0`,
  `export-wallet --network signet --format descriptor --descriptor "wpkh([73c5da0a/84h/1h/0h]T84/<0;0>/*)"`
  exits 0 and prints the one descriptor `wpkh([73c5da0a/84'/1'/0']T84/0/*)#2ag6nxcd`. `xpub-search
  account-of-descriptor` and `compare-cost --descriptor` were not probed.
  Review r1 measured `--format bitcoin-core` on the same input: one `/0/*` entry with `"internal": false` and no
  change descriptor, at exit 0. The FOLLOWUP is therefore graded I (not M) and routed as its own unit.
- The other R02 sub-notes (grade row L58: `unwrap_or(master)`, wc-codec regroup trailing zeros, raid padding
  debug-only check, version field, u=0) need their own triage.

## 8. Gate

`cargo nextest run --locked -p mnemonic-toolkit` (CI `rust.yml`) with the 16 rows above matched by the filter in
section 5 (16 expected, 0 skipped among them); `make -C docs/manual lint` (CI `manual.yml`; no new flag, anchor
or link target); CI `changelog-check.yml`; and `design/fix-sec-multipath-duplicate-refusal-red.sh
<built binary>` printing the "after" block of section 5.

## 9. WORDING_REVIEW row proposals (status: PROPOSED; wording decided by the FIX author)

- **MPD-W1** (toolkit `lex_placeholders` Rule D; every surface in §3's table; source this FIX §3): >
  @<i> multipath `<<body>>` repeats the value <v>; BIP-389 forbids duplicate values in a multipath tuple, so
  this descriptor is invalid; copy the tuple from your wallet's own descriptor (most wallets use `<0;1>`: receive 0, change 1)
- **MPD-W2** (toolkit manual 41-mnemonic.md "Non-representable use-site steps" paragraph; source this FIX §4
  item 1): > A multipath that repeats a value … Remedy: copy the tuple from the wallet's own exported
  descriptor; most wallets use `/<0;1>/*` (receive 0, change 1). A guessed tuple describes a different wallet.
