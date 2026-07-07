# P1 per-phase R0 review — mk1-repair-set-level-reverify — round 1

**Verdict: GREEN (0 Critical / 0 Important)** — 3 non-blocking informational Minors.
**Reviewer:** adversarial opus architect (read-only). `mnemonic-key` worktree @ `71c2b31` (base `85bca69`).
**Dispatched:** 2026-07-07 (Cycle E, per-phase P1 R0, FULL mk-cli suite). Persisted verbatim per CLAUDE.md (cross-repo audit trail kept in the toolkit).

## Independent counts
`cargo test -p mk-cli` → **105 passed, 0 failed** (14 bins; new `cli_mk1_repair_reverify.rs` 10 cells green). `cargo clippy -p mk-cli --all-targets -- -D warnings` → clean (exit 0).

## Funds-attack results — every route SAFE
1. **Full-set miscorrection `CHUNK0 CORRUPTED_MK1_CHUNK1` → exit 2 (not 5).** The corrected chunk1 aliases to a valid-but-≠ codeword with the same csid → group complete-and-consistent → `mk_codec::decode(&[CHUNK0,corrected1])` = `Err(CrossChunkHashMismatch)` → `GroupVerdict::Reject` → `CliError::SetReassemblyMismatch` → exit 2. Precondition test asserts `decode(corrupted).is_err()`.
2. **Bless only when `decode(exact corrected group)==Ok`.** `classify_mk1_set` (repair.rs:360-368) `Ok`→Bless / `Err`→Reject; confident exit-5-no-advisory only through the `Ok` arm. Mirrors toolkit `verify_mk1_set`.
3. **Message names the csid.** `describe_group_key(Chunked(csid))="chunk_set_id 0x{csid:05x}"` → `SetReassemblyMismatch` msg "…the set does not reassemble ({group}): {detail}"; text→stderr, JSON→stdout envelope. Reject tests assert `"does not reassemble"`+`"chunk_set_id"` substrings (not just exit).
4. **Pre-emit classification prevents emitting the wrong-fit chunk.** `classify_mk1_set(...)?` (repair.rs:139) runs BEFORE `emit_json`/`emit_text` (151-155); a dominant Reject `?`-propagates → main prints only the error; corrected chunk never written. Test asserts `stdout not contains CORRUPTED_MK1_CHUNK1` (text + JSON).

## Per-plate preservation (C1 guard) — SAFE
Single flipped chunk alone → `group_is_complete_and_consistent`=false (Chunked total=2, 1 member) → Candidate → `Unverified` → exit 5 + stderr advisory (`"UNVERIFIED"`+`"BIP-93"`), NOT exit 2. Text + `--json` cells pass (corrected chunk stays in the JSON envelope; advisory on stderr — no wire-shape change).

## Header-parse efficiency deviation — FUNCTIONALLY IDENTICAL, never diverges (traced mk-codec source)
The trap premise ("`decoded` is the PRE-correction header") is FALSE. `decode_string` applies BCH correction internally (bch.rs:658): `data_with_checksum = result.data` where `bch_correct_*` return the POST-correction codeword (`corrected[p]^=m`, re-verified residue 0, bch.rs:431-447). `DecodedString::data()` (bch.rs:604) = post-correction header+payload. So a header-region correction (csid/total/index bits) is ALREADY reflected in `decoded.data()`. The toolkit re-decodes the corrected string, but `corrected_chunk` re-encodes the same residue-0 codeword → `decode_string(corrected).corrections_applied==0` + byte-identical `data()`. Thus `from_5bit_symbols(decode_string(original).data())` (mk-cli) ≡ `from_5bit_symbols(decode_string(corrected).data())` (toolkit) in EVERY case incl. header-region correction. Pure optimization (one fewer BCH pass), zero behavioral difference — NO csid-misgrouping funds hole.

## Multi-group parity + no collateral — SAFE
`classify_mk1_set` mirrors `verify_mk1_set` (group by csid; complete-and-consistent = indices 0..total each once + consistent total, discriminating on parsed indices not the error string; fold reject>candidate>bless; decode on corrected group). Batch {miscorrection, clean} → clean group skipped (untouched), miscorrection folds Reject → dominant → exit 2, ALL output suppressed. Clean full/partial → `any_correction=false` → classifier not engaged → exit 0. Genuine ≤4 → Bless → exit 5 no advisory. Uncorrectable → `decode_string` Err → `CliError::Codec` → exit 2 (unchanged). New variant wired across kind/message/exit_code=2/details. Diff touches only `mk-cli/{cmd/repair.rs,error.rs,tests/...}` — NO mk-codec change (NO-BUMP); `mk decode` untouched. `expect`/`unwrap` sites (repair.rs:354/379/395) invariant-guarded.

## Minors (non-blocking, informational — NOT findings)
1. A hypothetical miscorrection corrupting the CSID header bits would de-group the corrected chunk into an incomplete singleton → Candidate → exit 5 + loud UNVERIFIED advisory (directs to `mk decode`, where `ChunkSetIdMismatch` catches it), rather than exit 2. Exactly SPEC §2 incomplete-group semantics; behaviorally identical to the R0-GREEN toolkit `verify_mk1_set`; not a confident BLESS, not a P1 regression.
2. `SetReassemblyMismatch` appended at enum end (mk-cli `CliError` has no alphabetical convention — that rule is toolkit-`ToolkitError`-scoped; existing enum unsorted → matches local style).
3. Single-plate cell uses chunk1 (regular code) vs the manual's chunk0 (long code) — identical classification path (incomplete group), faithful coverage.

**No Critical or Important. Phase P1 GREEN — clear to advance to P2.**
