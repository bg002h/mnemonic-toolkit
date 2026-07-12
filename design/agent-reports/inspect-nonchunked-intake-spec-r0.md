# SPEC R0 — inspect non-chunked md1 intake (`toolkit-inspect-nonchunked-md1-intake-gap`) — Opus, adversarial

**Persisted per CLAUDE.md.** VERDICT: **GREEN (0 Critical / 0 Important).** SPEC sound to advance to an implementation plan. 4 Minors (fold into plan/SPEC). Reviewer built the current binary + fed real non-chunked cards; no prompt-injection.

## Funds-safety core — 5 invariants VERIFIED against current source
- **INV-1 (structural, never fallback):** `decode_md1_string_with_opts` (`vendor/md-codec/src/decode.rs:187-196`) dispatches on `chunked_flag = (byte0 >> 3) & 1` → `flag==1` `reassemble_with_opts(&[s],opts)`, `flag==0` `decode_payload_with_opts`. No try/catch. Exact match to SPEC pseudocode.
- **INV-2 (BCH before flag):** `unwrap_string` runs the codex32 BCH check (`codex32.rs:182-186`) BEFORE the flag read (`decode.rs:191`) — a first-symbol corruption fails the checksum → reject; corruption cannot re-route.
- **INV-3 (content-id oracle unconditional for chunked):** `reassemble_with_opts` rejects `ChunkSetIdMismatch` regardless of `opts` (`chunk.rs:404-415`); every chunked input (chunked-of-1 + multi) still routes here. A single non-chunked payload legitimately has no cross-chunk oracle (one atomic unit; integrity = codex32 BCH + `decode_payload` validators). Inapplicability, not a dropped check.
- **INV-4 (partial threads; EmptyOriginOverride fatal-in-partial):** `validate_no_empty_origin_overrides` fires unconditionally (`decode.rs:143`); only `MissingExplicitOrigin` is swallowed under `allow_unresolved_origin` (`decode.rs:146`).
- **INV-5 (no new acceptance):** only `chunked_flag==0` inputs reach the new path → `decode_payload_with_opts` full gauntlet (root-tag allow-list `{Sh,Wsh,Wpkh,Pkh,Tr}` `decode.rs:98-106`; placeholder/multipath/taptree/xpub validators). Only a fully-valid non-chunked md1 is newly accepted.

**Adversarial dispatch-boundary check:** an incomplete chunk set of 1 carries `chunked_flag==1` → routes back through `reassemble_with_opts(&[s])` → `ChunkSetIncomplete` (`chunk.rs:378-383`), byte-identical reject to today. The new `decode_payload` path is reachable ONLY by genuine `chunked_flag==0` singles. Zero mis-routing risk; change touches only `len==1`.

**Empirical baseline:** built `mnemonic`; non-chunked valid single `md1yqpqqxqq8xtwhw4xwn4qh` + non-chunked dead card `md1yppqqxp3cg2x3r70ckk4kjaf` both fail today exit 3 `unsupported version 2` (the RED-proof premise).

## Findings
Critical: none. Important: none.
**Minor:**
- **M1** — add a regression lock: a BCH-clean structurally-invalid non-chunked single (e.g. bad root tag) maps to `MdCodec` (not `FutureFormat`), so inspect auto-fire (`inspect.rs:130-145`) DOES fire `try_repair_and_short_circuit`, but a BCH-clean string makes 0 edits → falls through to terminal `Err(orig)` (same as structurally-invalid chunked cards today). Add a test: such a single → NOT exit 5, terminal decode error. Whole-diff review must confirm no spurious exit-5.
- **M2 / Q5** — use the frozen `DEAD_SINGLES` KAT (`tests/cli_inspect_partial.rs:29-38`, `md encode --group-size 0`, non-chunked) directly for the dead-card fixture; its `dead_chunks()` helper (`:76-80`) already calls `decode_md1_string_with_opts(single, partial())`. Frozen KAT > `encode_md1_string` synthesis.
- **M3** — citation line drift (cosmetic): `codex32.rs` BCH at 182-186 (not 183-184); `error.rs` WireVersionMismatch→FutureFormat at 1055-1058; exit 3 at `:588`. Refresh SHA `a528eba5`→`d9dbbe92` (HEAD touches only 2 design docs — zero source drift). Re-cite at plan time.
- **M4** — truth-table note: genuine future-version singles (v8/v12, LSB 0 → `chunked_flag==0` → `WireVersionMismatch{got:8|12}` `header.rs:42-43`) KEEP exit 3.

## Answers to open questions
1. **Scope inspect-only: CONFIRM.** `verify_bundle.rs:696` `expected.md1 == args.md1` compares raw strings vs chunk-form synthesized (`:688-695`); a non-chunked supplied string can never string-equal chunk-form expected → broadening intake alone can't help there (needs comparison-canonicalization, own funds review). inspect is describe-only (no funds movement). Coherent, safe slice; keep the verify-bundle residual on the FOLLOWUP.
2. **Dispatch Option A (`chunks.len()==1` branch): CONFIRM.** Byte-identical for chunked-of-1 + multi; reuses the blessed codec discriminator; no codec-helper change needed. Option C (try/catch) correctly rejected.
3. **Diagnostic exit-code change: ACCEPTABLE (correction, not regression).** Today's exit-3 `FutureFormat` for a structurally-invalid same-version single is MISLEADING; moving it to `decode_payload`'s specific error (likely exit 2) is strictly better. Genuine future-version singles keep exit 3 (verified). MINOR-safe (only reshapes an already-rejected path).
4. **Manual: no lockstep required.** No clap flag/subcommand/dropdown change → no `schema_mirror`, no `lint.sh`. `--json` unchanged (`InspectJson::Md1`, `schema_version` "2"). Optional inspect-chapter one-liner is a docs-only same-PR touch (recommended for discoverability).
5. **Dead-card fixture: sound; prefer the frozen KAT (see M2).**

## VERDICT: GREEN (0C/0I) — advance to implementation plan; fold the 4 Minors into the plan-doc; re-cite drifted lines + refresh SHA.
