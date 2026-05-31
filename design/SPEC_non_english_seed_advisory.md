# SPEC — non-English BIP-39 wordlist-language advisory

**Branch:** `non-english-seed-advisory` (off `master` `9f11a31`)
**Crate:** `mnemonic-toolkit` (PATCH; next is 0.37.11 — confirm at ship).
**Source SHA:** `9f11a31` (all citations re-grepped against this tree).
**Origin:** path A of the `mnem` footgun (cycle-prep recon `mnemonic-secret/cycle-prep-recon-mnem-language-hint.md`). The wire fix (`mnem` payload kind) stays filed under `mnem-wordlist-language-hint-on-wire` (the ms-v0.2 arc); this is the **advisory-only** mitigation.

---

## §1. Problem

ms1 (and SeedQR / raw entropy) carry only the BIP-39 **entropy**, not the **wordlist language**. BIP-39 derives the seed via `PBKDF2(mnemonic_string, …)` where the mnemonic string is the language-specific words — so the **same entropy** in French vs English produces **different words → a different seed → a different wallet**. A non-English user who later recovers the entropy with English-defaulted third-party software silently derives the wrong wallet.

Our own `ms decode` already loud-annotates "DEFAULT" when `--language` is omitted (`mnemonic-secret/crates/ms-cli/src/cmd/decode.rs:43`). The gap is the **encode side** — the toolkit, which knows the language at encode time (`--language`), emits no warning. This SPEC adds a stderr advisory at every language-losing emit.

---

## §2. Source ground-truth (verified @ `9f11a31`)

- **`language.rs:10`** — `pub enum CliLanguage`; default = `English` (`:64` `default_is_english` test); `human_name()` (`:26`) → `"english"` etc. (need a non-English-comparison: `CliLanguage` must derive `PartialEq` — **verify at impl; add `#[derive(PartialEq)]` if absent**).
- **`cmd/bundle.rs`** — `pub language: Option<CliLanguage>` (`:42`); consumed at `:492`/`:598` (`language.unwrap_or_default()` in `resolve_slots`) and the unified-descriptor phrase/entropy arms (`:1267`/`:1339`/`:1583`). The synthesized `Bundle` has `any_secret_bearing()` (`synthesize.rs:35`) — true iff ≥1 slot emits a non-empty ms1.
- **`cmd/convert.rs`** — `pub language: Option<CliLanguage>` (`:245`); `language.unwrap_or_default()` (`:1111`) → `Mnemonic::parse_in(language, …)` (`:1130`). `enum ConvertTarget` (`:31`) variants incl. `Seedqr`, `Entropy` (language-dropping) vs `Phrase` (keeps language) and `Xpub/Xprv/Wif/Bip38/ElectrumPhrase/Address` (derived keys — the seed's language was already applied; no re-recovery ambiguity).
- **`cmd/seedqr.rs` + `seedqr.rs`** — **English-only**: `seedqr_encode` parses via `Mnemonic::parse_in(Language::English, …)` (`seedqr.rs:120/142/179`) + `Language::English.word_list()` (`:102/146`); module doc "English-wordlist" (`:4`). `SeedqrEncodeArgs` has **no `--language` flag** (`cmd/seedqr.rs:80-93`). → `mnemonic seedqr encode` REJECTS a non-English phrase; **no non-English standalone SeedQR is producible → NO advisory trigger here.** (Covered by `convert --to seedqr`.)
- **`ms encode` (ms-cli)** takes raw entropy with no language input → can't know the language → **toolkit-only**.

---

## §3. Design

### 3.1 The advisory message (single source of truth)

A pure helper — single message, unit-tested (precedent: the bip48 `bip48_nonstandard_script_type_warning` helper). Place in `language.rs` (or a small `advisory` module):

```rust
/// Returns a stderr advisory iff `lang` is a non-English BIP-39 wordlist (the
/// language is load-bearing for the seed but is NOT carried by `form`). `form`
/// names the language-dropping output ("an ms1 card", "a SeedQR", "raw entropy").
/// English → None (English self-recovers as the universal default).
pub(crate) fn non_english_seed_advisory(lang: CliLanguage, form: &str) -> Option<String> {
    if lang == CliLanguage::English {
        return None;
    }
    Some(format!(
        "warning: encoding a {} BIP-39 seed as {} — it carries only the entropy, \
         not the wordlist language. Record \"{}\" alongside the backup: recovering \
         the entropy with English-defaulted software derives a DIFFERENT seed and a \
         DIFFERENT wallet.",
        lang.human_name(), form, lang.human_name(),
    ))
}
```

### 3.2 Triggers (two live, one documented-moot)

| command | trigger | `form` | site |
|---|---|---|---|
| **`bundle`** | `args.language == Some(non-English)` **AND** synthesized `bundle.any_secret_bearing()` | `"an ms1 card"` | once, post-synthesis in the bundle run path (after the `Bundle` is built; both `args.language` and the bundle in scope) |
| **`convert`** | `args.language == Some(non-English)` **AND** `--to ∈ {Seedqr, Entropy}` | `"a SeedQR"` / `"raw entropy"` | once, in `convert` after the target is known |
| **`seedqr`** | — (English-only; no `--language`; rejects non-English input) | — | NONE — documented in §2; covered by `convert --to seedqr` |

- **English / absent `--language` → no advisory** (English is the universal default that self-recovers correctly).
- **Watch-only `bundle`** (xpub slots, no ms1) → `any_secret_bearing()` false → no advisory (no seed to mis-recover).
- **Emitted ONCE per invocation** (not per-slot — avoids N firings on multisig).
- **stderr only** — never on stdout / `--json` (so `--json` consumers are byte-unchanged).

### 3.3 Why `convert`'s key-deriving targets are excluded

`--to xpub/xprv/wif/bip38/address/electrum-phrase` are DERIVED from the seed (the language was already applied to produce the correct key) — they ARE the key, with no re-recovery step where a language must be re-guessed. Only `seedqr`/`entropy` are re-encodable seed-backup forms that drop the language. `--to phrase` keeps the language (it's a mnemonic in that language). So only `{Seedqr, Entropy}` trigger.

---

## §4. SemVer + lockstep

- **PATCH.** stderr-only behavior; **no new flag** (`--language` exists), **no `--json` wire change** → **no GUI schema-mirror, no manual lockstep** (matches the bip48 "bless+warn" stderr-only precedent [[feedback_silent_default_with_stderr_notice]]). No CHANGELOG `[Unreleased]` block in repo → add a `[0.37.11]` entry at ship per the release-doc invariant.
- Toolkit-only. No sibling/codec change. `mnem` wire FOLLOWUP unchanged (stays open under the v0.2 arc).

---

## §5. Test plan

1. **Helper unit tests** (`language.rs` test mod): `non_english_seed_advisory(English, _) == None`; `non_english_seed_advisory(French, "an ms1 card")` is `Some` containing `"french"` + `"DIFFERENT"` + the form.
2. **`bundle` integration:** `bundle --slot @0.phrase=<french 12-word> --language french …` → stderr contains the advisory; `--language english` (or absent, English phrase) → stderr does NOT; a **watch-only** `--language french` bundle (xpub slot) → does NOT (no ms1). Emitted exactly once for a 2-of-2 multisig (not twice). `--json` stdout byte-unchanged (advisory on stderr only).
3. **`convert` integration:** `convert --from phrase=<french> --language french --to seedqr` → advisory (form "a SeedQR"); `--to entropy` → advisory ("raw entropy"); `--to xprv`/`--to phrase` → NO advisory; `--language english` → NO advisory.
4. **`seedqr` non-regression:** `seedqr encode --from phrase=<french>` still errors (English-only, unchanged); no advisory machinery added there.
5. Full suite + `cargo clippy --all-targets -- -D warnings` (the toolkit's real gates; **no fmt CI gate** [[feedback_toolkit_has_no_fmt_gate_unlike_sibling_codecs]]).

---

## §6. Phases

**Phase 0 — helper + tests.** `non_english_seed_advisory` in `language.rs` (+ `#[derive(PartialEq)]` on `CliLanguage` if absent) + unit tests.

**Phase 1 — `bundle` site.** Emit once post-synthesis when non-English + `any_secret_bearing()`. Integration tests (incl. watch-only-no-fire + multisig-once + `--json`-stdout-unchanged).

**Phase 2 — `convert` site.** Emit when non-English + `--to ∈ {Seedqr, Entropy}`. Integration tests (incl. key-target-no-fire).

**Phase 3 — version + CHANGELOG + ship.** Bump (0.37.11), `[0.37.11]` CHANGELOG entry, full-suite + clippy gate, end-of-cycle opus R0 → GREEN, clean-tree → ff-merge `master` + push + tag.

Per the mandatory R0 gate: this SPEC + the plan-doc each get an opus R0 to 0C/0I before any code.
