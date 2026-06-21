# IMPLEMENTATION PLAN — cycle-8: ms-cli robustness/advisory cluster (H4 · H5 · L26 · L5)

**Status:** PLAN ONLY (no code yet). This plan-doc faithfully phases the R0-GREEN brainstorm spec
`design/BRAINSTORM_cycle8_mscli_panics.md` (R0 ROUND 3 = 0C/0I → GREEN,
`design/agent-reports/cycle8-spec-r0-round3-review.md`). Per CLAUDE.md the plan-doc itself MUST pass
the opus-architect **R0 loop to 0C/0I** before any implementation begins. This doc is the input to
that loop — it does NOT review, fold, or implement.

**Cycle:** constellation bug-fix program, cycle-8 (next after cycles 1–2 shipped toolkit v0.62.0 +
md-cli v0.8.0).

**Repo of record:** **mnemonic-secret** (`ms-codec` / `ms-cli`). Default branch `master`.

---

## 0. Source-of-truth SHA + stale-local caveat

| Repo | Pin / SHA | Notes |
|---|---|---|
| **mnemonic-secret** `origin/master` | **`44ac71f`** (`44ac71fd6cb055e25e3c3dd1daca230d5d45bafb`) | ms-codec `0.5.0`, ms-cli `0.8.1`. **All line numbers in this plan re-grepped against these BYTES** via `git -C /scratch/code/shibboleth/mnemonic-secret show origin/master:<path>` (2026-06-21). |
| mnemonic-toolkit `origin/master` | `8d2fe505` (advisory-text source + bughunt report) | NOT bumped by this cycle. |
| codex32 (crates.io dep) | `=0.1.0` exact-pinned | `Error::InvalidChecksum { checksum: &'static str, string: String }` — `string` carries the secret ms1 (L5 root). |

> ⚠ **STALE LOCAL TREE — DO NOT trust the `/scratch/code/shibboleth/mnemonic-secret` checkout.**
> It is **2 commits behind** `origin/master`, **dirty** (uncommitted edits to
> `crates/ms-cli/src/error.rs`, `crates/ms-codec/src/{error.rs,shares.rs}` + Cargo.toml/CHANGELOGs),
> and **structurally OLDER** (a flat `src/` with **no `cmd/` directory** — a naive `find`/`Read`
> returns a misleading tree lacking `src/cmd/{derive,verify,combine,decode}.rs`).
> **Branch implementation off `origin/master` `44ac71f`; stash/inspect the stray edits first;
> cite `git show origin/master:…` line numbers (this plan did).**

**Branch:** `feature/cycle8-mscli-panics-advisory` off `44ac71f` (spec D14).

**Re-grep verification (`44ac71f` bytes, 2026-06-21) — live line numbers used by this plan:**

| Site | Live `44ac71f` location | Confirmed |
|---|---|---|
| `derive` `--language` arg | `derive.rs:62` `pub language: Option<CliLanguage>` (already `Option`) | ✅ |
| `derive` cli_lang resolution | `derive.rs:165-168` `match args.language { Some(l)=>(l,false), None=>(English,true) }` | ✅ |
| `derive` H4 panic | `derive.rs:185` `_ => unreachable!("ms-codec v0.1 decodes only Payload::Entr")` (inside ms1-branch match `:184-186`) | ✅ |
| `derive` label sites | JSON `:231` `language: cli_lang.as_str()`, `:232` `language_defaulted: defaulted`; text `:245` `language: … (DEFAULT)`, `:246-249` english-default `note:` (inside `if defaulted`), `:251` non-default `language: {cli_lang.as_str()}` | ✅ |
| `verify` `--language` arg | `verify.rs:30-31` `#[arg(long, default_value = "english")] pub language: CliLanguage` (NON-`Option` — re-type target) | ✅ |
| `verify` decode match | `verify.rs:60-73` over `Result<(Tag,Payload),Error>`: `Ok((_,Entr(b)))` `:61`, `Ok((_,_)) => unreachable!` `:64` (H5 panic), `Err(ReservedTagNotEmittedInV01{got}) => emit_future_format` `:65-71` (exit-3 leg), `Err(e) => return Err(e.into())` `:72` | ✅ |
| `verify` `args.language` consumers | EXACTLY two: `:86` `let lang: Language = args.language.into();` + `:95` `emit_round_trip_ok(…, args.language.as_str(), …)` (grep-confirmed no third) | ✅ |
| `verify` round-trip leg | `:85-99` (`parse_in` `:87` / `from_entropy_in` `:88` / `emit_round_trip_ok` `:95`) | ✅ |
| `decode` helper template | `decode.rs:44-46` cli_lang idiom; `:63-81` `(effective_lang, effective_lang_defaulted)` 2-tuple match incl. `:72` note string + `:81` `_ => unreachable!("ms-codec decode returned unknown Payload variant")` | ✅ |
| `combine` language resolution | `combine.rs:95-108` `(entropy, language, kind)` match (`Entr`→English, `Mnem`→`from_code(*wire_code).unwrap_or(English)`); `:109` `match args.to`; `:110` Phrase, `:111` Entropy→`emit_entropy`, `:112` Ms1; `emit_entropy` fn `:157`, `language: None` `:165` | ✅ |
| `combine` args | `combine.rs:35-46` `CombineArgs { shares, to, json }` — NO `--language` | ✅ |
| `CliLanguage::from_code` | `language.rs:34-49` (0..=9 → `Some`, else `None`); `as_str()` `:51` (kebab); `From<CliLanguage> for bip39::Language` `:67` (NOT `From<Option<…>>`) | ✅ |
| `CliError` Debug | `error.rs:12` `#[derive(Debug)]`, `:14` `enum CliError`, `:20` `Codex32(codex32::Error)`; `Display` `:118`; `kind()` `:57`, `message()` `:73` (Codex32→`friendly_codex32`), `exit_code()` `:43`, `details()` `:98`; `From<ms_codec::Error>` `:132` (`:136` `Codex32(c) => CliError::Codex32(c)`) | ✅ |
| `friendly_codex32` sanitizer | `codex32_friendly.rs:27` `InvalidChecksum { checksum, .. } =>` (DROPS `string`) — Display safe, Debug leaks | ✅ |
| ms-cli advisory module | `crates/ms-cli/src/advisory.rs` (top-level `src/`, NOT under `cmd/`); existing `secret_in_argv_warning` `:13`, `emit_output_class_advisory` `:37` | ✅ |
| `cmd` module list | `cmd/mod.rs` declares `combine decode derive encode gui_schema inspect repair split vectors verify` — new helper module must be wired here if a new file is added | ✅ |
| toolkit advisory source | `mnemonic-toolkit` `crates/mnemonic-toolkit/src/language.rs:176-187` `non_english_seed_advisory(lang, form) -> Option<String>` (uses `human_name()`; English→`None`) | ✅ |
| bughunt report tick-boxes | `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` — H4 `:139`, H5 `:159`, L5 `:342`, L26 `:1023` (all `- [ ]`) | ✅ |

> **Note on spec vs live line drift:** the spec body cites a few combine line numbers as `:95-107` /
> `:111-115` / `:165-172`; the live `44ac71f` bytes are `:95-108` / `:109` / `:165`. Structure is
> byte-identical; only the span endpoints differ. This plan uses the **live** numbers above
> (per CLAUDE.md "re-grep at write time, use live line numbers"). The H4/H5/decode/error/language
> citations match the spec exactly.

---

## 1. Execution model (per CLAUDE.md)

- **Single implementer subagent in a git worktree off `origin/master` `44ac71f`** (NOT parallel
  re-implementations; NOT the dirty local tree). Branch `feature/cycle8-mscli-panics-advisory`.
- **TDD, RED-first, per phase:** write the failing test(s) first, confirm RED, then minimal impl to
  GREEN. Helper-unit tests live in a `#[cfg(test)] mod` beside the code; CLI integration tests are
  new `crates/ms-cli/tests/*.rs` files (`assert_cmd`-driven, one binary per file).
- **Per-phase gate (BOTH must pass before the phase is GREEN):**
  1. **FULL `cargo test -p ms-cli`** — NOT targeted `--test <one>` targets
     (per `feedback_r0_review_run_full_package_suite`: CLI/language changes ripple into
     argv/output-class/help-pointer/gui-schema lint tests outside any one finding's scope; a stale
     lint can be RED in a target you didn't run). Plus `cargo test -p ms-codec` (sanity — must stay
     untouched; ms-codec NO-BUMP).
  2. **`cargo clippy --all-targets -p ms-cli -- -D warnings`** — this IS the CI gate
     (`.github/workflows/rust.yml:153`).
- **NO `cargo fmt`.** The repo has **no fmt CI gate** (only `clippy` + `test` in `rust.yml`), and
  `crates/ms-cli/src/mlock.rs` is **permanently fmt-exempt** (MEMORY
  `project_g6_fmt_exemption_and_asymmetric_pin` — NEVER `cargo fmt` mlock.rs; a stray reformat
  desyncs the g6 byte-share with ms-cli-v0.7.0 and trips `mlock_g6_invariant.rs`). Format new code
  by hand to match the surrounding style; do **not** run any `cargo fmt` variant.
- **Per-phase opus R0 review** (full `cargo test -p ms-cli` in each review) persisted **verbatim**
  to `mnemonic-toolkit/design/agent-reports/cycle8-phase-N-<round>-review.md` BEFORE the
  fold-and-commit step. Reviewer-loop continues after every fold until 0C/0I.
- **Mandatory non-deferrable whole-diff post-impl adversarial review** before tag/publish (Phase 5,
  §"Post-impl gate").
- **Re-grep every cited line at edit time** — the table above is a `44ac71f` snapshot; if the
  branch base advances, re-verify.

**Phase disjointness:** P1 adds a new shared helper + its unit tests (no consumer wired yet — the
helper compiles and is unit-tested in isolation). P2 wires `derive`. P3 wires `verify` (incl. the
arg re-type). P4 adds the `combine` advisory. P5 fixes `CliError` Debug. P6 ships. Each phase
touches a disjoint consumer; only P1's helper is shared, and it is frozen after P1 (P2/P3 call it,
never edit it). This ordering means each phase's RED tests fail for exactly one reason.

---

## 2. Resolved decisions (settled in the spec — reference, do NOT re-decide)

These are already R0-GREEN in the brainstorm spec §9 (D1–D15) and round-3 review. The implementer
treats them as fixed inputs:

- **D1 — wire byte is authoritative for `Mnem`; `--language` is advisory-only for mnem cards.**
  BIP-39 seed = PBKDF2 over the language-specific *sentence*; the same entropy under two wordlists
  yields two different seeds → two different wallets. Oracle: all-zeros 16-byte entropy →
  English fp `73c5da0a` ≠ French fp `7d53dc37` (both already in `tests/cli_derive.rs`). A
  `--language`-default (English) patch would compute the WRONG fp for a non-English seed.
- **D2 / D2a — ONE shared helper** `payload_entropy_and_language` returning the 3-tuple
  `(Zeroizing<Vec<u8>>, CliLanguage, bool)` = `(entropy, effective_lang, effective_lang_defaulted)`
  (template = `decode.rs:63-81`'s 2-tuple language part + entropy). **EVERY label site reads
  `effective_lang`/`effective_lang_defaulted`, NEVER raw `cli_lang`/`defaulted`** — else a French
  card derives the correct fp but prints `language: english (DEFAULT)` (mislabeled card).
- **D3 — disagreement → wire wins, stderr `note:`, exit 0 (derive) / proceed (verify).** Byte-identical
  to `decode.rs`'s note string. `decode` is the note-emitting parity model; `combine` shares only
  the bare wire-resolution (no `--language` arg → no note).
- **D4 — `Mnem` becomes a real handled arm; `_ =>` stays as the `#[non_exhaustive]` future-variant
  guard only** (`from_code` covers 0..=9; codec rejects ≥10 at decode → no live path hits `_`).
- **D5 — verify `--phrase` round-trip: both supplied + derived mnemonics parsed/built under the
  WIRE language.** Mismatch → true negative (`Bip39` parse-fail / `VerifyPhraseMismatch`), never a
  false GREEN.
- **D5a — verify decode match is over `Result<(Tag,Payload),Error>` (FOUR arms).** Helper called
  ONLY on the `Ok((tag,payload))` arm; the `Err(ReservedTagNotEmittedInV01) => emit_future_format`
  (exit-3) arm + the generic `Err(e)` arm are **preserved VERBATIM**. Do NOT replace the whole match.
- **D5b — Option-ize verify's `--language`** from `CliLanguage`+`default_value="english"` to
  `Option<CliLanguage>` (matching `decode.rs:29`/`derive.rs:62`); compute `(cli_lang, defaulted)`
  via `match args.language { Some(l)=>(l,false), None=>(English,true) }`. Surface delta = removed
  `[default: english]` help annotation only → schema-mirror-neutral + manual-neutral.
- **D6/D7/D8 — L26: WARN (exit 0), only on `--to entropy`; port toolkit `non_english_seed_advisory`
  text into ms-cli `advisory.rs` using `CliLanguage::as_str()`** (== toolkit `human_name()` for all
  langs except Chinese word-order; cosmetic, ungated; tests use Japanese — M-2).
- **D9/D10 — L5: hand-roll `CliError` `Debug` delegating to sanitized `kind()`+`message()`; remove
  `#[derive(Debug)]`.** Latent (no live `{:?}` site) but fixed defensively. Option (b) rejected.
- **D11/D12/D13 — ms-cli MINOR `0.9.0`; ms-codec NO-BUMP; toolkit/GUI untouched; tag
  `ms-cli-v0.9.0` + `cargo publish -p ms-cli`; NO locksteps** (no flag-name / `--json` /
  dropdown-value change).
- **D14 — branch off `origin/master` `44ac71f`, not the dirty local tree.**
- **D15 — `cargo test -p ms-cli` (FULL package suite), per phase.**

Carried Minors (implementer notes, none gate): **M-2** keep `as_str()` (no `human_name` alias);
**M-4** optional L5 non-`InvalidChecksum`-arm hardening test; **M-5** round-trip cite span;
**M-6** L26 substring assertion; **M-7** keep supplied-side `parse_in` on `effective_lang`
(NOT a reintroduced `args.language`).

---

## 3. Phase 1 — shared `payload_entropy_and_language` helper (funds-safety core)

**Goal:** one private helper so `derive` + `verify` recover `(entropy, effective_lang,
effective_lang_defaulted)` from a decoded `Payload` identically, with the `#[non_exhaustive]` guard
living once. Mirrors the proven `decode.rs:63-81` policy.

**Location decision:** add a NEW module `crates/ms-cli/src/cmd/payload_lang.rs` and declare
`pub mod payload_lang;` in `cmd/mod.rs` (alphabetical: between `inspect` and `repair`). Rationale:
`derive`/`verify` both live under `cmd/`; a `cmd/`-local module is the lowest-friction shared home
and keeps the helper `pub(crate)`-scoped to the command layer. (The spec offered `cmd/mod.rs` OR a
new file as alternatives; a dedicated file is cleaner for the `#[cfg(test)] mod` unit tests.)

**Signature (spec §3.1):**
```text
use ms_codec::Payload;
use zeroize::Zeroizing;
use crate::language::CliLanguage;

/// Recover (entropy, effective wordlist language, effective-language-defaulted)
/// from a decoded Payload — the 3-tuple that derive/verify consume to reach
/// parity with `decode` (decode.rs:63-81 is the template for the 2-tuple
/// language part; this adds the entropy).
///
/// - Entr: language + defaulted are whatever the caller resolved from
///         --language/default → pass (cli_lang, cli_lang_defaulted) through.
/// - Mnem: the WIRE language byte is AUTHORITATIVE (CliLanguage::from_code);
///         --language is advisory-only; effective_lang_defaulted = FALSE (a real
///         wire language exists, never "defaulted"), mirroring decode.rs.
/// On Mnem/cli disagreement (cli explicit AND wire != cli) emit the decode.rs
/// note to stderr.
pub(crate) fn payload_entropy_and_language(
    payload: Payload,
    cli_lang: CliLanguage,
    cli_lang_defaulted: bool,
    stderr: &mut impl std::io::Write,
) -> (Zeroizing<Vec<u8>>, CliLanguage, bool)
```

**Body (behavior, lifted verbatim from `decode.rs:63-81`):**
- `Payload::Entr(b) => (Zeroizing::new(b), cli_lang, cli_lang_defaulted)`.
- `Payload::Mnem { language: wire_code, entropy } =>`
  - `let wire_cli = CliLanguage::from_code(wire_code).unwrap_or(CliLanguage::English);`
  - **advisory iff disagreement:**
    `if !cli_lang_defaulted && wire_cli != cli_lang { let _ = writeln!(stderr, "note: this ms1 carries wordlist language '{}'; ignoring --language {}", wire_cli.as_str(), cli_lang.as_str()); }`
    (byte-for-byte the `decode.rs:72` string).
  - return `(Zeroizing::new(entropy), wire_cli, false)`.
- `_ => unreachable!("ms-codec decode returned unknown Payload variant")` — STAYS as the
  `#[non_exhaustive]` guard (same wording as `decode.rs:81`).

> Note `Payload` takes `entropy` by value here (helper consumes `payload` by value) — unlike
> `combine.rs:95` which matches `&payload` because it also re-encodes via `emit_ms1(&payload,…)`.
> `derive`/`verify` consume the payload, so by-value move is correct and Zeroize-safe (the moved
> `Vec` is immediately re-wrapped in `Zeroizing`).

**RED-first unit tests (`#[cfg(test)] mod` in `payload_lang.rs`):**
1. **Entr pass-through:** `payload_entropy_and_language(Payload::Entr(vec![0;16]), English, true, &mut buf)`
   → returns `(entropy==[0;16], English, true)`; `buf` empty (no note).
2. **Entr non-defaulted pass-through:** same with `(French, false)` → `(…, French, false)`; no note.
3. **Mnem wire-wins, no flag:** build `Payload::Mnem { language: 6 /*french*/, entropy: [0;16] }`,
   call with `(English, true)` (defaulted) → returns `(…, French, false)`; `buf` empty
   (no disagreement note — flag was defaulted).
4. **Mnem disagreement note:** same Mnem(6), call with `(English, false)` (explicit english) →
   returns `(…, French, false)`; `buf` contains
   `note: this ms1 carries wordlist language 'french'; ignoring --language english`.
5. **Mnem agreement, explicit flag:** Mnem(6), call with `(French, false)` → `(…, French, false)`;
   `buf` empty (wire == cli → no note).

These are non-vacuous on their own (pure-function unit tests; no panic involved yet — the panic is
at the *consumer* sites P2/P3 wire). P1 is GREEN when all five pass + the helper compiles + clippy
clean. **No consumer is wired in P1** — `derive`/`verify` still panic on `Mnem` (their fix is
P2/P3). To avoid a dead-code clippy warning on the unused `pub(crate)` fn in P1, the helper is
exercised by the unit tests in the same crate (test usage counts), so `-D warnings` stays clean;
if clippy still flags it, gate it behind nothing — the P2 wiring lands one phase later and the
`#[cfg(test)]` usage suffices.

**Phase-1 gate:** `cargo test -p ms-cli` (full) GREEN + `cargo clippy --all-targets -p ms-cli -- -D warnings` clean.

---

## 4. Phase 2 — H4: `derive` consumes the helper

**Goal:** `ms derive <non-english-mnem-ms1>` stops panicking, derives the CORRECT (wire-language) fp,
and labels the card with the wire language (not `english (DEFAULT)`).

**Edit (`derive.rs`, ms1-branch `:182-186` + label sites):**
- Replace the ms1-branch match (`:183-186`):
  ```text
  let (_tag, payload) = ms_codec::decode(&ms1)?;
  let entropy: Zeroizing<Vec<u8>> = match payload {
      Payload::Entr(b) => Zeroizing::new(b),
      _ => unreachable!("ms-codec v0.1 decodes only Payload::Entr"),   // H4 PANIC
  };
  Mnemonic::from_entropy_in(lang, &entropy[..])…   // lang = CLI --language (WRONG for mnem)
  ```
  with a call to the P1 helper:
  ```text
  let (_tag, payload) = ms_codec::decode(&ms1)?;
  let (entropy, effective_lang, effective_lang_defaulted) =
      crate::cmd::payload_lang::payload_entropy_and_language(payload, cli_lang, defaulted, &mut stderr);
  // build the mnemonic under the WIRE language
  Mnemonic::from_entropy_in(effective_lang.into(), &entropy[..]).map_err(CliError::Bip39)?
  ```
- **The hex/`--phrase` source arms are UNTOUCHED** (no wire byte; `cli_lang`/`defaulted` govern,
  exactly as today). Those arms set `effective_lang = cli_lang`, `effective_lang_defaulted =
  defaulted` so the downstream label sites have one consistent pair regardless of source. Structure
  this so all three source branches produce `(mnemonic, effective_lang, effective_lang_defaulted)`
  — e.g. hex/phrase arms bind `effective_lang = cli_lang; effective_lang_defaulted = defaulted;`
  before/after building the mnemonic. (`cli_lang`/`defaulted` are still computed at `:165-168` as
  the helper INPUT — they are no longer read by the label sites.)
- **Thread `effective_lang`/`effective_lang_defaulted` into ALL FOUR label sites (D2a / I-1):**

  | Site (`44ac71f`) | Today reads | MUST read |
  |---|---|---|
  | `:231` JSON `language: cli_lang.as_str()` | `cli_lang` | `effective_lang.as_str()` |
  | `:232` JSON `language_defaulted: defaulted` | `defaulted` | `effective_lang_defaulted` |
  | `:245` text `language: … (DEFAULT)` + `:246-249` english-default `note:` (both in `if defaulted`) | `defaulted`/`cli_lang` | `effective_lang_defaulted`/`effective_lang` |
  | `:251` text non-default `language: {cli_lang.as_str()}` | `cli_lang` | `effective_lang.as_str()` |

  i.e. rename the `if defaulted {…} else {…}` at `:244-252` to `if effective_lang_defaulted {…}
  else {…}` and swap the two `cli_lang.as_str()` reads to `effective_lang.as_str()`. The
  `&mut stderr` passed to the helper is the same `stderr` the label sites already use (`derive.rs`
  binds `stderr` near top of `run`; confirm the binding is in scope at the ms1-branch — if it's
  created later, hoist it or pass a fresh `std::io::stderr().lock()`).

**RED-first integration test — new `crates/ms-cli/tests/derive_mnem_non_english.rs`** (model on
`cli_derive.rs` oracle fps + `encode_mnem_japanese.rs` build pattern):
1. **Funds-safety core (French, RED→GREEN):** build a French mnem ms1
   (`ms encode --language french --phrase <fr 12-word from [0;16]>` → ms1), then `ms derive <ms1>`.
   - TODAY: panics (`unreachable!`) → abort/non-zero.
   - AFTER: exit 0, stdout **contains `7d53dc37`** (correct French fp) AND **does NOT contain
     `73c5da0a`** (the wrong English fp a naive patch emits). *This is the funds-safety proof.*
2. **Japanese variant + derive-from-card == derive-from-phrase parity:** build a Japanese mnem ms1
   (`[0xAB;16]`, `decode_mnem_japanese.rs` pattern) → `ms derive <ms1>` exits 0, and its fp ==
   `ms derive --phrase <ja-phrase> --language japanese` fp for the same phrase.
3. **Disagreement note (explicit flag):** `ms derive --language english <french-ms1>` → exit 0,
   stdout `7d53dc37`, **stderr contains the `note:` ignoring `--language english`**.
4. **LABEL PIN (I-1 mislabel-card guard):** `ms derive <french-ms1>` (NO `--language`) → exit 0,
   fp `7d53dc37`, AND **stdout text contains `language: french`** (NOT `english (DEFAULT)`), AND
   **stderr does NOT contain the `:246-249` english-default note**. `--json` variant:
   `language == "french"` AND `language_defaulted == false`.
5. **Positive control (Entr no-regression):** `ms derive <english-entr-ms1>` (entropy-only card,
   e.g. `ms encode --hex <zeros>`) with no `--language` → exit 0, fp `73c5da0a`, stdout
   `language: english (DEFAULT)` + the english-default note on stderr, **no** disagreement note.
   *Contrast pin:* an english-`Mnem` card (built from an english phrase via `encode`) prints
   `language: english` WITHOUT `(DEFAULT)` and no default note (its `effective_lang_defaulted ==
   false`) — confirms the Entr-vs-Mnem label split.

**Phase-2 gate:** full `cargo test -p ms-cli` + clippy. (Confirm no existing `cli_derive.rs`
assertion regresses — its English/Entr paths render `effective_lang == cli_lang` unchanged.)

---

## 5. Phase 3 — H5: `verify` consumes the helper (both legs) + Option-ize `--language`

**Goal:** `ms verify <non-english-mnem-ms1>` stops panicking; the `--phrase` round-trip honors the
wire language; the exit-3 future-format leg is preserved; a bare (no-flag) non-English verify emits
NO spurious disagreement note.

**Edit A — Option-ize the arg (D5b / I-new), `verify.rs:30-31`:**
```text
-    /// BIP-39 wordlist for --phrase. Default `english`.
-    #[arg(long, default_value = "english")]
-    pub language: CliLanguage,
+    /// BIP-39 wordlist for --phrase.
+    #[arg(long)]
+    pub language: Option<CliLanguage>,
```
Then at the top of the round-trip resolution (mirroring `decode.rs:44-46`):
```text
let (cli_lang, defaulted) = match args.language {
    Some(l) => (l, false),
    None    => (CliLanguage::English, true),
};
```

**Edit B — restructure the decode match (D5a / I-2), `verify.rs:60-73`. PRESERVE the two `Err`
arms VERBATIM; route ONLY the `Ok((tag,payload))` arm through the helper:**
```text
let mut stderr = std::io::stderr().lock();
let (entropy, effective_lang, _effective_lang_defaulted): (Zeroizing<Vec<u8>>, CliLanguage, bool) =
    match ms_codec::decode(&ms1) {
        Ok((_tag, payload)) =>
            crate::cmd::payload_lang::payload_entropy_and_language(payload, cli_lang, defaulted, &mut stderr),
        Err(ms_codec::Error::ReservedTagNotEmittedInV01 { got }) => {   // exit-3 leg — VERBATIM
            emit_future_format(&got, args.json)?;
            return Ok(0);
        }
        Err(e) => return Err(e.into()),                                  // generic Err — VERBATIM
    };
```
The old `Ok((_, Payload::Entr(b)))` (`:61`) + `Ok((_, _)) => unreachable!` (`:64`) arms COLLAPSE
into the single `Ok((_tag, payload)) => helper(…)`. **DO NOT replace the whole match with the
helper** (the helper takes a `Payload`, not a `Result`; a whole-match swap drops the exit-3 path).

**Edit C — round-trip leg honors `effective_lang` (D5 / M-7), `verify.rs:85-99`:**
- `:86` was `let lang: Language = args.language.into();` → DELETE; both sides use `effective_lang`.
- `:87` supplied: `Mnemonic::parse_in(effective_lang.into(), supplied.as_str())?`
- `:88` derived:  `Mnemonic::from_entropy_in(effective_lang.into(), &entropy[..]).expect("ms-codec validates entropy length")`
- `:95` label: `emit_round_trip_ok(&derived_mnemonic, effective_lang.as_str(), args.json)?`
  (the WIRE language, NOT `args.language.as_str()` — verify `:95` is the I-1 verify label site).
- After the fold, `args.language` is consumed ONLY by the `match args.language { Some/None }`
  resolution (Edit A) — NO orphaned `.into()`/`.as_str()` on the bare `Option<CliLanguage>`
  (round-3 review §"Other args.language consumer check" confirms exactly two consumers, both
  replaced). The disagreement `note:` is emitted once by the helper (Edit B), not here.

**RED-first integration test — new `crates/ms-cli/tests/verify_mnem_non_english.rs`:**
1. **RED→GREEN (no `--phrase`):** Japanese mnem ms1 → `ms verify <ms1>`: TODAY panics; AFTER exit 0.
2. **`--phrase` round-trip (wire honored):** `ms verify --phrase <ja-phrase> <ja-ms1>` (matching
   Japanese phrase, NO `--language`): TODAY panics; AFTER exit 0 (round-trip OK under wire Japanese).
3. **True-negative preserved:** `ms verify --phrase <english-12-word> <ja-ms1>` → NON-zero
   (`Bip39` parse-fail on Japanese-card words, or `VerifyPhraseMismatch`); assert NOT exit 0.
4. **Disagreement note (EXPLICIT flag):** `ms verify --language english <ja-ms1>` → exit 0 +
   stderr `note:` (explicit disagreement fires).
5. **NO spurious note when flag OMITTED (I-new guard):** `ms verify <ja-ms1>` (NO `--language`) →
   exit 0 AND stderr contains **NO** `note: this ms1 carries wordlist language …`. RED-fails on
   `origin/master` (today's `default_value="english"` makes omission look like explicit
   `--language english` → spurious note); GREEN only after Option-izing + `defaulted=true`.
6. **Round-trip label pin (I-1 verify `:95`):** `ms verify --phrase <ja-phrase> <ja-ms1>` (no
   `--language`) → exit 0 AND the `emit_round_trip_ok` success label shows **`japanese`** (wire),
   NOT `english`.
7. **EXIT-3 NO-REGRESSION (I-2 guard):** a valid future-format string decoding to
   `Err(ReservedTagNotEmittedInV01)` → `ms verify <reserved-tag-string>` still **exits 3** via
   `emit_future_format`. Model on the existing reserved-tag fixture if present
   (`verify_future_format.rs` exists — reuse its input), else construct per
   `ms_codec::Error::ReservedTagNotEmittedInV01`.
8. **Positive control:** `ms verify <english-ms1>` and `ms verify --phrase <english> <english-ms1>`
   → exit 0, unchanged.

**Drift check (round-3 review):** the only existing label-asserting verify test
(`verify_phrase_round_trip_ok.rs`, asserts `language=english` on an `entr`/English card) stays GREEN
— its Entr arm yields `effective_lang == cli_lang == English`, `:95` renders `"english"` unchanged.
`verify_phrase_round_trip_mismatch`, `verify_quiet_pass/fail`, `verify_future_format`,
`encode_pipe_to_verify` pass no `--language` → unaffected.

**Phase-3 gate:** full `cargo test -p ms-cli` + clippy.

---

## 6. Phase 4 — L26: `combine --to entropy` non-English advisory

**Goal:** `ms combine --to entropy` on non-English shares still prints the correct hex but emits a
stderr advisory (the language is dropped); English shares + the `--to phrase`/`--to ms1` arms do not.

**Edit A — port the advisory text into `crates/ms-cli/src/advisory.rs`** (alongside the existing
`secret_in_argv_warning` / `emit_output_class_advisory`):
```text
use crate::language::CliLanguage;   // add if not already imported

/// Stderr advisory when a non-English mnem secret is emitted as a
/// language-dropping form (raw entropy). English → no advisory (self-recovers
/// as the universal default). Ported from mnemonic-toolkit
/// non_english_seed_advisory (toolkit language.rs:176); uses CliLanguage::as_str()
/// (kebab) — byte-equivalent to toolkit human_name() except Chinese word-order
/// (cosmetic, ungated — M-2).
pub fn non_english_seed_advisory<W: Write>(stderr: &mut W, lang: CliLanguage, form: &str) {
    if lang == CliLanguage::English { return; }
    let name = lang.as_str();
    let _ = writeln!(stderr,
        "warning: encoding a {name} BIP-39 seed as {form} — it carries only the \
         entropy, not the wordlist language. Record \"{name}\" alongside the backup: \
         recovering the entropy with English-defaulted software derives a DIFFERENT \
         seed and a DIFFERENT wallet.");
}
```
(Toolkit returns `Option<String>`; the ms-cli flavor writes directly to stderr — matching the
existing `advisory.rs` `*_warning` idiom. Behavior is identical: English → no output.)

**Edit B — arm-selective wiring in `combine.rs::run`, `match args.to` at `:109-113`.** The
`language` binding is already resolved at `:95-108` before the match. Emit ONLY on the
`--to entropy` arm:
```text
match args.to {
    CombineTo::Phrase  => emit_phrase(&entropy, language, kind, args.json)?,   // re-renders in-language → no advisory
    CombineTo::Entropy => {
        emit_entropy(&entropy, kind, args.json)?;
        crate::advisory::non_english_seed_advisory(&mut std::io::stderr().lock(), language, "raw entropy");  // L26
    }
    CombineTo::Ms1     => emit_ms1(&payload, &entropy, kind, args.json)?,       // re-encodes mnem payload, language preserved → no advisory
}
```
- `--to phrase` re-renders words in-language → no loss → no advisory.
- `--to ms1` re-encodes the `mnem` payload (carries the language byte) → no loss → no advisory.
- Only `--to entropy` drops the language → advisory. **No `--json` wire-shape change** (advisory is
  stderr-only; `emit_entropy`'s `CombineJson.language: None` at `:165` is untouched).

> Confirm the `language` binding (`CliLanguage`, `:95`) is in scope at the `match args.to` — it is
> (`:95-108` precede `:109`). Pass it by value (`CliLanguage` is `Copy`).

**RED-first integration test — new `crates/ms-cli/tests/combine_entropy_language_advisory.rs`**
(model on `cli_combine.rs` `split_shares` helper):
1. **RED→GREEN:** split a Japanese phrase (`ms split --language japanese --phrase <ja> -k 2 -n 3
   --json`) → combine 2 shares `--to entropy` → stdout = correct hex (unchanged) AND **stderr
   contains the advisory** (`"warning: encoding a japanese BIP-39 seed as raw entropy"`).
   TODAY: no advisory.
2. **Arm-selective:** same Japanese shares `--to phrase` → stderr has NO such warning; `--to ms1`
   → stderr has NO such warning.
3. **`--to ms1` language-byte preservation (M-3):** combine the same Japanese shares `--to ms1` →
   take the re-emitted ms1 (stdout or `--json` `ms1` field) → `ms decode <that-ms1>` → asserts it
   decodes back to a `Mnem` payload carrying the **same Japanese language byte**
   (`language: japanese`). Pins that `emit_ms1` passes the ORIGINAL `&payload`.
4. **English control:** English shares `--to entropy` → NO advisory.
5. **`--json` unchanged:** `--to entropy --json` → stdout `language: null` (or absent), advisory
   still on stderr.

**Phase-4 gate:** full `cargo test -p ms-cli` + clippy.

---

## 7. Phase 5 — L5: sanitize `CliError`'s Debug

**Goal:** removing `#[derive(Debug)]` + a hand-rolled `Debug` so a future `{:?}` on `CliError`
NEVER echoes the secret ms1 (carried inside `codex32::Error::InvalidChecksum.string`).

**Edit (`error.rs`):**
- Remove `#[derive(Debug)]` at `:12` (keep `#[non_exhaustive]` at `:13`).
- Add a hand-rolled impl delegating to the already-sanitized `kind()` + `message()`:
  ```text
  // Hand-rolled Debug — NEVER prints the raw inner error. codex32::Error::
  // InvalidChecksum carries the secret ms1 `string`; the derived Debug would
  // leak it. kind() is a stable non-secret discriminant; message() is sanitized
  // (Codex32 → friendly_codex32, which drops InvalidChecksum.string).
  impl std::fmt::Debug for CliError {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          write!(f, "CliError::{} {{ {} }}", self.kind(), self.message())
      }
  }
  ```
- No other change: `CliError` keeps `Display` (`:118`), `std::error::Error` (`:124`),
  `exit_code`/`kind`/`message`/`details`; no `From<ms_codec::Error>` (`:132`) churn; no field
  changes. (Option (b) "stop carrying the raw `codex32::Error`" is REJECTED — higher blast radius,
  no extra safety once Debug is sanitized.)
- **Caveat to confirm in impl:** removing the derive must not break any `derive(Debug)`-dependent
  site. `CliError` is the top-level error; `Result<u8>` doesn't Debug it on the success path. Grep
  for `#[derive(Debug)]` on any struct embedding `CliError` (none expected). The hand-rolled Debug
  is still a `Debug` impl, so `assert_cmd`/`serde_json` test code that Debug-prints errors keeps
  compiling and is now SAFE.

**RED-first unit test (`#[cfg(test)] mod` in `error.rs`):**
```text
let e = CliError::Codex32(codex32::Error::InvalidChecksum {
    checksum: "long",
    string: "ms1secret_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".into(),
});
assert!(!format!("{:?}", e).contains("ms1secret_"));   // secret string absent
assert!( format!("{:?}", e).contains("Codex32"));      // sanitized kind present
assert!(!format!("{:?}", e).is_empty());               // informative
assert!(!format!("{}",  e).contains("ms1secret_"));    // Display already safe (pin)
```
- **OPTIONAL hardening (M-4, not blocking):** also assert
  `format!("{:?}", CliError::Codex32(codex32::Error::Field(...)))` (a non-`InvalidChecksum` arm)
  contains no input string — forward-looking guard against a future codex32 arm that echoes input.
  (Construct a valid `codex32::Error` variant that exists in `=0.1.0`; if `Field` isn't trivially
  constructible, use another current arm.)

**Phase-5 gate:** full `cargo test -p ms-cli` + clippy. This test RED-fails on `origin/master`
(today's derived Debug prints the whole `codex32::Error` incl. `string`).

---

## 8. Phase 6 — ship (version bump + CHANGELOG + publish + status flips)

Per `project_toolkit_release_ritual_version_sites` (ms-flavored). NO `cargo fmt` at any step.

1. **Bump `crates/ms-cli/Cargo.toml`** `version = "0.8.1"` → `"0.9.0"`. **Confirm the ms-codec
   path-dep pin stays** `ms-codec = { path = "../ms-codec", version = "=0.5.0" }`
   (Cargo.toml `:20`) — ms-codec is NO-BUMP and already on crates.io at `0.5.0`.
2. **Update `crates/ms-cli/CHANGELOG.md`** — new `0.9.0` section:
   - H4/H5 panic-fix + wire-language authority (non-English `derive`/`verify` now derive the
     correct fp instead of panicking).
   - verify's `--language` is now optional and advisory-only for mnem cards — the
     `[default: english]` help annotation is removed (I-new).
   - L26 combine→entropy non-English advisory (stderr).
   - L5 `CliError` Debug sanitize (no secret echo).
3. **Update root `CHANGELOG.md`** (a root `CHANGELOG.md` exists alongside `crates/ms-codec/CHANGELOG.md`;
   `crates/ms-cli/` has its own `README.md` but **confirm whether it has a `CHANGELOG.md`** at edit
   time — the `44ac71f` tree shows root + `crates/ms-codec/CHANGELOG.md`; if ms-cli's CHANGELOG
   lives only in the root, write the `0.9.0` section there). Add the ms-cli `0.9.0` entry wherever
   the canonical ms-cli changelog lives.
4. **README sweep:** check `crates/ms-cli/README.md` + root `README.md` for any pinned `0.8.1`
   self-reference / install line; bump to `0.9.0` if present.
5. **FULL `cargo test -p ms-cli`** (NOT targeted) + `cargo test -p ms-codec` (sanity — must be
   untouched) + `cargo clippy --all-targets -p ms-cli -- -D warnings`.
6. **Tag `ms-cli-v0.9.0`; `cargo publish -p ms-cli`** (ms-codec `0.5.0` already on crates.io →
   publish ms-cli directly).
7. **FOLLOWUP / report status ticks (in the shipping commit, per
   `feedback_followup_status_discipline`):**
   - The four slugs are NOT yet in ms `design/FOLLOWUPS.md` (verified at `44ac71f`). The canonical
     record is the toolkit bughunt report
     `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` — **tick the
     `- [ ]` → `- [x]` checkboxes** for: **H4** (`:139`), **H5** (`:159`) (the two the spec calls
     out explicitly), plus **L26** (`:1023`) and **L5** (`:342`). Cite `ms-cli-v0.9.0` in the tick
     line. (Optionally file the four slugs in ms `design/FOLLOWUPS.md` as `shipped` in the same
     commit; the spec §8 treats this as optional since the bughunt report is canonical.)
8. **Locksteps — ALL N/A (confirm negative):** GUI schema-mirror (no flag-name / dropdown-value
   change; the verify `--language` re-type keeps the flag-name + `CliLanguage` value-enum, removing
   only the `[default]` help annotation), manual `docs/manual/src/40-cli-reference/43-ms.md` (verify's
   `--language` line `:297` is already default-free), `--json` wire-shape (L26 is stderr-only), and
   sibling-codec FOLLOWUPS (all fixes self-contained in ms-cli). NO toolkit pin, NO toolkit bump
   (toolkit deps `ms-codec` LIBRARY, not `ms-cli` BINARY).

---

## 9. Mandatory post-impl gate — independent adversarial whole-diff review

Per CLAUDE.md "(4) post-implementation — a mandatory, non-deferrable independent adversarial
execution review over the whole diff." Runs AFTER P5, BEFORE tag/publish (P6 step 6). Persist
verbatim to `mnemonic-toolkit/design/agent-reports/cycle8-whole-diff-review.md`.

**Funds-focused review charge — the reviewer must adversarially confirm:**
1. **Non-English fingerprint is CORRECT** — a French ms1 derives `7d53dc37` (NOT `73c5da0a`); the
   wire byte (not `--language`) drives `from_entropy_in` on EVERY derive/verify path. No code path
   silently falls back to the English default for a `Mnem` payload.
2. **No label mis-thread** — all four derive label sites (`:231/:232/:245/:246-249/:251`) + verify
   `:95` read `effective_lang`/`effective_lang_defaulted`, NOT raw `cli_lang`/`defaulted`. No
   French card prints `language: english (DEFAULT)`; no bogus english-default note on a real-wire
   card. Entr cards still print `(DEFAULT)` + the english-default note (no regression).
3. **verify's exit-3 future-format leg is PRESERVED VERBATIM** — `Err(ReservedTagNotEmittedInV01)
   => emit_future_format` still lands exit 3; the helper is invoked ONLY on the `Ok((tag,payload))`
   arm. No whole-match swap. The generic `Err(e)` arm is unchanged.
4. **No spurious disagreement note** on a bare (no-flag) non-English verify (the Option-ization
   correctly computes `defaulted=true` on omission → no note).
5. **L5 — no secret leak** — `format!("{:?}", CliError::Codex32(InvalidChecksum{ string }))`
   does NOT contain the secret ms1 string; Debug delegates to the sanitized `kind()`+`message()`;
   no derive-Debug-dependent site broke.
6. **L26 stderr-only** — no `--json` wire-shape change; advisory fires only on `--to entropy` for
   non-English; `--to ms1` preserves the language byte.
7. **No drift** — full `cargo test -p ms-cli` GREEN incl. all pre-existing
   argv/output-class/help-pointer/gui-schema lint tests; clippy `-D warnings` clean; ms-codec
   untouched; mlock.rs not reformatted (g6 byte-share intact).

If the Agent-API dispatch fails mid-session, **flag it explicitly** and defer the formal review to
API recovery — never silently substitute inline self-review (CLAUDE.md (5)).

---

## 10. Phase summary + disjointness

| Phase | Scope | Files | New tests | RED reason |
|---|---|---|---|---|
| **P1** | shared `payload_entropy_and_language` helper | `cmd/payload_lang.rs` (new) + `cmd/mod.rs` (declare) | 5 helper unit tests (`#[cfg(test)] mod`) | pure-function correctness (no consumer yet) |
| **P2** | H4 `derive` consumes helper + label threading | `derive.rs` | `tests/derive_mnem_non_english.rs` | `unreachable!` at `derive.rs:185` panics on French/Japanese mnem |
| **P3** | H5 `verify` consumes helper (both legs) + Option-ize `--language` (I-new) | `verify.rs` | `tests/verify_mnem_non_english.rs` (incl. exit-3 no-regress + bare-no-flag no-spurious-note) | `unreachable!` at `verify.rs:64` panics; `default_value` fires spurious note |
| **P4** | L26 `non_english_seed_advisory` + arm-selective wiring | `advisory.rs`, `combine.rs` | `tests/combine_entropy_language_advisory.rs` | no advisory on non-English `--to entropy` today |
| **P5** | L5 hand-rolled `CliError` Debug | `error.rs` | unit test in `error.rs` `#[cfg(test)] mod` | derived Debug echoes the secret `string` |
| **P6** | ship: 0.9.0 bump + CHANGELOG + README sweep + FULL suite + tag + publish + bughunt-report ticks | `Cargo.toml`, `CHANGELOG.md`(s), `README.md`(s), bughunt report | — (gate = full suite GREEN) | — |

**Disjointness:** P1's helper is the only shared artifact; it is frozen after P1 (P2/P3 call it,
never edit it). P2/P3/P4/P5 each touch a disjoint consumer file. Each phase's RED tests fail for
exactly one reason. Phase order respects dependency: helper (P1) → its two consumers (P2, P3) →
independent advisory (P4) → independent error-hygiene (P5) → ship (P6).

Per-phase opus R0 review (full `cargo test -p ms-cli` in each) persisted verbatim to
`mnemonic-toolkit/design/agent-reports/cycle8-phase-N-<round>-review.md` BEFORE fold-and-commit.

---

## 11. MANDATORY R0 GATE (this plan-doc)

Per CLAUDE.md: **NO code before GREEN (0C/0I).** This plan-doc MUST pass an opus-architect R0
review and converge to 0 Critical / 0 Important BEFORE implementation begins (the brainstorm spec
already passed its R0 round-3 GREEN; the plan-doc carries its own gate). Fold findings → persist the
review verbatim to `mnemonic-toolkit/design/agent-reports/` → re-dispatch → repeat until GREEN; the
reviewer-loop continues after EVERY fold. Implementation is a single subagent per phase (TDD) off
ms `origin/master` `44ac71f`, followed by per-phase R0 reviews (full `cargo test -p ms-cli`) and the
non-deferrable whole-diff post-impl adversarial review (§9). Proceeding past any gate — start
coding, advance phase, tag, publish — with an open Critical or Important finding is prohibited.
