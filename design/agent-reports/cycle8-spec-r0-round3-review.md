# Cycle-8 ms-cli spec — R0 review, ROUND 3

- **Artifact under review:** `design/BRAINSTORM_cycle8_mscli_panics.md` (cycle-8: H4 · H5 · L26 · L5), AFTER folding round-2's **I-new** (Option A — Option-ize verify's `--language`).
- **Repo of record:** mnemonic-secret, `origin/master` = **`44ac71f`** (`44ac71fd6cb055e25e3c3dd1daca230d5d45bafb`) — ms-codec 0.5.0 / ms-cli 0.8.1.
- **Reviewer:** opus software architect (adversarial R0 gate; NO code until 0C/0I).
- **Date:** 2026-06-21.
- **Round 2 verdict:** 0C / 1I → RED (I-new: verify's non-`Option` `--language` cannot source `cli_lang_defaulted`).
- **Verification basis:** every fold + every cited line re-checked against `origin/master` BYTES via `git -C /scratch/code/shibboleth/mnemonic-secret show origin/master:<path>` — NOT the spec's self-assessment, NOT the stale local checkout (2-behind, dirty, structurally older flat `src/`). Toolkit manual verified against THIS repo's `docs/manual/src/40-cli-reference/43-ms.md` bytes. Live files read this round: `cmd/{verify,decode,derive,combine}.rs`, `language.rs`, `cmd/gui_schema.rs`, `tests/{verify_phrase_round_trip_ok,verify_phrase_round_trip_mismatch,verify_quiet_pass,verify_quiet_fail,verify_future_format,encode_pipe_to_verify,cli_help_pointer,cli_output_class,gui_schema_emits_spec_v7_json,decode_default_english_in_stdout}.rs`; toolkit `43-ms.md`.

---

## Citation verification (all against `44ac71f` bytes / toolkit-repo manual bytes)

| Spec citation | Live truth | Verdict |
|---|---|---|
| §3.3 / D5b: `verify.rs:30-31` is `#[arg(long, default_value = "english")] pub language: CliLanguage` (non-`Option`) | `verify.rs:30` `#[arg(long, default_value = "english")]`, `:31` `pub language: CliLanguage` — EXACT | **ACCURATE** |
| §3.3 / D5b: `decode.rs:29` / `derive.rs:62` are `Option<CliLanguage>` (no `default_value`) | `decode.rs:29` `pub language: Option<CliLanguage>` (no default); `derive.rs:62` `pub language: Option<CliLanguage>` (no default) | **ACCURATE** |
| §3.3 / D5b idiom `match args.language { Some(l)=>(l,false), None=>(English,true) }` | `decode.rs` `let (cli_lang, defaulted) = match args.language { Some(l) => (l, false), None => (CliLanguage::English, true) }` — byte-identical idiom | **ACCURATE** (verbatim decode idiom) |
| §3.3: verify's ONLY `args.language` consumers are `:86` (`.into()`) + `:95` (`.as_str()`) | grep `args.language` in verify.rs → exactly `:86 let lang: Language = args.language.into();` and `:95 emit_round_trip_ok(…, args.language.as_str(), …)`. No third site. | **ACCURATE — only two consumers; both replaced by the fold** |
| `From<CliLanguage> for bip39::Language` (NOT `From<Option<…>>`) | `language.rs:67 impl From<CliLanguage> for bip39::Language` | **ACCURATE** — confirms `:86` `.into()` would break under `Option` unless replaced (it is, by `effective_lang.into()`) |
| §6/D5b: manual `43-ms.md:297` documents verify `--language` default-free | `43-ms.md:297` = `| \`--language <LANGUAGE>\` | BIP-39 wordlist for \`--phrase\` |` — NO `[default: english]` | **ACCURATE — manual line is default-free** |
| §3.3 line 126 contrast: derive `:149` / decode `:102` manuals DO annotate "default english" | `43-ms.md:149` `(load-bearing; default \`english\`, annotated \`DEFAULT\`)`; `:102` `(default \`english\`; annotated \`DEFAULT\`…)` | **ACCURATE — contrast holds** |
| §6/D13: gui-schema gates flag-NAMES + value-enums, NOT help-text defaults | `gui_schema.rs::classify_flag` keys on `get_action()` + `get_possible_values()`; per-flag entry = `{name, kind, choices}` — **no `default` field emitted at all** | **ACCURATE — schema is default-agnostic** |
| §6/D13: verify `--language` re-type stays a dropdown (flag-name + value-enum unchanged) | `classify_flag` → `dropdown` for ANY `ValueEnum` arg regardless of `Option`/`default_value`; gui_schema test asserts NO `verify --language` shape (only `verify` present `:77` + `verify --json` boolean `:166`) | **ACCURATE — schema-mirror genuinely N/A** |
| decode disagreement note string (helper template, §3.1 line 86) | `decode.rs` `"note: this ms1 carries wordlist language '{}'; ignoring --language {}"` + `Mnem` arm returns `(wire_cli_lang, false)` | **ACCURATE — verbatim; Mnem→false mirrored** |
| derive label sites `:231/:232/:245/:246-249/:251` | JSON `language: cli_lang.as_str()` / `language_defaulted: defaulted`; text `language: … (DEFAULT)` + english-default note in `if defaulted`; non-default `language: …` — all live | **ACCURATE** |
| §7.2 bare-no-flag no-spurious-note test present | §7.2 "NO spurious note when flag OMITTED (I-new …)" line present; RED-on-current rationale stated | **PRESENT & GENUINE** |
| D5b row exists + consistent with §3.3/§6 | D5b (line 334) states Option A + idiom + surface-delta; matches §3.3 lines 117-126 + §6 row + D13 | **CONSISTENT** |

**Drift sweep — does Option-izing break any existing verify test/lint?**
- `verify_phrase_round_trip_ok.rs`: input `ms10entr…` (English **`entr`** card) + `--phrase` (English), NO `--language`. Asserts `"OK: round-trip valid (12 words, language=english)"`. Post-fold: helper Entr arm → `(cli_lang=English, defaulted=true)`; `:95` becomes `effective_lang.as_str()` = `"english"`. Label **unchanged** → test **PASSES**. ✅
- `verify_phrase_round_trip_mismatch.rs`, `verify_quiet_pass/fail`, `verify_future_format`, `encode_pipe_to_verify`: NONE pass `--language` (grep-confirmed). Unaffected. ✅
- `cli_help_pointer.rs`: asserts only top-level `ms --help` btcrecover footer; touches no verify `--language` help text. ✅
- `cli_output_class.rs:154-197`: all language-note assertions target `ms derive`, NOT `ms verify`. `decode_default_english_in_stdout.rs`: decode-only. NO verify default-help assertion anywhere. ✅
- `gui_schema_emits_spec_v7_json.rs`: asserts `verify` present + `verify --json` boolean; NO assertion on `verify --language` kind/choices/default. Re-type → still `dropdown` → no test moves. ✅

**Other `args.language` consumer check (drift):** verify has EXACTLY two — `:86` and `:95`. The §3.3 fold replaces BOTH: round-trip derived side `from_entropy_in(effective_lang.into(), …)`, supplied side `parse_in(effective_lang.into(), …)`, and `:95 emit_round_trip_ok(…, effective_lang.as_str(), …)`. After the fold, `args.language` is consumed ONLY by the top-of-`run` `match args.language { Some/None }` resolution — there is NO orphaned `.into()`/`.as_str()` on the bare `Option<CliLanguage>`. The re-type is therefore self-consistent and compiles. ✅

---

## Critical

**None.** Funds-correctness is untouched by the I-new fold. The fold is purely about (a) verify's arg type and (b) the advisory-note firing condition; on EVERY path the wire byte remains authoritative, entropy/fp are correct, and the disagreement note is advisory-only. The oracle `assert_ne!(73c5da0a EN, 7d53dc37 FR)` and the verify round-trip "phrase reproduces card?" true-negative semantics are unchanged. No funds path depends on `cli_lang_defaulted`.

---

## Important

**None.**

**I-new is RESOLVED.** Verified point-by-point against the charge:

1. **Explicit Option A choice + `cli_lang_defaulted` source + §6/manual reconcile — YES.** §3.3 lines 117-124 explicitly re-type `verify.rs:30-31` to `#[arg(long)] pub language: Option<CliLanguage>` and resolve `(cli_lang, defaulted)` via the byte-identical decode idiom `match args.language { Some(l)=>(l,false), None=>(English,true) }`. D5b (line 334) records it. §3.3 line 124 states the funds-neutral rationale (avoids the spurious-note-on-omission that breaks decode-parity). The §6 surface-delta box (line 126) + D13 (line 342) reconcile both lockstep gates.

2. **§6 surface-delta reconciliation FACTUALLY correct — YES, all three sub-claims verified against bytes:**
   - **(a)** Option-izing removes ONLY the `[default: english]` clap-`--help` annotation. Confirmed structurally: decode/derive are already `Option<CliLanguage>` and their `--help` renders without a `[default]` (no `default_value` attr); verify's `default_value="english"` is the sole source of its `[default: english]` annotation, so dropping it is the entire help delta. No other arg/flag/positional changes.
   - **(b)** "manual `43-ms.md:297` documents verify's `--language` default-free" is **TRUE** against the manual bytes (`| --language <LANGUAGE> | BIP-39 wordlist for --phrase |`, no `[default]`). The contrast claim (derive `:149` / decode `:102` DO annotate `default english`) is also TRUE. So no manual line drifts — manual-mirror genuinely N/A.
   - **(c)** "GUI schema-mirror gates flag-names + value-enums, NOT help-text defaults" is **CONSISTENT** with the project description AND with the live `classify_flag` source: the emitted per-flag entry is `{name, kind, choices}` with NO `default` field; an `Option<ValueEnum>` and a `ValueEnum`+`default_value` both classify as `dropdown` with the same `CliLanguage` choices. Schema-mirror N/A confirmed at the source level, not just by assertion.

3. **New bare-no-flag no-spurious-note test (§7.2) — PRESENT and genuinely RED-on-current / GREEN-after.** §7.2 "NO spurious note when flag OMITTED (I-new — the Option-ization guard): `ms verify <ja-ms1>` (NO `--language`) → exit 0 AND stderr contains NO disagreement note." This RED-fails on `origin/master` because today's `default_value="english"` makes omission indistinguishable from explicit `--language english`, firing the spurious note; it GREENs only after Option-izing + computing `defaulted=true`. The complementary explicit-`--language english` note test (line 280) pins the other direction, so the pair is non-vacuous and brackets the exact behavior the re-type changes.

4. **D5b row added + consistent — YES.** D5b (line 334) carries Option A, the idiom, the surface-delta (removed `[default]` only), and cites §3.3/§6 + the §7.2 test. No contradiction with §3.3, §6, or D13.

5. **Helper 3-tuple consistency unbroken — YES.** The fold did NOT touch the helper signature; it supplies the missing `cli_lang_defaulted` INPUT for verify. The 3-tuple `(Zeroizing<Vec<u8>>, CliLanguage, bool)` remains consistent at §3.1 (55/72-77), §3.2 (100), §3.3 (133/135/138), D2 (328), D2a (329). No 2-tuple regression introduced.

**No NEW finding surfaced by the I-new fold.** The fold is minimal (one arg re-type + one `match` resolution, mirroring decode verbatim) and introduced no drift: every existing verify test/lint is accounted for (the only label-bearing test, `verify_phrase_round_trip_ok`, stays GREEN because its `entr`/English path renders `effective_lang.as_str() == "english"` unchanged), the schema-mirror and manual gates are source-confirmed N/A, and no orphaned `args.language` consumer remains.

---

## Minor

### M-7 (new, non-blocking) — §3.3 round-trip "supplied side parses under `effective_lang`" subtly changes verify-`--phrase` semantics vs the bare-arg model, but the change is funds-safe and already decision-justified
Pre-fold, verify's `--phrase` supplied+derived both parse under `args.language` (the CLI flag). Post-fold both parse under the WIRE `effective_lang`. For a `Mnem` card this is the intended fix (§3.3 line 140 rationale: the card's language is ground truth; a wrong-language phrase → `Bip39` parse-fail/exit 1 or `VerifyPhraseMismatch`/exit 4, never a false GREEN). For an `Entr` card `effective_lang == cli_lang` so behavior is byte-unchanged. This is correct and already R0-justified in §3.3; flagged only so the implementer keeps the supplied-side `parse_in` on `effective_lang` (NOT a reintroduced `args.language`). No design change. (Carries no funds risk; subsumed by the §7.2 true-negative test at line 279.)

### M-5 / M-2 / M-4 / M-6 (carried) — still correctly scoped Minor/optional
- M-5 (round-trip cite `:84-…` loose; load-bearing lines `:85-95`): cosmetic, unchanged. The §3.3 prose now also names `:86`/`:95` explicitly elsewhere, so the implementer has the precise sites.
- M-2 (Chinese `as_str()` word-order vs toolkit `human_name()`): §4.2 + D8 footnote it; tests use Japanese. Resolved as documented.
- M-4 (L5 Debug test hardening for a non-`InvalidChecksum` codex32 arm): §7.4 carries it OPTIONAL. Fine.
- M-6 (L26 `form` substring assertion): §7.3 asserts the distinctive substring. Fine.

---

## Cross-charge verdicts (explicit)

- **I-new RESOLVED.** Option A explicitly chosen (re-type `verify.rs:30-31` → `Option<CliLanguage>`), `cli_lang_defaulted` source stated (the verbatim decode `Some=>false / None=>true` idiom), §6 + manual + schema-mirror reconciled and SOURCE-VERIFIED (manual `:297` default-free; `classify_flag` emits no `default`; flag-name + `CliLanguage` value-enum unchanged → both gates N/A), D5b row added and consistent, bare-no-flag no-spurious-note test added (RED-on-current / GREEN-after). The helper 3-tuple is intact.
- **I-1 STILL RESOLVED.** Helper is the 3-tuple everywhere; all four derive label sites + verify `:95` thread `effective_lang`/`effective_lang_defaulted`; label-pin tests present. The I-new fold did not disturb the label threading (verify's `:95` already reads `effective_lang.as_str()`; the re-type only changes how `(cli_lang, defaulted)` is sourced upstream).
- **I-2 STILL RESOLVED.** Helper invoked ONLY on the `Ok((tag, payload))` arm; both `Err` arms (exit-3 `ReservedTagNotEmittedInV01` → `emit_future_format`, and generic `Err(e)`) preserved verbatim. Live verify structure (4-arm `Result` match, exit-3 via `FutureFormat`→`exit_code 3`) re-confirmed. The Option-ize fold touches the arg decl + the round-trip leg, NOT the decode match arms — no regression to I-2's resolution.
- **M-1 STILL RESOLVED.** decode-only note parity vs combine bare-resolution distinction intact (§3.4/D3); the I-new fold reinforces decode-parity (the whole point of Option A is to match decode's omission-suppresses-note behavior). Live-confirmed `CombineArgs` has no `--language`.
- **M-3 STILL RESOLVED.** `--to ms1` passes original `&payload` → `Mnem` language byte preserved; §7.3 test pins it. Untouched by the I-new fold.
- **No new drift** in helper signature, resolved-decisions table, open-questions, funds core, OR the existing test/lint surface (drift-swept: no verify test breaks under the re-type; schema-mirror + manual + help-pointer + output-class gates all source-confirmed neutral).

---

## TDD integrity

RED-first design remains genuine end-to-end. H4/H5 build real non-English mnem ms1 cards → panic today (`unreachable!` live at `derive.rs:185` / `verify.rs:64`) → cannot pass on `origin/master`. I-1 label-pin tests, I-2 exit-3 no-regression, and M-3 `--to ms1` decode-back are non-vacuous. The I-new gap is now closed by the §7.2 bare-no-flag no-spurious-note test, which is genuinely RED-on-current (today's `default_value="english"` fires the spurious note on a bare non-English verify) and GREEN-only-after Option-izing. The complementary explicit-`--language` note test (line 280) brackets the other direction. Drift-check: the sole existing label-asserting verify test (`verify_phrase_round_trip_ok`, `language=english`) stays GREEN under the re-type, so the fold ships no silent regression.

---

## VERDICT

**R0 ROUND 3: 0C / 0I → GREEN**

- C: none.
- I: none. **I-new RESOLVED** (Option A chosen + `cli_lang_defaulted` source stated + §6/manual/schema-mirror reconciled and source-verified + D5b row added + bare-no-flag no-spurious-note test added). No NEW Important surfaced by the fold.
- I-1, I-2, M-1, M-3 (round 1) remain RESOLVED and re-verified against `44ac71f` bytes; the helper is a consistent 3-tuple; the verify `--language` re-type is self-consistent (only two `args.language` consumers, both replaced) and breaks no existing test or lint; schema-mirror and manual-mirror are source-confirmed N/A.

The spec has converged. Per CLAUDE.md the R0 gate is satisfied — implementation MAY begin (single-subagent-per-phase TDD off `origin/master` `44ac71f`), followed by the mandatory per-phase R0 reviews (full `cargo test -p ms-cli`) and the non-deferrable whole-diff post-impl adversarial review. Carry the Minor items (M-2/M-4/M-5/M-6/M-7) as implementer notes; none gate.
