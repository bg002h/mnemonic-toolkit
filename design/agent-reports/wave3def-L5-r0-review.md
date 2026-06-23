## R0 review — LANE5-msrv spec — VERDICT: GREEN (0 Critical / 0 Important / 5 Minor)

**Reviewed against:** `HEAD = cc9f9dc2` (matches the spec's stated provenance SHA). All citations re-grepped live; all load-bearing behaviors empirically exercised, not just read.

### Gate result
**0 Critical, 0 Important → GREEN. Implementation may proceed.** The five Minor findings are all confined to the spec's *exploratory test-strategy prose* and latent-robustness notes; none blocks code, none requires a fold.

---

### What I verified (and how)

**Change 1 — install.sh rustc guard (the highest-risk element): EMPIRICALLY ROBUST.**
I extracted the exact proposed guard logic into a `#!/bin/sh; set -eu` probe and ran it across six version strings. Results, all correct, script reaching end with exit 0 every time:
- `rustc 1.85.0 (hash date)` → **SKIP-GUI** (suffix/date stripped, minor=85 < 88). ✅
- `rustc 1.88.0` → attempt GUI (floor met). ✅
- `rustc 1.97.0-nightly (…)` → attempt GUI (**nightly suffix stripped**, minor 97). ✅
- empty / rustc-absent → **fall-through, attempt GUI** (parse-failure path, never blocks a capable user). ✅
- `garbage no version` → fall-through, attempt GUI. ✅
- `rustc 2.0.0` (hypothetical major 2) → attempt GUI (`rustc_major != "1"` → skip-condition false; never blocks a future-major user). ✅

This directly discharges the task's required checks: robust `rustc --version` parse (nightly/pre-release stripped), **de-selects GUI + exits 0 (not hard-fail)**, **falls through on parse-failure**. I additionally confirmed `install.sh` runs under `set -eu` (line 18) — the spec is silent on this, but the guard's compound-`if`-condition + `2>/dev/null` structure is `errexit`-exempt and `nounset`-safe (`${EXCLUDE:+…}`, all new vars assigned-not-referenced). The skip-with-warning → `exit 0` contract is real: GUI added to `$EXCLUDE` → loop prints `skip mnemonic-gui` and `continue`s → `failed_count` stays 0 → final block exits 0 with "N installed." (verified against scripts/install.sh:232-304).

**Structural anchors (re-grepped):** component_info closing `}` @50 ✅; GUI arm @43-45 ✅; `ALL` @52 ✅; Validate-cargo @175-180 ✅; token-validation @228-229 ✅ (so `$ONLY`/`$EXCLUDE` are final there); Install-loop header @231 ✅; REQUIREMENTS heredoc @112-117 ✅; `selected()` @189-**202** (spec said 189-203 — off-by-one, Minor). The self-pin is at line 32 = `mnemonic-toolkit-v0.71.0`; the spec's "self-pin line 32 untouched" claim holds.

**sibling-pin-check / install-pin-check stay green: VERIFIED against the actual parser.** The CANONICAL regex is `grep -oE 'echo "[a-z-]+\|https://[^"]+\|…"'`; the new `GUI_MIN_RUSTC_MINOR=88` constant and the `echo "warning: …"` guard lines contain no `|https://…` and add no `cargo install --git … --tag` literal, so neither the CANONICAL set nor the workflow-scan set changes. install-pin-check fires only on `mnemonic-toolkit-v*` tags (NO-BUMP → never fires). **No component_info pin line is touched** — confirmed.

**Changes 2 & 3 — doc prereq fixes: accurate.** Manual `21-install.md`: `## Pre-requisites` @17, stale `1.77+` on **line 20** ✅; Path D @88 (prebuilt-artifacts only — the spec correctly acknowledges there is no from-source path and slots the note into the prose). Quickstart `21-install.md`: `## Pre-requisites` @7, stale `1.77` on **line 9** ✅; `## If you prefer a GUI` @45 ✅. The quickstart's **three**-CLI framing matches the spec's "three CLIs build on rustc ≥ 1.85" wording. **GUI floor facts confirmed**: `mnemonic-gui/Cargo.lock` contains `icu_collections/normalizer/properties@2.2.0`, `idna_adapter@1.2.2`, `image@0.25.10` (all the cited high-MSRV deps); GUI `Cargo.toml` declares `rust-version = "1.85"` (the wrong value the spec correctly fences into the separate paired-PR) at v0.49.0.

**Change 4 — cspell dict (the spec's stated highest-priority gate): EMPIRICALLY REQUIRED AND SUFFICIENT.** With cspell 8.19.4:
- Manual: bare `MSRV` **FAILS** on the current dict (3 hits); after inserting `"MSRV"` near the rust cluster → **0 issues, exit 0** on the full proposed prose including the `≥` glyph. `rustc` already present @120 (no edit needed). ✅
- Quickstart: bare `MSRV` AND lowercase `msrv` **FAIL** on the current dict; after the spec's exact `"words"` edit → **0 issues, exit 0**. Case-insensitive matching means the single `"MSRV"` entry covers both casings. ✅ (One nuance, Minor: `rustc` is already inherited via `import: [../manual/.cspell.json]`, so the spec's risk note overstates the `rustc`-in-quickstart risk — the added `rustc` token is harmless belt-and-suspenders.)
- markdownlint (G1-B-class gate): MD013 line-length disabled in BOTH `.markdownlint-cli2.jsonc` files; a probe of the exact proposed prose → **0 errors**.

**FOLLOWUP flip + out-of-scope: accurate.** Slug heading @278; rustc-MSRV sub-entry @284; Status @285 reads `partially-resolved`. The spec correctly flags the pre-existing stale figure `v0.40.0→v0.48.1` vs the live `scripts/install.sh:44` pin `v0.49.0` and makes the refresh optional. PROGRAM-doc Q4 @268 and Wave-3 row @117 both verified; the spec correctly says no PROGRAM edit is required. README.md:33-36 verified already-correct (CLIs ≥1.85, GUI ≥1.88, `--no-gui or upgrade rustc` contract) → correctly left as-is. No new external URLs (lychee unaffected). `cross-tool-differential.yml` / `rust.yml` / `changelog-check.yml` triggers confirmed not to fire (paths `parse_descriptor.rs` / `crates|Cargo.*` / `mnemonic-toolkit-v*` tag respectively).

**LOCAL GOTCHA respected:** all my doc-lint runs used the dedicated cspell/markdownlint binaries directly; no `md` PATH lookup, no `cargo fmt`, no mlock.rs touch. Working tree left CLEAN (all probes removed, both dicts restored — confirmed via `git status`).

---

### Minor findings (informational; no fold required before code)
1. **`--list` cannot exercise the guard** — it `exit 0`s during arg-parse (install.sh:149-165), before the guard at >229. The spec's test-strategy sentence suggesting otherwise is wrong; delete it. Manual-smoke items 1-5 are correct and self-sufficient.
2. **Unanchored `s/mnemonic-gui//` sed** — safe for the current component set (verified no substring collision; `mnemonic` prefix survives), latently fragile vs a future `mnemonic-gui-*` name. Prefer token-exact removal.
3. **`--only mnemonic-gui` on old rustc** lands on `0 installed` exit-0; ensure the stderr warning is unmistakable (it is) and TDD this exact axis (smoke item 4).
4. **`rustc`-in-quickstart is redundant** (already inherited via import) — harmless; the load-bearing token is `MSRV`.
5. **Citation/coverage drift:** table omits `manual-gui.yml` + `technical-manual.yml` (both correctly N-A on this lane's paths); `selected()` is 189-202 not 189-203; spec silent on `set -eu` (proven safe — preserve the compound-`if` shape, don't refactor into a bare `[ … ] || …`).

**Recommendation:** proceed to implementation. Optionally fold the deletion of the `--list` test sentence (Minor #1) for clarity, but the gate is GREEN as written.