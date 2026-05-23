# Recon — BIP-352 Silent Payments support in mnemonic-toolkit

**Date:** 2026-05-23
**Question:** can the toolkit support Silent Payments (e.g. address derivation)?
**Verdict:** **YES for the receiver-side static address derivation** (squarely in-scope; feasible with existing deps, no new crate). Sender output construction + chain scanning are **out of scope** (transaction-inputs / chain-access / signing-adjacent — the same boundary that excludes PSBT signing).
**Primary source:** BIP-352 (`bitcoin/bips/bip-0352.mediawiki`, fetched 2026-05-23) + official test vectors `bip-0352/send_and_receive_test_vectors.json`.

---

## What BIP-352 needs, per role

### Receiver — static address derivation  ← THE IN-SCOPE PIECE
From a BIP-39 seed, two hardened BIP-32 derivations (`purpose = 352'`):
- **scan key**:  `m / 352' / coin_type' / account' / 1' / 0` → `b_scan`, `B_scan = b_scan·G`
- **spend key**: `m / 352' / coin_type' / account' / 0' / 0` → `b_spend`, `B_spend = b_spend·G`
- `coin_type` per BIP-44 (0 mainnet / 1 testnet); `account` selects independent addresses.

**Address** = bech32m, **HRP `sp`** (mainnet) / **`tsp`** (testnet), **version char `q`** (v0), payload **66 bytes = `ser_P(B_scan) || ser_P(B_spend)`** (two compressed 33-byte SEC1 pubkeys). ~117 chars.
- **Base/unlabeled address** = `B_scan || B_spend` directly (the receiving address you publish).
- **Labels** (optional extension): replace `B_spend` with `B_m = B_spend + hash_BIP0352/Label(b_scan‖m)·G`; `m` is a label int. **Defer to a follow-on** — the base address needs no hashing/tweaking. (⚠️ exact base-vs-label semantics to be re-confirmed against the test vectors at impl.)

**This is pure key-derivation + encoding** — no ECDH, no signing, no chain. Directly analogous to `mnemonic nostr` (seed→key→address) and the existing address derivation.

### Sender — output construction  ← OUT OF SCOPE
Needs the sender's **input private keys** (`a = Σ a_i`), the tx's smallest outpoint, ECDH (`input_hash·a·B_scan`), per-output tagged-hash tweak, and emits a **P2TR** output `P_k = B_m + t_k·G`. This is transaction-construction/signing territory (the toolkit has no tx inputs and does not sign — same boundary as `bip174-psbt-signing`).

### Receiver — scanning  ← OUT OF SCOPE
Needs **chain data** (every candidate tx's inputs + taproot outputs), ECDH with `b_scan`, and an output-index search loop. Requires chain access the toolkit doesn't have.

---

## Toolkit capability check (all primitives already present)

| Need (receiver address) | Toolkit has it? |
|---|---|
| BIP-32 master + `m/352'/…` child derivation | ✓ `bitcoin::bip32::{Xpriv,DerivationPath}`; precedent `derive_slot.rs:59` (`Xpriv::new_master`) + `:65` (`.derive_priv(&secp,&path)`) |
| secp256k1 pubkey from priv (`B = b·G`) | ✓ `bitcoin::secp256k1` (used in `nostr.rs`, `bip85.rs`) |
| Compressed SEC1 serialization | ✓ `PublicKey::serialize()` (bitcoin) |
| bech32m encode, custom HRP `sp`/`tsp`, version `q` | ✓ `bitcoin::bech32` (decode used at `nostr.rs:54`; `bech32::encode` + `Hrp::parse("sp")` for emit) |
| tagged hash (only if labels) | ✓ `bitcoin::hashes::sha256t` (bitcoin dep) |
| network/coin-type mapping | ✓ `CliNetwork::coin_type()` (`network.rs`) — reuse |
| secret-handling (seed/scan/spend keys) | ✓ `secret_advisory` + `mlock` + `secrets::flag_is_secret` |

**No new dependency required** — the base address is hand-rollable (2 derivations + concat + bech32m), the same way SLIP-39/nostr were hand-rolled. (A dedicated `silentpayments`/`bip352` crate exists in the ecosystem but is unnecessary for receiver-address derivation, and `bitcoin = "0.32"` predates any in-crate SP module.)

---

## How it would fit (proposed shape — NOT a commitment)

A new top-level subcommand, e.g. **`mnemonic silent-payment`** (mirrors the `mnemonic nostr` cycle precedent):
- Input: a secret (BIP-39 phrase / ms1 / xprv, via the shared `--secret*` intake) + `--network` + `--account` (default 0).
- Output (human + `--json`): the **`sp1…`/`tsp1…` static address**, plus `B_scan` + `B_spend` compressed pubkeys + the two derivation paths. Optionally the scan/spend **private** keys behind the secret-on-stdout advisory (the address itself is public/watch-only).
- **No m-format cards** (SP is not an m-format artifact — same call as nostr).
- Scope v1 to the **unlabeled base address**; labels = a deliberate follow-on FOLLOWUP.
- Sender + scanning explicitly documented as out-of-scope (signing/chain boundary).

**SemVer:** new top-level subcommand → **MINOR** (next would be `v0.35.0`). **Lockstep:** GUI `schema_mirror` (new subcommand `SubcommandSchema`) + manual chapter — both mandatory.
**Cross-validation:** derive the addresses for the BIP-352 official `send_and_receive_test_vectors.json` receiver seeds and assert byte-exact `sp1…` match (the nostr-cycle "cross-impl vs authoritative oracle" pattern).

---

## Open design questions for a brainstorm
1. Base address only, or labels too (change-label m=0 + arbitrary labels)? — recommend base-only v1.
2. Emit scan/spend **private** keys at all, or address+pubkeys only? (scan key is "online", spend key is cold — emitting privs needs the secret-on-stdout advisory.)
3. Subcommand name (`silent-payment` vs `sp-address`) + whether it accepts an xprv directly (not just a phrase).
4. Re-confirm the base-vs-labeled address payload (`B_scan‖B_spend` vs `B_scan‖B_m`) against the test vectors at R0.

**Next step (per discipline):** if you want it, this feeds a **brainstorm → plan-doc → mandatory opus R0 (0C/0I) → impl**. Recon only — nothing built.
