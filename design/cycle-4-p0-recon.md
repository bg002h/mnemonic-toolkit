# Cycle 4 P0 STRICT-GATE recon — 2026-05-21

**Origin/master HEAD at recon time:** `da122fb` (v0.28.7).

This recon is a STRICT GATE per architect I1 fold (`design/BRAINSTORM_v0_28_plus_residual_followups.md` §"Architect review folds"). Plan-doc body must NOT be written until ALL findings landed.

---

## Part 1 — ToolkitError variant freeze (architect I1)

**Total variants: 44** (was 43 pre-v0.28.7; +1 `BsmsTaprootImportRefused` added in Cycle 3).

### Complete variant list (declaration order, `error.rs` L10-287)

```
 1. BadInput
 2. Bip39
 3. Bitcoin
 4. MsCodec
 5. MkCodec
 6. MdCodec
 7. ModeViolation
 8. BundleMismatch
 9. NetworkMismatch
10. FutureFormat
11. MultisigConfig
12. CosignerSpec
13. CosignersFile
14. DescriptorParse
15. DescriptorReparseFailed
16. Bip388Distinctness
17. Bip388VerifyDistinctness
18. SlotInputViolation
19. ConvertRefusal
20. ExportWalletSecretInput
21. ExportWalletFormatStub
22. ExportWalletTaprootMultisigUnsupported
23. ExportWalletMissingFields
24. DeriveChildUnsupportedApp
25. DeriveChildLengthOutOfRange
26. DeriveChildLengthNotApplicable
27. HrpMismatch
28. UnknownHrp
29. Repair
30. CompareCost
31. RepairShortCircuit
32. Io
33. EnvVarMissing
34. ImportWalletAmbiguousFormat
35. ImportWalletFormatMismatch
36. ImportWalletParse
37. ImportWalletSeedMismatch
38. ImportWalletWatchOnlyViolation
39. ImportWalletXprvForbidden
40. XpubSearchNoMatch
41. BsmsRound1Malformed
42. BsmsSignatureMismatch
43. BsmsTaprootImportRefused
44. BsmsTaprootRefused
```

### Variants out of alphabetical order

~30 of the 44 variants need to move in the declaration block. Notable violations:
- `MsCodec` / `MkCodec` / `MdCodec` (L14-16) before `ModeViolation` — `Md` < `Mk` < `Mo` < `Ms`; all three out of order
- `BundleMismatch` (L28) should follow `Bsms*` but precedes them
- `NetworkMismatch` (L33-36) should fall after `MultisigConfig`
- `FutureFormat` (L37-40) between `ExportWalletTaprootMultisigUnsupported` + `HrpMismatch`
- `MultisigConfig` / `CosignerSpec` / `CosignersFile` (L44-58) after `Mo...` instead of after `FutureFormat`
- `CompareCost` (L158) buried between `Repair` and `RepairShortCircuit`
- `Io` (L169) between `ImportWalletXprvForbidden` and `MdCodec`

### Cascade match blocks

- `exit_code` match: L428-482 (~44 arms)
- `kind` match: L489-536 (~44 arms)
- `message` match: L542-711 (~44 arms)
- `details` block: L718-742 (PARTIAL — wildcard `_ => None`; only 7 named arms need reordering)

**Estimate:** ~132 arm rewrites (3 exhaustive × 44 arms). Plus ~30 variant lines in declaration. **Pure reorder, no semantic change** — single commit acceptable if sonnet verifies zero-semantic-drift.

**DRIFT from FOLLOWUPS body:** body says "~50+ variants × 4 exhaustive match blocks = ~250 line moves" — actual is 44 variants × 3 exhaustive + 1 partial = ~132 arm moves.

---

## Part 2 — Slug recon

### Slug A — `pr-26-import-provenance-three-variant-cleanup`

**Source:** `crates/mnemonic-toolkit/src/wallet_import/mod.rs:69-139`.

**Current shape (POST-Cycle-3 expansion — significantly evolved from slug filing):**

```rust
pub(crate) enum ImportProvenance {
    BitcoinCore(CoreSourceMetadata),
    Bsms(Option<BsmsAuditFields>),
    Coldcard(coldcard::ColdcardSourceMetadata),
    ColdcardMultisig(coldcard_multisig::ColdcardMultisigSourceMetadata),
    Electrum(electrum::ElectrumSourceMetadata),
    Jade(jade::JadeSourceMetadata),
    Sparrow(sparrow::SparrowSourceMetadata),
    Specter(specter::SpecterSourceMetadata),
}
```

**DRIFT FLAG (load-bearing):** The slug's "3-variant target" was filed when the enum had only 2 variants. It now has 8. The slug body is stale. **Actual work is a 1-variant split**: replace `Bsms(Option<BsmsAuditFields>)` with `BsmsTwoLine` + `BsmsSixLine(BsmsAuditFields)`, leaving the other 6 variants intact.

The brainstorm spec at §"Cycle 4" Phase 2 says "exact shape decided at brainstorm-write for the cycle" — so plan-doc may lock the 2-variant split (cleaner) OR keep the 3-variant proposal (with a third `BsmsRoundtripDescriptorOnly` variant for future scope).

**Recommended:** 2-variant split (`BsmsTwoLine` + `BsmsSixLine(BsmsAuditFields)`). Third variant only adds value if a future Round-trip descriptor-only path exists; not yet implemented.

**Consumer match blocks for `Bsms(Option<_>)`:**
- `mod.rs:146` — `bsms_audit()` accessor (8-arm match)
- `mod.rs:160` — `source_metadata()` accessor (8-arm)
- `mod.rs:176+` — additional per-variant accessors (each 8-arm)
- `bsms.rs:342` — construction site `provenance: ImportProvenance::Bsms(audit)`
- `cmd/import_wallet.rs:1370` — `p.bsms_audit()` accessor call (via accessor; only accessor needs rewriting)
- `cmd/import_wallet.rs:1731` — text-path `b.bsms_audit().is_some()` check

**Tests using `Bsms(Some/None)` directly:** `mod.rs:505, 519, 526, 546, 553`.

**Citation drift:** slug body cites `bsms.rs:266` — actual is `bsms.rs:342` (line moved post-Cycle-3 taproot refusal additions).

### Slug B — `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution`

**Source files (all citations verified accurate at HEAD `da122fb`):**
- `cmd/xpub_search/path_of_xpub.rs:144` — `PathOfXpubResult`
- `cmd/xpub_search/passphrase_of_xpub.rs:169` — `PassphraseOfXpubResult`
- `cmd/xpub_search/account_of_descriptor.rs:155` — `AccountOfDescriptorResult`

**Current `PathOfXpubResult` (struct):**
```rust
pub struct PathOfXpubResult {
    pub result: &'static str,           // "match" | "no_match"
    pub path: Option<String>,           // null on no-match
    pub template: Option<String>,       // null on no-match
    pub account: Option<u32>,           // null on no-match or no-account-token
    pub target_xpub_canonical: String,
    pub target_xpub_variant: Option<&'static str>,
    pub searched_count: usize,
}
```

**`PassphraseOfXpubResult`** — identical field set to PathOf (same shape per plan §6.5; separate struct for future divergence).

**`AccountOfDescriptorResult`:**
```rust
pub struct AccountOfDescriptorResult {
    pub result: &'static str,
    pub matched_cosigners: Vec<MatchedCosignerJson>,
    pub cosigners_total: usize,
    pub searched_count_per_cosigner: usize,
    pub descriptor_shape: DescriptorShape,
    pub unspendable_internal_keys: Vec<usize>,
}
```

**JSON wire-shape break (SemVer-minor):**
- `PathOfXpubResult` + `PassphraseOfXpubResult`: no-match currently emits `"path": null, "template": null, "account": null`. Post-tagged-enum (`#[serde(tag = "kind")]`), no-match variant omits those keys. Consumers checking `.path === null` break.
- `AccountOfDescriptorResult`: `result` discriminant disappears; `matched_cosigners: []` on no-match becomes no-match variant with no `matched_cosigners` key.
- v0.27.0 fixture tests in `tests/fixtures/v0_27_0_envelopes/` need conversion to `#[ignore]` with SemVer rationale comment.

### Slug C — `error-rs-retroactive-alphabetical-sort`

See Part 1 list. FOLLOWUPS body says "~50+ variants × 4 exhaustive match blocks = ~250 line moves" — corrected to 44 variants × 3 exhaustive blocks ~132 arm moves.

---

## Part 3 — GUI baseline

- **mnemonic-gui repo:** `/scratch/code/shibboleth/mnemonic-gui/` — EXISTS.
- **`pinned-upstream.toml` `[mnemonic].tag`:** `mnemonic-toolkit-v0.28.4` (L33).
- **`Cargo.toml` workspace dep:** `mnemonic-toolkit = { git = "...", tag = "mnemonic-toolkit-v0.28.4" }` (L42).
- **`src/schema/mnemonic.rs` size:** 2484 lines.
- **Latest GUI release tag:** `mnemonic-gui-v0.13.0` (DRIFT: my memory `[[project_v0_24_0_cycle_shipped]]` cited v0.10.0; actual is v0.13.0 — 3 minor releases ahead).

### GUI pin lag

GUI is pinned to toolkit `v0.28.4`. Master HEAD is `v0.28.7`. The intervening v0.28.5/v0.28.6/v0.28.7 changes (including new `BsmsTaprootImportRefused` variant) are not yet reflected in GUI schema-mirror. Cycle 4's `v0.29.0` bump will pick up everything from v0.28.4 onward in one step.

Schema-mirror lockstep delta on Cycle 4:
- v0.28.5: no CLI surface change → no schema-mirror update needed
- v0.28.6: no CLI surface change → no schema-mirror update needed
- v0.28.7: no CLI surface change → no schema-mirror update needed
- **v0.29.0 itself:** xpub-search JSON wire-shape break needs explicit schema-mirror update

So GUI Cycle 4 schema-mirror touch is **xpub-search result-shape only** + the GUI's own v0.14.0 SemVer-minor bump (downstream wire-shape break from toolkit's v0.29.0).

---

## Cross-cutting drift summary

1. **Slug A:** "3-variant" framing stale; actual is 1-variant split (`Bsms(Option<_>)` → `BsmsTwoLine` + `BsmsSixLine`). Construction-site line drifted `:266` → `:342`.
2. **Slug C:** Match-block count overstated ("4 exhaustive" → actual 3 exhaustive + 1 partial); variant count overstated ("50+" → actual 44). Arm-move estimate corrected to ~132.
3. **GUI latest release:** `v0.13.0` not `v0.10.0` (memory drift, no action needed for cycle).
4. **GUI pin lag:** v0.28.4 → v0.29.0 jump captures 4 patch releases in one lockstep tag.
5. **Slug B citations:** All 3 struct-line citations verified accurate at HEAD; no drift.

## Recommendations for plan-doc body

1. **Slug A:** Lock 2-variant split (`BsmsTwoLine` + `BsmsSixLine(BsmsAuditFields)`); defer 3rd variant unless brainstorm-time discovery surfaces a need.
2. **Slug B:** Tagged enum conversion via `#[serde(tag = "kind")]`. Convert `tests/fixtures/v0_27_0_envelopes/` cells to `#[ignore]` with SemVer-rationale comments + capture v0.28.0 cells inline.
3. **Slug C:** Single "sort-only, no semantic change" commit OR fold into the same Cycle 4 commit if sonnet verifies zero-semantic-drift.
4. **GUI lockstep:** Update `pinned-upstream.toml` `[mnemonic].tag` + `Cargo.toml` dep tag to `mnemonic-toolkit-v0.29.0`; update schema-mirror for xpub-search result shape; bump GUI to `mnemonic-gui-v0.14.0` (SemVer-minor downstream).
