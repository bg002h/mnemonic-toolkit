# SPEC — wallet-file cross-format convergence + hop-idempotence test set

- **Date:** 2026-05-27
- **Source SHA:** `9a88a46` (toolkit `master`)
- **Status:** approved design (brainstorm + opus architect blueprint); **pending R0 to 0C/0I before any test code.**
- **Type:** test-only. Per the prior convergence cycles (bundle: F3/F4; convert: clean + by-design F-fp), a red cell is a *finding* to triage (test artifact / real product bug / by-design divergence), not designed around. Genuine product bugs → user fix-vs-defer decision.

## Purpose
Metamorphic convergence for the WALLET-FILE surface — the third and hardest domain (8 independent parsers/emitters). Two properties:

**Part 1 — cross-source envelope convergence.** The SAME wallet, expressed in different wallet-file formats, must import to the SAME canonical key-material. Construction: ONE in-test wallet → `export-wallet --format F` to each round-trippable F → `import-wallet --format F --json` each back → assert all envelopes converge on decoded key-material. The "same wallet regardless of source format" analog of cross-start.

**Part 2 — cross-format hop idempotence.** `import A → export B → import B` yields the same key-material as direct `import A` (no key-material lost crossing a format boundary). A representative non-redundant set of (A,B) pairs.

## Convergence target — DECODED key-material, NOT bytes/descriptor (F1)
F1 (documented, bundle cycle): bitcoin-core splits `<0;1>` into `/0/*`+`/1/*`; BSMS keeps multipath; formats differ on metadata + checksums. So byte- and descriptor-string-identity are impossible by design. Compare the tuple that MUST hold:
- **xpub multiset** — decode each `bundle.mk1[i]` via `mk_codec::decode` → `KeyCard.xpub` (normalized; handles electrum SLIP-132 `Zpub`→`xpub`);
- **per-cosigner triples** `(xpub, fingerprint_lc, origin_path)` from the same decode;
- **fingerprint set** — cross-check `bundle.multisig.cosigners[].master_fingerprint` against the mk1-derived set;
- **policy** — `threshold`, `cosigner_count`, and `md_codec::chunk::reassemble(bundle.md1)` facts (tag/script-type, `is_wallet_policy`);
- **network**.
**EXCLUDED (documented in the assert helper):** raw `bundle.descriptor` (F1), `ms1` (`""` watch-only sentinels), format metadata (name/label/blockheight/timestamp/range/gap/devices), BIP-380 checksums. A field that SHOULD converge but doesn't = a finding.

## Round-trippable format set + export→import consumability
Import formats (`import_wallet.rs:141`): {bitcoin-core, bsms, coldcard, coldcard-multisig, electrum, jade, sparrow, specter}. Export formats (`export_wallet.rs:22`): + bip388, green (export-only — excluded). Per-format export→import consumability (architect-verified; two seams):
- **bitcoin-core** — export emits a BARE array `[{desc,...}]`; import needs `{wallet_name, descriptors:[...]}`. **Needs object-wrap** (`wrap_export_in_object_envelope`, `cli_import_wallet_roundtrip.rs:361`). Mechanical.
- **jade** (R0 I1 — DETERMINED from source, not a runtime unknown): export emits bare Coldcard-style multisig text (`wallet_export/jade.rs:34-69`); import requires JSON with a top-level `multisig_file` string (`wallet_import/jade.rs:82-128`). So bare `export jade → import jade` fails — but the fix is the same mechanical class as bitcoin-core's object-wrap: **wrap the export body as `{"multisig_file": "<text>"}`**, which the jade importer delegates byte-identically to the coldcard-multisig parser (`jade.rs:131`). **jade IS a confirmed C2/C3 convergence node via this wrap** (not contingent, not dropped). Optional guard cell: assert the un-wrapped bare jade export is REJECTED by the jade importer (documents the seam). The seam is by-design, NOT a finding.
- bsms, coldcard(singlesig), coldcard-multisig, electrum, sparrow, specter — directly consumable (verify at write time; use `--bsms-form 2-line` if 4-line trips import).

## Construction (F2-safe)
ONE wallet per shape, export-generated from in-test-derived keys (pattern `cli_cross_start_convergence.rs:33-43` / `cli_bundle_multisig.rs:25`). Do NOT reuse `tests/fixtures/wallet_import/*` — different wallets, and coldcard/jade fixtures carry xpub-computed (not master) fingerprints. Thread explicit `--slot @N.fingerprint=` + `@N.path=` so the true master fp flows into every emitter. Single-sig: bip84 wpkh from TREZOR_24 @ m/84'/0'/0'. Multisig: wsh-sortedmulti 2-of-3, bip48 m/48'/0'/0'/2', from MS_PHRASES.

## Matrix

### Part 1 — cross-source envelope convergence
| Cell | Shape | Format set | Assertion |
|---|---|---|---|
| **C1** | SS bip84 wpkh | bitcoin-core, coldcard, electrum, sparrow, specter | all envelopes share `KeyMaterial` (1 xpub, 1 master fp, threshold None, N=1, md1 wpkh+wallet-policy, mainnet). **C1 coldcard caveat (R0 M1 + R1):** coldcard's fp is carried as top-level `xfp` (= true master fp, threaded via `--slot @N.fingerprint=`) and the importer builds the origin bracket from that `xfp` (`wallet_import/coldcard.rs:232-242`), so coldcard **is expected to converge** on the master fp like the others. The assert helper tolerates-and-records per node purely as a safety net (so a surprise single-node divergence is reported, not silently failed) — convergence is the expectation, not a predicted finding. |
| **C2** | wsh-sortedmulti 2-of-3 (bip48) | bitcoin-core, bsms, coldcard-multisig, electrum, jade, sparrow, specter (7) | all share `KeyMaterial` (xpub set {A,B,C}, triples, fp set, K=2/N=3, md1 wsh-sortedmulti, mainnet). jade via the `{"multisig_file":...}` wrap (R0 I1). |
| **C3** | sh-wsh-sortedmulti 2-of-3 | same 7 | as C2, md1 tag P2SH-P2WSH (distinct emit/parse path) — **KEEP** (cheap, separate code path) |
| **C4** | wsh-**multi UNSORTED** 2-of-3 (bip87) | order-preserving {bitcoin-core, bsms, sparrow, specter} + **probe coldcard-multisig** | xpub set + **positional order** converge; coldcard-multisig probe asserts converge-OR-documents reorder (coldcard importer always builds `sortedmulti`, `coldcard_multisig.rs:663` → wsh(multi) round-trips as wsh(sortedmulti); expected divergence to document) |
| **C-neg** | swap one cosigner xpub | — | different wallet ⇒ `KeyMaterial` NOT equal (anti-vacuity) |

Documentation sub-checks (C2/C3) — **SOFT (record, don't hard-fail; R0 M3):** probe whether the EXCLUDED `bundle.descriptor` differs across formats (bitcoin-core split vs bsms multipath). The importer may re-canonicalize the split back to `<0;1>` in `bundle.descriptor`, so this is exploratory documentation of F1, not a hard assertion (a hard assert here could itself become a false finding).

### Part 2 — hop idempotence (6 non-redundant (A,B) pairs)
| Cell | Shape | A → B | Boundary crossed |
|---|---|---|---|
| **H1** | MS | bsms → bitcoin-core | multipath `<0;1>` → split `/0/*,/1/*` (F1) |
| **H2** | MS | sparrow → coldcard-multisig | declaration-order JSON → lex-sorted text; fp JSON → `<XFP>:` |
| **H3** | MS | bitcoin-core → electrum | base58 xpub → SLIP-132 Zpub (normalization) |
| **H4** | MS | specter → jade | descriptor-JSON → coldcard text. **B-leg (R0 I2):** `export-wallet --format jade` with the multisig sortedmulti template + `--threshold 2` (jade refuses singlesig/taproot, `wallet_export/jade.rs:36-63`), then apply the `{"multisig_file":...}` wrap before `import-wallet --format jade`. |
| **H5** | SS | electrum → sparrow | SLIP-132 zpub → Sparrow keystore (single-sig) |
| **H6** | MS | coldcard-multisig → bsms | text → BSMS descriptor (fp-source change) |
Each: `KeyMaterial(import A) == KeyMaterial(import(A→export B→ )))`. Every format appears as A and B ≥ once.

## Predicted divergences (bug hunt; triage on red)
1. **C4 unsorted-multi reorder** (likeliest) — does coldcard/electrum silently lex-sort an unsorted `multi`? (changes scriptPubKey → real bug, or a refusal to document).
2. **C2/C3 fingerprint source** — coldcard-multisig/jade carry fp only in `<XFP>:` lines; does the emitter write the TRUE master fp or an xpub-derived one?
3. **electrum SLIP-132** normalization (C2/H3/H5).
4. origin-path normalization (apostrophe vs `h`, leading `m/`); network inference; md1 classification.

## Non-redundancy
Existing `cli_export_wallet_from_import_json.rs` P11B extracts xpubs from ONE source's own export (different fixture per source) — proves per-source preservation, NOT cross-source convergence. `cli_import_wallet_roundtrip.rs` export→import for bitcoin-core only. `cli_cross_start_convergence.rs` converges across INPUT MODES (seed/xpub/descriptor/walletfile), not across wallet-FILE formats. `cli_import_wallet_format_mismatch_matrix.rs` = off-diagonal refusals. None asserts "same wallet, N formats, one key-material envelope" or hop idempotence. No overlap.

## Out of scope
Taproot: `export-wallet --from-import-json` refuses tr (`export_wallet.rs:652`), and tr export needs `--taproot-internal-key` with no envelope round-trip. Excluded.

## Mechanism / placement / self-containment
Canonical in-test fixtures (not proptest — small enumerable format set, specific named divergence points, byte-precise per-format massaging). New `tests/cli_wallet_cross_format_convergence.rs`, `assert_cmd::cargo_bin("mnemonic")` + in-process `mk_codec::decode` / `md_codec::chunk::reassemble` (already deps). No sibling binary, no network → default `cargo test`, no `#[ignore]`.

## Verification & ship
`cargo test -p mnemonic-toolkit` green (or surfaced finding) + `cargo clippy -D warnings`. Test-only → commit-to-master no-bump (consistent with `9a88a46`), unless a red cell becomes a product fix (then PATCH + Phase-6).

## R0 history
R0 (`design/agent-reports/wallet-convergence-R0-review.md`): RED 0C/2I/3M → folded. I1 (jade = confirmed node via `{"multisig_file":...}` wrap, not a Phase-1 suspect/drop) + I2 (H4 jade B-leg: multisig sortedmulti + `--threshold` + wrap) + M1 (C1 coldcard fp reconstructed → tolerate-and-record) + M3 (descriptor sub-check soft). M2 resolved (`9a88a46` confirmed master tip; all cited line numbers verified). Feasibility (round-trippable set, export→import consumability, KeyMaterial APIs, order-preservation, non-redundancy, self-containment) all confirmed clean. R1 re-dispatch pending (jade rewrite touched C2/C3/H4).
