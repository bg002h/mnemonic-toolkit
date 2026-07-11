# RECON — T3 (wire goldens): items #3 / #4 / #5

**Read-only recon, no code/test changes.** Source SHAs at recon time (2026-07-10): toolkit
`da76e300f5e9bbc83ef100f3d45905d8552a77ec` (= T2's spec-authoring SHA, still current HEAD — T2 is
uncommitted, see §Overlap), md-codec (descriptor-mnemonic) **`main@db0e12754fe1624692f4d440d4da12ea237fede3`**
(the verified current HEAD via `git rev-parse`; the BIP-alignment cycle's release commit
`5aae2bd1` — which ADDED the `wsh_sortedmulti_2chunk` vector that resolves part of item #4 below — is an
**ancestor** of HEAD, i.e. that vector IS present in HEAD; `db0e1275` is one further `docs(FOLLOWUPS)` commit
on top, and post-dates the eval's 2026-07-06 snapshot — that snapshot-vs-HEAD drift is the citation decay this
recon exists to catch), mk-codec (mnemonic-key) `main@008182b703a4af2497ca6f14228c17a8ee169f70` (mk not
touched by T3 — see §Overlap). Source doc: `design/agent-reports/constellation-eval-2026-07-06.md`, §2
"Round-trip-only blind spots on write-once artifacts (symmetric-bug class)", items #3/#4/#5 (lines 226-240).

**Verification note:** every file:line citation below was independently spot-checked (`Read`/`grep`
against live source in both repos) in a second pass after the initial recon — encode/decode entry points,
`lib.rs` re-exports, `raid.rs` stripe-layout doc, TLV decode-loop + F-A8 trailing-zero check, `md-cli`
`bytecode`/`encode` subcommands, `codex32.rs` regular-code caps, and `origin_path.rs`'s `Divergent` variant
all confirmed accurate. No corrections needed beyond the SHA line above.

---

## Item #3 — wc-codec Word-Card wire golden

**Verbatim:**
> No frozen Word-Card wire golden anywhere — every wc-codec and `cli_word_card` test encodes *and*
> decodes with the code under test, so a symmetric wire change (field order, header/checkpoint CRC context,
> tag placement, RS parity ordering) passes all tests and both fuzzers while **bricking every already-engraved
> steel plate**. Add `tests/wire_golden.rs`: commit exact word sequences for fixed `(kind, payload,
> payload_bits, m, t, u)` tuples + one `n=3/r=2` RAID array, generated once from the shipped binary and
> frozen with provenance. Oracle = the frozen historical wire (independent of all future code).

### 1. Wire-format locus
`crates/wc-codec/src/pipeline.rs:1-38` (module doc) fixes the engraved-stream layout:
`[H0] [GEOM 4] [ledger 2U] [interleave(K data + checkpoints)] [parity m] [stop-sign 2]`. Emit path:
`build_header` (`pipeline.rs:350`) → `encode_inner` (`pipeline.rs:664`, `pub(crate)`) → public entry
`pub fn encode` (re-exported `lib.rs:39-41`). Parse path: `pub fn decode` (`pipeline.rs:781`). RAID
plate framing (item's "+ one n=3/r=2 RAID array"): `crates/wc-codec/src/raid.rs:28-44` (module doc,
length-prefix + stripe layout) → `raid_encode`/`raid_reconstruct` (re-exported `lib.rs:42`).
Word↔symbol mapping (needed to render a "word sequence" golden, not just symbols):
`crates/wc-codec/src/wordmap.rs`.

### 2. Current coverage — confirmed round-trip-only
- `crates/wc-codec/tests/pipeline.rs` helpers `enc`/`dec` (`:52,:61`) both call the SAME `pipeline::encode`/
  `decode` under test; every assertion (`full_round_trip_mk1_and_md1` `:81`, `large_md1_k_ge_241_round_trip`
  `:224`, etc.) compares recovered-vs-original, never a literal word list.
- `crates/wc-codec/tests/raid.rs` — same pattern (`raid_encode`/`raid_reconstruct` self-referential).
- `crates/mnemonic-toolkit/tests/cli_word_card.rs`: `encode_solo_words` (`:43-66`) shells the live binary,
  feeds the JSON `words` array straight back into `--decode` (`mk1_e2e_round_trip_recovers_same_xpub_not_literal_string`
  `:71`, `md1_e2e_round_trip_is_string_deterministic` `:105`). No test anywhere asserts a word list against a
  value not derived from the code under test.
- Grepped `frozen`/`golden`/`expected_words`/`const …: &[&str]` across `crates/wc-codec/{src,tests}`: every
  hit refers to a *structural rule* being frozen (zero-pad rule `pad.rs`, primitive polynomial `field.rs`,
  array-id digest layout `raid.rs:262`, RS codeword-is-wire-frozen-but-not-the-algorithm `rs.rs:17`) — **not**
  a frozen literal wire-byte/word sequence. Finding confirmed accurate against current source, not stale.

### 3. Golden location + independent oracle
**Location:** `crates/wc-codec/tests/wire_golden.rs` — a plain integration test in the `wc-codec` **library**
crate. `encode`/`decode`/`raid_encode`/`raid_reconstruct` are all `pub` at the crate root (`lib.rs:39-42`),
so (unlike toolkit's binary-crate visibility wall hit by T2-a) there is **no visibility blocker**: this is a
normal library-crate integration test, same shape as the existing `tests/pipeline.rs`/`tests/raid.rs`.
**Oracle:** the eval's own prescription — generate once from the shipped binary (or `wc_codec::encode`
directly, pinned to a fixed `(kind, payload, payload_bits, m, t, u)` tuple) and freeze the literal word
list as a `const` with a provenance comment (binary version + git SHA + exact call). This is **not**
independent of `wc-codec` itself (it's the codec's own historical output), but it IS independent of *future*
code — exactly the "frozen historical wire" oracle the item specifies, and the only oracle type that can
catch a *symmetric* encode+decode drift (an external/hand-derived GF(2¹¹)/RS/CRC re-implementation is a much
larger lift than this sub-cycle's scope; T2-b's PGZ reference decoder for md-codec is the closer analogue,
reserved for BCH correctness, not wire layout). Fixed seed candidates: reuse the toolkit's existing
deterministic `abandon×11 about` mk1/md1 fixtures already frozen in `word_card_adapter.rs:190-197` and
`cli_word_card.rs:20-25` (byte-identical inputs across files today) so the golden's `payload`/`payload_bits`
inputs need no new fixture-generation step — only the resulting *word list* needs freezing.

### 4. Named RED mutation (loci verified against current source)
All four of the item's named classes are single-line, single-function mutations; each is **symmetric across
encode+decode** (decode reads back whatever encode wrote, so every existing round-trip test survives), and
only a **frozen literal word list** REDs:
- **Field-order swap — H0 packing.** The H0 word packs `version(4) | source-kind(2) | has-raid(1) |
  reserved(4)` (read side `pipeline.rs:393-396`; written via `build_header` → `build_geom`, `pipeline.rs:365`).
  Swap the `has-raid` and a `source-kind` bit position on BOTH the write and the symmetric read → the header
  round-trips (decode reads the moved bits back) but the engraved H0 word changes → frozen golden REDs.
- **Header/checkpoint CRC context.** `CRC5_POLY` (`sync.rs:43`) or the checkpoint byte-range fed to the CRC-5
  → a different checkpoint word is engraved; decode recomputes with the same (mutated) poly and still
  recognizes/realigns → round-trip survives, golden REDs.
- **Tag placement — payload/tag split.** The integrity tag is the `t` bits AFTER `payload_bits`
  (`extract_payload_bytes_from_slice(&all_bytes, payload_bits)` + `extract_tag_bits(&data_symbols,
  payload_bits, t)`, `pipeline.rs:1082-1088`; the emit side mirrors this split). An off-by-one on the split
  boundary shifts where the tag sits on the wire; encode+decode agree on the wrong boundary → round-trip
  survives, golden REDs.
- **RS parity ordering.** `rs_parity` (`rs.rs:134`) emits `P(β_{k+i}), i=0..m-1`. Nuance (module contract
  `rs.rs:2-4,13`): the RS *algorithm* is NOT wire-frozen and parity is append-only, but the emitted parity
  SYMBOLS ARE engraved onto the plate — reversing/rotating the emitted parity order changes the wire while
  decode (Gao) still corrects → round-trip survives, golden REDs.
Each is single-line, single-function; because encode and decode both derive from the same source today, only
a **frozen literal expected value** (not a recompute-and-compare) REDs — confirming the item's own diagnosis.

### 5. Repo + gates + lockstep
**Repo:** `mnemonic-toolkit`, crate `wc-codec` (workspace member, `Cargo.toml:2`). **NO-BUMP** — additive
`tests/` file only, no `src/` change, no published-API/wire change. Toolkit master now has LIVE branch
protection (Cycle I, verified via `gh api`: `contexts=["examples","test (ubuntu-latest)","clippy"]`,
`enforce_admins:false`) — `test (ubuntu-latest)` runs `cargo test --workspace`, which **already covers**
`wc-codec`'s test dir (`Cargo.toml` workspace members = `mnemonic-toolkit` + `wc-codec`), so a new
`crates/wc-codec/tests/wire_golden.rs` is automatically gated on next PR/push without further CI wiring.
No manual/GUI/schema lockstep (wire goldens are internal test fixtures, no CLI/flag surface change) —
confirmed: `docs/manual/tests/lint.sh` only checks clap flag coverage, untouched by a `tests/` addition.

---

## Item #4 — md1 frozen corpus gaps

**Verbatim:**
> md1 frozen corpus omits every production-default shape — no ≥2-chunk vector, no long-code string, no
> pubkey / use-site-override / origin-override TLV vector (the use-site override being the locus of a past
> silent-wrong-address bug). The only byte-pinned drift gate never exercises multi-chunk framing (how D1
> stayed invisible) or the keyed-card TLV encodings. Add those vectors; the `v0.11/v0.13` "fixtures" are
> live-emitted at test time (encoder and fixture move together — not an independent anchor).

**⚠ PARTIALLY STALE as of today — re-verify before drafting the SPEC.** This item was written against the
eval's 2026-07-06 snapshot. The BIP-alignment cycle shipped **today** (2026-07-10, `md-codec 0.41.0`,
commit `5aae2bd1`) and **already added a genuine ≥2-chunk vector** (`wsh_sortedmulti_2chunk`,
`crates/md-codec/src/test_vectors.rs:94-104`). The "no ≥2-chunk vector" clause of this item is **RESOLVED**;
the remaining clauses (long-code string, pubkey/wallet-policy-mode, use-site-override, origin-override) are
**still open**. Scope T3's SPEC to the residual gaps only — do not re-add ≥2-chunk coverage.

### 1. Wire-format locus
TLV section: `crates/md-codec/src/tlv.rs:24-32` — `TlvSection { use_site_path_overrides:
Option<Vec<(u8, UseSitePath)>>, pubkeys: Option<Vec<(u8, [u8;65])>>, … }`; decode loop `tlv.rs:210-316`
(consumes remaining bits, F-A8 trailing-zero check `:317-330`). Origin-override (per-key divergent path,
distinct from the TLV section): `crates/md-codec/src/origin_path.rs:91-95` — `PathDeclPaths::{Shared,
Divergent(Vec<OriginPath>)}`. Long-code path: `crates/md-cli/src/cmd/encode.rs:30,41`
(`force_long_code` flag) vs. the regular-code 80-data-symbol cap (`crates/md-codec/src/codex32.rs:20-28`,
BCH(93,80,8)). The drift gate itself: `crates/md-cli/tests/vector_corpus.rs:15-42`
(`vectors_output_matches_committed_corpus`) diffs a fresh `md vectors --out <tmp>` run against the committed
`crates/md-codec/tests/vectors/`.

### 2. Current test coverage — confirmed self-referential (with today's partial fix)
`crates/md-cli/src/cmd/vectors.rs:17-45` regenerates every fixture by calling
`md_codec::encode::encode_payload` (`:47`) — the **same function** `Descriptor::canonical_payload_bytes`
delegates to (`encode.rs:71-73`) and the same function T3 item #5's `md1_to_canonical` calls transitively.
The corpus manifest is `crates/md-codec/src/test_vectors.rs:68-116` (`MANIFEST`, 15 entries as of today).
Grepped every entry: **all 15 have `keys: &[]`** (no vector exercises wallet-policy / embedded-pubkey mode,
i.e. `Descriptor::is_wallet_policy() == true` is never corpus-tested), **none set `path` to a per-key
divergent form** (`path: Option<&str>` only feeds `PathDeclPaths::Shared`, `vectors.rs:41-44` — divergent
origin is structurally unreachable via this manifest shape today), **none use `force_long_code`** (the doc
comment on `single_string_boundary`, `test_vectors.rs:107-116`, explicitly says "NOT chunked, NOT long-code" —
i.e. the corpus author was aware and still left long-code uncovered), and grep for `use_site_path_overrides`
outside `tlv.rs` itself confirms **zero** manifest entries set it (`wsh_divergent_paths`, `test_vectors.rs:76`,
varies the **use-site** path via `/<2;3>/*` template syntax, which is a `UseSitePath` template difference —
not the TLV `use_site_path_overrides` per-key *override* mechanism, a different wire feature). So: 4 of the
item's 5 named gaps remain open (≥2-chunk is closed); the "encoder and fixture move together" self-reference
critique is **still fully live** — `md vectors` and `vectors_output_matches_committed_corpus` both transit
`encode_payload`, so a self-consistent encoder regression (e.g. swapping the `pubkeys`/`use_site_path_overrides`
TLV tag-order) would regenerate matching (wrong) output and the diff test would stay green.

### 3. Golden location + independent oracle
**Location:** extend `crates/md-codec/src/test_vectors.rs::MANIFEST` with new entries (the existing
single-source-of-truth pattern — consumed by `md vectors`, md-codec's own integration tests, and
`md-cli/tests/{json_snapshots,template_roundtrip,vector_corpus}.rs`) — this is the natural, repo-idiomatic
location. **Independent-oracle problem:** since `md vectors` and the drift-diff test both call
`encode_payload`, simply adding manifest entries does **not**, by itself, close the "not an independent
anchor" critique — it only broadens the SHAPE coverage (multi-chunk, TLV) while leaving the SAME
self-referential mechanism. Two complementary fixes, both cheap:
  (a) **Freeze the `.bytes.hex` values as literal byte-string assertions in a NEW test**
      (e.g. `crates/md-codec/tests/wire_golden.rs`, mirroring item #3's approach) that does NOT regenerate
      via `md vectors`/`encode_payload` at test time but instead hardcodes the expected hex string(s) as a
      `const`, generated once and frozen with provenance — this is the actual "independent of all future
      code" oracle the eval wants, layered ON TOP of (not replacing) the existing self-consistency diff gate.
  (b) For the TLV-shape vectors specifically, a hand-computed bit-offset check is feasible and cheap: TLV
      entries are tag(5-bit varint-ish)+len+value (`tlv.rs:210-260`) — a human can hand-encode a *minimal*
      `pubkeys`/`use_site_path_overrides` TLV entry's bit pattern for a small fixed input and assert specific
      bytes/bit-offsets, independent of `encode_payload`'s internals (same spirit as T2-b's PGZ reference
      decoder, but far simpler since TLV framing is small-state, not iterative FEC math).
Long-code fixture: `md encode --force-long-code` on a template within the regular-code cap already works
(`cmd/encode.rs:30-41`); a minimal 1-key template forced long-code gives a small, easy-to-hand-verify vector.

### 4. Named RED mutation
- **Multi-chunk framing (D1-class):** already RED-provable today via `wsh_sortedmulti_2chunk` — but only
  self-referentially (see above); a hand-frozen `.bytes.hex` per chunk closes it. Mutation: byte- vs
  bit-boundary chunk-split framing (`crates/md-codec/src/chunk.rs`, the D1 documentation/conformance finding
  in the same eval report) — a regression here reproduces D1's exact failure mode.
- **TLV tag/order:** swapping `pubkeys`/`use_site_path_overrides` TLV tag values or entry order in
  `tlv.rs:210-260`'s encode side.
- **Origin-override:** `PathDeclPaths::Divergent` write path (`origin_path.rs:117-120`) — e.g. writing paths
  in the wrong per-key order.
- **Long-code:** cap/threshold mutation at `codex32.rs:20-28` (off-by-one on `REGULAR_DATA_SYMBOLS_MAX`).
Each needs the literal-value golden from §3(a)/(b) to RED-prove — the existing self-referential diff test
provably does NOT RED under any of these (same class as item #3).

### 5. Repo + gates + lockstep
**Repo:** `descriptor-mnemonic`, crate `md-codec` (+ `md-cli` if the long-code/TLV fixtures are generated via
CLI). **NO-BUMP** — additive test/fixture-only; `MANIFEST` already documents itself as versioned by
addition-not-mutation (`test_vectors.rs:44-45` "Part-3 additions"), consistent with prior additive rounds.
md `main` now has LIVE branch protection (Cycle I: `["cargo test (ubuntu-latest)","cargo clippy"]`,
`enforce_admins:false`, verified via `gh api`) — a new `tests/wire_golden.rs`-style file is auto-gated.
No manual/GUI/schema lockstep (md-codec has no CLI flag surface change; `md-cli`'s existing `--force-long-code`
flag already exists and is presumably already manual-documented — confirm during SPEC drafting, out of this
recon's scope to verify the manual mirror). Cross-repo companion: none needed (md-codec-internal fixtures).

---

## Item #5 — `payload_bits` mutation gap

**Verbatim:**
> `payload_bits` mutation gap — `word_card_adapter.rs::md1_to_canonical` carrying `bytes.len()*8` instead of
> the exact bit count (the module doc's named hazard) is **not caught**: the unit test asserts only
> `payload_bits <= 8*len`, and md-codec's TLV decoder tolerates <8 trailing pad bits, so every round-trip
> passes. Pin the exact bit count on a non-byte-aligned fixture.

**Distinction established (important for SPEC scoping): this is a pure test-gap, NOT a live bug.**
Current `md1_to_canonical` (`crates/mnemonic-toolkit/src/word_card_adapter.rs:102-110`) correctly computes
`payload_bits: total_bits` from `desc.canonical_payload_bytes()` (`:104`, delegating to md-codec's bit-precise
accessor) — **not** `bytes.len()*8`. The `bytes.len()*8` line the item names (`:88`) is `mk1_to_canonical`'s
computation, which is *correct* for mk1 (byte-aligned by design, per the module doc `:23`). The item is
describing a **hypothetical future regression** to the md1 path, which the current test suite would indeed
fail to catch (see below) — the eval's own item text ("carrying `bytes.len()*8` **instead of**") already
reads as "would carry", consistent with this reading; T3's SPEC should phrase this precisely as a mutation-gap
close, not a bugfix, to avoid a false "fixing a live bug" framing.

### 1. Exact locus
- `md1_to_canonical`: `crates/mnemonic-toolkit/src/word_card_adapter.rs:102-110`. Exact computation:
  `let (bytes, total_bits) = desc.canonical_payload_bytes()...; Ok(CanonicalPayload { … payload_bits: total_bits })`.
- Sibling `mk1_to_canonical`: `:85-94`, `let payload_bits = bytes.len() * 8;` (`:88`) — correct-as-is for mk1.
- Module doc's **named hazard** (quoted verbatim, `:21-28`): *"md1 canonical bytes are bit-precise: the
  descriptor packer returns `(bytes, total_bits)` where `total_bits` is generally NOT a multiple of 8 (the
  final byte carries up to 7 trailing zero-pad bits). `total_bits` is **load-bearing** — it MUST be carried
  verbatim into `wc-codec` and back into `Descriptor::from_canonical_payload_bytes`, never `bytes.len() * 8`."*
  This confirms the item's "module doc's named hazard" framing precisely.
- Source of truth for the true bit count: `md_codec::Descriptor::canonical_payload_bytes()`
  (`descriptor-mnemonic/crates/md-codec/src/encode.rs:71-73`, delegates to `encode_payload`), whose own doc
  (`:54-70`) states the same load-bearing contract independently on the md-codec side.

### 2. Current test coverage
Only test touching `payload_bits` on the md1 path: `md1_canonical_carries_bit_precise_total_bits`
(`word_card_adapter.rs:209-219`):
```rust
assert!(cp.payload_bits <= cp.bytes.len() * 8);
assert!(cp.payload_bits > (cp.bytes.len().saturating_sub(1)) * 8);
```
This pins `payload_bits` into the **last-byte range** `(8*(len-1), 8*len]` — i.e. "the last byte has between
1 and 8 real bits" — not the exact value. A regression to `bytes.len()*8` would still satisfy the first
assert (equality, `<=`) and vacuously the second (since `bytes.len()*8 > 8*(len-1)` always) — **the mutation
survives this test undetected**, confirming the item's claim exactly.

**md-codec's pad-bit tolerance, confirmed:** `decode_payload` (`descriptor-mnemonic/crates/md-codec/src/decode.rs:15-16`)
takes `total_bits` as a hard `BitReader::with_bit_limit` ceiling. The TLV section's decode loop
(`tlv.rs:210-320`) terminates when `remaining_bits() < 5` and then asserts (F-A8, `:317-330`) that any
≤7 leftover bits are all-zero. Since `bytes.len()*8` exceeds the true `total_bits` by at most 7 bits (padding
is always <8 bits), a wrongly-inflated `payload_bits` still (a) lands within the loop's ≤7-bit termination
slack and (b) passes the all-zero trailing-bit check (the extra bits genuinely are zero-pad) — so
`from_canonical_payload_bytes` decodes successfully and reconstructs the identical `Descriptor`. This is why
"every round-trip passes": encode and decode are called with the SAME wrong `payload_bits` (symmetric bug),
and md-codec's own leniency absorbs the discrepancy silently. Confirmed exactly as claimed.

### 3. Golden/fixture feasibility
**No new fixture needed.** The existing `MD1` 3-chunk fixture (`word_card_adapter.rs:194-198`, from
`mnemonic bundle --network mainnet --template bip84` over the `abandon×11 about` seed) is **already**
non-byte-aligned per the current test's own range assertion (`:214-218`) — it just isn't pinned to its exact
value. Two ways to obtain the exact value as an independent-of-this-adapter oracle:
  (a) **`md bytecode <MD1-chunks> --json`** (`descriptor-mnemonic/crates/md-cli/src/cmd/bytecode.rs:14,27`) —
      calls `encode_payload` directly and reports `"payload_bits": bit_len` in JSON. Not fully independent of
      md-codec's own encoder, but is an independent CLI invocation path from the toolkit adapter under test —
      run once, freeze the literal integer as `assert_eq!(cp.payload_bits, <N>)`.
  (b) A hand-derived bit count from the descriptor's known structure (header + path_decl + use_site_path +
      tree + tlv widths per SPEC) — more rigorous but unnecessary extra work for a single scalar; (a) is
      sufficient and matches the "generated once from the shipped binary and frozen" methodology item #3
      already establishes as this sub-cycle's accepted oracle standard.

### 4. Named RED mutation
Confirmed: replacing `:108`'s `payload_bits: total_bits` with `payload_bits: bytes.len() * 8` is the exact
named mutation. Under it: `bytes.len()*8` is *always* `>= total_bits` (0-7 bits higher), so both current
assertions (`<=`, `>`) stay green — need the frozen exact-value assertion from §3 to RED-prove.

### 5. Overlap with in-flight T2 — no file-level conflict for this item
T2-a's toolkit footprint (currently uncommitted in the working tree) is `crates/mnemonic-toolkit/src/main.rs`
lines 42-48 (new `#[cfg(test)] mod prop_repair_never_wrong;` inserted immediately after `mod wordlists;` at
`:41`) + new file `crates/mnemonic-toolkit/src/prop_repair_never_wrong.rs`. `word_card_adapter` is declared
at `main.rs:40` — **adjacent** to T2's insertion point but T3's item #5 fix only edits inside
`word_card_adapter.rs` itself (the `md1_to_canonical` fn + its `#[cfg(test)] mod tests` block, both well
below any `main.rs` line T2 touches) — **T3 item #5 needs zero `main.rs` edits**, so there is no textual
merge-conflict risk with T2's `main.rs` hunk. (Item #3's wire golden lives in a different crate entirely —
also zero conflict.) See the cross-cutting §Overlap section below for the sequencing recommendation anyway.

---

## Cross-cutting: is any item NOT actually a wire-golden gap?

None of #3/#4/#5 are mis-scoped as *test-gap* findings — all three are confirmed, present-tense gaps in the
"round-trip-only, no independent frozen oracle" sense the eval names. The one caveat: **item #4's "no
≥2-chunk vector" clause is RESOLVED** by today's BIP-alignment ship (`wsh_sortedmulti_2chunk`) — T3's SPEC
must not re-scope that clause; only long-code + pubkey/wallet-policy-mode + use-site-override +
origin-override remain open for #4 (§Item 4 above). Item #5 is confirmed a pure mutation-gap (not a live bug)
— the SPEC should say so explicitly to avoid an incorrect "bugfix" framing that would trigger a different
(funds-safety) review track than a test-hardening one.

## Overlap with T2 — repos, sequencing

- **T2's repos:** toolkit (#6, `main.rs` + new `prop_repair_never_wrong.rs`), md-codec (#7, new
  `tests/bch_exhaustive_sweep.rs` + rewrite `tests/parity_smoke.rs`), mk-codec (#8, new proptest + fuzz
  target in `crates/mk-codec/src/string_layer/bch.rs`'s test surface).
- **T3's repos:** toolkit (#3, new `crates/wc-codec/tests/wire_golden.rs`), md-codec (#4, extend
  `src/test_vectors.rs::MANIFEST` + new frozen-hex assertions), toolkit again (#5, edit
  `word_card_adapter.rs`'s test module). **mk-codec is untouched by T3** — zero overlap there, T3's mk-codec
  scope is empty, no sequencing concern in that repo at all.
- **Overlap repos: toolkit and md-codec.** File-level conflict risk is **low in both** — T2 and T3 touch
  disjoint files in each repo (toolkit: `main.rs`+`prop_repair_never_wrong.rs` vs. `wc-codec/tests/wire_golden.rs`+
  `word_card_adapter.rs`; md-codec: `bch_exhaustive_sweep.rs`+`parity_smoke.rs` vs. `test_vectors.rs`+new
  golden file). `design/FOLLOWUPS.md` is the one genuinely shared file (both cycles will append entries) —
  trivial line-level merge, not a functional conflict.
- **Sequencing recommendation: let T2 commit/ship first anyway, per the standing one-cycle-at-a-time R0
  discipline** (CLAUDE.md's per-phase gate + this session's practice of shipping each eval cycle before the
  next) — not because of textual conflict risk (there is essentially none), but because: (a) T2 is
  **mid-R0-loop, uncommitted, working tree dirty** (SPEC round-1 verdict OPEN — 1 Critical/4 Important,
  folded, awaiting round-2 re-dispatch; a partial T2-a implementer pass already ran and hit/resolved a
  visibility blocker) — starting T3 implementation against this same dirty tree risks attribution confusion
  between the two cycles' diffs at commit time; (b) both land in the SAME two repos' `design/FOLLOWUPS.md`
  and both want a clean `cargo test --workspace` / `cargo test -p md-codec` baseline to diff against for
  their own R0 per-phase reviews — reviewing T3's diff is cleaner against a tree where T2's changes are
  already a committed, stable baseline rather than untracked working-tree noise. **T3 SPEC-authoring
  (this recon → a SPEC doc) can proceed now in parallel** (no code touched); **T3 implementation should wait
  for T2's commit** in the shared toolkit/md-codec repos specifically. No wait needed for any mk-codec work
  (T3 has none).

---

## Proposed T3 SPEC skeleton

**Repos:** `mnemonic-toolkit` (#3 wc-codec wire golden, #5 payload_bits pin), `descriptor-mnemonic` md-codec
(#4 corpus TLV/long-code/origin-override goldens). No mk-codec leg (unlike T2's 3-repo shape, T3 is 2-repo).

### T3-a (#3) — `crates/wc-codec/tests/wire_golden.rs`
- **Deliverable:** new integration test file, public `wc_codec::{encode,decode,raid_encode,raid_reconstruct}`
  API only (no visibility blocker, unlike T2-a). Frozen `const` word-list(s) for: (i) one `mk1`-kind
  `(payload, payload_bits, m, t, u)` tuple, (ii) one `md1`-kind tuple (non-byte-aligned `payload_bits`,
  reusing the toolkit's existing `abandon×11 about` fixtures for input determinism), (iii) one `n=3, r=2`
  RAID array. Provenance comment on each: wc-codec version + toolkit git SHA + exact generating call.
- **Acceptance:** each golden RED-proven under ≥1 of the item's named mutation classes (field-order swap /
  CRC context / tag placement / RS parity order) via a scratch mutation, reported in the R0 review. Full
  `cargo test --workspace` (toolkit) stays green. NO-BUMP.
- **Independent oracle caveat:** document in the SPEC (as this recon does) that the oracle is
  "frozen-historical, not externally-independent" — matches the eval's own prescription; do not over-promise
  a from-first-principles derivation.

### T3-b (#4) — md-codec corpus: long-code + wallet-policy + use-site-override + origin-override
- **Deliverable:** extend `crates/md-codec/src/test_vectors.rs::MANIFEST` with ≥4 new entries covering the
  4 still-open gaps (do NOT re-add ≥2-chunk — resolved). PLUS a new `crates/md-codec/tests/wire_golden.rs`
  (or extend `vector_corpus.rs`) that asserts literal frozen `.bytes.hex` content directly (not via the
  self-referential `md vectors`-regenerate-and-diff path) for at least the TLV-bearing + origin-override
  entries, to actually close the "encoder and fixture move together" critique — extending `MANIFEST` alone
  does not.
- **Acceptance:** each new vector RED-proven under its named mutation (TLV tag/order swap, `Divergent`
  per-key order swap, long-code cap off-by-one). Full `cargo test -p md-codec` + `cargo test -p md-cli` green.
  `vectors_output_matches_committed_corpus` still passes (regenerate + commit the new fixture files).
  Confirm during drafting whether `--force-long-code` is already manual-documented (out of this recon's
  scope — flag as an open question for the SPEC author, not assumed either way).

### T3-c (#5) — `word_card_adapter.rs::md1_canonical_carries_bit_precise_total_bits` → exact-value pin
- **Deliverable:** replace (or add alongside) the current range assertion (`:217-218`) with
  `assert_eq!(cp.payload_bits, <N>)` where `<N>` is obtained once via `md bytecode <MD1> --json` and frozen
  with a provenance comment citing the command + toolkit git SHA.
- **Acceptance:** RED-proven under the exact named mutation (`total_bits` → `bytes.len() * 8` at
  `word_card_adapter.rs:108`). `cargo test -p mnemonic-toolkit` (or `--workspace`) green. NO-BUMP, and
  explicitly phrase the SPEC/commit as a **test-hardening mutation-gap close**, not a bugfix (see §Item 5
  distinction above) to keep it out of a funds-safety-bug review track it doesn't belong in.

### Phasing
Two legs, parallelizable across repos once T2 has committed in both: **T3-a + T3-c together** (both toolkit,
touch disjoint files — `crates/wc-codec/tests/` vs. `crates/mnemonic-toolkit/src/word_card_adapter.rs` — one
implementer or two in parallel, no shared-file risk even with each other) → **T3-b** (md-codec, independent
repo, can run fully in parallel with T3-a/T3-c). Per-leg + post-impl R0 per CLAUDE.md convention (same
pattern as T2's spec). Gate T3-a/T3-c's implementation start on T2's toolkit commit landing; gate T3-b's on
T2's md-codec commit landing (per §Overlap — not a hard technical blocker, a discipline/attribution one).
