# Holistic end-of-cycle architect review — manual-v0.2.0

**Verdict: YELLOW — 1 Important finding (I1) requires inline fold before tagging `manual-v0.2.0`.**

**Dispatched:** 2026-05-20 at master HEAD `fe32e9e`.
**Reviewer model:** opus.
**Cycle scope:** P0 → P5.1 (10 commits past `8977389` v0.28.1 baseline) plus the paired `mnemonic-toolkit-v0.28.2` tag at `615b10e`.

## Summary

Cycle work is high-quality across cross-phase consistency, F9 toolkit fix correctness, P3 CI wiring, FOLLOWUP partition integrity, and AUDIT_FINDINGS doc consistency. F1–F11 fix specs from P1b R0/R1 are faithfully reflected in P2c. F9 c2-B at `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:566-598` (commit `615b10e`) matches the R1 spec byte-for-byte: `descriptor_body_no_csum` validates+strips, `MsDescriptor::from_str` parses the body (BIP-380 makes the `#csum` suffix optional on parse), then `parsed_ms.to_string()` re-emits via miniscript's canonical `Display` which always appends `#<8-char-csum>` per BIP-380 §Checksum-on-emit. 2 new regression cells (`f9_from_import_json_bsms_l2_carries_bip380_checksum`, `f9_from_import_json_specter_descriptor_carries_bip380_checksum`) at `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs:1020-1099` lock the invariant.

P3 wiring is robust: `make audit = lint + verify-examples`; `lint` has `figures-cache-verify` as prerequisite (no orphan-cache drift); manual.yml builds the debug binary then runs `make audit` against the real `MNEMONIC_BIN`; SKIP_STEMS carve-out is documented + tracked via `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh` FOLLOWUP with explicit removal-trigger note. Tag `mnemonic-toolkit-v0.28.2` is at `615b10e` which IS the F9 fix commit.

FOLLOWUP partition is clean: `manual-yml-bind-real-mnemonic-bin` carries `resolved-partial 52f33f7`; two named successors (`manual-md-bin-real-binary-promote`, `manual-ms-bin-real-binary-promote`) filed with explicit forward-pointer + `Companion:` cross-citation per `project_v0_7_1_batch_b2_closed` precedent. `inheritance-example-transcript-coverage` flipped to `resolved f46ac70` + `52f33f7` with the FOLLOWUP-body path-correction note.

## Important finding (inline-foldable; gates tag)

**I1 — chapter-45 L436 + L504 carry uppercase fingerprints in toolkit-output descriptor prose; same F10 drift class as the chapter-39 L199 fix that DID land.**

- **Where:**
  - `docs/manual/src/45-foreign-formats.md:436` — claims toolkit output descriptor is `wpkh([B8688DF1/84'/0'/0']xpub6FQya7zGhR9.../<0;1>/*)`
  - `docs/manual/src/45-foreign-formats.md:504` — claims parser synthesizes `wsh(sortedmulti(2, [34A3A4F1/48'/0'/0'/2']xpub6FQya..., ...))`
- **Why this is a real finding:** the toolkit lowercases fingerprints unconditionally on emit (`crates/mnemonic-toolkit/src/wallet_import/json_envelope.rs:284`: `fingerprint.to_string().to_lowercase()`). All in-cycle captured transcripts confirm lowercase output (e.g., `docs/manual/transcripts/cross-format-recipes/recipe-6-coldcard-singlesig-to-bip388.out:4` — `[b8688df1/84'/0'/0']xpub6FQya...` lowercase).
- **Audit trail:** P1b R1 classification (`design/agent-reports/manual-v0_2_0-p1b-r1-classification.md:339`) explicitly flagged this as the "F10b candidate" and risk flag #2 (`:446`), recommending "Defer to P2 as a discretionary collateral fix." Commit `5d2c0a6` (P2c F1-F11 manual edits) did NOT touch L436 or L504. The source fixture (`crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-singlesig-bip84-mainnet.json:3`) IS uppercase `B8688DF1`, but the prose is describing the parser's OUTPUT descriptor (lowercase per toolkit canonicalization), not the source fixture string.
- **Suggested diff:**

```diff
--- a/docs/manual/src/45-foreign-formats.md
+++ b/docs/manual/src/45-foreign-formats.md
@@ -436 +436 @@
-whose descriptor is `wpkh([B8688DF1/84'/0'/0']xpub6FQya7zGhR9.../<0;1>/*)`
+whose descriptor is `wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR9.../<0;1>/*)`
@@ -504 +504 @@
-`wsh(sortedmulti(2, [34A3A4F1/48'/0'/0'/2']xpub6FQya..., ...))` and
+`wsh(sortedmulti(2, [34a3a4f1/48'/0'/0'/2']xpub6FQya..., ...))` and
```

- **Confidence:** 90.
- **Note on `docs/manual/src/30-workflows/37-wallet-export.md:206`:** the uppercase `B8688DF1: xpub6FQya...` on that line is a Coldcard text-file format sample (with `XFP:` header context) where uppercase IS faithful to the fixture format. That line should NOT be changed. Only the descriptor-output claims at chapter-45 L436 + L504 are drift.

## Minor (non-blocking; surface for awareness)

**M1 — `design/AUDIT_FINDINGS_manual_v0_28_0_content.md` does not carry post-P1b/P2 closure status per finding.** The doc remains as captured at P1a (with tentative classifications). The authoritative closure record lives in the P1b R0/R1 classification reports + the P2c commit message + the FOLLOWUPS entries. This is acceptable per cycle convention (audit-findings is the P1a artifact, not the resolution ledger) but a closure-status appendix to the AUDIT_FINDINGS doc would aid future readers. Defer; do not gate tag.

## Path to GREEN

Fold I1 inline (2-line diff to `docs/manual/src/45-foreign-formats.md`), commit as a single P5.2 patch on top of `fe32e9e`, then re-dispatch this architect for a confirmation round. After 0C/0I confirmation, `manual-v0.2.0` tag is clear. The `mnemonic-toolkit-v0.28.2` tag at `615b10e` is independently shippable; tag lockstep is not strictly required since CHANGELOG and tests are self-contained for the toolkit patch (per memory `project_v0_28_1_patch_shipped` toolkit-only precedent).

## Files reviewed

- `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/45-foreign-formats.md` (I1 fix sites L436, L504)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/export_wallet.rs` (F9 fix verified)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs` (F9 regression cells verified)
- `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/tests/verify-examples.sh` (P3 wiring verified)
- `/scratch/code/shibboleth/mnemonic-toolkit/.github/workflows/manual.yml` (P3 CI wiring verified)
- `/scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md` (partition integrity verified at L2006-2014, L2682-2710)
- `/scratch/code/shibboleth/mnemonic-toolkit/design/agent-reports/manual-v0_2_0-p1b-r1-classification.md` (F1-F11 spec source; risk flag #2 surfaced I1)
