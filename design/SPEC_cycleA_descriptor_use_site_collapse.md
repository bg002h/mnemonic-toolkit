# SPEC — Cycle A: descriptor use-site collapse fix (CRITICAL funds-safety)

**Status:** DRAFT rev-2 (post R0 round 1, Part 2 split). NO code until this + the implementation plan pass an
opus architect R0 review to 0 Critical / 0 Important (CLAUDE.md hard gate).
**Source SHA:** grep-verified against `origin/master` @ `8c8b9183` (2026-07-06).
**Scope (user-approved 2026-07-06):** **Part 1 (residue-reject floor) + Part 3 (uniform reject-with-remediation
across all import surfaces).** The bitcoin-core receive/change **pair-merge (former Part 2) is SPLIT** to a
dedicated follow-up cycle (R0 round-1 I-2: its `--select-descriptor`/`--json`/checksum/GUI-paired-PR ripple is
too large for this funds-critical cycle; Part 1 alone fully closes the funds hole).
**Bug source of truth:** `design/agent-reports/constellation-eval-2026-07-06.md` §1 C1; recon
`cycle-prep-recon-cycleA-descriptor-use-site-collapse.md`; R0 round 1 `design/agent-reports/cycleA-spec-r0-round-1.md`;
live anchor `design/CONTINUITY_cycleA_LIVE.md`.

---

## 1. Problem

`crates/mnemonic-toolkit/src/parse_descriptor.rs::lex_placeholders` (fn @:60) matches `@N[..]/<mpath>/*` with the
regex @:97-98, but the loop @:103-190 performs **no unconsumed-residue check**. A fixed use-site step such as
`/0` in `@0/0/*` (the standard Bitcoin Core / Sparrow / Blockstream Green single-descriptor receive form) matches
none of the trailing optional groups, so the `/0/*` is never captured into `PlaceholderOccurrence`.
`make_use_site_path` (@:290-302) then sees `{multipath: None, wildcard_hardened: false}` — i.e. the `/0/*`
**silently collapses to a bare `/*`**, encoding a DIFFERENT wallet into the md1 card.

**Un-representable, not merely unvalidated:** `md_codec::UseSitePath`
(`descriptor-mnemonic/crates/md-codec/src/use_site_path.rs:49-53`) has exactly two fields
(`multipath: Option<Vec<Alternative>>`, `wildcard_hardened`); `MIN_ALT_COUNT = 2` (:43) forbids a 1-alt
multipath; `wildcard_for` (`.../to_miniscript.rs:133-140`) ALWAYS emits a wildcard on restore. So a fixed single
step `/0/*` genuinely cannot ride in an md1 card. `xpub/0/*` derives `xpub/0/i`; `xpub/*` derives `xpub/i` —
**disjoint address sets**.

**Funds impact:** `bundle --descriptor` and `import-wallet --format descriptor|bitcoin-core|specter` encode the
wrong wallet; `verify-bundle` re-parses through the SAME collapsing lexer (`cmd/verify_bundle.rs:1307,1352-1357`)
so it **false-passes (exit 0)** even with the user's original descriptor + seed slot; `restore --md1` derives the
wrong address set. Single-sig-with-seed is masked by canonical regeneration; everything else unmasked.

**Oracle (authoritative-verified, BIP-84 Test vectors):** for `abandon×11 about`, correct first receive
`bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` at `m/84'/0'/0'/0/0`; the collapsed card currently (wrongly)
restores `bc1q8vph849lf3e9rrj85hsxrzlv949rtahe794k6p`. Zero test coverage of these shapes today.

## 2. Verified facts (cite in-code)

- **md-cli twin to mirror:** `descriptor-mnemonic/crates/md-cli/src/parse/template.rs` M5 residue reject @:128-137
  (`match_end = caps.get(0).end(); if next ∉ {')',',','}',whitespace,EOS} → reject`), placed AFTER the
  multipath-body validator (validator body ~`:77-110`; placement comment `:121-127`). **The toolkit ALREADY
  mirrors the multipath validator** (`parse_descriptor.rs:146-178`); ONLY the residue check is missing.
- **Adapt, don't copy:** the md-cli regex has a bare-origin group 2 (`((?:/\d+'?)*)`); the toolkit regex does NOT
  (origin path only inside `[...]`). A bare post-`@N` `/0` is a *use-site* residue (correct to reject), not an
  origin path. Do NOT "fix" by adding a group-2 origin capture.
- **Terminator set is complete (R0 round 1 Q5, verified):** `)` `,` `}` + whitespace + EOS are exactly the legal
  successors of a key expression in BIP-380/389 descriptors (arg-separator, wrapper-close, taptree-branch-close);
  none can appear where it would false-reject a valid continuation (verified vs multi-key, nested `sh(wsh())`,
  taproot-tree corpora).
- **`#` never directly follows a placeholder** — it follows the outer `)`, so it cannot false-reject on the direct
  `@N` path even where `#` is not pre-stripped (`bundle.rs:1389`, `verify_bundle.rs:1375`); on the import paths it
  IS stripped via `verify_checksum` (`bitcoin_core.rs:257` + specter/descriptor/bsms twins). (M-4 fold: this is
  the precise statement, superseding rev-1's over-broad "stripped before every call site.")
- **bare `@N`** is the canonical multisig keyless-template form (`json_envelope.rs:543`, `cli_ms1_slot.rs:64`
  `CANONICAL_DESC`, passing `bundle_two_cosigner(...)` in `cli_unrestorable_shape_advisory.rs:323`), used by the
  shipped `bundle --md1-form=template` (v0.60.0). ⇒ D1 deferred (§4).
- **Import surfaces feeding `concrete_keys_to_placeholders → parse_descriptor` (M-2 sweep, verified):**
  bitcoin-core, specter, `--format descriptor`, bsms, old-`--json` replay CAN carry a fixed `/0/*` step;
  sparrow/coldcard/electrum/coldcard_multisig hardcode `…/<0;1>/*` (`coldcard.rs:303`, `electrum.rs:607`,
  `coldcard_multisig.rs:705`, `sparrow.rs:380-393`) ⇒ unaffected — EXCEPT sparrow's descriptor-passthrough branch
  (must confirm at impl it can never forward a fixed use-site step).

## 3. Design overview

**Part 1 — residue-reject floor** (`lex_placeholders`): the fail-closed funds fix. Applies uniformly to every
caller (bundle/import/verify). **Part 3 — reject-with-remediation**: because Part 1 makes ALL fixed-step `/0/*`
imports reject, ensure each import surface produces a *pointed, actionable* message (not a mystery) and fix any
fixture that currently relies on the collapse. No new logic beyond messages + fixture/test correctness.
**Out of this cycle:** the bitcoin-core pair-merge (follow-up), which would have RESCUED Core imports from the
reject; until it ships, standard Core receive+change imports hard-fail loudly (workaround: combine to `<0;1>/*`
and import via `--format descriptor`).

## 4. Design decisions

- **D1 — bare `@N`: DEFERRED to follow-up `concrete-nonranged-xpub-implied-wildcard` (R0-confirmed sound).**
  A blanket bare-`@N` reject would break the canonical multisig keyless-template form (verified: `CANONICAL_DESC`
  + `bundle_two_cosigner` tests flow bare `@N` through lex). The genuine sub-concern (a CONCRETE non-ranged xpub,
  `wpkh([fp/84'/0'/0']xpub)` no `/*`, silently gaining `/*`) is INDISTINGUISHABLE from a keyless template at the
  lexer (wildcard-presence signal lost upstream at `concrete_keys_to_placeholders`) and must be handled upstream.
  Orthogonal to the residue floor. **Keep `lex_bare_at_zero` unchanged; file the follow-up with explicit funds
  framing (M-1); disclose the residual in the CHANGELOG** so users of non-ranged single-key descriptors are warned.
- **D2 — reuse `ToolkitError::DescriptorParse(String)`** (@:123, exit 2, kind `"DescriptorParse"`) on the
  encode/import paths (H13 precedent :162-174). **Verify-path error variant is PER-PATH** (plan-R0 I-B correction
  to M-3): the **concrete-descriptor** verify fork (`verify_bundle.rs:1352-1357` → `descriptor_concrete_to_resolved_slots`,
  `pipeline.rs:417-418`) re-wraps the lex reject as `DescriptorParse` → **exit 2** — this is the false-pass site
  SPEC §1 names, so the primary verify regression asserts exit 2 / `DescriptorParse`. The `@N`-**template** verify
  fork re-wraps as `DescriptorReparseFailed{detail}` (`verify_bundle.rs:1375`) → exit 4. No new variant, no
  schema/exit ripple.

## 5. Part 1 — residue-reject floor (THE funds fix)

**Change:** in `lex_placeholders` (`parse_descriptor.rs`), AFTER the multipath-body validator (:146-178) and
BEFORE `out.push(...)` (:183), add a per-occurrence unconsumed-residue terminator check mirroring md-cli
`template.rs:128-137`:

```rust
let match_end = caps.get(0).map(|m| m.end()).unwrap_or(0);
if let Some(next) = descriptor[match_end..].chars().next() {
    if !matches!(next, ')' | ',' | '}') && !next.is_whitespace() {
        // bounded/trimmed residue for the message
        return Err(ToolkitError::DescriptorParse(format!(
            "@{i}: derivation steps after the placeholder are not representable in md1; the use-site \
             path must be a multipath `/<a;b>/*` (or bare `/*`) as the final step — a fixed single \
             step like `/0/*` is un-representable (found residue near `{residue}`)"
        )));
    }
}
```

**Placement/rationale:** AFTER the multipath validator so a hardened/malformed `<…>` body keeps its byte-exact
H13 error (trap #6, R0-verified — the validator returns via `?` before this check). Terminator set = `)` `,` `}`
whitespace/EOS (trap #4). Message names the un-representable form + the multipath remedy (traps #8/#9 UX).

**Catches (all currently silent):** `@0/0/*`, `@0/0h/*`, `@0[fp/84'/0'/0']/0/*`, post-multipath `@0/<0;1>/0/*`,
pre-multipath `@0/0/<0;1>/*`, and (M-6) bare-unbracketed-origin `@0/48h/0h/0h/<0;1>/*` (residue `/48h…`).
**Does NOT catch** bare `@0)` (terminator — D1 deferred), `@0/<0;1>/*`, `@0/*`, `@0/*h`, keyless multisig
`wsh(sortedmulti(2,@0,@1))` — all R0-verified to still pass.

## 6. Part 3 — reject-with-remediation across import surfaces

Because Part 1 rejects every fixed-step import, each surface must fail *loudly and helpfully*. FIRST verify (at
impl) what form each surface's fixtures/importer actually consume TODAY (do any currently rely on the collapse?),
fix the fixtures to reality, and assert the reject.

- **bitcoin-core** (`wallet_import/bitcoin_core.rs`): Core always emits split `/0/*`(internal:false) +
  `/1/*`(internal:true). Both now reject. Message MUST point to the workaround: *"Bitcoin Core exports receive
  and change as separate `/0/*` and `/1/*` descriptors, which md1 cannot hold individually; combine them into the
  multipath form `…/<0;1>/*` and import via `--format descriptor` (automatic recombination is a planned
  follow-up)."* Document the interim limitation in the manual + CHANGELOG.
- **Specter** (`wallet_import/specter.rs`): shared `account_map` export is receive-only `/0/*` (no change branch;
  verified vs specter-desktop v2.1.10). Rejects. Message: Specter's QR/JSON omits the change branch; supply the
  full wallet-file JSON or the combined `<0;1>` form; NEVER silently assume `/1`.
- **`--format descriptor`** (`wallet_import/descriptor.rs`): a single `/0/*` descriptor rejects with the
  multipath-remedy message.
- **old `--json` replay** (`bundle --import-json`): `original_descriptor` stores raw `/0/*` (`bitcoin_core.rs:347`);
  replay re-parses through the same adapter → rejects. Message: this bundle was produced by a pre-fix version
  whose descriptor collapsed a use-site step; re-import from the source wallet. No silent acceptance.
- **BSMS `/**`** (trap #5): `xpub/**` → `@0[fp]/**`; the `wild` group eats `/*` leaving residue `*` → rejects.
  Verify `wallet_import/bsms.rs` handling; assert a pointed message. `wallet_export/bsms.rs:159-161` can emit a
  single-branch `/0/*` body — re-importing a self-produced BSMS file hits the reject; document.

## 7. Funds-safety traps — disposition (of the architect's 10)

In Cycle A: #4 (terminator set + `#`) → §2/§5; #5 (BSMS `/**`) → §6; #6 (residue AFTER validator) → §5; #8
(old-envelope replay) → §6; #9 (verify UX pointed error, `DescriptorReparseFailed{detail}`) → §4/§5; #10 (both
residue directions tested) → §8. **Moved to the pair-merge follow-up:** #1 (pairing predicate), #2 (order by
`internal`), #3 (actual step values), #7 (`--select-descriptor`/`--json` ripple) — none apply without the merge.

## 8. Test plan (TDD — failing tests FIRST)

**Part 1 (unit, `parse_descriptor.rs` lex tests):**
- Reject each dropped shape: `@0/0/*`, `@0/0h/*`, `@0[fp/84'/0'/0']/0/*`, post-multipath `@0/<0;1>/0/*`,
  pre-multipath `@0/0/<0;1>/*` (both residue directions — trap #10), bare-unbracketed-origin
  `@0/48h/0h/0h/<0;1>/*` (M-6). Assert `DescriptorParse` + exit 2.
- Positive controls (must still pass): `@0/<0;1>/*`, `@0/*`, `@0/*h`, `@0/<0;1>` sans-`*`, bare `@0)`
  (D1 deferred), `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))`, bare-multisig `wsh(sortedmulti(2,@0,@1))`,
  `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))`, nested `sh(wsh(...))`.
- `#`-never-lexed guard test (M-4 backstop).
**Funds regressions (CLI/integration):**
- **verify-bundle false-pass closure (M-7 mechanism):** a bundle-encode from `/0/*` now rejects, so the test
  CANNOT build a `/0/*` bundle post-fix. Instead: (a) verify a CONCRETE `/0/*` descriptor against any card and
  assert the reparse rejects (**exit 2 / `DescriptorParse`** — concrete verify fork, plan-R0 I-B/m1; the
  `@N`-template fork is exit 4 / `DescriptorReparseFailed`), and/or (b) load a pre-generated wrong-card fixture and
  assert verify now fails. Prove the false-pass (was exit 0) is closed.
- **BIP-84 oracle regression:** assert a correctly-encoded `<0;1>/*` card restores
  `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`; and that a `/0/*` input now rejects at encode rather than
  producing the collapsed `bc1q8vph849...` card.
**Part 3 (per surface):** bitcoin-core `/0/*` reject (+ workaround message), Specter receive-only reject,
`--format descriptor` `/0/*` reject, old-`--json` replay reject, BSMS `/**` reject.
**Migration — this list is NON-EXHAUSTIVE (I-1 fold, M-8):** the implementation plan MUST run
`grep -rn '/0/\*\|/1/\*' crates/mnemonic-toolkit/{src,tests}` and **classify EVERY hit** (rejects now / stays
passing / unrelated). **NO-WEAKENING RULE:** every `/0/*`-cell migration MUST assert the reject — NEVER silently
rewrite `/0/*`→`/<0;1>/*` to make a test "converge" again (that would delete the regression this cycle proves).
Known cells to migrate to a reject: `cli_cross_start_convergence.rs` (a4/a5 — literally encode the collapse as
"convergence"), `cli_import_wallet_bitcoin_core.rs` (all `/0/*` cells incl. `:898`
`core_fixture_file_mainnet_receive_change_pair_parses`, which flips from `bundles=2 .success()` to reject),
`cli_output_class.rs` (watch-only cells), `cli_older_advisory.rs` (concrete-key bundle cells),
`cli_export_wallet.rs::descriptor_to_bip388_non_multipath_refused` (STAYS-PASSING exit 1 — export-wallet parses via
`MsDescriptor::from_str` and NEVER enters the `@N` lexer; plan-R0 M-b supersedes rev-1's wrong "exit 2 not 1"),
`cli_compare_cost.rs::descriptor_wsh_wildcard_...`, `cli_descriptor_concrete.rs:174` (originless concrete `/0/*` —
classify: rejects at classify vs residue?), `cli_import_wallet_sniff.rs:79`, `coldcard.rs:728` (BSMS `/0/*` sniff
blob), `cli_import_wallet_bsms.rs`, Core fixtures `tests/fixtures/wallet_import/core-*.json`. **Must STAY passing
(already `<0;1>/*`, not affected):** `core_fixture_file_multipath_receive_change_pair_parses` (`:915`, different
keys) — it has a valid multipath, no residue; leave `bundles=2 .success()` unchanged (and note it as the future
merge-negative-control for the pair-merge follow-up).
**Suites:** `cargo test -p mnemonic-toolkit` (FULL — CLI/parse ripples into argv/schema/version lints) +
`cargo test -p wc-codec` (currently un-CI'd).

## 9. Lockstep ripples

- **Manual** (`docs/manual/src/40-cli-reference/`): update `import-wallet` (fixed-step reject behavior + error
  text) and add the **interim bitcoin-core limitation note** (Core receive+change imports hard-fail until the
  pair-merge follow-up; workaround = `<0;1>/*` + `--format descriptor`). Check the CLI-reference `@N` examples for
  any now-rejected fixed-step form (concrete `xpub/0/*` in prose is fine — those aren't `@N` placeholders).
- **GUI schema_mirror:** D2 adds NO flag/enum → **no ripple**. **No `--json` wire-shape change** (the pair-merge
  that would have changed it is split out). **No paired mnemonic-gui PR this cycle.**
- **Examples gate** (`examples.yml` / `docs/Examples.pdf`): sweep for any `@N`-placeholder example with a fixed
  use-site step that would now reject; regenerate if touched (none expected — examples use `<0;1>/*`).
- **Siblings:** md/mk/ms **NO-BUMP** (one-directional catch-up to md-cli's shipped M5; do NOT touch md-codec —
  `UseSitePath`'s inability to hold a fixed step is the CORRECT invariant enforced here).

## 10. Release ritual (per `project_toolkit_release_ritual_version_sites`)

Toolkit **MINOR** bump (plan-R0 M-d: this turns previously-accepted `/0/*`|`/**` imports into hard failures — a
breaking, user-visible behavior change; under 0.x semver a breaking change is a MINOR bump, matching the prior
funds-CRITICAL bughunt cycle which shipped MINOR): version + BOTH READMEs + `fuzz/Cargo.lock` +
`install.sh` SELF-pin (NOT the frozen md-cli sibling pin) + re-vendor iff a dep bumps (none expected) + CHANGELOG
(tag-gated) — the CHANGELOG MUST disclose (a) the funds fix, (b) the interim bitcoin-core-import limitation +
workaround, (c) the deferred concrete-nonranged-xpub residual (M-1). Direct-FF + tag. File + flip a new FOLLOWUP
slug for C1 (RESOLVED in the shipping commit). md/mk/ms unaffected.

## 11. Follow-ups to file

- **`bitcoin-core-receive-change-pair-merge`** (former Part 2): receive/change pair-merge → `<0;1>/*`. Carries
  the full R0 I-2 scope: pairing predicate (trap #1), order by `internal` (trap #2), actual step values (trap #3),
  `CoreSourceMetadata.internal: bool → Option<bool>`, `apply_select_descriptor` merged-entry rule, BOTH wire sites
  (`import_wallet.rs:1859` json + `:2265` text), merged-descriptor string-assembly + BIP-380 checksum recompute,
  paired mnemonic-gui PR, and pin `core_fixture_file_multipath_receive_change_pair_parses` (`:915`, different keys)
  as the merge-NEGATIVE-control. Own oracle-guarded, funds-reviewed cycle.
- **`concrete-nonranged-xpub-implied-wildcard`** (D1): upstream substitution-layer detection of a concrete xpub
  with no `/*` silently gaining a wildcard. Funds-adjacent; explicit funds framing.
- First-class fixed-step support in md1/`UseSitePath` — explicitly NOT pursued (md1 is multipath-centric).

## 12. Open questions for R0 round 2

1. With Part 2 split, does any Cycle-A test in the migration list still IMPLY a merge path (i.e. a genuine
   round-trip that must keep passing and therefore CANNOT just reject)? R0 round 1 found none for Part-1-only —
   confirm against the grep-sweep result the plan will produce.
2. Is the interim bitcoin-core reject message + manual limitation note sufficient, or does the loss of Core import
   warrant a louder signal (e.g. a dedicated exit code)? (Recommend no — reuse `DescriptorParse`/exit 2 per D2.)
