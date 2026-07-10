# R0 review — `SPEC_mk1_bip_alignment.md` — round 1 (Fable, adversarial, read-only)

**Reviewer:** Fable. Persisted verbatim per CLAUDE.md. Target vs `mnemonic-key @ origin/main 1c9fbf7` (mk-codec 0.4.1 / mk-cli 0.12.0), the prior Fable BIP review, and BUGLIST bucket C + DG ledger.
**Dispatched:** 2026-07-10 (MK SPEC, R0 round 1).

## Independent verification performed (not taken on faith)

- **Clean-room checksum recompute (funds-critical):** re-implemented in Python ONLY what SPEC C-C1 prescribes — `POLYMOD_INIT 0x23181b3`, `hrp_expand("mk") = [3,3,0,13,11]` prepend, append 13/15 zeros, XOR `MK_REGULAR_CONST 0x1062435f91072fa5c` / `MK_LONG_CONST 0x41890d7e441cbe97273`, verify `polymod(hrp_expand ‖ data_with_checksum) == target` — over the shipped corpus. **All 18 positive vectors reproduce byte-identically in BOTH directions**, both code variants. V1: chunk 0 = long (108), chunk 1 = regular (77) — empirically confirms C-I4's line-74 correction (mixed-code is the normal shape). The documented algorithm is EXACTLY the code (`bch.rs:198` init; create `:304-320`; verify `:322-338`; `hrp_expand` pinned by test `:975-982`).
- **Depth-0 (C-C2):** `path.rs:113-119` accepts `count == 0` (rejects only `> 10`); tests `:251`, `:262`; `xpub_compact.rs:86-108` reconstructs `depth=0, Normal{0}`; `encode.rs:31-48` invariant includes empty-path. BIP-only change correct; "2..=52 B" arithmetic checked. Confirmed no depth-0 vector exists (V1–V18 non-empty).
- **Every SPEC/BIP line cite spot-checked live** (8-myth `:29/:480/:504` vs `:144`; erasure `:145-146/:150`; `1..=10` `:332/:336/:378`; stub `:400` vs `:37/:268/:404`; "to be written" `:512-516`; slot-0 chunk_set_id `:182`; `error.rs:57`, `:115-117`; `consts.rs:50`; `key_card.rs:26-33`). All accurate. **0x16 status:** assigned in code since v0.2.0 (`path.rs:53`) AND in the BIP table (`:317`); only `error.rs:115-117` is stale — F-A6's verify-then-edit is correct (source: `FOLLOWUPS.md:263-269`, resolved).
- **Chunk framing (C-I4):** `chunk.rs:50-83` — stream = `bytecode ‖ SHA-256(bytecode)[0..4]`, fixed `frag_size = CHUNKED_FRAGMENT_LONG_BYTES = 53`, successive slices, last = remainder. Matches SPEC; md1-contrast present and correct.
- **SHA-anchoring (F-A7 stress):** exactly ONE pinned corpus SHA — `tests/vectors.rs:41 V0_1_SHA256` (test `:108-116`) + a `family_token == GENERATOR_FAMILY` equality assert (`:130-132`). No toolkit-side pin. Corpus `include_str!`-baked as pub `V0_1_JSON` (`test_vectors/mod.rs:16`), surfaced via mk-cli `vectors`; CI `vectors-roundtrip` enforces regenerated == committed.
- **Completeness:** C1–C3, I1–I6, M1–M6 all mapped; DG-1/2/3 ledgered w/ companion-FOLLOWUP; phasing matches BIPs-first; Phase-1-embed → Phase-3-resync handled.

## CRITICAL — none

An implementer following the prescribed §Checksum + §Chunking text reproduces every shipped card byte-identically (empirically demonstrated).

## IMPORTANT — 3

### I-1. F-A7's ruling rests on a false cost premise, contradicts the Q-10 precedent, and leaves two roll-on-minor comment sites mandatorily unfixed (SPEC :26-28, :70)
- **False premise:** "changing it would churn *every pinned vector SHA*… for zero benefit." Ground truth: exactly one pinned SHA (`tests/vectors.rs:41`), and **Phase 3's F-V-mk regenerates `v0.1.json`, forcing that SHA to be re-pinned this cycle regardless.** Rolling the token inside the same regeneration has zero marginal churn.
- **"Matching reality: 0.2→0.4 without a roll"** launders two missed rolls into intent. The only precedent for wire-additive + new vector — v0.2.0's 0x16 + V18 — **rolled the token, citing Q-10** (`FOLLOWUPS.md:269`). All three doc sites (`consts.rs:47-49`, `tests/vectors.rs:46-52`, BIP `:514`) state roll-on-minor; only practice diverged.
- **Coherence hazard:** the new depth-0 vector decodes only under v0.4.0+ (older decoders: `PathTooDeep(0)`). Embedding it in a corpus stamped `family_token: "mk-codec 0.2"` mislabels the corpus under the token's decode-family semantics — at the exact moment C-C3 normatively pins that corpus in the BIP.
- **Ruling (as SPEC requested):** roll `GENERATOR_FAMILY` → `"mk-codec 0.4"` **inside the Phase-3 regeneration** (framed as completing v0.4.0's missed roll), keep Q-10's convention in the BIP with an honesty note. If author still prefers stable-anchor, that is *permissible* but the SPEC must then (a) delete the false churn rationale, (b) add an explicit written waiver that a family-0.2-labeled corpus carries a v0.4-only vector, (c) make the rewrites of `consts.rs:47-49` + `tests/vectors.rs:46-52` + BIP §Test Vectors **mandatory and byte-consistent** — "Optionally add a code comment" (SPEC :28) recreates exactly the A6 self-contradiction class this cycle exists to purge. Either resolution requires a SPEC edit.

### I-2. "Comments-only / no SHA churn" scope claims contradict Phase 3's own mandatory mechanics (SPEC :10, :56, :61)
Goal (":10 the only code touches are comment/convention corrections"), Ripple (":56 comments + BIP + one added vector only"), and acceptance-1 (":61 …no SHA churn") collide with F-V-mk's requirements: (a) a new fixture in `src/bin/gen_mk_vectors.rs`; (b) regenerating `v0.1.json` — published-library content; (c) re-pinning `V0_1_SHA256` (test hard-fails otherwise); (d) CI `vectors-roundtrip`. Concrete failure: a per-phase reviewer holding acceptance-1 as a cycle-wide invariant hits a contradiction at the first Phase-3 test run, or rejects the gen_mk_vectors.rs edit as out-of-scope. **Fix:** scope acceptance-1 to "V1–V18 strings/bytecode byte-identical" and enumerate Phase 3's sanctioned churn set (gen fixture + corpus regen + `V0_1_SHA256` re-pin + family-token per I-1).

### I-3. Ripple's "toolkit pin refresh per ritual" re-arms the documented sibling-pin footgun (SPEC :56)
Toolkit `scripts/install.sh:41` pins sibling `mk-cli-v0.12.0` as a **FROZEN baseline** — the v0.75.0 incident was precisely a sibling-pin bump breaking `sibling-pin-check` post-tag. Toolkit `Cargo.toml:33` has `mk-codec = "0.4.1"` (caret — a 0.4.2 patch is compatible with zero toolkit edits). **Fix (one sentence):** "toolkit: NO install.sh sibling-pin change (frozen baseline; sibling-pin-check enforces); no toolkit release; Cargo.lock refresh rides the next toolkit cycle."

## MINOR — 4

- **M-1 (C-I6 wording, SPEC :44):** the decoder/wire rule (`{0x0488B21E, 0x043587CF}`, others → `InvalidXpubVersion`) is right — but the BIP should scope it to the wire layer and note that reference mk-cli *normalizes* SLIP-0132 prefixes (ypub/zpub/upub/vpub…) on input (`crates/mk-cli/src/slip132.rs`, FOLLOWUP `mk-slip0132-prefix-acceptance`); the wire never carries them. A blanket "encoders reject all others" would contradict observable CLI behavior.
- **M-2 (C-C1 content list, SPEC :36):** add the checksum-symbol **extraction order** (13/15 five-bit symbols, big-endian: first symbol = top 5 bits of the XORed residue — `bch.rs:310-312`) + a cross-ref to the code-selection threshold (≤93 data → regular; 94–108 → long). The clean-room recompute needed both.
- **M-3 (F-A6 guidance, SPEC :22):** the corrected `bch.rs:185-186` comment should flip "deliberately NOT [BIP-93's init]… starts from 1" to "IS BIP-93's published `ms32_polymod` init verbatim" while KEEPING the load-bearing equivalence note (0x23181b3 = fold of `hrp_expand("ms")` from 1; ms1/rust-codex32 uses the equivalent init-1 + prepend), consistent with the correct test comment at `:857-863`.
- **M-4 (FOLLOWUP hygiene):** C-C2's BIP fix closes the BIP-lockstep gap the resolved `mk1-no-path-depth0-support` (`FOLLOWUPS.md:365`) left — annotate in the shipping commit; the `error.rs:116` re-word should cite `md-path-dictionary-0x16-gap`'s resolved status (`FOLLOWUPS.md:263-269`) as verification source.

## Requested rulings

- **NO-BUMP vs patch: PATCH.** mk-codec 0.4.1 → 0.4.2 + mk-cli 0.12.0 → 0.12.1 lockstep. NO-BUMP untenable: `v0.1.json` is baked into the published library and the BIP §Test Vectors pins the *post-regen* corpus, which must exist in a published release; `mk vectors` output changes. Precedent: v0.1.1 added V9–V17 as a patch. No wire change (depth-0 shipped in 0.4.0). Toolkit: no action (per I-3).
- **F-A7 token:** roll to `"mk-codec 0.4"` in the Phase-3 regen (see I-1); stable-anchor acceptable only with the three mandatory consistency edits + written waiver.

**VERDICT: NOT GREEN — 0 Critical / 3 Important open** (I-1 F-A7 ruling+rationale+mandatory comment sites; I-2 Phase-3 scope/acceptance contradiction; I-3 sibling-pin ripple wording). Fold and re-dispatch.
