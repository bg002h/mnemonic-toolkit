# R1 re-review — SPEC_convert_path_independence_tests.md (verbatim, post-fold)

Reviewer: feature-dev:code-reviewer (opus). Base `9294723`. R0 was RED 0C/2I/3M (`convert-convergence-R0-review.md`); findings folded; this is the re-dispatch.

**VERDICT: GREEN — 0 Critical / 0 Important / 0 Minor residual.** All R0 findings folded correctly; verified against current source (`src/slip0132.rs`, `src/cmd/convert.rs`, `src/derive_slot.rs`, existing test files). No new drift. Mandatory pre-impl R0 gate satisfied (RED 0C/2I → fold → R1 GREEN 0C/0I). Cleared to implement.

## I1 — C4 network split: CORRECT
- `TREZOR_24_BIP84_MAINNET_XPUB` (`cli_convert_slip0132.rs:12`) + `TREZOR_12_BIP84_TESTNET_TPUB` (`:22`) are real pinned constants, each a valid neutral key for its network.
- Prefix mapping verified via `swap_target_for` (`slip0132.rs:141-155`) + `normalize_xpub_prefix` (`:78-97`): mainnet `{ypub,Ypub,Zpub}`→{ypub,Ypub,Zpub}; testnet `{ypub,Ypub,zpub,Zpub} --network testnet`→{upub`:147`,Upub`:149`,vpub`:151`,Vpub`:153`}. Byte-correct.
- Round-trip closure: testnet `output_xpub_prefix_testnet_zpub_emits_vpub` (`:273`) + `input_normalizer_testnet_vpub/upub_to_xpub_normalizes_to_tpub` (`:122,:142`) → closes on tpub; mainnet `input_normalizer_big_z/y_to_xpub_normalizes_to_neutral` (`:82,:102`) → closes on xpub. The R0 hazard (mainnet body through testnet swap → tpub≠original) is dissolved: testnet cells now seed from the tpub.

## I2 — C5 dropped: CORRECT
Matrix now 6 functions; no C5 cell remains. Dropped-section electrum citations `cli_convert_electrum.rs:48,60,76,88,121,142` all grep-verified exact. No lingering reference breaks consistency.

## Minors — applied
- M1: C7 bound to `TREZOR_12`; `BIP84/BIP86_RECEIVE_0_ADDRESS` (`cli_convert_address.rs:16,28`) confirmed 12-word seed vectors.
- M2: neutral-xpub note correct (`convert.rs:1172` neutral; `:1297` normalize-first).
- M3: C4 labels tied to flag+network; case-sensitive value set verified (`slip0132.rs:38-51`).

## No new drift (unchanged cells re-confirmed)
F-fp (`convert.rs:1174-1178`, `derive_slot.rs:84`; `:1283/:1303`), C1a/C2/C3 routing (`:1172-1173`, `:630-631`, `:614`), C7 convergence (`:1249-1268` master-path, `:1308-1320` relative), C6 edges (`:637,603,635`). No file collision (`cli_convert_convergence.rs` new; `cli_cross_start_convergence.rs` targets `bundle`).

## Internal consistency
Matrix count (6), C4 "7-cell loop" (3 mainnet + 4 testnet), Dropped, R0-history, Fixtures all agree. Benign coincidence noted: `cli_convert_slip0132.rs:388` (zpub-mainnet round-trip) vs `slip0132.rs:388` (testnet-string rejection) — two different files sharing line 388; both citations accurate.
