# Phase 1 R0 review — wallet-import v0.26.0

**Date:** 2026-05-18
**Reviewer:** opus architect
**Commit under review:** `6b0588f` (`phase 1: cross-cutting @env:VAR sentinel resolver`)
**Worktree:** `.claude/worktrees/wallet-import-export-multiformat-brainstorm`

**Verdict:** YELLOW — 0 Critical, 2 Important. Fold both Importants before Phase 2 dispatch.

The Phase 1 implementation is broadly sound. The resolver is correctly placed, correctly wired across 6 callsite files, and correctly defers to the existing v0.25.1 empty-string sentinel semantics. The two Important findings concern (1) a load-bearing UX inversion where the argv-leak advisory misinforms users of the new `@env:` path, and (2) a SPEC table inaccuracy (row 6 names a non-existent CLI form) that the implementer correctly worked around but did not file a FOLLOWUP for.

## Critical

(none)

## Important

### I1 — `secret-in-argv` advisory fires spuriously when `@env:VAR` was used

**Sites:**
- `crates/mnemonic-toolkit/src/cmd/bundle.rs:142-153` — resolve sentinels at L142-147, then unconditionally call `emit_secret_in_argv_advisories(args, stderr)` at L153.
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:163-174` — same pattern.
- `crates/mnemonic-toolkit/src/cmd/convert.rs:714` + the advisory site.
- `crates/mnemonic-toolkit/src/cmd/derive_child.rs:116-124` — same pattern.

After `resolve_env_sentinels` runs, the shadowed `args` carries the *resolved* secret value (no longer the literal `@env:VAR`). `emit_secret_in_argv_advisories` inspects `args.passphrase.is_some()` and emits `warning: secret material on argv (--passphrase) — pipe via --passphrase-stdin to avoid /proc/$PID/cmdline exposure` — but the user did NOT put the secret on argv; they used `@env:WALLET_PP`, exactly the leak-mitigation channel this cycle introduces.

Consequence: users following the new `@env:` workflow get a misleading warning, undermining the v0.26.0 design intent.

The argv-leak audit test only checks `/proc/<pid>/cmdline`; does not check stderr for spurious advisories.

**Fix:** Capture `had_sentinel` per-flag BEFORE resolution; skip advisory when the original value was a sentinel.

### I2 — SPEC §3.1 row 6 (`--slot @N.ms1=`) names a non-existent CLI form

**Sites:** `SPEC §3.1 row 6` + `BRAINSTORM §1.4 row 6` + `IMPLEMENTATION_PLAN §1.2`.

`SlotSubkey` at `slot_input.rs:17-32` is exhaustively `{Phrase, Entropy, Xpub, MasterXpub, Fingerprint, Path, Wif, Xprv}` — no `Ms1` variant. The token list at `slot_input.rs:147` confirms `ms1` is not accepted by `parse_slot_input`.

Additionally, the implementer's wiring expanded scope beyond SPEC §3.1's 6 rows:
- `convert.rs:1590-1595` resolves `--from <node>=` for any `is_secret_bearing()` node (covers `phrase`, `entropy`, `xprv`, `wif`).
- `derive_child.rs:399-402` — same pattern.
- `slip39.rs:261-263` (split) + `seed_xor.rs:140-141, 150-152` — `--from` + `--share <node>=`.

This is a correctness expansion (all genuinely secret-bearing), but it broadens beyond SPEC §3.1's locked enumeration. SPEC §3.2 says "ONLY at the 6 secret-flag surfaces enumerated in §3.1."

**Fix:** Amend SPEC §3.1 row 6 + add row 7 for `--from` composite-node form covering secret-bearing nodes; update §3.2 normative claim.

## Minor

### m1 — Cell §1.13 effectively a null test

`env_var_mixed_with_literal_ms1` at `cli_env_var_sentinel.rs:489-527` invokes verify-bundle without required `--mk1/--md1` → clap-time validation fires before `run()`; resolver never executes. The assertion is vacuously true. Acceptable for Phase 1 GREEN; file FOLLOWUP `phase-1-cell-1-13-restrengthen`.

### m2 — Empty `@env:VAR` on `--slot @N.phrase=@env:VAR` with `VAR=""` bypasses parser's empty-value rejection

Downstream BIP-39 / entropy parse fails differently than parser's rejection. Behavior remains an error; class diverges. Not load-bearing.

### m3 — `--slot @0.entropy=` cell (§1.11) assumes 64-hex entropy ≡ all-zero TREZOR 24-word identity

True per BIP-39 spec but load-bearing on that identity for the test assertion.

## Per-deviation verdict

- **§1.1 placement (`env_sentinel.rs` instead of `secrets.rs`): ACCEPT.** `secrets.rs` is library-public (`lib.rs:61`); `ToolkitError` is binary-private (`main.rs:9`). The binary-private `env_sentinel.rs` is the cleanest path.

- **§1.11 cell substitution (`entropy=` for `ms1=`): ACCEPT WITH AMEND.** Substitution correct; SPEC §3.1 row 6 needs amend per I2.

- **§1.9 cell substitution (Err-only): ACCEPT.** Wire exercised end-to-end via unset-fail; happy-path covered by unit test in env_sentinel.rs.

## Notable strengths

1. Ordering discipline — resolver runs before mlock pin so the actual secret bytes get pinned, not the sentinel.
2. Cheap pre-check pattern avoids `args.clone()` in no-sentinel common case.
3. `.env(VAR, VALUE)` subprocess scope keeps tests parallel-safe.
4. Empty-string preservation correctly threaded for v0.25.1 watch-only semantics.
5. Whole-value sentinel discipline (`strip_prefix`) matches SPEC §3.2.
6. Error templates match SPEC §2.4 byte-exactly.
7. D17 variant naming (`EnvVarMissing` no `ImportWalletEnvVarMissing`).
8. Argv-leak audit cell is robust (2s poll, positive + negative assertions, Linux-gated).

## Phase 2 dispatch readiness

After I1 + I2 fold (one commit; ~20 LOC + 2-3 assertions for I1, ~10 lines SPEC text for I2), Phase 1 ships GREEN.
