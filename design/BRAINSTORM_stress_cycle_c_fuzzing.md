# BRAINSTORM — stress Cycle C: cargo-fuzz malformed-input fuzzing (md1 / mk1 / ms1)

Status: R3 **GREEN (0C/0I)** — cleared for implementation. 2026-06-11.
Reviews (persisted verbatim):
design/agent-reports/cycle-c-fuzzing-r0-round1-review.md (RED: 3C/6I —
folded; markers `[C1]`…`[M8]`), …-round2-review.md (YELLOW: 0C/1I —
folded; marker `[I3-residual]` + 3 minors), and …-round3-review.md
(GREEN: 0C/0I; cosmetic 16-vs-18 count nit folded).
Repos + HEADs at write time: mnemonic-toolkit `e33c147` (master),
descriptor-mnemonic `cdd8501` (main), mnemonic-key (main), mnemonic-secret
(main). Program context: Cycle C of the 6-cycle stress program (A = toolkit
backup→restore proptest @ 9d3da6c; B = md-codec proptest expansion @
3ec324c — both found real bugs run #1).

## Problem statement (charter)

Malformed-input robustness today rests on proptest properties driven by
UNIFORM random inputs (md-codec P3 `proptest_roundtrip.rs:32-42`; mk-codec
P2a–c `tests/proptest_roundtrip.rs:24/:30/:39`; ms-codec none explicit) —
no COVERAGE-GUIDED exploration, no structured corpora seeded from valid
cards, no persistent corpus that ratchets. The charter: cargo-fuzz
bit-flip/truncate md1/mk1/ms1 (+ descriptors, now descoped — see below) →
assert never-panic, clean-error, no-secret-leak. NEW fuzz infra (none
exists in any repo).

**SCOPE CHANGE [C2]: the toolkit `descriptor_parse` target is DESCOPED to
a FOLLOWUP.** `parse_descriptor` is bin-private by the locked Option C
crate shape (`SPEC_secret_memory_hygiene_v0_9_B.md` §4 P2; declared
`mod parse_descriptor;` at main.rs:23, deliberately not in lib.rs), and
its transitive compile closure is ~35 modules / 56k lines. The honest hook
is a `#[cfg(fuzzing)]` bulk module mount in lib.rs + a
`[lints.rust] unexpected_cfgs` check-cfg declaration (cfg(fuzzing) trips
`unexpected_cfgs` on 1.85.0 AND nightly, and toolkit CI runs clippy
`-D warnings`) + replicating the root `[patch.crates-io]` miniscript-git
pin in the fuzz workspace (Cargo patches do not cross workspace
boundaries; the closure uses `Terminal::SortedMultiA` @
parse_descriptor.rs:595 which exists only in the patched rev) — i.e. real
library-surface changes deserving their own mini-R0. FOLLOWUP
`toolkit-descriptor-fuzz-target` files the full design; Cycle C ships
**9 targets across the 3 codec repos**, NO-BUMP everywhere.

## Recon facts (round-1 R0 re-verified every load-bearing claim empirically)

- **No fuzz/ anywhere; stable pins `1.85.0`** (descriptor-mnemonic,
  mnemonic-secret, mnemonic-toolkit; mnemonic-key has no pin file and its
  fmt-job comment mandates no ROOT rust-toolchain.toml — a fuzz/-scoped
  one does not violate it; cite that comment in the mk fuzz dir [M5]).
- **cargo-fuzz needs nightly AND must be installed with nightly active**:
  `cargo install cargo-fuzz` under the 1.85.0 pin FAILS
  (cargo-platform@0.3.3 needs rustc 1.91); `cargo +nightly install
  cargo-fuzz --locked` works (v0.13.2 proven locally). [I5]
- **Feasibility proven end-to-end locally**: scratch md-codec fuzz target
  built on the installed nightly, ran 6.5M execs in 46s with the
  fixed-point oracle live, and FOUND a planted panic in <1s with a crash
  artifact. Local validation is fully possible; only the GitHub-Actions
  wiring is CI-only surface.
- **Workspace isolation proven**: a nested fuzz/ dir with its own
  `[workspace]` is completely ignored by root `cargo fmt --all --check`,
  `cargo test --workspace`, `clippy --workspace --all-targets`
  (empirically tested with deliberately misformatted code). All repos use
  explicit member lists; no CI job globs all Cargo.toml.
- **Dependency-resolution drift is real**: the scratch fuzz workspace
  resolved miniscript 13.1.0 while md-codec's root Cargo.lock pins 13.0.0
  → every fuzz/ MUST commit its own `fuzz/Cargo.lock`, aligned via
  `cargo update --precise` with the root lock's shared-dep versions. [C3]
- **Entry points (citations re-verified at round 1):**
  - md-codec: `decode_md1_string` (decode.rs:79), `decode_payload`
    (decode.rs:15), `chunk::reassemble` (chunk.rs:305),
    `chunk::decode_with_correction` (chunk.rs:492).
  - mk-codec: `decode` (key_card.rs:115, `&[&str]`); re-encode via
    `encode_with_chunk_set_id` (key_card.rs:110) — NOT `encode`
    (key_card.rs:95-100 draws a fresh CSPRNG chunk_set_id → nondeterminism
    breaks crash reproducibility). [I2]
  - ms-codec: `decode` (decode.rs:42), `decode_with_correction`
    (decode.rs:221) [I4], `combine_shares` (shares.rs:186), `inspect`.
  - `CorrectionDetail` shapes: md chunk.rs:405-415 {chunk_index, position,
    was, now}; ms decode.rs:120-128 {position, was, now}.
- **ms-codec error Display ECHOES INPUT** [C1 — Cycle C's first finding,
  found at review time before any fuzzing]: `error.rs:118` Displays
  `Codex32(e)` as `{:?}`, and codex32-0.1.0's
  `InvalidChecksum { string: String }` carries the FULL input string —
  so any checksum-failing share echoes the whole (secret) share text into
  the error. `WrongHrp { got }` (error.rs:15-17/:122) echoes
  attacker-controlled text, and a data-char→`1` mutation shifts the bech32
  separator so the "HRP" contains a secret prefix. The toolkit's v0.53.4
  friendly-mapper withholding exists precisely because of this layer
  behavior; ms-codec-side it is now filed as FOLLOWUP
  `ms-codec-error-display-echoes-input` (mnemonic-secret repo, companion
  in toolkit) — fix = bound/withhold `InvalidChecksum.string` +
  `WrongHrp.got` at the ms-codec boundary, its own cycle.

## Proposed design

### Placement & layout (per-repo fuzz workspaces) — 3 codec repos

```
<repo>/fuzz/
  Cargo.toml            (own [workspace]; path-dep on the codec crate)
  Cargo.lock            (COMMITTED, aligned with root lock [C3])
  rust-toolchain.toml   (pinned nightly-YYYY-MM-DD, scoped to fuzz/ [M5])
  fuzz_targets/*.rs
  corpus/<target>/*     (committed seed corpus — cargo-fuzz default path [M1])
  dictionaries/bech32.dict
```

NO-BUMP in every repo (infra + tests only; zero library code changes).
Pinned nightly + FOLLOWUP `fuzz-nightly-quarterly-bump` (constellation-
wide entry, toolkit repo) to refresh quarterly. [M5]

### Targets (9 across 3 repos)

Structured multi-chunk inputs use a **sentinel-byte splitter**: split the
fuzz input on `\n` (outside the bech32 alphabet) into ≤8 chunks
(truncate excess) — libFuzzer insert/delete moves ONE boundary locally
instead of re-shearing all of them. [M2]

**descriptor-mnemonic (4):**
- `md1_decode_string`: bytes → utf8 (lossy ok [M7]) → `decode_md1_string`;
  on Ok, fixed-point oracle (re-encode → decode → equal).
- `md1_reassemble`: sentinel splitter → `reassemble`; fixed-point oracle
  via `split` on success.
- `md1_decode_with_correction`: sentinel splitter →
  `decode_with_correction`; apply-details idempotence oracle [I1]: apply
  each detail's `now` at (chunk_index, position) in the input data-parts,
  re-run, assert equal Descriptor + EMPTY details. **Coordinate**: md
  `CorrectionDetail.position` is a **post-HRP-and-separator** offset into
  the data-part of chunk `chunk_index` (chunk.rs:406-410) — apply `now`
  past the `md1` prefix, not at the raw chunk-string index.
- `md1_decode_payload`: **fuzz-chosen total_bits, CLAMPED** [I3/I3-residual]
  — first 2 bytes = LE candidate, `total_bits = candidate.min(remainder.len()*8)`,
  remainder = payload → `decode_payload`; fixed-point oracle on success.
  This fuzzes every partial-byte trailing-bit count `0..=len*8` (the real
  blind spot P3 leaves by pinning `total_bits = len*8`), and genuine short
  reads return `Err(BitStreamTruncated)` (bitstream.rs:129-133) — a clean
  error, not a finding. **The candidate MUST be clamped**: round 2 proved
  raw `decode_payload(bytes, total_bits)` with `total_bits > len*8` hits
  `BitReader::with_bit_limit`'s `debug_assert!(bit_limit <= bytes.len()*8)`
  (bitstream.rs:114), and cargo-fuzz builds release-WITH-debug-assertions
  by default → an unclamped prefix ABORTS vacuously on ~the first exec.
  There is no `>len*8` "validation" path to exercise via the raw entry
  point; exercising the assert would need a wrapper mapping it to `Err`,
  out of scope.

**mnemonic-secret (3):**
- `ms1_decode`: bytes → string → `decode` + `decode_with_correction`
  (with apply-details idempotence [I1]: ms `CorrectionDetail` is
  `{position, was, now}` — single data-part, position-only, NO
  `chunk_index`; `position` is the post-HRP-and-separator offset past the
  `ms1` prefix, decode.rs:121-123) + `inspect`; fixed-point oracle
  via `encode(tag, &payload)` on success.
- `ms1_combine`: sentinel splitter (2..=8 shares) → `combine_shares`;
  on Ok, re-encode the (Tag, Payload) via `encode` (encoder symmetry
  rule encode.rs:16-25 makes decode-accepted tags re-encodable).
- `ms1_no_secret_leak` [C1 fold]: embed a FIXED known share-set; fuzz
  input selects mutations (bit-flips/truncations/case-flips at
  fuzz-chosen positions) of the valid set; call decode/inspect/combine;
  on Err, scan BOTH `format!("{e}")` AND `format!("{e:?}")` [M3] for any
  ≥8-char window of any share's data-part — WITH a documented,
  variant-matched exclusion set for the KNOWN echo paths
  (`matches!(e, Error::Codex32(_) | Error::WrongHrp { .. })` → skip),
  each exclusion line citing FOLLOWUP
  `ms-codec-error-display-echoes-input`; the set SHRINKS as fixes land
  (when the FOLLOWUP ships, delete the exclusion → the oracle then
  guards the fix forever). **Exclusion minimality** (round 2): of the 16
  `ms_codec::Error` variants, ONLY `Codex32(_)` (Debug-wraps codex32's
  full-string `InvalidChecksum`/`MismatchedHrp`/`MismatchedId`) and
  `WrongHrp{got}` (echoes the observed HRP, which a data-char→`1` mutation
  can stretch into a long secret prefix) can emit a ≥8-char contiguous
  input echo; every other variant carries at most a `[u8;4]` tag (4 chars)
  or a single char — below the 8-char window — so the two-variant
  exclusion is provably minimal. **Non-vacuous from day 1**: the
  corrected-then-decode path in `decode_with_correction`, `inspect`'s
  non-table-tag path, and `combine`'s native rejections all reach
  non-excluded variants the scan genuinely guards now; it becomes the
  regression gate for the two excluded paths once the FOLLOWUP ships.

**mnemonic-key (2):**
- `mk1_decode`: sentinel splitter → `mk_codec::decode`; fixed-point via
  `encode_with_chunk_set_id(&card, FIXED_CSI)` [I2].
- `mk1_decode_single`: whole input as one string → `decode(&[s])`; same
  fixed-point.

### In-target oracles

1. **Never-panic / clean-error**: implicit (panic/abort/OOM = libFuzzer
   failure).
2. **Decode→re-encode fixed-point** (all decode targets): when decode
   succeeds, re-encode and decode again; assert equality. **Re-encode
   `Err` on a decode-accepted value is a REAL FINDING — panic in-target,
   never swallow** [I6] (it is the decode/encode-asymmetry class the
   charter targets).
3. **Apply-details idempotence** (correction targets) [I1]: as specified
   per-target above (replaces the round-0 "correction honesty" wording,
   which did not typecheck — both decode_with_correction functions return
   the decoded value, not corrected strings).
4. **No-secret-leak** (ms1_no_secret_leak): as above, Display+Debug,
   ≥8-char window (40 bits over the 32-symbol alphabet — false-positive
   odds negligible [M3]), documented shrinking exclusion set.

### Seed corpora + dictionary

- `corpus/<target>/` (cargo-fuzz default [M1]) committed, generated by a
  per-repo `gen-corpus` test/bin: valid single- and multi-chunk cards from
  existing test vectors + generators (md: Cycle-B strategies; mk: vectors;
  ms: `encode_shares` outputs), plus truncated/bit-flipped variants.
  **`gen-corpus` MUST assert every committed valid-class seed passes the
  SAME split-then-call the target uses** [I6/round-2 minor] — round 1
  proved doc-harvested strings silently rot (one in-repo md1 doc string no
  longer decodes). For single-string targets the gate is "raw bytes decode
  Ok"; for the four **splitter** targets (md1_reassemble,
  md1_decode_with_correction, ms1_combine, mk1_decode) the seed is
  `chunk0\nchunk1\n…` and the gate is "split on `\n` → call the target's
  entry (e.g. `reassemble(&parts)`) is Ok". gen-corpus joins chunks
  **between** only (no trailing `\n`) so a single-chunk seed doesn't gain
  an empty trailing `""` part. Seeds are small (≤ a few hundred bytes);
  total corpus ≤ a few hundred KB per repo — no bloat concern.
- `dictionaries/bech32.dict`: 32-char charset tokens, HRPs
  (`md1`/`mk1`/`ms1`), separator, common header symbols.

### CI integration (per repo): `fuzz-smoke.yml`

- **Compile gate** (every push/PR touching `fuzz/**` OR the codec src
  paths): nightly toolchain (dtolnay/rust-toolchain pinned to the fuzz
  nightly) + cargo-fuzz via `cargo +nightly install cargo-fuzz --locked`
  or a prebuilt-binary action (taiki-e/install-action) [I5] +
  `cargo fuzz build`. Catches API drift early (the lagging-indicator
  lesson from the GUI schema mirror).
- **Smoke run** (cron daily + `workflow_dispatch` [M8] — NOT every push):
  `cargo fuzz run <target> -- -max_total_time=60` per target
  (~10-15 min total, off the push-CI critical path). Crash artifacts
  uploaded via actions/upload-artifact@v5. Scheduled workflows run on the
  default branch only and auto-disable after 60 days of inactivity —
  workflow_dispatch is the recovery lever. [M8]
- Existing CI untouched: path filters verified non-overlapping (toolkit
  rust.yml is crates/**-scoped; sibling-pin-check scans only
  `cargo install --git --tag` lines). ms note [M4]: fuzz-smoke.yml will
  be the FIRST CI ever exercising ms-codec in that repo (rust.yml is
  ms-cli-scoped) — say so in the workflow header.

### Phasing — hard per-repo sub-gates (round-1 Q1 answer adopted)

Phase = repo, order **md → ms → mk**; each phase lands fuzz dir + targets
+ bring-up proof + smoke CI green BEFORE the next opens. Each phase is
standalone-valuable; stopping after any phase is a pre-authorized clean
exit. (Toolkit = descoped phase 4, FOLLOWUP `toolkit-descriptor-fuzz-
target`, own mini-R0 when picked up.)

### Bring-up proof (anti-vacuity, per phase)

- Plant a temporary panic reachable from the target (scratch worktree;
  e.g. a `debug_assert!` keyed to a corpus-adjacent input) and demonstrate
  the fuzzer finds it within the smoke budget; record in the phase notes.
- Run each target locally ≥10 min (or ~10^7 execs) before shipping;
  findings → FOLLOWUPs (fix only if funds-safety dictates, per charter;
  exception: a trivial-and-critical decode-hot-path panic may be fixed
  in-cycle with its own mini-R0).

### Scope exclusions

- Toolkit descriptor door (descoped → FOLLOWUP, above).
- Fixing `ms-codec-error-display-echoes-input` (filed, own cycle).
- GUI surfaces, import-wallet JSON schemas, build-descriptor --spec JSON.
- OSS-Fuzz onboarding, coverage reports, `arbitrary`-based structured
  mutators — later ratchets.
- mc-codex32 shared-BCH extraction (DECISIONS D-13).

## Resolved decisions (round-1 R0 answers, adopted)

1. One cycle, hard per-repo sub-gates md → ms → mk; toolkit descoped. [Q1/C2]
2. CI = compile gate on push (fuzz/** + src paths) + cron daily smoke +
   workflow_dispatch; never run-fuzz on every push. [Q2]
3. fuzz/ own-workspace isolation verified; commit fuzz/Cargo.lock aligned
   with root [C3]; .gitignore already covers fuzz/target. [Q3]
4. No-leak window ≥8 chars, Display+Debug, exclusion set per [C1]. [Q4]
5. Re-encode oracles: md `encode_md1_string`, mk
   `encode_with_chunk_set_id(FIXED)`, ms `encode(tag,&payload)`;
   re-encode Err = finding. [Q5/I2/I6]
6. Toolkit hook = cfg(fuzzing) lib.rs mount + check-cfg lint +
   patch replication — deferred wholesale to the FOLLOWUP. [Q6/C2/C3]
7. Pinned nightly (`nightly-YYYY-MM-DD`) + quarterly-bump FOLLOWUP. [Q7/M5]

## FOLLOWUPs this cycle files (at implementation time)

- `ms-codec-error-display-echoes-input` (mnemonic-secret; companion in
  toolkit linking v0.53.4's friendly-mapper withholding) — [C1].
- `toolkit-descriptor-fuzz-target` (toolkit) — [C2]/[C3] full design
  (cfg(fuzzing) mount, check-cfg lint, patch replication, closure facts).
- `fuzz-nightly-quarterly-bump` (toolkit, constellation-wide) — [M5].
