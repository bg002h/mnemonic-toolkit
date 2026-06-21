# cycle-prep recon — 2026-06-21 — cycle-9 md-cli lexer/parser robustness (M5, M2, M10, M11 + L4, L7, L19)

**Findings repo (descriptor-mnemonic) origin/main SHA at recon time:** `836faf8` (`836faf87c3d82b119a9f0f5c6589a7db1f8613a4` — "release: md-codec 0.38.0 (MINOR) + md-cli 0.8.1 (PATCH) — funds-safety domain caps (cycle-4 H6/M4/I1)")
**Toolkit origin/master SHA at recon time:** `8d2fe505`
**descriptor-mnemonic local branch:** `main` was at `54dd765` (7 behind origin); recon performed against a detached checkout of `origin/main` = `836faf8` (verified bytes, not stale working tree).
**Toolkit local branch:** `feature/own-account-subset-search` (9 ahead / 26 behind master).
**Untracked (toolkit):** examples dir + 3 cycle-prep recon md files + 1 design/agent-reports md.

Findings live in **descriptor-mnemonic** (the report `design/agent-reports/constellation-bughunt-2026-06-20.md` is in the toolkit repo but cites md-cli source). Source-of-truth for fix scope/lockstep: toolkit `design/PLAN_constellation_bughunt_fix_program.md` (already pre-slots all 7 into **WS-MD-CLI-LEX** = M2/M5/M10/M11 and **WS-MD-CLI-ADVISORY** = L4/L7/L19).

**Registry chain:** md-cli is a **standalone published binary** (crates.io); the toolkit does **NOT** depend on md-cli (only on `md-codec = "0.37"` as a library). md-cli depends on md-codec by `{ path = "../md-codec", version = "=0.38.0" }`. **All 7 fixes are pure md-cli-side** — no md-codec change is needed, so **no toolkit pin bump is forced** by these fixes. Chain: md-cli MINOR → tag (`descriptor-mnemonic-md-cli-v0.9.0`) + `cargo publish`. Toolkit consumes none of this directly; a toolkit pin is OPTIONAL and only relevant if the GUI / a toolkit subcommand re-exports md-cli behavior (it does not — verify-bundle/restore use md-codec, not md-cli).

Drift expectation: **the report's line citations are STALE** (snapshots pre-dating cycle-1 H13 + cycle-2 H7 growth of `template.rs` from ~400 to 1972 lines). Mechanisms are all intact; one finding (M5) was mis-dismissed by a first-pass agent and is RE-CONFIRMED as reproducing below.

---

## Per-finding verification

### M5 — Lexer/substitution regexes truncate at the multipath, so a non-last multipath group → use-site path ≠ parsed descriptor (FUNDS / fidelity)
- **WHAT:** For a non-BIP-389-canonical template where the multipath group is NOT the final pre-wildcard component (e.g. `wpkh(@0/<2;3>/0'/*)`), the lexer records `multipath=[2,3]` while the substituted descriptor string keeps the unmatched `/0'/*` suffix as a literal single-path key (no multipath, origin `/0'`). The two views are stitched into one `Descriptor` despite describing different derivations → emitted md1's recorded use-site path disagrees with its structural tree → wrong derivation/address, silently (exit 0).
- **Citations:**
  - `template.rs:32-91` (lexer `lex_placeholders`) — **DRIFTED.** The function is now `template.rs:32-128`; the live lexer regex is at **line 55**: `r"@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*(?:'|h)?)?"`. Group2 `((?:/\d+'?)*)` = origin path **BEFORE** the multipath; group3 = multipath body; group4 = wildcard. **There is NO capture for an origin path AFTER the multipath.**
  - `template.rs:357-381` (substitution `substitute_synthetic`) — **DRIFTED.** Now at `template.rs:482-514`; the live substitution regex is at **line 498**: `r"@(\d+)((?:/\d+'?)*)(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?"`. Same group2/origin-before-multipath structure; multipath is non-capturing here.
  - Capture-group ordering claim (origin captured before multipath in both) — **ACCURATE** (confirmed both regexes line 55 / line 498).
  - `h`-marker-in-origin claim (`@0/48h/…` unmatched → malformed `XPUBh/…`) — **ACCURATE.** Origin class is strictly `/\d+'?` (apostrophe only, no `h`); an `h` step is left unconsumed by both regexes.
- **STILL-REPRODUCES:** **YES.** Traced through the full `parse_template` pipeline (`template.rs:1874-1921`): `lex_placeholders` truncates its match at `<2;3>` (group4 wildcard can't match the following `/0'`, so the regex match ENDS there), recording `multipath_alts=[2,3], origin_path=None`. `substitute_synthetic` replaces only the matched `@0/<2;3>` span, leaving the literal `/0'/*`, producing `wpkh(XPUB/0'/*)`. `MsDescriptor::from_str` (line 1884) parses that as a single-path key with `/0'` origin + wildcard, no multipath. `walk_root` (line 1886) builds the tree from the **substituted** string; `resolve_placeholders` builds `use_site_path.multipath=Some([2,3])` from the **lexer occs**. The final `Descriptor` carries `use_site_path.multipath=Some([2,3])` over a structural tree that has none → divergence. **There is NO cross-validation** between the lexer view and the substituted descriptor (grepped `template.rs` — no `mismatch`/`cross`/`leftover`/`residual` check), and **NO test** covers a non-last multipath (the only positional test, line 171, puts the multipath last). **Note:** a first-pass verification agent mislabeled M5 STRUCTURALLY-WRONG/no-repro on the (true) observation that "both regexes agree on grammar" — that misreads the finding: the bug is the *silent retention vs drop of the unmatched suffix*, not a disagreement in the regex grammar. Overridden → **REPRODUCES.**
- **Fix-site:** `template.rs:55` (lexer regex) + `template.rs:498` (substitution regex) — anchor the multipath as the ONLY-final pre-wildcard component and **reject leftover path chars after the multipath with a typed `CliError::TemplateParse`**; add a cross-validation of the lexer occ-view vs the substituted miniscript before emitting (belt-and-suspenders). This is the funds crux of the cluster.
- **Action for brainstorm spec:** Cite live regex lines `template.rs:55` and `:498` against SHA `836faf8` (NOT the report's `:32-91,357-381`). Frame the contract: BIP-389 multipath appears **once**, as the final derivation component before the wildcard. Decide REJECT (recommended, funds-safe) vs faithful-represent — REJECT matches the cycle-1 H13 precedent (never silently encode an un-restorable/divergent card).

### M2 — Placeholder index 255 overflows `n` to 0 → density check skipped → `by_i[&0]` panic (or silent wrong key-count when `@0` present) (PANIC/DoS)
- **WHAT:** Lexer accepts `@0..=255` (index parsed as `u8`); `n = (max_index + 1) as u8` wraps `256 → 0` for max index 255. The `0..n` density loop becomes empty (skipped); `by_i[&0]` then panics ("no entry found for key") when `@0` is absent (e.g. `wpkh(@255/*)`), or `n=0` flows silently into the encoder when `@0` is present.
- **Citations:**
  - `template.rs:188-201` (n-computation + panic) — **DRIFTED-by-~120.** The `n` cast is now at **`template.rs:307-312`**: `let n = (by_i.keys().max().copied().ok_or_else(...)? as usize + 1) as u8;`. Density check at **`:313-319`** (`for i in 0..n`). Panic site is **`:320`**: `let at0 = by_i[&0];` (bracket index → panics). The mechanism is byte-identical; only the line numbers moved.
  - "lexer accepts `@0..=255`" — **ACCURATE.** `template.rs:60`: `let i: u8 = caps[1].parse()` (so `@255` accepted, `@256`+ rejected at lex). Mirror in `keys.rs:83-88` `parse_index` (also `u8`, "index must be 0..255").
- **STILL-REPRODUCES:** **YES** on `836faf8`. `(255 as usize + 1) as u8 == 0`; `0..0` is empty → density check skipped; `by_i[&0]` at line 320 panics when `@0` absent.
- **Fix-site:** `template.rs:307-312` — bound the max index before the `as u8` cast (reject `> 254` with a typed `CliError::TemplateParse`); and/or `template.rs:320` — replace `by_i[&0]` with `by_i.get(&0).ok_or(...)`. (Both, ideally: the bound is the real fix; the checked get is defense-in-depth.)
- **Action for brainstorm spec:** Cite `template.rs:307-312` + `:320` against `836faf8` (NOT `:188-201`). E-panic-dos; PATCH-class (no input newly accepted; a previously-panicking input now errors cleanly).

### M10 — BIP-86 single-key taproot `tr(@0)` falsely rejected (depth gate treats all `tr(` as depth-4 multisig) (AVAILABILITY false-reject)
- **WHAT:** `ctx_for_template` maps only `wpkh(`/`pkh(`/`sh(wpkh(` to `SingleSig` (depth 3); every other head — including key-path `tr(@0/<0;1>/*)` — falls to `MultiSig` (depth 4). A real BIP-86 account xpub is depth 3 (`m/86'/0'/0'`), so `parse_key` rejects it ("expected depth 4 … got 3"). Depth byte is advisory (discarded in derivation), so relaxing the gate cannot corrupt addresses → pure availability, not wrong-address.
- **Citations:**
  - `template.rs:1792-1799` (`ctx_for_template`) — **DRIFTED-by-~132.** Function is now at **`template.rs:1924-1932`**: `if head.starts_with("wpkh(") || head.starts_with("pkh(") || head.starts_with("sh(wpkh(") { SingleSig } else { MultiSig }`. `tr(` falls through to `MultiSig`. Confirmed.
  - `keys.rs:67-77` (depth gate) — **ACCURATE.** `keys.rs:67-77`: `let depth = bytes[4]; let expected_depth = match ctx { SingleSig => 3, MultiSig => 4 }; if depth != expected_depth { return Err(BadXpub{...}) }`.
- **STILL-REPRODUCES:** **YES.** `md encode "tr(@0/<0;1>/*)" --key @0=<depth-3 BIP-86 xpub>` → `ctx_for_template` returns MultiSig (no match for the 3 single-sig heads) → `parse_key` expects depth 4 → rejects the depth-3 xpub with exit 1.
- **Fix-site:** `template.rs:1927` — classify single-`@i` key-path `tr(` as SingleSig (depth 3); OR relax the depth gate at `keys.rs:68-77` to accept 3-or-4 for taproot (depth never participates in derivation). Primary edit is the classifier; the gate is the secondary consumer.
- **Action for brainstorm spec:** Cite `template.rs:1924-1932` + `keys.rs:67-77` against `836faf8` (NOT `:1792-1799`). **SemVer driver:** this WIDENS accepted input (`tr(@0)` BIP-86 now accepted where it was rejected) → **MINOR** behavior change → makes the whole md-cli release MINOR. BIP-86 fact: account xpub is at `m/86'/0'/0'` = depth 3 — VERIFIED against BIP-86 (single-key P2TR, no script tree). Beware: distinguishing key-path `tr(@0...)` from script-path `tr(NUMS,{...})` / `tr(@0,{...})` — only the bare single-key form is depth-3; a tr() with a script tree leaf still uses multisig-context keys. Scope the classifier to "tr( with exactly one @i and no internal `{`/`,`".

### M11 — `parse_key` accepts an off-curve xpub (no secp256k1 point check); failure deferred to derive (CORRUPT-INPUT accept)
- **WHAT:** `parse_key` validates base58check / length / version / depth then blindly copies `bytes[13..78]` (chaincode‖pubkey) WITHOUT checking `bytes[45..78]` is a valid compressed secp256k1 point. An off-curve (e.g. all-zero) pubkey passes intake, encodes into the Pubkeys TLV, and only fails later at `derive_address`.
- **Citations:**
  - `keys.rs:33-80` (esp. `:78-79` payload copy) — **ACCURATE** (modulo function now spanning `keys.rs:24-81`). The copy is at **`keys.rs:78-79`**: `let mut payload = [0u8; 65]; payload.copy_from_slice(&bytes[13..78]);`. Grepped the whole file: **no `PublicKey::from_slice`, no `secp256k1`, no `Xpub::decode`, no `is_valid`** — only `use bitcoin::base58;` (line 2). Confirmed absent.
- **STILL-REPRODUCES:** **YES.** An xpub with valid base58check/version/length/depth but an off-curve `bytes[45..78]` passes `parse_key` and returns `Ok(ParsedKey{...})`. `bytes[45..78]` = the 33-byte compressed pubkey; `bytes[13..45]` = chaincode.
- **Fix-site:** `keys.rs:78` — insert `bitcoin::secp256k1::PublicKey::from_slice(&bytes[45..78]).map_err(|e| CliError::BadXpub{ i, why: ... })?;` BEFORE the copy. The `bitcoin` crate (with secp256k1) is already a dep.
- **Action for brainstorm spec:** Cite `keys.rs:78-79` against `836faf8`. **Classification:** TRIVIAL→**FORMAL** — this adds a REJECT of previously-accepted input (rule clause 1/3), so it goes through the formal TDD lane. PATCH-class on its own (rejecting nonsense input is not a feature add), but rides the MINOR umbrella from M10. secp256k1 fact: a valid BIP-32 xpub's `bytes[45..78]` MUST be a compressed point on secp256k1 — VERIFIED (BIP-32 serialization).

### L4 — `md repair` always emits "keyless template (no keys)" advisory even when the md1 carries watch-only pubkeys (DOC/privacy)
- **WHAT:** `md repair` unconditionally emits `OutputClass::Template` ("stdout is a keyless descriptor template"), but operates on arbitrary md1 including wallet-policy md1 whose Pubkeys TLV carries pubkey entries (a natural repair input). `md address` correctly emits `WatchOnly`. Understates sensitivity of key-bearing output.
- **Citations:**
  - `repair.rs:156-159` — **ACCURATE.** `repair.rs:156-159`: `emit_output_class_advisory(OutputClass::Template, &mut stderr())` — unconditional.
  - Model to mirror: `md address` (`address.rs`) — note `address.rs` itself emits `WatchOnly` unconditionally BUT only after a line-23 guard `if !descriptor.is_wallet_policy() { return Err(...) }`, so its unconditional emit is sound (it only runs on wallet-policy). `repair` accepts BOTH classes → must branch.
- **STILL-REPRODUCES:** **YES.**
- **Fix-site:** `repair.rs:156-159` — branch on `descriptor.is_wallet_policy()` → `WatchOnly` when true, `Template` when false.
- **Action for brainstorm spec:** Cite `repair.rs:156-159` against `836faf8`. `is_wallet_policy()` confirmed available: `md-codec/src/encode.rs:50-52` `pub fn is_wallet_policy(&self) -> bool { matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty()) }`.

### L7 — `md repair --help` epilog falsely claims non-chunked single-string md1 are rejected (DOC drift)
- **WHAT:** The `repair` `after_long_help` epilog says non-chunked single md1 "are rejected with a wire-format error," but `decode_with_correction` gained v0.35.0 single-string auto-dispatch and now repairs them. Pure help-text drift.
- **Citations:**
  - `main.rs:241` — **ACCURATE** (the false sentence is in the `after_long_help` string spanning `main.rs:240-242`; the "...are rejected with a wire-format error..." phrase is at line 241).
  - Claim-now-false verification — **CONFIRMED.** `md-codec/src/chunk.rs:604-632` has the v0.35.0 single-string auto-dispatch (`if strings.len() == 1 { ... chunked_flag == 0 → decode_md1_string(...) }`). Non-chunked md1 ARE now repaired.
  - FOLLOWUP `md-codec-decode-with-correction-supports-non-chunked-md1` — **STATUS: RESOLVED in md-codec-v0.35.0** (per `FOLLOWUPS.md`). The epilog and FOLLOWUP-status reconcile are independent of each other; the epilog is just stale.
- **STILL-REPRODUCES:** **YES** (the false text is live).
- **Fix-site:** `main.rs:241` — delete/rewrite the "are rejected with a wire-format error" sentence. **Lockstep note:** if this epilog is mirrored in the manual (`docs/manual/src/40-cli-reference/`), update there in lockstep — CHECK at spec time whether the `repair` epilog text is mirrored (it is help text, so it likely is).
- **Action for brainstorm spec:** Cite `main.rs:241` against `836faf8`. Pure docs; no code, no behavior change.

### L19 — `md encode` emits "keyless template (no keys)" advisory even when `--key` embeds watch-only xpubs (DOC/privacy, sibling of L4)
- **WHAT:** Both `md encode` emit paths call `emit_output_class_advisory(OutputClass::Template, …)` unconditionally; with `--key` the card is wallet-policy (embeds xpubs = watch-only material). Same bug class as L4.
- **Citations:**
  - `encode.rs:73-76` (JSON path) — **ACCURATE.** Unconditional `OutputClass::Template`.
  - `encode.rs:110-113` (text path) — **ACCURATE.** Unconditional `OutputClass::Template`.
  - `output_advisory.rs:35` — **ACCURATE.** `OutputClass::Template => "note: stdout is a keyless descriptor template (no keys)"`.
- **STILL-REPRODUCES:** **YES** (both paths).
- **Fix-site:** `encode.rs:73-76` + `encode.rs:110-113` — branch on `descriptor.is_wallet_policy()` (same `is_wallet_policy()` helper as L4). Two emit sites in one file.
- **Action for brainstorm spec:** Cite `encode.rs:73-76,110-113` against `836faf8`. Identical fix-shape to L4 → fold together.

---

## Cross-cutting observations

1. **Citation drift is universal and large but mechanism-preserving.** Every M-finding's line number moved (M2 by ~120, M10 by ~132, M5 regexes from `:55`/`:498` vs report's `:32-91`/`:357-381`), because cycle-1 H13 + cycle-2 H7 grew `template.rs` from ~400 to **1972** lines. The L-findings (smaller files) are nearly on-target (`repair.rs:156-159`, `main.rs:241`, `encode.rs:73-76,110-113`, `output_advisory.rs:35` all ACCURATE). **Every brainstorm/SPEC must lift the LIVE line numbers above and cite SHA `836faf8`, not the report's stale snapshots.**

2. **H13/H7 lexer-collision check — M5 and M2 share the H13-touched file/function; M10/M11/L* do NOT.** The cycle-1 H13 fix (`081f61c` + C1 `ddddeff`) rewrote the **group-3 multipath body** handling inside `lex_placeholders` and `substitute_synthetic` — the exact functions M5 edits (regex anchoring) and adjacent to M2's `resolve_placeholders` n-computation. The current lexer regex (line 55) already carries H13's permissive `([^>]*)` body + strict in-loop validation (lines 77-110). **M5's fix (anchoring the multipath as final-only) MUST not regress H13's reject of hardened/malformed bodies** — both edits live in the same two regexes; coordinate carefully (this is the plan's noted "adjacent H13 (same lexer file)"). H7 ("prefix-form `[fp/path]@N`") was a **toolkit-side** cycle-2 fix (`parse_descriptor.rs`), not md-cli — no md-cli collision from H7. M10 (`ctx_for_template`/`keys.rs` depth gate), M11 (`keys.rs` point check), and L4/L7/L19 (advisory/help) are in code H13/H7 never touched → no collision.

3. **Registry / publish-chain reality.** md-cli is published standalone to crates.io; **the toolkit does not depend on md-cli** (only `md-codec = "0.37"` library). All 7 fixes are md-cli-only — **no md-codec change, no forced toolkit pin bump.** md-cli depends on md-codec via `{ path = "../md-codec", version = "=0.38.0" }`, so the existing 0.38.0 md-codec is fine. Publish chain is just: bump md-cli → tag `descriptor-mnemonic-md-cli-v0.9.0` → `cargo publish`. (Prior md-cli-only release precedent: `58cc9ec` "release: md-cli 0.8.0 (MINOR)".)

4. **SemVer aggregate = md-cli MINOR.** Per-finding: M2 PATCH, M5 PATCH/MINOR (REJECT of newly-rejected exotic input — PATCH if framed as bug-fix-reject), M11 PATCH (FORMAL-lane reject), L4/L7/L19 PATCH. **M10 is the MINOR driver** — it WIDENS accepted input (BIP-86 `tr(@0)` now accepted), which is additive behavior → the release as a whole is **MINOR → `md-cli v0.9.0`**. (Plan independently tags M10 "md-cli MINOR".)

5. **The fix-program plan already pre-slots all 7** into two named workstreams: **WS-MD-CLI-LEX** (M2/M5/M10/M11) and **WS-MD-CLI-ADVISORY** (L4/L7/L19) — see toolkit `design/PLAN_constellation_bughunt_fix_program.md:88,177-208`. cycle-9 = exactly these two workstreams. The plan's diff-oracle annotations match this recon: M10 = "BIP-86 depth-3 tr false-REJECT, NOT wrong-address → availability"; M11 = "new reject ⇒ FORMAL"; M5 = "adjacent H13"; L4/L19 = siblings.

6. **Lockstep flags.** GUI `schema_mirror` gates clap **flag-NAMES** — none of these 7 add/rename/remove a flag or dropdown value (they're parse-internal / advisory-text / help-text), so **schema_mirror is NOT triggered**. Manual mirror (`docs/manual/src/40-cli-reference/`) IS potentially triggered by **L7** if the `md repair` help epilog is mirrored there (CHECK at spec time) and arguably by **M10** if the manual documents which descriptor heads `md encode` accepts. The other findings are internal robustness with no doc-surface change. Sibling-codec FOLLOWUP companions: none required (md-codec untouched).

---

## Recommended brainstorm-session scope

**One cycle (cycle-9), md-cli MINOR → `descriptor-mnemonic-md-cli-v0.9.0` → tag + publish; no toolkit pin bump.** Two workstreams, both R0-gated:

- **WS-MD-CLI-LEX (FORMAL lane, the substance): M5 + M2 + M10 + M11.** ~80-150 LOC + tests. M5 is the funds crux (anchor multipath as final-only + reject leftover suffix + cross-validate lexer-view vs substituted descriptor) and must be coordinated with the H13 group-3 validation in the same two regexes (lines 55 / 498) — do M5 with H13's reject-tests in scope to prevent regression. M2 (bound max index before `as u8`; checked `by_i.get(&0)`). M10 (classify bare single-`@i` `tr(` as SingleSig depth-3 — scope to exclude script-path tr with a `{` tree; the MINOR driver). M11 (insert `PublicKey::from_slice(&bytes[45..78])` before the payload copy — FORMAL because it's a new reject). TDD: each gets a failing test first (M5 non-last-multipath divergence; M2 `wpkh(@255/*)` panic→clean-error; M10 `tr(@0/<0;1>/*)` + depth-3 BIP-86 xpub now accepted; M11 off-curve/all-zero pubkey now rejected).

- **WS-MD-CLI-ADVISORY (TRIVIAL lane, batch together): L4 + L19 + L7.** ~15-25 LOC. L4 + L19 share one fix (branch `OutputClass` on `is_wallet_policy()`) across 3 emit sites (`repair.rs:156-159`, `encode.rs:73-76`, `encode.rs:110-113`) — **fold into one commit.** L7 is a pure help-text delete (`main.rs:241`) — **batch in the same cycle**; reconcile the FOLLOWUP note that `decode_with_correction` non-chunked support shipped in v0.35.0. Check manual-mirror for the `repair` epilog (L7) and `md encode` accepted-heads (M10) at spec time.

**Ordering within the cycle:** WS-MD-CLI-LEX first (M11 → M2 → M10 → M5, simplest-to-hardest; do M5 last with H13 tests loaded), then WS-MD-CLI-ADVISORY (L4+L19 folded, L7 batched). Single release at the end. **Funds priority: M5 is the only fidelity/wrong-derivation finding** — it gets the most adversarial test coverage and the post-impl whole-diff review focus. M10/M11/M2 are robustness (availability / corrupt-accept / panic); L4/L7/L19 are doc/privacy.

**Source SHA to cite in the spec:** descriptor-mnemonic `origin/main` = `836faf8`.

---

## Reminder — mandatory R0 gate

This is recon only. Before ANY implementation: write the brainstorm spec + plan-doc, run the opus architect R0 loop to **0 Critical / 0 Important**, persist each review verbatim to `design/agent-reports/`, fold → re-dispatch until GREEN. No code, no implementer dispatch, no tag, no publish while any Critical/Important is open. (CLAUDE.md Conventions, first bullet.)
