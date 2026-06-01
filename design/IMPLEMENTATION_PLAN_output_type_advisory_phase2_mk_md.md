# Output-class advisory Phase 2 (mk + md) + Tier-0 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the 1-line stderr output-class advisory (shipped Phase 1 on `mnemonic`/`ms`) to mk-cli (→ watch-only) and md-cli (→ template / watch-only), completing the constellation invariant "no advisory line ⟺ inert stdout", and fold in the Tier-0 md-codec 0.34→0.35 pin-bump that fixes `mnemonic repair --md1` on non-chunked md1.

**Architecture:** Each bin-only CLI gets its own `output_advisory.rs` (no shared dep — codecs are upstream of the toolkit), a byte-for-byte copy of the **ms-cli precedent** `mnemonic-secret/crates/ms-cli/src/advisory.rs` (NOT the toolkit lib). Each emitting handler calls `emit_output_class_advisory` at every success return site. The toolkit phase bumps the md-codec pin, re-captures the one affected manual transcript, and bumps 5 sibling-pin tag sites.

**Tech Stack:** Rust (edition 2021/2024 per repo), clap-derive CLIs, `assert_cmd` integration tests, `cargo clippy --all-targets -- -D warnings` + `missing_docs=warn` CI gates.

**Source SHAs (re-grep line numbers before each edit — CLAUDE.md):** toolkit `64943f2`, mk `e5620ce`, md `c599292`, ms-cli precedent at `mnemonic-secret` `4e5266a`.

**Spec:** `design/SPEC_output_type_advisory_phase2_mk_md.md` (R0 GREEN). **Spec reviews:** `design/agent-reports/output-type-advisory-phase2-spec-R{0,1}-review.md`.

**Plan R0 gate:** **GREEN** — R0 (2C/3I) → R1 (0C/3I) → R2 (0C/1I) → **R3 GREEN (0C/0I)**, persisted at `design/agent-reports/output-type-advisory-phase2-plan-R{0,1,2,3}-review.md`. Cleared for code at Phase A. (Per-phase + end-of-cycle R0 reviews still apply during execution.)

**Cross-repo ship order (hard dependency):** Phase A (mk-cli, tag `mk-cli-v0.6.1`) → Phase B (md-cli, tag `descriptor-mnemonic-md-cli-v0.6.2`) → Phase C (toolkit; consumes both tags). Each phase ends with a per-phase opus review persisted to `design/agent-reports/` before its commit/tag (CLAUDE.md). Do not start a phase's code until the prior phase is GREEN + tagged.

**Class map (from spec §3, architect-resolved):**
- mk emitting (all **WatchOnly**): encode, decode, inspect, repair, derive, address. Inert: verify, vectors, gui-schema.
- md emitting: encode/decode/inspect/bytecode/repair/compile = **Template**; address = **WatchOnly**. Inert: verify, vectors, gui-schema.

---

## Phase A — mk-cli (repo `mnemonic-key`, branch `main`)

Working dir: `/scratch/code/shibboleth/mnemonic-key`. All paths below are relative to it.

### Task A1: Create the advisory module + wire the first caller (`decode`)

> The `pub fn` must have a real (non-test) caller in this task: in a bin-only crate, a `#[cfg(test)]`-only call does NOT prevent `dead_code` (R0 C1). So the module + its first handler land together.

**Files:**
- Create: `crates/mk-cli/src/output_advisory.rs`
- Modify: `crates/mk-cli/src/main.rs` (add `mod output_advisory;` near `:9-11`)
- Modify: `crates/mk-cli/src/cmd/decode.rs` (emit before the success return)
- Test: `crates/mk-cli/tests/cli_output_class.rs` (new)

- [ ] **Step 1: Write the failing integration test for `mk decode`**

Create `crates/mk-cli/tests/cli_output_class.rs`. There is **NO literal mk1 in `tests/`** (R0 C1) — mk tests build fixtures **in-process** via `mk_codec::encode`. Copy the `card()` helper + the single-sig account xpub from the shipped `crates/mk-cli/tests/cli_address.rs:17,33-44`. `V2_84_MAIN` at `m/84h/0h/0h` is a single-sig **depth-3** card that `mk address` accepts and infers `p2wpkh` (no `--address-type`, no off-depth advisory — R0 C2). `mk_codec`, `bitcoin`, `assert_cmd` are already dev-deps (cli_address.rs uses them).

```rust
use std::str::FromStr;
use assert_cmd::Command;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::KeyCard;

/// Exact watch-only advisory line (em-dash U+2014). MUST be byte-identical to
/// mnemonic-toolkit's secret_advisory.rs + ms-cli's advisory.rs.
const WATCH_ONLY_LINE: &str =
    "note: stdout is watch-only \u{2014} public keys only, cannot spend";

/// Single-sig depth-3 account xpub (m/84'/0'/0'), lifted from cli_address.rs corpus.
const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";

/// Build a single-sig mk1 card (Vec of chunk strings) — verbatim copy of cli_address.rs::card.
fn card(xpub: &str, origin_path: &str) -> Vec<String> {
    let kc = KeyCard::new(
        vec![[0xde, 0xad, 0xbe, 0xef]],
        Some(Fingerprint::from([0x73, 0xc5, 0xda, 0x0a])),
        DerivationPath::from_str(origin_path).unwrap(),
        Xpub::from_str(xpub).unwrap(),
    );
    mk_codec::encode(&kc).unwrap()
}

/// The single-sig fixture all mk cells use (address-accepted, no depth advisory).
fn mk1_fixture() -> Vec<String> { card(V2_84_MAIN, "m/84h/0h/0h") }

#[test]
fn decode_emits_watch_only_advisory() {
    let out = Command::cargo_bin("mk").unwrap()
        .arg("decode").args(mk1_fixture()).output().unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains(WATCH_ONLY_LINE), "missing advisory; stderr={stderr}");
}
```

> Note (R1 m7): `mk1_fixture()` returns a **multi-chunk `Vec<String>`** (the V2 card is 2 chunks). Every mk cell must spread it via `.args(mk1_fixture())` — do NOT "simplify" to a single `.arg(...)`. All mk output subcommands accept variadic positional mk1 strings (verified exit 0).

- [ ] **Step 2: Run it; verify it fails**

Run: `cargo test -p mk-cli --test cli_output_class decode_emits_watch_only_advisory`
Expected: FAIL (no advisory on stderr yet).

- [ ] **Step 3: Create the advisory module**

Create `crates/mk-cli/src/output_advisory.rs`:

```rust
//! Output-class stderr advisory (Phase 2 sibling sweep).
//!
//! Byte-for-byte duplicate of mnemonic-toolkit's
//! `secret_advisory::emit_output_class_advisory`. mk-cli is upstream of the
//! toolkit and cannot depend on it, so the helper is duplicated; cross-repo
//! byte parity is enforced by `tests/cli_output_class.rs::byte_parity_advisory_lines`.

use std::io::Write;

/// Security class of what a command wrote to stdout. Byte-identical variant set
/// to mnemonic-toolkit's `secret_advisory::OutputClass`.
///
/// `#[allow(dead_code)]`: mk-cli is a bin-only crate and constructs only
/// `WatchOnly`; `Template`/`PrivateKeyMaterial` exist for advisory-text parity
/// (guarded by the byte-parity test) but are never constructed here.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputClass {
    PrivateKeyMaterial,
    WatchOnly,
    Template,
}

/// Emit the one-line stderr class advisory. Byte-identical to mnemonic-toolkit's
/// `secret_advisory::emit_output_class_advisory` (cross-repo parity — see the
/// byte-parity test). Inert outputs do NOT call this.
pub fn emit_output_class_advisory<W: Write>(class: OutputClass, stderr: &mut W) {
    let line = match class {
        OutputClass::PrivateKeyMaterial =>
            "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')",
        OutputClass::WatchOnly => "note: stdout is watch-only \u{2014} public keys only, cannot spend",
        OutputClass::Template => "note: stdout is a keyless descriptor template (no keys)",
    };
    let _ = writeln!(stderr, "{line}");
}
```

- [ ] **Step 4: Register the module**

In `crates/mk-cli/src/main.rs`, add to the module block (near `:9`):

```rust
mod output_advisory;
```

- [ ] **Step 5: Wire the emit into `decode`'s success return**

In `crates/mk-cli/src/cmd/decode.rs`, immediately before the success `Ok(0)` (re-grep; the run() exits with a single trailing `Ok(0)` after the json/text dispatch), insert:

```rust
crate::output_advisory::emit_output_class_advisory(
    crate::output_advisory::OutputClass::WatchOnly,
    &mut std::io::stderr(),
);
```

- [ ] **Step 6: Run the test; verify it passes + clippy is clean**

Run: `cargo test -p mk-cli --test cli_output_class decode_emits_watch_only_advisory`
Expected: PASS
Run: `cargo clippy -p mk-cli --all-targets -- -D warnings`
Expected: clean (the `#[allow(dead_code)]` enum + the now-live `emit_output_class_advisory` caller satisfy the gate).

- [ ] **Step 7: Commit**

```bash
git add crates/mk-cli/src/output_advisory.rs crates/mk-cli/src/main.rs crates/mk-cli/src/cmd/decode.rs crates/mk-cli/tests/cli_output_class.rs
git commit -m "feat(mk-cli): output-class stderr advisory module + decode emit (Phase 2)"
```

### Task A2: Wire the remaining 5 emitting handlers

Apply the **same emit line** (WatchOnly) as A1-Step-5 to each handler below, at every success return site, and add one positive integration cell per handler to `tests/cli_output_class.rs`. mk handlers each have a single trailing success return (verified R0 I1); `repair` covers exit 0 AND exit 5 at one return and must NOT emit on its `?`-propagated error path.

| Handler file (re-grep return site) | Cell name | Invocation fixture |
|---|---|---|
| `cmd/encode.rs` (before `Ok(0)` ~`:97`) | `encode_emits_watch_only` | `mk encode --xpub V2_84_MAIN --origin-path "m/84h/0h/0h" --policy-id-stub deadbeef --privacy-preserving` (R1 I4: encode requires ≥1 of `--policy-id-stub`/`--from-md1` or exits 64; `--privacy-preserving` omits the fingerprint — matches `cli_derive.rs:124-131`) |
| `cmd/inspect.rs` (single trailing `Ok`) | `inspect_emits_watch_only` | `mk inspect <mk1_fixture()>` |
| `cmd/repair.rs` — emit in **`run()` before `:86 Ok(if any_correction {5} else {0})`** (covers exit 0 AND 5); NOT the `:149`/`:202` `emit_text`/`emit_json` helper `Ok(())` returns (R0 I2); error path is `?`-propagated → no emit | `repair_emits_watch_only` | a 1-**data**-symbol-corrupted copy of `mk1_fixture()[0]` — use the bech32-cyclic-shift `flip_at(chunk, pos)` idiom from `crates/mk-cli/tests/cli_repair.rs:46` (R1 m6; corrupt the payload region, not the `mk1` HRP) so the input stays parseable-but-BCH-invalid → repair exits 5 |
| `cmd/derive.rs` (single trailing `Ok`) | `derive_emits_watch_only` | `mk derive <mk1_fixture()> --index 0` |
| `cmd/address.rs` (single trailing `Ok`) | `address_emits_watch_only` | `mk address <mk1_fixture()> --count 1` (84' infers p2wpkh — no `--address-type`, no depth advisory) |

- [ ] **Step 1: For each row — write the failing cell** (assert `stderr.contains(WATCH_ONLY_LINE)` and `status.success()`), modeled on A1-Step-1.
- [ ] **Step 2: Run all 5; verify they fail.** `cargo test -p mk-cli --test cli_output_class`
- [ ] **Step 3: For each row — insert the WatchOnly emit line** at the handler's success return (the same 3-line call from A1-Step-5).
- [ ] **Step 4: Run all cells; verify they pass.** `cargo test -p mk-cli --test cli_output_class`
- [ ] **Step 5: Commit.**

```bash
git add crates/mk-cli/src/cmd/encode.rs crates/mk-cli/src/cmd/inspect.rs crates/mk-cli/src/cmd/repair.rs crates/mk-cli/src/cmd/derive.rs crates/mk-cli/src/cmd/address.rs crates/mk-cli/tests/cli_output_class.rs
git commit -m "feat(mk-cli): emit watch-only advisory on encode/inspect/repair/derive/address (Phase 2)"
```

### Task A3: Byte-parity guard + inert negative cells + version bump + audit + tag

**Files:** `crates/mk-cli/tests/cli_output_class.rs`, `crates/mk-cli/Cargo.toml`, `CHANGELOG.md` (if present).

- [ ] **Step 1: Add the byte-parity guard test** (asserts the 3 literal constants are byte-exact, mirroring ms-cli `byte_parity_advisory_lines`):

```rust
const PRIVATE_KEY_LINE: &str = "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')";
const TEMPLATE_LINE: &str = "note: stdout is a keyless descriptor template (no keys)";

#[test]
fn byte_parity_advisory_lines() {
    assert_eq!(PRIVATE_KEY_LINE, "warning: stdout carries private key material (can spend) \u{2014} redirect or encrypt (e.g. '> file.txt' or '| age -e ...')");
    assert_eq!(WATCH_ONLY_LINE, "note: stdout is watch-only \u{2014} public keys only, cannot spend");
    assert_eq!(TEMPLATE_LINE, "note: stdout is a keyless descriptor template (no keys)");
}
```

- [ ] **Step 2: Add 3 inert negative cells** — `verify`/`vectors`/`gui-schema` must NOT emit any advisory line:

```rust
#[test]
fn verify_emits_no_advisory() {
    // `mk verify <mk1>` with no content matcher → BCH check → "OK" (inert).
    let out = Command::cargo_bin("mk").unwrap()
        .arg("verify").args(mk1_fixture()).output().unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(!stderr.contains("note: stdout is watch-only"), "verify must be inert; stderr={stderr}");
}
// analogous: vectors_emits_no_advisory (`mk vectors`), gui_schema_emits_no_advisory (`mk gui-schema`)
```

- [ ] **Step 3: Run the full test suite + clippy.** `cargo test -p mk-cli` then `cargo clippy -p mk-cli --all-targets -- -D warnings`. Expected: all green.
- [ ] **Step 4: Bump version** in `crates/mk-cli/Cargo.toml` `version = "0.6.0"` → `"0.6.1"`; add a CHANGELOG entry if the repo keeps one (re-grep for `CHANGELOG.md`; if none / not CI-gated, skip per the manual-prose-execution-gate precedent).
- [ ] **Step 5: Persist the per-phase opus review** to `<toolkit>/design/agent-reports/output-type-advisory-phase2-mk-phase-A-R0-review.md` (dispatch the architect on the mk diff; loop to 0C/0I before tagging).
- [ ] **Step 6: Commit + tag (only on review GREEN).**

```bash
git add crates/mk-cli/Cargo.toml crates/mk-cli/tests/cli_output_class.rs   # + CHANGELOG.md if edited
git commit -m "release(mk-cli): v0.6.1 — output-class advisory sibling sweep (Phase 2)"
# tag only when the user authorizes the ship:
# git tag mk-cli-v0.6.1 && git push origin main --tags
```

---

## Phase B — md-cli (repo `descriptor-mnemonic`, branch `main`)

Working dir: `/scratch/code/shibboleth/descriptor-mnemonic`. Same module pattern as Phase A, with TWO differences: (1) class is **Template** for all emitting handlers except `address` (WatchOnly); (2) handlers have a `--json` **early `return Ok(0)`** — emit at BOTH the json branch and the text branch (R0 I1).

### Task B1: Create the advisory module + wire the first caller (`decode`, both branches)

**Files:**
- Create: `crates/md-cli/src/output_advisory.rs` (identical to Phase A's module, with the module doc referencing md-cli; md constructs Template+WatchOnly so only `PrivateKeyMaterial` is unconstructed — `#[allow(dead_code)]` still correct).
- Modify: `crates/md-cli/src/main.rs` (add `mod output_advisory;` near `:3-9`)
- Modify: `crates/md-cli/src/cmd/decode.rs` (emit before BOTH returns: json `:24`, text `:30` — re-grep)
- Test: `crates/md-cli/tests/cli_output_class.rs` (new)

- [ ] **Step 1: Write two failing cells** (text mode + json mode) for `md decode`:

```rust
use std::process::Command;
use assert_cmd::cargo::CommandCargoExt;

const TEMPLATE_LINE: &str = "note: stdout is a keyless descriptor template (no keys)";
/// Canonical v0.30 md1 — decodes clean (text + json). From `smoke.rs:19`
/// (`md encode "wpkh(@0/<0;1>/*)"` → this string). R1 I5: `cmd_decode.rs` has NO literal.
const MD1_FIXTURE: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

#[test]
fn decode_text_emits_template_advisory() {
    let out = Command::cargo_bin("md").unwrap().args(["decode", MD1_FIXTURE]).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains(TEMPLATE_LINE));
}
#[test]
fn decode_json_emits_template_advisory() {
    let out = Command::cargo_bin("md").unwrap().args(["decode", "--json", MD1_FIXTURE]).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains(TEMPLATE_LINE), "json branch missed the advisory (R0 I1)");
}
```

- [ ] **Step 2: Run; verify both fail.** `cargo test -p md-cli --test cli_output_class`
- [ ] **Step 3: Create `crates/md-cli/src/output_advisory.rs`** (copy Phase A module verbatim; change the module doc's `mk-cli` → `md-cli`).
- [ ] **Step 4: Register** `mod output_advisory;` in `crates/md-cli/src/main.rs` (near `:3`).
- [ ] **Step 5: Insert the Template emit at BOTH return sites** in `cmd/decode.rs` — before the json-branch `return Ok(0)` AND before the text `Ok(0)`:

```rust
crate::output_advisory::emit_output_class_advisory(
    crate::output_advisory::OutputClass::Template,
    &mut std::io::stderr(),
);
```

- [ ] **Step 6: Run; verify both pass + clippy clean.** `cargo test -p md-cli --test cli_output_class` ; `cargo clippy -p md-cli --all-targets -- -D warnings`
- [ ] **Step 7: Commit.**

```bash
git add crates/md-cli/src/output_advisory.rs crates/md-cli/src/main.rs crates/md-cli/src/cmd/decode.rs crates/md-cli/tests/cli_output_class.rs
git commit -m "feat(md-cli): output-class advisory module + decode emit (text+json) (Phase 2)"
```

### Task B2: Wire the remaining emitting handlers (template, multi-return) + address (watch-only)

For each handler, insert the class emit at **every** success return site (json early-return + text), and add text+json positive cells. `repair` is single-return (emit once before `Ok(if any_correction {5} else {0})` at `:152`; NOT on the `Ok(2)` error path `:121`). `compile` lives behind `#[cfg(feature = "cli-compiler")]` — its emit and its cells must be `cfg`-gated.

| Handler (re-grep returns) | Class | Cells (modes) |
|---|---|---|
| `cmd/encode.rs` (json `:69`, text `:95`) | Template | `encode_text_*`, `encode_json_*` |
| `cmd/inspect.rs` (json `:42`, text `:59`) | Template | `inspect_text_*`, `inspect_json_*` |
| `cmd/bytecode.rs` (json `:30`, text `:41`) | Template | `bytecode_text_*`, `bytecode_json_*` |
| `cmd/repair.rs` (single `:152`; error `:121` inert) | Template | `repair_emits_template` (+ a corrupted fixture) |
| `cmd/address.rs` (json `:57`, text) | **WatchOnly** | `address_text_emits_watch_only`, `address_json_*` (assert `WATCH_ONLY_LINE`) |
| `cmd/compile.rs` (json `:21`, text `:26`) — the **handler** that prints; NOT `src/compile.rs` (the compiler lib, R0 I1); `#[cfg(feature="cli-compiler")]` | Template | `compile_emits_template` under `#[cfg(feature="cli-compiler")]` |

**md fixtures (R1 I6 — provide a real mechanism, no placeholders):**
- `decode`/`inspect`/`bytecode`/`repair` cells → feed `MD1_FIXTURE` (`md1yqpqqxqq8xtwhw4xwn4qh`, smoke.rs:19). repair: corrupt one data symbol of it (flip a bech32 char in the payload).
- `encode` cell → `md encode "wpkh(@0/<0;1>/*)"` (keyless template → Template; the exact invocation smoke.rs:18 uses).
- `address` cell → copy `account_xpub(path, network)` from `crates/md-cli/tests/cmd_address.rs:12` (derives from the ABANDON mnemonic), then invoke the proven passing form (cmd_address.rs:67-86):
  ```rust
  let xpub = account_xpub("m/84'/0'/0'", bitcoin::Network::Bitcoin);
  let key_arg = format!("@0={xpub}");
  Command::cargo_bin("md").unwrap()
      .args(["address", "--template", "wpkh(@0/<0;1>/*)", "--key", &key_arg])
      .output().unwrap();   // assert stderr.contains(WATCH_ONLY_LINE)
  ```
  Needs `const WATCH_ONLY_LINE` + the `bip39`/`bitcoin` dev-deps (already present; cmd_address.rs uses them).
- `compile` cell (under `#[cfg(feature = "cli-compiler")]`) → `md compile "pk(@0)" --context segwitv0` (R2 I1: `md compile` requires BOTH `<EXPR>` and `--context` or exits 2; text → `wsh(pk(@0))` exit 0, `--json` exit 0 — invocation from cmd_compile.rs:9). Class = Template.

- [ ] **Step 1: Write the failing cells** for each row (text + json where applicable; `encode --key @i=XPUB` variant for encode to confirm it's still Template per R0 F4).
- [ ] **Step 2: Run; verify failures.** `cargo test -p md-cli --test cli_output_class`
- [ ] **Step 3: Insert the emit lines** at every return site per the table (Template, or WatchOnly for address). For `compile`, gate the call site; for `repair`, single site only.
- [ ] **Step 4: Run; verify passes** (default features). `cargo test -p md-cli --test cli_output_class`
- [ ] **Step 5: Verify the compile cell under its feature.** `cargo test -p md-cli --features cli-compiler --test cli_output_class compile_emits_template`
- [ ] **Step 6: Commit.**

```bash
git add crates/md-cli/src/cmd/encode.rs crates/md-cli/src/cmd/inspect.rs crates/md-cli/src/cmd/bytecode.rs crates/md-cli/src/cmd/repair.rs crates/md-cli/src/cmd/address.rs crates/md-cli/src/cmd/compile.rs crates/md-cli/tests/cli_output_class.rs
git commit -m "feat(md-cli): emit template/watch-only advisory across output subcommands (Phase 2)"
```

### Task B3: Byte-parity guard + inert negatives + version bump + audit + tag

**Files:** `crates/md-cli/tests/cli_output_class.rs`, `crates/md-cli/Cargo.toml`, `CHANGELOG.md`.

- [ ] **Step 1: Byte-parity guard test** (all 3 literal constants byte-exact — copy the `byte_parity_advisory_lines` body from Phase A Task A3-Step-1; md needs all 3 constants present).
- [ ] **Step 2: Inert negative cells** — `verify` (`md verify md1yqpqqxqq8xtwhw4xwn4qh --template "wpkh(@0/<0;1>/*)"` — R2 m8: that's `MD1_FIXTURE`'s actual template), `vectors` (**pass `--out <tempdir>`** via `tempfile::tempdir()` to avoid polluting the cwd `./vectors` default — R0 I3; assert stdout empty AND no advisory on stderr), `gui-schema` — assert none contains any of the 3 advisory lines. (`tempfile` is already a dev-dep.)
- [ ] **Step 3: Full suite + clippy, default AND cli-compiler features.** `cargo test -p md-cli` ; `cargo test -p md-cli --features cli-compiler` ; `cargo clippy -p md-cli --all-targets -- -D warnings` ; **`cargo clippy -p md-cli --all-targets --features cli-compiler -- -D warnings`** (R1 m2' — the compile emit only compiles under the feature). Expected: all green.
- [ ] **Step 4: Bump** `crates/md-cli/Cargo.toml` `version = "0.6.1"` → `"0.6.2"`; CHANGELOG entry if the repo keeps one.
- [ ] **Step 5: Persist the per-phase opus review** to `<toolkit>/design/agent-reports/output-type-advisory-phase2-md-phase-B-R0-review.md`; loop to 0C/0I.
- [ ] **Step 6: Commit + tag (on GREEN).**

```bash
git add crates/md-cli/Cargo.toml crates/md-cli/tests/cli_output_class.rs   # + CHANGELOG.md if edited
git commit -m "release(md-cli): v0.6.2 — output-class advisory sibling sweep (Phase 2)"
# git tag descriptor-mnemonic-md-cli-v0.6.2 && git push origin main --tags   (on ship authorization)
```

---

## Phase C — toolkit integration + Tier-0 (repo `mnemonic-toolkit`, branch `master`)

Working dir: `/scratch/code/shibboleth/mnemonic-toolkit`. Requires Phase A + B tags to exist.

### Task C1: Tier-0 — md-codec 0.34→0.35 pin bump + non-chunked repair smoke test

**Files:** `crates/mnemonic-toolkit/Cargo.toml:22`, `Cargo.lock`, a new smoke test under `crates/mnemonic-toolkit/tests/`, `design/FOLLOWUPS.md` (~:396 Resolution prose).

- [ ] **Step 1: Write the failing smoke test.** Add `crates/mnemonic-toolkit/tests/cli_repair_md1_non_chunked.rs`: generate a **non-chunked** md1 via `md encode` of a small payload (the form plain `md encode` emits for a short template — re-grep an existing small-template fixture; confirm it's non-chunked, i.e. a single string with no chunk header), corrupt exactly one **data** symbol in the payload region (NOT the `md1` HRP and NOT a checksum-only position — so it lands in a BCH-correctable spot; R0 m3), then assert `mnemonic repair --md1 <corrupted>` exits 5 and recovers the original. (Use `assert_cmd`; MD_BIN/MNEMONIC_BIN via cargo_bin.)
- [ ] **Step 2: Run it against the current 0.34 pin; verify it FAILS.** Run: `cargo test -p mnemonic-toolkit --test cli_repair_md1_non_chunked`. Expected: FAIL — on 0.34 the non-chunked md1 returns **exit 2** `PostCorrectionDecodeFailed` / `wire-format version mismatch: got 2, expected 4` (R2 m9 — empirically all 21 single-symbol corruptions exit 2 on 0.34, NOT "UnparseableInput"), so the test's `exit 5 + recovers` assertion fails. This proves the latent bug the false-`resolved` FOLLOWUP hid.
- [ ] **Step 3: Bump the pin.** Edit `crates/mnemonic-toolkit/Cargo.toml:22` `md-codec = "0.34.0"` → `md-codec = "0.35"`.
- [ ] **Step 4: Re-resolve the lockfile BEFORE staging it** (stale-lock gotcha). Run: `cargo build -p mnemonic-toolkit` (re-resolves `Cargo.lock` to md-codec 0.35.0 from crates.io — confirmed published).
- [ ] **Step 5: Run the smoke test; verify it PASSES.** `cargo test -p mnemonic-toolkit --test cli_repair_md1_non_chunked`. Expected: PASS.
- [ ] **Step 6: Run the full suite** to confirm the additive bump broke nothing. `cargo test -p mnemonic-toolkit`. Expected: green.
- [ ] **Step 7: Correct the false-`resolved` FOLLOWUP prose** at `design/FOLLOWUPS.md` ~:396 (`md-codec-decode-with-correction-supports-non-chunked-md1`): change the Resolution to state the pin bump actually landed in THIS cycle (cite the commit SHA after committing) and that the smoke test now guards it.
- [ ] **Step 8: Commit.**

```bash
git add crates/mnemonic-toolkit/Cargo.toml Cargo.lock crates/mnemonic-toolkit/tests/cli_repair_md1_non_chunked.rs design/FOLLOWUPS.md
git commit -m "fix(repair): consume md-codec 0.35 non-chunked decode (Tier-0) + smoke test + correct false-resolved FOLLOWUP"
```

### Task C2: Bump the 5 sibling-pin tag sites

**Files:** `scripts/install.sh:35,41`, `.github/workflows/manual.yml:77,84`, `.github/workflows/quickstart.yml:71` (re-grep each line).

- [ ] **Step 1: Edit all 5 sites** to the new tags (exact prefix forms): `install.sh:35` md → `descriptor-mnemonic-md-cli-v0.6.2`; `install.sh:41` mk → `mk-cli-v0.6.1`; `manual.yml:77` mk → `mk-cli-v0.6.1`; `manual.yml:84` md → `descriptor-mnemonic-md-cli-v0.6.2`; `quickstart.yml:71` mk → `mk-cli-v0.6.1`. (Leave `install.sh:38` ms-cli untouched — pre-existing, out of scope, not a gate trigger.)
- [ ] **Step 2: Run the sibling-pin gate locally.** The gate logic is **inline bash inside `.github/workflows/sibling-pin-check.yml`** (R0 m4 — not a standalone script): copy its parse-and-compare steps into a shell and run them against the edited `install.sh`/workflows, then `actionlint .github/workflows/*.yml`. Expected: exit 0 (install.sh ↔ all workflows consistent).
- [ ] **Step 3: Commit.**

```bash
git add scripts/install.sh .github/workflows/manual.yml .github/workflows/quickstart.yml
git commit -m "chore(pins): bump mk-cli v0.6.1 + md-cli v0.6.2 sibling pins (5 sites) (Phase 2)"
```

### Task C3: Re-capture the affected CI-gated manual transcript

**Files:** `docs/manual/transcripts/24-recover-md1.out` (sole affected — R0 M3).

- [ ] **Step 1: Build all 4 binaries** (mnemonic + md + ms + mk) at the new versions, exactly as the manual CI does (use the install.sh/manual.yml install path or `cargo build` of each + the new mk/md tags). (Memory: after a clean, ALL 4 must be rebuilt or transcripts drift.)
- [ ] **Step 2: Re-sweep transcripts** for `$MD_BIN`/`$MK_BIN` output invocations: `grep -rnE '\$(MD_BIN|MK_BIN)' docs/manual/transcripts/`. Expected: only `24-recover-md1.cmd` (`$MD_BIN decode`). If a new one appeared, handle it too.
- [ ] **Step 3: Re-capture `24-recover-md1.out`** against the rebuilt binaries (pair mode `2>&1`). The new `.out` now carries the `template` note.
- [ ] **Step 4: Confirm idempotency (R0 M3 — pair-mode interleave).** Re-run `verify-examples` TWICE; the captured `.out` must be stable across runs. Run: `make -C docs/manual verify-examples` (or the harness invocation with the 4 BIN vars). Expected: 0 diffs, both runs. Re-capture only because the new output is CORRECT (verify the template note is the only delta).
- [ ] **Step 5: Run the full manual audit.** `make -C docs/manual audit MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=…`. Expected: lint + verify-examples + anchor-check all green. (Use pre-built binary paths, not `cargo run`, to avoid sibling dead-code warnings leaking into transcripts.)
- [ ] **Step 6: Commit.**

```bash
git add docs/manual/transcripts/24-recover-md1.out
git commit -m "docs(manual): re-capture 24-recover-md1 for md decode template advisory (Phase 2)"
```

### Task C4: FOLLOWUP closures + toolkit version bump + end-of-cycle review + tag

**Files:** toolkit `design/FOLLOWUPS.md` (`output-type-stderr-advisory-sibling-sweep-mk-md` → resolved); mirror closures in `mnemonic-key/design/FOLLOWUPS.md` + `descriptor-mnemonic/design/FOLLOWUPS.md` (+ ms companion note); toolkit `crates/mnemonic-toolkit/Cargo.toml` version.

- [ ] **Step 1: Mark the FOLLOWUP resolved** in all repos' `design/FOLLOWUPS.md` (`output-type-stderr-advisory-sibling-sweep-mk-md`): Status → `resolved` with this cycle's SHAs; note "completes the constellation-wide no-line⟺inert invariant." Keep cross-citing `Companion:` lines in lockstep.
- [ ] **Step 2: Bump toolkit version** `crates/mnemonic-toolkit/Cargo.toml` `0.38.2` → **`0.38.3`** (PATCH — stderr-only consumer + Tier-0). Run `cargo build` then stage `Cargo.lock`.
- [ ] **Step 3: Full toolkit test + clippy + audit.** `cargo test -p mnemonic-toolkit` ; `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` ; manual audit (Task C3-Step-5). Expected: all green.
- [ ] **Step 4: Persist the end-of-cycle opus R0 review** to `design/agent-reports/output-type-advisory-phase2-end-of-cycle-R0-review.md` (whole cross-repo diff); loop to 0C/0I.
- [ ] **Step 5: Commit + tag (on ship authorization + GREEN).**

```bash
git add design/FOLLOWUPS.md crates/mnemonic-toolkit/Cargo.toml Cargo.lock
# (sibling FOLLOWUPS.md closures committed in their own repos)
git commit -m "release(toolkit): output-class advisory Phase 2 lockstep + Tier-0 md-codec 0.35"
# git tag mnemonic-toolkit-vX.Y.Z && git push origin master --tags   (on authorization)
```

---

## Self-review notes (spec coverage)
- Spec §3 class map → Tasks A1/A2 (mk), B1/B2 (md). Spec §4 helper → A1/B1 modules. Spec §5 byte-parity + per-subcommand cells → A1-A3, B1-B3 (incl. all `--json` cells per R0 I1, inert negatives, compile `cfg`-gate). Spec §6 transcript re-capture → C3. Spec §7 Tier-0 → C1. Spec §8 phasing/5-site pins → C2; git tags as deliverable → A3/B3 tag steps. Spec §9 footguns: `#[allow(dead_code)]` (A1/B1 modules), multi-return (B1/B2), 5-site pins (C2), stale-lock (C1-Step-4), missing_docs (module doc comments).
- **Tag/publish steps are gated on explicit user ship authorization** per the repo's "commit/push/publish only when asked" rule — the executor stops at the pre-tag commit and asks.
