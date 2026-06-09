<!-- VERBATIM opus-architect R0 review, round 7, descriptor-builder engine (Release A, v0.50.0). Persisted per CLAUDE.md. SPEC + Phase-1 impl @ post-r6-fold; source b596d3f. Verdict: GREEN 0C/0I — Phase 1 cleared, Phase 2 may begin. -->

# R0 REVIEW (round 7) — descriptor-builder engine — **GREEN**

**GREEN. 0 Critical / 0 Important / 0 Minor.** The round-6 I2 target (`schema.rs:81-87`) and the `ir.rs:332` softening are both now honest. My own exhaustive grep across all four source files plus the SPEC found NO fifth location of the enum-coverage overclaim species — every remaining hit is honest drift/partial/author-discipline framing, true serde/miniscript/fixture-freeze concept, or superseded fold-log audit trail. The enum-coverage overclaim that resurfaced across rounds 3-6 is fully closed.

## CRITICAL
None.

## IMPORTANT
None. **I1 (the enum-coverage overclaim, all instances) and I2 (`schema.rs:81-84`, round-6's fourth location) are CLOSED.**

- `schema.rs:82-83` now reads "`ir::tests::node_kinds_cover_enum` asserts the hand-maintained `all_variant_samples` tag set == NODE_KINDS" — correct samples→NODE_KINDS attribution, not the false "enum == NODE_KINDS" word-swap. The "completing the … chain (risk #2 freeze)" airtight claim is gone, replaced by "a partial (drift-only) freeze. Neither catches a variant jointly omitted from all three." Honest.
- `ir.rs:332-334` reads "Exhaustiveness REMINDER … does not by itself add the variant to `samples` (see fn doc)" — consistent with `ir.rs:303-310` and `ir.rs:359-365` and SPEC:103. Honest.

## MINOR
None.

## What passes

### Grep-classification table (exhaustive sweep, all 4 source files + SPEC)
Patterns: `freeze complet chain link direction guard airtight cannot silently ensures impossible forces absent miss enum== ==NODE_KINDS ==grammar synced "in sync" guaranteed always never NODE_GRAMMAR cover exhaustive discipline drift partial`.

Key classifications (a=LIVE overclaim/Important, b=honest de-claim, c=true serde/miniscript/fixture concept, d=superseded history):
- `ir.rs:29` "cross-checked against `kind()`'s sample outputs" — (b) names sample outputs, not enum.
- `ir.rs:31-34` "catch a *drift* … cannot by themselves force … airtight … needs a macro … author discipline" — (b) de-claim.
- `ir.rs:208` "Must stay in sync with NODE_KINDS" — (b) author-obligation imperative, not a guarantee.
- `ir.rs:303-310` "forces an author to visit … does NOT force the new variant into the `samples` vec … author discipline" — (b) canonical honest framing.
- `ir.rs:332-334` "Exhaustiveness REMINDER … does not by itself add the variant" — (b) round-6 softening.
- `ir.rs:359-365` "one link of the freeze … does NOT catch a variant omitted from all three … macro" — (b).
- **`fn node_kinds_cover_enum` NAME** — (b) **consciously accepted, NOT renamed.** The name alone reads as an airtightness claim (the "fifth location" a careless round-8 reaches for), but the docstring at `ir.rs:359-365` explicitly de-claims it. Prose de-claim is the sanctioned route (rounds 4-6); a rename is a code change that re-opens test/clippy verification and risks fresh drift — manufacturing the round 8 we are preventing. On record as accepted. **Do NOT rename.**
- `schema.rs:81-87` "Companion cross-check … `all_variant_samples` tag set == NODE_KINDS … partial (drift-only) freeze … airtight guard would need a macro" — (b) round-6 I2 fix.
- `mod.rs:20` "freezes the schema against reality" — (c) fixture-purpose.
- SPEC:6/98/100/114 "freeze the schema" — (c) versioned-contract/fixture-freeze concept.
- SPEC:33 "serde enforces"; :57 "'wrong timelock loses money' guard"; :80 "multipath guard" — (c) true behavior.
- SPEC:103 "catching *drift* … neither catches a variant jointly omitted … author discipline … airtight … macro … risk #2 partial freeze" — (b) round-5 I1 fix.
- SPEC:124-125 "residual drift surface is miniscript's Display" / "freeze + gate it" — (c) canonical-form-stability + versioned-contract concept.
- SPEC:147 "forcing it into NODE_KINDS" — (d) superseded round-3 history, corrected by :151/:153/:155.

**No category (a) — zero live present-tense overclaims about the enum-coverage guarantee — anywhere in the four files or the SPEC.**

### Other Phase-1 surface checks
- **Test count = 20**, comment-only since round-4 green (13 ir.rs + 2 schema.rs + 5 mod.rs fixtures). I2/r6 edits are pure comment changes (not compiled) → "20 green / clippy clean" unchanged by construction. (No Bash tool to re-execute; basis = comment-only delta, same as round 6.)
- **`NODE_KINDS` == `kind()` == `NODE_GRAMMAR` == `all_variant_samples`** hand-verified at 17 each, matching order; two self-tests gate the drift. v1 covers all 5 archetypes + `or_b`/`hash160`/`hash256`/`ripemd160` unit cells.
- **SPEC internally consistent** (§1/§3/§3.4/§4/§6/§7 cross-reference cleanly; F1 two-gate split, §3.4 keystone, multipath-projection glue coherent).
- **Phase-1→Phase-2 boundary clean.** `PolicyNode::children()` exhaustive over all 17 variants (leaves→empty, binary→2, andor→3, thresh→subs, wrap→sub) — the §3.4 deepest-first walk substrate. `SpecParseError` (`Json` + `UnsupportedVersion`, with `Display`+`Error`) is the Phase-2 failure channel. Both intentionally unused-by-`main` until Phase 3 (`#![allow(dead_code)]`).

---

**Phase 1 is fully cleared.** I1 and I2 CLOSED. The complete Phase-1 surface — SPEC + `ir.rs` + `schema.rs` + `mod.rs` + the 5 fixtures (`simple-timelocked-inheritance`, `decaying-multisig`, `kofn-recovery`, `tiered-recovery`, `hashlock-gated`) — is GREEN at 0C/0I. **Phase 2 (the validation gate) may begin.**

Non-blocking note on record: the test NAME `node_kinds_cover_enum` is de-claimed by its docstring (table category (b)); do NOT rename it (a rename is a code change re-introducing verification risk).
