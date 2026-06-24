# SPEC — Constellation man pages (`gen-man` across the 4 m-format CLIs + install.sh)

**Status:** DRAFT — R0 round 3 folded (4 findings: 1 Critical + 1 Important + 2 Minor — the leading toolkit-side exact-set gate). Pending re-dispatch convergence review (must reach 0C/0I before any code).
**Source SHA (toolkit `origin/master`):** `6163ef2e00ce427555d6432fa8ff0bf6b8936451` (re-grep before lifting any line citation into the plan-doc).
**Author date:** 2026-06-23.

> **R0 fold log (round 3 — the leading toolkit-side gui-schema exact-set gate).** Folded into this revision (round-2 clap_mangen-mechanism folds preserved verbatim below):
> - **C-3 (Critical) — adding `gen-man` breaks a LEADING, same-repo toolkit gate, NOT just the lagging GUI pin gate.** The toolkit test `crates/mnemonic-toolkit/tests/cli_gui_schema.rs::gui_schema_lists_all_subcommands` is a hard `assert_eq!` on the FULL alphabetical subcommand-name vector (live: **30** names at `6163ef2e`). It fires in **P1 the instant `gen-man` is added** — it is a leading same-repo gate, NOT the lagging GUI-pin `schema_mirror` gate that §4/§6 previously framed as the only relevant gate. It fires for **VISIBLE *and* HIDDEN** `gen-man`, because `build_schema` (`gui_schema.rs:990`) filters subcommands by NAME ONLY (`get_name() != "gui-schema" && get_name() != "help"`, `:1006-1008`) and **never consults `is_hide_set()`** — so `gen-man` appears in the `gui-schema` JSON regardless of `hide`. `gen-man` sorts between `final-word` and `import-wallet` (`f < g < i`), making the count **31**. FOLD: (1) P1 gains a required edit to `cli_gui_schema.rs` — insert `"gen-man"` between `"final-word"` and `"import-wallet"` in the golden vector and update the narrative comment (`30`→`31`, `all 30`→`all 31`); (2) §4/§6 reclassified — the toolkit-side `gui_schema_lists_all_subcommands` is a **LEADING same-repo gate** (fires for visible AND hidden); the GUI-pin `schema_mirror` remains the lagging one. **Sibling asymmetry (confirmed live):** the siblings' equivalent tests do **not** require the edit — md-cli `cmd_gui_schema.rs:45` (`gui_schema_lists_all_documented_subcommands`) uses `names.contains(&expected)` (a SUBSET check), mk-cli `gui_schema.rs:129` (`gui_schema_lists_all_five_v0_1_subcommands`) likewise uses `names.contains(&expected)`, and ms-cli's `gui_schema_emits_spec_v7_json.rs` has **no subcommand-name-set gate at all** — all three TOLERATE the extra `gen-man` page. The prior uniform 4-CLI treatment of the schema-test surface was therefore wrong; the gate-bearing exact-set test is **toolkit-only**.
> - **I-3 (Important) — re-derive §4 visibility from the corrected gate model; distinguish manual-discipline from toolkit-hard-gate in §6.** §4's VISIBLE recommendation previously rested on the now-falsified premise that "no automatic gate distinguishes visible vs hidden." The TRUE state, post-C-3: (a) `gen-man` appears in `gui-schema` JSON and trips the toolkit exact-set test EITHER WAY — `hide` does **not** avoid that gate; (b) `hide` DOES remove `gen-man` from the parent `mnemonic --help`, which is the *only* behavioral difference. The recommendation (VISIBLE) is **re-derived** from this corrected model (not the false "no gate either way" claim) so a reader can audit it. §6's manual-lint row ("no completeness gate so the list edit is the discipline") is verified accurate — `cli-subcommands.list` is hand-maintained and flag-coverage only checks documented-flags-for-listed-subs — but this is the OPPOSITE posture from the toolkit `gui-schema` exact-set test; §6 now DISTINGUISHES them: manual = **discipline** (no completeness gate), toolkit `gui-schema` exact-set test = **hard leading gate**.
> - **M-8 (Minor) — P1 done-criteria must run the FULL `cargo test -p mnemonic-toolkit` suite, not the man-gen target alone.** Per the project MEMORY lesson (`R0 reviews / per-phase gates must run the FULL package suite` — the `lint_argv_secret_flags` RED-through-P4/P5 case): the C-3 `cli_gui_schema.rs` golden-vector breakage is **INVISIBLE** to a P1 reviewer who runs only the new man-gen test target — it surfaces only at the full-package run. §8/P1 done-criteria now explicitly require `cargo test -p mnemonic-toolkit` (full package).
> - **M-9 (Minor) — disambiguate install.sh `if cargo install` line vs `installed_count` increment line.** Live at `6163ef2e`: crates.io branch — `if cargo install … ; then` at `:326`, the `installed_count=$((installed_count + 1))` increment at `:327`; git branch — `if cargo install … --git … ; then` at `:339`, the increment at `:340`. The hook attaches to the **increment** (success) side, which the spec correctly cited as `:327`/`:339` for the *git-branch increment* — but round-2 prose conflated the `if cargo install … ; then` line with the increment. §5/§8 prose now state BOTH lines per branch: crates.io `if`=`:326` / increment=`:327`; git `if`=`:339` / increment=`:340`. The cited increment line numbers themselves are corrected to `:327` (crates.io) and `:340` (git).
>
> **R0 fold log (round 2 — clap_mangen mechanism correction; preserved).**
> - **C-1 (Critical)** — The §2 prescription to call `root.build()` BEFORE `clap_mangen::generate_to` is WRONG. Empirically (clap_mangen 0.3.0 + clap 4.6.1, repro mirroring the toolkit's `seed-xor/slip39/ms-shares split|combine` shape) the pre-`.build()` materializes a full `help` PSEUDO-SUBCOMMAND SHADOW TREE into the output (15 files emitted, of which 10 were `*-help*.1` pages; on the full toolkit ~18 spurious `*-help-*.1` of ~37). Root cause: `generate_to` internally does `disable_help_subcommand(true)` THEN `cmd.build()`; an external `root.build()` runs FIRST and materializes the `help` subcommands as real tree entries before the internal disable can suppress them. The naive call `generate_to(Cli::command(), &dir)` with NO pre-build is clean: 6 files / 0 help-pages, exactly the per-(sub)command set wanted. → §2 (pre-build prescription DELETED; bare naive call specified), §8 P1.
> - **C-2 (Critical)** — The STATED RATIONALE for the pre-`.build()` (so lazy `_propagate_global_args` populates the `global = true --no-auto-repair` flag into every nested page) is FALSE under the pinned versions. Empirically `--no-auto-repair` renders in ZERO generated pages — not the root `mnemonic.1`, not `mnemonic-bundle.1`, not nested `mnemonic-seed-xor-split.1` — REGARDLESS of `.build()`. clap_mangen 0.3.0's `Man` renderer does not surface global args at any level for this tree. The pre-build buys nothing on the global-flag front (its sole rationale) while costing the entire help shadow tree. → §2 (rationale STRUCK; verified behavior stated; global-flag-in-man recorded as an upstream-limitation open question, §10 Q5), §8 P1 (any "nested page contains `--no-auto-repair`" assertion REMOVED).
> - **I-2 (Important)** — §8 P1's exact-page-set test was under-specified about the tree's BUILD STATE and, pre-fold, self-inconsistent (a walk of UNBUILT `Cli::command()` is help-free, but a pre-built generator emits the ~18-page help shadow tree → predicted-set != actual-set → either RED, or a plan author "fixes" it by pre-building the comparison tree and BAKES the help bloat into the golden expectation, silently shipping ~18 junk `*-help-*.1` pages into every user's manpath). → §8 P1 made build-state-explicit: naive generator + walk of the UNBUILT tree minus `is_hide_set()` + an explicit NEGATIVE assertion that NO produced filename matches `*-help.1` or `*-help-*.1`.
> - **M-6 (Minor)** — Factual drift in cited counts/versions. (a) The live `enum Command` (`main.rs:99`) has **23** arms at `6163ef2e` (Bundle…BuildDescriptor) — not 24. (b) ALL FOUR CLIs (toolkit/md/ms/mk) are clap **4.6.1** in their `Cargo.lock` — not "md clap 4.5"; the `clap_mangen ^4.0.0` constraint is satisfied so the compatibility conclusion holds. → §2, §3.
> - **M-7 (Minor)** — Stale install.sh line citations (further refined by M-9 this round). → §5 anchors corrected; §8 P3 + §5 install-layer verification note added.
>
> **Live re-greps (this fold, @ `6163ef2e`):** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:990` = `fn build_schema(cmd: &Command)`; `:1006-1008` filters `get_name() != "gui-schema" && get_name() != "help"` — never consults `is_hide_set()`. `crates/mnemonic-toolkit/tests/cli_gui_schema.rs::gui_schema_lists_all_subcommands` = hard `assert_eq!(names, vec![…30 names…])`, narrative "all 30 user-facing subcommands". Sibling tests: `descriptor-mnemonic/crates/md-cli/tests/cmd_gui_schema.rs:45` = `names.contains(&expected)` (subset); `mnemonic-key/crates/mk-cli/tests/gui_schema.rs:129` = `names.contains(&expected)` (subset); `mnemonic-secret/crates/ms-cli/tests/gui_schema_emits_spec_v7_json.rs` = NO name-set gate. `scripts/install.sh`: crates.io `if cargo install … ; then` `:326`, increment `:327`; git `if cargo install … --git … ; then` `:339`, increment `:340`; `for name in $ALL` `:298`; `EXCLUDE`/`selected()`/`for_each_token`/`--no-gui` `:66,:153-154,:203,:220`; `#!/bin/sh` `:1`, "No system files touched; no sudo required." `:15-16`, `set -eu` `:18`; warning convention `echo "..." >&2` (`:259-262,:329`) — NO `warn` helper. `crates/mnemonic-toolkit/src/main.rs:91` = `#[arg(long, global = true)] no_auto_repair: bool` (sole `global=true`); `:99` = `enum Command {` with **23** tuple-variant arms (do NOT bake a count — derive from the tree). `.github/workflows/rust.yml:51` = `name: fmt (pinned 1.95.0)`, `mlock.rs` the sole exemption.

---

## 1. Goal & non-goals

**Goal.** Every constellation CLI (`mnemonic`, `md`, `ms`, `mk`) can self-emit roff man pages from its compiled clap `Command` tree. `scripts/install.sh` drops them into the user manpath after `cargo install`, so `man mnemonic` / `man mnemonic-bundle` / `man md` / `man ms` / `man mk` resolve with **no sudo, no system files, no MANPATH edit** on Linux/man-db. The pages are clap-generated, hence **binary-faithful by construction**.

**Non-goals.**
- No hand-authored man content. **No content-fidelity gate is needed** — clap_mangen renders directly from the same `clap::Command` tree the binary parses, so the man page cannot drift from the binary's actual flag surface. (Contrast: the `docs/manual/` mirror is hand-authored prose and *is* lint-gated.) This is the load-bearing simplification of the whole design.
- No codec-crate changes (`md-codec`/`ms-codec`/`mk-codec` are NO-BUMP — man generation is a CLI-only concern).
- The GUI (`mnemonic-gui`) has no CLI man surface; it is excluded from the man step (it only participates via its schema mirror, below).

---

## 2. Mechanism decision (resolved)

**Chosen: a `gen-man` subcommand per CLI that calls `clap_mangen::generate_to(<root Command>, out_dir)` with NO pre-build.**

This is the *only* option compatible with the cargo-install-only distribution model, and it reuses plumbing the toolkit already has.

| Option | Verdict | Why |
|---|---|---|
| **(a) `gen-man` subcommand → `clap_mangen::generate_to`** | **CHOSEN** | Ships *inside* the published binary, so any `cargo install`-ed copy self-emits its pages. `install.sh` (already writes only to `~/.cargo/bin`) invokes `<bin> gen-man --out <dir>` post-install — zero extra artifacts, stays in the no-sudo contract. |
| (b) `build.rs` + clap_mangen at build time (OUT_DIR) | REJECTED | `cargo install` copies only the *binary* out of the ephemeral build dir; no `OUT_DIR` survives, so `install.sh` could never locate the page. |
| (c) separate `xtask`/`[[bin]]` generator | REJECTED for distribution | Not shipped in the published binary, so a post-install hook can't invoke it. (Fine for repo-local doc builds only.) |
| single combined `Man::new(root).render()` page | REJECTED | Loses per-subcommand flag detail; only emits a SUBCOMMANDS list. |

**Layout:** page-per-(sub)command (git/cargo convention) via `generate_to`, which recurses the whole tree.

**Dependency:** `clap_mangen = "0.3"` (latest 0.3.0, requires clap `^4.0.0`). **All four CLIs are clap 4.6.1** (toolkit/md/ms/mk, per each `Cargo.lock` at this SHA) — the `^4.0.0` constraint is satisfied, **no clap bump (M-6)**. Default features suffice: the toolkit has **zero `#[arg(env=...)]`**, so the `env` feature is not needed (add it later only if an env-backed arg appears).

**The generator call is the bare, naive form — NO pre-`.build()` (C-1):**

```rust
clap_mangen::generate_to(Cli::command(), &dir)?;   // Cli::command() unbuilt; generate_to builds internally
```

**Do NOT call `.build()` on the root `Command` before `generate_to` (C-1, empirically grounded).** Under clap_mangen 0.3.0 + clap 4.6.1, a pre-`.build()` POISONS the output with a `help` pseudo-subcommand SHADOW TREE: in a repro mirroring the toolkit's `seed-xor/slip39/ms-shares split|combine` shape, pre-build emitted **15 files of which 10 were `*-help*.1` pages** (`mnemonic-help.1`, `mnemonic-help-seed-xor-split.1`, `mnemonic-seed-xor-help.1`, `mnemonic-seed-xor-help-split.1`, …); on the full toolkit this is ~18 spurious `*-help-*.1` pages out of ~37. The naive call (no pre-build) is clean: **6 files / 0 help-pages**, exactly the per-(sub)command set wanted. Root cause: `generate_to` internally does `disable_help_subcommand(true)` THEN `cmd.build()`; an external `root.build()` runs FIRST and materializes the `help` subcommands as real tree entries before the internal disable can suppress them. If a build is ever needed for another reason, you MUST re-assert `disable_help_subcommand(true)` after that build and re-verify empirically — but v1 needs no build at all.

**Global-flag behavior (C-2, verified — do NOT assert otherwise).** The toolkit's sole `global = true` flag, `--no-auto-repair` (`main.rs:91`), renders in **ZERO** generated pages — not the root `mnemonic.1`, not `mnemonic-bundle.1`, not nested `mnemonic-seed-xor-split.1` — REGARDLESS of any `.build()`. clap_mangen 0.3.0's `Man` renderer does not surface global args at any level for this tree. There is therefore **no `.build()`-buys-global-propagation benefit** (this was the entire prior rationale for the pre-build, and it is false). Surfacing the global flag in man output is an **upstream limitation / open question** (§10 Q5), NOT something achieved by build — and **no P1 test may assert that any page (root or nested) contains `--no-auto-repair`** (such an assertion is RED on day one).

**Nested-page filenames are collision-safe:** `generate_to` names files via `get_display_name()` (hyphen-joined parent→child, e.g. `mnemonic-seed-xor-split.1`), so the three `split` children (seed-xor / slip39 / ms-shares) do NOT clash. It also skips `is_hide_set()` subcommands and disables the auto `help` subcommand (internally, on the UNBUILT tree the naive call passes it). (Historical `get_name()` collision bug is fixed in current `generate_to`.)

---

## 3. `gen-man` subcommand surface (uniform across all 4 CLIs)

```
<cli> gen-man --out <DIR>
```

- `--out <DIR>` (required): directory to write `*.1` files into. Subcommand creates it if absent (`fs::create_dir_all`), then `generate_to(Cli::command(), dir)` — **no pre-build (C-1)**.
- **No other flags in v1.** Keep the surface minimal: one required `--out`. (A `--section`/`--stdout` toggle is a YAGNI add — omit; revisit only if a packager asks.)
- **Toolkit reference impl** lives at `crates/mnemonic-toolkit/src/cmd/gen_man.rs` + `pub mod gen_man;` in `cmd/mod.rs` + a `Command::GenMan { out: PathBuf }` arm in `main.rs` (`enum Command` at `main.rs:99`; reuse the existing `Cli::command()` pattern from the `gui-schema` arm). The toolkit `main.rs` already imports `clap::CommandFactory` (for `gui-schema`), so `Cli::command()` is in scope.
- **Sibling impls** (`md`/`ms`/`mk`) are structurally identical. **Note:** unlike the toolkit, `md/ms/mk` `main.rs` do **not** yet import `clap::CommandFactory` — the sibling work must add `use clap::CommandFactory;` (or use `<Cli as CommandFactory>::command()`) to obtain the root `Command`.

**Toolkit page inventory — derive from the live tree, do NOT hardcode (M-2/M-6).** The live `enum Command` (`main.rs:99`) has **23** top-level arms as of `6163ef2e` (Bundle … BuildDescriptor — counted from the tree, never baked); of these, 5 are nested parents (`seed-xor`/`slip39`/`ms-shares`/`seedqr`/`xpub-search`) carrying their respective children (e.g. `split`/`combine`, and `xpub-search`'s 4 leaves). The emitted set is therefore: 1 root (`mnemonic.1`) + the top-level leaf pages + the 5 nested-parent pages (`mnemonic-seed-xor.1`, …) + the nested child pages (`mnemonic-seed-xor-split.1`, `mnemonic-xpub-search-path-of-xpub.1`, …) + the **new** `mnemonic-gen-man.1` + the already-visible `mnemonic-gui-schema.1` — on the order of **~37 `.1` files**, but the count is an *output* of walking the tree, never an input, and crucially carries **ZERO `*-help*.1` pages** under the naive call (C-1). Both `gen-man` and `gui-schema` are visible (not `is_hide_set`), so both yield pages. The P1 test asserts the EXACT page set by walking the live UNBUILT `Command` tree (filtering `is_hide_set()` + the auto `help`), explicitly including `mnemonic-gen-man.1` and `mnemonic-gui-schema.1`, AND asserts the negative canary (no `*-help*.1`) — see §8. **No magic integer in prose or test.**

**⚠ Adding `gen-man` ALSO trips a SECOND, leading toolkit-side gate (C-3).** Independent of the man-page set, the toolkit test `crates/mnemonic-toolkit/tests/cli_gui_schema.rs::gui_schema_lists_all_subcommands` is a hard `assert_eq!` on the FULL alphabetical `gui-schema` subcommand-name vector (live: **30** names). `build_schema` (`gui_schema.rs:990`) filters subcommands by NAME ONLY (`:1006-1008`) and **never** consults `is_hide_set()` — so `gen-man` enters the `gui-schema` JSON **whether visible or hidden**, sorting between `final-word` and `import-wallet`, taking the count to **31**. **P1 MUST edit this golden vector** (insert `"gen-man"`, bump the `30`→`31` narrative); failing to do so RED-fails the toolkit suite the instant `gen-man` is added — a *leading* same-repo gate, not the lagging GUI-pin one (see §4/§6). The siblings' equivalent tests use SUBSET checks (md/mk `names.contains(&expected)`) or no name-set gate at all (ms), so they tolerate the extra `gen-man` and require **no** test edit.

---

## 4. Visibility — GENUINE USER-FORK #1

**`gen-man` should be a normal (visible) subcommand, OR `hide = true` (developer-tooling, out of `--help`).**

**Recommendation: VISIBLE.** Re-derived from the CORRECTED gate model (C-3 — the prior rationale rested on a now-falsified "no automatic gate distinguishes visible vs hidden" premise). The true gate landscape:

- **The toolkit-side `gui_schema_lists_all_subcommands` exact-set test is a HARD LEADING gate that fires EITHER WAY (C-3).** `gen-man` appears in `gui-schema` JSON regardless of `hide` (the emitter filters by name, never by `is_hide_set()` — `gui_schema.rs:1006-1008`), so the exact-set `assert_eq!` breaks in P1 whether `gen-man` is visible or hidden. **`hide` does NOT avoid this gate.** It must be folded (golden-vector edit) in either case.
- **The ONLY behavioral difference `hide` buys** is removing `gen-man` from the parent `mnemonic --help` output. It buys **no** churn reduction on any gate: not the toolkit exact-set test (fires either way), not the GUI `schema_mirror`, not the `gui-schema` JSON surface.
  - **GUI `schema_mirror`** (`mnemonic-gui/tests/schema_mirror.rs`) iterates only the GUI's *hand-declared* subcommands — it is **not** a subcommand-SET exhaustiveness gate. A new subcommand the GUI omits is silently uncovered whether visible or hidden. So the schema-mirror lockstep for a *new* subcommand is a **paired-PR discipline + a LAGGING pin-bump gate**, not a leading one.
  - **`gui-schema` JSON emission** filters by name only (`gui_schema.rs:1006-1008`) — never `is_hide_set()`. A hidden `gen-man` STILL appears in `mnemonic gui-schema` output (this is precisely what trips the toolkit exact-set test).
  - **Manual flag-coverage lint** is driven by the hand-maintained `docs/manual/tests/cli-subcommands.list` with no completeness check; `hide` only keeps `gen-man` out of the *parent* `--help`, not out of `gen-man --help` extraction.
- Net: the real lever is "which mirror files you touch," not the `hide` attribute. Visible + fully documented is the internally-consistent choice and avoids the un-gated-drift trap (a *visible* subcommand silently left out of the manual/GUI surfaces later). If the user instead prefers HIDDEN, then deliberately omit it from the manual + GUI schema **and file a FOLLOWUP** recording the intentional omission so a future lagging-gate reviewer doesn't read it as an oversight — but note that even HIDDEN, the toolkit `cli_gui_schema.rs` golden-vector edit is **still mandatory** (C-3).

SemVer is **MINOR either way** — `gen-man` is a public CLI surface addition regardless of `hide`.

---

## 5. install.sh wiring (no-sudo / XDG user manpath)

**`scripts/install.sh` is `#!/bin/sh` — POSIX only, no bash-isms (M-3).** No arrays, no `[[ ]]`, `local` is borderline-avoid; mirror the existing `case`/`mkdir -p`/`${VAR:-default}` style already in the file. The new `--no-man` opt-out and man-step gating MUST reuse the existing `--no-gui`/`EXCLUDE`/`selected()`/`for_each_token` machinery (`install.sh:66,:153-154,:203,:220`) rather than a fresh flag-parse pattern.

**Install path:** `${XDG_DATA_HOME:-$HOME/.local/share}/man/man1`. **Verified on this machine — man-db 2.13.1 (this env), not asserted universal (M-1):** `man -w` / `manpath` list `~/.local/share/man` **first**, ahead of `/usr/share/man`; on THIS install man-db supplies the XDG user-manpath internally (`/etc/man_db.conf` lists only `/usr/share/man` + `/usr/local/share/man` as `MANDATORY_MANPATH`, yet `manpath` still emits `~/.local/share/man` first). So on man-db 2.13.1 there is **no MANPATH edit, no rc change, no config file** — `man mnemonic` resolves immediately after the page drops in. Older/other man-db builds, or distros that strip the XDG default, MAY not pre-seed the user manpath — which is why the tail hint below is mandatory and unconditional. Preserves install.sh's "No system files touched; no sudo required" invariant (`install.sh:15-16`).

**Changes to `scripts/install.sh`:**
- New `MAN_DIR` default `${XDG_DATA_HOME:-$HOME/.local/share}/man/man1`; `--man-dir <dir>` override (mirrors `CARGO_INSTALL_ROOT`).
- New `--no-man` opt-out, modelled on `--no-gui` (`install.sh:153-154`): set a `NO_MAN` flag in the same arg-parse `case`; respect `--dry-run` (`install.sh:161`) by printing the `gen-man` command and not running it (matches the existing `[dry-run]` echo branches at `:323`/`:336`).
- Post-install hook **inside the per-component loop** (`for name in $ALL` at `install.sh:298`), **only on the successful `cargo install` branch** — i.e. inside the `if cargo install … ; then` TRUE-arm. **Disambiguated per branch (M-9):** the crates.io branch's `if cargo install … ; then` is line **`:326`** and its success-arm `installed_count=$((installed_count + 1))` increment is line **`:327`**; the git branch's `if cargo install … --git … ; then` is line **`:339`** and its increment is line **`:340`**. The hook attaches on the **increment (success) side** — `:327` for crates.io, `:340` for git — NOT after a `FAILED` install (each `if` has a paired `else` with `echo "  FAILED" >&2` + `failed_count`). The short component **name == bin name** (`mnemonic`/`md`/`ms`/`mk`) while the cargo package differs (`mnemonic-toolkit`/`md-cli`/`ms-cli`/`mk-cli`). Gate the man step `case "$name" in mnemonic|md|ms|mk) … ;; esac` to exclude `mnemonic-gui`, and short-circuit when `NO_MAN` is set.
- **The hook MUST be `|| `-guarded — NEVER bare (I-1).** Under `set -eu` (`install.sh:18`) a bare nonzero `gen-man` would abort the ENTIRE install, killing an otherwise-working binary. Because `md`/`ms`/`mk` default to **crates.io-latest** (not the pinned git tag — that needs `--from-git`; `component_info` field 4 = `yes`), a freshly-published toolkit can run against a sibling whose crates.io-latest **still lacks `gen-man`** during the rollout window (until that sibling's own publish lands); `MAN_DIR` could also be read-only or the disk full. All of these must be **non-fatal**. The line is:

  ```sh
  "${CARGO_INSTALL_ROOT:-$HOME/.cargo/bin}/$name" gen-man --out "$MAN_DIR" 2>/dev/null \
      || echo "warning: man pages skipped for $name (needs a $name build with gen-man)" >&2
  ```

  (The file has **no `warn` helper** — its warning convention is `echo "…" >&2`, e.g. `install.sh:259-262,:329`; use that.) Precede it with `mkdir -p "$MAN_DIR"` (the `mkdir -p` is the create guard). State the subcommand-presence precondition explicitly in a comment: only sibling builds that carry `gen-man` emit pages; older crates.io-latest siblings are tolerated, not required.
- **Idempotent:** writing `*.1` overwrites. **Do NOT call system `mandb`** (sudo/perm-prompt risk); rely on man-db's on-demand indexing. (Optional best-effort `mandb "$MAN_DIR" 2>/dev/null || true` is allowed but not recommended.)
- **Tail message — portable hint is MANDATORY and unconditional (M-1).** Print a man hint that is true on ANY man-db that does not pre-seed the XDG user manpath (and on macOS/BSD man, which does not auto-read `~/.local/share/man`). Always include the portable fallback `man -M "$MAN_DIR" mnemonic` (or an explicit `MANPATH` hint) — **not** gated to macOS-only. macOS auto-discovery remains a **TO-VERIFY in the plan** (not testable in this Linux env), but the tail hint does not depend on that verification because it is unconditional.

**Install-layer verification note (M-7 / post-C-1 end-to-end check).** After folding C-1 (naive call, no help tree), the generator emits ZERO `*-help*.1` pages — so install.sh drops zero junk pages into `~/.local/share/man/man1`. The idempotent-overwrite model would NOT clean up such junk on a later fixed install, so the canary must be checked at the install layer too: **the P3 shellcheck/dry-run harness MUST grep the generated dir and confirm it contains zero `*-help*.1` pages before declaring P3 done.** No separate cleanup logic is needed once the generator is clean — but this end-to-end grep is the regression tripwire for an accidental future pre-build.

---

## 6. Lockstep & mirror obligations

Adding `gen-man` is a **CLI-surface addition** → it triggers the constellation mirror invariants. **No content-fidelity gate on the man pages themselves** (clap-generated). The mirrors below are about the *manual prose* and *GUI schema*, not the roff. **Crucially, the surfaces below split into HARD LEADING GATES vs DISCIPLINE — they are NOT all "discipline" (I-3).**

| Mirror | Action | Gate posture |
|---|---|---|
| **Toolkit `gui-schema` exact-set test** (`crates/mnemonic-toolkit/tests/cli_gui_schema.rs::gui_schema_lists_all_subcommands`) | Insert `"gen-man"` between `"final-word"` and `"import-wallet"` in the `assert_eq!` golden vector; bump narrative `30`→`31` / `all 30`→`all 31`. | **HARD LEADING GATE — same-repo, fires in P1 the instant `gen-man` is added, for VISIBLE *and* HIDDEN (C-3).** This is the gate-bearing surface; it is **toolkit-only** (siblings use subset checks / no name-set gate, so no sibling test edit). |
| **Manual** (`docs/manual/`) | Add `<cli> gen-man` to `docs/manual/tests/cli-subcommands.list` + a `gen-man` section in each chapter (`41-mnemonic.md`, `42-md.md`, `43-ms.md`, `44-mk-cli.md`). Lint = `docs/manual/tests/lint.sh` via `make -C docs/manual lint`. | **DISCIPLINE — NO completeness gate (I-3).** `cli-subcommands.list` is hand-maintained and flag-coverage only checks documented-flags-for-listed-subs; nothing forces `gen-man` *into* the list. The *list edit* is the discipline — the OPPOSITE posture from the toolkit `gui-schema` exact-set test above. |
| **GUI schema mirror** | Add `gen-man` `SubcommandSchema` to `mnemonic-gui/src/schema/{mnemonic,md,ms,mk}.rs`. **Minimal entry: the only flag is `--out` (Path kind, NO `ValueEnum`/dropdown), so there is no dropdown-value lockstep — trivially satisfiable (M-4).** | **PAIRED-PR DISCIPLINE + LAGGING pin-bump gate** — `schema_mirror` walks GUI-declared subs only, so omitting the entry stays green until the *next* GUI pin bump fires the drift gate against the accumulated delta (CLAUDE.md v0.27.x lagging-gate trap). The paired-PR is the real discipline. Because `gui-schema` *emits* `gen-man`, a GUI dev who adds the entry will have `schema_mirror` compare its flag-set — but with only `--out` (no enum) it is trivially green. P2 reviewers should NOT over-scope this GUI entry. |
| **CHANGELOG** | `## mnemonic-toolkit [0.73.0]` section. | YES — `changelog-check.yml` fires on `mnemonic-toolkit-v*` tag. |
| **Cross-repo FOLLOWUP companions** | Mirror an entry in each sibling's `design/FOLLOWUPS.md` with `Companion:` lines (CLAUDE.md cross-repo rule). | discipline |
| **Toolkit version-sites** | README ×2 + `scripts/install.sh` component_info self-pins + `fuzz/Cargo.lock`. `both_readmes` lint catches READMEs. | partial |

**Does a HIDDEN `gen-man` minimize lockstep?** **No** (see §4) — and critically, even HIDDEN, the **toolkit `gui-schema` exact-set test edit is still mandatory** (C-3: the emitter ignores `is_hide_set()`). The schema-mirror lockstep for a *new* subcommand is a discipline/lagging-gate not a leading gate, and `gui-schema` JSON emits hidden subs anyway. Visibility is a UX call, not a churn lever.

---

## 7. Release asset — GENUINE USER-FORK #2

**Should man pages also ship as a per-tag release asset (a `*-man.tar.gz`)?**

**Recommendation: YES, per-CLI, on each CLI's own tag.** There is a clean existing model: `.github/workflows/manual.yml` already does `gh release view "$REF_NAME" || gh release create … --generate-notes`, then `gh release upload "$REF_NAME" <artifact> --clobber`, with `permissions: contents: write` + `GH_TOKEN`. A man-tarball step mirrors this exactly: build the binary, `<bin> gen-man --out man/ && tar czf <cli>-man.tar.gz man/`, `gh release upload`. (The naive-call generator means the tarball is help-shadow-free by construction — C-1.)

- **(a) per-CLI on each CLI's tag — RECOMMENDED.** Versions the man set with the CLI that produced it; consistent with the constellation's independent-release model. Gives offline/packaged installs a downloadable man set without building.
- (b) one aggregate tarball on a single toolkit/manual tag — DISCOURAGED (couples 4 independently-versioned CLIs to one tag; against the repo's lockstep posture).

**This fork is optional/severable** — it can be deferred to P4 (or dropped) without affecting the install.sh path, which builds pages locally. If the user wants minimal scope, ship P1-P3 and skip P4.

---

## 8. Phasing

**P1 — `gen-man` in all 4 CLIs (codec-side, TDD).** Add `clap_mangen = "0.3"` dep + `gen-man --out <DIR>` subcommand to `mnemonic`/`md`/`ms`/`mk` (add `use clap::CommandFactory` to md/ms/mk). **Call the bare naive `clap_mangen::generate_to(Cli::command(), &dir)` — NO pre-`.build()` (C-1).**

**Required test/golden-vector edits:**
- **`crates/mnemonic-toolkit/tests/cli_gui_schema.rs::gui_schema_lists_all_subcommands` (MANDATORY — C-3):** insert `"gen-man"` between `"final-word"` and `"import-wallet"` in the `assert_eq!` golden vector, and update the narrative comment (`30`→`31`; `all 30 user-facing subcommands`→`all 31`). This breaks the instant `gen-man` is added — for VISIBLE *and* HIDDEN — because `build_schema` emits by name not by `is_hide_set()`. **The siblings need NO equivalent edit** (md/mk use `names.contains(&expected)`, ms has no name-set gate).
- each CLI's `gen-man --out <tmp>` produces a non-empty `*.1` set; each page contains a `.TH` roff header; the root page exists; nested children produce distinct hyphen-joined filenames (no `split.1` collision);
- **exact-page-set, build-state-explicit (I-2):** the produced set equals the set derived from walking the **UNBUILT** `Cli::command()` tree minus `is_hide_set()` (and minus the auto `help`), explicitly including `mnemonic-gen-man.1` and `mnemonic-gui-schema.1`; do NOT bake a magic integer (M-2);
- **NEGATIVE canary (I-2 / C-1):** assert NO produced filename matches `*-help.1` or `*-help-*.1` — the tripwire for an accidental pre-build / help-shadow-tree regression;
- **NO assertion that any page (root or nested) contains `--no-auto-repair` (C-2)** — the `global=true` flag renders in zero pages under clap_mangen 0.3.0; such an assertion is RED on day one.

**Per-CLI MINOR bump. Done-criteria:**
- **Run the FULL `cargo test -p mnemonic-toolkit` package suite — NOT the man-gen test target alone (M-8).** The `cli_gui_schema.rs` golden-vector regression is INVISIBLE to a target-scoped run (`cargo test --test gen_man`) and surfaces only at the full-package run (the established `lint_argv_secret_flags` RED-through-P4/P5 lesson). Declaring P1 green on a targeted run alone is prohibited. Run the sibling full-package suites likewise (`cargo test -p md-cli`/`ms-cli`/`mk-cli`).
- format the new `gen_man.rs` (+ sibling files) with `cargo +1.95.0 fmt` and verify the pinned `fmt (pinned 1.95.0)` gate (`rust.yml:51`) is green — NEVER bare `cargo fmt` (uses 1.85.0), NEVER touch `mlock.rs` (M-5).

**P2 — manual + GUI-schema mirrors (lockstep).** Add the 4 `gen-man` lines to `cli-subcommands.list` + `gen-man` sections in `41/42/43/44-*.md`; run `make -C docs/manual lint` green. Add `gen-man` `SubcommandSchema` (only the `--out` Path flag, no dropdown enum — minimal, M-4) to all 4 `mnemonic-gui/src/schema/*.rs` (paired PR; `schema_mirror` green against the bumped toolkit pin). File the cross-repo FOLLOWUP companions. **Done-criteria:** `cargo +1.95.0 fmt` clean on any new Rust, mlock.rs untouched.

**P3 — install.sh man step.** `MAN_DIR` default + `--man-dir`/`--no-man` flags (modelled on `--no-gui`/`EXCLUDE`/`selected()`; POSIX `/bin/sh` only, no bash-isms — M-3) + `--dry-run` support + the per-component post-install hook on the **successful**-install branch (the `installed_count` increment — crates.io `:327`, git `:340`; the paired `if cargo install … ; then` lines are `:326`/`:339` (M-9); exclude GUI) + the unconditional cross-platform tail hint (M-1). **The hook line MUST be `|| `-guarded so a `gen-man` failure is non-fatal under `set -eu` (I-1)** — covering the crates.io-latest-vs-pinned-tag version-skew window (a freshly-published toolkit may run against a sibling whose crates.io-latest still lacks `gen-man`), read-only `MAN_DIR`, and disk-full. **TDD assertions:** (a) a `shellcheck` pass on `install.sh` is clean; (b) a dry-run / grep harness asserts the `gen-man` invocation line is `|| `-guarded (matches `gen-man .* || `, never bare) (I-1); (c) **install-layer canary (M-7 / C-1):** the harness greps the generated man dir and asserts ZERO `*-help*.1` pages before declaring P3 done. Update toolkit version-sites (READMEs ×2, install.sh self-pins to the new sibling tags, `fuzz/Cargo.lock`) + `CHANGELOG.md`.

**P4 (severable, gated by fork #2) — per-CLI release-asset man tarball.** Add a `gh release upload <cli>-man.tar.gz --clobber` step to each CLI's release workflow, modeled on `manual.yml`.

**Each phase:** mandatory R0 GREEN (0C/0I) on the plan-doc *before* code; per-phase TDD (tests before impl); per-phase reviewer-loop persisted to `design/agent-reports/`; mandatory post-implementation whole-diff adversarial review. **Each phase's done-criteria runs the FULL `cargo test -p <pkg>` package suite, not targeted `--test` targets (M-8).** **No content gate on the roff** (clap-faithful by construction — state this explicitly in the plan so a reviewer doesn't ask for one). **fmt discipline applies to every phase:** any new/edited Rust is `cargo +1.95.0 fmt`-clean and the `fmt (pinned 1.95.0)` gate is green; `mlock.rs` is never reformatted (M-5).

---

## 9. Publish / tag train (ordering)

Codecs untouched, so the constraint is the usual "crates.io siblings before the toolkit pin":

1. **md-cli / ms-cli / mk-cli** — land + tag (`descriptor-mnemonic-md-cli-v0.11.0`, `ms-cli-v0.13.0`, `mk-cli-v0.11.0`) + `cargo publish` each (direct-FF + tag, these 3 publish to crates.io). **Note (I-1 corollary):** until ALL three siblings' crates.io-latest carry `gen-man`, an install.sh run that resolves crates.io-latest for a not-yet-republished sibling will hit the guarded warn-and-continue path — intended, non-fatal.
2. **mnemonic-toolkit** — land `mnemonic gen-man` + the `cli_gui_schema.rs` golden-vector edit (C-3) + all 4 manual chapters + install.sh man step + version-sites + CHANGELOG; bump install.sh component_info pins to the new sibling tags + `mnemonic-gui/pinned-upstream.toml`; tag `mnemonic-toolkit-v0.73.0` (git+tag, **no publish** — miniscript `[patch.crates-io]` blocker).
3. **mnemonic-gui** — PR + CI green (schema_mirror fires against the bumped toolkit pin) **before** its own tag; bump `pinned-upstream.toml`. (GUI = PR+CI-before-tag per the constellation gotcha list.)

**Watch the detached-HEAD push gotcha** (`git push origin HEAD:main`, not `origin main`) and verify GUI merges via `gh pr view --json state,mergeCommit` (worktree local-cleanup can fail while the server merge succeeds).

---

## 10. Open questions (for R0 / user)

1. **Visibility (fork #1):** visible (recommended) vs `hide = true`. **Note:** the toolkit `cli_gui_schema.rs` golden-vector edit is mandatory EITHER WAY (C-3); the only behavioral difference is `gen-man`'s presence in `mnemonic --help`.
2. **Release asset (fork #2):** ship per-CLI `*-man.tar.gz` (recommended) vs install.sh-builds-locally only.
3. **macOS auto-discovery:** confirm in the plan whether stock macOS BSD man auto-reads `~/.local/share/man` — TO-VERIFY, not testable in this Linux env. (Does NOT block the tail hint: per M-1 the portable `man -M "$MAN_DIR"` fallback is already unconditional.)
4. **Section override:** ship all pages as section 1 (clap_mangen default) — confirm no section-8/5 split is wanted (no daemon/config pages here, so section 1 is correct).
5. **Global-flag in man output (C-2):** `--no-auto-repair` (`global=true`) renders in ZERO generated pages under clap_mangen 0.3.0 + clap 4.6.1. This is an upstream renderer limitation, NOT a build-state defect. Decide: (a) accept (the flag is still discoverable via `mnemonic --help`); or (b) file an upstream/clap_mangen FOLLOWUP and/or manually append a GLOBAL OPTIONS roff stanza in `gen_man.rs`. v1 recommendation: **accept (a)** + note the limitation in the man chapter prose; do NOT block P1 on it.