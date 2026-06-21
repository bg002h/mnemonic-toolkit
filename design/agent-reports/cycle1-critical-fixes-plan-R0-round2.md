# R0 REVIEW — IMPLEMENTATION_PLAN_cycle1_critical_fixes (H13 / H12 / H1) — Plan-doc Round 2

**Reviewer role:** opus architect, mandatory R0 gate on the IMPLEMENTATION PLAN-DOC (CLAUDE.md hard-gate:
NO implementation before 0C/0I). Round-1 was NOT-GREEN (1 Critical C-PLAN-1, 1 Important I-PLAN-1); both
were folded. This round-2 confirms closure AND checks the fold introduced no new C/I.
**Plan-doc under review:** `design/IMPLEMENTATION_PLAN_cycle1_critical_fixes.md` (post-fold revision).
**Round-1 review (findings to verify closed):** `design/agent-reports/cycle1-critical-fixes-plan-R0-round1.md`.
**GREEN spec:** `design/BRAINSTORM_cycle1_critical_fixes.md`.
**Date:** 2026-06-20.
**Canonical sources verified live (NOT the working tree — this checkout is on another instance's WIP):**
- toolkit `origin/master` = `4d5872ed489e706155b0d88b02686977e59a20b6` (confirmed `git rev-parse`).
- descriptor-mnemonic (md-codec + md-cli) `origin/main` = `54dd765a11d490dc3d8dec2c842dae718bd3ef2b`
  (confirmed; NO `origin/master`, default `main`). Current versions: toolkit `0.60.0`, md-codec `0.37.0`,
  md-cli `0.7.1`.

Every load-bearing citation re-grepped against `git show origin/<branch>:<path>`. The verdict rests on
first-hand source reads, NOT the draft text.

---

## VERDICT: **GREEN — 0 Critical / 0 Important**

Both round-1 findings are correctly resolved and the fold introduced no new Critical or Important issue. The
central adversarial question — whether adding `use_site_path` AND the per-`@N` `tlv.use_site_path_overrides`
map to the H1 `==` gate can produce a spurious verify FAILURE on a legitimately-equal wallet — is answered
**NO** against canonical md-codec source: both fields are faithful, order-significant representations with NO
in-field canonicalization, AND every operand in the H1 path is a **decoded** `md_codec::Descriptor` whose
canonical form is enforced at the decode boundary. Three folded MINOR/NIT-class polish items below; none
block. **This clears implementation.**

---

## CRITICAL FINDINGS

**None.**

---

## IMPORTANT FINDINGS

**None.**

---

## ROUND-1 CLOSURE

### C-PLAN-1 (H1 scope) — **RESOLVED ✓**

The H1 compare is now `expected_md_decoded.tree == desc.tree && expected_md_decoded.use_site_path ==
desc.use_site_path` (plan §0 L41/L49, §1 L139-148, §2 P4 L305-306, §3 m-4 L409-435, §6.3 L539-546), with
`path_decl` and the origin/fingerprint TLV entries EXCLUDED. The fold ALSO binds the per-`@N`
`tlv.use_site_path_overrides` map (n-3). I verified each sub-claim adversarially:

**(a) Does `use_site_path` AND `use_site_path_overrides` genuinely drive derivation / the watched-address
set? — YES.**
- `md-codec/src/derive.rs::derive_address` (`:92`): the pre-flight reads `self.use_site_path.multipath`
  (`:110-111`) to bound `chain`, then calls `to_miniscript::to_miniscript_descriptor(self, chain)` (`:124`).
- `to_miniscript.rs::to_miniscript_descriptor` (`:54`) builds each cosigner key from `e.use_site_path`
  (`:62-65`) — the comment names passing the SHARED baseline instead of the per-`@N` resolved path as "the
  silent-wrong-address bug for per-cosigner override cards." The per-`@N` resolution scans
  `d.tlv.use_site_path_overrides` directly (`:85-92`), inheriting `d.use_site_path` only where no override
  exists. **So BOTH the baseline `use_site_path` AND `tlv.use_site_path_overrides` fix the derived address
  set.** Binding both in the gate is funds-correct; binding baseline alone would leave the per-cosigner
  divergent-path class (the exact #25/#26 silent-wrong-address bug class) unbound.

**(b) CRITICAL FALSE-FAIL CHECK — can two semantically-identical wallets produce non-equal
`use_site_path`/overrides and thus a spurious verify FAILURE? — NO, both fields are safe to `==`.**
- `use_site_path.rs:48-53` `UseSitePath { multipath: Option<Vec<Alternative>>, wildcard_hardened: bool }`
  derives `PartialEq, Eq`; `:18-23` `Alternative { hardened: bool, value: u32 }` derives `PartialEq, Eq,
  Copy`. A targeted grep of `use_site_path.rs` for `sort|canonical|normal` returns NONE — the field is a
  faithful, order-significant 1:1 of the descriptor string. An `==` on it does NOT false-fail. The plan's
  C-PLAN-1 basis for the BASELINE field is correct as written.
- **The overrides map needs a deeper check, and it passes — but for a reason the plan states imprecisely
  (see m-NEW-1 below).** `tlv.use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>` (`tlv.rs:26`); the
  `TlvSection` (`:23-24`) derives `PartialEq, Eq`. CRUCIALLY, unlike the baseline, the overrides map IS
  touched by canonicalization: `canonicalize.rs::canonicalize_placeholder_indices` (`:168`) remaps the
  `@N` index keys via `remap_tlv_vec` (`:144-149`), which permutes `*idx = perm[*idx]` AND re-sorts ascending
  (`:148`). So on its FACE the override `@N` keys are in the same placeholder-canonicalization category as
  origins. **What neutralizes the false-fail risk is the DECODE BOUNDARY, not field-level stability:** the H1
  gate compares two operands BOTH produced by decoding md1 — `expected_md_decoded` via
  `md_codec::chunk::reassemble(&expected_md1_strs)` (`verify_bundle.rs:2700`) and `desc` from
  `supplied_md_decoded` (the `Ok(desc)` arm). The decoder (`decode.rs::decode_payload`) enforces canonical
  form on EVERY decoded `Descriptor`: `validate_placeholder_usage` (`:56`) REJECTS any wire whose tree
  first-occurrences are not ascending (`Error::PlaceholderFirstOccurrenceOutOfOrder`, `validate.rs:30`);
  `validate_use_site_overrides_canonical` (`:62` / `validate.rs:164-178`) REJECTS an `@0` override
  (`BaselineUseSiteOverride`) or any override equal to the baseline (`RedundantUseSiteOverride`); and the TLV
  decode keeps the idx column strictly ascending (`canonicalize.rs:301-323`
  `tlv_indices_strictly_ascending_and_in_range`, `tlv.rs:200` `entries.sort_by_key`). Therefore two
  semantically-identical wallets decode to BYTE-IDENTICAL `tree` index labels AND `use_site_path_overrides`
  `@N` keys — `==` cannot false-fail. This is the SAME decode-canonicalization mechanism that already makes
  `tree ==` safe; the override map rides it. **Binding the override map is safe.**

**(c) Origins remain excluded (no L14 false-positive). — CONFIRMED ✓.** `path_decl` and the origin/fingerprint
TLV entries (`tlv.fingerprints`, `tlv.origin_path_overrides`) STAY out of the gate (§1 L147, §3 L428-433,
§6.3 L543). These carry elision/canonicalization brittleness (`canonical_origin.rs`,
`canonicalize_placeholder_indices` origin paths) that WOULD false-FAIL legitimately origin-elided
descriptor-mode bundles — the exact reason the v0.5.0 B.3 multiset change exists. Correctly excluded.

**(d) RED tests + green cases + differential shape. — CONFIRMED ✓.** §2 P4 test 5 is the C-PLAN-1 RED case:
identical `.tree`, `wsh(sortedmulti(2,@0/<0;1>/*,…))` vs `<2;3>` change-chains → DIFFERENT watched-address
set → `passed:false`; plus a bare-`/*` vs `<0;1>/*` presence/count variant. Test 6 (genuine match: identical
`.tree` AND identical `use_site_path` → `passed:true`) and test 7 (origin-elided-but-equal → `passed:true`,
origins excluded) are correct and NOT in tension with test 5. The `bitcoind_differential.rs` H1 row
(§2.5 L367-374) adds the `<0;1>`-vs-`<2;3>` divergent shape; the "different addresses" premise is anchored
by `derive_receive` (the harness already exercises divergent multipath groups), the verdict assertion itself
is verify-bundle exit-code-behavioral — correct (no Core derive needed for the mismatch verdict). The
`md1_xpub_match` NAME is preserved, predicate widened only — Q-WIRE unaffected (re-confirmed below).

**Net:** C-PLAN-1 is closed correctly AND the fold's extension to the override map is a genuine
funds-safety improvement, verified false-fail-free against the decode boundary.

### I-PLAN-1 (m-3 strip regex) — **RESOLVED ✓**

**Call-order premise verified on canonical source.** Toolkit `parse_descriptor.rs` (region `:758-782`):
`lex_placeholders` (`:767`) → `resolve_placeholders` (`:768`, which calls `make_use_site_path` —
to-be-widened-to-`Result` at `:223` — raising the primary `DescriptorParse` reject) runs BEFORE
`substitute_synthetic` (`:779`). The plan's `:767`/`:768`/`:779` citations are EXACT. md-cli
`parse/template.rs` (`parse_template` `:1741`): `lex_placeholders` (`:1747`) → `resolve_placeholders`
(`:1748`) → `substitute_synthetic` (`:1750`); plan's `:1747`/`:1750` EXACT. So the hardened reject (raised in
`make_use_site_path` during resolve) pre-empts BOTH strip regexes on the production path — they are
unreachable with hardened input. Correct.

**The "toolkit DOES have a second strip regex" correction verified.** `parse_descriptor.rs` has FOUR
`Regex::new`: `:69` (lexer), `:277` (`tr(NUMS\b`), `:299` (`^tr\(`), `:319` (strip,
`(?:/<[0-9;]+>)?` `[0-9;]` class inside `pub fn substitute_synthetic` `:313`). The `:319` regex IS the
structural twin of md-cli's `template.rs:365` (`(?:/<[0-9;]+>)?`, same `[0-9;]` class). The plan's earlier
"none a multipath strip" claim was indeed wrong; the correction is accurate.

**Adversarial check on the "symmetric widen both strip classes to `[0-9;'h]`" defense-in-depth — SAFE, no
regression.** The strip regexes are USED to remove the placeholder/path tokens before handing the residual
to miniscript parsing. Widening the multipath body class from `[0-9;]` to `[0-9;'h]` only makes the strip
ALSO match a hardened-marked alt (`<0';1'>`) so it is stripped-and-discarded rather than left as a residual
that mis-parses. For a NON-hardened input (`<0;1>`) the widened class `[0-9;'h]` still matches the same
characters `0;1` it matched before (the added `'`/`h` simply never appear) — so the strip removes EXACTLY
the same span. No non-hardened input now strips something it shouldn't: the surrounding anchors (`@(\d+)`,
the optional `/<…>` and `/\*` groups) are unchanged, and `'`/`h` are not metacharacters inside a class. This
is pure defense-in-depth for direct `pub`/test callers of `substitute_synthetic` (toolkit `:1441`/`:2594`;
md-cli test callers) that bypass the lexer/resolve reject — NOT the funds-safe close (the reject is). The
plan correctly states the widen is OPTIONAL-robustness and keeps the two regexes in lockstep. No regression.

---

## ADVERSARIAL FOLD-DRIFT + FINAL CHECKS

- **No residual `.tree`-only language.** A grep for `tree.only|\.tree.ONLY|only .tree` across the plan-doc
  returns ONE hit (§3 L410) — the explicit CORRECTION statement ("the prior `.tree`-ONLY resolution was
  WRONG and is CORRECTED"), not a live gate. Every H1-gate phrasing across §0/§1/§2/§3/§6.3/§8/§10/§11/§12
  reads `tree == && use_site_path ==` (origins excluded) uniformly. The two "root-tag-only" mentions (L157,
  L322) correctly name a REJECTED design (a wrapper hack), distinct from the chosen `tree ==`.
- **§0 vs body — consistent.** §0 index (L41) and four-resolutions summary (L49) match §3/§6.3/§12. No
  contradiction.
- **§12 fold log accurate.** Both folds described match what the body now says and what the round-1 review
  required; the "Internal consistency confirmed" paragraph (L764-768) holds against my grep.
- **SemVer — UNCHANGED by the fold ✓.** md-cli MINOR `0.7.1`→`0.8.0` (behavioral typed-reject; `main.rs`
  catch-all `Err(e)=>from(1)` confirmed). toolkit MINOR (H13-mirror behavioral floor), INDEPENDENT, NO
  pin bump — `Cargo.toml:36 md-codec="0.37"`, `:44 miniscript="13"`, NO `md-cli` dep (md-cli is `[[bin]]
  name="md"`, not a lib). C-PLAN-1's predicate widening adds no flag/wire-shape ⇒ SemVer floor unchanged
  (§4/§8 L606-609). Re-confirmed.
- **Q-WIRE — UNCHANGED by the fold ✓.** NAME stays `md1_xpub_match` (verify_bundle.rs emit site, `VerifyCheck`
  free-form `name: String` at `format.rs:132`); only the `passed` predicate (and `detail`) change. No new
  `checks[]` element, no field add, no rename → no `--json` wire-shape change → no GUI paired-PR / no
  `schema_mirror` (flag-NAMES only) / no manual leg. The widened predicate (adding the `use_site_path` term)
  does not touch the NAME or shape. Confirmed.
- **Exit codes (m-1) — CORRECT.** md-cli `TemplateParse` ⇒ exit 1 (`main.rs:256-258` catch-all; no numeric
  arm). toolkit `DescriptorParse` ⇒ exit 2 (`error.rs:539`). Per-repo exact-code asserts right.
- **H12 facts — VERIFIED.** `compute_default_origin_path` (`bundle.rs:2210`) hardcodes 4th `PathComponent
  {hardened:true, value:2}` (`:2228-2232`), 2-arg sig; `verify_bundle.rs:1373` calls the same helper;
  `bip48_script_type` (`template.rs:231-237`) `ShWsh→1/Wsh→2/Tr→3`; `descriptor_intake.rs:324/345`
  `bip48_default_path(.., 2)` literal, `:297` `MsDescriptor::from_str` (no Tag). Per-site detection design
  correct.
- **Version sites / FOLLOWUPS — VERIFIED.** toolkit `0.60.0` (`Cargo.toml`), `install.sh` self-pin
  `mnemonic-toolkit-v0.60.0`. Stale FOLLOWUP `verify-bundle-multisig-md1-xpub-match-set-equality` header
  `FOLLOWUPS.md:1635`, status `resolved by v0.5.0 Phase B.3` `:1641` — the entry to re-open. Matches §10.
- **Phase/TDD/exit-criteria — coherent.** Single-subagent-per-phase, RED-first, class-A fixes gated on their
  differential-oracle row (H12 row gates P3; H13 row gates P1/P2; H1 discriminator rows gate P4),
  per-phase reviewer-loop to 0C/0I, mandatory whole-diff post-impl review. All consistent with CLAUDE.md.

---

## MINOR / NITS (fold opportunistically; none block — GREEN stands)

- **m-NEW-1 (the only fold-introduced imprecision — MINOR, not Important):** §2 P4 (L311-312) and §3 m-4
  (L411-412) justify binding `tlv.use_site_path_overrides` by saying it "is validated against the baseline by
  md-codec `validate_multipath_consistency`." That validator (`validate.rs:127-148`) only checks multipath
  **alt-COUNT** equality across baseline+overrides — it is NOT what makes the override map `==`-safe. The
  ACTUAL false-fail-safety basis is the DECODE-BOUNDARY canonical enforcement
  (`validate_use_site_overrides_canonical` rejects `@0`/baseline-redundant overrides; placeholder-usage
  ordering + TLV ascending-sort give deterministic `@N` keys), since both H1 operands are decoded md1.
  Recommend the P4 subagent cite the decode-canonicalization basis (and note that `use_site_path_overrides`
  IS canonicalization-touched at `canonicalize.rs:221`, made safe by the decode boundary — contrast the
  baseline `use_site_path`, which is intrinsically stable). The DECISION (bind both; origins excluded) is
  CORRECT and safe; only the cited reason is slightly off. The plan's explicit subagent escape hatch ("MAY
  narrow to baseline-only with an in-code justification") keeps this from being load-bearing.
- **n-1 (carried, non-load-bearing):** §1 cites the toolkit `make_use_site_path` call sites as `:193`/`:197`
  in one place and `:194`/`:198` elsewhere; canonical lines are 193 (`at0 = by_i[&0]`), 194
  (`make_use_site_path(at0)`), 198 (`make_use_site_path(occ)`). Both calls exist and are correctly
  identified; the off-by-one is cosmetic.
- **n-2 (carried):** import-span `:17-21` vs `:17-22` (round-1 n-1). Non-load-bearing.

---

## WHAT THIS CLEARS

The plan-doc is **R0-GREEN (0 Critical / 0 Important)**. Both round-1 findings (C-PLAN-1, I-PLAN-1) are
correctly resolved; the fold introduced no new Critical/Important issue. The load-bearing adversarial
question — false-fail safety of adding `use_site_path` AND `tlv.use_site_path_overrides` to the H1 `==` gate —
is resolved SAFE against canonical md-codec source (decode-boundary canonicalization guarantees identical
operands for semantically-identical wallets). **Implementation MAY begin** per CLAUDE.md (single-subagent-
per-phase TDD in a worktree, RED-first, per-phase reviewer-loop to 0C/0I, mandatory whole-diff post-impl
review). Fold m-NEW-1's citation polish opportunistically into the P4 work; it does not gate.
