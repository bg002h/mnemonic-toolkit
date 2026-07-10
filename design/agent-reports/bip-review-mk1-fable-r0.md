# BIP review — mk1 (`bip-mnemonic-key.mediawiki`) vs `mk-codec` — Fable, adversarial, read-only

**Reviewer:** Fable (BIP-vs-impl recovery-independence review, read-only, adversarial). Persisted verbatim per CLAUDE.md.
**Dispatched:** 2026-07-10. Target: `mnemonic-key/bip/bip-mnemonic-key.mediawiki` @ `origin/main 1c9fbf7` (541 lines) vs `crates/mk-codec` v0.4.1 (mk-cli v0.12.0).
**Verification performed by the reviewer:** all four NUMS constants recomputed from domain strings (match); vector V1 (`v0.1.json`) decoded end-to-end from first principles — BCH residues of both chunks, header field unpacking, 53/35-byte fragment split, cross-chunk SHA-256 hash, and canonical bytecode hex all reproduce the shipped strings **only** under the undocumented checksum rule in C1.

---

## CRITICAL

### C1. The BCH checksum algorithm is unspecified; 2 of 3 good-faith readings reject every real card
- **BIP:** lines 84–116 (§"BCH plumbing", §"Why new target constants?"). The only normative content is: "reuses BIP 93's generator polynomials verbatim", "HRP-mixing (BIP 173-style HRP expansion folded into the polymod's initial state)", and the two target constants. There is **no §Checksum section, no pseudo-code, no polymod init value, no checksum-generation or verification equation, no worked example** anywhere in the BIP.
- **Code:** `crates/mk-codec/src/string_layer/bch.rs` — `polymod_run` (line 287) seeds `POLYMOD_INIT = 0x23181b3` (line 198), and `bch_create_checksum_regular`/`bch_verify_regular` (lines 304–329) **prepend `hrp_expand("mk")` = [3,3,0,13,11] as input symbols**, append 13 (regular) / 15 (long) zeros, and XOR the target constant; verify condition is `polymod == MK_*_CONST`.
- **Concrete failure (empirically confirmed against published vector V1, chunk 1):**
  - init `0x23181b3` + prepend `hrp_expand("mk")` → residue == `MK_REGULAR_CONST` ✓ (the reference rule);
  - init `1` + prepend `hrp_expand("mk")` (the literal BIP-173 reading the BIP text invites) → ✗;
  - init = fold of `hrp_expand("mk")` from 1 = `0x23181ab` (the most faithful reading of "HRP expansion folded into the initial state") → ✗.

  `0x23181b3` is itself the fold of `hrp_expand("ms")` from init 1 — i.e., mk1's checksum input is effectively `hrp_expand("ms") ‖ hrp_expand("mk") ‖ data` starting from 1. No reader can derive this from the BIP text; two of the three plausible implementations reject **every card ever engraved**. Compounding: the reference's own docs contradict each other on this exact point (`bch.rs:181–198` claims BIP-93 starts from 1; `bch.rs:857–860` correctly says `0x23181b3` is BIP-93's published init), and `bch.rs:980` cites a BIP §"Checksum" that does not exist.
- **Fix (BIP→code, no wire bump):** add a normative §Checksum with the full polymod pseudo-code including the explicit `0x23181b3` seed, the `hrp_expand("mk")` prepend, the create procedure (append 13/15 zeros, XOR target), the verify equation, and a worked example. Mirror whatever the md1 BIP does here (shared init + algorithm; only HRP and targets differ).

### C2. Depth-0 / empty-path cards: the BIP mandates rejecting what the reference encoder emits (exact md1-D1 class)
- **BIP:** line 332 (`component_count: 1 byte; MUST be in 1..=10`), line 336 and line 378 (decoder rule 5: reject `component_count > 10` **or `== 0`** with `Error::PathTooDeep`), lines 68/256 (explicit-path escape sized "3..=52 B"), lines 358–359 (`child_number := last_component(origin_path)` — undefined for an empty path).
- **Code (since mk-codec v0.4.0):** `bytecode/path.rs:112–131` — `decode_explicit_path` **accepts `count == 0`** as the no-path/depth-0 key; `encode_path` emits `[0xFE, 0x00]` (test `round_trip_empty_path`, line 262); `bytecode/xpub_compact.rs:86–108` reconstructs `depth = 0`, `child_number = Normal{0}`; `bytecode/encode.rs:31–48` accepts consistent depth-0 cards. The internal SPEC was updated in lockstep (`design/SPEC_mk_v0_1.md` lines 229, 237, 286: "MUST be in **0..=10**", "`component_count == 0` is **valid** as of v0.4.0") — **the BIP was not** (FOLLOWUP `mk1-no-path-depth0-support`, `design/FOLLOWUPS.md:365`, tracks the toolkit companion but no BIP lockstep).
- **Concrete failure:** the toolkit ships this surface (`mnemonic bundle --slot wif=…` builds a depth-0 xpub with empty origin_path, per the FOLLOWUP). A spec-conforming independent decoder rejects that card (`PathTooDeep`) → the engraved backup is unrecoverable under any BIP-only implementation. A spec-conforming encoder can't produce it at all. This is precisely the md1 D1 divergence class the review was tasked to find.
- **Fix (BIP→code):** update to `0..=10` with no-path semantics; add the `Normal{0}`/depth-0 reconstruction default to the compact-73 rule; change "3..=52 B" to "2..=52 B" (lines 68, 256); add a depth-0 test vector — **none exists even in the JSON corpus** (V1–V18 all have non-empty paths).

### C3. The BIP carries zero test vectors; §Test Vectors stale-claims they are "to be written" while a 40-vector corpus has shipped since v0.1
- **BIP:** lines 512–516. **Code:** `crates/mk-codec/src/test_vectors/v0.1.json` — 18 positive vectors (all ≥2 chunks, incl. two 3-chunk cards, testnet, no-fp, 3-stub, explicit-path, max-depth) + 22 negatives (one per decoder-reachable Error variant).
- **Concrete failure:** given C1 (checksum not reconstructable from the text) and C2 (contradictory path rule), an independent implementer has **no self-check whatsoever** — the exact mechanism that let md1's D1 survive review. With even one embedded 2-chunk vector, both C1 and C2 become self-catching. Under the recovery-independence rubric this is a Critical gap, not housekeeping: the artifact cannot fulfill its stated purpose without them.
- **Fix:** embed (or normatively pin by SHA-256) the positive vectors — per-chunk strings, `canonical_bytecode_hex`, pinned `chunk_set_id`, `total_chunks` — plus the negative table; add the missing depth-0 vector; update the stale prose.

---

## IMPORTANT

### I1. `XpubOriginPathMismatch` encoder invariant absent; "drift is impossible by construction" is false as stated
- **BIP:** lines 364, 396 claim the depth/child drift class is "impossible by construction"; the encoder-side-invariant paragraph (line 385) covers only the fingerprint flag. **Code:** `bytecode/encode.rs:31–48` rejects any xpub whose `depth`/`child_number` disagree with `origin_path` (`Error::XpubOriginPathMismatch`); internal SPEC §4 (`SPEC_mk_v0_1.md:295–304`) states it as a normative encoder MUST, including the empty-path `Normal{0}` clause.
- **Failure:** a BIP-only encoder silently emits a card that decodes to a **different-metadata xpub** (drift is impossible on the wire, but not between the supplied xpub and the path); the reconstructed 78-byte serialization then fails the Wallet Instance ID check at recovery, or silently imports wrong metadata where no external anchor exists. **Fix (BIP→code):** port SPEC §4's encoder-invariant paragraph verbatim.

### I2. Long-code substitution-correction capacity misstated as 8 in three places (it is 4)
- **BIP:** line 29 ("…up to 4 character substitutions for the regular code, **8 for the long code**"), line 480 (FAQ, same claim), line 504 ("4–8… depending on code variant") — contradicting the BIP's own correct §"Error-correction guarantees" line 144 ("Correction of up to 4 character substitutions").
- **Code:** both decoders cap at t = 4 (`bch.rs::bch_correct_long` doc: "full t = 4 capacity of BCH(108,93,8)"; `bch_decode.rs::decode_long_errors`). 8 is the *detection* radius, not correction. A user triaging a damaged card on the BIP's 8-substitution promise plans wrong. (Same myth persists in `error.rs:57`'s comment — flag for code-doc cleanup.)

### I3. Erasure-correction guarantees and reporting states are normative in the BIP but unimplemented anywhere in the codec
- **BIP:** lines 145–146 ("Correction of up to 8 erasures", "13 (regular) / 15 (long) consecutive erasures"), line 150 ("decoders MUST report… / N erasures corrected / structure-aided / failed").
- **Code:** zero erasure support in mk-codec (no hit for "erasure" in `crates/`); `decode_string` rejects any non-alphabet character (`InvalidChar`), so erasure positions cannot even be marked on input; the public `decode` (`pipeline.rs:118–152`) discards even the substitution-correction report ("structure-aided" exists only as the separate toolkit `repair` feature).
- **Fix:** downgrade erasure/burst correction to a code-capability note or MAY; define an erasure-marking input convention if it is meant to be conformance-testable; scope the MUST-report to clean/N-substitutions/failed — or implement.

### I4. Encoder chunking algorithm unspecified — and line 74's "lands in 2 long-code chunks" is factually wrong
- **BIP:** line 74. **Reality (code + verified vector V1):** chunk 0 = long code (108 chars), chunk 1 = **regular** code (77 chars). Mixed-code chunk sets are the *normal* v0.1 emit shape (`pipeline.rs` module doc).
- The BIP never states: fragments are successive **53-byte** slices of `bytecode ‖ hash` with the last fragment the remainder (`chunk.rs::split_into_chunks`, frag_size = `CHUNKED_FRAGMENT_LONG_BYTES`); each fragment independently 8→5-bit converted with zero padding; per-chunk code variant auto-selected by data-part length; decoders accept **any** byte-level fragment division (reassembly concatenates — `chunk.rs::reassemble_from_chunks`). Contrast md1's BIP, which is explicit ("encoder-chosen bit boundaries… Decoders MUST accept any valid division", `bip-mnemonic-descriptor.mediawiki:246, 773`).
- **Failure:** an implementer may reject a real card's regular-code final chunk as anomalous, or bake in fragment-size assumptions; byte-identical re-engraving from the BIP alone is not guaranteed. **Fix (BIP→code):** specify the emit policy as normative-for-reproducibility + "decoders MUST accept any division"; correct line 74.

### I5. §Linkage line 400 still uses the superseded stub formula, contradicting the BIP's own definitions; template-form stubs absent
- **BIP:** line 400 ("…any MD-encoded policy whose **canonical-bytecode SHA-256 prefix** matches one of these stubs") contradicts lines 37/48/268/404 (stub = top 4 bytes of **WalletPolicyId**, explicitly "not the md1 bytecode hash"). An orchestrator implementing line 400 computes stubs that match no post-md-v0.13 card → step-2 filter rejects every card.
- **Code:** `key_card.rs:26–33` — stubs are **form-aware**: WalletPolicyId for keyed md1 **or WalletDescriptorTemplateId for keyless template md1** (toolkit #28, mk-cli `derive_stub_from_md1`). The template-form rule is entirely absent from the BIP. **Fix:** rewrite line 400; add the keyless-template stub definition.

### I6. Accepted xpub version-byte set unspecified
- **BIP:** line 347 names only mainnet `0x0488B21E`; rule 7 (line 380) says "a known network's xpub prefix". **Code:** exactly `{0x0488B21E, 0x043587CF}` (`xpub_compact.rs:25–28, 63–69`). Implementers may accept ypub/zpub/vpub or reject tpub → accept/reject divergence on testnet cards (V3 is a tpub card). **Fix:** enumerate both constants; all others → `InvalidXpubVersion`.

---

## MINOR

- **M1.** §Decoder validity rules omit the string-layer errors the corpus pins: `InvalidHrp`, `MixedCase`, `InvalidChar`, `InvalidStringLength`, `BchUncorrectable`, empty-input (`ChunkedHeaderMalformed`, `pipeline.rs:118–123`), `CardPayloadTooLarge` — despite §Test Vectors promising one negative per variant.
- **M2.** Minimum data-part length (14 symbols) unstated; BIP says regular = "up to and including 93" with no floor (`bch.rs::bch_code_for_length` rejects <14).
- **M3.** The chunk_set_id derivation example (line 182) matches the toolkit only for cosigner slot 0; the toolkit XORs the slot index (`mnemonic-toolkit/…/synthesize.rs:73–75, 90–92`). Interop-unaffected (field is opaque), but byte-identical bundle reproduction from the BIP formula fails for slots ≥ 1.
- **M4.** §Test Vectors' token convention ("minor-version bumps roll the token") no longer matches practice: crate is 0.4.1, `family_token` still `"mk-codec 0.2"` (`consts.rs:50`).
- **M5.** Pre-submission code-doc cleanups that would mislead an auditor: `bch.rs:181–198` vs `:857–860` contradict on BIP-93's init (the test is right); `bch.rs:980` cites a nonexistent BIP §"Checksum"; `error.rs:115–117` still calls 0x16 reserved; `error.rs:57` repeats the "8 for long" myth.
- **M6.** Line 116 commits to a pre-submission structural-relationship audit that FOLLOWUPS (`design/FOLLOWUPS.md:224`) closed at md1's lower reproducer-in-BIP bar — align the text or re-open the item.

**Cross-format note (axis 5):** shared `POLYMOD_INIT = 0x23181b3` + per-HRP NUMS targets confirmed in code and consistent with the constellation's pinned residue architecture; MD's published T_REGULAR/T_LONG re-derived and match. mk1 (byte-boundary, fixed 53-byte fragments, per-chunk padding) and md1 (bit-boundary, encoder-chosen division) frame chunks **differently by design** — the C1/I4 fixes should state this contrast explicitly so implementers don't import md1's model, and the checksum pseudo-code must stay init/algorithm-identical with md1's BIP.

---

## Submission-readiness verdict

**NOT submission-ready — and not yet recovery-independent.** The wire format itself is sound, internally consistent in code, and fully vector-pinned in-repo; but the BIP as a standalone artifact cannot produce a compatible implementation: the checksum layer is unimplementable from the text (C1), one shipped card class is spec-mandated to be rejected (C2), and there are no vectors to catch either (C3). The document also lags the v0.4.0 wire reality that the internal SPEC already reflects — the process gap is "BIP not in the lockstep set."

**Top-3 must-fix:**
1. **C1** — add a normative §Checksum (init `0x23181b3`, `hrp_expand("mk")` prepend, create/verify equations, worked example).
2. **C2** — align the explicit-path rule to `0..=10` with depth-0/`Normal{0}` semantics (+ "2..=52 B") and add a depth-0 vector.
3. **C3** — embed the test vectors (incl. ≥2-chunk and 3-chunk cards) in the BIP with pinned chunk_set_ids and canonical bytecode hex.
