# R0 review — SPEC_convert_path_independence_tests.md (verbatim, persisted before fold)

Reviewer: feature-dev:code-reviewer (opus). Base `9294723`. Cycle: convert path-independence test set.

**Verdict: RED** — 0 Critical, 2 Important, 3 Minor. The F-fp finding and fingerprint semantics (the load-bearing Critical-risk claim) are **correct**. Two Important issues make specific cells un-writable as a clean byte-equality without a spec correction.

## Methodology
Verified against current source (read, not trusting cited line numbers): `src/cmd/convert.rs` (full), `src/derive_slot.rs`, `src/template.rs`, `src/slip0132.rs`, `src/electrum.rs`, `src/seedqr.rs`, and existing tests `cli_convert_{happy_paths,slip0132,slip0132_info,address,bip38,electrum,round_trips,bip39_vectors}.rs` + `cli_cross_start_convergence.rs`.

## Critical (0)
None. F-fp confirmed: `phrase/entropy→fingerprint` emits `derived.master_fingerprint` (`convert.rs:1174-1178`, depth-0 `master.fingerprint` before account derivation, `derive_slot.rs:84`); `xprv/xpub→fingerprint` emits `xpub.fingerprint()` (account node, `convert.rs:1283/1303`). `5436d724` = master (pinned `cli_convert_happy_paths.rs:304` etc.), `2bd87e08` = bip84 account-node (pinned `cli_convert_slip0132.rs:30`). C1a/C2/C3 routing confirmed: `(Ms1,*)`→{Entropy,Phrase} (`convert.rs:630-631`), `(Seedqr,Fingerprint)` supported (`:614`), `phrase→xprv` emits account_xpriv (`:1173`). C6 bip38 determinism sound (same WIF intermediate; `--bip38-passphrase` fallback on direct edge `:1355`, none on composite `:1228`).

## Important (2)
### I1 — C4 testnet cells: "== neutral xpub" is ill-defined when seeded from the mainnet xpub
C4's only fixture is the mainnet `TREZOR_24_BIP84_MAINNET_XPUB`, but it iterates 4 testnet cells. `apply_xpub_prefix(mainnet_xpub, Zpub, Testnet)` (`slip0132.rs:106-114`) swaps only the 4-byte version to vpub bytes, re-encoding the mainnet key body with a testnet prefix; round-tripping back via `normalize_xpub_prefix` goes to **tpub** (`slip0132.rs:80,89`), NOT the original mainnet xpub. Pinned by `input_normalizer_testnet_vpub_to_xpub_normalizes_to_tpub` (`cli_convert_slip0132.rs:122`). So "== neutral xpub" is wrong for the 4 testnet cells. **Fix:** seed testnet cells from a testnet-rooted neutral — `TREZOR_12_BIP84_TESTNET_TPUB` (`cli_convert_slip0132.rs:22`) — so the round-trip closes on the tpub; OR restate the invariant as "network-appropriate neutral (xpub mainnet / tpub testnet) + 78-byte key body preserved." Spec must name a per-cell starting fixture.

### I2 — C5-std / C5-segwit fully redundant
Both legs already exist: `encode_entropy_to_standard_phrase` (`cli_convert_electrum.rs:76`), `encode_entropy_to_segwit_phrase_via_flag` (`:88`), `decode_standard_phrase_to_entropy` (`:48`), `decode_segwit_phrase_to_entropy` (`:60`), and full loops `round_trip_standard_phrase_via_entropy` (`:121`) / `_segwit_` (`:142`). C5 adds no new convergence point (entropy↔electrum-phrase has ONE route — a plain round-trip, not path-independence). Per the spec's own non-redundancy mandate, **drop C5-std/C5-segwit** (8→6 functions). The entropy-first increment-search caveat (`electrum.rs:151-173`) is accurate; the issue is purely redundancy.

## Minor (3)
- **M1** — `BIP84_RECEIVE_0_ADDRESS`/`BIP86_RECEIVE_0_ADDRESS` (`cli_convert_address.rs:16,28`) are the **12-word** seed's reference addresses; C7 says only "phrase" while Fixtures lists Trezor-24 first. Bind C7 to `TREZOR_12` explicitly. Path semantics confirmed correct (`convert.rs:1249-1268` master-path; `:1308-1320` relative-to-account).
- **M2** — `phrase→xpub --template bip84` emits a neutral xpub (not zpub); `xpub→address` normalizes first (`convert.rs:1297`), so equality holds — note the intermediate is the neutral form.
- **M3** — C4 labels `(ypub,test)` are shorthand for `--xpub-prefix {ypub|Ypub|zpub|Zpub} --network testnet` (5 case-sensitive values; testnet strings rejected, `slip0132.rs:388`). Tie each testnet label to flag+network in the impl note.

## Confirmations (no action)
All edges supported (`convert.rs:593-646`); flag/value tokens correct (`--template bip84/bip44/bip86`, `--script-type p2wpkh/p2tr`, `--electrum-version standard/segwit`, `--account u32`, `--path` accepts absolute + relative, `--xpub-prefix {xpub,ypub,Ypub,zpub,Zpub}`); DIGITS_24 (`seedqr.rs:220`), MS1_24 (`cli_convert_round_trips.rs:18`), STANDARD_HEX/SEGWIT_HEX (`cli_convert_electrum.rs:19,23`) real; dropped cells genuinely covered (`cli_convert_happy_paths.rs:394`, `cli_convert_slip0132.rs:388`); self-containment holds (only `mnemonic` binary; no `#[ignore]`); no file-name collision (`cli_cross_start_convergence.rs` targets `bundle`).

## Required for GREEN
1. I1 — respecify C4 testnet cells (seed from `TREZOR_12_BIP84_TESTNET_TPUB` → round-trip to tpub) or restate invariant.
2. I2 — drop C5-std/C5-segwit; update count 8→6.
3. Recommended: M1 (bind C7 to Trezor-12), M2 (neutral-xpub note), M3 (C4 label clarity).
Re-dispatch the architect after folding (I1 touches C4 construction).
