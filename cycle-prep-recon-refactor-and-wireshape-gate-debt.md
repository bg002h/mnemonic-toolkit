# cycle-prep recon — 2026-06-06 — synthesize-dedup + lib-surface-promotion + ms-kofn-wireshape + gui-decode-verify-schema

**Origin/master SHA at recon time:** toolkit `70c88fd` · GUI `4b83a9f` (mnemonic-gui-v0.27.0)
**Local branch:** `master` · **Sync state:** `up-to-date (0/0)` · **Untracked:** recon/survey scratch + `.claude/` (none load-bearing).

Slug(s) verified: 4. **Two of the four are NOT actionable cycles as picked:** `gui-decode-address-verify-message-schema-mirror` is **ALREADY DONE** (shipped mnemonic-gui v0.21.0) — just flip the stale toolkit FOLLOWUP; `ms-kofn-json-wire-shape-ungated` is a **standing-posture tracker that fires no gate by design** — no code to write unless you build a general wire-shape gate. The two real refactors (`synthesize-dedup`, `lib-surface-promotion`) both have corrected scope below.

---

## Per-slug verification

### `synthesize-descriptor-deduplicate-with-unified`  — REAL refactor; FOLLOWUP undercounts + mis-signatures
- **WHAT:** Fold the shared card-emission logic of `synthesize_descriptor` + `synthesize_unified` into one helper.
- **Citations:**
  - `synthesize.rs:200-275` (`synthesize_descriptor`) — **DRIFTED.** fn opens `:229` (body ~`:229-`). 
  - `synthesize.rs:709-774` (`synthesize_unified`) — **DRIFTED.** fn opens `:745`.
  - "two call sites (`cmd/bundle.rs:1259` + `cmd/verify_bundle.rs:673`)" — **STRUCTURALLY-WRONG.** Real sites: `synthesize_descriptor` ×5 (`bundle.rs:1563`, `:1641`, `:1882`; `import_wallet.rs:1398`; `verify_bundle.rs:1002`); `synthesize_unified` ×4 (`bundle.rs:399`; `verify_bundle.rs:374`, `:464`, `:568`). **~9 call sites across 3 files, NOT 2** — and the cited line numbers don't exist as call sites.
  - **SIGNATURE DIVERGENCE (the real scoping fact, not in the FOLLOWUP):** `synthesize_descriptor(descriptor: &Descriptor, cosigners: &[CosignerKeyInfo], privacy, lang)` vs `synthesize_unified(slots: &[ResolvedSlot], template: CliTemplate, threshold: u8, network: CliNetwork, privacy, lang)`. They take DIFFERENT inputs — one is handed a pre-built `Descriptor`+`CosignerKeyInfo`s; the other derives the descriptor from `template`+`slots`+`threshold`+`network`. The FOLLOWUP's proposed `emit_unified_cards(descriptor, cosigners, privacy)` signature matches only `synthesize_descriptor`'s inputs. So this is NOT a whole-function merge — it's an **extract-the-shared-BACK-HALF** refactor (the ms1/mk1/md1 card-emission from a resolved descriptor + cosigner list), exactly the species of the just-shipped `emit_payload` dedup. Each fn keeps its front-half (input→descriptor+cosigners), then both call the shared emitter.
- **Action for brainstorm spec:** Correct the call-site list (5+4, not 2) + line numbers; frame as a back-half extraction (`fn emit_bundle_cards(descriptor, cosigners, privacy, lang) -> Result<Bundle>`), NOT a front-to-back merge. Brainstorm MUST diff the two bodies to confirm the back-halves are byte-shareable (the `synthesize.rs:255-257` ↔ `:710-723` mirror-comment is the anchor) and that `synthesize_unified`'s descriptor-derivation front-half feeds the same `(descriptor, cosigners)` the helper expects. Cite SHA `70c88fd`.

### `library-error-and-language-surface-promotion`  — REAL crate-shape refactor; ACCURATE; **80-file blast radius**
- **WHAT:** Move `error`/`language`/`friendly` from `main.rs`-private to `lib.rs`-public; re-route all `crate::{error,language,friendly}::*` → `mnemonic_toolkit::…`; delete the `FinalWordLanguage`/`FinalWordError` library-local mirror types.
- **Citations:**
  - `main.rs:14` `mod error;`, `:16` `mod friendly;`, `:18` `mod language;` — **ACCURATE** (all three are main.rs-private; none in lib.rs).
  - `FinalWordError`/`FinalWordLanguage` library-local mirrors — **ACCURATE** (referenced in the `lib.rs:15-17` module doc; the P1 pivot types to delete).
  - **Blast radius (new fact):** **80 files** under `src/` import `crate::{error,language,friendly}::*`. A move to lib.rs requires re-routing imports across all 80 (or a `pub use` re-export shim in main.rs to soften it). This is a WIDE mechanical refactor — low per-edit risk, very large diff.
- **Action for brainstorm spec:** Confirm whether a `pub use mnemonic_toolkit::{error,language,friendly};` shim in main.rs avoids touching all 80 files (likely — then the diff is small: move 3 files + add re-exports + delete 2 mirror types + reroute only `final_word.rs`). The "reroute 80 files" framing is the maximal interpretation; a re-export shim is the minimal one. Brainstorm must pick. Tier v1+, NO user-facing change, NO lockstep. Cite SHA `70c88fd`.

### `ms-kofn-json-wire-shape-ungated`  — NOT A CYCLE (standing-posture tracker, fires no gate by design)
- **WHAT (from FOLLOWUPS.md):** Documents that `ms-shares`/`ms split|combine|inspect` `--json` wire-shapes + the `combine --to` value-enum are not `schema_mirror`-gated.
- **Citations:** ACCURATE — but the entry's own Status says *"fires no automated gate by design"* and *"standing-posture / paired-PR tracking."* It is a **record**, not a work item. The CLAUDE.md gate scope ("flag-NAME parity, NOT JSON wire-shape") is the deliberate posture; the actionable generalization is the SEPARATE FOLLOWUP `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` option (b) — building a per-subcommand `--json` wire-shape drift gate, a v0.30+ design item spanning ALL toolkit `--json` surfaces, not just K-of-N.
- **Action for brainstorm spec:** **None as-written** — there is no code to change for THIS slug. If wire-shape drift protection is the actual goal, scope `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` as its own (larger) design cycle; do not "cycle" this tracker.

### `gui-decode-address-verify-message-schema-mirror`  — ALREADY DONE (stale-open FOLLOWUP)
- **WHAT:** Add GUI `SubcommandSchema`s for `decode-address` + `verify-message` (v0.36.0 surface).
- **Citations:** **STRUCTURALLY STALE — the work shipped 2026-05-23 in mnemonic-gui v0.21.0** (`2a2111d` "schema-mirror lockstep for toolkit decode-address + verify-message"). The GUI schema (`mnemonic-gui/src/schema/mnemonic.rs`) ALREADY carries both: `decode-address` `SubcommandSchema` (`:3504`, `DECODE_ADDRESS_FLAGS` + positionals) and `verify-message` (`:3513`, `VERIFY_MESSAGE_FLAGS`) + the `verify-message --format` value-enum (`:30`). GUI pin is v0.46.2 (≫ v0.36.0). The toolkit FOLLOWUP was simply never flipped (the [[feedback_per_phase_agents_forget_followup_status_flip]] class).
- **Action for brainstorm spec:** **No cycle.** Flip the toolkit FOLLOWUP `gui-decode-address-verify-message-schema-mirror` → `resolved (mnemonic-gui-v0.21.0, 2a2111d)` in a docs commit.

---

## Cross-cutting observations
1. **2 of 4 picks are non-cycles:** slug 4 done (flip it), slug 3 a by-design no-op tracker. cycle-prep earned its keep — neither would have produced shippable code.
2. **Slug 1's FOLLOWUP is materially wrong** on call-site count (2 vs ~9) AND mis-frames the refactor (whole-fn merge vs back-half extraction — the signatures differ). Same count-ambiguity class as the last two dedup recons ([[feedback_r0_must_read_source_off_by_n]]). The corrected framing is the `emit_payload`-style back-half lift.
3. **Slug 2's cost hinges on one decision** (re-export shim vs 80-file reroute) — recon flags it so the brainstorm picks deliberately; the shim makes it small, the reroute makes it a mega-diff.
4. No incidental cross-pin staleness (GUI pin current at v0.46.2 from the v0.27.0 cycle).

---

## Recommended brainstorm-session scope

**Do first (no cycle, ~5 min):** flip the stale `gui-decode-address-verify-message-schema-mirror` FOLLOWUP → resolved (slug 4). Drop slug 3 (no actionable work as-written).

**Cycle candidate (the one clean debt-paydown of the picked set): `synthesize-descriptor-deduplicate-with-unified`.** **SemVer PATCH** (pure refactor, no user-visible change — guard with the existing synthesize/bundle/verify-bundle suites + the per-slot ms1 cells). **Size: moderate** — extract a shared back-half card-emitter; ~9 call sites unchanged (they call the two wrappers, which now delegate). Net-negative-ish LOC. **Lockstep: NONE.** Same species + R0 discipline as the just-shipped `emit_payload` + origin-extraction dedups. **R0 MUST** verify the two back-halves are genuinely byte-shareable + correct the call-site/line citations.

**Separate larger cycle (defer or do alone): `library-error-and-language-surface-promotion`.** Crate-shape, no user-facing change, **decision-gated on the re-export-shim vs 80-file-reroute** scoping. If the shim works it's small + clean; if not it's a mega-diff better done in isolation. Recommend the brainstorm settle the shim question first, then size. Do NOT bundle with slug 1 (different blast radius + risk profile).

**Ordering:** slug 4 flip (now) → `synthesize-dedup` cycle → (optionally) `lib-surface-promotion` as its own cycle. Each real cycle gets the mandatory R0 gate.
