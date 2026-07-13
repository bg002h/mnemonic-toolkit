# CONVERGENCE R0 (round 2) — SPEC_verify_bundle_nonchunked_canonicalization.md

**Reviewer:** Fable architect (`model:"fable"`), dispatched 2026-07-12. Focused delta-only pass on the round-1 folds.
**Source SHA:** `de140a08` (HEAD == origin/master).
**Usage:** 19 tool-uses, ~377s, 65555 subagent tokens.

## Verdict: GREEN — 0 Critical / 0 Important / 3 Minor

All four round-1 folds (I-1, M-1, M-2, M-3) landed and are accurate to source, with one cosmetic exception (M-3 folded one of its two cite sites). Test #7 as re-specified is genuinely probative and BOTH offered constructions (non-standard use-site / retained-Fingerprints TLV) are constructible — construction (b) survives scrutiny and should NOT be struck (the reviewer specifically tried to break it: no fingerprint validator exists — `grep Fingerprint validate.rs` = 0 hits; `is_wallet_policy` keys on non-empty **pubkeys** only, so a fingerprints-only keyless card is not wallet-policy; non-empty fingerprints re-encode into the id via `encode.rs:123`). No contradiction, dangling reference, or new Critical/Important introduced.

## I-1 fold — LANDED CORRECTLY
- (i) §3.2b mechanism accurate: `cli_template_from_tree` matches `(tag,body)` only (`synthesize.rs:369-377`, Wpkh arm `:374`), takes `&tree` — `use_site_path` is structurally invisible; `encode_payload` writes use-site (`encode.rs:120`) → different `compute_md1_encoding_id` (`identity.rs:39-45`). Expected re-derived from card's own tree (`verify_bundle.rs:602,:687-695`).
- (ii) Test #7 probative (asserts `md1_template_match.passed==false`; a broken `md1_match=true` FAILS it) + both constructions constructible (all `Descriptor`/`TlvSection.fingerprints` fields pub, `encode_md1_string` re-exported `lib.rs:55`; under the 400-bit cap). Construction (b) additionally catches WDT-id over-relaxation (WDT-id excludes Fingerprints, `identity.rs:49-54`).
- (iii) wrong-seed row + §3.2b placement correct (`verify_bundle.rs:634-635` account-agnostic; exit-4 carried by `mk1_template_stub_bind` `:697-700,:712-721`).
- (iv) #7 vs #8 distinct (use-site/Fingerprints vs origin), both realizable.

## M-1 / M-2 / M-3 — LANDED (M-3 partial, see Minor 1)
- M-1: §3.1 "subset" + sh(wpkh) divergence cites accurate (`synthesize.rs:1120-1125,:1135`; `canonical_origin.rs:67-70`).
- M-2: §6.1 #1 shows `--mk1` + harness cite `cli_verify_bundle_md1_template.rs:88-91` line-exact.
- M-3: §0 OUT-2 updated to `:2508-2510`; §1 row still `:2509` (Minor 1).

## Minor findings (cosmetic — all one-line spec edits)
1. **M-3 second occurrence missed.** §1 table "Fall-through outcome (b)" still cites `:2509`; change to `:2508-2510` for consistency (`:2509` is factually the binding line within the `:2508-2510` site, so not wrong — consistency only).
2. **§3.2b cite under-inclusive.** "each enters `encode_payload` (`encode.rs:119-120`)" — Fingerprints TLV enters at `encode.rs:123` (`d.tlv.write`); cite should read `:119-123`.
3. **Test #7 wording looseness.** "only if Facet 2 actually compares ids" — the old byte-identity compare would also keep it green; the exact operative claim is the next clause (broken `md1_match=true` FAILS it). Optional tighten to "only if the compare is content-sensitive".

## New-defect scan — CLEAN
§3.2b ↔ §4 rows ↔ §5 INV-4 ↔ §6.3 #7/#8 mutually consistent; no new `--json` state (verdict routing `verify_bundle.rs:796-831` → `result ∈ {ok,mismatch}`); INV-4 form-only relaxation intact; no dangling references (new test name resolves everywhere; no old name survives — grepped `wrong_seed|doctored`).

## Proof of work
SPEC (all 183) + round-1 report (all 49); verify_bundle.rs :380-412/:580-610/:630-640/:680-725/:794-832/:2500-2515/:3108-3118; synthesize.rs :360-385/:1114-1145; decode.rs :95-200; validate.rs :215-265 (+grep Fingerprint=0); encode.rs :17-28/:40-55/:85-130/:170; tlv.rs :28/:123-129/:262-263/:431-444; identity.rs :39-45/:48-54; origin_path.rs :50-68; canonical_origin.rs :60-75; codex32.rs :20-30; lib.rs :45-65; cli_verify_bundle_md1_template.rs :80-100.
