# v0.34.7 argv-Hardening (`PR_SET_DUMPABLE`) — Cross-Repo Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development / executing-plans. Steps use checkbox (`- [ ]`). **NO parallel code-gen across repos — execute the 4 code repos in SERIES.**

**Goal:** Call `prctl(PR_SET_DUMPABLE, 0)` as the first statement in each of the 4 m-format CLIs' `main()`, denying other-uid `/proc/$PID/cmdline` reads + core dumps. Document the residual same-uid window. Closes `argv-overwrite-after-parse` (re-scoped to the user-approved `PR_SET_DUMPABLE` mitigation, not the fragile in-place argv overwrite).

**Architecture:** A tiny per-repo module `process_hardening` with `set_non_dumpable()` (Linux: `prctl(PR_SET_DUMPABLE,0)` best-effort; non-Linux: no-op) + a Linux-only unit test asserting `PR_GET_DUMPABLE == 0`. Called first in each `main()`. No shared crate (the 4 CLIs are independent) → identical ~12-line module duplicated per repo. No CLI surface change.

**Tech Stack:** Rust; `libc` (prctl). Repos: `mnemonic-toolkit`, `descriptor-mnemonic` (md-cli), `mnemonic-secret` (ms-cli), `mnemonic-key` (mk-cli), `mnemonic-gui` (pin bumps only).

**SemVer:** all PATCH. **No GUI `schema_mirror` / manual lockstep** (no flag). Cross-repo coordination via install.sh + pinned-upstream pin bumps. **crates.io publishes** for md/ms/mk (user-approved; re-confirm at the publish step — irreversible).

**Approved design:** user-approved 2026-05-22 — approach **A (`PR_SET_DUMPABLE`)**, **full cross-repo (4 binaries)**, **crates.io publishes yes**. Recon: `design/cycle-prep-recon-batch-4features.md`.

**Verified (live source):**
- `main()` openings: toolkit `main.rs:97` (`-> ExitCode`, then `Cli::try_parse()`); md-cli `main.rs:223` (`Cli::parse()`); ms-cli `main.rs:101`; mk-cli `main.rs:49` (all `-> ExitCode`).
- `libc` dep: toolkit ✓, ms-cli ✓; **md-cli ✗ + mk-cli ✗ (must add `libc = "0.2"`)**.
- Toolkit has a lib (`mnemonic_toolkit::mlock` precedent for libc FFI lib modules).
- Phase-1 advisory at `secret_advisory.rs:37` (warns, no mutation) — unchanged.
- Current versions: toolkit 0.34.6; md-cli 0.6.0, ms-cli 0.4.0, mk-cli 0.4.1; GUI 0.19.2.
- **Pins (R0-verified live):** `install.sh` = md `descriptor-mnemonic-md-cli-v0.6.0`, ms `ms-cli-v0.4.0`, mk `mk-cli-v0.4.1`, toolkit self `v0.34.6` (NOTE the **non-uniform** prefixes — md carries `descriptor-mnemonic-md-cli-`; ms/mk are un-prefixed). GUI `pinned-upstream.toml` = toolkit `v0.34.6`, md `…-md-cli-v0.6.0`, ms `ms-cli-v0.4.0`, **mk `mk-cli-v0.4.0`** (⚠️ the GUI mk pin is ALREADY a version behind — never bumped for mk-cli v0.4.1).
- **Codec publish prereqs (R0-verified on crates.io):** md-codec 0.35.0 ✓, ms-codec 0.2.0 ✓, mk-codec 0.3.1 ✓ — all published, so the 3 sibling-CLI `cargo publish` steps (their `=`/caret codec path-deps) resolve.

---

## The shared module (identical per repo)

`process_hardening.rs`:
```rust
//! Process-level secret-exposure hardening.
//!
//! `set_non_dumpable()` calls `prctl(PR_SET_DUMPABLE, 0)` (Linux), which:
//!   - makes `/proc/$PID/` owned by root + unreadable to OTHER non-root UIDs
//!     (so other users cannot read `/proc/$PID/cmdline` to harvest a secret
//!     passed inline on argv), and
//!   - disables core dumps (so a secret on argv/heap won't land in a core file).
//!
//! It does NOT hide cmdline from the SAME UID (a same-UID attacker already has
//! ptrace / `/proc/$PID/mem` access to the live process) — that residual is
//! accepted; the reliable, non-fragile companion to the `--*-stdin` advisories.
//! Best-effort: a `prctl` failure is ignored (hardening is advisory-grade).

/// Deny other-UID `/proc/$PID` reads + core dumps for this process.
/// Linux-only; a no-op on other platforms.
pub fn set_non_dumpable() {
    #[cfg(target_os = "linux")]
    unsafe {
        // SAFETY: prctl(PR_SET_DUMPABLE, 0) takes no pointers; always sound.
        let _ = libc::prctl(libc::PR_SET_DUMPABLE, 0);
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    #[test]
    fn set_non_dumpable_clears_dumpable_flag() {
        super::set_non_dumpable();
        // SAFETY: PR_GET_DUMPABLE takes no pointers.
        let d = unsafe { libc::prctl(libc::PR_GET_DUMPABLE) };
        assert_eq!(d, 0, "PR_SET_DUMPABLE(0) should leave dumpable == 0");
    }
}
```

The `main()` hook (first statement, before clap parse), per repo's module path:
```rust
fn main() -> ExitCode {
    <module>::set_non_dumpable();
    // …existing parse + dispatch…
}
```

---

## Task 1: md-cli (descriptor-mnemonic) → v0.6.1

**Files:** `descriptor-mnemonic/crates/md-cli/Cargo.toml` (+libc, version), `src/main.rs` (+module decl + hook), new `src/process_hardening.rs`, `CHANGELOG`.

- [ ] **Step 1:** Add `libc = "0.2"` to `[dependencies]` in `crates/md-cli/Cargo.toml`.
- [ ] **Step 2:** Create `crates/md-cli/src/process_hardening.rs` with the shared module (above).
- [ ] **Step 3:** In `crates/md-cli/src/main.rs`: add `mod process_hardening;` (with the other `mod` decls) + insert `process_hardening::set_non_dumpable();` as the first statement in `main()` (before `let cli = Cli::parse();` at `:224`).
- [ ] **Step 4:** Bump `version` in `crates/md-cli/Cargo.toml` `0.6.0 → 0.6.1`; `cargo build` to regen lock.
- [ ] **Step 5:** `cargo test -p md-cli` (the new unit test passes) + `cargo clippy -p md-cli --all-targets -- -D warnings` clean.
- [ ] **Step 6:** CHANGELOG entry (md-cli) `[0.6.1]` — argv-hardening PR_SET_DUMPABLE. File companion FOLLOWUP `argv-hardening-pr-set-dumpable` in this repo's `design/FOLLOWUPS.md` (if it has one) cross-citing the toolkit.
- [ ] **Step 7:** Commit on a branch `v0.6.1-argv-hardening`.

## Task 2: ms-cli (mnemonic-secret) → v0.4.1
Same as Task 1, EXCEPT **libc already present** (skip Step 1). Module + hook in `crates/ms-cli/src/main.rs` (first statement before `let cli = match Cli::try_parse()` at `:102`). Version `0.4.0 → 0.4.1`. CHANGELOG `[0.4.1]`.

## Task 3: mk-cli (mnemonic-key) → v0.4.2
Same as Task 1 (ADD `libc = "0.2"`). Hook in `crates/mk-cli/src/main.rs` (first statement before `let cli = match Cli::try_parse()` at `:50`). Version `0.4.1 → 0.4.2`. CHANGELOG `[0.4.2]`.

## Task 4: toolkit (mnemonic-toolkit) → v0.34.7

**Files:** `crates/mnemonic-toolkit/src/process_hardening.rs` (new, in the LIB), `src/lib.rs` (`pub mod process_hardening;`), `src/main.rs` (hook), `Cargo.toml` (version), `Cargo.lock`, `scripts/install.sh` (self-pin + sibling pins), `CHANGELOG.md`, `design/FOLLOWUPS.md`.

- [ ] **Step 1:** Create `crates/mnemonic-toolkit/src/process_hardening.rs` (shared module). Add `pub mod process_hardening;` to `src/lib.rs` (mirroring `pub mod mlock;`).
- [ ] **Step 2:** In `src/main.rs`: insert `mnemonic_toolkit::process_hardening::set_non_dumpable();` as the first statement in `main()` (before `let cli = match Cli::try_parse()` at `:98`).
- [ ] **Step 3:** Version `Cargo.toml` `0.34.6 → 0.34.7`; `cargo build -p mnemonic-toolkit` to regen lock (confirm Cargo.lock 0.34.7).
- [ ] **Step 4:** `scripts/install.sh`: self-pin `mnemonic-toolkit-v0.34.6 → -v0.34.7`; sibling pins `descriptor-mnemonic-md-cli-v0.6.0 → -v0.6.1`, `ms-cli-v0.4.0 → ms-cli-v0.4.1`, `mk-cli-v0.4.1 → mk-cli-v0.4.2`.
- [ ] **Step 5:** CHANGELOG `[0.34.7]` (text below). Close `argv-overwrite-after-parse` in `design/FOLLOWUPS.md` (resolution narrative below) + file the GUI/sibling companion FOLLOWUP cross-cites.
- [ ] **Step 6:** Full regression `cargo test -p mnemonic-toolkit` + clippy `--all-targets -D warnings` + manual lint 6/6 (no manual change; confirm no regression).
- [ ] **Step 7:** Commit on the toolkit branch `v0.34.7-argv-hardening` (this branch).

**CHANGELOG `[0.34.7]`:**
```
## mnemonic-toolkit [0.34.7] — 2026-05-22

**SemVer-PATCH — process argv-hardening (`PR_SET_DUMPABLE`).** `mnemonic` now calls `prctl(PR_SET_DUMPABLE, 0)` at the top of `main()` (Linux; no-op elsewhere), making `/proc/$PID/` unreadable to OTHER non-root UIDs and disabling core dumps — so a secret passed inline on argv (against the `--*-stdin` advice) can no longer be harvested by another user via `/proc/$PID/cmdline` or a core file. The residual same-UID `/proc/cmdline` window is documented + accepted (a same-UID attacker already has ptrace/`/proc/mem` access). The in-place argv-overwrite alternative was deliberately declined (glibc/musl/linking-fragile + racy). New `mnemonic_toolkit::process_hardening` lib module. Cross-repo: the same hardening lands in md-cli v0.6.1 / ms-cli v0.4.1 / mk-cli v0.4.2 (pins bumped). Closes `argv-overwrite-after-parse`.
```

**FOLLOWUP closure (`argv-overwrite-after-parse`):**
```
- **Status:** resolved — v0.34.7. Implemented the `PR_SET_DUMPABLE(0)` mitigation (SPEC §3 OOS-2 option (b)) across all 4 m-format CLIs: a `process_hardening::set_non_dumpable()` call at the top of each `main()` denies other-UID `/proc/$PID/cmdline` reads + core dumps (Linux; no-op elsewhere). The in-place argv-overwrite (option (a)) was deliberately DECLINED — Rust's std does not expose the original `argv`, and the `setproctitle`-style in-place mutation is glibc/musl/static-linking-fragile + racy + a corruption risk for marginal same-UID value (same-UID already implies ptrace/`/proc/mem` access). The residual same-UID `/proc/cmdline` window is documented + accepted. Shipped: mnemonic-toolkit v0.34.7 + md-cli v0.6.1 + ms-cli v0.4.1 + mk-cli v0.4.2. Closed via cycle-prep recon (SHA `<toolkit-tip>`).
```

## Task 5: GUI (mnemonic-gui) → v0.19.3 (pin bumps only)

**Files:** `mnemonic-gui/pinned-upstream.toml` (md/ms/mk + toolkit tags), `Cargo.toml` (toolkit git-dep tag + GUI version), `Cargo.lock`, `CHANGELOG.md`.

- [ ] **Step 1:** `pinned-upstream.toml`: toolkit `v0.34.6 → v0.34.7`; md `…-md-cli-v0.6.0 → -v0.6.1`; ms `ms-cli-v0.4.0 → -v0.4.1`; **mk `mk-cli-v0.4.0 → mk-cli-v0.4.2`** (R0 I1 — the GUI mk pin is currently `v0.4.0`, NOT v0.4.1; this is a 2-version catch-up bump, not v0.4.1→v0.4.2). `Cargo.toml` git-dep toolkit tag `v0.34.6 → v0.34.7`.
- [ ] **Step 2:** GUI version `0.19.2 → 0.19.3`; `cargo build --lib` (regen lock; fetch toolkit v0.34.7).
- [ ] **Step 3:** Schema gates with `MNEMONIC_BIN`=v0.34.7 binary: `schema_mirror` + `schema_mirror_secret_drift` + `gui_schema_conditional_drift` + `secret_taxonomy_pin` green (NO new flag → no schema delta; the `secret_taxonomy_pin` + supply-chain snapshot unchanged). Full suite + clippy.
- [ ] **Step 4:** GUI CHANGELOG `[0.19.3]` — pin bumps for the argv-hardening sibling/toolkit releases (no schema change).
- [ ] **Step 5:** Commit on a GUI branch.

## Task 6: Release train (GATED — user go-ahead per cadence; crates.io re-confirm)

Order (siblings first so the toolkit/GUI pins resolve). **Codec publish prereqs all satisfied (R0-verified): md-codec 0.35.0, ms-codec 0.2.0, mk-codec 0.3.1 are on crates.io** — the sibling `cargo publish` steps resolve. Run `cargo publish --dry-run` first per crate as a final guard.
- [ ] **6a:** md-cli — merge→master, push, tag `descriptor-mnemonic-md-cli-v0.6.1`, GH release, **`cargo publish` (crates.io — RE-CONFIRM)**.
- [ ] **6b:** ms-cli — tag `ms-cli-v0.4.1`, GH release, **`cargo publish`**.
- [ ] **6c:** mk-cli — tag `mk-cli-v0.4.2`, GH release, **`cargo publish`**.
- [ ] **6d:** toolkit — merge→master, push, tag `mnemonic-toolkit-v0.34.7`, GH release (install-pin-check: self-pin + sibling pins must match the just-cut tags).
- [ ] **6e:** GUI — merge→master, push, tag `mnemonic-gui-v0.19.3`, GH release (schema-mirror CI installs the new pins).

## Per-cycle gates
- Per-repo: the new unit test (Linux) + clippy `-D warnings` + that repo's existing suite stay green.
- **Mandatory opus R0 on THIS plan-doc → 0C/0I before any code** (in progress).
- **End-of-cycle opus review** on the combined cross-repo diff → GREEN before the release train.

---

## Self-review (writing-plans)
- **Spec coverage:** all 4 binaries hardened + GUI pin bump + FOLLOWUP close + release train. ✓
- **No placeholders:** the module + test + hook + per-repo paths/versions/libc-status all written. ✓
- **Type consistency:** `libc::prctl` / `PR_SET_DUMPABLE` / `PR_GET_DUMPABLE` are libc 0.2 items; `set_non_dumpable()` signature uniform. ✓
- **SemVer/lockstep:** all PATCH; no flag → no GUI schema_mirror / manual lockstep; pin bumps are the cross-repo coordination. ✓
- **Risk:** low per-repo (5-line prctl, well-trodden); the risk is cross-repo COORDINATION + the irreversible crates.io publishes (gated + re-confirmed). The unit test sets the test-binary process non-dumpable — harmless for `cargo test` (no ptrace/coverage dependency in CI); noted for the reviewer.
