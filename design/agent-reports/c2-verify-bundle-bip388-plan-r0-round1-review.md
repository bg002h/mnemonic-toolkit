# C2 (verify-bundle BIP-388 intake) plan-R0 round 1 — architect review (verbatim)

> Reviewer: opus architect (general-purpose, full tools). Plan-doc:
> `design/PLAN_C2_verify_bundle_bip388_intake_2026-06-16.md` @ toolkit `a69a9e3`.

---

The gui-schema uses a `VERIFY_BUNDLE_FLAGS` constant (a flag-NAME set). Since the feature adds no new flag/option/subcommand/dropdown-value (only widens what `--descriptor` accepts), `VERIFY_BUNDLE_FLAGS` is unchanged → `schema_mirror` passes with no diff. The plan's "NO surface" claim is correct.

A malformed cell-3 policy with `description_template: "wsh(multi(2,@0/**,@1/**,@2/**))"` and only 2 `keys_info` entries: WITHOUT the feature, the raw JSON goes to `classify_descriptor_form`. The JSON text contains `@0`,`@1`,`@2` (at_n=true) AND the 2 inline `[fp/path]xpub` keys in keys_info (key_regex=true) → `(true,true)` → "mixes @N placeholders with inline keys" (exit 2). WITH the feature, `is_bip388_policy_shape` fires, `expand_bip388_policy` substitutes @0/@1, leaves residual @2 → `is_at_n_form` true → `DescriptorParse("...@N beyond keys_info")` (exit 2). Both exit 2, but different messages. The plan correctly states cell 3 must assert on the SPECIFIC `"@N beyond keys_info"` string, which fails when the feature is reverted (it'd say "mixes @N" instead). Non-vacuity holds.

---

**Verdict: GREEN (0C/0I)**

The plan is sound, accurately cited against the live `a69a9e3` tree (which is exactly `HEAD`/`origin/master`), and the ~6-line insertion is correctly placed. Implementation may begin. Minor observations below are non-blocking polish.

## Critical
None.

## Important
None.

## Minor

**M1 — Cell-3 non-vacuity rests entirely on the error-MESSAGE assertion, not exit code; the plan should make the assert string explicit in the cell.**
`crates/.../tests/cli_verify_bundle_hashlock_and_bip388.rs` (new cell 3). I verified the nuance the plan relies on: a malformed `@2`-beyond-`keys_info` policy exits **2 in both** worlds — pre-feature via `classify_descriptor_form`'s `(true,true)` "mixes @N placeholders with inline keys" (`pipeline.rs:136-138`), post-feature via `expand_bip388_policy`'s residual-`@N` → `DescriptorParse("...@N beyond keys_info")` (`pipeline.rs:201-205`). Because exit codes coincide (both 2), the *only* thing that flips on revert is the message. The plan says this (line 107-110) but the cell MUST `.stderr(contains("@N beyond keys_info"))` and must NOT also assert a bare `.code(2)` as its non-vacuity hook. Recommend the plan's cell-3 sketch spell out the exact predicate string so the implementer doesn't accidentally make it pass on exit-code alone. (This is the one place the "all three cells go RED on revert" claim is subtle — cells 1/2 flip exit 0→2, cell 3 only flips the message.)

**M2 — Cite the existing watch-only multisig precedent for cell 1's round-trip shape.**
The plan's cell-1 round-trip (2-of-2 watch-only policy → `bundle --json` → `verify-bundle --bundle-json`) is proven viable by `tests/cli_verify_bundle_multi_cosigner_mk1.rs::audit_i10_same_xpub_two_paths_2of2_round_trips` (:94-157): it does exactly a watch-only descriptor-mode 2-of-2 verify-bundle via `--bundle-json` with **no** `--mk1`/`--md1` flags (the cards come from the envelope; `--bundle-json` conflicts_with ms1/mk1/md1 — see args at verify_bundle.rs:76,82,89,96). So the answer to the review's pointed question — "does the multisig round-trip need mk1 cards passed?" — is **no**, when using `--bundle-json` the envelope supplies them. The multisig round-trip is therefore NOT fragile and cell 1 is a fine primary positive proof; the single-sig cell 2 is good defense-in-depth but not load-bearing. Recommend the plan cite `audit_i10` (not just `non_canonical_wsh_andor`) as the structural template, since it is the closest watch-only-multisig-via-bundle-json analogue.

**M3 — Insertion-point trace fully confirmed; note the shadowing covers all 6 consumers.**
`descriptor_str` is consumed at verify_bundle.rs:**694** (classify), **696** (body_no_csum), **717** (lex_placeholders), **734** (canonicity probe), **1042** (parse_descriptor). The proposed `let descriptor_str = if is_bip388_policy_shape(...) { expand... } else { descriptor_str };` rebind immediately after the read (685→687) shadows the binding for **all** of them. Since an expanded policy is `@N`-free concrete, it routes through the Concrete fork at 694 and returns at 704 — never reaching 717/734/1042. Control flow is correct. Also confirmed `--descriptor` reaches `descriptor_mode_verify_run` only via `run()`'s dispatch at verify_bundle.rs:286-295 (no earlier `descriptor_str` consumption in `run()`), so the descriptor-mode path is the sole entry and the expansion is early enough. No issue — just recording the trace so the implementer doesn't second-guess it.

**M4 — `is_bip388_policy_shape` uses `trim_start`, so the `--descriptor-file` `.trim_end()` (verify_bundle.rs:682) is harmless.** A file with a trailing newline is trimmed at the end; a leading-`{` (possibly after leading whitespace) is still detected by `s.trim_start().starts_with('{')` (pipeline.rs:173). Both `--descriptor` (string) and `--descriptor-file` funnel through `descriptor_str` at :685 before the insertion, so both gain the capability uniformly, as the plan's scope-guard claims (line 75-77). Confirmed correct.

**M5 — "Drop policy-NAME" decision (mirror bundle.rs not export_wallet.rs) is verified safe.** verify-bundle has no wallet-name/policy-name surface: its only `name` references are the fixed check-label constants (`ms1_decode`, `md1_wallet_policy`, etc.) and the template `human_name()`. It never emits or round-trips a user-facing wallet name. `bip388_policy_name` (pipeline.rs:215) would be genuinely dead code here. The plan's rationale (lines 50-53) is correct.

**M6 — Citation drift: two trivial inaccuracies, neither blocking.**
(a) Plan line 42 cites the v0.49.0 mirror tests as "`cli_bip388_policy_intake.rs` (bundle cells `:251`, `:269`, `:290`)". `:251` and `:269` are the two bundle watch-only cells (correct); `:290` is `bundle_descriptor_bip388_bare_key_policy_refused` (the bare-key refusal), NOT the `@N`-beyond cell the negative-cell-3 mirrors — that one is `export_wallet_bip388_policy_at_n_beyond_keys_info_refused` at `:230` (which the plan *does* correctly cite at line 104). Harmless mislabel in the citation table; the body text is right.
(b) Plan line 44 cites `FOLLOWUPS.md:4168-4177`; the entry header is actually at line **4168** with the `Status: open` line at **4174** (plan line 135 cites 4174 correctly). The range end 4177 is approximate (Companion line is ~4177). No correction required.

**M7 — All release-ritual lockstep sites confirmed and complete; one self-pin location to flag for the implementer.** The plan lists README marker, install.sh self-pin, pinned-upstream.toml, CHANGELOG, schema_mirror, manual lint. Live state: there is exactly **one** README marker (`README.md:13` `<!-- toolkit-version: 0.56.0 -->`) and **one** install.sh self-pin (`scripts/install.sh:32` `mnemonic-toolkit-v0.56.0`) — the plan's "README marker ×2" recollection (line 131) is from a prior cycle; today there is only one, so don't hunt for a second. `pinned-upstream.toml` does not exist in the toolkit (the plan hedges "any … toolkit self-reference" — correctly conditional). No sibling pin changes, so `manual.yml`/`quickstart.yml` stay frozen, as the plan states. MINOR vs PATCH: MINOR is correct — this is a new accepted input FORMAT (a new capability), matching the v0.49.0 `feat:` precedent. Bump 0.56.0 → 0.57.0 is right.

**M8 — No new `ToolkitError` variant; alphabetical-ordering convention not triggered.** `expand_bip388_policy` returns only pre-existing `BadInput` (error.rs:11, exit 1) and `DescriptorParse` (error.rs:123, exit 2). No `enum ToolkitError` change, so the alphabetical-by-variant-name convention (CLAUDE.md) does not apply. Confirmed.

### Process reminders (not findings)
- The fmt-gate note (plan line 134) correctly preserves the mlock.rs g6 exemption (`cargo fmt --all` then revert mlock.rs) per MEMORY.
- Per CLAUDE.md, persist this R0 review verbatim to `design/agent-reports/c2-verify-bundle-bip388-plan-r0-round1-review.md` BEFORE folding, then re-dispatch the architect after the fold (the reviewer-loop continues after every fold, even at R0). Given this is GREEN with only Minor items, a single confirming re-dispatch after folding M1's assert-string clarification should converge immediately.
