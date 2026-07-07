# P0 per-phase R0 review — mk1-repair-set-level-reverify — round 1

**Verdict: GREEN (0 Critical / 0 Important)** — 2 Minor (non-gating).
**Reviewer:** adversarial opus architect (read-only). Worktree HEAD `67723ae1` (base `011eeb5d`).
**Dispatched:** 2026-07-07 (Cycle E, per-phase P0 R0, FULL suite). Persisted verbatim per CLAUDE.md.

The funds fix is correct, the tri-state invariant matches the SPEC, no auto-repair path can bless a miscorrected set, and "existing tests flipped: NONE" is verified true. Cleared to advance to P1.

## Independent counts (run in worktree)
`cargo test -p mnemonic-toolkit` → **3651 passed, 0 failed, 18 ignored, 204 bins** (exit 0). New file 16 passed + 2 ignored. `cargo clippy -p … -- -D warnings` → clean. Both ignored harness cells run: `rediscover_funds_anchor_seed_matches_pinned_constant` → ok (pinned `CORRUPTED_MK1_CHUNK1` re-derives byte-for-byte from `find_mk1_miscorrection_seed(CHUNK0,CHUNK1,0x4634_5F31,1e7)` — alias still exists under current mk-codec BCH); `cell_4_8_rate_harness_slow_high_power_bound` (N=1e6, ~356s) → ok (`hits>=1`, Clopper-Pearson bound printed; non-vacuous).

## Funds-attack results — every route SAFE
1. **Auto-repair short-circuit — SAFE.** `RepairShortCircuit{exit:5}`+`emit_repair_report` produced in EXACTLY ONE place (`repair.rs:1705-1706`) inside `try_repair_and_short_circuit`, now gated on `matches!(outcome.set_verify, SetVerify::Blessed)` (`:1701`). Reject→`Err`→`Err(_)=>Ok(())` fall-through (`:1684`); Candidate→`!Blessed` fall-through (`:1701`). Confirmed ALL 11 auto-repair call sites (inspect/convert/xpub_search seed_intake/verify_bundle ×6) route through this one wrapper — no bypass (grep of non-test `repair_card(` callers = wrapper + `cmd/repair.rs` only).
2. **`mnemonic repair` — SAFE.** Bless→5; Candidate→emit+4+advisory; Reject=`SetReassemblyMismatch` which `is_indel_trigger` EXCLUDES (`:1452`, exhaustive) so even `--max-indel>=1` → `Err(e)=>Err(e.into())` → `ToolkitError::Repair→2`, NO output emitted.
3. **"Bless iff decode Ok" holds.** `verify_mk1_set` blesses a touched group only when `complete_and_consistent && mk_codec::decode(&refs).is_ok()`. decode-Ok implies completeness (`reassemble_from_chunks` requires every index present). `decode` is order-independent (sorts internally) → first-seen ref order can't false-reject a genuine set (G1). Untouched groups skipped safely (`touched` derived from `repairs`; every `repair_chunk_one` correction pushes a `RepairDetail`).
4. **Pinned `CORRUPTED_SET` genuine miscorrection — SAFE.** Search filter (`header_still_chunked && decode(...).is_err()`) selects a corrupted chunk1 BCH-correcting to a valid-but-≠ codeword sharing chunk0's csid/total=2/index=1 → group as one complete set → fail cross-chunk SHA → decode Err → `"chunk_set_id 0x…"`. `cell_4_1b/c` assert the MESSAGE (`"does not reassemble"`+`"chunk_set_id"`), not just exit. Empirically the decode-path Reject (not header-parse path — that would emit `"chunk N (post-correction header)"` + fail the csid assertion; passing confirms decode-path).
5. **Batch reject-dominant — SAFE.** `fold_verdict` = reject>candidate>bless; dominant Reject `?`-propagates, discarding all corrected chunks. `cell_4_5b` + order-reversed variant confirm exit 2 + co-batched clean group NOT emitted.

## "existing tests flipped: NONE" — VERIFIED TRUE
Audited every mk1 cell in `cli_repair.rs`/`cli_auto_repair.rs`/`cli_indel.rs`/`cli_positional_hrp_autodetect.rs`/`cli_output_class.rs`. Full-set mk1 cases use ≤4-error → decode Ok → Bless→5 (unchanged); indel case errors early, never reaches `verify_mk1_set`. Only mk1 auto-repair case (`cell_20a`) exits 0 via mk-codec internal per-chunk correction — never reaches short-circuit. Single-chunk mk1 cases supply CLEAN chunks → untouched → exit 0. The PM-1-predicted flips (5→2, 5→4) are real in principle but NO pre-existing test asserted the old behavior — those scenarios were never covered. Claim accurate.

## 4 deviations — all SOUND
1. Header-parse-fail→Reject: SPEC §2 rule 2 lists header-region errors as Reject; fail-safe (2 stricter than 4); never masks a Bless. 2. `--json` via CLI: `RepairError` surfaces through the identical typed-error stderr path regardless of `--json` (which only reshapes the Ok emission) — can't convert Reject to false success; equivalent-or-stronger. 3. `rand` dev-dep: under `[dev-dependencies]` (Cargo.toml:83); Cargo.lock adds only a direct ref to already-resolved `rand 0.8.6` — no new `[[package]]`, zero prod/vendor impact (G6). 4. `candidate_seen`→`indel_exit_code`=4 = SPEC §3 VERIFY-ME; engine stays kind-agnostic (PM-r2-3).

## Minors (non-gating)
- **M1** — `cell_4_6` md1 lock injects 5 spread errors → asserts exit 2 but doesn't distinguish BCH `TooManyErrors` from the content-id catch (a pinned md1 alias would be stronger). Acceptable: md1 out-of-scope-for-structural-change (SPEC §0.4); `cell_4_6b` confirms `reassemble` is the sole decode path; the mk1 pinned anchor is the real proof. Optional future strengthening.
- **M2** — a mixed batch folding to CANDIDATE (not Reject) emits all corrected chunks under a single GLOBAL "UNVERIFIED" advisory + exit 4 without naming which group (unlike the Reject message which names the group per PM-r2-2). Within SPEC §2 rule 3 (exit 4 already = verify-everything, no bless occurs). Non-gating informational.

## Other
Diff = 5 files, no collateral. ms1/md1 arms unchanged except `set_verify:Blessed` (codecs return Ok only on full decode success). All 3 `RepairOutcome` constructions set the field; both `RepairError` exhaustive matches (Display, is_indel_trigger) updated (compile-checked). No clap surface change in P0 → no schema_mirror/manual owed this phase (P1/P2). **Cleared to advance to P1.**
