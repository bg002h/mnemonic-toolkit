# Security sweep — secret key-material hygiene in `mnemonic-gui`

- **Repo:** `/scratch/code/shibboleth/mnemonic-gui`
- **Audited against:** `origin/master` @ `1999323` (Merge PR #14, cycle-11a; just-shipped **v0.46.0**). `git fetch -q origin` performed; all line/byte claims via `git show origin/master:<path>`.
- **Scope:** secret key material (seed phrase / BIP-39 entropy / xprv / WIF / minikey / passphrase / ms1 share / BIP-38) **typed into or held by the GUI** — form-state `String`s, the descriptor-builder `TreeNode.key`/`.keys`, text-edit buffers, clipboard, on-disk autosave/state, run-result/pending-argv app holders.
- **Mode:** RECON / AUDIT ONLY. No fixes, no specs, no source edits. Candidate FOLLOWUP slugs only — orchestrator dedups + files.

---

## Verification of the two ALREADY-FIXED defenses

### M9 `TreeNode::zeroize_keys` (v0.46.0) — COMPLETE for the model-side tree

- `src/secrets.rs::zeroize_form_state` (last arm) calls `state.tree.as_mut().map(|t| t.root.zeroize_keys())`.
- `src/form/tree_model.rs:258` `zeroize_keys` zeroizes `self.key`, every `self.keys[i]`, and recurses **all** `children` (incl. surplus). Excludes `hex` (public digest) — correct, matches the on-disk redactor's exclusion.
- **Verdict: complete for `TreeNode`'s secret-bearing model fields** (`key`/`keys`). The `hex`/`w`/`n`/`k` fields are non-secret-class by design (hashlock `hex` is a public commitment; `w`/`n`/`k` carry no key material). No gap inside the tree model itself.

### On-disk redactor `blank_non_extended_public_keys` (`tree_model.rs:714`) + `redact_for_persistence` (`persistence.rs`) — COMPLETE for the persist path

- `persistence.rs::redact_for_persistence` drops: `SECRET_FLAG_NAMES`, every schema-secret flag NAME (union), `NodeValueComposite` whose node ∈ `SECRET_NODE_TYPES_ARGV` (incl. `minikey`, v0.45.0 H3), secret slot subkeys, **all** positionals (unconditional belt), `secret_widgets` (type-level `#[serde(skip)]`), and tree `key`/`keys[i]` via the positive extended-public allowlist (`blank_non_extended_public_keys` — fail-closed: WIF/raw-hex/xprv all blank, only xpub-family survives).
- **Verdict: the on-disk redactor is complete for the fields it owns.** The one *known* persist gap is already filed: tree `hex`/`w` free-text could carry a mis-pasted xprv (`tree-xprv-heuristic-only-covers-key-fields`, open). No NEW persist leak of a key/keys/flag/slot/positional secret found.

**Net on the two defenses:** both are COMPLETE within their declared scope. The gaps below are in surfaces **neither defense covers** — the app-level run holders (not in `FormState`) and on-screen widget cleartext.

---

## Candidate FOLLOWUP slugs

### 1. `gui-last-run-result-argv-stdout-not-zeroized` — NEW — **Med** (in-RAM only)

- **Secret type / gap class:** assembled argv secret-value tokens (seed phrase / passphrase / xprv / WIF / minikey / ms1 slot value) + captured stdout/stderr — **gap class 1 (in-RAM model residue) + 6 (bare `Vec` no zeroize-on-drop)**.
- **Where:** `src/main.rs:104` `last_run: Option<runner::RunResult>`; `src/runner.rs:18-31` `RunResult { argv: Vec<String>, mask: Vec<bool>, stdout: Vec<u8>, stderr: Vec<u8>, … }`; stored at `src/main.rs:1223` (`app.last_run = Some(result)`).
- **WHAT:** after a secret-bearing Run, the FULL assembled argv — **with the secret value tokens in cleartext** (`--passphrase <seed>`, `--from phrase=<seed>`, `@N.phrase=<seed>`) — plus `stdout`/`stderr` bytes live in `app.last_run`. `RunResult` is a bare struct (no `Zeroize`/`Drop`). The exit sweep (`on_exit` → `zeroize_form_state`) walks **only `self.form_state`**; it never touches `last_run`, so these secret bytes are **never scrubbed in RAM** at exit. The `mask` field governs DISPLAY only — the underlying `argv` bytes are cleartext.
- **Severity: Med.** In-RAM only (never written to disk; `RunResult` is not `Serialize`). Same residue class as M9 but for an app holder M9 didn't reach. (stdout *could* carry secret-class output for some subcommands, e.g. an `ms1`-emitting flow — unverified here; conservatively treated as in-RAM-only.)
- **WHY:** M9 brought `state.tree` to exit-sweep parity with values/slots/positionals; this is the symmetric gap one layer up — the run-result holder that sits OUTSIDE `FormState` and so is structurally invisible to `zeroize_form_state`.

### 2. `gui-pending-confirm-argv-not-zeroized` — NEW — **Med** (in-RAM only)

- **Secret type / gap class:** the real assembled argv (secret value tokens, cleartext) + `spec_stdin: Option<Vec<u8>>` held while the run-confirm modal is open — **gap class 1 + 6**.
- **Where:** `src/main.rs:114` `pending_confirm_argv: Option<PendingConfirm>` (= `(argv: Vec<String>, mask, stdin: Option<Vec<u8>>)`, alias `src/main.rs:1192-1193`); set at `src/main.rs:1045`; cleared by `= None` at `:1092` / `:1096` (plain drop, no zeroize).
- **WHAT:** the run-confirm modal exists **because** a secret-bearing argv is pending; that pending argv (and any tree spec_stdin) holds the cleartext secret in app state until Run/Cancel. On dismiss it is dropped via `= None` with no zeroize; it is also never visited by the exit sweep (not in `FormState`). Same structural blind spot as #1.
- **Severity: Med.** In-RAM only. Window of residence is bounded (modal lifetime) but the drop is non-scrubbing and exit during a pending modal leaves it un-swept.
- **WHY:** sibling of #1 — the two app-level holders that carry the *real* (unmasked) argv are the exact pair M9's `FormState`-scoped sweep cannot see; worth a single "sweep the app-level secret holders on exit + scrub-on-clear" cycle covering both.

### 3. `gui-composite-secret-value-rendered-cleartext-onscreen` — NEW — **Low/Med** (on-screen, no disk)

- **Secret type / gap class:** `NodeValueComposite` value for a secret node (`phrase`/`entropy`/`xprv`/`wif`/`minikey`/`bip38`/`electrum-phrase`/`seedqr`) — **gap class 2-adjacent (widget renders the secret unmasked) + 5 (on-screen reveal)**.
- **Where:** `src/form/widget.rs:653` — `let response = ui.text_edit_singleline(value);` in the `NodeValueComposite` arm. The in-source comment (`:649-651`) states outright: *"The value field is a plain (non-password) text edit."*
- **WHAT:** `convert --from phrase=<seed>` / `--from xprv=<key>` / `--from minikey=<key>` etc. render the typed/pasted secret **in cleartext on screen** (no `.password(true)`). v0.45.0 (H3) and v0.39.0 added paste-warn + Preview-label masking + persist-redaction + run-confirm for these nodes — but the **widget itself still shows the secret**. Contrast: secret Text flags route to `SecretLineEdit` (`.password(true)`), and secret slot values are `.password(true)` since v0.38.0. The composite value field is the lone secret-input widget that renders cleartext.
- **Severity: Low/Med.** On-screen reveal only — not persisted (redacted), not logged, masked in Preview/confirm. The exposure is shoulder-surf / OS-screenshot (Linux unmitigated per `gui-os-snapshot-secret-occlusion`). No disk/clipboard/log persistence.
- **WHY:** the resolved `composite-paste-warn-parity` note claims "the v0.39.0 masking already covers the DISPLAY of these composites" — but that masking is only the **Preview label**, not the live `text_edit_singleline`. This is the on-screen-widget facet that note left open. Closing it = render the composite value `.password(true)` when `node_type_is_argv_secret(node)` (mirrors slot v0.38.0).

### 4. `gui-tree-key-field-rendered-cleartext-onscreen` — NEW — **Low** (on-screen, no disk; non-canonical use)

- **Secret type / gap class:** xprv/WIF/raw-hex private key mis-typed into a `TreeNode.key`/`.keys[i]` build-descriptor field — **gap class 2-adjacent + 5**.
- **Where:** `src/form/tree_form.rs:697` (`ui.text_edit_singleline(&mut node.key)`) and `:717` (`ui.text_edit_singleline(&mut node.keys[i])`) — both plain (non-password). `xprv_hint` (`:785`) shows an amber "looks like an extended PRIVATE key … will not be saved and the gate will refuse it" warning but does not mask.
- **WHAT:** the build-descriptor tree is **watch-only by design** (keys are XPUBs); the gate refuses private keys and persist blanks them (M9 + redactor). But if a user pastes an xprv anyway, it renders cleartext on screen until exit. M9 zeroizes it from RAM on exit and the redactor blanks it on persist — but the live widget shows it, and (as M9's deferred residue `gui-tree-key-egui-undo-ring-residue` notes) the egui undo ring retains it.
- **Severity: Low.** On-screen only; non-canonical input (watch-only builder, gate-refused, persist-blanked, RAM-zeroized). Lowest of the four; arguably WONTFIX given the amber hint + watch-only contract. Filed for completeness / dedup against the undo-ring entry.
- **WHY:** the watch-only contract makes this an edge case, but it is the same "secret-input widget renders cleartext" class as #3; bundling them (`.password(true)` when `is_xprv_like`) would be cheap. NOT a duplicate of `gui-tree-key-egui-undo-ring-residue` (that is the undo-ring residue; this is the live-widget cleartext render).

---

## Considered and NOT filed (verified covered or out-of-class)

- **Clipboard copy of secret argv (no clear/expiry).** `src/main.rs:1030/1036` `ctx.copy_text(argv_posix/argv_windows)` copies the REAL command incl. secret value tokens to the OS clipboard, no clear/expiry. **Deliberate, documented design** (buttons relabeled "— reveals secret" v0.39.0; `PASTE_WARN_MODAL_TEXT` warns about clipboard managers). egui/eframe has no clipboard-clear API and OS clipboard-history is the OS's domain — same accepted-residue posture as the allocator/OS-snapshot caveats. Not a NEW finding; if filed at all it would be a doc-only "no clipboard auto-clear" note, but the reveal is informed-consent by design. **Left unfiled** (judgment: accepted design, not a regression).
- **`TaggedOrIndexedValue::Tag(String)` not zeroized in exit sweep** (`src/schema/mod.rs:556`; `zeroize_form_state` skips it). Tags are cosigner labels / `@N` indices — **not secret-class**. No finding.
- **`last_run_error: Option<String>`** (`src/main.rs:105`) = `e.to_string()` of an io/spawn error — no argv/secret bytes (runner argv-debug leak already fixed, `gui-runner-debug-logs-unmasked-secret-argv` RESOLVED v0.45.0). No finding.
- **`last_saved_snapshot: Option<String>`** (`src/main.rs:139`) holds the **redacted** serialization (`serialize_redacted` → `redact_persisted_state`) — no secret. No finding.
- **Debug/log/panic scan** (all non-test `.rs`): runner spawn log emits `program`/`argv_len`/`stdin` only (fixed v0.45.0); `panic!`/`expect` messages are on static infra (regex/schema/spawn), no secret values; `platform.rs` warns log handle `{:?}` (a window handle, not secret). No NEW log/panic leak.
- **`*.tmp` atomic-write strand** (`persistence.rs::write_atomic`) — the temp body is the **redacted** serialization (`save`/`save_if_changed` both call `serialize_redacted` first). No secret in the strand. No finding.
- **Tree-mode spec-JSON copy unmasked** — already filed (`tree-mode-posix-pipeline-spec-json-unmasked`, open; benign-today watch-only). NOT re-filed.

---

## Summary table

| # | slug | class | severity | new? | disk/clip/log? |
|---|------|-------|----------|------|----------------|
| 1 | `gui-last-run-result-argv-stdout-not-zeroized` | in-RAM residue (app holder) | Med | NEW | no (RAM only) |
| 2 | `gui-pending-confirm-argv-not-zeroized` | in-RAM residue (app holder) | Med | NEW | no (RAM only) |
| 3 | `gui-composite-secret-value-rendered-cleartext-onscreen` | on-screen widget cleartext | Low/Med | NEW | no (screen only) |
| 4 | `gui-tree-key-field-rendered-cleartext-onscreen` | on-screen widget cleartext | Low | NEW | no (screen only) |
