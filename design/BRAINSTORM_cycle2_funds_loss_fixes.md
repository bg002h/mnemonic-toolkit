# BRAINSTORM — cycle-2 constellation funds-loss fixes (H8 / H10 / H7)

**Status:** BRAINSTORM-SPEC ONLY — no code, no source edits. **Spec R0 round-1 folded** (2 Important +
6 Minor from `…spec-R0-round1.md`) **and round-2 folded** (1 Important I-A + 3 Minor from
`design/agent-reports/cycle2-funds-loss-fixes-spec-R0-round2.md`; see the fold logs in §9);
architect re-dispatch pending per the reviewer-loop. This doc goes through a MANDATORY opus-architect
**R0 review loop to 0 Critical / 0 Important** before ANY implementation begins (CLAUDE.md hard-gate).
Implementation (writing code, dispatching implementer subagents) MUST NOT start until this spec is
R0-GREEN; after each fold, re-dispatch the architect.

**Program:** constellation bug-fix program — cycle-2 = the next funds-loss batch (3 HIGH findings,
all toolkit, no codec-publish dependency).
**Source docs:** `design/agent-reports/constellation-bughunt-2026-06-20.md` (H8/H10/H7 detail +
differential-oracle proof); `cycle-prep-recon-cycle2-h8-h10-h7.md` (verified-LIVE recon);
`design/PLAN_constellation_bughunt_fix_program.md` (workstream zones); `CLAUDE.md`.

**Source SHA (all citations re-grepped against this, NOT the working tree):**
toolkit `origin/master` = **`f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5`** (release 0.61.0, post-cycle-1
H12/H1/H13 merged). The working tree is on another instance's WIP branch
`feature/bundle-md1-template-multisig` and was NOT used for any citation. Companion md repo (no edit
this cycle) `origin/main` = `58cc9ec2…`. Citations below carry LIVE line numbers verified
`git show origin/master:<path>` on 2026-06-21.

---

## 0. Executive summary

Three independent HIGH funds-loss findings, three **disjoint-file** workstreams, one toolkit MINOR
bump:

| WS | finding | one-line | zone (disjoint) | LOC |
|---|---|---|---|---|
| **S-TEMPLATE** | **H8** | `--md1-form=template` hardcodes BIP-39 English → non-English seed re-emits English → WRONG master seed | `synthesize.rs` | ~10–20 + 1 test |
| **WS-EXPORT-MULTISIG** | **H10** | unsorted `multi(...)` silently coerced to BIP-67 `sortedmulti` by field-less electrum/coldcard/jade → WRONG addresses (oracle-proven) | `wallet_export/{coldcard,jade,electrum,mod}.rs` + `error.rs` + `cmd/export_wallet.rs` | ~30–60 + tests |
| **S-VERIFY-LEX** | **H7** | documented prefix-form `[fp/path]@N` origin annotation silently ignored → origin dropped + fp cross-check bypassed | `parse_descriptor.rs` + `cmd/bundle.rs` + `cmd/verify_bundle.rs` | ~40–80 + tests |

**Key design calls (resolved in this spec):**
- **H7 → ACCEPT the prefix form** (alternation in `lex_placeholders` + mirror strip in
  `substitute_synthetic`); it is BIP-380-canonical AND the toolkit's own `--help` advertises it.
  Composes cleanly with cycle-1's H13 multipath reject (orthogonal regex segments).
- **H10 → PURE REFUSAL, no flag** (LOCKED user decision). New typed `ToolkitError` variant (struct-form,
  routing modeled on `ExportWalletTaprootMultisigUnsupported`); refusal lives in the shared pre-`emit`
  dispatch **`emit_payload`** (`export_wallet.rs:73`), **gated on a STRUCTURED predicate** — refuse iff the
  resolved `CliTemplate` ∈ {`WshMulti`,`ShWshMulti`} for {Electrum, Coldcard, ColdcardMultisig, Jade}
  (pinned round-1; §2.3). **Scope = the `--template` path and the `--from-import-json` path** (the two paths
  that resolve a `Some(WshMulti)`/`Some(ShWshMulti)` template). The direct `run --descriptor` path resolves
  `template_opt = None` and is ALREADY funds-safe — refused by each emitter's own generic `BadInput` (no
  silent coercion; round-2 I-A correction, §2.3). Disjoint from the existing taproot guard; never refuses
  `sortedmulti`/`multi_a`.
- **H8 → thread `run_language`** into `synthesize_template_descriptor`; replace the hardcoded English
  fallback with `c.language.unwrap_or(run_language)`, mirroring the keyed path.

**SemVer / lockstep:** toolkit **MINOR** (single bump if batched; current `origin/master` = 0.61.0 →
next MINOR; resolve the exact number at release — the paused own-account cycle is renumbering). **NO
new CLI flag** in any of the three → **NO GUI schema-mirror leg, NO manual flag-table leg**
(confirmed §6). H10's refusal text is the only user-facing string; it changes `--help`-adjacent
behavior, not a flag set.

---

## 1. H8 — S-TEMPLATE: template path drops the BIP-39 wordlist language

### 1.1 Defect (verified LIVE @ `f9467cc5`)

`crates/mnemonic-toolkit/src/synthesize.rs`:

| site | LIVE line | fact |
|---|---|---|
| keyed path (correct) | `:547` | `let emit_lang = c.language.unwrap_or(run_language);` |
| caller `synthesize_descriptor` sig | `:467`, has `run_language` at `:471` | run-level `--language` is in scope |
| call into template path **drops** `run_language` | `:487` | `return synthesize_template_descriptor(descriptor, cosigners, privacy_preserving);` |
| `synthesize_template_descriptor` sig **lacks** language | `:1158-1162` | 3 params: `descriptor, cosigners, privacy_preserving` |
| template ms1 emit **hardcodes English** | `:1265` | `let emit_lang = c.language.unwrap_or(bip39::Language::English);` |

**Why it is funds-loss (BIP-39, CONFIRMED):** seed = PBKDF2-HMAC-SHA512 over the NFKD-normalized
*mnemonic phrase string*, not the raw entropy. The same entropy rendered through a different-language
wordlist yields a different word string → a different 512-bit seed → different keys/addresses. So
re-emitting a non-English seed's ms1 as English silently changes the master seed; a card engraved
under `--md1-form=template` is then unrecoverable.

**Bite-scope nuance (for R0, not a downgrade):** the defect bites only via the `c.language == None`
fallback — the descriptor-`@N` phrase/entropy path where slot language is `None` and run-level
`--language` is the sole carrier (exactly what the `:547` comment calls out). If a slot already
carries `c.language = Some(non-English)` (e.g. an import-json mnem source) the template path emits
correctly today. The fix is the same regardless.

### 1.2 Fix design

1. Extend `synthesize_template_descriptor`'s signature to take `run_language: bip39::Language`
   (mirror the keyed twin's param). Place it consistently with `synthesize_descriptor`'s param
   order (after `privacy_preserving`).
2. Forward it at the `:487` call site:
   `synthesize_template_descriptor(descriptor, cosigners, privacy_preserving, run_language)`.
3. At `:1265` replace `c.language.unwrap_or(bip39::Language::English)` with
   `c.language.unwrap_or(run_language)` — byte-identical to the keyed path `:547`.
4. Update the **3 in-module test call sites** (`:2407`, `:2443`, `:2594`,
   `synthesize_template_descriptor(&descriptor, &cosigners, false)`) to pass a language argument
   (`bip39::Language::English` preserves their current behavior).

No other caller exists (grep-confirmed: only `:487` + the 3 tests). This is a private fn, so the
signature change has no external/CLI surface — **no clap flag, no schema-mirror, no manual.**

### 1.3 Test plan (TDD, RED first)

- **Unit / round-trip anchor (RED→GREEN):** build a Spanish (or Japanese) `CosignerKeyInfo` whose
  `language = None` and entropy set, run-level `run_language = Spanish`, through
  `synthesize_template_descriptor` (or the `synthesize_descriptor` public entry with
  `md1_form = template`). Assert the emitted ms1 decodes (via `ms_codec::decode`) to
  `Payload::Mnem { language: <spanish wire code>, .. }`, NOT `Payload::Entr` and NOT English.
  - **The all-zero-entropy oracle (`1b6aef92` Spanish / `73c5da0a` English) is COMPUTE-don't-hardcode
    (R0-M3).** These master-fp constants are lifted from the bughunt report (`:102`) and were NOT
    re-derived in R0 (no crypto execution in an R0). **The RED assertion is the DIVERGENCE, not the
    exact hex:** assert `spanish_master_fp != english_master_fp` AND that the template path's
    reconstructed fp equals the value the test ITSELF computes from the Spanish phrase (derive both in
    the test from the same all-zero entropy through the two wordlists) — do NOT hard-code an unverified
    hex literal, so a transcription typo cannot make the test vacuously pass. The `1b6aef92`/`73c5da0a`
    pair may be cited as the EXPECTED divergent values for documentation, but the live assertion derives
    them.
- **Regression guard:** an English `run_language` slot still emits `Payload::Entr` (the
  `emit_lang == English` branch at `:1266`), byte-identical to today.
- **Parity assertion (anti-drift):** a test that the template path and keyed path produce the SAME
  ms1 for the same `(entropy, run_language)` input — structurally pins the keyed↔template symmetry so a
  future divergence re-RED's.

### 1.4 Open items folded / deferred

L9 (parallel template-path missing refusals — `has_hardened_use_site` / taproot-override) is a SEPARATE
finding (Tier-3 in the program plan). **DECISION: do NOT fold L9 into cycle-2.** Justification, tightened
per R0-M5 (the earlier "same zone as H8" framing was slightly off): L9's MISSING guards
(`has_hardened_use_site` at `restore.rs:2779`, `taproot_override_card`/`restorable_taproot_override_card`
at `restore.rs:2786`, which `run_multisig_template_completion` at `restore.rs:1321` lacks) live in
**`restore.rs`, a DIFFERENT FILE from H8's `synthesize.rs`** — only the use-site *preservation* is in
`synthesize.rs`. And L9 (a) **fail-safes to NO-MATCH**, not a wrong-address emit, so it is NOT
funds-loss-equivalent to H8's silent wrong-seed; (b) folding it would cross into `restore.rs` and widen
the H8 diff/test surface beyond the single private-fn signature change. Keep cycle-2 = the 3 named HIGHs.
(R0 may revisit; see §8 Q4.)

---

## 2. H10 — WS-EXPORT-MULTISIG: unsorted `multi` silently exported as `sortedmulti`

### 2.1 Defect (verified LIVE @ `f9467cc5`)

The electrum / coldcard(-multisig) / jade multisig file formats have **no sorted-vs-unsorted field**;
all three reconstruct via BIP-67 `sortedmulti` unconditionally. The toolkit models the distinction
(`CliTemplate::WshMulti` ≠ `WshSortedMulti`, `ShWshMulti` ≠ `ShWshSortedMulti`;
`template.rs:27-36`) but loses it at these three emitters:

| site | LIVE line | fact |
|---|---|---|
| coldcard multisig emitter | `wallet_export/coldcard.rs:258-370` | `emit_coldcard_multisig_text` writes `Name:/Policy:/Derivation:/Format:/<XFP>:` — NO sorted/unsorted field; lex-sorts only for `*SortedMulti` |
| jade emitter | `wallet_export/jade.rs:43-46` | `WshMulti`/`ShWshMulti` (+sorted) delegate byte-identically to coldcard → inherits the defect |
| electrum emitter | `wallet_export/electrum.rs:131-191` | `emit_electrum_multisig_json` writes `x1/x2/…` in slot order; Electrum BIP-67-sorts on load; no field |
| dispatch | `cmd/export_wallet.rs:120-128` (`ColdcardMultisig` arm) + `:119-143` `match format` | accepts `WshMulti \| ShWshMulti` for these formats with no sorted-vs-unsorted refusal |

**Why it is funds-loss (BIP-67, CONFIRMED + differential-oracle-PROVEN):** `sortedmulti` is a
deterministic lexicographic sort of compressed pubkeys, so its witnessScript/address DIFFERS from
literal-order `multi(...)` whenever the per-index derived pubkeys are not already in BIP-67 order. The
oracle wave proved the address divergence empirically (`wsh-multi-2of3-divergent` row exists in
`tests/bitcoind_differential.rs:115`). **Faithful-format contrast (verified):** `descriptor.rs`
(`:20` passes `inputs.canonical_descriptor` verbatim), `bitcoin_core.rs`, `sparrow.rs`, `bip388.rs`
carry the literal `multi(`/`sortedmulti(` token → preserve the distinction.

### 2.2 LOCKED decision — PURE REFUSAL, no flag

Per the user's locked decision: refuse unsorted `wsh-multi`/`sh-wsh-multi` for the
electrum/coldcard(-multisig)/jade formats with a clear typed error that points to the faithful
formats (descriptor / sparrow / bitcoin-core, all of which preserve `multi` order). Do **NOT** add
`--allow-sortedmulti-coercion` (confirmed absent on `f9467cc5`; adding it would drag the GUI
schema-mirror flag-NAME gate + the manual flag table — explicitly avoided).

### 2.3 Where the refusal lives — DESIGN CALL

**DECISION: refuse in the shared `emit_payload` dispatch in `cmd/export_wallet.rs` (`:73`), BEFORE the
per-format `emit`, gated on `format ∈ {Electrum, Coldcard, ColdcardMultisig, Jade}` ∧ "resolved
`CliTemplate` ∈ {`WshMulti`,`ShWshMulti`}".** (The dispatch fn is `emit_payload`, NOT `emit_for_format` —
round-2 M-1 name correction.) Rationale:

1. **Single chokepoint — and `emit_payload` serves FOUR callers, not two (round-2 M-1).** The `--template`
   path (`run`, `:625` calls `emit_payload`) and the `--from-import-json` path (`run_from_import_json`,
   `:845` calls `emit_payload`) both route through the shared `emit_payload` dispatch — whose doc-comment (`:60-72`)
   states it dedups the formerly-4 byte-identical copies: `run`, `run_from_import_json`, **and restore's
   `build_import_payload` (`restore.rs:2150`/`:2190`) + `build_multisig_import_payload`
   (`restore.rs:2394`/`:2479`)**. The `emit` `match format` is at `export_wallet.rs:109` (round-2 M-2: the
   live `emit` match opens `:109`; `collect_missing` match `:82-101`). One guard there covers the
   template/import-json entry paths in a single place; per-emitter guards would need duplicating across
   coldcard/jade/electrum.
   - **Restore-path coverage is a CONSEQUENCE of the chokepoint placement, and it is funds-safe (round-2
     M-1).** Because `restore`'s `build_multisig_import_payload` passes `template: Some(t)`
     (`restore.rs:~2457`, which CAN be `Some(WshMulti)`) into `emit_payload`, the new guard ALSO fires on
     `restore --md1 --format electrum/coldcard/jade` for an unsorted-multi md1 — a desirable funds-safety
     extension (more refusals, never fewer; the same silent-sortedmulti coercion would otherwise bite the
     restore output). This does NOT break workstream file-disjointness — the guard lives in
     `export_wallet.rs::emit_payload`; `restore.rs` is NOT edited, it merely calls the shared fn. **The
     plan-doc MUST add a one-line `restore` regression note** pinning the intended behavior (restore of an
     UNSORTED `WshMulti` md1 to a field-less vendor is refused, not silently coerced); H10's behavioral
     tests remain scoped to `export-wallet` plus this one restore-path assertion.
2. **Detect on the resolved `CliTemplate` (STRUCTURED), refuse the unsorted-multi variants.**
   **PINNED predicate (was R0-Q2, now DECIDED): refuse iff the resolved `CliTemplate` is
   `WshMulti` or `ShWshMulti` (the two unsorted-multi variants) AND the format is one of the three
   field-less vendors.** Do **NOT** refuse `WshSortedMulti`/`ShWshSortedMulti` (BIP-67 — what these
   formats implement), `TrMultiA`/`TrSortedMultiA` (taproot — refused independently, see §2.5), or any
   single-sig variant. This is a structural check on the typed enum, not a string scan — structurally
   immune to the `sortedmulti(`-as-substring false-match that a naive `.contains("multi(")` would hit.

   **`run` has TWO descriptor-bearing sub-paths and they resolve the template DIFFERENTLY — the
   round-2 I-A control-flow correction.** The round-1 fold over-corrected: it claimed the resolved
   `CliTemplate` is non-`None` on EVERY refusal-target path "via `template_from_descriptor`." **That is
   FALSE for the direct `run --descriptor` path and is removed.** Verified LIVE @ `f9467cc5`,
   `template_from_descriptor` is called in EXACTLY ONE place — `run_from_import_json:812` — and NEVER in
   `run` (grep-confirmed; `format_requires_template` is consulted only at `run_from_import_json:791`).
   The three paths that reach the field-less emitters resolve the template thus:
   - **`--template wsh-multi`/`sh-wsh-multi` (`run`):** the template branch sets
     `resolved_template = Some((slots, WshMulti, k))` (`export_wallet.rs:542`) → `template_opt =
     Some(WshMulti)` (`:553-560` region). **Caught by the structured guard.** ✓
   - **`--from-import-json` unsorted `wsh(multi)`/`sh(wsh(multi))` (`run_from_import_json`):**
     `template_from_descriptor` (`export_wallet.rs:812`, gated on `format_requires_template`) derives a
     non-`None` template — and **preserves the distinction**:
     `let is_sorted = d.to_string().contains("sortedmulti(")` → `WshMulti` vs `WshSortedMulti`,
     `ShWshMulti` vs `ShWshSortedMulti` (`wallet_export/mod.rs:259-290`). **Caught by the structured
     guard.** ✓
   - **direct `run --descriptor 'wsh(multi(…))'` (concrete inline keys; `@N` forms rejected at
     `export_wallet.rs:443`):** the `if let Some(desc) = &args.descriptor` arm builds `canonical` but
     does **NOT** set `resolved_template` (it stays `None`), so `template_opt = None`
     (`:558-560` region). `template_from_descriptor` is NOT called here. **The new structured guard
     (matching `Some(WshMulti)`/`Some(ShWshMulti)`) does NOT fire** — and it does not need to: this path
     is ALREADY funds-safe, refused by each emitter's own `inputs.template.ok_or_else(…)` generic
     `BadInput` at the top of `emit` (electrum `electrum.rs:50-54` `"--format electrum requires
     --template; descriptor passthrough is not supported …"`; jade `jade.rs:36-40`; coldcard via
     `emit_coldcard_generic_json`'s `None`→`ok_or_else` at `coldcard.rs:111-114`; coldcard-multisig the
     `_ =>` arm at `export_wallet.rs:129-132` `"--format coldcard-multisig requires a multisig
     --template …"`). An unsorted `multi` on this path is **REFUSED, never silently coerced to
     sortedmulti** — so the funds-safety hole does not exist here. The only difference from the typed
     path is that this refusal is the untyped generic `BadInput` rather than the new
     `ExportWalletUnsortedMultisigUnsupported`; it does not name a faithful alternative format. **DECISION
     (round-2 I-A option (i), minimal): keep the direct `--descriptor` path on its existing generic
     refusal** — funds-safe, in scope of no silent coercion. The new typed error covers ONLY the
     `Some(WshMulti)`/`Some(ShWshMulti)` `--template` + `--from-import-json` paths. *(Optional FOLLOWUP —
     NOT this cycle: upgrade the direct-`--descriptor` generic message to the typed error by classifying
     the `template == None` direct path via the parsed descriptor (`Tag::Multi` / `script_type`); the
     spec deliberately does NOT scope that strictly-larger change. See §5.1.)*
   - *(Why structured beats the string form here: `template_from_descriptor` has already done the
     `multi(`-vs-`sortedmulti(` discrimination once on the import-json path; refusing on its typed
     output reuses that single classification instead of re-parsing the descriptor string. The
     house-style string precedent — `multi(` NOT preceded by `sorted`, per `wallet_export/mod.rs:264`'s
     `contains("sortedmulti(")` and `:207`/`:237`'s `multi_a(`/`sortedmulti_a(` checks — remains the
     documented FALLBACK only if the direct-`--descriptor` path is later upgraded to a typed refusal
     (the optional FOLLOWUP above); for THIS cycle the structured form is pinned and complete for the
     `--template` + `--from-import-json` paths. Either form MUST refuse unsorted `wsh-multi` AND
     `sh-wsh-multi`, and MUST NOT refuse `sortedmulti` / `multi_a` / `sortedmulti_a`.)*

> **Note for the implementer:** `format_requires_template` (`export_wallet.rs:53-59`) returns `true`
> for Sparrow/Coldcard/ColdcardMultisig/Jade/Electrum — Sparrow is faithful and must NOT be refused, so
> the refusal set is the THREE field-less formats {Electrum, Coldcard, ColdcardMultisig, Jade}, not
> `format_requires_template`. (`Coldcard` singlesig never carries a multisig descriptor, but gating it
> in is harmless and future-proofs the alias.) R0 Q3: confirm `coldcard` (the generic single-sig
> alias) cannot reach an unsorted-multi descriptor — if it can only via `coldcard-multisig`, scope the
> set to {Electrum, ColdcardMultisig, Jade}.
>
> **Guard ordering vs taproot (was R0-Q6, now RESOLVED by the structured predicate):** because the
> pinned predicate matches ONLY the `WshMulti`/`ShWshMulti` variants, it is structurally
> `multi`-specific and CANNOT match `TrMultiA`/`TrSortedMultiA`. A taproot-multisig shape therefore
> passes this guard untouched and reaches its existing per-emitter taproot refusal (jade `:48-52`,
> coldcard `:268` — the `if matches!` taproot block in `emit_coldcard_multisig_text` (fn `:258`,
> `None`-guard `:261-263`, taproot block `:268-277`); round-2 M-2 citation refresh from the stale
> `:266-276` — both verified) exactly as today. No ordering hazard: the new guard and the
> taproot guard match disjoint variant sets, so neither can shadow the other. (On the passthrough path,
> taproot is additionally refused upstream at the EmitInputs gate before `template_from_descriptor`, so
> a `tr-multi-a` never even reaches the resolved-`CliTemplate` site.)
>
> **No `multi_a` clause in the predicate (R0-M1):** because the predicate matches only the two
> unsorted-`WshMulti`/`ShWshMulti` variants, there is nothing to "exclude `multi_a` from" — taproot is
> never a candidate. The implementer must NOT over-engineer a tree-walk or a `multi_a(` carve-out solely
> to handle taproot: it is already refused upstream (passthrough) and by the per-emitter guards (direct
> path), so the `multi_a`/`sortedmulti_a` family is structurally out of this guard's scope.

### 2.4 Error variant — REUSE vs new

**DECISION: add ONE new typed variant** modeled on the existing precedent
`ExportWalletTaprootMultisigUnsupported(&'static str)` (`error.rs:169`, exit 2, kind
`"ExportWalletTaprootMultisigUnsupported"`, message routed through a `wallet_export` helper). The new
variant:

```
/// SPEC cycle-2 H10 — the electrum / coldcard / jade multisig file formats are
/// BIP-67 sortedmulti-only (no field to express literal `multi(...)` key order),
/// so exporting an UNSORTED `wsh-multi` / `sh-wsh-multi` to them would silently
/// coerce to sortedmulti → different witnessScript/address. Refuse, pointing to
/// a faithful format (descriptor / bitcoin-core / sparrow). Exit 2.
ExportWalletUnsortedMultisigUnsupported { format: &'static str },
```

- **Why a NEW variant, not `BadInput`:** the toolkit's convention is typed export refusals with a
  stable `kind()` string (every other export refusal has one); `BadInput` would be untyped and
  un-greppable in tests/GUI. Modeled on the taproot precedent (same exit 2, same `wallet_export`-helper
  message routing, same alphabetical cluster) keeps the audit trail uniform.
- **Variant shape — DELIBERATE struct form, NOT 1:1 with the precedent's tuple (R0-M4 note).** The
  precedent is the tuple `ExportWalletTaprootMultisigUnsupported(&'static str)`; this new variant uses
  the named-field struct form `{ format: &'static str }` for call-site readability
  (`ExportWalletUnsortedMultisigUnsupported { format: "electrum" }` self-documents the payload). This is
  an intentional shape divergence, not a transcription of the tuple — the plan-doc must NOT describe it
  as "modeled 1:1" on the precedent's shape. The exhaustive `match` arms use `{ .. }` / `{ format }`
  binding accordingly (mirror the existing `ExportWalletMissingFields { .. }` struct-variant arms at
  `error.rs:543/605/745`, which already establish the struct-variant arm style). (Implementer's choice:
  if matching the tuple form is preferred for minimal diff against the taproot precedent, that is
  equally acceptable — pin ONE in the plan-doc; this spec recommends the struct form.)
- **Alphabetical-ordering rule (CLAUDE.md):** new `ToolkitError` variants + their exhaustive
  `match self` arms (`exit_code`, `kind`, `message`/`user_text`) use alphabetical-by-variant-name
  ordering. `ExportWalletUnsortedMultisigUnsupported` sorts AFTER
  `ExportWalletTaprootMultisigUnsupported` and BEFORE `FutureFormat` — insert at that position in the
  enum and in EACH match block. *(The pre-v0.27.2 variants are not yet sorted — FOLLOWUP
  `error-rs-retroactive-alphabetical-sort` — but new variants still follow the rule relative to the
  v0.27.2+ alphabetical region; place it adjacent to the existing `ExportWallet*` cluster, which is
  already locally alphabetical.)*
- **Exit code 2** (mirror the taproot precedent and every export refusal).
- **Message** (byte-exact, in a `wallet_export` helper called from the `user_text()`/`message()`
  arm), e.g.:
  > `--format <format> cannot faithfully export an UNSORTED multisig (wsh-multi / sh-wsh-multi): the <format> multisig file format is BIP-67 sortedmulti-only and would silently reorder the keys, changing the witnessScript and every address. Use --format descriptor, --format bitcoin-core, or --format sparrow (which preserve literal multi(...) key order), or use a sortedmulti template if BIP-67 ordering is intended.`

  `<format>` = the offending format name (`electrum` / `coldcard-multisig` / `jade`). R0 Q5: finalize
  the byte-exact string + whether to name all three faithful formats or just `descriptor`.

### 2.5 Behavior to PRESERVE (anti-over-refusal)

- `sortedmulti` (`WshSortedMulti`/`ShWshSortedMulti`) → these three formats: still exports (BIP-67 is
  exactly what they implement). **Do not refuse.**
- Single-sig (`bip44/49/84`) → these formats: unaffected (no multisig token). **Do not refuse.**
- `descriptor` / `sparrow` / `bitcoin-core` / `bip388` ← unsorted `multi`: still allowed (faithful).
  **Do not refuse.**
- Taproot multisig (`tr-multi-a`/`tr-sortedmulti-a`) → electrum/coldcard/jade: already refused by the
  existing per-emitter taproot guards (jade `:48-52`, coldcard `:268` — round-2 M-2 refresh from `:266-276`;
  both verified) — and the new
  guard composes cleanly with them because the pinned structured predicate matches ONLY
  `WshMulti`/`ShWshMulti`, a variant set DISJOINT from `TrMultiA`/`TrSortedMultiA`. The taproot refusal
  therefore still fires (independently, not "first" — there is no shadowing) for taproot shapes; the new
  guard never matches them. (Resolves former R0-Q6; see the guard-ordering note in §2.3.)

### 2.6 Test plan (TDD, RED first)

- **Behavioral CLI (RED→GREEN):** `export-wallet --format electrum --template wsh-multi --threshold 2
  --slot @0.xpub=… --slot @1.xpub=…` → exit ≠ 0 (= 2), stderr contains the typed refusal naming a
  faithful format. Repeat for `coldcard-multisig` and `jade`. Repeat for `sh-wsh-multi`.
- **Direct-`--descriptor`-path coverage (round-2 I-A test correction — was asserting the wrong error):**
  `export-wallet --format electrum --descriptor 'wsh(multi(2,…))…'` (no explicit `--template`) → exit ≠ 0
  refused by the **existing emitter-level generic `BadInput`** (`"--format electrum requires --template;
  descriptor passthrough is not supported …"`, `electrum.rs:50-54`), **NOT** the new typed
  `ExportWalletUnsortedMultisigUnsupported`. Per the §2.3 I-A control-flow correction, this direct path
  resolves `template_opt = None` (`template_from_descriptor` is NOT called in `run` — only in
  `run_from_import_json:812`), so the structured guard does not fire; the funds-safety property is that the
  unsorted `multi` is **refused, never silently coerced**. **The test asserts refusal by ANY error
  (exit ≠ 0)** — and explicitly that the `kind()` is NOT `ExportWalletUnsortedMultisigUnsupported` (it is
  the generic `BadInput`), matching reality. (The typed-error path is covered separately: the
  `--from-import-json` test below exercises `template_from_descriptor → Some(WshMulti)` → the new typed
  refusal.)
- **`--from-import-json`-path typed-refusal coverage:** an `import-wallet --json` envelope whose descriptor
  is an unsorted `wsh(multi(2,…))` (or `sh(wsh(multi(…)))`), then `export-wallet --from-import-json <env>
  --format electrum` (repeat coldcard-multisig/jade) → exit 2 with the **typed**
  `ExportWalletUnsortedMultisigUnsupported` (assert `kind()`). This is the path where
  `template_from_descriptor` (`export_wallet.rs:812`, `wallet_export/mod.rs:259-290`) derives
  `Some(WshMulti)`/`Some(ShWshMulti)`, so the structured guard fires. Pins the typed refusal on the second
  entry path.
- **Restore-path regression note (round-2 M-1):** `restore --md1 --format electrum` (or coldcard/jade) of
  an md1 reconstructing an UNSORTED `WshMulti` → exit 2 with the typed
  `ExportWalletUnsortedMultisigUnsupported` (the guard fires via the shared `emit_payload` chokepoint that
  `restore`'s `build_multisig_import_payload` calls, `restore.rs:2479`, passing `template: Some(WshMulti)`).
  Intended behavior: bonus funds-safety coverage, refused not silently coerced. One-line assertion; H10's
  diff still does NOT edit `restore.rs`.
- **MANDATED `sortedmulti`-NOT-refused regression (false-refuse guard, RED-stays-GREEN):** the
  funds-safety-critical anti-over-refusal test — exporting a SORTED shape to each field-less vendor
  STILL SUCCEEDS (exit 0):
  - `--format electrum --template wsh-sortedmulti` → exit 0; repeat for `coldcard-multisig` and `jade`.
  - `--format electrum --template sh-wsh-sortedmulti` → exit 0; repeat for `coldcard-multisig`/`jade`.
  - taproot-multisig sorted/unsorted (`tr-multi-a` / `tr-sortedmulti-a`) → these three formats: hits the
    EXISTING taproot refusal, NOT the new unsorted-multi error (assert the error `kind()` is
    `ExportWalletTaprootMultisigUnsupported`, proving disjointness per §2.3/§2.5). This is the
    `multi_a`/`sortedmulti_a`-not-refused-by-the-new-guard proof.
  This test is the explicit guard the R0 review (I1) demands: it RED's immediately if a future predicate
  drift (e.g. a naive `.contains("multi(")` that false-matches `sortedmulti(`) starts refusing sorted
  shapes.
- **Other negatives / must-still-work:** `--format descriptor --template wsh-multi` → exit 0 and emits
  the literal `multi(`; `--format sparrow --template wsh-multi` → exit 0 (faithful); single-sig
  `--format coldcard --template bip84` → exit 0.
- **Differential-oracle (gate, optional value-add):** `tests/bitcoind_differential.rs` already has
  `wsh-multi-2of3-divergent` (`:115`). H10 is **exit-code-behavioral**, so the primary gate is the CLI
  refusal test; do NOT add an oracle row that EXPORTS to electrum (the refusal means no file is
  emitted). The oracle's existing divergent row already proves the underlying address divergence that
  motivates the refusal — cite it, don't duplicate.

### 2.7 Lockstep confirmation

Pure refusal adds **NO clap flag** → `mnemonic-gui/src/schema/mnemonic.rs` `schema_mirror` flag-NAME
gate is **untouched** (no GUI leg). Manual: the refusal is a new error condition, not a flag — at most
an OPTIONAL prose note under `docs/manual/src/40-cli-reference/` describing the refusal; this is NOT
gated by `docs/manual/tests/lint.sh` (which checks flag coverage, not error text). **DECISION: a
one-line manual note is courtesy, not required; defer to the plan-doc.** (R0 Q7.)

---

## 3. H7 — S-VERIFY-LEX: prefix-form `[fp/path]@N` origin annotation silently ignored

### 3.1 Defect (verified LIVE @ `f9467cc5`, drifted +~13/+34 by cycle-1)

`crates/mnemonic-toolkit/src/parse_descriptor.rs`:

| site | LIVE line | fact |
|---|---|---|
| `lex_placeholders` regex | `:82-84` | `@(\d+)(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'\|h)?)*)\])?(?:/<([^>]*)>)?(/\*(?:'\|h)?)?` — the `[fp/path]` block (caps 2/3) is anchored ONLY AFTER `@(\d+)` (SUFFIX form). Prefix `[fp/path]@N` → caps 2/3 = `None`, bracket dropped |
| `substitute_synthetic` strip regex | `:369-371` | `@(\d+)(?:\[[0-9a-fA-F]{8}(?:/\d+(?:'\|h)?)*\])?(?:/<[0-9;]+>)?(?:/\*(?:'\|h)?)?` — strips only the SUFFIX bracket; a leading `[fp/path]` LEAKS into the descriptor handed to `Descriptor::from_str` → md1/mk1 |
| bundle fp-guard read | `bundle.rs:1581-1582` | `let anno_fp = resolved_placeholders.fingerprint_annos[idx as usize];` — prefix form ⇒ `None` |
| bundle fp cross-check (BYPASSED) | `bundle.rs:1617-1620` | `if let Some(anno) = anno_fp { if anno != master_fp { Err … } }` — `None` ⇒ guard never entered ⇒ master-fp funds-safety cross-check skipped |
| verify-bundle shared lexer | `verify_bundle.rs:1342`, `:1346` | `lex_placeholders(&descriptor_str)` + `resolve_placeholders(&occs)` — SAME lexer ⇒ same prefix mis-parse; verify-bundle has no compensating per-@N fp check, so it inherits the dropped origin identically |

**Why it is funds-loss (BIP-380, CONFIRMED):** BIP-380 defines key-origin as a **PREFIX**
`[fingerprint/derivation/path]KEY` — the bracketed origin appears BEFORE the key. So `[fp/path]@N` is
the BIP-380-canonical position; the toolkit's `@N[fp/path]` suffix is non-canonical. A user/tool
following the standard writes the prefix form → the origin path is silently dropped (slot xpub built
at the master/default path → backup watches a different address set) AND the per-@N master-fingerprint
cross-check (a funds-safety guard) is bypassed.

### 3.2 ACCEPT vs REJECT — THE KEY R0 DESIGN CALL → **ACCEPT**

**DECISION: ACCEPT the prefix form** (extend the lexer to capture a leading `[<8hex>(/path)?]`
immediately before `@N`, populating `fingerprint_anno`/`origin_path_anno` identically to the suffix
form; mirror the strip in `substitute_synthetic`). Rationale, grounded in LIVE source:

1. **BIP-380-canonical.** `[fingerprint/path]KEY` is the standard's origin position (CONFIRMED). The
   suffix form the toolkit currently accepts is the non-canonical one. Rejecting the canonical form
   inverts the standard.
2. **The toolkit's OWN `--help` advertises the prefix form.** `bundle.rs:2300` (LIVE):
   `"… Override per-placeholder with [fp/path]@N or --slot @N.path=m/…"` — the help text documents the
   prefix form to users, while the lexer silently ignores it. Rejecting would mean either breaking the
   documented contract or rewriting the help; ACCEPT honors what is already promised.
3. **`bundle.rs:1516` Row-19 logic** comments "if inline `[fp/path]@N` AND `--slot @N.path=` both
   supplied … differ → refuse" — the codebase's own override-conflict logic is written as if the
   prefix form already works. ACCEPT makes that comment true.
4. **No existing test regresses.** ALL 20 `lex_placeholders` test call sites + every internal producer
   use the SUFFIX form `@N[fp/path]` (verified: `parse_descriptor.rs:1378/1433/1863/…`,
   `wallet_import/pipeline.rs:312-314` deliberately emits `@N[fp/path]` "to feed the toolkit's
   lexer"). Adding a prefix ALTERNATION keeps every suffix case matching unchanged — the prefix branch
   is purely additive. (REJECT, by contrast, would require either touching the help text or adding a
   new error path with no test demand.)
   - **Clarifying note for R0 (acuteness-claim correction):** the prefix-form occurrences at
     `parse_descriptor.rs:329`, `:2989/:2992`, `:3054` are in the **taproot-detection** path
     (`detect_bare_tr` / `substitute_nums_sentinel`), NOT the `lex_placeholders` annotation path —
     those functions already recognize `[fp/path]@N` for NUMS/bare-tr purposes. So the codebase
     ALREADY parses the prefix bracket elsewhere; only the origin-annotation lexer is suffix-only.
     This strengthens ACCEPT (internal precedent) and corrects the recon's "tests use prefix" framing:
     the lexer's own tests use suffix; the prefix appears in a sibling detector.

**If R0 prefers REJECT:** the justification would have to be (a) rewrite `bundle.rs:2300` help to the
suffix form, (b) add a typed `DescriptorParse` "prefix-form origin annotation unsupported; use
`@N[fp/path]`" error, (c) accept that the toolkit then refuses the BIP-380-canonical position. This
spec judges ACCEPT strictly superior (honors the standard + the existing help + zero test churn) and
recommends it. See §8 Q1.

### 3.3 Fix design — ACCEPT, composing with cycle-1 H13

**`lex_placeholders` regex (`:82-84`)** — add a prefix-origin ALTERNATION. Two viable shapes for R0
to pick (Q8):

- **(a) Optional prefix group + keep the suffix group:** prepend an optional
  `(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?` BEFORE `@(\d+)`, with NEW capture indices, then in the
  parse below take fingerprint/path from WHICHEVER of {prefix, suffix} is present. **Refuse if BOTH a
  prefix AND a suffix bracket are present on the same `@N`** (ambiguous double-origin) with a typed
  `DescriptorParse` error — funds-safety: never silently pick one.
- **(b) Single regex with both positions** — more compact but capture-index bookkeeping is error-prone
  against the existing groups 4/5. Recommend (a) for readability and a clean "both present → refuse"
  guard.

**Composition edges — DECIDED (was R0-I2 / prompt H7(d); these MUST be pinned, not left to implementer
discretion on a funds-safety annotation):**

- **(i) `8-hex fingerprint is MANDATORY inside the prefix bracket` — prefix-path-but-no-fp ⇒ REJECT
  (typed error), mirroring the suffix.** BIP-380 key-origin is `[fingerprint/path]` with the fingerprint
  REQUIRED. The pinned prefix regex caps-1 is `([0-9a-fA-F]{8})` (exactly 8 hex, NOT optional) —
  byte-mirroring the suffix grammar at `parse_descriptor.rs:84`, whose inner `[0-9a-fA-F]{8}` is
  required whenever the `[…]` bracket is present (the outer `(?:\[…\])?` only makes the WHOLE bracket
  optional, never the fp inside it). So a prefix bracket lacking a valid 8-hex fp (e.g.
  `[/84'/0'/0']@0` — path, no fp) does NOT match the prefix alternation and is **rejected as malformed**,
  the SAME outcome as the suffix form `@0[/84'/0'/0']`. **Rule: the prefix form requires the 8-hex
  fingerprint; a path-only bracket is rejected (typed `DescriptorParse`), never silently accepted as a
  bare `@N` with the bracket dropped.** The plan-doc must NOT relax caps-1 to an optional fp, which would
  diverge prefix from suffix.
- **(ii) prefix-fp vs `--slot @N.fingerprint=` precedence — if BOTH supplied, they MUST AGREE; mismatch
  ⇒ REFUSE.** This mirrors the suffix form's existing per-`@N` fp cross-check at `bundle.rs:1616-1620`
  (`if let Some(anno) = anno_fp { if anno != master_fp { Err(DescriptorParse …) } }`) and the Row-19
  inline-path-vs-`--slot @N.path=` conflict precedent at `bundle.rs:1516-1525`
  (`new_paths[idx] != user_origin → SlotInputViolation "path-mismatch"`). Once the lexer populates
  `fingerprint_annos[idx]` for the prefix form (identical to the suffix), the EXISTING `:1616-1620`
  cross-check fires UNCHANGED — it compares the annotation against the derived master fp on the phrase
  path. The composition concern is the explicit-`--slot @N.fingerprint=` (xpub-slot) case: an
  `@N.fingerprint=` subkey is parsed at `bundle.rs:603-607` into `ResolvedSlot.fingerprint`; a prefix-fp
  annotation that DISAGREES with an explicitly supplied `--slot @N.fingerprint=` must error, not
  silently let one win. **Rule: prefix-fp annotation and an explicit `--slot @N.fingerprint=` for the
  same `@N` must be equal; on mismatch, refuse with the same fp-mismatch `DescriptorParse` shape as
  `:1618-1620`** (mirroring how Row-19 refuses an inline-path vs `--slot @N.path=` disagreement).
  **The plan-doc MUST take the "add the explicit comparison" branch — the existing `:1616-1620`
  cross-check does NOT cover the xpub-slot case (round-2 M-3, verified LIVE):** that cross-check lives
  ONLY inside the phrase/entropy arm (`if let Some(anno) = anno_fp { if anno != master_fp { … } }` at
  `bundle.rs:1616-1620`), where a `master_fp` is derived. The **xpub-slot arm** (`bundle.rs:1637`, the
  `else if subkeys.contains(&…::Xpub)` branch) computes its `fp` as the explicit `--slot
  @N.fingerprint=` **`.or(anno_fp)`** at `bundle.rs:1654` with **NO equality check** — so today a
  prefix-anno fp that DISAGREES with an explicit `--slot @N.fingerprint=` is **silently resolved to the
  slot value** (the `.or()` only falls through when the slot fp is absent). The "confirm existing covers
  it" branch is therefore UNAVAILABLE for xpub slots; the implementer MUST add an explicit
  `prefix-anno-fp vs --slot @N.fingerprint=` equality check (refuse on mismatch) at the `:1654` site and
  a test pins it (§3.4). The already-specified "both a prefix AND a suffix origin bracket on the same
  `@N` → refuse" guard (shape (a) above) stands independently of this.

**CRITICAL CONSTRAINT — preserve cycle-1 H13's hardened-multipath reject.** The H13 commit
(`080ac03e` + `1e1e3f3d`) rewrote ONLY the **group-4 multipath segment** (`(?:/<([^>]*)>)?`, captured
permissively then strictly validated at `:124-152` to REJECT hardened `<0';1'>` / malformed
`<0'';1>`). The key-origin `[fp/path]` block (groups 2/3) is byte-for-byte unchanged from the original
suffix-only design. **These segments are ORTHOGONAL:** the origin block uses `[...]` brackets; the
multipath uses `<...>` brackets — they cannot overlap. Adding a prefix `[...]` alternation touches
only the origin-capture segment and leaves the H13 multipath capture + its strict validator at
`:124-152` byte-identical. **The plan-doc MUST assert (and a test MUST pin) that the H13 hardened
reject still fires** for both `@0/<0';1'>/*` AND a prefix-annotated `[fp/path]@0/<0';1'>/*`.

**`substitute_synthetic` strip regex (`:369-371`)** — mirror the prefix alternation in the strip so a
leading `[fp/path]` is removed before `Descriptor::from_str`, identically to the suffix bracket. The
strip's multipath class stays `[0-9;]+` (the C1 narrow class — do NOT widen; the lexer rejects
markers first). The strip change is symmetric with the lex change.

**`resolve_placeholders` / `fingerprint_annos` / `bundle.rs` fp-guard** — NO change needed beyond the
lexer: once `lex_placeholders` populates `fingerprint_anno`/`origin_path_anno` for the prefix form,
`resolve_placeholders` (`:175-247`), the `fingerprint_annos` vector (`:242-243`), and the bundle
guard (`bundle.rs:1581-1620`) consume them identically. The fix is localized to the two regexes; the
downstream funds-safety guard then fires for prefix exactly as for suffix.

**verify-bundle** — inherits the fix automatically (shares `lex_placeholders` at
`verify_bundle.rs:1342`). Add a verify-bundle test to pin it (it has no compensating per-@N guard, so
the lexer fix is its only protection).

### 3.4 Test plan (TDD, RED first)

- **Lex parity (RED→GREEN), unit:** `lex_placeholders("wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)")` →
  `occs[0].fingerprint_anno == Some(deadbeef)`, `origin_path_anno == Some(m/84'/0'/0')` — identical to
  the suffix `wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)`. (The existing suffix test at `:1863` is the
  oracle.)
- **Strip parity:** `substitute_synthetic` of the prefix form yields the SAME bare-xpub descriptor as
  the suffix form (no leaked bracket).
- **fp cross-check fires (funds-safety):** `bundle --slot @0.phrase=<seed> --descriptor
  "wpkh([WRONGFP/84'/0'/0']@0/<0;1>/*)"` → exit ≠ 0 with the fp-mismatch error
  (`bundle.rs:1620`), identical to the suffix form. Today the prefix form exits 0.
- **Composition edge (i) — prefix path-but-no-fp ⇒ same rejection as suffix (was R0-I2):**
  `lex_placeholders("wpkh([/84'/0'/0']@0/<0;1>/*)")` (path-only bracket, no 8-hex fp) → SAME error as
  the suffix path-only form `wpkh(@0[/84'/0'/0']/<0;1>/*)` (the bracket fails the mandatory `{8}`-hex
  caps-1, so it does not match the prefix alternation → malformed-descriptor error), NOT exit-0 with the
  bracket silently dropped. Pins that the prefix form does not relax the mandatory fingerprint.
- **Composition edge (ii) — prefix-fp vs `--slot @N.fingerprint=` must agree (was R0-I2; round-2 M-3:
  the xpub-slot case needs an ADDED explicit comparison):**
  `bundle --slot @0.xpub=<xpub> --slot @0.fingerprint=<FP_A> --descriptor
  "wpkh([FP_B/84'/0'/0']@0/<0;1>/*)"` with `FP_A != FP_B` → exit ≠ 0 with an fp-mismatch refusal; and the
  agreeing case (`FP_A == FP_B`) exits 0. Pins that a prefix-fp annotation and an explicit `--slot
  @N.fingerprint=` cannot silently disagree. **This is an xpub-slot scenario, and the existing
  `bundle.rs:1616-1620` phrase-arm cross-check does NOT reach it** (the xpub-slot arm at `:1637` resolves
  `fp` via `--slot @N.fingerprint=` `.or(anno_fp)` at `:1654` with no equality check — round-2 M-3), so
  the plan-doc MUST add the explicit comparison at the `:1654` site (refuse on mismatch, same
  `DescriptorParse` shape as `:1618-1620`; modeled on the Row-19 `:1516-1525` path-conflict precedent).
  Without the added check this test currently RED's (today the prefix-anno fp is silently overridden by
  the `--slot` value). The plan-doc cites the ADDED comparison (not "existing covers it" — that branch is
  unavailable for xpub slots).
- **Round-trip prefix ≡ suffix → identical md1/mk1:** `bundle` of the prefix form and the suffix form
  (same fp/path) produce byte-identical md1 + mk1 cards.
- **verify-bundle:** the prefix form carries the origin through verify-bundle's reparse (pin via a
  verify-bundle test exercising `:1342`).
- **H13 NON-REGRESSION (CRITICAL):** `lex_placeholders("wsh(multi(2,@0/<0';1'>/*,@1/<0';1'>/*))")`
  STILL errors (`:1507` existing test), AND a NEW test
  `lex_placeholders("wpkh([deadbeef/84'/0'/0']@0/<0';1'>/*)")` errors with the SAME hardened-multipath
  message — proving the prefix alternation did not regress the H13 reject.
- **Differential-oracle (origin-fidelity anchor, value-add):** the oracle's `wsh-multi-2of3-divergent`
  row (`:115`) already proves origin/path → address divergence; H7's round-trip test asserting
  prefix≡suffix md1/mk1 is the structural anchor. An additional oracle row is NOT required (the unit
  fp-mismatch + round-trip tests are decisive).

### 3.5 Cycle-1 zone interaction (confirmed clean off `f9467cc5`)

H7 edits `parse_descriptor.rs` + `bundle.rs` + `verify_bundle.rs` — the exact files cycle-1's
H12/H1/H13 just merged into. Off `f9467cc5` (post-merge) those changes are already present, so this is
a **within-file ordering note, not a merge conflict**: branch cycle-2 off `f9467cc5`, leave the H13
group-4 multipath capture/validator byte-identical, add the orthogonal prefix-origin alternation.

---

## 4. Workstream / concurrency — three disjoint-file agents

| WS | files touched | shares zone with? |
|---|---|---|
| **S-TEMPLATE (H8)** | `synthesize.rs` ONLY | none |
| **WS-EXPORT-MULTISIG (H10)** | `cmd/export_wallet.rs`, `error.rs`, `wallet_export/mod.rs` (refusal-message helper), and the three emitters only if a per-format assertion is added (preferred: dispatch-only, so emitters untouched) | `error.rs` shared with no other WS this cycle |
| **S-VERIFY-LEX (H7)** | `parse_descriptor.rs`, `cmd/bundle.rs`, `cmd/verify_bundle.rs` | none |

**File-disjointness CONFIRMED** — no file appears in two workstreams (H10's `error.rs` +
`export_wallet.rs` vs H8's `synthesize.rs` vs H7's `parse_descriptor.rs`/`bundle.rs`/
`verify_bundle.rs`). The three can run as **three concurrent single-subagent workstreams** (one
subagent per phase, TDD, in a worktree; never parallel re-impls of the same finding). Per CLAUDE.md:
single brainstorm SPEC + R0 GREEN gate (this doc) → plan-doc + R0 GREEN → per-WS single-subagent TDD →
mandatory post-impl adversarial whole-diff review persisted to `design/agent-reports/`.

> **Sequencing note:** H7's zone (`parse_descriptor.rs`/`bundle.rs`/`verify_bundle.rs`) is the
> post-cycle-1 code most recently merged; branch all three off `f9467cc5` so H7 rides the H13 reject
> already present. No inter-WS ordering dependency otherwise.

---

## 5. SemVer / lockstep

- **toolkit MINOR** — all three are behavior changes (corrected ms1 language / new refusal / newly
  honored annotation). One MINOR bump if batched. Current `origin/master` = **0.61.0** → next MINOR;
  **resolve the exact number at release** (the paused own-account cycle is renumbering — do not hard-code).
- **Release version-site ritual (CLAUDE.md / MEMORY) — the plan-doc MUST carry this as an EXPLICIT,
  enumerated checklist item (R0-M6); it is NOT gate-enforced and drifts silently:**
  1. bump `Cargo.toml` version;
  2. update **BOTH READMEs** (repo-root + `crates/mnemonic-toolkit/`) install-pin;
  3. update `fuzz/Cargo.lock` (the `cfg(fuzzing)` self-pin);
  4. re-run the full test suite + the fuzz harness AFTER the bump, BEFORE tag.
  (None of 2–3 has a CI gate; a missed self-pin surfaces only later. Pin the literal list in the
  plan-doc so the implementer cannot rely on a gate that does not exist.)
- **GUI schema-mirror:** **NO leg.** None of the three adds/removes/renames a clap flag, subcommand,
  or dropdown value. H10 = pure refusal (no flag, locked); H8 = private-fn signature; H7 =
  lexer-internal. `mnemonic-gui/src/schema/mnemonic.rs` is untouched; the lagging `schema_mirror` gate
  has nothing to catch. (CONFIRMED §2.7.)
- **Manual:** **NO flag-table leg** (no flag added/removed → `docs/manual/tests/lint.sh` bidirectional
  flag-coverage check is unaffected). OPTIONAL prose: a one-line H10-refusal note under
  `docs/manual/src/40-cli-reference/` is courtesy, not required (defer to plan-doc).
- **No codec-publish dependency** — all three are toolkit-only (no md/mk/ms tag→pin chain), unlike
  cycle-1's H13.

### 5.1 FOLLOWUP slugs to FILE (listed, NOT edited here)

To be filed in `design/FOLLOWUPS.md` (and flipped to RESOLVED in the shipping commit per the
status-discipline rule), one per finding:

- `template-form-md1-drops-bip39-wordlist-language` (H8)
- `export-wallet-unsorted-multi-silent-sortedmulti-coercion` (H10) — note "PURE REFUSAL, no flag"
- `descriptor-prefix-form-origin-annotation-ignored` (H7) — note "ACCEPT both positions; preserves
  cycle-1 H13 multipath reject"
- **(OPTIONAL, NOT this cycle — round-2 I-A)** `export-wallet-direct-descriptor-unsorted-multi-generic-refusal`:
  the direct `export-wallet --descriptor 'wsh(multi(…))'` path resolves `template_opt = None` and is
  refused by the emitters' generic `BadInput` (funds-safe — no silent coercion), NOT the new typed
  `ExportWalletUnsortedMultisigUnsupported`. A future cosmetic upgrade could classify the
  `template == None` direct path via the parsed descriptor (`Tag::Multi` / `script_type`) to surface the
  typed message there too. Cosmetic only (the funds-safety hole does not exist on this path); deliberately
  OUT of cycle-2 scope.

(The bug-hunt checklist items H8/H10/H7 in `constellation-bughunt-2026-06-20.md` get ticked with the
fixing commit SHA per that file's "fix checklist" contract.)

---

## 6. Confirmation matrix (no-GUI / no-manual / no-codec)

| concern | H8 | H10 | H7 |
|---|---|---|---|
| new clap flag? | NO (private fn sig) | NO (pure refusal, locked) | NO (lexer-internal) |
| GUI schema_mirror leg? | NO | NO | NO |
| manual flag-table leg? | NO | NO (optional prose only) | NO |
| codec tag→pin chain? | NO | NO | NO |
| new ToolkitError variant? | NO | YES (1, alphabetical; struct-form `{ format }`, modeled on taproot precedent's routing — NOT 1:1 shape) | NO |
| differential-oracle row needed? | NO (ms-decode unit) | NO (behavioral; existing divergent row cited) | NO (unit fp-mismatch + round-trip) |
| zone shared with another cycle-2 WS? | NO | only `error.rs` (no other WS edits it) | NO |

---

## 7. Test-plan / gating summary

- **H8:** unit ms-decode of emitted template-form ms1 reports run language + reconstructs the
  non-English phrase (master-fp divergence English↔Spanish) + English-regression + keyed↔template
  parity. TDD RED first.
- **H10:** CLI behavioral — unsorted `wsh-multi`/`sh-wsh-multi` (resolved `CliTemplate` ∈
  {`WshMulti`,`ShWshMulti`}) on the **`--template` path** → electrum/coldcard-multisig/jade exit 2 typed
  refusal; the **`--from-import-json` path** (template derived by `template_from_descriptor → Some(WshMulti)`)
  → same exit-2 typed refusal; **MANDATED false-refuse guard:
  `sortedmulti`/`sortedmulti_a`/`multi_a` + single-sig + descriptor/sparrow still exit 0**
  (taproot-multisig still hits the existing taproot refusal, not the new one). The **direct `--descriptor`
  path** (round-2 I-A) resolves `template_opt = None` and is refused by the EXISTING emitter-level generic
  `BadInput` (funds-safe; the test asserts ANY-error refusal, NOT the new typed kind). Restore-path
  regression (M-1): unsorted-`WshMulti` md1 → field-less vendor refused via the `emit_payload` chokepoint.
  Exit-code-behavioral; no new oracle row.
- **H7:** unit lex/strip parity prefix≡suffix; bundle fp-mismatch fires on prefix; round-trip
  prefix≡suffix byte-identical md1/mk1; verify-bundle pin; **composition edges (I2): prefix-path-no-fp
  ⇒ same rejection as suffix, and prefix-fp vs `--slot @N.fingerprint=` mismatch ⇒ refuse**; **H13
  hardened-multipath NON-REGRESSION test** (suffix AND prefix). TDD RED first.
- **`tests/bitcoind_differential.rs`:** extend ONLY where it adds value — it does NOT for any of the
  three (H10 refusal emits no file; H7/H8 are unit/round-trip provable). Cite the existing
  `wsh-multi-2of3-divergent` row as the motivating address-divergence proof.

---

## 8. Open questions for the R0 reviewer

1. **(H7, primary)** ACCEPT the prefix form? This spec recommends ACCEPT (BIP-380-canonical + the
   toolkit's own `--help` at `bundle.rs:2300` + the `detect_bare_tr` sibling already parses prefix +
   zero suffix-test regression). Confirm, or direct REJECT (with the help-text rewrite + typed error
   it entails).
2. **(H10) — CLOSED (round-1 fold I1).** Predicate PINNED in §2.3: refuse iff the resolved
   `CliTemplate` ∈ {`WshMulti`, `ShWshMulti`} (STRUCTURED check on the typed enum, immune to the
   `sortedmulti(`-substring false-match). The earlier "`template == None` on the descriptor path"
   rationale was FALSE for the three refusal targets (`format_requires_template == true` → non-`None`
   template both paths) and is corrected. Documented string FALLBACK (`multi(` NOT preceded by `sorted`,
   per the `wallet_export/mod.rs:264` house style) retained only if a no-`CliTemplate` path is found;
   none exists for the refusal set. Mandated `sortedmulti`/`sortedmulti_a`/`multi_a`-NOT-refused
   regression test added (§2.6). No longer open.
3. **(H10)** Refusal format set: {Electrum, Coldcard, ColdcardMultisig, Jade} vs {Electrum,
   ColdcardMultisig, Jade} — can the generic `coldcard` single-sig alias ever carry an unsorted-multi
   descriptor? If not, drop `Coldcard` from the set (harmless either way).
4. **(H8/L9)** Keep cycle-2 = the 3 named HIGHs (recommended), or fold L9 (template-path missing
   refusals — guards in `restore.rs`, a DIFFERENT file from H8's `synthesize.rs`) into S-TEMPLATE? This
   spec recommends NOT folding (L9 fail-safes to NO-MATCH, not funds-loss; folding crosses into
   `restore.rs` and widens the diff). See §1.4 (wording tightened per round-1 fold M5).
5. **(H10)** Byte-exact refusal message wording + whether to name all three faithful formats or just
   `descriptor`. Draft in §2.4.
6. **(H10) — CLOSED (round-1 fold M2; coldcard cite refreshed round-2 M-2).** Guard ordering vs the
   taproot refusals (jade `:48-52`, coldcard `:268`) is RESOLVED by the structured predicate: it matches ONLY
   `WshMulti`/`ShWshMulti`, a variant set DISJOINT from `TrMultiA`/`TrSortedMultiA`, so neither guard can
   shadow the other; taproot shapes still hit the taproot error independently. See §2.3 guard-ordering
   note + §2.5. No longer open.
7. **(H10)** Manual: file the optional one-line refusal note now, or defer entirely? (No flag-gate
   either way.)
8. **(H7)** Regex shape — separate optional prefix group + "both present → refuse" guard (recommended,
   §3.3a) vs single combined regex (§3.3b)? **(Composition edges DECIDED in round-1 fold I2 — §3.3
   (i)/(ii): prefix bracket requires the mandatory 8-hex fp (path-only ⇒ reject, mirroring suffix);
   prefix-fp vs `--slot @N.fingerprint=` must agree (mismatch ⇒ refuse, mirroring `bundle.rs:1620` /
   Row-19 `:1516-1525`). These are no longer open; only the regex SHAPE (a)-vs-(b) remains for R0.)**
9. **(cross)** Variant placement: confirm `ExportWalletUnsortedMultisigUnsupported` slots adjacent to
   the existing `ExportWallet*` alphabetical cluster (CLAUDE.md alphabetical rule), given the
   pre-v0.27.2 region is not yet globally sorted.

---

## 9. Spec R0 round-1 fold log

Source review: `design/agent-reports/cycle2-funds-loss-fixes-spec-R0-round1.md` (VERDICT NOT-GREEN —
0 Critical / 2 Important / 6 Minor, reviewed @ `f9467cc5`). All folds re-verified against
`git show origin/master:<path>` before writing. Each finding and its resolution:

| ID | finding | resolution |
|---|---|---|
| **I1** (H10) | §2.3 rationale #2 (`template == None` on the descriptor path) is FALSE for the 3 refusal targets; Q2 left the predicate open with a naive-substring trap | **FOLDED (round-1), then PARTLY OVER-CORRECTED — see round-2 I-A.** **PINNED the STRUCTURED predicate**: refuse iff resolved `CliTemplate` ∈ {`WshMulti`,`ShWshMulti`} — immune to the `sortedmulti(`-substring false-match (this part stands, funds-safe). **CORRECTION (round-2 I-A):** the round-1 fold's claim that "resolved `CliTemplate` is non-`None` on BOTH paths" is itself WRONG for the DIRECT `run --descriptor` path — `template_from_descriptor` is called ONLY in `run_from_import_json:812`, NEVER in `run`, so the direct `--descriptor` path stays `template_opt = None` (refused by the emitters' generic `BadInput`, funds-safe). The non-`None`-via-`template_from_descriptor` story holds ONLY for the `--from-import-json` path. See the round-2 fold log + §2.3. String form (`multi(` minus `sorted`, `mod.rs:264` house style) kept only as documented fallback. **Mandated the `sortedmulti`/`sortedmulti_a`/`multi_a`-NOT-refused regression test** (§2.6). Q2 → CLOSED in §8. |
| **I2** (H7) | §3.3/§3.4 under-specify the prefix-fp-vs-`--slot @N.fingerprint=` and prefix-path-no-fp composition edges | **FOLDED.** Added §3.3 (i)/(ii): (i) the prefix bracket's 8-hex fp is MANDATORY (caps-1 `{8}`, mirroring suffix `parse_descriptor.rs:84`) ⇒ path-only `[/path]@N` REJECTED as malformed, same as suffix; (ii) prefix-fp and an explicit `--slot @N.fingerprint=` MUST agree — mismatch ⇒ refuse (mirroring the suffix cross-check `bundle.rs:1616-1620` + Row-19 path-conflict `bundle.rs:1516-1525`). The pre-existing "prefix AND suffix bracket both present ⇒ refuse" guard stands. Two tests added (§3.4). Q8 composition part → DECIDED in §8 (only regex shape (a)/(b) remains). |
| **M1** (H10) | `multi_a` clause in the predicate is dead-weight (taproot refused upstream) | **FOLDED.** The structured predicate has no `multi_a` clause by construction (matches only `WshMulti`/`ShWshMulti`); §2.3 guard-ordering note states taproot is refused upstream + per-emitter, so no tree-walk/`multi_a`-carve-out over-engineering. |
| **M2** (H10) | Q6 guard-ordering vs taproot identified but unresolved | **FOLDED.** Resolved in §2.3 + §2.5: structured predicate's variant set {`WshMulti`,`ShWshMulti`} is DISJOINT from {`TrMultiA`,`TrSortedMultiA`} ⇒ no shadowing; taproot still hits its existing guard (jade `:48-52`, coldcard `:268` — round-2 M-2 refreshed the stale `:266-276`). Q6 → CLOSED. |
| **M3** (H8) | test-vector master-fps `1b6aef92`/`73c5da0a` unverified in R0 | **FOLDED.** §1.3: the RED assertion is the DIVERGENCE (`spanish_fp != english_fp` + template path matches the test-COMPUTED Spanish fp), COMPUTE-don't-hardcode; the hex pair is documentation, not a load-bearing literal. |
| **M4** (cross) | variant is struct-form `{ format }` but spec said "modeled 1:1" on the tuple precedent | **FOLDED.** §2.4 + §0 + §6 reworded: deliberate struct-form choice (mirrors the `ExportWalletMissingFields { .. }` struct-variant arm style at `error.rs:543/605/745`), "modeled on the precedent's ROUTING — NOT 1:1 shape"; implementer may pin the tuple form instead. |
| **M5** (H8/L9) | §1.4 "same `synthesize.rs` zone" framing is off — L9's guards are in `restore.rs` | **FOLDED.** §1.4 + §8-Q4 tightened: L9 guards (`restore.rs:2779`/`:2786`, missing from `run_multisig_template_completion` at `restore.rs:1321`) are in a DIFFERENT FILE from H8's `synthesize.rs`; NOT folding justified as "different file, fail-safe NO-MATCH, not funds-loss-equivalent." |
| **M6** (SemVer) | README×2 + `fuzz/Cargo.lock` self-pin ritual not pinned as an explicit checklist | **FOLDED.** §5 now carries the enumerated 4-step release checklist (bump → BOTH READMEs → `fuzz/Cargo.lock` → re-run suite+fuzz before tag), flagged as NOT gate-enforced, for the plan-doc to lift verbatim. |

**Cross-cut invariants re-confirmed after the round-1 folds (unchanged):** toolkit MINOR, NO GUI
schema-mirror leg, NO manual flag-table leg, NO codec tag→pin chain; workstream file-disjointness holds;
`ExportWalletUnsortedMultisigUnsupported` alphabetical placement intact; FOLLOWUP list (§5.1) — see the
round-2 optional addition. Per CLAUDE.md (reviewer-loop continues after every fold) the architect is
re-dispatched after the round-2 fold below.

---

## 9.1 Spec R0 round-2 fold log

Source review: `design/agent-reports/cycle2-funds-loss-fixes-spec-R0-round2.md` (VERDICT NOT-GREEN —
0 Critical / 1 Important (I-A) / 3 Minor, reviewed @ `f9467cc5`). The round-2 review CONFIRMED the H10
predicate is complete (no unsorted-multi shape escapes `{WshMulti,ShWshMulti}` into a silent sortedmulti
coercion — emitter-by-emitter enumeration) and the H7 edges (i)/(ii)/(both-present) are closed. The ONE
Important is fold-INTRODUCED: the round-1 fold corrected the import-json mechanism but substituted an
equally-wrong story for the DIRECT `--descriptor` path. All round-2 folds re-verified LIVE via
`git show origin/master:<path>` (read `run`, `run_from_import_json`, `emit_payload`, the three emitters,
`bundle.rs` fp-guard) before writing.

| ID | finding | resolution |
|---|---|---|
| **I-A** (H10, fold-introduced) | §2.3/§2.6/§7 assert the DIRECT `run --descriptor 'wsh(multi(…))'` path "derives a non-`None` template via `template_from_descriptor` → `WshMulti`"; FALSE on master — `template_from_descriptor` is NEVER called in `run` (only `run_from_import_json:812`), so `template_opt` stays `None` and the new structured guard does NOT fire there. The §2.6 descriptor-path test (asserting the new typed refusal) would FAIL. | **FOLDED.** Verified LIVE: `run` has TWO descriptor-bearing sub-paths — `--template wsh-multi` sets `resolved_template = Some(WshMulti)` (`export_wallet.rs:542`) → caught by the guard; the DIRECT `--descriptor` arm builds `canonical` but leaves `resolved_template = None` → `template_opt = None`, refused ALREADY by each emitter's `inputs.template.ok_or_else(…)` generic `BadInput` (electrum `electrum.rs:50-54`, jade `jade.rs:36-40`, coldcard `coldcard.rs:111-114`, coldcard-multisig `export_wallet.rs:129-132`) — funds-safe (refused, NEVER silently coerced). **Rewrote §2.3** (TWO-sub-path control-flow correction; removed every "non-`None` on every path" / "derived `WshMulti` on `--descriptor`" claim). **Pinned round-2 I-A option (i) (minimal):** direct `--descriptor` keeps its existing generic refusal; the new typed `ExportWalletUnsortedMultisigUnsupported` covers ONLY the `--template` + `--from-import-json` paths. **Rewrote the §2.6 descriptor-path test** to assert refusal by ANY error (the generic `BadInput`, exit ≠ 0) and explicitly `kind() != ExportWalletUnsortedMultisigUnsupported` — matching reality; ADDED a separate `--from-import-json` test for the typed refusal. Scoped §0/§7 wording to "the `--template` path and the `--from-import-json` path." Optional typed-upgrade FOLLOWUP filed (§5.1). |
| **M-1** (H10) | `emit_payload` (NOT `emit_for_format`) is the shared dispatch for FOUR callers — `run`, `run_from_import_json`, restore's `build_import_payload` + `build_multisig_import_payload` (passes `template: Some(WshMulti)`); the restore path is silently in scope of the guard | **FOLDED.** Aligned the fn name to `emit_payload` (`export_wallet.rs:73`) everywhere (§0, §2.3 heading + rationale #1, §7). §2.3 now states the restore-path coverage is a CONSEQUENCE of the chokepoint (funds-safe — more refusals, never fewer) and does NOT break file-disjointness (guard lives in `export_wallet.rs`; `restore.rs` only calls it). Added a one-line restore regression test note (§2.6) pinning that an unsorted-`WshMulti` md1 → field-less vendor is refused, not silently coerced. |
| **M-2** (H10) | stale citations: coldcard taproot guard `:266-276`; shared-dispatch `match format` `:119` | **FOLDED.** Refreshed to live lines: coldcard taproot block opens `:268` (fn `emit_coldcard_multisig_text` `:258`, `None`-guard `:261-263`, taproot block `:268-277`) in §2.3-note + §2.5 + the round-1 M2 log row; `emit` `match format` opens `export_wallet.rs:109` (`collect_missing` match `:82-101`) in §2.3. |
| **M-3** (H7) | I2(ii) xpub-slot fp cross-check: the existing `bundle.rs:1616-1620` guard covers phrase/entropy ONLY; the xpub-slot arm (`:1637`) resolves `fp` via `--slot @N.fingerprint=` `.or(anno_fp)` (`:1654`) with NO equality check, so the plan-doc MUST take the "add explicit comparison" branch | **FOLDED.** §3.3(ii) + §3.4-edge-(ii) now mandate the ADDED explicit `prefix-anno-fp vs --slot @N.fingerprint=` comparison at the `:1654` xpub-slot site (refuse on mismatch, same `DescriptorParse` shape as `:1618-1620`); the "confirm existing covers it" branch is explicitly marked UNAVAILABLE for xpub slots (verified LIVE: `:1654` `.or(anno_fp)`, no check). The phrase-arm cross-check `:1616-1620` is unchanged. |

**Cross-cut invariants re-confirmed after the round-2 folds (unchanged):** the H10 structured predicate
`{WshMulti,ShWshMulti}` is COMPLETE for under-refuse and NOT over-refusing (round-2 enumeration);
toolkit MINOR; NO GUI schema-mirror leg; NO manual flag-table leg; NO codec tag→pin chain; workstream
file-disjointness holds (H8 = `synthesize.rs`; H10 = `export_wallet.rs` + `error.rs` +
`wallet_export/mod.rs` — the M-1 restore observation does NOT add `restore.rs` to the H10 zone, it only
calls the shared fn; H7 = `parse_descriptor.rs` + `bundle.rs` + `verify_bundle.rs`);
`ExportWalletUnsortedMultisigUnsupported` alphabetical placement (after `…Taproot…`, before
`FutureFormat`) intact; H7/H8 decisions (ACCEPT prefix / thread `run_language`) intact. Per CLAUDE.md
(reviewer-loop continues after every fold) the architect is re-dispatched after this round-2 fold.

---

_Brainstorm-spec only. No code, no source edits. All citations verified LIVE against toolkit
`origin/master` = `f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5` (0.61.0) on 2026-06-21. MANDATORY R0
review loop to 0C/0I before any implementation; re-dispatch the architect after every fold._
