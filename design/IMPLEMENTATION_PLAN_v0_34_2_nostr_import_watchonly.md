# `mnemonic nostr --import` (read-only importdescriptors) Implementation Plan — v0.34.2

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `mnemonic nostr --import readonly [--timestamp <now|UNIX>]` that appends a ready-to-paste Bitcoin Core **read-only** `importdescriptors` recipe (one array, one entry per requested script type) built from the watch-only descriptor(s).

**Architecture:** Generalize the existing `wallet_export::bitcoin_core` importdescriptors emitter with a non-ranged single-key array builder (one JSON shape for both `export-wallet` HD and `nostr` raw-key), re-export it `pub(crate)`, reuse `export-wallet`'s `--timestamp` parser, and emit from `nostr`'s `run`. Watch-only only — spending is deferred (FOLLOWUP).

**Tech Stack:** Rust; `serde_json`; existing `wallet_export::{bitcoin_core::format_bitcoin_core_importdescriptors, TimestampArg}` (`TimestampArg` is `pub(crate)` + `Copy`; `to_json` callable within `wallet_export`); `cmd::export_wallet::{parse_timestamp, TimestampArgValue}`; `nostr::descriptor_for`. Spec: `design/BRAINSTORM_v0_34_2_nostr_import_watchonly.md`. **Source baseline:** branch `v0.34.2-toolkit-hygiene` tip `563d86e` (code == `origin/master` `1d6436d`; deltas are docs-only).

**SemVer:** PATCH → **v0.34.2** (additive flags). **Mandatory lockstep:** GUI `schema_mirror` (new `--import`/`--timestamp` flag NAMEs on `nostr`) + manual `41-mnemonic.md`.

---

## File structure
- **Modify** `crates/mnemonic-toolkit/src/wallet_export/bitcoin_core.rs` — add `import_array_single` (non-ranged single-key array builder).
- **Modify** `crates/mnemonic-toolkit/src/wallet_export/mod.rs` — `pub(crate) use bitcoin_core::import_array_single;` (bitcoin_core is a private submodule; re-export the one fn).
- **Modify** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` — `parse_timestamp` → `pub(crate)`.
- **Modify** `crates/mnemonic-toolkit/src/cmd/nostr.rs` — `--import`/`--timestamp` flags, `ImportMode` + `parse_import_mode`, `NostrJson.import` field, emission in both `run` paths.
- **Modify** `crates/mnemonic-toolkit/tests/cli_nostr.rs` — integration cells.
- **Modify** `docs/manual/src/40-cli-reference/41-mnemonic.md`, `Cargo.toml`, `scripts/install.sh`, `CHANGELOG.md`, `design/FOLLOWUPS.md` (Task 3).
- **Cross-repo (paired):** `mnemonic-gui/src/schema/mnemonic.rs` + pin (Task 4).

---

## Task 1: shared single-key import builder + `parse_timestamp` reuse

**Files:** Modify `wallet_export/bitcoin_core.rs`, `wallet_export/mod.rs`, `cmd/export_wallet.rs`

- [ ] **Step 1: Write the failing unit test** — append to `wallet_export/bitcoin_core.rs` (add a `#[cfg(test)] mod` if none):
```rust
#[cfg(test)]
mod import_single_tests {
    use super::*;
    use crate::wallet_export::TimestampArg;

    #[test]
    fn single_key_array_is_nonranged_watchonly() {
        let descs = vec![
            "wpkh(027e7e9c42…)#aaaaaaaa".to_string(),
            "tr(7e7e9c42…)#bbbbbbbb".to_string(),
        ];
        let v = import_array_single(&descs, TimestampArg::Unix(0));
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 2);
        for (i, e) in arr.iter().enumerate() {
            assert_eq!(e["desc"], descs[i]);
            assert_eq!(e["active"], false);
            assert_eq!(e["internal"], false);
            assert_eq!(e["timestamp"], 0);
            assert!(e.get("range").is_none(), "single-key entry must omit range");
        }
    }
}
```
(Use short fake descriptors — the builder doesn't parse them, unlike the ranged path. The real descriptors are tested end-to-end in Task 2.)

- [ ] **Step 2: Run, verify FAIL** — `cargo test -p mnemonic-toolkit --bin mnemonic import_single_tests` → FAIL (`import_array_single` not found).

- [ ] **Step 3: Implement** — add to `wallet_export/bitcoin_core.rs` (mirrors the existing `format_bitcoin_core_importdescriptors`'s `json!`/`to_json` usage, which already compiles here):
```rust
/// Build an `importdescriptors` array of NON-ranged, single-key, watch-only
/// entries (one per descriptor). Unlike `format_bitcoin_core_importdescriptors`
/// (HD/ranged), each entry omits `range` and is `active:false`/`internal:false`
/// — a single watched address. Used by `mnemonic nostr --import readonly`.
pub(crate) fn import_array_single(descs: &[String], timestamp: TimestampArg) -> Value {
    Value::Array(
        descs
            .iter()
            .map(|desc| {
                json!({
                    "desc": desc,
                    "active": false,
                    "internal": false,
                    "timestamp": timestamp.to_json(),
                })
            })
            .collect(),
    )
}
```
(`TimestampArg` is `Copy`, so `timestamp.to_json()` per entry is fine.)

- [ ] **Step 4: Re-export** — in `wallet_export/mod.rs`, near the other `pub(crate) use` lines, add:
```rust
pub(crate) use bitcoin_core::import_array_single;
```

- [ ] **Step 5: Make `parse_timestamp` reusable** — in `cmd/export_wallet.rs:216`, change `fn parse_timestamp(` to `pub(crate) fn parse_timestamp(` (returns `Result<TimestampArgValue, String>`; `TimestampArgValue(pub TimestampArg)` is already `pub`).

- [ ] **Step 6: Run + no-regression** — `cargo test -p mnemonic-toolkit --bin mnemonic import_single_tests` → PASS. `cargo test -p mnemonic-toolkit --test cli_export_wallet*` (export-wallet's bitcoin-core tests) → unchanged (the ranged path + `format_bitcoin_core_importdescriptors` are untouched). `cargo build -p mnemonic-toolkit` clean.

- [ ] **Step 7: Commit**
```bash
git add crates/mnemonic-toolkit/src/wallet_export/bitcoin_core.rs crates/mnemonic-toolkit/src/wallet_export/mod.rs crates/mnemonic-toolkit/src/cmd/export_wallet.rs
git commit -m "feat(wallet-export): non-ranged single-key importdescriptors builder + pub(crate) parse_timestamp" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: `nostr --import` / `--timestamp` flags + emission

**Files:** Modify `cmd/nostr.rs`, `tests/cli_nostr.rs`

- [ ] **Step 1: Write failing integration tests** — append to `tests/cli_nostr.rs`:
```rust
#[test]
fn import_readonly_emits_watchonly_recipe() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--script-type", "p2wpkh", "--import", "readonly"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("import:      importdescriptors '["), "got: {s}");
    // The single-quoted JSON parses + is a non-ranged watch-only entry.
    let json_str = s.split("importdescriptors '").nth(1).unwrap().split("'\n").next().unwrap();
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert_eq!(v[0]["active"], false);
    assert_eq!(v[0]["timestamp"], 0); // default
    assert!(v[0]["desc"].as_str().unwrap().starts_with("wpkh("));
    assert!(v[0].get("range").is_none());
}

#[test]
fn import_all_script_types_one_array_four_entries() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--all-script-types", "--import", "readonly"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let json_str = s.split("importdescriptors '").nth(1).unwrap().split("'\n").next().unwrap();
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 4);
}

#[test]
fn import_spending_and_both_are_refused() {
    for mode in ["spending", "both"] {
        Command::cargo_bin("mnemonic").unwrap()
            .args(["nostr", "--pubkey", NPUB, "--import", mode])
            .assert().failure().stderr(predicate::str::contains("deferred to a future cycle"));
    }
}

#[test]
fn import_timestamp_flag_overrides_default() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--script-type", "p2tr", "--import", "readonly", "--timestamp", "now"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    let json_str = s.split("importdescriptors '").nth(1).unwrap().split("'\n").next().unwrap();
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert_eq!(v[0]["timestamp"], "now");
}

#[test]
fn no_import_flag_emits_no_recipe() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB])
        .assert().success().stdout(predicate::str::contains("import:").not());
}

#[test]
fn import_in_json_envelope() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--import", "readonly", "--json"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(v["import"].is_array());
    assert_eq!(v["import"][0]["active"], false);
}
```

- [ ] **Step 2: Run, verify FAIL** — `cargo test -p mnemonic-toolkit --test cli_nostr import_` → FAIL (flags/output absent).

- [ ] **Step 3: Add the `ImportMode` enum + parser** — in `cmd/nostr.rs` (top-level):
```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImportMode {
    ReadOnly,
}

/// `--import` value parser. Only `readonly` is supported in v0.34.2; `spending`
/// and `both` are reserved (forward-compatible) and rejected with a clear note.
fn parse_import_mode(s: &str) -> Result<ImportMode, String> {
    match s {
        "readonly" => Ok(ImportMode::ReadOnly),
        "spending" | "both" => Err(
            "--import: 'spending'/'both' is deferred to a future cycle; only 'readonly' is supported".into(),
        ),
        other => Err(format!("--import must be 'readonly'; got {other:?}")),
    }
}
```

- [ ] **Step 4: Add the flags to `NostrArgs`** (after the `--json` field):
```rust
    /// Emit a ready-to-paste Bitcoin Core `importdescriptors` recipe for the
    /// derived address(es). `readonly` = watch-only (the pubkey descriptor).
    /// `spending`/`both` are reserved (future cycle).
    #[arg(long, value_parser = parse_import_mode)]
    pub import: Option<ImportMode>,

    /// Bitcoin Core `importdescriptors` rescan anchor: `now` or unix seconds.
    /// Default `0` (rescan from genesis to discover an existing key's funds).
    /// Only used with `--import`.
    #[arg(long, value_parser = crate::cmd::export_wallet::parse_timestamp, default_value = "0")]
    pub timestamp: crate::cmd::export_wallet::TimestampArgValue,
```
(Confirm `TimestampArgValue` is importable — it is `pub struct` in `cmd::export_wallet`. If clap needs it to impl `Clone`, it derives it already as the export-wallet `--timestamp` uses the same parser; verify and add `#[derive(Clone)]` on `TimestampArgValue` if the compiler asks.)

- [ ] **Step 5: Add the `import` field to `NostrJson`** (`cmd/nostr.rs:69`):
```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    import: Option<serde_json::Value>,
```

- [ ] **Step 6: Emit in both `run` paths.** After the `rows` vec is built and before each render branch (the pubkey path ~`:96` and the secret path ~`:155`), compute the recipe; thread it into the JSON envelope and the text block. Concretely, in EACH path:
```rust
        // Build the read-only importdescriptors recipe (once per path).
        let import_recipe: Option<serde_json::Value> = if args.import == Some(ImportMode::ReadOnly) {
            let descs: Vec<String> = rows.iter().map(|r| r.descriptor.clone()).collect();
            Some(crate::wallet_export::import_array_single(&descs, args.timestamp.0))
        } else {
            None
        };
```
Then:
- In the `if args.json { let envelope = NostrJson { … } }` construction, add `import: import_recipe.clone(),`.
- In the human-readable `else` branch, AFTER the per-row loop, add:
```rust
            if let Some(recipe) = &import_recipe {
                let line = serde_json::to_string(recipe).map_err(|e| {
                    ToolkitError::BadInput(format!("nostr: import recipe serialize: {e}"))
                })?;
                writeln!(stdout, "  import:      importdescriptors '{line}'").map_err(ToolkitError::Io)?;
            }
```
(`crate::wallet_export::import_array_single` resolves — `wallet_export` is a binary-crate module re-exporting the fn `pub(crate)`. `args.timestamp.0` is the inner `TimestampArg`.)

- [ ] **Step 7: Run, verify PASS** — `cargo test -p mnemonic-toolkit --test cli_nostr` → ALL green (the 6 new cells + all prior cli_nostr cells unchanged). `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

- [ ] **Step 8: Commit**
```bash
git add crates/mnemonic-toolkit/src/cmd/nostr.rs crates/mnemonic-toolkit/tests/cli_nostr.rs
git commit -m "feat(nostr): --import readonly + --timestamp (Bitcoin Core watch-only importdescriptors)" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: manual + FOLLOWUPs + version + regression

**Files:** Modify `docs/manual/src/40-cli-reference/41-mnemonic.md`, `design/FOLLOWUPS.md`, `Cargo.toml`, `scripts/install.sh`, `CHANGELOG.md`

- [ ] **Step 1: Manual** — in the `nostr` section of `41-mnemonic.md`, document `--import readonly` (+ reserved spending/both) and `--timestamp <now|unix>` (default `0`), the `import:` output line, and a worked paste-into-Core example. Mirror `--help`. Run the manual lint (`make -C docs/manual lint MNEMONIC_BIN=…` per the v0.34.0 cycle); flag-coverage must pass for the two new flags.

- [ ] **Step 2: File the 3 FOLLOWUPs** in `design/FOLLOWUPS.md` (alphabetical/append per convention):
  - `nostr-import-spending-descriptors` (`v0.34+`, `wallet`): deferred spending importdescriptors on `nostr` (nsec → `wpkh(<WIF>)`/`tr(<WIF>)`) + secret-on-stdout handling; enables `--import=spending|both`. Companion: `convert` spending import.
  - `export-wallet-timestamp-default-zero` (`v0.34+`): change `export_wallet.rs:117` `--timestamp` default `"now"` → `0` for consistency with `nostr`; a behavior change to the emitted recipe — own SemVer call.
  - `timestamp-zero-default-docs-sweep` (`v0.34+`): update all docs implying `--timestamp` defaults to `now` once the above lands.

- [ ] **Step 3: Version + install.sh + CHANGELOG.** `Cargo.toml:3` → `version = "0.34.2"`. `scripts/install.sh:32` self-pin `mnemonic-toolkit-v0.34.1` → `-v0.34.2` (the install-pin-check lesson). CHANGELOG `[0.34.2]` entry: new `mnemonic nostr --import readonly` + `--timestamp` (default 0) emitting a Bitcoin Core watch-only importdescriptors recipe; shared single-key emitter; also closes the 4 stale FOLLOWUPs (the `A` hygiene work on this branch). SemVer PATCH.

- [ ] **Step 4: Full regression.** `cargo test -p mnemonic-toolkit` → all green. `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.

- [ ] **Step 5: Commit**
```bash
git add docs/manual/src/40-cli-reference/41-mnemonic.md design/FOLLOWUPS.md crates/mnemonic-toolkit/Cargo.toml scripts/install.sh CHANGELOG.md
git commit -m "release(toolkit): mnemonic-toolkit v0.34.2 — nostr --import (read-only importdescriptors) + hygiene closes" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: paired GUI schema-mirror + ship (outward-facing)

- [ ] **Step 1:** In `mnemonic-gui`, add `--import` (text/dropdown) + `--timestamp` (text) to the `nostr` `SubcommandSchema` (`src/schema/mnemonic.rs`); neither is secret (no `flag_is_secret` change). Bump the toolkit pin → `v0.34.2`. Run `cargo test -p mnemonic-gui schema_mirror` (with `MNEMONIC_BIN` = the v0.34.2 build) → green. Bump GUI version (PATCH) + CHANGELOG. Commit on a paired branch.
- [ ] **Step 2 (controller/user-authorized):** merge toolkit → master, push, tag `mnemonic-toolkit-v0.34.2` (AFTER the install.sh-bumped commit), GH release; then the paired GUI release.

---

## Self-review (writing-plans checklist)

**1. Spec coverage:** §1 flags (`--import readonly` value-valued + reserved spending/both; `--timestamp` default 0) → Task 2 Steps 3-4. §2 behavior (watch-only descriptor; all-script-types one array; entry shape active:false/internal:false/no-range) → Task 1 builder + Task 2 Step 6 + tests. §3 shared helper → Task 1. §4 output (text line + json field; no new secret) → Task 2 Steps 5-6. §5 SemVer/lockstep → Tasks 3-4. §6 tests → Task 2 Step 1 (6 cells). §7 FOLLOWUPs → Task 3 Step 2.

**2. Placeholder scan:** none. The `…` inside fake descriptors in Task 1's test are intentional opaque test strings (the builder doesn't parse them); real descriptors are exercised in Task 2.

**3. Type consistency:** `import_array_single(&[String], TimestampArg) -> Value` used identically in Task 1 (def) + Task 2 Step 6 (call). `ImportMode::ReadOnly` + `parse_import_mode` consistent. `args.timestamp.0` = inner `TimestampArg` (`TimestampArgValue(pub TimestampArg)`). `NostrJson.import: Option<Value>` matches the `import_recipe.clone()` assignment. The text line label `  import:      ` matches the test's `import:      importdescriptors '[` assertion.

**Open items for execution:** (a) confirm `TimestampArgValue` derives `Clone` for clap (add if the compiler asks); (b) the two `run` render paths duplicate the import-emission insertion (matches the existing pubkey/secret duplication — do not refactor beyond inserting in both); (c) `--timestamp` without `--import` is silently inert (acceptable; no warning).

## Per-cycle reviewer-loop (CLAUDE.md / mandatory standard)
Dispatch opus R0 on this plan-doc before any implementation; persist to `design/agent-reports/v0_34_2-plan-r0-review.md`; converge to 0 Critical / 0 Important; re-dispatch after folds. End-of-cycle opus review before tagging.
