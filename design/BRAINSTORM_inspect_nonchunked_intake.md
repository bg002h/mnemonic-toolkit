# BRAINSTORM — `mnemonic inspect` non-chunked single-string md1 intake

**FOLLOWUP:** `toolkit-inspect-nonchunked-md1-intake-gap` (`design/FOLLOWUPS.md:33-35`).
**Source SHA:** `a528eba5` (`origin/master`, grep-verified; re-grep at plan time — citations decay every merge).
**Toolkit version at recon:** `v0.88.0`; md-codec vendored + pinned `0.42.0` (`crates/mnemonic-toolkit/Cargo.toml:34`).
**Status:** brainstorm — pre-R0. NO code. Feeds `design/SPEC_inspect_nonchunked_intake.md`.

---

## 1. The problem

`mnemonic inspect --md1 <STRING>` (and the positional form) decodes an md1 card by
**chunk reassembly only**. A plain **non-chunked single-string** md1 — the bare
output of `md encode 'wpkh(@0/<0;1>/*)'` — is rejected with `unsupported version 2`
/ **exit 3**, even though it is a perfectly valid canonical md1 wire form and
`md inspect <that-string>` accepts it. This is a **cross-binary asymmetry** and a
pure usability gap; it is a **pre-existing** limitation, NOT introduced by the
pathless partial-decode cycle (task #12 / v0.88.0 P2.3 deliberately left intake
untouched — see the in-source note at `inspect.rs:239-241`).

### 1.1 Exactly where and why it fails today

Intake site (`crates/mnemonic-toolkit/src/cmd/inspect.rs:242-245`):

```rust
CardKind::Md1 => Ok(InspectPayload::Md1(md_codec::reassemble_with_opts(
    chunks,
    md_codec::DecodeOpts::partial(),
)?)),
```

`reassemble_with_opts` (`vendor/md-codec/src/chunk.rs:328`) unwraps each string via
codex32 (`unwrap_string`, BCH-verified) then calls `ChunkHeader::read`
(`chunk.rs:68`), which reads a **4-bit version** field from the first 5-bit
symbol. But a non-chunked single-payload md1's first symbol is laid out
`[divergent][v3][v2][v1][v0]` (`vendor/md-codec/src/header.rs:1-10`), NOT the chunk
header's `[v3][v2][v1][v0][chunked]`. For the canonical `version=4,
divergent=false` first symbol `0b00100`, `ChunkHeader::read`'s 4-bit version read
lands on `0b0010 = 2`, so it returns `Error::WireVersionMismatch { got: 2 }`
(`chunk.rs:70-72`; the misread is documented in the codec's own test
`header.rs:100-110`).

That error is intercepted by the toolkit's `From<md_codec::Error>` and remapped to
`ToolkitError::FutureFormat { detail: "unsupported version 2" }`
(`crates/mnemonic-toolkit/src/error.rs:1054-1057`), whose `exit_code()` is **3**
(`error.rs:586-587`). Hence the FOLLOWUP's exact symptom.

### 1.2 The intake discriminator (KEY FINDING — clean, in-band, no heuristics)

md-codec already ships the correct dispatching entry point:
`decode_md1_string_with_opts(s, opts)` (`vendor/md-codec/src/decode.rs:187-196`).
It reads a **structural in-band bit** — the *chunked-flag* = bit 0 (LSB) of the
first 5-bit symbol = **bit 3 of byte 0** — and routes:

```rust
let chunked_flag = bytes.first().map(|b| (b >> 3) & 0x01).unwrap_or(0);
if chunked_flag == 1 {
    return crate::chunk::reassemble_with_opts(&[s], opts);  // chunked-of-1
}
decode_payload_with_opts(&bytes, symbol_aligned_bit_count, opts)  // single-payload
```

This is a **deterministic structural discriminator, not a try-one-then-the-other
heuristic.** The usable single-payload version set `{4, 8, 12}` is all-even, so
every valid single-payload string has first-symbol LSB `= 0`; the chunk header
always writes `chunked = 1` (`chunk.rs:53`). The two forms are unambiguous at the
bit level. Crucially, `unwrap_string` **verifies the codex32 BCH checksum before
the flag is read** (`vendor/md-codec/src/codex32.rs:183-184`), so a corrupted
first symbol that would flip the flag fails the checksum → reject; the flag can
never re-route a corrupted card.

`decode_md1_string_with_opts` also **threads `opts`** to whichever layer performs
the origin check, so `DecodeOpts::partial()` reaches `decode_payload_with_opts`
for the non-chunked case — the dead-card partial path composes for free.

### 1.3 Precedent already in the toolkit

The toolkit already handles non-chunked md1 on the **repair** path:
`repair.rs::is_non_chunked_md1` (`crates/mnemonic-toolkit/src/repair.rs:736-741`)
reads the identical chunked-flag bit, and `md_codec::decode_with_correction`
auto-dispatches a single string to `decode_md1_string`
(`vendor/md-codec/src/chunk.rs:643-661`). So the codec + toolkit already treat the
non-chunked single string as first-class on the correction path — this cycle
extends the same, already-blessed discriminator to the *inspect* read path.

### 1.4 Evidence the gap is real (test scaffolding)

`crates/mnemonic-toolkit/tests/cli_inspect.rs:211-239` holds a corpus of frozen
**single-string** md1 fixtures but must **re-chunk** them (`rechunk()`, line 233:
`decode_md1_string` → `split` → `reassemble`) precisely "so the toolkit's chunked
`decode_card` (reassemble-only) accepts it" (comment at line 213). That workaround
is the gap made visible: the single-string form cannot be fed directly today.

---

## 2. Scope question: inspect only, or also verify-bundle?

The FOLLOWUP names both `inspect.rs:207` [now `:242`] AND "the verify-bundle
SUPPLIED-card decode sites". Recon says: **inspect only for this cycle; keep
verify-bundle as remaining FOLLOWUP scope.** Reasons:

1. **verify-bundle's single-sig template path compares raw STRINGS, not decoded
   descriptors:** `verify_bundle.rs:696` is `let md1_match = expected.md1 ==
   args.md1;`. `expected.md1` is always toolkit-synthesized **chunk-form** (via
   `md_codec::chunk::split`, which the toolkit's own emitters always use — even a
   tiny template becomes a single *chunked-of-1* string, never a non-chunked
   single-payload string). A supplied non-chunked string can never string-equal a
   chunk-form expected, so **broadening intake alone does not make a non-chunked
   template md1 verify** — you would additionally have to canonicalize the
   comparison (compare decoded descriptors / content-ids rather than bytes). That
   is a funds-sensitive wire-semantics change with its own review surface.
2. **The realistic non-chunked source is `md encode`, and the realistic user need
   is `inspect`** (describe a card), not `verify-bundle` (which compares a
   *toolkit-produced* bundle). A user verifying a toolkit bundle already holds the
   chunk-form cards the toolkit emitted.
3. **`inspect` is a pure describe surface** — no comparison, no funds-moving
   decision — so broadening its intake is the lowest-risk, highest-value slice.
4. The verify-bundle SUPPLIED decode sites that *would* need updating span
   different semantics — the strict routing probe (`:388`
   `md_codec::chunk::reassemble`), the partial-indices probe (`md1_partial.rs:60`
   `reassemble_with_opts`), and the multisig supplied decode (`:2510`
   `reassemble_with_opts`) — so a coherent verify-bundle change is materially
   larger and should be its own cycle if pursued.

**Recommendation:** ship inspect-only; leave a scoped residual on the FOLLOWUP for
verify-bundle non-chunked intake (with the string-compare entanglement noted as
the blocking design question).

---

## 3. Options

### Option A — dispatch on `chunks.len()` in `decode_card` (RECOMMENDED)

In `inspect.rs::decode_card`'s `CardKind::Md1` arm:

```rust
CardKind::Md1 => {
    let d = if chunks.len() == 1 {
        // Single supply: md-codec auto-dispatches non-chunked (single-payload)
        // vs chunked-of-1 via the in-band chunked-flag bit; opts thread through.
        md_codec::decode_md1_string_with_opts(chunks[0], md_codec::DecodeOpts::partial())?
    } else {
        md_codec::reassemble_with_opts(chunks, md_codec::DecodeOpts::partial())?
    };
    Ok(InspectPayload::Md1(d))
}
```

- **Zero behavioral change for chunked-of-1:** `decode_md1_string_with_opts`
  reads `chunked_flag == 1` and routes to `reassemble_with_opts(&[s], opts)` —
  byte-identical to today's direct `reassemble_with_opts(chunks, opts)` for a
  1-element chunked set.
- **Zero behavioral change for multi-chunk:** `len > 1` keeps
  `reassemble_with_opts` verbatim.
- **Only new acceptance:** a valid non-chunked single-payload md1
  (chunked_flag == 0) now decodes via `decode_payload_with_opts` instead of being
  mis-rejected. Partial-decode threads through unchanged (dead card → exit 4).
- Smallest, most surgical diff; reuses the already-blessed codec discriminator.

**Tradeoff:** a diagnostic change — a codex32-valid-but-structurally-invalid
*same-version* single string now surfaces `decode_payload`'s specific error
instead of the misleading `unsupported version 2 / exit 3`. This is an
improvement, not a regression (genuine future-version single strings still exit 3
— `Header::read` yields `WireVersionMismatch { got: 8|12 }` for version ≠ 4,
`header.rs:42-43` → same `From` → exit 3). Enumerated in the SPEC's funds analysis.

### Option B — always route single supply through `decode_md1_string_with_opts`, multi through `reassemble`

Functionally identical to A (for `len == 1` the two are the same call). No
advantage; A's explicit `chunks.len()` branch reads clearer at the call site.

### Option C — try `decode_md1_string` then fall back to `reassemble` (REJECTED)

A try/catch fallback is exactly the anti-pattern to avoid on a funds-adjacent
path: a fallback can **mask a real error** (a genuine reassemble failure gets
retried as a single string, or vice-versa) and muddies which reject fired. The
in-band flag makes a fallback unnecessary. Rejected.

### Option D — do nothing / keep the `rechunk()` test workaround (REJECTED)

Leaves the cross-binary asymmetry and the confusing exit-3 diagnostic. The fix is
tiny and the discriminator is already proven; not fixing costs more in user
confusion than the change costs to ship.

---

## 4. Recommended approach

**Option A**, **inspect-only scope**, MINOR bump (`v0.89.0`). Dispatch on
`chunks.len()` in `decode_card`; single supply → `decode_md1_string_with_opts`
(auto-dispatch), multi → `reassemble_with_opts` (unchanged). Partial-decode /
exit-4 dead-card behavior is inherited for free because `opts` threads to
`decode_payload_with_opts`. No clap-flag change, no `--json` wire-shape change →
**no schema_mirror trigger, no GUI wire drift, no manual flag-mirror lockstep**
(optional non-gated inspect-chapter note that the single-string form is now
accepted). md/mk/ms/wc **NO-BUMP** (uses existing md-codec 0.42.0 public API).

---

## 5. Funds-safety posture (full analysis in SPEC §5)

This is an **intake dispatch** change on a funds-critical decode path. The verdict
is **SAFE** under five invariants: (1) the discriminator is structural + in-band,
never a fallback; (2) codex32 BCH is verified before the flag read, so corruption
cannot re-route; (3) chunked cards keep the unconditional cross-chunk content-id
oracle; (4) `opts = partial` threads identically so a non-chunked dead card
exits 4 consistently and `EmptyOriginOverride` stays fatal-in-partial; (5) no
previously-rejected malformed input becomes accepted — only the canonical
non-chunked wire form, which then runs `decode_payload`'s full validation
gauntlet. A single non-chunked string is self-contained and correctly has **no**
cross-chunk oracle (expected — there is only one payload to check).

---

## 6. Open questions for R0 (mirrored in SPEC §9)

1. Confirm **inspect-only** scope vs folding in verify-bundle now, given the
   `expected.md1 == args.md1` string-compare entanglement (`:696`).
2. Confirm Option A's `chunks.len() == 1` branch over a codec-level unification.
3. Is the diagnostic/exit-code change (codex32-valid-but-invalid same-version
   single string: `exit 3 "unsupported version 2"` → `decode_payload`'s error,
   possibly exit 2) acceptable? (Genuine future-version single strings keep
   exit 3.)
4. Confirm no manual **lockstep** is required (behavioral broadening, no flag
   surface change) and whether an optional inspect-chapter prose note is wanted.
