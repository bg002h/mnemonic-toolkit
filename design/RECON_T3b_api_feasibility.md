# RECON — T3-b public-API feasibility for direct-construction `tests/wire_golden.rs`

**Read-only recon, no code changes.** Repo: `descriptor-mnemonic`, `main@db0e1275` (HEAD at recon time,
one `docs(FOLLOWUPS)` commit past the BIP-alignment release `5aae2bd1`). Crate: `md-codec 0.41.0`
(`crates/md-codec/Cargo.toml`).

## Question

T3-b's SPEC (`design/SPEC_test_hardening_T3_wire_goldens.md` §T3-b) calls for extending
`crates/md-codec/src/test_vectors.rs::MANIFEST` with 4 new shapes, plus a frozen-hex `tests/wire_golden.rs`.
An R0 found that the MANIFEST/generator path cannot express 3 of the 4 shapes without editing production
`src/` code (`Vector` struct + `md-cli`'s `parse_template`/`vectors.rs` generator). This recon checks the
proposed workaround: build the 3 shapes **directly inside `crates/md-codec/tests/wire_golden.rs`**, using
only `md-codec`'s public library API (struct-literal `Descriptor` construction — no descriptor-string
parser exists inside `md-codec` itself; that logic lives downstream in `md-cli`, unreachable from
`md-codec`'s own `tests/`).

## Why the MANIFEST path is blocked (confirms the R0 finding)

`md_codec::test_vectors::Vector` (`crates/md-codec/src/test_vectors.rs:13-35`) has fields `name`, `template`,
`keys: &[(u8, &str)]`, `fingerprints`, `force_chunked`, `path: Option<&str>` — **no field for per-key
divergent origin paths, no field for `use_site_path_overrides`**. Worse, the generator
(`crates/md-cli/src/cmd/vectors.rs:38`) hardcodes `parse_template(v.template, &[], &fps)` — **`v.keys` is
never even read**; wallet-policy mode is entirely unreachable via the MANIFEST today regardless of the
struct's nominal field. Adding real support requires: (1) a new `Vector` field (`test_vectors.rs`, part of
`md-codec`'s published `pub mod test_vectors;` surface) and (2) generator logic in `md-cli/src/cmd/vectors.rs`
(`src/`, not `tests/`) to actually plumb `keys`, a divergent-origin encoding, and an override encoding into
`parse_template`/`Descriptor`. Both are production `src/` edits — confirms the R0's finding.

## Public API surface reachable from `crates/md-codec/tests/*.rs`

`lib.rs` re-exports `Descriptor, encode_payload, encode_md1_string` (from `encode`), `OriginPath,
PathComponent, PathDecl, PathDeclPaths` (`origin_path`), `Tag`, `TlvSection`, plus `pub mod`s `tree` and
`use_site_path` that are public but not re-exported at the crate root (still fully reachable via
`md_codec::tree::{Node, Body}` / `md_codec::use_site_path::{UseSitePath, Alternative}`). **Every field of
every one of these types is `pub`** — `Descriptor{n, path_decl, use_site_path, tree, tlv}`, `TlvSection{
use_site_path_overrides, fingerprints, pubkeys, origin_path_overrides, unknown}`, `PathDecl{n, paths}`,
`OriginPath{components}`, `PathComponent{hardened, value}`, `UseSitePath{multipath, wildcard_hardened}`,
`Node{tag, body}` (`Body` is a plain public enum). There is **no** descriptor-string→`Descriptor` parser
inside `md-codec` — `parse_template` lives in `md-cli` (`crates/md-cli/src/parse/template.rs:2114`), which
depends on `md-codec`, not vice versa, so it is categorically unreachable from `md-codec`'s own `tests/`.

**Precedent already in the tree, proving this compiles and passes today:** `crates/md-codec/tests/wallet_policy.rs`
and `crates/md-codec/tests/per_key_use_site_override.rs` both construct `Descriptor` via bare struct literals
using exactly this import set, and both are ordinary (non-experimental) integration tests in CI.

## Per-shape feasibility

| Shape | Public-API constructible? | Exact entry point | Example | Deterministic? | Wire-section confirmed? |
|---|---|---|---|---|---|
| **(b) wallet-policy / embedded-pubkey** | **YES** | `Descriptor{ n, path_decl, use_site_path, tree, tlv: TlvSection{ pubkeys: Some(vec![(u8,[u8;65])]), ..TlvSection::new_empty() } }` → `md_codec::encode::encode_payload(&d)` / `encode_md1_string(&d)`. Live precedent: `wallet_policy.rs:207-222` `cell_7_wpkh_full()`. | `wsh(sortedmulti(2,@0/<0;1>/*,...))` conceptually; concretely no string is needed — build the tree with `Tag::Wsh`/`Tag::SortedMulti`/`Body::MultiKeys` and set `tlv.pubkeys = Some(vec![(0, xpub_bytes)])`. `is_wallet_policy()` predicate: `matches!(&tlv.pubkeys, Some(v) if !v.is_empty())` (`encode.rs:49-51`). | **YES** — `encode_payload`/`BitWriter` are pure bit-packing, no RNG/clock; directly pinned by the existing test `encoder_determinism_2of3_cell_7_byte_identical_emit` (`wallet_policy.rs:824-846`), which asserts two `encode_payload(&d)` calls on a pubkey-bearing `Descriptor` are byte-identical. | **YES** — traced `TlvSection::write` (`tlv.rs:149-172`): `pubkeys: Some(_)` emits `TLV_PUBKEYS = 0x02` (tag, 5 bits) + varint(bit_len) + 65-byte-per-key payload, sorted into ascending-tag position among emitted TLVs (`tlv.rs:200-206`). |
| **(c) use-site path override** | **YES** | Same `Descriptor` literal, set `tlv.use_site_path_overrides = Some(vec![(1u8, UseSitePath{...})])` (must key on `@1..`, never `@0` — `Error::BaselineUseSiteOverride`). Live precedent: `per_key_use_site_override.rs:120-150` `divergent_wsh_multi()`. | `wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))` — the override on `@1` diverges from the baseline `use_site_path` (`<0;1>`). | **YES** — same pure `encode_payload` path; `per_key_use_site_override.rs`'s `decode_accepts_genuine_divergent_override` test round-trips this exact shape (`encode_payload` → `decode_payload`) and asserts the override survives byte-for-byte. | **YES** — `TlvSection::write` (`tlv.rs:99-122`): `use_site_path_overrides: Some(_)` emits `TLV_USE_SITE_PATH_OVERRIDES = 0x00` (tag) + varint(bit_len) + per-entry `(idx: kiw bits, UseSitePath::write)`. Traced directly in source. |
| **(d) origin override / `Divergent`** | **YES** | `path_decl: PathDecl{ n, paths: PathDeclPaths::Divergent(vec![origin_a, origin_b, ...]) }` (length must equal `n`, else `Error::DivergentPathCountMismatch`). Live precedent: `wallet_policy.rs:500-575` `divergent_paths_wallet_policy_2of2_round_trip` (2-of-2, full fp+xpub TLVs, `Divergent` path_decl). | Conceptually `[fp1/48'/0'/0'/2']xpub.../<0;1>/* , [fp2/48'/0'/1'/2']xpub2.../<0;1>/*` (per-cosigner divergent account step); concretely built as two distinct `OriginPath{components: vec![PathComponent{...}, ...]}` values in the `Divergent(vec![...])`. | **YES** — `encode_payload` sets `header.divergent_paths = matches!(d.path_decl.paths, PathDeclPaths::Divergent(_))` automatically (`encode.rs:113-117`) and calls the same pure `BitWriter` path; no RNG. | **YES** — traced `PathDecl::write` (`origin_path.rs:114-131`): `Divergent(paths)` writes each of the `n` `OriginPath`s in order (no extra per-path marker — the header bit 4, `header.rs:9,29` `divergent-paths flag`, is what signals divergent mode on the wire). |

All three shapes are **already exercised, today, by existing `tests/` files** using exactly this
direct-struct-construction pattern — this is not a novel technique, it is the established idiom in this
crate's test suite. No shape requires a production `src/` change to become *constructible*; the MANIFEST
gap is strictly about the **generator/CLI corpus-export path** (`md vectors`), not about whether `md-codec`'s
library API can express these shapes.

## Chunk-strings reproducibility (`wsh_sortedmulti_2chunk`)

**Confirmed reproducible.** The committed corpus fixture (`crates/md-codec/tests/vectors/wsh_sortedmulti_2chunk.descriptor.json`)
shows the *exact* `Descriptor` shape the MANIFEST entry resolves to: `path_decl: {tag: "Shared", data: "m"}`
(i.e. `PathDeclPaths::Shared(OriginPath{components: vec![]})` — elided origin, **not** the canonical
`m/48'/0'/0'/2'` bytes; `md-cli`'s `resolve_placeholders`/`make_path_decl`, `template.rs:495-509`, writes
`to_origin_path(None)` = empty components when no `[fp/path]` bracket is present in the template, and
`v.path` is `None` for this vector), `use_site_path: UseSitePath::standard_multipath()`, `tree: Wsh >
SortedMulti{k:2, indices:[0..7]}`, `tlv.fingerprints` = the 8 listed 4-byte values, `tlv.pubkeys: null`.
Every one of these is constructible via the same public struct literals as shapes (b)/(c)/(d) above
(mirroring `wallet_policy.rs`'s `wsh_sortedmulti_2of3()` tree helper + `cell_7_wsh_2of3_full()`'s TLV
population pattern). The chunk STRINGS themselves come from `md_codec::chunk::split(&d) -> Result<Vec<String>,
Error>` (re-exported at crate root, `lib.rs`) called directly on this hand-built `Descriptor` — `split` is
unconditional (it does not check whether the payload "fits" a single string; `force_chunked` in the MANIFEST
is purely a *generator-side choice* of which top-level fn to call, `md-cli/src/cmd/vectors.rs:58-71`, not a
`Descriptor`/wire property). `split` is pure `BitWriter` bit-packing (no RNG), already pinned deterministic
by `wallet_policy.rs`'s `multi_chunk_2of3_cell_7_split_reassemble_round_trip` (`:579-607`, re-splits and
compares chunk counts) and the encoder-determinism test's `split(&d)` double-call equality check (`:843-845`).
A hand-built `Descriptor` reproducing the JSON shape above, fed to `split`, will yield the byte-identical
2-chunk `md1...`/`md1...` pair already committed in `wsh_sortedmulti_2chunk.phrase.txt` — this can be
directly asserted as the frozen golden, closing the D1 chunk-framing mutation gap independent of `md vectors`.

## Public encode entry points a `tests/wire_golden.rs` would use

From `lib.rs`'s `pub use` block (all crate-root reachable) plus two `pub mod`s not re-exported at root but
still externally visible:
- `md_codec::encode::{Descriptor, encode_payload, encode_md1_string}`
- `md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths}`
- `md_codec::tag::Tag`
- `md_codec::tlv::TlvSection`
- `md_codec::tree::{Node, Body}` — public module, not re-exported at root; reachable via full path (used this way already in `wallet_policy.rs:21`, `per_key_use_site_override.rs:29`)
- `md_codec::use_site_path::{UseSitePath, Alternative}` — same (public module, full-path reachable)
- `md_codec::chunk::{split, reassemble, derive_chunk_set_id}` — for the chunk-strings golden
- `md_codec::decode::{decode_payload, decode_md1_string}` — for round-trip sanity inside the new test if desired
- `md_codec::canonicalize::canonicalize_placeholder_indices` — implicitly invoked by `encode_payload`; not needed directly for hand-canonical inputs (all example shapes above already satisfy canonical `@N` first-occurrence order)
- `md_codec::canonical_origin::canonical_origin` — not needed for construction (only for computing what the *elided* wire form should be, already resolved above as empty `Shared`), but available if the golden wants to assert against it

No new `dev-dependencies` are needed in `crates/md-codec/Cargo.toml` — shape (b)'s 65-byte xpub-shaped
payload is just a `[u8; 65]` array (no `bitcoin`/`bip39` derivation required, unlike the funds-safety address
goldens in `per_key_use_site_override.rs`). The `derive` feature (default-on) is not required either — none
of the three shapes touch `to_miniscript`/`derive_address`; `wallet_policy.rs` itself carries no `#![cfg(feature
= "derive")]` gate and exercises this exact pattern unconditionally.

## Verdict

**All 3 shapes (b, c, d) — plus the `wsh_sortedmulti_2chunk` chunk-strings golden — can be frozen tests-only,
NO-BUMP, entirely inside a new `crates/md-codec/tests/wire_golden.rs`, with zero production `src/` changes
in either `md-codec` or `md-cli`.** The workaround fully unblocks the shapes the MANIFEST/generator path
cannot express without touching `Vector`'s struct definition and `md-cli/src/cmd/vectors.rs`'s generator
logic. Nothing is DEFERRED — the recon found no shape unreachable via the public API. (Shape (a),
long-code, is out of this recon's scope per the task's 3-shape list; note in passing it's unaffected either
way since it's expressible today through the existing `--force-long-code` CLI flag path, not part of this
public-API question.)

**Caveat for the SPEC/plan author:** freezing these as MANIFEST-independent goldens forgoes the
`.template`/`.descriptor.json`/`md vectors --out` corpus-export side-channel T3-b's SPEC also wanted (item
§T3-b fix part 1) — that remains blocked on the production-code changes described above and stays a
separate, deferred FOLLOWUP if still desired; this recon only clears the `tests/wire_golden.rs`
frozen-hex-assertion leg (§T3-b fix part 2), which was always described as the actual "independent of future
code" anchor and can ship standalone.
