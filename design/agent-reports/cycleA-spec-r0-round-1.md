# R0 GATE REVIEW — SPEC_cycleA_descriptor_use_site_collapse.md — Round 1

**Reviewer:** opus architect. **Against:** `origin/master @ 8c8b9183`. Read-only, adversarial funds-safety posture.
**Verdict:** NOT GREEN — 0 Critical, 2 Important. Persisted verbatim per CLAUDE.md (transcript-only review text is unrecoverable).

---

## Summary of independent verification (what checks out)

**Part 1 residue-reject is correct and fail-closed.** Hand-traced the toolkit regex (`parse_descriptor.rs:97-98`) against every shape:

| Input (post-`concrete_keys_to_placeholders`) | regex match (group 0) | residue after `match_end` | verdict |
|---|---|---|---|
| `@0/0/*` | `@0` | `/0/*…` → `/` | REJECT ✓ |
| `@0/0h/*` | `@0` | `/0h/*…` → `/` | REJECT ✓ |
| `@0[fp/84'/0'/0']/0/*` | `@0[fp/84'/0'/0']` | `/0/*…` → `/` | REJECT ✓ |
| `@0/<0;1>/0/*` (post-mp) | `@0/<0;1>` (wild fails on `/0`) | `/0/*…` → `/` | REJECT ✓ |
| `@0/0/<0;1>/*` (pre-mp) | `@0` | `/0/<0;1>/*…` → `/` | REJECT ✓ |
| `@0)` (bare) | `@0` | `)` | PASS ✓ (D1 deferred, correct) |
| `@0/<0;1>/*` | `@0/<0;1>/*` | `,` or `)` | PASS ✓ |
| `wsh(sortedmulti(2,@0,@1))` | `@0`→`,`, `@1`→`)` | terminator | PASS ✓ (keyless template preserved) |

Critical structural point: the three descriptor separators `,` `)` `}` are exactly the chars that legally follow a key expression (arg-separator, wrapper-close, taptree-branch-close). `#` never directly follows a placeholder (it follows the outer `)`), so cannot false-reject on the direct `@N` path even where `#` is not pre-stripped (`bundle.rs:1389`, `verify_bundle.rs:1375`); on import paths it is stripped via `verify_checksum` (`bitcoin_core.rs:257` + specter/descriptor/bsms twins). **No currently-CORRECT card is false-rejected** (bare `@N`, `/<0;1>/*`, `/*`, `/*h`, `/<0;1>` sans-`*`, multi-key, `sh(wsh())`, `tr(NUMS,…)` trees all pass — verified `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))` and `wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*h))`).

**H13 preservation (trap #6) verified.** The multipath validator (`:146-178`) propagates its typed error via `?` BEFORE the residue check (placed after `:178`, before push at `:183`), so `@0/<0';1'>/*` still hits the byte-exact hardened-multipath reject.

**D1 deferral is correct (Q1).** `CANONICAL_DESC = "wsh(sortedmulti(2,@0,@1))"` (`cli_ms1_slot.rs:64`) and passing `bundle_two_cosigner("wsh(sortedmulti(2,@0,@1))")` (`cli_unrestorable_shape_advisory.rs:323`) flow bare `@N` through `lex_placeholders`; a blanket bare-`@N` reject WOULD break them. The concrete-nonranged concern is genuinely un-fixable at the lexer: `wpkh([fp/…]xpub)` (non-ranged) and keyless `wpkh(@0[fp/…])` both present as `@0[fp/…]` with no `wild` group — indistinguishable at lex; the wildcard-presence signal is only known upstream in `concrete_keys_to_placeholders`. Deferral sound.

**Part 2 is oracle-guarded.** §9's "merged `<0;1>/*` card; restore matches the oracle" + traps #1/#2/#3 make the merge invariants correct. Fixture `core-mainnet-receive-change-pair.json` exists.

**Citations spot-checked:** `lex_placeholders:60` ✓, regex `:97-98` ✓, `make_use_site_path:290-302` ✓, H13 `:162-174` ✓, `UseSitePath` `use_site_path.rs:49-53` ✓, `MIN_ALT_COUNT=2 :43` ✓, `wildcard_for` `to_miniscript.rs:133-140` ✓, md-cli M5 `template.rs:128-137` ✓, `bitcoin_core.rs:257/347`, `apply_select_descriptor mod.rs:394-444` ✓, `bundles[i].internal` at `import_wallet.rs:1859` (json) AND `:2265` (text) ✓.

## CRITICAL
**None.** Part 1 is fail-closed; it cannot emit a wrong card, only over/under-reject, and neither a false-reject of a correct card nor an under-reject of a collapsing shape was found. Part 2 is the only card-producing part and is oracle-guarded.

## IMPORTANT

### I-1. §9 migration set is provably incomplete AND lacks a no-weakening rule.
Anchor: SPEC §9; `cli_import_wallet_bitcoin_core.rs:898,915`; `cli_descriptor_concrete.rs:174`, `cli_import_wallet_sniff.rs:79`, `coldcard.rs:728`.
`grep -rn '/0/\*\|/1/\*' crates/mnemonic-toolkit/{src,tests}` surfaces omitted cells + two behavior-flips:
- **`core_fixture_file_mainnet_receive_change_pair_parses` (`:898`)** asserts `bundles=2` + `.success()`. Post-fix FLIPS to `bundles=1` (merge) or exit-2 (Part1-only) — a **count** change that shortens the `--json` `bundles[]` array (wire-shape effect for the GUI-paired-PR scope).
- **`core_fixture_file_multipath_receive_change_pair_parses` (`:915`)** is an already-`<0;1>/*` pair with DIFFERENT keys (FP_A bip84 + FP_B bip49). It must **stay** `bundles=2` (NOT merged — trap #1). Best existing negative-control that the merge does not conflate different-key entries; SPEC does not name it.
- Un-named affected: `cli_descriptor_concrete.rs:174` (originless concrete `/0/*`), `cli_import_wallet_sniff.rs:79`, `coldcard.rs:728` (BSMS `/0/*` sniff blob), `cli_import_wallet_bsms.rs`.
Failure scenario: an implementer migrating `a4`/`a5` (which encode the collapse as "convergence") "fixes" them by rewriting BOTH sides to `/<0;1>/*` → silently deletes the regression; or misses P10B.3 and a later merge refactor conflates different-key entries (funds bug) with no guarding assertion.
Fold: (a) mark §9 NON-exhaustive; mandate `grep -rn '/0/\*\|/1/\*'` + classify-every-hit in the plan; (b) add a **no-weakening rule** — every `/0/*`-cell migration MUST assert reject (or merge for the pair), NEVER silently swap `/0/*`→`/<0;1>/*`; (c) name the `:898` `bundles=2→1` flip; pin `:915` (P10B.3) as a REQUIRED merge negative-control (different keys ⇒ NOT merged).

### I-2. Part 2 wire-shape / select-descriptor semantics are self-inconsistent and touch a second un-named wire site — resolve in-SPEC or split Part 2.
Anchor: SPEC §6 "Ripple"; `mod.rs:411-441` (`apply_select_descriptor`); `CoreSourceMetadata.internal: bool` (`mod.rs:355`); `import_wallet.rs:1859` (json) + `:2265` (text).
§6 rules simultaneously "`bundles[i].internal … null/omitted`" AND "`active-receive`/`active-change` both resolve to the merged wallet." But `apply_select_descriptor` filters `m.active && !m.internal` (receive) / `m.active && m.internal` (change) over a **`bool`**. To satisfy BOTH selectors on a merged entry requires: (i) `CoreSourceMetadata.internal` `bool → Option<bool>` (or a `merged` flag); (ii) rewriting both active-* arms; (iii) updating BOTH wire sites — `--json` `"internal": meta.internal` (`:1859`) AND text summary `bundles[i].internal=` (`:2265`). SPEC names only `:1859`, so the GUI-paired-PR/manual scope is under-counted. Also unspecified for a card-producing path: how the merged `<R;C>/*` descriptor STRING is assembled per-key, and whether its BIP-380 checksum is recomputed — `bitcoin_core.rs:257` runs `verify_checksum` on the merged `desc`; a synthesized `<0;1>/*` string carries no valid Core checksum.
Failure scenario: implementer sets merged `internal=false` → `active-change` returns "no active-change descriptor found" for a bog-standard wallet (UX regression); or the merged descriptor fails `verify_checksum` (import breaks); or the text-summary consumer at `:2265` breaks outside the coordinated paired-PR.
Fold: EITHER (a) **split Part 2** (user-approved escape hatch) — ship Part 1+Part 3 now [RECOMMENDED given the ripple]; OR (b) enumerate in-SPEC: the `internal` field-type change, the `apply_select_descriptor` merged-entry rule, BOTH wire sites (`:1859`+`:2265`), and merged-descriptor string-assembly + checksum handling.

## MINOR
- **M-1.** Deferred concrete-nonranged case is a real, narrow, pre-existing funds-adjacent silent rewrite left open. Not a new regression; disclosed + filed. Fold: file `concrete-nonranged-xpub-implied-wildcard` with explicit funds framing; disclose the residual in release/CHANGELOG.
- **M-2.** §7 silent on sparrow/coldcard/electrum/coldcard_multisig. Verified coldcard (`:303`), electrum (`:607`), coldcard_multisig (`:705`) hardcode `…/<0;1>/*` ⇒ unaffected; sparrow synthesizes `…/<0;1>/*` (`:380-393`) but has a descriptor-passthrough branch. Fold: restate the sweep result; confirm the sparrow passthrough can never carry a fixed use-site step.
- **M-3.** On the verify path the lex reject is re-wrapped as `DescriptorReparseFailed` (`verify_bundle.rs:1375`), not `DescriptorParse` — detail carries the pointed message (trap #9 preserved). Note it so the trap-#9 test asserts the `DescriptorReparseFailed{detail}` shape on the verify path.
- **M-4.** §5's "`#` stripped before every `lex_placeholders` call site" is precise for import paths but not the direct `@N` paths (`bundle.rs:1389`, `verify_bundle.rs:1375`) — harmless only because `#` sits after the outer `)`. Reword; keep the `#`-never-lexed guard test.
- **M-5.** §2 "AFTER the multipath-body validator (md-cli :121-127)" — `:121-127` is the placement-COMMENT; validator body is ~`:77-110`. Cosmetic.
- **M-6.** Hand-written `@0/48h/0h/0h/<0;1>/*` (bare unbracketed origin) currently silently drops the prefix; post-fix correctly rejects (residue `/48h…`). Add one negative-test cell.
- **M-7.** §9's "build a bundle from a `/0/*` descriptor … assert verify-bundle FAILS" cannot build the bundle post-fix (encode now rejects `/0/*`). Test must (a) verify a `/0/*` descriptor against any card and assert reparse rejects (exit≠0), or (b) load a pre-generated wrong-card fixture. Clarify in the plan.

## Answers to SPEC open questions
1. D1 deferral correct — yes. 2. Merge-in-`parse()` vs card-production — entry-count 2→1 is unavoidable at bundle production regardless; merge-in-`parse()` is defensible but drives the I-2 ripple. 3. **Ship Part 2 or split? — SPLIT recommended.** Part 1 fully closes the funds hole; Part 2 is UX-restoration carrying the `internal`-type change + dual wire-site edits + `apply_select_descriptor` rewrite + GUI paired-PR. Split ⇒ Part 1+Part 3 ship immediately; Part 2 gets its own oracle-guarded, funds-reviewed cycle. If split, the interim state (Core receive+change imports hard-fail with a pointed message) MUST be documented in manual + release notes. 4. Specter — keep the verify-fixtures-first mandate; receive-only `/0/*` ⇒ Part 1 reject with remediation is correct. 5. Terminator set complete — yes; `) , }` + whitespace + EOS are exactly the legal successors of a key expression; none false-rejects a valid continuation. Verified vs multi-key, nested `sh(wsh())`, taproot-tree corpora.

## VERDICT
**NOT GREEN — 0 Critical, 2 Important.** Core funds fix (Part 1) correct/complete/fail-closed; D1 deferral sound. Fold I-1 (close the test-migration integrity gap — grep-sweep, no-weakening rule, pin P10B.3 negative-control) and I-2 (resolve Part 2 self-inconsistency + second wire site, OR split Part 2 [recommended]), persist this review, re-dispatch.
