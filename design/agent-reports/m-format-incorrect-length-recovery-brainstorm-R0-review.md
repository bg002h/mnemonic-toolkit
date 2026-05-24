# R0 Architect Review — BRAINSTORM_m_format_incorrect_length_recovery.md

**Round:** R0 (brainstorm-spec gate)
**Reviewer:** feature-dev:code-reviewer (opus)
**Date:** 2026-05-24
**Reviewed against:** `origin/master` = `925f5ed`, plus sibling codec checkouts (ms-codec 0.2.0, mk-codec 0.3.1, md-codec local 0.35.0).
**Persisted verbatim before fold, per CLAUDE.md.**

---

(Lead-in verification note from the agent, preserved:)

This is important for the trigger predicate. For an ms1 with a dropped char (length-49), `ms_codec::decode_with_correction`:
1. `parse_ms1_symbols` parses 46 data symbols (no length check there — line 112-142 only checks HRP and alphabet).
2. Computes residue over 46 symbols. Residue != 0 (it's a corrupted-length string).
3. Calls `decode_regular_errors(residue, 46)` — runs BCH on a 46-symbol "codeword". This likely returns None (>4 apparent errors) → `Err(TooManyErrors)`. OR it could spuriously return Some with corrections, then the defensive re-verify + final `decode(&corrected_str)` length-gate would catch it (length 49 not in VALID_STR_LENGTHS → `UnexpectedStringLength` → mapped to `PostCorrectionDecodeFailed` via the `Err(other)` arm at line 810).

So the trigger "normal repair yields no valid decode" maps to: `repair_card` returns `Err(RepairError::TooManyErrors | PostCorrectionDecodeFailed | UnparseableInput | ...)`. The indel feature must catch these specific failure variants and engage the indel search. The spec's trigger predicate (§5 line 96, §12 item 2) is correct in spirit, but the implementer must enumerate WHICH RepairError variants trigger indel search vs which pass through. This is a plan-doc detail the spec correctly defers to §12.2. Acceptable.

§4 false-positive floor: regular code has a 13-symbol checksum over GF(32), so a random data-part has probability `32⁻¹³ = 2⁻⁶⁵` of matching the target residue. ~4×10⁶ candidates at j=4 ⇒ expected false positives ≈ `10⁻¹³` — negligible. Sound. mk1 long code is 15 symbols (safer). bech32 ALPHABET `qpzry9x8gf2tvdw0s3jn54khce6mua7l` does NOT contain `1` — separator claim confirmed.

---

## Verdict: RED — 0 Critical / 2 Important

The core cryptographic premises (t=4 capacity, placeholder-then-decode reduction to a single substitution, delete-and-validate residue==0, no-erasure-primitive, false-positive floor) are all **verified sound against source**. No Critical defects. Two Important gaps must be resolved in the plan-doc before coding: the mk1 multi-string/per-chunk reality is unaddressed in the algorithm body, and the exit-5 framing is imprecise in a way that could mislead the implementer.

---

## Critical
None. Each load-bearing crypto/source claim was independently verified (details under "Verified-correct claims" below).

---

## Important

### I1 — mk1 multi-string (chunked) reality is not modeled in the algorithm body; §3's "single string" framing is the atypical mk1 case

**Claim (spec §3, lines 44-47):** "A full m-format string is `<hrp>1<data-part>` — exactly `ms1…` or `mk1…`" — treats the input as one string with one `<hrp>1<data>` structure, and §4's feasibility table uses a single "N ≈ 100" data-part length.

**Source evidence this is wrong for the common mk1 case:**
- `mnemonic-key/.../string_layer/pipeline.rs:7-22` — mk1 emits a **single string only when bytecode ≤ 56 bytes**; otherwise it is **chunked** into multiple `mk1` strings. The doc comment states the typical card: "≈84 bytes bytecode → 88-byte stream → fragments of 53 + 35 bytes" — i.e. **two mk1 strings** for a normal xpub card.
- `mnemonic-key/.../consts.rs:33` `SINGLE_STRING_LONG_BYTES = 56`; `:42` `MAX_CHUNKS = 32`.
- The toolkit already models this: `repair.rs:690-708` `repair_card` for `CardKind::Mk1` **iterates per-chunk** (`--mk1` is `repeating: true`, `cmd/repair.rs:46-47`) and is **atomic per D8** — if any one chunk fails, the whole call fails (`repair_chunk_one(...)?` at `repair.rs:700`).

**Why it matters:** A transcription indel lands in exactly one chunk, changing that chunk's length while sibling chunks stay intact. Under current atomic semantics the entire mk1 repair fails. The indel feature must therefore run **per-chunk**: detect which chunk failed normal repair, run P1/P2 on that chunk, validate the repaired chunk via the single-string oracle `decode_string` (`bch.rs:650`), then let the existing `decode(&[&str])` reassembly (`key_card.rs:114`) confirm cross-chunk-hash. None of this is in §3; the spec defers only the *validator-choice* question to §12 item 3, not the per-chunk *control-flow*.

**Why this is Important not Critical:** the per-chunk approach is sound and the validator choice (`decode_string`, BCH-only single string) is correct — a placeholder-then-decode recovery that passes per-chunk BCH reproduces the byte-exact original chunk, so the subsequent full `decode(&[&str])` reassembly + cross-chunk-hash succeeds. The design works; it is simply unspecified.

**Fix:** Add to §3 (or §12 → promote to a locked decision) an explicit mk1 per-chunk model: (a) indel search operates on the single failing chunk, not the joined input; (b) the per-chunk validator is `mk_codec::string_layer::bch::decode_string`; (c) the budget `N` is **per-chunk**; (d) state the post-recovery full-`decode(&[&str])` confirmation step; (e) note interaction with the existing D8 atomic-fail semantics. Update §4's runtime note to say "per-chunk data-part length" (a single mk1 chunk caps at 108 symbols, consistent with the table's N≈100, so the numbers stand).

### I2 — "exit 5 already taken by auto-fire" mischaracterizes 5; it is also the standalone-repair "correction applied" code — the implementer needs the precise distinction

**Claim (spec §6, line 103):** "`RepairShortCircuit{exit_code} => 5` (already taken by auto-fire — do NOT overload)".

**Source evidence:**
- `error.rs:508` — `ToolkitError::RepairShortCircuit { exit_code } => *exit_code` — the variant carries an **arbitrary** exit code, not a hardcoded 5. It is hardcoded to 5 only at the auto-fire call site `repair.rs:982` (`Err(ToolkitError::RepairShortCircuit { exit_code: 5 })`).
- More importantly, `cmd/repair.rs:122` — the **standalone `repair` subcommand** returns `5` directly (`Ok(if total_repairs == 0 { 0 } else { 5 })`), documented at `cmd/repair.rs:12-14`: "0 — all chunks already valid; 5 — at least one correction applied (REPAIR_APPLIED)". So **5 is the normal "a correction was applied" code for the very subcommand `--max-indel` is being added to** — not exclusively an auto-fire signal.

**Why it matters:** §6's outcome table maps unique→0, ambiguous→4, unrecoverable→2 — and does NOT use 5 for the unique-indel-recovery case. But an indel recovery IS "a correction applied." If the implementer follows the existing `cmd/repair.rs:122` contract literally, a successful unique indel recovery would return **5**, contradicting the spec's table (which says **0**). The spec must decide and state explicitly: does a successful indel recovery return 0 (per its table) or 5 (per the existing "correction applied" convention)? The spec's framing ("5 is auto-fire, don't overload") obscures this real collision with the standalone subcommand's own semantics.

**Fix:** In §6/§7, resolve the unique-recovery exit code against the existing `cmd/repair.rs:122` "repairs applied ⇒ 5" contract: either (a) indel recovery is a kind of repair and returns 5 (align the table), or (b) indel recovery is deliberately distinguished and returns 0 (then the plan must override the `total_repairs`-based exit in `cmd/repair.rs::run` for the indel path and justify the divergence). Correct the "already taken by auto-fire" phrasing to "5 = standalone-repair correction-applied AND auto-fire short-circuit."

---

## Minor

- **M1 (§7, line 117) — RepairError variant list is incomplete and not in source order.** The spec lists `EmptyInput, HrpMismatch, ReservedInvalidLength, TooManyErrors, UnparseableInput, UnsupportedCodeVariant, …`. Source order (`repair.rs:388-430`) is `EmptyInput, HrpMismatch, TooManyErrors, UnparseableInput, ReservedInvalidLength, UnsupportedCodeVariant, PostCorrectionDecodeFailed`. The spec presents them alphabetically (correctly anticipating the CLAUDE.md alphabetical-ordering convention for *new* variants) and the trailing "…" covers omissions, but it silently drops `PostCorrectionDecodeFailed`. Note in the plan-doc that the existing enum is **not** alphabetically sorted today (retroactive sort is tracked as `error-rs-retroactive-alphabetical-sort`), so a new variant inserted alphabetically will sit in a not-yet-sorted enum — fine, but call it out so the implementer does not "fix" the ordering as a surprise.

- **M2 (§3, §5) — ms1 valid-length sparsity is worth a sentence.** ms1 `VALID_STR_LENGTHS = [50, 56, 62, 69, 75]` (`ms-codec/.../consts.rs:33`) are **sparse** (gaps of 6-7), unlike mk1's dense `[14,93]∪[96,108]`. So the §3/§5 "a dropped char often lands on a still-code-valid length" rationale is really an **mk1** phenomenon; for ms1 a single drop usually lands off the rule-9 length set. The "trigger on no-valid-decode, not length-invalid" design is still correct for both (the toolkit-layer `bch_code_for_length` band accepts the off-by-one length for ms1 too, so it does not error early — verified at `repair.rs:559` + `repair_via_ms_codec` mapping `repair.rs:781-814`), but the prose generalizes an mk1-specific motivation to ms1. One clarifying clause would prevent an implementer mis-reasoning about ms1 candidate lengths. Note also that correct candidates auto-satisfy ms1 rule-9 (they restore the original valid length), so the sparsity is benign.

- **M3 (§9 FOLLOWUP (a)) — reaffirm scope split is correct.** Verified: no erasure/erase primitive exists in any of the three codecs (grepped the terms themselves in ms/mk/md `src/` — zero matches). The j=8 erasure reach is correctly a sibling-codec FOLLOWUP, not available now. The four deferred items (erasure→8, md1 chunked, cross-region split, indel+substitution) are all correctly deferred — none is load-bearing for a coherent ms1+mk1 single-region pure-indel v1.

- **M4 — md-codec skew is non-blocking.** Local md-codec is 0.35.0 vs pinned 0.34.0, but md1 is out of scope this cycle and the only md citation (`chunk.rs:12`, `chunk.rs:77`) is for the FOLLOWUP (b) rationale; both verified accurate in the local checkout. No action.

---

## Citation audit table

| Spec citation | Source-verified location | Status |
|---|---|---|
| §1 `repair.rs:559` — `bch_code_for_length` dispatch | `repair.rs:559` `let code = match bch_code_for_length(values.len())` | ACCURATE |
| §1 `repair.rs:406` — `RepairError::ReservedInvalidLength` | `repair.rs:406` `ReservedInvalidLength {` | ACCURATE |
| §1 `repair.rs:414` — `RepairError::UnsupportedCodeVariant` | `repair.rs:414` `UnsupportedCodeVariant {` | ACCURATE |
| §1 `cmd/inspect.rs:195` — `byte_length` | `cmd/inspect.rs:195` `writeln!(stdout, "byte_length: {}", ...)` | ACCURATE |
| §2 `mnemonic-key/.../string_layer/bch.rs:28,30` — BchCode Regular/Long | doc comments at `bch.rs:28` (BCH(93,80,8) 13-char) and `:30` (BCH(108,93,8) 15-char) | ACCURATE |
| §2 `compute_syndromes_regular -> [Gf1024; 8]`, `ms-codec/.../bch_decode.rs:190` | `ms-codec bch_decode.rs:190` `fn compute_syndromes_regular(...) -> [Gf1024; 8]` | ACCURATE |
| §2 `ms-codec/.../bch_decode.rs:416` — `if deg == 0 \|\| deg > 4` | `ms-codec bch_decode.rs:416` verbatim | ACCURATE |
| §2 `mk-codec/.../string_layer/bch_decode.rs:22` — "Runs in O(t²) for t = 4" | `mk-codec bch_decode.rs:22` verbatim; cap enforced at `:566` | ACCURATE |
| §2 `md-codec/.../chunk.rs:12` — "exceeding the BCH t = 4 capacity fails the whole call" | `md-codec chunk.rs:12` verbatim | ACCURATE |
| §2 `repair.rs:28` — 32-symbol bech32 ALPHABET | `repair.rs:27-30` `use mk_codec::...bch::{ ALPHABET, ... }` | ACCURATE |
| §2 `ms_codec::decode_with_correction ... decode.rs:188`; ascending order; empty vec ⇒ valid | `ms-codec decode.rs:188` signature; `:185-187` doc; `:202` returns `Vec::new()` when residue==0; sorted at `bch_decode.rs:443-445` | ACCURATE |
| §2 `mk1: decode_string ... bch.rs:650` | `mk-codec bch.rs:650` `pub fn decode_string(...)` | ACCURATE |
| §2 `mk_codec::decode(&[&str]) ... key_card.rs:114` | `mk-codec key_card.rs:114` `pub fn decode(strings: &[&str]) -> Result<KeyCard>` | ACCURATE |
| §5 `cmd/repair.rs:59,108` — `--json` flag / emitter call | `--json` field at `cmd/repair.rs:58-59`; emitter call at `:109`; `if args.json {` at `:108` | ACCURATE (±1) |
| §5/§6 `emit_repair_json :176` | `cmd/repair.rs:176` `fn emit_repair_json(...)` | ACCURATE |
| §6 `error.rs` — `ToolkitError::Repair(_) => 2` | `error.rs:507` | ACCURATE |
| §6 `error.rs` — `RepairShortCircuit{exit_code} => 5` | `error.rs:508` returns `*exit_code` (not literal 5); literal 5 only at `repair.rs:982`. Also 5 is the standalone-repair correction-applied code (`cmd/repair.rs:122`) | DRIFTED / imprecise — see I2 |
| §6 `error.rs` — `BundleMismatch / XpubSearchNoMatch / ImportWalletSeedMismatch all => 4` | `error.rs:474` / `:513` / `:496` | ACCURATE |
| §7 `repair.rs:388+` — RepairError variants | enum starts `repair.rs:388`; listed order/membership differs from source | ACCURATE (line) / incomplete (membership) — see M1 |
| §9(b) `md-codec/.../chunk.rs:77` — count header `read_bits(6)+1` | `md-codec chunk.rs:77` `let count = (r.read_bits(6)? + 1) as u8;` | ACCURATE |
| §11 `scripts/install.sh:32` self-pin | `install.sh:32` `...mnemonic-toolkit-v0.37.0...` | ACCURATE |
| §11 GUI `mnemonic-gui/src/schema/mnemonic.rs` repair SubcommandSchema | `REPAIR_FLAGS` at `:1513-1561`; subcommand entry `:3061` | ACCURATE |

---

## Verified-correct design/crypto claims (checked, holds up)

- **t=4 / 8-syndrome capacity (§2):** both ms-codec (`bch_decode.rs:190` 8 syndromes, `:416` `deg > 4`) and mk-codec (`bch_decode.rs:557` 8 syndromes via `decode_errors`, `:566` `deg > 4`, applied to BOTH regular `:520` and long `:537`) confirm `t = 4`. SOUND.
- **Placeholder-then-decode reduces the true omission site to ONE substitution (§3, the crux):** mathematically verified — inserting placeholder `x` at the true drop index `p` yields exactly the original codeword with `d[p]→x`, a single substitution BCH solves via BM/Forney (`ms-codec bch_decode.rs:403-449`); a wrong position misaligns the tail to Hamming distance ≫4 ⇒ `decode_regular_errors` returns `None` (>4 deg) ⇒ rejected. The ms-codec full path (`decode.rs:188-246`) additionally re-verifies residue (`:237`) and re-runs the rule-9 length gate via `decode` (`:244`, `:29`). SOUND.
- **Delete-and-validate ⇒ residue==0, no BCH, no t-limit (§3, too-long):** deleting the truly-inserted char reproduces the exact original codeword ⇒ `polymod == target` ⇒ residue 0; the toolkit already has a direct residue check (`repair.rs:621` `polymod_residue(...) == 0`). SOUND.
- **No erasure primitive (§2 FOLLOWUP (a)):** grepped `erasure|erase|Erasure|Erase` across all three `src/` trees — zero matches. SOUND; j=8 correctly deferred.
- **False-positive floor ~32⁻¹³ negligible to j=4 (§4):** 13-symbol GF(32) checksum ⇒ ~2⁻⁶⁵ per candidate; ~4×10⁶ candidates at j=4 ⇒ ~10⁻¹³ expected false positives; mk1 long code is 15 symbols (safer). SOUND.
- **Pure-indel-only acceptance is well-defined and implementable (§3, §9(d)):** both oracles return corrected positions in data-part coordinates (`ms_codec::CorrectionDetail.position`, `decode.rs:88-89`; `DecodedString.corrected_positions`, `bch.rs:578`), so "corrections == placeholder positions" is a directly comparable set. SOUND.
- **Default `--max-indel 0` preserves current behavior and leaves auto-fire untouched (§5):** the flag attaches to `RepairArgs` (standalone `repair` subcommand, `cmd/repair.rs:36-72`); auto-fire fires from inspect/convert/verify_bundle via `repair.rs:982` `RepairShortCircuit{5}` and never constructs `RepairArgs` / never sets the flag. SOUND.
- **SemVer PATCH + mandatory GUI schema_mirror + manual lockstep (§11):** new flag NAME ⇒ mandatory GUI flag-name mirror (CLAUDE.md) — correct; `--json` wire-shape extension is NOT schema_mirror-gated — correct per CLAUDE.md GUI-schema section. SOUND.
- **`--max-indel` is non-secret ⇒ no `secrets::flag_is_secret` delta (§8):** an integer budget carries no secret; ms1 candidate output reuses the existing D9 secret-on-stdout advisory (`cmd/repair.rs:118-120`). SOUND.

The two Important items are both specification-completeness gaps, not crypto/source errors; once I1 (mk1 per-chunk model) and I2 (exit-5 disambiguation) are folded and the architect re-dispatched per the CLAUDE.md per-fold loop, this spec is on track for GREEN.
