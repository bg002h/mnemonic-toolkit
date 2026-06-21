# R0 REVIEW — cycle-11a GUI hygiene (M9 · L12 · L13) — Round 2

> NOTE: reconstructed from the round-2 reviewer's agent summary (full verbatim transcript tangled under high parallelism). The round-3 review (`cycle11a-spec-r0-round3-review.md`) is the authoritative confirmation of the fold below.

## VERDICT: **NOT GREEN — 0 Critical / 1 Important (I2)**

The round-1 I1 fold corrected the fabricated `parse_descriptor.rs:113-118` "supply exactly one" citation, but **introduced new drift**: the replacement rationale (a) cited `parse_descriptor.rs:289` "consumed as a key annotation" — also not an accurate description of the v0.60.0 mechanism — and (b) created a **version contradiction** between the §3.2 line-91 master-grammar narrative and the line-115 v0.60.0-pin conclusion (master REFUSES the double-origin form; v0.60.0 ACCEPTS it — the spec conflated the two).

### I2 (Important) — version-anchoring contradiction + residual inaccurate citation

The L12 "benign over-acceptance" argument must be anchored to a SPECIFIC toolkit version because the behavior DIFFERS by version:
- **At the pin (v0.60.0):** the lexer regex (`parse_descriptor.rs:69-70`) matches only the SUFFIX bracket; a leading `[fp]` is not matched and is silently skipped by `captures_iter` → v0.60.0 ACCEPTS the double-origin form (`canonical`, exit 0). The `:289` "key annotation" claim was inaccurate — the real mechanism is the suffix-only regex + skip.
- **At master / v0.62.0+:** the named-group lexer (`parse_descriptor.rs:98`, groups `pfx_fp`/`sfx_fp`) matches BOTH positions and explicitly REFUSES the double-origin ("supply exactly one", `parse_descriptor.rs:113-116`, shipped by H7 commit `36095b88`).

**Required:** rewrite §3.2 (line 115) + the new version-note (line 117) + D7 (line 219) + the FOLLOWUP slug (line 203) to the DUAL-VERSION rationale: v0.60.0 ACCEPTS (pin fine), v0.62.0+ REFUSES at parse (pin moot) — strictly safer either way; D7's NO-tighten decision survives both. Drop the `:289` citation.

## Disposition

Fold I2, persist, re-dispatch round 3 (the new source claims `:69-70` / `:98` / `:113-116` / `36095b88` are load-bearing protocol facts that MUST be verified against authoritative source per the project recon discipline). M9 / L13 confirmed SOUND in round 1 and untouched by this fold.
