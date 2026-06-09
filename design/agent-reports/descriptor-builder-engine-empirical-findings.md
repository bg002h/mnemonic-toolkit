<!-- Empirical probe findings (throwaway tests, since deleted) against toolkit-pinned miniscript git rev 95fdd1c. Drives Phase-1 SPEC corrections feeding R0 round 3. Captured 2026-06-09. -->

# descriptor-builder engine — empirical findings (miniscript `95fdd1c`)

Three throwaway probe tests (run against the toolkit's pinned miniscript, then deleted) established the following load-bearing facts. They correct a false-proxy in the round-1 fold and redesign archetype #4.

## F1 — `Descriptor::from_str` is LENIENT on the funds-footgun sanity rules; only explicit `sanity_check()` catches them

Characterization (`wsh(<inner>)` via `Descriptor::<DescriptorPublicKey>::from_str` vs `Miniscript::from_str_ext(insane)` + `.sanity_check()`):

| Defect | `Descriptor::from_str` | explicit `sanity_check()` |
|---|---|---|
| Type error (missing `v:` on `and_v` left) | **REJECTS** (typecheck: "cannot accept children of types B and B") | also rejects (insane parse does NOT relax the `Base::B` typecheck) |
| Sigless branch (`or_d(pk,after)`) | **OK — lenient** | `Err(SiglessBranch)` |
| Mixed timelock (`and_v(v:after(h),and_v(v:after(t),pk))`) | **OK — lenient** | `Err(HeightTimelockCombination)` |
| Repeated keys (`or_b(pk(A),s:pk(A))`) | **OK — lenient** | `Err(RepeatedPubkeys)` |
| Good policy | OK | PASS |

**Consequences for the SPEC/implementation:**
- The round-1 fold used "`Descriptor::from_str` OK" as the empirical proxy for "passes the gate / archetypes sane-parse." That proxy is **FALSE for the funds-footgun dimensions** (it caught the type error but is lenient on sigless/mixed-timelock/repeated-keys).
- This **validates** the SPEC's two-separate-gates design (§3 step 2 type-check ≠ step 3 sanity_check). **Step 3 (explicit `sanity_check()`) is the sole funds-footgun gate; never collapse it into step 2, and never use `from_str`-OK as the safety oracle** (Phase-2 tests must assert against `.sanity_check()`).
- For §3.4 localization: a **type error fails even `from_str_ext(insane)`** (the `Base::B` typecheck is structural, not relaxed by `ExtParams::insane()`), so step-2 localization = "minimal subtree that fails to parse under insane." Funds-footgun localization = "subtree parses under insane but the matching predicate fails."

## F2 — archetype #4 "degrading-threshold" (same-key) is `RepeatedPubkeys` → un-emittable under cut-`--allow`; redesigned to "tiered-recovery" (distinct keys)

`or_d(multi(3,A,B,C),and_v(v:multi(2,A,B,C),older(N)))` reuses A,B,C → `sanity_check = Err(RepeatedPubkeys)`. A *true* same-key degrading threshold is an "insane" miniscript (`repeated_pk`), only emittable via the deferred `--allow`/`ExtParams::repeated_pk` (§3.5) or the raw `--descriptor` door. So archetype #4 as conceived is NOT a Release-A deliverable.

**Replacement archetype #4 — "tiered-recovery"** (distinct keys, sanity-PASS, and exercises the otherwise-uncovered `sortedmulti` + `or_i` + `thresh` fragments):

```
or_i(sortedmulti(2,A,B), and_v(v:older(4032), thresh(2,pk(C),s:pk(D),s:pk(E))))
```

= sortedmulti(2-of-2) primary, OR after a relative timelock a 2-of-3 `thresh` recovery quorum. `sanity_check = PASS`.

## F3 — final 5 archetypes, oracle = explicit `sanity_check()` (all PASS, distinct keys)

1. `or_d(pk(A),and_v(v:pkh(B),older(65535)))` — PASS
2. `andor(multi(2,A,B),older(1000),andor(multi(2,C,D),older(2000),and_v(v:pk(E),after(500000))))` — PASS
3. `or_d(multi(2,A,B,C),and_v(v:pk(D),older(52560)))` — PASS
4. `or_i(sortedmulti(2,A,B),and_v(v:older(4032),thresh(2,pk(C),s:pk(D),s:pk(E))))` — PASS
5. `andor(pk(A),sha256(H),and_v(v:pk(B),older(144)))` — PASS

aux (or_b render-cell, not an archetype): `or_b(pk(A),s:pk(B))` — PASS.

**C3 claim correction:** "all 5 archetypes sane-parse → cut `--allow` blocks nothing" becomes "all 5 *final* (distinct-key) archetypes pass explicit `sanity_check()`." Cutting `--allow` DOES block same-key degrading thresholds (an insane shape) — which is correct (miniscript deems it unsafe); the escape hatch is raw `--descriptor` and the deferred `--allow` (§3.5).
