# R0 review — SPEC_followup_toolkit_v0860_demote.md (round 1) — Fable, adversarial

**Persisted per CLAUDE.md.** Verified vs live master 14127582 (v0.85.0). Engineering claims VERIFIED (bless sites, kind-generic exit mapping, exactly-2 test flips [no third — 19 .code(5) sites swept], clean corpus, complete ritual, MINOR correct, manual-gui deferral sound). 4 Importants — all scope-precision on funds-adjacent #5.

## Important
**I1 — predicate under-specified; "single-string md1" conflates 2 forms with OPPOSITE oracle status.** Non-chunked (chunked_flag=0; v0.35 bypass `chunk.rs:615-631` SKIPS content-id → no oracle → demote) vs chunked-of-1 (flag=1,count=1; goes through `reassemble` → RETAINS ~2⁻²⁰ content-id → has oracle → stays Blessed). `chunk::split` emits count=1 ≤320 bits; toolkit `bundle` emits md1 via split (`synthesize.rs:392…`) → a template-form bundle card is a real chunked-of-1 WITH the oracle. The naive count==1 proxy OVER-demotes it. Fix: predicate = NON-CHUNKED only + a chunked-of-1 boundary test. **[FOLDED.]**
**I2 — reference-manual loci WRONG:** `docs/manual/src/40-cli-reference/*-repair.md` = nothing (those are manual-gui). Real sites: `41-mnemonic.md:3083/:3086/:3092-3097/:3154/:3188-3191` + auto-fire tables `:750/:771/:856-860`. Need a demotion subsection + anchor. **[FOLDED.]**
**I3 — sibling md-cli divergence undecided + missing CLAUDE.md companion:** post-demote `mnemonic repair --md1`→4 while `md repair`→5 (`42-md.md:334-353` D26 rule + golden). Qualify 42-md prose WITHOUT flipping md-cli behavior; file cross-repo FOLLOWUP mirrored in both repos. **[FOLDED.]**
**I4 — auto-fire advisory parity dropped:** `repair.rs:1741` Ms1-only → a correctable non-chunked md1 in convert/inspect/xpub-search/verify-bundle silently falls through (Cycle-F silent-invisibility). Widen to (Ms1 OR non-chunked-Md1). **[FOLDED.]**

## Minor (all folded)
M1 bless-site is `repair.rs:1599` not the doc `:451-453`; rewrite doc blocks `:443-462`/`:470-475`. M2 sweep stale comments `cli_mk1_repair_reverify.rs:679-685` + `prop_repair_never_wrong.rs:231-233`. M3 mk1 SingleString arm encoder-unreachable → defensive/untested (hand-construct or document). M4 mk1 own reason string. M5 pin JSON `verdict:"candidate"`.

## VERDICT: OPEN (0C/4I/5M). Engineering sound; scope-precision on #5. Riders #1/#2/#3 GREEN.

---
**FOLD STATUS (opus, 2026-07-11):** I1 (non-chunked-only predicate + chunked-of-1 boundary test), I2 (41-mnemonic.md loci + demotion subsection), I3 (42-md.md qualify + cross-repo companion FOLLOWUP), I4 (advisory widen), M1-M5 all folded. Acceptance #1 updated. Convergence R0 re-dispatched.
