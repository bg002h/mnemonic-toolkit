# PLAN — mnemonic-toolkit v0.27.0 (BSMS round-trip + wallet-import handoff)

**Status:** draft R6 — Phase 2 recon mid-execution pivot (Path B-lite per user direction). See §8 for diff vs R5; awaits opus architect validation before Phase 2 code starts. (R5 was approved via ExitPlanMode and Phase 0-1 executed at `b47ad2a` + `e908309`; R6 amends §2.2/§3/§4 for BIP-129 framing correction.)
**Scope:** 7 in-scope items; toolkit-only (no sibling lockstep).
**Pre-cycle baseline:** master `66c8a56` = tag `mnemonic-toolkit-v0.26.0` + FOLLOWUPS commit `2efe5b0`.
**Authorship:** single-instance (this Claude session); v0.26.0 multi-instance topology not in effect.
**Target tag:** `mnemonic-toolkit-v0.27.0`. GitHub release with full CHANGELOG.

---

## §1. Context

The v0.26.0 cycle shipped wallet-import (BSMS Round-2 + Bitcoin Core `listdescriptors`) with several known deferrals tracked as FOLLOWUPS. v0.27.0 closes six of those FOLLOWUPS and absorbs a seventh consumer-side flag pair so that the wallet-import → bundle / export-wallet → cross-format-conversion data-flow becomes a closed loop end-to-end.

**Scope decision (locked in plan-mode):** "Grow to 6 + envelope enrichment + both consumer-side flag directions" — items #1-4 from the kickoff + the `wallet-import-json-envelope-full-bundle` FOLLOWUP + **both** consumer-side flag directions (`bundle --import-json` and `export-wallet --from-import-json`). Item #5 from the kickoff (`xpub-search-manual-gui-chapters`) is **deferred out of v0.27.0** to a dedicated GUI-side cycle.

**Why a single cycle.** Items #5/#6/#7 are not independent — they share `import-wallet --json`'s envelope wire-format. Splitting envelope from consumer would create a stale-envelope window where downstream consumers encode against a transitional shape. Shipping them together preserves the v0.26.0 promise that "v0.26.0 summary is forward-compatible with v0.27's full shape."

**End-user outcome.** Three new closed loops:
1. **BSMS round-trip** — `mnemonic export-wallet --format bsms` exists; bundle-side test cells deferred at v0.26.0 Phase 4 R0 (the `roundtrip: { status: "blocked_no_emitter" }` JSON path) become runnable.
2. **Wallet → bundle** — `mnemonic import-wallet --json | mnemonic bundle --import-json -` synthesizes ms1/mk1/md1 cards from a parsed wallet (watch-only by default; seed-overlaid when `--ms1` supplied).
3. **Cross-format conversion** — `mnemonic import-wallet --format bsms ... --json | mnemonic export-wallet --from-import-json - --format sparrow` re-emits the same descriptor + cosigner set in any export format.

---

## §2. Brainstorm

### §2.1 The seven items

| # | FOLLOWUP slug | Tier | Cells | LOC | Depends on |
|---|---|---|---|---|---|
| 1 | `wallet-export-bsms-emitter` | feature | 8 | ~180 | — (R6 pivot: 4-line Round-2 emit is independent of #2 verify path; see §8) |
| 2 | `bsms-verify-signatures` | feature | ~15 | ~250 | — |
| 3 | `inspect-json-schema-version-backfill` | trivial | 2-3 | ~30 | — |
| 4 | `coordinator-runbook-into-design-dir` | doc-only | 1 (presence smoke) | 0 LOC | — |
| 5 | `wallet-import-json-envelope-full-bundle` | feature | 7-8 | ~120 | — |
| 6 | `bundle --import-json` consumer | NEW feature | 10-12 | ~180 | #5 |
| 7 | `export-wallet --from-import-json` consumer | NEW feature | 10-12 | ~180 | #5 |

**Total budget:** ~940 LOC + ~50-60 test cells (vs v0.26.0's ~1500 LOC + ~85 cells across 3 features). Comparable cycle size.

### §2.2 Locked design questions

**Q1 (BSMS Round-2 emit form). [REVISED — Phase 2 recon pivot; see §8.]** What output shapes does `export-wallet --format bsms` produce?
- **Lock:** Two shapes: **2-line** (lenient, descriptor-only excerpt — preserves v0.26.0 SPEC §4 lenient input shape's symmetric output form) and **4-line** (BIP-129-canonical Round-2 plaintext: `BSMS 1.0` / `<descriptor>#<checksum>` / `<path-restrictions>` / `<first-address>`). Default is **4-line** (BIP-129 spec-canonical). `--bsms-form 2-line|4-line` selects explicitly.
- **DROPPED:** 6-line emit. The plan-doc R0-R5 6-line shape (`BSMS 1.0` / `<TOKEN>` / `<descriptor>` / `<path>` / `<first-address>` / `<SIGNATURE>`) was authored against a misreading of BIP-129. Per the Phase 2 BIP-129 recon (`design/agent-reports/v0_27_0-phase-2-bip129-recon.md`): BIP-129 §Specification → §Setup Process → Round 2 specifies a 4-line plaintext (no SIG); signatures live only on **Round-1** key records (Signer → Coordinator) and are **BIP-322 legacy-format ECDSA recoverable signatures**, not HMAC. The toolkit's v0.26.0 6-line lenient INPUT shape stays for backward-compat (`SPEC_wallet_import_v0_26_0.md` §4 line 152); emit does NOT mirror it because no real-world wallet consumes 6-line input.
- Rationale: BIP-129 spec fidelity. Outward-bound blobs in BIP-129-canonical form are consumable by Coldcard / Sparrow / Nunchuk / Coinkite Python ref; 6-line emit is non-interoperable invention.

**Q2 (BSMS signature verify on import). [REVISED — Phase 2 recon pivot; see §8.]** What signature verification does `import-wallet` do, and where does it fit?
- **Lock:** Signature verify is a **separate input path** for BIP-129 Round-1 records (Signer → Coordinator), gated by a new flag `--bsms-round1 <FILE|->` (accepts one or more 5-line Round-1 records). The verify scheme is BIP-322 legacy-format ECDSA recoverable signature, verified against the signer's own pubkey/xpub extracted from line 3 of each Round-1 record. NO `--coordinator-hmac-key` (HMAC is BIP-129's encryption-envelope MAC, not its signature surface).
- **Behavior:** `--bsms-verify-strict` (kept) controls verify-failure semantics: strict → fail with `BsmsSignatureMismatch` exit 2 on mismatch; lenient (default) → stderr NOTICE + `signature_verified: false` per-record + proceed.
- **Untouched: v0.26.0 6-line `bsms_audit.signature` field.** Continues to be opaque-stored (parser preserves verbatim). Per `SPEC_wallet_import_v0_26_0.md` §4 line 152's lenient-input framing, the 6-line `signature` field has no agreed verify semantics in the toolkit (it was authored as a placeholder for "envelope-side HMAC/signature" that doesn't exist in BIP-129 as framed). v0.27.0 closes the `bsms-verify-signatures` FOLLOWUP via the NEW Round-1 path (which IS BIP-129); it does NOT retroactively define verify semantics for the 6-line `signature` field. The future cycle FOLLOWUP `bsms-bip129-full-cutover` (filed at v0.27.0 cycle close — see §8) deprecates the 6-line lenient shape after a stable-window.
- **v0.26.0 backward-compat preserved:** existing 2-line + 6-line import paths unchanged. v0.27.0 ADDS Round-1 verify; it does NOT modify v0.26.0 parser semantics.

**Q3 (InspectJson schema_version placement).** Top-level envelope wrapper OR additive field on each tagged variant?
- **Lock:** Top-level wrapper `InspectEnvelope { schema_version: "1", #[serde(flatten)] body: InspectJson<'a> }`. Mirrors `XpubSearchEnvelope` exactly (line 111-116 of `cmd/xpub_search/mod.rs`).
- **Companion:** Apply same wrapper to `RepairJson` in the same phase (lifetime parameter confirmed at Phase 1 implementation time against actual struct definition). FOLLOWUP slug `inspect-json-schema-version-backfill` body explicitly covers both envelopes ("both `InspectJson` and `RepairJson` envelopes" — FOLLOWUPS.md:51). Single-step consistency.

**Q4 (envelope `bundle` field shape — FOLLOWUP contract).** What does `import-wallet --json`'s `bundle:` field carry in v0.27.0?
- **Lock — FOLLOWUP-faithful BundleJson contract.** The `bundle` field is a literal **`crate::format::BundleJson`** (file `crates/mnemonic-toolkit/src/format.rs:120-145`). The FOLLOWUP text says: *"wire the `--json` envelope's `bundle:` field to emit the full toolkit-native `BundleJson` shape (the same `verify-bundle --bundle-json` consumes — with synthesized ms1/mk1/md1 cards). This requires invoking the synthesizer post-parse against the supplied / overlayed seeds; for watch-only cosigners, emit the ms1/mk1 sentinel forms per SPEC §5.8."* (FOLLOWUPS.md:2155.)
- **Synthesis path:** post-parse, invoke `crate::synthesize::synthesize_unified` against `(descriptor=ParsedImport.descriptor, slots=ParsedImport.cosigners)`. For watch-only imports (v0.26.0 default), ms1 array per SPEC §5.8 is `[""..]` (N sentinel-string entries) and mk1 array carries the cosigner-derived encoded mk1 cards.
- **v0.26.0 → v0.27.0 migration:**
  - v0.26.0 summary shape `bundle: { cosigners: [...], network, threshold }` is **REPLACED** with `bundle: BundleJson { schema_version: "2", mode, network, template, descriptor, account, origin_path(s), master_fingerprint, ms1, mk1, md1, multisig, privacy_preserving }`.
  - This IS a wire-shape change (not strictly additive). CHANGELOG entry MUST be `### Changed`. SemVer minor bump (v0.26 → v0.27) is appropriate per project's pre-1.0 stance (additive-OK in minor; replacement requires minor at a minimum).
  - Downstream consumers encoded against the v0.26.0 summary shape **WILL** need updates. The mnemonic-gui pin (currently at v0.11.0) is NOT bumped in this cycle (toolkit-only tag); GUI's next cycle adopts the new shape explicitly.
- **Why this isn't backward-compatible-via-alias:** Opus R0 considered emitting `bundle.network` as a deprecated alias alongside `bundle.descriptor.network`. Under the BundleJson contract, the network IS top-level in BundleJson, so there's no nested-vs-flat ambiguity to bridge. The change is a clean replacement.
- **Outer envelope** shape carries `bsms_audit`/`source_metadata`/`roundtrip` as siblings to `bundle`; see §3.2.

**Q5 (consumer flag input shape).** `bundle --import-json <FILE|->` and `export-wallet --from-import-json <FILE|->` — what shape do they accept?
- **Lock:** The literal `import-wallet --json` envelope (the FULL post-Q4 shape) — `{ schema_version, source_format, bundle: BundleJson, bsms_audit, source_metadata, roundtrip }`. Either flag accepts:
  - the array form (`[{ ... }, ...]` — multi-entry: Bitcoin Core can have multiple descriptors)
  - or a single envelope element (when import-wallet emitted a single entry)
  - or stdin via `-` (matches `--blob -` precedent in `import-wallet`).
- When the array has > 1 entry, both consumers require `--import-json-index <N>` (bundle) / `--from-import-json-index <N>` (export-wallet) to disambiguate; absence is `BadInput` exit 2. **Default N=0 is rejected explicitly** (silent picking is a wallet-misidentification footgun).
- Both consumers parse with `serde::Deserialize` against a typed struct that mirrors the emit-side serde shape.

**Q6 (export-wallet template / wallet name defaulting + `--account` discipline).** `ParsedImport`-derived BundleJson lacks `wallet_name` and explicit `CliTemplate`. What does `--from-import-json` do?
- **Lock:**
  - `--wallet-name` defaults to `"imported"` if not supplied alongside `--from-import-json`. (Some emitters bake the name into the output blob; this is OK to default to a constant.)
  - Template inference is two-step: (1) parse envelope's `bundle.descriptor` via `miniscript::Descriptor::<DescriptorPublicKey>::from_str` (FROM string MUST be miniscript-form, which the canonical descriptor in `bundle.descriptor: Option<String>` is). (2) Call `script_type_from_descriptor(&parsed_ms_descriptor) -> Result<WalletScriptType, ToolkitError>` (wallet_export/mod.rs:182) — returns `WalletScriptType` enum (NOT `&'static str` as the type-name suggests). For descriptor-mode wallet-import path, set `EmitInputs.template = None` (the descriptor itself is canonical-passthrough; `script_type` carries the variant). Fallback to descriptor-passthrough means `template = None` (already the default).
  - **`--account` is rejected with `BadInput` when supplied alongside `--from-import-json` OR `--import-json`** (symmetric across both consumers — opus R0 I4 fold). The envelope's `bundle.account` is canonical; manual override is a footgun.

**Q7 (consumer flag mutual exclusion).** Existing inputs to bundle are `--template` / `--descriptor` / `--descriptor-file` (mutually exclusive via clap). Where do the new flags slot?
- **Lock:** Add to the existing exclusion set. Both `--import-json` (bundle) and `--from-import-json` (export-wallet) are mutually exclusive with `--template` / `--descriptor` / `--descriptor-file`. Clap-derive `ArgGroup { required = true, multiple = false }` extended by one variant each.

**Q8 (BIP-129 verifier source-of-truth — Phase 2 recon outcome). [REVISED.]** The Phase 2 BIP-129 recon (`design/agent-reports/v0_27_0-phase-2-bip129-recon.md`) reads BIP-129 directly. Findings replace prior R0-R5 speculation:
- **What BIP-129 actually specifies for SIG:** BIP-322 legacy-format ECDSA **recoverable** signature (65 bytes = header + r + s) over the 4-line Round-1 body (lines 1-4 joined by `\n`, NO trailing newline), under the standard "Bitcoin Signed Message" double-SHA256 digest, base64-encoded on line 5. The signing key is the privkey behind the line-3 KEY (raw pubkey OR xpub's own embedded pubkey).
- **Test vectors:** BIP-129 `==Test Vectors==` (lines 211-453 of bip-0129.mediawiki) provides 5 in-spec signers across 4 modes (NO_ENCRYPTION/pubkey, NO_ENCRYPTION/xpub, STANDARD/xpub, EXTENDED/3-signers). All Round-1 SIGs are verifiable via the BIP-322 legacy path. Cross-validate against Coinkite Python ref `https://github.com/coinkite/bsms-bitcoin-secure-multisig-setup` (Peter Gray, BIP-129 co-author).
- **NOT in scope for v0.27.0:** PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 encryption-envelope MAC (STANDARD/EXTENDED modes). Filed at cycle close FOLLOWUP `bsms-bip129-full-cutover` for v0.28+.
- **Rust deps (all already in tree):** `secp256k1` (ECDSA recover/verify via `rust-bitcoin`), `bitcoin::sign_message::signed_msg_hash` (exact BIP-129 digest), `base64::engine::general_purpose::STANDARD.decode` (line-5 SIG decode).
- **Phase 2 R0 explicit scope item:** *"Did the verifier accept all 5 BIP-129 in-spec test-vector Round-1 SIGs (TV-1 Signer 1, TV-2 Signer 1, TV-3 Signer 1, TV-4 Signers 1-3)? Did it correctly reject a TV with a 1-byte-flipped SIG? Did it correctly handle BOTH raw-pubkey KEYs (TV-1) AND xpub KEYs (TV-2 through TV-4)?"*

**Q9 (CLI naming for the BSMS surfaces). [REVISED — Phase 2 recon pivot; see §8.]** Flag names align with BIP-129 reality:
- **DROPPED:** `--coordinator-hmac-key` (no HMAC key in BIP-129's signature path; the term reflected a misreading of BIP-129's encryption-envelope MAC).
- **NEW:** `--bsms-round1 <FILE|->` (repeating flag, accepts one Round-1 5-line record per file or one record per stdin invocation). Verifies BIP-322 ECDSA SIG against the line-3 KEY. Independent input path from `--blob <FILE>` (which still accepts BIP-129 Round-2 / Bitcoin Core listdescriptors).
- **CHANGED:** `--bsms-form 2-line|6-line` → `--bsms-form 2-line|4-line` (drops 6-line emit; adds 4-line BIP-129-canonical emit). Default `4-line`.
- **KEPT:** `--bsms-verify-strict` (now controls Round-1 SIG verify behavior; strict = exit 2 on mismatch, lenient = stderr NOTICE + `signature_verified: false`).
- Accepts `<FILE>` (raw text contents — the 5-line record), or `-` (stdin one record). No `@env:` form needed (Round-1 records are file-shaped, not key-shaped). NO TOKEN file input.
- Net flag count: was 8 new flags; revised to **7 new flags** (drop `--coordinator-hmac-key`, drop `6-line` value from `--bsms-form`, change to `4-line`, add `--bsms-round1`).

**Q10 (error variant names + ordering convention). [REVISED.]** New `ToolkitError` variants for BIP-129 Round-1 verify:
- **Lock:** Two top-level variants: `BsmsRound1Malformed { reason: String }` (Round-1 5-line parse error; line count != 5, malformed line-3 KEY, malformed line-5 base64 SIG, line-4 description contains `\n` or `\r`, etc.) and `BsmsSignatureMismatch { record_index: usize, signer_pubkey: String, reason: String }` (BIP-322 verify rejected the SIG). Variants are inserted at the end of the existing `ToolkitError` enum (per the existing "newest at bottom" convention — see error.rs:10-235 for current grouping). The `error-rs-canonical-ordering-doc` FOLLOWUP (which proposes alphabetical ordering) **stays open**; v0.27.0 does NOT close it.
- **Note on prior R0-R5 names:** `BsmsHmacKeyMissing` and `BsmsTokenMalformed` are DROPPED (no HMAC key, no TOKEN file input under Path B-lite scope).
- **Why two top-level variants** (not three): matches the granularity of distinct failure modes the user can act on (fix-the-record vs re-sign-the-record); maps to distinct exit code paths (both exit 2, but the message text differentiates).

### §2.3 Items deferred OUT of v0.27.0 (intentional)

- **#5 kickoff item — `xpub-search-manual-gui-chapters`** — large prose work (4 chapters × 200-500 LOC each); per kickoff "best after the implementation items so chapters can cite final shipped surfaces."
- **`wallet-import-fixture-corpus-expansion`** — coverage-class FOLLOWUP, not load-bearing for v0.27.0's correctness contract. Filed for v0.28+.
- **6-line BSMS sortedmulti-2of3 / decay-4032 / mainnet ypub/zpub / tr(NUMS) taproot fixtures** — folded into the corpus-expansion FOLLOWUP above.
- **gui-schema mirror for new `--import-json` / `--from-import-json` flags** — these will auto-emit via the existing `gui-schema` macro infrastructure; mnemonic-gui pin bump (gui v0.12.0) happens on consumer-side cycle, not v0.27.0 lockstep.
- **N×M cross-format conversion matrix expansion** — v0.27.0 ships ONE integration cell (BSMS → Sparrow). Other 6 conversions are mechanical re-runs of existing per-emitter tests with envelope-derived input. File FOLLOWUP `cross-format-conversion-matrix-expansion` at cycle close for v0.28+ (opus R0 D7 lock).
- **`error-rs-canonical-ordering-doc` codification** — proposed alphabetical-ordering FOLLOWUP stays open; v0.27.0 inserts new variants at end per existing convention.

---

## §3. SPEC

### §3.1 New CLI surfaces

#### §3.1.1 `mnemonic export-wallet --format bsms`

**[REVISED — Phase 2 recon pivot; see §8.]**

```
mnemonic export-wallet --format bsms [--bsms-form 2-line|4-line]
    [--wallet-name <STRING>]
    [--account <N>]
    [--template <NAME> | --descriptor <STRING> | --descriptor-file <PATH>]
    --slot @N.xpub=<XPUB> --slot @N.fingerprint=<HEX> --slot @N.path=<PATH>
    [...]
```

Output forms:
- **4-line** (default; BIP-129-canonical Round-2 plaintext):
  ```
  BSMS 1.0
  <descriptor>#<checksum>
  <PATH_RESTRICTIONS>   # comma-separated, leading-`/`, non-hardened; or `No path restrictions` literal
  <FIRST_ADDRESS>       # derived from descriptor at first path-restriction's leading index (e.g. `/0/0` for `/0/*,/1/*`)
  ```
- **2-line** (lenient excerpt, when `--bsms-form 2-line` explicit):
  ```
  BSMS 1.0
  <descriptor>#<checksum>
  ```

Notes:
- 4-line is BIP-129 §Specification → §Round 2 canonical. Per `design/agent-reports/v0_27_0-phase-2-bip129-recon.md`: line 1 is header, line 2 is descriptor with `#<checksum>`, line 3 is path restrictions (comma-separated `/0/*,/1/*` form OR literal `No path restrictions`), line 4 is wallet first address (derived from descriptor at the first path-restriction's index-0 or `0/0` for `No path restrictions`).
- 2-line is the v0.26.0 lenient excerpt, preserved as input-form-symmetric output. Phase 3 emits 2-line only on explicit `--bsms-form 2-line`; default is the spec-canonical 4-line.
- The 6-line shape is **DROPPED** from emit. Per Q1 + Phase 2 recon: that shape was a plan-doc invention that conflated BIP-129's Round-2 plaintext + (incorrectly-framed) "envelope-side HMAC/signature" into a single flat blob. BIP-129 reality: Round-2 carries NO signature; signatures are on **Round-1** (Signer → Coordinator) records, not in the toolkit's emit-from-bundle path.

Errors:
- Taproot descriptor (`tr(...)`) with `--format bsms` (either form) → `BadInput { reason: "BIP-129 does not specify taproot key records; see FOLLOWUP bsms-taproot-emit" }` exit 2 (BIP-129 prerequisites omit BIP-386; defer).
- Per-cosigner divergent-paths (mixed origin paths across cosigners) with `--bsms-form 4-line` → emit `No path restrictions` on line 3 (BIP-129 line-3 is wallet-level not cosigner-level path).

#### §3.1.2 `mnemonic import-wallet --bsms-round1` + `--bsms-verify-strict`

**[REVISED — Phase 2 recon pivot; see §8.]**

New opt-in flag(s) on `import-wallet`:
- `--bsms-round1 <FILE|->`: supplies a BIP-129 5-line Round-1 key record (Signer → Coordinator) for BIP-322 ECDSA signature verification. Repeating flag — one per Round-1 record. `<FILE>` reads file contents; `-` reads one record from stdin.
- `--bsms-verify-strict`: when present, Round-1 verify failures are fatal:
  - SIG mismatch → `BsmsSignatureMismatch { record_index, signer_pubkey, reason }` exit 2.
  - Malformed record → `BsmsRound1Malformed { reason }` exit 2.

Default (`--bsms-verify-strict` absent):
- SIG mismatch → stderr NOTICE "import-wallet: bsms-round1: signature verification failed for signer N (pubkey HHH) — record index M" + `signature_verified: false` per-record in `--json` envelope; proceed (exit 0).
- Malformed record → `BsmsRound1Malformed` exit 2 (parse errors are always fatal; only signature mismatches are lenient).

Behaviors:
- `--bsms-round1` is **independent** of the existing `--blob` / `--format` inputs. The Round-1 path verifies signatures; the `--blob` path parses Round-2 descriptors. A user wanting full BIP-129 round-trip supplies BOTH: one `--bsms-round1` per signer (for SIG verify) + one `--blob` (for the Round-2 descriptor record to be parsed into a bundle).
- Round-1 verify alone (without `--blob`) is a meaningful CLI use ("verify these Round-1 records' SIGs, emit JSON of who-signed-what"). When no `--blob` is supplied + `--bsms-round1` is supplied, the command's output is the per-record `signature_verified` envelope; no bundle is emitted.
- Existing v0.26.0 2-line / 6-line `--blob` paths are unchanged. The 6-line lenient parser continues to store `bsms_audit.signature` verbatim with `signature_verified: false`. v0.27.0 does NOT verify 6-line `signature` (no agreed semantics; see §8 + Q2).

Errors:
- `--bsms-round1` with `--format bitcoin-core` blob → `BadInput` exit 2 (Round-1 records are BSMS-only).
- Malformed Round-1 record (not 5 lines after CRLF→LF normalize; line 1 != `BSMS 1.0`; line-3 KEY not `[fp/path]raw-pubkey-hex|xpub`; line-5 not valid base64 → 65 bytes) → `BsmsRound1Malformed` exit 2.
- Taproot KEY (xpub indicating tr-only context via descriptor-comment hints) → `BadInput { reason: "BIP-129 does not specify taproot key records; see FOLLOWUP bsms-taproot-emit" }`. BIP-129's prerequisites omit BIP-386; defer.

#### §3.1.3 `mnemonic bundle --import-json <FILE|-> [--import-json-index <N>]`

New input mode for `bundle`. Mutually exclusive with `--template` / `--descriptor` / `--descriptor-file`. Reads an envelope-shaped JSON (per §3.2 below); extracts `descriptor` + cosigner-derived slots from the nested `BundleJson`; synthesizes ms1/mk1/md1 cards via `synthesize_unified` against the descriptor + slots. When the input has `> 1` envelope element (Bitcoin Core multi-descriptor case), `--import-json-index <N>` is required; absence is `BadInput` exit 2.

Behaviors:
- The envelope's `bundle.ms1` array is consulted for seed-bearing state: `ms1[i] == ""` ⇒ slot i is watch-only ⇒ user can fill via `--ms1` (existing flag). `ms1[i] != ""` ⇒ slot i carries an ms1 phrase from a prior overlay ⇒ supplying `--ms1` for the same slot is a **conflict** (`BadInput` exit 2; explicit error to prevent silent override).
- `--account` is rejected (per Q6); the envelope's `bundle.account` is canonical.
- Output: same as current `bundle` — ms1/mk1/md1 stdout (or `--bundle-json` envelope).

#### §3.1.4 `mnemonic export-wallet --from-import-json <FILE|-> [--from-import-json-index <N>]`

New input mode for `export-wallet`. Mutually exclusive with `--template` / `--descriptor` / `--descriptor-file`. Reads the same envelope as §3.1.3. Re-emits the descriptor + cosigners as the requested `--format <Y>`. Cross-format converter.

Behaviors:
- `--wallet-name` defaults to `"imported"`.
- `--account` rejected (per Q6).
- Template inference: `script_type_from_descriptor(envelope.bundle.descriptor.or(decode_md1))` → matches a built-in template (BIP-44/49/84/86) OR falls back to descriptor-passthrough mode.

### §3.2 `import-wallet --json` envelope shape (v0.27.0)

**Outer envelope** (per array element):

```json
{
  "schema_version": "1",
  "source_format": "bsms",
  "bundle": <BundleJson>,
  "bsms_audit": {
    "token": "...",
    "signature": "...",
    "first_address": "...",
    "derivation_path": "m/0/0",
    "signature_verified": false
  },
  "source_metadata": null,
  "roundtrip": {
    "byte_exact": true,
    "semantic_match": true,
    "diff": null,
    "status": "ok"
  }
}
```

**Multi-entry array example** (Bitcoin Core can emit several descriptors):

```json
[
  { "schema_version": "1", "source_format": "bitcoin-core", "bundle": {...}, "bsms_audit": null, "source_metadata": {...}, "roundtrip": {...} },
  { "schema_version": "1", "source_format": "bitcoin-core", "bundle": {...}, "bsms_audit": null, "source_metadata": {...}, "roundtrip": {...} }
]
```

**`bundle: BundleJson` shape (from `crates/mnemonic-toolkit/src/format.rs:120-145`):**

```rust
pub struct BundleJson {
    pub schema_version: &'static str,    // "4" — confirmed against synthesize.rs:1501 + cmd/bundle.rs:693 construct sites; format.rs:114 doc comment "v0.2: schema_version \"2\"" is historical and does NOT match current code
    pub mode: &'static str,              // "full" | "watch-only"
    pub network: &'static str,
    pub template: Option<&'static str>,  // &'static str lifetime — must source from a static set (e.g., script_type_from_descriptor returns &'static "wsh-sortedmulti" etc.); NEVER from heap-owned ParsedImport string
    pub descriptor: Option<String>,      // Some in descriptor-mode; None in template-mode
    pub account: u32,
    pub origin_path: Option<String>,
    pub origin_paths: Option<Vec<String>>,
    pub master_fingerprint: Option<String>,
    pub ms1: MsField,                    // length-N invariant; "" sentinel for watch-only slots per SPEC §5.8
    pub mk1: MkField,                    // MkField::Single(Vec<String>) for N=1; MkField::Multi(Vec<Vec<String>>) for N>1 — per-cosigner chunks
    pub md1: Vec<String>,
    pub multisig: Option<MultisigInfo>,  // Some when N>1; carries MultisigInfo { template, threshold, cosigner_count, path_family, cosigners: Vec<CosignerEntry> }
    pub privacy_preserving: bool,
}
```

For watch-only import (v0.26.0 default and v0.27.0 default unless seed-overlay flags supplied):
- `mode = "watch-only"`.
- `ms1` is length-N with each entry `""` (SPEC §5.8 sentinel).
- `mk1` carries the cosigner-derived encoded mk1 cards.
- `md1` carries the descriptor-encoded md1 cards.

**Migration from v0.26.0:**
- v0.26.0 shipped `bundle: { cosigners: [...], network, threshold }` (compact summary).
- v0.27.0 ships `bundle: BundleJson` (synthesized cards).
- This is a wire-shape **replacement** (not additive). CHANGELOG entry MUST be `### Changed`.
- Downstream encoded consumers (notably mnemonic-gui) will need updates; pin bump deferred to GUI cycle.

**Synthesis path (descriptor-mode — load-bearing for v0.27.0).** Both v0.26.0 wallet-import formats produce a literal descriptor (BSMS Round-2 carries an explicit descriptor; Bitcoin Core listdescriptors emits descriptors directly). Therefore ALL `ParsedImport`-derived BundleJson constructions in v0.27.0 use **descriptor-mode synthesis**:

```rust
// crate::synthesize::synthesize_descriptor at synthesize.rs:200
pub fn synthesize_descriptor(
    descriptor: &Descriptor,
    cosigners: &[CosignerKeyInfo],
    privacy_preserving: bool,
) -> Result<Bundle, ToolkitError>;
```

`Bundle` (the return type) carries only `ms1: MsField`, `mk1: MkField`, `md1: Vec<String>`. The envelope-emission path assembles a full `BundleJson` around the `Bundle` by populating the remaining fields from `ParsedImport` + descriptor parse.

**§3.2.1 `ParsedImport → BundleJson` field-by-field mapping** (Phase 4 implementer contract):

| BundleJson field | Source | Notes |
|---|---|---|
| `schema_version` | literal `"4"` | Pinned by `bundle_json_schema_version_pinned_to_4` test at synthesize.rs:1494; future bumps update both sites + this plan |
| `mode` | `"watch-only"` when all `cosigners[i].entropy.is_none()` else `"full"` | v0.26.0 import is always watch-only; v0.27.0 seed-overlay on import is out of scope |
| `network` | `network_human_name(parsed.network)` (existing helper at `cmd/import_wallet.rs:491`, signature `fn network_human_name(n: bitcoin::Network) -> &'static str`) | Must be `&'static`; helper returns "mainnet"/"testnet"/"signet"/"regtest" from a static set. **Phase 4 task:** promote this helper from private to `pub(crate)` so synthesize-envelope code path can call it without duplication |
| `template` | `None` — descriptor-mode | Never Some for wallet-import path |
| `descriptor` | `Some(parsed.original_descriptor.clone())` | `md_codec::Descriptor → String` is NOT a confirmed-existing API (per xpub_search/descriptor_intake.rs:4 comment). **Phase 4 prerequisite task:** add field `original_descriptor: String` to `ParsedImport` (wallet_import/mod.rs:57); populate at parse time from the **pre-strip raw descriptor** (BSMS Round-2 line 2 verbatim including `#<checksum>`; Bitcoin Core `desc` JSON field verbatim including `#<checksum>`). **Do NOT source from `descriptor_body_no_csum`** — that helper strips the checksum, and downstream §3.5 BSMS emitter + §3.7 export-wallet --from-import-json both assume `EmitInputs.canonical_descriptor` carries `#<checksum>`. Format mirrors `BundleJson.descriptor` doc-comment "User-supplied descriptor verbatim" at format.rs:128 |
| `account` | `0` (hardcoded for v0.27.0 wallet-import path) | The v0.26.0 summary envelope didn't carry account either; v0.27.0 doesn't expand surface. BundleJson's `account: u32` field is user-supplied at bundle.rs:693 (`args.account`); wallet-import has no user input here, so emit `0`. File cycle-close FOLLOWUP `wallet-import-derived-account-extraction` if/when this becomes load-bearing for a downstream consumer. **Phase 4 test cell language correction:** the Phase 4 cell description "`bundle.account` reflects descriptor's BIP-48 account index" (in §4.4) is INCONSISTENT with this hardcode lock — Phase 4 reconciles by changing the cell name to `bundle_account_hardcoded_zero_for_v0_27_0_wallet_import` and asserting `bundle.account == 0` regardless of descriptor BIP-48 index |
| `origin_path` / `origin_paths` | derived from `parsed.cosigners[*].path_raw`: shared-path → `origin_path: Some(...)`; divergent-path → `origin_paths: Some(vec![...])`; mutually exclusive per SPEC §5.3 | Match existing descriptor-mode bundle.rs logic |
| `master_fingerprint` | `None` for multisig (always None when N>1); `Some(parsed.cosigners[0].fingerprint.to_string().to_lowercase())` for N=1 | Mirrors live bundle.rs:677-678 emission rule. `bitcoin::bip32::Fingerprint` impls Display (8 lowercase hex) but NOT `LowerHex`; `format!("{:08x}", fingerprint)` would fail to compile |
| `ms1` | from `synthesize_descriptor`'s Bundle.ms1 (already length-N with sentinel forms per SPEC §5.8) | Direct passthrough |
| `mk1` | from `synthesize_descriptor`'s Bundle.mk1 | Direct passthrough |
| `md1` | from `synthesize_descriptor`'s Bundle.md1 | Direct passthrough |
| `multisig` | `Some(MultisigInfo { template, threshold, cosigner_count, path_family, cosigners })` when N>1; `None` for N=1 | template (`&'static str`) = mapped from `script_type_from_descriptor`'s `WalletScriptType` return via that enum's `as_str()` / Display impl — Phase 4 implementer greps for the existing mapping (likely an `impl WalletScriptType` block with `pub fn as_str(&self) -> &'static str` or similar); if no helper exists, add one in the same phase. threshold = `parsed.threshold.unwrap()`; cosigner_count = N; path_family = "bip48" or "bip87" per descriptor parse; cosigners = Vec<CosignerEntry { index, master_fingerprint, origin_path, xpub }> from parsed.cosigners |
| `privacy_preserving` | `false` (v0.27.0 wallet-import never opts in; user can re-derive via subsequent `bundle` invocation with `--privacy-preserving` if desired) | Hardcoded false in this code path |

For Bitcoin Core multi-descriptor input: emit one envelope-array-entry per descriptor, each invoking `synthesize_descriptor` independently against that descriptor's cosigner set.

### §3.3 `InspectJson` schema_version backfill (RepairJson already done)

**Verified at R3:** `RepairJson` at `cmd/repair.rs:153-159` ALREADY has `schema_version: &'static str` as a top-level inline field, set to `"1"` at construct-site line 178. No envelope wrapping needed on the Repair side.

`InspectJson` at `cmd/inspect.rs:244-266` does NOT have `schema_version`. v0.27.0 backfills via wrapper:

```rust
#[derive(serde::Serialize)]
pub struct InspectEnvelope<'a> {
    pub schema_version: &'static str,
    #[serde(flatten)]
    pub body: InspectJson<'a>,    // lifetime confirmed against cmd/inspect.rs:246
}
```

Constant: `pub const INSPECT_SCHEMA_VERSION: &str = "1";`. At "1" — no migration; this is the first version. Mirrors `XpubSearchEnvelope` precedent (`cmd/xpub_search/mod.rs:105-129`).

**FOLLOWUP closure narrative:** The `inspect-json-schema-version-backfill` FOLLOWUP body (FOLLOWUPS.md:48-56) calls out both `InspectJson` AND `RepairJson` envelopes as needing the field. R3 source-verification confirms `RepairJson` is already done. v0.27.0 closes the FOLLOWUP with: *"InspectJson backfilled to `schema_version: \"1\"` via InspectEnvelope wrapper (mirrors XpubSearchEnvelope precedent). RepairJson confirmed to ALREADY carry `schema_version: \"1\"` at cmd/repair.rs:155 + construct site cmd/repair.rs:178 (latent FOLLOWUP-body inaccuracy; closes as no-op for Repair side)."*

**Phase 1 scope reduced:** ship `InspectEnvelope` only. Snapshot tests in `tests/cli_inspect.rs` regenerate (cells 15-17). New cell: assert `schema_version == "1"` for each kind variant (mirrors `cli_xpub_search_path_of_xpub.rs:77-103`). No Repair-side change.

Snapshot tests in `tests/cli_inspect.rs` regenerate (cells 15-17). New cell: assert `schema_version == "1"` for each kind variant (mirrors `cli_xpub_search_path_of_xpub.rs:77-103`).

### §3.4 BIP-129 Round-1 verification engine

**[REVISED — Phase 2 recon pivot; see §8 and `design/agent-reports/v0_27_0-phase-2-bip129-recon.md`.]**

Modules (new files):
- `crates/mnemonic-toolkit/src/wallet_import/bsms_round1.rs` — Round-1 5-line record parser. ~80 LOC.
- `crates/mnemonic-toolkit/src/wallet_import/bsms_verify.rs` — BIP-322 legacy-format ECDSA verify primitive. ~100 LOC.

**Public surfaces (BIP-129-faithful per Q8 recon outcome):**

```rust
// crates/mnemonic-toolkit/src/wallet_import/bsms_round1.rs

/// A parsed BIP-129 Round-1 5-line key record (Signer → Coordinator).
/// Lines 1-4 are the signed body; line 5 is the base64-encoded SIG.
pub(crate) struct BsmsRound1Record {
    pub version: String,        // line 1 — must equal "BSMS 1.0"
    pub token_hex: String,      // line 2 — hex-encoded TOKEN per §2 Q8 recon
    pub key_field: KeyField,    // line 3 — parsed [fp/path]raw_pubkey OR [fp/path]xpub
    pub description: String,    // line 4 — text description, <=80 chars, no '\n'/'\r'
    pub signature_b64: String,  // line 5 — base64-encoded 65-byte recoverable ECDSA sig
}

pub(crate) enum KeyField {
    RawPubkey {
        fingerprint: bitcoin::bip32::Fingerprint,
        path: bitcoin::bip32::DerivationPath,
        pubkey: secp256k1::PublicKey,        // 33-byte compressed
    },
    Xpub {
        fingerprint: bitcoin::bip32::Fingerprint,
        path: bitcoin::bip32::DerivationPath,
        xpub: bitcoin::bip32::Xpub,
    },
}

/// Parse a Round-1 record from text. CRLF→LF normalize; verify line count == 5;
/// verify line 1 == "BSMS 1.0"; parse line 3 dispatched on KEY shape (33-byte
/// hex prefix vs base58 xpub).
pub(crate) fn parse_round1(text: &str) -> Result<BsmsRound1Record, BsmsVerifyError>;

/// Extract the signer's secp256k1 pubkey from a parsed Round-1 record.
/// For RawPubkey variant: returns the raw pubkey directly.
/// For Xpub variant: returns the xpub's OWN embedded pubkey (NOT a child-derived key).
/// Per BIP-129 line 81 + Coinkite Python ref consensus reading.
pub(crate) fn signer_pubkey(record: &BsmsRound1Record) -> secp256k1::PublicKey;
```

```rust
// crates/mnemonic-toolkit/src/wallet_import/bsms_verify.rs

/// Verify a BIP-129 Round-1 key-record signature.
///
/// Per BIP-129 §Round 1 Signer (line 81): the SIG on line 5 is a BIP-322
/// legacy-format ECDSA recoverable signature over the four-line body
/// (lines 1-4 joined by '\n', no trailing newline), under the "Bitcoin
/// Signed Message" double-SHA256 digest, base64-encoded.
///
/// Implementation: uses `bitcoin::sign_message::signed_msg_hash` for the
/// digest, `base64::engine::general_purpose::STANDARD.decode` for line-5
/// decoding, `secp256k1::Secp256k1::recover_ecdsa` for pubkey recovery,
/// asserts recovered_pubkey == signer_pubkey + uses standard `verify_ecdsa`
/// as belt-and-braces.
pub(crate) fn verify_round1_signature(
    record: &BsmsRound1Record,
) -> Result<(), BsmsVerifyError>;
```

**Error sub-enum:**

```rust
pub(crate) enum BsmsVerifyError {
    Round1Malformed { reason: String },
    SignatureMismatch {
        record_index: usize,        // 0-based index in --bsms-round1 multi-flag
        signer_pubkey_hex: String,  // 66-char compressed pubkey hex for the user to identify which signer failed
        reason: String,             // e.g., "ECDSA recover succeeded but pubkey mismatch" vs "ECDSA recover failed"
    },
    Base64Decode { reason: String },
}
```

All wrap into `ToolkitError::BsmsRound1Malformed` / `ToolkitError::BsmsSignatureMismatch` / `ToolkitError::BadInput` at the CLI dispatch layer.

**Test vectors (BIP-129 in-spec; per `design/agent-reports/v0_27_0-phase-2-bip129-recon.md` §Test Vectors):**
- TV-1 Signer 1 (NO_ENCRYPTION/pubkey): TOKEN=`00`, fingerprint=`59865f44`, raw pubkey, expected SIG matches.
- TV-2 Signer 1 (NO_ENCRYPTION/xpub): TOKEN=`00`, xpub, expected SIG matches against xpub's own pubkey.
- TV-3 Signer 1 (STANDARD/xpub): TOKEN=`a54044308ceac9b7`, xpub, expected SIG matches.
- TV-4 Signers 1, 2, 3 (EXTENDED/3-signers, 128-bit TOKEN, mixed signer set): all 3 SIGs match.
- Negative: TV-1 with last-byte-flipped SIG → `SignatureMismatch`.
- Negative: TV-1 with last-byte-flipped TOKEN → `SignatureMismatch` (TOKEN is part of signed body).
- Negative: malformed line-3 KEY (not `[fp/path]hex|xpub` shape) → `Round1Malformed`.
- Negative: malformed line-5 SIG (not base64 → 65 bytes) → `Base64Decode` → wrapped to `Round1Malformed`.
- Negative: line 1 != `BSMS 1.0` (e.g., `BSMS 2.0`) → `Round1Malformed` (NOT `FutureFormat` — parse-side, not blob-side).
- Negative: line-4 description contains `\n` (over-segments parser) → `Round1Malformed`.
- 15 total test cells: 6 positive (TVs) + 5 negative + 4 CLI cells (`--bsms-round1 <FILE>`, stdin `-`, multi-record, `--bsms-verify-strict` mode).

**Phase 2 R0 explicit scope (per Q8):** *"Did the verifier accept all 5 BIP-129 in-spec test-vector Round-1 SIGs (TV-1 Signer 1, TV-2 Signer 1, TV-3 Signer 1, TV-4 Signers 1-3)? Did it correctly reject a TV with a 1-byte-flipped SIG? Did it correctly handle BOTH raw-pubkey KEYs (TV-1) AND xpub KEYs (TV-2 through TV-4)? Did pubkey extraction use the xpub's OWN embedded pubkey (bytes 45-78 of serialized xpub), NOT any child-derived key?"*

**Crate dependency choice (revised):** use `secp256k1` (already in tree via `rust-bitcoin`) for ECDSA recover/verify; `bitcoin::sign_message::signed_msg_hash` for the BIP-129 digest (exact match per Coinkite ref); `base64` (already in tree via `rust-bitcoin`'s feature graph — verify at Phase 2 R0 recon). NO hand-rolled HMAC for the Round-1 path (HMAC is only the encryption-envelope MAC which is out of v0.27.0 scope; see §8 cycle-close FOLLOWUP `bsms-bip129-full-cutover`).

### §3.5 BSMS Round-2 emitter

**[REVISED — Phase 2 recon pivot; see §8.]**

Module: `crates/mnemonic-toolkit/src/wallet_export/bsms.rs` (new file). Implements `WalletFormatEmitter`.

**4-line implementation (BIP-129-canonical Round-2, default; ~80 LOC):**
- Line 1: literal `BSMS 1.0`.
- Line 2: `EmitInputs.canonical_descriptor` (wallet_export/mod.rs:336, `&'a str`) — already carries the `#<checksum>` suffix per existing pipeline convention (used live in sparrow/coldcard/etc. emitters). Emit verbatim. **NO** `md_codec::descriptor_checksum` call — that symbol does not exist; the codebase's checksum-related helper is `miniscript::descriptor::checksum::verify_checksum` (wallet_import/bsms.rs:141), and it's not needed at emit time because the canonical_descriptor is already canonical.
- Line 3: path restrictions. Compute from `EmitInputs.resolved_slots[*].path_raw` + descriptor parse — typical multisig descriptors carry `/0/*,/1/*` (receive + change) or `/0/*` only (receive-only); single-sig BIP-48/BIP-84 etc. expand similarly. Conservative default for v0.27.0: emit `/0/*,/1/*` for descriptor-derived wallets when both receive and change branches are addressable; emit `No path restrictions` for divergent-path multisig where the multi-cosigner origin paths differ and no single shared path family applies. SPEC patch §3.5.1 below codifies the path-emit rule.
- Line 4: first address. Derive at the first index of line 3's restrictions:
  - `/0/*,/1/*` (typical) → first receive address at `/0/0`.
  - `/0/*` only → `/0/0`.
  - `No path restrictions` → `/0/0` (BIP-129 spec convention).
  - For multisig: compute the witness-script address at the indexed derivation point across all cosigners (matches BIP-129 line 96 "the wallet's first address").
- **NEW helper required** by exact name (`derive_address_at_path`), BUT existing address-derivation primitives in `cmd/xpub_search/address_search.rs` (v0.26.0 addition: `scan_xpub_for_addresses` + `render_address<C: Verification>` + the `xpub.derive_pub(secp, &dp)` pattern at line 83) should be the implementation pattern source — Phase 3 extracts / reuses, does NOT re-implement parallel logic. The v0.26.0 FOLLOWUP `bsms-first-address-verify` at `design/FOLLOWUPS.md:2092` filed this exact gap for the BSMS path specifically. Phase 3 R0 recon locks the helper signature + the reuse-pattern citation before implementation. Limited to non-taproot; taproot errors out per §3.1.1. The `bsms-first-address-verify` FOLLOWUP resolves at v0.27.0 cycle close as resolved-by-implementation.

**2-line implementation (lenient excerpt; ~30 LOC):**
- Line 1: `BSMS 1.0`. Line 2: `<canonical_descriptor>#<checksum>`. No path-restrictions, no first-address. Symmetric input form per v0.26.0 SPEC §4 lenient lock.

**6-line emit DROPPED.** Per Q1 + §8: the plan-doc R0-R5 6-line emit shape combined BIP-129 Round-2 plaintext lines + (misframed) "envelope-side HMAC/signature". BIP-129 Round-2 carries no SIG; signatures are on Round-1 (Signer→Coordinator). Replaced by the 4-line BIP-129-canonical Round-2 emit above.

Add `Bsms` to `CliExportFormat` enum + dispatch arms in `cmd/export_wallet.rs`:

```rust
#[value(name = "bsms")]
Bsms,
```

#### §3.5.1 BSMS 4-line path-restrictions emit rule (load-bearing)

For `EmitInputs.resolved_slots` (length N), each slot's `path_raw: String` has shape `[<fingerprint>/<bip32-origin-path>]`. The descriptor itself carries the per-cosigner receive/change branch suffix (`/0/*,/1/*` typically).

Phase 3 implementer must:
1. Parse `EmitInputs.canonical_descriptor` via `miniscript::Descriptor::<DescriptorPublicKey>::from_str` to extract the per-key suffix (the `<MULTIPATH>` after the xpub).
2. Per BIP-129 line 96: "comma-separated list of derivation path restrictions" with non-hardened paths.
3. Common cases:
   - Multisig with `<0;1>/*` multipath → emit `/0/*,/1/*` on line 3.
   - Multisig with `/0/*` receive-only → emit `/0/*`.
   - Single-sig with explicit BIP-48/84/86/87 family → emit `/0/*,/1/*` (the toolkit's convention is to emit both branches).
   - Divergent or unrecognized → emit `No path restrictions`.
4. Reject Taproot descriptors with `BadInput` per BIP-386 not in BIP-129 prerequisites.

**Test cells (8 total — REVISED):**
- `bsms_4line_emit_2of2_wsh_sortedmulti_mainnet`
- `bsms_4line_emit_2of3_wsh_multi_testnet`
- `bsms_4line_emit_sortedmulti_3of5`
- `bsms_4line_path_restrictions_emits_slash_0_star_slash_1_star_for_multipath`
- `bsms_4line_first_address_byte_exact_against_descriptor_derivation` (cross-checks `/0/0` address against `cmd/xpub_search/address_search.rs` pattern)
- `bsms_4line_taproot_descriptor_errors_explicit_deferred`
- `bsms_2line_lenient_excerpt_emits_descriptor_only`
- `bsms_4line_then_import_byte_exact_idempotent` (regression: emit → 2-line/6-line lenient parser ingests cleanly; v0.27.0 ingest does NOT add 4-line parser; that's `bsms-bip129-full-cutover` FOLLOWUP)

### §3.6 `bundle --import-json` consumer

Wire-up in `crates/mnemonic-toolkit/src/cmd/bundle.rs`:
- Add to `BundleArgs` clap struct: `pub import_json: Option<String>` + `pub import_json_index: Option<usize>`.
- Extend `ArgGroup` mutual-exclusion with existing template/descriptor inputs.
- In `run()`: when `--import-json` present, parse the JSON envelope → extract `bundle.descriptor` (or decode `bundle.md1[0]` if descriptor is None) → extract cosigner xpubs by decoding `bundle.mk1` entries per §3.6.1 → build the `ResolvedSlot` vector → dispatch to `crate::synthesize::synthesize_descriptor(&descriptor, &resolved_slots, privacy_preserving=false)` (synthesize.rs:200; the same descriptor-mode entry point §3.2 locks for the envelope-emit path; consistent with R3 N-C2 fold).
- Seed overlay (existing `--ms1` / `--slot @N.phrase=`) continues to work transparently — applies to slots where envelope's `ms1[i] == ""`. Conflict precedence per Q5 / §3.1.3 — supplying `--ms1` for a slot where envelope `ms1[i] != ""` is `BadInput` exit 2.

**Test cells (10-12):**
- `bundle_import_json_bsms_2line_synthesizes_watch_only_bundle`
- `bundle_import_json_bsms_6line_with_seed_overlay_synthesizes_full_bundle`
- `bundle_import_json_bitcoin_core_multi_descriptor_requires_index`
- `bundle_import_json_bitcoin_core_index_picks_correct_descriptor`
- `bundle_import_json_with_template_flag_errors_mutex`
- `bundle_import_json_with_descriptor_flag_errors_mutex`
- `bundle_import_json_with_account_errors`
- `bundle_import_json_with_ms1_overlay_on_seeded_slot_errors_conflict`
- `bundle_import_json_stdin_dash_reads_envelope`
- `bundle_import_json_index_out_of_bounds_errors`
- `bundle_import_json_verify_bundle_round_trip_self_check` (R0-scope per memory [[feedback-verify-bundle-round-trip-per-phase-r0-scope]])

#### §3.6.1 mk1 → `ResolvedSlot` decode contract (load-bearing)

The consumer flags (`bundle --import-json` and `export-wallet --from-import-json`) extract cosigners from the envelope's `bundle.mk1`. This is the inverse of `synthesize_descriptor`'s mk1 encoding (synthesize.rs:219-255). The decode contract:

**Inputs.** `envelope.bundle.mk1: MkField` + `envelope.bundle.multisig: Option<MultisigInfo>` + `envelope.bundle.descriptor: Option<String>`.

**Dispatch:**
- `MkField::Single(chunks)` → N=1 (single-sig). One decode chain: `mk_codec::decode(&chunks) -> Result<KeyCard, ...>` (live API per 10+ call sites including `inspect.rs:177`, `verify_bundle.rs:1180`, `xpub_search/target_intake.rs:28`). Produces ONE `ResolvedSlot`. **Note:** BSMS Round-2 input always produces `MkField::Multi` (BSMS is multisig-only by spec); the Single branch is reached only via Bitcoin Core single-descriptor input.
- `MkField::Multi(per_cosigner)` → N>1 (multisig). One decode chain PER outer element. Produces N `ResolvedSlot` entries in declaration order.

**Per-cosigner decode** (`mk_codec::KeyCard → ResolvedSlot`). The KeyCard field name is `origin_path` (not `derivation_path`) per `inspect.rs:221` + `inspect.rs:292`:

```rust
fn mk1_card_to_resolved_slot(card: &mk_codec::KeyCard, index: u8) -> ResolvedSlot {
    let fingerprint = card.origin_fingerprint
        .unwrap_or_else(|| card.xpub.fingerprint()); // privacy_preserving==true fallback
    ResolvedSlot {
        xpub: card.xpub,                          // Xpub (typed)
        fingerprint,
        path: card.origin_path.clone(),           // KeyCard.origin_path is ALREADY bitcoin::bip32::DerivationPath per mk-codec-0.3.1/src/key_card.rs:42 — no conversion needed
        path_raw: format!("[{}/{}]",              // canonical re-serialization mirroring wallet_import/bsms.rs:179 build_slot_fields pattern
            fingerprint.to_string().to_lowercase(),
            card.origin_path.to_string().trim_start_matches("m/")),
        entropy: None,                            // mk1 carries no entropy; envelope's parallel ms1[i] determines seed-bearing state
        master_xpub: None,                        // mk1 carries the cosigner xpub at derivation path, not the master
        _entropy_pin: None,                       // ResolvedSlot 7th field at synthesize.rs:619; watch-only slots never carry a pin; mirrors wallet_import/bsms.rs:183-191 construction site
    }
}
```

**Privacy-preserving caveat.** When the original synthesizer was called with `privacy_preserving: true`, the mk1 omits `origin_fingerprint`. v0.27.0 wallet-import path always synthesizes with `privacy_preserving=false` (per §3.2.1), so envelopes produced by `import-wallet --json` always carry fingerprints. But the consumer (`bundle --import-json` / `export-wallet --from-import-json`) MAY consume a hand-crafted envelope or one passed through an intermediate tool. If `mk_codec::KeyCard.origin_fingerprint.is_none()` on decode, fall back to `card.xpub.fingerprint()` (the xpub-derived fingerprint — semantically equivalent for sortedmulti).

**Decode error handling.** `mk_codec::decode` returns `Result<KeyCard, mk_codec::DecodeError>`. Map decode failures to `ToolkitError::BadInput { reason: format!("--import-json: mk1[{i}] decode failed: {e}") }` exit 2. Single-chunk corruption in a multi-cosigner envelope is fatal (do NOT attempt repair from the consumer side; the envelope is supposed to be canonical).

**Test cells** (added to §3.6's enumeration):
- `bundle_import_json_mk1_single_decodes_to_single_slot`
- `bundle_import_json_mk1_multi_decodes_to_n_slots_in_declaration_order`
- `bundle_import_json_mk1_corrupted_chunk_errors_bad_input`
- `bundle_import_json_mk1_privacy_preserving_no_fingerprint_falls_back_to_xpub_derived`

### §3.7 `export-wallet --from-import-json` consumer

Wire-up in `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`:
- Add to `ExportWalletArgs`: `pub from_import_json: Option<String>` + `pub from_import_json_index: Option<usize>`.
- Mutual-exclusion with template/descriptor inputs.
- In `run()`: parse JSON envelope → extract `bundle.descriptor` + decode `bundle.mk1` per §3.6.1 → construct `EmitInputs` (16 fields — see §3.7.1 below) → dispatch to existing per-format emitter.
- `--account` supplied → `BadInput` (per Q6).

#### §3.7.1 EmitInputs construction (17-field contract)

**[REVISED post-Phase-3 — field count 16 → 17 (Phase 3 added `bsms_form`); see Phase 4 holistic review I/M1 fold.]**

`EmitInputs<'a>` at `wallet_export/mod.rs:333-389` has **17 fields**. Phase 5 implementer constructs all 17 from the envelope + Phase 5 helpers. Mirror the existing construction site in `cmd::export_wallet::run` for defaults.

| EmitInputs field | Source / value | Notes |
|---|---|---|
| `canonical_descriptor: &'a str` | From envelope `bundle.descriptor.as_ref()` (Phase 5 lifetime: borrow from a String owned in the run-scope) | Required `Some` for v0.27.0 wallet-import path (descriptor-mode always emits Some per §3.2.1) |
| `resolved_slots: &'a [ResolvedSlot]` | From §3.6.1 mk1 decode (owned in run-scope, borrowed for emit call) | Same lifetime story |
| `template: Option<CliTemplate>` | `None` | `CliTemplate` enum, NOT `&'static str` (which is BundleJson.template's type); descriptor-mode wallet-import always `None` |
| `script_type: WalletScriptType` | Two-step derivation: (1) `miniscript::Descriptor::<DescriptorPublicKey>::from_str(&envelope.bundle.descriptor.unwrap())` (2) `script_type_from_descriptor(&parsed_ms_descriptor)?` returns `WalletScriptType` (wallet_export/mod.rs:182) | Phase 5 NEW work: ensure miniscript parse succeeds for the descriptor flavors v0.27.0 supports; if parse fails (e.g., non-standard syntax), error loudly as `BadInput` |
| `network: CliNetwork` | **NEW helper required:** `fn cli_network_from_bitcoin_network(n: bitcoin::Network) -> CliNetwork` (Phase 5 adds) | No `impl From<bitcoin::Network> for CliNetwork` exists; envelope's `bundle.network: &'static str` → `bitcoin::Network` (via reverse `network_human_name`) → `CliNetwork` is the conversion chain |
| `account: u32` | `envelope.bundle.account` | Direct passthrough (always 0 for v0.27.0 wallet-import per §3.2.1) |
| `threshold: Option<u8>` | `envelope.bundle.multisig.as_ref().map(\|m\| m.threshold)` | `None` for single-sig (envelope's `multisig: None`) |
| `threshold_user_supplied: bool` | `false` | User didn't pass `--threshold`; envelope-derived |
| `master_xpub_at_0: Option<Xpub>` | `None` | Not envelope-derivable; existing default |
| `wallet_name: &'a str` | `"imported"` (default; `&'static str` literal) OR user-supplied `--wallet-name` (borrowed from clap arg) | Lifetime-sound because both are owned by longer-lived scopes |
| `wallet_name_was_user_supplied: bool` | Derived from clap: `args.wallet_name.is_some()` | Mirrors existing construction |
| `taproot_internal_key: Option<TaprootInternalKey>` | `None` for v0.27.0; Phase 5 errors loudly if envelope's descriptor is `tr(...)` (file FOLLOWUP `wallet-import-taproot-internal-key`) | Defers tr() consumer-side handling |
| `range: (u32, u32)` | `(0, 999)` (existing default per `cmd/export_wallet.rs:100-102`) | Source-verified literal: clap `default_value = "0,999"` |
| `timestamp: TimestampArg` | `TimestampArg::Now` (unwrap from `TimestampArgValue::Now`; args struct field at `cmd/export_wallet.rs:106` is `TimestampArgValue`; EmitInputs field expects `TimestampArg` — one newtype unwrap needed at Phase 5 wiring) | Source-verified default `now` |
| `bitcoin_core_version: u8` | `25` (existing default per `cmd/export_wallet.rs:108-110`) | Source-verified literal: clap `default_value = "25"`; doc-comment "24 or 25 (default 25)" |
| `bsms_form: BsmsForm` | **Phase 3 addition (post-original-plan).** `BsmsForm::default()` is acceptable when emitting non-BSMS formats; for BSMS emitter, defaults to `BsmsForm::FourLine` (BIP-129 canonical). When `--bsms-form` is supplied on the command line, it overrides. The wallet-import-emit path: pass through from clap (default `BsmsForm::FourLine` for `export-wallet --format bsms`; ignored for non-BSMS emitters). | Source: `wallet_export/mod.rs` near top of `EmitInputs` (Phase 3 ship `4a2b6e7`) |

**Phase 5 R0 explicit scope** (architect EDIT 6 fold + Phase 4 holistic M1 fold): enumerate all **17** fields against the live `cmd::export_wallet::run` construction site; assert no field is omitted or defaulted incorrectly. Architect's structural review confirmed the live struct has 17 fields after Phase 3 added `bsms_form`; any plan-doc that enumerates fewer compile-errors.

**Test cells (10-12):**
- `export_wallet_from_import_json_bsms_to_sparrow_emits_valid_sparrow` (headline integration cell)
- `export_wallet_from_import_json_bsms_to_jade_emits_valid_jade`
- `export_wallet_from_import_json_bsms_to_coldcard_emits_valid_coldcard`
- `export_wallet_from_import_json_core_to_bsms_emits_valid_bsms_2line`
- `export_wallet_from_import_json_core_to_specter`
- `export_wallet_from_import_json_with_account_errors`
- `export_wallet_from_import_json_with_template_errors_mutex`
- `export_wallet_from_import_json_with_descriptor_errors_mutex`
- `export_wallet_from_import_json_unsupported_script_type_falls_back_to_descriptor_passthrough`
- `export_wallet_from_import_json_multi_descriptor_requires_index`
- `cross_format_bsms_to_sparrow_to_import_round_trip` (R0-scope per memory: round-trip through verify-bundle if applicable)

---

## §4. Implementation Plan

### §4.0 Phase ordering

Six phases + cycle close. Each phase ends with R0 opus architect review → fold-and-commit. Per-phase reviews persist to `design/agent-reports/phase-N-r0-review.md` (CLAUDE.md line 30 convention; v0.26.0 compare-cost violated this and a FOLLOWUP was filed to back-fill — v0.27.0 follows the discipline from Phase 1).

```
Phase 1: trivial folds                  → #3 InspectJson + #4 runbook move          [SHIPPED — e908309]
Phase 2: BIP-129 Round-1 verify engine  → #2 bsms-verify-signatures (--bsms-round1 CLI)
Phase 3: BSMS emitter (independent #2)  → #1 wallet-export-bsms-emitter (4-line + 2-line)
Phase 4: import-wallet --json envelope  → #5 envelope-full-bundle (BundleJson contract)
Phase 5: consumer wiring                → #6 bundle --import-json + #7 export-wallet --from-import-json
Phase 6: manual mirror + cycle close    → docs/manual/ + CHANGELOG + FOLLOWUPS Status flips + release-branch + tag
```
*(R6 pivot: Phase 2 + Phase 3 reframed per Phase 2 recon; see §8 for diff vs R5.)*

**Phase 1 placement at start (opus R0 D5 lock):** clean baseline; ensures `schema_version` envelope wrappers are in place before Phase 4 introduces another envelope.

**Per-phase commit-shape brief MUST INCLUDE (opus R0 I8 fold):** "and flip `Status: open` → `Status: resolved` for FOLLOWUP `<slug>` in `design/FOLLOWUPS.md` in the same commit." The Phase 6 audit is a backstop, not the sole site (per memory [[feedback-per-phase-agents-forget-followup-status-flip]]).

### §4.1 Phase 1 — Trivial folds (#3 + #4)

**Scope:**
- #3: Add `InspectEnvelope` + `RepairEnvelope` wrappers in `src/cmd/inspect.rs` + (location TBD) `src/repair.rs`. Constants `INSPECT_SCHEMA_VERSION = "1"` + `REPAIR_SCHEMA_VERSION = "1"`. Regenerate snapshot tests (cells 15-17 in `tests/cli_inspect.rs`). Add 1 new cell asserting `schema_version == "1"` per kind variant.
- #4: `git mv .v0_26_0-merge-plan.md design/PLAN_v0_26_0_three_way_merge.md`. Add header note "Canonical record per `coordinator-runbook-into-design-dir` FOLLOWUP." Add 1-bullet to `CLAUDE.md` Conventions: `Multi-instance coordination playbook: see design/PLAN_v0_26_0_three_way_merge.md.`
- Add 1 presence-smoke test cell that asserts `design/PLAN_v0_26_0_three_way_merge.md` exists (catches future churn).

**R0 dispatch:** opus `feature-dev:code-reviewer` on the staged diff + plan-doc §4.1 verbatim citation. Expected findings: low (these are trivial).

**Commit shape:**
- `feat(inspect): add schema_version: "1" to InspectJson + RepairJson envelopes (closes inspect-json-schema-version-backfill)` + flip Status in FOLLOWUPS.md.
- `docs(coordinator): promote merge-plan to design/PLAN_v0_26_0_three_way_merge.md (closes coordinator-runbook-into-design-dir)` + flip Status.

### §4.2 Phase 2 — BIP-129 Round-1 verify engine (#2)

**[REVISED — Phase 2 recon pivot; see §8.]**

**Phase 2 recon is COMPLETE** (persisted at `design/agent-reports/v0_27_0-phase-2-bip129-recon.md`). BIP-129 §Specification → Round 1 mechanics are fully pinned. Implementer goes straight to code.

**Scope:**
- New modules:
  - `src/wallet_import/bsms_round1.rs` — 5-line Round-1 record parser (~80 LOC).
  - `src/wallet_import/bsms_verify.rs` — BIP-322 ECDSA recoverable verify primitive (~100 LOC).
- New CLI flags on `import-wallet`: `--bsms-round1 <FILE|->` (repeating), `--bsms-verify-strict` (bool).
- New `ToolkitError` variants: `BsmsRound1Malformed { reason: String }`, `BsmsSignatureMismatch { record_index: usize, signer_pubkey: String, reason: String }`. Inserted at end of enum per existing convention; `match self { ... }` blocks update in lockstep.
- `cmd/import_wallet.rs` integration: when `--bsms-round1` supplied (one or more), each record is parsed + verified BEFORE existing `--blob` parsing (or independently if no `--blob`). Per-record verify state flows into the `--json` envelope's NEW `bsms_round1_verifications: Vec<{ index, signer_pubkey, signature_verified }>` field.
- **Phase 2 R0 explicit scope (per Q8):** *"Did the verifier accept all 5 BIP-129 in-spec test-vector Round-1 SIGs (TV-1 Signer 1, TV-2 Signer 1, TV-3 Signer 1, TV-4 Signers 1-3)? Did it correctly reject a TV with a 1-byte-flipped SIG? Did it correctly handle BOTH raw-pubkey KEYs (TV-1) AND xpub KEYs (TV-2 through TV-4)? Did pubkey extraction use the xpub's OWN embedded pubkey (bytes 45-78 of serialized xpub), NOT any child-derived key?"*

**Test cells (15):** per §3.4 enumeration above.

**Crate deps:** verify at Phase 2 R0 — `secp256k1` (via `rust-bitcoin`) for ECDSA recover/verify; `bitcoin::sign_message::signed_msg_hash` for the BIP-129 digest; `base64` for line-5 decode. The toolkit's `Cargo.toml` already lists `bitcoin = "0.32"` (line 32) and `sha2 = "0.10"` (line 29). `base64` access via `bitcoin`'s public re-exports or direct `base64` dep — Phase 2 R0 verifies (if direct dep needed, add as workspace dependency).

**R0 dispatch:** opus on the bsms_round1 + bsms_verify modules + integration + new error variants. Expected findings: medium — opus must confirm TV alignment (all 5 in-spec SIGs verify) + cross-impl smoke against Coinkite Python ref.

### §4.3 Phase 3 — BSMS Round-2 emitter (#1)

**[REVISED — Phase 2 recon pivot; see §8.]**

**Scope:**
- New module `src/wallet_export/bsms.rs` implementing `WalletFormatEmitter` (trait at wallet_export/mod.rs:322-326: `collect_missing`, `emit`, `extension`).
- **NEW helper `derive_address_at_path` (Phase 3 in-cycle work; closes design/FOLLOWUPS.md:2092 `bsms-first-address-verify`).** Signature pinned at Phase 3 R0 recon.
- CLI: `--bsms-form 2-line|4-line` (default `4-line`; explicit override always honored).
- 4-line (default): BIP-129-canonical Round-2 plaintext. Lines: `BSMS 1.0` / `<descriptor>#<checksum>` / `<path-restrictions>` / `<first-address>`. Pulls `EmitInputs.canonical_descriptor` for line 2, computes path-restrictions per §3.5.1, derives first address via `derive_address_at_path`.
- 2-line: lenient excerpt; only `BSMS 1.0\n<descriptor>#<checksum>\n`.
- **6-line emit DROPPED** (no plan-doc invention; see §8). Independent of Phase 2's verify engine.
- Add `Bsms` to `CliExportFormat` enum + dispatch arms in `cmd/export_wallet.rs`.

**Test cells (8):** per §3.5 enumeration above.

**R0 dispatch:** opus on the emitter + dispatch wiring + `derive_address_at_path` helper. Expected findings: low-medium. Key risk: path-restrictions rule (§3.5.1) correctness for multipath vs single-branch descriptors — confirm v0.27.0 errors loudly for tr() rather than emitting garbage.

### §4.4 Phase 4 — import-wallet --json envelope (#5)

**Scope:**
- **Phase 4 prerequisite task:** add field `original_descriptor: String` to `ParsedImport` (wallet_import/mod.rs:57); populate at parse time from the **pre-strip raw descriptor including `#<checksum>`** (BSMS Round-2 line 2 verbatim; Bitcoin Core `desc` JSON field verbatim) — NOT from `descriptor_body_no_csum`. This unblocks §3.2.1 row `descriptor`. **Disjoint use:** `ParsedImport.descriptor: md_codec::Descriptor` (the existing field, `#[allow(dead_code)]` removed this phase) is the input to `synthesize_descriptor` (the typed shape); `ParsedImport.original_descriptor: String` (NEW) is the envelope wire-shape carry for downstream consumers. They are two siblings with disjoint uses.
- Rewrite `emit_json_envelope` in `cmd/import_wallet.rs` (lines 325-405).
- Surface `bundle: BundleJson` (synthesized post-parse via `synthesize_descriptor(&parsed.descriptor, &parsed.cosigners, false)`; synthesize.rs:200).
- Surface `schema_version: "1"` at outer envelope.
- Remove `ParsedImport.descriptor`'s `#[allow(dead_code)]` (becomes load-bearing).
- Promote `network_human_name` from private (cmd/import_wallet.rs:491) to `pub(crate)` so synthesize-envelope path can call it.
- For Bitcoin Core multi-descriptor input: one envelope-array-entry per descriptor.
- **NEW helper (Phase 4 in-cycle):** if `WalletScriptType → &'static str` mapping doesn't already exist in the codebase, add `impl WalletScriptType { pub fn as_static_str(&self) -> &'static str }` (or equivalent Display impl); Phase 4 implementer greps for existing pattern before adding.

**Test cells (7-8):**
- BSMS 2-line input → envelope's `bundle.mode == "watch-only"`.
- BSMS 2-line input → `bundle.ms1 == ["", "", ...]` (length N sentinel array per SPEC §5.8).
- BSMS 2-line input → `bundle.mk1` array decodes back to original cosigner xpubs.
- BSMS 2-line input → `bundle.descriptor.is_some()` (descriptor-mode).
- Bitcoin Core multi-descriptor → array of envelopes, one per descriptor.
- `bundle_account_hardcoded_zero_for_v0_27_0_wallet_import`: assert `bundle.account == 0` regardless of descriptor BIP-48 account index (v0.27.0 lock per §3.2.1 row `account`).
- v0.27.0 round-trip via verify-bundle: `import-wallet --json | jq '.[0].bundle' | mnemonic verify-bundle --bundle-json -` succeeds (R0-scope per memory [[feedback-verify-bundle-round-trip-per-phase-r0-scope]]).
- v0.27.0 envelope wire-shape fixture test: capture the v0.27.0 envelope shape (a sample BSMS 2-line import) into `tests/fixtures/wallet_import/envelope_v0_27_0.json` (hand-rolled JSON fixture per project convention — verified at R3 that the repo has no existing `.snap` files and no `insta` dev-dep). The cell does a byte-exact `assert_eq!(emitted, expected_fixture)` after `serde_json::to_string_pretty` of the actual envelope. This pins the wire shape against accidental drift and serves as the v0.26→v0.27 change-witness in CHANGELOG. Place fixture beside existing BSMS input fixtures (`tests/fixtures/wallet_import/bsms-*.txt`).

**Phase 4 R0 explicit scope item:** *"Exercise `import-wallet --json` output through `verify-bundle --bundle-json -`; if synthesis is lossy vs source descriptor, that's a Critical finding."* (Per memory [[feedback-verify-bundle-round-trip-per-phase-r0-scope]].)

**R0 dispatch:** opus on the envelope rewrite. Expected findings: medium — synthesize_unified wiring is non-trivial.

### §4.5 Phase 5 — consumer wiring (#6 + #7)

**Scope:**
- `bundle --import-json` + `--import-json-index` (per §3.6).
- `export-wallet --from-import-json` + `--from-import-json-index` (per §3.7 + §3.7.1's **17-field** EmitInputs contract — post-Phase-3-fold; the original plan-doc said 16-field, which became stale when Phase 3 added `bsms_form`).
- Shared helper: `crates/mnemonic-toolkit/src/wallet_import/json_envelope.rs` (new) — parses an `import-wallet --json` envelope element into a typed struct `ImportJsonEnvelope` + provides `envelope_to_resolved_slots(envelope) -> Vec<ResolvedSlot>` (decodes mk1 entries per §3.6.1) + `infer_emit_inputs_from_envelope(envelope, args) -> EmitInputs` (constructs all **17** fields per §3.7.1).
- **Phase 5 deserialization strategy (Phase 4 holistic review I1 fold).** `crate::format::BundleJson` is `Serialize`-only at `format.rs:119-145` (no `Deserialize` impl) and carries `&'static str` fields (`schema_version`, `mode`, `network`, `template: Option<&'static str>`, `multisig.template`) plus a `#[serde(untagged)]` Serialize-only `MkField`. **Phase 5 CANNOT deserialize directly into `BundleJson`.** The original §4.5 mention of `#[serde(deserialize_with = ...)]` is incomplete. Two acceptable patterns:
  - **(a) Parallel `BundleJsonView` mirror struct (RECOMMENDED for Phase 5).** Define in `wallet_import/json_envelope.rs` a mirror with `String` (or `Cow<'static, str>`) fields matching the wire shape one-for-one; derive `Deserialize`. `ImportJsonEnvelope` uses `BundleJsonView` for its `bundle:` field. Scales cleanly to all 13 BundleJson fields + the multisig sub-shape; mirrors the SemVer-decoupling convention (Phase 5's parser doesn't need to track future BundleJson field additions field-by-field).
  - **(b) `serde_json::Value` traversal (existing precedent).** `verify_bundle.rs:980-1010` walks `serde_json::Value` to extract `ms1/mk1/md1` fields manually. Works fine for the narrow card-list extraction; would be verbose for the full 13-field envelope.
  - **Phase 5 R0 explicit scope (Phase 4 holistic I1 fold):** confirm the chosen pattern matches the implementation, AND that taproot rejection happens at the typed-deser layer rather than panic'ing on a downstream miniscript parse.
- **NEW helper (Phase 5 in-cycle):** `fn cli_network_from_bitcoin_network(n: bitcoin::Network) -> CliNetwork` (no existing `impl From` exists). Phase 5 R0 scope: confirm helper covers all 4 variants (Mainnet/Testnet/Signet/Regtest); reject unknown variant with `BadInput`.

**Test cells (~22):** per §3.6 + §3.7 enumerations.

**Integration cell (1, cross-phase):** `cross_format_bsms_to_sparrow_round_trip`:
1. Start with a BSMS Round-2 blob fixture.
2. `import-wallet --format bsms --blob <fixture> --json` → capture stdout envelope.
3. Pipe envelope into `export-wallet --from-import-json - --format sparrow` → capture Sparrow JSON output.
4. Assert Sparrow output parses as valid Sparrow wallet config (descriptor matches, cosigner xpubs match, threshold matches).

This is the headline end-user feature for v0.27.0.

**Phase 5 R0 explicit scope items (per memory + opus R0 I1):**
- *"Exercise `bundle --import-json X | verify-bundle --bundle-json -` round-trip; if synthesis is lossy vs envelope, that's a Critical finding."*
- *"Confirm clap-derive mutex grouping enforces `--import-json` ↔ `--template`/`--descriptor`/`--descriptor-file` exclusivity (run the explicit-conflict test cells)."*

**R0 dispatch:** opus on full Phase 5 wiring + integration cell. Expected findings: medium-high — clap-derive mutual exclusion is finicky.

### §4.6 Phase 6 — manual mirror + cycle close

**[REVISED — Phase 2 recon pivot; see §8.]**

**Scope:**
- Update `docs/manual/src/40-cli-reference/41-mnemonic.md`. Explicit flag enumeration to verify against `lint.sh`:
  1. `mnemonic export-wallet --bsms-form` (new flag; values `2-line|4-line`)
  2. `mnemonic export-wallet --from-import-json` (new flag)
  3. `mnemonic export-wallet --from-import-json-index` (new flag)
  4. `mnemonic import-wallet --bsms-round1` (new flag — repeating)
  5. `mnemonic import-wallet --bsms-verify-strict` (new flag)
  6. `mnemonic bundle --import-json` (new flag)
  7. `mnemonic bundle --import-json-index` (new flag)
  8. `mnemonic inspect` — document `schema_version` field on envelope
  9. `mnemonic repair` — document `schema_version` field on envelope (note: already existed at v0.26.0; document the existing behavior)
- **Seven** new flags (revised from 8: dropped `--coordinator-hmac-key`; added `--bsms-round1`; `--bsms-form` value-set changed) + 2 envelope schema_version documentations. The lint check at `docs/manual/tests/lint.sh` will fail if any flag is missing.
- New format addition: `mnemonic export-wallet --format bsms` (new `--format` value; 4-line BIP-129-canonical default, 2-line lenient via `--bsms-form 2-line`).
- New recipe chapter `docs/manual/src/30-workflows/3X-cross-format-conversion.md` walking BSMS → Sparrow end-to-end.
- New recipe chapter `docs/manual/src/30-workflows/3Y-bsms-round1-verify.md` walking BIP-129 Round-1 SIG verification (`--bsms-round1` use cases).
- CHANGELOG.md: `### Added` (BSMS Round-2 emitter [4-line BIP-129-canonical default + 2-line lenient], --bsms-form, --bsms-round1, --bsms-verify-strict, --import-json, --from-import-json, --import-json-index, --from-import-json-index) + `### Changed` (import-wallet --json envelope shape: bundle field replaced from summary to BundleJson; mention SemVer minor-bump justification) + `### Closed FOLLOWUPS` entries (5 closed: `wallet-export-bsms-emitter`, `bsms-verify-signatures`, `inspect-json-schema-version-backfill`, `coordinator-runbook-into-design-dir`, `wallet-import-json-envelope-full-bundle` — items #6 and #7 are NEW features, not FOLLOWUP closures).
- Sweep `design/FOLLOWUPS.md` for any `Status: open` entries that the per-phase commits cited as Resolved (per memory [[feedback-per-phase-agents-forget-followup-status-flip]] — backstop check).
- Bump `Cargo.toml` workspace version to `0.27.0`.
- `pinned-upstream.toml` (mnemonic-gui sibling repo) — NOT touched by this cycle; GUI consumer cycle picks it up.
- Create release branch `release/v0.27.0`. Single squash PR. Tag `mnemonic-toolkit-v0.27.0`. GitHub release with full notes.

**File new FOLLOWUPS (at least 3):**
- `cross-format-conversion-matrix-expansion` — N×M coverage for the 7+ format combinations beyond the BSMS→Sparrow integration cell.
- `bsms-taproot-emit` (revised from `bsms-taproot-6-line`) — BIP-129 emit for tr() descriptors. Blocked on BIP-129 adding BIP-386 to prerequisites (not yet specified).
- `bsms-bip129-full-cutover` — v0.28+: deprecate v0.26.0 6-line lenient parser; add proper 4-line Round-2 parser; add encryption-envelope (STANDARD/EXTENDED with PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 MAC) support; possibly drop 6-line and 2-line lenient shapes after deprecation window. Filed pre-emptively at v0.27.0 cycle close — see §8 pivot record.

**R0 dispatch (end-of-cycle holistic):** opus full-cycle review of release branch. Catches manual-mirror gaps, missing FOLLOWUPS flips, CHANGELOG completeness.

---

## §5. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| BIP-129 §5 test vectors don't exist or are ambiguous | medium | high (verifier correctness) | Phase 2 R0 explicit scope: cite TV source or external reference. Plan-doc does NOT lock formula inline (per opus R0 C2 fold). |
| BSMS Round-2 emit for taproot tr() unspecified in BIP-129 (BIP-386 not in BIP-129 prerequisites) | medium | low (rare format) | Explicitly error on tr() with `--format bsms` in v0.27.0; FOLLOWUP `bsms-taproot-emit` for v0.28+. |
| envelope replacement breaks downstream mnemonic-gui parser | high | medium | v0.27.0 is wire-shape replacement, NOT additive. CHANGELOG `### Changed`. GUI's `pinned-upstream.toml` not bumped until GUI cycle explicitly adopts v0.27.0 envelope (next GUI cycle picks it up; gui-schema auto-emit handles flag additions). |
| clap-derive mutex extension breaks existing flag combinations | medium | medium | Phase 5 R0 explicitly enumerates pre-existing flag-combo cells that should continue passing. |
| `wallet-import-fixture-corpus-expansion` recurs as opus finding | high | low | Folded into §2.3 explicit deferral; opus R0 told this is intentional. |
| Per-phase agents forget FOLLOWUPS `Status: open → resolved` flips | high | medium | Explicit per-phase commit-shape brief (per opus R0 I8 fold); Phase 6 sweep is backstop, not sole site. |
| `RepairJson` may not exist as a struct (FOLLOWUP cite may be stale) | medium | low | Phase 1 prerequisite check; ship `InspectEnvelope` only if `RepairJson` proves missing, file FOLLOWUP. |
| Multi-entry envelope index defaults silently to N=0 | low | high (footgun) | Plan locks: absence of `--import-json-index` for multi-entry input is `BadInput` exit 2 (opus R0 D8 lock). |

---

## §6. Verification

### §6.1 Per-phase gates

Each phase passes R0 opus architect review with 0 Critical / 0 Important findings before commit. R0 reviews persist to `design/agent-reports/phase-N-r0-review.md`.

### §6.2 Cycle-level gates

1. **Test suite** — full `cargo test --workspace -- --include-ignored` passes. Baseline ~1153 tests (v0.25.1) + ~50-60 new cells = ~1200-1215 tests in v0.27.0.
2. **Manual lint** — `make -C docs/manual lint MNEMONIC_BIN=... MD_BIN=... MS_BIN=... MK_BIN=...` passes. Per opus R0 I2 fold: lint scope verified against the explicit 8-new-flags-plus-2-schema-version list at §4.6.
3. **gui-schema drift gate** — passes against the new flags (toolkit CI runs against pinned mnemonic-gui v0.11.0 tag; auto-emit covers flag additions; GUI cycle picks up the v0.27.0 envelope when bumping pin).
4. **CLI smoke** — end-to-end cross-format conversion fixture executes via `tests/cli_*roundtrip*` cells.
5. **CHANGELOG audit** — every closed FOLLOWUP has a CHANGELOG line; every CHANGELOG entry maps back to a FOLLOWUP or NEW feature; FOLLOWUPS Status flips align with CHANGELOG closures.
6. **`Cargo.toml` version** — bumped to `0.27.0`.
7. **GitHub release** — tag + release notes attached; CI workflow green.

### §6.3 End-user smoke

Run by user on real hardware before announcing release (smoke recipe, not gated):

```bash
# Step 1: Import a real BSMS bundle fixture
mnemonic import-wallet --format bsms --blob test-fixtures/sparrow-bsms.txt --json > /tmp/env.json

# Step 2: Re-emit as Bitcoin Core listdescriptors
mnemonic export-wallet --from-import-json /tmp/env.json --format bitcoin-core > /tmp/core.json

# Step 3: Round-trip back through import to verify semantic preservation
mnemonic import-wallet --format bitcoin-core --blob /tmp/core.json --json | grep -o '"descriptor":"[^"]*"'

# Step 4: Synthesize an m*1-bundle from the watch-only import (uses the BSMS envelope)
mnemonic bundle --import-json /tmp/env.json --ms1 "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

(Step 3 uses `grep -o` rather than `jq` to avoid an external tool dependency in the smoke recipe.)

If all four steps complete with semantically-matching outputs, the cycle's headline features work end-to-end.

---

## §7. Opus review iteration log (R0 → R1 → R2)

### R0 → R1 fold (3 Critical + 8 Important)

| R0 # | Severity | Finding | R1 fold |
|---|---|---|---|
| C1 | HIGH | §3.2 envelope shape didn't match FOLLOWUP "full BundleJson" contract | §3.2 rewritten to specify `bundle: BundleJson` literal; consumers in §3.6/§3.7 updated to consume BundleJson via mk1 decode |
| C2 | HIGH | §3.4 BIP-129 formula was wrong (conflated KDF + HMAC) | §2.2 Q8 and §3.4 rewritten to NOT lock formula in plan-doc; Phase 2 implementer reads BIP-129 §5 + TVs |
| C3 | HIGH | error.rs "alphabetical" claim was factually wrong | §2.2 Q10 + §4.2 rewritten to "newest at bottom" per existing convention; `error-rs-canonical-ordering-doc` FOLLOWUP stays open |
| I1 | MED | verify-bundle round-trip not in Phase 4/5 R0 scope | §4.4 + §4.5 R0 explicit scope items added |
| I2 | MED | Manual-mirror lint scope incomplete | §4.6 explicit 8-flag-plus-2-envelope enumeration added |
| I3 | MED | Flag naming symmetry | `--bsms-require-signature` renamed to `--bsms-verify-strict`; symmetric with `--bsms-form` |
| I4 | MED | `--account` discipline asymmetric | §3.1.3 + §3.1.4 + Q6 rewritten — both consumers reject `--account` |
| I5 | MED | Envelope entropy precedence undefined | §3.1.3 + §3.6 lock: envelope `ms1[i] != ""` + user `--ms1` for same slot → conflict `BadInput` |
| I6 | LOW-MED | Lifetime parameter claim unverified | §3.3 + Phase 1 prerequisite check added |
| I7 | MED | Deprecated alias migration | Moot under C1 fold (no aliasing under BundleJson contract) |
| I8 | MED | Per-phase Status flip discipline | Per-phase commit-shape briefs in §4.1-§4.5 + §4.0 generic instruction updated |

Minor findings folded inline: cell count reconciliation (§2.1 / §3.5), §6.3 jq dependency removed, multi-entry envelope example added to §3.2.

R0 open questions all resolved (opus answers D1-D10 absorbed into Q1-Q10 locks).

### R1 → R2 fold (3 NEW Critical regressions + 4 Important + 1 Minor)

R1 opus review verified R0 folds (10 of 11 GREEN; 1 YELLOW = C1 partial). It surfaced 3 NEW Critical regressions at the synthesis/decode boundary introduced by R0's C1 fold, plus 4 NEW Important findings and 1 Minor.

| R1 # | Severity | Finding | R2 fold |
|---|---|---|---|
| N-C1 | HIGH | §3.2 schema_version literal was `"2"` (from format.rs:114 doc comment); live construct sites at synthesize.rs:1501 + bundle.rs:693 say `"4"` | §3.2 corrected to `"4"` with explicit citation to both construct sites + the test pin `bundle_json_schema_version_pinned_to_4` at synthesize.rs:1494 |
| N-C2 | HIGH | §3.2 said "invoke `synthesize_unified` against (descriptor, slots)" but `synthesize_unified`'s signature is `(slots, template, threshold, network, privacy_preserving)` — NO descriptor argument; descriptor-mode uses `synthesize_descriptor` (synthesize.rs:200) instead | §3.2 rewritten to lock `synthesize_descriptor` as the load-bearing entry point. NEW subsection §3.2.1 `ParsedImport → BundleJson` field-by-field mapping with 13 explicit rules for each BundleJson field's source |
| N-C3 | HIGH | §3.6 said "decode mk1 entries" without specifying decode contract — `MkField::Single` vs `Multi`, `path_raw` reconstruction, privacy-preserving fingerprint fallback, error handling all under-specified | NEW subsection §3.6.1 `mk1 → ResolvedSlot decode contract (load-bearing)` with dispatch rules, per-cosigner decode pseudocode, privacy-preserving caveat, decode-error mapping, +4 new test cells |
| N-I1 | MED | §3.3 Phase 1 prereq language ("if RepairJson does not exist") created soft-fail path; opus confirmed `RepairJson<'a>` does exist at cmd/repair.rs:154 | §3.3 tightened: both envelopes MUST ship; FOLLOWUP stays partially-open if only one ships |
| N-I2 | MED | §3.4 BIP-129 engine signature TBD too loose; signature drift at Phase 3 R0 would be expensive | §4.2 Phase 2 prefaced with "begins with recon (pre-code)" — read BIP-129 §5 + pin engine signatures before any code |
| N-I3 | MED | §4.4 backward-incompat regression-gate cell ("assert v0.27 envelope does NOT parse v0.26 consumer") is a weak signal | Replaced with positive snapshot-test cell — capture v0.27.0 envelope into `tests/snapshots/import_wallet_envelope_v0_27_0.json.snap` for byte-exact pin |
| N-I4 | MED | `BundleJson.template: Option<&'static str>` lifetime constraint not noted in plan; naïve Phase 4 wiring would not compile | Note added to §3.2 BundleJson shape pseudocode |
| N-M1 | LOW | §4.6 CHANGELOG list said "6 closed FOLLOWUPS"; actual count is 5 (items #6/#7 are NEW features not FOLLOWUP closures) | §4.6 corrected to "5 closed" with explicit enumeration |

**R2 status:** awaits opus R2 architect review. (now superseded by R3 below)

### R2 → R3 fold (4 NEW Critical + 2 Important; root cause: cited helper/field/API names not source-verified)

R2 opus review caught the recurring `[[feedback-r0-must-read-source-off-by-n]]` pattern — plan-doc author reasoned about shape correctly but drifted at API-surface citations. R3 grep-verified every helper / field / API call before re-stating.

| R2 # | Severity | Finding | R3 fold |
|---|---|---|---|
| N1 | HIGH | `network_short_name` doesn't exist — actual is `network_human_name` (cmd/import_wallet.rs:491) | §3.2.1 row `network` corrected with file:line citation + Phase 4 task to promote pub(crate) |
| N2 | HIGH | `format!("{:08x}", fingerprint)` won't compile (Fingerprint impls Display, not LowerHex) | §3.2.1 row `master_fingerprint` corrected to `parsed.cosigners[0].fingerprint.to_string().to_lowercase()` mirroring bundle.rs:677-678 |
| N4 | HIGH | §3.6.1 cited `mk_codec::decode_chunks` (doesn't exist) + `card.derivation_path` (actual field: `origin_path`) | §3.6.1 corrected: `mk_codec::decode(&chunks)` (10+ live call sites) + `card.origin_path` (inspect.rs:221, 292) + path conversion via origin_path_to_derivation_path helper noted |
| N5 | HIGH | RepairJson ALREADY has `schema_version: "1"` inline at cmd/repair.rs:155 + 178; proposed RepairEnvelope wrapper would duplicate the field | §3.3 rewritten: drop RepairEnvelope entirely; ship InspectEnvelope only; close FOLLOWUP with narrative noting RepairJson is already done |
| N3 | MED | `descriptor_account_index` helper invented | §3.2.1 row `account` simplified to `0` hardcoded; FOLLOWUP `wallet-import-derived-account-extraction` filed if needed later |
| N6 | MED | Snapshot-test convention non-existent in repo | §4.4 cell changed to hand-rolled `tests/fixtures/wallet_import/envelope_v0_27_0.json` byte-exact compare (no new dev-dep) |
| N7 | LOW | BSMS Round-2 input never produces `MkField::Single` (multisig-only) | §3.6.1 note added that Single branch reached only via Bitcoin Core single-descriptor input |

**R3 status:** superseded by R4 architect pass below.

### R3 → R4 fold (15 architect EDITs from source-verification pass)

R4 was driven by `feature-dev:code-architect` (opus model) doing a structured source-verification pass — different agent type than R0-R3's `feature-dev:code-reviewer`. The architect identified that prior rounds' "design correct shape but drift at API surface" pattern was fundamentally an authorship problem, not a review problem: the reviewer was doing its job catching drift but the author kept generating it. The architect agent grep-verifies citations as part of its design output, breaking the cycle.

| EDIT # | Severity | Finding | R4 fold |
|---|---|---|---|
| 1 | HIGH | `parsed.descriptor.to_string()` not a confirmed-existing API | §3.2.1 row `descriptor` rewritten — add `original_descriptor: String` field to ParsedImport at parse time; Phase 4 prerequisite task added |
| 2 | HIGH | `md_codec::descriptor_checksum` does not exist | §3.5 2-line implementation rewritten — `EmitInputs.canonical_descriptor` already carries `#<checksum>` suffix; emit directly |
| 3 | HIGH | `derive_address_at_path` doesn't exist (FOLLOWUP `bsms-first-address-verify` already filed at FOLLOWUPS.md:2092) | §3.5 6-line + §4.3 Phase 3 scope updated — NEW in-cycle helper; closes the FOLLOWUP resolved-by-implementation |
| 4 | HIGH | `card.origin_path` is ALREADY `bitcoin::bip32::DerivationPath` (mk-codec-0.3.1/src/key_card.rs:42); no conversion needed; cited helper `origin_path_to_derivation_path` doesn't exist | §3.6.1 pseudocode rewritten — direct assignment `path: card.origin_path.clone()` |
| 5 | HIGH | ResolvedSlot has 7 fields including `_entropy_pin` (synthesize.rs:619); missing initializer is compile-error | §3.6.1 pseudocode adds `_entropy_pin: None` |
| 6 | HIGH | EmitInputs has 16 fields not 8 (wallet_export/mod.rs:333-375) | NEW §3.7.1 — 16-field construction contract with source-verified defaults |
| 7 | HIGH | `script_type_from_descriptor` returns `WalletScriptType` not `&'static str`; takes miniscript Descriptor not md_codec | §3.1.4 + §3.7.1 row `script_type` rewritten — two-step parse + derive |
| 8 | MED | `BundleJson.multisig.template` is `&'static str` but inference returns `WalletScriptType` | §3.2.1 row `multisig` rewritten — add WalletScriptType → &'static str mapping helper (Phase 4 in-cycle) |
| 9 | MED | `bitcoin::Network → CliNetwork` adapter doesn't exist | §3.7.1 row `network` + Phase 5 scope add NEW `cli_network_from_bitcoin_network` helper |
| 10 | MED | `path_raw` format had duplicate `m/` | §3.6.1 pseudocode corrected to `format!("[{}/{}]", fp, path.trim_start_matches("m/"))` |
| 11 | LOW | verify-bundle jq portability note | §4.4 cell language clarified — jq OK in tests, not in §6.3 smoke |
| 12 | LOW | InspectEnvelope<'a> lifetime narrative | §3.3 verified correct; preserved as Phase 1 R0 sanity check |
| 13 | LOW | §6.3 step 4 smoke ms1-overlay precondition | Documented in narrative |
| 14 | LOW | Phase 4 fixture byte-exact comparison path | §4.4 cell language clarified — use serde_json::to_string_pretty same path as production |
| 15 | LOW | BsmsVerifyError sub-enum field naming | §3.4 field names updated to `computed/declared` (drop `_hex` suffix) — symmetric with ToolkitError::BsmsSignatureMismatch |

**R4 status:** YELLOW (1 Critical + 3 Important + 1 Minor); micro-folded into R5.

### R4 → R5 fold (micro-fold; no fresh dispatch)

R4 reviewer (`feature-dev:code-reviewer` opus) returned YELLOW with 4 source-verifiable literal/citation issues. All folded by direct source-grep:

| R4 # | Severity | Finding | R5 fold |
|---|---|---|---|
| B-C1 | HIGH | §3.7.1 `range`/`timestamp`/`bitcoin_core_version` defaults wrong against export_wallet.rs:100-110 | §3.7.1 corrected: range `(0, 999)`, bitcoin_core_version `25`, timestamp newtype unwrap explicit. Source-verified by direct Read of export_wallet.rs:95-110 |
| B-I1 | HIGH | Phase 4 prereq sourcing from checksum-stripped `descriptor_body_no_csum` would break downstream emitters | §3.2.1 row `descriptor` + §4.4 Phase 4 prereq corrected — source from pre-strip raw descriptor (BSMS line 2 / BC `desc` field) verbatim including `#<checksum>` |
| B-I2 | HIGH | "No existing helper" misleading — v0.26.0 added address-derivation primitives in `cmd/xpub_search/address_search.rs` | §3.5 BSMS 6-line + §4.3 Phase 3 cite `cmd/xpub_search/address_search.rs` as the reuse-pattern source (extract/reuse, do NOT re-implement) |
| B-I3 | MED | Disjoint-use distinction between `ParsedImport.descriptor` and new `original_descriptor` not explicit | §4.4 Phase 4 prereq + §3.2.1 row `descriptor` add explicit disjoint-use note: `descriptor` (typed md_codec::Descriptor) → synthesize input; `original_descriptor` (String) → envelope wire carry |
| Minor | LOW | §3.2.1 `account` hardcode contradicts §4.4 test cell description | Renamed cell to `bundle_account_hardcoded_zero_for_v0_27_0_wallet_import`; assertion locked to `== 0` |

**R5 status:** GREEN (ready for ExitPlanMode). The plan has converged through 5 review rounds + 1 architect pass. All Critical findings folded. All Important findings folded or explicitly deferred to per-phase R0 verification (which is project convention). Remaining uncertainty is detail-level (Phase 2 BIP-129 §5 formula recon, Phase 3 address-derive helper signature, Phase 5 TimestampArgValue→TimestampArg unwrap site) — all appropriately deferred per CLAUDE.md "per-phase TDD: tests written before impl. Per-phase reviewer-loop until 0 critical / 0 important."

---

## §8. Phase 2 recon pivot (mid-execution plan revision)

**Status:** R6 — Phase 2 recon outcome; mid-execution plan revision; awaits opus architect validation. Date: 2026-05-18.

**Context.** The R5-GREEN plan-doc was approved via ExitPlanMode and execution began on `release/v0.27.0` branch (Phase 0 at `b47ad2a`, Phase 1 at `e908309`). Phase 2 was scoped per §4.2 R5 to "begin with recon (pre-code): read BIP-129 §5 directly + locate published test vectors + pin engine signature before any code." The recon ran via opus general-purpose agent with WebFetch and surfaced a **major framing error** in the R5 plan: BIP-129 Round-2 carries no SIG; signatures live on Round-1 (Signer → Coordinator) records and are BIP-322 legacy-format ECDSA recoverable signatures, not HMAC-SHA256. The plan-doc's `--coordinator-hmac-key` / per-cosigner HMAC key / 6-line emit framing matched no part of BIP-129 as actually specified.

**Recon artifact:** `design/agent-reports/v0_27_0-phase-2-bip129-recon.md` — full verbatim quotes from BIP-129 §Specification → Round 1 + §Specification → Round 2 + §Encryption + 5 published test vectors + reference-impl survey (Coinkite Python ref is authoritative; no Rust impl exists on crates.io as of 2026-05-18).

**Pivot path chosen (user-directed):** **Path B-lite + new FOLLOWUP for full v0.28+ cutover.** Pivots Phase 2 to BIP-129-faithful Round-1 verify (NEW input path; does not modify v0.26.0 6-line lenient parser). Pivots Phase 3 to BIP-129-faithful 4-line Round-2 emit (drops the plan-doc's 6-line invention). Files a new FOLLOWUP `bsms-bip129-full-cutover` for v0.28+ to deprecate the v0.26.0 6-line lenient input shape, add proper 4-line Round-2 input parsing, and add encryption-envelope support.

### §8.1 Section-by-section diff (old vs revised)

| Section | Old (R5) | Revised (R6) |
|---|---|---|
| §2.2 Q1 | 2-line + 6-line emit; `--coordinator-hmac-key` mandatory for 6-line | 2-line + 4-line emit (default 4-line BIP-129-canonical); 6-line DROPPED |
| §2.2 Q2 | 6-line verify with `--coordinator-hmac-key` + `--bsms-verify-strict` mode | Round-1 5-line verify via NEW `--bsms-round1` input path + `--bsms-verify-strict` mode; v0.26.0 6-line `signature` field stays opaque-stored |
| §2.2 Q8 | "Phase 2 implementer reads BIP-129 §5 directly" (formula not locked) | Phase 2 recon complete; BIP-322 ECDSA recoverable verify; 5 in-spec test vectors enumerated |
| §2.2 Q9 | `--coordinator-hmac-key`, `--bsms-form 2-line\|6-line`, `--bsms-verify-strict` (8 new flags total) | `--bsms-round1`, `--bsms-form 2-line\|4-line`, `--bsms-verify-strict` (7 new flags total) |
| §2.2 Q10 | 3 new ToolkitError variants (BsmsHmacKeyMissing, BsmsSignatureMismatch, BsmsTokenMalformed) | 2 new ToolkitError variants (BsmsRound1Malformed, BsmsSignatureMismatch) |
| §3.1.1 | `--bsms-form 2-line\|6-line` with HMAC-keyed signature on line 6 | `--bsms-form 2-line\|4-line` with BIP-129-canonical Round-2 plaintext (4 lines) |
| §3.1.2 | `--coordinator-hmac-key` keys both emit and verify | `--bsms-round1 <FILE>` separate input path; verify per-record via BIP-322 ECDSA recoverable |
| §3.4 | `derive_per_cosigner_key` + HMAC-keyed `verify_signature` | `bsms_round1` parser + BIP-322 `verify_round1_signature` |
| §3.5 | 2-line + 6-line emit | 4-line BIP-129-canonical + 2-line lenient; +new §3.5.1 path-restrictions rule |
| §4.2 | "Phase 2 begins with recon (pre-code); pin engine signature" | Phase 2 recon COMPLETE (`v0_27_0-phase-2-bip129-recon.md`); implementer goes straight to code |
| §4.3 | 6-line depends on Phase 2 `derive_per_cosigner_key` | 4-line independent of Phase 2 verify path |
| §4.6 | 8 new flags + 2 new FOLLOWUPS at cycle close | 7 new flags + 3 new FOLLOWUPS (added `bsms-bip129-full-cutover`); recipe chapter list expands by 1 (`3Y-bsms-round1-verify.md`) |
| Test-cell totals | Phase 2: 15, Phase 3: 8 | Phase 2: 15 (revised TV set; all BIP-129 in-spec), Phase 3: 8 (revised cell list) |
| §2.1 row 1 (Phase 3 `Depends on`) | `#2 (6-line shape)` | `—` (4-line independent of Phase 2 verify path) |
| §4.0 phase-ordering box | "Phase 3: BSMS emitter (depends on #2) → #1 ... (2-line + 6-line)" | "Phase 3: BSMS emitter (independent #2) → #1 ... (4-line + 2-line)" + Phase 1 marked SHIPPED |
| §5 Risks (tr() row) | "BSMS 6-line m/0/0 derivation differs for taproot tr()" + dead FOLLOWUP slug `bsms-taproot-6-line` | "BSMS Round-2 emit for taproot tr() unspecified in BIP-129" + FOLLOWUP slug `bsms-taproot-emit` |
| §3.6 (`bundle --import-json` synthesis call) | `synthesize_unified` | `synthesize_descriptor` (per R3 N-C2 fold; consistent with §3.2 lock) |

### §8.2 Phase 0-1 commits remain valid

Phase 0 (`b47ad2a` — plan-doc mirror) and Phase 1 (`e908309` — InspectEnvelope + runbook move) are NOT affected by the pivot. They closed the FOLLOWUPS they were scoped to close (`inspect-json-schema-version-backfill` + `coordinator-runbook-into-design-dir`). The plan-doc mirror at `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` is amended in-place; Phase 0 mirrored the R5 plan, and §8 is a R6 revision appended to that file (the revision diff is committed as a separate commit with rationale).

### §8.3 New FOLLOWUP `bsms-bip129-full-cutover`

Filed pre-emptively at the plan-revision commit (NOT deferred to cycle close). v0.28+ work:
- Deprecate the v0.26.0 6-line lenient parser. Add explicit deprecation NOTICE in v0.27.0 (informational only, no behavior change).
- Add proper 4-line Round-2 input parser (BIP-129-canonical Round-2 plaintext ingest).
- Add encryption-envelope (STANDARD/EXTENDED) support: PBKDF2-SHA512(`"No SPOF"`, TOKEN_raw_bytes, c=2048, dkLen=32) → ENCRYPTION_KEY → HMAC_KEY = SHA256(ENCRYPTION_KEY); AES-256-CTR decrypt + HMAC-SHA256 MAC verify.
- Possibly drop 6-line lenient input after a stable-version deprecation window.
- Cross-impl smoke against Coinkite Python ref (`github.com/coinkite/bsms-bitcoin-secure-multisig-setup test.py`).

### §8.4 Architect validation required

This revision (§8 + all in-place §2.2/§3.1.1/§3.1.2/§3.4/§3.5/§4.2/§4.3/§4.6 edits) requires a fresh opus architect dispatch BEFORE Phase 2 code starts. Dispatch must:
- Confirm the recon's BIP-129 claims are spec-faithful (re-read the recon doc + spot-check BIP-129 quotes).
- Validate Path B-lite scope vs Path A (toolkit-local) vs Path B-full (complete cutover) — confirm B-lite is the right cycle-size choice.
- Identify any residual drift between Q1/Q2/Q8/Q9/Q10 locks and §3/§4 details.
- Verify the new FOLLOWUP body adequately scopes the v0.28+ work.

---

**End of plan-doc draft R6 (Phase 2 recon pivot).**
