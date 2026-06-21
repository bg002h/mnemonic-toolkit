# IMPLEMENTATION PLAN — cycle-2 constellation funds-loss fixes (H8 / H10 / H7)

**Status:** PLAN-DOC ONLY — no code, no source edits. This plan-doc goes through a MANDATORY opus-architect
**R0 review loop to 0 Critical / 0 Important BEFORE any implementation begins** (CLAUDE.md hard-gate).
Implementation (writing code, dispatching implementer subagents) MUST NOT start until this plan-doc is
R0-GREEN; after each fold, re-dispatch the architect (reviewer-loop continues after every fold).

**Upstream gate cleared:** the brainstorm-spec `design/BRAINSTORM_cycle2_funds_loss_fixes.md` is **R0-GREEN**
(`design/agent-reports/cycle2-funds-loss-fixes-spec-R0-round3.md` — VERDICT GREEN, 0C/0I, 3 informational
citation-hygiene Minors carried here, §10). This plan-doc operationalizes that spec.

**Source SHA (ALL citations below re-grepped LIVE against this, NOT the working tree):**
toolkit `origin/master` = **`f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5`** (release **0.61.0**, post-cycle-1
H12/H1/H13). The working tree is on the other instance's WIP branch `feature/own-account-subset-search`
(SHA `364d296f`) and was NOT used for any citation. Companion md repo `origin/main` = `58cc9ec2…` — **no md
edit this cycle**. Every file:line below was verified on 2026-06-21 via `git show origin/master:<path>`.

> **NOTE — Cargo.toml on the working tree reads `0.60.0`; `origin/master` reads `0.61.0`.** The SemVer
> baseline for this cycle is **`origin/master` = 0.61.0** (the working-tree number belongs to a paused,
> renumbering own-account cycle). See §8.

---

## 0. Executive summary

Three independent HIGH funds-loss findings → **three disjoint-file workstreams runnable concurrently** →
one toolkit MINOR bump. Single-subagent-per-phase TDD (tests RED first), per-WS exit criteria, then a
mandatory post-impl adversarial whole-diff review.

| WS | finding | one-line | files (disjoint) | new error variant? |
|---|---|---|---|---|
| **WS-A / S-TEMPLATE** | **H8** | `--md1-form=template` hardcodes BIP-39 English → non-English seed re-emits English → WRONG master seed | `synthesize.rs` ONLY | no |
| **WS-B / WS-EXPORT-MULTISIG** | **H10** | unsorted `multi(…)` silently coerced to BIP-67 `sortedmulti` by field-less electrum/coldcard/jade → WRONG addresses | `error.rs` + `cmd/export_wallet.rs` | **yes (1)** |
| **WS-C / S-VERIFY-LEX** | **H7** | documented prefix-form `[fp/path]@N` origin annotation silently ignored → origin dropped + fp cross-check bypassed | `parse_descriptor.rs` + `cmd/bundle.rs` (+ `cmd/verify_bundle.rs` test only) | no |

**File-disjointness (CONFIRMED, no overlap):** WS-A = `synthesize.rs`; WS-B = `error.rs` +
`cmd/export_wallet.rs`; WS-C = `parse_descriptor.rs` + `cmd/bundle.rs` + `cmd/verify_bundle.rs`. No file
appears in two workstreams ⇒ three concurrent single-subagent worktrees (§7). (WS-B's guard lives in
`export_wallet.rs::emit_payload`; `restore.rs` is NOT edited — it merely calls the shared fn, so the
restore-path coverage is a free consequence, not a file in WS-B's zone.)

**SemVer / lockstep:** toolkit **MINOR** off 0.61.0 (single bump if batched; **resolve the exact number at
release** — §8). **NO new clap flag in any of the three** → **NO GUI schema-mirror leg, NO manual flag-table
leg** (§9). No codec tag→pin chain (all toolkit-only).

---

## 1. WS-A — H8 (S-TEMPLATE, `synthesize.rs`)

### 1.1 Change sites (verified LIVE @ `f9467cc5`)

| site | LIVE line | current | change |
|---|---|---|---|
| `synthesize_template_descriptor` sig | `synthesize.rs:1158-1162` | `fn synthesize_template_descriptor(descriptor, cosigners, privacy_preserving) -> Result<Bundle, ToolkitError>` | **add `run_language: bip39::Language`** (after `privacy_preserving`, mirroring `synthesize_descriptor:467-472`'s param order where `run_language` sits after `privacy_preserving`) |
| call site (drops `run_language`) | `synthesize.rs:487` | `return synthesize_template_descriptor(descriptor, cosigners, privacy_preserving);` | **forward it:** `…(descriptor, cosigners, privacy_preserving, run_language)` |
| template ms1 emit (hardcoded English) | `synthesize.rs:1265` | `let emit_lang = c.language.unwrap_or(bip39::Language::English);` | **`let emit_lang = c.language.unwrap_or(run_language);`** — byte-identical to the keyed path `:547` |
| 3 in-module test call sites | `synthesize.rs:2407`, `:2443`, `:2594` | `synthesize_template_descriptor(&descriptor, &cosigners, false)` | **pass a language arg** — use `bip39::Language::English` to preserve current behavior |

**Sole-site / sole-caller confirmation (grep-verified on master):** `synthesize_template_descriptor` has
EXACTLY one non-test caller (`:487`) and 3 test callers (`:2407/:2443/:2594`). The hardcoded-English
fallback in the template path is the SOLE site `:1265`; the keyed twin `:547` already uses `run_language`;
`synthesize_unified` (defined `:994` — m-2: the `mnemonic_lang = seed_mnemonic.language()` computation at
`:709` is in a DIFFERENT earlier per-slot emit fn, not `synthesize_unified`) uses the actual seed language
(correct). The template ms1-emit loop
(`:1262-1275`) is a SINGLE loop serving BOTH single-sig and multisig template forms — threading
`run_language` to `:1265` covers single-sig AND multisig template emit in one change. Private `fn` ⇒ NO
external/CLI surface ⇒ no clap flag, no schema-mirror, no manual.

**Why funds-loss (BIP-39, CONFIRMED):** seed = PBKDF2-HMAC-SHA512 over the NFKD-normalized *mnemonic phrase
string*, not raw entropy. Same entropy through a different-language wordlist → different word string →
different 512-bit seed → different keys/addresses. Re-emitting a non-English seed's ms1 as English silently
changes the master seed; the template card is then unrecoverable.

**Bite-scope (not a downgrade):** bites via the `c.language == None` fallback — the descriptor-`@N`
phrase/entropy path where slot language is `None` and run-level `--language` is the sole carrier. A slot
already carrying `c.language = Some(non-English)` emits correctly today; the fix is the same regardless.

### 1.2 Phase / TDD plan (single subagent, RED first)

**Phase A1 — RED tests, then GREEN impl.** Write tests FIRST (they fail because the param doesn't exist /
English is hardcoded), then thread the param.

Tests (in-module unit, `synthesize.rs` test section):
1. **Non-English round-trip anchor (the funds-safety anchor):** build a `CosignerKeyInfo` with
   `language = None` and a fixed all-zero (or fixed) entropy; run-level `run_language = bip39::Language::Spanish`
   (or `Japanese`); call `synthesize_template_descriptor(&descriptor, &cosigners, false, Spanish)` (or the
   public `synthesize_descriptor` entry with `md1_form = template`). Assert the emitted ms1 decodes via
   `ms_codec::decode` to `Payload::Mnem { language: <spanish wire code = 3>, .. }`, **NOT** `Payload::Entr`
   and **NOT** English. **Cover BOTH single-sig AND multisig template emit** (the loop is shared, but pin
   both: one `n==1` descriptor and one `n>=2` multisig descriptor).
2. **COMPUTE-don't-hardcode master-fp divergence (round-3 m-iii / spec §1.3 / R0-M3):** the all-zero-entropy
   master-fp oracle pair (`1b6aef92` Spanish / `73c5da0a` English) cited in the bughunt report was NOT
   re-derived in any R0 and **MUST NOT be hard-coded as a load-bearing literal.** The RED assertion is the
   DIVERGENCE: derive `spanish_master_fp` and `english_master_fp` IN THE TEST from the same all-zero entropy
   through the two wordlists, assert `spanish_master_fp != english_master_fp`, AND assert the template path's
   reconstructed fp equals the test-computed Spanish fp. The `1b6aef92`/`73c5da0a` pair may appear only as a
   documentation comment, never as the assertion's RHS — so a transcription typo cannot make the test
   vacuously pass.
3. **English regression guard:** an English `run_language` slot still emits `Payload::Entr` (the
   `emit_lang == English` branch at `:1266`), byte-identical to today.
4. **Keyed↔template parity (anti-drift):** for the same `(entropy, run_language)` input, the template path
   and the keyed path (`synthesize_descriptor` with `md1_form != template`) produce the SAME ms1 — structurally
   pins the keyed↔template symmetry so a future divergence re-RED's.

**Phase A1 GREEN impl:** the 4 edits in §1.1. No other production change.

### 1.3 L9 — NOT folded (decided in spec §1.4, R0-M5 confirmed)

L9 (parallel template-path missing refusals) lives in `restore.rs` — a DIFFERENT file from H8's
`synthesize.rs` — and fail-safes to NO-MATCH (not a wrong-address emit), so it is not funds-loss-equivalent
to H8. Folding would cross into `restore.rs` and widen the diff. **Cycle-2 stays the 3 named HIGHs.**

---

## 2. WS-B — H10 (WS-EXPORT-MULTISIG, `error.rs` + `cmd/export_wallet.rs`)

### 2.1 New error variant (`error.rs`)

Add ONE typed variant, **struct form** (deliberate divergence from the tuple precedent
`ExportWalletTaprootMultisigUnsupported(&'static str)`, for call-site readability — mirrors the
`ExportWalletMissingFields { .. }` struct-variant arm style at `error.rs:543/605/745`):

```rust
/// SPEC cycle-2 H10 — the electrum / coldcard / jade multisig file formats are
/// BIP-67 sortedmulti-only (no field to express literal `multi(...)` key order),
/// so exporting an UNSORTED `wsh-multi` / `sh-wsh-multi` to them would silently
/// coerce to sortedmulti → different witnessScript/address. Refuse, pointing to
/// a faithful format (descriptor / bitcoin-core / sparrow). Exit 2.
ExportWalletUnsortedMultisigUnsupported { format: &'static str },
```

**Alphabetical placement (CLAUDE.md rule — verified LIVE).** `ExportWalletUnsortedMultisigUnsupported`
sorts AFTER `ExportWalletTaprootMultisigUnsupported` (`T < U`) and BEFORE `FutureFormat` (`E < F`). Insert
at that position in EACH of the four sites:

| arm | LIVE anchor (insert immediately after the `…Taproot…` line) | new line |
|---|---|---|
| `enum ToolkitError` | `error.rs:169` `ExportWalletTaprootMultisigUnsupported(&'static str),` → before `:170` `FutureFormat {` | `ExportWalletUnsortedMultisigUnsupported { format: &'static str },` |
| `fn exit_code` (`:517`) | `error.rs:545` `…Taproot…(_) => 2,` → before `:546` `FutureFormat { .. } => 3,` | `ToolkitError::ExportWalletUnsortedMultisigUnsupported { .. } => 2,` |
| `fn kind` (`:579`) | `error.rs:607-609` `…Taproot…(_) => "ExportWalletTaprootMultisigUnsupported"` → before `:610` `FutureFormat { .. } => "FutureFormat",` | `ToolkitError::ExportWalletUnsortedMultisigUnsupported { .. } => "ExportWalletUnsortedMultisigUnsupported",` |
| `fn message` (`:646`) | `error.rs:749` `…Taproot…(name) => {…}` → before `:752` `FutureFormat { .. } => …` | `ToolkitError::ExportWalletUnsortedMultisigUnsupported { format } => { <byte-exact message, §2.4> }` |

**Exit code 2** (mirrors the taproot precedent and every export refusal). The `exit_code_table_per_variant`
and `kind_strings_stable` tests in `error.rs` (`:965`, `:1272`) are exhaustive-style assertion tables — if
they enumerate every variant the implementer adds the new variant's row there too (verify at impl time;
otherwise the exhaustive `match` change alone suffices).

> **Implementer latitude:** the tuple form `ExportWalletUnsortedMultisigUnsupported(&'static str)` (1:1 with
> the taproot precedent's shape) is equally acceptable for minimal diff. **This plan PINS the struct form**;
> if the implementer prefers the tuple form they MUST keep all four arms consistent with that choice.

### 2.2 The refusal — where it lives (`cmd/export_wallet.rs`)

**Single chokepoint: the shared `emit_payload` dispatch (`export_wallet.rs:73`), BEFORE the per-format
`emit` (the `match format {` at `:109`).** `emit_payload` serves FOUR callers (doc-comment `:60-72`): `run`
(`:625`), `run_from_import_json` (`:845`), and restore's `build_import_payload` +
`build_multisig_import_payload`. One guard there covers all entry paths.

**The guard (STRUCTURED, on the resolved typed enum):**

```text
refuse iff inputs.template ∈ { Some(CliTemplate::WshMulti), Some(CliTemplate::ShWshMulti) }
       AND format ∈ { Electrum, Coldcard, ColdcardMultisig, Jade }
→ Err(ToolkitError::ExportWalletUnsortedMultisigUnsupported { format: <name> })
```

Place it after the `collect_missing` block and BEFORE the `emit` `match format {` at `:109` (i.e. between
the missing-fields check ending ~`:107` and the `emit` match), reading `inputs.template` (the `EmitInputs`
field set at `:605`) and `format`. This is a structural check on the typed enum — **immune to the
`sortedmulti(`-as-substring false-match** a naive `.contains("multi(")` would hit. Do NOT add a string scan.

**Why `{WshMulti, ShWshMulti}` is complete AND not over-refusing (spec §2.3, R0 round-2/3 enumeration —
re-verified):** `CliTemplate` has EXACTLY 10 variants (`template.rs:16-42`) with NO bare `Multi` /
`sh(multi)` / general variant; the only unsorted-multi variants are `WshMulti` / `ShWshMulti`. Every route
to a field-less emitter resolves to either a member of the set (caught), a sorted variant (allowed — BIP-67
is what they implement), taproot (refused independently, §2.5), a general policy (refused upstream at
`export_wallet.rs:798` by `descriptor_is_general_policy`, defined `wallet_export/mod.rs:301`), legacy
`sh(multi)`/BIP-45 (refused by `template_from_descriptor`'s `ShInner::Ms` arm, `wallet_export/mod.rs:275-276`),
or `template == None` (refused by each emitter's own `template.ok_or_else` generic `BadInput`).

**The three entry paths (control-flow, verified LIVE — the round-2 I-A correction):**
- **`--template wsh-multi`/`sh-wsh-multi` (`run`):** sets `resolved_template = Some((…, WshMulti, k))`
  (`export_wallet.rs:542`) → `template_opt = Some(WshMulti)` (`:553-560`) → `inputs.template = Some(WshMulti)`
  (`:605`). **Guard fires (typed exit-2).** ✓
- **`--from-import-json` unsorted `wsh(multi)`/`sh(wsh(multi))` (`run_from_import_json`):**
  `template_from_descriptor` (called ONLY here, `export_wallet.rs:812`, gated by `format_requires_template`
  at `:791`, past the general-policy refusal `:798`) computes `is_sorted = …contains("sortedmulti(")`
  (`wallet_export/mod.rs:264`) and maps `Wsh(_) → WshMulti`/`WshSortedMulti` (`:279-282`), `Sh(Wsh) →
  ShWshMulti`/`ShWshSortedMulti` (`:270-273`) — **preserves the unsorted distinction** → `Some(WshMulti)`.
  **Guard fires (typed exit-2).** ✓
- **direct `run --descriptor 'wsh(multi(…))'`:** the `if let Some(desc) = &args.descriptor` arm (`@N` forms
  rejected at `:443`) builds `canonical` but NEVER assigns `resolved_template` → `template_opt = None` →
  `inputs.template = None`. `template_from_descriptor` is NOT called in `run` (grep-confirmed — ONLY at
  `:812`). **The new structured guard does NOT fire** — and need not: this path is ALREADY funds-safe,
  refused by each emitter's own `template.ok_or_else` generic `BadInput` (electrum `electrum.rs:52-54`, jade
  `jade.rs:36-39`, coldcard `coldcard.rs:111-114`, coldcard-multisig the `_ =>` `BadInput` arm body at
  `export_wallet.rs:130-132`, inside the `match inputs.template {` opening `:120` — m-3: the range start was
  off-by-one, arm body is `:130-132`). An unsorted `multi` here is **REFUSED, never silently coerced** — the
  funds-safety hole does not exist. (Optional typed-upgrade FOLLOWUP, §8 — deliberately OUT of scope.)

**Format set = the THREE field-less vendors {Electrum, Coldcard, ColdcardMultisig, Jade}, NOT
`format_requires_template`.** `format_requires_template` (`export_wallet.rs:53-59`) also returns `true` for
Sparrow, which is FAITHFUL (carries the literal `multi(`/`sortedmulti(` token) and must NOT be refused.
`Coldcard` (the generic single-sig alias) cannot carry a multisig descriptor, but gating it in is harmless
and future-proofs the alias. (See §11 Open-Q1 — R0 may drop `Coldcard` to scope the set to {Electrum,
ColdcardMultisig, Jade}; harmless either way.)

**Restore-path coverage is a free, funds-safe CONSEQUENCE (round-2 M-1).** `restore`'s
`build_multisig_import_payload` passes `template: Some(t)` (CAN be `Some(WshMulti)`) into `emit_payload`, so
the guard ALSO fires on `restore --md1 --format electrum/coldcard/jade` for an unsorted-`WshMulti` md1 — a
desirable funds-safety extension (more refusals, never fewer). **`restore.rs` is NOT edited** (file-disjoint
holds; the guard lives in `export_wallet.rs::emit_payload`, restore merely calls it). Pinned by a one-line
restore regression assertion (§2.6).

### 2.3 Guard ordering vs the per-emitter taproot refusals (no hazard)

Because the predicate matches ONLY `WshMulti`/`ShWshMulti` — a variant set DISJOINT from
`TrMultiA`/`TrSortedMultiA` — a taproot-multisig shape passes the new guard untouched and reaches its
existing per-emitter taproot refusal (electrum `electrum.rs:60-68`, jade `jade.rs:48-52`, coldcard the
`if matches!` taproot block in `emit_coldcard_multisig_text` — fn `:258`, `None`-guard `:261-263`, taproot
block `:268-277`) exactly as today. The new guard and the taproot guard match disjoint variant sets — neither
can shadow the other. (On the import-json path, taproot is additionally refused upstream at
`export_wallet.rs:741-744` before `template_from_descriptor`.)

### 2.4 Byte-exact message (§11 Open-Q2 — finalize wording at R0)

Drafted (the `message()` arm returns this; `{format}` = `electrum` / `coldcard-multisig` / `jade`):

> `--format {format} cannot faithfully export an UNSORTED multisig (wsh-multi / sh-wsh-multi): the {format} multisig file format is BIP-67 sortedmulti-only and would silently reorder the keys, changing the witnessScript and every address. Use --format descriptor, --format bitcoin-core, or --format sparrow (which preserve literal multi(...) key order), or use a sortedmulti template if BIP-67 ordering is intended.`

The implementer may route this through a small `wallet_export` helper (matching the taproot precedent's
message routing) or inline it in the `message()` arm — implementer's choice; pin the byte-exact string in
the test (§2.6) so the wording is regression-locked.

### 2.5 Behavior to PRESERVE (anti-over-refusal — pinned by the §2.6 regression test)

- `WshSortedMulti`/`ShWshSortedMulti` → these three formats: still exports (BIP-67 is what they implement).
- Single-sig (`bip44/49/84/86`) → these formats: unaffected.
- `descriptor`/`sparrow`/`bitcoin-core`/`bip388` ← unsorted `multi`: still allowed (faithful).
- `TrMultiA`/`TrSortedMultiA` → these formats: still hit the EXISTING per-emitter taproot refusal, NOT the
  new error (the new guard's variant set is disjoint).

### 2.6 Phase / TDD plan (single subagent, RED first)

**Phase B1 — RED tests, then GREEN impl** (new variant + guard).

Behavioral CLI tests (assert exit code AND `kind()`):
1. **`--template` typed refusal:** `export-wallet --format electrum --template wsh-multi --threshold 2
   --slot @0.xpub=… --slot @1.xpub=…` → exit 2, `kind() == "ExportWalletUnsortedMultisigUnsupported"`,
   stderr contains the typed message naming a faithful format. Repeat for `coldcard-multisig`, `jade`, and
   for `sh-wsh-multi`.
2. **`--from-import-json` typed refusal:** an `import-wallet --json` envelope whose descriptor is an
   unsorted `wsh(multi(2,…))` (then `sh(wsh(multi(…)))`), then `export-wallet --from-import-json <env>
   --format electrum` (repeat coldcard-multisig/jade) → exit 2 with the typed kind. This is the path where
   `template_from_descriptor → Some(WshMulti)`.
3. **Direct `--descriptor` — ANY-error refusal, NOT the new typed kind (round-2 I-A):**
   `export-wallet --format electrum --descriptor 'wsh(multi(2,…))…'` (no explicit `--template`) → exit ≠ 0
   refused by the EXISTING emitter-level generic `BadInput` (`electrum.rs:52-54`); assert the `kind()` is
   **NOT** `ExportWalletUnsortedMultisigUnsupported` (it is the generic `BadInput`). The funds-safety property:
   refused, never silently coerced.
4. **MANDATED `sortedmulti`-NOT-refused regression (false-refuse guard):** exporting a SORTED shape to each
   field-less vendor STILL SUCCEEDS (exit 0): `--template wsh-sortedmulti` → exit 0 (repeat
   coldcard-multisig/jade); `--template sh-wsh-sortedmulti` → exit 0 (repeat). This RED's immediately if a
   future predicate drift (e.g. a naive `.contains("multi(")` false-matching `sortedmulti(`) starts refusing
   sorted shapes.
5. **`multi_a`/`sortedmulti_a`-NOT-refused-by-the-new-guard proof:** `tr-multi-a` / `tr-sortedmulti-a` →
   these three formats: hits the EXISTING taproot refusal — assert `kind() ==
   "ExportWalletTaprootMultisigUnsupported"` (proving disjointness per §2.3/§2.5).
6. **Single-sig + faithful must-still-work:** `--format coldcard --template bip84` → exit 0;
   `--format descriptor --template wsh-multi` → exit 0 and emits the literal `multi(`;
   `--format sparrow --template wsh-multi` → exit 0.
7. **Restore-path regression (round-2 M-1, one line):** `restore --md1 --format electrum` (or coldcard/jade)
   of an md1 reconstructing an UNSORTED `WshMulti` → exit 2 with the typed kind (guard fires via the shared
   `emit_payload` chokepoint that `restore`'s `build_multisig_import_payload` calls). `restore.rs` is NOT
   edited — this asserts the free consequence.

**Differential-oracle:** do NOT add a row that EXPORTS to electrum (the refusal means no file is emitted).
The existing `wsh-multi-2of3-divergent` row (`tests/bitcoind_differential.rs:115`) already proves the
underlying address divergence — cite it, don't duplicate. H10 is exit-code-behavioral; the CLI refusal tests
are the primary gate.

**Phase B1 GREEN impl:** the new variant (§2.1) + the guard (§2.2). `error.rs` and `export_wallet.rs` only.

---

## 3. WS-C — H7 (S-VERIFY-LEX, `parse_descriptor.rs` + `cmd/bundle.rs`)

### 3.1 Change sites (verified LIVE @ `f9467cc5`)

| site | LIVE line | current | change |
|---|---|---|---|
| `lex_placeholders` regex | `parse_descriptor.rs:82-84` | `@(\d+)(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'\|h)?)*)\])?(?:/<([^>]*)>)?(/\*(?:'\|h)?)?` (suffix-origin only; **5 numeric caps consumed by paren order:** 1=idx `:89/:90`, 2=suffix-fp `:93`, 3=suffix-path `:104`, 4=multipath body / H13 strict validator `:119`, 5=wildcard `:152`) | **CONVERT TO ALL-NAMED GROUPS accessed BY NAME (C1, §3.3)** — name every existing group AND add a new OPTIONAL named prefix-origin group BEFORE `@(\d+)`. Named-by-name access is **position-independent** (verified by execution in the R0), so prepending the prefix can NEVER shift the H13 multipath group. Take fp/path from whichever of {prefix, suffix} is present; **both present → typed refuse**. |
| `lex_placeholders` consumers (5 sites) | `parse_descriptor.rs:89/90` (`caps[1]`), `:93` (`.get(2)`), `:104` (`.get(3)`), `:119` (`.get(4)` — **H13 strict validator `:119-152`**), `:152` (`.get(5)`) | numeric `caps[N]`/`caps.get(N)` | **rewrite EACH to `caps.name("…")`** per the §3.3 name map — INCLUDING the H13 hardened-multipath validator at `:119-152` (cycle-1 code now in `parse_descriptor.rs`; WS-C OWNS this file so the accessor change is IN-SCOPE). The validator BODY (`:119-152`) stays byte-identical; only its accessor changes `.get(4)` → `.name("mpath")`. |
| `substitute_synthetic` strip regex | `parse_descriptor.rs:369-371` | `@(\d+)(?:\[[0-9a-fA-F]{8}(?:/\d+(?:'\|h)?)*\])?(?:/<[0-9;]+>)?(?:/\*(?:'\|h)?)?` (strips suffix bracket only; brackets already **non-capturing**; **sole consumer = `caps[1]` at `:382`/`:393`**) | **mirror the prefix alternation as a NON-CAPTURING `(?:…)?` group** so a leading `[fp/path]` is stripped before `Descriptor::from_str`; keep the multipath class `[0-9;]+` (the C1 narrow class — do NOT widen). **NO collision** (R0-verified): the only consumer is `caps[1]`=index and a non-capturing prefix prepend keeps `caps[1]`=index. **Leave it numeric** (single group, no renumber risk); the prefix group MUST stay non-capturing `(?:…)` so `caps[1]` does not shift. |
| xpub-slot fp resolution (NO equality check today) | `bundle.rs:1654` | `.or(anno_fp)` (inside the `else if subkeys.contains(&…::Xpub)` arm opening `:1637`) | **add an explicit `prefix-anno-fp vs --slot @N.fingerprint=` equality check; refuse on mismatch** (same `DescriptorParse` shape as `:1618-1620`) — the phrase-arm guard `:1617-1620` does NOT reach xpub slots |

**Downstream consumers need NO change beyond the named-group regex conversion (+ its 5 in-fn `caps.name()`
rewrites), the non-capturing strip-regex prefix, and the xpub-slot check:** once `lex_placeholders` populates
`fingerprint_anno`/`origin_path_anno` for the prefix form, `resolve_placeholders`, the `fingerprint_annos`
vector, the phrase-arm fp cross-check (`bundle.rs:1581-1582` read, `:1617-1620` compare), and verify-bundle's
shared lexer (`verify_bundle.rs:1342`/`:1346`) consume them identically (they read the
`PlaceholderOccurrence` struct fields, not capture groups, so the named-group conversion is invisible to
them).

**ACCEPT (not REJECT) — decided in spec §3.2, R0-GREEN:** the prefix form `[fp/path]KEY` is BIP-380-canonical;
the toolkit's OWN `--help` advertises it (`bundle.rs:2300` LIVE: `"… Override per-placeholder with
[fp/path]@N or --slot @N.path=m/…"`); the prefix form is already RECOGNIZED as a non-bare-tr key form by a
sibling detector (`detect_bare_tr`'s regex is `^tr\([a-z][a-z_0-9]*\(` `:339` and `substitute_nums_sentinel`'s
is `tr\(NUMS\b` `:317` — neither EXTRACTS `[fp/path]@N`; the prefix form appears only in `detect_bare_tr`'s
doc-comment `:329` as a form it does NOT match — m-1, the spec's "already parses" wording was overstated, so
this is supporting rationale for ACCEPT, NOT a parse-the-bracket precedent the impl relies on); and ALL 20
existing `lex_placeholders` test call sites + every internal producer use the SUFFIX form, so an additive
prefix ALTERNATION regresses nothing.

**Why funds-loss (BIP-380, CONFIRMED):** BIP-380 key-origin is a PREFIX `[fingerprint/path]KEY`. A user/tool
following the standard writes `[fp/path]@N` → today the origin is silently dropped (slot xpub built at the
default path) AND the per-`@N` master-fingerprint cross-check is bypassed.

### 3.2 Composition edges — PINNED (spec §3.3, R0 round-1 I2 + round-2 M-3)

- **(i) 8-hex fingerprint MANDATORY inside the prefix bracket.** caps-1 of the prefix group is
  `([0-9a-fA-F]{8})` (exactly 8 hex, NOT optional) — byte-mirroring the suffix grammar. A path-only bracket
  (`[/84'/0'/0']@0`, no fp) does NOT match the prefix alternation and is **rejected as malformed** (typed
  `DescriptorParse`), the SAME outcome as the suffix form `@0[/84'/0'/0']`. Do NOT relax caps-1 to an
  optional fp.
- **(ii) prefix-fp vs `--slot @N.fingerprint=` must AGREE; mismatch ⇒ REFUSE.** The phrase/entropy arm's
  existing cross-check is `if let Some(anno) = anno_fp {` at `bundle.rs:1617`, `if anno != master_fp {` at
  `:1618`, typed `DescriptorParse` error at `:1620` (m-4: the block is `:1617-1620`; the **compare-and-error
  shape to model is `:1618-1620`**). It fires only where a `master_fp` is derived and does **NOT** reach
  xpub slots. The xpub-slot arm (`:1637`) resolves `fp` via the explicit `--slot @N.fingerprint=`
  **`.or(anno_fp)`** at `:1654` with **NO equality check** — so today a prefix-anno fp that DISAGREES with an
  explicit `--slot @N.fingerprint=` is silently resolved to the slot value. **The implementer MUST ADD the
  explicit equality check at the `:1654` site** (refuse on mismatch, modeled on the `:1618-1620`
  compare-and-`DescriptorParse` shape and the Row-19 inline-path-vs-`--slot @N.path=` precedent
  `bundle.rs:1516-1530`). The "confirm existing covers it" branch is UNAVAILABLE for xpub slots.
- **(both-positions) prefix AND suffix bracket on the same `@N` ⇒ REFUSE** (typed `DescriptorParse`,
  ambiguous double-origin — never silently pick one). Implemented in the regex-shape choice (a) below.

### 3.3 Regex shape — RESOLVED: ALL-NAMED capture groups, accessed BY NAME (C1, §11-Q3 closed)

**§11-Q3 is RESOLVED here, not deferred. The plan PINS the all-named-group design.** The previously-drafted
"(a) separate optional prefix group, prefix caps become 6/7, existing 1–5 unchanged" claim **was FALSE and is
DELETED**: in the `regex` crate capture-group numbers are assigned strictly by **opening-paren order, left to
right** (R0-verified by execution). A prefix group whose `(` opens BEFORE `@(\d+)` necessarily takes indices
1/2 and shifts every existing group +2 — so the H13 multipath validator's `caps.get(4)` (`:119`) would read
the suffix-path group, the validator would never see `<0';1'>`, and **the cycle-1 hardened-multipath reject
would SILENTLY STOP FIRING** (a funds-safety regression: an un-restorable hardened-multipath xpub card would
collapse to a bare `/*`). Named groups do NOT rescue numeric indices either (numbering is still by paren
order even with `(?P<name>…)`) — the fix is to stop indexing by number entirely and access BY NAME, which the
R0 verified is **position-independent** (the prefix bracket can sit anywhere and the multipath accessor still
reads the multipath body).

**PINNED named-group regex for `lex_placeholders`** (existing groups named, NEW prefix-origin added, both
brackets `[…]` non-overlapping by class with the multipath `<…>` per §3.4):

```text
(?:\[(?P<pfx_fp>[0-9a-fA-F]{8})(?P<pfx_path>(?:/\d+(?:'|h)?)*)\])?@(?P<idx>\d+)(?:\[(?P<sfx_fp>[0-9a-fA-F]{8})(?P<sfx_path>(?:/\d+(?:'|h)?)*)\])?(?:/<(?P<mpath>[^>]*)>)?(?P<wild>/\*(?:'|h)?)?
```

Group-name map (every name unique; access purely by name — numeric index of each name is irrelevant):

| name | sub-pattern | role | consumer rewrite (was → now) |
|---|---|---|---|
| `pfx_fp` | `([0-9a-fA-F]{8})` | **NEW** prefix fingerprint | `caps.name("pfx_fp")` (new) |
| `pfx_path` | `((?:/\d+(?:'\|h)?)*)` | **NEW** prefix origin path | `caps.name("pfx_path")` (new) |
| `idx` | `(\d+)` | placeholder index | `caps[1]` (`:89/:90`) → `caps.name("idx")` |
| `sfx_fp` | `([0-9a-fA-F]{8})` | suffix fingerprint | `caps.get(2)` (`:93`) → `caps.name("sfx_fp")` |
| `sfx_path` | `((?:/\d+(?:'\|h)?)*)` | suffix origin path | `caps.get(3)` (`:104`) → `caps.name("sfx_path")` |
| `mpath` | `([^>]*)` | **H13 multipath body / strict validator `:119-152`** | `caps.get(4)` (`:119`) → `caps.name("mpath")` (validator body byte-identical; accessor only) |
| `wild` | `(/\*(?:'\|h)?)` | wildcard-hardened | `caps.get(5)` (`:152`) → `caps.name("wild")` |

**Fold logic after capture:** the per-`@N` fp/path is taken from whichever of {prefix, suffix} is present:
`fingerprint_anno`/`origin_path_anno` source from `pfx_fp`/`pfx_path` when present, else `sfx_fp`/`sfx_path`.
**If BOTH a `pfx_*` AND a `sfx_*` bracket are present on the same `@N` ⇒ typed `DescriptorParse` refuse**
(ambiguous double-origin — never silently pick one; §3.2 both-positions edge). The prefix fp inside the
bracket is MANDATORY 8-hex (`pfx_fp` is non-optional inside the optional bracket) — a path-only prefix
(`[/84'/0'/0']@0`) does NOT match the prefix alternation and is rejected as malformed, mirroring the suffix
grammar (§3.2(i)).

**`substitute_synthetic` strip regex (`:369-371`) — NO collision; LEFT NUMERIC.** Its sole consumer is
`caps[1]` (index, `:382`/`:393`) and its bracket parts are already non-capturing `(?:…)`. Mirror the prefix
alternation as a **non-capturing** `(?:\[[0-9a-fA-F]{8}(?:/\d+(?:'|h)?)*\])?` prepended BEFORE `@(\d+)`; this
keeps `caps[1]`=index (R0-verified by execution). Do NOT name it (single group; consistency is not worth a
needless edit), and do NOT make the prefix group capturing.

### 3.4 CRITICAL constraint — preserve cycle-1 H13's hardened-multipath reject (the C1 funds-safety hazard)

H13 (`080ac03e` + `1e1e3f3d`) rewrote ONLY the **multipath segment** (`(?:/<([^>]*)>)?`, the live `caps.get(4)`
body, captured permissively then strictly validated at `parse_descriptor.rs:119-152` to REJECT hardened
`<0';1'>` / malformed `<0'';1>`). The origin `[fp/path]` block (the `[...]` brackets) is ORTHOGONAL to the
multipath block (the `<...>` brackets) — disjoint bracket classes, cannot overlap.

**The C1 hazard:** adding a prefix `[...]` alternation BEFORE `@(\d+)` with PLAIN capturing groups would
renumber the multipath body from group 4 → group 6, so the validator's `caps.get(4)` would silently read the
suffix-path group, never see `<0';1'>`, and **the H13 reject would stop firing** — a funds-safety regression.
**§3.3 closes this by converting to all-NAMED groups accessed by name:** the validator reads
`caps.name("mpath")` regardless of where the prefix bracket sits, so prepending the prefix can NEVER shift it.
The validator BODY (`:119-152`) stays **byte-identical**; ONLY its accessor changes `.get(4)` → `.name("mpath")`
(the in-scope WS-C edit, §3.1). The §3.5 test set MUST pin that the H13 reject still fires for both
`@0/<0';1'>/*` AND a prefix-annotated `[fp/path]@0/<0';1'>/*` — that pair is the guard proving this Critical
is closed (§3.5 test 9).

### 3.5 Phase / TDD plan (single subagent, RED first)

**Phase C1 — RED tests, then GREEN impl** (two regexes + the xpub-slot equality check).

Tests:
1. **Lex parity (unit):** `lex_placeholders("wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)")` →
   `occs[0].fingerprint_anno == Some(deadbeef)`, `origin_path_anno == Some(m/84'/0'/0')` — identical to the
   suffix `wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)`.
2. **Strip parity (unit):** `substitute_synthetic` of the prefix form yields the SAME bare-xpub descriptor as
   the suffix form (no leaked bracket).
3. **fp cross-check fires on a phrase slot (funds-safety):** `bundle --slot @0.phrase=<seed> --descriptor
   "wpkh([WRONGFP/84'/0'/0']@0/<0;1>/*)"` → exit ≠ 0 with the fp-mismatch error (`bundle.rs:1620`), identical
   to the suffix form. (Today the prefix form exits 0.)
4. **Edge (i) — prefix path-but-no-fp ⇒ same rejection as suffix:**
   `lex_placeholders("wpkh([/84'/0'/0']@0/<0;1>/*)")` → SAME malformed error as the suffix path-only form,
   NOT exit-0 with the bracket silently dropped.
5. **Edge (ii) — prefix-fp vs `--slot @N.fingerprint=` (xpub slot, the ADDED check):**
   `bundle --slot @0.xpub=<xpub> --slot @0.fingerprint=<FP_A> --descriptor
   "wpkh([FP_B/84'/0'/0']@0/<0;1>/*)"` with `FP_A != FP_B` → exit ≠ 0 (the ADDED `:1654` equality check);
   the agreeing case (`FP_A == FP_B`) → exit 0. **Without the added check this test RED's** (today the
   prefix-anno fp is silently overridden by the `--slot` value).
6. **Both-positions present ⇒ refuse:** a descriptor carrying BOTH a prefix and a suffix bracket on the same
   `@N` → typed `DescriptorParse` refusal.
7. **Round-trip prefix ≡ suffix → byte-identical md1/mk1:** `bundle` of the prefix form and the suffix form
   (same fp/path) produce byte-identical md1 + mk1 cards.
8. **verify-bundle:** the prefix form carries the origin through verify-bundle's reparse (a verify-bundle
   test exercising the shared lexer at `verify_bundle.rs:1342`).
9. **H13 NON-REGRESSION (CRITICAL — the guard that this Critical is actually closed; MANDATED, MUST
   stay/strengthen after the named-group conversion).** Three assertions, all proving the validator still
   reads the multipath group after the `.get(4)` → `.name("mpath")` accessor change:
   - (a) **Bare hardened multipath STILL rejects:** `lex_placeholders("wpkh(@0/<0';1'>/*)")` (and the existing
     multisig oracle `lex_placeholders("wsh(multi(2,@0/<0';1'>/*,@1/<0';1'>/*))")`, the `:1503`-region test)
     STILL errors with the hardened-multipath message — i.e. the named-group conversion did not detach the
     validator from the multipath body.
   - (b) **Prefix-annotated hardened multipath STILL rejects:** a NEW test
     `lex_placeholders("wpkh([deadbeef/84'/0'/0']@0/<0';1'>/*)")` errors with the SAME hardened-multipath
     message — proving the prefix alternation did NOT shift the multipath group out from under the validator.
   - These two together are the direct guard for C1: if a future edit reverts to plain numeric groups (or
     mis-numbers a consumer), BOTH (a) and (b) re-RED, catching the silent-stop-firing regression at the gate.

**Phase C1 GREEN impl:** the named-group `lex_placeholders` regex + its 5 in-fn `caps.name()` consumer
rewrites (INCLUDING the H13 validator accessor `.get(4)` → `.name("mpath")`, §3.1/§3.3) + the non-capturing
prefix prepend on the `substitute_synthetic` strip regex (§3.1/§3.3) + the xpub-slot equality check
(§3.1/§3.2(ii)).

### 3.6 Cycle-1 zone interaction (clean off `f9467cc5`)

H7 edits the exact files cycle-1's H12/H1/H13 merged into. Off `f9467cc5` (post-merge) those changes are
already present — a within-file ordering note, NOT a merge conflict. Branch off `f9467cc5`, leave the H13
group-4 multipath capture/validator byte-identical, add the orthogonal prefix-origin alternation.

---

## 4. Per-phase exit criteria (all three workstreams)

Each workstream's single phase is GREEN only when ALL hold:
1. Tests written FIRST and observed RED, then GREEN.
2. **Full `cargo test -p mnemonic-toolkit` passes** (NOT just the new tests — per the R0-runs-full-suite
   rule; catches cross-module regressions, especially for WS-C which touches the shared lexer and WS-B which
   touches `error.rs`'s exhaustive match blocks).
3. No new clippy warnings; the existing CI gates pass. **WS-A note: NEVER `cargo fmt`** the toolkit (mlock
   g6 — see MEMORY); make targeted edits only.
4. The exhaustive `match self` blocks in `error.rs` (WS-B) compile (the new variant is handled in
   `exit_code`/`kind`/`message` and any exhaustive test table).

**Mandatory post-implementation review (non-deferrable, CLAUDE.md).** After all three workstreams are GREEN,
a SINGLE independent adversarial execution review runs over the WHOLE combined diff (R0 = plan correctness;
this catches implementation-introduced regressions TDD misses). Persist it verbatim to
`design/agent-reports/cycle2-funds-loss-fixes-impl-review-round{1,2,…}.md` BEFORE the fold-and-commit step;
loop to 0C/0I. If Agent-API dispatch fails mid-session, flag it explicitly and defer the formal review to API
recovery — never silently substitute inline self-review.

---

## 5. Concurrency model

The three workstreams are file-disjoint (§0) ⇒ **three concurrent single-subagent workstreams**, one
subagent per phase, TDD, each in its own worktree (§7). NEVER parallel re-impls of the same finding. No
inter-WS ordering dependency (H7 rides the already-merged H13 reject because all three branch off
`f9467cc5`). Per CLAUDE.md: this plan-doc + R0 GREEN → per-WS single-subagent TDD → mandatory post-impl
whole-diff review.

---

## 6. Resolution of the 3 round-3 informational Minors (citation hygiene)

The spec is R0-GREEN; round-3 carried 3 informational Minors for the plan-doc author. Each resolved here:

- **m-i (H10, citation refresh):** the round-2 review's escape-table cited the import-json taproot refusal at
  `export_wallet.rs:733` and the legacy `sh(multi)` refusal at `mod.rs:281`. **This plan-doc uses the LIVE
  lines — `export_wallet.rs:741-744` (taproot, `WalletScriptType::P2tr | P2trMulti`) and
  `wallet_export/mod.rs:275-276` (`ShInner::Ms` legacy bare P2SH)** — both re-verified on master (§2.2). The
  spec body itself never cited stale lines for these (it relied on "refused upstream" without numbers); the
  plan-doc now pins the live numbers.
- **m-ii (H10, `descriptor_is_general_policy` line):** the general-policy refusal is cited with its live
  lines — the call at **`export_wallet.rs:798`**, defined at **`wallet_export/mod.rs:301`** (§2.2,
  verified).
- **m-iii (H8, master-fp pair):** the `1b6aef92` / `73c5da0a` pair is **documentation-only**; the H8 RED
  assertion is COMPUTE-don't-hardcode (derive the divergence in-test; never hard-code the literal as the
  assertion RHS). Pinned in §1.2 test 2.

No spec edit is required (these are plan-doc-author hygiene items); they are discharged by the live citations
above.

---

## 7. Branch / worktree plan

**Base:** all three off `origin/master` = **`f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5`** (0.61.0).
**Do NOT touch the working tree** — it is checked out on the other instance's branch
`feature/own-account-subset-search` (`364d296f`) and is paused/renumbering. Use fresh worktrees.

| WS | branch | worktree | files |
|---|---|---|---|
| WS-A / H8 | `feature/cycle2-h8-template-language` | `../mt-cycle2-h8` | `synthesize.rs` |
| WS-B / H10 | `feature/cycle2-h10-export-unsorted-multi` | `../mt-cycle2-h10` | `error.rs`, `cmd/export_wallet.rs` |
| WS-C / H7 | `feature/cycle2-h7-prefix-origin-lex` | `../mt-cycle2-h7` | `parse_descriptor.rs`, `cmd/bundle.rs`, `cmd/verify_bundle.rs` |

Each worktree: `git worktree add -b <branch> <path> f9467cc5`. Stage paths explicitly (NO `git add -A`).
Commit the design trail (this plan-doc + the spec + the 3 spec R0 reviews + the impl reviews) on a design
branch or with the first workstream — per the repo convention that design artifacts and per-phase reviews
persist to `design/`. Integration: merge the three branches (disjoint files ⇒ trivial), bump the version
(§8) on the integration branch, run the release-site ritual + full suite + fuzz before tag.

---

## 8. SemVer / lockstep / release ritual

- **toolkit MINOR off 0.61.0** — all three are behavior changes (corrected ms1 language / new typed refusal /
  newly honored annotation). One MINOR bump if batched. **Resolve the exact number at release** — the paused
  own-account cycle is renumbering; do NOT hard-code (the working tree's `0.60.0` is that paused cycle's
  number, not this baseline).
- **Release version-site ritual (CLAUDE.md / MEMORY — NOT gate-enforced; drifts silently). Explicit
  enumerated checklist (R0-M6):**
  1. bump `crates/mnemonic-toolkit/Cargo.toml` version;
  2. update **BOTH READMEs** (repo-root + `crates/mnemonic-toolkit/`) install-pin;
  3. update `fuzz/Cargo.lock` (the `cfg(fuzzing)` self-pin);
  4. re-run the FULL test suite + the fuzz harness AFTER the bump, BEFORE tag.
  (None of 2–3 has a CI gate; a missed self-pin surfaces only later.)
- **GUI schema-mirror: NO leg.** None of the three adds/removes/renames a clap flag, subcommand, or dropdown
  value. H10 = pure refusal (no flag; the new `ToolkitError` variant is NOT in the `gui-schema` flag-name
  set — the gate is flag-NAME parity, not error kinds); H8 = private-fn sig; H7 = lexer-internal.
  `mnemonic-gui/src/schema/mnemonic.rs` is UNTOUCHED.
- **Manual: NO flag-table leg** (no flag added/removed → `docs/manual/tests/lint.sh` bidirectional
  flag-coverage check unaffected). OPTIONAL prose: a one-line H10-refusal note under
  `docs/manual/src/40-cli-reference/` is courtesy, not required (§11 Open-Q4).
- **No codec-publish dependency** — all three toolkit-only (no md/mk/ms tag→pin chain).

### 8.1 FOLLOWUP slugs to FILE (in `design/FOLLOWUPS.md`; flip to RESOLVED in the shipping commit per the
status-discipline rule)

- `template-form-md1-drops-bip39-wordlist-language` (H8)
- `export-wallet-unsorted-multi-silent-sortedmulti-coercion` (H10) — note "PURE REFUSAL, no flag"
- `descriptor-prefix-form-origin-annotation-ignored` (H7) — note "ACCEPT both positions; preserves cycle-1
  H13 multipath reject"
- **(OPTIONAL, NOT this cycle — round-2 I-A)** `export-wallet-direct-descriptor-unsorted-multi-generic-refusal`:
  the direct `export-wallet --descriptor 'wsh(multi(…))'` path resolves `template_opt = None` and is refused
  by the emitters' generic `BadInput` (funds-safe — no silent coercion), NOT the new typed error. A future
  cosmetic upgrade could classify the `template == None` direct path via the parsed descriptor (`Tag::Multi`
  / `script_type`) to surface the typed message there too. Cosmetic only; deliberately OUT of cycle-2 scope.

The bug-hunt checklist items H8/H10/H7 in `constellation-bughunt-2026-06-20.md` get ticked with the fixing
commit SHA per that file's contract.

### 8.2 Alphabetical-error-variant rule (CLAUDE.md)

`ExportWalletUnsortedMultisigUnsupported` is the only new variant. It slots AFTER
`ExportWalletTaprootMultisigUnsupported` and BEFORE `FutureFormat` in the enum AND each exhaustive `match
self` arm (`exit_code`/`kind`/`message`), per §2.1. (The pre-v0.27.2 region is not yet globally sorted —
FOLLOWUP `error-rs-retroactive-alphabetical-sort` — but the new variant follows the rule relative to the
locally-alphabetical `ExportWallet*` cluster.)

---

## 9. Confirmation matrix (no-GUI / no-manual / no-codec)

| concern | H8 | H10 | H7 |
|---|---|---|---|
| new clap flag? | NO (private fn sig) | NO (pure refusal) | NO (lexer-internal) |
| GUI schema_mirror leg? | NO | NO | NO |
| manual flag-table leg? | NO | NO (optional prose) | NO |
| codec tag→pin chain? | NO | NO | NO |
| new `ToolkitError` variant? | NO | YES (1, struct form, alphabetical) | NO |
| differential-oracle row needed? | NO (ms-decode unit) | NO (behavioral; cite existing row) | NO (unit + round-trip) |
| edits `restore.rs`? | NO | NO (calls shared `emit_payload` only) | NO |

---

## 10. Spec→plan fold trace

This plan-doc operationalizes the R0-GREEN spec verbatim where the spec is decisive, and adds:
- Exact LIVE line numbers re-verified on `f9467cc5` for every change site (§1.1/§2.1/§2.2/§3.1).
- The 3 round-3 Minors discharged with live citations (§6).
- The branch/worktree plan off `f9467cc5`, the other-instance branch left untouched (§7).
- The enumerated release-site ritual + FOLLOWUP slugs + alphabetical rule (§8).
- Per-phase TDD + full-suite exit criteria + the mandatory post-impl whole-diff review (§4).

No spec decision is re-litigated here (ACCEPT prefix / thread `run_language` / pure-refusal `{WshMulti,
ShWshMulti}` predicate are all R0-GREEN and carried forward unchanged).

---

## 11. Open questions for the plan-doc R0 reviewer

1. **(H10 format set)** Refuse-set = {Electrum, Coldcard, ColdcardMultisig, Jade} vs {Electrum,
   ColdcardMultisig, Jade}? Can the generic `coldcard` single-sig alias ever carry an unsorted-multi
   descriptor (it should not — singlesig templates route through `coldcard`, multisig through
   `coldcard-multisig`)? Gating `Coldcard` in is harmless; confirm or drop.
2. **(H10 message)** Finalize the byte-exact refusal string (§2.4) and whether to name all three faithful
   formats or just `descriptor`. Route via a `wallet_export` helper (matching the taproot precedent) or inline
   in the `message()` arm?
3. **(H7 regex) — RESOLVED in §3.3 (plan-doc R0 round-1 C1).** NO LONGER OPEN. The earlier "(a) prefix caps
   become 6/7, existing 1–5 unchanged" claim was FALSE (regex numbering is by paren order, so a prefix group
   before `@(\d+)` shifts every existing group +2 → the H13 multipath validator's `caps.get(4)` would read
   the wrong group and the hardened-multipath reject would silently stop firing — funds-safety). **RESOLVED:
   convert `lex_placeholders` to ALL-NAMED groups accessed BY NAME (position-independent, R0-verified by
   execution); the H13 validator reads `caps.name("mpath")` and can never be shifted.** See §3.3 for the
   pinned regex + group-name map, §3.4 for the H13 constraint, and §3.5 test 9 for the non-regression guard.
4. **(H10 manual)** File the optional one-line H10-refusal prose note under `docs/manual/src/40-cli-reference/`
   now, or defer entirely? (No flag-gate either way.)
5. **(error variant shape)** Confirm the struct form `{ format }` (this plan's pin) over the tuple form
   `(&'static str)` (1:1 with the taproot precedent) — or leave it implementer's choice with the four arms
   kept consistent (§2.1)?
6. **(WS-B test scaffolding)** The direct-`--descriptor` test (§2.6 test 3) and the restore-path test (test 7)
   assert `kind()` boundaries across subcommands — confirm the test harness can reach `kind()` on the
   returned `ToolkitError` for both `export-wallet` and `restore` invocations (vs only asserting process exit
   code), so the typed-vs-generic distinction is actually pinned.

---

## 12. Plan-doc R0 round-1 fold log

Source review: `design/agent-reports/cycle2-funds-loss-fixes-plan-R0-round1.md` (VERDICT NOT-GREEN, 1 Critical /
0 Important, 4 Minor). All findings folded below; re-dispatch the architect after this fold (reviewer-loop
continues).

| finding | summary | resolution in this plan-doc |
|---|---|---|
| **C1** (funds-safety Critical) | H7's prefix-origin group prepended BEFORE `@(\d+)` renumbers every existing capture group +2 (numbering is by paren order, NOT the plan's claimed "6/7, existing 1–5 unchanged"), so the H13 hardened-multipath validator's `caps.get(4)` (`:119`) would read the suffix-path group → the cycle-1 hardened-multipath reject **silently stops firing** (un-restorable xpub card collapses to bare `/*`). §11-Q3 wrongly DEFERRED this to the reviewer. | **RESOLVED, not deferred.** §3.3 rewritten: the FALSE "6/7" claim is **DELETED**; `lex_placeholders` is converted to **ALL-NAMED capture groups accessed BY NAME** (`pfx_fp`/`pfx_path`/`idx`/`sfx_fp`/`sfx_path`/`mpath`/`wild`), R0-verified position-independent so the prefix can never shift the multipath group. §3.1 now mandates rewriting EVERY consumer to `caps.name()` — INCLUDING the H13 validator's accessor `.get(4)` → `.name("mpath")` (validator body byte-identical; WS-C owns the file so in-scope). §3.4 reframed around the C1 hazard + named-group fix. The `substitute_synthetic` strip regex confirmed NO collision (sole consumer `caps[1]`, brackets non-capturing) → **left numeric**, prefix prepended as a NON-CAPTURING group. §3.5 test 9 strengthened to MANDATE both bare `@0/<0';1'>/*` AND prefix `[fp]@0/<0';1'>/*` STILL reject — the guard proving C1 is closed. §11-Q3 marked RESOLVED. |
| **m-1** (H7) | "a sibling detector (`detect_bare_tr`/`substitute_nums_sentinel`) already PARSES the prefix bracket" overstated — those regexes (`^tr\([a-z…]\(` `:339`, `tr\(NUMS\b` `:317`) do NOT extract `[fp/path]@N`; it appears only in `detect_bare_tr`'s doc-comment `:329` as a non-matched form. | §3.1 ACCEPT paragraph tightened: the prefix form is RECOGNIZED-as-non-bare-tr (rationale), NOT parsed; live lines `:317/:329/:339` cited. |
| **m-2** (H8) | §1.1 said `synthesize_unified` is at `:709`; on master it is defined `:994` — the `:709` `mnemonic_lang` computation is a DIFFERENT earlier fn. | §1.1 corrected: `synthesize_unified` defined `:994`; `:709` is a different per-slot emit fn. |
| **m-3** (H10) | §2.2 cited the coldcard-multisig generic `_ =>` `BadInput` refusal at `export_wallet.rs:129-132`; the arm body is `:130-132` (the `match inputs.template {` opens `:120`). | §2.2 corrected to `:130-132`, noting the `match` opens `:120`. |
| **m-4** (H7) | The phrase-arm cross-check spans `:1617` (`if let Some(anno) = anno_fp`), `:1618` (`if anno != master_fp`), error `:1620`; the plan variously cited `:1620`/`:1617-1620`/`:1618-1620`. | §3.2(ii) now pins the full block `:1617-1620` AND names `:1618-1620` as the compare-and-error shape the ADDED xpub-slot check models; §3.1 table already cites `:1618-1620`. |

Everything else in the review's "Items verified CORRECT" list (H8, H10, the H7 non-regex sites, all tests,
SemVer/lockstep, workstream disjointness, FOLLOWUP list, alphabetical-variant rule) is GREEN and carried
forward unchanged.

---

_Plan-doc only. No code, no source edits. All citations verified LIVE against toolkit `origin/master` =
`f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5` (0.61.0) on 2026-06-21. MANDATORY R0 review loop to 0C/0I before
any implementation; re-dispatch the architect after every fold._
