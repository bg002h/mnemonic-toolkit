# R0 Architect Re-Review (round 2) — SPEC_restore_multisig_format_payloads.md

**Reviewer:** opus `feature-dev:code-reviewer`. **Date:** 2026-06-05. **Branch:** `restore-multisig-format-payloads` (master `a38ec41`).
**Verdict:** **0 Critical / 0 Important — GREEN.** (1 non-blocking Minor.)

> Persisted verbatim per CLAUDE.md. GREEN ⇒ R0 gate satisfied; implementation may proceed. The Minor (one §6 rationale sentence) folded after persisting.

---

Reviewing the round-1 Important (I1) fold plus a check for newly-introduced issues, against source ground truth (no Bash available; verified via source-grep of the emit `format!` call sites + existing golden fixtures, which is decisive for token-presence claims).

### I1 fold — VERIFIED CORRECT

Each pinned threshold token is load-bearing and confirmed against the emit source:

- **`2of3` (electrum):** `electrum.rs:141` `let wallet_type = format!("{k}of{n}")` → k=2,n=3 → `2of3`, serialized as `"wallet_type":"2of3"` (`:157`). A K=1 single-sig-ify yields `wallet_type:"standard"` (`:115`) or `1of3` — token does not match. Correct + non-vacuous.
- **`Policy: 2 of` (coldcard, coldcard-multisig, jade):** `coldcard.rs:355` `format!("Policy: {threshold} of {cosigner_count}")` → `Policy: 2 of 3`. coldcard-multisig and coldcard both route multisig templates through `emit_coldcard_multisig_text` via the six-variant match (`coldcard.rs:44-53`); jade delegates to the same function (`jade.rs:46`). All three produce the token. Correct + non-vacuous.
- **`sortedmulti(2,` (descriptor, bitcoin-core, bip388, sparrow, bsms):** descriptor/bitcoin-core/bsms carry the canonical descriptor verbatim (`bsms.rs:94`); bip388 (`bip388.rs:85-86`) and sparrow (`sparrow.rs:196-205,230-231`) build `sortedmulti({k},@N/**,…)` with k inlined. K=1 → `sortedmulti(1,`; single-sig template → `wpkh(...)`. Token does not match either regression. Correct + non-vacuous.

**3-fingerprint claim — VERIFIED.** The SPEC restricts the 3-fp check to the `[fp/…]`-embedding formats (descriptor/bitcoin-core/bsms) and correctly excludes bip388/sparrow (which carry fps in a separate `keys_info`/JSON field, NOT in the `@N/**` descriptor). The fps `73c5da0a`/`b8688df1`/`28645006` are the C0/C1/C2 abandon-vector master fingerprints — corroborated by existing golden fixtures (`tests/export_wallet/coldcard_multisig_2of3_wsh.txt:6-7` carries `28645006` and `B8688DF1`; same seeds across `sparrow_multi_2of3_wsh_sortedmulti.json`, `core-multisig-2of3.json`, `bsms-4line-sortedmulti-2of3.txt`). The case-difference (coldcard emits uppercase `B8688DF1`; the 3-fp check targets descriptor/bitcoin-core/bsms which emit lowercase) is consistent. The empirical pin is sound and self-correcting (a wrong constant fails loudly at Phase 2 GREEN — no vacuity risk).

### §2/§3 reword (M1/M2) — INTERNALLY CONSISTENT
§2's M1 description ("six-variant `CliTemplate` match … NOT an `is_multisig()` call") matches §3's reword and the actual source (`coldcard.rs:44-53`). The §6 token table lists exactly the 9 EMIT formats from §2/§5; no "9 EMIT" vs table mismatch. M2's softened byte-parity rationale is consistent with the `--format descriptor` exact-equality strategy in §6.

### Residual soundness (single-sig-ify across all 9) — SUFFICIENT
Every EMIT format's threshold token inlines `{k}`, so a K=1 single-sig-ify breaks every cell. The cosigner-SET dimension (dropped/wrong cosigner) is covered transitively: §4 builds one shared `EmitInputs` (`slots`/`k`/`descriptor` computed once at reconstruction, passed by reference to every format), and the `--format descriptor` 3-fp assertion validates that shared substrate. No emit format escapes the single-sig-ify check.

### Findings
**Critical:** none. **Important:** none.

**Minor (non-blocking):** §6, imprecise rationale: "For the non-fp-embedding formats the threshold token + `--format descriptor` equality below carry the fidelity guarantee." Descriptor-equality does not *literally* inspect sparrow/bip388 output; the actual guarantee for those two is the threshold token (catches single-sig-ify) plus the shared-`EmitInputs` + descriptor-3-fp argument (catches cosigner-set drift transitively). Tightening that one sentence would match the stated rationale to the real guarantee. No change to the test plan.

### VERDICT: 0 Critical / 0 Important — **GREEN**
The I1 fold landed correctly (tokens verified load-bearing against emit source; fps corroborated by golden fixtures), the M1/M2 rewords are internally consistent, and the tightened §6 is sufficient to catch a silent single-sig-ify across all 9 EMIT formats. The R0 hard gate is satisfied; implementation may proceed.

---

## Fold note (applied after persisting)
- **Minor — FOLDED:** §6 rationale sentence tightened — for the non-fp-embedding formats (bip388/sparrow), the guarantee is the threshold token (single-sig-ify) + the shared-`EmitInputs`/descriptor-3-fp argument (cosigner-set drift), not a literal descriptor-equality inspection of their output.
- GREEN ⇒ no re-dispatch.
