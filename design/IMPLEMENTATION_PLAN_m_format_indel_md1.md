# md1 indel recovery (mirror mk1) Implementation Plan — v0.37.2

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`).

**Goal:** Un-refuse `md1` in `mnemonic repair --max-indel`, by mirroring the shipped mk1 per-chunk recovery onto the md side. Resolves FOLLOWUP `m-format-indel-md1-chunked`.

**Architecture:** Toolkit-only. md-codec already exposes everything needed (`bch::MD_REGULAR_CONST`, `bch::polymod_run`, `bch_decode::decode_regular_errors`, `chunk::reassemble(&[&str])`), and md's `GEN_REGULAR` is byte-identical to mk's (shared codex32 generator), so the toolkit's existing `polymod_residue` + `decode_regular_errors` machinery works for md1 once the md **target constant** is re-acquired. Add `MD_REGULAR_TARGET`, extend `CardKind::target_residue(Md1, Regular)`, mirror `mk1_chunk_solve`/`Mk1IndelOracle` as `md1_chunk_solve`/`Md1IndelOracle` (cross-chunk oracle = `md_codec::chunk::reassemble`), and wire the `recover_indel_card` Md1 arm.

**SemVer:** PATCH → toolkit `v0.37.2`. **No GUI change** (no flag-name change — `--max-indel` already exists; md1 just stops being refused → `schema_mirror` is flag-name-only, unaffected). Manual updates (drop "md1 not yet supported"); FOLLOWUP flips to resolved.

---

## §0 Context

- Resolves `m-format-indel-md1-chunked` (`design/FOLLOWUPS.md`, corrected framing).
- **Base SHA:** `origin/master` = `950d5ef` (post-v0.37.1). Re-grep all `:line` anchors when touching files.
- **Recon-confirmed facts (this session):**
  - md-codec PUBLIC: `bch::MD_REGULAR_CONST = 0x0815c07747a3392e7`, `bch::polymod_run`, `bch::hrp_expand`, `bch_decode::decode_regular_errors` (regular-only; md has no long code), `chunk::reassemble(strings: &[&str]) -> Result<Descriptor, Error>` (pub, `chunk.rs:305`).
  - md `GEN_REGULAR` == mk `GEN_REGULAR` (byte-identical; shared codex32 BCH(93,80,8) regular). Regular SHIFT/MASK shared too. ⇒ the toolkit's `polymod_residue(hrp, vals, target, Regular)` computes md1's residue correctly with the md target.
  - `chunk::reassemble` does NOT self-correct (unlike `mk_codec::decode` which self-corrects t≤4 unguarded) — it unwraps each chunk (codex32 checksum-verified via `unwrap_string`), parses headers, and validates cross-chunk consistency: shared `chunk_set_id`/`count`/`version`, complete `0..count` index set, no gaps (`chunk.rs:339-365`). So a candidate must be a VALID codeword (residue 0) before reassembly accepts it — exactly the mk1 pattern, minus the unguarded-self-correction concern.
  - Header (`version`/`chunked-flag`/`chunk_set_id`/`count`/`index`) is the leading data-payload bits → **BCH-protected** (covered by the checksum); not a special surface.
- **Current state:** `recover_indel_card` Md1 arm = `Err(ToolkitError::BadInput("repair --max-indel: indel recovery is not yet supported for chunked md1"))` (`repair.rs:1031`). `CardKind::target_residue(Md1, _) => None` (`repair.rs:79`).

## §1 Locked decisions

1. **Toolkit-only**, mirroring the v0.37.1 mk1 path. No sibling-codec change, no GUI change.
2. Re-acquire `MD_REGULAR_TARGET = md_codec::bch::MD_REGULAR_CONST` (mirrors `MK_REGULAR_TARGET = mk_codec::MK_REGULAR_CONST`, `repair.rs:41`).
3. `target_residue(Md1, BchCode::Regular) => Some(MD_REGULAR_TARGET)`; `(Md1, BchCode::Long) => None` (md is regular-only → Long correctly yields `UnsupportedCodeVariant`).
4. Cross-chunk validation oracle = `md_codec::chunk::reassemble(&refs)` (NOT a self-correcting decode).
5. Same engine, same `--max-indel` flag, same exit contract (0/5/4/2), same pure-indel ⊆ rule, same single-region + single-failing-chunk constraints as mk1.

## §2 Architecture (file inventory — all in `crates/mnemonic-toolkit/src/repair.rs`)

### §2.1 Re-acquire the md target
Near `MK_REGULAR_TARGET`/`MK_LONG_TARGET` (`repair.rs:39-41`):
```rust
pub(crate) const MD_REGULAR_TARGET: u128 = md_codec::bch::MD_REGULAR_CONST;
```
(md has no long code → no `MD_LONG_TARGET`.)

### §2.2 Extend `CardKind::target_residue` (`repair.rs:72-81`)
```rust
fn target_residue(self, code: BchCode) -> Option<u128> {
    match (self, code) {
        (Self::Mk1, BchCode::Regular) => Some(MK_REGULAR_TARGET),
        (Self::Mk1, BchCode::Long) => Some(MK_LONG_TARGET),
        (Self::Md1, BchCode::Regular) => Some(MD_REGULAR_TARGET), // NEW
        // ms1 still delegates to ms_codec; md1 long is undefined.
        (Self::Ms1, _) | (Self::Md1, BchCode::Long) => None,
    }
}
```
Update the doc comment (the old one says "Ms1 + Md1 never call this helper post-v0.23.0" — md1 now does, for the indel path).

### §2.3 `md1_chunk_solve` (mirror `mk1_chunk_solve`)
```rust
/// ⊆-gated single-chunk BCH solve for md1 — mirror of `mk1_chunk_solve`
/// (md is regular-only). Reuses the shared-codex32-generator machinery
/// (`polymod_residue` + `decode_regular_errors`) with the md target.
fn md1_chunk_solve(cand: &str, allowed: &BTreeSet<usize>) -> Option<String> {
    let (values, code) = parse_chunk(cand, 0, CardKind::Md1).ok()?;
    let target = CardKind::Md1.target_residue(code)?; // None for Long ⇒ reject
    let residue = polymod_residue("md", &values, target, code);
    if residue == 0 {
        return Some(encode_chunk("md", &values));
    }
    let (positions, mags) = match code {
        BchCode::Regular => decode_regular_errors(residue, values.len()),
        BchCode::Long => return None, // md has no long code
    }?;
    if !positions.iter().all(|p| allowed.contains(p)) { return None; }
    let mut corrected = values.clone();
    for (&p, &m) in positions.iter().zip(&mags) {
        if p >= corrected.len() { return None; }
        corrected[p] ^= m;
    }
    if polymod_residue("md", &corrected, target, code) != 0 { return None; }
    Some(encode_chunk("md", &corrected))
}
```

### §2.4 `Md1IndelOracle` (mirror `Mk1IndelOracle`; oracle = `reassemble`)
```rust
/// md1 oracle — ⊆-gated solve the single failing chunk, then confirm full-set
/// reassembly via md_codec::chunk::reassemble (which does NOT self-correct, so
/// the solved chunk must already be a valid codeword).
pub(crate) struct Md1IndelOracle {
    pub all_chunks: Vec<String>,
    pub failing_index: usize,
}
impl crate::indel::IndelOracle for Md1IndelOracle {
    fn validate(&self, cand: &str, allowed: &BTreeSet<usize>) -> Option<String> {
        let corrected_chunk = md1_chunk_solve(cand, allowed)?;
        let mut chunks = self.all_chunks.clone();
        chunks[self.failing_index] = corrected_chunk.clone();
        let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        match md_codec::chunk::reassemble(&refs) {
            Ok(_) => Some(corrected_chunk),
            Err(_) => None,
        }
    }
}
```

### §2.5 `recover_indel_card` Md1 arm (replace the refusal; mirror Mk1 arm)
```rust
CardKind::Md1 => {
    let failing: Vec<usize> = chunks.iter().enumerate()
        .filter(|(i, c)| repair_chunk_one(CardKind::Md1, *i, c).is_err())
        .map(|(i, _)| i)
        .collect();
    if failing.len() != 1 {
        return Ok(crate::indel::IndelOutcome::Unrecoverable);
    }
    let f = failing[0];
    let oracle = Md1IndelOracle { all_chunks: chunks.to_vec(), failing_index: f };
    Ok(crate::indel::recover_indel(&chunks[f], "md", max_indel, &oracle))
}
```
(`repair_chunk_one(Md1, i, c)` now works per-chunk because `target_residue(Md1, Regular)` is `Some`; it uses the shared-generator `polymod_residue` + `decode_regular_errors`.)

## §3 Phases

### Phase 1 — md1 recovery engine (mirror mk1)
**Files:** Modify `crates/mnemonic-toolkit/src/repair.rs`.
Fixture (verified-real, 3-chunk bip84 md1 card; assert reassembly precondition):
```rust
const MD1_C0: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const MD1_C1: &str = "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const MD1_C2: &str = "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";
```
- [ ] **Step 1 — failing tests** (`repair.rs` test mod): (a) `assert!(md_codec::chunk::reassemble(&[MD1_C0,MD1_C1,MD1_C2]).is_ok())` precondition. (b) corrupt ONLY MD1_C1 with one inserted data char → `recover_indel_card(CardKind::Md1, &[C0, bad_C1, C2], 1)` → `Unique` with `recovered == MD1_C1`. (c) too-short (drop a data char in C1) → `Unique`. (d) two chunks corrupted → `Unrecoverable`. (e) `md1_chunk_solve` ⊆ rejection: a chunk with a substitution outside the placeholder set → `None`.
- [ ] **Step 2 — run, expect FAIL** (`Md1IndelOracle`/`md1_chunk_solve` undefined; Md1 arm is the BadInput refusal): `cargo test -p mnemonic-toolkit --bins indel_md1 -v`.
- [ ] **Step 3 — implement** §2.1–§2.5. (`use std::collections::BTreeSet;` already imported.)
- [ ] **Step 4 — run, expect PASS**: `cargo test -p mnemonic-toolkit --bins` (all prior green + new). `cargo clippy --all-targets -- -D warnings` clean.
- [ ] **Step 5 — commit** `feat(indel): Phase 1 — md1 per-chunk recovery (mirror mk1; reassemble oracle)`.

### Phase 2 — CLI integration, manual, release-prep (v0.37.2)
**Files:** `tests/cli_indel.rs`, `docs/manual/src/40-cli-reference/41-mnemonic.md`, `design/FOLLOWUPS.md`, `Cargo.toml`, `Cargo.lock`, both READMEs, `scripts/install.sh`, `CHANGELOG.md`.
- [ ] **Step 1 — update the cli_indel md1 test (BEHAVIOR CHANGE).** The existing `md1 + --max-indel 1 → refusal exit 1` cell (`tests/cli_indel.rs`) must be REPLACED: md1 is no longer refused. New cell: `repair --md1 <MD1_C0> --md1 <ins_data(MD1_C1)> --md1 <MD1_C2> --max-indel 1` → exit 5, stdout contains MD1_C1. (Generate `bad_C1` with the same `ins_data` helper.) Re-run, expect PASS after Phase 1 already shipped the engine.
- [ ] **Step 2 — manual mirror.** In `docs/manual/src/40-cli-reference/41-mnemonic.md`: the `--max-indel` row + prose subsection currently say "ms1/mk1 only (md1 not yet supported)" / "`md1` (chunked) is not yet supported." → change to "ms1/mk1/md1" and drop the not-yet-supported clause. Run `make -C docs/manual lint MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=…` (rebuild mnemonic first).
- [ ] **Step 3 — FOLLOWUP flip.** `m-format-indel-md1-chunked` → add `- **Resolution (v0.37.2):** …` + `- **Status:** resolved`.
- [ ] **Step 4 — version v0.37.2.** `Cargo.toml`; `cargo check` + stage `Cargo.lock`; both README `<!-- toolkit-version: -->` markers; `scripts/install.sh:32` pin `mnemonic-toolkit-v0.37.2`; `CHANGELOG.md` new `[0.37.2]` entry (SemVer PATCH; md1 un-refused; mirror mk1; toolkit-only; no GUI change).
- [ ] **Step 5 — verify + commit.** `cargo test -p mnemonic-toolkit` green (NO blanket `--include-ignored` — the 2 mlock G2 fault-injection tests need their env); clippy clean; manual lint green; `readme_version_current` green. Commit `release(indel): v0.37.2 — md1 indel recovery + manual + FOLLOWUP flip`.

## §4 Test corpus
1. md_codec reassemble precondition on the 3-chunk fixture.
2. md1 single-failing too-long / too-short → Unique (recovered == original chunk).
3. md1 two-failing → Unrecoverable.
4. md1_chunk_solve ⊆ rejection (indel+substitution → None).
5. cli_indel md1 multi-chunk recovery exit 5 (replacing the old refusal cell).
6. regression: existing cli_indel / cli_repair / schema-mirror-relevant tests unchanged green.

## §5 Risks
- (R1) The shared-generator reuse: `polymod_residue("md", …, MD_REGULAR_TARGET, Regular)` + mk's `decode_regular_errors` for md1 — rests on GEN_REGULAR + SHIFT/MASK being identical (verified for GEN; SHIFT/MASK are codex32-regular-structural, shared). R0 to confirm SHIFT/MASK sharing; if any doubt, switch `md1_chunk_solve` to md-codec's own `bch::polymod_run`/`bch_decode::decode_regular_errors` (also pub) — equivalent, md-authoritative.
- (R2) `reassemble` requires checksum-valid chunks (via `unwrap_string`). `md1_chunk_solve` produces a residue-0 chunk, so reassembly accepts. Confirm `unwrap_string` checksum-verifies (else a non-valid candidate could reach reassembly).
- (R3) Behavior change to the existing md1-refusal cli test — must be updated, not left asserting the old refusal.

## §6 Reviewer-loop
This plan-doc faces mandatory opus R0 (0C/0I) BEFORE Phase 1. Per-phase reviews persist to `design/agent-reports/m-format-indel-md1-phase-N-review.md`. End-of-cycle review before tag.

## §7 Rn fold log
- _(R0: pending dispatch.)_

## §8 Next steps
R0 → fold → GREEN → Phase 1 → Phase 2 → end-of-cycle review → tag `mnemonic-toolkit-v0.37.2` (no GUI PR).
