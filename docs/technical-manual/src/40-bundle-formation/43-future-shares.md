# Future Shares

The bundle envelope at v0.5 ships a **single-string** card per logical record per format: one md1 record, N mk1 records, 0..N ms1 records. None of the three formats currently supports **K-of-N share encoding** — Shamir-style splitting where any K of N shares reconstructs the secret. This chapter documents the invariants locked at v0.1 across all three formats so that v0.2-shares can be added *additively* without re-engraving v0.1 cards, and explains why ms1 will ship the share path first.

The migration contract is normative for ms-codec (SPEC `mnemonic-secret/design/SPEC_ms_v0_1.md` §5); the parallel mk1 and md1 contracts are tracked as forward-look annotations in their respective SPECs and are not yet wire-frozen.

## Why shares matter

A single-string backup card is a **single point of compromise**. An attacker who finds one ms1 card recovers the full BIP-39 entropy; a defender who loses one ms1 card loses the wallet. K-of-N share encoding decouples authentication from recovery: distribute N shares across geographically-separate locations, lose up to N-K of them, and the secret remains recoverable. BIP-93 codex32\index{BIP-93} specifies the math directly — Galois-field interpolation over GF(32) with the codex32 alphabet — and ms1 inherits that primitive verbatim from `rust-codex32`.

The three formats face different design problems for sharing:

| Format | Underlying primitive | v0.1 state | v0.2-shares engineering |
|---|---|---|---|
| **ms1** | BIP-93 codex32 (via `rust-codex32`) | Single-string `threshold = 0`, `share-index = "s"` | **Math is BIP-93-specified.** Add prefix-byte type discriminator + id-as-share-set-group-key. |
| **mk1** | HRP-`mk` forked-BCH + chunked-card framing | Chunked-card sets, one set per xpub record, grouped by `chunk_set_id` | New share-aware header bits + threshold field; no codex32-style interpolation primitive shipped yet |
| **md1** | HRP-`md` forked-BCH + chunked-card framing | Chunked-card sets, one set per wallet policy | Same primitive shortfall as mk1 plus a question of *what* to share (every cosigner needs the descriptor; sharing the descriptor itself is uncommon) |

## ms1 v0.1 → v0.2-shares migration contract

\index{v0.1 → v0.2-shares migration}Four invariants are locked at v0.1 to guarantee forward-compatibility with v0.2-shares (SPEC `mnemonic-secret/design/SPEC_ms_v0_1.md:212-226`):

### Invariant 1 — Reserved-prefix byte

\index{reserved-prefix byte (v0.2)}v0.1 emits a `0x00` byte as the first payload byte of every ms1 codex32 string, before the entropy bytes. The `0x00` value is **reserved**: v0.1 decoders reject any other value via `Error::ReservedPrefixViolation` (enforced at `mnemonic-secret/crates/ms-codec/src/envelope.rs`). v0.2 promotes this byte to a **type discriminator** (`0x01 = entr` share, future `0x02 / 0x03 / …` for additional payload kinds that fit BIP-93's brackets). A v0.2 decoder seeing prefix `0x00` falls back to v0.1's "type tag is in BIP-93 `id` field" interpretation, which always means `id = "entr"` in v0.1.

This is the most important of the four invariants: every v0.1 ms1 string remains **forward-readable** by every v0.2 decoder without re-engraving.

### Invariant 2 — Grouping discriminator

\index{share-set grouping}v0.2 readers assembling K-of-N entr shares must **gate on the prefix byte before** treating BIP-93's `id` field as a share-set group key. Three cases:

| Prefix byte | Interpretation | Share grouping? |
|---|---|---|
| `0x00` | v0.1 single-string secret | Never groups; dispatched to v0.1 single-string decode path |
| `0x01` | v0.2 entr share | Groups by BIP-93 `id` |
| `≥ 0x02` | Future kind-specific path | MUST NOT default to entr grouping; kind-specific dispatch required |

Without this gate, two unrelated v0.1 strings (each carrying `id = "entr"` in BIP-93's id field) would be misgrouped as shares of one secret. Without the `≥ 0x02` clause, any future payload kind (e.g., a `mnem` entropy-plus-wordlist-language payload) could be silently miscategorized as an entr share. The v0.2 SPEC must additionally maintain a small registry table of allocated prefix-byte values.

### Invariant 3 — Encoder anti-collision

\index{RESERVED\_TAG\_TABLE}v0.2 encoders MUST refuse to emit any `id` value that is a member of v0.1's `RESERVED_TAG_TABLE` (`entr`, `seed`, `xprv`, `mnem`, `prvk`, plus any tags added in v0.1.x patches). For random `id` generation: re-roll on collision (rate ≈ 5 / 32⁴ ≈ 1 in 209,715, negligible). For deterministic `id` derivation (e.g., hash-of-secret): hard error, caller must change derivation nonce or use the random-generation path.

The reason is that v0.1 already used the `id` field to carry the type-tag (against BIP-93's intent — BIP-93 leaves `id` "implementation-defined"). v0.2 reverts `id` to BIP-93's random-per-secret-set semantics; legacy type-tag values would alias share-set ids and break the grouping invariant.

### Invariant 4 — API back-compat

v0.1's public encoder signature `pub fn encode(tag: Tag, payload: &Payload) -> Result<String>` is preserved unchanged across v0.2. v0.2 adds a *new* overload `pub fn encode_shares(tag, threshold, payload_set) -> Result<Vec<String>>`. Critically, `encode_shares(tag, Threshold::ZERO, &[p])` MUST produce a **wire-bit-identical** string to v0.1's `encode(tag, &p)` for the same inputs — both paths emit the same envelope (prefix `0x00`, `id` = tag, threshold = 0, share-index = `s`). SHA-pinned regressions on v0.1 outputs continue to pass after callers swap to the new API.

The four invariants compose: a v0.2 reader holding a mixture of v0.1 single-string secrets and v0.2 entr shares routes each string by its prefix byte first, then dispatches v0.2 shares into BIP-93 `id`-based grouping, and the v0.2 SPEC's prefix-byte registry forecloses ambiguity with future payload kinds.

## mk1 v0.2-shares outlook

\index{mk1 chunked-card grouping}mk1 already engraves N records (one per cosigner) per bundle — that is a different kind of "many cards per logical wallet" than K-of-N shares, but it shares structural plumbing. The chunked-card framing (`mnemonic-key/crates/mk-codec/`) groups card chunks by a 20-bit `chunk_set_id` derived from the wallet's policy stub. v0.2 share-aware mk1 would extend this with:

- A **threshold field** in the chunked-header (currently `compact-73` and other modes carry no threshold metadata).
- A **share-index** field per chunk-set-id (currently unset; v0.1 implicitly treats every cosigner's mk1 set as the *same* logical record across multiple physical chunks).
- A share-aware decoder that interpolates over GF(32) — but mk1's wire format is HRP-`mk` forked-BCH, not codex32, so the interpolation primitive cannot be inherited from `rust-codex32` directly. This is the principal reason mk1 lags ms1 on share encoding.

The v0.1 chunked-header bit layout already reserves bits for future header-version bumps, leaving room for the threshold/share-index annotations without re-allocating the existing fields. v0.2-mk1 will land as a wire-format bump (`version` field tick), not as an in-place extension of v0.1.

## md1 v0.2-shares outlook

md1's sharing problem is **less acute** than ms1's: the md1 card carries no secret material, only wallet policy. A reader who recovers the md1 card alone learns the descriptor but not any cosigner's xpub or secret; a defender who loses the md1 card can reconstruct it from any cosigner who has retained their copy of the descriptor (since every cosigner who is to spend needs the descriptor anyway).

Nevertheless, v0.2-shares for md1 has a defined niche: distributing a single canonical md1 across N geographic locations so that a multisig spend after a partial loss still has the descriptor available. The engineering looks similar to mk1's — chunked-header threshold + share-index bits, GF(32)-based interpolation on the bytecode payload — and shares the same primitive shortfall (HRP-`md` forked-BCH, not codex32). md1 v0.2-shares specification is **not yet drafted** and is gated on toolkit-level demand (specifically, the user-facing question of whether "share the descriptor card itself" is a common-enough operational request to justify the format complexity).

## Why ms1 ships shares first

Three independent reasons converge on ms1 leading:

1. **BIP-93 specifies the math.** ms1's underlying primitive is BIP-93 codex32, and `rust-codex32` already exposes a public `Codex32String::shares` API for threshold-share construction. ms1's v0.2 work is structural plumbing (prefix-byte gates, anti-collision, API surface), not cryptography.
2. **The migration contract is already locked.** Every v0.1 ms1 card emitted at toolkit v0.5+ already carries the reserved-prefix byte and the type-tag-in-`id` framing that v0.2 will pivot on. No re-engraving needed.
3. **The use case is highest-value.** A lost ms1 card = lost wallet; sharing ms1 directly addresses the largest single point of compromise in the bundle. mk1 and md1 sharing addresses operational convenience (geographic redundancy of public material) rather than catastrophic-loss recovery.

The expected sequencing is: ms-codec v0.2 (entr K-of-N shares) → toolkit v0.6+ exposes `--threshold K --share-count N` for the ms1 slot → md-codec / mk-codec v0.2-shares follow opportunistically as the engineering effort warrants.

## Toolkit-level orchestration

When ms-codec v0.2 lands, the toolkit's bundle subcommand will gain (provisional, subject to v0.6+ design):

- `--ms1-threshold K --ms1-share-count N` flags on the `bundle` command, producing N ms1 cards per secret-bearing slot instead of one. Each card carries the same `id` (BIP-93's share-set group key) and a distinct `share-index`.
- A `--share-index <ix>` filter on `verify-bundle` to verify a specific share against the bundle's expected single-secret reconstruction (interpolating from the other K-1 supplied shares).
- An anti-collision check at bundle creation: the toolkit refuses to assign the same `id` to two different secret-bearing slots (a multi-source multisig where two cosigners' ms1 cards collide on `id` would create a recovery hazard if both sets were later supplied to a v0.2 reader).

The `chunk_set_id` cross-card binding from §IV.2 continues to police "are these cards from the same bundle" at the prefix level, orthogonally to ms1's BIP-93 `id`-based share grouping (which polices "are these ms1 cards from the same secret share-set").

## Source pointers

- `mnemonic-secret/design/SPEC_ms_v0_1.md:212-226` — the four-invariant migration contract (verbatim authority).
- `mnemonic-secret/design/SPEC_ms_v0_1.md:271-282` — out-of-scope items deferred to ms-codec v0.2+.
- `mnemonic-secret/crates/ms-codec/src/envelope.rs` — `Error::ReservedPrefixViolation` enforcement of invariant 1 (reserved-prefix byte locked at `0x00`).
- `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md:290` — toolkit-level acknowledgement that K-of-N share encoding is gated on ms-codec v0.2.
- BIP-93 §"Specification" — codex32 threshold/share-index field semantics, share-set `id` semantics, share-index `s` reserved value.
- §IV.2 §"Invariant 1" — the bundle-level `chunk_set_id` binding that polices cross-format card co-membership, orthogonal to ms1's BIP-93 `id` share-set grouping.
