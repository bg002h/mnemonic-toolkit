# End-of-cycle R0 review — v0.37.11 non-English BIP-39 seed advisory

Reviewer: feature-dev:code-reviewer (opus). Reviewed the full cycle diff `master..HEAD`
(branch `non-english-seed-advisory`, base `9f11a31`): helper (`src/language.rs`), four
call sites (`bundle.rs:718`, `convert.rs:952`, `slip39.rs:443` split + `:656` combine),
three integration test files, version/CHANGELOG/README hygiene, and the SPEC/plan against
live source at the branch tip.

## Critical

**C1 — `convert --to ms1` is an uncovered language-losing ms1 emit (the exact footgun, missed).**
`crates/mnemonic-toolkit/src/cmd/convert.rs:1192` — `convert` supports a live, permitted
`--to ms1` edge from `phrase`/`seedqr`/`entropy` inputs (`is_supported_direct_edge`,
`convert.rs:602/615/622`; not refused by `classify_edge`). It produces a real ms1 card via
`ms_codec::encode(Tag::ENTR, Payload::Entr(entropy))` — a card that carries only the
entropy, NOT the wordlist language. This is the *same* footgun the cycle exists to mitigate
(more so than raw entropy: an ms1 card is the canonical steel-engravable backup form). But
the convert advisory at `convert.rs:952` gates only on `targets.contains(&NodeType::Entropy)`,
so:

```
convert --from phrase=<french-12> --language french --to ms1
```

emits a French-entropy ms1 card with **no advisory**. The SPEC §2
(`SPEC_non_english_seed_advisory.md:22`) asserts "the only language-dropping LIVE target is
`Entropy`" and enumerates only the *derived-key* exclusions — it silently omits `Ms1` from
the analysis. That is a factual gap, not a documented scope decision. Five language-losing
ms1/entropy/shares emit-sites exist; the cycle covered four and the bundle path's ms1, but
missed convert's standalone ms1.

**Fix:** add an `Ms1`-target check alongside the existing `Entropy` check at `convert.rs:952`,
with form `"an ms1 card"`. (`mk1`/`--to mk1` needs no advisory — no `(_, Mk1)` edge is
reachable from a phrase; `(Xpub, Mk1)` is refused at `convert.rs:674`, and mk1 carries an
xpub with the language already baked in.) Add a `convert --to ms1` (French → fires) test and
a `--to ms1` no-fire English control. Note: this fix must move to the correct post-success
location too (see I1).

## Important

**I1 — convert advisory can fire *before* a primary-input parse error (advise-then-error),
violating the cycle's own I2 discipline.**
`crates/mnemonic-toolkit/src/cmd/convert.rs:952` — the advisory is emitted before
`compute_outputs` (`:964`), but the primary phrase is not parsed until *inside*
`compute_outputs` at `convert.rs:1147` (`Mnemonic::parse_in`). So
`convert --from phrase="<malformed french>" --language french --to entropy` prints the
advisory and then exits with a BIP-39 parse error. The plan's I1 fold (plan `:115`) placed it
"after all refusal guards, before `compute_outputs`" believing that point post-dated input
validation — but the actual mnemonic parse lives inside `compute_outputs`. The other three
sites correctly fire only *after* the fallible operation succeeds (bundle after synthesis;
slip39 split after `parse_master_to_entropy` `:437`; combine after `slip39_combine` `:650`).
This is an inconsistency with the cycle's stated "a bad phrase shouldn't advise-then-error"
rule.

**Fix:** move the convert advisory (both the existing `Entropy` emit and the new `Ms1` emit
from C1) to after `compute_outputs` succeeds — e.g. right after the `let (mut outputs, …) =
match computed { … }` block (after `convert.rs:~1018`, before the emit block at `:1052`).
`targets`, `args.language`, and `stderr` are all in scope there. This makes convert
consistent with the other three sites.

## Minor

**M1 — slip39 no-fire tests don't assert command success (non-vacuous, but fragile).**
`crates/mnemonic-toolkit/tests/cli_slip39_language_advisory.rs:47,66` —
`split_english_no_advisory` and `combine_french_to_phrase_no_entropy_advisory` use the raw
`.output()` helper and only assert the advisory is *absent*; they don't assert exit success.
They are not currently vacuous (the paired positive tests `:38`/`:56` prove the identical
invocation shapes reach the advisory chokepoint successfully, and only the language/`--to`
value is swapped), but if a future arg-shape regression made these commands error before the
chokepoint they'd pass silently. Suggest asserting `out.status.success()` in the helper or in
those two tests.

**M2 — convert `--to phrase` no-fire case is implied but not directly tested.**
The convert suite tests `--to xprv` (key target) no-fire but not `--to phrase` (the
language-keeping mnemonic target the SPEC §3.3 calls out by name). `--to xprv` adequately
proves a non-entropy target doesn't fire, so this is optional, but a one-line `--to phrase`
French no-fire assertion would lock the SPEC's stated rationale.

## Verdict basis
The helper, bundle site (watch-only gate via `any_secret_bearing()`, once-per-invocation,
`--json` stdout clean), slip39 split/combine sites (post-success placement, correct
`Slip39ToShape::Entropy` gate, direct `CliLanguage` type), version/Cargo.lock/README/CHANGELOG
hygiene (all `0.37.11`, consistent), and stderr-only / no-wire-change / PATCH justification
are all correct. But C1 leaves the cycle's core invariant — "warn at every language-losing
ms1/entropy/shares emit" — unmet for `convert --to ms1`, and I1 breaks the cycle's own
advise-after-success discipline at the convert site. Both must be folded before tag.

**VERDICT: RED (1C/1I)**

---

## Fold applied (controller, verified against source)

- **C1 + I1 (single fold):** removed the premature `convert.rs:952` (pre-`compute_outputs`)
  advisory block; inserted a combined emit *after* the `match computed { … }` block (post-
  `compute_outputs`-success) covering **both** `Entropy` and `Ms1` targets (forms `"raw
  entropy"` / `"an ms1 card"`). Confirmed at source: `convert.rs:1192` `Ms1 =>
  ms_codec::encode(... Payload::Entr ...)` reachable from a `Phrase` input via
  `Mnemonic::parse_in` (`:1147`) → `to_entropy()`; the parse lives inside `compute_outputs`,
  so post-success placement is the only advise-after-success point.
- **Tests:** `french_to_ms1_fires_advisory` (form `an ms1 card`),
  `english_to_ms1_no_advisory`, `malformed_french_phrase_errors_without_advisory`
  (bad-checksum French → exit failure, no advisory — I1 regression), and
  `entropy_to_french_phrase_no_advisory` (M2; `phrase→phrase` is a refused identity edge, so
  the language-keeping case is exercised `entropy→phrase`).
- **M1:** the shared `slip39()` test helper now asserts `out.status.success()` so every
  no-fire assertion is non-vacuous.
- **SPEC §2 amended** (C1 + I1 amendment notes) to record `Ms1` as a language-dropping target
  and the post-`compute_outputs` placement. **CHANGELOG** convert bullet widened to
  `entropy` **or** `ms1` + the malformed-phrase-orders-before-advisory note.
