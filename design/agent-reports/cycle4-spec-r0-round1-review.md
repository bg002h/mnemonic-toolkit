# Cycle-4 codec funds-safety spec — R0 review (Round 1)

**Reviewer role:** opus software architect, mandatory R0 gate (NO implementation until 0C/0I).
**Date:** 2026-06-21.
**Spec under review:** `design/BRAINSTORM_cycle4_codec_funds_fixes.md` (H6 + M4 in md-codec; M6 in ms-codec).

**Source SHAs independently verified against (live `git show <origin-ref>:<path>`):**

| repo | branch | origin SHA (verified this review) |
|---|---|---|
| descriptor-mnemonic (md-codec / md-cli) | `main` | **`58cc9ec25b3d35120c8e785d3c2ce7f48322529b`** |
| mnemonic-secret (ms-codec / ms-cli) | `master` | **`6b289186c12380c228974ce919eb85f758348aca`** |
| mnemonic-toolkit (consumer) | `master` | **`c578e123466a61ae62db9c54769cc4928ed52aff`** |
| codex32 (registry dep) | crates.io | `0.1.0` (`~/.cargo/.../codex32-0.1.0`, checksum `d230935f…`) |

**Verdict (jump):** `R0 ROUND 1: 0C / 2I` — **RED**. Two Important findings (I1 = M4 leaves the non-correcting decode path `decode_md1_string`/`unwrap_string` over-length-accepting, undermining the spec's own "regular code is 93-bounded" domain rationale and citation-completeness; I2 = the spec under-documents an intra-crate compile-forced gate (ms-codec's exhaustive `Display`) and the toolkit pin string edits, while citing several drifted line numbers in the load-bearing lockstep sites). Three findings REPRODUCE; the lockstep-inversion claim is CONFIRMED in both halves; the M6 membership-check is provably correct and non-false-positive.

---

## Method

Re-grepped every citation against the three live SHAs above. Traced each defect's reachability on origin. Read codex32-0.1.0 `interpolate_at` + `parts_inner` + `from_string` to judge M6 correctness. Read the BIP-93 authoritative text (`/tmp/bip93.mediawiki` §"Recovering Secret", §"Security", linearity rationale) to judge the M6 framing. Verified BOTH halves of the lockstep-inversion claim against live toolkit + codec source.

---

## CRITICAL

**None.**

---

## IMPORTANT

### I1 — M4's domain cap is incomplete: the NON-correcting decode path (`decode_md1_string` → `codex32::unwrap_string`) still accepts over-length md1 with no `len > 93` cap. The spec scopes M4 only to the *correcting* decoder.

**Evidence (live `58cc9ec`):**
- `crates/md-codec/src/codex32.rs:113` `unwrap_string` — BCH-verifies via `bch_verify_regular(HRP, &symbols)` (`:144`), which is `polymod`-based and **length-agnostic**. The only length guard is a *too-short* floor `symbols.len() < REGULAR_CHECKSUM_SYMBOLS` (`:151`-region). **No upper bound.** An over-length clean md1 (residue 0) BCH-verifies and proceeds to strip the checksum and decode a payload.
- `crates/md-codec/src/decode.rs:86` `decode_md1_string(s)` is a public entry point that calls `unwrap_string` → `decode_payload`. This is the *single-string non-correcting* decode (distinct from `decode_with_correction` in `chunk.rs`, the only path M4 guards).

**Why it matters:** The spec's M4 rationale (§3, §5) is "the codex32 regular code is defined only to 93 symbols; reject `len > 93` at the decoder boundary." But M4 (§5.2) only gates `decode_with_correction` / `decode_regular_errors` / `chien_search`. A hand-crafted **clean** (residue-0) over-length md1 fed to `md decode` (not `md repair`) flows through `decode_md1_string` → `unwrap_string`, BCH-verifies (length-agnostic), and decodes an out-of-domain payload **with no rejection** — the exact out-of-code artifact H6 stops the encoder from producing, now consumed on a sibling decode path the cap does not cover. This is a funds-domain gap: a third-party over-length card would decode to *some* descriptor rather than fail-closed.

The spec actually has the evidence in hand and mis-files it: §3/§5.1 correctly note that `decode_with_correction` skips correction when `residue == 0` (pass-through), so the *correcting* path never fires on a clean over-length string — but it does NOT follow the clean over-length string to its actual decode site (`unwrap_string`), which has no cap either.

**Required fix (choose one, state it in the SPEC):**
(a) **Preferred** — add the `symbols.len() > 93` reject to `unwrap_string` (`codex32.rs:113`) as well, mirroring the existing too-short floor (`< REGULAR_CHECKSUM_SYMBOLS`), so BOTH the correcting (`decode_with_correction`) and non-correcting (`decode_md1_string`) decode paths fail-closed at the regular-code domain boundary. Add a RED unit test `unwrap_string_rejects_over_93_symbols` (and a `decode_md1_string` integration twin). This makes M4 a true *decode-domain* cap, consistent with the spec's own stated rationale; or
(b) explicitly SCOPE M4 to the correcting decoder only and add a one-line documented carve-out in the SPEC stating that `decode_md1_string`/`unwrap_string` deliberately remains length-agnostic and WHY the non-correcting over-length decode is not a funds risk (it must argue that decoding a clean over-length payload cannot mis-derive an address — a claim the spec currently does not make and I do not find self-evident, since `decode_payload` reads `bit_count` from the symbol stream).

Without one of these, the spec's M4 domain-cap claim is internally inconsistent (caps one of two decode entry points while citing a domain rationale that applies to both). Treat as **Important** (funds-domain completeness + spec self-consistency), not Critical, because the *aliasing/silent-mis-correction* funds path (the report's actual H/M finding) IS closed by the correcting-decoder guard; this is the residual non-correcting leg.

---

### I2 — Several load-bearing lockstep citations are line-drifted, and the spec under-states two compile-forced gates it relies on. (Citation accuracy + completeness of the funds-relevant exit-code mapping plan.)

The lockstep-inversion *structure* is correct (see "Lockstep-inversion verdict" below), but the spec cites stale line numbers at exactly the sites that gate the new funds-error exit codes, and it omits one positive compile-gate. Per CLAUDE.md ("plan-doc + spec citations are grep-verified at write time"; "re-grep against current origin"), these must be corrected before the plan-doc lifts them.

**Drifted / imprecise citations (live SHAs above):**
- §6.4 cites `ms_codec_exit_code` exit-2 group at `error.rs:408-409` and `_ => 1` at `:411`. **Live:** the exit-2 group (`IsShareNotSingleString | SecretShareSuppliedToCombine => 2`) is at **`crates/mnemonic-toolkit/src/error.rs:416-417`** and `_ => 1` is at **`:419`** (fn opens `:399`). Off by ~8 lines.
- §6.4 cites the ms-cli `From<ms_codec::Error>` wildcard at `crates/ms-cli/src/error.rs:245`. **Live:** the `other => CliError::BadInput("unhandled ms_codec::Error variant: …")` arm is at **`:246`** (impl opens `:132`). Off by one.
- §6.1 cites M6 sites `shares.rs:244` (`k = fields[0].0 - b'0'`) and `:264` (`interpolate_at(&parsed, Fe::S)`). **Live:** `let k = (fields[0].0 - b'0')` is at **`:242`**; `interpolate_at(&parsed, Fe::S)` is at **`:263`**; the C1 reject `return Err(SecretShareSuppliedToCombine)` is at **`:235`** (spec §6.1 says `:234`, the `if` line). The arbitrary-index primitive `interpolate_at(&defining, *pool_idx)` IS at **`:153`** (spec correct).
- §6.0/§6.1 cite `error.rs:122` as the ms-codec enum tail (`SecretShareSuppliedToCombine`) — **correct** (`:122`). The manual `Display` impl opens at **`:125`** and the combine-family arms are at **`:215-221`** (spec §6.3 says "around `:221`" — correct).

**Under-stated compile-forced gate (must be added to the SPEC):**
- The ms-codec `impl fmt::Display for Error` (`crates/ms-codec/src/error.rs:125`) is an **EXHAUSTIVE match with NO `_ =>` wildcard** (grep `_ =>` over the file returns nothing; the match closes at `SecretShareSuppliedToCombine => write!(…)` `:221`). Therefore adding `Error::InconsistentShareSet` to the enum will **force a compile error inside ms-codec's own Display impl** — a GOOD intra-crate gate. The spec §6.3 says "add the new arm there," which is correct, but does NOT state that the Display match is exhaustive and thus self-enforcing. State it: it is the *intra-crate* compile-forcing that complements the *cross-crate* silent fallthrough at the toolkit/ms-cli layer (the contrast is the whole point of D14). (The `Debug` impl delegates to `Display` (`:236-240`) so it inherits the arm automatically — note this.)

**Under-stated pin-edit requirement (must be in the SPEC/plan):**
- Toolkit pins are `md-codec = "0.37"` (Cargo.toml `:36` → caret `^0.37`, i.e. `<0.38`) and `ms-codec = "0.4.4"` (`:29` → `^0.4.4`, i.e. `<0.5`). **Both pins MUST be edited** (`0.37`→`0.38`, `0.4.4`→`0.5.0`) for the bump to resolve — a caret bump does NOT auto-cross the minor boundary. §7.2 step 5 lists the target strings, but the SPEC never states that these are *blocking edits* (a `cargo update` alone will not pull 0.38/0.5.0). `codex32 = "=0.1.0"` (`:34`) needs NO change (ms-codec 0.5.0 still depends on codex32 =0.1.0). Confirm and state.

**Required fix:** re-grep and correct all of the above line numbers against `58cc9ec` / `6b28918` / `c578e123`; add the ms-codec-Display exhaustiveness note and the explicit pin-edit requirement. None of these is a defect in the *plan* (the exit-2 routing target and the variant additions are correct); they are accuracy/completeness gaps in a funds-relevant mapping section, which CLAUDE.md treats as gate-blocking for a spec that the plan-doc will lift citations from.

---

## MINOR

### M-min-1 — D16 / §7.1 own-account version-collision figure is not grounded in the branch's current state.
The spec asserts the unmerged `feature/own-account-subset-search` branch "plans to renumber to 0.63.0." **Live:** that branch's `crates/mnemonic-toolkit/Cargo.toml:3` is **`version = "0.60.0"`** (it was cut off v0.60.0 and never renumbered); toolkit `origin/master` is **0.62.0**. The "0.63.0" figure is speculative. The mitigation (cut the pin-bump PATCH off `origin/master` 0.62.0 → **0.62.1**, rebase own-account after) is sound regardless of own-account's eventual number; just drop or soften the "0.63.0" claim (own-account will renumber to whatever is next-available at *its* merge time, ≥ 0.62.1 + whatever ships between).

### M-min-2 — §4.1 "auto-chunk is documented but FALSE" — confirm the precise doc sites.
Verified: `chunk.rs:215-219` doc comment on `SINGLE_STRING_PAYLOAD_BIT_LIMIT` says "Encoders attempt single-string emit first; if the codex32 wrapping reports 'too long', split into N chunks" — currently FALSE (`wrap_payload` never reports too-long). The H6 fix makes it true. Also the `md repair` help epilog (`md-cli/src/main.rs:241`) claims auto-chunk — the spec correctly defers that as L7 (out of scope). Both confirmed accurate. Minor: the §4.1 doc comment also states the per-chunk budget as **64** data symbols (not 80) — that is the `split()` chunk-sizing budget (`SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320 = 64*5`), distinct from the 80-symbol *regular-code* ceiling. The SPEC should not conflate the two (it does not, but the plan-doc must keep "64 = split chunk budget" vs "80 = regular-code data max" straight; the M4 per-chunk cap is the 93-codeword/80-data bound, NOT 64).

### M-min-3 — H6 boundary variable: confirm the guard reads `data_symbols.len()`, computed by `bits_to_symbols` BEFORE checksum append.
Verified `wrap_payload` (`codex32.rs:67`): `let data_symbols = bits_to_symbols(payload_bytes, bit_count)?;` then appends the 13-symbol checksum. So `data_symbols.len() > 80` is the correct guard variable (data only; checksum not yet added). The spec §4.2/§4.3 is correct. Add the named const `REGULAR_DATA_SYMBOLS_MAX = 80` adjacent to `REGULAR_CHECKSUM_SYMBOLS` (`codex32.rs:18`) as proposed — good, and assert `REGULAR_DATA_SYMBOLS_MAX + REGULAR_CHECKSUM_SYMBOLS == 93` for self-documentation.

### M-min-4 — M4 `symbols.len()` is the codeword length (data + checksum), so the `> 93` boundary is correct and aligns with H6.
Verified: `parse_chunk_symbols` (`chunk.rs:429`) returns ALL symbols after `md1` = data + 13-symbol checksum; `decode_regular_errors(residue, symbols.len())` (`chunk.rs:536`) so `data_with_checksum_len == symbols.len()` = full codeword. `> 93` rejects codewords > 93 = data > 80. This aligns the M4 decode cap with the H6 encode cap at the same 93/80 boundary. Good. (No defect — recording the confirmation so the plan-doc does not accidentally cap at 80 on the symbol vector that already includes the 13 checksum symbols.)

### M-min-5 — `Error::ChunkSymbolCountOutOfRange` distinctness from `TooManyErrors`: confirmed.
`TooManyErrors` (`md-codec error.rs:422`, the enum tail) is an error-*weight* variant (`bound: u8 = 8`). The proposed `ChunkSymbolCountOutOfRange { chunk_index, symbols, max }` is a *length-domain* variant. Distinct names, distinct fields, distinct semantics, both routing to exit 2 — clean. The two-layer M4 guard (typed boundary in `chunk.rs` + `None`-floor in `decode_regular_errors`/`chien_search`) does NOT mask any test: the boundary test goes through `decode_with_correction` (typed reject); the `None`-floor test (§5.5 #2/#3) calls `decode_regular_errors`/`chien_search` directly (unit), bypassing the boundary. Independent surfaces; neither un-RED-able. Good.

---

## Cross-checks the spec PASSES (recorded so the next round does not re-litigate)

1. **All three findings REPRODUCE on live origin.** H6: `wrap_payload`/`encode_md1_string` have no cap; default `md encode` else-branch (`md-cli cmd/encode.rs:80`, JSON twin `:63`) emits single strings unconditionally; over-length round-trips via `unwrap_string` (length-agnostic `bch_verify_regular`). M4: `chien_search` (`bch_decode.rs:284`) loops `0..data_with_checksum_len` (`:293`) unbounded; `decode_regular_errors` (`:403`) gates only weight (`:416`); `k = data_with_checksum_len - 1 - d` (`:437`) aliases for `len>93`; β order 93 confirmed (`const BETA` `:145`, test `beta_has_order_93_regular` `:477`). M6: `combine_shares` (`shares.rs:186`) interpolates over `&parsed` (ALL shares) at `:263`; no truncate-to-k, no consistency gate.

2. **H6 boundary correct.** 80 data symbols (93-symbol codeword) legal; 81+ rejected; strict `>`. The longest legal single md1 still encodes. RED test #3 constructible: existing fixtures (`bch_adversarial.rs:79 multi_chunk_descriptor()`, `chunking.rs` tests asserting `chunks.len() >= 2` and `>= 8`) prove >80-symbol descriptors exist and are trivially built.

3. **`wrap_payload` IS the lowest shared chokepoint for the default encode path.** `encode_md1_string` (`encode.rs:136`) is its only intra-crate single-string caller and is a thin wrapper; the cap at `wrap_payload` top covers every single-string caller. (The chunked path uses `split()`, which calls `wrap_payload` per *chunk* with ≤64 data symbols — well under 80 — so the cap NEVER fires on legitimate chunk emission. Confirmed: `split()` sizes via `SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320 = 64*5`, `chunk.rs:249`.)

4. **M4 preserves legitimate chunked decode.** Each chunk = ≤64 data + 13 checksum = ≤77 symbols ≤ 93. The `> 93` per-chunk cap never fires on a legitimate `--force-chunked` set. §5.4 claim correct.

5. **M6 membership-check is PROVABLY CORRECT and NON-FALSE-POSITIVE.** codex32-0.1.0 `interpolate_at(shares, target)` (`lib.rs:217`): (i) checks header agreement only (length/hrp/threshold/id), (ii) **short-circuits returning the input share directly if `target` matches an input index** (`:259-263`) — so it never multiplies by zero, (iii) otherwise Lagrange-interpolates ALL payload symbols (threshold+id+share_index+payload+checksum) and emits a full `Codex32String` with index = `target` (`:296-308`). By BIP-93's linearity property ("Lagrange interpolation of valid codewords in a BCH code will always be a valid codeword … all derived shares will have the same identifier and the appropriate share index," `/tmp/bip93.mediawiki:362-365`), for a CONSISTENT extra share `parsed[j]`, `interpolate_at(k_set, idx_j)` is **byte-identical** to `parsed[j]` (same lowercase canonicalization on both sides; `parsed` is pre-lowercased `shares.rs:205-210`, interpolate output is lowercase for lowercase hrp `:305-306`). For an INCONSISTENT share, the interpolated codeword differs. ⇒ catches ALL detectable inconsistent sets; **cannot false-positive a valid k-of-n with >k consistent shares** (every extra lies on the polynomial ⇒ equal). The one irreducible blind spot (a fully-internally-consistent-but-wrong k-subset) is correctly documented as out-of-scope (§6.2 edge case 3) and is intrinsic to BIP-93 (any k shares define *a* polynomial). The arbitrary-index primitive call shape (`interpolate_at(&defining, *pool_idx)`) already exists at `shares.rs:153` — no new GF/codex32 capability needed.

6. **M6 BIP-93 framing is ACCURATE.** Verified against `/tmp/bip93.mediawiki`: §"Recovering Secret" line 217 — "The number of shares is exactly equal to the (common) threshold value" (recovers with EXACTLY k); `ms32_recover` = `ms32_interpolate(shares, 16)` with NO digest/integrity field (`:255-256`); the spec gives NO guidance for inconsistent sets (silent wrong value). So M6 is correctly framed as **beyond-BIP-93 defense-in-depth**, NOT a conformance fix. Note: the *current* code already DEVIATES from BIP-93 by interpolating over all supplied shares rather than exactly k (BIP-93 says exactly k); the fix's truncate-to-k brings the recovery step *closer* to BIP-93 while adding the membership assertion — a strict improvement, and bit-identical to today for exactly-k and all-consistent->k sets (positive controls §6.5 #2/#3 are sound).

7. **Exactly-k / all-consistent combines stay bit-identical.** For `n == k` the membership loop is empty (identical to today). For `n > k` all-consistent, `interpolate_at(k_set, Fe::S)` over the first k equals `interpolate_at(&parsed, Fe::S)` over all n (both lie on the same degree-(k-1) polynomial; Lagrange over a consistent superset yields the same value). Confirmed no regression to valid recovery.

8. **SemVer calls correct.** md-codec MINOR 0.38.0 (two additive public `Error` variants; NOT `#[non_exhaustive]` — an external exhaustive match WOULD break, but additive variant is conventionally MINOR and our only exhaustive consumer is the toolkit, handled in lockstep — see §7.3). ms-codec MINOR 0.5.0 (`#[non_exhaustive]` enum, additive variant). md-cli PATCH (opaque `CliError::Codec(md_codec::Error)` wrapper, `md-cli error.rs:5/42` — no per-variant match, no break). ms-cli PATCH + explicit arm. toolkit PATCH 0.62.1. All sound. Publish→pin order (md-codec→md-cli; ms-codec→ms-cli; then ONE toolkit PATCH) respects the crates.io boundary. No crate pins by git that this misses (md-cli `path + version=` dual, ms-cli same; toolkit by registry). Confirmed.

9. **No mandatory manual / GUI-schema obligation.** None of H6/M4/M6 adds/removes a CLI flag, subcommand, or dropdown value. `docs/manual/tests/lint.sh` gates flag NAMES (unaffected). `mnemonic-gui schema_mirror` gates flag NAMES (unaffected). §7.4 correct. Optional error-text doc note only.

10. **No pre-existing FOLLOWUP slug** for H6/M4 in md FOLLOWUPS or M6 in ms FOLLOWUPS (grep clean on both `origin` refs). §7.4/§8 "file new on ship" is correct.

---

## LOCKSTEP-INVERSION VERDICT (D14 / §7.3 / §6.4) — **CONFIRMED in BOTH halves.**

- **md side is COMPILE-FORCED (the genuine compile-break this cycle).** `crates/mnemonic-toolkit/src/error.rs:464` `fn md_codec_exit_code` is an **exhaustive** match with **NO `_ =>` wildcard** — it ends `md_codec::Error::WireVersionMismatch { .. } => 3` (`:518`); its own docstring states "md_codec::Error is NOT `#[non_exhaustive]`; match is exhaustive." md-codec `Error` is `#[derive(Debug, Error, PartialEq, Eq)]` and **NOT `#[non_exhaustive]`** (`crates/md-codec/src/error.rs:19`). ⇒ adding `PayloadTooLongForSingleString` + `ChunkSymbolCountOutOfRange` WILL break the toolkit build at `md_codec_exit_code` once the pin bumps. The `From<md_codec::Error>` (`error.rs:956`) has an `other =>` wildcard (`:966`) so it does NOT break — only the exhaustive exit-code fn. Spec §7.3 is **correct**; the new arms must route to **exit 2** (encode/decode-reject class) and the SPEC must add them in the same pin-bump PATCH.

- **ms side is a SILENT exit-1 fallthrough (paired-PR discipline, NOT compile-forced).** `ms_codec_exit_code` (`error.rs:399`) ends with `_ => 1` (`:419`); `From<ms_codec::Error>` (`:929`) has `other => ToolkitError::MsCodec(other)` (`:939`); `friendly_ms_codec` (`friendly.rs:45`) has `_ => "unhandled ms_codec::Error variant: …"` (`:147`); ms-cli `From<ms_codec::Error>` (`ms-cli error.rs:132`) has `other => CliError::BadInput("unhandled …")` (`:246`). ms-codec `Error` IS `#[non_exhaustive]` (`:18`). ⇒ a new `InconsistentShareSet` **silently maps to exit 1 (BadInput / "success-ish-adjacent" user-input class) with a generic message** at BOTH the toolkit and ms-cli unless explicit arms are added. **This is funds-relevant and the spec flags it (§6.4) — correct, and the mapping plan (exit 2, join the combine-family `FormatViolation`/exit-2 group) is sound.** The catch is the per-phase full-suite R0 run (no compile gate). The ONE compile gate that DOES help on the ms side is intra-crate: ms-codec's exhaustive `Display` impl forces the Display arm (see I2). Net: spec's D14 inversion claim (recon mis-attributed compile-forcing to ms; it is actually md) is **CONFIRMED**.

The only residual concern is completeness, not correctness: the spec must make the exit-2 mapping for all three new variants (md: 2 variants; ms: 1) an explicit, line-cited checklist in the plan-doc, because the ms side has no compiler to catch a miss → a new funds-error silently at exit 1 is the dangerous outcome the gate exists to prevent.

---

## Required folds before Round 2 (to reach 0C/0I)

1. **I1:** resolve the non-correcting decode leg — either add the `> 93` cap to `unwrap_string`/`decode_md1_string` (preferred; with a RED test) OR document an explicit, argued carve-out for why the non-correcting over-length decode is not a funds risk. The current spec caps one of two decode entry points while citing a domain rationale that covers both.
2. **I2:** re-grep and correct the drifted line numbers (`ms_codec_exit_code` exit-2 group `:416-417` / `_ => 1` `:419`; ms-cli wildcard `:246`; `shares.rs` `k` `:242`, `interpolate_at(&parsed,…)` `:263`, C1 `:235`); add the ms-codec exhaustive-`Display` intra-crate compile-gate note; add the explicit toolkit pin-string edit requirement (`0.37`→`0.38`, `0.4.4`→`0.5.0`; codex32 unchanged); make the three-variant exit-2 mapping an explicit line-cited checklist.
3. **M-min-1:** drop/soften the speculative own-account "0.63.0" figure (branch is live at 0.60.0).

After folding, persist this review verbatim, re-dispatch the architect, and re-run until GREEN (the reviewer-loop continues after every fold per CLAUDE.md).

---

## Verdict

`R0 ROUND 1: 0C / 2I` — **RED.**
