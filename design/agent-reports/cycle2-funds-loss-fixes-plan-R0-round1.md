# R0 architect review — cycle-2 funds-loss-fixes IMPLEMENTATION PLAN-DOC (round 1)

**Reviewed:** `design/IMPLEMENTATION_PLAN_cycle2_funds_loss_fixes.md` (H8 / H10 / H7).
**Upstream:** spec `design/BRAINSTORM_cycle2_funds_loss_fixes.md` is R0-GREEN (round-3,
`…spec-R0-round3.md`). This is the plan-doc's mandatory R0 (CLAUDE.md hard-gate); implementation MUST NOT
begin until 0C/0I.
**Source of truth:** toolkit `origin/master` = `f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5` (0.61.0), via
`git show origin/master:<path>` — NOT the working tree (paused `feature/own-account-subset-search`,
`364d296f`). Every load-bearing code fact re-traced on master; the H7 regex collision was verified by
*executing* the proposed regex shape against the `regex` crate (ground-truth capture numbering, not
inspection).
**Reviewer stance:** adversarial / refute-by-default.
**Date:** 2026-06-21.

---

## VERDICT: **NOT-GREEN — 1 Critical / 0 Important** (4 Minor, non-blocking)

The plan is accurate and well-cited on H8, H10, and the non-regex parts of H7. **One Critical blocks
GREEN: the H7 regex capture-group numbering the plan commits to (§3.1 / §3.3(a) / §11-Q3) is
internally contradictory and, if implemented literally, silently renumbers the cycle-1 H13
hardened-multipath validator's capture group → breaks the H13 reject (funds-safety regression).** The plan
DEFERS the index-collision confirmation to this reviewer (§11-Q3) instead of RESOLVING it — the prompt
requires it resolved. Fix is mechanical (named groups, or a fully-renumbered consumer set) but the plan
must specify it before coding. Re-dispatch after the fold.

---

## Critical

### C1 — H7 prefix-origin alternation renumbers H13's multipath capture group (§3.1, §3.3(a), §11-Q3)

**The plan's regex instructions are mutually contradictory, and the resolution it leans toward is wrong.**

§3.1 (change table, `lex_placeholders` row) says: *"add an OPTIONAL prefix-origin group
`(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?` **BEFORE** `@(\d+)` with NEW capture indices."*

§3.3(a) (recommended shape) says: *"Prefix group caps become **6/7** (after the existing 1–5) … the parse
code reads `prefix_fp = caps.get(6)`, `prefix_path = caps.get(7)` and folds with the existing
`suffix_fp = caps.get(2)` / `caps.get(3)`."*

These cannot both hold. In the `regex` crate, **capture-group numbers are assigned strictly by
opening-paren order, left to right** (verified by execution, below). A prefix group whose `(` opens BEFORE
`@(\d+)` necessarily takes indices **1 and 2**, shifting:

| segment | CURRENT idx | idx AFTER literal "prefix BEFORE @(\d+)" prepend |
|---|---|---|
| prefix fp (NEW) | — | **1** |
| prefix path (NEW) | — | **2** |
| `@(\d+)` index — `caps[1]` | **1** | **3** |
| suffix fp — `caps.get(2)` | **2** | **4** |
| suffix path — `caps.get(3)` | **3** | **5** |
| **H13 multipath body — `caps.get(4)`** | **4** | **6** |
| wildcard — `caps.get(5)` | **5** | **7** |

So the live consumers (`parse_descriptor.rs:89` `caps[1]`, `:93` `.get(2)`, `:104` `.get(3)`, `:120`
`.get(4)`, `:153` `.get(5)`) would each read the WRONG group:
- `caps[1]` (index parse) reads the prefix fingerprint → index parsing breaks.
- **`caps.get(4)` — the H13 hardened-multipath strict validator (`parse_descriptor.rs:120-152`) — reads
  the suffix-path group instead of the multipath body. The validator never sees `<0';1'>` → the H13
  hardened-multipath reject SILENTLY STOPS FIRING.** A hardened-multipath xpub card (permanently
  un-restorable per the H13 commit's own rationale) would once again lex as a bare `/*` and collapse to a
  wrong/un-restorable address. **This is precisely the Critical funds-safety regression the gate exists to
  prevent.**
- `caps.get(5)` (wildcard-hardened) reads the multipath body.

**The "6/7" outcome the plan commits to is unachievable with the regex shape it describes.** For a prefix
group's captures to land at indices 6/7 while keeping 1–5 stable, the prefix-bracket-matching parens would
have to open LAST in paren order — but the prefix bracket text MUST appear before `@` to match
`[fp/path]@N`. Those two requirements are mutually exclusive for plain capturing groups. **Named groups do
NOT rescue the numeric indices either** — numeric assignment is still by paren order even with `(?P<name>…)`
(verified). The plan provides none of the four disambiguations the gate demands (non-capturing `(?:…)` for
the new alternation — impossible here since we must *capture* the prefix fp/path; a NAMED group accessed
BY NAME; appending the new group LAST in paren order; or a full consistent renumber of every consumer).

**§11-Q3 explicitly DEFERS this to the reviewer** ("Confirm the prefix-group capture indices (6/7 after the
existing 1–5) don't collide with the H13 group-4/5 bookkeeping"). The answer is: **they DO collide, and the
6/7 numbering is impossible.** Per the prompt and CLAUDE.md, this regex-shape question must be RESOLVED in
the plan-doc, not deferred — given the funds-safety blast radius it is Critical.

**Verified by execution** (ran the literal proposed shape `(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?@(\d+)…`
against `regex = "1"`):

```
PROPOSED group count: 8
  suffix-form: g1=None g2=None g3="0" g4="deadbeef" g5="/84'/0'/0'" g6="0;1" g7="/*"
  prefix-form: g1="deadbeef" g2="/84'/0'/0'" g3="0" g4=None g5=None g6="0;1" g7="/*"
CURRENT  group count: 6   (g1=index g2=suffixfp g3=suffixpath g4=multipath g5=wildcard)
```

i.e. the multipath body (H13's group) moves from g4 → g6 and the index from g1 → g3. The
`@0/<0';1'>/*` reject would no longer fire because `caps.get(4)` (the validator's input) now points at the
suffix-path group, which is empty for that input.

**Required resolution (pick one; the plan must SPECIFY it, not punt to the implementer):**
1. **RECOMMENDED — convert `lex_placeholders` to all-NAMED groups and access by name** (`pfp/ppath/idx/sfp/spath/mp/wc`). Verified position-independent by execution: the multipath validator reads `caps.name("mp")` regardless of where the prefix bracket sits, so adding the prefix can never shift it. This is the minimal, collision-proof shape and keeps the diff legible. (The H13 validator body at `:120-152` stays byte-identical; only its accessor changes from `.get(4)` to `.name("mp")`.)
2. OR keep numeric groups but **explicitly renumber EVERY consumer** in `lex_placeholders`
   (`caps[3]` index, `.get(4)` suffix-fp, `.get(5)` suffix-path, `.get(6)` multipath, `.get(7)` wildcard,
   `.get(1)`/`.get(2)` prefix-fp/path) — error-prone and exactly the bookkeeping §3.3(b) warns against,
   but acceptable if pinned exhaustively.

In BOTH cases the plan must DELETE the "caps become 6/7" claim (it is false) and re-pin the actual indices /
names, and the §3.5 test-9 (H13 non-regression: `[deadbeef/84'/0'/0']@0/<0';1'>/*` still errors with the
hardened-multipath message) must be confirmed as the gate that catches a regression here.

**Note — the `substitute_synthetic` STRIP regex (`:369-371`) has NO collision** and needs no change beyond
the additive non-capturing prefix alternation: its only consumer is `caps[1]` (index) and its bracket parts
are already `(?:…)` non-capturing. Verified by execution: a non-capturing prefix prepend keeps `caps[1]` =
index and consumes the leading bracket. The plan's §3.1 strip-row instruction ("mirror the prefix
alternation") is correct as long as the prefix group there stays non-capturing — the plan should state that
explicitly so the implementer doesn't accidentally make it capturing.

---

## Important
**None.**

---

## Minor (informational — none block GREEN; for the plan-doc author)

### m-1 (H7, §3.2.4 / spec §3.2.4 precedent claim slightly overstated)
The plan (and spec) says "a sibling detector (`detect_bare_tr`/`substitute_nums_sentinel`) already PARSES
the prefix bracket." On master those functions do NOT parse/extract `[fp/path]@N` — `detect_bare_tr`'s
regex is `^tr\([a-z][a-z_0-9]*\(` and `substitute_nums_sentinel`'s is `tr\(NUMS\b`; the `[fp/path]@N`
mention is only in `detect_bare_tr`'s DOC-COMMENT (`:329`) listing forms it does NOT match. So the precedent
is "the prefix form is recognized as a non-bare-tr key form," not "the bracket is parsed." This is a
supporting rationale for ACCEPT, not load-bearing for the implementation; the spec is R0-GREEN with this
wording. Non-blocking; tighten if convenient.

### m-2 (H8, §1.1 `synthesize_unified` line cite)
§1.1 says "`synthesize_unified` (`:709`) uses the actual seed `mnemonic_lang`." On master
`synthesize_unified` is defined at `:994`; the `mnemonic_lang = seed_mnemonic.language()` computation at
`:709` is in a DIFFERENT function (an earlier per-slot emit path). The substantive claim (the keyed +
unified paths thread language correctly; only the template path `:1265` hardcodes English) is correct; the
`:709`→fn association is imprecise. Non-blocking.

### m-3 (H10, §2.2 coldcard-multisig arm line range)
§2.2 cites the coldcard-multisig generic `_ =>` refusal at `export_wallet.rs:129-132`; on master the `_ =>`
`BadInput` arm body sits at `:130-132` (the `match inputs.template {` opens `:120`). Off-by-one in the range
start; the arm and its behavior are exactly as described. Non-blocking.

### m-4 (H7, §3.4/§3.1 `bundle.rs:1617` vs `:1618`)
The phrase-arm cross-check is `if let Some(anno) = anno_fp {` at `:1617` and `if anno != master_fp {` at
`:1618`, error at `:1620`. The plan variously cites `:1617-1620` / `:1620` / `:1618-1620` — all land on the
correct block; just confirm the implementer models the ADDED xpub-slot check on the `:1618-1620`
`DescriptorParse` shape (the plan does say this). Non-blocking.

---

## H7 regex capture-group section (current-vs-proposed enumeration + collision verdict)

**Live `lex_placeholders` regex (`parse_descriptor.rs:83`):**
```
@(\d+)(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?(?:/<([^>]*)>)?(/\*(?:'|h)?)?
```
**Current capture groups (confirmed against the live consumers at `:89/:93/:104/:120/:153`):**

| idx | sub-pattern | consumer | role |
|---|---|---|---|
| 1 | `(\d+)` | `caps[1]` (`:89`) | placeholder index |
| 2 | `([0-9a-fA-F]{8})` | `caps.get(2)` (`:93`) | suffix fingerprint |
| 3 | `((?:/\d+(?:'|h)?)*)` | `caps.get(3)` (`:104`) | suffix origin path |
| 4 | `([^>]*)` | `caps.get(4)` (`:120`) | **H13 multipath body (strict validator `:120-152`)** |
| 5 | `(/\*(?:'|h)?)` | `caps.get(5)` (`:153`) | wildcard-hardened |

**Plan's proposed shape (§3.1 + §3.3(a)): prefix alternation prepended BEFORE `@(\d+)`, plan asserts
prefix caps = 6/7, existing 1–5 unchanged.**

**Actual post-change indices (executed against `regex = "1"`):** prefix-fp = **1**, prefix-path = **2**,
index = **3**, suffix-fp = **4**, suffix-path = **5**, **multipath = 6**, wildcard = **7**.

**Collision verdict: COLLISION CONFIRMED.** The plan's "6/7, existing 1–5 unchanged" is impossible — every
existing index shifts by +2. The H13 multipath validator's group moves 4→6, so reading `caps.get(4)` would
feed the validator the suffix-path group → **the hardened-multipath reject silently breaks** (Critical,
funds-safety). The plan must adopt all-NAMED groups (RECOMMENDED, verified position-independent) or a full
consistent numeric renumber of all consumers, and delete the false "6/7" claim. **This is C1.**

---

## Items verified CORRECT (no finding)

**H8 (§1, `synthesize.rs`):** sig `:467` + `run_language:471`; call dropping it `:487`; keyed
`unwrap_or(run_language):547`; template fn `:1158`; SOLE hardcoded-English site `:1265`; shared single+
multisig emit loop `:1262-1275`; English→`Entr` branch `:1266`; 3 test sites `:2407/:2443/:2594` all
`(&descriptor, &cosigners, false)`. Threading `run_language` to `:1265` covers single-sig AND multisig in
one edit. Private fn ⇒ no CLI surface ⇒ no schema-mirror, no manual. **The COMPUTE-don't-hardcode fp test
(§1.2-2) is sound** — it derives the Spanish/English master-fp divergence in-test and asserts
`spanish_fp != english_fp` + template-path-fp == test-computed Spanish fp; the `1b6aef92`/`73c5da0a` pair is
documentation-only, never the assertion RHS, so a transcription typo can't make it vacuously pass.

**H10 (§2, `error.rs` + `export_wallet.rs`):**
- New variant `ExportWalletUnsortedMultisigUnsupported { format: &'static str }` sorts AFTER
  `ExportWalletTaprootMultisigUnsupported` (`:169`, T<U) and BEFORE `FutureFormat` (`:170`, E<F). ✓
  Exhaustive-arm anchors verified: exit_code `:545→:546` (exit **2**, correct, mirrors the taproot
  precedent + every export refusal), kind `:607-609→:610`, message `:749-751→:752`. Struct-form precedent
  `ExportWalletMissingFields { .. }` at `:543/605/745` ✓. Alphabetical-variant rule satisfied.
- Guard placement EXPORT-scoped at `emit_payload:73`, between `collect_missing` (`:82`) and the `emit`
  `match format {` (`:109`); reads `inputs.template` (`EmitInputs.template` set `:605`) + `format`.
  STRUCTURED on the typed enum (immune to the `sortedmulti(`-substring false-match). ✓
- Format set {Electrum, Coldcard, ColdcardMultisig, Jade} is correct — `format_requires_template` (`:53-59`)
  also returns true for **Sparrow** (faithful, carries literal `multi(`) which must NOT be refused; gating
  `Coldcard` in is harmless. ✓ Membership is right.
- The 4 callers (run `:625`, run_from_import_json `:845`, restore's 2 builders) all correctly refuse
  unsorted-multi→field-less; `template_from_descriptor` is called ONLY at `:812` (run_from_import_json),
  NEVER in `run` (grep-confirmed) → direct `--descriptor` resolves `template_opt = None` and is refused by
  each emitter's generic `BadInput` (NOT the new typed kind). Restore coverage is a free funds-safe
  consequence; `restore.rs` is NOT edited (disjointness holds). Single-sig / sortedmulti / taproot / faithful
  formats never match the guard (no over-refusal). ✓
- **§2.6 tests are real RED→GREEN discriminators**, including: the MANDATED `sortedmulti`-NOT-refused
  regression (test 4 — RED's on a naive `.contains("multi(")` drift); the `multi_a`/`sortedmulti_a` proof
  (test 5 — asserts `kind()==ExportWalletTaprootMultisigUnsupported`, disjointness); the direct-`--descriptor`
  ANY-error-but-NOT-the-new-kind test (test 3); the restore-path free-consequence (test 7). Differential
  cites the existing `wsh-multi-2of3-divergent` row (`tests/bitcoind_differential.rs:115`, verified), no new
  export row. ✓

**H7 non-regex parts (§3, `bundle.rs` + `verify_bundle.rs`):**
- xpub-slot fp `.or(anno_fp)` at `bundle.rs:1654` with NO equality check (verified `:1637` xpub arm,
  `:1654` `.or`); the phrase-arm cross-check `:1617-1620` does NOT reach xpub slots (verified) → the plan
  CORRECTLY mandates ADDING the explicit `prefix-anno-fp vs --slot @N.fingerprint=` equality check at
  `:1654`, not "confirm existing covers it." ✓ The §3.5-5 test (FP_A≠FP_B → exit≠0; equal → exit 0) pins it.
- `--help` advertises the prefix form `[fp/path]@N` at `bundle.rs:2300` (verified) — ACCEPT honors the
  documented contract. ✓
- verify-bundle shares the lexer at `verify_bundle.rs:1342`/`:1346` (verified) → inherits the fix; §3.5-8
  pins it. ✓
- Composition edges (i) mandatory 8-hex fp inside the prefix bracket (caps non-optional), (ii) both-positions
  → refuse, (iii) the H13 non-regression test (§3.5-9: prefix-annotated `[deadbeef/…]@0/<0';1'>/*` STILL
  errors with the hardened-multipath message) — all correct AS DESIGNED, and §3.5-9 is exactly the test that
  would catch a C1 regression. **The C1 fix is a prerequisite for §3.5-9 actually passing.**

**3 round-3 citation-hygiene Minors (§6):** m-i (`:741-744` taproot / `:275-276` legacy `sh(multi)`),
m-ii (`descriptor_is_general_policy` call `:798` / def `wallet_export/mod.rs:301`), m-iii (master-fp pair
documentation-only) — each discharged with live lines; verified accurate against master.

**6 open R0 questions (§11):** Q1 (format set — keep `Coldcard`, harmless) ✓; Q2 (message wording —
implementer latitude, byte-pinned in test) ✓; **Q3 (regex shape — DEFERRED but MUST be RESOLVED → C1)**;
Q4 (manual note — defer, no gate) ✓; Q5 (struct vs tuple variant form — pinned struct, tuple acceptable
if consistent) ✓; Q6 (test harness reaches `kind()` — standard, fine) ✓. Only Q3 is mis-handled (deferred,
must be resolved) → C1.

**SemVer / lockstep:** toolkit MINOR off 0.61.0 ✓ (working-tree `0.60.0` correctly flagged as the paused
cycle's number); NO GUI schema-mirror leg (no clap flag; the new `ToolkitError` variant is not in the
flag-name set) ✓; NO manual flag-table leg ✓; no codec tag→pin chain ✓. Alphabetical-variant rule ✓.
FOLLOWUP slugs (H8/H10/H7 + optional direct-`--descriptor`) are new and correctly slated to be FILED ✓.
Workstream file-disjointness (A=`synthesize.rs`; B=`error.rs`+`export_wallet.rs`; C=`parse_descriptor.rs`+
`bundle.rs`+`verify_bundle.rs`) confirmed — no file in two zones ✓. Branch/worktree plan off `f9467cc5`,
working tree untouched ✓.

---

## Verdict restated

**NOT-GREEN — 1 Critical / 0 Important.** C1: the H7 prefix-origin regex, as specified (§3.1 "prefix group
BEFORE `@(\d+)`" + §3.3(a) "caps become 6/7"), is contradictory and would renumber the H13 multipath
validator's capture group (4→6), silently breaking the cycle-1 hardened-multipath funds-safety reject. The
plan DEFERS the collision question (§11-Q3) to the reviewer; it must be RESOLVED in the plan-doc —
RECOMMENDED: convert `lex_placeholders` to all-NAMED groups accessed by name (verified position-independent
and collision-proof), delete the false "6/7" claim, and re-pin the accessors. Everything else (H8, H10, the
H7 non-regex sites, tests, SemVer, lockstep, disjointness) is correct and GREEN. Fold C1 → re-dispatch the
architect (reviewer-loop continues after every fold).
