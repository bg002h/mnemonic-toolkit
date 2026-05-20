# P1b architect classification — F1-F6 (R0)

**Working SHA:** `c7baa3e` (post-AUDIT_FINDINGS commit; master HEAD at dispatch).
**Source-of-truth inputs:** `design/AUDIT_FINDINGS_manual_v0_28_0_content.md` (P1a output); `design/PLAN_manual_v0_2_0_content_audit.md` §1 Q7 + §7 P1b; `docs/manual/src/45-foreign-formats.md` (chapter under audit); `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` + `crates/mnemonic-toolkit/src/wallet_export/{sparrow,specter,coldcard,jade,electrum}.rs` (toolkit source for c1/c2).
**Worktree-isolation status:** dispatching toolkit root (no worktree assigned; per plan-doc Q11 fallback).

---

## Method

Each finding is locked against **two** ground-truth checks:
1. **Manual prose:** the recipe text at the cited line range in `docs/manual/src/45-foreign-formats.md`.
2. **Toolkit source:** the per-format emitter's `--template` requirement + the imported fixture's declared script-type (which determines the correct `--template` value).

Fixture script-types and the resulting correct `--template` values are read from the manual's per-format "Sniff signature / CLI invocation / Provenance metadata" prose (each section explicitly states the descriptor shape the fixture parses into), cross-validated against the per-format emitter's accept-set in `crates/mnemonic-toolkit/src/wallet_export/*.rs`.

For the gray-area F4 (c1 vs c2), I additionally read `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs` end-to-end — specifically `ColdcardEmitter::emit()` (the template-dispatched body at lines 42-55) and `emit_coldcard_multisig_text()` (lines 250-360+) — to establish whether the toolkit's existing `--format coldcard` path can semantically express a Coldcard-multisig text output today, and what `--format coldcard-multisig` would mean if added.

---

## F1 — Sparrow Round-trip example: export step missing required `--template`

**Classification (locked):** doc-update (mixed — includes collateral prose fix at L315-324).
**Confidence:** high.

**Fix spec:**

- **Anchor:** `docs/manual/src/45-foreign-formats.md:301-313` (the entire fenced `sh` block under `### Round-trip example`).
- **Imported fixture:** `sparrow-singlesig-p2wpkh.json` → descriptor `wpkh([fp/84'/0'/0']xpub.../<0;1>/*)` per L282 prose ("the `mk1` slot carries the single cosigner xpub and whose `md1` slot carries the descriptor `wpkh([5436d724/84'/0'/0']xpub6Bner3L3.../<0;1>/*)`"). Script type = BIP-84.
- **Locked `--template` value:** `bip84` (singlesig P2WPKH; matches the imported fixture's BIP-84 derivation per L282 + the SparrowEmitter's `--template` requirement at `wallet_export/sparrow.rs:104-108`).

**Diff (anchored to `c7baa3e`):**

```diff
 # Re-emit via export-wallet
 mnemonic export-wallet --from-import-json envelope.json \
-  --format sparrow > sparrow_re.json
+  --format sparrow --template bip84 > sparrow_re.json
```

**Collateral prose fix (load-bearing):**

The L315-324 deferral paragraph claims "The export-wallet side handles taproot via descriptor-passthrough; full taproot import support is queued as FOLLOWUP…". Reading `wallet_export/sparrow.rs:104-108`, the SparrowEmitter **unconditionally requires `--template`** — there is no descriptor-passthrough escape hatch on the Sparrow emit side. The "descriptor passthrough is not supported by Sparrow's file-import surface" stderr message confirms this (the L106 error literal). The prose is therefore misleading.

**Anchor:** `docs/manual/src/45-foreign-formats.md:317-324`.

**Diff (anchored to `c7baa3e`):**

```diff
 ### Deferral — taproot import

 Sparrow's emit side ships taproot wallets as *descriptor-passthrough*
 (concrete `[fp/path]xpub` keys embedded in
 `defaultPolicy.miniscript.script` instead of `@N/**` placeholders).
 v0.28.0's Sparrow *parse* path refuses any blob whose script contains
 `tr(` with the byte-exact error `error: import-wallet: sparrow:
-taproot scripts are not yet supported …` (exit 2). The export-wallet
-side handles taproot via descriptor-passthrough; full taproot import
-support is queued as FOLLOWUP `sparrow-taproot-descriptor-passthrough-import-support`.
+taproot scripts are not yet supported …` (exit 2). The export-wallet
+side requires a recognized `--template` (no descriptor-passthrough);
+taproot-multisig emit is supported via `--template tr-multi-a` /
+`tr-sortedmulti-a`. Full taproot **import** support (parsing
+Sparrow's descriptor-passthrough-on-emit shape) is queued as
+FOLLOWUP `sparrow-taproot-descriptor-passthrough-import-support`.
```

---

## F2 — Specter Round-trip example: export step missing required `--wallet-name`

**Classification (locked):** doc-update.
**Confidence:** high.

**Fix spec:**

- **Anchor:** `docs/manual/src/45-foreign-formats.md:369-374` (the fenced `sh` block under Specter's `### Round-trip example`).
- **Toolkit grounding:** `wallet_export/specter.rs:30-38` — `SpecterEmitter::collect_missing()` returns `MissingField::WalletName` iff `wallet_name_was_user_supplied == false`. The `--from-import-json` consumer path at `cmd/export_wallet.rs:590-593` defaults to `"imported-descriptor"` but explicitly sets `wallet_name_was_user_supplied = args.wallet_name.is_some()` (L610), so the user MUST supply `--wallet-name` for Specter to pass the §4 missing-info channel.
- **Locked `--wallet-name` value:** any placeholder string. SPEC §13 R1-L1 just requires non-empty; the imported Specter fixture's `label` field is the natural choice. For docs, a literal placeholder like `--wallet-name "Specter re-export"` works without committing to a fixture-specific string (we don't byte-compare in the recipe — the closing `diff` step compares JSON content via `jq -S` per the pattern in F1).

**Diff (anchored to `c7baa3e`):**

```diff
 mnemonic import-wallet --format specter \
   --blob specter-singlesig-p2wpkh.json --json > envelope.json
 mnemonic export-wallet --from-import-json envelope.json \
-  --format specter > specter_re.json
+  --format specter --wallet-name "Specter re-export" > specter_re.json
```

**Collateral prose fixes:** none required at this anchor. The L376-378 "blockheight is preserved in the provenance metadata but DROPPED on the canonicalize-side comparison" paragraph remains accurate.

---

## F3 — Coldcard single-sig Round-trip example: export step missing required `--template`

**Classification (locked):** doc-update.
**Confidence:** high.

**Fix spec:**

- **Anchor:** `docs/manual/src/45-foreign-formats.md:445-450`.
- **Imported fixture:** `coldcard-singlesig-bip84-mainnet.json` → descriptor `wpkh([B8688DF1/84'/0'/0']xpub6FQya7zGhR9.../<0;1>/*)` per L427-428 prose ("For the BIP-84 mainnet fixture, the result is a watch-only bundle whose descriptor is `wpkh([B8688DF1/84'/0'/0']xpub6FQya7zGhR9.../<0;1>/*)`"). Script type = BIP-84.
- **Locked `--template` value:** `bip84`. The ColdcardEmitter's singlesig-JSON path at `wallet_export/coldcard.rs:111-115` requires `--template` ∈ {bip44, bip49, bip84} (taproot is refused at the emit body's L120 guard).

**Diff (anchored to `c7baa3e`):**

```diff
 mnemonic import-wallet --format coldcard \
   --blob coldcard-singlesig-bip84-mainnet.json --json > envelope.json
 mnemonic export-wallet --from-import-json envelope.json \
-  --format coldcard > coldcard_re.json
+  --format coldcard --template bip84 > coldcard_re.json
```

**Collateral prose fixes:** none required at this anchor.

---

## F4 — Coldcard multisig Round-trip example: format-name asymmetry (c1 vs c2 lock)

**Classification (locked):** **mixed** — doc-update for the recipe (c1) PLUS a NEW FOLLOWUP filed at P2 to track c2 (export-side `coldcard-multisig` alias) as a future v0.28.2-or-v0.29.0 ergonomic surface fix.
**Confidence:** high.

### Reasoning chain — c1 vs c2

The asymmetry is **NOT intrinsic to coldcard's file-format surface** in the way the audit doc's tentative-c1 default frames it. Reading `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs`:

- `CliExportFormat::Coldcard` dispatches to `ColdcardEmitter::emit()` (L42-55).
- `ColdcardEmitter::emit()` **dispatches on template**: lines 44-55 — multisig templates (`WshMulti` / `WshSortedMulti` / `ShWshMulti` / `ShWshSortedMulti` / `TrMultiA` / `TrSortedMultiA`) route to `emit_coldcard_multisig_text()` (line 254+); singlesig templates route to `emit_coldcard_generic_json()` (line 105).
- The toolkit therefore **already emits a Coldcard-multisig-text wire-shape today** via `--format coldcard --template wsh-sortedmulti` (or any other multisig template).

So the semantics of a hypothetical `--format coldcard-multisig` value on `export-wallet` would be:
- **(c2 interpretation A — strict alias):** alias for `--format coldcard` that REQUIRES a multisig template (rejecting bip44/49/84/86 with a refusal pointer). The dispatch body would be a `template.is_multisig()` precheck followed by delegation to `ColdcardEmitter::emit()` (which would route to `emit_coldcard_multisig_text` automatically given the multisig template).
- **(c2 interpretation B — inference-eliminating alias):** alias that does NOT require `--template` and infers a default multisig template (e.g., `wsh-sortedmulti`) from the descriptor's script-type when `--from-import-json` is used. This is the user-ergonomic option since `--from-import-json` already supplies the descriptor.

Both c2 interpretations are defensible. Neither is the "no-op alias if there's no multisig coldcard export format" case the audit doc speculated about — the format DOES exist, the toolkit DOES emit it, just not under the symmetric flag name.

**c1 is the locked classification for THIS cycle** because:

1. **Surface-enlargement is a clap-derive change touching the GUI schema-mirror invariant** (CLAUDE.md "Mirror invariant"). A toolkit-fix for F4 would require a paired `mnemonic-gui/src/schema/mnemonic.rs` update or trip the schema_mirror drift gate at the next GUI pin bump. This cycle is scoped as `manual-v0.2.0` (manual-only); Q8 (release shape lock) defaults to manual-only with toolkit-bin promotion ONLY if a behavioral bug is found. The asymmetry is an ergonomic/surface gap, not a behavioral bug — the binary CAN emit the right output, just under a different flag.
2. **The doc fix is fully sufficient for round-trip parity.** The user-facing recipe can demonstrate `--format coldcard --template wsh-sortedmulti --threshold 2` and produce a byte-near-equivalent output to the import fixture; the recipe teaches the asymmetry explicitly, and the recipe IS the audit artifact (Q5 "transcripts-as-audit" lock).
3. **c2 raises a separate UX question:** should the export side accept BOTH `coldcard` and `coldcard-multisig` as flag values, with one rejecting singlesig templates and one rejecting multisig templates? That's a flag-grammar design decision that warrants its own architect review with Q7 promote-to-toolkit-fix framing — not in this cycle's R3-locked plan.

**Lock: c1 for this cycle + FOLLOWUP filed at P2 for c2 (deferred).**

### Fix spec (c1 — locked)

- **Anchor:** `docs/manual/src/45-foreign-formats.md:528-534`.
- **Imported fixture:** `coldcard-ms-2of3-p2wsh-with-xfp.txt` → descriptor `wsh(sortedmulti(2, [34A3A4F1/48'/0'/0'/2']xpub6FQya..., ...))` per L495-496 prose. Script type = wsh sortedmulti.
- **Locked `--template` value:** `wsh-sortedmulti` (the imported fixture's expected template). Per `wallet_export/coldcard.rs:46-52` (the multisig-template dispatch) + `emit_coldcard_multisig_text()` at L274-278 (`P2WSH` for wsh templates).
- **Locked `--threshold` value:** `2` (the K of `2-of-3` in the fixture).
- **Locked format:** swap `--format coldcard-multisig` (rejected by export's clap-parser) → `--format coldcard` (which dispatches correctly to the multisig text emitter via the template-dispatch logic).

**Diff (anchored to `c7baa3e`):**

```diff
 mnemonic import-wallet --format coldcard-multisig \
   --blob coldcard-ms-2of3-p2wsh-with-xfp.txt --json > envelope.json
 mnemonic export-wallet --from-import-json envelope.json \
-  --format coldcard-multisig > coldcard_ms_re.txt
+  --format coldcard --template wsh-sortedmulti --threshold 2 \
+  > coldcard_ms_re.txt
 diff coldcard-ms-2of3-p2wsh-with-xfp.txt coldcard_ms_re.txt
```

**Collateral prose fix (REQUIRED — load-bearing):**

The asymmetry needs an explicit narrative paragraph just below the recipe so future readers don't trip on the same flag-name mismatch. Suggested addition immediately after L534 (the closing `diff` line of the fenced block):

```diff
 diff coldcard-ms-2of3-p2wsh-with-xfp.txt coldcard_ms_re.txt
 ```

+> **Format-name asymmetry note.** `--format coldcard-multisig` is
+> accepted only on the **import** side (sniffs Coldcard's text
+> multisig setup file). On the **export** side, `--format coldcard`
+> emits Coldcard-multisig text when paired with a multisig
+> `--template` (e.g., `wsh-sortedmulti`) — see SPEC v0.8 §5.2. The
+> single `coldcard` export value covers both single-sig JSON
+> (singlesig templates) and multisig text (multisig templates);
+> tracked for export-side flag-name alignment as FOLLOWUP
+> `export-wallet-coldcard-multisig-alias` (filed at manual-v0.2.0 P2).
```

### New FOLLOWUP to file at P2 (c2 deferred)

- **Slug:** `export-wallet-coldcard-multisig-alias`
- **Tier:** `v0.28+-ergonomic-surface`
- **Body:** `mnemonic export-wallet --format` accepts `coldcard` (template-dispatched: singlesig→generic JSON, multisig→multisig text) but does NOT accept `coldcard-multisig` despite the import side accepting both `coldcard` and `coldcard-multisig`. Filing as future ergonomic-surface fix: add `coldcard-multisig` as a `CliExportFormat` variant that aliases to `Coldcard` with a multisig-template precheck (refusal pointer for singlesig templates). Surfaces parity between the import and export flag value sets. Requires paired `mnemonic-gui/src/schema/mnemonic.rs` update per CLAUDE.md schema-mirror invariant.
- **Source citation:** This FOLLOWUP, P1b classification at `design/agent-reports/manual-v0_2_0-p1b-r0-classification.md` (F4 reasoning chain).

---

## F5 — Jade Round-trip example: export step missing required `--template`

**Classification (locked):** doc-update.
**Confidence:** high.

**Fix spec:**

- **Anchor:** `docs/manual/src/45-foreign-formats.md:592-597`.
- **Imported fixture:** `jade-multisig-2of3-p2wsh.json` — Jade's parse path delegates to coldcard-multisig parser per L552-555 + L575-576 prose. The fixture's `multisig_file` body is a 2-of-3 P2WSH coldcard-multisig text. Script type = wsh sortedmulti.
- **Locked `--template` value:** `wsh-sortedmulti`. Per `wallet_export/jade.rs:41-46` — JadeEmitter accepts WshMulti/WshSortedMulti/ShWshMulti/ShWshSortedMulti, delegating byte-identical to `emit_coldcard_multisig_text()`. The audit doc's suggestion of `wsh-sortedmulti` is correct.
- **Locked `--threshold` value:** `2` (K of `2-of-3`).

**Diff (anchored to `c7baa3e`):**

```diff
 mnemonic import-wallet --format jade \
   --blob jade-multisig-2of3-p2wsh.json --json > envelope.json
 mnemonic export-wallet --from-import-json envelope.json \
-  --format jade > jade_re.json
+  --format jade --template wsh-sortedmulti --threshold 2 \
+  > jade_re.json
```

**Collateral prose fixes:** none required at this anchor. The L599-605 SeedQR deferral paragraph remains accurate.

---

## F6 — Electrum Round-trip example: export step missing required `--template`

**Classification (locked):** doc-update.
**Confidence:** high.

**Fix spec:**

- **Anchor:** `docs/manual/src/45-foreign-formats.md:686-691`.
- **NOTE — fixture-name correction:** the AUDIT_FINDINGS.md F6 entry at line 92 cites the fixture as `electrum-multisig-2of3-wsh.json`, but the recipe-as-written at `45-foreign-formats.md:688` imports `electrum-standard-bip84-mainnet.json` (the singlesig fixture). P1a's fixture-name claim was a transcription error; the locked-classification fix targets the singlesig case as the recipe-as-written demands.
- **Imported fixture:** `electrum-standard-bip84-mainnet.json` → singlesig BIP-84 yielding `wpkh([5436d724/84'/0'/0']zpub6qTB.../<0;1>/*)` per L666 prose ("Singlesig BIP-84 yields `wpkh([5436d724/84'/0'/0']zpub6qTB.../<0;1>/*)`"). Script type = BIP-84.
- **Locked `--template` value:** `bip84`. Per `wallet_export/electrum.rs:52-56` + the `is_multisig()`-dispatched branches at L70-74 — Electrum accepts both singlesig and multisig templates.

**Diff (anchored to `c7baa3e`):**

```diff
 mnemonic import-wallet --format electrum \
   --blob electrum-standard-bip84-mainnet.json --json > envelope.json
 mnemonic export-wallet --from-import-json envelope.json \
-  --format electrum > electrum_re.json
+  --format electrum --template bip84 > electrum_re.json
```

**Collateral prose fixes:** none required at this anchor.

---

## P1b synthesis

- **Total findings:** 6 (F1-F6).
- **Doc-update count:** 6 (all 6 classifications lock to doc-update; F1 + F4 also carry collateral prose fixes; F4 also files a NEW FOLLOWUP `export-wallet-coldcard-multisig-alias` for c2 deferred).
- **Toolkit-fix count:** 0 (this cycle remains manual-only per Q8 default; no `mnemonic-toolkit-v0.28.2` promotion triggered).
- **Confidence breakdown:** 6 high-confidence; 0 medium; 0 low (no findings need P2 architect re-look — the c1 lock for F4 is exhaustive per the reasoning chain above, and the c2 alternative is deferred to its own FOLLOWUP rather than punted as ambiguous).
- **P2 batched-fix scope estimate:** ~25-35 LOC diff to `45-foreign-formats.md`: 6 export-step modifications (1-3 lines each) + 1 prose-paragraph rewrite at L317-324 (Sparrow taproot deferral) + 1 NEW format-name-asymmetry note paragraph below L534 (Coldcard multisig). Plus 1 new FOLLOWUP entry in `design/FOLLOWUPS.md` (`export-wallet-coldcard-multisig-alias`, ~10 LOC). Total ~35-45 LOC across two files.

### New findings spotted during this review

- **F1 collateral — Sparrow deferral prose contradicts emit behavior** (folded into F1 fix spec, not a separate finding). The L322-324 sentence "The export-wallet side handles taproot via descriptor-passthrough" is false; SparrowEmitter unconditionally requires `--template`. The corrected prose is part of F1's collateral fix.
- **F6 fixture-name correction** (P1a transcription error, not a new manual finding). AUDIT_FINDINGS.md F6 line 92 says fixture is `electrum-multisig-2of3-wsh.json` but the recipe imports `electrum-standard-bip84-mainnet.json`. F6's locked fix uses `--template bip84` accordingly.
- **No other chapter-45 issues spotted in this review pass.** The 8 sniff signature / parse contract / provenance metadata blocks under each format heading were not re-audited line-by-line at P1b — they are within P1a's scope and not under the "Round-trip example" finding class. P1a's `### Round-trip example` H3 enumeration was 100% complete (6/6 captured), and no additional H3 round-trip subsections exist in chapter-45.

### Risk flags for the remaining P1a captures (chapter-30/39/41)

Reading the export-wallet code surface during the F4 c1/c2 lock surfaced three failure-mode classes the user should watch for in the cross-format-conversion recipe captures (`docs/manual/src/30-workflows/39-cross-format-conversion.md`):

1. **`--account` rejection on `--from-import-json` (cmd/export_wallet.rs:521-527):** Any recipe that supplies `--account <N>` alongside `--from-import-json` will hit `BadInput("--account is meaningful only with --template / --descriptor; --from-import-json reads the account from the envelope")`. Watch for recipes that show `--account 0` for clarity but inadvertently break under `--from-import-json`.

2. **`--from-import-json` envelope's null descriptor (cmd/export_wallet.rs:551-561):** If a recipe's import step produces an envelope whose `bundle.descriptor` is null (e.g., a corner case in a parser that doesn't synthesize a descriptor), the export step will hit `BadInput("--from-import-json: envelope.bundle.descriptor is null; v0.27.0 wallet-import path always emits the descriptor string verbatim")`. v0.27.0+ parsers all emit a descriptor, but a doc-vs-code drift would surface here.

3. **`--wallet-name` requirement parity across recipes:** Any recipe whose export step targets `--format specter` (recipes 5 + possibly 7 per the audit doc's prose) will hit the same Specter `MissingField::WalletName` failure as F2. Watch for this when capturing recipe 5 (Specter→Bitcoin Core) and recipe 8 if it terminates at Specter.

Additionally, watch for **recipe-7 BSMS-form coupling** — the F4 recipe-as-written has an implicit BSMS-export terminal (per AUDIT_FINDINGS F4 prose at L74 "the conceptual round-trip closing out at the BSMS export level via recipe 7's pattern"). If recipe 7 is the canonical BSMS-export-terminating recipe for multisig conversion, capture its `--bsms-form` argument explicitly — `4-line` is the default per `cmd/export_wallet.rs:147`, but the captured transcript should make the form choice visible for SPEC §3.5 traceability.

---

**Persisted at:** `design/agent-reports/manual-v0_2_0-p1b-r0-classification.md` (this file).
**Next phase:** plan-doc fold of locked classifications → resume P1a captures for chapter-30/39 + chapter-41 → batched P2 fix-in-cycle pass.
