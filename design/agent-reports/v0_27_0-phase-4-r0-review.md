# v0.27.0 Phase 4 R0 Architect Review — `import-wallet --json` envelope rewrite

**Date:** 2026-05-18
**Reviewer:** feature-dev:code-reviewer (model: opus)
**Phase:** v0.27.0 Phase 4 — closes FOLLOWUP `wallet-import-json-envelope-full-bundle`
**Verdict:** GREEN — 0 Critical / 0 Important (≥ 80 confidence) / 2 Minor (informational; ≤ 50 confidence)

## Plan-doc §3.2.1 13-field BundleJson conformance (verified)

Cross-referenced `cmd/import_wallet.rs:496-511` (Phase 4 construct site) against `cmd/bundle.rs:692-707` (reference site) and `format.rs:120-145` (struct definition). All 13 fields wired correctly:

| Field | Phase 4 value | §3.2.1 contract | Pass |
|---|---|---|---|
| `schema_version` | `"4"` | literal `"4"` (synthesize.rs:1494 pin) | yes |
| `mode` | `cosigners.iter().any(entropy.is_some())` | watch-only / full | yes (semantically equivalent to `bundle.any_secret_bearing()` because `synthesize_descriptor` produces empty-string `ms1[i]` iff `cosigners[i].entropy.is_none()`) |
| `network` | `network_human_name(p.network)` | `&'static str` via promoted helper | yes |
| `template` | `None` | always `None` for descriptor-mode wallet-import | yes |
| `descriptor` | `Some(p.original_descriptor.clone())` | pre-strip raw incl `#<csum>` | yes (verified at both parse sites: `bsms.rs:254` uses `descriptor_body.to_string()` = `lines[1]` or `lines[2]`; `bitcoin_core.rs:300` uses `desc_with_csum.to_string()`) |
| `account` | `0` hardcoded | `0` hardcoded (§3.2.1 row `account` reconciled lock) | yes |
| `origin_path` / `origin_paths` | mutex via shared-path detection | mutex per SPEC §5.3 | yes |
| `master_fingerprint` | `Some(...)` for N=1; `None` for N>1 | mirrors bundle.rs:677-678 | yes |
| `ms1` / `mk1` / `md1` | direct pass-through from `synthesize_descriptor` | direct pass-through | yes |
| `multisig` | `Some(MultisigInfo{...})` for N>1; `None` for N=1 | yes | yes |
| `privacy_preserving` | `false` | always `false` for wallet-import path | yes |

## Phase 4 R0 explicit scope (verify-bundle round-trip)

Cell 7 (`bsms_envelope_verify_bundle_round_trip_via_bundle_json`) implements the round-trip via template-mode `verify-bundle --bundle-json <FILE>` + explicit `--slot @i.xpub/fingerprint` args. The pivot from plan-doc's `jq | mnemonic verify-bundle --bundle-json -` to tempfile + template-mode is **sound**: `verify_bundle.rs:967` reads via `std::fs::read_to_string(path)` only (no stdin support), so tempfile is the only viable path. Using real BIP-32-derived xpubs at `m/87'/0'/0'` (per `skip_middle_3of3_blob` precedent) sidesteps the mk-codec depth/child reconstruction footgun that hand-rolled fixtures hit. Asserting `result == "ok"` correctly gates synthesis lossiness.

## Concerns evaluated below the report threshold

**M1 — `multisig.template` hardcoded `"descriptor"` instead of mapped from `script_type_from_descriptor` (confidence 35).** §3.2.1 row `multisig` suggests `template = &'static str` "mapped from `script_type_from_descriptor`'s `WalletScriptType` return via that enum's as_str() / Display impl". Phase 4 uses literal `"descriptor"` (matching bundle.rs:663 fallback `template.unwrap_or("descriptor")` for descriptor-mode). No current consumer reads `multisig.template` (grep `multisig\.template` empty); Phase 5 derives script_type from the descriptor directly per §3.7.1. Cosmetic deviation; not load-bearing.

**M2 — `path_family_from_paths` silent bip87 fallback for non-48'/non-87' purposes (confidence 25).** Helper at `cmd/import_wallet.rs:630-642` defaults to `"bip87"` for any unrecognized purpose component (e.g., BIP-45 `45'`, BIP-44 `44'` in a hypothetical multisig context). Matches `MultisigPathFamily::default()` semantics at `parse.rs:67`. Acceptable per project's permissive-input/expressive-output philosophy.

## User-raised review points (all answered above the threshold)

1. **§3.2.1 13-field enumeration** — all 13 fields conform. PASS.
2. **verify-bundle round-trip Cell 7 pivot** — sound (verify-bundle has no stdin for `--bundle-json`; tempfile is the correct workaround). Real-derivation BIP-87 xpubs are the right choice for path-mode re-derivation match.
3. **original_descriptor byte-for-byte preservation** — verified for both parse paths. BSMS captures `lines[1]` (2-line) or `lines[2]` (6-line) BEFORE `verify_checksum` strip; both lines include `#<csum>`. Bitcoin Core uses `desc_with_csum.to_string()` (raw JSON `desc` field). PASS.
4. **CHANGELOG `### Changed` for wire-shape replacement** — Phase 6 scope per plan-doc §4.6. Not a Phase 4 concern, but doc-comment at `import_wallet.rs:388-394` correctly characterizes the change as wholesale replacement.
5. **path_family detection** — see M2 above. Acceptable.
6. **BSMS roundtrip stays `blocked_no_emitter`** — §4.4 scope does NOT mandate rewiring; plan-doc §3.2 example shows `status:"ok"` for BSMS but that's the *target* for a future cycle. Phase 4 explicitly carries the doc-comment "BSMS round-trip caveat: ...Status stays `blocked_no_emitter` until a follow-up cycle rewires". **Acceptable scope cut** — neither Critical nor Important.
7. **`multisig.template = "descriptor"` literal** — matches bundle.rs:663 precedent; see M1. Not a defect.
8. **Updated seed_overlay + sniff tests semantic equivalence** — sample cell `seed_overlay_multi_cosigner_skip_middle` correctly probes `bundle.ms1` (length-N sentinel array) + `bundle.multisig.cosigners`. Semantically equivalent to v0.26.0's `bundle.cosigners[i].has_entropy` + `.fingerprint` probes; new probes are STRONGER (one-expression assertion of `[true, false, true]` pattern vs three separate per-cosigner asserts).

## FOLLOWUPS.md Status flip

`design/FOLLOWUPS.md` — `wallet-import-json-envelope-full-bundle` flipped to `**Status:** resolved` IN THIS SAME COMMIT per the recurring memory pattern `[[feedback-per-phase-agents-forget-followup-status-flip]]`.

## Verdict

**GREEN.** 0 Critical / 0 Important (≥ 80 confidence). 2 Minor (M1/M2) below report threshold. No Critical to fold pre-commit.
