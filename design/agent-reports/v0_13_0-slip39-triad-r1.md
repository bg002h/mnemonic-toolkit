# v0.13.0 Phase 1c (triad) — SLIP-39 error + wordlist + rs1024 R1 reviewer report

**Phase:** P1c (triad) — `Slip39Error` enum + 1024-word wordlist embedding + RS1024 BCH checksum
**Round:** R1 round 1
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commits under review:**
- `b6fff67` (P1c RED: Slip39Error tests)
- `246623a` (P1c GREEN: Slip39Error impl)
- `ba11828` (P1c RED: wordlist tests)
- `b74d77b` (P1c GREEN: wordlist embed + lookups)
- `7c6d429` (P1c RED: rs1024 BCH checksum tests)
- `8a11dda` (P1c GREEN: RS1024 BCH checksum impl)

**Predecessor:** `c02568d` (P1b R1 LOCK — Feistel/PBKDF2)

## Verdict

**0 Critical / 1 Important / 3 Nice-to-have — R1 next-round dispatch needed (doc-only fix).**

The cryptographic substance (RS1024 generator constants, polymod formula, customization-string handling, checksum derivation, wordlist content) all check out against the SLIP-0039 spec and the upstream `python-shamir-mnemonic` reference. One narrative inaccuracy in the `error.rs` module doc needs correction before P1c-D dispatch.

## Scope reviewed

All 14 mandatory reviewer checks per the dispatch:
- Critical: error variant SPEC mapping; wordlist data integrity vs upstream; GEN constants byte-for-byte; polymod formula vs Python reference; customization-string semantics; create_checksum unpack order.
- Important: wordlist allocations; `as u16` truncation safety; `u16 & 0x3ff` defensive mask; error-enum semver shape; Display non-leakage; test runnability.
- Nice-to-have: module-doc accuracy; `MemberThresholdMismatch` justification.

## Key validations

1. **GEN constants match SLIP-0039 §3.5 byte-for-byte.** Webfetched the spec; the 10-element array `[0xe0e040, 0x1c1c080, 0x3838100, 0x7070200, 0xe0e0009, 0x1c0c2412, 0x38086c24, 0x3090fc48, 0x21b1f890, 0x3f3f120]` in `rs1024.rs:46-49` matches exactly. Index 4 is `0xe0e0009` as expected.

2. **`polymod` formula matches spec Python reference exactly.** `chk = ((chk & 0xf_ffff) << 10) ^ u32::from(v & 0x3ff)` is the direct translation of `chk = (chk & 0xfffff) << 10 ^ v`; `0xf_ffff == 0xfffff` (20 bits). The GEN-XOR loop conditional `(b >> i) & 1 != 0` mirrors the Python ternary. `v & 0x3ff` is a defensive mask not present in Python but Rust-appropriate for `u16` inputs (caller could otherwise pass values ≥ 1024); since the spec's API contract is 10-bit symbols, masking is sound.

3. **`create_checksum` unpack order matches spec.** Python `[(polymod >> 10*(2-i)) & 1023 for i in range(3)]` yields `[poly>>20, poly>>10, poly>>0]` masked by `& 0x3ff`. The Rust impl emits the same triple in the same order at `rs1024.rs:82-86`. The XOR-with-1 is preserved.

4. **`verify_checksum` semantics correct.** `polymod(cs || data_with_checksum) == 1` matches the SLIP-39 valid-codeword sentinel. Module doc at `rs1024.rs:41-44` notes the factor-of-1 vs factor-of-0 distinction (the SLIP-39 convention that zero-data fails verification).

5. **Customization-string handling correct.** `b"shamir"` for ext=0, `b"shamir_extendable"` for ext=1, fed byte-by-byte (each promoted to a 10-bit symbol via `u16::from(*b)`). Tests `different_cs_yields_different_checksum` and `shamir_checksum_does_not_verify_under_extendable_cs` pin the differentiation. Vector #42 anchor test pins the extendable case both positive (under `shamir_extendable`) and negative (under `shamir`).

6. **Wordlist data integrity verified.** Local file `slip39_english.txt` has 1024 lines; first word is `academic`, last word is `zero`. Compared against upstream `https://raw.githubusercontent.com/trezor/python-shamir-mnemonic/17fcce14/shamir_mnemonic/wordlist.txt`: first 5 = `academic, acid, acne, acquire, acrobat`, last words ending in `zero` (index 1023). Match. Commit SHA `17fcce14` confirmed to exist on the upstream repo.

7. **Spec invariants asserted via tests.** `wordlist_is_lexicographically_sorted`, `words_are_ascii_lowercase`, `has_exactly_1024_words`, `round_trip_all_indices`. The first-4-character-uniqueness invariant is documented but explicitly not enforced at this layer (per module comment line 11-12) — acceptable.

8. **Spec-anchor RS1024 tests valid.** All four mnemonics inline in `lib_slip39_rs1024.rs` (vectors #1, #2, #21, #42) match the vendored `slip39_vectors.json` byte-for-byte. Every word in all four mnemonics resolves in the wordlist (67 unique words confirmed present).

9. **Error enum carries adequate data for CLI mapping.** `BadGroupSpec { n, t }` lets the CLI distinguish SPEC §B.2.5 row 4 vs row 5 (row 5 = `n==1 && t==1`). `InsufficientShares { group_idx, needed, got }` provides the three interpolation slots for the row-12 stem. No data shortage at the CLI mapping boundary.

10. **Display messages do not leak secret material.** All 16 branches emit non-secret metadata: indices, byte counts, share positions, group/member indices, threshold parameters. No share content, passphrase bytes, or master-secret material flows through Display.

11. **Allocations are bounded and one-time.** `WORDS` and `INDEX` are `OnceLock`-gated; `Vec<&'static str>` and `HashMap<&'static str, u16>` each populate once. The `&str` items are zero-copy borrows from `include_str!`'d data. `i as u16` cast in the index-builder is safe for `i ≤ 1023`.

## Findings

### Important

**I-1 — `error.rs` module doc miscounts SPEC §2.5 CLI-only refusal classes (conf 85).**

**File:** `crates/mnemonic-toolkit/src/slip39/error.rs:10-13`

**Issue:** The module doc claims:
> "Coverage: 15 of the 18 SPEC §2.5 refusal classes. The 3 omitted are CLI-only: `--from` variant other than `phrase=`/`entropy=`, multi-stdin contention, and `--passphrase` + `--passphrase-stdin` mutual exclusion (all rejected before reaching the library boundary)."

Cross-checked against `design/SPEC_slip39_v0_13_0.md`: SPEC §B.2.5 enumerates exactly 18 refusal classes. **Only 2** are CLI-only (rows 17 and 18). The third item mentioned (`--passphrase` + `--passphrase-stdin` mutual exclusion) is NOT a SPEC §2.5 row — it is handled by `clap::conflicts_with` and surfaces as a parse error at exit code 64, not a §2.5 refusal class.

The correct accounting is: **18 SPEC rows → 2 CLI-only (rows 17, 18) → 16 reach library → rows 4 and 5 fold into one `BadGroupSpec` variant → 15 library variants.** The fold-of-4+5 is described correctly elsewhere in `BadGroupSpec`'s doc comment (lines 36-40) but the module-level summary at line 10-13 inverts the explanation: it counts the fold as a "CLI-only" omission, which is incorrect.

**Why Important (not Nice-to-have):** This is the off-by-N narrative-inaccuracy pattern flagged by `feedback_r0_must_read_source_off_by_n.md`. The CLI handler at P2 will read this comment to plan its row-by-row stem mapping; mis-attributing one of the 15 variants to "CLI-only omission" can confuse the P2 implementer into looking for a CLI-only row that doesn't exist in §2.5, and missing the fold-handling logic.

**Fix:** Rewrite lines 10-13 as:

```rust
//! Coverage: 15 library variants spanning 16 of the 18 SPEC §B.2.5
//! refusal classes. The 2 CLI-only rows (17, 18) — `--from` variant
//! syntactically invalid; multi-stdin contention across `--share` /
//! `--from` / `--passphrase-stdin` — are rejected at the CLI boundary
//! before reaching the library. The fold from 16 rows to 15 variants
//! is rows 4 and 5 (both group-spec policy refusals) collapsing into
//! `BadGroupSpec`; the CLI handler distinguishes them at the
//! `ToolkitError` mapping layer based on the carried (n, t) values
//! (row 5 = `n == 1 && t == 1`; row 4 = all other group-spec
//! violations).
```

This is doc-only; no code semantics change.

## Nice-to-have findings (fold inline; not blocking)

**N-1 — Wordlist module doc cites a slightly stale upstream path.**

`wordlist.rs:3` says "Vendored from `python-shamir-mnemonic/wordlists/wordlist.txt`". The actual upstream path at commit `17fcce14` is `python-shamir-mnemonic/shamir_mnemonic/wordlist.txt` (no `wordlists/` subdirectory). The SHA-pin and content are correct; only the path is slightly off.

**N-2 — `parse_slip39_share` forward reference in `error.rs:21`.**

The doc comment lists `parse_slip39_share` as a `Slip39Error`-returning function, but the function will not exist until P1c-D (`share.rs`). This is intentional doc-as-contract, not staleness; flag only because future readers without P1c-D landed may grep for the symbol.

**N-3 — Defensive `v & 0x3ff` mask is undocumented.**

`rs1024.rs:60` does `u16::from(v & 0x3ff)` to keep only the low 10 bits of each input symbol. The spec's Python reference uses `v` directly (Python ints don't overflow); the mask is a Rust-appropriate defense against malformed callers passing `u16` values ≥ 1024. A one-line comment explaining the mask would aid the next reader.

## Cross-checks performed

- **WebFetched SLIP-0039 §3.5** reference Python — confirmed GEN constants and polymod formula match byte-for-byte.
- **WebFetched upstream wordlist** at commit `17fcce14` — confirmed first 5 (`academic, acid, acne, acquire, acrobat`) and trailing word (`zero`) match local file.
- **Grep'd test mnemonics** against local wordlist — all 67 unique words resolve.
- **Read `slip39_vectors.json`** vectors #1, #2, #21, #42 — inline test strings match the vendored fixture byte-for-byte.
- **Read SPEC §B.2.5** — confirmed 18 refusal classes, of which exactly 2 (rows 17, 18) are CLI-only; rows 4+5 fold into one library variant.
- **Read prior R1 LOCK reports** (math-r1, feistel-r1) — confirmed structural mirror.

## R1 next-round required

Reason: doc-only Important finding (I-1) requires a 3-line module doc rewrite in `error.rs:10-13` before the P1c-D dispatch. No code, test, or behavior change. Once fixed, this triad is clean for LOCK.

**No CI gate broken; no test broken; no semver risk; no spec-correctness risk.** The cryptographic primitives are correct; the wordlist data is correct; the error-enum data shape is correct. The fix is a narrative correction to the comment header that documents how the CLI handler will read this enum at P2.

v0.13.0 P1c (triad) R1 — next-round dispatch needed (doc-only fix to `error.rs:10-13`), then ready for LOCK round 2.

## References

- [SLIP-0039 specification (satoshilabs/slips)](https://github.com/satoshilabs/slips/blob/master/slip-0039.md)
- [python-shamir-mnemonic wordlist (trezor)](https://github.com/trezor/python-shamir-mnemonic/blob/master/shamir_mnemonic/wordlist.txt)
- [python-shamir-mnemonic commit 17fcce14](https://github.com/trezor/python-shamir-mnemonic/commit/17fcce14)
