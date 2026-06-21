# R0 architect review — cycle-2 funds-loss-fixes brainstorm-spec (round 3)

**Reviewed:** `design/BRAINSTORM_cycle2_funds_loss_fixes.md` (H8 / H10 / H7), post round-2 fold.
**Round-1:** NOT-GREEN — 0 Critical / 2 Important (I1, I2) / 6 Minor (all folded).
**Round-2:** NOT-GREEN — 0 Critical / 1 Important (I-A: wrong `--descriptor` control-flow rationale +
a §2.6 test that would FAIL-as-written) / 3 Minor (M-1/M-2/M-3). All folded.
**This round:** confirm I-A closure AND that the round-2 fold introduced no new Critical/Important.
**Source of truth:** toolkit `origin/master` = `f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5` (0.61.0), via
`git show origin/master:<path>` — NOT the working tree (on `feature/bundle-md1-template-multisig`).
**Reviewer stance:** adversarial / refute-by-default. Every load-bearing code fact re-traced on master.
**Date:** 2026-06-21.

---

## VERDICT: **GREEN — 0 Critical / 0 Important** (3 informational Minor, non-blocking)

I-A is **CLOSED**. The round-2 fold rewrote §2.3 with the correct two-sub-path control-flow, scoped the
typed refusal to the `--template` + `--from-import-json` paths everywhere (§0/§2.3/§2.6/§7/§9.1), pinned
the minimal option (i) (direct `--descriptor` keeps its existing emitter-level generic `BadInput`), and
rewrote the §2.6 direct-`--descriptor` test to assert refusal by ANY error AND `kind() !=
ExportWalletUnsortedMultisigUnsupported` — which is exactly what master does. I independently re-traced
the dispatch and confirmed every fact (trace below). No new Critical or Important was introduced by the
fold. The H10 structured predicate `{WshMulti, ShWshMulti}` remains COMPLETE for under-refuse and
NOT over-refusing. The three round-2 Minor (M-1 restore-path / M-2 citation refresh / M-3 xpub-slot
branch) are each substantively folded. **This clears the spec to the plan-doc stage.**

---

## Critical
**None.**

The make-or-break funds-safety question — *can any unsorted-multi shape reach the
electrum / coldcard / coldcard-multisig / jade emitters and silently become BIP-67 `sortedmulti` while
NOT being matched by the refusal?* — is answered **NO** (Option-A no-silent-coercion-hole, traced
below). Re-verified on master: `CliTemplate` has EXACTLY 10 variants (`template.rs:18-42`) with NO bare
`Multi` / `sh(multi)` / general-policy variant; the only unsorted-multi variants are `WshMulti` /
`ShWshMulti`, both in the refusal set. Every other route to those emitters is a sorted variant (allowed,
correct), taproot (refused by the per-emitter taproot guard), a general policy (refused upstream by
`descriptor_is_general_policy`), legacy `sh(multi)` (refused by `template_from_descriptor`'s `ShInner::Ms`
arm), or `template == None` (refused by each emitter's own `template.ok_or_else` generic `BadInput`). No
silent-coercion path escapes.

---

## Important
**None.** I-A is closed (see the dedicated closure section below). No new Important.

---

## Minor (informational — none block GREEN; for the plan-doc author)

### m-i (H10, M-2 residual) — one stale line cite survives in the round-2 review's escape-table, NOT in the spec
Non-blocking, and it is in the *round-2 review doc*, not the spec: the round-2 escape-set table cited the
import-json taproot refusal at `export_wallet.rs:733` and the legacy `sh(multi)` refusal at `mod.rs:281`;
on master they are `:741-744` (taproot, `WalletScriptType::P2tr | P2trMulti`) and `:275-276`
(`ShInner::Ms` legacy bare P2SH). The **spec itself does not cite either line** — it relies only on
"taproot refused upstream" and "legacy `sh(multi)` refused" without a line number, so there is nothing to
fix in the spec. Flagged only so the plan-doc, if it lifts those citations, uses `:741` / `:275`.

### m-ii (H10) — spec §2.3 cites `descriptor_is_general_policy` only implicitly
The spec leans on the general-policy refusal (the `Wsh(_) => WshMulti` collapse guard) as part of the
"no escape" argument but does not give it a line. Live: `export_wallet.rs:798`
(`if descriptor_is_general_policy(&parsed_ms)`), defined `wallet_export/mod.rs:301`. Optional: the
plan-doc may cite it when enumerating the import-json escape routes. Not load-bearing for the spec's
correctness.

### m-iii (H8) — the `1b6aef92` / `73c5da0a` master-fp pair is still cited as documentation
§1.3 already pins COMPUTE-don't-hardcode (the live assertion derives the divergence in-test; the hex pair
is documentation only). No action — re-flagged only to keep the plan-doc from hard-coding the literals;
the spec text is correct as written.

---

## I-A closure: `--descriptor` control-flow + Option-A funds-completeness

### My own source trace (master `f9467cc5`)

**Dispatch fn — `emit_payload`.** `export_wallet.rs:73` `pub(crate) fn emit_payload(inputs, format)`;
doc-comment `:60-72` states it "Consolidates the formerly-4 byte-identical copies … `run`,
`run_from_import_json`, and restore's single-sig + multisig `build_*import_payload`." It runs
`collect_missing`-first (match `:84-101`) then the `emit` match (`:109-138`). Callers: `run` at
`:625`; `run_from_import_json` at `:845`; restore's two builders. ✓ Matches the spec's "FOUR callers"
framing and the `emit_payload` name (M-1 name fix confirmed).

**`run` has TWO descriptor-bearing sub-paths that resolve the template DIFFERENTLY:**
- `let mut resolved_template … = None;` (`:418`).
- **Direct `--descriptor` arm** (`if let Some(desc) = &args.descriptor`, `:426`): `@N` forms rejected at
  `:443-447` (`is_at_n_form` → `BadInput`); a concrete descriptor is parsed + canonicalized to
  `d.to_string()` — and **`resolved_template` is NEVER assigned in this arm** (it stays `None`).
- **`--template` arm** (the `else`, `:457`): resolves slots, builds the descriptor, and sets
  `resolved_template = Some((resolved, template, k));` at **`:542`** (spec cites `:542` ✓).
- Then `template_opt` = `match &resolved_template { Some((_,tmpl,_)) => Some(*tmpl), None => None }`
  (`:553-560`). So `--template wsh-multi` → `template_opt = Some(WshMulti)`; direct `--descriptor` →
  `template_opt = None`. ✓

**`template_from_descriptor` is called in EXACTLY ONE place.** `grep` on master: the only call is
`export_wallet.rs:812`, inside `run_from_import_json`, gated by `format_requires_template(args.format)`
(`:791`) AND past the general-policy refusal (`:798`). **It is NEVER called in `run`.** ✓ This is the
exact fact round-2 I-A demanded the spec assert, and §2.3 now states it verbatim ("called in EXACTLY ONE
place — `run_from_import_json:812` — and NEVER in `run`").

**Direct-`--descriptor` is funds-safe via the emitters' own `template.ok_or_else`:**
- electrum `electrum.rs:52-54` → `BadInput("--format electrum requires --template; descriptor
  passthrough is not supported by Electrum's wallet-db schema")`.
- jade `jade.rs:36-39` → `BadInput("--format jade requires --template …")`.
- coldcard (generic) `coldcard.rs:111-114` (in `emit_coldcard_generic_json`) → `BadInput("--format
  coldcard requires --template …")`.
- coldcard-multisig `export_wallet.rs:129-132` (`_ =>` arm of the `ColdcardMultisig` match) →
  `BadInput("--format coldcard-multisig requires a multisig --template …")`.

All four refuse `template == None` BEFORE any sortedmulti-emitting code runs. So an unsorted `multi(...)`
on the direct `--descriptor` path is **REFUSED, never silently coerced** — funds-safe with the
pre-existing generic `BadInput` (not the new typed error). ✓ The spec's §2.3/§2.6/§7 now say exactly
this, and §5.1 files the optional (cosmetic-only) FOLLOWUP to upgrade the message to the typed form later
— deliberately out of scope. No residual "WshMulti on `--descriptor`" claim survives anywhere (re-grepped
§0/§2.3/§2.6/§7/§9.1).

**`--from-import-json` typed path:** `template_from_descriptor` (`mod.rs:259`) computes
`is_sorted = d.to_string().contains("sortedmulti(")` (`:264`) and maps `Wsh(_) → WshMulti`/`WshSortedMulti`
(`:279-282`), `Sh(Wsh) → ShWshMulti`/`ShWshSortedMulti` (`:270-273`) — **preserving the unsorted
distinction**, so an unsorted `wsh(multi)` import-json yields `Some(WshMulti)` and IS caught by the new
structured guard (typed exit-2). ✓ The `multi(`-as-substring-of-`sortedmulti(` trap is structurally
avoided (the `:671` test comment in `mod.rs` pins "`sortedmulti(` contains `multi(` — must NOT resolve to
WshMulti"); the guard matches the typed variant, not a string.

### Option-A funds-completeness — every coercion route accounted for

| route to a field-less emitter | resolves to | outcome on master | silent coercion? |
|---|---|---|---|
| `--template wsh-multi` / `sh-wsh-multi` (`run`) | `Some(WshMulti)` / `Some(ShWshMulti)` (`:542`→`:553-560`) | NEW typed guard fires (exit 2) | **NO** ✓ |
| `--from-import-json` unsorted `wsh(multi)` / `sh(wsh(multi))` | `template_from_descriptor` → `Some(WshMulti)` / `Some(ShWshMulti)` (`mod.rs:264/279-282`) | NEW typed guard fires (exit 2) | **NO** ✓ |
| direct `run --descriptor 'wsh(multi(…))'` | `template_opt = None` (`template_from_descriptor` NOT called in `run`) | emitter `template.ok_or_else` generic `BadInput` (exit 1) | **NO** ✓ |
| `--template wsh-sortedmulti` / `sh-wsh-sortedmulti` | `Some(WshSortedMulti)` / `Some(ShWshSortedMulti)` | emits correctly (BIP-67 is what they implement) | n/a (correct) ✓ |
| taproot `tr-multi-a` / `tr-sortedmulti-a` | `Some(TrMultiA)` / `Some(TrSortedMultiA)` | per-emitter taproot guard (jade `:48-52`, coldcard `:268-277`); import-json taproot refused upstream `:741-744` | n/a (refused elsewhere) ✓ |
| import-json general policy | refused at `:798` (`descriptor_is_general_policy`) before template derivation | `BadInput` | n/a (refused) ✓ |
| import-json legacy `sh(multi)` (BIP-45) | `template_from_descriptor` `ShInner::Ms` arm `mod.rs:275-276` | `BadInput` (no export template) | n/a (refused) ✓ |
| restore `build_multisig_import_payload` unsorted `Some(WshMulti)` | `emit_payload` chokepoint | NEW typed guard fires (bonus coverage; M-1) | **NO** ✓ |

**Conclusion: there is NO silent-coercion hole.** Every route by which an unsorted `wsh-multi` /
`sh-wsh-multi` could reach a field-less sortedmulti-emit is either the NEW typed refusal
(`--template` + `--from-import-json`, where `template ∈ {WshMulti, ShWshMulti}`), the pre-existing
emitter-level generic `BadInput` (direct `--descriptor`, `template == None`), or refused upstream. The
refusal set `{WshMulti, ShWshMulti}` is complete; Option A (pure refusal, no flag) achieves
funds-completeness with no silent coercion remaining. **No Critical.**

### §2.6 / test-set accuracy (no fail-as-written test remains)

- **Direct-`--descriptor` test (§2.6, the round-2 fail-as-written one):** now asserts "refusal by ANY
  error (exit ≠ 0)" AND "explicitly that the `kind()` is NOT `ExportWalletUnsortedMultisigUnsupported`
  (it is the generic `BadInput`)". This matches master (the direct path resolves `template_opt = None`,
  hits the emitter `ok_or_else`). ✓ The previously-failing assertion is gone.
- **`--from-import-json` typed-refusal test (§2.6):** asserts exit-2 with the typed
  `ExportWalletUnsortedMultisigUnsupported` `kind()`. Matches the `template_from_descriptor → Some(WshMulti)`
  path. ✓
- **Sortedmulti / multi_a / single-sig NOT-refused regression (§2.6, mandated):** `wsh-sortedmulti` /
  `sh-wsh-sortedmulti` → exit 0; `tr-multi-a` / `tr-sortedmulti-a` → hit the EXISTING taproot refusal
  (assert `kind() == ExportWalletTaprootMultisigUnsupported`, proving disjointness); single-sig /
  descriptor / sparrow → exit 0. Consistent with master's behavior. ✓
- **Restore-path regression note (§2.6, M-1):** unsorted-`WshMulti` md1 → field-less vendor refused via
  the `emit_payload` chokepoint that `restore`'s `build_multisig_import_payload` calls; one-line
  assertion; `restore.rs` NOT edited. ✓

No test in §2.6 would fail-as-written against master.

---

## emit_payload placement across its 4 callers — over-refuse check

The typed refusal sits at the shared `emit_payload` chokepoint, which also serves `restore` and `import`.
Confirmed this does NOT wrongly refuse any legitimate flow:

- The guard matches ONLY resolved `CliTemplate ∈ {WshMulti, ShWshMulti}` for the THREE field-less formats
  {Electrum, Coldcard, ColdcardMultisig, Jade} (`format_requires_template` minus the faithful Sparrow).
  Sorted variants, single-sig, taproot, and the faithful formats (descriptor/bitcoin-core/sparrow/bip388)
  never match — so it does not fire for them on ANY caller.
- Refusing an unsorted-multi → field-less-format export is **correct regardless of entry path** (the
  silent-sortedmulti coercion is funds-loss whether the descriptor arrived via `--template`,
  `--from-import-json`, or `restore`). The restore-path coverage is therefore a desirable funds-safety
  extension (more refusals, never fewer), not an over-refusal.
- **Disjointness holds:** the guard lives in `export_wallet.rs::emit_payload`; `restore.rs` is NOT edited
  (verified — it merely calls the shared fn via `build_multisig_import_payload`, which passes
  `template: Some(t)`). The spec's "export-scoped, restore coverage = extra funds-safe refusals" framing
  is accurate, not hand-waving — the chokepoint placement is the correct call and the broader coverage is
  funds-positive.

---

## M-3 (H7 xpub-slot fp) — confirmed the spec mandates the ADDED explicit comparison

Re-traced on master:
- Phrase/entropy arm cross-check: `bundle.rs:1617` `if let Some(anno) = anno_fp { if anno != master_fp {
  Err(DescriptorParse …) } }`, error text `:1620`. Fires ONLY where a `master_fp` is derived (phrase
  path). ✓
- **xpub-slot arm: `} else if subkeys.contains(&…::Xpub)` at `:1637`**; it computes `fp` as the explicit
  `--slot @N.fingerprint=` **`.or(anno_fp)` at `:1654`** with **NO equality check** — so today a
  prefix-anno fp that DISAGREES with an explicit `--slot @N.fingerprint=` is **silently resolved to the
  slot value** (the `.or()` only falls through when the slot fp is absent). ✓ Exactly as M-3 describes.
- §3.3(ii) + §3.4-edge-(ii) now **mandate the ADDED explicit `prefix-anno-fp vs --slot @N.fingerprint=`
  comparison at the `:1654` site** (refuse on mismatch, same `DescriptorParse` shape as `:1618-1620`,
  modeled on the Row-19 path-conflict precedent `bundle.rs:1516-1525`, `SlotInputViolation
  "path-mismatch"` at `:1523-1524`). The spec explicitly marks the "confirm existing covers it" branch
  UNAVAILABLE for xpub slots. ✓ This gap (the phrase-arm guard `:1617-1620` not reaching xpub slots) is
  correctly described against source. Closed.

---

## Round-1/round-2 settled items — re-confirmed still hold (off `f9467cc5`)

- **Refusal-set completeness `{WshMulti, ShWshMulti}`:** ✓ `CliTemplate` `template.rs:18-42` = 10 variants,
  no bare `Multi`/general variant; the two unsorted-multi variants are both in the set. Complete +
  not over-refusing.
- **H7 ACCEPT zero-regression:** ✓ lex regex `parse_descriptor.rs:82-84` is suffix-only (`[fp/path]`
  block anchored AFTER `@(\d+)`); strip `:369-371` suffix-only; an additive prefix ALTERNATION regresses
  nothing. `--help` at `bundle.rs:2300` advertises `[fp/path]@N` (prefix). `detect_bare_tr` (`:329`) +
  `substitute_nums_sentinel` tests (`:2989/:2992`) already parse the `[fp/path]@N` prefix for NUMS/bare-tr
  — internal precedent strengthening ACCEPT (the spec's §3.2.4 clarifying note is accurate; the *lexer's*
  own tests use suffix).
- **H13 composition non-regression:** ✓ §3.4 mandates BOTH `@0/<0';1'>/*` AND a NEW prefix-annotated
  `[deadbeef/…]@0/<0';1'>/*` still erroring with the hardened-multipath message. The H13 group-4 multipath
  capture/validator (`<...>` brackets) is orthogonal to the origin `[...]` block — disjoint bracket
  classes, cannot overlap.
- **H8 sole-site threading:** ✓ keyed `:547` uses `run_language`; template path hardcodes English at the
  SOLE site `:1265`; caller `synthesize_descriptor:467` has `run_language` in scope and drops it at the
  `:487` call into `synthesize_template_descriptor` (sig `:1158`, 3 params); 3 in-module test call sites
  `:2407/:2443/:2594` `(&descriptor, &cosigners, false)`. Private fn, no CLI surface — no schema-mirror,
  no manual.
- **SemVer toolkit-MINOR + NO GUI/manual leg:** ✓ H8 = private-fn sig; H10 = pure refusal (no clap flag;
  `--allow-sortedmulti-coercion` absent on master; the new `ToolkitError` variant is not in the
  `gui-schema` flag-name set); H7 = lexer-internal. No flag added/removed → no `schema_mirror` leg, no
  manual flag-table leg. README×2 + `fuzz/Cargo.lock` self-pin ritual carried as the §5 explicit checklist
  (M6).
- **Alphabetical variant:** ✓ `error.rs:169` `ExportWalletTaprootMultisigUnsupported(&'static str)`,
  `:170` `FutureFormat`. `ExportWalletUnsortedMultisigUnsupported` sorts AFTER `…Taproot…` (T<U) and
  BEFORE `FutureFormat` (E<F) — inserts at `:169a` in the enum and in each match arm (`exit_code:545`,
  `kind:607`, `user_text:749`). Struct form `{ format }` is a deliberate divergence from the tuple
  precedent, mirroring the `ExportWalletMissingFields { … }` struct-variant arm style (`:543/605/745`);
  the spec correctly labels it "NOT 1:1 shape" and offers the tuple form as an equally-acceptable
  implementer choice.
- **FOLLOWUP list (§5.1):** ✓ four slugs (H8/H10/H7 + the optional direct-`--descriptor` typed-upgrade,
  cosmetic, out of scope) — consistent with the round-2 I-A decision.
- **Fold logs §9 / §9.1:** ✓ accurate. §9.1 I-A row correctly records the two-sub-path control-flow
  correction with the live `:542` / `:812` / emitter-`ok_or_else` cites; M-1 (`emit_payload` name +
  restore-path), M-2 (coldcard taproot block `:268-277`, `emit` match `:109`), M-3 (xpub-slot `.or` at
  `:1654`) each map to a real body edit. The §9 round-1 I1 row was correctly amended to flag the over-
  correction now resolved by I-A.
- **Workstream file-disjointness:** ✓ H8 = `synthesize.rs`; H10 = `export_wallet.rs` + `error.rs` +
  `wallet_export/mod.rs`; H7 = `parse_descriptor.rs` + `bundle.rs` + `verify_bundle.rs`. No overlap; the
  M-1 restore observation does NOT add `restore.rs` to any zone.

---

## Verdict restated

**GREEN — 0 Critical / 0 Important.** I-A is closed: the direct-`run --descriptor` control-flow is now
stated correctly (resolves `template_opt = None`; `template_from_descriptor` only in
`run_from_import_json:812`), the typed refusal is scoped to the `--template` + `--from-import-json` paths,
and the §2.6 direct-`--descriptor` test asserts the existing generic `BadInput` (not the typed kind) —
matching master. The Option-A no-silent-coercion-hole conclusion holds: the refusal set
`{WshMulti, ShWshMulti}` is complete, with no unsorted-multi shape reaching a field-less emitter and
silently becoming BIP-67 sortedmulti. The three Minor are informational (citation hygiene for the
plan-doc) and do not block. **This spec is cleared to advance to the plan-doc stage** (which itself
carries the mandatory R0 loop per CLAUDE.md).
