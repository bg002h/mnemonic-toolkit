# LANE5-msrv — implementation-ready spec

**Item:** E-GUI-MSRV-gate (Q4 of the open-followups maturity program, toolkit leg only).
Surface/gate the GUI's `rustc ≥ 1.88` MSRV in the install flow and fix the stale/silent
documented install prerequisites. Continuation of FOLLOWUP
`install-sh-gui-sibling-pin-staleness-ungated`'s "rustc-MSRV tension" sub-entry.

**Provenance:** all citations below re-grepped against current `HEAD = cc9f9dc2`
(`fix(diagnostics): md1 card on export-wallet/bundle --descriptor → clear typed refusal`).
GUI facts verified against `mnemonic-gui` working tree (Cargo.toml v0.49.0, Cargo.lock).

**SemVer:** Toolkit **NO-BUMP** (see the `semver` field). The GUI Cargo.toml correction is a
separate mnemonic-gui paired-PR lane, explicitly excluded from this lane.

**Scope guard (funds/secret):** zero funds path, zero secret handling, zero clap surface, zero
wire-shape. Pure shell logic + prose + dict + FOLLOWUPS.

---

## LOCAL VERIFICATION GOTCHA (read first)

The fish profile aliases `md` → `mkdir`. Any local `make … lint`/`audit` invocation MUST pass an
**absolute** `MD_BIN=/abs/path/to/md`, never a bare `md` PATH lookup. CI itself is unaffected
(bash, no fish profile; manual.yml passes `MD_BIN=md` resolving to the installed md-cli; quickstart.yml
passes `MD_BIN=true`).

---

## Change 1 — install.sh: rustc ≥ 1.88 guard that de-selects the GUI (skip-with-warning)

**File:** `/scratch/code/shibboleth/mnemonic-toolkit/scripts/install.sh`

**Current behavior:** `scripts/install.sh` has **no `rustc` check anywhere**. It validates `cargo`
is on PATH (the `── Validate cargo ──` block, lines 175–180) but never reads `rustc --version`.
The GUI component arm is `component_info`'s `mnemonic-gui)` case (lines 43–45, pins
`mnemonic-gui-v0.49.0`, `cratesio=no` so it always takes the git+tag path). The generic install loop
(lines 243–292) runs `cargo install --locked --git "$url" --tag "$tag" … "$pkg"` for the GUI and lets
it **raw-exit-101** on rustc 1.85–1.87 (cargo refuses to downgrade the `--locked` high-MSRV deps —
`icu_collections`/`icu_normalizer`/`icu_properties@2.2.0`, `idna_adapter@1.2.2`, `image@0.25.10`, all
confirmed present in `mnemonic-gui/Cargo.lock`). This is exactly the "GUI silently fails the step"
symptom the user named; the loop's per-component `FAILED` line + non-zero `failed_count` exit is the
only signal.

**Exact edits (three small additions; preserve the README-promised "CLIs install fine, GUI is the
only casualty" contract — SKIP-WITH-WARNING, never hard-fail):**

**(1a) Named constant** near the top of the file (recommended: immediately after the
`component_info()` closing `}` at line 50, before `ALL="…"` at line 52), so the next MSRV bump is a
one-line edit mirroring the pin discipline:

```sh
# Minimum rustc for the mnemonic-gui overlay (its --locked deps' MSRV).
# The 4 CLIs build on the lower toolkit MSRV (rustc >= 1.85). Bump this
# one line when the GUI's dependency MSRV rises. See README.md:33-36 and
# design/FOLLOWUPS.md `install-sh-gui-sibling-pin-staleness-ungated`.
GUI_MIN_RUSTC_MINOR=88
```

(Storing the *minor* as an integer avoids shell float comparison; the floor is `1.88`, and rustc has
not bumped its major past 1, so an integer-minor compare against `88` is sufficient and robust. If you
prefer a full `GUI_MIN_RUSTC=1.88` string, you MUST then split it — do NOT use `[ "$a" -lt "$b" ]` on
dotted strings.)

**(1b) The guard itself** — place it AFTER the cargo presence check (after line 180) and AFTER the
`--only`/`--exclude` overlap + token validation (i.e. after line 229, so `$ONLY`/`$EXCLUDE` are final),
but BEFORE the `── Install loop ──` (line 231). It uses the existing `selected mnemonic-gui` helper
(defined at lines 189–203) to know if the GUI is in the set, then mutates `EXCLUDE` to auto-`--no-gui`
this run:

```sh
# ── GUI rustc-MSRV guard ────────────────────────────────────────────────
# The mnemonic-gui overlay needs a newer rustc than the 4 CLIs (its
# --locked deps' MSRV). On an older toolchain, skip the GUI WITH A CLEAR
# WARNING rather than letting `cargo install` raw-exit-101 mid-loop —
# the run still exits 0 with the 4 CLIs installed (matches README.md
# `--no-gui or upgrade rustc` contract). On any rustc-parse failure we
# FALL THROUGH (attempt the install) — never block a capable user.
if selected mnemonic-gui && [ -z "$DRY_RUN" ]; then
    rustc_ver=$(rustc --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1)
    rustc_minor=$(printf '%s' "$rustc_ver" | cut -d. -f2)
    rustc_major=$(printf '%s' "$rustc_ver" | cut -d. -f1)
    if [ -n "$rustc_major" ] && [ -n "$rustc_minor" ] && [ "$rustc_major" = "1" ] \
       && [ "$rustc_minor" -lt "$GUI_MIN_RUSTC_MINOR" ] 2>/dev/null; then
        echo "warning: mnemonic-gui needs rustc >= 1.$GUI_MIN_RUSTC_MINOR;" >&2
        echo "         your rustc is $rustc_ver — skipping the GUI this run." >&2
        echo "         The 4 CLIs install normally. Upgrade rustc and re-run" >&2
        echo "         to add the GUI (or this is expected if you only want the CLIs)." >&2
        EXCLUDE="${EXCLUDE:+$EXCLUDE,}mnemonic-gui"
    fi
fi
```

**Design notes the implementer MUST honor (regression-risk mitigations, per recon):**
- **Parse-failure fall-through:** if `rustc` is absent or its version doesn't parse,
  `rustc_ver`/`rustc_minor`/`rustc_major` are empty → the `if` is false → the GUI is NOT skipped (the
  loop will then attempt it; a genuinely-too-old toolchain still gets the old exit-101 behavior, which
  is acceptable — we only *improve* the parse-succeeds case, never *block* a capable user).
- **Nightly/pre-release suffix stripping:** `grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1` already
  strips `-nightly`, `-beta`, the trailing ` (hash date)`, etc. (e.g. `rustc 1.97.0-nightly (… )`
  → `1.97.0` → minor `97` → not skipped). Do NOT compare raw `rustc --version` strings.
- **`DRY_RUN` guard:** the `[ -z "$DRY_RUN" ]` means `--dry-run` prints the full 5-component plan
  unchanged (dry-run must not mutate the set; keeps the existing `--list`/`--dry-run` output stable).
- **Mutate `EXCLUDE`, not `ONLY`:** appending `mnemonic-gui` to `$EXCLUDE` works whether the user
  passed nothing, `--exclude X`, or `--only mnemonic-gui,…`. Re-check: `selected()` consults `$ONLY`
  first (line 191) — if the user did `--only mnemonic-gui`, `$ONLY` is non-empty and `selected` returns
  based on `$ONLY` membership, so appending to `$EXCLUDE` is INERT for an `--only` run. **Therefore also
  guard the `--only mnemonic-gui`-on-old-rustc case:** the cleanest fix is to make the warning fire
  regardless, but since mutating `$EXCLUDE` is inert under `$ONLY`, the GUI would still be attempted and
  exit-101. **Implementer decision (TDD this):** when `$ONLY` is non-empty, instead remove
  `mnemonic-gui` from `$ONLY` (e.g. rebuild `$ONLY` dropping the token). Simplest robust approach:
  after emitting the warning, set both `ONLY="$(printf '%s' "$ONLY" | sed 's/mnemonic-gui//; s/,,/,/g; s/^,//; s/,$//')"` AND append to `$EXCLUDE`. **Test both axes** (default-set and `--only mnemonic-gui`).
- **shellcheck cleanliness (quality bar, NOT a CI gate):** no shellcheck workflow exists, but install.sh
  carries inline `# shellcheck disable=…` pragmas implying ad-hoc runs. Quote the `rustc_ver` capture
  (done above), avoid new unquoted expansions, do not introduce a new `SC2086` unless intentional.

**(1c) REQUIREMENTS help-block line** — `usage()` REQUIREMENTS block, lines 112–117. Add a GUI-MSRV
line after the existing `- cargo` line (line 113) or after the Linux-libs line (line 117). Suggested,
matching the existing bullet style:

```
    - GUI only: rustc >= 1.88 (the mnemonic-gui overlay's dependency
      MSRV; the 4 CLIs build on rustc >= 1.85). On an older toolchain
      the installer auto-skips the GUI with a warning; pass --no-gui to
      skip it explicitly.
```

(This is a heredoc body — no markdown, no cspell. Keep `>=` ASCII to avoid encoding surprises in the
`cat <<EOF`.)

**Test/verify surface for Change 1:**
- TDD via `scripts/install.sh --dry-run` assertions is awkward (the guard is gated on `[ -z "$DRY_RUN" ]`).
  Recommended test harness: a small bash test that shadows `rustc` on `$PATH` with a stub printing a
  chosen version, then runs `install.sh --dry-run`? — no, dry-run skips the guard. Instead, run the
  guard logic with a **`--list` is also un-gated**; the cleanest is a dedicated shell unit-test that
  sources/extracts the guard or runs `install.sh` with a stubbed `rustc` AND a stubbed `cargo` (so the
  install loop is a no-op) and asserts on stderr + that the `install … mnemonic-gui` line is absent.
  Place under `scripts/tests/` if one exists, else a self-contained bash assertion in the PR description
  is acceptable for a NO-BUMP shell change (there is no existing install.sh test harness in CI).
- Manual smoke (the load-bearing checks):
  1. Stub `rustc` → `1.85.0`, run with a no-op `cargo`: assert the warning fires AND the GUI is skipped
     AND exit code is 0 with the 4 CLIs "installed".
  2. Stub `rustc` → `1.88.0` and `1.97.0-nightly`: assert NO warning, GUI attempted.
  3. Remove `rustc` from PATH: assert fall-through (no skip; GUI attempted).
  4. `--only mnemonic-gui` on `rustc 1.85.0`: assert the GUI is NOT silently attempted-and-101'd
     (warning + skip; run exits 0 having installed nothing OR clearly skipping — confirm chosen
     semantics in the test).
  5. `--dry-run`: full 5-component plan unchanged.

---

## Change 2 — manual install chapter: fix stale 1.77+ prereq + add GUI ≥1.88 to Path D

**File:** `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/20-quickstart/21-install.md`

**Current behavior:**
- `## Pre-requisites` (lines 17–28) says: "You need a recent **Rust toolchain** — the
  `rust-toolchain.toml` in each repository pins `1.77+`." The stale `1.77+` is on **line 20**.
- `## Path D — graphical interface (mnemonic-gui)` (lines 88–119) documents the GUI install with
  **zero MSRV note**.

**Exact edits:**

**(2a)** In the Pre-requisites paragraph (around line 19–20), correct the stale floor and split the
CLI vs GUI requirement. Replace the sentence "the `rust-toolchain.toml` in each repository pins
`1.77+`" with prose stating the CLIs build on `rustc ≥ 1.85` and the `mnemonic-gui` overlay needs
`rustc ≥ 1.88`. Suggested replacement paragraph (keep `rustc`/`MSRV` inside backticks where used as
literals to align with the existing README style AND to fall under the cspell `` `[^`]+` `` ignore
regex; the dict edits in Change 4 are the belt-and-suspenders backstop):

```
You need a recent **Rust toolchain**. The four CLIs (`mnemonic`,
`md`, `ms`, `mk`) build on `rustc` ≥ 1.85 (the toolkit MSRV). The
optional `mnemonic-gui` overlay (Path D below) currently needs
`rustc` ≥ 1.88 — its dependencies pin a newer MSRV. Install via
`rustup` if you do not have it:
```

**(2b)** In Path D (lines 88–119), add a one-line MSRV note where the GUI from-source/install path is
described (the artifacts list is prebuilt-binary; add the note in the prose paragraph at lines 90–96 or
as a fenced `:::note`-style callout consistent with the chapter's existing `:::primer` block at lines
7–15). Suggested sentence:

```
Building the GUI from source requires `rustc` ≥ 1.88 (its
dependencies' MSRV); the four CLIs build on `rustc` ≥ 1.85. The
constellation installer (`scripts/install.sh`) auto-skips the GUI
with a warning on an older toolchain and still installs the four
CLIs — upgrade `rustc` and re-run to add the GUI.
```

**Test/verify:** prose-only; NOT under the flag-coverage scanner (step 4/6 scans `40-cli-reference/`
only). MD013 line-length is DISABLED in `docs/manual/.markdownlint-cli2.jsonc`, so wrap width is free.
The ONLY risk is cspell on bare `MSRV` (handled in Change 4). Lychee: no new external URLs added.

---

## Change 3 — quickstart install chapter: fix stale 1.77 prereq + add GUI ≥1.88 note

**File:** `/scratch/code/shibboleth/mnemonic-toolkit/docs/quickstart/src/20-singlesig/21-install.md`

**Current behavior:**
- `## Pre-requisites` (lines 7–16): "A recent **Rust toolchain** (1.77 or newer)." — the stale `1.77`
  is on **line 9**.
- `## If you prefer a GUI` (lines 45–55) describes the GUI with **no MSRV note**.

**Exact edits:**

**(3a)** Line 9: replace "(1.77 or newer)" with the corrected CLI floor. Suggested:

```
A recent **Rust toolchain** — the three CLIs build on `rustc` ≥ 1.85.
(The optional GUI in the last section needs `rustc` ≥ 1.88.) If you
don't have one, the easiest install is:
```

**(3b)** In `## If you prefer a GUI` (lines 45–55), add one sentence after the artifacts/links prose
(around line 53–55):

```
Building the GUI from source requires `rustc` ≥ 1.88 (a newer MSRV
than the CLIs' ≥ 1.85). The constellation installer auto-skips the
GUI with a warning on an older toolchain.
```

**Test/verify:** quickstart.yml runs `make lint` (markdownlint + cspell + lychee; the 3-of-6 subset —
no flag-coverage). MD013 disabled. Risk = cspell on `MSRV` and on `rustc` (the quickstart dict has
NEITHER word) — handled in Change 4. No new URLs → lychee unaffected.

---

## Change 4 — cspell dictionaries: add `MSRV` (both) + `rustc` (quickstart)

**Files:**
`/scratch/code/shibboleth/mnemonic-toolkit/docs/manual/.cspell.json`
`/scratch/code/shibboleth/mnemonic-toolkit/docs/quickstart/.cspell.json`

**Current behavior (re-verified):**
- Manual dict (`docs/manual/.cspell.json`): `"rustc"` IS present (in the `"words"` array, line 120);
  `"MSRV"` is **absent**.
- Quickstart dict (`docs/quickstart/.cspell.json`): the `"words"` array (line 17) is
  `["custodied", "pasteable", "PSBT", "rekt"]` — **both** `rustc` AND `MSRV` are absent.
- Both dicts have `ignoreRegExpList` including `` "`[^`]+`" `` (backtick-wrapped tokens are not
  spell-checked) — so a literal in backticks is already safe. But because the prose above may render
  `MSRV` un-backticked in at least one place, and to make the edit robust against future
  un-backticked use, add the words explicitly.

**Exact edits:**
- **Manual** (`docs/manual/.cspell.json`): add `"MSRV"` to the `"words"` array (e.g. alphabetically
  near `"rustc"`/`"rustup"`/`"RUSTSEC"` at lines 120–122 — insert `"MSRV",` in the M-region or beside
  the RUST cluster; cspell `"words"` matching is case-insensitive by default since `caseSensitive` is
  not set, so `MSRV`/`msrv` both pass).
- **Quickstart** (`docs/quickstart/.cspell.json`): change the `"words"` array on line 17 to add BOTH
  tokens:
  `"words": ["custodied", "MSRV", "pasteable", "PSBT", "rekt", "rustc"]`

**Test/verify (this is the highest-priority gate):**
```
cspell --no-progress 'docs/manual/src/**/*.md'        # must exit 0
cspell --no-progress 'docs/quickstart/src/**/*.md'    # must exit 0
```
Without these dict edits, cspell goes RED on the bare word `MSRV` in manual.yml AND on `MSRV`/`rustc`
in quickstart.yml.

---

## CI-gate cascade summary (HOW to verify each — also in `ci_gates_to_verify`)

| Gate | Fires on this lane? | Verdict | HOW to verify locally |
|---|---|---|---|
| `manual.yml` (markdownlint+cspell+lychee via `make audit`) | YES (`docs/manual/**` path) | GREEN after dict edit | `make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=/ABS/md MS_BIN=/ABS/ms MK_BIN=/ABS/mk` → exit 0. ABSOLUTE MD_BIN (fish md→mkdir). Flag-coverage (4/6) scans `40-cli-reference/` only → unaffected. |
| `quickstart.yml` (markdownlint+cspell+lychee via `make lint`) | YES (`docs/quickstart/**` path) | GREEN after dict edit | `make -C docs/quickstart lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=/ABS/mk` → exit 0. |
| `cspell` (sub-gate of both) | YES | RED without Change 4; GREEN with | two `cspell --no-progress` commands above exit 0. |
| `install-pin-check.yml` | only on `mnemonic-toolkit-v*` TAG | GREEN / N-A | `grep -oE 'mnemonic-toolkit-v[0-9]+\.[0-9]+\.[0-9]+' scripts/install.sh \| head -n1` == `mnemonic-toolkit-v0.71.0` (self-pin line 32 untouched). NO-BUMP → never fires. |
| `sibling-pin-check.yml` | every push/PR | GREEN | The parser regex `grep -oE 'echo "[a-z-]+\|https://[^"]+\|…"'` returns the SAME set; the new `GUI_MIN_RUSTC_MINOR=…` constant + guard are NOT `echo "<pkg>\|https://…"` shape and add no `cargo install --git … --tag` literal. Run the gate's inline bash over the edited tree. |
| `manual.yml` flag-coverage / `make audit` (md-pin cascade) | NO | GREEN | Zero clap-flag delta in any of the 4 CLIs → binary `--help` surface byte-identical → no new flag to document. The user-feared md-pin manual cascade does NOT trigger. |
| `cross-tool-differential.yml` | NO | N-A | Paths (`parse_descriptor.rs` + its test + the workflow) untouched → workflow doesn't trigger; funds-oracle Match unaffected. |
| `rust.yml` / `changelog-check.yml` / `bitcoind-differential` / `fuzz-smoke` | NO | N-A | No Rust source, no Cargo.toml, no CHANGELOG entry (NO-BUMP, no tag). |
| shellcheck | NO CI GATE EXISTS | quality bar only | Keep the new shell shellcheck-clean (quoted captures); no workflow enforces it. |

---

## FOLLOWUP flips (apply in the SAME commit per the status-discipline rule)

**File:** `/scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md`
**Entry:** `install-sh-gui-sibling-pin-staleness-ungated` (heading line 278; body lines 280–285).

- The **rustc-MSRV tension sub-entry (line 284)** and the **Status line (line 285)** are the live record.
  Line 285 currently reads `partially-resolved` with "README rustc ≥1.88 prerequisite already
  documented, no new doc work" and an OPEN remainder = the systemic gate gap (option-(a) cross-repo
  `gh api` drift-check, "LANE-PIN-B").
- **Flip the MSRV piece to resolved.** Append to the Status line (line 285) that this lane (E-GUI-MSRV-gate,
  toolkit leg) SHIPPED: (i) the `scripts/install.sh` rustc ≥ 1.88 skip-with-warning guard +
  REQUIREMENTS help line; (ii) the three stale/silent doc prereqs corrected (manual `21-install.md`
  Pre-requisites 1.77+→1.85/1.88 + Path D GUI ≥1.88 note; quickstart `21-install.md` Pre-requisites
  1.77→1.85/1.88 + GUI ≥1.88 note); (iii) cspell dict additions (`MSRV` both, `rustc` quickstart).
  **Do NOT** mark the whole slug resolved — the **systemic cross-repo gh-api drift-check (option a /
  LANE-PIN-B) remains OPEN**, and the **GUI Cargo.toml `rust-version` "1.85"→"1.88" correction + GUI
  README prereq line are a SEPARATE mnemonic-gui paired-PR** (not shipped by this toolkit lane). Word
  the flip as: "MSRV-surfacing piece RESOLVED (toolkit leg, install.sh gate + doc completion);
  remaining OPEN: systemic drift-check (option a) + GUI Cargo.toml/README correction (paired PR)."
- **Note on a stale figure in line 285:** the current Status text references
  `mnemonic-gui-v0.40.0→v0.48.1` but `scripts/install.sh:44` now pins `mnemonic-gui-v0.49.0`. This is a
  pre-existing snapshot decay, not introduced by this lane; the implementer MAY refresh the figure to
  `…→v0.49.0` while editing the Status line (low-cost correctness), but it is not required by this lane.

**No PROGRAM-doc edit required** by this lane (Q4 at `PROGRAM_open_followups_maturity_2026-06-22.md:268`
and the Wave-3 row at `:117` describe the decision; updating the program ledger is the
program-maintenance step, optional here). If the implementer touches it, note Q4's toolkit leg as
delivered and the GUI-paired-PR + systemic-gate remainders as still open.

---

## Out-of-scope (do NOT touch in this lane)

- **GUI Cargo.toml `rust-version = "1.85"` → `"1.88"`** (`mnemonic-gui/Cargo.toml:5`, confirmed false:
  real MSRV ≥1.88 per the locked icu_*@2.2.0 / idna_adapter@1.2.2 / image@0.25.10) and the **GUI README
  install-section prereq line** — these are the Part-2 mnemonic-gui paired-PR (GUI = PR+CI-before-tag).
  Treat the rust-version edit as a CORRECTION of a wrong value (a doc/metadata fix), not a
  regression-forcing MINOR.
- **toolkit README.md:33–36** — already correct (the FOLLOWUPS-cited done-half). Leave as-is. (An
  OPTIONAL light reword to mention the new install.sh auto-skip is allowed but not required; if done,
  it's still prose-only and rides the same manual/quickstart-unrelated `README.md` which is NOT under
  either docs lint gate.)
- The **dedicated GUI `@1.88.0` MSRV CI job** (the only thing that converts the declared MSRV into a
  CI-tested invariant) — see `open_questions`; surface to user, do not silently add.