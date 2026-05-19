# Phase P0A — architect R0 review

**Reviewer:** Opus 4.7 via feature-dev:code-architect
**Branch:** `v0.28.0/p0a-spec-scaffolding`
**Commit under review:** `aa3a537` (P0A scope; `12c248f` cycle-followups infrastructure not under review)
**Source SHA verified against:** `aa3a537`
**Plan-doc:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` Phase 0 P0A row

---

## Critical (correctness-blocking; would break P0B.1+ execution)

**None.** P0A delivers exactly what it scopes for downstream phases: the §1.4 namespace lock, §2.2 schema_version-stay-at-"1" rationale, §6.1 sniff semantic carry-forward, §6.1.1 VENDOR_MARKER_KEYS list (with implementation in lockstep), §6.2 alphabetical SniffOutcome final order, §10 BIP-129 line-3 nomenclature lock per recon dossier, and §11.x per-parser provenance schemas. All five citations the task brief asked to re-verify are sound at SHA `aa3a537`. No execution-blocker for P0B.1+.

## Important (would block P0A merge)

**I1 — SPEC §11.6 Electrum `wallet_type` set is incomplete against the actual Electrum 4.x codebase.** The SPEC §11.6 sniff signature locks `wallet_type ∈ {"standard", "multisig", "2fa", "imported"}`. WebFetch against `https://github.com/spesmilo/electrum/blob/master/electrum/wallet_db.py` returns a wider value set: `"standard"`, `"imported"`, `"old"`, `"xpub"`, `"bip44"`, `"2fa"`, `"trezor"`, `"keepkey"`, `"ledger"`, `"btchip"`. At minimum `"old"`, `"xpub"`, `"bip44"` appear as `wallet_type` strings on legacy-format wallet files. P0A locks this set in §11.6 normatively, which means: (a) the P6A sniff signature will treat a legitimate `wallet_type: "old"` Electrum wallet as `NoMatch`; (b) the P6A test corpus will silently not cover the legacy types; (c) downstream P6B refusal-arm dispatch will not include explicit refusal templates for the legacy types. **Recommended P0A fold:** widen the sniff set to `{"standard", "multisig", "2fa", "imported", "old", "xpub", "bip44"}` AND add explicit refusal arms for `"old"`/`"xpub"`/`"bip44"` (all three are legacy-format wallets ingest cannot reconstruct cleanly — analogous to the `"imported"` refusal). OR explicitly defer the legacy-type sniff to a FOLLOWUP `wallet-import-electrum-legacy-wallet-types`, with §11.6 sniff signature documenting `{standard, multisig, 2fa, imported}` as the v0.28.0 covered set + explicit legacy-set rejection cell at P6A.

**I2 — SPEC §6.1.1 cites `wallet_import/bitcoin_core.rs:62` as the VENDOR_MARKER_KEYS site, but the actual `const VENDOR_MARKER_KEYS:` declaration is at line 74.** The SPEC's `:62` line cites the middle of the doc-comment that *references* SPEC §6.1.1; the const itself is at line 74. This is the recurring "off-by-N" pattern called out at `feedback-r0-must-read-source-off-by-n`. Same drift at SPEC `§12 — Module layout extensions` list. **Fold:** update both citations to `bitcoin_core.rs:74` (the `const` declaration), or use `bitcoin_core.rs:59-92` if the intent is to cite the doc-comment + const block as a whole. The plan-doc itself already drift-acknowledged at Q4 ("note R0 M1 corrected line") but the corrected line was NOT carried forward into the SPEC body.

**I3 — SPEC §6.1.1 `label` marker is documented as "weakly used by Sparrow" but Sparrow's positive sniff in §11.1 does NOT depend on `label`; meanwhile, the v0.26.0 SPEC §6.1.2 contract for VENDOR_MARKER_KEYS is "presence of marker → exclude from Core sniff" — `label` is generic enough (Bitcoin Core blobs CAN legitimately carry `label`-named keys in the future or via wrapper tooling) that adding it to the exclusion list creates a Type-II false-rejection risk for legitimate Core blobs.** If `label` is not load-bearing for Sparrow positive sniff (per §11.1 it isn't — the positive set is `policyType + scriptType + defaultPolicy.miniscript.script + keystores`), then `label` should be REMOVED from VENDOR_MARKER_KEYS, and the Specter positive sniff at §11.2 should rely on the stronger discriminator `blockheight` (Specter's distinctive integer marker) PLUS `devices` (already in the exclusion list). **Fold:** either remove `label` from the exclusion list, OR add a SPEC §6.1.1 footnote pinning a forward-compat contract: "Bitcoin Core blobs that wish to carry a top-level `label` key are out-of-scope at v0.28.0; users with such blobs must override sniff via `--format bitcoin-core` per v0.26.0 §6.2 explicit-override contract."

**I4 — SPEC §11.5 Jade sniff signature claims `multisig_file` is the marker; the SPEC §6.1.1 doc-comment + VENDOR_MARKER_KEYS list `register_multisig` AND `multisig_file` as Jade markers.** Re-reading §11.5: "top-level JSON object with a top-level `multisig_file` field" + the Q1-lock note: "v0.28.0 jade.rs handles only `register_multisig.multisig_file` JSON shape". So the actual on-disk Jade shape is `{ "register_multisig": { "multisig_file": "..." } }` — a NESTED `multisig_file` inside `register_multisig`, not a top-level `multisig_file`. The SPEC text in §11.5 says "top-level `multisig_file` field" which contradicts both the Q1 lock note within the same section AND the `register_multisig` vendor-marker entry in §6.1.1. **Fold:** rewrite §11.5 sniff signature to: "top-level JSON object with a `register_multisig` field whose value is a JSON object containing a `multisig_file` string field." Then VENDOR_MARKER_KEYS check on `register_multisig` (already present) is load-bearing; `multisig_file` at the JSON top-level is NOT a real Jade marker and should be REMOVED from VENDOR_MARKER_KEYS unless an empirical fixture justifies it.

## Minor (fold inline or defer to cycle-followups tracker)

**M1** — SPEC §10.3 6-line shape `derivation_path: <Line4>` is correct against `bsms.rs:108`. Verified clean.

**M2** — SPEC §10.5 cites `wallet_import/roundtrip.rs:87` correctly. Verified.

**M3** — SPEC §10.4 DEPRECATION notice text uses "in a future minor version" but plan-doc §S.7 uses "in v0.29.0" (specific version). Plan-doc-vs-SPEC drift. **Fold:** prefer the SPEC's "future minor version" wording (less brittle); update plan-doc §S.7 to match SPEC §10.4. OR explicitly tag this as a P7B-time decision and add a SPEC §10.4 footnote.

**M4** — SPEC §6.3 specifies the dispatch body verbatim including variable-declaration order and the votes array structure. This locks implementation form before P0D's R0 has run. The SPEC explicitly notes the section is P0D-scope. Including it in P0A's SPEC scaffolding is consistent with the SPEC's role as a normative artifact downstream phases cite. **No fold; gray area between "scaffolding" and "P0D pre-commit."** Recommend P0D R0 explicitly re-verify §6.3 is implementable as written without bool-shadowing or borrow-checker friction.

**M5** — SPEC §10.3 deferred design decision wording ("R5-I4 noted; deferred final design decision to execution-time") is unusual for a normative SPEC body. **Fold:** rewrite §10.3 paragraph 4 to drop "deferred final design decision to execution-time" and adopt straight normative voice: "Phase 7's 4-line shape uses the empty-string-sentinel pattern for `token` and `signature`. R5-I4's alternative (introducing a `BsmsLineShape` discriminator field) is filed as a v0.28+ FOLLOWUP `bsms-line-shape-discriminator-cleanup` to be opened only if the empty-string-sentinel pattern surfaces foot-guns in user-visible JSON envelopes."

**M6** — VENDOR_MARKER_KEYS list order: the v0.26.0 originals are listed in source order; v0.28.0 additions are NOT alphabetical. Per CLAUDE.md's alphabetical-by-variant-name discipline for new enum variants + matches, this is arguably not in scope (it's a const string array, not an enum variant or match arm), but the underlying rationale applies. The plan-doc itself wisely groups by parser (per-format banding) which is a different but valid grouping. **No hard fold; tag for FOLLOWUP** `vendor-marker-keys-ordering-discipline` if useful.

**M7** — SPEC §1.4 references `SPEC_mnemonic_toolkit_v0_5.md` §14 and `SPEC_wallet_export_v0_8.md` §9 as anchors. Citation not verified at this round. Recommend P0A author confirms these section numbers are correct at the cited SPEC heads. Not blocking P0A merge.

**M8** — SPEC §6.2 alphabetical claim: `Coldcard < ColdcardMultisig` because the prefix-match rule for alphabetical ordering treats shorter strings as preceding longer. Correct (Rust string ordering is lexicographic). Not a finding — just confirmation.

## Source-grep re-verification table

| Citation | Verified? | Status |
|---|---|---|
| BIP-129 line-3 canonical name `path-restrictions` per recon dossier | YES | The SPEC's hyphenated `path-restrictions` is a stylized name; defensible |
| Electrum `wallet_type` value set `{standard, multisig, 2fa, imported}` | PARTIAL — set is incomplete | See Important I1 |
| Electrum FINAL_SEED_VERSION = 71 | YES (WebFetch confirms) | |
| `wallet_import/bitcoin_core.rs:62` VENDOR_MARKER_KEYS site | NO — actual const at line 74 | See Important I2 |
| `derive_first_address` at `derive_address.rs:26` pub(crate) | YES | |
| `derive_first_address` consumed at `wallet_export/bsms.rs:36, 104` and `wallet_import/bsms.rs:225` | YES | |
| `IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION = "1"` at `cmd/import_wallet.rs:68` | YES | |
| `--format` PossibleValuesParser at `cmd/import_wallet.rs:88` | YES | |
| `SniffOutcome` enum at `sniff.rs:33-38` (current non-alphabetical) | YES | |
| `sniff_format` dispatch at `sniff.rs:43-52` (2-bool 2×2 match) | YES | |
| Truth-table test at `sniff.rs:150-186` | YES | |
| `ImportProvenance` enum at `mod.rs:63-71` (non-alphabetical Bsms before BitcoinCore) | YES | |
| `bsms.rs:131` error template `"expected 2 or 6 lines"` | YES | |
| `roundtrip.rs:87` canonicalize_bsms mirror | YES | |
| `canonicalize_bsms` at `roundtrip.rs:39`, `canonicalize_bitcoin_core` at `roundtrip.rs:117` | YES | |
| `wallet_import/bsms.rs:111-117` 6-line stderr notice | YES | |

## Scope-creep audit

| Item | In plan-doc P0A scope? | Acceptable? |
|---|---|---|
| §1.4 namespace disambiguation | YES | YES |
| §2.1 CLI `--format` value-set lock | NO (P0A row says §2.2 only) | Acceptable — §2.1 is a documentary table |
| §2.2 schema_version stay-at-"1" | YES | YES |
| §6.1 sniff semantic carry-forward | YES | YES |
| §6.1.1 VENDOR_MARKER_KEYS expansion | YES | YES — implementation matches SPEC verbatim |
| §6.2 alphabetical SniffOutcome final order | YES | YES |
| §6.3 `sniff_format` dispatch body (NEW per P0D) | NO (P0D scope) | Marginal — see Minor M4 |
| §10 BIP-129 4-line parser (full §10.1-§10.6) | PARTIAL (row says "line-3 canonical name") | Defensible as normative anchor downstream phases cite |
| §11.1-§11.6 per-parser SPECs | PARTIAL — P0A row says "§11.x per-parser provenance schemas" only | Acceptable per §S.1-§S.6 collation framing |
| §12 module layout extensions | NO (not in P0A row but logically required) | Acceptable |
| `VENDOR_MARKER_KEYS` expansion at bitcoin_core.rs | YES | YES |

**Net scope assessment:** the SPEC is broader than the strict P0A row "research-and-lock" scope, but the breadth is consistent with the plan-doc's framing of the SPEC as a normative artifact downstream phases cite. The included sections that are downstream-phase-scope are self-tagged ("NEW per Phase P7", "NEW per P0D"), preserving auditability. No hard scope-creep.

## Overall verdict

**YELLOW.**

Two P0A deliverables structurally sound — VENDOR_MARKER_KEYS expansion correct + cleanly implemented, BIP-129 line-3 nomenclature verified, SPEC carry-forward appropriately scoped, schema_version "1" rationale defensible. No execution-blocker for P0B.1.

But 4 Important findings will compound across downstream phases if not addressed:
- I1 (Electrum wallet_type set) locks a SPEC normative incomplete against real Electrum codebase
- I2 (bitcoin_core.rs:62 vs :74) is citation drift, recurring off-by-N pattern
- I3 (`label` marker) introduces forward-incompatible exclusion for legitimate Core blobs
- I4 (Jade `multisig_file` vs `register_multisig.multisig_file`) has internal SPEC self-contradiction

### R1 fold recommendations

1. **I1 fold (load-bearing):** widen §11.6 wallet_type set to `{standard, multisig, 2fa, imported, old, xpub, bip44}` AND add explicit refusal arms + stderr templates for `old`/`xpub`/`bip44`, OR file FOLLOWUP `wallet-import-electrum-legacy-wallet-types` and explicitly limit v0.28.0 sniff coverage to the 4-value set with explicit legacy-type rejection-cell coverage at P6A.

2. **I2 fold (citation drift):** update SPEC §6.1.1 + §12 citations from `bitcoin_core.rs:62` to `bitcoin_core.rs:74` (the `const VENDOR_MARKER_KEYS:` declaration site).

3. **I3 fold (false-positive risk):** remove `label` from VENDOR_MARKER_KEYS in `bitcoin_core.rs` AND SPEC §6.1.1, AND adjust SPEC §11.2 Specter positive sniff to rely on `blockheight` + `devices` only.

4. **I4 fold (Jade marker correctness):** rewrite §11.5 sniff signature to require nested `register_multisig.multisig_file`, NOT top-level `multisig_file`. Remove `multisig_file` from VENDOR_MARKER_KEYS if no fixture justifies it; retain `register_multisig` as the sole Jade vendor-marker.

After R1 folds, expect 0C/0I and GREEN for P0A merge.

---

**Sources:**
- [electrum/wallet_db.py at master · spesmilo/electrum (GitHub)](https://github.com/spesmilo/electrum/blob/master/electrum/wallet_db.py)
- [electrum/wallet.py at master · spesmilo/electrum (GitHub)](https://github.com/spesmilo/electrum/blob/master/electrum/wallet.py)
- [BIP-129 BSMS specification (GitHub)](https://github.com/bitcoin/bips/blob/master/bip-0129.mediawiki)
