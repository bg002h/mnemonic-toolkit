# cycle-6 review — LENS A: predicate funds-correctness + over/under-rejection

- **HEAD:** `1617aa77` (`feat(cycle6-timelock): P2+P3 — decay-ordering predicates + canon migration`)
- **Base:** `3fa2925b` (`origin/master`)
- **Worktree:** `/scratch/code/shibboleth/wt-cycle6`, branch `feature/cycle6-timelock-decay`
- **Date:** 2026-06-21
- **Lens:** predicate funds-correctness; hunt for (a) a mis-ordered config that still BUILDS (under-detection → funds-loss), (b) a legit decaying wallet now wrongly REFUSED (over-rejection → availability), (c) abs-floor monotone-safety.
- **Method:** read diff + source (archetype.rs, gate.rs, build_descriptor.rs, timelock_advisory.rs); built the bin; ran the FULL `cargo test -p mnemonic-toolkit` suite (0 failures); ran ~20 adversarial CLI probes across boundaries; mutation-tested all three predicate edges (each kill confirmed, mutations reverted); exhaustive standalone proof of the masked/clean handoff over the full 0..=0x410000 band.

---

## Critical

**NONE.**

## Important

**NONE.**

## Minor

### M1 — masked `--recovery-older` is refused with the WRONG (cosmetic) diagnostic message (not a funds bug; documented for clarity)

`archetype.rs:311-339`. `validate_params` (the decay predicate) runs BEFORE the gate (`build_descriptor.rs:278` → `return Ok(2)` on failure, short-circuiting the gate at `:287`). The decay-rel predicate calls `older_unit_value(n)` which reads ONLY bit-22 + low-16 and is well-defined for ANY `u32`, including a masked operand. Consequence: a *masked* `--recovery-older` whose low-16 value is small (e.g. `0x80000001` → `older_unit_value` = `(Blocks, 1)`; or `65537` = `0x10001` → `(Blocks, 1)`) is caught by the `v_r <= v_p` rel-ordering branch and refused with the message "tiers must unlock progressively later", rather than reaching the gate's masked-operand field diagnostic. Verified empirically:

```
--older 1000 --recovery-older 2147483649 (0x80000001) → exit 2, "requires --recovery-older (2147483649) > --older (1000): tiers must unlock progressively later"
--older 1000 --recovery-older 65537       (0x10001)    → exit 2, "...progressively later"
```

This is **cosmetic, not a funds bug**: the input is still REFUSED (exit 2), funds-safe. The message merely misattributes the cause (ordering vs masking). The symmetric case — masked `--older` (e.g. `0x80000001`, low-16=1) paired with a larger clean recovery — does NOT fire the rel branch (`v_r=2000 > v_p=1`), falls through, and is correctly caught by the gate's masked-operand diagnostic (verified: exit 2, schema_field "bit-31 disable flag is set"). So masked operands are refused on EITHER path; none builds. The diff's own comment (`archetype.rs:306-310`) acknowledges the gate handles masked operands "independently downstream"; the only inaccuracy is that for a small-low-16 masked *recovery* operand the rel branch pre-empts the gate with a less-precise message. No action required for funds-safety; optional polish would be to screen `older_consensus_masked(n).is_some()` before the ordering compare and defer to the gate, but that adds coupling for a cosmetic gain. **Recommend leaving as-is** (the predicate's job is fail-closed refusal; it achieves that).

---

## Hunt findings (per the 5 assigned probes)

### 1. `older_unit_value` correctness + masked/clean handoff — CORRECT

`timelock_advisory.rs:78-85`. `unit = (n & 0x0040_0000 != 0)` (bit-22, BIP-68 `SEQUENCE_LOCKTIME_TYPE_FLAG`), `value = (n & 0xFFFF) as u16` (low-16, `SEQUENCE_LOCKTIME_MASK`). Matches BIP-68 exactly. Bit-31 disable and >16-bit-value cases are NOT screened here by design — but they don't need to be:

- **The gate (`gate.rs:257-304`) refuses EVERY masked `older()` operand unconditionally** via `older_consensus_masked` (`Some` ⇒ field_diag), and `validate_params` either also refuses (M1 cosmetic case) or falls through to that gate. A masked operand therefore **cannot build on any path** — proven exhaustively: I scanned `n ∈ 0..=0x41_0000` (covers both clean bands + boundaries) and confirmed every non-masked `n` reconstructs exactly from `(unit, value)` (`recon = (bit22?0x400000:0) | value == n`) with `value != 0`, and the masked bands (`0x80000000+`, stray bit-16) are uniformly `masked == true`. The masked/clean handoff is sound: no masked operand slips past both gates, and for clean operands `older_unit_value` yields consensus's effective `(unit, value)`, so the ordering compare uses the correct effective durations.

### 2. D-decay-rel under-detection — NO mis-ordered same-unit config builds

The predicate fires for EVERY same-unit `recovery <= primary`. Verified empirically + by mutation:
- same-unit equal `2000/2000` → exit 2 ✓; `1000/1000` → exit 2 ✓
- boundary `recovery = primary + 1` (`1000/1001`) → **builds** (exit 0) ✓ (correct: strictly later)
- same-unit mis-order `2000/1000` → exit 2 ✓
- cross-unit pairs (both directions: `145`blk/`4194305`512s and `4194305`512s/`200`blk) → exit 2 (refused as un-orderable) ✓
- **Mutation kill:** changing `v_r <= v_p` → `v_r < v_p` flips `2000/2000` to a build → `preset_decay_ordering_violation_exit_2` goes RED (confirmed). The `<=` is load-bearing and tested.
- **Mutation note:** disabling the cross-unit branch (`if false && u_p != u_r`) does NOT make the headline repro (`145`blk/`4194305`512s, `v_r=1`) build — it falls into the `v_r <= v_p` branch (`1 <= 145`) and is still refused, just with the wrong message (the test asserts the "different units" substring, so it correctly goes RED). This is defense-in-depth: the value compare also catches that particular repro. The cross-unit branch IS load-bearing for the OTHER direction (e.g. `older 145`blk / `recovery 4194449` = `0x40_0091` = 512s value 145 — same low-16 value 145 but different unit; without the cross-unit branch `145 <= 145` would still catch it, but a 512s value strictly greater than the block value, e.g. `older 100`blk / `recovery 0x40_0065`=512s-value-101, would BUILD a cross-unit pair that is NOT real-time orderable). The cross-unit refusal is therefore the correct primary guard, not redundant.

### 3. D-decay-rel over-rejection — ACCEPTABLE (no real availability regression)

Refusing all cross-unit `older`/`recovery_older` pairs (decision R1) is the correct fail-closed choice: a block delay and a 512-second delay cannot be totally ordered offline without baking in a ~10-min block-interval assumption the tool must not assume. A user wanting `older(144 blocks)` + `recovery older(1 week in 512s units)` can trivially re-express both in one unit (or use `--spec`, the documented escape hatch). The off-by-one is correct: `recovery = primary + 1` builds, `recovery = primary` rejects (verified §2). Legitimate same-unit decaying wallets (both blocks `1000/2000`, both 512s `0x40_0001/0x40_0002`) build green — positive controls + canon golden all pass.

### 4. D-decay-abs monotone-safety — GENUINELY MONOTONE-SAFE (only ever false-NEGATIVE, never false-POSITIVE)

`archetype.rs:349-368`. BIP-65 split `is_height = after < 500_000_000`; height floor `900_000`, time floor `1_750_000_000`; STRICT `<`. Adversarial boundary sweep (all verified empirically):

| `after` | classified | result | correct? |
|---|---|---|---|
| `900000` (== height floor) | height | **builds** | ✓ strict `<`, not "past" |
| `899999` | height | refused | ✓ |
| `500000` (reported past) | height | refused | ✓ |
| `499999999` (just below threshold) | height | builds | ✓ (> 900k) |
| `500000000` (== threshold) | **time** | refused | ✓ BIP-65: `>= 500_000_000` is time; `900M < 1.75e9` floor ⇒ past |
| `500000001` | time | refused | ✓ (below time floor) |
| `1749999999` | time | refused | ✓ |
| `1750000000` (== time floor) | time | **builds** | ✓ strict `<` |
| `2000000000` (future time) | time | builds | ✓ |
| `4000000` (canon future height) | height | builds | ✓ |

The `is_height` split at `500_000_000` routes each value to the correct floor (matches BIP-65 `LOCKTIME_THRESHOLD` and the gate's own use at `gate.rs:1041`). The "legit future TIME locktime just above 500M but below the time floor" worry is moot: a Unix-time `after` a user sets today is necessarily `>= now ≈ 1.75e9`, which is `>= ABS_TIME_PAST_FLOOR (1_750_000_000 ≈ 2025-06-15)`, so it builds. Any value in `(500_000_000, 1_750_000_000)` IS genuinely past as a timestamp (e.g. `900_000_000` = 1998-07-09) and is correctly refused — this is exactly the R0-round-1 C1 trap the spec already closed (`900000000` is NOT a valid future control). The floors are static PAST values; they can only shrink the false-negative window over time (a borderline-recent-but-already-past locktime above the floor slips through) and can NEVER false-positive a legitimately-future locktime. **Monotone-safe: confirmed.** **Mutation kill:** `if false && past` → `preset_decay_past_after_height_refused_exit_2` goes RED (confirmed).

### 5. Completeness for the fixed decay shape — COMPLETE

`lower_decaying_multisig` (`archetype.rs`) emits exactly: tier1 `andor(multi(prim), older(N1), tier2)` → tier2 `andor(multi(recov), older(N2), tier3)` → tier3 `and_v(v:pk(final), after(T))`. Three tiers, fixed shape, no fourth tier or alternate arrangement. The two predicates cover (i) tier1↔tier2 relative ordering and (ii) tier3 absolute future-ness. The ONLY uncovered interaction is tier2(`older N2`, relative) ↔ tier3(`after T`, absolute) — which is genuinely un-orderable offline (different reference frames; ordering would require a per-UTXO confirmation-height assumption). The spec's argument that future-ness is the maximal sound offline-decidable invariant for tier-3 holds. No mis-ordering remains unchecked within the offline-decidable envelope.

---

## Migration / regression sanity (load-bearing for "no orphaned past `after`")

Full `cargo test -p mnemonic-toolkit` is GREEN (0 failures across all targets, incl. the in-crate `validate_params_decay_ordering` `diags.len()` count and the `repeated_keys` CLI path). Canon descriptor checksum regenerated (`#llvl05j9` → `#9fqrjy7e`) and parses. No orphaned `500000`/`500001` in any `validate_params`-reaching site (the two UNAFFECTED sites — `ir.rs` pure-render, `cli_compare_cost.rs` raw-miniscript — are correctly left as-is). Mutation discrimination test `preset_negative_discrimination_mutated_param_breaks_golden` migrated to `4000001` (future) so it still tests its intended "numeric mutation breaks golden", not the new abs reject.

---

## Verdict

**LENS-A PREDICATES: 0C / 0I — GREEN**

(One Minor M1, cosmetic-only diagnostic-message misattribution on a masked recovery operand that is still correctly refused; no action required for funds-safety.)
