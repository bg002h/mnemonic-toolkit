## R0 Review — SPEC_manual_gui_v1_1_modernization (Cycle-C P0)

**VERDICT: GREEN — 0 Critical / 0 Important / 4 Minor.** This is one of the highest-fidelity SPECs I have reviewed in this constellation. Every numeric claim, citation, and mechanism was independently re-derived LIVE against `mnemonic-toolkit` `origin/master` (`9a5f605d`) and `mnemonic-gui` (`v0.49.0` = `eecdb275`, HEAD `f8edc72` = v0.49.0+2), running the lint's OWN extractor (`extract_gui_schema.py`) and counting logic (`build_expected`, `expected_outlines`) against both the `v0.3.0` and `v0.49.0` schema sources. No claim required copying the recon; all schema sizes were re-derived from first principles.

### (a) The 5 pin bumps — CORRECT and authority-verified
- `[mnemonic-gui] tag` v0.3.0 → v0.49.0: v0.49.0 is the canonical current release tag; HEAD is v0.49.0+2 with `Cargo.toml` still `0.49.0` — pinning the tag (not HEAD) is right. ✓
- The 4 implied CLI tags (toolkit-v0.70.0 / md-cli-v0.7.0 / ms-cli-v0.8.0 / mk-cli-v0.9.0) **exactly match** GUI v0.49.0's own `pinned-upstream.toml` `[mnemonic]/[md]/[ms]/[mk]` `tag` fields (verified via `git show mnemonic-gui-v0.49.0:pinned-upstream.toml`). This is the correct authority — the GUI's schema was generated against those CLI tags, so they cannot orphan/miss anchors. ✓
- Confirmed the 4 CLI tags are IDENTICAL between v0.48.1 (G1-B's draft) and v0.49.0, so the SPEC's "reuse G1-B comment, swap only the `[mnemonic-gui] tag`" is correct. ✓
- The §1 "comment is STALE (lines 41/48/55)" claim verified: the GUI's pin format is now a `[mnemonic]/.../[mk]` table, not lines 41/48/55. The rewrite is load-bearing and correct. ✓

### (b) Anchor inventory — re-derived LIVE, EXACT
Re-ran the extractor + `build_expected`/`expected_outlines` against both tags:

| metric | v0.3.0 | v0.49.0 | delta | SPEC | match |
|---|---:|---:|---:|---:|:--:|
| subcommands | 28 | 56 | +28 | +28 | ✓ |
| flags | 161 | 407 | +246 | +246 | ✓ |
| variants | 270 | 502 | +232 | +232 | ✓ |
| **anchors** | **459** | **965** | **+506** | **+506** | ✓ |
| outlines | 59 | 128 | +69 | +69 | ✓ |

Per-tab anchor deltas: mnemonic +425, ms +56, mk +18, md +7 (sum = 506) — **all match §2.1 exactly**. Per-tab outline deltas mnemonic +56/ms +9/mk +3/md +1 — match. **All 28 per-subcommand budgets in §2.2 verified** (flags/anchors/outlines triple, incl. subtle "Dropdown 0v → no flag-outline" and "NodeValueComposite 1v → no flag-outline" annotations for `addresses --chain`, `seedqr --from`, `ms derive --template/--network`, `mk address`). **All 7 §2.3 growth deltas verified** (+10/+6/+8/+14/+5/+5/+5 with exact new-flag sets). I also independently scanned for ANY unlisted existing-chapter growth — **§2.3 is exhaustive** (seed-xor/slip39 chapters correctly did NOT grow). Subset invariant holds: **0 anchors / 0 outlines removed** from the schema side. Final reconciliation closes: mnemonic 387 new-chapter + 38 in-place = 425.

### (c) Coverage mechanism — correctly understood
`check_gui_schema_coverage.py` exit-0-iff-(missing==∅ AND orphans==∅), anchor-id derivation (`<tab>-<sub>-<flag>-<variant>`, kebab rule), `is_schema_shaped` orphan exemption for `-outline`-suffixed and non-`<tab>-<sub>`-prefixed anchors, the `id="..."`-any-element regex, and the strict column-0 bullet-count outline rule (`### Outline` for subs ≥2 flags, `#### Outline` for enum flags ≥2 variants) — **all read accurately** against the live scripts. Phase numbering 4/7 gui-schema-coverage + 5/7 outline-coverage confirmed in `lint.sh`. The planned chapter structure (per §7 recipe: mechanical outline-from-schema, exact F/V bullet counts) DOES satisfy the gate.

**§2.4 orphan trap — EMPIRICALLY REPRODUCED.** I pointed the CURRENT built HTML at the v0.49.0 schema (simulating the re-pin): the lint reported **exactly 7 orphans, character-for-character the §2.4 set** (the 3 `select-descriptor-{all,active-receive,active-change}` variants + `env-var-channel`, `no-auto-repair`, `walkthrough-bsms`, `walkthrough-core`) and 497 missing. The 506-vs-497 gap is fully explained (9 import-wallet base anchors pre-exist; 506−9=497 reconciles exactly). This is a genuinely sharp catch that the recon missed, and the SPEC's rename-to-`iw-*` / delete-variants / backfill-missing-flags remediation is correct.

### (d) Branch/merge gating — CORRECT
Verified the manual is currently GREEN at v0.3.0 (459 anchors, 0 orphans). The re-pin turns `gui-schema-coverage` RED until PE. The dedicated `manual-gui-v1.1-modernization` branch + merge-only-at-PE-7/7-GREEN gating is sound: the lint is global (exits 1 on ANY missing/orphan), so intermediate phase commits MUST stay off master. The per-tab "done when `<tab>-*` subset closes" claim is consistent with the lint's per-anchor reporting. `manual-gui.yml` Job-1 clone mechanism (parse `tag`, `git clone --depth 1 --branch`, set `MANUAL_GUI_UPSTREAM_ROOT`) and Job-3 release (CHANGELOG.md as `--notes-file`) verified — §1's CHANGELOG version-site gate claim is correct.

### (e) The ••• prose fix — located + correct
GUI redaction code verified live at v0.49.0: `SECRET_MASK = "••••"` (`invocation.rs:137`), `assemble_argv_with_secret_mask` (`:152`), `render_copy_command_masked` (`:524`), modal per-token mask (`main.rs:960`,`:1091-1095`), argv-echo masked (`main.rs:480`, v0.39.0 Item-1-D3). All 5 stale-prose sites verified at cited lines (14-secret-handling 79-114; 11-what-is 45-49; 32-run-and-output 95-97/134-145/174-176 incl. the ASCII modal `phrase=abandon...` art and the stale "does not redact `argv:` echo" claim; 84-secrets-and-os table 14-18; 42-bundle 17-21). The §4 "recon named 2, live grep found 5 — SPEC correction" is borne out. G1-B `758a44cc` (+ revert `a3ff1c3f`) exists, touched chapters 11+14+pin+cspell+FOLLOWUPS, and its comment block is the exact reword §1 prescribes. cspell: current 194 words, G1-B added exfiltration+unredacted (196), `sentinel` not yet present — §4's "add if not present" is correct. The §4/§8 deferral of the "multi-row widgets do not auto-mask" claim to phase-time re-verification against `runner.rs` is the right posture (the mask is computed whole-argv at assembly time, so it likely IS stale — but the SPEC correctly instructs grep-don't-assume). GUI companion `gui-run-confirm-modal-secret-redaction` confirmed `resolved` v0.39.0 (no GUI source PR needed). FOLLOWUPS slug at line 1006 confirmed.

### (f) Phasing — executable
File-numbering for new chapters is free and consistent: md `5a-repair.md` (md goes 51-59), mk `77/78/79` (mk goes 71-76), ms `67/68/69/6a` (ms goes 61-66), mnemonic 4c-import-wallet.md exists (reconcile). The new-subcommand lists per tab (mnemonic +20, md +1, ms +4, mk +3) match the live schema diff exactly. The §7 per-chapter recipe (schema-skeleton → CLI-manual prose reframe → exact mechanical outline bullets → worked-example with canonical abandon seed → index) is concrete enough for authoring agents, and correctly identifies the bullet-count-exactness as the dominant failure mode with the right mitigation (generate FROM the schema flag list).

### Minor findings (4) — none gate-affecting
1. §2.4 labels `--select-descriptor` as Text (source is Text, extractor reports Dropdown/0v due to a comment-match quirk in the fallback regex) — zero gate impact, same single anchor either way; only kind-mismatch in the whole schema.
2. §2.4 "ADD 506" is the schema-delta view; 9 import-wallet base anchors pre-exist, so net new authoring is ~497 + 7 orphan fixes (reconciles to the live 497-missing). §7 recipe still produces correct final state.
3. §2.5 cites single lines but each count word appears 2-3× per overview file; target counts (30/9/9/8) are correct.
4. §3's sub-outline prose says `### Outline` while §7 and 44-convert.md use `## Outline` — internal wording inconsistency only; the phase-5 gate matches `^(#+)` at any depth so it cannot fail. Also confirmed lychee runs `--offline` without `--include-fragments`, so §2.4 anchor deletions won't trip phase-3 on dangling fragments (though the existing import-wallet `## Outline` must be regrown 7→13 bullets and the deleted-variant Outline bullets removed — implied by §7 step 1).

**Recommendation: GATE GREEN. Proceed to fold the optional Minors (or accept as-is) and advance to P1.** The 4 Minors are clarity refinements that authoring agents following §7+§8 (re-run the extractor per phase) will self-correct; none block implementation. Per the reviewer-loop discipline, if any Minor is folded, re-dispatch a scoped convergence check before authoring begins — but no Critical/Important blocks this gate.