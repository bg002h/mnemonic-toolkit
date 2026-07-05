# `gui_example.pdf` tutorial — workaround / route-around audit

**Date:** 2026-07-05
**Instrument audited:** the `gui_example.pdf` GUI tutorial book shipped as
`mnemonic-gui-v0.56.0` (Leg 1) + `manual-gui-v1.2.0` (Leg 2). 25 shot-bearing
steps / 50 shots, Ch0 + J1-J5.
**Method:** swept the tutorial prose (`docs/manual-gui/tutorial/*.md`), the
tagged harness (`mnemonic-gui-v0.56.0:tests/tutorial/{mod,manifest}.rs`,
`tests/gui_tutorial_snapshots.rs`), the GUI runner
(`src/runner.rs`, `src/app_window.rs`, `src/form/*`), and both repos' FOLLOWUPS.
Analysis only — no code changed. The clone was deleted after the sweep.

---

## 0. The secret-on-argv question — DEFINITIVE ANSWER FIRST

**The GUI passes classified-secret flag VALUES on the child process ARGV, not
on stdin.** When a step drives a seed into a `phrase` slot or a `--from
phrase=` composite, the assembler bakes the plaintext straight into the argv
vector — `--slot @0.phrase=<plaintext seed>`, `--from phrase=<plaintext seed>`
— and `spawn_and_capture` hands that vector to
`runner::run_with_stdin(argv, stdin)` → `Command::new(argv[0]).args(&argv[1..])`
(`src/runner.rs:197-208`). The `stdin: Some(bytes)` pipe is wired for **exactly
one thing**: the tree-mode `--spec -` JSON of `build-descriptor`
(`src/app_window.rs:903-904` `spec_stdin = tree_form::spec_stdin_bytes(state)`;
build-descriptor carries **no** secret flags). Every seed the tutorial types
therefore transits argv.

Evidence chain:
- `src/form/invocation.rs:152-251` — `assemble_argv_with_secret_mask` pushes the
  secret VALUE token INTO `argv` (`--slot`/`@N.phrase=`, `--from` composite); the
  parallel `mask` bit governs DISPLAY only ("Used only to mask the last-run
  `argv:` display; never affects what is spawned", `src/runner.rs:22-27`).
- `src/app_window.rs:1021-1031, 1085-1090, 1215-1254` — the secret argv is
  spawned verbatim; `stdin` is `spec_stdin` (tree-only) on every path.
- The tutorial itself states the mechanism plainly: *"because the GUI passes a
  phrase as an argument, the tool emits its own 'secret material on argv'
  warning"* (`tutorial/10-ch0-orientation.md:120-124`;
  `tutorial/30-j2-multisig.md:42-47`).

**Is it a real exposure? Yes in principle, but it is a KNOWN, ACCEPTED, and
largely-MITIGATED posture — not a new or GUI-specific first-class defect.**
The spawned process is a pinned m-format CLI whose `main()` immediately calls
`process_hardening::set_non_dumpable()` (`PR_SET_DUMPABLE=0`), which — per the
project's own verified assessment — **denies other-UID `/proc/<pid>/cmdline`
reads and core dumps** (toolkit FOLLOWUP `argv-overwrite-after-parse`,
**resolved v0.34.7**; `design/FOLLOWUPS.md:1283-1289`). The residual same-UID
`/proc/cmdline` window is documented + accepted project-wide (same-UID already
implies ptrace / `/proc/mem` access, so no incremental exposure). On the GUI
side the in-RAM residue is also closed: `RunResult` and `PendingConfirm` are
`Zeroize + Drop` and swept on exit (`gui-last-run-result-argv-stdout-not-zeroized`
+ `gui-pending-confirm-argv-not-zeroized`, both **resolved v0.47.0**), and the
four on-screen masking surfaces are display-only by design.

Crucially, **the GUI's argv exposure is no worse than a CLI user typing the
same command** — it is the identical argv the CLI would receive, guarded by the
identical `PR_SET_DUMPABLE` mitigation and warned about by the identical CLI
advisory.

**The one genuinely GUI-specific residual:** the CLI offers an argv-avoiding
idiom for a single secret (`--from phrase=-`, `< seed.txt`, `--passphrase-stdin`)
and emits **no** warning when the secret arrives on stdin; the GUI has **no
stdin-routing affordance for the seed** (only tree-mode `--spec -`), so a GUI
user *always* trips the "secret material on argv" warning that a CLI user can
avoid. That gap is partially tracked: the `*-stdin` Boolean toggles are greyed
out precisely because "the GUI runner … provides no stdin channel for the value",
and a runner stdin-feed story was declared a **non-goal** at the user decision
(`mnemonic-gui/FOLLOWUPS.md::boolean-stdin-secret-toggles-never-emit`, resolved
v0.37.0). Note this only helps the single-secret case — the J2 all-seeds bundle
inherently carries 3 seeds and could never route all via stdin ("Only one secret
may arrive on standard input per run", `tutorial/30-j2-multisig.md:230-231`).

**Verdict on secret-on-argv: ALREADY-FILED / accepted-design (mitigated).**
Do not re-file. If the project chooses to raise the bar, the fix shape is a
single-secret stdin-routing path in `runner.rs` + the `phrase=-` argv form —
but that is a reserved design decision (current non-goal), not an unaddressed
defect.

---

## 1. Classified findings table

| # | Location (step + file:line) | What's avoided / routed-around | Classification |
|---|---|---|---|
| **F0** | Every secret step (J1 bundle, J2 converts + all-seeds, `convert`/`bundle` `--from phrase=`/`--slot phrase=`). `src/runner.rs:197-208`, `src/form/invocation.rs:152-251`, `tutorial/10-ch0-orientation.md:120-124` | Seed transits child **argv** (`--slot @N.phrase=<seed>`, `--from phrase=<seed>`), not stdin; GUI has no argv-avoiding stdin idiom for the seed | **ALREADY-FILED / accepted-design (mitigated).** `PR_SET_DUMPABLE(0)` in the spawned CLI (`argv-overwrite-after-parse`, resolved v0.34.7) + RAM scrub (v0.47.0) + `boolean-stdin-secret-toggles-never-emit` (runner-stdin-feed = non-goal, v0.37.0). Secret-hygiene-class, but no incremental exposure vs direct CLI use. Not re-filed. |
| **R1** | 6 restore steps: J2-08 (`tutorial/30-j2-multisig.md:255-265`), J3-13 (`40-j3:165-183`), J4-17 (`50-j4:122-134`), J4-nums-restore, J5-23/24 (`60-j5:47-89`). Harness `manifest.rs:8-14,136-144` (`restore_drives!`) | restore's `--template` materializes single-sig `bip44` default → **exit-2 refusal in `--md1` mode**. Routed around by selecting the wallet's MULTISIG template (inert in md1 mode, byte-identical output) | **IN-FLIGHT.** `mnemonic-gui/FOLLOWUPS.md::restore-form-single-sig-template-leaks-in-md1-mode` (open) + `design/SPEC_restore_template_none_affordance.md`. Fix = F1-style GUI-render `(none)` unset on restore's `--template`. **Doc-integrity note:** the prose presents the multisig-template choice as "for consistency" and does NOT disclose it is dodging a refusal (`40-j3:169-171`; `30-j2:261-265`) — the book disguises the papercut. |
| **F1** | Every export-wallet descriptor step: J2-04/05 canonicalise/BSMS, J3-10/11, J4-15/18/19/21. `manifest.rs:106-123` (`export_descriptor_text!`/`_fixture!` → `SelectDropdown{--template, value:""}`); `tutorial/30-j2-multisig.md:19-27` ("Unlocking the descriptor field"). Test `tests/export_wallet_template_none.rs` | Pre-fix the form was a trap: the always-materialized `--template=bip44` kept `--descriptor` permanently Disabled (mutex) → the descriptor arm was **unreachable** from the GUI | **FIXED (leg 1).** A1-APPEND `(none)` sentinel row on export-wallet `--template` (`EXPORT_WALLET_TEMPLATES`). Confirmed shipped + gated. export-wallet only (restore still open → R1). |
| **F2** | Engrave/bundle + fixture-descriptor steps: J2-06 (`tutorial/30-j2-multisig.md:175-179`), J3-12, J4-16, J5-22. Harness `manifest.rs:8-11,115-123,282,300`; `mod.rs:130-137` (`TypeTextFromFixture`) | Descriptor fed via `--descriptor` **TEXT**, not `--descriptor-file` (Path) | **HARNESS-ARTIFACT.** Form-to-form text paste is the intended GUI workflow (matches how the descriptor is chained between steps) and is deterministic. `--descriptor-file` is *drivable* (Path = text input, `TypePath` used for `--spec`), just not the natural GUI path. See B1 for the tangential Path-field UX note it glances. |
| **C1** | J2 canonicalise/BSMS/bundle-watch-only + J4 NUMS steps use `MULTISIG_DESC`/`TAPROOT_MULTI_DESC` constants (`tests/tutorial/mod.rs:53-72`) | The multisig descriptors are pre-assembled constants, not built live from the 6 `convert` fp+xpub runs | **HARNESS-ARTIFACT.** Explicitly for determinism/simplicity: "assembling them from six convert runs would add bespoke fp+xpub string-templating for zero determinism benefit" (`mod.rs:62-69`). The convert steps DO run and display real per-cosigner derivations; the constants carry only public material and are a pure function of S0/S1/S2. NOT because a build path is broken. |
| **C2** | J2/J3/J4 restore-feed steps (`TypeMd1Chain`, `mod.rs:156-161`; `tutorial/30-j2-multisig.md:398-405`) | The harness parses `md1` chunks out of a prior `bundle --json` run and types them into restore's `--md1` rows — replacing Examples' `jq -r ".md1[]" \| sed …` shell glue | **HARNESS-ARTIFACT** (real per-run output, not a fixture). **Latent UX note:** the GUI has **no in-app output→input chaining** — a real user must manually copy md1 chunks from the output panel into restore's rows. Missing-convenience-feature, not a defect; not filed. |
| **I1** | J3-09 build-descriptor refusal, exit 2 (`tutorial/40-j3-degrading-vault.md:28-43`; `manifest.rs:272-273`) | Guided builder refuses the 11-key policy (`over_envelope … > cap 4096`); journey hands the raw descriptor to `export-wallet`/`bundle` | **INTENTIONAL.** Correct funds-safety preview bound; the diagnostic itself points to the raw `--descriptor` path. Taught as such. |
| **I2** | J4-14 depth-2 taptree refusal, exit 2 (`tutorial/50-j4-taproot-twin.md:18-32`; `manifest.rs:293-294`) | Toolkit refuses a depth-≥2 taptree ("taptree branch must have 2 children, but found 1") rather than emit a malformed descriptor (upstream rust-miniscript PR-#953) | **INTENTIONAL** refusal / **ALREADY-FILED** underlying limitation. Correct to fail-closed. The real product constraint (can't emit depth-≥2 taptrees) is upstream-blocked and tracked: toolkit `design/FOLLOWUPS.md::upstream-miniscript-taptree-depth2-display-asymmetry` (`:4352`); J3-restore prose corroborates ("no experimental build", `40-j3:178-180`). A user attempting the "tidiest" 4-leaf layout hits it unexpectedly; the tutorial pre-empts by teaching the depth-1 packing. |
| **I3** | J4-19 BSMS-unsupported-for-taproot refusal, exit 2 (`tutorial/50-j4-taproot-twin.md:186-196`; `manifest.rs:312-313`) | `--format bsms` declines on a taproot descriptor; points to `bitcoin-core`/`sparrow` | **INTENTIONAL.** BIP-129 has no taproot encoding yet; correct refusal with working alternatives. |
| **I4** | "Materialised defaults" (`tutorial/10-ch0:127-137`, `30-j2:28-30`) — argv spells out `--network mainnet --language english …` even when untouched | GUI emits at-default Dropdown flags the shell examples omit | **INTENTIONAL.** Explicit-defaults design; identical wallet, harmless verbosity. (Text/Path at-default values ARE suppressed — `gui-prefilled-default-text-appends-on-type`, resolved.) Not a bug. |
| **B1** | Glanced by F2. GUI Path-flag rendering is a bare `TextEdit::singleline` with no browse dialog (`src/form/widget.rs:672-688`; no `rfd`/`native-dialog` dep in `Cargo.toml`). Affects `--descriptor-file`, `build-descriptor --spec` (driven as bare `"policy.json"`, `manifest.rs:273`), `--output`, `--passphrase-candidates-file` | To use a file-based flag the user must **type/paste an exact filesystem path** — no file picker | **BUG-TO-FILE (LOW, UX, not secret-hygiene).** Component: `mnemonic-gui` `src/form/widget.rs` Path arm. Untracked in either FOLLOWUPS. Fix shape: optional `rfd` "Browse…" button on Path fields (writes the chosen path into the text buffer; keeps the field drivable/headless-testable). **Caveat:** may be *declined as intentional* — the GUI is a deliberately thin, dialog-free, headless-testable argv builder; file-based flags are functional-if-awkward today. Flag for a product decision, not an obvious defect. |

---

## 2. Already-known workarounds — status confirmation (per the brief)

- **export-wallet `--template` (none)** — was a real bug (descriptor arm
  unreachable), **FIXED** in Leg 1 as the F1 A1-append (`export_descriptor_*`
  macros drive `SelectDropdown{--template, value:""}`; `tests/export_wallet_template_none.rs`;
  export-wallet only). ✔ status confirmed.
- **restore `--template` single-sig rejected in `--md1`** — real bug,
  **IN-FLIGHT** (`restore-form-single-sig-template-leaks-in-md1-mode`, open;
  `design/SPEC_restore_template_none_affordance.md`). 6 restore steps route
  around it via a multisig template. ✔ status confirmed; added doc-integrity
  note (R1) that the book presents the route-around as "consistency".

---

## 3. Summary

**Recommend filing: 1** — **B1** `gui-path-flag-no-file-picker` (LOW / UX /
not secret-hygiene): GUI Path flags are bare text inputs with no browse dialog,
so file-based flags (`--descriptor-file`, `--spec`, `--output`,
`--passphrase-candidates-file`) require typing an exact path. Flag as a product
decision — plausibly declined as intentional (thin dialog-free argv builder).

Everything else resolves to already-managed status:
- **Secret-on-argv (F0):** ARGV, definitively — but a known, accepted,
  PR_SET_DUMPABLE-mitigated posture no worse than direct CLI use; the one GUI
  residual (no seed-stdin idiom) is a declared non-goal. Not re-filed.
- **R1 restore-template:** IN-FLIGHT (spec in hand).
- **F1 export-template:** FIXED.
- **F2 / C1 / C2:** HARNESS-ARTIFACT (determinism / form-to-form fidelity).
- **I1 / I2 / I3 / I4:** INTENTIONAL correct behavior (I2's underlying upstream
  taptree limit is itself already tracked).

No new secret-hygiene defect and no correctness bug surfaced by the tutorial's
route-arounds; the instrument's workarounds map cleanly onto known/tracked
items plus one low-severity UX candidate.
