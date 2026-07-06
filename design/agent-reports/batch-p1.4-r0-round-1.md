# batch P1.4 per-phase R0 — round 1 (opus architect, adversarial)

**Phase:** P1.4 — combined tutorial re-drive (restore `(none)` + reveal 👁 markers)
**Repo/commit:** `mnemonic-gui` branch `feat/tutorial-surfaced-fixes`, commit `4cd878a` atop `390df12` (verified: `390df12` is the direct parent; branch `feat/tutorial-surfaced-fixes`; tree clean).
**Authority:** plan §P1.4; batch-plan R0 RULING 1 (4-step reveal set) + RULING 2 (`assert_no_plaintext` parameterization).
**Environment used:** `GUI_TUTORIAL_SNAPSHOTS=1 RUSTUP_TOOLCHAIN=stable WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1`; adapter `llvmpipe (LLVM 22.1.6)`, `device_type == Cpu`, `backend: Gl`; pinned CLI on `$PATH` = `mnemonic 0.75.0` (matches schema `pinned_version: "mnemonic 0.75.0"` and `pinned-upstream.toml` tag `mnemonic-toolkit-v0.75.0`).

---

## VERDICT: GREEN — advance to P1.5

0 Critical / 0 Important. 1 Minor (latent, non-blocking, noted below). Every hygiene gate re-run
independently (not trusted from the report); the 4 reveal PNGs, 3 masked modal/run shots, restore
`(none)` shots, and eye-chrome catch-up sample were opened and read visually. Double-run byte
determinism confirmed. Repo left clean at `4cd878a`.

---

## 1. Visual secret-hygiene ruling (PNGs opened + observed)

I opened and read the following committed PNGs (`tests/snapshots/tutorial/`):

### The 4 reveal `-form` shots — each shows the CORRECT allowlisted phrase, EXACTLY one field, no other/non-demo secret
- **`tut-j1-01-bundle-single-sig-form`**: slot `@0 . phrase` field shows **S0 in plaintext** (`…andon abandon abandon abandon abandon abandon about`, scrolled to tail). `--passphrase` empty. **No other secret unmasked.** The `has_mask_sentinel` requirement is satisfied by the command Preview line `…--slot ••••` (the masked argv echo), which resolves the "single-secret-revealed → is there still a `••••`?" question — the preview carries it.
- **`tut-j2-02-convert-fingerprint-form`**: `--from` composite (`phrase` node) field shows **S0 in plaintext** (`abandon abandon abandon abandon abandon ab…`); `--to = fingerprint`. Preview shows `--from •••••` (masked). No other secret unmasked.
- **`tut-j2-03-convert-xpub-form`**: identical to j2-02 but `--to = xpub`; `--from` = **S0 plaintext**; Preview `--from •••••`. No other secret unmasked.
- **`tut-j2-07-bundle-all-seeds-form`** (multi-slot invariant): slot `@0` = `••••••••`, slot `@1` = `••••••••` (both MASKED), slot `@2` (the LAST-driven) shows **S2 in plaintext** (`…amount doctor acoustic avoid letter advice cage above`, scrolled to tail). **Exactly ONE row revealed, the other two masked** — the single-revealed-field invariant holds visually. Reveal target = last drive = S2, correct.

Ruling: all four reveal the reader-typed, allowlisted demo phrase (j1-01 slot@0=S0; j2-02/03 `--from`=S0; j2-07 last slot=S2); no extra unmasked secret; no non-demo secret anywhere.

### Eye-chrome catch-up sample (masked, gained 👁, NO revealed value)
- **`tut-ch0-00-orientation-form`** (clearest): the `--passphrase` row now carries the **👁 eye Button** next to the label with an **empty (masked-capable) field** — the eye chrome is present, no value revealed. This layout shift is exactly why even the orientation shot re-rendered.
- **`tut-j5-22-bundle-json-form`** / **`tut-j3-12-engrave-form`**: `--descriptor` shows a **PUBLIC** wsh/xpub descriptor string (not a secret); xpub slot masked/empty; the secret-widget rows carry the eye chrome; no revealed secret value.

### `-modal` + `-run` of the reveal steps — stay MASKED (latch auto-hidden by the Run click)
- **`tut-j1-01-…-modal`**: "Confirm secret-bearing run" modal argv ends `--slot ••••` (masked); the slot field is re-masked `••••••••`. No plaintext.
- **`tut-j1-01-…-run`**: pane argv echo `…--slot ••••`; masked field; stdout carries only the intended PUBLIC `ms1/mk1/md1` cards; stderr warning is the flag-path-only form (no value). No plaintext.
- **`tut-j2-07-…-run`**: argv `--slot •••• --slot •••• --slot ••••` (all three masked); three flag-path-only stderr warnings; only public `ms1[0..2]` cards. No plaintext.

### Restore `-form` / `-run` — clean md1 flow, Template `(none)`
- **`tut-j3-13-restore-form`** and **`tut-j5-23-restore-descriptor-form`**: `--template` combo shows **`(none)`** (not the old `wsh-sortedmulti` workaround); md1 rows are public.
- **`tut-j4-17-restore-run`**: `--template (none)`; restore **completes** (exit 0; public `importdescriptors` tr(...) output; confirm-fingerprints note). No secret in the pane.

---

## 2. Transcripts — zero delta + no leak (RULING 2, display-only reveal)
- `git diff 390df12..4cd878a -- '*.txt'` = **empty** (0 lines). Transcripts do NOT move (byte-identical to the old multisig-`--template` route-around, as the P1.4 comment claims).
- Only `*.png` and `*.rs` files changed in the diff; nothing else.
- Grep of all **98** committed tutorial transcripts: **zero** hits for `abandon abandon` (S0), `legal winner thank` (S1), `letter advice cage` (S2), lone `abandon`, or `xprv`. The extended-privkey regex `\b[xt]prv[0-9A-Za-z]{50,}` = **empty** (no xprv/tprv string anywhere).
- The CLI's on-argv warning is the **value-free** form: `warning: secret material on argv (--slot @N.phrase=) — pipe via --slot @N.phrase=- …`; the only `@N.phrase=` occurrences are `@N.phrase=` and `@N.phrase=-` (never `=<seed>`). The `tprv|private` grep hit is the generic advisory `stdout carries private key material (can spend) — redirect or encrypt …`, not a key. The `ms1/mk1/md1` cards in the panes are the tool's designed encoded output of the PUBLIC BIP-39 test vectors (allowlisted demo data). Confirmed: the reveal is display-only in PNGs; transcripts carry no seed.

---

## 3. Census / hygiene gates re-run (independent, not trusted from report)

All 12 always-run gates + the capture harness GREEN:
- `reveal_markers_are_rule_derived` — **ok**. Marked set is EXACTLY `{tut-j1-01-bundle-single-sig, tut-j2-02-convert-fingerprint, tut-j2-03-convert-xpub, tut-j2-07-bundle-all-seeds}`; `reveal_marker_violations()` fail-closed BOTH ways (`reveal && !should_mark` and `!reveal && should_mark`), where `should_mark = capture && any(Drive::secret_value().is_some())` — NOT `Step::is_secret()` (which would over-select via `secret_modal`). The 6 restore steps type a PUBLIC `--md1` card (`TypeMd1Chain ⇒ secret_value()==None`) → no marker; the J2 devices-1/2 converts are `capture:false` → no marker.
- `reveal_in_scope_fields_are_checker_classified` (⊆-agreement) — **ok**. Every reveal-marked step's `revealed_value().is_some()`; and for EVERY `SlotSubkey`, `is_secret_bearing() == secrets::slot_subkey_is_secret(sk)` (widget-mask gate == checker taxonomy). Composites mask on the SAME `Drive::secret_value` predicate → agreement by construction. Nothing revealable escapes the allowlist gate.
- `secret_values_are_allowlisted` — **ok** (whole-manifest `check_allowlist(MANIFEST)`; `secret_drive_count() > 0` non-vacuity).
- `corpus_budget_under_ceiling` — **ok**: 50 committed PNGs, 27.101 MiB ≤ 32 MiB HARD ceiling.
- `corpus_png_count_matches_manifest`, `manifest_stems_regen_diff`, `fixtures_carry_no_secret_material`, `pinned_tier_version_gate_bites`, `same_frame_completion_gate_bites`, `same_frame_completion_direct_click_class` — all **ok**.
- `gui_tutorial_snapshots` (the capture harness, byte-compare vs committed for all 50) — **ok** in 131.57s.

`manifest-stems.txt` (`tests/tutorial/manifest-stems.txt`) unchanged in the diff; the 61-form gallery (`tests/snapshots/forms/`) untouched (empty diff-stat).

### The parameterized `assert_no_plaintext` — no hole
- Filled-form checkpoint (`tests/gui_tutorial_snapshots.rs:566-575`): loosened ONLY by `if Some(val) == revealed { continue; }`, where `revealed = step.revealed_value()` (the last secret drive). This skips ONLY the exact deliberately-revealed value — which is allowlist-gated INDEPENDENTLY by `secret_values_are_allowlisted`. `has_mask_sentinel` still asserted first (line 562).
- Populated-pane checkpoint (`:641-647`): iterates ALL `Drive::secret_value` first-word probes with **no `revealed` skip** — UNCONDITIONALLY strict.
- Confirm-modal checkpoint (`:711-716`): iterates ALL first-word probes with **no `revealed` skip** — UNCONDITIONALLY strict.
- Word-disjointness holds: first words `abandon` (S0) / `legal` (S1) / `letter` (S2) are mutually exclusive across the three vectors, so for j2-07 (reveals S2) the still-strict `abandon`/`legal` probes neither false-match the revealed S2 text nor lose their teeth. The single-latch invariant is **core-enforced** (`src/form/secret_widget.rs:56,80-88` — one ctx-transient `Option<egui::Id>`; `set_revealed_field` overwrites), so at most one field can render plaintext.

### Negative BITES (RULING 2) — confirmed on BOTH cases
`allowlist_checker_bites_on_non_allowlisted_secret` — **ok**. Verified in source (`:210-269`): (a) a NON-allowlisted secret drive → `check_allowlist` returns a `synthetic-leak` violation; (b) the SAME step with `reveal: true` → `check_allowlist` still non-empty (the reveal opens no hole, because `check_allowlist` ignores the `reveal` flag and gates every secret drive); (c) positive control — an allowlisted `S0` drive → empty. `check_allowlist` is parameterized over `&[Step]` so the negative feeds a synthetic manifest.

---

## 4. Double-run byte determinism — CONFIRMED
- Run #1 (byte-compare vs committed): all 50 committed PNGs byte-match current-code render (0 `.new`↔committed mismatches; 0 `.diff.png`). Aggregate `sha256(cat sorted 50 PNGs) = 198af24126b6…`.
- Run #2 (independent regen): `test result: ok. 1 passed` in 112.73s; all 50 `.new.png` again byte-identical to committed (0 mismatches). Aggregate `= 198af24126b6…`.
- **Run#1 render ≡ Run#2 render ≡ committed, byte-for-byte.** (My aggregate recipe is `sha256` over the sorted-basename concat of the 50 committed PNGs → `198af24126b6`; this differs in *recipe* from the report's `6d082be6` — likely a different file-set/tool — but the *determinism property* is what matters and it holds; the byte-compare + double-regen prove it directly.) The `.new.png` render artifacts were removed; `git status` clean.

---

## 5. 26-moved census — fully explained (4 reveal + 10 restore + 12 eye-chrome)
The 26 are `Modified` in place (not renamed).
- **4 reveal** `-form`: `tut-j1-01-bundle-single-sig-form`, `tut-j2-02-convert-fingerprint-form`, `tut-j2-03-convert-xpub-form`, `tut-j2-07-bundle-all-seeds-form`.
- **10 restore** (5 capture steps × `-form`+`-run`): `tut-j2-08-restore-{form,run}`, `tut-j3-13-restore-{form,run}`, `tut-j4-17-restore-{form,run}`, `tut-j5-23-restore-descriptor-{form,run}`, `tut-j5-24-restore-core-{form,run}` — all now `--template (none)`.
- **12 eye-chrome** (👁 layout shift and/or auto-hidden reveal, value stays masked): `tut-ch0-00-orientation-form`, `tut-j1-01-…-{modal,run}`, `tut-j2-02-…-run`, `tut-j2-03-…-run`, `tut-j2-06-bundle-watch-only-{form,run}`, `tut-j2-07-…-run`, `tut-j3-12-engrave-form`, `tut-j4-16-engrave-form`, `tut-j5-22-bundle-json-{form,run}`.

Decoys UNTOUCHED (verified in current `tests/tutorial/manifest.rs`): `convert_drives!` `--template wsh-sortedmulti` (line 138) and `tut-j2-07` `--template wsh-sortedmulti` (line 252) — both legitimate non-restore template choices, kept. The `--template ""` sentinel appears at restore_drives! (152, parameterized; callers pass `""`), j5-23/24 (364/369), and the PRE-EXISTING export-wallet F1 `(none)` unlock macros (116/125, not part of this diff).

---

## 6. Broader verification (plan / memory: full package suite, both clippy configs, headless)
- `cargo test --jobs 2` (RUSTUP_TOOLCHAIN=stable): **670 passed, 0 failed** (tutorial + gallery captures auto-skip-as-ok without their env gates).
- `cargo clippy --all-targets -- -D warnings`: exit 0.
- `cargo build -p mnemonic-gui --no-default-features` (headless, zero wgpu/winit): exit 0.
- `cargo clippy --no-default-features -- -D warnings`: exit 0.

---

## Findings

### Critical: none
### Important: none

### Minor (latent, NON-blocking — no action required for P1.4)
- **M-1 (defense-in-depth, not live).** The filled-form loosening `if Some(val) == revealed { continue; }` skips ALL drives whose string equals the revealed value, not just the single latched occurrence. If a FUTURE reveal-marked step drove the SAME phrase into two secret fields (revealing one), the filled-form checkpoint would skip the plaintext probe for the still-masked twin as well. This is inert today: no reveal-marked step repeats a phrase (j1-01=S0-only; j2-02/03=S0-only; j2-07=S0/S1/S2 distinct), the single-latch invariant is core-enforced so only one field can render plaintext, and the unconditionally-strict pane/modal checkpoints + the 61-form masking gallery would catch a genuine masked-field-leak. Optional hardening for a later cycle: scope the skip to the last-matching drive index rather than by value equality. Does not gate P1.5.

---

## Repo state on exit
Clean (`git status` empty), HEAD `4cd878a`, all `.new.png` render artifacts removed. No source or committed-corpus mutation by this review.
