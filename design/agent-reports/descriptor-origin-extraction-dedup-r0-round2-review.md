# R0 Architect Review (round 2) — `SPEC_descriptor_origin_extraction_dedup.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `descriptor-origin-extraction-dedup`. **Verdict:** **0 Critical / 0 Important** (+ 1 new Minor M4). **GREEN.**

> Persisted verbatim per CLAUDE.md. Round-1 folds (M1/M2/M3) accurate, no drift. M4 (new) is a one-sentence documentary addition recording the eager-vs-lazy path-parse proof. Fold → final convergence round.

---

## VERDICT: 0 Critical / 0 Important (1 Minor)

**GREEN — implementation may proceed.**

The 3 round-1 folds (M1/M2/M3) are accurate against source and introduced no drift or contradiction. The core design holds. One new Minor surfaced from deeper item-4 analysis (an eager-vs-lazy evaluation-order subtlety), provably behavior-preserving and documentary-only — not blocking.

---

### Critical / Important
None / None.

### Minor

**M4 — §4 should record WHY the eager path-parse can't change behavior (parallel to M2's xpub-decode guard).**
The shared `extract_origin_components(body, format_name)` parses **every** capture's fp-hex + path eagerly, before selection. Five parsers already do this (loop-then-`.nth`); but **electrum** (`electrum.rs:912-953`) is lazy today — `captures_iter().nth(slot_idx)` first, then parses only the selected slot's path. fp-hex eager-parse is inert (regex guarantees 8 hex chars). But a regex-valid path can still fail `DerivationPath::from_str` (an index ≥ 2³¹, e.g. `/2147483648`, matches `(?:/\d+'?)+` but is rejected). So in principle a malformed path in a non-selected electrum slot could flip from silently-skipped to hard-error.

**Provably can't-happen, but NOT via M2's guard.** `concrete_keys_to_placeholders` only *copies* the path string (pipeline.rs:111/146) — it never calls `DerivationPath::from_str`, so M2's proof does not extend to paths. The real guard is **`parse_descriptor` → `lex_placeholders`** (parse_descriptor.rs:90-105), which runs BEFORE `build_slot_fields` in all 6 parsers (bsms 227→251, bitcoin_core 279→292, coldcard 321→334, electrum 380→395, + specter/sparrow) and calls `DerivationPath::from_str` on **every** `@N[fp/path]` placeholder over the identical path strings. Any malformed path errors there first, so the loop is never reached and the eager parse can never fire a new error.

Fix: add one sentence to §4 (parallel to M2): "The eager path-parse in the shared `extract_origin_components` is likewise unreachable — `parse_descriptor`/`lex_placeholders` (parse_descriptor.rs:90-105) `DerivationPath::from_str`-validates every slot's path BEFORE any `build_slot_fields`, so electrum's lazy→eager shift is behavior-preserving." Documentary; does not gate Phase 1.

---

### What verified clean

1. **All 3 folds accurate against source — zero drift.** M1: xpub-decode cites exact (`bitcoin_core.rs:463`, `electrum.rs:949`, `sparrow.rs:631`, `specter.rs:410`); coldcard single-key `xpub decode: {e}` (coldcard.rs:539) folds correctly. M2: `concrete_keys_to_placeholders` call-sites exact (bitcoin_core:267, bsms:222, sparrow:406, specter:224, coldcard:313, electrum:373); pre-decode pipeline.rs:116-121; `debug_assert_eq!` bitcoin_core:293 / pipeline:199. M3: out-of-range cites exact (bitcoin_core:457, bsms:407, sparrow:625, specter:404, electrum:925) — live in the `.nth().ok_or_else` step the wrapper retains.

2. **No internal contradiction.** §1/§2/§4/§7 agree. M1 (xpub-decode converges, in `finalize_slot_fields`) and M3 (out-of-range stays in wrapper, in `.nth()`) target different code locations — consistent.

3. **Core design undisturbed.** 6/4/4 file sets + lines re-confirmed; 3 signatures preserved; bitcoin_core `entry_idx` message-only; h-form superset (inline copies byte-identical apostrophe-only, key_regex differs only in path group); PATCH/no-lockstep; RED cell well-formed.

4. **Item-4 accounting now complete.** fp-hex (`fingerprint hex: {e}`) + path-parse (`derivation-path parse: {e}`) messages are byte-identical-modulo-prefix in all 6 parsers (bitcoin_core:428/435, bsms:377/384, specter:377/384, sparrow:597/604, coldcard:525/532, electrum:936/943), NO per-slot context → reproduced exactly via `format_name` (fall under §1 clause (i)). Only second-order concern (electrum eager-vs-lazy) is provably inert per M4. No other context losses.

5. **No test/manual pins.** Grep of `tests/` for convergent strings returns only the unrelated comment at `cli_xpub_search_account_of_descriptor.rs:328`. The manual's `--descriptor` hits (41-mnemonic.md:59,:530) describe `bundle`/`verify-bundle` (already h-form-accepting), make NO apostrophe-only claim about import parsers, already document "both apostrophe and h-form" → §6's manual-prose check satisfied, no edit needed.

---

### Gate decision
Fold M4 (§4 one sentence) → re-persist → final convergence round (documentary, single round expected). 0C/0I — gate is substantively GREEN.
