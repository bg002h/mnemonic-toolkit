# cycle-prep recon — 2026-06-03 — bundle-slot-ms1-input

**Origin/master SHA at recon time:** `0814ab5`
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** `.claude/`, `CONTINUITY.md`, several prior-cycle `cycle-prep-recon-*.md`, `feature-coverage-survey-*.md`, `stderr*.txt` (none relevant)

Slug verified: **none exists** — this is a NET-NEW feature (no FOLLOWUP slug; proposed slug `bundle-slot-ms1-input`). Feature recon instead of citation re-verification. No drift to report (nothing to drift from); the recon establishes the current ground truth the brainstorm must build on.

---

## Feature recon — `bundle`/`verify-bundle --slot @N.ms1=<ms1-string>`

### WHAT
Accept a raw `ms1` (BIP-93 codex32 / ms-codec) string as a first-class `--slot` subkey input, parallel to the existing `@N.phrase=`/`@N.entropy=`/`@N.seedqr=` secret subkeys. The toolkit decodes the ms1 inline at slot-emit time (`ms_codec::decode` → `Payload::Entr` raw entropy, or `Payload::Mnem { entropy, language }`) and dispatches through the existing entropy/phrase materialization. Justification: (a) **language preservation** — a `mnem` ms1 carries its BIP-39 wordlist language, which is load-bearing for seed derivation; (b) BCH-checksum typo-detection on input; (c) round-trip symmetry (engrave an ms1 card → feed the same card back).

### Ground-truth citations (verified against `0814ab5`)

- **`crates/mnemonic-toolkit/src/slot_input.rs:16-42`** — `enum SlotSubkey` has **9 variants** (Phrase, Seedqr, Entropy, Xpub, MasterXpub, Fingerprint, Path, Wif, Xprv). **No `Ms1`.** — ACCURATE.
- **`slot_input.rs:44-58` `from_token` / `:59-71` `as_str`** — token↔variant map; **no `"ms1"`**. — ACCURATE.
- **`slot_input.rs:72-77` `is_secret_bearing`** — `{Phrase, Seedqr, Entropy, Xprv, Wif}`. ms1 (entr/mnem) is secret-bearing → a new `Ms1` arm returns `true`. — ACCURATE.
- **`slot_input.rs:114-120` (`cmd/bundle.rs`) clap `--slot`** — `value_parser = parse_slot_input`, value type `Vec<SlotInput>`, free-form `String` per occurrence. **The subkey token set is validated INSIDE `parse_slot_input` via `from_token` (slot_input.rs:160-165), NOT as a clap `PossibleValuesParser`/value-enum.** Therefore adding `ms1` is **NOT a new clap flag-name and NOT a new clap value-enum** → does **not** touch the `schema_mirror` flag-name gate. — ACCURATE (load-bearing for the lockstep call).
- **`slot_input.rs:160-165`** — the unknown-subkey error string enumerates the valid tokens (`"...expected one of: phrase, seedqr, entropy, xpub, master_xpub, fingerprint, path, wif, xprv"`). Must append `ms1`. — ACCURATE.
- **`slot_input.rs:330-352` `is_legal_set`** — legal subkey-set matrix. A new `Ms1` needs at minimum `[Ms1]`; brainstorm decides whether to mirror the Seedqr/Phrase exemptions `[Ms1, Path]` / `[Ms1, Fingerprint, Path]` (non-canonical descriptor mode). Ord position of the new variant determines which canonical-order arms appear. — ACCURATE.
- **`slot_input.rs:111` `SECRET_SLOT_SUBKEYS` parity test (`:404-425`)** + **`crates/mnemonic-toolkit/src/secret_taxonomy.rs:111`** `SECRET_SLOT_SUBKEYS = ["phrase","seedqr","entropy","xprv","wif"]` — a CI parity test (`secret_taxonomy_parity_with_is_secret_bearing`) asserts `is_secret_bearing()` ⟺ membership. Adding `Ms1` (secret) **MUST** add `"ms1"` here or the test fails. — ACCURATE (internal gate).
- **`cmd/bundle.rs:449-457` `resolve_slots`** — the materialization dispatcher; signature already carries `language: Option<CliLanguage>` + `passphrase`. Phrase/Seedqr arm at **`:487-535`** (`lang = language.unwrap_or_default()`), Entropy arm at **`:606-655`** (`lang_bip39: bip39::Language = lang.into()` → `derive_bip32_from_entropy(..., lang_bip39, ...)`). A new `Ms1` arm decodes then routes here. — ACCURATE. **The entropy-derivation path is already language-parameterized**, so the mnem wire-language hook exists.
- **`cmd/convert.rs:1464-1477`** — **the exact ms1-decode pattern already implemented**: `let (_tag, payload) = ms_codec::decode(value)?;` → `Payload::Entr(bytes) => …` / `Payload::Mnem { entropy, language: wire_lang, .. } => …`. The `Ms1` slot arm replicates this. — ACCURATE (reference implementation).
- **`crates/mnemonic-toolkit/src/language.rs:144-148`** — `wire_code_to_bip39(*language)` maps `Payload::Mnem { language }` → `bip39::Language`. The language-threading helper EXISTS. — ACCURATE.
- **`mnemonic-secret/crates/ms-codec/src/payload.rs:30-51`** (published 0.4.0) — `Payload::Entr(Vec<u8>)` + `Payload::Mnem { entropy, language, .. }`; **`decode.rs:42` `pub fn decode(s) -> Result<(Tag, Payload)>`**. A threshold-≠0 K-of-N **share** fed to `@N.ms1=` returns `Error::IsShareNotSingleString` (free footgun-guard pointing at `ms-shares combine`; friendly prose already shipped this cycle). — ACCURATE.
- **`cmd/verify_bundle.rs:119`** — `--slot` with the SAME `parse_slot_input`; calls `resolve_slots` at **`:363,:453,:557`** and special-cases `SlotSubkey::Phrase`/`Seedqr` at **`:719-720,:781`**. The shared `resolve_slots` arm covers verify-bundle automatically; the Phrase/Seedqr peer-checks may need an `Ms1` peer. — ACCURATE (verify-bundle is in-scope, shares the codepath).
- **`cmd/gui_schema.rs`** — emits `dropdown` value-enums only for true clap value-enums (`--template` via `to_possible_value`, `:281-292`); **does NOT project the `--slot` subkey token list.** — ACCURATE → no `schema_mirror` change.
- **`cmd/bundle.rs:94-113`** — the `--slot` clap `verbatim_doc_comment` enumerates the subkey list shown in `--help`. Must add the `ms1` line. (Help text is NOT manual-mirror-gated, but the manual prose IS authored from it.) — ACCURATE.

### mnem-language semantics (the one real design hazard)
BIP-39 seed = PBKDF2(NFKD(mnemonic_sentence), …). Different wordlists ⇒ different sentence ⇒ **different seed for the same entropy**. So a `mnem` ms1's **wire language is authoritative** for the slot's seed derivation — an `@N.ms1=<japanese-mnem>` decoded as English yields the WRONG xpub/addresses silently (the exact §6.3 footgun the `mnem` cycle closed on emit). Brainstorm MUST decide: (1) mnem ms1 → wire language overrides `--language`/default (with a stderr note if `--language` is also supplied and disagrees, or refuse-on-conflict); (2) entr ms1 → no language → `--language`/default (matches the existing `Entropy` subkey); (3) whether the emitted ms1 card re-encodes as `mnem` (preserving language out — `ResolvedSlot.language` field at `bundle.rs:533/603/653` is currently `None` everywhere and exists for exactly this).

### Action for brainstorm spec
Add `SlotSubkey::Ms1` (decide Ord position + secret-bearing), `from_token`/`as_str`/`is_legal_set` arms, the `SECRET_SLOT_SUBKEYS` entry, the error-string + doc-comment token, and one `resolve_slots` arm modeled verbatim on `convert.rs:1464-1477` + `language.rs:144-148`. Resolve the mnem-language-authority policy. Cite source SHA `0814ab5` (toolkit) + ms-codec 0.4.0.

---

## Cross-cutting observations

1. **No slug exists** — net-new feature; no citation drift possible. The `--slot @0.ms1=` string was cited *aspirationally* in the just-shipped K-of-N SPEC/plan but was never built (confirmed: K-of-N record + `feature-coverage` notes). File a FOLLOWUP slug `bundle-slot-ms1-input` if not implemented immediately.
2. **Stale doc-comments**: `bundle.rs:130-133,219,1576` reference a `--ms1` "seed overlay" flag that **no longer exists on `bundle`** (the v0.5.1 `--slot`-only unification, `bundle.rs:316`, replaced it). `bundle.ms1` (`:859,884,…`) is the OUTPUT card vector, not an input. Minor comment-rot, orthogonal to this feature (note for a future doc sweep; do not let it confuse the brainstorm into thinking ms1-input already exists).
3. **Lockstep is GUI-secret-projection, NOT schema_mirror.** Because slot subkeys are a free-form `--slot` value (not a clap value-enum), the `schema_mirror` flag-name gate is untouched. BUT `mnemonic-gui` carries a hand-maintained mirror: `src/form/slot_editor.rs::SlotSubkey` (the GUI's slot-row picker) + `src/secrets.rs:43-56` ("Snapshot of `SECRET_SLOT_SUBKEYS`") which drives `persistence.rs:91` slot-value redaction. Adding `ms1` (secret) requires a **paired GUI PR** (picker option + secret-snapshot entry) per `feedback_gui_schema_secret_projection_lockstep`. Verify at impl whether a drift test consumes the toolkit `SECRET_SLOT_SUBKEYS` (auto-gate) or it is hand-maintained (discipline-only).
4. **Manual mirror**: the manual flag-coverage lint gates long *flags*, not subkey tokens inside `--slot`; adding `ms1` likely won't trip it, but `docs/manual/src/40-cli-reference/41-mnemonic.md` bundle/verify-bundle `--slot` subkey docs should add `ms1` in prose (quality, not hard gate).
5. **No sibling-codec change.** Toolkit consumes the published ms-codec 0.4.0 `decode`/`Payload` API; no ms-codec/ms-cli edit, no companion FOLLOWUP.

---

## Recommended brainstorm-session scope

- **Single cycle, SMALL** (~150–250 LOC incl. tests). One subsystem (`slot_input` + `resolve_slots` + the two consumers bundle/verify-bundle). 1–2 architect rounds expected (per `feedback_smaller_cycle_scope_reduces_citation_surface`).
- **SemVer:** additive accepted-value on an existing subcommand ⇒ **PATCH** by the cycle-prep rule (toolkit v0.40.0 → **v0.40.1**), though a case exists for MINOR (new user-facing input *capability*); brainstorm to confirm. (Seedqr-slot precedent v0.31.3 shipped as a PATCH-level addition.)
- **Locksteps (in priority order):**
  1. Internal CI parity — `SECRET_SLOT_SUBKEYS` (`secret_taxonomy.rs`) + the `slot_input.rs` parity test (HARD gate, same-PR).
  2. **Paired `mnemonic-gui` PR** — `slot_editor.rs::SlotSubkey` picker + `secrets.rs` secret-snapshot (`feedback_gui_schema_secret_projection_lockstep`; `feedback_manual_gui_lockstep`).
  3. Manual prose `41-mnemonic.md` (+ `make -C docs/manual audit`).
  4. `schema_mirror` — **NOT triggered** (free-form value, confirmed).
- **The one design question for AskUserQuestion:** mnem-ms1 wire-language authority (override `--language` silently w/ stderr note vs refuse-on-conflict) + whether the emitted card re-encodes as `mnem` to preserve language on output (`ResolvedSlot.language`).
- **Constraint check:** read-only public derivation only — this feature decodes secret material to derive an xpub/bundle, NO signing. Within `feedback_no_signing_read_only_derivation_boundary`. ✓
- **Ordering:** standalone; no inter-slug dependency. Mandatory opus R0 on the SPEC + plan + per-phase + end-of-cycle (0C/0I before code), re-dispatch after every fold.
