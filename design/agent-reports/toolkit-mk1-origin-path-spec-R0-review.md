# R0 Review — SPEC_toolkit_mk1_origin_path.md

Opus architect (feature-dev:code-reviewer), mandatory pre-impl R0. Branch
`toolkit-mk-codec-0.4.0-repin`, base `master` `a255060` + applied re-pin. Verified vs
live toolkit + mk-codec 0.4.0 source. Persisted by controller.

## Headline confirmations
- **(a) "intermediates non-load-bearing" is HALF TRUE.** `reconstruct_xpub` (mk-codec-0.4.0
  `xpub_compact.rs:86-108`) uses only `len→depth` + `last→child`; network/parent_fp/chain_code/
  pubkey come from compact-73 bytes → the reconstructed **Xpub is byte-identical regardless of
  intermediates** (SPEC §3.1 right FOR THE XPUB). **BUT** `encode_bytecode` writes the FULL
  origin_path to the wire (`encode.rs:64`→`encode_path`); `decode_path` restores it verbatim →
  fabricated intermediates round-trip into `KeyCard.origin_path` and surface at every consumer of
  that field (C1/I2). SPEC's unqualified "loses nothing" is wrong for origin_path consumers.
- **(b) bitcoin 0.32 APIs confirmed** (`master()`=empty, `From<Vec<ChildNumber>>`, `depth:u8`,
  `child_number:ChildNumber`). Helper compiles except `ChildNumber` not imported (I3).
- **(c) 8 KeyCard::new sites confirmed** at the cited lines. Two more at `wallet_import/json_envelope.rs:681,712`
  are `#[cfg(test)]` + decode-direction (`mk1_card_to_resolved_slot`) → out of scope (M3).

## CRITICAL
- **C1 — verify-bundle stderr cross-checks false-positive on every correct mk1(depth-N)↔md1(deeper)
  bundle; §3.5 prescribes no concrete fix.** `emit_watch_only_xpub_path_cross_check` (`verify_bundle.rs:2024`,
  fires `:2117-2126` depth + `:2129-2146` child) and `emit_full_path_parent_fingerprint_check`
  (`:2239`, derives parent at `md_path[..N-1]` `:2374-2402`) compare the SUPPLIED mk1's decoded
  xpub.depth/child/parent_fp against md1 INDEPENDENTLY of the synthesized expectation. After the
  fix a correct 3→4 bundle has card.xpub.depth=3 vs md_depth=4 → spurious "internally inconsistent"
  warnings. The schema'd result verdict (`mk1_xpub_match` :1283 / `mk1_path_match` :1335, both sides
  use the same helper → symmetric) is unaffected; only the stderr cross-checks regress. SPEC must
  specify the actual comparison change (compare xpub.depth/child against md1 origin TRUNCATED to the
  xpub's length / treat mk1 path as a permitted prefix-or-extension), not "adjust if it false-positives."
- **C2 — §3.5 tampered-fixture rebuild recipe is impossible on 0.4.0.** Both fixtures
  (`tests/cli_verify_bundle_watch_only.rs:264-270,:355-361`) call `mk_codec::encode(&tampered).expect(...)`
  with inconsistent depth/child → 0.4.0 guard panics at construction. SPEC's "mutate the xpub to match
  the shortened path" makes the card CONSISTENT → the cross-check won't fire (self-contradictory).
  No public encoder bypasses the guard. Must rewrite to the two-internally-consistent-cards-that-disagree
  pattern already used by `cross_check_mk1_child_number_ne_md1_last_warns` (`:290-324`), and confirm a
  genuine depth-2-vs-depth-3 pairing is constructible.

## IMPORTANT
- **I1 — snapshot-regen masking risk for 4→3/4→4; "semantic round-trip" mitigation does NOT catch a
  wrong xpub** (only proves the card round-trips to the SAME xpub handed it, not the CORRECT one).
  Must manually audit the 4→3 (43) + 4→4 (1) buckets — confirm each depth-4 xpub is INTENDED (so
  extend-the-path is right) vs itself a derivation bug — BEFORE blessing regen. The taproot fixture
  read (`tests/cli_import_wallet_sparrow_taproot.rs:77-79`) is actually a consistent 3→3 BIP-87 case →
  SPEC's "4→3 = taproot multisig import" attribution is imprecise; re-ground.
- **I2 — the 3→0 fabricated path IS user-visible** (`inspect --mk1` prints `origin_path: m/{}` from
  decoded `card.origin_path`, `cmd/inspect.rs:222,:307`) → 14 cases show bogus `m/0'/0'/0'`. Decide NOW:
  accept helper-pad + pin a test, OR default `bind_watch_only_singlesig:1062` to a depth-consistent
  template path (cleaner but ripples into md1 path_decl `synthesize.rs:710-717` → descriptor/policy-id).
- **I3 — helper won't compile: `ChildNumber` not in scope** (`synthesize.rs:12` imports only
  `DerivationPath, Fingerprint, Xpriv, Xpub`). Add `ChildNumber`.

## MINOR
- **M1 — §3.4 framing overstates the check's role.** `synthesize_multisig_watch_only` + `_full` +
  `synthesize_watch_only` + `synthesize_full` are `#[allow(dead_code)]` TEST-ONLY helpers. LIVE
  production emit = `synthesize_descriptor` (`bundle.rs:1414,1649`; `import_wallet.rs:1383`) +
  `synthesize_unified`. Removing the `:494-503` loop is harmless but does NOT resolve production.
- **M2 — re-ground the 74-test/157-instance census** on the branch; cite actual failing test names
  per bucket (the table was not reproducible from the one taproot fixture read).
- **M3 — note the 2 decode-direction json_envelope.rs:681,712 sites as out-of-scope** explicitly.

## Test-coverage assessment
Helper unit tests (§5.2) sound — invariant `len==depth && last==child` hand-verified GREEN for all 6
classes. MISSING: an integration test pinning post-fix verify-bundle stderr for a real 3→4 bundle
(assert no spurious cross-check warning); a test pinning `inspect --mk1 origin_path` for the 3→0 case.
WIF round-trip regression (§3.6) well-targeted + confirmed consistent on 0.4.0.

## VERDICT: RED (2C / 3I / 3M)
Blocking: C1 (cross-check false-positives, under-specified), C2 (impossible fixture recipe). Fold I1
(audit 4→3/4→4 before regen), I2 (decide 3→0 now), I3 (import). The load-bearing premise splits:
non-load-bearing for the reconstructed xpub (confirmed) but preserved on-wire + surfaced in
origin_path consumers (refuted) — stop asserting "loses nothing" unqualified. Re-dispatch after fold.
