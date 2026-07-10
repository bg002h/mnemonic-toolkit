# Cycle H post-implementation whole-diff review — F3 network fail-open

**Reviewer:** fresh Fable (post-impl whole-diff = single-phase per-phase R0 + post-impl combined), read-only, adversarial. `feature/cycleH-network-fail-open` @ `6dbf270a` vs master `713484c3`, against SPEC r4 + plan (both R0-GREEN).
**Dispatched:** 2026-07-09 (Cycle H, post-impl whole-diff round 1). Persisted verbatim per CLAUDE.md.

## VERDICT: **GREEN (0 Critical / 0 Important)**

All suites run by the reviewer; RED-proof run against a rebuilt base binary; fixture cryptographically verified.

## 1. Guards vs SPEC §1 — all five correct and precondition-respecting
- **E1** `convert.rs:1524-1537` — `match args.network`: assert in `Some` arm only vs `xpub.network` (key's own bytes, G3); `None` arm = untouched inference. ✓
- **E2** `address_of_xpub.rs:214-227` — identical shape; covers mk1-decoded targets (mk1 cell passes). ✓
- **E3** `silent_payment.rs:139` — inside the `Xpriv::from_str` branch only, gating `return Ok(xpriv)`; ms1/phrase/entropy branches untouched (master minted AT `--network`). ✓
- **E4** `export_wallet.rs:679-695` — loop over `resolved_slots_ref` (`slot.xpub` + Minor-C `master_xpub`) before `EmitInputs`; inert for `--descriptor` (empty slots). Critical control-flow check: `--from-import-json` **short-circuits at :412-414 BEFORE the loop**, so envelope-network imports are never checked against the clap-default `args.network` — no over-rejection there (its own envelope-network guard at :816 stands). ✓
- **E5** `bsms.rs:113-137` — inside the `FourLine` arm only, after `parsed`, before `derive_first_address`; uses `xkey_network()` (verified `vendor/miniscript/src/descriptor/key.rs:1043-1049`: `Some` for XPub AND MultiXPub, `None` for Single — G4 honored); TwoLine untouched. ✓

## 2. No over-rejection — verified in-suite AND live against the built binary
E1c inference=0, signet+tpub=0, E3c phrase+both networks=0, E4d passthrough=0, E5c 2-line=0, E5d hex-Single=0, plus extra cells probed beyond the suite: `bip388` multipath tpub passthrough=0, `bitcoin-core` passthrough=0, E4 `--network signet`=0, E5 `--network regtest`=0. Restore's BSMS path mints keys AT `--network` so the E5 guard is a no-op there (restore suites green).

## 3. REJECT cells RED-proven — non-tautological
Built the pre-fix binary at `713484c3` and reproduced every mint at exit 0: E1a `bc1q…` from tpub; E3a mainnet `sp1…` from tprv; **E4a electrum `zpub6qRfxLnn…` minted on the DEFAULT network**; **E5a/E5e mainnet `bc1q…` line-4 from tpub, single-branch and `<0;1>/*` multipath**; E2a wrong-network scan ("no match", exit 0). Post-fix binary: all exit **2** with `network mismatch` + correct per-edge context. Every new test would fail pre-fix.

## 4. Fixture change is NOT a dodge — cryptographically verified
Base58check-decoded all three pairs in `tests/cli_export_wallet_bsms.rs`: versions `0488b21e`→`043587cf` and **bytes [4..78] identical** (same depth/parent-fp/child-number/chaincode/pubkey) for A, B, C — genuine offline version-byte swaps of the same key material, not different keys and not a `--network` flip. Test intent preserved (still testnet, still 2-of-3 wsh-multi, same 4-line + `tb1q` assertions).

## 5. Completeness — no 6th mint edge
Full `assert_network_agrees` call-site census = 5 new + all pre-adjudicated prior art. Independently probed `xprv→xpub` + `--xpub-prefix` re-encode — already covered by the cycle-5 guard at `convert.rs:1144` over ALL Xpub-typed outputs. Benign sites untouched by the diff.

## 6. Suites (run by the reviewer, exit codes captured)
- `cargo test -p mnemonic-toolkit`: **3693 passed / 0 failed / 18 ignored** (206 suites; exit 0). `cli_network_fail_open`: 22/22; `cli_export_wallet_bsms`: 9/9.
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: exit 0.
- `cargo fmt -p mnemonic-toolkit -- --check`: diffs ONLY in `src/mlock.rs` (g6 exemption; untouched this cycle).

## 7. No scope creep
Exactly 8 files; Cargo.toml still `0.82.0` (bump belongs to the release ritual); no error variant, no clap flag, no codec/GUI/schema/install.sh/gen.sh/mlock change; manual edit = description-column prose in 4 existing flag rows.

## Minor (informational, non-blocking — no fold required)
- **M-1:** The E5 guard also fail-closes a hand-crafted `--from-import-json` envelope whose descriptor keys contradict its declared network at `--format bsms` 4-line (slots were already guarded; descriptor keys weren't). Strictly fail-closed widening consistent with F3; toolkit-produced envelopes are consistent by construction. Worth one CHANGELOG clause at ship.
- **M-2:** In `bsms.rs`, if `for_each_key` doesn't short-circuit, `mismatch` holds the last mismatching key's network rather than the first — cosmetic only (both are genuine mismatches; message identical in kind).

**Release-readiness:** GREEN for **v0.83.0** — proceed to the release ritual.
