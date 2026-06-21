# cycle-5 review — LENS B (funds-correctness of the checks)

- **Cycle:** cycle-5 (S-NET network-provenance invariant)
- **HEAD:** `79028490`
- **Base:** `ac4eead0`
- **Date:** 2026-06-21
- **Lens:** Funds-correctness of the checks themselves — does each new check actually CATCH the
  wrong-network case it claims (no false-NEGATIVE), and are the error variants / exit codes correct?
- **Worktree:** `/scratch/code/shibboleth/wt-cycle5`, branch `feature/cycle5-snet-network-invariant` (4 commits).

## Method

Read all touched files (helper, error.rs, 7 import parsers + pipeline, convert/export/build-descriptor,
the 5 new test files + fixtures). Verified the bitcoin-0.32.8 `Xpub::decode` network semantics
(`bip32.rs:795-822`: `0x0488B21E`→Main, `0x043587CF`→Test) and `slip0132::normalize_xpub_prefix`
(all 4 mainnet SLIP-132 variants → `xpub`/Main, all 4 testnet → `tpub`/Test; neutral pass-through).
Built + ran the 5 cycle-5 integration test files (38 tests, all green) and the lib suite (158/0).
Ran a MUTATION TEST: disabled `assert_slots_network_agrees` in `descriptor.rs` → both descriptor reject
tests went RED (wrong-network blob exited 0 instead of 2), proving the check is load-bearing and the
comparison is NOT tautological. Restored; re-confirmed clean build.

---

## Critical

None.

## Important

None.

## Minor

### M-1 (informational, NOT a defect) — L1 build-descriptor "first network-bearing key wins"
`infer_descriptor_network_kind` (`cmd/build_descriptor.rs:477-498`) takes the FIRST network-bearing
xpub and ignores a hypothetical intra-descriptor disagreement (`inferred.is_none()` guard at :492). For
the L1 *display-preview* WARN this is correct and in-scope-bounded by the comment: a mixed-family
descriptor is a pre-existing malformed input, and the axis-2 import-side checks already reject a
coin-type/xpub-version mismatch per-cosigner before any descriptor reaches build-descriptor. The L1 WARN
correctly fires on a genuine `--network` vs first-key mismatch and is advisory-only (exit 0, still
renders) — matching the spec intent that the deliverable descriptor/bip388 are network-agnostic. No fix
required; noting only that L1 is a single-key heuristic by design.

### M-2 (informational) — axis-2 fires inside `parse()`, axis-1 `--network` after
Each parser calls `assert_slots_network_agrees(&cosigners, network, …)` against the **coin-type-derived**
network INSIDE `parse()` (validates the blob's internal xpub-vs-coin-type consistency); the H9 `--network`
override class-check + rebind runs AFTER parse in `cmd/import_wallet.rs:1191-1219`. This ordering is
correct: axis-2 validates blob self-consistency (exit 2), axis-1 validates user-override class (exit 1);
they are independent and never conflated. Noting the ordering only because a future reader might expect
the `--network` value to feed axis-2 — it deliberately does not (axis-2 uses the blob's own coin-type).

---

## Hunt findings (per the 7 directed questions)

1. **2-way NetworkKind check — no wrong-network slips through.** `assert_network_agrees(decoded,
   asserted, ctx)` (`network.rs:94-107`) compares the two `NetworkKind`s directly. The helper
   `assert_slots_network_agrees` (`pipeline.rs:137-147`) passes `slot.xpub.network` as `decoded` — the
   **xpub's OWN decoded version-byte network**, recovered via `finalize_slot_fields` →
   `normalize_xpub_prefix` → `Xpub::from_str` (the neutralized form still carries the genuine Main/Test
   version bytes, verified faithful: zpub/ypub/Ypub/Zpub → xpub/Main; vpub/upub/Upub/Vpub → tpub/Test).
   `asserted` is the coin-type-DERIVED `bitcoin::Network` from `network_from_*`. These come from
   INDEPENDENT data (xpub version bytes vs the path's coin-type child), so the compare is NOT
   tautological. All 7 parsers pass the xpub's own network: bitcoin_core (`:311`), bsms (`:272`),
   coldcard_multisig (`:493`), descriptor (`:105`), electrum (`:412`), sparrow (`:452`), specter (`:267`).
   MUTATION-TEST-CONFIRMED: disabling the call makes `tpub-on-0` and `xpub-on-1` exit 0 (RED).

2. **Two-axis separation — exit codes correct, never swapped.** H9 →
   `ImportWalletNetworkClassMismatch` exit **1** (`error.rs:581`); helper →
   `NetworkMismatch` exit **2** (`error.rs:592`). H9's per-entry fix (`import_wallet.rs:1200-1212`)
   reads each entry's OWN `p.network` in `parsed.iter()` BEFORE the `iter_mut()` rebind at `:1216` —
   exactly closing the old `first()`-only false-negative (the `mixed_mainnet_testnet_blob_override_
   mainnet_refused_per_entry` test is the load-bearing guard: entry0=mainnet passes the old check,
   entry1=testnet is now caught). No site conflates the two axes.

3. **Rename ripple complete.** Fields renamed `xpub_network→decoded_network`, `expected→
   expected_network`, `+context`. ALL readers updated: enum decl (`error.rs:278`), exit_code (`:592`),
   kind (`:661`), Display (`:835-843` — renders `key encodes {decoded_network} but {expected_network}
   was asserted`), details_json (`:920-928` — emits all three new keys), exit_code unit test
   (`:1022`), network.rs unit tests (`:148`, `:167`). Crate-wide grep for stale `xpub_network` finds
   only a TEST FUNCTION NAME (`l1_mainnet_xpub_network_omitted...`), no stale field reference. Display
   + detail_json output are correct, not merely compiling.

4. **WIF (L11) — correct.** `convert.rs:1490-1503` extracts `pk.network` (the WIF's OWN base58
   version-byte `NetworkKind`) via `PrivateKey::from_wif`, asserts it vs `network.network_kind()`
   (`--network` family) BEFORE building the sentinel xpub from `--network`. A mainnet WIF asserted as
   testnet (or vice versa) is caught (test `l11_testnet_wif_to_xpub_mainnet_rejects`, exit 2). WIF's
   2-way granularity matches the helper exactly. Without this, a testnet WIF would have minted a mainnet
   sentinel xpub.

5. **build-descriptor WARN (L1) — fires on genuine mismatch, reachable.** `infer_descriptor_
   network_kind` reads each xpub's `.xkey.network` (`build_descriptor.rs:484-485`), `None` for
   all-`Single` (raw pubkey) descriptors. The WARN path (`emit_human:518-528`) is reachable: `--network`
   supplied + `net.network_kind() != keys_kind` → stderr WARN, exit 0, still renders. Tests cover
   genuine-mismatch WARN, omitted-network-defaults-to-keys, and all-Single silence (7 tests green).

6. **L3 coldcard account bound — REJECTS, correctly placed.** `coldcard.rs:245-254` uses
   `u32::try_from(n)` on the `account` `u64`, returning `ImportWalletParse` (exit 2) on `>u32::MAX` —
   it REJECTS, does not saturate/truncate. Placed at the read site (Step 4) so it fires for ALL blobs
   regardless of which dominant-BIP branch is later taken (more conservative than strictly needed —
   the overflow only manifests in the legacy-xpub fallback, but rejecting early is sound). Test
   `l3_legacy_top_level_xpub_account_overflow_rejects` (account=2^32) → exit 2; positive control
   (account=5) bakes `m/84'/0'/5'` unchanged.

7. **Convert M14 / Export M13 — compare the right source.** M14 (`convert.rs:1107-1119`): decodes
   the OUTPUT xpub, asserts `xpub.network` vs `network.network_kind()` (the `--network` family) before
   the SLIP-132 prefix swap → refuses re-minting a tpub into a mainnet zpub family. M13
   (`export_wallet.rs:750-756`): asserts each `slot.xpub.network` vs the envelope's declared
   `bundle.network` family before any re-emit → refuses a mainnet-labeled envelope carrying testnet
   xpubs. Both compare xpub-OWN-network vs the asserted source; both `?`-propagate `NetworkMismatch`
   (exit 2). Tests `m14_…rejects` and `m13_mainnet_label_testnet_keys_envelope_rejects` green.

---

## Verdict

**LENS-B CORRECTNESS: 0C / 0I — GREEN**

The helper compares the xpub's OWN decoded NetworkKind vs the asserted (coin-type / `--network` /
envelope / WIF-byte) network — no tautological compare at any of the 11 call sites. Two-axis exit codes
(H9 → 1, helper → 2) are correct and never conflated. The `NetworkMismatch` rename ripple is complete
across exit_code / kind / Display / detail_json / unit tests. No wrong-network false-negative found;
a mutation test empirically confirms the check catches the case it claims (disabling it → wrong-network
exits 0, RED).
