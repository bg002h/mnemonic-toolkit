# PLAN R0 (round 1) — batched cycle `tutorial_surfaced_fixes_batch` (reveal 👁 toggle + restore `--template` `(none)`)

- **Plan under review:** `design/IMPLEMENTATION_PLAN_tutorial_surfaced_fixes_batch.md` (Fable-authored).
- **Reviewer:** opus architect, PLAN R0 hard gate (0C/0I required before ANY implementation).
- **Mode:** READ-ONLY, adversarial. Gate the PLAN (sequencing, hygiene control, completeness), NOT the two upstream specs (both R0-GREEN, not re-litigated). Every load-bearing citation re-verified against live source, not the plan's assertions.
- **Source SHAs at review:** mnemonic-toolkit `master @ 2ed3d369` (the plan commit; src cites identical to parent `16bbe67b` — the plan adds no `src/`). mnemonic-gui `master @ 40156b0` (= `cab940b`/`mnemonic-gui-v0.56.0` + one `FOLLOWUPS.md +7` commit; every `src/`/`tests/` cite is tag-identical). Both confirmed via `git rev-parse`.

---

## VERDICT: **GREEN — begin P1.1. 0 Critical / 0 Important.**

The plan is a faithful, correctly-sequenced composition of the two R0-GREEN specs. The single most important decision — **the P1.4(b) reveal-designation RULE is RATIFIED** (it reveals the exactly-right field in every affected step, can never designate a public/unintended field, and is census-enforceable). **The `assert_no_plaintext` hygiene control is SOUND and hole-free** — the parameterization is allowlist-bounded, applies to the filled-form checkpoint only, and leaves the modal + pane checkpoints unconditionally strict, all verified at the cited source lines. **The two-single-regen sequencing graph is deterministic + complete** — nothing re-pins twice, nothing is missed, and no half-updated corpus can reach CI. All 12 spec-R0 minors are folded; the two-tag release sequence, GOTCHAS header, and STOP ledger are complete. Six Minor precision items follow — all non-blocking, all self-correcting under the plan's own fail-closed gates.

---

## RULING 1 — THE OPEN QUESTION: the P1.4(b) reveal-designation RULE → **RATIFIED (rule-derived, not hand-picked)**

**The rule is correct, superior to a hand-picked step list, and census-enforceable. Ratify it.**

The rule ("every `capture: true` step with ≥1 secret-classified drive reveals its last-driven secret field's eye") is anchored to the manifest's existing classifier `Drive::secret_value()` (`mnemonic-gui/tests/tutorial/mod.rs:169-177`), which returns `Some` ONLY for `TypeSlot{subkey}` where `slot_subkey_is_secret(subkey)` and `TypeComposite{node}` where `node_type_is_argv_secret(node)`, and `None` otherwise. I traced every step in `manifest.rs`; the rule-designated set is **exactly four steps**:

| step | site | last-driven secret field revealed | correct field to show the reader? |
|---|---|---|---|
| `tut-j1-01-bundle-single-sig` (:164, capture:true) | #2 slot | `TypeSlot @0 Phrase S0` (:170) | YES — the only seed typed |
| `tut-j2-02-convert-fingerprint` (:183, capture:true) | #3 composite | `TypeComposite --from phrase S0` (:129-133) | YES — the seed typed |
| `tut-j2-03-convert-xpub` (:198, capture:true) | #3 composite | `TypeComposite --from phrase S0` | YES — the seed typed |
| `tut-j2-07-bundle-all-seeds` (:231, capture:true) | #2 slot | `TypeSlot @2 Phrase S2` (last of S0/S1/S2, :245-249) | YES within the single-revealed-field invariant |

**Why the rule reveals the RIGHT field and never an unintended one:**
- `secret_value()` can never return a public field. `TypeMd1Chain` (the restore steps' `--md1` card — public descriptor material) returns `None` → **all six restore steps carry zero reveal markers**, so their `-form` shots stay fully masked. That is exactly right: a restore form types no seed, so there is nothing to reveal. The rule structurally cannot designate the very fields the batch is switching to `(none)`.
- For single-secret steps (j1-01, j2-02, j2-03) "last-driven" is trivially the only secret field = the field the reader types.
- For the one multi-secret step (j2-07) the **single-revealed-field invariant (ruling 1, HARD hygiene) physically caps reveal at one row**; "last-driven" (S2) is a principled, deterministic tiebreak, and the two masked rows beside it are, as the plan says, themselves the teaching image. Revealing all three is *impossible* by construction — the plan is honest about this.

**Why rule-derived beats a hand-picked list:** the same taxonomies (`SECRET_SLOT_SUBKEYS`/`SECRET_NODE_TYPES_ARGV`) drive `secret_value()`, the allowlist checker (`secret_allowlist_violations()`, `mod.rs:384`), AND (via ruling 8's ⊆-agreement) the widget-mask classifiers. So a revealable field is provably allowlist-bound. A hand-picked list would drift silently as the corpus evolves; the bidirectional census ("no capture-true secret-bearing step lacks the marker; no secretless step carries one") is self-maintaining and fail-closed.

**One precision the fold must carry (Minor #1 + #2):** the plan's illustrative parenthetical "*J2 device converts are capture:false*" is **wrong** — that is true only of the dev1/dev2 *feed* converts (`:212-219`, `feed_step`, capture:false); the **device-0** converts `tut-j2-02`/`tut-j2-03` are `capture:true` and secret-bearing and WILL be designated. The RULE and census still resolve them correctly (the phase report enumerates the real set), so this is prose, not mechanism — but the fold should (a) enumerate the true 4-step set and (b) note that j2-02/03 exercise the **site-#3 composite eye** (`widget.rs:606`), so the tutorial reveal machinery must drive the composite eye via AccessKit, not only slot/flag eyes.

---

## RULING 2 — the `assert_no_plaintext` hygiene control → **SOUND, no hole**

Verified against `mnemonic-gui/tests/gui_tutorial_snapshots.rs` at the exact cited lines. The three checkpoints are structurally distinct:
- **filled-form** (`:436` full-value + `:438` word-probe) — iterates ALL of `step.drives … Drive::secret_value`.
- **populated-pane** (`:509` word-probe) — inside `if step.is_secret()`.
- **confirm-modal** (`:579` word-probe).

The plan parameterizes **only** the filled-form checkpoint (`:436-438`); pane (`:509`) and modal (`:576-579`) stay untouched. Adjudicating the three sub-checks the brief demanded:

- **(a) Is `:436` where reveal legitimately shows a phrase?** YES. `:436-438` gate the `-form` shot, which is exactly where the latch is active; the Run click (trigger 1) auto-hides reveal before the `-modal`/`-run` captures, so the pane/modal shots are masked by construction. Confirmed by the plan's P1.4(b) latch sequencing (persists through wheel-scroll `-form2`, cleared by Run).

- **(b) Is the parameterization `mask-set ⊆ {S0,S1,S2}`-bounded, NOT a blanket loosening?** YES. Ruling 8 + P1.4(c) keep `SECRET_ALLOWLIST` = `[S0,S1,S2]` (`mod.rs:51`, verified) and add the negative "a reveal-marked step whose value ∉ allowlist ⇒ RED." So the ONLY value that can be permitted-as-revealed is an allowlisted demo phrase; a non-demo secret revealed anywhere still REDs. The permit is per-drive: only the *revealed* drive's (allowlisted) value is permitted at `:436`; **every OTHER secret's full value AND word-probe stay strict.** I confirmed this is collision-safe on the real corpus: S0/S1/S2 have **distinct first words** ("abandon"/"legal"/"letter", `mod.rs:39-45`) and S2 (the revealed last slot in j2-07) contains neither "abandon" nor "legal" — so the strict word-probes for the two masked seeds do not false-RED against the revealed text. (Recorded as Minor #3: the fold should note this word-disjointness as the reason the multi-secret step is safe; a future collision would RED, not leak — fail-closed either way.)

- **(c) Is modal/pane strictness genuinely untouched?** YES. The plan parameterizes neither `:509` nor `:579`; both keep the unconditional word-probe. A non-demo secret revealed in those surfaces still REDs.

**No hole.** This is the load-bearing control and it is correctly bounded, correctly scoped to the one checkpoint where reveal legitimately shows a phrase, and correctly fail-closed everywhere else.

---

## RULING 3 — the sequencing / no-half-updated-corpus graph → **DETERMINISTIC + COMPLETE**

I traced the regen dependency graph for double-pins and missed pins:

**GUI-leg regenerated artifacts (each exactly once):**
- **61-form gallery** (`gui_form_snapshots`) — regenerated ONCE in **P1.2** (R-A). P1.3 asserts zero movement (restore append is virgin-form-inert: `opts[0]=bip44` and `default_value=None` unchanged → closed combo still shows `bip44`; verified `schema/mnemonic.rs:537-546`). P1.4 does not touch it. **No double-pin.**
- **`gui_render_emit.rs` exact-ASCII pins** — re-pinned ONCE in P1.2 (secret-row forms). Restore has **no** emit pin (grep-confirmed zero hits, restore-R0 item 5), so P1.3 touches it only via the new TDD file + schema. **No double-pin.**
- **Tutorial corpus** (50 PNG) — regenerated ONCE in **P1.4** (R-B), after BOTH code changes AND the manifest carries both `(none)` switches and reveal drives. P1.2/P1.3 leave it stale-but-inert *locally* (both snapshot suites are env-gated: `GUI_SNAPSHOTS` / `GUI_TUTORIAL_SNAPSHOTS`, skipped by plain `cargo test`), and **the PR opens only after P1.4** so CI's `tutorial-snapshots` job (`build.yml:149`, `GUI_TUTORIAL_SNAPSHOTS=1`) never sees a half-updated corpus. **No half-updated corpus reaches CI.**

**Toolkit-leg (P2.1, from the FRESH TAG CLONE) — each exactly once:** 28 `.gui` re-pins (`mnemonic-restore.gui` correctly carries BOTH deltas: the `:4` `--template … ,(none)` append AND the `:14` `--passphrase` eye marker — both land in the ONE regen), 28 gallery PNG copies, all-50 tutorial-figure copies (git shows the moved subset), inventory regen (documentary/un-gated, `expected_gui_schema_inventory.json`), transcripts ZERO delta. Every mover has a home; every phase census is fail-closed to STOP-5.

**Why reveal-before-restore is load-bearing:** the reveal is the phase that moves the gallery/emit surfaces; restore is one of the 28 masked-on-load forms, so its gallery PNG moves *in P1.2 (by the eye)* and its `.gui` moves *in P2.1 (both deltas)* — NOT in P1.3. The plan correctly restates the restore spec's standalone "byte-identical" claims as "zero **additional** movement from the `(none)` append," reconciling the spec-in-isolation view with batch reality. Deterministic and complete. **No re-pin twice, nothing missed.**

**Transcript invariant** holds batch-wide: restore `(none)` is byte-identical (restore-R0 ruling 7, re-confirmed at `restore.rs:3068-3076` / `error.rs:618` / dispatch `:314`→`:349`); reveal is display-only and its AccessKit-Click marker changes no argv (auto-hidden before Run). 98 `transcripts/tutorial/*` + all run bytes IDENTICAL; STOP-1 guards any delta.

---

## The other Fable findings (#2–#5) — all handled correctly

- **#2 decoy `--template wsh-sortedmulti` sites fenced.** VERIFIED both are genuine, non-restore selections that must NOT switch to `(none)`: `convert_drives!` macro body (`manifest.rs:129-133`, the `--template` at `:131`) and `tut-j2-07-bundle-all-seeds` (`:236`). These are *untouched* by the plan, so their transcripts are byte-identical by construction — the fencing is correct regardless of whether convert/bundle "consume" the value. Restore's route-around is distinguishable precisely because restore's md1-mode `--template` is INERT (ruling 7) whereas bundle/convert `--template` selects the policy family.
- **#3 restore "zero *additional* movement."** Correct and honest — see Ruling 3. Restore's PNG moves by the reveal (1 of 28), not by `(none)`.
- **#4 stale `45-export-wallet.md` prose.** VERIFIED at `:55`: "*the 11-vs-10 asymmetry is intended and export-wallet-scoped*" — falsified the moment restore also carries the sentinel. The plan's P2.2 required edit is correct (plan cites `:48-56`; the claim spans `:48-55`).
- **#5 pointer-no-latch ruling + bounded fallback.** SOUND. The plan's ruling 2 (pointer tap does NOT latch; latch arms only on keyboard/AccessKit-Click; discriminate via the frame's pointer-release vs an AccessKit action) discharges reveal-R0 M-1 exactly. The bounded fallback is sound *because its floor is the R0-accepted §4.5-bounded latch posture*: if egui cannot discriminate tap from AT-click, "tap latches" degrades to the already-accepted keyboard/AT latch (bounded by the 4 auto-hide triggers), never to something worse — and it is recorded (STOP-9 = recorded-downgrade, not a user STOP; tracked in `gui-secret-reveal-latch-timeout`).

## Ripple counts / census exactness (finding #5) — census-derived + fail-closed

`28` is the live `grep -rl '<masked>' docs/manual-gui/transcripts/gui/` cardinality (re-verified = 28); `61` `.gui` + `61` PNG + `50` tutorial shots + `98` transcripts all re-verified. The plan ties "28" to "the `<masked>`-on-load forms" (a derived set, export-wallet excluded as slot-only-`0 rows`) with STOP-5 on any 29th mover or unmoved expected-mover, and asserts `manifest-stems.txt` unchanged + count-50 constant. This is set-derived and fail-closed, not a bare magic number (Minor #4: prefer expressing the assertion as `moved-set == grep('<masked>')-set` rather than `count == 28`, so STOP-5 self-checks).

## All 12 spec-R0 minors folded (finding #6) — spot-checked GREEN

Restore m1 (three `TEMPLATES` consumers `:309`/`:866`/`:1271` enumerated — verified those are the exact four `Dropdown(TEMPLATES)` sites, restore `:539` re-pointed) ✓ · m2 (emit under-list, immaterial) ✓ · m3 (both `:314` check + `:349` call cited) ✓ · m4 (`--format` md1 clarification into `4d-restore.md:84-92`) ✓ · m5 (passphrase `:667`/`:677` — spec-internal, not plan-carried; Minor #5) ✓ · m6 (file `restore-md1-template-mutex-projection`) ✓. Reveal M-1 (tap-no-latch + file `gui-secret-reveal-latch-timeout`) ✓ · M-2 (`invocation.rs:152`/`:524`, verified — not `secrets.rs`) ✓ · M-3 (#3/#4/#5 back plain swept `String`s) ✓ · M-4 (one-frame defocus race in prose, no modal) ✓ · M-5 (`schema_mirror` firing wording; pin stays v0.75.0 → GREEN) ✓ · M-6 (`ui.horizontal` + stable per-field `egui::Id` via `Response.id`; call sites `app_window.rs:826`/`widget.rs:116`/`:159`) ✓.

## Two-tag release + cadence + GOTCHAS + STOP ledger (finding #7) — complete

GUI `v0.57.0` tagged (P1.5) with tag-run check-runs (`snapshots` + `tutorial-snapshots`) verified `success` via explicit remote URL BEFORE the toolkit pin bump (P2.1) → `manual-gui-v1.3.0` (P2.3). Per-phase opus review cadence table present; reviews persist verbatim before folds; folds re-enter the loop. GOTCHAS header carries every gui_example carry-forward: degenerate-agent→poll-git, no bg watchers, `make html` before `make lint`, fresh-tag-clone census, opus reviews, and trailers matching the LIVE harness (`Claude Opus 4.8` + this session URL — correctly NOT the stale Fable trailer from MEMORY.md). 9-item STOP ledger complete (STOP-1 transcript, STOP-2 no-`src`/clap, STOP-4 32-MiB budget = verified `BUDGET_HARD_MIB=32.0` at `gui_tutorial_snapshots.rs:73`, STOP-5 census, STOP-8 no allowlist-widen, STOP-9 recorded-downgrade).

---

## FINDINGS BY SEVERITY

### Critical — NONE.
### Important — NONE.

### Minor (fold before P1.1 dispatch; none blocks the START)
- **Minor #1** — P1.4(b)'s illustrative note "*J2 device converts are capture:false*" is factually wrong for `tut-j2-02-convert-fingerprint` (`manifest.rs:183`, capture:true) and `tut-j2-03-convert-xpub` (`:198`, capture:true), which ARE secret-bearing and rule-designated. Enumerate the true 4-step reveal set {j1-01, j2-02, j2-03, j2-07} and note j2-02/03 drive the **site-#3 composite eye** (`widget.rs:606`), so the tutorial reveal path must actuate the composite eye, not only slot/flag eyes. Self-correcting via census, but fix to avoid P1.4 friction.
- **Minor #2** — pin the reveal-marker census predicate to `Drive::secret_value().is_some()` (`mod.rs:169`), NOT `Step::is_secret()` (`mod.rs:267`, which ORs `secret_modal`). They coincide on today's corpus; a future `secret_modal`-only step would false-RED the "no secret-bearing step lacks the marker" census with no field to reveal. State the predicate explicitly.
- **Minor #3** — record in P1.4(c) that the per-drive checkpoint parameterization is collision-safe because S0/S1/S2 have distinct first words ("abandon"/"legal"/"letter") and the revealed j2-07 phrase (S2) contains neither other seed's first word. A future word collision would RED (fail-closed), not leak — but the rationale should be explicit in the test.
- **Minor #4** — express the P1.2/P2.1 "28" census as the derived set (`moved == grep('<masked>')`, export-wallet excluded) rather than a literal count, so STOP-5 self-checks against corpus drift.
- **Minor #5** — restore-R0 m5 (passphrase citation drift `:672`/`:683`→`:667`/`:677`) is a spec-internal fix not carried into the plan; non-load-bearing (the plan cites no passphrase lines). Acknowledge for completeness.
- **Minor #6 (watch-item)** — site #3 (composite, `widget.rs:606`) is the most likely source of a 29th gallery mover in P1.2: if any gallery form's `--from` composite defaults to a secret node with empty value on load, the eye (predicate ∧ `is_secret_node`) could appear and move that PNG. Reveal-R0 verified the on-load count is 28 (composite renders `<empty>`, not masked, on load); STOP-5 catches any surprise. Have the P1.2 phase explicitly confirm no composite form renders an on-load eye rather than discovering it via STOP-5.

---

## Bottom line

Every load-bearing citation checks out against `mnemonic-toolkit @ 2ed3d369` and `mnemonic-gui @ 40156b0`. The reveal-designation RULE is ratified, the `assert_no_plaintext` hygiene control is hole-free and allowlist-bounded, the two-single-regen sequencing graph is deterministic and complete, all 12 spec-R0 minors are folded, and the two-tag release sequence is correct. No Critical or Important finding blocks the start of implementation. **GATE: GREEN — begin P1.1.** Fold the six Minor precision items (they are plan-content, not spec re-review triggers). Per the reviewer-loop discipline, re-dispatch this reviewer after the fold (folds can introduce drift).

---

## CONVERGENCE ADDENDUM (round 1 fold-check) — the 6-minor fold at `c00049b7`

- **Reviewer:** same opus architect, SCOPED convergence check (re-dispatch after fold, per reviewer-loop discipline). READ-ONLY, adversarial.
- **Fold under review:** `git diff 2ed3d369..c00049b7` — the plan-doc edit that folds R0 round-1's six Minor items.
- **Refs at review:** toolkit `master @ c00049b7` (fold commit; parent `2ed3d369` = the R0-reviewed plan commit). mnemonic-gui sibling `@ 40156b0` (= the plan's cited ref; `git rev-parse` confirmed). m1 source facts re-traced against live `mnemonic-gui/tests/tutorial/{manifest.rs,mod.rs}` at that ref; m4 census re-run against the toolkit working tree.

### VERDICT: **GREEN — begin P1.1. 0 Critical / 0 Important.**

The fold is a faithful, strictly-localized application of the six Minor precision items. No drift, no ruling contradiction, no cross-phase inconsistency. The round-1 GREEN verdict stands unchanged. Implementation may begin at P1.1.

### Fold-diff localization (no-drift, mechanical)

`git diff --stat 2ed3d369..c00049b7` = **1 file changed, 3 insertions(+), 3 deletions(-)** — three single-line replacements, exactly the three edited spots the fold was scoped to touch and nothing else:
- **P1.4(b)** (`~L115`) — folds **m1 + m2 + m3** (one line rewritten).
- **P1.2 R-A** (`~L102`) — folds **m4 + m6** (one line rewritten).
- **P1.3 batch-inertness** (`~L109`) — folds **m5** (one clause appended).

No other line in the plan-doc moved; no other file touched. The dependency graph, RULINGS block, GOTCHAS, REVIEW CADENCE, FOLLOWUPs LEDGER, two-tag sequence, and 9-item STOP ledger are byte-unchanged.

### The six folds — each confirmed

- **m1 (the one that mattered) — CORRECT, source-verified.** P1.4(b) now states the reveal-marker set as **exactly these FOUR** `capture:true` secret-value steps: `tut-j1-01-bundle-single-sig` (slot @0 = S0), `tut-j2-02-convert-fingerprint` + `tut-j2-03-convert-xpub` (both explicitly `capture:true`, driving the **site-#3 composite `--from phrase=` eye**, S0), `tut-j2-07-bundle-all-seeds` (last slot = S2). The old, wrong parenthetical "*J2 device converts are capture:false*" is **removed**. Re-traced against `manifest.rs @ 40156b0`: `tut-j2-02` (`:183`) and `tut-j2-03` (`:198`) are `capture: true` and drive `convert_drives!` = `Drive::TypeComposite{--from, node:"phrase", value:S0}` (`:129`) → `secret_value()==Some(S0)` via the argv-secret-node arm. Only the dev1/dev2 **`feed_step`** converts (`:216-223`) are `capture:false` — the exact source of the old conflation. The FOUR-step set matches Ruling 1's table (round 1, §RULING 1). The restore-→-None claim is exact: `restore_drives!` (`:137`) emits `SelectDropdown{--template}` + `TypeMd1Chain{--md1}`; `TypeMd1Chain` is **absent** from `secret_value()`'s match arms (`mod.rs:169-177`) — it appears only in `flag()` (`:207`) — so all six restore steps yield `None` → **zero markers**.
- **m2 — CORRECT.** The census predicate is pinned to **`Drive::secret_value().is_some()`** (`mod.rs:169-177`), explicitly NOT `Step::is_secret()`. Verified `is_secret()` (`mod.rs:267-268`) = `self.secret_modal || self.drives.iter().any(|d| d.secret_value().is_some())` — it ORs `secret_modal` and would over-select a modal-only step with no revealable field. The fold names the correct seam and the correct rationale.
- **m3 — CORRECT.** Word-disjointness collision-safety recorded in P1.4(b): S0/S1/S2 first words `abandon`/`legal`/`letter`, and S2 contains neither other seed's first word. Verified at `mod.rs`: `S0="abandon abandon … about"`, `S1="legal winner … yellow"`, `S2="letter advice cage absurd amount doctor acoustic avoid letter advice cage above"` (contains no "abandon"/"legal"). `SECRET_ALLOWLIST = [S0,S1,S2]` confirmed. Ties to Ruling 8; no contradiction.
- **m4 — CORRECT.** "28" is now expressed as the census output `moved-PNG-set == grep -rl '<masked>' docs/manual-gui/transcripts/gui/` (`= 28 at 2ed3d369`), fail-closed via STOP-5 on ANY deviation, not a bare literal. Re-ran the grep against the toolkit working tree: **28** masked-on-load `.gui` of 61 total — matches. STOP-5 "any 29th mover / any unmoved expected-mover" wording preserved.
- **m5 — CORRECT.** P1.3 now records restore-R0 m5 (the `--passphrase` citation drift, actual `:667`/`:677`) as **spec-internal precision, NOT plan-carried; no plan action** — matching round-1 Minor #5 (the plan cites no passphrase lines, so nothing to carry). Line refs `:667`/`:677` match the R0's corrected target.
- **m6 — CORRECT.** P1.2 R-A now carries the explicit action: **P1.2 must confirm no composite form (`widget.rs:606`) renders an on-load eye** (the likely 29th-mover watch; verify each composite form's on-load node is non-secret). This is the round-1 Minor #6 watch-item promoted from "discover via STOP-5" to an explicit phase confirmation.

### No-drift statement

The fold is confined to the three intended single-line replacements (diff-stat 3+/3-, one file). Each fold matches both its R0 Minor and live source truth. No fold contradicts a BINDING RULING (m1↔Ruling 1, m3↔Ruling 8, m4/m6↔STOP-5, m2 pins the census seam that Ruling 1/8's ⊆-agreement already relies on) and no fold contradicts another phase (P1.2 still regens the gallery exactly once; P1.3 inertness assertions unchanged; P1.4 reveal designation now correctly enumerated; Ruling 7 transcript-invariant and the STOP ledger untouched). The round-1 GREEN verdict is undisturbed. **GATE: GREEN — implementation may begin at P1.1.**
