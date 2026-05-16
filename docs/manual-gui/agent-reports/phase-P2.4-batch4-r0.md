# Phase P2.4 batch 4 (Track M — 30-tour) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit) + `manual-gui-help-icons` (mnemonic-gui).
**Scope:** §3.2 P2.4 batch 4 — `docs/manual-gui/src/30-tour/31-first-launch.md` (NEW, ~110 LOC), `32-run-and-output.md` (NEW, ~140 LOC), `33-help-icons-and-deep-links.md` (NEW, ~150 LOC), `.cspell.json` (+5 words), inherited-bug remediation in batches-1/2 prose at `10-foundations/11-what-is-mnemonic-gui.md` and `10-foundations/14-secret-handling.md` (Defense 2 rewrite + new `:::danger` admonition with cold-node operational mitigation), plus cross-repo FOLLOWUPS lockstep (toolkit `design/FOLLOWUPS.md` companion entry + GUI-side `FOLLOWUPS.md` `gui-run-confirm-modal-secret-redaction` entry).

**Verdict:** **ITERATE 4C / 1I / 0N / 2n.**

The cross-repo lockstep work is well-formed: the FOLLOWUP entries cross-cite each other correctly, the `:::danger` admonition with cold-node operational mitigation (Blockstream Satellite + bitcoind `loadblock` sneakernet) is source-faithful to the actual GUI behavior, the chapter 11 feature-2 description correctly removes the false `***` claim, and the architect's source-grep for the redaction drift was the right discipline. The 91-button (28+43+20) breakdown matches schema source; the 11-flag-name list and per-tab attributions match schema source; both canonical worked-example strings (`mk1qprsq...854wq4` and `ms10entrsq...34v7f`) are grounded across `docs/manual/` and the canonical all-`abandon` BIP-39 test vector. The H1 `# First-launch walkthrough` produces the pre-committed anchor `#first-launch-walkthrough` correctly, satisfying the pre-existing reference at `10-foundations/13-bundle-mental-model.md:73`.

**Top issues:** the same source-grep discipline that caught the redaction drift surfaced four additional load-bearing source contradictions: (1) the `Pinned: <version>` display string format is wrong (uses git-tag form `mnemonic-toolkit-v0.13.0` where source uses runtime-banner form `mnemonic 0.13.0`); (2) the subcommand-selector "shows the human-readable name" claim is inverted (`selected_text(&active_sub)` shows the bare CLI name in the closed selector; only popup options show `human_name`); (3) the modal Escape→Cancel claim is unfounded (`egui::Window` has no built-in Escape handling and source has no manual intercept); (4) the GUI-side FOLLOWUP placement is ambiguous (sits between the modern `###`-form entries and a legacy bullet-list block, not at the end of `## Deferred to v0.3+` per task spec). One Important: chapter 32's mock-stdout block uses fictional `mk inspect` field names that don't match `40-cli-reference/44-mk-cli.md:107-167`.

---

## Critical

### C-1 — Pinned-tag display format drift (3 sites)

**Where:**
- `docs/manual-gui/src/30-tour/31-first-launch.md:21` — `| Pinned: mnemonic-toolkit-v0.13.0 ...`
- `docs/manual-gui/src/30-tour/31-first-launch.md:82` — `| Pinned: mk-cli-v0.3.1 ...`
- `docs/manual-gui/src/30-tour/33-help-icons-and-deep-links.md:37` — `Pinned: mnemonic-toolkit-v0.13.0 ...`

**Why source contradicts:** `mnemonic-gui/src/schema/mod.rs` documents `pub pinned_version: &'static str` as the **runtime `--version` banner** that the soft-check compares against `<cli> --version`. Actual source values are `pinned_version: "mnemonic 0.13.0"` (`schema/mnemonic.rs`), `"mk 0.3.1"` (`schema/mk.rs`), `"ms 0.2.1"` (`schema/ms.rs`), `"md 0.5.0"` (`schema/md.rs`). `main.rs:325-326` renders this verbatim via `ui.label("Pinned:"); ui.monospace(sch.pinned_version);`.

**Fix:** replace the three diagram strings with the runtime-banner format (`mnemonic 0.13.0`, `mk 0.3.1`); add a clarifying paragraph that the git-tag form (`mnemonic-toolkit-v0.13.0`) and the runtime banner (`mnemonic 0.13.0`) are intentionally distinct artifacts.

### C-2 — Subcommand-selector "shows the human-readable name" claim is source-incorrect

**Where:** `docs/manual-gui/src/30-tour/31-first-launch.md:82` (diagram), `:93-97` (prose).

**Why source contradicts:** `main.rs:328-345`:

```rust
let active_sub = self.active_subcommand.get(&active_tab).cloned()...;
egui::ComboBox::from_label("subcommand")
    .selected_text(&active_sub)                       // ← shows the BARE CLI NAME
    .show_ui(ui, |ui| {
        for sub in sch.subcommands {
            if ui.selectable_label(active_sub == sub.name, sub.human_name).clicked() {
                self.active_subcommand.insert(active_tab, sub.name.to_string());
            }
        }
    });
```

`active_sub` values are bare CLI names (`"bundle"`, `"inspect"`, etc., set at `main.rs:189-192`). egui ComboBox `selected_text(...)` controls the closed/collapsed display; popup options (inside `show_ui`) render `human_name`. The actual UI is the inverse of the prose: closed selector shows `inspect`, only the open dropdown shows `Inspect (structural commentary)`.

**Fix:** rewrite chapter 31:82 diagram (`subcommand: inspect ▾  ?`) and rewrite the :93-97 prose to invert the closed/open distinction.

### C-3 — Escape→Cancel modal claim is source-incorrect (2 sites)

**Where:**
- `docs/manual-gui/src/10-foundations/14-secret-handling.md:69` — "**Cancel** is the default action if you press Escape."
- `docs/manual-gui/src/30-tour/32-run-and-output.md:117` — "Pressing Escape behaves like **Cancel**."

**Why source contradicts:** `main.rs:512-535` uses `egui::Window` with `.collapsible(false).resizable(false)` and no `.open()`. No `consume_key(Key::Escape)`, no manual escape intercept anywhere in `src/` (`grep -rn -i 'escape\|consume_key\|key::escape' src/` returns only POSIX/Windows shell-quoting `escape` references in `form/invocation.rs`). egui::Window has **no built-in** Escape→close behavior.

The chapter 14:69 claim is inherited drift from batch 2 that the batch-4 rewrite preserved; chapter 32:117 is newly introduced in batch 4 and inherited the same false claim.

**Fix:** rewrite both sites to either delete the Escape claim or invert it ("There is no Escape-key affordance — the security-relevant-modal threat model treats accidental-Escape-fires as worse UX failure modes than requiring a deliberate click").

### C-4 — GUI-side FOLLOWUP entry placement is ambiguous

**Where:** `mnemonic-gui/FOLLOWUPS.md:150` — the new `gui-run-confirm-modal-secret-redaction` entry uses `### header` form and sits between line 148 (end of `### gui-manual-base-url-runtime-override`, the last clean modern entry) and line 162 (start of legacy bullet-list block of `gui-code-signing-*` and `gui-os-snapshot-*` items, which are still under `## Deferred to v0.3+` but without `###` headers).

The task spec said "at the end of `## Deferred to v0.3+`" but the entry sits *between* the modern `###` entries and the legacy bullets, not after the bullets.

**Fix:** move the entry to be the last `### header` entry under `## Deferred to v0.3+`, just above `## Resolved in v0.2` (line 170).

---

## Important

### I-1 — Chapter 32 example panel sketch shows fictional `mk inspect` output fields

**Where:** `docs/manual-gui/src/30-tour/32-run-and-output.md:25-30`.

**Why source contradicts:** `mk inspect` text-mode output (per `docs/manual/src/40-cli-reference/44-mk-cli.md:107-167`) uses field names `xpub:`, `origin_fingerprint:`, `origin_path:`, `policy_id_stubs:` (note plural+s), `chunks:`, `xpub_fingerprint:`, `component[N]:`, `chunk[N]:`. The chapter 32 mock uses `mk1 v1, account-level xpub, BIP-84 (P2WPKH), mainnet` (no such summary line in source), `xpub:` followed by `xpub6CYAr...` (real prefix is `xpub6CatWdiZi...` for the canonical mk1), `origin:` (real field is `origin_path:`), `policy_id_stub:` (real field is `policy_id_stubs:` plural).

Per `[[feedback-architect-must-run-prose-commands]]`, mockups in walkthroughs must be source-faithful or users distrust the manual.

**Fix:** replace the mock with the real field shape from `44-mk-cli.md:107-167` (xpub:, origin_fingerprint:, origin_path:, policy_id_stubs:, chunks:, xpub_fingerprint:, component[N], chunk[N]).

---

## Nitpicks

### n-1 — Chapter 31 slot-row sketch omits `.` and `=` separators and the `✕` button

**Where:** `docs/manual-gui/src/30-tour/31-first-launch.md:29` — `@0  [ xpub ▾ ] [ (empty) ]`.

Actual rendering per `slot_editor.rs:155-173`: `@ [N] . [subkey ▾] = [text] [✕]` with a `[+ Add slot]` button below. Optional polish for batch-4 R1 fold.

**Fix:** rewrite the slot-row sketch to `@ [0] . [ xpub ▾ ] = [             ] [✕]` plus a `[+ Add slot]` line.

### n-2 — Chapter 33:104-109 URL-formula prose says "kebab(subcommand)" but `kebab()` is a no-op for plain-ASCII subcommand names

**Where:** `docs/manual-gui/src/30-tour/33-help-icons-and-deep-links.md:106`.

The `kebab()` rule (`help/url.rs:39-57`) is a no-op on lowercase-ASCII inputs; all real subcommand names are already kebab-shaped. Strictly correct (kebab is the function applied) but a reader might infer non-kebab characters exist. Optional clarification.

**Fix:** add a parenthetical that `kebab(subcommand)` is a no-op in practice (all real subcommand names are pre-kebab) but is applied unconditionally for safety.

---

## Verification trace summary

All R0 verification-matrix items checked against source:

| # | Check | Result |
|---|-------|--------|
| 1 | Modal rendering claims | ✓ verbatim render; ✓ prefix exact; ✓ `should_confirm_run` predicate; ✗ Escape→Cancel (C-3) |
| 2 | Three-panel layout | ✓ tab strip; ✓ ◀ marker; ✓ output-panel checkboxes; ✗ Pinned format (C-1); ✓ ComboBox + ?; ✓ default tab; ✓ default subs; ✓ bundle defaults |
| 3 | mk inspect walkthrough | ✓ flag set; ✓ positional; ✓ human_name; ✓ canonical mk1; ✗ closed-selector display (C-2); ✗ pinned format (C-1) |
| 4 | mnemonic convert walkthrough | ✓ flag set; ✓ human_name; ✓ phrase secret-bearing; ✓ canonical ms1 byte-identical to 14+ cites |
| 5 | Help-icon URL formula | ✓ MANUAL_BASE_URL default; ✓ env var; ✓ kebab rule; ✓ 3 example URLs byte-correct |
| 6 | Three button-class counts | ✓ 28+43+20=91; ✓ 11 flag-name list + per-tab attributions; ✓ TaggedOrIndexed singleton |
| 7 | Pre-committed anchor | ✓ `#first-launch-walkthrough` resolves the 13-bundle-mental-model.md:73 reference |
| 8 | Run prose commands end-to-end | (architect-side; lint state preserved at 459/59 baseline) |
| 9 | FOLLOWUPS lockstep | ✓ cross-cites; ✓ Where fields; ✗ GUI-side placement (C-4) |

After folding C-1/C-2/C-3/C-4/I-1, R1 should LOCK. Nitpicks n-1 + n-2 are non-blocking polish.
