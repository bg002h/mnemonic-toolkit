# SPEC — `mnemonic convert` path-independence / route-convergence test set

- **Date:** 2026-05-27
- **Source SHA (citations verified against):** `9294723` (toolkit `master`)
- **Status:** approved design (brainstorm + opus architect matrix); **pending R0 architect review to 0C/0I before any test code** (per CLAUDE.md mandatory pre-impl gate).
- **Type:** test-only. Like the cross-start convergence cycle (v0.37.4/v0.37.5), a red cell is a *finding* (test artifact vs real product bug) to triage honestly, not designed around.

## Purpose
Add the highest-assurance, complete-but-non-redundant metamorphic test set for ONE property of `mnemonic convert`:

**PATH-INDEPENDENCE / ROUTE CONVERGENCE** — when multiple `convert` routes carry the same underlying key/secret from a source to the same target node, the output bytes MUST be byte-identical, regardless of route or source representation.

This is the same class that found F3/F4 in the `bundle` subcommand. It targets the `convert` graph's multi-route convergence points (a target reachable by a direct edge AND a composite multi-hop route), which is where a silently-ignored flag or a shifted path-origin hides.

## Key finding from the design pass — F-fp (by-design, NOT a bug)
`phrase→fingerprint` / `entropy→fingerprint` emit the **MASTER** fingerprint (`derived.master_fingerprint`, `convert.rs:1174-1178`; computed at depth-0 *before* account derivation, `derive_slot.rs:82-89`) — template/account-independent; Trezor-24 = `5436d724`.
`xprv→fingerprint` (`convert.rs:1283`) / `xpub→fingerprint` (`convert.rs:1303`) emit the **account node's own** fingerprint (`xpub.fingerprint()`) — Trezor-24 bip84 account = `2bd87e08`.
These are fingerprints of *different keys*, so `phrase→fingerprint != phrase→xpub→fingerprint`. This is intentional (documented `cli_convert_bip39_vectors.rs:23-26`). The matrix splits cell #1 accordingly — a naive "all fingerprints converge" assertion would have been false.

## Scope (decided — do not widen/narrow)
The 7 convergence points: fingerprint multi-route, xpub multi-route, cross-representation→fp, SLIP-0132 octet, electrum-phrase round-trip, bip38 composite-vs-explicit, address composite. Edge matrix: `convert.rs::is_supported_direct_edge` (~`:593-645`).

## Matrix — 6 test functions (complete, non-redundant)
(C1a, C2, C3, C4 [7-cell loop], C6, C7-p2wpkh, C7-p2tr. Was 8 pre-R0; C5-std/C5-segwit dropped per I2.)

| Cell | Assertion |
|---|---|
| **C1a** `master_fp_template_account_invariant` | `phrase→fp --template bip84` == `phrase→fp --template bip44 --account 5` == `entropy→fp --template bip86` == `5436d724` (master-fp is template/account-independent). |
| **C2** `phrase_xpub_vs_phrase_xprv_xpub` | `phrase→xpub --template bip84` == `phrase→xprv→xpub` == pinned bip84 acct xpub. Compound `xprv→xpub,fingerprint` folds in the account-node-fp leg (xpub line == C2 xpub; fp line == `2bd87e08`). |
| **C3** `four_encodings_same_master_fp` | `phrase→fp` == `entropy→fp` == `seedqr→fp` == `ms1→entropy then →fp`, all `--template bip84`, all == `5436d724`; also `ms1→entropy == ENT64`. (ms1 has no direct derivation target: `(Ms1,*)`→{Entropy,Phrase} only, `convert.rs:630-631`.) |
| **C4** `slip0132_variant_octet_round_trip` | Parametric loop, 7 cells split by network (R0 I1 fold — the round-trip neutral is network-dependent: mainnet→xpub, testnet→**tpub**, because `apply_xpub_prefix` swaps only the version bytes and `normalize_xpub_prefix` maps testnet variants → tpub, `slip0132.rs:80,89,106-114`). **Mainnet (seed = mainnet neutral xpub `TREZOR_24_BIP84_MAINNET_XPUB`):** `--xpub-prefix V --network mainnet` for V ∈ {ypub, Ypub, Zpub} → emitted prefix {ypub, Ypub, Zpub}; `xpub=Y →xpub` == seed xpub. **Testnet (seed = `TREZOR_12_BIP84_TESTNET_TPUB`, `cli_convert_slip0132.rs:22`):** `--xpub-prefix V --network testnet` for V ∈ {ypub→upub, Ypub→Upub, zpub→vpub, Zpub→Vpub} → emitted prefix {upub, Upub, vpub, Vpub}; `xpub=Y →xpub` == seed tpub. Each cell asserts both legs exit 0. Per-cell tuple: `(seed, --xpub-prefix value, --network, expected emitted prefix, expected round-trip neutral)`. (zpub,mainnet) dropped — covered by `cli_convert_slip0132.rs:388`. Flag values are case-sensitive `{xpub,ypub,Ypub,zpub,Zpub}` only; testnet strings rejected (`slip0132.rs:388`). |
| **C6** `phrase_bip38_composite_eq_explicit_wif` | `phrase→bip38 --path m/84'/0'/0'/0/0 --passphrase X --bip38-passphrase Y` == `phrase→wif (same path,X) then wif→bip38 --bip38-passphrase Y`. Byte-identical `6P…` ciphertext (bip38 non-EC deterministic, salt=address-hash; `cli_convert_bip38.rs:11-23`). |
| **C7-p2wpkh / C7-p2tr** `phrase_address_eq_phrase_xpub_address_*` | Uses **`TREZOR_12`** (the `BIP84/86_RECEIVE_0_ADDRESS` constants are the 12-word seed's reference addresses, `cli_convert_address.rs:16,28` — R0 M1). `phrase→address --path m/84'/0'/0'/0/0 --script-type p2wpkh` == `phrase→xpub --template bip84` (emits a **neutral** xpub; `xpub→address` normalizes first so equality holds — R0 M2) then `xpub→address --path m/0/0 --script-type p2wpkh` == `BIP84_RECEIVE_0_ADDRESS`. Repeat bip86/p2tr → `BIP86_RECEIVE_0_ADDRESS`. `phrase→address` applies `--path` from MASTER (`convert.rs:1249-1268`); `xpub→address` applies `--path` relative to the account xpub (`:1308-1320`) — constructed to hit the same leaf `m/84'/0'/0'/0/0`. |

**Dropped (already covered / out of scope):** standalone account-fp 3-route (`cli_convert_happy_paths.rs:394`); SLIP-0132 (zpub,mainnet) (`cli_convert_slip0132.rs:388`); **C5-std/C5-segwit electrum round-trip (R0 I2)** — entropy↔electrum-phrase has a single route (a plain round-trip, not path-independence) and both legs + the full loop are already pinned in `cli_convert_electrum.rs:48,60,76,88,121,142`. The phrase↔entropy↔ms1 bijections (`cli_convert_round_trips.rs`) are out of scope (different property).

## R0 history
R0 (`design/agent-reports/convert-convergence-R0-review.md`): RED 0C/2I/3M → folded (I1 C4 network-split; I2 drop C5; M1/M2/M3 clarity). F-fp + constants confirmed correct. R1 re-dispatch pending.

## Mechanism
Canonical fixtures, deterministic — **no proptest** (exact-byte route equality over a fixed key universe; route divergence reproduces on one canonical vector). Determinism caveat designed-around: bip38 non-EC determinism (C6, confirmed by exact-ciphertext vectors).

## Fixtures (reuse pinned repo constants; route-vs-route equality is F2-safe)
Trezor-24 (`abandon×23 art`, ENT64=`0×64`, master fp `5436d724`, bip84 acct xpub `xpub6Bner3L3tdQW3…HTMshg9`, acct-node fp `2bd87e08`), Trezor-12 (`abandon×11 about`), DIGITS_24 (96-digit SeedQR, `seedqr.rs:220`), MS1_24 (`ms10entrsqqq…cwugpdxtfme2w`), electrum `STANDARD_HEX`/`SEGWIT_HEX` + phrases (`cli_convert_electrum.rs`), `BIP84_RECEIVE_0_ADDRESS`/`BIP86_RECEIVE_0_ADDRESS` (`cli_convert_address.rs`). Reuse verbatim where already a pinned test constant; the multi-route equality holds even if an anchor were stale.

## Placement & self-containment
New `crates/mnemonic-toolkit/tests/cli_convert_convergence.rs`, `assert_cmd::Command::cargo_bin("mnemonic")`, a `convert_value(&[&str])` helper mirroring `cli_convert_round_trips.rs:22-32` (+ a `convert_lines` for compound `--to`). Drives ONLY the `mnemonic` binary — no sibling md/ms/mk binary, no libs, no network → default `cargo test`, **no `#[ignore]`**.

## Verification & ship
`cargo test -p mnemonic-toolkit` green (or a surfaced finding); `cargo clippy --all-targets -D warnings`. Test-only → ship mode (commit-to-master vs PATCH bump) confirmed with user at green. The `--locked` CI guard (shipped `9294723`) now protects any version bump.
