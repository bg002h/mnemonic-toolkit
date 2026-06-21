# cycle-9 whole-diff review — md-cli lexer/parser robustness cluster

**Scope:** mandatory independent adversarial execution review over the cycle-9 md-cli diff (M5 funds wrong-address + M2/M10/M11 + L4/L7/L19), before tag/publish.
**HEAD:** `2ff9309` (`fix(cycle9-m5): reject multipath-not-last / unconsumed-suffix template (FUNDS)`)
**Base:** `836faf8` (`release: md-codec 0.38.0 + md-cli 0.8.1`)
**Commits reviewed:** `bbdfb93` (P1: M11/M10/L4/L19/L7) · `b43107f` (M2) · `2ff9309` (M5)
**Date:** 2026-06-21
**Reviewer:** opus software architect (adversarial; mutation-driven)
**Worktree:** `/scratch/code/shibboleth/wt-cycle9` (`descriptor-mnemonic`, branch `feature/cycle9-mdcli-parser`)

Method: every load-bearing claim was mutation-tested (revert/perturb the fix → confirm the guard goes RED → restore). All mutations restored; final worktree is byte-identical to HEAD (`git diff HEAD` empty). Full `cargo test -p md-cli` + `cargo fmt --all --check` + `cargo clippy --all-targets -- -D warnings` run on the clean tree.

---

## Critical

**NONE.**

## Important

**NONE.**

## Minor

**Minor-1 — M5 residue check tolerates an EOS-terminated placeholder (non-issue on the production path).**
`template.rs:129` `if let Some(next) = template[match_end..].chars().next()` — when the matched placeholder is the *last* token in the string (end-of-string), `next` is `None` and the residue check does not fire. Mutation probe `lex_placeholders("@0/<0;1>/*")` returns `Ok`. This is harmless: `parse_template` always wraps the placeholder in a descriptor with a trailing `)` (the next char is a terminator), and a bare un-wrapped lex is rejected downstream by `MsDescriptor::from_str`. No funds impact; documented here only for completeness.

**Minor-2 — M5 `match_end` uses `caps.get(0).map(...).unwrap_or(0)` (defensive default unreachable).**
`template.rs:128` — `caps.get(0)` is always `Some` inside a `captures_iter` body (group 0 is the whole match), so the `.unwrap_or(0)` fallback is dead. Harmless defensive code; clippy-clean. No action needed.

**Minor-3 — `is_bare_keypath_tr` classifies a structurally-invalid `tr(foo(@0,@1))` as SingleSig.**
`template.rs:2144-2179` — a nested call with a depth-1 comma and no depth-0 comma (e.g. `tr(foo(@0,@1))`) returns `true` → SingleSig. This is address-neutral: such a string is not a valid descriptor (miniscript rejects the unknown fragment), so it never reaches a real key/depth-gate decision; and the depth byte is advisory (brainstorm D6). No over-acceptance of a *genuine* script-path taproot (verified: `tr(@0,{...})`, `tr(NUMS,multi_a(...))`, `tr(@0,pk(@1))`, `tr(pk(@0),pk(@1))` all stay MultiSig). No action needed.

---

## Evidence — per-finding adversarial verification

### 1. M5 residue reject (edit-1, the real fix) — MUTATION-PROVEN

- **Byte-identity of the H13 substrate (mandatory gate):** `diff` of `template.rs:50-110` between base `836faf8` and HEAD is **IDENTICAL** — the lexer regex (`:55` `r"@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*(?:'|h)?)?"`), the permissive group-3 capture `([^>]*)`, and the strict in-loop validator (`:77-110`) are byte-for-byte unchanged. The substitution strip-class regex (`r"...(?:/<[0-9;]+>)?..."`) is byte-identical to base (moved `:498`→`:663` by inserted code above it, but the regex string itself is unchanged — grep-confirmed).
- **Placement:** the M5 residue check is at `template.rs:128`, strictly AFTER the group-3 block closes at `:110` (`};`) and BEFORE `wildcard_hardened` (`:138`). It can never land mid-loop.
- **MUTATION (edit-1 disabled):** neutralized the residue branch (`if false && ...`). Probe `parse_template("wpkh(@0/<2;3>/0'/*)")` built **`Ok`** with `use_site_path.multipath = Some([Alternative{2}, Alternative{3}])` over a tree the substitution rendered single-path (the `/0'/*` dropped) — **the exact divergent wrong-address card.** The integration test `m5_multipath_not_last_reject::encode_post_multipath_fixed_step_rejects` went **RED** (FAILED) under the mutation. Restoring edit-1: the form is REJECTED with a typed `CliError::TemplateParse` naming "the multipath `<…>` must be the final derivation step"; all 4 m5 tests GREEN.
- **Scope = text OUTSIDE `<…>` only:** probed `wsh(multi(2,@0/<0;1;2>/*,@1/<0;1>/*))`, hardened-wildcard tails (`/*'`, `/*h`), origin-before-multipath (`/48'/0'/0'/2'/<0;1>/*`), tap-leaf comma separators (`tr(@0/<0;1>/*,pk(...))`), trailing whitespace — **none false-fire.** The check examines `template[caps.get(0).end()..]`, which begins past the `>`, so the multipath body (group 3) is never inspected. Double-multipath `<0;1>/<2;3>` correctly REJECTS (the second `<` is residue).
- `h`-in-origin sub-case `wpkh(@0/48h/0h/0h/<0;1>/*)` REJECTS (origin class is `/\d+'?`, apostrophe-only; the `h` step is unconsumed residue). Verdict on (1): **edit-1 is the real, non-vacuous fix; it rejects the divergent template (mutation-proven) and does not over-reject valid forms.**

### 2. H13-PRESERVATION (the gate) — BYTE-IDENTICAL + FIRES-FIRST, MUTATION-PROVEN

- **Byte-identity:** confirmed above — group-3 capture (`:55`), validator loop (`:77-110`), substitution strip class (`:663`/ex-`:498`) all have ZERO `+`/`-` changes.
- **Reject INTACT:** bare hardened `<0'';1>` → `lex_placeholders("wpkh(@0/<0'';1>/*)")` errors with *"@0 multipath alt `0''` is hardened; …watch-only (xpub) card"* (H13's message). The 5 H13 unit guards + the 5 H13 integration tests (`h13_hardened_multipath_reject.rs`) are GREEN. The only diff to that integration file is a cosmetic `cargo fmt` reflow of an `assert_eq!(code, 1, …)` — no semantic change.
- **Fires-FIRST (ordering):** the fused `wsh(multi(2,@0/<0'';1>/0'/*,@1/<0'';1>/0'/*))` errors with H13's *"hardened"* message, NOT the M5 *"final derivation step"* message — the group-3 validator's `?` at `:107` returns before the M5 check at `:128` is reached.
- **MUTATION (ordering reversed):** inserted a copy of the residue check BEFORE the group-3 block (`:77`). The unit test `lex_fused_hardened_body_with_suffix_hits_h13_first` went **RED**: it surfaced *"derivation steps after the multipath group must be the final derivation step"* (the M5 message) instead of H13's hardened message — **proving the ordering test is non-vacuous and pins the guarantee.** (Note: the *integration* H13 tests stayed GREEN under this mutation because they assert only that a hardened body rejects, not *which* message; the unit-level `…hits_h13_first` test is the load-bearing pin — it exists and is non-vacuous.) Restored; all H13 guards GREEN. Verdict on (2): **H13's reject is INTACT, byte-identical, and fires-first (mutation-proven).**

### 3. The D4 DEVIATION — SOUND, non-vacuous predicate, NO over-rejection

The implementer deviated from spec D4. **The deviation is justified and the implementer's note (`template.rs:2054-2072`) is correct.**

- **(a) Spec's literal D4 was over-firing/unconstructible.** Spec D4: "compare `occ.multipath_alts.len()` to the substituted `DescriptorPublicKey` multipath-step count." But `substitute_synthetic` strips EVERY suffix (`/<…>` + `/*` + origin) and substitutes a BARE synthetic xpub — verified by direct probe: the substituted strings are `wpkh(xpub6Bem…)`, `wsh(multi(2,xpub…,xpub…))`, `tr(xpub…)` — every substituted key is single-path (count 0). For a *valid* `wpkh(@0/<0;1>/*)` the lexer alt-count is 2 and the substituted count is 0 → the spec's literal `2 == 0` check would **REFUSE every valid multipath template.** The spec's D4 as written is therefore both vacuous-on-the-substituted-side and over-firing — the implementer correctly identified this.
- **(b) The reinterpreted belt is SOUND.** "No substituted key may be a `MultiXPub` → else refuse." A surviving `MultiXPub` genuinely means substitution failed to strip a `<…>` body the lexer recorded (or the two regexes matched divergent spans) — exactly the divergence class M5 guards. Refusing in that case is correct.
- **(b) NON-VACUOUS predicate.** Mutation-probe: a REAL `<0;1>` xpub (`wpkh(XPUB_DEPTH4/<0;1>/*)`) parses as a `MultiXPub` → `for_each_key`'s `matches!(k, MultiXPub(_))` returns true. The predicate is fireable, not always-false.
- **(c) NEVER fires on the production path → NO false-fire / NO over-rejection.** Probe: for `wpkh(@0/<0;1>/*)`, `wsh(multi(2,…))`, `tr(@0/<0;1>/*)`, the substituted descriptor's keys are all `XPub`/`Single` — never `MultiXPub`. Confirmed by the GREEN positive controls: `parse_template_multipath_last_still_builds_and_preserves_multipath` (multipath `<0;1>` round-trips, alt-count 2 preserved) and the live mutation in §1 (the divergent card built `Ok` even with edit-2 ACTIVE — i.e. edit-2 did NOT fire on the divergent input; only edit-1 catches it).
- **Load-bearing vs redundant:** edit-2 is **redundant-but-harmless given edit-1** — it is a structural drift-belt that fires only if a *future* regex change breaks substitution so a `<…>` survives into the substituted string. On the current code it can never trigger. That is the correct posture for a "checkable drift belt" (D4's stated intent), and it is NOT a vacuous always-true assertion (the predicate is fireable per (b)). Verdict on (3): **the D4 reinterpretation is sound, the predicate is non-vacuous, and it does NOT over-reject valid multipath-last templates.**

### 4. M10 — no over-accept

- `ctx_for_template("tr(@0)")` / `tr(@0/*)` / `tr(@0/<0;1>/*)` / `tr(@0/0'/1/<0;1>/*)` → **SingleSig** (BIP-86 depth-3 accepted). `bare_keypath_tr_accepts_depth3_bip86_xpub` confirms an `m/86'/0'/0'` depth-3 xpub now parses.
- `tr(@0,{pk(@1),pk(@2)})`, `tr(NUMS,multi_a(2,@0,@1,@2))`, `tr(<NUMS-hex>,multi_a(...))`, `tr(@0/<0;1>/*,pk(@1/<0;1>/*))`, `tr(@0,pk(@1))`, `tr(pk(@0),pk(@1))`, `tr(@0,{{…},{…}})` → **MultiSig** (depth gate stays strict). Adversarial probe `m10_overaccept_probe` passed all cases. Malformed `tr(@0` (no close paren) → conservative MultiSig.
- The depth gate (`keys.rs:67-77`) is NOT relaxed. The `synthetic_xpub_for` depth flip (2nd consumer) is address-neutral (synthetic discarded after `key_map`; depth advisory). Existing `tr_key_only` / `tr_with_and_v_*` / `tr_tap_leaf_*` / `tr_multi_branch_*` tests GREEN.

### 5. M11 / M2 / L4 / L19 / L7

- **M11:** point check inserted at `keys.rs:80` (after depth gate, before the `bytes[13..78]` copy), `PublicKey::from_slice(&bytes[45..78])` on the compressed-pubkey slice. `0x02 || 32×00` (off-curve x=0) REJECTS with `BadXpub { i, "not a valid secp256k1 point" }`; real depth-4 xpub + real depth-3 BIP-84 tpub still parse (positive controls GREEN). No new Cargo dep (`bitcoin::secp256k1` already reachable).
- **M2:** `resolve_placeholders` bounds `max == 255` BEFORE the `(max+1) as u8` cast (base code panicked: `(255+1) as u8 == 0` → `0..0` density loop skipped → `by_i[&0]` bracket-index panic). `@255` → typed `TemplateParse` ("at most @254"), no panic, both `@0`-absent (`wpkh(@255/*)`) and `@0`-present (`wsh(multi(2,@0/*,@255/*))`) cases. `@254` boundary still ACCEPTS (`n == 255`). Bracket indexes replaced with checked `.get()` (defense-in-depth).
- **L4/L19:** all three sites (`repair.rs:158`, `encode.rs:73` JSON, `encode.rs:111` text) branch `class = if descriptor.is_wallet_policy() { WatchOnly } else { Template }`. `repair.rs` `_descriptor`→`descriptor` rename is clean (clippy-green). Keyed md1 → WatchOnly, keyless → Template (new tests + existing keyless tests GREEN). `byte_parity_advisory_lines` GREEN (advisory strings untouched; only the call-site class changed).
- **L7:** `main.rs:241` epilog rewritten — false "Non-chunked single-string md1 … are rejected with a wire-format error" GONE (grep count 0 in `repair --help`), replaced with "Accepts BOTH chunked-form … AND non-chunked … Since md-codec v0.35.0, single-string md1 are repaired directly." ATOMIC-SEMANTICS note retained.

### 6. Build / gates (clean tree)

- `cargo test -p md-cli`: **all GREEN** — 23 test-result groups, 0 failures (incl. 131 bin unit tests, `m5_multipath_not_last_reject` 4/4, `h13_hardened_multipath_reject` 5/5, `cli_output_class` 19/19, `cli_repair` 8/8).
- `cargo fmt --all --check`: **clean** (exit 0).
- `cargo clippy --all-targets -- -D warnings`: **clean** (exit 0).
- No flake observed across repeated runs. NOTE: the crate version is still `0.8.1` in the worktree — the bump to `0.9.0` is a Phase-4 ship step (per plan), expected at this pre-ship review point; NOT a finding.

---

## Verdict

**CYCLE-9 WHOLE-DIFF: 0C / 0I**

**GREEN (0C/0I, cleared to tag/publish).** All three funds-relevant gates are mutation-proven: (a) M5 edit-1 rejects the divergent `@0/<2;3>/0'/*` card (with edit-1 disabled it builds the `Ok{multipath=Some([2,3]), single-path tree}` wrong-address card and the m5 test goes RED); (b) H13's hardened/malformed reject is byte-identical, intact, and fires-first (ordering mutation makes `…hits_h13_first` go RED); (c) the D4 reinterpretation is sound, its predicate is non-vacuous, and it never over-rejects valid multipath-last templates (the spec's literal D4 would have over-fired; the `MultiXPub`-survivor belt is the faithful realization). M10/M11/M2/L4/L19/L7 all verified, no over-acceptance. 3 Minor notes are non-blocking. Build/fmt/clippy clean.
