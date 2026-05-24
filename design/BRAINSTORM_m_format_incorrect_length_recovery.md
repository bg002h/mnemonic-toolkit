# BRAINSTORM / SPEC — m-format incorrect-length (indel) recovery

**Slug:** `m-format-incorrect-length-recovery`
**Date:** 2026-05-24
**Source ground-truth SHA:** `origin/master` = `925f5ed` (all source citations below grep-verified at this SHA; per CLAUDE.md, re-grep at plan-doc lift time).
**Status:** brainstorm APPROVED by user — feeds the implementation plan-doc (writing-plans), which then passes the **mandatory opus architect R0 gate (0C/0I) before any code**.
**SemVer:** **PATCH** (additive, default-off flag) — but a new flag NAME ⇒ mandatory GUI `schema_mirror` + manual mirror lockstep.
**Cycle scope:** toolkit-only; ms1 + mk1; md1 deferred.

---

## 1. Problem

Recover an `m*1` string (this cycle: `ms1` BIP-39 entropy, `mk1` xpub) where a character was **inserted (string too long)** or **dropped (string too short)** during hand-copy / steel engraving, so the bech32m string no longer decodes.

This is **distinct from `mnemonic repair`** (shipped v0.22.0+), which is BCH **substitution** correction at **fixed** length. `crates/mnemonic-toolkit/src/repair.rs` picks the BCH code variant *from the input length* via `bch_code_for_length` (dispatch at `repair.rs:559`); a wrong length therefore errors as `RepairError::ReservedInvalidLength` (`repair.rs:406`) or `UnsupportedCodeVariant` (`repair.rs:414`), or selects the wrong code. An indel shifts every subsequent symbol, breaking the fixed-length BCH codeword — so it needs a different algorithm layered *around* the existing decode, not inside it.

`mnemonic inspect` reports `byte_length` (`cmd/inspect.rs:195`) but offers no recovery.

---

## 2. Background — the m-format BCH codes (origin of the "1–4" ceiling)

All three sibling codecs share the same BCH code family (grep-verified at `925f5ed`):

- **Regular code: `BCH(93,80,8)`**, 13-symbol checksum. **Long code: `BCH(108,93,8)`**, 15-symbol checksum.
  (`mnemonic-key/.../string_layer/bch.rs:28,30`; same in `ms-codec`/`md-codec`.)
- The trailing `8` = **8 syndromes** computed (`compute_syndromes_regular -> [Gf1024; 8]`, `ms-codec/.../bch_decode.rs:190`), giving Berlekamp–Massey an **error-correction capacity of `t = 4`**:
  - `ms-codec/.../bch_decode.rs:416` — `if deg == 0 || deg > 4 { /* > 4 errors is above the BCH(93,80,8) / t = 4 capacity */ return None; }`
  - `mk-codec/.../string_layer/bch_decode.rs:22` — "Runs in `O(t²)` for `t = 4`."
  - `md-codec/.../chunk.rs:12` — "exceeding the BCH `t = 4` capacity fails the whole call".

**Consequence used by this design:** the existing decoders correct up to **4 substitution errors**. The too-short (omission) recovery leans on that error-decoder, so it is bounded at **j ≤ 4**. That is the exact origin of the user's "1–4 guaranteed repairable" — it is the code's `t`, not an arbitrary cap. (A *true erasure* decode, positions-known, would reach `2t = 8`, but the codecs expose **error**-decode only, not erasure-decode — see FOLLOWUP (a).)

Validity oracle = the codec full decode. The data charset is the 32-symbol bech32 `ALPHABET` (`repair.rs:28`); `1` is reserved as the bech32 separator and never appears in the data charset.

Decode oracle entry points (pinned versions ms-codec `0.2.0`, mk-codec `0.3.1`):
- ms1: `ms_codec::decode_with_correction(&str) -> (Tag, Payload, Vec<CorrectionDetail>)` (`ms-codec/.../decode.rs:188`). Returns corrections in ascending `position` order; **empty vec ⇒ input was already a valid codeword**.
- mk1: `mk_codec::string_layer::bch::decode_string(&str)` (`bch.rs:650`) / `mk_codec::decode(&[&str])` (`key_card.rs:114`).

---

## 3. The algorithm — one validator, two candidate producers

A full m-format **string** is `<hrp>1<data-part>` — `ms1…` or `mk1…`: a **3-char fixed prefix** (2-char HRP + the `1` separator) followed by the bech32 data-part (`payload ‖ checksum`). The BCH checksum covers `hrp_expand(hrp) ‖ data`; the `1` separator does not participate.

> **Note (mk1 multi-string):** an `ms1` card is always one string, but an `mk1` card is **one string only when the bytecode ≤ 56 bytes** (`mk-codec/.../consts.rs:33` `SINGLE_STRING_LONG_BYTES = 56`); a normal xpub card is **chunked into multiple `mk1` strings** (`mk-codec/.../string_layer/pipeline.rs:7-22` — "≈84 bytes → fragments of 53 + 35 bytes" = two strings; `MAX_CHUNKS = 32`, `consts.rs:42`). The per-chunk model is specified below.

**For a valid string the total length is fixed/known; an indel can land anywhere** — including the HRP or separator. Two regions ⇒ two producers, both feeding one validator.

### Validator
Reconstruct the candidate full string and call the codec's `decode_with_correction` (does `hrp_expand` + BCH + semantic decode). A candidate is **accepted** iff it decodes to a valid `(Tag, Payload)`.

### Producer P1 — prefix-region repair (trivial, separate algorithm)
The prefix must be exactly `ms1` or `mk1` (3 known chars). An indel in the prefix leaves the **data-part intact and correctly sized**, so the BCH search finds nothing; repair = recognize the leading region is within `j` indels of a known prefix.
- Enumerate indel-restorations of the leading region to a known `ms1`/`mk1` prefix (constant-size candidate set — the prefix is fixed and tiny).
- Reconstruct `ms1<data>` / `mk1<data>` and validate. The true HRP makes BCH pass (data intact).
- Disambiguation between `ms`/`mk` is automatic: only the original HRP's `hrp_expand` yields a valid checksum.

### Producer P2 — data-region repair (the bulk; bounded by `t = 4`)
Prefix intact (`ms1`/`mk1` present and correct); data-part is wrong-length. We **do not know** a priori whether the card is too long or too short (the codes accept a dense data-part length range — regular `[14,93]`, long `[96,108]` — so a dropped char often lands on a still-*code-valid* length and merely fails to decode). So **search is bidirectional** up to budget `N`:

- **Too long (engraving added char(s)) → delete-and-validate.** For each `j`-subset of data positions to delete (`C(L, j)`), form the shortened data, check `residue == 0`. The true deletion reproduces the **exact** original codeword → valid. **No BCH correction runs → no `t` limit**; bounded only by combinatorics + the false-positive floor.
- **Too short (engraving dropped char(s)) → placeholder-then-decode.** *(the "positions, not characters" insight)* For each choice of `j` insertion positions (`≈ C(V, j)`), insert a fixed **placeholder** symbol (restores length + realigns the tail) and call `decode_with_correction`. At the *true* omission site(s) the placeholder(s) are the only wrong symbols → the BCH decoder sees `≤ j ≤ 4` "errors", **solves the missing symbol(s)** via Berlekamp–Massey/Forney, and re-decodes. A *wrong* position misaligns the tail → ≫4 apparent errors → `None` → rejected. We test **N positions, not 32·N (symbol,position) pairs** — the code computes the omitted symbol.

### mk1 per-chunk model (locked, was R0 I1)
A transcription indel lands in exactly **one** mk1 chunk, changing that chunk's length while sibling chunks stay intact. The existing `repair_card` for `CardKind::Mk1` iterates per-chunk and is **atomic** (D8: any one chunk failing fails the whole call — `repair.rs:690-708`, `repair_chunk_one(...)?` `:700`; `--mk1` is `repeating: true`, `cmd/repair.rs:46-47`). Therefore indel recovery for mk1:
- operates on the **single failing chunk**, not the joined input;
- per-chunk validator = `mk_codec::string_layer::bch::decode_string` (`bch.rs:650`) — BCH over one string;
- budget `N` is applied **per chunk** (each chunk is its own codeword; a single chunk's data-part caps at 108 symbols, consistent with §4's N≈100);
- after the failing chunk is recovered, the full `mk_codec::decode(&[&str])` reassembly (`key_card.rs:114`) re-runs to confirm the cross-chunk hash — a byte-exact chunk recovery satisfies it.

ms1 is always single-string, so this reduces to the single-codeword case.

### Region-budget policy (v1)
**Single-region per attempt:** spend the full budget `N` within one region — run P1 up to `N` *and* P2 up to `N`, union the validated candidates. Cross-region split (indels in *both* prefix and data simultaneously) is **deferred** (FOLLOWUP (c)).

### Pure-indel only (v1)
Accept a P2 candidate only if `decode_with_correction`'s correction set equals exactly the placeholder positions (no extra substitutions). Indel **+** substitution (sharing the budget as `j_indel + e_subst ≤ 4`) is **deferred** (FOLLOWUP (d)).

---

## 4. Feasibility & uniqueness (why 1–4 is safe; runtime is the only gate)

Data-part length `N ≈ 100` worst case (for mk1 this is the **per-chunk** data-part length — a single chunk caps at 108 symbols, so the bound holds); 13-symbol checksum ⇒ false-positive floor ≈ `32⁻¹³` per accepted candidate.

| j | candidates ~C(N,j) | too-long cost | too-short cost | expected false positives |
|---|---|---|---|---|
| 1 | ~10² | instant | instant | ~0 |
| 2 | ~5·10³ | ms | ms | ~0 |
| 3 | ~1.6·10⁵ | <1 s | ~1–2 s | ~0 |
| 4 | ~4·10⁶ | ~1 s | tens of s | ~0 |

- **Uniqueness is never the binding constraint inside 1–4** — the long checksum keeps expected false positives ≈ 0 even at `j = 4`.
- **Runtime is the only gate**, and only for too-short at `j ≥ 3`. ⇒ stderr notice at `N ≥ 3`; hard ceiling `N = 4` enforced at the CLI (`value_parser` range `0..=4`).
- The earlier `32ᵏ` blow-up (naive "insert each of 32 symbols") is **eliminated** by the placeholder/error-decode mechanism.

---

## 5. CLI surface

- One flag on `mnemonic repair`: **`--max-indel <N>`**, `value_parser` range **`0..=4`**, **default `0`**.
  - `0` ⇒ today's behavior **byte-for-byte** (no indel attempts). The verify-bundle / repair **auto-fire** path never sets the flag, so it stays at `0` — auto-fire is unchanged (no expensive search runs automatically).
  - Active range `1–4`.
- `N ≥ 3` ⇒ stderr notice: "searching up to N indels; this may take a few seconds."
- **md1 input that would enter indel search** (normal decode failed *and* `--max-indel ≥ 1`) ⇒ explicit footgun-tagged refusal ("indel recovery not yet supported for chunked md1"); deferred to FOLLOWUP (b). An md1 that decodes cleanly is unaffected by the flag.
- Indel search engages **when normal repair yields no valid decode** (NOT merely "length invalid"). This matters most for **mk1**, whose code-valid data-part lengths are dense (`[14,93] ∪ [96,108]`), so a dropped char usually lands on a still-code-valid length and merely fails to decode. **ms1** lengths are *sparse* — `VALID_STR_LENGTHS = [50, 56, 62, 69, 75]` (`ms-codec/.../consts.rs:33`, gaps of 6–7) — so a single ms1 drop typically lands off the rule-9 length set; but the toolkit-layer `bch_code_for_length` band still accepts the off-by-one length without erroring early (`repair.rs:559` + `repair_via_ms_codec` mapping), so the "trigger on no-valid-decode" predicate is correct for both. (A correct candidate auto-satisfies ms1 rule-9 since it restores the original valid length, so the sparsity is benign.)
- `--json` already exists on repair (`cmd/repair.rs:59,108`; emitter `emit_repair_json` `:176`).

---

## 6. Exit / output contract

**Aligned with the standalone `repair` subcommand's EXISTING contract** (`cmd/repair.rs:122` `Ok(if total_repairs == 0 { 0 } else { 5 })`, documented `:12-14`): **`0` = input already valid (nothing to do); `5` = a correction was applied (REPAIR_APPLIED — "verify this").** An indel recovery IS a correction applied, so a **unique recovery returns `5`**, NOT `0` — reusing the existing `total_repairs > 0 ⇒ 5` exit path with no override (R0 I2). The other families (`error.rs`): `ToolkitError::Repair(_) => 2` (`:507`); `4` = "human-review / no single answer" (`BundleMismatch` `:474`, `XpubSearchNoMatch` `:513`, `ImportWalletSeedMismatch` `:496`). (Note: `5` is *also* the auto-fire short-circuit value at `repair.rs:982`, but the standalone subcommand reaching `5` for a correction is the primary meaning here — the earlier "5 is auto-fire only" framing was imprecise.)

| Outcome | stdout | stderr | exit |
|---|---|---|---|
| Input **already valid** (no indel needed) | (passthrough) | — | **0** |
| **Unique** recovery (correction applied) | recovered string | (ms1 secret advisory) | **5** |
| **Ambiguous** (≥2 candidates) | all candidates | "ambiguous: N candidates, choose manually" + ms1 advisory | **4** |
| **Unrecoverable** within budget | — | "unrecoverable within --max-indel N" | **2** (new `RepairError` variant) |

- `--json`: extend the repair envelope with **`status: "unique" | "ambiguous" | "unrecoverable"`** and **`candidates: [...]`**. The `--json` **wire-shape is NOT schema_mirror-gated** (that gate covers flag-NAMES only) → GUI consumers self-update via the paired-PR rule.

---

## 7. Error handling

- New `RepairError` variant for the unrecoverable-within-budget case, placed **alphabetically** per the CLAUDE.md convention for *new* variants. NB: the existing enum is **not** alphabetically sorted today — actual source order (`repair.rs:388+`) is `EmptyInput, HrpMismatch, TooManyErrors, UnparseableInput, ReservedInvalidLength, UnsupportedCodeVariant, PostCorrectionDecodeFailed` (retroactive sort tracked as `error-rs-retroactive-alphabetical-sort`). The implementer inserts the new variant alphabetically and must **not** "fix" the surrounding pre-existing order as a surprise. Maps to exit 2 via the existing `ToolkitError::Repair(_) => 2` arm (`error.rs:507`).
- Ambiguous-result **exit 4** is signaled distinctly — NOT a `RepairError` (candidates are still emitted to stdout, so it is not an error in the `Err` sense). Cleanest: the indel path returns `Ok(4)` from `cmd/repair.rs::run` after emitting the candidate list (parallel to how the unique/already-valid path returns `Ok(5)`/`Ok(0)`). Exact placement is an R0/plan detail (§12).

---

## 8. Secret hygiene (ms1)

- `--max-indel` is an **integer budget**, not secret-carrying ⇒ `secrets::flag_is_secret` is unchanged ⇒ **no GUI secret-projection delta**.
- ms1 candidate output IS secret-on-stdout (the D9 advisory class; same as `final_word`/`repair`/`seed-xor`). **Reuse repair's existing ms1 secret-on-stdout advisory**, fired per emitted candidate (ambiguous case emits several). mk1 is public.

---

## 9. Scope / deferred (FOLLOWUPs to file)

This cycle: **toolkit-only**, **ms1 + mk1**, single-region, pure-indel, `j ≤ 4`.

FOLLOWUPs (new):
- **(a) erasure-decode → j = 8** — add a position-known erasure-decode primitive to the ms/mk/md codecs (capacity `2t = 8`), reaching 5–8 omissions and removing the placeholder trick. **Sibling-codec change** ⇒ companion entries in `mnemonic-secret`/`mnemonic-key`/`descriptor-mnemonic` `design/FOLLOWUPS.md` with cross-citing `Companion:` lines (CLAUDE.md cross-repo convention).
- **(b) md1 chunked indel recovery** — md1 is chunked (each chunk a separate codeword; count header `read_bits(6)+1`, `md-codec/.../chunk.rs:77`); an indel lives inside one chunk and the chunk-count header is itself corruptible. Materially more algorithm.
- **(c) cross-region split** — indels distributed across prefix AND data-part simultaneously.
- **(d) combine indel + substitution** — share the `t = 4` budget as `j_indel + e_subst ≤ 4`.

At ship: flip `m-format-incorrect-length-recovery` `open → resolved` in `design/FOLLOWUPS.md`.

---

## 10. Testing strategy

Repair/inspect tests are **BIN-target** (`cargo test -p mnemonic-toolkit --bins` / `--test <name>`), **not** `--lib` (modules are bin-private).

- **Engine unit tests (`indel.rs`)** — a pure function over (symbols, budget, decode-oracle):
  - Canonical zero-entropy ms1/mk1 vectors; programmatically inject 1–4 indels (both directions, both regions) and assert exact recovery.
  - Assert **uniqueness** on real vectors; construct/confirm an ambiguous case if one exists at small length.
  - Assert **pure-indel rejection**: a candidate requiring corrections outside the placeholder positions is NOT accepted (v1).
  - md1 input ⇒ refusal.
  - `#[ignore]`-gated `j = 4` long-string **runtime-sanity** test.
- **Integration (`cmd/repair.rs`)** — flag plumbing; exit `0` (already valid) / `5` (unique recovery applied) / `4` (ambiguous) / `2` (unrecoverable); `--json` `status`/`candidates`; ms1 secret advisory fires; md1 refusal; `--max-indel 0` is byte-identical to current repair (regression guard); `--max-indel 5` rejected by clap.

---

## 11. Lockstep obligations (Phase-6 + paired-PR)

- **GUI `schema_mirror`** — add `--max-indel` to the `repair` `SubcommandSchema` in `mnemonic-gui/src/schema/mnemonic.rs` (paired PR; flag-NAME change ⇒ mandatory).
- **Manual mirror** — `docs/manual/src/40-cli-reference/` repair entry + a new indel-recovery section; `docs/manual/tests/lint.sh` flag-coverage.
- **No sibling-codec change** this cycle (FOLLOWUP (a) is the future one).
- **Phase-6 release-prep**: Cargo.toml + Cargo.lock (staged with the bump) + both README `<!-- toolkit-version: X -->` markers + `scripts/install.sh:32` self-pin + CHANGELOG + FOLLOWUP status flip + GUI-schema lockstep.

---

## 12. Open items to confirm at plan-doc R0

1. Exact placement of the `Ok(4)` ambiguous return in `cmd/repair.rs::run` (§7) relative to the existing `total_repairs`-based `Ok(0)`/`Ok(5)`.
2. Precise trigger predicate: per R0, "normal repair yields no valid decode" maps to `repair_card` returning `Err(RepairError::{TooManyErrors | PostCorrectionDecodeFailed | UnparseableInput | …})`. The plan must **enumerate exactly which `RepairError` variants trigger indel search vs pass through**, wired without disturbing the auto-fire short-circuit.
3. *(Resolved by §3 mk1 per-chunk model, R0 I1.)* mk1 indel recovery operates per-chunk; per-chunk validator = `decode_string`; full `decode(&[&str])` confirms reassembly. Plan-doc implements the per-chunk control flow against the D8 atomic-fail path.
4. Placeholder symbol choice and the residue/`decode_with_correction` reuse for the too-short producer — confirm `decode_with_correction` accepts an arbitrary placeholder and reports the corrected position(s) as expected (R0 verified the mechanism is sound; plan pins the placeholder value).
5. Re-grep all `:line` citations above against `origin/master` at plan-doc write time (they decay).
