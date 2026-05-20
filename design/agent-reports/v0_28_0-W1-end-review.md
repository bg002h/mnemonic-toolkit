# v0.28.0 Wave-1 end — architect R-W1-end review

**Reviewer:** Opus 4.7 via feature-dev:code-architect (lean-dispatch coverage round)
**Branch:** `release/v0.28.0` @ `091a313`
**Cycle scope:** Waves 0 + 1 (18 commits ahead of master)
**SPEC:** `design/SPEC_wallet_import_v0_28_0.md` (P0A scaffolding + P7A 4-line + P8A taproot refusal scaffold)
**Test surface:** 1974/0 pass; `clippy --all-targets -- -D warnings` clean
**Date:** 2026-05-19

---

## Per-LEAN-phase coverage

| Phase | LOC delta | Self-architect-review? | Spot-check verdict |
|---|---|---|---|
| P2-v2 Specter | merge `8548258` ~750 src + tests | YES — `P2{A,B,C}-v2-r0-review.md` GREEN | GREEN — sniff matches SPEC §11.1; accessor exhaustive over 8 variants. |
| P3-v2 Coldcard | merge `1304932` ~950 src + tests | **NO** — LEAN deferred to this review | GREEN — sniff matches SPEC §11.3 Q3-relaxed; dominant-BIP order matches §11.3.1. |
| P6-v2 Electrum | merge `2031609` ~1100 src + tests | **NO** — LEAN deferred | GREEN — `parse_multisig_wallet_type` faithful to upstream `electrum/util.py::multisig_type`. |
| P5-v2 Jade | merge `091a313` ~600 src + tests | **NO** — LEAN deferred | GREEN — single positive marker (top-level `multisig_file`); delegates to `coldcard_multisig::parse_text`. `unreachable!()` catch-all retired. |

All 4 LEAN-phase deltas spot-check clean against SPEC + downstream wiring.

## Critical: NONE
## Important (folded inline at end-of-Wave-1)

### I1. BSMS 4-line first-address mismatch implemented as WARNING (exit 0), drift from SPEC §10.2 step 4

**SPEC §10.2 step 4** locks: `Mismatch → ImportWalletParse exit 2`.

**Implementation at `wallet_import/bsms.rs:283-302`** emits stderr WARNING + continues parse (exit 0). Same cross-validation block used by 6-line shape (deliberately WARNING-only by v0.26.0 lock).

P7A R0 self-review acknowledged the deviation as "principle of least surprise" + 6-line symmetry. **Implementation is downstream-correct; SPEC text is the lagging artifact.**

**Fold (this commit):** patch SPEC §10.2 step 4 inline → "WARNING (exit 0); parse continues" with rationale citing v0.26.0 6-line precedent + BIP-129 §6 coordinator-output self-consistency intent.

### I2. SPEC §10.6 silent on cross-validation WARNING-vs-error semantic

**Fold (this commit):** add §10.6 bullet pinning WARNING-not-error as the v0.28.0 semantic; flag strict-mismatch-error as a candidate for a future cycle if user demand surfaces.

## Minor (deferred)

- M1: Jade sniff omits leading-whitespace trim (cosmetic; functional equivalence holds).
- M2: `cmd/import_wallet.rs:155-162` envelope doc-comment vestigial v0.26.0 wording. Tracked in W0-rint Minor M1/M2/M3 class. Defer.
- M3: `bsms.rs:264-266` comment written for 6-line context; refresh post-I1/I2 fold.
- M4: `wallet-import-format-mismatch-matrix-completion` — partial coverage; tracked in cycle-followups.
- M5: `bsms-import-taproot-refusal-parity` — pre-emit refusal closes most surface area; defer.

## Cross-PR interaction audit (all PASS)

- A. `sniff_format` 8-bool votes array: no two sniff signatures co-fire on real-vendor blobs. Contrived multi-match → `Ambiguous` exit 1. ✓
- B. `ImportProvenance` accessor exhaustiveness: 7 accessor matches × 8 variants = all exhaustive. ✓
- C. `cmd/import_wallet.rs` per-format dispatch — no silent fall-through to empty JSON. ✓
- D. `VENDOR_MARKER_KEYS` 13 entries byte-exact against SPEC §6.1.1. ✓
- E. `unreachable_patterns` lint silent post-P5A catch-all retirement. ✓

## SPEC §11.x source-vs-spec spot-checks (5/5 PASS)

1. §11.3.1 dominant-BIP order (bip86>bip84>bip49>bip44) — PASS
2. §11.4.1 5-row XFP truth table — PASS (byte-exact WARNING template)
3. §11.6 Electrum `wallet_type` regex (P6 in-phase correction) — PASS (live WebFetch confirms `(\d+)of(\d+)` start-anchored; toolkit's hand-rolled state-machine equivalent)
4. §11.5 Jade single-marker sniff — PASS (Q1 SeedQR-deferred lock honored)
5. §10.5 BSMS error template `"expected 2 or 6 lines"` → `"expected 2, 4, or 6 lines"` — PASS (7 sites updated; 0 residual occurrences)

## Cycle-followups audit

3 open items — all correctly deferred:
- `wallet-import-format-mismatch-matrix-completion` (v0.28+; benign fallthrough)
- `sparrow-taproot-descriptor-passthrough-import-support` (v0.29+; substantial second parse-path)
- `bsms-import-taproot-refusal-parity` (v0.28+; pre-emit closes most surface)

## Overall verdict

**GREEN. Recommend proceeding to Wave 2 (P11 cross-format matrix expansion).**

- 0 Critical findings.
- 2 Important — folded inline (SPEC §10.2/§10.6 patches).
- 5 Minor — deferred to docs-sweep / cycle-followups.
- All cross-PR interactions clean.
- All spot-check normatives PASS.
- 1974 tests / clippy clean on the integrated state.
- LEAN-pattern phases (P3/P5/P6) reviewed; no findings escaped per-sub-phase coverage.

**Wave 2 prerequisites confirmed:**
- 8/8 parsers wired end-to-end (sniff → parse → provenance → envelope)
- 6 canonicalize helpers wired
- ImportProvenance exhaustive over 8 variants
- VENDOR_MARKER_KEYS at 13-entry SPEC state
- `unreachable!()` catch-all retired
- 1974 tests / clippy clean

End of review.
