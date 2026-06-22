# WHOLE-DIFF REVIEW — cycle-13 Lane B (L8 + L9, `cmd/restore.rs`)

Reviewed-patch lane; this review is the gate. Worktree `wt-cycle13b`, off `origin/master = d55bf4c3` (v0.65.2). Commit `977c86fd`.

## VERDICT: GREEN (0 Critical / 0 Important)

### L8 — non-mainnet all-own coin-type (funds-restorability, FAIL-SAFE) — VERIFIED CORRECT
- **(a) Correct component.** Substitution targets `comps[1]` (BIP-44/48 coin-type, 2nd element). Verified vs pinned md-codec 0.39.0 `canonical_origin` (`canonical_origin.rs:45-79`): returns ONLY `m/44'|84'|86'/0'/0'` (3-comp) or `m/48'/0'/0'/{1,2}'` (4-comp) — coin at index 1 in EVERY shape. `comps.len() >= 2` guard (`restore.rs:1596`) always holds; purely defensive, never skips a needed substitution.
- **(b) Identity on mainnet.** `coin_type()` = 0 mainnet → 0'→0'. Positive control passes.
- **(c) `coin_type()` correct** (mainnet=0, testnet/signet/regtest=1, `network.rs:22-27`). Mirrors the emitter (`template.rs:72,99`; `parse.rs:94,98`) — coin at index 1 in both BIP-87 + BIP-48.
- **(d) md-codec untouched** (pinned crates.io 0.39.0). NO-BUMP correct.
- **(e) RED reproduces.** Reverting the L8 hunk → `testnet_all_own`/`signet_all_own` FAIL with `✗ NO MATCH`/exit-4; mainnet + L9 stay green. Goldens are independent rust-miniscript descriptors.
- **(f) No mainnet regression.**
- **Fail-safe HOLDS.** Substitution fires ONLY in the all-own / no-`--origin` / no-cosigner branch (`:1642-1646`). The `--origin` branch (`:1635`) uses the operator path verbatim; the cosigner-family branch (`:1637`) derives coin from the cosigner's actual origin (`own_origin_from_family`) — neither touched. A hypothetical "coin-0'-on-testnet" wallet either inherits coin from a cosigner mk1 or uses `--origin`. A wrong heuristic produces only NO-MATCH (wallet-id/address search must match before any address emit), NEVER a wrong own-key address.

### L9 — hardened-use-site / taproot-override early refusals — VERIFIED CORRECT
- **(a) Transcribed correctly, fire before reconstruction.** Guards at `restore.rs:1460`/`:1467` byte-identical (variant/mode/flag/message) to `run_multisig`'s `:2835`/`:2842`; after the I-1 own-account-max gate, before cosigner parse / origin build / search.
- **(b) Shared-core placement = established pattern.** The I-1 `--own-account-max` gate already lives in the shared core (`:1434`) for uniform restore + verify-bundle protection (callers `restore.rs:1379` + `verify_bundle.rs:874`). verify-bundle adding these refusals breaks nothing: a hardened/unrestorable-taproot template could never have produced a legitimate verify-bundle OK (renders unhardened → wrong address → NO-MATCH) — swaps NO-MATCH for a precise early refusal, same nonzero exit. No existing verify-bundle template test feeds such a descriptor (grep-confirmed).
- **(c) RED reproduces.** Removing the guards → `hardened_use_site_refused_early` FAILS (proceeds to search → opaque error instead of the precise refusal). `has_hardened_use_site` (pinned md-codec `to_miniscript.rs:89`) scans baseline + per-@N overrides.
- **(d) No over-rejection.** Non-hardened named-template positive control + all template/verify suites green.
- **(e) Omitting the `origin_path_overrides` refusal is safe.** The completion path builds origins FRESH (C1 invariant — never loads carried `path_decl`/overrides), so a carried override is already ignored by reconstruction → cannot cause a wrong derivation. No bug left.

### Scope / gates — CLEAN
Diff = `restore.rs` (+58/-1) + new `tests/cli_restore_md1_template_cycle13b.rs` (460 lines). File-disjoint from Lanes A + C. md-codec untouched. No version sites, no new `ToolkitError` variant (reuses `ModeViolation`/`BadInput`), no cargo-fmt churn. `cargo test -p mnemonic-toolkit` 3374 passed / 0 failed; `cargo clippy --workspace --all-targets -- -D warnings` clean.

## Disposition
GREEN. Lane B clears the gate; HELD for integration into toolkit v0.66.0 (with Lanes A + C).
