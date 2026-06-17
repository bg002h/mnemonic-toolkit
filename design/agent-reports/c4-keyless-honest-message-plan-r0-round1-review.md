# R0 architect review — PLAN_C4_keyless_honest_message (round 1)

**Plan-doc:** `design/PLAN_C4_keyless_honest_message_2026-06-17.md`
**Source SHA reviewed:** `1dec924` (HEAD == origin/master == v0.58.1)
**Reviewer:** opus architect (R0, pre-implementation hard gate)
**Date:** 2026-06-17

## Verdict: GREEN (0C / 0I)

3 Minor. None gate code. The make-or-break check (both existing
"must carry a key origin" assertions stay green) PASSES empirically.

---

## Verification performed

### Citations (all confirmed on disk @ `1dec924`)
- `pipeline.rs::classify_descriptor_form` — `:132-147`; the `(false,false)` arm `:141-145`. ✓
- `pipeline.rs::key_regex` — `:37-43` (`\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)`). ✓
- `pipeline.rs:454-462` — unit test asserting "must carry a key origin" on `wpkh(0279be667ef9…81798)`. ✓ (plan cites `:459`, the assert line — accurate.)
- `tests/cli_bip388_policy_intake.rs:290-298` — `bundle_descriptor_bip388_bare_key_policy_refused`, bare-xpub policy (`const A`, an `xpub…`) → "must carry a key origin". ✓ (plan cites `:298`, the assert — accurate.)
- `cmd/bundle.rs:325` — calls `classify_descriptor_form`. ✓
- `export-wallet --descriptor … --format descriptor` escape hatch — built the binary and RAN it: keyless `wsh(and_v(v:ripemd160(<40hex>),older(1234567)))` → **exit 0**, emits the watch-only descriptor (+ older() advisory). The same input to `bundle --descriptor` → **exit 2** with the current vacuous "must carry a key origin". ✓

### THE KEY RISK — both existing assertions stay green (item 2): PASS
Tested the plan's two `has_any_key_token` sub-patterns with the **actual Rust `regex` crate** (not just Python):
- `0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798` — exactly **66 hex**, prefix `02` → matches `\b0[23][0-9a-fA-F]{64}\b` → `has_any_key_token == true` → keeps OLD message. ✓
- bare xpub `xpub6FQya7zGhR92k…` → matches `[xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+` → `true` → keeps OLD message. ✓ (the bundle path EXPANDS the policy first, so the body classified is `wpkh(xpub…/0/*)` — still matches.)
- Ran the two baseline tests at HEAD: `--lib classify` (3 pass) + `cli_bip388_policy_intake bare_key_policy` (1 pass). Both GREEN today; the split preserves them because both inputs trip the probe.

Negative cases (correct → keyless message): keyless `sha256(<64hex>)` → `false`; keyless `ripemd160(<40hex>)` → `false`. ✓
`\b` boundary verified: the 66-hex token inside `wpkh(…)` is delimited by `(`/`)` (non-word chars), so `\b` fires; an over-long hex run does NOT match (the trailing `\b` requires a boundary after exactly 64 trailing hex). ✓

### Structural check (item 3): x-only-tr arm reachability
Tested key_regex against `tr([0a1b2c3d/86'/0'/0']<64-hex-xonly>)`: key_regex returns **false** (group 3 demands an `xpub`-family prefix; a raw 64-hex x-only key has none). So an x-only-tr key **WITH** a `[fp/path]` origin still reaches `(false,false)` — contradicting the plan's parenthetical that "the only x-only-tr that reaches the arm is the origin-less one." Under the split it would get the *keyless* message. This is a pre-existing exotic edge (raw x-only key, no xpub) and the status-quo message is also wrong for it ("must carry a key origin" when it DOES carry one). Acceptable; logged Minor M1. Normal `tr([fp/path]xpub…/*)` / `tr(xpub…/*)` contain an xpub → matched/Concrete → NOT mis-flagged (verified). ✓

### Item 4 — NO-BUMP vs PATCH
Both branches return `ToolkitError::DescriptorParse` → exit 2 (`error.rs:522`). No type/exit/flag/output change; only message text. No new `ToolkitError` variant (no alphabetical-ordering concern), no new exhaustive `match` (if/else inside the existing arm). NO-BUMP is defensible under the established error-text-non-stable precedent (SPEC §6.9 "byte-exact error text reference" exists, but message wording has been changed NO-BUMP before). Recommend: **NO-BUMP is fine**; a one-line CHANGELOG "Unreleased"/dev note is a nicety, not required (logged Minor M2).

### Item 5 — callers of the shared arm
Grepped: the ONLY direct callers of `classify_descriptor_form` are `cmd/bundle.rs:325` and `cmd/verify_bundle.rs:707`. **xpub-search does NOT call it** — it calls `expand_bip388_policy` directly (`cmd/xpub_search/descriptor_intake.rs:195`), and `export-wallet` deliberately uses `is_at_n_form` only (`export_wallet.rs:441-443`), NOT classify. So the plan's claim (plan §Design last line, "All existing `(false,false)` callers (bundle, the C2 verify-bundle path, xpub-search) inherit the split") **over-states the caller set** — xpub-search does not inherit it. Logged Minor M3. For the two REAL callers the honest message reads acceptably: a keyless descriptor genuinely can't be a coherent bundle in either bundle OR verify-bundle, so refusing is correct; the "export-wallet --descriptor" route is slightly off-context for verify-bundle (the user wanted to verify, not export) but not harmful.

---

## Critical
None.

## Important
None.

## Minor

**M1 — item-3 parenthetical is imprecise (no code impact).** `design/PLAN_…:38`.
The plan says the only x-only-`tr` reaching `(false,false)` is the origin-less one. Empirically (key_regex run), `tr([fp/path]<64-hex-xonly>)` — x-only key WITH an origin — also reaches the arm and would get the keyless message, because key_regex's group 3 requires an `xpub`-family prefix. This is an exotic degenerate input (raw x-only, no xpub, no derivation); the status-quo message is equally wrong for it. Recommend: tweak the parenthetical to "x-only-tr keys (origin-annotated or not) reach the arm; acceptable — raw x-only keys aren't bundleable cosigners regardless." No design change.

**M2 — NO-BUMP vs PATCH (recommend NO-BUMP, optional CHANGELOG dev-note).** `design/PLAN_…:81-84`.
Defensible as NO-BUMP (exit code + error type unchanged; user-facing text is non-stable per precedent). If the cycle keeps an "Unreleased" CHANGELOG section, a one-liner is a courtesy; not required. The plan already flags this as an R0 question — answer: NO-BUMP.

**M3 — caller-set claim over-states (xpub-search does not call classify).** `design/PLAN_…:60` (Design) and the item-5 framing.
Direct callers are bundle + verify-bundle ONLY. xpub-search reaches `expand_bip388_policy` directly and never `classify_descriptor_form`; export-wallet uses `is_at_n_form` only. Recommend: correct the sentence to "bundle and the C2 verify-bundle path" (drop xpub-search), so the test plan and FOLLOWUP record don't carry a wrong claim. No code change — the split itself is fine for both real callers.

---

## Sign-off
The hard gate is satisfied: the make-or-break regex check passes against BOTH existing literals (verified with the real Rust regex crate and by running the two baseline tests green), the refusal type/exit are unchanged, the escape hatch verifiably works (exit 0), and there is no new ToolkitError variant or exhaustive-match drift. The three Minors are wording/doc-accuracy and may be folded inline or at commit time. **Cleared to proceed to TDD/implementation.**
