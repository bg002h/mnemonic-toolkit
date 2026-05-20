# AUDIT FINDINGS — manual-v0.2.0 cycle (v0.28.0 content audit)

**Phase:** P1a (mechanical transcript capture) — findings gathered live as each transcript is captured. Per `design/PLAN_manual_v0_2_0_content_audit.md` §7 P1a + §1 Q7 classification rubric.

**Working SHA:** `223644d` (post-P0-recon-commit).
**Binary under test:** `target/debug/mnemonic` reports `mnemonic 0.28.1`.

---

## Systematic finding class — chapter-45 Round-trip examples uniformly broken

P1a captured all 6 chapter-45 `### Round-trip example` H3 subsections against the v0.28.1 binary. **All 6 fail at step 2 (the `export-wallet` invocation).** Three sub-classes:

- **Class A (4 of 6) — missing `--template`:** Sparrow / Coldcard / Jade / Electrum each require `--template <value>` on the export side but the documented recipe omits it. Sub-error texts differ but all share the pattern `--format <X> requires --template`.
- **Class B (1 of 6) — missing `--wallet-name`:** Specter export requires `--wallet-name <STRING>`; recipe omits it.
- **Class C (1 of 6) — format-name asymmetry:** Coldcard multisig's import accepts `--format coldcard-multisig` (per L462 H2 + the documented recipe), but export-wallet's clap-parser does NOT accept `coldcard-multisig` as a valid value — only `coldcard`. The Round-trip recipe is non-functional because the format name itself is asymmetric across import/export.

The systematic pattern strongly suggests **the chapter authors documented the import side correctly but did not exercise the export side end-to-end before merging P13A**. The lint gate at the time only validated flag-coverage (no command execution per `feedback_architect_must_run_prose_commands`), so the gap shipped silently.

**Q7 classification (tentative — P1b architect to confirm; defaults to doc-update + 1 toolkit-fix):**
- Class A: **doc-update.** Add `--template <value>` to each affected recipe with the value matching the imported descriptor's script-type. Plus reconcile prose drift: the per-format "Deferral" paragraphs (e.g., Sparrow L315-324 claiming "descriptor-passthrough" on emit) are misleading; tighten language to specify the descriptor-passthrough caveat is taproot-only.
- Class B: **doc-update.** Add `--wallet-name <STRING>` to the Specter recipe.
- Class C: **gray-area, leaning doc-update + clarification.** Two options:
  - (c1) Doc-fix: clarify that import accepts both `coldcard` and `coldcard-multisig` but export only emits via `coldcard` (single-sig format); document why the recipe-as-written cannot round-trip on the multisig side.
  - (c2) Toolkit-fix: add `coldcard-multisig` as a valid `--format` value to export-wallet, even if it just dispatches to the same code path as `coldcard` with multisig detection.
  - Default: c1 unless P1b architect locks c2 as a UX regression.

---

## Finding F1 — Sparrow Round-trip example: export step missing required `--template`

- **Where:** `docs/manual/src/45-foreign-formats.md:301-313`
- **Documented step 2:** `mnemonic export-wallet --from-import-json envelope.json --format sparrow > sparrow_re.json`
- **Actual stderr:** `error: --format sparrow requires --template; descriptor passthrough is not supported by Sparrow's file-import surface`
- **Exit code:** 1
- **Fixture:** `sparrow-singlesig-p2wpkh.json` (exact-match per P0c)
- **Import step (step 1):** Works (exit 0). `envelope.json` valid: single-entry array; `mode: watch-only`; descriptor `wpkh([5436d724/84'/0'/0']xpub6Bner3...)`; `roundtrip.byte_exact: false` with documented field-ordering diff in the envelope's `roundtrip.diff` field.
- **Suggested fix:** add `--template bip84` to step 2 (matching the import's `wpkh([fp/84'/0'/0']xpub)` script-type). Reconcile L315-324 deferral prose.

## Finding F2 — Specter Round-trip example: export step missing required `--wallet-name`

- **Where:** `docs/manual/src/45-foreign-formats.md:367-374`
- **Documented step 2:** `mnemonic export-wallet --from-import-json envelope.json --format specter > specter_re.json`
- **Actual stderr:** `error: mnemonic export-wallet --format specter requires the following missing fields:\n  - wallet_name (supply --wallet-name <STRING>)\nRe-invoke with all missing fields supplied.`
- **Exit code:** 2
- **Fixture:** `specter-singlesig-p2wpkh.json` (exact-match per P0c)
- **Import step:** Works (exit 0).
- **Suggested fix:** add `--wallet-name <STRING>` to step 2.

## Finding F3 — Coldcard single-sig Round-trip example: export step missing required `--template`

- **Where:** `docs/manual/src/45-foreign-formats.md:443-450`
- **Documented step 2:** `mnemonic export-wallet --from-import-json envelope.json --format coldcard > coldcard_re.json`
- **Actual stderr:** `error: --format coldcard requires --template (bip44 / bip49 / bip84); pass a recognized template or use a different format for descriptor passthrough`
- **Exit code:** 1
- **Fixture:** `coldcard-singlesig-bip84-mainnet.json` (exact-match per P0c)
- **Import step:** Works (exit 0).
- **Suggested fix:** add `--template bip84` to step 2.

## Finding F4 — Coldcard multisig Round-trip example: format-name asymmetry

- **Where:** `docs/manual/src/45-foreign-formats.md:526-???` (Round-trip example for `--format coldcard-multisig`; exact end-of-block line not captured at P1a; pin at P2)
- **Documented step 2:** `mnemonic export-wallet --from-import-json envelope.json --format coldcard-multisig > coldcard_re.json` (or equivalent — used `coldcard-multisig` per the L462 H2 documentation)
- **Actual stderr:**
  ```
  error: invalid value 'coldcard-multisig' for '--format <FORMAT>'
    [possible values: bitcoin-core, bip388, coldcard, jade, sparrow, specter, electrum, green, bsms]

    tip: a similar value exists: 'coldcard'
  ```
- **Exit code:** 64 (clap-derive: invalid-arg)
- **Fixture:** `coldcard-ms-2of3-p2wsh-with-xfp.txt` (exact-match per P0c)
- **Import step:** Works with `--format coldcard-multisig` (exit 0).
- **Suggested fix (P2 candidate):** Doc-fix preferred (c1 above) — document the asymmetry explicitly; show the recipe with the import side using `coldcard-multisig` and the conceptual round-trip closing out at the BSMS export level via recipe 7's pattern, since coldcard-multisig has no symmetric export side. Toolkit-fix (c2) is also defensible but enlarges surface.

## Finding F5 — Jade Round-trip example: export step missing required `--template`

- **Where:** `docs/manual/src/45-foreign-formats.md:590-???` (exact end-of-block line at P2)
- **Documented step 2:** `mnemonic export-wallet --from-import-json envelope.json --format jade > jade_re.json`
- **Actual stderr:** `error: --format jade requires --template (wsh-multi / wsh-sortedmulti / sh-wsh-multi / sh-wsh-sortedmulti); descriptor passthrough is not supported by Jade's file-import surface`
- **Exit code:** 1
- **Fixture:** `jade-multisig-2of3-p2wsh.json` (exact-match per P0c)
- **Import step:** Works (exit 0).
- **Suggested fix:** add `--template wsh-sortedmulti` (the imported fixture's expected template) to step 2.

## Finding F6 — Electrum Round-trip example: export step missing required `--template`

- **Where:** `docs/manual/src/45-foreign-formats.md:684-???` (exact end-of-block line at P2)
- **Documented step 2:** `mnemonic export-wallet --from-import-json envelope.json --format electrum > electrum_re.json`
- **Actual stderr:** `error: --format electrum requires --template; descriptor passthrough is not supported by Electrum's wallet-db schema`
- **Exit code:** 1
- **Fixture:** `electrum-multisig-2of3-wsh.json` (exact-match per P0c)
- **Import step:** Works (exit 0).
- **Suggested fix:** add `--template <value>` to step 2 — exact template value TBD at P2 (Electrum's clap-derive `--template` doesn't list values in the error message; P2 must enumerate them via `mnemonic export-wallet --format electrum --template ?` or similar).

---

## P1a chapter-45 status

| Round-trip | Finding | Transcript captured |
|---|---|---|
| Sparrow L299 | F1 | DEFERRED to P2 (await prose fix) |
| Specter L367 | F2 | DEFERRED to P2 |
| Coldcard single-sig L443 | F3 | DEFERRED to P2 |
| Coldcard multisig L526 | F4 | DEFERRED to P2 (format-name asymmetry — recipe may need restructure) |
| Jade L590 | F5 | DEFERRED to P2 |
| Electrum L684 | F6 | DEFERRED to P2 |

**Zero clean captures from chapter-45.** This is the major audit deliverable — 100% of foreign-format Round-trip recipes need prose fixes before they can ship as durable CI-gated transcripts.

---

## Open question for user / architect (PAUSE point)

The systematic chapter-45 finding suggests that running the 8 recipes in `30-workflows/39-cross-format-conversion.md` will likely surface similar export-side issues. Two options:

- **(α) Continue P1a for the 8 recipes + chapter-41 inheritance** before any P2 fix work. Total of 15 captures; expected pattern: many additional findings of the same class.
- **(β) Pause P1a after chapter-45; dispatch P1b architect classification on F1-F6 + P2 fix work first**, THEN return to recipe captures with the fixes applied. Tighter feedback loop; risks P1a re-runs if fixes change recipe-recipe interactions.

Recommend (β) — the systematic pattern is already proven, and fixing the prose first means the recipes can be captured against a known-good state in the same go. User direction needed.
