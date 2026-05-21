# v0.32.0 end-of-cycle architect review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** end-of-cycle
**Cycle:** Cycle 14 (seedqr-compact-variant)
**Date:** 2026-05-21
**Pre-tag SHA:** `8404910` (Phase 2-4; Phase 5 uncommitted)

## Verdict

**GREEN.** All 10 verification items pass. 0 Critical / 0 Important / 1 non-blocking note.

## Verifications

1. **encode_compact / decode_compact** (`seedqr.rs:168-204`): encode = parse→checksum→`to_entropy()`→`hex::encode`, refuses word-count ∉{12,24} before parse. decode strips whitespace→`hex::decode`→byte-count{16,32} check at L196 which PRECEDES `from_entropy_in` at L200. Order verified — `CompactByteCountUnsupported` fires before the generic BIP-39 path swallows 20/24/28.
2. **--variant flag**: derived `SeedqrVariant` ValueEnum (R0 M1) on BOTH args structs; dispatched in run_decode/run_encode; default standard.
3. **Envelope variant field**: both emit sites use `args.variant.as_str()`; zero remaining hardcoded "standard".
4. **Standard path unchanged**: dispatch adds only a Compact match arm; byte-identical.
5. **Tests**: 10 lib + 8 CLI = 18 cells. `decode_compact_rejects_20_byte_count` + `standard_decode_of_64_char_hex_clean_error` both present.
6. **3 SeedqrError variants**: meaningful Display arms.
7. **Manual mirror**: `--variant` on both subcommands + Scope flip + xxd→qrencode example; flag-coverage lint passes.
8. **CHANGELOG / install.sh / Cargo.toml**: all 0.32.0.
9. **SemVer MINOR**: correct (new flag NAME, additive).
10. **GUI lockstep**: CHANGELOG flags mandatory v0.17.0; both subcommands need the dropdown.

## Non-blocking note

`seedqr-compact-variant` FOLLOWUP still `open` + `gui-seedqr-variant-flag-mirror` not yet filed — correct sequencing (post-tag Phase 5/6 steps). The stale `--word-count` requirement in the FOLLOWUP body was correctly superseded by the plan (byte-count disambiguates); fold the supersession note into the closure entry.

## Cleared for tag.
