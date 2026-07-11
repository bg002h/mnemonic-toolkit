# R0 convergence review — `SPEC_test_hardening_T3_wire_goldens.md` (round 2) — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Verified against LIVE source: toolkit `c00ed813` (=HEAD, T2 committed), descriptor-mnemonic `b9662e5f` (=HEAD, T2 committed). Scoped to fold verification (T3-c not re-litigated).

## Fold-by-fold
**C1 (long-code dead) — CLOSED.** SPEC drops the shape + cap mutation explicitly (residual = 3 shapes); no cap mutation in the RED list; manual/lockstep note zeroed. Reconfirmed live: `md-cli/src/cmd/encode.rs:41-45` hard-errors `--force-long-code`; grep `long_code|force_long|bch.*long` in `md-codec/src` = one comment ("NOT long-code", `test_vectors.rs:66`). No long-code path exists.

**C2 (re-scope to direct construction) — CLOSED.** (a) Entry points exist: `lib.rs` re-exports `Descriptor, encode_md1_string, encode_payload, split` (`chunk.rs:240` `pub fn split(&Descriptor)->Result<Vec<String>>`, no gate), `OriginPath/PathComponent/PathDecl/PathDeclPaths`. All fields pub (`Descriptor{n,path_decl,use_site_path,tree,tlv}` encode.rs:17-28; `TlvSection` 5 pub fields tlv.rs:24-39; `PathDecl/OriginPath/PathComponent/UseSitePath/Alternative` all pub). (b) Precedents `wallet_policy.rs:209,227,248,318` + `per_key_use_site_override.rs:128` use bare `Descriptor{…}` literals. (c) Tag-emit loci EXACT: pubkeys tlv.rs:149-172 (push :171); use-site :99-122 (push :121); divergent header bit encode.rs:113-117 (:116 `matches!(…Divergent(_))`); divergent in-order loop origin_path.rs:125-127. (d) NO-BUMP genuine (new `tests/wire_golden.rs` only; no dev-dep — `[u8;65]` arrays, no derivation; `Vector` struct + generator `parse_template(v.template,&[],&fps)` confirm corpus-export would need production edits in BOTH crates). (e) Deferred FOLLOWUP correct. 15 MANIFEST entries `keys:&[]`; `wsh_sortedmulti_2chunk` test_vectors.rs:94; `.descriptor.json` matches recon shape.

**I4 (chunk-string golden) — CLOSED.** `split(&d)` deterministic-pure (encode_payload chunk.rs:245 + derive_chunk_set_id :248-249; byte slicing :266-273); generator emits `phrase.txt` via the same `split` (cmd/vectors.rs:58-71). A `.bytes.hex` golden cannot RED a :266-273 mutation (bytes.hex written pre-split). No md-codec test references `2chunk`/`phrase.txt` (grep=0) → genuine `cargo test -p md-codec` gap-close.

**I1/I2 (T3-a mechanics) — CLOSED.** `SRC_MK1=0b00`/`SRC_MD1=0b01` private consts pipeline.rs:60,62, read only by source_kind_code/from_code :196-209 (single-locus, round-trip-symmetric via parse_header:397; no test pins them). build_h0 pipeline.rs:213-220 (pushes :216-217; read :394-395 = 2 mirrored lines). CRC5_POLY sync.rs:43 (`0b10_0101` primitive; tests/sync.rs:71 → must stay primitive, correct). RS-parity DROP-unless-complemented correct (rs_parity rs.rs:134-161; single-line reversal ≈m>⌊m/2⌋ → Uncorrectable → round-trips RED; complement locus pipeline.rs:1060 exact). Supporting: lib.rs:39-42 exports; tests/pipeline.rs enc/dec :52/:61; wc-codec deps bip39+sha2 only.

**I3 (TLV citation) — CLOSED.** Tag consts tlv.rs:11-19 (`TLV_USE_SITE_PATH_OVERRIDES=0x00` :11 / `TLV_PUBKEYS=0x02` :16); both write (:121,:171) + read (:212+) share them; grep across md tests/src = 0 pins. Bonus: the corpus's 15 vectors emit only `TLV_FINGERPRINTS=0x01`, so the 0x00↔0x02 swap leaves the committed corpus byte-identical → `vectors_output_matches_committed_corpus` survives, only new goldens RED.

## Findings
Critical: none. Important: none.
**Minor (implementation-time nits, no re-round):**
1. The (e) chunk-split mutation ALSO REDs md-cli `vectors_output_matches_committed_corpus` (`vector_corpus.rs:15-42` diffs regenerated `phrase.txt`) — expected; it's a frozen-oracle diff (`#[cfg(all(unix,feature="json"))]`), not a round-trip test, so acceptance #1 holds. [FOLDED into SPEC line 30.]
2. M3 "fixed `array_id_seed`" wasn't textual — structurally forced (`raid_encode(…,array_id_seed:&[u8],…)` raid.rs:268-270). [FOLDED line 14.]
3. M1 "freeze both sides" — wc-codec has no mk/md deps → input `(payload,payload_bits)` must be frozen consts too. [FOLDED line 14.]
4. Citations: `cmd/bytecode.rs` encode_payload at :15 (not :14); `word_card_adapter.rs:190-198`. [FOLDED.]

## VERDICT: GREEN (0C/0I)
Both Criticals closed as prescribed; all four Importants closed with live-source-exact loci; no new gaps. T3 implementation may begin.

---
**FOLD STATUS (opus, 2026-07-10):** 4 Minors folded into the SPEC (this pass). T3 GREEN — two implementers dispatched: toolkit (T3-a wc-codec + T3-c payload_bits, disjoint files) ‖ md (T3-b direct-construction + chunk-string golden). NO-BUMP; RED-under-mutation first.