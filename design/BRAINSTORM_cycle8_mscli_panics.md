# BRAINSTORM — cycle-8: ms-cli robustness/advisory cluster (H4 · H5 · L26 · L5)

**Status:** DESIGN ONLY (no code). Feeds the mandatory opus-architect **R0 loop to 0C/0I** before any implementation.
**Cycle:** constellation bug-fix program, cycle-8.
**Repo of record:** **mnemonic-secret** (`ms-codec` / `ms-cli`). Default branch `master`.
**Author intent:** decision-complete, R0-ready, **no open questions** (see Resolved-decisions table §9).

---

## 0. Source-SHA table & stale-local-tree caveat

| Repo | Pin / SHA | Notes |
|---|---|---|
| **mnemonic-secret** `origin/master` | **`44ac71f`** (`44ac71fd6cb055e25e3c3dd1daca230d5d45bafb`) | ms-codec `0.5.0`, ms-cli `0.8.1`. **All citations below verified against these BYTES** via `git -C /scratch/code/shibboleth/mnemonic-secret show origin/master:<path>`. |
| mnemonic-toolkit `origin/master` | `8d2fe505` | Cited only for the ported advisory text + oracle context. NOT bumped by this cycle. |
| codex32 (crates.io dep) | `=0.1.0` (exact-pinned) | `Error::InvalidChecksum { checksum, string }` carries the secret string (L5 root). |

> ⚠ **STALE LOCAL TREE — DO NOT trust the checkout.** The mnemonic-secret working tree is **2 commits behind** `origin/master`, **dirty** (uncommitted edits to `crates/ms-cli/src/error.rs`, `crates/ms-codec/src/{error.rs,shares.rs}` + Cargo.toml/CHANGELOGs), and **structurally OLDER** — a FLAT `src/` with **no `cmd/` directory**. A naive `find`/`Read` of the checkout returns a misleading tree that lacks `src/cmd/{derive,verify,combine,decode}.rs`. **Branch implementation off `origin/master` `44ac71f`; stash/inspect the stray edits first; cite `git show origin/master:…` line numbers (this spec did).**

**Branch:** `feature/cycle8-mscli-panics-advisory` off `44ac71f`.

---

## 1. Finding summary — ALL 4 REPRODUCE at `44ac71f` (zero drift per cycle-prep recon)

| # | Class | Command | Site (`origin/master`) | One-line |
|---|---|---|---|---|
| **H4** | E-panic-dos | `ms derive` | `crates/ms-cli/src/cmd/derive.rs:185` | `_ => unreachable!("ms-codec v0.1 decodes only Payload::Entr")` panics on a valid non-English (mnem) ms1 — and would compute a WRONG fingerprint if naively patched with the `--language` default. |
| **H5** | E-panic-dos | `ms verify` | `crates/ms-cli/src/cmd/verify.rs:64` | `Ok((_, _)) => unreachable!("ms-codec v0.1 only decodes to Payload::Entr")` panics on a valid non-English ms1; the `--phrase` round-trip leg uses `args.language` (CLI flag) not the wire byte. |
| **L26** | B-policy-collapse (advisory gap) | `ms combine --to entropy` | `crates/ms-cli/src/cmd/combine.rs:157` (`emit_entropy`) | The recovered hex is correct, but the mnem wordlist-language is dropped with NO advisory → a language change between split and combine silently yields a different seed. |
| **L5** | D-secret-leak (latent) | `CliError` (all commands) | `crates/ms-cli/src/error.rs:20` | `#[derive(Debug)] enum CliError { … Codex32(codex32::Error) … }` carries the raw `codex32::Error`; `InvalidChecksum { string }` echoes the secret ms1 → any future `{:?}`/`unwrap`/`expect`/`panic` leaks it. Production uses the sanitized `Display`/`message()` → **latent, not yet live**. |

**Root-cause clustering:** H4 + H5 = ONE root cause ("stale v0.1 assumption: decode only yields `Entr`") at TWO edit-sites, sharing ONE funds-safety nuance (wire language vs `--language`). L26 is the advisory sibling of the same wire-language theme. L5 is unrelated (an error-type Debug hygiene bug) but co-shipped because it's ms-cli-only and tiny.

---

## 2. Authoritative-source verification (protocol facts)

These load-bearing facts were checked against primary source text + in-repo bytes, NOT plausibility (per the "external-protocol-facts" recon rule):

1. **BIP-39 seed derivation is wordlist-language-dependent.** The seed = `PBKDF2-HMAC-SHA512(password = NFKD(mnemonic sentence), salt = "mnemonic" ‖ passphrase, c = 2048, dkLen = 64)`. The **sentence** (the actual words) is the PBKDF2 password, and the words differ per wordlist. Therefore **the same entropy under two wordlists yields two different 64-byte seeds → two different master keys → two different wallets.** Empirical confirmation in-repo: `crates/ms-cli/tests/cli_derive.rs` carries `MASTER_FP_EN = 73c5da0a` and `MASTER_FP_FR = 7d53dc37` — the **same all-zeros 16-byte entropy** derives a DIFFERENT master fingerprint under English vs French. This is the funds-safety crux: **a panic-fix that rebuilds the mnemonic under the CLI `--language` default (English) computes a wrong fingerprint/xpub for a non-English seed.** The fix MUST use the wire language byte.
2. **codex32 / BIP-93 `mnem` payload carries the language.** ms-codec `Payload::Mnem { language: u8, entropy }` (`crates/ms-codec/src/payload.rs`), on-wire `[0x02][language_byte][entropy]`, `language` ∈ `0..=9` (validated by `Payload::validate` → `Error::MnemUnknownLanguage` for `≥10`). `ms_codec::decode()` returns `Payload::Mnem { language, entropy }` for any ms1 built from a non-English BIP-39 phrase (`crates/ms-codec/src/decode.rs:88-95`). **The wire language IS recoverable and is authoritative.**
3. **`Payload` is `#[non_exhaustive]`** (`payload.rs`). The `_ => unreachable!()` arm is the genuine forward-compat guard for a future v0.x variant — it must STAY as the catch-all, but `Mnem` must be a real handled arm, not swept into it.
4. **The 10 BIP-39 wordlists** map 1:1 from the wire code via `CliLanguage::from_code(c: u8) -> Option<CliLanguage>` (`crates/ms-cli/src/language.rs:34`): 0 english · 1 japanese · 2 korean · 3 spanish · 4 chinese-simplified · 5 chinese-traditional · 6 french · 7 italian · 8 czech · 9 portuguese. `from_code` returns `None` for `≥10` (codec already rejects those at decode, so a decoded `Mnem` always has `language ≤ 9` → `.unwrap_or(English)` is unreachable-but-safe, mirroring the in-repo idiom).
5. **`codex32::Error::InvalidChecksum { checksum: &'static str, string: String }`** (codex32-0.1.0 `src/lib.rs:57-63`) — the `string` field is the **full input ms1 string with the bad checksum** (= the secret-equivalent for an ms1; an attacker holding it can recover the entropy). The ms-cli sanitizer `friendly_codex32` (`crates/ms-cli/src/codex32_friendly.rs`) **deliberately drops `string`** in its `InvalidChecksum` arm — so `Display`/`message()` is safe, but the DERIVED `Debug` on `CliError` would print the whole `codex32::Error` including `string`. Confirmed via `From<ms_codec::Error> for CliError` mapping `Error::Codex32(c) => CliError::Codex32(c)` (`error.rs:132`).

> **Correction to a plausible-but-wrong framing:** the recon/report header in some places shorthands "ms-codec v0.4.4 returns `Mnem`". The CURRENT `origin/master` codec is **0.5.0** (the `unreachable!` comment still says "v0.1"). The substance (decode returns `Mnem`) is correct; the stale-comment string is just `v0.1` vs reality 0.5.0. No protocol fact changes — noted so R0 isn't tripped by the version string in the panic message.

---

## 3. H4 + H5 fix design — shared wire-language helper (funds-safety core)

### 3.1 The shared helper

Add ONE private helper so `derive`, `verify` (mirroring the existing `decode` idiom) recover `(entropy, effective_lang, effective_lang_defaulted)` from a `Payload` identically and the `#[non_exhaustive]` guard lives once. Proposed signature (in a shared module, e.g. `crates/ms-cli/src/cmd/mod.rs` or a new `crates/ms-cli/src/payload_lang.rs`):

```text
/// Recover (entropy, effective wordlist language, effective-language-defaulted)
/// from a decoded Payload. This is a 3-tuple, mirroring decode.rs's proven
/// 2-tuple for the language part (`(effective_lang, effective_lang_defaulted)`,
/// decode.rs:63) PLUS the entropy. The `effective_lang_defaulted` flag is what
/// the DEFAULT-vs-not label sites consume — without it the labels cannot decide
/// whether to print "(DEFAULT)" / emit the english-default note.
///
/// - Entr: language + defaulted are whatever the CALLER resolved from
///         --language/default → return (cli_lang, cli_lang_defaulted).
/// - Mnem: the WIRE language byte is AUTHORITATIVE (CliLanguage::from_code);
///         the CLI --language is advisory-only (see the disagreement note).
///         effective_lang_defaulted is FALSE (a real wire language exists; it is
///         never "defaulted"), exactly as decode.rs returns `false` for Mnem.
/// entropy is returned Zeroizing-wrapped (caller-wrap contract, payload.rs).
fn payload_entropy_and_language(
    payload: ms_codec::Payload,
    cli_lang: CliLanguage,     // resolved from --language (or English default)
    cli_lang_defaulted: bool,  // true when --language was NOT explicitly passed
    stderr: &mut impl std::io::Write,
) -> (Zeroizing<Vec<u8>>, CliLanguage, bool)  // (entropy, effective_lang, effective_lang_defaulted)
```

**Behavior (this is the proven `ms decode` policy — `crates/ms-cli/src/cmd/decode.rs:63-89` — lifted into a helper so `derive`/`verify` reach parity with `decode`, which already does exactly this; the 2-tuple `(effective_lang, effective_lang_defaulted)` at `decode.rs:63` is the template for the language part):**

- `Payload::Entr(b)` → `(Zeroizing::new(b), cli_lang, cli_lang_defaulted)`. (Entropy-only cards never carried a language; the CLI default governs, unchanged behavior — and the caller's `defaulted` flag passes through so the DEFAULT label is unchanged.)
- `Payload::Mnem { language: wire_code, entropy }` →
  - `let wire_cli = CliLanguage::from_code(wire_code).unwrap_or(CliLanguage::English);`
  - **wire wins** → return `(Zeroizing::new(entropy), wire_cli, false)`. (`effective_lang_defaulted = false`: a real wire language exists, so the label NEVER prints "(DEFAULT)" and NEVER emits the english-default note — mirroring `decode.rs:63`'s `false` for the `Mnem` arm.)
  - **advisory iff disagreement:** `if !cli_lang_defaulted && wire_cli != cli_lang { writeln!(stderr, "note: this ms1 carries wordlist language '{}'; ignoring --language {}", wire_cli.as_str(), cli_lang.as_str()).ok(); }` — byte-for-byte the `decode.rs` note string.
- `_ => unreachable!("ms-codec decode returned unknown Payload variant")` — STAYS as the `#[non_exhaustive]` guard. (Same wording as `decode.rs`.)

### 3.2 `derive` consumes it (H4)

`crates/ms-cli/src/cmd/derive.rs` ms1 branch (`:182-190`). Today:
```text
let (_tag, payload) = ms_codec::decode(&ms1)?;
let entropy: Zeroizing<Vec<u8>> = match payload {
    Payload::Entr(b) => Zeroizing::new(b),
    _ => unreachable!("ms-codec v0.1 decodes only Payload::Entr"),   // ← H4 PANIC
};
Mnemonic::from_entropy_in(lang, &entropy[..])…   // ← lang = CLI --language (WRONG for mnem)
```
After: call `payload_entropy_and_language(payload, cli_lang, defaulted, &mut stderr)` → `(entropy, effective_lang, effective_lang_defaulted)`, then `Mnemonic::from_entropy_in(effective_lang.into(), &entropy[..])`. The hex/`--phrase` source arms are untouched (they have no wire byte; `cli_lang`/`defaulted` govern, exactly as today). `derive` computes `(cli_lang, defaulted)` at `:165-169`; that pair is the INPUT to the helper, NOT what the downstream sites read.

**CRITICAL — thread `effective_lang`/`effective_lang_defaulted` into EVERY label site, NOT `cli_lang`/`defaulted` (I-1).** All four downstream label sites in `derive.rs` (live line numbers re-grepped against `44ac71f`) MUST read the helper's `(effective_lang, effective_lang_defaulted)`, not the raw `(cli_lang, defaulted)`:

| Site (`44ac71f`) | Today reads | MUST read |
|---|---|---|
| `:231` JSON `language: cli_lang.as_str()` | `cli_lang` | `effective_lang.as_str()` |
| `:232` JSON `language_defaulted: defaulted` | `defaulted` | `effective_lang_defaulted` |
| `:245` text `language: … (DEFAULT)` line + the `:246-249` `note: --language defaulted to english …` stderr note (both inside the `if defaulted` branch) | `defaulted` / `cli_lang` | `effective_lang_defaulted` / `effective_lang` (so a real-wire-language card prints the plain `language:` line and does NOT emit the bogus english-default note) |
| `:251` text `language: {cli_lang.as_str()}` (non-defaulted branch) | `cli_lang` | `effective_lang.as_str()` |

Without this, a French ms1 derived with no `--language` would compute the CORRECT fp `7d53dc37` but print **`language: english (DEFAULT)`** + a bogus `:246-249` english-default note — a **mislabeled card** that flatly contradicts the actual derivation. For a tool whose whole L26/derive thesis is "record the wordlist language alongside the backup," a wrong language label is a real defect, not cosmetic. (The Entr arm passes `cli_lang`/`defaulted` straight through, so entropy-card behavior is byte-unchanged.)

**Funds outcome:** a French ms1 now derives master fp `7d53dc37` (CORRECT, French) instead of panicking — and crucially NOT `73c5da0a` (which a naive `--language`-default patch would have produced) — AND prints `language: french` (NOT `english (DEFAULT)`).

### 3.3 `verify` consumes it (H5)

**PREREQUISITE — Option-ize verify's `--language` so it can compute `cli_lang_defaulted` (I-new).** The widened helper (§3.1) requires a `cli_lang_defaulted: bool`. But verify's arg is declared `#[arg(long, default_value = "english")] pub language: CliLanguage` (`verify.rs:30-31`) — a **non-`Option`** type, UNLIKE `decode.rs:29` and `derive.rs:62` which both use `Option<CliLanguage>`. With clap's `default_value`, an explicit `--language english` and an omitted flag **collapse into the identical value `CliLanguage::English`** — verify cannot distinguish them, so it cannot compute a true `defaulted`. **Resolution = Option A:** change `verify.rs:30-31` to `#[arg(long)] pub language: Option<CliLanguage>` (byte-identical to `decode.rs:29` / `derive.rs:62`), then resolve `(cli_lang, defaulted)` via the same idiom they use:
```text
let (cli_lang, defaulted) = match args.language {
    Some(l) => (l, false),
    None    => (CliLanguage::English, true),
};
```
This `(cli_lang, defaulted)` pair is the INPUT to `payload_entropy_and_language`. **Why Option A (not "document the asymmetry"):** without it, the helper's disagreement note fires on the WRONG condition — a bare `ms verify <ja-ms1>` (no flag) would, under `default_value="english"`, look like an explicit `--language english` and emit a **spurious** disagreement note on every non-English card, breaking the decode-parity thesis (decode/derive, being `Option`, correctly suppress the note when the flag is omitted). Option-izing restores exact decode-parity.

> **§6 surface-delta reconciliation (I-new).** Option-izing removes the `[default: english]` annotation from verify's clap `--help`. This is NOT a flag/subcommand/dropdown add/remove/rename → **GUI schema-mirror unaffected** (flag-name `--language` + the `CliLanguage` value enum are unchanged). The **manual mirror is also unaffected**: `docs/manual/src/40-cli-reference/43-ms.md:297` already documents verify's `--language` as "BIP-39 wordlist for `--phrase`" with **no `[default: english]` annotation** (verified against the toolkit manual) — so no manual line drifts. The §6 "no CLI-surface break" claim is refined: there is a help-text delta (the removed `[default]`), but it is manual-neutral and schema-neutral, hence no lockstep fires. (Contrast derive `:149` / decode `:102`, whose manuals DO annotate `default english` — because those args were already `Option` and the manual chose to surface the default in prose; verify's manual never did.)

`crates/ms-cli/src/cmd/verify.rs:60-72` decode match. **The match is over a `Result<(Tag, Payload), ms_codec::Error>`, NOT a bare `Payload` (I-2).** It has FOUR arms — `Ok((_, Payload::Entr(b)))`, the `Ok((_, _)) => unreachable!(…)` panic, `Err(ReservedTagNotEmittedInV01 { got }) => emit_future_format(…)` (the **exit-3 future-format leg**), and `Err(e) => return Err(e.into())`. The helper takes a `Payload` by value, so it CANNOT replace the whole match.

**DO NOT "replace the whole match with the helper" — that would silently drop the exit-3 future-format path (a behavior regression on a safety command).** Instead, restructure minimally:

- The two `Err` arms are **preserved VERBATIM**: `Err(ms_codec::Error::ReservedTagNotEmittedInV01 { got }) => { emit_future_format(&got, args.json)?; return Ok(0); }` (exit-3 leg) and `Err(e) => return Err(e.into())` are byte-for-byte unchanged.
- The helper is invoked **ONLY on the `Ok((tag, payload))` arm** — collapse the old `Ok((_, Payload::Entr(b)))` + `Ok((_, _)) => unreachable!` arms into a single `Ok((_tag, payload)) => payload_entropy_and_language(payload, …)` that yields `(entropy, effective_lang, effective_lang_defaulted)`. The `Mnem` payload now flows through the helper's `Mnem` arm instead of the panic.

Two consumer legs of the recovered `(entropy, effective_lang, effective_lang_defaulted)`:

- **No `--phrase` (quick exit-0):** decode succeeds → success-shaped output, exit 0. Just needs to NOT panic on `Mnem`; the helper supplies entropy + `effective_lang`.
- **`--phrase` round-trip (`:84-...`):** today `let lang: Language = args.language.into();` (CLI flag) builds BOTH the supplied and derived mnemonics. For a `Mnem` card this is wrong twice over (a Japanese ms1 verified against a Japanese `--phrase` with `args.language` defaulted to English fails the round-trip falsely; or a user who omits `--language` can't verify a non-English card at all). Fix: the **derived** side uses the `effective_lang` from the helper (the wire language); the **supplied** side parses under `effective_lang` too (the phrase the user typed is, by construction, the card's language — verifying a card means proving the phrase reproduces it). Concretely: `Mnemonic::from_entropy_in(effective_lang.into(), &entropy)` for the derived side, `Mnemonic::parse_in(effective_lang.into(), supplied.as_str())` for the supplied side. The success-log language label at `:95 emit_round_trip_ok(…, effective_lang.as_str(), …)` MUST pass `effective_lang.as_str()` (the wire language), NOT `args.language.as_str()` (I-1 verify label site). The disagreement `note:` is emitted once by the helper.

  *Decision rationale (R0):* parsing the user's `--phrase` under the WIRE language is funds-safe because verify's whole job is "does this phrase reproduce this card?" — the card's language is ground truth; if the user pasted a different-language phrase, `parse_in` fails (unknown words) → `CliError::Bip39` → exit 1 (a true negative), or if it parses but the entropy differs → `VerifyPhraseMismatch` exit 4. Neither is a false GREEN. There is no funds-safety path where honoring `--language` over the wire is more correct.

### 3.4 What happens to `--language` for mnem cards (DECISION)

**`--language` becomes advisory-only for `Mnem` payloads; the wire byte is authoritative; a disagreement prints a stderr `note:` and exits 0 (derive) / proceeds (verify).** This is NOT new policy. Two distinct parities, kept distinct (M-1):

- The **disagreement-`note:`** behavior mirrors **`ms decode`** specifically (`decode.rs:63-89`) — `decode` has a `--language` flag and emits the note when the wire byte disagrees. `derive`/`verify` join `decode` in this note-emitting parity.
- The bare **wire-language resolution** (wire byte → `CliLanguage::from_code`, wire wins) is shared by BOTH `decode` AND `combine` (`combine.rs:95-107`). But **`combine` has NO `--language` arg** (`CombineArgs = {shares, to, json}`), so it resolves the wire language and emits **no disagreement note** — there is nothing for a user flag to disagree with.

So the cycle brings the two outlier commands (`derive`, `verify`) into parity with **`decode`** (the note-emitting model); `combine` is wire-language-only. For `Entr` payloads `--language` governs unchanged (no wire byte exists). This symmetry is the strongest R0 posture: "make `derive`/`verify` consistent with `decode`, which already does it right," not "invent behavior."

**The `unreachable!()` becomes reachable-handled:** every valid `Payload::Mnem` (language 0..=9) now flows through the helper's `Mnem` arm. The `_ =>` arm survives ONLY as the `#[non_exhaustive]` future-variant guard (no current input reaches it). Confirmed: `from_code` covers 0..=9; codec rejects `≥10` at decode; so no live path hits `unwrap_or(English)`'s fallback or the `_ =>` arm.

---

## 4. L26 advisory design — `ms combine --to entropy` drops the language

### 4.1 Decision: **WARN, do not block.**

The recovered **entropy hex is correct** — `combine.rs:95-107` already resolves the wire language into the `language` binding; only `emit_entropy` (`:157`) fails to *surface* it. The wordlist language is **metadata the user needs to re-encode**, not a correctness failure of the hex. Blocking a correct output would be user-hostile and inconsistent with the rest of the constellation (the toolkit `MsSharesToShape::Entropy` path WARNS, never blocks — `toolkit src/ms_shares.rs`). **Recommendation: warn on stderr, exit 0.**

### 4.2 Shape

Port the toolkit's `non_english_seed_advisory` text (`mnemonic-toolkit src/language.rs:177`) into ms-cli's `advisory.rs` as a new helper:

```text
/// Stderr advisory when a non-English mnem secret is emitted as a
/// language-dropping form (raw entropy). English → None (self-recovers as
/// the universal default). Ported from mnemonic-toolkit non_english_seed_advisory.
pub fn non_english_seed_advisory<W: Write>(stderr: &mut W, lang: CliLanguage, form: &str) {
    if lang == CliLanguage::English { return; }
    let name = lang.as_str();   // ms-cli CliLanguage has as_str() (kebab); equivalent to toolkit human_name()
    let _ = writeln!(stderr,
        "warning: encoding a {name} BIP-39 seed as {form} — it carries only the \
         entropy, not the wordlist language. Record \"{name}\" alongside the backup: \
         recovering the entropy with English-defaulted software derives a DIFFERENT \
         seed and a DIFFERENT wallet.");
}
```

> **Helper-name note (M-2):** ms-cli `CliLanguage` exposes `as_str()` (kebab-lowercase: "japanese", "chinese-simplified") but NOT `human_name()`. Use `as_str()`. Its output is byte-equivalent to the toolkit's `human_name()` for every language EXCEPT Chinese, where the word order reverses: ms-cli `as_str()` = `chinese-simplified` / `chinese-traditional`, toolkit `human_name()` = `simplified-chinese` / `traditional-chinese`. This is cosmetic and not under any gate — (a) the §7.3 L26 tests use Japanese (unaffected), and (b) this advisory text is NOT under the `cli_output_class.rs` byte-parity gate (that gate covers only `emit_output_class_advisory`). Keep `as_str()` (Minor M-2; do NOT add a `human_name` alias). Keeping the wording aligned is good hygiene, not a gate.

### 4.3 Wiring (arm-selective)

In `combine.rs::run`, the `language` is already resolved before the `match args.to` (`:111-115`). Emit the advisory ONLY on the `--to entropy` arm:

```text
match args.to {
    CombineTo::Phrase  => emit_phrase(&entropy, language, kind, args.json)?,   // re-renders IN language → no advisory
    CombineTo::Entropy => {
        emit_entropy(&entropy, kind, args.json)?;
        non_english_seed_advisory(&mut std::io::stderr().lock(), language, "raw entropy");  // ← L26
    }
    CombineTo::Ms1     => emit_ms1(&payload, &entropy, kind, args.json)?,       // re-encodes mnem payload, language preserved → no advisory
}
```

- `--to phrase` re-renders the words in-language → no loss → no advisory.
- `--to ms1` re-encodes the `mnem` payload (carries the language byte) → no loss → no advisory.
- Only `--to entropy` drops the language → advisory. Mirrors the toolkit's arm-selective behavior exactly.

**No `--json` wire-shape change.** The advisory is **stderr-only**; `CombineJson` for the `--to entropy` arm already sets `language: None` (`:165-172`) and is untouched. (Per CLAUDE.md `--json` shape isn't schema-gated anyway — and here nothing changes for GUI consumers to self-update.)

---

## 5. L5 fix design — sanitize `CliError`'s Debug

### 5.1 Reachability confirmation

**Latent, not currently live.** The production error path is `Display`/`message()` → `friendly_codex32(e)`, whose `InvalidChecksum` arm drops `string` → safe today. The DERIVED `Debug` (`error.rs:12 #[derive(Debug)]`) is the leak vector: `Codex32(codex32::Error)` (`:20`) carries the raw error, and `codex32::Error::InvalidChecksum { string }` echoes the full ms1. **No live `{:?}`/`unwrap`/`expect`/`panic` formats a `CliError` today** (grep-verified across `origin/master` ms-cli: no `.unwrap()`/`format!("{:?}", err)` on the error path). The risk is a FUTURE caller (a test, a new command, a `.expect()` on `Result<u8>`) Debug-printing it. Minimal, defensive fix.

### 5.2 Decision: **option (a) — hand-roll `Debug` to delegate to the sanitized `Display`/`message()`.**

Lowest blast radius (Display is already sanitized; no field churn, no `From` mapping change, no exit-code/`kind`/`details` touch):

```text
// Replace `#[derive(Debug)]` on `enum CliError` with a hand-rolled impl that
// NEVER prints the raw inner error (codex32::Error::InvalidChecksum carries the
// secret ms1 string). Delegate to the sanitized message() so Debug == Display
// shape, no secret echo.
impl std::fmt::Debug for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // kind() is a stable non-secret discriminant; message() is sanitized
        // (Codex32 → friendly_codex32, which drops InvalidChecksum.string).
        write!(f, "CliError::{} {{ {} }}", self.kind(), self.message())
    }
}
```

- Removing `#[derive(Debug)]` requires nothing else: `CliError` already has `Display` (`:118`), `std::error::Error` (`:124`), `exit_code`/`kind`/`message`/`details`. No other derive depends on `Debug` of `CliError` (it's the top-level error; `Result<u8>` doesn't Debug it on the success path).
- **Caveat to verify in impl (flag for R0):** `serde_json`/`assert_cmd` test code occasionally Debug-prints errors. The hand-rolled Debug is *sanitized*, so any such use is now SAFE (and still informative — it prints `CliError::Codex32 { BCH checksum invalid … }`). No compile break (Debug is still implemented, just hand-rolled).
- Rejected option (b) "stop carrying the raw `codex32::Error`" — higher blast radius (changes the `From<ms_codec::Error>` mapping + every `friendly_codex32` call site + the `inspect_codex32_parse_failure.rs` test) for no extra safety once Debug is sanitized. Not chosen.

---

## 6. SemVer / publish / lockstep

| Crate | Action | Rationale |
|---|---|---|
| **ms-cli** | **MINOR `0.8.1` → `0.9.0`** + tag `ms-cli-v0.9.0` + `cargo publish -p ms-cli` | Pre-1.0 (`0.X` is the breaking axis per ms CHANGELOG). Behavior CHANGES for non-English `derive`/`verify` (panic→correct result; wire-language now honored), a new stderr advisory (L26), a Debug-shape change (L5), and verify's `--language` Option-ization (I-new) → MINOR. **No flag/subcommand/dropdown add/remove/rename** (the I-new change re-types verify's `--language` from `CliLanguage`+`default_value` to `Option<CliLanguage>`; flag-name + value-enum unchanged — only the `[default: english]` help annotation is removed; schema-mirror-neutral, manual-neutral — see lockstep checklist). |
| **ms-codec** | **NO-BUMP** | `decode()` already returns `Payload::Mnem`; `Payload` already `#[non_exhaustive]`; `from_code` already exists. Zero codec change. ms-cli already path-deps `ms-codec = "=0.5.0"` (on crates.io) → publish ms-cli `0.9.0` directly. |
| **mnemonic-toolkit** | **NO pin, NO bump** | Toolkit depends on **`ms-codec` (LIBRARY)**, NOT `ms-cli` (BINARY). An ms-cli-only MINOR does not enter the toolkit dependency graph. ms-codec NO-BUMP → toolkit `Cargo.lock` unchanged. |
| **mnemonic-gui** | **NO bump, NO schema-mirror** | No clap flag/subcommand/dropdown-value add/remove/rename (H4/H5 = match-arm + language sourcing; L26 = stderr line; L5 = Debug impl; I-new = re-type verify `--language` to `Option<CliLanguage>` — flag-name `--language` + `CliLanguage` value-enum BOTH unchanged, so schema-mirror's flag-name + dropdown-value gate does not fire). The ms-cli `gui-schema` SPEC §7 contract is unaffected. |

**Lockstep checklist (all NEGATIVE — confirmed):**
- GUI schema-mirror: **N/A** (no flag-name / dropdown-value change; the I-new `--language` re-type from `CliLanguage`+`default_value` to `Option<CliLanguage>` keeps the flag name + value enum identical, removing only the `[default]` help annotation — schema-mirror gates names + enums, NOT help-text defaults).
- Manual (`docs/manual/src/40-cli-reference/43-ms.md`): **N/A** — verify's `--language` is already documented at `:297` with NO `[default: english]` annotation, so removing it from clap `--help` (I-new) drifts no manual line. *Optional polish:* a one-line note about the new L26 advisory in the manual prose — NOT a gate.
- Sibling-codec `design/FOLLOWUPS.md` companion: **none** (all fixes self-contained in ms-cli; the new shared helper is internal).
- `--json` wire-shape: **unchanged** (L26 is stderr-only; `CombineJson.language: None` untouched).

**Publish ritual (per `project_toolkit_release_ritual_version_sites`, ms-flavored):**
1. Bump `crates/ms-cli/Cargo.toml` `version = "0.9.0"`.
2. Update ms-cli `CHANGELOG.md` (new `0.9.0` section: H4/H5 panic-fix + wire-language; verify `--language` is now optional and advisory-only for mnem cards — the `[default: english]` help annotation is removed (I-new); L26 advisory; L5 Debug sanitize).
3. Check ms-cli `README.md` for any pinned-version self-reference / install line (sweep for `0.8.1`).
4. Re-run the FULL `cargo test -p ms-cli` suite (NOT targeted targets — per `feedback_r0_review_run_full_package_suite`: CLI/language changes ripple into argv/output-class/help-pointer lint tests outside any one finding's scope). Then `cargo test -p ms-codec` (should be untouched, sanity).
5. Tag `ms-cli-v0.9.0`; `cargo publish -p ms-cli` (ms-codec 0.5.0 already on crates.io).

---

## 7. Tests (TDD — RED-first, per-phase)

**Test target (CRITICAL invocation):** ms-cli integration tests live at **`crates/ms-cli/tests/*.rs`** (`assert_cmd`-driven, one binary per file). The correct invocation is **`cargo test -p ms-cli`** (compiles + runs every `tests/*.rs` integration binary + any `#[cfg(test)]` unit mods in `src/`). NOT `cargo test --test <one>` for the gate (full-suite per the memory note). Existing fixtures to model on: `tests/cli_derive.rs` (oracle fps), `tests/decode_mnem_japanese.rs` + `tests/encode_mnem_japanese.rs` + `tests/cli_combine.rs` (Japanese mnem build pattern). Helper-unit tests (the new `payload_entropy_and_language`, the L5 Debug) go in a `#[cfg(test)] mod` beside the code.

### 7.1 H4 — `ms derive` (new `tests/derive_mnem_non_english.rs`)

Reuse the in-repo oracle: same all-zeros 16-byte entropy, English fp `73c5da0a`, French fp `7d53dc37` (both already in `cli_derive.rs`).

- **RED→GREEN (funds-safety core):** build a **French** mnem ms1 (`ms encode --language french --phrase <fr 12-word from [0;16]>` → ms1) → `ms derive <ms1>`.
  - TODAY: panics (`unreachable!`) → non-zero/abort.
  - AFTER: exit 0, stdout contains **`7d53dc37`** (CORRECT French fp) and **does NOT contain `73c5da0a`** (the wrong English fp a naive patch would emit). *This single assertion is the funds-safety proof.*
- **Japanese variant:** build a Japanese mnem ms1 (the `decode_mnem_japanese.rs` pattern, entropy `[0xAB;16]`) → `ms derive` exits 0, produces the Japanese-seed fp (independently derivable; assert == `ms derive --phrase <ja> --language japanese` fp for the same phrase — derive-from-card == derive-from-phrase parity).
- **`--language` disagreement note:** `ms derive --language english <french-ms1>` → exit 0, stdout still `7d53dc37`, **stderr contains the `note:` ignoring `--language english`** (wire wins).
- **LABEL PIN (I-1 — the mislabel-card guard):** `ms derive <french-ms1>` (NO `--language`) → exit 0, fp `7d53dc37`, AND **stdout text contains `language: french`** (NOT `english (DEFAULT)`), AND **stderr does NOT contain the `:246-249` `note: --language defaulted to english …` bogus default note**. JSON variant (`--json`): `language == "french"` AND `language_defaulted == false`. Without this assertion an implementer who threaded only the entropy (leaving the labels on `cli_lang`/`defaulted`) ships the mislabeled card GREEN.
- **Positive control (no regression):** `ms derive <english-entr-ms1>` (an entropy-only `Entr` card, e.g. from `ms encode --hex <zeros>` with no language byte) with no `--language` → exit 0, fp `73c5da0a`, stdout `language: english (DEFAULT)` + the english-default note on stderr (the `Entr` arm passes `cli_lang`/`defaulted` through unchanged — DEFAULT label preserved), **no** disagreement `note:` on stderr. *Contrast pin:* an english-`Mnem` card (built from an english phrase via `encode`) prints `language: english` WITHOUT `(DEFAULT)` and no default note (its `effective_lang_defaulted == false`), confirming the Entr-vs-Mnem label split.

### 7.2 H5 — `ms verify` (new `tests/verify_mnem_non_english.rs`)

- **RED→GREEN:** Japanese mnem ms1 → `ms verify <ms1>` (no `--phrase`): TODAY panics; AFTER exit 0 (valid card).
- **`--phrase` round-trip (wire language honored):** `ms verify --phrase <ja phrase> <ja-ms1>` (the matching Japanese phrase, NO `--language`): TODAY would panic on decode (or, past the panic, fail under English default); AFTER exit 0 (round-trip OK under the wire's Japanese).
- **True-negative preserved:** `ms verify --phrase <english-12-word> <ja-ms1>` → NON-zero (`Bip39` parse-fail on Japanese-card words, OR `VerifyPhraseMismatch`); assert it is NOT exit 0 (no false GREEN).
- **`--language` disagreement note (EXPLICIT flag):** `ms verify --language english <ja-ms1>` → exit 0 + stderr `note:` (user explicitly disagreed → note fires).
- **NO spurious note when flag OMITTED (I-new — the Option-ization guard):** `ms verify <ja-ms1>` (NO `--language` at all) → exit 0 AND stderr contains **NO** `note: this ms1 carries wordlist language …` disagreement note. This RED-fails if verify's `--language` stays a non-`Option` `default_value="english"` (omission would look like explicit `--language english` and fire the spurious note); it GREENs only after Option-izing the arg + computing `defaulted=true` for omission. Pins decode-parity (decode/derive suppress the note on omission).
- **Round-trip label pin (I-1 verify `:95`):** `ms verify --phrase <ja phrase> <ja-ms1>` (no `--language`) → exit 0 AND the `emit_round_trip_ok` success label shows the **wire language** (`japanese`), NOT `english`. (Pins that `:95` passes `effective_lang.as_str()`, not `args.language.as_str()`.)
- **EXIT-3 NO-REGRESSION (I-2 — guards the round-trip refactor):** `ms verify <reserved-tag-string>` (a valid future-format string that decodes to `Err(ReservedTagNotEmittedInV01)`) still **exits 3** via `emit_future_format`, unchanged. This pins that restructuring the decode match (Ok-arm → helper) did NOT drop the exit-3 future-format leg. (Model on the existing reserved-tag fixture if present, else construct the reserved-tag input per `ms_codec::Error::ReservedTagNotEmittedInV01`.)
- **Positive control:** `ms verify <english-ms1>` and `ms verify --phrase <english> <english-ms1>` → exit 0, unchanged.

### 7.3 L26 — `ms combine --to entropy` advisory (new `tests/combine_entropy_language_advisory.rs`)

Use the `cli_combine.rs` `split_shares` helper pattern.

- **RED→GREEN:** split a **Japanese** phrase (`ms split --language japanese --phrase <ja> -k 2 -n 3 --json`) → combine 2 shares `--to entropy` → stdout = correct hex (unchanged) AND **stderr contains the advisory** ("warning: encoding a japanese BIP-39 seed as raw entropy …"). TODAY: no advisory.
- **Arm-selective (no advisory where language is preserved):** same Japanese shares `--to phrase` → stderr has NO such warning; `--to ms1` → stderr has NO such warning.
- **`--to ms1` language-byte preservation (M-3):** combine the same Japanese shares `--to ms1` → take the re-emitted ms1 from stdout (or `--json` `ms1` field) and `ms decode <that-ms1>` → assert it decodes back to a `Mnem` payload carrying the **same Japanese language byte** (`language: japanese`). Pins that `emit_ms1` passes the ORIGINAL `&payload` (still `Payload::Mnem{language,…}`) into `ms_codec::encode`, so a future refactor that reconstructs a `Payload::Entr` would RED this test and the missing-advisory correctness of the `--to ms1` arm cannot silently regress.
- **English control:** English shares `--to entropy` → NO advisory (English self-recovers).
- **`--json` unchanged:** `--to entropy --json` → stdout `language: null` (or absent), advisory still on stderr (stderr ≠ json wire-shape).

### 7.4 L5 — `CliError` Debug no-echo (unit test in `src/error.rs` `#[cfg(test)] mod`)

- `let e = CliError::Codex32(codex32::Error::InvalidChecksum { checksum: "long", string: "ms1secret_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".into() });`
- assert `!format!("{:?}", e).contains("ms1secret_")` (the secret substring) AND `!format!("{:?}", e).contains("InvalidChecksum")` is NOT required — just assert the **secret `string` value does not appear**.
- assert `format!("{:?}", e)` still contains the sanitized `kind()` ("Codex32") + a non-empty message (informative, no leak).
- regression sibling: assert `format!("{}", e)` (Display) also does not contain the secret (already true — pins it).
- **OPTIONAL hardening (M-4, not blocking):** also assert that `format!("{:?}", CliError::Codex32(codex32::Error::Field(...)))` (a non-`InvalidChecksum` codex32 arm) contains no input string — guards against a FUTURE codex32 arm that starts echoing the input. The other current arms (`Field`, `InvalidChar`, `MismatchedHrp`, …) print only structural/char data, never the full secret, so the new sanitized Debug is already safe; this is forward-looking hardening only.

---

## 8. FOLLOWUP slugs

No NEW open FOLLOWUPs are required (all 4 fully resolved this cycle). Status flips to record in the shipping commit (per `feedback_followup_status_discipline` — verify "open" at decision time, flip in the shipping commit):

| Slug (ms repo `design/FOLLOWUPS.md`) | Action |
|---|---|
| `ms-derive-mnem-payload-panic` (H4) | flip **open → shipped** (cite `ms-cli-v0.9.0`) |
| `ms-verify-mnem-payload-panic` (H5) | flip **open → shipped** |
| `ms-cli-combine-entropy-non-english-advisory-gap` (L26) | flip **open → shipped** (file if not yet present, then flip in same commit) |
| `ms-cli-clierror-codex32-bypasses-sanitized-debug` (L5) | flip **open → shipped** |

*(If a slug isn't yet in ms `design/FOLLOWUPS.md`, the bughunt report `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` is the canonical source; the shipping commit ticks the `- [ ]` checkbox there.)*

Optional polish (NOT a gate, file as `open` if desired): `ms-cli-manual-l26-advisory-prose` — add a sentence to `docs/manual/src/40-cli-reference/43-ms.md` about the new combine→entropy advisory.

---

## 9. Resolved decisions (NO open questions)

| # | Decision point | RESOLUTION | Why (funds-safe / parity) |
|---|---|---|---|
| D1 | H4/H5 — wire byte vs `--language` for mnem | **Wire `Mnem.language` byte is authoritative** via `CliLanguage::from_code`; `--language` advisory-only for mnem. | BIP-39 seed = PBKDF2 over the language-specific sentence; `--language` default (English) gives a WRONG fp for a non-English seed (oracle: French `7d53dc37` ≠ English `73c5da0a`). |
| D2 | H4/H5 — shared helper vs duplicate | **One shared `payload_entropy_and_language` helper** returning the **3-tuple `(Zeroizing<Vec<u8>>, CliLanguage, bool)` = `(entropy, effective_lang, effective_lang_defaulted)`** (template = `decode.rs:63-89`, whose language part is the 2-tuple `(effective_lang, effective_lang_defaulted)` at `:63`). `Mnem` → `effective_lang_defaulted = false`; `Entr` → pass `cli_lang`/`cli_lang_defaulted` through. | Single `#[non_exhaustive]` guard; `derive`/`verify` reach parity with `decode`. The `defaulted` flag is REQUIRED for the label sites to decide DEFAULT-vs-not (I-1). |
| D2a | H4/H5 — label sites threaded? | **EVERY label site reads `effective_lang`/`effective_lang_defaulted`, NOT raw `cli_lang`/`defaulted`:** derive `:231` JSON `language`, `:232` `language_defaulted`, `:245` DEFAULT line + `:246-249` english-default note, `:251` text `language:`; verify `:95` `emit_round_trip_ok` language label. | Threading only the entropy (leaving labels on `cli_lang`/`defaulted`) yields a correct fp but a MISLABELED card (`language: english (DEFAULT)` + bogus default note on a French card). I-1 fix; pinned by a stdout/JSON label test (§7.1/§7.2). |
| D3 | H4/H5 — the `--language` disagreement behavior | **Wire wins, stderr `note:`, exit 0 (derive) / proceed (verify).** Byte-identical to **`decode.rs`** note string. | `decode` is the note-emitting parity model (it has a `--language` flag). `combine` shares only the bare wire-resolution (NO `--language` arg → NO note) (M-1). No new behavior invented. |
| D4 | H4/H5 — `unreachable!()` fate | **`Mnem` becomes a real handled arm; `_ =>` stays as the `#[non_exhaustive]` future-variant guard only.** | `from_code` covers 0..=9; codec rejects ≥10 at decode → no live path hits `_`. |
| D5 | H5 `--phrase` round-trip language | **Both supplied + derived mnemonics parsed/built under the WIRE language.** | Verify's job is "phrase reproduces card?"; the card's language is ground truth; mismatch → true negative (Bip39 / PhraseMismatch), never a false GREEN. |
| D5a | H5 — verify decode match (over `Result`, not `Payload`) | **Helper called ONLY on the `Ok((tag, payload))` arm; the `Err(ReservedTagNotEmittedInV01) => emit_future_format` (exit-3) arm + the generic `Err(e)` arm are PRESERVED VERBATIM.** Do NOT "replace the whole match with the helper." | The match is over `Result<(Tag, Payload), Error>` with an exit-3 future-format leg; a naive whole-match swap drops the exit-3 path (silent safety-command regression). I-2 fix; pinned by an exit-3 no-regression test (§7.2). |
| D5b | H5 — verify `--language` arg type (`cli_lang_defaulted` source) | **Option A: re-type `verify.rs:30-31` from `CliLanguage`+`default_value="english"` to `Option<CliLanguage>`** (matching `decode.rs:29` / `derive.rs:62`); compute `(cli_lang, defaulted)` via `match args.language { Some(l)=>(l,false), None=>(English,true) }`; pass `defaulted` into the helper. | The widened 3-tuple helper requires `cli_lang_defaulted`; with `default_value`, explicit `--language english` and omission collapse to the same value → verify cannot compute it → a bare `ms verify <non-english-ms1>` would emit a SPURIOUS disagreement note, breaking decode-parity. I-new fix. Surface delta = removed `[default]` help annotation only: schema-mirror-neutral (flag-name + value-enum unchanged) + manual-neutral (`43-ms.md:297` already default-free). Pinned by the bare-no-flag no-spurious-note test (§7.2). |
| D6 | L26 — block vs warn | **WARN, exit 0.** | The entropy hex is correct; language is re-encode metadata. Mirrors the toolkit's warn-not-block. |
| D7 | L26 — which arms warn | **Only `--to entropy`.** `--to phrase` (re-renders in-language) + `--to ms1` (re-encodes mnem payload) preserve language → no advisory. | Arm-selective, matches toolkit. |
| D8 | L26 — advisory text source | **Port toolkit `non_english_seed_advisory` into ms-cli `advisory.rs`, using `CliLanguage::as_str()`.** | Reuse proven wording; `as_str()` == toolkit `human_name()` for every language EXCEPT Chinese (word order reverses; cosmetic, NOT under the byte-parity gate — M-2). §7.3 tests use Japanese (unaffected). |
| D9 | L5 — fix approach | **Hand-roll `CliError` `Debug` delegating to sanitized `kind()`+`message()`; remove `#[derive(Debug)]`.** | Lowest blast radius; Display already sanitized; no `From`/exit-code/details churn. Option (b) rejected. |
| D10 | L5 — reachability | **Latent (no live `{:?}` site); fix anyway (defensive).** | A future Debug-print would leak the ms1 secret; trivial to preclude now. |
| D11 | SemVer | **ms-cli MINOR 0.9.0; ms-codec NO-BUMP; toolkit/GUI untouched.** | Behavior change, no surface break; toolkit deps ms-codec (lib) not ms-cli (bin). |
| D12 | Publish | **tag `ms-cli-v0.9.0` + `cargo publish -p ms-cli`** (ms-codec 0.5.0 already on crates.io). | Registry crate; full `cargo test -p ms-cli` before tag. |
| D13 | Locksteps | **None** (no GUI schema-mirror, no manual gate, no sibling FOLLOWUP). | No flag-name / `--json` / dropdown-value change. The I-new verify `--language` re-type removes only the `[default]` help annotation — schema-mirror gates names+enums (not help defaults), and the manual's verify `--language` line is already default-free → no lockstep fires. |
| D14 | Branch base | **`feature/cycle8-mscli-panics-advisory` off `origin/master` `44ac71f`** — NOT the dirty local tree. | Local checkout is 2-behind, dirty, structurally older (flat `src/`, no `cmd/`). |
| D15 | Test invocation | **`cargo test -p ms-cli` (FULL package suite), per-phase.** | Targeted `--test` targets miss argv/output-class/help-pointer lint ripple (memory: full-suite-required). |

---

## 10. Phasing (single-subagent-per-phase TDD, per CLAUDE.md)

| Phase | Scope | LOC (incl. tests) |
|---|---|---|
| P1 | Shared `payload_entropy_and_language` helper + unit tests | ~40-60 |
| P2 | H4 `derive` consumes helper + `tests/derive_mnem_non_english.rs` (RED→GREEN, French/Japanese oracle) | ~30-50 |
| P3 | H5 `verify` consumes helper (both legs) + Option-ize `--language` → `Option<CliLanguage>` (I-new) + `tests/verify_mnem_non_english.rs` (incl. exit-3 no-regression + bare-no-flag no-spurious-note) | ~35-55 |
| P4 | L26 `non_english_seed_advisory` in `advisory.rs` + arm-selective wiring + `tests/combine_entropy_language_advisory.rs` | ~30-50 |
| P5 | L5 hand-rolled `CliError` Debug + `error.rs` no-echo unit test | ~20-35 |
| P6 | Version bump 0.9.0 + CHANGELOG + README sweep + FULL `cargo test -p ms-cli` + `cargo test -p ms-codec` sanity; FOLLOWUP status flips staged | — |

Per-phase opus R0 review (full `cargo test -p ms-cli` in each review) persisted verbatim to `mnemonic-toolkit/design/agent-reports/cycle8-phase-N-<round>-review.md`. Mandatory whole-diff post-impl adversarial review before tag/publish.

---

## 11. MANDATORY R0 GATE

> Per CLAUDE.md: **NO code before GREEN (0C/0I).** This brainstorm spec MUST pass an opus-architect **R0 review** and converge to **0 Critical / 0 Important** BEFORE any implementation (and again as a plan-doc). Fold findings → persist the review verbatim to `mnemonic-toolkit/design/agent-reports/` → re-dispatch → repeat until GREEN. The reviewer-loop continues after EVERY fold (a fold can introduce drift). Proceeding past this gate — or any per-phase gate, or tag/publish — with an open Critical or Important finding is **prohibited**. Implementation branches off ms `origin/master` `44ac71f` and is a single subagent per phase (TDD), followed by a mandatory non-deferrable whole-diff adversarial execution review.
