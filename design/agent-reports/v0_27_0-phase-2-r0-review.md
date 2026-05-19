# v0.27.0 Phase 2 R0 review — BIP-129 Round-1 BIP-322 verify engine

**Phase:** 2 (per `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §4.2 R6)
**Reviewer:** opus / feature-dev:code-reviewer
**Verdict:** GREEN — 0 Critical / 0 Important / 3 Minor / 13 Praise
**Date:** 2026-05-18

---

## Scope reviewed

Phase 2 implements BIP-129 Round-1 BIP-322 ECDSA recoverable signature verification per the R6 plan (post Phase 2 recon pivot). Files:
- `Cargo.toml` — enable `bitcoin` crate's `base64` feature.
- `src/error.rs` — 2 new variants `BsmsRound1Malformed` + `BsmsSignatureMismatch` at end of enum.
- `src/wallet_import/bsms_round1.rs` — NEW parser (~340 LOC). `parse_round1`, `signer_pubkey`, `signed_body` + 9 unit cells.
- `src/wallet_import/bsms_verify.rs` — NEW verifier (~180 LOC). `verify_round1_signature` + 6 unit cells against BIP-129 TVs 1/2/3.
- `src/wallet_import/mod.rs` — register modules.
- `src/cmd/import_wallet.rs` — CLI flags `--bsms-round1` + `--bsms-verify-strict`; standalone-vs-combined-mode wiring; `Round1Verification` struct + helpers.
- `tests/cli_bsms_round1.rs` — NEW 15 integration cells.
- `tests/fixtures/bsms_round1/tv{1,2,3}-*.bsms` — BIP-129 in-spec TV fixtures verbatim.
- `design/FOLLOWUPS.md` — `bsms-verify-signatures` Status flipped to resolved with R6 closure narrative.

Test totals: 1491 pass (1476 baseline + 15 integration cells; plus +6 verify-unit + +9 parser-unit included in baseline crate-internal count). NO regressions.

---

## Critical: 0

(None.)

## Important: 0

(None.)

## Minor: 3

**M1. `bsms_verify.rs` module doc-comment phrasing.** Lines 19-21 say "Recovery is sufficient — if `recover_pubkey` returns a pubkey, the signature IS valid against that pubkey by construction; comparing against the declared `signer_pubkey` is the load-bearing security gate." The implementation runs BOTH recover AND compare-against-declared (lines 50-65). The phrasing "recovery is sufficient" reads ambiguous in isolation. Consider clarifying to "recovery + compare-against-declared is the two-step gate." Non-blocking.

**M2. `Round1Verification.failure_reason` is `None` for successful-verify case.** This is by design and matches the JSON envelope shape regression-guarded in cell 12 (`assert!(verifications[0]["failure_reason"].is_null())`). No issue. Calling out for completeness because it's the kind of "Option-vs-empty-string" choice reviewers second-guess.

**M3. Doc-comment-path nit.** `bsms_round1.rs:23` brings in `bitcoin::secp256k1::PublicKey` (re-export of `secp256k1::PublicKey`); plan §3.4 prose says `secp256k1::PublicKey` directly. Functionally equivalent; path-name nit only. Non-blocking.

## Praise: 13

**P1. BIP-129 spec fidelity at `signed_body()` is byte-exact.** Unit test asserts the verbatim 4-line body for TV-1: `"BSMS 1.0\n00\n[59865f44/48'/0'/0'/2']026d154124...\nSigner 1 key"`. Matches BIP-129 line 81 specification verbatim — 4 lines joined by `\n`, NO trailing newline, line 3 = `[fingerprint/path-without-m-prefix]KEY`. `format_path_no_m_prefix` correctly strips the `m/` prefix.

**P2. BIP-322 digest correctness.** `bsms_verify.rs:39` uses `signed_msg_hash(&body)` from `bitcoin::sign_message` — the exact `rust-bitcoin` primitive recon doc §3.4 calls out as the BIP-129 digest. Coinkite Python ref `bitcoin_msg` produces byte-identical output.

**P3. Xpub OWN-embedded-pubkey rule.** `signer_pubkey()` returns `xpub.public_key` directly, NOT a child-derived key. Dedicated unit test pins this with a hard-asserted `KeyField::Xpub` destructure + equality check. Matches BIP-129 line 81 + recon doc Ambiguity #4 explicitly.

**P4. Recover + compare two-step verify.** `bsms_verify.rs:50-65` does recover-pubkey THEN explicit equality check against the declared pubkey from line 3. Both gates run. The comparison uses `recovered.inner != declared`, with the `.inner` accessor extracting the raw `secp256k1::PublicKey` from `bitcoin::PublicKey`. Correct.

**P5. TOKEN-in-signed-body invariant is regression-guarded.** `bsms_verify::tests::tv1_with_flipped_token_byte_rejects_with_signature_mismatch` flips `\n00\n` → `\nff\n` and asserts `BsmsSignatureMismatch`. Because TOKEN is part of the signed body, flipping it changes the digest → recovery yields a different pubkey → mismatch. Unit test plus integration cell 7 both guard this load-bearing invariant.

**P6. Backward-compat preservation.** `wallet_import/bsms.rs` (the v0.26.0 6-line lenient parser) is unchanged; the `signature_verified: false` hard-code remains. `sniff()` + `parse()` for the v0.26.0 path are untouched.

**P7. Standalone-vs-combined mode wiring is clean.** `import_wallet.rs` short-circuits to the standalone envelope when `args.blob` is `None`; `emit_json_envelope` threads `round1_verifications` through as the `bsms_round1_verifications` field on each per-bundle envelope when `--blob` IS supplied. Cells 12 + 13 both regression-guard the shape.

**P8. Error variant exit codes match Q10.** Both `BsmsRound1Malformed` and `BsmsSignatureMismatch` route to exit 2; `kind()` matches; `message()` Display is informative. Newest-at-bottom convention preserved.

**P9. `record_index` propagation.** `verify_round1_signature` takes `record_index: usize` and threads it into `BsmsSignatureMismatch`. `verify_bsms_round1_files` passes the enumerated 0-based index. Cell 15 regression-guards the multi-record case (index 1 surfaces in stderr).

**P10. Cargo.toml feature wiring.** `bitcoin = { version = "0.32", features = ["base64"] }` keeps the defaults (`std` + `secp-recovery`) AND adds `base64`. `MessageSignature::from_base64` is gated by the `base64` feature; `recover_pubkey` is gated by `secp-recovery` (default-on). Both are available.

**P11. Recon doc adherence in doc-comments.** Both new modules cite BIP-129 line 81 verbatim AND the recon doc path. The verbatim BIP-129 quote in `bsms_round1.rs` is identical to the recon doc §3.

**P12. CLI flag validation is comprehensive for v0.27.0 scope.** `--bsms-verify-strict` without `--bsms-round1` is explicitly rejected with `BadInput` exit 1 (cell 10 regression-guards). Stdin `-` is rejected with deferred-feature messaging (cell 11 regression-guards). The `required_unless_present` on `--blob` is clap-enforced.

**P13. FOLLOWUPS.md Status flip applied.** Per the cycle's per-phase agent feedback memory ([[feedback-per-phase-agents-forget-followup-status-flip]]), the `bsms-verify-signatures` FOLLOWUP Status was correctly flipped open → resolved with closure narrative citing the R6 pivot. No split-state hazard.

---

## Verdict

**GREEN — commit Phase 2 immediately; proceed to Phase 3.**

Headline: Phase 2 is a spec-faithful, well-instrumented BIP-129 Round-1 BIP-322 verifier. All 12 numbered checklist items in the review dispatch pass. The 3 minor findings are doc-comment polish; not plan-violating or load-bearing.
