# R0 architect review — cycle-2 funds-loss-fixes IMPLEMENTATION PLAN-DOC (round 2)

**Reviewed:** `design/IMPLEMENTATION_PLAN_cycle2_funds_loss_fixes.md` (H8 / H10 / H7), after the round-1 C1 fold.
**Upstream:** spec `design/BRAINSTORM_cycle2_funds_loss_fixes.md` R0-GREEN (round-3).
**Round-1:** `design/agent-reports/cycle2-funds-loss-fixes-plan-R0-round1.md` — NOT-GREEN, **1 Critical (C1)** /
0 Important / 4 Minor. C1: the H7 prefix-origin group prepended BEFORE `@(\d+)` with plain capturing groups
renumbers the cycle-1 H13 hardened-multipath validator's capture group (4→6) → the hardened-multipath reject
silently stops firing (funds-safety). Resolution mandated: convert `lex_placeholders` to all-NAMED groups
accessed by name.
**Source of truth:** toolkit `origin/master` = `f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5` (0.61.0), via
`git show origin/master:<path>` — NOT the working tree (paused `feature/own-account-subset-search`).
**Method:** every load-bearing fact re-traced on master; the named-group regex shape the plan now PINS was
**executed against `regex = "1"` (cached 1.12.x)** to ground-truth position-independence and the strip-regex
capture count — not inspected.
**Reviewer stance:** adversarial / refute-by-default.
**Date:** 2026-06-21.

---

## VERDICT: **GREEN — 0 Critical / 0 Important** (0 Minor)

C1 is **CLOSED**. The plan now PINS the all-named-group `lex_placeholders` regex (§3.3) and **mandates
rewriting EVERY one of the five capture consumers to `caps.name(...)`** — including the H13 validator's
accessor `.get(4) → .name("mpath")` (§3.1/§3.3). I enumerated every capture-group consumer in the real
`parse_descriptor.rs` on master and confirmed the plan converts ALL of them; **no numeric `caps[N]`/`.get(N)`
consumer is left orphaned** against the renumbered regex. The named-group regex was executed and confirmed
position-independent: `c.name("mpath")` returns the hardened body `"0';1'"` for BOTH the bare and the
prefix-annotated form, so the H13 reject still fires. The strip-regex non-capturing prefix prepend was
executed and confirmed to keep `caps[1]` = index (capture count unchanged). No new Critical or Important
surfaced. **This clears implementation.**

---

## Critical
**None.** (C1 closed — see the consumer-conversion completeness table and the execution evidence below.)

## Important
**None.**

## Minor
**None.** (The 4 round-1 Minors were citation-hygiene items, all discharged with correct live lines — verified
below. No new Minor surfaced.)

---

## C1 closure — `lex_placeholders` consumer-conversion COMPLETENESS

### Authoritative consumer enumeration (real source, `git show origin/master:…parse_descriptor.rs`)

`grep -nE 'caps\b'` over the WHOLE file returns exactly these capture-token sites. There are **two** regexes
with capture consumers in the file, and **no other** capture access anywhere:

**(A) `lex_placeholders` regex (`:82-84`), consumed in the `for caps in re.captures_iter` loop (`:88`):**

| consumer line(s) | OLD numeric group | role | PLAN's named replacement (§3.3) | converted? |
|---|---|---|---|---|
| `:89`, `:90` | `caps[1]` | `@N` index | `caps.name("idx")` | ✅ yes |
| `:92-93` | `caps.get(2)` | suffix fingerprint | `caps.name("sfx_fp")` | ✅ yes |
| `:103-104` | `caps.get(3)` | suffix origin path | `caps.name("sfx_path")` | ✅ yes |
| `:119-120` (validator body `:120-152`) | `caps.get(4)` | **H13 multipath body** | `caps.name("mpath")` (body byte-identical; accessor only) | ✅ yes |
| `:152-153` | `caps.get(5)` | wildcard-hardened | `caps.name("wild")` | ✅ yes |
| — (NEW) | — | prefix fingerprint | `caps.name("pfx_fp")` | ✅ new |
| — (NEW) | — | prefix origin path | `caps.name("pfx_path")` | ✅ new |

All **5** existing numeric consumers are converted to named access by §3.1 (change table, row 2) +
§3.3 (group-name map). **ZERO orphaned numeric `caps[N]`/`.get(N)` remain** against the renumbered regex.
This is the load-bearing completeness check: if even one consumer (especially the `.get(4)` H13 validator)
were left numeric while the regex gained the two prefix groups, it would read the WRONG group — the same
funds-safety class as C1. The plan leaves none.

**(B) `substitute_synthetic` strip regex (`:369-371`), consumed at `:382` and `:393`:**

| consumer line(s) | group | role | PLAN's treatment (§3.1/§3.3) | safe? |
|---|---|---|---|---|
| `:382`, `:393` | `caps[1]` | `@N` index (SOLE capturing group) | **LEFT NUMERIC**; prefix mirrored as **NON-CAPTURING** `(?:…)?` prepend | ✅ yes |

The strip regex's bracket parts are already non-capturing on master; its only capturing group is `@(\d+)`.
The plan correctly leaves it numeric and prepends a **non-capturing** prefix so `caps[1]` cannot shift.
**Executed proof:** current strip `captures_len() == 2` (1 real group + whole-match slot 0); proposed
strip with the non-capturing prefix `captures_len() == 2` (unchanged); `caps[1]` resolves to the index
("0", "0", "7") for suffix-form / prefix-form / bare inputs. No collision.

### Conclusion: **NO orphaned numeric access.** All 5 `lex_placeholders` consumers → named; the 1 strip
consumer stays numeric and is collision-proof under a non-capturing prefix. C1's completeness question
(itself Critical-if-orphaned) is satisfied.

---

## C1 closure — regex validity, match-semantics, and H13 non-regression (executed)

**1. Named-group syntax is valid for the `regex` crate AND match-preserving.** The pinned regex
```
(?:\[(?P<pfx_fp>[0-9a-fA-F]{8})(?P<pfx_path>(?:/\d+(?:'|h)?)*)\])?@(?P<idx>\d+)(?:\[(?P<sfx_fp>[0-9a-fA-F]{8})(?P<sfx_path>(?:/\d+(?:'|h)?)*)\])?(?:/<(?P<mpath>[^>]*)>)?(?P<wild>/\*(?:'|h)?)?
```
compiled cleanly (`captures_len() == 8`; named groups `["pfx_fp","pfx_path","idx","sfx_fp","sfx_path","mpath","wild"]`).
`(?P<name>…)` groups are still **capturing** — same matches, just named — so match semantics are unchanged from
the suffix-only grammar. Executed cases:
- **Suffix form** `@0[deadbeef/84'/0'/0']/<0;1>/*`: `idx=0`, `sfx_fp=deadbeef`, `sfx_path=/84'/0'/0'`,
  `mpath=0;1`, `pfx_*=None`. Identical to today.
- **Prefix form** `[deadbeef/84'/0'/0']@0/<0;1>/*`: `idx=0`, `pfx_fp=deadbeef`, `pfx_path=/84'/0'/0'`,
  `mpath=0;1`, `sfx_*=None`. New form parses.
- **Suffix-bracket-not-swallowed:** `@0[deadbeef/1']/<0;1>/*` → `pfx_fp=None`, `sfx_fp=deadbeef`. The OPTIONAL
  prefix alternation cannot greedily eat a bracket that sits after `@N` (no leading bracket to match) — the
  prefix position/optionality does NOT alter what the suffix/multipath/wildcard match.
- **Both-positions** `[deadbeef/0']@0[cafebabe/1']/<0;1>/*` → both `pfx_fp` and `sfx_fp` populate → the §3.3
  fold logic typed-refuses (ambiguous double-origin). Structurally reachable.

**2. Position-independence — the proof that C1 is closed (executed):**
```
input="@0/<0';1'>/*"                         name("mpath")=Some("0';1'")   get(4)=None
input="[deadbeef/84'/0'/0']@0/<0';1'>/*"     name("mpath")=Some("0';1'")   get(4)=None
```
For BOTH bare and prefix-annotated hardened multipath, `name("mpath")` yields the hardened body `"0';1'"`,
so the H13 validator (reading `caps.name("mpath")`) still SEES the hardened body and REJECTS. The same inputs
return `None` for the numeric `.get(4)` in the named regex — which is exactly why leaving any consumer numeric
would have re-opened C1. The named-access design is verified collision-proof and position-independent.

**3. §3.5 test 9 is SPECIFIED, not hand-waved.** §3.5 test 9 mandates (a) bare `@0/<0';1'>/*` (plus the
`wsh(multi(2,@0/<0';1'>/*,@1/<0';1'>/*))` oracle) STILL errors with the hardened-multipath message, AND (b) a
NEW `[deadbeef/84'/0'/0']@0/<0';1'>/*` STILL errors with the SAME message — "the guard proving this Critical
is closed." The validator body the test exercises is byte-identical on master (`:120-152`, the hardened-marker
reject at `:135-141`). Both assertions re-RED if a future edit reverts to numeric groups or mis-numbers a
consumer. This is a real RED→GREEN regression gate for the C1 class, not prose.

---

## Round-1 Minors — discharged with correct live lines (re-verified on master)

| round-1 Minor | plan's fold | live verification |
|---|---|---|
| **m-1** (H7 precedent overstated) | §3.1 ACCEPT ¶ now says the prefix form is RECOGNIZED-as-non-bare-tr (rationale), NOT parsed; cites `:317`/`:329`/`:339` | ✅ `substitute_nums_sentinel` regex `tr\(NUMS\b` at `:317`; `detect_bare_tr` regex `^tr\([a-z][a-z_0-9]*\(` at `:339`; `[fp/path]@N` appears ONLY in `detect_bare_tr`'s doc-comment (`:329` region, "annotated key (`tr([fp/path]@N)`)") as a form NOT matched |
| **m-2** (H8 `synthesize_unified` line) | §1.1 corrected: `synthesize_unified` defined `:994`; `:709` is a different earlier per-slot emit fn | ✅ `fn synthesize_unified` at `:994`; `mnemonic_lang = seed_mnemonic.language()` at `:709` is in the keyed `synthesize_descriptor` per-slot path |
| **m-3** (H10 coldcard-multisig arm range) | §2.2 corrected to `:130-132`, noting the `match inputs.template {` opens `:120` | ✅ `match inputs.template {` opens `:120`; the `_ =>` `BadInput` arm body is `:130-132` |
| **m-4** (H7 cross-check block) | §3.2(ii) pins `:1617-1620` and names `:1618-1620` as the compare-and-error shape; §3.1 table cites `:1618-1620` | ✅ `if let Some(anno) = anno_fp` `:1617`; `if anno != master_fp` `:1618`; `DescriptorParse` error `:1620` — inside the phrase/entropy `master.fingerprint` branch, does NOT reach the xpub arm |

All four discharged accurately. §12 fold log enumerates exactly C1 + m-1..m-4 — matching round-1.

---

## Round-1 GREEN items — hold under the fold (re-confirmed on master)

**H8 (§1, `synthesize.rs`):** sig `:1158-1162`; sole non-test caller `:487`; SOLE hardcoded-English
`unwrap_or(bip39::Language::English)` at `:1265` inside the shared single+multisig template emit loop
`:1262-1275`; keyed twin already `unwrap_or(run_language)` at `:547`; English→`Entr` branch `:1266`; 3 test
callers `:2407/:2443/:2594` all `(&descriptor, &cosigners, false)`. Threading `run_language` to `:1265` covers
single-sig AND multisig in one edit. Private fn ⇒ no CLI surface. The COMPUTE-don't-hardcode fp test (§1.2-2)
is sound (derives the Spanish/English master-fp divergence in-test; `1b6aef92`/`73c5da0a` doc-only). ✓

**H10 (§2, `error.rs` + `export_wallet.rs`):**
- New variant slots AFTER `ExportWalletTaprootMultisigUnsupported` (`:169`, T<U) and BEFORE `FutureFormat`
  (`:170`, E<F). Exhaustive-arm anchors: `exit_code` insert after `:545` (exit **2**) before `:546`; `kind`
  after `:607-609` before `:610`; `message` after `:749` before `:752`. Struct-form precedent
  `ExportWalletMissingFields { .. }` at `:155`/`:605`/`:745`. Exhaustive test tables `exit_code_table_per_variant`
  (`:965`) + `kind_strings_stable` (`:1272`) exist — the implementer adds the new rows there. Alphabetical
  rule satisfied. ✓
- Guard EXPORT-scoped at `emit_payload:73`, between `collect_missing` (`:82`) and the `emit` `match format {`
  (`:109`); reads `inputs.template` (set `:602-605` region) + `format`; STRUCTURED on the typed enum (immune
  to the `sortedmulti(`-substring false-match). ✓
- Format set {Electrum, Coldcard, ColdcardMultisig, Jade} correct — `format_requires_template` (`:53-59`)
  also returns true for Sparrow (faithful, must NOT be refused). `CliTemplate` has exactly **10** variants
  (template.rs), unsorted-multi = only `WshMulti`/`ShWshMulti` → set is complete and not over-refusing. ✓
- `template_from_descriptor` (`:259`, called ONLY at `:812` on the import-json path) preserves the unsorted
  distinction: `is_sorted = …contains("sortedmulti(")` (`:264`); `Wsh → WshSortedMulti/WshMulti` (`:280-282`);
  `Sh(Wsh) → ShWshSortedMulti/ShWshMulti` (`:271-273`); legacy `ShInner::Ms → BadInput` (`:275-276`).
  Import-json taproot refused upstream `:741-744`; general-policy refused `:798`
  (`descriptor_is_general_policy` def `:301`). Direct `--descriptor` resolves `template_opt = None` → generic
  `BadInput` (NOT the new typed kind). Restore-path coverage is a free funds-safe consequence; `restore.rs`
  NOT edited. The sortedmulti-NOT-refused test (§2.6-4) + the `multi_a`/`sortedmulti_a` disjointness test
  (§2.6-5) are real discriminators. ✓

**H7 non-regex (§3, `bundle.rs` + `verify_bundle.rs`):**
- xpub-slot fp `.or(anno_fp)` at `bundle.rs:1654` (inside the xpub arm opening `:1637`) with NO equality
  check; phrase-arm cross-check `:1617-1620` does NOT reach xpub slots → plan CORRECTLY mandates ADDING the
  explicit `prefix-anno-fp vs --slot @N.fingerprint=` equality check at `:1654` (§3.1/§3.2(ii)), modeled on
  `:1618-1620` + Row-19 `:1516-1530`. §3.5-5 pins it (FP_A≠FP_B → exit≠0; equal → exit 0). ✓
- `--help` advertises the prefix form `[fp/path]@N` at `bundle.rs:2300` → ACCEPT honors the documented
  contract. ✓
- verify-bundle shares the lexer at `verify_bundle.rs:1342` → inherits the fix; §3.5-8 pins it. ✓
- Composition edges (i) mandatory 8-hex `pfx_fp` (path-only prefix → NO match → malformed reject, executed:
  `[/84'/0'/0']@0/<0;1>/*` did NOT match the prefix alternation; the bracket is left for `Descriptor::from_str`
  to reject), (ii) both-positions → typed refuse (executed: both groups populate → fold-logic refuse). ✓

**SemVer / lockstep:** toolkit MINOR off 0.61.0 (working-tree `0.60.0` correctly flagged as the paused
cycle's number); NO GUI schema-mirror leg (no clap flag — the new `ToolkitError` variant is NOT in the
flag-name set, the gate is flag-NAME parity); NO manual flag-table leg; no codec tag→pin chain. The 3 new
FOLLOWUP slugs (`template-form-md1-drops-bip39-wordlist-language`,
`export-wallet-unsorted-multi-silent-sortedmulti-coercion`,
`descriptor-prefix-form-origin-annotation-ignored`) are confirmed ABSENT on master → genuinely new.
Workstream file-disjointness (A=`synthesize.rs`; B=`error.rs`+`export_wallet.rs`;
C=`parse_descriptor.rs`+`bundle.rs`+`verify_bundle.rs`) confirmed — no file in two zones. Branch/worktree plan
off `f9467cc5`, working tree untouched. ✓

**§12 fold log accuracy:** the log records C1 (RESOLVED via all-named groups + every-consumer `caps.name()`
rewrite incl. the H13 accessor; strip regex left numeric with non-capturing prefix; §3.5-9 strengthened) and
m-1..m-4 with the corrected live lines — accurate against round-1 and against master. §11-Q3 marked RESOLVED
(not deferred), satisfying the round-1 Critical's procedural complaint. ✓

---

## Verdict restated

**GREEN — 0 Critical / 0 Important / 0 Minor.** C1 is closed: §3.3 PINS the all-named-group `lex_placeholders`
regex (executed: valid, match-preserving, position-independent); §3.1 mandates rewriting ALL FIVE numeric
capture consumers to `caps.name(...)` — including the H13 validator's `.get(4) → .name("mpath")` — with **zero
orphaned numeric accesses** against the renumbered regex; the strip regex is left numeric with a verified
non-capturing prefix; §3.5-9 is a specified RED→GREEN gate proving both bare and prefix-annotated hardened
multipath STILL reject. The 4 round-1 Minors are discharged with correct live lines; all round-1 GREEN items
hold under the fold. **This plan-doc clears implementation** (per-WS single-subagent TDD → mandatory
post-impl whole-diff review, per CLAUDE.md). No re-dispatch required.
