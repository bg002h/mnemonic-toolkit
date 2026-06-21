# Cycle-8 PLAN R0 — Round 1 Review

**Artifact:** `design/IMPLEMENTATION_PLAN_cycle8_mscli_panics.md` (ms-cli H4 · H5 · L26 · L5)
**Spec it implements:** `design/BRAINSTORM_cycle8_mscli_panics.md` (R0 ROUND 3 GREEN)
**mnemonic-secret `origin/master` SHA:** `44ac71f` (`44ac71fd6cb055e25e3c3dd1daca230d5d45bafb`) — verified against bytes
**Date:** 2026-06-21
**Reviewer charge:** adversarial; HARD R0 gate (NO code until 0C/0I); FUNDS item (H4/H5 must derive the RIGHT fingerprint).

---

## Verification log (all against live `44ac71f` bytes, not the local tree)

| Claim | Result |
|---|---|
| `origin/master` == `44ac71f` | ✅ confirmed (`git rev-parse origin/master`) |
| `derive.rs:185` H4 `unreachable!` inside ms1-branch match `:183-186` | ✅ exact |
| `derive.rs:165-168` `(cli_lang, defaulted)` resolution; `:169` `lang = cli_lang.into()` | ✅ exact |
| `derive.rs:62` `--language` already `Option<CliLanguage>` | ✅ exact |
| `derive` label sites `:231` JSON `language`, `:232` `language_defaulted`, `:244` `if defaulted`, `:245` `(DEFAULT)`, `:246-249` english-default note, `:251` non-default `language:` | ✅ exact |
| `derive` binds `stderr` at `:121` (`std::io::stderr()`, `mut`) — in scope at ms1-branch | ✅ (resolves the plan's "hoist" caveat — no hoist needed) |
| `verify.rs:30-31` `#[arg(long, default_value="english")] pub language: CliLanguage` (NON-Option) | ✅ exact (re-type target) |
| `verify.rs:60-73` 4-arm match over `Result<(Tag,Payload),Error>`; `:61` Entr, `:64` H5 panic, `:65-71` exit-3 `ReservedTagNotEmittedInV01 => emit_future_format`, `:72` generic `Err(e)` | ✅ exact |
| `verify.rs` `args.language` consumers EXACTLY two: `:86` `let lang: Language = args.language.into();` + `:95` `emit_round_trip_ok(…, args.language.as_str(), …)` | ✅ exact (grep-confirmed no third) |
| `verify.rs:87` supplied `parse_in(lang,…)`, `:88` derived `from_entropy_in(lang,…)` | ✅ exact |
| `verify::run` does NOT bind a `stderr` at top (plan's Edit B introduces `let mut stderr = std::io::stderr().lock();`) | ✅ correct — net-new binding needed |
| `decode.rs:44-46` cli_lang idiom; `:63-82` 2-tuple language match; `:72` note string; `:81` `_ => unreachable!`; `:83-87` entropy match | ✅ (plan cites `:63-81` for the 2-tuple part; live closes at `:82`/the entropy match runs `:83-87`. Load-bearing `:72`/`:81` exact.) |
| decode note string `:72` = `"note: this ms1 carries wordlist language '{}'; ignoring --language {}"` | ✅ byte-for-byte == plan/spec helper |
| `combine.rs:95-107` `(entropy, language, kind)` match (Entr→English, Mnem→`from_code`); `:109` `match args.to`; `:111` Entropy arm; `emit_entropy` fn `:157`; `language: None` `:165`; `emit_ms1` `:178` takes `&payload` | ✅ (plan says `:95-108`/`:109`; live match closes `:107`, `:109` is `match args.to`. Immaterial off-by-one.) |
| `combine` args `{shares, to, json}` — NO `--language` | ✅ exact (`:35-47`) |
| `CliLanguage::from_code` `:34-48` (0..=9→Some, else None); `as_str()` `:51`; `From<CliLanguage> for bip39::Language` `:67`; NO `human_name()` | ✅ (plan says `:34-49`; body is `:34-48`. M-2 `as_str()`-only confirmed.) |
| `CliError` `:12` derive(Debug), `:13` non_exhaustive, `:14` enum, `:20` `Codex32(codex32::Error)`; `Display:118`, `kind:57`, `message:73`, `exit_code:43`, `details:98`; `From<ms_codec::Error>:132`, `:136` `Codex32(c)=>` | ✅ exact |
| `codex32_friendly.rs:27` `InvalidChecksum { checksum, .. } =>` (DROPS `string`) | ✅ exact — Display safe, derived Debug leaks (L5 genuinely latent) |
| `advisory.rs:13` `secret_in_argv_warning`, `:37` `emit_output_class_advisory`, `:10` `use std::io::Write` (already imported) | ✅ exact |
| `cmd/mod.rs` declares `combine decode derive encode gui_schema inspect repair split vectors verify` (no `payload_lang`) | ✅ exact — `payload_lang` alphabetically between `inspect` and `repair` is correct |
| toolkit `language.rs:176` `non_english_seed_advisory(lang, form) -> Option<String>`, English→None, text body | ✅ exact; ms-cli port text matches byte-for-byte (uses `as_str()` for `{name}` vs toolkit `human_name()`) |
| `Payload` `#[non_exhaustive]` (payload.rs:29); `Mnem { language: u8, entropy: Vec<u8> }` `:51-55`; `Entr(Vec<u8>)` `:44` | ✅ exact — `_ =>` guard legally required AND a future-variant guard |
| Oracle: `cli_derive.rs:14-15` `MASTER_FP_EN=73c5da0a` / `MASTER_FP_FR=7d53dc37`; `:85-87` asserts EN≠FR | ✅ exact (funds oracle real) |
| Existing fixtures: `verify_future_format.rs` (exit-3 input `Codex32String::from_seed("ms",0,"seed",Fe::S,…)` → tag "seed"), `verify_phrase_round_trip_ok.rs` (uses `ms10entr…` **Entr** card, no `--language`, asserts `language=english`), `decode_mnem_japanese.rs`, `encode_mnem_japanese.rs`, `cli_combine.rs`, `decode_explicit_language_no_warning.rs` | ✅ all present; new test files absent (net-new) |
| Manual `43-ms.md:297` verify `--language` line | ✅ `\| --language <LANGUAGE> \| BIP-39 wordlist for `--phrase` \|` — NO `[default]`/`DEFAULT` annotation (Option-ization drifts no manual line; contrast derive `:149`/decode `:102/:125` which DO annotate) |
| ms-codec `=0.5.0` pin (Cargo.toml `:20`); ms-codec `0.5.0` on crates.io | ✅ sparse-index confirms ms-codec `[0.4.1..0.5.0]` published → `cargo publish -p ms-cli` precondition holds |
| ms-cli current `0.8.1`; `0.9.0` not yet published | ✅ sparse-index `[…0.8.0, 0.8.1]` → tag/publish `0.9.0` is a valid new version |
| CHANGELOG: root `CHANGELOG.md` (crate-prefixed; carries `## ms-cli [0.8.1]`) + `crates/ms-codec/CHANGELOG.md`; **NO `crates/ms-cli/CHANGELOG.md`** | ✅ root IS the canonical ms-cli changelog |
| No `0.8.1` self-pins in ms-cli source / READMEs | ✅ README sweep will find nothing |

**Funds core (per charge item 2):** the 3-tuple helper routes BOTH derive (P2) and verify (P3) through `payload_entropy_and_language` on the `Ok((tag,payload))` arm ONLY; Mnem returns the WIRE-byte language (correct fp), Entr passes `(cli_lang, cli_lang_defaulted)` through. derive threads `effective_lang`/`effective_lang_defaulted` to ALL FOUR label sites (`:231/:232/:244-249/:251`); verify threads it to `:95` and to both round-trip `parse_in`/`from_entropy_in`. Exit-3 + generic-Err arms preserved verbatim. **Confirmed faithful to the spec; no funds-wrong path and no mislabel path in the plan as written.**

---

## CRITICAL

*(none)*

---

## IMPORTANT

*(none)*

---

## MINOR

**M1 — Bughunt-report tick-box line numbers are STALE (wrong by 4–23 lines).**
`§0` table (line 55) and `§8` step 7 (lines 537–540) cite the toolkit bughunt-report checkbox lines as **H4 `:139`, H5 `:159`, L5 `:342`, L26 `:1023`**, and the `§0` table asserts these were "all `- [ ]`" re-grepped against `44ac71f`. The LIVE `### - [ ]` checkbox header lines in `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` (origin/master) are:
- **H4 → `:143`** (`### - [ ] H4 · `ms derive` panics …`)
- **H5 → `:163`**
- **L5 → `:351`**
- **L26 → `:1046`**

Every cited number lands several lines BEFORE the actual checkbox (on adjacent prose). This is exactly the "citations decay every merge — re-grep at write time" rule (CLAUDE.md). Not gate-blocking because (a) it only affects a status-tick step at SHIP, not any code edit, and (b) the *target slugs* are unambiguous by name. **Required fix:** correct the four line numbers to `:143/:163/:351/:1046` (or instruct the implementer to grep `^### - \[ \]` by finding-ID at tick time rather than trusting the snapshot).

**M2 — `§8` step 2 names a nonexistent `crates/ms-cli/CHANGELOG.md` authoritatively, then step 3 hedges correctly.**
There is NO `crates/ms-cli/CHANGELOG.md` at `44ac71f` — only the root `CHANGELOG.md` (the canonical, crate-prefixed ms-cli/ms-codec changelog, currently carrying `## ms-cli [0.8.1]`) and `crates/ms-codec/CHANGELOG.md`. Step 2 says "Update `crates/ms-cli/CHANGELOG.md`"; step 3 then correctly says "if ms-cli's CHANGELOG lives only in the root, write the `0.9.0` section there." The two are internally inconsistent. **Required fix:** state definitively that the ms-cli `0.9.0` entry goes in the **root `CHANGELOG.md`** as `## ms-cli [0.9.0]`, and drop the nonexistent-file reference in step 2. (Resolution is unambiguous, hence Minor.)

**M3 — RED-first MECHANISM of the bare-no-flag verify test (P3 test 5) is described imprecisely.**
The plan/spec frame the "NO spurious note when flag OMITTED" test as RED because "today's `default_value="english"` makes omission look like explicit `--language english` → spurious note." But at `44ac71f` verify PANICS at `verify.rs:64` (`unreachable!`) on any Japanese `Mnem` ms1 BEFORE any note logic runs — so today the test is RED via the **panic/non-zero exit** (failing assertion *a*, exit 0), not via an emitted spurious note. The test is genuinely RED-first and non-vacuous, and assertion *b* (no disagreement note) is the precise Option-ization guard once *a* is satisfied. **No fix required**, but the phase note could state "RED today via the `unreachable!` panic; assertion *b* specifically guards the Option-ization once the panic is removed" for accuracy. (Same applies to P3 tests 1/2/4/6 — all RED today via the panic, which is fine and standard.)

**M4 — `§0` / `§3` span endpoints drift by 1–2 lines vs live (load-bearing anchors correct).**
The plan cites the decode template as `:63-81` (live 2-tuple match closes at `:82`; entropy match `:83-87`), `from_code` as `:34-49` (live `:34-48`), combine as `:95-108`/`:109` (live match closes `:107`). All load-bearing single-line anchors (`decode.rs:72` note, `:81` guard; `combine.rs:165` `language: None`; `error.rs:20/:132/:136`; `derive.rs:185`; `verify.rs:64`) are EXACT. The plan itself acknowledges "structure is byte-identical; only span endpoints differ." Cosmetic. **No fix required.**

---

## Charge-item adjudication (explicit)

1. **Edit sites exist as cited** — YES. All H4/H5/L26/L5 anchors + label sites + decode parity + `from_code` verified exact against `44ac71f`. The plan's claim that it corrected the spec's stale combine citations is TRUE (live `:95-107`/`:109`/`:165` used). Only the **bughunt-report tick lines** are stale (M1).
2. **Funds core** — YES. Helper routes BOTH derive + verify through the 3-tuple on the `Ok((tag,payload))` arm ONLY; Mnem uses the WIRE byte (correct fp), Entr passes through; `effective_lang`/`effective_lang_defaulted` threaded to EVERY label site (derive `:231/:232/:244-249/:251`, verify `:95`) AND both round-trip parse/build sites. Verify Err arms (exit-3 `ReservedTagNotEmittedInV01` + generic) preserved verbatim. No entropy-path miss (would be funds-wrong); no label miss (would mislabel).
3. **`--language` Option-ization** — YES. Plan re-types verify `:30-31` to `Option<CliLanguage>`, removes `default_value="english"`, replaces BOTH consumers (`:86` deleted in favor of `effective_lang`; `:95` → `effective_lang.as_str()`), and computes `defaulted` via the `Some/None` idiom. Ripple accounted for: schema_mirror N/A (flag-name + value-enum unchanged), manual `43-ms.md:297` already default-free (verified), no third `args.language` consumer, existing `verify_phrase_round_trip_ok.rs` stays GREEN (Entr/English card → `effective_lang==English`, verified).
4. **L26 / L5** — YES. L26 advisory fires on the `--to entropy` arm ONLY, stderr-only, no `--json`/exit change (`CombineJson.language:None` at `:165` untouched; `emit_ms1` re-encodes original `&payload` preserving the language byte). L5 hand-rolled `Debug` delegates to `kind()+message()`; `friendly_codex32`'s `InvalidChecksum { checksum, .. }` arm DROPS `string` → the secret cannot leak via the sanitized path; genuinely latent (no live `{:?}` site; Display already safe).
5. **TDD integrity** — YES. Each RED is genuinely RED-first today: non-English derive/verify panic at `derive.rs:185`/`verify.rs:64`; bare-no-flag spurious-note (RED via panic, then guards Option-ization — M3); L5 derived-Debug leak. Label-pin + exit-3 no-regression (real fixture `verify_future_format.rs`) + `--to ms1` decode-back + English positive controls all present and non-vacuous. French oracle `7d53dc37`≠`73c5da0a` is real (`cli_derive.rs:14-15,85-87`).
6. **SemVer/publish** — YES. ms-cli MINOR `0.8.1`→`0.9.0`; ms-codec NO-BUMP (`=0.5.0` path-pin stays, `0.5.0` already on crates.io → publish works); NO toolkit pin/bump (toolkit deps ms-codec lib, not ms-cli bin); tag `ms-cli-v0.9.0` + `cargo publish -p ms-cli`. Version sites: ms-cli Cargo.toml `:3` + root `CHANGELOG.md` (NOT a per-crate ms-cli CHANGELOG — M2); no README self-pin to bump. Bughunt-report is canonical (the 4 slugs are NOT in ms `FOLLOWUPS.md`) — only the tick-line NUMBERS are stale (M1).

---

## VERDICT

**PLAN R0 ROUND 1: 0C / 0I** — **GREEN.**

Four Minors (M1 stale bughunt tick-line numbers; M2 nonexistent ms-cli CHANGELOG named in step 2; M3 RED-mechanism wording; M4 cosmetic span drift) — none gate implementation. The plan faithfully executes the R0-GREEN spec, the funds core is correct (wire-byte authority on every derive/verify entropy path + every label site), verify's exit-3 leg is preserved verbatim, the Option-ization ripple is fully accounted, and the publish preconditions hold. Recommend folding M1/M2 (cheap, ship-step accuracy) and proceeding to implementation.
