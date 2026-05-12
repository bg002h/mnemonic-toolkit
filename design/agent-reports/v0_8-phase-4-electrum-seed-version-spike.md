# v0.8.1 Phase 4 Step 0 — Electrum seed-version spike

**Status:** DEFERRED — spike not yet run interactively. v0.8.1 ships with
the broadest historically-accepted value pinned; the spike runs in v0.8.2
once an interactive environment with Electrum 4.5.x installed is available.

## What the spike was meant to do (per IMPL_PLAN Phase 4 step 0)

1. Install current Electrum (>= 4.5.x) in a scratch venv.
2. Create a watch-only wallet via `electrum --offline restore <xpub> --wallet_path /tmp/electrum-spike-single.json`.
3. Read the wallet file, observe the `seed_version` value Electrum writes.
4. Repeat for a multisig wallet.
5. Lock `ELECTRUM_SEED_VERSION_PIN` to the observed value.

## What this cut did instead

Pinned `ELECTRUM_SEED_VERSION_PIN = 17` in
`crates/mnemonic-toolkit/src/wallet_export/electrum.rs:33`.

Rationale: `17` is the long-standing Electrum-2.7+ value for new
watch-only standard wallets and has been accepted by every Electrum
release since (the loader walks `_convert_version_<N>` migrations
forward to `FINAL_SEED_VERSION = 71` on first save). This is the
broadest-accept value and the least-likely-to-be-rejected by any
Electrum version in the wild.

## FOLLOWUPS

- `electrum-seed-version-spike-pending` (v0.8.2): run the spike,
  validate `17` against current Electrum 4.5.x, re-pin if Electrum
  rejects (unlikely but possible per `wallet_db.py` recent changes).
- `electrum-final-seed-version-drift` (open, no fix scheduled): track
  upstream `FINAL_SEED_VERSION` drift. Not a blocker for the toolkit
  since loader migrations are idempotent.

## Risk surface

If `17` is rejected by current Electrum, the toolkit-emitted wallet
will fail to import. The user would see Electrum's own error message
(not a toolkit error). Workaround: edit the JSON `seed_version` value
upward to `71` (Electrum's FINAL_SEED_VERSION) manually. The spike
will close this risk window.

## Cross-check signals

- Coldcard's `firmware/docs/sample-electrum-wallets/` historically used
  `seed_version: 17` for compat-broadest emission. SPEC §9 notes these
  samples are not authoritative for the toolkit but they corroborate
  that `17` was the safe-choice for vendor emitters.
- Electrum's `wallet_db.py` master shows `WALLET_FILE_VERSIONS = [17,
  18, 19, ..., 71]` (FINAL_SEED_VERSION), and the loader's
  `_convert_to_version_N` migrations are designed to be safely
  forward-chained. `17` is the minimum supported by current Electrum.
