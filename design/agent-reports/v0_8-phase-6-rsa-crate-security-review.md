# v0.8 Phase 6 SPIKE — `rsa` crate security audit

**Status:** SPIKE COMPLETE — verdict **DEFER `rsa` + `rsa-gpg`** (ship DICE only in Phase 7).

## Scope

Phase 6 audit per the v0.8 plan: pre-implementation review of the
RustCrypto `rsa` crate to determine whether it is safe to add as a
direct dependency for BIP-85 application `828365'` (RSA private key
generation) and `67797633'` (RSA-GPG keypair generation).

## Findings

### F1 — Unfixed timing-attack advisory: RUSTSEC-2023-0071

**Source:** `rustsec/advisory-db/crates/rsa/RUSTSEC-2023-0071.md` as fetched
on 2026-05-07.

> **patched = []**

The advisory is **still unpatched** as of 2026-05-07. The published
Marvin-attack timing sidechannel against RSA PKCS#1 v1.5 decryption is
known and acknowledged by the upstream maintainers ("work is underway to
migrate to a fully constant-time implementation"), but no patched
release is available on crates.io.

**CVSS 3.1:** `CVSS:3.1/AV:N/AC:H/PR:N/UI:N/S:U/C:H/I:N/A:N`. Network
attack vector; high attack complexity; high confidentiality impact.

**Aliases:** CVE-2023-49092, GHSA-c38w-74pg-36hr, GHSA-4grx-2x9w-596c.

### F2 — Marvin attack scope vs. BIP-85 RSA usage

The Marvin-attack vector is RSA PKCS#1 v1.5 **signature decryption** when
the attacker can observe timing. BIP-85 RSA application generates an RSA
key from BIP-85 entropy and emits the key as a PEM/DER string for export.
Key generation itself is **not in the Marvin attack surface**.

However:

- Any downstream consumer that uses the BIP-85-derived key with the same
  `rsa` crate's signing/decryption APIs becomes vulnerable.
- Our `cargo audit` output would flag the advisory in the dep tree,
  affecting downstream auditability.
- The advisory's CVSS-3.1 high-confidentiality-impact rating means
  organizational policies that gate on RustSec advisories would block
  consumption of mnemonic-toolkit wholesale.

### F3 — Crate stability

`rsa` crate's recent release cadence (per `RustCrypto/RSA/tags`) is
`v0.10.0-rc.18` (most recent), `v0.10.0-rc.17`, `v0.10.0-rc.16`, ... —
i.e., the crate is in extended pre-release (rc-18 cycles is unusual).
No stable `v0.10.0` is published. Stable line is `0.9.x`.

### F4 — User demand signal

BIP-85 application `828365'` (RSA) and `67797633'` (RSA-GPG) are niche
even within BIP-85. Practical use cases:
- Deterministic GPG identity recovery from a Bitcoin seed.
- RSA TLS certificates from a backup mnemonic.

These are research-grade applications. No major Bitcoin wallet implements
BIP-85 RSA. The reference Python impl (`ethankosakovsky/bip85`) supports
them but as a "completeness" feature.

## Verdict: DEFER `rsa` + `rsa-gpg`

**Reasoning:**

1. **Unfixed Marvin advisory.** Adding `rsa` to the direct dep tree adds an
   open security advisory to mnemonic-toolkit's `cargo audit` output. Even
   though BIP-85 key generation isn't directly vulnerable, the advisory
   propagates downstream.
2. **Pre-release stability.** Crate is in 18+ rc cycles for the
   constant-time refactor (`0.10.0-rc.x`). Adopting an unreleased major
   version is a maintenance burden and ties our release timeline to
   upstream's not-yet-published constant-time implementation.
3. **Niche demand.** No user has requested BIP-85 RSA / RSA-GPG support;
   the items are FOLLOWUP carry-overs from the v0.7 BIP-85 scope decision
   that explicitly deferred them pending demand signal.
4. **Plan natural-seam supports deferral.** Per the v0.8 plan:
   > "**Phase 6 spike returns rsa crate concerns:** defer Item #4 RSA +
   > RSA-GPG to v0.9; ship DICE only (no new dep)."

## Phase 7 disposition

**Phase 7 (BIP-85 RSA / RSA-GPG / DICE):** narrow Phase 7 scope to **DICE only**.

- **DICE** (`89101'`) needs no new dep — pure deterministic rejection
  sampling on BIP-85 entropy. Implement in ~40 LOC + tests.
- **RSA** (`828365'`) and **RSA-GPG** (`67797633'`): defer to v0.9 /
  pending crate stability + demand signal.

The v0.7.1 FOLLOWUP `bip85-rsa-rsa-gpg-dice-applications` is split into:
- `bip85-dice-application` — close in Phase 7 v0.8.0.
- `bip85-rsa-rsa-gpg-applications` — re-tier from `v0.8` to `v0.9 / pending-rsa-crate-stability`.

## Decision rule for v0.9 reconsideration

Reopen the audit if EITHER of the following holds:
1. **`rsa` crate publishes a patched stable version** (RUSTSEC-2023-0071
   marked `patched = ["X.Y.Z"]`), OR
2. **A user requests** BIP-85 RSA / RSA-GPG with a stated downstream use
   case.

Until both signals are absent, BIP-85 RSA + RSA-GPG remain DEFERRED.

## Spike duration

~25 minutes (no code written; advisory database lookup +
RustCrypto/RSA tag history + impact analysis).
