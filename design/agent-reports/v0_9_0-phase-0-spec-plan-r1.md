# v0.9.0 Phase 0 — SPEC + plan + survey R1 architect review + disposition

**Date:** 2026-05-13
**Reviewer:** `feature-dev:code-reviewer` (Sonnet 4.6), dispatched
per plan Phase 0 step 4.
**Artifacts reviewed:**
- Survey: `agent-reports/v0_9_0-secret-memory-survey.md`
- SPEC: `SPEC_secret_memory_hygiene_v0_9_0.md`
- Plan: `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`

## R1 verdict

**2C / 3I / 2N.** Folded all 5 substantive findings before R2.
Pending architect re-review (R2) to verify folds.

## R1 findings + disposition

### C-1 — argv flag count contradiction: FOLDED

R1 finding: SPEC §2 claimed "21 inline-secret flags / 13 needing
closure" for toolkit + "+2" for ms-cli, but survey §5 toolkit table
has 20 rows (not 21) with 9 flag-rows marked NO (not 13), and
survey §5 ms-cli table marks all 5 flag-rows YES (not +2).

**Fold:**
- SPEC §2 toolkit argv row: `8 / 21 → 21 / 21 | +13` →
  `11 / 20 → 20 / 20 | +9 (via 5 distinct implementation changes)`.
- SPEC §2 ms-cli argv row: `3 / 5 → 5 / 5 | +2` →
  `5 / 5 → 5 / 5 | +0 (ms-cli is Phase 2-only)`.
- SPEC §2 added "Toolkit argv count derivation" + "ms-cli argv
  count derivation" deterministic breakdowns showing exactly which
  survey rows go to YES vs NO.
- Plan Phase 1 scope: rewrite from "13 toolkit flags + 2 ms-cli
  flags" to "9 toolkit flag-rows closed via 5 distinct
  implementation changes; ms-cli has no argv work this phase."
- Plan Phase 1 RED: 9 cells (was 13 + 2 = 15).
- Plan Phase 1 exit gate: "All 9 argv-closure cells pass + 9
  secret-in-argv warning cells pass" (was "All 15 cells").

### C-2 — ms-cli OWNED row undercount: FOLDED

R1 finding: SPEC §2 claimed "7 ms-cli rows" but survey §1 ms-cli
table has 11 rows, 10 carry OWNED component. Three clap-field
rows (`encode.rs:30` `EncodeArgs::phrase`, `encode.rs:34`
`EncodeArgs::hex`, `verify.rs:27` `VerifyArgs::phrase`) carrying
OWNED dispositions for the entire process lifetime were omitted
from plan Phase 2 scope.

**Fold:**
- SPEC §2 ms-cli Zeroizing row: `7 → 10`.
- Plan Phase 2 Impl step 3 (ms-cli wraps): explicitly added the 3
  clap-field rows with the "consume + immediately wrap" pattern
  (`Zeroizing::new(std::mem::take(&mut args.phrase).unwrap_or_default())`
  at `run()` entry) since clap-derive does not natively produce
  `Zeroizing<String>`.
- `decode.rs:67-94` STDOUT-LEAK row stays OOS (the dominant
  disposition); added new SPEC §3 OOS-10 entry to record this
  explicitly.

### I-1 — ms-codec lib.rs:18-19 doc-example OOS-or-include: FOLDED (OOS)

R1 finding: survey §1 ms-codec table has 5 OWNED rows including
the public doc-test at `lib.rs:18-19,29-30`. SPEC §2 claimed "4
ms-codec rows." The doc-example is OWNED-tagged but is doc-test-
only synthetic data, not real production secret material.

**Fold:** OOS rather than include. New SPEC §3 OOS-7 entry:
`ms-codec-doc-example-zeroize-consistency` (ms-codec) — doc-tests
use synthetic vectors, wrapping adds visual noise to the public
API documentation without security benefit. SPEC §2 ms-codec
count stays at 4 (production rows: `payload.rs:29`, `decode.rs:45`,
`envelope.rs:122-131`, `envelope.rs:141-156`).

### I-2 — ms-cli positional stdin reaffirmation ambiguity: FOLDED

R1 finding: survey §5 marks all 5 ms-cli flag-rows YES (have
stdin), but SPEC §2 claimed "+2" implying 2 ms-cli flags need
closure. Plan Phase 1 framed it as "verify stdin route works for
positional."

**Fold:** removed ms-cli from Phase 1 entirely (survey is
authoritative — all 5 ms-cli flag-rows already have stdin
escape). SPEC §2 ms-cli argv row corrected to `+0`. Phase 1
scope says "ms-cli has no argv-closure work this phase per R1
I-2 fold; ms-cli participation is Phase 2-only." The
`lint_argv_secret_flags.rs` canonical-list-check still includes
all 5 ms-cli flag-rows for symmetry-verification (asserts each
has its known stdin escape registered) but no new flag is added.

### I-3 — Phase 3 RED stub design ambiguity: FOLDED

R1 finding: Plan Phase 3 RED stub said `secret_mem.rs` exports
`unimplemented!()` but didn't make explicit whether the stub's
declared signatures match the test's expected calls (compile-
error RED vs runtime-panic RED).

**Fold:** Plan Phase 3 RED step 1 now explicitly declares the
full type signatures (`SoftReason` enum + `MlockOutcome` enum +
`try_mlock_region` fn with body `unimplemented!()`). Tests
compile against the full signature; RED is runtime panic via
`unimplemented!()`. The `MNEMONIC_TEST_MLOCK_FAIL_MODE` env-var
mock toggle is added at Phase 3 Impl step 3 (the env-var read is
behind a `#[cfg(test)]` gate inside `secret_mem.rs`; production
builds ignore it).

### N-1 — "OWNED" qualifier missing from SPEC §2: FOLDED

R1 finding: SPEC §2 toolkit Zeroizing row said "21 toolkit
survey-§1 rows" without "OWNED" qualifier, while plan correctly
said "21 toolkit OWNED rows."

**Fold:** SPEC §2 toolkit Zeroizing row reframed as
"survey-§1 OWNED rows" + "~30 toolkit rows with OWNED component"
+ explicit Toolkit Zeroizing count derivation paragraph
explaining the row-counting filter applied.

### N-2 — lib.rs file inventory: NOT APPLICABLE

R1 finding: file inventory might need to list `ms-codec/src/lib.rs`
if doc-example included.

**Disposition:** N-2 moot now that I-1 was folded as OOS. No
file inventory change needed for lib.rs.

## R1 focus-area additional gaps folded

**Focus area 2 gap (survey §4 items 4-5 silent deferral):** added
new SPEC §3 OOS-8 (`bip85-entropy-heap-promote-mlock`) and OOS-9
(`ms-cli-stdin-buffer-mlock`) FOLLOWUP entries.

**Focus area 7 gap (FOLLOWUPS entry body for no-scope repos):**
added SPEC §5 "Per-repo phase field is filled in at FOLLOWUPS-open
time" sub-section enumerating the per-repo phase field for each
of the 4 sibling repos.

## Carry-forward

R1's N-1 / N-2 nits are folded (N-2 moot per I-1 fold). R1's
focus-area observations on Phase E semver (defensible-but-borderline
toolkit patch-vs-minor) and acceptance gate #2 (structurally sound,
relies on reviewer discipline) are carry-forward — no plan change
needed; will be re-litigated at Phase E gate.

## Architect agent

R1 agent ID: `aac4aefa211259868` (returned in tool result;
SendMessage not available in current tool palette, so R2 dispatched
as fresh `feature-dev:code-reviewer` invocation with the same
artifacts + fold-set context).

## R2 disposition (2026-05-13)

R2 returned **0C / 0I / 1N** with verdict **CLEAR** — Phase 0
closes at 0C/0I.

R2 fold-verifications (all `fold-verified`):
- **C-1** confidence 97 — toolkit `11 / 20 → 20 / 20 | +9` and
  ms-cli `5 / 5 | +0` deltas + derivation paragraphs verified.
- **C-2** confidence 95 — three clap-field rows (encode.rs:30,
  encode.rs:34, verify.rs:27) added to Phase 2 scope with
  `Zeroizing::new(std::mem::take(...))` pattern verified;
  `decode.rs:67-94` OOS-10 entry verified.
- **I-1** confidence 98 — SPEC §3 OOS-7 `ms-codec-doc-example-zeroize-consistency`
  verified.
- **I-2** confidence 99 — ms-cli removed from Phase 1 verified.
- **I-3** confidence 98 — Phase 3 RED stub full type signatures +
  env-var mock toggle verified.
- **N-1** confidence 99 — "OWNED" qualifier verified.
- **N-2** moot (confirmed).
- **Focus area 2 gap** confidence 97 — OOS-8 / OOS-9 verified.
- **Focus area 7 gap** confidence 99 — SPEC §5 per-repo phase
  field sub-section verified.

R2 surfaced **N-R2-1** (confidence 82): SPEC §4 Phase 1 description
still echoed the pre-fold "13 toolkit flags + 2 ms-cli flags"
language. **Folded** in R2 post-disposition: §4 Phase 1 bullet
updated to read "9 toolkit flag-rows ... no ms-cli argv work this
phase per R1 I-2 fold" — surrounding implementation language also
cleaned up to remove the stale `cmd/encode.rs` ms-cli reference.

R2 agent ID: `a3a7a71a4b1b01180`.

## Phase 0 close gate (post-R2): CLEAR — but R3 followed up

R2 cleared 0C/0I but the user requested an additional Opus-level
review before committing.

## R3 Opus disposition (2026-05-13)

R3 (Opus 4.7, `feature-dev:code-reviewer`, agentId `a469993b24add693d`)
returned **3C / 4I / 2N** with verdict **SPLIT-CYCLE**. Three
architectural defects R1/R2 missed:

- **C-R3-1** cycle is overscoped (~3-4 weeks; Phase 2+3 sequential;
  Phase 1↔Phase 2 file overlap; new cross-platform FFI risk).
  Recommended split: Cycle A (argv + Zeroizing) + Cycle B (mlock).
- **C-R3-2** SPEC's "consolidate five BIP-39→BIP-32 spines into a
  single canonical helper" claim is false against the source —
  the 5 sites differ on input type, network handling, derivation
  path source, and return shape; only the 1-line seed step
  consolidates.
- **C-R3-3** plan's "drop(xpriv) after last use so future Zeroize
  tightens automatically" is false reassurance — `Xpriv: Copy +
  no Drop` makes drop() a memory-no-op; upstream Copy removal
  would be a breaking-change cascade, not automatic tightening.
- **C-R3-4** patch tag (v0.9.2) violates project's pre-1.0 SemVer
  convention because changing `pub DerivedAccount.entropy: Vec<u8>`
  to `Zeroizing<Vec<u8>>` is a public-field-type breaking change.
  Should be v0.10.0 minor — UNLESS `impl Drop` is used instead
  (keeps public type stable).
- **I-R3-1** allocator-pool + libc-OsString residue not
  acknowledged in SPEC.
- **I-R3-2** `try_mlock_region(&[u8])` API signature traps callers
  into page-vs-byte granularity wastefulness.
- **I-R3-3** cycle naming overstates closure ("secret-memory-hygiene
  v0.9.0" implies categorical fix); rename to "first-pass" framing.
- **I-R3-4** md/mk no-scope-symmetry matrices are pure overhead;
  drop them.
- **N-R3-1** SPEC labeled "v0.9.0 cycle" but v0.9.0 toolkit tag
  already shipped (toolkit at v0.9.1); cycle internal version is
  fine but artifact filename inconsistency noted.
- **N-R3-2** Phase 0 FOLLOWUPS not yet opened (discipline
  reminder).

## User decisions on R3 findings (2026-05-13)

User answered three AskUserQuestion prompts:

1. **C-R3-1 cycle scope** → "Split per R3 recommendation."
   Cycle A (argv + Zeroizing, toolkit + ms-secret) ships now;
   Cycle B (mlock, toolkit-only) ships later as a separate cycle.
2. **C-R3-4 semver axis** → "Use impl Drop to keep public type
   stable." `DerivedAccount` keeps `pub entropy: Vec<u8>` field
   type; gets `impl Drop` for scrub-on-drop. Patch tag (v0.9.2)
   defensible. Residual semver risk (move-out destructuring
   break) accepted; documented in SPEC §3 OOS-pub-struct-drop.
3. **I-R3-4 md/mk symmetry stubs** → "Drop md/mk symmetry stubs."
   Phase 4 → Phase 3; only toolkit + ms-secret get matrix files.

## R3 architectural-honesty folds applied (regardless of scope decisions)

- **C-R3-2** SPEC §1 item 2 + plan Phase 2 step 4 rewritten:
  helper consolidates the seed-derivation step (1 line × 5 sites),
  not the full spines. Per-site master/account/leaf derivation
  remains site-specific.
- **C-R3-3** plan Phase 2 step 4 Xpriv paragraph rewritten:
  "Phase 2 makes no in-cycle attempt to scrub Xpriv memory…
  Note that upstream Copy removal would be a breaking change
  cascade, not automatic tightening."
- **I-R3-1** SPEC §3 added OOS-libc-osstring and
  OOS-allocator-residue entries with FOLLOWUPS.
- **I-R3-2** SPEC §3 added OOS-secret-arena entry; Phase 3 matrix
  §3 wording adjusted (mlock candidates → Cycle B).
- **I-R3-3** SPEC §1 reframed Cycle A as "OWNED-buffer first-pass"
  with explicit "what this cycle does NOT close" prose section
  (plan Phase 3 step 3 §0.5 mandate).
- **N-R3-1** artifact filenames kept (per v0.8.0 precedent: cycle
  internal version is independent of ship tag; "v0.9.0 cycle" is
  the cycle's name, "v0.9.2" is the toolkit ship tag).

## Cycle A scope after all folds

- Phases: 0 (this) → 1 (argv) → 2 (zeroize) → 3 (matrix) → E (rollup).
- Repos: toolkit (Phases 1, 2, 3, E) + ms-secret (Phases 2, 3, E).
- No md/mk participation.
- Tags: ms-codec-v0.1.3 + ms-cli-v0.1.X+1 + mnemonic-toolkit-v0.9.2
  (all patch).
- Cycle B (mlock) deferred; SPEC drafting begins post-Cycle-A ship.

## Phase 0 close gate (post-R3 + user folds): PENDING R4

R4 architect re-review needed on the substantially-rewritten SPEC
+ plan to confirm:
- Cycle A scope is internally consistent (no leftover Phase 3 / 4
  / mlock references in any artifact).
- `impl Drop` approach correctly described for `DerivedAccount`
  in both SPEC §3 OOS-pub-struct-drop and plan Phase 2 step 4.
- All 10 OOS-§3 entries are well-formed FOLLOWUPS pointers.
- Cycle A → Cycle B forward-pointers are correctly stated.

R4 will dispatch after R3 fold completion.

R3 agent ID: `a469993b24add693d`.

## R4 Opus disposition (2026-05-13)

R4 (Opus 4.7, `feature-dev:code-reviewer`, agentId
`aecc2e8246c021a3b`) verified the substantial post-R3 rewrite and
returned **1C / 3I / 0N**. The four R3-Critical findings landed
correctly. R4 caught one new critical issue from the rewrite + three
consistency drifts:

- **C-R4-1** `impl Drop` adds E0509 break for 3 in-toolkit
  move-out sites (`bundle.rs:325-329`, `bundle.rs:421-425`,
  `synthesize.rs:741-744`). SPEC §3 OOS-pub-struct-drop +
  plan Phase 2 step 4 had silently characterized this as
  external-only residual risk. Phase 2 must pre-empt by adding
  `DerivedAccount::into_parts()` consuming method before
  adding `impl Drop`.
- **I-R4-2** plan Phase 2 scope blurb still said "7 ms-cli rows
  / 32 rows total" (stale from before R1 C-2 fold updated to 10).
- **I-R4-3** SPEC §2 + plan Phase 3 step 6 cited "10 SPEC §3
  FOLLOWUPS" — actual count is 14 post-rewrite (added: OOS-pub-struct-drop,
  OOS-libc-osstring, OOS-allocator-residue, OOS-mlock-cycle-b,
  OOS-secret-arena).
- **I-R4-4** SPEC §3 OOS-mlock-cycle-b only generically referenced
  "originally-planned Phase 3" without naming the specific 5
  survey-§4 candidates (#1-3 + #4-5). The pre-R3 separate FOLLOWUPS
  for #4 (`bip85-entropy-heap-promote-mlock`) and #5
  (`ms-cli-stdin-buffer-mlock`) were dropped without explicit
  roll-in.

## R4 folds applied

1. **C-R4-1 fold** — plan Phase 2 step 4 DerivedAccount sub-bullet
   now includes "Phase 2 prerequisite — migrate internal move-out
   sites before adding impl Drop" with the three sites named and
   the `DerivedAccount::into_parts()` consuming-method remediation
   pattern described. SPEC §3 OOS-pub-struct-drop now distinguishes
   external residual risk vs internal Phase 2 prerequisite.
2. **I-R4-2 fold** — plan Phase 2 scope blurb refreshed to
   "~30 toolkit + 4 ms-codec + 10 ms-cli (incl. 3 clap-field rows
   added post-R1 C-2) ≈ ~44 rows". Phase 2 RED step 3 refreshed
   to "10 ms-cli rows".
3. **I-R4-3 fold** — SPEC §2 "Net new hardening cells" line
   refreshed to "14 entries post-R3+R4". Plan Phase 3 step 6
   refreshed to "14 SPEC §3 FOLLOWUPS".
4. **I-R4-4 fold** — SPEC §3 OOS-mlock-cycle-b expanded with
   explicit "Cycle B target set" sub-list naming all 5 survey-§4
   candidates (#1 clap-Args, #2 ResolvedSlot.entropy, #3
   DerivedAccount.entropy, #4 bip85 entropy heap-promote, #5
   ms-cli read_stdin). The two pre-R3 sub-FOLLOWUPS for #4 / #5
   are now rolled into the parent OOS-mlock-cycle-b entry rather
   than separate.

R4 agent ID: `aecc2e8246c021a3b`. R5 mechanical fold-verify
pending.

## R5 Sonnet mechanical fold-verify (2026-05-13)

R5 (Sonnet 4.6, `feature-dev:code-reviewer`, agentId
`a83784d19aa59f716`) returned **0C / 0I / 0N** with verdict
**CLEAR**. All four R4 folds verified at cited file:line.

R5 evidence:
- C-R4-1: plan lines 247-265 (Phase 2 step 4 prerequisite +
  three sites + into_parts pattern); SPEC lines 228-246 (OOS-pub-struct-drop
  internal vs external risk distinction).
- I-R4-2: plan lines 154-158 (~44 rows); plan line 176 (10 ms-cli
  rows).
- I-R4-3: SPEC lines 161-166 (14 entries); plan line 357 (14
  FOLLOWUPS).
- I-R4-4: SPEC lines 281-305 (5-candidate Cycle B target set).

R5 agent ID: `a83784d19aa59f716`.

## Phase 0 close gate: CLEAR (final)

All R1+R2+R3+R4 findings folded and R5-verified. Phase 0
deliverables ready to commit:
1. `design/SPEC_secret_memory_hygiene_v0_9_0.md`
2. `design/agent-reports/v0_9_0-secret-memory-survey.md`
3. `design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md` (this file)

Phase 1 authorization pending user approval. Cross-repo FOLLOWUPS
entries to open in toolkit + ms-secret at commit time per plan
Phase 0 step 6.

## Architect-review summary

| Round | Model | Verdict | Folded |
|---|---|---|---|
| R1 | Sonnet 4.6 | 2C / 3I / 2N | 5 substantive + 1N |
| R2 | Sonnet 4.6 | 0C / 0I / 1N | 1N |
| R3 | Opus 4.7 | 3C / 4I / 2N (SPLIT-CYCLE) | 4C + 4I + 2N + user decisions |
| R4 | Opus 4.7 | 1C / 3I / 0N | 4 findings |
| R5 | Sonnet 4.6 | 0C / 0I / 0N (CLEAR) | mechanical verify |
