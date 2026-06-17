# cycle-prep recon — 2026-06-04 — restore-multisig-cosigner-scope / verify-bundle-descriptor-entropy-slot-gap / schema-mirror-flag-name-vs-wire-shape-conceptual-clarification

**Origin/master SHA at recon time:** `0f404ae`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** prior `cycle-prep-recon-*.md`, `feature-coverage-survey-*.md`, `CONTINUITY.md`, `.claude/` notes (no tracked-file drift)

Slug(s) verified: `restore-multisig-cosigner-scope`, `verify-bundle-descriptor-entropy-slot-gap`, `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`. **All three are CLEAN — zero DRIFTED-by-N, zero STRUCTURALLY-WRONG** (all filed within the last ~2 weeks; slug 1 was filed in the immediately-prior cycle, so citations have not yet decayed).

Sibling-repo states at recon time: `descriptor-mnemonic` md-codec `0.35.0` (== toolkit pin); `mnemonic-gui` origin/master `48a3a0f` (0 ahead / 0 behind).

---

## Per-slug verification

### `restore-multisig-cosigner-scope`
- **WHAT (from FOLLOWUPS.md):** Extend `mnemonic restore` to multisig-cosigner: own seed (`ms1`/`phrase`/`entropy`/`seedqr`) + shared `md1` policy template + other cosigners' `mk1`/`xpub` → concrete watch-only multisig descriptor, with own-seed-derived cosigner xpub cross-checked against the md1 slot. Same fingerprint hard-gate / `--allow-mismatch` / watch-only-out invariants as single-sig (shipped v0.43.0).
- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/restore.rs` "a multisig branch" — **ACCURATE** — multisig is DEFERRED, not yet implemented: `restore.rs:14` "//! Multisig restore is DEFERRED (SPEC §11 …)"; `:168-170` rejects a multisig `--template` via `is_multisig()`. The branch to add does not yet exist (correct).
  - `crates/mnemonic-toolkit/src/wallet_export/mod.rs:262` `template_from_descriptor` takes a miniscript `MsDescriptor`, NOT `md_codec::Descriptor` — **ACCURATE** — `262: pub(crate) fn template_from_descriptor(` / `263: d: &MsDescriptor<DescriptorPublicKey>,`.
  - `crates/mnemonic-toolkit/src/cmd/bundle.rs:1015` `extract_multisig_threshold(node: &md_codec::tree::Node) -> Option<u8>` — **ACCURATE** — `1015: fn extract_multisig_threshold(node: &md_codec::tree::Node) -> Option<u8> {` and it is **private** (`fn`, no `pub`), matching the "currently private / bump to `pub(crate)`" claim.
  - `crates/mnemonic-toolkit/src/cmd/bundle.rs:1138` `bundle_run_unified_descriptor` (production md1→concrete `--descriptor` STRING lex/resolve/parse/bind path) — **ACCURATE** — `1138: fn bundle_run_unified_descriptor<W: Write, E: Write>(`.
  - `crates/mnemonic-toolkit/src/wallet_export/pipeline.rs:18` `build_descriptor_string` — **ACCURATE** — `18: pub(crate) fn build_descriptor_string(`.
  - `descriptor-mnemonic/crates/md-codec/src/to_miniscript.rs:53` `to_miniscript_descriptor` — **ACCURATE** — `53: pub fn to_miniscript_descriptor(`.
  - `…to_miniscript.rs:72` errors `MissingPubkey { idx }` on a template-only md1 — **ACCURATE** — `72: let xpub_bytes = e.xpub.ok_or(Error::MissingPubkey { idx: e.idx })?;`.
  - Claim: `md_codec::Descriptor` has no `Display` — **ACCURATE** — only `pub struct Descriptor` def at `md-codec/src/encode.rs:17`; no `impl Display for Descriptor` and no `#[derive(…Display…)]` anywhere in `md-codec/src/`.
  - `design/SPEC_mnemonic_restore.md §11` + `design/IMPLEMENTATION_PLAN_mnemonic_restore.md` — **ACCURATE** — §11 at `SPEC:144` ("DEFERRED — multisig-cosigner scope"); it spells out the same three bridge options (a)/(b)/(c) + I4 (wallet-policy-vs-template-only `tlv.pubkeys` auto-detect) + cosigner cross-check + `--cosigner @N=mk1|xpub` (decode mk1 via `mk_codec::decode`; no new slot subkey). IMPLEMENTATION_PLAN exists.
  - md-codec pinned `0.35.0` — **ACCURATE** — `Cargo.toml:27 md-codec = "0.35"`; `Cargo.lock` `md-codec 0.35.0` (registry); local `descriptor-mnemonic` tree is `0.35.0` (matches the consumed crate).
- **Action for brainstorm spec:** Citations are live — lift them verbatim, citing source SHA `0f404ae` (toolkit) and md-codec `0.35.0`. The follow-on SPEC must still **choose + R0 one of bridge options (a)/(b)/(c)** from SPEC §11 before any code (the route is provably NOT implementable from the originally-cited APIs as-is). Per SPEC §11 the recommended low-risk path is **(a)** — accept a `--descriptor '<@N-template-string>'` and reuse the already-verified `bundle_run_unified_descriptor` bind path, with `--md1` as cross-check-only; option **(b)** (derive `CliTemplate` from md1 policy params: `extract_multisig_threshold(&d.tree)` bumped to `pub(crate)` + `d.n` → `build_descriptor_string`) is the alternative. New SPEC + mandatory R0 to 0C/0I required.

### `verify-bundle-descriptor-entropy-slot-gap`
- **WHAT (from FOLLOWUPS.md):** The `verify_bundle` descriptor-mode binding loop has arms for `Phrase`/`Seedqr`, `Xpub`, `Ms1` but **no `SlotSubkey::Entropy` arm**, so `verify-bundle --descriptor <template> --slot @N.entropy=<hex>` falls through to the catch-all `else → DescriptorReparseFailed` (exit 2). Mirror the `bundle`-loop `Entropy` arm into the verify-bundle descriptor loop, deriving via `derive_slot::derive_bip32_from_entropy_at_path` at `anno_path`.
- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` descriptor binding loop `if Phrase||Seedqr … else if Xpub … else if Ms1 … else → DescriptorReparseFailed`, "~`:788-890`" — **ACCURATE** — chain at `:788` (`if … Phrase || :789 Seedqr`), `:830` (`else if … Xpub`), `:855` (`else if … Ms1`), `:886` (`return …DescriptorReparseFailed`). Catch-all sits at 886, inside the cited `788-890` window.
  - Claim: "NO `SlotSubkey::Entropy` arm" — **ACCURATE** — `grep -c 'SlotSubkey::Entropy' verify_bundle.rs` → `0`.
  - Claim: the `bundle` descriptor loop (`bundle_run_unified_descriptor`) DOES have an `Entropy` arm — **ACCURATE** — `bundle.rs:1438 } else if subkeys.contains(&…SlotSubkey::Entropy) {`, deriving at `:1486 derive_slot::derive_bip32_from_entropy_at_path(`.
  - Claim: the `verify_bundle` TEMPLATE path resolves `@N.entropy=` via shared `resolve_slots` — **ACCURATE** — `bundle.rs:453 pub(crate) fn resolve_slots(`, `Entropy` arm at `:610`.
  - Derivation helper `derive_slot::derive_bip32_from_entropy_at_path` — **ACCURATE** — `crates/mnemonic-toolkit/src/derive_slot.rs:65 pub(crate) fn derive_bip32_from_entropy_at_path(`.
- **Action for brainstorm spec:** Citations live — cite SHA `0f404ae`. Mirror the `bundle.rs:1438` `Entropy` arm into the verify_bundle descriptor loop (insert a new `else if … SlotSubkey::Entropy` arm before the `:886` catch-all), deriving the cosigner xpub via `derive_slot::derive_bip32_from_entropy_at_path` at the `anno_path`. No new flag/value — `entropy` is an established `--slot` subkey (in `SECRET_SLOT_SUBKEYS`); this only makes it work in one more mode. Add a `cli_verify_bundle` descriptor+`@N.entropy=` round-trip test (currently uncovered). **No GUI `schema_mirror` and no manual lockstep** (no clap-surface change).

### `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`
- **WHAT (from FOLLOWUPS.md):** Document that the GUI `schema_mirror` gate enforces **clap flag-NAME parity** (+ dropdown value-enums) between the hand-maintained `SubcommandSchema` and `gui-schema` JSON — NOT runtime `--json` wire-shape. Option (c) [document gap in CLAUDE.md] **shipped v0.34.3**; residual = option (b) [per-consumer `--json` wire-shape regression tests on the GUI side for high-traffic subcommands].
- **Citations:**
  - `mnemonic-gui/tests/schema_mirror.rs:91-121` (gate iterates flag names) — **ACCURATE** — `91: fn assert_schema_matches_help(schema: &schema::Schema)`; body (91→~122) computes `from_schema.difference(&from_upstream)` over flag-name **sets** and asserts both diffs empty. (Dropdown value-enums travel via the JSON `choices` field; cf. `:337`/`:355`/`:358`.)
  - `mnemonic-gui/tests/xpub_search_schema_mirror.rs` — **ACCURATE** — file exists.
  - `mnemonic-toolkit`'s `gui-schema` subcommand at `cmd/gui_schema.rs` — **ACCURATE** — `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:1 //! mnemonic gui-schema subcommand …`; the dispatch is `cmd::gui_schema` (`lib.rs:88`).
  - Status claim "option (c) shipped v0.34.3 (CLAUDE.md documents the gap)" — **ACCURATE** — CLAUDE.md "GUI schema-mirror coverage" section states the gate enforces clap flag-NAME parity only, NOT runtime `--json` wire-shape, and names this slug's option (b) as the v0.30+ extension.
- **Action for brainstorm spec:** **Not a toolkit code cycle.** The primary recommendation (option c, document) already shipped; the only residual is **option (b)** — GUI-side per-consumer `--json` wire-shape regression tests for `xpub-search`/`import-wallet`/`export-wallet`. That work is **owned by `mnemonic-gui`**, not the toolkit. Recommend: do NOT bundle into a toolkit brainstorm; either (1) open/keep a GUI-repo task for option (b), or (2) close this as "documented (c) shipped; (b) standing-posture, paired-PR rule covers wire-shape evolution." If kept, cite the standing-posture note in `CLAUDE.md` + the companion `ms-kofn-json-wire-shape-ungated` entry.

---

## Cross-cutting observations
1. **Zero citation drift across all three slugs.** Every file path, line number, symbol, and factual claim matches current `origin/master` (`0f404ae`) — including the two sibling-repo citations (md-codec `0.35.0`, mnemonic-gui `48a3a0f`). This is expected: slug 1 was filed in the immediately-prior cycle (v0.43.0, 2026-06-04), slug 2 one day earlier, and slug 3 was last narrowed 2026-05-22. None has yet survived a churning merge.
2. **The three slugs are NOT one cycle — they are three different sizes/owners.** Slug 1 = a large toolkit MINOR needing its own SPEC + R0 + a bridge-option decision (GUI + manual lockstep fire). Slug 2 = a tiny toolkit-only additive PATCH, no lockstep. Slug 3 = a near-closed documentation FOLLOWUP whose residual is GUI-owned. Bundling them would couple a big design cycle to a trivial fix and to a no-code item — anti-pattern per `feedback_smaller_cycle_scope_reduces_citation_surface`.
3. **SemVer / sequencing note for slug 1.** FOLLOWUP `Tier: v0.5` (a priority bucket) vs SPEC §11 which explicitly sequences multisig restore as the **next minor, v0.44.0** ("multisig is additive (v0.44.0)"). The SPEC is operative: current shipped is v0.43.0, so multisig restore → **v0.44.0 MINOR** (new `--cosigner`/`--md1`/`--descriptor` inputs on the existing `restore` subcommand; additive feature warrants MINOR). Resolve the Tier-vs-SPEC framing when the SPEC is written.
4. **No claim-counting ambiguity.** Each slug's "arms"/"options" counts are exact and verified (slug 2: exactly 3 present arms + 0 Entropy arm; slug 1: exactly 3 bridge options + I4).
5. **Slug 1 depends on a sibling crate at a fixed pin.** The md1→descriptor route reads `md_codec` `0.35.0` (registry pin). If the chosen bridge option needs an md-codec API change (e.g. a `Display` for `Descriptor`, or exposing wallet-policy parse), that becomes a **cross-repo companion FOLLOWUP + paired publish** (descriptor-mnemonic) — but bridge options (a) and (b) avoid that (they reuse existing toolkit-side APIs), which is why the SPEC prefers them.

---

## Recommended brainstorm-session scope

**Split into (at most) two cycles; drop slug 3 from toolkit scope.**

- **Cycle A — slug 2 (`verify-bundle-descriptor-entropy-slot-gap`): standalone toolkit PATCH (`v0.43.1` or fold into the next PATCH).** ~30–60 LOC (one `else if Entropy` arm mirrored from `bundle.rs:1438` + a `cli_verify_bundle` descriptor `@N.entropy=` round-trip test). **No GUI `schema_mirror` lockstep, no manual mirror** (no clap-surface change — `entropy` is already a recognized `--slot` subkey). Low risk; closes a real asymmetry (template + bundle-descriptor modes already resolve `@N.entropy=`; only verify-bundle-descriptor mode errors). Needs the mandatory R0 like any cycle, but should converge in ~1 round given the tiny, mirror-of-existing surface. Ship this first.

- **Cycle B — slug 1 (`restore-multisig-cosigner-scope`): dedicated MINOR (`v0.44.0`) with its OWN SPEC + mandatory R0.** Larger (new multisig branch in `restore.rs`, cosigner cross-check, `--cosigner @N=mk1|xpub` + `--md1`/`--descriptor` inputs, bridge-option implementation). The SPEC **must choose + R0 one of bridge options (a)/(b)/(c)** before any code — option (a) (reuse the verified `bundle_run_unified_descriptor` bind path; `--md1` cross-check-only) or (b) (md1 policy-param → `build_descriptor_string`) are the low-risk, no-sibling-change paths. **Lockstep fires:** GUI `schema_mirror` (new `RESTORE_FLAGS` entries — paired `mnemonic-gui` PR) **and** the manual mirror under `docs/manual/src/40-cli-reference/41-mnemonic.md` (new `restore` flags). Carry the same fingerprint hard-gate / `--allow-mismatch` / watch-only-out invariants as single-sig. Do NOT couple this to Cycle A.

- **Slug 3 (`schema-mirror-flag-name-vs-wire-shape-…`): do NOT open a toolkit cycle.** Option (c) shipped v0.34.3; residual option (b) is GUI-owned per-consumer `--json` regression tests. Either keep it as a GUI-repo task or close as "documented; (b) standing-posture covered by the paired-PR rule." If the user wants it actioned, it is an `mnemonic-gui` test-authoring task, not a toolkit brainstorm.

**Ordering / dependency:** A and B are independent (different commands); A is a quick win, B is the substantive cycle the user originally asked for (multisig restore). No inter-slug dependency. Recommend A → B, or B alone if the user only wants the multisig feature.
