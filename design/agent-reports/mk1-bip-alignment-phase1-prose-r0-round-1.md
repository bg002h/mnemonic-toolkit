# R0 per-phase review — mk1 BIP-alignment cycle, Phase 1 (BIP prose) — Fable, adversarial, read-only

**Persisted verbatim per CLAUDE.md.** Target: `/scratch/code/shibboleth/mnemonic-key/bip/bip-mnemonic-key.mediawiki` (727 lines) vs SPEC Part 2.
**Verification:** clean-room Python implementation written strictly from the BIP text (no reference-code consultation), run against the shipped `crates/mk-codec/src/test_vectors/v0.1.json`; BIP 93 text fetched from bitcoin/bips master.

## Axis 1 — §Checksum recovery-independence: PASS (independently reproduced)
Implemented from BIP lines 105–210 alone. NUMS derivation reproduces `MK_REGULAR_CONST 0x1062435f91072fa5c` / `MK_LONG_CONST 0x41890d7e441cbe97273`; `"shibbolethnums"` cross-check reproduces MD's `T_REGULAR`/`T_LONG`. V1 chunk 0 (long, 93 pre-checksum→108) checksum `x98j76m4mjlwphf`; chunk 1 (regular, 64→77) checksum `pug2mmjtfel6x`; both full strings byte-identical to shipped V1; verify == target both. Falsification of the warned-against readings (init 1+prepend, init 0x23181ab): both reject V1. Thresholds as written match `bch.rs:111-124` at every probed edge. The algorithm AS WRITTEN reproduces V1 — genuinely recovery-independent.

## Axis 2/3/4/5/6 — all C-items present; consistent; downgrades honest; cross-BIP reconciled
C-C1..C-C3, C-I1..C-I6, C-M1..C-M6 all verified landed with source cites (depth-0 `0..=10`, `Normal{0}`, "2..=52 B"; XpubOriginPathMismatch invariant; 8→4 in all 3 spots; form-aware stub; wire xpub-version set + SLIP-0132 note; string-layer error list + 94-95 gap; slot-XOR note). Greps for `8 for the long`, `1..=10`, `3..=52`, superseded stub formula, `2 long-code`, `to be written`: no survivors. Downgrades: substitution t=4 stays MUST; erasure/guided/confidence informative w/ FOLLOWUP slugs. Cross-BIP contrast reconciled both sides (both byte-aligned; mk1 fixed 53-byte; md1 variable near-equal whole-byte, deterministic). §Test Vectors carries no stale pins.

## IMPORTANT (1)
**I-1. §Checksum line 130 asserts a false BIP 93 fact — the exact myth F-A6 purges.** Line 130: *"This is not `1` — the value BIP 93's ms32_polymod starts from…"*. BIP 93's published `ms32_polymod` initializes `residue = 0x23181b3` and prepends no HRP expansion; the "1" reading is false (ms-codec `bch.rs:52` seeds `0x1` + `hrp_expand("ms")`). Imports the OLD `mk-codec bch.rs:185-198` framing that F-A6 flips. Creates a line-120↔130 internal contradiction now + a BIP↔`bch.rs` contradiction after Phase 2 F-A6. Not Critical (rule stated redundantly; clean-room build never needed line 130). **Fix:** split the two facts — `1` is the generic BIP 173 start / the seed `ms1` uses (with `hrp_expand("ms")` prepend); `0x23181b3` IS BIP 93's `ms32_polymod` init verbatim.

## MINOR (2)
- **M-a.** Line 120 "init- and algorithm-identical to MD's §Checksum… only the HRP and target constants differ" — code-variant availability also differs (MD has no long code). Add a parenthetical.
- **M-b.** FAQ line 655 "up to 4 substitutions per card" — the guarantee is per string. Prefer "per string".

**VERDICT: OPEN (0C / 1I)** — fold I-1 (single-sentence rewrite) and re-dispatch; the funds-critical recovery-independence gate is verified sound.

---
**FOLD STATUS (opus, 2026-07-10):** I-1 folded (line 130 rewritten: `1` = generic BIP-173 start + ms1's seed w/ hrp_expand("ms") prepend; `0x23181b3` IS BIP-93's ms32_polymod init). M-a folded (line 120 code-variant parenthetical added). M-b folded (line 655 per card→per string). + md1-cross-ripple: line 112 T_LONG reworded (MD retired its long code). Re-dispatched for convergence.
