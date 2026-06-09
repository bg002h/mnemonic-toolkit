<!-- VERBATIM opus-architect Phase-3 per-phase review, round 2, descriptor-builder. Persisted per CLAUDE.md. @ post-I1-fold; source b596d3f, miniscript 95fdd1c. Verdict: GREEN 0C/0I — Phase 3 cleared, only ship-lockstep remains. -->

# Phase-3 Review — descriptor-builder engine — round 2 — **GREEN** (0 Critical / 0 Important)

I confirm I1 is closed and the fold introduced no new Critical or Important findings. The SPEC watch-only-out invariant is now both enforced (step-1 screen) and pinned (no-leak test), and — the round-1 residual worry — **no secret form is emittable** rather than merely refused-with-possible-leak. Verified against the pinned miniscript source (`95fdd1c`).

## CRITICAL
None.

## IMPORTANT
None.

The raw-hex-privkey-as-x-only hole (#3) is **not a hole.** Refused at step 2, message no-echo. Verified: `DescriptorPublicKey::from_str` (`key.rs:725-784`) would parse a bare 64-hex as `SinglePubKey::XOnly`, BUT the IR renderer (`ir.rs::render`/`with_multipath`) unconditionally appends `/<0;1>/*` to every key, so the string fed to `from_str` is `<64hex>/<0;1>/*` — a `Single` key cannot carry a derivation suffix → `key_part.len()` ≈ 72 → `_ =>` → `InvalidPublicKeyLength` (`"public keys must be 64, 66 or 130 characters in size"`, static, no echo). Never reaches emit. The WIF path hits `KeyTooShort` (`"key too short"`, static). The only echoing key-parse arms (`DerivationIndexError{index}`, `MasterFingerprint{fingerprint}`) echo non-secret structural fragments and are unreachable for a bare secret in `pk()`.

## MINOR
- **M1 (carry-forward note):** `emit_human` writes the descriptor before the `?`-propagating cost preview → partial stdout on a cost error. Cap-agreement makes this practically unreachable for gate-passing input. Not a fix.
- **M2 (test-hardening, non-blocking):** the no-leak test lacked a raw-hex case (hits a different no-echo step-2 arm). Recommended adding one row. [Folded by implementer: `("pk(raw-hex)", ...)` added → pins the `InvalidPublicKeyLength` arm + proves raw-hex is refused-not-emitted.]

## What passes
- **I1 closed.** `check_secret_key` strips an optional `[origin]` via `rsplit(']').next().unwrap_or(key)` (correct in the no-origin case), flags `prv` at bytes `1..4` with a char-boundary guard. Catches all extended-private prefixes incl. SLIP-132 capitals (`Yprv`/`Zprv`/`Uprv`/`Vprv`). NO false positives on `xpub`/`tpub`/`[fp/path]xpub` (`pub` at 1..4). Message never echoes the key. Applied to Pk/Pkh + every multi/sortedmulti key.
- **No-leak test genuine + non-vacuous** across xprv (step-1 screen), WIF (step-2 `KeyTooShort`), raw-hex (step-2 `InvalidPublicKeyLength`), multi(xprv): exit 2, secret absent from BOTH channels, `stdout.is_empty()` (no descriptor), `--json` `descriptor` null + diagnostics present.
- **`validate_fields` exhaustive over all 17 variants**, no wildcard — future variant breaks compilation. Screen doesn't disturb archetypes (all xpub).
- **`DiagnosticKind::SecretKey` serializes `"secret_key"`** (`rename_all = "snake_case"` + `as_str`).
- 10 CLI / 34 bin / clippy-clean consistent with the code.

---

**Verdict: GREEN.** Phase 3 cleared at 0C/0I. Only the ship-lockstep remains:
- **manual mirror** (`docs/manual/src/40-cli-reference/` — `build-descriptor` is a new subcommand);
- **GUI schema-mirror** (`mnemonic-gui/src/schema/mnemonic.rs` — new subcommand `build-descriptor` + flags `--spec`/`--network`/`--format`/`--json`/`--spec-schema` + the `CliBuildFormat` value enum `descriptor`/`bip388`; lockstep per CLAUDE.md);
- **version bump** MINOR → v0.50.0 (new subcommand = new capability);
- **tag**.
