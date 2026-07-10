# R0 convergence review — BIP-prose folds (md1 + mk1), Phase 1, round 2 — Fable, adversarial, read-only

**Persisted verbatim per CLAUDE.md.** Convergence check of the folds applied after the round-1 per-phase reviews (`md1-bip-alignment-phase1-prose-r0-round-1.md` 0C/3I/8M; `mk1-…-round-1.md` 0C/1I/2M).

## md1 — fold verification
**IMP-1 (chunk-count determinism) — formula CORRECT (re-derived against `chunk.rs:236-291` term-by-term), fold INCOMPLETE.** Line 807's normative split matches code exactly: `count = ⌈padded_payload_bits/320⌉` ↔ `payload_bytes.len()*8 div_ceil SINGLE_STRING_PAYLOAD_BIT_LIMIT`; MUST-reject `>64` ↔ `ChunkCountExceedsMax`; `bytes_per_chunk = ⌈plen/count⌉` ↔ `div_ceil`; fragment `i = [i·bpc, min((i+1)·bpc, plen))`; leading full-size + short remainder. Edge verified: `bpc ≤ 40`, `(count−1)·bpc < plen`, final fragment always non-empty. "Capped at 64" clamp-reading gone. **BUT line 265 (chunk-header field table, bits 11–6) still reads "Encoders SHOULD use the smallest count that fits the payload"** — the phrase IMP-1 ordered deleted, surviving in the table a wire implementer codes from. Concrete divergence: `plen=90` → `count=3` per 807 vs `count=2` per smallest-fits — two encoders emit different strings. Fold-status note inaccurately claimed removal (removed only from 807). **Fix: delete/replace 265 with a cross-ref to §Chunking.**
**IMP-2 — folded ✓.** Line 651 branches (all-zero → clean end; non-zero → reject per §Payload `<PAD_ERROR_VARIANT_TBD_P2>`). No surviving "MUST be tolerated". Line 313 scoped to reassembled stream + per-chunk ≤4-bit pad discarded (closes Minor 6).
**IMP-3 — folded ✓, code-accurate.** Line 67 states per-@i divergence + on-card xpubs (TLV 0x00-0x03) supported; only standalone Xpub Card deferred. Cross-checked `tlv.rs:11-19` (exact tag set) + divergent flag bit 4. No surviving "per-placeholder…deferred"; line-51 Versioning cite fixed.
**Minors — verified.** 284 version bits correct; no "Tier 0/1/3"; preamble no self-suggested numbers; pad-MUST scoped; 327 §Backwards-Compat cite resolves; Minor 7 tracked; 8b skipped.

## mk1 — fold verification
**I-1 — folded ✓, independently fact-checked.** Line 130 correct: `1` = generic BIP-173 start + ms1's seed (hrp_expand("ms") prepend); `0x23181b3` IS BIP-93's ms32_polymod init verbatim. Re-derived: folding `hrp_expand("ms")=[3,3,0,13,19]` into residue from `1` → `0x23181b3` (0x23→0x463→0x8c60→0x118c0d→0x23181b3). Line-120↔130 contradiction gone.
**M-a ✓** (120 code-variant parenthetical). **M-b ✓** (655 per string). **T_LONG ✓** (112 "was MD's T_LONG before MD retired its long code").

## Drift / cross-BIP / funds-safety
No new contradiction (807↔805/829/831 consistent; 320-below-360 arithmetic checks: 400−37=363→45 bytes=360). Cross-BIP contrast intact (md1:831 ↔ mk1:364, both byte-aligned, mk1 fixed-53, md1 variable near-equal deterministic). Checksum init 0x23181b3 shared. Recovery-independence holds EXCEPT line-265's stray SHOULD re-muddies encoder policy. The 265 residue is not funds-Critical (reassembly accepts the division; CSI verifies over assembled payload) — damage is reproducibility/internal-consistency, same Important class as the original.

## Findings
**IMPORTANT (1): md1:265** — "smallest count that fits" survives, contradicts the normative formula at 807 (90-byte → 3 vs 2), undermines "identical strings" at 274/807. One-sentence fix.
**MINOR (process, non-gating):** correct the round-1 fold-status note (claims a removal that didn't fully land).

**VERDICT: NOT GREEN — 0 Critical / 1 Important.** All other folds complete/correct/drift-free; 807 formula bit-exact to `chunk.rs`. One-line edit from GREEN.

---
**FOLD STATUS (opus, 2026-07-10):** md1:265 folded (replaced with "count is fixed by the normative sizing formula in §Chunking (count = ⌈padded_payload_bits/320⌉); encoders MUST NOT vary it"). Round-1 fold-status note corrected. Re-dispatched a scoped final convergence check → **round-3 final check GREEN (0C/0I)**: line 265 defers to the formula, both BIPs grep-clean of chunk-count discretion (the two "smallest" hits are the varint-minimality MUST + the fingerprint-flag SHOULD, both out-of-scope), no collateral drift. **Phase 1 (BIP prose) CONVERGED — both BIPs R0-GREEN.**
