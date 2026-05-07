# v0.8 Phase 10 — Final cumulative ship review

**Verdict:** No critical bugs. Two Important findings (both doc-tracker gaps).

---

## Critical

None.

---

## Important

### I1 — Three CHANGELOG-listed FOLLOWUP IDs lack standalone entries (confidence 85)

**Files:** `CHANGELOG.md` §"FOLLOWUPS resolved this cycle" + `design/FOLLOWUPS.md`.

CHANGELOG lists these as resolved/re-tiered, but none has a standalone `### \`...\`` entry:
- `bip85-dice-application` (split product)
- `18-remaining-bip39-trezor-corpus-vectors` (Phase 8 deliverable)
- `bip85-rsa-rsa-gpg-applications` (split product, re-tiered)

**Fix:** add three standalone entries to `design/FOLLOWUPS.md`:
- `bip85-dice-application` — `resolved 1dde4dc` (v0.8 Phase 7), tier `v0.8`.
- `18-remaining-bip39-trezor-corpus-vectors` — `resolved 85694b2` (v0.8 Phase 8), tier `v0.7.1-carry`.
- `bip85-rsa-rsa-gpg-applications` — `open`, tier `v0.9 / pending-rsa-crate-stability`. Reopen criteria: rsa crate publishes patched stable release OR user requests with stated downstream use case.

### I2 — `DeriveChildUnsupportedApp` doc-comment still names `dice` as out-of-scope (confidence 82)

**File:** `crates/mnemonic-toolkit/src/error.rs:102-103`.

Phase 7 updated the runtime stderr message but left the doc-comment unchanged. After Phase 7 ships DICE, `dice` is in-scope; the doc-comment is stale.

**Fix:** rewrite to mention `rsa|rsa-gpg` only + reference RUSTSEC-2023-0071.

---

## Low

### L1 — SPEC paths still named `_v0_7.md` (cosmetic, no fix required)

In-place amendments are consistent with how `SPEC_export_wallet_v0_7.md` and `SPEC_convert_v0_6.md` are handled. References are accurate. No action.

---

## Verified-correct

1. **SemVer `0.8.0`** correct for the BIP-38 composite passphrase BREAKING change (pre-1.0 convention: minor = breaking-change axis; documented in CHANGELOG header).
2. **`[BREAKING]` tag visibility.** CHANGELOG `[0.8.0]` header carries `[BREAKING]`; dedicated `### [BREAKING]` section opens with verbatim migration sentence per R2-L3 plan-mandate.
3. **Test corpus claim 484 → 527 active +43 net** confirmed: 252 integration + 277 lib − 2 ignored = 527. Per-phase delta sum (+12 +22 +8 +6 net of Phase 8's −5 + 18 coverage) = +43.
4. **Per-phase FOLLOWUP traceability (11 of 14).** All 11 entries with standalone forms have correct commit hashes matching the per-phase commits. The 3 missing standalone entries are I1 above.
5. **Deferred entry re-tier.** `bip38-ec-multiplied-encrypt-mode-support` now `open`/`v0.8.1+`. Correct.
6. **`Cargo.toml` version.** `version = "0.8.0"`. Both new direct deps present (`sha3 = "0.10"`, `unicode-normalization = "0.1"`).

---

## Resolution actions applied

- **I1:** added 3 standalone FOLLOWUPS entries.
- **I2:** rewrote `DeriveChildUnsupportedApp` doc-comment.
