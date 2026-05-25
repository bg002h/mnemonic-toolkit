# indel v2 (cross-region + substitution + HrpMismatch-fallback) Implementation Plan — v0.37.3

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`).

**Goal:** Extend `mnemonic repair --max-indel` with cross-region indel search, an opt-in `--max-subst <E>` substitution-tolerance axis (candidate-list + verify advisory), and an HrpMismatch suggestion-fallback — implemented together with explicit non-breaking guarantees.

**Architecture:** Three layers of the existing engine: (1) candidate generator → two-level cross-region search subsuming the single-region producers; (2) per-kind oracle accept gate → relax `corrections ⊆ placeholders` to `|corrections \ placeholders| ≤ E` and return a `subst_count`; (3) `cmd/repair.rs::run` error/exit → candidate-list output + verify advisory + exit 4 on substitution-bearing results + downstream HrpMismatch fallback. Toolkit-only (error-decoder + placeholder).

**Tech Stack:** Rust; ms/mk/md codecs (error-decode + reassemble); clap; serde_json.

**SemVer:** PATCH → toolkit `v0.37.3`; paired GUI schema PATCH → `mnemonic-gui v0.21.3`.

---

## §0 Context
- **Spec:** `design/BRAINSTORM_indel_cross_region_subst_fallback.md`. **Base SHA:** `master` = `a6987f4`. Re-grep all `:line` anchors when touching files.
- Resolves `m-format-indel-cross-region-split`, `m-format-indel-plus-substitution`, `m-format-indel-hrpmismatch-suggestion-fallback`. Leaves `erasure-decode-extend-to-8` + `asymmetric-delete-budget` open.
- **Exit invariant preserved:** `5 = trust it`, `4 = verify it`. `--max-subst 0` default ⇒ byte-identical to v0.37.2.

## §1 Current shapes (post-v0.37.2)
- `indel.rs`: `IndelCandidate { recovered, indel_count, region, direction }` (`:30`); `IndelRegion::{Prefix, DataPart}` (`:14`); `recover_indel(input, hrp, max_indel, oracle)` (`:61`) loops 3 producers per j; `collect_prefix` (`:88`), `collect_data_delete` (`:152`), `collect_data_insert` (`:186`); `IndelOracle::validate(&self, &str, &BTreeSet<usize>) -> Option<String>` (`:51`); `dedup_by_recovered` (`:225`); `combinations` (`:123`); `data_part_bounds` (`:231`); `levenshtein` (`:241`).
- `repair.rs`: oracles `Ms1IndelOracle::validate` (`:886`), `mk1_chunk_solve` (`:910`), `md1_chunk_solve` (`:950`), `Mk1IndelOracle`/`Md1IndelOracle` (`:986`/`:1008`); `is_indel_trigger` (`:1038`); `indel_exit_code(ambiguous_seen, total_repairs)` (`:1055`); `recover_indel_card(kind, chunks, max_indel)` (`:1071`).
- `cmd/repair.rs`: imports (`:18`); `RepairArgs` (`:37`), `--max-indel` (`:66`), `json` (`:60`); `run` loop (`:106-192`) — match on `recover_indel_card`, `Unique→emit+total_repairs+=1`, `Ambiguous→emit+ambiguous_seen=true`, `Unrecoverable→Err(IndelUnrecoverable)`; post-loop ms1 advisory (`:189`); final `Ok(indel_exit_code(ambiguous_seen, total_repairs))` (`:192`); `IndelJson` (`:294`); `emit_indel_text`/`emit_indel_json` (`:333`/`:341`).

## §2 Phase decomposition

> Per-phase TDD + per-phase opus review to 0C/0I → persist to `design/agent-reports/indel-v2-phase-N-review.md`. BIN-target tests (`--bins` / `--test`), NOT `--lib`. Re-grep `:line` anchors before editing.

### Phase 1 — substitution accept gate (single-region) + `subst_count`
**Files:** `indel.rs` (trait, `IndelCandidate`, `recover_indel` + 3 producers thread `e_subst`), `repair.rs` (4 oracle sites + `recover_indel_card` thread `e_subst`).

**Signature changes (the §13.1 decision — `validate` returns the substitution count):**
```rust
// indel.rs
pub trait IndelOracle {
    fn validate(&self, candidate: &str, allowed: &BTreeSet<usize>, e_subst: usize)
        -> Option<(String, usize)>; // (recovered, subst_count = |corrections \ allowed|)
}
pub struct IndelCandidate { pub recovered: String, pub indel_count: usize,
    pub region: IndelRegion, pub direction: IndelDirection, pub subst_count: usize } // + subst_count
pub fn recover_indel(input: &str, hrp: &str, max_indel: usize, e_subst: usize, oracle: &dyn IndelOracle) -> IndelOutcome
```
Each producer threads `e_subst` into `validate` and, on `Some((rec, sc))`, pushes `IndelCandidate { …, subst_count: sc }`. (Delete/prefix pass `allowed=∅`, so any accepted substitution there is `sc≥1`.)

**Oracle gate (the relaxation) — Ms1 example (`repair.rs:886`):**
```rust
fn validate(&self, cand, allowed, e_subst) -> Option<(String, usize)> {
    match ms_codec::decode_with_correction(cand) {
        Ok((_t, _p, corrections)) => {
            let off = corrections.iter().filter(|c| !allowed.contains(&c.position)).count();
            if off <= e_subst {
                let (corrected, _) = apply_ms_corrections(cand, &corrections);
                Some((corrected, off))
            } else { None }
        }
        Err(_) => None,
    }
}
```
`mk1_chunk_solve`/`md1_chunk_solve` change identically: replace `if !positions.iter().all(|p| allowed.contains(p)) { return None }` with
```rust
let off = positions.iter().filter(|p| !allowed.contains(p)).count();
if off > e_subst { return None; }
```
and return `Option<(String, usize)>` (the `usize` = `off`); the `residue==0` early path returns `(encode_chunk(...), 0)`. `Mk1IndelOracle`/`Md1IndelOracle::validate` thread `e_subst` into `*_chunk_solve`, reassemble, and return `Some((corrected_chunk, off))`. **Note:** the decoder caps at t=4, so `placeholders + off ≤ 4` is auto-enforced (no extra clamp). `recover_indel_card` gains `e_subst: usize` and passes it to `recover_indel`.

- [ ] **Step 1 — failing test** (`repair.rs` tests): take `VALID_MS1`, drop one data char AND substitute another; assert `recover_indel(corrupted, "ms", 1, /*e_subst*/1, &Ms1IndelOracle)` is `Unique` with `recovered == VALID_MS1` and `subst_count == 1`. Plus: same corruption with `e_subst=0` → `Unrecoverable` (pure-indel rejects). Plus mk1/md1 single-chunk indel+subst (1 chunk of the 2/3-chunk fixtures) → Unique, subst_count=1.
- [ ] **Step 2 — run, expect FAIL** (signature mismatch / not yet recovering): `cargo test -p mnemonic-toolkit --bins indel_subst -v`.
- [ ] **Step 3 — implement** the signature + gate changes above across `indel.rs` + the 4 oracle sites + `recover_indel_card`. **EXHAUSTIVE call-site update (R0 I1 — every one of these compile-breaks at the signature change; re-grep line numbers first, they will have drifted):**
  - `IndelOracle::validate` impls (gate + tuple return): `repair.rs` `Ms1IndelOracle:886`, `Mk1IndelOracle:987`, `Md1IndelOracle:1009`; the `mk1_chunk_solve:910` / `md1_chunk_solve:950` helpers (gain `e_subst`, return `Option<(String, usize)>`, `residue==0` path returns `(…, 0)`).
  - test mocks `NoOracle` (`indel.rs:271`) + `AcceptAll` (`indel.rs:315`) → new `validate(&self, _, _, _e_subst) -> Option<(String, usize)>` (e.g. `AcceptAll` returns `Some((candidate.to_string(), 0))`).
  - `recover_indel(...)` 5-arg (add `e_subst`): `repair.rs:1082,1102,1123` (the 3 `recover_indel_card` arms) + test calls `repair.rs:1844,1863,1885,1907,1925,1939,1955,1972,2019,2036` + `indel.rs:280,320` (pass `e_subst=0` in all tests to preserve pure-indel assertions).
  - `recover_indel_card(...)` 4-arg (add `e_subst`): `cmd/repair.rs:142` + test calls `repair.rs:2054,2072,2152,2163,2177` (pass `0`).
  - `md1_chunk_solve(...)` direct test calls `repair.rs:2188,2190` (`indel_md1_chunk_solve_rejects_out_of_set_substitution`): add `e_subst=0` AND adapt the tuple return — e.g. `md1_chunk_solve(&bad, &allowed, 0).is_none()` and `md1_chunk_solve(MD1_C1, &allowed, 0).map(|(s,_)| s).as_deref() == Some(MD1_C1)`.
  - **`IndelCandidate { … }` struct-literal field-completeness (R1 I1 — adding `subst_count` breaks EVERY literal; the struct has no `#[non_exhaustive]`/`Default`, so each is a hard E0063):** producers `indel.rs:112,176,215` (set `subst_count` from the oracle's returned count) + test literals `indel.rs:292,298` (`dedup_collapses_same_recovered_with_differing_metadata`) + `cmd/repair.rs:368,374` (`emit_indel_two_candidate_text_and_json`) — the four test literals add `subst_count: 0`.
  - (`indel_exit_code` call sites are Phase 3, not here.)
  Trial `cargo build -p mnemonic-toolkit --tests` after wiring to flush any missed site before running tests.
- [ ] **Step 4 — run, expect PASS**: `cargo test -p mnemonic-toolkit --bins` (new + all prior green; the `e_subst=0` call sites keep pure-indel behavior). `cargo clippy --all-targets -- -D warnings` clean.
- [ ] **Step 5 — commit** `feat(indel): Phase 1 — substitution accept gate (|corrections\placeholders|≤E) + subst_count`.

### Phase 2 — cross-region two-level search
**Files:** `indel.rs` (restructure `recover_indel`; add `IndelRegion::CrossRegion`).

Restructure into prefix-restoration × data-edit, allocating the budget across regions:
```rust
pub enum IndelRegion { Prefix, DataPart, CrossRegion } // + CrossRegion

pub fn recover_indel(input, hrp, max_indel, e_subst, oracle) -> IndelOutcome {
    let mut hits = Vec::new();
    let k = format!("{hrp}1");
    // (data_part_string, j_prefix, prefix_direction) — j_prefix=0 yields the input's own
    // data-part iff `input` starts with k (prefix intact); j_prefix≥1 restores k within
    // exactly j_prefix edits (reuse the levenshtein window logic from old collect_prefix).
    for (data, j_prefix, pfx_dir) in prefix_restorations(input, &k, max_indel) {
        let data_budget = max_indel - j_prefix;
        for j_data in 0..=data_budget {
            if j_prefix == 0 && j_data == 0 { continue; } // un-edited input is not a recovery
            // generate data-region candidates (delete + insert) for exactly j_data edits;
            // j_data==0 ⇒ the single candidate `k + data` (validate as-is, allowed=∅).
            for (cand, allowed, data_dir) in data_variants(&k, &data, j_data) {
                if let Some((rec, sc)) = oracle.validate(&cand, &allowed, e_subst) {
                    let region = match (j_prefix > 0, j_data > 0) {
                        (true, true) => IndelRegion::CrossRegion,
                        (true, false) => IndelRegion::Prefix,
                        (false, true) => IndelRegion::DataPart,
                        (false, false) => unreachable!(),
                    };
                    let direction = if j_data > 0 { data_dir } else { pfx_dir };
                    hits.push(IndelCandidate { recovered: rec, indel_count: j_prefix + j_data,
                        region, direction, subst_count: sc });
                }
            }
        }
    }
    dedup_by_recovered(&mut hits);
    match hits.len() { 0 => Unrecoverable, 1 => Unique(...), _ => Ambiguous(hits) }
}
```
Extract `prefix_restorations` (from `collect_prefix`'s window+levenshtein, but yielding the data-part + j_prefix instead of validating) and `data_variants` (from `collect_data_delete`+`collect_data_insert`, yielding `(cand_full, allowed, direction)` for a given j_data; j_data=0 → the as-is candidate). Delete the old `collect_*` once subsumed. **`prefix_restorations` yield contract (R0 M3 — pin exactly, off-by-one risk):** the `j_prefix=0` case yields `(input_data_part, 0, _)` iff `data_part_bounds(input, hrp)` is `Some` (prefix intact); each `j_prefix ≥ 1` case enumerates split points `p ∈ [3.saturating_sub(j_prefix) ..= (3+j_prefix).min(len)]`, keeps those with `levenshtein(&chars[..p], k) == j_prefix` (exact, as the old `collect_prefix`), and yields `(chars[p..], j_prefix, Inserted if p<3 else Deleted)`. Total prefix-edit count stays in `1..=max_indel`. `direction` for CrossRegion is the data-part direction (documented; metadata-only — dedup is on `recovered`).

- [ ] **Step 1 — failing test:** `VALID_MS1` with the leading `m` dropped (prefix indel) AND one data char dropped (data indel); `recover_indel(corrupted, "ms", 2, 0, &Ms1IndelOracle)` → `Unique`, `recovered==VALID_MS1`, `region==CrossRegion`, `indel_count==2`. Plus regression: the existing prefix-only and data-only tests still pass (now via the unified path).
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement** the restructure + `prefix_restorations`/`data_variants` extraction + `IndelRegion::CrossRegion`. Keep `dedup_by_recovered`, `combinations`, `levenshtein`, `data_part_bounds`. Update any `IndelRegion` match (e.g. region-string mapping in `cmd/repair.rs`) for the new variant.
- [ ] **Step 4 — run, expect PASS** (cross-region test + all prior single-region tests green); clippy clean.
- [ ] **Step 5 — commit** `feat(indel): Phase 2 — cross-region two-level search (subsumes single-region producers)`.

### Phase 3 — CLI surface: `--max-subst`, candidate-list, advisory, exit, `--json`
**Files:** `cmd/repair.rs` (flag, run threading, emit, advisory, exit), `repair.rs` (`indel_exit_code` signature).

- **Flag** (after `max_indel`, `cmd/repair.rs:66`):
```rust
/// Also tolerate up to E substitution (wrong-but-in-place) errors alongside the
/// indels (default 0 = pure indel). Results that used a substitution are printed
/// as VERIFY-ME candidates (exit 4), not confident corrections. ms1/mk1/md1.
#[arg(long, value_name = "E", default_value_t = 0, value_parser = clap::value_parser!(u8).range(0..=4))]
pub max_subst: u8,
```
- **Thread** `args.max_subst as usize` into `recover_indel_card(*kind, chunks, args.max_indel as usize, args.max_subst as usize)`.
- **No-op notice (R0 M1):** `--max-subst` only takes effect via the indel path (gated by `args.max_indel >= 1`). If `args.max_subst >= 1 && args.max_indel == 0`, print a one-line stderr notice ("notice: --max-subst has no effect without --max-indel ≥ 1") before the loop (silent-default-with-notice convention).
- **Exit helper** (`repair.rs:1055`) — extend signature + fold substitution into the 4-tier:
```rust
pub(crate) fn indel_exit_code(ambiguous_seen: bool, substitution_seen: bool, total_repairs: usize) -> u8 {
    if ambiguous_seen || substitution_seen { 4 } else if total_repairs == 0 { 0 } else { 5 }
}
```
Update its test (`repair.rs:2088`) to the 3-arg form (add cells: `(false,true,1)→4`, `(false,false,5)→5`).
- **run()** (`cmd/repair.rs:106-192`): add `let mut substitution_seen = false;`. In the `Unique(c)` arm: `if c.subst_count >= 1 { substitution_seen = true; }`. In the `Ambiguous(v)` arm: `if v.iter().any(|c| c.subst_count >= 1) { substitution_seen = true; }`. After emitting, if `substitution_seen`, print the verify advisory to `stderr`:
```rust
writeln!(stderr, "repair: WARNING — candidate(s) required a substitution and are NOT confirmed corrections; derive an address from each and verify it controls your funds before trusting any (some may be false positives)").ok();
```
Final return: `Ok(repair::indel_exit_code(ambiguous_seen, substitution_seen, total_repairs))`.
- **`--json`** (`IndelJson`/`emit_indel_json`, `cmd/repair.rs:294/341`): add `subst_count: usize` per candidate (from `IndelCandidate.subst_count`) and a top-level `confident: bool` = `candidates.iter().all(|c| c.subst_count == 0)`. (Wire-shape not schema_mirror-gated.)

- [ ] **Step 1 — failing integration tests** (`tests/cli_indel.rs`): (a) ms1 indel+subst (`--max-indel 1 --max-subst 1` on drop+substitute) → exit **4**, stdout has recovered string, stderr has the verify WARNING; (b) ms1 pure-indel under `--max-subst 1` (only an indel, no subst needed) → exit **5** (no WARNING); (c) `--max-subst 0` regression → byte-identical to v0.37.2 (exit 5 pure-indel); (d) `--max-subst 5` → clap usage error; (e) `--json` carries `subst_count` + `confident:false` on a substitution recovery.
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement** the flag, threading, exit-helper change + test update, advisory, `--json` fields.
- [ ] **Step 4 — run, expect PASS**; full `cargo test -p mnemonic-toolkit`; clippy clean.
- [ ] **Step 5 — commit** `feat(indel): Phase 3 — --max-subst CLI + candidate-list verify advisory + exit-4 + --json subst_count/confident`.

### Phase 4 — HrpMismatch suggestion-fallback
**Files:** `cmd/repair.rs::run` (the `Unrecoverable` arm).

In the trigger match arm, the originating error `e` is in scope. Change the `Unrecoverable` handling: if `matches!(e, RepairError::HrpMismatch { .. })`, return the **original** error (which carries the "did you mean" suggestion via its `Display`) instead of `IndelUnrecoverable`:
```rust
IndelOutcome::Unrecoverable => {
    return match e {
        RepairError::HrpMismatch { .. } => Err(e.into()), // original suggestion preserved
        _ => Err(ToolkitError::Repair(RepairError::IndelUnrecoverable {
            hrp: kind.hrp(), max_indel: args.max_indel as usize })),
    };
}
```
(`e` is the `RepairError` bound by the `Err(e) if …` arm; `e.into()` uses the existing `From<RepairError>`.)

- [ ] **Step 1 — failing test** (`tests/cli_indel.rs`): a genuine wrong-HRP value (`--ms1 mk1<valid mk1 data>` — or a short `--ms1 mk1xxx`) with `--max-indel 1` → indel search fails → exit reflects the original `HrpMismatch` and stderr contains the "did you mean" / HRP-mismatch message (NOT "could not be recovered within --max-indel"). Plus regression: a recoverable prefix-drop (`--ms1 s10…`) still recovers (exit 5), proving the fallback only fires on genuine failure.
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement** the `Unrecoverable`-arm branch. **Also fold R0 I2 (doc drift):** update the `is_indel_trigger` doc-comment (`repair.rs:~1028-1031`), which currently documents the OLD behavior ("wrong-HRP … returns `IndelUnrecoverable` instead of the 'did you mean' suggestion") — now REVERSED: a wrong-HRP that fails indel search falls back to the original `HrpMismatch` suggestion. Audit the v0.37.x `CHANGELOG.md` entries for the same now-stale claim and correct any narrative (the manual exit-2 row stays accurate — same exit code).
- [ ] **Step 4 — run, expect PASS**; full test; clippy clean.
- [ ] **Step 5 — commit** `feat(indel): Phase 4 — HrpMismatch suggestion-fallback on Unrecoverable (+ doc-comment fix)`.

### Phase 5 — lockstep + release-prep (v0.37.3)
**Files:** `mnemonic-gui` (paired), `docs/manual/.../41-mnemonic.md`, `design/FOLLOWUPS.md`, `Cargo.toml`, `Cargo.lock`, both READMEs, `scripts/install.sh`, `CHANGELOG.md`.
- [ ] **Step 1 — manual mirror:** add the `--max-subst` flag row + a short "Recovering an indel that also has a wrong character" prose para; refine the exit-codes table row 4 to "ambiguous **or** a candidate required ≥1 substitution — verify before trusting"; document the `--json` `subst_count`/`confident` fields. Run `make -C docs/manual lint MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=…` (rebuild bin first; add cspell words if needed).
- [ ] **Step 2 — FOLLOWUPS:** flip `m-format-indel-cross-region-split`, `m-format-indel-plus-substitution`, `m-format-indel-hrpmismatch-suggestion-fallback` → `resolved` with `Resolution (v0.37.3)` notes.
- [ ] **Step 3 — version v0.37.3:** `Cargo.toml`; `cargo check` + **stage `Cargo.lock`**; both README `<!-- toolkit-version: -->` markers; `scripts/install.sh:32` pin `mnemonic-toolkit-v0.37.3`; `CHANGELOG.md` entry (SemVer PATCH; `--max-subst` candidate-list model + exit-4-verify; cross-region; HrpMismatch fallback; toolkit-only; erasure→8 still open; GUI v0.21.3 paired).
- [ ] **Step 4 — verify:** `cargo test -p mnemonic-toolkit` green (NO blanket `--include-ignored` — mlock G2 env-gated); clippy clean; manual lint green; `readme_version_current` green.
- [ ] **Step 5 — commit** `release(indel): v0.37.3 — cross-region + --max-subst + HrpMismatch fallback`.
- [ ] **Step 6 — GUI paired PR (post-tag):** `mnemonic-gui v0.21.3` — add `max-subst` to `REPAIR_FLAGS` (`FlagKind::Number{min:0, max:NumberMax::Static(4)}`), bump toolkit pin → v0.37.3; `schema_mirror` green. (Can only run after the toolkit tag.) **R0 M2:** write the new flag's comment as "ms1/mk1/md1" — do NOT copy the stale "ms1/mk1 only" phrasing from the existing `--max-indel` comment (`mnemonic-gui/src/schema/mnemonic.rs:~1562`, stale since md1 un-refused in v0.37.2).

## §3 Test corpus (consolidated — the §11 integration matrix)
1. ms1 indel+subst (j=1,e=1) → Unique, subst_count=1, exit 4 + WARNING (Phase 1 engine + Phase 3 CLI).
2. ms1 indel+subst with e_subst=0 → Unrecoverable (pure-indel rejects).
3. cross-region: prefix-drop + data-drop (N=2,E=0) → Unique, region CrossRegion, exit 5.
4. **all three:** prefix indel + data indel + data subst (N=2,E=1) → recovered, exit 4.
5. `--max-subst 0` regression at various N → byte-identical to v0.37.2 (exit 5/4-ambiguous/2).
6. substitution-bearing unique → exit 4 (not 5); pure-indel unique under E≥1 → exit 5.
7. over-budget (placeholders+subst > 4) → Unrecoverable.
8. genuine wrong-HRP + N=1,E=1 → original HrpMismatch suggestion (Phase 4).
9. recoverable prefix-drop still recovers (fallback doesn't fire).
10. `--json` `subst_count` + `confident:false`; `--max-subst 5` clap-rejected.
11. mk1 + md1 indel+subst single-chunk (Phase 1) + cross-region where applicable.
12. `indel_exit_code` unit cells (3-arg).

## §4 Risks
- (R1) `validate` signature change ripples to ALL `recover_indel(...)` call sites + the mock oracles in `indel.rs` tests + `repair.rs` tests — Phase 1 Step 3 must update them all to the 5-arg / `Option<(String,usize)>` form (pass `e_subst=0` to preserve pure-indel assertions). Trial `cargo build` early.
- (R2) Phase 2 restructure must preserve the v0.37.2 single-region behavior exactly (the regression cells 5/9 guard it). `prefix_restorations`/`data_variants` extraction is where a subtle off-by-one could change candidate generation — keep the window/levenshtein/`combinations` logic identical.
- (R3) FP: the candidate-list + verify-advisory + exit-4 contract is the safety net (spec §8/§9); no extra clamp needed (decoder caps at t=4).
- (R4) GUI is post-tag (schema_mirror runs against the pinned binary).

## §5 Reviewer-loop
This plan-doc faces mandatory opus R0 (0C/0I) BEFORE Phase 1. Per-phase reviews persist to `design/agent-reports/indel-v2-phase-N-review.md`. End-of-cycle review before tag.

## §6 Rn fold log
- **R0 (plan-doc):** RED 0C/2I/3M (`design/agent-reports/indel-v2-plan-R0-review.md`). Design confirmed sound (the three compose; Phase-4 ownership, exit invariant, gate logic all verified). Folded: **I1** exhaustive Phase-1 call-site inventory (5 `recover_indel_card` + 2 `md1_chunk_solve` test sites + the tuple-return `.as_deref()` adaptation; trial `cargo build --tests`); **I2** Phase-4 updates the `is_indel_trigger` doc-comment (+ CHANGELOG narrative) that documented the now-reversed wrong-HRP behavior; **M1** Phase-3 stderr no-op notice when `--max-subst≥1 && --max-indel==0`; **M3** pinned the `prefix_restorations` yield contract; **M2** Phase-5 GUI comment says "ms1/mk1/md1" (not the stale "ms1/mk1 only"). → R1.
- **R1 (plan-doc):** RED 0C/1I (`design/agent-reports/indel-v2-plan-R1-review.md`). I2/M1/M2/M3 RESOLVED; regression clean. **I1 still missed the `IndelCandidate` struct-literal field-completeness class** (the `subst_count` field breaks all literals; 4 test literals `indel.rs:292,298` + `cmd/repair.rs:368,374` unenumerated). Folded into Phase 1 Step 3. → R2 dispatched.

## §7 Next steps
R0 → fold → GREEN → Phases 1-5 (per-phase TDD + review) → end-of-cycle review → tag `mnemonic-toolkit-v0.37.3` → paired `mnemonic-gui v0.21.3`.
