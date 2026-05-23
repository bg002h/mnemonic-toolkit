# v0.34.6 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-22
**Cycle:** mnemonic-toolkit v0.34.6 — `import-wallet --network` signet/regtest override
**Branch:** `v0.34.6-signet-network-override`
**Reviewer:** opus (feature-dev:code-reviewer), R0 (round 0)
**Target:** `design/IMPLEMENTATION_PLAN_v0_34_6_signet_network_override.md`

---

## Critical
(none)

Load-bearing mechanics verified correct: override block placement (after `import_wallet.rs:1135`, before seed-overlay :1151 + select-descriptor shadow :1179) mutates `parsed` in place via `iter_mut` → propagates to emit (`network_human_name(p.network)` @:1440); guard sound (BitcoinCoreParser yields only Bitcoin/Testnet); `network_human_name` (:2033-2041) covers all 4 + `_=>"unknown"`; JSON path `v[0]["bundle"]["network"]` matches real assembly (array @:1315, `env.insert("bundle",…)` @:1691, `Value::Array` @:1873); `coin_type()` returns `u32` (`network.rs:22`) so the `u32` annotation type-checks; standalone Round-1 path returns @:382 before :1119 so override scoping is correct.

## Important

**I1 — `message()` arm uses wrong rendering mechanism (`write!(f,…)` won't compile).** Plan Task 1 Step 3. Actual mechanism is `pub fn message(&self) -> String` (`error.rs:547`) whose arms return `String` via `format!`/`.clone()`/`.to_string()` (sibling FormatMismatch @`error.rs:662-664` = `format!(…)`). `Display` (`error.rs:753-755`) wraps `message()` with `error: {}`. The `write!(f,…)` form has no `f` in scope → won't compile. **Fix:** `format!`-returning arm in `message()`, inserted between `:664`/`:665`. exit_code arm (`=>1` between :468/:469) + kind arm (`=>"ImportWalletNetworkClassMismatch"` between :524/:525) correctly shaped.

**I2 — `CliNetwork::as_str()` does not exist — method is `human_name()`.** Plan lines 17, 188, 276. Actual accessor: `pub fn human_name(&self) -> &'static str` (`network.rs:49-56`). `override_net.as_str()` won't compile. **Fix:** use `override_net.human_name()` in the override block + correct refs.

**I3 — GUI toolkit pin is at `v0.34.2`, not `v0.34.5`.** Plan Task 5 Step 2. Live pins: `mnemonic-gui/Cargo.toml:42` + `pinned-upstream.toml:22` = `mnemonic-toolkit-v0.34.2` (v0.34.3-5 were toolkit-only, no GUI lockstep). A `v0.34.5→v0.34.6` replace no-ops. **Fix:** bump `v0.34.2 → v0.34.6` in both; note v0.34.3-5 carried no CLI-surface change so the only schema-mirror delta is `--network` (no cumulative backfill).

## Minor
- **M1** — Task 5 Step 1 omits the required `help: &str` field on the new `FlagSchema` (won't compile). Add a help string mirroring `--format`'s entry.
- **M2** — verified-facts bullet (plan:19) says `network_human_name` is "inside `BundleJson`"; it's a free `pub(crate) fn` @:2033 *called* in the `BundleJson{…}` initializer @:1440. Cosmetic.
- **M3** — manual import-wallet `--format` value-list is pre-existing stale (2 of 8 values); flag-coverage gates on flag NAMES not value-content, so non-blocking + out of scope.

---

VERDICT: YELLOW (3I/3M)

Three Important are hard compile/correctness blockers (message mechanism, nonexistent as_str, wrong GUI from-version). No deep design flaw — guard logic, override propagation, variant placement, coin_type type-match, JSON path all verified correct. Fold + re-dispatch R1; converges quickly.

---

## Fold disposition (controller) — round 0 → R1
Folded all 3 Important + M1 + M2 into the plan-doc:
- I1: message arm → `format!`-returning in `message()` (insert :664/:665); exit_code :468/:469; kind :524/:525.
- I2: `as_str()` → `human_name()` (override block + refs).
- I3: GUI pin bump `v0.34.2 → v0.34.6` (both files) + no-backfill note.
- M1: added `help:` to the GUI FlagSchema entry.
- M2: corrected `network_human_name` description (free fn).
- M3: left (pre-existing, out of scope).
Re-dispatching R1 against the folded plan.
