# cycle-prep recon — 2026-06-08 — toolkit-trmultia-nums-internal-key + friendly-mapper-unit-test-gaps + canonicity-drift-gate-floor-too-lenient + bundle-emit-bypasses-chunk-mk1-alias

**Origin/master SHA at recon time:** `8665d91`
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** recon scaffolding only.

Slug(s) verified: 4. **Headline: these are NOT one cycle.** #1 is a substantial wire-change bug; #2+#4 are tiny toolkit test/cleanup; **#3 lives in a DIFFERENT REPO (`mnemonic-gui`), not the toolkit.** Citations all verified accurate (minor drift); one structural scope finding (#3 cross-repo).

---

## Per-slug verification

### `toolkit-trmultia-nums-internal-key`  (TOOLKIT code — the real bug)
- **WHAT:** `bundle` tr multisig (`tr-multi-a`/`tr-sortedmulti-a`) emits a placeholder cosigner-internal-key (`is_nums:false, key_index:0`) instead of NUMS → the md1 is non-standard + non-round-trippable; blocks `restore-multisig-taproot-reconstruction`.
- **Citations:**
  - `template.rs:194-215` `TrMultiA|TrSortedMultiA` arm → `Body::Tr { is_nums: false, key_index: 0 }` — **ACCURATE** (arm at `:194`; `is_nums: false`/`key_index: 0` at `:208-209`, with an in-code comment *naming this FOLLOWUP* at `:203-208`).
  - `template.rs:446` `assert!(!is_nums, "TrMultiA wrapper currently uses key_index=0 (real key), not NUMS sentinel")` — **ACCURATE** (exact, still at `:446`).
  - `synthesize.rs:399` bundle emit consumes `wrapper_node` verbatim — **DRIFTED-by-1**: the multisig emit path is `synthesize.rs:398` (`wrapper_node(threshold, cosigner_count)`); also call sites at `:119/:564/:789`.
  - `parse_descriptor.rs::substitute_nums_sentinel` (only `is_nums:true` setter, `--descriptor` intake) — **ACCURATE** (`:275`).
- **Action for brainstorm spec:** the meaty one. **Decide + implement** emitting NUMS (`is_nums:true`) for `wrapper_node` tr-multisig templates: flip `template.rs:208` + the `:446` assert, and update **md1 wire fixtures** (this CHANGES the emitted md1 for tr multisig). Confirm it unblocks `restore-multisig-taproot-reconstruction` (the dependent). Watch: `md_codec::to_miniscript` can't build `SortedMultiA` — verify the NUMS form is round-trippable before claiming the unblock. Cite SHA `8665d91`. **Own cycle (design decision + wire change + fixtures).**

### `friendly-mapper-unit-test-gaps`  (TOOLKIT test-only)
- **WHAT:** `friendly.rs` per-error-mapper unit tests thin (FOLLOWUP: covered 3, ~67 untested).
- **Citations:**
  - `friendly.rs::tests` — **ACCURATE** (5 mappers: `friendly_bip39:10`, `friendly_bitcoin:34`, `friendly_ms_codec:42`, `friendly_mk_codec:133`, `friendly_md_codec:185`).
  - "covers 3 / `#[test]` count" — **DRIFTED**: the v0.1-era "covers 3" is stale — `friendly.rs` now has **12 `#[test]`s** (more added since 2026-05-05). The exact current arm-coverage must be re-counted in the brainstorm (the "~3 of 70" the user quoted is the as-filed figure, now understated-coverage).
- **Action for brainstorm spec:** re-enumerate current `match`-arm coverage vs the 12 existing tests, then add unit tests for the still-uncovered arms (table-driven). Pure test addition — no wire/CLI/behavior change. Cite SHA `8665d91`. **Bundle with #4 (small toolkit test+cleanup cycle).**

### `canonicity-drift-gate-floor-too-lenient`  ⚠️ **DIFFERENT REPO — `mnemonic-gui`, NOT toolkit**
- **WHAT:** the GUI's drift gate floor (`classified >= FIXTURES.len()/2`) is too lenient — broad toolkit-parser regression passes silently.
- **Citations:**
  - `mnemonic-gui/tests/canonicity_drift.rs:138` floor — **DRIFTED-by-6 + CROSS-REPO**: the file is in **`../mnemonic-gui`** (present in workspace); the floor `classified >= FIXTURES.len() / 2` is now at **`:132`** (`FIXTURES` at `:60`). The Companion line confirms a `bg002h/mnemonic-gui` FOLLOWUP.
- **Action for brainstorm spec:** this is a **mnemonic-gui cycle**, not a toolkit one — it edits the GUI repo's test, follows GUI conventions (`+1.94.0` toolchain, GUI R0/lockstep), and lands as a GUI commit. Design: replace the 50% floor with a **per-fixture classified-expectation table** (the FOLLOWUP's own "right answer"; a blind floor-bump to `len()-4` is brittle). Cite GUI source SHA. **Separate GUI cycle — do NOT bundle with toolkit work.**

### `bundle-emit-bypasses-chunk-mk1-alias`  (TOOLKIT — trivial)
- **WHAT:** `bundle` mk1 emit calls `chunk_5char` directly; the reserved `chunk_mk1` alias is dead.
- **Citations:**
  - `format.rs::chunk_mk1` — **ACCURATE**: `chunk_mk1` at `:33` = `chunk_5char` (alias), comment `:30` "Reserved: mk1 currently uses `chunk_5char` directly; mk-specific helper retained".
  - `bundle.rs::emit` calls `chunk_5char` for mk1 — **ACCURATE (with a count refinement)**: there are **TWO** mk1 sites — `MkField::Single` (`bundle.rs:~962`) and `MkField::Multi` per-cosigner (`:~974`), both `chunk_5char(s)`. (The ms1 site `:951` stays `chunk_5char`.) Verify the enclosing fn name in the brainstorm (cited `emit`; confirm vs a possible rename).
- **Action for brainstorm spec:** switch **both** mk1 call sites `chunk_5char(s)` → `chunk_mk1(s)` so the future mk-codec grouping-helper swap is single-edit. `chunk_mk1` ≡ `chunk_5char` → **byte-identical output → no-bump**. Cite SHA `8665d91`. **Bundle with #2.**

---

## Cross-cutting observations
1. **#3 is cross-repo (`mnemonic-gui`).** The user grouped it with 3 toolkit items, but it cannot share a toolkit cycle — different repo, toolchain, R0/lockstep, version namespace. Surface this before any plan-doc.
2. **#1 is the only substantive code change** — and it's a **wire-shape change** (md1 for tr multisig) + a design decision + fixture updates + it unblocks a dependent FOLLOWUP (`restore-multisig-taproot-reconstruction`). Treat as its own cycle; do not lump with the trivia.
3. **Drift is minor everywhere** (synthesize `:399→:398`, canonicity `:138→:132`, friendly "3→12 tests") — no structural mis-citation; #1's citation even carries an in-code FOLLOWUP back-reference.
4. **#2's premise softened** (12 tests now, not 3) — re-count coverage before scoping the test additions; don't trust the as-filed "~67 untested".

---

## Recommended brainstorm-session scope
**Three cycles, not one** (the user's 4 split by repo + size):

- **Cycle A — `toolkit-trmultia-nums-internal-key` (toolkit, substantial).** Own cycle: brainstorm the NUMS-internal-key decision → SPEC → mandatory R0 → implement (`template.rs:208`/`:446` + md1 wire fixtures) → verify round-trip + the `restore-multisig-taproot-reconstruction` unblock. **SemVer: MINOR** (changes emitted md1 wire for tr multisig). **Locksteps:** check `docs/manual` if it documents the tr-multisig md1 internal-key shape; no GUI `schema_mirror` (wire output, not clap flags); update md1 fixtures.
- **Cycle B — `friendly-mapper-unit-test-gaps` + `bundle-emit-bypasses-chunk-mk1-alias` (toolkit, small).** Bundle: both are toolkit, low-risk, no wire/CLI change. Test additions + a 2-line alias swap. **SemVer: no-bump** (test + byte-identical cleanup). One SPEC + one R0. ~quick.
- **Cycle C — `canonicity-drift-gate-floor-too-lenient` (mnemonic-gui repo, separate).** GUI cycle: per-fixture expectation table replacing the 50% floor. GUI conventions + companion FOLLOWUP. **SemVer:** GUI's own (likely no-bump test-hygiene).

Ordering: A independent (the real fix); B independent (trivial, can go first as a warm-up); C is in another repo (do whenever, with the GUI toolchain). No inter-slug dependency among A/B/C (A's dependent is the separate `restore-multisig-taproot-reconstruction`, not B/C).
