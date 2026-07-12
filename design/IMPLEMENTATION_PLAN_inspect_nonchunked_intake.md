# IMPLEMENTATION PLAN — `mnemonic inspect` non-chunked md1 intake

**Followup:** `toolkit-inspect-nonchunked-md1-intake-gap`. **SPEC:** `design/SPEC_inspect_nonchunked_intake.md` (R0-GREEN, `design/agent-reports/inspect-nonchunked-intake-spec-r0.md`, 0C/0I). **Source SHA:** `d9dbbe92` (master, `mnemonic-toolkit-v0.88.0` + burndown-closes). **Status:** plan-doc — awaiting opus R0 before impl.

## Goal
`mnemonic inspect --md1 <single-non-chunked-md1>` currently fails `unsupported version 2` / exit 3 (the chunk-layer `reassemble` misreads a non-chunked string's 4-bit version as `2`). Make it accept a plain non-chunked single-string md1 (the bare `md encode` form), consistent with the chunked path — a valid single renders its template (exit 0); a non-chunked DEAD single partial-decodes (template + `origin: «unspecified»` marker + exit 4), matching the chunked partial path. **inspect-only** (verify-bundle carved out — see Non-goals).

## Phase 1 (single) — length-dispatch the Md1 intake arm
**File:** `crates/mnemonic-toolkit/src/cmd/inspect.rs`, `decode_card` `CardKind::Md1` arm (live at `:242-245`). Replace the unconditional `reassemble_with_opts(chunks, partial())` with a length-dispatch (SPEC Option A, R0-confirmed):
```rust
CardKind::Md1 => Ok(InspectPayload::Md1(match chunks {
    [single] => md_codec::decode_md1_string_with_opts(single, md_codec::DecodeOpts::partial())?,
    _ => md_codec::reassemble_with_opts(chunks, md_codec::DecodeOpts::partial())?,
})),
```
- `[single]` → `decode_md1_string_with_opts` (the codec's in-band chunked-flag discriminator, `decode.rs:187-196`, routes a chunked-of-1 string internally BACK to `reassemble_with_opts(&[s], opts)` → byte-identical to today; a non-chunked single → `decode_payload_with_opts`, full validation gauntlet).
- `len > 1` → `reassemble_with_opts` verbatim (unchanged).
- `partial()` threads so a non-chunked dead card exits 4 consistently; `EmptyOriginOverride` stays fatal-in-partial (`decode.rs:143`, unconditional). Content-id oracle stays unconditional for chunked (`chunk.rs:404-415`); a single non-chunked payload has no cross-chunk oracle by construction (integrity = codex32 BCH + `decode_payload`).

No other source change. No clap/JSON/schema change (`InspectJson::Md1`, `schema_version` "2" unchanged).

## Tests (TDD — RED first). `tests/cli_inspect_partial.rs` (+ a new/extended cell file)
Reuse the FROZEN non-chunked KAT — do NOT synthesize (R0 M2/Q5): `tests/cli_inspect_partial.rs:29-38` `DEAD_SINGLES` (`md encode --group-size 0`, non-chunked) + a valid non-chunked single (mint via `md encode 'wpkh(@0/<0;1>/*)' --group-size 0`, or the R0's `md1yqpqqxqq8xtwhw4xwn4qh`).
1. **RED-1 (accept valid single):** `inspect --md1 <valid non-chunked single>` → exit 0, `template:` rendered (fails today: exit 3 `unsupported version 2`).
2. **RED-2 (dead single partial):** `inspect --md1 <DEAD_SINGLES[i]>` → exit 4, `template:` + `origin: «unspecified — supply on restore»` marker + VERIFY-ME stderr note (fails today: exit 3). Cross-check: same template as `md decode` on the same string (cross-binary parity, `MD_BIN`).
3. **RED-3 (M1 auto-fire lock):** a BCH-clean structurally-invalid non-chunked single (e.g. a valid-checksum string whose decoded root tag is out of the allow-list) → NOT exit 5 (no spurious repair short-circuit); terminal `MdCodec` decode error. (BCH-clean → 0 edits → auto-fire falls through, same as structurally-invalid chunked cards today.)
4. **BOUNDARY (no regression):** a chunked-of-1 card (existing rechunk fixture, `cli_inspect.rs:211-239` pattern) AND a multi-chunk card inspect byte-identically to today (exit + full text). Any drift fails.
5. **M4 (future-version single):** a v8/v12 single (`chunked_flag==0`) still → exit 3 (`WireVersionMismatch`→`FutureFormat`), NOT the new decode path.
6. **JSON unchanged:** valid single `--json` → `InspectJson::Md1`, `schema_version` "2", byte-shape identical to the chunked equivalent.

## Non-goals (carved out, stay on the FOLLOWUP)
- **verify-bundle non-chunked intake** — `verify_bundle.rs:696` compares RAW strings (`expected.md1 == args.md1`) against chunk-form synthesized cards; broadening intake alone can't make a non-chunked template verify (needs comparison-canonicalization, its own funds review). R0-confirmed. The FOLLOWUP stays OPEN for this residual (demote to a verify-bundle-scoped note).
- No codec-level change (the codec already owns the in-band dispatch).

## Release ritual (MINOR → v0.89.0)
Cargo.toml 0.88.0→0.89.0; BOTH READMEs `toolkit-version`; root + `fuzz/Cargo.lock` (via `cargo check`); `scripts/install.sh` SELF-pin ONLY (md/ms/mk FROZEN); `.examples-build` gen.sh pins + **regen Examples.md** (confirm only version drift — inspect examples don't use non-chunked singles); CHANGELOG v0.89.0 entry; FOLLOWUP `toolkit-inspect-nonchunked-md1-intake-gap` → **RESOLVED (inspect leg)** with the verify-bundle residual carved. Optional inspect-chapter manual one-liner ("a single non-chunked md1 string is now accepted") — docs-only, same-PR, not gated (R0 Q4).

## Gates (per phase + pre-tag)
`cargo test -p mnemonic-toolkit` (FULL); `cargo clippy -p mnemonic-toolkit --all-targets`; **`cargo +1.95.0 fmt --all -- --check` (mlock-exempt) BEFORE tag** (the v0.87.0 lesson); `MD_BIN=…/md cargo test … --include-ignored` for the cross-binary parity cell; vendor-freshness unaffected (no dep change); **post-impl whole-diff R0** before tag. Citations re-grepped at write time (R0 M3: `codex32.rs:182-186`, `error.rs:1055-1058`, exit-3 `error.rs:588`).

## Plan-R0 folds (GREEN 0C/0I — `design/agent-reports/inspect-nonchunked-intake-plan-r0.md`)
Snippet compile-verified by the reviewer (scratch transplant). Fold into impl:
- **M-a** correct cites: exit-3 `error.rs:587`; `WireVersionMismatch→FutureFormat` `error.rs:1054-1057`; codex32 BCH `codex32.rs:182-186`.
- **M-b** rewrite the now-false comments: `inspect.rs:239-241` (the "CHUNK-FORM only / unsupported version 2 gap" note) + the test-module doc `tests/cli_inspect_partial.rs:15-19`.
- **M-c** implement the FULL SPEC §6 9-test plan (not the condensed 6): add positional-form intake, the doctored-chunk-set-id → `ChunkSetIdMismatch` INV-3 lock, and the bad-BCH → codex32-reject INV-2 lock (guarded by construction, retained explicitly).
- **M-d** cross-binary parity cell uses `md inspect` (v0.75.0 parity gate), not `md decode`.
