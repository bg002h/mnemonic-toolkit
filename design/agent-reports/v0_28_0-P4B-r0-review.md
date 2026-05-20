# v0.28.0 Phase P4B — architect R0 self-review (GREEN)

**Sub-phase:** P4B — Line-oriented parser + descriptor synthesis + xfp policy +
`canonicalize_coldcard_multisig` + 4 fixtures.

**Plan-doc anchors:** §S.4 + table-row P4B (plan-doc line 516).

**SPEC anchors:** SPEC_wallet_import_v0_28_0.md §11.4 + §11.4.1 (5-row xfp
truth table) + §12.

**Branch:** `v0.28.0/p4-coldcard-multisig` (continued from P4A).

**Predecessor commit:** P4A @ `ed0f953`.

---

## Scope verification (P4B boundary)

Per plan-doc P4B scope:
1. Line-oriented parser handling BOTH Coldcard firmware-variance shapes
   (shared-derivation + per-cosigner). ✓
2. xfp policy per SPEC §11.4.1 5-row truth table:
   - Row 1 (header present + computed available + match) → silent ✓
   - Row 2 (header present + computed available + mismatch) → WARNING + use
     header. Byte-exact template matches SPEC §11.4.1 verbatim. ✓
   - Row 3 (header present + xpub malformed) → use header silently;
     xpub-parse error surfaces downstream. ✓
   - Row 4 (no header + computed available) → use computed silently. ✓
   - Row 5 (no header + no computed) → `ImportWalletParse` citing
     "coldcard-multisig: cannot compute xfp" per SPEC §11.4.1 verbatim. ✓
3. Descriptor synthesis: `wsh(sortedmulti(K, ...))` /
   `sh(wsh(sortedmulti(K, ...)))` / `sh(sortedmulti(K, ...))` per Format
   header. Multipath suffix `/<0;1>/*` per Coldcard convention. BIP-380
   checksum recomputed via miniscript `ChecksumEngine`. ✓
4. Network detection via BIP-48 coin-type (component index 1, hardened)
   on the first cosigner's path. Heterogeneity rejected. ✓
5. `canonicalize_coldcard_multisig` real body in
   `wallet_import/roundtrip.rs`: parses via `parse_text`, re-emits in
   shared-derivation shape with cosigners sorted lex by xpub, XFP header
   dropped (redundant with per-cosigner `<XFP>: <xpub>` lines). ✓
6. Fixtures (4 total):
   - `coldcard-ms-2of3-p2wsh-with-xfp.txt` — silent (row 1) ✓
   - `coldcard-ms-2of3-p2wsh-no-xfp.txt` — silent (row 1 via per-cosigner
     `<XFP>:` form) ✓
   - `coldcard-ms-3of5-p2wsh.txt` — silent (5 cosigners) ✓
   - `coldcard-ms-malformed-missing-format.txt` — refused with
     "missing `Format:` header" diagnostic ✓
7. `parse_text` exposed `pub(super)` for Phase E (Jade) P5B delegation. ✓
   (This unblocks Phase E's P5B once P4B merges to `release/v0.28.0`.)

Out-of-scope (P4C only): dispatch arm flips at `cmd/import_wallet.rs`
(8 sites), integration test file `tests/cli_import_wallet_coldcard_multisig.rs`
with xfp-divergence WARNING cell.

## SPEC §11.4 fidelity

- **Sniff signature** (delegated from P4A, unchanged): text format (NOT JSON);
  Name + Policy + Format header presence required.
- **Parse contract**:
  - Both shared-derivation and per-cosigner shapes accepted. ✓
  - Optional `XFP:` top-level header tolerated. ✓
  - Descriptor body shape: `<wrapper>(sortedmulti(K, [fp/path]xpub/<0;1>/*, ...))`
    where wrapper is `wsh` / `sh(wsh)` / `sh` per Format. ✓
- **§11.4.1 xfp policy 5-row truth table**: all 5 rows have dedicated unit
  test cells:
  - `parse_shared_derivation_no_xfp_header_silent` → row 1 (with-XFP-header
    fixture exercises this too)
  - `parse_xfp_header_mismatch_warns_uses_header` → row 2 (header mismatch)
  - `parse_per_cosigner_xfp_divergence_warns` → row 2 (per-line divergence)
  - `parse_no_header_no_per_cosigner_xfp_uses_computed_silent` → row 4
  - `parse_no_header_malformed_xpub_hard_errors` → row 5
  - Row 3 (header + xpub malformed) is implicitly covered — the header
    fingerprint is used, then the downstream xpub-parse fails in `Xpub::from_str`.
- **§11.4 provenance struct**: All 6 fields populated correctly
  (`xfp_was_blob_supplied`, `xfp_header_disagreed` flag semantics match
  SPEC).
- **§11.4 byte-exact WARNING template**: verified by unit test
  `parse_xfp_header_mismatch_warns_uses_header` against the SPEC §11.4.1
  template character-for-character (`warning: import-wallet: coldcard-
  multisig: xfp header `XFP: <hex>` disagrees with computed fingerprint
  `<hex>` from cosigner xpub; using blob-supplied header value as
  authoritative`).
- **§11.4 error template** for row 5: matches SPEC §11.4.1 verbatim
  ("coldcard-multisig: cannot compute xfp: no XFP header and xpub parse
  failed: <e>").

## Cross-instance handoff: P5 (Jade) unblock

Per plan-doc R3-C1 fold, Phase E's `wallet_import/jade.rs` P5B delegates to
`coldcard_multisig::parse_text` and `JadeSourceMetadata` embeds
`ColdcardMultisigSourceMetadata`. With P4B merged to `release/v0.28.0`:
- `parse_text` is `pub(super)` — Jade can call it via `crate::wallet_import::coldcard_multisig::parse_text`.
- `ColdcardMultisigSourceMetadata` is `pub(crate)` — Jade can embed it via
  `use super::coldcard_multisig::ColdcardMultisigSourceMetadata`.

E's P5B may begin as soon as P4B lands on `release/v0.28.0`. P5A (sniff
skeleton + struct decl) may have already begun parallel-to-P4A per plan
recommendation; P5B was hard-blocked, now unblocked.

## Architectural decisions

- **Sortedmulti exclusively**: Coldcard's multisig firmware emits
  `sortedmulti` (the lexicographic key sort is part of the script-image
  canonical form). The parser doesn't try to handle unsorted `multi(...)`
  exports — they're not part of the format. SPEC §11.4 says
  "Synthesize descriptor: `wsh(sortedmulti(K, ...))`".
- **Multipath suffix `/<0;1>/*` baked into the synthesized descriptor**: the
  Coldcard text file carries bare xpubs (no `/<0;1>/*` suffix); per BIP-389
  + the toolkit's convention, multisig descriptors emit `/<0;1>/*`
  multipath to cover both the receive and change branches. Mirrors how
  BSMS Round-2 descriptors carry the same suffix in `bsms-*.txt` fixtures.
- **canonicalize drops XFP header**: per-cosigner `<XFP>: <xpub>` lines
  carry the same XFP value (effective per truth table); duplicating it
  in a top-level header is purely diagnostic. Canonical form drops the
  duplicate. Cosigners sorted lex by xpub (sortedmulti convention).
- **`parse_text` returns `ParsedImport` (not `Vec<ParsedImport>`)**: a
  Coldcard multisig text file is single-descriptor by construction (N
  cosigners → 1 multisig wallet). The `WalletFormatParser::parse` impl
  wraps `parse_text` in `vec![..]` to match the trait signature.
- **CRLF normalization mirrors BSMS**: identical strategy (single
  `.replace("\r\n", "\n")` early in parse_text).
- **`pub(super)` on `parse_text`**: confines visibility to the
  `wallet_import` module. Mirror precedent: `bitcoin_core::extract_threshold`
  is `pub(super)`.
- **xpub depth detection**: I use `xpub.fingerprint()` regardless of depth
  (per SPEC §11.4.1's general formula). Real Coldcard files typically have
  depth-0 xpubs (master) or depth-4 xpubs (BIP-48 derived); for depth-4,
  the fingerprint is THIS xpub's own (not the master). The SPEC explicitly
  accepts this as "computed-available, may disagree with header" — which
  is row 2 of the truth table.

## Edge cases handled

- Comment lines (`# …`) tolerated everywhere (header AND post-header).
- Blank lines tolerated everywhere.
- Trailing whitespace on lines stripped at line_key/value extraction.
- CRLF and LF both accepted via single normalization step.
- Both `K of N` and `K-of-N` Policy forms parsed.
- Case-insensitive Format matching (`p2wsh`, `P2WSH`, `P2WSH-P2SH`,
  `P2SH-P2WSH`, `P2SH_P2WSH` all accepted).
- u8 overflow on Policy K or N → typed error citing field + value.
- K out of range (K > N or K = 0) → typed error.
- Wrong cosigner count vs declared N → typed error.
- Heterogeneous coin-type across cosigners → typed error mirroring
  `bsms::network_from_origins` rule.

## Verification

- `cargo clippy --all-targets -- -D warnings` — clean. ✓
- `cargo test --bin mnemonic wallet_import::` — 134/134 unit tests pass
  (28 new this phase: P4B parser + canonicalize cells; 106 inherited from
  P4A + earlier). ✓
- Full `cargo test` workspace run — all suites pass; no regressions in
  existing BSMS / Bitcoin Core / dispatch suites. ✓

## Findings

**Critical:** NONE.
**Important:** NONE.
**Minor:** NONE worth blocking on.

Pre-fold-of-self note: the `skeleton_canonicalize_helpers_accept_empty_blob`
test in `wallet_import/roundtrip.rs:1064-1081` was updated to OMIT
`coldcard-multisig` from the skeleton list (it's no longer a skeleton).
The omission is annotated inline with a comment pointing to the new real
canonicalize cells. This is a structural fold — the previous test would
have failed once the body landed, so the test must be updated in lockstep.

## Overall R0 verdict

**GREEN.** P4B scope satisfied per SPEC §11.4 + §11.4.1; xfp policy 5-row
truth table covered by dedicated unit tests; descriptor synthesis +
canonicalize work end-to-end against all 4 fixtures + synthetic cells;
no regressions in pre-existing suites; cross-instance handoff to Phase E
(Jade) is structurally complete (`parse_text` published as `pub(super)`,
`ColdcardMultisigSourceMetadata` is `pub(crate)`).

Recommendation: commit P4B, push immediately so Phase E P5B can unblock,
proceed to P4C (dispatch + integration cells).
