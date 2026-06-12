# R0 Review — Cycle C fuzzing (round 3)

Reviewer: Fable 5 architect agent (a48d8c441e9b415d7), 2026-06-11.
Target: design/BRAINSTORM_stress_cycle_c_fuzzing.md (R2 fold) @ toolkit e33c147.
Persisted verbatim per CLAUDE.md convention.

## Verdict: GREEN

The single round-2 Important [I3-residual] and all three Minors are correctly folded. The clamp is empirically panic-free under cargo-fuzz's exact build profile (debug-assertions + overflow-checks), genuinely non-vacuous (drives partial-trailing-bit paths P3's `len*8` pin never isolates), and introduces NO oracle false-positive. The two coordinate folds (md/ms `position` = post-HRP data-part offset) are source-verified correct. 0C/0I — the hard gate is met. One non-blocking factual nit (variant count) recorded as Minor; it does not affect the design.

## Critical
- none

## Important
- none

## Minor
- **[count nit, non-load-bearing] The C1 minimality sentence says "18 `ms_codec::Error` variants"; the enum has exactly 16.** `ms_codec::Error` (mnemonic-secret `crates/ms-codec/src/error.rs:9-113`) has 16 named variants (Codex32, MnemUnknownLanguage, WrongHrp, ThresholdNotZero, ShareIndexNotSecret, TagInvalidAlphabet, UnknownTag, ReservedTagNotEmittedInV01, ReservedPrefixViolation, UnexpectedStringLength, PayloadLengthMismatch, TooManyErrors, InvalidShareCount, InvalidThreshold, IsShareNotSingleString, SecretShareSuppliedToCombine). The round-2 review carried the same "18". Cosmetic miscount; the **minimality conclusion is unaffected** — only `Codex32(_)` (wraps codex32's 3 String variants) and `WrongHrp{got: String}` carry an unbounded ≥8-char String; the other 14 cap at `[u8;4]` (4 chars) or smaller. Correct "18"→"16" at implementation time; not a gate.

## Checks

**1. [I3-residual] clamp — RESOLVED, panic-free AND non-vacuous (empirically proven).**
- Fold text mandates `total_bits = candidate.min(remainder.len()*8)`; drops the false ">len*8 validation" rationale ("There is no `>len*8` 'validation' path to exercise via the raw entry point"); explains the cargo-fuzz reason ("cargo-fuzz builds release-WITH-debug-assertions by default → an unclamped prefix ABORTS vacuously on ~the first exec", citing `bitstream.rs:114`).
- **Panic-free under the real profile:** scratch crate path-dep'ing `md-codec`, ran `decode_payload` with `RUSTFLAGS="-Cdebug-assertions=on -Coverflow-checks=on"` on nightly (`cfg!(debug_assertions)==true` confirmed at runtime). Swept clamped `k` across `[0, len*8]` for 5 remainders (incl. round-2 reproducer `[0x01,0x02,0x03]`, all-0xff, 40-byte ramp, empty) AND a real canonical wpkh descriptor (canon_bits=58, len*8=64). Every `k` returned cleanly: `k=0,1` → `Err(BitStreamTruncated)`; mid-range short reads → `Err(BitStreamTruncated)`; `k≥canon` → `Ok`. No panic, no abort. `with_bit_limit` assert never fires (`k ≤ len*8` by clamp construction).
- **Non-vacuous:** the real descriptor's canonical bit length (58) is itself below `len*8` (64); P3 pins only `len*8=64`. The clamped target sweeps `k=58..64` (canonical boundary through the ≤7-bit padding-tolerance band, tlv.rs:217-298) PLUS every `k<58` — strictly more BitReader/TLV partial-trailing-bit logic than P3 reaches.
- **No other reachable debug_assert / overflow with a clamped total_bits:** swept all `debug_assert!` in the closure. `bitstream.rs:127` (count≤64) — decoder-supplied widths, never input-controlled past 64. `tree.rs:129` is encoder-side (re-encode path), holds by construction. `remaining_bits` uses `saturating_sub`. `bytes.len()*8` can't overflow usize. overflow-checks-ON run confirmed no overflow.

**2. The three Minors — all folded and source-verified.**
- (a) md apply-details coordinate: "post-HRP-and-separator offset into the data-part of chunk `chunk_index`" ✓. Source `chunk.rs:404-415` + `:550-552` `position:pos` indexes `parse_chunk_symbols` (post-`md1`, `:436`).
- (b) ms apply-details coordinate: "single data-part, position-only, NO chunk_index; post-HRP past `ms1`" ✓. Source `decode.rs:119-128` (no chunk_index) + `:256-257` `position:pos` indexes `parse_ms1_symbols` (post-`ms1`, `:154`).
- (c) gen-corpus splitter validity gate: split-then-call (`split \n → reassemble(&parts) Ok`) + between-only joins (no trailing `\n`) ✓.
- (d) C1 exclusion minimality: present, conclusion correct; only the "18"→16 count nit (Minor above).

**3. Clamp does NOT false-positive the fixed-point oracle.**
- Oracle is a VALUE round-trip: `decode→D; encode_payload(&D)→(bytes',tb'); decode(bytes',tb')→D'; assert D==D'` (`Descriptor: PartialEq`, encode.rs:16). Compares Descriptor values, never input-bytes-vs-reencode or k-vs-tb'.
- `encode_payload` computes its OWN canonical `total_bits = w.bit_len()` (encode.rs:90) after canonicalize (`:67`), independent of clamped `k`; re-decode uses that tb' → self-consistent by construction.
- **Feared scenario occurs and stays safe:** clamped `k=59..63` (>canon 58, within padding tolerance) decoded successfully, re-encoded to `tb'=58 ≠ k`, yet `D==D'` held every time. No byte/total_bits comparison exists for the clamp to perturb. No false-positive. (Re-encode-Err-on-accepted-value would be a real finding per [I6]; never fired.)

## Evidence log
```
HEADs: descriptor-mnemonic cdd8501 (clean); mnemonic-secret 1b53e53 (disk); toolkit e33c147
md decode_payload pub lib.rs:45; encode_payload lib.rs:46
md with_bit_limit bitstream.rs:113-114 debug_assert!(bit_limit<=bytes.len()*8); short-read→Err(BitStreamTruncated) :128-133; remaining_bits saturating_sub :165
md TLV padding tolerance tlv.rs:217-298; encode_payload total_bits=w.bit_len() encode.rs:90 (own bit count, not input k); Descriptor PartialEq encode.rs:16
md CorrectionDetail chunk.rs:404-415 {chunk_index,position,was,now}, position:pos :552 post-md1 (:436)
md write_node debug_assert tree.rs:129 encoder-side, holds on decode-produced Tr (is_nums⇒key_index=0)
ms Error error.rs:9-113 = 16 variants (grep -oE 'Error::[A-Z]\w*'|sort -u); only Codex32(_)+WrongHrp{got} ≥8-char; other 14 ≤[u8;4]/char/unit (spec says 18 → Minor)
codex32-0.1.0: 3 String variants all via ms Codex32(_) ⇒ excluded; InvalidChar/InvalidCase single-char (<8)
ms CorrectionDetail decode.rs:119-128 {position,was,now} no chunk_index, position:pos :257 post-ms1 (:154)
SCRATCH (removed): /tmp/r3scratch path-dep md-codec, RUSTFLAGS="-Cdebug-assertions=on -Coverflow-checks=on", +nightly
  cfg!(debug_assertions)==true; synthetic remainders {[1,2,3],[ff;8],[00;16],0..40,[]}: clamped k∈[0,len*8] ALL clean Err, NO panic
  real wpkh: canon=58<len*8=64; k<58→Err(BitStreamTruncated); k∈{58..64}→Ok, re-encode tb'=58, D==D' EVERY case (oracle SAFE)
  no panic across sweep under debug-assertions+overflow-checks
```

GREEN — 0C/0I. The one Minor (16-vs-18 count) is cosmetic spec-prose; fix in the implementation fold. Cleared past the R0 gate.
