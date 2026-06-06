# R0 Architect Review (round 3, final convergence) — `SPEC_descriptor_origin_extraction_dedup.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `descriptor-origin-extraction-dedup`. **Verdict:** **0 Critical / 0 Important / 0 Minor.** **GREEN — converged.**

> Persisted verbatim per CLAUDE.md. M4 fold accurate; adversarial re-read surfaced nothing new. Implementation may proceed.

---

### VERDICT: 0 Critical / 0 Important (0 Minor)

**GREEN — converged, implementation may proceed.**

The M4 fold is accurate against source, and an adversarial re-read surfaced no new Critical, Important, or Minor. The SPEC is implementable as written.

---

### What verified clean

**1. M4 fold accurate — full proof chain confirmed against source.**
- `parse_descriptor.rs:90-105` is the `origin_path_anno` block: calls `DerivationPath::from_str(&s)` at :101 inside the `captures_iter` loop (:75) over EVERY `@N[fp/path]` placeholder, `.transpose()?` (:105) propagating errors. Cited range exact.
- Path-string identity holds: `concrete_keys_to_placeholders` (pipeline.rs:106-147) copies the path verbatim at :146 (`placeholder_form.push_str(path)`), never `from_str`s it — confirming M4's distinction that M2's guard does NOT extend to paths.
- All four orderings verified — `parse_descriptor` (running `lex_placeholders`) precedes `build_slot_fields`: bsms 222→227→251, bitcoin_core 279→292, coldcard 321→334, electrum 380→395.
- electrum is the only lazy parser (electrum.rs:923 `captures_iter().nth(slot_idx)`), so the only lazy→eager-shift site; `parse_descriptor` (:380) runs unconditionally before it. Malformed path (index ≥ 2³¹) in a non-selected slot errors in `lex_placeholders` first → eager loop unreachable → behavior-preserving. The other 4 already loop-then-`.nth`.

**2. §1(iii) message-convergence enumeration airtight (directly verified).** bitcoin_core's fp-hex (:427-428) + path-parse (:433-435) carry NO `entry_idx`/`slot_idx` — byte-identical-modulo-prefix, reproducible via `format_name`. Per-entry/slot context lives ONLY in xpub-decode (:463, converges per M1, unreachable per M2) + out-of-range (:457, stays in wrapper per M3). Electrum corroborates.

**3. No new behavior delta on adversarial re-read.** Every converged message sits on a proven-unreachable/defensive path: xpub-decode (M2), eager path-parse (M4), empty-result (upstream "no keys found" pipeline.rs:159-163 fires first), fp-hex (8-hex regex guarantee). The only reachable change is the h-form widening — `key_regex` (pipeline.rs:40) is a true superset of apostrophe-only; capture-group structure (1/2/3) identical.

**4. 6/4/4 file sets, 3 preserved signatures, no-lockstep re-confirmed.** bsms `(body, slot_idx)`, bitcoin_core `(body, slot_idx, entry_idx)` (entry_idx message-only), coldcard `(body)` single-key all preserved as thin wrappers. No clap change → no schema_mirror/manual mirror; reuses `ImportWalletParse`; no sibling change.

---

### Gate decision
**0 Critical / 0 Important / 0 Minor. GREEN — converged. Implementation may proceed** to Phase 1 (RED h-form cell).
