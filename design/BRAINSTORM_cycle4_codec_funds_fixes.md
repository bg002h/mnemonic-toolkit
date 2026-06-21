# BRAINSTORM — cycle-4 codec funds-safety fixes (H6 / M4 / M6)

**Status:** DESIGN ONLY — no code, no edits. This brainstorm spec feeds the **mandatory opus-architect
R0 review loop to 0 Critical / 0 Important** (CLAUDE.md hard gate) BEFORE any implementation begins.

**Scope:** three CODEC funds-safety findings across two registry crates, grouped under one cycle-4 umbrella
as **two disjoint tracks**:
- **Track A — WS-MD-BCH (md-codec):** H6 (encode-side length cap) + M4 (decode-side length cap). PAIRED but
  INDEPENDENT (two guards, not one).
- **Track B — WS-MS-CODEC (ms-codec):** M6 (cross-share polynomial-consistency check in `combine_shares`).

Tracks A and B touch **disjoint files in different repos** → independently implementable, independently
R0-gatable, independently publishable. The only convergence point is a single downstream toolkit PATCH
pin-bump that consumes both new codec versions (§7).

This spec is **decision-complete**: every design choice is resolved with a recommended lean and there are
**no open questions** (see §9). It supersedes the report prose verbatim where the recon corrected it.

---

## 1. Source-of-truth SHA table (grep-verified at write time)

All file:line citations below were re-grepped against the live origin SHAs on 2026-06-21
(`git show <origin-ref>:<path>`). The report's citations omit the `crates/` path prefix uniformly; this
spec uses the **live `crates/…` paths**.

| repo | path | default branch | pinned origin SHA | crate version (origin) |
|---|---|---|---|---|
| md-codec / md-cli | `/scratch/code/shibboleth/descriptor-mnemonic` | `main` | **`58cc9ec`** | md-codec **0.37.0** · md-cli **0.8.0** (pins `md-codec =0.37.0`) |
| ms-codec / ms-cli | `/scratch/code/shibboleth/mnemonic-secret` | `master` | **`6b28918`** | ms-codec **0.4.4** · ms-cli **0.8.0** (pins `ms-codec =0.4.4`) · codex32 **=0.1.0** |
| mnemonic-toolkit (consumer) | `/scratch/code/shibboleth/mnemonic-toolkit` | `master` | **`c578e123`** | toolkit **0.62.0**; pins `md-codec = "0.37"`, `ms-codec = "0.4.4"`, `codex32 = "=0.1.0"` |

> **Citation-decay note:** these are 2026-06-21 snapshots. The plan-doc + every per-phase TDD MUST re-grep
> against the workstream branch HEAD before code (CLAUDE.md "plan-doc + spec citations are grep-verified at
> write time"). Pin the workstream-base SHA in each plan-doc.

---

## 2. Finding summary — all three REPRODUCE on current origin (recon-confirmed + re-verified here)

| id | finding | repo | report § | live sites (verified this write) | verdict |
|---|---|---|---|---|---|
| **H6** | default `md encode` (no `--force-chunked`) emits an arbitrary-length single md1 with no length cap; codex32 regular code is **BCH(93,80,8)** → data capped at **80 symbols** | md-codec | §172 | `codex32.rs:67` (`wrap_payload`), `encode.rs:136` (`encode_md1_string`), md-cli `cmd/encode.rs:80` (`else` → `encode_md1_string`) | **REPRODUCES** — no length cap on either fn; `else` branch emits over-length unconditionally |
| **M4** | `chien_search` loops `0..len` unbounded; `decode_regular_errors` gates error-weight only, not length; `decode_with_correction` passes `symbols.len()` uncapped → degree aliasing at `len > 93` | md-codec | §258 | `bch_decode.rs:284` (`chien_search`), `:293` (loop), `:403` (`decode_regular_errors`), `:416` (deg gate), `:437` (position map); `chunk.rs:536` (call), `:518` (`parse_chunk_symbols`), `:599` (`strings.len()==1` dispatch) | **REPRODUCES** — no `len>93` guard anywhere on the path |
| **M6** | `combine_shares` interpolates over ALL supplied shares (no truncation-to-k, no cross-share consistency check) → silent WRONG secret from a same-id inconsistent set | ms-codec | §468 | `shares.rs:186` (`combine_shares`), `:244` (`k = fields[0].0 - b'0'`), `:264` (`interpolate_at(&parsed, Fe::S)`), `:153` (existing `interpolate_at(&defining, idx)` primitive), `envelope.rs:192` (`dispatch_payload`, probabilistic backstop), `error.rs:122` (enum tail) | **REPRODUCES** — interpolates over `&parsed` (all shares); no consistency gate |

**Protocol facts corrected vs the report prose (do NOT propagate the report verbatim):**
1. **H6 cap is 80 data symbols, NOT "~67".** BIP-93: codex32 regular code = "80 characters of data and 13
   characters of the checksum" = 93 total. Math: `data_symbols ≤ 80`, `codeword = data_symbols + 13 ≤ 93`.
   Confirmed live: `REGULAR_CHECKSUM_SYMBOLS = 13` (`codex32.rs:18`); generator order 93
   (`bch_decode.rs:477` test `beta_has_order_93_regular`, `:146` `β has order 93`). The "~67" figure is
   harmless to the verdict but MUST NOT enter the SPEC.
2. **M6 is defense-in-depth BEYOND BIP-93 — NOT a conformance fix.** See §6.0 (the load-bearing framing).
3. **`dispatch_payload` backstop accepts 2 of 256 prefix bytes (~2/256), NOT ~1/256.** It branches on
   `data[0] ∈ {0x00 RESERVED, 0x02 MNEM}` then `validate()` (`envelope.rs:192-225`). Order-of-magnitude
   identical to the report's "~1/256"; stated precisely here.

---

## 3. Why H6 and M4 are TWO guards, not one (locked)

Same protocol root (the codex32 regular code is 93-symbol-bounded), same WS-MD-BCH zone, but **distinct code
paths** that each need their own gate:

- **H6 is encode-side:** `wrap_payload`/`encode_md1_string` produce an over-length artifact. Fixing only H6
  stops the *toolkit's own encoders* from emitting un-decodable cards.
- **M4 is decode-side:** a hand-crafted (or third-party) over-93-symbol md1 fed to `md repair` /
  `mnemonic repair --md1` bypasses the encoder entirely. M4 needs its own gate **even after H6 lands**.

**Reachability ordering (verified, important for the M4 RED test):** in `decode_with_correction`
(`chunk.rs:502`) the per-chunk BCH correction (`decode_regular_errors`, `chunk.rs:536`) runs in the loop
**BEFORE** the `strings.len()==1` single-vs-chunked dispatch (`chunk.rs:599`), but **only when the polymod
residue ≠ 0** (`chunk.rs:524-532` passes clean strings through). Therefore:
- An H6-produced **clean** over-length md1 (residue 0) does NOT trigger M4 by itself.
- M4 fires on an over-length md1 **carrying transcription errors** (residue ≠ 0) — the report's
  "hand-crafted over-length md1 fed to `md repair`".

This confirms H6 ≠ M4: closing the encoder does not close the decoder.

---

# TRACK A — WS-MD-BCH (H6 + M4), md-codec

**Repo:** `descriptor-mnemonic`, branch off `origin/main` **`58cc9ec`**.
**File zone:** `crates/md-codec/src/{codex32.rs, encode.rs, bch_decode.rs, chunk.rs}`. Disjoint from Track B.
**One md-codec release carries both H6 and M4** (paired change set; no inter-fix code dependency).

## 4. H6 — encode-side 80-symbol data cap

### 4.1 Locked facts
- `wrap_payload(payload_bytes, bit_count)` (`codex32.rs:67`) calls `bits_to_symbols` → builds
  `HRP + data_symbols + 13-symbol checksum`. **No `data_symbols.len()` ceiling.** (verified `:67-84`)
- `encode_md1_string(d)` (`encode.rs:136`) = `let (bytes, bit_len) = encode_payload(d)?;
  wrap_payload(&bytes, bit_len)`. **No cap.** (verified `:136-139`)
- Default `md encode` else-branch (`md-cli/src/cmd/encode.rs:80`, also `--json` `:57/:63`) calls
  `encode_md1_string` **unconditionally**; `split()` (chunked) runs ONLY under `--force-chunked`. There is
  **no length-triggered auto-chunk**. (verified)
- `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64*5 = 320` (`chunk.rs:219`) is used by `split()` chunk-sizing
  (`chunk.rs:249` `div_ceil`), NOT by the single-string path. Its doc comment (`chunk.rs:215-219`) *claims*
  "if the codex32 wrapping reports 'too long', split" — **that contract is currently FALSE**; `wrap_payload`
  never reports too-long. This fix makes the comment true.
- The `md repair` help epilog (`md-cli/src/main.rs:241`) asserts "automatic chunking when the payload
  exceeds 320 bits" — **aspirational/wrong prose**; no auto-chunk exists. (This is report finding **L7**, a
  separate trivial doc fix; NOT in cycle-4 scope. Note it; do not fix here.)

### 4.2 Decision — reject (fail-closed), do NOT auto-chunk (LEAN: REJECT)
**Enforce the cap by REJECTING with a typed error in `wrap_payload`**, directing the caller to
`--force-chunked`. Do **not** silently auto-chunk inside `wrap_payload`/`encode_md1_string`.

Rationale (decision-complete):
1. **Layer correctness.** `wrap_payload` returns a *single* `String`; auto-chunking would have to return a
   `Vec<String>`, a signature/contract change rippling through every single-string caller. Chunking already
   has a dedicated entry point (`split()` / `--force-chunked`). Keep single-string single.
2. **Funds-safety = fail-closed.** An over-length md1 that cannot round-trip under the regular code MUST
   FAIL CLOSED with a clear error, never silently emit an un-decodable/aliasing-prone card. A reject is the
   strongest fail-closed posture; an auto-chunk that silently changes the output *shape* (single→multi-card)
   without the user asking is a surprising, harder-to-audit behavior on a steel-engraving surface.
3. **Smallest correct change.** One guard + one variant + tests; no caller-signature churn.
4. **Discoverability.** The error message names `--force-chunked` (and, for library callers, `split()`), so
   the remedy is in-band. The doc comment at `chunk.rs:215-219` becomes accurate.

**Where the guard lives:** at the TOP of `wrap_payload` (`codex32.rs:67`), immediately after computing
`data_symbols` (so the cap is enforced for **every** caller of `wrap_payload`, not only `encode_md1_string`
— defense-in-depth at the lowest shared chokepoint). `encode_md1_string` inherits it transitively (no second
guard needed there, but the SPEC may add a doc note on `encode_md1_string`).

### 4.3 Exact boundary (locked)
- Reject when `data_symbols.len() > 80` (equivalently `data_symbols.len() + REGULAR_CHECKSUM_SYMBOLS > 93`).
- **Boundary is inclusive-OK at 80:** exactly 80 data symbols (93-symbol codeword) is the maximal LEGAL
  regular code and MUST still succeed. The guard is strictly `>`.
- Use the named constant for the data cap (introduce `pub(crate) const REGULAR_DATA_SYMBOLS_MAX: usize = 80;`
  in `codex32.rs`, adjacent to `REGULAR_CHECKSUM_SYMBOLS`, so `REGULAR_DATA_SYMBOLS_MAX +
  REGULAR_CHECKSUM_SYMBOLS == 93` is self-documenting). LEAN: yes, add the constant (avoids a magic 80).

### 4.4 New error variant (locked)
Add `Error::PayloadTooLongForSingleString { data_symbols: usize, max: usize }` to md-codec
`error.rs` (the report's suggested name). md-codec `Error` is `#[derive(Debug, Error, PartialEq, Eq)]`
(thiserror) and is **NOT `#[non_exhaustive]`** (verified `error.rs:18-20`). Provide a `#[error("…")]`
attribute, e.g.:
`#[error("payload is {data_symbols} data symbols; the codex32 regular code caps single strings at {max} (use chunked encoding / --force-chunked)")]`.
- **Placement:** the md-codec enum is semantic-grouped (NOT alphabetical); place the new variant adjacent to
  the BCH-family variant `TooManyErrors` (`error.rs:422`), consistent with the existing style. (The toolkit's
  alphabetical-variant rule applies to `ToolkitError`, not codec enums.)
- **PartialEq/Eq derive** holds (the fields are `usize`).

### 4.5 H6 tests (TDD, RED-first)
1. **`wrap_payload_rejects_over_80_data_symbols` (md-codec unit, RED→GREEN):** build a `payload_bytes`/
   `bit_count` whose `bits_to_symbols` yields **81** data symbols; assert `wrap_payload` returns
   `Err(Error::PayloadTooLongForSingleString { data_symbols: 81, max: 80 })`. **RED today** (returns `Ok`).
2. **`wrap_payload_accepts_exactly_80_data_symbols` (boundary positive control):** 80 data symbols → `Ok`
   with a 93-symbol codeword. Guards against an off-by-one over-reject. (GREEN today and after.)
3. **`encode_md1_string_rejects_oversize_descriptor` (md-codec integration, RED→GREEN):** construct a
   `Descriptor` whose `encode_payload` exceeds 80 symbols (the report's 2-of-3 keyed template, ~331 symbols);
   assert `encode_md1_string` errors `PayloadTooLongForSingleString`. **RED today** (emits an out-of-code
   string that even round-trip-verifies).
4. **`md_encode_default_rejects_oversize` (md-cli integration):** `md encode <oversize template>` (no
   `--force-chunked`) → non-zero exit + message naming `--force-chunked`; `md encode --force-chunked <same>`
   → exit 0 (chunked). Confirms the remedy path is live and small payloads are unaffected.

## 5. M4 — decode-side `len > 93` rejection

### 5.1 Locked facts
- `chien_search(lambda, data_with_checksum_len)` (`bch_decode.rs:284`) loops `for d in 0..data_with_checksum_len`
  (`:293`) with **no upper bound**; β has order 93 → degrees `d` and `d+93` alias for `len > 93`.
- `decode_regular_errors(residue, data_with_checksum_len)` (`bch_decode.rs:403`) gates only error weight
  (`deg == 0 || deg > 4`, `:416`); **no length gate.** Its docstring (`:395`) even says "in the `0..=93`
  range for the regular code" but never enforces it. Position map `k = data_with_checksum_len - 1 - d`
  (`:437`) aliases for `len > 93`. Returns `Option` (`None` = fail).
- `decode_with_correction` (`chunk.rs:502`) calls `decode_regular_errors(residue, symbols.len())` at
  `chunk.rs:536` with `symbols` from `parse_chunk_symbols` (`:518`), which has **no length cap**. The `None`
  is mapped to `Error::TooManyErrors { chunk_index, bound: 8 }` (`chunk.rs:537-541`).

### 5.2 Decision — reject `len > 93` at the decoder boundary with a typed error (LEAN: TWO-LAYER GUARD)
Place a length guard at **two** layers (defense-in-depth, cheapest correct):
1. **Primary (semantic, typed):** at the TOP of `decode_regular_errors` (`bch_decode.rs:403`), before
   `compute_syndromes_regular`, return early when `data_with_checksum_len > 93`. Because `decode_regular_errors`
   returns `Option`, the cleanest minimal change is to return `None` here — which the existing call site maps
   to `Error::TooManyErrors`. **BUT** `TooManyErrors` mis-describes an over-length word (it isn't an
   error-weight problem). **LEAN:** introduce a distinct typed reject at the **`chunk.rs` boundary** (item 2
   below + §5.3) so the user-facing error is accurate, and keep the `bch_decode` guard as a `None`-returning internal
   floor (belt-and-suspenders; an internal caller that ignored the boundary still can't alias).
2. **Boundary (typed, user-facing) — the load-bearing guard:** in `decode_with_correction` (`chunk.rs:502`),
   **before** the residue/correction logic, reject any chunk whose `symbols.len() > 93` with a new typed
   `Error::ChunkLengthOutOfRange { chunk_index, symbols: usize, max: 93 }` (or reuse an existing
   out-of-domain variant — see §5.3). This gives an accurate message and runs for **clean and dirty**
   over-length strings alike (it precedes the `residue == 0` pass-through at `chunk.rs:524`), so even a clean
   over-length md1 is rejected on `repair` (stricter than strictly necessary for M4, but the correct domain
   gate and it composes cleanly with H6).

> **Note on the `chien_search`-level guard:** an additional `data_with_checksum_len > 93 → None` at the top
> of `chien_search` (`:284`) is acceptable as a third floor but is **redundant** once both above are in
> place. LEAN: add it too (one line, zero risk) so the unbounded loop can never be entered out-of-domain —
> matches the "reject BEFORE the unbounded loop" mandate.

### 5.2.3 NON-correcting decode path also needs the cap (R0-r1 I1 — load-bearing)

**Gap found at R0:** §5.2 caps only the *correcting* decoder (`decode_with_correction` →
`decode_regular_errors`/`chien_search`, the `md repair` path). The **non-correcting** decode primitive
`decode_md1_string` (`crates/md-codec/src/decode.rs:86`) → `codex32::unwrap_string`
(`crates/md-codec/src/codex32.rs:113`) BCH-verifies via the **length-agnostic** `bch_verify_regular(HRP,
&symbols)` (`codex32.rs:144`) and has only a too-SHORT floor (`symbols.len() < REGULAR_CHECKSUM_SYMBOLS`,
`:151`) — **NO upper cap.** So a **clean** (residue == 0) over-length md1 fed to plain `md decode` decodes an
out-of-domain payload with no rejection — inconsistent with the "regular code is 93-bounded" rationale, which
governs BOTH decode entry points. (The aliasing in §5 is the *correcting*-path harm; this is the
*non-correcting*-path harm — a structurally over-domain word silently accepted.)

**Fix (locked):** add a too-LONG ceiling to `unwrap_string`, symmetric with the existing too-short floor —
reject `symbols.len() > 93` **before** `bch_verify_regular` (`:144`) so an out-of-domain word fails closed
earliest. **Variant LEAN:** `ChunkSymbolCountOutOfRange` carries a `chunk_index` that has no meaning for a
single string, so add a sibling **`Error::StringSymbolCountOutOfRange { symbols: usize, max: usize }`**
(decode-domain, no chunk index). This makes md-codec gain **three** new variants this cycle
(`PayloadTooLongForSingleString`, `ChunkSymbolCountOutOfRange`, `StringSymbolCountOutOfRange`) — still
additive → MINOR (see §7.1, updated). Routes to exit 2 (decode-reject class) in consumers, same as its
siblings.

**Guarantee:** the longest LEGAL single-string md1 has ≤ 80 data + 13 checksum = 93 symbols, so `> 93` never
rejects an in-domain string (symmetry with H6's 80-data cap; H6 gates encode-data, this gates decode-codeword).

### 5.3 New error variant for M4 (locked)
Add `Error::ChunkSymbolCountOutOfRange { chunk_index: usize, symbols: usize, max: usize }` to md-codec
`error.rs`, placed adjacent to the existing `ChunkSetEmpty` / `ChunkCountExceedsMax` family (`error.rs:262-294`)
since it is a chunk-shape reject. `#[error("chunk {chunk_index} has {symbols} symbols; the codex32 regular
code caps a string at {max}")]`. Do **not** overload `TooManyErrors` (semantic accuracy + clean exit-code
routing). Routes to exit 2 in consumers (decode-reject class), same as the sibling chunk-shape variants.

### 5.4 "legitimate chunked decode still works" guarantee (locked)
The cap is **per-chunk symbol count ≤ 93**, NOT a cap on the number of chunks or total payload. A valid
chunked md1 set (each chunk ≤ 64 data symbols + 13 checksum = 77 ≤ 93) is **well within** the 93 cap → fully
unaffected. The M4 RED test (§5.5) and a chunked positive-control prove this.

### 5.5 M4 tests (TDD, RED-first)
1. **`decode_with_correction_rejects_over_93_symbol_chunk` (md-codec, RED→GREEN):** hand-craft a single md1
   string of `> 93` symbols carrying ≥1 transcription error (residue ≠ 0) such that, today, `chien_search`
   aliases and `decode_regular_errors` "succeeds" at a wrong aliased position. Assert
   `Err(Error::ChunkSymbolCountOutOfRange { .. })`. **RED today** (decoder mis-corrects / accepts).
   - To make the RED deterministic, the test pins the **aliasing demonstration** the report cites empirically
     (a single error at position 100 in a 331-symbol word aliases to roots 7/100/193/286). The pre-fix
     assertion is "decode does NOT cleanly reject" (today it enters the aliasing path); the post-fix
     assertion is the typed reject.
2. **`decode_regular_errors_returns_none_for_len_over_93` (md-codec unit):** direct call with
   `data_with_checksum_len = 94` → `None`. **RED today** (proceeds into `chien_search`).
3. **`chien_search_returns_none_for_len_over_93` (md-codec unit, optional third floor):** `data_with_checksum_len
   = 94` → `None`. **RED today**.
4. **`valid_chunked_md1_still_repairs` (positive control, stays GREEN):** a legitimate `--force-chunked` md1
   set (each chunk ≤ 93 symbols) with a single in-capacity error per chunk → repairs correctly. Proves the
   guard does not regress legitimate chunked decode.
5. **`unwrap_string_rejects_clean_over_93_symbol_string` (md-codec, RED→GREEN — the §5.2.3 I1 gap):**
   construct a **clean** (residue == 0, BCH-valid) md1 of `> 93` symbols and call `decode_md1_string` (the
   non-correcting path). Assert `Err(Error::StringSymbolCountOutOfRange { .. })`. **RED today** (`unwrap_string`
   accepts it and decodes an out-of-domain payload). Positive control: a 93-symbol (80-data) legal string
   still decodes GREEN.

---

# TRACK B — WS-MS-CODEC (M6), ms-codec

**Repo:** `mnemonic-secret`, branch off `origin/master` **`6b28918`**.
**File zone:** `crates/ms-codec/src/{shares.rs, error.rs}` (and a Display/exit arm; see §6.4). Disjoint from
Track A.

## 6. M6 — cross-share polynomial-consistency check in `combine_shares`

### 6.0 BEYOND-SPEC FRAMING (load-bearing — read FIRST; verify against BIP-93 at R0)
**This is additive defense-in-depth, NOT a spec-conformance fix.** Per BIP-93, codex32 K-of-N Shamir
recovery:
- has **NO digest share** (unlike SLIP-39) — there is no integrity field to catch a wrong reconstruction;
- **mandates NO cross-share consistency check** — `ms32_recover` interpolates and the spec gives **no**
  guidance for inconsistent sets (interpolation simply computes a wrong value silently);
- **recovers from EXACTLY k shares.**

So M6 is an **inherited spec gap**, not a violation. The fix adds a NEW guard
(`Error::InconsistentShareSet`) whose rationale is **preventing a SILENT WRONG SECRET** when a user mixes
shares from different splits that happen to share the same `hrp/id/threshold/length` (same 20-bit id space,
birthday-bound at scale, or an attacker crafting a valid-checksum same-id share). The SPEC body MUST state
this honestly: it is hardening beyond BIP-93, and it documents that codex32 K-of-N carries no integrity
share. **Hard invariant:** a valid k-of-n combine (exactly k consistent shares, OR > k all-consistent
shares) MUST still recover the correct secret unchanged (§6.5 positive controls).

> R0 reviewer: confirm against BIP-93 §"Recovering the master seed" that codex32 K-of-N has no digest share
> and recovers with exactly k shares (authoritative-source check per CLAUDE.md recon policy).

### 6.1 Locked facts
- `combine_shares(shares: &[String]) -> Result<(Tag, Payload)>` (`shares.rs:186`): validates (1) per-string
  parse/checksum, (1b) lowercase canonicalization (`:203-211`), (2/C1) reject secret-at-`s`
  (`SecretShareSuppliedToCombine`, `:235`), (3) `parsed.len() < k → ThresholdNotPassed` (`:243`) where
  `k = (fields[0].0 - b'0')` (`:242`), (4) distinct indices → `RepeatedIndex`, then
  (5) `let secret = Codex32String::interpolate_at(&parsed, Fe::S)` over **ALL** `parsed` (`:263`).
  **No truncation to k; no cross-share consistency check.** (verified live `origin/master` 6b28918, `:186-266`)
- `interpolate_at` checks only header agreement (Mismatched{Hrp,Id,Threshold,Length}) — NOT polynomial
  membership (the step-5 call is `:263`). (verified)
- **The membership primitive ALREADY EXISTS in this file:** `shares.rs:153` calls
  `Codex32String::interpolate_at(&defining, *pool_idx)` to derive a share at an **arbitrary index**. So the
  fix needs no new codex32 capability — `interpolate_at(k_set, idx)` is exactly the existing call shape.
  (verified `:153`; codex32 pinned `=0.1.0`, `Cargo.toml:16` / workspace `:13`)
- `dispatch_payload` (`envelope.rs:192-225`) is a probabilistic backstop only: prefix byte ∈ {0x00, 0x02}
  then `validate()` → accepts ≈ 2/256 of random wrong secrets that also pass length/language. NOT a
  consistency check.
- `Error` (`error.rs:19`) is `#[non_exhaustive]` (`:18`), semantic-grouped (NOT alphabetical), ending at
  `SecretShareSuppliedToCombine` (`:122`). **No consistency variant exists.** (verified)
- **Not a duplicate of prior cycles:** `combine_shares` was hardened in ms-codec v0.4.1
  (`combine-no-length-validation-panic`, Entr-arm `validate()`) and v0.4.2 (uppercase canonicalization +
  same-id secret-`S` bypass). Neither added a cross-share polynomial-consistency check. M6 is distinct/open.
  (verified ms `design/FOLLOWUPS.md`)

### 6.2 Decision — truncate-to-k, then verify each remaining supplied share lies on the polynomial (LEAN)
Replace step 5 (`shares.rs:263`) with:
1. **Interpolate the secret from exactly k shares.** After the existing distinct-index check (`:254-262`),
   take the **first k** of the (sorted-or-as-given) `parsed` vector as the defining set `k_set`. Recover the
   secret at `Fe::S` from `k_set` only: `interpolate_at(&k_set, Fe::S)` (unchanged surfacing of
   `Mismatched{Hrp,Id,Threshold,Length}`).
2. **Verify EVERY remaining supplied share is consistent.** For each share `j` in `parsed[k..]`, compute the
   polynomial's value at that share's index — `interpolate_at(&k_set, idx_j)` — and assert it equals the
   supplied `parsed[j]`. On any mismatch, return the new `Error::InconsistentShareSet`.

**Why this is the cheapest correct check (decision-complete):**
- It reuses the existing `interpolate_at(set, idx)` primitive (`shares.rs:153` proves the call shape) — no
  new GF/Lagrange code, no codex32 change.
- Cost is `(n − k)` extra interpolations (each over k points). For codex32 K-of-N (`k ≤ 9`, `n ≤ 31`) this
  is trivially bounded. A full pairwise cross-check would be `O(n²)` and redundant.
- It is **exactly the spec's `ms32_recover`-from-k semantics for the recovery itself** (k-set drives the
  secret) PLUS an additive membership assertion on the extras — so a valid exactly-k combine is bit-identical
  to today, and a valid > k combine recovers the same secret AND passes (all extras lie on the curve).

**Index comparison detail (locked):** compare the **full canonical share string** (or its payload + index
fields) `interpolate_at(&k_set, idx_j)` vs `parsed[j]`. Both are `Codex32String` over the already-lowercased
canonical vector (`shares.rs:205-210`), so equality is well-defined and case-normalized. The header fields
(hrp/id/threshold/length) are already cross-checked by `interpolate_at`; the new assertion adds the
**data/polynomial** dimension.

**Edge cases (resolved):**
- `n == k` (no extras): the membership loop is empty → behavior identical to today for an in-spec exactly-k
  combine. Correct by construction.
- A same-id set where the FIRST k are themselves internally consistent but extras diverge: caught by the
  membership loop (the whole point).
- A same-id set where even the first k are from mixed splits: `interpolate_at(&k_set, Fe::S)` still returns
  *a* secret (it interpolates whatever k points it's given) — this is the irreducible k-share ambiguity BIP-93
  itself has (any k shares define a polynomial). The membership check cannot detect a fully-internally-
  consistent-but-wrong k-subset; that is **out of scope and unavoidable per spec** (document it). The fix
  closes the *detectable* case: any combine where the supplied set is NOT all-on-one-polynomial.

### 6.3 New error variant (locked)
Add `Error::InconsistentShareSet` to ms-codec `error.rs` (the report's suggested name; a unit variant — no
payload needed, though an optional `{ index: char }` naming the first divergent share index is a nice-to-have
— **LEAN: unit variant**, keep minimal; the message can name "one or more shares are not from the same
split"). Place it adjacent to the other combine-family variants (`SecretShareSuppliedToCombine` `:122`,
`IsShareNotSingleString` `:113`). `ms-codec Error` is `#[non_exhaustive]` so the addition is non-breaking for
EXTERNAL exhaustive matches (they need a catch-all already). **INTRA-crate, however, the manual `Display` impl
(`error.rs:125`, `fn fmt` `:126`) is an EXHAUSTIVE `match self` with NO `_ =>` arm** — so adding
`InconsistentShareSet` is **compile-forced** to get a `Display` arm there (a non-derived-`Debug`/`Display`
secret-leak-bound impl, per the `ms-codec-error-display-echoes-input` note at `:128`). That is the GOOD kind
of lockstep: the new variant cannot ship without its message. Add the arm in the `:125` impl.

### 6.4 Downstream lockstep (HARD facts — recon corrected here)
The recon flagged a "hard compile-time-forced lockstep". **Re-verification shows it is NOT compile-forced;
it is a SILENT exit-code/prose fallthrough** because every consumer of `ms_codec::Error` has a catch-all and
the enum is `#[non_exhaustive]`. The lockstep is therefore a **semantic-correctness discipline** (paired-PR),
NOT a build break. Sites verified:
- **ms-cli `From<ms_codec::Error>`** (`crates/ms-cli/src/error.rs:132`): has a wildcard
  `other => CliError::BadInput("unhandled ms_codec::Error variant: {:?}")` (`:246`) → a new variant
  **silently maps to `BadInput` (exit 1)** with a generic message. ms-cli SHOULD add an explicit arm so
  `InconsistentShareSet` surfaces as a **format/funds-safety violation** with an accurate message and the
  right exit code (the existing combine-family variants route to `FormatViolation`/exit 2). **Not
  compile-forced** (wildcard absorbs it).
- **toolkit `From<ms_codec::Error> for ToolkitError`** (`crates/mnemonic-toolkit/src/error.rs:929`): wildcard
  `other => ToolkitError::MsCodec(other)` (`:939`) → the new variant maps to `ToolkitError::MsCodec`
  automatically (no compile break).
- **toolkit `ms_codec_exit_code`** (`error.rs:399`): ends with `_ => 1` (`:419`) → a new variant **silently
  routes to exit 1**. It SHOULD join the exit-2 group (the `=> 2` arms ending at `SecretShareSuppliedToCombine
  => 2`, `:417`, the format-violation/funds-safety class). **This is the silent fallthrough the recon meant**
  — corrected here: it is an exit-code-correctness lockstep, NOT a compiler-enforced one (the `_ => 1`
  wildcard absorbs the new variant with no build error → the plan MUST carry an explicit line-cited arm-add at
  `:417`, since no compiler will catch a miss).
- **toolkit `friendly_ms_codec`** (`crates/mnemonic-toolkit/src/friendly.rs:45`): renders prose per variant;
  add a friendly message for `InconsistentShareSet` (verify whether it has a `_ =>` fallback — if so, the
  miss is silent-generic, not a compile error).
- **toolkit call site** `cmd/ms_shares.rs:409` `ms_codec::combine_shares(&shares_view).map_err(ToolkitError::from)`
  (verified) — inherits the fix via the pin bump; needs the exit-code + friendly arms above.

**Net lockstep verdict:** ms-codec is the only crate where the new variant is *load-bearing*; ms-cli +
toolkit need **explicit arms for correct exit-code/prose** (else silent exit-1 + generic message), filed in
the same paired bump. No compile break gates this, so it is a paired-PR discipline item (flag it loudly in
the plan-doc; the per-phase R0 full-suite run is the catch).

### 6.5 M6 tests (TDD, RED-first)
1. **`combine_inconsistent_same_id_set_rejected` (ms-codec, RED→GREEN):** construct two DIFFERENT secrets A
   and B, each split 2-of-3 with the **same** hrp/id/threshold/length (mirror the existing `encode_shares`
   fixtures, `shares.rs` tests). Combine `[A_share_1, B_share_2]` (or `[A1, A2, B3]` for the n>k extras
   case). Assert `Err(Error::InconsistentShareSet)`. **RED today** (returns B's-or-garbage secret with no
   error). This is the funds-safety RED.
2. **`combine_valid_exactly_k_unchanged` (positive control, MUST stay GREEN):** a clean 2-of-3, supply
   exactly k=2 consistent shares → recovers the correct secret A, byte-identical to current behavior.
   (Guards §6.0's hard invariant.)
3. **`combine_valid_n_gt_k_all_consistent` (positive control, MUST stay GREEN):** supply all 3 consistent
   shares of A → recovers A (extras pass the membership check). Proves the truncate-to-k + verify path does
   not regress the over-supplied legitimate case.
4. **`combine_inconsistent_extra_share_rejected` (ms-codec):** 2 consistent A-shares + 1 same-id B-share
   (n>k) → `Err(InconsistentShareSet)` (the extra fails membership even though the first k recover A).
5. **Toolkit-side (Track B pin-bump phase):** `mnemonic ms-shares combine <inconsistent same-id set>` →
   non-zero exit **2** (funds-safety class) with friendly prose, AND a valid set still combines to the right
   secret. Confirms the toolkit exit-code + friendly arms (§6.4).

---

## 7. SemVer, publish→pin chain, lockstep (cross-cutting)

### 7.1 SemVer per crate (locked)
- **md-codec → MINOR (0.37.0 → 0.38.0).** **THREE** new public `Error` variants
  (`PayloadTooLongForSingleString` [H6 encode], `ChunkSymbolCountOutOfRange` [M4 correcting-decode],
  `StringSymbolCountOutOfRange` [I1 non-correcting-decode, §5.2.3]) + previously-accepted out-of-domain
  input now rejected on encode/repair/decode (behavior tightening on never-contracted out-of-code input).
  Additive → MINOR. (Note: md-codec `Error` is NOT `#[non_exhaustive]`, so a downstream **exhaustive** match
  would break — see §7.3; within our constellation the consumers are handled in lockstep, so MINOR holds.)
- **ms-codec → MINOR (0.4.4 → 0.5.0).** New public `Error::InconsistentShareSet` + a previously-"successful"
  inconsistent combine now errors. Additive variant (enum is `#[non_exhaustive]`) → MINOR.
- **md-cli → PATCH.** Inherits H6/M4 via the exact-pin bump (`md-codec =0.37.0` → `=0.38.0`). New errors
  surface via the opaque `CliError::Codec(_)` wrapper (`md-cli/src/error.rs:5/42`) → exit 1 with the codec
  Display message (`main.rs:251-258`). No per-variant arm needed → PATCH. (MINOR only if it chooses to add
  bespoke exit-2 routing/help text for the new rejects — LEAN: PATCH, keep minimal.)
- **ms-cli → PATCH (functionally), with an explicit-arm edit (§6.4).** Inherits M6 via `ms-codec =0.4.4` →
  `=0.5.0`. Adds an explicit `InconsistentShareSet` arm in `From<ms_codec::Error>` (exit-2 FormatViolation +
  message) — a behavior refinement on an already-failing path, not a new flag/wire → PATCH. (MINOR if the
  team treats the new user-visible exit-2 error class as a feature — LEAN: PATCH.)
- **toolkit → PATCH (single combined pin-bump).** Consumes both new codec versions; adds the ms exit-code +
  friendly arms (§6.4) and (if needed) md exit-code arms (§7.3). No new toolkit flag/subcommand → PATCH
  (0.62.0 → 0.62.1). Per the release ritual: BOTH READMEs + `fuzz/Cargo.lock` version-site updates; re-run
  the full suite + fuzz before tag. **Beware the own-account branch:** toolkit `origin/master` is 0.62.0; the
  unmerged `feature/own-account-subset-search` plans to renumber to 0.63.0 — coordinate so the pin-bump
  PATCH and that MINOR don't collide on the version site (LEAN: cut the pin-bump PATCH off `origin/master`
  0.62.0 → 0.62.1; rebase own-account after).

### 7.2 Publish → pin ORDER (locked — respects crates.io boundary)
These are PUBLISHED registry crates; the toolkit pin cannot bump until the codec version is on crates.io.

**Track A (WS-MD-BCH):**
1. md-codec: brainstorm-R0 → plan-R0 → TDD (H6 + M4) → per-phase reviews → whole-diff review → tag
   `descriptor-mnemonic-md-codec-v0.38.0` → `cargo publish`.
2. md-cli: bump pin to `md-codec =0.38.0` → tag → `cargo publish` (PATCH).

**Track B (WS-MS-CODEC):**
3. ms-codec: brainstorm-R0 → plan-R0 → TDD (M6) → reviews → tag `mnemonic-secret-ms-codec-v0.5.0` →
   `cargo publish`.
4. ms-cli: bump pin to `ms-codec =0.5.0` + add the explicit error arm → tag → `cargo publish` (PATCH).

**Convergence:**
5. **ONE toolkit PATCH pin-bump** (0.62.0 → 0.62.1) consuming both `md-codec = "0.38"` and
   `ms-codec = "0.5.0"`, after BOTH codec crates are on crates.io. Adds the lockstep arms (§6.4, §7.3),
   the toolkit-side characterization tests (M4 repair round-trip, M6 combine), updates both READMEs +
   `fuzz/Cargo.lock`, re-runs full suite + fuzz, tags `mnemonic-toolkit-v0.62.1`.
   - **BLOCKING pin-string edits (R0-r1 I2 — `cargo update` is NOT enough):** the toolkit Cargo.toml pins
     `md-codec = "0.37"` (`crates/mnemonic-toolkit/Cargo.toml:36`) and `ms-codec = "0.4.4"` (`:29`) are
     **caret** requirements; for `0.x` crates caret treats the minor as the breaking digit, so `^0.37`
     resolves `<0.38` and `^0.4.4` resolves `<0.5.0`. The 0.38.0 / 0.5.0 bumps are therefore OUTSIDE the
     existing ranges — `cargo update -p md-codec` will NOT cross them. The pin STRINGS must be hand-edited to
     `"0.38"` / `"0.5"` (then `cargo update` / build refreshes `Cargo.lock` + `fuzz/Cargo.lock`). Same for the
     md-cli/ms-cli EXACT pins (`=0.37.0`/`=0.4.4` → `=0.38.0`/`=0.5.0`). The plan-doc must list these as
     explicit edits, not a `cargo update`.
   - Tracks A and B can run **fully in parallel** (disjoint repos/files, independent publish chains). The
     single combined toolkit pin-bump is the only join — cleaner than two PATCHes (one README/fuzz-lock
     touch, one suite+fuzz run).

### 7.3 md-codec lockstep — the ONE compile-forced site (recon correction, important)
Unlike the ms side, **the toolkit's `md_codec_exit_code` (`error.rs:464`) is an EXHAUSTIVE match with NO
`_ =>` wildcard** (verified — it ends `WireVersionMismatch => 3`). Because md-codec `Error` is **NOT
`#[non_exhaustive]`**, the **three** new H6/M4/I1 variants will cause a **COMPILE ERROR** in the toolkit
`md_codec_exit_code` once the toolkit pins md-codec 0.38.0. **This is the genuine compile-forced lockstep in
this cycle** (the recon attributed compile-forcing to the ms side; it is actually the md side):
- The toolkit pin-bump PATCH MUST add arms for `PayloadTooLongForSingleString` (encode-reject → exit 2, the
  decode/format class), `ChunkSymbolCountOutOfRange` (correcting-decode reject → exit 2), and
  `StringSymbolCountOutOfRange` (non-correcting-decode reject → exit 2) — all alongside the
  chunk-shape/`TooManyErrors` exit-2 group at `error.rs:516`. (The compiler WILL flag any missed arm here —
  the good lockstep; contrast the ms `_ => 1` silent fallthrough in §6.4.)
- md-cli is unaffected (opaque `CliError::Codec(_)` wrapper, no per-variant match) — verified.
- The `From<md_codec::Error> for ToolkitError` (`error.rs:956`) has a `other =>` wildcard (`:966`) → no break
  there; only the exhaustive `md_codec_exit_code` breaks.

### 7.4 Manual / GUI-schema / FOLLOWUP lockstep (locked)
- **Manual mirror (`docs/manual/`):** none of H6/M4/M6 add/remove a CLI **flag**. The flag-coverage lint
  (`docs/manual/tests/lint.sh`) gates flag NAMES → **no mandatory manual update**. Optional: mirror the new
  error TEXT/exit codes under `docs/manual/src/40-cli-reference/` for `md encode` / `md repair` /
  `ms combine` / `mnemonic ms-shares combine` if the team documents error references. **LEAN: optional
  error-doc note, not mandatory.**
- **GUI schema-mirror (`mnemonic-gui/src/schema/mnemonic.rs`):** no flag/subcommand/dropdown-value change →
  **no GUI update required.** `schema_mirror` is a flag-NAME gate, unaffected. (Confirmed against recon.)
- **FOLLOWUP companions:** none of H6/M4/M6 currently has a FOLLOWUP slug in either codec repo (verified —
  only the report carries the ids). On ship, FILE+FLIP per the shipping-commit discipline (§8 slugs). If a
  shared BCH-domain doc note crosses md↔mk surfaces, mirror companion `Companion:` lines per CLAUDE.md — but
  none is required by this fix (the cap is md-codec-local).

---

## 8. FOLLOWUP slugs (file on ship; flip status in the shipping commit)

| slug | repo | finding | what it records |
|---|---|---|---|
| `encode-no-regular-code-length-cap` | descriptor-mnemonic (md-codec) | H6 | encode-side 80-data-symbol cap; `Error::PayloadTooLongForSingleString`; ships md-codec 0.38.0 |
| `chien-search-unbounded-length` | descriptor-mnemonic (md-codec) | M4 | decode-side `len>93` reject; `Error::ChunkSymbolCountOutOfRange`; ships md-codec 0.38.0 |
| `w2-ms-slip39-gf256-1` | mnemonic-secret (ms-codec) | M6 | cross-share consistency check; `Error::InconsistentShareSet`; beyond-BIP-93 defense-in-depth; ships ms-codec 0.5.0 |
| `md-codec-exit-code-exhaustive-match-lockstep` (NEW, optional) | mnemonic-toolkit | §7.3 | records that `md_codec_exit_code` is an exhaustive match (compile-forced lockstep) so future md-codec variant adds don't surprise the next cycle |

Companion `Companion:` cross-cites: none required (no cross-codec shared action). The toolkit pin-bump PATCH
is tracked as the consumer step, not a separate slug.

---

## 9. Resolved decisions (no open questions — leans recorded)

| # | decision point | RESOLUTION (lean) |
|---|---|---|
| D1 | H6: reject vs auto-chunk | **REJECT** with typed error in `wrap_payload` (fail-closed; no signature churn; auto-chunk stays opt-in via `--force-chunked`). §4.2 |
| D2 | H6: guard location | TOP of `wrap_payload` (`codex32.rs:67`), lowest shared chokepoint; `encode_md1_string` inherits. §4.2 |
| D3 | H6: exact boundary | `data_symbols.len() > 80` (strict `>`); 80 data / 93 codeword is the maximal LEGAL value and MUST pass. Add `REGULAR_DATA_SYMBOLS_MAX = 80`. §4.3 |
| D4 | H6: error name/shape | `Error::PayloadTooLongForSingleString { data_symbols, max }`, semantic-grouped near `TooManyErrors`. §4.4 |
| D5 | M4: where to gate | TWO layers — typed `ChunkSymbolCountOutOfRange` at `decode_with_correction` boundary (user-facing, runs pre-residue-check) + `None`-return floor at `decode_regular_errors`/`chien_search` top (internal belt-and-suspenders). §5.2 |
| D6 | M4: reuse `TooManyErrors`? | NO — distinct variant for semantic accuracy + correct exit-code routing. §5.3 |
| D7 | M4 vs H6 | TWO independent guards (encode-cap vs decode-cap); hand-crafted over-length md1 bypasses the encoder. §3 |
| D8 | M6: check shape | Truncate-to-k (first k define the polynomial) → recover secret from k-set → verify each extra share via `interpolate_at(k_set, idx)` membership. Cheapest correct; reuses existing primitive (`shares.rs:153`). §6.2 |
| D9 | M6: framing | Beyond-BIP-93 defense-in-depth, NOT conformance; codex32 K-of-N has no digest share. Valid exactly-k / all-consistent combines unchanged. §6.0 |
| D10 | M6: error shape | `Error::InconsistentShareSet` unit variant; near the combine-family variants; manual `Display` arm. §6.3 |
| D11 | M6: comparison granularity | Compare full canonical `Codex32String` (lowercased) at the extra share's index vs supplied; header fields already cross-checked by `interpolate_at`. §6.2 |
| D12 | SemVer | md-codec MINOR 0.38.0; ms-codec MINOR 0.5.0; md-cli/ms-cli PATCH; toolkit PATCH 0.62.1. §7.1 |
| D13 | publish order | md-codec→md-cli; ms-codec→ms-cli (parallel tracks); then ONE combined toolkit PATCH pin-bump. §7.2 |
| D14 | compile-forced lockstep | The md side (`md_codec_exit_code` exhaustive match + md-codec NOT `#[non_exhaustive]`) is the genuine compile-forced site (recon mis-attributed to ms). ms side is a SILENT exit-1 fallthrough (paired-PR discipline). §6.4, §7.3 |
| D15 | manual / GUI | No mandatory manual flag-mirror; no GUI schema change (flag-NAME gate unaffected). Optional error-doc note. §7.4 |
| D16 | toolkit version-site collision | Cut the pin-bump PATCH off `origin/master` 0.62.0 → 0.62.1; rebase the unmerged own-account MINOR (0.63.0) after. §7.1 |
| D17 | M4/I1: non-correcting decode path (R0-r1 I1) | ALSO cap `unwrap_string` (`codex32.rs:113`, the `decode_md1_string` primitive) at `symbols.len() > 93` before `bch_verify_regular` (`:144`); new `Error::StringSymbolCountOutOfRange { symbols, max }` (no chunk_index). md-codec gains 3 variants total → still MINOR. §5.2.3 |
| D18 | toolkit caret-pin blocking edits (R0-r1 I2) | `md-codec = "0.37"`/`ms-codec = "0.4.4"` caret pins resolve `<0.38`/`<0.5.0`; the 0.38/0.5.0 bumps require HAND-editing the pin strings (+ md-cli/ms-cli exact pins) — `cargo update` alone won't cross. §7.2 |

---

## 10. MANDATORY R0 GATE NOTE

**NO code, no implementer dispatch, no plan-doc finalization until this brainstorm spec passes the
opus-architect R0 review loop to 0 Critical / 0 Important** (CLAUDE.md hard gate). The loop is: dispatch the
architect → fold findings → **persist the review verbatim to `design/agent-reports/`** → re-dispatch →
repeat until GREEN (the reviewer-loop continues after EVERY fold; stopping at R0→fold→done is insufficient).
Each track (A and B) then gets its own R0-gated plan-doc before its single-subagent TDD phases; each phase's
review persists verbatim before the fold-and-commit; a mandatory independent adversarial whole-diff review
runs post-implementation per track. Per-phase R0 reviews MUST run the FULL package `cargo test -p` suite
(not targeted `--test` targets) — the ms-cli/toolkit exit-code + friendly arms (§6.4) and the toolkit
exhaustive-match break (§7.3) are exactly the cross-phase ripples a targeted run would miss.
