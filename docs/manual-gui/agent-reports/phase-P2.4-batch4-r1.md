# Phase P2.4 batch 4 (Track M — 30-tour) — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit) + `manual-gui-help-icons` (mnemonic-gui)
**Scope:** R1 verification of R0 folds (4C / 1I / 2n) for `30-tour/{31,32,33}.md`, `10-foundations/{11,14}-*.md`, `mnemonic-gui/FOLLOWUPS.md`, `design/FOLLOWUPS.md`, `.cspell.json`.

**Verdict:** **LOCK 0C / 0I / 0N / 0n.** All 5 must-fold findings byte-correct; both optional nitpicks also folded correctly.

## Per-fold verification

| Fold | Status | Evidence |
|---|---|---|
| **C-1** Pinned-tag format (3 sites) | PASS | `31-first-launch.md:21` `Pinned: mnemonic 0.13.0`; `:91` `Pinned: mk 0.3.1`; `33-help-icons-and-deep-links.md:37` `Pinned: mnemonic 0.13.0`; clarifying paragraph at 31:40-46 distinguishes runtime banner (`mnemonic 0.13.0`) vs git-tag (`mnemonic-toolkit-v0.13.0` in `pinned-upstream.toml`). Source-truth confirmed: `mnemonic-gui/src/schema/{mnemonic,mk}.rs` `pinned_version: "mnemonic 0.13.0"` / `"mk 0.3.1"`. |
| **C-2** Closed-selector display | PASS | `31:91` diagram shows bare `subcommand: inspect ▾`; `:102-109` prose now correctly inverts: closed shows bare CLI name (`inspect`), open dropdown shows human form (`Inspect (structural commentary)`); `Encode (xpub → mk1)` example added. Matches `main.rs:328-345` `selected_text(&active_sub)`. |
| **C-3** Escape→Cancel claim (2 sites) | PASS | `14:68-73` Defense 2 now says "no Escape-key affordance: you must click **Run** or **Cancel** explicitly" with security-relevant-modal threat-model rationale; `32:130-132` cross-references §14 Defense 2. `grep -i 'escape\|consume_key\|key::escape' src/` in mnemonic-gui returns only `form/invocation.rs` shell-quoting matches (none in modal code). |
| **C-4** GUI FOLLOWUP placement | PASS | `mnemonic-gui/FOLLOWUPS.md:170` `gui-run-confirm-modal-secret-redaction` is the LAST `###` entry under `## Deferred to v0.3+` (line 124), immediately above `## Resolved in v0.2` (line 180). Legacy bullets (`gui-code-signing-*`, `gui-os-snapshot-secret-occlusion-linux`) at 150-168 sit before the new `###` entry as required. Modern entries `gui-help-icon-per-flag-affordance` (130) + `gui-manual-base-url-runtime-override` (140) intact. |
| **I-1** mk inspect mock | PASS | `32:25-37` uses real field names `xpub:`, `origin_fingerprint:`, `origin_path:`, `policy_id_stubs:` (plural), `chunks:`, `xpub_fingerprint:`, `component[N]:`, `chunk[N]:` matching `docs/manual/src/40-cli-reference/44-mk-cli.md:107-167`. Real xpub prefix `xpub6CatWdiZi...VMrjPC7PW6V`. Fictional summary line `mk1 v1, account-level xpub, BIP-84 (P2WPKH), mainnet` removed. |
| **n-1** Slot-row sketch polish | PASS (folded) | `31:29-30` now `@ [0] . [ xpub ▾ ] = [             ] [✕]` + `[ + Add slot ]` row. |
| **n-2** kebab no-op clarification | PASS (folded) | `33:119-125` adds "In practice this is a no-op on real subcommand names ... but it is applied unconditionally for safety". |

## Hygiene

- **`.cspell.json`** — exactly the 5 expected additions (`redactions`, `airgapped`, `sneakernet`, `fullwidth`, `codepoint`), all referenced in batch-4 prose (`14:81,95,100`; `33:14`).
- **Cross-repo lockstep** — toolkit-side companion `gui-run-confirm-modal-secret-redaction-manual-companion` at `design/FOLLOWUPS.md:48` correctly cross-cites `bg002h/mnemonic-gui` `FOLLOWUPS.md` `gui-run-confirm-modal-secret-redaction`; GUI-side reciprocates.
- **Cross-batch anchors** — `#first-launch-walkthrough` (chapter 31 H1) resolves the pre-committed `13-bundle-mental-model.md:73` reference; `#secret-handling` cross-refs from chapters 11/12/32 still resolve.
- **Chapter 11 inherited-bug remediation** — `11:43-49` correctly removes the false `***` claim and points to `[§14 Secret handling](#secret-handling)` Defense 2; chapter 14:79-114 `:::danger` admonition with cold-node/Blockstream-Satellite/sneakernet operational mitigation is source-faithful.
- **No new drift** in surrounding prose; closed-selector inversion + pinned-format clarifier + Defense-2 rewrite all read coherently.

## Lint + build state (from architect-side verification)

- Phases 1-3 GREEN.
- Phase 4 (gui-schema-coverage) RED at exactly **459** missing — unchanged from batch-3 baseline. Expected (no schema-driven content in batch 4).
- Phase 5 (outline-coverage) RED at exactly **59** missing — unchanged from batch-3 baseline. Expected.
- Phases 6-7 WARN-skip (90-appendices arrives batch 10).
- `make pdf` produces **42 pages** (was 30 at batch-3 baseline → 42 at batch-4 = +12 for 3 new tour files + chapter-14 Defense-2 expansion).
- `make html` produces **14 H1 chapters** (was 11 at batch-3 baseline → 14 at batch-4 = +3 for 31/32/33).

## Final verdict

**LOCK 0C / 0I / 0N / 0n.** All R0 must-fold findings byte-correct against source-of-truth; both nitpicks polished; lint + build state matches expectation. Batch 4 ready to commit.
