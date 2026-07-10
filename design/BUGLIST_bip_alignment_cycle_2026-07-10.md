# BUG LIST — BIP-alignment cross-repo cycle (2026-07-10)

**Origin:** Fable adversarial BIP-vs-impl reviews of md1 (`descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki` @ `ef1f3e71`) and mk1 (`mnemonic-key/bip/bip-mnemonic-key.mediawiki` @ `1c9fbf7`). Verbatim reviews: `design/agent-reports/bip-review-{md1,mk1}-fable-r0.md`.
**User directive:** "BIP-alignment cross-repo cycle now" + "Keep a list of all the bugs, we need to address them all."
**Scope:** align both BIPs → shipped code (no wire change), embed missing vectors, AND fix the live code bugs the review surfaced. Then test-hardness → minors → docs.

**Status legend:** ☐ open · ⧗ in progress · ☑ done · 🔬 VERIFIED (independently reproduced by me with the real binary) · ⚖ needs product decision.

---

## BUCKET A — LIVE CODE BUGS (shipped binaries; fix in the repo)

These are real defects independent of BIP text. All four reproduced with `target/release/md` @ `ef1f3e71`.

- ☐ 🔬 **A1. `sh(wpkh)` elided-origin self-rejecting card** (md1 C2 sub-finding; **funds/usability**). `md encode 'sh(wpkh(@0/<0;1>/*))'` (no `--path`) → `md1yqpqqxpsq258xsks3kh0ye`; `md decode` on it → `non-canonical wrapper requires explicit origin for @0, but none provided`. Root cause: `sh(wpkh)` absent from `crates/md-codec/src/canonical_origin.rs` table → `None` → encoder emits an elided depth-0 card the decoder then refuses. **Known/documented** asymmetry (`test_vectors.rs:36-40`: "encode requires explicit origin; decode strips canonical … 49'/0'/0'"). Fix options: (a) add `sh(wpkh)`→`m/49'/0'/0'` to the canonical table (symmetric round-trip), or (b) make encode fail-closed at encode time instead of emitting a self-rejecting card. Repo: `descriptor-mnemonic`.
- ☐ 🔬 **A2. `md decode` cannot read chunked cards** (md1 I1; **recovery/usability**). `md decode <chunk-string>` → `wire-format version mismatch: got 9, expected 4`; `md repair <same>` round-trips. Only `chunk.rs::decode_with_correction` (behind `md repair`) auto-dispatches on the in-band chunk bit; `decode.rs::decode_md1_string` (behind `md decode`) does not. Fix: route `md decode` through the dispatching path (or add auto-dispatch to `decode_md1_string`). Repo: `descriptor-mnemonic`.
- ☐ 🔬 **A3. `md encode --force-long-code` is a silent no-op** (md1 C3). Output byte-identical with/without the flag (long code was dropped from the impl). Fix: remove the flag (regular-only) OR reinstate long-code support (feature, out of scope — FOLLOWUP). Repo: `descriptor-mnemonic`.
- ☐ 🔬 **A4. `pkh` / `sh(multi)` / `sh(sortedmulti)` accepted despite BIP MUST-reject** (md1 C4). **DECISION (user 2026-07-10): HYBRID.** Decode stays permissive for ALL forms (recovery-safe — never lose the ability to read a card that may already be engraved; `pkh_basic` stays in the corpus). Encoder keeps producing the card but emits a **calibrated** advisory: a real footgun warning for bare `sh(multi)`/`sh(sortedmulti)` (legacy P2SH multisig — third-party txid malleability, 520-byte redeemScript ≤~15-key ceiling, no witness discount, universally superseded by `wsh`/`sh(wsh)`); a **milder or no** note for `pkh` (legacy but funds-safe — rationale to reject it was always weak). BIP re-scoped to match: delete the sh-matrix MUSTs + the "rejects sh(multi) at spec level" footgun FAQ claim; document the permissive-decode + encode-advisory model. Advisory calibration + exact gate (warn-only vs `--allow-legacy`) = SPEC detail. Repo: both. **No wire change.**

- ☐ **A1b. Self-rejecting-card defect is a CLASS (R0 C-1) → SPUN OUT to a separate brainstorm** (user 2026-07-10). `md encode` mints un-decodable "dead cards" for EVERY `canonical_origin=None` shape without `--path` (tr+tree, sh(sortedmulti), bare wsh, miniscript) — R0-verified (`md1yppqqxqj2s4dk6hk0wrt5n3` etc. all decode-reject). **User direction: NOT a hard encode refusal.** Instead the redesign is: (1) a **loud encode-time advisory** when backing up pathless wallets (encoder still emits), + (2) a **decoder modified to partial-decode** — output the template with **placeholders** for keys/paths not supplied, instead of rejecting `MissingExplicitOrigin`. Both halves = a **separate brainstorm** (`pathless-wallet-backup-partial-decode`), NOT this cycle. **Consequence:** the C-1 class stays OPEN until that brainstorm ships; this cycle's md1 BIP marks the non-canonical/pathless case "under separate design" rather than locking MUST-reject. `sh(wpkh)→49'` (A1) still lands (real canonical path beats a placeholder).

## BUCKET A′ — CODE-COMMENT / MINOR CODE FIXES (surfaced by review)

- ☐ **A5. `md-codec bch.rs` wrong comment** (md1 M1): asserts codex32/BIP-93 init is `1`; BIP-93's `ms32_polymod` init IS `0x23181b3` (reviewer verified vs BIP-93 source). Fix comment. Repo: `descriptor-mnemonic`.
- ☐ **A6. `mk-codec bch.rs` self-contradicting docs** (mk1 M5): `bch.rs:181-198` (init=1, wrong) vs `:857-860` (0x23181b3, right); `:980` cites a BIP §"Checksum" that doesn't exist; `error.rs:115-117` calls 0x16 reserved; `error.rs:57` repeats the "8 substitutions for long code" myth. Fix comments. Repo: `mnemonic-key`.
- ☐ **A7. `mk-codec` stale `family_token`** (mk1 M4): crate is 0.4.1 but `consts.rs:50` `family_token` still `"mk-codec 0.2"`; §Test Vectors' "minor bumps roll the token" convention no longer matches practice. Reconcile token OR the convention. Repo: `mnemonic-key`.
- ☐ **A8. `md-codec` fictional error variant** (md1 I4): BIP cites `Error::InvalidBytecode{ kind: BytecodeErrorKind::MalformedPayloadPadding }` — no such variant exists; TLV rollback (`tlv.rs:286-302`) tolerates ≤7 trailing bits WITHOUT checking they're zero → non-zero pad silently accepted. Fix: implement the zero-check (+ real variant) OR soften the BIP to the rollback contract. Repo: `descriptor-mnemonic` (+ BIP).
- ☐ **A9. toolkit `chunk_set_id` XOR-slot vs BIP formula** (mk1 M3): toolkit XORs cosigner slot index (`synthesize.rs:73-75,90-92`); the BIP's derivation example matches only slot 0 → byte-identical bundle reproduction from the BIP formula fails for slots ≥ 1 (interop-unaffected; field opaque). Fix: BIP documents the slot-XOR. Repo: `mnemonic-toolkit` BIP-adjacent / mk1 BIP.

---

## BUCKET B — md1 BIP → code alignment (`descriptor-mnemonic`)

### Critical
- ☐ 🔬 **MD1-C1 (=D1). Chunk framing: BIP mandates bit-boundary + "accept any valid division"; code splits at byte boundaries + floor-drops ≤7 bits/chunk.** Every real ≥2-chunk card unreadable by a spec-only impl, both directions (reviewer live-demoed: re-split at bit 259 → reference rejects `tag 0x3b out of range`). Internal contradiction: 3 incompatible framings coexist (bit-exact / byte-aligned / 8-symbol+3-slack). Fix BIP→code: fragments = whole bytes, boundary at bit 37 contiguous, delete "any valid division" + slack framing, fix fragment-max arithmetic, **commit a ≥2-chunk vector**.
- ☐ 🔬 **MD1-C2. Elided-origin (`m` = canonical default) convention entirely unspecified.** BIP says path "is encoded explicitly"; code encodes depth-0 `m` + resolves via `canonical_origin.rs`. BIP-only decoder derives at master node → wrong wallet on most committed vectors (verified: `wsh(multi(2,…))` no-path → `md1yppqqxppsg2vlumagltz27le`, `path_decl:"m"`). Also unspecified: `MissingExplicitOrigin` decode-reject rule; default-path table disagrees with `canonical_origin.rs` (BIP has sh(wpkh)→49', omits pkh; code has pkh→44', omits sh(wpkh) → **A1**). Fix: specify elision convention normatively + reconcile tables.
- ☐ 🔬 **MD1-C3. Long code specified but not implemented; 93-cap location wrong.** BIP normatively defines BCH(108,93,8) long code; impl is regular-only (cap 80 data symbols / 400 bits). BIP §Length-envelope wrongly counts HRP into the 93. Both directions break in the 386–400-bit band. Fix BIP→code: excise long code, restate cap ≤80 data / ≤93 codeword (HRP excluded) / ≤400 bits, chunk above. (Couples with **A3**.)
- ☐ 🔬 **MD1-C4. pkh/sh(multi) scope MUSTs vs code acceptance** — see **A4** (product decision). Corpus ships `pkh_basic`; FAQ claims footgun-rejection that's untrue of code.

### Important
- ☐ **MD1-I1. Auto-dispatch self-contradictory + half-implemented** — see **A2**. BIP line 194 (in-band, no hint) vs 330 (by reader role). Make in-band normative; add "got 9" trace row.
- ☐ **MD1-I2. PolicyId / WalletInstanceId as specified don't exist.** No `PolicyId`/`WalletInstanceId` types; the 12-word anchor is `WalletPolicyId::to_phrase()` over an elaborate canonical-record preimage no BIP reader could reconstruct; wire carries 65-byte entries so the BIP's 78-byte `bip32_serialize` concat isn't computable from card data. Independent tool's 12-word phrase never matches → cross-verification silently fails. Fix: rewrite §Policy-identifier/§Wallet-Instance-ID to the actual `WalletPolicyId` preimage (documented `identity.rs:141-186`); reconcile line 217 vs 798.
- ☐ **MD1-I3. Decoder-side canonical-form validators unspecified.** `PlaceholderNotReferenced`, `PlaceholderFirstOccurrenceOutOfOrder`, `MultipathAltCountMismatch`, `BaselineUseSiteOverride`, `RedundantUseSiteOverride`, tap-leaf forbidden-tag set, `MissingExplicitOrigin`, `InvalidXpubBytes` — all reject wires the BIP permits. Fix: enumerate every decode-rejection rule in the BIP.
- ☐ **MD1-I4. Non-zero padding rejection: fictional error + not implemented** — see **A8**.
- ☐ **MD1-I5. Guided recovery + erasure decoding + confidence reporting REQUIRED by BIP, entirely absent.** Zero erasure code; `md repair` reports no confidence tiers. Reference non-conformant with own MUSTs. Fix: downgrade to SHOULD/informative (or companion spec) until implemented — ⚖ or implement (large; FOLLOWUP).
- ☐ **MD1-I6. Test-vector section misdescribes machinery + no ≥2-chunk vector.** `wsh_multi_chunked` is chunked-of-1; regeneration command `--test vectors` doesn't exist (it's `md vectors --out` / `src/test_vectors.rs::MANIFEST`); "three plain-text files" table lists four. Fix: commit ≥2-chunk + 94–96-char-boundary + NUMS-taproot vectors; correct machinery text. (Couples MD1-C1/C3.)
- ☐ **MD1-I7. Unknown-TLV: BIP says "skip", impl preserves-and-re-emits — CSI-visible.** Skip-style decoder CSI-mismatches any future chunked card with a reserved-tag TLV. Fix: specify preserve-verbatim (incl. re-emission ordering) as normative.
- ☐ **MD1-I8. Varint canonicality unspecified.** `read_varint` accepts non-minimal `L`/`l_high==0`; non-minimal chunked card → wire CSI ≠ derived CSI → rejected. Fix: mandate minimal encodings for encoders; state decoder leniency.
- ☐ **MD1-I9. Chunk-count sizing / auto-chunk mismatch (SHOULD).** Impl uses 320-bit budget (emits more chunks than minimal); CLI requires `--force-chunked` rather than auto-chunking. Document both.

### Minor
- ☐ **MD1-M1** bch.rs init comment — see **A5**.
- ☐ **MD1-M2** line 798 "12 words = 128 bits" → 132 bits (128 entropy + 4 checksum).
- ☐ **MD1-M3** stale/fabricated refs: `md-signer-compat` workspace member (not in repo); FAQ `gen_vectors --output`/`v0.2.json` machinery (nonexistent); "Python/TypeScript bindings planned v0.2"; wrong `rust-miniscript#1` link; CLI list omits `repair`/`address`/`compile`/`gen-man`.
- ☐ **MD1-M4** §"Why descriptor-codec" stale v0.x text (path dictionary contradiction; "@i encodes as 2 bytes" — actually kiw ≤5 bits).
- ☐ **MD1-M5** error-taxonomy: `PathDepthExceeded`/`KeyCountOutOfRange`/`AltCountOutOfRange`/`ChunkCountOutOfRange` "decoders MUST reject depth>15" are wire-impossible (bounded fields) → state as encoder-side.
- ☐ **MD1-M6** preamble not BIP-2-conformant (`BIP: ?`, non-standard Status, empty Comments-URI).
- ☐ **MD1-M7** erasure-guarantee table needs burst-erasure citation (moot until I5).
- ☐ **MD1-M8** §Definitions "canonical bytecode is unique" only true given I3/I8 unwritten rules — point at them.

---

## BUCKET C — mk1 BIP → code alignment (`mnemonic-key`)

### Critical
- ☐ **MK1-C1. BCH checksum algorithm ENTIRELY UNSPECIFIED.** No §Checksum, no init, no pseudo-code, no worked example. 2 of 3 good-faith readings reject **every card ever engraved**; only undocumented `init 0x23181b3 + prepend hrp_expand("mk")` works (reviewer confirmed vs vector V1). Fix BIP→code: add normative §Checksum (init `0x23181b3`, hrp_expand prepend, append 13/15 zeros, XOR target, verify equation, worked example) — mirror md1's algorithm exactly.
- ☐ **MK1-C2. Depth-0/empty-path: BIP mandates rejecting what the encoder emits** (pure D1-class). BIP `component_count MUST be 1..=10`; mk-codec v0.4.0+ accepts/emits `0..=10` (depth-0 = no-path key, `bytecode/path.rs:112-131`). Internal SPEC updated in lockstep; **BIP not** (FOLLOWUP `mk1-no-path-depth0-support`). Toolkit `mnemonic bundle --slot wif=…` ships this. Fix BIP→code: `0..=10`, depth-0/`Normal{0}` semantics, "2..=52 B", add depth-0 vector.
- ☐ **MK1-C3. BIP carries ZERO test vectors** (says "to be written") while a 40-vector corpus (18 pos incl. two 3-chunk + 22 neg) ships since v0.1. Given C1+C2, an implementer has no self-check. Fix: embed/pin positive vectors (per-chunk strings, canonical_bytecode_hex, chunk_set_id, total_chunks) + negatives + depth-0 vector; update stale prose.

### Important
- ☐ **MK1-I1. `XpubOriginPathMismatch` encoder invariant absent;** "drift impossible by construction" false as stated. Code rejects xpub whose depth/child disagree with origin_path (`encode.rs:31-48`). Fix: port SPEC §4's encoder-invariant paragraph verbatim.
- ☐ **MK1-I2. Long-code substitution-correction misstated as 8 (it's 4)** in 3 places (lines 29, 480, 504) contradicting line 144. 8 is the detection radius. Fix text (+ `error.rs:57` comment — see A6).
- ☐ **MK1-I3. Erasure-correction guarantees + reporting states normative but unimplemented** (mirrors MD1-I5). Fix: downgrade to MAY/note or implement.
- ☐ **MK1-I4. Encoder chunking unspecified + line 74 factually wrong** ("2 long-code chunks"; reality: chunk 0 long, chunk 1 regular — mixed is normal). Fix: specify 53-byte-fragment emit policy + "decoders MUST accept any division"; correct line 74. Contrast md1's bit-boundary model explicitly.
- ☐ **MK1-I5. §Linkage line 400 superseded stub formula** ("canonical-bytecode SHA-256 prefix") contradicts lines 37/48/268/404 (stub = top-4-bytes of WalletPolicyId). Code stubs are form-aware (WalletPolicyId OR WalletDescriptorTemplateId for keyless template). Fix: rewrite line 400; add keyless-template stub rule.
- ☐ **MK1-I6. Accepted xpub version-byte set unspecified.** Code = exactly `{0x0488B21E, 0x043587CF}`; BIP names only mainnet. Fix: enumerate both; all others → `InvalidXpubVersion`.

### Minor
- ☐ **MK1-M1** §Decoder validity omits string-layer errors the corpus pins (InvalidHrp, MixedCase, InvalidChar, InvalidStringLength, BchUncorrectable, empty-input, CardPayloadTooLarge).
- ☐ **MK1-M2** minimum data-part length (14 symbols) unstated.
- ☐ **MK1-M3** chunk_set_id slot-XOR — see **A9**.
- ☐ **MK1-M4** stale family_token — see **A7**.
- ☐ **MK1-M5** code-doc cleanups — see **A6**.
- ☐ **MK1-M6** line 116 pre-submission structural-relationship audit commitment vs FOLLOWUPS closure — align text or re-open.

---

## CROSS-CUTTING
- Both BIPs share `POLYMOD_INIT 0x23181b3` + per-HRP NUMS targets (confirmed consistent w/ pinned residue architecture). md1 checksum §exists (w/ wrong init comment A5); mk1 checksum §absent (MK1-C1) — the C1 fixes must be algorithm-identical, HRP/target-different.
- md1 = bit-boundary framing, mk1 = byte-boundary fixed-53B — **different by design**; the MD1-C1 / MK1-I4 fixes must state the contrast so implementers don't cross-import.
- Both need embedded ≥2-chunk vectors (MD1-I6, MK1-C3).
- Erasure/guided-recovery downgrade is symmetric (MD1-I5 / MK1-I3) — decide once, apply both.

---

## BIP DOWNGRADE LEDGER (user 2026-07-10: "make BIPs honest but keep a list so we can implement later")

Every place we soften a BIP MUST→SHOULD/informative or excise a specified-but-unimplemented feature. Each gets a FOLLOWUP so the capability can be built later and the BIP re-upgraded. **Ground-truth check done:** erasure decoding exists ONLY in `wc-codec` (word-card RS layer: `recover_up_to_m_erasures`, RAID, single-deletion candidate search) — NOT in md-codec/mk-codec, whose `repair` does BCH *substitution*-error correction (BM+Forney, t=4) only. So all md1/mk1 erasure/guided-recovery MUSTs are genuine downgrades.

| # | BIP | What we downgrade/excise | From → To | Re-implement FOLLOWUP | Notes |
|---|-----|--------------------------|-----------|----------------------|-------|
| DG-1 | md1 (I5) + mk1 (I3) | **Erasure-aware BCH decoding** ("correct up to 8 erasures / 13–15 consecutive") | MUST → SHOULD/informative (code-distance note) | `impl-bch-erasure-decoding-md-mk` | Code distance (n−k parity) supports it; the BM+Forney decoder implements substitution errors only. Port wc-codec-style erasure marking to the md/mk BCH layer. |
| DG-2 | md1 (I5) + mk1 (I3) | **Guided recovery** (constrained radius-12 structure search) | REQUIRED → SHOULD | `impl-guided-recovery-md-mk` | `repair` does BCH correction, not structure-elicited candidate search. wc-codec's single-deletion candidate search is the nearest prior art. |
| DG-3 | md1 (I5) + mk1 (I3) | **Confidence-tier reporting** (4-tier outcome/method/confidence ladder) | MUST → SHOULD | `impl-confidence-tier-reporting-md-mk` | `repair` reports corrections but no tiers. Keep the honest substitution-correction reporting; ledger the tier ladder. |
| DG-4 | md1 (C3 / A3) | **Long BCH code** (BCH(108,93,8), 96–108-char single strings) | Excised from BIP | `reconsider-md1-long-code` | Code is regular-only (≤80 data / ≤400 bits, then chunk). Reinstating gives bigger single-string cards (fewer chunks). Also remove/So-note `--force-long-code`. |
| DG-5 | md1 (I4 / A8) | **Non-zero trailing-pad rejection** (IF we soften rather than implement) | TBD in SPEC | `impl-nonzero-pad-check-md` | Prefer IMPLEMENT the cheap zero-check + real error variant (removes the downgrade). Ledger only if SPEC opts to soften. |

**Kept (NOT downgraded — honest as-is):** BCH substitution-error correction (`repair` genuinely does t-error correction on both codecs) stays a MUST/normative.

---

## SEQUENCING (user 2026-07-10: **BIPs → bugs → test-hardness → minors → docs**)

**Rationale for BIP-first:** the BIP is the normative spec; it should state the *correct target* behavior, and the code bugs are the code failing to meet it. So the BIP phase writes the corrected spec (incl. sh(wpkh)→49', auto-dispatch, non-zero-pad-reject, permissive-decode scope); the bugs phase makes the code conform + generates code-dependent vectors.

0. ✅ **Decisions resolved** (A4 hybrid; erasure/long-code → make-honest + DG ledger).
1. **BIPs (doc) FIRST** — both md1 + mk1 BIP text corrections stating correct normative behavior; embed non-code-dependent vectors. Coupled findings (A1/A2/A8/A4 targets) are written as the spec target; a **conformance-gap list** records which code fixes the bugs phase must land to match. Defer code-dependent vector generation (sh_wpkh) to phase 2.
   - md1: C1-C4 text + I1-I9 + M1-M8; ≥2-chunk + boundary + NUMS vectors (generable now).
   - mk1: C1 (§Checksum) + C2 (depth-0) + C3 (embed vectors) + I1-I6 + M1-M6.
2. **Bugs (code) SECOND** — make code conform to the phase-1 BIPs: A1, A2, A3, A8, A4-advisory, A5 (md) + A6, A7 (mk); TDD; generate/validate coupled vectors (sh_wpkh); release codecs (md-codec+md-cli; mk-codec+mk-cli PATCH). **Toolkit re-pin DEFERRED** (R0 round-3 I-1): F-A1 flips toolkit descriptor-mode sh(wpkh) 48'/0'/0'/1'→49' = same-command-different-wallet → re-pin becomes its own tested follow-up (`toolkit-repin-sh-wpkh-canonical-flip`), NOT this cycle.
3. **Test-hardening** (constellation-eval §2 #3-#15) — property/oracle harnesses + wire goldens (now gate via Cycle I).
4. **Minors** (eval M2 SLIP-39, M3 BIP-85 Portuguese, M4 BSMS Round-1 exit code, D1 note — distinct from the BIP minors above).
5. **Docs** (gui-manual repair exit-code lockstep — deferred docs item 3).

Post-impl whole-diff review per phase; release ripple (codec bumps → toolkit re-pin if API/behavior changes).
