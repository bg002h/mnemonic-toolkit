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

P1b classification persisted at `design/agent-reports/manual-v0_2_0-p1b-r0-classification.md` (6/6 locked doc-update; F4 c1 locked + new FOLLOWUP `export-wallet-coldcard-multisig-alias` filed for c2 deferred).

---

## P1a chapter-30/39/41 — chapter-39 recipes + chapter-41 inheritance composite

P1a resumed 2026-05-20 (post-P1b classification) per β-with-nuance scheduling: dispatch P1b architect on F1-F6 → resume P1a for remaining captures → batched P2.

**Capture method:** per-cmd `mktemp -d` cwd staging with `cp $FIXTURES_DIR/<actual-fixture> <bare-filename-from-prose>` for the 3 Q10-substituted recipes (1/2/3); bare-filename copy for the 5 exact-named recipes (4-8). Recipes captured as `.cmd`/`.out`/`.err` triples; chapter-41 inheritance composite captured as `.cmd`/`.out` pair (2>&1 merge per §2.1 existing convention).

| Capture | Source line | Exit | Finding | Status |
|---|---|---|---|---|
| recipe-1 BSMS → Bitcoin Core | 39-L52 | 0 | none | Clean — stderr 2-line-excerpt warning is fixture-substitution artifact (P0c locked `bsms-shwsh-2of3.txt`, which is a 2-line fixture), not a prose finding |
| recipe-2 Bitcoin Core → bundle | 39-L83 | 4 (step 3) | none | Steps 1+2 clean (exit 0). Step 3 exit-4 is the prose's documented failure mode (L102-104 explicitly states "Mismatch returns exit 4") — abandon-test-vector phrase doesn't derive to the fixture's xpub. Consistent with prose. |
| recipe-3 BSMS → BIP-388 | 39-L112 | 0 | none | Clean — stderr 2-line-excerpt warning is same fixture-substitution artifact class as recipe-1 |
| recipe-4 Sparrow → BSMS | 39-L144 | 0 | F7 + F8 + F9 | Exit 0 but output L2 missing `#checksum` (F9 toolkit-fix gray area); prose component list wrong (F8); doc spec at chapter-45 L85-97 wrong (F7) |
| recipe-5 Specter → Bitcoin Core | 39-L166 | 0 | none | Clean. F2's `--wallet-name` failure class does NOT trigger (recipe targets `bitcoin-core`, not `specter`) — P1b risk flag #3 ruled out. |
| recipe-6 Coldcard → BIP-388 | 39-L192 | 0 | F10 | Clean exit; prose L199 fingerprint case (`B8688DF1` uppercase) ≠ actual output (`b8688df1` lowercase per BIP-388 canonicalization). Minor cosmetic. |
| recipe-7 Jade → BSMS | 39-L216 | 0 | F9 | Same `#checksum`-missing-from-L2 toolkit behavior as recipe-4. Recipe-7 prose at L221-225 is generic enough to NOT trip F8's class. |
| recipe-8 Electrum multisig → BSMS | 39-L235 | 0 | F8 + F9 | Same as recipe-4: prose component list wrong (F8) + L2 missing `#checksum` (F9). |
| chapter-41 inheritance composite | 41-L100-102 + L222-230 + L366-373 | 0 | F11 | Stdout block (22 lines, ms1/mk1 decode + match per cosigner, final `result: ok`) byte-matches prose `text` block at 41-L379-401 exactly. Stderr (8 lines: 6 secret-warnings + 1 non-canonical-info + 1 stdout-warning) NOT disclosed in prose; minor completeness gap. |

**P1a chapter-30/39/41 summary:** 9 captures complete; 5 clean + 4 carrying findings F7-F11. P1b risk flags #1 (`--account` rejection) and #2 (null `bundle.descriptor`) did NOT surface in any chapter-39 recipe (no recipe supplied `--account` on `--from-import-json`; no fixture produced a null descriptor envelope). Risk flag #3 (Specter `--wallet-name`) ruled out by recipe-5 targeting bitcoin-core. Risk flag #4 (recipe-7 `--bsms-form`) is moot — recipe-7 doesn't override the default 4-line and the captured output is the 4-line shape (the `#checksum` issue dominates that capture instead).

---

## Finding F7 — chapter-45 L85-97 BSMS 4-line shape documentation contradicts bsms.rs source-of-truth

- **Where:** `docs/manual/src/45-foreign-formats.md:85-97`
- **Prose claim:** the 4-line shape is `BSMS 1.0` / `<TOKEN>` / `<descriptor>#<checksum>` / `<DERIVATION_PATH>` with "No first-address verification line and no signature line".
- **Actual emit (`bsms.rs:1-12` + bsms.rs:90-115):** 4-line shape is `BSMS 1.0` / `<descriptor>#<checksum>` / `<path-restrictions>` / `<first-address>`. **No token line.** First-address IS present (derived via `derive_first_address`).
- **Disagreement scope:**
  - Token line claim: chapter-45 says line 2 is `<TOKEN>`; toolkit emits descriptor at line 2 and never emits a token line.
  - Descriptor line position: chapter-45 says line 3; toolkit puts at line 2.
  - Line 3 content: chapter-45 says "derivation path"; toolkit emits path-restrictions string (per §3.5.1 cosigner-key suffix walk).
  - First-address line: chapter-45 explicitly says "No first-address verification line"; toolkit emits one as line 4 via `derive_first_address`.
- **Captured evidence:** `docs/manual/transcripts/cross-format-recipes/recipe-{4,7,8}-*.out` all show the 4-line `BSMS 1.0` / descriptor / path-restrictions / first-address shape — consistent with bsms.rs source, contradicting chapter-45.
- **Classification (tentative — P1b architect to confirm):** **doc-update.** Chapter-45 L85-97 must be rewritten to match the actual emit. The `<TOKEN>` line and "No first-address verification line" claims are both wrong. The bsms.rs module-doc-comment is the authoritative spec for the shape (BIP-129 Round-2 plaintext form with first-address; the token+signature travel out-of-band per the bsms.rs:8-12 comment). Note that chapter-45 L67-73 also mentions "token, signature, first-address verification fields" in a general overview paragraph that is consistent with the bsms.rs reality — the bug is localized to the L85-97 4-line shape ```text block and its surrounding L93-98 explanatory prose.

## Finding F8 — chapter-39 recipes 4/8 prose claims about BSMS 4-line components contradict bsms.rs emit shape

- **Where:** `docs/manual/src/30-workflows/39-cross-format-conversion.md:149-150` (recipe-4) + `:240-242` (recipe-8)
- **Recipe-4 prose claim:** "a 4-line BSMS Round-2 blob (`BSMS 1.0`, token, descriptor with `#<checksum>`, derivation path)"
- **Recipe-8 prose claim:** "BIP-129-canonical 4-line BSMS Round-2 blob (`BSMS 1.0` header, token, descriptor, derivation path)"
- **Actual output (captured):** `BSMS 1.0` / descriptor (with-or-without `#checksum` per F9) / path-restrictions / first-address. **No token line; not "derivation path" but "path-restrictions"; first-address line present but not described.**
- **Recipe-7 (Jade → BSMS) is NOT in scope of F8** — its prose at L221-225 is generic ("re-emits as 4-line BSMS Round-2") without enumerating components, so no specific component claim drifts.
- **Classification (tentative):** **doc-update.** Recipes 4/8 prose should mirror chapter-45's corrected 4-line shape from F7's fix. Both prose blocks need rewriting to drop the "token" claim, change "derivation path" → "path-restrictions", and add "first-address" to the component list.

## Finding F9 — `export-wallet --from-import-json --format bsms` emits L2 descriptor WITHOUT `#checksum`

- **Where:** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:566-567` (where `canonical_descriptor` is built via `descriptor_body_no_csum` for the `--from-import-json` path) → flows into `wallet_export/bsms.rs:91-92` (`let line2 = inputs.canonical_descriptor;`).
- **bsms.rs source-of-truth invariant (`bsms.rs:86-90`):** "Lines 1 + 2 are shared between the 2-line and 4-line shapes. Line 2 is `EmitInputs.canonical_descriptor` verbatim — the canonical builder (`wallet_export::pipeline::build_descriptor_string`) and descriptor-passthrough both produce strings with the `#<checksum>` suffix already attached."
- **Bug:** The `--from-import-json` path **violates this invariant** because `descriptor_body_no_csum` (at export_wallet.rs:567) strips the checksum before constructing `EmitInputs`. BSMS L2 is therefore the descriptor BODY without checksum, which the comment claims is impossible.
- **Captured evidence:** `recipe-4-sparrow-to-bsms.out` L2 ends with `*))` — confirmed via `cat -A` showing no `#xxxxxxxx` suffix before the line-terminator. Same in recipe-7 and recipe-8 BSMS outputs.
- **Behavioral consequence:** BIP-129 Round-2 plaintext requires the BIP-380 checksum on the descriptor line so coordinators can verify integrity. A BSMS Round-2 blob emitted by `mnemonic export-wallet --from-import-json - --format bsms` is non-compliant on this dimension and downstream BSMS-consuming coordinators (Coldcard Mk4, Specter Desktop, etc.) may reject it.
- **Classification (gray area — P1b architect locks):** Q7 gray-area: stderr/output template drift from documented behavior. Two options:
  - **(c1) Doc-fix:** update bsms.rs:86-90 comment + chapter-45 (post-F7 fix) + recipes 4/7/8 prose to acknowledge the `--from-import-json` path strips the checksum. Document the limitation. Cycle stays manual-only.
  - **(c2) Toolkit-fix:** re-add the BIP-380 checksum to `canonical_descriptor` after `descriptor_body_no_csum` strips it (either in cmd/export_wallet.rs:567 by re-attaching the checksum, or in bsms.rs:91-92 by computing it just-in-time before emit). Cycle promotes to paired `mnemonic-toolkit-v0.28.2` patch per Q8.
  - **Tentative recommendation (subject to architect lock):** **c2 (toolkit-fix).** The bsms.rs:86-90 comment is correct *as the invariant should hold* — the bug is in the invariant-breaking `--from-import-json` path, not in the BSMS emitter's expectation. Re-adding the checksum is a small fix (one call to `descriptor.to_string_with_checksum()` or equivalent on the parsed `MsDescriptor`). This is a behavioral bug whose user-visible blast radius is downstream coordinator rejection — exactly the Q7 "toolkit-bin behavior bug" category that triggers Q8 promotion.

## Finding F10 — recipe-6 prose L199 fingerprint case (minor)

- **Where:** `docs/manual/src/30-workflows/39-cross-format-conversion.md:199`
- **Prose claim:** "a single-element `keys_info` array (`[B8688DF1/84'/0'/0']xpub6FQya7zGhR9...`)"
- **Actual output:** `[b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX`
- **Drift:** `B8688DF1` (uppercase) → `b8688df1` (lowercase per BIP-388 canonicalization).
- **Classification:** **doc-update.** Lowercase the fingerprint in the L199 prose example. Cosmetic only; no behavioral implications.

## Finding F11 — chapter-41 prose doesn't disclose verify-bundle / bundle stderr (minor completeness gap)

- **Where:** `docs/manual/src/40-cli-reference/41-mnemonic.md:222-230` (JSON-form bundle) + `:366-373` (verify-bundle).
- **Captured stderr (from 41-inheritance.out, 2>&1 merge):** 8 lines preceding the documented 22-line stdout block — 3× "secret material on argv (--slot @0.phrase=)" warnings from bundle, 1× "info: non-canonical descriptor; defaulting origin path for @0,@1,@2 to m/48'/0'/0'/2' (BIP-48 cosigner path)" info notice from bundle (v0.19.0 silent-default-with-stderr-notice feature), 1× "secret material on stdout — consider redirecting" warning from bundle, 3× "secret material on argv (--slot @i.phrase=)" warnings from verify-bundle.
- **Prose disclosure scope:** chapter-41 prose at L195-220 documents the engraving-card stderr from the TEXT-form bundle (L111-118 command), but does NOT document any stderr block from the JSON-form bundle or from verify-bundle. The 8-line stderr from these two commands is therefore undocumented.
- **Captured stdout block:** byte-identical match to the chapter-41 prose's "Expected output" `text` block at L379-401. The audit deliverable for the verify-bundle stdout is clean.
- **Classification (tentative):** **doc-update.** Either (a) add a brief stderr disclosure paragraph after L373 noting the warning + info-notice classes that fire; (b) leave undocumented because the warnings are generic (apply to every command with `--slot @N.phrase=`) and the info-notice is documented in chapter-19 v0.19.0 context. Default is (a) — chapter-41 is the worked example chapter, and the v0.19.0 info-notice IS load-bearing here (the recipe demonstrates a non-canonical `wsh(andor(...))` descriptor that triggers the BIP-48 default-path inference). Showing the info-notice in the expected-output block teaches the reader what to expect. Lower-priority than F7/F8/F9 (no behavioral risk).

---

## P1a chapter-30/39/41 captured transcripts (committed to repo)

- `docs/manual/transcripts/cross-format-recipes/recipe-{1..8}-*.{cmd,out,err}` (24 files; triple format per §2.1)
- `docs/manual/transcripts/41-inheritance.{cmd,out}` (2 files; pair format per §2.1 + existing convention)

Total 26 new files. All captures run via direct `bash -c "$cmd" 2>err >out` in per-cmd `mktemp -d` cwd against `target/debug/mnemonic` (mnemonic 0.28.1) per plan-doc §7 P1a. Round-trip invariant per §7 P1a holds — the captures are replayable byte-identical when P3's verify-examples.sh extension runs them.

---

## Overall P1a status (after chapter-45 + chapter-30/39/41)

- **Chapter-45 (6 captures):** 0/6 clean — F1-F6 all DEFERRED to P2 awaiting prose fixes per P1b classification.
- **Chapter-39 (8 captures):** 5/8 clean (recipes 1/2/3/5/6 — recipe-2 step 3's documented exit-4 counts as clean per Q5); 3/8 carry findings F7/F8/F9/F10 (recipes 4/6/7/8 — recipe-6 is F10 only, recipes 4/7/8 share F9, recipes 4/8 also F8, F7 is the chapter-45 doc-spec root issue affecting all BSMS-emit recipes).
- **Chapter-41 (1 capture):** 1/1 stdout-clean (byte-match to prose's expected block); F11 is a minor stderr-completeness gap.
- **Net new findings (F7-F11):** 5 findings — 4 doc-update (F7/F8/F10/F11), 1 gray-area-leaning-toolkit-fix (F9).
- **Total findings F1-F11:** 11. P2 batched-fix scope estimate (post-F1-F6 P1b lock): ~35-45 LOC chapter-45 + recipes 4/6/8 (~10-15 LOC) + chapter-45 BSMS spec rewrite (~10-15 LOC) + chapter-41 stderr disclosure paragraph (~5 LOC) + 1-2 new FOLLOWUPs in `design/FOLLOWUPS.md` = ~70-90 LOC total IF F9 lands as doc-update; +~20 LOC toolkit `cmd/export_wallet.rs` + new test cell IF F9 lands as toolkit-fix.

**P1a complete.** Next step: dispatch P1b architect on F7-F11 (chapter-30/39/41 batch) — same opus dispatch pattern as F1-F6 — with explicit Q7 lock on F9 c1 vs c2 reasoning chain. The c2 decision determines whether this cycle promotes to `mnemonic-toolkit-v0.28.2` paired tag per Q8.
