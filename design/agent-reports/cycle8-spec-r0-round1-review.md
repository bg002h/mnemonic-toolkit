# Cycle-8 ms-cli spec — R0 review, ROUND 1

- **Artifact under review:** `design/BRAINSTORM_cycle8_mscli_panics.md` (cycle-8: H4 · H5 · L26 · L5)
- **Repo of record:** mnemonic-secret, `origin/master` = **`44ac71f`** (`44ac71fd6cb055e25e3c3dd1daca230d5d45bafb`) — ms-codec 0.5.0 / ms-cli 0.8.1
- **Reviewer:** opus software architect (adversarial R0 gate; NO code until 0C/0I)
- **Date:** 2026-06-21
- **Verification basis:** every citation re-checked against `origin/master` BYTES via `git -C /scratch/code/shibboleth/mnemonic-secret show origin/master:<path>` (local HEAD is `6b289186`, 2 behind + dirty + structurally older — NOT trusted). codex32 variant verified against `~/.cargo/.../codex32-0.1.0/src/lib.rs`. Toolkit advisory text verified against toolkit `origin/master`.

---

## Citation verification (all against `44ac71f`)

| Spec citation | Live truth | Verdict |
|---|---|---|
| H4 `derive.rs:185` `_ => unreachable!("…v0.1 decodes only Payload::Entr")` | line 185, exact text; match at `:184-185` inside ms1 `else` branch | **ACCURATE** |
| derive `cli_lang, defaulted` at `:165` | `:165 let (cli_lang, defaulted) = match args.language`; `:169 let lang: bip39::Language = cli_lang.into()` | **ACCURATE** |
| derive label sites `:231/:245/:251` | `:231 language: cli_lang.as_str()` (JSON); `:245 "language: … (DEFAULT)"`, `:251 "language: …"` (text), both `cli_lang.as_str()` | **ACCURATE** |
| H5 `verify.rs:64` `Ok((_, _)) => unreachable!("…only decodes to Payload::Entr")` | line 64, exact; match `:60-72` (also has `ReservedTagNotEmittedInV01` + `Err(e)` arms) | **ACCURATE** |
| verify `--phrase` `let lang = args.language.into()` | `:86 let lang: Language = args.language.into()`; `:87 parse_in`; `:88 from_entropy_in`; `:95 emit_round_trip_ok(…, args.language.as_str(), …)` | **ACCURATE** |
| L26 `combine.rs:157` `emit_entropy` | `:157 fn emit_entropy(entropy, kind, json)` — no `language` param; `--to entropy` arm at `:111`; `CombineJson.language: None` at `:165` | **ACCURATE** |
| L5 `error.rs:20` `Codex32(codex32::Error)` | `:20`, exact; `#[derive(Debug)]` `:12`; `#[non_exhaustive]` `:13`; `Display` `:118`; `message()`→`friendly_codex32` `:77`; `From` mapping `:136` | **ACCURATE** |
| parity template `decode.rs:63-89` | `:63 let (effective_lang, effective_lang_defaulted) = match &payload` … wire-wins + disagreement `note:` `:68-76`; second `match payload` for entropy `:83-86`; both end `≤:89` | **ACCURATE** |
| parity template `combine.rs:95-107` | `:95-107 match &payload` resolves wire language → binds `language`; **NO disagreement note** (combine has no `--language` arg) | **ACCURATE as a *resolution* template; INACCURATE as a *note/advisory* template** (see I-1) |
| `language.rs:34 from_code` + `as_str` | `:34 pub fn from_code(c: u8) -> Option<CliLanguage>` (0..=9 → Some, else None); `:51 as_str` kebab-lowercase | **ACCURATE** |
| oracle fps `cli_derive.rs` EN `73c5da0a` / FR `7d53dc37` | `:14 MASTER_FP_EN`, `:15 MASTER_FP_FR`; `:84-87` same `ZEROS_HEX` → `assert_ne!(EN, FR)` | **ACCURATE & LOAD-BEARING** |
| codex32 `InvalidChecksum { checksum, string }` echoes input | codex32-0.1.0 `lib.rs:58-62` `{ checksum: &'static str, string: String }`; `:165-167` `string: s` (full input) | **ACCURATE** |
| `friendly_codex32` drops `string` | `codex32_friendly.rs:27` `InvalidChecksum { checksum, .. }` — `string` dropped via `..` | **ACCURATE** |

**Sweep for omitted sibling panics:** I grepped EVERY `cmd/*.rs` for `unreachable!`/`panic!`/`.unwrap()` over `Payload`. Only `derive.rs:185` + `verify.rs:64` carry the buggy stale-v0.1 arm that sweeps `Mnem` into a panic. `decode.rs`/`combine.rs`/`split.rs` already handle `Mnem`; `inspect.rs`/`repair.rs` don't decode payloads. **The spec's H4+H5 set is COMPLETE — no missed panic site.** (Confidence: high.)

**Main error path (L5 latency):** `main.rs` is `fn main() -> ExitCode` (NOT `-> Result`, so no `Termination`-trait Debug-print), and `emit_error` (`:220`) uses `writeln!(stderr, "{}", e)` (Display) + `kind()`/`message()`/`details()` for JSON — never `{:?}`. **L5 is genuinely latent / not CLI-reachable today.** Confirmed.

---

## Critical

**None.**

The funds-correctness core is sound: the wire-language decision is the correct policy, it mirrors the *already-shipped* `decode.rs` behavior, and the oracle (`assert_ne!(73c5da0a, 7d53dc37)` for identical entropy) proves the `--language`-default patch would be funds-wrong. The spec correctly mandates the wire byte. No funds-incorrect path was found in the design.

---

## Important

### I-1 — `derive`/`verify` label sites are NOT threaded by the proposed §3 design (mislabeled-card risk + an internal contradiction)

**This is the one substantive gap.** §2(b) of the review charge asks whether `effective_lang` threads to BOTH the entropy path AND the label sites `:231/:245/:251`. The spec's *prose* (§3 intro, line ~"thread `effective_lang` into ALL downstream uses … AND the `language:` output labels at `:231/:245/:251`") **promises** this. But the spec's *concrete edit recipe* in §3.2 does NOT deliver it:

> §3.2: "call `payload_entropy_and_language(...)` → `(entropy, effective_lang)`, then `Mnemonic::from_entropy_in(effective_lang.into(), …)`. … `derive` already computes `(cli_lang, defaulted)` at `:165-169` — reuse it."

The label sites at `:231` (`language: cli_lang.as_str()`), `:245`/`:251` (text `language:` lines), and `:232/:244` (`language_defaulted: defaulted` / the `if defaulted` branch) all read `cli_lang` and `defaulted` — **NOT** `effective_lang`. If P2 only swaps the `from_entropy_in` language and "reuses `(cli_lang, defaulted)`" verbatim for the labels, then:

- A French ms1 derived with **no** `--language` → entropy/fp is CORRECT French (`7d53dc37`), but the card **prints `language: english (DEFAULT)`** and emits the bogus `:247` "note: --language defaulted to english …" — a **mislabeled card**. The fp is right (not funds-wrong), but the human-readable language label and the `language_defaulted` JSON field contradict the actual derivation. For a *language*-recording tool whose entire L26/derive thesis is "record the wordlist language alongside the backup," printing the WRONG language label is a real defect, not cosmetic polish.
- This also breaks the §7.1 test design: the disagreement-note test asserts stderr carries the helper's `note: … ignoring --language english` AND the positive control asserts the *correct* fp — but neither test pins the stdout `language:` label, so an implementer following §3.2 literally could ship the mislabeled card GREEN.

**This is an internal contradiction in the spec** (prose promises label threading; recipe + "reuse `(cli_lang, defaulted)`" omits it), and it lands on a user-facing correctness surface, so it is Important, not Minor.

**Required fix (spec):** make §3.2/§3.3 explicit and add tests:
1. State that for `derive`, the `:231` JSON `language` + `:245/:251` text `language:` + `:232` `language_defaulted` + the `:244-248 if defaulted` branch (including suppressing the `:247` "defaulted to english" note when the wire supplied a real language) MUST all use the helper's `(effective_lang, effective_lang_defaulted)` — NOT the raw `(cli_lang, defaulted)`. Have the helper return the `defaulted`-equivalent too (decode's template returns `(effective_lang, effective_lang_defaulted)` — `false` for a `Mnem` card — at `decode.rs:63`; mirror that 2-tuple, don't return a 1-tuple lang).
2. For `verify`, the success label at `:95 emit_round_trip_ok(…, args.language.as_str(), …)` must pass `effective_lang.as_str()` (the spec's §3.3 says this in prose — good — but lift it into the resolved-decisions/test matrix so it can't be dropped).
3. Add a test asserting the **stdout `language:` label == the wire language** (and `language_defaulted == false`) for a non-English-card derive with no `--language`, and that the `:247` english-default note is ABSENT. Without this assertion the mislabel ships GREEN.

(Note the helper signature in §3.1 returns `(Zeroizing<Vec<u8>>, CliLanguage)` — a 2-tuple WITHOUT the `defaulted` flag. decode's proven template returns the *defaulted* flag as well. Widen the signature to `(Zeroizing<Vec<u8>>, CliLanguage, bool)` or the label sites cannot correctly decide DEFAULT-vs-not.)

### I-2 — `verify`'s decode match is over a `Result`, not a bare `Payload`; the helper signature as specified does not slot in, and the `ReservedTagNotEmittedInV01` arm must be preserved

§3.3 says "replace the `Ok((_, _)) => unreachable!(…)` panic by extracting entropy via the helper." But `verify.rs:60-72` matches over `decoded: Result<(Tag, Payload), ms_codec::Error>` with FOUR arms: `Ok((_, Payload::Entr(b)))`, `Ok((_, _))` (the panic), `Err(ReservedTagNotEmittedInV01 { got })` (→ `emit_future_format`, exit-3 path), and `Err(e)`. The proposed helper takes a *`Payload` by value* (§3.1). To use it, P3 must first restructure: decode → keep the `Err(ReservedTagNotEmittedInV01)` and `Err(e)` arms exactly as-is, then for the `Ok((tag, payload))` case call `payload_entropy_and_language(payload, …)`. The spec does NOT call out that the `ReservedTagNotEmittedInV01` exit-3 leg and the generic `Err` leg MUST be preserved verbatim — a naive "swap the match for a helper call" could drop the exit-3 future-format path (a behavior regression: `verify` of a reserved-tag string would change exit code 3 → something else).

**Required fix (spec):** §3.3 must explicitly state: the helper is called ONLY on the `Ok((tag, payload))` arm; the `Err(ReservedTagNotEmittedInV01 { got }) => emit_future_format(...)` (exit-3) and `Err(e) => return Err(e.into())` arms are preserved byte-for-byte. Add a P3 positive-control test that a reserved-tag string still exits 3 (no regression). This is Important because a dropped exit-3 leg is a silent behavior break on a safety command, and the current spec text invites it.

---

## Minor

### M-1 — "`combine` already does exactly this" overclaims the disagreement-note parity
§3.4 and §4 repeatedly assert the wire-language + disagreement-`note:` policy is "**exactly** what `ms decode` AND `ms combine` already do (`decode.rs:63-89`, `combine.rs:95-107`)." Verified: `combine` has **no `--language` flag** (`CombineArgs` = `{shares, to, json}` only), so it resolves the wire language but **never emits a disagreement note** — there is nothing to disagree with. Only `decode` emits the note. The *resolution* template is shared by both; the *note* template is `decode`-only. Tighten the wording to "the disagreement-`note:` behavior matches `ms decode`; the bare wire-resolution matches both `decode` and `combine`." Does not change the design; prevents a future reader mis-modeling the helper on `combine` (which omits the note).

### M-2 — ported advisory text: `as_str()` ≠ `human_name()` for Chinese (word order differs)
§4.2's "Helper-name note" claims ms-cli `as_str()` is "byte-equivalent to the toolkit's `human_name()` for the languages that matter." Verified: toolkit `human_name()` = `simplified-chinese` / `traditional-chinese`; ms-cli `as_str()` = `chinese-simplified` / `chinese-traditional` (reversed word order). For Japanese/French/etc. they match; only Chinese differs. Since (a) the §7.3 L26 tests use Japanese, and (b) the spec correctly states this text is NOT under the byte-parity gate, this is cosmetic — but the spec's "byte-equivalent for the languages that matter" should be footnoted "(except Chinese, which reverses word order; not gated)" so the claim isn't literally false. No funds impact.

### M-3 — `emit_ms1` re-encodes with `Tag::ENTR`; confirm `--to ms1` truly preserves the mnem language byte
§4.3 asserts `--to ms1` "re-encodes the mnem payload, language preserved → no advisory." Verified at `combine.rs:178-180`: `emit_ms1` does `ms_codec::encode(Tag::ENTR, payload)` and `payload` is the original `&Payload` (still `Payload::Mnem{language,…}` for a mnem recovery), so the language byte IS carried into the re-encoded ms1. The claim holds. Flagging only so the implementer keeps passing the *original* `&payload` (not a reconstructed `Payload::Entr`) into `emit_ms1` — the no-advisory correctness of `--to ms1` depends on that. (Currently correct; a refactor must not regress it.) Add an assertion in the §7.3 test that `--to ms1` output decodes back to a `Mnem` with the same language byte, to pin this.

### M-4 — §7.4 L5 unit test should also assert `friendly_codex32`'s OTHER arms don't leak via the new Debug
The new Debug delegates to `message()` → `friendly_codex32`. Verified the `InvalidChecksum` arm drops `string`. The other arms (`Field(fe)`, `InvalidChar(c)`, `MismatchedHrp(a,b)`, etc.) print only structural/char data, never the full secret string — so Debug stays safe. This is fine, but the test in §7.4 only exercises `InvalidChecksum`. A one-line addition asserting Debug of a `Codex32(Field(...))` also contains no input string would harden against a future codex32 arm that starts echoing input. Optional; not blocking.

---

## Cross-charge verdicts (explicit)

**(a) Is the wire-language fix funds-correct + parity-accurate?**
**YES, funds-correct.** The wire `Mnem.language` byte is authoritative; `--language` advisory-only for mnem; this is the policy `decode.rs:63-89` already ships, and the in-repo oracle (`cli_derive.rs:84-87`, identical entropy → `73c5da0a` EN ≠ `7d53dc37` FR, `assert_ne!`) proves the naive `--language`-default patch would compute a wrong fingerprint. The §3.3 verify round-trip decision (parse the user's `--phrase` under the WIRE language) is funds-safe: verify's job is "does this phrase reproduce this card?", the card's language is ground truth, and a wrong-language phrase fails (`Bip39` parse → exit 1, or entropy-mismatch → exit 4) — never a false GREEN. The `--language english` + French-wire disagreement → advisory `note:` + proceed-with-wire is correct (wire wins → correct fp) and should NOT error (erroring would block a user from deriving their own non-English card just because they mistyped a flag; the wire is authoritative and the note informs them). **Parity is accurate for `decode`; mildly overstated for `combine`** (M-1: combine resolves wire language but emits no note — no `--language` arg). The `unreachable!()` is correctly retained ONLY as the `#[non_exhaustive]` guard; `from_code` covers 0..=9, the codec rejects ≥10 at decode, so no valid input reaches `_` or the `unwrap_or(English)` fallback — the panic is truly unreachable for valid input post-fix.

**(b) Does `effective_lang` thread to EVERY entropy AND label site?**
**Entropy path: YES** (§3.2/§3.3 route `from_entropy_in`/`parse_in` through the helper's lang — funds-correct). **Label sites: NO, as written** (I-1). The §3.2 recipe reuses raw `(cli_lang, defaulted)` for the `:231` JSON `language`, `:232` `language_defaulted`, and `:244-251` text labels/notes, contradicting the spec's own prose promise. Result with the literal recipe: correct fp but a **mislabeled card** (`language: english (DEFAULT)` + a bogus english-default note on a French card). The helper must also return the `defaulted`-equivalent (decode returns a 2-tuple `(effective_lang, effective_lang_defaulted)`; the spec's §3.1 helper returns only the lang). Verify's `:95` label is handled in prose but not pinned by a test. **Must fix before GREEN.**

**(c) Does the L5 Debug fix actually stop the leak?**
**YES.** `codex32::Error::InvalidChecksum { string }` carries the full input ms1 (codex32 `lib.rs:165-167`); the DERIVED Debug would print it; the production path never Debug-prints (verified `main.rs`/`emit_error` use Display + kind/message/details, and `main() -> ExitCode` ≠ `Result` so no `Termination` Debug) → genuinely **latent, not reachable today**. The hand-rolled Debug delegating to `kind()` (static discriminant) + `message()` (→ `friendly_codex32`, which drops `string` via `..`) cannot echo the secret. No other Debug/Display leak path exists: `details()` maps `Codex32` to `_ => None`; `friendly_codex32`'s other arms print only structural/char data. **Fix is sound** (one optional hardening, M-4).

---

## TDD integrity

RED-first design is genuine and non-vacuous: the H4/H5 RED tests build a **real non-English mnem ms1** (French/Japanese via `encode`) and feed it to `derive`/`verify` — which **panic today** (verified `unreachable!` live at `derive.rs:185` / `verify.rs:64`), so the test cannot pass on `origin/master`. The GREEN assertion `7d53dc37 ∧ ¬73c5da0a` is the funds-safety oracle and is the strongest possible single assertion. The existing `cli_derive.rs` oracle uses `--hex` (not a card), so the card-round-trip RED test is genuinely new coverage, not a duplicate. The L5 unit test asserts the secret substring is absent from `{:?}` — RED today (derived Debug echoes it), GREEN after. **One gap (I-1):** no test pins the stdout `language:` label/`language_defaulted` to the wire language, so the mislabeled-card defect would ship GREEN — add it. **One gap (I-2):** no positive-control that a reserved-tag verify still exits 3 — add it to guard the round-trip refactor.

---

## VERDICT

**R0 ROUND 1: 0C / 2I → RED**

- C: none.
- I-1: `derive`/`verify` label sites (`:231/:232/:245/:251`, verify `:95`) are not threaded by the §3.2 recipe (helper returns only lang, not the `defaulted` flag) → mislabeled card; spec prose contradicts its own recipe; add label-pinning tests.
- I-2: `verify`'s decode match is over a `Result` with an exit-3 `ReservedTagNotEmittedInV01` leg + generic `Err` leg the spec doesn't call out preserving; "replace the match with the helper" risks dropping the exit-3 future-format path; add a no-regression test.

Fold I-1 + I-2 (and, recommended, M-1/M-2/M-3), persist this review, re-dispatch round 2. **NO implementation until GREEN (0C/0I).**
