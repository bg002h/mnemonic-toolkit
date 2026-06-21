# R0 REVIEW — IMPLEMENTATION_PLAN_cycle1_critical_fixes (H13 / H12 / H1) — Plan-doc Round 1

**Reviewer role:** opus architect, mandatory R0 gate on the IMPLEMENTATION PLAN-DOC (CLAUDE.md hard-gate:
NO implementation before 0C/0I). The brainstorm-spec is already R0-GREEN (round-2); this review gates the
plan-doc.
**Plan-doc under review:** `design/IMPLEMENTATION_PLAN_cycle1_critical_fixes.md`.
**GREEN spec:** `design/BRAINSTORM_cycle1_critical_fixes.md`; spec R0 round-2:
`design/agent-reports/cycle1-critical-fixes-spec-R0-round2.md` (the 4 Minors m-1..m-4 fed this plan).
**Date:** 2026-06-20.
**Canonical sources verified live (NOT the working tree — this checkout is on another instance's WIP):**
- toolkit `origin/master` = `4d5872ed489e706155b0d88b02686977e59a20b6` (confirmed `git rev-parse`).
- descriptor-mnemonic (md-codec + md-cli) `origin/main` = `54dd765a11d490dc3d8dec2c842dae718bd3ef2b`
  (confirmed; NO `origin/master`, default `main`). Current versions: toolkit `0.60.0`, md-codec `0.37.0`,
  md-cli `0.7.1`.

Every load-bearing citation re-grepped against `git show origin/<branch>:<path>`. The verdict rests on
first-hand source reads.

---

## VERDICT: **NOT-GREEN — 1 Critical / 1 Important**

The plan-doc is well-constructed and its release/dependency analysis (Q-PLAN-R0-1) and Q-WIRE / Q-PLAN-R0-3
resolutions are correct against source. **But the load-bearing m-4 / Q-PLAN-R0-2 decision — compare
`.tree` ONLY for the H1 verify-bundle gate — leaves a runtime false-GREEN funds-safety gap**, because
`use_site_path` (the change-chain / multipath that drives address derivation) is funds-relevant, is
excluded from `.tree`, has NO canonicalization ambiguity (so an `==` on it would NOT false-fail), and is
bound by NO other check in the multisig verify path. Separately, an **Important** factual error: the
plan-doc asserts "the toolkit has NO second strip regex" (m-3) — it DOES
(`parse_descriptor.rs::substitute_synthetic`, regex at `:319`, `[0-9;]` class), so the m-3 resolution
rests on a false premise.

---

## CRITICAL FINDINGS

### C-PLAN-1 — `.tree`-only H1 gate GREENs a `use_site_path`-divergent wallet with a DIFFERENT watched-address set (Q-PLAN-R0-2 REFUTED)

**Summary:** The plan compares `expected_md_decoded.tree == desc.tree` and EXCLUDES `use_site_path`
(m-4 / §3 / §6.3). `use_site_path` carries the multipath change-chains (`<0;1>` etc.) and
`wildcard_hardened` — fields that **directly determine which addresses the wallet watches**. A
user-supplied md1 with an identical `.tree` (same Tag, k, wrapper, key `@N` indices, nesting) but a
**different `use_site_path`** (e.g. `<2;3>/*` instead of `<0;1>/*`, or bare `/*` vs `<0;1>/*`) derives a
**different address set**, yet the `.tree`-only gate returns `passed: true` → overall GREEN verdict on a
wrong wallet. For a verification tool this is the worst failure class — a false assurance — and is the
SAME structural-blindness category H1 exists to close.

**Source evidence (first-hand):**
1. **`use_site_path` is funds-relevant — it drives derivation.**
   `md-codec/src/derive.rs::derive_address` (verified): after the hardened pre-flight, it does
   `if let Some(alts) = &self.use_site_path.multipath { if (chain as usize) >= alts.len() {…} }` and then
   `to_miniscript::to_miniscript_descriptor(self, chain)` — the `chain` index selects an
   `Alternative` from `use_site_path.multipath`, and the `Alternative.value` becomes the derivation step.
   **The multipath alternatives ARE the change-chain branches that fix the address set.**
2. **`use_site_path` is excluded from `.tree`.** `md-codec/src/encode.rs:16-28` — `Descriptor { n,
   path_decl, use_site_path, tree, tlv }` (5 fields, all derive `Eq`). `tree: Node` (the operator AST)
   does NOT include `use_site_path`; `tree.rs:8-73` `Node`/`Body` carry Tag/k/indices/nesting/timelock/hash
   nodes but NO change-chain/multipath. So `.tree ==` is silent on the multipath.
3. **`use_site_path` is arbitrary/user-controllable.** The lexer regex group 4
   (`parse_descriptor.rs:319` baseline at `:69`) is `(?:/<([0-9;]+)>)?` → `multipath_alts: Vec<u32>`
   (`parse_descriptor.rs:54`, populated `:106-130`), threaded verbatim into
   `make_use_site_path` → `Alternative { hardened:false, value:*v }` (`:223-235`). A descriptor string
   `wsh(sortedmulti(2,@0/<2;3>/*,@1/<2;3>/*))` is accepted and yields `use_site_path.multipath = [2,3]`.
4. **NO other check in the multisig path binds `use_site_path`.** `verify_bundle.rs` `emit_multisig_checks`
   (the md1 block at `:2702-2760`, the per-cosigner path at `:2702-2860`) emits ONLY `md1_decode`,
   `md1_wallet_policy`, `md1_xpub_match`, and the per-cosigner `mk1_*` checks — verified by enumerating
   every `name:` in the block. There is **NO `use_site_path`/`multipath` comparison and NO address
   derivation** in this path (grep of `verify_bundle.rs` for `use_site_path|multipath|derive_address`
   returns only single-sig/search-path sites at `:684/:923`, NOT the multisig md1 block).
5. **The single-sig path does NOT have this gap — proving the asymmetry is real and the multisig fix is
   incomplete.** `verify_bundle.rs:645` (keyless single-sig) uses `let md1_match = expected.md1 == args.md1`
   — a **full byte-exact string compare** that DOES bind `use_site_path`. The keyless-template arm
   (`:882`) uses `compute_wallet_descriptor_template_id`. ONLY the keyed-multisig path falls back to a
   structural sub-field compare, and the plan's chosen sub-field set omits the very field that fixes the
   address set.

**Why the plan's m-4 rationale does NOT hold (each refuted against source):**
- *"the oracle (the funds-truth backstop) polices receive/change derivation"* — the
  `bitcoind_differential.rs` oracle is a **test harness**, not a runtime `verify-bundle` check. It cannot
  police a user's actual `verify-bundle` invocation. The runtime verdict GREENs the wrong wallet regardless
  of what a CI test asserts.
- *"the H12 fix independently closes the origin/path-divergence wrong-address class"* — H12 fixes the
  **`path_decl` ORIGIN default** (`m/48'/…/2'` vs `/3'`, `bundle.rs::compute_default_origin_path`). That is
  the `path_decl` field, an ORTHOGONAL concern. H12 does nothing to bind the **`use_site_path`** change-chain.
- *"binding `use_site_path` risks false-fails on canonically-equal-but-differently-encoded multipaths"* —
  **unsubstantiated.** `md-codec/src/use_site_path.rs` has NO sort/normalize/canonicalization (grep for
  `sort|canonical|normal` = none); the field is a faithful 1:1, order-significant representation. Contrast
  `path_decl`/origins, which DO have elision/canonicalization (`canonical_origin.rs`,
  `canonicalize_placeholder_indices`) — THAT is what justifies excluding `path_decl`/`tlv`. The m-4
  reasoning correctly excludes origins for elision-brittleness, then **incorrectly generalizes the same
  exclusion to `use_site_path`, which has no such brittleness.** An `==` on `use_site_path` would NOT
  false-fail a legitimately-equal wallet.

**Note on `wildcard_hardened`:** the H13 reject closes hardened input at parse, so a hardened
`use_site_path` cannot reach verify. The residual, unclosed gap is **non-hardened multipath value
divergence** (`<0;1>` vs `<2;3>`, count/presence divergence) — fully reachable, fully funds-relevant.

**Additional surface (scope-completeness):** per-`@N` use-site overrides also live in
`tlv.use_site_path_overrides` (`encode.rs` references it; `md-codec/src/encode.rs` validates
`validate_multipath_consistency(&d.use_site_path, overrides)`). The remedy must consider whether the gate
binds baseline `use_site_path` AND the per-`@N` override map (both affect the derived address set), not
just the baseline.

**Required remedy (the plan-doc must adopt one and re-justify; this is a Critical funds-safety gate, not a
shape preference):**
- **(preferred)** Extend the H1 gate to `expected.tree == desc.tree && expected.use_site_path ==
  desc.use_site_path` (and account for the per-`@N` `tlv.use_site_path_overrides` map, which also fixes
  the address set). `use_site_path` derives `Eq` (`use_site_path.rs:48`) and has no canonicalization
  ambiguity → no false-fail risk. Keep `path_decl`/`tlv`-origins excluded (elision brittleness — that
  exclusion IS correct).
- **(alternative, simpler, stricter)** For the keyed-multisig path, compare the decoded **whole**
  `Descriptor` MINUS the origin-bearing fields — i.e. compare `(tree, use_site_path)` and the
  address-fixing TLV overrides, excluding `path_decl` and the origin/fingerprint TLVs. Equivalent in
  funds-safety to the preferred option; the plan-doc must specify exactly which fields.
- Whichever is chosen: add a P4 RED test that two `.tree`-equal md1s differing ONLY in `use_site_path`
  (`<0;1>/*` engraved vs `<2;3>/*` supplied) → `passed:false`; and a clean-negative that an identical
  `use_site_path` (incl. a legitimately origin-elided-but-equal md1) still → `passed:true`. The plan's
  current §2 P4 test 6 ("origin-elided-but-equal → passed:true") stays valid and is NOT in tension with
  this (origins remain excluded; only `use_site_path` is added).

**This refutes Q-PLAN-R0-2 as posed.** `.tree`-only is NOT the funds-safe minimal gate — it misses a
change-chain/multipath divergence that yields a different watched-address set, which is exactly the
"funds-relevant policy difference outside `.tree`" the reviewer brief asked to hunt for.

---

## IMPORTANT FINDINGS

### I-PLAN-1 — m-3 rests on a FALSE premise: the toolkit DOES have a second multipath-strip regex

**Summary:** The plan-doc asserts, repeatedly and as a "verified" fact, that the toolkit has no second
strip regex and therefore no m-3 work:
- §1 (lines 82-84): *"No second strip regex — the toolkit has only ONE multipath-relevant regex
  (`lex_placeholders:70`)… So m-3 has NO toolkit analog… verified `grep -c 'Regex::new'` = 4, none a
  multipath strip."*
- §3 m-3 (lines 308-313): *"The toolkit has NO second strip regex… so there is no toolkit m-3 work. The
  plan-doc records this asymmetry so the toolkit-side subagent does not invent a phantom mirror edit."*
- §6.1 (line 410): *"No second strip regex (m-3 N/A)."*

**This is factually wrong.** `crates/mnemonic-toolkit/src/parse_descriptor.rs` has FOUR `Regex::new` calls
(`:69`, `:277` `tr(NUMS\b`, `:299` `^tr\(`, **`:319`**). The regex at **`:319`**, inside
`pub fn substitute_synthetic` (`:313`), is
`r"@(\d+)(?:\[[0-9a-fA-F]{8}(?:/\d+(?:'|h)?)*\])?(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?"` — a **non-capturing
multipath strip** with the `[0-9;]` class. This is the **exact structural twin** of md-cli's
`substitute_synthetic` strip regex (`template.rs:365`, same `(?:/<[0-9;]+>)?`) that m-3 widens. The
plan's "`grep -c = 4, none a multipath strip`" is self-contradicting — `:319` IS a multipath strip.

**Why it matters (not merely cosmetic):**
1. A subagent instructed "there is no second regex, do not invent a phantom mirror edit" will be confused
   on encountering one, or will skip a legitimate symmetry edit citing the plan's (wrong) statement.
2. The m-3 *robustness intent* (a hardened body matched-and-stripped so no confusing SECONDARY parse error
   fires before the primary reject) DOES potentially apply to `substitute_synthetic:319`. `substitute_synthetic`
   is `pub` and called directly (tests `:1441`, `:2594`) and is documented as having callers that may
   bypass `resolve_placeholders` (`:359`). On `<0';1'>` input the `[0-9;]` strip would NOT match the `'`,
   leaving a residual that mis-parses.
3. The plan SHOULD instead make the correct argument: in the production `parse_descriptor` pipeline
   (`:748`), `resolve_placeholders` (`:768`, which calls the now-`Result` `make_use_site_path` → the
   primary `DescriptorParse` reject) runs **BEFORE** `substitute_synthetic` (`:779`) — so for the
   production path the primary reject pre-empts the secondary error, and widening `:319` is OPTIONAL
   (robustness-only, for direct `pub` callers). That is a real, defensible resolution — but it is NOT the
   resolution the plan states (which is "the regex does not exist").

**Required fix:** Correct §1 / §3 m-3 / §6.1 to (a) acknowledge `substitute_synthetic:319` exists and is a
`[0-9;]` multipath strip, and (b) decide explicitly: either widen `:319` to `[0-9;'h]` for md-cli symmetry
(robustness for direct `pub` callers), OR justify leaving it via the call-order pre-emption argument
(`resolve_placeholders` reject at `:768` fires before `substitute_synthetic` at `:779`). Both are
acceptable; the false "no such regex" claim is not.

---

## EXPLICIT RESOLUTIONS TO THE PLAN-DOC'S OPEN QUESTIONS + Q-WIRE

### Q-PLAN-R0-1 (release/deps — H13 two lexers INDEPENDENT) — **CONFIRMED ✓**
The plan-doc's CORRECTION of the spec's "publish-before-pin" framing is correct against source:
- `crates/mnemonic-toolkit/Cargo.toml` (verified): `md-codec = "0.37"` (`:36`), `miniscript = "13"`
  (`:44`). **NO `md-cli` dependency anywhere** (`git grep -lE 'md-cli' origin/master -- '*Cargo.toml'`
  returns nothing; workspace `Cargo.toml` `members = ["crates/mnemonic-toolkit"]`).
- md-cli is a **binary crate**: `crates/md-cli/Cargo.toml` has `[[bin]] name="md"`, the only source file is
  `src/main.rs` (no `lib.rs`), and it pins `md-codec = { path=…, version="=0.37.0" }`. The toolkit cannot
  and does not consume md-cli as a library. The two lexers are independent copies of the same defect.
- The toolkit H13 fix needs no md-codec API change: `parse_descriptor.rs:17-22` imports
  `md_codec::use_site_path::{Alternative, UseSitePath}`, `md_codec::tag::Tag`, `md_codec::tree::{Body,
  Node}`, etc. — all present in md-codec 0.37.0. The fix widens a local regex + a local
  `make_use_site_path` to `Result<_, ToolkitError>` reusing the existing `DescriptorParse` variant.
- **No transitive Cargo.lock reason to bump md-codec.** The workspace pins `md-codec = "0.37"` and the
  fix touches no md-codec API; a `cargo check --workspace` (the plan's stated discipline, NOT
  `cargo update -w`) refreshes the lock without pulling a newer md-codec. The toolkit MAY ship cycle-1
  against the unchanged `0.37` pin. **Confirmed: no publish-before-pin, no md-codec pin bump, no
  transitive churn from H13.**
*(Citation nit: the plan cites the imports as `:17-21`; they span `:17-22`. Non-load-bearing.)*

### Q-PLAN-R0-2 (the `.tree`-ONLY H1 scope) — **REFUTED → see C-PLAN-1**
(a) `.tree` DOES contain all in-tree policy structure — Tag, threshold k (`Body::MultiKeys.k`,
`Variable.k`, `Tr`), wrapper/nesting (`Children`/nested `Node`, incl. the `sh(multi)` vs `sh(wsh(multi))`
distinction per `decode.rs`), multi-vs-sortedmulti (distinct `Tag` variants), key `@N` indices
(`MultiKeys.indices` order-significant, `KeyArg.index`, `Tr.key_index`), AND timelock/hashlock/branch nodes
(`Timelock`, `Hash256Body`, `Hash160Body`, `Variable`/Thresh, `Tr.tree`) — all verified in `tree.rs:8-73`.
(b) **YES — a funds-relevant policy difference lives OUTSIDE `.tree`: `use_site_path` (change-chain /
multipath).** It drives `derive_address` (the watched address set), is excluded from `.tree`, has no
canonicalization ambiguity, and is bound by no other multisig-path check. `.tree`-only would GREEN a wrong
(different-address) wallet. **`.tree`-only is NOT complete — this is the Critical gap C-PLAN-1.** The
excluded `path_decl`/`tlv`-origins ARE correctly excluded (elision brittleness); `use_site_path` is NOT in
that category and must be added to the gate.

### Q-PLAN-R0-3 (Q-H12-1 — no taproot→`2'` escape-hatch flag) — **CONFIRMED ✓**
No escape-hatch flag is the right call; always emit `3'` for `Tag::Tr`.
- The value is **tree-deterministic** (`Tag::Tr` → 3 via `template.rs::bip48_script_type()`, verified
  `:231-237`) and matches what template-mode already emits AND what every major coordinator
  (Sparrow/Coldcard/Jade) and Core's `deriveaddresses` derive. There is no ambiguity for a user to resolve.
- A flag would be a footgun (it would let a user re-select the wrong `2'` that produces a non-cosignable
  taproot wallet — the exact H12 defect) AND would drag GUI `schema_mirror` (a new flag-NAME) + the manual
  flag-reference (the mirror invariant), flipping H12 from PATCH-clean to MINOR-with-lockstep. No
  legitimate use-case wants a taproot descriptor's cosigners defaulted to P2WSH `2'`. **Confirmed: no flag.**

### Q-WIRE (H1 changes only `md1_xpub_match.passed`; GUI does not consume `checks[]`) — **CONFIRMED ✓**
- The H1 fix keeps the check NAME `md1_xpub_match` and changes only its `passed` predicate (and `detail`).
  No new `checks[]` array element, no field add, no rename — verified the emit site (`verify_bundle.rs:2740`
  pushes `VerifyCheck { name: "md1_xpub_match", … }`) and the `VerifyCheck` shape (`format.rs:132`,
  free-form `name: String`).
- **GUI non-consumption verified against `mnemonic-gui` (`origin/master`):** a grep over `mnemonic-gui/src`
  for `md1_xpub_match`/`md1_policy_match`/`"checks"`/`checks[` returns **nothing**. The GUI references
  `verify-bundle` only for clap-flag conditional-visibility modeling (`src/form/conditional.rs:381
  verify_bundle()`, `src/runner.rs:61` doc-comment, `src/schema/mod.rs`). The runner is a generic
  subprocess capture; it keys on no `checks[]` entry. So a NAME-preserving `passed`-value change carries
  **no GUI paired-PR obligation** and `schema_mirror` (flag-NAMES only) is not triggered. **Confirmed.**
  *(NB: C-PLAN-1's remedy keeps the same NAME `md1_xpub_match` and only changes its `passed` predicate to
  include the `use_site_path` term — so adopting C-PLAN-1 does NOT introduce any wire-shape change and this
  Q-WIRE confirmation still holds.)*

---

## OTHER ADVERSARIAL CHECKS

- **Exit codes (m-1) — CORRECT.** md-cli: `main.rs:251-258` — `Ok(code)=>from(code)`,
  `Err(BadArg)=>from(2)`, catch-all `Err(e)=>from(1)`; `CliError::TemplateParse` has no numeric arm
  (`error.rs:6/28`) → returned from `main` ⇒ exit **1** (matches `exit_codes.rs::encode_bad_template_returns_1`).
  Toolkit: `error.rs:539` `DescriptorParse(_) => 2` ⇒ exit **2**. Plan's per-repo exact-code assertions
  (md-cli 1 / toolkit 2) are right.
- **m-2 (strict alternation) — sound.** The stricter `((?:\d+(?:'|h)?)(?:;\d+(?:'|h)?)*)` over the looser
  `[0-9;'h]+` is the better choice (primary typed reject on a malformed body); plan-doc-level, no gate.
- **H12 facts — VERIFIED.** `compute_default_origin_path` (`bundle.rs:2210`) hardcodes the 4th
  `PathComponent { hardened:true, value:2 }` (`:2228-2231`), signature has no script-type;
  `verify_bundle.rs:1373` calls the same 2-arg helper; `descriptor_intake.rs:297` parses to
  `MsDescriptor::from_str` (rust-miniscript, no `Tag`), `:324`/`:345` pass literal `2`, `:410`
  `bip48_default_path` already takes `script_type: u32`; `bsms.rs:295` `matches!(parsed,
  MsDescriptor::Tr(_))` is the proven per-site detection precedent. The per-site detection design (Tag
  sites via `bip48_script_type()`, miniscript site via `Descriptor::Tr(_)`) is correct.
- **H1 facts — VERIFIED.** The md1 multiset block (`verify_bundle.rs:2718-2740`) extracts `exp_pubs`/
  `act_pubs` discarding the slot index, sorts, compares; `expected_md_decoded` (`:2698`, from
  `expected.md1`) and `desc` (the supplied `:2702`) are both decoded `md_codec::Descriptor`s in scope.
  The `tree`/`Body`/`Tag` derives (`encode.rs:16`, `tree.rs:8/17`, `tag.rs:14`) all `PartialEq, Eq` —
  `tree ==` is order-significant on `Body::MultiKeys.indices` and distinguishes `sh(multi)` from
  `sh(wsh(multi))` (`decode.rs` Sh-covers-both). Keeping the pubkey-set check subordinate (catches key
  SUBSTITUTION) + `tree ==` (catches Tag/k/wrapper/order/nesting) is correct — but **incomplete without
  the `use_site_path` term** (C-PLAN-1).
- **Version sites — COMPLETE & ACCURATE.** Two version-bearing READMEs (`README.md`,
  `crates/mnemonic-toolkit/README.md`, gated by `tests/readme_version_current.rs`); `fuzz/Cargo.lock`
  present; `scripts/install.sh` self-pins `mnemonic-toolkit-v0.60.0`. Current versions: toolkit `0.60.0`,
  md-cli `0.7.1` (→`0.8.0` MINOR correct). The §8 checklist matches the release ritual.
- **GUI schema_mirror + manual genuinely NOT triggered — CONFIRMED.** No flag/dropdown/subcommand change in
  any of H13/H12/H1; error TEXT is not `schema_mirror`-gated (flag-NAMES only); no manual CLI-surface change.
- **FOLLOWUPS — VERIFIED.** Stale entry `verify-bundle-multisig-md1-xpub-match-set-equality` header at
  `FOLLOWUPS.md:1635`, status `resolved by v0.5.0 Phase B.3` at `:1641` — the entry to re-open. The defer-
  to-shipping-commit handling (no `design/` edits now, contention with the other instance) is correct.
- **Internal consistency with the GREEN spec — clean.** No stale "publish→pin" residue (correctly
  superseded by §4's independent-release verdict); no "faithfully represent" residue (H13 is REJECT
  throughout); the 4 Minors are reflected (m-1 ✓, m-2 ✓, m-3 reflected but on a FALSE premise — I-PLAN-1,
  m-4 reflected but funds-incomplete — C-PLAN-1). Phase ordering / TDD-RED-first / single-subagent-per-
  phase / worktree / class-A differential-row gating all match the spec and CLAUDE.md.

---

## MINOR / NITS (fold; none block beyond the C/I above)

- **n-1:** The import citation `parse_descriptor.rs:17-21` should read `:17-22` (the
  `use miniscript::{Descriptor as MsDescriptor, …}` line at `:22` is part of the set). Non-load-bearing.
- **n-2:** §1 cites `Cargo.toml:36 md-codec`, `:44 miniscript` — both verified exact. Good.
- **n-3:** The per-`@N` `tlv.use_site_path_overrides` map (referenced in `md-codec/src/encode.rs`,
  validated by `validate_multipath_consistency`) is a second address-set-fixing surface beyond baseline
  `use_site_path`. When folding C-PLAN-1, state explicitly whether the gate binds the override map too
  (it should, for completeness) — or argue why baseline `use_site_path` alone suffices for the engraved-
  vs-supplied comparison.
- **n-4:** §8.4 risk 2 already flags "the `use_site_path`-equality add-on (if needed) is the only open
  shape — plan-doc to confirm." C-PLAN-1 is the confirmation: it IS needed. Convert that risk note into
  the decided design.

---

## WHAT MUST CHANGE BEFORE GREEN

1. **C-PLAN-1:** Add `use_site_path` (and consider `tlv.use_site_path_overrides`) to the H1 keyed-multisig
   gate — `tree == && use_site_path ==` (origins still excluded). Add the RED P4 test for a `.tree`-equal /
   `use_site_path`-divergent md1 → `passed:false`, plus the clean-negative. Re-justify m-4 / Q-PLAN-R0-2 on
   the corrected (no-canonicalization-ambiguity) basis. Keep the NAME `md1_xpub_match` (Q-WIRE unaffected).
2. **I-PLAN-1:** Correct the false "toolkit has no second strip regex" claim (§1 / §3 m-3 / §6.1):
   acknowledge `substitute_synthetic:319` (`[0-9;]` strip), then decide — widen to `[0-9;'h]` for md-cli
   symmetry, OR justify no-widening via the `resolve_placeholders`(reject)-before-`substitute_synthetic`
   call order (`parse_descriptor.rs:768` before `:779`).

Fold both, persist this review verbatim, re-dispatch the plan-doc R0 (the reviewer-loop continues after
every fold, per CLAUDE.md). **Implementation MUST NOT begin until the plan-doc converges to 0C/0I.**
