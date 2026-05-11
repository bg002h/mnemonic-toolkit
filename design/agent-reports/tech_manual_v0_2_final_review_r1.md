# tech-manual v0.2.0 — final whole-cut reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.2.0` |
| Phase | 2.5 (final whole-cut, tag-time) |
| Commit under review | `07ddfc0` (Phase 2.4 close — pre-tag HEAD) |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `30-address-derivation/{31,32,33}*.md` + `60-back-matter/{61,62,63,64}*.md` delta |

## Findings: 0 Critical / 0 Important / 3 Low / 1 Nit

Per `feedback_zero_followups_from_release_cycles`: all findings fold inline at the closing commit. None deferred.

---

## Low

**L-1. BIP-379 cross-reference row attributes §III.2 but no body chapter actually cites BIP-379**

`64-bip-cross-reference.md:20` reads `| BIP-379 | Miniscript | §III.2 |`. `grep -rln "BIP-379" docs/technical-manual/src/` returns only the back-matter (glossary entry for "miniscript" cites BIP-379 parenthetically; bibliography lists it; the cross-ref table itself). No body chapter cites BIP-379 normatively. The §III.2 attribution is stale.

Fix: change the §III.2 cell to `Glossary (§"miniscript"), Bibliography` to accurately reflect where BIP-379 appears.

**L-2. Source-path prefix inconsistency within §III.1 and §III.2**

Every source reference in §III.1 (lines 5, 123-128) and §III.2 (lines 5, 47, 166-175) uses the full `descriptor-mnemonic/crates/md-codec/src/...` prefix. Two inline citations deviate:

- §III.1 line 82: `crates/md-codec/src/origin_path.rs:82-96` — missing `descriptor-mnemonic/`.
- §III.2 line 31: `crates/md-codec/src/canonicalize.rs` — missing `descriptor-mnemonic/`.

Fix: prepend `descriptor-mnemonic/` to both bare `crates/...` path citations.

**L-3. §III.3 intro says "four-network surface" but the chapter enumerates five variants**

`33-network-and-addressing.md:5` says "This chapter walks the four-network surface and the SLIP-0132 prefix interactions." The table at line 14-19 has five rows (Bitcoin, Testnet, Testnet4, Signet, Regtest), and the prose at line 18 correctly says "`bitcoin::Network` enumerates **five** variants relevant to md1". Phase 2.3's I-1 fold introduced Testnet4 but didn't propagate the count to the chapter intro.

Fix: change "four-network surface" to "five-network surface".

---

## Nit

**N-1. §III.2 Bucket 1 `sh(wpkh)` row prose is heavier than the repo's terse-code preference**

`32-shape-coverage.md:57` Test-cell content: "(covered by `Descriptor::new_sh_wpkh` at `to_miniscript.rs:220-223`; canonical-wrapper origin requirement gates external invocation)". The "canonical-wrapper origin requirement gates external invocation" clause is unexplained and reader-hostile.

Fix: rewrite to "converter shape covered at `to_miniscript.rs:220-223`; no standalone abandon-mnemonic test (CLI invocation requires annotated origin metadata)".

---

## Acceptance criteria (SPEC §7 v0.2)

| Criterion | Gate | Status |
|---|---|---|
| A1 (partial) | One walk-through per BIP-388-parseable form; seven buckets in §III.2 | PASS |
| A4 | Glossary ≥50 entries (57) | PASS |
| A5 | Index ≥150 rows (159); bidirectional lint gated | PASS |
| A6 | Part III in Pandoc TOC (gated by PDF build success) | PASS |
| A8 | verify-examples 8/8 | PASS |
| A10 | PDF ≥40pp (119pp) | PASS |

---

## Cross-cut quality (no findings)

- Voice consistency across §III.1 / §III.2 / §III.3 is good; bullet/table balance and H2/H3 section depth are uniform.
- All intra-document cross-references verified: §III.1→§II.1 (multiple), §III.2→§II.1 §"NUMS encoding for tr()", §III.2→§III.1 "framed the three-tier model", §III.3→§III.1, §III.3→end-user manual `mnemonic convert` chapter. All anchor strings now correct (Phase 2.1/2.2 closes corrected all known drift).
- BIP cross-reference rows for BIP-32, BIP-44, BIP-48, BIP-49, BIP-84, BIP-86, BIP-341, BIP-380, BIP-388, BIP-389 correctly list every section in Part III that actually cites them.
- NUMS H-point hex `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` confirmed against `to_miniscript.rs:34-35`.
- Open FOLLOWUP `cross-repo md1-wsh-multi-unsorted-integration-test` is correctly cited at §III.2:141 without being re-promoted to a new filing.
- All `\index{}` markers in Part III have a corresponding row in the index table (bidirectional lint gate).
- `derive.rs` pre-flight logic matches §III.1's four-bullet description exactly (hardened wildcard, chain-out-of-range, hardened alt, no-multipath chain≠0).
