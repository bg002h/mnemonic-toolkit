# R0 Review — CI/test hygiene (g6 pin + synthesize correctness) — ROUND 1

**Source SHA:** `5fc805f`. **Verdict: 🟡 YELLOW — 0 Critical / 3 Important / 6 Minor.** Citations accurate; NO-BUMP correct. Findings narrow/sharpen both parts.

## Critical
None.

## Important

**I1 — Part B: mk1→xpub + md1→policy round-trip ALREADY exist; the only real gap is ms1→entropy.** `cross_binding_holds_round_trip` (`synthesize.rs:1046-1071`) already, on the same `fixture_full`, decodes mk1 → `assert_eq!(decoded_mk1.xpub, xpub)` (`:1069`) + `origin_fingerprint == Some(fp)` (`:1070`), and md1 → policy-id-stub binding + `is_wallet_policy()`. So adding mk1/md1 decode to `full_bundle_emits_three_cards` is NOT new coverage. **Narrow Part B to the ms1→entropy decode assertion** (no test decodes the emitted ms1 and asserts the entropy round-trips — the genuine gap). Drop the mk1 add (or label it explicit non-new colocation).

**I2 — Part A: option (a) drift story + exact extraction.** `actions/checkout` `ref:` DOES accept `${{ steps.x.outputs.y }}`. But "no drift possible" is imprecise — it shifts the risk to the grep pattern; and a step-output `ref:` is invisible to `sibling-pin-check` too. Resolution: option (a) IS the right call (single source of truth = install.sh) IF (1) the extraction MIRRORS `sibling-pin-check.yml:60-62` parser (so a format change breaks both together → caught), and (2) the step FAILS LOUD on an empty tag (else empty `ref:` → checkout defaults to master → silently re-introduces the bug). Specify the exact command + the empty-guard.

**I3 — Part B: ms1 assertion form + RED-proof direction.** `ms_codec::decode` returns `(Tag, Payload)`; the English fixture emits `Payload::Entr(Vec<u8>)`. Specify: `let (_, payload) = ms_codec::decode(&bundle.ms1[0]).unwrap(); match payload { ms_codec::Payload::Entr(b) => assert_eq!(b, entropy, ...), other => panic!(...) }`. RED-proof: assert decoded == `entropy` (the PRE-synthesis input), then perturb the EXPECTED side of the assert (NOT the synthesis input) to confirm RED; restore. (Perturbing the input would compare a wrong card to a wrong expectation = could falsely pass.)

## Minor
- **m1/m2/m3** — citations (rust.yml:219, install.sh:38, synthesize.rs test lines) all ACCURATE @ 5fc805f. No action.
- **m4** — `fixture_full` returns `entropy: Vec<u8>` (plain); compare `Vec<u8> == Vec<u8>`; no zeroize obligation in test (don't let the test pattern suggest omitting zeroize in non-test callers).
- **m5** — CHANGELOG convention: NO-BUMP CI/test-only commits get NO CHANGELOG entry (per the prior friendly-mapper NO-BUMP cycle). State definitively; drop the hedge.
- **m6** — bundling the 1-line rust.yml change + the synthesize test addition in one NO-BUMP commit is coherent. Confirmed.

## Confirmations
- mlock.rs byte-identical master vs ms-cli-v0.7.0 (verified) → pinning passes.
- `ms-codec`/`mk-codec` are `[dependencies]` → reachable from the in-crate synthesize test module.
- No other `ref: master` sibling checkout in the workflows (g6 is the only one).
- Truly NO-BUMP (no binary/wire/CLI change).
