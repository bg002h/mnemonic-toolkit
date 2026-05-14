# v0.13.0 P1c-D — SLIP-39 share parse + render — TEST-DESIGN review (R1, pre-GREEN)

**Phase:** v0.13.0 P1c-D (share.rs + harness)
**Round:** R1 (test-design review, pre-GREEN)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14

This is a test-design review of `crates/mnemonic-toolkit/tests/lib_slip39_share.rs`
performed BEFORE `src/slip39/share.rs` was authored, so that any spec
misreadings would be caught cheaply rather than after a ~500-LOC GREEN
implementation. The conventional `-r1.md` filename is reserved for the
post-GREEN review of the impl + tests together; the `-test-design`
suffix disambiguates.

**Verdict: 1 Critical / 2 Important / 0 Nice-to-have.**

The test file is well-structured, the spec narrative is mostly accurate,
and the parse-error ordering matches Python ground truth. ONE critical
hand-computation error in the vector #42 metadata assertions will fail
the test regardless of how correctly the GREEN impl is written. Two
important coverage gaps — too-short mnemonic + disallowed word counts
at parse time — are within parse-producible territory but not
exercised.

---

## Critical

### C1. Vector #42 `iteration_exponent` asserted as 0 but actually decodes to 3

**Location:** `crates/mnemonic-toolkit/tests/lib_slip39_share.rs:121`

**Inaccuracy:** The test asserts `assert_eq!(s.iteration_exponent, 0)`
for vector #42. Bit-decoding the first two share words by hand against
the SLIP-0039 §3.1 layout gives `iteration_exponent = 3`, not 0.

**Ground truth (computed against `slip39_english.txt` 0-indexed):**

- `testify` = index `906`
- `swimming` = index `883`
- `id_exp_int = (906 << 10) | 883 = 928627` (= `0xE2B73`, 20-bit
  big-endian)
- Field decode per Python `Share.from_mnemonic`:
  - `identifier = 928627 >> 5 = 29019`
  - `extendable = (928627 >> 4) & 1 = 1` (matches test line 120 — OK)
  - `iteration_exponent = 928627 & 0xF = 3` (test asserts 0 — **WRONG**)

Cross-check vector #1 (test asserts `iteration_exponent = 0` on line
86 — that one is correct): `id_exp_int = (248 << 10) | 288 = 254240`;
`254240 & 0xF = 0`. ✓ So the bit math is sound; only vector #42's
assertion is mis-computed.

The fixture description "Valid extendable mnemonic without sharing (128
bits)" does NOT constrain `iteration_exponent` to 0 — it only
constrains group_count=group_threshold=member_threshold=1 ("without
sharing") and extendable=true. The author appears to have assumed
`iteration_exponent=0` by symmetry with vector #1, which only happens to
be 0 because the upstream test-generator chose those word indices.

**Recommended fix:** Change `tests/lib_slip39_share.rs:121` to
`assert_eq!(s.iteration_exponent, 3);` — or, more robustly, replace the
unmotivated literal with a bit-extraction expression mirroring the
identifier-extraction pattern on lines 95–106, e.g.:

```rust
let t = u32::from(wordlist::word_to_index("testify").expect("testify in wordlist"));
let sw = u32::from(wordlist::word_to_index("swimming").expect("swimming in wordlist"));
let id_exp_int = (t << 10) | sw;
let expected_iter_exp = (id_exp_int & 0xF) as u8;
// ...
assert_eq!(s.iteration_exponent, expected_iter_exp);
```

The robust form anchors against the wordlist itself and survives any
future re-vendoring; the literal form is fine but inherits the bit-math
hazard.

---

## Important

### I1. Test omits coverage for two pre-checksum length refusals the Python ref checks

**Location:** `crates/mnemonic-toolkit/tests/lib_slip39_share.rs:14–17, 39–50`

**Inaccuracy:** The module doc is correct that padding is left-padded
(SLIP-0039 §3.1: "This value is left-padded with '0' bits"). However,
the coverage matrix on lines 39–50 lists only 4 negative cases —
InvalidChecksum, InvalidPadding, UnknownWord — and omits the two
pre-checksum length refusals that the Python reference (`share.py`)
checks BEFORE the checksum:

1. `len(mnemonic_data) < MIN_MNEMONIC_LENGTH_WORDS` (= 20 words for
   SLIP-39) — Python raises `MnemonicError("Invalid mnemonic length.
   ...at least 20 words.")`.
2. `padding_len = (10 * (len - 7)) % 16 > 8` — Python raises
   `MnemonicError("Invalid mnemonic length.")` — fires for word counts
   not in the valid set, e.g. 21 words (`(10*14)%16 = 12 > 8`).

The SPEC §2.5 refusal-class table elides these into row 9
(InvalidChecksum) or row 16 (InvalidPadding) — but the Python ref
checks them as a separate pre-checksum gate. The toolkit's
`Slip39Error` enum has no `BadShareWordCount` variant; the most natural
fold is into `InvalidPadding { share_idx }` per `error.rs:91-94`'s
"encoding violation" semantics, but the test does not pin this contract.
Whichever fold the GREEN impl chooses, the test should anchor it
explicitly so the off-by-N pattern doesn't recur.

**Ground truth quote (Python `share.py`):**

```python
if len(mnemonic_data) < MIN_MNEMONIC_LENGTH_WORDS:
    raise MnemonicError("Invalid mnemonic length. The length of each mnemonic "
                        f"must be at least {MIN_MNEMONIC_LENGTH_WORDS} words.")

padding_len = (RADIX_BITS * (len(mnemonic_data) - METADATA_LENGTH_WORDS)) % 16
if padding_len > 8:
    raise MnemonicError("Invalid mnemonic length.")
```

**Recommended fix:** Add two negative tests:

```rust
#[test]
fn parse_too_short_mnemonic_returns_invalid_padding() {
    // 19 words — below MIN_MNEMONIC_LENGTH_WORDS=20.
    let words: Vec<&str> = VECTOR_1.split_whitespace().take(19).collect();
    let short = words.join(" ");
    let err = parse_slip39_share(&short).expect_err("19-word share must refuse");
    assert_eq!(err, Slip39Error::InvalidPadding { share_idx: 0 });
}

#[test]
fn parse_disallowed_word_count_returns_invalid_padding() {
    // 21 words: (10 * (21-7)) % 16 = 12, which is > 8.
    let words: Vec<&str> = VECTOR_1.split_whitespace()
        .chain(std::iter::once("academic"))
        .collect();
    let long = words.join(" ");
    let err = parse_slip39_share(&long).expect_err("21-word share must refuse");
    assert_eq!(err, Slip39Error::InvalidPadding { share_idx: 0 });
}
```

If the GREEN impl introduces a distinct `BadShareWordCount` variant
rather than folding into `InvalidPadding`, the SPEC §2.5 and `error.rs`
need a matching update; either way, this is a contract the test
currently leaves un-pinned.

### I2. Render contract is only tested transitively (parse → render → string-equal)

**Location:** `crates/mnemonic-toolkit/tests/lib_slip39_share.rs:133–145`

**Inaccuracy:** Both render tests round-trip through
`parse_slip39_share(VECTOR_X).expect(...)`. If `parse_slip39_share` and
`render_slip39_share` share a common bit-packing/word-mapping helper
that is symmetrically wrong (e.g., swapped endianness, off-by-one on
`member_threshold - 1` encode/decode), the round-trip can succeed while
every individually-emitted share fails to match the canonical vectors.

**Mitigating factor:** The vectors-on-disk come from
`python-shamir-mnemonic` (an INDEPENDENT reference impl). A symmetric
bug would have to produce the SAME wrong wire bytes as Python's
correct-by-construction reference, which is implausible for the
metadata-encoding paths. The trap is real but narrow.

**Recommended fix:** Add at least one direct render test that
constructs a `Share` from explicit field values and asserts the emitted
string equals VECTOR_1. Construction requires either (a) a public
`Share::new(...)` constructor exposed by P1c-D, (b) `pub(crate)` on the
private `value` field, or (c) Feistel-coupling to compute the expected
EMS from the hex_secret fixture entry (since for 1-of-1 the share value
== encrypted master secret, NOT raw entropy bytes).

(c) is the spec-cleanest but couples P1c-D's test to P1b's Feistel
primitive. (a) is the most ergonomic API addition for downstream
consumers (`Share::new` is a natural surface). (b) is the smallest
diff.

This finding interacts with the GREEN design decision on `Share`
constructibility; flag for P1c-D author to surface before writing impl.

---

## No findings (verified clean)

The following claims in the test were verified against source ground
truth and stand:

- **Bit-field layout in module doc** matches Python `share.py`
  `_encode_id_exp` / `_encode_share_params` / `Share.from_mnemonic`
  and SLIP-0039 §3.1:
  - id_exp = identifier(15) | extendable(1) | iter_exp(4): ✓
  - share_params = group_index(4) | (group_threshold−1)(4) |
    (group_count−1)(4) | member_index(4) | (member_threshold−1)(4): ✓
  - Thresholds stored as T−1: ✓ (Python re-adds the 1 on decode)
  - Group/member INDICES stored as-is: ✓ (no `+1` on decode)
  - cs routing ext=0 ⇒ b"shamir", ext=1 ⇒ b"shamir_extendable": ✓
  - Left-padding (padding bits BEFORE the value): ✓ — SLIP-0039 §3.1
    verbatim quote.

- **Parse-error ordering** matches Python `Share.from_mnemonic` step
  order:
  1. UnknownWord (via wordlist lookup KeyError)
  2. word-count + padding_len pre-checksum sanity
  3. RS1024 checksum
  4. InvalidPadding via OverflowError on `value_int.to_bytes(...)`

- **Vector citations** are byte-for-byte equal to
  `tests/fixtures/slip39_vectors.json` entries 1, 2, 3, 42 (1-based
  vector #42 = JSON 0-based index 41).

- **Vector #1 metadata assertions** all correct: `extendable=false`,
  `iteration_exponent=0`, all thresholds 1, all indices 0 (because
  share_params words are both "academic"=index 0).

- **Vector #1 identifier extraction:** `id_exp_int = (248 << 10) | 288
  = 254240`; `identifier = 254240 >> 5 = 7945`. Test derives this via
  `word_to_index` rather than hardcoding, so transcription error in
  this report's narrative does not affect the test.

- **Vector #42 ext-bit decoding** anchored at
  `lib_slip39_rs1024.rs::vector_42_extendable_verifies_under_extendable_cs`
  (passes) and `vector_42_extendable_fails_verify_under_shamir_cs`
  (passes). A successful parse therefore proves the parser decoded
  `ext=1` and routed RS1024 to b"shamir_extendable".

- **Vector #2 parse refusal:** 20 valid words; `padding_len =
  130 % 16 = 2 ≤ 8`; rs1024.verify_checksum under cs=b"shamir" fails —
  InvalidChecksum at share_idx=0 is the only producible refusal.

- **Vector #3 parse refusal:** Constructed with valid RS1024 checksum
  but non-zero leading-padding-bits — InvalidPadding fold is consistent
  with `error.rs:91-94` semantics.

- **UnknownWord position semantics:** Python returns 0-based positions
  via list-comprehension order. Test's `word_idx: 5` matches.

- **`share_idx: 0` convention:** SPEC §2.1 doesn't explicitly pin it
  but `error.rs:69-70, 73-74, 91-94` doc-anchors the position as
  0-based within the combine input — for a single-share parse,
  position 0 is the only sensible value.

- **Foundation-triad API surface usage:** `wordlist::word_to_index`,
  `Slip39Error::{InvalidChecksum, InvalidPadding, UnknownWord}` all
  exist with matching field names and required derives
  (`Debug + PartialEq + Eq`).

- **`Share` field-name alignment** between SPEC §2.1
  (`member_index`) and test (`s.member_index`). Python uses `index`;
  toolkit chooses `member_index` for symmetry with `group_index` —
  internally consistent.

- **No clippy/lint smells:** All `expect`/`expect_err` use messages;
  `split_whitespace + join(" ")` for word reconstruction.

---

## References

- Test file: `crates/mnemonic-toolkit/tests/lib_slip39_share.rs`
- Triad foundation: `crates/mnemonic-toolkit/src/slip39/{mod.rs,error.rs,wordlist.rs,rs1024.rs}`
  (R1 LOCKed at commit `7dd56fe`)
- Fixture: `crates/mnemonic-toolkit/tests/fixtures/slip39_vectors.json`
- SPEC: `design/SPEC_slip39_v0_13_0.md` §2.1, §2.5
- [SLIP-0039 spec](https://github.com/satoshilabs/slips/blob/master/slip-0039.md)
- [python-shamir-mnemonic @ 17fcce14](https://github.com/trezor/python-shamir-mnemonic/tree/17fcce14)
