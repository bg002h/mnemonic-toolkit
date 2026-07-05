# gui_example tutorial — Leg 1 (GUI leg) post-implementation whole-diff review

**Scope:** `mnemonic-gui` branch `feat/gui-example-leg1`, tip `cf05123`, off master
`0d4429d` (PR #30). Whole-leg diff `git diff 0d4429d..cf05123` — 171 files,
+5133/−1223. Independent adversarial review; gates re-run with REAL tool calls
(not trusting phase reports). Reviewer: opus architect. Date: 2026-07-05.

---

## VERDICT: **GREEN — merge + tag `mnemonic-gui-v0.56.0` CLEARED.**

**0 Critical / 0 Important.** 2 Minor (cosmetic/traceability, non-blocking).
Full `cargo test --jobs 2` suite GREEN (exit 0); every gate re-run locally under
`WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 GUI_TUTORIAL_SNAPSHOTS=1`. Behavior
preserved, full-corpus secret hygiene clean, Examples-faithful, byte-deterministic,
CI gate sound on all four escape classes.

---

## 1. Behavior-preservation ruling — **PASS (pure relocation + the one intended F1 change).**

The `src/` diff is exactly 4 files. The extraction (`main.rs` → `app_window.rs`
+ `lib.rs`) is behavior-preserving; I did not trust the module docstring's
"pure relocation" claim — I mechanically diffed the moved code:

- **`lib.rs`**: adds `#[cfg(feature = "gui")] pub mod app_window;` — correctly
  gated identically to the egui form-widget modules (a `--no-default-features`
  build will not pull the egui stack). Nothing else.
- **Struct + field set**: field set byte-identical modulo visibility. Exactly
  **5 fields became `pub`** (`app_state`, `active_subcommand`, `form_state`,
  `last_run`, `last_run_error`) — the harness read/seed set the doc names.
  Visibility is not behavior.
- **Constructor split (seam 1)**: old `new()` = OS effects (window-capture
  protection → 1 Hz wayland-keepalive thread → SIGINT/SIGTERM handlers) →
  `AppState::detect_all()` → field init. New `new()` = **same OS effects in the
  same order** → `detect_all()` → `Self::new_headless(app_state, loaded,
  state_path)`; `new_headless()` = the pure field-init tail verbatim. The
  demo-seed block, the show_* toggle logic, `restore_selections`, and the final
  `Self { … }` are byte-identical between old `new` and new `new_headless`. No
  OS side-effect leaked into the headless path; execution order preserved.
- **`update` → `ui` (seam 2)**: path-normalized diff of old `fn update` body
  (`main.rs:385-1144`) vs new `fn ui` body (`app_window.rs:360-1119`) is EMPTY
  (the lone reported line is my range's trailing brace). `impl eframe::App::update`
  now reduces to `self.ui(ctx)`. The `_frame` param was already unused → faithful.
- **`on_exit` + helpers tail**: identical except **one added `///` doc-comment**
  on `spawn_and_capture` documenting the synchronous-completion / populated-pane
  contract. Documentation only — zero code delta.
- **`crate::` ↔ `mnemonic_gui::` path rewrites** are the required consequence of
  moving code from the binary into the library; not a behavior change.

**F1 (`schema/mnemonic.rs`) — the ONLY intended behavior change, correct:** a new
`EXPORT_WALLET_TEMPLATES` const = the 10 shared `TEMPLATES` values IN ORDER **plus
a trailing `""`** (sentinel APPENDED, opts[0] stays `bip44`); `--template.kind`
switches to `Dropdown(EXPORT_WALLET_TEMPLATES)`; **`default_value` stays `None`**
(a `Some("bip44")` would trip `is_at_default` suppression — correctly avoided).
Shared `TEMPLATES` is untouched, so bundle/verify-bundle + the SINGLE_SIG/MULTISIG
partitions keep 10 (the intended bundle-10 vs export-wallet-11 asymmetry).
`pinned_version` bumped `mnemonic 0.74.0` → `mnemonic 0.75.0`. This exactly
matches the F1 mini-R0 verdict ("A1 — APPEND the sentinel (opts[0] stays bip44)").

**F1 census cell re-verified:** `gui_form_snapshots` 2/2; the export-wallet form
PNG (`tests/snapshots/forms/mnemonic-export-wallet.png`) is **NOT in the leg diff**
(0 form-census PNGs changed) — virgin default stayed `bip44`, so the 61-form
gallery is byte-stable. `export_wallet_template_none` 5/5 (incl.
`virgin_default_stability_append_pin`); `gui_render_emit` 15/15 (export-wallet
exact-ASCII pin, the redundant append-placement guard).

## 2. Full-corpus secret-hygiene ruling — **PASS (first whole-corpus review; no leak).**

Corpus: 50 PNGs + 99 transcripts under `tests/snapshots/tutorial/`, fixtures under
`tests/tutorial/fixtures/`.

- **(a) Allowlist assert is non-vacuous and covers every secret step, all
  journeys.** `secret_allowlist_violations()` iterates ALL manifest steps × drives
  and classifies via the **production** taxonomy (`slot_subkey_is_secret` /
  `node_type_is_argv_secret`), NOT a hand-list — so it cannot false-negative a
  secret it fails to recognize. Allowlist = {S0 (fp 73c5da0a), S1 (b8688df1),
  S2 (28645006)}, the published Examples phrases. Non-vacuity guarded
  (`secret_drive_count() > 0`) + taxonomy-reachability guarded. The single
  whole-manifest driver `gui_tutorial_snapshots()` calls `execute_step` for every
  step, and `execute_step` asserts, for **every** `is_secret()` step: `••••` mask
  sentinel present before Run; whole-tree no-plaintext; each secret argv token
  carries its display-mask bit; secret Run defers to the confirm modal (two-click)
  and the modal's token list carries no plaintext. Driven across J1/J2 (the only
  secret-bearing journeys).
- **(b) Transcripts + fixtures clean; masking is REAL in rendered pixels.** Grep of
  all transcripts/fixtures for extended private keys (`[xtyzuv]prv…`), `ms1` cards,
  and any ≥11-word lowercase phrase run → **empty**. Fixtures are watch-only by
  construction: `policy.desc` / `taproot.desc` / `taproot-4leaf.desc` / `policy.json`
  carry only origin-annotated **xpubs** (the pathological vault + taproot trees, fp
  73c5da0a/b8688df1/28645006) and the SHA256 hashlock — no xprv/tprv/wif/phrase.
  `fixtures_carry_no_secret_material` machine-asserts no allowlist phrase inlines a
  fixture. I spot-opened 4 secret PNGs: **j1-01-run** shows `--slot ••••` in the
  argv echo while stdout carries the correctly-derived `ms1`/`mk1`/`md1` cards (pin
  `mnemonic 0.75.0` visible); **j1-01-modal** masks the slot in the confirm dialog
  + preview; **j2-02-convert** masks the `--from ••••` composite yet stdout emits
  the correct `fingerprint: 73c5da0a`; **j2-07-all-seeds** masks three `--slot ••••`
  and emits the derived 2-of-3 multisig (ms1[0..2]/mk1[0..2]). A real secret fed →
  masked in the render → derived output still correct. CONFIRMED.
- **(c)** J1 SlotEditor phrase row (slot subkey), J2-convert composite `--from
  phrase=` (argv-node), and J2-07 multi-slot all mask. J3/J4/J5 are xpub-only
  (no secret drives) — consistent with watch-only fixtures.

## 3. Examples-fidelity spot-check — **PASS (real per-run values, byte-match).**

Cross-checked tutorial transcript outputs against
`mnemonic-toolkit/.examples-build/Examples.md`:

| Journey artifact | tutorial checksum | Examples.md |
|---|---|---|
| J2 single-sig device-0 fp | `73c5da0a` (j2-02) | matches (Examples device 0) |
| J2 multisig canonical | `#4wup4at0` (j2-04) | line 357/367 — byte-identical |
| J2 multisig restore (recv/change/combined) | `#y65a0dtg` `/0/*`, `#k0gfvz2t` `/1/*`, `#yjp7hj7w` `<0;1>/*` | lines 588/605/615/626 — byte-identical |
| J3 pathological vault canonical | `#4ld0crxa` (j3-10) | 4× match |
| J3 vault restore canonical | `#jgulue7j` (j3-13 stderr) | line 1133 — match |
| J4 taproot restore canonical | `#7cy3x3q9` (j4-17 stderr) | line 1271 — match |
| J4 NUMS taproot | `#k0lsap8u` (j4-nums stderr) | line 1460 — match |

The five stdout checksums with zero Examples.md match (`#9aqfmzqf`, `#qcjjdeln`,
`#qlw09dfg`, `#zznefv9a`, `#frmcw06g`) were run to ground: they are the `/0/*` and
`/1/*` **branch-split importdescriptors JSON** of the SAME canonical descriptors
that DO match (Examples tabulates the split JSON only for the simple J2 multisig,
not the vault/taproot). Faithful — not a divergent wallet. The **md1 chaining
produced REAL per-run values**: `ChainStore` (`HashMap<stem, Vec<String>>`) is
populated live from `run.stdout` via `parse_md1_chunks` (`gui_tutorial_snapshots.rs:519`)
and read by later `TypeMd1Chain` steps — NO committed fixture; loud-fails on empty
chunks. The restore reconstructs from md1+seed and lands byte-identical on Examples.

## 4. Byte-determinism (full corpus) — **PASS.**

Plain re-run: all **50/50** regenerated `.new.png` byte-identical (`cmp -s`) to
committed. Authoritative `UPDATE_SNAPSHOTS=1 … cargo test --test gui_tutorial_snapshots`
regen → suite GREEN (9/9, 47.3s) → `git status --short` **empty** (50 PNGs +
transcripts byte-stable). Note: local `gl`/llvmpipe and CI `vulkan`/lavapipe
software backends produce byte-identical output here (stronger than
threshold-tolerant).

## 5. `tutorial-snapshots` CI gate soundness — **SOUND on all four escape classes.**

Dual census is MANIFEST-DERIVED (`grep -c` off committed `manifest-stems.txt` = **50**
`.png` + **33** `.exit.txt`, matching disk), anchored by the always-run
`manifest_stems_regen_diff` (byte-verifies stems ↔ manifest) + `corpus_png_count_matches_manifest`.

- **(a) render diff → CAUGHT.** `tutorial-snapshot-suite` step byte-compares every
  shot (kittest threshold) + every transcript inside the harness; a diff fails.
- **(b) missing/extra file → CAUGHT.** `census` step: `.new.png` count == manifest
  `.png` count AND `.exit.txt` count == manifest run count (`test N -eq M`);
  disk-vs-manifest anchored by the two always-run tests.
- **(c) silently-skipped suite → CAUGHT.** kittest writes `<stem>.new.png` per shot
  even on a PASSING compare; a skip (env unset / adapter early-return) → 0 `.new.png`
  → `test 0 -eq 50` fails. `adapter_guard()` panics (not skips) without a CPU adapter.
- **(d) wrong-tier binary → CAUGHT.** `run_pinned_tier_version_gate()` probes
  `mnemonic --version` against `SCHEMA.pinned_version` (`mnemonic 0.75.0`) BEFORE any
  render/spawn; CI installs `pinned-upstream.toml [mnemonic].tag =
  mnemonic-toolkit-v0.75.0` (== Cargo.toml dep tag == schema pin — all consistent).

**Install-scope deviation (only `mnemonic`) — SOUND.** Every manifest Run step is
the Mnemonic tab (no non-Mnemonic tab exists in the manifest); `spawned_clis()`
derives from Run-step `tab.bin_name()` → `["mnemonic"]`. md/ms/mk are LIBRARY deps
of the toolkit, never subprocessed. NO path filter — fires on PR/master/tag (the
tag-push run is the Leg-2 provenance anchor).

## 6. 50-vs-51 honesty — **honest at the gate; stale in prose (Minor-1).**

The shipped corpus is **50** shots (manifest-stems + disk); one modal was trimmed
after the USER "keep all 51" decision. The GATE never hardcodes 51 — census is
manifest-derived (build.yml explicitly: "NEVER a hardcoded 50/51"). No doc asserts
51 as the gate count. However ~6 prose sites still say "51 shots"
(`gui_tutorial_snapshots.rs:69-70` incl. "all-51-shots corpus measured 27.1 MiB",
`manifest.rs:4`, `mod.rs:9,22,348`). Cosmetic drift, non-blocking.

## 7. Novel mechanisms — **sound.**

- **ChainStore carry:** live-fed from run stdout, in-memory, deterministic (§3/§4).
- **F1 `(none)` route:** exercised by `export_descriptor_text!` /
  `export_descriptor_fixture!` (`SelectDropdown{--template, ""}` releases the
  template/descriptor mutex, then `--descriptor`) — J5 export + engrave shapes.
- **F2 descriptor-text route-around:** engrave-shape bundle + export-wallet
  descriptor inputs ride `--descriptor` TEXT, not `--descriptor-file` (ruling 6);
  reaches the engrave shape with Examples-faithful output.

## 8. What's owed at the release commit (correctly ABSENT from this PR).

Confirmed NONE of these is wrongly present here:
- crate `version` bump `0.55.0 → 0.56.0` (Cargo.toml — currently **0.55.0** ✓) + Cargo.lock.
- `CHANGELOG.md` 0.56.0 entry (currently tops at 0.55.0 ✓).
- README self-pin refresh (if the GUI README pins its own version).
- admin-bypass push (branch protection) + tag `mnemonic-gui-v0.56.0` + verify the
  tag-run of `tutorial-snapshots` (provenance anchor) goes green post-tag.
- delete the `spike/gui-example-p0` branch.
- **Deferred, correctly NOT attempted here:** making `tutorial-snapshots` a
  *required* branch-protection check = the `gui-branch-protection-scope` item.

(The schema `pinned_version 0.74.0→0.75.0` IS correctly in this PR — it is the
pinned-toolkit dependency the harness spawns, not the GUI crate version.)

## Findings by severity

**Critical: 0. Important: 0.**

**Minor:**
1. **Stale "51 shots" prose** in ~6 sites while the shipped corpus is 50 (one modal
   trimmed post-USER-decision). Gate is manifest-derived (honest); prose overstates
   by 1, and the budget rationale docstring ("all-51-shots corpus measured 27.1 MiB")
   should be re-worded to the 50-shot reality. Doc-sync sweep, non-blocking.
   Cites: `tests/gui_tutorial_snapshots.rs:69-70`, `tests/tutorial/manifest.rs:4`,
   `tests/tutorial/mod.rs:9,22,348`.
2. **Restore-form single-sig `--template` papercut — traceability.** The tutorial
   documents (and routes around, byte-identically) a REAL pre-existing GUI papercut:
   the restore form materializes a single-sig `--template` that md1-mode restore
   rejects, so journeys select the wallet's multisig template (inert in md1 mode).
   Commit `1843abf` claims "+ restore-template FOLLOWUP", but I could not positively
   locate the entry in `mnemonic-gui/design/FOLLOWUPS.md` or the toolkit FOLLOWUPS.
   Verify the FOLLOWUP is actually filed. Out of scope for this leg (zero restore-form
   `src/` change); non-blocking. Cite: `tests/tutorial/manifest.rs:11-14`.

## Gates re-run (all GREEN, `--jobs 2`)

- Full suite `cargo test` — exit 0.
- `gui_tutorial_snapshots` 9/9 (47s); `gui_form_snapshots` 2/2; `export_wallet_template_none`
  5/5; `gui_render_emit` 15/15; `schema_mirror` 21/21.
- Byte-determinism: 50/50 PNGs `cmp`-identical; `UPDATE_SNAPSHOTS` regen → clean tree.

**Repo left clean** — regen restored, all `.new.png` deleted, `git status` empty.
