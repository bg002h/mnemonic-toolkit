# SPEC R0 review — bip388-double-star-shorthand-support — round 1

**Verdict: NOT GREEN (0 Critical / 2 Important / 4 Minor)**
**Reviewer:** fresh independent opus architect, source basis `origin/master` `0964462d`.
**Dispatched:** 2026-07-06 (Cycle C, SPEC R0 loop round 1). Persisted verbatim before fold per CLAUDE.md.

The primary-source BIP attribution (§2), the equivalence-oracle design (§7.3), the idempotence claim (§5), and the SemVer/no-GUI-impact calls (§8-9) are all **sound**. Every §4 citation is **line-accurate**. The blockers: (1) the recommended two-call-site mechanism **provably misses a primary in-scope surface** — bare `@N/**` template via `bundle`/`verify-bundle --descriptor` — because that surface calls `lex_placeholders` **directly**, bypassing `concrete_keys_to_placeholders`; and (2) the misattribution/stale-text cleanup is incomplete, including the user-facing reject message itself.

## Critical
None. The I1 gap outcome is **fail-closed** (a valid form is rejected, never a wrong wallet accepted) → Important, not Critical. No accepted-output correctness or funds defect found.

## Important

### I1 — The recommended 2-call-site design misses the bare `@N/**` template surface; §5 comparison is materially incomplete
`bundle --descriptor` and `verify-bundle --descriptor` route by `classify_descriptor_form` into two branches; only ONE hits `concrete_keys_to_placeholders`:
- `src/cmd/bundle.rs:338-346` dispatch: `is_bip388_policy_shape`→`expand_bip388_policy` (JSON); `Concrete`→`bundle_run_concrete_descriptor`→`concrete_keys_to_placeholders`; **else `AtN`→`bundle_run_unified_descriptor`**.
- `src/cmd/bundle.rs:1389`: `let occs = lex_placeholders(&descriptor_str)?;` — raw `@N` string lexed DIRECTLY, no `concrete_keys_to_placeholders`.
- `src/cmd/verify_bundle.rs:1375`: AtN fork → `lex_placeholders(&descriptor_str)` directly (`DescriptorReparseFailed`, exit 4).
- `DescriptorForm` = `{AtN, Concrete}` (`pipeline.rs:175-180`); `@\d`-only → `AtN` (`pipeline.rs:189-196`).

So `mnemonic bundle --descriptor "wpkh(@0/**)"` → `AtN` → `lex_placeholders(1389)` → residue reject (`parse_descriptor.rs:203-213`), **unchanged** by an expander in `concrete_keys_to_placeholders`. Same for verify-bundle. **BIP-388 defines `/**` in the `@N` placeholder-template context**, so `@0/**` is the *canonical* BIP-388 spelling; the toolkit already accepts the sibling `wpkh(@0/<0;1>/*)` (AtN, lexed at 1389). The SPEC's §5 parenthetical ("the JSON path already handles `@N/**`") rests on a false premise: `expand_bip388_policy` handles `@N/**` ONLY inside JSON (`pipeline.rs:282-302`, gated by `is_bip388_policy_shape`); a bare non-JSON `@N/**` is handled by NEITHER expander.

**§5 comparison is backwards.** SPEC rejects regex-ext because "it does NOT reach xpub-search" (true) — but the shared-expander-in-`concrete_keys_to_placeholders` symmetrically MISSES the AtN direct-lex path, which regex-ext WOULD cover.

| mechanism | concrete | **AtN template** | xpub-search |
|---|---|---|---|
| shared expander in `concrete_keys_to_placeholders` + `parse_literal_xpub` (SPEC pick) | ✅ | **❌** | ✅ |
| regex-ext in `lex_placeholders` + `parse_literal_xpub` (recon opt-b) | ✅ | ✅ | ✅ |

§0 IN-1a also mischaracterizes routing: primary `bundle --descriptor` (AtN) is `lex_placeholders` direct (1389); `concrete_keys_to_placeholders` is only `--import-json` (bundle.rs:2096) + the concrete branch.

**Fix (pick one, explicitly):**
- **(A) Scope AtN IN** — adopt recon option-(b) lexer-native path OR apply the shared `expand_literal_double_star` at the two direct-lex AtN entry points (`bundle.rs:1389`, `verify_bundle.rs:1375`) AND at the top of `parse_descriptor` alongside `substitute_nums_sentinel` (`parse_descriptor.rs:875`) — note `parse_descriptor` also feeds the raw string to `MsDescriptor::from_str(&substituted)` at line 897, so expansion must land before that, not only inside `lex_placeholders` (which returns occurrences, not an expanded string). ~3-4 call sites, full coverage.
- **(B) Scope AtN OUT** — bare `@N/**` still rejects, with a remedy pointer; update §7.3 to test only concrete.

*(Bonus: `concrete_keys_to_placeholders` has ~10 callers not 4 — coldcard `coldcard.rs:313`/`coldcard_multisig.rs:498`, electrum `electrum.rs:370`, specter `specter.rs:226`, sparrow `sparrow.rs:411` — chokepoint-covered, fine, but §0 should acknowledge them.)*

### I2 — Misattribution + stale-behavior text cleanup (§2/§8) incomplete; omits the user-facing reject message
A full `grep -rn "BIP-389"` shows §2 misses live sites, and §2/§8 never touch the reject message whose meaning inverts:
- **`src/parse_descriptor.rs:189`** — comment `// placeholder (e.g. /0 in @0/0/*, or the BIP-389 /** shorthand, whose wild group eats only /* and leaves a stray *)`. Post-fix a concrete `/**` is pre-expanded and never reaches this residue check → actively-misleading comment at the fix site.
- **`src/parse_descriptor.rs:206-211`** — the reject MESSAGE: `"…a fixed single step like /0/* (or the /** shorthand) is un-representable…"`. Under (A) `/**` is representable → the `(or the /** shorthand)` clause is false/unreachable, must be dropped; under (B) reword to point at the remedy. User-facing.
- **`crates/mnemonic-toolkit/tests/cli_import_wallet_descriptor.rs:159`** — comment `"…or the BIP-389 combined shorthand (/**)…"`. §2 lists only `:191`.
- **`src/wallet_import/sparrow.rs:42`** — `"@N/** cosigner placeholders (BIP-389 multipath shorthand)"`.

`git grep` confirms every OTHER `BIP-389` hit is a CORRECT multipath reference (`<0;1>/*` is genuinely BIP-389) and must be LEFT (`41-mnemonic.md:141`, `sparrow.rs:372`, `45-foreign-formats.md:272/307/1038`, tech-manual glossary, `cli_import_wallet_bitcoin_core.rs`). Spot-check confirms §2's key directive: `41-mnemonic.md:141` = "BIP-389 **multipath** `/<a;b>/*`" (leave) vs `:145` = "BIP-389 **combined-wildcard shorthand** `/**`" (correct to BIP-388).

**Fix:** add `parse_descriptor.rs:189`, `parse_descriptor.rs:206-211` (message), `cli_import_wallet_descriptor.rs:159`, `sparrow.rs:42` to §2/§8; word the reject message jointly with the I1 (A/B) decision.

## Minor
- **M1** — §9 `install.sh` path wrong: it is `scripts/install.sh:32` (line# ✓); no repo-root install.sh. Also note the sibling pins right below (`scripts/install.sh:35` md-cli) are a FROZEN baseline — only the line-32 self-pin bumps; touching md-cli breaks `sibling-pin-check`.
- **M2** — §5 must pin the terminator set + clarify "final". Over-broad-match is SAFE only with a terminator lookahead: rewrite `/**`→`/<0;1>/*` ONLY when immediately followed by `)`, `,`, `}`, whitespace (per `parse_descriptor.rs:204`) plus `#` and EOS; excludes `/***` (next `*`) and `/**'` (next `'`). Clarify "final use-site step" = per-key/terminator-bounded — `wsh(sortedmulti(2,K0/**,K1/**))` has TWO, both must expand. Precedent to reuse: `substitute_nums_sentinel` (`parse_descriptor.rs:373`, invoked `:875`) — existing string pre-pass in exactly the right style/position.
- **M3** — §2 mislabels manual `:164` ("The `/**` shorthand rewrites to the same explicit `/<0;1>/*` form.") as an attribution site; it has no BIP number — it is semantic, swept by §8's 137-168 block rewrite. Move from §2 to §8.
- **M4** — §7.3 oracle should name the spelling: feed the concrete-xpub form for `bundle`/`verify-bundle`; if I1→(A), add an explicit AtN-form (`@N/**`) oracle cell; if (B), add an AtN-`/**`-still-rejects cell.

## Citation verification (§4 + §2/§9)
All §4 anchors **ACCURATE** (lex_placeholders@60; wild@97-99; residue@203-213/msg@206-211; reject-test@1731; concrete_keys_to_placeholders@330-400/push_str@391; expand_bip388_policy@282-302; parse_literal_xpub@291-298 direct from_str@297; parse_bip388_json@189-199; cli reject-test@191-217). §2 `41-mnemonic.md:141` leave/`:145` correct — ACCURATE. §2 correction-site list — **DRIFTED/INCOMPLETE** (misses parse_descriptor.rs:189, cli_import_wallet_descriptor.rs:159, sparrow.rs:42; I2). §9 install.sh — **PATH WRONG** (scripts/install.sh:32; M1). §9 gen.sh:44 — ACCURATE.

## Design adversarial findings
- Over-broad match (`/***`,`/**'`,stray `**`): **SAFE** given terminator-lookahead anchor — must be specified (M2).
- Idempotence w/ expand_bip388_policy (JSON `@N/**`): **SAFE** (JSON emits `/<0;1>/*`; no `/**` survives).
- xpub-search parse_literal_xpub covers literal + no-ops JSON/md1: **SAFE**.
- Two-call-site completeness (§5): **HOLE** — bare `@N/**` via bundle.rs:1389 / verify_bundle.rs:1375 uncovered (I1).
- "No 4th path" (§10.3): **HOLE resolved** — the 4th path IS the AtN direct-lex.
- §7.3 equivalence oracle non-tautological: **SAFE** (compares `/**` output vs independently-computed `/<0;1>/*` output; the `/<0;1>/*` path is pre-existing, not derived from the expander).
- Funds invariant "expanded `/**` ≡ explicit `/<0;1>/*`" (§6): **SAFE / correct**.
- SemVer MINOR + no GUI/schema_mirror: **SAFE** (no clap change; matches v0.77.0).

**To GREEN:** resolve I1 (choose A or B; correct §0/§5/§7.3) + I2 (extend §2/§8 to the 4 missed text sites incl. the reject message), fold M1-M4, re-dispatch (the I1(A) mechanism change can introduce drift).
