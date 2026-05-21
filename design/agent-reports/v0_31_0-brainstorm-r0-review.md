# v0.31.0 brainstorm R0 review (dispatched at Cycle 6b execute-start)

**Reviewer:** opus
**Round:** R0 (deferred from 6a per the user-locked scope cut)
**Specs under review:**
- design/BRAINSTORM_v0_31_0_electrum_encrypted.md
- design/cycle-6-p0-recon.md
- design/CYCLE_6_KICKOFF.md
**Date:** 2026-05-21

## Critical (C)

### C1 — Parser-integration design rests on a sensitive-field set the parser DOES NOT READ

**Location:** brainstorm §"Decisions locked" item 4 (line 38); KICKOFF Phase 3 (lines 56-60); P0 §A2 ("sensitive fields like `seed: <base64-ciphertext>`").

**Finding.** The brainstorm specifies pre-decryption of `seed`, `xprv`, `keystore.seed`, `keystore.xprv` "etc." But the toolkit's existing Electrum parser at `crates/mnemonic-toolkit/src/wallet_import/electrum.rs` (current HEAD `1724477`) reads ONLY these JSON paths:
- Singlesig: `keystore.xpub` (L494), `keystore.derivation` (L504), `keystore.root_fingerprint` (L514), `keystore.label` (L531).
- Multisig: per-cosigner `xN/.xpub`, `xN/.derivation`, `xN/.root_fingerprint`, `xN/.label` (L778-816).
- Top-level: `seed_version` (L285), `wallet_type` (L318), `use_encryption` (L306).

Per Electrum's `electrum/keystore.py` (verified via webfetch 2026-05-21), the field-level encryption (`pw_encode_bytes`) protects ONLY `seed`, `xprv`, `passphrase`, `keypairs`. Plaintext under both encrypted and unencrypted wallets: `xpub`, `derivation`, `root_fingerprint`, `label`, `type`, `seed_version`, `wallet_type`.

**Implication.** The parser-needed field set ∩ encrypted-field set = ∅. After the orchestrator "decrypts in place" there is literally nothing to decrypt at any path the parser inspects. The current `use_encryption: true` refusal at electrum.rs:305-313 is therefore over-restrictive in principle — the wallet's watch-only public-key material is already accessible WITHOUT a password.

Two valid paths forward, neither matches the brainstorm-as-written:

- **Path A (recommended):** drop `--decrypt-password*` entirely; instead, downgrade the L305-313 refusal to: parse plaintext xpub/derivation/fingerprint/label and ignore the encrypted `seed`/`xprv` fields. Emit stderr advisory "wallet is encrypted; only watch-only material is imported." This is the architecturally correct fix and ships with NO new CLI flags, NO new deps beyond what 6a already added (which are now exclusively for the optional symmetric encrypt-side test fixture).
- **Path B:** preserve `--decrypt-password*` but justify it as future-proofing — and document loudly in the plan-doc + manual that the password is OPTIONAL even when `use_encryption: true`, and that supplying a wrong password is informational-only (since the toolkit never needs the seed material). This is awkward UX.

The brainstorm currently asserts (item 5) that absent-password + `use_encryption: true` triggers refusal. Under either Path A or Path B that refusal is no longer warranted. The plan-doc MUST resolve this before Phase 3 dispatch.

Source SHA cited: master HEAD `d890de4` (post-6a). Grep-verified at electrum.rs:494, 504, 514, 531, 778, 305-313.

### C2 — Field-set citation in the brainstorm is unverified and partially wrong

**Location:** brainstorm §"Decisions locked" item 4: "decrypt the top-level fields the parser actually reads (`seed`, `xprv`, `keystore.seed`, `keystore.xprv`, etc.)".

**Finding.** Per Electrum's keystore.py: `seed` and `xprv` are NESTED keystore subfields, never top-level (the brainstorm has the nesting backwards — it says "top-level fields like `seed`" but they live at `keystore.seed`). Additionally, `passphrase` and `keypairs` are missing from the enumeration despite also being encrypted. Cross-reference with P0 §A2 which says "sensitive fields like `seed`" — same defect propagated from P0 → brainstorm.

This is Critical (not Important) because the field-set enumeration drives the orchestrator's pre-decrypt loop. Wrong enumeration → either over-decrypts (passes wrong-typed plaintext to parser; no obvious symptom because parser never reads those keys; security non-issue but the password verification semantics drift) or under-decrypts. Combined with C1 it makes the entire Phase 3 design suspect.

**Required fix.** Resolve C1 first. If Path B prevails, the field-set enumeration must come from `electrum/keystore.py` (not "etc.") and the plan-doc must lock the exact path set: `keystore.seed`, `keystore.xprv`, `keystore.passphrase`, `keystore.keypairs` for singlesig; `xN/.seed`, `xN/.xprv`, etc. for multisig. None of these are parser-consumed; they exist only for password-validation purposes (decrypt-and-discard).

## Important (I)

### I1 — Password-validation strategy is unspecified

**Location:** brainstorm §"Decisions locked" items 4-6; KICKOFF Phase 3.

If C1 Path B is chosen, supplying a wrong password under the current design produces: silent acceptance (because parser-needed fields are plaintext) UNTIL the parser hits the still-encrypted `seed` field — which it never does. The user sees a successful import with an undetected wrong password. To validate the password, the plan-doc MUST specify a synthetic decrypt-and-verify step (e.g., decrypt `keystore.seed` if present + check UTF-8 / PKCS7 strip success). This is missing entirely from the brainstorm.

### I2 — 3-form `--decrypt-password{,-file,-stdin}` is a NET-NEW pattern; existing precedent is 2-form

**Location:** brainstorm §"Decisions locked" item 2 (lines 32-36).

Verified via grep: bundle.rs:56 has `--passphrase` + `--passphrase-stdin` (2-form). No `--passphrase-file` precedent exists in toolkit. Adding `-file` is a NET-NEW pattern that should be either (a) justified explicitly in the brainstorm (industry parity? user request? security ergonomics?) OR (b) downgraded to 2-form for precedent consistency. The brainstorm asserts "User picked 3-form" but provides no in-doc rationale.

If the 3-form survives, this is a precedent-setting decision that the plan-doc should explicitly call out as such (and probably file a FOLLOWUP to backfill `--passphrase-file` / `--bip38-passphrase-file` for parity).

### I3 — FOLLOWUP body update at close is under-specified

**Location:** brainstorm §"FOLLOWUP closure semantics" (lines 159-167).

Per CLAUDE.md "Plan-doc + spec citations are grep-verified at write time" + the post-merge line-decay convention, the FOLLOWUP body correction ("PBKDF2 + AES-CBC" → "sha256d + AES-256-CBC") needs (a) the exact slug — `wallet-import-electrum-encrypted` — confirmed against `design/FOLLOWUPS.md` HEAD; (b) line-number citation refreshed at write-time; (c) a "corrected at Cycle 6 P0 recon (`design/cycle-6-p0-recon.md` §A1)" note. The brainstorm has (c) but not (a) + (b).

P0 recon §A1 cites L2576 — verify against `d890de4` at plan-doc write time.

### I4 — Brainstorm self-defers R0 review; the deferral now executes — but the brainstorm hasn't been updated post-6a

**Location:** brainstorm §"R0 review scope" (lines 147-155).

The brainstorm shipped with 6a explicitly stating "If review surfaces issues, they fold inline before 6b dispatch." This R0 IS that review. Findings C1-C2 require a brainstorm UPDATE (not just plan-doc folds) because they invalidate locked decisions 4 + 5 + 6. The fold must rev the brainstorm (in-place edit + new "Cycle 6b R0 findings folded" preamble note) before the plan-doc is written.

### I5 — SemVer-MINOR justification is correct, but only if the CLI surface ships

**Location:** brainstorm §"Decisions locked" item 7 + KICKOFF §"Cycle 6 scope" (line 22).

If C1 Path A prevails (no new CLI flags), v0.31.0 becomes a PATCH (`v0.30.1`), not a MINOR. The brainstorm's MINOR-bump rationale ("password-on-argv is MINOR per architect I3 policy IF passed inline") doesn't apply when no password flag exists. Plan-doc must re-derive the version bump after C1 resolution. The GUI lockstep paired version (`v0.16.0`) similarly depends on whether the schema-mirror gains new entries.

## Minor (M)

### M1 — `electrum_crypto.rs` `derive_key` invariant comment

The library's `derive_key` returns `Zeroizing<[u8; 32]>`, but the doc-comment doesn't note that the caller's password buffer is NOT zeroized by this function (caller's responsibility). Cosmetic; the call sites in 6b can wrap in `Zeroizing<Vec<u8>>` per KICKOFF Phase 2 line 51. Worth a one-line clarification in the function rustdoc.

### M2 — Test cell `decrypt_field_wrong_password` accepts two error variants

L325-328 of electrum_crypto.rs accepts EITHER `AesDecryptFailure` OR `Utf8DecodeFailure` for wrong-password. Comment correctly notes the ~1-in-65536 collision rate where PKCS7 strip survives. For the cross-impl smoke vector (deterministic IV `0x00112233...`), the outcome should be deterministic — picking ONE variant + asserting that single variant is stronger. Not a defect (the wider match is defensively correct), just leaves keyspace uncovered.

### M3 — P0 recon §A3 says `aes` is transitive via `bitcoin = "0.32" features=["base64"]`

The `base64` feature on `bitcoin` is unrelated to whether `aes` is transitively pulled in. Worth verifying via `cargo tree -p mnemonic-toolkit -i aes` post-6a-ship; if false, the "no-op promotion to direct dep" claim needs revision. Doesn't gate 6b but the recon framing is suspect.

## Verdict

**RED — re-brainstorm (specifically: fold C1 + C2 + propagate to decisions 4/5/6/7 + revise FOLLOWUP-closure semantics + revise SemVer).**

The C1 finding is foundational: the brainstorm's parser-integration design pre-decrypts JSON fields that the parser never reads, while the encrypted-field set (`seed`/`xprv`/`passphrase`/`keypairs`) lives entirely outside the parser's consumption surface. Path A (drop `--decrypt-password*` entirely; admit Electrum-encrypted wallets via a downgraded refusal that imports watch-only material) is materially simpler and ships as PATCH v0.30.1 with no new CLI flags. Path B (preserve flags as future-proofing + add explicit decrypt-and-discard password-validation) is viable but awkward. Either way the brainstorm's decisions 4/5/6 are invalidated as written; 6b cannot proceed to plan-doc without a brainstorm rev resolving C1 + C2. The 6a-shipped library (`electrum_crypto.rs`) is unaffected and ships fine regardless — Path A keeps it as an unused-by-CLI internal helper (filed as FOLLOWUP) or Path B retains it as the orchestrator's decrypt primitive.
