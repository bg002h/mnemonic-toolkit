# POST-IMPL WHOLE-DIFF REVIEW — Cycle D — round 1

**Verdict: GREEN (0 Critical / 0 Important)**
**Reviewer:** FRESH independent opus execution reviewer (cold read vs the R0-GREEN SPEC/plan).
**Scope:** `git diff cec8acc2..6b80adee` — 1 commit: `pipeline.rs` (+19, the reject check) + new `cli_concrete_nonranged_xpub_reject.rs` (+411, 11 §6 cells). Read-only. Persisted verbatim per CLAUDE.md.

The code faithfully, correctly, and safely implements the R0-GREEN design. No regression. Cleared for the v0.79.0 release ritual.

## Verification (firsthand, in the worktree)
`cargo test -p mnemonic-toolkit` → exit 0, every `test result:` `0 failed`, 0 warnings. New target 11/11. `cargo clippy -p … -- -D warnings` → exit 0. Diff exactly 2 files; `error.rs`/`mlock.rs` untouched; no new `ToolkitError` variant.

## Core funds question: is `!descriptor[m.end()..].starts_with('/')` exactly "no derivation suffix"? — YES
`key_regex` group 3 base58 class excludes `/`, so greedy match halts at the first non-base58 byte → `m.end()` lands right after the xpub, on `/` (ranged) or a terminator (non-ranged). Byte-exact discriminator. `m.end()` always ASCII char boundary → no panic; EOF → `""` → reject (correct).

Attack vectors — all SAFE:
- `xpub/*` → `/`→ACCEPT (§6.3). `xpub/<0;1>/*` → ACCEPT (§6.4/6.9). `xpub/**` (v0.78.0) → passthrough/expand-downstream, not false-rejected. `xpub/*'` hardened wildcard → passthrough (not this check's gate). `xpub/0/*` fixed step → passthrough → Cycle-A floor rejects (NOT the new check; §6.7 asserts both). nested `sh(wsh(…/*))`/`tr(…/*)` → key still `/`-followed → ACCEPT. `xpub)` non-ranged → REJECT exit 2 (§6.1). `xpub#…`/EOF non-ranged → REJECT (correct). multisig mix → per-capture `idx` at loop-top → first non-ranged named by true `@N` (§6.6 → `@0`). origin-less `tr(xpub)` → no `[fp/path]` → key_regex never matches → origin-required path, never silently ranged.
**No false-reject of any ranged form; no false-accept of any non-ranged form.**

## Per-question
3. **verify-bundle false-pass closed.** `verify_bundle.rs:1368` `descriptor_concrete_to_resolved_slots(...)?` propagates the reject BEFORE `verify_emit_from_expected` @1373. Test non-tautological: builds card via independent ranged `/*` spelling, asserts the reject MESSAGE (`@0`+"no derivation suffix"), `!result: ok`, `!import-wallet: bsms:` — proves re-parse reject, not card-mismatch.
4. **Hand-typed `@N` unaffected.** Check inside key_regex loop; key_regex never matches `@N`. `parse_descriptor.rs` 0 diff; `lex_residue_floor_accepts_bare_at_n_d1_deferred` intact @1906; §6.5 green.
5. **Error/exit consistent.** Byte-exact `import-wallet: bsms: parse error: ` @354 → bundle/verify strip → DescriptorParse; import parsers `.replacen` → `import-wallet: <fmt>:`; bsms keeps native prefix. Both exit 2 (error.rs:597,610) — remap exit-neutral. No `bsms:` leak.
6. **No collateral.** All 14 `concrete_keys_to_placeholders` callers are encode/import/verify surfaces where a non-ranged concrete key SHOULD reject. Pre-existing unit tests (pipeline.rs:512/528/538 ranged/keyless) unmodified + green — none encoded the old silent-accept (matches PLAN M5). Matrix maps 1:1 to §6.1-6.10.

## Minor observations (non-blocking, NOT findings)
- Pathological `[fp]xpub/` (trailing slash) / `[fp]xpub/z` (non-numeric) pass the syntactic check but are caught downstream (exit 2, SPEC §8.2) — not a funds hole.
- Message wording matches SPEC §5; the load-bearing byte-exact prefix is satisfied.

**GREEN — the v0.79.0 release ritual can proceed.**
