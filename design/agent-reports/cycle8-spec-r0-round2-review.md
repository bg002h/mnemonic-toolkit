# Cycle-8 ms-cli spec — R0 review, ROUND 2

- **Artifact under review:** `design/BRAINSTORM_cycle8_mscli_panics.md` (cycle-8: H4 · H5 · L26 · L5), AFTER folding round-1's I-1 + I-2.
- **Repo of record:** mnemonic-secret, `origin/master` = **`44ac71f`** (`44ac71fd6cb055e25e3c3dd1daca230d5d45bafb`) — ms-codec 0.5.0 / ms-cli 0.8.1
- **Reviewer:** opus software architect (adversarial R0 gate; NO code until 0C/0I)
- **Date:** 2026-06-21
- **Round 1 verdict:** 0C / 2I → RED (I-1 label threading; I-2 verify exit-3 leg).
- **Verification basis:** every fold re-checked against `origin/master` BYTES via `git -C /scratch/code/shibboleth/mnemonic-secret show origin/master:<path>` — NOT the spec's self-assessment. Local tree is stale (2-behind, dirty, structurally older). Live files read this round: `cmd/decode.rs`, `cmd/derive.rs`, `cmd/verify.rs`, `cmd/combine.rs`, `language.rs`, `error.rs`; toolkit `src/language.rs::non_english_seed_advisory`.

---

## Fold verification (against `44ac71f` bytes)

### I-1 — label threading → **RESOLVED** (derive + verify `:95`), but the fold SURFACED a new gap (see I-new)

The helper is now the **3-tuple** `(Zeroizing<Vec<u8>>, CliLanguage, bool)` = `(entropy, effective_lang, effective_lang_defaulted)` EVERYWHERE in the spec:
- Signature §3.1 line 77 → 3-tuple. Doc-comment §3.1 lines 58-71 describes all three returns + the `effective_lang_defaulted` rationale.
- Derive consumer §3.2 lines 100-111: the label table (lines 104-109) maps ALL FOUR derive label sites to the helper outputs — `:231` JSON `language`→`effective_lang.as_str()`, `:232` `language_defaulted`→`effective_lang_defaulted`, `:245`/`:246-249` DEFAULT line + english-default note→`effective_lang_defaulted`/`effective_lang`, `:251` text→`effective_lang.as_str()`. The recipe no longer says "reuse `(cli_lang, defaulted)`" for labels; line 100 explicitly states `(cli_lang, defaulted)` is the **INPUT** to the helper, "NOT what the downstream sites read."
- Verify `:95` §3.3 line 127: `emit_round_trip_ok(…, effective_lang.as_str(), …)`, NOT `args.language.as_str()`.
- Resolved-decisions D2 + D2a (lines 316-317) state the 3-tuple + every-label-site rule consistently.
- Label-pin TEST specified: §7.1 line 261 — `ms derive <french-ms1>` (no `--language`) → fp `7d53dc37`, stdout `language: french` (NOT `english (DEFAULT)`), stderr NO bogus default note; JSON `language == "french"` ∧ `language_defaulted == false`. The Entr-vs-Mnem split is pinned by the line-262 positive control + the english-`Mnem` contrast (Entr → `english (DEFAULT)` + default note; english-`Mnem` → `english` WITHOUT `(DEFAULT)`, no default note). Verify label pinned §7.2 line 270 (wire `japanese`, not `english`).

**Live-source cross-check of every cited line number (all ACCURATE at `44ac71f`):** derive `:165` `(cli_lang, defaulted) = match args.language`, `:185` unreachable, `:231` JSON `language: cli_lang.as_str()`, `:232` `language_defaulted: defaulted`, `:245` `(DEFAULT)`, `:248` default-note (writeln! spans 246-249 ✓), `:251` non-default text. The decode template `decode.rs:63` returns `(effective_lang, effective_lang_defaulted)` from `match &payload`, `Mnem` arm → `(wire_cli_lang, false)` — the helper's `Mnem→false` mirrors it byte-for-byte; the note string `"note: this ms1 carries wordlist language '{}'; ignoring --language {}"` matches decode's verbatim. **Entr/Mnem `defaulted` coherence confirmed:** Entr passes `cli_lang_defaulted` through (line 82); Mnem → `false` (line 85) — both match decode.

### I-2 — verify exit-3 leg → **RESOLVED**

§3.3 lines 117-122 + D5a (line 321) state: the helper is invoked **ONLY on the `Ok((tag, payload))` arm**; "DO NOT replace the whole match with the helper." Both `Err` arms preserved verbatim: `Err(ms_codec::Error::ReservedTagNotEmittedInV01 { got }) => { emit_future_format(&got, args.json)?; return Ok(0); }` (exit-3 leg) + `Err(e) => return Err(e.into())`.

**Live structure confirmed** (`verify.rs:59-72`): the match is over `Result<(Tag, Payload), ms_codec::Error>` with FOUR arms — `Ok((_tag, Payload::Entr(b)))` (`:63`), `Ok((_, _)) => unreachable!` (`:64`, the H5 panic), `Err(ReservedTagNotEmittedInV01 { got })` (`:65`), `Err(e)` (`:72`). The spec's quoted arms are byte-faithful. **Exit-3 mechanism verified end-to-end:** `emit_future_format` (`:126`) returns `Err(CliError::FutureFormat { tag })` (`:142`), whose `exit_code()` → **3** (`error.rs:51`). So the `return Ok(0)` after the `?` is genuinely unreachable and exit-3 is preserved. No-regression TEST specified: §7.2 line 271 (reserved-tag string still exits 3).

### M-1 (parity), M-3 (`--to ms1` preservation) → **RESOLVED**

- **M-1:** §3.4 lines 135-138 + D3 now distinguish the two parities: disagreement-`note:` mirrors `decode` (has `--language`); bare wire-resolution shared by `decode` AND `combine` — but `combine` has NO `--language` arg → NO note. **Live-confirmed:** `CombineArgs` has no language field; `combine.rs:95-107` resolves wire language, emits no note. Accurate.
- **M-3:** §7.3 line 280 adds the `--to ms1` language-byte-preservation test (combine Japanese shares `--to ms1` → `ms decode` that ms1 → assert `Mnem` carrying `language: japanese`). **Live-confirmed:** `emit_ms1(&payload, …)` calls `ms_codec::encode(Tag::ENTR, payload)` with the ORIGINAL `&Payload` (still `Payload::Mnem{language,…}`), so the language byte survives. Claim holds; the test pins it against a future `Payload::Entr` reconstruction regression.

### No-new-drift sweep

- **Helper signature consistency:** ✅ 3-tuple everywhere (lines 55, 59, 77, 100, 122, 124, 316). The only "2-tuple" strings (lines 60, 80, 316) are explicit, correct descriptions of decode.rs's *language-part* template (`(effective_lang, effective_lang_defaulted)`), each framed as "for the language part" and contrasted against the helper's 3-tuple. No leftover 2-tuple signature reference.
- **Resolved-decisions table:** internally consistent (D2/D2a/D3/D5a all reflect the folds; no contradiction with §3).
- **No new open question** introduced.
- **Funds core unchanged:** wire-language authoritative; `--language` advisory; `unreachable!` retained only as the `#[non_exhaustive]` guard (`from_code` covers 0..=9, codec rejects ≥10 → `_` unreachable for valid input). Oracle `assert_ne!(73c5da0a EN, 7d53dc37 FR)` intact. ✅

---

## Critical

**None.** Funds-correctness is unchanged and sound: wire byte authoritative, oracle proves the naive `--language`-default patch is funds-wrong, fp/entropy are correct on every path regardless of the advisory note. The I-new gap below does NOT touch any funds path.

---

## Important

### I-new — `verify`'s `--language` is a non-`Option` `CliLanguage` (`default_value="english"`); the spec's helper requires a `cli_lang_defaulted: bool` that `verify` CANNOT correctly compute, and the decode-parity claim is unmet as written

This is a **NEW gap surfaced by the I-1 fold** (the fold widened the helper to take `cli_lang_defaulted: bool` — round 1 did not catch this because round 1's helper was a 2-tuple without that flag).

**Live structural fact (verified this round):**
- `decode.rs:29` and `derive.rs:62`: `#[arg(long)] pub language: Option<CliLanguage>` — NO `default_value`. This is *precisely* what lets them compute `(cli_lang, defaulted)`: `Some`→user-set→`false`, `None`→defaulted→`true` (decode `:44`, derive `:165`).
- **`verify.rs:30-31`: `#[arg(long, default_value = "english")] pub language: CliLanguage`** — a NON-`Option` field. clap collapses "explicit `--language english`" and "omitted (defaulted)" into the SAME `CliLanguage::English` value. **Verify cannot distinguish the two from `args.language` alone.**

**Why this breaks the spec as written:**
The §3.1 helper gates its disagreement note on `if !cli_lang_defaulted && wire_cli != cli_lang`. The §3.4/D3 design + §7.2 line 269 test (`ms verify --language english <ja-ms1>` → expects stderr `note:`) require verify to pass `cli_lang_defaulted = false` for an explicit flag. But the spec NEVER states what `cli_lang_defaulted` value `verify` passes, and verify's arg type makes a correct value impossible without a code change:

- If verify hardcodes `cli_lang_defaulted = false` (the only value consistent with the line-269 test): then a **bare `ms verify <ja-ms1>` (no `--language` at all)** evaluates `!false && japanese != english` → `true` → emits a **spurious disagreement note** the user never triggered. This is the exact opposite of the decode/derive contract ("note only on EXPLICIT disagreement"), and directly contradicts the §3.4/§9-D3 "parity with `decode`" thesis ("verify joins `decode` in this note-emitting parity").
- If verify hardcodes `cli_lang_defaulted = true`: then the line-269 test (`--language english` explicit) would NOT fire the note → that test goes RED.

There is **no value of a hardcoded constant that satisfies both** the line-269 explicit-disagreement test AND a (currently-missing) bare-no-flag-no-spurious-note expectation. The clean resolution is to change `VerifyArgs.language` to `Option<CliLanguage>` (matching decode/derive) and compute `(cli_lang, cli_lang_defaulted)` identically — but the spec does not specify this, and **it has a CLI-surface consequence the spec's §6 explicitly denies**: removing `default_value = "english"` drops the `[default: english]` annotation from `ms verify --language`'s clap `--help`. §6 asserts "No CLI-surface break" and "Manual: N/A (mirrors clap `--help`; unchanged)" — a `--help` default-annotation change is a manual-mirror lockstep touch (`docs/manual/src/40-cli-reference/43-ms.md`), even though it is not a flag add/remove/rename.

**Severity = Important (not Critical):** funds-correctness is untouched (wire wins on every path; the note is advisory-only; entropy/fp are correct regardless). But (a) the spec is **decision-INCOMPLETE** on a behavior it tests (line 269 has no defined `cli_lang_defaulted` source), (b) the most natural fix collides with the §6 "no CLI-surface break / manual N/A" claim, and (c) the stated decode-parity is unachievable with verify's current arg type. The author intent is "decision-complete, no open questions" (§0 line 6) — this is an open decision.

**Required fix (spec), pick ONE and pin it:**
1. **Option A (recommended — full decode/derive parity):** change `VerifyArgs.language` to `#[arg(long)] pub language: Option<CliLanguage>`; compute `(cli_lang, cli_lang_defaulted)` via the decode/derive idiom (`Some`→`false`, `None`→`true`) and pass it to the helper. Then: (i) update §6 — this IS a `--help` surface change (the `[default: english]` annotation disappears); confirm/declare the manual-mirror lockstep (`43-ms.md`) and re-check the GUI schema-mirror is still N/A (flag-NAME unchanged → likely N/A, but state it explicitly rather than asserting "no surface change"); (ii) add a §7.2 test that **bare `ms verify <ja-ms1>` (no `--language`) emits NO disagreement note** (the no-spurious-note pin), complementing the line-269 explicit-`--language` note test.
2. **Option B (keep arg type, accept asymmetry):** keep `CliLanguage`+`default_value`, pass `cli_lang_defaulted = false` always, and EXPLICITLY document that verify (unlike decode/derive) emits the disagreement note whenever the wire ≠ english because it cannot detect defaulting — i.e. **drop the "verify joins decode in note parity" claim** and state the divergence as intentional. This avoids the surface change but means a bare non-english verify always prints the note; pin that behavior with a test. (Weaker: it abandons the parity thesis the spec leans on.)

Either way the spec must (1) state the chosen `cli_lang_defaulted` source for verify, (2) reconcile §6's "no CLI-surface break" with the choice, and (3) add the missing bare-no-flag verify-note test.

---

## Minor

### M-5 — §3.3 lists the round-trip line range as `:84-...` but the relevant lines are `:85-95`; trivial citation slack
§3.3 line 127 cites the round-trip block as "`:84-...`". The `if let Some(supplied)` block runs `:85-101`; the load-bearing lines are `:86` (`args.language.into()`), `:87` `parse_in`, `:88` `from_entropy_in`, `:95` `emit_round_trip_ok`. Not wrong, just loose — tighten to `:85-95` for the implementer. No design impact.

### M-2 / M-4 (carried from round 1) — still appropriately scoped as Minor/optional
- M-2 (Chinese `as_str()` word-order vs toolkit `human_name()`): §4.2 line 169 + D8 footnote it correctly ("except Chinese; cosmetic; not gated"). Live-confirmed: ms-cli `as_str()` = `chinese-simplified`/`chinese-traditional`; toolkit `human_name()` = `simplified-chinese`/`traditional-chinese`. §7.3 tests use Japanese (unaffected). Resolved as documented.
- M-4 (L5 Debug test hardening for a non-`InvalidChecksum` codex32 arm): §7.4 line 290 carries it as OPTIONAL. Fine.

### M-6 — L26 advisory `form` string: spec passes `"raw entropy"`, toolkit oracle uses `"raw entropy"` too — consistent, but confirm the ms-cli test asserts the substring, not the whole line
§4.3 line 180 passes `"raw entropy"` as `form`; §7.3 line 278 asserts the stderr contains "warning: encoding a japanese BIP-39 seed as raw entropy". Toolkit `non_english_seed_advisory` body (live) emits exactly `"…seed as {form} — it carries only the entropy…"`. Byte-aligned. Keep the §7.3 assertion as a `contains` on the distinctive substring (it is). No change needed; noting for the implementer.

---

## Cross-charge verdicts (explicit)

- **I-1 RESOLVED** (derive: 3-tuple + all 4 label sites threaded + label-pin test + Entr/Mnem contrast control; verify `:95` threaded + pinned). Helper signature is the 3-tuple consistently everywhere; no leftover 2-tuple signature reference. BUT the fold surfaced **I-new** (verify's non-`Option` arg can't source `cli_lang_defaulted`).
- **I-2 RESOLVED** (helper on `Ok` arm only; both `Err` arms verbatim; exit-3 mechanism verified `FutureFormat`→exit_code 3; no-regression test specified). Live verify structure (4-arm `Result` match) confirms there genuinely are two `Err` arms + the exit-3 path.
- **M-1 RESOLVED** (decode-only note parity; combine = bare resolution, no `--language`, no note — live-confirmed).
- **M-3 RESOLVED** (`--to ms1` passes original `&payload` → `Mnem` language byte preserved; test pins it).
- **No new drift** in the helper signature, resolved-decisions table, open-questions, or funds core — EXCEPT the I-new decision gap, which is a *consequence* of widening the helper to carry `cli_lang_defaulted` and lands on verify specifically.

---

## TDD integrity

RED-first design remains genuine: H4/H5 build real non-English mnem ms1 cards → panic today (`unreachable!` live at `derive.rs:185`/`verify.rs:64`) → cannot pass on `origin/master`. The I-1 label-pin tests (§7.1 line 261, §7.2 line 270) and I-2 exit-3 no-regression (§7.2 line 271) close round 1's two test gaps. M-3's `--to ms1` decode-back test is non-vacuous. **Remaining gap (I-new):** no test pins bare `ms verify <ja-ms1>` (no `--language`) for the absence of a spurious disagreement note — so whichever verify-side `cli_lang_defaulted` decision is made, its bare-no-flag behavior would ship unpinned. Add it with the I-new fix.

---

## VERDICT

**R0 ROUND 2: 0C / 1I → RED**

- C: none.
- I-new: `verify.rs`'s `--language` is a non-`Option` `CliLanguage` (`default_value="english"`) — verify cannot compute the `cli_lang_defaulted: bool` the §3.1 helper now requires; the §7.2 line-269 explicit-disagreement test forces `false`, which makes a bare no-flag non-english verify emit a SPURIOUS note (contradicting the decode-parity claim and §6's "no CLI-surface break"); the natural fix (Option-ize the arg, matching decode/derive) is a `--help` surface change the spec denies. Decision-incomplete on a tested behavior. Pick Option A or B, state the `cli_lang_defaulted` source, reconcile §6, add the missing bare-no-flag verify-note test.

I-1, I-2, M-1, M-3 from round 1 are all RESOLVED and verified against `44ac71f` bytes; the helper is a consistent 3-tuple everywhere. Fold I-new, persist this review, re-dispatch round 3. **NO implementation until GREEN (0C/0I).**
