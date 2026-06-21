# cycle-prep recon — 2026-06-21 — cycle-8 ms-cli robustness/advisory cluster (H4, H5, L26, L5)

**ms `origin/master` SHA at recon time:** `44ac71f` (`44ac71fd6cb055e25e3c3dd1daca230d5d45bafb`) — UNCHANGED from cycle-4 last-known.
**Toolkit `origin/master` SHA:** `8d2fe505`.
**ms-codec / ms-cli versions @ `origin/master`:** ms-codec `0.5.0`, ms-cli `0.8.1`.
**Local branch (mnemonic-secret):** `master` — **0 ahead / 2 behind** `origin/master`, and the **working tree is DIRTY** (uncommitted edits to `crates/ms-cli/src/error.rs`, `crates/ms-codec/src/{error.rs,shares.rs}`, both Cargo.toml/CHANGELOGs). The local checkout is an OLDER state (Cargo.toml shows ms-codec 0.4.4 / ms-cli 0.8.0, and the working tree has a FLAT `src/` with no `cmd/` dir). **All citations in this recon were verified against `origin/master` BYTES via `git show origin/master:…`, NOT the working tree.** A naive `find`/`Read` of the checkout misleads (it lacks `cmd/`). Brainstorm/impl MUST branch off `origin/master`, not the dirty local tree.
**Untracked (ms repo):** two prior cycle-prep recons + design notes; none relevant.

Findings live in **mnemonic-secret**; the bughunt report is `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md`. Drift expectation going in: low (report is recent). Verdict: **all 4 ACCURATE; zero drift.**

---

## Per-finding verification

### H4 — `ms derive` panics `unreachable!` on a valid non-English (mnem) ms1
- **WHAT:** `ms derive`'s ms1-decode arm matches only `Payload::Entr`; a non-English ms1 decodes to `Payload::Mnem` → `_ => unreachable!()` panic mid-recovery (E-panic-dos).
- **Citations:**
  - `crates/ms-cli/src/cmd/derive.rs:185` `_ => unreachable!("ms-codec v0.1 decodes only Payload::Entr")` — **ACCURATE** (exact line, exact text). The match is at `:183-186`, inside the `else` (ms1) branch of `run()`.
  - "siblings handle both; only derive panics" — **ACCURATE.** `combine.rs:95-107` already matches both `Entr` and `Mnem`; `decode.rs` handles both.
  - "decode() returns `Payload::Mnem { language, entropy }` for non-English" — **ACCURATE** vs ms-codec `decode.rs:88-95` (validates + returns `Mnem`). `Payload` is `#[non_exhaustive]` (`payload.rs:29`); `Mnem { language: u8, entropy }` with `language` 0..=9.
  - Secondary "wire-language" correctness bug — **ACCURATE & LOAD-BEARING.** In `derive.rs` `lang` is resolved from CLI `--language` (default English) at `:165-169`, then `Mnemonic::from_entropy_in(lang, …)` at `:188`. The wire `Mnem.language` byte is **discarded**. A naive panic-fix (just match `Mnem(b) => b`) re-builds the mnemonic in the WRONG (English-default) language → seed = PBKDF2 over the wrong sentence → **wrong fingerprint/account-xpub** for non-English seeds. The fix MUST source the wordlist language from the WIRE byte, not the `--language` flag.
  - cited "`CliLanguage::from_code`" helper — **ACCURATE & EXISTS**: `language.rs:34` `pub fn from_code(c: u8) -> Option<CliLanguage>` (0→English…9→Portuguese). Clean primitive: `from_code(mnem.language)` → `CliLanguage` → `bip39::Language` via existing `From`.
- **STILL-REPRODUCES:** **YES.** Verbatim panic site live at `origin/master`.
- **Fix-site:** `crates/ms-cli/src/cmd/derive.rs` ms1 branch (`:183-188`). Match `Payload::Mnem { language, entropy }` → build `bip39::Language` from `CliLanguage::from_code(language)` (NOT the CLI `lang`); keep `Payload::Entr` on the CLI/default lang; keep `_ => unreachable!` as the genuine `#[non_exhaustive]` future-variant guard. **ms-cli-only** (codec already emits `Mnem`).

### H5 — `ms verify` panics `unreachable!` on a valid non-English (mnem) ms1
- **WHAT:** Same stale v0.1 assumption — `ms verify`'s decode match only handles `Entr`; `Mnem` is reachable → panic. The safety-check command DoS's on a valid non-English card.
- **Citations:**
  - `crates/ms-cli/src/cmd/verify.rs:64` `Ok((_, _)) => unreachable!("ms-codec v0.1 only decodes to Payload::Entr")` — **ACCURATE** (exact line + text). Match at `:60-66`: `Ok((_tag, Payload::Entr(b)))` handled; `Ok((_, _))` panics; `Err` dispatched.
  - "--phrase round-trip uses CLI `--language` not wire language" — **ACCURATE.** `verify.rs:86` `let lang: Language = args.language.into();` then `Mnemonic::from_entropy_in`/`parse_in` use it. Same wire-language nuance as H4 for the round-trip leg.
- **STILL-REPRODUCES:** **YES.**
- **Fix-site:** `crates/ms-cli/src/cmd/verify.rs:60-66`. Extract entropy from BOTH `Entr` and `Mnem`; for the `--phrase` round-trip, honor the WIRE language byte when present (mnem) rather than the CLI default; keep `unreachable!` as the future-variant guard. **ms-cli-only.**

### H4 + H5 = ONE shared fix?
- **Conceptually ONE root cause, but TWO distinct edit-sites** (`derive.rs` + `verify.rs`), no shared helper today. They share the SAME funds-safety nuance: **use the wire `Mnem.language` byte (via `CliLanguage::from_code`), never the `--language` flag**, to avoid a wrong fingerprint/wrong round-trip on non-English seeds. Recommend a small shared helper (e.g. `payload_to_entropy_and_language(&Payload) -> (Zeroizing<Vec<u8>>, CliLanguage)`) — `combine.rs:95-107` is the proven in-crate template — so derive/verify/combine resolve the wire language identically and a future `Payload` variant is handled once. Both are still ms-cli-only.

### L26 — `ms combine --to entropy` silently drops the mnem wordlist-language (no advisory)
- **WHAT:** `ms combine` correctly resolves the wire language, but the `--to entropy` arm emits raw hex with NO language advisory → a user recording only the hex + recovering with English-default software derives a different seed/wallet (B-policy-collapse, advisory gap). Asymmetric vs the toolkit, which warns.
- **Citations:**
  - cited `combine.rs:91-117` (payload routing) — **ACCURATE (DRIFTED-trivially):** the routing `match &payload` is at `:95-108`; it correctly does `CliLanguage::from_code(*wire_code).unwrap_or(English)` and binds `language`. The cited `:91-117` envelope is right; the exact match opens at `:95`.
  - cited `:157-175` `emit_entropy` (hex only) — **ACCURATE:** `fn emit_entropy` at `:157`; it takes only `(entropy, kind, json)` — the resolved `language` is NOT passed in, so the hex-only/`--to entropy` (`CombineTo::Entropy`) arm at `:111` drops it. JSON `CombineJson` for this arm already sets `language: None` (`:165-172`).
  - "toolkit emits `non_english_seed_advisory` for exactly this; ms-cli has no such helper" — **ACCURATE.** Toolkit: `ms_shares.rs:449` (`MsSharesToShape::Entropy` → `non_english_seed_advisory(cli_lang, "raw entropy")` to stderr, keyed off the RECOVERED payload language); helper at `toolkit language.rs:177`. ms-cli `advisory.rs` has `emit_output_class_advisory` / `secret_in_argv_warning` but NO language-specific seed advisory.
- **STILL-REPRODUCES:** **YES** (advisory gap; entropy hex itself is correct).
- **Fix-site:** `crates/ms-cli/src/cmd/combine.rs`. Plumb the already-resolved `language` into the `--to entropy` arm and, when it's a non-English `mnem` recovery, emit a stderr advisory (port the toolkit's `non_english_seed_advisory` text, or add it to ms-cli `advisory.rs`). `--to phrase` (re-renders in-language) and `--to ms1` (re-encodes the mnem payload, language preserved) need NO advisory — mirror the toolkit's arm-selective behavior. **ms-cli-only.** Stderr-only — NOT a `--json` wire-shape change (the `language: None` JSON field is unchanged).

### L5 — `CliError::Codex32` wraps a raw `codex32::Error`, bypassing the sanitizing Debug (latent secret leak)
- **WHAT:** `#[derive(Debug)] enum CliError` carries `Codex32(codex32::Error)` directly; `codex32::Error::InvalidChecksum { string }` carries the full secret ms1 string. Any future `{:?}`/`unwrap`/`expect`/`panic` on this variant leaks it. Production path uses `Display`/`message()` (safe today) → latent.
- **Citations:**
  - `crates/ms-cli/src/error.rs:20` `Codex32(codex32::Error)` — **ACCURATE** (exact line). `#[derive(Debug)]` at `:12`; `#[non_exhaustive]` at `:13`; `enum CliError` at `:14`.
  - "production uses Display/`message()` (safe)" — **ACCURATE.** `Display` (`:118-121`) → `self.message()` (`:73`) → `friendly_codex32(e)` (`:77`), which renders sanitized prose. The DERIVED `Debug` is the latent leak vector; nothing live `{:?}`-formats `CliError` today.
- **STILL-REPRODUCES:** **YES** (latent — derived Debug still carries the raw error at `origin/master`).
- **Fix-site:** `crates/ms-cli/src/error.rs`. Either (a) hand-roll `impl std::fmt::Debug for CliError` delegating to the sanitized Display/`message()`, or (b) stop carrying the raw `codex32::Error` (carry a pre-sanitized message + kind). (a) is lower-blast-radius (Display already sanitized). Add a no-echo test asserting `format!("{:?}", CliError::Codex32(InvalidChecksum{string: "<secret ms1>"}))` does NOT contain the secret. **ms-cli-only.**

---

## Cross-cutting observations

1. **Dirty/behind local checkout is the only real hazard.** The mnemonic-secret working tree is 2-behind + has uncommitted edits, and is an OLDER structural state (FLAT `src/`, no `cmd/`). `find`/`Read` of the checkout returns a misleading flat tree. Cycle-8 brainstorm + impl MUST branch off `origin/master` (`44ac71f`) and cite `git show origin/master:…` line numbers (used throughout this recon). Investigate/stash the stray working-tree edits before branching.
2. **Zero structural drift across all 4 findings.** Every cited path + line + symbol + text verified ACCURATE against `origin/master`. Only L26's `match`-open line is trivially inside (not at the top of) the cited `:91-117` envelope — not a structural error.
3. **No ms-codec change needed.** `ms_codec::decode()` already returns `Payload::Mnem { language, entropy }` (`decode.rs:88-95`); `Payload` is already `#[non_exhaustive]` (`payload.rs:29`); `CliLanguage::from_code` already exists (`language.rs:34`). All four fixes are **ms-cli-only**. ms-codec stays **NO-BUMP**.
4. **No clap-surface change → GUI schema-mirror + manual NOT triggered.** None of the 4 fixes add/remove/rename a subcommand, flag, or dropdown value (H4/H5 = match-arm + language sourcing; L26 = a stderr advisory line; L5 = a Debug impl). The ms-cli `gui-schema` SPEC §7 contract (`tests/gui_schema_emits_spec_v7_json.rs`) is unaffected. The toolkit manual `docs/manual/src/40-cli-reference/43-ms.md` mirrors clap `--help` — unchanged. **No GUI schema-mirror lockstep, no manual lockstep.** (If the brainstorm CHOOSES to document the new advisory text in the manual prose, that's optional polish, not a gate.)
5. **No `--json` wire-shape change.** L26 adds a stderr advisory only; the `CombineJson` `--to entropy` payload already has `language: None` and is untouched. (Per CLAUDE.md, `--json` wire-shape isn't schema-gated anyway, but worth noting there's nothing for GUI consumers to self-update.)
6. **Wire-language nuance (the funds-safety crux) verified against primary semantics.** BIP-39: the seed = PBKDF2-HMAC-SHA512 over the NFKD-normalized mnemonic SENTENCE; the SENTENCE is wordlist-language-specific, so the same entropy under two wordlists yields two different seeds → two different wallets. codex32/BIP-93 `mnem` payload carries `[0x02][language_byte][entropy]` (`payload.rs:45-53`), so the wire language IS recoverable and is authoritative. The fix must use it; the `--language` flag (English default) must NOT override it. This matches the memory note: "fix must use the WIRE language byte, not a `--language` flag, to avoid a wrong fingerprint."

---

## SemVer + publish→pin chain

- **ms-cli:** **MINOR → `0.8.1` → `0.9.0`.** Pre-1.0 convention (per ms CHANGELOG header): `0.X` is the breaking-change axis; behavior changes that fix panics / add a new stderr advisory / re-source the wire language (a behavior change for non-English `derive`/`verify`) warrant a MINOR. No CLI-surface break. → **tag + publish ms-cli `0.9.0` to crates.io** (registry crate, tag+publish lane).
- **ms-codec:** **NO-BUMP** (no codec change required; decode already returns `Mnem`).
- **Toolkit pin:** **NONE.** The toolkit depends on **`ms-codec = "0.5"`** (LIBRARY), NOT `ms-cli` (BINARY). An ms-cli-only MINOR does not enter the toolkit dependency graph. ms-codec NO-BUMP → toolkit Cargo.lock unchanged → **no toolkit pin, no toolkit bump, no GUI bump.**
- **Cross-repo FOLLOWUP companions:** none required — all fixes are self-contained in ms-cli. (If a shared `payload_to_entropy_and_language` helper is added, it's purely internal to ms-cli.)

---

## Recommended brainstorm-session scope

- **Single cycle = cycle-8, one ms-cli MINOR (`0.9.0`).** All 4 are ms-cli-only, NO-BUMP ms-codec, no toolkit/GUI touch.
- **Group + ordering:**
  - **Group A (panics, funds-safety core): H4 + H5.** ONE root cause / TWO edit-sites + shared wire-language nuance. Strongly recommend a **shared in-crate helper** resolving `Payload → (entropy, wire-CliLanguage)` (template: `combine.rs:95-107`) so derive + verify + combine route identically and the future-`Payload` guard lives once. TDD: a non-English (e.g. Japanese) ms1 → `ms derive` yields the SAME fingerprint as the original phrase (oracle: derive from the phrase directly); `ms verify --phrase <ja phrase>` round-trips OK. ~60-110 LOC incl. tests.
  - **Group B (advisory gap): L26.** Port `non_english_seed_advisory` text into ms-cli `advisory.rs`; emit on the `--to entropy` arm keyed off recovered wire language; arm-selective (not phrase/ms1). TDD: stderr advisory present for a non-English mnem `--to entropy`, absent for English/entr and for `--to phrase|ms1`. ~30-50 LOC.
  - **Group C (latent leak): L5.** Hand-roll `CliError` Debug → sanitized Display (option a). TDD: no-echo `{:?}` test. ~20-35 LOC.
- **SemVer:** ms-cli **MINOR `0.9.0`**; ms-codec **NO-BUMP**; toolkit **untouched**.
- **Locksteps:** **none** — no clap flag/subcommand/dropdown change (no GUI schema-mirror), no CLI-surface change (no manual mirror), no sibling-codec change (no FOLLOWUP companion). Optional: a manual-prose note for the new L26 advisory (not a gate).
- **Gate reminder:** per CLAUDE.md, brainstorm spec + plan-doc each pass the opus R0 loop to **0C/0I** BEFORE any code; persist reviews verbatim to `design/agent-reports/`; single-subagent-per-phase TDD; mandatory whole-diff post-impl review. **Branch off ms `origin/master` `44ac71f`** (NOT the dirty local tree) and cite that SHA in the spec.
