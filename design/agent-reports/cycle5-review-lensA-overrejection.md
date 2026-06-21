# cycle-5 review — LENS A: OVER-REJECTION / AVAILABILITY

**Cycle:** cycle-5 "S-NET" network-provenance invariant (fail-closed `assert_network_agrees`)
**HEAD:** `79028490` (4 commits: `f65b94a4` / `9b80cfea` / `515dd4f4` / `79028490`)
**Base:** `ac4eead0` (origin/master, toolkit 0.62.1)
**Lens:** OVER-REJECTION — find any LEGITIMATE input the new checks now wrongly REJECT (a false-positive that bricks a valid wallet import/convert/export/build).
**Date:** 2026-06-21
**Method:** read the full diff + every `assert_network_agrees` call site (12 production sites traced); built the `mnemonic` binary at HEAD and ran live CLI probes for each hunt vector; ran the full `cargo test -p mnemonic-toolkit` suite (3× — see Minor-1).

---

## Summary verdict

**No legitimate input is over-rejected by this diff.** Every new reject fires only on genuinely network-inconsistent (corrupt / hand-edited / wrong-asserted) input; every legitimate consistent input I could construct still passes. The no-op precondition (originless / no-coin-type) is correctly preserved because the import parsers already errored on originless input on `origin/master` (pre-existing, BEFORE the new check), and the one site that genuinely defaults a network (the WIF arm, L11) is a spec-ratified intentional reject whose prior behavior was a *wrong-network artifact*, with a documented escape (`--network testnet`).

One genuine defect surfaced — a **flaky full-suite failure of `m14_tpub_reemit_into_mainnet_prefix_rejects`** — but it is a build-race test-harness artifact (stale-binary exec), NOT an over-rejection and NOT a deterministic logic bug (the M14 reject is exit-2 deterministic across 10 isolated runs; the test passes in isolation). Recorded as Minor-1 for the panel; out of my lens, but it falsifies the implementer's "0 failed" claim and should be re-confirmed clean before ship.

---

## Critical (over-rejection of legitimate input)

**NONE.**

---

## Important

**NONE** (within the over-rejection lens).

---

## Minor

### Minor-1 — `m14_tpub_reemit_into_mainnet_prefix_rejects` flakes in the full-suite run (test-harness build race; NOT over-rejection, NOT a logic bug)

`tests/cli_snet_convert_export_network_mismatch.rs:49`. In one of my three full-suite runs the test failed with `left: Some(0), right: Some(2)` — the `mnemonic` process exited 0 instead of the expected 2, and its stderr carried the *success*-path `note: stdout is watch-only` line (i.e. the conversion ran to completion, the M14 assert never fired). Root cause: `assert_cmd::cargo_bin("mnemonic")` exec'd a **stale (pre-cycle-5) binary** that `cargo test` was concurrently rewriting — in the pre-S-NET binary the M14 assert does not exist, so a `tpub`→`zpub` conversion succeeds at exit 0. Evidence it is a flake, not a logic defect:
- Run the test in isolation → **7 passed / 0 failed** (binary already built, no concurrent rebuild).
- `convert --from xpub=<tpub> --to xpub --xpub-prefix zpub --network mainnet` exits **2 deterministically across 10 consecutive runs** against the stable HEAD binary.
- The debug trace at the call site shows the assert correctly computing `xpub.network=Test asserted=Main` → `NetworkMismatch` (exit 2).

This is out-of-lens (it is a *missed*-reject flake, the opposite of over-rejection), but it (a) falsifies the implementer's "3343 passed / 0 failed" claim as a stable property and (b) is a recurrence of the known `assert_cmd` + `cargo test`-rebuild race. **Recommendation:** re-run the full suite to a clean green before ship (my third run was launched to confirm); optionally pre-build the bin (`cargo build` then `cargo test --no-run` ordering, or `CARGO_TARGET_DIR` isolation) to kill the race. Does not change the over-rejection verdict.

### Minor-2 — bitcoin-core mixed mainnet/testnet blob with NO `--network` is accepted (under-reject, out-of-lens, noted for the panel)

`core-mixed-mainnet-testnet.json` (one mainnet desc entry + one testnet desc entry) imported with **no `--network`** flag exits 0 — the H9 cross-entry class-check lives inside `if let Some(override_net) = args.network` (`import_wallet.rs:1191`), so a heterogeneous multi-entry blob is only caught when `--network` is supplied. This is an *under*-rejection (the other reviewers' lens), and it is the *pre-existing* behavior preserved (not introduced or worsened by cycle-5) — so it cannot be an availability regression. Flagged only so the panel's under-reject lens has it. With `--network mainnet` the same blob correctly rejects exit 1 (`ImportWalletNetworkClassMismatch`).

---

## Verified-safe — the legitimate cases I confirmed STILL PASS (live binary at HEAD)

All exit codes captured directly (no shell-pipe `$status` artifact).

1. **Originless / no-coin-type (the #1 over-reject risk).** The import parsers' new check is gated behind `network_from_origins` / `coin_type_from_path`, which on `origin/master` ALREADY error on an originless or sub-2-component-origin descriptor (`descriptor.rs:180` `origins.is_empty()`, `:210` `comps.len() < 2`) — this error predates S-NET and fires BEFORE the new `assert_slots_network_agrees` call. So `import-wallet --format descriptor` on an originless `tpub` was already rejected pre-cycle-5 (not a regression). The genuinely-legit originless ACCEPT path is `export-wallet --descriptor` (regression-guarded by the pre-existing `export_wallet_originless_concrete_still_accepted` test): I confirmed `export-wallet --descriptor 'wpkh(tpub…/0/*)' --network testnet` → **exit 0**, and even with `--network mainnet` → exit 0 (the M13 check only fires in `run_from_import_json`, never the direct `--descriptor` path). No-op precondition holds.

2. **Signet / regtest / testnet equivalence (NetworkKind, not Network).** A `tpub` (NetworkKind::Test) on a coin-type-1 path imported with `--network testnet`, `--network signet`, AND `--network regtest` → **all exit 0** (Test == Test; `CliNetwork::{Signet,Regtest,Testnet}.coin_type()==1`). No site compares `Network` where it should compare `NetworkKind`; H9 maps `Bitcoin→0 else →1` and uses `override_net.coin_type()`, both 2-way-correct.

3. **WIF (L11), compressed AND uncompressed, both families.** mainnet WIF → `--to xpub` with `--network mainnet` AND with `--network` omitted (default mainnet) → **exit 0**, correct `xpub…`; uncompressed mainnet WIF (`5…`) → exit 0; uncompressed testnet WIF (`9…`) with `--network testnet` → exit 0 `tpub…`. `pk.network` extraction maps correctly. The testnet-WIF-without-`--network` case now exits 2 — this is the spec-ratified intentional L11 reject (decision #11/§5.2); prior behavior was a *wrong-network* mainnet `xpub…` sentinel, and the user gets their intended result with `--network testnet`. Not over-rejection of legit output.

4. **Consistent multisig.** `core-two-mainnet.json` (external + change descriptor entries sharing one mainnet xpub) → exit 0 with AND without `--network`. Consistent electrum single-sig (`electrum-standard-bip84-mainnet`, `…-bip49-mainnet`) → exit 0. H9's per-entry extension only rejects cross-entry coin-type heterogeneity; within a single `ParsedImport` all cosigners already must share a coin-type (pre-existing parser invariant), and no legitimate BIP-48 multisig mixes xpub-version families. No legit watch-only / multi-cosigner wallet is rejected.

5. **build-descriptor (L1) WARNs, never REJECTs.** `tpub` keys + `--network mainnet` (disagree) → **exit 0** + stderr `warning: …--network mainnet disagrees with descriptor keys (testnet)…`; `tpub` keys + no `--network` → exit 0, **inferred testnet** preview (`tb1q…`), no warning. The `emit_human` network path returns no `Err` for a network disagreement (it is an inline compare + stderr WARN). The all-`Single`/raw-pubkey path (`infer_descriptor_network_kind` → `None`) falls back to the historical Mainnet default with no spurious warning. No availability regression.

6. **export-from-import-json (M13).** Consistent testnet envelope (`network:"testnet"` + `tpub` keys) → exit 0 as `--format bitcoin-core` AND as `--format sparrow` (the prefix-re-emitting path) → 601 bytes emitted, no reject. The inconsistent `mainnet-label-testnet-keys` envelope correctly rejects exit 2 BEFORE any `apply_xpub_prefix`.

7. **Default-network behavior.** No import/export site invents an asserted network and then rejects: the import parsers derive the asserted side from the blob's own coin-type (so it can only "disagree" with the blob's own xpub when the blob is internally inconsistent); M13's asserted side is the envelope's declared network (consistent envelopes pass); M14's asserted side is `--network`, which §5.a (`convert.rs:923`) makes MANDATORY for any non-default prefix — so the `unwrap_or(Mainnet)` at `:1104` never actually defaults when the M14 check runs (the check is inside `if !prefix.is_default()`, line 1101). The only genuine default is the WIF arm (item 3), covered above.

**Helper correctness.** `assert_network_agrees` is a pure 2-way `NetworkKind` compare with no hidden coercion; `network_kind_name` is total over the 2-variant enum; `NetworkKind::from(Signet|Regtest|Testnet)==Test`, `from(Bitcoin)==Main` (unit-tested). The pipeline wrapper `assert_slots_network_agrees` iterates resolved slots only — an empty `cosigners` vec is a vacuous OK (no spurious reject).

**Full-suite sweep.** `cargo test -p mnemonic-toolkit` run 3× at HEAD. Run 3 is fully green — **3343 passed / 0 failed / 15 ignored across 191 binaries** (exactly matching the implementer's claimed count). The new S-NET reject/positive-control tests all pass; the isolated `cli_snet_*` and `cli_import_wallet_*` suites are green. The single observed failure (an earlier run) is the Minor-1 build-race flake (M14), non-deterministic and exec-of-stale-binary — confirmed transient by the clean 3343/0 run, not a logic regression and not an over-rejection.

---

## Verdict

**LENS-A OVER-REJECTION: 0C / 0I — GREEN.**

No legitimate input is wrongly rejected. The fail-closed checks are correctly scoped (no-op preserved for originless/no-asserted-network; NetworkKind-granular so signet/regtest/testnet all pass; WIF/build-desc/export defaults all sane). The one full-suite test failure is an out-of-lens build-race flake (Minor-1) that should be re-confirmed clean before ship but does not bear on availability.
