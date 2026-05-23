# v0.34.2 — End-of-cycle architect review (opus)

**Date:** 2026-05-22
**Cycle:** mnemonic-toolkit v0.34.2 — `mnemonic nostr --import` (read-only Bitcoin Core importdescriptors) + `--timestamp` + hygiene closes
**Branch:** `v0.34.2-toolkit-hygiene`
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle gate
**Scope reviewed:** full impl diff `/tmp/v0_34_2_impl.diff` + live source (commits `6a8d3a2`, `12133c5`, `ab82bff`)

---

## Summary of verification

**Correctness of importdescriptors JSON** — `import_array_single` emits `active:false`, `internal:false`, no `range`, `timestamp` via `to_json()`. Verified against Bitcoin Core RPC semantics: `active` defaults false (valid for watch-only), `internal` defaults false (receive), `range` is only for ranged descriptors (omitting it on a single-key descriptor is correct — including it would error), `timestamp` is required and present. The descriptor carries a BIP-380 `#csum` because `descriptor_for` returns `desc.to_string()` (miniscript Display appends the checksum), confirmed by the existing `descriptor_has_checksum_and_round_trips` test.

**Secret hygiene** — The recipe is built from `row.descriptor` (always the public pubkey descriptor), never the WIF. On the secret path, only the pubkey descriptor enters the recipe. `flag_is_secret` correctly does NOT need `--import`/`--timestamp` (neither carries secret material). No new secret reaches stdout/JSON.

**Deferred-refusal** — `spending`/`both` are rejected by clap's `value_parser` during arg parsing, before `run()`, so nothing emits; exit code 2 (clap usage error). Clean.

**Lockstep** — Manual updated (flag table + example) and matches `--help`; flag-coverage lint will pass. The toolkit `gui-schema` auto-derives from clap so it emits `--import`/`--timestamp` automatically; the hand-maintained mirror is in the separate `mnemonic-gui` repo — that update is the required paired-PR follow-on.

**ToolkitError ordering** — No new variant added; reuses `BadInput` (serialize failures) + clap value_parser error. No ordering concern.

**Cargo.lock** — Bumped 0.34.0 → 0.34.2, now consistent with Cargo.toml. FOLLOWUP `cargo-lock-version-bump-lockstep` filed with full detail.

**install.sh / CHANGELOG** — self-pin `mnemonic-toolkit-v0.34.2` matches the tag; CHANGELOG has `[0.34.2]` entry.

---

## Critical

(none)

## Important

(none)

## Minor

**M1 — No integration test exercises `--import readonly` on the SECRET path (watch-only-recipe-from-nsec guard missing)**
`crates/mnemonic-toolkit/tests/cli_nostr.rs:155-210`
All 6 new `--import` tests use `--pubkey`. The feature's most security-load-bearing claim — that `--import readonly` on a `--secret`/`--secret-stdin`/`nsec` input emits only the *public* descriptor (no WIF, no scalar) into the recipe — has no regression guard. The code is correct by construction (`build_import_recipe` maps `row.descriptor`, which is always the public descriptor; the WIF lives only in `row.electrum`/the `wif` field, neither of which feeds the recipe), and the secret-path/json-secret tests above confirm the WIF still appears in its own field. So this is not a bug — but a future refactor that accidentally routed a spending descriptor into `import_array_single` would not be caught. Concrete fix: add one test, e.g. `nostr --secret <NSEC> --script-type p2wpkh --import readonly`, asserting the parsed import JSON `desc` starts with `wpkh(02` (a pubkey descriptor) and that the import JSON substring does NOT contain the WIF string. Confidence 80.

**M2 — New `pub(crate) use` re-export is misplaced in the otherwise-alphabetical-by-module re-export block**
`crates/mnemonic-toolkit/src/wallet_export/mod.rs:42`
The block at lines 31-48 is ordered alphabetically by module path (`bip388`, `bitcoin_core`, `bsms`, `coldcard`, `electrum`, `green`, `jade`, `pipeline`, `sparrow`, `specter`). The added `pub(crate) use bitcoin_core::import_array_single;` was inserted at line 42 between `coldcard::ColdcardEmitter` and `electrum::ElectrumEmitter`, breaking the convention. This is not the CLAUDE.md `ToolkitError`-variant rule (that is specifically about error variants + their match arms), but it violates the file's evident ordering and will be a recurring merge-conflict snag. Concrete fix: move the line up to sit with the other `bitcoin_core::` re-export at line 32 (i.e. group `bitcoin_core::BitcoinCoreEmitter` and `bitcoin_core::import_array_single` together, before `bsms`). Confidence 85.

---

VERDICT: GREEN (0C/0I)

The two Minor items are non-blocking polish (M1 is a defense-in-depth test gap on already-correct code; M2 is cosmetic ordering). The cycle is clear to tag/ship. Recommend folding M1 (cheap, raises confidence on the secret-hygiene claim) before the tag if time permits, and M2 opportunistically. The one outward obligation that must not be dropped: the paired `mnemonic-gui/src/schema/mnemonic.rs` `schema_mirror` update registering `nostr --import` and `nostr --timestamp` as `kind:"text"` flags — this is a separate repo and is the Task 4 follow-on; the obligation is real (per R0 M3) and a stale mirror will fire the drift gate on the next GUI pin bump.

---

## Fold disposition (controller)

Both Minors folded before tag (commit follows this report):
- **M1** — added secret-path regression test `import_readonly_from_nsec_emits_only_pubkey_descriptor` to `cli_nostr.rs`.
- **M2** — moved `import_array_single` re-export to group with `bitcoin_core::BitcoinCoreEmitter` in `wallet_export/mod.rs`.
