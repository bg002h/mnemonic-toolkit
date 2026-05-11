# tech-manual v0.1 Phase 1.4 review r1 — ms1 wire format (commit cdc807b)

Reviewed: `docs/technical-manual/src/20-wire-formats/23-ms1-wire-format.md` (~300 lines, ~11pp). Cross-checked against `/scratch/code/shibboleth/mnemonic-secret/`: `design/SPEC_ms_v0_1.md`, `crates/ms-codec/src/{consts,error,decode,envelope,tag,payload}.rs`, and the SHA-pinned vector corpus emitted by HEAD `ms-cli`.

## Critical: 0

## Important: 1 (folded inline)

### I1 — Long-code total-length parenthetical was wrong: said "length ≥ 96"; correct is "99–111 for HRP=ms"

Chapter line 99 (the post-table sentence on rule 9) wrote: "This single rule rejects, in particular, every long-checksum BIP-93 string (length ≥ 96)..." But `length = 96` is itself a valid BIP-93 short-code total (data-part 93 + 3-char HRP+sep). The actual BIP-93 long-code total range for HRP=ms is **99–111 characters** (data-part 96..=108 per BIP-93 §"Long codex32" / `mk-codec` reference at `string_layer/bch.rs::bch_code_for_length`, plus 3 chars of HRP+separator). The chapter's "≥ 96" claim was therefore off by 3 at the lower bound and missed the upper bound entirely.

SPEC §4 rule 9 says "length ≥ 125" as the parenthetical, which is also imprecise (the actual long-code maximum total for HRP=ms is 111). Both documents had drifted from BIP-93's authoritative bracket boundaries.

**Fix applied:** rewrote line 99 to read: "...every BIP-93 long-code string (total 99–111 chars for HRP=ms — data-part length 96..=108 plus 3 chars of HRP+separator)..." The explicit min/max + the bracketed data-part-range note makes the boundary auditable against BIP-93 directly. No corresponding SPEC change is in scope at this commit (Phase 1.4 is a chapter fold-only; the SPEC's drift is a separate `pre-bip-submission` audit item that the ms1 BIP draft will resolve when it's filed).

## Low: 2 (folded inline)

### L1 — Citation range `consts.rs:36-39` was off by 2 lines at the start

Chapter line 138 cited `crates/ms-codec/src/consts.rs:36-39` for `RESERVED_NOT_EMITTED_V01`. Line 36 of `consts.rs` is `TAG_ENTR` (a different constant); `RESERVED_NOT_EMITTED_V01` lives at line 39 (with doc-comment at line 38).

**Fix applied:** changed the citation to `consts.rs:36,39` (comma-separated pair pointing at *both* relevant constant declarations — `TAG_ENTR` at 36 and `RESERVED_NOT_EMITTED_V01` at 39 — since the chapter table describes both the v0.1-emit tag and the reserved-not-emitted set together).

### L2 — Three `RESERVED_TAG_TABLE` rows omitted the `Error::` variant name

Chapter lines 144–146 (the `xprv`, `mnem`, `prvk` rows) had `reject` cells without spelling out `Error::ReservedTagNotEmittedInV01`. The `seed` row directly above did spell it out. SPEC §3.3 specifies the same variant for all four tags. Consistency-only fix.

**Fix applied:** appended `(`Error::ReservedTagNotEmittedInV01`)` to each of the three rows so the chapter table is internally consistent and self-citing.

## Nit: 0

## Verified correct (sample of clean items)

- All 5 length-envelope-table rows re-derived from first principles: `total = 3 + 1 + 4 + 1 + ⌈(entropy_bytes+1)×8/5⌉ + 13`. Pad-bit set `{4, 2, 0, 3, 1}` for `{16, 20, 24, 28, 32}` entropy bytes confirmed via `(entropy_bytes+1)*8 mod 5` arithmetic.
- All 5 entries of `VALID_ENTR_LENGTHS` and `VALID_STR_LENGTHS` match `consts.rs:29,33` verbatim.
- V1 worked-encode bit math (17 zero bytes → 136 bits → 28 symbols → 4 pad bits) re-derived correctly.
- V1 vector string `ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` matches `ms vectors --pretty` first vector byte-for-byte.
- 50-char total decomposition `3 + 1 + 4 + 1 + 28 + 13 = 50` correct.
- Character-by-character alphabet decomposition: `q = 0`, `s = 16`, `0 = 15` all match the canonical bech32 alphabet `qpzry9x8gf2tvdw0s3jn54khce6mua7l`. 28 × `q` = 17 zero bytes MSB-first with 4 zero pad bits confirmed.
- Wire-position re-parse offsets (`THRESHOLD_OFFSET = 1`, `ID_START_OFFSET = 2`, `ID_END_OFFSET = 6`, `SHARE_INDEX_OFFSET = 6` relative to `sep`) match `envelope.rs:36-41`. For `sep = 2` in the V1 string, derived positions `s[3] = '0'`, `s[4..8] = "entr"`, `s[8] = 's'` are correct.
- All 10 validity-rules-table Error variants exist with exact names in `error.rs` (`Codex32`, `WrongHrp`, `ThresholdNotZero`, `ShareIndexNotSecret`, `TagInvalidAlphabet`, `UnknownTag`, `ReservedTagNotEmittedInV01`, `ReservedPrefixViolation`, `UnexpectedStringLength`, `PayloadLengthMismatch`).
- `ms-codec` version "v0.1.1" claim matches HEAD `crates/ms-codec/Cargo.toml`.
- `rust-codex32 = "=0.1.0"` exact-pin claim matches `Cargo.toml` dep declaration.
- Four v0.1 → v0.2 migration invariants summarized accurately against SPEC §5 (reserved-prefix-byte discriminator, grouping invariant, encoder anti-collision, API back-compat).
- Both transcripts (`ms1-encode-12word-abandon`, `ms1-decode-12word-abandon`) round-trip cleanly against HEAD `ms-cli` v0.1.1 binary via `verify-examples.sh`.
- ms1 has no BIP draft yet (the opening paragraph's claim is accurate — `mnemonic-secret/bip/` directory does not exist).

## Post-fix verification

- `make lint` 6/6 green.
- `make verify-examples` 6/6 transcripts pass.
- `SOURCE_DATE_EPOCH=1746921600 make pdf` builds (83pp, unchanged page count — the fixes were small textual corrections).

## Disposition

I1 was a precision error in the long-code-bracket parenthetical that the Phase 1.3 review's `data-part-vs-total-string` discipline would have caught at draft time; the chapter's explicit `3 + N + ... = M (total)` decomposition was applied to the *worked walks* but the rule-9 commentary used a one-off legacy phrasing from the SPEC. Folded inline.

L1 + L2 were citation precision + consistency cleanups. Folded inline. Phase 1.4 close: 0C/0I/0L/0N.

**Operational note:** Phase 1.4 progressed the wire-format-chapter reviewer-round trend from {2, 5, 1, 1} Importants across {1.1, 1.2, 1.3, 1.4}. The `3 + N + ... = M (total)` decomposition discipline kept the *worked walks* error-free; the residual Important migrated to a non-worked-walk parenthetical. For Phase 1.5 (back-matter skeleton) the bit-math discipline is no longer load-bearing, but the source-citation precision lesson (line ranges, error-variant naming) carries forward.
