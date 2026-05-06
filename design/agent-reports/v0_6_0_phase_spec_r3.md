# v0.6.0 Phase SPEC — code-architect convergence

**r1:** 2C/6I/3L/3N. Two Critical findings (§9 dispatch shape for phrase source; §7 stderr precedent + bundle inconsistency) plus six Important architectural gaps.

**r2:** 0C/1I. Critical findings + 5 of 6 Important findings folded. One residual: §9 WIF branch missing the §4 `--path` guard.

**r3:** 0C/0I — APPROVED.

## Final state

The SPEC at `crates/mnemonic-toolkit/design/SPEC_convert_v0_6.md` covers:

- §1 — node table (9 active nodes; `seed`/`raw_privkey`/`xprv`-via-ms1/`seed`-via-ms1 deferred for ms-codec v0.2; `md1` deliberately excluded as a bundle-only artifact).
- §2 — edge table with explicit BIP-39 dispatch path (parse phrase to entropy → derive_slot helper; entropy direct to helper); secp context note for non-BIP-39 edges.
- §3 — refusal taxonomy in three classes (cryptographic / lossy / cross-format pivot) with byte-exact stderr templates; `xpub → mk1` distinct refusal redirecting to `bundle`.
- §4 — specific refusals (WIF + `--path`; unreachable graph cells).
- §5 — grammar with single-from-value v0.6 constraint; §5.a stdin convention.
- §6 — ConvertJson schema-1 (independent from BundleJson); §6.a `from_value` privacy policy (omit when secret-bearing); §6.b array-order.
- §7 — secret-on-stdout warning (new convention; bundle inconsistency tracked in FOLLOWUPS).
- §8 — passphrase/language scope per (from, to); explicit `--passphrase`-on-non-PBKDF2-edge warning.
- §9 — implementation hooks; WIF branch guards `--path` per §4 before constructing sentinel xpub.
- §10 — out-of-scope deferrals.

## Architect-round drift addressed across implementation

Implementation-side changes pre-implementation:
- `derive::DerivedAccount` extended with `account_xpriv: Xpriv` field (folded I-1 across the SPEC and the helper). Build verified clean.
