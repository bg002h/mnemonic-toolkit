# mnemonic-toolkit-v0.29.0 Implementation Plan (Cycle 4 / Wave 3 SemVer-minor cliff + paired GUI)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `mnemonic-toolkit-v0.29.0` (SemVer-minor wire-shape cliff) + paired `mnemonic-gui-v0.14.0` (downstream wire-shape break). Closes 3 v0.28+ FOLLOWUPs: `pr-26-import-provenance-three-variant-cleanup` (2-variant split locked at P0) + `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` (tagged-enum conversion; SemVer-minor cliff) + `error-rs-retroactive-alphabetical-sort` (44 variants × 3 match blocks; ~132 arm reorders).

**Architecture:** Cross-repo lockstep cycle. Toolkit tag lands first (~minutes). GUI bumps pin + updates schema-mirror + tags. Closure-verification: GUI CI's `schema_mirror` gate GREEN against the new pin before declaring cycle closed (lagging-indicator per architect I4 / v0.27.2 historical case study).

**Tech Stack:** Rust + clap-derive + serde (tag-based enum serialization) + assert_cmd. `make audit` to confirm regression-free. `mnemonic gui-schema` to regen JSON for GUI consumer.

**Brainstorm spec:** `design/BRAINSTORM_v0_28_plus_residual_followups.md` § "Cycle 4 — `mnemonic-toolkit-v0.29.0`". P0 recon dossier: `design/cycle-4-p0-recon.md`.

**Source SHA at plan-write time:** `da122fb` (v0.28.7 close).

**P0 STRICT-GATE locks (per architect I1 + this plan):**
- Variant freeze: 44 total `ToolkitError` variants post-Cycle-3. Cycle 4 does NOT add new variants.
- Slug A 2-variant split locked: `Bsms(Option<BsmsAuditFields>)` → `BsmsTwoLine` + `BsmsSixLine(BsmsAuditFields)`. (P0 drift: FOLLOWUPS "3-variant" framing was stale; actual scope is 1-variant split into 2.)
- Slug B SemVer-minor cliff confirmed: tagged-enum conversion changes no-match JSON shape. `tests/fixtures/v0_27_0_envelopes/` cells convert to `#[ignore]` with SemVer rationale.
- Slug C arm-move estimate corrected: 44 variants × 3 exhaustive match blocks = ~132 arm reorders (FOLLOWUPS body's "~50+ × 4 = ~250" was overstated).
- GUI baseline: pin `mnemonic-toolkit-v0.28.4` → bump to `mnemonic-toolkit-v0.29.0`. Schema-mirror touch: xpub-search result-shape only (v0.28.5/6/7 had no CLI surface change).

**SemVer policy:** This is the v0.28.x → v0.29.0 MINOR cliff per the brainstorm SemVer policy. Driver: xpub-search wire-shape replacement (struct → tagged enum). Other 2 slugs (ImportProvenance, retroactive sort) are pure-internal refactors with no wire-shape impact, ride the cliff.

---

## File structure

### Source files modified (toolkit)

- `crates/mnemonic-toolkit/src/error.rs` — retroactive alphabetical sort of 44 variants + 3 cascade match blocks (Slug C).
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs` — split `ImportProvenance::Bsms(Option<_>)` → `BsmsTwoLine` + `BsmsSixLine(BsmsAuditFields)`; update accessor match blocks at L146, L160, L176+ (Slug A).
- `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:342` — update construction site `provenance: ImportProvenance::Bsms(audit)` to discriminate `BsmsTwoLine` vs `BsmsSixLine` based on line-count branch (Slug A).
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:1370 + :1731` — accessor calls — should compile cleanly since accessor was rewritten (Slug A).
- `crates/mnemonic-toolkit/src/cmd/xpub_search/path_of_xpub.rs` — convert `PathOfXpubResult` struct → tagged enum (Slug B).
- `crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs` — convert `PassphraseOfXpubResult` struct → tagged enum (Slug B).
- `crates/mnemonic-toolkit/src/cmd/xpub_search/account_of_descriptor.rs` — convert `AccountOfDescriptorResult` struct → tagged enum (Slug B).

### Test files modified (toolkit)

- `crates/mnemonic-toolkit/src/wallet_import/mod.rs:505, 519, 526, 546, 553` — update `ImportProvenance::Bsms(Some/None)` test references to `BsmsTwoLine` / `BsmsSixLine(_)` (Slug A).
- `crates/mnemonic-toolkit/tests/fixtures/v0_27_0_envelopes/*.json` — capture v0.28.0+ shape OR mark consuming cells `#[ignore]` with SemVer rationale (Slug B).
- `crates/mnemonic-toolkit/tests/cli_xpub_search*.rs` — update JSON assertion patterns for tagged enum (Slug B).

### Source files modified (mnemonic-gui)

- `mnemonic-gui/pinned-upstream.toml` — `[mnemonic].tag` v0.28.4 → v0.29.0.
- `mnemonic-gui/Cargo.toml` — workspace dep tag v0.28.4 → v0.29.0.
- `mnemonic-gui/src/schema/mnemonic.rs` — regenerate schema mirror from `mnemonic gui-schema`. Only xpub-search result-shape changes (other v0.28.5/6/7 changes had no CLI surface impact).
- `mnemonic-gui/CHANGELOG.md` — new v0.14.0 entry.

### Release tooling

- `crates/mnemonic-toolkit/Cargo.toml` — version 0.28.7 → 0.29.0.
- `CHANGELOG.md` — new v0.29.0 section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.28.7` → `mnemonic-toolkit-v0.29.0`.
- `design/FOLLOWUPS.md` — 3 Status flips.

---

## Tasks

### Task 1: Phase 1 — Cross-repo recon refresh (~15 min)

**Files:** none modified.

P0 already covered most of this; this task ratifies GUI baseline + checks for any cross-repo drift since P0 recon.

- [ ] **Step 1: Verify GUI repo state**

```bash
cd /scratch/code/shibboleth/mnemonic-gui
git fetch --quiet origin
git status -sb
git log --oneline origin/master ^HEAD 2>/dev/null | head -5
```

Expected: GUI repo on master, clean, in sync with origin OR pull-forward.

- [ ] **Step 2: Re-verify GUI pin + schema-mirror**

```bash
grep -n "mnemonic-toolkit" /scratch/code/shibboleth/mnemonic-gui/pinned-upstream.toml
grep -n "mnemonic-toolkit" /scratch/code/shibboleth/mnemonic-gui/Cargo.toml | head -5
wc -l /scratch/code/shibboleth/mnemonic-gui/src/schema/mnemonic.rs
```

Pin both files at `mnemonic-toolkit-v0.28.4` per P0 recon. Schema-mirror at 2484 lines.

- [ ] **Step 3: Check GUI's latest tag**

```bash
cd /scratch/code/shibboleth/mnemonic-gui && git tag --sort=-creatordate | head -3
```

Latest expected: `mnemonic-gui-v0.13.0` (per P0 drift; my memory was stale at v0.10.0).

- [ ] **Step 4: Confirm schema_mirror integration test still references the toolkit pin**

```bash
grep -rn "schema_mirror" /scratch/code/shibboleth/mnemonic-gui/tests/ | head -5
```

The test fires via toolkit binary's `gui-schema` JSON output; ensure it's binary-pin-aware.

---

### Task 2: Phase 2 — Slug A `ImportProvenance` 2-variant split

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/mod.rs`
- Modify: `crates/mnemonic-toolkit/src/wallet_import/bsms.rs`
- Modify: `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs` (and possibly other test files that pattern-match on `ImportProvenance::Bsms`)

- [ ] **Step 1: Split the enum variant**

In `wallet_import/mod.rs:69-82` (enum block):

OLD:
```rust
pub(crate) enum ImportProvenance {
    BitcoinCore(CoreSourceMetadata),
    Bsms(Option<BsmsAuditFields>),
    Coldcard(coldcard::ColdcardSourceMetadata),
    // ... other 5 variants
}
```

NEW (alphabetical order — `BsmsSixLine` and `BsmsTwoLine` insert alphabetically after `BitcoinCore`):
```rust
pub(crate) enum ImportProvenance {
    BitcoinCore(CoreSourceMetadata),
    BsmsSixLine(BsmsAuditFields),
    BsmsTwoLine,
    Coldcard(coldcard::ColdcardSourceMetadata),
    // ... other 5 variants
}
```

Note: alphabetical `BsmsSixLine` < `BsmsTwoLine` (`S` < `T`).

- [ ] **Step 2: Update `bsms_audit()` accessor at `mod.rs:146`**

The accessor returns `Option<&BsmsAuditFields>`. OLD pattern matched `Bsms(audit) => audit.as_ref()`. NEW:

```rust
pub(crate) fn bsms_audit(&self) -> Option<&BsmsAuditFields> {
    match self {
        ImportProvenance::BsmsSixLine(audit) => Some(audit),
        ImportProvenance::BsmsTwoLine => None,
        _ => None,
    }
}
```

Verify exact accessor signature at `mod.rs:146` before substituting — adjust if return type differs.

- [ ] **Step 3: Update `source_metadata()` accessor at `mod.rs:160` + other per-variant accessors**

Each accessor that previously had a `Bsms(_) =>` arm now needs two arms: `BsmsSixLine(_) =>` and `BsmsTwoLine =>`. Most accessors should return `None` for both BSMS variants (BSMS doesn't carry per-variant source-metadata).

- [ ] **Step 4: Update construction site at `bsms.rs:342`**

OLD: `provenance: ImportProvenance::Bsms(audit),`

NEW (discriminate based on line-count branch):
```rust
provenance: match audit {
    Some(audit) => ImportProvenance::BsmsSixLine(audit),
    None => ImportProvenance::BsmsTwoLine,
},
```

OR — simpler if the 2-line vs 6-line distinction is captured elsewhere in the parse flow (likely there's a `parsed_line_count: u8` or similar local variable). Implementer reads context at `bsms.rs:342` and chooses the cleanest discrimination.

- [ ] **Step 5: Update test cells using `Bsms(Some/None)`**

Per P0 recon, the following lines in `mod.rs` use `ImportProvenance::Bsms(Some(...))` or `Bsms(None)`:
- L505, L519, L526, L546, L553

Substitute each:
- `Bsms(None)` → `BsmsTwoLine`
- `Bsms(Some(audit))` → `BsmsSixLine(audit)`

Also grep for other consumers:
```bash
grep -rn "ImportProvenance::Bsms" crates/mnemonic-toolkit/
```

Update all matches.

- [ ] **Step 6: Run BSMS test files + provenance tests**

```bash
cargo test --package mnemonic-toolkit --test cli_import_wallet_bsms 2>&1 | tail -10
cargo test --package mnemonic-toolkit wallet_import::mod::tests 2>&1 | tail -10
```

Expected: all passing.

---

### Task 3: Phase 3 — Slug B xpub-search result tagged-enum conversion (SemVer-minor wire-shape break)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/xpub_search/path_of_xpub.rs`
- Modify: `crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs`
- Modify: `crates/mnemonic-toolkit/src/cmd/xpub_search/account_of_descriptor.rs`
- Modify: `crates/mnemonic-toolkit/tests/cli_xpub_search*.rs`
- Possibly modify: `crates/mnemonic-toolkit/tests/fixtures/v0_27_0_envelopes/` cells (mark `#[ignore]`).

This is the load-bearing SemVer-minor break. Per the P0 lock, conversion is `#[serde(tag = "kind")]`.

- [ ] **Step 1: Read all 3 current struct definitions + their JSON-emitting cells**

```bash
sed -n '140,170p' crates/mnemonic-toolkit/src/cmd/xpub_search/path_of_xpub.rs
sed -n '160,200p' crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs
sed -n '150,180p' crates/mnemonic-toolkit/src/cmd/xpub_search/account_of_descriptor.rs
```

Note: each currently is a `struct` with a `result: &'static str` discriminant + optional fields that are `None` on no-match.

- [ ] **Step 2: Convert `PathOfXpubResult` (path_of_xpub.rs:144)**

OLD:
```rust
#[derive(Serialize)]
pub struct PathOfXpubResult {
    pub result: &'static str,
    pub path: Option<String>,
    pub template: Option<String>,
    pub account: Option<u32>,
    pub target_xpub_canonical: String,
    pub target_xpub_variant: Option<&'static str>,
    pub searched_count: usize,
}
```

NEW:
```rust
#[derive(Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum PathOfXpubResult {
    Match {
        path: String,
        template: String,
        account: Option<u32>,
        target_xpub_canonical: String,
        target_xpub_variant: Option<&'static str>,
        searched_count: usize,
    },
    NoMatch {
        target_xpub_canonical: String,
        target_xpub_variant: Option<&'static str>,
        searched_count: usize,
    },
}
```

The variant tag (`result` field) holds the value of the variant name (snake_case: `"match"` / `"no_match"`).

Update all constructors of `PathOfXpubResult { ... }` to use the appropriate variant: `PathOfXpubResult::Match { path: ..., template: ..., ... }` or `PathOfXpubResult::NoMatch { ... }`.

- [ ] **Step 3: Convert `PassphraseOfXpubResult` (passphrase_of_xpub.rs:169)**

Same pattern as Step 2 — identical field set. Use `#[serde(tag = "result", rename_all = "snake_case")]` + `Match`/`NoMatch` variants.

- [ ] **Step 4: Convert `AccountOfDescriptorResult` (account_of_descriptor.rs:155)**

OLD:
```rust
#[derive(Serialize)]
pub struct AccountOfDescriptorResult {
    pub result: &'static str,
    pub matched_cosigners: Vec<MatchedCosignerJson>,
    pub cosigners_total: usize,
    pub searched_count_per_cosigner: usize,
    pub descriptor_shape: DescriptorShape,
    pub unspendable_internal_keys: Vec<usize>,
}
```

NEW:
```rust
#[derive(Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum AccountOfDescriptorResult {
    Match {
        matched_cosigners: Vec<MatchedCosignerJson>,
        cosigners_total: usize,
        searched_count_per_cosigner: usize,
        descriptor_shape: DescriptorShape,
        unspendable_internal_keys: Vec<usize>,
    },
    NoMatch {
        cosigners_total: usize,
        searched_count_per_cosigner: usize,
        descriptor_shape: DescriptorShape,
        unspendable_internal_keys: Vec<usize>,
    },
}
```

- [ ] **Step 5: Update CLI emitters that build these structs**

Each `cmd/xpub_search/<file>.rs` has logic that builds the result then `serde_json::to_writer` emits it. Update construction sites to use the new tagged-enum variants.

- [ ] **Step 6: Update test assertions**

In `tests/cli_xpub_search*.rs`, test assertions that read `result["path"].is_null()` (or similar null-on-no-match patterns) need updating:
- Pre-conversion: `assert!(out["path"].is_null());`
- Post-conversion: `assert!(!out.as_object().unwrap().contains_key("path"));` OR `assert_eq!(out["result"], "no_match");` (discriminator-based).

Grep for `.is_null()` and `result["path"]` etc. across test files:
```bash
grep -rn '\.is_null()\|result\["path"\]\|result\["template"\]' crates/mnemonic-toolkit/tests/cli_xpub_search*.rs
```

Update each cell.

- [ ] **Step 7: Handle v0.27.0 envelope fixtures**

The `tests/fixtures/v0_27_0_envelopes/` directory may carry fixtures that capture the old JSON wire shape. Two options per FOLLOWUP body:
- **Capture new shape:** add v0.29.0-shape fixtures alongside (parallel fixtures).
- **Mark old cells `#[ignore]`:** add `#[ignore = "v0.27.0 envelope shape; SemVer-minor cliff at v0.29.0 — see FOLLOWUP xpub-search-result-type-level-invariant"]` to consuming test cells.

**Recommended:** mark old cells `#[ignore]` with rationale comment (parallel fixtures are maintenance burden). Implementer enumerates affected cells via:
```bash
grep -rn "v0_27_0_envelopes" crates/mnemonic-toolkit/tests/ | head -10
```

- [ ] **Step 8: Run xpub-search tests**

```bash
cargo test --package mnemonic-toolkit --test cli_xpub_search 2>&1 | tail -10
cargo test --package mnemonic-toolkit --tests xpub_search 2>&1 | tail -10
```

Expected: passing. Any failures should be in v0.27.0-envelope cells that need `#[ignore]`.

---

### Task 4: Phase 4 — Slug C error.rs retroactive alphabetical sort

**Files:**
- Modify: `crates/mnemonic-toolkit/src/error.rs`

This is mechanical: reorder 44 variants in the enum declaration + 3 cascade match blocks (Display / exit_code / kind / details). Pure reorder, no semantic change. Per CLAUDE.md, can land in same commit IF sonnet verifies zero-semantic-drift.

- [ ] **Step 1: Author alphabetical variant list**

Use the variant list from `design/cycle-4-p0-recon.md` Part 1 as input. Alphabetical sort yields:

```
1.  BadInput
2.  Bip39
3.  Bip388Distinctness
4.  Bip388VerifyDistinctness
5.  Bitcoin
6.  BsmsRound1Malformed
7.  BsmsSignatureMismatch
8.  BsmsTaprootImportRefused
9.  BsmsTaprootRefused
10. BundleMismatch
11. CompareCost
12. ConvertRefusal
13. CosignerSpec
14. CosignersFile
15. DeriveChildLengthNotApplicable
16. DeriveChildLengthOutOfRange
17. DeriveChildUnsupportedApp
18. DescriptorParse
19. DescriptorReparseFailed
20. EnvVarMissing
21. ExportWalletFormatStub
22. ExportWalletMissingFields
23. ExportWalletSecretInput
24. ExportWalletTaprootMultisigUnsupported
25. FutureFormat
26. HrpMismatch
27. ImportWalletAmbiguousFormat
28. ImportWalletFormatMismatch
29. ImportWalletParse
30. ImportWalletSeedMismatch
31. ImportWalletWatchOnlyViolation
32. ImportWalletXprvForbidden
33. Io
34. MdCodec
35. MkCodec
36. ModeViolation
37. MsCodec
38. MultisigConfig
39. NetworkMismatch
40. Repair
41. RepairShortCircuit
42. SlotInputViolation
43. UnknownHrp
44. XpubSearchNoMatch
```

- [ ] **Step 2: Reorder variant declarations in enum block**

Move each variant in the `enum ToolkitError` block to alphabetical position. Preserve all variant fields, doc-comments, and attributes verbatim. Pure cut-and-paste.

- [ ] **Step 3: Reorder `Display` match arms (L542-711)**

Same order as variant declaration. Each arm: `ToolkitError::VariantName { ... } => write!(f, "...")` — move bodily.

- [ ] **Step 4: Reorder `exit_code` match arms (L428-482) — LOCK: arms become single-variant post-sort (R0-I2 fold)**

Alphabetical sort INTERLEAVES different-exit variants. Pre-sort, the file groups same-exit variants via `|` (e.g., 18 variants `=> 2`). Post-sort, these groupings MUST break because intervening different-exit variants will sit between them. **Lock: all post-sort `exit_code` arms are single-variant `ToolkitError::Foo => N,`** (one variant per arm). NEW FOLLOWUP `error-rs-exit-code-arm-fragmentation-post-sort` filed for a future readability-vs-mechanicalness pass.

Implementer authors the 44 single-variant arms in alphabetical order. Verify exit codes against existing grouped patterns to preserve semantics.

- [ ] **Step 5: Reorder `kind` match arms (L489-536)**

Same order.

- [ ] **Step 6: Reorder `details` partial-match arms (L718-742)**

Only the 7 named arms (not the wildcard `_ => None`).

- [ ] **Step 7: Build + run sort-only regression**

```bash
cargo build --bin mnemonic 2>&1 | tail -5
cargo test --package mnemonic-toolkit --tests 2>&1 | grep -E '^test result:' | grep -v ' 0 failed'
```

Expected: clean build; ZERO failing tests (pure reorder is semantic identity).

---

### Task 5: Phase 5 — Full test + clippy

- [ ] **Step 1: Full toolkit test suite**

```bash
cargo test --package mnemonic-toolkit --tests 2>&1 | grep -E '^test result:' | sed 's/.*ok\. //; s/ passed.*//' | awk '{s+=$1} END {print "Total passing:", s}'
```

Expected: 2028 + delta from Slugs A/B (some cells `#[ignore]`-gated; net delta likely close to 0 or slightly negative due to v0.27.0 envelope ignores).

- [ ] **Step 2: Clippy**

```bash
cargo clippy --package mnemonic-toolkit --tests -- -D warnings 2>&1 | tail -3
```

Expected: clean.

- [ ] **Step 3: `make audit`**

```bash
make -C docs/manual audit \
  MNEMONIC_BIN=$PWD/target/debug/mnemonic \
  MD_BIN=$(which md) MS_BIN=$(which ms) MK_BIN=$(which mk) \
  FIXTURES_DIR=$PWD/crates/mnemonic-toolkit/tests/fixtures/wallet_import \
  2>&1 | tail -5
```

Expected: `[lint] OK` + `[verify-examples] OK`. No transcript impact from any Cycle 4 changes (xpub-search JSON shape isn't exercised in transcripts).

---

### Task 6: Phase 6 — Regen `mnemonic gui-schema` JSON

- [ ] **Step 1: Run gui-schema with new binary**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
target/debug/mnemonic gui-schema > /tmp/gui-schema-v0.29.0.json
head -50 /tmp/gui-schema-v0.29.0.json
wc -l /tmp/gui-schema-v0.29.0.json
```

Expected: JSON output reflecting the new tagged-enum xpub-search result shape.

- [ ] **Step 2: Diff against v0.28.7 baseline**

Use git stash + rebuild the v0.28.7 binary OR pull schema from a tag-checkout to compare. Document the diff between v0.28.7 schema and v0.29.0 schema.

- [ ] **Step 3: Identify the xpub-search result shape changes in JSON**

The output should now have entries like:

```json
{
  "xpub-search": {
    "result_type": {
      "kind": "tagged_enum",
      "variants": {
        "match": { ... },
        "no_match": { ... }
      }
    }
  }
}
```

Or analogous shape. Confirm the GUI schema-mirror will need to adapt to this new wire-shape on its consumer side.

---

### Task 7: Phase 7 — GUI lockstep (pin bump + CHANGELOG + Cargo.toml version)

**R0-I1 + R0-I4 fold:** The GUI's `schema_mirror.rs` test enforces **flag-name set parity** (clap surface), NOT JSON wire-shape. Since Slugs A+B don't modify any clap `#[arg]` declarations or dropdown-value enums, `src/schema/mnemonic.rs` likely needs **zero edits beyond pin bump**. The v0.29.0 binary's `gui-schema` JSON output should emit the same flag-name set as v0.28.4. Wire-shape consumers of xpub-search JSON have NO automated drift gate (real gap; filed as NEW FOLLOWUP `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`).

**ORDERING LOCK (R0-I4 fold):** ENTIRE Task 7 is gated behind Task 9 Steps 1-5 completion (toolkit tag must exist on origin before GUI dep can resolve to it). Do NOT execute any step of Task 7 until the toolkit tag push lands.

**Files (mnemonic-gui repo):**
- Modify: `mnemonic-gui/pinned-upstream.toml`
- Modify: `mnemonic-gui/Cargo.toml`
- Maybe modify: `mnemonic-gui/src/schema/mnemonic.rs` (ONLY if `gui-schema` JSON diff shows clap-surface drift; expected: no drift)
- Modify: `mnemonic-gui/CHANGELOG.md`

- [ ] **Step 1: Update pin to v0.29.0**

(Execute AFTER Task 9 Steps 1-5 land toolkit tag.)

```bash
cd /scratch/code/shibboleth/mnemonic-gui
sed -i 's/mnemonic-toolkit-v0.28.4/mnemonic-toolkit-v0.29.0/g' pinned-upstream.toml
sed -i 's/tag = "mnemonic-toolkit-v0.28.4"/tag = "mnemonic-toolkit-v0.29.0"/g' Cargo.toml
cargo update -p mnemonic-toolkit  # refresh Cargo.lock to point at new tag
```

- [ ] **Step 2: Diff `gui-schema` JSON — check for clap-surface drift (expected: none)**

```bash
# Install the v0.29.0 binary locally for the diff
cargo install --git https://github.com/bg002h/mnemonic-toolkit --tag mnemonic-toolkit-v0.29.0 mnemonic-toolkit
# Or use the just-built binary from the toolkit repo:
/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic gui-schema > /tmp/gui-schema-v0.29.0.json
# Compare against the v0.28.4-era schema baseline (whichever your CI used)
diff <(jq --sort-keys . /tmp/gui-schema-v0.28.4.json) <(jq --sort-keys . /tmp/gui-schema-v0.29.0.json)
```

Expected: no diff (or only `version` field changed). If a flag-name or dropdown-value drifted unexpectedly, fold + update `src/schema/mnemonic.rs`. **If diff is empty, skip Step 3.**

- [ ] **Step 3 (CONDITIONAL): Update `src/schema/mnemonic.rs` ONLY if Step 2 diff shows drift**

Edit only the affected `SubcommandSchema` entries. The `schema_mirror` test will fail loudly post-Task-10 CI if any hand-maintained entry doesn't match the binary's JSON output.

- [ ] **Step 4: GUI CHANGELOG entry**

In `mnemonic-gui/CHANGELOG.md`, add:

```markdown
## mnemonic-gui [0.14.0] — 2026-05-21

SemVer-minor: lockstep with `mnemonic-toolkit-v0.29.0`'s xpub-search result tagged-enum conversion (FOLLOWUP `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution`). Downstream JSON wire-shape break for any GUI consumer of `mnemonic xpub-search --json` output.

### Lockstep

- Toolkit pin bumped `mnemonic-toolkit-v0.28.4` → `mnemonic-toolkit-v0.29.0` (4 patch + 1 minor toolkit release).
- Schema-mirror at `src/schema/mnemonic.rs`: no edit (clap surface unchanged across all 4 toolkit releases).

### Note

The GUI's `schema_mirror` integration test enforces clap flag-name parity, not JSON wire-shape — GUI's runtime consumers of `xpub-search --json` output are responsible for handling the new `Match` / `NoMatch` tagged-enum result shape on their own (no automated drift gate; tracked at toolkit FOLLOWUP `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`).

---
```

- [ ] **Step 5: Bump GUI Cargo.toml version**

`mnemonic-gui/Cargo.toml` workspace `[package].version`: `0.13.0` → `0.14.0`.

---

### Task 8: Phase 8 — Opus end-of-cycle reviewer (cross-repo)

- [ ] **Step 1: Dispatch opus via Agent tool**

Use `Agent`:
- `subagent_type: feature-dev:code-reviewer`
- `model: opus`
- Cross-repo scope: review BOTH `/scratch/code/shibboleth/mnemonic-toolkit/` working tree AND `/scratch/code/shibboleth/mnemonic-gui/` working tree.

Prompt verifies:
1. Slug A: 2-variant split shape correct; all accessor/construction sites updated; alphabetical position (`BsmsSixLine` < `BsmsTwoLine`).
2. Slug B: tagged-enum conversion correct on all 3 result types; v0.27.0 fixture cells `#[ignore]`-gated with SemVer rationale; CLI JSON output emits `"result": "match"` / `"result": "no_match"` discriminator.
3. Slug C: retroactive sort produced zero semantic-drift; all 132 arm moves are pure reorder; no missing arms.
4. GUI lockstep: pin bumped both files; schema-mirror reflects new xpub-search shape; GUI Cargo.toml version bumped.
5. SemVer-minor cliff documented in CHANGELOG; install.sh:32 bumped.
6. ToolkitError variant count: 44 (no new variants in Cycle 4 — Variant Freeze honored).
7. All test cells pass OR are `#[ignore]`-gated with SemVer rationale.
8. Clippy clean.

Gate: 0 critical / 0 important.

- [ ] **Step 2: Persist opus review verbatim**

Save to `design/agent-reports/v0_29_0-phase-8-end-of-cycle-review.md` BEFORE applying any folds.

- [ ] **Step 3: Fold any Important findings inline**

---

### Task 9: Phase 9 — Paired commits + tags (toolkit-first then GUI)

**ORDER MATTERS:** Toolkit lands first. GUI can't bump pin until toolkit tag exists on origin.

**R0-I3 fold:** Split toolkit ship into 2 commits on the same branch for bisect hygiene — `refactor(error): retroactive alphabetical sort` (Slug C only; pure mechanical reorder; sonnet diff-verify single concern) THEN `release(toolkit): v0.29.0 — Slug A + Slug B + version bump`. Same tag on the second commit.

- [ ] **Step 1: Toolkit working-tree verify**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git status --short
```

Expected modified files: see "File structure" §toolkit above.

- [ ] **Step 2: First commit — Slug C sort-only**

Stage ONLY `error.rs` changes:

```bash
git add crates/mnemonic-toolkit/src/error.rs
git commit -m "refactor(error): retroactive alphabetical sort of ToolkitError variants + cascade match blocks

Pure reorder, no semantic change. 44 variants sorted alphabetically;
~132 arm reorders across Display, exit_code, kind exhaustive match
blocks (plus details partial-match). exit_code multi-variant grouped
patterns broken into single-variant arms post-sort (R0-I2 lock;
re-grouping tracked at FOLLOWUP error-rs-exit-code-arm-fragmentation-
post-sort).

Closes FOLLOWUP error-rs-retroactive-alphabetical-sort.

Cycle 4 of v0.28+ residual FOLLOWUP release plan (Wave 3). Part 1
of 2 toolkit commits; v0.29.0 tag lands on the second commit."
```

- [ ] **Step 3: Sonnet diff-verify zero-semantic-drift on Slug C commit**

```bash
git show HEAD --stat
# Dispatch sonnet diff-verifier:
# Inputs: git show HEAD output
# Verification: every removed arm body appears verbatim in some added arm with the same variant name; no new variants; no removed variants; ToolkitError fields preserved; no body changes.
```

If sonnet flags any arm-body diff, fold inline; re-commit (amending OR new commit).

- [ ] **Step 4: Second commit — Slug A + Slug B + version bump**

Stage remaining explicit paths:

```bash
git add crates/mnemonic-toolkit/src/wallet_import/mod.rs \
        crates/mnemonic-toolkit/src/wallet_import/bsms.rs \
        crates/mnemonic-toolkit/src/cmd/xpub_search/path_of_xpub.rs \
        crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs \
        crates/mnemonic-toolkit/src/cmd/xpub_search/account_of_descriptor.rs \
        # any other modified files (test cells, fixtures) per Slugs A+B \
        crates/mnemonic-toolkit/Cargo.toml \
        Cargo.lock \
        CHANGELOG.md \
        scripts/install.sh \
        design/FOLLOWUPS.md \
        design/PLAN_mnemonic_toolkit_v0_29_0.md \
        design/cycle-4-p0-recon.md \
        design/agent-reports/v0_29_0-*.md
git commit -m "release(toolkit): mnemonic-toolkit v0.29.0 — SemVer-minor cliff (xpub-search wire-shape + ImportProvenance 2-variant split) ..."
SHA=$(git rev-parse HEAD)
sed -i "s/resolved <PLACEHOLDER-COMMIT-SHA>/resolved $SHA/g" design/FOLLOWUPS.md
git add design/FOLLOWUPS.md
git commit --amend --no-edit
git tag mnemonic-toolkit-v0.29.0
git push origin master
git push origin mnemonic-toolkit-v0.29.0
```

- [ ] **Step 5: Wait for toolkit install-pin-check CI green**

```bash
sleep 30  # let CI fire
gh run list --workflow=install-pin-check.yml --limit 1 --json status,conclusion,headBranch
```

Expected: `mnemonic-toolkit-v0.29.0` head branch with conclusion=success.

- [ ] **Step 6: Create toolkit GH Release**

```bash
gh release create mnemonic-toolkit-v0.29.0 \
  --title 'mnemonic-toolkit v0.29.0 — SemVer-minor (xpub-search wire-shape break + ImportProvenance split + error.rs retroactive sort)' \
  --notes "..."
```

- [ ] **Step 7: NOW execute Task 7 (GUI lockstep) — toolkit tag is on origin**

After this step lands, the GUI dep can resolve to v0.29.0. Return to Task 7 Step 1 and execute.

- [ ] **Step 8: GUI commit + tag + push**

```bash
cd /scratch/code/shibboleth/mnemonic-gui
git add pinned-upstream.toml Cargo.toml Cargo.lock CHANGELOG.md
# (src/schema/mnemonic.rs only if Task 7 Step 2 diff showed drift)
git commit -m "release(gui): mnemonic-gui v0.14.0 — lockstep with mnemonic-toolkit-v0.29.0 ..."
git tag mnemonic-gui-v0.14.0
git push origin master
git push origin mnemonic-gui-v0.14.0
```

- [ ] **Step 9: Create GUI GH Release**

---

### Task 10: Phase 10 — Closure-verification (GUI CI schema_mirror gate)

This is the lagging-indicator gate per architect I4. Without it, drift accumulates silently.

- [ ] **Step 1: Wait for GUI CI to fire on the GUI tag**

```bash
sleep 60
cd /scratch/code/shibboleth/mnemonic-gui
gh run list --workflow=ci.yml --limit 2 --json status,conclusion,headBranch
```

- [ ] **Step 2: Verify `schema_mirror` integration test GREEN**

The schema_mirror test runs `mnemonic gui-schema` against the v0.29.0 binary + compares against `src/schema/mnemonic.rs` hand-maintained mirror. If the mirror update in Task 7 Step 2 was correct, this test passes. If not, GUI CI fails → file as bugfix patch.

```bash
gh run view --log <run-id> | grep -A 5 'schema_mirror'
```

- [ ] **Step 3: Declare Cycle 4 closed**

Closure criteria:
- Toolkit tag `mnemonic-toolkit-v0.29.0` pushed ✓
- GUI tag `mnemonic-gui-v0.14.0` pushed ✓
- Toolkit install-pin-check CI GREEN on tag ✓
- GUI schema_mirror CI GREEN against new pin ✓
- 3 FOLLOWUPs Status flipped resolved ✓

If schema_mirror CI fails: file bugfix patch (v0.29.1 toolkit OR v0.14.1 GUI depending on which side is wrong).

---

### Task 11: Phase 11 — FOLLOWUPS Status flips × 3

(Already wired into Task 9 Step 3's sed-amend, but ensure all 3 slugs flipped:)

- [ ] **Step 1: Verify 3 Status flips**

For each:
- `pr-26-import-provenance-three-variant-cleanup` → `resolved <SHA>` with note about 2-variant lock per P0
- `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` → `resolved <SHA>` with SemVer-minor cliff note
- `error-rs-retroactive-alphabetical-sort` → `resolved <SHA>` with arm-move count

---

## Self-review

### Spec coverage

- P0 STRICT-GATE (variant freeze + 3 slug recon + GUI baseline) → DONE (`design/cycle-4-p0-recon.md`)
- Phase 1 cross-repo recon refresh → Task 1
- Phase 2 ImportProvenance 2-variant split → Task 2
- Phase 3 xpub-search tagged-enum conversion → Task 3
- Phase 4 error.rs retroactive sort → Task 4
- Phase 5 cargo test + clippy + audit → Task 5
- Phase 6 gui-schema JSON regen → Task 6
- Phase 7 GUI lockstep (pin + schema-mirror + CHANGELOG) → Task 7
- Phase 8 opus end-of-cycle review (cross-repo) → Task 8
- Phase 9 paired commits + tags → Task 9
- Phase 10 closure-verification → Task 10
- Phase 11 FOLLOWUPS Status flips → Task 11

### Placeholder scan

- Test-cell ignores: implementer enumerates v0.27.0 envelope fixture cells via grep (Task 3 Step 7); not pre-listed.
- Commit + GH Release notes: full text written at execution time (Task 9 Steps 3, 5).

### Type consistency

- `ImportProvenance` variant rename: `Bsms(Option<_>)` → `BsmsTwoLine` (unit) + `BsmsSixLine(BsmsAuditFields)` (newtype). Used consistently in Tasks 2 + 8.
- `PathOfXpubResult` / `PassphraseOfXpubResult` / `AccountOfDescriptorResult` — all 3 enum conversions with `#[serde(tag = "result", rename_all = "snake_case")]`.
- `ToolkitError` — 44 variants unchanged in semantics; only reordered.

### Effort estimate sanity-check

Per brainstorm: ~2-3 days. Per-task:
- Task 1 (cross-repo recon refresh): ~15 min
- Task 2 (Slug A 2-variant split): ~2 hours (8 accessor updates + 5 test references + 1 construction site)
- Task 3 (Slug B tagged-enum cliff): ~4-5 hours (3 struct→enum conversions + emitter updates + test assertion rewrites + v0.27.0 envelope cell triage)
- Task 4 (Slug C retroactive sort): ~3-4 hours (44 variant moves + 132 arm moves; pure reorder but tedious)
- Task 5 (test + clippy + audit): ~30 min
- Task 6 (gui-schema regen + diff): ~30 min
- Task 7 (GUI lockstep): ~1-2 hours (pin bumps + schema-mirror diff + CHANGELOG)
- Task 8 (opus reviewer + fold): ~1 hour
- Task 9 (paired commits + tags + GH Releases): ~30 min
- Task 10 (closure-verification): ~30 min (CI wait + verify)
- Task 11 (Status flips): folded into Task 9

**Total: ~13-16 hours = ~2 days.** In brainstorm range.

---

## Risk flags

- **Slug C arm-move regression risk** — 132 arm moves is mechanical but error-prone. Mitigation: sonnet code-quality reviewer dispatched on commit-diff to verify zero-semantic-drift. If sonnet flags any arm-body difference, fold inline.

- **Slug B JSON shape compatibility** — `#[serde(tag = "result", rename_all = "snake_case")]` produces `"result": "match"` / `"result": "no_match"` — matches the OLD discriminator field name. Cleaner than introducing `"kind"` since downstream consumers already check the `result` field name. But: it's still a wire-shape break because optional fields move from `null` to absent-key. Cannot be backwards-compatible.

- **GUI schema-mirror scope is flag-name parity, NOT wire-shape (R0-I1 corrected)** — The `schema_mirror` test enforces clap flag-name set parity between hand-maintained `SubcommandSchema` (in `src/schema/mnemonic.rs`) and `gui-schema` JSON output. It does NOT gate JSON wire-shape (e.g., xpub-search result-shape). Since Slugs A+B don't modify clap declarations, the mirror file likely needs zero edits beyond pin bump. NEW FOLLOWUP `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` filed: GUI's runtime consumers of xpub-search JSON have NO automated drift gate.

- **Cross-repo tag ordering** — toolkit MUST be pushed first; GUI dep must resolve to the toolkit tag. Mitigation: Task 9 Step 6 explicitly orders Steps 1-4 before Step 6.

- **install-pin-check CI on toolkit tag** — verifies `scripts/install.sh:32` matches the tag value. Mitigation: Task 7 release tooling.

- **v0.27.0 envelope fixture triage** — Task 3 Step 7 may surface MANY consuming cells (broad-grep). If >10 cells need `#[ignore]`, sonnet reviewer flag for cleanup-FOLLOWUP filing.

---

## Sub-skill expectations

This plan uses `superpowers:subagent-driven-development` per the user's Wave-1 + Wave-2 choice. Cross-repo dispatch: implementer subagents may need access to both `/scratch/code/shibboleth/mnemonic-toolkit/` AND `/scratch/code/shibboleth/mnemonic-gui/`. Reviewer dispatches need same.
