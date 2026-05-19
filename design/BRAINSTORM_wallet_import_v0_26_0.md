# BRAINSTORM: `wallet-import` (multi-format) — v0.26.0 cycle

**Date:** 2026-05-18
**Toolkit target:** `mnemonic-toolkit-v0.26.0` (minor bump)
**GUI target:** `mnemonic-gui-v0.11.0` (lockstep — static-form schema mirror only; dynamic widget deferred to v0.12.0)
**Status:** brainstorm approved across all 5 design sections (each section reviewer-looped by opus architect-review until 0 Critical / ≤1 Important folded inline); ready for SPEC consumption + implementation-plan drafting via `superpowers:writing-plans`.
**Predecessor:** `mnemonic-toolkit-v0.25.1` (`7c1f874`, 2026-05-18) — empty-string `--ms1` sentinel restored.
**Driving user direction (2026-05-18):**

> "I think we need to tackle this in a larger fashion: wallet import and export for a variety of formats (sparrow, spectre, electrum, core, and more)… in a larger fashion."

**Continuity:** `.wallet-import-export-multiformat-kickoff.md` (untracked in master; full session context at brainstorm start).

External authorities:
- [BIP-129 BSMS specification](https://github.com/bitcoin/bips/blob/master/bip-0129.mediawiki)
- [BIP-380 Output Script Descriptors](https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki) (descriptor-checksum algorithm)
- [BIP-389 Multipath descriptors](https://github.com/bitcoin/bips/blob/master/bip-0389.mediawiki)
- Bitcoin Core `listdescriptors` RPC (Core ≥ 0.21; xpub-only form scoped this cycle)

---

## §0 Motivation

The m-format constellation today **exports** to 8 third-party wallet formats (bitcoin-core, bip388, coldcard, jade, sparrow, specter, electrum, green — all shipped via `mnemonic export-wallet --format <X>` since v0.7..v0.8.1). The **import** direction is 0% built. Users today cannot ingest a third-party wallet descriptor / coordinator blob and convert it into a toolkit bundle for engraving or verification.

The user's seed-case during this session (BSMS Round-2 blob with `wsh(thresh(2, ..., sln:older(32768)))` decaying-multisig descriptor) demonstrated the gap concretely: the toolkit can deduce all 4 cosigner cards (2 × mk1 + 1 × md1 + 2 × empty-string ms1 sentinels) from the descriptor + origins, but has no command to do so today.

## §1 Research baseline (architect-review R0 + recon, 2026-05-18)

### §1.1 Existing export-side surface (already shipped)

- `mnemonic export-wallet --format <X>` covers 8 vendor formats. Trait-based dispatcher: `WalletFormatEmitter` at `wallet_export/mod.rs:322`, per-format zero-sized structs (`BitcoinCoreEmitter`, `SparrowEmitter`, etc.).
- Watch-only inputs only: `--slot @N.xpub=` + `--slot @N.fingerprint=` + `--slot @N.path=`. Optional `--slot @N.master_xpub=`.
- `wallet_export/pipeline.rs:160-205` `descriptor_to_bip388_wallet_policy` performs the inverse-direction transformation (concrete `[fp/path]xpub` → `@N/**` placeholders) — direct template for the import-side adapter.
- Cosigner ordering: `wallet_export/pipeline.rs:72-78` iterates slots in **slot-index order** (NOT lexsorted). The kickoff-doc claim that "toolkit's lexsort rule is wrong for non-sortedmulti" appears incorrect; Phase 0 verifies empirically.

### §1.2 Existing descriptor-parse path

- `parse_descriptor::parse_descriptor(input, &[ParsedKey], &[ParsedFingerprint])` at `parse_descriptor.rs:747-751` is the canonical descriptor→`md_codec::Descriptor` pipeline. Expects `@N[fp/path]` placeholder syntax; substitutes synthetic xpubs from `SEED_PREFIX = b"toolkit-v0.3"` before calling `MsDescriptor::from_str`.
- v0.19.0 introduced non-canonical descriptor support via `cmd/convert.rs --from <descriptor>`; v0.20.0 added `--classify-descriptor` diagnostic.
- BIP-380 checksum is auto-validated by `MsDescriptor::from_str`; explicit `verify_checksum` rarely needed.
- AST walk at `parse_descriptor.rs:398` `walk_root` already collects `KeyExpr` with origin — no new walk needed for import.

### §1.3 Existing intermediate types

- **`md_codec::Descriptor`** (`parse_descriptor.rs:21` import) — toolkit's normative descriptor wire-shape (`n`, `path_decl`, `use_site_path`, `tree`, `tlv`).
- **`CosignerKeyInfo`** (`synthesize.rs:190`, re-exported via `parse_descriptor.rs:12`) — per-cosigner-slot tuple aliasing `ResolvedSlot` (with `entropy: Option<Zeroizing<Vec<u8>>>`).
- **`DescriptorBinding`** (`parse_descriptor.rs:864`) — composite of `keys`, `fingerprints`, `cosigners`.
- **`md_codec::TlvSection`** has no `cosigner_order` TLV in v0.34.0 — adding one would be a sibling-repo wire-format break; **avoid**.

### §1.4 Existing secret-flag enumeration

`crates/mnemonic-toolkit/src/secrets.rs:49-59` authoritatively lists 6 inline-value secret-bearing flags:
1. `--passphrase` (bundle, verify-bundle, convert, derive-child, slip39)
2. `--passphrase-stdin` (already non-argv)
3. `--bip38-passphrase` (convert)
4. `--bip38-passphrase-stdin` (already non-argv)
5. `--ms1` (verify-bundle; new: import-wallet)
6. `--share` (slip39 combine, seed-xor combine)

Plus slot-subkey forms: `--slot @N.phrase=`, `--slot @N.ms1=`.

### §1.5 Existing exit-code tier discipline

`error.rs:296-328` `ToolkitError::exit_code` fixes the tier mapping:
- 1 = user-input/generic (`BadInput`)
- 2 = format-violation/refusal (`DescriptorParse`, `ConvertRefusal`, `ExportWalletSecretInput`)
- 3 = `FutureFormat` (use From-impl for BSMS 2.0)
- 4 = `BundleMismatch` (mismatch tier; ms1 ↔ blob xpub diverge)
- 5 = repair short-circuit (auto-fire BCH correction)

New error variants must MAP into this tier discipline. Per-error exit codes outside the tier table are disallowed.

### §1.6 GUI baseline

- 4 fixed top-level CLI tabs (`mnemonic` / `md` / `ms` / `mk`) at `main.rs:265-293`. **Per-subcommand is a combobox WITHIN the tab.** `import-wallet` adds one entry to the existing `mnemonic` tab's combobox; **no new top-level surface**.
- Run-confirm modal at `main.rs:686-688` renders argv tokens **VERBATIM** — no redaction currently exists. Memory claim that "redaction shipped" was incorrect (verified 2026-05-18 by Explore agent; grep `redact|argv_for_display|mask` returns 0 hits in `mnemonic-gui/src/`).
- Schema-mirror drift gate is version-tolerant (`>=1`); additive SubcommandSchema entries do **not** require schema-version bump.
- Schema v5 envelope at `gui_schema.rs:13`; bump rule: "Predicate/Effect changes bump the version; additive fields don't."
- Existing `FlagKind::TaggedOrIndexed(&'static [&'static str])` already supports the `--select-descriptor <N|active-receive|active-change|all>` shape — no new FlagKind needed.

## §2 Locked design decisions (D1–D17)

### Q1 — Direction

**D1 — Strict round-trip pairs.** Quality bar: bundle round-trip + semantic blob round-trip, with stderr unified-diff when blob bytes don't match exactly. Per-cycle 1-2 formats only; multi-cycle initiative.

### Q2 — First format(s)

**D2 — BSMS Round-2 + Bitcoin Core `listdescriptors`.** Both directions, lockstep. BSMS is fully greenfield round-trip; Bitcoin Core is import-side only (export already ships).

### Q3 — Seed material at import

**D3 — Pure watch-only imports for both formats this cycle.** Bitcoin Core `listdescriptors true` (xprv-bearing) refused; export-side `--private=true` form deferred to FOLLOWUP `bitcoin-core-xprv-handling`.

**D4 — Optional seed overlay at import time.** `wallet-import --ms1 <S>` (and `--slot @N.phrase=`) accept seed material; toolkit cross-validates supplied seed against blob xpub at the blob's declared origin path. Mismatch → exit 4 (`BundleMismatch` tier). Single-invocation UX; reuses verify-bundle's ms1↔mk1 cross-check machinery.

### Q4 — Round-trip rigor

**D5 — Bundle struct equality + semantic blob canonicalize + stderr diff.** Bundle round-trip: `bundle → export → import → bundle' == bundle` (full struct equality on toolkit `Bundle`). Semantic blob round-trip: per-format `canonicalize()` re-serialization; stderr unified-diff WARNING when raw bytes don't match (`--json` mode emits `roundtrip: {byte_exact, semantic_match, diff}` field in-envelope).

### Q5 — Command shape

**D6 — `import-wallet` with auto-detect + optional `--format` override.** Single subcommand `mnemonic import-wallet`; sniffs blob shape (BSMS header / Core JSON) by default; `--format <bsms|bitcoin-core>` overrides and validates against sniff. Ambiguous sniff → exit 1 with "supply `--format`."

### Q6 — GUI lockstep

**D7 — CLI + GUI lockstep this cycle, split across two GUI cycles.** v0.11.0 ships CLI + static-form schema mirror (matches `bundle` subcommand's existing repeating-flag GUI shape; no dynamic per-cosigner widget). v0.12.0 ships dynamic per-cosigner widget + `--inspect` pre-sniff flag + format-sniffer label + roundtrip-diff panel + run-confirm-modal argv-redaction (defense-in-depth). Architect recommendation; verified scope realism against v0.6/v0.7/v0.10 cycle sizes.

### Q7 — BSMS round support

**D8 — Round-2 only this cycle; Round-1 + verify-signatures filed as FOLLOWUPs.** Lenient acceptance: 2-line shape (`BSMS 1.0\n<descriptor>#checksum`) emits stderr WARNING; 6-line full Round-2 (`BSMS 1.0\n<token>\n<descriptor>#checksum\n<derivation_path>\n<first_address>\n<signature>`) parsed; first-address verification informational (WARNING on mismatch; not hard-error this cycle); token + signature + first_address + derivation_path preserved in `ParsedImport.bsms_audit` for `--json` envelope.

### Q8 — Output shape

**D9 — Engraving cards on stdout by default; `--json` for bundle JSON envelope.** Matches existing `synthesize` pattern; `--json` array of envelopes when N > 1.

### Q9 — Multi-descriptor Core handling

**D10 — Emit N bundles; `\n;\n` separator (cards) or JSON array (`--json`).** Default `--select-descriptor all`; explicit selector accepts `N | active-receive | active-change | all`. Under `--format bsms` (single descriptor), any selector value is equivalent to `all` with stderr NOTICE on `active-*` or non-zero `N`.

### Q-arch1 — Module layout (from architect R1)

**D11 — Mirror the `WalletFormatEmitter` trait pattern.** New `WalletFormatParser` trait:
```rust
pub(crate) trait WalletFormatParser {
    fn sniff(blob: &[u8]) -> bool;
    fn parse(blob: &[u8]) -> Result<Vec<ParsedImport>, ToolkitError>;
}
```
Per-format zero-sized structs (`BsmsParser`, `BitcoinCoreParser`) stay private to their modules; only trait + `ParsedImport` are `pub(crate)`.

### Q-arch2 — Intermediate types (from architect R1)

**D12 — Reuse `md_codec::Descriptor` + `ResolvedSlot`; do NOT invent `ImportedDescriptor`.** `ParsedImport` carries `descriptor: md_codec::Descriptor`, `cosigners: Vec<ResolvedSlot>` (INVARIANT: all `entropy: None` for watch-only imports), `network: Network`, `threshold: Option<u8>`, `bsms_audit: Option<BsmsAuditFields>`. Per §7.0.c amendment: `CosignerKeyInfo` is a deprecated alias for `ResolvedSlot` (`synthesize.rs:182-188`); new code uses the canonical `ResolvedSlot` name.

### Q-arch3 — AST walk (from architect R1)

**D13 — Reuse existing `parse_descriptor::parse_descriptor()`.** Build an adapter pre-step: lex concrete `[fp/path]xpub` occurrences from raw descriptor string with a regex; substitute `@N` placeholders + collect `(ParsedKey, ParsedFingerprint)` per `@N`; preserve declaration order. Hand placeholder-form descriptor + collected keys/fingerprints to `parse_descriptor::parse_descriptor(input, &keys, &fingerprints)`.

### Q-arch4 — Checksum + network (from architect R1)

**D14 — BIP-380 (not BIP-389) checksum; auto-validated via `MsDescriptor::from_str`.** Network detection via `slip0132.rs` (supports ypub/zpub/etc., not just tpub/xpub substring).

### Q-arch5 — Cosigner ordering (from architect R1)

**D15 — `multi(...)` vs `sortedmulti(...)` is the discriminator.** No new `cosigner_order: declaration` TLV (would require md-codec sibling-repo wire-format break). `@N` ordering preserved via dense `0..n` invariant in `parse_descriptor::resolve_placeholders`.

### Q-redact — Run-confirm-modal (from architect R2 — GUI section)

**D16 — `@env:VAR` sentinel cross-cutting across all 6 inline-value secret-flag surfaces.** SPEC §5.11 (NEW) — "CLI value-source sentinels." Resolution: clap-parse-time substitution. Missing env-var → exit 1 with cross-cutting `ToolkitError::EnvVarMissing` variant (canonical name; NOT import-wallet-specific). Empty-string env-var preserves v0.25.1 watch-only sentinel semantics. Literal `@env:` cannot be escaped this cycle (FOLLOWUP `env-var-sentinel-literal-escape`).

By construction, the sentinel obviates v0.11.0's need for argv-redaction: argv contains `--ms1 @env:MNEMONIC_MS1_0`, not the secret. Run-confirm-modal renders the sentinel. **GUI argv-redaction becomes a v0.12.0 defense-in-depth FOLLOWUP, not a v0.11.0 blocker.**

### Q-error — Exit-code tier discipline (from architect R3 — Section 5)

**D17 — Map import-wallet variants to existing tiers; no per-error exit numbering.**

| Tier | Variants |
|---|---|
| 1 | `ImportWalletAmbiguousFormat`, `ImportWalletFormatMismatch`, `EnvVarMissing` (cross-cutting, NOT import-wallet-specific) |
| 2 | `ImportWalletParse`, `ImportWalletXprvForbidden` |
| 3 | (via `FutureFormat` From-impl for BSMS 2.0+) |
| 4 | `ImportWalletSeedMismatch` |
| 5 | (existing repair short-circuit) |

ToolkitError naming: drop `Error` suffix to match `DescriptorParse`/`ConvertRefusal` convention.

## §3 Architecture

### §3.1 Module layout (toolkit)

```
crates/mnemonic-toolkit/src/
├── cmd/
│   └── import_wallet.rs              — CLI entry; clap glue; dispatch via WalletFormatParser
├── wallet_import/
│   ├── mod.rs                        — pub(crate) trait WalletFormatParser + ParsedImport
│   ├── sniff.rs                      — auto-detect; ambiguous → exit 1
│   ├── bsms.rs                       — struct BsmsParser; impl WalletFormatParser
│   ├── bitcoin_core.rs               — struct BitcoinCoreParser; impl WalletFormatParser
│   ├── pipeline.rs                   — concrete-keys → @N-placeholder adapter (inverse of wallet_export::pipeline::descriptor_to_bip388_wallet_policy)
│   └── roundtrip.rs                  — semantic round-trip + diff helper
├── secrets.rs                        — extension: env-var sentinel resolution (cross-cutting)
└── error.rs                          — new ToolkitError variants per §1.5 mapping
```

### §3.2 CLI surface

```
mnemonic import-wallet [OPTIONS]
  --blob <FILE|->                                             required
  --format <bsms|bitcoin-core>                                optional; auto-detect default
  --ms1 <STRING>                                              repeatable, positional cosigner-index
  --slot @<N>.phrase=<STRING>                                 existing slot-subkey pattern
  --select-descriptor <N|active-receive|active-change|all>    default `all`
  --json                                                      bool; emit JSON envelope array on stdout
  --no-auto-repair                                            global (auto-attached)
```

Any inline-value secret flag accepts `@env:<VAR>` sentinel (`--ms1 @env:MNEMONIC_MS1_0`, `--slot @0.phrase=@env:WALLET_PHRASE`, etc.).

### §3.3 Output

- **Default stdout:** human-readable engraving card(s); cards separated by `\n;\n` when N > 1.
- **`--json` stdout:** JSON array of bundle envelopes.
- **Stderr:** progress / NOTICEs / WARNINGs / roundtrip-diff (when not byte-exact).

## §4 GUI lockstep scope (v0.11.0)

### §4.1 SubcommandSchema entry

New entry in `mnemonic-gui/src/schema/mnemonic.rs`:
- `--blob <FILE>` → existing FilePicker widget.
- `--format <bsms|bitcoin-core>` → `FlagKind::TaggedOrIndexed(&["bsms","bitcoin-core"])`.
- `--ms1 <STRING>` → repeating-text-input (mirrors `bundle`).
- `--slot @<N>.phrase=<STRING>` → existing slot-subkey pattern.
- `--select-descriptor` → `FlagKind::TaggedOrIndexed(&["active-receive","active-change","all"])`.
- `--json` → Bool.
- `--no-auto-repair` → auto-attached.

### §4.2 Surface placement

Lives in the existing `mnemonic` top-level tab's subcommand combobox. **No new top-level UI.** Schema version stays **v5** (additive).

### §4.3 Env-var seed channel flow

1. User enters seed in GUI text input.
2. On Run, GUI sets `MNEMONIC_MS1_0`, `MNEMONIC_MS1_1`, ... env-vars on spawned subprocess env.
3. argv contains `--ms1 @env:MNEMONIC_MS1_0` (sentinel, not seed) per cosigner.
4. Run-confirm-modal renders sentinel — secret never in argv.
5. Subprocess reads env-var, processes, exits; env clears with subprocess.

### §4.4 Deferred to v0.12.0

- Dynamic per-cosigner widget rendering (via new `--inspect` flag).
- `--inspect` JSON output: `{format, cosigner_count, descriptors: [...]}`.
- SlotEditor reuse for per-cosigner labeled rows.
- Format-sniffer label widget.
- Roundtrip-diff panel widget.
- Run-confirm-modal argv-redaction (defense-in-depth).
- Full GUI manual chapter (v0.11.0 ships short static-form chapter only).

## §5 Test strategy (cell budget per phase)

| Phase | Topic | Cells |
|---|---|---|
| 0 | Recon (empirical lexsort verification; md-codec wire shape confirm) | 0 |
| 1 | Cross-cutting `@env:VAR` sentinel (6 secret-flag surfaces) | 12-18 |
| 2 | BSMS Round-2 parser (2-line + 6-line happy paths + checksum/version/SLIP-132 negatives) | 10-14 |
| 3 | Bitcoin Core `listdescriptors` parser (single + multi + multisig + xprv-refusal + dropped-fields NOTICE) | 10-14 |
| 4 | Round-trip (bundle struct equality + semantic blob canonicalize; 12-15 fixtures each format) | 24-30 |
| 5 | Auto-detect + seed overlay + sniff-path round-trip | 8-10 |
| 6 | GUI lockstep (schema-mirror + kittest + 41-mnemonic.md manual subsection) | 6-8 |
| | **Total** | **70-94** |

Phase 4 round-trip cells use `--format <explicit>` only; Phase 5 adds sniff-path round-trip coverage.

Per-phase opus architect R0/R1+ until 0 Critical / 0 Important (per `[[feedback-opus-primary-review-agent]]`).

## §6 Deferred FOLLOWUPs (filed at cycle close — 13 entries)

1. `bsms-round1-and-coordinator-output-import`
2. `bsms-verify-signatures` (full BIP-129 HMAC + token verification)
3. `bsms-first-address-hard-error-mode` (`--strict-first-address`)
4. `bitcoin-core-xprv-handling` (refuse vs strip vs new xprv card design call)
5. `wallet-import-inspect-flag` (v0.12.0 prerequisite for dynamic GUI widget)
6. `mnemonic-gui-import-wallet-dynamic-widget` (v0.12.0)
7. `mnemonic-gui-run-confirm-redaction` (v0.12.0; defense-in-depth)
8. `mnemonic-gui-import-wallet-roundtrip-diff-panel` (v0.12.0)
9. `wallet-import-format-parity-with-export-side-vendors` (cites closed `wallet-export-industry-formats`; enumerates Sparrow / Specter / Electrum / Coldcard / Jade / Green as v0.27+ targets)
10. `env-var-sentinel-literal-escape` (allow literal `@env:` strings via escape mechanism)
11. `bip388-wallet-policy-import` (NEW; not a fold of any existing entry — kickoff doc's `bip388-bidirectional` claim was incorrect; the closed `export-wallet-descriptor-bip388-interop` covers only EXPORT direction; Companion entry in toolkit `design/FOLLOWUPS.md` cites the closed export-side companion)
12. `wallet-import-sniff-bitcoin-core-tighten-heuristic` (positive-marker check for `wallet_name` + `timestamp` / `next_index` to disambiguate from Sparrow/Specter `descriptors`-array JSON shapes)
13. `bsms-audit-field-regeneration` (`--coordinator-key <FILE>` enabling re-signed BSMS Round-2 export from a bundle + coordinator HMAC key)

## §7 Artifact updates

- `design/SPEC_wallet_import_v0_26_0.md` (NEW; this brainstorm's normative companion).
- `design/SPEC_mnemonic_toolkit_v0_5.md` — amendments (anchor numbers verified against v0.5 TOC `grep '^## '` on 2026-05-18):
  - **§5.11 (NEW)**: cross-cutting CLI value-source sentinels (`@env:VAR`). Placed textually after the §5-cluster (§5.5, §5.5.a, §5.6, §5.7, §5.8); numerically discontiguous-OK per v0.5 delta-only ordering.
  - **§6.11 (NEW)**: `import-wallet` CLI grammar (mirrors §6.7 verify-bundle structure). Placed after existing §6.10 (Conditional-applicability projection).
  - **§6.11.a (NEW)**: wallet_import round-trip discipline. Per §7.0.b amendment: sub-section of §6.11 (NOT a new §7 top-level) to preserve v0.5 SPEC's delta-only ordering convention. Mirrors `§4.12.a-g` precedent established by the v0.19.0 non-canonical descriptor cycle.
- `design/FOLLOWUPS.md` — 11 new entries (§6 above).
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — new `## import-wallet` subsection (Phase 6; load-bearing per CLAUDE.md mirror invariant).
- `docs/manual/src/<new>-foreign-formats.md` — new chapter on BSMS Round-2 + Bitcoin Core blob shapes.
- `docs/manual-gui/` — short static-form import chapter.
- `CHANGELOG.md` (toolkit) — v0.26.0 entry.
- `CHANGELOG.md` (`mnemonic-gui`) — v0.11.0 entry.

## §8 Reviewer-loop fold history

R0 architect review of Section 3 returned 8 findings (4 Important: trait pattern + intermediate-type reuse + AST walk reuse + checksum naming; 2 Critical: cosigner_order TLV nonexistent + canonicalize precision; 2 Minor). All folded into D11–D15.

R1 convergent review of revised Section 3: 11/12 prior findings satisfied; 5 new findings (Important: watch-only invariant + BSMS 6-line wire shape + `parse_descriptor` signature flow; Minor: `--output` ambiguity + visibility contract). All folded.

R0 architect review of Section 4 returned 8 findings (3 Critical: top-level-tab misread + run-confirm verbatim + stdin double-booking; 5 Important: FlagKind reuse + schema-version bump + MNEMONIC_FORCE_TTY + dynamic-widget prereq + cycle-split). All folded; cycle split into v0.11.0 static + v0.12.0 dynamic.

R0 architect review of Section 5 returned 13 findings (2 Critical: exit-code tier conflict + env-var-sentinel cross-cutting scope; 6 Important: SPEC §-numbering + FOLLOWUP duplicates/wrong-closure + test budget; 5 Minor). All folded; test budget grew from 38-55 to 70-94 cells.

## §9 Open questions (resolved at SPEC drafting)

- BSMS first-address verification SPEC byte-exact wording (NOTICE template, JSON envelope shape).
- Empty-string env-var resolution semantics interplay with v0.25.1 sentinel: explicit SPEC text.
- `--ms1` sentinel + repeated positional + stdin (`--ms1 - --ms1 @env:VAR`) — exit-1 disambiguation.

---

**End of BRAINSTORM.**
