# IMPLEMENTATION PLAN — concrete-nonranged-xpub-implied-wildcard

**Executes `design/SPEC_concrete_nonranged_xpub_implied_wildcard.md` (R0-GREEN rev-2, SHA `e092f679`).**

- **Model (CLAUDE.md phase-3):** a SINGLE implementer subagent in a git worktree, TDD; per-phase opus R0 (FULL
  `cargo test -p mnemonic-toolkit`); mandatory post-impl whole-diff review.
- **Status:** ✅ **R0-GREEN (0C/0I) at round 1** (+5 test-precision Minor folds: M1 real-`[fp/path]xpub` fixture, M2 literal-`@N` template accept, M3 prefix-assertion scope, M4 message-anchor, M5 pre-existing-test caveat). Reviews: `design/agent-reports/cycleD-{spec,plan}-r0-round-1.md`. SPEC also R0-GREEN (rev-2). **Cleared for single-implementer TDD.**
- **Target:** `mnemonic-toolkit-v0.79.0` (MINOR); codecs NO-BUMP; NO GUI/`schema_mirror`; NO manual lockstep.
- **Branch:** `feature/concrete-nonranged-xpub-reject` off current `origin/master`, worktree.

---

## Guard-rails
- **TDD:** the §6 tests FIRST (RED for the right reason), then the check. The funds anchor (§6.2 verify-bundle
  false-pass-closed) is RED-first.
- **No `git add -A`; NEVER `cargo fmt --all`** (mlock.rs fmt-exempt); `cargo fmt -p` only; clippy `-D warnings`.
- **Per-phase gate:** full `cargo test -p mnemonic-toolkit` green + clippy + persist per-phase R0 to
  `design/agent-reports/cycleD-phase-P0-*.md` before advance.

## Phase P0 — the reject check + tests (the whole implementation)

**Single production change** — `src/wallet_import/pipeline.rs`, inside `concrete_keys_to_placeholders`'s
`for cap in re.captures_iter(descriptor)` loop, immediately after `let m = cap.get(0)…` (@341), BEFORE the
xpub decode (@351):
```rust
// concrete-nonranged-xpub-implied-wildcard (v0.79.0): a concrete [fp/path]xpub
// with NO `/…` derivation suffix is un-representable in md1 (UseSitePath always
// wildcards) — silently ranging it to `@N/*` engraves a different wallet and
// verify-bundle false-passes. Reject fail-closed. (`/`-suffixed keys pass through:
// `/*`/`/<a;b>/*` are ranged+representable; a fixed step `/0/*` is caught by the
// Cycle-A residue floor downstream.) key_regex never matches a hand-typed `@N`
// template, so that canonical form is unaffected (structurally unreachable here).
if !descriptor[m.end()..].starts_with('/') {
    return Err(ToolkitError::ImportWalletParse(format!(
        "import-wallet: bsms: parse error: concrete key @{idx} has no derivation \
         suffix; a fixed xpub cannot be represented in md1 (which always encodes a \
         ranged use-site) — append `/*` (ranged) or `/<0;1>/*` (receive/change) to \
         the key to intend a ranged wallet"
    )));
}
```
- **Byte-exact prefix (§5/M4):** message MUST begin `import-wallet: bsms: parse error: ` so the
  `descriptor_concrete_to_resolved_slots` remap (@413-414) → `DescriptorParse` exit 2 and importer `.replacen`
  both fire. (Exact wording is the implementer's, but the prefix is a hard acceptance criterion; a test pins it.)
- **`idx` is the current key index** (0-based `@N`) at the point of the loop — matches the SPEC's "@N" naming.
- Place the check where `idx`/`m` are in scope; do NOT alter the substitution/`last_end` logic.

**TDD — write ALL §6 tests FIRST (RED), then the check.** Locate the existing concrete-descriptor test file(s)
(grep `concrete_keys_to_placeholders` / `bundle --descriptor` / `verify-bundle --descriptor` tests) and add a
module or cells:
- §6.1 REJECT `bundle --descriptor "wpkh([fp]xpub)"` → exit 2, message names `@0` + `/*`/`/<0;1>/*` remedy.
- **§6.2 (FUNDS ANCHOR) REJECT the verify-bundle false-pass:** replay the recon 6-step repro — build the ranged
  card from `wpkh([fp]xpub)` is NO LONGER POSSIBLE (bundle now rejects), so construct the card via the ranged
  spelling `wpkh([fp]xpub/*)`, then `verify-bundle --descriptor "wpkh([fp]xpub)" --md1 <card> --mk1 <…>` now
  REJECTS (exit 2) at re-parse BEFORE card comparison, instead of `result: ok`. (Confirm the reject timing:
  the `?` at `verify_bundle.rs:1368` fires before `verify_emit_from_expected` @1373.)
- §6.3 ACCEPT `wpkh([fp]xpub/*)`; §6.4 ACCEPT `wpkh([fp]xpub/<0;1>/*)`; §6.5 ACCEPT hand-typed bare `@N`
  template (`wpkh(@0)` / `bundle --md1-form=template`) — pin against `lex_residue_floor_accepts_bare_at_n_d1_deferred`
  (parse_descriptor.rs:1905); §6.6 REJECT multisig with one non-ranged key `wsh(sortedmulti(2,[fpA]xpubA,[fpB]xpubB/*))`
  names `@0`; §6.7 `wpkh([fp]xpub/0/*)` still rejects via the floor (new check passes the `/` through); §6.8
  import-wallet descriptor/bsms concrete non-ranged rejects (right exit + prefix remap); §6.9 (M1) taproot
  `tr([fp]xpub)` rejects + `tr([fp]xpub/<0;1>/*)` accepts; §6.10 existing concrete tests stay green.
- **Message-prefix test:** assert the bundle/verify-bundle user-facing error does NOT contain a stray
  `import-wallet: bsms:` (i.e., the remap fired) and DOES name the key + remedy.

**Test guard-rails (PLAN-R0 M1-M5 — get the cells right):**
- **M1 — fixture:** every §6 cell MUST use a real `[fp/path]xpub` with ≥1 path component (`key_regex`'s path
  group `(?:/\d+(?:'|h)?)+` is mandatory — a no-path `[deadbeef]xpub` does NOT match `key_regex`, so the check
  never fires and a reject cell would pass for the WRONG reason "no [fp/path]xpub keys found"). Use the recon
  fixture `[73c5da0a/84h/0h/0h]xpub6CatWdiZ…` (derive the seed/xpub as the recon did: `bundle --template bip84
  --slot @0.phrase=<12-word abandon×11 about-vector>` then `inspect`; or reuse an existing with-path test
  vector).
- **M2 — §6.5 ACCEPT:** feed a hand-typed LITERAL `@N` descriptor (`wpkh(@0)`) — it routes via the AtN
  direct-lex path and NEVER enters `concrete_keys_to_placeholders` (the invariant). Do NOT feed a concrete
  descriptor with `--md1-form=template` (that correctly STILL rejects). Keep `lex_residue_floor_accepts_bare_at_n_d1_deferred`
  green + unmodified.
- **M3 — prefix assertion scope:** the "no `import-wallet: bsms:`" assertion applies ONLY to bundle/verify-bundle
  (fully stripped → `DescriptorParse`). §6.8 `import-wallet --format bsms` LEGITIMATELY keeps `import-wallet:
  bsms:`; `--format descriptor` → `import-wallet: descriptor:`. Do NOT write a blanket "no bsms: anywhere" test.
- **M4 — §6.2 anchor on the MESSAGE:** assert the parse-reject TEXT ("@0 … no derivation suffix …"), not
  merely exit 2 (a card-comparison failure also exits ≠0) — this is what proves the reject fired at re-parse
  BEFORE `verify_emit_from_expected`.
- **M5 — pre-existing test:** if the full suite surfaces a test asserting the OLD silent-accept, UPDATE the
  test (it encoded the bug), do NOT weaken the fix. (Corpus grep found ZERO such tests — the one
  concrete-non-ranged descriptor is an `xpub-search` `contains_at_n_placeholder` helper, off the choke-point
  path.)

**Gate:** full `cargo test -p mnemonic-toolkit` green; clippy clean; per-phase opus R0 (funds-weighted on the
false-reject/false-accept boundary + the verify-bundle-false-pass-closed anchor + the bare-`@N`-template
unaffected invariant).

## Post-impl whole-diff review (MANDATORY endpoint)
Fresh opus over the whole diff. Weighted to: the `!starts_with('/')` precision (no false-reject of ranged
forms, no false-accept of non-ranged), the bare-`@N` template unaffected, the verify-bundle false-pass genuinely
closed at re-parse, error-prefix remap correctness, and no collateral to other importers. Persist to
`design/agent-reports/cycleD-postimpl-whole-diff-review.md`.

## Release ritual (v0.79.0) — per SPEC §7
Version sites (v0.77.0/v0.78.0 sequence): Cargo.toml + workspace `Cargo.lock` + `fuzz/Cargo.lock` + both READMEs
+ `scripts/install.sh:32` self-pin (NOT frozen sibling pins) + `.examples-build/gen.sh` version-check + embedded
strings + regen `Examples.md` + **new CHANGELOG `[0.79.0]` entry (leave [0.76.0] intact)** + **flip
`design/FOLLOWUPS.md` `concrete-nonranged-xpub-implied-wildcard` → RESOLVED in the shipping commit**. Re-vendor
N/A (no dep bump; no `Cargo.toml` dep change). Direct-FF + tag `mnemonic-toolkit-v0.79.0` + push + verify all CI
gates green.

## Risk register (for plan R0)
1. **`!starts_with('/')` boundary** — the funds core; false-reject of any ranged form, or false-accept of a
   non-ranged one, is the whole bug. §6.3/6.4/6.9 (accept ranged) + §6.1/6.6/6.9 (reject non-ranged) bracket it.
2. **Bare `@N` template unaffected** (§6.5) — structurally unreachable (key_regex), but pin it.
3. **verify-bundle false-pass closed at re-parse before comparison** (§6.2) — the anti-C1 anchor; confirm timing.
4. **Error prefix remap** (byte-exact) across bundle/verify-bundle/import-wallet — no `bsms:` leak.
5. **`Examples.md` regen** must be version-string-only (no example exercises a concrete non-ranged descriptor —
   confirm at release; expected none).
