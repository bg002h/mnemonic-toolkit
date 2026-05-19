# v0.27.0 Plan R6 architect validation — Phase 2 recon pivot

**Reviewer:** opus / feature-dev:code-architect
**Date:** 2026-05-18
**Subject:** `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` R6 revision (Path B-lite pivot)
**Verdict:** GREEN — 0 Critical / 2 Important / 4 Minor / 3 Praise
**Recommended action:** fold the 2 Importants inline (they are 3-line edits each, no plan-shape change); commit the plan revision; proceed with Phase 2.

---

## Summary

The R6 pivot is BIP-129-faithful, the recon doc's verbatim quotes match my independent fetch of BIP-129, the engine signatures in §3.4 are correct, and the new FOLLOWUP scopes the v0.28+ work well. Path B-lite is the right cycle-size: it closes the spirit of `bsms-verify-signatures` (BIP-129 SIG verification, which the FOLLOWUP body misframed as HMAC), preserves v0.26.0 backward-compat completely, and defers the encryption envelope cleanly. Cycle is still tractable in the remaining 4 implementation phases.

The two Important findings are residual-drift in three sections that were touched by the pivot but missed by the §8.1 diff table — easy folds.

---

## Critical: 0

(None.)

## Important: 2

### I1. Residual drift: §2.1 dependency cell + §4.0 phase-ordering box + §5 Risks row still reference 6-line emit

Three sites in the plan-doc still carry pre-pivot text that the §8.1 diff table did not capture:

- **§2.1 line 32** — `| 1 | wallet-export-bsms-emitter | feature | 7-8 | ~180 | #2 (6-line shape) |` — `Depends on` cell still says `#2 (6-line shape)`. Under the R6 pivot, Phase 3 (#1) is **independent** of Phase 2 (#2) per the §8.1 diff table (`§4.3` row: "4-line independent of Phase 2 verify path"). The dependency cell should read `—` (no dependency).
- **§4.0 phase-ordering box line 594** — `Phase 3: BSMS emitter (depends on #2)   → #1 wallet-export-bsms-emitter (2-line + 6-line)` — still says "2-line + 6-line" and "depends on #2". Should be `Phase 3: BSMS emitter (independent of #2) → #1 wallet-export-bsms-emitter (4-line + 2-line)`.
- **§5 Risks table line 744** — `BSMS 6-line m/0/0 derivation differs for taproot tr()` row + mitigation `Explicitly error on tr() 6-line emit in v0.27.0; FOLLOWUP bsms-taproot-6-line for v0.28+.` — the row's framing ("6-line") is dead and the FOLLOWUP slug `bsms-taproot-6-line` doesn't exist (the actual slug is `bsms-taproot-emit` per §4.6). Replace with: `BSMS taproot tr() emit unspecified in BIP-129` / mitigation: `Explicitly error on tr() with --format bsms in v0.27.0; FOLLOWUP bsms-taproot-emit for v0.28+.`

**Severity rationale:** these are not Critical because they don't cause incorrect code — Phase 3 implementer reading §3.5/§4.3 will get the right framing — but a future reader landing on §2.1 or §4.0 first will be confused about whether Phase 3 still has a 6-line surface. The §8.1 diff table claims to be "section-by-section" so these three sites need an explicit row.

**Fold:** add three rows to §8.1 (or edit the three sites in place with `[REVISED — Phase 2 recon pivot; see §8.]` tags). 3-line edits each.

### I2. §3.6 still references `synthesize_unified`; the rest of the plan now uses `synthesize_descriptor` consistently

§3.6 paragraph at the `bundle --import-json` wire-up section reads: *"dispatch to existing `synthesize_unified` path with `descriptor=<extracted>` + `template=None`."* — but R3's N-C2 fold (per §7 R1→R2 row) corrected the §3.2 path to use `synthesize_descriptor` (which is the descriptor-mode entry point with the matching `(descriptor, cosigners, privacy_preserving)` signature). §3.6's `bundle --import-json` consumer goes through the same descriptor-mode path and should call `synthesize_descriptor`, not `synthesize_unified`. This is not part of the §8 pivot but is a stray residual from an earlier round that the pivot review surfaces.

**Severity rationale:** Important because Phase 5 implementer reading §3.6 will likely call `synthesize_unified` and either compile-error or get wrong wiring; the contradiction with §3.2 will be caught at Phase 5 R0, but at higher cost than fixing now.

**Fold:** change §3.6's `synthesize_unified` to `synthesize_descriptor` and adjust the inline contract ("with descriptor=<extracted>" stays correct since synthesize_descriptor's first arg is descriptor). 1-line edit.

## Minor: 4

### m1. §3.2 paragraph already correctly uses `synthesize_descriptor`

The §3.2 prose reads correctly: *"Both v0.26.0 wallet-import formats produce a literal descriptor ... Therefore ALL ParsedImport-derived BundleJson constructions in v0.27.0 use **descriptor-mode synthesis**:"* + locks `synthesize_descriptor`. Good. (Mention here only as orientation for I2.)

### m2. `KeyField` vs `Xpub` are consistent across the plan

§3.4's `KeyField::Xpub { ... xpub: bitcoin::bip32::Xpub }` is a new BSMS-local enum; §3.6.1's `mk_codec::KeyCard.xpub` is the unrelated mk-codec field. The plan uses `Xpub` (PascalCase) consistently as the Rust enum/type identifier. XPUB is only used as the descriptor-format token (line-3 KEY of BSMS Round-1, where BIP-129 writes "XPUB" in caps). No drift; consistent.

### m3. §3.4 `BsmsVerifyError::SignatureMismatch.signer_pubkey_hex` vs `ToolkitError::BsmsSignatureMismatch.signer_pubkey`

§3.4's sub-enum uses field name `signer_pubkey_hex`; the ToolkitError variant uses `signer_pubkey`. R4 EDIT 15 (per §7) claimed these were unified to "computed/declared (drop _hex suffix)". The §3.4 pseudocode still says `signer_pubkey_hex: String`. Phase 2 implementer will likely match the ToolkitError naming. Minor; doesn't affect correctness.

### m4. §3.4 says "asserts recovered_pubkey == signer_pubkey + uses standard `verify_ecdsa` as belt-and-braces"

This is implementation detail; the recovery path alone is correct per BIP-322 legacy. The "belt-and-braces" verify_ecdsa is redundant (if recovery returns a pubkey, the signature IS valid against that pubkey by construction; the verify_ecdsa pass is a sanity check, not a security gate). Phase 2 R0 may prune to just the recover+compare. Leave as guidance.

## Praise: 3

### p1. The recon doc is exemplary

`design/agent-reports/v0_27_0-phase-2-bip129-recon.md` provides verbatim quotes from BIP-129 lines 81, 96, 94, 136-165 + 5 published test vectors with TOKEN/fingerprint/path/pubkey/SIG/ENCRYPTION_KEY values + reference-impl survey + crate-level Rust API mappings. I cross-validated the Round-1 5-line spec and Round-2 4-line spec quotes against an independent WebFetch of BIP-129; they match. The recon also flags the PBKDF2-salt-vs-MAC-hex-TOKEN asymmetry which would be an easy foot-gun for v0.28's encryption-envelope work. This is the kind of recon artifact that makes mid-execution pivots safe.

### p2. The FOLLOWUP closure narrative is honest

The `bsms-verify-signatures` FOLLOWUP body says "HMAC token + signature verification flow" (incorrect). The R6 closure narrative inline in FOLLOWUPS.md:2131 explicitly says *"the FOLLOWUP body wording above ... misreads BIP-129 ... v0.27.0 closes this FOLLOWUP by implementing BIP-129-faithful Round-1 verify ... NOT the HMAC-keyed Round-2 verify the FOLLOWUP body initially called for."* — this is an honest closure, not stretching. The FOLLOWUP's intent was "implement BIP-129 signature verification"; v0.27.0 implements exactly that, just discovering mid-execution that BIP-129's signature surface is BIP-322 Round-1, not HMAC Round-2. The new `bsms-bip129-full-cutover` FOLLOWUP cleanly carries the residual (encryption envelope, 4-line input parser, 6-line deprecation). Companion-line cross-citation present in both entries.

### p3. v0.26.0 backward-compat is fully preserved

§2.2 Q2 lock + §3.1.2 errors paragraph + §8.2 "Phase 0-1 commits remain valid" + §8.3 "deprecate v0.26.0 6-line lenient parser" filed to v0.28+: the v0.26.0 2-line/6-line `--blob` ingest is untouched, `bsms_audit.signature_verified` stays hard-coded `false` for the 6-line lenient path, and the deprecation is properly sequenced behind the BIP-129-faithful 4-line parser landing in v0.28+. This is the right shape.

---

## Specific checks against the dispatch's 8 numbered questions

**1. BIP-129 fidelity** — PASS. BIP-322 legacy-format ECDSA recoverable is correct. Message digest formula `dSHA256("\x18Bitcoin Signed Message:\n" + compact_size + body)` is correct. Signed body `"BSMS 1.0\n{TOKEN_hex}\n{KEY}\n{description}"` with NO trailing newline is correct. Pubkey extraction for xpub mode = xpub's OWN embedded pubkey, NOT child-derived, is correct (Ambiguity #4 in recon doc; Phase 2 implementer to pin explicitly in toolkit SPEC).

**2. Path B-lite scope** — PASS. Closes `bsms-verify-signatures` in spirit. Tractable in 4 remaining phases. New FOLLOWUP body covers (a) deprecation, (b) 4-line input parser, (c) encryption envelope, (d) drop window, (e) doc plan — comprehensive.

**3. Internal consistency** — 7 new flags lock matches §4.6 enumeration. 2 new ToolkitError variants match §3.4 + §4.2. `derive_address_at_path` is correctly identified as a new helper closing `bsms-first-address-verify`. 2 Important drifts noted (I1, I2).

**4. v0.26.0 backward-compat** — PASS. 2-line and 6-line `--blob` ingest unchanged. `bsms_audit.signature_verified` hard-coded `false` preserved.

**5. Recon doc accuracy spot-check** — PASS. Independent WebFetch of BIP-129 confirmed the Round-1 5-line spec quote, the Round-2 4-line spec quote, and that Round-2 records have NO SIG line.

**6. §8 diff completeness** — PARTIAL. §8.1 captures the major revisions but misses three residual-drift sites (folded as I1).

**7. FOLLOWUP closure precision** — HONEST CLOSURE. Praise item p2 covers the rationale.

**8. Plan-doc structural integrity** — ACCEPTABLE. The R5/R6 hybrid is navigable because (a) Status line at top says R6, (b) revised sections carry `[REVISED — Phase 2 recon pivot; see §8.]` tags, (c) §8 captures the diff.

---

## Recommended fold sequence

1. Fold I1 (3 sites): §2.1 row 32 `Depends on` cell, §4.0 phase-ordering box, §5 Risks row.
2. Fold I2 (1 site): §3.6 `synthesize_unified` → `synthesize_descriptor`.
3. Commit plan revision + recon doc + this validation review.
4. Proceed with Phase 2 implementation (BIP-322 verifier + 5-line parser + 15 test cells per §3.4 + §4.2).

No re-dispatch needed — these are mechanical drift folds, not architectural disagreements.

---

**End of architect validation review.**
