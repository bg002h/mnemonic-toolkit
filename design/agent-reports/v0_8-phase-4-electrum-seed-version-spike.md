# v0.8.1 Phase 4 Step 0 — Electrum seed-version spike (executed)

**Status:** EXECUTED — 2026-05-12 against Electrum 4.5.5 in
`/tmp/electrum-spike-venv/`. `ELECTRUM_SEED_VERSION_PIN = 17` is
**empirically validated** as accepted on load by current Electrum.

## What was done

1. Cloned `https://github.com/spesmilo/electrum.git --branch 4.5.5` to
   `/tmp/electrum-src/`. Confirmed `FINAL_SEED_VERSION = 59` at
   `electrum/wallet_db.py:75` (master at the time of the SPEC §9
   reference may have been 71; release tag 4.5.5 is 59).
2. Created scratch venv at `/tmp/electrum-spike-venv/` (Python 3.14.4).
3. Installed Electrum 4.5.5 editable: `pip install -e .`.
4. Installed `cryptography` + `electrum-ecc` (the latter provides the
   bundled libsecp256k1 binary). Symlinked the libsecp into
   `/tmp/electrum-src/electrum/libsecp256k1.so.2`.
5. Ran `electrum --offline -w /tmp/electrum-spike-single.json restore
   <TREZOR_24-zpub>` — Electrum **wrote** a wallet file with
   `seed_version: 59` (FINAL_SEED_VERSION).
6. Copied the toolkit's own pinned fixture (`tests/export_wallet/electrum_single.json`,
   `seed_version: 17`) to `/tmp/electrum-toolkit-v17-single.json` and
   ran `electrum --offline -w /tmp/electrum-toolkit-v17-single.json listaddresses`.

## Result

`listaddresses` returned the expected BIP-84 receive set derived from
the toolkit's pinned zpub. Electrum's loader walked the migration
chain from 17 → 59 cleanly:

```
$ electrum --offline -w /tmp/electrum-toolkit-v17-single.json listaddresses
[
    "bc1q2m88xc45pfc8jugwe2t79yz3lrfkta2mjm28pq",
    "bc1qtwrqfrrvacuuge7rwwyndjmekcwxvtssh5nemm",
    "bc1qpx2mkpmq40a6t3tqggemrp8zeztkhr0lzty59z",
    "bc1q83eunsqpmxwnfzm99vp0xsnjsu8j99laajkrem",
    "bc1q0yjun54qxs70uv4gtkdaqec6qh7ttkdtxwrt7y",
    "bc1qdmr6q7shm3pswdpl9dcp0f2xa9p2fwgp0s3fjt",
    "bc1qvpv8zxvtm9nz82rh4xfwv664ju92nkzq0tuk5p"
]
```

The wallet file was rewritten in-place by Electrum's save logic; the
post-load file carries `seed_version: 59` (Electrum migration completes
on first save).

## Source-code cross-check

`electrum/wallet_db.py:1195-1211` (`get_seed_version`):

```python
if seed_version >= 12:
    return seed_version
if seed_version not in [OLD_SEED_VERSION, NEW_SEED_VERSION]:
    self._raise_unsupported_version(seed_version)
```

- `seed_version >= 12` is True for 17 → returns 17 (no rejection).
- Specific rejections at lines 1203 (`seed_version == 14 and seed_type
  == 'segwit'`) and 1205 (`seed_version == 51 and _detect_insane_version_51()`)
  do not match 17.
- `_raise_unsupported_version` rejects in `[5, 7, 8, 9, 10, 14]` only.

`17` is in the supported range and the migration chain (`_convert_version_13_b`
through `_convert_version_X`) handles forward migration to FINAL_SEED_VERSION.

## Pin rationale (validated)

- **Why 17 (not 59):** SPEC §9 says "minimum seed_version that current
  Electrum imports cleanly for watch-only wallets". 17 is in the
  always-accepted range; 59 is the value Electrum WRITES, not the
  minimum it ACCEPTS. Pinning to 17 maximizes downstream compatibility
  with older Electrum installs while remaining cleanly loadable by
  current Electrum.
- **Why not lower (e.g., 12-16):** 14 is rejected for `seed_type ==
  'segwit'` per line 1203; 11 (NEW_SEED_VERSION) and 4 (OLD_SEED_VERSION)
  are accepted but trigger legacy-format code paths. 17 is the
  oldest version above the special-case rejection band.

## Side observation (informational, not blocking)

Electrum's loader nulled the `root_fingerprint` field in its
re-serialized form despite the toolkit emitting
`"root_fingerprint": "5436d724"`. The wallet still imports + lists
addresses correctly; the fingerprint is required only for PSBT-with-origin
flows (not for watch-only address derivation). Tracked as
`electrum-root-fingerprint-roundtrip-quirk` (new FOLLOWUPS entry,
informational/tracking).

## FOLLOWUPS update

`electrum-seed-version-spike-pending` → **resolved**. Citation: this
report file at `design/agent-reports/v0_8-phase-4-electrum-seed-version-spike.md`.
Pin of 17 retained (empirically validated; no re-pinning needed).
