# R0 review — SPEC_wallet_cross_format_convergence_tests.md (verbatim, persisted before fold)

Reviewer: feature-dev:code-reviewer (opus). Base `9a88a46`. Cycle: wallet-file cross-format convergence + hop idempotence.

## Verdict: RED — 0 Critical / 2 Important / 3 Minor
No Critical: every cell implementable, convergence-target APIs real + in active use, predicted divergences correctly scoped as findings-not-blockers. Two Important spec-accuracy corrections gate GREEN (both change how a cell is written).

## Verified clean (load-bearing)
- Import set `import_wallet.rs:141` exact; bip388+green export-only, correctly excluded.
- Envelope `format.rs:120` carries mk1 (untagged outer array / per-cosigner Vec<Vec<String>>), md1, multisig.cosigners[].master_fingerprint, threshold, cosigner_count, network.
- KeyMaterial APIs real + used: `mk_codec::decode(&chunks)→KeyCard{xpub,origin_fingerprint,origin_path}` (`cli_bundle_watch_only.rs:20`, `cli_bundle_import_json.rs:82-86` — direct precedent decoding per-cosigner mk1 positionally), `md_codec::chunk::reassemble` (`cli_standalone_bijections.rs:189`).
- electrum SLIP-132 normalize BEFORE descriptor build (`wallet_import/electrum.rs:557/836/947`) → xpub set converges.
- bitcoin-core object-wrap real (`wallet_export/bitcoin_core.rs:85` bare array; `wallet_import/bitcoin_core.rs:104-134` needs object; `wrap_export_in_object_envelope` `cli_import_wallet_roundtrip.rs:361`).
- Construction: `--template wsh-multi` unsorted exists (`template.rs:26`); `--threshold`/`--multisig-path-family` (`export_wallet.rs:71,74`); watch-only slots accepted.
- C4 order-preservation on {bitcoin-core,bsms,sparrow,specter} confirmed (`pipeline.rs:85` slot-order; `parse_descriptor.rs:785/792` sorts by placeholder INDEX not key — positional preserved; Tag::Multi vs SortedMulti distinct).
- C4 coldcard probe correct: emits slot-order but no sorted marker, importer ALWAYS builds sortedmulti (`coldcard_multisig.rs:663`) → wsh(multi) round-trips as wsh(sortedmulti) = the converge-or-document case.
- C3 sh-wsh: coldcard `Format: P2SH-P2WSH` ↔ `sh(wsh(sortedmulti))` (`coldcard.rs:276`); others pass through. Feasible all 7.
- Non-redundancy: P11B uses different fixture per source (`:621-645`) — not cross-source convergence. Novel.
- Self-containment: all APIs already deps + used in integration tests; no sibling binary/network; no #[ignore].
- specter import (`wallet_import/specter.rs:53/98/157`) + electrum SS export+import (`wallet_export/electrum.rs:101` + `wallet_import/electrum.rs:500`) → H4-A, H5, C1 electrum feasible.

## Important (fold before code)
### I1 — jade is a CONFIRMED node via a JSON-wrap, NOT a Phase-1 suspect
Source is definitive: jade EXPORT (`wallet_export/jade.rs:34-69`) delegates to `emit_coldcard_multisig_text` → bare Coldcard text (NOT JSON). jade IMPORT (`wallet_import/jade.rs:82-128`) requires JSON with top-level `multisig_file` string → bare text fails `serde_json::from_slice`. So `export jade → import jade` cannot round-trip directly — determinable NOW. **Fix is mechanical (same class as bitcoin-core wrap):** wrap export body as `{"multisig_file": "<text>"}`; the jade importer delegates the inner text byte-identically to the coldcard-multisig parser (`jade.rs:131`), so the wrapped form round-trips. Jade IS a usable C2/C3 node via this wrap — do NOT drop or make contingent. **Fold:** rewrite lines 27 + 44 + C2 set: jade export→import seam is a known `{"multisig_file":...}` wrap (analogous to bitcoin-core); remove the drop/FOLLOWUP contingency. Optional guard: assert bare (un-wrapped) jade export is REJECTED by the jade importer (documents the seam).

### I2 — H4 (specter→jade) needs multisig sortedmulti template + --threshold + the I1 wrap
jade emitter refuses singlesig/taproot, requires a multisig template (`wallet_export/jade.rs:36-63`). H4 B-leg must drive `export-wallet --format jade --template wsh-sortedmulti --threshold 2 …` (or `--from-import-json` deriving the template) and apply the `{"multisig_file":...}` wrap before re-import. **Fold:** add this note to H4 (same root cause as I1).

## Minor
- **M1** — C1 coldcard fp is RECONSTRUCTED (top-level `xfp`=`slot.fingerprint` `coldcard.rs:211`; `bipNN.xfp`=parent fp `:155`; `bipNN.deriv`), not a `[fp/path]xpub` bracket. A coldcard-node fp/path mismatch is a candidate finding, not a test bug. C1 assert helper should tolerate-and-record per node.
- **M2** — Source SHA `9a88a46` unverifiable from the reviewer's tree view (cosmetic; all 4 cited line numbers verified correct against current working tree). Confirm live tip. [Resolved: `9a88a46` IS the master tip.]
- **M3** — Line-46 descriptor-difference sub-check is fragile: the importer may re-canonicalize bitcoin-core's split back to `<0;1>` in `bundle.descriptor`, defeating a hard assert. Keep it SOFT (record, don't hard-fail) or verify at write-time which field carries the split.

## Fold summary
Fold I1 (jade = confirmed node via `{"multisig_file":...}` wrap; rewrite lines 27/44 + C2 set) + I2 (H4 jade B-leg: multisig sortedmulti + --threshold + wrap). M1 (C1 coldcard tolerate-and-record), M3 (soft descriptor sub-check) recommended. Re-dispatch R0 after fold (jade rewrite touches C2/C3/H4).
