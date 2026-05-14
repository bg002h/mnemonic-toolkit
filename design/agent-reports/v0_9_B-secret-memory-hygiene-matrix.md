# v0.9.0 Cycle B — secret-memory-hygiene matrix (mnemonic-toolkit)

**Cycle:** v0.9.0 Cycle B — `mlock(2)` page-pinning, POSIX-only (Linux + macOS). Cross-repo: toolkit + `ms-cli`.
**SPEC:** `design/SPEC_secret_memory_hygiene_v0_9_B.md` (P0 ship `0c02247`; v3-fold Path B-lite RESCOPE LOCK `7cb2527`).
**Plan:** `/home/bcg/.claude/plans/v0_9_B-secret-memory-hygiene-cycle-b.md`.
**Path B-lite proposal (LOCKED):** `~/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`.
**Cycle reports:**
  - Phase 0: `v0_9_B-phase-0-spec-r1.md`, `…-r2.md` (R1 2C/3I folded; R2 0C/0I).
  - Phase 1: `v0_9_B-phase-1-bip85-heap-promote-r0.md`, `…-r1.md` (bip85 heap-promote precursor).
  - Phase 2: `v0_9_B-phase-2-mlock-module-r0.md` (Fix B trigger), `…-r0-fixb-verify.md`, `…-r1.md`, `…-r2.md` (mlock module).
  - Phase 3a (RESCOPE): `v0_9_B-phase-3a-rescope-r0.md`, `…-r0-v3.md`, `…-r0-v3-fold.md` (Path B-lite LOCK 0/0).
  - Phase 3a (impl): `v0_9_B-phase-3a-r1.md` (CLEAR).
  - Phase 3b (cross-repo impl): `v0_9_B-phase-3b-r1.md` (CLEAR).
  - Phase 3a R0 v2 LOCK (superseded by v3-fold): `v0_9_B-phase-3a-toolkit-applications-r0.md`.

This matrix is the cross-repo audit hub for Cycle B's mlock layering atop Cycle A's OWNED-buffer Zeroizing discipline. Every SPEC §2 row gets a status cell here (§1); every SPEC §3 OOS entry is surfaced for forward visibility (§3); the Cycle A → Cycle B carry-over list (Cycle A matrix §4 lines 247-269) is closed-out per-site here (§2); the Path B-lite carve-out to v0.10.1 patch is named (§4); SPEC §6 acceptance gates are checklisted (§5).

## §0 Cross-repo coverage

| Repo | Tag (PE close) | Phases participated | Matrix file | Delta cells |
|------|----------------|---------------------|-------------|-------------|
| mnemonic-toolkit (this repo) | `mnemonic-toolkit-v0.10.0` @ PE tag commit | 0 (SPEC), 1 (bip85 heap-promote), 2 (mlock module + CI workflow), 3a (Sites 1+2+3+4 apply + main wire + release-build CI job), PE (this matrix + FOLLOWUPS close + CHANGELOG + tag) | this file (§1 + §2 + §3 + §4 + §5) | 4 mlock-site applies (Sites 1+2+3+4) + 1 mlock module (`src/mlock.rs`, 533 LOC) + 1 new Rust CI workflow + 1 release-build subprocess job + 13 ctor sites populated (`_entropy_pin`) + 7 bip85 function-local pins + 4 per-handler Site 1 pins |
| mnemonic-secret | `ms-cli-v0.3.0` @ PE tag commit | 3b (inline mlock.rs copy + Site 5 apply + main wire + libc dep + companion FOLLOWUP), PE (companion matrix companion line + FOLLOWUPS close + CHANGELOG + tag) | none (toolkit-side canonical; this file is the cross-repo hub) | 1 mlock module inline-copy (`crates/ms-cli/src/mlock.rs`, 538 LOC; differs from toolkit only in `//!` mod-doc text — see §5 G6) + 1 Site-5 function-local pin + 1 main wire + 1 new `libc` dep + 1 new Rust CI workflow (PE) + 1 release-build subprocess job (PE) + 1 G6 invariant test mirror (PE) |
| ms-codec | — | — (no syscall layer; SPEC §1 names ms-cli only) | — | 0 |
| descriptor-mnemonic (md) | — | — (xpub-only material; Cycle A `OOS-md-mk` carries forward) | — | 0 |
| mnemonic-key (mk) | — | — (same as md) | — | 0 |

## §0.5 What this cycle does NOT close

Cycle B is bounded to the swap / coredump threat model. Six classes of residual exposure persist post-Cycle-B:

1. **Live-RAM disclosure (`ptrace`, `/proc/PID/mem`, kernel debugger).**
   `mlock(2)` only pins pages against swap and (on Linux, default `coredump_filter`) coredumps; it does NOT defend against a same-UID-or-root attacker reading process memory at runtime. SPEC §1 "Threat model NOT addressed" makes this explicit.

2. **Co-resident page-residue.** The slice-fn primitive `pin_pages_for(&[u8])` pins entire pages, so non-secret allocations co-resident on the same heap page are incidentally pinned. Cycle B accepts this; eliminating it requires a page-aligned-with-guard-pages allocator. FOLLOWUP: `dedicated-secret-arena` (Cycle C, toolkit; tier `v1+`).

3. **Windows `VirtualLock`.** Out-of-scope per SPEC §3 `OOS-windows-virtuallock` (Q4 brainstorming decision). Different semantics (no EPERM equivalent; soft-fail signals are `ERROR_NOT_ENOUGH_QUOTA` + working-set limits). Cycle B's `pin_pages_for` API is shape-compatible with a future `cfg(windows)` branch addition.

4. **Stack-resident short-lifetime secrets.** `mlock(2)` requires stable virtual addresses; stack regions remap on every function call. Cycle A survey §4 lines 206-210 lists ~5+ short-lifetime stack secrets beyond SPEC site 4 (`bip85`); none are in Cycle B's scope. Each requires a per-site heap-promote (Cycle B Phase 1 promoted `bip85::derive_entropy` only). SPEC §3 `OOS-secrets-on-stack`.

5. **Upstream-blocked Zeroize residue (carry-forward from Cycle A).** `secp256k1::SecretKey`, `bip39::Mnemonic` interior, `bitcoin::bip32::Xpriv` Copy-residue continue to leak post-mlock. Mlock-ing upstream-owned memory is fragile (upstream can reallocate without notice); SPEC §3 `OOS-upstream-zeroize-mlock` defers to the existing upstream FOLLOWUPS (`rust-secp256k1-secretkey-zeroize-upstream`, `rust-bip39-mnemonic-zeroize-upstream`, `rust-bitcoin-xpriv-zeroize-upstream`).

6. **`ResolvedSlot.entropy` + `DerivedAccount.entropy` field-type migration to `Zeroizing<Vec<u8>>` (deferred to v0.10.1).** Path B-lite carved this out (FOLLOWUP `resolved-slot-derived-account-zeroizing-field`, supersedes `resolved-slot-entropy-zeroizing-field`); the Cycle A baseline (`entropy: Option<Vec<u8>>` for `ResolvedSlot`, `entropy: Vec<u8>` + `impl Drop for DerivedAccount` for `DerivedAccount`) ships UNCHANGED in Cycle B. mlock-pinned during buffer lifetime; bytes-may-persist-on-heap-after-dealloc risk is unchanged from Cycle A for Site 2 (no Drop scrub under Cycle A baseline) — closes in v0.10.1. For Site 3 the Cycle A `impl Drop for DerivedAccount` scrub already fires before munlock, so the bytes-may-persist gap is already closed there. See §4.

## §1 SPEC §2 site coverage

Status legend:

- **SHIPPED**: site pin in place, residency test (in-source `#[cfg(test)]` or subprocess) asserts pin coverage.
- **DEFERRED-FIELD-MIGRATION**: site pin in place via sibling-field discipline, but the underlying `entropy:` field type stays at Cycle A baseline; closes in v0.10.1 patch via FOLLOWUP `resolved-slot-derived-account-zeroizing-field`.

### Site 1 (toolkit) — clap-args per-handler

| Handler | File | Anchor | Status | Evidence |
|---------|------|--------|--------|----------|
| `bundle` | `cmd/bundle.rs` | post-`apply_stdin_substitutions` re-binding | SHIPPED | per-field `pin_pages_for(p.as_bytes())` for the passphrase (`bundle.rs:129`) plus per-slot `pin_pages_for(s.value.as_bytes())` (`bundle.rs:133`), each immediately after `apply_stdin_substitutions()` returns (Phase 3a impl commit `c3d6ccd`); residency verified by `bundle.rs` `#[cfg(test)]` mod under `MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm` + `attempts_for_test()` mechanism. |
| `verify-bundle` | `cmd/verify_bundle.rs` | post-`apply_stdin_substitutions` re-binding | SHIPPED | same per-field pattern (`verify_bundle.rs:143-150`); per-handler in-source test. |
| `convert` | `cmd/convert.rs` | post-`effective_passphrase` / `effective_bip38_passphrase` / `primary_value` local-binding | SHIPPED | `convert.rs` has NO `apply_stdin_substitutions`; pin anchors are the local effective_*/primary_value bindings (Path B-lite §3.1 correction; SPEC §2 row 5). |
| `derive-child` | `cmd/derive_child.rs` | post-`from_value: Zeroizing<String>` + `stdin_passphrase: Option<Zeroizing<String>>` local-binding | SHIPPED | same — local-binding anchors (Path B-lite §3.1). |

### Site 2 (toolkit) — `ResolvedSlot` sibling-pin

| Element | Site | Status | Evidence |
|---------|------|--------|----------|
| `_entropy_pin: Option<Rc<PinnedPageRange>>` field | `synthesize.rs:604` | DEFERRED-FIELD-MIGRATION | declared AFTER `entropy: Option<Vec<u8>>` (struct-field RFC 1857 drop order: `entropy` drops first then `_entropy_pin` final-Rc-drops and munlocks). `Rc` (not `Arc`) preserves `derive(Clone)` for the cosigner-bridging clone sites at `cmd/bundle.rs`; commit `ddb371c` switched `Arc → Rc` after clippy `arc_with_non_send_sync` flagged `PinnedPageRange` as `!Send + !Sync`. Cycle A baseline `entropy` field type preserved; bytes-may-persist-after-dealloc gap closes in v0.10.1 per FOLLOWUP `resolved-slot-derived-account-zeroizing-field`. |
| 12 ctor sites populated (`pub type CosignerKeyInfo = ResolvedSlot;` alias counts as additional sites) | `synthesize.rs:{1059,1213}`, `parse_descriptor.rs:{1176,1741,1755}`, `cmd/bundle.rs:{371,441,475,518,1049,1099}`, `cmd/verify_bundle.rs:496` | SHIPPED | 6 sites populate `Some(Rc::new(pin_pages_for(...)))` (real entropy at construction time); 6 sites populate `None` (watch-only / partial-construction). The R0 v3-fold off-by-N (the `pub type CosignerKeyInfo` alias added 6 ctor sites missed at proposal time) is canonical recurring-pattern evidence — see memory `feedback_r0_must_read_source_off_by_n`. |

### Site 3 (toolkit) — `DerivedAccount` sibling-pin

| Element | Site | Status | Evidence |
|---------|------|--------|----------|
| `_entropy_pin: PinnedPageRange` field (plain, not Rc-wrapped — `DerivedAccount` is not Clone; consumed via `into_parts`) | `derive.rs:34` | SHIPPED | declared AFTER `entropy: Vec<u8>`. On Drop: `entropy` triggers Cycle A's `impl Drop for DerivedAccount` zeroize first (zeroize-while-still-pinned — strictest threat-model ordering), then `_entropy_pin` munlocks. The Cycle A baseline `entropy` field type AND `impl Drop for DerivedAccount` are PRESERVED unchanged under Path B-lite; the v0.10.1 migration (FOLLOWUP `resolved-slot-derived-account-zeroizing-field`) replaces `Vec<u8>` with `Zeroizing<Vec<u8>>` and deletes the manual `impl Drop`. |
| 1 ctor site populated | `derive_slot.rs:89` (inside `derive_bip32_from_entropy`) | SHIPPED | the singular `DerivedAccount` construction site. `into_parts()` body UNCHANGED. |

### Site 4 (toolkit) — bip85 `format_*` function-local pins

| Function | Site | Status | Evidence |
|----------|------|--------|----------|
| `format_bip39_phrase` | `bip85.rs:84` | SHIPPED | `let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);` immediately after `derive_entropy(...)?` binding. Local-binding drop order (Rust Reference §"destructors") reverses declaration: `_entropy_pin` munlocks first then `entropy: Zeroizing<Vec<u8>>` (Phase 1) zeroizes. Microsecond post-munlock-pre-zeroize window not load-bearing for the swap-out threat model. |
| `format_hd_seed_wif` | `bip85.rs:110` | SHIPPED | same pattern. |
| `format_xprv_child` | `bip85.rs:138` | SHIPPED | same pattern. |
| `format_hex_bytes` | `bip85.rs:170` | SHIPPED | same pattern. |
| `format_password_base64` | `bip85.rs:188` | SHIPPED | same pattern. |
| `format_password_base85` | `bip85.rs:203` | SHIPPED | same pattern. |
| `format_dice_rolls` | `bip85.rs:241` | SHIPPED | same pattern. (Phase 1 R0/R1 caught a "6 callees" → 7 callees off-by-one; `format_dice_rolls` was missed in original Phase 1 framing per SPEC §2 row 4 narrative.) |

### Site 5 (ms-cli) — `read_stdin` function-local pin

| Function | Site | Status | Evidence |
|----------|------|--------|----------|
| `parse::read_stdin` | `parse.rs:65` | SHIPPED | `let _entropy_pin = crate::mlock::pin_pages_for(buf.as_bytes());` post-receipt, scope-bound to `s`. Phase 3b inline-copy commit `87965b6` (mnemonic-secret). |

### `main()` wires (both binaries)

| Binary | File | Status | Evidence |
|--------|------|--------|----------|
| `mnemonic` (toolkit) | `main.rs:101` | SHIPPED | `mnemonic_toolkit::mlock::report_at_exit();` between `match result` close and `ExitCode` return; covers Ok + Err paths. Clap-parse-error early-return path at `main.rs:62` skipped per SPEC §3 `OOS-cross-process-aggregation`. |
| `ms` (ms-cli) | `main.rs:130` | SHIPPED | `mlock::report_at_exit();` before exit (Phase 3b). |

## §2 Cycle A → Cycle B carry-overs (closed out)

Cycle A matrix §4 (lines 247-269) named 5 mlock candidates. Disposition:

1. **Top-priority — `DerivedAccount.entropy` Vec + `derive_master_seed` seed.** `DerivedAccount.entropy` pinned via Site 3 sibling-field. The `derive_master_seed` `Zeroizing<[u8; 64]>` seed return is NOT independently pinned in Cycle B (the seed flows into downstream consumers via `derive_bip32_*` whose internal `Xpriv` buffers are upstream-blocked per `rust-bitcoin-xpriv-zeroize-upstream`); residual stack-bound seed exposure is in §0.5 class 4 (`OOS-secrets-on-stack`) and class 5 (Xpriv-Copy residue).
2. **Top-priority — `secp256k1::SecretKey` scalar bytes (bip85).** OUT-OF-SCOPE per SPEC §3 `OOS-upstream-zeroize-mlock` (Q1 brainstorming decision). Mlock-ing upstream-owned memory is fragile; FOLLOWUP `rust-secp256k1-secretkey-zeroize-upstream` is the canonical path.
3. **Top-priority — `bip39::Mnemonic` interior.** Same OOS class as #2; FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`.
4. **Lower-priority — `--passphrase-stdin` / `--bip38-passphrase-stdin` read buffer.** Pinned via Site 1 per-handler anchors (Site 1's per-handler pin covers the post-stdin-read `Zeroizing<String>` bindings).
5. **Lower-priority — `electrum::phrase_to_entropy` accumulator.** NOT pinned in Cycle B (function-local short-lifetime; Site 4 covers the bip85 family but the electrum family was not added to Path B-lite scope). Carry-forward residual; if user feedback demands, add to a future cycle.

## §3 SPEC §3 FOLLOWUPS forward-visibility list

SPEC §3 enumerates 8 OOS classes. Plus 1 cycle-surfaced entry (Path B-lite carve-out) opened during Phase 3a R0 v3-fold. All entries open in the respective repos' `design/FOLLOWUPS.md` files at PE close; the parent cycle entry `secret-memory-hygiene-cycle-b` closes at PE with reciprocal commit-SHA Companion line.

**SPEC §3 OOS entries (8):**

| FOLLOWUP id | SPEC §3 anchor | Tier | Repo | Status |
|-------------|---------------|------|------|--------|
| (NEW — Cycle C cycle) `dedicated-secret-arena` | `OOS-page-residue-elimination` + `OOS-secret-arena` (Cycle A carry-forward; Cycle B does not undertake allocator-level work) | future-cycle | toolkit | open |
| `rust-secp256k1-secretkey-zeroize-upstream` | `OOS-upstream-zeroize-mlock` (substitution alternative) | external | toolkit | open (carry-forward) |
| `rust-bip39-mnemonic-zeroize-upstream` | `OOS-upstream-zeroize-mlock` (substitution alternative) | external | toolkit | open (carry-forward) |
| (no FOLLOWUP) | `OOS-windows-virtuallock` | future-cycle | — | intentional final shape; revisit when POSIX abstraction settles |
| (no FOLLOWUP) | `OOS-secrets-on-stack` | future-cycle | — | per-site heap-promote required; Site 4 (bip85) done in Phase 1; others TBD |
| (no FOLLOWUP) | `OOS-capability-probe` | — | — | intentional final shape (try-and-soft-fail) |
| (no FOLLOWUP) | `OOS-cross-process-aggregation` | — | — | intentional (per-process MlockState) |
| (no FOLLOWUP) | `OOS-suppression-flag` | future-cycle | — | revisit if user feedback demands |
| (no FOLLOWUP) | `OOS-shared-mlock-crate` | future-cycle | — | "fork-and-document-pattern over shared-crate-extraction" per `mc-codex32-extraction-retired-2026-05-03` precedent |

**Cycle-surfaced entries (1) — not in SPEC §3 but opened during Phase 3a R0 v3-fold RESCOPE:**

| FOLLOWUP id | Surfaced | Tier | Repo | Status |
|-------------|----------|------|------|--------|
| `resolved-slot-derived-account-zeroizing-field` | Phase 3a R0 v3-fold RESCOPE (Path B-lite carve-out; supersedes `resolved-slot-entropy-zeroizing-field` from Cycle A) | v0.10.1-patch | toolkit | open |

**Cycle meta entry:** `secret-memory-hygiene-cycle-b` (cross-repo; closes at PE rollup with reciprocal commit-SHA Companion lines in toolkit + mnemonic-secret).

## §4 Path B-lite carve-out (v0.10.1)

Phase 3a R0 v3-fold RESCOPE (proposal `~/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`; reviewer reports `v0_9_B-phase-3a-rescope-r0{,-v3,-v3-fold}.md`) carved the following out of Cycle B Phase 3a, deferred to v0.10.1 patch via FOLLOWUP `resolved-slot-derived-account-zeroizing-field` (supersedes `resolved-slot-entropy-zeroizing-field`):

- `ResolvedSlot.entropy: Option<Vec<u8>>` → `Option<Zeroizing<Vec<u8>>>` migration
- `DerivedAccount.entropy: Vec<u8>` → `Zeroizing<Vec<u8>>` migration
- DELETE `impl Drop for DerivedAccount` (Zeroizing carries the scrub)
- `into_parts()` body change (Deref through Zeroizing)
- `tests/lint_zeroize_discipline.rs` row relabel + new ResolvedSlot row
- CHANGELOG migration entry (v0.10.1)

**Kept in Cycle B Phase 3a/3b (this matrix):** all struct-sibling pins on `ResolvedSlot` (Rc-wrapped) and `DerivedAccount` (plain), Site 4 bip85 pins, main.rs wires, CI release-build job. Threat-model coverage is equivalent to the R0 v2 LOCK (struct-sibling pin discipline is the same shape; only the underlying `entropy:` field type and the `impl Drop for DerivedAccount` element are deferred). For Site 2 the bytes-may-persist-after-dealloc gap remains until v0.10.1 closes it; for Site 3 the Cycle A `impl Drop` scrub already fires before munlock and that gap is already closed in Cycle B.

## §5 Cycle-close gates (SPEC §6)

All seven SPEC §6 gates satisfied at PE close:

1. **G1 — Functional correctness.** ✓ G1.1 (single-page), G1.2 (multi-page), G1.3 (zero-length no-op), G1.4 (page-aligned) all green in mlock.rs unit tests under both Linux (`/proc/self/smaps`) and macOS test runners. CI: ubuntu-latest + macos-latest matrix in toolkit's `.github/workflows/rust.yml` (Phase 2 + 3a); ms-cli adds the same matrix at PE via `mnemonic-secret/.github/workflows/rust.yml` (PE.T2).
2. **G2 — Soft-fail coverage.** ✓ G2.1 (eperm), G2.2 (enomem), G2.3 (einval — debug-assert + release soft-fail), G2.4 (off control), G2.5 (stderr summary). G2.3-release coverage shipped in Phase 3a via the dedicated `test-release-mlock-einval` CI job at toolkit `4a5335a` (Linux-only; the `cfg(test)` FAIL_MODE harness is platform-uniform so one platform is sufficient for the release-build-coverage gate). PE mirrors the release-build job in ms-cli's workflow.
3. **G3 — Platform coverage.** ✓ Ubuntu + macOS matrix green. `ulimit -l 65536` set in Linux jobs per SPEC §6 G3.
4. **G4 — Cycle A discipline preserved + Rust-level safety verified.**
   - G4.a (Zeroize-on-Drop preserved): ✓ Sites 2/3/4 preserve Cycle A's `impl Drop for DerivedAccount` (Site 3) and `Zeroizing<Vec<u8>>` (Site 4 entropy + Site 1 binding pins). Site 2's `ResolvedSlot.entropy` stays at Cycle A baseline (no Drop scrub); the bytes-may-persist-after-dealloc gap closes in v0.10.1 per §4. Cycle A's `tests/lint_zeroize_discipline.rs` ships UNCHANGED in Cycle B Phase 3a (relabel + new ResolvedSlot row deferred to v0.10.1).
   - G4.b (Miri on unsafe blocks): ✓ `cargo +nightly miri test -p mnemonic-toolkit mlock::` green in the `miri` CI job. The 2 `unsafe` blocks (`libc::mlock` in `pin_pages_for`; `libc::munlock` in `PinnedPageRange::drop`) carry SAFETY comments verified by `tests/lint_safety_first_party_mlock.rs`. PE adds the same Miri job to ms-cli's workflow.
5. **G5 — Cross-repo lockstep.** ✓ `mnemonic-toolkit-v0.10.0` + `ms-cli-v0.3.0` push within the same PE session. CHANGELOG entries in both repos cross-cite each other's tag commit SHAs. `secret-memory-hygiene-cycle-b` parent FOLLOWUP resolves with reciprocal `Companion:` lines in both repos.
6. **G6 — Inline-copy equivalence (diff manifest + name-export check).** ✓ Workspace-level integration test `tests/mlock_g6_invariant.rs` in BOTH repos. Manifest (14 top-level items, asserted as `const MANIFEST: &[&str]` in the test source): `{MLOCK_STATE, MlockState, PinnedPageRange, attempts_for_test, errno_to_name, failure_count_for_test, first_errno_for_test, last_os_errno, mlock_state, page_size, page_size_for_test, pin_pages_for, report_at_exit, round_to_pages}`. The nested `cfg(test) mod fail_mode` (containing `FailMode` + `FailMode::parse` + `FailMode::current`) is column-indented and the test's column-zero `extract_top_level_names` filter excludes it — its drift is caught instead by the normalized-source byte-equality check (test 1), not by the name-export manifest check (test 2). Normalization per SPEC §6 G6: strip `//`, `///`, `//!` comment lines at start-of-trimmed-line; preserve `use` statements + `#[cfg]` attrs + internal string-literal whitespace. Both repos' rust.yml workflows check out the OTHER repo at the matching tag (or `master` pre-tag) before running the test.
7. **G7 — No wire-format regression.** ✓ v0.1 + v0.2 fixture-corpus SHA pins continue to hold post-Cycle-B. Mlock is functionally transparent (a soft-failed mlock does not change any output). Toolkit's existing fixture tests cover the regression surface; no new wire-format pin needed.

PE (release rollup) is the final cycle-close step. PE deliverables: this matrix doc + ms-cli Rust CI workflow (PE.T2) + G6 invariant test (PE.T3) + SPEC §2 row 5 line-number drift fix (PE.T7) + version bumps (PE.T4) + FOLLOWUPS closures (PE.T5) + CHANGELOG entries (PE.T6) + lockstep tag push `mnemonic-toolkit-v0.10.0` + `ms-cli-v0.3.0`.
