# mnemonic-toolkit bug-hunt PR: final review (fable)

Branch `claude/project-thread-utm1aa` at `14e85b3`, base `d72a18b` (merge-base with origin/master).
Read-only. Built at HEAD (`abe8742`, docs-only on top of 14e85b3) with `CARGO_INCREMENTAL=0 -j 2`, no `--release`.
Probed with `target/debug/mnemonic`; seeds A = `abandon ... about` (fp 73c5da0a), B = `legal winner ... yellow` (fp b8688df1).

**Verdict: 0 Critical, 1 Important, 4 Minor, 2 Nit.**

Prior Minor/Nit items of r1, r2, r3 are not repeated here.

## What I checked and found sound

- `template::DefaultOriginFamily` (sh(wsh) 48'/1', wsh 48'/2', other sh and every tr BIP-87), used by bundle, verify-bundle and xpub-search from the same tree.
- Canonical fill (`bind_descriptor_mode_paths`): seed slots derive at the shape's canonical origin with the coin from `--network`; the origin is on the card. Round trips mint -> verify-bundle -> restore --from pass for `wpkh`, `sh(wsh(sortedmulti))`, bare `sh(sortedmulti)`; the wpkh first address is the standard BIP-84 one (`bc1qcr8te4...`).
- `--account != 0` on a canonical descriptor is refused (exit 2), so the canonical fill cannot silently ignore an account request; non-canonical descriptors honour `--account` in the default (`m/87'/0'/1'` seen).
- Refusals that hold: canonical `[fp]@N`, canonical `--slot @N.path=m`, bare `@N`, concrete descriptors with no origin, `import-wallet` of an origin-less descriptor, WIF in export-wallet.
- `check_defaulted_xpub_origin` is reached for template defaults, canonical fills and non-canonical defaults, in bundle and verify-bundle.
- Restore search: path list matches the ruling; exact 65-byte key match; one match proceeds, none or several refuse; without a seed nothing is guessed. `derive_accounts_xpub_only` returns results in path order, so the zip is safe.
- Emitters (Sparrow, Electrum, Coldcard, BIP-388) take the slot's own path.
- `bughunt_1125*` (31 tests) pass at HEAD.

## Important

**I1: `bundle --template` still mints a keyed card with an EMPTY origin when the operator types `--slot @N.path=m` beside an xpub.**
- Where: `crates/mnemonic-toolkit/src/cmd/bundle.rs:723-726` (`resolve_slots`, xpub arm: a typed path is taken as is, with no empty-path check). The descriptor-mode twin is refused at `bundle.rs:2782-2792`.
- Repro (both exit 0):
  - `bundle --network mainnet --template bip84 --slot @0.xpub=<A's master xpub> --slot @0.fingerprint=73c5da0a --slot @0.path=m --json`
  - `bundle --network mainnet --template wsh-sortedmulti --threshold 1 --slot @0.xpub=<A's master xpub> --slot @0.fingerprint=73c5da0a --slot @0.path=m --slot @1.xpub=<B at 87'/0'/0'> --slot @1.fingerprint=b8688df1` (engraving card: `@0: ... (73c5da0a @ (no path))`).
- Confirmed on the card: `restore --md1` on the bip84 result prints "cosigner @0 carries NO origin path on the card" and `wpkh([73c5da0a]xpub661My...)`; `md decode` says "the origin path of @0 is NOT on this card".
- Why it matters: the settled ruling is that an empty origin on a keyed card is never minted, and the PR says so ("never put a keyed md1 key at an empty origin"; "`@N.path=m` on a canonical shape is refused because it cannot be recorded"). Both template shapes are canonical, so the wire reads the empty origin as "canonical origin, elided": a reader that follows the md BIP shows `[fp/84'/0'/0']<master key>` (the toolkit#2 class), and the toolkit's own restore needs the seed to find `m`.
- Mitigation: needs an explicit `@N.path=m`; the card is self-consistent; pre-existing behaviour, not introduced here. But it is the one mint route the ruling's fix leaves open.
- Fix: refuse a typed empty path on an xpub slot at mint in template mode, with the descriptor-mode text. `resolve_slots` is shared with verify-bundle and export-wallet, so put the refusal in `bundle_run_unified` (or pass the bind mode) to keep verify read-only. Add a test row beside `tk1_explicit_master_path_on_canonical_shape_is_refused`.

## Minor

**m1: restore prints two contradictory warnings when the search finds the key at `m`.**
- Where: `crates/mnemonic-toolkit/src/cmd/restore.rs:4525` (the "unresolved" loop filters on `origin.is_empty()`, which is also true for a key found at the master node).
- Repro: `restore --md1 <the bip84 card from I1> --from phrase=-` with seed A. stderr has "Its key was FOUND ... at m ... MASTER key", then "Its key is shown as [fingerprint]xpub with no path; no path is guessed ... Rerun with --from <that cosigner's seed>". The cosigner line says "your seed (verified)".
- `tk2_master_key_card_is_found_at_m_with_master_warning` asserts only "at m." and "MASTER key", so it does not see the second warning.
- Fix: track resolved indices (for example a `found_at_master` set) and skip them in the loop; assert the absence in the test.

**m2: on an empty-origin card, a seed that matches nothing now exits 1, not 4, and `--allow-mismatch` no longer applies.**
- Where: `restore.rs:4392-4415` (`return Err(bad(...))` for no match and for several matches).
- Before: the same input fell to the `cosigner-seed` `RestoreMismatch` (exit 4, "MISMATCH", overridable). Probe: seed B on the I1 card gives exit 1 with and without `--allow-mismatch`.
- Impact: an exit-code and error-kind change for a wrong seed or passphrase on such a card; a caller keyed on exit 4 sees a generic error. Only empty-origin cards are affected, and nobody holds one.
- Fix: either report it as `RestoreMismatch` (reference `cosigner-seed`, same text) or state the exit code in the manual and the PR body. `tk2_restore_with_unrelated_seed_refuses` asserts only non-zero.

**m3: the canonical-origin notice advises an override that is refused for the slots it is printed for.**
- Where: `bundle.rs:3084` ("Set it explicitly with [fp/path]@N or --slot @N.path=m/...").
- Probe: `--descriptor 'wpkh(@0/<0;1>/*)' --slot @0.phrase=- --slot "@0.path=m/84'/0'/5'"` exits 2, "slot @0 has both secret-bearing input and watch-only input"; `--account 5` is refused too. Only the inline `[fp/path]@N` works for a seed slot on a canonical shape (checked: it mints at `84'/0'/5'`).
- Fix: for seed slots say only `[fp/path]@N` (keep `--slot @N.path=` for xpub slots).

**m4: PR body statements that are not accurate.**
- Tests: the body gives `tests/all.rs` 2936 passed / 19 ignored and 4533 in total. The binary at 14e85b3 lists 2958 tests (r3 ran 2939 passed / 19 ignored), so the counts predate the r2 fold while the text says "once, at the end".
- Reviews: round 3 (0 Critical, 0 Important, 3 Minor, 4 Nit; committed in abe8742) is not listed.
- "Decoding and restoring an existing card are unchanged": restore of a card with an empty origin changes by design (search, refusal, exit code per m2), and verify-bundle of a pre-ruling default card needs `--slot @N.path=` (r2 M4). Say "cards that carry their origins".
- X4 "Reading existing WIF cards is unchanged and pinned by a pre-change fixture": the fixture pins `inspect` of one mk1 only. verify-bundle of a WIF bundle is not pinned, and toolkit#8 adds a network refusal on that path.
- toolkit#1 row and commit 6e304aa title ("never put a keyed md1 key at an empty origin") are false for template mode until I1 is fixed.

## Nit

- **n1:** `BoundDescriptorPaths::defaulted` is documented as "given the BIP-48 default origin" (`bundle.rs`, the struct after `bind_descriptor_mode_paths`); it is BIP-48 or BIP-87 now.
- **n2 (outside the diff):** a concrete multisig descriptor with a fingerprint-only origin, `wsh(sortedmulti(1,[73c5da0a]xpub.../<0;1>/*,[b8688df1/48'/0'/0'/2']xpub.../<0;1>/*))`, exits 2 with "internal: synthetic key ... not found in key map" (`parse_descriptor.rs:611`). It fails closed, but it should give the I2-style refusal text.

## Not examined

- The `#[ignore]`d Bitcoin Core rows, the manual lint and the whole suite (per the brief).
- The wc-codec parity-budget arithmetic (r1 and r2 covered it; the decoder is unchanged).
- verify-bundle against a card actually minted by the pre-change binary (none was built).
