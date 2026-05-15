# Phase P2.4 batch 2 (Track M — 10-foundations) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** §3.2 P2.4 batch 2 — `docs/manual-gui/src/10-foundations/11-what-is-mnemonic-gui.md` (NEW, 68 LOC), `.../12-relation-to-cli.md` (NEW, 78 LOC), `.../13-bundle-mental-model.md` (NEW, 90 LOC), `.../14-secret-handling.md` (NEW, 119 LOC), `pandoc/filters/mermaid-cache-filter.lua` (pre-emptive format-gate fix), `FOLLOWUPS.md` (new entry `gui-manual-html-mermaid-svg`), `.cspell.json` (+7 words).

**Verdict:** **ITERATE 0C / 3I / 1N / 0n.**

The four new chapters land cleanly at H1, the pre-emptive `mermaid-cache-filter.lua` format-gate fix is structurally identical to batch 1's C-1 fold (and was the right call — the FOLLOWUP entry is well-shaped and properly tiered `v1`), all expected anchors resolve in the rendered HTML, and the mermaid block survives in the HTML render as source-preserving `<pre class="mermaid">` while the LaTeX path correctly emits `\includegraphics`. **But three content-accuracy issues land at Important severity:** (I-1) chapter 11's secret-class flag list contradicts the schema's `secret: true` markings (`--mk1` and `--md1` are `secret: false`); (I-2) chapter 14's "three classes of secret material" omits `--bip38-passphrase`, SLIP-39 passphrases, and SLIP-39 share phrases (5+ classes per schema); (I-3) chapter 13's recovery-table is internally inconsistent (says `mk1` alone gives watch-only, but the table prose says `mk1 + md1` needed) AND diverges from CLI manual `30-workflows/35-recovery-paths.md:144`. One nit: terminology divergence on `cross-bind` vs CLI-manual canonical `cross-binding`.

---

## Critical

None.

## Important

### I-1 — Chapter 11 names `--mk1` and `--md1` as secret-class flags; schema marks them `secret: false`

`docs/manual-gui/src/10-foundations/11-what-is-mnemonic-gui.md:37-39` includes `--mk1, --md1` in the secret-class flag enumeration. But `mnemonic-gui/src/schema/mnemonic.rs:329-335` defines `--mk1` with `secret: false`, and lines 337-343 define `--md1` with `secret: false`. The semantic reality matches the schema: `mk1` is `xpub + origin` (public watch-only material) and `md1` is the descriptor / wallet policy (also public). Treating `--mk1` / `--md1` as triggers for the run-confirm modal would be incorrect GUI behavior. Chapter 14 declares the schema's `secret: true` boolean as the single source of truth.

**Fix:** drop `--mk1, --md1` from chapter 11:37-39. Optionally expand chapter 14's three-class list to include the broader `secret: true` schema set.

### I-2 — Chapter 14's "three classes of secret material" omits 5+ schema secret classes

`docs/manual-gui/src/10-foundations/14-secret-handling.md:3-9` enumerates only BIP-39 phrases / `ms1` strings / BIP-39 passphrases. Schema `secret: true` grep across `mnemonic-gui/src/schema/` returns at minimum: `--bip38-passphrase` (`mnemonic.rs:469` — distinct cryptographic passphrase, schema help explicitly distinguishes from `--passphrase`), SLIP-39 `--passphrase` (`mnemonic.rs:783, 854` — mechanically distinct from BIP-39 passphrase, different subcommand), SLIP-39 share phrases (`mnemonic.rs:846, 946` — the actual share-secret material). All flow through `FormState.secret_widgets`. The "three classes" claim is inaccurate against the schema's at-least-five-class set.

**Fix:** rephrase to a broader category sketch tied to the schema's `secret: true` predicate (not exhaustively enumerated; future-schema-additions-safe).

### I-3 — Chapter 13's three-card recovery table omits the `mk1 + md1` watch-only path and conflicts with CLI manual recovery-paths chapter

`docs/manual-gui/src/10-foundations/13-bundle-mental-model.md:37-47` table says `mk1` "alone" recovers a watch-only wallet, but the prose at line 43-44 says it requires `mk1 + md1`. Both can't be true. CLI manual `docs/manual/src/30-workflows/35-recovery-paths.md:144` resolves: "Single-sig, mk1 + md1 only | yes (watch-only); spending requires the seed" — confirming the `mk1 + md1` pair is needed (you need `md1` to know the descriptor template / script type to derive addresses). The table also omits the `mk1 + md1` (no `ms1`) recovery scenario as a first-class row.

**Fix:** restructure to per-CARD-SUBSET framing (matches CLI manual's table) rather than per-single-card "alone" framing.

## Nice-to-have

### N-1 — `cross-bind` terminology diverges from CLI manual's `cross-binding`

`docs/manual-gui/src/10-foundations/13-bundle-mental-model.md:85` uses `cross-bind` (+ `\index{cross-bind}`). The CLI manual uses `cross-binding` consistently (12 sites across `docs/manual/src/`). Two index terms for the same concept will produce inconsistent glossaries when 90-appendices arrives.

**Fix:** replace `cross-bind` with `cross-binding` (4 sites in chapter 13: mermaid arrows + noun-form + `\index{}`).

## Nit

None.

---

## Verification trace

1. **Plan §1.4 coverage:** four files map 1:1 to plan §1.4 line 188-189 description. PASS.
2. **AUTHORING heading-level rule:** all four files start at H1. PASS.
3. **Intra-chapter cross-references resolve:** all four target anchor IDs exist in `build/m-format-gui-manual.html`. PASS.
4. **Forward-link policy:** `#first-launch-walkthrough` (in chapter 13) → batch 4 (30-tour) forward ref. Acceptable interim state. PASS.
5. **Pre-emptive mermaid-filter fix correctness:** format-gate at `mermaid-cache-filter.lua:60`, positioned after the mermaid class short-circuit, comment cites `primer-box.lua:28` + `wrap-long-code.lua:82,96`. PASS.
6. **HTML render preserves mermaid source:** `<pre class="mermaid"><code>flowchart LR…</code></pre>` at HTML line 607. PASS.
7. **PDF render still emits `\includegraphics`:** TeX line 636 emits cache PDF path. PASS.
8. **FOLLOWUP entry shape:** all 5 required fields present; Tier `v1`. PASS.
9. **cspell additions well-grounded:** all 7 words appear in batch-2 prose. PASS.
10. **Schema-anchor non-regression:** 0 accidental schema-anchor matches; 459 RED baseline unchanged. PASS.
11. **Secret-class flag-list accuracy:** **FAIL — see I-1 + I-2.**
12. **Three-card recovery table accuracy:** **FAIL — see I-3.**
13. **Mermaid block source-faithfulness:** flow matches CLI manual's `mnemonic bundle` invocation; see N-1 for terminology. PASS.

---

## Final verdict

**ITERATE 0C / 3I / 1N / 0n.**

The pre-emptive `mermaid-cache-filter.lua` format-gate fix was the right call: structurally byte-identical to batch 1's C-1 fold, properly idiom-mirrored to `primer-box.lua:28`, empirically validated. FOLLOWUP `gui-manual-html-mermaid-svg` properly tiered `v1`.

I-1 + I-2 + I-3 are content-accuracy issues against the schema source-of-truth + CLI manual cross-consistency. After folding all three (plus N-1 terminology), batch 2 is ready for R1.
