# Review — toolkit ms-repair all-length regression cell (cli_repair cell_12b)

Opus code-reviewer. Reviewed `cell_12b_ms1_repair_works_for_all_entropy_lengths` (committed master `739acc9`). Persisted by controller (review agent had no Write tool).

## Confirmations (file:line)
- **Exit-code contract matches the cell.** `ToolkitError::Repair(_) => 2` (error.rs:507); `From<RepairError>` (error.rs:336-339). Pre-fix, clean longer ms1 → `repair_via_ms_codec` → `ms_codec::decode_with_correction` returns `Err(TooManyErrors)` (repair.rs:834) → exit **2**. Cell asserts `code(0)` → would have FAILED pre-fix. Genuine gate.
- **Repaired path gated too:** 1-error longer ms1 pre-fix → `TooManyErrors` → exit 2 not 5. Cell asserts `code(5)` → fails pre-fix.
- **Fixtures valid** for all 4 lengths (`encode(Tag::ENTR, &Payload::Entr({20,24,28,32}B))`); `flip_at(&valid,5)` = one correctable in-data-part substitution; HRP untouched.
- **ms-codec is a direct toolkit dep** (Cargo.toml:20) → in-test `use ms_codec` legal.
- Codec-level coverage exists upstream (`ms-codec/tests/bch_all_lengths.rs`); 16-byte covered toolkit-side by the `VALID_MS1` cells; {20,24,28,32} is the correct non-overlapping complement.

## CRITICAL — None.   ## IMPORTANT — None.
## MINOR (no action): M1 echo assertion slightly weak (could also assert the report position line); M2 no `--nocapture` needed; M3 exit-code literals stable (pinned by error.rs + unit test).

## Scope-D completeness: ADD a `--max-indel` all-lengths cell
cell_12b covers ONLY the substitution path (`repair --ms1` BCH correction). Scope D named BOTH `repair_card(CardKind::Ms1)` AND `repair --max-indel`. The `Ms1IndelOracle::validate` (repair.rs:885-908) delegates to the SAME `decode_with_correction`, so `--max-indel` was ALSO broken for longer seeds (every candidate → `Err` → `None` → `Unrecoverable` → exit 2). It's a distinct caller route (oracle candidate-enumeration with the ⊆-gate), NOT covered by the substitution cell → ADD `cell ms1_max_indel_recovers_all_entropy_lengths` (per length: drop one char, `repair --ms1 <short> --max-indel 1`, exit 5 + recovered). Additive, not redundant.

## VERDICT: GREEN
cell_12b is correct, non-vacuous, gates the fix — stands as-is. The `--max-indel` all-lengths cell completes scope D (recommended follow-on, not a blocker).
