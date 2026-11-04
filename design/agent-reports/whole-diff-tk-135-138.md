# Whole-diff review: mnemonic-toolkit #135 (V7) + #138 (envelope E1-E3), head b944de90

Reviewer: Sonnet 5.5 (adversarial, probes run with target/debug/mnemonic built from this head, signet keys).

Outcome: no Critical, no Important. One lead graded (Minor), two Minors/Nits.

## Probes run (all as described, built binary)
- Tampered md1 (md1 taken from a different-path envelope, descriptor unchanged): `bundle --import-json` and `export-wallet --from-import-json` both exit 1 with the E1 line. Good.
- Tampered `multisig.threshold=1`, `multisig.template`, `template`, and `multisig=null`: bundle prints "Threshold: 2 of 3", export emits `sortedmulti(2`. K comes only from the descriptor. Good.
- Partial mk1 (first chunk only of a 3-chunk card, and `["mk1x"]`): exit 1, "mk1[0] decode failed: chunked-header malformed: received 1 chunks, header declares total_chunks = 3". Refused in bundle and in export-wallet (descriptor, sparrow).
- General policy (or_d(pk,multi), and_v(v:multi,older), thresh with older): threshold null, general Recovery line, sparrow refuses with the general-policy text, descriptor/bitcoin-core pass through. thresh(2,pk,s:pk,s:pk) reports 2 of 3 (it is a plain 2-of-3 semantically). multi/sortedmulti/sh(wsh()) give 2 of 3. No wrong K seen.
- Honest envelopes with depth-3 origin / depth-5 origin on a depth-4 xpub: not falsely refused.

## Lead from coordinator: swapped mk1[0] with a non-standard-path card
Reproduced the nearest accepted variant, not the exact one. The FIRST-CHUNK-ONLY form is refused (above). What IS accepted (exit 0): replacing mk1[0] with the COMPLETE 3-chunk card of the same xpub carrying origin path m/48'/1'/7'/2' while bundle.descriptor and bundle.origin_paths say m/48'/1'/0'/2'. Output of `bundle --import-json` and `export-wallet --from-import-json` (descriptor, sparrow) is byte-identical to the untampered envelope's output.
Cause: json_envelope.rs envelope_to_resolved_slots overwrites slot.path with envelope origin_path(s) (lines ~351-360), so the card's own origin path never reaches E2; E2 compares key + fingerprint + (envelope-sourced) path. Output mk1 cards are regenerated from the descriptor, so the foreign card path is dropped. No wrong K, path, fingerprint, address or descriptor results.

### M1 (Minor) card-only origin-path mismatch accepted though E2 message claims it is checked
json_envelope.rs:546-549 (E2 message "its key, fingerprint or origin path"). Fails open on an inconsistent envelope, but the output is unaffected and the path needs a hand-edited mk1 card whose path differs (2 conditions: crafted mk1 card with valid BCH, and path-only divergence). Optional fix: in check_envelope_consistency also decode envelope.bundle.mk1[i] and require card.origin_path to equal slots[i].path (or its depth-3 truncation per mk1_origin_path), same E2 line. Not required for correctness; the survivor at :546 is a fingerprint/path mix and is being handled separately.

## M2 (Minor) general-policy Recovery line omits timelocks
format.rs (new None arm): "keep all N signing keys, any hash preimages it names, and md1". For and_v(v:multi(2,..),older(100)) / thresh(...,sln:older(5)) it is silent about the timelock. The text is not wrong (md1 decides) and not a failing command; wording only. Fix text: "...any hash preimages it names, mind its timelocks, and md1 (template card)."

## N1 (Nit) 
For or_d(pk(K2),multi(2,K0,K1)) "keep all 3 signing keys" is conservative (K2 alone spends); acceptable, noted only.

Inherited/unrelated parse errors seen during probing (bare tpub without origin, `tr(` with internal key on import-wallet) occur at import-wallet, before this change's code; not reported.

C / I / M / N: 0 / 0 / 2 / 1
