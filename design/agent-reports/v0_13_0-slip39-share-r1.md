# v0.13.0 P1c-D — SLIP-39 share parse + render — R1 (post-GREEN)

**Phase:** v0.13.0 P1c-D (share.rs impl + harness, post-GREEN)
**Round:** R1, round 1 (post-GREEN review of impl + tests together)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14

**Verdict: 0 Critical / 0 Important / 0 Nice-to-have.**

The GREEN implementation at `crates/mnemonic-toolkit/src/slip39/share.rs`
is correct against the SLIP-0039 spec and the `python-shamir-mnemonic`
@ `17fcce14` reference. Every bit-pack offset, threshold subtraction
direction, parse-error ordering check, customization-string routing
branch, and zeroize annotation aligns with ground truth. The 10 tests
in `tests/lib_slip39_share.rs` all pass and pin the contracts that
matter most (bit-extraction via the wordlist, ext-bit routing via
vector #42, refusal-class folds via vectors #2 / #3 / synthetic
too-short / synthetic disallowed-count). The two pre-GREEN findings
(C1 vector #42 iter_exp, I1 length-gate coverage) are both folded.

No fold round is required before LOCK.

---

## No findings (verified clean against grep'd source)

### Bit-packing — encode path (`render_slip39_share`)

- **id_exp encoding** matches Python `_encode_id_exp` (share.py):
  ```python
  id_exp_int = self.identifier << (ITERATION_EXP_LENGTH_BITS + EXTENDABLE_FLAG_LENGTH_BITS)
  id_exp_int += self.extendable << ITERATION_EXP_LENGTH_BITS
  id_exp_int += self.iteration_exponent
  ```
  Constants: `ITERATION_EXP_LENGTH_BITS = 4`,
  `EXTENDABLE_FLAG_LENGTH_BITS = 1`. Toolkit:
  `(identifier << 5) | (extendable << 4) | (iteration_exponent & 0xF)`.
  Identical.

- **share_params encoding** matches Python `_encode_share_params`
  top-down shift sequence:
  - `group_index << 16` ✓
  - `(group_threshold − 1) << 12` ✓
  - `(group_count − 1) << 8` ✓
  - `member_index << 4` ✓
  - `member_threshold − 1` ✓

- **Threshold subtraction discipline**: `group_threshold`,
  `group_count`, `member_threshold` each subtract 1; `group_index`,
  `member_index` do not. Matches Python verbatim.

- **Word split** at `>> 10` / `& 0x3FF` consistent with Python
  `_int_to_word_indices`.

### Bit-packing — decode path (`parse_slip39_share`)

- **id_exp decode** is the inverse of encode and matches Python
  `Share.from_mnemonic`:
  ```python
  identifier = id_exp_int >> (EXTENDABLE_FLAG_LENGTH_BITS + ITERATION_EXP_LENGTH_BITS)
  extendable = bool((id_exp_int >> ITERATION_EXP_LENGTH_BITS) & 1)
  iteration_exponent = id_exp_int & ((1 << ITERATION_EXP_LENGTH_BITS) - 1)
  ```
  Toolkit: `id_exp_int >> 5`, `(id_exp_int >> 4) & 1`, `id_exp_int & 0xF`.
  Identical.

- **id_exp decoded BEFORE checksum**: correct, because `extendable`
  selects the customization string fed to `verify_checksum`. Matches
  Python share.py ordering.

- **share_params decode** is the inverse of encode. The `+ 1` is
  applied to `member_threshold`, `group_count`, `group_threshold` and
  NOT to the two `_index` fields. ✓

- **`_int_from_word_indices` parity:** Python uses `value = value *
  RADIX + index` accumulating left-to-right. For exactly 2 words, this
  is `indices[0] * 1024 + indices[1]`, which (since `indices[1] <
  1024`) equals `(indices[0] << 10) | indices[1]`. Toolkit's
  `(u32::from(indices[0]) << 10) | u32::from(indices[1])` is
  equivalent.

### Parse-error ordering

Step order verified against Python `Share.from_mnemonic`:

1. Word lookup — fires on first unknown word, carries `word_idx` from
   `enumerate()`. Matches Python `wordlist.mnemonic_to_indices`
   KeyError-by-position.
2. `indices.len() < MIN_MNEMONIC_LENGTH_WORDS` (`MIN_MNEMONIC_LENGTH_WORDS
   = 20`) — `< MIN`, NOT `<= MIN`. Matches Python.
3. `padding_bits > 8` — `> 8`, NOT `>= 8`. Matches Python.
4. RS1024 checksum — runs AFTER id_exp decode, because cs depends on
   the `extendable` bit. ✓
5. Non-zero leading padding bits in value — surfaced as
   `InvalidPadding { share_idx: 0 }`. Maps to Python `OverflowError`
   on `value_int.to_bytes(value_byte_count, "big")` — semantically
   equivalent (a leading bit set in the padding region inflates
   `value_int` past `2**(value_byte_count*8)`).

All 5 error variants carry `share_idx: 0`. Documented at share.rs
module-doc with the SPEC §2.5 refusal-class fold.

### `decode_value` / `encode_value` bit-stream helpers

- **MSB-first within 10-bit words** — `(9 - bit_in_word)` shift and
  `w = (w << 1) | ...` construction. Matches the SLIP-0039 §3.1
  "big-endian" requirement.
- **MSB-first within value bytes** — `(7 - bit % 8)` and
  `b = (b << 1) | ...`.
- **Padding bits BEFORE value bits** (left-padded, per SLIP-0039
  §3.1 "left-padded with '0' bits") — the leading `padding_bits` are
  read FIRST, then value bits start at `padding_bits + byte_idx * 8 +
  j`.
- **`get_bit` returns 0 or 1** — `& 1` mask.
- **`debug_assert_eq!` invariants** anchor
  `word_count * 10 == padding_bits + value.len() * 8`, which is the
  correct full-bits identity. They do NOT shadow off-by-one bugs
  because `total_value_bits - padding_bits = value_byte_count * 8` is
  guaranteed by `padding_bits = total_value_bits % 16` combined with
  `value_byte_count = (total_value_bits - padding_bits) / 8`.

### Customization-string routing (`cs_for`)

- `ext = 0` ⇒ `CS_NON_EXTENDABLE = b"shamir"`.
- `ext = 1` ⇒ `CS_EXTENDABLE = b"shamir_extendable"`.

Matches Python `constants.py`:
- `CUSTOMIZATION_STRING_ORIG = b"shamir"`
- `CUSTOMIZATION_STRING_EXTENDABLE = b"shamir_extendable"`

Routing direction verified end-to-end: vector #42 (extendable=true)
parses only under `b"shamir_extendable"` (anchored at
`lib_slip39_rs1024.rs::vector_42_extendable_*`).

### Zeroize discipline

- `#[derive(Zeroize, ZeroizeOnDrop)]` on `Share`. ✓
- `#[zeroize]` on `value: Vec<u8>`. ✓
- `#[zeroize(skip)]` on all 8 metadata fields. ✓
- `value` is private (no `pub`); metadata fields are `pub`. Matches
  SPEC §2.1.
- Custom `Debug` impl redacts `value` as `"<N bytes redacted>"` via
  `format_args!`. The 8 metadata fields are emitted verbatim —
  non-secret per SPEC §2.1 ("on the wire in the encoded mnemonic").
- `Cargo.toml` has `zeroize = { version = "1.8", features = ["derive"] }`.
  The `derive` feature pulls `zeroize_derive` as a proc-macro dep
  (build-only); no public-API leakage to downstream consumers.

### `Share::from_parts` constructor

`pub(crate)`, all 9 fields passed positionally with
`#[allow(clippy::too_many_arguments)]`. Matches SPEC §2.1's design
intent (private constructor; external callers reach `Share` via
`parse_slip39_share` or P1c-E's `slip39_split`). The I2 deferred
concern from the test-design review remains un-resolved by P1c-D's
surface choice — see "I2 status" note below.

### `mod.rs` re-export shape

```rust
pub use error::Slip39Error;
pub use share::{parse_slip39_share, render_slip39_share, Share};
```

Matches SPEC §2.1's public surface. The leading
`pub mod {error, feistel, gf256, lagrange, rs1024, share, wordlist}`
exposes the internal modules; this is consistent with current
foundation-triad shape and not a P1c-D regression. Final surface
narrowing belongs at the v0.13.0 PE close (per the
`library-error-and-language-surface-promotion` FOLLOWUP).

### Tests (`tests/lib_slip39_share.rs`)

- **`parse_vector_1_decodes_one_of_one_metadata`** — pins all 7
  metadata fields for the non-extendable 1-of-1 anchor.
- **`parse_vector_1_identifier_matches_bit_extraction`** — anchors
  identifier extraction against wordlist indices, not a literal.
  Robust to wordlist re-vendoring.
- **`parse_vector_42_decodes_extendable_bit`** — folds the C1 fix
  from the pre-GREEN review: derives `expected_iter_exp =
  (id_exp_int & 0xF) as u8` from `testify` + `swimming` wordlist
  indices rather than asserting a literal. The vector #42 fixture
  decodes to `iter_exp = 3`, not 0 by accidental symmetry with vector
  #1.
- **`render_round_trip_vector_1` / `render_round_trip_vector_42`** —
  bidirectional anchor against the byte-equal upstream-vector
  strings.
- **`parse_vector_2_returns_invalid_checksum`** — pins
  `InvalidChecksum { share_idx: 0 }`.
- **`parse_vector_3_returns_invalid_padding`** — pins
  `InvalidPadding { share_idx: 0 }` for the post-checksum-pass
  non-zero-padding fold.
- **`parse_unknown_word_reports_position`** — anchors
  `word_idx: 5` (0-based) for the substituted 6th word.
- **`parse_too_short_mnemonic_returns_invalid_padding`** — folds the
  I1 fix: 19-word refusal as `InvalidPadding { share_idx: 0 }`.
- **`parse_disallowed_word_count_returns_invalid_padding`** — folds
  the I1 fix: 21 words ⇒ `(10 * 14) % 16 = 12 > 8` ⇒ `InvalidPadding`.

All 10 tests align with the GREEN impl's fold choices.

---

## Status of pre-GREEN deferred concern (I2 — direct render test)

The pre-GREEN review's I2 concern — that render is tested only
transitively via `parse → render → string-equal` — remains
structurally un-resolved by P1c-D's surface choice (`from_parts` is
`pub(crate)`).

The mitigating factor still applies: the upstream vectors come from
an INDEPENDENT reference impl (`python-shamir-mnemonic`), so a
symmetric pack/unpack bug in the toolkit would have to produce the
same wrong wire bytes as Python's correct-by-construction reference —
implausible for the metadata-encoding paths. The vectors-set covers
`extendable ∈ {false, true}`, `iter_exp ∈ {0, 3}`, 1-of-1 metadata;
the symmetric-bug surface that escapes round-trip is restricted to
metadata configurations NOT covered by these anchors.

P1c-E will exercise the render path positively under split-driver-
generated metadata (varied group/member thresholds, group_count,
identifier values), which closes the I2 gap structurally.
**Recommendation: leave open until P1c-E lands, then re-evaluate at
PE.** Not a P1c-D LOCK blocker.

---

## Status of pre-GREEN findings

- **C1 (vector #42 `iteration_exponent = 0` literal)** — FOLDED. Test
  now extracts expected iter_exp from wordlist indices.
- **I1 (too-short + disallowed-word-count coverage)** — FOLDED. Two
  new tests pin both refusals to `InvalidPadding { share_idx: 0 }`.
- **I2 (direct render test)** — DEFERRED to P1c-E (structurally
  closes by exercising varied metadata via the split driver).

---

## Recommendation

**LOCK P1c-D without further folds.** All findings from R1 round 1
are exhausted; the 0/0/0 verdict reflects that the GREEN impl is
correct, the harness is faithful, and the pre-GREEN review's findings
folded cleanly. Proceed to P1c-E (`slip39_split` + `slip39_combine`
driver) as planned.

## References

- GREEN commit: `750f079`
- RED commit: `6f29bb7`
- Pre-GREEN test-design review: `1f771b1` (report at
  `design/agent-reports/v0_13_0-slip39-share-r1-test-design.md`)
- SPEC: `design/SPEC_slip39_v0_13_0.md` §2.1, §2.5
- [SLIP-0039 spec](https://github.com/satoshilabs/slips/blob/master/slip-0039.md)
- [python-shamir-mnemonic @ 17fcce14](https://github.com/trezor/python-shamir-mnemonic/tree/17fcce14)
