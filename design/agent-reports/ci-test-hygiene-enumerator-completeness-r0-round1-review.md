# R0 Review — CI/test hygiene Cycle B (PolicyNode macro + zeroize completeness) — ROUND 1

**Source SHA:** `aa3e46d`. **Verdict: 🟡 YELLOW — 0 Critical / 3 Important / 3 Minor.** Part A macro mechanics sound; Part B's allowlist is under-audited. Recommendation: **ship Part A; re-spec Part B after the 19-file audit (split).**

## Critical
None.

## Important

**I1 — Part A: make the "delete manual `kind()`" instruction explicit (else E0201 duplicate-method).** The SPEC's "removed-from-impl" wording is ambiguous; a misread leaves both the manual `kind()` (`ir.rs:209-229`) AND the macro-generated `impl PolicyNode { fn kind }` → `error[E0201]: duplicate definitions with name 'kind'`. Fix: direct imperative — "Delete the manual `kind()` at `ir.rs:209-229`; the macro replaces it."

**I2 — Part A: do NOT remove the REMINDER match; `node_kinds_cover_enum` is NOT redundant.** The macro de-vacuifies via: new variant → macro `kind()` non-exhaustive (COMPILE error) → macro input grows → NODE_KINDS grows → `node_kinds_cover_enum` (samples==NODE_KINDS) FAILS at RUNTIME until the sample is added. The REMINDER match (`ir.rs:309-333`) currently forces the `all_variant_samples` SAMPLE visit at COMPILE time — removing it downgrades that to a runtime test failure. KEEP the REMINDER match (belt-and-suspenders) + KEEP the test. Don't frame either as redundant.

**I3 — Part B: the "19 transient files" classification is NOT done; spot-check shows real owned-secret sites.** `verify_bundle.rs` (`Zeroizing::new` on passphrase + entropy at :817/:820/:874/:879/:909/:934) and `ms_shares.rs` (14 sites: entropy, passphrase, shares) carry OWNED secrets — same shape as the canonical `bundle.rs` row — yet have NO ZEROIZE_ROW. Blanket-allowlisting them as "transient" would write a FALSE record (strictly worse than the status quo). Crypto-internal ones (`bsms_crypto`, `electrum_crypto`, `slip39/feistel`) ARE plausibly transient. **Before Part B can go green, each of the 19 must be classified (genuinely-transient / pass-through / MISSING-ROW), and the missing-row ones (≥ verify_bundle, ms_shares) promoted to canonical ZEROIZE_ROWS** (a real coverage improvement, not just a lint gate). This audit is the actual Part B work.

## Minor
- **m1** — note the precedent (`declare_node_type_variants!`, convert.rs:1767) generates a const VALUE array (unit variants); the new macro generates a `match self` METHOD (PolicyNode carries data) — structurally different, same forcing pattern.
- **m2** — removing the REMINDER match (if done) drops the `#[allow(clippy::match_like_matches_macro)]` user cleanly. (Moot if I2 keeps it.)
- **m3** — Part B RED-proof should test the real filesystem-scan loop, not just the predicate.

## Recommendation
**Adopt Part A (clean, self-contained de-vacuification). SPLIT Part B out** — re-spec it with the 19 files individually classified + missing rows added (verify_bundle/ms_shares appear to need them), then R0 round-2. An incorrectly-stamped allowlist is worse than the current count+evidence lint. Part A ships independently.

## Confirmations
- All 17 PolicyNode variants are single-data-field tuple variants → `PolicyNode::$variant(..)` matches each; macro `kind()` exhaustiveness chain is real.
- NO-BUMP test-only; no binary/wire/CLI/CHANGELOG.
