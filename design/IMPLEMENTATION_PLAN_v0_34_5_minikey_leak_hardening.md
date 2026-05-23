# v0.34.5 MiniKey-Leak Hardening — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development / executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Close the MiniKey stdout leak in `convert` (the Casascius mini-key input echoed unredacted in `--json` `from_value`) by switching the two stdout-redaction call sites to the wider `is_argv_secret_bearing` predicate; and promote `pub const SECRET_NODE_TYPES_ARGV` as the public mirror of that wide set (+ parity test).

**Architecture:** Two call-site predicate swaps in `convert.rs` + one new public const + two parity tests + one new integration cell. No new flag, no new behavior path. TDD: the new minikey-JSON cell is RED until the `:1042` swap lands.

**Tech Stack:** Rust; `crates/mnemonic-toolkit/src/cmd/convert.rs`; `crates/mnemonic-toolkit/src/secret_taxonomy.rs`; `crates/mnemonic-toolkit/tests/cli_convert_minikey.rs`.

**Source SHA:** `b17444b` (v0.34.4 tip). **SemVer:** PATCH (`v0.34.4 → v0.34.5`). Toolkit-only, **no GUI/manual lockstep** (a *new* `SECRET_NODE_TYPES_ARGV` const does not change the GUI's existing `SECRET_NODE_TYPES`/`SECRET_SLOT_SUBKEYS` snapshot).

**Approved design:** presented + user-approved 2026-05-22 ("Both" — switch both call sites). Recon: `design/cycle-prep-recon-batch-4features.md`.

**Verified behavior facts:**
- `is_secret_bearing()` (`convert.rs:94`) excludes MiniKey; `is_argv_secret_bearing()` (`convert.rs:117-119`) = narrow + MiniKey.
- `:1042` `from_value` redaction (JSON branch only) uses the narrow predicate → minikey input echoed unredacted in `--json`. **This is the leak.**
- `:1069` secret-on-stdout warning checks *outputs*; MiniKey is one-way (only edge `MiniKey→Wif`) so MiniKey is never an output → switching `:1069` is a **no-op today** (the WIF output already trips it) but keeps both pathways on one predicate.
- `SECRET_NODE_TYPES` (`secret_taxonomy.rs:76-85`) = `["phrase","entropy","xprv","wif","ms1","bip38","electrum-phrase","seedqr"]` (8). The wide set adds `"minikey"` (9).
- Narrow parity test `secret_taxonomy_parity_with_is_secret_bearing` at `convert.rs:1733` iterates `ALL_NODE_TYPE_VARIANTS`, asserts `v.is_secret_bearing() == SECRET_NODE_TYPES.contains(&v.as_str())`.
- No existing minikey test uses `--json` → no existing test breaks; `serde_json` is available to integration tests (cli_nostr.rs precedent).

---

## Task 1: Redact MiniKey input in JSON `from_value` (TDD)

**Files:** `tests/cli_convert_minikey.rs` (add 1 cell); `src/cmd/convert.rs` (swap 2 predicates).

- [ ] **Step 1: Add the failing test** (append to `tests/cli_convert_minikey.rs`):

```rust
#[test]
fn minikey_input_redacted_in_json_from_value() {
    // v0.34.5: --from minikey= is a private-key carrier; its echoed
    // `from_value` in --json output MUST be redacted (None), per the wider
    // is_argv_secret_bearing predicate. Closes `convert-minikey-stdout-redaction`.
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["convert", "--from", &format!("minikey={VEC22_KEY}"), "--to", "wif", "--json"])
        .assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid convert JSON");
    assert_eq!(v["from_node"], "minikey");
    assert!(
        v["from_value"].is_null(),
        "minikey from_value must be redacted in --json; got: {}", v["from_value"]
    );
    // The decoded WIF output is itself secret-bearing and still appears in `to`.
    assert!(v["to"][0]["value"].as_str().unwrap().starts_with('5'));
    // The minikey private key must NOT leak anywhere in the JSON.
    assert!(!stdout.contains(VEC22_KEY), "minikey input leaked into JSON: {stdout}");
}
```

- [ ] **Step 2: Run — verify RED.**

Run: `cargo test -p mnemonic-toolkit --test cli_convert_minikey minikey_input_redacted_in_json_from_value`
Expected: FAIL (`from_value` currently echoes the minikey, so `is_null()` fails and the `!contains(VEC22_KEY)` assert fails).

- [ ] **Step 3: Swap the two predicates in `src/cmd/convert.rs`.**
  - `:1042` — `let from_value = if primary.node.is_secret_bearing() {` → `if primary.node.is_argv_secret_bearing() {`
  - `:1069` — `if outputs.iter().any(|(n, _)| n.is_secret_bearing()) {` → `if outputs.iter().any(|(n, _)| n.is_argv_secret_bearing()) {`, with a one-line comment: `// is_argv_secret_bearing widens to MiniKey for stdout-redaction parity; MiniKey is currently output-unreachable (one-way MiniKey→Wif) so this is a no-op today but keeps both redaction pathways on one predicate.`

- [ ] **Step 4: Run — verify GREEN** (the new cell + all existing minikey cells).

Run: `cargo test -p mnemonic-toolkit --test cli_convert_minikey`
Expected: all pass (existing non-JSON cells unaffected; new JSON cell passes).

- [ ] **Step 5: Commit.**

```bash
git add crates/mnemonic-toolkit/src/cmd/convert.rs crates/mnemonic-toolkit/tests/cli_convert_minikey.rs
git commit -m "fix(convert): redact MiniKey input in --json from_value (is_argv_secret_bearing)"
```

---

## Task 2: Promote `SECRET_NODE_TYPES_ARGV` public const + parity test

**Files:** `src/secret_taxonomy.rs` (add const); `src/cmd/convert.rs` (add parity test + import).

- [ ] **Step 1: Add the const** to `secret_taxonomy.rs` immediately after `SECRET_NODE_TYPES` (after its closing `];` at L85):

```rust

/// Token-form strings of every `NodeType` variant whose **wider**
/// `is_argv_secret_bearing()` predicate returns `true` — the persistence
/// set `SECRET_NODE_TYPES` PLUS `MiniKey` (Casascius mini-key, a private-key
/// encoding). This is the argv-leakage / stdout-redaction superset, mirrored
/// by `NodeType::is_argv_secret_bearing` at `cmd/convert.rs:117`. Kept in
/// lockstep by the `secret_taxonomy_argv_parity_with_is_argv_secret_bearing`
/// parity test. Downstream argv-redaction consumers (e.g. a GUI run-confirm
/// preview) should use THIS set, not the narrower `SECRET_NODE_TYPES`.
pub const SECRET_NODE_TYPES_ARGV: &[&str] = &[
    "phrase",
    "entropy",
    "xprv",
    "wif",
    "ms1",
    "bip38",
    "electrum-phrase",
    "seedqr",
    "minikey",
];
```

- [ ] **Step 2: Add the parity test** to `convert.rs` test module, immediately after `secret_taxonomy_parity_with_is_secret_bearing` (ends ~L1742). Also extend the import at `convert.rs:1680` to include the new const:
  - import: `use mnemonic_toolkit::secret_taxonomy::SECRET_NODE_TYPES;` → `use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES, SECRET_NODE_TYPES_ARGV};`
  - test:

```rust
    #[test]
    fn secret_taxonomy_argv_parity_with_is_argv_secret_bearing() {
        for &v in ALL_NODE_TYPE_VARIANTS {
            let predicate = v.is_argv_secret_bearing();
            let in_taxonomy = SECRET_NODE_TYPES_ARGV.contains(&v.as_str());
            assert_eq!(
                predicate, in_taxonomy,
                "drift: NodeType::{:?}.is_argv_secret_bearing()={} but \
                 SECRET_NODE_TYPES_ARGV.contains({:?})={}",
                v, predicate, v.as_str(), in_taxonomy
            );
        }
    }
```

- [ ] **Step 3: Run the parity tests.**

Run: `cargo test -p mnemonic-toolkit secret_taxonomy`
Expected: both `secret_taxonomy_parity_with_is_secret_bearing` and `secret_taxonomy_argv_parity_with_is_argv_secret_bearing` PASS.

- [ ] **Step 4: Commit.**

```bash
git add crates/mnemonic-toolkit/src/secret_taxonomy.rs crates/mnemonic-toolkit/src/cmd/convert.rs
git commit -m "feat(secret-taxonomy): promote pub const SECRET_NODE_TYPES_ARGV (wide argv set) + parity test"
```

---

## Task 3: Close FOLLOWUPs + release artifacts

**Files:** `design/FOLLOWUPS.md`, `Cargo.toml`, `Cargo.lock`, `scripts/install.sh`, `CHANGELOG.md`.

- [ ] **Step 1: Close both slugs in `design/FOLLOWUPS.md`.**
  - `convert-minikey-stdout-redaction` Status → resolved:
```
- **Status:** resolved — v0.34.5. Switched the two stdout-redaction call sites (`convert.rs:1042` from_value JSON echo + `:1069` secret-on-stdout warning) from `is_secret_bearing()` to the wider `is_argv_secret_bearing()`. The actual fix is `:1042` — `--from minikey= --to wif --json` no longer echoes the minikey private key in `from_value` (regression cell `minikey_input_redacted_in_json_from_value`). `:1069` is a no-op for MiniKey today (one-way `MiniKey→Wif`; the WIF output already trips it) but keeps both pathways on one predicate. Closed via cycle-prep recon (SHA `f4d553e`/`b17444b`).
```
  - `secret-taxonomy-argv-superset-promotion` Status → resolved:
```
- **Status:** resolved — v0.34.5. Added `pub const SECRET_NODE_TYPES_ARGV` to `secret_taxonomy.rs` (the wide argv-leakage set = `SECRET_NODE_TYPES` + `minikey`), mirrored by `NodeType::is_argv_secret_bearing` and locked by the new `secret_taxonomy_argv_parity_with_is_argv_secret_bearing` parity test (sibling of the narrow-set parity test). Additive public const — no GUI lockstep forced (the GUI's existing `SECRET_NODE_TYPES` snapshot is unchanged). The GUI-side `gui-run-confirm-modal-secret-redaction` consumer can now adopt it. Closed via cycle-prep recon (SHA `b17444b`).
```

- [ ] **Step 2: Version bump + lock regen.** `Cargo.toml` `0.34.4`→`0.34.5`; `cargo build -p mnemonic-toolkit`; confirm `Cargo.lock` mnemonic-toolkit = `0.34.5`.

- [ ] **Step 3: install.sh self-pin** `mnemonic-toolkit-v0.34.4`→`-v0.34.5`.

- [ ] **Step 4: CHANGELOG** `[0.34.5]` above `[0.34.4]`:

```
## mnemonic-toolkit [0.34.5] — 2026-05-22

**SemVer-PATCH — MiniKey stdout-redaction hardening + `SECRET_NODE_TYPES_ARGV`.** `convert --from minikey=<KEY> --to wif --json` no longer echoes the Casascius mini-key (a private key) unredacted in the JSON `from_value` field: the two `convert` stdout-redaction call sites now use the wider `is_argv_secret_bearing()` predicate (which includes MiniKey) instead of the narrow `is_secret_bearing()`. Promotes `pub const secret_taxonomy::SECRET_NODE_TYPES_ARGV` (the public mirror of that wide set = persistence set + `minikey`), locked by a new parity test against `is_argv_secret_bearing`. Additive public const → no GUI lockstep (existing `SECRET_NODE_TYPES` snapshot unchanged). Closes `convert-minikey-stdout-redaction` + `secret-taxonomy-argv-superset-promotion`.
```

- [ ] **Step 5: Full regression + clippy + manual lint.**

Run: `cargo test -p mnemonic-toolkit` → green. `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean. `make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk` → 6/6 OK.

- [ ] **Step 6: Commit release artifacts.**

```bash
git add design/FOLLOWUPS.md crates/mnemonic-toolkit/Cargo.toml Cargo.lock scripts/install.sh CHANGELOG.md design/IMPLEMENTATION_PLAN_v0_34_5_minikey_leak_hardening.md design/agent-reports/v0_34_5-plan-r0-review.md
git commit -m "release(toolkit): mnemonic-toolkit v0.34.5 — MiniKey stdout-redaction hardening"
```

- [ ] **Step 7: End-of-cycle opus review → GREEN (0C/0I)**; persist to `design/agent-reports/v0_34_5-end-of-cycle-review.md`.

- [ ] **Step 8: Auto-ship** (per the user's batch-cadence decision — clean toolkit-only PATCH ships without re-asking): merge→master (ff), push, tag `mnemonic-toolkit-v0.34.5`, GH release. Then PROCEED to cycle 3 (signet) and PAUSE for user go-ahead there.

---

## Self-review (writing-plans)

- **Spec coverage:** both slugs (minikey redaction + SECRET_NODE_TYPES_ARGV) closed. ✓
- **No placeholders:** test code, predicate swaps, const, parity test all verbatim. ✓
- **Type consistency:** `is_argv_secret_bearing` exists (`convert.rs:117`); `SECRET_NODE_TYPES_ARGV` mirrors `is_argv_secret_bearing` = narrow + minikey; parity test mirrors the narrow-set test's exact shape. ✓
- **SemVer/lockstep:** PATCH; additive const → no GUI lockstep; no flag → no manual lockstep. ✓
- **Risk:** low — `:1042` is the only behavior change (JSON-only, additively redacts); `:1069` no-op; const is additive; parity test guards drift.
