# Convergence R0 — SPEC_followup_toolkit_v0860_demote.md (round 2, scoped to folds) — Fable, adversarial

**Persisted verbatim per CLAUDE.md before implementation begins.** Verified vs live master `14127582`. VERDICT: **GREEN (0C/0I)** — v0.86.0 implementation may begin. 3 residual Minors are citation touch-ups (all folded opportunistically into the SPEC before dispatch; no re-review needed per reviewer).

---

## Probe 1 — I1, oracle-boundary predicate (LOAD-BEARING): VERIFIED CORRECT + IMPLEMENTABLE

**The oracle claim is genuine.** `vendor/md-codec/src/chunk.rs:615-633`: for `strings.len()==1`, `chunked_flag = symbols.first() & 0x01` (:622); flag==0 → `decode_md1_string(&corrected_strings[0])` and **return** (:623-630) — the reassemble content-id check is bypassed. flag==1 falls through (:632) to `reassemble` (:637), which unconditionally re-derives `chunk_set_id` from the decoded descriptor and compares against the header csid (`chunk.rs:379-387`, `ChunkSetIdMismatch`) — so chunked-of-1 **retains** the ~2⁻²⁰ oracle. Bit-layout consistent: header doc `chunk.rs:3-4` (`[v3][v2][v1][v0][chunked]` MSB-first → chunked = bit 0).

**Bundle cards are protected chunked-of-1.** `chunk::split` clamps `count` to ≥1 (`chunk.rs:256-260`) and `ChunkHeader::write` hard-codes `chunked = 1` (`chunk.rs:53`); toolkit `bundle` emits md1 exclusively via `md_codec::chunk::split` (`synthesize.rs:392`, `:428`, `:499`). The SPEC's KEEP-Blessed for chunked-of-1 is therefore mandatory and correctly specced (SPEC line 8, behavior table line 10).

**Implementable at the bless site with zero new plumbing.** `repair.rs:1599` (`SetVerify::Blessed` inside `repair_via_md_codec`) has `chunks`, `corrected_chunks`, and `repairs` in scope (:1588-1595). The toolkit's own `parse_chunk` (`repair.rs:637`, symbol values at :667-668) yields the data-part 5-bit values → `values[0] & 0x01` reads the identical flag md-codec reads at chunk.rs:622. The advisory-widen site also needs no extra info: within the `:1732` non-Blessed fall-through, an `Md1` `Unverified` is by construction the non-chunked demote case (the only md1 Unverified source), so `:1741` can gate on `kind`+`set_verify` alone.

**Flip fixtures are genuinely non-chunked.** `VALID_SINGLE_MD1 = "md1yqpqqxqq8xtwhw4xwn4qh"` (prop file :58) and the CLI fixture both start data-part `'y'` = codex32 value 4 → bit 0 = 0. The flipped assertions are therefore correct post-demote.

**Boundary test:** specced (SPEC line 11(b) + acceptance #1) and constructable — `pub fn split` (chunk.rs:236) on any small descriptor emits a real chunked-of-1 at test time (also `md encode --force-chunked`); chunked-of-1 already flows through repair today ("shipped in v0.34.0", chunk.rs:613).

## Probe 2 — I2 (manual loci): FOLDED, two citation-drift nits

Verified live in `docs/manual/src/40-cli-reference/41-mnemonic.md`: `:3083` exit-5 row ("md1 (content-id check passes)") EXACT; `:3154` ("unlike mk1/md1, there is no cross-chunk hash or content-id") EXACT; `:3188-3191` ("reassembly hash mk1/md1 are ALREADY blessed on") EXACT; auto-fire `:750` (md1 exit-5 row), `:771`, `:856-860` (inspect/verify-bundle md1 exit-5 rows at :858/:860) EXACT. Demotion subsection + anchor specced, scoped to non-chunked. Drift: **exit-4 row is `:3084`** (SPEC says :3086), and the **"never exposed to this gap" sentence is `:3098-3099`** (SPEC range :3092-3097 stops one sentence short). Verbatim quotes in the SPEC disambiguate both → Minor.

## Probe 3 — I3 (md-cli divergence): FOLDED, one path-prefix error

Qualify-don't-flip + cross-repo FOLLOWUP `md-cli-non-chunked-single-string-repair-demote` mirrored in BOTH repos with `Companion:` lines — specced (SPEC line 13, acceptance #1). **The asymmetry is real:** `md repair` exit-5 is live md-cli behavior (`descriptor-mnemonic/crates/md-cli/src/main.rs:236` D26 exit-code doc; goldens in `crates/md-cli/tests/cli_repair.rs`; non-chunked repair supported since md-codec 0.35 per the after_long_help at main.rs:242), and the SPEC touches no md-cli code. **Path error:** SPEC line 13 cites `descriptor-mnemonic/.../42-md.md:334-353` — no such file exists in descriptor-mnemonic; the file lives in THIS repo at `docs/manual/src/40-cli-reference/42-md.md` (D26 "md1 is unaffected by later cycles… stays exit 5" prose at :339-353). This actually makes I3(a) easier (in-repo edit), but the citation points at the wrong repo → Minor.

## Probe 4 — I4 (advisory widen): FOLDED

SPEC line 9 specs the `(Ms1 OR non-chunked-Md1)` widen with `--md1` pointer text; live site verified Ms1-only (`repair.rs:1741` `matches!(kind, CardKind::Ms1)`); fall-through `:1732` kind-generic as claimed.

## Probe 5 — M1-M5: ALL FOLDED, loci verified live

M1: bless site `:1599` correct; doc blocks live-verified stale — `repair.rs:443-462` (":451-453 Md1 always reports Blessed") + `:470-475` (Unverified enum doc enumerates only mk1/ms1). M2: `cli_mk1_repair_reverify.rs:679-685` verified stale ("there is no md1 SingleString bypass… at all" — false since v0.35; the test body :686-694 is behavior-neutral, comment-only, no third flip) + `prop_repair_never_wrong.rs:231-233` (":233 md1 delegate is always Blessed"). M3: mk1 `GroupKey::SingleString` singleton arm verified at `repair.rs:1049-1062` (SingleString → `headers.len()==1` complete at :915 → `mk_codec::decode` Ok → Bless); encoder-unreachability corroborated by `cell_4_7` + the SPEC-§1 comment in the same test file; hand-construct-or-document specced. M4: own reason specced (SPEC line 8), distinct from the reassembly reason at :1069-1072. M5: `verdict_str` maps Unverified→`"candidate"` (`cmd/repair.rs:361-366`; SPEC's :363-365 within range); pin specced (line 11(a)).

## Probe 6 — new gaps from the non-chunked refinement: NONE

Predicate needs no info unavailable at either site (above); boundary fixture constructable (above); manual sweep — `grep -rn content-id docs/manual/src/` hits ONLY 41-mnemonic.md + 42-md.md, both already in the SPEC's locus list; 43-ms.md carries no md1-oracle claim; corpus-drift check and manual-gui FOLLOWUP already in the ritual/spec (lines 14, 27).

## Findings

**Critical:** none.
**Important:** none.
**Minor (3, all citation-precision; verbatim quotes/test names make each target unambiguous — non-blocking):**
- **M-a:** SPEC line 13 path prefix `descriptor-mnemonic/.../42-md.md` → should be `docs/manual/src/40-cli-reference/42-md.md` (this repo; prose at :339-353). One-line fix recommended before implementation.
- **M-b:** 41-mnemonic.md exit-4 row is `:3084` (not :3086); "never exposed" sentence is `:3098-3099` (extend :3092-3097).
- **M-c:** the `.code(5)` flip assertion is `cli_repair_md1_non_chunked.rs:65` (SPEC's :52 is the test-fn head, `non_chunked_md1_single_error_repair_exits_5_and_recovers` — rename accompanies the flip).

## VERDICT: GREEN (0C/0I)

All 4 Importants + 5 Minors from round 1 are folded and live-source-verified; the load-bearing non-chunked-vs-chunked-of-1 oracle boundary is correct, the predicate is implementable at both sites with existing machinery, and the boundary test is specced and constructable. The 3 residual Minors are citation touch-ups (recommend folding M-a's one-line path fix opportunistically — no re-review needed, no engineering impact). v0.86.0 implementation may begin.

---
**FOLD STATUS (opus, 2026-07-11):** GREEN — no C/I to fold. The 3 citation Minors (M-a path prefix, M-b exit-4 row :3084 + prose :3092-3099, M-c flip assertion at :65) were folded opportunistically into the SPEC before dispatching the implementer (reviewer: no re-review needed). Implementation authorized.
