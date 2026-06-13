# Phase 4 (Task 4, docs) — combined spec-compliance + quality review (verbatim)

> Persisted verbatim per CLAUDE.md. Docs-only task → ONE combined reviewer (spec-compliance + prose
> accuracy overlap for prose). Dispatched via Agent (general-purpose, Opus 4.8). Diff BASE `b9b4e75` →
> HEAD `2fa44dc`. **Verdict: ✅ compliant + accurate, 0 defects** (1 non-blocking out-of-scope note).

---

## Review of Task 4 (commit `2fa44dc`) — ✅ COMPLIANT + ACCURATE

Read the diff, the shipped code, the tests, the FOLLOWUPS entries, and ran the full manual lint. No defects found.

### Spec compliance — all 5 points done
1. **restore.rs `build_multisig_import_payload` comment** (`restore.rs:853-856`): refreshed to "`Some(Nums)` or `Some(Cosigner(idx))`"; stale `(R0 v2 I2.)` tail replaced with `(Non-NUMS real-trunk support: v0.55.3.)`. ✅
2. **`classify_taproot_restore` doc-comment** (`restore.rs:678-681`): already updated in Task 1, accurate; Task 4 did not duplicate. ✅
3. **Manual prose** (`41-mnemonic.md`, 3 blocks: intro :768-775, general-policies :1001-1011, --format :1024-1054): all updated for v0.55.3. ✅
4. **Manual lint**: ran it — all 6 stages pass (markdownlint 0 err, cspell 0 issues, lychee 0 err, flag-coverage/glossary/index clean). ✅
5. **FOLLOWUPS.md**: `restore-non-nums-taproot-internal-key` RESOLVED; `restore-non-nums-tr-internal-key-also-in-leaf` filed open. ✅

### Accuracy — prose matches shipped behavior (cross-checked against code + tests)
- Non-NUMS general single-leaf/depth-1 + distinct-trunk multisig reconstruct: `classify_taproot_restore` `is_nums:false` routing (`restore.rs:702-744`); N1/N2/N3 pass.
- @-in-both refused exit 2: `refuse_at_in_both` → `ModeViolation` (`restore.rs:772-790`); N4/N4b/N4c assert exit 2 + slug.
- depth-≥2 refused: `ensure_taptree_depth_le_one`. Not over-claimed.
- --format matrix: bip388 + green refused only in the route-around `None` branch (`restore.rs:897-914`); distinct-trunk multisig takes `Some(t)` Template path and emits bip388 (`tr(@idx/**,multi_a(k,…))`) — N5/N6/N7/N8/N9 confirm.

### FOLLOWUPS correctness
- Resolved entry cites impl commits `9c7edeb`/`a8fe9bd`/`11cd75e` (all exist), "v0.55.3 (pending tag)", the persisted phase-1/2/3 reviews + spec/plan.
- Deleted test cell genuinely gone (removed in `9c7edeb`, not this docs commit).
- **Slug parity confirmed**: guard message `restore.rs:784` cites `restore-non-nums-tr-internal-key-also-in-leaf`, byte-matching the FOLLOWUPS heading + manual prose `:1010`.

### No code/test changes
`--stat`: only `restore.rs` (doc-comment hunk — filtering non-`///` changed lines returned nothing, zero logic), `FOLLOWUPS.md`, `41-mnemonic.md`.

### Minor (non-blocking, pre-existing, out of scope)
The prior FOLLOWUP `restore-general-and-multi-leaf-taproot-roundtrip` item (v) (`FOLLOWUPS.md:4099`) still names the old slug `restore-multisig-taproot-reconstruction` for the non-NUMS remainder, now superseded by the new entry. Task 4 did not touch that line — no defect, just a stale cross-reference for a future cleanup pass.

**Verdict: ✅ Task 4 is spec-compliant and prose-accurate. No prose contradicts shipped behavior; no over-claims.**
