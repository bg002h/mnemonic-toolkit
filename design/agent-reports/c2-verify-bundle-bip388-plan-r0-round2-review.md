# C2 (verify-bundle BIP-388 intake) plan-R0 round 2 — architect confirmation (verbatim)

> Reviewer: opus architect (general-purpose, full tools). Confirms the round-1 folds (M1, M2,
> M6, M7) on `design/PLAN_C2_verify_bundle_bip388_intake_2026-06-16.md` @ toolkit `a69a9e3`.

---

**Verdict: GREEN (0C/0I)**

Round-2 confirmation of the R0 fold for `design/PLAN_C2_verify_bundle_bip388_intake_2026-06-16.md`. All four points verified against on-disk state at SHA `a69a9e3` (HEAD == origin/master):

**1. M1 fold — CORRECT.** Plan cell-3 sketch now explicitly states the assertion MUST key on `.stderr(predicate::str::contains("@N beyond keys_info"))` and must NOT use a bare `.code(2)` as the non-vacuity hook, with the correct rationale that exit-2 coincides pre-feature (`classify_descriptor_form` `(true,true)` → "mixes @N", pipeline.rs:136-138) and post-feature (expander residual-`@N` → `DescriptorParse`, pipeline.rs:201-205). The non-vacuity paragraph correctly says cells 1/2 flip on exit code (0→2) and cell 3 flips on the message. Matches the round-1 M1 finding exactly.

**2. M2 fold — CORRECT and re-confirmed against the test file.** Plan cell-1 and the citation table now cite `cli_verify_bundle_multi_cosigner_mk1.rs:94 audit_i10_same_xpub_two_paths_2of2_round_trips` as the structural template. Read the actual test (lines 94-157): it does a watch-only 2-of-2 round-trip via `bundle --json` → `verify-bundle --bundle-json <file>` with NO `--mk1`/`--md1` flags — cards come from the envelope. Confirmed `--bundle-json` `conflicts_with`/`conflicts_with_all` ms1/mk1/md1 at verify_bundle.rs:76,82,89,96. Accurate.

**3. M6/M7 folds — CORRECT.** Citation table now distinguishes positive bundle cells `:251` (multisig, `bundle_descriptor_bip388_policy_watch_only`) and `:269` (singlesig, `bundle_descriptor_bip388_singlesig_policy_watch_only`) from the negative `@N`-beyond cell `:230` (`export_wallet_bip388_policy_at_n_beyond_keys_info_refused`). The round-1 `:290` mislabel is gone. Lockstep section says exactly ONE README marker (`README.md:13` `<!-- toolkit-version: 0.56.0 -->`, confirmed verbatim) + ONE install.sh self-pin (`scripts/install.sh:32` `mnemonic-toolkit-v0.56.0`, confirmed verbatim). Both spot-checked on disk.

**4. No regression — CONFIRMED.** Core design unchanged and sound: the ~6-line insertion between verify_bundle.rs:685 (close of the `descriptor_str` read `match`) and :687 (open of the bare-concrete fork block, confirmed on disk) mirroring bundle.rs:319; MINOR → v0.57.0; no schema_mirror surface; manual note. Insertion point and control flow verified (expanded `@N`-free descriptor routes through the Concrete fork at :694, returns at :704-713, never reaching downstream `@N` machinery).

**One non-blocking wording nuance (Minor, not a fold error):** The plan's phrase "pinned-upstream.toml does NOT exist in the toolkit" was literally imprecise — `docs/manual-gui/pinned-upstream.toml` exists (pins `mnemonic-gui-v0.3.0` + frozen v1.0-era `toolkit-tag-implied = v0.13.0`); it is bumped only by a GUI-manual cycle, NOT by a toolkit release bump. The plan's substantive claim — no toolkit version self-reference for the release ritual to bump in this file — is correct. Recommend softening the literal phrasing. [FOLDED post-round-2: phrasing softened to "No toolkit version-bump self-reference in pinned-upstream.toml … is a GUI-manual pin bumped by a GUI-manual cycle".] Raised as observation only — not Critical/Important.

No drift introduced, no fold mis-applied, core design sound. Implementation may proceed.
