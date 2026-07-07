# SPEC R0 review — bitcoin-core-receive-change-pair-merge — round 2

**Verdict:** NOT GREEN (0C / 1I)
**Reviewer:** opus architect, SHA d9063523
**Dispatched:** 2026-07-06 (Cycle B, SPEC R0 loop round 2 — convergence check on rev-2). Persisted verbatim before fold per CLAUDE.md.

## Critical (must fix before implementation)
- none

## Important (must fix before implementation)

- **[I-a] (residual of round-1 I3) Script-path `tr` scope is NOT actually locked — §0 ↔ §8.15 contradict, and §12's "no open design decisions remain" is false** — `SPEC §0` (scope table) vs `§8.15` vs `§12`. §0 declares script-path `tr` **IN scope** and asserts it "merge[s] under the same per-key-uniform guard," and §4.2 cond. 7 specifies the internal-key-plus-leaf-keys uniformity for it. But §8.15's `core_tr_scriptpath_pair_merges` carries an escape hatch: *"If constructing a valid split script-path `tr` Core fixture proves infeasible in-cycle, DOWNGRADE to: assert the guard REFUSES … and document script-path `tr` merge as floor-reject — **decided in P-planning**."* That makes a **behavior/scope** decision (does a uniform script-path `tr` pair MERGE, or FLOOR-REJECT?) contingent on **test-fixture feasibility** — backwards: behavior belongs locked in the SPEC, the test then verifies it. The result is a self-contradiction (§0 says merges; §8.15 permits floor-reject) plus a deferred decision that directly falsifies §12's "no open design decisions remain." A SPEC that contradicts itself and misrepresents its convergence state is not R0-GREEN, even though **both** outcomes are funds-safe (neither mis-merges).
  - **Fix (one-line lock, either direction — recommend floor-reject):** Core's standard taproot `listdescriptors` export is **key-path bip86 only** (`tr(xpub/0/*)`; the fixture `core-bip86-mainnet.json` confirms); Core does not emit `tr(NUMS, sortedmulti_a(...))` script-path wallets via the standard flow, so a split script-path `tr` is not a real Core shape. Recommend: **lock script-path `tr` merge OUT of scope → floor-reject** (funds-safe; user hand-combines if ever needed), keep **single-key bip86 `tr` merge IN and mandatory** (§8.15 first test). Then reconcile §0 (script-path `tr` → floor-reject, not merge), §4.2 cond. 7 (uniformity applies to single-key `tr` internal key; script-path `tr` → reject), §8.15 (make the second test a *refusal* test, not a contingent one), and §12 (now genuinely closed). Alternatively, if merge IS wanted, mandate the positive test unconditionally and confirm the fixture is constructible NOW — but do not leave it contingent.

## Minor (fold if cheap; not a gate)

- **[m1] §0 non-goal 4 label `#10/#10t`** — the taproot tests are **§8 item 15** (`core_tr_bip86_…`, `core_tr_scriptpath_…`), not `#10t` (no such item exists). Point the caveat at "#10 (multisig) / #15 (tr)".
- **[m2] §4.1 anchor "inserts after `:197`, before `:207`" is ~8 lines early** — the aggregate-dropped-fields NOTICE **emits** at `bitcoin_core.rs:198-205` (the build loop is 185-197; the `if !aggregate_dropped.is_empty()` writeln block is 198-205). The pre-pass must insert **after :205** (after the NOTICE fires), before :207. Immaterial because §4.5 states the intent unambiguously ("run the NOTICE loop on the original array … the pre-pass runs after it"), but tighten the line anchor to avoid an implementer inserting between the build loop and the NOTICE emit.

## Citation verification (rev-2 deltas)
- `parse_bool_field` @ `bitcoin_core.rs:361`: **ACCURATE** (`fn parse_bool_field` at 361).
- `parse_entry` reads `active`/`internal` via `parse_bool_field` @ `:326-327`: **ACCURATE** (active :326, internal :327).
- `apply_select_descriptor` ActiveReceive arm `:411` / ActiveChange arm `:427`: **ACCURATE** (match arms open at 411/427; filter closures at 416/432).
- extraction `:158-177`: **ACCURATE** (wallet_name 158-161, descriptors 163-171, empty-check 172-177).
- FIVE `recompute_descriptor_checksum` copies — `descriptor.rs:246`, `electrum.rs:1027`, `coldcard.rs:515`, `specter.rs:444`, `sparrow.rs:668`: **ACCURATE** (grep-confirmed exactly 5; bitcoin_core = 6th).
- fixed-step floor reject `parse_descriptor.rs:203-213` (residue `:205`, message `:207-210`): **ACCURATE**.
- taproot fixture `core-bip86-mainnet.json` is `tr([b8688df1/86'/0'/0']xpub…/<0;1>/*)`: **ACCURATE** (read the file; single already-multipath `tr` entry).
- `sample_core_metadata` @ `mod.rs:534`: **ACCURATE**.
- `--json` `:1859` / text-summary `:2265` / `CoreSourceMetadata.internal` `mod.rs:355` / `core-mainnet-receive-change-pair.json` / flip tests `:926`,`:1108` / keep-green `:952`: **ACCURATE** (unchanged from round 1).
- Manual wrong sentence `41-mnemonic.md:1404-1405`: **ACCURATE** (read the file; the "Bitcoin Core 25+ emits … `<0;1>/*` multipath shape" claim + the following one-bundle-per-entry sentence both land in the ~1401-1410 worked-example block).

## Fold-by-fold convergence check
- **I1 (internal=None mechanism) → §5: RESOLVED.** Explicit `parse_entry(internal: Option<bool>)` param; passthrough → `Some(parse_bool_field)`, merged → `None`; shape-based detection explicitly FORBIDDEN; invariant "pre-pass-merged ⇒ None, NOT multipath ⇒ None." §8.13 strengthened to assert `active-receive`/`active-change` on the `:952` fixture each return exactly the one flagged entry — the correct non-tautological guard that a shape-based implementation would FAIL. No drift (the `active`-from-JSON / `internal`-from-param split is internally consistent because §4.3 writes `active = A||B` into the merged JSON entry).
- **I2 (tautological oracle + misfired-multisig false-pass) → §4.3 + §8.4/§8.10: RESOLVED.** §4.3 mandates all-keys structurally-anchored per-key replacement (explicitly forbids `str::replace`/first-only `replacen`). §8.4 rewritten to derive addresses **independently from the ORIGINAL split descriptors** for BOTH chains and assert equality to the merged bundle's chain-0/chain-1 — anchored on pre-merge truth, catches a partial-rewrite misfire that verify-bundle alone false-passes. §8.10 uses the same oracle. Genuinely non-tautological.
- **I3 (taproot unscoped) → §0/§4.2 cond.7/§8.15: PARTIALLY RESOLVED.** Single-key bip86 `tr` is correctly in-scope, cond.7 correctly extended to the `tr` internal key + all leaf keys, and §8.15's bip86 test is mandatory with the §8.4 oracle — that half is clean. **The script-path `tr` half is the open item [I-a] above.**
- **I4 (self-contradictory grouping key) → §4.1: RESOLVED.** Grouping key now explicitly EXCLUDES the final step, is positional/ordered (not set-based), = (template kind, threshold, ordered per-key (fp, origin, xpub), wildcard-hardness); differing steps + disagreeing internal are the within-group discriminant. Unambiguous; a swapped-order `multi` correctly fails to group.
- **I5 (open decisions) → §4.3 range / §5 / §6 / §7 / §12: RESOLVED for range/mechanism/checksum/message.** Range = union/widening, NEVER blocks merge, both-absent→absent: locked. §6 = 6th local copy: locked. §7 message: locked default with example + direct pre-pass emit. **The §12 "no open decisions remain" claim is undermined only by the I-a script-path `tr` deferral** — closing I-a makes §12 true.
- **M1 (checksum count) → §3/§6: RESOLVED** (5 existing enumerated + 6th).
- **M2 (manual line + whole block) → §9.2: RESOLVED** (`:1404-1405` + following sentence + whole `~:1401-1410` block + re-grep-at-impl).
- **M3 (non-goal 4 caveat) → §0: RESOLVED** (the `#10t` label nit is m1 above).
- **M4 (hardened/wildcard-hardness) → §4.2 cond.3 + §8.16: RESOLVED.**
- **M5 (parse_bool_field semantics) → §4.2 cond.5: RESOLVED.**
- **M6 (dropped-fields NOTICE on original array) → §4.5: RESOLVED** (intent unambiguous; m2 is only a line-anchor tightening).
- **M7 (token `both` + message) → §5/§7: RESOLVED.**
- **M8 (remove-both-insert-one, N-pair) → §4.4: RESOLVED.**
- **M9 (validate input checksums) → §4.4 + §8.17: RESOLVED.**

## Notes
- **Guard-matrix safety re-confirmed under rev-2.** The core property is unchanged and sound: any pair passing conditions 1-7 is byte-identical except an unhardened fixed final step (recv vs chg) with disagreeing `internal`, over positionally-identical `(fp, origin, xpub)` keys and identical template — provably the same wallet's two chains, whose `<recv;chg>` expands element-wise to exactly the two originals. No wrong-merge (different-wallet) can pass. The rev-2 additions (structural all-keys replacement §4.3, original-anchored oracle §8.4/§8.10, input-checksum validation §4.4, hardened/wildcard-hardness exclusion §4.2 cond.3) tighten the edges I flagged in round 1.
- **Plan-phase feasibility to confirm (NOT a SPEC gate):** the §8.4/§8.10/§8.15 oracle needs programmatic access to the **merged descriptor string** (to expand chain-0/chain-1 for comparison against the originals). Confirm at plan time that import-wallet surfaces it (e.g. `original_descriptor` in `--json`, or a verify-bundle/inspect round-trip) so the oracle is built in its non-tautological form and cannot silently regress to comparing against a re-authored `<0;1>`.
- **Round-3 disposition:** this is a **single-item convergence** — fold I-a (lock the script-path `tr` behavior in the SPEC, recommend floor-reject; reconcile §0/§4.2/§8.15/§12) plus the two trivial Minors (m1 label, m2 line-anchor), then the SPEC is GREEN. No architectural change; no re-litigation of the merge design. Given the sole remaining Important is a one-line scope lock with both outcomes funds-safe, I expect round 3 to close at 0C/0I.
