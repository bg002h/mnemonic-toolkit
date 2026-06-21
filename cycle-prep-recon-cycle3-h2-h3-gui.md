# cycle-prep recon — 2026-06-21 — cycle-3 (H2 + H3 GUI secret-leak findings)

Constellation bug-fix program, cycle-3. Two **D-secret-leak** findings, both in **mnemonic-gui** (the fix repo). Recon ONLY — no implementation, no source edits.

## Repo state (live SHAs at recon time)

| Repo | Path | Default branch | origin/master SHA | Local branch | Sync |
|---|---|---|---|---|---|
| **mnemonic-gui** (fix repo) | `/scratch/code/shibboleth/mnemonic-gui` | `master` | **`0b1e024`** (= hunt-time snapshot; no drift since) | `master` | up-to-date (0/0) |
| **mnemonic-toolkit** (dep) | `/scratch/code/shibboleth/mnemonic-toolkit` | `master` | **`c9168aac`** (cycle-2 shipped) | `feature/own-account-subset-search` | 9 ahead / 14 behind |

- GUI working tree clean (no untracked). Toolkit recon performed against `origin/master` bytes (local branch is unrelated feature work).
- **GUI `Cargo.toml` pins `mnemonic-toolkit` at tag `mnemonic-toolkit-v0.60.0`** (toolkit `origin/master` is now 0.62.0). All toolkit citations below verified at BOTH `origin/master` (`c9168aac`) AND the pinned tag `mnemonic-toolkit-v0.60.0`.

Drift expectation going in: GUI source identical to snapshot → H2 expected clean; H3's schema/mnemonic.rs line cite is the most likely to have drifted (large, fast-growing file).

---

## Per-finding verification

### H2 — runner logs the full UNMASKED argv to stderr at debug level

- **WHAT:** `run_with_stdin` emits `debug!(target: "mnemonic_gui::runner", argv = ?argv, …)`. With `--debug` or `RUST_LOG=…=debug`, any secret-bearing invocation (BIP-39 phrase / entropy / passphrase / WIF / minikey) prints the raw secret value verbatim to stderr (terminal / journald / log file). The runner is mask-oblivious at that point (`mask` is `Vec::new()` until the GUI caller overwrites it post-spawn), so the logged argv is the unmasked spawn argv.

- **Citations:**
  - `src/runner.rs:119` — `debug!(target: "mnemonic_gui::runner", argv = ?argv, stdin = stdin.is_some(), "subprocess spawn");` — **ACCURATE** (exact line 119, verbatim match).
  - `run_with_stdin` is the spawn fn (`src/runner.rs:106`) — **ACCURATE**; `run()` (`:81`) delegates to it (`:89`), so EVERY spawn path hits line 119.
  - Secrets ARE assembled into `argv` — **CONFIRMED**: `src/form/invocation.rs` `assemble_argv_with_secret_mask` (`:151`) pushes secret VALUE tokens into `argv` at four sources (secret Text flag, secret slot row `@N.subkey=value`, secret positional, secret-class `NodeValueComposite`); the parallel `mask: Vec<bool>` exists PRECISELY because argv carries plaintext secrets. The runner discards/ignores that mask for the `debug!`.

- **Primary-intent check (`--debug`/`RUST_LOG` actually enables `debug!`):** **CONFIRMED** at `src/main.rs`: `init_tracing(cli.debug)` (`:39`); `init_tracing` (`:73`) sets `default_filter = if debug_flag { "debug" } else { "warn" }` (`:77-80`), then `EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter))` (`:82-83`) → `tracing_subscriber::fmt()…try_init()` (`:84-87`). So `--debug` sets the GLOBAL filter to `debug` and `RUST_LOG` overrides it when set — either path enables the runner `debug!`. The `--debug` doc-comment (`:31-32`) confirms intent.

- **REPRODUCES verdict:** **YES — reproduces on current source (`0b1e024`).** `mnemonic-gui --debug` + any secret-bearing run → the secret value appears verbatim on stderr. Zero drift.

- **Action for spec:** Drop the raw-argv leak at `runner.rs:119`. Cleanest options: (a) log only `argv.len()` and `argv[0]` (binary name, non-secret); or (b) thread the assembly-time mask into the runner and log a masked argv; or (c) drop the `argv = ?argv` field entirely (keep `stdin`, `"subprocess spawn"`). Option (a) is lowest-risk and matches the existing mask sentinel philosophy. NOTE: the exit-code `debug!`/`warn!` lines (`:166-168`) and the stdin-write `warn!` (`:144`) do NOT log argv — leak is isolated to line 119. Cite source SHA `0b1e024`.

---

### H3 — `convert --from minikey=<key>` leaks a Casascius mini PRIVATE KEY (no argv mask, no run-confirm, no paste-warn, AND plaintext persisted to `state.json`)

- **WHAT:** The GUI classifies node secrecy via the NARROW `mnemonic_toolkit::secret_taxonomy::SECRET_NODE_TYPES`, which **excludes `minikey`**. The toolkit exposes a WIDER `SECRET_NODE_TYPES_ARGV` (= narrow set + `minikey`) for exactly this argv-leakage/redaction surface, but the GUI does not import it. Consequence for `--from minikey=<key>`: (1) composite argv mask bit = `false` → unmasked in copy-command and last-run display; (2) `should_confirm_run` returns false → no run-confirm; (3) paste-warn does not fire; (4) `redact_for_persistence` does NOT drop it → the mini private key is written PLAINTEXT to `state.json` via autosave + on_exit.

- **Citations:**
  - **toolkit `secret_taxonomy.rs` — `SECRET_NODE_TYPES` vs `SECRET_NODE_TYPES_ARGV` re `minikey`:** **ACCURATE + CRITICAL CONFIRMATION.** At `origin/master` (`c9168aac`) AND at pinned tag `mnemonic-toolkit-v0.60.0`: `SECRET_NODE_TYPES` (line 76) = `[phrase, entropy, xprv, wif, ms1, bip38, electrum-phrase, seedqr]` — **NO minikey**. `SECRET_NODE_TYPES_ARGV` (line 95) = the same 8 PLUS `"minikey"` (line 104). Doc-comment (`:88-94`) states explicitly: "Downstream argv-redaction consumers (e.g. a GUI run-confirm preview) should use THIS set, not the narrower `SECRET_NODE_TYPES`." Introduced by toolkit `4ecb8df0` (`feat(secret-taxonomy): promote pub const SECRET_NODE_TYPES_ARGV`), which is an **ancestor of tag v0.60.0** → present at the GUI's pin.
  - `src/secrets.rs:160` — `pub fn node_type_is_secret(node: &str) -> bool { SECRET_NODE_TYPES.contains(&node) }` — **ACCURATE** (line 160-161). Backed by the NARROW set.
  - `src/secrets.rs:34` — `pub use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS};` — **ACCURATE** (line 34). The GUI imports ONLY the narrow set + slot subkeys; it does NOT import `SECRET_NODE_TYPES_ARGV`.
  - `src/form/invocation.rs:457` — composite mask bit `flag_is_secret(flag) || node_type_is_secret(node)` — **ACCURATE** (the `node_type_is_secret(node)` call is exactly at line 457, inside the `NodeValueComposite` `mask.push(...)`). Because both disjuncts are false for `--from minikey=…` (flag `secret:false`, node not in narrow set) → `mask.push(false)` → leaks.
  - `src/persistence.rs:94-99` — `redact_for_persistence` NodeValueComposite drop — **ACCURATE** (comment "Drop secret-class NodeValueComposite entries." at :94; the `if SECRET_NODE_TYPES.contains(&node.as_str()) { return false; }` guard at :96). Uses the NARROW set → `minikey` composite NOT dropped → persisted.
  - `src/main.rs:362-366` — autosave path: `build_persisted_state` (`:348`) maps each form_state through `persistence::redact_for_persistence(v)` (`:365`) — **ACCURATE** (range :362-366 lands on the `.map(...redact_for_persistence...)` call).
  - `src/main.rs:1138` — on_exit persist: `on_exit` (`:1121`) → `self.build_persisted_state()` (`:1137`) → `persistence::save(&persisted, path)` (`:1138`) — **ACCURATE**. Both autosave (timer, `:408-417`) and on_exit (`:1136-1141`) persist via the same redactor, so the narrow-set gap leaks on both paths.
  - `src/schema/mnemonic.rs:912-921` — the `convert --from` composite (`NodeValueComposite`, `secret:false`) + the false "per-row paste-warn fires" comment — **DRIFTED-by-~202.** At the cited lines :912-921 the file now holds `--cosigner` / `--search-address` (newer `bundle` flags). The ACTUAL `convert --from` composite is at **`:1114-1121`**: `name: "--from"` (:1115), `kind: FlagKind::NodeValueComposite(NODE_TYPES)` (:1116), `secret: false, // secrecy is value-dependent; per-row paste-warn fires` (:1119). `NODE_TYPES` (defined `:140-154`) DOES include `"minikey"` (`:151`), so the GUI dropdown OFFERS minikey as a `--from` node.

- **Primary-intent check (`SECRET_NODE_TYPES` vs `_ARGV` membership re `minikey`):** **CONFIRMED** by reading the toolkit `secret_taxonomy.rs` directly (see first citation). `minikey` ∈ `SECRET_NODE_TYPES_ARGV` (line 104) and ∉ `SECRET_NODE_TYPES`. The narrow/wide split is intentional and documented; the wide set is the correct one for the GUI's argv/persist/confirm/paste surfaces.

- **Extra finding — the "per-row paste-warn fires" comment is false MORE broadly than the FOLLOWUP states.** Paste-warn is gated by `should_warn_on_paste` (`secrets.rs:194`) = `flag_is_secret(flag) && paste_len ≥ THRESHOLD`. It is **flag-level, never node/per-row-aware.** For `--from` (flag `secret:false`, name not in `SECRET_FLAG_NAMES`), `flag_is_secret` is false, so paste-warn fires for NO `--from` node value — not just minikey. So the comment at schema `:1119` is wrong for every secret node typed into `--from` (phrase, xprv, wif, …), though those nodes ARE still caught by the OTHER three surfaces (argv mask, run-confirm, persist-redact) via `node_type_is_secret`. minikey is the one node that falls through ALL of them. The spec should either (a) make paste-warn node-aware (preferred — fold the wide set into `should_warn_on_paste` for composite flags), or (b) correct the comment to reflect the real flag-level behavior. Recommend (a) so the fix actually delivers paste-warn for minikey as the FOLLOWUP fix-shape promises.

- **REPRODUCES verdict:** **YES — reproduces on current source (GUI `0b1e024`, toolkit pin v0.60.0).** `convert --from minikey=<key>` is unmasked in argv/copy, skips run-confirm, never paste-warns, and persists plaintext to `state.json`. The wider taxonomy constant the fix needs is already available at the pin.

- **Action for spec:** GUI-only fix. Import `mnemonic_toolkit::secret_taxonomy::SECRET_NODE_TYPES_ARGV` into `secrets.rs`, and route the four argv-facing surfaces through the WIDE set: (1) `node_type_is_secret` callers in the **argv-mask** (`invocation.rs:457`), (2) **`should_confirm_run`** (`secrets.rs:230`), (3) **paste-warn / copy-reveal** (`should_warn_on_paste` — make composite-flag paste node-aware against the wide set), and (4) **`redact_for_persistence`** (`persistence.rs:96`). Design note: `node_type_is_secret` is shared by both persist-redaction and argv surfaces — either add a second predicate `node_type_is_argv_secret` backed by `SECRET_NODE_TYPES_ARGV` (cleanest, keeps the persist semantic explicit) or widen the single predicate (simpler, but be sure persist intentionally wants the wide set too — it does: a persisted minikey is a plaintext private key, so persist SHOULD drop it). Add a drift-guard test mirroring the `v0_3_canonical_fallback` supply-chain pattern at `secrets.rs:38-94` (snapshot `SECRET_NODE_TYPES_ARGV` byte-equal to the toolkit import; assert `minikey` ∈ wide, ∉ narrow). Correct the false comment at schema `:1119`. Cite source SHAs: GUI `0b1e024`, toolkit `c9168aac` (+ pin tag `mnemonic-toolkit-v0.60.0`).

---

## Cross-cutting observations

1. **Toolkit pin verdict — H3 is GUI-ONLY, NO pin bump required.** The GUI pins `mnemonic-toolkit-v0.60.0`, and `SECRET_NODE_TYPES_ARGV` (with `minikey`) is present at that tag (introducing commit `4ecb8df0` is an ancestor of v0.60.0). The fix imports an already-available constant. (A pin bump to 0.62.0 is NOT needed for cycle-3 and should be kept out of scope to minimize blast radius.)

2. **`schema_mirror` lockstep — NOT implicated.** The H3 fix touches mask/redaction/confirm/paste logic + a comment + a drift-guard test. It adds/removes/renames NO clap flag and NO dropdown VALUE: `minikey` already lives in `NODE_TYPES` (schema `:151`) and is already an offered `--from`/`--to` value. The GUI `schema_mirror.yml` gate (`tests/schema_mirror.rs`) gates flag-NAMES + dropdown values only → no paired-PR concern, no toolkit `gui-schema` change. H2 likewise touches only `runner.rs` logging → no schema surface. **Confirmed: no `schema_mirror` lockstep for either finding.**

3. **No `cargo fmt` CI gate on the GUI (per project memory) — do NOT run `cargo fmt` on the GUI during impl.** Verified against `origin/master` workflows: `build.yml` has a `clippy --all-targets -D warnings` gate and `schema-mirror.yml` runs `cargo test --test schema_mirror`; there is NO rustfmt/fmt step in any GUI workflow. Hand-format edits to match surrounding style; rely on clippy, not fmt.

4. **Drift summary:** H2 = zero drift (all cites ACCURATE on an unchanged GUI tree). H3 = one DRIFTED-by-~202 cite (schema `:912-921` → real `:1114-1121`); all other H3 cites (secrets.rs, invocation.rs, persistence.rs, main.rs ×2, toolkit secret_taxonomy.rs) ACCURATE. No STRUCTURALLY-WRONG citations.

5. **H2 and H3 are independent** (different files: `runner.rs` vs `secrets.rs`/`invocation.rs`/`persistence.rs`/schema). No inter-finding ordering dependency. Both are funds-adjacent secret-leak class (D-severity).

6. **The H3 paste-warn finding is broader than the FOLLOWUP scoped** (see Extra finding): the false comment misrepresents paste-warn for ALL `--from` nodes, not just minikey. The spec should explicitly decide whether to make composite-flag paste-warn node-aware (recommended) vs. only correct the comment, because the FOLLOWUP fix-shape lists "paste-warn" as a target surface for minikey and the current code cannot deliver it without a node-aware paste path.

---

## Recommended cycle-3 scope

**Single cycle, both findings together** (both GUI-only, both D-secret-leak, independent, small). Group H2 first (isolated 1-line logging change), H3 second (taxonomy widening across 4 surfaces + comment + drift-guard test).

- **H2** — ~1-5 LOC in `src/runner.rs` (replace `argv = ?argv` with `argv_len`/`argv[0]` or drop the field). + 1 regression test asserting the debug log does not contain a planted secret token. ~15-30 LOC w/ test.
- **H3** — import `SECRET_NODE_TYPES_ARGV`; add a wide-set predicate (`node_type_is_argv_secret`) or widen `node_type_is_secret`; route argv-mask / `should_confirm_run` / paste-warn / `redact_for_persistence` through it; make composite-flag paste node-aware; correct the schema `:1119` comment; add the supply-chain drift-guard test (snapshot + byte-equal + minikey membership). ~40-80 LOC across `secrets.rs`, `invocation.rs`, `persistence.rs`, `schema/mnemonic.rs` (comment only) + tests.
- **Total:** ~60-110 LOC. Small cycle.

- **SemVer:** **GUI MINOR** (its own version line; currently `0.44.0` → `0.45.0`). Secret-leak fixes are behavior changes worth a MINOR; no breaking API. **No toolkit version change** (pin stays at v0.60.0).
- **Locksteps:** `schema_mirror` — NOT triggered (no flag-name/dropdown-value change). Manual mirror (`docs/manual/`) — NOT triggered (no CLI surface change; the GUI manual surface is unaffected, and toolkit CLI is untouched). Sibling-codec FOLLOWUP companions — none (the toolkit constant is already shipped; no toolkit-side action).
- **CI gates to respect:** GUI `clippy --all-targets -D warnings` (must stay clean) + `schema_mirror` (will still pass — no schema delta). Do NOT `cargo fmt` the GUI.
- **Funds/safety framing:** both are private-key/seed-material exposure (stderr logs for H2; copy/confirm/paste/`state.json` plaintext for H3). Treat as funds-safety; full R0 gate + post-impl adversarial execution review apply.

### Mandatory next gate (project standard)
This recon FEEDS the R0 gate — it does not replace it. Any brainstorm spec / plan-doc for cycle-3 MUST pass an opus architect **R0 review to 0 Critical / 0 Important BEFORE any implementation** (fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN). No code, no implementer dispatch, no phase advance, no tag while any Critical/Important is open.
