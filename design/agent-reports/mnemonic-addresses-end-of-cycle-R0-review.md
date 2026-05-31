# End-of-cycle R0 review — v0.38.0 `mnemonic addresses`

Reviewer: feature-dev:code-reviewer (opus). Reviewed the full cycle diff `master..HEAD` against the
GREEN spec/plan + real source (address_render, addresses.rs end-to-end, the rewired convert/xpub-search
sites, derive_slot, env_sentinel, the coverage-gate tests, cli_addresses, CHANGELOG, READMEs,
Cargo.toml/lock, manual lockstep surfaces).

Confirmed-correct: dedup behavior-preserving (byte-faithful lifts, by-value/by-ref to_pub() preserved,
all 4 call-sites rewired, no dead imports); xpub-branch guards (account/passphrase reject, kind-guard,
bcrt1/tb1/mismatch); `resolve_indices` ceiling math (rejects 2^31+1 / allows 2^31; range b<2^31, a≤b;
per-index map_err; chain 0/1 unwrap safe); exit codes → BadInput/Bitcoin (no panic); argv advisory
scoping; single-stdin guard; non-English non-fire; JSON shape (string schema_version, account omitted
for xpub); coverage gates (addresses alphabetical count 26; FROM/FLAG_ROUTES evidence anchors present);
release hygiene (0.38.0, lock synced, CHANGELOG, both READMEs); 15+2 tests non-vacuous, convert oracle.

## Critical — None.

## Important

**I1 — seed-branch intermediate `entropy` is a plain `Vec<u8>`, not `Zeroizing` (deviates from the
crate convention + spec §3.1 hygiene claim).** `addresses.rs:193` `let entropy: Vec<u8> = match …
.to_entropy()…` holds the raw BIP-39 entropy (master secret) and is freed WITHOUT scrubbing. Every
analogous convert site wraps it `zeroize::Zeroizing<Vec<u8>>` (convert.rs:1147/1162/1440). Defense-in-
depth (not a stdout leak), but a clear deviation. **Fix:** `let entropy = zeroize::Zeroizing::new(match
…)` (or annotate `Zeroizing<Vec<u8>>`); `&entropy` still derefs to `&[u8]` for `derive_bip32_from_entropy`.

**I2 — manual lockstep not done; CLAUDE.md mirror invariant unsatisfied at the tag boundary.** No
`mnemonic addresses` section in `docs/manual/src/40-cli-reference/41-mnemonic.md`; absent from
`docs/manual/tests/cli-subcommands.list`. CLAUDE.md requires the manual under `40-cli-reference/` to
update in lockstep with the implementing PR (`manual.yml` CI-gates it). The plan deferred this to
Phase 6 (after ship) — but the in-repo manual is not lagging-by-design; it must land before the tag.
Same for the paired `mnemonic-gui/src/schema/mnemonic.rs` `addresses` SubcommandSchema + pin (sibling
repo; schema_mirror fires on next pin bump). **Fix:** complete Phase 6 before tag — add the manual
section (every flag) + cli-subcommands.list line + flag-coverage lint; land the paired GUI entry.

## Minor
- **M1** — `cli_gui_schema.rs:104` message still says "all 25 user-facing subcommands"; list is now 26.
  Sync the string (the assert is correct).
- **M2** — `addresses.rs:134-142`: the inline-`--passphrase` argv advisory fires for ANY source incl.
  `xpub=`, immediately before the xpub branch rejects `--passphrase`. Harmless; could gate on non-xpub.

**VERDICT: RED (0C/2I)**

---

## Fold applied (controller, verified @ branch)
- **I1:** confirmed convert.rs:1147 Zeroizing precedent. `addresses.rs` entropy → `Zeroizing::new(match …)`.
- **I2:** manual section + cli-subcommands.list + flag-coverage; paired GUI schema-mirror + pin (done
  before tag, not deferred).
- **M1:** message "25"→"26". **M2:** passphrase advisory gated to non-xpub sources.
