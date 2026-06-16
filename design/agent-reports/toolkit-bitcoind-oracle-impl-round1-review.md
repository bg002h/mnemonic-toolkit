# Impl review (toolkit bitcoind end-to-end oracle) round 1 — code-reviewer (verbatim)

> Persisted verbatim per CLAUDE.md. **Verdict: GREEN (0C/0I).** Reviewed the NEW test
> `tests/bitcoind_differential.rs`, the NEW shape-6 cell in `cli_restore_multisig_general.rs`,
> and the NEW `.github/workflows/bitcoind-differential.yml`. The test was EMPIRICALLY VALIDATED
> against a live offline `-chain=main` Bitcoin Core node BEFORE this review: 9 shapes, 45
> receive-address checks + original cross-check + change-branch equivalence + checksum
> round-trip per shape, all byte-identical.

---

## Verbatim agent output

## Review Verdict: GREEN (0C/0I)

No Critical or Important findings. Detailed verification of each focus area:

**1. Anti-vacuity integrity — sound.** The `#[ignore]` skip returns early only when all three wiring vars are unset (`bitcoind_differential.rs:316-321`); the workflow always exports them (`.yml:127-129`), so CI never green-by-skip, and partial-set panics (`read_wiring`). The `getblockchaininfo`/`chain=="main"` gate fails loud (lines 324-329). The golden is asserted three ways — independent rust-miniscript derivation at startup (line 336), restore's reported address per-shape (line 360), and a `golden_asserted` end-assert (line 424). The golden is a frozen constant (line 59), independent of the unit-under-test. The primary `reported[i] == core_recon[i]` compares the toolkit's own derivation vs Core's external C++ `deriveaddresses` — not trivially satisfiable.

**2. Funds-correctness — sound, not circular.** `reported` (toolkit) vs `core_recon` (Core on reconstructed) is a cross-implementation check. The original cross-check (lines 402-409) and change-branch equivalence (lines 415-421) catch a restore that reconstructs a different-but-valid descriptor. Indices correct: N=4, count=5, Core range `[0,4]` inclusive (5 addrs), loop `0..=(N as usize)` (5 iters) — no off-by-one. Confirmed `restore.rs:1354-1379` derives `first_recv` from the reconstructed descriptor STRING via `derive_receive_addresses`, which (`derive_address.rs:80-92`) splits multipath + takes the receive branch — matching `single_chain_desc(&recon_desc, 0)`.

**3. Multipath split + checksum — sound.** `into_single_descriptors().remove(chain)` correct for 0/1; restore emits `<0;1>/*` multipath (`restore.rs:1010-1078`), so the chain-1 split never panics on `singles.len() > chain`. `rsplit_once('#')` with a `panic!` fallback is sound (miniscript always emits `#csum` on `to_string()`; the panic is a loud harness failure, never a silent pass).

**4. Shape-6 cell — non-vacuous.** Asserts `and_v(v:pk(`, `older(144)`, NOT `wsh(pk(`, plus md1 fixed-point. Uniquely exercises the n=1 `v:pk` general path (no multi at trunk); RED if shape-6 restore collapsed to plain `wsh(pk(...))` or broke the timelock.

**5. Workflow correctness — sound.** Path triggers cover all relevant toolkit sources (5 source files + test + workflow all exist). Dropping the fork-clone is correct (toolkit pins `rev=95fdd1c` via `[patch.crates-io]` `Cargo.toml:28-29`). `rust-toolchain@1.85.0` matches `rust-toolchain.toml`. `MNEMONIC_BIN` build-then-run correct. `@v5` actions. Offline `-chain=main` lifecycle, cookie auth, 60s poll, stop-`if: always()`, debuglog-on-failure all match the validated md-codec template. No unquoted-var/silent-no-op; no command-injection surface (every `run:` input is a workflow-controlled constant).

**6. CI-impact / NO-BUMP — confirmed.** `#[ignore]` keeps default `cargo test` green; new workflow push-path/PR-path/cron/dispatch only; no flag/API/schema change (schema_mirror/manual-mirror untouched).

**7. No Rust correctness bug, panic-on-valid-input, or resource leak.** All `panic!`s are intentional fail-loud on broken provisioning/harness bugs; the test never spawns bitcoind (connect-only) → no lifecycle leak.

Sub-threshold (no action): the workflow sets no explicit `timeout-minutes`, relying on the runner default + 60s readiness cap — identical to the reference md-codec workflow, bounded in practice.
