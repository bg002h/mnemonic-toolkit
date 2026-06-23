## R0 adversarial review — `design/SPEC_wave3_toolkit_bundle_lane.md`

**VERDICT: GREEN — 0 Critical / 0 Important / 3 Minor. Cleared to implement.**

Source SHA at review time `0ca0d69e7682…` matches the spec's declared write-time SHA exactly. Every cited path/line was re-grepped against current `master`; all load-bearing citations are accurate, and where the spec corrects an upstream recon/FOLLOWUP citation it is right to do so.

### Citation re-verification (all PASS)
| Claim | Spec | Live | OK |
|---|---|---|---|
| friendly.rs code wording | `:165` `"mk1 mixed case in input string"` | exact | ✓ |
| SPEC §6.4.4 OLD row | `:659` `"mixed case in mk1 input string"` | exact | ✓ |
| integration test needle | `:574` `mk1 mixed case in input string` | exact (fn at :567 — see Minor) | ✓ |
| unit test survives | `:871` needle `"mixed case"` (substring of code) | exact | ✓ |
| install.sh pins | md :35 v0.7.1 / ms :38 v0.7.0 / mk :41 v0.8.0 / GUI :44 v0.40.0 / self :32 v0.71.0 | exact | ✓ |
| manual.yml | mk :79 / md :86 / ms :90 | exact | ✓ |
| quickstart.yml mk | `:73` v0.8.0 | exact | ✓ |
| cross-tool-differential md | `:50` v0.7.1 | exact | ✓ |
| rust.yml comment | `:44` `ms-cli-v0.7.0`; exemption 45–49 | exact | ✓ |
| 44-mk-cli prose pin | `:12` `mk-cli-v0.7.0` (spec correctly overrides recon's wrong "v0.8.0") | exact | ✓ |
| 44-mk-cli --pretty row | `:389` `(ignored when --out is set)` | exact | ✓ |

### The Important-fold anchors (the heart of the R0 census) — all PASS
- `- **Status:** \`open\`` occurs **69×** (spec: 69); bare `- **Status:** open` **12×** (spec: 12); `- **Status:** open. **Tier:** \`cross-repo\`.` **2×** at lines **77 + 124** (spec correctly names line 77 = `mstar-prepolicy-key-backup`).
- **W3-2**: Why-deferred UNIQUE at 1390, status 1391. ✓
- **slug-1**: header 120, Why-deferred UNIQUE at 123, status 124 (collides with 77). Two-line block anchor correct; collision called out by slug name. ✓
- **slug-2**: Tier line 285 UNIQUE (count=1) → single-line edit safe. ✓ (see Minor #2 re: ellipsis)
- **slug-4**: header 4041, Severity UNIQUE at 4047, status 4048 (bare `open` collides 12×). Two-line block anchor correct. ✓
- **Minor #2 fold (mk-vectors)** confirmed: status at **2403** (not 2406); spec §8 cites 2403. ✓
- **Minor #5 fold** confirmed against live mk-cli checkout: doc-comment now `vectors.rs:22` (slug's :23), honored-branch `:70-71` (slug's :70-74), and `--pretty` IS honored under `--out` (branches on `pretty` at :70-71) — so the manual's `(ignored…)` is genuinely wrong and the corrected prose is accurate. Out-of-scope sibling site; no toolkit edit. ✓

### CI-gate analysis — the hardest scrutiny (per the prompt's HARD CI-GATE DISCIPLINE)
- **sibling-pin-check.yml (CI-only, no path filter):** the gate scans EXACTLY 5 workflow `--tag` lines (mk×2, ms×1, md×2 — enumerated live). After the edits: mk all three sites → v0.10.1 (install.sh:41 == manual.yml:79 == quickstart.yml:73); ms two sites → v0.11.0 (install.sh:38 == manual.yml:90); md all three HELD at v0.7.1. **Per-pkg equality holds.** Atomicity is sound: all moving sites are in ONE commit, so no split-push intermediate REDs. **No workflow contains a `--tag mnemonic-gui` line** (verified) → the GUI bump cannot RED this gate. ✓
- **rust.yml g6-invariant (CI-only) — the prompt's "ms g6-invariant" concern:** the job **dynamically resolves the ms-cli pin from install.sh** (`awk '$1=="ms-cli"{print $3}'`), so after the bump it checks out `mnemonic-secret@ms-cli-v0.11.0`. **`git diff ms-cli-v0.7.0 ms-cli-v0.11.0 -- crates/ms-cli/src/mlock.rs` is EMPTY** → byte-identical → toolkit mlock.rs (unchanged) still byte-equal to the sibling → **g6 GREEN, no codex32/mlock re-baseline.** The rust.yml:44 comment edit is itself under rust.yml's path filter so the whole rust.yml (g6 + fmt + test/clippy/miri) re-fires — all GREEN. ✓
- **rust.yml fmt (1.95.0, mlock exempt):** exemption stays valid because v0.11.0 mlock == v0.7.0 mlock (NOT 1.95.0-formatted). Exemption logic lines 45–49 untouched; only the cosmetic literal at :44 changes. mlock.rs correctly never `cargo fmt`-ed; spec §5 forbids `cargo fmt --all`. ✓
- **manual.yml flag-coverage:** fires on `docs/manual/**` + its own file (both edited). `lint.sh:93` is `grep -qF -- "$flag"` (flag-NAME only) → the `--pretty` prose edit cannot RED it; no flag add/remove; forward-only (manual ahead of binaries) → not a G1-B trap. ✓
- **cross-tool-differential.yml:** md leg EXCLUDED → :50 untouched, none of its path-filter files change → does NOT re-fire. The LANE-C walker-baseline coupling is correctly side-stepped. ✓
- **manual-gui.yml gui-schema-coverage (the G1-B-class CI-only trap that bit Wave-2):** keys on `docs/manual-gui/pinned-upstream.toml`'s `mnemonic-gui-v0.3.0` — verified live and correctly listed in the DO-NOT-TOUCH set. Different axis from install.sh's GUI pin → NOT re-fired. ✓
- **GUI schema_mirror (lives in mnemonic-gui):** no clap flag/subcommand/dropdown add/remove/rename in this lane (binary byte-identical, NO-BUMP) → flag-NAME parity unaffected even at a future GUI pin bump. The prompt's "golden snapshots captured from the pinned binary" question is N/A: this lane produces no Rust code and no snapshots. ✓
- **Tag-gated gates** (install-pin-check, changelog-check): NO tag created → do not fire → no CHANGELOG entry required. NO-BUMP is internally consistent (crate version 0.71.0 unchanged, self-pin already v0.71.0, no `crates/` source / tests / Cargo.toml touched → READMEs + fuzz/Cargo.lock version sites untouched and correct). ✓

### Prompt-question scorecard
(a) Every change does what it claims without breaking a CI gate, including the CI-only sibling-pin-check (atomic, GUI-safe), the g6-invariant (proven byte-identical), and the GUI schema-mirror (no flag change). ✓
(b) Golden snapshots: N/A — no Rust code, no snapshots in this lane. ✓
(c) Pin de-stale is ATOMIC (one commit) and md-leg correctly EXCLUDED across all 3 md sites. ✓
(d) mlock.rs correctly excluded from fmt (`--all` forbidden; exemption proven still valid). ✓
(e) SemVer NO-BUMP + version sites: complete and consistent (nothing to bump). ✓
(f) No scope creep: zero references to export-wallet / W3-4 / W3-5; md-leg, LANE-PIN-B, and the LANE-MK source fix all explicitly deferred. ✓

### Minor findings (non-blocking; do not gate)
1. §1.1 pairs `fn inspect_mixed_case_mk1_codec_attributed` with line 574 (the assertion); the fn is at 567. The load-bearing assertion-line citation is correct and the file is DO-NOT-TOUCH.
2. §2.3.2 slug-2 OLD anchor is ellipsized; the implementer must expand `…` to the full unique line-285 text before Editing (the NEW text already preserves the full Tier sentence).
3. §6's "GUI excluded" wording is a nuance: GUI is in the parsed table but unreferenced by any workflow `--tag` line, so it is unchecked rather than excluded — the conclusion (GUI bump can't RED the gate) is correct.

**Recommendation: GREEN. Proceed to the single-implementer commit.** Folding the 3 Minors is optional polish; re-dispatch not required.