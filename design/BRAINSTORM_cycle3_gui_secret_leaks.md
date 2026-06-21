# BRAINSTORM — cycle-3 — GUI secret-leak findings H2 + H3

Constellation bug-fix program, **cycle-3**. Two **D-secret-leak** findings, both
fixed in **`mnemonic-gui`** (no toolkit change). Formal brainstorm spec — DESIGN
ONLY, no code. Feeds the mandatory opus-architect **R0 loop to 0 Critical / 0
Important** before any implementation (per toolkit + GUI `CLAUDE.md`).

## Source-of-truth SHAs (verified live at write time)

| Repo | Path | `origin/master` SHA | Notes |
|---|---|---|---|
| **mnemonic-gui** (fix repo) | `/scratch/code/shibboleth/mnemonic-gui` | **`0b1e024`** | every citation below `git show origin/master:<path>`-verified |
| **mnemonic-toolkit** (dep) | `/scratch/code/shibboleth/mnemonic-toolkit` | `c9168aac` | the exposing crate; **NO change this cycle** |
| toolkit dep **pin** (GUI `Cargo.toml:42`) | — | tag **`mnemonic-toolkit-v0.60.0`** | `SECRET_NODE_TYPES_ARGV` verified present at this tag |

Inputs folded: `cycle-prep-recon-cycle3-h2-h3-gui.md` (verified citations +
REPRODUCES verdicts + GUI-only verdict); `design/agent-reports/constellation-bughunt-2026-06-20.md`
(H2 §93, H3 §111). All recon line-cites were **re-grepped against `origin/master`
`0b1e024` at write time** and are accurate except where corrected below.

---

## Finding summary (both REPRODUCE on `0b1e024` + pin v0.60.0)

- **H2** (`src/runner.rs:119`) — `run_with_stdin` emits
  `debug!(target: "mnemonic_gui::runner", argv = ?argv, …)`, Debug-formatting the
  **entire argv in cleartext**. Secret values (BIP-39 phrase / entropy / passphrase
  / WIF / minikey) are assembled INTO argv; `--debug` or `RUST_LOG=…=debug` enables
  the log → master secret printed verbatim to stderr (terminal / journald / file).
  The runner is mask-oblivious here (`RunResult.mask` is `Vec::new()` until the GUI
  caller overwrites it post-spawn), so the logged argv is the unmasked spawn argv.
- **H3** (`secrets.rs` / `invocation.rs` / `persistence.rs` / `schema/mnemonic.rs`)
  — the GUI classifies composite-node secrecy via the **narrow**
  `mnemonic_toolkit::secret_taxonomy::SECRET_NODE_TYPES` (the persistence-redaction
  set, which **excludes `minikey`**). The toolkit ships the wider
  `SECRET_NODE_TYPES_ARGV` (= narrow + `minikey`) for exactly the argv/preview/
  redaction surface, but the GUI never imports it. So `convert --from minikey=<key>`
  (a Casascius **mini PRIVATE KEY**) falls through FOUR surfaces: argv-mask
  (unmasked in preview / copy / last-run), `should_confirm_run` (no run-confirm),
  paste-warn (never fires), and `redact_for_persistence` (written **plaintext** to
  `~/.config/mnemonic-gui/state.json`, default 0644, surviving restarts).

**Class:** both D-secret-leak (private-key / seed-material exposure). Funds-safety.
Full R0 gate + mandatory post-impl adversarial execution review apply.

---

## H2 — runner argv-leak fix

### Locked facts (verified `0b1e024`)

- `src/runner.rs:119` — `debug!(target: "mnemonic_gui::runner", argv = ?argv, stdin = stdin.is_some(), "subprocess spawn");` (exact line, verbatim).
- `run_with_stdin` (`:106`) is the single spawn fn; `run()` (`:81`) delegates to it (`:89`), so EVERY spawn path hits `:119`.
- The exit-code logs (`:166-168`) and the stdin-write `warn!` (`:144`) do **not** log argv — the leak is isolated to `:119`.
- `RunResult.mask` is set to `Vec::new()` in `run_with_stdin` (`:160`) with the comment "runner stays mask-oblivious; the GUI caller (`spawn_and_capture`) overwrites this with the assembly-time mask." → **the assembly-time mask is NOT available inside the runner** at the `:119` call site.
- `--debug` → `init_tracing(cli.debug)` sets the global filter to `debug`; `RUST_LOG` overrides — either enables `:119` (recon CONFIRMED at `main.rs:39/73/77-87`).

### DECISION — H2 approach: **(a) drop the `argv = ?argv` field; replace with non-secret shape fields `argv_len` + `program`.**

The prompt offered (a) drop entirely, (b) log `argv[0]` + `argv.len()`, (c)
substitute the assembly-time mask. **Choose a hybrid of (a)+(b): drop the raw
`argv` field and replace it with `argv_len = argv.len()` + `program = ?argv[0]`,
keeping `stdin = stdin.is_some()` and the `"subprocess spawn"` message.**

```text
debug!(
    target: "mnemonic_gui::runner",
    program = %argv[0],          // binary path/name — never secret
    argv_len = argv.len(),       // token count — never secret
    stdin = stdin.is_some(),
    "subprocess spawn",
);
```

**Why not (c) — the mask substitution.** The recon establishes the runner is
**deliberately mask-oblivious**: `RunResult.mask` is `Vec::new()` at `:119` and the
GUI caller (`spawn_and_capture`) overwrites it only AFTER `run_with_stdin` returns.
Threading the assembly-time mask into `run_with_stdin` to render a `••••`-masked
argv would (1) change the `run_with_stdin` signature (a public fn with its own
tests + a SPEC §0/§2.1 contract about argv/stdin handling), (2) re-introduce a
secret-adjacent code path into the layer whose explicit design invariant is "stays
mask-oblivious", and (3) risk a mask-length/argv-length desync class inside the
runner. Option (c) is *more* surface for *less* benefit: a masked argv preview
already exists at the GUI display layer (`render_copy_command_masked`, v0.39.0) —
the runner debug log does not need to reproduce it. `program` (the binary) +
`argv_len` give all the operationally useful debug signal (which CLI spawned, how
many tokens) with **zero secret bytes** and **no signature change**.

`program = %argv[0]` is safe: `argv[0]` is always the resolved binary
path/name (recon: `Command::new(OsStr::new(&argv[0]))`), never a secret token.

**Blast radius:** one `debug!` macro call, `runner.rs` only. No public signature
change. No SPEC contract touched.

### H2 regression test (TDD — RED first)

Add to the `runner.rs` `#[cfg(test)] mod tests` (or a `tests/runner_no_argv_leak.rs`):
capture tracing output via a `tracing_subscriber` test layer (the repo already
inits tracing in tests — see `cell_2_tracing_init_logs_subprocess_spawn` referenced
in the GUI FOLLOWUPS `runner-tracing-test-flaky-under-parallel-load`), run
`run_with_stdin` with a **planted sentinel secret** token in argv (e.g.
`["echo", "SENTINEL_SEED_abandon_abandon_…"]`), and assert BOTH:
1. **(MANDATORY positive control — load-bearing, not an aside)** the captured `debug`
   output DOES contain the new `argv_len` field (proves the `:119` event was actually
   captured by THIS test's subscriber). **Without this assertion the test is
   worthless** — see the vacuous-pass hazard below.
2. **(the actual leak assertion)** the captured output does NOT contain the sentinel
   substring.

Must be RED on the current `argv = ?argv` line (sentinel present → assertion 2 fails)
and GREEN after the fix (sentinel gone, `argv_len` present).

> **R0 I1 — vacuous-pass hazard (MANDATORY mitigation, not optional).** `tracing`
> caches per-callsite interest GLOBALLY (process-wide `Interest` cache). If another
> test (or an earlier run of this one) registered the `:119` callsite under a
> subscriber that did NOT enable `debug`, the cached `Interest::never()` makes the
> `debug!` macro a **no-op even inside a fresh `with_default(local_layer, …)` scope**
> → the local layer captures NOTHING → the negative assertion ("sentinel absent")
> passes **VACUOUSLY** (the existing `cell_2_tracing_init_logs_subprocess_spawn`
> flake proves exactly this mechanism). The fix therefore MUST:
> 1. Use a **scoped** subscriber: `tracing::subscriber::with_default(local_capture_layer, || …)` (NOT global `try_init`).
> 2. Call **`tracing::callsite::rebuild_interest_cache()`** at the start of the scope so the `:119` callsite is re-evaluated against the local layer (defeats a stale cached `never`).
> 3. Keep the **positive control (assertion 1) MANDATORY** — it is the only thing that distinguishes "captured + no sentinel" (real GREEN) from "captured nothing" (vacuous pass).
> 4. If captures still prove racy under `cargo test` parallelism, fall back to marking the test `#[serial]` (serial_test crate) — see resolved Q1. The positive control stays regardless.

---

## H3 — wider-set switch across 4 surfaces + node-aware paste

### Locked facts (verified `0b1e024` + toolkit tag v0.60.0)

Toolkit `secret_taxonomy.rs` @ tag `mnemonic-toolkit-v0.60.0` (verified byte-exact):
- `SECRET_NODE_TYPES` (`:76`) = `["phrase","entropy","xprv","wif","ms1","bip38","electrum-phrase","seedqr"]` — **8 entries, NO `minikey`**.
- `SECRET_NODE_TYPES_ARGV` (`:95`) = the **same 8 PLUS `"minikey"`** (`:104`) — **9 entries**. Doc-comment: "Downstream argv-redaction consumers (e.g. a GUI run-confirm preview) should use THIS set, not the narrower `SECRET_NODE_TYPES`." Lockstep-pinned by toolkit's `secret_taxonomy_argv_parity_with_is_argv_secret_bearing` parity test.
- **Delta = exactly `{minikey}`** (confirmed by direct slice compare: the two arrays are identical for indices 0..8; `_ARGV` appends `minikey` at index 8). minikey is a **Casascius mini PRIVATE KEY** encoding — a secret.

GUI sites (all verified):
- `secrets.rs:34` — `pub use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS};` (imports narrow only; NOT `_ARGV`).
- `secrets.rs:160-162` — `pub fn node_type_is_secret(node) -> bool { SECRET_NODE_TYPES.contains(&node) }` (backed by NARROW).
- `secrets.rs:200-235` — `should_confirm_run`; its composite branch (`:228-232`) calls `node_type_is_secret(node)` at `:230`.
- `secrets.rs:194` — `should_warn_on_paste(flag, paste_len) = flag_is_secret(flag) && paste_len >= PASTE_WARN_THRESHOLD` — **flag-level only, no node parameter** (see node-aware paste decision below).
- `form/invocation.rs:455-458` — `emit_one` composite mask bit `mask.push(flag_is_secret(flag) || node_type_is_secret(node))` (the `node_type_is_secret(node)` call is at `:457`).
- `persistence.rs:31` imports `SECRET_NODE_TYPES`; `persistence.rs:94-99` — `redact_for_persistence` composite drop: `if SECRET_NODE_TYPES.contains(&node.as_str()) { return false; }` (NARROW → `minikey` NOT dropped → persisted).
- `main.rs:951` — the ONLY non-test caller of `should_confirm_run`. `main.rs` autosave (`:362-366`) + `on_exit` (`:1138`) both persist via `redact_for_persistence`.
- **schema `--from` composite + false comment — recon's `:1114-1121` is now `:1113-1122`** (re-verified live; ~1-line drift from the recon snapshot). `name: "--from"` (`:1114`), `kind: FlagKind::NodeValueComposite(NODE_TYPES)` (`:1115`), and the FALSE comment **`secret: false, // secrecy is value-dependent; per-row paste-warn fires`** at **`:1119`**. `NODE_TYPES` (`:140-154`) includes `"minikey"` (`:151`) → the GUI dropdown OFFERS minikey as a `--from` node.

Existing GUI taxonomy guards (relevant to the new drift-guard):
- `secrets.rs:37-99` — the `v0_3_canonical_fallback` mod (snapshot of the NARROW `SECRET_NODE_TYPES`) + the compile-time `const _: () = assert!(secret_slice_eq(SECRET_NODE_TYPES, …))` supply-chain drift guards (`:78-99`). Helper `secret_slice_eq` (`:103`) + `const_str_eq`.
- `tests/secret_taxonomy_pin.rs` — runtime min-membership pins on `SECRET_NODE_TYPES` / `SECRET_SLOT_SUBKEYS` (the "third layer of defense").

### Surface-by-surface fix

**Predicate design DECISION — add a SECOND predicate, do NOT widen the single one.**
Introduce `node_type_is_argv_secret(node: &str) -> bool { SECRET_NODE_TYPES_ARGV.contains(&node) }`
alongside the existing `node_type_is_secret` (which stays backed by the narrow set).
Rationale: the recon flagged the choice and the toolkit doc-comment is explicit that
the two sets are **intentionally distinct**. Keeping two named predicates makes each
call site self-documenting about *which* semantic it wants, and prevents a future
reader from "simplifying" the widened single predicate back and silently
re-narrowing persist. Add `SECRET_NODE_TYPES_ARGV` to the `secrets.rs:34`
re-export.

The four surfaces route as:

1. **argv-mask** (`invocation.rs:457`) → `node_type_is_argv_secret(node)` (WIDE). A
   `minikey` composite value now gets `mask.push(true)`, so it is redacted in the
   **Preview** display and the **last-run `argv:`** line (both rendered via
   `render_copy_command_masked`, v0.39.0), and the **copy button flips to the
   informed-reveal label** ("… — reveals secret"), bringing minikey to parity with
   `phrase`/`xprv`. **R0 I2 clarification (do NOT mis-implement):** the actual
   `ctx.copy_text` clipboard PAYLOAD stays **cleartext BY DESIGN** — the existing
   secret nodes (phrase/xprv/wif/…) copy the real command so the user can paste it;
   the mask drives the *visual* redaction + the reveal-warning label, NOT the
   clipboard bytes. The implementer/test MUST NOT attempt to redact the copy payload
   (that would break the intentional informed-reveal contract); the mask bit's job
   here is Preview/last-run redaction + the reveal label only.
2. **`should_confirm_run`** (`secrets.rs:230`) → `node_type_is_argv_secret(node)`
   (WIDE). A non-empty `--from minikey=…` now returns `true` → run-confirm modal
   fires.
3. **`redact_for_persistence`** (`persistence.rs:96`) → see DECISION (i) below.
4. **paste-warn** → see DECISION (ii) below (node-aware paste).

### DECISION (i) — `redact_for_persistence` uses the WIDE set (via `SECRET_NODE_TYPES_ARGV` directly), NO dedicated set.

**Over-redaction check (required by the prompt).** Switching the persistence
composite-drop from `SECRET_NODE_TYPES` to `SECRET_NODE_TYPES_ARGV` adds **exactly
one** node to the drop set: `minikey`. Verified the delta is `{minikey}` only (slice
compare above). `minikey` is a Casascius mini **private key** — it MUST NOT persist
to `state.json`. Therefore the wider set **adds nothing that should legitimately
persist**; every other entry is already dropped, and the one delta is correctly
dropped. **Conclusion: use `SECRET_NODE_TYPES_ARGV` directly for the persistence
composite-drop** (change `persistence.rs:96`
`SECRET_NODE_TYPES.contains` → `SECRET_NODE_TYPES_ARGV.contains`, and the import at
`:31`). A dedicated third "persistence set" is unnecessary — there is no node that
is argv-secret-but-persistence-safe, so the two sets coincide for persistence
purposes once `minikey` is added.

> Persistence semantics note for R0: the `redact_for_persistence` module doc
> (`persistence.rs:16,73`) and the inline comment (`:94` "Drop secret-class
> NodeValueComposite entries.") reference `SECRET_NODE_TYPES` — these doc strings
> must be updated to `SECRET_NODE_TYPES_ARGV` in lockstep so the doc does not drift
> from the code (a known recurring "stale comment" class in the bughunt themes).

### DECISION (ii) — node-aware paste-warn: **make composite paste-warn node-aware (the WIDE set), wiring detection into the composite value widget** — close the existing `composite-paste-warn-parity` FOLLOWUP.

**The recon's bonus finding is correct AND there is more structure than the recon
states.** Two facts:

1. The schema comment "per-row paste-warn fires" (`schema:1119`) is **false for
   every `--from` node**, not just minikey: `should_warn_on_paste` is `flag_is_secret
   && len≥THRESHOLD`, and `--from` has `secret:false` / is not in `SECRET_FLAG_NAMES`,
   so `flag_is_secret(--from)` is false → paste-warn fires for NO `--from` value.
2. **Verified structural fact (critical for scoping):** the composite VALUE field is
   rendered by a **plain `ui.text_edit_singleline(value)`** at `form/widget.rs:646`
   — it does NOT use `SecretLineEdit`. Paste DETECTION (the `egui::Event::Paste`
   capture + `paste_warn_id()` bus-flag raise) lives ONLY in
   `form/secret_widget.rs:85-105`, which renders only for secret Text fields.
   `should_warn_on_paste` (the public predicate) is **only exercised by tests**
   (`tests/widget_secret.rs`, `tests/secrets.rs`) — it is NOT on the live paste path;
   the live path is the bus flag inside `SecretLineEdit::show`. **Therefore the
   composite value field currently has NO paste detection at all** — a node-aware
   predicate swap alone cannot deliver paste-warn for minikey; the widget needs the
   detection wired in.

**This is the already-filed FOLLOWUP `composite-paste-warn-parity`** (GUI
`FOLLOWUPS.md:40-46`, surfaced 2026-06-11 / v0.39.0 R0 M1), whose documented fix is
exactly: "extend the paste-detection to the composite value widget (it could set the
same `secret_widget::paste_warn_id()` flag with the same `changed() + Event::Paste`
check)." Cycle-3 **resolves** this slug.

**Fix shape (node-aware):** in the `NodeValueComposite` arm of `widget.rs` (~`:644-646`),
replace the bare `ui.text_edit_singleline(value)` with a paste-detecting render that
mirrors `SecretLineEdit::show`'s detection (capture `egui::Event::Paste(s)` from the
focused response; on `response.changed()` with `len ≥ PASTE_WARN_THRESHOLD`), **gated
on the selected `node` being argv-secret-class** — i.e. raise `paste_warn_id()` ONLY
when `secrets::node_type_is_argv_secret(node)` is true. The existing
`update()` read-once `remove_temp` (`main.rs:1057`) already consumes the bus flag and
renders the modal — **no new modal wiring**, single chokepoint preserved.

**Over-warn check (required by the prompt).** Gating on `node_type_is_argv_secret(node)`
means paste-warn fires for a composite `--from` value ONLY when the selected node is
secret-class (phrase, entropy, xprv, wif, ms1, bip38, electrum-phrase, seedqr,
**minikey**). For non-secret `--from` nodes (`xpub`, `fingerprint`, `path`, `mk1`,
`address`) it does NOT fire → **no over-warn**. This is strictly correct: the warn
should track value secrecy, which for a composite is node-determined.

> **Scope-honesty note for R0:** this widget change is the single largest LOC item in
> the cycle and is genuine UI wiring (not a predicate swap). It is in-scope because
> the H3 FOLLOWUP fix-shape explicitly lists "paste-warn" as a minikey leak surface
> and the recon recommends option (a) "make paste-warn node-aware." The **sibling**
> slug `slot-field-paste-warn-uncovered` (slot `@N.phrase=` boxes) is the SAME class
> but a DIFFERENT widget (`slot_editor.rs`) and is **OUT of scope** for cycle-3
> (no minikey reaches a slot; minikey is `--from`-only) — leave it filed.
> **RESOLVED (R0 round 1, Q2): leave OUT** — verified no minikey funds-leak in the
> slot gap, so folding it would expand scope past the two confirmed funds-leaks.

> **Scope-correction (R0 round-1 Minor):** wiring paste detection into the composite
> value widget closes `composite-paste-warn-parity` for **all 9 composite nodes**,
> not just `minikey` — the detection fires on every `--from` composite value, gated
> by `node_type_is_argv_secret(node)` so the warn only RAISES for the secret-class
> nodes. The FOLLOWUP is fully resolved (the spec earlier under-claimed it as a
> minikey-only fix).

### Correct the false comment

`schema/mnemonic.rs:1119` — replace
`secret: false, // secrecy is value-dependent; per-row paste-warn fires`
with an accurate comment, e.g.
`secret: false, // secrecy is node-dependent; composite paste-warn + argv-mask + run-confirm + persist-redact key on node_type_is_argv_secret (cycle-3)`.
The `secret: false` field itself **stays** (the FLAG is not unconditionally secret;
secrecy is per-node) — only the comment changes. **No `schema_mirror` impact** (no
flag-name / dropdown-value change; see lockstep below).

### H3 tests (TDD — RED first)

1. **Drift-guard (supply-chain) test** — add to `tests/secret_taxonomy_pin.rs`
   (the established "third layer of defense" file, mirroring the existing
   min-membership pins). Assert: (a) the GUI's re-exported `SECRET_NODE_TYPES_ARGV`
   contains every entry of `SECRET_NODE_TYPES`; (b) `minikey ∈ SECRET_NODE_TYPES_ARGV`
   and `minikey ∉ SECRET_NODE_TYPES`; (c) the wide set's membership equals what the
   GUI classification uses (i.e. `node_type_is_argv_secret("minikey") == true`,
   `node_type_is_secret("minikey") == false`). This makes a future toolkit-side
   widening of `_ARGV` that the GUI fails to track a **test failure**, not a silent
   leak. **Also** extend the compile-time drift-guard pattern (mirroring the narrow
   set's `v0_3_canonical_fallback`): add a snapshot of `SECRET_NODE_TYPES_ARGV` + a
   `const _: () = assert!(secret_slice_eq(SECRET_NODE_TYPES_ARGV, snapshot))`
   so a pin bump that changes the wide set fails to compile (same belt-and-suspenders
   posture as the narrow set). **RESOLVED (R0 round 1, Q3): put the wide-set snapshot
   + `const _` assert in a NEW sibling mod** (e.g. `argv_canonical_fallback`), NOT in
   `v0_3_canonical_fallback` (slated for v0.5.0 deletion), so the wide-set guard
   outlives the v0.3.3 belt-and-suspenders retirement.
2. **Regression test (the four surfaces)** — a `--from minikey=<KEY>` form state:
   assert (i) `assemble_argv_with_secret_mask` marks the value token `mask = true`
   (masked in preview); (ii) `should_confirm_run` returns `true`; (iii) the
   node-aware paste path raises `paste_warn_id()` for an over-threshold paste of a
   minikey value (kittest, mirroring `tests/paste_warn_wiring_v0_40_0.rs`); (iv)
   `redact_for_persistence` DROPS the `--from minikey=…` composite (NOT written to
   the persisted state). Negative controls: a `--from xpub=…` composite is NOT
   masked, does NOT confirm, does NOT paste-warn, and IS persisted (xpub is
   watch-only).
3. **On-disk persistence coverage (R0 round-1 Minor)** — widen the existing
   `tests/persistence.rs:156` secret-node loop to iterate the WIDE set so `minikey`
   gets explicit round-trip coverage: assert a persisted state containing a
   `--from minikey=<KEY>` composite serializes with the value DROPPED (the on-disk
   `state.json` bytes do not contain the minikey), matching the other 8 secret nodes.

---

## SemVer / lockstep / version sites

### SemVer: **GUI MINOR `0.44.0 → 0.45.0`.** No toolkit version change.

Secret-leak behavior fixes (new masking / confirm / paste-warn / redaction
behavior) are a MINOR; no breaking API. **Toolkit pin stays at
`mnemonic-toolkit-v0.60.0`** — `SECRET_NODE_TYPES_ARGV` is already present at that
tag (introducing commit `4ecb8df0` is an ancestor of v0.60.0; recon-confirmed). A
pin bump to 0.62.0 is explicitly **out of scope** (minimize blast radius).

### `schema_mirror` — **NOT triggered.** Confirmed.

The H3 fix touches mask / redaction / confirm / paste logic + one comment + tests.
It adds/removes/renames **no clap flag** and **no dropdown VALUE**: `minikey`
**already lives** in `NODE_TYPES` (`schema:151`, verified) and is already an offered
`--from`/`--to` value. The `schema_mirror` gate (`tests/schema_mirror.rs`) +
`schema_mirror_secret_drift` (`tests/schema_mirror_secret_drift.rs`, which mirrors
per-flag `FlagSchema.secret`) check flag-NAMES + per-flag secret bools + dropdown
value enums — **none change** (`--from` stays `secret:false`; no node value added/
removed). H2 touches only `runner.rs` logging → no schema surface. **No paired-PR /
no toolkit `gui-schema` change.**

### Manual mirror (`docs/manual/`) — **NOT triggered.** No CLI surface change
(toolkit CLI untouched; the GUI is not in the toolkit manual's mirror set).

### Sibling-codec FOLLOWUP companions — **none.** The toolkit constant is already
shipped; no toolkit-side action. (No `Companion:` cross-cite needed.)

### `cargo fmt` — **MUST NOT run `cargo fmt` on the GUI** (project memory:
GUI has NO `cargo fmt` CI gate; recon re-verified no rustfmt step in any GUI
workflow). Hand-format edits to match surrounding style; rely on clippy.

### GUI version-site list (bump `0.44.0 → 0.45.0`)

| Site | Path | Gate |
|---|---|---|
| crate version | `Cargo.toml:3` `version = "0.44.0"` | — |
| lockfile | `Cargo.lock:2266` (`name = "mnemonic-gui"` block) | build |
| README install line | `README.md:42` `--tag mnemonic-gui-v0.44.0` | **`readme_pin_coherence`** asserts self-tag == `mnemonic-gui-v{Cargo.toml version}` → **MUST bump in lockstep or test FAILS** |
| CHANGELOG | `CHANGELOG.md` — prepend a `## mnemonic-gui [0.45.0] — <date>` entry | — |

**Do NOT touch** `schema/mnemonic.rs:4344` `pinned_version: "mnemonic 0.59.0"` — that
is the **TOOLKIT** display banner (no functional consumer; `schema_check.rs` reads
`pinned-upstream.toml`, not this string), and we are not bumping the toolkit pin.
*(R0 round-1 Minor: the banner string `0.59.0` vs the actual pin tag `v0.60.0` is a
**pre-existing** skew, NOT introduced by cycle-3 and explicitly OUT of scope — do not
"fix" it here; if it matters, file it separately.)*
**Do NOT touch** `pinned-upstream.toml` `[mnemonic].tag = "mnemonic-toolkit-v0.60.0"`
(no pin bump). `pin_coherence` / `readme_pin_coherence` stay green because the
toolkit tag is unchanged and the README's toolkit/md/ms/mk lines are unchanged; only
the `mnemonic-gui` self-line bumps with `Cargo.toml`.

### CI gates to respect
- `clippy --all-targets -D warnings` (must stay clean).
- `schema_mirror` (4 CLIs) + `schema_mirror_secret_drift` + `archetype_schema_mirror`
  + `gui_schema_conditional_drift` + `xpub_search_schema_mirror` — all stay green
  (no schema delta).
- `readme_pin_coherence` + `pin_coherence` — green only if the README self-tag
  bumps in lockstep with `Cargo.toml` (the one gate the version bump can trip).
- `secret_taxonomy_pin` (extended this cycle) + the new H3 drift-guard.

---

## Workstreams / concurrency

| WS | Files | Disjoint? |
|---|---|---|
| **H2** | `src/runner.rs` (+ a runner tracing test) | YES — touches no H3 file |
| **H3** | `src/secrets.rs`, `src/form/invocation.rs`, `src/form/widget.rs`, `src/persistence.rs`, `src/schema/mnemonic.rs` (comment), `tests/secret_taxonomy_pin.rs`, a new H3 regression test | YES — touches no H2 file |

**Assessment:** H2 and H3 are **file-disjoint** and have **no ordering
dependency** (recon §5 confirms). Per the GUI/toolkit `CLAUDE.md` per-phase pattern,
implementation is a **single subagent in a worktree, TDD** — concurrency is
*available* (the two are independent) but the cycle is small enough (~60-110 LOC
total) that a single sequential implementer (H2 first — isolated 1-line + test; H3
second — the taxonomy widening + widget wiring) is the recommended execution.
Version-site edits (`Cargo.toml` / `Cargo.lock` / `README.md` / `CHANGELOG.md`) are
done once at the end of H3, owned by the single implementer.

---

## FOLLOWUP slugs

- **H2 → `gui-runner-debug-logs-unmasked-secret-argv`** (bughunt id) — file/flip in
  the GUI `FOLLOWUPS.md` as RESOLVED in the shipping commit; tick the H2 checkbox in
  `design/agent-reports/constellation-bughunt-2026-06-20.md` (cite the fixing
  commit).
- **H3 → `gui-minikey-secret-not-masked-in-argv-preview`** (bughunt id; folds the
  Wave-3 `w3-gui-minikey-persist-plaintext` facet) — same RESOLVED treatment.
- **`composite-paste-warn-parity`** (GUI `FOLLOWUPS.md:40`) — **flip to RESOLVED**
  this cycle (the node-aware composite paste-warn closes it). Update its entry with
  the cycle-3 SPEC + commit.
- **`slot-field-paste-warn-uncovered`** (GUI `FOLLOWUPS.md:48`) — **stays OPEN**
  (out of scope; different widget, no minikey path). Note in cycle-3 record that it
  was considered and deferred. **RESOLVED at R0 round 1 (Q2): leave OUT** — no
  minikey funds-leak in the slot gap.

(Per project memory `feedback_followup_status_discipline`: verify "open" status at
decision time and flip status **in the shipping commit**, not lazily.)

---

## Resolved decisions (R0 round 1 — formerly open questions; "decisions before code")

All four are now CLOSED per the R0 round-1 review (`design/agent-reports/cycle3-spec-r0-round1-review.md`), which ratified each lean (Q1 with the I1 amendment). These are binding on the implementer; no open question remains at code time.

- **Q1 (H2 test isolation) — RESOLVED: scoped subscriber + `rebuild_interest_cache()` + MANDATORY positive control; `#[serial]` fallback.** Use `tracing::subscriber::with_default(local_capture_layer, || …)` (NOT global `try_init`); call `tracing::callsite::rebuild_interest_cache()` at scope entry so the `:119` callsite is re-evaluated against the local `debug`-enabled layer (defeats a stale cached `Interest::never` from another test → the vacuous-pass hazard, R0 I1); keep the positive control (`argv_len` present) assertion MANDATORY as the only discriminator between real-GREEN and captured-nothing. If still racy under parallel `cargo test`, mark `#[serial]` (serial_test). See the H2 regression-test section above for the full mitigation.
- **Q2 (slot paste-warn fold) — RESOLVED: leave OUT.** `slot-field-paste-warn-uncovered` (`slot_editor.rs`) stays filed/OPEN. Verified: `minikey` is `--from`-only and is never a `SlotSubkey`, so the slot gap carries NO minikey funds-leak — folding it would expand scope past the two confirmed funds-leaks for no funds-safety gain.
- **Q3 (wide-set compile-time guard home) — RESOLVED: new sibling mod.** Put the `SECRET_NODE_TYPES_ARGV` snapshot + `const _: () = assert!(secret_slice_eq(…))` drift-assert in a NEW sibling mod (e.g. `argv_canonical_fallback`), NOT inside `v0_3_canonical_fallback` (which is slated for v0.5.0 deletion) — so the wide-set guard outlives the v0.3.3 belt-and-suspenders retirement.
- **Q4 (predicate naming) — RESOLVED: `node_type_is_argv_secret`.** Parallels the existing `node_type_is_secret`, operates on `&str` (so the toolkit `NodeType::is_argv_secret_bearing` method name would mislead).

---

## Mandatory next gate (project standard)

This brainstorm spec MUST pass an opus-architect **R0 review to 0 Critical / 0
Important BEFORE any implementation** (fold findings → persist the review verbatim
to `design/agent-reports/` → re-dispatch until GREEN; the reviewer-loop continues
after every fold). No code, no implementer dispatch, no phase advance, no tag while
any Critical/Important is open. Post-implementation: mandatory independent
adversarial execution review over the whole diff.
