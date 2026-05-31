# Non-English BIP-39 wordlist-language advisory — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development or executing-plans. Steps use checkbox (`- [ ]`).

**Goal:** A stderr advisory at every language-losing seed emit (when `--language` is non-English) telling the user to record the wordlist language — ms1 / raw entropy / SLIP-39 shares carry only the entropy, not the language, so a non-English seed recovered with English-defaulted software derives a different wallet.

**Architecture:** One pure helper `non_english_seed_advisory(lang, form) -> Option<String>` (single message; `None` on English) + 4 call sites (bundle, convert, slip39 split, slip39 combine). stderr-only. PATCH 0.37.10 → 0.37.11; no GUI/manual lockstep.

**Source SPEC (R1 GREEN):** `design/SPEC_non_english_seed_advisory.md`. Base `master` `9f11a31`.

---

## Phase 0 — helper + unit tests

### Task 0.1 — `non_english_seed_advisory` in `language.rs`

**Files:** Modify `crates/mnemonic-toolkit/src/language.rs` (add helper + test mod cases). `CliLanguage` already derives `PartialEq` (`:8`) — no change.

- [ ] **Step 1 — Add the helper** (after the `impl CliLanguage`):
```rust
/// Returns a stderr advisory iff `lang` is a non-English BIP-39 wordlist (the
/// language is load-bearing for the seed but is NOT carried by `form`). `form`
/// names the language-dropping output ("an ms1 card", "raw entropy",
/// "SLIP-39 shares"). English → None (English self-recovers as the universal
/// default). See `design/SPEC_non_english_seed_advisory.md`.
pub(crate) fn non_english_seed_advisory(lang: CliLanguage, form: &str) -> Option<String> {
    if lang == CliLanguage::English {
        return None;
    }
    let name = lang.human_name();
    Some(format!(
        "warning: encoding a {name} BIP-39 seed as {form} — it carries only the \
         entropy, not the wordlist language. Record \"{name}\" alongside the backup: \
         recovering the entropy with English-defaulted software derives a DIFFERENT \
         seed and a DIFFERENT wallet."
    ))
}
```

- [ ] **Step 2 — Unit tests** (in `language.rs` `#[cfg(test)] mod tests`):
```rust
#[test]
fn advisory_none_for_english() {
    assert_eq!(non_english_seed_advisory(CliLanguage::English, "an ms1 card"), None);
}
#[test]
fn advisory_some_for_french_with_form() {
    let m = non_english_seed_advisory(CliLanguage::French, "an ms1 card").unwrap();
    assert!(m.contains("french"), "{m}");
    assert!(m.contains("an ms1 card"), "{m}");
    assert!(m.contains("DIFFERENT"), "{m}");
}
#[test]
fn advisory_uses_kebab_name() {
    let m = non_english_seed_advisory(CliLanguage::SimplifiedChinese, "raw entropy").unwrap();
    assert!(m.contains("simplified-chinese"), "{m}");
}
```
(Confirm the exact variant names `French` / `SimplifiedChinese` against `language.rs` `enum CliLanguage` at impl time.)

- [ ] **Step 3 — Run** `cargo test -p mnemonic-toolkit --bin mnemonic 'language::tests::advisory'` → 3 pass.

- [ ] **Step 4 — Commit.** `git add crates/mnemonic-toolkit/src/language.rs && git commit -m "feat(toolkit): non_english_seed_advisory helper (single message, None on English)"`

---

## Phase 1 — `bundle` site

### Task 1.1 — Emit in `emit_unified`

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/bundle.rs` (`emit_unified`, `:698`) + an integration test in `tests/`.

`emit_unified<W, E>(args: &BundleArgs, bundle: &Bundle, …, stderr: &mut E, …)` has `args.language: Option<CliLanguage>` (`:42`), the synthesized `bundle` (`Bundle::any_secret_bearing()`, `synthesize.rs:35`), and `stderr` — all in scope. It emits stderr advisories regardless of `--json` (per its own §5.5.a comment).

- [ ] **Step 1 — Add the emit** near the other stderr advisories in `emit_unified` (e.g. alongside the bip48 warning emit pattern):
```rust
    if bundle.any_secret_bearing() {
        if let Some(msg) = crate::language::non_english_seed_advisory(
            args.language.unwrap_or_default(),
            "an ms1 card",
        ) {
            writeln!(stderr, "{msg}").ok();
        }
    }
```
(Import path: `non_english_seed_advisory` is `pub(crate)` in `crate::language`.)

- [ ] **Step 2 — Integration test** (`tests/cli_bundle_*` — e.g. a new `tests/cli_bundle_language_advisory.rs`). **I3 — concrete checksum-valid vectors (all-zeros entropy; verified via `convert --to phrase`):**
```rust
// French all-zeros-entropy 12-word (word[0]=abaisser, word[11]=abeille).
const FRENCH_12: &str = "abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abeille";
// English all-zeros-entropy 12-word (the canonical no-fire control).
const ENGLISH_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
```
  Assert:
  - `bundle --slot @0.phrase=<FRENCH_12> --language french --template bip84 --network mainnet --no-engraving-card` → stderr contains `"BIP-39 seed as an ms1 card"` + `"french"`.
  - `--slot @0.phrase=<ENGLISH_12> --language english` → stderr does NOT contain the advisory.
  - a **watch-only** `--slot @0.xpub=<xpub> --slot @0.fingerprint=<fp> --language french` → does NOT (no ms1).
  - a 2-of-2 multisig (both FRENCH_12 phrase slots, `--language french`) → the advisory appears **exactly once** (`stderr.matches("BIP-39 seed as an ms1 card").count() == 1`).
  - `--json` → stdout parses as JSON unchanged; the advisory is on stderr only.
  - **M2 note:** `bundle --import-json <french-secret-bearing-envelope> --language french` ALSO fires (emit lives in `emit_unified`, reached by the import-json branch) — this is correct (re-emitting ms1 still drops language); add one assertion or a comment so it's not read as accidental.

- [ ] **Step 3 — Run** the bundle suite + the new test → green.

- [ ] **Step 4 — Commit.** `git add crates/mnemonic-toolkit/src/cmd/bundle.rs crates/mnemonic-toolkit/tests/cli_bundle_language_advisory.rs && git commit -m "feat(toolkit): bundle emits non-English seed advisory (ms1)"`

---

## Phase 2 — `convert` site

### Task 2.1 — Emit when `--to entropy`

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/convert.rs` (`run`, `:737`) + integration test.

`run<R, W, E>(args: &ConvertArgs, …, stderr: &mut E, …)` has `args.language: Option<CliLanguage>` (`:245`) + `targets: Vec<NodeType>` (built `:850`). **I1 fold — insert AFTER ALL refusal guards**, not just the §3 pre-check loop (`:882-886`): there are 3 more `return Err` after it — WIF+`--path` (`:890`), `--xpub-prefix`-without-`--network` (`:896`), BIP-38-without-passphrase (`:905`) — plus the existing stderr-advisory cluster (`:926-947`). Insert **immediately after `:947`, before `compute_outputs` (`:952`)**, so a refused/failed edge never advises. `targets` + `args.language` + `stderr` are all in scope there.

- [ ] **Step 1 — Add the emit**:
```rust
    if targets.contains(&NodeType::Entropy) {
        if let Some(msg) = crate::language::non_english_seed_advisory(
            args.language.unwrap_or_default(),
            "raw entropy",
        ) {
            writeln!(stderr, "{msg}").ok();
        }
    }
```
(Evaluated ONCE — `targets.contains` handles co-occurring `--to xpub,entropy` with a single advisory.)

- [ ] **Step 2 — Integration test** (`tests/cli_convert_*`): declare `FRENCH_12` (the I3 const from Phase 1). Assert:
  - `convert --from phrase=<FRENCH_12> --language french --to entropy` → stderr advisory (`"raw entropy"`).
  - `--to xpub,entropy` (French) → advisory appears exactly once.
  - `--to xprv` (French) → NO advisory; `--to phrase` (French) → NO advisory.
  - `--language english` `--to entropy` → NO advisory.
  - `--to seedqr` → still rejected at parse (exit 64), non-regression.

- [ ] **Step 3 — Run** → green.

- [ ] **Step 4 — Commit.** `git add crates/mnemonic-toolkit/src/cmd/convert.rs crates/mnemonic-toolkit/tests/cli_convert_language_advisory.rs && git commit -m "feat(toolkit): convert --to entropy emits non-English seed advisory"`

---

## Phase 3 — `slip39` sites

### Task 3.1 — split (always) + combine (`--to Entropy`)

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/slip39.rs` (`run_split` `:359`, `run_combine`) + integration test. (Note: `cmd/slip39.rs`, NOT the `src/slip39/` library module — M-new-2.)

`run_split<R, W, E>(args, …, stderr)` has `args.language: CliLanguage` (`:132`, default english, NOT Option). `run_combine` has `args.language` (`:171`) + `args.to: Slip39ToShape` (`:168`, variants `Entropy` `:185` / `Phrase` `:188`).

- [ ] **Step 1 — `run_split`**: split → shares always lose the language (phrase→entropy→shares). Add **after `parse_master_to_entropy` succeeds (`:437`)** (I2 — a bad phrase shouldn't advise-then-error):
```rust
    if let Some(msg) = crate::language::non_english_seed_advisory(args.language, "SLIP-39 shares") {
        writeln!(stderr, "{msg}").ok();
    }
```

- [ ] **Step 2 — `run_combine`**: only `--to Entropy` drops the language (`--to phrase` re-encodes in `args.language`). Add **after the successful `slip39_combine` (`:644`), before the `match args.to` output render (`:647`)** (I2 — a bad share shouldn't advise-then-error); `Slip39ToShape::Entropy` is at `:185`, `Phrase` `:187`:
```rust
    if matches!(args.to, Slip39ToShape::Entropy) {
        if let Some(msg) = crate::language::non_english_seed_advisory(args.language, "raw entropy") {
            writeln!(stderr, "{msg}").ok();
        }
    }
```

- [ ] **Step 3 — Integration test** (`tests/cli_slip39_*`): declare `FRENCH_12`/`ENGLISH_12` (the I3 consts). Assert:
  - `slip39 split --from phrase=<FRENCH_12> --language french …` → stderr advisory (`"SLIP-39 shares"`).
  - `slip39 split --from phrase=<ENGLISH_12> --language english …` → NO advisory.
  - `slip39 combine <french shares> --language french --to entropy` → advisory (`"raw entropy"`). (Generate the French shares by capturing the `split --from phrase=<FRENCH_12> --language french` stdout.)
  - `slip39 combine … --to phrase` (or the default phrase form) → NO advisory.

- [ ] **Step 4 — Run** → green.

- [ ] **Step 5 — Commit.** `git add crates/mnemonic-toolkit/src/cmd/slip39.rs crates/mnemonic-toolkit/tests/cli_slip39_language_advisory.rs && git commit -m "feat(toolkit): slip39 split/combine emit non-English seed advisory"`

---

## Phase 4 — version + CHANGELOG + ship

### Task 4.1 — version + CHANGELOG

**Files:** `crates/mnemonic-toolkit/Cargo.toml:3`, `CHANGELOG.md`, both README markers.

- [ ] **Step 1 — Bump** `0.37.10` → `0.37.11`; both README `<!-- toolkit-version: 0.37.11 -->` markers (`README.md:13`, `crates/mnemonic-toolkit/README.md:9`).
- [ ] **Step 2 — CHANGELOG** `[0.37.11] — <date>` entry: "SemVer-PATCH — stderr advisory at language-losing seed emits (bundle ms1 / convert --to entropy / slip39 split shares / slip39 combine --to entropy) when --language is non-English; ms1/entropy/shares carry only the entropy, not the BIP-39 wordlist language. Path A of the `mnem` footgun; the wire fix stays filed under `mnem-wordlist-language-hint-on-wire`. No flag/wire change → no GUI/manual lockstep."
- [ ] **Step 3 — Commit.** `git add crates/mnemonic-toolkit/Cargo.toml CHANGELOG.md README.md crates/mnemonic-toolkit/README.md && git commit -m "release(toolkit): v0.37.11 — non-English seed advisory"`

### Task 4.2 — Full gate + end-of-cycle R0 + ship

- [ ] **Step 1 — FULL gate:** `cargo test -p mnemonic-toolkit --no-fail-fast` 0 failures; `cargo clippy --all-targets -- -D warnings` clean. (Toolkit CI has **no fmt gate** — do NOT block on `cargo +stable fmt`; the whole-crate drift under newer rustfmt is a non-gate, [[feedback_toolkit_has_no_fmt_gate_unlike_sibling_codecs]].)
- [ ] **Step 2 — End-of-cycle opus R0** over the full branch diff → persist to `design/agent-reports/`. Fold to GREEN (0C/0I).
- [ ] **Step 3 — Clean-tree check** (`git status --porcelain` empty), ff-merge `master` + push + tag `mnemonic-toolkit-v0.37.11`.

---

## Self-review
- **Spec coverage:** §3.1 helper→0.1; §3.2 bundle→1.1, convert→2.1, slip39 split+combine→3.1; SeedQR-impossible (no task, correct); §4 SemVer→4.1; §5 tests→per-phase; §6 phases→0-4.
- **Placeholders:** variant names (`French`/`SimplifiedChinese`/`Slip39ToShape::Entropy`) + the convert insertion line confirmed at impl. No TODO/TBD.
- **Type consistency:** `non_english_seed_advisory(CliLanguage, &str) -> Option<String>` identical across all 4 sites; bundle/convert pass `args.language.unwrap_or_default()` (Option), slip39 passes `args.language` (direct CliLanguage). stderr-only at every site.
