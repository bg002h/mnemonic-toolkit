# v0.34.0 nostr-key-wrappers — end-of-cycle opus review (verbatim)

**Date:** 2026-05-22
**Reviewer:** opus `feature-dev:code-reviewer` (agent `a8193bef7f79fb873`)
**Scope:** full feature delta `f501ec3..HEAD` (crates/ + docs/), cross-validated against Electrum `bitcoin.py` source (the spec §9-O1 open item). Per-phase A/B reviews already GREEN; this focuses on holistic + C-phase + the Electrum-hint surface.
**Verdict:** **YELLOW** — 0 Critical, 2 Important, 2 Minor.

End-to-end feature matches approved scope (npub/nsec → p2tr native + non-taproot even-y, descriptor wrapper, WIF, no m-format cards). Secrets projection + gui-schema (21→22) + error wiring + ArgGroup + JSON + secret hygiene all correct. Two Important findings, both in the Electrum-import-hint surface (the §9-O1 item per-phase reviews couldn't close), both verified against Electrum source.

## Critical — None

## Important

### I1 — Manual's `electrum:` prefix table contradicts the code for `p2pkh` (code is right, manual wrong). Confidence 95.
`docs/manual/.../41-mnemonic.md:1895` documents `p2pkh` → "(no prefix — bare WIF)", but `nostr.rs:137` returns `"p2pkh:"` and `cmd/nostr.rs:159` prepends it unconditionally — so `--script-type p2pkh` emits `electrum: p2pkh:<WIF>`. Electrum master `bitcoin.py serialize_privkey` prefixes ALL types incl. `p2pkh` → the code is Electrum-faithful; the manual row is wrong. The flag-coverage lint checks flag presence, not content, so not CI-caught.
**Fix:** manual table row `p2pkh` → `p2pkh:`.

### I2 — Default-path `electrum: p2tr:<WIF>` hint is not importable into Electrum. Confidence 85.
`nostr.rs:139` emits `"p2tr:"`; manual `:1892` + primary nsec example `:1959` show it; `p2tr` is the DEFAULT `--script-type`. Electrum master `WIF_SCRIPT_TYPES` = `{p2pkh, p2wpkh, p2wpkh-p2sh, p2sh, p2wsh, p2wsh-p2sh}` — **no `p2tr`**; importing `p2tr:<WIF>` fails. The WIF is valid; only the hint is broken. §9-O1 verification closed `p2wpkh-p2sh` ordering but not the `p2tr`-unsupported case.
**Fix (chosen):** suppress the `electrum:` line for `p2tr` (`electrum_prefix` → `Option`, `None` for p2tr; cmd emits the line only when `Some`; JSON `electrum` already `skip_serializing_if`). Update the manual table + the default nsec example accordingly.

## Minor
- **M1** (`CHANGELOG.md:11`): claims "new direct dep `bech32`" but the impl uses the `bitcoin::bech32` re-export (`nostr.rs:54,56`); `Cargo.toml` adds no `bech32` line. Confidence 90. Fix: drop the clause.
- **M2** (`41-mnemonic.md:1977-1983`): the even-y `0x…06` worked example's `notice:` is not asserted by any CI-executed test (confidence 45, below threshold; optional). Optional: add a stderr-assertion cell on a known odd-y nsec.

## Verdict: YELLOW
Fold I1 (manual p2pkh row) + I2 (suppress electrum line for p2tr; manual + example) + M1 (changelog) before tagging v0.34.0 to satisfy the 0C/0I gate. Paired GUI (C5) deferred post-tag is NOT a blocker.
