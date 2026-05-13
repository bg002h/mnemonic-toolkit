# v0.9.0 Phase 2 — Zeroizing wrappers R2 (cross-repo fold-verification)

**Reviewer:** Sonnet 4.6 via `feature-dev:code-reviewer` agent, 2026-05-13.
**Branches:**
- mnemonic-toolkit: `v0_9_0-phase-2-zeroize` at HEAD `0af7634`
- mnemonic-secret: `v0_9_0-phase-2-zeroize` at HEAD `d5c490f`

## Verdict

**0C / 0I — Phase 2 READY TO CLOSE.**

R2 trivial fold-verification per project convention. R1 was 0C/4I/5N
(Opus); all 4 Important findings + 1 Notable folded in `0af7634`
(toolkit) + `d5c490f` (ms-secret). R2 confirms each fold is present
and the exit-gates remain clean.

## Per-fold confirmation

- **I-1** ✓ `synthesize.rs:404-405` wraps `seed_mnemonic.to_entropy()`
  in `Zeroizing::new(...)`; Payload::Entr receives a cloned Vec.
- **I-2** ✓ 5 production `SecretKey::from_slice` sites carry
  SAFETY anchors; `lint_safety_third_party_blocked.rs:55` adds
  `"SecretKey::from_slice"` to CALL_PATTERNS;
  `rust-secp256k1-secretkey-zeroize-upstream` FOLLOWUP entry exists.
- **I-3** ✓ `derive_child.rs:108-119` declares
  `Option<zeroize::Zeroizing<String>>`; usage at L135-137 uses
  `.as_ref().map(|z| z.as_str())`.
- **I-4** ✓ ZEROIZE_ROWS evidence anchors are per-row specific in
  all three lint files (toolkit + ms-codec + ms-cli). No generic
  `["Zeroizing"]` anchors remain.
- **N-1** ✓ ms-cli encode.rs removes `entropy_for_codec` intermediate;
  calls `Payload::Entr((*entropy).clone())` directly.

## Exit-gate verification (verified by parent agent post-Sonnet R2)

- **toolkit:** `cargo test --workspace`: **43/43 green**, 0 failed.
  `cargo clippy --workspace --all-targets -- -D warnings`: **clean**.
- **ms-secret:** `cargo test --workspace`: **44/44 green**, 0 failed.
  `cargo clippy --workspace --all-targets -- -D warnings`: **clean**.

## Disposition

**MERGE.** R1 + R2 jointly close Phase 2 at 0C/0I across both repos.
Phase 2 of v0.9.0 Cycle A (Zeroizing wrappers — OWNED-buffer secret-
memory hygiene) is COMPLETE. Phase 3 (cross-repo secret-memory-
hygiene audit matrix) is the next workstream per plan §"Phase 3".
