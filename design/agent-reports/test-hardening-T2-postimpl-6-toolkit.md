# Post-impl whole-diff R0 — T2-a (#6) toolkit `prop_repair_never_wrong` — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Reviewed the UNCOMMITTED test-only diff @ `mnemonic-toolkit master` (in-crate `#[cfg(test)] mod prop_repair_never_wrong` + `main.rs` decl + `FOLLOWUPS.md` entry). Tree left byte-clean.

## Execution evidence

**1. Suites green in my hands (pinned 1.85.0):**
- `cargo test -p mnemonic-toolkit`: **3704 passed / 0 failed / 0 ignored**, 206 result lines, EXIT=0.
- `cargo test -p mnemonic-toolkit prop_repair_never_wrong`: **9/9** in 1.47s (and 9/9 again post-revert, 0.69s).

**2. Oracle independence — audited, no tautology.** The 3 ≤t cells + the >4 smoke assert against const fixtures / `ms_codec::encode` output (original bytes; `prop_repair_never_wrong.rs:207,221-222,244-245,305`). The indel cells' oracle is the original card / a mock. `f4_b`'s ground truth is an inline `mk_codec::decode` Err assert (`:416-419`) — sibling-codec, zero `repair.rs` involvement; the fixture builder (`:160-174`) uses only public `mk_codec::string_layer::encode_5bit_to_string` (verified: 13-symbol regular checksum, 8-symbol chunked header, auto code-select — `vendor/mk-codec/src/string_layer/bch.rs:508-565`) and is self-validating (searches alter_idx 8..64 for a decode-breaking alteration). `f4_a`/`f4_c` pin repair's *classification* — that's the property, ground-truthed by independent equality + `repairs` non-empty first. No cell's oracle is the code under test.

**3. All RED-proofs reproduced by execution** (in-tree mutate → run → revert; `repair.rs`/`indel.rs` sha256-verified back to pristine after each):
- **M1 (crown jewel, `f4_b`)** — deleted `verify_mk1_set` at repair.rs:1115 → `Ok(RepairOutcome{... repairs:[chunk 1, positions (5,'c','h'),(20,'d','v'),(40,'y','r')], set_verify: Blessed})` re-emitting the doctored chunk — the exact F4 wrong-wallet shape; panic at `prop_repair_never_wrong.rs:426`. The fixture assert passed before the panic, proving the doctored-2-chunk construction is genuinely per-chunk-valid + reassembly-breaking. Determinism argument holds: distance(corrupted, doctored)=3 ≤ t; distance(corrupted, original) ≥ d−3 ≥ 6 > t (13 recomputed checksum symbols), so correction lands on the doctored codeword every run.
- **M2 (`f4_a`)** — flipped repair.rs:1163 to `!repairs.is_empty()` → RED: "a touched ms1 correction must be Unverified (Cycle-F demotion)" (`:389`).
- **M3 (indel fold)** — replaced indel.rs:116-121 with return-first-hit → RED: `got Unique(IndelCandidate{recovered:"ms1pzr",...})` (`:350`).
- **M4/M5/M6 (≤t cells)** — repair.rs:782 `^ m ^ 1` → mk1 cell RED (via `.expect("repair Ok")`: the defensive re-verify at repair.rs:793-799 converts it to `TooManyErrors`); `apply_ms_corrections` repair.rs:1267 no-op emit → ms1 cell RED **on the pure equality oracle** (`…sxrnq…` ≠ `…sxraq…`); `apply_md_corrections` repair.rs:1677 no-op emit → md1 cell RED on equality. M5/M6 prove the original-bytes oracle catches a wrong-codeword emit no internal check sees.
- **M7 (`f4_c` flip-RED-first)** — simulated the future FOLLOWUP demotion at repair.rs:1599 → `f4_c` RED with the FOLLOWUP slug in the panic message. Confirmed the KAT pins current `Blessed` and does NOT assert never-bless.

**4. FOLLOWUP entry accurate + funds-adjacent.** Verified against source: single-string mk1 has no cross-chunk hash (`vendor/mk-codec/src/string_layer/pipeline.rs:7-9`), is unreachable from real v0.1 encoders (repair.rs:841-843), md1 delegate always-`Blessed` (repair.rs:1599). Cross-cites eval F4's "second oracle or demote" prescription; correctly notes ms1 already demoted.

**5. Generator flake-hunt: clean.** `substitute` shift 1..=31 mod 32 guarantees new≠old; shuffle+truncate gives distinct positions; edits are data-part-only over the all-lowercase `ALPHABET` (no '1' in bech32 charset → `rfind('1')` always hits the separator; verified for all 4 fixtures); per-chunk ≤4 budget honored in the mk1/md1 cells. Triangle-inequality analysis: ≤t recovery and >4 never-original are both deterministic (no proptest-seed sensitivity); indel Unique-but-≠original is impossible (the original-restoring candidate is always a hit → Unique(original) or Ambiguous, both accepted).

**6. NO-BUMP + gates.** main.rs adds only a `#[cfg(test)] mod` decl; nothing in the `#[cfg(fuzzing)]` lib block (lib.rs:144-190 untouched); Cargo.toml/Cargo.lock unmodified (`proptest`/`rand` dev-deps pre-existing). `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: clean. rustfmt **1.95.0** `--check` on the new file + main.rs: clean (never `--all`). Final: `git status` tracked-modified = exactly `main.rs` + `design/FOLLOWUPS.md`, untracked-new = `prop_repair_never_wrong.rs`; `git diff repair.rs indel.rs mlock.rs` = **0 lines**; all three sha256-match pristine.

## Findings

**Critical:** none. **Important:** none.

**Minor (non-blocking):**
1. `prop_repair_never_wrong.rs:337-354` duplicates `indel.rs:357-374` byte-identically (same `AcceptAll` recipe, input `"ms1qpzr"`, assertion). Spec-directed mirroring; redundant coverage, harmless.
2. The ≤t leg's RED-proof comment (`prop_repair_never_wrong.rs:181-187`) claims the repair.rs:782 mutation fails `prop_assert_eq!`; in execution the mk1 cell fails earlier at `.expect("repair Ok")` because the defensive re-verify intercepts. Cell still REDs; comment mischaracterizes the mk1 failure mode.
3. `f4_b` REDs via the tri-state match arm rather than the SPEC §T2-a(b)-worded post-hoc `mk_codec::decode(corrected)` re-check. Equivalent, deterministic RED; the independent decode is used as a fixture-validity assert instead.
4. FOLLOWUP phrase "bless any correction that re-checksums cleanly" slightly overstates for single-string mk1 (its group verify runs full `mk_codec::decode`, so an alias must fully decode, not merely re-checksum). Conservative-direction imprecision only.

**VERDICT: GREEN (0C/0I)**

---
**FOLD STATUS (opus, 2026-07-10):** 0C/0I → no re-dispatch required. Cosmetic Minors #2 (comment accuracy) + #4 (FOLLOWUP phrasing) to be fixed in the ship commit (doc-only, non-source). #1 (redundant ambiguity cell) — dispatcher-directed for a self-contained harness; kept. #3 (equivalent-RED path) — accepted as designed. Dispatcher will independently re-reproduce F4(b) in the pre-ship pass. Awaiting md #7 + mk #8 R0 before the T2 ship.