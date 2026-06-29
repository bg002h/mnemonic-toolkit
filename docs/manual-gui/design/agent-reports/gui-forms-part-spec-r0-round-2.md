# R0 review — SPEC_gui_forms_dedicated_part.md (round 2)

**Reviewer:** opus architect (adversarial; round-2 convergence + fold-drift hunt)
**Subject:** `design/SPEC_gui_forms_dedicated_part.md` after folding round-1's single Important (I-r1-1: the 61 cross-links were un-gated).
**Round-1 result:** 0C / 1I (RED). Anchor convention (§3) ruled GREEN/bulletproof and is NOT re-litigated here.
**Fold under review:** §4 now single-sources both the gallery anchor and the chapter cross-link from the canonical `transcripts/gui/*.gui` stem list (remedy a); §6 adds a NEW bidirectional fail-closed `gui-form-xref` lint phase (`tests/check_gui_form_xref.py`) (remedy b). Round-1 recommended BOTH.
**Re-verified live against current source (manual-gui @ toolkit master):** `tests/lint.sh` (7-phase, `$SRC_DIR`-based phases 3/7), `tests/check_gui_schema_coverage.py` (`is_schema_shaped` def@101, `startswith` match@113; `build_expected`@66; `collect_html_ids`@91 — spec's "101-115 / 66-88 / 91-98" citations still accurate, no fold-induced citation drift), `Makefile` `lint:` target (args@297-305), `.github/workflows/manual-gui.yml` (CI runs `make lint`@106), `transcripts/gui/` (61 files; per-tab 32/10/10/9 confirmed by `ls`), `src/30-tour/31-first-launch.md` (2 `include="gui/…"`@21,82), `src/40-mnemonic/42-bundle.md` (render fence@28, schema anchors intact).

---

## VERDICT: GREEN — 0 Critical / 0 Important

I-r1-1 is **fully closed**. The convergence is real, not papered-over: the fold adopts BOTH halves of round-1's remedy (single-source generation + a bidirectional fail-closed gate), and the gate provably catches every enumerated cross-link failure mode — single-side typos are *double*-caught (forward "missing" + reverse "orphan"). No fold-introduced contradiction. Three non-blocking Minors are carried to the plan-doc R0. **Proceed to plan-doc.**

---

## 1. Does `gui-form-xref` fully close I-r1-1? — YES, sound + complete

**The check is enumerate-stems-then-require-exact-match, NOT scan-links-then-validate.** §6's phrasing is unambiguous on this: *"enumerate the canonical stem list `transcripts/gui/*.gui`; assert **for each** `<tab>-<sub>` — (i) EXACTLY ONE `{#gui-form-<tab>-<sub>}` anchor … AND (ii) EXACTLY ONE `](#gui-form-<tab>-<sub>)` cross-link …"*. The loop is keyed on the stem list (the census source of truth), and for each stem requires the *exact* expected anchor/link string. That is exactly the form the prompt asked me to confirm. Plus a reverse "orphan" clause: *"no `gui-form-*` anchor/link exists without a matching `.gui` stem."*

Failure-mode coverage (the five the prompt enumerated), traced through the spec's logic:

| Failure mode | Caught by | How |
|---|---|---|
| Typo'd cross-link `](#gui-form-mnemonic-bundel)` | forward (ii) **and** reverse orphan | stem `mnemonic-bundle` finds 0 exact links → MISSING; token `…-bundel` maps to no stem → ORPHAN. Double-caught. |
| Missing cross-link | forward (ii) | stem finds 0 links → MISSING |
| Missing gallery anchor | forward (i) | stem finds 0 anchors → MISSING |
| Duplicate gallery anchor | forward (i) "exactly one" | stem finds 2 anchors → FAIL |
| Orphan `gui-form-X` with no `.gui` stem | reverse orphan | token X maps to no stem → ORPHAN |

All five caught; the single-side-typo class is caught twice over (the most likely real-world defect). I could not construct a cross-link failure mode the check misses. The "exactly one … **in the subcommand chapters**" scoping on (ii) is a deliberately good choice: it confines the link-count assertion to the subcommand chapters, so a future gallery-overview "table of contents" that *also* links each form would not false-trip the count (those links still pass the reverse orphan check because they map to real stems). This neatly forecloses the one over-strictness false-positive I went looking for.

**Wiring soundness.** `tests/check_gui_form_xref.py` belongs in the same `$SRC_DIR`-grep family as the existing phase-3 (lychee) and phase-7 (index bidirectional) checks, both of which already operate purely on `$SRC_DIR` markdown — so the new phase fits the house pattern. CI picks it up for free: `manual-gui.yml:106` runs `make lint`, and `make lint` (Makefile@296) shells `tests/lint.sh`; appending an 8th phase to `lint.sh` is automatically exercised by the existing CI lint job. (The spec's "wired into … manual-gui.yml" is therefore belt-and-suspenders — no separate workflow *step* is required; the phase rides the existing `make lint` invocation. Harmless, not a defect.)

**Conclusion:** I-r1-1 closed. GREEN on item 1.

## 2. Fold drift / coherence — NONE found

- **§3 vs §4 vs §6 are complementary, not contradictory.** §3 (anchor namespace disjointness, gated by `gui-schema-coverage`) and §6 (cross-link resolution, gated by the new `gui-form-xref`) govern *different* invariants — no overlap to contradict. §4's "agree by construction" (single-source generation) and §6's "the gate that PROVES it" are the exact generator-plus-gate pairing round-1 recommended ("Recommended to do BOTH"): construction makes the two ends agree; the gate proves they didn't drift. That is defense-in-depth, not a self-contradiction.
- **The §3 "R0-r1 VERIFIED bulletproof" notes (lines 18, 23-era reasoning) match the live source** — `is_schema_shaped` is prefix-anchored (`anchor == shape or anchor.startswith(shape + "-")`@113), no tab is named `gui`, citations un-decayed. The fold did not perturb the settled anchor ruling.
- **The rest of the spec still coheres.** Placement (§2, `75-`/`85-` free choice), per-subcommand edits (§4 remove-fence-plus-lead-in, keep schema anchors + `.out` include), tour handling (§5, 2 inline renders kept), and `verify-examples-gui` (§6, placement-agnostic, untouched) are all unchanged by the fold and remain internally consistent. Per-tab counts (32/10/10/9 = 61) re-verified against `transcripts/gui/` — exact, no miscount.

## 3. What the gate's existence changes re: the tour — handled, invisible to the gate

The tour double-includes `mnemonic-bundle.gui` and `mk-inspect.gui` via `include="gui/…"`. The `gui-form-xref` gate keys exclusively on `{#gui-form-*}` anchors (gallery) and `](#gui-form-*)` links (subcommand chapters). The tour's includes match **neither** token pattern, so:
- each stem still has **exactly one** gallery anchor (the tour adds none — §5 states the inline renders carry no `gui-form-*` heading anchor), and
- each stem still has **exactly one** subcommand cross-link (the tour's `include=` is not a `](#gui-form-*)` link).

So the "exactly one anchor / exactly one cross-link per stem" counts are unaffected by the double-include; the only thing the double-include touches is `include-transcript.lua` file-embedding (no uniqueness constraint) and the census (file *existence*, not inclusion count) — both already settled in §5/N4. The prompt's reasoning is correct and the spec is consistent with it. **Minor M-r2-2** below notes the spec proves this only implicitly.

## 4. New Critical / Important — NONE

The fold is additive and introduced no new funds/secret/build surface. No new C/I.

---

## Minor / Nit (carry to plan-doc R0 — none blocks GREEN)

- **M-r2-1 — the new check needs the transcripts/gui path, which `lint.sh` does not currently receive.** `lint.sh` is invoked (Makefile@297-305) with `SRC_DIR / BUILD_DIR / TESTS_DIR / MANUAL_GUI_UPSTREAM_ROOT / *_BIN` only — no transcripts path. The canonical stem list lives at `transcripts/gui/*.gui` (Makefile already has `TRANSCRIPTS_GUI := $(TRANSCRIPTS)/gui`). The plan must either thread a new `TRANSCRIPTS_GUI=` arg into the `lint:` target + forward it through `lint.sh`, or have `check_gui_form_xref.py` derive it as `$SRC_DIR/../transcripts/gui` (deterministic sibling). Pure plumbing, data is available, trivially resolved at plan time — not a design defect.
- **M-r2-2 — make the tour/xref interaction explicit in the spec.** §5 explains the tour's no-collision only w.r.t. the *gui-schema-coverage* anchor gate; it does not explicitly state that the tour's double-`include=` adds neither a `gui-form-*` anchor nor a `](#gui-form-*)` link and therefore cannot perturb the new gate's exactly-one counts. The invariant holds regardless (verified in §3 above), but one sentence in §5 or §6 would close the reasoning gap for the plan author.
- **M-r2-3 — confirm the implementation keys on the FULL filename stem.** Several stems carry multi-hyphen subs (`mnemonic-xpub-search-account-of-descriptor`, `mnemonic-ms-shares-combine`, `mnemonic-seed-xor-split`, …). The spec writes `<tab>-<sub>`; the anchor/link must be `gui-form-<full-stem>` (everything after `gui-form-` == the `.gui` filename minus extension), exactly as round-1's `gui-form-<stem>` phrasing. State "key on the full `.gui` filename stem" in the plan so `<sub>` is never mis-split at the first hyphen.

---

## One-line summary

I-r1-1 is **fully closed**: single-source generation (§4) + a bidirectional fail-closed `gui-form-xref` gate (§6) that catches every enumerated cross-link failure mode — single-side typos double-caught (missing + orphan), enumerate-stems-then-exact-match as required, tour double-includes provably invisible to the count. No fold-introduced contradiction; anchor ruling and counts re-verified against live source. Three plumbing/clarity Minors to the plan-doc. **GREEN — 0 Critical / 0 Important. Converged; proceed to plan-doc R0.**
