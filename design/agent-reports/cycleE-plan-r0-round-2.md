# PLAN R0 review — mk1-repair-set-level-reverify — round 2

**Verdict: GREEN (0 Critical / 0 Important)** — 4 Minor polish items (non-blocking).
**Reviewer:** adversarial opus architect (read-only, cross-repo). Verified @ toolkit `db2c8b0d`, mk `main@85bca69`.
**Dispatched:** 2026-07-07 (Cycle E, plan-R0 loop round 2 — convergence on rev-2). Persisted verbatim per CLAUDE.md.

The two Important gaps are closed correctly, and I adversarially traced the new return-shape + 3-caller map through the actual call sites — it holds with no funds hole. Remaining items are cosmetic or fail-safe.

## Convergence verification
**PI-1 — tri-state propagation — ✅ CLOSED.** `RepairOutcome.set_verify: SetVerify{Blessed, Unverified}` (repair.rs:437-441 confirmed); ms1/md1 default Blessed (no behavior change). **Reject = `Err(RepairError)`**, not an `Ok`. Three distinguishable outcomes: Bless=`Ok{Blessed}`, Candidate=`Ok{Unverified}`, Reject=`Err`. Traced `try_repair_and_short_circuit` (repair.rs:1340): `Ok` arm branches on `set_verify` — Blessed→emit+`RepairShortCircuit{5}`; Unverified→`Ok(())` no short-circuit; `Err` arm already `Ok(())`. So **both Candidate and Reject fall through to the original error** in auto-repair. Matches SPEC §2. No path blesses an unverified set. `repair_card` owns csid-grouping+dominant-fold (correct layer — `resolve_groups` groups by kind → batch reaches `repair_card(Mk1,[all])` as one call). G7 added.

**PI-2 — deterministic pinned seed — ✅ CLOSED.** §4.1 pins the fully-resolved corrupted chunk strings (`const CORRUPTED_SET`), no re-encode; correct rationale (encode() randomizes `chunk_set_id` at pipeline.rs:45-47, inside the BCH codeword). Literal strings → `bch_correct`+`decode` deterministic → reproduces exactly, RNG-independent. Re-pin message + ~10⁷ cap. G3 updated. Non-vacuous.

**PM-1/2/3 — ✅ folded**, no drift. Citations re-checked (repair.rs:760/437-441/1340/766-783; cmd/repair.rs:143-144; pipeline.rs:45-47/67).

## Minor polish (do not block GREEN; fold opportunistically in P0)
- **PM-r2-1 — Reject variant vs `is_indel_trigger`.** `mnemonic repair`'s loop has `Err(e) if args.max_indel>=1 && is_indel_trigger(&e)` (repair.rs:143-144, 1105). If Reject maps to an indel-trigger `RepairError` (e.g. `PostCorrectionDecodeFailed`, in the trigger set at :1109) and `--max-indel>=1`, a full-set miscorrection routes through `recover_indel_card` → all chunks BCH-correct → `failing.len()==0` → `Unrecoverable` → exit 2. **Funds-safe** (exit 2 either way) but the user gets the generic "indel unrecoverable" message, not "corrected each chunk but the set does not reassemble." Fix: pick a Reject variant NOT in `is_indel_trigger`, or short-circuit Reject before the indel check, so the precise message survives at `--max-indel>=1`. **[Recommend folding in P0.]**
- **PM-r2-2 — batch mixed-group return/message.** A batch folding to Reject (one group miscorrects, another blesses) returns `Err` → the co-batched blessed group's chunks are suppressed too. **Fail-safe** (never emits the bad group as recovered) but the plan should (a) state a dominant-Reject suppresses all output, and (b) ensure the reject message names WHICH `chunk_set_id` group failed so the user can re-run the good group alone. **[Recommend folding in P0.]**
- **PM-r2-3 — loose wording (plan P0 src bullet).** "return the VERIFY-ME (exit-4) candidate outcome" — exit 4 is a `mnemonic repair` CALLER mapping; `repair_card` should return the kind-agnostic `Unverified` verdict (the authoritative subsection has it right). Align wording to "return `Unverified`; caller maps to exit 4" so no one bakes exit-4 into the shared engine.
- **PM-r2-4 — `--json` candidate signal.** For a Candidate, `mnemonic repair --json`(exit 4)/`mk repair --json`(exit 5) emit corrected_chunks with the advisory only on stderr; a stdout-only `--json` consumer can't distinguish blessed vs candidate. Not funds-critical (exit code + advisory carry it; wire-shape not gated). Consider an `unverified: true`/`set_verify` envelope field; if added, note it's a wire-shape change consumers self-update.

## Gate result
0 Critical / 0 Important. **The plan passes plan-R0.** Fold PM-r2-1 + PM-r2-2 during P0 for message precision (recommended, non-gating). Implementation may proceed per CLAUDE.md phase-3 (single implementer, TDD tests-before-src, worktree; per-phase R0 running FULL `cargo test -p`; mandatory post-impl whole-diff review).
