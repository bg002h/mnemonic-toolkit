# WHOLE-DIFF ADVERSARIAL REVIEW — cycle-10 md-codec cluster (M3 + L14/L15/L17 + L6)

Post-implementation mandatory review over the full diff (worktree `wt-cycle10`, off `descriptor-mnemonic origin/main = 1a4b322`).

## VERDICT: GREEN (0 Critical / 0 Important)

Correct, fully tested, safe to publish. One non-blocking Minor (folded post-review: the `compute_wallet_policy_id` INVARIANT doc block, `identity.rs:168-178`, updated to reflect L14's empty-origin canonical-fill).

### Axis 1 — M3 fail-closed (FUNDS-CRITICAL): VERIFIED SAFE
The widened coarse gate (`derive.rs:114-132`) is a pure *availability* widening — only "derive correctly" or "still reject", never a wrong address. Per-key authority `use_site_to_derivation_path` (`to_miniscript.rs:277-292`) re-checks `chain` against each key's OWN multipath and fail-closes with `ChainIndexOutOfRange` (`:282-285`) when that key lacks the alt. Empirically verified all three adversarial cases:
- (a) baseline `None` + override `<0;1>`, chain=1 → derives at the override's chain-1 (committed `derive_address_override_change_chain_derivable`).
- (b) mixed wallet (@0 `<0;1>`, @1 `None`-multipath), chain=1 → both derive, chain0≠chain1 (only @0 splits), the `None` key contributes its fixed path for both chains (correct — a fixed key has no receive/change split), no wrong subtree.
- (c) over-max chain=2 → still rejects with `alt_count:2` (committed `derive_address_override_chain_over_max_still_rejects`).
- `max_alts`: `.iter().flatten()` over `Option<Vec>`, `unwrap_or(1)`, `fold(baseline, max)` — no index/subtraction/overflow.

### Axis 2 — 9 snapshot changes (snapshot-masking risk): VERIFIED, NO MASKING
All 10 MANIFEST vectors use the elided-origin form; `wsh_multi_chunked` is `force_chunked`→skipped → exactly 9 testable, all 9 changed. Per-snapshot: (a) ONLY `wallet_policy_id` changed (every hunk strictly inside that block; `md1_encoding_id` + `wallet_descriptor_template_id` untouched in all 9), (b) each is a legitimate canonical-fill (`path_decl.data: "m"` elided), (c) NO `wallet_descriptor_template_id` changed. Empirically diffed base vs branch: every elided vector → POLICY-CHANGED + WDT-SAME. Explicit-origin invariant held: L14 fires only when `e.origin_path.components.is_empty()` (`identity.rs:208`); explicit origins take the unchanged `else` branch. The insta gate passes against committed snapshots (no regen-masking).

### Axis 3 — In-memory-only SemVer: VERIFIED
`encode.rs` references NO id function / `canonical_origin`. Ids are display-only (md-cli, gated behind `--policy-id-fingerprint`); the wire string comes from independent `encode_md1_string`. SemVer-MINOR correct — no engraved/persisted md1 card changes.

### Axis 4 — L6 guard + L17 test: VERIFIED
`DivergentPathCountMismatch` pre-exists (`error.rs:66`, `n:u8, got:usize`) — REUSED, no new variant. `n_keys = d.n` copied before the mutable `paths` borrow (sound). Identity fast-path returns `Ok(())` (`:199-201`) before the guard — canonical inputs not over-rejected (`canonicalize_identity_short_divergent_not_reached`). L17 test is a true RED→GREEN gate (confirmed by temporarily reverting L14 → test FAILS).

### Axis 5 — Scope / regressions: VERIFIED
Diff = 3 md-codec source files + md-cli pin/9 snapshots + version sites + CHANGELOG. No clap/CLI/manual/schema surface. No public signature changes. `cargo fmt --all --check` clean; clippy clean; full md-codec + md-cli suites green. Version sweep: md-codec 0.39.0, md-cli 0.9.1 + pin `=0.39.0`, fuzz/Cargo.lock corrected from stale 0.35.1.

### Minor (folded post-review)
`identity.rs:168-178` INVARIANT doc block claimed the fn does NOT consult `canonical_origin` at hash time — L14 now does for the in-memory empty-origin case. Updated to document the L14 behavior (decoded wires unaffected). Doc-only; commit `8c73b4d`.

## Disposition
GREEN. Cleared for the publish→tag→push chain: md-codec `md-codec-v0.39.0` first, then md-cli `descriptor-mnemonic-md-cli-v0.9.1`. Toolkit pin-bump (md-codec "0.38"→"0.39") deferred to after cycle-11b ships toolkit 0.65.1 → lands as 0.65.2.
