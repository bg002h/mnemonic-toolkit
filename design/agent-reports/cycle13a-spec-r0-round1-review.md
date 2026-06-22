# R0 REVIEW — cycle-13 Lane A: Coldcard/Jade multisig fidelity pair (H11 + H14) — Round 1

**Spec:** `design/BRAINSTORM_cycle13a_coldcard_multisig_fidelity.md`
Verified against current `origin/master = 9b2a8ae3` (spec pins `d55bf4c3`, its parent; all cited source byte-identical between them — only the recon `.md` differs).

## VERDICT: NOT GREEN — 0 Critical / 2 Important / 4 Minor

The spec is strong (protocol facts verified against authoritative source; H14 refuse reasoning correct; Q1 coupling well-analyzed). But it misses two live round-trip surfaces H11's divergent emit would silently corrupt.

## Citation verification (re-grepped, all accurate)
H11 collapse `coldcard.rs:329-336`, single `Derivation:` push `:361`, `cs.fingerprint` (master) `:366`, `origin_path_bare()` `:327`; `jade.rs:46` delegation. H14 truth table `coldcard_multisig.rs:363-400`; `computed_fp = xpub.fingerprint()` `:359-360`; `effective_fp` `:415`; NO `.depth` anywhere (grep-confirmed). `<XFP>:` arm clears pending path `:256`; bare-xpub arm consumes `:268-274`. json_envelope NOTICE-substitute `:388-398`. Exit codes `ImportWalletParse→2` (`error.rs:582`), `BadInput→1` (`:549`). SPEC §11.4.1 `:419-429`. **Protocol fact VERIFIED vs vendored source:** `bitcoin 0.32.8` (`Cargo.lock:198`), `bip32.rs:833-842` — `Xpub::fingerprint()` = HASH160 of `self.public_key.serialize()` (CURRENT key, not master/parent); `pub depth: u8` "(master = 0)" `:111`. (Note: rust-bitcoin doc-comment on `identifier()` wrongly says "HASH160 of the chaincode" — the BODY hashes the pubkey; spec's claim matches the body.)

## IMPORTANT (block GREEN)

### I-1. `canonicalize_coldcard_multisig` collapses divergent paths → H11 breaks the import round-trip-verify surface (UNADDRESSED)
`wallet_import/roundtrip.rs:361` `canonicalize_coldcard_multisig` re-emits in shared-derivation canonical form, deriving the single `Derivation:` from ONLY `parsed.cosigners[0].path` (`:401-403`), with a comment that it "ASSUMES homogeneous derivation" (`:397-400`). LIVE surface: `cmd/import_wallet.rs:1447` (`--round-trip-verify` compare) + `roundtrip.rs:570` (Jade round-trip). H11's divergent exports run through this canonicalizer get cosigner-0's path stamped on all → silently discards the per-cosigner paths H11 preserved → divergent export run through round-trip-verify falsely passes or mismatches. The spec never mentions `roundtrip.rs`.
**Required:** plan must EITHER (a) extend `canonicalize_coldcard_multisig` to emit per-cosigner `Derivation:` on heterogeneous paths (+ idempotence test) — the coherent co-design; OR (b) explicitly scope-out divergent round-trip-verify with a test asserting the chosen behavior. R0 must pick one.

### I-2. Sorted-multisig path↔xpub pairing under-specified — and SORTED is the ONLY reachable divergent case
(1) Cycle-2's H10 refusal (`cmd/export_wallet.rs:123-135`, `ExportWalletUnsortedMultisigUnsupported`) refuses unsorted (`WshMulti`/`ShWshMulti`) export to coldcard/jade → `emit_coldcard_multisig_text` is reachable for these ONLY via `WshSortedMulti`/`ShWshSortedMulti`, so EVERY reachable divergent coldcard-multisig export is sorted. (2) `derivations` is built from `inputs.resolved_slots` in SLOT order (`coldcard.rs:324-328`), but the cosigner emit loop iterates `cosigners` independently lex-sorted by xpub (`:338-345`/`:365`). A naive per-cosigner emit pairing `derivations[i]` with the i-th SORTED cosigner scrambles path↔xpub whenever sort order ≠ slot order — a corruption WORSE than the `m/0'/0'` it replaces.
**Required:** H11-b must mandate emitting each cosigner's `Derivation:` from its OWN sorted-slot origin (sort `(path, xpub, fp)` tuples together; read `cs.origin_path_bare()`+`cs.fingerprint`+`cs.xpub`), never index a separate slot-order vector. Add a RED test with cosigners whose xpub-sort order ≠ slot order + divergent paths, asserting correct path↔xpub pairing. (RED test #1 as written — `@0`==`@2` — does not exercise this.)

## MINOR (fold; non-blocking)
- **M-1.** H14's depth>0/no-XFP refusal also fires on JADE import (`coldcard_multisig::parse_text` shared via `jade.rs:105`/`mod.rs:122`). Correct, but add a Jade-import RED test + blast-radius note.
- **M-2.** The toolkit's OWN shared-path export already round-trips through Row 2 (spurious warning) today: `synthesize.rs:577-644` derives cosigner xpubs to depth 4 while `ResolvedSlot.fingerprint` is the true depth-0 master fp (`:646`) → emits `<XFP_master>: <depth-4-xpub>` → on re-import `supplied≠computed` → Row 2 warning on the toolkit's own all-agree export. H14-c silences a warning that mis-fires today. State explicitly; check the all-agree round-trip test during fixture rewrite.
- **M-3.** Re-pin spec header `d55bf4c3` → `9b2a8ae3`.
- **M-4.** Line nits: fixture FP consts `:945-947` (not `:939-947`); rust-bitcoin `:833-842`/`:111` (not `:835-844`); flag the stale rust-bitcoin `identifier()` doc-comment.

## Open questions — RATIFIED
- **Q1 — RATIFY resolution (A)** with I-1+I-2 conditions. The `<XFP>:` arm (`:245-256`) does NOT consume `pending_per_cosigner_path` (clears at `:256`); only the bare-xpub arm does. Extend the `<XFP>:` arm to consume the pending path → H11 emits `Derivation: <path_i>` + `<XFP_master_i>: <xpub_i>` → H14-c silent accept. Shared-path regression bounded (cosigners 2..N fall back to `shared_derivation` `:341`). **Plan must add a 3-cosigner shared-path regression test** proving 2..N still resolve to the shared path, and the change must NOT clear `shared_derivation`.
- **Q2 — RATIFY refuse** (depth>0/no-XFP master fp provably unrecoverable; HASH160 one-way; refusing before steel-engraving is funds-safety-correct; no real Coldcard export omits XFP at depth>0 that users rely on — firmware stamps `XFP:`; the no-XFP older form pairs with depth-0 master xpubs which still pass via H14-a).
- **Q3 — RATIFY `xpub.depth == 0`** (`pub depth: u8` public field, authoritative, independent of declared path; toolkit's own export emits depth-4 account xpubs so a re-serialized account xpub won't be depth 0 — concern moot).

## Scope/SemVer — CONFIRMED
Toolkit MINOR v0.66.0 (H11 wire-shape + H14 intake refusal). No clap flag → schema_mirror untouched. md/ms/mk NO-BUMP. Manual `45-foreign-formats.md` + `37-wallet-export.md` (exist, voluntary). SPEC §11.4.1 correction + fixture rewrite in-scope; ADD roundtrip.rs canonicalizer fixtures (I-1) + sort≠slot divergent fixture (I-2).

## Path to GREEN
Resolve I-1 (canonicalizer divergent treatment) + I-2 (sorted-slot origin pairing) to concrete decisions; fold M-1..M-4 + the Q1 shared-path regression condition; persist this review; re-dispatch R0. Core design (refuse depth>0/no-XFP, accept supplied XFP silently, per-cosigner `Derivation:` via resolution-A) is sound and well-evidenced.
