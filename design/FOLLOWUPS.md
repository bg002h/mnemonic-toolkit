# Follow-up tracker

Single source of truth for items that surfaced during a review or implementation pass but were not fixed in the same commit. Mirrors the conventions of the sibling `descriptor-mnemonic`, `mnemonic-key`, and `mnemonic-secret` repos.

## How to use this file

**Format for each entry:**

```markdown
### `<short-id>` — <one-line title>

- **Surfaced:** Phase X review of commit <SHA>, or "inline TODO at <file>:<line>"
- **Where:** `<file>:<line>` or "design — SPEC §X"
- **What:** 1–3 sentences describing the gap or improvement opportunity
- **Why deferred:** the reason it didn't ship in the original commit
- **Status:** `open` | `resolved <COMMIT>` | `wont-fix — <one-line reason>`
- **Tier:** `v0.1-blocker` | `v0.1-nice-to-have` | `v0.2` | `cross-repo` | `v1+` | `external`
```

Reference the `<short-id>` from commit messages when closing: `closes FOLLOWUPS.md <short-id>`.

## Tiers (definitions)

- **`v0.1-blocker`**: must fix before tagging `mnemonic-toolkit-v0.1.0`. (Empty after release.)
- **`v0.1-nice-to-have`**: should fix before v0.1 if time permits, but won't block release. Documented in v0.1's CHANGELOG if shipped.
- **`v0.2`**: explicitly deferred to v0.2 (multisig templates, non-zero account, K-of-N share bundles).
- **`v0.2-nice-to-have`**: surfaced during v0.2 review; non-blocking. Documented in v0.2's CHANGELOG if shipped.
- **`v0.3`**: explicitly deferred to v0.3 (user-supplied descriptor passthrough; resolve during v0.3 cycle).
- **`v0.3-nice-to-have`**: surfaced during v0.3 review; non-blocking.
- **`v0.4-cross-repo`**: deferred to v0.4 AND requires coordination with sibling repos.
- **`v0.4-nice-to-have`**: surfaced during v0.4 review; non-blocking. Documented in v0.4's CHANGELOG if shipped.
- **`v0.4.1`**: explicitly deferred from v0.4.0 to a v0.4.1 follow-on patch (typically scope-safety deferrals).
- **`v0.4.2`**: explicitly deferred from v0.4.1 to a v0.4.2 follow-on patch.
- **`v0.4.2-nice-to-have`**: surfaced during v0.4.1 review; non-blocking. Documented in v0.4.2's CHANGELOG if shipped.
- **`v0.4.3`**: explicitly deferred to a v0.4.3 follow-on patch.
- **`v0.4.3-nice-to-have`**: surfaced during v0.4.2 review; non-blocking.
- **`v0.4.4`**: explicitly deferred to a v0.4.4 follow-on patch.
- **`v0.4.4-nice-to-have`**: surfaced during v0.4.3 review; non-blocking.
- **`v0.5`**: explicitly deferred to a v0.5 minor release (typically scope too large for a v0.4.x patch).
- **`cross-repo`**: depends on coordination with sibling repos (`descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret`). Mirrored by a companion entry in the affected sibling's tracker; both cite each other.
- **`v1+`**: deferred indefinitely.
- **`external`**: depends on upstream work (e.g., a sibling crate exposing a helper).

---

## Open items

### `inspect-json-schema-version-backfill` — backfill `schema_version: "1"` field on `InspectJson` envelope to match `XpubSearchJson`

- **Surfaced:** 2026-05-18, v0.26.0 C1 (path-of-xpub) — `XpubSearchEnvelope` introduces a top-level `schema_version: "1"` field for forward-compat versioning of the per-mode tagged-union body. `InspectJson` (and `RepairJson`) carry no equivalent field; consumers that learn to read `schema_version` from `xpub-search` JSON have no parallel signal on inspect/repair envelopes.
- **Where:** `crates/mnemonic-toolkit/src/cmd/inspect.rs` `InspectJson` struct; `crates/mnemonic-toolkit/src/repair.rs` `RepairJson` struct.
- **What:** Add a top-level `schema_version: "1"` (or fresh integer initializer) to both `InspectJson` and `RepairJson` envelopes; document the SemVer compatibility policy (`Major.Minor.Patch` shape; additive fields require a Minor bump). Coordinate with mnemonic-gui consumer paths.
- **Why deferred:** scope discipline at C1; the existing consumers parse the existing envelope shape and would break on the additive field unless their parsers tolerate unknown top-level keys. v0.27+ touch.
- **Status:** resolved — v0.27.0 Phase 1 (`InspectEnvelope { schema_version, body: InspectJson }` wrapper mirroring `XpubSearchEnvelope` precedent at `cmd/xpub_search/mod.rs:111-116`). FOLLOWUP-body wording cited "both `InspectJson` and `RepairJson`"; source-verification (R3) found `RepairJson` ALREADY carries `schema_version: "1"` inline at `cmd/repair.rs:155` + construct site `cmd/repair.rs:178` (latent FOLLOWUP-body inaccuracy). Closes as no-op for Repair side; ships InspectEnvelope only.
- **Tier:** `v0.27`

### `xpub-search-passphrase-bruteforce` — brute-force passphrase scanning over a candidates file / wordlist for `xpub-search passphrase-of-xpub`

- **Surfaced:** 2026-05-18, v0.26.0 C4 (passphrase-of-xpub MVP scope). The C4 mode verifies a SINGLE passphrase; brute-force scanning is a deliberate MVP exclusion.
- **Where:** `crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs` + new `passphrase_search.rs` for the iterator/streaming.
- **What:** Three modes to consider: (a) `--passphrases-file <path>` newline-delimited candidates; (b) `--passphrases-stdin` streamed candidates; (c) generated wordlists (e.g. EFF Long, common passphrases). Per-candidate stderr-advisory budget; rate-limit / progress reporting; deterministic iteration order; abort-on-first-match. Engineering surface includes resource-bounding (memory + time) and a clear "give-up" exit code.
- **Why deferred:** scope discipline at C4; the single-passphrase verification covers the common-case forensic question ("does X passphrase produce this xpub?"). Brute-force needs a careful UX + rate-limit story to avoid foot-guns.
- **Status:** open
- **Tier:** `v0.27`

### `xpub-search-manual-gui-chapters` — `mnemonic-gui` user manual chapters for the 4 new `xpub-search` modes

- **Surfaced:** 2026-05-18, v0.26.0 C5 cycle-close — manual-gui chapters were deferred per user direction during the C5→C6 boundary check.
- **Where:** `docs/manual-gui/src/40-mnemonic/4c-xpub-search-path-of-xpub.md` (and 4d/4e/4f for the other 3 modes); `docs/manual-gui/tests/expected_gui_schema_inventory.json` (regenerate from extract_gui_schema.py); `docs/manual-gui/pinned-upstream.toml` (bump mnemonic-gui pin to v0.11.0 once tagged).
- **What:** Four new chapters mirroring the existing `4b-slip39-combine.md` pattern (~200-500 LOC each): synopsis, per-flag anchor sections with `id="mnemonic-<sub>-<flag>"`, NodeValueComposite node enumerations (where applicable), dropdown variant anchors, exit-codes table, warnings + secret-class material disclaimers. Drives the `gui-schema-coverage` lint gate (bidirectional anchor parity vs the GUI's `SubcommandSchema` source at the pinned tag).
- **Why deferred:** user direction during C5→C6 boundary — 4 chapters × ~200-500 LOC each (~800-2000 LOC total prose). Out-of-band cycle.
- **Status:** open
- **Tier:** `v0.27`

### `mlock-g1-1-test-page-alignment-luck` — `mlock_unit::g1_1_single_page_pin_has_page_count_one` flakes under parallel test execution

- **Surfaced:** 2026-05-18, v0.26.0 C1 cycle observation. After C1's binary-layout shift the test fails under `cargo test`'s default parallel test runner because the heap allocator's bump pointer for a fresh `Box<[u8; 64]>` happens to straddle a page boundary; passes single-threaded (`cargo test -- --test-threads=1`). Pre-existing brittleness pattern (heap-allocator-luck), not a regression introduced by xpub-search.
- **Where:** `crates/mnemonic-toolkit/tests/mlock_unit.rs:28` (assertion site); `crates/mnemonic-toolkit/src/mlock.rs::pin_pages_for` (page-count derivation).
- **What:** Pin the test buffer at a known page-aligned address (e.g., `std::alloc::alloc` with a Layout that forces alignment to `*PAGE_SIZE*`) so the assertion is invariant across parallel-execution heap states. Alternative: relax the assertion to `>= 1 && <= 2` and add a paired test that uses an aligned allocator to pin the exact-page-count guarantee.
- **Why deferred:** non-regression (single-threaded passes; the v0.10.0 mlock cycle landed under this pre-existing flake too — see `feedback-default-cargo-test-runs-sibling-dependent-tests` memory). v0.27+ touch.
- **Status:** open
- **Tier:** `v0.27`

### `xpub-search-gui-bespoke-hub-pane` — discoverable umbrella hub UI for `xpub-search` modes

- **Surfaced:** 2026-05-18, v0.26.0 C5 plan-vs-codebase recon. Plan §7.2 enumerated a "hub" navigation pane with nav cards linking to the 4 mode panes. The GUI has no pane abstraction — every subcommand is a flat row in the subcommand-name ComboBox.
- **Where:** `mnemonic-gui/src/main.rs:346-602` (central panel renderer; net-new per-pane dispatch branch); `mnemonic-gui/src/schema/mnemonic.rs` (a new `SubcommandSchema` entry for the hub itself, or a sibling navigation manifest).
- **What:** Introduce a "hub" pseudo-pane visible when the user picks the umbrella `xpub-search` from a dropdown above the subcommand selector. Hub renders 4 cards (one per mode) with mode-name + 1-line description + click-through. v0.12.0 UI polish; not a v0.11.0 blocker.
- **Why deferred:** C5 plan-vs-codebase recon revealed plan §7.2's "pane" architecture was overspecified; v0.11.0 ships the 4 modes via the generic flag-renderer + the existing subcommand-name ComboBox.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `xpub-search-gui-bespoke-hub-pane`.

### `xpub-search-gui-bespoke-widgets` — per-mode composite widgets (TargetXpubField / DescriptorIntakeField / TargetAddressField / etc.)

- **Surfaced:** 2026-05-18, v0.26.0 C5 plan-vs-codebase recon. Plan §7.3 enumerated `SeedIntakeWidget`, `TargetXpubField`, `DescriptorIntakeField`, `TargetAddressField`, `AddPathRepeater`, `XpubSearchResultRenderer` as net-new widgets. GUI codebase has NO `PhraseField` / `PassphraseField` / `Ms1Field` named types; the plan's "widget reuse" framing was wrong.
- **Where:** `mnemonic-gui/src/form/` (new modules).
- **What:** Per-mode composite widgets with affordances beyond the generic `widget::render` dispatch: TargetXpubField with prefix-detect badge; AddressTypeField that auto-suggests from xpub prefix; DescriptorIntakeField with multi-line textarea + shape-detect badge; AddPathRepeater with +/− buttons. v0.12.0 polish.
- **Why deferred:** v0.11.0 ships via the generic FlagKind dispatcher; the bespoke widgets are UX polish, not functional blockers.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `xpub-search-gui-bespoke-widgets`.

### `xpub-search-gui-positional-intake` — positional ms1 (HRP-autodetect) routing in mnemonic-gui

- **Surfaced:** 2026-05-18, v0.26.0 C5. The toolkit accepts a positional ms1 (HRP-autodetect) on P1/P2/P4; the GUI's argv assembler does not surface this affordance — the GUI forces users into `--ms1` explicitly.
- **Where:** `mnemonic-gui/src/form/invocation.rs::assemble_argv`; `mnemonic-gui/src/schema/mnemonic.rs` `positional_args: NO_POSITIONALS` on the 4 xpub-search entries.
- **What:** Add a "drop any card" textarea/file-drop affordance that auto-routes via HRP detection (`ms1` → positional, `mk1`/`md1` → would be future modes' surfaces). v0.12.0 polish.
- **Why deferred:** v0.11.0 keeps GUI argv assembly simple; positional intake is a polish item.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `xpub-search-gui-positional-intake`.

### `xpub-search-descriptor-md1-detection-bech32-validate` — md1 tie-break uses `starts_with("md1")` not full bech32 validation

- **Surfaced:** 2026-05-18, v0.26.0 holistic architect review (C2 R0 m2 carry-forward). `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs:167` routes any tokens-list whose entries all start with the literal `"md1"` prefix into the md1 chunk-assembly funnel. A garbage token like `md1xxx` routes there and fails at `md_codec::chunk::reassemble` with a clear typed error rather than falling through to literal-xpub.
- **Where:** `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs:167`.
- **What:** Tighten the tie-break to a real bech32 syntactic validation (e.g. `bech32::decode(t).is_ok() && t.starts_with("md1")` — even a shape-only check would be tighter). Defensible to leave as-is (the md1 HRP is unambiguous and false-positives surface as typed errors); fold or defer based on appetite for tightening.
- **Why deferred:** v0.26.0 behavior is correct (false-positives produce clean error messages, not silent misroutes); not a v0.26.0 blocker.
- **Status:** open
- **Tier:** `v0.27-nice-to-have`

### `xpub-search-address-of-xpub-searched-count-semantic` — P3 `XpubSearchNoMatch.searched` over-reports candidates

- **Surfaced:** 2026-05-18, v0.26.0 holistic architect review (C3 R0 m2 carry-forward). `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:290-293` computes the `searched` count for `ToolkitError::XpubSearchNoMatch` as `num_targets × gap_limit × chains`. The actual candidate-set is `gap_limit × chains` (one shared rendered-address Vec; see `address_search.rs:75-90` for the shared-build). For 3 targets / gap_limit=20 / 2 chains: reports 120 truth-is-40.
- **Where:** `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:290-293`.
- **What:** Decide on the canonical semantic — "total candidate-comparisons performed" (current; correct under that read) vs "unique child-addresses derived" (truth: `gap_limit * chains`). Either fix the count or document the chosen semantic inline. The per-target JSON envelope's `scanned_external` + `scanned_internal` fields are already correct for the per-target perspective.
- **Why deferred:** semantic-debate, not a correctness regression; the per-target JSON envelope fields are correct. v0.26.0 ships the slightly-over-reported aggregate.
- **Status:** open
- **Tier:** `v0.27-nice-to-have`

### `xpub-search-gui-flag-mutex-visibility` — cross-flag conditional visibility for `xpub-search` mutex groups

- **Surfaced:** 2026-05-18, v0.26.0 C5. The 4 xpub-search SubcommandSchema entries set `conditional: None` for v0.11.0; cross-flag mutex visibility (e.g., greying `--ms1` when `--phrase` is filled in) is open.
- **Where:** `mnemonic-gui/src/form/conditional.rs` (new per-subcommand functions following the existing pattern at `slip39_split` / `slip39_combine` / `repair` / `inspect` / `derive_child` / etc.).
- **What:** Per-subcommand `fn(&FormState) -> FlagVisibility` functions that grey/hide flags based on cross-flag state. For xpub-search modes: enforce the seed-intake mutex visually (only one of `--phrase` / `--phrase-stdin` / `--ms1` / `--ms1-stdin` interactive at a time); surface the P4 mandatory-passphrase requirement before run-confirm; flag the multi-`--target-address` repeating affordance in P3. v0.12.0 polish.
- **Why deferred:** v0.11.0 ships with `conditional: None`; the user sees all flags simultaneously and clap-side handles the mutex at exec. The GUI's run-confirm modal will surface clap errors verbatim — functional but not ideal UX.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `xpub-search-gui-flag-mutex-visibility`.

### `verify-bundle-watch-only-xpub-path-internal-consistency` — watch-only verify-bundle does not cross-check mk1 xpub byte-level fields against md1's claimed OriginPath

- **Surfaced:** 2026-05-17, Q&A session in `descriptor-mnemonic` repo while explaining xpub↔keypath semantics. User asked whether `verify-bundle` cross-checks md1 and mk1 internal claims; code-read confirmed it does not.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — `run_watch_only` at `:306`, watch-only multisig branch in `run_multisig` at `:386–:413`, and the comparison sites in `emit_verify_checks` (`mk1_path_match` at `:1087–:1108`, `mk1_fingerprint_match` at `:1058–:1085`) and `emit_md1_checks` (`md1_xpub_match` at `:1654–:1691`). All checks currently compare supplied cards against a synthesized `expected` Bundle built from `--slot @N.xpub=` + template + canonical BIP path; none read mk1's `KeyCard.xpub` byte-level fields (depth, child_number, parent_fingerprint) for direct comparison against md1's `Descriptor.tlv.origin_paths` length/last-element/parent-fingerprint.
- **What:** Add a no-seed-required internal-consistency cross-check between the supplied mk1 xpub and the supplied md1 `OriginPath`. Three byte-comparison assertions (all derivable from already-decoded structs, no key derivation):
  1. `xpub.depth == origin_path.len()` (path length matches BIP-32 depth field).
  2. `xpub.child_number == origin_path.last()` (final index matches, with hardened-bit comparison preserved).
  3. For multisig: cross-cosigner `parent_fingerprint` consistency where the same parent is claimed.
  Emit as new checks (e.g., `mk1_md1_path_internal_consistency[i]`) in the SPEC §5.4 / §5.7 schema; SPEC bump per the schema-version policy. Closes the daylight between the existing stderr warning ("watch-only … does not verify --slot @0.xpub= is actually at the claimed BIP path") and the broader claim that watch-only mode can catch *internally* inconsistent bundles even without seed access.
- **Why deferred (historical):** not a correctness regression — current behavior matches the disclaimed warning at `:317–:331`. No concrete bundle-mismatch report driving urgency. Touching the SPEC §5.4 / §5.7 check schema (additive but version-bumping) is non-trivial scope for an unscheduled enhancement; better folded into a future verify-bundle hardening cycle than bolted on as a patch.
- **Resolution:** RESOLVED in v0.24.0 cycle (Tranche A.1 D30 tier upgrade — rationale: "consolidating verify-bundle defense-in-depth work in v0.24.x cycle; cross-check is cheap and the related code is in active flight"). Shipped a stderr-WARNING (not hard-error / not VerifyCheck-schema-additive) cross-check in `emit_watch_only_xpub_path_cross_check` called from `run_watch_only` and the watch-only branch of `run_multisig`. Three checks per cosigner:
  1. `xpub.depth == md1 OriginPath length` (mk-codec reconstructs depth from origin_path at decode, so this effectively asserts mk1.origin_path.len() == md1 path-decl length).
  2. `xpub.child_number == md1 OriginPath last component` (value + hardened bit).
  3. Parent-fingerprint structural sanity: at md_depth 0 the BIP-32 master invariant requires `parent_fingerprint == [0;4]`; at md_depth 1 the parent IS the master, so `parent_fingerprint` must equal the claimed master fingerprint (via md1 TLV `fingerprints` or mk1's `origin_fingerprint`). Deeper paths skip the check (would require parent xpub derivation, infeasible without seed).
  Failure mode: stderr WARNING per cosigner. Verify-bundle exit code + `result: ok / mismatch` verdict UNCHANGED — the SPEC §5.4 / §5.7 check schema is intentionally NOT extended (kept the design simple; the existing VerifyCheck rows already cover the load-bearing checks). 5 new integration cells in `crates/mnemonic-toolkit/tests/cli_verify_bundle_watch_only.rs` covering happy-path silence + 3 single-cosigner failure modes + 1 multi-cosigner failure mode. Resolution commit: TODO (this cycle's release tag).
- **Status:** resolved v0.24.0 cycle
- **Tier:** `v0.24.0`

### `gui-schema-global-flag-emission` — `mnemonic gui-schema` JSON omits global flags from per-subcommand schemas

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle Phase A.1 execution (mnemonic-gui v0.9.0 catchup). Realized R7 risk from `/home/bcg/.claude/plans/nifty-wiggling-gosling.md` §5. mnemonic-gui v0.9.0 attempted to mirror `--no-auto-repair` into its 10 per-subcommand `*_FLAGS` arrays and the schema-mirror drift gate hard-failed: the toolkit's `cmd::gui_schema` v4 JSON emitter does not include global flags in any subcommand's `flags` array; only clap's per-subcommand `--help` TEXT propagates them.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (the v4 emitter); downstream consumers across `mnemonic-gui` (currently the only known consumer). GUI workaround at `mnemonic-gui/src/runner.rs::prepend_no_auto_repair` + `MnemonicGuiApp.no_auto_repair` field + action-bar checkbox in `mnemonic-gui/src/main.rs` (load-bearing fallback shipped at `mnemonic-gui-v0.9.0`; ~30 LOC).
- **What:** Extend `cmd::gui_schema`'s emitter to include global flags (e.g. `--no-auto-repair`, `--debug`) per-subcommand so downstream consumers can mirror them in their per-subcommand schemas without inventing fallbacks. Either (a) duplicate the global-flag entries into every subcommand's `flags` array, or (b) add a sibling top-level `global_flags` array consumed alongside per-subcommand flags. Bump schema version per SPEC §6.10.6 additive-bump policy if needed.
- **Why deferred (historical):** the GUI's R7 fallback (action-bar checkbox prepending the flag to argv) was functionally complete at v0.9.0; the toolkit-side emitter fix is a future cycle's mechanical extension. UX improvement (per-subcommand native vs top-level affordance) but not a correctness gap.
- **Resolution:** RESOLVED in v0.24.0 cycle (Tranche B). `cmd/gui_schema.rs` v5 envelope adds three additive fields to every `flags[]` entry: `default_value: Option<String>`, `global: bool`, `secret: bool`. Schema integer version bumped `4 → 5`. `--no-auto-repair` propagates to every subcommand's flags array with `global: true`. mnemonic-gui v0.10.0 consumes the v5 fields, mirrors `--no-auto-repair` natively per-subcommand, and retires the R7 action-bar fallback. Companion FOLLOWUP closure in `bg002h/mnemonic-gui` `FOLLOWUPS.md` lockstep at v0.24.0 release tag.
- **Status:** resolved v0.24.0 cycle
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-global-flag-emission`.

### `toolkit-mnemonic-force-tty-promote-from-test-only` — promote `MNEMONIC_FORCE_TTY` env-var from test-only to first-class public contract

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle D23 lock execution (mnemonic-gui v0.9.0 catchup). Realized R1 risk from `/home/bcg/.claude/plans/nifty-wiggling-gosling.md` §5.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run` doc-comment (currently classifies the env-var as test-only); `crates/mnemonic-toolkit/src/cmd/convert.rs` + `crates/mnemonic-toolkit/src/cmd/inspect.rs` (same env-var consumed via the `is_terminal()` gate); downstream consumer at `mnemonic-gui/src/runner.rs::run` (sets `MNEMONIC_FORCE_TTY=1` on subprocess spawn at `mnemonic-gui-v0.9.0`).
- **What:** mnemonic-gui v0.9.0 sets `MNEMONIC_FORCE_TTY=1` in the toolkit subprocess env so that the toolkit's `std::io::stdout().is_terminal() && !no_auto_repair` auto-fire gate fires for GUI-spawned invocations (GUI subprocesses are piped, not TTY — without the env override the GUI would never see auto-fire repair reports from `convert` / `inspect` / `verify-bundle`). The env-var is currently documented test-only in `verify_bundle::run` doc-comment. GUI consumption creates a load-bearing dependency on the env-var's behavior; promotion to a first-class public contract (with explicit semver guarantee on its semantics) would harden the GUI side against silent toolkit-internal refactors. Update doc-comment in lockstep to reflect public-contract status; consider adding to the manual's "environment variables" section.
- **Why deferred (historical):** functional risk is documentary, not behavioral; the env-var works correctly at v0.22.1. Toolkit-side promotion is a future cycle's documentation + semver-contract addition.
- **Resolution:** RESOLVED in v0.24.0 cycle (Tranche A.1 sub-item 2). Doc-comment in `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run` rewritten to reflect first-class public-contract status with semver-stable semantics; cites mnemonic-gui v0.9.0+ as a known consumer. New "Environment variable `MNEMONIC_FORCE_TTY`" subsection added to the user manual at `docs/manual/src/40-cli-reference/41-mnemonic.md` under the verify-bundle auto-fire section. Companion FOLLOWUP closure in `bg002h/mnemonic-gui` `FOLLOWUPS.md` lockstep at v0.24.x release tag.
- **Status:** resolved v0.24.0 cycle
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `toolkit-mnemonic-force-tty-promote-from-test-only`.

### `verify-bundle-auto-fire-helper-refactor` — wire BCH auto-fire short-circuit into `verify-bundle` decode failures (v0.22.1 patch)

- **Surfaced:** 2026-05-17, v0.22.0 cycle Phase 5 scope-reduction.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — 8 sites: `:887` `:937` `:1096` `:1101` `:1122` `:1127` `:1219` `:1532`. Helpers `emit_verify_checks` (`:856`), `emit_multisig_checks` (`:1083`), `emit_md1_checks` (`:1526`) need signature change `Vec<VerifyCheck>` → `Result<Vec<VerifyCheck>, ToolkitError>` so `?` propagation can short-circuit. 4 production callers (`:283`, `:338`, `:420`, `:682`) + 6 in-file test callers (`:1671`, `:1718`, `:1748`, `:1822`, `:1924`, `:1999`) need updates.
- **What:** v0.22.0 shipped auto-fire on `convert` (sites #9, #10) and `inspect` (site #11) but DEFERRED the `verify-bundle` helper refactor because the signature cascade through 10 callers (including v0.20/v0.21 round-trip regression cells) was high-risk for a single-shot tag window.
- **Why deferred:** scope discipline; the 3 sites already shipped cover the highest-frequency user-visible decode paths (`mnemonic convert --from ms1=… --to phrase` and `mnemonic inspect`). `verify-bundle` keeps its current UX (decode failures still surface as `VerifyCheck { passed: false, decode_error: Some(...) }` rows) until v0.22.1.
- **Resolution:** SHIPPED in v0.22.1 (2026-05-17). 3 helpers refactored to `Result<_, ToolkitError>`; 8 originally-planned sites resolved to 6 actual supplied-side wire-ups (Phase 0 R0 caught the plan's double-count of 2 expected-side decodes at original `:1096`/`:1101`). TTY-conditional default per D18 — auto-fire fires under `is_terminal() && !no_auto_repair`; pipe-context preserves the legacy VerifyCheck row behavior.
- **Status:** resolved v0.22.1 cycle
- **Tier:** `v0.22.1`

### `ms-codec-decode-with-correction-public-api` — promote `ms_codec::decode_with_correction` for downstream BCH consumers

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm + R0.
- **Where:** `mnemonic-secret/crates/ms-codec/src/decode.rs` (cross-repo).
- **What:** Add `pub fn decode_with_correction(s: &str) -> Result<(Tag, Payload, Vec<RepairDetail>)>` that internally runs BCH correction within t=4 capacity before the existing decode pipeline. This lets the toolkit's `repair.rs` consume the sibling-codec native API instead of replicating BCH primitives.
- **Why deferred:** v0.22.0 toolkit-side launch first; consume sibling-codec native APIs once promoted.
- **Status:** `resolved 2026-05-17` — v0.22.x follow-ups cycle Phase B.3+B.4: ms-codec v0.2.0 shipped at `bg002h/mnemonic-secret` `f3fa531` (Phase B.4) with new `bch` module (B.3 `676097d`) + `bch_decode` module + `decode_with_correction(&str) -> Result<(Tag, Payload, Vec<CorrectionDetail>), Error>` + new `Error::TooManyErrors { bound: 8 }` variant. Toolkit-side consumer migration tracked at the companion `toolkit-repair-consume-native-codec-api` entry below (resolved at toolkit `b8ca6df`).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-secret`

### `md-codec-decode-with-correction-public-api` — promote BCH primitives + `decode_with_correction` for md HRP

- **Surfaced:** 2026-05-17, v0.22.0 R0.
- **Where:** `descriptor-mnemonic/crates/md-codec/src/bch.rs` (currently `pub(crate)`; cross-repo).
- **What:** Promote `bch::polymod_run` / `bch::hrp_expand` / `bch::MD_REGULAR_CONST` to `pub`, OR add a `pub fn decode_with_correction(strings: &[&str]) -> Result<Descriptor>` wrapper. Either path lets the toolkit's `repair.rs` consume md-codec primitives instead of vendoring `MD_NUMS_TARGET`.
- **Why deferred:** v0.22.0 vendored the constant + drift-gates it via `#[cfg(test)]` against an md1 stability suite.
- **Status:** `resolved 2026-05-17` — v0.22.x follow-ups cycle Phase B.1+B.2: md-codec v0.34.0 shipped at `bg002h/descriptor-mnemonic` `56dc300` (Phase B.2) with visibility promotions (B.1 `94069ea`: `pub const GEN_REGULAR` / `MD_REGULAR_CONST` + `pub fn polymod_run` / `hrp_expand` / `bch_create_checksum_regular` / `bch_verify_regular`) + new `bch_decode` module (~450 LOC BM+Chien port) + `decode_with_correction(&[&str]) -> Result<(Descriptor, Vec<CorrectionDetail>), Error>` + new `Error::TooManyErrors { chunk_index, bound: 8 }` variant. Toolkit-side consumer migration tracked at the companion `toolkit-repair-consume-native-codec-api` entry below (resolved at toolkit `b8ca6df`). Follow-up scope tracked at new `md-codec-decode-with-correction-supports-non-chunked-md1` (this file).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/descriptor-mnemonic`

### `ms-cli-repair-flag` — `ms repair` subcommand mirroring toolkit's `mnemonic repair`

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm.
- **Where:** `mnemonic-secret/crates/ms-cli/src/cmd/` (NEW subcommand; cross-repo).
- **What:** Add `ms repair <ms1>` for ms1 BCH error-correction. Mirrors `mnemonic repair --ms1`. Blocked on `ms-codec-decode-with-correction-public-api`.
- **Status:** `resolved 2026-05-17` — v0.22.x follow-ups cycle Phase B.5: ms-cli v0.4.0 shipped at `bg002h/mnemonic-secret` `18f558a`. New `ms-cli/src/cmd/repair.rs` with `--ms1 <MS1>` required option + `--json` + exit-code parity (`0`/`5`/`2`) + cross-CLI `RepairJson` parser-reuse (D27) + D9 secret-on-stdout advisory preserved. Wraps `ms_codec::decode_with_correction` (B.4). D25 handler-signature unification cascade (5 pre-existing handlers `Result<()>` → `Result<u8>` with `Ok(0)` terminators). 5 integration cells.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-secret`

### `mk-cli-repair-flag` — `mk repair` subcommand mirroring toolkit's `mnemonic repair`

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm.
- **Where:** `mnemonic-key/crates/mk-cli/src/cmd/` (NEW subcommand; cross-repo).
- **What:** Add `mk repair <mk1>...` for mk1 BCH error-correction (regular + long codes). `mk-codec` already does internal correction within `decode`, so this is mostly a UX-parity feature.
- **Status:** `resolved 2026-05-17` — `mk-cli-v0.4.0` shipped at `bg002h/mnemonic-key` `0ecbf1a` (mnemonic-toolkit v0.22.x follow-ups cycle Phase A.3'; plan `/home/bcg/.claude/plans/nifty-wiggling-gosling.md` §2.A.2).
- **Resolution:** new `mk-cli/src/cmd/repair.rs` consumes `mk_codec::string_layer::decode_string` (already-public BCH primitive per `mk-codec-v0.3.1`); surfaces full `DecodedString` (`code`/`corrections_applied`/`corrected_positions`/`corrected_char_at`); exit 5 = REPAIR_APPLIED per D26; JSON envelope byte-matches toolkit's `RepairJson` schema per D27. 7 new integration cells. D25 handler-signature cascade (6 handlers `Result<()> → Result<u8>`) shipped in the same release commit. Companion FOLLOWUP closure at `bg002h/mnemonic-key` `design/FOLLOWUPS.md` `mk-cli-repair-flag` lockstep.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-key`

### `md-cli-repair-flag` — `md repair` subcommand mirroring toolkit's `mnemonic repair`

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm.
- **Where:** `descriptor-mnemonic/crates/md-cli/src/cmd/` (NEW subcommand; cross-repo).
- **What:** Add `md repair <md1>...` for md1 BCH error-correction. Blocked on `md-codec-decode-with-correction-public-api`.
- **Status:** `resolved 2026-05-17` — v0.22.x follow-ups cycle Phase B.6: md-cli v0.6.0 shipped at `bg002h/descriptor-mnemonic` `d4fbe48`. New `md-cli/src/cmd/repair.rs` with variadic `<MD1_STRINGS>...` positional + `--json` + atomic per-chunk semantics per plan §1 D28 (any chunk failing BCH capacity aborts the whole call; no partial output) + exit-code parity (`0`/`5`/`2`) + cross-CLI `RepairJson` parser-reuse (D27). Wraps `md_codec::decode_with_correction` (B.2). D25 handler-signature unification cascade (9 pre-existing handlers `Result<(), CliError>` → `Result<u8, CliError>` with `Ok(0)` terminators). 5 integration cells. Surfaced new chunked-form-only constraint tracked at companion `md-codec-decode-with-correction-supports-non-chunked-md1`.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/descriptor-mnemonic`

### `toolkit-repair-consume-native-codec-api` — replace toolkit-side BCH replication with sibling-codec native APIs

- **Surfaced:** 2026-05-17, v0.22.0 R1.
- **Where:** `crates/mnemonic-toolkit/src/repair.rs`.
- **What:** Once `ms-codec` + `md-codec` expose native `decode_with_correction` (siblings to mk-codec's existing internal correction), refactor `repair.rs` to delegate per-HRP to each codec's native correction primitive instead of parameterizing the toolkit's own polymod calls. Cleaner layering; one BCH implementation per codec instead of toolkit-side parameter-juggling.
- **Why deferred:** cross-repo dep chain; awaits items #2 and #3.
- **Status:** `resolved b8ca6df` — v0.22.x follow-ups cycle Phase B.7 (mnemonic-toolkit v0.23.0). Deleted `MS_NUMS_TARGET` (was `repair.rs:42`) + `MD_NUMS_TARGET` (was `repair.rs:45`) vendored constants. Deleted `(Self::Ms1, BchCode::Regular)` and `(Self::Md1, BchCode::Regular)` arms from `CardKind::target_residue()` (mk1 arms unchanged — mk-codec primitives still consumed natively). New `repair_via_ms_codec` + `repair_via_md_codec` private helpers delegate to `ms_codec::decode_with_correction` (B.4) and `md_codec::decode_with_correction` (B.2) respectively, with sibling-codec error → `RepairError` translation per plan §2.B.4 D29 error-mapping table. New `RepairError::PostCorrectionDecodeFailed { chunk_index: Option<usize>, detail: String }` catch-all variant absorbs orphan §4-rule decoder errors that the per-variant translation table did not enumerate. Public `repair_card` contract unchanged. All 32 pre-existing repair cells stay green via substring-match (not byte-exact equality).
- **Tier:** `cross-repo`

### `md-codec-decode-with-correction-supports-non-chunked-md1` — toolkit consumer perspective of the chunked-form-only constraint

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle Phase B.6 implementation surfaced the gap; filed at Phase B.8 (release-boundary docs). Toolkit-side companion to the descriptor-mnemonic primary entry.
- **Where:** `crates/mnemonic-toolkit/src/repair.rs::repair_via_md_codec` (Phase B.7 delegation point) → `md_codec::decode_with_correction` (B.2 surface) → `chunk::split` + `chunk::reassemble` (the chunked-form-only integration points). After Phase B.7 migration, the toolkit's `mnemonic repair --md1` inherits the chunked-form-only constraint: users attempting to repair a non-chunked single-string md1 (the form emitted by plain `md encode` for small payloads) see a wire-format-mismatch error from the sibling codec, wrapped through `repair_via_md_codec` as `RepairError::UnparseableInput` or `RepairError::PostCorrectionDecodeFailed`.
- **What:** Toolkit-side consumer tracker for the md-codec primary. When the primary lands its non-chunked-form coverage, the toolkit migration (no code change required — the delegation already routes through the updated md-codec API) needs a CHANGELOG note + manual chapter update + smoke test confirming non-chunked-form repair works end-to-end through the toolkit's `mnemonic repair --md1`. No toolkit code change in v0.23.0; consumption tracked for the future md-codec patch release.
- **Why deferred (historical):** Primary work lived in md-codec (B.2's `chunk::split`/`chunk::reassemble` integration); toolkit consumed whatever md-codec exposed. The chunked-form path covers the most common multi-chunk error-recovery use case; the non-chunked-form tail-case was a UX nice-to-have until shipped.
- **Resolution:** RESOLVED in v0.24.0 cycle (downstream consumer of Tranche D). md-codec v0.35.0 (`crates/md-codec/src/chunk.rs::decode_with_correction`) added non-chunked-form detection pre-pass: routes `strings.len() == 1` inputs whose first-symbol bit-0 is `0` (non-chunked header sentinel) directly into `decode_payload`, bypassing `chunk::reassemble`. Toolkit consumes the broadened API transparently through the unchanged `repair_via_md_codec` delegation in `crates/mnemonic-toolkit/src/repair.rs` — no toolkit code change required beyond the md-codec dep version bump. `mnemonic repair --md1` now accepts non-chunked single-string md1 inputs end-to-end. Companion FOLLOWUP closures in `bg002h/descriptor-mnemonic` (primary) + `bg002h/mnemonic-gui` (GUI-side consumer) + `bg002h/mnemonic-secret` (sibling-codec mirror) lockstep.
- **Status:** resolved v0.24.0 cycle
- **Tier:** `cross-repo`
- **Companion:** `bg002h/descriptor-mnemonic` `design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (primary; resolved at md-codec v0.35.0); `bg002h/mnemonic-secret` `design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (sibling-codec mirror); `bg002h/mnemonic-gui` `FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (GUI-side consumer).

### `hrp-correction-heuristics` — Levenshtein-1 HRP-typo auto-suggestion

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm §9 open question.
- **Where:** `crates/mnemonic-toolkit/src/repair.rs::RepairError::HrpMismatch` site.
- **What:** When the user supplies `ns1…` / `mz1…` / `mb1…` etc., compute Levenshtein-1 over `{"ms", "mk", "md"}` and suggest the closest valid HRP in the error message. Today the user gets `expected 'ms', found 'ns'` but no "did you mean 'ms'?" prompt.
- **Resolution:** SHIPPED in v0.22.1 (2026-05-17) per D19. Vendored 10-line `hrp_lev1` + `suggest_hrp` in `repair.rs`; extended `RepairError::HrpMismatch` Display arm. Ambiguous inputs (`mb` is 1-sub from all three known HRPs) silently omit the suffix.
- **Status:** resolved v0.22.1 cycle
- **Tier:** `v0.22.1`

### `repair-json-short-circuit-output` — JSON envelope for auto-fire short-circuit when `--json` was requested

- **Surfaced:** 2026-05-17, v0.22.0 D14 decision.
- **Where:** `crates/mnemonic-toolkit/src/repair.rs::emit_repair_report`.
- **What:** When auto-fire fires under a `convert --json` / `inspect --json` invocation, today the repair report is emitted as TEXT-form even though the calling context expected JSON. v0.23 should detect the JSON context and emit a structured JSON envelope wrapping the repair report.
- **Resolution:** SHIPPED in v0.22.1 (2026-05-17) per D20. `try_repair_and_short_circuit` extended with `json_context: bool` param; new `emit_repair_report_json` body emits the AutoFireRepairJson schema with `auto_repair_short_circuit: true` + `exit_code: 5` discriminator fields. Applies to all 3 auto-fire surfaces (convert / inspect / verify-bundle).
- **Status:** resolved v0.22.1 cycle
- **Tier:** `v0.22.1`

### `bech32-correction-api-version-pin` — track upstream `bech32` crate correction API stability

- **Surfaced:** 2026-05-17, v0.22.0 Phase 0 (rust-bech32 v0.11.1 had no public `Corrector` API; vendored via mk-codec primitives instead).
- **Where:** monitor `bech32` crate releases.
- **What:** If/when `bech32 v0.12+` publishes a stable `primitives::correction::Corrector` API, evaluate migrating `repair.rs` off mk-codec primitives onto upstream. Probably blocked indefinitely; mk-codec primitives are stable and serve all 3 HRPs.
- **Upstream status (last checked 2026-05-17):** **still incomplete — no migration unblock signal.** Latest published version on crates.io is **`bech32 v0.11.1`** (docs.rs confirms: no items mentioning `correction`, `Corrector`, `repair`, BCH, or polymod in the public API). Available modules per docs.rs are `hrp`, `primitives`, `segwit` — none expose a correction primitive. The release-tracking PR #189 for `v0.12.0` ("Release tracking PR: `v0.12.0`") was opened 2026-01-16 and remains open as of last check (last update 2026-01-16); CHANGELOG.md on master shows no entries past `0.11.0 - 2024-02-23`. Issue #95 ("Support identifying potentially errorneous characters", open since 2024) is the only feature-tracking signal for correction work and remains unassigned. Recent commit activity on master is limited to CI / rustc-toolchain maintenance (most recent commits 2026-05-12). No open PR introduces a `Corrector` type or `decode_with_correction` API. **Future-cycle action if a `v0.12+` upstream lands a public correction primitive:** evaluate migrating `repair.rs`'s mk1 branch off vendored mk-codec primitives to upstream `bech32`; this would let the toolkit drop its mk-codec dependency on `string_layer::decode_string`'s correction internals and consolidate on a single upstream BCH implementation. Until then the toolkit stays on mk-codec primitives (which work; see `toolkit-repair-consume-native-codec-api` resolution).
- **Status:** open
- **Tier:** `v1+`

### `verify-bundle-auto-fire-feature-flag-survey` — survey users on default-on vs default-off for verify-bundle auto-fire

- **Surfaced:** 2026-05-17, v0.22.0 R6 risk.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (once helper refactor ships per `verify-bundle-auto-fire-helper-refactor`).
- **What:** Auto-fire on `verify-bundle` decode failures changes UX from "VerifyCheck row" to "exit-5 short-circuit + repair report." Survey users whether default-on (helpful) or default-off (preserves existing automation expectations) is preferable. Inform v0.22.1 default.
- **Resolution:** RESOLVED in v0.22.1 (2026-05-17) via D18 TTY-conditional default: auto-fire under TTY, legacy behavior when piped. This middle path obviates the user survey — interactive users get the helpful UX while scripts/CI keep the automation contract. No survey was conducted; the TTY split is the answer.
- **Status:** resolved v0.22.1 cycle
- **Tier:** `v0.22.1`

### `gui-schema-conditional-rules-v1` — project SPEC §6.6/§6.9 mutex/conditional rules into gui-schema JSON (drift-gated)

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle (`design/IMPLEMENTATION_PLAN_gui_conditional_applicability_v1.md`). Motivating bug: GUI bundle form default state (template = `bip84`, single-sig) emits `--threshold 1 --multisig-path-family bip48` which CLI rejects with SPEC §6.6 byte-exact errors (`crates/mnemonic-toolkit/src/cmd/bundle.rs:120`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (P1 target — emit `conditional_rules` array, version bump 1→2); `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10 (P0 canonical home — Predicate AST + Effect + drift invariant); `mnemonic-gui/src/form/conditional.rs` (P2 — ~14 new rules); `mnemonic-gui/src/form/invocation.rs` (P3 — visibility gate); `mnemonic-gui/tests/gui_schema_conditional_drift.rs` (P4 — NEW drift gate); `mnemonic-gui/src/main.rs:197-211` (P5 — remove bad default seed).
- **What:** Cross-repo mechanism + comprehensive rule coverage. Adds machine-readable `conditional_rules` to `mnemonic gui-schema` JSON; GUI's `assemble_argv` gains a visibility gate (Hidden + Disabled suppress emission, Required does not); drift gate test enforces parity between toolkit JSON and GUI hand-coded `conditional.rs`. v1 encodes ~17 enforceable visibility rules across `bundle`, `verify-bundle`, `export-wallet`, `convert`, `derive-child`. Runtime/slot-count-dependent rules deferred; see companion `gui-schema-runtime-conditional-projection`.
- **Why deferred:** in-progress this cycle (toolkit v0.16.0 + mnemonic-gui v0.5.0 lockstep; `mnemonic-gui v0.4.3` cut first as scope-isolated v0.15.0 wire-format catchup per plan §4 prerequisite).
- **Status:** `resolved 519bcfc` — shipped at `mnemonic-toolkit v0.16.0` (2026-05-16). SPEC §6.10 added; gui_schema.rs JSON projection emitted (schema v2); 1001/1001 workspace tests green. Lockstep GUI release at `mnemonic-gui v0.5.0` (commit `7b7e07d`) ships the consumer side.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-conditional-applicability-drift-fix` (resolved at GUI v0.5.0 commit `7b7e07d`).

### `gui-schema-runtime-conditional-projection` — project SPEC §6.6 slot-count-dependent + runtime rules into gui-schema JSON

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle. Filed at cycle open per plan §1.4 + §7 item 1.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (toolkit side — extend `Predicate` AST + `conditional_rules` emitter with slot-count-aware predicates); `mnemonic-gui/src/form/conditional.rs` (gui side — slot-count signal from `FormState` to conditional engine); SPEC §6.6 rows 8, 9, 10, 11, 13, 14 (cross-cite into §6.10.7's "Runtime-deferred rules" block).
- **What:** v1 cycle deferred slot-count-dependent and post-binding rules because the GUI's conditional engine consumes `FormState` snapshots that don't natively expose `len(slots)` or `max(@N)+1`. A future cycle will plumb a slot-count signal through `FormState` + extend the `Predicate` AST with `slot_count_op` / `slot_count_min` / etc. variants. Concrete rules to add: §6.6 row 9 (T-in-range vs N), row 10 (single-sig with N > 1), row 11 (multisig with N == 1), row 13 (BIP-388 distinct-key), row 14 (per-`@N` annotation inconsistency).
- **Why deferred:** Out of v1 scope per plan §1.4 — these rules need a dynamic slot count signal not knowable until the form is filled, and surface naturally at Run time via the CLI's typed error. v1 ships argv-level submission + lets the CLI emit the error.
- **Status:** `resolved 0329800` — fully closed 2026-05-16 across three sub-cycles. v2-cycle (`mnemonic-toolkit-v0.17.0`, `4758168`) shipped the **predicate-machinery** half: schema v3 + `SlotCountEq`/`SlotCountGte`/`SlotCountLte` Predicate variants + SPEC §6.10.2 grammar docs (`a26c809`); GUI consumer + drift gate at `mnemonic-gui-v0.6.0` (`9d447d0`). Row 12 closed via the separate `pin_value` Effect (v2 cycle). The remaining row partition closed via two child FOLLOWUPs (both now resolved): `gui-schema-effect-on-dropdown-options-vocab` → resolved `c7ac604` (Batch B-1: `mnemonic-toolkit-v0.18.0` + `mnemonic-gui-v0.7.0` — `disable_options` Effect grammar for rows 10/11 + GUI-internal `NumberMax::FromSlotCount` for row 9); `gui-schema-cross-slot-predicate-projection` → resolved `38ad066` (Batch B-2: `mnemonic-gui-v0.7.1` — row 8 GUI-internal `detect_slot_index_gaps`; rows 13/14 wontfix with CLI-rejection-sufficient rationale). SPEC §6.10.7 mapping table partition reflects the final disposition: rows 8/9 `ENCODED v3 (GUI-internal)`, rows 10/11 `ENCODED v3` (toolkit-emitted), rows 12 `ENCODED v2`, rows 13/14 CLI-rejection-sufficient (wontfix).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-runtime-conditional-projection` (resolved in lockstep).

### `gui-number-widget-unset-sentinel` — Number/Range/Timestamp/TaggedOrIndexed widgets lack a "no value" sentinel

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, plan §7 item 2 + plan §1.4 third bullet. Cross-referenced here for cycle bookkeeping; primary tracking lives in `bg002h/mnemonic-gui` `FOLLOWUPS.md`.
- **Where:** `mnemonic-gui/src/schema/mod.rs:263-268` (`flag_value_is_present` always returns true for Number/Range/Timestamp/TaggedOrIndexed); `mnemonic-gui/src/form/widget.rs:101-126` (`default_flag_value_for` seeds Number widgets to `min` regardless of user interaction).
- **What:** Number/Range/Timestamp/TaggedOrIndexed widgets currently have no "no value" sentinel — once seeded by `default_flag_value_for`, the value is always-present per `flag_value_is_present`. v1 sidesteps this via the §6.10 visibility gate (Hidden + Disabled flags don't emit regardless of widget value). A future cycle may add an explicit unset state for UX clarity (e.g., a "clear" affordance next to numeric widgets so users can explicitly opt out of supplying a numeric flag).
- **Why deferred:** Out of v1 scope per plan §1.4 — the visibility gate makes this unnecessary for the motivating bug. UX-quality improvement, not a correctness gap.
- **Status:** `resolved 84a69b8` — `mnemonic-gui-v0.6.0` P3 (2026-05-16; GUI-only, no toolkit code touched). +`FlagValue::Unset` variant with `#[serde(other)]` for forward-compat; argv assembler treats Unset as absent uniformly. See companion entry in `bg002h/mnemonic-gui` for full closure notes + caveat about serde-other on externally-tagged enums.
- **Tier:** `cross-repo` (gui-impact-only; cross-referenced for cycle completeness)
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-number-widget-unset-sentinel` (primary tracking, to be filed at cycle close).

### `gui-default-form-state-template-aware-seed` — replace static default-state seed with template-aware seed

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, plan §7 item 3. Natural successor to P5 (the v1 cycle's static seed cleanup at `mnemonic-gui/src/main.rs:203`). Cross-referenced here for cycle bookkeeping; primary tracking lives in `bg002h/mnemonic-gui` `FOLLOWUPS.md`.
- **Where:** `mnemonic-gui/src/main.rs:197-211` (default form-state seed; v1's P5 removes the `--multisig-path-family bip87` line but leaves the static structure intact).
- **What:** Replace the static screenshot-mode default seed with a template-aware default. When the user picks a multisig template (e.g., `wsh-sortedmulti`), the form auto-seeds multisig defaults (e.g., `--multisig-path-family bip87`, `--threshold` to a reasonable default); when the user picks single-sig, the form omits those flags entirely.
- **Why deferred:** Out of v1 scope per plan §7 — optional follow-on. The v1 P5 cleanup removes the unconditionally-wrong seed; the template-aware version is a UX enhancement.
- **Status:** `resolved 538dc70` — `mnemonic-gui-v0.6.0` P4 (2026-05-16; GUI-only, no toolkit code touched). `form::conditional::template_defaults_for(template)` returns canonical multisig defaults (`--threshold = 2`, `--multisig-path-family = bip48`) for non-single-sig templates; per-frame egui hook applies them via seed-on-empty discipline. See companion entry in `bg002h/mnemonic-gui` for full closure notes.
- **Tier:** `cross-repo` (gui-impact-only; cross-referenced for cycle completeness)
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-default-form-state-template-aware-seed` (primary tracking, to be filed at cycle close).

### `gui-schema-numeric-flag-value-pin-effect` — add `pin_value` Effect variant for §6.6 row 12 ("--account != 0 when --descriptor present") projection

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, R1 I3 reviewer fold. Plan §2.1 row 12 + §3 manifest rule 7 + §6.10.7 mapping table all marked DEFERRED for this rule.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.3 (Effect vocabulary); `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (Effect enum + serializer); `crates/mnemonic-toolkit/src/cmd/bundle.rs:200-205` (the rule the projection encodes — `DESCRIPTOR_WITH_NONZERO_ACCOUNT`); `mnemonic-gui/src/form/conditional.rs` (consumer — Number widget value-coerce-to-zero handler).
- **What:** Add a `pin_value: { flag, value }` Effect variant to the §6.10.3 vocabulary so the GUI can coerce `--account` to 0 (or any pinned numeric value) when `--descriptor` is present, mirroring SPEC §6.6 row 12's CLI rejection at `bundle.rs:200-205`. Without this, the GUI's Number widget for `--account` defaults to `0` (per `default_flag_value_for`), which is the safe value; the rule only fires when the user actively types a nonzero value, in which case the CLI's byte-exact error suffices for v1.
- **Why deferred:** Out of v1 scope per R1 I3 reviewer fold — the GUI default of 0 makes this rare misuse; the CLI error is informative. Adding a `pin_value` Effect requires SPEC §6.10.3 expansion + GUI Number-widget coercion semantics not warranted by user evidence.
- **Status:** `resolved 4758168` — `mnemonic-toolkit-v0.17.0` P0+P1 (2026-05-16). SPEC §6.10.3 v3 grammar extension: `pin_value` Visibility variant with wire shape `{"pin_value": {"value": V}}` (`a26c809`); §6.10.4 NEW emission table enumerates PinValue's REPLACE-user-value semantic (distinct from Hidden/Disabled which suppress); §6.10.7 row 12 flipped DEFERRED → ENCODED v2; `gui_schema.rs::VisibilityProjection +PinValue` + manual `Serialize` impl preserving v2 bare-string back-compat (Copy dropped); `bundle_conditional_rules` emits the row 12 rule (`76db841`). GUI consumer + custom Deserialize accepting both v2 + v3 wire shapes + `assemble_argv` PinValue emission path at `mnemonic-gui-v0.6.0` (`9d447d0`).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-numeric-flag-value-pin-effect` (to be filed at cycle close).

### `gui-schema-template-groups-meta-field` — emit per-subcommand `meta.template_groups` to retire `SINGLE_SIG_TEMPLATES` const

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, R1 I4 reviewer fold. Plan §7 item 5.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (toolkit side — emit `meta.template_groups: { single_sig: [..], multisig: [..] }` block sourced from `Template::is_multisig()`); `mnemonic-gui/src/form/conditional.rs` (gui side — replace module-level `SINGLE_SIG_TEMPLATES: &[&str] = &["bip44", "bip49", "bip84", "bip86"]` with parse from JSON `meta.template_groups`); `crates/mnemonic-toolkit/src/template.rs:46-56` (`is_multisig()` source-of-truth — unchanged).
- **What:** v1 cycle replicates the single-sig template set client-side as a module-level `SINGLE_SIG_TEMPLATES` const in `conditional.rs`. The drift gate test detects divergence, but a future cleanup cycle can collapse the const by having the toolkit emit `meta.template_groups` in the gui-schema JSON.
- **Why deferred:** Out of v1 scope — the drift gate suffices for parity enforcement. Cleanup-class change.
- **Status:** `resolved 4758168` — `mnemonic-toolkit-v0.17.0` P0+P1 (2026-05-16). SPEC §6.10.8 NEW per-subcommand `meta` block documentation (`a26c809`); `gui_schema.rs::build_subcommand_meta` emits `meta.template_groups: { single_sig, multisig }` sourced from `CliTemplate::is_multisig()` (`76db841`). GUI-side `SINGLE_SIG_TEMPLATES` const promoted `pub(crate) → pub` + new parity test `tests/schema_mirror.rs::single_sig_templates_const_matches_meta_template_groups` (MNEMONIC_BIN-gated) at `mnemonic-gui-v0.6.0` `9d447d0`. Pair-of-checks posture (drift gate for per-rule projection + const-vs-meta for the bulk list) closes without coupling conditional-fn purity to a runtime subprocess fetch. **Defect carried forward**: `build_subcommand_meta` emits the meta block for `derive-child` but derive-child has no `--template` flag (toolkit-side bug surfaced by opus reviewer at cycle close) — tracked at new FOLLOWUP `gui-schema-derive-child-meta-template-groups-spurious`.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-template-groups-meta-field` (to be filed at cycle close).

### `spec-v0_5-missing-v0_3-descriptor-mode-rows` — SPEC §6.6 table missing the v0.3-NEW descriptor-mode rows

- **Surfaced:** 2026-05-16, during the GUI conditional-applicability v1 cycle (P0 SPEC read, pre-write phase). Discovered while reading `design/SPEC_mnemonic_toolkit_v0_5.md` §6.6 (lines 189-217) to draft the §6.10 GUI-projection subsection.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_5.md` §6.6 table (lines 199-213); `crates/mnemonic-toolkit/src/cmd/bundle.rs:124-129` (the comment at line 124 reads "v0.3 NEW rows (SPEC §6.9). Byte-exact.")
- **What:** Four byte-exact error consts at `bundle.rs:124-129` — `DESCRIPTOR_AND_DESCRIPTOR_FILE`, `DESCRIPTOR_WITH_THRESHOLD`, `DESCRIPTOR_WITH_PATH_FAMILY`, `DESCRIPTOR_WITH_NONZERO_ACCOUNT` — were tagged "v0.3 NEW rows" in the source, but the v0.5 SPEC §6.6 table does NOT enumerate them. They are runtime-enforced at `bundle.rs:179-205` and pinned byte-exactly by `tests/cli_mode_violations_v0_5.rs`, so runtime behavior is correct; the gap is purely SPEC documentation drift. §6.9's text "Rows 1-14 in §6.6 plus rows 15-17 in §6.7" undercounts: actual enforced rows include the four missing descriptor-mode rules above. A future SPEC-cleanup cycle should add them to the §6.6 table (e.g., as rows 12.1/12.2/12.3/12.4 between the existing row 12 descriptor-threshold conflict and row 13 BIP-388 distinct-key) with cross-citations to `bundle.rs::mode_text::*`.
- **Why deferred:** Surfaced mid-cycle on an unrelated patch (GUI conditional-applicability v1 added a NEW §6.10 next to §6.6 and deliberately did NOT modify §6.6 itself to keep this drift fix decoupled from the cycle's scope). Independent SPEC-only fix.
- **Status:** `open`
- **Tier:** `v1+`

### `gui-run-confirm-modal-secret-redaction-manual-companion` — manual-prose lockstep companion to GUI run-confirm-modal redaction fix

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle M-P2.4 batch 4 R0 source-grep. The Defense-2 prose in `docs/manual-gui/src/10-foundations/14-secret-handling.md` (LOCKed in M-P2.4 batch 2) and the feature-2 description in `11-what-is-mnemonic-gui.md` both claim the run-confirm modal "shows the assembled argv with secret values replaced by `***`". `mnemonic-gui/src/main.rs:512-535` shows no such redaction; the modal renders each argv token verbatim. The manual prose was patched in the M-P2.4 batch-4 commit to honestly describe the actual (undesired) behavior + recommend cold-node-only operation as an operational mitigation.
- **Where:** `docs/manual-gui/src/10-foundations/14-secret-handling.md` Defense-2 section; `docs/manual-gui/src/10-foundations/11-what-is-mnemonic-gui.md` feature-2 description; `docs/manual-gui/pinned-upstream.toml` (currently pinned to `mnemonic-gui-v0.3.0`, must bump to whatever GUI tag ships the redaction fix).
- **What:** When the GUI ships the redaction fix (tracked at sibling `bg002h/mnemonic-gui` `FOLLOWUPS.md` `gui-run-confirm-modal-secret-redaction`), this manual must (i) revert the v1.0 honest-broken framing in chapters 11 + 14, (ii) restore the `***` redaction claim, (iii) drop the cold-node-only operational warning to a hover-tooltip-grade general-hygiene remark (still useful but no longer load-bearing for the security model), and (iv) bump `pinned-upstream.toml` to the GUI tag that ships the fix.
- **Why deferred:** Surfaced AFTER M-P2.4 batches 1-2 LOCKed; the manual cannot fix the GUI behavior, only describe it. v1.0 manual ships with honest-broken framing + cold-node operational mitigation; v1.1 will close the loop in lockstep with the GUI fix.
- **Status:** `open`
- **Tier:** `v1.1+`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-run-confirm-modal-secret-redaction`.

### `gui-manual-cross-refs-to-cli-manual` — bidirectional links between docs/manual-gui/ and docs/manual/ chapters where concepts overlap

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per the v1.0 cycle plan (in-flight; to be archived at PE close to `design/PLAN_manual_gui_v1.md`) §2.7.
- **Where:** docs/manual-gui/src/ chapters that document concepts also covered in docs/manual/ (foundations, glossary terms, secret-bearing advisories, BIP-39 / BIP-32 primers). Currently no cross-links exist; the manuals are independent build units.
- **What:** v1.1 enhancement (Option C in the v1 planning §1.1). Add bidirectional `[CLI manual: §X.Y](https://...manual.pdf#section)` links between the GUI manual's foundations chapter and the CLI manual's equivalent chapters. Both PDFs hosted; deep-links use stable kebab-case anchors. Concept-overlap zones: bundle/card/slot terminology, BIP-39 entropy primer, BIP-32 derivation primer, codex32 / BCH primers, m-format constellation glossary.
- **Why deferred:** v1.0 cycle scope-locked to Option B (~165 pages, GUI-shaped only). Cross-references introduce bidirectional drift hazard because the CLI manual's chapters are NOT structured around the GUI's IA. Wait until both manuals are in steady state.
- **Status:** `open`
- **Tier:** `v1.1+`

### `cli-manual-html-target` — add HTML target + gh-pages deploy for the CLI manual

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per the v1.0 cycle plan (in-flight; to be archived at PE close to `design/PLAN_manual_gui_v1.md`) §2.7.
- **Where:** docs/manual/Makefile (currently has `md` and `pdf` targets only; no `html`). docs/manual/Dockerfile.build (would need no changes — pandoc + xelatex already installed). .github/workflows/manual.yml (would need a gh-pages deploy job mirroring docs/manual-gui/.github/workflows/manual-gui.yml's pattern).
- **What:** v1.1 enhancement. Add `make html` target producing `build/m-format-manual.html`, plus a gh-pages deploy step in `manual.yml` triggered on `manual-v*` tags. Output lands at `https://bg002h.github.io/mnemonic-toolkit/manual/` (parallel to the GUI manual at `/manual-gui/`). Enables in-app help-icon deep-linking for any future CLI-output-driven tooling (e.g., a `mnemonic --help <subcommand>` that emits a manual URL alongside the help text).
- **Why deferred:** CLI manual is hand-curated and stable at v0.1; no current user-facing demand for HTML hosting. The GUI manual v1.0 cycle is the first time gh-pages infrastructure lands in this repo; CLI-manual HTML can piggyback once that's proven.
- **Status:** `open`
- **Tier:** `v1.1+`

### `gui-manual-localization` — non-English content support for the GUI manual

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per the v1.0 cycle plan (in-flight; to be archived at PE close to `design/PLAN_manual_gui_v1.md`) §2.7.
- **Where:** docs/manual-gui/src/. Pandoc supports multi-language documents via the `lang` metadata field + Babel/Polyglossia LaTeX packages.
- **What:** Translate the GUI manual into at least one additional language (likely Spanish, given the existing BIP-39 wordlist support for `spanish`). Add a language-selector to the GUI's help-icon URL helper to deep-link to localized anchors. Requires a translation infrastructure: per-language `src/` trees + a build matrix in CI + native-speaker review.
- **Why deferred:** v1.0 ships English-only. Localization is a substantial undertaking (translation cost + per-language QA cost + ongoing drift maintenance). Defer until the manual stabilizes AND there's specific demand from a user community.
- **Status:** `open`
- **Tier:** `v2+`

### `library-error-and-language-surface-promotion` — move `error` + `language` + `friendly` modules from main.rs to lib.rs

- **Surfaced:** 2026-05-13, v0.11.0 Phase 1 R1 reviewer-loop. The P1 GREEN impl pivoted to a self-contained library surface (`FinalWordLanguage` + `FinalWordError` library-local enums) because exposing `error`/`language`/`friendly` from lib.rs today would require moving them out of `src/main.rs`'s private-module set — a cross-module refactor touching every binary file that imports `ToolkitError`. R1 reviewer endorsed the P1 pivot but recommended filing this FOLLOWUP for the future cleaner refactor.
- **Where:** Move `crates/mnemonic-toolkit/src/{error,language,friendly}.rs` from main.rs-private to lib.rs-public. Audit every `crate::error::*` / `crate::language::*` / `crate::friendly::*` import in the binary tree and re-route to `mnemonic_toolkit::error::*` / `mnemonic_toolkit::language::*` / `mnemonic_toolkit::friendly::*`. Delete `FinalWordLanguage` + `FinalWordError` library-local types and route `final_word_candidates` through `CliLanguage` + `ToolkitError` directly.
- **What:** Future cleaner crate-shape. Avoids the per-feature pattern of "library-local mirror enums" that v0.11.0 final-word and any future feature would need. Lowers boilerplate on every CLI-boundary wrapper.
- **Why deferred:** Out-of-scope for v0.11.0 — this is a crate-shape refactor that affects every binary module, not a feature-localized change. The duplication cost in v0.11.0 (10 BIP-39 language variants + 2 error variants) is bounded and trivially stable; the refactor cost is the inverse. Defer to a focused crate-shape cycle.
- **Status:** `open`
- **Tier:** `v1+`-refactor (no user-facing impact; pure crate-hygiene)

### `bip39-final-word-completer` — `mnemonic final-word` subcommand (v0.11.0)

- **Surfaced:** 2026-05-13, post-v0.10.1 user feature-request. New feature, not a deferral from a prior cycle. Plan + brainstorm at `~/.claude/plans/radiant-seeking-teacup.md`; SPEC at `design/SPEC_final_word_v0_11_0.md`.
- **Where:** New module `crates/mnemonic-toolkit/src/final_word.rs` (lib surface) + new `crates/mnemonic-toolkit/src/cmd/final_word.rs` (CLI surface) + new `Command` variant in `src/main.rs`. Library entry: `pub fn final_word_candidates(partial_phrase: &str, language: FinalWordLanguage) -> Result<Vec<&'static str>, FinalWordError>` (P1 pivoted to library-local types per FOLLOWUP `library-error-and-language-surface-promotion`). CLI: `mnemonic final-word --from phrase=<N-1-words-or-> [--language <L>] [--json-out <path>]` (single stdin route via `phrase=-` per R0 round 1 C1; no paired `--phrase-stdin` flag).
- **What:** Given an incomplete BIP-39 mnemonic of length N-1 (N ∈ {12, 15, 18, 21, 24}) and a language, emit the complete set of wordlist entries that, when appended as the Nth word, yield a phrase with a valid BIP-39 checksum. Set size is deterministic: 2^(11 − CS) ∈ {128, 64, 32, 16, 8}. Use cases: paper-backup recovery (smudged last word), manual entropy generation (dice/coin → N-1 words → checksum-fixing Nth word), verification (does my last word match what the checksum implies?). Algorithm: naïve enumeration over the 2048-entry wordlist with `bip39::Mnemonic::parse_in` as the correctness oracle.
- **Status:** `resolved f6c036a` — `mnemonic-toolkit-v0.11.0` tag pushed 2026-05-14. As-shipped: P0 SPEC `design/SPEC_final_word_v0_11_0.md` (R0 LOCK across 3 rounds at `design/agent-reports/v0_11_0-final-word-spec-r0.md`); P1 library `mnemonic_toolkit::final_word` with self-contained `FinalWordLanguage` + `FinalWordError` (R1 LOCK clean round 1, `design/agent-reports/v0_11_0-final-word-lib-r1.md`); P2 CLI handler `cmd/final_word.rs` with Cycle A `Zeroizing<String>` + `secret_in_argv_warning` + Cycle B `pin_pages_for` (R1 LOCK clean round 1, `design/agent-reports/v0_11_0-final-word-cli-r1.md`); 37 CLI tests + 17 lib tests (all green); 2 SHA-pinned JSON envelope anchors; P3 manual chapter + cli-subcommands.list mirror (R1 LOCK clean round 1, `design/agent-reports/v0_11_0-final-word-manual-r1.md`). Filed companion FOLLOWUP `library-error-and-language-surface-promotion` for the future crate-shape cleanup that would unify the library-local types with `CliLanguage` + `ToolkitError`.
- **Tier:** `v0.11.0-feature`.
- **Companion:** N/A (toolkit-only; no cross-repo work — ms-cli has no candidate insertion point per Phase 1 exploration in the plan).

### `seed-xor-coldcard-compat` — `mnemonic seed-xor` Coldcard-compatible BIP-39 ↔ BIP-39 all-or-nothing splitter (v0.12.0)

- **Surfaced:** 2026-05-14, post-v0.11.0 user feature-request. Filed at P0 alongside [[slip39-shamir-secret-sharing]] as the two-cycle pair planned in `~/.claude/plans/radiant-seeking-teacup.md`.
- **Where:** New module `crates/mnemonic-toolkit/src/seed_xor.rs` (lib surface; library-local `SeedXorError`) + new `crates/mnemonic-toolkit/src/cmd/seed_xor.rs` (CLI surface; `Split` + `Combine` sub-subcommands) + new `Command::SeedXor` variant in `src/main.rs`. Library entry: `pub fn seed_xor_split(entropy: &[u8], n_shares: usize, rng: &mut (impl rand_core::CryptoRng + rand_core::RngCore)) -> Result<Vec<Zeroizing<Vec<u8>>>, SeedXorError>` + paired deterministic + combine functions. CLI: `mnemonic seed-xor split --from phrase=<v-or-> --shares N [--language LANG] [--deterministic-from-master] [--json-out PATH]` and `mnemonic seed-xor combine --share phrase=<v-or-> ... --shares N [--language LANG] [--json-out PATH]`.
- **What:** Given a single BIP-39 entropy (12/15/18/21/24 words), split into N BIP-39 phrases such that bytewise XOR of all N entropies reconstitutes the master. Per-share BIP-39 checksum is recomputed so each share is itself a parseable, structurally-valid BIP-39 phrase. ALL-OR-NOTHING (not a threshold scheme — for K-of-N use SLIP-39 via [[slip39-shamir-secret-sharing]]). Coldcard-compatible at 12/18/24-word sizes (per `shared/xor_seed.py:assert len(raw_secret) in (16, 24, 32)`); 15/21 are toolkit-only extensions that Coldcard hardware cannot round-trip. No MAC; substitution of a wrong-but-valid-BIP-39 share is mathematically undetectable (per §A.2.6 advisory text). NEW advisory class introduced: multi-secret-on-stdout (K-of-N share emit pattern, first toolkit use).
- **Status:** `resolved 63b4503` — `mnemonic-toolkit-v0.12.0` tag pushed 2026-05-14. As-shipped: P0 SPEC `design/SPEC_seed_xor_v0_12_0.md` (R0 LOCK clean round 1, `design/agent-reports/v0_12_0-seed-xor-spec-r0.md`); P1 library `mnemonic_toolkit::seed_xor` with library-local `SeedXorError` + `seed_xor_split` / `seed_xor_split_deterministic` / `seed_xor_combine` (R1 LOCK clean round 1, `design/agent-reports/v0_12_0-seed-xor-lib-r1.md`; 17 tests + 2000 round-trip property-test pairs + Coldcard byte-pin anchor); P2 CLI handler `cmd/seed_xor.rs` with `split` + `combine` sub-subcommands wired through Cycle A/B discipline (R1 LOCK clean round 1, `design/agent-reports/v0_12_0-seed-xor-cli-r1.md`; 44 CLI tests; 2 SHA-pinned JSON envelope anchors); P3 manual chapter + cli-subcommands list (R1 LOCK clean round 1, `design/agent-reports/v0_12_0-seed-xor-manual-r1.md`). New advisory class introduced (multi-secret-on-stdout for K-of-N share emit) ready for SLIP-39 v0.13.0 to parameterize.
- **Tier:** `v0.12.0-feature`.
- **Companion:** [[slip39-shamir-secret-sharing]] (the v0.13.0 cycle's larger K-of-N counterpart; two-cycle plan ships v0.12.0 first to validate the new advisory class then v0.13.0 to extend it parameterized).

### `seed-xor-coldcard-doc-test-vectors` — vendor Coldcard `docs/seed-xor.md` test vectors

- **Surfaced:** 2026-05-14, post-v0.12.0 user request. v0.12.0 P1 already pins one Coldcard byte-pin anchor (`abandon × 12` deterministic share[0] in `tests/lib_seed_xor.rs::deterministic_split_abandon_12_share_0_byte_pin`) and the algorithm is byte-correct against `shared/xor_seed.py` (verified at P1 R1 LOCK). The Coldcard *documentation* additionally publishes two worked examples that demonstrate the algorithm by hand at the BIP-39-phrase level — these are valuable as end-to-end regression anchors at the **CLI surface** layer (not just the lib byte-XOR layer), since they exercise the full "BIP-39 phrase → entropy bytes → per-share recompute → BIP-39 phrase" round-trip with non-trivial master entropy (not the all-zeros `abandon × N` degenerate).
- **Where:** `crates/mnemonic-toolkit/tests/cli_seed_xor_happy_paths.rs` — add `coldcard_doc_24_word_vector` + `coldcard_doc_12_word_vector` tests that invoke `mnemonic seed-xor combine --share <3-phrases> --shares 3` and assert byte-equality with the documented master. NO `split` half — the doc doesn't claim Coldcard's RNG-generated shares are derivable from the master (they're random) so we can only round-trip the combine direction.
- **What:** Two worked-example vectors from <https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md>:

  **Vector 1 (24-word, N=3):**
  - Master: `silent toe meat possible chair blossom wait occur this worth option bag nurse find fish scene bench asthma bike wage world quit primary indoor`
  - Share A: `romance wink lottery autumn shop bring dawn tongue range crater truth ability miss spice fitness easy legal release recall obey exchange recycle dragon room`
  - Share B: `lion misery divide hurry latin fluid camp advance illegal lab pyramid unaware eager fringe sick camera series noodle toy crowd jeans select depth lounge`
  - Share C: `vault nominee cradle silk own frown throw leg cactus recall talent worry gadget surface shy planet purpose coffee drip few seven term squeeze educate`

  **Vector 2 (12-word, N=3):**
  - Master: `cannon opinion leader nephew found yard metal galaxy crouch between real trade`
  - Share A: `romance wink lottery autumn shop bring dawn tongue range crater truth ability`
  - Share B: `boat unfair shell violin tree robust open ride visual forest vintage approve`
  - Share C: `lion misery divide hurry latin fluid camp advance illegal lab pyramid unhappy`

  Coldcard's docs caveat that the examples are "illustrative" rather than formally normative test vectors, but the BIP-39 phrases + the XOR algorithm + per-share checksum recompute uniquely determine the round-trip — they're verifiable.
- **Why deferred:** v0.12.0 already ships a Coldcard byte-pin anchor on the deterministic-split share[0] computation (covers `Batshitoshi` prefix + SHA256d + slice-width regression). The doc-vectors are additive: end-to-end CLI round-trip evidence with non-trivial entropy. Skipped in v0.12.0 to keep P1/P2 scope tight; the patch cycle is free of any other open seed-xor work so it's a clean isolated bump.
- **Status:** `open` (filed at v0.12.0 PE+1; closes at v0.12.1 PE).
- **Tier:** `v0.12.1-patch`.
- **Companion:** N/A (toolkit-only; doc-only upstream — no sibling-repo coordination).

### `resolved-slot-entropy-zeroizing-field` — change `ResolvedSlot.entropy` to `Option<Zeroizing<Vec<u8>>>`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 GREEN (deferred from in-cycle landing due to 19-site cascade).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:582` — `pub entropy: Option<Vec<u8>>` field on the `ResolvedSlot` (private) struct. 19 read/write sites cascade through bundle.rs, verify_bundle.rs, parse_descriptor.rs (incl. test mods).
- **What:** Per plan §"Phase 2 — Impl" step 4, ResolvedSlot.entropy was scheduled to become `Option<Zeroizing<Vec<u8>>>` so the field-resident entropy scrubs on drop. Phase 2 GREEN landed local-wrap discipline at every producer + consumer site (entropy entering the field is `Zeroizing` at construction; reads clone to a local `Zeroizing`) but left the field type as `Option<Vec<u8>>` — so the field-resident copy itself is unwrapped during its lifetime.
- **Why deferred:** 19-site cascade across 3 files + test mods is mechanically large and not representative of the per-row wrap discipline the Phase 2 zeroize-lint is enforcing. The local-wrap discipline at producer + consumer sites covers the value's transit; only the brief field-resident lifetime is unwrapped. A separate small commit can complete the field type change in one shot.
- **Status:** `superseded by resolved-slot-derived-account-zeroizing-field` (2026-05-13, Phase 3a R0 v3-fold RESCOPE per `~/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`). The R0 v2 LOCK that bundled this migration with the Cycle B Phase 3a `_entropy_pin` apply work was reviewed and re-scoped to Path B-lite, which carves out the field-type migration to a focused v0.10.1 patch. The new entry [[resolved-slot-derived-account-zeroizing-field]] takes broader scope (covers both `ResolvedSlot.entropy` AND `DerivedAccount.entropy: Vec<u8>` → `Zeroizing<Vec<u8>>`, plus `impl Drop for DerivedAccount` deletion, plus `into_parts` body change, plus the lint anchor relabel + new row, plus CHANGELOG entry).
- **Tier:** `v0.10.1-patch` (escalated from `v0.9.2-nice-to-have`; broader scope under the superseding entry)

### `resolved-slot-derived-account-zeroizing-field` — migrate Cycle-A `Vec<u8>` entropy fields to `Zeroizing<Vec<u8>>` (supersedes [[resolved-slot-entropy-zeroizing-field]])

- **Surfaced:** 2026-05-13, Cycle B Phase 3a R0 v3-fold RESCOPE (Path B-lite). Carved out from the R0 v2 LOCK (commit `9be0f0f`) which had bundled this migration with the `_entropy_pin` apply work; the rescope keeps the pin work in Phase 3a (toolkit `v0.10.0`) and defers the field-type migration to a focused v0.10.1 patch.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:585` (`pub entropy: Option<Vec<u8>>`); `crates/mnemonic-toolkit/src/derive.rs:21,49` (`pub entropy: Vec<u8>` + `impl Drop for DerivedAccount`); `crates/mnemonic-toolkit/src/derive.rs:37` (`into_parts()` body); `crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs` (row labels at lines 14-16, 109-113); `CHANGELOG.md`.
- **What** (7 deliverables, all in one v0.10.1 patch):
  1. `ResolvedSlot.entropy: Option<Vec<u8>>` → `Option<Zeroizing<Vec<u8>>>` at `synthesize.rs:585`. Cascade: 6 ctor sites (`cmd/bundle.rs:{348,449,1065}` real-pin sites + `:417,491` watch-only None sites + `synthesize.rs:1184` test) wrap construction in `Zeroizing::new(...)`. ~6 read-site edits become Deref-through-Zeroizing.
  2. `DerivedAccount.entropy: Vec<u8>` → `Zeroizing<Vec<u8>>` at `derive.rs:21`. 1 ctor site (`derive_slot.rs:77`) wraps in `Zeroizing::new(...)`.
  3. DELETE `impl Drop for DerivedAccount` at `derive.rs:49-58` (Zeroizing's Drop carries the scrub responsibility; `pub-struct-drop-semver-risk-monitor` FOLLOWUP exit also closes here).
  4. `into_parts()` body change at `derive.rs:37`: `mem::take(&mut self.entropy)` → `mem::take(&mut *self.entropy)` (Deref through Zeroizing). Outward signature returning `Vec<u8>` is preserved.
  5. `tests/lint_zeroize_discipline.rs` row "DerivedAccount impl Drop scrubs entropy on drop" relabeled to "DerivedAccount entropy field is `Zeroizing<Vec<u8>>`" with new evidence; lint lines 109-113 deferred-FOLLOWUP comment block DELETED; new row "ResolvedSlot entropy field is `Option<Zeroizing<Vec<u8>>>`" with evidence `pub entropy: Option<Zeroizing<Vec<u8>>>` against `src/synthesize.rs`.
  6. `CHANGELOG.md` v0.10.1 entry: "Field-type migration: `ResolvedSlot.entropy` and `DerivedAccount.entropy` to `Zeroizing<Vec<u8>>`; deletes `impl Drop for DerivedAccount` (Zeroizing carries scrub). Closes deferred FOLLOWUP `resolved-slot-entropy-zeroizing-field`. Closes monitoring FOLLOWUP `pub-struct-drop-semver-risk-monitor`."
  7. R1 Opus review per `feedback_opus_primary_review_agent`; report at `design/agent-reports/v0_10_1-zeroizing-field-migration-r1.md`.
- **Why deferred (separated from Cycle B Phase 3a):** Carving the field-type migration out of Phase 3a removes audit-trail entanglement (Cycle A's `lint_zeroize_discipline.rs` stays untouched in Cycle B), eliminates the Arc-wrap design from being conflated with the Zeroizing migration in reviewer eyes, and lets v0.10.1 ship as a clean focused patch with no concurrent mlock work. The cascade cost is roughly equal whether bundled or split (each ctor site takes 1 edit per landing; combined = 14 edits in one PR; split = 7 + 7 edits across two PRs). The structural-discipline gap (human-maintained `impl Drop` scrub vs. structurally-guaranteed Zeroizing field-type) stays at Cycle-A levels through Cycle B; v0.10.1 closes it.
- **Status:** `resolved ed5a1d9` — `mnemonic-toolkit-v0.10.1` tag pushed 2026-05-13. As-shipped scope: 12 ctor sites wrapped (6 direct `ResolvedSlot {` + 6 via `pub type CosignerKeyInfo = ResolvedSlot;` alias trap, caught by R0 round 1; the inline body enumeration above is the pre-R0-grep snapshot and intentionally not retroactively refreshed). 7 explicit read-site fixes (`Option::as_deref` single-step Deref mismatch, `e.clone()` over Zeroizing-ref, double-wrap break, PartialEq mismatch, =-assignment re-wrap; plus 2 sites covered by ctor-local rebind). `impl Drop for DerivedAccount` deleted (Zeroizing-drives-scrub structural guarantee replaces it). Companion FOLLOWUP `pub-struct-drop-semver-risk-monitor` also resolved (DerivedAccount-specific watch — closure follows from the Drop deletion). Plan at `~/.claude/plans/v0_10_1-zeroizing-field-migration.md` (R0 LOCK across 3 rounds); R1 impl-review CLEAR at `design/agent-reports/v0_10_1-zeroizing-field-migration-r1.md`. 620 tests green; clippy clean; miri clean.
- **Tier:** `v0.10.1-patch`.
- **Companion:** N/A (toolkit-only patch — no cross-repo work; ms-cli `v0.3.0` ships in Cycle B PE without coordination).

### `rust-secp256k1-secretkey-zeroize-upstream` — `secp256k1::SecretKey` has no Drop+Zeroize

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 R1 (Opus, finding I-2). The lint at `lint_safety_third_party_blocked.rs` now scans for `SecretKey::from_slice` patterns in addition to the original Mnemonic/Xpriv anchors.
- **Where:** Upstream crate `bitcoin = "0.32"` (transitive `secp256k1`). Affects every `SecretKey::from_slice` construction in `crates/mnemonic-toolkit/src/{bip85,parse_descriptor,cmd/convert}.rs` — 5 production call sites. Each carries a `SAFETY: third-party-blocked` doc-comment pointing at this FOLLOWUP.
- **What:** `secp256k1::SecretKey` is stack-bound, provides `non_secure_erase()` (which is best-effort and compiler-defeatable, per the upstream's own doc) but does NOT implement Drop with Zeroize. The toolkit's mitigation is lifetime minimization + SAFETY-anchored doc-comments at the construction sites; the residual gap is that the 32-byte scalar lives in stack memory until function exit unscrubbed. Closes when upstream `rust-secp256k1` ships a Drop+Zeroize impl for SecretKey (or when the toolkit migrates to a different curve library that does).
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `rust-bip39-mnemonic-zeroize-upstream` — `bip39::Mnemonic` has no Drop+Zeroize

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 GREEN — surfaced while landing the `SAFETY: third-party-blocked` doc-comment discipline at every `Mnemonic::parse_in` / `Mnemonic::from_entropy_in` call site in this repo.
- **Where:** Upstream crate `bip39 = "2"`. Affects every `Mnemonic` construction in `crates/mnemonic-toolkit/src/{bip85,derive,derive_slot,synthesize,parse_descriptor,cmd/{bundle,convert,derive_child}}.rs` — 25 production call sites enumerated by `lint_safety_third_party_blocked.rs::SCAN_FILES`. Each site carries a `SAFETY: third-party-blocked` doc-comment pointing at this FOLLOWUP.
- **What:** `bip39::Mnemonic` holds the phrase + internal entropy buffer but does not implement `Drop` with `Zeroize::zeroize`. The toolkit's mitigation is lifetime minimization (construct → `to_entropy()` / `to_seed()` into `Zeroizing` → immediate drop), but a residual gap remains: the secret bytes inside `Mnemonic` are not actively scrubbed before deallocation. Closes when upstream `bip39` adds `impl Drop` + `zeroize` dep.
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `rust-bitcoin-xpriv-zeroize-upstream` — `bitcoin::bip32::Xpriv` is Copy + no Drop + no Zeroize

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 GREEN — surfaced while landing the `SAFETY: third-party-blocked` doc-comment discipline at every `Xpriv::new_master` / `Xpriv::derive_priv` call site in this repo.
- **Where:** Upstream crate `bitcoin = "0.32"`. Affects every `Xpriv` construction in `crates/mnemonic-toolkit/src/{bip85,derive_slot,synthesize,parse_descriptor,cmd/{bundle,convert,derive_child}}.rs` — also enumerated by `lint_safety_third_party_blocked.rs`.
- **What:** `bitcoin::bip32::Xpriv` is `Copy` and has no Drop hook upstream. Phase 0 R3 C-R3-3 verified that `drop(xpriv)` on a `Copy` type is a no-op for memory cleanup (the value bitwise-copies into `drop()` and the original binding remains untouched; every `derive_priv` call leaves a fresh stack copy). Closes when upstream `bitcoin` removes `Copy` from `Xpriv` + adds `impl Drop` + `zeroize` dep — a coordinated breaking change requiring downstream migration at every `Xpriv` call site.
- **Status:** `open` (upstream-blocked; non-trivial breaking change for upstream)
- **Tier:** `external`

### `convert-minikey-stdout-redaction` — widen `NodeType::is_secret_bearing` to cover Casascius MiniKey on the stdout-redaction pathway

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 1 R1 review (Opus 4.7, finding N-2 partial — surfaced while folding the wider-tag method lift onto `NodeType`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` — `NodeType::is_secret_bearing` (around `convert.rs:85-96`) excludes `MiniKey` from the existing redaction + secret-on-stdout pathways at `convert.rs:769` (`from_value` redaction in `--from <secret>=` echo) and `convert.rs:796` (`secret-on-stdout` warning). Phase 1 added a wider `NodeType::is_argv_secret_bearing` method (around `convert.rs:98-110`) that DOES include MiniKey for argv-leakage advisory purposes; the narrower predicate is preserved to avoid expanding Phase 1's scope.
- **What:** MiniKey (Casascius mini-key — a private-key encoding) is a private-key carrier per survey §5 row "convert --from minikey=" but is currently NOT redacted in the `from_value` echo path and does NOT fire the `secret-on-stdout` warning on convert edges that emit a MiniKey value to stdout. Tightening: either widen `is_secret_bearing` to include MiniKey, or change the two call sites to use the wider `is_argv_secret_bearing` predicate. Either approach is small and additive.
- **Why deferred:** Phase 1 scope (argv-leakage closure) ships in lockstep with SPEC v0.9.0 §1 item 1; widening the existing secret-on-stdout warning is a separate user-facing behavior change that would entrain additional fixture updates in `tests/cli_convert_minikey.rs` (currently no advisory is expected) and warrants its own SPEC/disposition pass.
- **Status:** `open`
- **Tier:** `v0.9.1-nice-to-have` (small mechanical fix; can ship in a Phase E cycle-close patch or in Cycle B planning).

### `argv-overwrite-after-parse` — rewrite `argv[]` post-clap to clear secret bytes from `/proc/$PID/cmdline`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-2` (/proc/self/cmdline post-parse overwrite class).
- **Where:** Hypothetical new module `crates/mnemonic-toolkit/src/argv_overwrite.rs` (does not yet exist). Touches every binary entry-point (`mnemonic`, `md`, `mk`, `ms`) to invoke the overwrite shim immediately after `clap::Parser::parse()` returns. The kernel-owned mirror lives in `/proc/$PID/cmdline`; on Linux a raw FFI write into the original `argv[][i]` byte ranges (via `libc::__progname`-adjacent pointer arithmetic, or the `set_proctitle`-style trick) is the only path that actually mutates the in-kernel copy.
- **What:** Phase 1 added a stderr advisory whenever a secret is detected on argv but did NOT mutate argv. The residual gap: an attacker reading `/proc/$PID/cmdline` (same-UID; or any UID without `PR_SET_DUMPABLE=0`) sees the secret bytes for the lifetime of the process. Real fix is to (a) zero-overwrite the in-place argv slots immediately after clap consumes them, OR (b) call `prctl(PR_SET_DUMPABLE, 0)` to deny `/proc/$PID/cmdline` reads to other UIDs (narrower mitigation — does not protect same-UID reads or core dumps). Both are FFI-heavy and platform-specific.
- **Why deferred:** Phase 1's `--*-stdin` paired-flag + `=-` route closes argv-leakage for documented usage; the residual covers users who ignore the warning. SPEC §3 explicitly defers this to a future cycle pending the raw-FFI route.
- **Status:** `open`
- **Tier:** `v1+`

### `clap-argv-pre-parse-residue` — libc `OsString` heap copies of `argv[]` live un-scrubbed before clap parses

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-libc-osstring` class.
- **Where:** `std::env::args_os() -> Vec<OsString>` materialization, called by Rust startup before user code runs. The toolkit cannot intercept earlier than `main()` without replacing libc rt0 (or per-invocation raw-FFI argv intercept). Mirrors the `mnemonic-gui/secret_widget.rs` doc-comment caveat for cross-repo parity.
- **What:** Phase 2's `std::mem::take(&mut args.phrase)` + `Zeroizing::new(...)` scrubs the clap-created `String` allocation but cannot reach the prior `OsString` heap allocation that libc materialized BEFORE clap parsed. Those `OsString` buffers drop un-scrubbed. Addressable only by libc replacement (e.g., musl + custom rt0) or per-invocation raw-FFI argv intercept before clap.
- **Why deferred:** Outside the toolkit's reach (kernel/libc layer). Mirrors mnemonic-gui caveat for parity.
- **Status:** `open`
- **Tier:** `v1+`

### `allocator-pool-residue` — `Zeroizing<Vec<u8>>` drop-time scrub may be defeated by custom-allocator page retention

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-allocator-residue` class.
- **Where:** Allocator-layer concern — applies to every `Zeroizing<Vec<u8>>` / `Zeroizing<String>` / `Zeroizing<Box<[u8]>>` allocation in the workspace when the binary is built against a non-default allocator (e.g., `jemallocator` with cache retention, `mimalloc` with retention, `tcmalloc`). The Cycle A test environment uses the system allocator (glibc malloc); custom allocators are NOT in scope.
- **What:** When a `Zeroizing<Vec<u8>>` drops, the bytes are zeroed in-place by `zeroize::Zeroize` BEFORE the deallocation call returns the page to the allocator. With the system allocator this is sound — the zeroed pages are returned to the OS or to a free-list with the zeros intact. With a retention-class custom allocator (jemalloc with `lg_dirty_mult=-1` or `dirty_decay_ms=-1`, mimalloc with retain pools, etc.), the allocator may *re-zero* or *re-use* the page for an unrelated allocation in ways that briefly expose the secret bytes to in-process readers. Mitigation requires a secret-class-aware page management discipline (custom allocator hook) or a dedicated mmap'd secret arena ([[dedicated-secret-arena]]).
- **Why deferred:** Defense-in-depth class; system allocator (default for `mnemonic`, `md`, `mk`, `ms` binaries) is sound. Custom allocators are an opt-in build configuration not in Cycle A scope.
- **Status:** `open`
- **Tier:** `v1+`

### `pub-struct-drop-semver-risk-monitor` — `impl Drop` on `DerivedAccount` breaks move-out destructure for external library users

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-pub-struct-drop` class.
- **Where:** `crates/mnemonic-toolkit/src/derive.rs:14-66` — `pub struct DerivedAccount` with `impl Drop` (Phase 2 prereq landing at commit `16f64d7`). Rust E0509 forbids move-out destructuring (`let DerivedAccount { entropy, .. } = derived;` and `let entropy = derived.entropy;`) when the enclosing struct has `impl Drop`. Field-borrow access (`&derived.entropy`) and Deref-via-method-call (`derived.entropy.as_slice()`) remain compatible.
- **What:** Cycle A chose `impl Drop` over changing the field type to `Zeroizing<Vec<u8>>` because the latter would force a v0.10.0 minor bump per the pre-1.0 SemVer convention. The trade-off is that any external library user with a move-out destructure pattern on `DerivedAccount` will get an E0509 compile error post-fold. We treat this as patch-tag-compatible (move-out is uncommon in external use; field-borrow is the typical access pattern). Monitor: if downstream library users surface complaints, the cycle re-tags retroactively as a minor bump (v0.10.0).
- **Why deferred:** The fold itself shipped in Phase 2; this entry is the monitoring artifact for the residual semver risk.
- **Status:** `resolved ed5a1d9` — `mnemonic-toolkit-v0.10.1` tag pushed 2026-05-13. `impl Drop for DerivedAccount` deleted in v0.10.1 as part of the Cycle B Path B-lite carve-out completion ([[resolved-slot-derived-account-zeroizing-field]]). Move-out destructuring (`let DerivedAccount { entropy, .. } = derived;` and `let entropy = derived.entropy;`) is once again E0509-free. The watched semver-risk concern is removed; no retroactive minor-bump retag needed.
- **Tier:** `v1+`

### `dedicated-secret-arena` — mmap-allocated page-aligned secret arena for secret-class allocations

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-secret-arena` class.
- **Where:** Hypothetical new module pattern — would replace the default heap allocator for `Zeroizing`-wrapped allocations with a dedicated mmap'd region (à la rust `secrecy` / `secure_string` crates). Touches the GlobalAlloc surface or requires a typed allocator parameter on `Zeroizing<T, A>`-style wrappers.
- **What:** `mlock(2)` pins pages, not bytes; future maintainers may need a dedicated mmap-allocated secret arena for page-aligned heap placement of secret-class allocations. This avoids both the page-vs-byte granularity trap (Cycle B mlock will hit this) and the allocator-pool residue ([[allocator-pool-residue]] becomes addressable via a dedicated allocator path). Out of scope for Cycles A and B both — this is the third-pass design class.
- **Why deferred:** Design class — Cycle A is "first-pass at OWNED-buffer hygiene"; Cycle B is mlock; arena is the natural third cycle.
- **Status:** `open`
- **Tier:** `v1+`

### `sha3-shake256-zeroize-upstream` — `sha3::Shake256` XOF reader state has no `Zeroize`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 carry-over from survey §3.
- **Where:** Upstream crate `sha3 = "0.10"`. Affects `crates/mnemonic-toolkit/src/bip85.rs` callees that use `Shake256::default()` + `.update()` + `.finalize_xof()` to derive BIP-85 child entropy. The XOF reader holds Keccak sponge state with BIP-85 child entropy mixed in until the reader drops.
- **What:** `sha3::digest::ExtendableOutput` returns a `Shake256Reader` that reads the XOF stream lazily. The reader's internal Keccak state carries the absorbed entropy (the child secret) until the reader drops. Upstream `sha3` does not implement `Zeroize` on the Keccak state or the XOF reader. Toolkit mitigation: minimize XOF reader lifetime — call `.read(...)` into a Zeroizing<[u8; N]> output immediately and drop the reader. Residual gap: the Keccak state inside the reader is not actively scrubbed before deallocation. Closes when upstream `sha3` adds `impl Zeroize` on its core state types.
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `bip38-crate-internal-zeroize-upstream` — `bip38` crate's scrypt intermediate state is not zeroize-aware

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 carry-over from survey §3.
- **Where:** Upstream crate `bip38 = "1"`. Affects `crates/mnemonic-toolkit/src/cmd/convert.rs::Bip38` decrypt arm at `convert.rs:1135` (and the V3 NULL-byte-passphrase path closed in Phase 1).
- **What:** The `bip38` crate's internal scrypt KDF intermediate state and the AES round buffers are not Zeroize-wrapped. Toolkit can `Zeroizing`-wrap the *returned* `(privkey_bytes, compressed_flag)` tuple but cannot reach into the crate's stack frames during decrypt. Residual gap: scrypt intermediate state (~1 MiB by default cost factor) lives un-scrubbed on the stack/heap during the decrypt call. Closes when upstream `bip38` adds Zeroize discipline (or when the toolkit replaces with an internally-controlled scrypt + AES implementation).
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `secret-memory-hygiene-cycle-b` — `mlock(2)` / `VirtualLock` page-pinning infrastructure (Cycle B)

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 0 R3 architect-review SPLIT-CYCLE recommendation; opened as standalone tracker entry per Phase 3 hygiene-matrix R1 (Opus, finding C-1).
- **Where:** New module `crates/mnemonic-toolkit/src/mlock.rs` (533 LOC, Fix-B slice-fn-only design). Cross-repo: toolkit handles Sites 1-4; ms-cli handles Site 5 via inline copy. Sites realized:
  1. **Site 1** (4 per-handler clap-binding pins): `cmd/bundle.rs:129+133` (passphrase + per-slot), `cmd/verify_bundle.rs:143-150` (same), `cmd/convert.rs:668+` (effective_passphrase / effective_bip38_passphrase / primary_value), `cmd/derive_child.rs:122+` (from_value + stdin_passphrase).
  2. **Site 2** `ResolvedSlot._entropy_pin: Option<Rc<PinnedPageRange>>` at `synthesize.rs:604` (Rc not Arc per clippy `arc_with_non_send_sync` at `ddb371c`). 12 ctor sites populated (incl. `pub type CosignerKeyInfo = ResolvedSlot;` alias).
  3. **Site 3** `DerivedAccount._entropy_pin: PinnedPageRange` at `derive.rs:34`. 1 ctor site at `derive_slot.rs:89`. Cycle A `impl Drop for DerivedAccount` PRESERVED.
  4. **Site 4** bip85 7 function-local pins at `bip85.rs:{84,110,138,170,188,203,241}` (heap-promoted in Phase 1).
  5. **Site 5** ms-cli `parse::read_stdin()` pin at `parse.rs:65` (post Cycle A `Zeroizing<String>` shift).
- **What:** Phase 0 SPEC (`design/SPEC_secret_memory_hygiene_v0_9_B.md`) → Phase 1 (bip85 heap-promote precursor) → Phase 2 (mlock module Fix-B slice-fn-only + first Rust CI workflow + Miri) → Phase 3a Path B-lite (Sites 1-4 + main wire + release-build CI job; Cycle-A→Zeroizing field-type migration on ResolvedSlot/DerivedAccount carved out to v0.10.1 via [[resolved-slot-derived-account-zeroizing-field]]) → Phase 3b cross-repo (ms-cli inline mlock.rs copy + Site 5 + main wire) → PE (audit matrix + G6 invariant test + lockstep tags). POSIX-only (Linux + macOS); Windows VirtualLock deferred (SPEC §3 `OOS-windows-virtuallock`).
- **Why deferred:** R3 SPLIT-CYCLE finding — combining mlock with Zeroizing would have doubled Cycle A's review surface; splitting keeps each cycle's blast radius reviewable.
- **Status:** `resolved 9f63e8e` — `mnemonic-toolkit-v0.10.0` tag pushed 2026-05-13. Companion lockstep tag: `ms-cli-v0.3.0` (mnemonic-secret `2e7c275`). All 7 SPEC §6 gates satisfied (G1 functional / G2 soft-fail / G3 platform / G4.a Cycle A Drop preserved + G4.b Miri / G5 lockstep tags / G6 inline-copy invariant test / G7 wire-format unchanged). Cycle-close artifacts: audit matrix `design/agent-reports/v0_9_B-secret-memory-hygiene-matrix.md`; PE R0 report `design/agent-reports/v0_9_B-PE-r0.md`. Open continuation: [[resolved-slot-derived-account-zeroizing-field]] (v0.10.1 patch).
- **Tier:** `v0.9.x`
- **Companion:** `mnemonic-secret/design/FOLLOWUPS.md` — same `secret-memory-hygiene-cycle-b` short-id (cross-repo cycle entry).

### `cycle-b-pre-spec-questions` — pre-SPEC scoping questions blocking Cycle B drafting

- **Surfaced:** 2026-05-13, v1.0 roadmap-survey Bucket 1 drill-down (Opus scoping read-out, atop `mnemonic-toolkit-v0.9.2` ship). Companion to `secret-memory-hygiene-cycle-b`.
- **Where:** Resolves into the eventual `design/SPEC_secret_memory_hygiene_v0_9_B.md` Phase 0 (not yet drafted). Source artifacts surveyed: `design/FOLLOWUPS.md` Cycle B entry; `design/SPEC_secret_memory_hygiene_v0_9_0.md` §3 OOS-mlock-cycle-b (lines 271-305); `design/agent-reports/v0_9_0-secret-memory-survey.md` §4 (lines 161-210); `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md` §4 (lines 247-269); `design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md` (R3 SPLIT-CYCLE + R1 mlock-module-shape seed).
- **What:** Cycle B has enough material to start SPEC drafting, but 4 open questions + 1 architectural trap + ~4 unaddressed items must resolve before / during Phase 0 SPEC. Drafting paused until these are dispositioned.
  1. **Canonical 5-site list reconciliation.** SPEC §3 OOS-mlock-cycle-b lists `{clap-args, ResolvedSlot.entropy, DerivedAccount.entropy, bip85 [u8;64], ms-cli stdin String}`. Hygiene-matrix §4 substitutes `secp256k1::SecretKey` + `bip39::Mnemonic` interiors for slots 2 and 3 (defense-in-depth reframing). The two lists overlap by ~3 sites only. SPEC list is canonical per R4 I-R4-4 fold; Cycle B SPEC must explicitly pick one enumeration and document the matrix's alternative as OOS or supplementary coverage.
  2. **Toolkit-only scope vs ms-cli site #5.** Cycle B is framed "toolkit-only" (FOLLOWUPS + hygiene-matrix §4) but SPEC site #5 lives at `mnemonic-secret/.../ms-cli/parse.rs:45`. Either drop site #5 or revise scope to "toolkit + ms-cli stdin" (which makes Cycle B cross-repo, with a companion entry needed in `mnemonic-secret/design/FOLLOWUPS.md`).
  3. **bip85 `[u8; 64]` heap-promotion ordering.** SPEC site #4 is stack-resident `[u8; 64]` today. Survey §4 lines 206-210 says "heap-promote first; mlock those *if* they get heap-promoted." Either Cycle B absorbs heap-promotion as a precursor Phase 1, or a separate predecessor cycle does it first. Affects Cycle B's plan shape.
  4. **Platform commitment scope.** FOLLOWUPS commits to 3 platforms (Linux mlock + macOS mlock + Windows VirtualLock). Hygiene-matrix §4 narrows to "libc / Linux-specific." Pick one — the soft-fail abstraction shape (single backend with platform-gates vs three backends behind a trait) depends on this.

  **Architectural trap on record (R3 I-R3-2, Phase 0 R1 report lines 188-260):** The Phase 0 R1 prototype `try_mlock_region(&[u8])` byte-slice API "traps callers into page-vs-byte granularity wastefulness." `mlock(2)` pins pages, not bytes; SPEC §3 OOS-secret-arena defers proper page-aligned allocation to a future Cycle C (`dedicated-secret-arena`). Cycle B accepts residual page-residue from co-allocated non-secret data on locked pages; SPEC must document this and pick a signature shape that doesn't pretend byte-granularity is real.

  **Items not addressed in existing artifacts** (Phase 0 design decisions): soft-fail logging channel / level / format; `RLIMIT_MEMLOCK` exhaustion semantics (no soft-fail story beyond `EPERM` today); `CAP_IPC_LOCK` probe-up-front vs fail-per-call; cgroup memory limits.

  **Resolutions (2026-05-13 session, user decisions):**
  - **Q1 resolved:** SPEC §3 OOS-mlock-cycle-b 5-site list is canonical. Matrix's `secp256k1::SecretKey` + `bip39::Mnemonic` substitutions are out-of-Cycle-B supplementary coverage (filed in Cycle B SPEC §3 as `OOS-upstream-zeroize-mlock`); revisit when those upstreams gain Drop+Zeroize.
  - **Q2 resolved (toward cross-repo):** ms-cli site #5 stays IN Cycle B's target list. Cycle B becomes cross-repo (toolkit + ms-cli). Companion FOLLOWUP `secret-memory-hygiene-cycle-b` to be filed in `mnemonic-secret/design/FOLLOWUPS.md` at P0 SPEC ship. The "toolkit-only" framing in earlier artifacts is superseded by this SPEC's §5 cross-repo coordination.
  - **Q3 resolved:** Cycle B absorbs bip85 `[u8; 64]` heap-promotion as Phase 1 (P1 toolkit-only precursor refactor; P2 builds mlock module; P3a applies at toolkit sites; P3b applies at ms-cli; PE rollup).
  - **Q4 resolved:** Linux + macOS (POSIX path) committed for Cycle B. Windows `VirtualLock` deferred to a separate future cycle once the POSIX soft-fail abstraction has settled. Filed in Cycle B SPEC §3 as `OOS-windows-virtuallock`.

  **Architectural trap resolved (R3 I-R3-2):** Cycle B's `pin_pages_for(&[u8]) -> PinnedPageRange` returns the actual page range pinned (page-granularity explicit in the return type), NOT the byte-slice fiction. SPEC §3 `OOS-page-residue-elimination` documents that co-resident non-secret data on locked pages is incidentally pinned; full isolation deferred to Cycle C `dedicated-secret-arena`.

  **Brainstorming-session resolutions (5 additional Qs, 2026-05-13):**
  - **API shape:** hybrid — `MlockedZeroizing<T>` wrapper (sites 2/3/4) + `pin_pages_for(&[u8])` slice fn (sites 1/5). Matches libsodium's two-tier API.
  - **Capability detection:** try-and-soft-fail per call (no upfront probe). `MlockState` process-static singleton aggregates failures into a single 2-line stderr summary at end of process via `report_at_exit()`.
  - **Logging:** stderr plain-text, 2 lines, no suppression flag/env-var.
  - **Errno discipline:** all errnos soft-fail in release; `debug_assert!` on unreachable `EINVAL` in debug builds.
  - **Cross-repo sharing:** inline copy of `pin_pages_for` in both repos; CI invariant test diffs the two implementations (normalized) and fails on drift. No shared `mnemonic-mlock` crate; constellation stays at 4 crates.
- **Why deferred:** v1.0 roadmap pass; user direction is to capture pre-SPEC scope state so a future SPEC-drafting session starts cold-but-informed rather than re-discovering the discrepancies.
- **Status:** `resolved by P0 ship (commit 0c02247, 2026-05-13) — Cycle B SPEC at design/SPEC_secret_memory_hygiene_v0_9_B.md; reviewer-loop CLEAR 0C/0I across R1 (design/agent-reports/v0_9_B-phase-0-spec-r1.md: 2C/3I folded) and R2 (design/agent-reports/v0_9_B-phase-0-spec-r2.md: 0C/0I confirmed). All 4 pre-SPEC questions plus 5 brainstorming-session questions dispositioned; resolutions inlined in the What block above. Companion FOLLOWUP secret-memory-hygiene-cycle-b filed in mnemonic-secret at P0 close per SPEC §5.`
- **Tier:** `v0.9.x`
- **Companion:** `secret-memory-hygiene-cycle-b` (parent cycle entry) at `design/FOLLOWUPS.md`. If Q2 resolves toward "ms-cli stdin is in scope," a companion entry in `mnemonic-secret/design/FOLLOWUPS.md` is needed at SPEC drafting time.

### `md-mk-private-key-surface-watch` — reopen md/mk Cycle A participation if either repo grows a private-key surface

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 0 R3 architect-review I-R3-4 fold (drop md/mk symmetry-stubs); opened as standalone tracker entry per Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-md-mk` class.
- **Where:** `descriptor-mnemonic` repo (md-codec + md-cli) and `mnemonic-key` repo (mk-codec + mk-cli). Currently both hold xpub-only / descriptor-only material with no private-key buffer.
- **What:** Cycle A drops the no-scope-symmetry matrix stubs originally planned for md/mk repos because they have no secret material to audit. If either repo later gains a private-key surface (e.g., a future md-codec descriptor-binding with embedded xprv, or an mk-codec xprv passthrough), this FOLLOWUP fires and Cycle A's hygiene discipline (Zeroizing + SAFETY anchors + matrix delta) reopens for the affected sibling.
- **Why deferred:** No secret material to audit today.
- **Status:** `open` (monitoring)
- **Tier:** `cross-repo`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md` — same `md-mk-private-key-surface-watch` short-id.

### `secret-memory-hygiene-v0_9-cycle-a` — cross-repo cycle: OWNED-buffer secret-memory hygiene v0.9.0 Cycle A

- **Surfaced:** 2026-05-13. Cycle SPEC at `design/SPEC_secret_memory_hygiene_v0_9_0.md`. Plan at `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`. Survey precursor at `design/agent-reports/v0_9_0-secret-memory-survey.md`. R1+R2+R3+R4+R5 architect-review disposition at `design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md` (5 rounds: Sonnet/Sonnet/Opus/Opus/Sonnet, cleared CLEAR 0C/0I after R3 SPLIT-CYCLE pushback + user decisions on impl-Drop approach + drop md/mk symmetry-stubs).
- **Where:** mnemonic-toolkit Phases 1 (argv close: 9 toolkit flag-rows + 5 distinct impl changes), 2 (zeroize discipline: ~30 toolkit OWNED rows + `derive_master_seed` seed-step helper + `impl Drop for DerivedAccount` with `into_parts()` migration of 3 internal move-out sites at `bundle.rs:325-329`, `bundle.rs:421-425`, `synthesize.rs:741-744`), 3 (hygiene matrix file), E (rollup). Sibling participation: mnemonic-secret Phase 2 (ms-cli + ms-codec zeroize, 4 + 10 OWNED rows) + Phase 3 (matrix file).
- **What:** OWNED-buffer first-pass at secret-memory hygiene. Closes argv leakage on toolkit's `bundle` / `verify-bundle` / `derive-child` / `convert --bip38-passphrase` flags (via new `--*-stdin` flags + `slot_input.rs` `=-` parser extension). Adds zeroize-on-drop semantics to every OWNED secret allocation in ms-codec + ms-cli + mnemonic-toolkit. Cycle B (mlock infrastructure) is a separate post-Cycle-A cycle per R3 SPLIT-CYCLE finding.
- **Status:** `resolved 9035656` — `mnemonic-toolkit-v0.9.2` tag pushed 2026-05-13. Sibling-repo tags shipped in lockstep: `ms-codec-v0.1.3` (mnemonic-secret `b1694e2`), `ms-cli-v0.2.2` (mnemonic-secret `ab8c73f`). All 6 SPEC §6 gates satisfied; cycle B (mlock) deferred to a separate cycle.
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-secret/design/FOLLOWUPS.md` — same `secret-memory-hygiene-v0_9-cycle-a` short-id. md / mk repos do NOT receive a companion entry this cycle (xpub-only material; SPEC §3 OOS-md-mk + R3 I-R3-4 fold).

### `bip-vector-adoption-v0_8` — cross-repo cycle: BIP-vector adoption v0.8.0

- **Surfaced:** 2026-05-13. Cycle SPEC at `design/SPEC_test_vector_audit_v0_8_0.md`. Plan at `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`. R1 review at `design/agent-reports/v0_8_0-phase-0-spec-plan-r1.md`.
- **Where:** mnemonic-toolkit Phase 3 = BIP-85 v85.3 (24-word BIP-39) cell at `crates/mnemonic-toolkit/tests/cli_derive_child.rs::cell_2b_bip39_24_words_reference_vector`. BIP-39 Trezor English fill (the other v0.7.1 §5 carry-over named in SPEC §2) was already closed by `feat(v0.8-phase-8)` commit `85694b2` *before* this cycle started; SPEC §2 row updated to record this. Net new for this cycle from the toolkit side: +1 vector (v85.3) plus Phase 4 audit-matrix cross-repo lift + Phase 0 SPEC + plan landed at `d0e6afc`.
- **What:** This repo's contribution to the v0.8.0 cross-repo vectors-only patch cycle. Closes when the cycle's audit-matrix successor doc lands at `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` (Phase 4) and the patch tag is cut at Phase E. Sibling-repo phases: descriptor-mnemonic Phase 1 (BIP-341, committed `b464f3f`); mnemonic-secret Phase 2 (BIP-93, committed `7101c16`).
- **Status:** `resolved f036737` — mnemonic-toolkit-v0.9.1 tag pushed; cycle close PR #15 merged. Companion sibling-repo tags: ms-codec-v0.1.2 (mnemonic-secret 527c9c7), md-codec-v0.32.1 (descriptor-mnemonic ef00e07), mnemonic-key PR #10 (6d43115, docs-only no tag).
- **Tier:** `cross-repo`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md` — same `bip-vector-adoption-v0_8` short-id in each.

### `bip340-schnorr-signing-surface-evaluation` — BIP-340 Schnorr test vectors deferred pending a signing surface

- **Surfaced:** 2026-05-13, v0.8.0 Phase 0 (SPEC §3 OUT-OF-SCOPE classification).
- **Where:** No file; cross-repo classification. No signing surface exists in any of the four sibling crates (grep for `schnorr` / `sign` / `signing_key` returns zero matches across `descriptor-mnemonic`, `mnemonic-toolkit`, `mnemonic-secret`, `mnemonic-key`).
- **What:** BIP-340 ships a sidecar CSV at `bip-0340/test-vectors.csv` with Schnorr signature test vectors. None of the four sibling crates exposes a signing surface, so BIP-340 is OUT-OF-SCOPE-PER-LAYER for the v0.8.0 vectors-only cycle. If a future cycle introduces signing (e.g., a `mnemonic sign-message` BIP-322 surface, or hardware-signer integration), this FOLLOWUP closes by mirroring the CSV into the relevant repo.
- **Status:** `open` (deferred until signing surface lands).
- **Tier:** `v1+`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` — `bip341-keypath-signing-vector-coverage` (the BIP-341 `keyPathSpending` corpus has the same gating).

### `bip39-japanese-wordlist-support` — BIP-39 Japanese vectors require JP wordlist surface

- **Surfaced:** 2026-05-13, v0.8.0 Phase 0 (SPEC §3 OUT-OF-SCOPE classification).
- **Where:** `ms-codec` and `mnemonic-toolkit` are English-only at v0.8.x. The Trezor python-mnemonic repo also publishes a Japanese vector corpus at `https://github.com/bip32JP/bip32JP.github.io/blob/master/test_JP_BIP39.json`.
- **What:** Extending BIP-39 support to the Japanese wordlist would add ~24 more Trezor-style vectors. Out-of-scope-per-product at v0.8.x; if a future cycle adds JP support, this FOLLOWUP closes by mirroring the JP vector file into `tests/`.
- **Status:** `open` (deferred; product decision, not a regression).
- **Tier:** `v1+`
- **Companion:** None (single-repo concern; `ms-codec` carries the wordlist plumbing).

### `md-cli-unspendable-key-v0.19-error-string-stale-companion` — companion: md-cli error string still references "v0.19+"

- **Surfaced:** 2026-05-11, toolkit-repo Phase 0.B audit review r1 (commit `713178c`). Surfaced from this repo's audit pass but the fix lives in the sibling `descriptor-mnemonic` repo.
- **Where:** `bg002h/descriptor-mnemonic/crates/md-cli/src/main.rs:224`.
- **What:** Companion entry — see the primary entry in `descriptor-mnemonic/design/FOLLOWUPS.md` (`md-cli-unspendable-key-v0.19-error-string-stale`) for the full action item. No toolkit-side action; closure will happen when the md1-repo entry resolves.
- **Status:** `resolved` (2026-05-11 alongside the primary md1-side fix; see the md1 FOLLOWUP entry for the resolving commit).
- **Tier:** `cross-repo`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` — `md-cli-unspendable-key-v0.19-error-string-stale`

### `manual-v0.18-stale-md1-scenario-phrases` — quickstart/workflow chapters carry v0.17-era md1 phrases that no longer round-trip under v0.18

- **Surfaced:** 2026-05-09, PR #11 architect review (manual-mirror for descriptor-mnemonic v0.18 cycle).
- **Where:** `docs/manual/src/30-workflows/31-singlesig-steel.md` (lines 87–89), `docs/manual/src/20-quickstart/22-first-bundle.md` (~line 63), `docs/manual/src/20-quickstart/23-verify.md` (~line 24), `docs/manual/src/40-cli-reference/44-mk-cli.md` (~line 54). All reference the v0.17-era 3-chunk `md1zsxdspqqqpm6jzzqq...` scenario phrase set (3-of-3 multisig).
- **What:** descriptor-mnemonic v0.18 is a wire-format break (`Tag::TrUnspendable` removed, `key_index_width` formula changed). v0.17 phrases now reject under v0.18 with `Error::UnknownExtensionTag(0x05)`. PR #11 limited scope to CLI surface (per `manual-cli-surface-mirror` invariant). The scenario phrases need regenerating from source (run the `mnemonic` derivation pipeline against the abandon mnemonic with v0.18 binaries) and the chapters re-published.
- **Why deferred:** PR #11 maintains narrow CLI-surface-mirror scope. Scenario-content refresh is a separate concern that involves running the full 4-format pipeline and regenerating multiple chunked phrases. Local-only impact; toolkit CI runs `make lint` not `make verify-examples`.
- **Status:** `open`
- **Tier:** `v0.2` (next minor; non-blocking for descriptor-mnemonic v0.18 release).

### `lint-md-flag-coverage-vacuous-with-md_bin-true` — CI flag-coverage step skipped for md/ms/mnemonic via `MD_BIN=true` substitution

- **Surfaced:** 2026-05-09, PR #11 architect review.
- **Where:** `.github/workflows/manual.yml` invokes `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk`. The `flag-coverage` step in `tests/lint.sh` runs `eval "$cmd <subcommand> --help"`; when `cmd=true`, the shell builtin `true` ignores all args and emits no flags, triggering the `warn "no flags parsed"` skip path.
- **What:** Only `mk` actually executes flag-coverage in CI (mk is `cargo install`'d in the workflow). md/ms/mnemonic flag-coverage is silently vacuous. Pre-existing gap; not introduced by PR #11. The `lint` claim "every CLI flag is documented" is therefore not enforced for 3 of 4 binaries in CI — manual `make lint MD_BIN=/path/to/md` runs catch flag drift, but no CI gate.
- **What to fix:** install `md`, `ms`, and `mnemonic` binaries in the manual.yml workflow (similar to how `mk` is installed) and pass them to `make lint` instead of `=true`.
- **Why deferred:** orthogonal to PR #11's CLI-surface-mirror scope; pre-existing infrastructure gap.
- **Status:** `open`
- **Tier:** `v0.2` (CI hardening; non-blocking).

### `manual-cli-surface-mirror` — manual mirrors the four-format CLI/API surface

- **Surfaced:** 2026-05-07, m-format-star user manual v0.1 release (`manual-v0.1.0` tag; PR #1).
- **Where:** `docs/manual/src/40-cli-reference/` (`41-mnemonic.md`, `42-md.md`, `43-ms.md`, `44-mk-codec-rust.md`); CI gate at `docs/manual/tests/lint.sh` `flag-coverage` step (per-`<binary, subcommand>` pair).
- **What:** v0.1 of the manual mirrors `mnemonic` (this repo), `md-cli` (`descriptor-mnemonic`), `ms-cli` (`mnemonic-secret`), and the `mk-codec` Rust API (`mnemonic-key`) verbatim against toolkit v0.8.0. **Any flag addition or removal in any of those four surfaces must touch `docs/manual/src/40-cli-reference/` in lockstep with the implementing PR**; the manual's `flag-coverage` lint step gates on missing flags. Companion entries: `manual-cli-surface-mirror` in `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md`, and `mnemonic-key/design/FOLLOWUPS.md`.
- **Why filed:** the manual is a separate artifact (independent `manual-v*` versioning); without an explicit cross-repo mirror invariant, sibling-side flag changes would silently drift the manual.
- **Status:** `open` (mirror invariant active for the lifetime of `docs/manual/`)
- **Tier:** `cross-repo`

### `spec-5-5-kind-enum-gap` — SPEC §5.5 `kind` enum table omits `NetworkMismatch` and `FutureFormat`

- **Surfaced:** Phase 1 review r1 (L-1).
- **Where:** `design/SPEC_mnemonic_toolkit_v0_1.md` §5.5.
- **What:** SPEC §5.5 enumerates `"kind"` JSON values as `"BadInput" | "Bip39" | … | "ModeViolation"` but doesn't list `NetworkMismatch` and `FutureFormat`. The implementation correctly returns those discriminants; the SPEC prose is just incomplete.
- **Why deferred:** SPEC-prose-only; no code change required. Update during the next SPEC revision.
- **Status:** `resolved (this commit, 2026-05-13) — NetworkMismatch + FutureFormat added to §5.5 kind enum.`
- **Tier:** `v0.1-nice-to-have`

### `mk-codec-chunked-visual-grouping-helper` — mk-codec lacks a per-string visual grouping helper

- **Surfaced:** Phase 1 spike memo + Phase 1 review r1 (L-2).
- **Where:** `crates/mnemonic-toolkit/src/format.rs::chunk_mk1` (cross-repo: would consume a new `mk_codec::encode::render_grouped` if it existed).
- **What:** md-codec exposes `render_codex32_grouped(s, 5)` for engraving-friendly hyphenated 5-char groups; mk-codec has no equivalent. Toolkit's `chunk_mk1` falls back to space-separated 5-char groups via `chunk_5char`. v0.1 fixtures pin the space-separated behavior.
- **Why deferred:** non-blocking; functionally equivalent fallback. Library-API gap in mk-codec.
- **Status:** `open`
- **Tier:** `cross-repo`

### `plan-spike-md-codec-filler-bug` — IMPLEMENTATION_PLAN's `spike_md_codec.rs` snippet uses invalid SEC1 filler

- **Surfaced:** Phase 1 review r1 (Nit-1) + Task 1.1 spike memo.
- **Where:** `design/IMPLEMENTATION_PLAN_mnemonic_toolkit_v0_1.md` Task 1.1, `spike_md_codec.rs` snippet (~line 232–260).
- **What:** Plan-given snippet uses `[0x42; 65]` as `tlv.pubkeys` filler, which violates the SEC1-compressed-pubkey prefix invariant (must be 0x02/0x03) and panics with `InvalidXpubBytes`. Spike memo documents the working filler `[0x11; 32] || 0x02 || [0x22; 32]` from `md_codec::identity::deterministic_xpub`. Plan source not patched — future readers running the snippet verbatim will trip the same panic.
- **Why deferred:** spike memo supersedes plan source; cosmetic plan-source bug.
- **Status:** `resolved (this commit, 2026-05-13) — Task 1.1 snippet now uses 32B 0x11 chain_code || 0x02 SEC1 prefix || 32B 0x22 x-coordinate.`
- **Tier:** `v0.1-nice-to-have`

### `plan-trezor-24-fingerprint-stale` — IMPLEMENTATION_PLAN has wrong 24-word zero-entropy master fingerprint

- **Surfaced:** Task 2.1 implementer (verified via spike harness `/tmp/toolkit-spike/spike_trezor_fp.rs`).
- **Where:** `design/IMPLEMENTATION_PLAN_mnemonic_toolkit_v0_1.md` Task 2.1 test assertion (~line 1540) + Task 2.3 commit-message body.
- **What:** Plan asserts `73c5da0a` as the Trezor 24-word "abandon × 23 art" master fingerprint. That value is the **12-word** "abandon × 11 about" vector's fingerprint (rust-miniscript test corpus). Correct 24-word fingerprint is `5436d724`. Handoff doc was corrected during execution; plan source unpatched.
- **Why deferred:** test code uses correct value; only plan documentation is stale.
- **Status:** `resolved (this commit, 2026-05-13) — plan now references 5436d724 (24-word fingerprint) in all 3 sites.`
- **Tier:** `v0.1-nice-to-have`

### `friendly-mk-codec-mixedcase-wording` — `friendly_mk_codec` `MixedCase` text word-order differs from SPEC §6.4.4

- **Surfaced:** Phase 3 review r1 (L-1).
- **Where:** `crates/mnemonic-toolkit/src/friendly.rs:friendly_mk_codec` (`MixedCase` arm).
- **What:** SPEC §6.4.4 row says `"mixed case in mk1 input string"`. Code says `"mk1 mixed case in input string"`. Functionally equivalent; word order differs.
- **Why deferred:** no integration test pins the byte-exact text yet; cosmetic.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `bundle-emit-bypasses-chunk-mk1-alias` — `bundle.rs::emit()` calls `chunk_5char` directly for mk1; `chunk_mk1` alias dead

- **Surfaced:** Phase 3 review r1 (L-2) + Phase 5 review r1 (L-2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::emit` + `crates/mnemonic-toolkit/src/format.rs::chunk_mk1`.
- **What:** `chunk_mk1` is a reserved alias for `chunk_5char`, retained against the future mk-codec grouping helper (see `mk-codec-chunked-visual-grouping-helper`). `bundle.rs::emit` calls `chunk_5char` directly, leaving `chunk_mk1` flagged as dead code. Switch the call site to `chunk_mk1` so the swap point is single-edit.
- **Why deferred:** functionally identical; one-line cleanup.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `watch-only-stderr-warning-suborder` — depth advisory ordering vs account-index hazard unspecified

- **Surfaced:** Phase 3 review r1 (L-3).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_watch_only`.
- **What:** Watch-only path emits the conditional depth advisory before the unconditional account-index hazard. SPEC §5.2 lists "watch-only mode warning" as item 3 without specifying the sub-order between these two. Phase 5 fixtures don't cover stderr ordering.
- **Why deferred:** SPEC-ambiguous; Phase 5 doesn't pin the ordering.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `spec-2-2-2-vs-5-4-checks-count-prose` — SPEC §2.2.2 prose says "four checks" but §5.4 schema mandates 9-element array

- **Surfaced:** Phase 4 review r1 (L-1).
- **Where:** `design/SPEC_mnemonic_toolkit_v0_1.md` §2.2.2 vs §5.4.
- **What:** §2.2.2 lists 4 substantive watch-only checks; §5.4 schema (line 552) requires all 9 check-name slots populated, with `skipped` for non-applicable. Implementation follows §5.4 (correct). §2.2.2 prose should clarify "4 substantive (5 of the 9 schema slots are `skipped` per §5.4)".
- **Why deferred:** SPEC-internal inconsistency; implementation behavior is correct per the schema.
- **Status:** `resolved (this commit, 2026-05-13) — §2.2.2 prose clarified: "four substantive checks" with explicit §5.4 9-slot schema reference.`
- **Tier:** `v0.1-nice-to-have`

### `bundle-mismatch-card-static-str-constraint` — `BundleMismatch.card: &'static str` constrains future runtime-id callers

- **Surfaced:** Phase 4 review r1 (L-2). Confirmed as Phase 0 mandatory fixup by 2026-05-05 v0.1 audit (`design/audit-v0_1-for-v0_2-extension.md` IMP-1).
- **Where:** `crates/mnemonic-toolkit/src/error.rs::ToolkitError::BundleMismatch`.
- **What:** Field type was `&'static str`. v0.2 multisig emits per-cosigner card identifiers like `"mk1[0]"` that are runtime-formatted; `&'static str` would force a breaking field-type change mid-v0.2-cycle. Resolved as part of v0.2 Phase 0.
- **Status:** `resolved 9396a58 — field changed to String; test construction sites updated to .into(); doc-comment clarified.`
- **Tier:** `v0.2`

### `verify-bundle-text-mode-trailing-space` — `"{}: {} {}"` produces trailing space when `detail` is empty

- **Surfaced:** Phase 4 review r1 (L-3).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run` text-mode output.
- **What:** Skipped checks with empty `detail` render as `"md1_xpub_match: skipped "` (trailing space). SPEC §5.4 only pins JSON byte-exact; text mode is unpinned.
- **Why deferred:** cosmetic; not test-covered.
- **Status:** `resolved by v0.5.0 Phase F (commit 85c678b) — branch on detail.is_empty() at 3 emit sites`
- **Tier:** `v0.1-nice-to-have`

### `error-allow-comments-staleness` — `error::Result<T>` and `BundleMismatch` doc-comments will rot

- **Surfaced:** Phase 4 review r1 (N-1, N-2) + Phase 5 review r1 (N-2). Bundled into Phase 0 fixup by 2026-05-05 v0.1 audit (`design/audit-v0_1-for-v0_2-extension.md` IMP-2).
- **Where:** `crates/mnemonic-toolkit/src/error.rs` `Result` alias + `BundleMismatch` variant doc.
- **What:** `Result<T>` allow-comment said "reserved for in-crate use" but the type is `pub type` (exported). `BundleMismatch` doc-comment said "Constructed by integration tests in Phase 5" — stale once v0.2 wires the variant as a live runtime error.
- **Status:** `resolved 9396a58 — Result<T> comment now reads "Convenience alias; exported for downstream-crate use." BundleMismatch comment now reads "Exit-4 verify-bundle mismatch variant; card identifies the mismatching card (e.g., mk1, md1, or mk1[N] for multisig cosigner N)."`
- **Tier:** `v0.1-nice-to-have`

### `cli-watch-only-test-hardcodes-fingerprint` — `cli_bundle_watch_only.rs` hardcodes `5436d724` rather than reading from decoded mk1

- **Surfaced:** Phase 5 review r1 (L-3).
- **Where:** `crates/mnemonic-toolkit/tests/cli_bundle_watch_only.rs`.
- **What:** Test extracts the xpub from the mk1 fixture via `mk_codec::decode` (correct), but passes `"5436d724"` as the master-fingerprint argument literally. Works because the Trezor 24-word zero vector's fingerprint is constant; future vector swap requires updating the fingerprint in two places. Read it from `card.origin_fingerprint` instead.
- **Why deferred:** works; two-place edit risk only.
- **Status:** `resolved (this commit, 2026-05-13) — both tests read fp_hex from card.origin_fingerprint via .to_string(); cargo test --test cli_bundle_watch_only passes.`
- **Tier:** `v0.1-nice-to-have`

### `changelog-sha-pin-no-reproduction-command` — CHANGELOG SHA pin doesn't document how to reproduce it

- **Surfaced:** Phase 5 review r1 (N-1).
- **Where:** `CHANGELOG.md` Wire-format SHA pin section.
- **What:** SHA `81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6` is documented as `sha256(crates/mnemonic-toolkit/tests/vectors/v0_1/)` but doesn't specify the exact reproduction command (`shasum -a 256 *.txt | sort | shasum -a 256`). Verifiers may need to guess.
- **Why deferred:** verifiers can re-derive; doc-only clarity gap.
- **Status:** `resolved (2026-05-13 survey verified moot) — CHANGELOG.md v0.2 section (lines 1296-1301) already includes the reproduction command \`shasum -a 256 ... | sort | shasum -a 256\` with explicit "(resolves v0.1 FOLLOWUPS N-1)" attribution; this FOLLOWUPS entry's status update was missed in the v0.2 cycle.`
- **Tier:** `v0.1-nice-to-have`

### `cli-mode-violations-byte-exact-naming` — test names say "byte_exact" but use `str::contains`

- **Surfaced:** Phase 5 review r1 (N-3).
- **Where:** `crates/mnemonic-toolkit/tests/cli_mode_violations.rs`.
- **What:** Several test names use the suffix `_byte_exact` but the assertions use `predicate::str::contains(...)` (substring match). Tests are correct; naming overstates assertion strictness. Either rename to `_substring` or tighten the assertions to full-stderr equality (and pin the byte-exact stderr in fixtures).
- **Why deferred:** assertion strength is sufficient for current SPEC pinning; naming is the only mismatch.
- **Status:** `resolved (2026-05-13 survey verified moot) — \`tests/cli_mode_violations.rs\` no longer exists; the \`_byte_exact\` naming pattern is now distributed across per-feature test files (cli_bundle_full, cli_export_wallet_*, etc.) where the original test's scope was absorbed.`
- **Tier:** `v0.1-nice-to-have`

### `phase-2-review-byte-determinism-blind-spot` — process: byte-determinism invariants need a spike, not just a review

- **Surfaced:** Phase 5 implementer caught the bug; Phase 2 r1 + r2 reviews missed it.
- **Where:** Process / `feedback_spike_before_locking_wire_format` memory rule.
- **What:** Phase 2 reviews looked at code correctness against SPEC §4 but didn't run encode twice and diff the bytes. The result: `mk_codec::encode` drew `chunk_set_id` from CSPRNG, which broke v0.1's byte-reproducible-output contract. The fix (`derive_mk1_chunk_set_id` + `encode_with_chunk_set_id`) shipped in the Phase 5 release commit (`f2bd20a`). Process improvement: when a phase locks wire-format invariants that downstream phases will SHA-pin, the per-phase review checklist should include "encode twice, assert identical bytes".
- **Why deferred:** post-mortem item; resolved via the v0.1.0 release fix. Lesson worth carrying forward.
- **Status:** `resolved f2bd20a — Phase 5 fix shipped; process lesson captured here.`
- **Tier:** `v0.1-nice-to-have`

### `mk1-bip-chunk-set-id-determinism-guidance` — mk1 BIP recommendation for deterministic encoders

- **Surfaced:** Phase 5 byte-determinism fix (`f2bd20a`) — the toolkit-side derivation needs lifting into the mk1 BIP so other implementations producing reproducible corpora reach the same wire bits. Companion: same-id entry in `mnemonic-key/bip/bip-mnemonic-key.mediawiki`.
- **Where:** `bip/bip-mnemonic-key.mediawiki` String-layer header section in `mnemonic-key`.
- **What:** Toolkit shipped a `derive_mk1_chunk_set_id(&policy_id_stub)` helper deriving 20 bits from the leading bytes of the policy_id_stub. mk1 BIP edited to recommend this pattern (with the explicit formula `(stub[0] << 12) | (stub[1] << 4) | (stub[2] >> 4)`) and clarify decoders MUST accept any 20-bit value.
- **Why deferred:** mk1 BIP is a sibling-repo asset; toolkit's fix landed first.
- **Status:** `resolved 87bbc11 (mnemonic-key@main) — mk1 BIP §"String-layer header" updated 2026-05-04 with deterministic-encoder guidance + decoder-acceptance clarification. Pushed to bg002h/mnemonic-key.`
- **Tier:** `cross-repo`

### `dead-assert-tautological` — `synthesize.rs` invariant 1 debug-assert is tautological by construction

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-1).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:99` (`debug_assert_eq!(&card.policy_id_stubs[0], &stub)`).
- **What:** `stub` is computed from `policy_id.as_bytes()[..4]` and immediately passed as `policy_id_stubs[0]`. The assertion can never fail at the construction site. Phase 2 r1 originally flagged this as L-4. Pre-existing; meaningful assertion is invariant 2 (`is_wallet_policy()`).
- **Why deferred:** v0.2 multisig will need a meaningful assertion that loops over all per-cosigner stubs; resolve as part of v0.2 Phase C.
- **Status:** `resolved (this commit, 2026-05-13) — tautological debug_assert_eq removed from both single-sig synthesize paths in synthesize.rs (now lines 139, 171); the meaningful is_wallet_policy assert is retained. v0.2 multisig's proper looped assertion never materialized; removing the dead code rather than waiting indefinitely.`
- **Tier:** `v0.2`

### `dead-inner-guard-bundle-watch-only` — redundant `--xpub`-needs-`--master-fingerprint` guard inside `bundle_watch_only`

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:200` (inside `bundle_watch_only`).
- **What:** A redundant guard exists that would emit `BadInput` (exit 1) if `--master-fingerprint` is missing. Unreachable in practice — the mode-violation pre-check at `cmd/bundle.rs:93` rejects the same condition earlier with exit 2 + byte-exact §6.6 text. Future-refactor inconsistency risk.
- **Why deferred:** not currently triggered; v0.2 will refactor mode dispatch and naturally clean this up.
- **Status:** `open`
- **Tier:** `v0.2`

### `friendly-mapper-unit-test-gaps` — friendly-mapper unit tests cover only 3 of ~70 match arms

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-3).
- **Where:** `crates/mnemonic-toolkit/src/friendly.rs::tests`.
- **What:** Unit tests cover `friendly_bip39::UnknownWord`, `friendly_ms_codec::WrongHrp`, `friendly_mk_codec::PathTooDeep`. Untested at unit level: 4 of 5 `friendly_bip39`, all 3 `friendly_bitcoin`, 8 of 9 `friendly_ms_codec`, 21 of 22 `friendly_mk_codec`, all 41 `friendly_md_codec`. Integration tests likely exercise some paths end-to-end but unit isolation is thin.
- **Why deferred:** v0.2 will add new error paths through these mappers; expand the tests in lockstep with v0.2 Phase E.
- **Status:** `open`
- **Tier:** `v0.2-nice-to-have`

### `hex-dep-unused` — `hex = "0.4"` declared in Cargo.toml but unused in non-test source

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-4).
- **Where:** `crates/mnemonic-toolkit/Cargo.toml:27`.
- **What:** No `use hex` statement in any source module. Inert dependency carried from ms-cli precedent or SPEC §10.3 dep list.
- **Why deferred:** user's `feedback_dont_drop_reserved_deps` rule applies — confirm with user before removal. v0.2 may use `hex` for new error-message formatting (e.g., printing fingerprints in mode-violation output), in which case the dep activates naturally.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `parse_template-regex-line-ref` — SPEC v0.3 §4.9 step 2 cites wrong line range

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §4.9 step 2.
- **What:** Step 2 cites `descriptor-mnemonic/crates/md-cli/src/parse/template.rs:19-27` for the placeholder regex; the actual `Regex::new` call is at `:25-27` (line 19-24 are imports/doc-comments). Docs-only nit — implementation will read the actual regex from the source.
- **Why deferred:** non-blocking; can be patched alongside any v0.3 SPEC revision.
- **Status:** `resolved (this commit, 2026-05-13) — §4.9 step 2 line range updated to \`parse/template.rs:25-27\`.`
- **Tier:** `v0.3-nice-to-have`

### `unsupported-fragment-error-style` — SPEC v0.3 §6.8 error message text is verbose

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §6.8 (error message wording).
- **What:** The message reads `unsupported miniscript fragment: <fragment-string>; v0.3 walker covers BIP-388 surface modulo multi-leaf tap trees (deferred to v0.4)`. This is verbose for a CLI error; a tighter form (e.g. drop the parenthetical) would be friendlier.
- **Why deferred:** SPEC pins the message for byte-exactness; can be revisited at impl time if friendlier wording surfaces. Not blocking.
- **Status:** `open`
- **Tier:** `v0.3-nice-to-have`

### `walker-backport-to-md-cli` — toolkit's expanded walker should be backported to md-cli

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** cross-repo: `mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs` ↔ `descriptor-mnemonic/crates/md-cli/src/parse/template.rs`.
- **What:** v0.3 toolkit ships an expanded `walk_miniscript_node` covering all 24 v0.3-NEW `Terminal` arms (hash terminals, timelocks, wrappers, AND/OR/Thresh). md-cli's walker is the inspiration but currently rejects all of these. Backporting (or extracting both into a shared crate `descriptor-walker`) avoids divergence.
- **Why deferred:** scope of v0.3 is toolkit-only by user direction. Cross-repo coordination cycle in v0.4.
- **Status:** `open`
- **Tier:** `v0.4-cross-repo`

### `spike-report-citation` — v0.3 SPEC §9 Q2 closure should cite SPIKE report

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §9 Q2 closure.
- **What:** §9 Q2 declared "moot — v0.3 implements its own walker arms for hash terminals." Pre-Phase-A SPIKE produced `design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §2 confirming hash-terminal round-trip. §9 Q2 updated to cite the report.
- **Status:** `resolved 2026-05-05` (closed inline with SPIKE report patches).
- **Tier:** `v0.3`

### `synthesize-descriptor-fn-naming` — single-vs-split synthesize entry-point decision

- **Surfaced:** v0.3 SPEC § resolved at IMPLEMENTATION_PLAN drafting 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs` (Phase C of v0.3 plan).
- **What:** v0.3 SPEC §10 originally named `synthesize_descriptor_full` / `synthesize_descriptor_watch_only` (mirroring v0.2's two-function shape). v0.3 plan resolves to a single `synthesize_descriptor` entry point that dispatches single-sig vs multisig internally. This is slightly asymmetric with v0.2's pattern.
- **Why deferred:** flagged for Phase C reviewer to confirm the single-entry-point shape doesn't regress code clarity. Not a blocker.
- **Status:** `resolved by IMPLEMENTATION_PLAN_v0_3 Phase C.1` (single entry point chosen)
- **Tier:** `v0.3`

### `v0.2-spec-§8-tier-citation` — v0.3 SPEC §8 citation against v0.2 SPEC §8

- **Surfaced:** v0.3 SPEC architect review r3 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §8 deferred-items table (K-of-N row).
- **What:** §8 cites v0.2 tier of K-of-N share encoding as "v0.3 (gates on ms-codec v0.2)". Verify against v0.2 SPEC §8 verbatim language at impl time for citation accuracy.
- **Why deferred:** non-blocking; doc-only.
- **Status:** `resolved (2026-05-13 survey verified) — v0.3 SPEC §8 line 313 cites v0.2 tier of K-of-N as "v0.3 (gates on ms-codec v0.2)", matching v0.2 SPEC §8 line 582 outcome wording verbatim. Citation is accurate.`
- **Tier:** `v0.3-nice-to-have`

### `ctx-for-descriptor-heuristic-misroutes` — Phase A `ctx_for_descriptor` is string-prefix heuristic

- **Surfaced:** v0.3 Phase A end-of-phase architect review I-2 (2026-05-05).
- **Resolved:** v0.3 Phase C end-of-phase r1 (2026-05-05). Replaced string-prefix heuristic with post-resolve n-based classification inside `parse_descriptor`: `n == 1 → SingleSig`, `n ≥ 2 → MultiSig`. The dead `ctx_for_descriptor` function was removed.
- **Status:** `resolved by Phase C.6 r2 (2026-05-05)`
- **Tier:** `v0.3`

### `parse-descriptor-allow-dead-code-audit` — module-level `#![allow(dead_code)]` audit

- **Surfaced:** v0.3 Phase A end-of-phase architect review L-1 (2026-05-05).
- **Resolved:** v0.3 Phase C end-of-phase r1 (2026-05-05). Lifted module-level `#![allow(dead_code)]`. Two items remained dead at the binary-compile boundary (`DescriptorMode` enum + `determine_mode` fn, used only in tests + Phase D verify-bundle re-parse path); both received per-item `#[allow(dead_code)]`.
- **Status:** `resolved by Phase C.6 r2 (2026-05-05)`
- **Tier:** `v0.3`

### `descriptor-mode-engraving-card` — engraving card omitted in descriptor mode

- **Surfaced:** v0.3 Phase C end-of-phase architect review L-5 (2026-05-05).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` `descriptor_mode_emit` (Phase C.6).
- **What:** `engraving_card: None` for descriptor mode. The existing `engraving_card()` builder takes a `CliTemplate` + path-family + `EngravingMode`, which descriptor mode lacks. v0.3 ships without a descriptor-mode card; v0.4 should add a descriptor-aware engraving card (custom text including the descriptor string + per-cosigner xpub origins).
- **Why deferred:** out of v0.3 scope; engraving card logic is template-coupled.
- **Status:** `open`
- **Tier:** `v0.4`

### `engraving-card-unified-1-master-card` — Phase E unified engraving card deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase E scope decision 2026-05-05 (autonomous-mode time risk).
- **Where:** `crates/mnemonic-toolkit/src/format.rs::engraving_card` + `EngravingMode` enum + `crates/mnemonic-toolkit/src/cmd/bundle.rs` per-mode card emit sites.
- **What:** SPEC §5.5 specifies a single unified `BundleInputForCard` shape + `engraving_card_unified` render function emitting one master card per bundle (in place of v0.2/v0.3's per-mode `EngravingMode` variants). Phase E was originally scoped to land this in v0.4.0 with deprecation of `EngravingMode::*`; deferred to v0.4.1 because it is tightly coupled to the BundleJson schema-4 cutover and the multi-source synthesis path (the unified card needs `MsField` + per-slot blocks). Will land in lockstep with `bundle-json-schema-4-cutover`.
- **Why deferred:** scope-coupling to schema-4 cutover; foundation-only Phase D made standalone Phase E delivery low-value.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `verify-bundle-9-3plus6n-forensics` — Phase G verify-bundle 9/3+6N parity + per-cell forensics deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase G scope decision 2026-05-05 (autonomous-mode time risk).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_*` + `crates/mnemonic-toolkit/src/format.rs::VerifyCheck`.
- **What:** SPEC §5.7 specifies (a) descriptor-mode emits the same 9 / 3+6N check schema as template-mode (replacing v0.3's 3-element coarse ladder), (b) `VerifyCheck` gains four forensic fields (`expected`, `actual`, `diff_byte_offset`, `decode_error`), (c) verify-bundle dispatches on `schema_version` for schema-4 BundleJson with per-slot `MsField` array. All three sub-deliverables depend on the schema-4 cutover landing first. Bip388-distinctness symmetric enforcement (SPEC §4.11.c) IS shipping in v0.4.0 (Phase A wired `Bip388VerifyDistinctness` into `descriptor_mode_verify_run`).
- **Why deferred:** depends on `bundle-json-schema-4-cutover`; will land in lockstep with v0.4.1.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `bundle-json-schema-4-cutover` — full BundleJson schema-4 cutover deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase D scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/format.rs::BundleJson` + `crates/mnemonic-toolkit/src/cmd/bundle.rs::emit` + `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` + `crates/mnemonic-toolkit/src/synthesize.rs::Bundle`.
- **What:** v0.4.0 ships the `MsField = Vec<String>` type alias + multi-source synthesis primitives as a foundation, but DEFERS the full `BundleJson.ms1: Option<String>` → `ms1: MsField` migration + `schema_version: "3" → "4"` bump + verify-bundle schema-4 dispatch to v0.4.1. v0.4.0 retains the schema-3 envelope so all existing v0.2/v0.3 fixtures + JSON integration tests pass byte-identically. v0.4.1 lands the cutover with: (a) BundleJson.ms1 → MsField; (b) Bundle.ms1 → Vec<String>; (c) all integration test JSON assertions updated; (d) verify-bundle schema_version dispatch (read schema_version FIRST per SPEC §5.6); (e) regenerate or update v0.2/v0.3 carry-forward tests under the new envelope shape per SPEC §5.6 cross-schema invariant; (f) synthesize_multisig_multisource + synthesize_multisig_hybrid wired into bundle::run via BundleMode dispatch (Phase C foundation already in place); (g) **bundle::run top-level dispatch rewiring**: in v0.4.0 `args.slot` is parsed by clap into `BundleArgs.slot: Vec<SlotInput>` but `bundle::run` itself never reads it. v0.4.1 must wire `expand_legacy_to_slots(args.slot, ...)` → `validate_slot_set(&slots)?` → `detect_bundle_mode(&slots)?` → match-arm dispatch into the new `synthesize_multisig_multisource` / `synthesize_multisig_hybrid` paths AND rewrite the legacy `bundle_full` / `bundle_watch_only` / `bundle_multisig_*` calls to flow through the same SlotInput-driven path. This is a top-level surgery in `cmd/bundle.rs::run` itself, not just additions to the synthesis helper crate.
- **Why deferred:** scope risk in autonomous v0.4.0 release window — full surgery touches ≥10 source files + ~15 test assertions + fixture envelopes; landing without user oversight risks bugs the foundation-only approach avoids.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `bip388-distinctness-path-normalization-phase-b-decision` — typed-vs-raw path semantics in check_key_vector_distinctness

- **Surfaced:** v0.4 Phase A end-of-phase architect review L-1 (2026-05-05).
- **Where:** `crates/mnemonic-toolkit/src/parse_descriptor.rs:1049` (`check_key_vector_distinctness`); SPEC `design/SPEC_mnemonic_toolkit_v0_4.md` §4.11.b.
- **What:** Phase A compares `cs[i].path.to_string()` on typed `bitcoin::bip32::DerivationPath`. The bitcoin library normalizes `48h/0h/0h/2h` ↔ `48'/0'/0'/2'` at `from_str` time, so collision detection is normalization-aware. SPEC §4.11.b says "raw user-supplied path string ... no path canonicalization". In Phase A this is safe because all paths arrive through the typed lex/cosigner parser; in Phase B the `--slot @N.path=` raw string flows into the binding directly. Phase B must lock whether `CosignerKeyInfo.path` stores typed `DerivationPath` (normalizing) or raw `String` (preserving), then update SPEC §4.11.b's normalization-domain paragraph in lockstep.
- **Why deferred:** Phase A's typed approach is correct under the v0.3 binding model; the decision is a Phase B design choice (slot input parsing).
- **Status:** `resolved by v0.5.0 Phase C.1 (commit 4a650aa) — typed DerivationPath equality replaces raw-string in check_key_vector_distinctness`
- **Tier:** `v0.4-nice-to-have`

### `verify-bundle-helper-and-full-forensics-rollout-v0.4.4` — Phase P.1-P.5 deferred from v0.4.3 to v0.4.4 — SUPERSEDED

- **Status:** `superseded by verify-bundle-helper-call-sites-rollout-v0.4.5 (2026-05-06)`. v0.4.4 P.1+P.2 landed the `emit_verify_checks` helper foundation (#[allow(dead_code)] with 4 unit tests + SuppliedCards struct + watch-only short-circuit + multisig TODO stub). The ~78-site call-site refactors (run_full / run_multisig / descriptor_mode_verify_run consolidation + descriptor-mode 9/3+6N parity + watch-only test migration) deferred again to v0.4.5 per the v0.4.4 plan scope reduction.

- **Surfaced:** v0.4.3 Phase P scope decision 2026-05-06 (P.0 struct shape correction landed; P.1-P.5 deferred for scope safety).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (~78 VerifyCheck push sites + new `emit_verify_checks` helper + descriptor-mode 9/3+6N parity refactor).
- **What:** v0.4.3 P.0 corrected the VerifyCheck struct shape per SPEC §5.7 (`result: &'static str` → `passed: bool`). The full SPEC §5.7 rollout — `emit_verify_checks` helper + refactor of run_full / run_multisig / descriptor_mode_verify_run + per-cell forensic field population at every push site + descriptor-mode 9/3+6N parity (closes `verify-bundle-9-3plus6n-descriptor-mode-parity` simultaneously) + skipped-check decode_error population — is deferred to v0.4.4. v0.4.3 ships passing checks with `passed: false` set on failures but forensic fields (expected/actual/diff_byte_offset/decode_error) only populated at the one v0.4.1 J.7 proof-of-shape site.
- **Why deferred:** scope-safety in v0.4.3 release window. Full helper + refactor estimated at ~800-1000 lines deleted in verify_bundle.rs alongside ~70 push-site updates.
- **Tier:** `v0.4.4`

### `verify-bundle-multisig-helper-full-mode-unit-test` — add unit-level coverage for emit_multisig_checks full-mode ms1 branch

- **Surfaced:** v0.4.5 final cross-phase review I-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` helper_tests mod.
- **What:** v0.4.5 ships `helper_multisig_watch_only_emits_3plus6n_checks_in_spec_order` (renamed from `_full_` after review confirmed the fixture exercises watch-only synthesis with empty `expected.ms1`). The full-mode multisig ms1 branch (`emit_multisig_checks` lines ~1096-1159: substantive ms1_decode + ms1_entropy_match per cosigner) has end-to-end coverage via `cli_bundle_multisig.rs` integration tests but no isolated unit-level test. Add a companion `helper_multisig_full_emits_3plus6n_checks_in_spec_order` that uses `synthesize_multisig_full` (or constructs a synthetic Bundle with non-empty `expected.ms1` strings) to exercise the substantive ms1 path.
- **Why deferred:** integration coverage is sufficient for v0.4.5; the unit-level gap is test isolation hygiene, not behavior.
- **Status:** `resolved by v0.5.0 Phase B.1 (commit 9f1a4e7) — helper_multisig_full_emits_3plus6n_checks_in_spec_order added`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-positional-fallback-condition-cosmetic` — cosmetic dead `unwrap_or(false)` in card_for_cosigner positional fallback

- **Surfaced:** v0.4.5 final cross-phase review L-2 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` (`card_for_cosigner` positional fallback condition).
- **What:** Condition `supplied_md_decoded.is_err() || supplied_md_decoded.as_ref().map(|d| d.tlv.pubkeys.is_none()).unwrap_or(false)` — the `.map().unwrap_or(false)` chain is unreachable when `supplied_md_decoded.is_err()` short-circuits OR semantically dead inside the Ok branch. Refactor to `match` for clarity.
- **Why deferred:** cosmetic; no logic impact.
- **Status:** `resolved by v0.5.0 Phase B.2 (commit 9f1a4e7) — refactored to clean match expression`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-md1-xpub-match-set-equality` — md1_xpub_match uses ordered Vec equality

- **Surfaced:** v0.4.5 Phase P.4 review I-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` (md1_xpub_match arm).
- **What:** Helper compares `expected_md1.tlv.pubkeys` and `supplied_md1.tlv.pubkeys` as ordered `Vec<[u8; 65]>` via `==`. SPEC §5.7 line 103 says the shared `md1_xpub_match` confirms "all N pubkeys match expected" — semantics are arguably set-equality (the script-level pubkey set must be identical), not ordered. Template-mode synthesis preserves cosigner-index order, so ordered equality is correct for that path. Descriptor-mode verify-bundle (P.5) where the user supplies a descriptor with arbitrary `@N` placement could false-fail under ordered equality even when the logical pubkey set is identical.
- **Why deferred:** template-mode P.4 doesn't trigger this; descriptor-mode P.5 lands in v0.4.5 but the SPEC clarification needed to choose set-vs-ordered semantics is itself open. Re-evaluate after P.5 implementation surfaces real-world cases.
- **Status:** `resolved by v0.5.0 Phase B.3 (commit 9f1a4e7) — sort-then-compare multiset equality`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-cosigner-mapping-diagnostic` — distinguish "card not supplied" from "xpub not in policy"

- **Surfaced:** v0.4.5 Phase P.4 review I-2 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` (`card_for_cosigner` mapping + `mk1_decode[i]` emission).
- **What:** When supplied md1 decodes successfully and pubkeys-TLV is present but a supplied mk1 card's xpub matches no entry, `card_for_cosigner[i]` stays `None` and `mk1_decode[i]` emits "skipped: mk1[i] not supplied or decode failed". This conflates two distinct failure modes:
  1. User forgot to supply --mk1 for cosigner i.
  2. User supplied an mk1 card whose xpub doesn't appear in the descriptor's pubkey set (wrong-key attack scenario).
- **Why deferred:** diagnostic clarity, not correctness. Could split into two distinct check names or add a per-card "policy-membership" field.
- **Status:** `resolved by v0.5.0 Phase B.4 (commit 9f1a4e7) — MappingFailure enum with precedence XpubNotInPolicy > DecodeFailed > NotSupplied`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-missing-ms1-passes-true` — full-mode multisig with no --ms1 supplied reports passed=true

- **Surfaced:** v0.4.5 Phase P.4 review N-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` ("Expected substantive but supplied missing/empty" branch).
- **What:** When `expected.ms1[i]` is non-empty (full-mode) but the caller supplies no corresponding --ms1 value, `ms1_decode[i]` and `ms1_entropy_match[i]` are emitted with `passed: true, decode_error: "skipped: ms1[i] not supplied"`. A full-mode multisig bundle verified without supplying any ms1 cards thus reports `result: ok` if mk1+md1 match. SPEC §5.7 line 104 specifies "skipped: watch-only slot" semantics ONLY for `ms1[i] == ""` (watch-only sentinel); the missing-but-expected case is unspecified.
- **Why deferred:** policy decision — should missing-but-expected ms1 be a hard fail (like missing mk1[i])? Or stays as soft skip (current behavior)? Defer for SPEC clarification.
- **Status:** `resolved by v0.5.0 Phase B.5 (commit 9f1a4e7) — SPEC §5.7 four-case table, case 4 passed=false on missing-but-expected ms1`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-watch-only-spurious-ms1-handling` — watch-only with user-supplied --ms1 produces ms1_entropy_match: fail

- **Surfaced:** v0.4.5 Phase P.3 review L-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_watch_only` + `emit_verify_checks` watch-only short-circuit.
- **What:** Pre-v0.4.5 `watch_only_checks` ignored `args.ms1` (always emitted "watch-only mode: no entropy known to toolkit" passing-vacuously). Post-v0.4.5 P.3 wire-up: run_watch_only synthesizes the watch-only Bundle (`ms1: vec![""]`) and the helper compares supplied vs expected. If user spuriously supplies `--ms1 <non-empty>` in watch-only mode, `ms1_decode` runs against the supplied string, then `ms1_entropy_match` fails because `expected="" ≠ supplied=non-empty`. Behavior change vs v0.4.4: arguably more useful (tool flags the user's mistake) but not formally specified.
- **Why deferred:** non-blocking; SPEC §5.7 doesn't address this edge. Decide whether to short-circuit in run_watch_only (ignore args.ms1, force-empty SuppliedCards.ms1) or document the behavior in SPEC §2.2.2.
- **Status:** `resolved by v0.5.0 Phase C.2 (commit 4a650aa) — SPEC §5.7 case 1 codification + integration test`
- **Tier:** `v0.4-nice-to-have`

### `verify-bundle-helper-foundation-cleanup-v0.4.5` — 2 Low/Nit cleanups from v0.4.4 final cross-phase review

- **Surfaced:** v0.4.4 final cross-phase review 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_verify_checks` (and surrounding helper code).
- **What:**
  - **L-1** — Doc-comment in `emit_verify_checks` cites SPEC §5.8 for the watch-only sentinel discrimination (`expected.ms1[i].is_empty()`); the watch-only short-circuit logic actually lives in §5.7. §5.8 is the MsField wire-format definition. Fix: change `§5.8` → `§5.7` in the doc-comment near `verify_bundle.rs:1882`.
  - **L-2** — `MkField::Multi` arm in single-sig branch returns early with potentially fewer than 9 checks; this path is unreachable in production (single-sig bundles always have `MkField::Single`) and is documented with a comment, but the early return is an implicit invariant assumption. Fix: replace early return with `unreachable!("single-sig branch reached MkField::Multi — invariant violation")` or `debug_assert!(false, ...)`. Land alongside P.3 wiring in v0.4.5.
- **Why deferred:** non-blocking nits; helper is `#[allow(dead_code)]` so no runtime exposure. Bundle with the v0.4.5 P.3-P.7 call-site rollout.
- **Status:** `resolved by v0.4.5 Phase L (commit 40638c8)` — L-1 §5.7 cited; L-2 `unreachable!()` invariant assertion in place.
- **Tier:** `v0.4.5`

### `verify-bundle-helper-call-sites-rollout-v0.4.5` — Phase P.3-P.7 call-site rollout deferred from v0.4.4 to v0.4.5

- **Surfaced:** v0.4.4 Phase P scope decision 2026-05-06 (P.1+P.2 helper foundation landed; P.3-P.7 deferred for scope safety).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_full` + `run_multisig` + `descriptor_mode_verify_run` + `crates/mnemonic-toolkit/tests/watch_only_tests.rs` + new integration tests for full forensic rollout.
- **What:** v0.4.4 P.1+P.2 shipped `emit_verify_checks` (single-sig 9-check shape per SPEC §5.7 ordering), `SuppliedCards<'a>` struct, `emit_md1_checks` shared md1 helper, watch-only short-circuit (passed=true + decode_error="skipped: watch-only slot"), multisig TODO stub returning `[VerifyCheck { name: "TODO_multisig_v0_4_5", passed: false, ... }]`, and 4 helper unit tests. The helper is `#[allow(dead_code)]`; v0.4.5 wires it up:
  - **P.3** — `run_full` (single-sig template-mode) calls `emit_verify_checks(SuppliedCards::singlesig(...), false)` and replaces ~30 push sites.
  - **P.4** — `run_multisig` (template-mode multisig) replaces TODO stub with the 3-shared-checks + 6N-per-cosigner pattern; emits real forensics.
  - **P.5** — `descriptor_mode_verify_run` emits the 9 / 3+6N schema (closes `verify-bundle-9-3plus6n-descriptor-mode-parity`) via the helper.
  - **P.6** — `watch_only_tests.rs` migrates to the new shape (`passed` + forensic field assertions).
  - **P.7** — Add integration tests for full forensic field population: tampered-cell roundtrips that assert `expected`/`actual`/`diff_byte_offset` populated; skipped checks assert `decode_error` populated.
- **Why deferred:** scope-safety in v0.4.4 release window. The helper-foundation pattern is the right shape; consolidating ~78 call sites at the same time was estimated at ~800-1000 lines deleted plus ~70 push-site updates and risked release timeline.
- **Status:** `resolved by v0.4.5 commits 679ded7 (P.3+P.6) + d3207dd (P.4) + 57f62eb (P.5) + 40638c8 (L+P.7)` — all 5 sub-phases shipped; net cmd/verify_bundle.rs delete ~660 lines; 3 forensic integration tests added.
- **Tier:** `v0.4.5`

### `verify-bundle-emit-checks-helper-and-full-forensics-rollout` — Phase J.2 + J.3 + full forensic field rollout deferred from v0.4.1 to v0.4.2 — SUPERSEDED

- **Status:** `superseded by verify-bundle-helper-and-full-forensics-rollout-v0.4.4 (2026-05-06)`. v0.4.3 P.0 landed the struct shape correction; the helper + full rollout deferred again to v0.4.4.

- **Surfaced:** v0.4.1 Phase J scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (~78 VerifyCheck push sites) + new `emit_verify_checks` helper.
- **What:** v0.4.1 ships the structural pieces of SPEC §5.7: VerifyCheck struct gains `expected` / `actual` / `diff_byte_offset` / `decode_error` Option fields with Default impl + serde skip_serializing_if (J.1), and the `--ms1` CLI repeating-flag migration (J.5). Forensic fields are populated on ONE prominent failure path (descriptor-mode `ms1_entropy_match` mismatch — proof-of-shape in cmd/verify_bundle.rs:1456-1469); the remaining ~70 push sites continue to default to `None` for forensic fields. The `emit_verify_checks` helper (J.2) and the run_full / run_multisig / descriptor_mode_verify_run refactor (J.3) to use it are deferred. Full per-cell forensics rollout requires the helper to land first; otherwise duplicating the population logic at every push site is unmaintainable.
- **Why deferred:** scope-safety in v0.4.1 release window. The 78-site refactor is mechanical but error-prone; helper-first approach is the right shape and lands cleanly in v0.4.2.
- **Status:** `open`
- **Tier:** `v0.4.2`

### `verify-bundle-9-3plus6n-descriptor-mode-parity` — Phase G/J descriptor-mode 9/3+6N parity deferred from v0.4.1 to v0.4.2

- **Surfaced:** v0.4.1 Phase J scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::descriptor_mode_verify_run`.
- **What:** SPEC §5.7 specifies descriptor-mode verify-bundle emits the same 9 / 3+6N check schema as template-mode (replacing v0.3's 3-element coarse ladder). v0.4.1 retains the v0.3 coarse ladder (cmd/verify_bundle.rs:1361 onward) with the H.1 shim for the schema-4 ms1 vec. v0.4.2 lands the parity refactor atomically with the `emit_verify_checks` helper (FOLLOWUP `verify-bundle-emit-checks-helper-and-full-forensics-rollout`).
- **Why deferred:** depends on the helper; bundled with the same v0.4.2 cycle.
- **Status:** `resolved by v0.4.5 Phase P.5 (commit 57f62eb)` — descriptor_mode_verify_run dispatches to emit_verify_checks(... is_multisig: descriptor.n > 1); single-sig descriptors emit the 9 schema, multisig descriptors emit 3+6N.
- **Tier:** `v0.4.2`

### `legacy-cli-flag-deletion` — delete --phrase / --xpub / --cosigner / --master-fingerprint / --cosigner-count / --cosigners-file CLI flags entirely

- **Surfaced:** v0.4.2 cycle planning 2026-05-06 (user-confirmed during scope brainstorm).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::BundleArgs` + `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::VerifyBundleArgs`; consumer test files under `crates/mnemonic-toolkit/tests/`.
- **What:** v0.4.2 lands the unified `--slot @N.<subkey>=<value>` dispatch and routes legacy CLI flags through `expand_legacy_to_slots` (option (a) per the v0.4.2 brainstorm). v0.5 takes the next step: delete the legacy CLI flags entirely from `BundleArgs` + `VerifyBundleArgs`. Estimated cost: rewrite ~25 integration tests (~1500 lines of test churn) to use `--slot` syntax. The unified path itself is unchanged; only the CLI surface contracts.
- **Why deferred:** the user accepted the bigger v0.4.2 scope (legacy-flag-deprecation under option a) but routes the cleaner-CLI-surface end-state to v0.5 to amortize the test-rewrite churn against a separate cycle. Captured as a follow-on after v0.4.2 ships.
- **Status:** `resolved by v0.5.1 commit d782a2d` — 6 legacy fields deleted from both `BundleArgs` and `VerifyBundleArgs`; `bundle_args_to_slots` + `expand_legacy_to_slots` shims deleted; 9 mode-violation guards + 11 mode-text consts removed; 3 retained guards covered by new `cli_mode_violations_v0_5.rs`. `bundle::resolve_slots` refactored to take an explicit args-tuple + promoted to `pub(crate)`; `verify_bundle.rs` dispatch reshaped to consume slots. 13 consumer test files rewritten per the v0.5.0 mapping table.
- **Tier:** `v0.5.1`

### `engraving-card-unified-legacy-migration` — migrate 4 legacy engraving_card() call sites to engraving_card_unified

- **Surfaced:** v0.4.1 Phase I scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` 4 legacy call sites (bundle_full, bundle_watch_only, bundle_multisig_full, bundle_multisig_watch_only) + `crates/mnemonic-toolkit/src/format.rs` legacy `engraving_card` + `EngravingMode` enum.
- **What:** v0.4.1 ships `engraving_card_unified` + `BundleInputForCard` per SPEC §5.5 and wires only the new `bundle_run_unified` (--slot-driven) path through it. Migrating the 4 legacy call sites to the unified card requires removing 3 byte-exact format.rs unit tests for `EngravingMode::*` variants and verifying integration tests still pass with the new card layout. v0.4.2 lands the migration + drops `EngravingMode`.
- **Why deferred:** scope-safety in v0.4.1 release window; legacy call sites work unchanged via the existing `engraving_card` function.
- **Status:** `resolved by v0.5.0 Phase A.3 (commit 456c878) — BundleJson.engraving_card field deleted; doc-comment rewritten`
- **Tier:** `v0.4.2`

### `unified-slot-xpub-missing-path-origin-path-null` — origin_path empty-string vs null divergence

- **Surfaced:** v0.4.1 Phase H r1 review L-1.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots` (xpub branch) + `emit_unified` (single-sig N=1 origin_path emission).
- **What:** When `--slot @0.xpub=X` is supplied without `--slot @0.path=`, `emit_unified` emits `"origin_path": ""` in the JSON envelope. Legacy `emit` for the equivalent `--xpub X` (no path) invocation emits `"origin_path": null`. SPEC §4.11.b defines `""` as the absent-path sentinel for collision purposes but does not govern the JSON envelope value. Two paths diverge for semantically equivalent inputs.
- **Why deferred:** non-blocking; tooling that reads the envelope can treat `""` and `null` as equivalent. v0.4.2 unifies emission to `null`.
- **Status:** `resolved by v0.5.0 Phase E (commit 990ccad) — origin_path_for_json helper emits null on empty path_raw`
- **Tier:** `v0.4.2-nice-to-have`

### `unified-slot-additional-subkey-shapes` — entropy / xprv / wif / partial-xpub-only resolution deferred from v0.4.1 to v0.4.2

- **Surfaced:** v0.4.1 Phase H.5 scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots`.
- **What:** v0.4.1's unified `--slot` dispatch (`bundle_run_unified`) supports two slot subkey shapes: `{phrase}` (BIP-39 → derived xpub) and `{xpub, fingerprint, path}` (watch-only with full origin metadata). The remaining SPEC §6.6.b shapes (`{entropy}` raw entropy → ms-codec ENTR; `{xprv}` xpriv-direct; `{wif}` degenerate single-key; `{xpub}` alone; `{xpub, fingerprint}`; `{xpub, path}`) return BadInput with a pointer to this FOLLOWUP. v0.4.2 lands the resolution logic for each shape + integration tests per shape.
- **Why deferred:** scope-safety in v0.4.1 release window; the two supported shapes cover the headline multi-source-secrets and watch-only-multisig use cases.
- **Status:** `open`
- **Tier:** `v0.4.2`

### `unified-slot-descriptor-mode-support` — descriptor mode under unified --slot dispatch deferred from v0.4.1 to v0.4.2

- **Surfaced:** v0.4.1 Phase H.5 scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_run_unified`.
- **What:** v0.4.1's unified `--slot` dispatch supports `--template` only; supplying `--descriptor` alongside `--slot` is rejected with a pointer to this FOLLOWUP. Legacy descriptor-mode dispatch (no `--slot`) continues to work via `descriptor_mode_run`. v0.4.2 unifies the two paths so `--slot` works with both `--template` and `--descriptor`, including descriptor-mode multi-source via per-`@N` slot binding.
- **Why deferred:** scope-safety; the legacy descriptor-mode path remains the recommended invocation for descriptor-driven workflows in v0.4.1.
- **Status:** `open`
- **Tier:** `v0.4.2`

### `descriptor-binding-entropy-field-redundant` — DescriptorBinding.entropy field is redundant after v0.4.3 N

- **Surfaced:** v0.4.3 Phase N (CosignerKeyInfo type alias merge) 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/parse_descriptor.rs::DescriptorBinding`.
- **What:** v0.4.3 N merged CosignerKeyInfo into ResolvedSlot via type alias; ResolvedSlot has per-slot `entropy: Option<Vec<u8>>`. The bundle-level `DescriptorBinding.entropy: Option<Vec<u8>>` field is now semantically redundant with `binding.cosigners[0].entropy`. v0.4.4 retires the field; ~10 call sites (parse_descriptor.rs tests, verify_bundle.rs, bundle.rs::bundle_run_unified_descriptor) update to read `binding.cosigners[0].entropy.as_deref()` instead.
- **Why deferred:** non-blocking; harmless redundancy.
- **Status:** `resolved by v0.4.4 Phase S (commit c99a78b)` — DescriptorBinding.entropy field deleted; `entropy_at_0()` helper method (Option<&[u8]>) reads `cosigners[0].entropy`; bind_full_mode sets `cosigners[0].entropy` before construction; all readers migrated; 244 tests pass.
- **Tier:** `v0.4.4`

### `bundle-json-cli-flag-and-dispatch` — `--bundle-json <file>` verify-bundle intake + schema-version dispatch

- **Surfaced:** v0.4.1 Phase J.4 scope decision 2026-05-05 (per impl plan r1 review I2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::VerifyBundleArgs` + new JSON-intake handler.
- **What:** SPEC §6.7 reserves `--bundle-json <file>` as a verify-bundle flag for round-tripping a `bundle --json` envelope. v0.4.3 added the CLI flag + the `serde_json::Value` peek-then-typed-decode dispatch on `schema_version` (schema-4 only; schema-2/3 retro-compat tracked at NEW FOLLOWUP `bundle-json-schema-2-3-retro-compat` at v0.4.4+).
- **Status:** `resolved by v0.4.3 Phase Q (commit pending)` — clap flag with `conflicts_with_all = ["ms1", "mk1", "md1"]`; `load_bundle_json_into_args` synthesizes a VerifyBundleArgs with extracted card vecs; rest of run() unchanged. 3 integration tests in `cli_bundle_json_intake.rs`.
- **Tier:** `v0.4.2` (target met)

### `cosigner-keyinfo-resolved-slot-merge` — retire CosignerKeyInfo into ResolvedSlot

- **Surfaced:** v0.4.1 Phase H.6 (impl plan r1 review I1).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs::CosignerKeyInfo` + `ResolvedSlot`.
- **What:** v0.4.1 carried two near-identical typed shapes; v0.4.3 N merged them via `pub type CosignerKeyInfo = ResolvedSlot;` alias. ResolvedSlot is now the sole binding type. CosignerKeyInfo retained as a #[allow(dead_code)] alias for source-compat.
- **Status:** `resolved by v0.4.3 Phase N (commit 25581f3)` — type alias merge; per-slot entropy lives on ResolvedSlot; legacy DescriptorBinding.entropy field retained but redundant (tracked at NEW FOLLOWUP `descriptor-binding-entropy-field-redundant` at v0.4.4).
- **Tier:** `v0.4.2` (target met)

### `bundle-json-schema-2-3-retro-compat` — `--bundle-json` schema-2/3 retro-compat intake

- **Surfaced:** v0.4.3 Phase Q scope decision 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::load_bundle_json_into_args`.
- **What:** v0.4.3 ships schema-4-only intake. Schema-2/3 envelopes (theoretical; no real-world bundles exist since v0.4.1) error with byte-exact stderr pointing at this FOLLOWUP. v0.4.4+ adds schema-2/3 typed dispatch IF a real-world need surfaces.
- **Why deferred:** speculative; no real bundles to consume.
- **Status:** `resolved by v0.5.0 Phase D (commit 6e4b87e) — placeholder rejection branch deleted; schema-mismatch fails at field extraction`
- **Tier:** `v0.4.4-nice-to-have`

### `wif-multisig-resolution` — wif slots in multisig contexts

- **Surfaced:** v0.4.2 Phase K.3 (single-sig-only guard introduced; multisig deferred).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots`.
- **What:** v0.4.3 R lifted the single-sig-only guard. Wif slots in multisig produce ResolvedSlots with the wif's pubkey + zero chain code + empty path. BIP-388 distinctness applies normally (same WIF twice → row 13 collision).
- **Status:** `resolved by v0.4.3 Phase R (commit 610bef6)` — 3 new integration tests cover hybrid 2-of-3 + pure 2-of-2 + same-WIF-twice collision.
- **Tier:** `v0.4.3` (target met)

### `legacy-flag-deprecation` — full migration of --phrase / --xpub / --cosigner to alias-only deferred from v0.4.1 to v0.5+

- **Surfaced:** v0.4.1 Phase H.5 scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::run` legacy dispatch path.
- **What:** SPEC §9 v0.4 promises that legacy `--phrase` / `--xpub` / `--cosigner` flags become deprecation aliases that auto-expand into `--slot` form. v0.4.1 ships unified `--slot` as opt-in alongside the unchanged legacy dispatch. v0.5+ (a future BREAKING release) deletes the legacy dispatch entirely and routes everything through `bundle_run_unified` via `expand_legacy_to_slots`.
- **Why deferred:** would force fixture regeneration of 16+ v0.1 byte-exact fixture files + v0.2 carry-forward fixtures; too large for v0.4.1 release window.
- **Status:** `resolved by v0.5.1 commit d782a2d` — superseded by `legacy-cli-flag-deletion`. Legacy dispatch path is deleted entirely; `--slot` is the sole input shape.
- **Tier:** `v0.5.1`

### `bundle-removed-subcommand-trap-positional-eq-bypass` — `bundle multisig-full=value` token bypasses pre-clap trap

- **Surfaced:** v0.4 Phase 2 SPIKE r1 architect review L-2 (2026-05-05).
- **Where:** Phase C.1 `detect_removed_subcommand` (locked SPIKE shape at `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` SPIKE-2).
- **What:** Trap matches `argv[i+1] == "multisig-full"` with exact string equality. A token like `multisig-full=value` would not match and would fall through to clap's generic "unexpected argument" error rather than the byte-exact §6.6 row 1 message. Positional args do not idiomatically take `=value` form in shells, so this is essentially theoretical.
- **Why deferred:** no realistic user invocation produces this argv shape; a post-trap fallback in clap already rejects with exit 2.
- **Status:** `resolved by v0.5.0 Phase C.3 (commit 4a650aa) — entire detect_removed_subcommand trap deleted`
- **Tier:** `v0.4-nice-to-have`

### `bundle-removed-subcommand-trap-double-dash-bypass` — `mnemonic bundle -- multisig-full` bypasses pre-clap trap

- **Surfaced:** v0.4 Phase 2 SPIKE r1 architect review L-3 (2026-05-05).
- **Where:** Phase C.1 `detect_removed_subcommand` (locked SPIKE shape at `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` SPIKE-2).
- **What:** With a `--` separator inserted between `bundle` and `multisig-full`, the trap reads `argv[i+1] == "--"` and skips. Clap then processes `multisig-full` as a positional after `--` and emits a generic "unexpected argument" error rather than the byte-exact §6.6 row 1 text. UX difference matters only if a user intentionally inserts `--` before a removed subcommand name — not a realistic migration-error path.
- **Why deferred:** vanishingly unlikely user error; clap's fallback still rejects with exit 2.
- **Status:** `resolved by v0.5.0 Phase C.4 (commit 4a650aa) — entire detect_removed_subcommand trap deleted`
- **Tier:** `v0.4-nice-to-have`

### `tr-sortedmulti-a-via-upstream` — toolkit-side resolved in v0.3.1; v0.3.2 is the cleanup release

- **Surfaced:** v0.3 pre-Phase-A SPIKE 2026-05-05 (`design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §1).
- **Resolution timeline:**
  - 2026-04-03: rust-miniscript PR #910 ("Add support for sortedmulti_a") merged; closed issue #320.
  - 2026-04-04: PR #915 ("refactor: remove SortedMultiVec and use Terminal::SortedMulti") merged.
  - 2026-05-05: upstream search confirmed both PRs on master rev `95fdd1c5773bd918c574d2225787973f63e16a66`; no published crate release contains them.
  - 2026-05-05: v0.3.1 adopted via `[patch.crates-io] miniscript = { git = ..., rev = "95fdd1c..." }` after a read-only build experiment confirmed feasibility; walker refactored for the post-#915 API; SPEC §4.9.a Layer 1+2 patched; new `Terminal::SortedMulti` + `Terminal::SortedMultiA` arms added; wire-bit-identical regression test passes (descriptor-mode `tr(@0, sortedmulti_a(...))` md1 == template-mode `--template tr-sortedmulti-a` md1).
- **Where:** `crates/mnemonic-toolkit/Cargo.toml` (`[patch.crates-io]` entry); `crates/mnemonic-toolkit/src/parse_descriptor.rs` (walker arms); `descriptor-mnemonic/crates/md-cli/src/parse/template.rs` (md-cli still pre-#910 — separate FOLLOWUP `walker-backport-to-md-cli`).
- **Toolkit-side status:** `partially resolved by v0.3.1` — `tr(K, sortedmulti_a(...))` works end-to-end via the master `[patch]`. md-cli divergence is the remaining cross-repo concern (FOLLOWUP `walker-backport-to-md-cli`).
- **v0.3.2 cleanup release** (mechanical, when miniscript crates.io publishes a post-#910+#915 release):
  1. Drop the `[patch.crates-io]` entry from `Cargo.toml`.
  2. Bump `miniscript` version in `crates/mnemonic-toolkit/Cargo.toml` to the new release.
  3. Update CHANGELOG; tag `mnemonic-toolkit-v0.3.2`.
  4. No code, SPEC, or test changes expected — the patched master and the new published release should be wire-identical for the surface this toolkit uses.
  5. Watch via `gh api repos/rust-bitcoin/rust-miniscript/tags --jq '.[].name' | grep -E 'miniscript-(13\.[1-9]|14|15)'`.
- **Status:** `partially resolved by v0.3.1; v0.3.2 cleanup pending miniscript crates.io release`
- **Tier:** `v0.3.2` (toolkit-side; was `v0.4-cross-repo` until v0.3.1 shipped)

### `secret-on-stdout-warning-bundle-retrofit` — apply convert's §7 secret-on-stdout warning to bundle

- **Surfaced:** v0.6.0 SPEC architect review r1 C-2 + impl decision 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` emit_unified ms1 emission paths.
- **What:** v0.6.0 introduces a stderr warning `"warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')"` when convert emits secret-bearing material to stdout (phrase/entropy/xprv/wif/ms1). The bundle subcommand also emits secret-bearing ms1 strings to stdout but does NOT have the warning. Retrofit for cross-tool consistency.
- **Why deferred:** convert was the natural place to introduce the convention (ad-hoc one-shot operations where stdout-redirect-discipline is most likely overlooked); bundle retrofit is a separate scope-bounded change.
- **Status:** `resolved 66ff7c0` (v0.6.1 Phase D — `bundle.rs::emit_unified` emits the warning when `Bundle::any_secret_bearing()` returns true; SPEC §5.5.a; +1 positive (text mode) +1 positive (JSON mode) +2 negative (watch-only single + multisig) test assertions).
- **Tier:** `v0.6.1`

### `convert-seed-and-raw-privkey-nodes` — add seed / raw_privkey / xprv-via-ms1 / seed-via-ms1 nodes to convert when ms-codec v0.2 ships

- **Surfaced:** v0.6.0 SPEC §1 deferral 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType` + edge table + SPEC §1.
- **What:** ms-codec v0.1.0's `SEED`, `XPRV`, `PRVK` tags are `RESERVED_NOT_EMITTED_V01`. v0.6.0 SPEC §1 documents `seed` and `raw_privkey` as deferred-not-rejected nodes. When ms-codec ships v0.2 with the reserved tags activated, add these nodes + their edges to convert (and update SPEC §1 / §2 accordingly).
- **Why deferred:** upstream codec library limit; additive.
- **Status:** `open`
- **Tier:** `cross-repo`

### `convert-phrase-to-leaf-wif` — implement phrase/entropy → wif (path-to-leaf-WIF derivation)

- **Surfaced:** v0.6.0 SPEC §10 deferral 2026-05-06 + impl r1 review.
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` Phrase|Entropy arm.
- **What:** v0.6.0 SPEC §2 lists `phrase/entropy → wif` as not directly defined; impl returns `BadInput` with deferral message. Implementing requires a leaf-depth BIP-32 path (`m/<purpose>'/<coin>'/<account>'/<chain>/<index>`, depth 5) and serializing the leaf privkey to WIF. v0.6.1+ adds the missing edge.
- **Why deferred:** scope-safety in v0.6.0; the headline conversion graph nodes were prioritized.
- **Status:** `resolved 62b4f23` (v0.6.1 Phase B — SPEC-A in `SPEC_convert_v0_6.md` §2 + §8; sibling helper `derive_slot::derive_bip32_at_path` for path-driven derivation; `bitcoin::PrivateKey { compressed: true, network, inner: leaf_xpriv.private_key }.to_wif()`; explicit `--path` REQUIRED with byte-exact `ConvertRefusal` stderr (exit 2) when absent; `edge_uses_pbkdf2` extended to include `Wif` so `--passphrase` does not spuriously fire the ignored-warning).
- **Tier:** `v0.6.1`

### `convert-slip0132-prefix-support` — accept zpub/ypub on input + emit modes (consolidated v0.6.1)

- **Surfaced:** v0.6.0 post-release UX audit 2026-05-06 (user prompt about SLIP-0132 prefix interpretation).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` (input normalization + new edge); possibly cross-cutting into `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots` Xpub branch (if input normalization is repo-wide rather than convert-only).
- **What:** SLIP-0132 (`ypub`/`Ypub`/`zpub`/`Zpub` mainnet, plus `tpub`/`upub`/`vpub`/`Upub`/`Vpub` testnet) extended-key prefixes encode the intended script type (BIP-49 single/multi, BIP-84 single/multi) in the version bytes. Bitcoin Core + rust-miniscript + BIP-388 wallet policies reject non-`xpub` prefixes; the canonical modern path is descriptor-native xpub + descriptor wrapper. v0.6.0 currently fails at `Xpub::from_str` for a SLIP-0132 prefix. Both directions ship together in v0.6.1:

  **Permissive input (mechanical):** add a SLIP-0132 → xpub normalizer (`src/slip0132.rs` helper or inline). On input, detect non-`xpub` prefix; recompute version bytes to the matching `xpub`/`tpub` neutral prefix and re-base58check. The 78 payload bytes are byte-identical across SLIP-0132 variants, so no ECC work — pure prefix swap. Applies to:
    - `convert --from xpub=<zpub-string>`
    - `convert --from xpub=<ypub-string>` (etc.)
    - Cross-cutting: `bundle --slot @0.xpub=<zpub>` and `verify-bundle --slot @0.xpub=<zpub>` normalize identically for input symmetry across the toolkit.

  **Expressive output (design fork — resolve early in the cycle):** add output-side SLIP-0132 emission. Two grammar shapes to choose between via a SPEC amendment + one architect review round at the start of the v0.6.1 cycle:
    - (a) New target nodes `ypub`/`zpub`/`Ypub`/`Zpub` plus testnet `upub`/`vpub`/`Upub`/`Vpub`. Adds 8 nodes to NodeType. Edges: `xpub → ypub` etc. (pure prefix swap; no derivation).
    - (b) Existing `--to xpub` plus a `--xpub-prefix <neutral|y|z|Y|Z>` modifier flag. Single new flag; no new nodes.
   Option (b) is grammar-lighter and preserves the convention that SLIP-0132 variants are *encodings of the same xpub*, not different artifact classes. Lock the choice before implementation begins.

- **Why deferred:** v0.6.0 prioritized the headline single-format conversion graph; SLIP-0132 is a UX-convenience layer over BIP-32 + BIP-388 descriptors. Both directions ship together in v0.6.1 to close the SLIP-0132 story in one release cycle.
- **Status:** `resolved bb77164` (v0.6.1 Phase C — Option (b) selected per architect convergence: `--xpub-prefix <variant>` modifier flag with 5 case-sensitive values (`xpub`/`ypub`/`Ypub`/`zpub`/`Zpub`) per SPEC §11.a; testnet variants are network-context-derived via `--network` (no separate flag values); `--network` REQUIRED when `--xpub-prefix` is non-default. Input normalizer in new `src/slip0132.rs` handles all 8 SLIP-0132 prefixes (4 mainnet + 4 testnet); cross-cut wired at `convert.rs:515`, `bundle.rs:327`, `bundle.rs:853`. New `(xpub, xpub)` edge in §2 for the §11.a round-trip primitive).
- **Tier:** `v0.6.1`

### `convert-test-coverage-tightening` — close convert subcommand test gaps (6 direct-edge + 2 deferral + 3 round-trip tests)

- **Surfaced:** v0.6.0 post-release coverage audit 2026-05-06 (user-prompted enumeration of supported edges vs. test coverage).
- **Where:** `crates/mnemonic-toolkit/tests/cli_convert_happy_paths.rs`.
- **What:** v0.6.0 ships 23 convert tests covering 14 of 20 supported direct edges. Three coverage gaps to close in v0.6.1:
  1. **6 untested supported direct edges** — add at least one happy-path test each:
     - `phrase → ms1`
     - `entropy → xpub`
     - `entropy → xprv`
     - `entropy → fingerprint`
     - `xprv → fingerprint`
     - `wif → fingerprint`
  2. **2 deferral-message negative tests** — assert the v0.6.0 BadInput stderr ("not yet supported in v0.6 (path-to-leaf-WIF derivation deferred)") for:
     - `phrase → wif`
     - `entropy → wif`
     These tests pin the deferral text byte-exactly so the v0.6.1+ implementation of `convert-phrase-to-leaf-wif` will need to update them in lockstep (intentional: forces the deferral-→-implementation transition to be explicit).
  3. **3 explicit round-trip loop tests** (A→B→A) for the supported bidirectional pairs:
     - `phrase ↔ entropy` — assert `phrase → entropy → phrase` produces the canonical phrase byte-for-byte.
     - `entropy ↔ ms1` — assert `entropy → ms1 → entropy` produces identical entropy bytes.
     - `phrase ↔ ms1` (via entropy intermediate) — assert `phrase → ms1 → phrase` produces the canonical phrase. v0.6.0 has one-direction tests on each leg but no full-loop assertion.
- **Why deferred:** v0.6.0 prioritized headline-edge coverage and refusal-taxonomy correctness; the missing tests are tightening, not net-new functionality. The 6 uncovered edges are exercised indirectly through the JSON envelope test (#3 in `cli_convert_json.rs`) and the v0.5.2 16-cell parametric byte-identity test, but lack explicit asserts.
- **Status:** `resolved 59140c5` (v0.6.1 Phase E — 6 direct-edge tests added to `cli_convert_happy_paths.rs`; 3 round-trip loop tests added in new `cli_convert_round_trips.rs`. The 2 deferral-message tests are explicitly NOT written — Phase B (62b4f23) implemented `phrase/entropy → wif` so the deferrals no longer exist).
- **Tier:** `v0.6.1`

### `convert-run-step-numbering-duplicate-8` — `cmd::convert::run` has duplicate `// 8)` step labels

- **Surfaced:** Phase B code-reviewer r1 (Nit, deferred — predates Phase B).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs:382` and `:385`.
- **What:** The dispatch in `convert::run` numbers its steps `// 1)` through `// 9)`; both "Compute outputs" and "Emit" are labeled `// 8)`. The second should be `// 9)` to keep the comment numbering monotonic. Comment-only nit; no behavioral effect.
- **Why deferred:** Pre-existed Phase B; out of scope for the SPEC-A `phrase/entropy → wif` commit. Cleanly fixable in the next convert-touching patch.
- **Status:** `resolved a52b2aa` — v0.6.2 cosmetic micro-commit (Phase 5). `// 8) Emit.` → `// 9) Emit.`; `// 9) §7 secret-on-stdout warning.` → `// 10)`. Comment-only.
- **Tier:** `v0.6.2-nice-to-have`

### `slip0132-input-normalization-stderr-info` — emit a one-line stderr note when SLIP-0132 input is silently normalized

- **Surfaced:** v0.6.1 post-release UX discussion 2026-05-06.
- **Where:** new helper at `crates/mnemonic-toolkit/src/slip0132.rs`; emitter at the 3 production cross-cut sites (`convert.rs:515`, `bundle.rs:327`, `bundle.rs:853`).
- **What:** v0.6.1 Phase C silently normalizes SLIP-0132 prefix variants (zpub/ypub/Zpub/Ypub mainnet + uvpub/UVpub testnet) to neutral xpub/tpub on input. The user gets correct math but loses the intent signal their prefix carried (BIP-49 vs BIP-84, single-sig vs multisig). Add a one-line stderr informational note when normalization actually fires — pattern:
  ```
  info: normalized <variant> input to neutral <xpub|tpub> (encoding-only; no key change). Re-emit with --xpub-prefix <variant> if you need the SLIP-0132 form.
  ```
  Suppressed when input is already neutral. Quiet for users who already understand the normalization; informative for users discovering the round-trip primitive. The emitter must thread `&mut dyn Write` for stderr to all 3 cross-cut sites OR be implemented as an out-parameter on `normalize_xpub_prefix` so the caller decides where to write. Implementation tip: if `normalize_xpub_prefix` returns `Result<(String, Option<&'static str> /* variant-name */), ToolkitError>`, callers can match on `Some(_)` and emit per their stderr convention.
- **Why deferred:** v0.6.1 shipped the silent-normalization MVP intentionally (smaller blast radius; no new stderr bytes that could break byte-exact tests; Phase D's stderr-ordering invariant stays simple). UX-improvement work fits a v0.6.2 patch.
- **Caveat:** new stderr lines at the 3 cross-cut sites must NOT break the Phase D §5.5.a "secret-on-stdout warning is the LAST stderr write" invariant. Either fire the info note BEFORE the engraving card / before the secret-on-stdout warning, or relax the §5.5.a SPEC clause. Spike before SPEC-amending.
- **Status:** `resolved e4fedd7` — v0.6.2 lean cycle. SLIP-0132 input-normalization stderr info-line shipped; SPEC §5.5.a relaxation + multi-slot ordering + `--json` / `--no-engraving-card` independence locked. Phase 1 RED scaffold (`38c4272` + `740a917`), Phase 2 helper signature (`11c8edb` + `957db16`), Phase 3 emission (`e4fedd7` + `7bf1f1e` review-fix DRY refactor), Phase 4 SPEC + CHANGELOG (`39fa359` + `42561f3`), Phase 5 cosmetic step-numbering (`a52b2aa` + `96c2e3b`), Phase 6 release (`1fddf3b`).
- **Tier:** `v0.6.2`

### `slip0132-info-line-spec-text-not-byte-pinned` — SPEC §11 info-line wording isn't programmatically locked to the production format string

- **Surfaced:** v0.6.2 final cumulative review 2026-05-06.
- **Where:** `design/SPEC_convert_v0_6.md` §11 (canonical info-line paragraph); `crates/mnemonic-toolkit/src/slip0132.rs::render_slip0132_info_line` (production helper); `crates/mnemonic-toolkit/src/slip0132.rs::tests::render_slip0132_info_line_pins_canonical_text` (existing pin test, locks production ↔ slip0132 internal only).
- **What:** v0.6.2 introduced `render_slip0132_info_line(variant)` as the single production source of truth for the info-line text, with a unit test pinning the byte sequence for representative variants. The SPEC body in §11 carries the canonical text but as a templated example with `<variant>` and `<xpub|tpub>` placeholders. There is no test that asserts the SPEC body matches the production format-string structure. A future editor "improving" SPEC §11 prose (e.g., changing "Re-emit with" to "Re-encode with") would silently desync the SPEC from shipped behavior; CI catches nothing.
- **Why deferred:** v0.6.2 lean cycle scope; not a correctness bug. Test-side helpers in `tests/cli_*_slip0132_info.rs` provide bidirectional locking against production already (any production-text drift fails the integration tests), so the practical drift hazard is bounded — this entry is about catching SPEC-prose drift specifically.
- **Possible fix:** add a doc-test or unit test that grep-matches the SPEC §11 paragraph against a structural pattern, OR convert SPEC §11's example block into a fenced ```text block whose canonical form is read at test time. The first option is lower-overhead.
- **Status:** `resolved 354c945`
- **Resolution:** v0.7 Phase 7 — `slip0132::tests::spec_info_line_template_matches_production_render` reads `SPEC_convert_v0_6.md` §11 via `include_str!`, slices between HTML markers, and asserts byte-equality against `render_slip0132_info_line` for all 8 SLIP-0132 variants. SPEC↔production drift now CI-locked.
- **Tier:** `v0.7-nice-to-have`

### `verify-bundle-discards-slip0132-input-variant-asymmetry` — `verify-bundle` silently drops the SLIP-0132 input-normalization signal across 4 callsites

- **Surfaced:** v0.6.2 Phase 3 implementation (`e4fedd7`); confirmed in v0.6.2 final cumulative review 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:209`, `:261`, `:337`, `:407` — each callsite destructures `let (resolved, _slip0132_signals) = resolve_slots(...)?;` and discards the signal with the comment `// verify-bundle does not surface SLIP-0132 input-normalization signals.`
- **What:** v0.6.2 made `mnemonic convert` and `mnemonic bundle` emit a stderr informational line when a user-supplied SLIP-0132 prefix is silently normalized. `mnemonic verify-bundle` calls the same `pub(crate) resolve_slots` helper but discards the signal. Result: a user who pastes a `zpub` to `bundle` gets the info-line; pasting the same `zpub` to `verify-bundle` does not. The discard is semantically correct for v0.6.2's scope (verify-bundle is structurally a checker that emits check-pass/fail status, not a renderer of user inputs), but it creates a UX asymmetry within the toolkit.
- **Why deferred:** parity decision is its own UX policy question (does verify-bundle want to also emit the info-line? Or remain silent on stderr by design?). v0.6.2 lean cycle did not litigate this — the discard was the no-op-on-verify-bundle choice that minimized blast radius.
- **Possible fix (v0.7+ brainstorm):** decide whether `verify-bundle` should also emit the info-line for symmetry. If yes, thread `slip0132_signals` to a stderr emitter near each of the 4 callsites; SPEC §5.5.a's stderr-ordering invariant applies (notes precede any conditional warnings). If no, document the asymmetry intentionally in `SPEC_convert_v0_6.md` §11 / verify-bundle SPEC.
- **Status:** `resolved 354c945`
- **Resolution:** v0.7 Phase 7 — Option B locked per architect R1-I8. The 4 callsite-comments at `verify_bundle.rs:208/:261/:336/:406` gain a SPEC §11 v0.7 amendment cross-pointer; verify-bundle remains silent on SLIP-0132 input-normalization signals as intentional checker semantics. Zero new emission code.
- **Tier:** `v0.7-nice-to-have`

### `bip38-distinct-passphrase-flag` — split composite `(Phrase|Entropy, Bip38)` passphrase into two channels

- **Surfaced:** v0.7 Phase 1 code-quality review (commit `c3d0a85`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` composite arm + `convert::ConvertArgs` clap struct; SPEC §12.b reference.
- **What:** v0.7 ships dual-purpose `--passphrase` for composite paths flowing `phrase → wif → bip38` (or `entropy → wif → bip38`). One passphrase value drives both BIP-39 PBKDF2 mnemonic extension and BIP-38 Scrypt encryption. A user wanting distinct values must invoke `convert` twice. v0.8 may add `--bip38-passphrase` as a distinct flag so a single composite invocation can use different passphrases per layer. Implementation: thread the new flag through `compute_outputs`'s composite arms; if `--bip38-passphrase` is supplied, use it for the Scrypt step and use `--passphrase` (or `""` if absent) for the PBKDF2 step.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `--bip38-passphrase` flag added with locked R1-I3 semantics (composite arm: independent passphrases, no fallback, BREAKING change from v0.7's dual-purpose dispatch; direct arm: fallback to `--passphrase`). CHANGELOG `[0.8.0]` migration sentence pinned. SPEC v0.8 §12.b amendment.
- **Tier:** `v0.8`

### `bip38-encrypted-wif` — accept + emit BIP-38 passphrase-encrypted privkeys (`6P...`)

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `BipEncryptedWif` (or `Bip38`) `NodeType` variant in `convert.rs`; new edges from/to `wif`; SPEC §1 + §2 amendments.
- **What:** BIP-38 (`6P...`, base58check, 58 chars) is a passphrase-encrypted WIF format — widely used for paper wallets and key handoff. Two pieces: non-EC-multiplied form (encrypts an existing privkey under a passphrase via Scrypt) and EC-multiplied form (generates new privkey from passphrase + intermediate code; less common). Add as a new convert node so users can decrypt `6P → wif` (with `--passphrase`) and encrypt `wif → 6P` (with `--passphrase`); composite edges `phrase → 6P` follow naturally. Refusal class for `6P → 6P` and any cross-format pivot.
- **Why deferred:** v0.6.x focused on BIP-39 + BIP-32 + SLIP-0132 graph completeness; BIP-38 is its own well-defined Scrypt-backed format and merits a dedicated phase. Implementation likely uses the `bip38` crate or hand-rolled Scrypt against `secp256k1` primitives already in the dep tree.
- **Status:** `resolved c3d0a85`
- **Resolution:** v0.7 Phase 1 — `Wif↔Bip38` edges + composite paths shipped via `bip38 = "1.1"` crate (Apache-2.0). Security review at `design/agent-reports/v0_7-phase-1-bip38-security-review.md`. SPEC §12.
- **Tier:** `v0.7`

### `casascius-mini-private-key` — accept Casascius mini-key (`S...`, 22/26/30 chars) on input

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `MiniKey` `NodeType` (or absorb into `Wif` arm with prefix-detect) in `convert.rs`; SPEC §1 + §2.
- **What:** Casascius mini private key is a compact base-58 alphabet encoding starting with capital `S` (22, 26, or 30 chars) used historically on physical Bitcoin coins. Format: `S` + N chars; SHA256 of the full string + `?` must hash to `0x00` prefix (typo-checksum). Decoding: SHA256 of the mini-key string yields a 32-byte privkey scalar. One-way edge `mini-key → wif` (encoding-only; no key change). No `wif → mini-key` (mini-key generation requires a search for the typo-checksum-passing string; not deterministic from a given privkey). Refusal class: encode direction is a `§3.b lossy compression barrier` (the typo-checksum embedded in the mini-key string is not recoverable from a raw privkey).
- **Why deferred:** small but distinct format with its own checksum spec (Casascius's typo-check rule); fits a v0.7 grab-bag of less-common formats.
- **Status:** `resolved 89d29ab`
- **Resolution:** v0.7 Phase 2 — `(MiniKey, Wif)` decode-only edge shipped; SHA256 self-checksum rule enforced; encode direction refused as one-way (§3.b lossy-compression barrier). SPEC §13.
- **Tier:** `v0.7`

### `bip85-deterministic-entropy` — derive child seeds from a BIP-32 master

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new top-level `mnemonic derive-child` subcommand OR new edge `xprv → entropy` (with `--bip85-path` modifier); SPEC §1 / new top-level SPEC.
- **What:** BIP-85 derives deterministic child entropy from a BIP-32 master xpriv via HMAC-SHA512 at the path `m/83696968'/<application>'/<index>'`. Use cases: managing many seeds from one master; per-application sub-seeds; password derivation; WIF derivation. Standard application codes per BIP-85: `39'` (BIP-39 entropy of length L words), `2'` (HD-seed), `32'` (xprv child), `128169'` (hex bytes), `707764'` (passwords). Output node depends on the application code: `entropy` for `39'`, `xprv` for `32'`, etc. Grammar lean: `mnemonic derive-child --from xprv=<master> --application <bip39|hd-seed|xprv|hex|password> --length <N> --index <N>` to keep convert's edge-table model untouched (BIP-85 is a *derivation* operation, not a *single-format conversion*).
- **Why deferred:** BIP-85 is a useful but narrow derivation utility; doesn't fit `convert`'s "single-format conversion" framing cleanly. Likely wants its own subcommand. SPEC question to resolve at brainstorm: subcommand vs. extending convert.
- **Status:** `resolved 965cc3e`
- **Resolution:** v0.7 Phase 6 — new `mnemonic derive-child` subcommand shipped with 6 in-scope applications (`bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`). RSA / RSA-GPG / DICE refused with v0.8 deferral stubs. New SPEC `design/SPEC_derive_child_v0_7.md`.
- **Tier:** `v0.7`

### `slip39-shamir-secret-sharing` — SLIP-39 Trezor Shamir backup format

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `Slip39` `NodeType` (or new top-level subcommand `mnemonic slip39`); SPEC additions or new SPEC document.
- **What:** SLIP-39 (Trezor's standard) is a K-of-N Shamir-secret-sharing scheme for BIP-39 entropy with its own wordlist (1024 words, distinct from BIP-39's 2048). Shares carry: identifier, iteration exponent, group threshold, group count, member threshold, member index, share value, checksum (Reed-Solomon). Two-level scheme: groups × shares-within-group. Used by Trezor Model T for its native backup. Edges: `entropy → slip39-shares` (split; takes `--group-threshold` + per-group `--member-threshold`); `slip39-shares → entropy` (combine; needs ≥ K shares). Composite via entropy intermediate: `phrase → slip39-shares` etc. The 1024-word SLIP-39 wordlist must be embedded.
- **Why deferred:** Largest single addition in the queue — SLIP-39 is essentially an alternative to BIP-39's wordlist + a Shamir layer. The toolkit's secret-material slot would gain a second-class citizen alongside BIP-39 entropy. Significant SPEC + impl work. Trezor's `python-shamir-mnemonic` library is the reference impl. Note: the planned `mnemonic-secret` v0.2 cycle (sibling repo) is shipping K-of-N share encoding for ms1 codex32 — that may obviate the need for SLIP-39 in this toolkit, depending on user priorities. Brainstorm should resolve "do we want SLIP-39 *and* ms1-shares, or just ms1-shares?" before any impl. v0.7 cycle resolved to defer (lib audit returned hand-roll-required; no maintained Rust crate). v0.8 cycle re-tiered to v1+: scope is too large for a v0.8 minor cycle alongside the locked BIP-38/BIP-85/Electrum/export-wallet menu, and ms1-shares may obviate.
- **Status:** `resolved 6a80343` — shipped as `mnemonic-toolkit-v0.13.0` (2026-05-14). New `mnemonic slip39 split/combine` CLI subcommands + ~2000 LOC hand-rolled SLIP-39 library + 443-LOC canonical manual chapter. Trezor SLIP-0039 K-of-N threshold splitter; bit-identical to `python-shamir-mnemonic@17fcce14`; cross-impl smoke recipe validated against `shamir-mnemonic 0.3.0`. SPEC accumulated 9 patches across the cycle. Status drift (stale `open` until 2026-05-16 Bucket 5 v1.0 drill) flagged by the drill methodology and corrected here. Closes the K-of-N gap that `seed-xor-coldcard-compat` (v0.12.0) deferred. See [[project-v0-13-0-slip39-closed]].
- **Tier:** `v0.13.0-feature` (re-tiered from `v1+`, shipped).
- **Companion:** [[seed-xor-coldcard-compat]] (the v0.12.0 cycle's all-or-nothing counterpart; closed at `mnemonic-toolkit-v0.12.0` tag `63b4503`; introduced the multi-secret-on-stdout advisory class that v0.13.0 parameterizes for K-of-N).

### `slip39-cli-extendable-flag` — surface `--extendable` toggle on `mnemonic slip39 split`

- **Surfaced:** v0.13.0 P1c-E.1 R0 Q1 resolution 2026-05-14; refiled at v0.13.0 P2.1 RED 2026-05-14 per plan `design/PLAN_v0_13_0_p2.md` §2.3 + §3.1.
- **Where:** `crates/mnemonic-toolkit/src/cmd/slip39.rs` (`Slip39SplitArgs` flag table + `run_split` forwarding); SPEC §2.2 split flag table; manual `docs/manual/src/40-cli-reference/41-mnemonic.md`.
- **What:** v0.13.0 P2 hardcodes `extendable=false` for both `slip39 split` and `slip39 combine` per P1c-E.1 R0 Q1. The library's `slip39_split` and `slip39_combine` already accept `extendable: bool` (verified at P1c-E.2 LOCK; SPEC §2.5 row 22 `ExtendableMismatch` already exists for the combine-time refusal). v0.14 adds a user-facing `--extendable` CLI flag on `split` and a combine-time validation that all parsed shares share the bit. Refusal class is already wired at the library level so this is purely a CLI-surface add + manual mirror.
- **Why deferred:** v0.13.0 priority is SLIP-39 K-of-N parity with Trezor's reference behavior (which defaults to `extendable=false`); adding a CLI flag at P2 would expand the user-facing surface beyond the SPEC §2.2 v0.13.0 contract. Library parameter is already plumbed so v0.14 is a small CLI-only delta.
- **Status:** `open`
- **Tier:** `v0.14-feature`
- **Companion:** [[slip39-shamir-secret-sharing]] (the parent feature; this is a v0.14 follow-on that surfaces a library parameter already shipped at v0.13.0).

### `electrum-native-seed-format` — Electrum seed wordlist + version-prefix checksum

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `ElectrumPhrase` `NodeType` in `convert.rs` (or distinct from BIP-39's `Phrase`); SPEC §1 + §2.
- **What:** Electrum's seed format is its own wordlist + checksum scheme distinct from BIP-39. The seed validates by HMAC-SHA512 of the phrase prefixed with `"Seed version"`; the resulting hash's hex prefix encodes the seed-type (`01` = standard, `100` = segwit, `101` = 2FA standard, `102` = 2FA segwit). Conversion: `electrum-phrase → entropy` (different mapping than BIP-39); `electrum-phrase → seed → master xpriv`. Edges symmetric to BIP-39's. Wordlist embedding required (Electrum English wordlist is similar to BIP-39's but differs).
- **Why deferred:** medium scope — own wordlist + checksum + seed-version dispatch. Used by Electrum users transitioning to / from BIP-39-based wallets. Less urgent than BIP-38 / BIP-85 because most Electrum users can re-derive into BIP-39 via the wallet. Brainstorm should weigh user demand.
- **Status:** `resolved 892139c`
- **Resolution:** v0.7 Phase 3 — `ElectrumPhrase ↔ Entropy` edges shipped with 4-version HMAC-SHA512 prefix dispatch (`01`/`100`/`101`/`102`); 2FA versions (`101`/`102`) refused. Corpus spike at `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`. SPEC §14.
- **Tier:** `v0.7`

### `miniscript-beyond-bip388` — accept full miniscript policies beyond BIP-388's descriptor-template subset

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_run_unified_descriptor`; descriptor-input handling in `parse_descriptor.rs`; new SPEC §4.12 (v0.19.0 cycle); plan-doc at `design/PLAN_v0_19_0_non_canonical_descriptors.md`.
- **What:** v0.5+ accepts BIP-388-conformant descriptors (placeholder-template form `wpkh(@0/<0;1>/*)`, multipath, sortedmulti, etc.). The full miniscript language has many additional policies the toolkit doesn't surface as supported wallet types: `andor`, `thresh`, `pk_h`, time-locked branches via `older` / `after`, hash-locked `hash160` / `sha256` / `ripemd160`, `multi_a` (taproot multi without sortedness), arbitrary `tr` taproot trees with multi-leaf miniscript. Rust-miniscript supports parsing these; the gap is the toolkit's wallet-policy validation and the engraving-card / verify-bundle UX.
- **v0.19.0 cycle scope (locked 2026-05-16):** plan-doc V6 converged 0C/0I across opus R0-R5 + user-direction Q1-reversal at R4. Locked design: (Q1) silent default-path inference `m/48'/<coin>'/<account>'/2'` (BIP-48 cosigner path) for non-canonical wsh/sh-wsh/tr wrappers; (Q2) `tr(NUMS, <ms>)` sentinel substitution for script-path-only P2TR wallets; (Q3) trust rust-miniscript for fragment validity (toolkit gates wrapper class + per-`@N` origin coverage + slot grammar); (Q4) GUI Option-A inline `conditional.rs::bundle()` rules including canonicity-aware override of the existing `--account → pin_value(0)` rule. Lockstep release `mnemonic-toolkit-v0.19.0` + `mnemonic-gui-v0.8.0`.
- **Status:** `resolved 087d0e4` — shipped at `mnemonic-toolkit-v0.19.0` (2026-05-17) + `mnemonic-toolkit-v0.19.1` patch (`d4e7935`; clippy collapsible-if fix); lockstep `mnemonic-gui-v0.8.0` (`10a1abd`). Phases 1-7 across both repos converged 0C/0I. End-of-cycle Phase 7 opus review caught C1 (verify-bundle round-trip break for non-canonical default-inferred bundles); folded by mirroring bundle's canonicity-aware path-decl inference in `verify_bundle.rs::descriptor_mode_verify_run`. 3 new FOLLOWUPs filed at cycle close (`verify-bundle-multi-cosigner-mk1-chunk-assembly` pre-existing bug class deferred to v0.20-bugfix; `gui-schema-classify-descriptor-subcommand` diagnostic for drift gate, v0.20-feature; `gui-non-canonical-descriptor-banner-and-placeholder` GUI perceptibility patch, v0.8.1-gui-patch). User-run-the-feature smoke confirmed on user's flagship `wsh(andor(...))` invocation + `tr(NUMS, ...)` variant; stderr info notice byte-exact per SPEC §4.12.d.
- **Tier:** `v0.19-feature` (re-tiered from `v1+` per user direction 2026-05-16; shipped 2026-05-17).

### `vault-construction-covenant-based` — accept covenant-based vault descriptors (CTV / OP_CAT / OP_VAULT)

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** `parse_descriptor.rs` (descriptor parser extension); SPEC additions.
- **What:** Vault constructions use covenant opcodes (BIP-119 `OP_CHECKTEMPLATEVERIFY`, BIP-348 `OP_CAT` re-enable, BIP-345 `OP_VAULT`) to enforce spending paths beyond what current Bitcoin script allows: time-delayed spends, recovery paths, batch authorizations, etc. None of these opcodes are activated on mainnet today. When/if activated, vault descriptors become a wallet-type class distinct from current single-sig/multisig descriptors.
- **Why deferred:** **Gated on Bitcoin consensus activation.** No mainnet support today; signet test-cases exist for some of these. Re-evaluate when the relevant BIP advances to mainnet. The plumbing in this toolkit (descriptor parsing, mk1 xpub binding, md1 wallet-policy encoding) generalizes to vaults, so the impl gap is small once the script-side BIP activates — but speculative until then.
- **Status:** `open`
- **Tier:** `v1+`

### `address-derivation-from-xpub-path` — xpub + path + script-type → bitcoin address

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06; in-scope confirmed 2026-05-06 (SPEC §10 exclusion struck through, see `SPEC_convert_v0_6.md` §10 v0.6.1 amendment).
- **Where:** new edge `(xpub, address)` in `convert.rs::is_supported_direct_edge`; new `Address` `NodeType`; SPEC §1 + §2 amendments.
- **What:** Edge: `xpub` source + `--path` (or `--address-index N` + `--chain receive|change`) + script-type inferred from `--template` (or explicit `--script-type p2wpkh|p2sh-p2wpkh|p2tr|...`) → bech32 / bech32m / base58 address string. Composite from `phrase` / `entropy` via the existing BIP-32 derivation pipeline. Refusal classes: address → anything (one-way; addresses are hash160/SHA256 of pubkeys). Read-only display only — does NOT extend to PSBT / signing (PSBT remains out-of-scope per `bip174-psbt-signing` v1+).
- **Why deferred:** Useful but not blocking; v0.7 cycle slot. SPEC §10 amendment from "out of scope" to "in scope, deferred to v0.7" was committed alongside this entry update.
- **Status:** `resolved 940ec0b`
- **Resolution:** v0.7 Phase 4 — `(Xpub, Address)` edge shipped with `--path` mandatory + `--script-type` inferred from `--template` for BIP-44/49/84/86 → P2PKH/P2SH-P2WPKH/P2WPKH/P2TR. Composite paths via the existing BIP-32 derivation pipeline. SPEC §10.a.
- **Tier:** `v0.7`

### `bip327-musig2-collective-keys` — MuSig2 collective-key wallet-policy support

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** descriptor parser + new wallet-policy class.
- **What:** BIP-327 (MuSig2) defines a 2-round Schnorr multi-signature scheme that produces a single aggregated public key from N participants. Wallet-policy formats for MuSig2 collective keys are still maturing — there's no settled "BIP-388-equivalent" for MuSig2 wallets yet. When the spec settles, add support for MuSig2 collective keys as a wallet-policy variant alongside multisig (sortedmulti) and single-sig.
- **Why deferred:** **Standards-maturity gate.** No settled wallet-policy spec; rust-miniscript support is partial; hardware-wallet vendor adoption is preliminary. Re-evaluate when the wallet-policy spec for MuSig2 stabilizes.
- **Status:** `open`
- **Tier:** `v1+`

### `bip174-psbt-signing` — Partially Signed Bitcoin Transactions support

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** would be an entirely new subcommand surface area.
- **What:** BIP-174 PSBT (Partially Signed Bitcoin Transactions) is the standard format for unsigned/partially-signed transactions exchanged between wallets and signers. Adding PSBT support would expand the toolkit from "key/wallet management" into "transaction signing" — a fundamentally different problem class.
- **Why deferred:** **Out of scope per `convert` SPEC §10:** "different problem class." This toolkit is explicitly about key/wallet info, not transaction signing. PSBT belongs in a distinct tool (or in a separate signer subcommand of this toolkit if scope is reframed at v1+).
- **Status:** `open`
- **Tier:** `v1+`

### `frost-threshold-keys` — FROST threshold signature scheme support

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new wallet-policy class; cryptographic primitive set extension.
- **What:** FROST (Flexible Round-Optimized Schnorr Threshold signatures) is a K-of-N threshold signature scheme that produces a single aggregated Schnorr signature without requiring a trusted dealer. Distinct from MuSig2 in that it's threshold (K-of-N) rather than n-of-n. Wallet-policy and key-aggregation formats are still being standardized.
- **Why deferred:** **Standards-maturity gate.** No settled wallet-policy spec; cryptographic primitives not yet in `bitcoin` / `secp256k1` crates; hardware-wallet adoption preliminary. Re-evaluate when the spec stabilizes.
- **Status:** `open`
- **Tier:** `v1+`

### `liquid-confidential-extended-keys` — Liquid sidechain extended-key formats

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new network variant in `network.rs` (`Liquid` / `LiquidTestnet`); xpub/xprv version-byte table extension; SPEC §11 SLIP-0132 swap table additions.
- **What:** Elements/Liquid sidechain uses its own xpub/xprv version bytes plus blinding-key extensions for confidential transactions. Asset-blinding keys, value-blinding keys, and the address-blinding-key derivation (SLIP-0077) are Liquid-specific cryptographic primitives. Adding Liquid support would require: (a) network variant + version-byte table; (b) blinding-key derivation primitives; (c) Confidential Address derivation (different from mainnet bech32 addresses).
- **Why deferred:** Different chain context. Liquid is a federated sidechain with its own wallet ecosystem (Blockstream Green, Elements). The toolkit's primary user base is Bitcoin mainnet/testnet; Liquid users typically use Liquid-specific tooling. Re-evaluate if there's demand from cross-chain users.
- **Status:** `open`
- **Tier:** `v1+`

### `wallet-export-industry-formats` — `mnemonic export-wallet` (or `bundle --wallet-export <format>`) for Bitcoin Core / Sparrow / Specter / BIP-388 import

- **Surfaced:** v0.6.1 post-release UX discussion 2026-05-06.
- **Where:** new subcommand `mnemonic export-wallet` OR new flag on `bundle`; output formatters under `crates/mnemonic-toolkit/src/wallet_export.rs` (new module).
- **What:** Today the canonical "all wallet info, no secret" representation IS `mnemonic bundle --json` in watch-only mode (per SPEC §5.8: ms1 omitted-or-empty-sentinel; mk1 carries xpub bindings; md1 carries the descriptor/template). It is correct and complete BUT only the toolkit can re-ingest it. Users who want to feed the watch-only artifact to another wallet (Bitcoin Core, Sparrow, Specter, hardware-wallet HWI flows) must hand-translate. Add an industry-format export layer with at least:
  - **Bitcoin Core `importdescriptors` JSON** — `{"desc": "wpkh([fp/path]xpub.../{0,1}/*)#checksum", "active": true, "internal": false, "range": [0, 999], "timestamp": "now"}` per descriptor (one for receive, one for change; or `<0;1>` multipath split). Matches Bitcoin Core 25+ descriptor-wallet expectations.
  - **BIP-388 wallet policy** — formal `wallet_policy` JSON with `name`, `description_template`, `keys_info` array. Matches Ledger / hardware-wallet vendors that follow BIP-388.
  - **Sparrow / Specter wallet JSON** (optional; format is per-wallet). Lower priority — both can ingest output descriptors directly via the Bitcoin Core format.
  - **HWI signer JSON** (optional) — for cosigner export.
  Grammar lean: `mnemonic export-wallet --format <bitcoin-core|bip388|sparrow|specter> --output <path-or-->` with the same `--slot @N.<subkey>=<value>` input shape as `bundle`. Refuses if any slot supplies entropy/phrase (export-wallet is watch-only by definition). SPEC question to resolve at brainstorm: does this live as a new top-level subcommand OR a `bundle --wallet-export` flag? Lean: new subcommand because the input grammar is a strict subset of bundle (no entropy/phrase) and the output is a different wire format from `BundleJson`.
- **Why deferred:** v0.6.1 was a polish patch for `convert` + `bundle` UX. New subcommand or new bundle flag is its own minor scope. Brainstorm should resolve the format priority list (Bitcoin Core first vs BIP-388 first), the subcommand-vs-flag fork, and whether `range`/`timestamp` defaults need to be configurable.
- **Status:** `resolved 3821f66`
- **Resolution:** v0.7 Phase 5 — new `mnemonic export-wallet` subcommand shipped. Bitcoin Core `importdescriptors` JSON (default) + BIP-388 `wallet_policy` JSON. Sparrow / Specter formats stubbed (refuse with v0.8 deferral). `--range` / `--timestamp` / `--bitcoin-core-version` overrides. Watch-only enforced (refuses entropy/phrase slot input). New SPEC `design/SPEC_export_wallet_v0_7.md`.
- **Resolution-extended (v0.8.1 Phase 1):** Coldcard generic JSON skeleton (singlesig bip44/bip49/bip84) + Coldcard multisig text (wsh / sh-wsh, sorted and unsorted) + Blockstream Jade multisig text (byte-identical to Coldcard's, delegated emitter) shipped. New `wallet_export/{coldcard,jade}.rs`. `CliExportFormat::Coldcard` + `CliExportFormat::Jade` variants. `--wallet-name <STRING>` clap flag for formats publishing wallet names (Coldcard generic JSON, Sparrow / Specter / Electrum land in subsequent phases). New slot subkey `@N.master_xpub=` (depth-0 root xpub, optional, watch-only-class). Coverage now 2/8 → 4/8 of the SPEC §11 priority list; Sparrow/Specter/Electrum/Green land in Phases 2-5.
- **Resolution-extended (v0.8.1 Phase 5):** All six new vendor formats now shipped — `coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`. The complete v0.8.1 SPEC §11 priority list (8 formats: `bitcoin-core`, `bip388`, `coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`) is fully realized. New emitter modules: `wallet_export/{sparrow,specter,electrum,green}.rs`. New `CliExportFormat` variants: `Sparrow`, `Specter`, `Electrum`, `Green`. Per-format pinned byte-exact fixtures + SPEC §4 missing-info refusal channel exercised by Sparrow (Threshold) and Specter (WalletName). Status remains `resolved`.
- **Tier:** `v0.6.2`

### `coldcard-master-xpub-plumbing-pending` — `@N.master_xpub=` slot subkey parses but is dropped before reaching the Coldcard emitter

- **Surfaced:** v0.8.1 Phase 1 R1 reviewer-loop fold (I-2).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs::ResolvedSlot` + `crates/mnemonic-toolkit/src/wallet_export/mod.rs::EmitInputs` + `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs::emit_coldcard_generic_json`.
- **What:** SPEC §2 + §5.1 ship two normative claims: (a) the slot grammar accepts `@N.master_xpub=<base58>` (shipped in Phase 1.1 via `SlotSubkey::MasterXpub`); (b) the Coldcard generic-JSON top-level `xpub` field is emitted iff `@0.master_xpub=` was supplied. Phase 1 shipped (a) but not (b); Phase 1.9.R1 added a refuse-on-supply guard in `cmd::export_wallet::run` so the gap was not silent.
- **Status:** `resolved` (v0.8.2 plumbing cycle).
- **Resolution:** v0.8.2 follow-up — `ResolvedSlot` gained a `master_xpub: Option<Xpub>` field populated in the `{Xpub, ...}` arm of `resolve_slots` via `crate::slip0132::normalize_xpub_prefix` + `Xpub::from_str`. `EmitInputs` gained `master_xpub_at_0: Option<Xpub>` plumbed from `resolved_slots[0].master_xpub` in `cmd::export_wallet::run`. `emit_coldcard_generic_json` now emits the top-level `xpub` field conditionally (`Some(x) → x.to_string()`, `None → field omitted via `#[serde(skip_serializing_if = "Option::is_none")]`). The Phase 1.9.R1 refuse-on-supply guard at `cmd::export_wallet::run:182-197` was retired. New byte-exact fixture `tests/export_wallet/coldcard_generic_bip84_mainnet_with_master_xpub.json`; new test cells `cell_8_coldcard_master_xpub_plumbing_byte_exact` (supplied case) and `cell_9_coldcard_master_xpub_absent_omits_top_level_xpub` (absent case). All other resolution arms (Phrase / Entropy / Wif / synthesize-test-helper / verify-bundle-rebuild) set `master_xpub: None` since master_xpub semantically only exists on user-supplied watch-only xpub slots.
- **Tier:** `v0.8.2`

### `coldcard-bip86-generic-export-pending-firmware` — `--template bip86 --format coldcard` refuses (BIP-86 not in upstream schema)

- **Surfaced:** v0.8.1 Phase 1 (SPEC R1-I2 reviewer-loop fold).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs::emit_coldcard_generic_json`.
- **What:** Coldcard's canonical `generic-wallet-export.md` (upstream master) documents only `bip44` / `bip49` / `bip84` sub-objects. BIP-86 (P2TR singlesig) has no slot in the schema. The toolkit refuses `--template bip86 --format coldcard` with the SPEC §5.1 byte-exact pointer until Coldcard firmware extends the schema. Workaround: use `--format bitcoin-core` (descriptor passthrough) or `--format sparrow` (native P2TR support).
- **Status:** open (pending Coldcard firmware). Last upstream-checked **2026-05-12**: `gh api repos/Coldcard/firmware/contents/docs/generic-wallet-export.md` — no `bip86` / `p2tr` / `taproot` mentions. `releases/ChangeLog.md` — no taproot / schnorr / bip86 entries.
- **Tier:** `v1+`

### `coldcard-tr-multi-a-pending-firmware` — `--template tr-multi-a` / `tr-sortedmulti-a` refuses under `--format coldcard`

- **Surfaced:** v0.8.1 Phase 1.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs::emit_coldcard_multisig_text`.
- **What:** Coldcard's multisig text emitter ingests only `P2WSH` / `P2SH-P2WSH` / `P2SH` formats per the SPEC §5.2 `Format` field. Taproot-multisig (tr-multi-a / tr-sortedmulti-a) is not in the firmware's import surface. The toolkit refuses with a pointer at `--format bitcoin-core` (descriptor) / `--format sparrow` for taproot multisig watch-only setup. Companion: Jade has the same gap (`jade-tr-multi-a-pending-firmware` below).
- **Status:** open (pending Coldcard firmware taproot-multisig support). Last upstream-checked **2026-05-12**: `releases/ChangeLog.md` — no taproot / schnorr entries; firmware most recent commit `ca06dfd2` 2026-04-25 is unrelated regression fix.
- **Tier:** `v1+`

### `jade-tr-multi-a-pending-firmware` — `--template tr-multi-a` / `tr-sortedmulti-a` refuses under `--format jade`

- **Surfaced:** v0.8.1 Phase 1.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/jade.rs::JadeEmitter::emit`.
- **What:** Blockstream Jade's `register_multisig.multisig_file` accepts the Coldcard §5.2 multisig text shape; taproot multisig is not yet in that surface. The toolkit refuses with a pointer at `--format bitcoin-core` / `--format sparrow` for taproot multisig. Companion: `coldcard-tr-multi-a-pending-firmware` above (Jade shares the schema; once Coldcard ships, Jade follows).
- **Status:** open (pending Blockstream Jade firmware taproot-multisig support). Last upstream-checked **2026-05-12**: `Blockstream/Jade:CHANGELOG.md` — singlesig BIP-86 P2TR SHIPPED (`Add support for signing bip86 single-key p2tr inputs and for registering bip86 p2tr(key) descriptors`); taproot **multisig** (`multi_a` / `sortedmulti_a`) NOT yet shipped. Entry remains accurately open for the multisig case; singlesig P2TR is already a separate emitter path (Sparrow `tr(@0/**)`).
- **Tier:** `v1+`

### `electrum-non-latin-wordlists` — Electrum native seed format hard-codes the English wordlist

- **Surfaced:** v0.7 Phase 3 review (commit `69ac560`).
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs` (wordlist embedding + lookup).
- **What:** Electrum supports 9 wordlists across English, Japanese, Spanish, Chinese (simplified/traditional), Korean, Italian, Portuguese, Dutch, etc. v0.7 Phase 3 ships only the English wordlist; non-English Electrum users cannot decode their phrases through `mnemonic convert`. Add a `--language` parameter mirroring BIP-39 + bundle the additional embedded wordlists.
- **Status:** `resolved 5dc83eb` (v0.8 Phase 2).
- **Resolution:** v0.8 Phase 2 — embedded 4 non-English Electrum wordlists (zh-Hans, ja, pt, es) from `spesmilo/electrum` upstream commit `e1099925e30d91dd033815b512f00582a8795d25`. Plan correction noted: upstream Electrum has 5 total wordlists, not 9 (zh-Hant, German, French, Italian are NOT upstream). Separate `--electrum-language` flag distinct from `--language` (R1-I2 lock); `--electrum-language` wins on Electrum arms (R2-L2 lock). Portuguese is base-1626 (Monero copyright header); base-N arithmetic correctly parameterized.
- **Tier:** `v0.8`

### `electrum-encode-iteration-bound` — encode mining loop has no upper iteration cap

- **Surfaced:** v0.7 Phase 3 review (commit `69ac560`).
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs::encode_phrase` (or equivalent encode-from-entropy path).
- **What:** Electrum's encode direction iterates by incrementing a nonce until the HMAC-SHA512 prefix matches the requested SeedVersion (`01` standard / `100` segwit). The loop has no iteration bound; on adversarially-chosen entropy or for rare versions, the search may run unboundedly long. Add a sane upper bound (e.g., `2^24` iterations) with a byte-exact stderr refusal on exhaustion.
- **Status:** `resolved 5dc83eb` (v0.8 Phase 2).
- **Resolution:** v0.8 Phase 2 — `MAX_ENCODE_ITERATIONS = 1<<20` cap on `entropy_to_phrase` rejection-search loop. New `ElectrumError::EncodeIterationBoundExceeded` mapped to user-visible refusal.
- **Tier:** `v0.8`

### `electrum-version-info-stderr` — decode emits the detected SeedVersion silently

- **Surfaced:** v0.7 Phase 3 review (commit `69ac560`).
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs::decode_phrase` + `cmd::convert::run` electrum arm.
- **What:** When `mnemonic convert --from electrum-phrase=... --to entropy` decodes a phrase, the toolkit dispatches via the SeedVersion prefix (`01` / `100` / `101` / `102`) but does not surface which version it detected. Adding a stderr info-line (e.g., `info: Electrum SeedVersion=01 (standard)`) parallel to the SLIP-0132 input-normalization note (SPEC §11) would help users confirm the dispatch matches their wallet's expectation.
- **Status:** `resolved 5dc83eb` (v0.8 Phase 2).
- **Resolution:** v0.8 Phase 2 — `note: detected Electrum SeedVersion <01|100> (<standard|segwit>)` emitted to stderr on decode arms. `compute_outputs` extended to triple-tuple return surfacing the detected SeedVersion.
- **Tier:** `v0.8-nice-to-have`

### `tr-multi-a-tr-sortedmulti-a-export-wallet-support` — `mnemonic export-wallet` refuses taproot multisig templates

- **Surfaced:** v0.7 Phase 5 code-quality review (commit `f8369d3`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export.rs` (descriptor pipeline + taproot-multisig validator).
- **What:** `mnemonic export-wallet` refuses `--template tr-multi-a` and `--template tr-sortedmulti-a` at runtime with an error pointing at this v0.8 deferral. Reason: taproot multisig descriptors require an internal-key designation (NUMS point or shared key) plus the script-path tree; the export-wallet pipeline doesn't yet thread the internal-key choice through to Bitcoin Core / BIP-388 formatters. Single-leaf `tr` (BIP-86) IS supported.
- **Status:** `resolved 86647ca` (v0.8 Phase 3).
- **Resolution:** v0.8 Phase 3 — new `--taproot-internal-key <nums|@N>` flag designates the BIP-341 internal key. NUMS uses the canonical reference `50929b74...0ac0` x-only point; `@N` makes cosigner N the key-path key (removed from multi_a leaf set). v0.7 stub refusal replaced with flag-pointing message. Bounds-checked + n=1 degenerate refusal.
- **Tier:** `v0.8`

### `export-wallet-descriptor-bip388-interop` — `--descriptor` mode + `--format bip388` is refused

- **Surfaced:** v0.7 Phase 5 code-quality review (commit `f8369d3`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export.rs` (format dispatch + descriptor-mode validator).
- **What:** `mnemonic export-wallet --descriptor <user-supplied> --format bip388` is refused at runtime; `--descriptor` mode currently only supports `--format bitcoin-core`. Reason: user-supplied descriptors arrive as opaque strings; converting them to BIP-388 `wallet_policy` requires re-parsing into the placeholder-template form (`@0/<0;1>/*`), which the watch-only template-mode pipeline already does but the descriptor-mode pipeline skips.
- **Status:** `resolved 86647ca` (v0.8 Phase 3).
- **Resolution:** v0.8 Phase 3 — new `descriptor_to_bip388_wallet_policy` helper parses canonical descriptor via miniscript, iterates `iter_pk()` to collect `[fp/path]xpub` keys (stripping `/<0;1>/*`), strips `#checksum`, and replaces each full key-expression with `@N/**` placeholder via longest-first substitution. Refused for non-multipath descriptors.
- **Tier:** `v0.8`

### `bip85-rsa-rsa-gpg-dice-applications` — RSA / RSA-GPG / DICE BIP-85 applications deferred

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/bip85.rs` application dispatch + `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface.
- **What:** BIP-85 application codes `828365'` (RSA), `67797633'` (RSA-GPG), `89101'` (DICE) are refused with v0.8 deferral stubs. Reason: RSA derivation requires an RSA crate not currently in the dep tree; DICE is a niche application (deterministic dice-roll output) with limited demand. The 6 in-scope applications (`bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`) cover the primary use cases.
- **Status:** `split-resolved 1dde4dc` (v0.8 Phase 7); split into `bip85-dice-application` (resolved) + `bip85-rsa-rsa-gpg-applications` (re-tiered).
- **Resolution:** v0.8 Phase 7 — DICE shipped with BIP85-DRNG-SHAKE256 + rejection sampling per BIP-85 v1.3.0 §"DICE". Spec reference vector (`m/83696968'/89101'/6'/10'/0'` → `1,0,0,2,0,1,5,5,2,4`) pinned. New `--dice-sides` flag. New `sha3 = "0.10"` direct dep. RSA + RSA-GPG re-tiered to v0.9 / pending-rsa-crate-stability per Phase 6 SPIKE (`design/agent-reports/v0_8-phase-6-rsa-crate-security-review.md`): RUSTSEC-2023-0071 Marvin-attack timing sidechannel is **unpatched** (`patched = []`); rsa crate is in extended pre-release (`v0.10.0-rc.18`). Reopen criteria: rsa crate publishes patched stable release OR user requests with stated downstream use case.
- **Tier:** `v0.8` (DICE) / `v0.9` (RSA + RSA-GPG)

### `bip85-passphrase-protected-master` — `--from phrase=` + `--passphrase` direct path

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface.
- **What:** `mnemonic derive-child --from xprv=...` requires xprv input. A user with a passphrase-protected BIP-39 phrase must currently route through `mnemonic convert --from phrase=... --passphrase ... --to xprv` first, then pipe the xprv to `derive-child`. A direct `--from phrase=... --passphrase ...` path on `derive-child` would be more ergonomic.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `--from phrase=...` accepted; internal `phrase → seed → master xprv` (mainnet, BIP-85-network-agnostic) before BIP-85 derivation. New `--passphrase` for BIP-39 mnemonic extension.
- **Tier:** `v0.8-nice-to-have`

### `bip85-non-english-bip39-language-codes` — `--language` flag inert for BIP-39 application

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface + `bip85::derive_bip39`.
- **What:** `--language` is plumbed through clap on `mnemonic derive-child` but ignored for BIP-85's `bip39` application. BIP-85 supports 9 wordlists (`0'` English through `8'` Czech) for the BIP-39 application via the language-index sub-path component. v0.7 hardcodes English (`0'`).
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — new `resolve_bip85_language` maps `CliLanguage` → (BIP-85 path code, `bip39::Language`). 9 BIP-85-coded languages supported. Portuguese refused (no BIP-85 code assigned).
- **Tier:** `v0.8`

### `bip85-testnet-emission` — `--network` flag inert for hd-seed / xprv applications

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface + `bip85::derive_hd_seed` / `derive_xprv`.
- **What:** `--network` is plumbed through clap but unused. v0.7 hardcodes mainnet WIF / xprv emission for `--application hd-seed` and `--application xprv`. Testnet users must post-process via `mnemonic convert` to swap version bytes.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `format_hd_seed_wif` + `format_xprv_child` now take `NetworkKind` parameter. Testnet emits `c…` WIF / `tprv…` xprv. Driven by `--network` flag (default mainnet to match BIP-85 spec test vectors).
- **Tier:** `v0.8`

### `bip85-spec-prose-byte-formula-clarification` — SPEC §3 prose vs. worked-example formula mismatch

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `design/SPEC_derive_child_v0_7.md` §3 (BIP-39 byte slicing).
- **What:** SPEC §3 prose says BIP-39 byte slicing uses `2 * length_in_words / 3`; the worked examples (12 words → 16 bytes, 24 words → 32 bytes) match the correct formula `words * 4 / 3`. The two are equivalent for word counts divisible by 3 but the prose formula is not the canonical BIP-39 form. Pure SPEC text fix — implementation is correct.
- **Status:** `resolved 4dfea5a` (v0.8 Phase 0).
- **Resolution:** v0.8 Phase 0 — `2 * length_in_words / 3` → `length_in_words * 4 / 3` in SPEC §3.
- **Tier:** `v0.7-nice-to-have`

### `bip85-stdin-master-xprv` — `--from xprv=-` parses but does not read stdin

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs::run`.
- **What:** `mnemonic derive-child --from xprv=-` parses through clap (the `=-` sentinel is recognized) but `derive_child::run` does not read stdin to populate the xprv value. `mnemonic convert` does honor `=-`. Add stdin-read parity for cross-subcommand consistency.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `derive_child::run` now reads stdin when `args.from.value == "-"` via `crate::cmd::convert::read_stdin_to_string` (made `pub(crate)`). Works for both `xprv=-` and `phrase=-`.
- **Tier:** `v0.8`

### `derive-child-spec-2-grammar-uniformity-tension` — SPEC §2 prose-internal contradiction on `--length` mandatoriness

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `design/SPEC_derive_child_v0_7.md` §2.
- **What:** SPEC §2 has internal tension: it says `--length` is mandatory at clap level AND that `--length` is refused for `--application hd-seed` / `--application xprv`. Phase 6 implementation adopted a sentinel-0 convention: clap requires `--length`, and `hd-seed`/`xprv` arms refuse only when the supplied value is non-zero (`0` is treated as sentinel-absent). The SPEC text should be edited to reflect this; current prose reads as a contradiction.
- **Status:** `resolved 4dfea5a` (v0.8 Phase 0).
- **Resolution:** v0.8 Phase 0 — SPEC §2 + §4 prose updated to document the sentinel-0 convention canonically.
- **Tier:** `v0.7-nice-to-have`

### `bip38-ec-multiplied-encrypt-mode-support` — emit BIP-38 EC-multiplied form via intermediate codes

- **Surfaced:** v0.7.1 Phase 3 (BIP test vector audit cycle); rescoped from `bip38-ec-multiplied-mode-support` after Phase 3 forensics.
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` `(Wif, Bip38)` arm; `bip38 = "1.1"` crate.
- **What:** v0.7.1 supports BIP-38 EC-multiplied DECRYPT transparently (4 spec vectors pinned). ENCRYPT to EC-multiplied form requires the intermediate-code workflow per BIP-38 §"Generation of intermediate code": the passphrase owner generates a passphrase code; a third party combines it with random entropy to derive the encrypted privkey + the corresponding bitcoin address. Implementation: new subcommand `mnemonic intermediate-code` (or `--passphrase-code <code>` flag on the `(Wif, Bip38)` arm). Out of scope for v0.7.1 vectors-only audit.
- **Why deferred:** v0.8 Phase 4 SPIKE returned DEFER verdict (`design/agent-reports/v0_8-phase-4-bip38-ec-mult-encrypt-spike.md`). The `bip38 v1.1.1` `Generate` trait covers owner-only path only with internal `rand::thread_rng()` (non-deterministic) and exposes no intermediate-code workflow + no confirmation code. Hand-rolling spec-compliant API costs ~155 LOC of cryptographic code (AES + scrypt + secp256k1 + Unicode normalization). Marginal user value (paper-wallet niche). Re-tiered to `v0.8.1+`.
- **Status:** `open`
- **Tier:** `v0.8.1+`

### `bip38-spec-section-12-ec-multiplied-erratum` — SPEC §12 incorrectly claimed EC-multiplied was refused

- **Surfaced:** v0.7.1 Phase 3 (audit cycle).
- **Where:** `design/SPEC_convert_v0_6.md` §12.
- **What:** The v0.7.0 SPEC §12 stated the `bip38` crate's `Decrypt` impl rejected EC-multiplied codes. Empirical testing in Phase 3 disconfirmed: all 4 EC-multiplied spec vectors decrypt correctly. SPEC §12 corrected in this cycle (commit pinned in matrix). Filed for cross-referencing the erratum source: the v0.7 Phase 1 security review report at `design/agent-reports/v0_7-phase-1-bip38-security-review.md` likely contains the source claim — re-read on next sec-review touch.
- **Why deferred:** documentation-only; closed in this cycle. Filed for audit history continuity.
- **Status:** `resolved 2c59b27`
- **Tier:** `v0.7.1`

### `bip85-dice-application` — BIP-85 `89101'` dice rolls (split product of `bip85-rsa-rsa-gpg-dice-applications`)

- **Surfaced:** v0.8 Phase 6 SPIKE split decision.
- **Where:** `crates/mnemonic-toolkit/src/bip85.rs::format_dice_rolls` + `crates/mnemonic-toolkit/src/cmd/derive_child.rs` dispatch.
- **What:** BIP-85 §"DICE" deterministic dice rolls via SHAKE256 BIP85-DRNG + rejection sampling. Spec at BIP-85 v1.3.0 §"DICE".
- **Status:** `resolved 1dde4dc` (v0.8 Phase 7).
- **Resolution:** v0.8 Phase 7 — `--application dice` + new `--dice-sides <N>` flag. Spec reference vector pinned (`m/83696968'/89101'/6'/10'/0'` → `1,0,0,2,0,1,5,5,2,4`). New `sha3 = "0.10"` direct dep.
- **Tier:** `v0.8`

### `bip85-rsa-rsa-gpg-applications` — BIP-85 RSA + RSA-GPG (split product, deferred)

- **Surfaced:** v0.8 Phase 6 SPIKE split decision (`design/agent-reports/v0_8-phase-6-rsa-crate-security-review.md`).
- **Where:** `crates/mnemonic-toolkit/src/bip85.rs` (would need new app dispatchers); `crates/mnemonic-toolkit/src/cmd/derive_child.rs` (would lift `rsa` / `rsa-gpg` from out-of-scope refusal).
- **What:** BIP-85 application codes `828365'` (RSA) + `67797633'` (RSA-GPG) generate RSA keys deterministically from BIP-85 entropy. Implementation requires the `rsa` crate.
- **Why deferred:** v0.8 Phase 6 SPIKE returned DEFER verdict. RUSTSEC-2023-0071 (Marvin attack: timing sidechannel against PKCS#1 v1.5 decryption) is **unpatched** as of 2026-05-07 (`patched = []`). `rsa` crate is in extended pre-release (`v0.10.0-rc.18`). Adding it as direct dep would import an open advisory into mnemonic-toolkit's `cargo audit` output. BIP-85 RSA / RSA-GPG demand signal is absent.
- **Reopen criteria:** rsa crate publishes patched stable release (`patched = ["X.Y.Z"]` in advisory) OR a user requests BIP-85 RSA / RSA-GPG with a stated downstream use case.
- **Status:** `open`
- **Tier:** `v0.9 / pending-rsa-crate-stability`

### `18-remaining-bip39-trezor-corpus-vectors` — pin remaining 18 of 24 Trezor english corpus cells

- **Surfaced:** v0.7.1 Phase 1.B (BIP test vector audit cycle).
- **Where:** `crates/mnemonic-toolkit/tests/cli_convert_bip39_vectors.rs`.
- **What:** v0.7.1 Phase 1.B pinned 6 of 24 BIP-39 §"Test Vectors" English Trezor corpus cells via hand-rolled tests; the remaining 18 stayed MISSING per the v0.7.1 audit matrix. v0.8 lifts to a parametric loop over the full corpus.
- **Status:** `resolved 85694b2` (v0.8 Phase 8).
- **Resolution:** v0.8 Phase 8 — refactored `cli_convert_bip39_vectors.rs` to a single `bip39_trezor_english_corpus_full` test that loops over all 24 english entries via vendored `tests/bip39_trezor_vectors.json` (Trezor `python-mnemonic` SHA `b57a5ad77a981e743f4167ab2f7927a55c1e82a8`). Audit-matrix coverage 6/24 → 24/24 ✓.
- **Tier:** `v0.7.1-carry`

### `bip38-spec-vector-3-null-byte-passphrase` — V3 Unicode passphrase contains U+0000; not representable via argv

- **Surfaced:** v0.7.1 Phase 3.A (BIP test vector audit cycle).
- **Where:** `crates/mnemonic-toolkit/tests/cli_convert_bip38.rs::{encrypt,decrypt}_..._spec_vector3_unicode_nfc_passphrase` (`#[ignore]`'d); `crates/mnemonic-toolkit/src/cmd/convert.rs` passphrase input plumbing.
- **What:** BIP-38 §"Test vectors" vector 3 specifies a passphrase of 5 codepoints (U+03D2 + U+0301 + U+0000 + U+10400 + U+1F4A9). The U+0000 NULL byte cannot be passed via argv (POSIX `execve` truncates at NULL); the existing `--passphrase=-` stdin path also fails because `read_stdin_to_string` calls `.trim()`. To exercise this vector end-to-end the toolkit needs a NULL-safe input channel — e.g. `--passphrase-bytes-hex <hex>` accepting the raw byte sequence, or a stdin path that reads bytes verbatim (no trim, no UTF-8 reinterpretation). The `bip38` crate itself NFC-normalizes whatever string slice it receives; the gap is purely at the toolkit's input plumbing.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — new `--passphrase-stdin` flag with line-ending-only trim (preserves leading/trailing spaces + internal NULL). Both V3 ignored tests unignored and now active. Phase 1 review I1 added a separate `read_stdin_passphrase` helper distinct from `read_stdin_to_string` to prevent the trim issue.
- **Tier:** `v0.8`

### `electrum-seed-version-spike-pending` — Phase 4 step 0 interactive spike

- **Surfaced:** v0.8.1 Phase 4 (`design/agent-reports/v0_8-phase-4-electrum-seed-version-spike.md`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs:33` — `ELECTRUM_SEED_VERSION_PIN = 17`.
- **What:** SPEC v0.8 §9 + IMPL_PLAN Phase 4 step 0 mandate an interactive spike against current Electrum (>= 4.5.x) to lock `ELECTRUM_SEED_VERSION_PIN` to a verified-cleanly-imports value.
- **Status:** `resolved` (2026-05-12 spike against Electrum 4.5.5).
- **Resolution:** Spike executed against Electrum 4.5.5 in `/tmp/electrum-spike-venv/`. Empirical result: a toolkit-emitted wallet file with `seed_version: 17` loads cleanly via `electrum --offline -w <file> listaddresses` (returns the expected BIP-84 receive set; Electrum migrates the in-memory state to FINAL_SEED_VERSION=59 on save). Source-code cross-check at `wallet_db.py:1195-1211` confirms `seed_version >= 12 → return seed_version` with no rejection at 17. Pin retained at 17 (the SPEC's "minimum cleanly-imports" specification matches 17; 59 is what Electrum WRITES, not the minimum it ACCEPTS). Full report: `design/agent-reports/v0_8-phase-4-electrum-seed-version-spike.md`.
- **Tier:** `v0.8.2`

### `electrum-tr-multi-a-pending-libsecp-taproot` — `--template tr-multi-a` refuses under `--format electrum`

- **Surfaced:** v0.8.1 Phase 4.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` `emit()` guard; refusal fixture `tests/export_wallet/electrum_tr_multi_a_refusal.stderr`.
- **What:** Electrum's `wallet_db.py` does not currently ingest taproot multisig wallet shapes (pending libsecp-taproot integration in Electrum's signer surface). `--format electrum --template tr-multi-a` (or `tr-sortedmulti-a`) emits a byte-exact refusal with pointer to `--format bitcoin-core` (descriptor) or `--format sparrow` (which supports taproot multisig via descriptor-passthrough).
- **Status:** `open` (last upstream-checked 2026-05-12 against Electrum 4.5.5 source; `grep -E "'p2tr'|p2tr" electrum/transaction.py` returns no matches in the script-type enum, confirming taproot script type not yet wired).
- **Tier:** `v1+ / pending-electrum-firmware`

### `electrum-final-seed-version-drift` — track Electrum FINAL_SEED_VERSION upstream

- **Surfaced:** v0.8.1 Phase 4.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` — `ELECTRUM_SEED_VERSION_PIN` doc-comment.
- **What:** Electrum's `wallet_db.py` `FINAL_SEED_VERSION` drifts upward over releases (4.5.5 = 59; the v0.8.1 SPEC §9 cited 71 from master at SPEC-write time). Toolkit pins to 17 (minimum cleanly-imports) and relies on Electrum's migration loader to walk forward. Track in case the loader ever drops support for old migration paths.
- **Status:** `open` (no fix scheduled; tracking only).
- **Tier:** `v1+ / informational`

### `electrum-root-fingerprint-roundtrip-quirk` — Electrum nulls `root_fingerprint` on load

- **Surfaced:** v0.8.1 Phase 4 step 0 spike (2026-05-12, Electrum 4.5.5).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` `emit_electrum_standard_json` + `electrum/keystore.py` `BIP32_KeyStore`.
- **What:** The toolkit emits `keystore.root_fingerprint` per SPEC §9.1 (e.g., `"5436d724"`). Electrum 4.5.5's loader successfully imports the wallet, derives the correct BIP-84 addresses, but its re-serialized form has `"root_fingerprint": null` — the `_root_fingerprint` private attribute on the in-memory `BIP32_KeyStore` is not populated from the on-disk JSON field. Functionally inert for watch-only address derivation; required only for PSBT-with-origin flows. Likely an Electrum-side bug or intentional drop; cross-check against current master may surface a fix.
- **Status:** `open` (informational).
- **Tier:** `v1+ / informational`

### `green-native-multisig-pending-server-support` — `--format green` refuses multisig

- **Surfaced:** v0.8.1 Phase 5.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/green.rs` `emit()` guard; refusal fixture `tests/export_wallet/green_multisig_refusal.stderr`.
- **What:** Blockstream Green's multisig surface is server-mediated (Green Multisig Shield + Liquid), not a direct file-import shape. `--format green` is therefore singlesig-only; multisig templates return a byte-exact refusal with pointer to `--format bitcoin-core` (descriptor) or `--format sparrow`. Resolves once Green publishes a self-custody multisig file-import format.
- **Status:** `open`. Last upstream-checked **2026-05-12**: Green Help Center article `19340800530713-Set-up-watch-only-wallet` returns HTTP 403 to programmatic fetchers (Zendesk-hosted, browser-only). Status cannot be verified autonomously; entry remains open pending manual browser check.
- **Tier:** `v1+ / pending-green-server-support`

### `mnemonic-gui-schema-mirror` — companion to `bg002h/mnemonic-gui` schema gate

- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`; CI gate at `.github/workflows/schema-mirror.yml`.
- **Where:** This CLI's clap-derive `Args` blocks (currently `cmd/{bundle,verify_bundle,convert,export_wallet,derive_child}.rs`); the introspection subcommand at `cmd/gui_schema.rs`.
- **What:** The `mnemonic-gui` GUI mirrors this CLI's clap-derive flag surface at pinned tag `mnemonic-toolkit-v0.9.0` (was `v0.8.1` pre-v0.2). Any flag add / remove / rename / `conflicts_with` / `required_unless_present_any` change in this repo's CLI surface must land in lockstep with a companion `mnemonic-gui` PR that bumps the schema + the `pinned-upstream.toml` tag for this CLI. The `mnemonic-gui` CI gate runs `cargo install --locked --git <this-repo> --tag <pin>` + `cargo test --test schema_mirror`, so drift surfaces as a CI failure. Additionally, the GUI's `build.rs` codegen reads `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType::is_secret_bearing()` and `crates/mnemonic-toolkit/src/slot_input.rs::SlotSubkey::is_secret_bearing()` via `syn::parse_file` to populate its `SECRET_*` constants — drift in those impls is also caught by a runtime source-audit test in the GUI repo.
- **v0.2 update (2026-05-12, mnemonic-toolkit v0.9.0):** `mnemonic gui-schema` introspection subcommand shipped (SPEC §7 contract). The GUI consumes its JSON output instead of (or alongside) the `syn` codegen path. `cli_gui_schema.rs` (16 tests) pins the SPEC §7 contract on this side. The companion `mnemonic-gui` v0.2 Phase C.2 PR consumes the schema via `cargo run -p mnemonic-toolkit -- gui-schema` at build time.
- **Status:** `open` (mirror-invariant; tracking only — every flag-surface PR carries this lockstep work).
- **Tier:** `v1 / mirror-invariant`

### `mk-vectors-pretty-out-help-mismatch` — `mk vectors --pretty` help-text vs source behavior drift

- **Surfaced:** manual-gui v1.0 cycle batch 8 R0 review (2026-05-15), filed at toolkit `63397ef`+. Cited in `docs/manual-gui/src/70-mk/76-vectors.md` and `docs/manual-gui/src/90-appendices/94-release-history.md`.
- **Where:** `mk-cli` source at `crates/mk-cli/src/cmd/vectors.rs:23` (help-text doc-comment) and the mirror at `mnemonic-gui/src/schema/mk.rs:208` (schema help-text).
- **What:** `mk vectors --help` advertises `--pretty: Ignored when --out is supplied`. Source (vectors.rs:70-74 in the `write_per_fixture_files` arm) actually honors `--pretty` — each per-fixture .json file is written via `serde_json::to_string_pretty` when `pretty=true`. The manual-gui v1.0 manual sides with source-truth and notes the help-text drift.
- **Why deferred:** Source-side fix lives in the `bg002h/mnemonic-key` repo (mk-cli `cmd/vectors.rs`); the schema-mirror lives in the `bg002h/mnemonic-gui` repo. Three-cite fix at v1.1 cycle close.
- **Status:** `open`.
- **Tier:** `v1+ / cross-repo`
- **Companion:** intended companion entries in `bg002h/mnemonic-key/design/FOLLOWUPS.md` and `bg002h/mnemonic-gui/FOLLOWUPS.md` at the matching short-id; both currently missing (file with this entry at next cross-repo cycle).

### `secret-taxonomy-public-api-promotion` — promote `SECRET_*` to public toolkit crate API; retire `mnemonic-gui` build.rs source-walker

- **Surfaced:** 2026-05-16, during the `mnemonic-gui` v0.3.3 emergency security fix (closed at sibling `bg002h/mnemonic-gui` commit `6851d1b`). Architect-vetted in this session — see "Suggested fix" below.
- **Where:** New module `crates/mnemonic-toolkit/src/secret_taxonomy.rs` (proposed). Existing private modules at `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType::is_secret_bearing` (line 85) and `crates/mnemonic-toolkit/src/slot_input.rs::SlotSubkey::is_secret_bearing` (line 60). `crates/mnemonic-toolkit/src/lib.rs` (currently exposes `final_word`/`mlock`/`seed_xor`/`slip39`; would add `pub mod secret_taxonomy`).
- **What:** The `mnemonic-gui` repo's `build.rs` parses this toolkit's private `cmd/convert.rs` + `slot_input.rs` via `syn::parse_file` to extract the `is_secret_bearing()` match-arm sets and codegen `SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS` constants. The codegen is the GUI's workaround for the fact that these security-class taxonomies live in *private* toolkit modules (neither is `pub mod` in `main.rs` or re-exported from `lib.rs`), so the GUI has no versioned, addressable contract to depend on. Every fragility of the codegen path (cargo install sandbox having no adjacent toolkit checkout → empty `&[]` stub fallback → silent disable of `persistence::redact_for_persistence` → BIP-39 phrases leaking to `~/.config/mnemonic-gui/state.json` in plaintext; HIGH-severity bug in GUI v0.3.0..v0.3.2) descends from that contract gap. The GUI v0.3.3 tactical patch (committed canonical fallback in `build.rs` + drift gate) pins a second source of truth that must be manually kept in sync; the root issue remains.
- **Why deferred:** GUI v0.3.3 tactical fix is shipped + verified + released. Long-term fix requires a coordinated minor bump on both sides (toolkit `v0.14.0` + GUI `v0.4.0` lockstep). Filed here as the durable architectural cleanup; aim is the v0.4.x mnemonic-gui cycle.
- **Status:** `resolved 1a52612` (mnemonic-toolkit v0.14.0, 2026-05-16). Adds `pub mod secret_taxonomy` exposing `SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS` as `pub const &[&str]`. Per-variant parity tests in `cmd::convert::secret_taxonomy_parity_tests` (NodeType) + `slot_input::tests` (SlotSubkey) use a `declare_*_variants!` macro to make the variant array and the exhaustiveness check share a single source-of-truth list. R1 opus review caught a Critical (closure+driver design was not load-bearing) + 6 Importants; all folded in the same commit before tag. GUI half closed at `bg002h/mnemonic-gui@6fe44b6` (mnemonic-gui v0.4.0).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui/FOLLOWUPS.md` companion entry `secret-taxonomy-public-api-consumption` (resolved at `6fe44b6`).
- **Related memory:** `feedback_build_rs_stub_fallback_security_audit` (the pattern-class lesson) + `feedback_default_cargo_test_runs_sibling_dependent_tests` (same install-path-vs-CI-path divergence trap, opposite direction).

#### Suggested fix (architect-vetted, 2026-05-16)

##### Root structural issue

The GUI treats two safety-critical security predicates as **derivable from upstream source text at the GUI's build time**, with a degradation ladder that ends in a silent fallback. The taxonomy is owned by the toolkit binary's *private* modules — the GUI has no versioned, addressable contract to depend on. Codegen-from-source is the workaround for the missing contract; every fragility descends from that gap. The v0.3.3 patch added a second source of truth (committed `CANONICAL_FALLBACK_*` arrays) but did not fix the contract gap.

##### Recommended option: A — promote `SECRET_*` to public toolkit crate API

Toolkit exports `pub const SECRET_NODE_TYPES: &[&str]` and `pub const SECRET_SLOT_SUBKEYS: &[&str]` (string slices, **not** enum re-exports — avoids leaking enum semver surface) from a new `pub mod secret_taxonomy` re-exported by `lib.rs`. GUI depends on `mnemonic-toolkit` as a regular git dep and `use mnemonic_toolkit::secret_taxonomy::*`. No build.rs codegen.

**Why A wins (vs. the four alternatives evaluated):**
- **Eliminates the empty-array failure mode by construction.** GUI compile fails outright if the toolkit lib does not export the consts; no degradation ladder, no silent fallback.
- **`cargo install --git` works with zero ceremony.** Cargo's resolver pulls the toolkit lib through the normal dependency graph; no env vars, no clone scripts, no source-walker.
- **No load-bearing network call at install time** (unlike Option B: auto-clone by default).
- **Schema-mirror invariant becomes a stronger version-pin gate.** The drift question becomes "GUI's pinned `mnemonic-toolkit` git tag must include the secret-taxonomy module" — easier to enforce and reason about than parse-and-compare against private impl shape.

Alternatives considered + rejected: B (auto-clone always-on; load-bearing network), C (strict-mode build.rs; breaks `cargo install` for everyone), D (keep v0.3.3 tactical patch; preserves the codegen architecture + second source of truth), E (runtime JSON manifest via `mnemonic secret-taxonomy --json`; reintroduces silent-failure on a different axis).

##### Toolkit-side migration (v0.14.0)

1. New file `crates/mnemonic-toolkit/src/secret_taxonomy.rs`:
   ```rust
   //! Public secret-class taxonomy. Source of truth for the
   //! `is_secret_bearing()` predicates on `NodeType` / `SlotSubkey`,
   //! exposed for downstream tools (e.g., mnemonic-gui's persistence
   //! redaction) that cannot import the private enum modules.
   //!
   //! Drift-gated: see unit tests in `cmd/convert.rs` and `slot_input.rs`
   //! that assert `Self::as_str()` of every secret-bearing variant ∈
   //! these slices.

   pub const SECRET_NODE_TYPES: &[&str] = &[
       "phrase", "entropy", "xprv", "wif", "ms1", "bip38", "electrum-phrase",
   ];

   pub const SECRET_SLOT_SUBKEYS: &[&str] = &["phrase", "entropy", "xprv", "wif"];
   ```
2. `crates/mnemonic-toolkit/src/lib.rs`: add `pub mod secret_taxonomy;`.
3. In `cmd/convert.rs::NodeType::is_secret_bearing` and `slot_input.rs::SlotSubkey::is_secret_bearing`, add `#[cfg(test)]` unit tests that walk every variant `V` where `V.is_secret_bearing() == true` and assert `secret_taxonomy::SECRET_*_TYPES.contains(&V.as_str())`. This makes the in-tree predicate and the public constants a single source of truth, enforced at toolkit test time.
4. Update SPEC + manual chapter; minor bump to `v0.14.0` (new pub surface; pre-1.0 0.X-axis bump per repo policy).

##### GUI-side migration (v0.4.0)

1. `Cargo.toml`: add `mnemonic-toolkit = { git = "...", tag = "mnemonic-toolkit-v0.14.0" }` under `[dependencies]`.
2. **Delete** `build.rs` entirely (and the `[build-dependencies]` block).
3. `src/secrets.rs`: replace `include!(concat!(env!("OUT_DIR"), "/secrets_generated.rs"));` with `use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS};` (and `pub use` for crate-internal callers).
4. Delete `tests/secrets_canonical_fallback.rs`. Add a one-test backstop `tests/secret_taxonomy_pin.rs` asserting non-empty + minimum-membership (mirrors the v0.3.3 always-on guard).
5. `pinned-upstream.toml`: tag becomes documentary; load-bearing version pin lives in `Cargo.toml`.
6. `.github/workflows/schema-mirror.yml`: remove the `cargo-test-secrets-canonical-fallback` step; keep the flag-name parity job.
7. `install.sh`: no changes required.
8. GUI minor bump to `v0.4.0`.

##### One-cycle overlap (recommended)

In GUI `v0.4.0`, retain the v0.3.3 `CANONICAL_FALLBACK_*` constants AND add a compile-time assertion that they equal `mnemonic_toolkit::secret_taxonomy::SECRET_*`. Drop the fallback in `v0.5.0`. This catches a malformed-upstream-tag class of regression during the cycle where contributors still expect the old contract.

##### Non-obvious risks

1. **Toolkit dep tree bloats GUI compile** (bitcoin, miniscript, bip39, clap, etc. — none called from `secret_taxonomy`, all linked into the GUI's cargo graph; ~30-60s cold compile cost). Mitigation: feature-gate heavy modules under a `cli` default-on feature; GUI depends with `default-features = false, features = ["secret-taxonomy"]`. Optional; can defer if compile cost is acceptable.
2. **Toolkit becomes a load-bearing library API surface.** Renaming or relocating `secret_taxonomy` is now a semver event. Document in `lib.rs` and FOLLOWUPS.
3. **GUI git-dep pin must stay current** with toolkit's taxonomy releases. If `mnemonic-toolkit-v0.15` adds `ElectrumEntropy` as secret-bearing but GUI still pins `v0.14`, GUI silently lacks the new class. Mitigation: the CI flag-name schema-mirror gate (already running against the live `mnemonic` binary) catches the toolkit-side widening; add a parallel gate that asserts the GUI's pinned-toolkit `SECRET_*` equals the locally-installed `mnemonic`'s reported taxonomy (a future `mnemonic gui-schema` extension can emit it).
4. **`pub const &[&str]` vs `pub use SECRET_NODE_TYPES`.** Resist re-exporting `NodeType` / `SlotSubkey` enums — string slices are a far smaller semver surface and decouple GUI compile from internal enum shape (which evolves with every new node type).
5. **`mnemonic-toolkit` lib platform-compat audit.** Verify the lib builds cleanly on GUI's full platform matrix (macOS, Windows, Linux × x86_64 + aarch64) before the v0.14.0 release; `mlock.rs` uses `libc` and may need cfg-gating audit (already in place but should be revisited).
6. **One-shot lockstep required.** The toolkit v0.14.0 tag and the GUI v0.4.0 PR must be planned together (mirroring the manual-gui v1.0 lockstep pattern). Surface in both repos' FOLLOWUPS with `Companion:` lines per repo convention in `CLAUDE.md`.

### `secret-taxonomy-argv-superset-promotion` — promote `is_argv_secret_bearing()` wider set to a parallel public taxonomy

- **Surfaced:** 2026-05-16, toolkit v0.14.0 reviewer-loop (R1). Sibling to `secret-taxonomy-public-api-promotion` which promoted the narrower persistence-class set; this entry covers the wider argv-leakage set that includes MiniKey.
- **Where:** `crates/mnemonic-toolkit/src/secret_taxonomy.rs` (add `SECRET_NODE_TYPES_ARGV: &[&str]`); `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType::is_argv_secret_bearing` (line 107; add parity test).
- **What:** `NodeType::is_argv_secret_bearing()` returns the **wider** set: `is_secret_bearing()` plus `MiniKey`. The v0.14.0 release promoted only the narrower `SECRET_NODE_TYPES` for persistence-redaction use. The wider set has no public mirror yet, so any future downstream consumer that needs argv-leakage protection (e.g., a GUI run-confirm modal that redacts the argv preview, per the `gui-run-confirm-modal-secret-redaction` GUI-side FOLLOWUP) will face the same private-symbol-scraping pressure that motivated v0.14.0's narrower promotion. Add `pub const SECRET_NODE_TYPES_ARGV: &[&str]` to `secret_taxonomy` as an additive minor surface (compatible with v0.14.0's stability contract). Add a parity test against `is_argv_secret_bearing` in the same shape as the existing narrower-set parity test.
- **Why deferred:** Reviewer flagged this as out-of-scope for v0.14.0 (which was explicitly scoped to closing the GUI v0.3.0..v0.3.2 persistence-leak bug). Filed here so the wider-set promotion doesn't silently fall off the radar.
- **Status:** `open`
- **Tier:** `cross-repo`
- **Companion:** `convert-minikey-stdout-redaction` (toolkit FOLLOWUP that tracks the narrow/wide asymmetry inside the toolkit); `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-run-confirm-modal-secret-redaction` (the GUI-side use case that would consume the wider set).

### `secret-taxonomy-feature-gate-toolkit-lib` — feature-gate the toolkit lib so consumers can pull `secret_taxonomy` without the heavy dep tree

- **Surfaced:** 2026-05-16, post-v0.14.0 architect-audit (Important #4). Surfaced separately from `secret-taxonomy-public-api-promotion` because the architect's original Option A blueprint contemplated it as an optional follow-up risk #1 mitigation, but no concrete FOLLOWUP was filed at v0.14.0 ship time.
- **Where:** `crates/mnemonic-toolkit/Cargo.toml` (`[features]` block, currently absent); `crates/mnemonic-toolkit/src/lib.rs` (`pub mod` declarations); downstream `mnemonic-gui/Cargo.toml` (`[dependencies] mnemonic-toolkit` — would add `default-features = false, features = ["secret-taxonomy"]`).
- **What:** Today `mnemonic-gui` v0.4.0+ depends on this crate as a regular cargo dep for `pub mod secret_taxonomy`. That pulls the toolkit's full dep tree into the GUI's cargo graph: `bitcoin`, `miniscript`, `bip39`, `bip38`, `clap`, `secp256k1`, `bech32`, `hmac`, `sha2`, `pbkdf2`, plus the new git-deps on `ms-codec` / `mk-codec` / `md-codec`. None of these are referenced from `secret_taxonomy`, but the cargo dep graph links them in. Cold-build cost for the GUI grew by ~30-60s on first-time builds (the architect's risk #1). Feature-gate the heavy modules behind a default-on `cli` feature; expose `secret-taxonomy` as a default-off small-surface feature. GUI depends with `default-features = false, features = ["secret-taxonomy"]`; toolkit's own bin builds with the default features (no change). Lib API for `secret_taxonomy` is preserved as-is.
- **Why deferred:** Architect risk #1 was flagged as optional ("Optional; can defer if compile cost is acceptable"). The v0.14.1 cfg-gate of `mlock` already unblocked the Windows compile failure that motivated this audit; the dep-tree cost is purely a quality-of-life issue for GUI cold builds, not a correctness gap.
- **Status:** `open`
- **Tier:** `v0.15` / `nice-to-have`
- **Companion:** Architect's "non-obvious risks" #1 from the resolved `secret-taxonomy-public-api-promotion` entry.

### `mnemonic-toolkit-cratesio-publish` — swap codec git deps for crates.io versions; publish toolkit to crates.io

- **Surfaced:** 2026-05-16, post-v0.14.2 crates.io publish audit (the cycle that yanked the vulnerable mnemonic-gui v0.3.0/v0.3.1 + published md-codec v0.33.1 + md-cli v0.5.2).
- **Where:** `crates/mnemonic-toolkit/Cargo.toml` (lines 20-22: `ms-codec` / `mk-codec` / `md-codec` are all `{ git = ..., tag = ... }`); the codec crate names already exist on crates.io (`ms-codec` v0.1.3, `mk-codec` v0.3.0, `md-codec` v0.33.1).
- **What:** `mnemonic-toolkit` cannot currently be published to crates.io because its three codec dependencies (`ms-codec`, `mk-codec`, `md-codec`) are git-only deps. crates.io's publish rules require version-or-version+git/path deps; pure-git deps are forbidden in published crates. The codec siblings are already on crates.io at versions that satisfy or supersede the toolkit's git pins:
   - `ms-codec`: toolkit pins git@v0.1.3; crates.io has v0.1.3 (✓ aligned)
   - `mk-codec`: toolkit pins git@v0.2.1; crates.io has v0.3.0 (newer — likely breaking; needs audit)
   - `md-codec`: toolkit pins git@v0.16.1; crates.io has v0.33.1 (significantly newer — multiple breaking minors; needs substantial audit)

   Work: bump toolkit's codec deps from git to crates.io versions in lockstep with whatever code changes those minor-bumps require, then `cargo publish --dry-run` and `cargo publish`. Once toolkit is on crates.io, `mnemonic-gui` can drop its git dep too (see `mnemonic-gui-cratesio-publish` companion).
- **Why deferred:** Significant lift — bumping `md-codec` from v0.16.1 to v0.33.1 is 17 minor versions of API drift to audit. Currently working around via the constellation's `install.sh` script (`cratesio=no` for toolkit). Only matters for users running `cargo install mnemonic-toolkit` directly (without `--git`); install-script users + GUI consumers (who pull via the script-managed git+tag path) are unaffected.
- **Status:** `open`
- **Tier:** `v1+ / nice-to-have`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` companion entry `mnemonic-gui-cratesio-publish` (blocked by this).

### `gui-schema-effect-on-dropdown-options-vocab` — dropdown-option-disable Effect grammar for SPEC §6.6 rows 9/10/11

- **Surfaced:** 2026-05-16, GUI v0.6.0 cycle (`mnemonic-toolkit-v0.17.0` + `mnemonic-gui-v0.6.0`) close. Filed per the §6.10.7 closing list — unblocked by the v3 SlotCount* predicate-machinery (now expressible) but the *effect* side requires a new Effect grammar.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.3 (Effect vocabulary extension); `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::VisibilityProjection` (toolkit emitter); `mnemonic-gui/src/schema_check.rs::VisibilityProjection` (GUI consumer); `mnemonic-gui/src/form/widget.rs` (Dropdown widget — per-option-disable rendering).
- **What:** SPEC §6.6 rows 9/10/11 need a per-option Effect — e.g., row 9 disables `--threshold` values > N when slot-count is N; row 10 disables single-sig templates when N > 1; row 11 disables multisig templates when N == 1. v3 grammar offers `hidden` / `disabled` / `required` / `pin_value` — all acting on the whole flag. New variant candidate: `disable_options: { values: [...] }` for Dropdown FlagKind, paired with the `slot_count_*` Predicates already in v3.
- **Why deferred:** Predicate-machinery shipped in v3 unblocks the predicate side; Effect grammar extension is the next half. Out of v0.6.0 cycle scope; would need SPEC §6.10.3 extension + Dropdown widget rendering refactor.
- **Status:** `resolved c7ac604` — Batch B-1 cycle (`mnemonic-toolkit-v0.18.0` + `mnemonic-gui-v0.7.0`, 2026-05-16) shipped the disable_options Effect grammar (rows 10/11) + GUI-internal NumberMax::FromSlotCount FlagKind extension (row 9). Schema bumps `v3 → v4`. Row 9 closes GUI-side without a toolkit wire-format change (Option A per the v0.7.0 design doc — single-consumer pragma; promotable if a second `gui-schema` consumer ever appears). Toolkit emitter: VisibilityProjection::DisableOptions variant + 2 new bundle_conditional_rules entries (count 11 → 13). GUI consumer at `mnemonic-gui-v0.7.0` (`f86a696`) ships VisibilityProjection deserialize arm + Visibility::DisableOptions + NumberMax enum + render-time orthogonal composition.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-effect-on-dropdown-options-vocab` (resolved at `f86a696`).

### `gui-schema-cross-slot-predicate-projection` — cross-slot relational predicate types for SPEC §6.6 rows 8/13/14

- **Surfaced:** 2026-05-16, GUI v0.6.0 cycle close. Filed per the §6.10.7 closing list — these rows need predicate types beyond the v3 `slot_count_*` extensions.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.2 (Predicate AST extension); `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::Predicate` (toolkit emitter); `mnemonic-gui/src/schema_check.rs::Predicate` (GUI consumer); `mnemonic-gui/tests/gui_schema_conditional_drift.rs::synthesize_satisfying` (drift gate extension).
- **What:** §6.6 row 8 (cross-slot equality, e.g., "two slots must NOT share an xpub"), row 13 (BIP-388 distinct-key — pairwise distinctness across `@i` slots), row 14 (per-`@N` annotation consistency — e.g., if `@1.xpub` is `external`, all `@1.*` annotations must agree). Candidate Predicate variants: `slot_subkey_distinct: { subkey: "xpub" }`, `slot_annotation_consistent: { annotation: "external" }`.
- **Why deferred:** Predicate-machinery for relational predicates is missing in v3; full design requires SPEC §6.10.2 grammar extension. Out of v0.6.0 cycle scope.
- **Status:** `resolved 38ad066` — Batch B-2 close 2026-05-16. **Row 8 resolved Option A** (`mnemonic-gui-v0.7.1` `38ad066`): GUI-internal `slot_editor.rs::detect_slot_index_gaps` helper + inline warning banner. Pure GUI-side pre-check; no toolkit wire-format change (mirrors the v0.7.0 row-9 NumberMax::FromSlotCount pattern). 9 new test cells in `tests/slot_editor_contiguity.rs`. SPEC §6.10.7 row 8 flipped to `ENCODED v3 (GUI-internal)`. **Rows 13/14 wontfix** with rationale: row 13 (BIP-388 distinct-key) requires xpub derivation that the GUI can't replicate for phrase-bearing slots (toolkit-binding-logic duplication is high-cost low-value); row 14 (per-`@N` annotation consistency) requires descriptor-string parsing + cross-slot annotation cross-reference (similarly high-cost low-value). Both surface authoritatively at CLI run-time per §6.6 rows 13/14 stderr; GUI pre-check adds marginal UX over the existing CLI rejection. SPEC §6.10.7 "Runtime-deferred rules" section updated to enumerate row 13/14 under a new "CLI-rejection-sufficient (wontfix)" partition. All v0.6.0-cycle-close FOLLOWUPs now closed (Batch A v0.6.1 + Batch B-1 v0.7.0 + Batch B-2 v0.7.1).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-cross-slot-predicate-projection` (resolved at `38ad066`).

### `gui-schema-derive-child-meta-template-groups-spurious` — toolkit emits `meta.template_groups` on a subcommand with no `--template` flag

- **Surfaced:** 2026-05-16, GUI v0.6.0 cycle-close opus reviewer audit. Important finding (confidence 95): `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:244-259` `build_subcommand_meta` matches `name == "derive-child"` and emits a `template_groups` block, but `crates/mnemonic-toolkit/src/cmd/derive_child.rs` has ZERO `--template` references (grep-confirmed). SPEC §6.10.8 also lists derive-child as a template-consumer in error; toolkit test `derive_child_emits_meta_template_groups` enshrines the wrong invariant. Recurring `[feedback-r0-must-read-source-off-by-n]` failure mode.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:244-259` (the spurious match arm); `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.8 (matching mis-claim); `crates/mnemonic-toolkit/tests/cli_gui_schema_v3_extensions.rs` (`derive_child_emits_meta_template_groups`).
- **What:** Either (a) remove `derive-child` from the `build_subcommand_meta` match arm + delete the matching test cell + correct SPEC §6.10.8 prose, or (b) consciously document why derive-child gets the meta block despite having no `--template` widget. (a) is the source-faithful fix.
- **Why deferred:** Cosmetic — was scheduled for the next toolkit cycle, but folded faster as the v0.17.1 patch (the original "Folding into the next toolkit cycle" deferral rationale was outweighed by the cycle's small scope + lockstep-with-GUI-v0.6.1 patch cadence).
- **Status:** `resolved 7ed3784` — shipped at `mnemonic-toolkit-v0.17.1` (2026-05-16). Took option (a): P0 (`598b4ba`) removed `derive-child` from `build_subcommand_meta` match arm at `gui_schema.rs:248`; deleted `derive_child_emits_meta_template_groups` test cell + added replacement negative-cell `derive_child_omits_meta_template_groups` as regression guard; corrected SPEC §6.10.8 paragraph 2 prose + added parenthetical noting the v0.17.1 correction. TDD discipline: negative cell ran RED against unmodified source (panic showed the spurious `multisig: [...], single_sig: [...]` block), GREEN after the match-arm fix. 30 test binaries pass; 8 cells in `cli_gui_schema_v3_extensions` (was 8 before; one cell replaced). Companion GUI v0.6.1 picks up the cleaner JSON shape via the v0.17.1 pin bump.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-derive-child-meta-template-groups-spurious`.

### `gui-schema-classify-descriptor-subcommand` — diagnostic subcommand for the GUI canonicity-classifier drift gate

- **Surfaced:** 2026-05-16, v0.19.0 cycle Phase 7 end-of-cycle opus review I1.
- **Where:** new flag on `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::run` (`--classify-descriptor <STR>`); manual chapter row in `docs/manual/src/40-cli-reference/41-mnemonic.md` under `mnemonic gui-schema`; mnemonic-gui's `tests/canonicity_drift.rs` (drift-gate kittest that shells out to the new flag).
- **What:** The GUI's `classify_descriptor_canonicity` (5-regex wrapper-form matcher at `mnemonic-gui-v0.8.0` `src/form/conditional.rs:99-126`) mirrors md-codec's `canonical_origin` table verbatim. To keep the GUI and toolkit views drift-free as md-codec evolves (e.g., adds a new canonical shape), the toolkit should expose a `mnemonic gui-schema --classify-descriptor <STR>` flag that prints `canonical` or `non-canonical` (exit 0 on success, exit 2 on parse failure). A new GUI kittest cell shells out to this flag for each fixture in the canonicity-corpus and asserts the verdict matches the GUI's classifier.
- **Why deferred:** Out of v0.19.0 scope per Phase 7 escalation gate. The GUI classifier is small (~50 LOC) and mirrors md-codec's 5 shapes verbatim today — drift risk is real but bounded; the cycle ships v0.19.0 + v0.8.0 without the drift gate. Closing this FOLLOWUP adds ~20 LOC toolkit + ~80 LOC GUI test + 1 manual chapter row.
- **Status:** `resolved 14c8119` — shipped at `mnemonic-toolkit-v0.20.0` (2026-05-17) as F2 of the three-FOLLOWUP fold cycle. New `GuiSchemaArgs::classify_descriptor: Option<String>` field + run() short-circuit branch at `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:52-61,1076-1085`. 8 toolkit test cells at `tests/cli_gui_schema_classify_descriptor.rs` (5 canonical shapes + 2 non-canonical + 1 parse-failure exit 2). Manual chapter row + cspell dict update. Lockstep with `mnemonic-gui-v0.8.1` drift gate at `tests/canonicity_drift.rs` (18-fixture corpus + floor assertion).
- **Tier:** `v0.20-feature` (small scope; can ride along with any v0.20+ patch).
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `canonicity-drift-gate-via-toolkit-classify-subcommand` (to file in lockstep).

### `gui-non-canonical-descriptor-banner-and-placeholder` — info banner + slot_editor path-placeholder for default-inferred non-canonical descriptors

- **Surfaced:** 2026-05-16, v0.19.0 cycle Phase 7 end-of-cycle opus review I2; predecessor lesson at `[[project-v0-18-1-v0-7-2-b1-bugfix-closed]]` (UX flaw missed by reviewer-loop + only caught by user-running-the-feature).
- **Where:** `mnemonic-gui-v0.8.0` `src/form/conditional.rs` (new `descriptor_non_canonical_default_path_notice` helper); `mnemonic-gui-v0.8.0` `src/form/slot_editor.rs:191` (path-field placeholder via egui `hint_text`); `mnemonic-gui-v0.8.0` `src/main.rs` (banner-render site adjacent to slot grid).
- **What:** v0.19.0 ships the canonicity-aware `--account` pin lift in `mnemonic-gui-v0.8.0` `src/form/conditional.rs::bundle()` (so the user's typed `--account N` flows through to the toolkit's default-path inference). What did NOT ship: (a) an inline info banner stating "non-canonical descriptor; @{N} will use default path m/48'/<coin>'/<account>'/2'"; (b) a `hint_text` placeholder in the slot_editor's path field showing the computed default. CLI users see the stderr info notice; GUI users get the correct behavior but NO visual cue that a default was assumed. Per the predecessor cycle's user-running-the-feature reviewer-dimension lesson, this surface is load-bearing for UX-perceptibility.
- **Why deferred:** Out of v0.19.0 scope per Phase 7 escalation. The canonicity-aware override behaves correctly; the perceptibility surface is a follow-on cosmetic. Targets a v0.8.1 GUI patch.
- **Status:** `resolved 14c8119 + mnemonic-gui-v0.8.1` — shipped at `mnemonic-toolkit-v0.20.0` (lockstep) as F3 of the three-FOLLOWUP fold cycle. New helpers in `mnemonic-gui/src/form/conditional.rs`: `coin_type_for_network` (inline mirror of `network::CliNetwork::coin_type`) + `descriptor_non_canonical_default_path_notice` (banner-text producer with empty-descriptor guard); new `FormState::number_value` accessor at `src/schema/mod.rs`; new `path_hint: Option<&str>` parameter on `src/form/slot_editor.rs::render` with `match (row.subkey, path_hint)` body for hint_text on Path subkey + empty value; `src/main.rs` orchestration with snapshot-then-render pattern + suppress-when-account-out-of-u32-range semantic (Phase 3 R0 I2 fold). 4+3 GUI test cells across `tests/descriptor_non_canonical_default_path_notice.rs` and `tests/slot_editor_path_hint_text.rs`. Banner text restored `--slot @N.path=m/...` literal flag-override syntax (Phase 3 R0 I1 fold). Companion drift gate at `mnemonic-gui/tests/canonicity_drift.rs` shells out to the `--classify-descriptor` flag from F2.
- **Tier:** `v0.8.1-gui-patch`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-non-canonical-descriptor-banner-and-placeholder` (to file in lockstep).

### `verify-bundle-multi-cosigner-mk1-chunk-assembly` — verify-bundle's --mk1 / --bundle-json intake fails to reassemble multi-chunk cosigner mk1 cards

- **Surfaced:** 2026-05-16, v0.19.0 cycle Phase 7 end-of-cycle opus review C1-followup investigation. Pre-existing (not introduced by v0.19.0); confirmed by reproducing the same failure shape against a CANONICAL `wsh(sortedmulti(2,@0,@1))` descriptor (whose verify-bundle round-trip should have worked since v0.4.x).
- **Where:** root cause was at the synthesize-side `derive_mk1_chunk_set_id` callers (NOT the intake side as originally hypothesized). Four hazard sites at `crates/mnemonic-toolkit/src/synthesize.rs:246/391/561/754` derived csi from a shared `policy_id` stub for n>1 cosigners; downstream `emit_multisig_checks` BTreeMap-by-csi grouping collapsed all chunks into one bucket; mk-codec decode then errored with `ChunkedHeaderMalformed`.
- **What:** When a multi-cosigner descriptor (n ≥ 2) emits a bundle, each cosigner's mk1 string is chunked across multiple bech32-family strings (the canonical chunking for long encoded payloads). The bundle JSON envelope shape is `mk1: Vec<Vec<String>>` (outer per cosigner, inner per chunk). Two intake failure shapes:
  - **Flat `--mk1` repetition** (passing each chunk as a separate `--mk1` occurrence flat across all cosigners): md-codec's chunked reader sees `total_chunks = 2` from the header of the FIRST cosigner's chunk, then errors with `ChunkedHeaderMalformed("received 4 chunks, header declares total_chunks = 2")` when the second cosigner's first chunk arrives.
  - **`--bundle-json <path>`** (v0.4.3+ envelope intake): cosigner[1]'s mk1 is reported "not supplied" while cosigner[0]'s mk1 fails decode — the envelope reader's per-cosigner unpacking does not correctly hand each cosigner's chunk-vec to the per-cosigner decoder.
- **Why deferred:** Out of scope for v0.19.0 cycle (Phase 7 user-run-the-feature smoke confirmed `bundle --self-check` round-trip succeeds for the non-canonical default-inferred bundle, which is the load-bearing C1 verification). The mk1-chunk-assembly bug class predates v0.19.0 and affects canonical multi-cosigner bundles equally; v0.19.0's verify-bundle parity work is correctly anchored at the synthesize side. Fixing the intake side is a separate cycle.
- **Status:** `resolved 14c8119` — shipped at `mnemonic-toolkit-v0.20.0` (2026-05-17) as F1 of the three-FOLLOWUP fold cycle. Phase 0 reconnaissance confirmed C1 hypothesis (all 4 mk1 chunks shared `csi=907005`; xpub fingerprints distinct between cosigners). Phase 1 R0 surfaced 1 Important (4th hazard site at `:391` in `synthesize_multisig_full` was originally missed; plan-doc conflated this function with `synthesize_multisig_watch_only` at `:561`). All 4 sites now derive csi from `<loop_var>.xpub.fingerprint().to_bytes()`. Single-sig n=1 paths at `:228` + `:737` use `&stub` (bare) and are byte-identity-pinned via `cli_verify_bundle_multi_cosigner_mk1.rs` cell 5 + `tests/fixtures/v0_20_0_single_sig_bip84_bundle.json`. 6 new test cells exercise `bundle | verify-bundle --bundle-json` round-trip for canonical wsh-sortedmulti (template mode), non-canonical wsh(andor(...)) (descriptor mode), 3-cosigner non-canonical, flat `--mk1` argv repetition, and `--self-check` sanity (gap-B documented at `.v0_20_0-phase0-artifact.md`: self-check decodes per-cosigner chunks separately, bypassing csi-grouping). User's flagship v0.19.0 `wsh(andor(...))` 3-of-3 invocation round-trips end-to-end.
- **Tier:** `v0.20-bugfix` (or sooner if user demand surfaces — the mk1-chunk-assembly intake bug is invisible to users running `bundle --self-check` or `bundle | shasum` workflows; it primarily blocks the verify-bundle three-card forensic check on multi-cosigner bundles)
- **Companion:** none yet.

### `manual-yml-bind-real-mnemonic-bin` — CI manual.yml uses placeholder `MNEMONIC_BIN=true`, weakening flag-coverage gate

- **Surfaced:** 2026-05-17, v0.20.0 cycle Phase 5 end-of-cycle opus review I2.
- **Where:** `.github/workflows/manual.yml:81` invokes `make -C docs/manual lint` with `MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk`. The flag-coverage gate at `docs/manual/tests/lint.sh:84` then runs `true gui-schema --help` (exits 0 with empty stdout); the per-subcommand flag-extraction loop at `:85-87` short-circuits via `warn "no flags parsed... skipping"`.
- **What:** The bidirectional `cli-surface-mirror` invariant relies on the flag-coverage lint asserting that every clap-derive `--flag` is mirrored in the manual chapter. With placeholder binaries, that assertion never runs in CI — drift can ship and the manual lint will pass green. Local pre-commit lint catches the gap because developers run with real binaries on PATH, but the CI gate is a fig leaf.
- **Why deferred:** v0.20.0 cycle F2 inherited the gap; F2's `--classify-descriptor` is correctly documented at `41-mnemonic.md:962-996` and verified locally, but the CI gate didn't enforce it. Closing this FOLLOWUP requires either (a) building the four real binaries in `manual.yml` via cargo-install pinned-tag pre-step (slow but authoritative); or (b) bumping a probe binary that exits 1 on unknown subcommand so the placeholder approach fails-loud instead of silently skipping.
- **Status:** `open`
- **Tier:** `v0.20+-ci-hygiene`
- **Companion:** none.

### `canonicity-drift-gate-floor-too-lenient` — F2 drift gate floor of 50% allows broad regression to pass silently

- **Surfaced:** 2026-05-17, v0.20.0 cycle Phase 5 end-of-cycle opus review I4.
- **Where:** `mnemonic-gui/tests/canonicity_drift.rs:138` floor assertion `classified >= FIXTURES.len() / 2`.
- **What:** The drift gate iterates 18 canonical/non-canonical fixtures; today 15 classify successfully and 3 parse-fail (BIP-388 `@N/**` shorthand). The 50% floor (= 9) allows the toolkit's parser to regress such that only 9 of 18 fixtures classify and the gate still passes. The `feedback-ci-snapshot-test-substring-vacuity` lesson recommends tight floors; current floor has ~3x headroom which is generous.
- **Why deferred:** Bumping the floor to e.g. `FIXTURES.len() - 4` (allowing the 3 known parse-fails plus 1 cushion) is brittle if a future toolkit change makes one of the BIP-388 fixtures start parsing successfully (the floor would suddenly trip on the FIRST parse-fail it sees). Right answer is probably a per-fixture classified expectation table rather than a count-only floor.
- **Status:** `open`
- **Tier:** `v0.20+-test-hygiene`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry to file in lockstep.

### `descriptor-mode-all-or-nothing-slot-set` — should descriptor mode reject mixed phrase/watch-only slot sets?

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 1.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:1077-1237` (per-slot resolver loop). UX question raised by the v0.21.0 SPEC §5.8 per-slot emission rule: when a user supplies `--slot @0.phrase=X --slot @1.xpub=Y --slot @2.phrase=Z`, the toolkit happily emits a "hybrid" bundle with `ms1 = ["pop", "", "pop"]`. Is that ever a meaningful real-world configuration, or should the CLI refuse mixed-mode slot sets?
- **What:** The SPEC §5.8 emission rule defines the wire-format encoding for hybrid bundles, but doesn't take a position on whether the toolkit should EMIT them. A bundle where slots are inconsistently configured (some phrase-bearing, some watch-only) is hard to reason about in a real deployment: inheritance ceremonies typically want either "I have all phrases, just emit everything" OR "I have NONE of the phrases, this is watch-only". Mixed-mode is most plausibly a user error. A future cycle could add a CLI refusal guard (`error: descriptor mode requires either all-phrase or all-watch-only slot bindings; got mixed`) gated behind a `--allow-hybrid` opt-in flag for advanced use cases.
- **Why deferred:** UX question, not a correctness gap. The v0.21.0 cycle's scope was conformance to SPEC §5.8 emission, not slot-binding policy. Filing for the v0.22+-feature tier so a future cycle can revisit with full UX-research context.
- **Status:** `open`
- **Tier:** `v0.22+-feature`
- **Companion:** none.

### `synthesize-descriptor-deduplicate-with-unified` — refactor synthesize_descriptor and synthesize_unified into a shared helper

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 2.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:200-275` (`synthesize_descriptor` body) + `crates/mnemonic-toolkit/src/synthesize.rs:709-774` (`synthesize_unified` body). After v0.21.0's per-slot ms1 emission fix, both functions now iterate `cosigners`/`slots` and emit ms1/mk1 the same way (cf. plan §2.2 mirror-comment at synthesize.rs:255-257 vs synthesize.rs:710-723).
- **What:** The two synthesis paths now share most of their logic (ms1 per-slot emission + mk1 per-cosigner chunking + md1 splitting). A shared helper `fn emit_unified_cards(descriptor, cosigners, privacy_preserving) -> Result<Bundle>` would deduplicate the bodies and reduce the maintenance surface. Pre-v0.21.0, the divergent ms1 emission rule (ms1[0]-only-for-descriptor vs per-slot-for-unified) blocked this refactor; the per-slot fix unlocks it.
- **Why deferred:** Pure refactor; no user-visible behavior change. v0.21.0 scope was the SPEC §5.8 conformance fix + manual regen; deduplication is a separate concern. The two call sites (cmd/bundle.rs:1259 + cmd/verify_bundle.rs:673) currently distinguish descriptor-mode vs template-mode via the calling code path, which a refactor could preserve via a `mode: BundleMode` enum dispatch inside the shared helper.
- **Status:** `open`
- **Tier:** `v0.22+-refactor`
- **Companion:** none.

### `inheritance-example-transcript-coverage` — add `41-inheritance.{cmd,out}` transcript fixture to `docs/manual/tests/transcripts/`

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 3.
- **Where:** `docs/manual/src/40-cli-reference/41-mnemonic.md:209-216` (bundle command) + `:351-357` (verify-bundle command) — the inheritance worked example. Today's `make verify-examples` lint pass at `docs/manual/Makefile` doesn't cover chapter 41's `wsh(andor(...))` recipe.
- **What:** Add a transcript pair `docs/manual/tests/transcripts/41-inheritance.cmd` + `41-inheritance.out` that drives the chapter-41 bundle + verify-bundle commands end-to-end against the installed `mnemonic` binary, diffs the captured stdout against expected, and fails CI if the manual's documented output drifts from the binary's actual output. This would catch a future regression to the SPEC §5.8 per-slot emission (or any other example-block drift) at lint time instead of at user-complaint time.
- **Why deferred:** Phase 4 architect-must-run-prose discipline caught the post-v0.21.0 ms1 strings byte-exact in this cycle; the CI gap is real but the manual content is currently correct. Adding transcript coverage is test-hygiene work that can be done in a dedicated docs cycle.
- **Status:** `open`
- **Tier:** `v0.22+-test-hygiene`
- **Companion:** none.

### `pre-v0_21-bundle-shape-detection` — verify-bundle stderr advisory when supplied bundle matches pre-v0.21.0 broken descriptor-mode shape

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 4.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (case-4 ms1 emission at lines ~1273-1276) + SPEC §5.8 v0.21.0 migration note paragraph at `design/SPEC_mnemonic_toolkit_v0_5.md:153`.
- **What:** When v0.21.0 verify-bundle encounters a bundle where `ms1[0]` is populated but `ms1[1+]` are all `""` AND the supplied slots are all `--slot @i.phrase=` (signaling the user thinks the bundle was full-mode), the failure mode is `ms1_decode[i]: fail cosigner[i] ms1 expected (full-mode bundle) but not supplied` per SPEC §5.7 case 4. The error is cryptic without context. A future enhancement: detect this specific shape (pre-v0.21.0 descriptor-mode bundle with phrases for @1+) and print a stderr advisory `info: bundle ms1 shape matches pre-v0.21.0 descriptor-mode @0-only emission. Regenerate with v0.21.0+ to fix per SPEC §5.8 emission rule.` before the check output.
- **Why deferred:** UX improvement; the SPEC migration note + the GitHub release notes already document the migration. The error message itself is technically accurate per SPEC §5.7 case 4. A stderr advisory would be a nice-to-have for users replaying old bundles but is not load-bearing.
- **Status:** `open`
- **Tier:** `v0.22+-ux`
- **Companion:** none.

### `self-check-ms1-iteration-audit` — should `self_check_bundle` iterate ms1 like verify-bundle now does?

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 5. Companion to v0.20.0 cycle's `feedback-self-check-bypasses-csi-grouping` memo.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::self_check_bundle::MkField::Multi` (at bundle.rs:1478-1504 per [[feedback-self-check-bypasses-csi-grouping]] anchor); ms1 iteration is similarly skipped in self-check.
- **What:** The `--self-check` flag at `mnemonic bundle` only validates per-cosigner mk1 chunks decode; it does NOT iterate ms1 to validate the per-slot emission rule. Post-v0.21.0, this gap is more visible: self-check could regress to the @0-only emission pattern silently. Audit: should `self_check_bundle` add an ms1 iteration that asserts every phrase-bearing slot's ms1 decodes back to the supplied entropy? This would make self-check a regression guard for the SPEC §5.8 emission rule, complementing the verify-bundle --bundle-json round-trip in the new Cell 7 integration test.
- **Why deferred:** Test-coverage gap, not a correctness gap. The v0.21.0 Cell 7 (`descriptor_mode_3_of_3_emits_per_slot_ms1_post_v0_21`) and the SPEC §5.8 amendment together cover the user-facing regression surface; self-check is an internal sanity check. Audit + fix is a separate cycle.
- **Status:** `open`
- **Tier:** `v0.22+-test-coverage`
- **Companion:** none.

### `api-harvest-drift-on-synthesize-descriptor-signature` — technical-manual API table documents the dropped 3rd arg

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 6 + R0 I7 fold.
- **Where:** `docs/technical-manual/transcripts/api-harvest-mnemonic-toolkit.md:261` documents the v0.20.x signature `synthesize_descriptor(descriptor, cosigners, entropy, privacy_preserving)`. Post-v0.21.0 the function is `synthesize_descriptor(descriptor, cosigners, privacy_preserving)` (3-arg).
- **What:** The technical manual's API harvest table is generated from a stale snapshot of the public surface. After v0.21.0 drops `entropy: Option<&[u8]>` from `synthesize_descriptor`, the doc line drifts. Closing this FOLLOWUP requires either (a) regenerating the API harvest at the next technical-manual cycle (which already has its own update cadence per the project's `docs/manual/` vs `docs/technical-manual/` distinction), or (b) adding a CI lint that grep-asserts each documented signature exists verbatim in the live source.
- **Why deferred:** `docs/technical-manual/` is a distinct surface from `docs/manual/` per CLAUDE.md. The v0.21.0 cycle's mirror invariant covers `docs/manual/` only. Technical-manual updates ride a separate cadence and shouldn't block toolkit releases.
- **Status:** `open`
- **Tier:** `v0.22+-doc-hygiene`
- **Companion:** none.

### `verify-bundle-xpub-parent-fingerprint-derivation` — extend xpub-vs-md1 parent_fingerprint check to depth ≥ 2

- **Surfaced:** 2026-05-17, v0.24.0 Tranche A.1 end-of-phase architect review (folded `verify-bundle-watch-only-xpub-path-internal-consistency` as stderr-warning helper, but architect noted the parent_fingerprint check is narrower than plan).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — `emit_watch_only_xpub_path_cross_check` parent_fingerprint branch around `:1996-2029`. v0.24.0 implementation did structural sanity only: depth-0 enforces BIP-32 all-zeros invariant; depth-1 compares against claimed master fingerprint (mk1 origin_fingerprint or md1 TLV fingerprints); depth ≥ 2 was SKIPPED.
- **What:** Original framing said "derive parent xpub from supplied mk1" — this is **structurally impossible**: BIP-32 `derive_pub` is parent→child only; child→parent is the cryptographic one-way step. v0.25.0 corrects the mechanism: derive the parent xpub from the supplied **ms1** (seed) — `ms_codec::decode → entropy → BIP-39 → seed → master xpriv → derive_priv at path[..N-1] → from_priv → fingerprint`. New helper `emit_full_path_parent_fingerprint_check` at `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (after `emit_watch_only_xpub_path_cross_check`). Wired into `run_full`, `run_watch_only`, and `run_multisig`. Reuses passphrase / language / network from `VerifyBundleArgs` (no NOTICE-and-skip fallback). For the watch-only depth ≥ 2 case (no ms1 supplied for that cosigner), emits an explicit stderr NOTICE `notice: cosigner[{idx}] mk1 parent_fingerprint at depth {N} unverified (requires ms1 to derive parent xpub)` — explicit wontfix partition documenting the BIP-32 one-wayness ceiling. Failure mode: stderr WARNING / NOTICE only; verify-bundle exit code unchanged (permissive-input / expressive-output).
- **Status:** `RESOLVED in v0.25.0` (mechanism correction + ms1-driven derivation + watch-only NOTICE partition).
- **Tier:** `v0.25.0` (promoted from `v1+` per user direction).
- **Companion:** none.

### `gui-schema-global-flag-id-disjointness-debug-assert` — guard against future global-vs-local flag-id collision in v5+ emitter

- **Surfaced:** 2026-05-17, v0.24.0 Tranche B.1 end-of-phase architect review.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:1029-1031` skips args by `global_ids` set lookup; `:1042-1047` then re-emits globals via explicit list. `seen_flag_names` HashSet at `:1043` is dead defense — only the `global_ids` skip is load-bearing. Works correctly today because the only global flag is `--no-auto-repair` (boolean, no naming collision risk against subcommand-local flags). If a future cycle adds a global flag whose long-name collides with a subcommand-local flag of a DIFFERENT clap ID, the global would shadow the local.
- **What:** Add either (a) a debug-assert in `emit_flag` confirming `global_ids` IDs are disjoint from each subcommand's local `arg.get_id()` set, OR (b) a one-shot test cell asserting the same invariant. Optionally remove the `seen_flag_names` dead defense or rewrite it to be load-bearing (e.g., assert no name collisions across global-and-local rather than skip silently).
- **Why deferred:** Not a current correctness issue (no global flags besides `--no-auto-repair` today). Cheap to fix later; safer to file than to fold mid-cycle without expanding scope.
- **Resolution:** RESOLVED in v0.25.0 Phase 3 — added `pub(crate) fn assert_global_local_id_disjointness` helper in `cmd/gui_schema.rs` (load-bearing debug_assert; called from `build_subcommand` before global propagation). Deleted dead `seen_flag_names` defense per option (a) + supplementary cleanup. Cells: `global_local_id_disjointness_invariant_holds_in_current_schema` (integration, both debug + release) + `global_local_collision_triggers_debug_assert` (`#[cfg(debug_assertions)]`-gated `#[should_panic]` unittest).
- **Status:** resolved v0.25.0 cycle
- **Tier:** `v0.25.0`
- **Companion:** none.

### `cmd-repair-inspect-helper-duplication` — extract `count_dashes` / `expand_dashes` / `resolve_groups` shared between `cmd/repair.rs` and `cmd/inspect.rs`

- **Surfaced:** 2026-05-17, v0.24.0 Tranche C.1 end-of-phase architect review (during the D34/I5 fold).
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/repair.rs` — `count_dashes`, `expand_dashes`, `resolve_groups` (~80 LOC).
  - `crates/mnemonic-toolkit/src/cmd/inspect.rs` — byte-for-byte duplicates of the same three helpers (~80 LOC, modulo a `inspect:` vs `repair:` substring difference in two `BadInput` error messages).
  - `validate_flag_hrp` already extracted to `crates/mnemonic-toolkit/src/repair.rs` in the C.1 D34/I5 fold (this FOLLOWUP picks up the remaining `cmd/`-local helpers).
- **What:** Move the three shared helpers to a new `crates/mnemonic-toolkit/src/cmd/_shared_card_args.rs` (or co-locate next to `validate_flag_hrp` in `src/repair.rs`). Signature for `resolve_groups` would need to be parameterized over the per-subcommand error-prefix string (`"repair"` vs `"inspect"`) and the args-struct shape — likely cleanest via a small trait or a free function consuming `(Option<String>, &[String], &[String], &[String])` (ms1/mk1/md1/extra_strings + a `&'static str` subcommand-name for error messages). Update both `cmd::repair::run` + `cmd::inspect::run` to delegate. Subsumes any future verify-bundle equivalent if positional intake grows beyond the three-flag set.
- **Why deferred:** Tranche C.1 architect review surfaced this as Important #3 alongside D34 (Critical) + I5 (Important — confusing case-mismatch error). The D34/I5 folds were on the critical path for v0.24.0 ship; the helper-extraction refactor is purely DRY (no correctness gap) and was deferred to keep C.1's diff focused. Pattern is well-known: `synthesize_unified` extraction was the analogous earlier cycle (see CLAUDE.md memory `synthesize-unified-is-cli-hotpath`); duplicated helpers between sibling `cmd/` modules carry the same maintenance-drift risk.
- **Resolution:** RESOLVED in v0.25.0 Phase 1 — co-located next to `validate_flag_hrp` in `crates/mnemonic-toolkit/src/repair.rs` (option B). Added `pub(crate) trait CardArgs` with `ms1()`/`mk1()`/`md1()`/`extra_strings()` accessors; implemented for both `RepairArgs` + `InspectArgs`. Three helpers (`count_dashes`, `expand_dashes`, `resolve_groups`) now consume `&impl CardArgs`. `resolve_groups` parameterized over `subcmd_name: &'static str` for the 2 error-message substrings. Net -19 LOC across the cmd files; 1142 baseline tests preserved.
- **Status:** resolved v0.25.0 cycle
- **Tier:** `v0.25.0`
- **Companion:** none.

### `convert-inspect-auto-fire-tty-gate-asymmetry` — convert/inspect auto-fire lacks TTY gate; design-intent says interactive-only

- **Surfaced:** 2026-05-17, v0.24.0 Tranche A.1 end-of-phase architect review (during the `MNEMONIC_FORCE_TTY` doc-promote fold).
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/convert.rs:919` calls `crate::repair::try_repair_and_short_circuit(...)` UNCONDITIONALLY when `!no_auto_repair`.
  - `crates/mnemonic-toolkit/src/cmd/inspect.rs:85` same pattern.
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:155` consults `MNEMONIC_FORCE_TTY` + `is_terminal()` correctly (D18 design).
  - `docs/manual/src/40-cli-reference/41-mnemonic.md:505-508` reads "Interactive users see the helpful auto-fire UX; piped consumers see the v0.22.0-and-earlier behavior unchanged" — generic-sounding but only true for verify-bundle.
  - v0.24.0 cycle Tranche A.1 narrowed the `MNEMONIC_FORCE_TTY` scope claim at `:537-541` to verify-bundle-only, surfacing the asymmetry.
- **What:** Two resolutions:
  - (a) **Extend TTY gate to convert/inspect** for design-intent parity. Wraps `try_repair_and_short_circuit` calls in `if MNEMONIC_FORCE_TTY-aware is_terminal()` blocks. This matches the stated design intent + makes the env-var semantically uniform.
  - (b) **Document the asymmetry intentionally.** Update the manual at `:505-508` to explicitly note convert/inspect auto-fire is unconditional regardless of TTY (existing behavior is preserved; the doc just reads as design-intent).
  - User+architect choice; (a) is the more principled fix, (b) is the cheaper backfill.
- **Why deferred:** Latent issue, not a correctness regression. v0.24.0 A.1's scope was force-tty doc-promote, NOT TTY-gate-mechanism extension. Folding (a) would require expanding A.1 scope mid-cycle. Filing for future-cycle resolution; folding (b) is cheap if user prefers that approach.
- **Resolution:** RESOLVED in v0.25.0 Phase 2 — chose option (a). Single-gate-at-entry pattern: `let effective_no_auto_repair = crate::repair::resolve_no_auto_repair(no_auto_repair);` at top of `cmd/convert.rs::run` + `cmd/inspect.rs::run`; downstream `try_repair_and_short_circuit` sites thread `effective_no_auto_repair`. Mirror verify_bundle.rs:168-173 pattern. Manual L504-541 hoist + revert: A.1's narrowing paragraph reverted; new shared `### Auto-fire behavior (all three subcommands) (v0.25.0)` H3 section above `## mnemonic convert`. Behavior change documented in CHANGELOG `### Changed` with before/after example. 6 cli_auto_repair cells updated with `MNEMONIC_FORCE_TTY=1`; 3 new cells lock TTY-negative legacy path. Piped users opt back in via `MNEMONIC_FORCE_TTY=1` — same mechanism the GUI uses globally.
- **Status:** resolved v0.25.0 cycle
- **Tier:** `v0.25.0`
- **Companion:** none.

### `verify-bundle-empty-ms1-watch-only-sentinel-or-explicit-flag` — pre-v0.24.0 `--ms1 ""` watch-only sentinel hard-fails v0.24.0 strict-HRP gate

- **Surfaced:** 2026-05-18, v0.25.0 Phase 4 R0 architect review. The pre-v0.24.0 multi-cosigner watch-only convention was `--ms1 <s1> --ms1 ""` (empty string `""` per SPEC §5.8 as the watch-only sentinel for a specific cosigner). v0.24.0 §2.C.1 added a strict per-flag HRP validation gate (`crate::repair::validate_flag_hrp` called from `cmd/verify_bundle.rs::run` at `:162`) which rejects empty-string `--ms1` values for the "HRP must be lowercase canonical 'ms'" check. v0.25.0 Phase 4 worked around this by omitting the flag entirely for watch-only cosigners (`supplied_ms1.get(idx).map(...).unwrap_or("")` falls through cleanly), and updated the `cmd/verify_bundle.rs:62-67` doc-comment to describe omission as the new sentinel convention. But the SPEC §5.8 reference text still describes empty-string-as-sentinel; this FOLLOWUP captures the design question.
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:62-67` — doc-comment now describes omission (correct post-v0.25.0); historical pre-v0.24.0 doc said empty-string.
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:162` — `validate_flag_hrp("--ms1", "ms", v)?` rejects `""` strings at clap-parse-time.
  - SPEC §5.8 prose — describes empty-string sentinel; needs reconciliation OR explicit version-fork note.
- **What:** Choose one of:
  - (a) **Document the omission-as-sentinel convention canonically in SPEC §5.8.** Update SPEC to remove the empty-string-sentinel claim; add omission-as-sentinel description with `--ms1 <s1>` (skip cosigner) examples. Cheapest fix; treats the strict-HRP gate as authoritative.
  - (b) **Add an explicit `--watch-only-cosigner-index N` flag** that takes an integer index, marking the Nth cosigner as watch-only without needing any ms1 value at all. More expressive but adds flag surface.
  - (c) **Relax `validate_flag_hrp` to accept empty strings as the SPEC §5.8 sentinel** (with a separate check that rejects non-empty non-`ms1`-HRP values). Restores pre-v0.24.0 behavior but creates a hidden empty-string-special-case in the validator.
- **Why deferred:** v0.25.0 Phase 4's doc-comment fix is sufficient for the cycle close; the deeper SPEC reconciliation OR flag-surface change is a v0.26+ design question. No correctness regression — the new convention works.
- **Resolution:** RESOLVED in v0.25.1 (chose option (c) per user direction — relax `validate_flag_hrp` to accept empty strings, with explicit NOTICE-on-stderr to guard the accidental-empty-shell-variable footgun, plus option (a) SPEC §5.8 wording clarification documenting both equivalent CLI input forms). Restores the pre-v0.24.0 positional empty-string sentinel convention so middle-cosigner watch-only (e.g., `--ms1 <s0> --ms1 "" --ms1 <s2>`) is expressible — flag-omission alone can't represent this case. Mechanism: `crate::repair::validate_flag_hrp` early-returns `Ok(())` on `value.is_empty()` (alongside the existing `"-"` stdin exemption); `cmd::verify_bundle::run` iterates `args.ms1` and emits the NOTICE per skipped cosigner; SPEC §5.8 gains a new "CLI input forms" subsection. 1 new cell `watch_only_empty_ms1_sentinel_marks_cosigner_skip_with_notice` exercises middle-cosigner skip (un-expressible via flag omission alone). Released 2026-05-18 as v0.25.1 (single-FOLLOWUP patch).
- **Status:** resolved v0.25.1 cycle
- **Tier:** `v0.25.1`
- **Companion:** none.

### `bsms-first-address-verify` — toolkit-side first-address derivation + mismatch WARNING for 6-line BSMS Round-2

- **Surfaced:** 2026-05-18, Phase 2 R0 architect review of v0.26.0 wallet-import cycle.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:198-205` — `parse_six_line` path deferred verification; comment `let _ = &audit;` is the no-op fence.
  - `design/SPEC_wallet_import_v0_26_0.md` §4.1 (post-Phase-2-fold) — defers first-address verification to v0.27+.
  - `design/SPEC_wallet_import_v0_26_0.md` §2.4 row 3 (post-Phase-2-fold) — WARNING template struck through.
  - `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs::bsms_first_address_field_preserved_unverified` (post-fold rename) — pins audit-field preservation; no derivation-side assertions.
- **What:** v0.27+: derive descriptor at `<DERIVATION_PATH>/0/0` via existing toolkit derivation helpers, compute the address per the network detected in §4.2 step 8, and compare against `audit.first_address`. On mismatch, emit stderr WARNING per (restored) SPEC §2.4 row 3 template: `warning: import-wallet: bsms: first-address mismatch at path <P>: computed <C>, blob declares <D>` — informational only (exit 0, not hard-error). The check is BIP-129 §6's intended coordinator-output-self-consistency guard.
- **Why deferred:** Phase 2 surfaced that descriptor → address rendering at a specific derivation path is non-trivial in the v0.26.0 toolkit surface (no `derive_address_at_path` helper exists). The WARNING was informational-only (not hard-error), so deferring doesn't weaken the import-path correctness contract — concrete-keys checksum (BIP-380), xpub parse (`MsDescriptor::from_str`), and watch-only invariant remain load-bearing.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-signet-regtest-disambiguation` — coin-type-1 collapses signet/regtest to testnet

- **Surfaced:** 2026-05-18, Phase 0 R0 architect review I2 fold (during §7.0.a SPEC amendment) + cited in `wallet_import/bsms.rs:14-15` of Phase 2.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:14-15` — module-level doc comment citing the FOLLOWUP for the canonical testnet collapse rule.
  - `design/SPEC_wallet_import_v0_26_0.md` §4.2 step 8 — explicit normative text: "Signet and regtest are not distinguishable from testnet via origin-path inspection... imported as testnet."
  - Future `wallet_import/bitcoin_core.rs` (Phase 3) — same coin-type extraction will share this behavior.
- **What:** BIP-129 BSMS + Bitcoin Core `listdescriptors` origin annotations use coin-type `1` for testnet, signet, AND regtest — the blob is intrinsically ambiguous. v0.26.0 picks `Network::Testnet` as the canonical interpretation. v0.27+ may add either (a) a `--network signet|regtest` override on `import-wallet` (post-parse network re-binding), or (b) a separate origin-path-side disambiguator (e.g., a sibling `network_hint:` annotation that some wallets emit). User-direction needed before implementation.
- **Why deferred:** Surface-area trade-off: adding `--network` to `import-wallet` introduces a flag that 99% of users will never set (BIP-129 blobs don't carry signet/regtest as a separate type today), but the ambiguity exists and warrants explicit handling for users who run signet/regtest workflows. Testnet collapse is a safe v0.26.0 default.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-bsms-checksum-delegation-note` — SPEC §4.4 wording inaccuracy (checksum NOT auto-delegated)

- **Surfaced:** 2026-05-18, Phase 2 R0 architect review of v0.26.0 wallet-import cycle (implementer's finding 2).
- **Where:**
  - `design/SPEC_wallet_import_v0_26_0.md` §4.4 — text reads "BIP-380 8-character polymod checksum. Auto-validated when `MsDescriptor::from_str` is called by `parse_descriptor`."
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:26-27,140-145` — implementation calls `miniscript::descriptor::checksum::verify_checksum` explicitly up-front because `parse_descriptor::substitute_synthetic` (`parse_descriptor.rs:776`) swaps `@N` placeholders for synthetic xpubs BEFORE `MsDescriptor::from_str`, so the concrete-keys checksum never reaches `from_str`.
- **What:** Phase 7 SPEC-amend: rewrite §4.4 to describe the actual mechanism — "BIP-380 8-character polymod checksum. Validated UP-FRONT by `wallet_import::bsms::parse` via `miniscript::descriptor::checksum::verify_checksum` on the concrete-keys descriptor body, BEFORE the `concrete_keys_to_placeholders` adapter rewrites the body to `@N` placeholder form for `parse_descriptor`. The downstream `MsDescriptor::from_str` inside `parse_descriptor` operates on the synthetic-xpub-substituted form and cannot reach the original checksum." The implementation is correct; only the SPEC wording is wrong.
- **Why deferred:** Documentation-only fix; can ride the Phase 7 cycle-close SPEC-amend commit. No correctness change needed in code.
- **Status:** resolved (Phase 7 cycle-close commit; SPEC §4.4 amended with the up-front-validation prose; implementation unchanged at `wallet_import/bsms.rs:26-27,140-145`).
- **Tier:** `v0.26.0-cycle-close`
- **Companion:** none.

### `bsms-verify-signatures` — full BIP-129 HMAC token + signature verification on Round-2 ingest

- **Surfaced:** 2026-05-18, planned in `design/BRAINSTORM_wallet_import_v0_26_0.md` §6 item 2 as a cycle-close FOLLOWUP; cited in `crates/mnemonic-toolkit/src/wallet_import/mod.rs:65` of Phase 2.
- **Where:**
  - `design/BRAINSTORM_wallet_import_v0_26_0.md:264` — planned FOLLOWUP enumeration.
  - `design/SPEC_wallet_import_v0_26_0.md:150` — defers verification: "not verified in v0.26.0 (FOLLOWUP `bsms-verify-signatures`)".
  - `crates/mnemonic-toolkit/src/wallet_import/mod.rs:65` (approx; cite is in BsmsAuditFields doc-comment) — `signature_verified: bool` field locked to `false` in v0.26.0.
- **What:** v0.27+: implement full BIP-129 §5 HMAC token + signature verification flow. Coordinator's HMAC key derivation: PBKDF2(passphrase, salt, iterations) → HMAC-SHA256 over canonical Round-2 body. Toolkit-side flow: prompt for HMAC key (via `--coordinator-hmac-key <FILE>` or `@env:`), recompute HMAC, compare against `<SIGNATURE>` field; on success set `signature_verified: true`. Mismatch → exit 2 `ImportWalletParse` with stderr "bsms: signature mismatch — coordinator HMAC key does not match blob".
- **Why deferred:** Scope. v0.26.0 cycle target is parse + watch-only invariant + round-trip discipline; BIP-129 HMAC verification is a distinct security primitive that needs its own design pass (key-distribution UX, env-var/file/stdin input forms, refusal-on-missing-key default, etc.).
- **Status:** open — **NOTE:** the FOLLOWUP body wording above (and the v0.26.0 SPEC framing it reflects) misreads BIP-129. Per the v0.27.0 Phase 2 BIP-129 recon at `design/agent-reports/v0_27_0-phase-2-bip129-recon.md`: BIP-129's signature surface is NOT HMAC; it is BIP-322 legacy-format ECDSA recoverable signatures on **Round-1** (Signer → Coordinator) records, not Round-2. v0.27.0 closes this FOLLOWUP by implementing BIP-129-faithful Round-1 verify (NEW input path `--bsms-round1 <FILE>`), NOT the HMAC-keyed Round-2 verify the FOLLOWUP body initially called for. The HMAC primitives in BIP-129 are encryption-envelope MAC (PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256), separate from signature verify; that surface is filed at v0.27.0 cycle close as `bsms-bip129-full-cutover` for v0.28+.
- **Tier:** `v0.27`
- **Companion:** new sibling FOLLOWUP `bsms-bip129-full-cutover` (filed at v0.27.0 plan-revision time per Phase 2 recon pivot — see plan §8).

### `bsms-bip129-full-cutover` — complete BIP-129 conformance: 4-line Round-2 input parser + encryption envelope + deprecate v0.26.0 lenient parser

- **Surfaced:** 2026-05-18, v0.27.0 cycle Phase 2 BIP-129 recon (`design/agent-reports/v0_27_0-phase-2-bip129-recon.md`). v0.27.0's Path B-lite ships BIP-129 Round-1 verify (`--bsms-round1`) + BIP-129 Round-2 4-line emit (`--bsms-form 4-line`), but does NOT pivot the v0.26.0 6-line lenient input parser nor implement the encryption-envelope MAC surface.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:104-125` — the v0.26.0 6-line lenient parser whose `signature` field has no agreed verify semantics under BIP-129.
  - `design/SPEC_wallet_import_v0_26_0.md:152` — the documented lenient-input framing that motivated the 6-line shape.
  - `design/agent-reports/v0_27_0-phase-2-bip129-recon.md` — full BIP-129 spec recon (verbatim quotes from §Specification → Round 1, Round 2, Encryption + 5 in-spec test vectors).
- **What:** v0.28+:
  - (a) **Deprecate v0.26.0 6-line lenient parser.** Add stderr DEPRECATION notice when 6-line input is detected; planned removal in a future minor version.
  - (b) **Add BIP-129-faithful 4-line Round-2 input parser.** `BSMS 1.0` / `<descriptor>#<checksum>` / `<path-restrictions>` / `<first-address>`. Cross-validates the descriptor against the supplied path-restrictions + first-address (BIP-129 §Round 2 verify gate). Adds 4 + extra fixture coverage from BIP-129 in-spec test vectors.
  - (c) **Add encryption-envelope (STANDARD/EXTENDED) support.** PBKDF2-SHA512(`"No SPOF"`, TOKEN_raw_bytes, c=2048, dkLen=32) → ENCRYPTION_KEY → HMAC_KEY = SHA256(ENCRYPTION_KEY); AES-256-CTR decrypt of ciphertext + HMAC-SHA256 MAC verify per BIP-129 §Encryption. New CLI flag `--bsms-encryption-token <FILE|->` carrying the raw nonce. Refer to recon doc §2 for byte-level construction. Cross-impl smoke against Coinkite Python ref (`github.com/coinkite/bsms-bitcoin-secure-multisig-setup` `test.py`).
  - (d) Drop the v0.26.0 6-line shape (and possibly the 2-line lenient excerpt) after a stable-version deprecation window.
  - (e) Document the v0.26.0 → v0.27 → v0.28 BSMS history in `design/SPEC_wallet_import_v0_28+.md` + manual chapter at `docs/manual/src/40-cli-reference/41-mnemonic.md`.
- **Why deferred from v0.27.0:** Scope. v0.27.0 Path B-lite focuses on BIP-129 Round-1 verify + Round-2 emit (the two clean primitives that close the round-trip cycle). Adding the encryption-envelope primitives in v0.27.0 would ~double the cycle scope; deprecating v0.26.0's lenient parser pre-needs a stable BIP-129-faithful replacement input path (which requires the 4-line parser of (b) here). v0.28+ cycle.
- **Status:** open
- **Tier:** `v0.27-cycle-close`
- **Companion:** sibling of `bsms-verify-signatures` (v0.27.0 closes the Round-1 SIG subset of the original FOLLOWUP body's intent; this entry covers what stays open after that closure).

### `wallet-export-bsms-emitter` — `mnemonic export-wallet --format bsms` is unimplemented; blocks BSMS bundle round-trip cells

- **Surfaced:** 2026-05-18, Phase 4 implementer (commit `120e6b4`) noted at the "Phase 5 deferrals" section of the commit body; corroborated at Phase 4 R0 architect review.
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` — current emitter enumeration lacks `bsms`; the toolkit has only Bitcoin Core JSON + descriptor-passthrough surfaces at v0.25.x.
  - `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §4.5 (lines 308-318) — bundle round-trip BSMS direction is **structurally blocked** without the emitter.
  - `design/SPEC_wallet_import_v0_26_0.md` §7.1 — round-trip discipline assumes both formats can emit; BSMS direction was acknowledged blocked at Phase 4 R0.
- **What:** Implement the BSMS Round-2 export-side emitter so that `mnemonic export-wallet --format bsms` produces a BIP-129 lenient 2-line (or 6-line, with `--coordinator-hmac-key` for the optional MAC) output. Pairs with v0.26.0's import-side surface to close the bundle round-trip discipline cells deferred in §4.5. The 6-line shape additionally depends on `bsms-verify-signatures` (HMAC key material plumbing).
- **Why deferred:** Outside v0.26.0 scope; the cycle goal was import-side correctness + round-trip discipline. The 2-line emitter is feasible standalone (no HMAC required); 6-line emitter pairs with `bsms-verify-signatures`. Splitting into two FOLLOWUPs (2-line in v0.27.x, 6-line bundled with `bsms-verify-signatures`) is a viable plan.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none. (Pairs in spirit with `bsms-verify-signatures` for the 6-line shape.)

### `wallet-import-json-envelope-full-bundle` — `--json` envelope `bundle:` field is a parse-side summary, not the full toolkit-native `BundleJson`

- **Surfaced:** 2026-05-18, Phase 5 R0 architect review I2 (commit `ff1c85c` under review).
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:336-359` — `emit_json_envelope` hand-builds a summary `bundle_view: { cosigners: [...], network, threshold }`.
  - `crates/mnemonic-toolkit/src/wallet_import/mod.rs:59` — `ParsedImport.descriptor: md_codec::Descriptor` is parsed but never read by `cmd::import_wallet::run` (carries a narrow `#[allow(dead_code)]` per Phase 5 fold pointing at this FOLLOWUP).
  - `design/SPEC_wallet_import_v0_26_0.md` §2.2 — post-Phase-5-fold lock: v0.26.0 ships the summary shape; full BundleJson tracked here.
- **What:** v0.27+: wire the `--json` envelope's `bundle:` field to emit the full toolkit-native `BundleJson` shape (the same `verify-bundle --bundle-json` consumes — with synthesized ms1/mk1/md1 cards). This requires invoking the synthesizer post-parse against the supplied / overlayed seeds; for watch-only cosigners, emit the ms1/mk1 sentinel forms per SPEC §5.8. The `descriptor: md_codec::Descriptor` field on `ParsedImport` becomes load-bearing in this wire-up (currently unused).
- **Why deferred:** v0.26.0's scope was parse + watch-only invariant + round-trip discipline; envelope-side synthesis is a distinct integration with `crate::synthesize` that exceeds the cycle budget. The summary shape is forward-compatible with v0.27: the envelope key remains `bundle`, the shape itself extends. Downstream consumers encoding against v0.26.0 should target the summary; v0.27 will treat the legacy summary as a strict subset of the full shape.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-fixture-corpus-expansion` — broaden BSMS + Bitcoin Core fixture coverage to SPEC §10.1/§10.2 full set

- **Surfaced:** 2026-05-18, Phase 4 R0 architect review I1 (commit `120e6b4` under review).
- **Where:**
  - `crates/mnemonic-toolkit/tests/fixtures/wallet_import/` — v0.26.0 ships 8 BSMS + 5 Bitcoin Core fixtures (post-Phase-4-fold close M3 adds `bsms-2line-multi-2of3.txt` for declaration-order discipline).
  - `design/SPEC_wallet_import_v0_26_0.md` §10.1 / §10.2 — full corpus enumerated (12-15 per format); §10.1 amended post-Phase-4-fold to lock the v0.26.0 shipped subset + cite this FOLLOWUP.
  - `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §4.3 / §4.4 — full fixture list.
- **What:** v0.27+: ship remaining fixtures per SPEC §10.1/§10.2 — BSMS decay-4032, 6-line sortedmulti-2of3, sortedmulti-3of5, mainnet+ypub, mainnet+zpub, tr(NUMS,...) taproot; Core BIP-44 P2PKH, BIP-86 P2TR, wsh-sortedmulti 3-of-5, native `<0;1>/*` multipath, explicit `active: false` cell name. Each new fixture pairs with a per-fixture canonicalize-idempotency cell + a sniff cell.
- **Why deferred:** v0.26.0's shipped subset exercises the load-bearing canonicalize + idempotency + Core envelope-shape branches + declaration-order preservation invariant. Missing fixtures are coverage-expansion targets, not load-bearing for v0.26.0's correctness contract. Phase 4 implementer's commit body explicitly flagged the corpus reduction; the architect review confirmed the load-bearing paths are covered and the missing fixtures are expansion-class rather than correctness-class.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `gui-import-wallet-env-var-secret-channel` — v0.12.0+ GUI: auto-rewrite literal seeds in repeating `--ms1` widgets to `@env:MNEMONIC_MS1_<i>` sentinels + spawn-time env-var injection

- **Surfaced:** 2026-05-18, Phase 6 R0 architect review C1 (toolkit-manual cycle).
- **Where:**
  - `mnemonic-gui/src/runner.rs:74-114` — current spawn flow injects only `MNEMONIC_FORCE_TTY`; no per-cosigner secret env-var bag.
  - `mnemonic-gui/src/form/invocation.rs:236-251` — repeating-secret branch routes values verbatim.
  - `mnemonic-gui/src/main.rs:683-688` — run-confirm modal renders argv verbatim (per `[[feedback-run-confirm-modal-renders-argv-verbatim]]`).
  - `mnemonic-gui/tests/kittest_import_wallet_form.rs:154-213` — pins literal-pass-through contract.
  - `design/SPEC_wallet_import_v0_26_0.md` §9.3 — describes the aspirational behavior (toolkit-side accepts `@env:VAR` sentinels at parse time, but GUI does NOT pre-rewrite).
- **What:** v0.12.0+ `mnemonic-gui`: on subprocess spawn, collect per-cosigner-index secret values from `--ms1` repeating-widget state into a per-spawn env-var bag (`MNEMONIC_MS1_<i>=<value>`), rewrite `args[--ms1+1]` to `@env:MNEMONIC_MS1_<i>` sentinels, render the sentinel-bearing argv in the run-confirm modal, drop the env-vars on subprocess exit. Same pattern for `--passphrase`, `--share` (slip39-combine, seed-xor-combine), and other secret-bearing flags. Toolkit-side already accepts the sentinel at parse-time per Phase 1 cross-cutting helper.
- **Why deferred:** v0.26.0 scope is wallet-import-side parse + watch-only invariant + round-trip; the env-var-channel rewrite is GUI-side runner work that affects ALL repeating-secret surfaces, not just `--ms1`. Pre-existing `gui-run-confirm-modal-secret-redaction` (mnemonic-gui/FOLLOWUPS.md:454-462) covers the modal-redaction direction; this FOLLOWUP covers the argv-rewrite direction. Both need to land together in v0.12.0.
- **Status:** open
- **Tier:** `v0.12.0`
- **Companion:** `mnemonic-gui/FOLLOWUPS.md::gui-import-wallet-env-var-secret-channel` (primary). v0.26.0 manual prose calls out the user-must-type-explicitly fallback.

### `gui-import-wallet-cell-coverage-gap` — Phase 6 plan §6.4.a + §6.4.b + §6.8 + §6.9 cells deferred to v0.12.0+

- **Surfaced:** 2026-05-18, Phase 6 R0 architect review I2.
- **Where:**
  - `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §6.4.a-§6.9 — plan items not shipped at `b2e281a`.
  - `mnemonic-gui/tests/kittest_import_wallet_form.rs` — 8 cells shipped; 4 plan items not exercised (file-picker extension filter; blob-paste-textarea → stdin; env-var unset → subprocess exit 1; env-var no parent leak).
- **What:** v0.12.0+: ship the 4 deferred kittest cells. §6.8 + §6.9 ride `gui-import-wallet-env-var-secret-channel` close (no point asserting env-var lifecycle until the GUI does env-var injection). §6.4.a + §6.4.b are independent and can ship earlier.
- **Why deferred:** §6.4.a + §6.4.b ride file-picker / textarea widget plumbing not yet enumerated in the v0.11.0 GUI's file-selection surface (currently a one-shot file path string; no extension filter; no textarea paste). §6.8 + §6.9 ride the env-var-channel FOLLOWUP.
- **Status:** open
- **Tier:** `v0.12.0`
- **Companion:** none.

### `wallet-import-sparrow` — Sparrow Wallet JSON export-format ingest

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Sparrow parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: add `wallet_import/sparrow.rs` (or merged dispatcher) parsing Sparrow's wallet-export JSON shape. Inverse of `wallet_export::sparrow_wallet_emit` if a wallet_export-side Sparrow emitter exists; otherwise build forward.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-specter` — Specter-DIY JSON descriptor export

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Specter parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: add Specter-DIY JSON descriptor parser (non-BSMS path). Specter's wallet-export schema diverges from BSMS Round-2's line-oriented shape and from Bitcoin Core's `listdescriptors` envelope; needs its own sniff signature + parser.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-electrum` — Electrum 4.x wallet file ingest

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Electrum parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest Electrum 4.x wallet file (Python-dict-serialized JSON with `xpub` / `wallet_type` keys; multisig shapes via `x1`/`x2`/... per-cosigner subkeys). Encrypted variants (Electrum's stretched-key envelope) are out of scope; sibling FOLLOWUP for encrypted ingest if user-direction warrants.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-coldcard` — Coldcard wallet.json export (single-sig)

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Coldcard parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest Coldcard's single-sig `wallet.json` export shape (BIP-44 / BIP-49 / BIP-84 / BIP-86 per-path xpub blocks under a fixed envelope; Coldcard-specific provenance metadata). Multisig descriptor-text shape is tracked separately under `wallet-import-coldcard-multisig`.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-coldcard-multisig` — Coldcard multisig.txt (descriptor + cosigner list)

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Coldcard-multisig parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest Coldcard's multisig descriptor-text export (`Name`, `Policy`, `Format`, per-cosigner `Derivation` + xpub blocks; output script type as a separate header). Distinct shape from Coldcard's single-sig `wallet.json`; line-oriented `Key: Value` grammar more similar to BSMS than to Sparrow's JSON.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-jade` — Jade SeedQR or descriptor JSON export

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Jade parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest Blockstream Jade's `register_multisig` JSON-like export shape (multisig descriptor + per-cosigner xpub + name + threshold; signer-fingerprint annotations). SeedQR formats — distinct surface — may be folded later as an inline mode rather than a wallet-import format if user-direction warrants.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-bsms-round-1` — BSMS Round-1 share ingest (multi-cosigner setup phase)

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` — current parser handles Round-2 only (concrete descriptor + audit envelope).
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest BSMS Round-1 share files (per-cosigner contribution prior to coordinator assembly; carries token + signer-fingerprint + xpub but NOT the assembled descriptor). Multi-share collation requires N-of-N Round-1 inputs to produce a single Round-2-equivalent bundle; semantically distinct from single-blob ingest.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 only. Round-1 multi-share orchestration is a multi-input pipeline (vs Round-2's single-blob ingest); needs its own CLI surface (e.g., `--shares share1 share2 share3` repeating-flag) and threshold-consistency invariants.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-bsms-encrypted` — BSMS encrypted-envelope decryption + Round-2 ingest

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` — current parser handles unencrypted Round-2 only.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: decrypt BSMS encrypted-envelope shape per BIP-129 §5 (AES-CTR over the Round-2 payload keyed by a coordinator-shared token-derived key), then route the decrypted plaintext through the existing Round-2 parser. Requires CLI flag for the decryption key material (e.g., `--bsms-key <hex>` or `@env:BSMS_KEY` sentinel) and clear stderr templates for decryption failure vs format failure.
- **Why deferred:** v0.26.0 scope was unencrypted BSMS Round-2 only. Encrypted-envelope decryption is a distinct cryptographic surface that warrants its own design discussion (key material handling, argv leak vectors, key-derivation choice). The user can decrypt out-of-band today and pipe plaintext into `import-wallet`.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `coordinator-runbook-into-design-dir` — promote merge-plan-doc into `design/`

- **Surfaced:** 2026-05-18, v0.26.0/v0.11.0 cycle close — per architect R2 fold.
- **Where:**
  - Source: `.v0_26_0-merge-plan.md` (project-root scratch file, R3 — folds R0+R1+R2 architect reviews).
  - Destination: `design/PLAN_v0_26_0_three_way_merge.md` (canonical-record location per project convention; cf. `design/PLAN_v0_26_0_xpub_search.md`).
  - `CLAUDE.md` should cross-cite as the multi-instance coordination playbook.
- **What:** Copy `.v0_26_0-merge-plan.md` into `design/PLAN_v0_26_0_three_way_merge.md` (verbatim or with a "canonical record" header). Delete the scratch file at project root. Add a one-line bullet in `CLAUDE.md` Conventions section pointing at the design-dir copy as the recipe for future multi-instance cycles.
- **Why deferred:** Cycle-close polish; no correctness regression. The scratch artifact at root continues to function as the audit trail for the cycle itself; promotion is a future-reference cleanup.
- **Status:** resolved — v0.27.0 Phase 1 (file relocated with canonical-record header; `CLAUDE.md` Conventions cross-cite added; presence-smoke `tests/design_artifacts_presence.rs::three_way_merge_runbook_lives_in_design_dir` guards future churn).
- **Tier:** `v0.27`
- **Companion:** memory entry `[[project-v0-26-0-cycle-shipped]]`.

### `gui-schema-arm-drop-detector` — formalize `build_subcommand_conditional_rules` grep-c assertion as regression test

- **Surfaced:** 2026-05-18, v0.26.0 cycle close — per architect R0 finding I1.
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::build_subcommand_conditional_rules` — the match-block dispatcher whose arm count grew from a few to 11+ across v0.26.0 (4 xpub-search + 1 import-wallet + 1 compare-cost).
  - Proposed test location: `crates/mnemonic-toolkit/tests/cli_gui_schema_arm_count.rs` (or as a new cell in `cli_gui_schema_conditional_rules.rs`).
- **What:** Three-way merge of the dispatcher arm-set across concurrent feature PRs is silently-dropping-risky when two PRs insert at adjacent positions. Mitigation that worked this cycle: manual `grep -c '=> .*_conditional_rules()' crates/mnemonic-toolkit/src/cmd/gui_schema.rs` per rebase, with a documented expected count. Formalize as a `#[test]` that asserts the live count against a pinned constant; bumping the constant becomes the explicit signal whenever a new arm is added (and forces conscious decision-making in multi-PR rebases).
- **Why deferred:** Per-PR rebase verification worked this cycle; codification is hardening rather than gap-fix.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** `[[project-v0-26-0-cycle-shipped]]`.

### `error-rs-canonical-ordering-doc` — record alphabetical variant-ordering rule in CONVENTIONS

- **Surfaced:** 2026-05-18, v0.26.0 cycle close — per architect R0 finding I2.
- **Where:**
  - `crates/mnemonic-toolkit/src/error.rs` — `ToolkitError` enum and its `match self` blocks (Display impl, `exit_code`, `kind`).
  - Proposed doc location: `CLAUDE.md` Conventions section, OR a new `design/CONVENTIONS.md`.
- **What:** Adopt alphabetical-by-variant-name as the canonical ordering rule for:
  - the `enum ToolkitError` variant declarations, and
  - each `match self { ... }` block that exhaustively matches it (Display, exit_code, kind).
  Drift across concurrent feature PRs (9+ new variants in v0.26.0 cycle) is otherwise a guaranteed merge-conflict generator; the rule makes the resolution mechanical. Codify this in CONVENTIONS so future cycles converge on the same order without per-PR negotiation.
- **Why deferred:** v0.26.0 cycle resolved this in-flight via the plan-doc cheat-sheet (P-I2 fold); codification is for future cycles.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** `[[project-v0-26-0-cycle-shipped]]`.

### `compare-cost-agent-reports-back-fill` — persist architect reviews verbatim, in real time

- **Surfaced:** 2026-05-18, v0.26.0 compare-cost cycle close — per multi-aspect code review finding (CLAUDE.md line 30 compliance gap).
- **Where:**
  - `crates/mnemonic-toolkit/CLAUDE.md` line 30 ("Per-phase opus reviews persist to `design/agent-reports/`") — load-bearing convention; the compare-cost cycle violated it.
  - `design/agent-reports/compare-cost-cycle-meta.md` — meta-record back-filling the audit trail with commit pointers, but verbatim review text was lost.
- **What:** Establish per-cycle discipline: when an architect-review agent dispatch completes, **write its verbatim output** to `design/agent-reports/phase-N-r0-review.md` (or similar) BEFORE the per-phase fold-and-commit step. The compare-cost cycle's reviews were inlined in the session transcript only; a back-fill meta-record exists but verbatim text is unrecoverable from outside the transcript. Future cycles MUST persist verbatim — recommend wiring into a per-phase task with an explicit "write report file" step. Optionally extend the plan-doc template at `.v0_26_0-merge-plan.md` to enumerate this discipline.
- **Why deferred:** Convention codification; no per-PR regression. Future cycles will benefit from real-time persistence.
- **Status:** open
- **Tier:** `v0.27`
- **Companion:** none.

### `gui-workflow-trigger-include-release-branches` — CI gates silently skip PRs targeting release branches

- **Surfaced:** 2026-05-19, v0.11.0 GUI cycle — discovered mid-G2/G3 when no CI workflows queued for 14+ min after force-pushes on `compare-cost/p4-gui` and `worktree-xpub-search-v0-11-0`.
- **Where:**
  - Cross-repo: `mnemonic-gui/.github/workflows/build.yml` and `mnemonic-gui/.github/workflows/schema-mirror.yml`
  - Trigger blocks: `pull_request: branches: [master]`
- **What:** Both workflow files currently filter `pull_request: branches: [master]` — meaning **no CI fires for PRs targeting `release/v0.11.0`** (or any future integration branch). v0.11.0 cycle worked around this via local pre-merge vetting (`cargo build` + `cargo clippy --all-targets -- -D warnings` + `cargo test` with `MNEMONIC_BIN` pointing at the v0.26.0 toolkit binary) plus `--admin` merges against the integration branch. The integration PR (`release/v0.11.0 → master`) DID trigger workflows normally (base=master), so the load-bearing gate worked. Fix: extend trigger filter to `branches: [master, release/*]` so per-PR CI runs on integration branches too. Reduces reliance on out-of-band local vetting.
- **Why deferred:** Cycle workaround was sound and architecturally consistent (per plan-doc §G3.5.2, the integration PR is the load-bearing gate). Trigger-filter fix is a future-cycle ergonomics improvement.
- **Status:** open
- **Tier:** `v0.27` (cross-repo companion in mnemonic-gui).
- **Companion:** `mnemonic-gui/FOLLOWUPS.md::gui-workflow-trigger-include-release-branches` (this cycle close).
