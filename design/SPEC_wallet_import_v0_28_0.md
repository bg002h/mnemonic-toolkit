# SPEC — `mnemonic import-wallet` (v0.28.0)

**Status:** Phase P0A scaffolding — author + R0 reviewer-loop.
**Cycle:** v0.28.0 (toolkit minor bump + `mnemonic-gui` v0.12.x lockstep).
**Predecessor:** v0.27.2 (`ec04a00`, 2026-05-19).
**Supersedes:** [`SPEC_wallet_import_v0_26_0.md`](SPEC_wallet_import_v0_26_0.md). Carries forward §1-§9 of v0.26.0 unchanged EXCEPT the targeted amendments in §1.4, §2.2, §6.1, §6.1.1, §6.2 listed in §A below. Adds new §10, §11.
**Plan-doc:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` (R6 GREEN; locked 2026-05-19).
**External authorities:**
- [BIP-129 BSMS specification](https://github.com/bitcoin/bips/blob/master/bip-0129.mediawiki) (canonical 4-line Round-2 shape; §10 of this SPEC)
- [BIP-380 Output Script Descriptors](https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki)
- [BIP-389 Multipath descriptors](https://github.com/bitcoin/bips/blob/master/bip-0389.mediawiki)
- BSMS 4-line shape recon: [`design/agent-reports/v0_27_0-phase-2-bip129-recon.md`](agent-reports/v0_27_0-phase-2-bip129-recon.md) (cites BIP-129 §Specification verbatim; locks line-3 nomenclature as `path-restrictions` per BIP-129 line 96)
- Per-parser vendor schemas: cited inline at §11.x

---

## §A — Summary of changes vs v0.26.0

This SPEC carries forward all of `SPEC_wallet_import_v0_26_0.md` UNCHANGED except the following amendments + new sections. Where this SPEC is silent on a topic, the v0.26.0 SPEC governs.

| Change | Section(s) | Origin |
|---|---|---|
| NEW §1.4 — namespace disambiguation (`src/electrum.rs` vs `wallet_export/electrum.rs` vs `wallet_import/electrum.rs`) | §1.4 (new) | Plan-doc Q5 lock |
| NEW §2.1 — `--format` flag value-set expanded from `{bsms, bitcoin-core}` to 8 values | §2.1 (amended) | Plan-doc P1C-P6C |
| NEW §2.2 — envelope `schema_version` stays at `"1"`; `source_format` is open-set | §2.2 (amended) | Plan-doc R0 I3 lock |
| AMENDED §6.1 — sniff semantic LOCKED: all-parsers-consulted; ≥2-match→Ambiguous | §6.1 (clarified-not-changed) | Plan-doc R0 I4 lock |
| NEW §6.1.1 — `VENDOR_MARKER_KEYS` exclusion list expanded with 8 new format markers (5 originals + 8 additions = 13 entries; R1 I3/I4 folds removed 2 prior candidates) | §6.1.1 (new) | Plan-doc P0A scope + Q4 lock + R1 I3+I4 folds |
| NEW §6.2 — `SniffOutcome` enum alphabetical-variant-order | §6.2 (amended) | Plan-doc P0B.1 + R1-C1/R2 locks |
| NEW §6.3 — `sniff_format` dispatch-shape: consult-all-then-count for 8 parsers (replaces v0.26.0's 2-bool 2×2 match) | §6.3 (new) | Plan-doc P0D + R3-C2/R4-I1/R4-I3 locks |
| NEW §10 — BIP-129 4-line Round-2 parser; line-3 canonical name `path-restrictions` (per BIP-129 line 96) | §10 (new) | Plan-doc §S.7 + R0 I6 lock |
| NEW §11 — Per-parser sniff signatures + provenance schemas + CLI surface for Sparrow / Specter / Coldcard / Coldcard-multisig / Jade / Electrum | §11.1-§11.6 (new) | Plan-doc §S.1-§S.6 collated |

The above lock the foundation for Phases P0B.1, P0B.2, P0C, P0D (Wave 0 remaining sub-phases) + Phases P1-P6 (per-parser Wave 1 instances) + Phase P7 (BSMS 4-line) + Phase P11 (cross-format matrix). All downstream code MUST cite this SPEC for normative anchors; v0.26.0 SPEC governs only where this SPEC is silent.

---

## §1.4 — Namespace disambiguation (NEW)

Three distinct `electrum`-named modules coexist in the toolkit; reviewers and contributors MUST distinguish them. Confusion between these is a recurring foot-gun called out at cycle plan-doc R0 / recon dossier §"cross-cutting #6":

| Path | Role | Touched in v0.28.0? |
|---|---|---|
| `crates/mnemonic-toolkit/src/electrum.rs` | Electrum **native-seed-format** codec (HMAC-SHA512 prefix dispatch + per-wordlist base-N mapping). SPEC anchor: `SPEC_mnemonic_toolkit_v0_5.md` §14. | NO (unchanged) |
| `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` | Electrum **wallet-file emit** (writes the JSON wallet shape Electrum 4.x consumes via "Import Wallet"). SPEC anchor: `SPEC_wallet_export_v0_8.md` §9. | NO (unchanged) |
| `crates/mnemonic-toolkit/src/wallet_import/electrum.rs` | Electrum **wallet-file ingest** (parses the same JSON wallet shape, inverse of wallet_export emit). SPEC anchor: §11.6 of this SPEC. | **YES — NEW** in Phase P6 |

Per Q5 lock: the new ingest module is named `wallet_import/electrum.rs` (parent module `wallet_import::` disambiguates from the other two surfaces; no `_wallet` suffix or other naming distinguisher). Future contributors confused about which module to edit should consult this §1.4 BEFORE making changes.

---

## §2.1 — CLI `--format` value-set (AMENDED)

The `--format` flag at `cmd/import_wallet.rs:88` value-parser accepts the following alphabetically-sorted values in v0.28.0:

```
[bitcoin-core, bsms, coldcard, coldcard-multisig, electrum, jade, sparrow, specter]
```

Adding to v0.26.0's `{bsms, bitcoin-core}`: 6 new parsers per Phases P1-P6. Each new value's parse semantics + provenance + sniff signature is normatively locked in §11.x of this SPEC.

The CLI flag remains optional — sniff auto-dispatch (per §6 semantic) handles the no-flag case. Explicit-format override behavior unchanged from v0.26.0 §2.1.

---

## §2.2 — Envelope `schema_version` cutover decision (NEW)

**Lock (R0 I3 fold):** `IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION` at `cmd/import_wallet.rs:68` STAYS at `"1"` for v0.28.0. The schema version is NOT bumped despite the addition of 6 new `source_format` values.

**Rationale:** `source_format` is an open-set string enumeration. v0.26.0 specified `{"bsms", "bitcoin-core"}`; v0.28.0 extends to 8 values per §2.1. Consumers parsing v0.26.0 envelopes via tolerant `serde_json::Value` semantics (string-equality discrimination, not exhaustive enum-match) transparently absorb the new values. mnemonic-gui's `schema_mirror` consumer is the canonical such consumer; its post-Phase-P15 update extends the dropdown enumeration via the schema-mirror lockstep PR.

**Forward-compat contract:** consumers MAY exhaustively match `source_format` on a known-closed set IF they accept that unknown future values are treated as "unrecognized" — error class downstream-specific. The toolkit guarantees: (a) `source_format` is always a non-empty ASCII string of `[a-z-]+`; (b) the v0.28.0 enumeration grows monotonically across minor releases; (c) wire-shape break (with `schema_version` bump) is announced via CHANGELOG `### Changed` and FOLLOWUP entry citing the bump.

When the wire shape itself changes (e.g., new top-level fields not absorbable by additive-tolerance), the schema_version bumps to `"2"` in a future cycle; that's NOT v0.28.0's scope.

---

## §6.1 — Sniff heuristic (CLARIFIED — NOT CHANGED)

The dispatch semantic in v0.28.0 is **identical to v0.26.0 §6**, just extended over more parsers:

1. All N parser sniff functions (N=2 in v0.26.0; N=8 in v0.28.0) are consulted unconditionally on every blob.
2. Each returns `bool` independently.
3. Aggregate outcome computed by counting `true` returns:
   - 0 `true` → `SniffOutcome::NoMatch`
   - exactly 1 `true` → `SniffOutcome::<that_parser_variant>`
   - ≥2 `true` → `SniffOutcome::Ambiguous`

**Lock (R0 I4 fold):** the precedence ordering in plan-doc §B.2 #4 ("BSMS prefix → JSON-shape parsers in vendor-marker specificity order → text-format parsers → ...") is **documentary guidance for sniff-signature design only, NOT load-bearing dispatch logic**. Sniff signatures SHOULD be engineered such that parsers do not co-fire on the same blob (via vendor-marker discipline at §6.1.1) — but when a contrived blob does match multiple sniffs, the outcome is `Ambiguous` (not "first-match-wins"). This preserves the v0.26.0 contract exactly.

---

## §6.1.1 — Vendor-marker exclusion list (NEW)

**Lock (R0 Q4 + R1 I3/I4 + R2 N1 folds):** the Bitcoin Core sniff's vendor-marker exclusion list — the `VENDOR_MARKER_KEYS` `const` at `wallet_import/bitcoin_core.rs:81` (R2 N1 citation fix: const declaration sits at `:81` post-R1-doc-comment-expansion; lines `:59-80` are the doc-comment) — expands from 5 to 13 entries to absorb the new format markers introduced by Phases P1-P6:

```rust
const VENDOR_MARKER_KEYS: &[&str] = &[
    // v0.26.0 originals (Bitcoin Core / generic-vendor exclusion):
    "chain", "policy", "version", "bipname", "extendedPublicKey",
    // v0.28.0 P0A additions (per format vendor markers):
    "seed_version",       // Electrum wallet (§11.6)
    "wallet_type",        // Electrum wallet (§11.6)
    "policyType",         // Sparrow Wallet (§11.1)
    "defaultPolicy",      // Sparrow Wallet (§11.1)
    "keystores",          // Sparrow Wallet (§11.1)
    "devices",            // Specter (§11.2)
    "blockheight",        // Specter (§11.2; integer marker — strong disambiguator)
    "multisig_file",      // Jade (§11.5; top-level reply field from `get_registered_multisig` RPC)
];
```

**R1 I3 fold — `label` deliberately omitted.** The original P0A scope included `"label"` as a Specter marker. Removed in R1 because (a) `label` is generic enough that a legitimate Core blob carrying a top-level `label` key should not be excluded; (b) Specter is still strongly disambiguated by `blockheight` (integer) + `devices` (array) + `descriptor` (string) per §11.2 positive sniff. The Specter positive sniff still requires `label`, but its absence from the EXCLUSION list does not weaken Specter discrimination because `blockheight` is the load-bearing exclusion marker.

**R1 I4 fold — `register_multisig` deliberately omitted.** The original P0A scope included `"register_multisig"` as a Jade marker. Removed in R1 because: per Blockstream/Jade docs (`https://github.com/Blockstream/Jade/blob/master/docs/index.rst`), `register_multisig` is an RPC **command name**, not a JSON field present in any on-disk export. The actual top-level Jade export marker is `multisig_file` (the reply field of the `get_registered_multisig` RPC). The plan-doc's `register_multisig.multisig_file` notation referred to **command.reply-field**, not JSON nesting. Only `multisig_file` is retained as the load-bearing Jade vendor-marker.

When any of these keys appears at the JSON top level, the Bitcoin Core sniff returns `false` (the blob is excluded from Bitcoin Core's positive-sniff territory and falls through to the other parsers). Each non-Core parser's sniff in §11.x conducts its own positive-marker check.

**The exclusion list is conservative by design**: a legitimate Bitcoin Core blob that happens to contain `chain` as a custom key would also be excluded. The v0.26.0 SPEC §6.1 documents this trade-off; v0.28.0 inherits it unchanged.

---

## §6.2 — `SniffOutcome` enum (AMENDED)

**Lock (P0B.1 + per-parser P1A-P6A insertions):** `SniffOutcome` enum at `wallet_import/sniff.rs:33-38` reorders to alphabetical variant order, then per-parser sub-phases insert their new variants at the alphabetically-correct position. Final v0.28.0 order (post-all-phases):

```rust
pub(crate) enum SniffOutcome {
    Ambiguous,
    BitcoinCore,
    Bsms,
    Coldcard,
    ColdcardMultisig,
    Electrum,
    Jade,
    NoMatch,
    Sparrow,
    Specter,
}
```

Variant ordering has no semantic impact (exhaustive matches everywhere). Alphabetical discipline matches CLAUDE.md's `ToolkitError` ordering rule + prevents merge-conflict-generators during concurrent feature PRs (per recon dossier cross-cutting #5).

---

## §6.3 — `sniff_format` dispatch body shape (NEW per P0D)

**Lock (R3-C2 + R4-I1 + R4-I3 folds):** the dispatch body at `wallet_import/sniff.rs:43-52` rewrites from v0.26.0's 2-bool 2×2 truth-table match to a **consult-all-then-count** pattern handling 8 parsers in v0.28.0. The required shape (Phase P0D scope):

```rust
pub(crate) fn sniff_format(blob: &[u8]) -> SniffOutcome {
    let bitcoin_core = BitcoinCoreParser::sniff(blob);
    let bsms = BsmsParser::sniff(blob);
    let coldcard = false;            // P3A: replace with ColdcardParser::sniff(blob)
    let coldcard_multisig = false;   // P4A: replace with ColdcardMultisigParser::sniff(blob)
    let electrum = false;            // P6A: replace with ElectrumParser::sniff(blob)
    let jade = false;                // P5A: replace with JadeParser::sniff(blob)
    let sparrow = false;             // P1A: replace with SparrowParser::sniff(blob)
    let specter = false;             // P2A: replace with SpecterParser::sniff(blob)

    let votes: [(bool, SniffOutcome); 8] = [
        (bitcoin_core, SniffOutcome::BitcoinCore),
        (bsms, SniffOutcome::Bsms),
        (coldcard, SniffOutcome::Coldcard),
        (coldcard_multisig, SniffOutcome::ColdcardMultisig),
        (electrum, SniffOutcome::Electrum),
        (jade, SniffOutcome::Jade),
        (sparrow, SniffOutcome::Sparrow),
        (specter, SniffOutcome::Specter),
    ];

    let matched: Vec<SniffOutcome> = votes.iter().filter(|(b, _)| *b).map(|(_, v)| *v).collect();
    match matched.len() {
        0 => SniffOutcome::NoMatch,
        1 => matched[0],
        _ => SniffOutcome::Ambiguous,
    }
}
```

**Normative requirements (lock):**

1. **Bool-variable declarations enumerated in alphabetical PARSER order** (R4-I3). Note that `Ambiguous` and `NoMatch` are NOT parsers — they are aggregate outcomes — and do NOT appear in the votes array.
2. **All 8 bools are read in the votes array** to avoid `unused_variables` warnings on the placeholder `false` bools (R4-I1).
3. **Per-parser P{N}A sub-phases flip ONE bool** from `let <name> = false;` to `let <name> = <Parser>::sniff(blob);` — single-line edit at a known location.
4. **The truth-table behavior is preserved exactly:** 0→NoMatch, exactly-1→that parser's variant, ≥2→Ambiguous. Verified by the equivalence-class-coverage test at §6.3.1.

### §6.3.1 — Truth-table regression test (Phase P0D)

The embedded test at `wallet_import/sniff.rs:150-186` (`sniff_format_dispatches_ambiguous_when_both_parsers_match`) currently uses a 2×2 inline match literal mirroring the v0.26.0 dispatch shape. **P0D rewrites this test** to assert equivalence-class coverage on 8-bool synthetic tuples:

```rust
#[test]
fn sniff_format_dispatches_consult_all_then_count() {
    // Equivalence-class regression: the consult-all-then-count semantic at
    // sniff_format collapses the 2^8 = 256-row truth table into 3 classes.
    // This test pins one representative per class.
    //
    // v0.27.1 Phase 4 I16 fold's exhaustive-2×2 documentation generalizes
    // to: (a) 0-true equivalence class → NoMatch; (b) exactly-1-true
    // equivalence class (8 representatives, one per parser) → that
    // parser's variant; (c) ≥2-true equivalence class (1+ representative)
    // → Ambiguous. The class collapse is by construction of the dispatch.
    //
    // (Equivalence-class assertions enumerated here; one representative per
    // class. Per-parser sub-phases extend this with fixture-based tests
    // exercising real sniff functions.)
    // ...
}
```

Per Phase P0D scope: the test asserts at MINIMUM (a) one 0-true tuple → NoMatch; (b) at least one 1-true tuple per parser-position → that parser's variant (8 cells); (c) at least one ≥2-true tuple → Ambiguous. Renamed from `sniff_format_dispatches_ambiguous_when_both_parsers_match` to `sniff_format_dispatches_consult_all_then_count` to reflect the new semantic.

The 256-row exhaustive truth table is NOT enumerated; the equivalence-class collapse is the documented coverage.

---

## §10 — BIP-129 4-line Round-2 parser (NEW per Phase P7)

**Lock (R0 I6 fold):** Line-3 canonical name is `path-restrictions` per BIP-129 line 96 (verbatim from BIP-129 §Specification → Round 2, as cited at `design/agent-reports/v0_27_0-phase-2-bip129-recon.md:26-30`). The v0.26.0 SPEC §4.2 line 152 used the term `derivation_path` — that wording is **inaccurate against BIP-129's canonical naming** and is corrected to `path-restrictions` in this SPEC §10.

### §10.1 — 4-line parse shape

`BsmsParser::parse` at `wallet_import/bsms.rs::parse` extends its line-count match (existing `2 =>` at line 97 and `6 =>` at line 105) with a new `4 =>` arm per the BIP-129 canonical Round-2 record. The accepted shape:

```
Line 1: BSMS_VERSION       (literal "BSMS 1.0")
Line 2: DESCRIPTOR         (descriptor body; optional #checksum suffix per BIP-380)
Line 3: PATH-RESTRICTIONS  (e.g., "/0/*,/1/*" — BIP-129 line 96's term)
Line 4: FIRST_ADDRESS      (e.g., "bc1q...")
```

CRLF normalization applies (same as v0.26.0 2-line / 6-line shapes). Trailing whitespace per line is stripped. Empty trailing lines are tolerated (handled by existing `strip_trailing_empty` helper).

### §10.2 — Cross-validation contract

The 4-line parse arm performs a **first-address cross-validation** before returning the parsed bundle:

1. Parse Line 2 as a descriptor via `MsDescriptor::from_str` (with BIP-380 checksum if present).
2. Derive the first receive address at path `/0/0` of the descriptor's first path-restriction branch, using existing helper `crate::derive_address::derive_first_address` at `derive_address.rs:26` (`pub(crate)`; already consumed by `wallet_export/bsms.rs:36, 104` and `wallet_import/bsms.rs:225`).
3. Compare byte-exact against Line 4's supplied FIRST_ADDRESS.
4. Mismatch → stderr WARNING `warning: import-wallet: bsms: first-address mismatch at path <P>: computed <X>, blob declares <Y>` (exit 0); **parse continues**. **R-W1-end I1 fold:** the 4-line path reuses the existing v0.26.0 §2.4 `bsms: first-address mismatch` WARNING semantic (informational, not refusal). Rationale: matches the 6-line precedent + BIP-129 §6 coordinator-output self-consistency intent (coordinator emits both DESCRIPTOR + FIRST_ADDRESS for cross-check transparency; mismatch should be visible without being a hard failure). Earlier drafts of this SPEC framed it as `ImportWalletParse` exit 2; the implementation followed the established v0.26.0 WARNING-only behavior. Strict-mismatch-error remains a candidate for a future cycle if user demand surfaces; see §10.6.
5. Match → continue normal parse.

Taproot descriptors are SKIPPED for cross-validation (BIP-129 §1 prerequisites pre-date BIP-386; first-address derivation surface for tr() requires additional taproot-context infrastructure). This is consistent with v0.27.0's existing taproot-skip discipline at `wallet_import/bsms.rs:217-225`.

### §10.3 — Provenance representation

The existing `ImportProvenance::Bsms(Option<BsmsAuditFields>)` variant accommodates the 4-line shape via the empty-string-sentinel pattern (R5-I4 noted; deferred final design decision to execution-time):

- 2-line shape → `Bsms(None)` (unchanged)
- 4-line shape (NEW) → `Bsms(Some(BsmsAuditFields { token: "", signature: "", first_address: <Line4>, derivation_path: <Line3>, verification: BsmsVerification::NotAttempted }))`
- 6-line shape → `Bsms(Some(BsmsAuditFields { token: <Line2>, signature: <Line6>, first_address: <Line5>, derivation_path: <Line4>, verification: NotAttempted }))` (unchanged)

**Downstream-discriminator contract:** envelope consumers that need to distinguish 4-line vs 6-line provenance MAY do so by checking `audit.token.is_empty() && audit.signature.is_empty()` (4-line) vs both-non-empty (6-line). R5-I4's alternative design (introduce a `BsmsLineShape` discriminator field) is deferred to a v0.28+ FOLLOWUP if the empty-string-sentinel pattern proves to be a foot-gun in practice.

**Field-name note:** the `derivation_path` field name on `BsmsAuditFields` is a v0.26.0 legacy holdover; it carries the BIP-129 `path-restrictions` value for both 4-line and 6-line shapes. The field-rename to `path_restrictions` is a wire-shape change deferred to a future minor cycle (would break envelope-consumer compat).

### §10.4 — 6-line shape DEPRECATION notice (Phase P7B)

The existing stderr notice at `wallet_import/bsms.rs:111-117` (currently "2/6-line parser does not verify signature inline; supply --bsms-round1") is REPLACED in Phase P7B with a DEPRECATION-class notice:

```
notice: import-wallet: bsms: 6-line lenient shape is DEPRECATED in v0.28+ and
will be removed in a future minor version; convert your blob to the BIP-129-
canonical 4-line shape (BSMS_VERSION + DESCRIPTOR + path-restrictions +
FIRST_ADDRESS) for forward compatibility. See SPEC §10 for the canonical shape.
```

The 6-line PARSING behavior is preserved at v0.28.0 (lenient parse continues to accept the legacy shape; signature audit metadata continues to populate `BsmsAuditFields`). Only the stderr message text changes.

### §10.5 — Error template update

The `other =>` error template at `wallet_import/bsms.rs:131` updates from `"expected 2 or 6 lines"` to `"expected 2, 4, or 6 lines"`. The `canonicalize_bsms` mirror at `wallet_import/roundtrip.rs:87` updates correspondingly (per R5-C2 lock). All test-file occurrences of the old literal MUST update in lockstep per the comprehensive grep-sweep discipline in plan-doc §"Verification — string-literal sweep before any literal change".

### §10.6 — Scope limitations

This SPEC §10 implements ONLY sub-item (b) of the canonical `bsms-bip129-full-cutover` FOLLOWUP (4-line input parser). Deferred to v0.28+:

- **(c) STANDARD/EXTENDED encryption envelope** (PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256). Tracked at NEW FOLLOWUP `bsms-bip129-encryption-envelope` filed at Phase P14A.
- **(d) Drop 2-line / 6-line shapes after deprecation window.** Deferred indefinitely; deprecation window opens with v0.28.0 (notice firing) and closes in a future cycle when full BIP-129 conformance ships.
- **Real BSMS Round-2 taproot emit** — tracked at canonical `bsms-taproot-emit` FOLLOWUP (upstream-blocked on BIP-129 §1 prerequisites adding BIP-386). Phase P8 ships only refusal-scaffold UX improvements.
- **Strict first-address-mismatch refusal (exit 2)** — v0.28.0 ships the WARNING-only semantic per §10.2 step 4 (matching v0.26.0 §2.4 6-line precedent + BIP-129 §6 coordinator-output self-consistency intent). A future cycle MAY introduce a `--strict-first-address` flag or equivalent opt-in to escalate mismatch to `ImportWalletParse` exit 2 if user demand surfaces. Tracking deferred until demand signal; no v0.28+ FOLLOWUP filed pre-emptively per the "avoid speculative-feature backlog" discipline.

---

## §11 — Per-parser SPECs (NEW per Phases P1-P6)

This section collates the per-parser SPEC content from plan-doc §S.1-§S.6. Each §11.N normatively locks the sniff signature, parse contract, provenance schema, and CLI surface for one new parser. Per-parser implementation is Phase P{N}A/B/C scope.

### §11.1 — Sparrow Wallet (Phase P1)

**Sniff signature:** top-level JSON object containing all of:
- `policyType` ∈ {`"SINGLE"`, `"MULTI"`}
- `scriptType` (string)
- `defaultPolicy.miniscript.script` (nested string)
- `keystores` (non-empty array)

Vendor markers are sufficient to disambiguate Sparrow from Bitcoin Core / Specter / other JSON formats. Sniff is positive-marker-based; no false-positive co-fire risk with other §11 parsers.

**Parse contract:** Decode `keystores[i]` → cosigners (extract `xpub`, derivation path, optional label). Extract descriptor from `defaultPolicy.miniscript.script` (Sparrow's stored form is miniscript; convert to descriptor via standard `wsh(...)` / `sh(wsh(...))` wrapping based on `scriptType`).

**Provenance:** `ImportProvenance::Sparrow(SparrowSourceMetadata)`:

```rust
pub(crate) struct SparrowSourceMetadata {
    pub label: Option<String>,             // top-level "label" if present
    pub policy_type: SparrowPolicyType,    // Single | Multi
    pub script_type: String,               // verbatim "P2WSH" / "P2WPKH" / etc.
    pub dropped_fields: Vec<String>,       // analogous to CoreSourceMetadata
}

pub(crate) enum SparrowPolicyType {
    Single,
    Multi,
}
```

**CLI:** `--format sparrow` added to `cmd/import_wallet.rs:88` PossibleValuesParser.

**Canonicalize helper:** new `pub(crate) fn canonicalize_sparrow(blob: &[u8]) -> Result<String, ToolkitError>` in `wallet_import/roundtrip.rs`, mirroring `canonicalize_bsms` at `roundtrip.rs:39` and `canonicalize_bitcoin_core` at `roundtrip.rs:117`.

### §11.2 — Specter-DIY (Phase P2)

**Sniff signature:** top-level JSON object containing all of:
- `label` (string)
- `blockheight` (integer; distinctive Specter marker — no other format uses this top-level field)
- `descriptor` (string)
- `devices` (array)

The `blockheight` integer field is the strongest discriminator (Sparrow doesn't carry it; Bitcoin Core doesn't carry it; etc.).

**Parse contract:** Extract `descriptor` verbatim. Preserve `label` as wallet name. `devices` array becomes per-cosigner provenance hints (vendor-type strings; not load-bearing for descriptor parse).

**Provenance:** `ImportProvenance::Specter(SpecterSourceMetadata)`:

```rust
pub(crate) struct SpecterSourceMetadata {
    pub label: String,
    pub blockheight: u64,
    pub devices: Vec<SpecterDeviceMarker>,
    pub dropped_fields: Vec<String>,
}

pub(crate) struct SpecterDeviceMarker {
    pub device_type: String,  // e.g., "coldcard", "trezor", "unknown"
    pub label: String,
}
```

**CLI:** `--format specter`.

**Canonicalize helper:** new `canonicalize_specter` in `wallet_import/roundtrip.rs`.

### §11.3 — Coldcard single-sig wallet.json (Phase P3)

**Sniff signature (Q3-lock relaxed per R0 I8):** top-level JSON object containing ALL of:
- `chain` ∈ {`"BTC"`, `"XTN"`}
- `xfp` (master fingerprint as 8-char uppercase hex string)
- At-least-one-of: `xpub`, `bip44`, `bip49`, `bip84`, `bip86`, `bip48_1`, `bip48_2`

The disjunction in the third clause absorbs Coldcard firmware variance (different firmware versions emit different combinations of per-BIP derivation blocks).

**Firmware-variance table** (informational; researched + locked at Phase P3A):

| Firmware era | Emits | Discriminator |
|---|---|---|
| Coldcard Mk1/Mk2 (pre-2022) | `xpub` only (single BIP-44) | `xpub` top-level |
| Coldcard Mk3 (2022+) | `bip44`/`bip49`/`bip84` blocks | per-bipN sub-objects |
| Coldcard Mk4 (2023+) | + `bip86` (taproot) | adds `bip86` block |
| Coldcard Q (2024+) | + `bip48_1`/`bip48_2` (multisig hints) | adds `bip48_*` blocks |

**Parse contract:** Extract `chain` → network mapping (BTC → mainnet, XTN → testnet). Extract `xfp` → master fingerprint. Dominant-BIP selection per heuristic at SPEC §11.3.1 below.

**§11.3.1 Dominant-BIP selection:**

Coldcard single-sig exports list multiple BIP-derivation blocks side-by-side. The parser picks ONE dominant block per network heuristic:

1. If `bip86` block present → select BIP-86 (taproot; most modern).
2. Else if `bip84` block present → select BIP-84 (P2WPKH).
3. Else if `bip49` block present → select BIP-49 (P2SH-P2WPKH).
4. Else if `bip44` block present → select BIP-44 (P2PKH).
5. Else if top-level `xpub` present (legacy firmware) → infer BIP from xpub's SLIP-132 prefix (zpub→BIP-84, ypub→BIP-49, xpub→BIP-44).
6. `bip48_1` / `bip48_2` blocks (multisig-context) → IGNORED by single-sig parser; the multisig text file (Phase P4) is the authoritative multisig surface.

**Provenance:** `ImportProvenance::Coldcard(ColdcardSourceMetadata)`:

```rust
pub(crate) struct ColdcardSourceMetadata {
    pub chain: ColdcardChain,           // BTC | XTN
    pub xfp: [u8; 4],                   // master fingerprint
    pub bip_derivation: ColdcardBip,    // Bip44 | Bip49 | Bip84 | Bip86
    pub raw_account: u32,
    pub dropped_fields: Vec<String>,
}

pub(crate) enum ColdcardChain { Btc, Xtn }
pub(crate) enum ColdcardBip { Bip44, Bip49, Bip84, Bip86 }
```

**CLI:** `--format coldcard`.

**Canonicalize helper:** new `canonicalize_coldcard` in `wallet_import/roundtrip.rs`.

### §11.4 — Coldcard multisig text (Phase P4)

**Sniff signature:** **text format (NOT JSON).** Leading lines (in order):
- `Name: <name>`
- `Policy: <K>-of-<N>` (e.g., `2-of-3`)
- `Format: <script-type>` (e.g., `P2WSH`)

Followed by N per-cosigner blocks of:
- `Derivation: m/...`
- `<xpub>` (single-line standalone xpub)

Some firmware variants prefix a `XFP: <hex>` header line; sniff tolerates both (header-present and header-absent).

**Parse contract:** line-oriented parser. Extract Name, Policy (K-of-N), Format (P2WSH / P2SH-P2WSH / P2SH), per-cosigner Derivation+xpub pairs. Synthesize descriptor: `wsh(sortedmulti(K, [xfp/path]xpub, ...))` or `sh(wsh(...))` per Format header.

**§11.4.1 — xfp policy (cycle-13a H14 DEPTH-GATED truth table):**

The supplied `XFP:` (top-level header) / per-cosigner `<XFP>:` prefix is, by Coldcard convention, the **master** (depth-0) fingerprint. `bitcoin::bip32::Xpub::fingerprint()` is the HASH160 of the **current** key, so it equals the master fingerprint **only when `xpub.depth == 0`** (the xpub IS the master). At `depth > 0` the computed value is the account-key's own id — NOT the master — and the master fingerprint is **unrecoverable** from a child xpub (HASH160 is one-way; you cannot ascend the tree). The decoded `xpub.depth` byte is therefore the authoritative discriminator (independent of the declared `Derivation:` path). The table is gated on it:

| `xpub.depth == 0`? | XFP supplied? | computed available? | matches? | action |
|---|---|---|---|---|
| 0 | Y | Y | Y | use supplied (silent) |
| 0 | Y | Y | N | WARNING + use supplied. Byte-exact template: `` warning: import-wallet: coldcard-multisig: xfp header `XFP: <hex>` disagrees with computed fingerprint `<hex>` from cosigner xpub; using blob-supplied header value as authoritative `` |
| 0 | N | Y | — | use computed (= master fp at depth 0) (silent) |
| >0 | Y | (any) | — | use supplied (silent) — the supplied value is the only signal for the master fp; comparing it to the account-key id would emit a guaranteed-spurious warning, so the disagreement warning is **suppressed** at depth>0 and `xfp_header_disagreed` is NOT set |
| >0 | N | Y | — | **`ImportWalletParse` REFUSE** (exit 2): the master fingerprint is unrecoverable from a depth-N account xpub. Message cites the cosigner index, the depth, "master fingerprint", and directs the user to re-export with the device's XFP (a top-level `XFP:` header or a per-cosigner `<XFP>: <xpub>` line) |
| any | Y | N (xpub malformed) | — | use supplied (silent); xpub-parse error surfaces elsewhere via `ImportWalletParse` |
| any | N | N (xpub malformed) | — | `ImportWalletParse` error: `"coldcard-multisig: cannot compute xfp: no XFP header and xpub parse failed: <e>"` |

The `xfp_header_disagreed` WARNING (`xfp_header_disagreed=true`) is gated on `xpub.depth == 0` — it fires only when the computed fp is legitimately comparable to the supplied master XFP.

**Computed-fingerprint formula (corrected, cycle-13a):** the BIP-380 master fingerprint is `bitcoin::bip32::Xpub::fingerprint()` **if and only if `xpub.depth == 0`**. At `depth > 0` the master fingerprint is conveyed ONLY by a supplied XFP (top-level `XFP:` header or a per-cosigner `<XFP>:` line) and is otherwise unrecoverable → **REFUSE**. (The pre-cycle-13a formula — "or on the cosigner xpub itself if depth>0" — was the source of the H14 bug: it conflated the account-key's own id with the master fingerprint a key-origin requires.)

**Provenance:** `ImportProvenance::ColdcardMultisig(ColdcardMultisigSourceMetadata)`:

```rust
pub(crate) struct ColdcardMultisigSourceMetadata {
    pub name: String,
    pub policy: PolicyKOfN,                  // (k: u8, n: u8)
    pub script_format: ColdcardMsFormat,     // P2WSH | P2SH_P2WSH | P2SH
    pub xfp_was_blob_supplied: bool,         // header present
    pub xfp_header_disagreed: bool,          // header present but computed disagrees (WARNING surfaced)
    pub dropped_fields: Vec<String>,
}

pub(crate) struct PolicyKOfN { pub k: u8, pub n: u8 }
pub(crate) enum ColdcardMsFormat { P2wsh, P2shP2wsh, P2sh }
```

**CLI:** `--format coldcard-multisig` (separate from `--format coldcard`; sniff usually auto-detects via JSON-vs-text shape discriminator).

**Canonicalize helper:** new `canonicalize_coldcard_multisig` in `wallet_import/roundtrip.rs`.

### §11.5 — Blockstream Jade (Phase P5)

**Sniff signature:** top-level JSON object with a top-level `multisig_file` field (string containing the inner Coldcard-multisig text shape). The `multisig_file` field is the distinctive marker — no other format uses it.

**On-disk shape clarification (R1 I4 fold):** Jade's `register_multisig` is the RPC command name in the Jade firmware API. The `get_registered_multisig` RPC reply carries a top-level `multisig_file` field whose value is the same flat-file text format Coldcard's multisig export produces. Per Blockstream/Jade docs (`https://github.com/Blockstream/Jade/blob/master/docs/index.rst`), the export shape is:
```json
{
  "id": "<request-id>",
  "multisig_name": "<wallet-name>",
  "multisig_file": "Name: …\nPolicy: …\nFormat: …\nDerivation: …\n\n<xfp>: <xpub>\n…"
}
```
The `multisig_file` field at the JSON top level is the load-bearing v0.28.0 sniff marker (per §6.1.1).

**Q1 lock:** SeedQR variant (`register_multisig` RPC + `seedqr` reply field, exact shape pending field-research at Phase P14A) is DEFERRED. v0.28.0 jade.rs handles only the `get_registered_multisig`-reply JSON shape (top-level `multisig_file` field). New FOLLOWUP `wallet-import-jade-seedqr` filed at Phase P14A.

**Parse contract:** Extract `multisig_file` field. Delegate to `coldcard_multisig::parse_text(&inner_text)` (per §11.4). Annotate provenance as Jade rather than Coldcard.

**Provenance:** `ImportProvenance::Jade(JadeSourceMetadata)`:

```rust
pub(crate) struct JadeSourceMetadata {
    pub coldcard_compat: ColdcardMultisigSourceMetadata,
    pub jade_specific_fields: Vec<String>,  // empty for now; future-proof for SeedQR
}
```

**CLI:** `--format jade`.

**Canonicalize helper:** new `canonicalize_jade` in `wallet_import/roundtrip.rs`.

### §11.6 — Electrum 4.x wallet file (Phase P6)

**Sniff signature:** top-level JSON object with all of:
- `seed_version` (integer ∈ {11..71}; current Electrum FINAL_SEED_VERSION is 71)
- `wallet_type` (string ∈ {`"standard"`, `"<k>of<n>"` regex per `electrum/util.py::multisig_type` `(\d+)of(\d+)`, `"2fa"`, `"imported"`})

**P6A in-phase SPEC correction (R1-I1 v2 fold):** the original P0A draft listed `wallet_type` value-set as `{"standard", "multisig", "2fa", "imported"}`. This was empirically wrong: Electrum stores multisig wallets under a `<k>of<n>` regex pattern (e.g., `"2of3"`, `"3of5"`), NOT the literal string `"multisig"`. Verified via:
1. WebFetch of `electrum/util.py::multisig_type` (regex `r'(\d+)of(\d+)'`).
2. The toolkit's own `wallet_export/electrum.rs:141` emits `format!("{k}of{n}")` for multisig.
3. The toolkit's own fixture `tests/export_wallet/electrum_multi_2of4.json` carries `"wallet_type": "2of4"`.

The corrected enumeration is therefore `{"standard", "<k>of<n>", "2fa", "imported"}`, with `<k>of<n>` recognized at sniff/parse time via the same `(\d+)of(\d+)` regex Electrum uses.

**Electrum-version scoping note (R1 I1 fold):** the 4-value `wallet_type` set above is the **current Electrum 4.x post-upgrade enumeration**. Legacy values `"old"`, `"xpub"`, `"bip44"` appear in pre-4.x wallet files but Electrum 4.x's auto-upgrade machinery rewrites them to `"standard"` on load (verified at `electrum/wallet_db.py::_convert_wallet_type` — see also: WebFetch confirmation that legacy values are upgrade-only). Users with pre-Electrum-4.x wallets must open them in Electrum 4.x first (auto-upgrade behavior) before exporting for ingest into mnemonic-toolkit. If a v0.28.0 user's blob carries a legacy `wallet_type` value, sniff returns `NoMatch` and P6 surfaces no Electrum-specific error; the user must upgrade via Electrum 4.x as a prerequisite. Tracking: pre-4.x direct ingest is out of v0.28.0 scope; if user demand surfaces, file new FOLLOWUP `wallet-import-electrum-pre-4x-legacy-types`.

Electrum's wallet file is Python-dict-serialized JSON; specific quirks (e.g., string-keyed nested dicts) require careful parsing. Sniff only validates top-level structure; parsing depth follows in §11.6 parse contract.

**Parse contract per `wallet_type`:**

| wallet_type | parse-action |
|---|---|
| `"standard"` | Singlesig parse: extract `keystore.xpub` + `keystore.derivation`. Compute descriptor via standard BIP-84/49/44 wrapping based on xpub SLIP-132 prefix. |
| `<k>of<n>` (regex `(\d+)of(\d+)`) | Multisig parse: iterate `x1/`, `x2/`, ... per-key sub-objects; extract per-cosigner xpub + derivation. Synthesize `wsh(sortedmulti(K, ...))` descriptor. |
| `"2fa"` | **REFUSE** — TrustedCoin two-factor wallet; not natively reconstructible from xpubs alone. Specific stderr error per §11.6.1 below. |
| `"imported"` | **REFUSE** — Electrum "imported addresses" wallet has no derivation chain to reconstruct. Specific stderr error per §11.6.1. |

**Encrypted wallets** (`use_encryption: true` + base64-encrypted sensitive fields) → **REFUSE** with specific stderr error per Q2 lock at §11.6.1.

**§11.6.1 — Refusal stderr templates (Q2 lock):**

- 2fa: `error: import-wallet: electrum: 2fa wallets require TrustedCoin two-factor restoration; ingest not supported`
- imported: `error: import-wallet: electrum: imported-addresses wallets have no derivation chain to reconstruct; ingest not supported`
- encrypted: `error: import-wallet: electrum: encrypted wallet files require decrypting via 'electrum --decrypt-wallet' first; encrypted ingest not yet supported (FOLLOWUP wallet-import-electrum-encrypted)` (new FOLLOWUP filed at P14A per Q2 lock).

**Provenance:** `ImportProvenance::Electrum(ElectrumSourceMetadata)`:

```rust
pub(crate) struct ElectrumSourceMetadata {
    pub seed_version: u64,
    pub wallet_type: ElectrumWalletType,     // Standard | Multisig { k, n }
    pub wallet_name: Option<String>,
    pub dropped_fields: Vec<String>,
}

pub(crate) enum ElectrumWalletType {
    Standard,
    /// `k`-of-`n` multisig per `electrum/util.py::multisig_type` regex
    /// `(\d+)of(\d+)`. P6A in-phase SPEC correction (see §11.6 intro).
    Multisig { k: u8, n: u8 },
}
```

(Refused variants — 2fa / imported / encrypted — do not produce a `ParsedImport` and therefore have no provenance.)

**CLI:** `--format electrum` (distinct from `mnemonic electrum {encode,decode}` which is the native-seed-format codec, NOT this wallet-file parser per §1.4 disambiguation).

**Canonicalize helper:** new `canonicalize_electrum` in `wallet_import/roundtrip.rs`.

---

## §12 — Module layout extensions (carry-forward + amendment)

v0.26.0 §8 module layout governs. v0.28.0 amendments:

- NEW: `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` (Phase P1)
- NEW: `crates/mnemonic-toolkit/src/wallet_import/specter.rs` (Phase P2)
- NEW: `crates/mnemonic-toolkit/src/wallet_import/coldcard.rs` (Phase P3)
- NEW: `crates/mnemonic-toolkit/src/wallet_import/coldcard_multisig.rs` (Phase P4)
- NEW: `crates/mnemonic-toolkit/src/wallet_import/jade.rs` (Phase P5)
- NEW: `crates/mnemonic-toolkit/src/wallet_import/electrum.rs` (Phase P6)
- AMENDED: `crates/mnemonic-toolkit/src/wallet_import/mod.rs` — `ImportProvenance` enum extended with 6 new alphabetically-sorted variants (per §6.2 discipline).
- AMENDED: `crates/mnemonic-toolkit/src/wallet_import/sniff.rs` — `SniffOutcome` enum extended (§6.2); `sniff_format` body rewritten (§6.3).
- AMENDED: `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs` — 6 new `canonicalize_<format>` helpers (one per new parser, per §11.x).
- AMENDED: `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:81` (`const VENDOR_MARKER_KEYS:` declaration; doc-comment at `:59-80`) — `VENDOR_MARKER_KEYS` expanded per §6.1.1 (R1 I2 citation fix → R2 N1 re-fix; const drifted from `:74` to `:81` after R1 I3+I4 doc-comment expansion).
- AMENDED: `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` — 4-line parser arm (§10) + DEPRECATION notice (§10.4) + error template update (§10.5).
- AMENDED: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — 8 dispatch sites extended for 6 new formats (per plan-doc P0C pre-stub + per-parser P{N}C arm-flips).

---

## §13 — Verification

Per plan-doc P0A reviewer-loop gate: architect R0 verifies this SPEC against (a) BIP-129 recon doc citations + (b) per-format vendor schemas + (c) existing v0.26.0 SPEC carry-forward consistency.

Phase P0A SPEC is REFERENCED (not duplicated) in downstream phase plans. Per-phase SPEC fidelity is reviewer-loop-gated: each P{N}A/B/C sub-phase's R0 verifies implementation conforms to §11.N normatives.

End of SPEC v0.28.0 Phase P0A scaffolding scope.
