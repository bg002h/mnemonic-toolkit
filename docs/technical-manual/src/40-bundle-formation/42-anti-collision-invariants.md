# Anti-Collision Invariants

Bundle integrity rests on five invariants that police whether a set of recovered cards is *the same bundle* or fragments from different wallets pretending to be one. This chapter walks each invariant against its HEAD implementation: the shared-`chunk_set_id` prefix, the multiset `md1_xpub_match` rule, the four-case ms1 short-circuit table, the mk1 cosigner-mapping diagnostic, and BIP-388 distinct-key enforcement.

All five fire during `mnemonic verify-bundle`; only the BIP-388 rule additionally fires at bundle creation time (`mnemonic bundle`). The verify-bundle dispatch entry is `cmd::verify_bundle::run` at `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:98`; the multisig per-cosigner emission core is `emit_multisig_checks` at `:838-1277`; the BIP-388 distinctness checks live at `mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs:1104-1117` (typed) and `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs:261-275` (raw-string template path). Subsequent references to these four files within this chapter use bare filenames.

## Invariant 1 — Shared `chunk_set_id` prefix

\index{chunk\_set\_id binding}The `chunk_set_id` printed on the engraving card (and recoverable from any mk1 string via `chunk_set_id_extract` at `format.rs:379-395`) is the **bundle-level binding key**. Every ms1, mk1, and md1 card produced from the same bundle synthesis derives its identifier from the same wallet `policy_id`:

| Card | Bits | Hex chars | Source | Code site |
|---|---|---|---|---|
| md1 | 16 | 4 | `policy_id[0..2]` | `bundle.rs:707` |
| ms1, mk1 | 20 | 5 | `derive_mk1_chunk_set_id(policy_id[0..4])` packed as `((b0 << 12) \| (b1 << 4) \| (b2 >> 4))` | `synthesize.rs:42-44` formats via `bundle.rs:724` |

The 4-byte stub passed into `derive_mk1_chunk_set_id` is the first 4 bytes of the SHA-256-truncated `policy_id` (§II.1). Both formats agree on their leading 16 bits because md1 takes exactly those 16 bits and the mk1/ms1 20-bit packing places `policy_id[0]` in bits 19..12 and `policy_id[1]` in bits 11..4. The fifth hex char of the mk1/ms1 identifier is the upper nibble of `policy_id[2]`; md1 does not encode that nibble in its on-card identifier.

\index{cross-card binding (bundle)}**The binding rule:** cards from the same bundle share at minimum their leading 16 `chunk_set_id` bits. A reader who finds an mk1 card with `chunk_set_id` `1c017` and an md1 card with `chunk_set_id` `1c01` can reasonably conjecture they belong together; the full verification then runs `verify-bundle`. Cards whose leading 16 bits disagree are definitely from different bundles. (False positives at the 16-bit prefix happen with probability ≈ 2⁻¹⁶ ≈ 1 in 65,536 under random `policy_id`s; the substantive verification — the next four invariants — is what *certifies* bundle membership.)

§II.1's `chunk_set_id` definition (a 20-bit identifier on the wire derived from `policy_id` for chunked headers) is the foundation; this chapter is about the cross-card binding *use* of that identifier.

## Invariant 2 — Multiset `md1_xpub_match` (sort-then-compare)

\index{md1\_xpub\_match}\index{multiset semantics}The `md1_xpub_match` check (`verify_bundle.rs:1194-1232`) asserts that the **multiset** of pubkeys in the supplied md1's `Tag::Pubkeys = 0x02` TLV equals the multiset of pubkeys in the expected md1's same TLV. Implementation:

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
- **65-byte form, not 33-byte form.** The pubkeys-TLV stores the md1 65-byte form (`chain_code || compressed_pubkey`, `synthesize.rs:69-74`). Two xpubs with the same compressed pubkey but different chain codes are *distinct* under this comparison — which is the correct behavior, since a BIP-32 derivation step depends on the chain code as well as the parent pubkey.

The check fails with `passed: false` and populated forensic fields (`expected` and `actual` set to comma-joined hex; `diff_byte_offset` set to first-differ index). The `detail` text reads `"md1 pubkeys differ from expected set"`.

Single-sig (N=1) uses a separate path: `emit_md1_checks` (`verify_bundle.rs:1280-1355`) compares only the *first* pubkey via `.first()` rather than the full sorted multiset — there is only one cosigner, so multiplicity is vacuous. Its success detail reads `"65-byte xpub matches expected"` (`verify_bundle.rs:1323`); failure detail reads `"md1 xpub differs from expected"`. The multiset semantics described above apply to the multisig path (`emit_multisig_checks`, `verify_bundle.rs:1194`) only.

## Invariant 3 — Four-case ms1 short-circuit table

\index{ms1 four-case table}Per-cosigner ms1 checks divide into four mutually-exclusive cases per SPEC v0.5 §5.7 (`verify_bundle.rs:956-1040`). The check emits *exactly two* rows per slot: `ms1_decode[i]` and `ms1_entropy_match[i]`. The case-split discriminator is `expected.ms1[i].is_empty()` (watch-only sentinel) combined with whether `supplied.ms1[i]` is present and whether it decodes:

| Case | `expected.ms1[i]` | `supplied.ms1[i]` | `ms_codec::decode(supplied)` | `ms1_decode[i]` | `ms1_entropy_match[i]` |
|---|---|---|---|---|---|
| 1 | `""` (watch-only) | any | any | `passed: true`, `decode_error: "skipped: watch-only slot"` | `passed: true`, `decode_error: "skipped: watch-only slot"` |
| 2 | non-empty | non-empty | `Ok(...)` | `passed: true` | `passed: true` if byte-equal; else `false` + forensic fields |
| 3 | non-empty | non-empty | `Err(e)` | `passed: false`, `decode_error: <e>`† | `passed: true`, `decode_error: "skipped: ms1 decode failed"` |
| 4 | non-empty | empty / missing | n/a | `passed: false`, `decode_error: "error: ms1[{i}] expected (full-mode bundle) but not supplied"` | `passed: false`, `decode_error: "skipped: ms1[{i}] not supplied"` |

†Rows marked `<e>` or `<mk_codec error message>` carry the `format!("{:?}", e)` Debug representation of the underlying codec error (`verify_bundle.rs:1004`, `:877`), not the `Display` form.

\index{cascade-skip}Three principles to internalize:

- **Case 1 is silent absorption.** Supplying `--ms1 ms1...` to a slot whose expected ms1 is empty is *not* an error — the table treats it as a noop. This is essential for hybrid multisig: a user re-running `verify-bundle` after stamping with the original CLI invocation may pass all ms1 strings for all slots; only the secret-bearing slots are checked.
- **Case 3 cascades.** A malformed ms1 fails `ms1_decode[i]` but emits `ms1_entropy_match[i]` with `passed: true, decode_error: "skipped: ms1 decode failed"`. This is **vacuous-skip semantics**: the dependent check has no oracle to evaluate against, so it cannot fail; the diagnostic is the absent decode, not a phantom byte-mismatch.
- **Case 4 is the absent-secret signal.** If the bundle was created as full-mode (`expected.ms1[i]` was synthesized as a real BIP-39 entropy ms1 string) but the user forgot to supply that slot's `--ms1`, both checks fail. The decode-error text contains the slot index `{i}` (verbatim curly-brace substitution from `format!`) so a reader can spot the missing slot directly.

Single-sig (N=1) uses an analogous but simpler path in `emit_verify_checks` (`verify_bundle.rs:620-687`), discriminating via `expected.ms1.first().map(|s| s.is_empty())` since there is only one slot.

`wif` slots are treated as watch-only for ms1 purposes per SPEC §5.7 line 145 — the ms1 check pair short-circuits per case 1 because the bundle synthesis writes `""` into `expected.ms1[i]` for wif slots (the wif's secret material lives in the mk1 origin metadata; ms1 has no role).

## Invariant 4 — mk1 cosigner-mapping diagnostic

\index{cosigner-mapping diagnostic}When a multisig verify-bundle invocation fails to attach a supplied `--mk1` group to a cosigner slot, the helper distinguishes three failure modes (`verify_bundle.rs:831-836`):

```rust
enum MappingFailure {
    NotSupplied,
    DecodeFailed(String),
    XpubNotInPolicy,
}
```

Each surfaces as a distinct `mk1_decode[i]` `detail` and `decode_error` (`verify_bundle.rs:1117-1149`):

| `MappingFailure` | `mk1_decode[i]` `passed` | `detail` (cosigner `i`) | `decode_error` |
|---|---|---|---|
| `NotSupplied` | `false` | `cosigner[i] mk1 not supplied` | `skipped: mk1[{i}] not supplied` |
| `DecodeFailed(msg)` | `false` | `cosigner[i] mk1 decode failed` | `<mk_codec error message>`† |
| `XpubNotInPolicy` | `false` | `cosigner[i] supplied mk1 card xpub absent from descriptor policy` | `supplied mk1 card xpub absent from descriptor policy` |

\index{XpubNotInPolicy}Each diagnostic encodes a different incident type. `NotSupplied` is a recoverable user error (forgot to type a `--mk1` flag). `DecodeFailed` is a card-stamping or transcription error (BCH checksum doesn't validate, malformed envelope, etc.). `XpubNotInPolicy` is the **wrong-key attack indicator** — a supplied mk1 card decoded cleanly but its xpub is absent from the descriptor's `tlv.pubkeys` set. That is the signature of an attacker substituting an attacker-controlled mk1 card into the user's bundle, OR of the user supplying an mk1 card from a different wallet by mistake.

**Precedence: `XpubNotInPolicy > DecodeFailed > NotSupplied`** (`verify_bundle.rs:830`, enforced by the two-pass algorithm at `:895-947`). The first pass attempts xpub-based mapping; surplus successfully-decoded cards with no matching slot promote a `NotSupplied` slot to `XpubNotInPolicy`. The second pass assigns `DecodeFailed` to remaining unfilled slots. The ordering matters because a single forensic message should describe the most-actionable failure: an `XpubNotInPolicy` finding tells the user "this card is from a different wallet" — strictly more diagnostic than "you forgot a card."

The three dependent checks (`mk1_xpub_match[i]` / `mk1_fingerprint_match[i]` / `mk1_path_match[i]`) cascade-skip with `passed: true, decode_error: "skipped: mk1[{i}] decode failed"` (`:1141-1149`) — vacuous-skip because no oracle is available.

## Invariant 5 — BIP-388 distinct-key enforcement

\index{BIP-388 distinct-key}\index{distinct-key rule}BIP-388 §"Specification" requires that the key-information vector contain **distinct** entries — two `@N` slots resolving to the same `(xpub, derivation_path)` tuple is forbidden. The toolkit enforces this symmetrically across bundle creation (SPEC §4.11.b, exit code 2) and verify-bundle (SPEC §4.11.c, exit code 4).

\index{hardened apostrophe folding}\index{h-notation}The **normalization domain** (SPEC v0.5 §4.11.b) is **typed `DerivationPath` equality** via `bitcoin::bip32::DerivationPath`'s parse-then-compare. The typed form folds `h`-notation into `'`-notation, so `48h/0h/0h/2h` and `48'/0'/0'/2'` compare EQUAL and produce a collision. This is the **v0.4→v0.5 deliberate reversal**: v0.4 used raw-string equality, v0.5 reversed to typed equality because the SPEC reasoned that `h` and `'` are syntactic sugar for the same hardened-bit encoding and bundles distinguished only by that notation are de-facto identical.

The typed check is `check_key_vector_distinctness` at `parse_descriptor.rs:1104-1117`:

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

### A bifurcation: template-mode bundle synthesis uses raw-string equality

\index{bifurcation (BIP-388 enforcement)}Template-mode bundle synthesis (where the user supplies `--template <name>` + per-slot subkeys) goes through `check_resolved_slots_distinctness` at `bundle.rs:261-275`, which compares **`(xpub.to_string(), path_raw)`** — the user-supplied raw path string, not the parsed `DerivationPath`. The doc-comment at `bundle.rs:259-260` was written under v0.4 SPEC (raw-string) and has not been resynced to v0.5's typed-equality reversal.

Practical consequence: under template-mode synthesis, two slots with `path_raw = "48h/0h/0h/2h"` and `path_raw = "48'/0'/0'/2'"` (and the same xpub) would *not* collide at synthesis (raw strings differ) but *would* collide at any subsequent verify-bundle (typed equality folds). For **phrase / entropy** slots this is unreachable: `path` and `path_raw` are computed from `--template` + `--account` + family by `template.rs`, so `path_raw` always comes out as the canonical `'`-notation string. For **xpub** slots, however, the bifurcation is live: template-mode accepts user-supplied `--slot @N.path=...` per `bundle.rs:355-363`, which preserves the raw user string into `path_raw`. A user who supplied the same xpub with `h`-notation in one slot and `'`-notation in another would create a bundle whose `check_resolved_slots_distinctness` does not fire (raw strings differ) but whose `verify-bundle` does (typed equality folds). That edge case is the live scope of the bifurcation. Descriptor-mode bundle synthesis (where the user supplies `--descriptor`) calls the typed-equality check (`bundle.rs:982`), so the asymmetry is confined to template-mode xpub-slot paths.

### Error surfacing

Bundle creation collision (exit 2): byte-exact stderr `error: BIP-388 distinct-key violation: slot @{i} and slot @{j} resolve to identical (xpub, path)` (`error.rs:325`).

Verify-bundle collision (exit 4): byte-exact stderr `error: bundle violates BIP-388 distinct-key rule; regenerate with distinct keys` (`error.rs:328`). The verify-bundle path re-wraps the typed-check failure into `Bip388VerifyDistinctness` (`verify_bundle.rs:470-471`) so the exit code and stderr text differ from the creation-time variant.

## Worked example — a colliding bundle

The simplest BIP-388 collision: a 2-of-2 multisig where both slots are the same wallet, supplied as two `@N.phrase=...` slots with identical phrases. Both resolve to the same `(xpub, path_raw)` pair, the distinctness check fires, and synthesis aborts before any cards are emitted.

The full invocation, stderr, and exit code are captured at `transcripts/mnemonic-bundle-bip388-collision.cmd` / `.out`. Re-running via `tests/verify-examples.sh` produces the byte-exact one-line error and exit 2.

```text
error: BIP-388 distinct-key violation: slot @0 and slot @1 resolve to identical (xpub, path)
```

The diagnostic identifies the two colliding slot indices (`@0` and `@1`); for an N>2 bundle, only the first colliding pair is reported (the pairwise scan returns at the first collision rather than enumerating all of them).

## Source pointers

- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:838-1277` — `emit_multisig_checks` (4-case ms1, cosigner-mapping diagnostic, multiset md1_xpub_match).
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:831-836` — `MappingFailure` enum.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:895-947` — two-pass mapping algorithm enforcing precedence.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:1194-1232` — multiset `md1_xpub_match` (sort-then-compare).
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs:1104-1117` — typed `check_key_vector_distinctness`.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs:261-275` — raw-string `check_resolved_slots_distinctness` (template-mode path; v0.4 doc-comment stale relative to v0.5 SPEC).
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs:707` — md1 4-hex `chunk_set_id` format string.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/bundle.rs:724` — mk1/ms1 5-hex `chunk_set_id` format string.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs:42-44` — `derive_mk1_chunk_set_id` packing.
- `mnemonic-toolkit/crates/mnemonic-toolkit/src/error.rs:68-76` — `Bip388Distinctness` / `Bip388VerifyDistinctness` variants and exit-code mapping. The variant doc-comment at `error.rs:69-71` shares the same v0.4-era "raw-string equality" framing as `bundle.rs:259-260`; both lag the v0.5 SPEC reversal and resync at next release.
- BIP-388 §"Specification" — wallet-policy template + distinct key-information vector requirement.
- Toolkit SPEC v0.5 §4.11.b — typed `DerivationPath` equality (the deliberate v0.4 → v0.5 reversal). §5.7 — multiset `md1_xpub_match` + four-case ms1 table + mk1 cosigner-mapping diagnostic. §6.6 row 13 — `Bip388Distinctness` exit-2 row.
