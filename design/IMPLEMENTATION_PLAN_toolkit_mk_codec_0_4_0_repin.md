# Toolkit re-pin to mk-codec 0.4.0 (no-path support) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans. Steps use checkbox (`- [ ]`).

**Goal:** Re-pin `mnemonic-toolkit` to the published `mk-codec 0.4.0` so `bundle --slot wif=…` produces a depth-0 / no-path mk1 card that **decodes** (the round-trip the 0.3.1 pin silently broke), fix the two internally-inconsistent verify_bundle test fixtures the 0.4.0 encode-guard now correctly rejects, harden the two toolkit error-mirrors with explicit `XpubOriginPathMismatch` arms, and add a WIF→decode round-trip regression.

**Architecture:** Pure re-pin + test/error-handling cycle. No new CLI flags / subcommands / output-shape → **no GUI schema-mirror, no manual lockstep**. Binary-private behavior fix → **PATCH** (0.37.9 → 0.37.10).

**Empirical grounding (branch `toolkit-mk-codec-0.4.0-repin`, re-pin applied):** `cargo build` clean; full suite = **834 passed / 2 failed**, the 2 failures being `verify_bundle.rs` `helper_multisig_full_emits_3plus6n_checks_in_spec_order` (:2691) and `helper_multisig_missing_ms1_emits_passed_false_per_spec_5_7_case_4` (:2767). `bundle_slot_wif_stdin_succeeds` now PASSES (the WIF case works on 0.4.0). The two error-mirror files compile unchanged (both have `_ =>` fallbacks).

**Source SHAs:** toolkit base `master` `a255060`; consumes published `mk-codec = "0.4.0"` (crates.io).

---

## Task 1 — Re-pin + lockfile (DONE on branch; verify)

**Files:** `crates/mnemonic-toolkit/Cargo.toml:21`, `Cargo.lock`.

- [x] `mk-codec = "0.3.1"` → `mk-codec = "0.4.0"`; `cargo update -p mk-codec --precise 0.4.0` (lockfile `0.3.1`→`0.4.0`). `cargo build -p mnemonic-toolkit` clean.
- [ ] **Commit (after Task 2 makes the suite green — do not commit a red re-pin).**

## Task 2 — Fix the two internally-inconsistent verify_bundle fixtures

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (helpers at ~`:2643` and ~`:2725`).

**Root cause:** both helpers derive `xpub_a`/`xpub_b` at `m/48'/0'/0'/2'` (depth 4) but call `synthesize_full(&entropy, fp, xpub, CliTemplate::Bip84, …)`. Inside `synthesize_full` (`synthesize.rs:136-139`) the path is `template.derivation_path` = `m/84'/0'/0'` (depth 3); the resulting KeyCard pairs a depth-4 xpub with a depth-3 path → `mk-codec 0.4.0` `XpubOriginPathMismatch`. The fixture is genuinely inconsistent; the guard is correct. The helpers use ONLY `bundle.ms1[0]` (an `ms_codec::encode(ENTR, entropy)` string — **independent of the xpub/path**), so deriving a bip84-consistent xpub for these two calls leaves the asserted output byte-identical.

- [ ] **Step 1 — Derive bip84-consistent xpubs in `helper_multisig_full_…`.** After the `fp_b`/`xpub_b` block (before the `cosigners` vec), add:

```rust
        // The synthesize_full calls below build an mk1 card internally; mk-codec
        // 0.4.0 enforces xpub.depth == path depth, so the xpub must be derived at
        // the Bip84 path (m/84'/0'/0', depth 3) the template uses — NOT the depth-4
        // multisig path above. ms1 (the only output this test reads) is entropy-only,
        // so the derived xpub does not affect the assertions.
        let path84 = DerivationPath::from_str("m/84'/0'/0'").unwrap();
        let xpub84_a = Xpub::from_priv(&secp, &master_a.derive_priv(&secp, &path84).unwrap());
        let xpub84_b = Xpub::from_priv(&secp, &master_b.derive_priv(&secp, &path84).unwrap());
```

Then change the two `synthesize_full` calls to pass `xpub84_a` / `xpub84_b`:
```rust
        let bundle_a = synthesize_full(
            &entropy_a, fp_a, xpub84_a, CliTemplate::Bip84, CliNetwork::Mainnet, 0,
        )
        .unwrap();
        let bundle_b = synthesize_full(
            &entropy_b, fp_b, xpub84_b, CliTemplate::Bip84, CliNetwork::Mainnet, 0,
        )
        .unwrap();
```

- [ ] **Step 2 — Apply the identical fix to `helper_multisig_missing_ms1_…`** (same `path84`/`xpub84_a`/`xpub84_b` derivation + `synthesize_full(... xpub84_a ...)` / `xpub84_b`).

- [ ] **Step 3 — Run the two helpers.** `cargo test -p mnemonic-toolkit --lib 'cmd::verify_bundle::helper_tests::helper_multisig' ` → both PASS.

## Task 3 — Explicit `XpubOriginPathMismatch` arms in both error-mirrors

**Files:** `crates/mnemonic-toolkit/src/friendly.rs` (~`:123`), `crates/mnemonic-toolkit/src/error.rs::mk_codec_exit_code` (~`:391`).

Both files have a `_ =>` fallback (mk_codec::Error is `#[non_exhaustive]`, so the catch-all stays), but an explicit arm gives a real message + correct exit code instead of "unhandled mk_codec::Error variant".

- [ ] **Step 1 — `friendly.rs`** — add before the `_ =>` fallback (`:123`):
```rust
        E::XpubOriginPathMismatch {
            xpub_depth,
            path_depth,
            ..
        } => format!(
            "mk1 xpub/origin-path depth-child mismatch: xpub depth {} vs origin_path depth {} (encoder-side invariant; the xpub's depth/child must agree with the path, or be depth-0 with no path)",
            xpub_depth, path_depth,
        ),
```

- [ ] **Step 2 — `error.rs::mk_codec_exit_code`** — add before the `_ => 1` fallback (`:392`):
```rust
        mk_codec::Error::XpubOriginPathMismatch { .. } => 2,
```
(Exit 2 = the structural/encode-error class, matching the other bytecode-layer variants in this match.)

- [ ] **Step 3 — Extend `friendly.rs`'s existing mk-codec test** (the one at ~`:302` asserting `PathTooDeep` friendliness): add an assertion that `friendly_mk_codec(&mk_codec::Error::XpubOriginPathMismatch { xpub_depth: 4, path_depth: 3, xpub_child: ChildNumber::Hardened { index: 1 }, path_child: None })` contains `"depth-child mismatch"` and does NOT contain `"unhandled"`. (Import `ChildNumber` in the test.)

- [ ] **Step 4 — Run.** `cargo test -p mnemonic-toolkit --lib friendly` → PASS.

## Task 4 — WIF → decode round-trip regression

**Files:** Modify `crates/mnemonic-toolkit/tests/cli_argv_leakage.rs` (sibling to `bundle_slot_wif_stdin_succeeds` at `:99`) OR a dedicated `tests/cli_bundle_wif_roundtrip.rs`.

The 0.3.1 bug: `bundle --wif` emitted an mk1 card that `mk_codec::decode` rejected (`PathTooDeep(0)`). The regression must DECODE the emitted card. `mnemonic inspect --mk1 <chunk…>` decodes via `mk_codec::decode` (inspect.rs:178).

- [ ] **Step 1 — Write `bundle_wif_mk1_round_trips_via_inspect`:**
  - Run `bundle --slot @0.wif=- --network mainnet --template bip84 --no-engraving-card --json`, stdin = `MAINNET_WIF`, capture stdout.
  - Parse stdout as JSON (`serde_json`); extract the mk1 card chunk strings (pin the exact field at impl time — inspect `bundle --json` shape; likely `.mk1` as a string or array).
  - Run `inspect --mk1 <chunk1> [--mk1 <chunk2> …]`, assert `.success()` and stdout mentions the depth-0 / no-path card (at minimum: decode does not error). This FAILS on 0.3.1 (decode → `PathTooDeep(0)`), PASSES on 0.4.0.
  - If the `--json` mk1 extraction proves fiddly, fall back to `bundle … --json` → write the mk1 chunk(s) to a temp, then `inspect --mk1 -` via stdin.

- [ ] **Step 2 — Run** → PASS on the 0.4.0 branch.

## Task 5 — FOLLOWUPs

**Files:** `crates/mnemonic-toolkit/design/FOLLOWUPS.md` (or wherever the toolkit registry lives — `git ls-files | grep -i followups`).

- [ ] **Step 1 — File `mk1-wif-bundle-depth0-invalid-card`** as `Status: resolved <Task-2 SHA>`: "bundle --slot wif builds a depth-0/empty-path mk1 card; mk-codec 0.4.0 makes it round-trip (decode accepts count==0, reconstruct → depth-0/Normal{0}, guard accepts the consistent depth-0 card). Toolkit re-pinned 0.3.1→0.4.0; 2 inconsistent verify_bundle fixtures fixed; error-mirrors + WIF round-trip regression added." Companion: `mnemonic-key` `mk1-no-path-depth0-support`.
- [ ] **Step 2 — Update `mk1-depth-child-compensating-check-watch`** (`design/FOLLOWUPS.md:3335`): note the mk-codec precondition is now FULLY met (option (a) shipped 0.3.2 + extended to depth-0 in 0.4.0); the `synthesize.rs:494-503` check is **kept as defense-in-depth** (friendlier per-cosigner error message than the codec's `XpubOriginPathMismatch`). Keep `Status: open` (monitoring) with the updated note — do NOT remove the code this cycle (behavior/error-message change is out of scope).

## Task 6 — Version bump

**Files:** `crates/mnemonic-toolkit/Cargo.toml:3`.

- [ ] `version = "0.37.9"` → `"0.37.10"`; `cargo build -p mnemonic-toolkit` (Cargo.lock self-version updates); commit with the lockfile.

## Task 7 — Full-suite gate + end-of-cycle R0 + ship

- [ ] **Step 1 — FULL suite (the reverted-re-pin lesson — NEVER skip this).** `cargo test -p mnemonic-toolkit` → all green (was 834+2; now 836+/0). Plus `cargo test --workspace` if other crates consume the toolkit. `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`. `cargo +stable fmt -p mnemonic-toolkit -- --check` (authoritative; edition 2024). Note pre-existing clippy drift (bip85.rs:289 manual_div_ceil, bundle.rs:1822/1862, convert.rs:903/942, cli_bundle_seedqr_slot.rs:5) is SEPARATE — if `-D warnings` trips on it, scope clippy to the touched files or confirm it pre-dates this branch (`git stash` + re-run) and leave it for its own hygiene cycle.
- [ ] **Step 2 — End-of-cycle opus R0** over the full branch diff → persist to `design/agent-reports/`. Fold to GREEN (0C/0I); re-dispatch after any fold.
- [ ] **Step 3 — Clean-tree check** (`git status --porcelain` empty), then ff-merge `master` + push + tag `mnemonic-toolkit-v0.37.10`.

---

## Self-review
- **Coverage:** re-pin (T1) + the 2 empirically-confirmed fixture failures (T2) + error-mirror hygiene (T3) + the WIF round-trip the 0.3.1 pin broke (T4) + FOLLOWUPs (T5) + version (T6) + the full-suite gate that the reverted re-pin skipped (T7).
- **Placeholders:** `<Task-2 SHA>` filled at T5. The T4 JSON-field name is pinned at impl time (flagged).
- **No GUI/manual lockstep** (no CLI surface change); **no new mk_codec variant** (mirrors are hygiene, fallback-safe).
