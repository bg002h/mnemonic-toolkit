# mnemonic-toolkit v0.4.3 implementation plan — verify-bundle finish + unified-path edges

**Cycle scope:** close all 4 v0.4.3-tagged FOLLOWUPS + 1 NEW v0.4.3 (`wif-multisig-resolution`) + opportunistic nits. Theme: **finish verify-bundle (full forensics + descriptor parity + JSON intake) and close the unified-path edges (binding-type merge + wif multisig)**.

**Authoritative SPEC:** `design/SPEC_mnemonic_toolkit_v0_4.md` esp §4.11.b (BIP-388), §5.7 (verify-bundle 9 / 3+6N + forensics), §5.8 (MsField), §6.7 (verify-bundle CLI grammar). v0.4.3 amends in-place via revision-history block.

**Discipline (per `feedback_iterative_review_every_phase`):**
- Per-phase architect review at end-of-phase; iterate to 0C/0I.
- Per-implementation-phase reports persist to `design/agent-reports/phase-<id>-<slug>-review-r<N>.md`.
- L/nit findings → `design/FOLLOWUPS.md` at `v0.4.4-nice-to-have`.
- TDD-first per phase where practical.

## Locked decisions (user-confirmed defaults)

- **Q1 (Phase P aggressiveness): (a) full rollout.** Every `VerifyCheck { ... }` push site at `cmd/verify_bundle.rs` (~78 sites) gets forensic field population where applicable (string-mismatch checks → `expected`/`actual`/`diff_byte_offset`; decode-failure checks → `decode_error`). The `emit_verify_checks` helper centralizes the population logic so maintenance scales.
- **Q2 (Phase Q schema-2/3 retro-compat): SKIP.** v0.4.3 ships schema-4-only intake; schema-2/3 intake routes to existing FOLLOWUP `bundle-json-schema-2-3-retro-compat` at `v0.4.4+` for real-need-driven implementation.

## Phase ordering rationale

```
Phase N (binding merge) ─→ Phase R (wif-multisig) ─→ Phase P (verify-bundle helper + forensics + parity) ─→ Phase Q (--bundle-json + dispatch) ─→ Cleanup + Release
```

N is foundational (cleanup of dual binding types). R extends Phase K's wif resolution to multisig. P depends on N (single binding type makes the helper signature cleaner) but only loosely — P can land first if N proves harder. Q depends on P's helper (the JSON intake feeds into emit_verify_checks).

## Phase N — CosignerKeyInfo → ResolvedSlot merge

**Goal:** retire `CosignerKeyInfo` (legacy v0.3 binding shape); sole binding type is `ResolvedSlot`. Closes FOLLOWUP `cosigner-keyinfo-resolved-slot-merge`.

### N.1 — Promote ResolvedSlot to the canonical binding type

`crates/mnemonic-toolkit/src/synthesize.rs::ResolvedSlot` already has the superset of fields (xpub + fingerprint + path + path_raw + entropy). `CosignerKeyInfo` has 4 of 5 fields (no entropy). Merge: ResolvedSlot stays; CosignerKeyInfo deleted.

### N.2 — Rewrite parse_descriptor.rs::bind_descriptor_keys + DescriptorBinding

Currently:
```rust
pub struct DescriptorBinding {
    pub keys: Vec<ParsedKey>,
    pub fingerprints: Vec<ParsedFingerprint>,
    pub cosigners: Vec<CosignerKeyInfo>,
    pub entropy: Option<Vec<u8>>,
}
```

After v0.4.3 N:
```rust
pub struct DescriptorBinding {
    pub keys: Vec<ParsedKey>,
    pub fingerprints: Vec<ParsedFingerprint>,
    pub cosigners: Vec<ResolvedSlot>,  // entropy now per-slot, not bundle-level
}
```

The `entropy: Option<Vec<u8>>` bundle-level field is dropped; per-slot entropy lives in `ResolvedSlot.entropy`. Callers that need "is this binding secret-bearing at @0?" use `binding.cosigners[0].entropy.is_some()`. Callers that need the entropy bytes use `binding.cosigners[0].entropy.as_ref()`.

**Per r1 review nit (traceability):** `bind_descriptor_keys` MUST set `ResolvedSlot.entropy = None` for every cosigner slot at @1+ (v0.3 contract: only @0 carries phrase-derived entropy; @1+ slots come from `--cosigner` triples which are watch-only by definition). v0.4.3 unified-path `--slot @N.phrase=` invocations populate per-slot entropy independently via `bundle_args_to_slots → resolve_slots`; descriptor-mode legacy invocations always have @1+ as `entropy: None`. Add a comment block at `bind_descriptor_keys` site documenting this invariant.

### N.3 — Update verify_bundle.rs callers

`cmd/verify_bundle.rs::descriptor_mode_verify_run` (around line 1331-1349) constructs DescriptorBinding via `bind_descriptor_keys` and consumes `binding.cosigners: Vec<CosignerKeyInfo>` + `binding.entropy: Option<Vec<u8>>`. After N.2:
- `binding.cosigners` is now `Vec<ResolvedSlot>`.
- `binding.entropy` access becomes `binding.cosigners[0].entropy.clone()`.

### N.4 — Update synthesize_descriptor signature

`synthesize_descriptor(descriptor: &Descriptor, cosigners: &[CosignerKeyInfo], entropy: Option<&[u8]>, ...)` → `synthesize_descriptor(descriptor: &Descriptor, cosigners: &[ResolvedSlot], ...)`. The `entropy` parameter is now redundant (read from `cosigners[0].entropy`).

### N.5 — Update bundle.rs `bundle_run_unified_descriptor` bridging

The Phase L bridging code (cmd/bundle.rs:~1490+) currently builds CosignerKeyInfo intermediates. After N.2 it just builds ResolvedSlot directly and calls synthesize_descriptor.

### N.6 — Delete CosignerKeyInfo

`synthesize.rs::CosignerKeyInfo` removed. Tests that constructed CosignerKeyInfo update to ResolvedSlot.

**Phase N architect review:** end-of-phase only. ~80-line refactor across 3 files; mechanical.

## Phase R — wif slot resolution in multisig contexts

**Goal:** allow `--slot @N.wif=<wif>` in multisig contexts (any N≥1). Closes FOLLOWUP `wif-multisig-resolution`.

### R.1 — Remove the v0.4.2 single-sig-only guard

`cmd/bundle.rs::resolve_slots` currently rejects wif when `by_index_len > 1`. Remove the guard.

### R.2 — Wif slot in multisig synthesis

`synthesize_unified` already handles slot vecs of any length; the wif slot's depth-0 xpub-with-zero-chaincode flows through identically to other watch-only slots (entropy: None → ms1 sentinel "" → mk1 carries the wif's pubkey).

### R.3 — Tests

- `unified_slot_wif_in_multisig_2_of_3` — `@0.phrase + @1.wif + @2.xpub` produces a valid 2-of-3 watch-multisig bundle.
- `unified_slot_wif_alone_in_2_of_2` — pure wif multisig with TWO DISTINCT WIFs (degenerate but legal).
- **Per r1 review nit:** `unified_slot_same_wif_twice_emits_bip388_row13` — same WIF supplied for `@0.wif` AND `@1.wif` produces identical (xpub, path) tuples → SPEC §6.6 row 13 fires (BIP-388 distinct-key violation). Critical correctness regression test for Phase R; without it the wif-multisig path could silently allow self-multisig collisions.

**Phase R architect review:** end-of-phase only. Tiny scope; ~10 lines deleted + 2 tests added.

## Phase P — verify-bundle helper + full forensics + descriptor 9/3+6N parity

**Goal:** introduce `emit_verify_checks` helper; refactor 3 run_* entry points to use it; populate forensic fields at all ~78 push sites; descriptor-mode emits the same 9/3+6N schema as template-mode. Closes FOLLOWUPS `verify-bundle-emit-checks-helper-and-full-forensics-rollout` + `verify-bundle-9-3plus6n-descriptor-mode-parity`.

### P.0 — VerifyCheck struct shape correction (SPEC §5.7 drift fix)

**Per r1 review:** SPEC §5.7 specifies the JSON envelope as `{passed: bool, ...}` but the v0.4.1 implementation introduced `result: &'static str` ("ok" | "fail" | "skipped"). This is long-standing SPEC drift. Phase P.0 corrects it before the helper rollout to avoid baking the drift into all ~78 sites.

```rust
// Before (v0.4.1):
pub struct VerifyCheck {
    pub name: String,
    pub result: &'static str,      // "ok" | "fail" | "skipped"
    pub detail: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub diff_byte_offset: Option<usize>,
    pub decode_error: Option<String>,
}

// After (v0.4.3 P.0):
pub struct VerifyCheck {
    pub name: String,
    pub passed: bool,              // SPEC §5.7 conformant
    pub detail: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub diff_byte_offset: Option<usize>,
    pub decode_error: Option<String>,
}
```

**Skipped checks** are represented as `passed: true + decode_error: Some("skipped: <reason>")` per SPEC §5.7's hybrid-mode treatment ("ms1_decode[i] and ms1_entropy_match[i] checks pass-vacuously: set `passed: true`, all forensic fields null, decode_error = `\"skipped: watch-only slot\"`").

JSON envelope changes:
- `"result": "ok"` → `"passed": true`
- `"result": "fail"` → `"passed": false`
- `"result": "skipped"` → `"passed": true, "decode_error": "skipped: <reason>"`

Per "no users → no migration" license: existing JSON consumers don't exist; the breaking change is internal. Update Default impl + 4 unit tests for the new shape.

### P.1 — `emit_verify_checks` helper signature

```rust
pub fn emit_verify_checks(
    expected: &Bundle,
    supplied: &SuppliedCards,
    is_multisig: bool,
) -> Vec<VerifyCheck>
```

Per impl plan v0.4.2 §P.1 locked decision: signature uses `is_multisig: bool`, NOT `BundleMode`. Per-slot watch-only inferred from `expected.ms1[i].is_empty()` sentinels.

`SuppliedCards` is a NEW struct in `cmd/verify_bundle.rs` packaging `--ms1`/`--mk1`/`--md1` arg vectors:

```rust
pub struct SuppliedCards<'a> {
    pub ms1: &'a [String],   // already empty-or-non-empty per slot
    pub mk1: &'a [String],   // flat for single-sig; per-cosigner flattened for multisig
    pub md1: &'a [String],   // chunked
}
```

### P.2 — Helper internal logic — schema and forensics

Emits the SPEC §5.7 schema:
- N=1 (single-sig): 9 checks — `ms1_decode + ms1_entropy_match + mk1_decode + mk1_xpub_match + mk1_fingerprint_match + mk1_path_match + md1_decode + md1_wallet_policy + md1_xpub_match`.
- N≥2 (multisig): 3 + 6N checks — 3 shared (md1_decode + md1_wallet_policy + md1_xpub_match) + 6 per-cosigner (`ms1_decode[i]` + `ms1_entropy_match[i]` + `mk1_decode[i]` + `mk1_xpub_match[i]` + `mk1_fingerprint_match[i]` + `mk1_path_match[i]`).

Per-slot `expected.ms1[i].is_empty()` → ms1 checks for slot @i emit `passed: true` with `decode_error: Some("skipped: watch-only slot")` per the SPEC §5.7 + Phase P.0 representation. All other passing checks have `decode_error: None`.

Forensic field population:
- **String-mismatch** (e.g., `mk1_xpub_match` where decoded xpub differs from expected): populate `expected: Some(<expected_str>)`, `actual: Some(<actual_str>)`, `diff_byte_offset: Some(VerifyCheck::diff_offset(expected, actual))`.
- **Decode-failure** (e.g., `ms1_decode` where ms-codec returns Err): populate `decode_error: Some(<error_string>)`.
- **Length-mismatch** (e.g., `mk1_path_match[i]` where path strings differ in length): same as string-mismatch with diff_byte_offset = `min(len(exp), len(act))`.

### P.3 — Refactor `run_full` / `run_multisig` / `descriptor_mode_verify_run`

Each becomes a thin wrapper:
1. Decode supplied cards into a "what we got" view.
2. Re-derive expected Bundle from --phrase / --xpub / --cosigner / --slot inputs (existing logic).
3. Build SuppliedCards from --ms1/--mk1/--md1.
4. Call `emit_verify_checks(&expected, &supplied, is_multisig)`.
5. Print VerifyBundleJson { schema_version: "4", result: <ok|mismatch>, checks }.

The 3 run_* entry points share ~80% of logic post-refactor; estimated 800-1000 lines of verify_bundle.rs deleted.

### P.4 — Descriptor-mode 9/3+6N

`descriptor_mode_verify_run` currently emits a 3-element coarse ladder (ms1/mk1/md1 byte-equality only). After P.3 refactor, descriptor-mode emits the SAME 9/3+6N schema as template-mode (the helper doesn't know or care which path the expected Bundle came from).

### P.5 — Tests

- 3+6N count assertion for descriptor-mode multisig (`cli_descriptor_mode.rs`).
- Per-slot "skipped: watch-only slot" check in hybrid mode.
- Tampered-mk1 detection emits forensic fields per SPEC §5.7 (existing test enhanced).
- 9-check single-sig assertion already exists; verify it still passes.

**Phase P architect review:** mid-phase after P.2 (helper API + internal logic) + end-of-phase after P.5.

## Phase Q — `--bundle-json` CLI + schema-4 dispatch

**Goal:** `mnemonic verify-bundle --bundle-json <file>` reads a JSON-envelope bundle (output of `bundle --json`) + dispatches on `schema_version: "4"`. Schema 2/3 deferred to FOLLOWUP `bundle-json-schema-2-3-retro-compat` (v0.4.4+).

### Q.1 — SPEC §6.7 amendment + `--bundle-json <path>` clap flag

**Per r1 review C-1: SPEC §6.7 amended in lockstep** (already applied to `design/SPEC_mnemonic_toolkit_v0_4.md`). The amendment: adds `[--bundle-json <path>]` to the §6.7 grammar block with full flag semantics (mutually exclusive with `--ms1`/`--mk1`/`--md1` triplet; re-derivation flags `--slot`/`--phrase`/etc. STILL required to compute the `expected` side; schema-4-only in v0.4.3 with v0.4.4+ FOLLOWUP for schema-2/3 retro-compat).

`cmd/verify_bundle.rs::VerifyBundleArgs.bundle_json: Option<PathBuf>`. Mutually exclusive with the explicit `--ms1` / `--mk1` / `--md1` flag triplet via clap `conflicts_with`.

### Q.2 — `serde_json::Value` peek + schema-4 typed dispatch

```rust
let raw = std::fs::read_to_string(&path)?;
let v: serde_json::Value = serde_json::from_str(&raw)?;
let schema = v["schema_version"].as_str().unwrap_or("");
match schema {
    "4" => {
        let bundle: BundleJson = serde_json::from_str(&raw)?;
        // Convert BundleJson back to Bundle (extracting ms1, mk1, md1)
        // + re-derive expected from --phrase/--slot/etc + run emit_verify_checks.
    }
    other => Err(format!(
        "--bundle-json schema_version {other} not supported in v0.4.3; this toolkit emits and reads schema_version \"4\" only. Schema-2/3 retro-compat intake tracked at FOLLOWUP `bundle-json-schema-2-3-retro-compat`."
    )),
}
```

### Q.3 — Tests

**Per r1 review C-2 — round-trip semantics LOCKED:** `--bundle-json` supplies ONLY the card strings (extracted from JSON envelope). The user MUST also supply `--slot @N.<subkey>=<value>` (or legacy `--phrase` / `--xpub` / `--cosigner`) to drive the expected-bundle re-derivation. Without re-derivation flags verify-bundle has no `expected` to compare against the supplied cards. Q.3 tests must include both flag sets explicitly:

- `verify_bundle_via_bundle_json_schema_4_round_trip` — `bundle --json --slot @0.phrase=X --template bip84 --network mainnet > tmp.json; verify-bundle --bundle-json tmp.json --slot @0.phrase=X --template bip84 --network mainnet` → exit 0.
- `verify_bundle_via_bundle_json_unsupported_schema_rejected` — handcrafted schema-3 fixture → exit 4 + byte-exact stderr `error: --bundle-json schema_version 3 not supported in v0.4.3; this toolkit emits and reads schema_version "4" only. Schema-2/3 retro-compat intake tracked at FOLLOWUP \`bundle-json-schema-2-3-retro-compat\`.\n`.
- `verify_bundle_via_bundle_json_conflicts_with_ms1` — supplying both `--bundle-json` AND `--ms1` → clap rejects with conflict (exit 64 or whichever clap chooses).

**Phase Q architect review:** end-of-phase only.

## Cleanup + Release (post-Phase Q)

Final architect review across all phases (transcript-only). CHANGELOG v0.4.3 entry. Tag `mnemonic-toolkit-v0.4.3`. GitHub release.

`cargo publish` for the toolkit remains gated on ms-codec / mk-codec / md-codec landing on crates.io. v0.4.3 distributed via GitHub tag only.

### Opportunistic nits to pick up

- `unified-slot-xpub-missing-path-origin-path-null` — emit `null` instead of `""` for missing origin_path in `emit_unified` JSON. ~5-line change; do during Phase N or Q.
- The 2 trap-bypass nits — defer (still legitimately niche).
- v0.3-nice-to-have items — defer (orthogonal to v0.4.3 theme).

## Test impact summary

- Phase N: ~3 unit tests update (CosignerKeyInfo construction sites in tests/).
- Phase R: +2 wif-in-multisig integration tests.
- Phase P: ~800-1000 lines deleted in verify_bundle.rs; +5 forensic+parity tests.
- Phase Q: +2 --bundle-json tests.
- Cleanup: 1 nit test for null-origin-path.

Estimated post-v0.4.3: ~245-250 lib + integration tests; significant verify_bundle.rs simplification.

## Out of scope (deferred to v0.4.4+)

- `bundle-json-schema-2-3-retro-compat` (v0.4.4+; gated on real need).
- `bundle-removed-subcommand-trap-{positional-eq,double-dash}-bypass` (v0.4-nice-to-have nits; vanishingly unlikely user paths).
- v0.3-nice-to-have items: `multisiginfo-magic-strings-enumify`, `descriptor-string-normalization-policy`.
- v0.5: `legacy-cli-flag-deletion` (delete --phrase/--xpub/--cosigner CLI surface).
- v0.5+: `unified-slot-xprv-resolution-needs-ms-codec-extension`.
