# Phase 4 — feature-dev:code-architect review, round 1

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (Phase 4 author commit)
**Verdict:** Not converged. 3 critical / 3 important / 3 nits.

## Critical

### C-1 — ch33: `--taproot-internal-key` is a phantom flag on `mnemonic bundle` and `mnemonic verify-bundle`

`--taproot-internal-key` exists only on `mnemonic export-wallet` (per `cli-help/mnemonic-export-wallet.txt` and `src/cmd/export_wallet.rs`). The two `mnemonic bundle` invocations and the verify-bundle claim in ch33 fail with "unexpected argument `--taproot-internal-key`". Verified empirically: `mnemonic bundle --template tr-sortedmulti-a --threshold 2 --taproot-internal-key nums …` → exit 2, "error: unexpected argument".

**Fix applied:** Rewrote ch33 §"Step 1" + §"Step 2":
- NUMS variant: drop the flag entirely; `--template tr-sortedmulti-a` defaults to NUMS internal key.
- Designated-cosigner variant: use `--descriptor 'tr(@N,sortedmulti_a(K,@0,…))'` to promote a cosigner explicitly.
- Cross-reference §"Taproot multisig export" in ch37 for `export-wallet`'s `--taproot-internal-key <nums|@N>` toggle.

### C-2 — ch38: BIP-85 RSA-GPG application code is wrong

Chapter said: "RSA and RSA-GPG (BIP-85 codes 828365' and 707785')". `707785'` is `password-base85` (already correctly listed as in-scope two sentences earlier). RSA-GPG falls under code `828365'`.

**Fix applied:** Rewrote sentence to "RSA and RSA-GPG (both under BIP-85 code 828365')". Also corrected the in-scope count from 6 to 7 (DICE = 89101' is the seventh).

### C-3 — ch37: BIP-388 `keys_info` example shows BIP-48 paths but default is BIP-87

Default is `--multisig-path-family bip87` (cli-help line 28). BIP-87 → `m/87'/0'/0'`; BIP-48 → `m/48'/0'/0'/2'`.

**Fix applied:** Changed example paths to `87h/0h/0h`. Added parenthetical noting `--multisig-path-family bip48` for Coldcard / SeedSigner / older-Sparrow compatibility.

## Important

### I-1 — ch32: intro overpromises air-gapped property

Intro claimed "no single device ever sees more than one cosigner's secret" — directly contradicted by the coordinated flow described next.

**Fix applied:** Qualified the claim: in the coordinated flow one laptop sees all three phrases briefly during the bundle pass; the air-gapped variant (described at the end) preserves the no-shared-secret property.

### I-2 — `cli-help/mnemonic-derive-child.txt` transcript stale: `dice` listed as out-of-scope

Source docstring in `crates/mnemonic-toolkit/src/cmd/derive_child.rs` still says "3 out-of-scope tokens (`rsa`, `rsa-gpg`, `dice`)" but v0.8 promoted dice to in-scope. The chapter is correct; the source doc-comment is stale.

**Action:** This is a toolkit source-code issue, not a manual issue. Filed in `mnemonic-toolkit/design/FOLLOWUPS.md` for a v0.8.1 patch (out of manual scope).

### I-3 — Anchor `#appendix-d-descriptors-and-bip-388-primer` claim is FALSE

Reviewer claimed the anchor needs double hyphen because of em-dash. Verified empirically: `pandoc -f markdown -t html` on `# Appendix D — Descriptors and BIP-388 primer` emits `id="appendix-d-descriptors-and-bip-388-primer"` (single hyphen). The chapter's anchor is correct.

The leftover Phase 1 r2 fix to AUTHORING.md was incomplete: the example link was fixed but the prose rule still claimed double-dash. **Fix applied:** AUTHORING.md prose now states the correct slug rule (em-dashes are non-alphanumerics, dropped; runs of whitespace collapse to single `-`).

## Minor / nits

### N-1 — ch38: dice `--length` upper bound `1..=10000` is undocumented in code

Source (`derive_child.rs`) only checks `rolls < 1`; no ceiling. Filed for FOLLOWUPS as a toolkit-side fix (either add a cap or drop the upper-bound from the chapter's table).

### N-2 — ch32 air-gapped step has redundant `--account 0`

Default; deferred to a later polish pass.

### N-3 — ch35 BCH error output illustrative, not transcribed

Filed for FOLLOWUPS — replace with actual transcript line before PDF freeze.

## Convergence assessment

After applying fixes for C-1, C-2, C-3, I-1, and I-3 (the false-positive that exposed AUTHORING.md residue), Phase 4 is at 0C/0I. I-2 and N-1 are upstream toolkit issues filed as FOLLOWUPS. N-2 and N-3 deferred to FOLLOWUPS. No round-2 dispatch needed.
