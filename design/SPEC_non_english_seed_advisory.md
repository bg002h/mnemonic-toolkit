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

- **`language.rs:10`** — `pub enum CliLanguage`; **already derives `PartialEq`** (`:8` `#[derive(… PartialEq …)]`, so `lang == CliLanguage::English` compiles as-is); default = `English` (`:64` `default_is_english` test); `human_name()` (`:26`) → lowercase/kebab names (`"english"`, `"simplified-chinese"`, `:29`).
- **`cmd/bundle.rs`** — `pub language: Option<CliLanguage>` (`:42`); consumed at `:492`/`:598` (`language.unwrap_or_default()` in `resolve_slots`) and the unified-descriptor phrase/entropy arms (`:1267`/`:1339`/`:1583`). The synthesized `Bundle` has `any_secret_bearing()` (`synthesize.rs:35`) — true iff ≥1 slot emits a non-empty ms1.
- **`cmd/convert.rs`** — `pub language: Option<CliLanguage>` (`:245`); `language.unwrap_or_default()` (`:1111`) → `Mnemonic::parse_in(language, …)` (`:1130`). `--to` is `pub to: Vec<String>` (`:226`) parsed into `targets: Vec<NodeType>` (`:850`) — **multi-target allowed** (`--to xpub,entropy`); the type is `enum NodeType` (`:31`), NOT `ConvertTarget` (no such type). **`--to seedqr` is REFUSED at parse** (input-only node; `:867-871` comment + `Seedqr => unreachable!` `:1207`). The only language-dropping LIVE target is `Entropy` (`Phrase` keeps the language; `Xpub/Xprv/Wif/Bip38/ElectrumPhrase/Address` are derived keys — the seed's language was already applied, no re-recovery ambiguity).
- **`cmd/seedqr.rs` + `seedqr.rs`** — **English-only**: `seedqr_encode` parses via `Mnemonic::parse_in(Language::English, …)` (`seedqr.rs:120/142/179`) + `Language::English.word_list()` (`:102/146`); module doc "English only" (`:11`). `SeedqrEncodeArgs` has **no `--language` flag** (`cmd/seedqr.rs:80-98`). → `mnemonic seedqr encode` REJECTS a non-English phrase. Combined with the `convert --to seedqr` refusal above: **NO path in the toolkit produces a non-English SeedQR.** The SeedQR footgun does not exist → no advisory anywhere for SeedQR.
- **`cmd/slip39.rs`** — BOTH subcommands carry `--language` (`pub language: CliLanguage`, default `english`): **split** (`:132`, `--from phrase=/entropy=` → SLIP-39 shares) and **combine** (`:171`, `--to Slip39ToShape` `:168`, where `Slip39ToShape::Entropy` `:185`). split→shares and combine→entropy are both language-losing emits with `--language` in hand → advisory sites (I1).
- **`seed-xor`** shares are BIP-39 phrases (carry the language) — NOT a language-losing emit; out of scope.
- **`ms encode` (ms-cli)** takes raw entropy with no language input → can't know the language → **toolkit-only**.

---

## §3. Design

### 3.1 The advisory message (single source of truth)

A pure helper — single message, unit-tested (precedent: the bip48 `bip48_nonstandard_script_type_warning` helper). Place in `language.rs` (or a small `advisory` module):

```rust
/// Returns a stderr advisory iff `lang` is a non-English BIP-39 wordlist (the
/// language is load-bearing for the seed but is NOT carried by `form`). `form`
/// names the language-dropping output ("an ms1 card", "raw entropy", "SLIP-39 shares").
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

### 3.2 Triggers (four live sites; SeedQR genuinely impossible)

Resolve each site's language (bundle/convert: `Option` → `unwrap_or_default()`; slip39: direct `CliLanguage`, default english), then call the helper — it no-ops on English, so each site calls it with the resolved language + its form; the *gate* below decides whether a language-losing emit even happens.

| command | gate (when a language-losing emit occurs) | `form` | site (once per invocation) |
|---|---|---|---|
| **`bundle`** | synthesized `bundle.any_secret_bearing()` (>=1 ms1) | `"an ms1 card"` | `emit_unified` (`bundle.rs:698`) — all 3 dispatch branches converge here |
| **`convert`** | `targets.contains(&NodeType::Entropy)` (`convert.rs:850/874`) | `"raw entropy"` | once after target-parse |
| **`slip39 split`** | always (split -> shares lose the BIP-39 language) | `"SLIP-39 shares"` | once in the split run path |
| **`slip39 combine`** | `--to == Slip39ToShape::Entropy` | `"raw entropy"` | once in the combine run path |
| ~~`seedqr` / SeedQR~~ | — **no path produces a non-English SeedQR** (`seedqr encode` English-only + `convert --to seedqr` refused) | — | NONE — the footgun does not exist |

- **English / absent `--language` -> the helper returns `None` -> no advisory** (English is the universal default that self-recovers correctly).
- **Watch-only `bundle`** (xpub slots, no ms1) -> gate false -> no advisory.
- **Emitted ONCE per invocation** — for `convert`, evaluate `targets.contains(Entropy)` once (NOT a per-target loop) so co-occurring targets (`--to xpub,entropy`) fire a single advisory. For multisig `bundle`, the single `emit_unified` call fires once (not per-cosigner).
- **stderr only** — never on stdout / `--json` (`--json` consumers byte-unchanged).
- **Known limitation (M1):** a `bundle --slot @0.entropy=<hex>` / `slip39 split --from entropy=` with NO `--language` emits a secret-bearing form but carries no language signal -> the advisory cannot fire (nothing to act on). It keys off the *declared* `--language`; raw-entropy-without-language is silent by necessity. When `--language <non-english>` accompanies an entropy input it IS load-bearing (`bundle.rs:1342`) and fires correctly.

### 3.3 Why `convert`'s key-deriving targets are excluded

`--to xpub/xprv/wif/bip38/address/electrum-phrase` are DERIVED from the seed (the language was already applied to produce the correct key) — they ARE the key, with no re-recovery step where a language must be re-guessed. `--to phrase` keeps the language (it's a mnemonic in that language). Only `entropy` is a re-encodable seed-backup form that drops the language (`--to seedqr` is refused at parse, §2). So only `NodeType::Entropy` triggers the convert advisory.

---

## §4. SemVer + lockstep

- **PATCH.** stderr-only behavior; **no new flag** (`--language` exists), **no `--json` wire change** → **no GUI schema-mirror, no manual lockstep** (matches the bip48 "bless+warn" stderr-only precedent [[feedback_silent_default_with_stderr_notice]]). No CHANGELOG `[Unreleased]` block in repo → add a `[0.37.11]` entry at ship per the release-doc invariant.
- Toolkit-only. No sibling/codec change. `mnem` wire FOLLOWUP unchanged (stays open under the v0.2 arc).

---

## §5. Test plan

1. **Helper unit tests** (`language.rs` test mod): `non_english_seed_advisory(English, _) == None`; `non_english_seed_advisory(French, "an ms1 card")` is `Some` containing `"french"` + `"DIFFERENT"` + the form; **and a kebab-name language** (`SimplifiedChinese`, `"raw entropy"`) → `Some` containing `"simplified-chinese"` (M3 — lock the format).
2. **`bundle` integration:** `bundle --slot @0.phrase=<french 12-word> --language french …` → stderr advisory; `--language english` (or absent, English phrase) → NOT; a **watch-only** `--language french` bundle (xpub slot) → NOT (no ms1). Emitted exactly once for a 2-of-2 multisig. `--json` stdout byte-unchanged.
3. **`convert` integration:** `convert --from phrase=<french> --language french --to entropy` → advisory ("raw entropy"); co-occurring `--to xpub,entropy` → advisory fires exactly once; `--to xprv`/`--to phrase` → NO advisory; `--language english` → NO advisory. **`--to seedqr` is rejected at parse (exit 64)** — non-regression, no advisory path.
4. **`slip39` integration:** `slip39 split --from phrase=<french> --language french …` → advisory ("SLIP-39 shares"); `--language english` → NOT. `slip39 combine … --language french --to entropy` → advisory ("raw entropy"); `--to <phrase-form>` → NOT.
5. **`seedqr` non-regression:** `seedqr encode --from phrase=<french>` still errors (English-only, unchanged); no advisory machinery there.
6. Full suite + `cargo clippy --all-targets -- -D warnings` (the toolkit's real gates; **no fmt CI gate** [[feedback_toolkit_has_no_fmt_gate_unlike_sibling_codecs]]).

---

## §6. Phases

**Phase 0 — helper + tests.** `non_english_seed_advisory` in `language.rs` (`CliLanguage` already derives `PartialEq`, `language.rs:8` — no change needed) + unit tests (incl. kebab-name).

**Phase 1 — `bundle` site.** Emit at `emit_unified` (`bundle.rs:698`) when `any_secret_bearing()` + non-English. Integration tests (watch-only-no-fire + multisig-once + `--json`-stdout-unchanged).

**Phase 2 — `convert` site.** Emit once when `targets.contains(&NodeType::Entropy)` + non-English. Integration tests (multi-target-once, key-target-no-fire, `--to seedqr`-still-rejected).

**Phase 3 — `slip39` sites.** Emit on `split` (always, non-English → shares) + `combine` (`--to Entropy`, non-English). Integration tests.

**Phase 4 — version + CHANGELOG + ship.** Bump (0.37.11), `[0.37.11]` CHANGELOG entry, full-suite + clippy gate, end-of-cycle opus R0 → GREEN, clean-tree → ff-merge `master` + push + tag.

Per the mandatory R0 gate: this SPEC + the plan-doc each get an opus R0 to 0C/0I before any code.
