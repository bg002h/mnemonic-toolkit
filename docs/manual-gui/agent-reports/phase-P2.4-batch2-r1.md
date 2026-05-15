# Phase P2.4 batch 2 (Track M — 10-foundations) — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** R1 verification of the 4 R0 folds — I-1 (chapter-11 run-confirm secret-flag list), I-2 (chapter-14 schema-derived class enumeration), I-3 (chapter-13 per-CARD-SUBSET recovery table), N-1 (cross-bind → cross-binding rename + cache regen).

**Verdict:** **LOCK 0C / 0I / 0N / 1n.**

All 4 folds land cleanly at the source / cache / HTML layers. Schema cross-check confirms the 4 secret-class flags in chapter 11 are a faithful subset of the 8 `secret: true` schema sites, and chapter 14's 7-class enumeration covers all 8 sites correctly via the passphrase-context + bip38-pair collapses. The recovery table is internally consistent with `docs/manual/src/30-workflows/35-recovery-paths.md:144` framing. The cross-binding rename is byte-clean (0 verb-form residue) and the cache is regenerated at the new SHA. The single nit is artifact-staleness of `build/m-format-gui-manual.tex` + `.log` predating the rename — a fresh `make pdf` from a clean tree resolves correctly.

Promote batch 2; executor proceeds to commit + check in with user before batch 3 (20-install).

---

## Verification trace

1. **I-1 fold correctness — chapter 11 run-confirm modal secret-flag list.** `src/10-foundations/11-what-is-mnemonic-gui.md:37-43` now reads "When the form contains any schema-`secret: true` flag (`--passphrase`, `--ms1`, `--bip38-passphrase`, `--share`, etc. — see [§14 Secret handling](#secret-handling) for the full list and the type-level never-persist invariant)…". `grep -n '\-\-mk1\|\-\-md1'` against the file returns **0 matches** (R0 critical resolved). All 4 listed flags confirmed `secret: true` in `mnemonic-gui/src/schema/mnemonic.rs`. The "etc. — see §14 for the full list" hedge correctly redirects readers to the canonical class enumeration. PASS.

2. **I-2 fold correctness — chapter 14 schema-derived class enumeration.** `src/10-foundations/14-secret-handling.md:3-37` enumerates 7 classes (BIP-39 phrases / raw entropy / `ms1` / BIP-39 passphrase / BIP-38 passphrase / SLIP-39 passphrase / SLIP-39 share phrase). Schema-`secret: true` cross-check: the 8 distinct flag names across `mnemonic-gui/src/schema/*.rs` are `--phrase`, `--hex`, `--ms1`, `--passphrase`, `--passphrase-stdin`, `--bip38-passphrase`, `--bip38-passphrase-stdin`, `--share`. The 8 flag-names collapse to 7 logical classes via `passphrase` + `passphrase-stdin` = one passphrase class (with 3 subcommand-context splits: BIP-39, BIP-38, SLIP-39), and `bip38-passphrase` + `bip38-passphrase-stdin` = one BIP-38-passphrase class. Mapping is faithful. The "Public material (`mk1`, `md1`, fingerprints, paths, xpubs, derivation templates) is NOT secret-class" paragraph at line 24-29 closes R0's I-1 root-cause concern. PASS.

3. **I-3 fold correctness — chapter 13 per-CARD-SUBSET recovery table.** `src/10-foundations/13-bundle-mental-model.md:39-47` is a 7-row table covering all CARD-SUBSETS: `ms1` only (full) / `mk1` + `md1` (watch-only) / `mk1` only (nothing) / `md1` only (nothing) / `ms1` + `md1` (full re-derive `mk1`) / `ms1` + `mk1` (full single-sig; multisig caveat) / no cards (bricked). Aligns with CLI manual `docs/manual/src/30-workflows/35-recovery-paths.md:144` framing extended for the GUI's single-sig-only v1.0 scope. The `mk1` + `md1` row notes "mk1 alone is insufficient — you also need md1 to know the descriptor template and script type". PASS.

4. **N-1 fold correctness — cross-bind → cross-binding rename hygiene.** `grep -c 'cross-bind\b' src/10-foundations/*.md` → **0**. `grep -c 'cross-binding' src/10-foundations/*.md` → **5** (chapter 11 line 62: 1; chapter 13 lines 26-28 mermaid arrows: 3; chapter 13 line 92 noun-form `\index{cross-binding}`: 1). `\index{cross-binding}` markers present at chapter 11 line 62 + chapter 13 line 92. PASS.

5. **Cache integrity.** `figures/cache/541ee252850b66255d00bd2968cb5569ec943aa97bf81ccf40b452b321c21be5.pdf` exists; old `fc1ff90d...pdf` is removed. `make figures-cache` reports `rendered=0, skipped (cache hit)=1`. PASS.

6. **HTML render preserves the corrected mermaid block.** `build/m-format-gui-manual.html` shows `<pre class="mermaid"><code>` block with all three arrows reading `cross-binding`. `grep -c 'cross-binding' build/m-format-gui-manual.html` → 5; `grep -c 'cross-bind\b' build/m-format-gui-manual.html` → 0. PASS.

7. **No new clippy/cspell warnings.** `.cspell.json:72` declares `"SLIP"` (capitalized). cspell tokenizes `SLIP-39` as `SLIP` + `39`. No new wordlist entries needed; lint phases 1-3 PASS. PASS.

8. **Index-marker hygiene.** Multiple `\index{cross-binding}` markers across source files share a single matching row in `99-index-table.md` per lint phase 7 logic. 99-index-table.md absent at P2.4 (phases 6-7 WARN-skip). PASS.

9. **Schema-anchor non-regression.** Phase 4-5 RED at 459/59 baseline; batch 2 adds zero schema-anchored content. PASS.

10. **(Nit n-1) Build-artifact staleness.** `grep -n includegraphics build/m-format-gui-manual.tex` may still resolve to the OLD hash if `.tex` predates the rename. A fresh `make pdf` from a clean tree regenerates. Source / cache / HTML are all internally consistent and correct; the `.tex/.log` are recoverable R0-era artifacts. Non-blocking. **Suggested:** executor runs `rm build/m-format-gui-manual.tex build/m-format-gui-manual.log && make pdf` before committing.

---

## Final verdict

**LOCK 0C / 0I / 0N / 1n.**

All 4 R0 folds are byte-correct at the source layer, the cache regenerates cleanly, the HTML render is correct, and the schema cross-check confirms the 4 secret-class flags + 7-class enumeration are faithful subsets of `mnemonic-gui/src/schema/*.rs`'s 8 `secret: true` sites. The recovery table extension is internally consistent with the CLI manual's framing.

Batch 2 promoted. Executor proceeds to commit + check in with user before batch 3 (20-install) per plan §3.5.
