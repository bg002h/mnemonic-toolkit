# GUI-form-renders cycle — Leg-2 P5 R0 review (round 1)

**Phase:** Leg-2 P5 — generate + embed + GATE the 61 GUI form renders + replace tour mockups
**Branch:** `feat/manual-gui-form-renders` @ `2f0cefce` (P5 commits `b32344c5` renders, `b3ccc09e` gate+CI, `1f3465c7` embed+tour+cspell, `2f0cefce` plan-note; on P4 `4abf56ad`)
**Reviewer:** opus architect (adversarial, gate-bite re-verified against live gates + source)
**Date:** 2026-06-29

---

## Verdict: **GREEN — 0 Critical / 0 Important**

All four deliverables verified against ground truth, not the draft:
- the `verify-examples-gui` gate is **real and fail-closed on all three failure classes** (content drift, census mismatch, secret-unmask — each re-proven by perturbation),
- the 61 renders are complete, leak-free, and byte-identical to the pinned `gui-render`,
- the embeds resolve (no empty fences) and **all anchor/outline coverage still passes** (982 schema anchors / 61 subs; 129 outlines),
- the tour drift-fix is accurate to the real v0.53.0 GUI,
- the P2-R0-Minor-1 `(required)` caveat is on **exactly** the genuine at-least-one / XOR groups,
- the cspell change is a hygiene *improvement*, not a regression.

Minor/Nit items below are non-blocking and do not gate.

---

## Critical

None.

## Important

None.

## Minor / Nit

1. **`.cspell.json` added 11 words, not the 10 the brief enumerates.** The diff also adds `manpath` (gen-man / man-page path env) beyond the brief's list (andor, Authenticode, authorised, CSPRNG, GLES, keepalive, notarised, notarytool, satoshis, containerised). All 11 are legitimate domain/locale terms; none is a key/address fragment. Informational only — the work is correct; the brief undercounts.

2. **Local Makefile `GUI_RENDER_BIN` default builds from the sibling checkout `$(MANUAL_GUI_UPSTREAM_ROOT)` (= `../mnemonic-gui`), a *floating* working-tree ref, not a tag-pinned install** (`Makefile:279`). This is acceptable and **in-pattern**: the lint's `gui-schema-coverage` already reads the *same* sibling checkout, so locally both the schema gate and the render gate share one source and stay consistent. The **authoritative** pin is CI job 1c, which `cargo install --git … --tag "$PINNED_TAG" --no-default-features --bin gui-render` where `$PINNED_TAG` is parsed from `pinned-upstream.toml` (single version-site) — correctly tag-pinned (`manual-gui.yml:234-245`). Consider a one-line Makefile comment reminding local devs the sibling checkout must sit at the pinned tag (already implied by the lint convention). Non-blocking.

3. **Secret-hygiene allowlist tolerates `[disabled]` for secret `*-stdin` checkboxes** (`verify-examples-gui.sh:95-96`). Correct today (every secret stdin checkbox renders `[ ] off [disabled]`, carrying no value). If a *future* GUI default-enabled a secret stdin checkbox, the row would render `[x] on` and the gate would flag it — a false-positive, but in the **fail-closed** direction (safe). Worth a one-line note for a future maintainer; not a defect now.

4. **Faithfully-rendered upstream asymmetry, out of P5 scope:** `ms encode` renders `--phrase` / `--hex` as `(required)` (conditional `ms_encode` marks both Required when neither set, `conditional.rs:742-745`), while `ms split` renders them as `(secret)`-only / not-required (no `ms_split` conditional fn; phrase/hex not statically required). The render is byte-faithful to the pinned binary (the gate proves it), and the manual correctly adds the caveat only where multiple `(required)` markers are *shown* (ms encode), not ms split. Any view that this asymmetry is itself wrong is a GUI/upstream concern, not a manual-render concern. Informational.

---

## Gate-bites re-verification (the core deliverable)

Ran the real gate from the repo with `env RUSTUP_TOOLCHAIN=stable make verify-examples-gui`. It regenerated with the **pinned, headless, `--no-default-features` gui-render** from the sibling checkout pinned at `mnemonic-gui-v0.53.0` (`git -C ../mnemonic-gui describe` = `mnemonic-gui-v0.53.0`), and diffed byte-for-byte == committed:

```
regenerating … cargo run --manifest-path …/mnemonic-gui/Cargo.toml --no-default-features --bin gui-render -- --emit-all <tmp>
wrote 61 form renders …
[verify-examples-gui] OK (61/61 renders match the pinned gui-render; no secret leak)
```

Then proved it **bites** (each restored clean afterward; final `git status` shows no modified tracked files):

| Negative test | Perturbation | Result |
|---|---|---|
| **Content drift** | `mnemonic-bundle.gui`: `-> mainnet` → `-> testnet` | `FAIL: regenerated renders drifted …` + unified diff; `make … Error 1` |
| **Census** | removed `ms-verify.gui` | `FAIL: census — committed … has 60 … expected 61` **and** `Only in <regen>: ms-verify.gui`; `Error 1` |
| **Secret-unmask** | `ms-repair.gui`: `<masked>` → `abandonabandon…about` | `FAIL: secret-marked render row without <masked>/[disabled] (possible cleartext secret leak): …ms-repair.gui:2` — **fired independently** of the (also-firing) content diff; `Error 1` |

All three classes are genuinely fail-closed. The secret-scan regex `\(([a-z]+, )*secret([, ][a-z]+)*\)` matches every secret-row variant present — `(secret)`, `(required, secret)`, `(secret, repeating)`, `(required, secret, repeating)` — and the `grep -vE '<masked>|\[disabled\]'` allowlist exactly covers the only two legitimate forms in the corpus. CI job 1c (`manual-gui.yml:207-252`) reads the same pin from `pinned-upstream.toml` and runs the same gate against the tag-pinned installed binary; tag-parse simulated → `mnemonic-gui-v0.53.0`. Fail-closed if the tag can't be parsed (`exit 1`).

## Secret-hygiene ruling

**PASS.** Swept all 61 renders: every `(secret)`-marked flag/positional row shows `<masked>`, and every secret `*-stdin` checkbox shows `[ ] off [disabled]`. No cleartext seed phrase, entropy hex, xprv/tprv, WIF, passphrase, or address appears as a rendered **value** (`-> …`) in any file (broad scans for `xprv|tprv|ms1q|…` as values, and for any long bech/base58-shaped token after `->`, returned nothing). Confirmed the `xprv` / `ms1` / `phrase` / `entropy` tokens that *do* appear are **option lists** inside `composite[…]` / `dropdown[…]` / `tagged-or-indexed[…]` — i.e. type/mode NAMES, not secrets — exactly as the brief notes. The renderer masks by construction; the committed bytes confirm it; the gate re-asserts it on every run. Built-output cross-check: the embedded renders carry 46 `<masked>` rows into `build/m-format-gui-manual.md` and zero leaked values. Layering is sound: `transcripts/**` is in cspell `ignorePaths` (renders never reach cspell), renders are fenced (would be cspell-ignored regardless), and `verify-examples-gui`'s secret-scan is the dedicated defense for render bytes — no gap.

## cspell-fix soundness

**SOUND — a net hygiene improvement, no regression.**
- Inline-code ignore tightened `` `[^`]+` `` → `` `[^`\n]+` `` (`.cspell.json:15`). The old form was greedy and `[^`]` spans newlines, so a stray single backtick could swallow many lines (masking real misspellings *and* prose). Bounding to one line makes cspell check **more**, not less — it cannot now *miss* a real misspelling it previously caught.
- Added `"/```[\\s\\S]*?```/g"` to ignore fenced blocks (`.cspell.json:11`). Required because the tightened inline rule no longer accidentally swallows fences; lazy `*?` pairs each opening fence to the next closing one; an unbalanced fence fails to match → falls back to *more* checking (fail-safe).
- The 11 added words are all legitimate domain/locale terms (verified); none is a base58/bech32 key fragment, so nothing in the dictionary can hide a leaked secret in prose.
- Live proof: `make lint` phase 2 cspell = **87 files checked, 0 issues** — the regex parses, the words resolve, and no real misspelling was newly exposed-and-unfixed.

## Anchor-intactness check

**PASS — the include= swaps dropped no required anchor and broke no link.** `make md` + `make html` build green (exit 0); every `include="gui/…"` fence was resolved (built markdown contains **0** occurrences of the placeholder body text — all 61 placeholders were replaced; `include-transcript.lua` is fail-closed, so an unresolved include would have aborted the build). `make lint` (7/7) green against the fresh HTML:
- markdownlint: 0 errors
- cspell: 0 issues / 87 files
- lychee: 1868 OK, **0 errors**
- **gui-schema-coverage: 982 schema anchors (61 subcommands) all present in HTML**
- **outline-coverage: 129 outlines all present with correct bullet count**
- glossary-coverage: OK
The 3 caveat chapters (`4h-inspect`, `4i-repair`, `60-ms/63-encode`) added prose + an include above the `## Outline` heading without disturbing the outline bullet sets.

## Caveat-placement check (P2-R0-Minor-1)

Enumerated every render carrying ≥1 `(required)` marker and classified each multi-required form against `mnemonic-gui/src/form/conditional.rs`:
- **inspect / repair** → `three_way_card_at_least_one` (`conditional.rs:926-960`): 0-set ⇒ all of `--ms1`/`--mk1`/`--md1` Required, ≥1-set ⇒ none. Genuine at-least-one. **Caveat present** ("At-least-one input (not a conjunction)").
- **ms encode** → `ms_encode` (`conditional.rs:730-746`): neither set ⇒ both Required, mutually exclusive. Genuine XOR. **Caveat present** ("Exactly-one input (not a conjunction)" — correctly distinguishes XOR from at-least-one).
- **verify-bundle** mk1+md1 both `(required)` → NOT a fill-one group: upstream `--mk1`/`--md1` are each `required_unless_present = "bundle_json"` (`conditional.rs:376-428`), i.e. **conjunctively** required (you need both, unless you supply `--bundle-json`). Showing both required is literal/accurate; **no caveat correctly added**.
- All other multi-required renders (md-compile, md-verify, mk-encode, addresses, convert, derive-child, ms-shares-split, seed-xor-*, slip39-split, verify-message, ms-split) are genuinely conjunctive; `(required)` is literal; **no caveat correctly added**.

Caveat lands on **exactly** {inspect, repair, ms encode} (`grep 'not a conjunction'` → 3 files). Placement is correct and would not mislead.

## Tour drift-fix check

`30-tour/31-first-launch.md`: the two hand-drawn mockups (which had drifted to `--template bip84`, a stale multi-row slot editor, and `--multisig-path-family bip87`) are replaced by `include="gui/mnemonic-bundle.gui"` and `include="gui/mk-inspect.gui"`. New prose matches the real v0.53.0 render exactly: `--template bip44` (single-sig), `--multisig-path-family` default **bip48** greyed, `--threshold` greyed, `--account` unset, **slot editor: 0 rows**, and the seed supplied through the slot editor (not a `--ms1` flag). Out-of-render chrome (tab strip, output panel, action bar, `Preview:` line) is described in prose and explicitly fenced out of the generated render — correct narrowing; the remaining tour ASCII (output panels / modal / help-icon) is correctly left out of scope.

## Scope / hygiene

- Diff is P5/manual-gui-scoped: no path outside `docs/manual-gui/**` + `.github/workflows/manual-gui.yml`.
- `.out`/`.cmd` worked-example gate **unaffected** — P5 added only `.gui` files; the shared `tests/verify-examples.sh` is an untouched symlink → `../../manual/tests/verify-examples.sh`.
- Overviews / CHANGELOG / release-history / glossary are **P4** (`4abf56ad`), not touched by any P5 commit — correctly out of this phase's review.
- `build/` artifacts are gitignored; working tree has no modified/staged tracked files after my perturbation tests (all restored).
- `git add -A` not used; staging is path-explicit per repo convention.

---

**Recommendation:** Proceed past the P5 gate. GREEN, 0C/0I.
