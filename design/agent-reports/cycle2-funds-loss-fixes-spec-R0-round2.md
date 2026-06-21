# R0 architect review — cycle-2 funds-loss-fixes brainstorm-spec (round 2)

**Reviewed:** `design/BRAINSTORM_cycle2_funds_loss_fixes.md` (H8 / H10 / H7), post round-1 fold.
**Round-1:** NOT-GREEN — 0 Critical / 2 Important (I1 H10 unpinned predicate + wrong rationale; I2 H7
edges) / 6 Minor. Both Important folded; this round confirms closure AND that the fold introduced no new
Critical/Important.
**Source of truth:** toolkit `origin/master` = `f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5` (0.61.0), via
`git show origin/master:<path>` — NOT the working tree (on `feature/bundle-md1-template-multisig`).
**Reviewer stance:** adversarial / refute-by-default. Every code fact re-grepped on master.
**Date:** 2026-06-21.

---

## VERDICT: **NOT-GREEN** — 0 Critical / **1 Important** / 3 Minor

I1 and I2 are both substantively CLOSED at the funds-safety level: the H10 structured predicate
`CliTemplate ∈ {WshMulti, ShWshMulti}` is **complete** (no unsorted-multi shape escapes it into a
silent sortedmulti coercion — verified emitter-by-emitter), and the H7 composition edges (i)/(ii) are
specified with tests. **No Critical.** But the I1 fold, while fixing the predicate, RE-INTRODUCED a
factually-wrong rationale of the *same class* round-1's I1 flagged: §2.3 / §2.6 assert that the direct
`export-wallet --descriptor 'wsh(multi(…))'` path "derives a non-`None` template via
`template_from_descriptor` → `WshMulti`". **That is FALSE on master** — `template_from_descriptor` is
NEVER called on the direct `run` `--descriptor` path; `template_opt` stays `None`, and the four
field-less emitters refuse `None` with their *own* generic `BadInput`, not the new typed error. The
§2.6 descriptor-path test, as written, asserts the NEW typed refusal naming a faithful format and will
FAIL. This is a fold-introduced Important (I-A). The predicate stays funds-SAFE; the rationale and one
test do not match source.

---

## Critical
**None.**

The make-or-break under-refuse question — *can any unsorted-multi shape reach the
electrum/coldcard/coldcard-multisig/jade emitters and silently become BIP-67 sortedmulti while NOT being
in `{WshMulti, ShWshMulti}`?* — is answered **NO**. See the refusal-set completeness table below. The
`CliTemplate` enum has exactly 10 variants and NO bare `Multi` / `sh(multi)` / general-policy variant; the
only typed unsorted-multi variants are `WshMulti` and `ShWshMulti`, both in the refusal set. Every other
route to these emitters is either a sorted variant (allowed, correct), taproot (refused by the existing
per-emitter guard), a general policy (refused upstream by `descriptor_is_general_policy`), legacy
`sh(multi)`/BIP-45 (refused by `template_from_descriptor`'s `ShInner::Ms` arm), or `template == None`
(refused by each emitter's own `template.ok_or_else`). No silent-coercion path escapes the set.

---

## Important

### I-A (fold-introduced, H10) — §2.3/§2.6 rationale for the direct `--descriptor` path is factually wrong; the §2.6 descriptor-path test will FAIL as written
**Where:** spec §2.3 (the round-1 "correcting the prior rationale" paragraph, lines ~188–200) and §2.6
("Descriptor-path coverage" bullet) + §7 H10 summary.

The fold pinned the predicate correctly but, in *justifying* it, asserted:

> "`format_requires_template(Electrum|Coldcard|ColdcardMultisig|Jade) == true` … so BOTH entry paths
> derive a non-`None` template — the direct `run` path via `template_opt`, and `run_from_import_json` via
> `template_from_descriptor`."

and §2.6:

> "the `--descriptor` path (template derived by `template_from_descriptor` → `WshMulti`, non-`None`) also
> refused."

**Both are false for the direct `run` `--descriptor` path. Verified on master:**

1. `run` (`export_wallet.rs:340`) has TWO descriptor-bearing sub-paths, and they behave differently:
   - `--template wsh-multi` → `resolved_template = Some((slots, WshMulti, k))` (`:545`) → `template_opt =
     Some(WshMulti)` (`:553-560`). Caught by the structured guard. ✓
   - `--descriptor 'wsh(multi(…))'` (concrete inline keys; `@N` forms rejected at `:443`) → the
     `else { … }` does NOT set `resolved_template`, so it stays `None` → `template_opt = None` (`:558-560`).
     **`template_from_descriptor` is NOT called anywhere in `run`** (grep-confirmed: its only callers are
     `run_from_import_json:812` and `restore.rs`). `format_requires_template` is consulted ONLY in
     `run_from_import_json:791`, never in `run`'s `--descriptor` branch.
2. So on the direct `--descriptor` path the resolved template is `None`, and the new structured guard
   (matching `Some(WshMulti)|Some(ShWshMulti)`) does NOT fire. What actually refuses the request is each
   emitter's own `inputs.template.ok_or_else(…)` at the top of `emit`:
   - electrum `electrum.rs:53-57` → `BadInput("--format electrum requires --template; descriptor
     passthrough is not supported by Electrum's wallet-db schema")`
   - coldcard `coldcard.rs:108` (via `emit_coldcard_generic_json`, the `None`→`_ =>` arm) → `BadInput("…
     requires --template …")`
   - jade `jade.rs:35-39` → `BadInput("--format jade requires --template …")`
   - coldcard-multisig `emit_payload:120-127` `_ =>` arm → `BadInput("--format coldcard-multisig requires a
     multisig --template …")`

   This is funds-SAFE (an unsorted `multi` on the direct `--descriptor` path is REFUSED, never coerced) —
   but it is refused by a DIFFERENT, untyped `BadInput` that does NOT name a faithful alternative format the
   way the new `ExportWalletUnsortedMultisigUnsupported` does.

3. **Consequence:** §2.6's "Descriptor-path coverage" test —
   `export-wallet --format electrum --descriptor 'wsh(multi(2,…))' → same refusal … structured guard
   catches it without re-parsing the descriptor string` — is wrong about the mechanism AND about the
   surfaced error. It will get exit≠0 (so a naive `exit != 0` assertion passes), but NOT the new typed
   refusal: the spec elsewhere (§2.6 behavioral bullet, §2.4) says the refusal "stderr contains the typed
   refusal naming a faithful format" with `kind() == ExportWalletUnsortedMultisigUnsupported`. If the
   descriptor-path test asserts that typed kind/message (consistent with the rest of §2.6), it FAILS.

This is the same *class* of defect round-1's I1 called out ("a factually-wrong supporting rationale … the
spec must not inherit a false premise") — the fold corrected the `import-json` path's mechanism but
substituted an equally-wrong story for the *direct `--descriptor`* path, conflating `run`'s two sub-paths.

**Fold (round-2 → round-3):**
- (a) **Correct §2.3 and §2.6:** state that the direct `run` `--descriptor` path resolves `template_opt =
  None` (NOT a derived `WshMulti`); `template_from_descriptor` is consulted ONLY on `run_from_import_json`.
  An unsorted `multi` on the direct `--descriptor` path is already refused — but by the emitters'
  `template.ok_or_else` generic `BadInput`, not the new typed error.
- (b) **Decide and pin the direct-`--descriptor` behavior:** EITHER (i) accept that the direct
  `--descriptor` path keeps its existing generic "requires `--template` / passthrough unsupported"
  refusal (funds-safe; the new typed error then covers only the `Some(WshMulti)|Some(ShWshMulti)`
  template/import-json paths) and FIX the §2.6 descriptor-path test to assert that existing generic refusal
  (not the new typed kind); OR (ii) if a typed-error-on-direct-`--descriptor` is wanted, the guard must also
  classify the `template == None` direct path via the descriptor string (`script_type` /
  `template_from_descriptor`) — a strictly larger change the spec does not currently scope. Recommend (i):
  the funds-safety hole (silent coercion) does NOT exist on this path, so a generic refusal is sufficient;
  only the test + rationale need correcting.
- (c) Re-grep the §0 / §2.3 / §2.6 / §7 wording for any other "non-`None` on every path" claim and scope it
  to "the `--template` path and the `--from-import-json` path" explicitly.

(Defer-OK: the exact byte text of the existing generic refusal is not load-bearing for the spec; only the
*which-error* must be pinned so the test is correct.)

---

## Minor

### M-1 (H10) — `emit_payload` is the shared dispatch for THREE subcommand callers, not two; the restore path is silently in scope
The spec places the guard "in the shared `emit_for_format` dispatch" — on master this is `emit_payload`
(`export_wallet.rs:73`), whose own doc-comment (`:60-71`) says it dedups "formerly-4 byte-identical
copies": `run`, `run_from_import_json`, **and restore's `build_import_payload` (`restore.rs:2150/2190`) +
`build_multisig_import_payload` (`restore.rs:2394/2479`)**. `build_multisig_import_payload` passes
`template: Some(t)` which CAN be `Some(WshMulti)` (`restore.rs:2454`, `template` field). So a guard inside
`emit_payload` will ALSO fire on `restore --md1 --format electrum/coldcard/jade` for an unsorted-multi md1
— a desirable funds-safety extension, but the spec scopes H10 to `export-wallet` and tests only
`export-wallet`. Net effect is funds-SAFE (more refusals, never fewer), but the spec should (a) acknowledge
the restore path is covered as a consequence of the chokepoint placement, and (b) add a one-line restore
regression note (does `restore` reconstruct an UNSORTED `WshMulti` to a field-less format today, and is
refusing it the intended behavior?). Not a blocker — placement in the shared dispatch is the right call and
the broader coverage is correct — but the behavior change on a second subcommand should be stated, not
incidental. (Naming nit: the spec calls the fn `emit_for_format`; on master it is `emit_payload` — align
the name so the plan-doc cites the real symbol.)

### M-2 (H10) — minor citation drift (non-material)
- Spec cites the coldcard taproot guard at `coldcard.rs:266-276`; on master the `if matches!` opens at
  `:269` (the `emit_coldcard_multisig_text` fn starts `:258`, the `template.ok_or_else` None-guard is
  `:261-263`, the taproot block runs ~`:265-277`). Close enough; the guard exists and refuses
  `TrMultiA|TrSortedMultiA` after the None-guard, so the disjointness argument holds.
- Spec cites the shared dispatch's `match format` at `export_wallet.rs:119`; on master the
  `collect_missing` match is `:84-106` and the `emit` match is `:113-138` (the fn opens `:73`). The
  reasoning is unaffected (single chokepoint), but the plan-doc should use the live line for the `emit`
  match.

### M-3 (H7) — I2(ii) xpub-slot fp cross-check: confirmed the existing `:1614-1620` guard does NOT cover it; the spec's "either confirm or add" is correctly conservative
Verified on master: the per-`@N` fp cross-check at `bundle.rs:1614-1620` fires ONLY on the
phrase/entropy arms (where a `master_fp` is derived). The **xpub-slot arm** (`bundle.rs:1648-1655`)
computes `fp` as `--slot @N.fingerprint=` **`.or(anno_fp)`** with NO equality check — so a prefix-anno fp
that DISAGREES with an explicit `--slot @N.fingerprint=` is **silently resolved to the `--slot` value
today** (the `.or()` only falls through when the slot fp is absent). The spec's §3.3(ii) correctly does
NOT claim the existing guard covers this; it mandates EITHER confirming coverage OR adding the explicit
comparison, with a test (§3.4). Since coverage does NOT exist for the xpub-slot case, the plan-doc MUST
take the "add the explicit `prefix-anno vs --slot @N.fingerprint=` comparison" branch (the "confirm
existing covers it" branch is unavailable for xpub slots). Flag so the plan-doc picks the right branch;
the spec text itself is adequate (it offers both and pins the test).

---

## CliTemplate refusal-set completeness — enumeration

Refusal set (pinned): `{WshMulti, ShWshMulti}` for formats `{Electrum, Coldcard, ColdcardMultisig, Jade}`.
All 10 `CliTemplate` variants (`template.rs:16-42`, complete enum — no bare `Multi`, no `sh(multi)`/BIP-45
variant, no general/thresh variant):

| CliTemplate variant | unsorted-multi? | reachable by these 4 emitters as a coercible multi? | in refusal set? | safe? |
|---|---|---|---|---|
| `Bip44` | no (singlesig) | n/a (singlesig generic JSON / per-format singlesig refusal) | no | ✓ |
| `Bip49` | no (singlesig) | n/a | no | ✓ |
| `Bip84` | no (singlesig) | n/a | no | ✓ |
| `Bip86` | no (singlesig tr) | refused (coldcard `bip86` §5.1 / jade singlesig / electrum) | no | ✓ |
| **`WshMulti`** | **YES** | **YES → would sortedmulti-coerce** | **YES** | ✓ refused |
| `WshSortedMulti` | no (already sorted) | YES — emits correctly (BIP-67 is what they implement) | no | ✓ allowed |
| **`ShWshMulti`** | **YES** | **YES → would sortedmulti-coerce** | **YES** | ✓ refused |
| `ShWshSortedMulti` | no (already sorted) | YES — emits correctly | no | ✓ allowed |
| `TrMultiA` | unsorted, but taproot | refused by per-emitter taproot guard (coldcard `:~269`, jade `:48-52`, electrum `:58-67`) | no (disjoint variant set) | ✓ refused elsewhere |
| `TrSortedMultiA` | sorted, taproot | refused by per-emitter taproot guard | no | ✓ refused elsewhere |

**Non-template / non-`CliTemplate` routes to these emitters (the adversarial "escape" set):**

| route | resolves to | outcome | safe? |
|---|---|---|---|
| direct `run --descriptor 'wsh(multi(…))'` | `template_opt = None` (NO `template_from_descriptor` call in `run`) | refused by each emitter's `template.ok_or_else` generic `BadInput` (NOT the new typed error) | ✓ refused (but see **I-A**: rationale/test wrong) |
| `run_from_import_json` general policy (`wsh(<ms>)`, timelock/andor) | refused at `export_wallet.rs:798` by `descriptor_is_general_policy` BEFORE `template_from_descriptor` | `BadInput("cannot represent a general wallet policy…")` | ✓ refused |
| `run_from_import_json` legacy `sh(multi)`/`sh(sortedmulti)` (BIP-45) | `template_from_descriptor` `ShInner::Ms` arm `mod.rs:281` | `BadInput("legacy bare P2SH multisig … has no export-wallet template…")` | ✓ refused |
| `run_from_import_json` taproot | refused at `export_wallet.rs:733` (script_type `P2tr/P2trMulti`) BEFORE template derivation | `BadInput` taproot-not-supported | ✓ refused |
| `run_from_import_json` unsorted `wsh(multi)` / `sh(wsh(multi))` | `template_from_descriptor` → `Some(WshMulti)`/`Some(ShWshMulti)` (`mod.rs:259-290`, preserves `is_sorted`) | caught by the new structured guard | ✓ refused (new typed error) |
| `restore` `build_multisig_import_payload` unsorted `Some(WshMulti)` | `emit_payload` chokepoint | caught by the new guard (bonus coverage) — see **M-1** | ✓ refused |

**Conclusion: the refusal set `{WshMulti, ShWshMulti}` is COMPLETE for under-refuse.** There is no
unsorted-multi shape that reaches the four field-less emitters and silently becomes BIP-67 sortedmulti while
sitting outside the set. Every coercion path either (a) resolves to `Some(WshMulti)`/`Some(ShWshMulti)`
(caught), or (b) is refused upstream/by the emitter before reaching the sortedmulti-emitting code. The
guard is also NOT over-refusing: sorted variants, single-sig, taproot, and faithful-format passthrough all
remain unaffected (the mandated §2.6 regression test pins this).

The ONLY defect is descriptive, not structural: **I-A** — the spec mis-states *which* mechanism refuses the
direct `--descriptor` path (it claims a derived `WshMulti`; the truth is `template == None` →
emitter-level generic refusal), so the §2.6 descriptor-path test will not behave as the spec says.

---

## I1 / I2 closure confirmations

### I1 (H10 predicate) — **PARTIALLY CLOSED → re-opens as I-A**
- ✓ **Predicate pinned, structured, substring-immune.** §2.3 pins "refuse iff resolved `CliTemplate` ∈
  `{WshMulti, ShWshMulti}`" — a typed-enum check, immune to the `sortedmulti(`-substring false-match. The
  enum has no other unsorted-multi variant (verified), so the set is complete (table above).
- ✓ **False-refuse regression test mandated** (§2.6: `wsh-sortedmulti` / `sh-wsh-sortedmulti` /
  `tr-*multi-a` still exit 0; taproot hits the existing taproot error, asserted by `kind()`).
- ✗ **Rationale re-introduced a false premise** for the direct `--descriptor` path (claims a derived
  non-`None` `WshMulti`; actually `None` → emitter-level refusal). The §2.6 descriptor-path test will FAIL
  as written. → **I-A** (must fold). The funds-safety predicate itself is sound; only the rationale + that
  one test are wrong.

### I2 (H7 composition edges) — **CLOSED**
- ✓ **(a) prefix bracket missing a valid 8-hex fp ⇒ rejected as malformed.** §3.3(i) + §3.4 pin caps-1 =
  `([0-9a-fA-F]{8})` (NOT optional), byte-mirroring the suffix grammar at `parse_descriptor.rs:84` (inner
  `[0-9a-fA-F]{8}` required whenever the bracket is present). A path-only `[/84'/0'/0']@0` does not match the
  alternation → malformed-descriptor error, same outcome as suffix `@0[/84'/0'/0']`. BIP-380 key-origin
  requires the fingerprint — consistent. Test mandated (§3.4 "Composition edge (i)"). ✓
- ✓ **(b) prefix-fp vs `--slot @N.fingerprint=` mismatch ⇒ refuse.** §3.3(ii) mandates equality-or-refuse,
  citing the suffix per-`@N` cross-check (`bundle.rs:1614-1620`) and the Row-19 path-conflict precedent
  (`bundle.rs:1516-1525`, `SlotInputViolation "path-mismatch"` — both verified). The spec correctly leaves
  "confirm existing covers it OR add the explicit comparison" to the plan-doc with a mandated test (§3.4
  "Composition edge (ii)"). NOTE (M-3): the existing `:1614-1620` guard does NOT cover the xpub-slot case
  (`:1648-1655` uses `.or(anno_fp)` with no equality check), so the plan-doc must take the "add explicit
  comparison" branch — the spec's framing permits that. ✓
- ✓ **(c) both-positions-present ⇒ refuse.** §3.3(a) retains the "prefix AND suffix bracket both present on
  the same `@N` ⇒ refuse (typed `DescriptorParse`)" guard. Consistent with the suffix-form's funds-safety
  posture (never silently pick one origin). ✓

All three edges are specified WITH tests and are consistent with suffix-form behavior on source. I2 closed.

---

## Fold-drift + final cross-checks

- **H7 ACCEPT zero-regression:** ✓ confirmed. `lex_placeholders` regex (`:82-84`) suffix-only; ALL existing
  tests + producers use the suffix form; an additive prefix ALTERNATION regresses nothing. `--help` at
  `bundle.rs:2300` advertises `[fp/path]@N` (prefix). Orthogonal to H13: origin block uses `[...]`, the H13
  multipath validator (`:122-152`) uses `<...>` — disjoint bracket classes, cannot overlap.
- **H13 composition non-regression test mandated:** ✓ §3.4 mandates BOTH `@0/<0';1'>/*` (existing `:1507`
  test) AND a NEW prefix-annotated `[deadbeef/…]@0/<0';1'>/*` still erroring with the hardened-multipath
  message. The H13 validator at `:122-152` is byte-unchanged. ✓
- **H8 sole-site threading:** ✓ the SOLE hardcoded-English fallback in the template path is
  `synthesize.rs:1265`; the keyed twin `:547` uses `run_language`; `synthesize_unified:709` uses the seed
  `mnemonic_lang`. The emit loop `:1262-1275` is a SINGLE loop serving single-sig AND multisig template
  forms — one change covers both. Private `fn` (`:1158`), sole non-test caller `:487`, 3 test call sites
  `:2407/:2443/:2594`. ✓ No external/CLI surface.
- **SemVer toolkit-MINOR + NO GUI/manual leg:** ✓ H8 = private-fn sig; H10 = pure refusal (no clap flag;
  `--allow-sortedmulti-coercion` absent on master; the new `ToolkitError` variant is not in the
  `gui-schema` flag-name set); H7 = lexer-internal. No flag added/removed → no `schema_mirror` leg, no
  manual flag-table leg. README×2 + `fuzz/Cargo.lock` self-pin ritual pinned as an explicit §5 checklist
  (M6 fold). ✓
- **Workstream file-disjointness:** ✓ H8 = `synthesize.rs`; H10 = `export_wallet.rs` + `error.rs` +
  `wallet_export/mod.rs`; H7 = `parse_descriptor.rs` + `bundle.rs` + `verify_bundle.rs`. No overlap. (The
  M-1 restore observation does NOT break disjointness — the guard lives in `export_wallet.rs::emit_payload`;
  `restore.rs` is not edited, it merely calls the shared fn.)
- **Alphabetical `ExportWalletUnsortedMultisigUnsupported`:** ✓ sorts AFTER
  `ExportWalletTaprootMultisigUnsupported` (T<U, enum `:169`, tuple form) and BEFORE `FutureFormat`
  (`:170`) — inserts at `:169a` in the enum and in all four match arms (`exit_code:545→546`,
  `kind:607→610`, `user_text:749→752`). Struct form `{ format }` is a deliberate divergence from the
  precedent's tuple form, mirroring the `ExportWalletMissingFields { … }` struct-variant style (`:155`). M4
  reword accurate.
- **§9 fold log accuracy:** the 6 minors (M1–M6) are each genuinely addressed in the body (M1 no-`multi_a`
  clause; M2 guard-ordering disjointness; M3 compute-don't-hardcode fps; M4 struct-form reword; M5
  `restore.rs`-not-`synthesize.rs` L9 framing; M6 release checklist). The two-Important fold entries (I1,
  I2) are accurately summarized — EXCEPT the I1 entry inherits the same direct-`--descriptor` overclaim
  flagged in I-A ("resolved `CliTemplate` is non-`None` on BOTH paths"). Fix the fold-log I1 row in lockstep
  with the I-A body correction.
- **Q5 (byte-exact message wording):** ✓ fine to defer to plan-doc/impl, as the prompt allows.

---

## What to fold (round-2 → round-3)
1. **I-A** — correct §2.3 / §2.6 / §7 (and the §9 fold-log I1 row): the direct `run --descriptor` path
   resolves `template_opt = None`, NOT a derived `WshMulti`; `template_from_descriptor` is consulted only on
   `run_from_import_json`. Pin the direct-`--descriptor` behavior (recommend: keep the existing emitter-level
   generic refusal — funds-safe, no silent coercion exists there) and FIX the §2.6 descriptor-path test to
   assert that existing refusal (NOT the new typed `ExportWalletUnsortedMultisigUnsupported` kind), OR
   explicitly scope the larger change to also typed-refuse the `template==None` direct path.
2. **M-1** — align the dispatch fn name (`emit_payload`, not `emit_for_format`); acknowledge the restore
   path (`build_multisig_import_payload`) is covered by the chokepoint placement and add a one-line restore
   regression note.
3. **M-2** — refresh the `:266-276` / `:119` citations to live lines.
4. **M-3** — note the plan-doc must take the "add explicit comparison" branch for the xpub-slot
   prefix-fp-vs-`--slot @N.fingerprint=` cross-check (the existing `:1614-1620` guard covers phrase/entropy
   only, not xpub slots).

Re-dispatch the architect after the fold (CLAUDE.md: reviewer-loop continues after every fold). GREEN only
at 0C/0I — currently **1 Important (I-A)** open.
