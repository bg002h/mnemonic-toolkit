# Advisor (opus) — second opinion on SPEC + PLAN (verbatim)

> User-requested `/advisor opus` consult on the spec + implementation plan, AFTER the spec R0 loop
> reached GREEN. Dispatched via Agent tool (feature-dev:code-architect, model=opus). Verdict:
> **PROCEED-WITH-CHANGES.** Branch `older-timelock-advisory`, HEAD `4096d4c`. All action items folded
> in the commit following this file (load-bearing claims independently verified: build_descriptor.rs:500/530
> cost-preview discard is safe because gate::validate refuses masked older() upstream; Task-2 fixture
> helpers older_tree/field_diags exist at gate.rs:930/942).

---

VERDICT: PROCEED-WITH-CHANGES — premise right, architecture sound, R0 loop did real work. Three
gaps the R0 loop under-weighted (one a genuine correctness/swallowing risk on build-descriptor's
cost preview) + a proportionality push-back on xpub-search. None blockers; all cheap.

## 1. Premise & approach
Advisory-only is correct; framing right. Funds-safety asymmetry handled correctly: a masked older()
on a deployed wallet must round-trip untouched (refusing strands funds); authoring fresh is
hard-stopped (build-descriptor already does). bundle dual-nature (§2) reasoned correctly —
loud-but-non-blocking is the only tool that doesn't strand a mid-recovery backup. Rejecting the
--allow variant is the right trade. Nuance: the gate-comment reword shouldn't imply the gate
"should have" covered bundle — the refuse-on-author / advise-on-intake split is the INTENDED design,
not a historical gap.

## 2. Scope / proportionality
7 surfaces defensible; xpub-search genuinely marginal (spec knows it). Accept it only because nearly
free once both adapters exist — BUT the A-raw-card bit-31 machinery exists SOLELY for xpub-search's
md1 funnel: the most complex part justified by the weakest surface (tail wagging the dog). Nothing
important excluded (inspect/repair/addresses/convert/decode-address + verify-bundle --md1 correctly
ruled out; after() correctly excluded). Strategic risk: advisory-blindness — older(65536)→0 is a
catastrophic silent failure so the signal is high-value when it fires, but it fires on EVERY intake
of an already-known-masked wallet, every surface, every run, unsuppressable → habituation. Don't add
a suppress flag now (scope creep); keep wording unmistakable; file a FOLLOWUP for eventual
--quiet-advisories.

## 3. Over/under-engineering
Two-adapter + generic-core split is correct, not over-engineered — R0 rounds 1→2 discovered
compare-cost holds Miniscript<DefiniteDescriptorKey,_> vs others' Descriptor<DescriptorPublicKey>,
so the generic core is FORCED. Adapter A vs B forced too (different parsed forms; unifying = re-parse
= worse). The bit-31 three-regime distinction is the one place to challenge: real (md_codec
read_node does zero validation, tree.rs:293-295) but only a hand-crafted/corrupted card fed to
xpub-search's md1 funnel produces it — adversarial-only, not a real user footgun. Keep the variant
(gate's IR path reaches bit-31 regardless — cost sunk), but don't let the implementer treat the
bit-31 cell as load-bearing-for-real-users. Masked{0} (older(65536)) is the case that matters.

## 4. Risk — items the R0 loop under-weighted
(a) **build_descriptor.rs cost-preview call sites — the real one.** run_compare_cost has TWO
production call sites inside build-descriptor (build_descriptor.rs:500, :530 cost_preview_value).
Task 8 Step 3d's "pass &mut Vec::new()" would SWALLOW the advisory there. Acceptable — build-descriptor
hard-refuses masked older() in validate_fields BEFORE cost preview, so unreachable/redundant — but
must be a DELIBERATE, DOCUMENTED discard, not a mechanical empty Vec. An implementer following Step 3d
literally routes a production path's stderr to /dev/null with no comment. R0 should have flagged
run_compare_cost is reused inside the gated surface.
(b) **Dedup** is by operand value (BTreeSet<u32>): older(65536) twice → one line; older(65536)+
older(131072) → two lines (both distinct, both masked-to-0, correctly shown). Does NOT hide a distinct
second masked value. Two operands masking to same effective but different literal (65536 vs 0x800000)
kept distinct (keyed on raw u32). Correct. Task-11 dedup assertion should pin OPERAND-keyed dedup
explicitly.
(c) **stderr-parsing downstream tooling** — low risk; --json writes to stdout, advisory to stderr;
secret_advisory precedent (~8 surfaces) means downstream already tolerates toolkit stderr advisories.
Add one assertion: advisory does NOT appear on stdout for --json invocations.
(d) **compare-cost signature change blast radius** — adequately planned; Task 8 Step 4 "cargo build
catches missed call sites" is the right net (compiler-enforced).
(e) **taproot/tapscript older()** — adequately handled (Tr leaves via TapTree::leaves()→core; Adapter A
Body::Tr{tree}; compare-cost segv0-only justified by reverse-projection round-trip, R0-r3 m1).
(f) **gate refactor risking the diagnostic** — well-mitigated; characterization test pins byte-identical
BEFORE refactor; message strings stay in gate. Caveat: Task 2 Step 1's {"older":65536} is a placeholder
to replace with the real SpecDoc fixture.

## 5. Plan quality
Executable, three stall points to pre-empt:
- Task 2 Step 1 ships placeholder JSON + guessed substrings — pin the exact existing fixture
  (older_tree/field_diags near gate.rs:990; substrings confirmed correct).
- Task 3 Step 1 hand-constructs an md_codec::Descriptor literal (flagged uncertain). Recommend: build
  the tree via md_codec encode/decode round-trip rather than literal fields — avoids PathDecl/UseSitePath
  fragility; serves both tests.
- Task 10 Step 1(b) bit-31 crafted card — commit to one construction (round-trip OR direct bit-encode),
  don't offer both.
TDD/regression adequacy strong (predicate table + byte-identical gate characterization + per-surface +
cross-surface + dedup + A-raw-card). One gap: no integration cell that the advisory does NOT fire on a
legitimately clean 512-second-unit descriptor (older(0x400001)) — the most likely false-positive vector
(fat-fingered bit-22 mask). Add one clean-512s integration cell.

## 6. Anything else
Sequencing fine (independent of open program work). 51 untracked cycle-prep scratch files in the tree →
explicit-stage discipline matters (accidental git add hazard); remind in Task 13. No better alternative
— unified single-walk would require re-parsing (worse). Architecture is the right one.

## Prioritized action items
MUST-FIX:
1. Document build_descriptor.rs:500,530 advisory-discard as deliberate (comment: authoring gate already
   refused masked older() upstream), not a silent empty Vec.
2. Commit Task 3 + Task 10 md1-card construction to encode/decode round-trip (not hand-built Descriptor
   literal).
3. Pin the exact existing gate test/fixture name in Task 2 Step 1 (replace {"older":65536} placeholder).
SHOULD-CONSIDER:
4. Add a clean-512s-unit integration cell (older(0x400001) → silent on a real surface).
5. Add a --json-stdout-cleanliness assertion (advisory stays on stderr).
6. Make the Task-11 dedup assertion operand-keyed.
7. File a FOLLOWUP for advisory-blindness mitigation (eventual --quiet-advisories).
8. Reframe the gate-comment reword so it doesn't imply the gate should have covered bundle.
