# IMPLEMENTATION PLAN — cycle-3 — GUI secret-leak fixes H2 + H3

Phased TDD execution plan for the cycle-3 brainstorm spec
(`design/BRAINSTORM_cycle3_gui_secret_leaks.md`, **R0-GREEN 0C/0I** at round 2 —
`design/agent-reports/cycle3-spec-r0-round{1,2}-review.md`). DESIGN ONLY — feeds the
mandatory opus-architect **plan-doc R0 loop to 0C/0I BEFORE any code** (per
toolkit + GUI `CLAUDE.md`). All fixes land in **`mnemonic-gui`**; no toolkit change.

## Source-of-truth SHAs (verified live)

| Repo | `origin/master` | Role |
|---|---|---|
| **mnemonic-gui** (fix repo) | **`0b1e024`** | all citations `git show origin/master:<path>`-verified; crate **0.44.0** |
| mnemonic-toolkit (dep) | `c9168aac` | exposes `SECRET_NODE_TYPES_ARGV` at pinned tag `mnemonic-toolkit-v0.60.0`; **NO change** |

**Execution model (per `CLAUDE.md` per-phase pattern):** a **single implementer
subagent in a git worktree**, strict TDD (RED test before GREEN code), running the
**FULL `cargo test` + `cargo clippy --all-targets -D warnings`** suite at each phase
gate (per project memory `feedback_r0_review_run_full_package_suite` — full suite,
not targeted `--test`). Phases are sequential; each ends GREEN before the next
starts. Per-phase opus review persists to `design/agent-reports/cycle3-phase-N-*.md`
before commit. **NEVER `cargo fmt` the GUI** (no fmt CI gate; hand-format).

---

## Phase 1 — H2 runner argv-leak (isolated, `src/runner.rs` only)

**Defect:** `src/runner.rs:119` Debug-formats the **entire argv in cleartext**
(`debug!(target: "mnemonic_gui::runner", argv = ?argv, …)`); `--debug`/`RUST_LOG`
prints secret tokens (phrase/entropy/WIF/minikey) to stderr.

**RED test (write first):** add `runner_no_argv_leak` to **`tests/runner_integration.rs`**
(NOT a new file — reuse that file's proven harness). Per spec **R0 I1** AND
**plan-R0 I1**: the naive `with_default + rebuild + #[serial]` scheme is
**insufficient and uncompilable** — `serial_test` is NOT a dependency (verified), and
the sibling `cell_2_tracing_init_logs_subprocess_spawn` (`tests/runner_integration.rs:140-193`)
proves that even `set_default + rebuild_interest_cache()` still needs a **3-attempt
retry loop** because a concurrent test re-races the GLOBAL callsite-interest cache
between rebuild and spawn. Since this test's load-bearing assertion is NEGATIVE, a
poisoned/empty capture would flip to flaky-RED, not stable-GREEN. **Adopt the proven
`cell_2` harness verbatim (no new dep):**
```rust
// reuse the file's CapturedWriter(Arc<Mutex<Vec<u8>>>) MakeWriter
let mut captured = String::new();
for attempt in 1..=3 {
    let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(CapturedWriter(buf.clone())).with_ansi(false).finish();
    let guard = tracing::subscriber::set_default(subscriber);
    tracing::callsite::rebuild_interest_cache();
    let _ = runner::run([mnemonic_bin(), "--version".into(),
        "SENTINEL_SEED_abandon_abandon_xxxxxxxx".into()]).expect("spawn ok");
    drop(guard);
    captured = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    if captured.contains("subprocess spawn") {    // CAPTURE GATE: the event message is present
                                                  // in BOTH old + new code → negative assertion
                                                  // is always reached on a captured attempt
        assert!(!captured.contains("SENTINEL_SEED"), // the leak assertion (the discriminator)
            "argv leaked to debug log: {captured}");
        assert!(captured.contains("argv_len"),    // positive control: the NEW field shape shipped
            "expected argv_len field in the fixed spawn log: {captured}");
        return;                                    // PASS
    }
    eprintln!("attempt {attempt}/3 missed the spawn event (interest race); retrying");
}
panic!("could not capture runner spawn debug event after 3 attempts:\n{captured}");
```
- **plan-R0 M6 — gate on the always-present `"subprocess spawn"` message, NOT the
  GREEN-only `argv_len`** (mirrors `cell_2` exactly). This makes the negative
  (sentinel-absent) assertion reachable on BOTH old and new code, so an empty capture
  retries-then-panics (never a false-GREEN) AND RED surfaces for the RIGHT reason.
- **RED now:** current `:119` logs `argv = ?argv` → the captured "subprocess spawn"
  line CONTAINS `SENTINEL_SEED` → `assert!(!contains)` **fails** (true RED, leak
  detected — not a misleading "could not capture" panic).
- **GREEN after the fix:** `program=%argv[0]` + `argv_len` shipped → sentinel gone
  (first assert passes) AND `argv_len` present (positive control passes).

**GREEN edit:** replace the `:119` macro with (exact shape from spec):
```rust
debug!(
    target: "mnemonic_gui::runner",
    program = %argv[0],     // resolved binary path/name — never secret
    argv_len = argv.len(),  // token count — never secret
    stdin = stdin.is_some(),
    "subprocess spawn",
);
```
No signature change; `run_with_stdin`'s public contract (SPEC §0/§2.1) untouched. The
exit-code logs (`:166-168`) and the stdin `warn!` (`:144`) already never log argv —
leak is isolated to `:119`.

**Phase-1 gate:** full `cargo test` GREEN (incl. the new RED→GREEN test) + clippy
clean. **Blast radius:** `runner.rs` only — disjoint from all H3 files.

---

## Phase 2 — H3 taxonomy widen + 3 predicate-swap surfaces (no widget change)

**Defect:** the GUI classifies composite-node secrecy via the **narrow**
`SECRET_NODE_TYPES` (excludes `minikey`, a Casascius mini **private key**). Route the
argv/confirm/persist surfaces to the wide `SECRET_NODE_TYPES_ARGV` (= narrow +
`minikey`; verified delta **exactly `{minikey}`**).

**RED tests (write first):**
1. **Drift-guard** (`tests/secret_taxonomy_pin.rs`): assert (a) wide ⊇ narrow; (b) `minikey ∈ wide ∧ minikey ∉ narrow`; (c) `node_type_is_argv_secret("minikey")==true ∧ node_type_is_secret("minikey")==false`. **Compile-time belt:** add a **NEW sibling fallback mod** (spec Q3 — e.g. `argv_canonical_fallback`, NOT `v0_3_canonical_fallback` which is v0.5.0-doomed) holding a `SECRET_NODE_TYPES_ARGV` snapshot + `const _: () = assert!(secret_slice_eq(SECRET_NODE_TYPES_ARGV, snapshot));`.
2. **Regression — surfaces (i)/(ii)/(iv)** — **AUTHORITATIVE minikey coverage; this test MUST construct a form state that actually contains a `--from minikey=<KEY>` composite** (per plan-R0 M2: the `tests/persistence.rs` fixture does NOT, so the loop-widening below is vacuous for minikey on its own). Assert: (i) `assemble_argv_with_secret_mask` marks the minikey value token `mask=true`; (ii) `should_confirm_run` returns `true`; (iv) `redact_for_persistence` DROPS the minikey composite AND the serialized `state.json` bytes do NOT contain the minikey value. Negative controls: a `--from xpub=…` composite is NOT masked, does NOT confirm, and IS persisted (watch-only).
3. **On-disk loop (defense-in-depth, NOT the minikey proof):** widen the `tests/persistence.rs:156` `for node in SECRET_NODE_TYPES` on-disk-absence loop to iterate the WIDE set. **Note (plan-R0 M2):** the existing `mixed_form_with_secrets()` fixture has only `phrase`/`xpub` composites, so this loop does NOT by itself exercise `minikey` — it just keeps the loop iterating the authoritative set; the real minikey on-disk proof lives in regression test 2.(iv) above, which adds a genuine minikey composite.

**GREEN edits:**
- `secrets.rs:34` — add `SECRET_NODE_TYPES_ARGV` to the `pub use mnemonic_toolkit::secret_taxonomy::{…}` re-export.
- `secrets.rs` (after `node_type_is_secret`, ~`:161`) — add
  ```rust
  /// True iff `node` is in the WIDER argv/redaction secret set (narrow + minikey).
  /// Use for argv-mask / run-confirm / persistence-redact / paste-warn; the narrow
  /// `node_type_is_secret` stays for any persistence-narrow semantics.
  pub fn node_type_is_argv_secret(node: &str) -> bool {
      SECRET_NODE_TYPES_ARGV.contains(&node)
  }
  ```
- **Surface 1 — argv-mask** (`form/invocation.rs:457`): `node_type_is_secret(node)` → `node_type_is_argv_secret(node)` in the `mask.push(flag_is_secret(flag) || …)`.
- **Surface 2 — run-confirm** (`secrets.rs:230`, the `NodeValueComposite` branch of `should_confirm_run`): `node_type_is_secret(node)` → `node_type_is_argv_secret(node)`.
- **Surface 3 — persistence** (`persistence.rs:96`): `SECRET_NODE_TYPES.contains(&node.as_str())` → `SECRET_NODE_TYPES_ARGV.contains(&node.as_str())`; update the import at `:31`; update the inline comment (`:94`) + module docs (`:16`,`:73`) `SECRET_NODE_TYPES` → `SECRET_NODE_TYPES_ARGV` (stale-comment hygiene). **Over-redaction check folded:** the only added drop is `minikey` (a private key) — nothing argv-secret-but-persist-safe exists, so wide==correct for persistence.

**Surface-1 clipboard note (spec I2 — do NOT mis-implement):** the mask bit drives
**Preview + last-run redaction + the copy-button informed-reveal label** only; the
`ctx.copy_text` clipboard PAYLOAD stays cleartext BY DESIGN (parity with
phrase/xprv). The regression test asserts masked-PREVIEW, **never** clipboard
redaction.

**Phase-2 gate:** full `cargo test` GREEN + clippy clean. **Files:** `secrets.rs`,
`form/invocation.rs`, `persistence.rs`, `tests/secret_taxonomy_pin.rs`,
`tests/persistence.rs`, a new H3 regression test — disjoint from Phase 1's
`runner.rs` and Phase 3's `widget.rs`.

---

## Phase 3 — H3 surface 4: node-aware composite paste-warn (widget wiring) + comment

**Defect/structure (spec-verified):** the composite VALUE field
(`form/widget.rs:646`) is a bare `ui.text_edit_singleline(value)` with **no
`response` captured and NO paste detection** — live paste detection exists ONLY in
`form/secret_widget.rs:85-105` (the `SecretLineEdit` bus-flag). So a predicate swap
alone delivers nothing here; the widget needs detection wired in. This closes the
already-filed FOLLOWUP `composite-paste-warn-parity` for **all 9 composite nodes**
(the warn RAISES only for secret-class nodes via the gate).

**RED test (write first):** regression surface (iii) — a `--from minikey=<KEY>`
over-threshold paste raises `paste_warn_id()` (kittest, mirroring
`tests/paste_warn_wiring_v0_40_0.rs`). **plan-R0 M5 (egui role — do NOT copy the
model verbatim):** the model focuses `Role::PasswordInput` (the `SecretLineEdit`
password widget); the composite value field is a **plain non-password**
`text_edit_singleline` → its accessibility role is **`Role::TextInput`**. The test
MUST query/focus the composite value field by `Role::TextInput` (or by its label),
NOT `Role::PasswordInput`, or it queries the wrong node and passes vacuously.
Negative control: `--from xpub=…` paste does NOT raise. **Must be RED now** (no
detection on the composite widget).

**GREEN edit** (`form/widget.rs` `NodeValueComposite` arm, ~`:644-646`) — replace the
bare `ui.text_edit_singleline(value)` with a paste-detecting render mirroring
`SecretLineEdit::show`'s pattern:
```rust
let response = ui.text_edit_singleline(value);
let pasted_len = ui.input(|i| {
    i.events.iter().find_map(|e| match e {
        egui::Event::Paste(s) => Some(s.chars().count()),
        _ => None,
    })
});
if response.changed() {
    if let Some(len) = pasted_len {
        if len >= crate::secrets::PASTE_WARN_THRESHOLD
            && crate::secrets::node_type_is_argv_secret(node.as_str())  // node-aware gate → no over-warn (plan-R0 M4: explicit .as_str())
        {
            ui.ctx().data_mut(|d| d.insert_temp(
                crate::form::secret_widget::paste_warn_id(), true));
        }
    }
}
```
The existing `update()` read-once `remove_temp` (`main.rs:1057`) already consumes the
flag + renders the modal → **no new modal wiring**, single chokepoint preserved.
**Over-warn check folded:** the gate fires only for secret-class nodes
(phrase/entropy/xprv/wif/ms1/bip38/electrum-phrase/seedqr/**minikey**); non-secret
`--from` nodes (xpub/fingerprint/path/mk1/address) do NOT warn.

**Comment fix** (`schema/mnemonic.rs:1119` **AND `:2095`** — plan-R0 M1: the SAME
false comment appears a second time on the `seedqr decode --from` composite at
`:2095`; fix both so the stale-comment class the spec elevates does not survive):
replace the FALSE
`secret: false, // secrecy is value-dependent; per-row paste-warn fires` with
`secret: false, // secrecy is node-dependent; composite paste-warn + argv-mask + run-confirm + persist-redact key on node_type_is_argv_secret (cycle-3)`.
The `secret: false` field STAYS at both sites (no `schema_mirror`/`schema_mirror_secret_drift`
delta — no flag-name / dropdown-value / per-flag-secret-bool change).

**Phase-3 gate:** full `cargo test` GREEN + clippy clean. **Files:** `form/widget.rs`,
`schema/mnemonic.rs` (comment), the paste regression test — disjoint from Phases 1-2.

---

## Phase 4 — version sites + FOLLOWUP flips + ship (single wrap commit set)

**SemVer: GUI MINOR `0.44.0 → 0.45.0`** (new masking/confirm/paste/redaction
behavior; no breaking API). **Toolkit pin STAYS `mnemonic-toolkit-v0.60.0`** (no bump).

**Version sites (all four — `readme_pin_coherence` is the one gate the bump can trip):**
| Site | Path | Edit |
|---|---|---|
| crate version | `Cargo.toml:3` | `0.44.0` → `0.45.0` |
| lockfile | `Cargo.lock` (`name = "mnemonic-gui"` block, ~`:2266`) | regenerate via build |
| README self-tag | `README.md:42` `--tag mnemonic-gui-v0.44.0` | → `v0.45.0` (**`readme_pin_coherence` asserts self-tag == Cargo.toml version**) |
| CHANGELOG | `CHANGELOG.md` | prepend `## mnemonic-gui [0.45.0] — 2026-06-21` (H2 + H3) |

**Do NOT touch:** `schema/mnemonic.rs:4344` `pinned_version: "mnemonic 0.59.0"`
(toolkit display banner — pre-existing `0.59.0`-vs-pin-`v0.60.0` skew is OUT of
scope) or `pinned-upstream.toml` (no pin bump).

**FOLLOWUP / checklist flips (in the shipping commit — `feedback_followup_status_discipline`).**
NOTE (plan-R0 M3): the GUI FOLLOWUPS registry is at the **repo ROOT `FOLLOWUPS.md`**
(not `design/`); verified the two bughunt-id slugs are NOT yet present → **file as
new RESOLVED entries**, the two paste-warn slugs ARE present (`:40`/`:48`) → flip/note:
- **FILE-NEW (RESOLVED):** `gui-runner-debug-logs-unmasked-secret-argv` (H2); `gui-minikey-secret-not-masked-in-argv-preview` (H3, folding the `w3-gui-minikey-persist-plaintext` facet) — add as RESOLVED entries with the cycle-3 SPEC + fixing-commit SHA.
- **FLIP existing:** `composite-paste-warn-parity` (`FOLLOWUPS.md:40`) → RESOLVED (the all-9-composites node-aware paste-warn closes it).
- **STAYS OPEN (note considered+deferred):** `slot-field-paste-warn-uncovered` (`:48`) — no minikey path (minikey is `--from`-only, never a slot).
- toolkit `design/agent-reports/constellation-bughunt-2026-06-20.md`: tick the **H2** and **H3** `[ ]`→`[x]` checkboxes citing the GUI fixing commit SHA.

**CI gates to stay green:** `clippy -D warnings`; `schema_mirror` (×4 CLIs) +
`schema_mirror_secret_drift` + `archetype_schema_mirror` + `gui_schema_conditional_drift`
+ `xpub_search_schema_mirror` (no schema delta); `readme_pin_coherence` + `pin_coherence`
(green iff README self-tag bumps in lockstep); `secret_taxonomy_pin` (extended) + the
new H3 drift-guard. **No `cargo fmt`.**

---

## Mandatory post-implementation gate

After Phase 3 GREEN, **before** the version bump/ship: a **mandatory, non-deferrable
independent adversarial execution review over the whole GUI diff** (R0 = plan
correctness; this catches implementation-introduced regressions TDD misses — per
`CLAUDE.md` per-phase pattern (4)). Persist to `design/agent-reports/`. Ship only
after it is clean.

## Workstream disjointness (file ownership)

| Phase | Files | Disjoint from |
|---|---|---|
| 1 (H2) | `src/runner.rs` (+ test) | 2, 3 |
| 2 (H3 taxonomy+3 surfaces) | `src/secrets.rs`, `src/form/invocation.rs`, `src/persistence.rs`, `tests/secret_taxonomy_pin.rs`, `tests/persistence.rs`, new regression test | 1, 3 |
| 3 (H3 paste widget) | `src/form/widget.rs`, `src/schema/mnemonic.rs` (comment), paste regression test | 1, 2 |
| 4 (ship) | `Cargo.toml`, `Cargo.lock`, `README.md`, `CHANGELOG.md`, GUI `FOLLOWUPS.md`, toolkit bughunt checklist | — |

Sequential single-implementer execution recommended (small cycle, ~70-110 LOC);
phase order 1→2→3→4. R0 plan-review must converge to 0C/0I before Phase 1 code.
