# tech-manual v0.3 — Phase 3.3 reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.3.0` (in progress) |
| Phase | 3.3 (Part IV §IV.3 — Future Shares) |
| Commit under review | `ba950af` |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `docs/technical-manual/src/40-bundle-formation/43-future-shares.md` · `docs/technical-manual/src/60-back-matter/62-index-table.md` (+4 rows) · `docs/technical-manual/.cspell.json` (+2 words "miscategorized", "misgrouped") |

## Findings: 1 Critical / 1 Important / 1 Low / 0 Nit

---

## Critical

**C-1. `Codex32String::shares` API does not exist in `rust-codex32 v0.1.0` (confidence: 95)**

`43-future-shares.md:73`:

> `rust-codex32` already exposes a public `Codex32String::shares` API for threshold-share construction.

The complete public API surface of `Codex32String` in the pinned `codex32 = "=0.1.0"` crate (confirmed at `/home/bcg/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/codex32-0.1.0/src/lib.rs:110-381`) is:

- `from_unchecksummed_string` (line 121)
- `from_string` (line 148)
- `parts` (line 209)
- `interpolate_at` (line 217)
- `from_seed` (line 312)

No method named `shares` exists. No method with the purpose of generating a share set from a secret exists. `interpolate_at` performs Lagrange interpolation to derive a share at a target index from an *existing* share set — that is reconstruction, not generation. Generating K shares from a secret requires evaluating the secret polynomial at K distinct GF(32) elements, which is not provided by any public function in the v0.1.0 crate.

The chapter's primary argument for "why ms1 ships shares first" uses this claim as its load-bearing premise: "ms1's v0.2 work is structural plumbing (prefix-byte gates, anti-collision, API surface), not cryptography."

Resolution: replace line 73 with an accurate description noting that BIP-93 specifies the share-generation math but `rust-codex32 v0.1.0` only exposes reconstruction (`interpolate_at`); ms-codec v0.2 will need to implement the generation step. The "no novel cryptographic design" framing survives because BIP-93 prescribes the algorithm fully (folded inline at phase close).

---

## Important

**I-1. Invariant 4 signature elides typed parameters, losing the SPEC's explicit `Threshold` type call-out (confidence: 85)**

`43-future-shares.md:49`:

> v0.2 adds a *new* overload `pub fn encode_shares(tag, threshold, payload_set) -> Result<Vec<String>>`

`SPEC_ms_v0_1.md:222` gives the full typed signature plus the key parenthetical:

```
pub fn encode_shares(tag: Tag, threshold: Threshold, payload_set: &[Payload]) -> Result<Vec<String>>
(Threshold is a v0.2-introduced type; v0.1 has no public Threshold symbol)
```

The chapter's elided form strips the types. A reader implementing a v0.2 decoder might assume `threshold` is a `usize` or `u8`. The SPEC calls out `Threshold` explicitly because it is the salient type-level change distinguishing v0.2's encoder surface from a trivially extended v0.1 function.

Resolution: restore typed signature with the SPEC parenthetical (folded inline at phase close).

---

## Low

**L-1. Source-pointer description for `SPEC_mnemonic_toolkit_v0_5.md:290` mischaracterizes the structural role of that line (confidence: 82)**

`43-future-shares.md:94`:

> `mnemonic-toolkit/design/SPEC_mnemonic_toolkit_v0_5.md:290` — toolkit-level acknowledgement that K-of-N share encoding is gated on ms-codec v0.2.

Line 290 is a bullet in SPEC §8 "Future / Deferred" — a FOLLOWUPS/deferral register, not an "acknowledgement section". A reader who looks up the pointer expecting an acknowledgement section will find a deferral table entry.

Resolution: change "toolkit-level acknowledgement" to "toolkit SPEC §8 deferral entry noting" (folded inline at phase close).

---

## Resolution (Phase 3.3 close)

All three findings folded inline at the closing commit. None deferred — the Critical fix is structural prose, the Important is a one-line restoration, the Low is a three-word substitution.

---

## Verified-correct items (no action needed)

- SPEC line range `SPEC_ms_v0_1.md:212-226` — §5 "v0.1 → v0.2 Migration Contract". Confirmed exact.
- SPEC line range `SPEC_ms_v0_1.md:271-282` — §8 out-of-scope table. Confirmed.
- `Error::ReservedPrefixViolation` enforcement at `mnemonic-secret/crates/ms-codec/src/envelope.rs` — confirmed.
- `SPEC_mnemonic_toolkit_v0_5.md:290` line number — confirmed.
- Invariant 1 fidelity — faithful paraphrase of SPEC §5 invariant 1 (line 216).
- Invariant 2 three-row table — prefix `0x00` / `0x01` / `≥0x02` dispatch structure matches SPEC §5 invariant 2 (line 218).
- Invariant 3 collision rate — "1 in 209,715" matches SPEC §5 invariant 3.
- Invariant 4 wire-bit-identical guarantee — substance matches SPEC §5 invariant 4 (line 222).
- `RESERVED_TAG_TABLE` members (`entr`, `seed`, `xprv`, `mnem`, `prvk`) — confirmed against `SPEC_ms_v0_1.md:156-163`.
- Forward-looking framing discipline in mk1 and md1 sections — "would extend this with", "looks similar to", "not yet drafted", "gated on toolkit-level demand" — all clearly signal structural reasoning, not wire-frozen specs.
- Toolkit-level orchestration section — opens with "(provisional, subject to v0.6+ design)". Flag names framed as provisional. Confirmed.
- `chunk_set_id` 20-bit description for mk1 consistent with §IV.2's 5-hex (20-bit) mk1 identifier.
- Four new `\index{}` markers each have a matching index-table row.
- "Why ms1 ships shares first" reasons 2 and 3 are sound and independent of the disputed API claim in reason 1.
- `chunk_set_id` / BIP-93 `id` orthogonality claim architecturally sound.
