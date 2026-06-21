# cycle-7 PLAN — R0 review, round 2

**Plan-doc:** `design/IMPLEMENTATION_PLAN_cycle7_m8_build_descriptor.md` (M8 build-descriptor extra-derivation-suffix → silent wrong subtree; + L23 ecies zero-scalar panic)
**Implements (R0-GREEN spec):** `design/BRAINSTORM_cycle7_m8_build_descriptor.md` (spec decisions D1–D16 not re-litigated).
**Round-1 review (the finding folded):** `design/agent-reports/cycle7-plan-r0-round1-review.md` — round 1 = **0C / 1I (RED)**; the single Important was **I-1** (the M8 preset reject flag-annotation: false universal-`--key` claim + a single-key-only T1b blind to its own counter-example).
**Reviewed against SHA:** `d6398b57` (toolkit 0.64.0; current `origin/master`). Cycle-7 code zone (`descriptor_builder/`, `electrum_crypto.rs`) byte-stable; all source citations re-verified live. Miniscript pinned 13.0.0.
**Reviewer:** opus software architect (adversarial R0; HARD gate — no code until 0C/0I).
**Date:** 2026-06-21
**Scope of this round:** verify the I-1 fold resolves the finding and introduced no new drift. The funds core was 0-Critical and sound in round 1; re-confirmed unchanged here.

---

## Verification log (independent, against `d6398b57` + empirical re-derivation)

### I-1 fold — the quorum→`--threshold` claim, re-derived from scratch (NOT taken on the plan's word)

I extracted the live `resolve_flag` (`archetype.rs:395-413` @ `d6398b57`) and the live provenance tables for all five archetypes, and re-implemented the tiebreak as a standalone program (`/tmp/probe_resolve_flag.rs`) fed the exact node_paths the M8 reject produces. The tiebreak is `max_by_key((prefix.len(), k.is_some()))` after `filter(boundary_ok && k.map_or(true, |k| k == kind))`.

**Mechanical re-derivation** (the M8 reject reuses `field_diag` ⇒ `kind = SchemaField`; for a `Multi`/`Sortedmulti` key the node_path is `root.<quorum>[0].{multi|sortedmulti}.keys[i]`, built at `gate.rs:240-244` via `node.kind()` = `"multi"`/`"sortedmulti"` per `ir.rs:62-63`):

| archetype | quorum node | M8 node_path (kind=SchemaField) | matching prov entries (same prefix len) | tiebreak winner |
|---|---|---|---|---|
| kofn-recovery | `OrD`→Multi | `root.or_d[0].multi.keys[0]` | `(…or_d[0], Some(SchemaField), THRESHOLD)` **vs** `(…or_d[0], None, KEY)` | `k.is_some()=true` wins → **`--threshold`** |
| tiered-recovery | `OrI`→Sortedmulti | `root.or_i[0].sortedmulti.keys[0]` | `(…or_i[0], Some(SchemaField), THRESHOLD)` vs `(…or_i[0], None, KEY)` | → **`--threshold`** |
| decaying-multisig | `Andor`→Multi | `root.andor[0].multi.keys[0]` | `(…andor[0], Some(SchemaField), THRESHOLD)` vs `(…andor[0], None, KEY)` | → **`--threshold`** |
| hashlock-gated | `Andor`→Pk (bare node) | `root.andor[0]` | only `(…andor[0], None, KEY)` | → **`--key`** |
| simple-timelocked | `OrD`→Pk (bare node) | `root.or_d[0]` | only `(…or_d[0], None, KEY)` | → **`--key`** |

**Empirical confirmation (standalone binary, live tables):**
```
=== M8 reject (SchemaField) at the primary-quorum key node ===
kofn   root.or_d[0].multi.keys[0]        SchemaField -> Some("--threshold")
tiered root.or_i[0].sortedmulti.keys[0]  SchemaField -> Some("--threshold")
decay  root.andor[0].multi.keys[0]       SchemaField -> Some("--threshold")
=== M8 reject (SchemaField) at single-key node ===
hashlock root.andor[0]  SchemaField -> Some("--key")
simple   root.or_d[0]   SchemaField -> Some("--key")
=== compare: xprv reject (SecretKey) at quorum key node ===
kofn   root.or_d[0].multi.keys[0]  SecretKey -> Some("--key")   ← the existing precedent
```

The quorum→`--threshold` claim is **EMPIRICALLY TRUE**. The mechanism the plan states is exactly right: at equal prefix length the `Some(SchemaField)` quorum override beats the `None` catch-all on the `k.is_some()` tiebreaker; the **existing** xprv reject escapes because it carries `kind: SecretKey`, which the override's kind-filter (`k.map_or(true, |k| k==kind)`) excludes — so its catch-all `--key` wins. Independent of my probe, this exact behavior is ALREADY PINNED by a live test (`archetype.rs:747-749` asserts `resolve_flag(kofn, "root.or_d[0]", SchemaField) == Some("--threshold")`; `:756-758` asserts the `keys[i]`-prefix `SecretKey → --key` precedent). So the fold's claim is not novel — it restates already-tested behavior. The L13 citation's added `(SecretKey→--key) test :757-758` reference is accurate (the `assert_eq!` body line).

### I-1 fold checklist (the three asks)

1. **States the per-archetype reality correctly — YES.**
   - T1b (line 82): "single-key archetypes → `flag=--key`; quorum archetypes (kofn-recovery / tiered-recovery / decaying-multisig) → `flag=--threshold`", with the correct mechanism (the `Some(SchemaField)` override wins the tiebreak; the xprv reject escapes via `SecretKey`). Explicitly framed as **"a provenance-system artifact, NOT a defect"** — the diagnostic `path` + `message` correctly name the offending key, and the exit-2 refusal fires regardless of `flag` ⇒ funds-safety unaffected.
   - §P1 prose (line 102): now reads "**`--key` for single-key archetypes, `--threshold` for quorum archetypes**" — the false universal-`--key` claim is GONE.
   - Grep of the whole plan for residual unqualified `--key` annotation claims: NONE survive. Every annotation statement is now per-archetype-qualified.

2. **T1b now covers a QUORUM archetype, non-vacuously — YES.** T1b's "Assert BOTH" instruction is explicit: "a single-sig preset → `flag=--key`; a quorum preset (e.g. decaying-multisig) → `flag=--threshold`", and adds the guard rationale "the test must cover a quorum archetype, not just single-key, **else it masks the real behavior**." The quorum assertion targets `--threshold` (the value that a single-key-only test would never exercise), so it is non-vacuous for the quorum case — it would FAIL if the code resolved `--key` (the round-1 mistaken belief). This directly closes the round-1 "test blind to its own counter-example" defect.

3. **No new drift — CONFIRMED.** The fold touched ONLY: T1b row, §P1 line-102 prose, and the L13 citation (added `(SecretKey→--key) test :757-758`). The funds core is untouched and re-verified sound:
   - **Guard predicate** (line 96): `key_part = key.rsplit(']').next().unwrap_or(key)` then reject on `key_part.contains('/')` — `check_secret_key` live at `gate.rs:347`, `key_part` line `:348`. The `contains('/')` post-`]` is exactly the M8 class (only legitimate `/`-bearing token is `[origin]`, stripped) — no over-rejection; T3 positive control pins it.
   - **Recursion / both intakes:** recursion driver `gate.rs:333` `for (cpath, child) in child_paths(node, path) { validate_fields(child, &cpath, out); }` confirmed; `child_paths` (`:646`) covers AndV/OrD/OrI/OrB/Andor/Thresh/Wrap. Both preset (`build_descriptor.rs:282-298`) and spec (`:323-326`) intakes route through `validate_with_allow` → `validate_fields` → `check_secret_key`. Unchanged.
   - **SchemaField reuse ⇒ no `--json`/schema_mirror delta:** `field_diag` → `DiagnosticKind::SchemaField` + `flag: None` (`gate.rs:679-686`); `as_str` already maps `SchemaField => "schema_field"` (`:121-135`) — NO new discriminant, NO clap-surface change. Confirmed live.
   - **L23:** typed `InvalidScalar` (`electrum_crypto.rs:247`) before `mul_tweak().expect()` (`:350-351`); `privkey:&[u8;32]` sig makes `.iter().all(|&b| b==0)` valid; latent (sole CLI caller `derive_storage_eckey` already guards, `:309-310`); firewalled from P1 (different file/test). Unchanged.
   - **Version sites (0.64.0 → 0.65.0):** Cargo.toml:3, README.md:13, crates README:9, install.sh:32, fuzz/Cargo.lock:575, CHANGELOG:9 — ALL verified `0.64.0` live; `0.65.0` free. README markers auto-gated by `readme_version_current.rs`. Complete.
   - **Bughunt tick lines:** M8 `:724` / L23 `:833` live (plan cites `:721`/`:830`, +3 stale) — the plan's own "re-grep at impl time" note (lines 144, 154) covers this. Carried Minor from round 1, unchanged.

---

## Critical

**None.** (Unchanged from round 1 — the funds core was 0-Critical and remains so; the fold did not touch it.)

## Important

**None.** I-1 is **RESOLVED**. The plan no longer asserts the false universal-`--key` property; it states the empirically-correct per-archetype reality (single-key → `--key`, quorum → `--threshold`), frames the `--threshold` resolution as a benign provenance artifact (path/message still name the key; exit-2 refusal is flag-independent; funds-safety unaffected), and T1b now non-vacuously tests a quorum archetype against `--threshold` — the exact case the round-1 single-key scoping was blind to. Independently re-derived AND cross-checked against the live pinned test (`archetype.rs:747-749`/`:756-758`). No new Important introduced.

## Minor

**M-1 (carried from round 1, unchanged — non-blocking).** Bughunt-report checkbox cites are +3 stale: M8 is `:724` (plan `:721`), L23 is `:833` (plan `:830`) @ `d6398b57`. Covered by the plan's own "re-grep the current line numbers at impl time" instruction (lines 144, 154). Cosmetic; not a gate blocker.

**M-2 (new, cosmetic — non-blocking).** T1b's general example node_path is written `root.<quorum>[0].multi.keys[i]`, but **tiered-recovery**'s primary quorum is a `Sortedmulti`, so its key node_path is `root.or_i[0].sortedmulti.keys[i]` (`node.kind()="sortedmulti"`), not `.multi.`. The plan's concrete "Assert BOTH" recommendation uses **decaying-multisig** (a `Multi`), where `.multi.keys[i]` is exact — so the recommended test is correct as written; only the generic illustration is slightly imprecise for the sortedmulti case. No action required (the chosen test archetype is unaffected); noting for accuracy if the implementer swaps to tiered-recovery.

---

## Scope confirmations (all hold)

- **Funds property:** M8 fails closed (exit 2, no descriptor) on every key-bearing field + every nested subtree + both intakes — structurally + empirically verified (unchanged from round 1). ✅
- **No over-rejection:** `contains('/')` post-`]` is exactly the M8 class; bare/`[origin]`/SLIP-132 keys still build (T3). ✅
- **I-1 (flag-provenance):** folded **CORRECTLY** — per-archetype reality stated, quorum→`--threshold` empirically true, T1b non-vacuous for quorum. ✅
- **Minor-2 (path-fidelity):** `field_diag` takes the `path` arg; pass-through correct; T2 pins `root.multi.keys[0]`. ✅ (unchanged)
- **No `--json`/schema_mirror/manual/codec trigger:** `SchemaField` reuse adds no `as_str` discriminant; no clap surface change. ✅
- **L23:** typed before `mul_tweak().expect()`; latent; firewalled; no new variant. ✅
- **Version sites:** all six enumerated + verified `0.64.0` → bump `0.65.0`. ✅
- **TDD:** RED-first, non-vacuous; BIN-target suite. ✅

---

## Verdict

The round-1 Important (I-1) is fully and correctly resolved. The plan now states the per-archetype flag reality, the quorum→`--threshold` claim is empirically TRUE (independently re-derived against the live provenance tables and corroborated by an existing pinned test), and T1b non-vacuously covers a quorum archetype against `--threshold` — closing the test-blindness gap. The fold introduced no new drift: the funds core (guard predicate, recursion coverage, both-intake routing, no over-rejection, no-leak), the L23 typing, the `SchemaField` no-`--json`-delta property, and the version-site enumeration are all unchanged and re-verified sound. Two Minors remain (stale +3 bughunt cites covered by the plan's re-grep note; a cosmetic sortedmulti node_path illustration) — neither is a gate blocker.

**PLAN R0 ROUND 2: 0C / 0I — GREEN**
