# SPEC — Wave-3 toolkit-bundle lane (ONE atomic NO-BUMP commit)

- **Repo:** `mnemonic-toolkit` (this repo) ONLY. No sibling-repo edits, no GUI paired-PR, no crates.io publish.
- **SemVer:** **NO-BUMP** — no version bump, no tag, no CHANGELOG entry. Doc/CI/install-hygiene only; the toolkit binary + library surface stay byte-identical.
- **Ship mechanism:** ONE atomic commit, direct fast-forward to `master`. Stage paths explicitly (no `git add -A`).
- **Source SHA at write time:** `0ca0d69e7682e3faf79b7995a4332a3167edbce9` (branch `master`). All cited line numbers below were re-grepped against this SHA; re-verify at implementation time if any intervening merge lands.
- **Single TDD implementer.** This is a cosmetic/CI-hygiene bundle — there is no new Rust code and no new test. The "test surface" is verification commands (sibling-pin-check trial-run + full suite + g6 byte-anchor), all listed in §5.

This commit bundles **three file-disjoint NO-BUMP changes**:

1. **W3-2** friendly-mk-codec MixedCase wording — SPEC-amend direction (doc-only; binary byte-identical).
2. **W3-6** LANE-PIN-A de-stale — ms-cli / mk-cli / GUI pins ONLY (md-cli leg **EXCLUDED**, stays at v0.7.1).
3. **W3-3** toolkit-side doc-mirror — fix the `44-mk-cli.md` `--pretty` prose to match the corrected mk-cli help (forward-only; NO pin bump to chase it).

Atomicity scope: the W3-6 pin de-stale MUST be a single atomic commit (see §6). Bundling all three into ONE commit satisfies that; they touch disjoint files.

---

## 1. W3-2 — friendly-mk-codec MixedCase wording (SPEC-amend)

### 1.1 Current behavior (verified)

- **Code (canonical, KEEP):** `crates/mnemonic-toolkit/src/friendly.rs:165`
  ```rust
  E::MixedCase => "mk1 mixed case in input string".to_string(),
  ```
- **SPEC (outlier, FIX):** `design/SPEC_mnemonic_toolkit_v0_1.md:659`, §6.4.4 table row:
  ```
  | `MixedCase` | exit 2 | "mixed case in mk1 input string" |
  ```
- **Integration test PINS the CODE wording** (so the deferral rationale in the FOLLOWUP is STALE):
  `crates/mnemonic-toolkit/tests/cli_hrp_case_insensitive.rs:574`
  (`fn inspect_mixed_case_mk1_codec_attributed`) asserts
  `predicate::str::contains("mk1 mixed case in input string")`.
- **Unit-test row is order-agnostic:** `crates/mnemonic-toolkit/src/friendly.rs:871`
  (`mk_codec_remaining_arms_render_prose`) uses needle `"mixed case"` — a substring of BOTH orderings; survives untouched.

Divergence: word-order swap of `"mk1"` vs `"mixed case"`. The code form (`mk1 <noun-phrase>`) matches the **dominant `mk1 <noun-phrase>` pattern** of the same §6.4.4 table (the majority of rows, lines 660–676, read `mk1 <thing>`; two rows do NOT carry the prefix — `InvalidHrp` at :658 reads `"wrong HRP …"` and `InvalidChar` at :661 reads `"invalid character …"`), so the **code is canonical and the SPEC row is the outlier typo**. (The edit direction is unaffected by the two prefix-less rows — the MixedCase row matches the dominant form once amended.)

### 1.2 Exact edit

Direction: **amend the SPEC to match the code** (binary byte-identical, zero test churn).

- **File:** `design/SPEC_mnemonic_toolkit_v0_1.md`
- **Line 659**, replace:
  - OLD: `| `MixedCase` | exit 2 | "mixed case in mk1 input string" |`
  - NEW: `| `MixedCase` | exit 2 | "mk1 mixed case in input string" |`

**DO NOT** touch `crates/mnemonic-toolkit/src/friendly.rs:165` and **DO NOT** touch `crates/mnemonic-toolkit/tests/cli_hrp_case_insensitive.rs:574`. The whole point of the SPEC-amend direction is that the test stays GREEN untouched. (The alt code-amend path — editing friendly.rs:165 + co-editing the :574 assertion — was explicitly rejected by the orchestrator: it changes user-visible stderr + the binary, forcing test churn at :574.)

### 1.3 FOLLOWUP flip

- **File:** `design/FOLLOWUPS.md`
- **Slug:** `friendly-mk-codec-mixedcase-wording` (header at line 1385).
- **MULTI-LINE Edit anchor required — the status line is NOT unique.** The bare status form `- **Status:** `open`` occurs **69×** in the file, so a literal exact-string Edit on it alone is ambiguous (fails the unique-match check) and a `replace_all` would corrupt 68 unrelated slugs. Anchor the Edit on the slug's **UNIQUE `Why deferred:` line (line 1390)** TOGETHER WITH the following status line (line 1391) as a single contiguous two-line block, and replace both at once:
  - OLD (the contiguous 1390–1391 block):
    ```
    - **Why deferred:** no integration test pins the byte-exact text yet; cosmetic.
    - **Status:** `open`
    ```
  - NEW:
    ```
    - **Why deferred (stale):** the original "no integration test pins the byte-exact text yet" became FALSE at v0.53.3 (`8289eef3`) — `tests/cli_hrp_case_insensitive.rs:574` (`inspect_mixed_case_mk1_codec_attributed`) now pins the code wording byte-exactly.
    - **Status:** `resolved` (Wave-3, NO-BUMP, no tag — doc-only, binary byte-identical): the SPEC §6.4.4 row at `SPEC_mnemonic_toolkit_v0_1.md:659` was amended to match the canonical code wording `"mk1 mixed case in input string"` (the code is the dominant `mk1 <noun-phrase>` form of the table); code + test untouched.
    ```
  The `- **Why deferred:** no integration test pins the byte-exact text yet; cosmetic.` text at line 1390 IS unique (grep-verified — single occurrence), so this two-line block matches exactly once.
- The `- **What:**` line at **1389** still quotes the OLD SPEC string (`"mixed case in mk1 input string"` as "SPEC §6.4.4 row says"). **Leave 1389 AS-IS** — it is a historical description of the pre-resolution divergence (not load-bearing); the resolution direction is recorded on the amended Status line above.

(Keep the `Surfaced` / `Where` / `What` / `Tier` lines as-is.)

### 1.4 Files touched (W3-2)

- `design/SPEC_mnemonic_toolkit_v0_1.md` (line 659)
- `design/FOLLOWUPS.md` (lines 1390–1391)

---

## 2. W3-6 — LANE-PIN-A de-stale (ms / mk / GUI legs; md-cli EXCLUDED)

### 2.1 Current behavior (verified)

Canonical pins in `scripts/install.sh` `component_info` table:

| line | pkg | current pin | action |
|---|---|---|---|
| `scripts/install.sh:35` | md-cli | `descriptor-mnemonic-md-cli-v0.7.1` | **LEAVE UNTOUCHED** |
| `scripts/install.sh:38` | ms-cli | `ms-cli-v0.7.0` | → `ms-cli-v0.11.0` |
| `scripts/install.sh:41` | mk-cli | `mk-cli-v0.8.0` | → `mk-cli-v0.10.1` |
| `scripts/install.sh:44` | mnemonic-gui | `mnemonic-gui-v0.40.0` | → `mnemonic-gui-v0.48.1` |
| `scripts/install.sh:32` | mnemonic-toolkit (self-pin) | `mnemonic-toolkit-v0.71.0` | **NOT IN SCOPE** (self-pin; gated by install-pin-check.yml) |

Locked workflow `--tag` pins that `sibling-pin-check.yml` forces to equal install.sh (per-pkg name match):

| line | pkg | current pin | action |
|---|---|---|---|
| `.github/workflows/manual.yml:79` | mk-cli | `mk-cli-v0.8.0` | → `mk-cli-v0.10.1` |
| `.github/workflows/manual.yml:86` | md-cli | `descriptor-mnemonic-md-cli-v0.7.1` | **LEAVE UNTOUCHED** |
| `.github/workflows/manual.yml:90` | ms-cli | `ms-cli-v0.7.0` | → `ms-cli-v0.11.0` |
| `.github/workflows/quickstart.yml:73` | mk-cli | `mk-cli-v0.8.0` | → `mk-cli-v0.10.1` |
| `.github/workflows/cross-tool-differential.yml:50` | md-cli | `descriptor-mnemonic-md-cli-v0.7.1` | **LEAVE UNTOUCHED** |

Prose install command (NOT scanned by sibling-pin-check — hand-bump):

| line | pin | action |
|---|---|---|
| `docs/manual/src/40-cli-reference/44-mk-cli.md:12` | `mk-cli-v0.7.0` | → `mk-cli-v0.10.1` |

> NOTE: `44-mk-cli.md:12` ground-truth is `mk-cli-v0.7.0` (NOT `v0.8.0` as the recon stated — the recon's "v0.8.0" was wrong; re-grep beats recon). It is independently stale; bump it straight to `v0.10.1`.

Cosmetic comment literal (non-gating; update for accuracy):

| line | literal | action |
|---|---|---|
| `.github/workflows/rust.yml:44` | `ms-cli-v0.7.0` (inside the fmt-exemption comment) | → `ms-cli-v0.11.0` |

> **md-cli leg fully EXCLUDED.** All three md-cli sites (`install.sh:35`, `manual.yml:86`, `cross-tool-differential.yml:50`) stay at `descriptor-mnemonic-md-cli-v0.7.1`. Because install.sh-md == manual.yml-md == cross-tool-differential.yml-md all remain `v0.7.1`, the md axis stays internally consistent → sibling-pin-check stays GREEN for md, and `cross-tool-differential.yml` does NOT re-fire at all (no md edit; its path-filter is `parse_descriptor.rs` / the test / its own file). This is precisely why excluding md sidesteps the LANE-C differential-rebaseline coupling.

Target tags all confirmed published / present locally: `ms-cli-v0.11.0`, `mk-cli-v0.10.1`, `mnemonic-gui-v0.48.1`.

### 2.2 Exact edits

**`scripts/install.sh`**
- Line 38: `ms-cli-v0.7.0` → `ms-cli-v0.11.0` (keep the rest of the line: `ms-cli|https://github.com/bg002h/mnemonic-secret|…|yes|`).
- Line 41: `mk-cli-v0.8.0` → `mk-cli-v0.10.1` (keep `mk-cli|https://github.com/bg002h/mnemonic-key|…|yes|`).
- Line 44: `mnemonic-gui-v0.40.0` → `mnemonic-gui-v0.48.1` (keep `mnemonic-gui|https://github.com/bg002h/mnemonic-gui|…|no|`).
- **Line 35 (md-cli) UNCHANGED.**

**`.github/workflows/manual.yml`**
- Line 79: `--tag mk-cli-v0.8.0 mk-cli` → `--tag mk-cli-v0.10.1 mk-cli`.
- Line 90: `--tag ms-cli-v0.7.0 ms-cli` → `--tag ms-cli-v0.11.0 ms-cli`.
- **Line 86 (md-cli) UNCHANGED.**

**`.github/workflows/quickstart.yml`**
- Line 73: `--tag mk-cli-v0.8.0 mk-cli` → `--tag mk-cli-v0.10.1 mk-cli`.

**`.github/workflows/cross-tool-differential.yml`**
- **Line 50 (md-cli) UNCHANGED.** (Listed here only to assert it is deliberately NOT edited.)

**`docs/manual/src/40-cli-reference/44-mk-cli.md`**
- Line 12: `--tag mk-cli-v0.7.0 --bin mk` → `--tag mk-cli-v0.10.1 --bin mk`.

**`.github/workflows/rust.yml`** (cosmetic comment ONLY)
- Line 44: `# pins the FROZEN `ms-cli-v0.7.0` tag (via scripts/install.sh), whose copy is`
  → `# pins the FROZEN `ms-cli-v0.11.0` tag (via scripts/install.sh), whose copy is`
- **DO NOT** touch lines 45–49 (the exemption logic + the "when next bumped to a 1.95.0-formatted tag, reformat mlock.rs in both repos and drop this exemption" note). The exemption REMAINS valid AND required: `ms-cli-v0.11.0`'s `crates/ms-cli/src/mlock.rs` is **byte-identical** to `ms-cli-v0.7.0`'s (proven: `git diff ms-cli-v0.7.0 ms-cli-v0.11.0 -- crates/ms-cli/src/mlock.rs` is EMPTY), so v0.11.0's copy is ALSO NOT 1.95.0-formatted → mlock.rs stays fmt-exempt and g6 stays byte-equal. **DO NOT `cargo fmt` mlock.rs** (g6-exempt in both repos).

### 2.3 FOLLOWUP flips (W3-6)

Four pin-staleness slugs. Per orchestrator: flip the ones this lane closes; explicitly NOTE the md-leg + LANE-PIN-B remain open.

1. **`install-sh-sibling-pins-stale-vs-flag-bearing-clis`** (header line 120; Status at line 124).
   - The ms/mk/GUI legs are now de-staled by this lane; the **md-cli leg is intentionally held at v0.7.1** (LANE-C differential-rebaseline coupling). So this slug is PARTIALLY closed.
   - **MULTI-LINE Edit anchor required — the status string is NOT unique.** The status line at 124 is `- **Status:** open. **Tier:** `cross-repo`.`, and that EXACT string also occurs at **line 77** (the unrelated slug `mstar-prepolicy-key-backup`). A literal exact-string Edit on it alone is ambiguous; `replace_all` would corrupt line 77. Anchor the Edit on this slug's **UNIQUE `Why deferred:` line (line 123)** TOGETHER WITH the following status line (line 124) as a single contiguous two-line block. The line-123 text begins `- **Why deferred:** bumping the pins is NOT a free `sed` — the `sibling-pin-check` gate forces …` (grep-verified single occurrence). Replace the contiguous block:
   - OLD (the contiguous 123–124 block, i.e. the full line-123 `Why deferred` line as it currently stands + the line-124 status line):
     ```
     - **Why deferred:** bumping the pins is NOT a free `sed` — the `sibling-pin-check` gate forces `manual.yml`/`quickstart.yml`/`cross-tool-differential.yml` to MATCH install.sh's canonical, and TWO of those pins are deliberately FROZEN: `cross-tool-differential.yml:46`'s md-cli is the walker-divergence comparison BASELINE ("a future md-cli tag must not silently move the comparison baseline"), and `rust.yml`'s g6 mlock byte-equality gate pins the FROZEN ms-cli tag (its mlock.rs is NOT 1.95.0-formatted; bumping needs a coordinated mlock reformat in both repos + dropping the fmt exemption). A clean bump must verify the differential still reports Match at the new md-cli tag and that g6 stays byte-equal (P2 didn't touch mlock.rs, so ms-cli v0.7.0→v0.8.0 should be safe — but verify). Doing it under the v0.56.0 release risked reddening those frozen jobs, so the pins stay at the canonical values for this release.
     - **Status:** open. **Tier:** `cross-repo`.
     ```
   - NEW (keep the `Why deferred` line verbatim, flip only the Status line — but they must be replaced as one block to keep the match unambiguous):
     ```
     - **Why deferred:** bumping the pins is NOT a free `sed` — the `sibling-pin-check` gate forces `manual.yml`/`quickstart.yml`/`cross-tool-differential.yml` to MATCH install.sh's canonical, and TWO of those pins are deliberately FROZEN: `cross-tool-differential.yml:46`'s md-cli is the walker-divergence comparison BASELINE ("a future md-cli tag must not silently move the comparison baseline"), and `rust.yml`'s g6 mlock byte-equality gate pins the FROZEN ms-cli tag (its mlock.rs is NOT 1.95.0-formatted; bumping needs a coordinated mlock reformat in both repos + dropping the fmt exemption). A clean bump must verify the differential still reports Match at the new md-cli tag and that g6 stays byte-equal (P2 didn't touch mlock.rs, so ms-cli v0.7.0→v0.8.0 should be safe — but verify). Doing it under the v0.56.0 release risked reddening those frozen jobs, so the pins stay at the canonical values for this release.
     - **Status:** `partially-resolved` (Wave-3, NO-BUMP, no tag) — ms-cli `v0.7.0→v0.11.0`, mk-cli `v0.8.0→v0.10.1`, mnemonic-gui `v0.40.0→v0.48.1` de-staled in `scripts/install.sh` + lockstep workflow `--tag` pins (`manual.yml` mk/ms, `quickstart.yml` mk) + the `44-mk-cli.md:12` prose pin. **The md-cli leg is deliberately HELD at `descriptor-mnemonic-md-cli-v0.7.1`** (all 3 md sites: install.sh:35, manual.yml:86, cross-tool-differential.yml:50) because bumping it re-fires the frozen `cross-tool-differential` walker-divergence baseline (needs a `#[ignore]`-gated differential re-run against md-cli v0.9.x, unprovable in this lane). g6 confirmed byte-equal at ms-cli v0.11.0 (no mlock re-baseline). **Tier:** `cross-repo`.
     ```
   - WARNING: do NOT key the Edit on the bare `- **Status:** open. **Tier:** `cross-repo`.` string — it would mechanically mis-edit line 77 (`mstar-prepolicy-key-backup`).

2. **`install-sh-gui-sibling-pin-staleness-ungated`** (header line 278; Tier at line 285).
   - The GUI **pin de-stale half** is done by this lane (`v0.40.0→v0.48.1`). The **systemic gate gap** (option (a): a cross-repo `gh api` drift-check) is LANE-PIN-B and remains OPEN. MSRV already documented in README (no doc work).
   - **Line 285**, change `- **Tier:** deferred (install-hygiene; …).` to:
     `- **Status:** `partially-resolved` (Wave-3 pin de-stale: `mnemonic-gui-v0.40.0→v0.48.1` in `scripts/install.sh:44`; README rustc ≥1.88 prerequisite already documented, no new doc work). **OPEN remainder:** the systemic gate gap — option (a)'s cross-repo `gh api` drift-check comparing `install.sh`'s GUI pin to the latest `mnemonic-gui-v*` tag — is NOT implemented here (LANE-PIN-B). **Tier:** deferred (install-hygiene; not funds-safety, but it silently ships stale security fixes to GUI installers).`

3. **`manual-yml-sibling-pin-vs-install-sh-drift-gate`** (header line 573; Status at line 579).
   - **Already `resolved`** (2026-05-28; `sibling-pin-check.yml` shipped). Verify-only — **NO EDIT** unless a stale `open` mention is found (none found at write time). Leave as-is.

4. **`sibling-pin-check-skips-manual-prose-install-commands`** (header line 4041; Status at line 4048).
   - The **live stale-prose instance** at `44-mk-cli.md:12` is fixed by this lane (bumped to `mk-cli-v0.10.1` in lockstep with the workflow/install.sh mk pins). The **gate gap itself** (extend `sibling-pin-check.yml` to scan `docs/manual/src/**` prose) is LANE-PIN-B and remains OPEN.
   - **MULTI-LINE Edit anchor required — the status string is NOT unique.** The bare `- **Status:** open` form at line 4048 occurs **12×** in the file; a literal exact-string Edit on it alone is ambiguous and `replace_all` would corrupt 11 unrelated slugs. Anchor the Edit on this slug's **UNIQUE `Severity:` line (line 4047)** TOGETHER WITH the following status line (line 4048) as a single contiguous two-line block. The line-4047 text (`- **Severity:** Medium — a stale prose pin silently ships a wrong-version install instruction to end users; undetected across 2 cycles.`) is grep-verified single-occurrence. Replace the contiguous block:
   - OLD (the contiguous 4047–4048 block):
     ```
     - **Severity:** Medium — a stale prose pin silently ships a wrong-version install instruction to end users; undetected across 2 cycles.
     - **Status:** open
     ```
   - NEW (keep the `Severity` line verbatim, flip only the Status line — but replace as one block to keep the match unambiguous):
     ```
     - **Severity:** Medium — a stale prose pin silently ships a wrong-version install instruction to end users; undetected across 2 cycles.
     - **Status:** `open` (gate gap unaddressed) — but the live stale instance is fixed: Wave-3 bumped `docs/manual/src/40-cli-reference/44-mk-cli.md:12`'s prose `--tag mk-cli-v0.7.0` → `mk-cli-v0.10.1` in lockstep with the workflow/install.sh mk pins. The CLASS fix (extend `sibling-pin-check.yml` to scan `docs/manual/src/**` + quickstart prose) is LANE-PIN-B, still OPEN.
     ```
   - WARNING: do NOT key the Edit on the bare `- **Status:** open` string — it matches 12 lines.

### 2.4 Files touched (W3-6)

- `scripts/install.sh` (lines 38, 41, 44)
- `.github/workflows/manual.yml` (lines 79, 90)
- `.github/workflows/quickstart.yml` (line 73)
- `.github/workflows/rust.yml` (line 44, cosmetic comment)
- `docs/manual/src/40-cli-reference/44-mk-cli.md` (line 12)
- `design/FOLLOWUPS.md` (lines 124, 285, 4048)

---

## 3. W3-3 — toolkit-side doc-mirror for `mk vectors --pretty`

### 3.1 Current behavior (verified)

- **Toolkit doc mirror (FIX):** `docs/manual/src/40-cli-reference/44-mk-cli.md:389`
  ```
  | `--pretty` | indent the JSON output for human readability (ignored when `--out` is set) |
  ```
  The parenthetical `(ignored when --out is set)` is the WRONG claim, mirrored from the stale mk-cli help text.
- **Ground truth in mk-cli source** (out of scope for this lane; ships in LANE-MK): `crates/mk-cli/src/cmd/vectors.rs` `write_per_fixture_files` arm BRANCHES on `pretty` (`serde_json::to_string_pretty` when `pretty=true`) — i.e. `--pretty` IS honored under `--out`. The mk-cli help-text fix ships in LANE-MK.

This is a forward-only-safe documentation correction: the toolkit manual flag-coverage lint checks flag NAME presence only (`grep -qF -- "$flag"` at `docs/manual/tests/lint.sh:93`), never the parenthetical prose. **No toolkit mk-cli pin bump** — do NOT chase the source fix with a pin bump (the v0.8.0→v0.10.1 mk pin bump in §2 is for W3-6 install-hygiene, NOT for the help-prose; the flag-coverage lint never reads help prose, so the prose row is correct independent of which mk-cli is pinned).

### 3.2 Exact edit

- **File:** `docs/manual/src/40-cli-reference/44-mk-cli.md`
- **Line 389**, replace:
  - OLD: `| `--pretty` | indent the JSON output for human readability (ignored when `--out` is set) |`
  - NEW: `| `--pretty` | indent the JSON output for human readability (also applies to the per-fixture files written under `--out`) |`

The flag NAME `--pretty` is unchanged → flag-coverage lint unaffected.

### 3.3 FOLLOWUP flip (W3-3)

**NONE in this lane.** Per orchestrator decision (3), the toolkit doc-mirror is forward-only-safe and ships independently; the W3-3 FOLLOWUP `mk-vectors-pretty-out-help-mismatch` (header line 2397) is NOT flipped here — the source-of-truth fix + the 3-cite lockstep (mk-cli `vectors.rs` + GUI `schema/mk.rs` + companion FOLLOWUP entries) ship in **LANE-MK** (+ GUI), and the slug flips when that source fix lands. Leave `design/FOLLOWUPS.md:2397–2405` UNTOUCHED.

> Implementer note: the W3-3 FOLLOWUP slug at line 2400 cites `vectors.rs:23` (doc-comment) / `vectors.rs:70-74` (honored-under-`--out` branch) / `mk.rs:208`. These citations have drifted by ~one line against the live checked-out mk-cli source: the doc-comment is currently at `crates/mk-cli/src/cmd/vectors.rs:22` and the honored-branch at `vectors.rs:70-71`. **All are OUT-OF-SCOPE sibling sites — no toolkit edit here**, so the drift is harmless to this lane; the only toolkit file W3-3 touches is `44-mk-cli.md:389`. When LANE-MK is authored, **re-grep** mk-cli `crates/mk-cli/src/cmd/vectors.rs` (doc-comment currently :22, honored-branch :70-71) and GUI `src/schema/mk.rs` against their then-current source before citing line numbers.

### 3.4 Files touched (W3-3)

- `docs/manual/src/40-cli-reference/44-mk-cli.md` (line 389)

---

## 4. Complete file manifest (explicit `git add` list)

ONE atomic commit stages exactly these files (no `git add -A`):

```
git add \
  design/SPEC_mnemonic_toolkit_v0_1.md \
  design/FOLLOWUPS.md \
  scripts/install.sh \
  .github/workflows/manual.yml \
  .github/workflows/quickstart.yml \
  .github/workflows/rust.yml \
  docs/manual/src/40-cli-reference/44-mk-cli.md \
  design/SPEC_wave3_toolkit_bundle_lane.md
```

(Add this spec doc itself if persisting it to the repo; otherwise drop the last line.)

Per-file edit summary:

| File | Lines | Change | Item |
|---|---|---|---|
| `design/SPEC_mnemonic_toolkit_v0_1.md` | 659 | SPEC §6.4.4 MixedCase row → `"mk1 mixed case in input string"` | W3-2 |
| `design/FOLLOWUPS.md` | 1390–1391 | flip `friendly-mk-codec-mixedcase-wording` → resolved + de-stale rationale | W3-2 |
| `design/FOLLOWUPS.md` | 124 | `install-sh-sibling-pins-…` → partially-resolved (md held) | W3-6 |
| `design/FOLLOWUPS.md` | 285 | `install-sh-gui-sibling-pin-staleness-ungated` → partially-resolved (pin done, gate open) | W3-6 |
| `design/FOLLOWUPS.md` | 4048 | `sibling-pin-check-skips-manual-prose-…` → live instance fixed, class open | W3-6 |
| `scripts/install.sh` | 38, 41, 44 | ms-cli→v0.11.0, mk-cli→v0.10.1, GUI→v0.48.1 (md UNCHANGED at :35) | W3-6 |
| `.github/workflows/manual.yml` | 79, 90 | mk-cli→v0.10.1, ms-cli→v0.11.0 (md UNCHANGED at :86) | W3-6 |
| `.github/workflows/quickstart.yml` | 73 | mk-cli→v0.10.1 | W3-6 |
| `.github/workflows/rust.yml` | 44 | cosmetic comment ms-cli-v0.7.0→v0.11.0 (exemption logic UNCHANGED) | W3-6 |
| `docs/manual/src/40-cli-reference/44-mk-cli.md` | 12 | prose install pin mk-cli-v0.7.0→v0.10.1 | W3-6 |
| `docs/manual/src/40-cli-reference/44-mk-cli.md` | 389 | `--pretty` row prose corrected (honored under `--out`) | W3-3 |

**Files explicitly NOT touched (assert in review):**
`crates/mnemonic-toolkit/src/friendly.rs` (code wording is canonical),
`crates/mnemonic-toolkit/tests/cli_hrp_case_insensitive.rs` (test stays GREEN),
`scripts/install.sh:35` + `.github/workflows/manual.yml:86` + `.github/workflows/cross-tool-differential.yml:50` (md-cli leg held at v0.7.1),
`crates/mnemonic-toolkit/src/mlock.rs` (g6-exempt; never `cargo fmt`),
`docs/manual-gui/pinned-upstream.toml` (manual-gui's own GUI pin `v0.3.0` — separate axis / G1-B-class gate; NOT in scope),
`design/FOLLOWUPS.md:2397–2405` (W3-3 slug flips in LANE-MK, not here),
`design/FOLLOWUPS.md:573–581` (`manual-yml-sibling-pin-vs-install-sh-drift-gate` already resolved).

---

## 5. Test / verification surface

No new Rust test, no changed Rust test. Verification is the following commands (run locally before push; the orchestrator owns the final pre-push verification):

1. **sibling-pin-check parity (THE load-bearing CI gate — see §5 CI gates).** Re-grep every workflow `--tag` line and confirm per-pkg equality with install.sh canonical:
   ```
   grep -rnE 'cargo install --git +https?://[^ ]+ +--tag +[^ ]+ +[a-z]' .github/workflows/
   grep -nE 'descriptor-mnemonic-md-cli-v|ms-cli-v|mk-cli-v|mnemonic-gui-v' scripts/install.sh
   ```
   Expect: md-cli `v0.7.1` everywhere (install.sh:35, manual.yml:86, cross-tool-differential.yml:50);
   ms-cli `v0.11.0` (install.sh:38, manual.yml:90);
   mk-cli `v0.10.1` (install.sh:41, manual.yml:79, quickstart.yml:73);
   GUI `v0.48.1` (install.sh:44 only — no workflow GUI --tag).
   Optionally run the gate's own bash logic locally against the edited tree (copy the `Verify sibling-CLI pins` step from `sibling-pin-check.yml`) → expect exit 0 with all `OK …` lines, no `::error::`.

2. **Full toolkit suite GREEN** (the §6.4.4 MixedCase wording + the friendly tests live here):
   ```
   cargo test -p mnemonic-toolkit
   ```
   Expect: `inspect_mixed_case_mk1_codec_attributed` PASS (untouched), `mk_codec_remaining_arms_render_prose` PASS (order-agnostic needle). No SPEC/doc edit touches any test.

3. **fmt + clippy clean** (string/comment/doc edits only; mlock.rs untouched):
   ```
   cargo fmt --check && cargo clippy -p mnemonic-toolkit -- -D warnings
   ```
   Expect clean. **Do NOT `cargo fmt --all`** (would re-wrap g6-exempt mlock.rs).

4. **g6 byte-anchor unaffected** (proven, but re-confirm if mnemonic-secret is checked out locally):
   ```
   git -C /scratch/code/shibboleth/mnemonic-secret diff ms-cli-v0.7.0 ms-cli-v0.11.0 -- crates/ms-cli/src/mlock.rs
   ```
   Expect EMPTY → mlock.rs byte-identical → g6 stays GREEN at v0.11.0; fmt-exemption stays valid.

5. **(Informational) manual flag-coverage** — CI runs `make -C docs/manual lint` against the PINNED binaries; it is forward-only (flag-NAME presence) and the manual is already AHEAD of the bumped binaries (43-ms.md documents `--language`/`--group-size`/`--separator`; mk has no new/removed long flags vs v0.10.1). The `--pretty` flag NAME is unchanged, so the W3-3 prose edit cannot RED it. Not locally reproducible without building the v0.11.0/v0.10.1 binaries; CI runs it for real (see §5 CI gates).

---

## 6. CI gates to verify (with HOW + which are CI-only)

| Gate | Fires when | Re-fires here? | Verdict + HOW to verify |
|---|---|---|---|
| **`sibling-pin-check.yml`** (CI-only; fires on EVERY push, NO path filter) | every push/PR | **YES** | **THE load-bearing gate.** Greps install.sh `component_info` → `pkg→tag` table, scans every workflow `cargo install --tag <tag> <pkg>` line, asserts per-pkg equality (md/ms/mk; GUI + toolkit-self excluded). PASSES iff md=`v0.7.1` everywhere, ms=`v0.11.0` (install.sh:38 == manual.yml:90), mk=`v0.10.1` (install.sh:41 == manual.yml:79 == quickstart.yml:73). HOW: §5 step 1 — re-grep + optionally run the gate's bash step locally; expect exit 0. **ATOMICITY:** all install.sh + workflow pins move in this ONE commit, so the gate never sees a split (install.sh ahead of a workflow) intermediate push. |
| **`rust.yml` g6-invariant job** (CI-only; dynamically resolves the ms-cli pin FROM install.sh) | rust.yml triggers (path-filter `crates/**`, `Cargo.toml`, `Cargo.lock`, `.github/workflows/rust.yml`) | the rust.yml comment edit at :44 IS under that path-filter, so rust.yml (incl. g6) re-fires | **GREEN.** g6 greps the ms-cli pin from install.sh (now `v0.11.0`), checks out mnemonic-secret@v0.11.0, byte-compares mlock.rs. PROVEN byte-identical to v0.7.0 → GREEN, no codex32/mlock re-baseline. HOW: §5 step 4. NOTE: an install.sh-only edit would NOT trigger rust.yml (install.sh not in its path-filter) — but the rust.yml:44 comment edit DOES trigger it; either way the outcome is GREEN. |
| **`rust.yml` fmt job** (pinned 1.95.0, mlock.rs exempt) | same path-filter as above | **YES** (rust.yml edited) | **GREEN.** The exemption stays valid: v0.11.0 mlock.rs == v0.7.0 mlock.rs byte-for-byte → still NOT 1.95.0-formatted → still exempt + still required. Only the cosmetic literal at :44 changed; the exemption GREP/logic (lines 45–49) untouched. HOW: §5 step 3 (`cargo fmt --check`). DO NOT `cargo fmt` mlock.rs. |
| **`rust.yml` test/clippy/miri** | same path-filter | **YES** (rust.yml edited) | **GREEN.** Toolkit builds against codec LIBRARIES from crates.io (Cargo.lock), ORTHOGONAL to install.sh CLI tags. No source/test change. HOW: §5 steps 2–3. |
| **`manual.yml` flag-coverage** (`make audit`) | `docs/manual/**` OR `.github/workflows/manual.yml` edits | **YES** (both: manual.yml pin edits + 44-mk-cli.md prose edits) | **GREEN (forward-only).** Installs the NEW pins (mk-cli v0.10.1, ms-cli v0.11.0; md-cli v0.7.1 unchanged), greps each `<bin> <sub> --help` flag NAME into the chapter. `--pretty` NAME unchanged; no new/removed flags; manual already ahead. NOT a G1-B trap (forward-only; manual ahead of binaries REDUCES the gap). HOW: CI-only confirmation; local `make -C docs/manual lint MD_BIN=… MS_BIN=… MK_BIN=…` if binaries are built. |
| **`quickstart.yml`** | `.github/workflows/quickstart.yml` edits | **YES** (mk pin edit) | **GREEN.** MD_BIN/MS_BIN mocked (`true`); only mk-cli real → installs v0.10.1. No quickstart prose pin asserted beyond the workflow line. |
| **`cross-tool-differential.yml`** (#[ignore]-gated md1 differential) | `parse_descriptor.rs` / `cli_cross_tool_differential.rs` / its own workflow file | **NO** | **NOT re-fired** — the md-cli leg is EXCLUDED, so :50 is untouched and none of its path-filter files change. This is the LANE-C coupling we deliberately avoid by holding md at v0.7.1. (If a future cycle bumps the md pin, this gate re-fires and must re-run the `--ignored` differential against md-cli v0.9.x.) |
| **`install-pin-check.yml`** (self-pin↔tag) | `mnemonic-toolkit-v*` tag push ONLY | **NO** | NO tag created (NO-BUMP) → does not fire. |
| **`changelog-check.yml`** | `mnemonic-toolkit-v*` tag push ONLY | **NO** | NO tag → does not fire → no CHANGELOG entry required. |
| **`manual-gui.yml` gui-schema-coverage** (the G1-B-class CI-only trap) | `docs/manual-gui/**` or its pin | **NO** | NOT re-fired — keys on `docs/manual-gui/pinned-upstream.toml`'s `mnemonic-gui-v0.3.0` pin, which this lane does NOT touch. The install.sh GUI bump (v0.40.0→v0.48.1) is a DIFFERENT axis. |
| **GUI `schema_mirror`** (lives in mnemonic-gui; fires on toolkit-binary pin bumps) | GUI pins a new toolkit binary | **NO** | No toolkit flag/subcommand/dropdown add/remove/rename → flag-NAME parity unaffected even at a future GUI pin bump. |
| `bitcoind-differential.yml`, `technical-manual.yml` | their own paths | **NO** | No sibling cargo-install pins; no source change. UNAFFECTED. |

**Net:** SPEC/doc/comment edits + the ms/mk/GUI pin de-stale re-fire `sibling-pin-check`, `rust.yml` (g6/fmt/test all GREEN), `manual.yml` (forward-only GREEN), `quickstart.yml` (GREEN). The md-only `cross-tool-differential` gate is deliberately NOT re-fired. No tag-gated gate fires (NO-BUMP). No G1-B-class gate fires.

---

## 7. Atomicity / ordering

- **ONE atomic commit.** The W3-6 pin de-stale is the binding constraint: `sibling-pin-check` fires on every push with no path filter and asserts install.sh == every workflow `--tag` per pkg. A split push (install.sh ahead of a workflow, or vice-versa) REDs the intermediate. Therefore install.sh:38/41 + manual.yml:79/90 + quickstart.yml:73 MUST all be in the SAME commit. Bundling W3-2 + W3-3 into the same commit is free (disjoint files) and satisfies atomicity.
- No internal ordering between the three items (file-disjoint). Within W3-6, all pin sites change together (atomic).
- Direct fast-forward to `master` after the orchestrator's local verification (sibling-pin-check + full suite + g6) passes.

---

## 8. FOLLOWUP flips summary

> **Anchor note (load-bearing):** three of these status lines are NOT unique strings, so the Edit MUST be keyed on the unique PRECEDING line + status line as a contiguous two-line block (see §1.3, §2.3.1, §2.3.4). The "Edit anchor (status@line)" column gives the status line to flip; the "Anchor on" column gives the unique preceding line that makes the two-line block unambiguous. NEVER key on the bare status string alone for the three flagged rows.

| Slug | Status@line | Anchor on (unique preceding line) | Action |
|---|---|---|---|
| `friendly-mk-codec-mixedcase-wording` | FOLLOWUPS.md:1391 | line 1390 `Why deferred:` (unique) | open → **resolved** (+ de-stale rationale) — W3-2 |
| `install-sh-sibling-pins-stale-vs-flag-bearing-clis` | FOLLOWUPS.md:124 | line 123 `Why deferred:` (unique; status string ALSO at line 77 — `mstar-prepolicy-key-backup`) | open → **partially-resolved** (ms/mk/GUI done; md held) — W3-6 |
| `install-sh-gui-sibling-pin-staleness-ungated` | FOLLOWUPS.md:285 | the Tier line at 285 IS unique → single-line Edit safe | → **partially-resolved** (pin done; gate gap open) — W3-6 |
| `sibling-pin-check-skips-manual-prose-install-commands` | FOLLOWUPS.md:4048 | line 4047 `Severity:` (unique; bare `- **Status:** open` matches 12× otherwise) | live instance fixed; **class stays open** — W3-6 |
| `manual-yml-sibling-pin-vs-install-sh-drift-gate` | FOLLOWUPS.md:579 | n/a (no edit) | already resolved — **NO EDIT** (verify-only) |
| `mk-vectors-pretty-out-help-mismatch` | FOLLOWUPS.md:2403 (`- **Status:** `open`.`) | n/a (no edit — flips in LANE-MK) | **NOT flipped here** — flips in LANE-MK with the source fix — W3-3 |

---

## 9. Deferred notes (NOT in this lane)

- **md-cli pin leg (LANE-PIN-A md half):** install.sh:35 / manual.yml:86 / cross-tool-differential.yml:50 stay at `descriptor-mnemonic-md-cli-v0.7.1`. Bumping to md-cli v0.9.x re-fires the frozen `cross-tool-differential` walker baseline (550-line parse/template.rs delta) — needs a `#[ignore]`-gated `cli_cross_tool_differential -- --ignored` all-Match confirmation against the installed md-cli v0.9.x before the pin moves. Held to a separate cycle.
- **LANE-PIN-B (new CI gates):** (b2) extend `sibling-pin-check.yml` to scan `docs/manual/src/**` + quickstart prose `cargo install --tag` lines; (b3) `install-sh-gui-sibling-pin-staleness-ungated` option (a) cross-repo `gh api` GUI-pin drift-check. Both are independent CI-authoring; not in this lane.
- **W3-3 source-of-truth fix:** mk-cli `crates/mk-cli/src/cmd/vectors.rs` help-text reword + GUI `src/schema/mk.rs` mirror + 2 companion FOLLOWUP entries (mk + gui repos) ship in LANE-MK (+ GUI paired-PR). The slug `mk-vectors-pretty-out-help-mismatch` flips there, not here.
- **`docs/manual-gui/pinned-upstream.toml:19`** `mnemonic-gui-v0.3.0` (the G1-B-class gui-schema-coverage pin) is the deferred v1.1 manual-gui modernization cycle — explicitly NOT bumped here.
