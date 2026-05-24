# BRAINSTORM / SPEC — v0.37.0 — `export-wallet --from-import-json` template auto-derive

**FOLLOWUP:** `export-wallet-from-import-json-template-format-reemit`
**Recon:** `cycle-prep-recon-export-wallet-from-import-json-template-format-reemit-and-manual-prose-command-execution-gate.md`
**Source SHA at authoring:** `36e6bfa` (origin/master, 2026-05-24)
**SemVer:** MINOR → **v0.37.0** (new `export-wallet` behavior; no flag added/removed)
**Sibling cycle (deferred to its own PR):** `manual-prose-command-execution-gate` (Cycle B — lands after this)
**Review status:** ✅ **GREEN (0C/0I) — cleared the mandatory R0 gate.** R0 RED 1C/5I → R1 RED 0C/2I (fold-drift; R1 corrected an R0 misread of the bsms fixture) → R2 RED 0C/1I (one un-propagated §3→§6 fix) → **R3 GREEN**. Reviews persisted at `design/agent-reports/v0_37_0-brainstorm-r{0,1,2,3}-review.md`. All citations re-grepped against `36e6bfa`.

## 0. Core invariant (the contract this cycle establishes)

`export-wallet --from-import-json E --format F` MUST behave **identically to** `export-wallet --template T --format F` where `T = template_from_descriptor(E.bundle.descriptor)`, modulo the three fields the envelope is authoritative for (`wallet_name`, `account`, `threshold`). Same exit-code class; on success, byte-identical output once `--wallet-name`/`--account` are matched. This is the testable contract (§5) and it means the fix introduces **no new per-format accept/refuse logic** — each emitter's existing direct-`--template` acceptance is inherited wholesale.

---

## 1. Problem

`export-wallet --from-import-json <envelope>` re-emits a watch-only wallet from a prior `import-wallet --json` envelope. It works for **descriptor-passthrough** formats (bitcoin-core / bip388 / bsms) and for specter (via `--wallet-name`), but **REFUSES** the four **template-requiring** file-import formats:

- `sparrow.rs:106`, `coldcard.rs:113`, `jade.rs:38`, `electrum.rs:54` each refuse with "…requires `--template`; descriptor passthrough is not supported…".

And `--from-import-json` `conflicts_with_all = ["template", "descriptor"]` (`export_wallet.rs:171`), so the user **cannot supply** the needed template either. Net: sparrow/coldcard/jade/electrum **cannot round-trip** via `--from-import-json` at all.

**Shipped-bug evidence (chapter-45 recipes, all verbatim at `36e6bfa`):** 5 of 6 documented round-trips are impossible as written —

| Recipe | Line | Command tail | Status |
|---|---|---|---|
| specter | `45:405` | `--format specter --wallet-name …` | ✅ works (no template) |
| sparrow | `45:313` | `--format sparrow --template bip84` | ❌ clap conflict |
| coldcard (singlesig) | `45:481` | `--format coldcard --template bip84` | ❌ clap conflict |
| coldcard (multisig) | `45:564` | `--format coldcard --template wsh-sortedmulti --threshold 2` | ❌ clap conflict |
| jade (multisig) | `45:639` | `--format jade --template wsh-sortedmulti --threshold 2` | ❌ clap conflict |
| electrum | `45:752` | `--format electrum --template bip84` | ❌ clap conflict |

This is the v0.28.1-class breakage recurring: a reactive "add `--template`" doc fix invalidated by a later CLI change (the `conflicts_with_all`), shipped silently because the manual lint never runs recipes (→ Cycle B).

## 2. Design

**The envelope already carries an unambiguous descriptor.** `run_from_import_json` parses it (`export_wallet.rs:613` `parsed_ms`). The template-requiring emitters only refuse because `EmitInputs.template` is **hardcoded `None`** (`export_wallet.rs:666`, comment "template is always None for descriptor-mode"). Fix: **derive the template from the parsed descriptor and inject it for the template-requiring formats.**

### 2.1 Derive from the descriptor, NOT from `WalletScriptType`

The FOLLOWUP frames the fix as `WalletScriptType → CliTemplate` and warns of inverse-ambiguity. **That ambiguity is an artifact of routing through the lossy enum** — `script_type_from_descriptor` collapses `wsh(multi)`/`wsh(sortedmulti)` to one `P2wshMulti` (`mod.rs:229`), `sh(wsh(multi))`/`sh(wsh(sortedmulti))` to one `P2shP2wshMulti` (`mod.rs:221`), and `tr(multi_a)`/`tr(sortedmulti_a)` to one `P2trMulti`. The **descriptor string itself is unambiguous** (it literally says `sortedmulti` vs `multi`). Deriving directly from the descriptor dissolves all three ambiguities.

### 2.2 New helper — `template_from_descriptor`

`wallet_export/mod.rs`, new `pub(crate) fn template_from_descriptor(d: &MsDescriptor<DescriptorPublicKey>) -> Result<CliTemplate, ToolkitError>`. Discriminates sorted vs unsorted by substring-checking the **rendered descriptor** (`d.to_string()`) — the same heuristic the existing `script_type_from_descriptor` Tr branch uses (`mod.rs:237`), chosen for robustness across the miniscript-#915 `sortedmulti` representation change that structural walking is sensitive to (M1: the existing `Wsh`/`Sh(Wsh)` arms do NOT render, only `Tr` does; rendering the whole `parsed_ms` is sound here because no taproot reaches this path so `sortedmulti(`/`multi(` are the only multisig tokens present). **`sortedmulti(` MUST be checked before `multi(`** (the latter is a substring of the former):

| Descriptor shape | → `CliTemplate` |
|---|---|
| `Pkh(_)` | `Bip44` |
| `Wpkh(_)` | `Bip84` |
| `Sh(ShInner::Wpkh(_))` | `Bip49` |
| `Sh(ShInner::Wsh(_))`, rendered contains `sortedmulti(` | `ShWshSortedMulti` |
| `Sh(ShInner::Wsh(_))`, else contains `multi(` | `ShWshMulti` |
| `Sh(ShInner::Ms(_))` (bare legacy `sh(multi)`/`sh(sortedmulti)`, → `P2shMulti`) | **`Err(BadInput)`** — no template exists (see 2.4) |
| `Wsh(_)`, rendered contains `sortedmulti(` | `WshSortedMulti` |
| `Wsh(_)`, else contains `multi(` | `WshMulti` |
| `Tr(_)` | **unreachable on this path** (taproot pre-walled, 2.4); defensive `Err(BadInput)` |
| `Bare(_)` | `Err(BadInput)` (already rejected upstream by `script_type_from_descriptor`) |

`CliTemplate` enum (`template.rs:15`) has exactly these 10 variants; there is **no** variant for bare `sh(multi)` (legacy BIP-45 P2SH) → it must error cleanly, never `unwrap`.

### 2.3 Conditional injection — passthrough formats stay `None` (REGRESSION GUARD)

`EmitInputs.template` must **not** be set `Some` unconditionally. Two emitters branch on `template.is_some()` and would change output:
- `bip388.rs:33` — `Some` → `@N/**` placeholder render; `None` → descriptor passthrough. **Different output.** bip388 works today via the `None` path.
- `sparrow.rs:42` — `Some` → multisig threshold-missing check fires.

So derive+inject **only for the formats that would otherwise refuse**. Add `fn format_requires_template(f: CliExportFormat) -> bool` returning `true` for `Sparrow | Coldcard | ColdcardMultisig | Jade | Electrum`, `false` for `BitcoinCore | Bip388 | Bsms | Green | Specter` (the passthrough/template-agnostic set — exactly the formats whose emitters do **not** `inputs.template.ok_or_else(...)`-refuse on `None`; confirmed by R0's full reader audit: refusers are sparrow `:104`, coldcard `:111`, jade `:36`, electrum `:52`; green reads only `script_type`, and bitcoin-core/bsms/specter never read `template`). **(I5)** The predicate lives in **`cmd/export_wallet.rs`** (alongside `CliExportFormat`, defined there at `:22`) — NOT `wallet_export/mod.rs`, which would invert the module layering (`cmd::export_wallet` imports from `wallet_export`, not vice-versa). Use an **exhaustive `match` with no `_` arm** so a future `CliExportFormat` variant forces a compile-time accept/refuse decision (M4-Q1). In `run_from_import_json`:

```rust
let derived_template: Option<CliTemplate> = if format_requires_template(args.format) {
    Some(template_from_descriptor(&parsed_ms)?)
} else {
    None
};
```

and set `template: derived_template` in the `EmitInputs` at `:666`. Passthrough formats receive `None` exactly as today → **zero blast radius** on the currently-working paths.

### 2.4 Taproot stays walled off

`run_from_import_json` hard-refuses `P2tr | P2trMulti` at `export_wallet.rs:629-639` (BadInput, FOLLOWUP `wallet-import-taproot-internal-key`) **before** `EmitInputs` is built. So the derivation only ever sees non-taproot descriptors; `bip86`/`tr-multi-a`/`tr-sortedmulti-a` re-emit remains blocked on that separate FOLLOWUP. **This cycle does not claim to fix taproot round-trips.** The `Tr(_)` arm in `template_from_descriptor` is defensive only (unreachable on this path). The derivation call therefore sits **after** the `:629` taproot refusal in `run_from_import_json`.

### 2.5 Threshold trap — `threshold_user_supplied` (REGRESSION GUARD #2)

`sparrow.rs:43` is the only reader of `threshold_user_supplied` (R0-confirmed): for a multisig template it pushes `MissingField::Threshold` when `!threshold_user_supplied`. The from-import-json `EmitInputs` hardcodes `threshold_user_supplied: false` (`export_wallet.rs:671`) even though the threshold is authoritatively known from `envelope.bundle.multisig.threshold` (`:659`, `let threshold = …map(|m| m.threshold)`). Injecting a multisig template (e.g. `wsh-sortedmulti` for the jade/coldcard-ms recipes) would therefore make sparrow **spuriously refuse**. Fix: set `threshold_user_supplied: threshold.is_some()` on the from-import-json path (the envelope supplying the threshold counts as authoritative — mirrors the direct path's `:454` `threshold_user_supplied: args.threshold.is_some()`).

### 2.6 `coldcard-multisig` singlesig interaction (I4)

`--format coldcard-multisig` has a pre-emit guard in BOTH emit-dispatch blocks (`export_wallet.rs:493-511` direct-`run` path + `:713-735` `run_from_import_json` path; the `_ => Err` arms at `:510`/`:730`) requiring a **multisig** template; singlesig templates hit that arm ("`--format coldcard-multisig` requires a multisig --template … For Coldcard singlesig export use --format coldcard"). (The `collect_missing` dispatches at `:469`/`:687` carry no such guard — the refusal is purely in emit-dispatch, M-R2-a.) Post-fix behavior (both inherited from the direct-`--template` path, per §0):
- multisig envelope (`wsh(sortedmulti)`) → derives `WshSortedMulti` → passes the guard → **emits** ✅.
- singlesig envelope (`wpkh`) → derives `Bip84` → hits the `_ => Err` arm → **refuses** with the existing pointer message (correct, unchanged contract).

This is desired (a singlesig wallet genuinely has no multisig coldcard export); it is NOT a regression. A test cell asserts the singlesig→coldcard-multisig refusal, so the C1 matrix rewrite is unambiguous about which coldcard-multisig cells refuse vs succeed.

## 3. Recipe / manual lockstep (in this same PR)

`docs/manual/src/45-foreign-formats.md`: strip the conflicting `--template …` (and the now-redundant `--threshold …`, which is silently ignored on this path — threshold is envelope-derived at `:659`) from the broken recipes. Each fenced command spans **two lines** — the `mnemonic export-wallet --from-import-json envelope.json \` head and the `--format … --template …` continuation on the NEXT line (I3) — so the strip targets both:

| Format | head line | `--template` token line |
|---|---|---|
| sparrow | `:313` | `:314` |
| coldcard (singlesig) | `:481` | `:482` |
| coldcard (multisig) | `:564` | `:565` |
| jade | `:639` | `:640` |
| electrum | `:752` | `:753` |

After the fix they become e.g. `mnemonic export-wallet --from-import-json envelope.json --format sparrow > sparrow_re.json`. **Plus the prose recipe at `45:577-578`** ("`--format coldcard-multisig --template wsh-sortedmulti --threshold 2` is equivalent on v0.28.4+") is also stale on this path and must be updated (I3). Before locking the plan-doc, **grep the whole chapter** for every `--from-import-json` … `--template` co-occurrence (and the `coldcard-multisig` alias) — do not trust the indexed recipe heads alone. Update the general prose at **`45:347`** ("The export-wallet side requires a recognized `--template`…") to describe auto-derivation. **Leave `45:352-357` unchanged** (M-R1-a): that note is the taproot-gated `--from-import-json` round-trip caveat, which stays accurate (§2.4 keeps taproot walled). **No GUI `schema_mirror` lockstep** (no clap flag-name change). The `40-cli-reference/41-mnemonic.md:669` `--from-import-json` row gets a one-line "auto-derives the template for file-import formats" note (M5).

## 4. Edge cases / errors

- `sh(multi)`/`sh(sortedmulti)` envelope (→ `P2shMulti`) to a template-requiring format → clean `BadInput` ("legacy bare P2SH multisig has no `export-wallet` template; use `--format bitcoin-core` for descriptor passthrough"). To a passthrough format → unchanged (derivation not attempted).
- Taproot envelope → still the existing `:629` BadInput (unchanged).
- Passthrough format + any envelope → byte-identical to today (template stays `None`).

## 5. Test plan (BIN-target — `wallet_export`/`export_wallet` tests run under the bin, not `--lib`)

All in `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs` (the existing 1209-line suite).

### 5.1 C1 — rewrite the inverted pinned-refusal cells (the dominant work item)
The fix deliberately changes the contract these cells pin. Rewrite them to assert the §0 **equivalence invariant** rather than blanket refusal:
- **`p11c_refusal_matrix_strict_template_only_dests`** (`:841-874`, `assert_eq!(cell_count, 40)`): replace "every cell refuses" with, for each (src, dest) cell, **assert it matches the equivalent direct `export-wallet --template <derived> --format <dest>` invocation** (same exit-code class; on success, equal output per 5.3). Self-defining against real emitter behavior → no hand-maintained 40-cell truth table. **All 8 `ALL_SOURCES` (`:563`) resolve to one of two templates** (verified against the fixture map `:539-558`): the 3 singlesig sources (`bitcoin-core`/`coldcard`/`electrum`, all bip84 `wpkh`) → `bip84`; the **5** `wsh(sortedmulti)` sources (`bsms`/`coldcard-multisig`/`jade`/`sparrow`/`specter`) → `wsh-sortedmulti`. **No source resolves to P2shMulti** — the bsms fixture (`bsms-2line-sortedmulti-2of3.txt`) is `wsh(sortedmulti(2,…))`, NOT `sh(multi)` (R1 correction of an R0 misread). So bsms is not a special case: bsms behaves identically to the other 4 multisig sources.
- **`p11a_helper_returns_nonzero_exit_on_template_only_dest_refusal`** (`:611`): re-point to a cell that still genuinely refuses post-fix. The only such cell type reachable through `run_export_from_import_envelope` is **singlesig → coldcard-multisig** (e.g. `bitcoin-core → coldcard-multisig`), which refuses via the §2.6 `_ => Err` arm (`:511`). (There is NO `sh(multi)`/P2shMulti source in `happy_path_fixture`, so a P2shMulti-source refusal cell is unbuildable without authoring a new fixture — do not attempt it.)
- **Cell 3 `…refuses_with_helpful_message`** (`:96-119`): its `envelope_v0_27_0.json` IS `sh(multi)` → `P2shMulti` (this is the *only* P2shMulti path in the suite, and Cell 3 invokes the binary directly at `:101-118`, not via the matrix helper). After the fix it refuses via `template_from_descriptor`'s P2shMulti `Err` whose message contains NEITHER currently-asserted substring. Update **Cell 3's own inline assertion (`:114-117`)** to the new "legacy bare P2SH … has no export-wallet template" message.
- **`REFUSAL_STDERR_PATTERNS`** (`:814-822`): **add the jade-singlesig refusal literal** "`emits multisig wallet config only`" (`jade.rs:62`). Post-fix, the 3 singlesig sources → `jade` now refuse via jade's *singlesig* refusal (template IS set, so it bypasses the old `requires --template` path) with a message NOT in the current set — so without this add, those 3 cells fail the pattern check. The coldcard-multisig literal "requires a multisig --template" (the other post-fix refusal reason, for singlesig → coldcard-multisig) is **already present** at `:817`. The new P2shMulti literal is NOT reachable by any matrix cell (P2shMulti is exercised only by Cell 3's inline assertion, above) — it belongs in Cell 3, not this shared const.

**The verified post-fix outcome table** (8 sources × 5 dests; source template class derived from §2.2): the 3 singlesig sources (`bitcoin-core`/`coldcard`/`electrum` → bip84) SUCCEED for `{sparrow, coldcard, electrum}`, REFUSE for `{coldcard-multisig` (multisig-required guard, `:730`), `jade` (singlesig refusal, `jade.rs:53-62`)}; the 5 `wsh(sortedmulti)` sources SUCCEED for ALL 5 dests (coldcard via `emit_coldcard_multisig_text` `coldcard.rs:46`, jade via delegation `jade.rs:43-45`, electrum via `emit_electrum_multisig_json` `electrum.rs:71`). **= 34 succeed / 6 refuse** (the 6 = 3 singlesig × {coldcard-multisig, jade}). p11c asserts each cell's exit-code class against this table (encode `SINGLESIG_SOURCES = {bitcoin-core, coldcard, electrum}`; `expected_refuse = is_singlesig_source(src) && dest ∈ {coldcard-multisig, jade}`).
- **`p11c_green_descriptor_passthrough_singlesig_passes_multisig_refused`** (`:892`) + `p11b_happy_path_matrix` (24 descriptor-capable cells, `:722`): UNCHANGED — green and the descriptor-capable dests stay `template: None` (passthrough), so their behavior is untouched; keep as the regression guard that the passthrough set is byte-stable.

### 5.2 New round-trip success cells
For each newly-unblocked pair {sparrow·bip84, coldcard·bip84, electrum·bip84, coldcard-multisig·wsh-sortedmulti, jade·wsh-sortedmulti, sparrow·wsh-sortedmulti}: import a wallet → `import-wallet --json` → `export-wallet --from-import-json` (no `--template`) succeeds AND equals the direct path per 5.3.

### 5.3 Round-trip equality — handle the `wallet_name` divergence (I2)
The direct path defaults `wallet_name` to `<template-human-name>-<account>` (`:435`), the from-import-json path to `"imported-descriptor"` (`:656`) — and that name is emitted into sparrow `name`/keystore `label` (`sparrow.rs:125,137`), coldcard `Name:` (`coldcard.rs:302,353`), electrum `keystore.label` (`electrum.rs:122,181`). So a naive byte-compare FAILS on the name field. **Resolution (chosen):** pass a matching explicit `--wallet-name X` on BOTH sides, then assert **byte-equality**. (Rejected alt: field-scoped comparison excluding name — more fragile, per-format.) This mirrors why the existing specter recipe at `45:405` passes `--wallet-name` explicitly. **Account match (M-R1-c):** `--account` is rejected on the from-import-json path (`:554`), so the direct side must use the envelope's account — all suite fixtures are account 0 (the direct default is also 0), so `--wallet-name`-only matching suffices; no `--account` on either side. **Implementation refinement (plan M2):** the round-trip test compares re-emit against the *original same-format source file* (import F → re-emit F), NOT against a separately-constructed direct `--template` invocation — the from-import-json path rebuilds slots from the envelope's mk1 cards (`envelope_to_resolved_slots`) while the direct `--template` path builds them from `--slot @N.xpub=` (`bundle::resolve_slots`), two pipelines whose xpub/fingerprint encoding could diverge orthogonally to this feature. §0's byte-identity remains the conceptual contract; the round-trip-against-source equality + the §5.1 exit-class matrix lock it without that fragility.

### 5.4 Unit tests on `template_from_descriptor`
Each reachable row in §2.2: `pkh`→Bip44, `wpkh`→Bip84, `sh(wpkh)`→Bip49, `wsh(multi)`→WshMulti, `wsh(sortedmulti)`→WshSortedMulti, `sh(wsh(multi))`→ShWshMulti, `sh(wsh(sortedmulti))`→ShWshSortedMulti, `sh(multi)`/`sh(sortedmulti)`→`Err`, `bare`→`Err`. Explicit assert that `wsh(sortedmulti)` does NOT mis-resolve to `WshMulti` (the substring-ordering guard).

### 5.5 Passthrough byte-identity regression
bitcoin-core / bip388 / bsms output is byte-identical with and without the change (these stay `template: None`; this is the guard that §2.3's partition didn't accidentally pull a passthrough format into the inject set).

## 6. Phasing

- **Phase 0 — RED:** write 5.4 unit tests + 5.2 round-trip success cells + 5.5 passthrough byte-identity; run, confirm they fail (compile-fail on the missing fn is acceptable RED). **Also rewrite the C1-inverted cells (5.1) in this phase** — they go RED against current `master` behavior the moment the new invariant is asserted; that is the intended RED.
- **Phase 1 — GREEN:** add `template_from_descriptor` (`wallet_export/mod.rs`) + `format_requires_template` (`cmd/export_wallet.rs`, exhaustive match); wire `derived_template` (after the `:629` taproot refusal) and `threshold_user_supplied: threshold.is_some()` into the `:666`/`:671` `EmitInputs`. All of Phase-0's tests (incl. the rewritten C1 cells) pass.
- **Phase 2 — manual lockstep:** strip `--template`/`--threshold` from the 5 recipes (both head+token lines per §3 table) + the `45:577` prose + the `45:347` prose (**leave the `45:352-357` taproot round-trip note unchanged** — §2.4 keeps taproot walled, M-R1-a) + `40-cli-reference/41-mnemonic.md:669` note (make clear the derivation is internal — the user still cannot pass `--template`, M-R1-b). Re-grep the whole chapter for residual `--from-import-json … --template` co-occurrences. (`docs/manual` builds; `make -C docs/manual lint` passes.)
- **Phase 3 — release prep:** Cargo `0.36.4 → 0.37.0` + `Cargo.lock` staged; CHANGELOG; `install.sh` self-pin `TAG=`; FOLLOWUP `Status: open → resolved` (BOTH the CHANGELOG and FOLLOWUPS.md entry); per-phase opus reviews persisted to `design/agent-reports/`.

Each phase: tests-before-impl, per-phase opus reviewer-loop to 0C/0I before advancing. **Mandatory R0 on THIS doc before any code.**

## 7. Open questions — RESOLVED at R0 (architect endorsed all three, M4)

1. **`format_requires_template` partition** — RESOLVED: hand-maintained **exhaustive `match` with no `_` arm** (a future `CliExportFormat` variant forces a compile-time accept/refuse decision). Lives in `cmd/export_wallet.rs` (§2.3, I5). Structural probing rejected as over-engineering.
2. **Keep `conflicts_with_all` as-is** — RESOLVED: yes. User omits `--template`; the descriptor is authoritative. An override flag is YAGNI and would re-introduce the very ambiguity this design dissolves.
3. **Recipe cleanup drops `--threshold`** — RESOLVED: drop it. It is genuinely ignored on this path (threshold is envelope-derived at `:659`); leaving it is misleading.
