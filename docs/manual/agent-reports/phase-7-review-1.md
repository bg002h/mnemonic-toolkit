# Phase 7 — feature-dev:code-reviewer review, round 1

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (Phase 7 author commit)
**Verdict:** Not converged. 2 critical / 4 important / 0 nits.

## Critical

### C-1 — Troubleshooting `--format sparrow / specter` directs users to error

`67-troubleshooting.md:45` recommended `--format sparrow` / `--format specter` for native shapes — both are stub refusals in v0.8 (`ExportWalletFormatStub`). Users following the matrix would hit an error.

**Fix applied:** Replaced with: stub deferral noted; recommend `--format bip388` (or `--format bitcoin-core`) and importing the JSON via the wallet's BIP-388 / descriptor-import dialog.

### C-2 — Release-history: `md-codec 0.16.0` date wrong + highlights misattributed

`68-release-history.md` had:
- `md-codec 0.16.0 | 2026-05-07 | v0.16 cycle: BIP test-vector audit matrix.`

Wrong on both counts: 0.16.0 shipped 2026-05-03 (CLI extraction to `md-cli`); the audit-matrix work was 0.16.2 (2026-05-07).

**Fix applied:** Split into two correct rows and corrected highlights.

## Important

### I-1 — BIP-39 wordlist 4-letter-prefix example wrong

`62-bip39-primer.md` cited "abandon" / "ability" as differing at the 4th character. They differ at the 3rd (`a-b-a` vs `a-b-i`). The 4-letter uniqueness property is real; the example was wrong.

**Fix applied:** Dropped the misleading example and stated the property directly: "the first four letters identify any word unambiguously."

### I-2 — BIP-32 hardened-derivation formula missed `0x00` prefix byte

`63-bip32-primer.md` showed `HMAC-SHA-512(cc, privkey || (i + 2^31))`. Per BIP-32, hardened derivation prepends `0x00` to make the HMAC input length-match a compressed-pubkey hex (33 bytes) and the index inside HMAC is `ser32(i)` (the `+2^31` is the *child label*, not what's inside the HMAC).

**Fix applied:** Updated formula to `HMAC-SHA-512(cc, 0x00 || privkey || ser32(i))` with the resulting child labelled `i + 2^31`. Added one-sentence note about why the `0x00` byte exists.

### I-3 — Release-history missed `ms-codec 0.1.1`

ms-codec 0.1.1 (2026-05-07, v0.7.1 audit-cycle patch) was missing from the ms section.

**Fix applied:** Added explicit row: `ms-codec 0.1.1 | 2026-05-07 | v0.7.1 audit-cycle: extra corpus vectors; BIP-93 §"Test vectors" cross-format pinning.`

### I-4 — `--force-long-code` described as working debug flag

`65-bch-codex-primer.md` mentioned `--force-long-code` as a debug flag. Per md-codec CHANGELOG v0.12.0, the flag has been a documented no-op since long-code mode was dropped on the codec side.

**Fix applied:** Rewrote: "no user flag is needed; `md encode --force-long-code` is a forward-compat scaffold but is a documented no-op since md-codec v0.12.0."

## Verification of correct elements

| Check | Status |
|---|---|
| Test seed DANGER admonition (66) | OK |
| BIP-39 PBKDF2 parameters: 2048 iter, HMAC-SHA-512, "mnemonic" salt | OK |
| codex32 correction radius "up to 4 substitution errors" | OK (BIP-93) |
| mc-codex32 retirement note (65) | OK (vs descriptor-mnemonic CLAUDE.md) |
| BIP-388 placeholder semantics (64) | OK |
| BIP-44/49/84/86/87/48 path table (63) | OK |
| mk-codec 0.2.0 / 0.1.0 dates (68) | OK |
| Cross-repo coordination note (68) | OK |
| BIP-85 RSA/RSA-GPG deferral (67) | OK |
| `--dice-sides` (67) | OK |

## Convergence assessment

After applying C-1 + C-2 + I-1 + I-2 + I-3 + I-4, Phase 7 is at 0C/0I. No round-2 dispatch needed.
