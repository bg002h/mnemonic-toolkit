# R0 Review — older() timelock mask gate — ROUND 2

**Source SHA:** `a7c1920` (verified == local HEAD) · **SPEC:** `design/SPEC_older_timelock_mask_gate.md` (folded) · **Round-1:** `design/agent-reports/older-timelock-mask-gate-r0-round1-review.md`
**Verdict:** 🟡 — **0 Critical / 1 Important / 1 Minor** (both NEW fold-induced; all six round-1 findings resolved — one more fold round needed, both fixes are one-sentence edits)

## Round-1 finding dispositions

- **I1: RESOLVED.** SPEC:36-44 branches `<CONSEQUENCE>` on `n & 0x8000_0000`. Consensus-correct per BIP-112: a CSV operand with the disable flag set makes CHECKSEQUENCEVERIFY a NOP — "no relative timelock at all" is the right wording, and it correctly dominates even when bit-22/value bits are also set (as in `0x80000090`). The else-branch masked-value arithmetic + bit-22 unit-suffix selection live only in the bit-31-CLEAR branch — correct (verified the suffix is consensus-accurate even for oddballs like `500000000` = bit-22 set → "(512-second units)", value 25856). Test plan (SPEC:53) adds `older(0x80000090)` asserting the no-op wording AND `older(65536)` asserting the "effective value" wording — both branches pinned. Branch logic is unambiguous for an implementer (explicit if/else on one bit test). *But see new M-A on the RED-proof parenthetical for this very cell.*
- **M2: RESOLVED.** SPEC:50 mandates one-timelock-per-tree + `DiagnosticKind::SchemaField` assertions; the accept cell says "each in its own single-timelock tree"; `rejects_after_above_max` puts after(1)/after(500000000)/after(0x7FFFFFFF) in own trees. I scanned every cell in §Tests — no remaining cell mixes height+time in one tree. Sufficient.
- **M3: RESOLVED.** Verified against `archetype.rs`: kofn-recovery's params include `p(OLDER, true, false, 1, ParamKind::Blocks)` (≈:165) with `const OLDER: &str = "--older"` (:96) — the flag IS `--older` (`--recovery-older` exists only on decaying-multisig/tiered-recovery). Provenance row `("root.or_d[1].and_v[1]", None, OLDER)` confirms attribution attaches. `--archetype kofn-recovery --older 105120` is the correct cell.
- **M4: RESOLVED.** Verified against source: `mod.rs:56-88` preset values 65535, **1000, 2000** (decaying-multisig), 52560, 4032, 144 — all now listed (SPEC:33); `older(4194305)` at `cli_compare_cost.rs:889` ✓ (intake, success-asserted, accepted by new predicate); `older(32768)` in intake fixtures ✓ (4 test files, all <65536). Enumeration now accurate; conclusion (zero false-reject) holds.
- **M1: RESOLVED-WITH-NOTE.** FOLLOWUP `archetype-older-blocks-flag-accepts-time-units` filed (SPEC:67) with the `validate_params` boundary rationale. **The deferral is defensible — do not escalate.** I verified `archetype.rs:711` `validate_params_does_not_duplicate_gate_rules`: the preset layer deliberately lets gate-class rules flow to the gate, and bounding `ParamKind::Blocks` is a genuine design call (it is NOT a gate rule — the gate legitimately accepts 512s-unit encodings; the "blocks" constraint is preset-semantic). The residual window (4194305..4259839 as a "block count" ≈ 80 years) is implausible-authoring-intent, near-nil exposure, and the FOLLOWUP captures the design question correctly. In-cycle code would breach the SPEC's own gate-boundary scoping for a near-nil win. NOTE: round-1 M1's "at minimum note the window in the manual prose" direction was folded into a sentence that overclaims — that is the new I-A below.
- **M5: RESOLVED.** FOLLOWUP `intake-surfaces-accept-masked-older-no-advisory` filed (SPEC:68) with the correct must-not-block-import rationale and advisory direction.

## Re-confirmations (cheap)
- Reject predicate unchanged at SPEC:19: `(*n & !0x0040_FFFFu32) != 0 || (*n & 0x0000_FFFFu32) == 0` ✓. Edge table unchanged ✓.
- PATCH v0.53.9 ✓; no schema_mirror/GUI/sibling lockstep ✓ (re-confirmed `schema.rs:31-32` `"older"`/`"after"` grammar still `uint`); CHANGELOG + 3 self-pin sites unchanged from round-1 verification ✓.
- Manual `:4009` domain-line target re-verified in source (`older` `1 ≤ N < 2³¹` prose present) ✓; `:3993` is the `--older` flag row ✓.

## Critical

None.

## Important

**I-A (NEW, fold-induced) — the `:3993` manual sentence claims a rejection the code will not do.** SPEC:63 instructs: add to the `--older` flag prose that values "exceeding 65535 are rejected (sets up the M1 FOLLOWUP)". That claim is **false for exactly the window M1 defers**: `--older 4200000` (`0x401640`, in `4194305..=4259839`) exceeds 65535 and is **accepted** by the new gate (valid 512s-unit encoding), silently reinterpreted as ~33.8 days. Since the SPEC defers the preset blocks-bound to the FOLLOWUP, this sentence would ship a factually wrong manual claim — the same "imprecise funds-safety prose" class as round-1 I1, and the inverse of round-1 M1's actual direction ("note the window"). **Fix (one sentence):** make the prose honest — e.g. "interpreted as **blocks**; malformed BIP-68 encodings are rejected, but valid 512-second-unit encodings (`0x400001`–`0x40FFFF`) are currently accepted and reinterpreted as time units (tracked: `archetype-older-blocks-flag-accepts-time-units`)" — or drop the >65535 claim entirely and let the `:4009` domain line carry the accuracy. Do NOT state a blanket >65535 rejection until M1 ships.

## Minor

**M-A (NEW, fold-induced) — the RED-proof parenthetical at SPEC:53 is inaccurate for the `0x80000090` cell.** "RED-proven (these produce no field diag before the fix)" is true for 65536/105120/0x400000 but FALSE for `older(0x80000090)`: the current arm (`gate.rs:245`, verified) already rejects `*n >= (1u32 << 31)` with a SchemaField diag today. That cell's RED component is the **bit-31 no-op wording assertion only** (the old message lacks it), not diag-existence. Reword so the implementer's scratch-revert check doesn't read the partially-green cell as a broken RED proof: "the first three produce no field diag before the fix; the `0x80000090` cell is RED via its wording assertion (the pre-fix message rejects it but with the old `1 ≤ N < 2^31` text)."

## Gate status

NOT GREEN — 0C/1I/1M. Both findings are single-sentence SPEC edits (I-A: reword the `:3993` manual instruction; M-A: reword one parenthetical). No code, predicate, test-cell, SemVer, or lockstep change required. Fold and re-dispatch for round 3; expect GREEN on sight if the two sentences land as directed.
