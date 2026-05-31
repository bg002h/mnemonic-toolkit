# Output-Type Stderr Advisory Implementation Plan (Cycle B, Phase 1)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every output-producing command in `mnemonic` (toolkit) and `ms` (ms-cli) emits exactly one stderr line classifying the worst-case security nature of what it wrote to stdout — `private key material (can spend)` / `watch-only` / `template` — so "no advisory line ⟺ inert output" holds across both CLIs.

**Architecture:** A new `OutputClass` lattice + `emit_output_class_advisory` + `worst_class_on_stdout -> Option<OutputClass>` in `secret_advisory.rs` subsumes the legacy D9 `secret_on_stdout_warning{,_unconditional}` by complement. Each command computes its class at the run-level (post-stdout-write) site — a constant for fixed commands, `worst_class_on_stdout`/predicates for multi-artifact ones. ms-cli gets a byte-parity-tested duplicate (independent crate). Stderr-only → PATCH; the bulk is re-capturing worked-example transcripts across 4 doc trees.

**Tech Stack:** Rust (edition 2021, pinned 1.85.0), `assert_cmd`; cross-repo `mnemonic-toolkit` (tag-only v0.38.2) + `mnemonic-secret` ms-cli (crates.io v0.5.1).

**SPEC:** `design/SPEC_output_type_advisory.md` (R0 gate GREEN, R2 0C/0I) — the per-command class table (§3) is the authoritative checklist. **Source SHA at plan-write:** `18cfdce`.

**Branches:** `cycle-b-output-type-advisory` (toolkit, exists — design committed); create a sibling branch in `mnemonic-secret` for P4.

---

## File Structure

| File | Responsibility | Phase |
|---|---|---|
| `crates/mnemonic-toolkit/src/secret_advisory.rs` | `OutputClass`, `emit_output_class_advisory`, `worst_class_on_stdout`, `card_kind_class`; legacy helpers removed at end | P0, P3 |
| `crates/mnemonic-toolkit/src/cmd/{derive_child,silent_payment,electrum_decrypt,seedqr,addresses,export_wallet,final_word,seed_xor,slip39}.rs` | fixed-class wiring + TTY-gate drop + addenda | P1 |
| `crates/mnemonic-toolkit/src/cmd/{bundle,convert,repair,inspect,import_wallet,nostr}.rs` | multi-artifact/conditional wiring | P2 |
| `crates/mnemonic-toolkit/src/repair.rs` (`emit_repair_report`) + `cmd/repair.rs` | auto-repair short-circuit re-route | P3 |
| `crates/mnemonic-toolkit/tests/cli_output_class.rs` | **NEW** — per-command class cells | P1-P3 |
| `mnemonic-secret/crates/ms-cli/src/advisory.rs` + `cmd/{encode,decode,derive,repair}.rs` | duplicate helper + wiring | P4 |
| `mnemonic-secret/crates/ms-cli/tests/cli_output_class.rs` | **NEW** — ms class cells + byte-parity | P4 |
| 4 doc trees + 12 toolkit + 3 ms test files | re-word sweep + transcript re-capture | P5 |

---

## Task 0 (P0): the `OutputClass` lattice + helpers

**Files:** Modify `crates/mnemonic-toolkit/src/secret_advisory.rs` (+ its inline `#[cfg(test)] mod tests`).

- [ ] **Step 1: Write the failing tests** — append to (or create) the inline `mod tests`:

```rust
    #[test]
    fn output_class_lattice_and_lines() {
        use super::{OutputClass::*, worst_class_on_stdout, emit_output_class_advisory};
        // worst = most-sensitive present; None if empty (all-inert).
        assert_eq!(worst_class_on_stdout(&[]), None);
        assert_eq!(worst_class_on_stdout(&[Template, WatchOnly]), Some(WatchOnly));
        assert_eq!(worst_class_on_stdout(&[WatchOnly, PrivateKeyMaterial, Template]), Some(PrivateKeyMaterial));
        // exact bytes per SPEC §2.2 (em-dash U+2014, '> file.txt').
        let mut b = Vec::new();
        emit_output_class_advisory(PrivateKeyMaterial, &mut b);
        assert_eq!(String::from_utf8(b).unwrap(),
            "warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')\n");
        let mut b = Vec::new(); emit_output_class_advisory(WatchOnly, &mut b);
        assert_eq!(String::from_utf8(b).unwrap(), "note: stdout is watch-only — public keys only, cannot spend\n");
        let mut b = Vec::new(); emit_output_class_advisory(Template, &mut b);
        assert_eq!(String::from_utf8(b).unwrap(), "note: stdout is a keyless descriptor template (no keys)\n");
    }

    #[test]
    fn card_kind_maps_to_class() {
        use super::{card_kind_class, OutputClass};
        use crate::repair::CardKind;
        assert_eq!(card_kind_class(CardKind::Ms1), OutputClass::PrivateKeyMaterial);
        assert_eq!(card_kind_class(CardKind::Mk1), OutputClass::WatchOnly);
        assert_eq!(card_kind_class(CardKind::Md1), OutputClass::Template);
    }
```

- [ ] **Step 2: Run — verify FAIL**

Run: `cargo test -p mnemonic-toolkit --bins secret_advisory::tests::output_class`
Expected: FAIL — `OutputClass`/`worst_class_on_stdout`/`emit_output_class_advisory`/`card_kind_class` not defined.

- [ ] **Step 3: Implement** — add to `secret_advisory.rs` (variant order is ASCENDING sensitivity so `#[derive(Ord)]` makes `.max()` = most-sensitive; the SPEC's `{P,W,T}` listing is conceptual, not declaration order):

```rust
/// Security class of what a command wrote to stdout. Variant declaration order
/// is ascending sensitivity (Template < WatchOnly < PrivateKeyMaterial) so
/// `#[derive(Ord)]`'s `.max()` returns the most-sensitive class. "inert" is the
/// ABSENCE of a class (modeled as `Option::None`), not a variant. SPEC §2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputClass { Template, WatchOnly, PrivateKeyMaterial }

/// Max over the artifacts a command wrote to stdout; `None` == all-inert → no line.
pub fn worst_class_on_stdout(artifacts: &[OutputClass]) -> Option<OutputClass> {
    artifacts.iter().copied().max()
}

/// Map a repaired/inspected card kind to its output class.
pub fn card_kind_class(kind: crate::repair::CardKind) -> OutputClass {
    match kind {
        crate::repair::CardKind::Ms1 => OutputClass::PrivateKeyMaterial,
        crate::repair::CardKind::Mk1 => OutputClass::WatchOnly,
        crate::repair::CardKind::Md1 => OutputClass::Template,
    }
}

/// Emit the one-line stderr class advisory. Inert outputs do NOT call this.
pub fn emit_output_class_advisory<W: Write + ?Sized>(class: OutputClass, stderr: &mut W) {
    let line = match class {
        OutputClass::PrivateKeyMaterial =>
            "warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')",
        OutputClass::WatchOnly => "note: stdout is watch-only — public keys only, cannot spend",
        OutputClass::Template => "note: stdout is a keyless descriptor template (no keys)",
    };
    let _ = writeln!(stderr, "{line}");
}
```
Do NOT remove the legacy `secret_on_stdout_warning{,_unconditional}` yet — call sites migrate in P1-P3; removal is P3 Step (last).

- [ ] **Step 4: Run — verify PASS** Run: `cargo test -p mnemonic-toolkit --bins secret_advisory::tests` Expected: PASS. `cargo build -p mnemonic-toolkit` clean (new helpers may `dead_code`-warn until P1 wires them — acceptable, do not `#[allow]` unless build fails).

- [ ] **Step 5: Commit**
```bash
git add crates/mnemonic-toolkit/src/secret_advisory.rs
git commit -m "feat(advisory): OutputClass lattice + emit/worst/card_kind helpers (B P0)"
```

---

## Task 1 (P1): toolkit fixed-class wiring + TTY-gate drop

**Files:** the 9 `cmd/*.rs` listed; Test: `crates/mnemonic-toolkit/tests/cli_output_class.rs` (create).

**The wiring pattern** (apply at each command's run-level site, AFTER the stdout artifact is written, where `stderr` is in scope):
```rust
crate::secret_advisory::emit_output_class_advisory(crate::secret_advisory::OutputClass::<CLASS>, stderr);
```
Replacing each command's current D9 / `_unconditional` call (or adding it where none exists). **Drop the TTY gate** (`if std::io::stdout().is_terminal()`) on `final-word`/`seed-xor split`/`seed-xor combine`/`slip39 split`/`slip39 combine` — emit unconditionally — and KEEP each command's bespoke clause as an ADDENDUM line AFTER the unified line (SPEC §2.3). **[folds I5]** Dropping the only `is_terminal()` call in a file orphans its `use std::io::IsTerminal` (and any `stdout()` binding) → `unused_imports` hard-fails the P6 clippy `-D warnings` gate AND reds the P1 commit — **remove the orphaned `IsTerminal` import in the same edit** — `slip39.rs:54`, `seed_xor.rs:24`, `final_word.rs:20` (all `use std::io::{IsTerminal, …}`) [M1]. Do NOT touch `compare_cost.rs:6`'s `IsTerminal` (its gate is not dropped).

**Per-command checklist (P1 — fixed class):**

| Command | Class | Emit-site (re-grep) | Action |
|---|---|---|---|
| `derive-child` | P | `derive_child.rs:308` | replace inlined literal → `emit_output_class_advisory(PrivateKeyMaterial, stderr)` |
| `silent-payment` | P | `silent_payment.rs:286` | replace `_unconditional` call |
| `electrum-decrypt` | P | `electrum_decrypt.rs:149` (stdout branch only) | replace `_unconditional`; `--json-out` branch stays no-line (I5) |
| `seedqr encode` | P | **run-level `run_encode` (`seedqr.rs:215`)** [folds I1 — `:323` is inside stdout-only `emit_encode_output`, no stderr] | **add** after `emit_encode_output(...)`, gated `if args.json_out.is_none() { emit(P, stderr) }` (file→inert, I5) |
| `seedqr decode` | P | **run-level `run_decode` (`seedqr.rs:133`)** [folds I1 — `:295` is stdout-only `emit_decode_output`] | **add** after `emit_decode_output(...)`, gated `args.json_out.is_none()` |
| `addresses` | W | `addresses.rs` run-level (post-render) | **add** `emit(WatchOnly)` |
| `export-wallet` | W | `export_wallet.rs:250-254` run-level + `run_from_import_json` | **add** `emit(WatchOnly)` (incl. `--from-import-json`) |
| `final-word` | P + addendum | `final_word.rs:101` (TTY-gated) | drop TTY gate; `emit(P)` then addendum `:104` text |
| `seed-xor split` | P + addendum | `seed_xor.rs:241` | drop TTY gate; `emit(P)` then "ALL N shares … no auth tag" addendum |
| `seed-xor combine` | P + addendum | `seed_xor.rs:365` | drop TTY gate; `emit(P)` then addendum |
| `slip39 split` | P + addendum | `slip39.rs:544` | drop TTY gate; `emit(P)` then group/member addendum |
| `slip39 combine` | P + addendum | `slip39.rs:681,684` | drop TTY gate; `emit(P)` then "verify the recovered wallet's address" addendum |

- [ ] **Step 1: Write the failing integration cells** — create `tests/cli_output_class.rs`:

```rust
//! Cycle B — output-type stderr advisory: per-command class assertions.
use assert_cmd::Command;
const P_LINE: &str = "warning: stdout carries private key material (can spend)";
const W_LINE: &str = "note: stdout is watch-only — public keys only, cannot spend";
const ABANDON: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
fn mnemonic() -> Command { Command::cargo_bin("mnemonic").unwrap() }
fn stderr(o: &std::process::Output) -> String { String::from_utf8_lossy(&o.stderr).into() }

#[test]
fn derive_child_emits_private_key_material() {
    let o = mnemonic().args(["derive-child", "--phrase", ABANDON, "--application", "bip39", "--index", "0"]).output().unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}
#[test]
fn export_wallet_emits_watch_only() {
    // export-wallet with a concrete descriptor → watch-only wallet file.
    let o = mnemonic().args(["export-wallet", "--descriptor",
        "wpkh([704c7836/84h/0h/0h]tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/0/*)",
        "--network", "testnet"]).output().unwrap();
    assert!(stderr(&o).contains(W_LINE), "{}", stderr(&o));
}
#[test]
fn slip39_split_emits_on_pipe_not_just_tty() {
    // TTY-gate-removal regression: piped (non-TTY) stdout still emits the P line.
    let o = mnemonic().args(["slip39", "split", "--phrase", ABANDON, "--groups", "1", "--threshold", "1", "--shares", "1"]).output().unwrap();
    assert!(stderr(&o).contains(P_LINE), "{}", stderr(&o));
}
```
(Before writing each cell, run the command once to confirm the exact required args — e.g. `mnemonic derive-child --help`, `mnemonic slip39 split --help` — and adjust args so a CORRECT impl exits 0/with-output. Do NOT weaken the `contains` assertion.) Add one cell per P1 command following this shape (the per-command class is in the checklist + SPEC §3).

- [ ] **Step 2: Run — verify FAIL** Run: `cargo test -p mnemonic-toolkit --test cli_output_class` — the net-new (seedqr/addresses/export-wallet) + TTY-gated (slip39 etc.) cells FAIL; the already-emitting ones (derive-child) may already pass for P-presence but FAIL the re-word until P5 — assert on `contains(P_LINE)` (new wording) so they fail now and pass after wiring.

- [ ] **Step 3: Implement** the P1 checklist rows. Worked example — `slip39 split` (`cmd/slip39.rs:544`), drop the TTY gate + addendum:
```rust
    // was: if std::io::stdout().is_terminal() { writeln!(stderr, "<old slip39 advisory>")?; }
    crate::secret_advisory::emit_output_class_advisory(
        crate::secret_advisory::OutputClass::PrivateKeyMaterial, stderr);
    writeln!(stderr, "note: each share is secret material — distribute across separate locations; \
        SLIP-39 shares have no authentication tag")?; // addendum (keep the bespoke clause; pin exact bytes)
```
Worked example — `addresses` (net-new W): at the run-level after the address table is written to stdout, `emit_output_class_advisory(OutputClass::WatchOnly, stderr);`.

- [ ] **Step 4: Run — verify PASS + no regression, re-pinning the P1-touched suites IN P1** (so the P1 commit is green; only the cross-cutting/doc remainder defers to P5). Run `cargo test -p mnemonic-toolkit --test cli_output_class` + the touched suites (`ls tests | grep -E 'slip39|seed_xor|final_word|seedqr|electrum|silent|derive_child|addresses|export'`). Re-pin NOW, in P1: (a) positive `.contains("secret material on stdout")` → new P wording; (b) **the 4 TTY-gate NEGATIVE cells [R1-I-new]** — `cli_final_word_advisories.rs:72`, `cli_seed_xor_advisories.rs:96`, `cli_seed_xor_advisories.rs:133`, `cli_slip39_advisories.rs:310` — INVERT to assert the new P-line is PRESENT on piped stdout (they assert non-literal strings the `secret material on stdout` grep won't find). Each P1-command suite green at the P1 commit.

- [ ] **Step 5: Commit** `git add` the 9 cmd files + the test; `git commit -m "feat(advisory): fixed-class wiring + drop TTY gate on 5 commands (B P1)"`

---

## Task 2 (P2): multi-artifact / conditional wiring

**Files:** `cmd/{bundle,convert,repair,inspect,import_wallet,nostr}.rs`; same test file.

**Pattern:** compute the class, then `if let Some(c) = <class-expr> { emit_output_class_advisory(c, stderr) }` (the `Option` form handles convert-all-inert).

| Command | Class expr (SPEC §3/§4.3) |
|---|---|
| `bundle` | `Some(if bundle.any_secret_bearing() { PrivateKeyMaterial } else { WatchOnly })` (replace the inlined literal at `bundle.rs:931`, which was gated on `any_secret_bearing`) |
| `convert` | **[folds I3 — `outputs.iter()` yields `&(NodeType, String)`, not `NodeType`]** at `convert.rs:1099` (stderr in scope), replace the literal at `:1102` with `if let Some(c) = worst_class_on_stdout(&outputs.iter().filter_map(\|(n,_)\| convert_target_class(*n)).collect::<Vec<_>>()) { emit_output_class_advisory(c, stderr) }`. Add `fn convert_target_class(t: NodeType) -> Option<OutputClass> { if t.is_argv_secret_bearing() { Some(PrivateKeyMaterial) } else if t.is_side_input_only() { None } else { Some(WatchOnly) } }` (predicates by-value `self` at `convert.rs:117,121`). `filter_map` drops `None` (path/fingerprint) → all-side-input → empty → `None` → no line. |
| `repair` | **[folds C2]** `kind` is NOT in scope at `cmd/repair.rs:215-216` (the per-chunk kind is loop-local at `:144`; only `any_ms1` is tracked). Inside the existing chunk loop, collect `let mut kinds: Vec<OutputClass> = Vec::new();` pushing `card_kind_class(chunk_kind)` for each card written to stdout; replace `if any_ms1 { secret_on_stdout_warning(CardKind::Ms1, stderr) }` with `if let Some(c) = worst_class_on_stdout(&kinds) { emit_output_class_advisory(c, stderr) }` |
| `inspect` | **[folds C2]** same — `inspect.rs:155-156` tracks only `any_ms1`; collect the `CardKind`s reaching stdout (loop var at `:111`) into `Vec<OutputClass>`, then `worst_class_on_stdout` + emit (now mk1→W, md1→T, not just ms1→P) |
| `import-wallet` | **[folds I2 — `:2111` is inside `emit_summary` (stdout-only); `entropy=` there is a diagnostic flag, not key material]** emit at the run-level site `~import_wallet.rs:1257-1270` (stdout + stderr + `parsed` in scope): `let cls = if parsed.iter().flat_map(\|p\| &p.cosigners).any(\|c\| c.entropy.is_some()) { PrivateKeyMaterial } else { WatchOnly }; emit_output_class_advisory(cls, stderr)` (predicate confirmed `:1456/:2106`) |
| `nostr` | npub branch (`:186`): `emit(WatchOnly)` (net-new); nsec branch (`:251`): `emit(PrivateKeyMaterial)` (replace `_unconditional`) |

- [ ] **Step 1: Failing cells** — both-ways per command. Examples:
```rust
#[test]
fn convert_to_xprv_is_secret_to_xpub_is_watch_only() {
    let s = stderr(&mnemonic().args(["convert", "--phrase", ABANDON, "--to", "xprv"]).output().unwrap());
    assert!(s.contains(P_LINE), "{s}");
    let w = stderr(&mnemonic().args(["convert", "--phrase", ABANDON, "--to", "xpub"]).output().unwrap());
    assert!(w.contains(W_LINE), "{w}");
}
#[test]
fn convert_to_path_only_is_inert() {
    let s = stderr(&mnemonic().args(["convert", "--phrase", ABANDON, "--to", "path"]).output().unwrap());
    assert!(!s.contains("note: stdout") && !s.contains("warning: stdout carries"), "path-only must be inert: {s}");
}
#[test]
fn nostr_npub_watch_only_nsec_secret() {
    // adjust args per `mnemonic nostr --help`; pubkey/npub branch → W, secret/nsec branch → P.
}
```
- [ ] **Step 2: Run — verify FAIL** (the conditional/net-new branches).
- [ ] **Step 3: Implement** the P2 table. For convert, add a `fn convert_target_class(t: NodeType) -> Option<OutputClass>` helper (P if `is_argv_secret_bearing`, None if `is_side_input_only`, else W) and `worst_class_on_stdout` over the `.flatten()`ed targets.
- [ ] **Step 4: Run — verify PASS + no regression** (existing bundle/convert/repair/inspect/nostr/import suites; re-pin their old-wording assertions as you go).
- [ ] **Step 5: Commit** `git commit -m "feat(advisory): multi-artifact/conditional wiring via worst_class_on_stdout (B P2)"`

---

## Task 3 (P3): inert audit + auto-repair re-route + file-output suppression + remove legacy helpers

**Files:** `repair.rs` (`emit_repair_report:1333`), `cmd/repair.rs`, `secret_advisory.rs` (remove legacy), test file.

- [ ] **Step 1: Failing cells**
```rust
#[test]
fn decode_address_is_inert() {
    let s = stderr(&mnemonic().args(["decode-address", "bc1q..."]).output().unwrap()); // valid addr
    assert!(!s.contains("note: stdout") && !s.contains("warning: stdout carries"), "{s}");
}
#[test]
fn auto_repair_short_circuit_emits_class() {
    // verify-bundle / xpub-search / inspect / convert given a 1-char-corrupt card with auto-repair
    // → exit 5, repaired card on stdout → class line. Cover ms1→P AND [M2 — locks the C1 widening]
    // mk1→W and md1→T (the existing cli_auto_repair.rs:104 md1 cell only asserts stdout-only).
    // Construct corrupt ms1/mk1/md1 fixtures; assert P_LINE / W_LINE / template line respectively.
}
#[test]
fn seedqr_jsonout_file_is_inert() {
    let dir = tempfile::tempdir().unwrap(); let p = dir.path().join("q.json");
    let s = stderr(&mnemonic().args(["seedqr","encode","--phrase",ABANDON,"--json-out",p.to_str().unwrap()]).output().unwrap());
    assert!(!s.contains("warning: stdout carries"), "file-output → no stdout-class line: {s}");
}
```
- [ ] **Step 2: Run — verify FAIL/behavior.**
- [ ] **Step 3: Implement.** (a) Re-route `emit_repair_report` (`repair.rs:1331-1334`) [folds C1 — the live code is `if matches!(outcome.kind, CardKind::Ms1) { secret_on_stdout_warning(outcome.kind, stderr); }`; the `Ms1` guard would leave mk1→W / md1→T SILENT]: **remove the `if matches!(outcome.kind, CardKind::Ms1)` guard** and emit unconditionally `crate::secret_advisory::emit_output_class_advisory(crate::secret_advisory::card_kind_class(outcome.kind), stderr)` (`RepairOutcome.kind: CardKind`, `repair.rs:408-409`). (b) Confirm the inert commands (verify-bundle/decode-address/verify-message/compare-cost/gui-schema/xpub-search×4) emit nothing on the normal branch — they already have no D9 call, so no change beyond the test. (c) File-output suppression is already structural (electrum-decrypt/seedqr `--json-out` branches don't reach the stdout emit) — assert via the cells. (d) **Remove the now-unused legacy `secret_on_stdout_warning` + `secret_on_stdout_warning_unconditional`** from `secret_advisory.rs` (grep confirms zero callers first).
- [ ] **Step 4: Run — verify PASS.** `cargo build -p mnemonic-toolkit` (no dead-code, no unresolved). Consolidation guard: `! grep -rq 'secret material on stdout' crates/mnemonic-toolkit/src` (zero orphaned literals).
- [ ] **Step 5: Commit** `git commit -m "feat(advisory): auto-repair re-route + inert audit + remove legacy D9 helpers (B P3)"`

---

## Task 4 (P4): ms-cli duplicate helper + wiring + byte-parity

**Files (mnemonic-secret repo):** `crates/ms-cli/src/advisory.rs`, `cmd/{encode,decode,derive,repair}.rs`; Test: `crates/ms-cli/tests/cli_output_class.rs` (create). **Branch:** create `cycle-b-output-type-advisory` in `mnemonic-secret`.

- [ ] **Step 1: Failing cells** (in mnemonic-secret):
```rust
use assert_cmd::Command;
const P_LINE: &str = "warning: stdout carries private key material (can spend)";
const W_LINE: &str = "note: stdout is watch-only — public keys only, cannot spend";
const ZEROS: &str = "00000000000000000000000000000000";
fn ms(args: &[&str]) -> std::process::Output { Command::cargo_bin("ms").unwrap().args(args).output().unwrap() }
#[test]
fn ms_decode_emits_private_key_material() {
    let card = String::from_utf8(ms(&["encode","--hex",ZEROS]).stdout).unwrap();
    let c = card.lines().next().unwrap().trim();
    let e = String::from_utf8_lossy(&ms(&["decode", c]).stderr).to_string();
    assert!(e.contains(P_LINE), "{e}");
}
#[test]
fn ms_derive_emits_watch_only_and_language_note() {
    let e = String::from_utf8_lossy(&ms(&["derive","--hex",ZEROS,"--template","bip84"]).stderr).to_string();
    assert!(e.contains(W_LINE), "{e}");        // new W line
    assert!(e.contains("defaulted"), "{e}");   // existing language note coexists [m3]
}
```
- [ ] **Step 2: Run — verify FAIL.**
- [ ] **Step 3: Implement.** Add to `ms-cli/src/advisory.rs` the byte-identical `OutputClass` + `emit_output_class_advisory` (the three lines must be byte-for-byte the toolkit's — copy them). Wire: `ms encode`/`ms decode` → `emit(PrivateKeyMaterial)` (net-new, run-level after the stdout write — these have TWO stdout branches (`--json` vs text, `encode.rs:135`/`:90`); place the emit AFTER both converge so the `--json` path isn't missed [M3]); `ms derive` → `emit(WatchOnly)` at **run-level AFTER the whole `if args.json { } else { }` block (`~derive.rs:253`), UNCONDITIONAL** [folds I4 — the language note at `:246-249` is inside `else { if defaulted { } }`, so emitting "after the language note" would skip `--json`/non-defaulted invocations]; `ms repair` → replace the inline literal at `cmd/repair.rs:106-109` with `emit(PrivateKeyMaterial)`. (`ms derive` HAS a threaded `stderr` param; encode/decode/repair emit via `eprintln!`/`std::io::stderr()` — the helper's `&mut impl Write` accepts `&mut std::io::stderr().lock()`.) (ms-cli `run(args)` is by-value, emits via `eprintln!`/`std::io::stderr()` — pass `&mut std::io::stderr().lock()` or have the helper use `eprintln!`.)
- [ ] **Step 4: Run — verify PASS** + **byte-parity test**: a test asserting ms-cli's 3 emitted lines equal the toolkit's literals (hard-code both, assert equal — cross-repo string sync guard). `cargo test -p ms-cli`.
- [ ] **Step 5: Commit** (in mnemonic-secret) `git commit -m "feat(advisory): ms output-class stderr advisory + byte-parity (B P4)"`

---

## Task 5 (P5): D9 re-word sweep + transcript re-capture (4 doc trees)

**Files:** SPEC §5's enumerated set — re-grep at this point.

- [ ] **Step 1: Re-pin the REMAINING test assertions** (the P1/P2/P4-touched suites are already re-pinned in their own phase — see those Step-4 notes — so each phase commit stays green). `grep -rl 'secret material on stdout' crates/mnemonic-toolkit/tests` (11 asserting + `cli_secret_in_argv_warning.rs` comment-only [M2]) + `mnemonic-secret/crates/ms-cli/tests`. Three re-pin classes [folds I6]:
  - **positive `.contains("secret material on stdout")`** → new wording (or the class predicate for conditional commands);
  - **NEGATIVE/absence assertions broken by the TTY-gate drop [folds R1-I-new — the `secret material on stdout` grep does NOT find these; they assert DIFFERENT literals]. ALL FIVE, enumerated (discover by test-name pattern `piped.*does_not_emit` / `non.?tty`, not the literal):** `cli_slip39_advisories.rs:310-313` (`!contains("reconstructed secret material on stdout")`), `cli_final_word_advisories.rs:72-75` (`!contains("candidate list is secret material")`), `cli_seed_xor_advisories.rs:96-98` (`!contains("Seed XOR shares on stdout")`), `cli_seed_xor_advisories.rs:133-135` (`!contains("combined phrase is secret material")`). Each command now emits unconditionally → **INVERT** each to assert the new unified P-line IS PRESENT on piped (non-TTY) stdout (do NOT "re-pin to new wording"). **These MUST be folded in P1** (their commands are P1) to keep the P1 commit green — not deferred to P5;
  - **conditional-command** assertions → assert the class predicate (P vs W) both ways.
  `cargo test -p mnemonic-toolkit` + `cargo test -p ms-cli` GREEN.
- [ ] **Step 2: Rebuild all 4 binaries** (mnemonic + md + ms + mk) from current source (MEMORY lesson — stale siblings cause false transcript drift).
- [ ] **Step 3: Re-capture transcripts in ALL 4 doc trees** (each has its own gate): `docs/manual` (`make -C docs/manual audit MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=…`), `docs/technical-manual`, `docs/manual-gui` (`make -C docs/manual-gui …` / its `tests/verify-examples.sh`), `docs/quickstart` (`docs/quickstart/Makefile`). For each failing transcript, regenerate the `.out`/`.err` (capture-not-author, per the coldcard/recipe-2 precedent) + update the prose mirror lines (`41-mnemonic.md`, `43-ms.md`, the gui/quickstart `.md`). Re-run each gate → GREEN.
- [ ] **Step 4: Verify** all 4 gates green + full suites green (both repos).
- [ ] **Step 5: Commit** the test re-pins + transcripts + prose mirrors (stage explicitly per tree) `git commit -m "docs+test(advisory): re-word D9 → output-class; re-capture 4 doc trees (B P5)"`

---

## Task 6 (P6): versions + FOLLOWUP + end-of-cycle R0 + ship

- [ ] **Step 1: Versions.** mnemonic-toolkit → v0.38.2 (Cargo.toml + both READMEs `<!-- toolkit-version: -->` markers + CHANGELOG). ms-cli → v0.5.1 (Cargo.toml + top-level CHANGELOG `## ms-cli [0.5.1]`). Verify `cargo test -p mnemonic-toolkit --test readme_version_current`.
- [ ] **Step 2: File FOLLOWUP** `output-type-stderr-advisory-sibling-sweep-mk-md` mirrored in `mnemonic-toolkit/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md`, `descriptor-mnemonic/design/FOLLOWUPS.md` (+ companion line in mnemonic-secret) with cross-citing `Companion:` lines — Phase 2 (mk watch-only + md template/watch-only), bounded by the next install.sh sibling-pin bump.
- [ ] **Step 3: End-of-cycle R0.** Dispatch the opus architect against the full diff (both repos: `git diff master...HEAD`). Persist to `design/agent-reports/output-type-advisory-end-of-cycle-R0-review.md`. Fold → re-dispatch until GREEN.
- [ ] **Step 4: Pre-ship.** Both repos: clean tree, full suite green, `cargo clippy --all-targets -- -D warnings` clean (CI gate — run it; the implementers' `cargo test` won't catch clippy), all 4 doc gates green.
- [ ] **Step 5: Ship.** mnemonic-secret: ff-merge → master, push, `cargo publish -p ms-cli` (v0.5.1, crates.io). mnemonic-toolkit: ff-merge → master, push, tag `mnemonic-toolkit-v0.38.2` (tag-only). No GUI lockstep; the toolkit ms-cli sibling pin does NOT need a bump (independent advisories) — confirm via sibling-pin-check.

---

## Self-Review (controller, post-write)
**Spec coverage:** §2 lattice+lines→T0; §3 table→the P1/P2/P3 checklists; §4.1 helper→T0, replacement sites→P1/P2/P3; §4.2 ms-cli→T4; §4.3 wiring rules (worst_class_on_stdout, TTY-drop, file-suppression, emit-at-run-level)→P1/P2/P3; §5 sweep→T5; §6 SemVer→T6; §7 tests→cells across T1-T4; §8 phases→T0-T6; §10 FOLLOWUP→T6. No gap.
**Placeholder scan:** the P1/P2 per-command checklists give class + emit-site + action per command (deterministic) rather than 32 full code blocks — each row is a complete instruction backed by SPEC §3; worked examples show the exact pattern. The auto-repair fixture (T3) and nostr/import-wallet arg specifics are flagged "re-grep/`--help` first" (verification points, not placeholders). No "TBD"/"handle edge cases".
**Type consistency:** `OutputClass{Template,WatchOnly,PrivateKeyMaterial}` (ascending for Ord), `emit_output_class_advisory`, `worst_class_on_stdout->Option`, `card_kind_class`, `convert_target_class->Option` used consistently T0→T4.
**Known verification points** (named per task): exact `--help` args for derive-child/slip39/nostr/import-wallet/seedqr cells; the corrupt-ms1 auto-repair fixture; the `entropy=` import predicate; clippy `-D warnings` (T6 S4).
