# `repair --max-indel` (m-format incorrect-length recovery) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `mnemonic repair --max-indel <N>` so a transcribed `ms1`/`mk1` string with an inserted-too-long or dropped-too-short character is recovered by enumerate-and-validate around the existing BCH decode.

**Architecture:** A new toolkit-only `indel.rs` engine generates candidate full-strings via two producers — prefix-region known-string restore (P1) and data-region BCH-guided edits (P2: delete-and-validate for too-long, placeholder-then-decode for too-short) — and validates each through a per-kind oracle that reuses the existing `repair.rs` primitives (`parse_chunk`, `polymod_residue`, `decode_*_errors`, `encode_chunk`) and the sibling codecs' decode. `cmd/repair.rs` wires the flag, engages the engine when normal repair fails, and maps the result to exit `0/5/4/2`.

**Tech Stack:** Rust; `ms-codec 0.2.0` (`decode_with_correction`), `mk-codec 0.3.1` (`string_layer::bch` + `bch_decode` + `decode(&[&str])`); clap-derive; serde_json.

**SemVer:** PATCH → toolkit `v0.37.1`; paired GUI schema PATCH → `mnemonic-gui v0.21.2`.

---

## §0 Context

- **Spec (source of truth):** `design/BRAINSTORM_m_format_incorrect_length_recovery.md` (GREEN through R0→R1; reviews in `design/agent-reports/m-format-incorrect-length-recovery-brainstorm-R{0,1}-review.md`).
- **Source ground-truth SHA:** branch `m-format-incorrect-length-recovery` = `origin/master 925f5ed` + the design commit (source files unchanged from `925f5ed`). **All `:line` anchors below were read at this tip; per CLAUDE.md re-grep each at the moment you touch the file.**
- **Distinct from existing `repair`** (BCH substitution at fixed length). An indel changes length / shifts the tail, so it needs the enumerate-and-validate layer specified here.
- **BCH facts (verified):** regular `BCH(93,80,8)` 13-symbol checksum / long `BCH(108,93,8)` 15-symbol; 8 syndromes ⇒ substitution capacity **t = 4**, enforced per-codec: ms1 path at `ms-codec bch_decode.rs:416` (`if deg == 0 || deg > 4`), mk1 path at `mk-codec/.../string_layer/bch_decode.rs:566` (R0 M4). This is the recovery ceiling for the too-short direction.

## §1 Locked decisions (from spec, R0/R1-confirmed)

1. **Surface:** one flag `--max-indel <N>` on `mnemonic repair`. `value_parser` range `0..=4`, **default `0`** (= current behavior byte-for-byte; auto-fire never sets it).
2. **HRPs:** `ms1` + `mk1` this cycle. **`md1` refused** (footgun-tagged) when it would enter indel search; FOLLOWUP (b).
3. **Engine:** structure A — new `indel.rs`, one validator + two producers (P1 prefix, P2 data-part). **Single-region per attempt** (P1 up to N OR P2 up to N; cross-region split deferred — FOLLOWUP (c)).
4. **Pure-indel only:** accept a candidate iff its BCH corrections are a **subset of the inserted-placeholder positions** (∅ for delete/prefix). Indel+substitution deferred — FOLLOWUP (d). *(⊆, not ==, to admit the placeholder-equals-true-symbol edge case.)*
5. **mk1 per-chunk:** indel search runs on the single failing chunk; per-chunk validator = `decode_string`; after recovery the full `mk_codec::decode(&[&str])` reassembly confirms the cross-chunk hash (this is what disambiguates).
6. **Exit contract** (aligned to `cmd/repair.rs:122` `Ok(if total_repairs==0 {0} else {5})`): `0` already-valid · **`5` unique recovery (correction applied)** · `4` ambiguous (≥2 candidates) · `2` unrecoverable (new `RepairError` variant).
7. **Trigger:** engage indel search only when normal `repair_card` returns `Err(RepairError::{HrpMismatch | TooManyErrors | PostCorrectionDecodeFailed | UnparseableInput | ReservedInvalidLength})` AND `--max-indel ≥ 1`. (`EmptyInput`/`UnsupportedCodeVariant` pass through unchanged — see §2.4.) **`HrpMismatch` IS a trigger (mid-execution amendment, Phase-5 R0):** a prefix-region indel surfaces as `HrpMismatch` (`parse_chunk` checks the HRP before length — dropping 'm' from `ms1…` → `s1…` → "found 's'"), so excluding it would make the Phase-3 prefix producer CLI-unreachable, contradicting the user's explicit "recover indels at the HRP and separator" requirement. (Separator-drop → `UnparseableInput`, already a trigger.) **Tradeoff:** with `--max-indel ≥ 1`, a *genuine* wrong-HRP typo (e.g. `mk1…` to `--ms1`) now enters indel search and, failing, returns `IndelUnrecoverable` (exit 2) instead of the `HrpMismatch` "did you mean 'mk'?" suggestion. This is opt-in only (default `--max-indel 0` preserves the suggestion). A future refinement could fall back to the original `HrpMismatch` error on Unrecoverable; deferred (noted as FOLLOWUP candidate, non-blocking for v1).
8. **Secret:** `--max-indel` is non-secret (no `flag_is_secret` change); ms1 candidate output reuses the existing D9 advisory, fired once per emitted candidate.
9. **Reach FOLLOWUP:** erasure-decode → j=8 is a sibling-codec change, deferred (FOLLOWUP (a), companion entries).

## §2 Architectural strategy (file-level inventory)

### §2.1 New module `crates/mnemonic-toolkit/src/indel.rs`

Owns the HRP-agnostic search engine + types + the per-kind oracle trait. Pure logic, independently unit-testable.

```rust
//! Incorrect-length (indel) recovery for m-format strings — enumerate-and-
//! validate around the existing BCH decode. SPEC:
//! design/BRAINSTORM_m_format_incorrect_length_recovery.md.
//!
//! Two candidate producers feed one per-kind validator (`IndelOracle`):
//!   P1 prefix-region restore to the known `ms1`/`mk1` prefix;
//!   P2 data-region — delete-and-validate (too long) / placeholder-then-decode
//!      (too short, BCH solves the missing symbol).
//! Pure-indel only: a candidate's BCH corrections must be ⊆ the placeholder
//! positions we inserted (∅ for delete/prefix).

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndelRegion { Prefix, DataPart }

/// The repair OPERATION applied to the corrupted input to recover the original.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndelDirection { Inserted, Deleted } // Inserted = restored dropped char(s) (input too short); Deleted = removed added char(s) (input too long)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndelCandidate {
    pub recovered: String,        // full m*1 string (canonical, post-solve)
    pub indel_count: usize,       // j
    pub region: IndelRegion,
    pub direction: IndelDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndelOutcome {
    Unique(IndelCandidate),
    Ambiguous(Vec<IndelCandidate>), // ≥2 candidates with DISTINCT `recovered`
    Unrecoverable,
}

/// Per-kind single-string validator. `allowed` are the data-part indices of
/// placeholders we inserted (∅ for delete/prefix producers). Returns the
/// canonical recovered full-string iff the candidate decodes cleanly AND its
/// BCH corrections ⊆ `allowed`.
pub trait IndelOracle {
    fn validate(&self, candidate: &str, allowed: &BTreeSet<usize>) -> Option<String>;
}

/// Engine entry point. `input` is one full m*1 string (one ms1, or ONE mk1
/// chunk). `hrp` ∈ {"ms","mk"}. Produces the dedup'd outcome.
pub fn recover_indel(input: &str, hrp: &str, max_indel: usize, oracle: &dyn IndelOracle) -> IndelOutcome {
    let mut hits: Vec<IndelCandidate> = Vec::new();
    for j in 1..=max_indel {
        collect_prefix(input, hrp, j, oracle, &mut hits);   // P1
        collect_data_delete(input, hrp, j, oracle, &mut hits); // P2 too-long
        collect_data_insert(input, hrp, j, oracle, &mut hits); // P2 too-short
    }
    dedup_by_recovered(&mut hits);
    match hits.len() {
        0 => IndelOutcome::Unrecoverable,
        1 => IndelOutcome::Unique(hits.into_iter().next().unwrap()),
        _ => IndelOutcome::Ambiguous(hits),
    }
}
```

Helper contracts (full bodies in Phases 1-3):
- `fn data_part_bounds(input, hrp) -> Option<usize>` — byte offset where the data-part begins (`hrp.len()+1`) iff `input` starts with `"{hrp}1"`, else `None`.
- `collect_data_delete` — for each j-subset of data-part char positions, delete, re-assemble `K + d'`, `oracle.validate(cand, ∅)`; push `IndelCandidate{ region: DataPart, direction: Deleted, indel_count: j }`.
- `collect_data_insert` — for each j-subset of insertion slots in the data-part, insert PLACEHOLDER `PLACEHOLDER_CHAR` (a fixed bech32 symbol, `'q'` = value 0), record placeholder data-part indices `P`, `oracle.validate(cand, P)`; push `direction: Inserted`. (The oracle BCH-solves the placeholder; we accept iff corrections ⊆ P.)
- `collect_prefix` — for split point `p ∈ (3-j)..=(3+j)` (clamped ≥0, ≤ input len), if `levenshtein(&input[..p], &format!("{hrp}1")) == j` (EXACTLY j — the `1..=max_indel` outer loop assigns the exact `indel_count`; `<= j` would re-test at multiple j and mislabel `indel_count`) then `cand = format!("{hrp}1{}", &input[p..])`, `oracle.validate(cand, ∅)`; push `region: Prefix`. `direction` = Inserted if `p < 3` (chars were dropped from prefix) else Deleted.
- `PLACEHOLDER_CHAR: char = 'q'` (ALPHABET[0]); rationale comment: any fixed symbol works because the BCH decoder solves the true value; ⊆-check tolerates the collision case.
- `fn levenshtein(a, b) -> usize` — standard DP (small inputs; prefix region ≤ ~7 chars).
- `fn dedup_by_recovered(hits: &mut Vec<IndelCandidate>)` — **keys on `recovered` ONLY** (R0 I2), never the derived `PartialEq` (P1 and P2 can recover the same string with different `region`/`direction`; the derived `PartialEq` over all fields would leave both → false `Ambiguous`):
  ```rust
  hits.sort_by(|a, b| a.recovered.cmp(&b.recovered));
  hits.dedup_by(|a, b| a.recovered == b.recovered);
  ```

### §2.2 Oracles in `crates/mnemonic-toolkit/src/repair.rs` (reuse private primitives)

Implement both oracles in `repair.rs` (they need `parse_chunk`/`polymod_residue`/`decode_*_errors`/`encode_chunk`, which are private there) and expose constructors:

```rust
/// ms1 oracle — single string is the whole card; delegate to ms-codec.
pub(crate) struct Ms1IndelOracle;
impl crate::indel::IndelOracle for Ms1IndelOracle {
    fn validate(&self, cand: &str, allowed: &BTreeSet<usize>) -> Option<String> {
        match ms_codec::decode_with_correction(cand) {
            Ok((_t, _p, corrections)) => {
                if corrections.iter().all(|c| allowed.contains(&c.position)) {
                    // canonical = cand with corrections applied (apply_ms_corrections)
                    let (corrected, _) = apply_ms_corrections(cand, &corrections);
                    Some(corrected)
                } else { None } // a correction outside placeholder set ⇒ not pure-indel
            }
            Err(_) => None,
        }
    }
}

/// mk1 oracle — ⊆-gated BCH-solve the single failing chunk (mk_codec::decode
/// self-corrects t≤4 UNGUARDED, which would defeat the pure-indel rule — so we
/// solve under the ⊆ gate ourselves first), then confirm full-card reassembly
/// via mk_codec::decode(&[&str]) on the already-clean chunk (R0 M1).
pub(crate) struct Mk1IndelOracle { pub all_chunks: Vec<String>, pub failing_index: usize }
impl crate::indel::IndelOracle for Mk1IndelOracle {
    fn validate(&self, cand: &str, allowed: &BTreeSet<usize>) -> Option<String> {
        // 1. BCH-validate/solve the single chunk (reuse parse_chunk + polymod + decode_*_errors).
        let corrected_chunk = mk1_chunk_solve(cand, allowed)?; // None if unparseable / corrections ⊄ allowed
        // 2. Substitute into the full chunk set and confirm reassembly.
        let mut chunks = self.all_chunks.clone();
        chunks[self.failing_index] = corrected_chunk.clone();
        let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        match mk_codec::decode(&refs) { Ok(_) => Some(corrected_chunk), Err(_) => None }
    }
}

/// Reuse of repair_chunk_one's BCH core, gated by the ⊆-allowed pure-indel rule.
fn mk1_chunk_solve(cand: &str, allowed: &BTreeSet<usize>) -> Option<String> {
    let (values, code) = parse_chunk(cand, 0, CardKind::Mk1).ok()?;
    let target = CardKind::Mk1.target_residue(code)?;
    let residue = polymod_residue("mk", &values, target, code);
    if residue == 0 { return if allowed.is_empty() { Some(cand.to_string()) } else { Some(encode_chunk("mk", &values)) }; }
    let (positions, mags) = match code {
        BchCode::Regular => decode_regular_errors(residue, values.len()),
        BchCode::Long => decode_long_errors(residue, values.len()),
    }?;
    if !positions.iter().all(|p| allowed.contains(p)) { return None; }
    let mut corrected = values.clone();
    for (&p, &m) in positions.iter().zip(&mags) { if p >= corrected.len() { return None; } corrected[p] ^= m; }
    if polymod_residue("mk", &corrected, target, code) != 0 { return None; }
    Some(encode_chunk("mk", &corrected))
}
```

Add `use std::collections::BTreeSet;` to `repair.rs`. (ms-codec `CorrectionDetail.position` confirmed `decode.rs:88`; `apply_ms_corrections` at `repair.rs:822`.)

### §2.3 New `RepairError` variant (`repair.rs:388-430`)

Insert `IndelUnrecoverable { hrp: &'static str, max_indel: usize }` **alphabetically** (between `HrpMismatch` and `PostCorrectionDecodeFailed` → after `HrpMismatch`, before `PostCorrectionDecodeFailed`; place at the alphabetically-correct slot — `I` sorts after `H`). Add its `Display` arm. NB the surrounding enum is **not** alphabetized today (`error-rs-retroactive-alphabetical-sort`); do **not** reorder the pre-existing variants. Maps to exit 2 via the unchanged `ToolkitError::Repair(_) => 2` (`error.rs:507`).

### §2.4 CLI wiring (`cmd/repair.rs`)

- Add to `RepairArgs` (after `json`, `:59`):
```rust
/// Maximum insert/delete (indel) distance to search when a chunk fails
/// normal repair — recovers a single transcribed character that was
/// added (too long) or dropped (too short). 0 disables (default).
/// ms1/mk1 only; md1 indel recovery is not yet supported.
#[arg(long, value_name = "N", default_value_t = 0, value_parser = clap::value_parser!(u8).range(0..=4))]
pub max_indel: u8,
```
- `run()` (`:89-123`) — **multi-group-safe control flow (R0 I1).** A single invocation may carry multiple groups (`--ms1 X --mk1 Y`; D35). The current `repair_card(*kind, chunks)?` at `:103` short-circuits on the first failure; we replace the `?` with a `match` and aggregate, **without early-returning on the non-fatal (Ambiguous/recovered) outcomes**, so every group still emits. New orchestrator:
  ```rust
  pub(crate) fn recover_indel_card(kind: CardKind, chunks: &[String], max_indel: usize)
      -> Result<crate::indel::IndelOutcome, ToolkitError>;
  ```
  In `run()`, add `let mut ambiguous_seen = false;` before the loop and, per group:
  ```rust
  match repair::repair_card(*kind, chunks) {
      Ok(outcome) => { total_repairs += outcome.repairs.len(); /* emit as today (incl. `if matches!(kind, CardKind::Ms1) { any_ms1 = true; }` per :105-107) */ }
      Err(e) if args.max_indel >= 1 && is_indel_trigger(&e) => {
          match repair::recover_indel_card(*kind, chunks, args.max_indel as usize)? { // md1 → Err(BadInput) propagates here (exit 1)
              IndelOutcome::Unique(c)   => { emit_recovered(&c, args.json, stdout)?; total_repairs += 1; }
              IndelOutcome::Ambiguous(v)=> { emit_candidates(&v, args.json, stdout)?;
                                             writeln!(_stderr, "repair: ambiguous — {} candidates, choose manually", v.len()).ok();
                                             ambiguous_seen = true; } // NO return — continue the loop
              IndelOutcome::Unrecoverable => {
                  return Err(ToolkitError::Repair(RepairError::IndelUnrecoverable{ hrp: kind.hrp(), max_indel: args.max_indel as usize }));
              } // exit 2; retains today's first-failure short-circuit semantics
          }
          if matches!(kind, CardKind::Ms1) { any_ms1 = true; }
      }
      Err(e) => return Err(e.into()), // not a trigger / max_indel==0 → today's behavior
  }
  ```
  - `is_indel_trigger(&RepairError) -> bool` — true for `HrpMismatch | TooManyErrors | PostCorrectionDecodeFailed | UnparseableInput | ReservedInvalidLength` (§1.7 — `HrpMismatch` included so prefix-region indels engage); false for `EmptyInput | UnsupportedCodeVariant | IndelUnrecoverable`.
  - **md1 refusal:** `recover_indel_card` returns `Err(ToolkitError::BadInput("repair --max-indel: indel recovery is not yet supported for chunked md1".into()))` (exit 1) for `CardKind::Md1`. (Chosen `BadInput` because no `RepairError` variant fits a "not-yet-supported" refusal; R0-confirmed acceptable.)
  - **mk1 with >1 failing chunk** (exceeds single-region v1): `recover_indel_card` returns `Ok(IndelOutcome::Unrecoverable)`.
  - **ms1 advisory:** the existing post-loop site (`:118-120`, fired for `any_ms1`) is reached because the Ambiguous/recovered branches do NOT early-return — it covers recovered and ambiguous ms1 candidates. (Only the Unrecoverable `Err` short-circuits, which emits no stdout, so no advisory is owed.)
  - **Final exit (replaces `:122`):** `Ok(if ambiguous_seen { 4 } else if total_repairs == 0 { 0 } else { 5 })`. **Precedence:** `unrecoverable(2, via Err short-circuit) > ambiguous(4) > recovered/repaired(5) > already-valid(0)`.
- Runtime notice: if `args.max_indel >= 3`, `writeln!(_stderr, "repair: searching up to {} indels; this may take a few seconds", args.max_indel)`.
- `--json`: extend with a status envelope (new struct `IndelJson { schema_version:"1", status: "unique"|"ambiguous", candidates: Vec<IndelCandidateJson> }`); `IndelCandidateJson { recovered, indel_count, region, direction }`. Only the emitting outcomes (Unique/Ambiguous) produce a JSON envelope; **Unrecoverable surfaces via the `Err`/exit-2 path and emits NO JSON** (so `status` has no `"unrecoverable"` value — m3). Wire-shape is NOT schema_mirror-gated → GUI self-updates (§5 paired-PR).

`recover_indel_card(kind, chunks, max_indel, …)` (new in `repair.rs`) orchestrates per-kind:
- ms1: single chunk → `recover_indel(chunk, "ms", n, &Ms1IndelOracle)`.
- mk1: find the failing chunk (the one whose `repair_chunk_one` errored), build `Mk1IndelOracle{ all_chunks, failing_index }`, run `recover_indel(failing_chunk, "mk", n, &oracle)`. (If >1 chunk fails, that exceeds single-region v1 → `Unrecoverable`.)

### §2.5 Module registration

`crates/mnemonic-toolkit/src/main.rs` declares modules alphabetically; add `mod indel;` in its alphabetical slot — **between `mod friendly;` (`:15`) and `mod language;` (`:16`)** (R0 M3). Confirm with `grep -n "^mod " src/main.rs`.

## §3 Phase decomposition

> Per-phase TDD (test first), then per-phase opus reviewer-loop to 0C/0I, persist to `design/agent-reports/`. Tests are **BIN-target** (`cargo test -p mnemonic-toolkit --bins` / `--test <name>`), NOT `--lib`.

### Phase 0 — scaffolding (compiles, no behavior)

**Files:** Create `crates/mnemonic-toolkit/src/indel.rs`; Modify `src/main.rs` (`mod indel;`), `src/repair.rs` (`RepairError::IndelUnrecoverable` + Display + `use BTreeSet`).

- [ ] **Step 1 — failing test.** In `indel.rs` `#[cfg(test)] mod tests`:
```rust
#[test]
fn recover_indel_empty_budget_is_unrecoverable() {
    struct NoOracle;
    impl IndelOracle for NoOracle { fn validate(&self,_:&str,_:&BTreeSet<usize>)->Option<String>{None} }
    assert_eq!(recover_indel("ms1qqqq","ms",0,&NoOracle), IndelOutcome::Unrecoverable);
}
```
- [ ] **Step 2 — run, expect FAIL** (`recover_indel`/types undefined): `cargo test -p mnemonic-toolkit --bins indel::tests::recover_indel_empty_budget -v` → FAIL (unresolved).
- [ ] **Step 3 — add the §2.1 types + `recover_indel` skeleton** (loop `1..=max_indel` runs zero iterations at N=0 → `Unrecoverable`), the three `collect_*` as empty `fn(..) {}` stubs, `dedup_by_recovered`, `data_part_bounds`, `levenshtein`, `PLACEHOLDER_CHAR`. Add `mod indel;` to main.rs. Add `RepairError::IndelUnrecoverable{hrp,max_indel}` + Display arm.
- [ ] **Step 4 — run, expect PASS**; `cargo clippy --all-targets -- -D warnings` clean (watch `doc_lazy_continuation` on multi-line `///`).
- [ ] **Step 5 — commit** `feat(indel): Phase 0 scaffolding — engine types + recover_indel skeleton + RepairError::IndelUnrecoverable`.

### Phase 1 — P2 delete producer (too-long) + ms1 oracle

**Files:** Modify `indel.rs` (`collect_data_delete`, `dedup_by_recovered`), `repair.rs` (`Ms1IndelOracle` + `pub(crate) fn recover_indel_card` ms1 arm).

- [ ] **Step 1 — failing test** (`tests/cli_indel.rs` is later; this is a `repair.rs` BIN unit test that can build a real ms1 vector). Use a canonical zero-entropy ms1 string `MS1_VALID` (the v0.31.5 SeedQR zero-entropy vectors / derive empirically via `mnemonic` once; pin as a test const). Insert one extra char at index k of the data-part → assert `recover_indel(corrupted,"ms",1,&Ms1IndelOracle)` is `Unique` with `recovered == MS1_VALID`, `direction == Deleted`.
- [ ] **Step 2 — run, expect FAIL** (collect_data_delete is a stub).
- [ ] **Step 3 — implement** `collect_data_delete`:
```rust
fn collect_data_delete(input:&str, hrp:&str, j:usize, oracle:&dyn IndelOracle, out:&mut Vec<IndelCandidate>) {
    let Some(dstart) = data_part_bounds(input, hrp) else { return };
    let data: Vec<char> = input[dstart..].chars().collect();
    if data.len() <= j { return; }
    for combo in combinations(data.len(), j) {            // j-subsets of indices
        let kept: String = data.iter().enumerate()
            .filter(|(i,_)| !combo.contains(i)).map(|(_,c)| *c).collect();
        let cand = format!("{hrp}1{kept}");
        if let Some(rec) = oracle.validate(&cand, &BTreeSet::new()) {
            out.push(IndelCandidate{ recovered: rec, indel_count: j, region: IndelRegion::DataPart, direction: IndelDirection::Deleted });
        }
    }
}
```
plus `combinations(n,k) -> impl Iterator<Item=Vec<usize>>` (lexicographic k-subsets; cap guard not needed — clap bounds N≤4). Implement `Ms1IndelOracle` (§2.2) and `dedup_by_recovered` (§2.1 — keyed on `recovered` only). Add ms1 arm of `recover_indel_card`.
- [ ] **Step 4 — run, expect PASS**; add j=2 test (two inserted chars).
- [ ] **Step 5 — commit** `feat(indel): Phase 1 — too-long delete-and-validate + ms1 oracle`.

### Phase 2 — P2 insert producer (too-short) for ms1

**Files:** Modify `indel.rs` (`collect_data_insert`).

- [ ] **Step 1 — failing tests:** drop one char from `MS1_VALID` data-part at index k → `Unique`, `recovered==MS1_VALID`, `direction==Inserted`. Plus: **placeholder-collision** (drop a char whose value is `'q'` → still recovered, exercises the ⊆ rule). Plus: **pure-indel rejection** — drop one char AND flip another data char → `Unrecoverable` (a non-placeholder correction appears ⇒ rejected).
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement** `collect_data_insert`:
```rust
fn collect_data_insert(input:&str, hrp:&str, j:usize, oracle:&dyn IndelOracle, out:&mut Vec<IndelCandidate>) {
    let Some(dstart) = data_part_bounds(input, hrp) else { return };
    let data: Vec<char> = input[dstart..].chars().collect();
    let slots = data.len() + j;                  // post-insertion length
    for combo in combinations(slots, j) {        // which post-insertion indices are placeholders
        let mut built: Vec<char> = Vec::with_capacity(slots);
        let mut src = data.iter();
        for i in 0..slots {
            if combo.contains(&i) { built.push(PLACEHOLDER_CHAR); }
            else if let Some(c) = src.next() { built.push(*c); }
        }
        if built.len() != slots { continue; }    // ran out of source chars
        let allowed: BTreeSet<usize> = combo.iter().copied().collect();
        let cand: String = format!("{hrp}1{}", built.iter().collect::<String>());
        if let Some(rec) = oracle.validate(&cand, &allowed) {
            out.push(IndelCandidate{ recovered: rec, indel_count: j, region: IndelRegion::DataPart, direction: IndelDirection::Inserted });
        }
    }
}
```
- [ ] **Step 4 — run, expect PASS** (all three tests; j=2 too-short variant).
- [ ] **Step 5 — commit** `feat(indel): Phase 2 — too-short placeholder-then-decode (ms1), pure-indel ⊆ rule`.

### Phase 3 — P1 prefix producer + ambiguity contract (ms1)

**Files:** Modify `indel.rs` (`collect_prefix`).

- [ ] **Step 1 — failing tests:** (a) drop the `m` from `MS1_VALID` → `"s1…"` → `Unique`, `recovered==MS1_VALID`, `region==Prefix`, `direction==Inserted`. (b) add a char in the prefix `"msx1…"` → `Unique`, `region==Prefix`, `direction==Deleted`. (c) **dedup load-bearing test (R0 I2):** feed `dedup_by_recovered` two `IndelCandidate`s with IDENTICAL `recovered` but mismatched `region`/`direction` (one `Prefix/Inserted`, one `DataPart/Deleted`) → assert they collapse to ONE (so `recover_indel` reports `Unique`, not `Ambiguous`). (d) a genuine `Ambiguous` case (two DISTINCT `recovered` strings) → `Ambiguous(len 2)`.
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement** `collect_prefix`:
```rust
fn collect_prefix(input:&str, hrp:&str, j:usize, oracle:&dyn IndelOracle, out:&mut Vec<IndelCandidate>) {
    let k = format!("{hrp}1"); // known 3-char prefix
    let chars: Vec<char> = input.chars().collect();
    let lo = 3usize.saturating_sub(j);
    let hi = (3 + j).min(chars.len());
    for p in lo..=hi {
        let head: String = chars[..p].iter().collect();
        if levenshtein(&head, &k) != j { continue; } // exactly j edits in the prefix region
        let tail: String = chars[p..].iter().collect();
        let cand = format!("{k}{tail}");
        if let Some(rec) = oracle.validate(&cand, &BTreeSet::new()) {
            let direction = if p < 3 { IndelDirection::Inserted } else { IndelDirection::Deleted };
            out.push(IndelCandidate{ recovered: rec, indel_count: j, region: IndelRegion::Prefix, direction });
        }
    }
}
```
- [ ] **Step 4 — run, expect PASS.**
- [ ] **Step 5 — commit** `feat(indel): Phase 3 — prefix-region restore + ambiguity/dedup contract (ms1)`.

### Phase 4 — mk1 support (per-chunk + reassembly oracle)

**Files:** Modify `repair.rs` (`Mk1IndelOracle`, `mk1_chunk_solve`, mk1 arm of `recover_indel_card`).

- [ ] **Step 1 — failing tests** (BIN, in `repair.rs` tests using real mk1 vectors built via `mk_codec`): (a) single-chunk mk1, drop one data char → `Unique`, reassembles. (b) **multi-chunk** mk1 (two chunks), corrupt ONE chunk with an inserted char → `Unique` recovered chunk, full `decode(&[&str])` succeeds. (c) reassembly disambiguation: a chunk-level BCH-ambiguous candidate set collapses to one after the cross-chunk-hash check.
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement** `Mk1IndelOracle` + `mk1_chunk_solve` (§2.2) and the mk1 arm of `recover_indel_card` (locate failing chunk via per-chunk `repair_chunk_one` error; >1 failing ⇒ `Unrecoverable`).
- [ ] **Step 4 — run, expect PASS.**
- [ ] **Step 5 — commit** `feat(indel): Phase 4 — mk1 per-chunk recovery + reassembly oracle`.

### Phase 5 — CLI wiring (`cmd/repair.rs`) + integration tests

**Files:** Modify `cmd/repair.rs` (flag, trigger, exit, --json, notice, md1 refusal, advisory); Create `tests/cli_indel.rs`.

- [ ] **Step 1 — failing integration tests** in `tests/cli_indel.rs` (use `MNEMONIC_FORCE_TTY` as needed; pattern off `tests/cli_repair.rs`):
  - too-long ms1 recovered → stdout has recovered string, exit `5`.
  - too-short ms1 recovered → exit `5`.
  - unrecoverable (budget too small) → exit `2`, stderr "unrecoverable within --max-indel".
  - ambiguous (synthetic) → exit `4`, all candidates on stdout.
  - `--max-indel 0` on a broken-length input → unchanged failure (regression guard: identical to today).
  - `--max-indel 5` → clap rejects (exit 2, usage error).
  - md1 + `--max-indel 1` on a broken md1 → refusal message, exit 1.
  - ms1 recovery fires the secret advisory on stderr.
  - `--json` emits `status`+`candidates`.
- [ ] **Step 2 — run, expect FAIL** (flag/trigger absent).
- [ ] **Step 3 — implement** §2.4 (flag, trigger branch, `recover_indel_card` dispatch, exit mapping, `--json` envelope, notice, md1 refusal, advisory).
- [ ] **Step 4 — run, expect PASS**; full `cargo test -p mnemonic-toolkit` + `cargo clippy --all-targets -- -D warnings`.
- [ ] **Step 5 — commit** `feat(indel): Phase 5 — repair --max-indel CLI surface + exit/json/advisory wiring`.

### Phase 6 — lockstep + release-prep

**Files:** Modify `mnemonic-gui/src/schema/mnemonic.rs` (paired repo), `docs/manual/src/40-cli-reference/41-mnemonic.md` (+ a recovery section), `design/FOLLOWUPS.md`, `Cargo.toml`, `Cargo.lock`, both README version markers, `scripts/install.sh:32`, `CHANGELOG.md`.

- [ ] **Step 1 — GUI schema_mirror (paired PR):** add `max-indel` to `REPAIR_FLAGS` in `mnemonic-gui/src/schema/mnemonic.rs` (`~:1513-1561`) with `FlagKind::Number { min: 0, max: NumberMax::Static(4) }` (R0 M2 — `value_parser!(u8)` flags emit `gui-schema` kind `"number"`; the enum is `mnemonic-gui/src/schema/mod.rs:121`), NOT `FlagKind::Text`. The `schema_mirror` gate keys on flag-NAME only, but the kind drives GUI widget rendering (spinner vs free-text). Bump GUI to `v0.21.2`; pin toolkit `v0.37.1`. Verify `schema_mirror` passes against the pinned binary.
- [ ] **Step 2 — manual mirror:** add `--max-indel` to the `repair` flag table in `41-mnemonic.md` + a short "Recovering an incorrect-length card" subsection; run `make -C docs/manual lint MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=…`.
- [ ] **Step 3 — FOLLOWUPS:** flip `m-format-incorrect-length-recovery` `open → resolved`; FILE four new entries: (a) erasure-decode→j=8 (with companion entries in `mnemonic-secret`/`mnemonic-key`/`descriptor-mnemonic` `design/FOLLOWUPS.md` per CLAUDE.md cross-repo convention), (b) md1 chunked indel recovery, (c) cross-region split, (d) indel+substitution.
- [ ] **Step 4 — version + release-prep:** `Cargo.toml` → `0.37.1`; `cargo check` then **stage `Cargo.lock`** (memory: check-only leaves stale lock); both `<!-- toolkit-version: X -->` README markers (`readme_version_current` guard); `scripts/install.sh:32` `mnemonic-toolkit-v0.37.1`; `CHANGELOG.md` entry.
- [ ] **Step 5 — commit + tag prep** (do NOT tag/push until end-of-cycle opus review GREEN); clean `git status --porcelain` before any ship sequence.

## §4 Test corpus

### §4.1 `indel.rs` unit tests (engine, oracle-mocked + ms1-real where helpful)
1. N=0 ⇒ Unrecoverable (Phase 0).
2. combinations(n,k) correctness (lexicographic, count `C(n,k)`).
3. levenshtein small cases (incl. insert/delete).
4. dedup_by_recovered collapses two hits with identical `recovered` but mismatched `region`/`direction` → one (load-bearing per R0 I2; must NOT use the derived `PartialEq`).

### §4.2 `repair.rs` BIN unit tests (real vectors)
5. ms1 too-long j=1/2 → Unique.
6. ms1 too-short j=1/2 → Unique (incl. placeholder-collision).
7. ms1 pure-indel rejection (indel + substitution ⇒ Unrecoverable).
8. ms1 prefix drop/add → Unique (region Prefix).
9. ms1 Ambiguous (synthetic or constructed).
10. mk1 single-chunk too-long/too-short → Unique.
11. mk1 multi-chunk, one corrupted chunk → Unique + reassembly Ok.
12. mk1 >1 failing chunk ⇒ Unrecoverable (single-region v1).
13. `#[ignore]` runtime-sanity: ms1 too-short j=4 on a 75-len card completes < a few seconds.

### §4.3 `tests/cli_indel.rs` integration (exit/json/advisory)
14-22 per Phase 5 Step 1 (exit 0/5/4/2; --max-indel 0 regression; clap range; md1 refusal; ms1 advisory; --json shape).
22a. **Multi-group (R0 I1):** `--ms1 <too-long, recovers> --mk1 <valid>` → BOTH groups emit (ms1 recovered string + mk1 passthrough), exit `5`. And `--ms1 <ambiguous> --mk1 <valid>` → ms1 candidates emitted AND mk1 emitted (no group skipped), exit `4` (ambiguous precedence over the valid mk1's 0). Guards the no-early-return aggregation.

### §4.4 Cross-cycle regression
23. existing `tests/cli_repair.rs` + `cli_auto_repair.rs` unchanged green (default `--max-indel 0` path identical; auto-fire untouched).

### §4.5 Manual lint (Phase 6)
24. `docs/manual/tests/lint.sh` flag-coverage green for the new `--max-indel`.

## §5 Risks & cycle-close FOLLOWUPs

**Risks.** (R1) clap `value_parser!(u8).range(0..=4)` semantics — verify the exact builder API at impl time (it changed across clap versions). (R2) ms1 canonical test vectors — derive empirically once and pin; do not hand-fabricate. (R3) mk1 reassembly (R0 M1): `mk_codec::decode(&[&str])` → `decode_string` → `bch_correct_*` (`mk-codec/.../string_layer/bch.rs:683-687`) **self-corrects up to t=4 UNGUARDED**. We must NOT rely on that built-in correction (it would silently apply ≤4 substitutions anywhere, defeating the pure-indel ⊆ rule). So `mk1_chunk_solve` does the ⊆-gated BCH solve itself, then hands `decode` an already-clean chunk (it then sees 0 corrections). Re-confirm `bch.rs:683-687` in Phase 4. (R4) performance at j=4 too-short for mk1 long (108) — keep the `#[ignore]` sanity test; clap caps N≤4.

**FOLLOWUPs (file in Phase 6):** (a) erasure-decode→j=8 + sibling companions; (b) md1 chunked indel; (c) cross-region split; (d) indel+substitution. Flip the parent slug to `resolved`.

## §6 Reviewer-loop expectations

- **This plan-doc faces its own mandatory opus R0** (0C/0I) BEFORE Phase 0 — per the user's request and CLAUDE.md. Fold → persist to `design/agent-reports/` → re-dispatch until GREEN.
- Per-phase: tests first; per-phase opus reviewer-loop to 0C/0I; persist each round to `design/agent-reports/m-format-incorrect-length-recovery-phase-N-<round>.md` BEFORE fold-and-commit.
- End-of-cycle opus review GREEN before tag/push.

## §7 Critical files

- `crates/mnemonic-toolkit/src/indel.rs` (NEW — engine).
- `crates/mnemonic-toolkit/src/repair.rs` (oracles + `recover_indel_card` + `RepairError::IndelUnrecoverable`; reuse `parse_chunk:511`, `polymod_residue:584`, `encode_chunk:595`, `repair_chunk_one:607`, `apply_ms_corrections:822`).
- `crates/mnemonic-toolkit/src/cmd/repair.rs` (flag `:36-72`, `run:89-123`, exit `:122`, advisory `:118-120`, `--json` `:153-204`).
- `crates/mnemonic-toolkit/src/error.rs` (`Repair(_) => 2` `:507` — unchanged).
- `crates/mnemonic-toolkit/src/main.rs` (`mod indel;`).
- `mnemonic-gui/src/schema/mnemonic.rs` (`REPAIR_FLAGS`), `docs/manual/src/40-cli-reference/41-mnemonic.md`, `design/FOLLOWUPS.md`, `Cargo.toml`/`Cargo.lock`, READMEs, `scripts/install.sh`, `CHANGELOG.md`.

## §8 Verification (end-of-cycle)

```
cargo test -p mnemonic-toolkit                       # all green incl. cli_indel
cargo test -p mnemonic-toolkit -- --include-ignored  # runtime-sanity j=4
cargo clippy --all-targets -- -D warnings
make -C docs/manual lint MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=…
# GUI repo: cargo test schema_mirror  (after pin bump to v0.37.1)
git status --porcelain                               # clean before ship
```

## §9 Rn fold log

- **R0 (plan-doc):** RED 0C/2I/4M (`design/agent-reports/m-format-incorrect-length-recovery-plan-R0-review.md`). All ~20 API/citation anchors ACCURATE. Folded: **I1** multi-group `run()` exit-aggregation (no mid-loop return on Ambiguous; per-group emission; precedence 2>4>5>0; multi-group test §4.3 #22a); **I2** `dedup_by_recovered` keyed on `recovered` only + load-bearing dedup test; **M1** mk_codec::decode self-corrects-unguarded rationale; **M2** GUI `FlagKind::Number{0..=4}`; **M3** `mod indel;` alphabetical slot; **M4** dual-codec t=4 citation. → R1 dispatched.
- **R1 (plan-doc):** **GREEN 0C/0I** (`design/agent-reports/m-format-incorrect-length-recovery-plan-R1-review.md`). All 6 folds RESOLVED against source; no new C/I; full regression set re-confirmed. 4 residual non-blocking Minors folded inline (m1 explicit `any_ms1` on the Ok arm; m2 `collect_prefix` `== j`; m3 `IndelJson.status` drops `"unrecoverable"`; m4 mk path). **Plan clears the mandatory 0C/0I gate — cleared for Phase 0.**
- **Phase 5-prep amendment:** `is_indel_trigger` now INCLUDES `RepairError::HrpMismatch` (§1.7/§2.4). A prefix-region indel (the user's explicit "HRP and separator" requirement) surfaces as `HrpMismatch` because `parse_chunk` validates the HRP before length; excluding it would make the Phase-3 prefix producer CLI-unreachable. Opt-in `--max-indel≥1` tradeoff (loses "did you mean" on genuine typos) documented §1.7; default-0 unchanged. Surfaced while tracing Phase-5 trigger logic; to be scrutinized by the Phase-5 per-phase review.

## §10 Next steps

1. Dispatch opus architect **R0 of this plan-doc** → fold → persist → re-dispatch until GREEN (0C/0I). **No code before GREEN.**
2. Execute Phase 0…6 with per-phase TDD + reviewer-loop.
3. End-of-cycle review GREEN → release-prep → tag `mnemonic-toolkit-v0.37.1` + paired `mnemonic-gui v0.21.2`.
