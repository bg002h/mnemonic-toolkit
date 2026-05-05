# Phase 1.5 Pre-SPEC Spike — Toolkit v0.2 Sibling-API Verification

**Date:** 2026-05-05
**Reviewer:** the spike runner
**Gate role:** Go/no-go for Phase 2 SPEC drafting per `feedback_spike_before_locking_wire_format`.

Locked deps: `bitcoin = "0.32"`, `mk-codec @ mk-codec-v0.2.1`, `md-codec @ md-codec-v0.16.1`. Spike crate at `/tmp/toolkit-v0_2-spike/` (ephemeral, not committed).

## Verified API surface

### Multisig descriptor construction (Q1)

`spike_multisig_descriptor` confirms:

- `Body::Variable { k, children }` — field names are `k: u8` and `children: Vec<Node>` (NOT `threshold` / `n`); `n` is implicit from `children.len()`.
- `Tag::PkK` — correct tag for key-reference leaves inside `SortedMulti`/`Multi`/`MultiA`/`SortedMultiA`/`Thresh` (matches `tree.rs::sortedmulti_2of3_round_trip` test).
- `Tag::Wsh` wraps the `SortedMulti` with `Body::Children(vec![multisig_node])`.
- `Tag::SortedMulti` exists (separate from `Tag::Multi`); used here for BIP-388 wallet policies.
- `descriptor.is_wallet_policy()` returns `true` when `tlv.pubkeys` is `Some(non-empty)`.
- `compute_wallet_policy_id(&Descriptor)` returns `WalletPolicyId([u8; 16])` with `as_bytes() -> &[u8; 16]` accessor.

Observed for 2-of-3 wsh-sortedmulti at `m/48'/0'/0'/2'` with 3 generator-G xpub fillers:

- `is_wallet_policy: true`
- `split` produced **6 strings**, lengths `85, 85, 85, 85, 85, 81` (max chunk size 85 chars)
- `policy_id` (16B): `e9e393609870f0ecd51fa0a923a6f336`
- Round-trip OK; `policy_id` stable across encode/decode (re-derived bytes match).

Sample chunks:
```
md1z2mgvzsqqjtvyyy4qqxz8pz2zj4qhe3h4dhmhh90auqdsjx3t8s90pupzyg3zyg3zyg3qdxwxpx5l8qp28
md1z2mgvz3txvesy7d7vel0nh9m4326qc54e6rskpczn07dktww9rv4nu5ptvt0s9ucqj5cqh83ac5n7w
```

### Privacy-preserving + multi-stub mk1 (Q6 + Q1)

`spike_privacy_multistub` confirms:

- `KeyCard::new(stubs: Vec<[u8;4]>, origin_fingerprint: Option<Fingerprint>, path: DerivationPath, xpub: Xpub)` — signature matches; `origin_fingerprint = None` is accepted for privacy-preserving mode.
- `mk_codec::{encode, decode}` re-exported from crate root.
- Decoder reconstructs `origin_fingerprint == None` (NOT `Some([0u8;4])` or similar sentinel).
- Multi-stub list (3 entries) round-trips byte-exact.

Observed for 3-stub privacy-mode card with path `m/48'/0'/0'`:

- `encode` produced **3 strings**, lengths `111, 111, 28` (header chunk + middle + tail; total ~250 chars).
- `decoded.policy_id_stubs == card.policy_id_stubs` (3 entries preserved verbatim).
- `decoded.origin_fingerprint == None` ✓
- `decoded.xpub == card.xpub` ✓ (after path adjusted to match xpub's intrinsic depth/child_number — see Errata).

Sample chunks:
```
mk1qpq5r5zqqqp64w7vm5gjyv6y24n80z87qwcgpqyqpzqgpqyqpzqgpqyqpqzg3vs7z4du5kfa5j7pjz3xsqg36v06mlwfqynss39k3rmjyjuh
mk1qpq5r5zzt9mqpm9ksjk0tjjwx
```

### Divergent paths (Q4 + Q2)

`spike_divergent_paths` confirms:

- `PathDeclPaths::Divergent(Vec<OriginPath>)` is the variant shape; encoder accepts `n=2` with two distinct `OriginPath`s, decoder reconstructs the `Divergent` variant (does NOT collapse to `Shared`).
- `PathDecl { n, paths }` — encoder enforces `paths.len() == n` for `Divergent` (`Error::DivergentPathCountMismatch` on mismatch; not exercised here).

Observed for 2-of-2 wsh-sortedmulti with cosigner @0 at `m/48'/0'/0'/2'` and cosigner @1 at `m/48'/0'/5'/2'` (different BIP-48 accounts):

- `split` produced **4 strings**, lengths `88, 88, 88, 86`.
- `decoded.path_decl.paths` matched `Divergent` variant; `paths[0] == path_a` and `paths[1] == path_b` (both `OriginPath`s preserved verbatim, both BIP-48 accounts intact).

## SPEC patches needed

(none — brainstorm Q1/Q2/Q4/Q6 assumptions hold against pinned sibling sources)

## Errata / surprises

Three deviations from the spike-as-written; none invalidate any brainstorm assumption, but the SPEC author should be aware:

1. **md-codec `Pubkeys` TLV requires valid secp256k1 points.** The spike's first iteration used a synthetic 33-byte pubkey (`0x02 || seed_byte × 32`). `split` accepted it (encoder is permissive), but `reassemble` rejected with `Error::InvalidXpubBytes { idx: 0 }` — the decoder calls `validate_xpub_bytes` (`validate.rs:191`), which runs `bitcoin::secp256k1::PublicKey::from_slice(&xpub[32..65])`. Fix: use the secp256k1 generator G as the compressed pubkey (matches md-codec's own `one_test_xpub_bytes()` helper at `derive.rs:634`). The 32-byte chain-code prefix is unvalidated, so distinct entries can differ in chain code only. **Implication for toolkit:** when toolkit constructs xpubs from `bitcoin::bip32::Xpub`, this is a non-issue (real xpubs always have valid points); only synthetic test fixtures need to be careful. SPEC §4.6.x or wherever toolkit calls into md-codec should note: pubkey validity is enforced on decode, not just encode.

2. **mk-codec drops `xpub.depth` / `xpub.child_number` on the wire and reconstructs them from `origin_path`.** This is documented in `key_card.rs:48-52`:
   ```text
   depth        := component_count(origin_path)
   child_number := last_component(origin_path)
   ```
   The spike's first iteration used `origin_path = m/48'/0'/0'/2'` (depth 4, last `2'`) with the BIP-32 test-vector xpub (depth 3, child `0'`). Encode-then-decode produced an `Xpub` whose `chain_code` and `public_key` matched perfectly but whose `depth` and `child_number` had been overwritten by the path-derived values, so `decoded.xpub == card.xpub` fails. Fix: align `origin_path` with the xpub's intrinsic metadata (used `m/48'/0'/0'` to match depth=3, child=`0'`). **Implication for toolkit:** when toolkit derives a fresh xpub from a master via `master.derive_priv(&path).into()`, depth and child_number naturally agree with the path, so this only bites if the toolkit ever re-uses an xpub with a path that doesn't terminate at it. Worth a SPEC sentence in §4.x: "the toolkit's `KeyCard` construction MUST use a path whose depth and final component match the xpub being declared, else round-trip identity is silently lost on the BIP-32 metadata fields."

3. **No capacity / length-bracket overflows observed at v0.2 fixture sizes.** The 2-of-3 multisig fits in 6 md1 chunks (max 85 chars); the 3-stub privacy mk1 card fits in 3 chunks (max 111 chars). Brainstorm-locked v0.2 fixture set (2-of-3 multisig + per-cosigner mk1 card with 3 stubs) is comfortably within both formats' chunked envelopes — no SPEC-blocker analogous to the ms1 v0.1 r5 finding.

## Verdict

**GREEN — advance to Phase 2 SPEC.**

All three load-bearing brainstorm assumptions (Q1 multisig descriptor construction, Q6 privacy-preserving + multi-stub mk1, Q4+Q2 divergent paths) hold against pinned sibling sources `md-codec-v0.16.1` and `mk-codec-v0.2.1`. The two errata above are documentation refinements, not assumption breakers; they should be folded into the SPEC's "calling sibling APIs" section but do not change the brainstorm's locked direction.
