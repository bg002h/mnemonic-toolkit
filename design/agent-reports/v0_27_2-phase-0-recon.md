# v0.27.2 Phase 0 recon dossier

**Date:** 2026-05-19
**origin/master SHA:** fbdc0cf26d48ee1fa77c64c57a999565d72bb216
**release/v0.27.2 tip:** ac08d591d4066774bdde82161b52dd7318994f1e

---

## Item 1 — ImportProvenance refactor

### cmd/import_wallet.rs access sites

Grep command: `grep -nE '\.(bsms_audit|source_metadata)([^_a-zA-Z]|$)' crates/mnemonic-toolkit/src/cmd/import_wallet.rs`

Raw output (7 lines total):
```
587:        if let Some(audit) = &p.bsms_audit {
599:        if let Some(meta) = &p.source_metadata {
806:        let audit_str = if b.bsms_audit.is_some() {
811:        writeln!(stdout, "bundles[{i}].bsms_audit={audit_str}").map_err(ToolkitError::Io)?;
818:        let src_meta_str = if b.source_metadata.is_some() {
823:        writeln!(stdout, "bundles[{i}].source_metadata={src_meta_str}")
825:        if let Some(m) = &b.source_metadata {
```

**Plan expected:** 7 total — 5 actionable at `{587, 599, 806, 818, 825}` + 2 string-literal false-positives at `{811, 823}`.

**Verdict: ACCURATE.** All 7 lines match exactly. Classification breakdown:
- **Actionable (5):** 587 (`.bsms_audit` field read, `p`), 599 (`.source_metadata` field read, `p`), 806 (`.bsms_audit.is_some()`, `b`), 818 (`.source_metadata.is_some()`, `b`), 825 (`.source_metadata` field read, `b`)
- **False-positives (2):** 811 (`writeln!` format string `"bundles[{i}].bsms_audit={audit_str}"`), 823 (`writeln!` format string `"bundles[{i}].source_metadata={src_meta_str}"`)

Phase 2 Task 2.6 task text is correct as written.

### wallet_import/mod.rs apply_select_descriptor sites

Grep command: `grep -nE '\.(bsms_audit|source_metadata)([^_a-zA-Z]|$)' crates/mnemonic-toolkit/src/wallet_import/mod.rs`

Raw output:
```
150:                    p.source_metadata
167:                    p.source_metadata
```

**Plan expected:** 2 lines at `{150, 167}`, with predicates at lines 152 + 169.

**Verdict: ACCURATE.** Lines 150 and 167 match exactly. Context-read confirms:
- Line 150: `.source_metadata` field access inside `SelectDescriptor::ActiveReceive` filter closure; `.as_ref()` at 151, predicate `m.active && !m.internal` at 152.
- Line 167: `.source_metadata` field access inside `SelectDescriptor::ActiveChange` filter closure; `.as_ref()` at 168, predicate `m.active && m.internal` at 169.

### wallet_import/bsms.rs ParsedImport construction

Grep command: `grep -n 'ParsedImport {' crates/mnemonic-toolkit/src/wallet_import/bsms.rs`

Raw output:
```
266:        Ok(vec![ParsedImport {
```

**Plan expected:** line 266.

**Verdict: ACCURATE.** `Ok(vec![ParsedImport {` at line 266, with fields `bsms_audit: audit` at line 272 and `source_metadata: None` at line 273.

### wallet_import/bitcoin_core.rs ParsedImport construction

Grep command: `grep -n 'ParsedImport {' crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs`

Raw output:
```
299:    Ok(ParsedImport {
```

**Plan expected:** line 299 (with let-binding range 291-307).

**Verdict: ACCURATE.** `Ok(ParsedImport {` at line 299. Context-read confirms full construction range:
- 291: `let source_metadata = Some(CoreSourceMetadata {` — let-binding open
- 297: `};` — let-binding close
- 299: `Ok(ParsedImport {` — struct open
- 305: `bsms_audit: None::<BsmsAuditFields>,` — bsms_audit field
- 306: `source_metadata,` — source_metadata field
- 307: `})` — struct close

---

## Item 5 — gui-schema arm count

Grep command: `grep -cE '^[[:space:]]+"[a-z-]+" => [a-z_]+_conditional_rules\(\),$' crates/mnemonic-toolkit/src/cmd/gui_schema.rs`

Raw output: `6`

**Plan expected:** `6`.

**Verdict: ACCURATE.** Arm count matches. Phase 1 Task 1.4 `EXPECTED_ARM_COUNT` constant should be set to `6` as written in the plan.

---

## Item 6 — drift cells (anticipated zero)

### cli_xpub_search_drift_v0_27_0.rs

Grep command: `grep -nE '\bsearched\b[^_]' crates/mnemonic-toolkit/tests/cli_xpub_search_drift_v0_27_0.rs`

Raw output: (no matches, exit 1)

**Plan expected:** ZERO matches.

**Verdict: ACCURATE.** Zero bare `searched` tokens — drift cells reference `searched_count` and `searched_count_per_cosigner`, both correctly excluded by the `[^_]` negative lookahead. Item 6 remains doc-only; no test updates required.

### cli_import_wallet_envelope_v0_27_0.rs

Grep command: `grep -nE '\bsearched\b[^_]' crates/mnemonic-toolkit/tests/cli_import_wallet_envelope_v0_27_0.rs`

Raw output: (no matches, exit 1)

**Plan expected:** ZERO matches.

**Verdict: ACCURATE.** Zero bare `searched` tokens. Same conclusion as above.

---

## Sibling-repo (Phase 3 sizing)

### GUI repo presence

Path: `/scratch/code/shibboleth/mnemonic-gui/` — **EXISTS** (last modified 2026-05-18 18:01).

### Toolkit pin (Cargo.toml)

```
mnemonic-toolkit = { git = "https://github.com/bg002h/mnemonic-toolkit", tag = "mnemonic-toolkit-v0.26.0" }
```

**Plan assumed:** `v0.26.0`. **Verdict: ACCURATE.**

### Toolkit pin (pinned-upstream.toml)

```
url = "https://github.com/bg002h/mnemonic-toolkit"
# Cross-cite with Cargo.toml's [dependencies] mnemonic-toolkit tag.
tag = "mnemonic-toolkit-v0.26.0"
workspace-member-path = "crates/mnemonic-toolkit"
```

**Plan assumed:** `v0.26.0`. **Verdict: ACCURATE.** Both Cargo.toml and pinned-upstream.toml are at `v0.26.0`; Phase 3 must bump both to `v0.27.2`.

### GUI envelope consumers (schema_version references)

`grep -r --include='*.rs' -l 'schema_version' /scratch/code/shibboleth/mnemonic-gui/src/`

Result:
```
/scratch/code/shibboleth/mnemonic-gui/src/persistence.rs
```

One file. Phase 3 GUI smoke cells should exercise this consumer.

### GUI import-wallet / xpub-search / bsms consumers

`grep -r --include='*.rs' -l 'import-wallet\|xpub-search\|bsms_round1\|bsms-round1' /scratch/code/shibboleth/mnemonic-gui/src/`

Result:
```
/scratch/code/shibboleth/mnemonic-gui/src/schema/mnemonic.rs
```

One file. Phase 3 GUI smoke cells should target this schema consumer for envelope shape verification.

### Schema-mirror.yml auto-track mechanism

`grep -A 3 'tomllib\|pinned-upstream' /scratch/code/shibboleth/mnemonic-gui/.github/workflows/schema-mirror.yml`

Confirmed: Python `tomllib` parse-pre step named `parse-pinned-upstream` reads `pinned-upstream.toml` via stdlib `tomllib`, exports per-CLI tag values as step outputs. The `mnemonic_tag` value propagates to install steps via `env: TAG`. Single source of truth confirmed — bumping `pinned-upstream.toml [mnemonic].tag` cascades to schema-mirror install without a separate workflow edit.

---

## Cross-cutting observations

1. **All citations ACCURATE.** Every line-number citation from the plan (Item 1 access sites, construction sites, Item 5 arm count, Item 6 zero-match expectation) verified byte-exact against `origin/master` tip `fbdc0cf`. No drifted citations; Phase 1 + Phase 2 task text is safe to dispatch as-is.

2. **Item 1 access-site categorization holds.** The 5 actionable vs. 2 false-positive split is structurally clean — the false-positives (lines 811 + 823) are `writeln!` format strings embedding the field name as a display label, not field access. Phase 2 Task 2.6 correctly identifies only the 5 actionable sites for mechanical syntax shift.

3. **GUI pin at v0.26.0 confirmed.** Both Cargo.toml and pinned-upstream.toml hold `v0.26.0`. The auto-track tomllib mechanism is in place; Phase 3 bump to `v0.27.2` requires editing only `pinned-upstream.toml [mnemonic].tag` + `Cargo.toml` dep tag + regenerating `Cargo.lock`.

4. **GUI envelope consumer surface is narrow.** Only two source files implicated: `persistence.rs` (schema_version) and `src/schema/mnemonic.rs` (import-wallet/xpub-search/bsms routing). Phase 3 smoke cells can be scoped tightly to these two files' downstream behavior.

5. **Item 6 remains doc-only.** Zero bare `searched` occurrences in either drift test file confirms no test-update cascade; only the `error.rs` docstring + `address_of_xpub.rs` inline comment edits in Phase 1 Task 1.6 are needed.
