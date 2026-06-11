# R0 Review — cell-27 temp race + anti-collision terminology — ROUND 1

**Source SHA:** `be1a581`. **Verdict: 🟡 YELLOW — 0 Critical / 2 Important / 3 Minor.**

## Part B GROUND TRUTH (architect-verified, definitive)
The invariant "leading 16 bits of chunk_set_id agree across all 3 cards" is **TRUE for the ENGRAVED identifiers**, FALSE for the WIRE chunk_set_ids — so "clarify, don't reverse" is CORRECT. Evidence: `bundle.rs::build_unified_card:1071-1078` md1 engraved = `compute_wallet_policy_id[0..2]`; `synthesize.rs:259-297` mk1/ms1 = `derive_mk1_chunk_set_id_for_slot(policy_id[0..4], slot)` (same root) → leading-16 agree. `21-md1-wire-format.md:50` md1 WIRE csi = `Md1EncodingId` (bytecode SHA-256) = different root. `chunk_set_id_extract` is MK1-only (`41:138`).

## Important (folded)
- **I1** — Part B: `42:19` had a dangling "see FOLLOWUPS" forward-ref; the SPEC said "promote to definitive" without the prose. FOLDED: added the verbatim replacement statement (engraved = policy-id = agrees; md1 wire csi = Md1EncodingId = separate, serves chunk-grouping not binding).
- **I2** — CHANGELOG citation WRONG: `:1899` is an unrelated BIP-86 line; the real §IV.2 anti-collision line is `CHANGELOG.md:2009` ("leading 16 bits agree across all three cards"). It's TRUE for engraved → add a clarifying parenthetical, not a reword. FOLDED.

## Minor (folded)
- **m1** — note `Ordering::Relaxed` sufficiency (distinct values, no cross-thread ordering). FOLDED (code comment).
- **m2** — `41:193` cites `bundle.rs::build_unified_card` twice → collapse to one anchor. FOLDED into the site list.
- **m3** — `glossary.md:57` "`chunk_set_id` cross-prefix agreement" also unqualified → add "engraved". FOLDED.

## Part A confirmations
`write_temp_json:412-419` uses `std::process::id()` only; 5 callers share the path → torn-content race. Atomic-counter fix correct + sufficient (distinct per call; pid disambiguates across processes); callers don't rely on a stable path; temp non-cleanup out of scope. NO-BUMP test+docs; no CHANGELOG version entry; bundling test+doc fixes coherent.
