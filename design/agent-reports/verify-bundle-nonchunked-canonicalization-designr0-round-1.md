# R0 Design Review (round 1) — verify-bundle non-chunked md1 canonicalization

**Reviewer:** Fable architect (`model:"fable"`), dispatched 2026-07-12.
**Cycle:** `toolkit-inspect-nonchunked-md1-intake-gap` — VERIFY-BUNDLE leg.
**Source SHA reviewed:** `de140a08` (HEAD == origin/master).
**Usage:** 45 tool-uses, ~783s, 127033 subagent tokens.
**Main-loop independent verification:** Q2 safety (validate.rs:221-224 + synthesize.rs:1205-1218), Minor#1 (verify_bundle.rs:634-635), Minor#2 (codex32.rs:25 = 400-bit cap), identity.rs:39-45 — all confirmed by me against source before accepting GREEN.

---

## Verdict: GREEN — 0 Critical / 0 Important (6 Minor/Nit)

The design's central claims all check out against source at HEAD (de140a08). The highest-risk question (Q2: does strict decode reject a legitimate template card?) resolves definitively in the design's favor: every card `bundle --md1-form=template` can emit strict-decodes, by construction of the emitter's C1-conditional origin mutation.

---

## Question-by-question findings

### Q1 — Is ~388 the sole classify gate, and is the fall-through observably a bug? CONFIRMED, both.
- verify_bundle.rs:386-408 is the only site routing into `verify_singlesig_template` (:394) and `verify_multisig_template` (:405). Grep of the whole file shows no other call sites (defs at :591, :859).
- Mechanism of the fall-through: a non-chunked wire's first 5-bit symbol is `[divergent][v3v2v1v0=0100]` (header.rs:27,31); `ChunkHeader::read` (chunk.rs:69-71) reads the first 4 bits as version → `0010` = 2 → `Err(WireVersionMismatch { got: 2 })`, swallowed by `if let Ok(d)` at :388.
- Observable bug, two ways: (a) without `--template` (the normal template-card invocation) → hard `ModeViolation` at :435-443, exit 2 (error.rs:637) on a valid card; (b) with `--template` + slots → general path decodes supplied md1 chunk-only (`reassemble_with_opts(..., partial())` at :2509 and :3113) → `md1_decode` FAIL → `mismatch`/exit 4 on a valid card. Wrong result either way; the fix is warranted.
- Nothing earlier blocks a non-chunked card: separator-strip (:279-302) and `validate_flag_hrp` (repair.rs:161-185, prefix-only, case-insensitive) both pass it.

### Q2 — Strict `decode_md1_string` semantics (THE funds question). CONFIRMED SAFE.
- `decode_md1_string` is strict-default (decode.rs:178-180); a chunk-form single routes internally to `reassemble_with_opts(&[s], opts)` (decode.rs:191-193), and `chunk::reassemble` ≡ `reassemble_with_opts(default)` (chunk.rs:311-313) — so the `[single]` chunk-form arm is byte-identical to today. Flag read is post-`unwrap_string` (BCH-verified; codex32.rs:139-195).
- The `MissingExplicitOrigin` trap does NOT fire on legitimate template cards:
  - `validate_explicit_origin_required` returns `Ok` immediately when `canonical_origin(&d.tree).is_some()` (validate.rs:221-224).
  - Single-sig template admissibility is exactly the 3 canonical-elidable types (`template_admissible`, synthesize.rs:1132-1141), all `canonical_origin = Some` (canonical_origin.rs:49-55).
  - Multisig/general templates: the emitter's mutation 3 is C1-conditional (synthesize.rs:1205-1218) — origins elided ONLY when `canonical_origin` is `Some`; a general (non-canonical) template keeps explicit origins on the wire precisely so `validate_explicit_origin_required` passes. The C1 regression the design worries about was already designed out at emit time.
- Strict-vs-partial routing parity: today's gate is strict `reassemble` (:388), so a dead/pathless card falls through today; strict `decode_md1_string` preserves that exactly. The inspect leg chose `partial()` (inspect.rs:256-261, rationale at :232-255) because inspect must render dead cards with a partial verdict — a different contract from verify-bundle's classify gate. The design's strict choice is correct.

### Q3 — `compute_md1_encoding_id` soundness. CONFIRMED.
- identity.rs:39-45: id = SHA-256(`encode_payload(d)` bytes)[0..16].
- `encode_payload` canonicalizes ONLY placeholder-index ordering (encode.rs:99-102); `path_decl` is written verbatim (`OriginPath::write`, origin_path.rs:54-66) — no origin-elision canonicalization at encode time, so the id stays sensitive to explicit-vs-elided origin, exactly as claimed. (identity.rs's "canonical-fill at encode time" comment refers to the WalletPolicyId L14 path, not `encode_payload`.)
- Chunk-agnosticism: `reassemble` strips chunk framing and hands one payload to `decode_payload` — two chunkings of the same payload decode to the same `Descriptor`, hence the same id.
- No false-match vector: the decoder REJECTS non-canonical placeholder ordering (`PlaceholderFirstOccurrenceOutOfOrder`, validate.rs:27-35), so decoded descriptors are already canonical, `encode_payload` is a faithful re-serialization, and id equality ⇔ identical canonical payload ⇔ identical descriptor content, up to a truncated-SHA-256 128-bit collision (computationally infeasible). In this path `d.n == 1`, so even placeholder canonicalization is the identity.
- No-regression proof (a) holds trivially; broadening (b) is exactly form-equivalence (chunking, case, checksum).

### Q4 — Is `d` in scope in `verify_singlesig_template`? CONFIRMED.
Signature at verify_bundle.rs:591-598 takes `d: &md_codec::Descriptor`; the gate passes `&d` at :394. Inside the function, the only supplied-md1 use is the :696 compare itself — no re-decode needed.

### Q5 — Is `expected.md1` always well-formed chunk-form? CONFIRMED.
`Bundle.md1: Vec<String>` (synthesize.rs:29), produced by `md_codec::chunk::split(&template)` (synthesize.rs:1232), which always emits chunk-form (chunked flag written per chunk, `count >= 1`; chunk.rs:240-285). Existing precedent treats it as infallible: `.expect("expected bundle is well-formed")` at :2904 and :3131-3132.

### Q6 — Scope soundness / other raw compares. CONFIRMED CLEAN, with a strong structural argument the design didn't state:
- A keyed md1 can never be non-chunked. One `Pubkeys` TLV entry is 65 bytes = 520 bits; a single codex32 string caps at 80 data symbols = 400 payload bits (`REGULAR_DATA_SYMBOLS_MAX`, codex32.rs:25, enforced at :89 encode / :174 decode). So the "non-chunked KEYED verify path" is structurally empty — excluding it is not a gap at all. Worth stating in the plan/changelog as the reason the scope is complete.
- :696 is the only supplied-md1 raw-string compare. The general path's md1 checks are decode-based (:2963-2983, :3113+); the multisig template path is id-based (:937-941).
- Residual asymmetry (declared out of scope, and fail-closed): a non-chunked KEYLESS dead card in the general path reads `mismatch` (chunk-only decode at :2509/:3113 fails) where its chunked twin reads `partial` — but both exit 4. `md1_partial::supplied_md1_unresolved_indices` (md1_partial.rs:55-63) is also chunk-only; unchanged by this cycle.

### Q7 — Verdict routing. CONFIRMED.
verify_bundle.rs:796-831: `any_fail → "mismatch"`, exit 4; else `"ok"`, exit 0. `md1_match: false` feeds `passed` on the existing `md1_template_match` check (:702-711). No new `--json` result state, no partial in this path — provided the implementation keeps the id-compare feeding `passed` and does not `?`-error on a mismatched card (it can't: id compute never fails on mismatch, only equality differs).

### Q8 — `?`-propagation on id-compute. ACCEPTABLE AS DESIGNED, fail-closed.
- Propagation lands as `ToolkitError::MdCodec` → `md_codec_exit_code` (error.rs:264, :509-516, :635) → nonzero exit, never a false pass.
- For the SUPPLIED `d`: it already passed strict decode's full validator gauntlet (decode.rs:118-154), which is a superset of `encode_payload`'s validators (encode.rs:103-111); the writers can only fail on data decode cannot produce (e.g. `PathDepthExceeded` needs >15 components but decode reads a 4-bit depth ≤ 15, origin_path.rs:43,55-61). The error arm is effectively unreachable. A hard error is the RIGHT call: mapping an internal codec-invariant break to "mismatch" would mislabel a toolkit bug as a card defect. Do not convert to a mismatch.
- For the EXPECTED side, `?` is fine; `.expect("expected bundle is well-formed")` would mirror the :2904 precedent — either is fail-closed.

---

## Critical
None.

## Important
None.

## Minor / Nit
1. Rationale overstatement (docs only): the "WDT-id would silently match a wrong-account template card" justification is moot for genuine single-sig template cards — they are origin-ELIDED and byte-identical across accounts (verify_bundle.rs:634-635 says so explicitly), so today's byte-compare, WDT-id, AND the new encoding-id all match a wrong-account genuine card equally; account correctness is carried by `--account`/`--origin` at recompose, not by the card. The encoding-id choice is still right (it stays strict against hand-crafted explicit-origin/doctored keyless cards, which WDT-id would blur) — just state the rationale accurately in the plan.
2. State the structural keyed-exclusion argument (Q6 above: 520-bit pubkey > 400-bit single-string cap) in the plan/CHANGELOG — it upgrades "keyed paths are out of scope" from a choice to a proof of completeness.
3. Dead-card verdict asymmetry (non-chunked keyless dead card → `mismatch`, chunked → `partial`; both exit 4) survives this cycle, declared out — mirror the 0.89.0 changelog's residual-carving note.
4. Exit-code surface change for previously-erroring inputs (non-chunked template card: exit 2 `--template`-required / exit-4 false-mismatch → exit 0/4 real verdict): fine for MINOR per the 0.89.0 precedent ("previously-rejected input now decodes"), but write the migration note the same way.
5. Type nit: `chunk::reassemble` takes `&[&str]`; `expected.md1` is `Vec<String>` — needs the `.iter().map(String::as_str)` conversion (mirror :2902).
6. Asymmetric tolerance post-Facet-2: `mk1_template_stub_bind` (:697-700) stays a raw byte-compare, so a case-variant mk1 transcription still mismatches while the md1 now tolerates form variance. Pre-existing, different codec, fine to leave — but a test pinning that md1-tolerance does not extend to mk1 would be prudent.

Suggested R0 test pins: non-chunked template verify → ok; non-chunked wrong-seed → mismatch/exit 4; chunked single + multi-chunk regression byte-identical; non-chunked multisig template routes to `verify_multisig_template` (and refuses without `--from`, matching chunked); dead-card fall-through parity (strict gate).

---

## Proof of work (reviewer's files + line ranges; main-loop spot-checked Q2/Minor1/Minor2 + identity.rs)

(Full table as returned — verify_bundle.rs 276-362/370-574/576-832/834-1019/1090-1218/2494-3145; decode.rs 1-196; chunk.rs 68-353; validate.rs 17-331; identity.rs 1-240; encode.rs 1-120; origin_path.rs 28-115; canonical_origin.rs 36-76; header.rs 4-46; codex32.rs 17-195; synthesize.rs 44-1240; inspect.rs 220-263; md1_partial.rs 55-63; repair.rs 161-191; error.rs 264/509-637; CHANGELOG.md 1-30; Cargo.toml:3 = 0.89.0.)
