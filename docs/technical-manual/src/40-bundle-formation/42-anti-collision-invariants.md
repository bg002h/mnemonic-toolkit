# Anti-Collision Invariants

Bundle integrity rests on five invariants that police whether a set of recovered cards is *the same bundle* or fragments from different wallets pretending to be one. This chapter walks each invariant against its HEAD implementation: the shared-`chunk_set_id` prefix, the multiset `md1_xpub_match` rule, the four-case ms1 short-circuit table, the mk1 cosigner-mapping diagnostic, and BIP-388 distinct-key enforcement.

All five fire during `mnemonic verify-bundle`; only the BIP-388 rule additionally fires at bundle creation time (`mnemonic bundle`). The verify-bundle dispatch entry is `cmd::verify_bundle::run` at `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run`; the multisig per-cosigner emission core is `emit_multisig_checks` at `verify_bundle.rs::emit_multisig_checks`; the BIP-388 distinctness checks live at `mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs::check_key_vector_distinctness` (descriptor layer) and `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs::check_resolved_slots_distinctness` (template-mode CLI layer) — both now typed `DerivationPath` equality. Subsequent references to these four files within this chapter use bare filenames.

## Invariant 1 — Shared `chunk_set_id` prefix

\index{chunk\_set\_id binding}The `chunk_set_id` printed on the engraving card (and recoverable from any mk1 string via `chunk_set_id_extract` at `format.rs::chunk_set_id_extract`) is the **bundle-level binding key**. Every ms1, mk1, and md1 card produced from the same bundle synthesis derives its identifier from the same wallet `policy_id`:

| Card | Bits | Hex chars | Source | Code site |
|---|---|---|---|---|
| md1 | 16 | 4 | `policy_id[0..2]` | `bundle.rs::build_unified_card` |
| ms1, mk1 (single-sig) | 20 | 5 | `derive_mk1_chunk_set_id(policy_id[0..4])` packed as `((b0 << 12) \| (b1 << 4) \| (b2 >> 4))` | `synthesize.rs::derive_mk1_chunk_set_id` formats via `bundle.rs::build_unified_card` |
| ms1, mk1 (multisig, per cosigner) | 20 | 5 | `derive_mk1_chunk_set_id(policy_id[0..4]) ^ slot_index` | `synthesize.rs::derive_mk1_chunk_set_id_for_slot` formats via `bundle.rs::build_unified_card` |

The 4-byte stub passed into `derive_mk1_chunk_set_id` is the first 4 bytes of the SHA-256-truncated `policy_id` (§II.1). Both formats agree on their leading 16 bits because md1 takes exactly those 16 bits and the mk1/ms1 20-bit packing places `policy_id[0]` in bits 19..12 and `policy_id[1]` in bits 11..4. The fifth hex char of the mk1/ms1 identifier is the upper nibble of `policy_id[2]` (single-sig); for **multisig** it is `(policy_id[2] >> 4) XOR slot_index`. The slot index (a cosigner is ≤ 16, so 0..15 = exactly 4 bits) XORs into bits 3..0 only — the **leading 16 bits are unchanged**, so every cosigner's mk1 csi still shares the bundle-binding prefix while being distinct per cosigner (audit I10: a slot-unique csi is required so `verify-bundle` can group each cosigner's chunks; the prior per-fingerprint scheme collided same-xpub-different-path cosigners). md1 does not encode the fifth nibble in its on-card identifier.

\index{cross-card binding (bundle)}**The binding rule:** cards from the same bundle share at minimum their leading 16 `chunk_set_id` bits. A reader who finds an mk1 card with `chunk_set_id` `1c017` and an md1 card with `chunk_set_id` `1c01` can reasonably conjecture they belong together; the full verification then runs `verify-bundle`. Cards whose leading 16 bits disagree are definitely from different bundles. (False positives at the 16-bit prefix happen with probability ≈ 2⁻¹⁶ ≈ 1 in 65,536 under random `policy_id`s; the substantive verification — the next four invariants — is what *certifies* bundle membership.) In a **multisig** bundle the per-cosigner mk1 cards all share that leading-16-bit prefix (and the md1 prefix) but differ in the fifth hex char by their slot index — so `1c010`, `1c011`, … denote cosigners 0, 1, … of the same wallet. (This concerns only the mk1/ms1 *display* and chunk-grouping; whether md1's own on-wire prefix matches mk1's is governed separately by the `policy_id` derivation each uses — see the `anti-collision-16bit-invariant` note in `design/FOLLOWUPS.md`.)

§II.1's `chunk_set_id` definition (a 20-bit identifier on the wire derived from `policy_id` for chunked headers) is the foundation; this chapter is about the cross-card binding *use* of that identifier.

## Invariant 2 — Multiset `md1_xpub_match` (sort-then-compare)

\index{md1\_xpub\_match}\index{multiset semantics}The `md1_xpub_match` check (`verify_bundle.rs::emit_multisig_checks`) asserts that the **multiset** of pubkeys in the supplied md1's `Tag::Pubkeys = 0x02` TLV equals the multiset of pubkeys in the expected md1's same TLV. Implementation:

```rust
let exp_pubs: Vec<[u8; 65]> = expected_md_decoded.tlv.pubkeys ...
let act_pubs: Vec<[u8; 65]> = desc.tlv.pubkeys ...
let mut exp_sorted = exp_pubs.clone();
let mut act_sorted = act_pubs.clone();
exp_sorted.sort();
act_sorted.sort();
let pubkeys_match = exp_sorted == act_sorted;
```

\index{multiplicity (multiset)}Three pieces of pedantry matter here:

- **Set equality, not order equality.** A `wsh(multi(2,@0,@1,@2))` template with cosigners written in slot-index order vs. xpub-sort order would otherwise produce two distinct bundles with the same wallet semantics. Sort-then-compare neutralizes that.
- **Multiplicity matters.** A degenerate `wsh(multi(K,@0,@0))` (same key twice) would compare equal to `wsh(multi(K,@0,@1))` under plain set semantics. The sorted-Vec equality preserves multiplicity: two copies of pubkey *X* in `exp_pubs` require two copies in `act_pubs`. SPEC v0.5 §5.7 line 138 makes this normative.
- **65-byte form, not 33-byte form.** The pubkeys-TLV stores the md1 65-byte form (`chain_code || compressed_pubkey`, `synthesize.rs::xpub_to_65`). Two xpubs with the same compressed pubkey but different chain codes are *distinct* under this comparison — which is the correct behavior, since a BIP-32 derivation step depends on the chain code as well as the parent pubkey.

The check fails with `passed: false` and populated forensic fields (`expected` and `actual` set to comma-joined hex; `diff_byte_offset` set to first-differ index). The `detail` text reads `"md1 pubkeys differ from expected set"`.

Single-sig (N=1) uses a separate path: `emit_md1_checks` (`verify_bundle.rs::emit_md1_checks`) compares only the *first* pubkey via `.first()` rather than the full sorted multiset — there is only one cosigner, so multiplicity is vacuous. Its success detail reads `"65-byte xpub matches expected"` (`verify_bundle.rs::emit_md1_checks`); failure detail reads `"md1 xpub differs from expected"`. The multiset semantics described above apply to the multisig path (`emit_multisig_checks`, `verify_bundle.rs::emit_multisig_checks`) only.

## Invariant 3 — Four-case ms1 short-circuit table

\index{ms1 four-case table}Per-cosigner ms1 checks divide into four mutually-exclusive cases per SPEC v0.5 §5.7 (`verify_bundle.rs::emit_multisig_checks`). The check emits *exactly two* rows per slot: `ms1_decode[i]` and `ms1_entropy_match[i]`. The case-split discriminator is `expected.ms1[i].is_empty()` (watch-only sentinel) combined with whether `supplied.ms1[i]` is present and whether it decodes:

| Case | `expected.ms1[i]` | `supplied.ms1[i]` | `ms_codec::decode(supplied)` | `ms1_decode[i]` | `ms1_entropy_match[i]` |
|---|---|---|---|---|---|
| 1 | `""` (watch-only) | any | any | `passed: true`, `decode_error: "skipped: watch-only slot"` | `passed: true`, `decode_error: "skipped: watch-only slot"` |
| 2 | non-empty | non-empty | `Ok(...)` | `passed: true` | `passed: true` if byte-equal; else `false` + forensic fields |
| 3 | non-empty | non-empty | `Err(e)` | `passed: false`, `decode_error: <e>`† | `passed: true`, `decode_error: "skipped: ms1 decode failed"` |
| 4 | non-empty | empty / missing | n/a | `passed: false`, `decode_error: "error: ms1[{i}] expected (full-mode bundle) but not supplied"` | `passed: false`, `decode_error: "skipped: ms1[{i}] not supplied"` |

†Rows marked `<e>` or `<mk_codec error message>` carry the `format!("{:?}", e)` Debug representation of the underlying codec error (`verify_bundle.rs::emit_multisig_checks`, `verify_bundle.rs::emit_multisig_checks`), not the `Display` form.

\index{cascade-skip}Three principles to internalize:

- **Case 1 is silent absorption.** Supplying `--ms1 ms1...` to a slot whose expected ms1 is empty is *not* an error — the table treats it as a noop. This is essential for hybrid multisig: a user re-running `verify-bundle` after stamping with the original CLI invocation may pass all ms1 strings for all slots; only the secret-bearing slots are checked.
- **Case 3 cascades.** A malformed ms1 fails `ms1_decode[i]` but emits `ms1_entropy_match[i]` with `passed: true, decode_error: "skipped: ms1 decode failed"`. This is **vacuous-skip semantics**: the dependent check has no oracle to evaluate against, so it cannot fail; the diagnostic is the absent decode, not a phantom byte-mismatch.
- **Case 4 is the absent-secret signal.** If the bundle was created as full-mode (`expected.ms1[i]` was synthesized as a real BIP-39 entropy ms1 string) but the user forgot to supply that slot's `--ms1`, both checks fail. The decode-error text contains the slot index `{i}` (verbatim curly-brace substitution from `format!`) so a reader can spot the missing slot directly.

Single-sig (N=1) uses an analogous but simpler path in `emit_verify_checks` (`verify_bundle.rs::emit_verify_checks`), discriminating via `expected.ms1.first().map(|s| s.is_empty())` since there is only one slot.

`wif` slots are treated as watch-only for ms1 purposes per SPEC §5.7 line 145 — the ms1 check pair short-circuits per case 1 because the bundle synthesis writes `""` into `expected.ms1[i]` for wif slots (the wif's secret material lives in the mk1 origin metadata; ms1 has no role).

## Invariant 4 — mk1 cosigner-mapping diagnostic

\index{cosigner-mapping diagnostic}When a multisig verify-bundle invocation fails to attach a supplied `--mk1` group to a cosigner slot, the helper distinguishes three failure modes (`verify_bundle.rs::MappingFailure`):

```rust
enum MappingFailure {
    NotSupplied,
    DecodeFailed(String),
    XpubNotInPolicy,
}
```

Each surfaces as a distinct `mk1_decode[i]` `detail` and `decode_error` (`verify_bundle.rs::emit_multisig_checks`):

| `MappingFailure` | `mk1_decode[i]` `passed` | `detail` (cosigner `i`) | `decode_error` |
|---|---|---|---|
| `NotSupplied` | `false` | `cosigner[i] mk1 not supplied` | `skipped: mk1[{i}] not supplied` |
| `DecodeFailed(msg)` | `false` | `cosigner[i] mk1 decode failed` | `<mk_codec error message>`† |
| `XpubNotInPolicy` | `false` | `cosigner[i] supplied mk1 card xpub absent from descriptor policy` | `supplied mk1 card xpub absent from descriptor policy` |

\index{XpubNotInPolicy}Each diagnostic encodes a different incident type. `NotSupplied` is a recoverable user error (forgot to type a `--mk1` flag). `DecodeFailed` is a card-stamping or transcription error (BCH checksum doesn't validate, malformed envelope, etc.). `XpubNotInPolicy` is the **wrong-key attack indicator** — a supplied mk1 card decoded cleanly but its xpub is absent from the descriptor's `tlv.pubkeys` set. That is the signature of an attacker substituting an attacker-controlled mk1 card into the user's bundle, OR of the user supplying an mk1 card from a different wallet by mistake.

**Precedence: `XpubNotInPolicy > DecodeFailed > NotSupplied`** (`verify_bundle.rs::emit_multisig_checks`, enforced by the two-pass algorithm at `verify_bundle.rs::emit_multisig_checks`). The first pass attempts xpub-based mapping; surplus successfully-decoded cards with no matching slot promote a `NotSupplied` slot to `XpubNotInPolicy`. The second pass assigns `DecodeFailed` to remaining unfilled slots. The ordering matters because a single forensic message should describe the most-actionable failure: an `XpubNotInPolicy` finding tells the user "this card is from a different wallet" — strictly more diagnostic than "you forgot a card."

The three dependent checks (`mk1_xpub_match[i]` / `mk1_fingerprint_match[i]` / `mk1_path_match[i]`) cascade-skip with `passed: true, decode_error: "skipped: mk1[{i}] decode failed"` (`verify_bundle.rs::emit_multisig_checks`) — vacuous-skip because no oracle is available.

## Invariant 5 — BIP-388 distinct-key enforcement

\index{BIP-388 distinct-key}\index{distinct-key rule}BIP-388 §"Specification" requires that the key-information vector contain **distinct** entries — two `@N` slots resolving to the same `(xpub, derivation_path)` tuple is forbidden. The toolkit enforces this symmetrically across bundle creation (SPEC §4.11.b, exit code 2) and verify-bundle (SPEC §4.11.c, exit code 4).

\index{hardened apostrophe folding}\index{h-notation}The **normalization domain** (SPEC v0.5 §4.11.b) is **typed `DerivationPath` equality** via `bitcoin::bip32::DerivationPath`'s parse-then-compare. The typed form folds `h`-notation into `'`-notation, so `48h/0h/0h/2h` and `48'/0'/0'/2'` compare EQUAL and produce a collision. This is the **v0.4→v0.5 deliberate reversal**: v0.4 used raw-string equality, v0.5 reversed to typed equality because the SPEC reasoned that `h` and `'` are syntactic sugar for the same hardened-bit encoding and bundles distinguished only by that notation are de-facto identical.

The typed check is `check_key_vector_distinctness` at `parse_descriptor.rs::check_key_vector_distinctness`:

```rust
for i in 0..cs.len() {
    for j in (i + 1)..cs.len() {
        if cs[i].xpub.to_string() == cs[j].xpub.to_string() && cs[i].path == cs[j].path {
            return Err(ToolkitError::Bip388Distinctness { i, j });
        }
    }
}
```

`cs[i].path: DerivationPath` compares via the typed `PartialEq` derived for `DerivationPath`, which is what folds `h` ↔ `'`.

### Both layers are typed: no bifurcation

\index{bifurcation (BIP-388 enforcement)}Template-mode bundle synthesis (where the user supplies `--template <name>` + per-slot subkeys) goes through `check_resolved_slots_distinctness` at `bundle.rs::check_resolved_slots_distinctness`, which compares **`slots[i].xpub.to_string() == slots[j].xpub.to_string() && slots[i].path == slots[j].path`** — the **typed** `DerivationPath`, exactly like the descriptor-layer `check_key_vector_distinctness` (`parse_descriptor.rs::check_key_vector_distinctness`). `h`/`'`-notation folds in **both** layers, so `48h/0h/0h/2h` and `48'/0'/0'/2'` (with the same xpub) collide at synthesis **and** at verify-bundle. There is no raw-string-vs-typed asymmetry.

This convergence is the result of the v0.37.9 path unification (`SPEC_path_raw_bracketed_bare_unification.md` A2): the former `ResolvedSlot.path_raw` raw-string field was **deleted**, leaving the typed `path` (plus the `origin_path_bare()` / `bracketed_origin()` accessors) as the only path representation. The `bundle.rs::check_resolved_slots_distinctness` doc-comment was updated to the typed framing (v0.5 §4.11.b deliberate reversal) at the same time, and the `error.rs::ToolkitError::Bip388Distinctness` variant doc-comment was likewise resynced to the typed `(xpub.to_string(), path)` `DerivationPath` framing. Both the runtime behavior at both layers and the source doc-comments are now typed; no raw-string lag remains.

### Error surfacing

Bundle creation collision (exit 2): byte-exact stderr `error: BIP-388 distinct-key violation: slot @{i} and slot @{j} resolve to identical (xpub, path)` (`error.rs::ToolkitError::message`).

Verify-bundle collision (exit 4): byte-exact stderr `error: bundle violates BIP-388 distinct-key rule; regenerate with distinct keys` (`error.rs::ToolkitError::message`). The verify-bundle path re-wraps the typed-check failure into `Bip388VerifyDistinctness` (`verify_bundle.rs::descriptor_mode_verify_run`) so the exit code and stderr text differ from the creation-time variant.

## Worked example — a colliding bundle

The simplest BIP-388 collision: a 2-of-2 multisig where both slots are the same wallet, supplied as two `@N.phrase=...` slots with identical phrases. Both resolve to the same `(xpub, path)` pair, the distinctness check fires, and synthesis aborts before any cards are emitted.

The full invocation, stderr, and exit code are captured at `transcripts/mnemonic-bundle-bip388-collision.cmd` / `.out`. Re-running via `tests/verify-examples.sh` produces the byte-exact one-line error and exit 2.

```text
error: BIP-388 distinct-key violation: slot @0 and slot @1 resolve to identical (xpub, path)
```

The diagnostic identifies the two colliding slot indices (`@0` and `@1`); for an N>2 bundle, only the first colliding pair is reported (the pairwise scan returns at the first collision rather than enumerating all of them).

## Source pointers

- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` — `emit_multisig_checks` (4-case ms1, cosigner-mapping diagnostic, multiset md1_xpub_match).
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::MappingFailure` — `MappingFailure` enum.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` — two-pass mapping algorithm enforcing precedence.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` — multiset `md1_xpub_match` (sort-then-compare).
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs::check_key_vector_distinctness` — typed `check_key_vector_distinctness`.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs::check_resolved_slots_distinctness` — typed-`DerivationPath` `check_resolved_slots_distinctness`; doc-comment updated (v0.5 §4.11.b).
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs::build_unified_card` — md1 4-hex `chunk_set_id` format string.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs::build_unified_card` — mk1/ms1 5-hex `chunk_set_id` format string.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs::derive_mk1_chunk_set_id` — `derive_mk1_chunk_set_id` packing.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs::derive_mk1_chunk_set_id_for_slot` — slot-unique multisig csi (`base ^ slot_index`; audit I10).
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/error.rs::ToolkitError::Bip388Distinctness` — `Bip388Distinctness` / `Bip388VerifyDistinctness` variants (exit-code mapping at `error.rs::ToolkitError::exit_code`). The `Bip388Distinctness` variant doc-comment at `error.rs::ToolkitError::Bip388Distinctness` carries the typed `(xpub.to_string(), path)` `DerivationPath`-equality framing (v0.5 §4.11.b), resynced in lockstep with the `bundle.rs` doc-comment.
- BIP-388 §"Specification" — wallet-policy template + distinct key-information vector requirement.
- Toolkit SPEC v0.5 §4.11.b — typed `DerivationPath` equality (the deliberate v0.4 → v0.5 reversal). §5.7 — multiset `md1_xpub_match` + four-case ms1 table + mk1 cosigner-mapping diagnostic. §6.6 row 13 — `Bip388Distinctness` exit-2 row.
