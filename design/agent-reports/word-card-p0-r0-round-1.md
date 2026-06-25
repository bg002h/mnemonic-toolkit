# Word-Card Encoding — P0 R0 Review (Round 1)

**Scope:** Per-phase R0 gate (0C/0I) for P0 — the purely-additive "canonical
payload" accessor added to two sibling codecs on feature branches (NOT merged).
Reviewer: opus architect. Adversarial.

**Branches under review (both `feat/wc-p0-canonical-payload`):**
- mk-codec `/scratch/code/shibboleth/mnemonic-key` — base `master@46631c6`,
  branch head `7cbd5da`. (NB: the `git diff master...branch` form the prompt
  cited fails — `master` resolves to `46631c6` which **is** the branch's
  merge-base, but the three-dot form errored in this checkout; the two-dot
  `46631c6..feat/wc-p0-canonical-payload` is the correct, equivalent diff and
  is what was reviewed. Branch is 1 commit ahead of master, no divergence.)
- md-codec `/scratch/code/shibboleth/descriptor-mnemonic` — base `main@7764145d`.

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

Both accessors are correct, purely-additive facades over already-public
delegation targets, with no Cargo.toml/version/behavior change (NO-BUMP holds).
The KATs are genuinely adversarial — not vacuous — and the two I2-risk
determinism concerns (mk1 CSPRNG `chunk_set_id`; md1 `total_bits` /
TLV-ordering) are each closed by a load-bearing KAT that I independently
verified against the source, not just the test text. Full package suites pass
(mk 176 + md 419 incl. the new 4 + 5 KATs); clippy and fmt clean in both repos.
P0 is cleared to merge; P1 may start.

One forward-looking item (not a P0 blocker) is confirmed real and flagged for
P6 reframing. Two Nits below are optional polish only.

---

## Critical

None.

---

## Important

None.

---

## Minor / Nit

**N1 (mk-codec, cosmetic) — `from_canonical_payload_bytes_rejects_garbage`
does not assert *which* error.**
`tests/canonical_payload.rs:181-224` checks `.is_err()` for empty / 1-byte /
8-zero / trailing-byte inputs but not the variant. The doc-comment on the
method (`key_card.rs:127-138`) promises specific variants (`UnexpectedEnd`,
`TrailingBytes`). The no-panic + clean-Err contract is fully met as written, so
this is non-blocking. *Optional fold:* tighten the trailing-byte case to
`assert!(matches!(err, Error::TrailingBytes))` and the empty case to
`Error::UnexpectedEnd`, so the test pins the documented contract rather than
merely "some error." (I confirmed `decode_bytecode` does return exactly those:
`bytecode/decode.rs:46-48` → `TrailingBytes`, `read_u8`/`read_array:58-70` →
`UnexpectedEnd`.)

**N2 (md-codec, cosmetic) — KAT5's over-count branch adds a non-zero byte AND
+8 bits, conflating two effects.**
`tests/canonical_payload.rs:312-315` pushes `0xFF` and decodes at
`total_bits + 8`. This proves "a non-zero extra byte past the boundary isn't
absorbed," which is true and useful, but it changes both the buffer and the bit
count at once. The load-bearing demonstration is already carried cleanly by the
under-count branch (`total_bits - 1`, `total_bits - 8`, lines 308-309), which I
verified is the correct direction: the decoder's TLV-rollback tolerates only
≤7 **trailing-zero** bits (`tlv.rs:294-296`, `remaining_at_entry_start <= 7`),
so an over-count up to `bytes.len()*8` is genuinely absorbed (hence the
implementer's NOTE at lines 297-305 is accurate) and the under-count direction
is the right and sufficient proof. *No change required* — N2 is just an
observation that the over-count branch is supplementary, not the crux. Leaving
it in is fine (it's an extra true assertion).

---

## Detailed verification (what I checked, and the evidence)

### 1. Delegation correctness & purely-additive (both repos) — PASS

- **mk-codec** (`key_card.rs:108-110`, `:139`):
  `canonical_payload_bytes` → `crate::bytecode::encode_bytecode(self)`;
  `from_canonical_payload_bytes` → `crate::bytecode::decode_bytecode(bytes)`.
  Both delegation targets were **already `pub`** pre-P0 (verified against
  `46631c6`: `bytecode/encode.rs:23`, `bytecode/decode.rs:19`) and re-exported
  via `pub mod bytecode` + `pub use {encode,decode}_bytecode`
  (`bytecode/mod.rs:27-28`). So this is a convenience facade on existing public
  surface — **no new wire/ABI surface, no visibility widening**. Types match
  (`Result<Vec<u8>>` / `Result<KeyCard>`); error paths flow straight through.
- **md-codec** (`encode.rs:71-86`):
  `canonical_payload_bytes` → `encode_payload(self)` returning
  `(Vec<u8>, usize)`; `from_canonical_payload_bytes(&[u8], usize)` →
  `decode::decode_payload`. Both already `pub` pre-P0 (verified against `main`:
  `encode.rs:65`, `decode.rs:15`). Signatures and `total_bits` semantics match
  the underlying functions exactly. Purely additive.
- **Diff scope** confirms additive-only: mk = `key_card.rs` (+39) + new test
  (+224), nothing else; md = `encode.rs` (+34) + new test (+320), nothing else.
  **No `Cargo.toml`, no version line, no edit to any existing function body** in
  either repo.

### 2. KATs are adversarial, not vacuous — PASS

**mk1 cross-`chunk_set_id` determinism** (`canonical_payload.rs:154-203`):
- Uses `V5_explicit_path_4_components_with_fp`, which I confirmed is a genuine
  **3-string (multi-chunk)** clean vector carrying `canonical_bytecode_hex`
  (parsed `src/test_vectors/v0.1.json`: V5 = 3 strings, V7 = 3 strings; V1/V4 =
  2 strings — all four exist and are non-negative).
- Forces **two distinct framings** via `encode_with_chunk_set_id(card, 0x00AA)`
  vs `0xFF55`, asserts both are multi-chunk (`> 1`), asserts the mk1
  **strings DIFFER** (`assert_ne!`), then decodes each and asserts the recovered
  **payload is byte-identical** (`assert_eq!(payload_a, payload_b)`). This is
  exactly the (a)-differ + (b)-byte-identical structure required, on a genuinely
  multi-chunk card. **Non-vacuous and on-point for the I2 risk.**
- Why it's correct at the source level: `chunk_set_id` lives only in the
  string-layer header (`string_layer/header.rs`, `string_layer/chunk.rs:52`),
  drawn from the CSPRNG only in `encode()` (`key_card.rs:140-143`), and is
  *absent* from `encode_bytecode` (`bytecode/encode.rs:23-68` — no rand, no
  `chunk_set_id`). So the bytecode is structurally invariant to it.
- **Corpus-hex match is real:** `canonical_payload_matches_corpus_hex`
  (`:131-153`) decodes each pinned vector's mk1 strings and asserts
  `card.canonical_payload_bytes() == hex::decode(expected.canonical_bytecode_hex)`
  byte-for-byte, across all four representative vectors. Correct fixtures.

**md1 `total_bits`-is-load-bearing** (`canonical_payload.rs:262-316`, KAT5):
- The KAT **proves the precondition** that the fixture is bit-unaligned via
  `assert_ne!(bytes.len()*8, total_bits, "fixture must be bit-unaligned …")`
  (`:288-292`). This guard passed in the suite run (test green), so the
  load-bearing branches are actually reached — not skipped on a false
  precondition.
- The crux is the **under-count / truncation** direction
  (`assert_wrong_bits_not_silently_ok(d, bytes, total_bits-1)` and `-8`,
  `:308-309`). I verified this is the *correct and sufficient* demonstration:
  the decoder absorbs ≤7 trailing **zero** bits as codex32 padding
  (`tlv.rs:294-296`), so over-counting up to the byte boundary is silently
  absorbed (the implementer's NOTE at `:297-305` correctly documents this
  surprise), whereas truncating the declared count **drops real payload bits**,
  which can never be re-interpreted as trailing zero-padding → the decode must
  error or yield a different descriptor. The helper accepts either outcome
  (`Ok(other)` with `assert_ne!`, or `Err`) and forbids only a silent identical
  reproduction. **This is exactly why returning bytes-only would be unsafe**: a
  bytes-only API forces the consumer to assume `bytes.len()*8`, which on a
  bit-unaligned payload over-reads up to 7 real bits as padding → and for a
  payload whose true tail bits are non-zero, that is a lossy / wrong decode.
  Confirmed the demonstration is correct.

**md1 multi-0x02-TLV / canonicalization normalizer** (KAT3, `:179-228`):
- Builds a **non-canonical** descriptor (sortedmulti first-occurrence order
  `[2,0,1]`) and its canonical twin (`[0,1,2]`), encodes both, and asserts the
  payloads are **byte-identical** (`assert_eq!((bytes_nc,bits_nc),(bytes_c,bits_c))`)
  and both decode to the canonical descriptor. This proves canonical
  determinism (the I2 md1 TLV-ordering risk). KAT2 (`:158-177`) independently
  exercises the **multi-0x02-TLV wallet-policy** path (cell-7 wsh-2of3 with
  per-`@N` fingerprints + pubkeys) round-trip + re-encode-equals + repeat-call
  stability. Together they cover the wallet-policy multi-TLV shape and the
  normalizer property concretely. Non-vacuous.

**Both — malformed/empty/truncated rejected without panic** — PASS:
- mk: empty / 1-byte / 8-zero / trailing-byte all `.is_err()`
  (`:181-224`); the underlying decoder returns `UnexpectedEnd` /
  `TrailingBytes` (no `unwrap`/`panic` in the decode path; the package's
  `proptest_roundtrip` "decode_never_panics_*" suite — passing — independently
  fuzzes this). md: KAT5 feeds wrong bit counts and an extra `0xFF` byte; the
  decoder returns `Err` or a non-equal descriptor (no panic). md's
  `proptest_roundtrip` (10 cases, passing) also covers no-panic on arbitrary
  input.

### 3. Determinism / lossless-decode (source spot-check) — PASS

- **mk encode_bytecode** (`bytecode/encode.rs:23-68`): linear, no randomness, no
  unordered iteration — deterministic by construction. `decode_bytecode`
  (`decode.rs:19-56`) is an exact inverse and rejects trailing bytes.
- **md encode_payload** (`encode.rs:99-126`): canonicalizes then writes a fixed
  field order. `canonicalize_placeholder_indices` (`canonicalize.rs:168-…`)
  operates over **Vecs** with a first-occurrence scan + `sort_by_key` on indices
  (`:148`), and remapped sparse-TLV vectors are re-sorted ascending — **no
  `HashMap`/`HashSet` iteration in the production encode path** (grep over
  `encode.rs`/`tlv.rs`/`tree.rs` = none; the single `HashMap` in
  `canonicalize.rs:1076` is `#[cfg(test)]`). `decode_payload` (`decode.rs:15-79`)
  runs a fixed validator order bounded by `bit_limit`, tolerating only ≤7
  trailing-zero pad bits. **No non-deterministic path; decode is lossless w.r.t.
  the (bytes,total_bits) payload.**

### 4. NO-BUMP correctness — PASS

Confirmed by diff: additive `pub fn`s only, delegating to pre-existing public
functions; **no Cargo.toml change, no version change, no wire/behavior change**
in either repo. Correctly NO-BUMP.

---

## Suite results

Ran the **full package** suites (per project rule — not targeted), on the
respective feat branches:

- **mk-codec** (`cargo test -p mk-codec`): **ALL GREEN.**
  lib unittests 157; `bch_adversarial` 4; **`canonical_payload` 4**
  (`from_canonical_payload_bytes_rejects_garbage`,
  `canonical_payload_is_chunk_set_id_invariant`,
  `canonical_payload_matches_corpus_hex`, `canonical_payload_round_trips`);
  `error_coverage` 2; `indel_reject_contract` 2; `proptest_roundtrip` 4;
  `round_trip` 3; `vectors` 3; doctests 0. **= 176 passed, 0 failed, 0 ignored.**
  `cargo clippy -p mk-codec --all-targets`: clean (no warnings).
  `cargo fmt -p mk-codec -- --check`: clean (exit 0).

- **md-codec** (`cargo test -p md-codec`): **ALL GREEN.**
  lib unittests 225; address_derivation 21; bch_adversarial 13; bch_decode 11;
  bch_visibility_pin 1; bip341_wallet_vectors 9; bitcoind_differential 0 (1
  ignored — env-gated, expected); **`canonical_payload` 5**; chunking 9;
  display_grouping_conformance 1; forward_compat 1; indel_reject_contract 4;
  mixed_case_reject 8; parity_smoke 1; per_key_use_site_override 14;
  proptest_roundtrip 10; proptest_to_miniscript 49; smoke 8; wallet_policy 21;
  doctests 0. **= 419 passed, 0 failed, 1 ignored (expected).**
  `cargo clippy -p md-codec --all-targets`: clean (no warnings).
  `cargo fmt -p md-codec -- --check`: clean (exit 0).

---

## P6 forward-flag (not a P0 blocker)

**CONFIRMED REAL — the plan's P6 KAT must be reframed.** The plan's §7 P6
states "m*1 → word-card → recover → m*1 byte-identical." For **mk1 this literal
string round-trip is impossible**: `encode()` draws a fresh 20-bit
`chunk_set_id` from the CSPRNG (`key_card.rs:140-143`) and embeds it in the
chunked-string header (`string_layer/header.rs`, `chunk.rs:52-87`), so two
encodes of the same card produce **different mk1 strings** — exactly the
property KAT3 (`canonical_payload_is_chunk_set_id_invariant`) relies on. Only
the **bytecode/xpub identity** round-trips, not the literal mk1 string. (md1 has
no such randomness and *can* round-trip its string, per
`encode_md1_string`/`decode_md1_string` — so the asymmetry is mk1-specific.)
**Fold for P6 (when P6 is authored):** reframe the mk1 leg of the P6 KAT to
assert **canonical-payload / xpub equality** (or fix the `chunk_set_id` via
`encode_with_chunk_set_id`), not byte-identical mk1 strings. Single-chunk mk1
cards have no `chunk_set_id` on the wire and *may* round-trip the string, but
multi-chunk cards never will — so the KAT must not assume string identity in
general. This is a P6 spec correction, not a P0 defect.

---

## Bottom line

P0 is **GREEN (0C/0I)**. The accessors are correct additive facades over
already-public targets; NO-BUMP is correct; determinism holds at the source
level (no randomness, no unordered iteration in either encode path); the
I2-risk KATs are genuinely adversarial and verified non-vacuous; both full
suites + clippy + fmt are clean. The two Nits are optional polish (assert exact
error variants; the over-count branch is supplementary). The P6 mk1
string-identity impossibility is real and flagged for reframing. **Cleared to
merge P0 and start P1.**
