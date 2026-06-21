# R0 architect review — cycle-2 funds-loss-fixes brainstorm-spec (round 1)

**Reviewed:** `design/BRAINSTORM_cycle2_funds_loss_fixes.md` (H8 / H10 / H7).
**Source of truth:** toolkit `origin/master` = `f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5` (0.61.0), via
`git show origin/master:<path>` — NOT the working tree (which is on `feature/bundle-md1-template-multisig`).
**Reviewer stance:** adversarial / refute-by-default. Every code fact below independently re-grepped on master.
**Date:** 2026-06-21.

---

## VERDICT: **NOT-GREEN** — 0 Critical / **2 Important** / 6 Minor

The three fixes are architecturally sound and the central funds-safety design calls (H10 PURE-REFUSAL on the
descriptor token NOT-preceded-by-`sorted`; H7 ACCEPT-both-positions; H8 thread `run_language`) are CORRECT
and source-grounded. **No Critical findings** — in particular the H10 token predicate, as specified, CANNOT
false-refuse `sortedmulti`. But two Important items must be folded before implementation: (I1) the spec leaves
the H10 predicate as an unresolved open-question (Q2) with a factually-wrong supporting rationale, rather than
pinning the one safe canonical form — an implementer could ship the naive `.contains("multi(")` the spec
itself warns against; and (I2) the spec under-specifies the H7 "both prefix AND suffix present" /
prefix-fp-vs-`--slot fingerprint` composition edges that the prompt flags, leaving a silent-wrong-origin hole.

---

## Critical
**None.**

The make-or-break H10 token-predicate is SAFE *as described in §2.3 / Q2* ("match a `multi(` not immediately
preceded by `sorted`"). Empirically verified (string tests):

| canonical descriptor | `.contains("multi(")` naive | `multi(` not-preceded-by-`sorted` (spec's rule) |
|---|---|---|
| `wsh(multi(2,…))` | YES (correct) | **REFUSE** ✓ |
| `wsh(sortedmulti(2,…))` | **YES — FALSE-MATCH** | allow ✓ |
| `sh(wsh(multi(2,…)))` | YES (correct) | **REFUSE** ✓ |
| `sh(wsh(sortedmulti(2,…)))` | **YES — FALSE-MATCH** | allow ✓ |
| `tr(NUMS,{multi_a(2,…)})` | no (`multi_a(` ≠ `multi(`) | allow (also refused upstream) ✓ |
| `tr(NUMS,{sortedmulti_a(2,…)})` | no | allow ✓ |

The spec EXPLICITLY warns against the naive substring (§2.3 parenthetical + Q2) and mandates the
`sorted`-qualified form, so it is not Critical — but see I1 (it must be *pinned*, not left to Q2).

---

## Important

### I1 — H10 predicate left as an open question (Q2) with a factually-wrong rationale; pin the canonical safe form before impl
**Where:** spec §2.3 (rationale #2 + the implementation parenthetical) and Open-Q2.

Two defects, jointly Important because they bear on the one funds-safety-critical line of the whole cycle:

1. **Rationale #2 is factually wrong for the refusal-target formats.** The spec argues the descriptor-token
   predicate (over `CliTemplate`) is necessary because "`EmitInputs.template` is `None` on the
   descriptor-passthrough path … a `CliTemplate`-only guard would MISS it." Verified on master this is FALSE
   for the three refusal-target formats: `format_requires_template(Electrum|Coldcard|ColdcardMultisig|Jade) ==
   true` (`export_wallet.rs:53-59`), so BOTH entry paths derive a non-None template — the direct `run` path
   via `template_opt`, and `run_from_import_json` via `template_from_descriptor` (`:791`, gated on
   `format_requires_template`). `template_from_descriptor` (`mod.rs:259-290`) even **preserves** the
   distinction: `let is_sorted = d.to_string().contains("sortedmulti(")` → `WshMulti` vs `WshSortedMulti`. So
   `template == None` happens ONLY for passthrough formats (descriptor/bitcoin-core/bip388/specter/green),
   which are NOT in the refusal set. A `CliTemplate`-based guard would in fact catch both paths. The
   descriptor-token predicate is still a perfectly good (and house-style) choice, but the spec's *stated
   justification for preferring it over CliTemplate* is wrong and must be corrected so the plan-doc doesn't
   inherit a false premise.

2. **Q2 is left OPEN ("token-scan/tree-walk vs string token … confirm").** Because this is the single
   funds-safety line of the cycle, the spec must RESOLVE it, not defer it. Resolution (grounded in live
   precedent): use a **string token check, qualified by the `sorted` predecessor** — this is the established
   house style. Live precedent on master: `mod.rs:264` `contains("sortedmulti(")`; `mod.rs:237`
   `contains("multi_a(") || contains("sortedmulti_a(")`; `mod.rs:207` doc "a substring check for `multi_a(` /
   `sortedmulti_a(` to discriminate". The canonical safe predicate to pin in the plan-doc:
   *"refuse iff `canonical_descriptor` contains `multi(` that is NOT part of `sortedmulti(` — e.g. a regex
   `(?:^|[^d])multi\(` or `contains("multi(") && !contains("sortedmulti(") && !contains("multi_a(")`, or a
   miniscript-tree walk for `Terminal::Multi` (unsorted) vs `Terminal::MultiA`."* Note the bare
   `contains("multi(") && !contains("sortedmulti(")` is sufficient and safe for the in-scope shapes (a single
   wsh/sh-wsh multisig descriptor cannot simultaneously carry both an unsorted and a sorted multi token), and
   is the simplest correct form; the `multi_a` term is belt-and-suspenders only.

**Fold:** (a) delete/correct rationale #2's `template == None` claim (state instead: "the descriptor token is
chosen for house-style consistency with `template_from_descriptor`'s own `sortedmulti(` check and to be
template-source-agnostic"); (b) close Q2 by pinning ONE canonical predicate form (string `multi(` minus
`sortedmulti(`) and an explicit unit test that `wsh(sortedmulti(…))` / `sh(wsh(sortedmulti(…)))` is NOT
refused (the false-refuse regression guard the prompt demands).

### I2 — H7 composition edges (both-positions, prefix-fp vs `--slot fingerprint`) under-specified
**Where:** spec §3.3 + §3.4. The prompt's H7-ACCEPT check (d) explicitly asks these be handled or rejected.

The spec's §3.3(a) DOES mandate "refuse if BOTH a prefix AND a suffix bracket are present on the same `@N`"
(good — that closes the double-origin ambiguity). But two further composition edges the prompt names are NOT
addressed and each is a silent-wrong-origin / bypassed-guard risk:

1. **Prefix-fp vs `--slot @N.fingerprint=` cross-check.** The bundle code already has Row-19 conflict logic
   for inline-path-vs-`--slot @N.path=` (`bundle.rs:1516-1525`, verified — `new_paths[idx] != user_origin →
   SlotInputViolation`). Once the prefix form populates `fingerprint_anno`, an analogous fp-conflict can
   arise (prefix `[WRONGFP…]@N` vs a `--slot @N.fingerprint=` or the derived master fp). §3.4 tests the
   prefix-fp-vs-derived-master cross-check (good — rides `bundle.rs:1617-1620`), but the spec does not state
   what happens when a prefix fp ALSO conflicts with an explicit `--slot` fingerprint. The plan-doc must
   either confirm the existing Row-19-style guard covers it or mandate a test pinning the behavior. (Likely
   the existing master-fp cross-check at `:1617` already fires; the spec must SAY so with a citation, not
   leave it implicit.)

2. **Prefix path-but-no-fp (`[/84'/0'/0']@N`).** The spec's regex sketch §3.3(a) is
   `\[([0-9a-fA-F]{8})((?:/\d+…)*)\]` — fingerprint is MANDATORY (8 hex required). But the suffix grammar on
   master (`parse_descriptor.rs:84`) makes the fp optional inside the bracket region only via the outer
   `(?:\[…\])?` — the inner `[0-9a-fA-F]{8}` is required when the bracket is present (caps 2 always 8-hex).
   So `[/path]@N` (path-only, no fp) is already rejected for the suffix form and the prefix mirror inherits
   that — but the spec must EXPLICITLY state the prefix form requires the 8-hex fp (matching suffix), so the
   plan-doc doesn't accidentally relax it to an optional-fp prefix (which would diverge prefix from suffix).
   Add a one-line test: prefix `[/84'/0'/0']@0` (no fp) → same error as suffix.

**Fold:** add to §3.3/§3.4 explicit handling (or explicit "already covered by `bundle.rs:1617`, cited")
for: (i) prefix-fp vs `--slot @N.fingerprint`, (ii) prefix path-but-no-fp ⇒ same rejection as suffix; with a
test for each. These are the prompt's H7(d) edges and must not be left to implementer discretion on a
funds-safety annotation.

---

## Minor

### M1 — H10 §2.3 `multi_a` in the predicate is dead-weight (taproot already refused upstream); say so
On `run_from_import_json`, taproot (`P2tr | P2trMulti`) is refused at the EmitInputs gate
(`export_wallet.rs:~733`, the §2.4 refusal) BEFORE `template_from_descriptor`, so an unsorted `multi_a` never
reaches electrum/coldcard/jade via passthrough. On the direct `--template` path, `tr-multi-a`/`tr-sortedmulti-a`
hit the per-emitter taproot guards (coldcard.rs:266-276, jade.rs:48-52 — both verified). So the predicate's
`multi_a(` clause is belt-and-suspenders, never load-bearing. Harmless, but the spec should note this so the
implementer doesn't over-engineer a tree-walk solely to catch `multi_a`.

### M2 — H10 guard-ordering vs taproot (Q6) is correctly identified but not resolved
§2.5 + Q6 flag that the new unsorted-multi guard must not pre-empt the existing taproot refusal. Verified the
taproot guards are template-based and live INSIDE the per-emitter `emit` (coldcard:266, jade:48); the spec's
chosen location is the shared `emit_payload` dispatch (`export_wallet.rs:75`, the `collect_missing`→refuse
chokepoint) which runs BEFORE per-format `emit`. So the new guard would fire FIRST and could shadow the
taproot refusal for a `tr-*` shape — EXCEPT taproot is already refused upstream of `emit_payload` on the
passthrough path, and on the direct path the predicate (`multi(` not `multi_a(`) won't match taproot. Net:
no real collision, but the spec should RESOLVE Q6 by stating the predicate is `multi(`-specific (excludes
`multi_a(`) so taproot shapes pass through to their existing guard. Pin it.

### M3 — H8 test-vector fingerprints (`1b6aef92` Spanish / `73c5da0a` English) are unverified in this review
The spec lifts these all-zero-entropy master-fp vectors from the bughunt report (`:102` / report §H8). I did
NOT re-derive them here (no crypto execution in an R0). The plan-doc's TDD step must treat the RED assertion
as "Spanish master-fp ≠ English master-fp AND equals the value the test itself computes", not hard-code an
unverified hex — or verify the two constants empirically before pinning them. Low risk (the divergence is the
real assertion, not the exact hex), but flag it so a transcription typo can't make the test vacuously pass.

### M4 — `error.rs` alphabetical placement confirmed but the insertion is between unsorted neighbors
Verified: `ExportWalletTaprootMultisigUnsupported` (`error.rs:169`) immediately precedes `FutureFormat`
(`:170`) in the enum and in `exit_code` (`:545`→`:546`), `kind` (`:607`→`:610`), `message`/`user_text`
(`:749`→`:752`). `ExportWalletUnsortedMultisigUnsupported` sorts AFTER `…Taproot…` and BEFORE `FutureFormat`
— so it inserts at `:169a` in all four arms. The CLAUDE.md alphabetical rule is satisfied *relative to the
local `ExportWallet*` cluster* (which is already locally sorted). Exit 2 matches the taproot precedent
(`exit_code` arm returns 2). Spec §2.4 / Q9 is correct. Minor only: the spec says the variant is
`{ format: &'static str }` (struct form) while the precedent `ExportWalletTaprootMultisigUnsupported(&'static
str)` is tuple form — both fine, but the plan-doc should not claim it is "modeled 1:1" while changing the
shape; either note the deliberate struct-field choice or match the tuple form.

### M5 — H8/L9 fold decision (Q4) is correctly resolved as DO-NOT-FOLD; concur
Verified `run_multisig_template_completion` (`restore.rs:1321`) lacks the `has_hardened_use_site`
(`restore.rs:2779`) and `taproot_override_card`/`restorable_taproot_override_card` (`:2786`) guards that
`run_multisig` (`:2720`) applies. L9 is real but (a) fail-safes to NO-MATCH (not wrong-address), (b) lives in
`restore.rs` not `synthesize.rs` — so it is NOT even the same file as H8's `synthesize.rs` zone (the spec's
§1.4 "same zone" framing is slightly off: the GUARDS are in restore.rs; only the use-site *preservation* is
in synthesize.rs). NOT folding is correct; the "same zone" justification should be tightened to "different
file, fail-safe, not funds-loss-equivalent."

### M6 — SemVer/lockstep claims confirmed; one wording nit
Confirmed against the actual proposed changes: H8 = private-fn signature (`synthesize_template_descriptor` is
`fn`, not `pub fn` — verified `synthesize.rs:1158`), H10 = pure refusal (no new clap flag;
`--allow-sortedmulti-coercion` confirmed absent on master), H7 = lexer-internal. So **NO GUI schema-mirror
leg, NO manual flag-table leg** — correct. The new `ToolkitError` variant is NOT a CLI flag and does not
touch `gui-schema` output (kind strings aren't in the flag-name set). Workstream file-disjointness holds:
H8=`synthesize.rs`; H10=`export_wallet.rs`+`error.rs`+`wallet_export/mod.rs`; H7=`parse_descriptor.rs`+
`bundle.rs`+`verify_bundle.rs` — no overlap. Toolkit MINOR. All correct. Nit: §5 says "resolve the exact
number at release" — fine, but the plan-doc should pin the BOTH-READMEs + `fuzz/Cargo.lock` self-pin ritual
as an explicit checklist item (it's not gate-enforced).

---

## Explicit resolutions to the three central R0 questions

### H10 token-predicate SAFETY — can it false-refuse `sortedmulti`?
**NO — the predicate AS SPECIFIED (§2.3/Q2: "`multi(` not immediately preceded by `sorted`") cannot
false-refuse `sortedmulti`/`sortedmulti_a`/`multi_a`.** Empirically verified by string test (table above):
`wsh(sortedmulti(…))` and `sh(wsh(sortedmulti(…)))` contain `multi(` as a SUBSTRING (the Critical trap), but
the spec's `sorted`-qualified rule correctly allows them; `multi_a(`/`sortedmulti_a(` do not contain `multi(`
at all. The simplest safe form `contains("multi(") && !contains("sortedmulti(")` is correct for the in-scope
single-multisig descriptors. It catches BOTH real unsorted cases (`wsh-multi`, `sh-wsh-multi`) on BOTH paths
(direct `--template` and `--descriptor`/`--from-import-json` passthrough — though note passthrough always
derives a non-None template via `template_from_descriptor`, contra the spec's rationale, see I1). Composition
with the existing taproot guard is clean (M2). **The ONLY risk is implementer drift to the naive substring —
which is why I1 requires the spec to PIN the canonical form rather than leave Q2 open.** With I1 folded, SAFE.

### H7 ACCEPT — zero-regression + composition
**ACCEPT is correct and zero-regression.** Verified: (a) the toolkit's own help advertises the prefix form —
`bundle.rs:~2300` `"Override per-placeholder with [fp/path]@N or --slot @N.path=m/…"`; ALL `lex_placeholders`
tests use the SUFFIX form `@N[fp/path]` (e.g. `parse_descriptor.rs:1378`
`@0[deadbeef/48'/0'/0'/2']` inside a `sortedmulti(…)`), so an additive prefix ALTERNATION regresses nothing.
(b) Orthogonality with cycle-1 H13 CONFIRMED: the H13 reject lives in the group-4 `<…>` multipath validator
(`parse_descriptor.rs:122-152`), the origin block is the `[…]` groups 2/3 — different bracket classes, cannot
overlap; the prefix alternation touches only origin-capture. The §3.4 H13-non-regression test (suffix AND
prefix-annotated `<0';1'>`) is mandated — good (riding the existing `:1507` reject test). (c) Populating the
anno + firing the per-`@N` fp cross-check verified: `bundle.rs:1581-1582` reads
`fingerprint_annos[idx]`, `:1617-1620` errors on `anno != master_fp` — once the lexer populates the anno for
the prefix form this guard fires identically. (d) Edge cases: §3.3(a) handles BOTH-present (refuse); but
prefix-fp-vs-`--slot fingerprint` and prefix-path-no-fp are under-specified → **I2** (must fold). Net: ACCEPT
is the right call, conditional on I2.

### H8 completeness — all template-emit ms1 paths covered + wire round-trip
**COMPLETE.** Verified the SOLE hardcoded-English fallback in the template path is
`synthesize.rs:1265` (`c.language.unwrap_or(bip39::Language::English)`). The other non-test `Language::English`
sites are: `:548` (keyed-path comparison, correct, uses `run_language` at `:547`), `:710`
(`synthesize_unified` uses the actual seed `mnemonic_lang` at `:709`, correct), and `:1266` (template-path
comparison, correct once `:1265` is fixed). Critically, the template-path ms1 emit loop (`:1262-1275`) is a
SINGLE loop serving BOTH single-sig and multisig template forms (the `MkField::Single`/`Multi` split at
`:1234`/`:1256` is the mk1 back-half only) — so threading `run_language` to `:1265` covers single-sig AND
multisig template emit in one change. The 3 in-module test call sites needing the new arg are exactly `:2407,
:2443, :2594` (grep-confirmed; plus the `:487` production call site). Wire-language round-trips via
`bip39_to_wire_code(emit_lang)` (Spanish=3, `language.rs:60`) into `Payload::Mnem { language }`, which
`ms decode` reports — so a non-English template ms1 decodes to its true wordlist. H8's fix design + test plan
are complete and correct.

---

## What to fold (round-1 → round-2)
1. **I1** — correct §2.3 rationale #2 (`template==None` claim is false for the 3 formats) and CLOSE Q2 by
   pinning the canonical `multi(`-minus-`sortedmulti(` predicate + a "sortedmulti NOT refused" regression test.
2. **I2** — specify H7 prefix-fp-vs-`--slot fingerprint` (cite the `bundle.rs:1617` guard or add a test) and
   prefix-path-no-fp ⇒ same rejection as suffix (with tests).
3. Minors M1–M6 as noted (resolve Q6 predicate-is-`multi(`-specific; tighten L9 "same zone" wording; flag the
   H8 test-vector constants as compute-don't-hardcode; note the struct-vs-tuple variant shape; pin the
   README/fuzz-lock release ritual).

Re-dispatch the architect after the fold (CLAUDE.md: reviewer-loop continues after every fold).
