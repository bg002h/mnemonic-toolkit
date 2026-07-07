# PLAN R0 review — bip388-double-star-shorthand-support — round 1

**Verdict: NOT GREEN (0 Critical / 2 Important / 3 Minor)**
**Reviewer:** opus architect, source basis `0964462d`.
**Dispatched:** 2026-07-06 (Cycle C, IMPLEMENTATION_PLAN R0 loop round 1). Persisted verbatim before fold per CLAUDE.md.

Structure, TDD discipline, phase gates, release ritual, and the bsms.rs:300 soft-gap analysis are sound. Both blockers are in **P0 Task 1 — the call-site completeness classification** (the gate the plan exists to nail): it MISSES an IN-scope BSMS soft-gap (`canonicalize_bsms`→`recanonicalize_descriptor`→roundtrip.rs:241) and asserts a factually FALSE OUT-rationale for three user-`/**`-reachable sites. No Critical: every gap is fail-closed (reject) or cosmetic-diagnostic — no funds/wrong-wallet risk.

## Important

### I1 — Missed IN-scope BSMS surface: `canonicalize_bsms`→`recanonicalize_descriptor` from_str soft-gap
- `import_wallet.rs:1458` `"bsms" => Some(canonicalize_bsms(blob)…)` dispatches the raw BSMS blob.
- `roundtrip.rs:96` `canonicalize_bsms` extracts the descriptor line → `recanonicalize_descriptor(descriptor_with_csum)`.
- `roundtrip.rs:241` `MsDescriptor::from_str(body_no_csum)` on the RAW body → rejects `/**` (`ImportWalletParse: "canonicalize: descriptor parse failed"`).
- `import_wallet.rs:1457` captures the error into `canon_orig` → surfaced in the `--json` roundtrip/canonicalize-failed envelope, NOT hard-failed.

**Consequence:** `import-wallet --format bsms --json` on a `/**` descriptor SUCCEEDS (main parse expands at `parse_descriptor:875`) but its roundtrip/canonical field reports a bogus parse failure — a soft-gap exactly parallel to bsms.rs:300 which the plan already elevated to a first-class IN fix. BSMS is explicitly IN-scope (SPEC §0 IN-1). The plan lumps `roundtrip.rs:241` into OUT with rationale "roundtrip … toolkit-generated" — FALSE: `canonicalize_bsms` feeds it raw user BSMS text.

**Fix:** expand inside `recanonicalize_descriptor` (roundtrip.rs:231, before from_str@241) — single chokepoint for both callers (the bitcoin-core caller roundtrip.rs:170 is a harmless no-op, Core never emits `/**`). Add a §7 cell: `import-wallet --format bsms --json` `/**` yields a clean roundtrip/canonical field == the `/<0;1>/*` spelling. Also: P0 Task 1 must NOT blanket-OUT `roundtrip.rs` — individually check each `canonicalize_*` (coldcard/electrum/sparrow/specter, roundtrip.rs:305/484/637/807) for the same raw-body-from_str class.

### I2 — OUT rationale factually wrong for three user-`/**`-reachable sites
- **`export_wallet.rs:517`** — `export-wallet --descriptor "wpkh([fp]xpub/**)"`: after `reject_md1_card` + `expand_bip388_policy` (JSON only) + `is_at_n_form` reject, a CONCRETE `/**` hits `from_str(desc)@517` → HARD-rejects. Asymmetry: `import-wallet --format descriptor` accepts `/**` but `export-wallet --descriptor` rejects.
- **`cost/strip.rs:21`** — `compare-cost --descriptor "wsh(…xpub/**)"` (`compare_cost.rs:86`) → `translate_descriptor` → `from_str(input)@21` → HARD-rejects.
- **`gui_schema.rs:1319`** — `gui-schema --classify-descriptor "…/**"` → `parse_descriptor(input,…)` → user input, but AUTOMATICALLY covered by the `:875` expander (beneficial). Disposition (no separate touch) right; rationale ("non-user") wrong.

**Fix:** split OUT into (a) genuinely toolkit-generated/never-`/**` — `restore.rs:2066/2380/2760/3238` (md1-card reconstructions, encode `/<0;1>/*` never `/**`), `nostr.rs:142` (built string), `export_wallet.rs:633/796` (post-517 canonical), `wallet_export/*`, `descriptor_builder/gate.rs`, `bitcoin_core.rs:337/1026`, roundtrip.rs:170-via-bitcoin-core — CONFIRMED OUT-correct; vs (b) user-`/**`-reachable OUT-of-SPEC-scope-command — `export_wallet.rs:517`, `cost/strip.rs:21`. For (b), make an EXPLICIT documented IN/OUT decision. Recommend scoping both IN for consistency (import-accepts/export-rejects is poor UX; each is a one-line `expand_literal_double_star` before from_str). If kept OUT, document the asymmetry + improve the raw error to point at `/<0;1>/*`. Reclassify `gui_schema.rs:1319` as "user-input, chokepoint-covered by :875" + add a one-line accept test.

## Minor
- **M1** — TDD ordering: the 3 flipped tests (§7.1 parse_descriptor.rs:1731, §7.2 cli_import_wallet_descriptor.rs:191, §7.9 message) go RED when flipped, green only when expander+reword land → flip in the SAME commit/phase as impl (preserve "RED for the right reason").
- **M2** — State re-vendor N/A this cycle (no dep bump; `Cow` is std; `vendor/` untouched) so vendor-freshness isn't skipped-in-confusion.
- **M3** — Post-impl "5th path" grep should explicitly enumerate the `canonicalize_*` family + `compare-cost`/`export-wallet --descriptor` surfaces (I1/I2) as known adjacencies to confirm resolution.

## Scrutiny answers
1. **Classification correct+complete?** NO (I1,I2). (a) `parse_descriptor:875` covers every concrete path — all ~10 concrete callers feed parse_descriptor; `/**` survives key→@N substitution as a terminator-bounded token (`[fp]xpub/**)`→`@N[fp]/**)`, still `)`-bounded), reaching lex@884 + from_str@897. ✅ (b) `restore.rs:*`, `nostr.rs:142`, `export_wallet.rs:633/796`, `bitcoin_core.rs` OUT-correct — but `export_wallet.rs:517`, `cost/strip.rs:21`, `gui_schema.rs:1319` mis-classified (I2). (c) bsms.rs:300 analysis correct — `derive_first_address` handles multipath (splits via into_single_descriptors, branch 0), so `/**`→`/<0;1>/*` routes into the same path normal BSMS `/<0;1>/*` uses → safe; but the plan missed the parallel canonicalize_bsms soft-gap (I1).
2. **Phase decomposition sound?** Yes. P0-as-one-phase fine; message reword IS behavior (correctly coupled). No ordering hazard beyond M1.
3. **TDD completeness?** §7.1-7.10 assigned; §7.3/7.4 RED-first funds anchor; §7.9 non-tautological (tests message vs a different still-rejected `/0/*`, also proving floor preserved). New I1/I2 coverage to add once scoped.
4. **New funds/behavior risk?** None funds-bearing. bsms.rs:300 no-op on non-`/**`, `/**` uses existing multipath derive path — safe. Only new surface = export/cost hard-reject inconsistency + BSMS canonicalize soft-gap (I1/I2), both fail-closed/cosmetic.
5. **Missing steps?** fmt/clippy/full-suite/whole-diff/version-sites all ✅ (matches v0.77.0 ritual). Only M2 (re-vendor N/A) missing.

**To GREEN:** resolve I1 (add recanonicalize_descriptor/roundtrip.rs:241 IN + BSMS-`--json` test; audit other canonicalize_*), I2 (correct OUT split + explicit decision on export-wallet/compare-cost --descriptor, recommend IN; reclassify gui_schema:1319), fold M1-M3, re-dispatch (the I2 scope decision adds call sites + tests that need review).
