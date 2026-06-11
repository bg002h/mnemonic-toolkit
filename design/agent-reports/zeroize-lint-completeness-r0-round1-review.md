# R0 Review — zeroize-lint source→declared completeness — ROUND 1

**Source SHA:** `438de94`. **Verdict: 🟡 YELLOW — 0 Critical / 1 Important / 2 Minor.**

## Audit classification — INDEPENDENTLY VERIFIED (the load-bearing input)
The architect independently read 4-5 files and confirmed: all **14 CANONICAL** files genuinely own secrets (no transient noise); all **5 allowlist** files correctly classified (3 crypto-internal where the consumer owns the plaintext; `nostr.rs` PASS-THROUGH; `secret_string.rs` PRIMITIVE) — no real owned-secret site wrongly allowlisted. **8 evidence substrings spot-checked → all match byte-exact.** Partition complete: 16 existing row-files + 14 new canonical + 5 allowlist = 35 (matches live grep). The prior-R0 I3 concern (verify_bundle/ms_shares blanket-allowlisted) is RESOLVED — they're PROMOTED to rows. `crate_root()` base (`Path::new(".")`) consistent for the new scan.

## Important (folded)
- **I1 — RED-proof under-specified.** The "OR predicate-level" escape hatch let an implementer skip exercising the real glob loop (prior R0 m3 required the loop). FOLDED: (a) mandate the real-loop RED-proof (dev-time allowlist-entry removal → scan REDs → restore), dropping the predicate-only option; (b) add a PERSISTENT glob-cardinality FLOOR (`>= 35` files) so a future broken glob (vacuous pass) is caught in CI — mirrors the `ZEROIZE_ROWS.len()` count-range guard.

## Minor (folded)
- **m1** — count arithmetic: 36 rows today (not ~35); 36+16=52; `18..=60` covers it. FOLDED.
- **m2** — renamed `NON_CANONICAL_SECRET_FILES` → `NON_ROW_SECRET_FILES` (clearer "scanned but deliberately not a row"). FOLDED.

## Scope
Test-only, no binary change. Coherent as ONE cycle (the scan can't pass without Part A's rows — the 14 new canonical files would fail). Bonus FOLLOWUP `addresses-restore-passphrase-not-zeroizing` correctly scoped out.
