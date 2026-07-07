# SPEC R0 review — bip388-double-star-shorthand-support — round 2

**Verdict: GREEN (0 Critical / 0 Important)**
**Reviewer:** opus architect, source basis `0964462d`.
**Dispatched:** 2026-07-06 (Cycle C, SPEC R0 loop round 2 — convergence on rev-2). Persisted verbatim per CLAUDE.md. N1 folded post-GREEN.

Both round-1 Important findings and all four Minors are fully and correctly folded, no new drift from the scope-AtN-IN change. All newly-added §4 anchors re-verified line-accurate.

## I1 — AtN coverage + mechanism reframe → RESOLVED
**§0 scope-in correct:** three in-scope surfaces match traced routing — (1) concrete via `concrete_keys_to_placeholders`→`parse_descriptor`; (2) AtN direct-lex `bundle.rs:1389` + `verify_bundle.rs:1375`; (3) `parse_literal_xpub`. ~10 `concrete_keys_to_placeholders` callers correctly acknowledged as chokepoint-covered.
**§5 mechanism sound + complete:** shared string pre-expander before each parser resolves the hole; SPEC states the exact reason a lexer-internal fix is insufficient (`lex_placeholders` returns occurrences not a string; `parse_descriptor` feeds `from_str(&substituted)`@897; `parse_literal_xpub` bypasses the lexer). Independently re-confirmed coverage closes:
- §5.1 (parse_descriptor top ~`:875`): all ~10 concrete callers feed `parse_descriptor` (descriptor.rs:68, bsms:227, bitcoin_core:682, bundle:2098, coldcard:321, coldcard_multisig:508, specter:234, electrum:377, sparrow:419, pipeline:417). Expander at `:875` (before lex@884 and from_str@897) covers all concrete paths + the from_str hole. `/**`+terminator survives `concrete_keys_to_placeholders` pass-through (`[fp]xpub/**)`→`@N[fp]/**)`, still `)`-bounded).
- §5.2 (bundle.rs:1389 + verify_bundle.rs:1375): the AtN direct-lex sites; the earlier-chokepoint option correctly deferred to the plan.
- §5.3 (parse_literal_xpub:297): xpub-search, no-op on JSON (idempotent).
No residual "2-call-site misses AtN" contradiction.

## I2 — misattribution + reject message → RESOLVED
§2 enumerates all four sites (re-verified present): `parse_descriptor.rs:189` (comment), `parse_descriptor.rs:206-211` (reject MESSAGE — drop `(or the /** shorthand)`, keep `/0/*`), `cli_import_wallet_descriptor.rs:159`(+`:191`), `sparrow.rs:42`. §8 marks the message behavior-lockstep; §7.9 tests it non-tautologically on a genuine `/0/*` reject. The LEAVE list correctly preserves the genuine BIP-389 multipath refs; `:141` spot-checked = "BIP-389 **multipath** `/<a;b>/*`" (left).

## M1-M4 → all folded
M1 §9 `scripts/install.sh:32` + frozen-sibling-pin note (verified no root install.sh). M2 §5 terminator set (`)`,`,`,`}`,ws,`#`,EOS), excludes `/***`/`/**'`, per-key/terminator-bounded, cites `substitute_nums_sentinel`. M3 manual `:164` → §8 semantic. M4 §7.3 concrete-xpub spelling + §7.4 AtN oracle.

## New-drift check → clean
§7 oracle spellings consistent with §0/§5 (7.3 concrete, 7.4 AtN, 7.5 xpub-search, 7.7 precision, 7.8 JSON regression, 7.9 message). Equivalence oracles non-tautological (vs pre-existing `/<0;1>/*`). New §4 citations (substitute_nums_sentinel 373/875/897, classify_descriptor_form 175-196, AtN 1389/1375) accurate.

## Optional nits (non-blocking)
- **N1 (fold — good cell):** a `/0/**` composite test. Post-expansion `/0/**` → `/0/<0;1>/*`, whose leading `/0` fixed step the Cycle-A floor STILL rejects — proves the expander does not weaken the floor for fixed-step+shorthand combos. Behavior already correct; the cell locks it.
- **N2 (immaterial):** `#` in the terminator set is a harmless superset (`/**#` never legitimately arises). No action.

GREEN clears the spec R0 gate; the plan-doc must run its own R0 loop and (per §5/§10.2) grep-verify the complete minimal call-site set as its first task.
