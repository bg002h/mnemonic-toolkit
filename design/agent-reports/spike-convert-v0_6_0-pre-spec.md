# v0.6.0 Phase 0 — codec call-shape spike

Read-only verification of `ms-codec` v0.1.0 + `mk-codec` v0.2.1 public surfaces against the planned `convert` subcommand edges. No new code; no library writes.

## ms-codec surface

```rust
// public re-exports (lib.rs:50-55)
pub use decode::decode;
pub use encode::encode;
pub use payload::{Payload, PayloadKind};
pub use tag::Tag;
```

```rust
// signatures
pub fn encode(tag: Tag, payload: &Payload) -> Result<String>     // single string, not Vec
pub fn decode(s: &str) -> Result<(Tag, Payload)>                 // tuple
```

`Payload` is `#[non_exhaustive]` with one variant in v0.1: `Payload::Entr(Vec<u8>)`. Other tags (`SEED`, `XPRV`, `PRVK`) are `RESERVED_NOT_EMITTED_V01` per `decode.rs:36-39` and `tag.rs:72`.

`Payload::validate()` enforces BIP-39-valid entropy lengths (16/20/24/28/32 bytes) before encode; decode runs validation after extracting the payload bytes. Encoder MUST call validate; SDK callers should rely on this.

### Verified call sites for convert

```rust
// entropy → ms1
let s: String = ms_codec::encode(Tag::ENTR, &Payload::Entr(entropy_bytes.clone()))?;

// ms1 → entropy
let (tag, payload) = ms_codec::decode(s)?;
match payload {
    Payload::Entr(bytes) => bytes,                 // entropy
    _ => unreachable!("v0.1 ms-codec only emits Entr"),  // future Payload variants
}
```

## mk-codec surface

```rust
// public re-exports (lib.rs:50)
pub use key_card::{KeyCard, decode, encode, encode_with_chunk_set_id};
```

```rust
// signatures
pub fn encode(card: &KeyCard) -> Result<Vec<String>>   // multi-string (chunked)
pub fn encode_with_chunk_set_id(card: &KeyCard, chunk_set_id: u32) -> Result<Vec<String>>
pub fn decode(strings: &[&str]) -> Result<KeyCard>
```

`KeyCard` is `#[non_exhaustive]` with 4 fields:

```rust
pub struct KeyCard {
    pub policy_id_stubs: Vec<[u8; 4]>,             // MUST be non-empty
    pub origin_fingerprint: Option<Fingerprint>,
    pub origin_path: DerivationPath,
    pub xpub: Xpub,
}
```

**Critical:** `policy_id_stubs` MUST be non-empty (encoder rejects `count == 0` with `Error::InvalidPolicyIdStubCount`). For the `xpub → mk1` edge in `convert`, there is **no descriptor context** to derive a stub from — a standalone xpub without a policy binding has no canonical stub.

### Verified call sites for convert

```rust
// mk1 → xpub (+ fingerprint + path as sub-outputs)
let card: KeyCard = mk_codec::decode(strings)?;
// card.xpub, card.origin_fingerprint, card.origin_path are the outputs.
// card.policy_id_stubs is ignored (the wire-format demands it but convert doesn't surface it).

// xpub → mk1 — REQUIRES a policy_id_stubs decision (see design fork below).
```

## Design fork surfaced by the spike

`xpub → mk1` requires a non-empty `policy_id_stubs: Vec<[u8; 4]>`. The plan's brainstorm flagged this as an open SPEC question ("may need a sentinel stub or a `--policy-id-stub` flag"). Two reasonable resolutions:

### Option A: REFUSE `xpub → mk1` in v0.6.0 (recommended)

Standalone `xpub → mk1` is conceptually incomplete: mk1 cards exist to live alongside md1 (descriptor) cards via the policy stub binding. Encoding an xpub to mk1 with a fabricated zero-stub produces a malformed-by-intent card.

SPEC-level resolution: refuse `xpub → mk1` with stderr:
```
error: --to mk1 requires a policy descriptor binding (mk1 cards bind xpubs to specific policies via policy_id_stubs). Use 'mnemonic bundle --slot @0.xpub=... --template ...' to emit a complete bundle.
```

Adds `xpub → mk1` to the refusal taxonomy under §3.c (type-class mismatch / cross-format pivot redirect to `bundle`).

`mk1 → xpub` remains SUPPORTED — decode is policy-context-free, and the stubs list in the resulting KeyCard is simply ignored.

### Option B: `--policy-id-stub <8-hex>` (repeatable)

Add a flag accepting one or more `policy_id_stubs` entries. Default to no flag = no encode (forces explicit user choice; refuses without `--policy-id-stub`).

Cost: introduces a wire-format-specific concept to the convert grammar. mnemonic-toolkit's other subcommands hide policy_id_stubs entirely; surfacing it to convert users is a leak of bundle-specific semantics.

## Recommendation

**Option A.** Refusal preserves the convert subcommand's role as a single-format conversion utility; redirects the workflow that does need policy bindings (xpub + descriptor → mk1) to `bundle`, which is already designed for it.

The bidirectional `xpub ↔ mk1` symmetry breaks, but this is consistent with the toolkit's general principle: encoding edges that depend on additional artifacts (descriptors, in this case) belong to `bundle`, not `convert`.

Edge table delta vs. plan §2:

| From | To | Status | Refusal class |
|------|----|--------|---------------|
| `xpub` | `mk1` | REFUSED in v0.6.0 (was: planned with optional `--fingerprint`, optional `--path`) | §3.c — sibling-pivot/redirect to bundle |
| `mk1` | `xpub` | SUPPORTED (no change) | n/a |

## Bottom line

Spike validates that ms-codec + mk-codec library APIs are well-shaped for the convert subcommand. The `xpub → mk1` edge needs a SPEC-level refusal decision before SPEC commit. Recommend Option A.
