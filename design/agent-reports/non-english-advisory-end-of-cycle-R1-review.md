# End-of-cycle R1 re-review — v0.37.11 non-English BIP-39 seed advisory

Reviewer: feature-dev:code-reviewer (opus). Re-review after the R0 C1+I1+M1+M2 fold
(`design/agent-reports/non-english-advisory-end-of-cycle-R0-review.md`). Read the full current
cycle diff `master..HEAD` against live source.

## Verification summary

**C1 fold (convert `--to ms1`):**
- The premature pre-`compute_outputs` advisory block is GONE. The new block is placed AFTER
  `let (mut outputs, input_variant, electrum_seed_version) = match computed { … }`. `targets`,
  `args.language`, `stderr` all in scope and valid.
- Gates on BOTH `NodeType::Entropy` (form "raw entropy") and `NodeType::Ms1` (form "an ms1
  card").
- `convert --to ms1` from a phrase reaches `ms_codec::encode(... Payload::Entr ...)` (the
  `Ms1 =>` arm). Exhaustively checked every other phrase-reachable target: `Phrase`
  (re-encodes, keeps language), `Xpub/Xprv/Fingerprint/Wif/Bip38/Address` (derived keys),
  `ElectrumPhrase` (only reachable from Entropy), `Mk1` (`unreachable!`/refused). Entropy and
  Ms1 are the ONLY two uncovered-loss targets and both are now gated. C1 complete.

**I1 fold (advise-after-success):**
- `Mnemonic::parse_in` lives inside `compute_outputs`; a malformed phrase → `Err` → returns
  before the advisory. Regression test `malformed_french_phrase_errors_without_advisory`
  asserts `.failure()` + no advisory. Correct.

**No fold-introduced drift:**
- `french_multi_target_with_entropy_fires_once` (`--to xprv,entropy`): the new `Ms1` block
  doesn't fire (ms1 absent), Entropy fires once → still exactly 1. Holds.
- `entropy_to_french_phrase_no_advisory` (M2): `(Entropy, Phrase)` is a supported edge;
  re-encodes French; target neither Entropy nor Ms1 → no fire. Comment correctly notes
  `phrase→phrase` is a refused identity edge → uses `entropy=` input. Sound.
- `slip39()` helper now asserts `out.status.success()`; all invocations are valid success
  cases. Non-vacuous.
- SPEC §2 amendment + CHANGELOG convert bullet say `entropy` **or** `ms1` — match the gate.
- Bundle site + slip39 sites unchanged and correct. Release hygiene intact (Cargo.toml +
  Cargo.lock + both READMEs `0.37.11`; CHANGELOG `[0.37.11]` accurate).

## Critical
None.

## Important
None.

## Minor
- `design/SPEC_non_english_seed_advisory.md:62,75` — §3.2 table row and §3.3 prose still said
  the convert gate is `Entropy`-only, not updated to `Entropy || Ms1`. §2 amendment + CHANGELOG
  already correct, so SPEC-internal staleness only, no code/test/artifact impact. Optional.

**VERDICT: GREEN (0C/0I)**

---

## Post-R1 fold (controller)

- Folded the optional R1 Minor: SPEC §3.2 convert table row + §3.3 prose now state the gate is
  `Entropy` **or `Ms1`** and note the post-`compute_outputs` placement, matching §2 / code /
  CHANGELOG. Documentation-only; no code/test/artifact change. The cycle remains 0C/0I.
